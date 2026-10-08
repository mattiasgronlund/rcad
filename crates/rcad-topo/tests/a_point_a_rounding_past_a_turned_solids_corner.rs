//! A point a rounding past a turned cube's outermost corner, which every one of
//! the cube's stored planes puts inside (`docs/decisions.md` §17.1, §346.7,
//! Q1157).
//!
//! `classify_uncertain_point` answers `Outside` without firing a ray for a point
//! past the box [`SolidView::box_around_it`] draws, and that box holds a flat
//! face's stored corners. The face is its stored plane, though, and a turn
//! leaves a corner a rounding off where its three planes meet. So the cube the
//! planes bound can poke a rounding past that box at its outermost corner, and
//! a point in that tip came back `Outside` where every plane put it inside.
//! `SolidView::shares_space_with` read the same box the same way, and called a
//! block resting a spacing above that box `Apart` from the cube (Q1331).
//!
//! Every answer here is checked against the stored planes' exact signs. A cube
//! is convex, so it holds a point exactly when each of its six planes puts the
//! point on the cube's side.

use rcad_geom::{Axis, Isometry, Plane, Point, Scale, Vector};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, FaceView, Model, SharedSpace, SolidKey, SolidView};

/// A point one spacing above the top of a cube's box, turned `2.5` about
/// `(1, 1, 1)`, under the corner whose three planes meet above that box. The
/// box alone put it outside.
#[test]
fn a_point_above_a_turned_cubes_box_that_every_plane_puts_inside_is_inside() {
    let (model, cube) = a_cube_of_side_2_turned([1.0, 1.0, 1.0], 2.5);
    let view = model.solid(cube).expect("this model made the key");
    let point = [
        -0.891_818_517_366_445_4,
        0.490_293_696_637_200_8,
        1.401_524_820_729_244_6,
    ];
    let (_, high) = view.box_around_it().expect("a cube has a box");
    assert_eq!(high.coordinates()[2].next_up(), point[2]);
    assert_eq!(what_the_stored_planes_say(view, point), Containment::Inside);
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        assert_eq!(
            view.classify_point(
                Point::new(point[0], point[1], point[2]),
                &policy,
                &mut ProvenanceLog::new()
            ),
            Containment::Inside,
            "{policy:?}"
        );
    }
}

/// A block of side 1 standing one spacing above the box of the cube above,
/// over the corner whose three planes meet above that box (Q1331).
///
/// The two boxes miss, and the boxes alone answered `Apart` both ways round,
/// which is a proof that no point is in both. Yet a point three spacings above
/// the cube's box, so two into the block, is inside both by every stored plane
/// of each. Both are convex, so that is exact, and `Shared` is the whole
/// answer. No corner of either is inside the other and no two faces share a
/// plane, so the search finds nothing: `NothingFound`, which claims nothing.
#[test]
fn a_block_one_spacing_above_a_turned_cubes_box_is_not_called_apart_from_it() {
    let policy = Policy::certified();
    let (mut model, cube) = a_cube_of_side_2_turned([1.0, 1.0, 1.0], 2.5);
    let block = model
        .add_cube(1.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side 1");
    model
        .place_solid(
            block,
            &Isometry::translation(Vector::new(-1.4, 0.0, 1.401_524_820_729_244_6))
                .expect("a finite shift"),
        )
        .expect("this model made the key");
    let cube = model.solid(cube).expect("this model made the key");
    let block = model.solid(block).expect("this model made the key");

    let (_, top_of_the_cube) = cube.box_around_it().expect("a cube has a box");
    let (bottom_of_the_block, _) = block.box_around_it().expect("a cube has a box");
    assert_eq!(
        top_of_the_cube.coordinates()[2].next_up(),
        bottom_of_the_block.coordinates()[2]
    );
    let in_both = [
        -0.891_818_517_366_446,
        0.490_293_696_637_201_5,
        1.401_524_820_729_245,
    ];
    assert_eq!(
        what_the_stored_planes_say(cube, in_both),
        Containment::Inside
    );
    assert_eq!(
        what_the_stored_planes_say(block, in_both),
        Containment::Inside
    );

    for [one, other] in [[cube, block], [block, cube]] {
        assert_eq!(
            one.shares_space_with(other, &policy, &mut ProvenanceLog::new()),
            SharedSpace::NothingFound
        );
    }
}

/// Forty cubes of side 2, five axes by eight angles. At every corner outermost
/// along an axis, every `f64` point within six spacings of the box round where
/// that corner's three planes meet, past the cube's box and put inside by every
/// stored plane, is classified.
///
/// The box alone answered all 3897 of them `Outside`. Now 1 is `Inside` and
/// 3811 are `Undecidable`. The 85 still `Outside` are answered by the ray,
/// past the box: one face's corners and plane part about each, and
/// `is_beside` lets it through because that face's outline, read at the plane,
/// holds none of the point's box stretched there, so the corners count it
/// (Q1156's first shape, §346.7).
#[test]
fn how_often_a_point_past_a_turned_cubes_box_that_every_plane_puts_inside_is_answered_outside() {
    let axes = [
        [1.0, 2.0, 3.0],
        [3.0, -1.0, 2.0],
        [1.0, 1.0, 1.0],
        [-2.0, 1.0, 5.0],
        [5.0, 3.0, -1.0],
    ];
    let angles = [0.3, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 5.0];
    let (mut points, mut inside, mut undecided, mut outside) = (0, 0, 0, 0);
    for axis in axes {
        for angle in angles {
            let (model, cube) = a_cube_of_side_2_turned(axis, angle);
            let view = model.solid(cube).expect("this model made the key");
            for point in points_past_its_box_every_plane_puts_inside(view) {
                points += 1;
                let answer = view.classify_point(
                    Point::new(point[0], point[1], point[2]),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                if answer == Containment::Inside {
                    inside += 1;
                } else if matches!(answer, Containment::Undecidable { .. }) {
                    undecided += 1;
                } else {
                    assert_eq!(answer, Containment::Outside, "{point:?}");
                    outside += 1;
                }
            }
        }
    }
    println!(
        "{points} points past the box that every plane puts inside: \
         {inside} Inside, {undecided} Undecidable, {outside} Outside"
    );
    assert_eq!((points, inside, undecided, outside), (3897, 1, 3811, 85));
}

/// Every `f64` point near a corner outermost along an axis that is past the
/// box along that axis and that every stored plane puts inside.
fn points_past_its_box_every_plane_puts_inside(view: SolidView<'_>) -> Vec<[f64; 3]> {
    let (low, high) = view.box_around_it().expect("a cube has a box");
    let (low, high) = (low.coordinates(), high.coordinates());
    let faces: Vec<FaceView<'_>> = view.faces().collect();
    let mut found = Vec::new();
    for corner in corners_of(&faces) {
        let around: Vec<Plane> = faces
            .iter()
            .filter(|face| {
                face.outer_wire()
                    .points()
                    .any(|at| at.coordinates() == corner)
            })
            .map(|face| face.plane())
            .collect();
        assert_eq!(around.len(), 3, "a cube's corner has three faces round it");
        let meet = Plane::meet_of_three(&around[0], &around[1], &around[2])
            .expect("a cube's three planes at a corner meet in a point")
            .enclosure();
        for axis in 0..3 {
            for (bound, is_past) in [
                (high[axis], f64::gt as fn(&f64, &f64) -> bool),
                (low[axis], f64::lt),
            ] {
                if corner[axis] != bound {
                    continue;
                }
                let near: [Vec<f64>; 3] = core::array::from_fn(|along| {
                    f64s_between(meet[along].lo(), meet[along].hi(), 6)
                        .into_iter()
                        .filter(|&at| along != axis || is_past(&at, &bound))
                        .collect()
                });
                for &x in &near[0] {
                    for &y in &near[1] {
                        for &z in &near[2] {
                            if what_the_stored_planes_say(view, [x, y, z]) == Containment::Inside {
                                found.push([x, y, z]);
                            }
                        }
                    }
                }
            }
        }
    }
    found
}

fn corners_of(faces: &[FaceView<'_>]) -> Vec<[f64; 3]> {
    let mut corners: Vec<[f64; 3]> = Vec::new();
    for face in faces {
        for at in face.outer_wire().points() {
            if !corners.contains(&at.coordinates()) {
                corners.push(at.coordinates());
            }
        }
    }
    corners
}

/// Every `f64` from `spacings` below `lo` to `spacings` above `hi`.
fn f64s_between(lo: f64, hi: f64, spacings: usize) -> Vec<f64> {
    let (mut at, mut end) = (lo, hi);
    for _ in 0..spacings {
        at = at.next_down();
        end = end.next_up();
    }
    let mut every = Vec::new();
    while at <= end {
        every.push(at);
        at = at.next_up();
    }
    every
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Side {
    Inner,
    Outer,
    On,
}

/// The cube is convex, so it holds a point exactly when every stored plane
/// puts the point on the cube's side.
fn what_the_stored_planes_say(view: SolidView<'_>, point: [f64; 3]) -> Containment {
    let sides: Vec<Side> = view
        .faces()
        .map(|face| side_of_its_plane(face, point))
        .collect();
    if sides.contains(&Side::Outer) {
        Containment::Outside
    } else if sides.contains(&Side::On) {
        Containment::OnBoundary
    } else {
        Containment::Inside
    }
}

fn side_of_its_plane(face: FaceView<'_>, point: [f64; 3]) -> Side {
    let outer = if face.orientation().is_reversed() {
        Decision::Neg
    } else {
        Decision::Pos
    };
    let side = face.plane().side_of_point(PredicatePoint::from(point));
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

/// A cube of side 2 about the origin, turned `angle` about `axis` through it.
fn a_cube_of_side_2_turned(axis: [f64; 3], angle: f64) -> (Model, SolidKey) {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let cube = model
        .add_cube(2.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side 2");
    let turn = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(axis[0], axis[1], axis[2])).expect("an axis"),
        angle,
    )
    .expect("a finite turn");
    let centred = Isometry::translation(Vector::new(-1.0, -1.0, -1.0)).expect("a finite shift");
    model
        .place_solid(cube, &turn.compose(&centred))
        .expect("this model made the key");
    (model, cube)
}
