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
                [(back, Orientation::Reversed), (there, Orientation::Reversed)]
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

    /// The two halves a closed circle between a ball and another curved face
    /// is described as, or `None` for every other edge.
    ///
    /// **A closed edge has no room in the model for the surface that names its
    /// circle.** [`EdgeRun::RightRoundWhereItsFacesCross`] carries nothing, and
    /// `rcad-topo` reads that kind and no other as closed. So the run is named
    /// the way an arc already is — by the circle's plane and a half-space
    /// standing on its chord — and a circle with one corner has no chord. The
    /// second corner is put a quarter turn on from the first, `c + w × (A − c)`,
    /// which lies on the circle's plane and on every face the circle was
    /// checked against to within the rounding of one cross product. The two
    /// pieces run the way the edge does, `A` to the second corner and back.
    ///
    /// **Why a quarter turn and not the opposite point is measured, not
    /// argued.** On `kicad/LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step` the
    /// opposite point lands on the dome's longitude `π`, and the mesher refuses
    /// the dome by `AWireWillNotLieFlatOnItsFace`; a quarter turn gets the dome
    /// through. A quarter turn the other way is refused again, and the seven
    /// sibling files are refused on the dome whichever quarter is taken, so
    /// the mesher's wire walk depends on where this corner lands — that is
    /// `rcad-topo`'s to fix (§189.3).
    ///
    /// This is the one place the reader describes an edge the file wrote as
    /// one as two. It is limited to the circle [`Walk::a_circle_every_face_holds`]
    /// passes, where a ball and a curved face only touch and nothing but the
    /// file's circle draws where (`docs/decisions.md` §189.3).
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
        let a_ball_and_a_curved_face = self
            .surfaces_along_each_edge
            .get(&edge_curve)
            .is_some_and(|faces| a_ball_where_curved_faces_meet(faces).is_some());
        if !a_ball_and_a_curved_face {
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
        if !self.a_circle_every_face_holds(reading, edge_curve, curve, "CIRCLE", &place)? {
            return Ok(None);
        }
        let corner = reading.reference_at(edge_curve, "EDGE_CURVE", 1)?;
        let along_the_curve = reading.flag_at(edge_curve, "EDGE_CURVE", 4)?;
        let near = self.point_of_vertex(reading, corner)?;
        let Ok(axis) = place.axis.normalized() else {
            return Ok(None);
        };
        let far = place.origin + axis.cross(near - place.origin);
        let and = Surface::Plane(Plane::from_point_and_normal(place.origin, place.axis).map_err(
            |why| Refusal::TheGeometryIsNotAWorkingShape {
                id: edge_curve,
                why,
            },
        )?);
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
    /// goes right round is
    /// [`EdgeRun::RightRoundWhereItsFacesCross`]; anything less than right round
    /// is an arc, and an arc needs saying *which* arc — see
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
    fn round_run(
        &self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
        curve: EntityId,
        keyword: &'static str,
        ends: [EntityId; 2],
        along_the_curve: bool,
    ) -> Result<EdgeRun, Refusal> {
        if ends[0] == ends[1] {
            return Ok(EdgeRun::RightRoundWhereItsFacesCross);
        }
        let position = reading.reference_at(curve, keyword, 1)?;
        let place = reading.placement_at(curve, position)?;
        let corners = [
            self.point_of_vertex(reading, ends[0])?,
            self.point_of_vertex(reading, ends[1])?,
        ];
        let and =
            self.the_surface_that_names_the_curve(reading, edge_curve, curve, keyword, &place)?;
        self.the_arc_between(edge_curve, &place, and, corners, along_the_curve)
    }

    /// The surface an arc's run names beside the faces along it: the curve's
    /// own plane, or — for a circle every face along which is flat — the
    /// cylinder of the circle's radius about its axis.
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
    /// Where a curved face is along the edge the plane stays too: the walk
    /// reads the faces there, and a cylinder would change what a sphere's seam
    /// names from a great circle to a tangency.
    ///
    /// **Between a ball and another curved face the walk reads the plane**, so
    /// the plane is written only where the file's circle has been checked to
    /// lie on every face along it — [`Walk::a_circle_every_face_holds`]. Such a
    /// circle is on the ball and on the plane, and a plane crosses a ball in
    /// one circle, so `rcad_geom::walk` draws it from the pair rather than
    /// following two surfaces that only touch (`docs/decisions.md` §189.3).
    /// Where the check fails the run names the ball instead, which is one of
    /// its own two faces, and the walk reads the faces exactly as it did before.
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
        if let Some(ball) = faces_along_it.and_then(|faces| a_ball_where_curved_faces_meet(faces))
            && !self.a_circle_every_face_holds(reading, edge_curve, curve, keyword, place)?
        {
            return Ok(Surface::Sphere(ball));
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

    /// Whether the file's `CIRCLE` under this edge lies on the surface of every
    /// face along it, to within what writing the numbers down costs.
    ///
    /// **This is what lets the reader name a run by the file's circle rather
    /// than by its faces, and the file's word is not taken for it.** Each face's
    /// surface is asked in closed form, against the circle's centre `c`, unit
    /// axis `w` and radius `r` — see [`a_surface_holds_the_circle`] for the two
    /// bounds. A surface this does not have a bound for holds nothing, so the
    /// run keeps being named by its faces there.
    ///
    /// The allowance is the one `Model::validate` gives a corner of a curved
    /// face: sixteen times the rounding of writing the largest number involved
    /// down. It is not a modelling tolerance. On the KiCad LED body it passes a
    /// dome whose centre is the circle's own and a body whose axis the centre
    /// misses by `2.1e-16`, and it refuses a circle a thousandth of a unit off.
    fn a_circle_every_face_holds(
        &self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
        curve: EntityId,
        keyword: &'static str,
        place: &crate::geometry::Placement,
    ) -> Result<bool, Refusal> {
        if keyword != "CIRCLE" {
            return Ok(false);
        }
        let Some(faces) = self.surfaces_along_each_edge.get(&edge_curve) else {
            return Ok(false);
        };
        let radius = reading.real_at(curve, "CIRCLE", 2)?;
        let Ok(axis) = place.axis.normalized() else {
            return Ok(false);
        };
        Ok(faces
            .iter()
            .all(|face| a_surface_holds_the_circle(face, place.origin, axis, radius)))
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

/// The ball among the faces along an edge, when every face along it is curved
/// and they are not all one surface.
///
/// One surface on both sides is a seam, and a seam's run already names its
/// second surface for the walk to read.
fn a_ball_where_curved_faces_meet(faces: &[Surface]) -> Option<Sphere> {
    let all_curved = faces.len() >= 2 && faces.iter().all(|face| face.as_plane().is_none());
    let one_surface = faces.windows(2).all(|pair| pair[0] == pair[1]);
    if !all_curved || one_surface {
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
/// These are worked out in plain `f64`, and the allowance is sixteen times
/// the rounding of the largest number in them, so the few roundings each bound
/// costs cannot carry a circle that is off by the allowance back onto the
/// surface — only one that is off by that much less a few ulps.
///
/// Anything else holds nothing, which is the safe direction: the run is then
/// named by its faces as before.
fn a_surface_holds_the_circle(surface: &Surface, centre: Point, axis: Vector, radius: f64) -> bool {
    let allowance = |points: [Point; 2], lengths: [f64; 2]| {
        let largest = points
            .iter()
            .map(Point::largest_coordinate_magnitude)
            .chain(lengths.map(f64::abs))
            .fold(0.0, f64::max);
        16.0 * NEAREST_F64_POINT_ROUNDOFF * largest
    };
    match surface {
        Surface::Sphere(ball) => {
            let off = centre - ball.centre();
            let along = off.dot(axis);
            let across = (off - axis * along).length();
            let room = allowance([centre, ball.centre()], [radius, ball.radius()]);
            [radius + across, (radius - across).abs()]
                .into_iter()
                .all(|round| ((along * along + round * round).sqrt() - ball.radius()).abs() <= room)
        }
        Surface::Cylinder(pipe) => {
            let line = pipe.axis();
            let tilt = axis.cross(line.direction()).length();
            let across = line.distance_to_point(centre);
            let room = allowance([centre, line.origin()], [radius, pipe.radius()]);
            (radius - pipe.radius()).abs() + across + radius * tilt <= room
        }
        Surface::Plane(_) | Surface::Cone(_) | Surface::Torus(_) => false,
        // A surface kind this reader has no bound for holds nothing, which
        // names the run by its faces as before.
        _ => false,
    }
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
    use rcad_geom::{Axis, Cylinder, Point, Sphere, Surface, Vector};
    use rcad_predicates::{Decision, Point as PredicatePoint};

    use super::{a_half_space_holding_both, a_surface_holds_the_circle};

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
