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
//! Measured over both STEP corpora, that was every undecided decision either of
//! them leaves behind: 31 of them, all `side of a half-space`, all reached from
//! `Model::validate` checking which way a flat face's wire winds.
//! `crates/rcad-step/tests/whether_a_tolerant_policy_buys_anything_a_certified_one_does_not.rs`
//! is that measurement; this file is the one decision it is made of, built by
//! hand so it can be read.
//!
//! # What the budget buys, and what it does not
//!
//! It buys the caller a **resolved** decision and an account of it:
//! [`Cause::UndecidableResolved`] with a wobble on it, rather than a record that
//! says only "gave up". That is the difference [`a_tolerant_budget_resolves_a_half_space_on_the_boundary`]
//! measures.
//!
//! It does **not** move which piece of the crossing is picked, and
//! [`resolving_the_half_space_does_not_change_which_piece_is_picked`] is that
//! measurement rather than a paragraph. `every_half_space_holds` rules a piece
//! out on [`Decision::Neg`] and on nothing else, so an undecided half-space
//! already held the piece and a resolved [`Decision::Zero`] holds it too. A
//! tolerant importer therefore gets the same solids and a better account of
//! them — which is exactly what the corpus reports.

use rcad_geom::{
    Axis, Deflection, EdgeRun, Error, HalfSpace, NamedRun, Picker, Plane, Point,
    SIDE_OF_A_HALF_SPACE, Surface, Torus, Vector, walk,
};
use rcad_predicates::{Cause, Decision, Policy, ProvenanceLog, Record};

/// A ring four across the middle with a tube one thick, so a plane down its
/// axis cuts two circles of radius one centred four either side of the axis.
fn a_ring() -> Torus {
    Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring")
}

/// A plane holding the ring's axis, which is what cuts the two meridian
/// circles.
fn a_plane_through_the_axis() -> Plane {
    Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("a normal with a length")
}

/// A run named as one of the two meridian circles by a half-space whose own
/// boundary passes exactly through the witness on it.
///
/// Each candidate circle is asked at its own angle zero, which is its centre
/// plus its `across`: `(−5, 0, 0)` for the circle centred at `(−4, 0, 0)` and
/// `(5, 0, 0)` for its sibling at `(4, 0, 0)`. Every one of those coordinates is
/// a whole number an `f64` holds exactly, which is what lets the picker be put
/// **on** a witness rather than near it.
///
/// `x ≤ −5` holds the far circle's witness on its boundary and rules the near
/// one out at `x = 5`, so exactly one of the two circles is picked and the walk
/// has an answer to give — the tie the other arrangement produces would refuse
/// under either policy and say nothing about a budget.
///
/// The witness is not a point but the box `POINT_ALONG_A_CROSSING_ROUNDOFF · 4`
/// wide the walk knows the circle to within, and that box straddles the
/// picker's boundary. So the half-space is honestly undecided: the answer is
/// not a rounding it could have got right.
fn a_run_picked_by_a_half_space_it_sits_on() -> NamedRun {
    let picker = Picker::InFrontOf(
        HalfSpace::new(Vector::new(-1.0, 0.0, 0.0), 5.0).expect("a normal with a length"),
    );
    NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
        and: Surface::Torus(a_ring()),
        picked_by: [picker].into_iter().collect(),
    })
    .expect("a piece of a crossing with a picker on it")
}

/// Walk that run under `policy`, and hand back what came of it and every record
/// it left.
fn walked_under(policy: &Policy) -> (Result<Vec<Point>, Error>, Vec<Record>) {
    let ring = a_ring();
    let flat = a_plane_through_the_axis();
    let mut log = ProvenanceLog::new();
    let walked = walk(
        &a_run_picked_by_a_half_space_it_sits_on(),
        [&Surface::Plane(flat), &Surface::Torus(ring)],
        Deflection::of(0.05).expect("a positive deflection"),
        policy,
        &mut log,
    )
    .map(|along| along.points().to_vec());
    (walked, log.records().to_vec())
}

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
    let (_, certified) = walked_under(&Policy::certified());
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
    let (_, resolved) = walked_under(&wide);
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
    let (_, refused) = walked_under(&narrow);
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
/// one resolved to [`Decision::Zero`] holds it too. This is why the STEP corpus
/// reports the same solids under both policies while the records differ, and
/// saying so here is cheaper than reading that off 1101 pairs of runs.
#[test]
fn resolving_the_half_space_does_not_change_which_piece_is_picked() {
    let (certified, _) = walked_under(&Policy::certified());
    let (tolerant, _) = walked_under(&Policy::tolerant(100.0));
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
