//! A walk nobody could refine says so in the log, and a walk that met its
//! deflection says nothing.
//!
//! `docs/decisions.md` §51.7. `walk` asks `room_left_over` how much of the
//! deflection is left once the points' own deviation is paid for. When there is
//! none left a **certified** caller is refused; a **tolerant** one is waved
//! through and gets the two ends of the run with nothing between them. That
//! wave-through used to take a `&Policy` and record nothing at all, which is
//! CLAUDE.md's "provenance on non-certified outcomes" unenforced at one site —
//! and the STEP door is tolerant, so it is the route by which a real file
//! reaches a starved polyline with nothing said about it.
//!
//! # What the return value already said, and what it did not
//!
//! §51.7 reads as if a caller could not tell the wave-through from a run that
//! legitimately needed no interior points. It could:
//! `PointsAlongACrossing::deviation` is a positive length on the wave-through
//! and exactly zero on a straight run, and no other path out of `walk` hands
//! back an empty point list — the step count is floored at
//! `FEWEST_STEPS_THAT_SHOW_A_BEND`, which is two.
//! `a_straight_run_and_a_waved_through_one_were_already_told_apart_by_what_comes_back`
//! pins that half so it cannot quietly stop being true.
//!
//! What was missing is the other half, and it is the half that leaves the
//! function: **nothing landed in the `ProvenanceLog`**. A caller that folds a
//! walk's log into a fuse's — which is what `rcad-topo` does — had no record to
//! fold, so the operation's `worst_wobble` did not know a walk had been starved.
//! That is what these tests are about.
//!
//! # The record is unresolved, and that is a handback rather than a choice
//!
//! What happened here is exactly `Cause::TolerantFallback`: "an operation chose
//! an approximate construction over failing, set by the operation, not by this
//! crate". Writing that cause onto the record needs `Record::resolve_to`, which
//! is `pub(crate)` to `rcad-predicates`, and `Record::new` panics on a certified
//! outcome — so from `rcad-geom` the record can only be pushed unresolved. The
//! magnitude and the cause are not lost: they ride back on
//! `PointsAlongACrossing::deviation`. See the report for the constructor that
//! would close it.

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Error, HalfSpace, NamedRun, Plane, Point, Surface, Vector,
};
use rcad_predicates::{
    Decision, Level, Measures, Point as PredicatePoint, Policy, ProvenanceLog, Record,
};

/// The name `room_left_over` files its wave-through under.
///
/// `rcad_geom::walk`'s module has it as a `pub const`, but `rcad-geom`'s
/// `lib.rs` does not re-export it yet, so the string is written out here. Every
/// other provenance name in the crate — `DYNAMIC_RANGE_CHECK`,
/// `WHERE_THREE_SURFACES_MEET` — is reachable from outside, and this one should
/// be too.
const ROOM_LEFT_TO_REFINE_A_WALK: &str = "room left to refine a walk";

/// Big enough that the points along it are worked out from numbers far larger
/// than any deflection worth asking for, which is what makes their deviation
/// reachable.
const RADIUS: f64 = 1e6;

/// A working size sixteen times the crossing's own.
///
/// `Policy::max_wobble` is `64·ε·working_size` and the deviation is
/// `256·ε·RADIUS`, so this clears it fourfold. Four times over would be the
/// exact edge, and an edge is not what this file is measuring.
fn a_tolerant_caller() -> Policy {
    Policy::tolerant(16.0 * RADIUS)
}

/// The sentence this lane earns its fix with.
///
/// A tolerant caller asked for a deflection no point along this crossing can
/// meet, was waved through with an empty walk, and the log says so.
#[test]
fn a_walk_that_could_not_be_refined_leaves_a_record_a_caller_can_act_on() {
    let deviation = how_far_off_a_point_on_this_crossing_sits();
    let policy = a_tolerant_caller();
    let mut log = ProvenanceLog::new();

    let walked = the_half_circle(
        Deflection::of(deviation / 2.0).expect("half a positive length is positive"),
        &policy,
        &mut log,
    )
    .expect("a tolerant caller is waved through rather than refused");

    assert!(
        walked.points().is_empty(),
        "the walk this file is about is the one that came back with nothing, and this one \
         came back with {}",
        walked.points().len()
    );

    let waved_through = records_named(&log, ROOM_LEFT_TO_REFINE_A_WALK);
    assert_eq!(
        waved_through.len(),
        1,
        "one wave-through is one record, and the log holds {log:?}"
    );
    let record = waved_through[0];
    assert_eq!(record.level(), Level::Float);
    assert_eq!(
        record.measures(),
        Measures::Length,
        "the deviation is already a distance, so the record needs no converting to be read as one"
    );
    assert_eq!(
        record.outcome(),
        Decision::Undecidable {
            enclosure: deviation
        },
        "the enclosure is how far off the crossing every point of this walk may sit"
    );
    assert_eq!(
        record.wobble(16.0 * RADIUS),
        deviation,
        "and it reads back as that same distance"
    );
    assert_eq!(
        record.wobble(16.0 * RADIUS),
        walked.deviation().magnitude(),
        "the record and the returned tag are the one number, not two"
    );
    assert!(
        policy.wobble_is_acceptable(record.wobble(16.0 * RADIUS)),
        "a record pushed here is one the policy already accepted, so folding it into an \
         operation's log cannot turn an accepted walk into a refused solid"
    );
}

/// The other half of the rule, and the reason the first half is worth anything.
///
/// A fully certified outcome logs nothing (§5), so a walk that met the
/// deflection it was asked for must leave no wave-through record behind. Without
/// this the fix could have been "push a record every time" and still looked
/// green.
#[test]
fn a_walk_that_met_its_deflection_records_no_wave_through() {
    let mut log = ProvenanceLog::new();

    let walked = the_half_circle(
        Deflection::of(RADIUS / 8.0).expect("a positive length"),
        &a_tolerant_caller(),
        &mut log,
    )
    .expect("a deflection an eighth of the radius leaves room to spare");

    assert!(
        !walked.points().is_empty(),
        "a walk with room to spare puts points down"
    );
    assert!(
        records_named(&log, ROOM_LEFT_TO_REFINE_A_WALK).is_empty(),
        "nothing was waved through, so nothing is filed under it: {log:?}"
    );
}

/// A refusal is not a wave-through either.
///
/// A certified caller at the same deflection is told
/// `CannotPlaceAPointThatCloseToACrossing`, and the record belongs to the walks
/// that went ahead anyway. Filing one for a refusal would put a wobble in the
/// log for points that were never built.
#[test]
fn a_certified_caller_at_the_same_deflection_is_refused_and_files_nothing() {
    let deviation = how_far_off_a_point_on_this_crossing_sits();
    let mut log = ProvenanceLog::new();

    let refused = the_half_circle(
        Deflection::of(deviation / 2.0).expect("half a positive length is positive"),
        &Policy::certified(),
        &mut log,
    );

    assert!(
        matches!(
            refused,
            Err(Error::CannotPlaceAPointThatCloseToACrossing { .. })
        ),
        "got {refused:?}"
    );
    assert!(
        records_named(&log, ROOM_LEFT_TO_REFINE_A_WALK).is_empty(),
        "no points were built, so no wobble was taken on: {log:?}"
    );
}

/// The half of §51.7's sentence that was already true, pinned so it stays true.
///
/// Both of these come back with no points. A straight run needed none and says
/// so with a deviation of exactly zero; the waved-through one could not place
/// any and carries the deviation it could not get under. A caller reading the
/// returned struct alone can already tell them apart, which is why the fix is
/// the log record and not a second flag on the way out.
#[test]
fn a_straight_run_and_a_waved_through_one_were_already_told_apart_by_what_comes_back() {
    let deviation = how_far_off_a_point_on_this_crossing_sits();
    let mut log = ProvenanceLog::new();

    let straight = the_straight_run(&mut log).expect("a straight run is never refused");
    let waved_through = the_half_circle(
        Deflection::of(deviation / 2.0).expect("half a positive length is positive"),
        &a_tolerant_caller(),
        &mut log,
    )
    .expect("a tolerant caller is waved through");

    assert!(straight.points().is_empty());
    assert!(waved_through.points().is_empty());
    assert_eq!(
        straight.deviation().magnitude(),
        0.0,
        "a straight run needed no help and took no slop"
    );
    assert!(
        waved_through.deviation().magnitude() > 0.0,
        "and the walk that could not be refined carries what it could not get under"
    );
    assert_eq!(
        records_named(&log, ROOM_LEFT_TO_REFINE_A_WALK).len(),
        1,
        "only one of the two was waved through: {log:?}"
    );
}

/// How far off this crossing a point `walk` places may sit, read off a walk
/// coarse enough to have room to spare.
///
/// Measured rather than worked out from [`RADIUS`], so that a change to what
/// [`rcad_geom::POINT_ALONG_A_CROSSING_ROUNDOFF`] is multiplied by moves this
/// file's deflections with it instead of quietly emptying the tests.
fn how_far_off_a_point_on_this_crossing_sits() -> f64 {
    let mut log = ProvenanceLog::new();
    let walked = the_half_circle(
        Deflection::of(RADIUS / 8.0).expect("a positive length"),
        &a_tolerant_caller(),
        &mut log,
    )
    .expect("a deflection an eighth of the radius leaves room to spare");
    let deviation = walked.deviation().magnitude();
    assert!(
        deviation > 0.0 && deviation.is_finite(),
        "a point on a circle of radius {RADIUS} sits some way off it, and this said {deviation}"
    );
    deviation
}

/// The half of the circle where a plane cuts a pipe that lies over the `x` axis.
fn the_half_circle(
    deflection: Deflection,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> rcad_geom::Result<rcad_geom::PointsAlongACrossing> {
    let flat = a_plane_at_the_origin();
    let run = NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(flat),
            picked_by: smallvec::smallvec![rcad_geom::Picker::from(
                HalfSpace::through_a_point_facing(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                    .expect("an axis direction has length")
            )],
        },
        [[RADIUS, 0.0, 0.0], [-RADIUS, 0.0, 0.0]].map(PredicatePoint::from),
    )?;
    rcad_geom::walk(
        &run,
        [&Surface::Plane(flat), &the_pipe()],
        deflection,
        policy,
        log,
    )
}

/// The same two ends joined by a straight run, which is the other way out of
/// [`rcad_geom::walk`] with no points.
fn the_straight_run(log: &mut ProvenanceLog) -> rcad_geom::Result<rcad_geom::PointsAlongACrossing> {
    let flat = a_plane_at_the_origin();
    let run = NamedRun::new(
        EdgeRun::Straight,
        [[RADIUS, 0.0, 0.0], [-RADIUS, 0.0, 0.0]].map(PredicatePoint::from),
    )?;
    rcad_geom::walk(
        &run,
        [&Surface::Plane(flat), &the_pipe()],
        Deflection::of(RADIUS / 8.0).expect("a positive length"),
        &a_tolerant_caller(),
        log,
    )
}

fn a_plane_at_the_origin() -> Plane {
    Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("an axis direction has length")
}

fn the_pipe() -> Surface {
    Surface::Cylinder(Cylinder::new(Axis::z(), RADIUS).expect("a positive radius"))
}

fn records_named<'log>(log: &'log ProvenanceLog, predicate: &str) -> Vec<&'log Record> {
    log.records()
        .iter()
        .filter(|record| record.predicate() == predicate)
        .collect()
}
