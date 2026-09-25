//! How long an edge or a wire is, against the textbook: a cube's side, the long
//! side of a right triangle, a round bar's cap `2πr`, a ball's seam `πr` — and
//! the ellipse a leaning cap leaves, refused by name.
//!
//! Every textbook value here is worked out to about a hundred bits, so the
//! oracle's own rounding sits far below the error it is checked against.

use core::f64::consts::PI;

use rcad_geom::{Axis, EdgeRun, Isometry, Plane, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{EdgeView, Error, Length, Model, ProfileBuilder, Rule, SolidKey};

/// The worst error over length [`Length`]'s doc publishes for a straight edge
/// that is not along an axis.
const STRAIGHT: f64 = 4.0 * f64::EPSILON;

/// The same for a whole circle on a pipe — a round bar's cap.
const WHOLE_CIRCLE_ON_A_PIPE: f64 = 2.0 * f64::EPSILON;

/// The same for half a circle round a ball — its seam.
const HALF_A_CIRCLE_ROUND_A_BALL: f64 = 6.0 * f64::EPSILON;

/// `π` to about a hundred bits: the nearest `f64`, and what that leaves out.
const PI_LEFT_OUT: f64 = 1.224_646_799_147_353_2e-16;

#[test]
fn every_edge_of_a_cube_is_exactly_as_long_as_the_cube_is_wide() {
    for side in [1.0, 2.0, 0.1, 3.7, 1e-3, 12_345.678, 1e150, 1e-150] {
        let mut model = Model::new(Scale::millimetres(side).expect("a scale"));
        let cube = model
            .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube");
        let edges = every_edge_of(&model, cube);
        assert_eq!(edges.len(), 24, "twelve edges, each walked twice");
        for edge in edges {
            let length = edge.length().expect("a straight edge has a length");
            assert_eq!(
                (length.length(), length.error()),
                (side, 0.0),
                "an edge along an axis is one difference, taken exactly"
            );
        }
    }
}

/// Four times a side is exact, so the textbook value is exact too. Adding the
/// four up is not always: `0.1 + 0.1 + 0.1` rounds, and a sum that rounded
/// carries a step of the running total per edge in its error.
#[test]
fn a_cube_faces_outer_wire_is_four_sides_long() {
    for side in [1.0, 0.5, 0.1, 3.7, 1e-3, 12_345.678] {
        let mut model = Model::new(Scale::millimetres(side).expect("a scale"));
        let cube = model
            .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube");
        for face in model.solid(cube).expect("the cube").faces() {
            let length = face.outer_wire().length().expect("four straight edges");
            assert!(
                (length.length() - 4.0 * side).abs() <= length.error(),
                "{length:?} against four sides of {side}"
            );
            assert!(length.error() <= 3.0 * f64::EPSILON * length.length());
            if side == 1.0 || side == 0.5 {
                assert_eq!(length.error(), 0.0, "no sum of halves rounds");
            }
        }
    }
}

#[test]
fn the_long_side_of_a_right_triangle_is_the_root_of_the_squares_of_the_other_two() {
    let mut worst: f64 = 0.0;
    for (across, up) in [
        (1.0, 1.0),
        (1.0, 2.0),
        (1.5, 2.25),
        (3.0, 7.0),
        (1e-3, 3e-3),
    ] {
        let mut model = Model::new(Scale::millimetres(across).expect("a scale"));
        let mut log = ProvenanceLog::new();
        let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [across, 0.0], [0.0, up]])
            .build(&Policy::certified(), &mut log)
            .expect("a right triangle");
        let prism = model
            .add_extrusion(&triangle, 1.0, &Isometry::identity())
            .expect("a prism");
        let long_sides: Vec<Length> = every_edge_of(&model, prism)
            .into_iter()
            .filter(|edge| {
                let [from, to] = [edge.start().point(), edge.end().point()];
                from.x() != to.x() && from.y() != to.y()
            })
            .map(|edge| edge.length().expect("a straight edge has a length"))
            .collect();
        assert_eq!(long_sides.len(), 4, "top and bottom, each walked twice");
        for length in long_sides {
            let off = how_far_from_the_root_of(length.length(), across * across + up * up);
            assert!(
                off.abs() <= length.error(),
                "{length:?} is {off:e} from the textbook for {across} by {up}"
            );
            worst = worst.max(length.error() / length.length());
        }
    }
    println!(
        "straight, off an axis: worst error / length {worst:e}, {} × f64::EPSILON",
        worst / f64::EPSILON
    );
    assert!(
        worst <= STRAIGHT,
        "{worst:e} is past the published {STRAIGHT:e}"
    );
}

#[test]
fn a_round_bars_cap_is_two_pi_r_round() {
    let mut worst: f64 = 0.0;
    for radius in [1.0, 2.5, 0.1, 1e-3, 1e3, 7.3, 1e100] {
        let mut model = Model::new(Scale::millimetres(radius).expect("a scale"));
        let bar = model
            .add_round_bar(radius, 3.0 * radius, &Isometry::identity())
            .expect("a bar");
        let caps: Vec<Length> = every_edge_of(&model, bar)
            .into_iter()
            .filter(|edge| edge.is_closed().expect("a run this crate knows"))
            .map(|edge| edge.length().expect("a circle has a length"))
            .collect();
        assert_eq!(caps.len(), 4, "two caps, each walked twice");
        for length in caps {
            let off = how_far_from_pi_times(length.length(), 2.0, radius);
            assert!(
                off.abs() <= length.error(),
                "{length:?} is {off:e} from 2π·{radius}"
            );
            worst = worst.max(length.error() / length.length());
        }
    }
    println!(
        "a whole circle on a pipe: worst error / length {worst:e}, {} × f64::EPSILON",
        worst / f64::EPSILON
    );
    assert!(
        worst <= WHOLE_CIRCLE_ON_A_PIPE,
        "{worst:e} is past the published {WHOLE_CIRCLE_ON_A_PIPE:e}"
    );
}

#[test]
fn a_round_bars_cap_wire_is_its_one_circle() {
    let radius = 2.5;
    let mut model = Model::new(Scale::millimetres(radius).expect("a scale"));
    let bar = model
        .add_round_bar(radius, 4.0, &Isometry::identity())
        .expect("a bar");
    let solid = model.solid(bar).expect("the bar");
    let flat = solid
        .faces()
        .filter(|face| face.surface().as_plane().is_some())
        .collect::<Vec<_>>();
    assert_eq!(flat.len(), 2, "two caps");
    for cap in flat {
        let length = cap.outer_wire().length().expect("one circle");
        let off = how_far_from_pi_times(length.length(), 2.0, radius);
        assert!(
            off.abs() <= length.error(),
            "{length:?} is {off:e} from 2π·{radius}"
        );
    }
}

#[test]
fn a_balls_seam_is_half_way_round_it() {
    let mut worst: f64 = 0.0;
    for radius in [1.0, 2.5, 0.1, 1e-3, 1e3, 7.3, 1e100] {
        let mut model = Model::new(Scale::millimetres(radius).expect("a scale"));
        let ball = model
            .add_ball(radius, &Isometry::identity())
            .expect("a ball");
        let seams = every_edge_of(&model, ball);
        assert_eq!(seams.len(), 2, "one seam, walked down and back up");
        for seam in seams {
            let length = seam.length().expect("half a circle has a length");
            let off = how_far_from_pi_times(length.length(), 1.0, radius);
            assert!(
                off.abs() <= length.error(),
                "{length:?} is {off:e} from π·{radius}"
            );
            worst = worst.max(length.error() / length.length());
        }
        let face = model
            .solid(ball)
            .expect("the ball")
            .faces()
            .next()
            .expect("one face");
        let round = face.outer_wire().length().expect("the seam, twice");
        let off = how_far_from_pi_times(round.length(), 2.0, radius);
        assert!(
            off.abs() <= round.error(),
            "{round:?} is {off:e} from 2π·{radius}"
        );
    }
    println!(
        "half a circle round a ball: worst error / length {worst:e}, {} × f64::EPSILON",
        worst / f64::EPSILON
    );
    assert!(
        worst <= HALF_A_CIRCLE_ROUND_A_BALL,
        "{worst:e} is past the published {HALF_A_CIRCLE_ROUND_A_BALL:e}"
    );
}

/// A quarter of a bar and the three quarters left over, cut by a block over
/// one quadrant: every cap edge is an arc the pickers have to choose, because
/// its two ends bound a quarter circle one way round and three quarters the
/// other — the choice a ball's seam, half way either way, cannot tell apart.
#[test]
fn a_bar_cut_by_a_block_over_one_quadrant_has_quarter_and_three_quarter_arcs() {
    let radius = 1.0;
    for (rule, turns) in [(Rule::Intersection, 0.5), (Rule::Difference, 1.5)] {
        let mut model = Model::new(Scale::millimetres(10.0).expect("a scale"));
        let mut log = ProvenanceLog::new();
        let policy = Policy::certified();
        let bar = model
            .add_round_bar(radius, 2.0, &Isometry::identity())
            .expect("a bar");
        let square = ProfileBuilder::new(vec![[-3.0, -3.0], [0.0, -3.0], [0.0, 0.0], [-3.0, 0.0]])
            .build(&policy, &mut log)
            .expect("a square");
        let block = model
            .add_extrusion(
                &square,
                4.0,
                &Isometry::translation(Vector::new(0.0, 0.0, -1.0)).expect("a shift"),
            )
            .expect("a block");
        let fused = model
            .fuse(&[bar], &[block], rule, &policy, &mut log)
            .expect("a bar and a block fuse");
        let [piece] = fused.solids() else {
            panic!("one piece, got {:?}", fused.solids())
        };
        for edge in every_edge_of(&model, *piece) {
            let run = edge.run();
            if let Err(refusal) = edge.length() {
                panic!("{rule:?}: an edge along {run:?} refused with {refusal:?}");
            }
        }
        let solid = model.solid(*piece).expect("the piece");
        let caps: Vec<Length> = solid
            .faces()
            .filter(|face| {
                face.surface()
                    .as_plane()
                    .is_some_and(|plane| plane.normal().z() != 0.0)
            })
            .map(|cap| cap.outer_wire().length().expect("arcs and straight sides"))
            .collect();
        assert_eq!(caps.len(), 2, "{rule:?}: a top and a bottom");
        for length in caps {
            let off = how_far_from_pi_times_and(length.length(), turns, radius, 2.0 * radius);
            assert!(
                off.abs() <= length.error(),
                "{rule:?}: {length:?} is {off:e} from {turns}·π·{radius} and two radii"
            );
        }
    }
}

/// A movement changes no length, and the answer does not change with one
/// either — not by a bit — because it is read from the frame the solid is
/// stored in.
#[test]
fn the_same_solids_a_million_units_away_and_turned_measure_the_same_to_the_bit() {
    let turned = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("an axis"),
        0.7,
    )
    .expect("a turn");
    let away = Isometry::translation(Vector::new(1e6, -2e6, 3e6))
        .expect("a shift")
        .compose(&turned);
    let mut model = Model::new(Scale::millimetres(10.0).expect("a scale"));
    let mut log = ProvenanceLog::new();
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [3.0, 0.0], [1.0, 2.5]])
        .build(&Policy::certified(), &mut log)
        .expect("a triangle");
    let pairs = [
        (
            model.add_extrusion(&triangle, 4.0, &Isometry::identity()),
            model.add_extrusion(&triangle, 4.0, &away),
        ),
        (
            model.add_round_bar(2.5, 4.0, &Isometry::identity()),
            model.add_round_bar(2.5, 4.0, &away),
        ),
        (
            model.add_ball(2.5, &Isometry::identity()),
            model.add_ball(2.5, &away),
        ),
    ];
    for (home, far) in pairs {
        let [home, far] = [home, far].map(|solid| {
            every_edge_of(&model, solid.expect("a solid"))
                .into_iter()
                .map(|edge| edge.length().expect("a length"))
                .collect::<Vec<_>>()
        });
        assert!(!home.is_empty());
        assert_eq!(home, far);
    }
}

#[test]
fn a_bar_cut_off_at_a_lean_measures_its_square_cap_and_refuses_its_ellipse_by_name() {
    let radius = 1.0;
    let mut model = Model::new(Scale::millimetres(radius).expect("a scale"));
    let square = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the bottom cap");
    let leaning =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 5.0), Vector::new(0.3, 0.0, 1.0))
            .expect("the top cap");
    let bar = model
        .add_bar_cut_off_by(
            radius,
            [square, leaning],
            &Isometry::identity(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar with a leaning top");
    let solid = model.solid(bar).expect("the bar");
    for cap in solid
        .faces()
        .filter(|face| face.surface().as_plane().is_some())
    {
        let circle = cap
            .outer_wire()
            .coedges()
            .next()
            .expect("one edge round the cap")
            .edge();
        let answer = circle.length();
        if cap.surface().as_plane() == Some(&square) {
            let length = answer.expect("the square cap is a circle");
            let off = how_far_from_pi_times(length.length(), 2.0, radius);
            assert!(off.abs() <= length.error(), "{length:?} is {off:e} from 2π");
        } else {
            let refusal = answer.expect_err("the leaning cap is an ellipse");
            assert!(
                matches!(
                    refusal,
                    Error::CannotYetMeasureHowLongThisCurveIs { edge } if edge == circle.key()
                ),
                "{refusal:?}"
            );
            println!("the leaning cap: {refusal}");
            assert!(
                cap.outer_wire().length().is_err(),
                "a wire with an edge that refuses refuses too"
            );
        }
    }
}

/// Every edge of `solid`, once for each time a wire walks it, in the order its
/// faces, their wires and those wires' coedges come.
fn every_edge_of(model: &Model, solid: SolidKey) -> Vec<EdgeView<'_>> {
    model
        .solid(solid)
        .expect("the solid")
        .faces()
        .flat_map(|face| face.wires().collect::<Vec<_>>())
        .flat_map(|wire| wire.coedges().collect::<Vec<_>>())
        .map(|coedge| coedge.edge())
        .collect()
}

/// How far `measured` is from `turns · π · radius`, worked out in about a
/// hundred bits.
///
/// Every `turns` here is a half, one, one and a half or two, and every radius
/// a short binary fraction or a power of ten whose product with those is still
/// exact — so `turns · radius` rounds nothing; the product with `π` is
/// split into its rounded part and what the rounding lost; and the difference
/// with `measured` is exact wherever the two are within a factor of two.
fn how_far_from_pi_times(measured: f64, turns: f64, radius: f64) -> f64 {
    how_far_from_pi_times_and(measured, turns, radius, 0.0)
}

/// How far `measured` is from `turns · π · radius + straight`, the same way,
/// with the sum split into its rounded part and what the rounding lost.
fn how_far_from_pi_times_and(measured: f64, turns: f64, radius: f64, straight: f64) -> f64 {
    let scaled = turns * radius;
    let product = scaled * PI;
    let lost = scaled.mul_add(PI, -product);
    let sum = product + straight;
    let summed_away = (product - (sum - (sum - product))) + (straight - (sum - product));
    (sum - measured) + (summed_away + lost + scaled * PI_LEFT_OUT)
}

/// How far `measured` is from `√square`, worked out in about a hundred bits:
/// the rounded root, and one Newton step's correction to it.
fn how_far_from_the_root_of(measured: f64, square: f64) -> f64 {
    let root = square.sqrt();
    let correction = -root.mul_add(root, -square) / (2.0 * root);
    (root - measured) + correction
}
