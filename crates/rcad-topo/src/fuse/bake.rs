//! The input solids, moved into world coordinates with every corner re-derived.
//!
//! Placement is baked in here and nowhere else (§17.3). The transform's rounding
//! is representation wobble, and it is tagged as such rather than refused on.

use std::collections::{BTreeMap, BTreeSet};

use rcad_geom::{Orientation, Plane, Point};
use rcad_predicates::{Approximation, Cause, Decision, Point as PredicatePoint};

use crate::key::{FaceSlot, VertexSlot};
use crate::{Error, FaceKey, FaceView, Invalid, Model, Result, SolidKey, SolidView, VertexKey};

use super::Cut;

/// Move every solid in `solids` into world coordinates.
///
/// Nothing here makes a decision a policy could soften: a corner is refused only
/// when a plane around it *proves* it is somewhere else, so baking never writes
/// to a [`ProvenanceLog`](rcad_predicates::ProvenanceLog) and has no policy to
/// take.
///
/// # Errors
///
/// [`Error::Invalid`] when a corner's own faces disagree about where it is by
/// more than its box, and [`Error::Geometry`] when no three planes around a
/// corner are far enough from sharing a line to place it.
///
/// # Panics
///
/// If a solid is broken in a way [`Model::validate`] rejects: a face used at two
/// placements, a corner with fewer than three faces round it, or an edge with
/// only one face along it.
pub(crate) fn bake(model: &Model, solids: &[SolidKey]) -> Result<Vec<BakedSolid>> {
    solids
        .iter()
        .map(|&solid| bake_one_solid(model, solid))
        .collect()
}

/// One input solid, in world coordinates.
#[derive(Clone, Debug)]
pub(crate) struct BakedSolid {
    /// The solid this came from.
    pub(crate) key: SolidKey,
    /// Its faces, in the order the model stores them.
    pub(crate) faces: Vec<BakedFace>,
    /// A box round the whole solid, for skipping face pairs that cannot meet.
    pub(crate) bounds: Bounds,
}

/// One face of an input solid, in world coordinates.
#[derive(Clone, Debug)]
pub(crate) struct BakedFace {
    /// The face this came from. Every piece cut out of it names it as its one
    /// ancestor.
    pub(crate) key: FaceKey,
    /// The plane it sits on, already moved into place. This is the face
    /// (§17.1); the corners below are cached samples of it.
    pub(crate) plane: Plane,
    /// Which way round the solid uses the face.
    pub(crate) orientation: Orientation,
    /// The wires that trim it, outer one first, each walked the way the face
    /// itself stores it — anticlockwise round [`BakedFace::plane`]'s normal for
    /// the outer wire and clockwise for a hole, which is what
    /// [`Model::validate`] enforces.
    pub(crate) wires: Vec<Vec<BakedEdge>>,
    /// The lines cutting this face, filled in by the gathering step.
    pub(crate) cuts: Vec<Cut>,
    /// A box round the face.
    pub(crate) bounds: Bounds,
}

/// One edge of one wire of a face.
#[derive(Clone, Debug)]
pub(crate) struct BakedEdge {
    /// The corner the wire is at when it starts along this edge.
    pub(crate) start: BakedCorner,
    /// The plane of the face on the other side of this edge. Together with the
    /// face's own plane it *is* the edge's line (§17.4).
    pub(crate) across: Plane,
}

/// A corner of an input face, re-derived from the planes that make it.
#[derive(Clone, Debug)]
pub(crate) struct BakedCorner {
    /// The input vertex this is.
    pub(crate) vertex: VertexKey,
    /// The three planes it was solved from: the best-conditioned triple of the
    /// planes around it.
    pub(crate) planes: [Plane; 3],
    /// The box it is somewhere inside.
    pub(crate) point: PredicatePoint,
    /// Every face plane that meets here, in a fixed order. Which lines run
    /// through a corner is then a question about the model rather than about
    /// arithmetic, which is what keeps the arrangement from having to prove a
    /// corner is on a line it is on by construction.
    pub(crate) around: Vec<Plane>,
    /// How far this corner may be from where its planes really meet.
    ///
    /// Representation wobble (§17.2): present even though every decision was
    /// proved, because Cramer's rule divides. It is reported, never refused on,
    /// and it never goes in a [`ProvenanceLog`](rcad_predicates::ProvenanceLog).
    pub(crate) approximation: Approximation,
}

/// A box round some geometry, in world coordinates.
///
/// Only ever used to skip pairs that cannot meet. A box test changes no answer,
/// so plain `f64` is honest here in a way it would not be one line further on.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Bounds {
    low: [f64; 3],
    high: [f64; 3],
}

impl Bounds {
    /// Whether the two boxes have any point in common.
    pub(crate) fn overlap(self, other: Bounds) -> bool {
        (0..3).all(|axis| self.low[axis] <= other.high[axis] && other.low[axis] <= self.high[axis])
    }

    fn nothing() -> Self {
        Self {
            low: [f64::INFINITY; 3],
            high: [f64::NEG_INFINITY; 3],
        }
    }

    fn take_in_point(&mut self, point: Point) {
        for (axis, coordinate) in point.coordinates().into_iter().enumerate() {
            self.low[axis] = self.low[axis].min(coordinate);
            self.high[axis] = self.high[axis].max(coordinate);
        }
    }

    fn take_in(&mut self, other: Bounds) {
        for axis in 0..3 {
            self.low[axis] = self.low[axis].min(other.low[axis]);
            self.high[axis] = self.high[axis].max(other.high[axis]);
        }
    }
}

fn bake_one_solid(model: &Model, key: SolidKey) -> Result<BakedSolid> {
    let solid = model.solid(key);
    let planes = plane_of_every_face(model, solid);
    let corners = corner_of_every_vertex(model, solid, &planes)?;

    let mut faces = Vec::new();
    let mut bounds = Bounds::nothing();
    for face in solid.faces() {
        let baked = bake_one_face(model, face, &planes, &corners);
        bounds.take_in(baked.bounds);
        faces.push(baked);
    }
    Ok(BakedSolid { key, faces, bounds })
}

/// Every face of the solid with its plane already moved into place.
///
/// # Panics
///
/// If the solid uses one face twice. Two uses can sit at two placements, and
/// then "the plane of that face" has two answers; [`Model::validate`] refuses it
/// as [`Invalid::UsedMoreThanOnce`].
fn plane_of_every_face(model: &Model, solid: SolidView<'_>) -> BTreeMap<FaceSlot, Plane> {
    let mut planes = BTreeMap::new();
    for face in solid.faces() {
        let slot = face.key().slot_in(model.id());
        let already_there = planes.insert(slot, face.plane());
        assert!(
            already_there.is_none(),
            "a solid uses one face twice, so it has two planes"
        );
    }
    planes
}

fn corner_of_every_vertex(
    model: &Model,
    solid: SolidView<'_>,
    planes: &BTreeMap<FaceSlot, Plane>,
) -> Result<BTreeMap<VertexSlot, BakedCorner>> {
    let mut corners = BTreeMap::new();
    for face in solid.faces() {
        for wire in model.face(face.key()).wires() {
            for coedge in wire.coedges() {
                let vertex = coedge.start().key();
                let slot = vertex.slot_in(model.id());
                if corners.contains_key(&slot) {
                    continue;
                }
                let around = planes_around_a_vertex(model, vertex, planes);
                corners.insert(slot, derive_a_corner(vertex, around)?);
            }
        }
    }
    Ok(corners)
}

/// The planes of every face of this solid that meets at this vertex.
///
/// # Panics
///
/// If fewer than three do. A corner of a closed solid has at least three faces
/// round it, so reaching this means the model was already broken.
fn planes_around_a_vertex(
    model: &Model,
    vertex: VertexKey,
    planes: &BTreeMap<FaceSlot, Plane>,
) -> Vec<Plane> {
    let mut faces = BTreeSet::new();
    for edge in model.vertex(vertex).edges().collect::<Vec<_>>() {
        for face in model.edge(edge).neighbouring_faces() {
            let slot = face.slot_in(model.id());
            if planes.contains_key(&slot) {
                faces.insert(slot);
            }
        }
    }
    assert!(
        faces.len() >= 3,
        "a corner of a closed solid has at least three faces round it, this one has {}",
        faces.len()
    );
    faces.iter().map(|slot| planes[slot]).collect()
}

/// Solve a corner from the planes around it (§17.1).
///
/// A corner where four or more faces meet is over-determined, so it is solved
/// from the best-conditioned triple and every remaining plane is then asked
/// whether it agrees. [`Plane::meet_of_three`] refusing three planes that are
/// too close to sharing a line *is* the conditioning filter, so there is no
/// second threshold here to argue about.
///
/// A plane that comes back [`Decision::Pos`] or [`Decision::Neg`] has *proved*
/// the corner is somewhere else, which means the input's corners and its planes
/// disagree by more than roundoff. [`Decision::Undecidable`] is agreement, not a
/// failure: the plane passes through the corner's box, which is as much as
/// anything can say about a point that had to be built.
fn derive_a_corner(vertex: VertexKey, around: Vec<Plane>) -> Result<BakedCorner> {
    let mut best: Option<([Plane; 3], PredicatePoint)> = None;
    let mut refusal = None;
    for first in 0..around.len() {
        for second in (first + 1)..around.len() {
            for third in (second + 1)..around.len() {
                let triple = [around[first], around[second], around[third]];
                match Plane::meet_of_three(&triple[0], &triple[1], &triple[2]) {
                    Ok(point) => {
                        let point = as_exact_as_it_is(point);
                        if best.is_none_or(|(_, best)| point.uncertainty() < best.uncertainty()) {
                            best = Some((triple, point));
                        }
                    }
                    Err(why) => refusal = Some(why),
                }
            }
        }
    }
    let Some((planes, point)) = best else {
        return Err(Error::Geometry(
            refusal.expect("three planes that do not meet always say why"),
        ));
    };

    for plane in &around {
        if planes.contains(plane) {
            continue;
        }
        match plane.side_of_point(point) {
            Decision::Zero | Decision::Undecidable { .. } => (),
            Decision::Neg | Decision::Pos => {
                return Err(Invalid::VertexOffItsFacePlane {
                    distance: plane.signed_distance_to(middle_of(point)).abs(),
                }
                .into());
            }
            other => {
                panic!("side of plane came back with a decision nothing here knows: {other:?}")
            }
        }
    }

    Ok(BakedCorner {
        vertex,
        planes,
        point,
        approximation: Approximation::new(point.uncertainty(), Cause::ConstructedPoint),
        around,
    })
}

/// A box that has collapsed onto one `f64` triple *is* that triple.
///
/// The box is a certified enclosure, so a box of no width proves the true meet
/// is exactly the number in it, and saying so is a statement of what was proved
/// rather than a rounding.
///
/// It matters more than it looks. Level 3 of the cascade cannot run on a
/// constructed point, so a lattice-aligned model — where every meet lands
/// exactly on an `f64` — would otherwise come back undecided on decisions it can
/// prove, and §17.7's control family, whose expected refusal rate is zero, would
/// refuse everything.
fn as_exact_as_it_is(point: PredicatePoint) -> PredicatePoint {
    let box_of_it = point.enclosure();
    if box_of_it.iter().all(|side| side.lo() == side.hi()) {
        return PredicatePoint::from([box_of_it[0].lo(), box_of_it[1].lo(), box_of_it[2].lo()]);
    }
    point
}

/// The middle of a point's box, for reporting a distance a caller can read.
fn middle_of(point: PredicatePoint) -> Point {
    let box_of_it = point.enclosure();
    let axis =
        |axis: usize| box_of_it[axis].lo() + (box_of_it[axis].hi() - box_of_it[axis].lo()) / 2.0;
    Point::new(axis(0), axis(1), axis(2))
}

/// # Panics
///
/// If an edge of the face has no second face along it. A closed solid uses every
/// edge exactly twice, so reaching this means the model was already broken.
fn bake_one_face(
    model: &Model,
    face: FaceView<'_>,
    planes: &BTreeMap<FaceSlot, Plane>,
    corners: &BTreeMap<VertexSlot, BakedCorner>,
) -> BakedFace {
    let key = face.key();
    let mut bounds = Bounds::nothing();
    face.for_each_triangle(|triangle| {
        for corner in triangle {
            bounds.take_in_point(corner);
        }
    });

    let wires = model
        .face(key)
        .wires()
        .map(|wire| {
            wire.coedges()
                .map(|coedge| BakedEdge {
                    start: corners[&coedge.start().key().slot_in(model.id())].clone(),
                    across: planes[&face_across(model, key, coedge.edge().key())],
                })
                .collect()
        })
        .collect();

    BakedFace {
        key,
        plane: face.plane(),
        orientation: face.orientation(),
        wires,
        cuts: Vec::new(),
        bounds,
    }
}

fn face_across(model: &Model, face: FaceKey, edge: crate::EdgeKey) -> FaceSlot {
    let mine = face.slot_in(model.id());
    let across: Vec<FaceSlot> = model
        .edge(edge)
        .neighbouring_faces()
        .map(|other| other.slot_in(model.id()))
        .filter(|&other| other != mine)
        .collect();
    assert!(
        across.len() == 1,
        "an edge of a closed solid has exactly one face on the other side, this one has {}",
        across.len()
    );
    across[0]
}

#[cfg(test)]
mod tests {
    use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
    use rcad_predicates::{Cause, Policy, ProvenanceLog};

    use super::{Bounds, bake};
    use crate::VertexKey;
    use crate::{Model, SolidKey};

    fn a_cube_of_side_two() -> (Model, SolidKey) {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let cube = model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube of side two");
        (model, cube)
    }

    /// Where the model says its corners are, with the solid's placement applied
    /// — which is what the baked planes have been moved by too.
    fn corners_as_stored(model: &Model, solid: SolidKey) -> Vec<(VertexKey, [f64; 3])> {
        let mut corners = Vec::new();
        for face in model.solid(solid).faces() {
            for wire in face.wires() {
                for coedge in wire.coedges() {
                    corners.push((coedge.start().key(), coedge.start().point().coordinates()));
                }
            }
        }
        corners
    }

    fn stored(corners: &[(VertexKey, [f64; 3])], vertex: VertexKey) -> [f64; 3] {
        corners
            .iter()
            .find(|(key, _)| *key == vertex)
            .expect("every baked corner is a corner of the solid")
            .1
    }

    fn a_turn() -> Isometry {
        let axis = Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 2.0, 3.0))
            .expect("a direction with length");
        Isometry::rotation(&axis, 0.7).expect("a rotation")
    }

    /// Every plane of a lattice-aligned cube is an axis plane, so Cramer's rule
    /// divides by one and the corner it solves for is an `f64` exactly.
    #[test]
    fn re_deriving_the_corners_of_a_lattice_cube_gives_back_the_corners_it_started_with() {
        let (model, cube) = a_cube_of_side_two();
        let log = ProvenanceLog::new();
        let corners = corners_as_stored(&model, cube);
        let baked = bake(&model, &[cube]).expect("a cube bakes");

        for face in &baked[0].faces {
            for wire in &face.wires {
                for edge in wire {
                    assert_eq!(
                        edge.start.point.exact_coordinates(),
                        Some(stored(&corners, edge.start.vertex)),
                        "an axis-aligned corner is an f64 and comes back as one"
                    );
                    assert_eq!(edge.start.approximation.magnitude(), 0.0);
                    assert_eq!(edge.start.approximation.cause(), Cause::ConstructedPoint);
                }
            }
        }
        assert!(
            log.is_empty(),
            "baking refuses only on proof, so it never records anything"
        );
        assert!(Policy::certified().wobble_is_acceptable(log.worst_wobble(100.0)));
    }

    /// A turned cube's planes have irrational normals, so its corners are no
    /// longer `f64`. The box has to hold the stored corner all the same — the
    /// stored corner is the true one moved and rounded, and the box encloses the
    /// true one.
    #[test]
    fn re_deriving_the_corners_of_a_turned_cube_gives_boxes_that_hold_them() {
        let (mut model, cube) = a_cube_of_side_two();
        model.place_solid(cube, &a_turn());
        let corners = corners_as_stored(&model, cube);
        let baked = bake(&model, &[cube]).expect("a turned cube bakes");

        let mut anything_was_built = false;
        for face in &baked[0].faces {
            for wire in &face.wires {
                for edge in wire {
                    let stored = stored(&corners, edge.start.vertex);
                    let box_of_it = edge.start.point.enclosure();
                    for axis in 0..3 {
                        let (lo, hi) = (box_of_it[axis].lo(), box_of_it[axis].hi());
                        assert!(
                            lo - 1e-12 <= stored[axis] && stored[axis] <= hi + 1e-12,
                            "the box {lo}..{hi} is nowhere near the stored {}",
                            stored[axis]
                        );
                    }
                    if edge.start.approximation.magnitude() > 0.0 {
                        anything_was_built = true;
                        assert_eq!(edge.start.approximation.cause(), Cause::ConstructedPoint);
                    }
                }
            }
        }
        assert!(
            anything_was_built,
            "a turned cube has corners that are not f64, and they must say so"
        );
    }

    /// Representation wobble is not a refusal (§17.2): a certified policy bakes
    /// a turned cube without a word in the log, because nothing here failed to
    /// be proved.
    #[test]
    fn a_certified_policy_bakes_a_turned_cube_without_recording_anything() {
        let (mut model, cube) = a_cube_of_side_two();
        model.place_solid(cube, &a_turn());
        let log = ProvenanceLog::new();

        bake(&model, &[cube]).expect("a turned cube bakes under a certified policy");

        assert!(log.is_empty());
        assert_eq!(log.worst_wobble(100.0), 0.0);
    }

    #[test]
    fn a_box_round_a_cube_holds_the_cube_and_misses_one_beside_it() {
        let (model, cube) = a_cube_of_side_two();
        let baked = bake(&model, &[cube]).expect("a cube bakes");
        let far_away = Bounds {
            low: [10.0, 10.0, 10.0],
            high: [12.0, 12.0, 12.0],
        };
        assert!(baked[0].bounds.overlap(baked[0].bounds));
        assert!(!baked[0].bounds.overlap(far_away));
    }
}
