//! A `MANIFOLD_SOLID_BREP` becomes a solid, or is refused by the `#id` that
//! stopped it.
//!
//! The walk is the one an AP203 file is laid out in — closed shell, advanced
//! face, face bound, edge loop, oriented edge, edge curve, vertex point — and it
//! goes straight into a [`SolidBuilder`], which is the public door §42.5 said
//! the kernel did not have. Nothing is written into the caller's model until
//! the whole description has been through `Model::validate`.

use std::collections::BTreeMap;

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, NEAREST_F64_POINT_ROUNDOFF, Orientation, Picker, Plane,
    Point, Sphere, Surface, Vector,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface};
use rcad_topo::{
    DraftCoedge, DraftEdge, DraftFace, DraftStep, DraftVertex, Invalid, Model, SolidBuilder,
    SolidKey,
};
use smallvec::smallvec;

use crate::geometry::Reading;
use crate::graph::EntityId;
use crate::refusal::Refusal;

/// Turn one `MANIFOLD_SOLID_BREP` into one solid in `model`.
pub(crate) fn solid_from(
    reading: &Reading<'_>,
    brep: EntityId,
    model: &mut Model,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SolidKey, Refusal> {
    let shell = reading.reference_at(brep, "MANIFOLD_SOLID_BREP", 1)?;
    let mut walk = Walk::new(brep);
    walk.describe_the_shell(reading, shell)?;
    walk.build(reading, model, policy, log)
}

/// Every refusal one `MANIFOLD_SOLID_BREP` would earn, rather than the first
/// one [`solid_from`] stops at: a face, a bound or an edge that is refused is
/// dropped and the walk carries on past whatever is left of the solid.
///
/// **This exists because a first refusal is an upper bound and nothing else**
/// (§45.2). A file freed of its first blocker usually meets a second one right
/// behind it, so the count of files one reason stops *first* says nothing about
/// what fixing that reason is worth. What prices a fix is the count of files
/// that have **only** that reason, and no walk that stops at the first one can
/// take it.
///
/// Two buckets, never mixed. [`ProbeOutcome::skipped`] is every refusal that
/// cost the walk one entity — the walk went on to whatever was left.
/// [`ProbeOutcome::fatal`] is a refusal that stopped this solid before it had a
/// face list to skip through at all: its `MANIFOLD_SOLID_BREP` does not name a
/// `CLOSED_SHELL` that resolves, or that shell's own face list does not.
///
/// **Nothing that skipped an entity is ever asked to assemble what is left of
/// it, at any of the three levels below.** A dropped edge leaves a wire that
/// cannot close; a dropped bound leaves a face with a hole in it; a dropped
/// face leaves a shell that is not closed — and `add_wire`, `add_face`,
/// `add_shell` and the final `build` all refuse exactly that, by name, every
/// time. That was measured rather than assumed: before
/// [`ProbeWalk::probe_the_bound`] and [`ProbeWalk::probe_the_face`] stopped
/// asking `add_wire`/`add_face` to assemble a known-incomplete wire or face,
/// this reported `an arc this reader could not name` as never once a file's
/// *only* reason — every file that had it also seemed to have a wire that would
/// not close, when what it actually had was the one open wire that arc itself
/// belongs to. `build` (and `add_wire`, `add_face`, `add_shell`) is asked only
/// of a wire, a face, a shell or a solid that lost nothing — which is exactly
/// when its refusal is the real one [`solid_from`] would give for it too, and
/// not an echo of a hole this walk itself put there.
pub(crate) fn probe_solid(
    reading: &Reading<'_>,
    brep: EntityId,
    model: &mut Model,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> ProbeOutcome {
    let shell = match reading.reference_at(brep, "MANIFOLD_SOLID_BREP", 1) {
        Ok(shell) => shell,
        Err(why) => {
            return ProbeOutcome {
                skipped: Vec::new(),
                fatal: vec![why],
            };
        }
    };
    let mut probe = ProbeWalk::new(brep);
    if let Err(why) = probe.probe_the_shell(reading, shell) {
        return ProbeOutcome {
            skipped: probe.refusals,
            fatal: vec![why],
        };
    }
    if probe.refusals.is_empty()
        && let Err(why) = probe.walk.build(reading, model, policy, log)
    {
        probe.refusals.push(why);
    }
    ProbeOutcome {
        skipped: probe.refusals,
        fatal: Vec::new(),
    }
}

/// What [`probe_solid`] found, in the two buckets its own doc comment names.
pub(crate) struct ProbeOutcome {
    /// A refusal that cost the walk one face, bound or edge — never the whole
    /// solid.
    pub(crate) skipped: Vec<Refusal>,
    /// A refusal that stopped the walk before it had anything left to skip
    /// through.
    pub(crate) fatal: Vec<Refusal>,
}

/// One solid being described, and what every draft entity in it was called in
/// the file.
///
/// The name maps are the point. `SolidBuilder` refuses by an index into its own
/// arrays, which tells an importer nothing; a file names its entities and the
/// refusal has to say which one (§42.10).
struct Walk {
    brep: EntityId,
    builder: SolidBuilder,
    vertices: BTreeMap<EntityId, DraftVertex>,
    edges: BTreeMap<EntityId, DraftEdge>,
    /// The closed circles described as two halves — see
    /// [`Walk::describe_the_halves_of_a_circle`].
    halves: BTreeMap<EntityId, [DraftEdge; 2]>,
    vertex_names: Vec<EntityId>,
    edge_names: Vec<EntityId>,
    coedge_names: Vec<EntityId>,
    wire_names: Vec<EntityId>,
    face_names: Vec<EntityId>,
    face_surfaces: Vec<Surface>,
    /// The surfaces either side of each closed edge, filled in before any face
    /// of the shell is described — see [`Walk::find_the_closed_edges`].
    closed_edges: BTreeMap<EntityId, Vec<Surface>>,
    /// The surfaces of the faces along every edge of the shell, filled in by
    /// the same pass — [`Walk::the_surface_that_names_the_curve`] reads it to tell a circle
    /// imprinted on a flat face from one a curved face places.
    surfaces_along_each_edge: BTreeMap<EntityId, Vec<Surface>>,
}

impl Walk {
    fn new(brep: EntityId) -> Self {
        Self {
            brep,
            builder: SolidBuilder::new(),
            vertices: BTreeMap::new(),
            edges: BTreeMap::new(),
            halves: BTreeMap::new(),
            vertex_names: Vec::new(),
            edge_names: Vec::new(),
            coedge_names: Vec::new(),
            wire_names: Vec::new(),
            face_names: Vec::new(),
            face_surfaces: Vec::new(),
            closed_edges: BTreeMap::new(),
            surfaces_along_each_edge: BTreeMap::new(),
        }
    }

    /// Every face of the `CLOSED_SHELL`, each with the way round the shell uses
    /// it.
    fn describe_the_shell(
        &mut self,
        reading: &Reading<'_>,
        shell: EntityId,
    ) -> Result<(), Refusal> {
        let faces = reading.references_at(shell, "CLOSED_SHELL", 1)?;
        self.find_the_closed_edges(reading, &faces);
        let mut skin: Vec<(DraftFace, Orientation)> = Vec::with_capacity(faces.len());
        for face in &faces {
            skin.push(self.describe_the_face(reading, *face)?);
        }
        self.builder
            .add_shell(&skin)
            .map_err(|why| self.refused(shell, why))
    }

    /// Record the surfaces either side of every closed edge of the shell,
    /// before any face of it is described.
    ///
    /// **A closed edge's [`Orientation`] cannot be read off one face, which is
    /// the whole reason this pass exists.** An edge with one corner has nothing
    /// in its ends to say which way round it goes, so what says instead is the
    /// *pair* of surfaces either side of it — and a face being described knows
    /// only its own. See [`Walk::which_way_round_a_closed_edge_is_stored`] for
    /// the rule this feeds, and `docs/decisions.md` §115 for why the file's own
    /// flags cannot answer it.
    ///
    /// A face this cannot read is passed over rather than refused. Describing it
    /// is about to refuse it by name, or — in a [`ProbeWalk`] — to record the
    /// reason and carry on, and a second copy of that decision here could only
    /// come to disagree with the first.
    fn find_the_closed_edges(&mut self, reading: &Reading<'_>, faces: &[EntityId]) {
        for face in faces {
            let Ok(keyword) =
                reading.keyword_of(self.brep, *face, &["ADVANCED_FACE", "FACE_SURFACE"])
            else {
                continue;
            };
            let Ok((surface, _)) = self.surface_of_the_face(reading, *face, keyword) else {
                continue;
            };
            let Ok(bounds) = reading.references_at(*face, keyword, 1) else {
                continue;
            };
            for bound in bounds {
                let Ok((loop_id, _)) = self.loop_of_bound(reading, *face, bound) else {
                    continue;
                };
                let Ok(uses) = reading.references_at(loop_id, "EDGE_LOOP", 1) else {
                    continue;
                };
                for use_of_an_edge in uses {
                    let Ok(edge_curve) = reading.reference_at(use_of_an_edge, "ORIENTED_EDGE", 3)
                    else {
                        continue;
                    };
                    self.surfaces_along_each_edge
                        .entry(edge_curve)
                        .or_default()
                        .push(surface);
                    if an_edge_with_one_corner(reading, edge_curve) {
                        self.closed_edges
                            .entry(edge_curve)
                            .or_default()
                            .push(surface);
                    }
                }
            }
        }
    }

    /// One `ADVANCED_FACE`: the surface it sits on, the wires that trim it, and
    /// which way its own normal points.
    ///
    /// **`same_sense` is false does not turn the loops round, and working out
    /// why is most of what makes a real file import.** A closed shell's loops
    /// are wound about the *face's* normal, so two faces sharing an edge always
    /// walk it opposite ways whatever either one's `same_sense` says. Reversing
    /// a loop for `same_sense` would make the two walk it the same way, which
    /// `SolidBuilder` refuses by name — and did, on 13 of the 323 KiCad files,
    /// before this was got right.
    ///
    /// What `same_sense = .F.` changes is where the *surface* points, and this
    /// is the same two-line table the fuse already keeps
    /// (`fuse/assemble.rs`, §31.3's ON split): a plane is stored already turned
    /// round and is then used forwards, because a plane welds "which surface"
    /// to "which way"; a curved surface has no turned-round form — a sphere is
    /// a sphere — so it is stored as it is and the shell says
    /// [`Orientation::Reversed`].
    fn describe_the_face(
        &mut self,
        reading: &Reading<'_>,
        face: EntityId,
    ) -> Result<(DraftFace, Orientation), Refusal> {
        let keyword = reading.keyword_of(self.brep, face, &["ADVANCED_FACE", "FACE_SURFACE"])?;
        let (surface, used) = self.surface_of_the_face(reading, face, keyword)?;

        let bounds = reading.references_at(face, keyword, 1)?;
        let outer_first = self.outer_bound_first(reading, face, &bounds, &surface)?;
        let mut wires = Vec::with_capacity(outer_first.len());
        for bound in outer_first {
            wires.push(self.describe_the_bound(reading, face, bound)?);
        }
        let drafted = self
            .builder
            .add_face(surface, &wires)
            .map_err(|why| self.refused(face, why))?;
        self.face_names.push(face);
        self.face_surfaces.push(surface);
        Ok((drafted, used))
    }

    /// The surface a face sits on as it is stored, and how the shell uses it.
    ///
    /// The two-line table is [`Walk::describe_the_face`]'s, and it is here
    /// because [`Walk::find_the_closed_edges`] needs the same answer a whole
    /// pass earlier — read twice off one spelling rather than written twice.
    fn surface_of_the_face(
        &self,
        reading: &Reading<'_>,
        face: EntityId,
        keyword: &'static str,
    ) -> Result<(Surface, Orientation), Refusal> {
        let sits_on = reading.reference_at(face, keyword, 2)?;
        let written = reading.surface_at(face, sits_on)?;
        let same_sense = reading.flag_at(face, keyword, 3)?;
        Ok(match (written.as_plane(), same_sense) {
            (_, true) => (written, Orientation::Forward),
            (Some(flat), false) => (Surface::Plane(flat.reversed()), Orientation::Forward),
            (None, false) => (written, Orientation::Reversed),
        })
    }

    /// The bounds of a face with the outer one first, which is the order
    /// [`SolidBuilder::add_face`] takes them in.
    ///
    /// A face with one bound has that one as its outer, whatever the file called
    /// it. A face with several says which is which in one of two ways, and this
    /// reads both rather than guessing:
    ///
    /// - a `FACE_OUTER_BOUND` among them says so by its own type name;
    /// - **and where every bound is a plain `FACE_BOUND`, the winding says so.**
    ///   ISO 10303-42 winds an outer bound the way a right hand turns about the
    ///   face's normal and a hole the other way, and `Model::validate` asks for
    ///   exactly that. So the outer bound is the one whose corners sweep a
    ///   positive area, and this is reading what the file wrote rather than
    ///   working out which loop encloses which. **312 of the 323 KiCad files
    ///   have such a face**, and 266 of them were refused for it before this.
    ///
    /// The sweep of the area here is plain `f64` and its sign is a **hint and
    /// not a decision** — which is what makes it allowed to be plain `f64`.
    /// `Model::validate` asks the same question of the same corners moments
    /// later with a certified predicate, so a hint that came out wrong is a
    /// refusal by name and never a model with its holes and its outline swapped.
    ///
    /// A curved face has no winding rule to read — the sweep asks none of one —
    /// so its bounds are taken in file order.
    fn outer_bound_first(
        &self,
        reading: &Reading<'_>,
        face: EntityId,
        bounds: &[EntityId],
        surface: &Surface,
    ) -> Result<Vec<EntityId>, Refusal> {
        if bounds.len() <= 1 {
            return Ok(bounds.to_vec());
        }
        let named = bounds.iter().position(|bound| {
            reading
                .file()
                .instance(*bound)
                .is_some_and(|held| held.part("FACE_OUTER_BOUND").is_some())
        });
        let outer = match (named, surface.as_plane()) {
            (Some(outer), _) => outer,
            (None, None) => return Ok(bounds.to_vec()),
            (None, Some(flat)) => {
                let mut sweeping_forwards = Vec::new();
                for (position, bound) in bounds.iter().enumerate() {
                    if self.area_swept_by(reading, face, *bound, flat)? > 0.0 {
                        sweeping_forwards.push(position);
                    }
                }
                match sweeping_forwards[..] {
                    [only] => only,
                    _ => {
                        return Err(Refusal::AFaceThatDoesNotSayWhichBoundIsTheOuterOne {
                            face,
                            bounds: bounds.len(),
                        });
                    }
                }
            }
        };
        let mut ordered = Vec::with_capacity(bounds.len());
        ordered.push(bounds[outer]);
        ordered.extend(
            bounds
                .iter()
                .enumerate()
                .filter_map(|(position, bound)| (position != outer).then_some(*bound)),
        );
        Ok(ordered)
    }

    /// Twice the area a bound sweeps round the plane's normal, in the plane's
    /// own two coordinates.
    ///
    /// Positive means it turns the way a right hand turns about the normal. The
    /// corners give a polygon and each arc gives what it bulges past its own
    /// chord — without which a bound made of two half-circles, whose corners are
    /// two points and so no polygon at all, sweeps exactly nothing.
    ///
    /// See [`Walk::outer_bound_first`] for why plain `f64` is enough here and is
    /// not enough anywhere else.
    fn area_swept_by(
        &self,
        reading: &Reading<'_>,
        face: EntityId,
        bound: EntityId,
        flat: &Plane,
    ) -> Result<f64, Refusal> {
        let (loop_id, walk_it_backwards) = self.loop_of_bound(reading, face, bound)?;
        let mut oriented = reading.references_at(loop_id, "EDGE_LOOP", 1)?;
        if walk_it_backwards {
            oriented.reverse();
        }
        let mut corners = Vec::with_capacity(oriented.len());
        let mut bulge = 0.0;
        for use_of_an_edge in oriented {
            let edge_curve = reading.reference_at(use_of_an_edge, "ORIENTED_EDGE", 3)?;
            let forwards =
                reading.flag_at(use_of_an_edge, "ORIENTED_EDGE", 4)? != walk_it_backwards;
            let ends = [
                reading.reference_at(edge_curve, "EDGE_CURVE", 1)?,
                reading.reference_at(edge_curve, "EDGE_CURVE", 2)?,
            ];
            corners.push(self.point_of_vertex(reading, ends[usize::from(!forwards)])?);
            bulge += self.area_an_arc_bulges_past_its_chord(reading, edge_curve, forwards, flat)?;
        }
        let mut twice_the_area = 2.0 * bulge;
        for step in 0..corners.len() {
            let (from, to) = (
                flat.plane_coordinates_of(corners[step]),
                flat.plane_coordinates_of(corners[(step + 1) % corners.len()]),
            );
            twice_the_area += from.0 * to.1 - to.0 * from.1;
        }
        Ok(twice_the_area)
    }

    /// How much area an arc puts between itself and its own chord, signed by
    /// which way round the plane's normal the wire walks it.
    ///
    /// A circular arc of radius `r` spanning `φ` bulges past its chord by
    /// `r²(φ − sin φ)/2`, which is positive for every `φ` in `(0, 2π]` — a whole
    /// circle is `πr²` and is not a special case. What has to be got right is
    /// *which* `φ`: two ends on a closed curve leave two arcs, and
    /// `EDGE_CURVE.same_sense` says which of them this edge is, so the other is
    /// `2π` minus it.
    ///
    /// An ellipse's `φ` here is the angle its two ends subtend rather than its
    /// own parameter, which is the wrong angle by however far the ellipse is
    /// from round. It stays because this is a hint: the number is never
    /// compared against anything but zero, and an ellipse's bulge does not
    /// change sign for being measured at the wrong angle.
    ///
    /// Zero for a straight and for a curve kind this reader does not name, which
    /// leaves the corners' own polygon as the whole of the hint.
    fn area_an_arc_bulges_past_its_chord(
        &self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
        forwards: bool,
        flat: &Plane,
    ) -> Result<f64, Refusal> {
        let curve = reading.reference_at(edge_curve, "EDGE_CURVE", 3)?;
        let Some(held) = reading.file().instance(curve) else {
            return Ok(0.0);
        };
        let (keyword, half_the_whole) = match held.parts()[0].keyword() {
            "CIRCLE" => {
                let radius = reading.real_at(curve, "CIRCLE", 2)?;
                ("CIRCLE", radius * radius / 2.0)
            }
            "ELLIPSE" => {
                let across = reading.real_at(curve, "ELLIPSE", 2)?;
                let along = reading.real_at(curve, "ELLIPSE", 3)?;
                ("ELLIPSE", across * along / 2.0)
            }
            _ => return Ok(0.0),
        };
        let position = reading.reference_at(curve, keyword, 1)?;
        let place = reading.placement_at(curve, position)?;
        let facing = place.axis.dot(flat.normal());
        if facing == 0.0 {
            return Ok(0.0);
        }
        let ends = [
            reading.reference_at(edge_curve, "EDGE_CURVE", 1)?,
            reading.reference_at(edge_curve, "EDGE_CURVE", 2)?,
        ];
        let along_the_curve = reading.flag_at(edge_curve, "EDGE_CURVE", 4)?;
        let span = self.how_far_round_the_arc_goes(reading, &ends, &place, along_the_curve)?;
        let bulge = half_the_whole * (span - span.sin());
        Ok(if (along_the_curve == forwards) == (facing > 0.0) {
            bulge
        } else {
            -bulge
        })
    }

    /// The angle an edge's own arc turns through, in `(0, 2π]`.
    fn how_far_round_the_arc_goes(
        &self,
        reading: &Reading<'_>,
        ends: &[EntityId; 2],
        place: &crate::geometry::Placement,
        along_the_curve: bool,
    ) -> Result<f64, Refusal> {
        if ends[0] == ends[1] {
            return Ok(core::f64::consts::TAU);
        }
        let from = self.point_of_vertex(reading, ends[0])? - place.origin;
        let to = self.point_of_vertex(reading, ends[1])? - place.origin;
        let turned = from
            .cross(to)
            .dot(place.axis)
            .atan2(from.dot(to) * place.axis.dot(place.axis).sqrt());
        let turned = if turned <= 0.0 {
            turned + core::f64::consts::TAU
        } else {
            turned
        };
        Ok(if along_the_curve {
            turned
        } else {
            core::f64::consts::TAU - turned
        })
    }

    /// The `EDGE_LOOP` a bound holds, and whether the bound walks it backwards.
    fn loop_of_bound(
        &self,
        reading: &Reading<'_>,
        face: EntityId,
        bound: EntityId,
    ) -> Result<(EntityId, bool), Refusal> {
        let keyword = reading.keyword_of(face, bound, &["FACE_OUTER_BOUND", "FACE_BOUND"])?;
        let loop_id = reading.reference_at(bound, keyword, 1)?;
        let held = reading
            .file()
            .instance(loop_id)
            .ok_or(Refusal::AReferencePointsNowhere {
                from: bound,
                to: loop_id,
            })?;
        if held.part("EDGE_LOOP").is_none() {
            return Err(Refusal::ALoopThisReaderDoesNotWalk {
                face,
                id: loop_id,
                keyword: held.parts()[0].keyword().to_owned(),
            });
        }
        Ok((loop_id, !reading.flag_at(bound, keyword, 2)?))
    }

    /// One `FACE_BOUND` or `FACE_OUTER_BOUND`, as a wire.
    fn describe_the_bound(
        &mut self,
        reading: &Reading<'_>,
        face: EntityId,
        bound: EntityId,
    ) -> Result<rcad_topo::DraftWire, Refusal> {
        let (loop_id, walk_it_backwards) = self.loop_of_bound(reading, face, bound)?;
        let mut oriented = reading.references_at(loop_id, "EDGE_LOOP", 1)?;
        if walk_it_backwards {
            oriented.reverse();
        }
        let mut steps = Vec::with_capacity(oriented.len());
        for use_of_an_edge in oriented {
            let coedges = self.describe_the_coedge(reading, use_of_an_edge, walk_it_backwards)?;
            steps.extend(coedges.into_iter().map(DraftStep::Along));
        }
        let wire = self
            .builder
            .add_wire(&steps)
            .map_err(|why| self.refused(loop_id, why))?;
        self.wire_names.push(loop_id);
        Ok(wire)
    }

    /// One `ORIENTED_EDGE`, as one use of one edge running one way round.
    ///
    /// **[`Orientation`] means two different things and the file answers only
    /// one of them.** On an edge with two corners it means *from the start
    /// corner towards the end one*, and `ORIENTED_EDGE.orientation` against the
    /// bound's own says exactly that. On a **closed** edge there are no two
    /// corners to run between, and this kernel gives the word its other meaning
    /// — the way the curved surface either side of the edge winds — which the
    /// file never wrote down. [`Walk::which_way_round_a_closed_edge_is_stored`]
    /// works it out; `docs/decisions.md` §115.
    ///
    /// **A closed circle between a ball and another curved face is two uses of
    /// two halves**, in the order the loop meets them — see
    /// [`Walk::describe_the_halves_of_a_circle`]. Each half has two corners, so
    /// the file's own flag says which way it is walked, as it does for any edge
    /// with two.
    fn describe_the_coedge(
        &mut self,
        reading: &Reading<'_>,
        oriented: EntityId,
        walk_it_backwards: bool,
    ) -> Result<Vec<DraftCoedge>, Refusal> {
        let edge_curve = reading.reference_at(oriented, "ORIENTED_EDGE", 3)?;
        let along_the_loop = reading.flag_at(oriented, "ORIENTED_EDGE", 4)? != walk_it_backwards;
        if let Some([there, back]) = self.describe_the_halves_of_a_circle(reading, edge_curve)? {
            let uses = if along_the_loop {
                [(there, Orientation::Forward), (back, Orientation::Forward)]
            } else {
                [
                    (back, Orientation::Reversed),
                    (there, Orientation::Reversed),
                ]
            };
            let mut coedges = Vec::with_capacity(uses.len());
            for (half, way) in uses {
                let coedge = self
                    .builder
                    .add_coedge(half, way)
                    .map_err(|why| self.refused(oriented, why))?;
                self.coedge_names.push(oriented);
                coedges.push(coedge);
            }
            return Ok(coedges);
        }
        let edge = self.describe_the_edge(reading, edge_curve)?;
        let forwards = self
            .which_way_round_a_closed_edge_is_stored(reading, edge_curve, along_the_loop)?
            .unwrap_or(along_the_loop);
        let coedge = self
            .builder
            .add_coedge(
                edge,
                if forwards {
                    Orientation::Forward
                } else {
                    Orientation::Reversed
                },
            )
            .map_err(|why| self.refused(oriented, why))?;
        self.coedge_names.push(oriented);
        Ok(vec![coedge])
    }

    /// The two halves a closed circle is described as, or `None` for an edge
    /// that is read as it was written.
    ///
    /// Two kinds of closed circle are cut: one between a ball and another
    /// curved face, and one whose corner is a point where the curved face
    /// beside it has no way round — a ball's pole, where
    /// [`Surface::the_way_its_first_coordinate_runs_at`] answers `None`.
    ///
    /// **A closed edge at a pole has no readable direction, and two corners
    /// give it one.** A closed edge is stored running the way its curved face
    /// winds at the corner ([`Walk::which_way_round_a_closed_edge_is_stored`]),
    /// and `rcad-topo`'s mesher reads the same rule back. At a pole every
    /// longitude runs through the one point, so neither side has an answer and
    /// the mesher walks the rim in whatever order it came: backwards, for the
    /// KiCad LED dome closed by a flat disc, refused by
    /// `AWireWillNotLieFlatOnItsFace` (§193.3). Cut in two, each half runs from
    /// one corner to the other and the file's own flag says which way, as for
    /// any edge with two — and the same file written with the cut already in it
    /// imports and measures half a ball.
    ///
    /// **Only where the direction is missing, not on every ball-and-plane
    /// rim.** A rim whose corner is off the poles already has a direction, and
    /// cutting it would move a corner the file did not write onto a face that
    /// imports today. The halves here are what the reader makes of the same
    /// circle written as two arcs, which is read with no check that the circle
    /// lies on its faces, so this cut asks for none either; the check below is
    /// the ball-and-curved-face cut's own (§189.3).
    ///
    /// **A closed edge has no room in the model for the surface that names its
    /// circle.** [`EdgeRun::RightRoundWhereItsFacesCross`] carries nothing, and
    /// `rcad-topo` reads that kind and no other as closed. So the run is named
    /// the way an arc already is — by the circle's plane and a half-space
    /// standing on its chord — and a circle with one corner has no chord. The
    /// second corner is the opposite point, `c − (A − c)`, which lies on the
    /// circle's plane and on every face the circle was checked against to
    /// within the rounding of one subtraction. The two pieces run the way the
    /// edge does, `A` to the second corner and back.
    ///
    /// **Why the opposite point is measured, not argued.** Every KiCad LED
    /// dome puts its file corner on the ball's pole, and its equator is a
    /// circle through both poles. The opposite point is then the other pole,
    /// so the mesher meets both as corners. A quarter turn left the second
    /// pole in the middle of a piece, where it is a walked point or not
    /// depending on how finely the piece is walked, and a pole passed between
    /// two walked points has no corner to open a pole side at: 18
    /// `LED_D*_Horizontal` files were refused on the dome by
    /// `AWireWillNotLieFlatOnItsFace` that way. The quarter turn was chosen
    /// when the mesher took a pole side at a corner the short way round, which
    /// at exactly half a turn went the wrong way; `rcad-topo`'s
    /// `turn_at_a_corner_on_a_pole` is that fix.
    ///
    /// This is the one place the reader describes an edge the file wrote as
    /// one as two. Between a ball and a curved face it is limited to the circle
    /// [`Walk::every_face_holds_the_curve`] passes, where the two only touch and
    /// nothing but the file's circle draws where (`docs/decisions.md` §189.3).
    fn describe_the_halves_of_a_circle(
        &mut self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
    ) -> Result<Option<[DraftEdge; 2]>, Refusal> {
        if let Some(&halves) = self.halves.get(&edge_curve) {
            return Ok(Some(halves));
        }
        if !an_edge_with_one_corner(reading, edge_curve) {
            return Ok(None);
        }
        let corner = reading.reference_at(edge_curve, "EDGE_CURVE", 1)?;
        let near = self.point_of_vertex(reading, corner)?;
        let a_ball_and_a_curved_face = self
            .surfaces_along_each_edge
            .get(&edge_curve)
            .is_some_and(|faces| a_ball_where_curved_faces_meet(faces).is_some());
        let no_way_round_at_its_corner = self
            .closed_edges
            .get(&edge_curve)
            .and_then(|carried_by| the_curved_one_of(carried_by))
            .is_some_and(|curved| curved.the_way_its_first_coordinate_runs_at(near).is_none());
        if !a_ball_and_a_curved_face && !no_way_round_at_its_corner {
            return Ok(None);
        }
        let curve = reading.reference_at(edge_curve, "EDGE_CURVE", 3)?;
        let Some(held) = reading.file().instance(curve) else {
            return Ok(None);
        };
        if held.parts()[0].keyword() != "CIRCLE" {
            return Ok(None);
        }
        let position = reading.reference_at(curve, "CIRCLE", 1)?;
        let place = reading.placement_at(curve, position)?;
        if a_ball_and_a_curved_face
            && !self.every_face_holds_the_curve(reading, edge_curve, curve, "CIRCLE", &place)?
        {
            return Ok(None);
        }
        let along_the_curve = reading.flag_at(edge_curve, "EDGE_CURVE", 4)?;
        if place.axis.normalized().is_err() {
            return Ok(None);
        }
        let far = place.origin - (near - place.origin);
        let and = Surface::Plane(
            Plane::from_point_and_normal(place.origin, place.axis).map_err(|why| {
                Refusal::TheGeometryIsNotAWorkingShape {
                    id: edge_curve,
                    why,
                }
            })?,
        );
        let there = self.the_arc_between(edge_curve, &place, and, [near, far], along_the_curve)?;
        let back = self.the_arc_between(edge_curve, &place, and, [far, near], along_the_curve)?;

        let near = self.describe_the_vertex(reading, corner)?;
        let far = self
            .builder
            .add_vertex(far)
            .map_err(|why| self.refused(edge_curve, why))?;
        self.vertex_names.push(edge_curve);
        let mut halves = Vec::with_capacity(2);
        for (from, to, run) in [(near, far, there), (far, near, back)] {
            let half = self
                .builder
                .add_edge(from, to, run)
                .map_err(|why| self.refused(edge_curve, why))?;
            self.edge_names.push(edge_curve);
            halves.push(half);
        }
        let halves = [halves[0], halves[1]];
        self.halves.insert(edge_curve, halves);
        Ok(Some(halves))
    }

    /// Which way round a closed edge this use of it goes, or `None` for an edge
    /// whose two corners already say.
    ///
    /// **The rule is `rcad-topo`'s and this only reads it.** A closed edge is
    /// stored running the way the curved surface either side of it winds, which
    /// is what [`Surface::the_way_its_first_coordinate_runs_at`] gives;
    /// `Model::validate` reads the same rule back with
    /// `a_closed_crossing_is_walked_the_way_it_is_stored` and refuses by
    /// `AClosedEdgeIsWalkedAgainstItsSurface` when a coedge disagrees. So
    /// [`Orientation::Forward`] on a closed edge is this use of it travelling
    /// that way and nothing else — `ORIENTED_EDGE.orientation` answers a
    /// different question and agreeing with this one is luck (§115).
    ///
    /// Which of the two surfaces names the direction is [`the_curved_one_of`],
    /// and it has to be the one `rcad-topo` picks or the file is walked against
    /// the very rule it is read by.
    ///
    /// **The sign is a hint and not a decision, which is what makes plain `f64`
    /// allowed here** — the same standing [`Walk::area_swept_by`]'s sweep has.
    /// `Model::validate` asks the certified question of this edge moments
    /// later, so a hint that came out wrong is a refusal by name and never a
    /// solid with a seam running backwards.
    ///
    /// **A travel that dots to exactly zero against the winding is no answer,
    /// and it says so.** `which_way_a_closed_crossing_turns` reads the surface's
    /// latitude there, and this has no gradient to build one from — so the hint
    /// goes back to the file's own flag rather than picking a side. Picking one
    /// is worse than useless: the two uses of a closed edge travel exact
    /// opposite ways, so a tie-break that answers the same for both is
    /// *provably* wrong for one of them, and `SolidBuilder` refuses the whole
    /// solid by `ADraftEdgeIsWalkedTheSameWayTwice`. That is where 26
    /// of the corpus's files went, every one of them stopped by this and
    /// nothing else: a bend of pipe capped by a plane, whose cap circle runs
    /// round the *ring* while [`Surface::the_way_its_first_coordinate_runs_at`]
    /// on a bend runs round the **tube**, square to it at every corner.
    ///
    /// `ORIENTED_EDGE.orientation` answers a different question (§115), so
    /// falling back to it is luck — but it is luck that at least tells the two
    /// uses apart, and `Model::validate` asks the certified question moments
    /// later and refuses by `AClosedEdgeIsWalkedAgainstItsSurface` when the luck
    /// runs out.
    fn which_way_round_a_closed_edge_is_stored(
        &self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
        along_the_loop: bool,
    ) -> Result<Option<bool>, Refusal> {
        if !an_edge_with_one_corner(reading, edge_curve) {
            return Ok(None);
        }
        let Some(carried_by) = self.closed_edges.get(&edge_curve) else {
            return Ok(None);
        };
        let at = reading.reference_at(edge_curve, "EDGE_CURVE", 1)?;
        let corner = self.point_of_vertex(reading, at)?;
        let Some(winding) = the_curved_one_of(carried_by)
            .and_then(|curved| curved.the_way_its_first_coordinate_runs_at(corner))
        else {
            return Ok(None);
        };
        let Some(travel) =
            self.which_way_a_closed_edge_is_travelled(reading, edge_curve, corner, along_the_loop)?
        else {
            return Ok(None);
        };
        let leaning = travel.dot(winding);
        Ok(if leaning > 0.0 {
            Some(true)
        } else if leaning < 0.0 {
            Some(false)
        } else {
            None
        })
    }

    /// Which way this use of a closed edge travels in space, at its one corner.
    ///
    /// The `CIRCLE` or `ELLIPSE` under the edge turns the way a right hand turns
    /// about its own axis, so `w × (corner − centre)` is where its parameter
    /// goes; `EDGE_CURVE.same_sense` and the way the loop uses the edge each
    /// turn that round.
    ///
    /// **An ellipse's tangent is not exactly that, and the difference cannot
    /// change which way round it goes.** The true tangent `(−a sin t, b cos t)`
    /// dots `w × (corner − centre)` to `ab`, which is positive whatever `t` is,
    /// so the two never point opposite ways round the axis. The caller reads a
    /// sign and not an angle, which is the standing an ellipse already has in
    /// [`Walk::area_an_arc_bulges_past_its_chord`].
    ///
    /// `None` for a curve kind that is neither, which [`Walk::run_along`] has
    /// already refused this edge by name for.
    fn which_way_a_closed_edge_is_travelled(
        &self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
        corner: Point,
        along_the_loop: bool,
    ) -> Result<Option<Vector>, Refusal> {
        let curve = reading.reference_at(edge_curve, "EDGE_CURVE", 3)?;
        let Some(held) = reading.file().instance(curve) else {
            return Ok(None);
        };
        let keyword = match held.parts()[0].keyword() {
            "CIRCLE" => "CIRCLE",
            "ELLIPSE" => "ELLIPSE",
            _ => return Ok(None),
        };
        let position = reading.reference_at(curve, keyword, 1)?;
        let place = reading.placement_at(curve, position)?;
        let round = place.axis.cross(corner - place.origin);
        let along_the_curve = reading.flag_at(edge_curve, "EDGE_CURVE", 4)?;
        Ok(Some(if along_the_loop == along_the_curve {
            round
        } else {
            -round
        }))
    }

    /// One `EDGE_CURVE`, described once however many faces walk it.
    fn describe_the_edge(
        &mut self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
    ) -> Result<DraftEdge, Refusal> {
        if let Some(&already) = self.edges.get(&edge_curve) {
            return Ok(already);
        }
        let starts_at = reading.reference_at(edge_curve, "EDGE_CURVE", 1)?;
        let ends_at = reading.reference_at(edge_curve, "EDGE_CURVE", 2)?;
        let curve = reading.reference_at(edge_curve, "EDGE_CURVE", 3)?;
        let along_the_curve = reading.flag_at(edge_curve, "EDGE_CURVE", 4)?;

        let start = self.describe_the_vertex(reading, starts_at)?;
        let end = self.describe_the_vertex(reading, ends_at)?;
        let run = self.run_along(
            reading,
            edge_curve,
            curve,
            [starts_at, ends_at],
            along_the_curve,
        )?;
        let edge = self
            .builder
            .add_edge(start, end, run)
            .map_err(|why| self.refused(edge_curve, why))?;
        self.edge_names.push(edge_curve);
        self.edges.insert(edge_curve, edge);
        Ok(edge)
    }

    /// What an edge runs along, from the curve the file put under it.
    ///
    /// Three shapes and everything else refused by the name the file gave it. A
    /// straight line is [`EdgeRun::Straight`]; a circle or an ellipse the edge
    /// goes right round is [`EdgeRun::RightRoundWhereItsFacesCross`] where its
    /// two faces cross and [`EdgeRun::RightRoundWhereItCrosses`] where they are
    /// the one surface and name nothing; anything less than right round is an
    /// arc, and an arc needs saying *which* arc — see
    /// [`Walk::the_arc_between`].
    fn run_along(
        &mut self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
        curve: EntityId,
        ends: [EntityId; 2],
        along_the_curve: bool,
    ) -> Result<EdgeRun, Refusal> {
        let held = reading
            .file()
            .instance(curve)
            .ok_or(Refusal::AReferencePointsNowhere {
                from: edge_curve,
                to: curve,
            })?;
        let keyword = held.parts()[0].keyword();
        match keyword {
            "LINE" => Ok(EdgeRun::Straight),
            "CIRCLE" => self.round_run(reading, edge_curve, curve, "CIRCLE", ends, along_the_curve),
            "ELLIPSE" => {
                self.round_run(reading, edge_curve, curve, "ELLIPSE", ends, along_the_curve)
            }
            other => Err(Refusal::ACurveThisKernelDoesNotHold {
                edge: edge_curve,
                curve,
                keyword: other.to_owned(),
            }),
        }
    }

    /// A closed curve the edge either goes right round or takes one arc of.
    ///
    /// **Right round between two faces that are the one surface names
    /// nothing**, and that is why the closed case asks rather than answering at
    /// once. A circle where two halves of one pipe meet, or one a file
    /// imprinted on a flat face, has the same surface on both sides, so a
    /// reader asked where the edge runs gets that surface crossed with itself:
    /// no curve at all where both are flat, and — for a ball written twice a
    /// hair apart — a great circle in the wrong place
    /// (`docs/decisions.md` §218.6). Such a run names its second surface the way
    /// an arc's already does, and [`EdgeRun::RightRoundWhereItCrosses`] is the
    /// kind that carries one (§216).
    ///
    /// **What counts as naming nothing is
    /// [`the_faces_along_it_name_no_crossing`], and it is exact both ways.**
    /// Two flat faces never name a closed curve whatever their numbers, which is
    /// a fact about planes and not a tolerance; anything else has to be the one
    /// surface bit for bit. A nearness would have to name a tolerance this
    /// reader has no principle for, and it would risk the mistake that costs —
    /// an `and` whose curve is not where the file's is (§216, §16.10).
    ///
    /// **Between curved faces that are not one surface bit for bit, the run is
    /// named the way an arc's is**: by the curve's own plane where the file's
    /// curve has been checked to lie on every face, and by the faces
    /// ([`EdgeRun::RightRoundWhereItsFacesCross`]) where it has not — see
    /// [`Walk::the_surface_that_names_the_curve`]. That covers one pipe, cone or
    /// torus written twice a hair apart, whose two spellings cross nowhere near
    /// the rim, with no tolerance on the surfaces: the check is on the file's
    /// curve, and the plane crosses either face in exactly that curve (§220.4
    /// for arcs). It is also right for two genuinely different curved faces
    /// that hold one circle — a ball and a pipe of its radius on its axis, which
    /// only touch round it (§189.3), a ball resting in a cone, a pipe round a
    /// torus's outer rim, or two balls that cross in it: the plane crosses each
    /// of them in the one circle, so `rcad_geom::walk`'s `[face, and]` draws
    /// the same curve the two faces would, and draws it where they only touch.
    ///
    /// **Everywhere else the faces keep
    /// [`EdgeRun::RightRoundWhereItsFacesCross`]**: a flat face and a curved
    /// one cross in the curve and want nothing from this reader, and a curve
    /// [`Walk::every_face_holds_the_curve`] has no bound for is left to them.
    fn round_run(
        &self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
        curve: EntityId,
        keyword: &'static str,
        ends: [EntityId; 2],
        along_the_curve: bool,
    ) -> Result<EdgeRun, Refusal> {
        let right_round = ends[0] == ends[1];
        let faces_along_it = self.surfaces_along_each_edge.get(&edge_curve);
        let they_name_no_crossing =
            faces_along_it.is_some_and(|faces| the_faces_along_it_name_no_crossing(faces));
        let curved_faces_meet =
            faces_along_it.is_some_and(|faces| a_face_where_curved_faces_meet(faces).is_some());
        if right_round && !they_name_no_crossing && !curved_faces_meet {
            return Ok(EdgeRun::RightRoundWhereItsFacesCross);
        }
        let position = reading.reference_at(curve, keyword, 1)?;
        let place = reading.placement_at(curve, position)?;
        if right_round
            && curved_faces_meet
            && !self.every_face_holds_the_curve(reading, edge_curve, curve, keyword, &place)?
        {
            return Ok(EdgeRun::RightRoundWhereItsFacesCross);
        }
        let and =
            self.the_surface_that_names_the_curve(reading, edge_curve, curve, keyword, &place)?;
        if right_round {
            return Ok(EdgeRun::RightRoundWhereItCrosses { and });
        }
        let corners = [
            self.point_of_vertex(reading, ends[0])?,
            self.point_of_vertex(reading, ends[1])?,
        ];
        self.the_arc_between(edge_curve, &place, and, corners, along_the_curve)
    }

    /// The surface a run names beside the faces along it: the curve's own
    /// plane, or — for a circle every face along which is flat — the cylinder
    /// of the circle's radius about its axis.
    ///
    /// An arc always needs one; a closed run needs one exactly where the faces
    /// either side are the one surface — see [`Walk::round_run`].
    ///
    /// **The plane names nothing when the faces are flat too.** A circle
    /// imprinted on a flat face has that face's plane on both sides of it, and
    /// its own plane is the same plane again, so the run said "where this plane
    /// crosses itself" and the circle's centre and radius never reached the
    /// model (`docs/decisions.md` §176.2). The cylinder crosses the face in
    /// exactly the circle, so `rcad_geom::walk` has a curve to walk round.
    ///
    /// **An `ELLIPSE` keeps its plane**, and on flat faces is then refused by
    /// the mesher as an edge that bends with nothing naming it. No single
    /// surface this kernel holds crosses a plane in a given ellipse and is
    /// written down from the ellipse alone without choosing a slope for it — a
    /// tilted cylinder would do, and which tilt is a choice this reader has no
    /// consumer to make it for (§13).
    ///
    /// Where a flat face and a curved one are along the edge the plane stays
    /// too: the walk reads the faces there. Where every face is the one surface
    /// bit for bit — a seam — the plane stays as well, and is the second surface
    /// the walk reads; a cylinder would change what a sphere's seam names from
    /// a great circle to a tangency.
    ///
    /// **Between two curved faces the walk reads the plane**, so the plane is
    /// written only where the file's curve has been checked to lie on every
    /// face along it — [`Walk::every_face_holds_the_curve`]. The walk has no way
    /// to check the plane itself, so the reader does. A ball and a pipe that
    /// only touch round the circle leave the marcher nothing to follow
    /// (`docs/decisions.md` §189.3). A pipe written twice a hair apart names two
    /// nearly-coincident pipes, which cross nowhere near the rim (§220.4). In
    /// both the plane crosses a face in the file's curve, and `rcad_geom::walk`
    /// draws it from that pair.
    ///
    /// **Where the check fails the run names one of its own faces instead** —
    /// the ball if there is one, since the ball is the face the walk pairs with
    /// the plane — so the walk reads the two faces exactly as it did before the
    /// plane was read at all. That is every curve the check has no bound for:
    /// an `ELLIPSE` on anything but a pipe, and a circle on a torus that is
    /// neither square to its axis nor round its tube. A plane crosses a torus
    /// in two circles and a slanted pipe in an ellipse, so an unchecked plane
    /// written there would hand the walk a curve the file never drew. A plane
    /// beside two curved faces therefore always means a curve every face holds.
    fn the_surface_that_names_the_curve(
        &self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
        curve: EntityId,
        keyword: &'static str,
        place: &crate::geometry::Placement,
    ) -> Result<Surface, Refusal> {
        let not_a_working_shape = |why| Refusal::TheGeometryIsNotAWorkingShape {
            id: edge_curve,
            why,
        };
        let faces_along_it = self.surfaces_along_each_edge.get(&edge_curve);
        if let Some(face) = faces_along_it.and_then(|faces| a_face_where_curved_faces_meet(faces))
            && !self.every_face_holds_the_curve(reading, edge_curve, curve, keyword, place)?
        {
            return Ok(face);
        }
        let only_flat_faces_along_it =
            faces_along_it.is_some_and(|faces| faces.iter().all(|face| face.as_plane().is_some()));
        if keyword == "CIRCLE" && only_flat_faces_along_it {
            let radius = reading.real_at(curve, "CIRCLE", 2)?;
            let round = Axis::new(place.origin, place.axis)
                .and_then(|axis| Cylinder::new(axis, radius))
                .map_err(not_a_working_shape)?;
            return Ok(Surface::Cylinder(round));
        }
        let flat =
            Plane::from_point_and_normal(place.origin, place.axis).map_err(not_a_working_shape)?;
        Ok(Surface::Plane(flat))
    }

    /// Whether the file's `CIRCLE` or `ELLIPSE` under this edge lies on the
    /// surface of every face along it, to within what writing the numbers down
    /// costs.
    ///
    /// **This is what lets the reader name a run by the file's curve rather
    /// than by its faces, and the file's word is not taken for it.** Each face's
    /// surface is asked in closed form, against the curve's centre, the normal
    /// of the plane it lies in, and its size — see [`a_surface_holds_the_circle`]
    /// and [`a_surface_holds_the_ellipse`] for the bounds. A surface this does
    /// not have a bound for holds nothing, so the run keeps being named by its
    /// faces there.
    ///
    /// **The normal is the one the plane this run would name is stored with**,
    /// not a second rounding of the file's direction. The walk tells a torus's
    /// two families of flat crossings apart by whether that stored normal
    /// crosses the torus's axis to exactly nothing, so the check has to ask the
    /// same vector the walk will.
    ///
    /// The allowance is the one `Model::validate` gives a corner of a curved
    /// face: sixteen times the rounding of writing the largest number involved
    /// down. It is not a modelling tolerance. On the KiCad LED body it passes a
    /// dome whose centre is the circle's own and a body whose axis the centre
    /// misses by `2.1e-16`, and it refuses a circle a thousandth of a unit off.
    ///
    /// An `ELLIPSE` whose placement leaves out its reference direction holds
    /// nothing: which way its long axis runs is then a default the schema
    /// chooses, and nothing here reads that default.
    fn every_face_holds_the_curve(
        &self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
        curve: EntityId,
        keyword: &'static str,
        place: &crate::geometry::Placement,
    ) -> Result<bool, Refusal> {
        let Some(faces) = self.surfaces_along_each_edge.get(&edge_curve) else {
            return Ok(false);
        };
        let Ok(its_plane) = Plane::from_point_and_normal(place.origin, place.axis) else {
            return Ok(false);
        };
        let normal = its_plane.normal();
        match keyword {
            "CIRCLE" => {
                let radius = reading.real_at(curve, "CIRCLE", 2)?;
                Ok(faces
                    .iter()
                    .all(|face| a_surface_holds_the_circle(face, place.origin, normal, radius)))
            }
            "ELLIPSE" => {
                let Some(reference) = place.reference else {
                    return Ok(false);
                };
                let Ok(first_way) = (reference - normal * reference.dot(normal)).normalized()
                else {
                    return Ok(false);
                };
                let ellipse = Ellipse {
                    centre: place.origin,
                    first_way,
                    second_way: normal.cross(first_way),
                    semi_axes: [
                        reading.real_at(curve, "ELLIPSE", 2)?,
                        reading.real_at(curve, "ELLIPSE", 3)?,
                    ],
                };
                Ok(faces
                    .iter()
                    .all(|face| a_surface_holds_the_ellipse(face, &ellipse)))
            }
            other => unreachable!("round_run is only handed a CIRCLE or an ELLIPSE, not {other}"),
        }
    }

    /// The arc of a circle or an ellipse that runs from one corner to the other.
    ///
    /// Two ends on a closed curve name **two** arcs, so a run needs a half-space
    /// to say which — the same reason a ball's seam needs one (see
    /// [`EdgeRun::PartOfWhereItCrosses`]). The half-space stands on the chord,
    /// and which way it faces is settled without computing a single point on the
    /// arc:
    ///
    /// > Writing `w` for the axis the curve turns about and `A`, `B` for the two
    /// > ends, every point of the arc that runs from `A` to `B` the way the
    /// > parameter increases is **strictly** on the positive side of
    /// > `(B − A) × w`.
    ///
    /// Take the centre at the origin, `w = ẑ` and a unit circle. With `A` at
    /// angle `0` and `B` at angle `s ∈ (0, 2π)`, the point at angle `t` gives
    /// `((B − A) × w)·(X − A) = 4·sin(s/2)·sin(t/2)·sin((s − t)/2)`, and all
    /// three factors are positive for `t ∈ (0, s)`. An ellipse is that circle
    /// under a map that scales two directions of the plane by positive numbers,
    /// which takes chords to chords and keeps which side of one a point is on,
    /// so the same sign holds. `the_far_side_of_a_chord_is_where_the_arc_is` is
    /// the test.
    ///
    /// When `EDGE_CURVE.same_sense` is false the edge runs the other way round
    /// the curve, which is the other arc, and the half-space is reversed.
    fn the_arc_between(
        &self,
        edge_curve: EntityId,
        place: &crate::geometry::Placement,
        and: Surface,
        corners: [Point; 2],
        along_the_curve: bool,
    ) -> Result<EdgeRun, Refusal> {
        if corners[0] == corners[1] {
            return Err(Refusal::AnArcThatStartsAndEndsInTheSamePlace { edge: edge_curve });
        }
        let across = (corners[1] - corners[0]).cross(place.axis);
        let facing = if along_the_curve { across } else { -across };
        let picking = a_half_space_holding_both(corners, facing)
            .ok_or(Refusal::AnArcThisReaderCouldNotName { edge: edge_curve })?;
        Ok(EdgeRun::PartOfWhereItCrosses {
            and,
            picked_by: smallvec![Picker::from(picking)],
        })
    }

    /// One `VERTEX_POINT`, described once however many edges end at it.
    ///
    /// **Two `VERTEX_POINT`s at the same coordinates stay two corners.** Merging
    /// them is the first item on roadmap §9.2's healing list and M5 does none of
    /// it (§42.9); what a file that writes one corner twice gets is a wire that
    /// does not close, by name.
    fn describe_the_vertex(
        &mut self,
        reading: &Reading<'_>,
        vertex_point: EntityId,
    ) -> Result<DraftVertex, Refusal> {
        if let Some(&already) = self.vertices.get(&vertex_point) {
            return Ok(already);
        }
        let point = self.point_of_vertex(reading, vertex_point)?;
        let drafted = self
            .builder
            .add_vertex(point)
            .map_err(|why| self.refused(vertex_point, why))?;
        self.vertex_names.push(vertex_point);
        self.vertices.insert(vertex_point, drafted);
        Ok(drafted)
    }

    fn point_of_vertex(
        &self,
        reading: &Reading<'_>,
        vertex_point: EntityId,
    ) -> Result<Point, Refusal> {
        let at = reading.reference_at(vertex_point, "VERTEX_POINT", 1)?;
        reading.point_at(vertex_point, at)
    }

    /// Check the whole description and write it into `model`.
    ///
    /// `&self` rather than `self`: [`ProbeWalk`] asks this of a walk it does
    /// not own, having already spent `self` on probing every face.
    /// `self.builder.clone()` was already how the one call this had was
    /// answered, so taking `self` by value never bought anything here.
    fn build(
        &self,
        reading: &Reading<'_>,
        model: &mut Model,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<SolidKey, Refusal> {
        let builder = self.builder.clone();
        match builder.build(model, policy, log) {
            Ok(solid) => Ok(solid),
            Err(why) => Err(self.what_stopped_the_build(reading, why)),
        }
    }

    /// The refusal to publish for a build that was stopped, which is **the
    /// reason that came back and never a search for a different one**.
    ///
    /// Only one reason gets looked into further, and only because it is the one
    /// reason that arrives with no number attached:
    /// [`Invalid::VertexOffItsFaceSurface`] says a corner is certified off a
    /// surface and says nothing about *which* corner or by *how much*. Both
    /// scans below answer exactly that question about exactly that reason.
    ///
    /// **Every other reason is published as it stands.** Before `arms0`, the
    /// curved-face scan ran on every refusal whatever it said, so a file the
    /// builder stopped for a wire that does not close could be published as a
    /// corner off a curved face — a corner that had nothing to do with why the
    /// build failed, and whose miss the corpus report then printed as if it
    /// measured the refusal.
    ///
    /// **Which of the two scans answers first is still a guess, and it is a
    /// measured one.** `Model::validate` reaches this reason from two places —
    /// `check_edges`, on the plane an arc is *named* by, and then
    /// `check_curved_face`, on the surface a face sits on — and the reason it
    /// hands back does not say which. Asking `check_edges`' scan first because
    /// `check_edges` runs first is wrong in practice: `check_edges` spends
    /// `Model::what_writing_a_plane_down_costs` of room, which is 64 coordinate
    /// spacings, and the arc misses this scan finds in the KiCad corpus have a
    /// median of 0.64 of one. So the corners it would name are corners the
    /// sweep let straight through, and the face is what stopped the build. The
    /// face is asked first for that reason and not for the order of the checks.
    ///
    /// Closing that properly wants the sweep's own two allowances, and neither
    /// `Model::what_writing_a_plane_down_costs` nor
    /// `Model::what_writing_a_point_down_costs` is reachable from this crate.
    fn what_stopped_the_build(&self, reading: &Reading<'_>, why: rcad_topo::Error) -> Refusal {
        if matches!(
            why,
            rcad_topo::Error::Invalid(Invalid::VertexOffItsFaceSurface)
        ) && let Some(off) = self
            .a_corner_provably_off_a_curved_face(reading)
            .or_else(|| self.a_corner_provably_off_the_plane_of_its_own_arc(reading))
        {
            return off;
        }
        match self.which_entity_the_builder_meant(&why) {
            Some(entity) => Refusal::ThisEntityOfTheSolidWasRefused {
                solid: self.brep,
                entity,
                why: Box::new(why),
            },
            None => Refusal::TheSolidTheFileDescribesIsNotOne {
                solid: self.brep,
                why: Box::new(why),
            },
        }
    }

    /// The worst corner this description puts on a curved face it is provably
    /// not on, with how far off it is.
    ///
    /// **Asked only of a build the sweep stopped for exactly this**, so nothing
    /// is loosened and no import that succeeded went through here.
    ///
    /// The sign comes from the same certified predicate the sweep asked —
    /// `Decision::Pos` or `Decision::Neg` and nothing else counts, so a corner
    /// the arithmetic merely could not place is not reported as one that is off.
    /// The distance is the corner's own distance to the point the surface's own
    /// two coordinates put it at, which for a corner near a place the surface
    /// pinches is an over-estimate and never an under-estimate.
    fn a_corner_provably_off_a_curved_face(&self, reading: &Reading<'_>) -> Option<Refusal> {
        let mut worst: Option<Refusal> = None;
        let mut worst_by = 0.0;
        for (face, surface) in self.face_names.iter().zip(&self.face_surfaces) {
            if surface.as_plane().is_some() {
                continue;
            }
            let Ok(corners) = self.corners_of_face(reading, *face) else {
                continue;
            };
            for (corner, point) in corners {
                let at = PredicatePoint::from(point.coordinates());
                if !matches!(side_of(surface, at), Decision::Pos | Decision::Neg) {
                    continue;
                }
                let off_by = how_far_off(surface, point);
                if worst.is_none() || off_by > worst_by {
                    worst_by = off_by;
                    worst = Some(Refusal::ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn {
                        solid: self.brep,
                        face: *face,
                        corner,
                        off_by,
                        coordinate_spacing: point.largest_coordinate_magnitude() * f64::EPSILON,
                    });
                }
            }
        }
        worst
    }

    /// The worst corner this description puts at the end of an arc whose own
    /// plane it is provably not on.
    ///
    /// **Asked only of a build the sweep stopped for a corner off a surface**,
    /// the same as its curved-face twin above, and asked only when that one
    /// found nothing.
    ///
    /// An arc is named by the surfaces either side of it *and a second
    /// surface*, because two ends on a closed curve name two arcs — and
    /// `check_edges` asks the corners of a named arc to be on that second
    /// surface. The second surface is the plane the circle lies in, and a
    /// [`Plane`] scales its normal to length one and rounds its offset, so a
    /// corner is exactly on it only where the numbers cancel.
    ///
    /// **This measures more corners than the sweep refuses**, because
    /// `d343191` gave `check_edges` `Model::what_writing_a_plane_down_costs` of
    /// room for exactly that rounding and this crate cannot read that number.
    /// So the corner it names is the worst one off the plane and not
    /// necessarily the one the sweep stopped at.
    fn a_corner_provably_off_the_plane_of_its_own_arc(
        &self,
        reading: &Reading<'_>,
    ) -> Option<Refusal> {
        let mut worst: Option<Refusal> = None;
        let mut worst_by = 0.0;
        for edge_curve in self.edges.keys() {
            let Some((flat, corners)) = self.plane_of_the_arc(reading, *edge_curve) else {
                continue;
            };
            for (corner, point) in corners {
                let at = PredicatePoint::from(point.coordinates());
                if !matches!(flat.side_of_point(at), Decision::Pos | Decision::Neg) {
                    continue;
                }
                let off_by = flat.signed_distance_to(point).abs();
                if worst.is_none() || off_by > worst_by {
                    worst_by = off_by;
                    worst = Some(Refusal::ACornerIsNotOnThePlaneOfTheArcItEndsOn {
                        solid: self.brep,
                        edge: *edge_curve,
                        corner,
                        off_by,
                        coordinate_spacing: point.largest_coordinate_magnitude() * f64::EPSILON,
                    });
                }
            }
        }
        worst
    }

    /// The plane an arc lies in and the two corners it ends at, for an edge
    /// that is one, and nothing for an edge that is not.
    fn plane_of_the_arc(
        &self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
    ) -> Option<(Plane, [(EntityId, Point); 2])> {
        let ends = [
            reading.reference_at(edge_curve, "EDGE_CURVE", 1).ok()?,
            reading.reference_at(edge_curve, "EDGE_CURVE", 2).ok()?,
        ];
        if ends[0] == ends[1] {
            return None;
        }
        let curve = reading.reference_at(edge_curve, "EDGE_CURVE", 3).ok()?;
        let keyword = match reading.file().instance(curve)?.parts()[0].keyword() {
            "CIRCLE" => "CIRCLE",
            "ELLIPSE" => "ELLIPSE",
            _ => return None,
        };
        let position = reading.reference_at(curve, keyword, 1).ok()?;
        let place = reading.placement_at(curve, position).ok()?;
        let flat = Plane::from_point_and_normal(place.origin, place.axis).ok()?;
        Some((
            flat,
            [
                (ends[0], self.point_of_vertex(reading, ends[0]).ok()?),
                (ends[1], self.point_of_vertex(reading, ends[1]).ok()?),
            ],
        ))
    }

    /// Every corner of one face, with the `VERTEX_POINT` it was written under.
    fn corners_of_face(
        &self,
        reading: &Reading<'_>,
        face: EntityId,
    ) -> Result<Vec<(EntityId, Point)>, Refusal> {
        let keyword = reading.keyword_of(self.brep, face, &["ADVANCED_FACE", "FACE_SURFACE"])?;
        let mut corners = Vec::new();
        for bound in reading.references_at(face, keyword, 1)? {
            let (loop_id, _) = self.loop_of_bound(reading, face, bound)?;
            for use_of_an_edge in reading.references_at(loop_id, "EDGE_LOOP", 1)? {
                let edge_curve = reading.reference_at(use_of_an_edge, "ORIENTED_EDGE", 3)?;
                for position in 1..=2 {
                    let at = reading.reference_at(edge_curve, "EDGE_CURVE", position)?;
                    corners.push((at, self.point_of_vertex(reading, at)?));
                }
            }
        }
        Ok(corners)
    }

    /// The `#id` behind an index a builder refusal carries.
    ///
    /// Every draft entity was pushed onto its own name list in the order the
    /// builder handed the handle out, so the index the refusal names is the
    /// position in that list. A refusal that names no entity comes back `None`.
    ///
    /// **The sweep's own refusals used to be all of that `None`**, and the one
    /// this reader meets most was the whole of it: 44 KiCad files stopped on a
    /// crossing the marcher could not follow, and every one of them published
    /// `#15`, the `MANIFOLD_SOLID_BREP` (`docs/decisions.md` §109.1). That
    /// refusal names an edge now —
    /// [`rcad_topo::Error::ADraftEdgeHasACrossingThatCannotBeFollowed`] — and an
    /// `EDGE_CURVE` names the two faces either side of it, which is the pair the
    /// marcher gave up between. Every *other* sweep refusal is still `None` and
    /// still about the solid as a whole.
    fn which_entity_the_builder_meant(&self, why: &rcad_topo::Error) -> Option<EntityId> {
        let (names, index) = match why {
            rcad_topo::Error::ADraftPointIsNotFinite { vertex } => (&self.vertex_names, *vertex),
            rcad_topo::Error::ADraftEdgeHasNoLength { edge }
            | rcad_topo::Error::ADraftEdgeIsWalkedTheSameWayTwice { edge }
            | rcad_topo::Error::ADraftEdgeIsNotWalkedByTwoCoedges { edge, .. }
            | rcad_topo::Error::ADraftEdgeHasACrossingThatCannotBeFollowed { edge, .. } => {
                (&self.edge_names, *edge)
            }
            rcad_topo::Error::ADraftEntityIsUsedTwice { entity, index }
            | rcad_topo::Error::ADraftEntityIsUsedByNothing { entity, index } => {
                (self.names_of(entity)?, *index)
            }
            rcad_topo::Error::ADraftFaceCouldNotBeMeshed { face, .. } => (&self.face_names, *face),
            _ => return None,
        };
        names.get(index).copied()
    }

    fn names_of(&self, entity: &str) -> Option<&Vec<EntityId>> {
        match entity {
            "vertex" => Some(&self.vertex_names),
            "edge" => Some(&self.edge_names),
            "coedge" => Some(&self.coedge_names),
            "wire" => Some(&self.wire_names),
            "face" => Some(&self.face_names),
            _ => None,
        }
    }

    fn refused(&self, entity: EntityId, why: rcad_topo::Error) -> Refusal {
        Refusal::ThisEntityOfTheSolidWasRefused {
            solid: self.brep,
            entity,
            why: Box::new(why),
        }
    }
}

/// Wraps a [`Walk`] and drives it fault-tolerantly for [`probe_solid`].
///
/// `Walk`'s own `describe_the_shell`/`describe_the_face`/`describe_the_bound`
/// all use `?` to stop at the first entity that refuses, which is exactly right
/// for [`solid_from`] and exactly wrong for counting how many reasons a file
/// gives. These three methods are the ones this rewrites; everything below them
/// — `describe_the_coedge`, `describe_the_edge`, `run_along`,
/// `describe_the_vertex` — is called through `self.walk` unchanged, because
/// nothing past one edge or one vertex has a sibling to carry on to.
struct ProbeWalk {
    walk: Walk,
    /// Every refusal found so far, in the order the walk met them.
    refusals: Vec<Refusal>,
}

impl ProbeWalk {
    fn new(brep: EntityId) -> Self {
        Self {
            walk: Walk::new(brep),
            refusals: Vec::new(),
        }
    }

    /// Every face of the shell that could be described, skipping and recording
    /// any that could not — the fault-tolerant twin of
    /// [`Walk::describe_the_shell`].
    ///
    /// Only the face list itself not resolving is handed back as an `Err`: with
    /// no list, there is nothing to skip through, which is
    /// [`ProbeOutcome::fatal`]'s case and not this one's.
    ///
    /// `SolidBuilder::add_shell` is asked only when every face came back — the
    /// same rule [`Self::probe_the_face`] and [`Self::probe_the_bound`] apply
    /// one level down, for the same reason: a shell missing a face is open, and
    /// a refusal for that is the shape of the missing face, not a second reason
    /// next to it. [`probe_solid`] makes the same call about the final `build`,
    /// one level up from here.
    ///
    /// **Unlike the two levels below, this one is not known to cost anything.**
    /// `add_shell` took a block's five remaining faces without complaint when
    /// the sixth was dropped, so the artefactual reason it guards against is
    /// one no test here reaches; what actually catches an open skin is
    /// `build`, which [`probe_solid`] already withholds from a solid that lost
    /// something. The rule is kept at all three levels because a walk that
    /// applies it at two is a walk whose numbers depend on which level a future
    /// check lands at.
    fn probe_the_shell(&mut self, reading: &Reading<'_>, shell: EntityId) -> Result<(), Refusal> {
        let faces = reading.references_at(shell, "CLOSED_SHELL", 1)?;
        let face_count = faces.len();
        self.walk.find_the_closed_edges(reading, &faces);
        let mut skin: Vec<(DraftFace, Orientation)> = Vec::with_capacity(face_count);
        for face in faces {
            if let Some(described) = self.probe_the_face(reading, face) {
                skin.push(described);
            }
        }
        if skin.len() == face_count
            && let Err(why) = self.walk.builder.add_shell(&skin)
        {
            self.refusals.push(self.walk.refused(shell, why));
        }
        Ok(())
    }

    /// One `ADVANCED_FACE`, or `None` with the reason recorded — the
    /// fault-tolerant twin of [`Walk::describe_the_face`]. A face whose surface
    /// or bound list will not even resolve is dropped whole.
    ///
    /// **`SolidBuilder::add_face` is asked only when every one of the face's
    /// bounds came back whole.** A face missing a bound is a face with a hole
    /// where a wire should be, and `add_face` then reports *that hole* —
    /// `crossing`, `winding` and the rest of its refusals are about a bound
    /// that is there, so a missing one is not a second reason the file has, it
    /// is the shape of the first one. Recording it as one cost
    /// `B_SPLINE_SURFACE_WITH_KNOTS` and `SURFACE_OF_REVOLUTION` every file
    /// where either is the *only* reason: each always left a face-shaped hole,
    /// and `add_face` on the hole always refused for a second, artefactual
    /// reason before this asked the real question.
    fn probe_the_face(
        &mut self,
        reading: &Reading<'_>,
        face: EntityId,
    ) -> Option<(DraftFace, Orientation)> {
        let keyword = self.record(reading.keyword_of(
            self.walk.brep,
            face,
            &["ADVANCED_FACE", "FACE_SURFACE"],
        ))?;
        let sits_on = self.record(reading.reference_at(face, keyword, 2))?;
        let written = self.record(reading.surface_at(face, sits_on))?;
        let same_sense = self.record(reading.flag_at(face, keyword, 3))?;
        let (surface, used) = match (written.as_plane(), same_sense) {
            (_, true) => (written, Orientation::Forward),
            (Some(flat), false) => (Surface::Plane(flat.reversed()), Orientation::Forward),
            (None, false) => (written, Orientation::Reversed),
        };

        let bounds = self.record(reading.references_at(face, keyword, 1))?;
        let outer_first = self.record(
            self.walk
                .outer_bound_first(reading, face, &bounds, &surface),
        )?;
        let bound_count = outer_first.len();
        let mut wires = Vec::with_capacity(bound_count);
        for bound in outer_first {
            if let Some(wire) = self.probe_the_bound(reading, face, bound) {
                wires.push(wire);
            }
        }
        if wires.len() < bound_count {
            return None;
        }
        match self.walk.builder.add_face(surface, &wires) {
            Ok(drafted) => {
                self.walk.face_names.push(face);
                self.walk.face_surfaces.push(surface);
                Some((drafted, used))
            }
            Err(why) => {
                self.refusals.push(self.walk.refused(face, why));
                None
            }
        }
    }

    /// One `FACE_BOUND` or `FACE_OUTER_BOUND`, or `None` with the reason
    /// recorded — the fault-tolerant twin of [`Walk::describe_the_bound`]. A
    /// bound whose own loop will not resolve is dropped whole.
    ///
    /// **`SolidBuilder::add_wire` is asked only when every one of the loop's
    /// steps read cleanly**, for the same reason [`Self::probe_the_face`] asks
    /// `add_face` only of a face with every bound: a wire missing a step is
    /// open where a closed loop is asked for, and "does not close" is the shape
    /// of the step that is missing, not a second thing wrong with the edge that
    /// could not be named.
    fn probe_the_bound(
        &mut self,
        reading: &Reading<'_>,
        face: EntityId,
        bound: EntityId,
    ) -> Option<rcad_topo::DraftWire> {
        let (loop_id, walk_it_backwards) =
            self.record(self.walk.loop_of_bound(reading, face, bound))?;
        let mut oriented = self.record(reading.references_at(loop_id, "EDGE_LOOP", 1))?;
        if walk_it_backwards {
            oriented.reverse();
        }
        let mut steps = Vec::with_capacity(oriented.len());
        let mut every_edge_described = true;
        for use_of_an_edge in oriented {
            match self
                .walk
                .describe_the_coedge(reading, use_of_an_edge, walk_it_backwards)
            {
                Ok(coedges) => steps.extend(coedges.into_iter().map(DraftStep::Along)),
                Err(why) => {
                    self.refusals.push(why);
                    every_edge_described = false;
                }
            }
        }
        if !every_edge_described {
            return None;
        }
        match self.walk.builder.add_wire(&steps) {
            Ok(wire) => {
                self.walk.wire_names.push(loop_id);
                Some(wire)
            }
            Err(why) => {
                self.refusals.push(self.walk.refused(loop_id, why));
                None
            }
        }
    }

    /// Record `result`'s `Err` and turn it into `None`, or hand back its `Ok`.
    /// Every one of the three methods above is this pattern repeated on a
    /// different reading call, so it earns the name.
    fn record<T>(&mut self, result: Result<T, Refusal>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(why) => {
                self.refusals.push(why);
                None
            }
        }
    }
}

/// The half-space facing `facing` whose flat surface stands on the chord, low
/// enough that **neither** end is behind it.
///
/// `Model::validate` refuses an edge whose corner is provably outside something
/// picking it, and an offset is one dot product and rounds — so a surface put
/// exactly through one end lands on a coin-toss side of the other. The fix is
/// arithmetic and not a tolerance: drop the offset by the width the dot product
/// itself carries.
///
/// A dot product of three terms computed in `f64` is within `γ₃ = 3u/(1 − 3u)`
/// of the exact value times the sum of the terms' magnitudes, and `γ₃ < 2ε`.
/// **Eight epsilons of that sum is the drop**, which leaves five to spare for
/// the rounding of the subtraction itself; the flat surface moves off the true
/// chord by at most `8ε·Σ|nᵢpᵢ| / |n|`, which is a rounding and not a length.
/// What that costs is bounded and worth stating: the half-space stops telling
/// the two arcs apart within that distance of the chord, and the two arcs meet
/// **on** the chord, so the ambiguity is a rounding around the corners the two
/// share anyway.
///
/// `None` when the numbers do not leave a working half-space at all — an
/// infinite offset, or a chord and an axis that are along each other so the
/// cross product has no length. Both are the file's numbers and neither is
/// reachable from a well-formed arc.
/// Whether an `EDGE_CURVE` starts and ends at the same `VERTEX_POINT`, which is
/// what [`EdgeRun::RightRoundWhereItsFacesCross`] is named for.
///
/// **Two `VERTEX_POINT`s at the same coordinates are two corners here too**, the
/// same as everywhere else in this walk — see [`Walk::describe_the_vertex`]. An
/// edge between those does not close, and asking it to be walked by the
/// closed-edge rule would be reading a rule off a wire that is open.
fn an_edge_with_one_corner(reading: &Reading<'_>, edge_curve: EntityId) -> bool {
    matches!(
        (
            reading.reference_at(edge_curve, "EDGE_CURVE", 1),
            reading.reference_at(edge_curve, "EDGE_CURVE", 2),
        ),
        (Ok(start), Ok(end)) if start == end
    )
}

/// Which of the surfaces either side of a closed edge names which way it runs.
///
/// **The same pick `rcad-topo`'s `the_curved_one_of` makes, and it has to be the
/// same one.** A plane never names one — two planes cross in a straight line,
/// which does not close — and where both are curved the order over the numbers
/// each surface is written with settles it, so that both faces of the edge land
/// on one surface however the file happened to write the pair down.
/// [`Surface::how_it_sorts_against`] is that order, and `rcad-topo` sorts its own
/// copy of the same bits into the same rows.
fn the_curved_one_of(carried_by: &[Surface]) -> Option<Surface> {
    carried_by
        .iter()
        .filter(|surface| surface.as_plane().is_none())
        .copied()
        .min_by(Surface::how_it_sorts_against)
}

/// The face a run names where every face along it is curved and they are not
/// all one surface: the ball if there is one, and otherwise the first face.
///
/// The ball first, because it is the face `rcad_geom::walk` pairs with a plane
/// when there is one, so a run naming it rather than another face changes
/// nothing a ball's run did before §220.4.
fn a_face_where_curved_faces_meet(faces: &[Surface]) -> Option<Surface> {
    let all_curved = faces.len() >= 2 && faces.iter().all(|face| face.as_plane().is_none());
    if !all_curved || the_faces_along_it_name_no_crossing(faces) {
        return None;
    }
    Some(a_ball_where_curved_faces_meet(faces).map_or(faces[0], Surface::Sphere))
}

/// The ball among the faces along an edge, when every face along it is curved
/// and they are not all one surface.
///
/// One surface on both sides is a seam, and a seam's run already names its
/// second surface for the walk to read.
fn a_ball_where_curved_faces_meet(faces: &[Surface]) -> Option<Sphere> {
    let all_curved = faces.len() >= 2 && faces.iter().all(|face| face.as_plane().is_none());
    if !all_curved || the_faces_along_it_name_no_crossing(faces) {
        return None;
    }
    faces.iter().find_map(|face| {
        if let Surface::Sphere(ball) = face {
            Some(*ball)
        } else {
            None
        }
    })
}

/// Whether the faces along an edge name no crossing at all, so that a run right
/// round it has to carry the second surface itself.
///
/// Two ways, and **neither is a nearness**: both answer `true` only where the
/// pair provably names nothing, so no run gets an `and` its curve might not lie
/// on. That is the mistake worth being exact about — §216's measured 1.45 mm on
/// a 100 mm solid is what a wrong second surface costs. Answering `false` where
/// the pair in truth names nothing leaves the answer this reader already gave.
///
/// **Every face along it is flat.** Two planes cross in a straight line, which
/// never closes, so whatever the two are — one plane written twice, two
/// spellings of it a hair apart, or two that really do lean on each other — the
/// pair does not name a closed curve. This is a fact about the kinds and needs
/// no tolerance, and it is the one that reaches the corpus: the flat faces
/// either side of a circle a file imprinted on one come out `1e-15` apart, so
/// the `==` below misses every one of them. `Model::validate` says the same
/// thing back as `ClosedEdgeBetweenTwoFlatFaces`, and a reader whose model the
/// validator refuses by name should not have written it.
///
/// **Or every face is the one surface, `==` and not near it.** A surface
/// crossed with itself is not a curve. Written twice a hair apart it fails this,
/// and [`Walk::round_run`] then names the run by the file's circle where it
/// has checked that circle lies on every face, as an arc's is named — which is
/// how a curved surface written twice a hair apart is answered without a
/// nearness here.
///
/// Fewer than two faces names nothing either way, and the shell that holds such
/// an edge is about to be refused by name.
fn the_faces_along_it_name_no_crossing(faces: &[Surface]) -> bool {
    faces.len() >= 2
        && (faces.iter().all(|face| face.as_plane().is_some())
            || faces.windows(2).all(|pair| pair[0] == pair[1]))
}

/// Whether every point of the circle with centre `centre`, unit axis `axis`
/// and radius `radius` is within the allowance of `surface`.
///
/// Write `c`, `w`, `r` for the circle and `e` for a unit direction square to
/// `w`, so a point of it is `X = c + r·e`.
///
/// - **A ball** of centre `s` and radius `ρ`. With `d = c − s` split into
///   `h = d·w` along the axis and `q = |d − h·w|` across it,
///   `|X − s|² = h² + |d − h·w + r·e|²`, and the second term runs from
///   `(r − q)²` to `(r + q)²` as `e` goes round. Both ends are held against `ρ`.
/// - **A pipe** of axis direction `a` and radius `R`. With `q` the distance
///   from `c` to the axis and `sin θ = |w × a|`, the part of `r·e` square to
///   the axis is at least `r·(1 − sin θ)` long and at most `r`, so every point
///   is between `r·(1 − sin θ) − q` and `r + q` from the axis, and
///   `|r − R| + q + r·sin θ` bounds how far any point is from the pipe.
///
/// The cone and the torus are surfaces turned round an axis, and a point's
/// distance from one is read off two numbers: `ρ`, how far the point is from
/// the axis, and `z`, how far along it. For the circle those move by
/// `|ρ − r| ≤ q + r·sin θ` and `|z − z_c| ≤ r·sin θ` from what its centre
/// gives, with `q`, `θ` as for the pipe and `z_c` the centre's own `z`.
///
/// - **A cone** of apex `p`, axis `a` and slope `s = tan α`. Its two nappes
///   are the lines `ρ = ±s·z` in every half-plane through the axis, so a
///   point is `|ρ·cos α − |z|·sin α|` from it. That moves by at most
///   `|Δρ| + |Δz|`, so `|r·cos α − |z_c|·sin α| + q + 2·r·sin θ` bounds every
///   point of the circle.
/// - **A torus** of centre `o`, axis `a`, ring `R` and tube `ρ_t`. A point is
///   `|√((ρ − R)² + z²) − ρ_t|` from it, and the root moves by at most
///   `|Δρ| + |Δz|`, so a circle turning about the torus's axis is bounded by
///   `|√((r − R)² + z_c²) − ρ_t| + q + 2·r·sin θ`, where `sin θ` is a
///   rounding at most because this is only asked where `w × a` came out
///   exactly nothing, and is left out. A circle round the tube is
///   bounded instead by how far it is from the tube circle it is nearest:
///   with `m` the point of the centre circle nearest `c` and `t` the centre
///   circle's direction there, that circle is `m`, `t`, `ρ_t`, and a point of
///   one circle is at most `|c − m| + |r − ρ_t| + 2·r·sin φ` from the point
///   of the other the turn from `w` to `t` takes it to, `sin φ = |w × t|`.
///   Which of the two is asked is what the walk asks — whether `w` crosses
///   `a` to exactly nothing — because the walk draws a flat crossing of a
///   torus from the torus alone once it has picked a family, and a circle
///   leaning a rounding off square would otherwise be drawn round the tube.
///
/// These are worked out in plain `f64`, and the allowance is
/// [`A_CIRCLE_MAY_MISS_ITS_FACE_BY`] times the rounding of the largest number
/// in them, so the few roundings each bound costs cannot carry a circle that
/// is off by the allowance back onto the surface — only one that is off by
/// that much less a few ulps.
///
/// Anything else holds nothing, which is the safe direction: the run is then
/// named by its faces as before.
fn a_surface_holds_the_circle(surface: &Surface, centre: Point, axis: Vector, radius: f64) -> bool {
    how_far_off_the_circle_can_be(surface, centre, axis, radius).is_some_and(|(off, largest)| {
        off <= A_CIRCLE_MAY_MISS_ITS_FACE_BY * NEAREST_F64_POINT_ROUNDOFF * largest
    })
}

/// How many times [`NEAREST_F64_POINT_ROUNDOFF`] of the largest number
/// involved a file's `CIRCLE` may sit off a face along its edge and still be
/// read as lying on it.
///
/// **Why a width and not a certified sign.** The files write these circles
/// off their faces, in exact arithmetic and by any reading of the decimals.
/// `kicad/LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step` puts circle `#316` at
/// `(1.27, -5.07, 10.)` turning about `(-5.551115123126E-17, -1., 0.)`, on
/// pipe `#324` whose axis starts at `(1.27, -1.27, 10.)` and leans the same
/// way. The centre is `3.8` along a line leaning `5.6e-17`, so it misses the
/// pipe's axis by `2.1e-16`, and the pipe's own equation at every point of
/// the circle has a sign that is not zero. A check that asked the sign would
/// refuse the circle, and the answer "it is on the face" does not exist for
/// those numbers — that is a limit, not a gap (`docs/decisions.md` §16.10).
/// Measured with an allowance of zero, which keeps only circles whose miss
/// rounds to nothing in `f64`: the KiCad corpus imports 115 files rather
/// than 160, the 45 lost being the D3.0 and D5.0 horizontal LEDs, 24 relays
/// and one fuseholder.
///
/// **Where sixteen comes from.** It is the sixteen `Model::validate` gives a
/// corner of a curved face (`MEASURING_SLACK` in `rcad-topo`'s
/// `validate.rs`), so the reader holds a circle to the standard the validator
/// holds its corners to. It was not fitted to the corpus, and the corpus
/// needs much less: over the whole corpus on `b3701c4c`, the furthest a
/// circle sits off its face in any file that imports is `1.20` of these
/// units, on `Relay_3PST_COTO_3650.step` and `_3660.step`. An allowance of
/// `1` loses those two relays (158 imports); every allowance from `4` to
/// `1000` gives the same 160. The circles sixteen turns away short of `34`
/// are all in `nist_stc_08_asme1_ap242-e3.stp` and the two Fujitsu `FTR-F1`
/// relays, which are refused for something else first.
/// `the_furthest_a_circle_an_imported_file_needs_is_inside_the_allowance`
/// holds the relay's circle against this number.
const A_CIRCLE_MAY_MISS_ITS_FACE_BY: f64 = 16.0;

/// The furthest any point of the circle can be from `surface`, and the
/// largest number the bound was worked out from — see
/// [`a_surface_holds_the_circle`] for the bounds. `None` for a surface with no
/// bound here.
fn how_far_off_the_circle_can_be(
    surface: &Surface,
    centre: Point,
    axis: Vector,
    radius: f64,
) -> Option<(f64, f64)> {
    match surface {
        Surface::Sphere(ball) => {
            let off = centre - ball.centre();
            let along = off.dot(axis);
            let across = (off - axis * along).length();
            let [outer, inner] = [radius + across, (radius - across).abs()]
                .map(|round| ((along * along + round * round).sqrt() - ball.radius()).abs());
            Some((
                the_further_of(outer, inner),
                largest_of(&[centre, ball.centre()], &[radius, ball.radius()]),
            ))
        }
        Surface::Cylinder(pipe) => {
            let line = pipe.axis();
            let tilt = axis.cross(line.direction()).length();
            let across = line.distance_to_point(centre);
            Some((
                (radius - pipe.radius()).abs() + across + radius * tilt,
                largest_of(&[centre, line.origin()], &[radius, pipe.radius()]),
            ))
        }
        Surface::Cone(cone) => {
            let line = cone.axis();
            let tilt = axis.cross(line.direction()).length();
            let across = line.distance_to_point(centre);
            let along = (centre - cone.apex()).dot(line.direction()).abs();
            let hypotenuse = cone.slope().hypot(1.0);
            let [cos, sin] = [hypotenuse.recip(), cone.slope() / hypotenuse];
            Some((
                (radius * cos - along * sin).abs() + across + 2.0 * radius * tilt,
                largest_of(&[centre, cone.apex()], &[radius, along]),
            ))
        }
        Surface::Torus(ring) => {
            let line = ring.axis();
            let up = line.direction();
            let off = centre - ring.centre();
            let along = off.dot(up);
            let largest = largest_of(&[centre, ring.centre()], &[radius, ring.ring_radius()]);
            if axis.cross(up) == Vector::zero() {
                let across = line.distance_to_point(centre);
                let from_the_tube = (radius - ring.ring_radius()).hypot(along);
                return Some(((from_the_tube - ring.tube_radius()).abs() + across, largest));
            }
            let outward = off - up * along;
            let reach = outward.length();
            let round_the_ring = up.cross(outward.normalized().ok()?);
            Some((
                (reach - ring.ring_radius()).hypot(along)
                    + (radius - ring.tube_radius()).abs()
                    + 2.0 * radius * axis.cross(round_the_ring).length(),
                largest,
            ))
        }
        Surface::Plane(_) => None,
        // A surface kind this reader has no bound for holds nothing, which
        // names the run by its faces as before.
        _ => None,
    }
}

/// The largest coordinate or length a bound was worked out from, which is
/// what its roundings are measured against.
fn largest_of(points: &[Point], lengths: &[f64]) -> f64 {
    points
        .iter()
        .map(Point::largest_coordinate_magnitude)
        .chain(lengths.iter().map(|length| length.abs()))
        .fold(0.0, f64::max)
}

/// The larger of two distances, and a NaN if either is one.
///
/// Not `f64::max`, which drops a NaN and would hold the curve on the other
/// bound. Both arguments are `abs` of something, which leaves a NaN positive,
/// and `total_cmp` sorts a positive NaN above everything, so it reaches the
/// comparison with the allowance and fails it.
fn the_further_of(one: f64, other: f64) -> f64 {
    if one.total_cmp(&other).is_ge() {
        one
    } else {
        other
    }
}

/// An `ELLIPSE` as the file writes it: `centre + a·cos t·first_way +
/// b·sin t·second_way` for `semi_axes = [a, b]`, the two ways unit and square
/// to each other and to the plane's normal.
struct Ellipse {
    centre: Point,
    first_way: Vector,
    second_way: Vector,
    semi_axes: [f64; 2],
}

/// Whether every point of `ellipse` is within the allowance of `surface`.
///
/// **Only a pipe has a bound**, and it is exact but for how far the centre is
/// off the axis. With `d` the pipe's unit axis, a point of the ellipse is
/// `c + M(t)` and its part square to the axis is that of `c` plus that of
/// `M(t)`. The second is as long as `M(t) × d`, and
///
/// ```text
/// |M(t) × d|² = A²·cos²t + B²·sin²t + 2·C·cos t·sin t
/// A = |a·u × d|,  B = |b·v × d|,  C = (a·u × d)·(b·v × d)
/// ```
///
/// runs over exactly the two eigenvalues of `[[A², C], [C, B²]]` as `t`
/// goes round, `(A² + B²)/2 ± √(((A² − B²)/2)² + C²)`. With `q` the distance
/// from `c` to the axis, every point is within `q` of those two lengths from
/// the axis, and the further of `|√λ − R|` over the two, plus `q`, bounds how
/// far any point is from the pipe. An ellipse drawn where a plane cuts the
/// pipe has `A = B = R` and `C = 0`.
///
/// **A ball, a cone and a torus hold nothing.** A ball holds no ellipse that
/// is not a circle, and a file that writes a circle as an `ELLIPSE` with two
/// equal axes loses nothing by being named by its faces. A plane cuts a cone
/// in an ellipse too, but no file in either corpus writes one between two
/// curved faces that anyone has counted, and a bound for it is §13's to wait
/// for. A plane cuts a torus in an ellipse nowhere.
fn a_surface_holds_the_ellipse(surface: &Surface, ellipse: &Ellipse) -> bool {
    how_far_off_the_ellipse_can_be(surface, ellipse).is_some_and(|(off, largest)| {
        off <= A_CIRCLE_MAY_MISS_ITS_FACE_BY * NEAREST_F64_POINT_ROUNDOFF * largest
    })
}

/// The furthest any point of `ellipse` can be from `surface`, and the largest
/// number the bound was worked out from — see [`a_surface_holds_the_ellipse`].
/// `None` for a surface with no bound here.
fn how_far_off_the_ellipse_can_be(surface: &Surface, ellipse: &Ellipse) -> Option<(f64, f64)> {
    let Surface::Cylinder(pipe) = surface else {
        return None;
    };
    let line = pipe.axis();
    let [first, second] = ellipse.semi_axes;
    let across_first = ellipse.first_way.cross(line.direction()) * first;
    let across_second = ellipse.second_way.cross(line.direction()) * second;
    let [firsts, seconds, both] = [
        across_first.length_squared(),
        across_second.length_squared(),
        across_first.dot(across_second),
    ];
    let middle = (firsts + seconds) / 2.0;
    let spread = ((firsts - seconds) / 2.0).hypot(both);
    let [nearest, furthest] = [(middle - spread).max(0.0).sqrt(), (middle + spread).sqrt()];
    let off = the_further_of(
        (furthest - pipe.radius()).abs(),
        (nearest - pipe.radius()).abs(),
    ) + line.distance_to_point(ellipse.centre);
    Some((
        off,
        largest_of(
            &[ellipse.centre, line.origin()],
            &[first, second, pipe.radius()],
        ),
    ))
}

fn a_half_space_holding_both(corners: [Point; 2], facing: Vector) -> Option<HalfSpace> {
    let below = |corner: Point| {
        let terms = facing.components().into_iter().zip(corner.coordinates());
        let spread: f64 = terms.clone().map(|(n, p)| (n * p).abs()).sum();
        let dot: f64 = terms.map(|(n, p)| n * p).sum();
        dot - 8.0 * f64::EPSILON * spread
    };
    let offset = below(corners[0]).min(below(corners[1]));
    let picking = HalfSpace::new(facing, offset).ok()?;
    corners
        .iter()
        .all(|corner| {
            picking.side_of_point(PredicatePoint::from(corner.coordinates())) != Decision::Neg
        })
        .then_some(picking)
}

/// Which side of a surface a point is on, through the same certified predicates
/// the sweep uses.
fn side_of(surface: &Surface, point: PredicatePoint) -> Decision {
    match surface {
        Surface::Plane(flat) => flat.side_of_point(point),
        Surface::Sphere(ball) => ball.side_of(point),
        Surface::Cylinder(pipe) => pipe.side_of(point),
        Surface::Cone(cone) => cone.side_of(point),
        Surface::Torus(ring) => ring.side_of(point),
        // `Surface` grows a variant every milestone this kernel learns a shape.
        // A new one has no sign predicate here until somebody writes it, and
        // reporting "not off it" about a surface nothing can ask about would be
        // a claim rather than a measurement.
        _ => Decision::Undecidable {
            enclosure: f64::INFINITY,
        },
    }
}

/// How far a point is from a surface, measured against the point the surface's
/// own two coordinates put it at.
///
/// Exact for a plane. For a curved surface it is the distance to the point the
/// surface's own chart maps the corner's parameters back to, which is a real
/// point of the surface — so it is an upper bound on the true distance and
/// never an under-estimate, which is the direction a refusal wants.
///
/// **Zero means the miss is smaller than the numbers can show, not that there
/// is none.** A corner an ulp off a pipe projects onto the pipe and rounds
/// straight back onto itself, while `x² + y² − r²` worked out exactly is still
/// not zero — which is the whole of why the certified predicate refuses a corner
/// this measurement cannot see. Half of the KiCad corpus's refused corners are
/// this case and the other half are a modelling tolerance somebody chose; the
/// two are told apart by exactly this number.
fn how_far_off(surface: &Surface, point: Point) -> f64 {
    match surface {
        Surface::Plane(flat) => flat.signed_distance_to(point).abs(),
        Surface::Sphere(ball) => {
            let (round, up) = ball.coordinates_of(point);
            ball.point_at(round, up).distance_to(point)
        }
        Surface::Cylinder(pipe) => {
            let (round, along) = pipe.coordinates_of(point);
            pipe.point_at(round, along).distance_to(point)
        }
        Surface::Cone(cone) => {
            let (round, along) = cone.coordinates_of(point);
            cone.point_at(round, along).distance_to(point)
        }
        Surface::Torus(ring) => {
            let (over, round) = ring.coordinates_of(point);
            ring.point_at(over, round).distance_to(point)
        }
        // See `side_of`: a surface with no chart here has no distance either.
        _ => f64::NAN,
    }
}

#[cfg(test)]
mod tests {
    use rcad_geom::{
        Axis, Cone, Cylinder, NEAREST_F64_POINT_ROUNDOFF, Point, Sphere, Surface, Torus, Vector,
    };
    use rcad_predicates::{Decision, Point as PredicatePoint};

    use super::{
        A_CIRCLE_MAY_MISS_ITS_FACE_BY, Ellipse, a_half_space_holding_both,
        a_surface_holds_the_circle, a_surface_holds_the_ellipse, how_far_off_the_circle_can_be,
        how_far_off_the_ellipse_can_be,
    };

    /// The LED body's circle, written as the file writes it: on its dome
    /// exactly, and `2.1e-16` off its body's axis. Both hold it, and a circle a
    /// thousandth of a unit off either is held by neither.
    #[test]
    fn the_leds_circle_is_held_by_its_body_and_its_dome_and_a_shifted_one_is_not() {
        let tilted = Vector::new(-5.551_115_123_126e-17, -1.0, 0.0);
        let centre = Point::new(1.27, -5.07, 10.0);
        let axis = tilted.normalized().expect("a direction");
        let body = Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::new(1.27, -1.27, 10.0), tilted).expect("an axis"),
                1.5,
            )
            .expect("a radius"),
        );
        let dome = Surface::Sphere(Sphere::new(centre, 1.5).expect("a radius"));

        for face in [body, dome] {
            assert!(
                a_surface_holds_the_circle(&face, centre, axis, 1.5),
                "{face:?} does not hold the file's own circle"
            );
            let shifted = Point::new(1.27, -5.07, 10.001);
            assert!(
                !a_surface_holds_the_circle(&face, shifted, axis, 1.5),
                "{face:?} holds a circle a thousandth off"
            );
            assert!(
                !a_surface_holds_the_circle(&face, centre, axis, 1.501),
                "{face:?} holds a circle a thousandth wider"
            );
        }
    }

    /// The worst case [`A_CIRCLE_MAY_MISS_ITS_FACE_BY`] publishes, held against
    /// it (`docs/decisions.md` §15.6): circle `#1359` of
    /// `kicad/Relay_3PST_COTO_3650.step` on its pipe `#1365`, the furthest off
    /// its face of any circle in a file that imports.
    ///
    /// Its centre is `15.764` along an axis leaning `2.2e-16`, so it misses the
    /// pipe's axis by `3.5e-15`, which is `1.20` roundings of the pipe's
    /// `13.462`. That is more than one, so an allowance of the bare rounding
    /// refuses a file the corpus imports, and it is far under sixteen.
    #[test]
    fn the_furthest_a_circle_an_imported_file_needs_is_inside_the_allowance() {
        let leaning = Vector::new(2.220_446_049_25e-16, 1.0, 0.0);
        let centre = Point::new(-1.54, 2.302, 11.06);
        let axis = leaning.normalized().expect("a direction");
        let pipe = Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::new(-1.54, -13.462, 11.06), leaning).expect("an axis"),
                1.0,
            )
            .expect("a radius"),
        );

        let (off, largest) =
            how_far_off_the_circle_can_be(&pipe, centre, axis, 1.0).expect("a pipe has a bound");
        let roundings = off / (NEAREST_F64_POINT_ROUNDOFF * largest);
        assert!(
            (1.0..1.25).contains(&roundings),
            "the relay's circle is {roundings} roundings off its pipe, where 1.20 was measured; \
             the corpus measurement in A_CIRCLE_MAY_MISS_ITS_FACE_BY's doc comment is stale"
        );
        assert!(
            roundings < A_CIRCLE_MAY_MISS_ITS_FACE_BY,
            "the allowance no longer covers the furthest circle an imported file needs"
        );
        assert!(a_surface_holds_the_circle(&pipe, centre, axis, 1.0));
    }

    /// A NaN in the numbers holds no circle. `f64::max` would drop it and
    /// answer with the other bound.
    #[test]
    fn a_ball_does_not_hold_a_circle_with_a_nan_radius() {
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 1.0).expect("a radius"));
        let axis = Vector::new(0.0, 0.0, 1.0);
        assert!(!a_surface_holds_the_circle(
            &ball,
            Point::origin(),
            axis,
            f64::NAN
        ));
    }

    /// A cone, a torus and a pipe, each with a curve drawn on it the way a
    /// plane cuts it, and each curve held. The same curve a thousandth off —
    /// its centre moved, or its size grown — is held by none of them
    /// (`docs/decisions.md` §222.2).
    #[test]
    fn a_curve_on_a_cone_a_torus_or_a_pipe_is_held_and_one_a_thousandth_off_is_not() {
        for (surface, curve) in curves_drawn_on_their_surfaces() {
            assert!(
                curve.held_by(&surface),
                "{surface:?} does not hold {curve:?}, which is drawn on it"
            );
            for off in [
                curve.moved(Vector::new(0.0, 0.0, 0.001)),
                curve.moved(Vector::new(0.001, 0.0, 0.0)),
                curve.grown(0.001),
            ] {
                assert!(
                    !off.held_by(&surface),
                    "{surface:?} holds {off:?}, a thousandth off the curve drawn on it"
                );
            }
        }
    }

    /// Every bound [`how_far_off_the_circle_can_be`] and
    /// [`how_far_off_the_ellipse_can_be`] publish for the cone, the torus and
    /// the ellipse on a pipe, held against the curve itself (§15.6): 720
    /// points round each curve, each point's distance from the surface worked
    /// out from the closed form for that surface, and none of them further than
    /// the bound.
    ///
    /// The curves are the ones drawn on their surfaces, moved, grown, tilted
    /// and turned by `1e-9` up to `0.3`, so every bound here is many orders of
    /// magnitude above what working a point out in `f64` costs. That cost is
    /// the four roundings of the largest number allowed on top of the bound,
    /// and it is the whole of the slack: a bound that fell short by more than
    /// a rounding would fail here.
    ///
    /// **The control is that each curve's bound is reached**: over its draws,
    /// some point comes within half of it, or a bound of infinity would pass
    /// this test as well as a good one.
    #[test]
    fn no_point_of_a_curve_is_further_off_its_surface_than_its_bound() {
        let mut drawn = 0;
        for (surface, curve) in curves_drawn_on_their_surfaces() {
            let mut the_most_of_its_bound_reached: f64 = 0.0;
            for size in [0.0, 1e-9, 1e-6, 1e-3, 0.3] {
                for off in [
                    curve.moved(Vector::new(size, -0.5 * size, 0.25 * size)),
                    curve.grown(size),
                    curve.grown(-size),
                    curve.tilted(size),
                    curve.turned(size),
                    curve
                        .moved(Vector::new(-size, size, size))
                        .tilted(size)
                        .grown(size),
                ] {
                    let (bound, largest) = off.bound_on(&surface).expect("a bound for each");
                    let furthest = off
                        .points(720)
                        .map(|point| distance_from(&surface, point))
                        .fold(0.0, f64::max);
                    assert!(
                        furthest <= bound + 4.0 * NEAREST_F64_POINT_ROUNDOFF * largest,
                        "{off:?} has a point {furthest} off {surface:?}, past its bound {bound}"
                    );
                    if size > 0.0 {
                        the_most_of_its_bound_reached =
                            the_most_of_its_bound_reached.max(furthest / bound);
                    }
                    drawn += 1;
                }
            }
            assert!(
                the_most_of_its_bound_reached >= 0.5,
                "no draw of {curve:?} on {surface:?} came within half its bound, the most being \
                 {the_most_of_its_bound_reached}"
            );
        }
        assert_eq!(
            drawn, 150,
            "the draw must not shrink without somebody saying so"
        );
    }

    /// A curve these tests draw, as the reader would hand it to a bound.
    #[derive(Clone, Copy, Debug)]
    enum Curve {
        Circle {
            centre: Point,
            axis: Vector,
            radius: f64,
        },
        Ellipse {
            centre: Point,
            first_way: Vector,
            second_way: Vector,
            semi_axes: [f64; 2],
        },
    }

    impl Curve {
        fn held_by(&self, surface: &Surface) -> bool {
            match *self {
                Curve::Circle {
                    centre,
                    axis,
                    radius,
                } => a_surface_holds_the_circle(surface, centre, axis, radius),
                Curve::Ellipse { .. } => a_surface_holds_the_ellipse(surface, &self.as_ellipse()),
            }
        }

        fn bound_on(&self, surface: &Surface) -> Option<(f64, f64)> {
            match *self {
                Curve::Circle {
                    centre,
                    axis,
                    radius,
                } => how_far_off_the_circle_can_be(surface, centre, axis, radius),
                Curve::Ellipse { .. } => {
                    how_far_off_the_ellipse_can_be(surface, &self.as_ellipse())
                }
            }
        }

        fn as_ellipse(&self) -> Ellipse {
            let Curve::Ellipse {
                centre,
                first_way,
                second_way,
                semi_axes,
            } = *self
            else {
                unreachable!("only an ellipse is asked for as one")
            };
            Ellipse {
                centre,
                first_way,
                second_way,
                semi_axes,
            }
        }

        fn moved(self, by: Vector) -> Self {
            match self {
                Curve::Circle {
                    centre,
                    axis,
                    radius,
                } => Curve::Circle {
                    centre: centre + by,
                    axis,
                    radius,
                },
                Curve::Ellipse {
                    centre,
                    first_way,
                    second_way,
                    semi_axes,
                } => Curve::Ellipse {
                    centre: centre + by,
                    first_way,
                    second_way,
                    semi_axes,
                },
            }
        }

        fn grown(self, by: f64) -> Self {
            match self {
                Curve::Circle {
                    centre,
                    axis,
                    radius,
                } => Curve::Circle {
                    centre,
                    axis,
                    radius: radius + by,
                },
                Curve::Ellipse {
                    centre,
                    first_way,
                    second_way,
                    semi_axes: [first, second],
                } => Curve::Ellipse {
                    centre,
                    first_way,
                    second_way,
                    semi_axes: [first + by, second + by],
                },
            }
        }

        /// The plane the curve lies in turned by `angle` about the curve's
        /// first direction across.
        fn tilted(self, angle: f64) -> Self {
            match self {
                Curve::Circle {
                    centre,
                    axis,
                    radius,
                } => {
                    let (_, and) = square_to(axis);
                    Curve::Circle {
                        centre,
                        axis: axis * angle.cos() + and * angle.sin(),
                        radius,
                    }
                }
                Curve::Ellipse {
                    centre,
                    first_way,
                    second_way,
                    semi_axes,
                } => {
                    let normal = first_way.cross(second_way);
                    Curve::Ellipse {
                        centre,
                        first_way,
                        second_way: second_way * angle.cos() + normal * angle.sin(),
                        semi_axes,
                    }
                }
            }
        }

        /// The curve turned by `angle` in its own plane: nothing for a circle,
        /// and the long direction of an ellipse swung round.
        fn turned(self, angle: f64) -> Self {
            match self {
                Curve::Circle { .. } => self,
                Curve::Ellipse {
                    centre,
                    first_way,
                    second_way,
                    semi_axes,
                } => Curve::Ellipse {
                    centre,
                    first_way: first_way * angle.cos() + second_way * angle.sin(),
                    second_way: second_way * angle.cos() - first_way * angle.sin(),
                    semi_axes,
                },
            }
        }

        fn points(self, how_many: u32) -> impl Iterator<Item = Point> {
            (0..how_many).map(move |step| {
                let angle = std::f64::consts::TAU * f64::from(step) / f64::from(how_many);
                match self {
                    Curve::Circle {
                        centre,
                        axis,
                        radius,
                    } => {
                        let (across, and) = square_to(axis);
                        round_the_circle(centre, across, and, radius, angle)
                    }
                    Curve::Ellipse {
                        centre,
                        first_way,
                        second_way,
                        semi_axes: [first, second],
                    } => {
                        centre
                            + first_way * (first * angle.cos())
                            + second_way * (second * angle.sin())
                    }
                }
            })
        }
    }

    /// Two unit directions square to `axis` and to each other.
    fn square_to(axis: Vector) -> (Vector, Vector) {
        let across = axis
            .cross(Vector::new(0.3, -0.7, 0.2))
            .normalized()
            .expect("no axis drawn here runs along (0.3, -0.7, 0.2)");
        (across, axis.cross(across))
    }

    /// How far `point` is from `surface`, from the closed form for each: a
    /// point `ρ` from a turned surface's axis and `z` along it is
    /// `|ρ·cos α − |z|·sin α|` from a cone and `|√((ρ − R)² + z²) − ρ_t|` from
    /// a torus, and `|ρ − R|` from a pipe.
    fn distance_from(surface: &Surface, point: Point) -> f64 {
        let (line, apart) = match surface {
            Surface::Cone(cone) => (cone.axis(), point - cone.apex()),
            Surface::Torus(ring) => (ring.axis(), point - ring.centre()),
            Surface::Cylinder(pipe) => (pipe.axis(), point - pipe.axis().origin()),
            other => unreachable!("no curve is drawn on {other:?} here"),
        };
        let along = apart.dot(line.direction());
        let across = (apart - line.direction() * along).length();
        match surface {
            Surface::Cone(cone) => {
                let hypotenuse = cone.slope().hypot(1.0);
                (across - along.abs() * cone.slope()).abs() / hypotenuse
            }
            Surface::Torus(ring) => {
                ((across - ring.ring_radius()).hypot(along) - ring.tube_radius()).abs()
            }
            Surface::Cylinder(pipe) => (across - pipe.radius()).abs(),
            other => unreachable!("no curve is drawn on {other:?} here"),
        }
    }

    /// A circle on each nappe of a cone, a circle square to a torus's axis and
    /// one round its tube, and an ellipse where a leaning plane cuts a pipe —
    /// each drawn from its surface's own numbers.
    fn curves_drawn_on_their_surfaces() -> Vec<(Surface, Curve)> {
        let cone = Cone::new(
            Axis::new(Point::new(1.0, -2.0, 0.5), Vector::new(0.3, -0.2, 1.0)).expect("an axis"),
            0.7,
        )
        .expect("a slope");
        let circle_at = |along: f64| Curve::Circle {
            centre: cone.apex() + cone.axis().direction() * along,
            axis: cone.axis().direction(),
            radius: cone.radius_at(along),
        };
        let ring = Torus::new(
            Axis::new(Point::new(0.5, 1.0, -1.0), Vector::new(0.1, 1.0, 0.2)).expect("an axis"),
            5.0,
            1.5,
        )
        .expect("two radii");
        let up = ring.axis().direction();
        let (out, _) = ring.basis();
        let over = 0.7_f64;
        let pipe = Cylinder::new(
            Axis::new(Point::new(1.0, 2.0, 3.0), Vector::new(0.2, 0.3, 1.0)).expect("an axis"),
            2.0,
        )
        .expect("a radius");
        let along_the_pipe = pipe.axis().direction();
        let (sideways, _) = square_to(along_the_pipe);
        let normal = (along_the_pipe + sideways * 0.8)
            .normalized()
            .expect("a leaning normal");
        let first_way = (along_the_pipe - normal * along_the_pipe.dot(normal))
            .normalized()
            .expect("the pipe does not run square to the plane");
        vec![
            (Surface::Cone(cone), circle_at(3.0)),
            (Surface::Cone(cone), circle_at(-2.0)),
            (
                Surface::Torus(ring),
                Curve::Circle {
                    centre: ring.centre() + up * (ring.tube_radius() * over.sin()),
                    axis: up,
                    radius: ring.distance_from_the_axis_at(over),
                },
            ),
            (
                Surface::Torus(ring),
                Curve::Circle {
                    centre: ring.centre() + out * ring.ring_radius(),
                    axis: up.cross(out),
                    radius: ring.tube_radius(),
                },
            ),
            (
                Surface::Cylinder(pipe),
                Curve::Ellipse {
                    centre: pipe.axis().origin() + along_the_pipe * 1.5,
                    first_way,
                    second_way: normal.cross(first_way),
                    semi_axes: [pipe.radius() / along_the_pipe.dot(normal), pipe.radius()],
                },
            ),
        ]
    }

    /// A point on the circle of `radius` about `centre`, at `angle` round the
    /// frame `(across, and)`.
    fn round_the_circle(
        centre: Point,
        across: Vector,
        and: Vector,
        radius: f64,
        angle: f64,
    ) -> Point {
        centre + across * (radius * angle.cos()) + and * (radius * angle.sin())
    }

    /// The claim [`Walk::the_arc_between`] rests on, drawn rather than argued.
    ///
    /// Writing `w` for the axis a circle turns about and `A`, `B` for two points
    /// on it, every point of the arc that runs from `A` to `B` the way the
    /// parameter grows is on the **positive** side of `(B − A) × w`. That is
    /// what says which of the two arcs an edge is without computing a point on
    /// either, and it is what an imported arc's picking half-space is built
    /// from.
    ///
    /// 6 480 draws: nine axes, four radii, six starting angles, six spans and
    /// five points along each arc. The spans stay clear of `0` and `2π` and the
    /// sampled points clear of the two ends, because the points here are worked
    /// out with `sin` and `cos` and a quantity a rounding wide would be a
    /// statement about `sin` rather than about the geometry.
    #[test]
    fn the_far_side_of_a_chord_is_where_the_arc_is() {
        let mut drawn = 0;
        for axis in [
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(0.0, 0.0, -1.0),
            Vector::new(1.0, 0.0, 0.0),
            Vector::new(0.0, 1.0, 0.0),
            Vector::new(1.0, 1.0, 1.0),
            Vector::new(-2.0, 1.0, 3.0),
            Vector::new(0.5, -0.25, 0.125),
            Vector::new(1e6, 1.0, -1.0),
            Vector::new(1e-6, 2.0, 5.0),
        ] {
            let along = Axis::new(Point::origin(), axis)
                .expect("a direction with length")
                .direction();
            let across = Axis::new(Point::origin(), along.cross(Vector::new(0.0, 1.0, 3.0)))
                .expect("no axis drawn here runs along (0, 1, 3)")
                .direction();
            let and = along.cross(across);
            let centre = Point::new(3.0, -7.5, 0.25);
            for radius in [1.0, 0.001, 1000.0, 12.7] {
                for start in 0..6 {
                    let start = f64::from(start) * 1.047;
                    for span in 1..=6 {
                        let span = f64::from(span) * 0.897;
                        let from = round_the_circle(centre, across, and, radius, start);
                        let to = round_the_circle(centre, across, and, radius, start + span);
                        let facing = (to - from).cross(axis);
                        for part in 1..=5 {
                            let along_the_arc = span * f64::from(part) / 6.0;
                            let at = round_the_circle(
                                centre,
                                across,
                                and,
                                radius,
                                start + along_the_arc,
                            );
                            assert!(
                                facing.dot(at - from) > 0.0,
                                "a point {along_the_arc} round an arc of {span} at radius \
                                 {radius} about {axis:?} came out on the wrong side"
                            );
                            drawn += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(
            drawn, 6480,
            "the draw must not shrink without somebody saying so"
        );
    }

    /// The bound [`a_half_space_holding_both`] publishes: eight epsilons of the
    /// dot product's own term magnitudes is enough that **neither** end of the
    /// chord is ever provably behind the half-space standing on it.
    ///
    /// `Model::validate` refuses an edge whose corner is provably outside
    /// something picking it, so an offset put exactly through one end lands on a
    /// coin-toss side of the other. This is the demonstration §15.6 asks a
    /// published bound for.
    ///
    /// The control is the same draw with the slack taken out, which **has** to
    /// put an end behind the surface somewhere or the bound is describing
    /// something that does not happen.
    #[test]
    fn a_chord_never_leaves_its_own_ends_outside() {
        let mut drawn = 0;
        let mut coin_tosses_lost = 0;
        for first in [0.1, 1.0 / 3.0, 12.7, 1e6, 1e-6] {
            for second in [0.7, 2.0 / 7.0, -25.4, 3e5, 7e-7] {
                for third in [-0.3, 1.0 / 11.0, 100.1, 1e7] {
                    let from = Point::new(first, second, third);
                    let to = Point::new(second, third, first);
                    let facing = (to - from).cross(Vector::new(third, first, second));
                    let Some(picking) = a_half_space_holding_both([from, to], facing) else {
                        continue;
                    };
                    for corner in [from, to] {
                        let at = PredicatePoint::from(corner.coordinates());
                        assert_ne!(
                            picking.side_of_point(at),
                            Decision::Neg,
                            "an end of the chord came out behind the half-space standing on it"
                        );
                        // The same half-space with the slack taken out: its
                        // offset is the dot product as `f64` rounded it, and the
                        // exact one is on whichever side the rounding fell.
                        let unslacked = rcad_geom::HalfSpace::through_a_point_facing(from, facing)
                            .expect("the same normal the picking one holds");
                        if unslacked.side_of_point(at) == Decision::Neg {
                            coin_tosses_lost += 1;
                        }
                        drawn += 1;
                    }
                }
            }
        }
        assert_eq!(
            drawn, 200,
            "the draw must not shrink without somebody saying so"
        );
        assert!(
            coin_tosses_lost > 0,
            "with no slack at all an end has to land behind the surface somewhere, or \
             the eight epsilons are paying for nothing"
        );
    }
}
