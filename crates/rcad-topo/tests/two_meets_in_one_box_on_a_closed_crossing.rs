//! Two places a block edge cuts a ball, whose boxes are the same bits.
//!
//! A box that names one `f64` triple is one point, so two nodes with that box
//! are one point, proved. A box with width is not: two different points can
//! sit in one box. `round_a_closed_crossing` used to answer `Zero` for any two
//! equal boxes without asking a policy, and so fold two nodes into one that
//! nobody proved is one point (`docs/decisions.md` §248.5, §318). Since §244.5
//! it answers `Zero` only for boxes of no width, and leaves the rest to the
//! policy.
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
//! `round_a_closed_crossing` about the pair, before any arithmetic runs.
//!
//! # What this pins
//!
//! Every fuse of this ball and block refuses by name, under both policies, on
//! the order of the two places, a box `1.3e-5` wide along its longest side and
//! `2.0e-5` corner to corner, which is how far apart two points in it may be.
//! The tolerant policy's budget at a 25 mm working size does not stretch that
//! far, so it refuses too.
//!
//! **What is right:** a refusal, though not this one alone. The order of two
//! different points exists, and a box this wide cannot give it. Before §244.5
//! the two were folded without asking, and every fuse refused one step later on
//! the turn where the top's circle and the wall's circle cross at the box. The
//! two circles cross one way at one of the points and the other way at the
//! other, so the turn is zero somewhere in any box that holds both, and kept as
//! two nodes with one box each, the pair would meet that refusal next (§318.3).
//!
//! **Q699 does not order them.** It orders two places in a line with a start
//! that is off the circle, where each box holds the start at one end. Here the
//! start, `(10000000000.809, 10000000000.139208, 10000000000.25)`, is outside the
//! box, the sweep from it is not a proved zero, and the two boxes are one box,
//! so there is no end of either to read. Telling the two apart takes which root
//! of the edge line's quadratic each meet is, which no box here carries.

use rcad_geom::{Isometry, MeetOfThree, Scale, Surface, Vector, meet_of_three_surfaces};
use rcad_predicates::{Level, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule, SolidKey};

/// The name `order_along` records a refusal under.
const ALONG_A_CROSSING: &str = "order along a crossing";

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

/// The regression: every fuse of the ball and the block refuses on the order of
/// the two places, whose inputs are the one box twice, filed as how far apart
/// two points of that box may be, its diagonal, rounded up. Until Q2144 it was
/// filed as the box's longest side.
///
/// Before §244.5 the two were folded into one node here, and every fuse went
/// on to refuse on `orient3d`, the turn between the top's circle and the
/// wall's at that box.
#[test]
fn every_fuse_refuses_on_the_order_of_the_two_places_in_one_box() {
    for policy in [Policy::certified(), Policy::tolerant(25.0)] {
        for rule in EVERY_RULE {
            let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
            let block = a_block_with_a_leaning_wall(&mut model);
            let ball = a_ball(&mut model, RADIUS);
            let [top, wall] = the_top_and_the_leaning_wall(&model, block);
            let skin = the_only_surface_of(&model, ball);
            let place = the_one_box_the_two_places_share([top, wall], skin);
            let diagonal = the_diagonal_of(place);
            let mut log = ProvenanceLog::new();
            let fused = model.fuse(&[ball], &[block], rule, &policy, &mut log);
            let Err(Error::Undecidable { records }) = fused else {
                panic!(
                    "{rule:?} {policy:?}: {:?}",
                    fused.map(|_: Fused| "an answer")
                );
            };
            let records = records.records().to_vec();
            assert_eq!(records.len(), 1, "{rule:?} {policy:?}: {records:#?}");
            let refused = &records[0];
            assert_eq!(refused.predicate(), ALONG_A_CROSSING, "{rule:?} {policy:?}");
            assert_eq!(refused.level(), Level::Interval, "{rule:?} {policy:?}");
            assert_eq!(refused.inputs(), [place, place], "{rule:?} {policy:?}");
            assert!(
                diagonal <= refused.enclosure()
                    && refused.enclosure() <= diagonal * (1.0 + 8.0 * f64::EPSILON),
                "{rule:?} {policy:?}: {} against {diagonal}",
                refused.enclosure()
            );
        }
    }
}

/// How far apart two points of one box may be: corner to opposite corner.
fn the_diagonal_of(place: PredicatePoint) -> f64 {
    place
        .enclosure()
        .iter()
        .map(|side| (side.hi() - side.lo()) * (side.hi() - side.lo()))
        .sum::<f64>()
        .sqrt()
}

fn the_one_box_the_two_places_share(planes: [Surface; 2], skin: Surface) -> PredicatePoint {
    let meet = meet_of_three_surfaces(
        [&planes[0], &planes[1], &skin],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a certified answer");
    let MeetOfThree::At(places) = meet else {
        panic!("the edge line cuts the ball: {meet:?}");
    };
    let [one, other] = places[..] else {
        panic!("{places:?}");
    };
    assert_eq!(one, other, "the two boxes are the same bits");
    one
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
            &Isometry::translation(Vector::new(0.0, 0.0, FAR + 0.25 - 1.0))
                .expect("a finite shift"),
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
