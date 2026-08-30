//! A caller who names nothing but `rcad::` can ask the predicates themselves,
//! not only the operations built on top of them.
//!
//! `docs/decisions.md` §10 exit criterion 6 ships [`Sphere`] and [`Cylinder`]
//! "as types with sign predicates only", and criterion 3 wants
//! [`Decision::Undecidable`] reachable through a constructed point. Both of
//! those are predicates, and a predicate is a question a caller asks — so both
//! have to be askable from out here. The three crates underneath are an
//! implementation detail (`docs/decisions.md` §3.6, §7.2), and a caller forced
//! to name one to bring a trait into scope has been told a lie.
//!
//! Nothing in this file imports anything but `rcad`. That is the test.

use rcad::{
    Axis, Cylinder, Decision, ORIENT2D, ORIENT3D, Plane, Point, Policy, PredicatePoint,
    ProvenanceLog, SideOfSurface, Sphere, orient2d, orient2d_tolerantly, orient3d,
    orient3d_of_points, orient3d_tolerantly,
};

/// Says which side, and refuses to let an undecided answer pass as one.
///
/// Every question below is one whose answer follows from exact whole numbers,
/// so anything but a certified sign means the cascade has broken. Writing the
/// four cases out here once is what keeps [`Decision::Undecidable`] from being
/// quietly read as a sign at the twenty call sites that follow.
#[track_caller]
fn certified(decision: Decision) -> Decision {
    match decision {
        Decision::Neg | Decision::Zero | Decision::Pos => decision,
        Decision::Undecidable { enclosure } => {
            panic!(
                "exact coordinates decide exactly, so this should not be a close call: {enclosure}"
            )
        }
        // `Decision` gains variants at every milestone. Guessing what a new one
        // means is worse than stopping.
        other => panic!("unhandled decision: {other:?}"),
    }
}

#[test]
fn a_point_on_the_skin_of_a_sphere_is_neither_in_nor_out() {
    let ball = Sphere::new(Point::new(0.0, 0.0, 0.0), 2.0).expect("a ball of radius two");

    assert_eq!(
        certified(ball.side_of(PredicatePoint::from([2.0, 0.0, 0.0]))),
        Decision::Zero,
        "a point two away from the centre is on the skin"
    );
    assert_eq!(
        certified(ball.side_of(PredicatePoint::from([1.0, 1.0, 1.0]))),
        Decision::Neg,
        "the corner of the unit cube is closer than two"
    );
    assert_eq!(
        certified(ball.side_of(PredicatePoint::from([3.0, 0.0, 0.0]))),
        Decision::Pos,
        "three away from the centre is outside a ball of radius two"
    );
}

#[test]
fn a_point_on_the_skin_of_a_pipe_is_neither_in_nor_out() {
    let pipe = Cylinder::new(Axis::z(), 2.0).expect("a pipe of radius two");

    assert_eq!(
        certified(pipe.side_of(PredicatePoint::from([2.0, 0.0, 7.0]))),
        Decision::Zero,
        "two away from the axis is on the skin, however far along it you are"
    );
    assert_eq!(
        certified(pipe.side_of(PredicatePoint::from([0.0, 0.0, 500.0]))),
        Decision::Neg,
        "a pipe has no ends, so a point on its axis is inside it"
    );
    assert_eq!(
        certified(pipe.side_of(PredicatePoint::from([3.0, 0.0, 0.0]))),
        Decision::Pos,
        "three away from the axis is outside a pipe of radius two"
    );
}

/// The advice in [`Plane::side_of_point`]'s doc, followed from the front door.
///
/// All three points define the plane, so the truthful answer for each is
/// `Zero`. Building the plane scaled its normal to length one and that division
/// rounded, so the stored plane is certain — and wrong — about the very points
/// it was built through. [`orient3d`] never divides, and says what is true.
///
/// The doc tells a caller to reach for `orient3d` when they need the certified
/// answer. Before the facade re-exported it, that was advice a caller who named
/// only `rcad::` could not take.
#[test]
fn the_points_that_built_a_plane_are_not_on_the_plane_it_stored() {
    let corners = [[1.0, 2.0, 3.0], [4.0, 5.0, 7.0], [2.0, 9.0, 4.0]];
    let [first, second, third] = corners.map(|[x, y, z]| Point::new(x, y, z));
    let plane = Plane::through_three_points(first, second, third).expect("three points, one plane");

    for corner in corners {
        assert_eq!(
            certified(plane.side_of_point(PredicatePoint::from(corner))),
            Decision::Neg,
            "the stored plane is certain, and wrong, about {corner:?}"
        );
        assert_eq!(
            certified(orient3d(corners[0], corners[1], corners[2], corner)),
            Decision::Zero,
            "the three defining points say what is true about {corner:?}"
        );
    }
}

/// Exit criterion 3, end to end and from the facade only.
///
/// Three planes cross at (1/3, 1/3, 1/3), which is not an `f64`, so the meeting
/// point is known only as a box. A fourth plane runs exactly through it. Level
/// 1 fails, level 2 straddles zero, and level 3 cannot help because the input
/// itself is inexact — so the honest answer is that nobody knows
/// (`docs/decisions.md` §2.5).
#[test]
fn three_planes_meeting_at_a_point_cannot_be_placed_on_a_plane_through_it() {
    let meet = Plane::meet_of_three(
        &Plane::from_coefficients([1.0, 1.0, 1.0, -1.0]).expect("x + y + z = 1"),
        &Plane::from_coefficients([1.0, -1.0, 0.0, 0.0]).expect("x = y"),
        &Plane::from_coefficients([0.0, 1.0, -1.0, 0.0]).expect("y = z"),
    )
    .expect("three planes at angles to each other cross somewhere");

    assert_eq!(
        meet.exact_coordinates(),
        None,
        "a third is not a number f64 can write down"
    );
    assert!(meet.uncertainty() > 0.0, "so all we have is a box");

    let on_the_same_plane = [
        Point::new(1.0, 0.0, 0.0),
        Point::new(0.0, 1.0, 0.0),
        Point::new(0.0, 0.0, 1.0),
    ]
    .map(PredicatePoint::from);

    match orient3d_of_points(
        on_the_same_plane[0],
        on_the_same_plane[1],
        on_the_same_plane[2],
        meet,
    ) {
        Decision::Neg | Decision::Pos => panic!("the box lies on both sides of the plane at once"),
        Decision::Zero => panic!("nothing here can prove the point is on the plane"),
        Decision::Undecidable { enclosure } => {
            assert!(enclosure > 0.0, "a straddling box has some width");
            assert!(enclosure < 1e-12, "and this one is narrow: {enclosure}");
        }
        other => panic!("unhandled decision: {other:?}"),
    }
}

/// The same question under a tolerant policy, which is the other half of
/// `docs/decisions.md` §2.1: the certified entry point and the tolerant one,
/// separately named, both reachable.
#[test]
fn a_tolerant_policy_reads_that_meeting_point_as_on_the_plane_and_writes_down_what_it_cost() {
    let meet = Plane::meet_of_three(
        &Plane::from_coefficients([1.0, 1.0, 1.0, -1.0]).expect("x + y + z = 1"),
        &Plane::from_coefficients([1.0, -1.0, 0.0, 0.0]).expect("x = y"),
        &Plane::from_coefficients([0.0, 1.0, -1.0, 0.0]).expect("y = z"),
    )
    .expect("three planes at angles to each other cross somewhere");
    let corners = [
        Point::new(1.0, 0.0, 0.0),
        Point::new(0.0, 1.0, 0.0),
        Point::new(0.0, 0.0, 1.0),
    ]
    .map(PredicatePoint::from);

    let mut log = ProvenanceLog::new();
    let decision = orient3d_tolerantly(
        corners[0],
        corners[1],
        corners[2],
        meet,
        &Policy::tolerant(1.0),
        &mut log,
    );

    match decision {
        Decision::Zero => {}
        Decision::Neg | Decision::Pos => panic!("a box straddling the plane has no sign to take"),
        Decision::Undecidable { enclosure } => {
            panic!("a box this narrow is inside a tolerant budget: {enclosure}")
        }
        other => panic!("unhandled decision: {other:?}"),
    }

    assert_eq!(log.len(), 1, "one decision was not certified");
    let record = &log.records()[0];
    assert_eq!(
        record.predicate(),
        ORIENT3D,
        "the caller can name the predicate that gave up without leaving `rcad::`"
    );
    assert!(record.approximation().is_some(), "and see what it cost");
}

#[test]
fn three_points_turning_left_turning_right_and_going_straight_on() {
    assert_eq!(
        certified(orient2d([0.0, 0.0], [1.0, 0.0], [0.0, 1.0])),
        Decision::Pos,
        "up and to the left of the way we came is a left turn"
    );
    assert_eq!(
        certified(orient2d([0.0, 0.0], [1.0, 0.0], [0.0, -1.0])),
        Decision::Neg,
        "and down and to the right is a right turn"
    );
    assert_eq!(
        certified(orient2d([0.0, 0.0], [1.0, 1.0], [2.0, 2.0])),
        Decision::Zero,
        "three points on one line turn neither way"
    );
}

/// A certified answer leaves the log empty, which is itself the signal that
/// nothing was fudged (`docs/decisions.md` §5). The tolerant entry point is
/// worth having at the facade because a caller preparing loops for
/// [`ProfileBuilder`](rcad::ProfileBuilder) needs the same turn test the
/// builder uses; the alternative is a hand-rolled `> 0.0`, which is the answer
/// this kernel exists to stop people writing.
#[test]
fn a_turn_the_cascade_can_prove_costs_a_tolerant_policy_nothing() {
    let mut log = ProvenanceLog::new();
    let decision = orient2d_tolerantly(
        [0.0, 0.0],
        [1.0, 0.0],
        [0.0, 1.0],
        &Policy::tolerant(1.0),
        &mut log,
    );

    assert_eq!(certified(decision), Decision::Pos);
    assert!(log.is_empty(), "a certified answer records nothing at all");
}

/// Three points so close together that the products of their coordinates fall
/// below the smallest `f64`. Exact arithmetic cannot run on numbers that have
/// already been rounded away, so there is no honest sign to report — and the
/// record says which predicate ran out of road.
#[test]
fn a_turn_too_small_for_f64_names_itself_in_the_log() {
    let mut log = ProvenanceLog::new();
    let decision = orient2d_tolerantly(
        [1e-200, 0.0],
        [0.0, 1e-200],
        [0.0, 0.0],
        &Policy::certified(),
        &mut log,
    );

    match decision {
        Decision::Neg | Decision::Pos => panic!("nothing here can prove a sign"),
        Decision::Zero => panic!("these three points are not on one line"),
        Decision::Undecidable { enclosure } => assert!(enclosure >= 0.0),
        other => panic!("unhandled decision: {other:?}"),
    }

    assert_eq!(log.len(), 1);
    assert_eq!(log.records()[0].predicate(), ORIENT2D);
}
