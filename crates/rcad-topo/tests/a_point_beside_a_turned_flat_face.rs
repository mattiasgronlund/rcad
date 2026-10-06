//! A point a rounding from a turned block's side, on a solid that also has a
//! torus face, whose ray is fired from an exact point beside it
//! (`docs/decisions.md` §323.3, Q1068).
//!
//! A torus is asked where a ray meets it only along a line whose numbers are
//! exact, and most points have no such line to any target. So the ray is fired
//! from an exact point beside the asked one instead, once no face of the solid
//! is shown to meet the box round the two, and its answer is taken for the
//! asked point. That check reads a flat face with straight sides by its fanned
//! triangles. A turn leaves the corners of a block's side a rounding off its
//! stored plane, which is the face (§17.1), so the box round a point just off
//! the side and the point beside it can miss every triangle while the stored
//! plane passes through it, over the side.
//!
//! Then the point beside is in the side's sliver and the asked point is not.
//! The ray from it passes through the side's triangle, where the triangle and
//! the stored plane read the crossing differently, and the ray's count leaves
//! that crossing out. The count without it was taken as the answer, one off,
//! and the point came back on the wrong side of the side. Now the side's stored
//! plane places the asked point itself, as it does where a ray from the point
//! itself parts at a face.
//!
//! The solid is a block with a quarter bend growing out of one side, built face
//! by face and turned (`tests/a_ray_from_beside_a_box.rs` builds it unturned).
//! The block is convex and each point here is far from the bend, so the block's
//! six stored planes say where each point is.

use std::collections::BTreeMap;

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale,
    Surface, Torus, Vector,
};
use rcad_predicates::{
    Decision, Point as PredicatePoint, Policy, ProvenanceLog, orient3d_of_points,
};
use rcad_topo::{
    Containment, DraftStep, FaceView, Model, Result, SolidBuilder, SolidKey, SolidView,
};
use smallvec::smallvec;

/// A point just inside the block's floor `z = −2` as turned, by its stored
/// plane and by both its triangles. The point the ray is fired from instead is
/// a few spacings away, past the stored plane and still inside the triangles,
/// and its ray went out through the floor without counting it: `Outside`.
#[test]
fn a_point_inside_whose_ray_from_beside_parts_at_the_floor_is_not_put_outside() {
    assert_placed_by_the_stored_planes(
        [3.0, -1.0, 2.0],
        1.5,
        [
            2.473_345_055_686_643,
            2.730_108_300_387_340_8,
            0.655_036_566_663_705_8,
        ],
        Containment::Inside,
    );
}

/// A point just outside the block's top `z = 2` as turned, by its stored plane
/// and by both its triangles. The point the ray is fired from instead is a few
/// spacings away along `y`, inside the stored plane and still outside the
/// triangles, and its ray came in through the top without counting it and
/// went out through the floor: `Inside`.
#[test]
fn a_point_outside_whose_ray_from_beside_parts_at_the_top_is_not_put_inside() {
    assert_placed_by_the_stored_planes(
        [3.0, -1.0, 2.0],
        2.5,
        [
            4.066_396_927_615_434,
            -0.795_879_808_369_035_8,
            3.752_464_704_392_332,
        ],
        Containment::Outside,
    );
}

/// That `point`, on the block with a bend turned `angle` about `axis`, is in no
/// side's sliver, that the point the first ray is fired from instead is in one
/// side's sliver across that side's stored plane from `point`, with its ray's
/// line through one of that side's triangles, and that `point` is placed where
/// the stored planes put it, `should`, under both policies.
fn assert_placed_by_the_stored_planes(
    axis: [f64; 3],
    angle: f64,
    point: [f64; 3],
    should: Containment,
) {
    let (model, solid) = a_bend_out_of_a_turned_block(axis, angle);
    let view = model.solid(solid).expect("this model made the key");
    let sides = the_blocks_sides(view);
    assert_eq!(what_the_stored_planes_say(&sides, point), should);
    for side in &sides {
        assert!(!its_corners_and_plane_part_at(*side, point));
    }

    let target = the_first_ray_target(view);
    let beside = the_point_the_ray_is_fired_from(point, target);
    assert_ne!(
        beside, point,
        "the step to the target is rounded, so the torus cannot take the point's own ray"
    );
    let parting: Vec<&FaceView<'_>> = sides
        .iter()
        .filter(|side| its_corners_and_plane_part_at(**side, beside))
        .collect();
    let [side] = parting[..] else {
        panic!("one side parts about {beside:?}, and {} do", parting.len());
    };
    assert_ne!(
        side_of_its_plane(*side, point),
        side_of_its_plane(*side, beside),
        "the side's stored plane passes between the point and the point beside it"
    );
    assert!(
        its_line_passes_through(*side, beside, target),
        "the ray from {beside:?} passes through the side's triangle"
    );

    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        assert_eq!(
            view.classify_point(
                Point::new(point[0], point[1], point[2]),
                &policy,
                &mut ProvenanceLog::new()
            ),
            should,
            "{policy:?}"
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Side {
    Inner,
    Outer,
    On,
}

/// The block is convex, so it holds a point exactly when every one of its
/// stored planes puts the point on its side.
fn what_the_stored_planes_say(sides: &[FaceView<'_>], point: [f64; 3]) -> Containment {
    let placed: Vec<Side> = sides
        .iter()
        .map(|side| side_of_its_plane(*side, point))
        .collect();
    if placed.contains(&Side::Outer) {
        Containment::Outside
    } else if placed.contains(&Side::On) {
        Containment::OnBoundary
    } else {
        Containment::Inside
    }
}

fn side_of_its_plane(face: FaceView<'_>, point: [f64; 3]) -> Side {
    read(
        face.plane().side_of_point(PredicatePoint::from(point)),
        the_outer_sign(face),
    )
}

fn the_outer_sign(face: FaceView<'_>) -> Decision {
    if face.orientation().is_reversed() {
        Decision::Neg
    } else {
        Decision::Pos
    }
}

/// The side an exact sign puts a point on, where `outer` is the sign of the
/// outer side.
fn read(side: Decision, outer: Decision) -> Side {
    assert!(
        matches!(side, Decision::Pos | Decision::Neg | Decision::Zero),
        "an exact point is always placed: {side:?}"
    );
    if side == Decision::Zero {
        Side::On
    } else if side == outer {
        Side::Outer
    } else {
        Side::Inner
    }
}

/// Whether some triangle of `face` puts `point` on the other side from the
/// face's stored plane. A triangle's outer side is the side it puts a point one
/// unit out along the face's outward normal from its middle.
fn its_corners_and_plane_part_at(face: FaceView<'_>, point: [f64; 3]) -> bool {
    let by_the_plane = side_of_its_plane(face, point);
    let outward = face.outward_normal().components();
    let mut parts = false;
    face.for_each_triangle(|corners| {
        let at = corners.map(|corner| corner.coordinates());
        let out: [f64; 3] = core::array::from_fn(|axis| {
            (at[0][axis] + at[1][axis] + at[2][axis]) / 3.0 + outward[axis]
        });
        let [a, b, c] = at.map(PredicatePoint::from);
        let outer = orient3d_of_points(a, b, c, PredicatePoint::from(out));
        let by_the_triangle = read(
            orient3d_of_points(a, b, c, PredicatePoint::from(point)),
            outer,
        );
        parts |= by_the_triangle != Side::On
            && by_the_plane != Side::On
            && by_the_triangle != by_the_plane;
    });
    parts
}

/// Whether the line from `from` to `target` passes through a triangle of
/// `face`, by the three signs across its sides.
fn its_line_passes_through(face: FaceView<'_>, from: [f64; 3], target: [f64; 3]) -> bool {
    let [from, target] = [from, target].map(PredicatePoint::from);
    let mut through = false;
    face.for_each_triangle(|corners| {
        let [a, b, c] = corners.map(|corner| PredicatePoint::from(corner.coordinates()));
        let signs = [
            orient3d_of_points(a, b, target, from),
            orient3d_of_points(b, c, target, from),
            orient3d_of_points(c, a, target, from),
        ];
        through |= signs.iter().all(|&sign| sign == Decision::Pos)
            || signs.iter().all(|&sign| sign == Decision::Neg);
    });
    through
}

/// The point `classify.rs`'s `a_point_an_exact_ray_leaves_for` builds beside
/// an exact `point`: the step to `target` rounded, and the point read back off
/// it, whose step to `target` is then exact.
fn the_point_the_ray_is_fired_from(point: [f64; 3], target: [f64; 3]) -> [f64; 3] {
    core::array::from_fn(|axis| {
        let there = target[axis] - (target[axis] - point[axis]);
        let step = target[axis] - there;
        assert_eq!(
            there + step,
            target[axis],
            "the step from {there} to {} is exact",
            target[axis]
        );
        there
    })
}

/// The first of `classify.rs`'s three ray targets, worked out the way its
/// `ray_targets` does from the solid's box.
fn the_first_ray_target(view: SolidView<'_>) -> [f64; 3] {
    let (low, high) = view.box_around_it().expect("the solid has a box");
    let (low, high) = (low.coordinates(), high.coordinates());
    let span = (0..3)
        .map(|axis| high[axis] - low[axis])
        .fold(0.0, f64::max);
    [
        high[0] + span,
        high[1] + span * 0.618_033_988_749_895,
        high[2] + span * 0.414_213_562_373_095_1,
    ]
}

/// The block's six sides: the flat faces whose outer wire has four sides.
fn the_blocks_sides(view: SolidView<'_>) -> Vec<FaceView<'_>> {
    let sides: Vec<FaceView<'_>> = view
        .faces()
        .filter(|face| face.surface().as_plane().is_some() && face.outer_wire().coedge_count() == 4)
        .collect();
    assert_eq!(sides.len(), 6, "a block has six sides");
    sides
}

/// `tests/a_ray_from_beside_a_box.rs`'s block `[2, 6] × [−2, 0] × [−2, 2]`
/// with a quarter bend of ring four and tube one growing out of its side
/// `y = 0`, turned `angle` about `axis` through the origin.
fn a_bend_out_of_a_turned_block(axis: [f64; 3], angle: f64) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_bend_out_of_a_block(&mut model).expect("a bend growing out of a block's side");
    let turn = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(axis[0], axis[1], axis[2])).expect("an axis"),
        angle,
    )
    .expect("a finite turn");
    model
        .place_solid(solid, &turn)
        .expect("this model made the key");
    (model, solid)
}

/// The side `y = 0` holds the bend's first cap circle as a hole, walked the
/// way round that builds; the bend's wall walks it the other way. The other
/// five sides are rectangles.
fn a_bend_out_of_a_block(model: &mut Model) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let mut corners = BTreeMap::new();
    for x in [2, 6] {
        for y in [-2, 0] {
            for z in [-2, 2] {
                let at = Point::new(f64::from(x), f64::from(y), f64::from(z));
                corners.insert([x, y, z], builder.add_vertex(at)?);
            }
        }
    }
    let start_corner = builder.add_vertex(Point::new(3.0, 0.0, 0.0))?;
    let end_corner = builder.add_vertex(Point::new(0.0, 3.0, 0.0))?;
    let flat_through_the_middle =
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let seam = builder.add_edge(
        start_corner,
        end_corner,
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(flat_through_the_middle),
            picked_by: smallvec![
                Picker::InsideAPipe(Cylinder::new(Axis::z(), 4.0).expect("the ring's radius")),
                Picker::from(
                    HalfSpace::new(Vector::new(1.0, 1.0, 0.0), 0.0)
                        .expect("a normal with a length")
                ),
            ],
        },
    )?;
    let cap_circle = builder.add_edge(
        start_corner,
        start_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let end_circle = builder.add_edge(
        end_corner,
        end_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;

    let mut straight = BTreeMap::new();
    let mut shell = Vec::with_capacity(8);
    for (normal, round) in [
        ([0, 1, 0], [[2, 0, -2], [2, 0, 2], [6, 0, 2], [6, 0, -2]]),
        (
            [0, -1, 0],
            [[2, -2, -2], [6, -2, -2], [6, -2, 2], [2, -2, 2]],
        ),
        ([1, 0, 0], [[6, -2, -2], [6, 0, -2], [6, 0, 2], [6, -2, 2]]),
        ([-1, 0, 0], [[2, -2, -2], [2, -2, 2], [2, 0, 2], [2, 0, -2]]),
        ([0, 0, 1], [[2, -2, 2], [6, -2, 2], [6, 0, 2], [2, 0, 2]]),
        (
            [0, 0, -1],
            [[2, -2, -2], [2, 0, -2], [6, 0, -2], [6, -2, -2]],
        ),
    ] {
        let mut steps = Vec::with_capacity(4);
        for position in 0..4 {
            let from = round[position];
            let to = round[(position + 1) % 4];
            let (edge, turn) = if let Some(&edge) = straight.get(&(to, from)) {
                (edge, Orientation::Reversed)
            } else {
                let edge = builder.add_edge(corners[&from], corners[&to], EdgeRun::Straight)?;
                straight.insert((from, to), edge);
                (edge, Orientation::Forward)
            };
            steps.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
        }
        let mut wires = vec![builder.add_wire(&steps)?];
        if normal == [0, 1, 0] {
            let round_the_cap = builder.add_coedge(cap_circle, Orientation::Forward)?;
            wires.push(builder.add_wire(&[DraftStep::Along(round_the_cap)])?);
        }
        let [x, y, z] = normal.map(f64::from);
        let on_it = round[0].map(f64::from);
        let plane = Plane::from_point_and_normal(
            Point::new(on_it[0], on_it[1], on_it[2]),
            Vector::new(x, y, z),
        )
        .expect("a coordinate direction is a unit normal");
        shell.push((
            builder.add_face(Surface::Plane(plane), &wires)?,
            Orientation::Forward,
        ));
    }

    let end_cap = {
        let plane = Plane::from_point_and_normal(Point::origin(), Vector::new(-1.0, 0.0, 0.0))
            .expect("a coordinate direction is a unit normal");
        let coedge = builder.add_coedge(end_circle, Orientation::Reversed)?;
        let wire = builder.add_wire(&[DraftStep::Along(coedge)])?;
        builder.add_face(Surface::Plane(plane), &[wire])?
    };
    let mut round_the_wall = Vec::with_capacity(4);
    for (edge, turn) in [
        (cap_circle, Orientation::Reversed),
        (seam, Orientation::Forward),
        (end_circle, Orientation::Forward),
        (seam, Orientation::Reversed),
    ] {
        round_the_wall.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
    }
    let wire = builder.add_wire(&round_the_wall)?;
    let ring = Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring");
    let wall = builder.add_face(Surface::Torus(ring), &[wire])?;
    shell.push((end_cap, Orientation::Forward));
    shell.push((wall, Orientation::Forward));
    builder.add_shell(&shell)?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}
