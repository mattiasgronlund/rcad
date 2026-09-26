//! A meet of three surfaces is folded into a corner of the face only when it is
//! no further from the corner than writing the corner down cost (§253).
//!
//! `place_the_meets` used to fold a meet into any corner on both of its
//! crossings whose box it touched, without asking the policy, however wide the
//! meet's box was. Across every test in the workspace, one fold reached further
//! than the allowance `bake.rs` grants a stored corner: a draw in the fuse
//! target's corpus, whose meet reached `1.4e-10` from its corner, 2 012 times
//! that allowance. Under the tolerant policy the harness hands it, the fuse
//! answered with two solids. With the meet kept as a node, the pair goes to
//! `merge_points_that_are_one_point`, and the fuse refuses.
//!
//! The draw is named rather than rebuilt here because it is four and a half
//! kilobytes of `Model` and reads as nothing, the reason
//! `a_fused_draw_hands_its_provenance_back_through_the_door.rs` gives for the
//! same choice.

#![forbid(unsafe_code)]

mod fuzzing;

use std::path::PathBuf;

use fuzzing::fuse::two_models_split_and_fused;

/// The one draw in the corpus whose fold reached past the allowance.
const THE_DRAW: &str = "a45aa71b95846001b737b93456a65be63b2b815f";

#[test]
fn a_meet_further_from_a_corner_than_rounding_reaches_is_not_folded_into_it() {
    let seed = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fuzz/corpus/two_solids_fused_by_a_stranger")
        .join(THE_DRAW);
    let bytes = std::fs::read(&seed).expect("the draw is checked in beside the fuse target");
    let reached = two_models_split_and_fused(&bytes);
    assert_eq!(
        (reached.fused, reached.one_sided),
        (1, 1),
        "the draw no longer reaches the one fuse it is kept for: {reached:?}"
    );
    assert_eq!(
        (reached.ok, reached.refused),
        (0, 1),
        "the fuse answered, so a meet 2 012 times the allowance from its corner was folded \
         into it again without the policy being asked: {reached:?}"
    );
}
