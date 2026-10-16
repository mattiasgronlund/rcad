//! A `MANIFOLD_SOLID_BREP` becomes a solid, or is refused by the `#id` that
//! stopped it.
//!
//! The walk is the one an AP203 file is laid out in — closed shell, advanced
//! face, face bound, edge loop, oriented edge, edge curve, vertex point — and it
//! goes straight into a [`SolidBuilder`], which is the public door §42.5 said
//! the kernel did not have. Nothing is written into the caller's model until
//! the whole description has been through `Model::validate`.

use std::collections::{BTreeMap, HashMap};
use std::f64::consts::SQRT_2;

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, NEAREST_F64_POINT_ROUNDOFF, Orientation, Picker, Plane,
    Point, Sphere, Surface, Torus, Vector, which_two_surfaces_a_run_is_between,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface};
use rcad_topo::{
    DraftCoedge, DraftEdge, DraftFace, DraftStep, DraftVertex, Invalid, Model, SolidBuilder,
    SolidKey,
};
use smallvec::smallvec;

use crate::Value;
use crate::geometry::{Reading, plane_through, the_walk_centres_it_at};
use crate::graph::EntityId;
use crate::names::{
    ADVANCED_FACE, CIRCLE, CLOSED_SHELL, CONICAL_SURFACE, EDGE_CURVE, EDGE_LOOP, ELLIPSE,
    FACE_BOUND, FACE_OUTER_BOUND, FACE_SURFACE, Keyword, LINE, MANIFOLD_SOLID_BREP, ORIENTED_EDGE,
    VERTEX_LOOP, VERTEX_POINT,
};
use crate::refusal::Refusal;

/// Turn one `MANIFOLD_SOLID_BREP` into one solid in `model`.
pub(crate) fn solid_from(
    reading: &Reading<'_>,
    brep: EntityId,
    model: &mut Model,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SolidKey, Refusal> {
    let shell = reading.reference_at(brep, MANIFOLD_SOLID_BREP, 1)?;
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
    let shell = match reading.reference_at(brep, MANIFOLD_SOLID_BREP, 1) {
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
    /// The two corners of every edge described so far, start first, which is
    /// what [`with_a_pole_where_it_turns_back`] reads a turn off.
    ends: HashMap<DraftEdge, [DraftVertex; 2]>,
    /// Where every corner described so far is, kept as it is read rather than
    /// read again: [`Import::integers_read_as_reals`](crate::Import) counts
    /// every read.
    corner_points: HashMap<DraftVertex, Point>,
    vertex_names: Vec<EntityId>,
    edge_names: Vec<EntityId>,
    coedge_names: Vec<EntityId>,
    wire_names: Vec<EntityId>,
    face_names: Vec<EntityId>,
    face_surfaces: Vec<Surface>,
    /// The surfaces either side of each closed edge, filled in before any face
    /// of the shell is described — see [`Walk::find_the_closed_edges`].
    closed_edges: BTreeMap<EntityId, Vec<Surface>>,
    /// The run every closed edge was described with, which
    /// [`Walk::which_way_round_a_closed_edge_is_stored`] reads the two surfaces
    /// the edge is between off where the longitude says nothing.
    closed_runs: BTreeMap<EntityId, EdgeRun>,
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
            ends: HashMap::new(),
            corner_points: HashMap::new(),
            vertex_names: Vec::new(),
            edge_names: Vec::new(),
            coedge_names: Vec::new(),
            wire_names: Vec::new(),
            face_names: Vec::new(),
            face_surfaces: Vec::new(),
            closed_edges: BTreeMap::new(),
            closed_runs: BTreeMap::new(),
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
        let faces = reading.references_at(shell, CLOSED_SHELL, 1)?;
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
            let Ok(keyword) = reading.keyword_of(self.brep, *face, &[ADVANCED_FACE, FACE_SURFACE])
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
                let Ok(uses) = reading.references_at(loop_id, EDGE_LOOP, 1) else {
                    continue;
                };
                for use_of_an_edge in uses {
                    let Ok(edge_curve) = reading.reference_at(use_of_an_edge, ORIENTED_EDGE, 3)
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
        let keyword = reading.keyword_of(self.brep, face, &[ADVANCED_FACE, FACE_SURFACE])?;
        let (surface, used) = self.surface_of_the_face(reading, face, keyword)?;

        let bounds = self.the_bounds_that_bound_something(reading, face, keyword, &surface)?;
        let outer_first = self.outer_bound_first(reading, face, &bounds, &surface)?;
        let mut wires = Vec::with_capacity(outer_first.len());
        if let [only] = outer_first[..]
            && let Some((ball, loop_id)) =
                self.a_whole_ball_off_its_chart(reading, face, only, &surface)?
        {
            wires.push(self.describe_a_whole_ball(&ball, loop_id)?);
        } else {
            for bound in outer_first {
                wires.push(self.describe_the_bound(reading, face, bound, &surface)?);
            }
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
        keyword: Keyword,
    ) -> Result<(Surface, Orientation), Refusal> {
        let sits_on = reading.reference_at(face, keyword, 2)?;
        let written =
            reading.surface_at(face, sits_on, || corners_as_written(reading, face, keyword))?;
        let same_sense = reading.flag_at(face, keyword, 3)?;
        Ok(match (written.as_plane(), same_sense) {
            (_, true) => (written, Orientation::Forward),
            (Some(flat), false) => (Surface::Plane(flat.reversed()), Orientation::Forward),
            (None, false) => (written, Orientation::Reversed),
        })
    }

    /// A face's bounds, less a `VERTEX_LOOP` standing on the apex of the cone
    /// the face sits on.
    ///
    /// **This kernel's pointed cone has no wire at its apex**, so that is the
    /// one loop that is read as bounding nothing. A cone face that comes to a
    /// point is its base circle and nothing else: the mesher closes the face
    /// up at the apex the cone stores, and `Model::validate` asks nothing of a
    /// corner there because there is none. A file says the same face with a
    /// second bound, a `VERTEX_LOOP` of the apex, in 136 `CONICAL_SURFACE`
    /// faces of the NIST files (`nist_ctc_05` `#1340`; `docs/decisions.md`
    /// §376).
    ///
    /// **The corner has to be at the apex as `Model::validate` would judge a
    /// pole there**, because dropping the loop drops the corner and nothing
    /// after this sees it again. [`with_a_pole_where_it_turns_back`] can ask a
    /// ball exactly, with [`Surface::the_way_its_first_coordinate_runs_at`],
    /// because the pole it puts back is still a corner the validator checks
    /// against the pinch. On a cone that test answers `None` on the whole axis
    /// and not only at the apex, and 118 of the 136 corners are `2.5e3` to
    /// `1.0e6` roundings from the apex, 87 of them on the axis exactly and a
    /// little along it, which is a modelling tolerance and not where the cone
    /// pinches (§376). So the
    /// question asked is the validator's own for a pole —
    /// `stands_where_it_pinches`: is the apex inside the box of
    /// [`A_CIRCLE_MAY_MISS_ITS_FACE_BY`] roundings round the corner, the
    /// sixteen `Model::validate` gives a corner of a curved face. The rounding
    /// is of the numbers the apex is made from rather than of the whole
    /// model's largest coordinate, which the validator reads and this crate
    /// cannot, so for a cone that is not the solid's largest part this refuses
    /// where the validator might have passed. 18 of the 136 corners are inside
    /// it, the furthest `8.5` roundings out.
    ///
    /// Any other `VERTEX_LOOP` — on a face that is not a cone, or at a corner
    /// off its apex — is kept, and [`Walk::loop_of_bound`] refuses it by name.
    fn the_bounds_that_bound_something(
        &self,
        reading: &Reading<'_>,
        face: EntityId,
        keyword: Keyword,
        surface: &Surface,
    ) -> Result<Vec<EntityId>, Refusal> {
        let bounds = reading.references_at(face, keyword, 1)?;
        let Surface::Cone(cone) = surface else {
            return Ok(bounds);
        };
        let mut bounding = Vec::with_capacity(bounds.len());
        for bound in bounds {
            if !self.a_loop_standing_on_the_apex(reading, face, keyword, bound, cone)? {
                bounding.push(bound);
            }
        }
        Ok(bounding)
    }

    /// Whether `bound` holds a `VERTEX_LOOP` whose one corner is the apex of
    /// `cone` — see [`Walk::the_bounds_that_bound_something`].
    ///
    /// **The roundings are measured against the numbers the apex was worked
    /// out from**, and not against the apex alone. A file places a cone at its
    /// base and the reader moves back `radius / slope` along the axis to the
    /// apex, so a cone whose tip is at the origin has an apex a rounding of
    /// that length from zero, and a box sized by the apex's own coordinates
    /// would be no box at all. The radius is read off the file's
    /// `CONICAL_SURFACE` again for this, because [`Cone`](rcad_geom::Cone)
    /// keeps the apex and not where the file placed it.
    fn a_loop_standing_on_the_apex(
        &self,
        reading: &Reading<'_>,
        face: EntityId,
        face_keyword: Keyword,
        bound: EntityId,
        cone: &rcad_geom::Cone,
    ) -> Result<bool, Refusal> {
        let keyword = reading.keyword_of(face, bound, &[FACE_OUTER_BOUND, FACE_BOUND])?;
        let loop_id = reading.reference_at(bound, keyword, 1)?;
        let Some(corner) = reading
            .file()
            .instance(loop_id)
            .and_then(|held| held.part(VERTEX_LOOP.name()))
            .and_then(|record| record.attribute(1))
            .and_then(Value::as_reference)
        else {
            return Ok(false);
        };
        let at = self.point_of_vertex(reading, corner)?;
        let sits_on = reading.reference_at(face, face_keyword, 2)?;
        let back_to_the_apex = reading.real_at(sits_on, CONICAL_SURFACE, 2)? / cone.slope();
        Ok(a_corner_at_the_apex(at, cone.apex(), back_to_the_apex))
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
                .is_some_and(|held| held.part(FACE_OUTER_BOUND.name()).is_some())
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
        let mut oriented = reading.references_at(loop_id, EDGE_LOOP, 1)?;
        if walk_it_backwards {
            oriented.reverse();
        }
        let mut corners = Vec::with_capacity(oriented.len());
        let mut bulge = 0.0;
        for use_of_an_edge in oriented {
            let edge_curve = reading.reference_at(use_of_an_edge, ORIENTED_EDGE, 3)?;
            let forwards = reading.flag_at(use_of_an_edge, ORIENTED_EDGE, 4)? != walk_it_backwards;
            let ends = [
                reading.reference_at(edge_curve, EDGE_CURVE, 1)?,
                reading.reference_at(edge_curve, EDGE_CURVE, 2)?,
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
        let curve = reading.reference_at(edge_curve, EDGE_CURVE, 3)?;
        let Some(held) = reading.file().instance(curve) else {
            return Ok(0.0);
        };
        let (keyword, half_the_whole) = match Keyword::of(held.parts()[0].keyword()) {
            Some(CIRCLE) => {
                let radius = reading.real_at(curve, CIRCLE, 2)?;
                (CIRCLE, radius * radius / 2.0)
            }
            Some(ELLIPSE) => {
                let across = reading.real_at(curve, ELLIPSE, 2)?;
                let along = reading.real_at(curve, ELLIPSE, 3)?;
                (ELLIPSE, across * along / 2.0)
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
            reading.reference_at(edge_curve, EDGE_CURVE, 1)?,
            reading.reference_at(edge_curve, EDGE_CURVE, 2)?,
        ];
        let along_the_curve = reading.flag_at(edge_curve, EDGE_CURVE, 4)?;
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
        let keyword = reading.keyword_of(face, bound, &[FACE_OUTER_BOUND, FACE_BOUND])?;
        let loop_id = reading.reference_at(bound, keyword, 1)?;
        let held = reading
            .file()
            .instance(loop_id)
            .ok_or(Refusal::AReferencePointsNowhere {
                from: bound,
                to: loop_id,
            })?;
        if held.part(EDGE_LOOP.name()).is_none() {
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
        surface: &Surface,
    ) -> Result<rcad_topo::DraftWire, Refusal> {
        let (loop_id, walk_it_backwards) = self.loop_of_bound(reading, face, bound)?;
        let mut oriented = reading.references_at(loop_id, EDGE_LOOP, 1)?;
        if walk_it_backwards {
            oriented.reverse();
        }
        let mut walked = Vec::with_capacity(oriented.len());
        for use_of_an_edge in oriented {
            walked.extend(self.describe_the_coedge(reading, use_of_an_edge, walk_it_backwards)?);
        }
        let steps = with_a_pole_where_it_turns_back(&walked, surface);
        let wire = self
            .builder
            .add_wire(&steps)
            .map_err(|why| self.refused(loop_id, why))?;
        self.wire_names.push(loop_id);
        Ok(wire)
    }

    /// The ball a face covers whole, where the file cut it open along a slit
    /// whose two corners are not both where this kernel's ball pinches, with
    /// the loop that says so; `None` for every other bound.
    ///
    /// **A loop that walks one edge there and straight back bounds the whole
    /// ball**, wherever the edge runs on it: it encloses nothing, so the face
    /// is the ball less the slit. A ball turned off `z` comes as one (Q1010),
    /// and this kernel's [`Sphere`] pinches only on the `z` line through its
    /// centre, so the poles [`with_a_pole_where_it_turns_back`] would put at
    /// the file's corners would stand where the ball does not pinch. The ball
    /// is read on its own chart instead — see [`Walk::describe_a_whole_ball`].
    /// A loop whose two corners both stand on that line is the kernel's own
    /// seam, and is walked as the file wrote it.
    ///
    /// **The slit and its corners are dropped, so they are only checked, and
    /// to the digits the file was written in** (`docs/decisions.md` §404). The
    /// slit is a `CIRCLE` that lies on the ball and each corner is on the ball,
    /// both to [`A_SLIT_WRITTEN_IN_TWELVE_DIGITS_MAY_MISS_ITS_BALL_BY`]. A file
    /// whose slit misses its ball by more is walked as it was before, which
    /// refuses it. The corners are read ahead, so a file walked that way does
    /// not count an integer in them twice; the circle's numbers it does.
    ///
    /// **Where the ball is too small for its centre to hold two poles apart**
    /// — `z + r` or `z − r` rounds to `z` — there is no chart to read it on,
    /// and it is walked as before too. The writer refuses to write such a ball
    /// for the same reason.
    fn a_whole_ball_off_its_chart(
        &self,
        reading: &Reading<'_>,
        face: EntityId,
        bound: EntityId,
        surface: &Surface,
    ) -> Result<Option<(Sphere, EntityId)>, Refusal> {
        let Surface::Sphere(ball) = surface else {
            return Ok(None);
        };
        let (loop_id, _) = self.loop_of_bound(reading, face, bound)?;
        let uses = reading.references_at(loop_id, EDGE_LOOP, 1)?;
        let [there, back] = uses[..] else {
            return Ok(None);
        };
        let edge_curve = reading.reference_at(there, ORIENTED_EDGE, 3)?;
        if reading.reference_at(back, ORIENTED_EDGE, 3)? != edge_curve
            || reading.flag_at(there, ORIENTED_EDGE, 4)?
                == reading.flag_at(back, ORIENTED_EDGE, 4)?
            || an_edge_with_one_corner(reading, edge_curve)
        {
            return Ok(None);
        }
        let curve = reading.reference_at(edge_curve, EDGE_CURVE, 3)?;
        let Some(held) = reading.file().instance(curve) else {
            return Ok(None);
        };
        if held.parts()[0].keyword() != CIRCLE.name() {
            return Ok(None);
        }
        let corner = |position| -> Result<Point, Refusal> {
            let vertex_point = reading.reference_at(edge_curve, EDGE_CURVE, position)?;
            let at = reading.reference_at(vertex_point, VERTEX_POINT, 1)?;
            reading.point_read_ahead_at(vertex_point, at)
        };
        let corners = [corner(1)?, corner(2)?];
        if corners.iter().all(|corner| {
            surface
                .the_way_its_first_coordinate_runs_at(*corner)
                .is_none()
        }) || the_poles_of_a_ball_on_its_chart(ball).is_none()
        {
            return Ok(None);
        }
        let on_the_ball = |(off, largest): (f64, f64)| {
            off <= (A_CIRCLE_MAY_MISS_ITS_FACE_BY * NEAREST_F64_POINT_ROUNDOFF
                + A_SLIT_WRITTEN_IN_TWELVE_DIGITS_MAY_MISS_ITS_BALL_BY)
                * largest
        };
        if !corners.iter().all(|corner| {
            on_the_ball((
                ((*corner - ball.centre()).length() - ball.radius()).abs(),
                largest_of(&[*corner, ball.centre()], &[ball.radius()]),
            ))
        }) {
            return Ok(None);
        }
        let position = reading.reference_at(curve, CIRCLE, 1)?;
        let place = reading.placement_at(curve, position)?;
        let radius = reading.real_at(curve, CIRCLE, 2)?;
        let Ok(its_plane) = plane_through(place.origin, place.axis) else {
            return Ok(None);
        };
        if !how_far_off_the_circle_can_be(surface, place.origin, its_plane.normal(), radius)
            .is_some_and(on_the_ball)
        {
            return Ok(None);
        }
        Ok(Some((*ball, loop_id)))
    }

    /// The whole ball as `add_ball` builds it, on the chart this kernel's ball
    /// carries: poles at `centre ± (0, 0, r)` and the seam in the plane through
    /// the centre square to `y`, on the side facing `+x`. The file's own slit
    /// and its two corners are not read into the solid. It is the ball the
    /// writer puts in a file for a whole ball turned off `z`
    /// (`Writing::a_whole_ball_on_its_chart`), so a turned slit reads as the
    /// file this kernel would have written for it.
    fn describe_a_whole_ball(
        &mut self,
        ball: &Sphere,
        loop_id: EntityId,
    ) -> Result<rcad_topo::DraftWire, Refusal> {
        let centre = ball.centre();
        let not_a_working_shape = |why| Refusal::TheGeometryIsNotAWorkingShape { id: loop_id, why };
        let meridian = Plane::from_point_and_normal(centre, Vector::new(0.0, 1.0, 0.0))
            .map_err(not_a_working_shape)?;
        let facing_x = HalfSpace::through_a_point_facing(centre, Vector::new(1.0, 0.0, 0.0))
            .map_err(not_a_working_shape)?;
        let mut poles = Vec::with_capacity(2);
        for at in the_poles_of_a_ball_on_its_chart(ball)
            .expect("a_whole_ball_off_its_chart reads only a ball that holds two poles apart")
        {
            let pole = self
                .builder
                .add_vertex(at)
                .map_err(|why| self.refused(loop_id, why))?;
            self.vertex_names.push(loop_id);
            self.corner_points.insert(pole, at);
            poles.push(pole);
        }
        let (north, south) = (poles[0], poles[1]);
        let seam = self
            .builder
            .add_edge(
                north,
                south,
                EdgeRun::PartOfWhereItCrosses {
                    and: Surface::Plane(meridian),
                    picked_by: smallvec![Picker::from(facing_x)],
                },
            )
            .map_err(|why| self.refused(loop_id, why))?;
        self.edge_names.push(loop_id);
        self.ends.insert(seam, [north, south]);
        let mut steps = Vec::with_capacity(4);
        for (way, pole) in [
            (Orientation::Forward, south),
            (Orientation::Reversed, north),
        ] {
            let coedge = self
                .builder
                .add_coedge(seam, way)
                .map_err(|why| self.refused(loop_id, why))?;
            self.coedge_names.push(loop_id);
            steps.push(DraftStep::Along(coedge));
            steps.push(DraftStep::Pole(pole));
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
    ) -> Result<Vec<Walked>, Refusal> {
        let edge_curve = reading.reference_at(oriented, ORIENTED_EDGE, 3)?;
        let along_the_loop = reading.flag_at(oriented, ORIENTED_EDGE, 4)? != walk_it_backwards;
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
                coedges.push(self.walk_along(oriented, half, way)?);
            }
            return Ok(coedges);
        }
        let edge = self.describe_the_edge(reading, edge_curve)?;
        let forwards = self
            .which_way_round_a_closed_edge_is_stored(reading, edge_curve, along_the_loop)?
            .unwrap_or(along_the_loop);
        let way = if forwards {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        Ok(vec![self.walk_along(oriented, edge, way)?])
    }

    /// One coedge on `edge`, running `way`, with the corners it walks from and
    /// to.
    fn walk_along(
        &mut self,
        oriented: EntityId,
        edge: DraftEdge,
        way: Orientation,
    ) -> Result<Walked, Refusal> {
        let coedge = self
            .builder
            .add_coedge(edge, way)
            .map_err(|why| self.refused(oriented, why))?;
        self.coedge_names.push(oriented);
        let [start, end] = self.ends[&edge];
        let (from, to) = if way.is_reversed() {
            (end, start)
        } else {
            (start, end)
        };
        Ok(Walked {
            coedge,
            edge,
            from,
            to: (to, self.corner_points[&to]),
        })
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
        let corner = reading.reference_at(edge_curve, EDGE_CURVE, 1)?;
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
        let curve = reading.reference_at(edge_curve, EDGE_CURVE, 3)?;
        let Some(held) = reading.file().instance(curve) else {
            return Ok(None);
        };
        if held.parts()[0].keyword() != CIRCLE.name() {
            return Ok(None);
        }
        let position = reading.reference_at(curve, CIRCLE, 1)?;
        let place = reading.placement_at(curve, position)?;
        if a_ball_and_a_curved_face
            && !self.every_face_holds_the_curve(reading, edge_curve, curve, CIRCLE, &place)?
        {
            return Ok(None);
        }
        let along_the_curve = reading.flag_at(edge_curve, EDGE_CURVE, 4)?;
        if place.axis.normalized().is_err() {
            return Ok(None);
        }
        let far = place.origin - (near - place.origin);
        let and = Surface::Plane(plane_through(place.origin, place.axis).map_err(|why| {
            Refusal::TheGeometryIsNotAWorkingShape {
                id: edge_curve,
                why,
            }
        })?);
        let there = self.the_arc_between(edge_curve, &place, and, [near, far], along_the_curve)?;
        let back = self.the_arc_between(edge_curve, &place, and, [far, near], along_the_curve)?;

        let near = self.describe_the_vertex(reading, corner)?;
        let far_point = far;
        let far = self
            .builder
            .add_vertex(far_point)
            .map_err(|why| self.refused(edge_curve, why))?;
        self.vertex_names.push(edge_curve);
        self.corner_points.insert(far, far_point);
        let mut halves = Vec::with_capacity(2);
        for (from, to, run) in [(near, far, there), (far, near, back)] {
            let half = self
                .builder
                .add_edge(from, to, run)
                .map_err(|why| self.refused(edge_curve, why))?;
            self.edge_names.push(edge_curve);
            self.ends.insert(half, [from, to]);
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
    /// **A travel that dots to exactly zero against the winding is a circle
    /// round a torus's ring**, square at every corner to the winding, which runs
    /// round the tube. There the way the edge is stored is read the way
    /// `rcad-topo` reads it, by
    /// [`the_way_a_closed_edge_runs_where_its_longitude_says_nothing`]: the
    /// torus's latitude where its two surfaces cross, which is what
    /// `Model::validate` checks, and the mesher's own rule where a plane
    /// touches the torus, which `Model::validate` passes over. The disc of
    /// §229.4 with a round rim in two bands has one of each: its equator, and
    /// the two circles where its flat faces rest on the rim (§390.5).
    ///
    /// **Where neither has an answer the hint goes back to the file's own flag**
    /// rather than picking a side. Picking one is worse than useless: the two
    /// uses of a closed edge travel exact opposite ways, so a tie-break that
    /// answers the same for both is *provably* wrong for one of them, and
    /// `SolidBuilder` refuses the whole solid by
    /// `ADraftEdgeIsWalkedTheSameWayTwice`. `ORIENTED_EDGE.orientation` answers
    /// a different question (§115), so falling back to it is luck — but it is
    /// luck that at least tells the two uses apart, and `Model::validate` asks
    /// the certified question moments later and refuses by
    /// `AClosedEdgeIsWalkedAgainstItsSurface` when the luck runs out.
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
        let at = reading.reference_at(edge_curve, EDGE_CURVE, 1)?;
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
        let mut leaning = travel.dot(winding);
        if leaning == 0.0
            && let (Some(run), [one, other]) =
                (self.closed_runs.get(&edge_curve), carried_by.as_slice())
            && let Some(stored) = the_way_a_closed_edge_runs_where_its_longitude_says_nothing(
                run,
                [one, other],
                corner,
            )
        {
            leaning = travel.dot(stored);
        }
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
        let curve = reading.reference_at(edge_curve, EDGE_CURVE, 3)?;
        let Some(held) = reading.file().instance(curve) else {
            return Ok(None);
        };
        let keyword = match Keyword::of(held.parts()[0].keyword()) {
            Some(round @ (CIRCLE | ELLIPSE)) => round,
            _ => return Ok(None),
        };
        let position = reading.reference_at(curve, keyword, 1)?;
        let place = reading.placement_at(curve, position)?;
        let round = place.axis.cross(corner - place.origin);
        let along_the_curve = reading.flag_at(edge_curve, EDGE_CURVE, 4)?;
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
        let starts_at = reading.reference_at(edge_curve, EDGE_CURVE, 1)?;
        let ends_at = reading.reference_at(edge_curve, EDGE_CURVE, 2)?;
        let curve = reading.reference_at(edge_curve, EDGE_CURVE, 3)?;
        let along_the_curve = reading.flag_at(edge_curve, EDGE_CURVE, 4)?;

        let start = self.describe_the_vertex(reading, starts_at)?;
        let end = self.describe_the_vertex(reading, ends_at)?;
        let run = self.run_along(
            reading,
            edge_curve,
            curve,
            [starts_at, ends_at],
            along_the_curve,
        )?;
        if starts_at == ends_at {
            self.closed_runs.insert(edge_curve, run.clone());
        }
        let edge = self
            .builder
            .add_edge(start, end, run)
            .map_err(|why| self.refused(edge_curve, why))?;
        self.edge_names.push(edge_curve);
        self.edges.insert(edge_curve, edge);
        self.ends.insert(edge, [start, end]);
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
        match Keyword::of(keyword) {
            Some(LINE) => Ok(EdgeRun::Straight),
            Some(CIRCLE) => {
                self.round_run(reading, edge_curve, curve, CIRCLE, ends, along_the_curve)
            }
            Some(ELLIPSE) => {
                self.round_run(reading, edge_curve, curve, ELLIPSE, ends, along_the_curve)
            }
            _ => Err(Refusal::ACurveThisKernelDoesNotHold {
                edge: edge_curve,
                curve,
                keyword: keyword.to_owned(),
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
        keyword: Keyword,
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
        let mut arc = self.the_arc_between(edge_curve, &place, and, corners, along_the_curve)?;
        if let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = &mut arc
            && let Some(circle) =
                faces_along_it.and_then(|faces| which_of_a_rings_two_circles(faces, &and, corners))
        {
            picked_by.insert(0, circle);
        }
        Ok(arc)
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
    /// **Only where every flat face holds the file's circle**
    /// ([`Walk::every_face_holds_the_curve`]), and this used to write the
    /// cylinder without asking. The cylinder crosses each face's plane, not
    /// the circle's, so a disc tilted off the face round it named a circle on
    /// one plane and an ellipse on the other, and the solid's volume followed
    /// the order its faces were listed in (Q386, Q574). A circle a face does
    /// not hold keeps its own plane instead, and `Model::validate` refuses that
    /// by name, as it refuses the pair of planes this used to write.
    ///
    /// **An `ELLIPSE` keeps its plane**, and on flat faces is then refused by
    /// the mesher as an edge that bends with nothing naming it. No single
    /// surface this kernel holds crosses a plane in a given ellipse and is
    /// written down from the ellipse alone without choosing a slope for it — a
    /// tilted cylinder would do, and which tilt is a choice this reader has no
    /// consumer to make it for (§13).
    ///
    /// Where a flat face and a curved one are along the edge the plane stays
    /// too: the walk reads the faces there. An arc whose plane is square to a
    /// ring's axis also says which of the ring's two circles it is on — see
    /// [`which_of_a_rings_two_circles`]. Where every face is the one surface
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
    ///
    /// **The plane keeps the normal the file wrote**, through the function that
    /// reads a `PLANE` ([`plane_through`]). A ball turned by any angle has its
    /// seam cut from a plane whose normal the turn left an ulp off length one,
    /// and scaling it to length one again moved its last bit (Q822,
    /// `docs/decisions.md` §366).
    ///
    /// **The pipe a circle on flat faces is named by keeps the axis the file
    /// wrote** ([`Axis::as_written`]), as a `CYLINDRICAL_SURFACE` does, and the
    /// writer asks the same constructor whether the pipe reads back
    /// (`Writing::an_arc_of_a_circle_on_a_flat_face`). A disc drawn on a turned
    /// bar's top has that axis an ulp off length one, and [`Axis::new`] moved
    /// the last bit of 76 of 372 such axes (Q1001, §355.6).
    ///
    /// **That pipe runs through the circle's centre only where its walk
    /// centres the circle there.** `rcad_geom::walk` centres a circle by moving
    /// the pipe's own origin up its axis to each flat face, and from the
    /// circle's centre that second rounding can land a few ulps away: a disc on
    /// a bar's top at `z = 4` drawn by a pipe up `z` from `z = −15.9` is centred
    /// at `z = 3.9999999999999982`, and the pipe through there walks it to
    /// `4.0`. Where it does, the writer writes the model's own pipe beside the
    /// circle as a `CYLINDRICAL_SURFACE` nothing refers to, and the run takes
    /// the first pipe of the file's with the same direction and radius, to the
    /// bit, whose walk centres the circle on the centre the file wrote
    /// ([`the_walk_centres_it_at`], shared with the writer;
    /// `docs/decisions.md` §249.8). Where the file holds none, the pipe through
    /// the centre stays.
    fn the_surface_that_names_the_curve(
        &self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
        curve: EntityId,
        keyword: Keyword,
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
        if keyword == CIRCLE
            && only_flat_faces_along_it
            && self.every_face_holds_the_curve(reading, edge_curve, curve, keyword, place)?
        {
            let radius = reading.real_at(curve, CIRCLE, 2)?;
            let through_the_centre = Axis::as_written(place.origin, place.axis)
                .and_then(|axis| Cylinder::new(axis, radius))
                .map_err(not_a_working_shape)?;
            let faces: Vec<Plane> = faces_along_it
                .into_iter()
                .flatten()
                .filter_map(|face| face.as_plane().copied())
                .collect();
            let walks_it_round_the_centre =
                |pipe: &Cylinder| the_walk_centres_it_at(pipe, &faces, place.origin);
            if walks_it_round_the_centre(&through_the_centre) {
                return Ok(Surface::Cylinder(through_the_centre));
            }
            let bits = |way: Vector| way.components().map(f64::to_bits);
            let one_the_file_holds = reading.every_pipe_the_file_holds().iter().find(|pipe| {
                bits(pipe.axis().direction()) == bits(through_the_centre.axis().direction())
                    && pipe.radius().to_bits() == radius.to_bits()
                    && walks_it_round_the_centre(pipe)
            });
            return Ok(Surface::Cylinder(
                one_the_file_holds.copied().unwrap_or(through_the_centre),
            ));
        }
        let flat = plane_through(place.origin, place.axis).map_err(not_a_working_shape)?;
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
    ///
    /// **An `ELLIPSE`'s two sizes are worked out from the first face's pipe,
    /// not read from the file**, and that one ellipse is held against every
    /// face. A plane cuts a pipe of radius `R` about the unit axis `d` in the
    /// ellipse whose size along each of its two ways `w` is `R / |w × d|`, and
    /// that is the curve the walk will draw: the sizes never reach the model,
    /// which reads only the plane. So what has to hold is that this one ellipse
    /// lies on every face, which also holds one pipe's radius and lean against
    /// the other's. The file's own sizes cannot be held to that: KiCad writes
    /// twelve digits, and the `0.141421356237` its exporter writes for `0.1·√2`
    /// is `3.1e-13` short, dozens of roundings of the relay it is in
    /// (`docs/decisions.md` §335 has the measurement).
    ///
    /// **The file's sizes are still checked**, so a file whose sizes are
    /// plainly wrong is named by its faces, as before, and not read as the
    /// curve the plane cuts. Its ellipse is held against every face to the
    /// same allowance plus what writing a size in twelve significant digits
    /// costs: half a unit in the twelfth digit is at most five parts in `10¹²`
    /// of the size. No ellipse in either corpus that the pipe's ellipse fits
    /// fails this, and an eleven-digit size does.
    fn every_face_holds_the_curve(
        &self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
        curve: EntityId,
        keyword: Keyword,
        place: &crate::geometry::Placement,
    ) -> Result<bool, Refusal> {
        const A_SIZE_WRITTEN_IN_TWELVE_DIGITS_MAY_BE_OFF_BY: f64 = 5e-12;
        let Some(faces) = self.surfaces_along_each_edge.get(&edge_curve) else {
            return Ok(false);
        };
        let Ok(its_plane) = plane_through(place.origin, place.axis) else {
            return Ok(false);
        };
        let normal = its_plane.normal();
        match keyword {
            CIRCLE => {
                let radius = reading.real_at(curve, CIRCLE, 2)?;
                Ok(faces
                    .iter()
                    .all(|face| a_surface_holds_the_circle(face, place.origin, normal, radius)))
            }
            ELLIPSE => {
                let Some(reference) = place.reference else {
                    return Ok(false);
                };
                let Ok(first_way) = (reference - normal * reference.dot(normal)).normalized()
                else {
                    return Ok(false);
                };
                let second_way = normal.cross(first_way);
                let Some(Surface::Cylinder(pipe)) = faces.first() else {
                    return Ok(false);
                };
                let from_the_pipe = [first_way, second_way]
                    .map(|way| pipe.radius() / way.cross(pipe.axis().direction()).length());
                let written = [
                    reading.real_at(curve, ELLIPSE, 2)?,
                    reading.real_at(curve, ELLIPSE, 3)?,
                ];
                // A plane along the pipe's axis cuts it in two lines, which
                // gives an infinite size; and `largest_of` takes the sizes in,
                // so one infinite size makes the allowance infinite too.
                if !from_the_pipe
                    .iter()
                    .chain(&written)
                    .all(|size| size.is_finite())
                {
                    return Ok(false);
                }
                let an_ellipse = |semi_axes| Ellipse {
                    centre: place.origin,
                    first_way,
                    second_way,
                    semi_axes,
                };
                let (drawn, as_written) = (an_ellipse(from_the_pipe), an_ellipse(written));
                let writing_the_sizes = A_SIZE_WRITTEN_IN_TWELVE_DIGITS_MAY_BE_OFF_BY
                    * from_the_pipe[0].max(from_the_pipe[1]);
                Ok(faces.iter().all(|face| {
                    a_surface_holds_the_ellipse(face, &drawn)
                        && how_far_off_the_ellipse_can_be(face, &as_written).is_some_and(
                            |(off, largest)| {
                                off <= A_CIRCLE_MAY_MISS_ITS_FACE_BY
                                    * NEAREST_F64_POINT_ROUNDOFF
                                    * largest
                                    + writing_the_sizes
                            },
                        )
                }))
            }
            other => unreachable!(
                "round_run is only handed a CIRCLE or an ELLIPSE, not {}",
                other.name()
            ),
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
    ///
    /// Where the half-space stands, along `(B − A) × w`, is
    /// [`a_half_space_naming_the_arc`]'s: an arc longer than half its curve
    /// needs one a fuse can read that a shorter arc does not.
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
        let picking = a_half_space_naming_the_arc(corners, facing, place.origin)
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
        self.corner_points.insert(drafted, point);
        Ok(drafted)
    }

    fn point_of_vertex(
        &self,
        reading: &Reading<'_>,
        vertex_point: EntityId,
    ) -> Result<Point, Refusal> {
        let at = reading.reference_at(vertex_point, VERTEX_POINT, 1)?;
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
            let Ok(corners) = self.corners_of_face(reading, *face, surface) else {
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
    /// surface. The second surface is the plane the circle lies in, built
    /// as the run's plane is ([`plane_through`]), which rounds its offset, so
    /// a corner is exactly on it only where the numbers cancel.
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
            reading.reference_at(edge_curve, EDGE_CURVE, 1).ok()?,
            reading.reference_at(edge_curve, EDGE_CURVE, 2).ok()?,
        ];
        if ends[0] == ends[1] {
            return None;
        }
        let curve = reading.reference_at(edge_curve, EDGE_CURVE, 3).ok()?;
        let keyword = match Keyword::of(reading.file().instance(curve)?.parts()[0].keyword()) {
            Some(round @ (CIRCLE | ELLIPSE)) => round,
            _ => return None,
        };
        let position = reading.reference_at(curve, keyword, 1).ok()?;
        let place = reading.placement_at(curve, position).ok()?;
        let flat = plane_through(place.origin, place.axis).ok()?;
        Some((
            flat,
            [
                (ends[0], self.point_of_vertex(reading, ends[0]).ok()?),
                (ends[1], self.point_of_vertex(reading, ends[1]).ok()?),
            ],
        ))
    }

    /// Every corner of one face, with the `VERTEX_POINT` it was written under.
    ///
    /// A cone's apex loop is passed over, the way the walk passed it over: it
    /// put no corner into the description for the sweep to have refused.
    fn corners_of_face(
        &self,
        reading: &Reading<'_>,
        face: EntityId,
        surface: &Surface,
    ) -> Result<Vec<(EntityId, Point)>, Refusal> {
        let keyword = reading.keyword_of(self.brep, face, &[ADVANCED_FACE, FACE_SURFACE])?;
        let mut corners = Vec::new();
        for bound in self.the_bounds_that_bound_something(reading, face, keyword, surface)? {
            let (loop_id, _) = self.loop_of_bound(reading, face, bound)?;
            for use_of_an_edge in reading.references_at(loop_id, EDGE_LOOP, 1)? {
                let edge_curve = reading.reference_at(use_of_an_edge, ORIENTED_EDGE, 3)?;
                for position in 1..=2 {
                    let at = reading.reference_at(edge_curve, EDGE_CURVE, position)?;
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
        let faces = reading.references_at(shell, CLOSED_SHELL, 1)?;
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
        let keyword =
            self.record(reading.keyword_of(self.walk.brep, face, &[ADVANCED_FACE, FACE_SURFACE]))?;
        let sits_on = self.record(reading.reference_at(face, keyword, 2))?;
        let written = self.record(
            reading.surface_at(face, sits_on, || corners_as_written(reading, face, keyword)),
        )?;
        let same_sense = self.record(reading.flag_at(face, keyword, 3))?;
        let (surface, used) = match (written.as_plane(), same_sense) {
            (_, true) => (written, Orientation::Forward),
            (Some(flat), false) => (Surface::Plane(flat.reversed()), Orientation::Forward),
            (None, false) => (written, Orientation::Reversed),
        };

        let bounds = self.record(
            self.walk
                .the_bounds_that_bound_something(reading, face, keyword, &surface),
        )?;
        let outer_first = self.record(
            self.walk
                .outer_bound_first(reading, face, &bounds, &surface),
        )?;
        let bound_count = outer_first.len();
        let mut wires = Vec::with_capacity(bound_count);
        let whole = match outer_first[..] {
            [only] => self.record(
                self.walk
                    .a_whole_ball_off_its_chart(reading, face, only, &surface),
            )?,
            _ => None,
        };
        if let Some((ball, loop_id)) = whole {
            let described = self.walk.describe_a_whole_ball(&ball, loop_id);
            wires.push(self.record(described)?);
        } else {
            for bound in outer_first {
                if let Some(wire) = self.probe_the_bound(reading, face, bound, &surface) {
                    wires.push(wire);
                }
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
        surface: &Surface,
    ) -> Option<rcad_topo::DraftWire> {
        let (loop_id, walk_it_backwards) =
            self.record(self.walk.loop_of_bound(reading, face, bound))?;
        let mut oriented = self.record(reading.references_at(loop_id, EDGE_LOOP, 1))?;
        if walk_it_backwards {
            oriented.reverse();
        }
        let mut walked = Vec::with_capacity(oriented.len());
        let mut every_edge_described = true;
        for use_of_an_edge in oriented {
            match self
                .walk
                .describe_the_coedge(reading, use_of_an_edge, walk_it_backwards)
            {
                Ok(coedges) => walked.extend(coedges),
                Err(why) => {
                    self.refusals.push(why);
                    every_edge_described = false;
                }
            }
        }
        if !every_edge_described {
            return None;
        }
        match self
            .walk
            .builder
            .add_wire(&with_a_pole_where_it_turns_back(&walked, surface))
        {
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

/// Every corner of a face as the file wrote it — each end of each edge of its
/// `EDGE_LOOP`s, and the one corner of a `VERTEX_LOOP` — which a cone's slope
/// is put back from where the file cut its half-angle (`cone_from`, in
/// `geometry.rs`).
///
/// Read before the face is described, so it reads the file and not the walk,
/// and it counts no integer it meets ([`Reading::point_read_ahead_at`]). `None`
/// where a bound, a loop or a corner does not read, or a loop is neither kind:
/// describing the face refuses it by name a moment later, and a slope from the
/// corners that did read would be a slope from part of the face.
///
/// **The corner of a `VERTEX_LOOP` counts**, though the walk drops it where
/// it stands on the apex ([`Walk::the_bounds_that_bound_something`]). On a
/// cone placed with its radius on the cap, as `a_pointed_cone_with_its_apex_written_as_a_loop`
/// is, the apex is the one corner off the placement's plane, and it says the
/// slope; a cone placed at its apex finds it on that plane, where it says
/// nothing and is passed over.
fn corners_as_written(
    reading: &Reading<'_>,
    face: EntityId,
    keyword: Keyword,
) -> Option<Vec<Point>> {
    let mut vertices = Vec::new();
    for bound in reading.references_at(face, keyword, 1).ok()? {
        let bound_keyword = reading
            .keyword_of(face, bound, &[FACE_OUTER_BOUND, FACE_BOUND])
            .ok()?;
        let loop_id = reading.reference_at(bound, bound_keyword, 1).ok()?;
        if let Some(record) = reading.file().instance(loop_id)?.part(VERTEX_LOOP.name()) {
            vertices.push(record.attribute(1)?.as_reference()?);
            continue;
        }
        for use_of_an_edge in reading.references_at(loop_id, EDGE_LOOP, 1).ok()? {
            let edge_curve = reading
                .reference_at(use_of_an_edge, ORIENTED_EDGE, 3)
                .ok()?;
            for position in 1..=2 {
                vertices.push(
                    reading
                        .reference_at(edge_curve, EDGE_CURVE, position)
                        .ok()?,
                );
            }
        }
    }
    vertices
        .into_iter()
        .map(|vertex| {
            let at = reading.reference_at(vertex, VERTEX_POINT, 1).ok()?;
            reading.point_read_ahead_at(vertex, at).ok()
        })
        .collect()
}

/// One coedge as a loop walks it, with the corners it goes from and to, and
/// where the second one is.
#[derive(Clone, Copy, Debug)]
struct Walked {
    coedge: DraftCoedge,
    edge: DraftEdge,
    from: DraftVertex,
    to: (DraftVertex, Point),
}

/// A loop's coedges as the steps of a wire on `surface`, **with a pole
/// wherever the loop turns back along the edge it has just walked at a corner
/// where the surface has no way round**.
///
/// That is how a real file writes a ball's pole, and how this crate's writer
/// writes one: the loop runs up the seam to the pole and straight back down it,
/// and the pole itself is left unwritten (`kicad/LED_D5.0mm.step` `#17`).
/// `SolidBuilder` takes a pole as a step of its own, [`DraftStep::Pole`], and
/// refuses a whole ball's two steps of seam and back as a wire too short to
/// bound anything (`ADraftWireIsTooShort { steps: 2 }`).
///
/// **The corner has to be exactly where the surface has no way round**
/// ([`Surface::the_way_its_first_coordinate_runs_at`] answers `None`), the
/// same test [`Walk::describe_the_halves_of_a_circle`] reads a pole with. A
/// turn anywhere else is left as the file wrote it, which is what this reader
/// did with every turn before: the horizontal KiCad LEDs turn back along a
/// dome's seam at a corner their ball, which this kernel pinches only on the
/// `z` line through its centre, does not pinch at, and they import with no
/// pole there. A pole put at that corner is one `Model::validate` refuses as
/// `APoleStandsWhereTheSurfaceDoesNotPinch`, and 20 pinned files stopped
/// importing when this asked nothing of the corner. The upright
/// `LED_D5.0mm.step` writes its pole `1.6e-15` off that line, so it gets no
/// pole either, and imports as it did.
///
/// **A loop that is nothing but one edge walked there and back is not walked
/// here** unless both its corners stand on that line: it is a whole ball, cut
/// open along a slit, and [`Walk::a_whole_ball_off_its_chart`] reads it on the
/// ball's own chart instead (`docs/decisions.md` §404). So what is left to
/// turn back at a corner the ball does not pinch at is a part of a ball, as
/// the domes above are.
///
/// An edge with one corner is never turned at: it has no far end to come back
/// from.
fn with_a_pole_where_it_turns_back(walked: &[Walked], surface: &Surface) -> Vec<DraftStep> {
    let mut steps = Vec::with_capacity(walked.len() + 2);
    for (position, there) in walked.iter().enumerate() {
        steps.push(DraftStep::Along(there.coedge));
        let back = walked[(position + 1) % walked.len()];
        let (corner, at) = there.to;
        if back.edge == there.edge
            && there.from != corner
            && back.from == corner
            && surface.the_way_its_first_coordinate_runs_at(at).is_none()
        {
            steps.push(DraftStep::Pole(corner));
        }
    }
    steps
}

/// The two poles of `ball` where this kernel's ball pinches, north first:
/// `centre ± (0, 0, r)`, which is where `add_ball` puts them. `None` where
/// `z + r` or `z − r` rounds to `z`, so that a pole would stand on the centre
/// and not on the ball.
pub(crate) fn the_poles_of_a_ball_on_its_chart(ball: &Sphere) -> Option<[Point; 2]> {
    let [x, y, z] = ball.centre().coordinates();
    let [north, south] = [z + ball.radius(), z - ball.radius()];
    (north != z && south != z).then(|| [Point::new(x, y, north), Point::new(x, y, south)])
}

/// How far, in parts of the largest number involved, a slit cutting a whole
/// ball open, and each of its two corners, may sit off the ball beyond
/// [`A_CIRCLE_MAY_MISS_ITS_FACE_BY`] roundings and still be read as on it —
/// see [`Walk::a_whole_ball_off_its_chart`].
///
/// **It is what writing the numbers in twelve significant digits costs**, the
/// allowance [`Walk::every_face_holds_the_curve`] already gives an
/// `ELLIPSE`'s sizes, and for the same reason: the numbers never reach the
/// model, so the check only tells a slit on the ball from a file that is
/// wrong, and a ball turned off `z` puts its corners at irrational
/// coordinates a file writes rounded. Each number is off by at most half a
/// unit in its twelfth digit, `5e-12` of the largest. A corner's distance off
/// the ball is moved by at most `√3` of that for the corner, `√3` for the
/// centre and one for the radius; a circle's by `√3` for each of the two
/// centres and one for each of the two radii, `2√3 + 2 < 5.5` in all, and
/// six of them is this. A slit a thousandth of its radius off is refused
/// (`docs/decisions.md` §404).
const A_SLIT_WRITTEN_IN_TWELVE_DIGITS_MAY_MISS_ITS_BALL_BY: f64 = 6.0 * 5e-12;

/// Whether the apex is inside the box of [`A_CIRCLE_MAY_MISS_ITS_FACE_BY`]
/// roundings round `corner`, a rounding being [`NEAREST_F64_POINT_ROUNDOFF`]
/// of the largest of the two points' coordinates and `back_to_the_apex`, how
/// far the apex was worked out from where the file placed the cone — see
/// [`Walk::the_bounds_that_bound_something`].
///
/// This is the box test `rcad-topo`'s `provably_away_from_every_pinch` makes
/// with intervals, made in `f64`: a subtraction here rounds only by a part of
/// its own size, so the two can disagree only about a corner within a rounding
/// of the edge of the box.
fn a_corner_at_the_apex(corner: Point, apex: Point, back_to_the_apex: f64) -> bool {
    let reach = A_CIRCLE_MAY_MISS_ITS_FACE_BY
        * NEAREST_F64_POINT_ROUNDOFF
        * largest_of(&[corner, apex], &[back_to_the_apex]);
    (corner - apex)
        .components()
        .iter()
        .all(|apart| apart.abs() <= reach)
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
            reading.reference_at(edge_curve, EDGE_CURVE, 1),
            reading.reference_at(edge_curve, EDGE_CURVE, 2),
        ),
        (Ok(start), Ok(end)) if start == end
    )
}

/// Which way a closed edge round a torus's ring is stored running, read the
/// way `rcad-topo` reads it, or `None` where that reads nothing here.
///
/// **This mirrors `mesh.rs`'s `the_way_a_closed_edge_is_stored_running`**,
/// which `rcad-step` cannot call, and through it `Model::validate`'s rule
/// (`a_closed_crossing_is_walked_the_way_it_is_stored`). The two surfaces are
/// the ones [`which_two_surfaces_a_run_is_between`] reads off the run, so for
/// the equator between two bands of one torus they are the torus and the
/// equator's own plane, not the two bands. The edge runs along `∇S × ∇T`,
/// turned round where that runs against the torus's longitude `L`; where the
/// dot is exactly zero, as it is all the way round a circle round the ring,
/// against the second reference, the gradient crossed into the first:
/// `∇C × L`. At a torus's outer equator that runs down the ring angle.
///
/// **Where the two surfaces touch rather than cross, `∇S × ∇T` is nothing**
/// and `Model::validate` passes the edge over. The mesher still puts its points
/// in an order, and a face that walks them the other way is refused
/// `AWireWillNotLieFlatOnItsFace`. That order is
/// [`the_way_a_circle_where_a_plane_touches_a_torus_runs`].
///
/// Only a torus is read: the curved one of any other pair has a longitude that
/// is not square to its closed circles, so the caller never gets here, or
/// gets here at a corner this has not been measured on, and keeps the file's
/// flag there.
fn the_way_a_closed_edge_runs_where_its_longitude_says_nothing(
    run: &EdgeRun,
    faces: [&Surface; 2],
    at: Point,
) -> Option<Vector> {
    let pair = which_two_surfaces_a_run_is_between(run, faces);
    let curved = the_curved_one_of(&pair)?;
    let Surface::Torus(ring) = curved else {
        return None;
    };
    let longitude = curved.the_way_its_first_coordinate_runs_at(at)?;
    let gradient = gradient_of(&curved, at)?;
    let forward = gradient_of(&pair[0], at)?.cross(gradient_of(&pair[1], at)?);
    let with_the_longitude = forward.dot(longitude);
    let against_the_frame = if with_the_longitude == 0.0 {
        forward.dot(gradient.cross(longitude))
    } else {
        with_the_longitude
    };
    if against_the_frame > 0.0 {
        Some(forward)
    } else if against_the_frame < 0.0 {
        Some(-forward)
    } else {
        the_way_a_circle_where_a_plane_touches_a_torus_runs(&pair, &ring, at)
    }
}

/// Which way the mesher puts the points of a circle where a plane rests on a
/// torus, round the top or the bottom of its tube, or `None` for any other
/// pair.
///
/// **This mirrors `mesh.rs`'s `the_way_a_closed_edge_where_its_faces_touch_runs`
/// together with what the mesher does with its answer**, neither of which
/// `rcad-step` can call. That function hands back the torus's longitude `L`,
/// and the mesher turns the points `rcad_geom::walk` drew round where the
/// first of them lies against `L` from the corner. `L` runs round the tube,
/// square to this circle, so it is the circle's bend that decides and not its
/// first step: the first point lies towards the circle's centre whichever way
/// it is drawn. The walk draws a circle round a torus's axis the way a right
/// hand turns about that axis (`circles_round_the_axis`, from
/// `Torus::basis`). So the drawn order is kept where `L` points towards the
/// centre, which is round the top of the tube, and turned round where it
/// points away, round the bottom.
fn the_way_a_circle_where_a_plane_touches_a_torus_runs(
    pair: &[Surface; 2],
    ring: &Torus,
    at: Point,
) -> Option<Vector> {
    let flat = pair.iter().find_map(Surface::as_plane)?;
    let along = ring.axis().direction();
    if flat.normal().cross(along) != Vector::zero() {
        return None;
    }
    let out = at - ring.centre();
    let centre = ring.centre() + along * (out.dot(along) / along.dot(along));
    let drawn = along.cross(at - centre);
    let longitude = Surface::Torus(*ring).the_way_its_first_coordinate_runs_at(at)?;
    Some(if (centre - at).dot(longitude) < 0.0 {
        -drawn
    } else {
        drawn
    })
}

/// Which way a plane or a torus faces at `at`, or `None` for any other surface.
///
/// **Spelled exactly as `rcad-topo`'s `validate.rs` spells `gradient_of`**,
/// the bulge term included, so that a sign read off it here is the sign
/// `Model::validate` reads: `(w·w)(x·x + R² − r² − 2R²)x + 2R²(x·w)w` for
/// `x = at − centre`, the quartic's gradient divided by four.
fn gradient_of(surface: &Surface, at: Point) -> Option<Vector> {
    if let Some(flat) = surface.as_plane() {
        return Some(flat.normal());
    }
    let Surface::Torus(ring) = surface else {
        return None;
    };
    let (centre, along) = (ring.centre(), ring.axis().direction());
    let out = at - centre;
    let (big, small) = (ring.ring_radius(), ring.tube_radius());
    let bulge = out.dot(out) + big * big - small * small - 2.0 * big * big;
    Some(out * (along.dot(along) * bulge) + along * (2.0 * big * big * along.dot(out)))
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
pub(crate) fn the_curved_one_of(carried_by: &[Surface]) -> Option<Surface> {
    carried_by
        .iter()
        .filter(|surface| surface.as_plane().is_none())
        .copied()
        .min_by(Surface::how_it_sorts_against)
}

/// The face a run names where every face along it is curved and they are not
/// all one surface: [`the_curved_one_of`] them, the same pick a run's second
/// surface makes when the file writes its own circle apart from every face
/// (`Walk::every_face_holds_the_curve` refuses it).
///
/// **Not `faces[0]`.** A run's stored second surface used to be the ball if
/// there was one and otherwise the first face, so which face it named — Cone
/// against Cylinder, one ball or the other — depended on the order the file's
/// faces happened to be listed in (Q541, and Q388 before it for two balls).
/// `the_curved_one_of` sorts on the faces' own numbers instead, so the pick no
/// longer depends on that order.
fn a_face_where_curved_faces_meet(faces: &[Surface]) -> Option<Surface> {
    let all_curved = faces.len() >= 2 && faces.iter().all(|face| face.as_plane().is_none());
    if !all_curved || the_faces_along_it_name_no_crossing(faces) {
        return None;
    }
    the_curved_one_of(faces)
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

/// The picker that says which of a ring's two circles round its axis an arc
/// is on, where the plane the arc is named by is square to that axis and cuts
/// the tube in two; `None` everywhere else.
///
/// **A half-space on the chord does not tell the two apart.** A plane square to
/// a ring's axis, nearer its centre than the tube's radius, crosses it in two
/// circles about the axis, one inside the other, and a half-space that holds
/// an arc of the one holds part of the other too. `rcad_geom`'s own crossing of
/// a plane and a ring tells them apart by the pipe of the ring's own radius
/// about its axis, outside it for the outer circle and inside it for the inner
/// one (`two_circles_round_the_axis` in `crates/rcad-geom/src/intersect.rs`),
/// and a bend built in the kernel names its seam along the inner equator that
/// way. Read without it, that seam came back picked by the half-space alone,
/// a run the kernel's own rule calls unnamed (Q407).
///
/// **Square is asked the way the walk asks it**, as the cross product of the
/// plane's normal and the axis coming out exactly nothing, which is also how
/// [`how_far_off_the_circle_can_be`] tells a circle round the axis from one
/// round the tube. **Which circle is read off the corners**, as the side of
/// the ring's radius both are on: the two circles are `√(r² − h²)` either side
/// of it, so a corner on one is that far from the pipe. A plane resting on the
/// tube touches it in the one circle of the ring's own radius, and a corner
/// that is not further off that circle than the reader lets a circle miss its
/// face ([`A_CIRCLE_MAY_MISS_ITS_FACE_BY`]) is read as on it, with no picker
/// added. Without that room, `Relay_DPST_Fujitsu_FTR-F1A`,
/// `Relay_DPDT_Fujitsu_FTR-F1C` and `nist_ftc_08_asme1_ap242-e2` logged 18, 18
/// and 56 sides of a pipe the certified policy gave up on and the tolerant one
/// answered (`whether_a_tolerant_policy_buys_anything_a_certified_one_does_not.rs`).
/// All three files are refused either way, and which arcs those were was not
/// traced.
fn which_of_a_rings_two_circles(
    faces: &[Surface],
    and: &Surface,
    corners: [Point; 2],
) -> Option<Picker> {
    let Surface::Plane(cut) = and else {
        return None;
    };
    let mut rings = faces.iter().filter_map(|face| {
        if let Surface::Torus(ring) = face {
            Some(*ring)
        } else {
            None
        }
    });
    let ring = rings.next()?;
    if rings.any(|other| other != ring) {
        return None;
    }
    let along = ring.axis().direction();
    if cut.normal().cross(along) != Vector::zero() {
        return None;
    }
    let from_the_axis = |corner: Point| {
        let out = corner - ring.centre();
        (out - along * out.dot(along)).length()
    };
    let between = Cylinder::new(ring.axis(), ring.ring_radius()).ok()?;
    let resting = A_CIRCLE_MAY_MISS_ITS_FACE_BY
        * NEAREST_F64_POINT_ROUNDOFF
        * largest_of(
            &[corners[0], corners[1], ring.centre()],
            &[ring.ring_radius(), ring.tube_radius()],
        );
    let [first, second] = corners.map(|corner| from_the_axis(corner) - ring.ring_radius());
    if first < -resting && second < -resting {
        Some(Picker::InsideAPipe(between))
    } else if first > resting && second > resting {
        Some(Picker::OutsideAPipe(between))
    } else {
        None
    }
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
/// - **A plane** of unit normal `n`. A point is `n·X − d` off it, which is
///   `n·c − d` plus `r·(n·e)`, and `|n·e|` is at most `|n × w|` as `e` goes
///   round, so `|n·c − d| + r·|n × w|` bounds every point.
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
///   one circle is at most `|c − m| + |r − ρ_t| + √2·r·sin φ` from the point
///   of the other that a turn by `φ` takes it to. The turn is the one from
///   `w` onto whichever of `t` and `−t` is nearer, since each circle is the
///   same points whichever way its axis points, so `φ` is at most a right
///   angle and `sin φ = |w × t|` either way. A turn by `φ` moves a unit
///   vector at most `2·sin(φ/2) = sin φ / cos(φ/2)`, and `cos(φ/2) ≥ 1/√2`
///   up to a right angle, so `2·sin(φ/2) ≤ √2·sin φ`, equal at a right angle,
///   where a point of the turned circle lying along `t` is `√2·ρ_t` from every
///   point of the tube circle. Both of the circles Q563 pinned turn about a
///   `w` within `1.04e-13` of `−t` (`docs/decisions.md` §387).
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
pub(crate) fn a_surface_holds_the_circle(
    surface: &Surface,
    centre: Point,
    axis: Vector,
    radius: f64,
) -> bool {
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
/// holds its corners to. It was not fitted to the corpus. Over the whole
/// corpus on `b3701c4c`, when only a ball and a pipe had a bound here, the
/// furthest a circle sat off its face in any file that imports was `1.20` of
/// these units, on `Relay_3PST_COTO_3650.step` and `_3660.step`; an allowance
/// of `1` lost those two relays (158 imports), and every allowance from `4`
/// to `1000` gave the same 160. The circles sixteen turned away short of `34`
/// were all in `nist_stc_08_asme1_ap242-e3.stp` and the two Fujitsu `FTR-F1`
/// relays, which are refused for something else first.
/// `the_furthest_a_circle_an_imported_file_needs_is_inside_the_allowance`
/// holds that relay's circle against this number.
///
/// **That is no longer the furthest.** Since Q563 (`docs/decisions.md` §387)
/// `Relay_1-Form-C_Schrack-RYII_RM3.2mm.step` imports with two circles round
/// a torus's tube whose bound reads `13.74` and `13.65` of these units, so an
/// allowance under `13.74` refuses a file the corpus imports, and sixteen
/// clears it by less than a sixth.
/// `the_relays_two_circles_round_their_tubes_are_inside_the_allowance` holds
/// both.
///
/// **That `13.74` is still the furthest a circle whose naming this allowance
/// decides ever needs** (§387.5), but it is not the furthest circle the
/// corpus writes. `the_furthest_a_circle_sits_off_its_face_over_the_whole_corpus`
/// (`mise exec -- just t the_furthest_a_circle_sits_off_its_face_over_the_whole_corpus`)
/// sweeps every `CIRCLE` that bounds an `ADVANCED_FACE` in a file that
/// imports, read straight off the file rather than through this allowance,
/// and finds `29.43` roundings: `nist_ctc_03_asme1_ap242-e2.stp`'s circle
/// `#1681` sits that far off the plane `#1291` its edge `#2757` bounds. That
/// circle's own axis is the plane's normal turned end for end, almost
/// exactly, and its centre misses the plane by `1.1e-12` of a unit `193.8`
/// across — the file's own digits, not this reader's arithmetic. The file
/// still imports: nothing here asks whether that circle lies on that plane,
/// because the edge is named by the plane and vertices that already close
/// it, and this allowance only ever chooses between two ways to *name* a
/// run that reaches it in the first place (see `every_face_holds_the_curve`'s
/// doc comment) — never a reason to refuse one. So `29.43` is the furthest a
/// circle sits off its face anywhere in the corpus, and `13.74` is still the
/// furthest one whose naming sixteen has to cover.
pub(crate) const A_CIRCLE_MAY_MISS_ITS_FACE_BY: f64 = 16.0;

/// The furthest any point of the circle can be from `surface`, and the
/// largest number the bound was worked out from — see
/// [`a_surface_holds_the_circle`] for the bounds. `None` for a surface with no
/// bound here.
pub(crate) fn how_far_off_the_circle_can_be(
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
                    + SQRT_2 * radius * axis.cross(round_the_ring).length(),
                largest,
            ))
        }
        Surface::Plane(flat) => Some((
            flat.signed_distance_to(centre).abs() + radius * flat.normal().cross(axis).length(),
            largest_of(&[centre], &[radius, flat.offset()]),
        )),
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

/// The half-space that picks the arc between `corners` lying the way `facing`
/// points, in a shape a fuse reads, with `centre` the centre of the arc's curve.
///
/// Two ways of telling the two arcs between one pair of ends apart read only the
/// picker and the ends (`a_picker_that_tells_the_two_arcs_apart` in
/// `rcad-topo`): a plane that holds **both ends exactly**, or one with both ends
/// on or in front and the **curve's centre on or behind**.
/// [`a_half_space_holding_both`] stands its plane a few roundings below both
/// ends, so it is the second kind exactly where the centre stays on or behind it.
///
/// **An arc longer than half its curve never has that.** Its centre is on the
/// arc's own side of the chord, so in front of the plane through the chord and
/// further in front of one lowered below it. Such an arc takes a plane through
/// both ends instead, where the numbers hold both ends on one plane exactly
/// ([`a_half_space_through_both`]); the side of it the arc is on is then the
/// side `facing` points to, which is what the lowered plane said too, and
/// nothing about how near the plane came to an end is read. Where no such plane
/// can be found it keeps the lowered one
/// (`a_long_arc_with_no_plane_through_both_its_ends_keeps_the_lowered_one`).
/// Neither of the two ways reads that, but a fuse reads it by a third, which asks
/// whether the middle of the other arc is behind the picker
/// (`into_the_arc_on_the_centres_side` in `rcad-topo`, since Q971): the long
/// side of a bar sliced through two ends no plane holds comes back and cuts the
/// same (`a_long_arc_whose_ends_no_plane_holds_comes_back_and_cuts_the_same`,
/// Q939).
///
/// **An arc whose centre is on or behind the lowered plane keeps it** even
/// where a plane through both ends exists, so every arc that read back before
/// reads back with the very picker it had.
fn a_half_space_naming_the_arc(
    corners: [Point; 2],
    facing: Vector,
    centre: Point,
) -> Option<HalfSpace> {
    let lowered = a_half_space_holding_both(corners, facing)?;
    match lowered.side_of_point(PredicatePoint::from(centre.coordinates())) {
        Decision::Neg | Decision::Zero => Some(lowered),
        Decision::Pos | Decision::Undecidable { .. } | _ => {
            Some(a_half_space_through_both(corners, facing).unwrap_or(lowered))
        }
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

/// A half-space facing `facing`, or `facing` scaled to length one, whose plane
/// both `corners` are exactly on — `None` when neither is.
///
/// The test is the exact predicate, so the plane holds both ends or is not
/// handed back; nothing is near enough. Whether one exists is a question about
/// the numbers: `facing · corner` has to round to the same offset at both
/// corners with nothing lost. The length-one normal is what lets a slice
/// square to an axis at any offset qualify: there `facing` is `(0, −2x, 0)` or
/// the like, `2x·y` need not be a number `f64` holds, and `(0, −1, 0)` makes
/// the offset `y` itself (`a_long_arc_is_named_by_a_plane_through_both_its_ends`).
fn a_half_space_through_both(corners: [Point; 2], facing: Vector) -> Option<HalfSpace> {
    [Some(facing), facing.normalized().ok()]
        .into_iter()
        .flatten()
        .find_map(|normal| {
            let through = HalfSpace::through_a_point_facing(corners[0], normal).ok()?;
            corners
                .iter()
                .all(|corner| {
                    through.side_of_point(PredicatePoint::from(corner.coordinates()))
                        == Decision::Zero
                })
                .then_some(through)
        })
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
    use std::f64::consts::{PI, SQRT_2};
    use std::path::{Path, PathBuf};
    use std::{env, fs};

    use rcad_geom::{
        Axis, Cone, Cylinder, NEAREST_F64_POINT_ROUNDOFF, Plane, Point, Sphere, Surface, Torus,
        Vector,
    };
    use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};

    use super::{
        A_CIRCLE_MAY_MISS_ITS_FACE_BY, ADVANCED_FACE, CIRCLE, EDGE_CURVE, EDGE_LOOP, Ellipse,
        FACE_BOUND, FACE_OUTER_BOUND, FACE_SURFACE, ORIENTED_EDGE, Reading,
        a_half_space_holding_both, a_half_space_naming_the_arc, a_surface_holds_the_circle,
        a_surface_holds_the_ellipse, how_far_off, how_far_off_the_circle_can_be,
        how_far_off_the_ellipse_can_be, plane_through,
    };
    use crate::{EntityId, Import, StepFile};

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

    /// A flat face holds a circle drawn on its plane, and on that plane written
    /// again a hair off, as a file writes a disc's own `PLANE` (§219.6); it does
    /// not hold one on Q386's disc tilted a tenth through the circle's corner.
    /// A plane had no bound here, so the reader wrote a cylinder round any
    /// circle between flat faces without asking (Q574).
    #[test]
    fn a_plane_holds_a_circle_on_it_and_not_one_on_a_disc_tilted_off_it() {
        let lid =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 0.0, 1.0))
                .expect("a plane");
        let a_hair_off = Plane::from_point_and_normal_as_written(
            Point::new(0.0, 0.0, 4.0),
            Vector::new(1e-17, 0.0, 1.0),
        )
        .expect("a plane");
        let tilted =
            Plane::from_point_and_normal(Point::new(1.0, 0.0, 4.0), Vector::new(-0.1, 0.0, 1.0))
                .expect("a plane");
        let centre = Point::new(0.0, 0.0, 4.0);

        for face in [lid, a_hair_off] {
            assert!(
                a_surface_holds_the_circle(&Surface::Plane(face), centre, lid.normal(), 1.0),
                "{face:?} does not hold the circle drawn on it"
            );
        }
        assert!(!a_surface_holds_the_circle(
            &Surface::Plane(lid),
            centre,
            tilted.normal(),
            1.0
        ));
        assert!(!a_surface_holds_the_circle(
            &Surface::Plane(tilted),
            centre,
            lid.normal(),
            1.0
        ));
    }

    /// Before Q560 (`docs/decisions.md` §357.6), [`Cone::point_at`] stepped
    /// across the axis by `self.slope * along`, which goes negative on the
    /// nappe behind the apex, so a point there came back at half a turn
    /// round from where it really is. This cone has its apex at the origin,
    /// slope `0.5`, and points about `-z`, so `(1, 0, 2)` sits at `along =
    /// -2`, where `radius_at(-2) = 1`: that stepping-across bug would have
    /// turned it into `(-1, 0, 2)` and `how_far_off` would have read about
    /// `2.0`, the diameter, instead of near zero. Q560 already fixed
    /// `point_at`; this pins that it stays fixed.
    ///
    /// The bound is four roundings of the point's largest coordinate, `2.0`
    /// (`NEAREST_F64_POINT_ROUNDOFF`'s own unit): generous enough that a true
    /// hit on the cone never trips it, and still two orders of magnitude under
    /// the `2.0` a regression to the old `point_at` would read.
    #[test]
    fn how_far_off_reads_near_zero_on_the_cones_far_nappe() {
        let cone = Cone::new(
            Axis::new(Point::origin(), Vector::new(0.0, 0.0, -1.0)).expect("an axis"),
            0.5,
        )
        .expect("a slope");
        let point = Point::new(1.0, 0.0, 2.0);

        let off = how_far_off(&Surface::Cone(cone), point);
        assert!(
            off < 4.0 * NEAREST_F64_POINT_ROUNDOFF * 2.0,
            "(1, 0, 2) is on the cone's far nappe, where point_at(0, -2) should \
             land on it exactly; how_far_off read {off}, not near zero"
        );
    }

    /// The worst case [`A_CIRCLE_MAY_MISS_ITS_FACE_BY`] published on
    /// `b3701c4c`, held against it (`docs/decisions.md` §15.6): circle `#1359`
    /// of `kicad/Relay_3PST_COTO_3650.step` on its pipe `#1365`, then the
    /// furthest off its face of any circle in a file that imports. Two circles
    /// round a torus's tube now read further and are held by
    /// `the_relays_two_circles_round_their_tubes_are_inside_the_allowance`
    /// (§387).
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

    /// The measurement `A_CIRCLE_MAY_MISS_ITS_FACE_BY`'s doc comment asks for
    /// (§387.5): over every file that imports, every `CIRCLE` that bounds an
    /// `ADVANCED_FACE`, held against that face's own surface and turned into
    /// roundings of the unit the allowance is counted in. The furthest one
    /// found, and the file and entities it came from, are printed so the
    /// doc comment can be updated from this test's own output.
    ///
    /// **This reads the file's entities directly, not through `Walk`**,
    /// because `Walk` only ever checks the faces one edge's own naming
    /// needs — never every face a circle happens to bound. So this sweep is
    /// wider than the reader itself walks, which is the point: it is asking
    /// what the allowance would need to cover, not what the reader happened
    /// to check on its way to a solid.
    ///
    /// Point `RCAD_STEP_CORPUS` at a directory holding `nist/` and `kicad/`;
    /// this **fails** rather than skips when it is missing
    /// (`docs/decisions.md` §44.10).
    #[test]
    fn the_furthest_a_circle_sits_off_its_face_over_the_whole_corpus() {
        let root = corpus();
        let mut furthest = 0.0_f64;
        let mut worst = String::new();
        let (mut files, mut imported) = (0usize, 0usize);

        for source in ["nist", "kicad"] {
            for path in files_under(&root.join(source)) {
                files += 1;
                let Ok(bytes) = fs::read(&path) else { continue };
                let Ok(file) = StepFile::read(&bytes) else {
                    continue;
                };
                if Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new()).is_err() {
                    continue;
                }
                imported += 1;
                let reading = Reading::of(&file);
                for face in file.instances_named(ADVANCED_FACE.name()) {
                    for (roundings, what) in circle_roundings_on_a_face(&reading, face.id()) {
                        if roundings > furthest {
                            furthest = roundings;
                            worst = format!("{} {what}", name_of(&path));
                        }
                    }
                }
            }
        }

        assert!(
            files > 0,
            "the walk found no STEP files, so this measurement did nothing"
        );
        assert!(
            imported > 0,
            "no corpus file imported at all, so this measurement says nothing"
        );
        println!(
            "furthest a circle sits off its face over {imported} of {files} files that import: \
             {furthest} roundings, at {worst}"
        );
        // `A_CIRCLE_MAY_MISS_ITS_FACE_BY` only ever picks between two ways to
        // *name* a run; nothing here refuses a file over it (see
        // `every_face_holds_the_curve`'s doc comment), so the furthest circle
        // over the whole corpus is free to sit past the allowance without
        // that allowance being wrong. This pins the figure found rather than
        // holding it under the allowance, so a corpus that moves is noticed.
        assert!(
            (furthest - 29.43).abs() < 0.5,
            "the furthest a circle sits off its face over the corpus moved to {furthest} \
             ({worst}), where 29.43 was measured; update A_CIRCLE_MAY_MISS_ITS_FACE_BY's doc \
             comment from this test's own output"
        );
    }

    /// Every `CIRCLE` that bounds `face`, each as its roundings off the
    /// face's own surface and the `#id`s of the circle and its edge. A
    /// circle, an edge or a face this cannot read through is dropped rather
    /// than refused: this is a measurement over whatever the file happens to
    /// hold, not a second copy of the reader's own refusals.
    fn circle_roundings_on_a_face(reading: &Reading<'_>, face: EntityId) -> Vec<(f64, String)> {
        let mut found = Vec::new();
        let Ok(keyword) = reading.keyword_of(face, face, &[ADVANCED_FACE, FACE_SURFACE]) else {
            return found;
        };
        let Ok(sits_on) = reading.reference_at(face, keyword, 2) else {
            return found;
        };
        let Ok(surface) = reading.surface_at(face, sits_on, || None) else {
            return found;
        };
        let Ok(bounds) = reading.references_at(face, keyword, 1) else {
            return found;
        };
        for bound in bounds {
            let Ok(bound_keyword) =
                reading.keyword_of(face, bound, &[FACE_OUTER_BOUND, FACE_BOUND])
            else {
                continue;
            };
            let Ok(loop_id) = reading.reference_at(bound, bound_keyword, 1) else {
                continue;
            };
            let Ok(oriented) = reading.references_at(loop_id, EDGE_LOOP, 1) else {
                continue;
            };
            for oriented_edge in oriented {
                let Ok(edge_curve) = reading.reference_at(oriented_edge, ORIENTED_EDGE, 3) else {
                    continue;
                };
                let Ok(curve) = reading.reference_at(edge_curve, EDGE_CURVE, 3) else {
                    continue;
                };
                let Ok(position) = reading.reference_at(curve, CIRCLE, 1) else {
                    continue;
                };
                let Ok(place) = reading.placement_at(curve, position) else {
                    continue;
                };
                let Ok(radius) = reading.real_at(curve, CIRCLE, 2) else {
                    continue;
                };
                let Ok(its_plane) = plane_through(place.origin, place.axis) else {
                    continue;
                };
                let Some((off, largest)) = how_far_off_the_circle_can_be(
                    &surface,
                    place.origin,
                    its_plane.normal(),
                    radius,
                ) else {
                    continue;
                };
                found.push((
                    off / (NEAREST_F64_POINT_ROUNDOFF * largest),
                    format!("circle {curve} on edge {edge_curve} of face {face}"),
                ));
            }
        }
        found
    }

    fn corpus() -> PathBuf {
        let root = env::var_os("RCAD_STEP_CORPUS").unwrap_or_else(|| {
            panic!(
                "RCAD_STEP_CORPUS is not set. Point it at a directory holding nist/ and kicad/ \
                 — this test does not skip on a missing corpus."
            )
        });
        let root = PathBuf::from(root);
        assert!(
            root.is_dir(),
            "RCAD_STEP_CORPUS points at {}, which is not a directory",
            root.display()
        );
        root
    }

    fn files_under(directory: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let Ok(entries) = fs::read_dir(directory) else {
            return found;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                found.extend(files_under(&path));
                continue;
            }
            let is_step = path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    extension.eq_ignore_ascii_case("stp") || extension.eq_ignore_ascii_case("step")
                });
            if is_step {
                found.push(path);
            }
        }
        found.sort();
        found
    }

    fn name_of(path: &Path) -> String {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("a file with no name")
            .to_owned()
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

    /// The worst-case search behind the `√2·r·sin φ` a circle round a torus's
    /// tube is charged for leaning off its tube circle (§15.6, Q563): the tube
    /// circle at the outside of the ring, turned by every 5° from nothing to a
    /// half turn about three axes, 720 points round each, each held against
    /// the bound by its distance from the tube circle the bound measures from
    /// and from the torus itself.
    ///
    /// **Past a right angle is half the draw.** The bound reads `|w × t|`,
    /// which is the same for a turn `φ` and for `180° − φ`; the circle is the
    /// same points whichever way its axis points, so those draws are where a
    /// bound that took `φ` for the turn and not `180° − φ` would fail.
    ///
    /// **The control is that the factor is reached**: at a right angle about
    /// the torus's own axis, the point of the circle that lies along `t` is
    /// `√2·r` from every point of the tube circle, so a factor below `√2`
    /// would fail this test. Each circle's points are the tube circle's,
    /// starting at the outside of the ring, turned with it, so that point is
    /// the first of them at that angle.
    #[test]
    fn a_circle_round_the_tube_turned_by_any_angle_is_within_its_bound() {
        let ring = Torus::new(
            Axis::new(Point::new(0.5, 1.0, -1.0), Vector::new(0.1, 1.0, 0.2)).expect("an axis"),
            5.0,
            1.5,
        )
        .expect("two radii");
        let surface = Surface::Torus(ring);
        let up = ring.axis().direction();
        let (out, _) = ring.basis();
        let round_the_ring = up.cross(out);
        let centre = ring.centre() + out * ring.ring_radius();
        let tube = ring.tube_radius();
        let mut drawn = 0;
        let mut the_most_of_the_lean_reached: f64 = 0.0;
        for turn_about in [
            up,
            (up + out).normalized().expect("a direction"),
            (up - out * 0.6 + round_the_ring * 0.5)
                .normalized()
                .expect("a direction"),
        ] {
            for step in 0..=36 {
                let angle = PI * f64::from(step) / 36.0;
                let axis = turned(round_the_ring, turn_about, angle);
                let (bound, largest) = how_far_off_the_circle_can_be(&surface, centre, axis, tube)
                    .expect("a torus has a bound");
                let slack = 4.0 * NEAREST_F64_POINT_ROUNDOFF * largest;
                let lean = SQRT_2 * tube * axis.cross(round_the_ring).length();
                let [across, and] = [out, up].map(|way| turned(way, turn_about, angle));
                for at in 0..720 {
                    let point = round_the_circle(
                        centre,
                        across,
                        and,
                        tube,
                        std::f64::consts::TAU * f64::from(at) / 720.0,
                    );
                    let off_the_tube_circle = {
                        let apart = point - centre;
                        let along = apart.dot(round_the_ring);
                        let from_the_middle = (apart - round_the_ring * along).length();
                        along.hypot(from_the_middle - tube)
                    };
                    for (off, from) in [
                        (off_the_tube_circle, "the tube circle"),
                        (distance_from(&surface, point), "the torus"),
                    ] {
                        assert!(
                            off <= bound + slack,
                            "a circle turned {angle} about {turn_about:?} has a point {off} off \
                             {from}, past its bound {bound}"
                        );
                    }
                    if lean > 0.0 {
                        the_most_of_the_lean_reached =
                            the_most_of_the_lean_reached.max(off_the_tube_circle / lean);
                    }
                }
                drawn += 1;
            }
        }
        assert_eq!(
            drawn, 111,
            "the draw must not shrink without somebody saying so"
        );
        assert!(
            the_most_of_the_lean_reached >= 1.0 - 1e-9,
            "no point came within a rounding of √2·r·sin φ off its tube circle, the most being \
             {the_most_of_the_lean_reached} of it, so this draw cannot tell √2 from a smaller \
             factor"
        );
    }

    /// Circles `#1375` and `#1568` of
    /// `kicad/Relay_1-Form-C_Schrack-RYII_RM3.2mm.step`, each round the tube
    /// of torus `#1558` or `#1588`, from the file's decimals (Q563). Each
    /// turns about a direction `1.04e-13` off the opposite of the way round
    /// the ring at its centre, a turn past a right angle that the bound reads
    /// as the small one. At `2·r·sin φ` they read `19.43` and `19.31`
    /// roundings, past the sixteen allowed, and the file was refused; at
    /// `√2·r·sin φ` they read the numbers below.
    #[test]
    fn the_relays_two_circles_round_their_tubes_are_inside_the_allowance() {
        for (centre, leaning, ring_centre, measured) in [
            (
                Point::new(-0.9, 1.56, 12.05),
                Vector::new(-1.0, -1.039_781_074_449e-13, 0.0),
                Point::new(-0.9, 2.31, 12.05),
                13.74,
            ),
            (
                Point::new(-0.15, 5.31, 12.05),
                Vector::new(1.033_119_736_301e-13, -1.0, 0.0),
                Point::new(-0.9, 5.31, 12.05),
                13.65,
            ),
        ] {
            let ring = Surface::Torus(
                Torus::new(
                    Axis::new(ring_centre, Vector::new(0.0, 0.0, 1.0)).expect("an axis"),
                    0.75,
                    0.25,
                )
                .expect("two radii"),
            );
            let axis = Plane::from_point_and_normal(centre, leaning)
                .expect("a plane")
                .normal();
            let (off, largest) =
                how_far_off_the_circle_can_be(&ring, centre, axis, 0.25).expect("a bound");
            let roundings = off / (NEAREST_F64_POINT_ROUNDOFF * largest);
            assert!(
                (roundings - measured).abs() < 0.01,
                "the circle at {centre:?} is {roundings} roundings off its torus, where \
                 {measured} was measured"
            );
            assert!(a_surface_holds_the_circle(&ring, centre, axis, 0.25));
        }
    }

    /// `v` turned by `angle` about the unit direction `about`.
    fn turned(v: Vector, about: Vector, angle: f64) -> Vector {
        v * angle.cos()
            + about.cross(v) * angle.sin()
            + about * (about.dot(v) * (1.0 - angle.cos()))
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
        if let Surface::Cone(cone) = surface {
            let (along, across) = along_and_across(cone.axis(), point - cone.apex());
            return (across - along.abs() * cone.slope()).abs() / cone.slope().hypot(1.0);
        }
        if let Surface::Torus(ring) = surface {
            let (along, across) = along_and_across(ring.axis(), point - ring.centre());
            return ((across - ring.ring_radius()).hypot(along) - ring.tube_radius()).abs();
        }
        if let Surface::Cylinder(pipe) = surface {
            let (_, across) = along_and_across(pipe.axis(), point - pipe.axis().origin());
            return (across - pipe.radius()).abs();
        }
        unreachable!("no curve is drawn on {surface:?} here")
    }

    /// How far `apart` runs along `line` and how far it is from it.
    fn along_and_across(line: Axis, apart: Vector) -> (f64, f64) {
        let along = apart.dot(line.direction());
        (along, (apart - line.direction() * along).length())
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

    /// A circle of radius `5s` about `z` sliced by `y = −3s`, with the
    /// corners at `(±4s, −3s, 0)`: the long arc is named by a plane both ends
    /// are exactly on, and the short arc keeps the lowered plane it always had.
    ///
    /// `s = 1` is the plane `facing` itself, offset `−24`. `s = 1 + 2⁻³⁰` is the
    /// one where `facing · corner = −24s²` needs 62 bits, so only the
    /// length-one normal `(0, 1, 0)` holds both ends — the control checks that
    /// `facing` itself does not.
    #[test]
    fn a_long_arc_is_named_by_a_plane_through_both_its_ends() {
        let side = |half: &rcad_geom::HalfSpace, at: Point| {
            half.side_of_point(PredicatePoint::from(at.coordinates()))
        };
        for (s, facing_itself_holds_both) in [(1.0, true), (1.0 + 2f64.powi(-30), false)] {
            let corners = [
                Point::new(4.0 * s, -3.0 * s, 0.0),
                Point::new(-4.0 * s, -3.0 * s, 0.0),
            ];
            let into_the_long_arc = (corners[1] - corners[0]).cross(Vector::new(0.0, 0.0, 1.0));
            let centre = Point::origin();

            let long = a_half_space_naming_the_arc(corners, into_the_long_arc, centre)
                .expect("a plane through both ends of the long arc");
            assert_eq!(
                corners.map(|corner| side(&long, corner)),
                [Decision::Zero; 2],
                "s = {s}: the long arc's plane holds both its ends"
            );
            assert_eq!(side(&long, centre), Decision::Pos, "s = {s}: centre");
            let lowered =
                a_half_space_holding_both(corners, into_the_long_arc).expect("a lowered plane");
            assert_eq!(
                side(&lowered, centre),
                Decision::Pos,
                "s = {s}: the lowered plane has the long arc's centre in front, which is \
                 what a fuse does not read"
            );
            let through_facing_itself =
                rcad_geom::HalfSpace::through_a_point_facing(corners[0], into_the_long_arc)
                    .expect("a normal with length");
            assert_eq!(
                corners
                    .iter()
                    .all(|corner| side(&through_facing_itself, *corner) == Decision::Zero),
                facing_itself_holds_both,
                "s = {s}: whether `facing` itself holds both ends"
            );

            let into_the_short_arc = -into_the_long_arc;
            let short = a_half_space_naming_the_arc(corners, into_the_short_arc, centre)
                .expect("a plane for the short arc");
            assert_eq!(
                Some(short),
                a_half_space_holding_both(corners, into_the_short_arc),
                "s = {s}: the short arc keeps the lowered plane"
            );
            assert_eq!(
                corners.map(|corner| side(&short, corner)),
                [Decision::Pos; 2],
                "s = {s}: the short arc's ends are in front of its lowered plane"
            );
            assert_eq!(side(&short, centre), Decision::Neg, "s = {s}: centre");
        }
    }

    /// A long arc whose ends no plane facing `facing`, or `facing` scaled to
    /// length one, holds exactly keeps the lowered plane, with its centre in
    /// front, which neither way of `a_picker_that_tells_the_two_arcs_apart` in
    /// `rcad-topo` reads. A fuse reads it by `into_the_arc_on_the_centres_side`
    /// all the same, and `a_long_arc_whose_ends_no_plane_holds_comes_back_and_cuts_the_same`
    /// in the round-trip tests is a whole solid that reaches it (Q939).
    #[test]
    fn a_long_arc_with_no_plane_through_both_its_ends_keeps_the_lowered_one() {
        let corners = [Point::new(0.1, 0.7, 0.0), Point::new(-0.3, 0.2, 0.0)];
        let facing = (corners[1] - corners[0]).cross(Vector::new(0.0, 0.0, 1.0));
        let centre = Point::new(-1.0, 1.2, 0.0);
        let named = a_half_space_naming_the_arc(corners, facing, centre).expect("a picker");
        assert_eq!(Some(named), a_half_space_holding_both(corners, facing));
        assert_eq!(
            named.side_of_point(PredicatePoint::from(centre.coordinates())),
            Decision::Pos
        );
    }
}
