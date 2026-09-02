//! A `MANIFOLD_SOLID_BREP` becomes a solid, or is refused by the `#id` that
//! stopped it.
//!
//! The walk is the one an AP203 file is laid out in — closed shell, advanced
//! face, face bound, edge loop, oriented edge, edge curve, vertex point — and it
//! goes straight into a [`SolidBuilder`], which is the public door §42.5 said
//! the kernel did not have. Nothing is written into the caller's model until
//! the whole description has been through `Model::validate`.

use std::collections::BTreeMap;

use rcad_geom::{EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Surface, Vector};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface};
use rcad_topo::{
    DraftCoedge, DraftEdge, DraftFace, DraftStep, DraftVertex, Model, SolidBuilder, SolidKey,
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
    vertex_names: Vec<EntityId>,
    edge_names: Vec<EntityId>,
    coedge_names: Vec<EntityId>,
    wire_names: Vec<EntityId>,
    face_names: Vec<EntityId>,
    face_surfaces: Vec<Surface>,
}

impl Walk {
    fn new(brep: EntityId) -> Self {
        Self {
            brep,
            builder: SolidBuilder::new(),
            vertices: BTreeMap::new(),
            edges: BTreeMap::new(),
            vertex_names: Vec::new(),
            edge_names: Vec::new(),
            coedge_names: Vec::new(),
            wire_names: Vec::new(),
            face_names: Vec::new(),
            face_surfaces: Vec::new(),
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
        let mut skin: Vec<(DraftFace, Orientation)> = Vec::with_capacity(faces.len());
        for face in faces {
            skin.push(self.describe_the_face(reading, face)?);
        }
        self.builder
            .add_shell(&skin)
            .map_err(|why| self.refused(shell, why))
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
        let sits_on = reading.reference_at(face, keyword, 2)?;
        let written = reading.surface_at(face, sits_on)?;
        let same_sense = reading.flag_at(face, keyword, 3)?;
        let (surface, used) = match (written.as_plane(), same_sense) {
            (_, true) => (written, Orientation::Forward),
            (Some(flat), false) => (Surface::Plane(flat.reversed()), Orientation::Forward),
            (None, false) => (written, Orientation::Reversed),
        };

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
    ///   have such a face** and every one of them was refused before this.
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
    /// corners give a polygon, and a **closed** edge — a whole circle or
    /// ellipse, which has one corner and so no polygon at all — gives its own
    /// area, signed by which way round it is walked. Without that second term a
    /// washer's two circular bounds both sweep exactly nothing and neither can
    /// be told from the other.
    ///
    /// An *arc* contributes only its chord, which is a hint sometimes short by a
    /// segment. See [`Walk::outer_bound_first`] for why a hint is all this has
    /// to be.
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
        let mut round = 0.0;
        for use_of_an_edge in oriented {
            let edge_curve = reading.reference_at(use_of_an_edge, "ORIENTED_EDGE", 3)?;
            let forwards =
                reading.flag_at(use_of_an_edge, "ORIENTED_EDGE", 4)? != walk_it_backwards;
            let ends = [
                reading.reference_at(edge_curve, "EDGE_CURVE", 1)?,
                reading.reference_at(edge_curve, "EDGE_CURVE", 2)?,
            ];
            corners.push(self.point_of_vertex(reading, ends[usize::from(!forwards)])?);
            if ends[0] == ends[1] {
                let along_the_curve = reading.flag_at(edge_curve, "EDGE_CURVE", 4)? == forwards;
                round +=
                    self.area_a_closed_curve_sweeps(reading, edge_curve, along_the_curve, flat)?;
            }
        }
        let mut twice_the_area = 2.0 * round;
        for step in 0..corners.len() {
            let (from, to) = (
                flat.plane_coordinates_of(corners[step]),
                flat.plane_coordinates_of(corners[(step + 1) % corners.len()]),
            );
            twice_the_area += from.0 * to.1 - to.0 * from.1;
        }
        Ok(twice_the_area)
    }

    /// The area a whole circle or ellipse encloses, signed by which way round
    /// the plane's normal it is walked.
    ///
    /// Zero for anything else, which leaves the bound's own polygon as the whole
    /// of the hint.
    fn area_a_closed_curve_sweeps(
        &self,
        reading: &Reading<'_>,
        edge_curve: EntityId,
        along_the_curve: bool,
        flat: &Plane,
    ) -> Result<f64, Refusal> {
        let curve = reading.reference_at(edge_curve, "EDGE_CURVE", 3)?;
        let Some(held) = reading.file().instance(curve) else {
            return Ok(0.0);
        };
        let (keyword, area) = match held.parts()[0].keyword() {
            "CIRCLE" => {
                let radius = reading.real_at(curve, "CIRCLE", 2)?;
                ("CIRCLE", core::f64::consts::PI * radius * radius)
            }
            "ELLIPSE" => {
                let across = reading.real_at(curve, "ELLIPSE", 2)?;
                let along = reading.real_at(curve, "ELLIPSE", 3)?;
                ("ELLIPSE", core::f64::consts::PI * across * along)
            }
            _ => return Ok(0.0),
        };
        let position = reading.reference_at(curve, keyword, 1)?;
        let place = reading.placement_at(curve, position)?;
        let facing = place.axis.dot(flat.normal());
        if facing == 0.0 {
            return Ok(0.0);
        }
        Ok(area * facing.signum() * if along_the_curve { 1.0 } else { -1.0 })
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
            steps.push(DraftStep::Along(self.describe_the_coedge(
                reading,
                use_of_an_edge,
                walk_it_backwards,
            )?));
        }
        let wire = self
            .builder
            .add_wire(&steps)
            .map_err(|why| self.refused(loop_id, why))?;
        self.wire_names.push(loop_id);
        Ok(wire)
    }

    /// One `ORIENTED_EDGE`, as one use of one edge running one way round.
    fn describe_the_coedge(
        &mut self,
        reading: &Reading<'_>,
        oriented: EntityId,
        walk_it_backwards: bool,
    ) -> Result<DraftCoedge, Refusal> {
        let edge_curve = reading.reference_at(oriented, "ORIENTED_EDGE", 3)?;
        let forwards = reading.flag_at(oriented, "ORIENTED_EDGE", 4)? != walk_it_backwards;
        let edge = self.describe_the_edge(reading, edge_curve)?;
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
        Ok(coedge)
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
        self.the_arc_between(edge_curve, &place, corners, along_the_curve)
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
        corners: [Point; 2],
        along_the_curve: bool,
    ) -> Result<EdgeRun, Refusal> {
        if corners[0] == corners[1] {
            return Err(Refusal::AnArcThatStartsAndEndsInTheSamePlace { edge: edge_curve });
        }
        let flat = Plane::from_point_and_normal(place.origin, place.axis).map_err(|why| {
            Refusal::TheGeometryIsNotAWorkingShape {
                id: edge_curve,
                why,
            }
        })?;
        let across = (corners[1] - corners[0]).cross(place.axis);
        let facing = if along_the_curve { across } else { -across };
        let picking = a_half_space_holding_both(corners, facing)
            .ok_or(Refusal::AnArcThisReaderCouldNotName { edge: edge_curve })?;
        Ok(EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(flat),
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
    fn build(
        self,
        reading: &Reading<'_>,
        model: &mut Model,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<SolidKey, Refusal> {
        let brep = self.brep;
        let builder = self.builder.clone();
        match builder.build(model, policy, log) {
            Ok(solid) => Ok(solid),
            Err(why) => Err(match self.a_corner_provably_off_a_curved_face(reading) {
                Some(off) => off,
                None => match self.which_entity_the_builder_meant(&why) {
                    Some(entity) => Refusal::ThisEntityOfTheSolidWasRefused {
                        solid: brep,
                        entity,
                        why: Box::new(why),
                    },
                    None => Refusal::TheSolidTheFileDescribesIsNotOne {
                        solid: brep,
                        why: Box::new(why),
                    },
                },
            }),
        }
    }

    /// The worst corner this description puts on a curved face it is provably
    /// not on, with how far off it is.
    ///
    /// **Asked only after a build has already been refused**, so nothing is
    /// loosened and no import that succeeded went through here. What it adds is
    /// the number: `Model::validate` says a corner of a curved face is off its
    /// surface and says nothing about *which* corner or by *how much*, and both
    /// are what somebody deciding what the next milestone builds needs.
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
        if let Some(off) = worst {
            return Some(off);
        }
        self.a_corner_provably_off_the_plane_of_its_own_arc(reading)
    }

    /// The worst corner this description puts at the end of an arc whose own
    /// plane it is provably not on.
    ///
    /// **This one is the vocabulary's and not the file's.** An arc is named by
    /// the surfaces either side of it *and a second surface*, because two ends
    /// on a closed curve name two arcs — and `Model::validate` asks the corners
    /// of a named arc to be exactly on that second surface. The second surface
    /// is the plane the circle lies in, and a [`Plane`] scales its normal to
    /// length one and rounds its offset, so a corner is exactly on it only where
    /// the numbers cancel: an axis-aligned circle whose ends share the centre's
    /// coordinate along the axis. Every other arc in the world is refused for a
    /// rounding in a name rather than for anything about the shape.
    ///
    /// **A gap, and what closes it is one crate away.** The corner really is on
    /// the curve; what it is off is the *stored* plane, by the rounding of one
    /// division and one dot product. `check_flat_face` already allows a corner
    /// that much slack against the plane its face sits on
    /// (`Model::allowed_wobble`) and `check_edges` allows none against the plane
    /// an edge is *named* by, which is the weaker claim of the two. Reported
    /// here rather than worked around, because `rcad-topo` is not this lane's
    /// crate (§15.9).
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
    /// position in that list. A refusal that names no entity — the sweep's own,
    /// which is about the solid as a whole — comes back `None`.
    fn which_entity_the_builder_meant(&self, why: &rcad_topo::Error) -> Option<EntityId> {
        let (names, index) = match why {
            rcad_topo::Error::ADraftPointIsNotFinite { vertex } => (&self.vertex_names, *vertex),
            rcad_topo::Error::ADraftEdgeHasNoLength { edge }
            | rcad_topo::Error::ADraftEdgeIsWalkedTheSameWayTwice { edge }
            | rcad_topo::Error::ADraftEdgeIsNotWalkedByTwoCoedges { edge, .. } => {
                (&self.edge_names, *edge)
            }
            rcad_topo::Error::ADraftEntityIsUsedTwice { entity, index }
            | rcad_topo::Error::ADraftEntityIsUsedByNothing { entity, index } => {
                (self.names_of(entity)?, *index)
            }
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
