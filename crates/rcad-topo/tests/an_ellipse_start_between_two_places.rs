//! A round bar cut by a tilted block, whose top crosses the bar's wall in an
//! ellipse that is walked from a start between two places the block's side cuts
//! it at: the arm that puts the one ahead of the start first, reached on an
//! ellipse (`docs/decisions.md` §244.8).
//!
//! # The shape
//!
//! A round bar of radius `1 + 2⁻⁵¹` up `z`, its axis through `(−16, 0, 0)`, and a
//! block on the square `x ∈ [−17, −13]`, `y ∈ [−6, 6]`, 4 tall, moved down 2 and
//! turned `−0.5` about `x`. The turn leaves `x` alone, so the block's side is
//! exactly `x = −17`, and its top leans across the bar and cuts the wall in an
//! ellipse.
//!
//! An ellipse where a plane cuts a pipe is walked from an end of its short axis:
//! the centre, where the axis meets the plane, plus the bar's radius along the
//! way square to both the axis and the plane's normal. Here that way is `−x`, so
//! the start is `−16 − (1 + 2⁻⁵¹)`, which rounds to `−17`: on the block's side,
//! `2⁻⁵¹` inside the bar. The side cuts the ellipse at two places `y ≈ ±2⁻²⁵`,
//! one either side of the start.
//!
//! # Why it reaches the arm
//!
//! The start and both places have `x = −17` exactly and the top's normal has no
//! `x`, so seen along that normal the three are on one line and sweep exactly
//! nothing. Neither place's box holds the start and the two boxes are apart, so
//! `the_one_the_start_is` in `fuse/arrange.rs` answers from its `(false, false)`
//! arm, which asks `the_one_ahead_of_the_start`. Neither place is the one the
//! start is a rounding off, since the step to each has a sign along the way the
//! ellipse runs, so `the_start_is_a_rounding_off` puts neither first before
//! that.
//!
//! # What the arm reads, and what it is checked against
//!
//! The arm puts first the place whose step from the start has a positive dot
//! with `∇S × ∇T` at the start. The row asked for two checks: the sweep from the
//! start to a third point across the ellipse, and `det(n, from − c, P − c)` with
//! `c` the ellipse's centre. On an ellipse the second is not the arm's reading
//! everywhere, but at an end of the short axis it is, and here it is bit for bit.

use rcad_geom::{
    Axis, Error as GeometryError, Isometry, MeetOfThree, Point, Scale, Surface, SurfaceMeet,
    Vector, intersect, meet_of_three_surfaces, where_a_closed_crossing_starts,
    which_comes_first_round_a_crossing,
};
use rcad_predicates::{Decision, Interval, Level, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule, SolidKey};

/// The name `order_along` records a refusal under.
const ALONG_A_CROSSING: &str = "order along a crossing";

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The input the arm needs, with either surface first: the start is exactly
/// `(−17, 0, c_z)`, the end of the short axis on the side `x = −17`, with `c`
/// the centre worked out as `where_a_plane_cuts_a_pipe` works it out. The two
/// places have `x = −17` exactly and `y` boxes either side of the start's `0`,
/// so neither holds the start and the two are apart, and the sweep through all
/// three is exactly zero.
#[test]
fn the_tops_ellipse_starts_on_the_side_between_the_two_places_the_side_cuts_it_at() {
    let [top, side, wall] = the_top_the_side_and_the_wall();
    let centre = where_the_axis_meets_the_top(&top, &wall);
    assert_eq!(centre[..2], [-16.0, 0.0]);
    let [below, above] = the_two_places(&top, &side, &wall);
    for place in [below, above] {
        let x = place.enclosure()[0];
        assert_eq!([x.lo(), x.hi()], [-17.0, -17.0], "{place:?}");
    }
    assert!(below.enclosure()[1].hi() < 0.0, "{below:?}");
    assert!(above.enclosure()[1].lo() > 0.0, "{above:?}");
    for pair in [[&top, &wall], [&wall, &top]] {
        let start = the_walked_start(pair);
        assert_eq!(start, [-17.0, 0.0, centre[2]], "{pair:?}");
        assert_eq!(
            which_comes_first_round_a_crossing(pair, PredicatePoint::Exact(start), below, above),
            Ok(Decision::Zero),
            "{pair:?}"
        );
    }
}

/// What the arm reads, worked out here with its arithmetic, and checked against
/// the sweep to a third point and against the centre.
///
/// The arm's run is `n × ∇wall` at the start, top first, where `∇wall` is
/// `|w|²(x − o) − (w·(x − o))w` as `gradient_at` has it. Every step of that is
/// exact here, so the order the arithmetic is done in cannot move it, and it is
/// the exact `(−1, 0, 0)`. The step to the place below `y = 0` then reads `Pos`
/// and the one above reads `Neg`, so the arm puts the place below first.
///
/// `det(n, from − c, P − c)` is `(n × (from − c)) · (P − c)`, and since the
/// start is an end of the short axis, `∇wall` there is `|w|²(from − c)`: here
/// `n × (from − c)` is the run bit for bit, and the det gives the same signs.
///
/// The third point is `(−15, 0, c_z)`, the other end of the short axis and
/// where the bar's seam crosses the top, give or take a rounding. Top first,
/// the sweep puts the place below before it and it before the place above, so
/// going round the three come below, across, above, and the arm's order is
/// the only one of the two that agrees. With the wall first every sign turns
/// over and the order runs the other way.
#[test]
fn the_one_ahead_of_the_start_comes_before_the_point_across_the_ellipse_and_the_other_after_it() {
    let [top, side, wall] = the_top_the_side_and_the_wall();
    let Surface::Plane(flat) = top else {
        panic!("the top is a plane: {top:?}")
    };
    let normal = flat.normal().components().map(Interval::exactly);
    let centre = where_the_axis_meets_the_top(&top, &wall);
    let [below, above] = the_two_places(&top, &side, &wall);
    let start = the_walked_start([&top, &wall]);

    let gradient = the_walls_gradient_at(&wall, start);
    assert_eq!(gradient, [-1.0, 0.0, 0.0].map(Interval::exactly));
    let runs = cross(normal, gradient);
    let off_the_centre = [0, 1, 2].map(|axis| Interval::exactly(start[axis] - centre[axis]));
    assert_eq!(runs, cross(normal, off_the_centre));

    let step_to = |place: PredicatePoint| {
        [0, 1, 2].map(|axis| place.enclosure()[axis] - Interval::exactly(start[axis]))
    };
    let from_the_centre = |place: PredicatePoint| {
        [0, 1, 2].map(|axis| place.enclosure()[axis] - Interval::exactly(centre[axis]))
    };
    let turned_over = cross(gradient, normal);
    for (place, top_first, wall_first) in [
        (below, Decision::Pos, Decision::Neg),
        (above, Decision::Neg, Decision::Pos),
    ] {
        assert_eq!(dot(step_to(place), runs).sign(), top_first, "{place:?}");
        assert_eq!(
            dot(cross(normal, off_the_centre), from_the_centre(place)).sign(),
            top_first,
            "{place:?}"
        );
        assert_eq!(
            dot(step_to(place), turned_over).sign(),
            wall_first,
            "{place:?}"
        );
    }

    let across = PredicatePoint::Exact([-15.0, 0.0, centre[2]]);
    let from = PredicatePoint::Exact(start);
    for (pair, first) in [
        ([&top, &wall], Decision::Pos),
        ([&wall, &top], Decision::Neg),
    ] {
        assert_eq!(
            which_comes_first_round_a_crossing(pair, from, below, across),
            Ok(first),
            "{pair:?}"
        );
        assert_eq!(
            which_comes_first_round_a_crossing(pair, from, across, above),
            Ok(first),
            "{pair:?}"
        );
    }
}

/// The pin: under either policy, every rule with either solid first gets past
/// the order of the two places with nothing recorded about the order along a
/// crossing, and is refused while its answer is written, with
/// `CannotNameWhichPieceOfACrossingThisIs` out of `rcad-geom`. The records are
/// all `side of a half-space`, at `Interval`: three for the union and for the
/// bar less the block, one for the intersection and for the block less the bar.
///
/// **Why this says the arm gave an order.** Every fuse here goes through
/// `the_one_ahead_of_the_start`, and goes there from `the_one_the_start_is`'s
/// `(false, false)` arm: a run under `gdb` with no change to the source, a
/// breakpoint on each, counted 72 hits of the function and 72 of the arm over
/// the twelve fuses, and none of the overlap arm or of the line that answers
/// `Undecidable`. That is a run and not a test. The rest is this file: the
/// sweep is a proved zero, neither place is a rounding off the start, the
/// boxes are apart and neither holds the start, which is the only way to that
/// arm. Had the arm left the pair `Undecidable`, a certified fuse would have
/// refused with an `order along a crossing` record.
///
/// Why each fuse is then refused has not been traced.
#[test]
fn every_fuse_gets_past_the_order_of_the_two_places_and_is_refused_naming_a_piece_of_a_crossing() {
    for policy in [Policy::certified(), Policy::tolerant(25.0)] {
        for rule in EVERY_RULE {
            for bar_first in [true, false] {
                let case = format!("{policy:?} {rule:?}, bar first {bar_first}");
                let mut log = ProvenanceLog::new();
                match fuse_the_bar_and_the_block(rule, bar_first, &policy, &mut log) {
                    Err(Error::Geometry(GeometryError::CannotNameWhichPieceOfACrossingThisIs)) => {}
                    Ok(fused) => panic!(
                        "{case}: the fuse answered with {} solids; check each against its \
                         volume and pin it",
                        fused.solids().len()
                    ),
                    Err(other) => panic!("{case}: {other:?}"),
                }
                let records = log.records();
                assert!(
                    records
                        .iter()
                        .all(|record| record.predicate() != ALONG_A_CROSSING),
                    "{case}: {records:#?}"
                );
                let how_many = match (rule, bar_first) {
                    (Rule::Union, _) | (Rule::Difference, true) => 3,
                    (Rule::Intersection, _) | (Rule::Difference, false) => 1,
                    _ => unreachable!("{case}: EVERY_RULE names three rules"),
                };
                assert_eq!(
                    records
                        .iter()
                        .map(|record| (record.predicate(), record.level()))
                        .collect::<Vec<_>>(),
                    vec![("side of a half-space", Level::Interval); how_many],
                    "{case}: {records:#?}"
                );
            }
        }
    }
}

fn fuse_the_bar_and_the_block(
    rule: Rule,
    bar_first: bool,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> rcad_topo::Result<Fused> {
    let (mut model, bar, block) = the_bar_and_the_block();
    if bar_first {
        model.fuse(&[bar], &[block], rule, policy, log)
    } else {
        model.fuse(&[block], &[bar], rule, policy, log)
    }
}

/// The block's top, the plane whose normal points mostly up, its side
/// `x = −17`, and the bar's wall, each as the model places it.
fn the_top_the_side_and_the_wall() -> [Surface; 3] {
    let (model, bar, block) = the_bar_and_the_block();
    let surfaces = |key: SolidKey| -> Vec<Surface> {
        model
            .solid(key)
            .expect("this model made the key")
            .faces()
            .map(|face| face.surface())
            .collect()
    };
    let block = surfaces(block);
    let plane_facing = |facing: fn([f64; 3]) -> bool| {
        let mut found = block.iter().filter(
            |surface| matches!(surface, Surface::Plane(flat) if facing(flat.normal().components())),
        );
        let (Some(plane), None) = (found.next(), found.next()) else {
            panic!("one face of the block faces that way: {block:?}")
        };
        *plane
    };
    let top = plane_facing(|normal| normal[2] > 0.5);
    let side = plane_facing(|normal| normal == [-1.0, 0.0, 0.0]);
    let bar = surfaces(bar);
    let wall = *bar
        .iter()
        .find(|surface| matches!(surface, Surface::Cylinder(_)))
        .expect("a round bar has a wall");
    [top, side, wall]
}

/// The centre of the ellipse the top cuts the wall in, worked out the way
/// `where_a_plane_cuts_a_pipe` in `rcad-geom`'s `walk.rs` works it out.
fn where_the_axis_meets_the_top(top: &Surface, wall: &Surface) -> [f64; 3] {
    let (Surface::Plane(flat), Surface::Cylinder(pipe)) = (top, wall) else {
        panic!("a plane and a pipe: {top:?}, {wall:?}")
    };
    let normal = flat.normal();
    let along = pipe.axis().direction();
    let up_the_axis =
        (flat.offset() - normal.dot(pipe.axis().origin() - Point::origin())) / normal.dot(along);
    (pipe.axis().origin() + along * up_the_axis).coordinates()
}

/// The two places the block's side cuts the top's ellipse at, the one below
/// `y = 0` first.
fn the_two_places(top: &Surface, side: &Surface, wall: &Surface) -> [PredicatePoint; 2] {
    let MeetOfThree::At(places) = meet_of_three_surfaces(
        [side, top, wall],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("the three meet") else {
        panic!("the side, the top and the wall meet at places")
    };
    let [one, other] = places[..] else {
        panic!("two places: {places:?}")
    };
    if one.enclosure()[1].hi() < 0.0 {
        [one, other]
    } else {
        [other, one]
    }
}

fn the_walked_start(pair: [&Surface; 2]) -> [f64; 3] {
    let SurfaceMeet::Curves(runs) = intersect(
        pair[0],
        pair[1],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("the top crosses the wall") else {
        panic!("the top crosses the wall in a curve")
    };
    let [ellipse] = &runs[..] else {
        panic!("one ellipse: {runs:?}")
    };
    where_a_closed_crossing_starts(ellipse, pair, &mut ProvenanceLog::new())
        .expect("an ellipse has a start")
        .coordinates()
}

/// `|w|²(x − o) − (w·(x − o))w`, the gradient `fuse/arrange.rs`'s `gradient_at`
/// takes of a pipe, in the same arithmetic.
fn the_walls_gradient_at(wall: &Surface, at: [f64; 3]) -> [Interval; 3] {
    let Surface::Cylinder(pipe) = wall else {
        panic!("a pipe: {wall:?}")
    };
    let origin = pipe.axis().origin().coordinates();
    let along = pipe.axis().direction().components().map(Interval::exactly);
    let step = [0, 1, 2].map(|axis| Interval::exactly(at[axis]) - Interval::exactly(origin[axis]));
    let (squared, shadow) = (dot(along, along), dot(along, step));
    [0, 1, 2].map(|axis| squared * step[axis] - shadow * along[axis])
}

fn cross(one: [Interval; 3], other: [Interval; 3]) -> [Interval; 3] {
    [0, 1, 2].map(|axis| {
        let (next, last) = ((axis + 1) % 3, (axis + 2) % 3);
        one[next] * other[last] - one[last] * other[next]
    })
}

fn dot(one: [Interval; 3], other: [Interval; 3]) -> Interval {
    (0..3)
        .map(|axis| one[axis] * other[axis])
        .fold(Interval::exactly(0.0), |so_far, term| so_far + term)
}

fn the_bar_and_the_block() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let bar = model
        .add_round_bar(
            1.0 + f64::powi(2.0, -51),
            10.0,
            &Isometry::translation(Vector::new(-16.0, 0.0, 0.0)).expect("a finite shift"),
        )
        .expect("a positive radius and height");
    let square = ProfileBuilder::new(vec![
        [-17.0, -6.0],
        [-13.0, -6.0],
        [-13.0, 6.0],
        [-17.0, 6.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    let placement = Isometry::translation(Vector::new(0.0, 0.0, -2.0))
        .expect("a finite shift")
        .compose(&Isometry::rotation(&Axis::x(), -0.5).expect("a finite turn"));
    let block = model
        .add_extrusion(&square, 4.0, &placement)
        .expect("a positive height");
    (model, bar, block)
}
