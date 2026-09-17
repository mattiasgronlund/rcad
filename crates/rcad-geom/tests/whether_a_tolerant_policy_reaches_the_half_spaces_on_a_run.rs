//! Does a tolerant caller's budget reach the half-spaces that say which piece
//! of a crossing a run is?
//!
//! [`walk`] takes a `&Policy` and hands it to `room_left_over`, which is what
//! decides whether the straight pieces between the points are close enough to
//! the curve. It did **not** hand it to the half-spaces on the run: those went
//! through `which_side_of`, which pushed its own record and gave back
//! [`Decision::Undecidable`] without ever looking at the policy. So a caller
//! that had said it would accept a point a hundredth of a millimetre out of
//! place was told a decision worth picometres could not be made, and nothing
//! spent its budget on it.
//!
//! **This file is where that measurement lives, and it is hand-built for a
//! reason.** It used to be measured over the two STEP corpora as well, in
//! `crates/rcad-step/tests/whether_a_tolerant_policy_buys_anything_a_certified_one_does_not.rs`.
//! Every undecided decision either corpus left behind came from one family —
//! two equal-radius cylinders whose axes cross — through the quartic crossing
//! route. `walk` answers that family in closed form now, so neither corpus
//! leaves a single decision undecided under any policy and the measurement
//! there had nothing left to measure. The rings below are not rescued by a
//! closed form and cannot be emptied out from under it the same way.
//!
//! # What the budget buys, and what it does not
//!
//! It buys the caller a **resolved** decision and an account of it:
//! [`Cause::UndecidableResolved`] with a wobble on it, rather than a record that
//! says only "gave up". That is the difference [`a_tolerant_budget_resolves_a_half_space_on_the_boundary`]
//! measures on one decision, and
//! [`every_decision_a_certified_walk_gives_up_on_is_settled_by_a_budget_wide_enough`]
//! over a whole family of them, with a floor that goes red the moment the
//! family stops leaving anything undecided — which is the floor that caught the
//! corpus and is the reason it stands here now.
//!
//! It does **not** move which piece of the crossing is picked, and
//! [`resolving_the_half_space_does_not_change_which_piece_is_picked`] is that
//! measurement rather than a paragraph. `every_half_space_holds` rules a piece
//! out on [`Decision::Neg`] and on nothing else, so an undecided half-space
//! already held the piece and a resolved [`Decision::Zero`] holds it too. A
//! tolerant importer therefore gets the same solids and a better account of
//! them.

use rcad_geom::{
    Axis, Deflection, EdgeRun, Error, HalfSpace, NamedRun, Picker, Plane, Point,
    SIDE_OF_A_HALF_SPACE, Surface, Torus, Vector, walk,
};
use rcad_predicates::{Cause, Decision, Policy, ProvenanceLog, Record};

/// A ring `across` across the middle with a tube one thick, so a plane down its
/// axis cuts two circles of radius one centred `across` either side of the axis.
fn a_ring(across: f64) -> Torus {
    Torus::new(Axis::z(), across, 1.0).expect("a tube narrower than its ring")
}

/// A plane holding the ring's axis, which is what cuts the two meridian
/// circles.
fn a_plane_through_the_axis() -> Plane {
    Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("a normal with a length")
}

/// A run named as one of the two meridian circles by a half-space whose own
/// boundary sits at `x = −boundary_at`.
///
/// Each candidate circle is asked at its own angle zero, which is its centre
/// plus its own radius: `(−(across + 1), 0, 0)` for the circle centred at
/// `(−across, 0, 0)` and `(across + 1, 0, 0)` for its sibling at
/// `(across, 0, 0)`. Every coordinate the tests below hand in is a whole number
/// or a half, which an `f64` holds exactly — that is what lets the picker be put
/// **on** a witness rather than near it.
///
/// `x ≤ −boundary_at` has to hold the far circle's witness and rule the near
/// one out, so that exactly one of the two circles is picked and the walk has an
/// answer to give: the tie the other arrangement produces would refuse under
/// either policy and say nothing about a budget.
///
/// At `boundary_at = across + 1` the boundary passes exactly through the far
/// witness. That witness is not a point but the box
/// `POINT_ALONG_A_CROSSING_ROUNDOFF · 4` wide the walk knows the circle to
/// within, and the box straddles the boundary — so the half-space is honestly
/// undecided and the answer is not a rounding it could have got right. At any
/// smaller `boundary_at` the box clears the boundary and the half-space is
/// decided outright.
fn a_run_picked_by_a_half_space_at(across: f64, boundary_at: f64) -> NamedRun {
    let picker = Picker::InFrontOf(
        HalfSpace::new(Vector::new(-1.0, 0.0, 0.0), boundary_at).expect("a normal with a length"),
    );
    NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
        and: Surface::Torus(a_ring(across)),
        picked_by: [picker].into_iter().collect(),
    })
    .expect("a piece of a crossing with a picker on it")
}

/// Walk that run under `policy`, and hand back what came of it and every record
/// it left.
fn walked_under(
    policy: &Policy,
    across: f64,
    boundary_at: f64,
) -> (Result<Vec<Point>, Error>, Vec<Record>) {
    let flat = a_plane_through_the_axis();
    let mut log = ProvenanceLog::new();
    let walked = walk(
        &a_run_picked_by_a_half_space_at(across, boundary_at),
        [&Surface::Plane(flat), &Surface::Torus(a_ring(across))],
        Deflection::of(0.05).expect("a positive deflection"),
        policy,
        &mut log,
    )
    .map(|along| along.points().to_vec());
    (walked, log.records().to_vec())
}

/// The ring four across the middle with its picker sitting exactly on the far
/// witness, which is the one arrangement the two tests below are written about.
const A_RING_AND_A_PICKER_ON_ITS_WITNESS: (f64, f64) = (4.0, 5.0);

/// The one record a walk of that run leaves, whichever policy walked it.
fn the_one_record(records: &[Record]) -> &Record {
    assert_eq!(
        records.len(),
        1,
        "one half-space is asked about one witness it sits on, so one decision is recorded: {records:?}"
    );
    assert_eq!(records[0].predicate(), SIDE_OF_A_HALF_SPACE);
    &records[0]
}

/// A budget wide enough covers the half-space's wobble and the caller is told
/// so; a budget too narrow does not, and the caller is told that instead.
///
/// **Both halves matter.** Without the narrow one this would pass just as well
/// if the budget were never read and every undecided decision were waved
/// through, which is the other way to get a tolerant policy wrong.
#[test]
fn a_tolerant_budget_resolves_a_half_space_on_the_boundary() {
    let (across, boundary_at) = A_RING_AND_A_PICKER_ON_ITS_WITNESS;
    let (_, certified) = walked_under(&Policy::certified(), across, boundary_at);
    let wobble = the_one_record(&certified).wobble(100.0);
    assert!(
        wobble > 0.0,
        "a box that straddles a plane is undecided by a real amount, not by zero"
    );
    assert_eq!(
        the_one_record(&certified).approximation(),
        None,
        "a certified policy resolves nothing, so the record it leaves carries no wobble it \
         accepted"
    );
    assert!(
        matches!(
            the_one_record(&certified).outcome(),
            Decision::Undecidable { .. }
        ),
        "a certified policy leaves the half-space undecided: {:?}",
        the_one_record(&certified).outcome()
    );

    let wide = Policy::tolerant(100.0);
    assert!(
        wide.wobble_is_acceptable(wobble),
        "this test needs a budget that covers {wobble:e}, and {:e} does not",
        wide.max_wobble()
    );
    let (_, resolved) = walked_under(&wide, across, boundary_at);
    let approximation = the_one_record(&resolved)
        .approximation()
        .expect("a tolerant policy that covers the wobble resolves the decision and says so");
    assert_eq!(approximation.cause(), Cause::UndecidableResolved);
    assert_eq!(approximation.magnitude(), wobble);
    assert_eq!(
        the_one_record(&resolved).outcome(),
        Decision::Zero,
        "a band around zero this narrow is read as zero, which is the whole of what the budget \
         bought"
    );

    let narrow = Policy::tolerant(1.0);
    assert!(
        !narrow.wobble_is_acceptable(wobble),
        "this test needs a budget that does not cover {wobble:e}, and {:e} does",
        narrow.max_wobble()
    );
    let (_, refused) = walked_under(&narrow, across, boundary_at);
    assert_eq!(
        the_one_record(&refused).approximation(),
        None,
        "a budget of {:e} does not cover a wobble of {wobble:e}, so nothing may be resolved \
         under it",
        narrow.max_wobble()
    );
    assert!(
        matches!(
            the_one_record(&refused).outcome(),
            Decision::Undecidable { .. }
        ),
        "a budget that does not cover the wobble leaves the half-space undecided: {:?}",
        the_one_record(&refused).outcome()
    );
}

/// Spending the budget on the half-space changes what the caller is **told**
/// and not which piece of the crossing it gets.
///
/// `every_half_space_holds` rules a piece out on [`Decision::Neg`] and on
/// nothing else, so a half-space that could not tell already held the piece and
/// one resolved to [`Decision::Zero`] holds it too. This one arrangement is
/// where that is read off the points themselves;
/// [`every_decision_a_certified_walk_gives_up_on_is_settled_by_a_budget_wide_enough`]
/// asks it of the whole family at once and compares what the walk handed back.
#[test]
fn resolving_the_half_space_does_not_change_which_piece_is_picked() {
    let (across, boundary_at) = A_RING_AND_A_PICKER_ON_ITS_WITNESS;
    let (certified, _) = walked_under(&Policy::certified(), across, boundary_at);
    let (tolerant, _) = walked_under(&Policy::tolerant(100.0), across, boundary_at);
    let certified = certified.expect("one of the two circles is picked and the other is not");
    assert_eq!(
        tolerant.as_deref(),
        Ok(certified.as_slice()),
        "the same circle, walked into the same points"
    );
    assert!(
        certified.len() > 2,
        "a circle of radius one walked to a deflection of 0.05 takes more than two steps, or \
         this is comparing almost nothing"
    );
}

/// **Every decision a certified walk gives up on is settled by a budget wide
/// enough, and settling them refuses nothing.**
///
/// This is the measurement the two STEP corpora used to carry. It moved here
/// when a closed form for two equal-radius cylinders whose axes cross emptied
/// the corpus's undecided set, and the floor moved with it: `left_undecided > 0`
/// is what stops this passing over nothing the day the same thing happens to
/// these rings. The count lives in the assertions and nowhere in the prose,
/// which is the whole of why it can be trusted a month from now.
///
/// **The family is chosen by how each ring and picker is built and never by
/// what the walk said about it.** Half the runs put the picker's boundary
/// exactly on a witness and half put it half a unit clear, and both halves are
/// walked and counted. That second half is what `runs_that_decided_everything`
/// buys: if the walk ever started leaving a record on a decision it could make,
/// counting records would stop being counting decisions given up on, and this
/// goes red instead of quietly counting the wrong thing.
#[test]
fn every_decision_a_certified_walk_gives_up_on_is_settled_by_a_budget_wide_enough() {
    let wide = Policy::tolerant(100.0);
    let narrow = Policy::tolerant(1.0);
    let runs = runs_to_walk();

    let mut left_undecided = 0;
    let mut runs_that_decided_everything = 0;
    let mut runs_that_walked_to_points = 0;
    let mut settled_on_a_wide_budget = 0;
    let mut settled_on_a_narrow_budget = 0;
    let mut still_given_up_on_a_wide_budget = 0;
    let mut worst_wobble = 0.0_f64;
    let mut walks_that_moved = Vec::new();

    for &(across, boundary_at) in &runs {
        let (certified, gave_up_on) = walked_under(&Policy::certified(), across, boundary_at);
        left_undecided += how_many_were_given_up_on(&gave_up_on);
        if gave_up_on.is_empty() {
            runs_that_decided_everything += 1;
        }
        for record in &gave_up_on {
            worst_wobble = worst_wobble.max(record.wobble(100.0));
        }
        if certified.as_ref().is_ok_and(|points| points.len() > 2) {
            runs_that_walked_to_points += 1;
        }

        let (widely, on_a_wide_budget) = walked_under(&wide, across, boundary_at);
        settled_on_a_wide_budget += how_many_a_budget_settled(&on_a_wide_budget);
        still_given_up_on_a_wide_budget += how_many_were_given_up_on(&on_a_wide_budget);
        settled_on_a_narrow_budget +=
            how_many_a_budget_settled(&walked_under(&narrow, across, boundary_at).1);

        if certified != widely {
            walks_that_moved.push(format!(
                "a ring {across} across picked at x <= -{boundary_at}: certified walked to \
                 {certified:?} and a tolerant one to {widely:?}"
            ));
        }
    }

    assert_eq!(
        runs_that_walked_to_points,
        runs.len(),
        "every one of these pickers holds one circle and rules the other out, so every walk has \
         a circle to hand back, and {runs_that_walked_to_points} of {} did",
        runs.len()
    );
    assert!(
        left_undecided > 0,
        "not one of these {} walks left a decision undecided, so this says nothing about what a \
         budget does with one — which is exactly how the corpus this measurement came from \
         stopped being a measurement",
        runs.len()
    );
    assert!(
        runs_that_decided_everything > 0,
        "every one of these walks left a record behind, so counting records is counting every \
         decision rather than the ones that were given up on"
    );

    assert!(
        wide.wobble_is_acceptable(worst_wobble),
        "this needs a budget that covers the worst wobble of {worst_wobble:e}, and {:e} does not",
        wide.max_wobble()
    );
    assert_eq!(
        settled_on_a_wide_budget,
        left_undecided,
        "a budget of {:e} covers every one of those {left_undecided} decisions, so every one has \
         to come back settled, and {settled_on_a_wide_budget} did",
        wide.max_wobble()
    );
    assert_eq!(
        still_given_up_on_a_wide_budget, 0,
        "a budget that covers every wobble here leaves nothing given up on, and \
         {still_given_up_on_a_wide_budget} were"
    );

    assert!(
        !narrow.wobble_is_acceptable(worst_wobble),
        "this needs a budget that does not cover {worst_wobble:e}, and {:e} does",
        narrow.max_wobble()
    );
    assert_eq!(
        settled_on_a_narrow_budget,
        0,
        "a budget of {:e} covers no wobble here, so it may settle nothing, and it settled \
         {settled_on_a_narrow_budget}",
        narrow.max_wobble()
    );

    assert!(
        walks_that_moved.is_empty(),
        "settling a half-space moved which piece of a crossing the walk handed back, on {} of \
         these runs: {walks_that_moved:#?}",
        walks_that_moved.len()
    );
    println!(
        "{left_undecided} decisions given up on over {} runs, worst wobble {worst_wobble:e}, all \
         settled on a {:e} budget and none on a {:e} one",
        runs.len(),
        wide.max_wobble(),
        narrow.max_wobble()
    );
}

/// The rings walked above, as `(across, boundary_at)`: six with the picker's
/// boundary exactly on the far witness, and the same six with it half a unit
/// clear of both.
///
/// Six sizes rather than one, because one fixture is a claim with a fuse in it.
/// A closed form that rescued a single ring would leave the other five, and the
/// count above moves rather than the file quietly staying true.
fn runs_to_walk() -> Vec<(f64, f64)> {
    let mut runs = Vec::new();
    for across in [2.0, 3.0, 4.0, 5.0, 6.0, 7.0] {
        runs.push((across, across + 1.0));
        runs.push((across, across + 0.5));
    }
    runs
}

/// How many of those records are of a decision nothing could settle.
fn how_many_were_given_up_on(records: &[Record]) -> usize {
    records
        .iter()
        .filter(|record| matches!(record.outcome(), Decision::Undecidable { .. }))
        .count()
}

/// How many of those records are of a decision a budget was spent on.
///
/// Keyed on the cause the record itself carries, so a record filed under a
/// cause this file has never heard of counts as unsettled rather than being
/// quietly counted as settled.
fn how_many_a_budget_settled(records: &[Record]) -> usize {
    records
        .iter()
        .filter(|record| {
            record
                .approximation()
                .is_some_and(|approximation| approximation.cause() == Cause::UndecidableResolved)
        })
        .count()
}
