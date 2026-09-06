//! Doubling the room a walk works in takes it from its step cap to 741 455
//! steps.
//!
//! `rcad-topo`'s `ROOM_ABOVE_THE_FLOOR` is two, and its doc comment says what
//! the two is for: at exactly the floor the walk is asked for a step
//! `2·acos(1 − room/reach)` wide, three millionths of a radian, where the
//! subtraction has thrown away most of its digits and a step count that lands
//! one over [`MOST_POINTS_ALONG_ONE_CROSSING`] is a refusal. **Twice the room is
//! `√2` times the angle and 741 455 steps rather than 1 048 576.** That is a
//! published number with no test under it, and §15.6 says it needs one.
//!
//! The arithmetic being measured is `walk`'s own `how_many_steps`, so the test
//! lives here rather than beside the constant. Nothing below re-derives it: each
//! row calls [`walk`] on a real circle — the one a plane square across a pipe of
//! radius one cuts — and counts the points that come back.
//!
//! # How a room is asked for, when what `walk` takes is a deflection
//!
//! `walk` spends the deflection twice: the points it places carry their own
//! deviation, and what is left over is the room the steps get. The deviation is
//! public — [`PointsAlongACrossing::deviation`] — so a room is asked for by
//! handing over that room plus that deviation, and one coarse walk is what reads
//! the deviation off.
//!
//! # Why the room is written out here and not imported
//!
//! [`THE_ROOM_THE_FLOOR_IS_BUILT_TO_LEAVE`] is `rcad-topo`'s
//! `how_far_a_step_at_the_cap_stands_off_its_chord`, which is private to that
//! crate. It is spelled out rather than reached for, so the thing under test does
//! not get to say what the test expects — and `MOST_POINTS_ALONG_ONE_CROSSING`,
//! which it is derived from, is asserted against so that raising the cap cannot
//! leave this file quietly measuring the old one.

use rcad_geom::{
    Axis, Cylinder, Deflection, Error, MOST_POINTS_ALONG_ONE_CROSSING, Plane, Point, Surface,
    SurfaceMeet, Vector, intersect, walk,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// How far the arc of one step stands off its own chord when a whole turn is
/// walked in as many steps as [`walk`] allows, per unit of how far the crossing
/// reaches.
///
/// `2·sin²(turn / (4 · cap))`, which is what `rcad-topo`'s mesh floor is built
/// out of and what `whether_the_floor_predicts_the_step_cap.rs` publishes as
/// `4.488176455689416e-12`.
const THE_ROOM_THE_FLOOR_IS_BUILT_TO_LEAVE: f64 = 4.488176455689416e-12;

/// The cap the room above is worked out from. A cap that moved without this
/// moving would leave every row below measuring nothing.
const THE_STEP_CAP_THAT_ROOM_COMES_FROM: usize = 1 << 20;

/// How many steps a whole turn is walked in when the walk is left exactly the
/// room the floor is built to leave.
///
/// One under the cap. That is the whole of what "no margin" means here: a
/// rounding in the other direction is a refusal.
const STEPS_AT_THE_ROOM_THE_FLOOR_LEAVES: usize = 1_048_575;

/// How many steps the same turn is walked in when that room is doubled, which is
/// what `ROOM_ABOVE_THE_FLOOR` buys.
const STEPS_AT_TWICE_THAT_ROOM: usize = 741_455;

/// The finest room this circle can be walked in at all, hunted for by halving
/// `f64`s between a room the walk takes and one it refuses.
const THE_FINEST_ROOM_THIS_CIRCLE_CAN_BE_WALKED_IN: f64 = 4.4881320881984266e-12;

/// Coarse enough that the walk is nowhere near its cap, so the deviation it
/// reports is read off a run that plainly worked.
const COARSE_ENOUGH_TO_READ_A_DEVIATION_OFF: f64 = 1e-3;

/// The floor leaves the walk one step of margin, and one step is all of it.
///
/// The room the floor is built to leave is a hundred-thousandth above the finest
/// room this circle can be walked in at all. Ask for a room a single `f64` below
/// that finest one and the walk refuses by name.
#[test]
fn the_room_the_mesh_floor_leaves_is_one_step_clear_of_the_walks_cap() {
    assert_eq!(
        MOST_POINTS_ALONG_ONE_CROSSING, THE_STEP_CAP_THAT_ROOM_COMES_FROM,
        "the cap moved, so the room this file works from is no longer the room the floor \
         is built to leave"
    );
    assert_eq!(
        THE_ROOM_THE_FLOOR_IS_BUILT_TO_LEAVE,
        2.0 * (core::f64::consts::TAU / MOST_POINTS_ALONG_ONE_CROSSING as f64 / 4.0)
            .sin()
            .powi(2)
    );

    assert_eq!(
        how_many_steps_a_whole_turn_takes_in(THE_ROOM_THE_FLOOR_IS_BUILT_TO_LEAVE),
        Ok(STEPS_AT_THE_ROOM_THE_FLOOR_LEAVES)
    );
    assert_eq!(
        STEPS_AT_THE_ROOM_THE_FLOOR_LEAVES + 1,
        MOST_POINTS_ALONG_ONE_CROSSING,
        "the floor is built to land one step under the cap, and it landed {} under",
        MOST_POINTS_ALONG_ONE_CROSSING - STEPS_AT_THE_ROOM_THE_FLOOR_LEAVES
    );

    let finest = the_finest_room_this_circle_can_be_walked_in();
    assert_eq!(finest, THE_FINEST_ROOM_THIS_CIRCLE_CAN_BE_WALKED_IN);
    assert!(
        THE_ROOM_THE_FLOOR_IS_BUILT_TO_LEAVE > finest
            && THE_ROOM_THE_FLOOR_IS_BUILT_TO_LEAVE < finest * 1.0001,
        "the floor leaves {THE_ROOM_THE_FLOOR_IS_BUILT_TO_LEAVE:e} where the walk needs \
         {finest:e}, which is not the hair's breadth the doubling exists to widen"
    );
    assert_eq!(
        how_many_steps_a_whole_turn_takes_in(f64::from_bits(finest.to_bits() - 1)),
        Err(Error::TooManyPointsAlongOneCrossing {
            most: MOST_POINTS_ALONG_ONE_CROSSING
        })
    );
}

/// Twice the room, and the count the doubling was chosen for.
#[test]
fn twice_that_room_walks_a_whole_turn_in_741455_steps() {
    assert_eq!(
        how_many_steps_a_whole_turn_takes_in(2.0 * THE_ROOM_THE_FLOOR_IS_BUILT_TO_LEAVE),
        Ok(STEPS_AT_TWICE_THAT_ROOM)
    );

    let widened =
        the_widest_step_a_whole_turn_is_walked_in(2.0 * THE_ROOM_THE_FLOOR_IS_BUILT_TO_LEAVE)
            / the_widest_step_a_whole_turn_is_walked_in(THE_ROOM_THE_FLOOR_IS_BUILT_TO_LEAVE);
    assert!(
        (widened - core::f64::consts::SQRT_2).abs() < 1e-5,
        "doubling the room widened the step by {widened} where the doc says √2"
    );
}

/// Walk the circle with `room` left over after the points' own deviation, and
/// say how many steps a whole turn was cut into.
///
/// The walk leaves both ends of a closed run out, so the count is one more than
/// the points that come back.
fn how_many_steps_a_whole_turn_takes_in(room: f64) -> Result<usize, Error> {
    let (cap, pipe) = a_plane_square_across_a_pipe_of_radius_one();
    let along = walk(
        &the_circle_they_cut(&cap, &pipe),
        [&cap, &pipe],
        Deflection::of(room + the_deviation_a_point_of_this_circle_carries())
            .expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )?;
    Ok(along.points().len() + 1)
}

/// The step the walk asks `acos` for, in radians, read back off the count.
fn the_widest_step_a_whole_turn_is_walked_in(room: f64) -> f64 {
    let steps =
        how_many_steps_a_whole_turn_takes_in(room).expect("this room is one the walk takes");
    core::f64::consts::TAU / steps as f64
}

/// Halve `f64`s until the walk's answer changes from a count to a refusal.
fn the_finest_room_this_circle_can_be_walked_in() -> f64 {
    let (mut refused, mut taken) = (1e-13, 1e-9);
    assert!(how_many_steps_a_whole_turn_takes_in(refused).is_err());
    assert!(how_many_steps_a_whole_turn_takes_in(taken).is_ok());
    while refused.to_bits() + 1 < taken.to_bits() {
        let between = f64::from_bits((refused.to_bits() + taken.to_bits()) / 2);
        if how_many_steps_a_whole_turn_takes_in(between).is_err() {
            refused = between;
        } else {
            taken = between;
        }
    }
    taken
}

/// How far off the circle the points the walk places may sit, read off a walk
/// coarse enough that nothing else is in question.
fn the_deviation_a_point_of_this_circle_carries() -> f64 {
    let (cap, pipe) = a_plane_square_across_a_pipe_of_radius_one();
    walk(
        &the_circle_they_cut(&cap, &pipe),
        [&cap, &pipe],
        Deflection::of(COARSE_ENOUGH_TO_READ_A_DEVIATION_OFF).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a circle of radius one walked to a thousandth")
    .deviation()
    .magnitude()
}

fn the_circle_they_cut(cap: &Surface, pipe: &Surface) -> rcad_geom::NamedRun {
    let met = intersect(cap, pipe, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a plane square across a pipe cuts one circle");
    let SurfaceMeet::Curves(runs) = met else {
        panic!("a plane square across a pipe met it in {met:?}");
    };
    assert_eq!(runs.len(), 1, "a pipe has one circle at any one height");
    runs.into_iter().next().expect("the one circle")
}

fn a_plane_square_across_a_pipe_of_radius_one() -> (Surface, Surface) {
    let pipe = Surface::Cylinder(
        Cylinder::new(
            Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0)).expect("an axis"),
            1.0,
        )
        .expect("a pipe of radius one"),
    );
    let cap = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal"),
    );
    (cap, pipe)
}
