//! Two places a block edge cuts a ball, whose boxes are the same bits.
//!
//! `round_a_closed_crossing` answers `Zero` for two nodes whose boxes are equal
//! without asking a policy. A box that names one `f64` is one point, so for an
//! exact point that is a proof. A box with width is not: two different points
//! can sit in one box, and then the two are folded into one node that nobody
//! proved is one point (`docs/decisions.md` §248.5, §318).
//!
//! # The shape
//!
//! A block ten thousand million units out, whose top `z = 10¹⁰ + 0.25` and a
//! wall leaning at 45° across `x` and `y` meet in an edge, and a ball whose
//! radius is the smallest `f64` at which it reaches that edge line. One `f64`
//! less and the three surfaces share nothing. At the radius used they share two
//! points, because `meet_of_three_surfaces` proves the discriminant positive.
//! A radius one step past touching leaves a chord far shorter than the step of
//! an `f64` out there, `1.9e-6`, so the two points land in one box, and the two
//! boxes come out the same bits.
//!
//! On the ball's face both points lie on two closed crossings, the circles the
//! top and the wall cut. So `merge_points_that_are_one_point` asks
//! `round_a_closed_crossing` about the pair, and it answers `Zero` before any
//! arithmetic runs.
//!
//! # What this pins, and what it does not
//!
//! **Only what is true today.** Every fuse of this ball and block refuses by
//! name, under both policies, on the turn where the top's circle and the wall's
//! circle cross at that one box. The two circles cross one way at one of the
//! two points and the other way at the other, so the turn is zero somewhere in
//! any box that holds both, and no box arithmetic can settle it. That refusal
//! would come just the same with the two meets kept as two nodes, so it says
//! nothing about whether the fold was right.
//!
//! The fold itself is not visible through the public API, and a probe that
//! logged it is recorded in §318, not pinned here.

use rcad_geom::{Isometry, MeetOfThree, Scale, Surface, Vector, meet_of_three_surfaces};
use rcad_predicates::{Level, Point as PredicatePoint, Policy, ProvenanceLog, Record};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule, SolidKey};

/// How far out the block and the ball sit, on every axis.
const FAR: f64 = 1.0e10;

/// The ball's centre: above the block's top and in front of its leaning wall.
const CENTRE: [f64; 3] = [10_000_000_000.809, 9_999_999_999.85, 10_000_000_000.967_9];

/// The smallest `f64` radius at which the ball reaches the edge line. Found by
/// halving on the bits of the radius, with `meet_of_three_surfaces` as the
/// judge, and checked in the first test below.
const RADIUS: f64 = 0.773_963_870_867_393_5;

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The geometry the fuse is handed: the two places the edge line cuts the ball
/// are two points, and their boxes are the same bits.
#[test]
fn the_two_places_the_edge_cuts_the_ball_are_one_box() {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let block = a_block_with_a_leaning_wall(&mut model);
    let ball = a_ball(&mut model, RADIUS);
    let [top, wall] = the_top_and_the_leaning_wall(&model, block);
    let skin = the_only_surface_of(&model, ball);

    let mut log = ProvenanceLog::new();
    let meet = meet_of_three_surfaces([&top, &wall, &skin], &Policy::certified(), &mut log)
        .expect("a certified answer");
    assert!(log.is_empty(), "{log:?}");
    let MeetOfThree::At(places) = meet else {
        panic!("the edge line cuts the ball: {meet:?}");
    };
    assert_eq!(
        places.len(),
        2,
        "two places is a discriminant proved positive, so two different points: {places:?}"
    );
    assert_eq!(places[0], places[1], "the two boxes are the same bits");
    let PredicatePoint::Constructed(sides) = places[0] else {
        panic!("a box, not an exact point: {places:?}");
    };
    assert!(
        sides.iter().all(|side| side.lo() < side.hi()),
        "every side of the box has width, so it can hold two points: {sides:?}"
    );

    let smaller = a_ball(&mut model, f64::from_bits(RADIUS.to_bits() - 1));
    let skin = the_only_surface_of(&model, smaller);
    let meet = meet_of_three_surfaces([&top, &wall, &skin], &Policy::certified(), &mut log)
        .expect("a certified answer");
    assert_eq!(
        meet,
        MeetOfThree::Nowhere,
        "one f64 less and the ball misses the edge line"
    );
}

/// What every fuse of the ball and the block answers today: a refusal on the
/// turn at the one box, whose first input is the ball's gradient there and
/// whose other two are the normals of the top and the leaning wall.
#[test]
fn every_fuse_refuses_on_the_turn_where_the_two_circles_cross() {
    for policy in [Policy::certified(), Policy::tolerant(25.0)] {
        for rule in EVERY_RULE {
            let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
            let block = a_block_with_a_leaning_wall(&mut model);
            let ball = a_ball(&mut model, RADIUS);
            let [top, wall] = the_top_and_the_leaning_wall(&model, block);
            let mut log = ProvenanceLog::new();
            let records = match model.fuse(&[ball], &[block], rule, &policy, &mut log) {
                Err(Error::Undecidable { records }) => records.records().to_vec(),
                other => panic!("{rule:?} {policy:?}: {:?}", other.map(|_: Fused| "an answer")),
            };
            assert_eq!(records.len(), 1, "{rule:?} {policy:?}: {records:#?}");
            assert!(
                is_the_turn_between_the_top_and_the_wall(&records[0], &top, &wall),
                "{rule:?} {policy:?}: {records:#?}"
            );
        }
    }
}

/// Whether a refusal is `orient3d` on a box and the two planes' normals, which
/// is the turn between the two circles those planes cut on the ball.
fn is_the_turn_between_the_top_and_the_wall(record: &Record, top: &Surface, wall: &Surface) -> bool {
    let normal = |surface: &Surface| match surface {
        Surface::Plane(flat) => PredicatePoint::from(flat.normal().components()),
        other => panic!("a plane: {other:?}"),
    };
    let inputs = record.inputs();
    record.predicate() == "orient3d"
        && record.level() == Level::Interval
        && inputs.len() == 3
        && matches!(inputs[0], PredicatePoint::Constructed(_))
        && inputs[1] == normal(top)
        && inputs[2] == normal(wall)
}

/// A block one unit tall whose top is `z = FAR + 0.25`, with a wall leaning at
/// 45° across `x` and `y` that faces the ball.
fn a_block_with_a_leaning_wall(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![
        [FAR - 1.4, FAR + 1.65],
        [FAR - 4.4, FAR - 1.35],
        [FAR - 0.4, FAR - 5.35],
        [FAR + 2.6, FAR - 2.35],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square turned 45°");
    model
        .add_extrusion(
            &square,
            1.0,
            &Isometry::translation(Vector::new(0.0, 0.0, FAR + 0.25 - 1.0)).expect("a finite shift"),
        )
        .expect("a positive height")
}

fn a_ball(model: &mut Model, radius: f64) -> SolidKey {
    model
        .add_ball(
            radius,
            &Isometry::translation(Vector::new(CENTRE[0], CENTRE[1], CENTRE[2]))
                .expect("a finite shift"),
        )
        .expect("a positive radius")
}

/// The block's top and its leaning wall, as the fuse is handed them.
fn the_top_and_the_leaning_wall(model: &Model, block: SolidKey) -> [Surface; 2] {
    let planes: Vec<Surface> = model
        .solid(block)
        .expect("the block")
        .faces()
        .map(|face| face.surface())
        .collect();
    let facing = |wanted: fn([f64; 3]) -> bool| {
        let found: Vec<&Surface> = planes
            .iter()
            .filter(|surface| matches!(surface, Surface::Plane(flat) if wanted(flat.normal().components())))
            .collect();
        assert_eq!(found.len(), 1, "{planes:?}");
        *found[0]
    };
    [
        facing(|normal| normal[2] > 0.9),
        facing(|normal| normal[0] > 0.5 && normal[1] > 0.5),
    ]
}

fn the_only_surface_of(model: &Model, solid: SolidKey) -> Surface {
    let mut faces = model.solid(solid).expect("the ball").faces();
    let surface = faces.next().expect("a face").surface();
    assert!(faces.next().is_none(), "a ball is one face");
    surface
}
