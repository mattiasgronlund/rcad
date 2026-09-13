//! The fuse's checked-in corpus, replayed on every commit.
//!
//! `fuzz/fuzz_targets/two_solids_fused_by_a_stranger.rs` is the `cargo-fuzz`
//! target, needs nightly, and does not run in `just gate`. This file does, over
//! the same harness — `tests/fuzzing/fuse.rs` — which the target reaches across
//! to rather than copying.
//!
//! # This is the row that was already worth replaying
//!
//! Measured 2026-09-13, before the two control seeds below were added: 1740
//! seeds, 3480 halves, **910 of them decoded into a `Model`, 669 reached
//! `Model::fuse`, 255 came back with a model and 414 refused**. The models are
//! real — 7 and 12 faces are the common shapes — and the refusals are the
//! interesting half: `ResultIsNotManifold`, and `Undecidable` raised by *where
//! three surfaces meet*, *a point inside a piece of a face* and *where two
//! planes cross*. That is 669 fuses a commit can break, for 0.79 s.
//!
//! It is also the only one of the three corpora that was healthy, and the
//! reason is worth writing down: this target splits its input in two and
//! decodes a `Model` from each half, so a run gets two draws per seed off the
//! same `postcard` door that leaves the writer's corpus at 16 models out of
//! 1180.
//!
//! # The split and the rule used to be the same number
//!
//! For a model with two solids `header % (solids.len() + 1)` and `header % 3`
//! are the same expression, so only three of the nine (split, rule) pairs could
//! be drawn and **a two-sided union or difference could not be drawn at all**.
//! §95.4 measured that rather than arguing it: over the corpus as it stood, the
//! 678 fuses landed on 23 distinct `(sidedness, rule, outcome)` triples and not
//! one of them was a two-sided union or a two-sided difference.
//!
//! Taking the rule off the high nibble instead — `(header >> 4) % 3` — costs no
//! seed its model, because the byte the input is cut at does not move. It is 28
//! triples now, 9 gained and 4 lost, and `two_sided_unions` and
//! `two_sided_differences` below are the two counters that were flatly zero
//! before it. §95.4 has the whole table, including why a second header byte was
//! declined: it moves the payload, and 678 fuses become 336 or 80.
//!
//! **It buys coverage and that is measured, not argued.** The same 1751 files
//! through the `cargo-fuzz` binary reach 14 788 to 14 793 edges under the new
//! layout against 14 765 to 14 766 under the old — three runs and two, because
//! the count is not deterministic, and the ranges do not overlap.
//!
//! **`one_sided` did not move, and that is the point of keeping it.** One-sided
//! draws come from the split's own modulus having two degenerate outcomes of
//! three, which is a separate matter from the rule and is left alone; §95.4 says
//! why.
//!
//! # The four control seeds
//!
//! Every one is `[header] ++ bytes ++ bytes`, so both halves decode to the same
//! model and the second header is the first one rotated by four.
//!
//! | seed | header | what it draws |
//! |---|---|---|
//! | `two_cubes_that_fuse` | `0x01` | two cubes of side 2 overlapping at `(1, 1, 1)`, unioned then intersected, both `Ok` |
//! | `two_cubes_touching_along_one_edge` | `0x02` | two cubes meeting along one edge at `(2, 2, 0)`, refused `ResultIsNotManifold { count: 4 }` |
//! | `two_cubes_one_taken_out_of_the_other` | `0x22` | the overlapping cubes again, a **two-sided difference** both halves, both `Ok` |
//! | `a_curved_face_with_a_hole_that_goes_too_far_round` | `0xa0` | a **two-sided intersection** refused `CurvedFaceWithAHoleGoesTooFarRound` |
//!
//! One either side of the fuse, named, so the floors below cannot be met by a
//! corpus that has quietly stopped doing one of them. The last two are the
//! rule's move: the difference seed holds up the thinnest floor here, and the
//! curved-face seed is the **one branch the move would otherwise have dropped**
//! — exactly one half in the corpus reached
//! `CurvedFaceWithAHoleGoesTooFarRound`, at header `0xf4`, and under the new
//! layout that half draws a union instead. `0xa0` draws the same tolerant
//! two-sided intersection on the same model, so the branch is kept by name
//! rather than by luck.
//!
//! # Where the floors below come from
//!
//! Measured on this tree, 2026-09-13, by
//!
//! ```text
//! just t -p rcad-topo \
//!     --test every_model_the_fuse_fuzzer_kept_is_fused_again --no-capture
//! ```
//!
//! which reported, with all four control seeds in:
//!
//! ```text
//! Reached { halves: 3502, decoded: 926, validated: 682, fused: 682,
//!           one_sided: 569, two_sided_unions: 69, two_sided_differences: 7,
//!           ok: 265, refused: 417 }
//! ```
//!
//! in 0.81 s, and each floor is set at roughly three quarters of that.
//!
//! Before the rule moved off the split the same run read `halves: 3498,
//! decoded: 922, validated: 678, fused: 678, one_sided: 569, ok: 260,
//! refused: 418` — so the move itself cost no half its model, and the four extra
//! halves are the two seeds added with it.
//!
//! **The seven crash seeds §92 rescued are in that number and cost nothing.**
//! They landed on `main` while this file was being written, and replaying them
//! here moves halves 3484 → 3498, fuses 673 → 678, `ok` 257 → 260 and refusals
//! 416 → 418, for no measurable time at all. Seven seeds a night found, five
//! more fuses a commit can break.
//!
//! The `one_sided` assertion is not a floor and is written the other way round:
//! it fails when the number *drops*, because a drop means the split's modulus
//! has been changed and §95.4 is out of date. It survived the rule moving off
//! the split untouched, which is the evidence that the two are separate defects.

#![forbid(unsafe_code)]

mod fuzzing;

use std::path::PathBuf;

use fuzzing::fuse::{Reached, two_models_split_and_fused};

/// Where the checked-in corpus lives.
fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fuzz/corpus/two_solids_fused_by_a_stranger")
}

/// Every seed checked in beside the fuzz target, replayed.
fn every_seed() -> impl Iterator<Item = Vec<u8>> {
    let listing = std::fs::read_dir(corpus()).expect("the corpus is checked in beside the target");
    listing.filter_map(|entry| {
        let path = entry.expect("a readable directory entry").path();
        path.is_file()
            .then(|| std::fs::read(&path).expect("a readable seed"))
    })
}

#[test]
fn every_seed_in_the_checked_in_corpus_is_fused_without_panicking() {
    let mut reached = Reached::default();
    for bytes in every_seed() {
        reached = reached.and(two_models_split_and_fused(&bytes));
    }

    println!("{reached:?}");
    assert!(
        reached.halves >= 2_600,
        "only {} halves are checked in, so the corpus has shrunk",
        reached.halves
    );
    assert!(
        reached.decoded >= 680,
        "only {} halves decoded into a model, so the door has moved under the \
         corpus",
        reached.decoded
    );
    assert!(
        reached.validated >= 500,
        "only {} models passed the certified sweep, so the corpus is decoding \
         models the sweep turns away",
        reached.validated
    );
    assert!(
        reached.fused >= 500,
        "only {} fuses were reached, so the models that pass the sweep now \
         hold no solid to fuse",
        reached.fused
    );
    assert!(
        reached.ok >= 190,
        "only {} fuses came back with a model, so the corpus now refuses \
         nearly everything and this replay is asserting far less than it looks \
         like it is",
        reached.ok
    );
    assert!(
        reached.refused >= 300,
        "only {} fuses refused, so the refusal paths are no longer reached",
        reached.refused
    );
    assert!(
        reached.two_sided_unions >= 50,
        "only {} fuses were a union with something on both sides, and there \
         were none at all until the rule moved off `header % 3` — so the split \
         and the rule are the same number again (§95.4)",
        reached.two_sided_unions
    );
    assert!(
        reached.two_sided_differences >= 5,
        "only {} fuses were a difference with something on both sides, and \
         there were none at all until the rule moved off `header % 3` — so the \
         split and the rule are the same number again (§95.4). Two of them are \
         `two_cubes_one_taken_out_of_the_other`, so reaching zero here means \
         that seed stopped decoding too",
        reached.two_sided_differences
    );
    assert!(
        reached.one_sided * 2 > reached.fused,
        "fewer than half of {} fuses had a side with nothing in it, which \
         would mean the split's own modulus has been changed — move this \
         assertion and §95.4 rather than deleting them",
        reached.fused
    );
}

/// One named seed, replayed.
fn control_seed(name: &str) -> Reached {
    let bytes = std::fs::read(corpus().join(name)).expect("the control seed is checked in");
    two_models_split_and_fused(&bytes)
}

/// The four named seeds, each asserted to be the thing it was checked in as.
#[test]
fn the_named_control_seeds_still_land_either_side_of_the_fuse() {
    let reached = control_seed("two_cubes_that_fuse");
    assert_eq!(reached.halves, 2, "{reached:?}");
    assert_eq!(reached.fused, 2, "{reached:?}");
    assert_eq!(reached.ok, 2, "both halves are the same model: {reached:?}");
    assert_eq!(
        reached.one_sided, 0,
        "header 0x01 cuts between the two cubes on both halves: {reached:?}"
    );
    assert_eq!(
        reached.two_sided_unions, 1,
        "the first half unions the two cubes, which nothing could draw before \
         the rule moved off the split: {reached:?}"
    );

    let reached = control_seed("two_cubes_touching_along_one_edge");
    assert_eq!(reached.fused, 2, "{reached:?}");
    assert_eq!(
        reached.refused, 2,
        "two cubes meeting along one edge leave a result that is not a \
         manifold: {reached:?}"
    );

    let reached = control_seed("two_cubes_one_taken_out_of_the_other");
    assert_eq!(reached.fused, 2, "{reached:?}");
    assert_eq!(
        reached.two_sided_differences, 2,
        "header 0x22 is its own nibble swap, so both halves take one cube out \
         of the other: {reached:?}"
    );
    assert_eq!(reached.ok, 2, "{reached:?}");

    let reached = control_seed("a_curved_face_with_a_hole_that_goes_too_far_round");
    assert_eq!(reached.fused, 2, "{reached:?}");
    assert_eq!(
        reached.one_sided, 0,
        "both halves cut between the solids: {reached:?}"
    );
    assert_eq!(
        reached.refused, 1,
        "the first half is the tolerant two-sided intersection that refuses \
         `CurvedFaceWithAHoleGoesTooFarRound`, the only branch moving the rule \
         off the split would otherwise have dropped: {reached:?}"
    );
}
