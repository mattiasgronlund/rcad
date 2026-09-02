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
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
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
    walk.build(model, policy, log)
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
    /// whether its own normal is the surface's or the other way about.
    ///
    /// **`same_sense` is false means two things at once, and both have to be
    /// said.** The face's outward normal is the surface's reversed, which is
    /// [`Orientation::Reversed`] on the shell; and the file's loops wind the
    /// right way round *that* normal, which is the wrong way round the one the
    /// kernel stores. `Model::validate` reads a wire's winding against the
    /// stored surface and never against the shell's use of it, so the loop is
    /// walked backwards to match. Two reversals and not one.
    fn describe_the_face(
        &mut self,
        reading: &Reading<'_>,
        face: EntityId,
    ) -> Result<(DraftFace, Orientation), Refusal> {
        let keyword = reading.keyword_of(self.brep, face, &["ADVANCED_FACE", "FACE_SURFACE"])?;
        let sits_on = reading.reference_at(face, keyword, 2)?;
        let surface = reading.surface_at(face, sits_on)?;
        let same_sense = reading.flag_at(face, keyword, 3)?;
        let bounds = reading.references_at(face, keyword, 1)?;
        let outer_first = outer_bound_first(reading, face, &bounds)?;

        let mut wires = Vec::with_capacity(outer_first.len());
        for bound in outer_first {
            wires.push(self.describe_the_bound(reading, face, bound, same_sense)?);
        }
        let drafted = self
            .builder
            .add_face(surface, &wires)
            .map_err(|why| self.refused(face, why))?;
        self.face_names.push(face);
        Ok((
            drafted,
            if same_sense {
                Orientation::Forward
            } else {
                Orientation::Reversed
            },
        ))
    }

    /// One `FACE_BOUND` or `FACE_OUTER_BOUND`, as a wire.
    fn describe_the_bound(
        &mut self,
        reading: &Reading<'_>,
        face: EntityId,
        bound: EntityId,
        face_agrees_with_its_surface: bool,
    ) -> Result<rcad_topo::DraftWire, Refusal> {
        let keyword = reading.keyword_of(face, bound, &["FACE_OUTER_BOUND", "FACE_BOUND"])?;
        let loop_id = reading.reference_at(bound, keyword, 1)?;
        let bound_agrees = reading.flag_at(bound, keyword, 2)?;
        let walk_it_backwards = bound_agrees != face_agrees_with_its_surface;

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
        model: &mut Model,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<SolidKey, Refusal> {
        let brep = self.brep;
        let builder = self.builder.clone();
        match builder.build(model, policy, log) {
            Ok(solid) => Ok(solid),
            Err(why) => Err(match self.which_entity_the_builder_meant(&why) {
                Some(entity) => Refusal::ThisEntityOfTheSolidWasRefused {
                    solid: brep,
                    entity,
                    why: Box::new(why),
                },
                None => Refusal::TheSolidTheFileDescribesIsNotOne {
                    solid: brep,
                    why: Box::new(why),
                },
            }),
        }
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

/// The bounds of a face with the outer one first, which is the order
/// [`SolidBuilder::add_face`] takes them in.
///
/// A face with one bound has that one as its outer, whatever the file called it
/// — the schema's `FACE_BOUND` is the supertype and a lone bound is the
/// boundary. A face with several and no `FACE_OUTER_BOUND` has not said which
/// is which, and working it out is geometry this reader does not do.
fn outer_bound_first(
    reading: &Reading<'_>,
    face: EntityId,
    bounds: &[EntityId],
) -> Result<Vec<EntityId>, Refusal> {
    if bounds.len() <= 1 {
        return Ok(bounds.to_vec());
    }
    let outer = bounds.iter().position(|bound| {
        reading
            .file()
            .instance(*bound)
            .is_some_and(|held| held.part("FACE_OUTER_BOUND").is_some())
    });
    let Some(outer) = outer else {
        return Err(Refusal::AFaceThatDoesNotSayWhichBoundIsTheOuterOne {
            face,
            bounds: bounds.len(),
        });
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
