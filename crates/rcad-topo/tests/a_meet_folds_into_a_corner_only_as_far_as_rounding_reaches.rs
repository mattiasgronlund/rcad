//! A meet of three surfaces is folded into a corner of the face only when its
//! box is no further from the corner than writing the corner down cost (§253).
//!
//! `place_the_meets` used to fold a meet into any corner on both of its
//! crossings whose box it touched, without asking the policy, however wide the
//! meet's box was. Across every test in the workspace, one fold's box reached
//! further than the allowance `bake.rs` grants a stored corner: a draw in the
//! fuse target's corpus, a tolerant union of two blocks that do not meet. Two
//! of the meet's three planes are about `1.5e-6` off parallel, so its box is
//! `±1.43e-10` wide, 2 012 times the allowance, while the point it holds is
//! `4.1e-16` from the corner (§253.7, worked in exact rationals). The old fold
//! gave two solids, and that answer was right. With the meet kept as a node
//! the fuse refuses on "order along a crossing", so this pins a right answer
//! refused, not a wrong one prevented.
//!
//! **When the meet's box is tightened, this pin should move to `ok: 1` with
//! two solids made**, which is the answer the old fold gave. A fold that
//! comes back past the allowance moves it the same way, so check which of
//! the two happened before moving it.
//!
//! The draw is a copy of `fuzz/corpus/two_solids_fused_by_a_stranger/a45aa71b…`
//! beside this file, so that minimising the corpus cannot drop it. It is named
//! rather than rebuilt here because it is four and a half kilobytes of `Model`
//! and reads as nothing, the reason
//! `a_fused_draw_hands_its_provenance_back_through_the_door.rs` gives for the
//! same choice.

#![forbid(unsafe_code)]

mod fuzzing;

use fuzzing::fuse::two_models_split_and_fused_noting_refusals;
use rcad_topo::Error;

const THE_DRAW: &[u8] =
    include_bytes!("a_meet_folds_into_a_corner_only_as_far_as_rounding_reaches.draw");

/// What §253.3 could not say without digging the refusal out by hand: the
/// predicate that gave up (§253.7).
const ALONG_A_CROSSING: &str = "order along a crossing";

/// The predicate an `Undecidable` refusal names, read the way
/// `Error::Undecidable`'s doc comment says to: the last record in its log.
///
/// # Panics
///
/// If `why` is not `Error::Undecidable`, or if its log is empty.
fn which_predicate_the_refusal_names(why: &Error) -> &'static str {
    match why {
        Error::Undecidable { records } => records
            .records()
            .last()
            .expect("a refusal names at least one decision")
            .predicate(),
        other => panic!("expected `Undecidable`, got {other:?}"),
    }
}

#[test]
fn a_meet_further_from_a_corner_than_rounding_reaches_is_not_folded_into_it() {
    let mut refused_on: Option<&'static str> = None;
    let reached = two_models_split_and_fused_noting_refusals(THE_DRAW, &mut |why| {
        refused_on = Some(which_predicate_the_refusal_names(why));
    });
    assert_eq!(
        (reached.fused, reached.one_sided),
        (1, 1),
        "the draw no longer reaches the one fuse it is kept for: {reached:?}"
    );
    assert_eq!(
        (reached.ok, reached.refused),
        (0, 1),
        "the fuse answered: either the meet's box was tightened, and two solids is \
         the right answer, or a meet whose box is 2 012 times the allowance was folded \
         into its corner again without the policy being asked: {reached:?}"
    );
    assert_eq!(
        refused_on,
        Some(ALONG_A_CROSSING),
        "the refusal names a different predicate than \"{ALONG_A_CROSSING}\": {refused_on:?}"
    );
}
