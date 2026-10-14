//! Two slanted walls of a prism meet in the `z` axis, and a ball grazes that
//! edge: going round either wall's circle from where it starts, the two places
//! the edge cuts the ball sweep exactly zero.
//!
//! # The shape
//!
//! A prism on the triangle `(0, 0)`, `(4, 3)`, `(−3, 3)`, 32 tall. Its walls
//! `3x = 4y` and `x = −y` both hold the `z` axis, and their stored normals,
//! `(0.6, −0.8, 0)` and `(−0.7071067811865476, −0.7071067811865476, 0)`, have
//! two parts that are not zero. Both offsets are exactly zero, so every point
//! `(0, 0, z)` is on both walls bit for bit.
//!
//! A ball of radius `1 + 2⁻⁵¹` at `(1, 2⁻²⁵, 16)`. The `z` axis misses its
//! centre by `√(1 + 2⁻⁵⁰)`, so it cuts the ball at `z = 16 ± 2⁻⁵¹`, which are
//! not `f64`s: `(2m)² + (2m²)² + 1 = (2m² + 1)²` with `m = 2²⁵`, scaled by
//! `2⁻⁵¹`, as in `docs/decisions.md` §244.2.
//!
//! # Why the sweep is zero
//!
//! Both walls' circles start at exactly `(0, 0, 16)`, on the axis between the
//! two places and `2⁻¹⁰³` inside the ball, so off the circle by a rounding.
//! The two places come back as boxes whose `x` and `y` are exactly zero and
//! whose `z` sides overlap. Every point of the three is on one line along `z`,
//! so the sweep `n · ((one − from) × (other − from))` multiplies each `z` by
//! an exact zero and comes out `[0, 0]`: proved zero for two different points.
//! Round either circle from a start that is not on it, "which comes first" has
//! no answer the boxes can give.
//!
//! # What this pins
//!
//! Until §244.5, `round_a_closed_crossing` answered `Zero` for that pair
//! because the boxes overlap, and the two places were folded into one node
//! without a policy being asked. Every certified fuse then refused on the turn
//! at the folded node. Now the pair goes to the policy: a certified fuse
//! refuses on the order of the two places, and a tolerant one merges them and
//! its intersection is the cap past the first wall. The union and the
//! difference under a tolerant policy do not mesh the ball's face, which is
//! another row's to mend (§244.5), and are not pinned here.
//!
//! # Why Q699 does not order these two
//!
//! Q699 orders two places whose boxes hold the start **at one end each**, as
//! the edge grazing a ball at `x = 16` has them: the one point of such a box
//! at the start is the start, which is off the ball, so the box proves its
//! place lies past the start or short of it. Here both `z` boxes hold `16`
//! inside them, and each is wide enough, `2.3e-14`, to hold both true places
//! `16 ± 2⁻⁵¹`. So nothing in either box says which of the two places it is,
//! and the order still goes to the policy. What does say it is which root of
//! the line's quadratic each meet is, and that is `rcad-geom`'s to hand over.
//!
//! # Two balls cut by `z = 1`
//!
//! Q74 also reported a circle where two balls meet reaching the overlap arm.
//! Two balls of radius `1 + 2⁻⁵¹` at `(0, y, 0)` and `(2⁻²⁴, y, 0)` cross in a
//! circle in the plane `x = 2⁻²⁵`, and `z = 1` cuts that circle at
//! `(2⁻²⁵, y ± 2⁻⁵¹, 1)`. At `y = 16` those two are not `f64`s. They come back
//! as two boxes with `x` and `z` exact that touch at `y = 16`, and round the two
//! balls' circle from its start `(2⁻²⁵, 16, 1)` they sweep exactly zero. At
//! `y = 0` they are exact and apart.
//!
//! The fuses are a whole ball at `(0, y, 0)` against the other ball with its
//! cap above `z = 1` cut off. At `y = 16` a certified fuse refuses on the
//! order of the two places round a ball's circle in `z = 1`, which does not
//! sweep them to zero. A tolerant fuse reaches the overlap arm, takes the order
//! there, and goes on to §346's limit: a piece no solid can place a point in. At
//! `y = 0` both policies refuse on a point inside a piece of a face.
//!
//! # A tilted plane
//!
//! `docs/decisions.md` §244.5 argues that a crossing in a plane whose normal
//! has three non-zero parts reaches the overlap arm only with two nodes that
//! are one place. The last three tests run each step of the argument on the
//! plane through the origin with normal `(1, 2, 2)`. That plane cuts a ball of
//! radius 9 at `(6, −6, 3)` in a circle through the origin, which is the start
//! every sweep is taken from.

use rcad_geom::{
    Deflection, EdgeRun, Isometry, MeetOfThree, NamedRun, Plane, Point, Scale, Sphere, Surface,
    Vector, meet_of_three_surfaces, where_a_closed_crossing_starts,
    which_comes_first_round_a_crossing,
};
use rcad_predicates::{
    Decision, Interval, Level, Measures, Point as PredicatePoint, Policy, ProvenanceLog, Record,
};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule, SolidKey};

/// The name `order_along` records a refusal under.
const ALONG_A_CROSSING: &str = "order along a crossing";

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

const WORKING_SIZE: f64 = 25.0;

/// How wide the sweep round a ball's circle in `z = 1` straddles, from its
/// walked start, for the two places at `y = 16`.
const ROUND_A_BALLS_CIRCLE_IN_Z_1: f64 = 1.5881867761018131e-22;

/// The geometry the fuse is handed: both walls' circles start on the axis,
/// the two places are apart, and round either circle they sweep exactly zero.
#[test]
fn round_either_walls_circle_the_two_places_sweep_exactly_zero() {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let prism = a_prism_with_two_walls_through_the_z_axis(&mut model);
    let ball = a_ball_grazing_the_z_axis(&mut model);
    let walls = the_walls_through_the_z_axis(&model, prism);
    let skin = the_only_surface_of(&model, ball);

    let places = where_the_edge_cuts_the_ball(walls, skin);
    assert_ne!(places[0], places[1], "{places:?}");
    for place in places {
        let [x, y, z] = place.enclosure();
        assert_eq!([x.lo(), x.hi(), y.lo(), y.hi()], [0.0; 4], "{place:?}");
        assert!(z.lo() < 16.0 && 16.0 < z.hi(), "{place:?}");
        assert!(
            z.lo() <= 16.0 - f64::powi(2.0, -49) && 16.0 + f64::powi(2.0, -48) <= z.hi(),
            "each box holds both places, 16 ± 2⁻⁵¹, past the f64s either side of 16: {place:?}"
        );
    }

    for wall in walls {
        let Surface::Plane(flat) = wall else {
            panic!("a wall is a plane: {wall:?}");
        };
        let [nx, ny, nz] = flat.normal().components();
        assert!(nx != 0.0 && ny != 0.0 && nz == 0.0, "{flat:?}");
        assert_eq!(flat.offset(), 0.0, "{flat:?}");

        let run = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed crossing is a whole run");
        let from = where_a_closed_crossing_starts(&run, [&wall, &skin], &mut ProvenanceLog::new())
            .expect("a wall cuts the ball in a circle");
        assert_eq!(from.coordinates(), [0.0, 0.0, 16.0], "{wall:?}");

        let swept = which_comes_first_round_a_crossing(
            [&wall, &skin],
            PredicatePoint::from(from.coordinates()),
            places[0],
            places[1],
        )
        .expect("a plane and a ball cross in a circle");
        assert_eq!(swept, Decision::Zero, "{wall:?}");
    }
}

/// The regression: every certified fuse refuses on the order of the two
/// places, by name, with nothing else in its log.
///
/// With the ball first, the ball's face is split first, and the order asked is
/// round a wall's circle: the arm §244.5 changed, as wide as the wider box.
/// Before §244.5 the two were folded into one node there, and every fuse went
/// on to refuse on `orient3d`, the turn between the two walls' circles at that
/// node. With the prism first, a wall's face is split first, and the order
/// asked is along the edge the two walls share, a length, which the policy
/// was always asked about.
#[test]
fn a_certified_fuse_refuses_on_the_order_of_the_two_places() {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let prism = a_prism_with_two_walls_through_the_z_axis(&mut model);
    let ball = a_ball_grazing_the_z_axis(&mut model);
    let places = where_the_edge_cuts_the_ball(
        the_walls_through_the_z_axis(&model, prism),
        the_only_surface_of(&model, ball),
    );
    let widest = places[0].uncertainty().max(places[1].uncertainty());
    for rule in EVERY_RULE {
        for ball_first in [true, false] {
            let refused = match fuse(rule, ball_first, &Policy::certified()) {
                (_, Err(Error::Undecidable { records }), _) => records.records().to_vec(),
                (_, other, _) => panic!("{rule:?}, ball first {ball_first}: {other:?}"),
            };
            assert_eq!(
                refused.len(),
                1,
                "{rule:?}, ball first {ball_first}: {refused:#?}"
            );
            let refused = &refused[0];
            assert!(
                is_the_order_of(refused, places),
                "{rule:?}, ball first {ball_first}: {refused:#?}"
            );
            if ball_first {
                assert_eq!(refused.measures(), Measures::Length, "{rule:?}");
                assert_eq!(refused.enclosure(), widest, "{rule:?}");
            } else {
                assert_eq!(refused.measures(), Measures::Length, "{rule:?}");
            }
        }
    }
}

/// A tolerant policy merges the two places, which are `2⁻⁵⁰` apart, and the
/// intersection is the cap of the ball past the first wall, within what the
/// measurement can promise.
///
/// The second wall takes nothing off that cap: the ball reaches past it only
/// in a sliver about `2⁻¹⁰³` wide round the axis.
#[test]
fn a_tolerant_intersection_is_the_cap_past_the_first_wall() {
    let radius = 1.0 + f64::powi(2.0, -51);
    let past = radius - (3.0 - 4.0 * f64::powi(2.0, -25)) / 5.0;
    let cap = core::f64::consts::PI * past * past * (3.0 * radius - past) / 3.0;
    for ball_first in [true, false] {
        let (model, fused, _) = fuse(Rule::Intersection, ball_first, &Policy::tolerant(25.0));
        let fused = fused.unwrap_or_else(|why| panic!("ball first {ball_first}: {why:?}"));
        let [piece] = fused.solids() else {
            panic!("ball first {ball_first}: {:?}", fused.solids());
        };
        let measured = model
            .solid(*piece)
            .expect("the fuse made it")
            .measure(
                Deflection::of(0.001).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a cap measures");
        assert!(
            (measured.volume() - cap).abs() <= measured.volume_error(),
            "ball first {ball_first}: {} ± {} against {cap}",
            measured.volume(),
            measured.volume_error()
        );
    }
}

/// At `y = 16` the two places `z = 1` cuts the two balls' circle at are boxes
/// that touch, and round that circle they sweep exactly zero: the overlap arm's
/// input. Round either ball's own circle in `z = 1` they do not.
#[test]
fn two_balls_at_y_16_cut_z_1_in_two_boxes_that_sweep_zero_round_their_circle() {
    let [one, other] = two_balls_side_by_side_at(16.0);
    let places = where_z_1_cuts_two_balls(one, other);
    assert_eq!(places, the_two_places_at_y_16());

    for pair in [[&one, &other], [&other, &one]] {
        let from = walked_start(pair);
        assert_eq!(
            from,
            PredicatePoint::from([f64::powi(2.0, -25), 16.0, 1.0]),
            "{pair:?}"
        );
        let swept = which_comes_first_round_a_crossing(pair, from, places[0], places[1])
            .expect("two balls cross in a circle");
        assert_eq!(swept, Decision::Zero, "{pair:?}");
    }

    for up in [1.0, -1.0] {
        let flat = the_plane_z_1(up);
        for pair in [
            [&flat, &one],
            [&one, &flat],
            [&flat, &other],
            [&other, &flat],
        ] {
            let swept =
                which_comes_first_round_a_crossing(pair, walked_start(pair), places[0], places[1])
                    .expect("a plane and a ball cross in a circle");
            assert_eq!(
                swept,
                Decision::Undecidable {
                    enclosure: ROUND_A_BALLS_CIRCLE_IN_Z_1
                },
                "{pair:?}"
            );
        }
    }
}

/// At `y = 0` the two places are the exact `(2⁻²⁵, ±2⁻⁵¹, 1)`, apart, so they
/// never reach the overlap arm, though they sweep zero round the two balls'
/// circle.
#[test]
fn two_balls_at_y_0_cut_z_1_in_two_exact_places_that_are_apart() {
    let [one, other] = two_balls_side_by_side_at(0.0);
    let places = where_z_1_cuts_two_balls(one, other);
    assert_eq!(places, the_two_exact_places_at_y_0());
    for pair in [[&one, &other], [&other, &one]] {
        let from = walked_start(pair);
        assert_eq!(
            from,
            PredicatePoint::from([f64::powi(2.0, -25), 0.0, 1.0]),
            "{pair:?}"
        );
        let swept = which_comes_first_round_a_crossing(pair, from, places[0], places[1])
            .expect("two balls cross in a circle");
        assert_eq!(swept, Decision::Zero, "{pair:?}");
    }
}

/// A certified fuse at `y = 16` refuses before the overlap arm: the order of
/// the two places is asked round a ball's circle in `z = 1` first, and that
/// sweep straddles.
#[test]
fn two_balls_at_y_16_refuse_under_a_certified_policy_round_a_circle_in_z_1() {
    let places = the_two_places_at_y_16();
    for rule in EVERY_RULE {
        for ball_first in [true, false] {
            let refused = refusal_of_two_balls(16.0, rule, ball_first, Policy::certified());
            assert_eq!(
                refused.len(),
                1,
                "{rule:?}, ball first {ball_first}: {refused:#?}"
            );
            let refused = &refused[0];
            assert!(
                is_the_order_of(refused, places),
                "{rule:?}, ball first {ball_first}: {refused:#?}"
            );
            assert_eq!(refused.measures(), Measures::Area, "{rule:?}");
            assert_eq!(refused.enclosure(), ROUND_A_BALLS_CIRCLE_IN_Z_1, "{rule:?}");
        }
    }
}

/// A tolerant fuse at `y = 16` accepts every order it could not prove,
/// reaches the overlap arm, and refuses on §346's limit: a sliver between the
/// two places that holds no `f64` point.
#[test]
fn two_balls_at_y_16_under_a_tolerant_policy_refuse_on_a_piece_no_solid_can_place() {
    let policy = Policy::tolerant(WORKING_SIZE);
    for rule in EVERY_RULE {
        for ball_first in [true, false] {
            let refused = refusal_of_two_balls(16.0, rule, ball_first, policy);
            let past_the_budget: Vec<&Record> = refused
                .iter()
                .filter(|record| !policy.wobble_is_acceptable(record.wobble(WORKING_SIZE)))
                .collect();
            let [past] = past_the_budget[..] else {
                panic!("{rule:?}, ball first {ball_first}: {refused:#?}");
            };
            assert_eq!(
                past.predicate(),
                "a point in a piece that every solid could place",
                "{rule:?}, ball first {ball_first}: {past:#?}"
            );
            assert_eq!(past.level(), Level::Interval, "{rule:?}");
            assert_eq!(past.measures(), Measures::Length, "{rule:?}");
            assert_eq!(past.enclosure(), f64::INFINITY, "{rule:?}");
        }
    }
}

/// At `y = 0` both policies refuse on the two exact places: the sliver between
/// them holds no `f64` point (§244.6, §346).
#[test]
fn two_balls_at_y_0_refuse_on_a_point_inside_a_piece_of_a_face() {
    let places = the_two_exact_places_at_y_0();
    for policy in [Policy::certified(), Policy::tolerant(WORKING_SIZE)] {
        for rule in EVERY_RULE {
            for ball_first in [true, false] {
                let refused = refusal_of_two_balls(0.0, rule, ball_first, policy);
                let [refused] = &refused[..] else {
                    panic!("{policy:?}, {rule:?}, ball first {ball_first}: {refused:#?}");
                };
                assert_eq!(
                    refused.predicate(),
                    "a point inside a piece of a face",
                    "{policy:?}, {rule:?}, ball first {ball_first}: {refused:#?}"
                );
                assert_eq!(refused.level(), Level::Interval, "{rule:?}");
                assert_eq!(refused.measures(), Measures::Length, "{rule:?}");
                assert_eq!(refused.enclosure(), f64::INFINITY, "{rule:?}");
                let inputs = refused.inputs();
                assert!(
                    inputs == places || inputs == [places[1], places[0]],
                    "{policy:?}, {rule:?}, ball first {ball_first}: {refused:#?}"
                );
            }
        }
    }
}

/// §244.5 case A: where one box is wide in two axes or three, the other is a
/// point on the line through the start along the normal. Away from the start
/// the true sweep is zero, but each wide coordinate enters the sum twice, once
/// with each sign, and the interval sum straddles. At the start it is zero,
/// which is why `one == from` answers before the sum is asked.
#[test]
fn a_point_along_a_tilted_normal_against_a_wide_box_does_not_sweep_zero() {
    let (flat, ball) = a_tilted_plane_through_a_balls_centre();
    let normal = the_normal_of(flat);
    let from = PredicatePoint::from([0.0; 3]);
    let wide_in_two =
        PredicatePoint::Constructed([between(-1.0, 1.0), between(-1.0, 1.0), between(0.5, 0.5)]);
    let wide_in_three = PredicatePoint::Constructed([between(-1.0, 1.0); 3]);
    for scale in [1.0, -1.0, 2.0, 0.5, f64::powi(2.0, -60)] {
        let along = PredicatePoint::from(normal.map(|part| scale * part));
        for wide in [wide_in_two, wide_in_three] {
            for [one, other] in [[along, wide], [wide, along]] {
                let swept = which_comes_first_round_a_crossing([&flat, &ball], from, one, other)
                    .expect("a plane and a ball cross in a circle");
                assert!(
                    matches!(swept, Decision::Undecidable { .. }),
                    "{scale} n, {one:?} against {other:?}: {swept:?}"
                );
            }
        }
    }
    let at_the_start = which_comes_first_round_a_crossing([&flat, &ball], from, from, wide_in_two)
        .expect("a plane and a ball cross in a circle");
    assert_eq!(at_the_start, Decision::Zero);
}

/// §244.5 case B: two boxes on the `x` axis through the start sweep exactly
/// zero, and that axis crosses the plane once, at the start, so two places on
/// the circle in both boxes would be one place. Moved off the start, the same
/// two boxes straddle.
#[test]
fn two_boxes_on_an_axis_through_the_start_sweep_zero_and_that_axis_crosses_the_plane_once() {
    let (flat, ball) = a_tilted_plane_through_a_balls_centre();
    let normal = the_normal_of(flat);
    let from = PredicatePoint::from([0.0; 3]);
    let sweep = |one, other| {
        which_comes_first_round_a_crossing([&flat, &ball], from, one, other)
            .expect("a plane and a ball cross in a circle")
    };
    let along_x =
        |lo, hi, y, z| PredicatePoint::Constructed([between(lo, hi), exactly(y), exactly(z)]);

    let (behind, ahead) = (along_x(-0.5, 0.0, 0.0, 0.0), along_x(0.0, 0.25, 0.0, 0.0));
    assert_eq!(sweep(behind, ahead), Decision::Zero);
    assert_eq!(sweep(ahead, behind), Decision::Zero);

    let Surface::Plane(plane) = flat else {
        panic!("a plane: {flat:?}");
    };
    assert_ne!(normal[0], 0.0, "{plane:?}");
    let side = |x: f64| plane.side_of_point(PredicatePoint::from([x, 0.0, 0.0]));
    assert_eq!(side(0.0), Decision::Zero, "{plane:?}");
    assert_eq!(
        [side(-0.5), side(0.25)],
        [Decision::Neg, Decision::Pos],
        "{plane:?}"
    );

    let [y, z] = [normal[1] / 2.0, normal[2] / 2.0];
    let swept = sweep(along_x(-0.5, 0.0, y, z), along_x(0.0, 0.25, y, z));
    assert!(
        matches!(swept, Decision::Undecidable { .. }),
        "off the start: {swept:?}"
    );
}

/// §244.5's second step: where one box is wide along one axis, the other can
/// be wide along no other. A box along `x` and a box along `y`, both ending at
/// the start, straddle.
#[test]
fn boxes_along_two_different_axes_from_the_start_do_not_sweep_zero() {
    let (flat, ball) = a_tilted_plane_through_a_balls_centre();
    let from = PredicatePoint::from([0.0; 3]);
    let along_x = PredicatePoint::Constructed([between(-0.5, 0.0), exactly(0.0), exactly(0.0)]);
    let along_y = PredicatePoint::Constructed([exactly(0.0), between(0.0, 0.25), exactly(0.0)]);
    for [one, other] in [[along_x, along_y], [along_y, along_x]] {
        let swept = which_comes_first_round_a_crossing([&flat, &ball], from, one, other)
            .expect("a plane and a ball cross in a circle");
        assert!(
            matches!(swept, Decision::Undecidable { .. }),
            "{one:?} against {other:?}: {swept:?}"
        );
    }
}

fn fuse(
    rule: Rule,
    ball_first: bool,
    policy: &Policy,
) -> (Model, rcad_topo::Result<Fused>, ProvenanceLog) {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let prism = a_prism_with_two_walls_through_the_z_axis(&mut model);
    let ball = a_ball_grazing_the_z_axis(&mut model);
    let (one, other) = if ball_first {
        (ball, prism)
    } else {
        (prism, ball)
    };
    let mut log = ProvenanceLog::new();
    let fused = model.fuse(&[one], &[other], rule, policy, &mut log);
    (model, fused, log)
}

/// Whether a record is `order_along`'s about the two places, in either order.
fn is_the_order_of(record: &Record, places: [PredicatePoint; 2]) -> bool {
    let inputs = record.inputs();
    record.predicate() == ALONG_A_CROSSING
        && record.level() == Level::Interval
        && (inputs == places || inputs == [places[1], places[0]])
}

fn where_the_edge_cuts_the_ball(walls: [Surface; 2], skin: Surface) -> [PredicatePoint; 2] {
    let mut log = ProvenanceLog::new();
    let meet = meet_of_three_surfaces(
        [&walls[0], &walls[1], &skin],
        &Policy::certified(),
        &mut log,
    )
    .expect("a certified answer");
    assert!(log.is_empty(), "{log:?}");
    let MeetOfThree::At(places) = meet else {
        panic!("the z axis cuts the ball: {meet:?}");
    };
    let [one, other] = places[..] else {
        panic!("a line that nearly touches a ball cuts it twice: {places:?}");
    };
    [one, other]
}

fn a_prism_with_two_walls_through_the_z_axis(model: &mut Model) -> SolidKey {
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 3.0], [-3.0, 3.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a triangle");
    model
        .add_extrusion(&triangle, 32.0, &Isometry::identity())
        .expect("a positive height")
}

fn a_ball_grazing_the_z_axis(model: &mut Model) -> SolidKey {
    model
        .add_ball(
            1.0 + f64::powi(2.0, -51),
            &Isometry::translation(Vector::new(1.0, f64::powi(2.0, -25), 16.0))
                .expect("a finite shift"),
        )
        .expect("a positive radius")
}

/// The two walls whose planes hold the `z` axis, as the fuse is handed them.
fn the_walls_through_the_z_axis(model: &Model, prism: SolidKey) -> [Surface; 2] {
    let on_the_axis = |surface: &Surface| {
        let Surface::Plane(flat) = surface else {
            return false;
        };
        [0.0, 32.0].into_iter().all(|z| {
            flat.side_of_point(PredicatePoint::from(Point::new(0.0, 0.0, z).coordinates()))
                == Decision::Zero
        })
    };
    let walls: Vec<Surface> = model
        .solid(prism)
        .expect("the prism")
        .faces()
        .map(|face| face.surface())
        .filter(on_the_axis)
        .collect();
    let [one, other] = walls[..] else {
        panic!("a triangle with a corner at the origin has two walls there: {walls:?}");
    };
    [one, other]
}

fn the_only_surface_of(model: &Model, solid: SolidKey) -> Surface {
    let mut faces = model.solid(solid).expect("the ball").faces();
    let surface = faces.next().expect("a face").surface();
    assert!(faces.next().is_none(), "a ball is one face");
    surface
}

/// What a fuse of a whole ball with the other ball cut flat at `z = 1` refuses
/// on, panicking on anything but a refusal.
fn refusal_of_two_balls(y: f64, rule: Rule, ball_first: bool, policy: Policy) -> Vec<Record> {
    let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("25 mm is a working size"));
    let whole = a_ball_of_two_at(&mut model, 0.0, y);
    let cut = the_other_ball_cut_flat_at_z_1(&mut model, y);
    let (one, other) = if ball_first {
        (whole, cut)
    } else {
        (cut, whole)
    };
    match model.fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new()) {
        Err(Error::Undecidable { records }) => records.records().to_vec(),
        other => panic!("y = {y}, {policy:?}, {rule:?}, ball first {ball_first}: {other:?}"),
    }
}

/// The ball at `(2⁻²⁴, y, 0)` with a block 2 tall standing on `z = 1` taken
/// out of it, under a certified policy.
fn the_other_ball_cut_flat_at_z_1(model: &mut Model, y: f64) -> SolidKey {
    let ball = a_ball_of_two_at(model, f64::powi(2.0, -24), y);
    let square = ProfileBuilder::new(vec![
        [-2.0, y - 2.0],
        [2.0, y - 2.0],
        [2.0, y + 2.0],
        [-2.0, y + 2.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    let block = model
        .add_extrusion(
            &square,
            2.0,
            &Isometry::translation(Vector::new(0.0, 0.0, 1.0)).expect("a finite shift"),
        )
        .expect("a positive height");
    let cut = model
        .fuse(
            &[ball],
            &[block],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("y = {y}: {why:?}"));
    let [piece] = cut.solids() else {
        panic!("y = {y}: {:?}", cut.solids());
    };
    *piece
}

fn a_ball_of_two_at(model: &mut Model, x: f64, y: f64) -> SolidKey {
    model
        .add_ball(
            1.0 + f64::powi(2.0, -51),
            &Isometry::translation(Vector::new(x, y, 0.0)).expect("a finite shift"),
        )
        .expect("a positive radius")
}

fn two_balls_side_by_side_at(y: f64) -> [Surface; 2] {
    [0.0, f64::powi(2.0, -24)].map(|x| {
        Surface::Sphere(
            Sphere::new(Point::new(x, y, 0.0), 1.0 + f64::powi(2.0, -51))
                .expect("a positive radius"),
        )
    })
}

fn where_z_1_cuts_two_balls(one: Surface, other: Surface) -> [PredicatePoint; 2] {
    let mut log = ProvenanceLog::new();
    let meet = meet_of_three_surfaces(
        [&one, &other, &the_plane_z_1(1.0)],
        &Policy::certified(),
        &mut log,
    )
    .expect("a certified answer");
    assert!(log.is_empty(), "{log:?}");
    let MeetOfThree::At(places) = meet else {
        panic!("z = 1 cuts the two balls' circle: {meet:?}");
    };
    let [one, other] = places[..] else {
        panic!("z = 1 cuts the two balls' circle twice: {places:?}");
    };
    [one, other]
}

fn the_plane_z_1(up: f64) -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 1.0), Vector::new(0.0, 0.0, up))
            .expect("a unit normal"),
    )
}

/// Where a closed crossing of these two is walked from, as the fuse hands it
/// to the sweep.
fn walked_start(pair: [&Surface; 2]) -> PredicatePoint {
    let run = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a closed crossing is a whole run");
    let from = where_a_closed_crossing_starts(&run, pair, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{pair:?}: {why:?}"));
    PredicatePoint::from(from.coordinates())
}

/// `(2⁻²⁵, 16 ± 2⁻⁵¹, 1)`, which are not `f64`s, as boxes that touch at 16.
fn the_two_places_at_y_16() -> [PredicatePoint; 2] {
    let x = exactly(f64::powi(2.0, -25));
    [
        PredicatePoint::Constructed([x, between(15.999999999999998, 16.0), exactly(1.0)]),
        PredicatePoint::Constructed([x, between(16.0, 16.000000000000004), exactly(1.0)]),
    ]
}

fn the_two_exact_places_at_y_0() -> [PredicatePoint; 2] {
    [-1.0, 1.0]
        .map(|side| PredicatePoint::from([f64::powi(2.0, -25), side * f64::powi(2.0, -51), 1.0]))
}

/// The plane through the origin with normal `(1, 2, 2)`, and a ball of radius
/// 9 centred on it at `(6, −6, 3)`, so their circle runs through the origin.
fn a_tilted_plane_through_a_balls_centre() -> (Surface, Surface) {
    let flat = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 2.0, 2.0))
            .expect("a non-zero normal"),
    );
    let ball =
        Surface::Sphere(Sphere::new(Point::new(6.0, -6.0, 3.0), 9.0).expect("a positive radius"));
    (flat, ball)
}

/// A plane's stored normal, checked to have three parts that are not zero.
fn the_normal_of(flat: Surface) -> [f64; 3] {
    let Surface::Plane(plane) = flat else {
        panic!("a plane: {flat:?}");
    };
    let normal = plane.normal().components();
    assert_eq!(
        normal,
        [0.3333333333333333, 0.6666666666666666, 0.6666666666666666],
        "{plane:?}"
    );
    assert_eq!(plane.offset(), 0.0, "{plane:?}");
    normal
}

fn between(lo: f64, hi: f64) -> Interval {
    Interval::new(lo, hi).expect("lo is not above hi")
}

fn exactly(value: f64) -> Interval {
    between(value, value)
}
