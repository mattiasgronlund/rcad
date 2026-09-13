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
//! # The split and the rule are the same number
//!
//! `fuse.rs`'s header comment has it: for a model with two solids,
//! `header % (solids.len() + 1)` and `header % 3` are the same expression, so
//! only three of the nine (split, rule) pairs can be drawn and two of the three
//! have an empty side. `Reached::one_sided` counts what that costs rather than
//! describing it — **569 of 678 fuses, measured below** — and
//! `docs/decisions.md` §95 records why the layout is left alone for now: a
//! change to it reinterprets 1740 checked-in seeds, and the measurement of what
//! that disturbs is the thing to have before making it — the same argument
//! `every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs` makes about
//! its own byte layout.
//!
//! # The two control seeds
//!
//! Both are `[header] ++ bytes ++ bytes`, so both halves decode to the same
//! model and the second header is the first one rotated by four.
//! `two_cubes_that_fuse` is two cubes of side 2 overlapping at `(1, 1, 1)` under
//! header `0x01`, which intersects them and comes back with a model.
//! `two_cubes_touching_along_one_edge` is two cubes meeting along one edge at
//! `(2, 2, 0)` under header `0x02`, refused `ResultIsNotManifold { count: 4 }`.
//! One either side, named, so the floors below cannot be met by a corpus that
//! has quietly stopped doing one of the two.
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
//! which reported, with the two control seeds in:
//!
//! ```text
//! Reached { halves: 3498, decoded: 922, validated: 678, fused: 678,
//!           one_sided: 569, ok: 260, refused: 418 }
//! ```
//!
//! in 0.75 s, and each floor is set at roughly three quarters of that.
//!
//! **The seven crash seeds §92 rescued are in that number and cost nothing.**
//! They landed on `main` while this file was being written, and replaying them
//! here moves halves 3484 → 3498, fuses 673 → 678, `ok` 257 → 260 and refusals
//! 416 → 418, for no measurable time at all. Seven seeds a night found, five
//! more fuses a commit can break.
//!
//! The
//! `one_sided` assertion is not a floor and is written the other way round: it
//! fails when the number *drops*, because a drop means the shared modulus has
//! been changed and §95 is out of date.

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
        reached.one_sided * 2 > reached.fused,
        "fewer than half of {} fuses had a side with nothing in it, which \
         would mean the shared modulus `fuse.rs` names has been fixed — move \
         this assertion and §95 rather than deleting them",
        reached.fused
    );
}

/// The two named seeds, each asserted to be the thing it was checked in as.
#[test]
fn the_two_named_control_seeds_still_land_either_side_of_the_fuse() {
    let fuses = std::fs::read(corpus().join("two_cubes_that_fuse"))
        .expect("the control seed is checked in");
    let reached = two_models_split_and_fused(&fuses);
    assert_eq!(reached.halves, 2, "{reached:?}");
    assert_eq!(reached.fused, 2, "{reached:?}");
    assert_eq!(reached.ok, 2, "both halves are the same model: {reached:?}");

    let refuses = std::fs::read(corpus().join("two_cubes_touching_along_one_edge"))
        .expect("the control seed is checked in");
    let reached = two_models_split_and_fused(&refuses);
    assert_eq!(reached.fused, 2, "{reached:?}");
    assert_eq!(
        reached.refused, 2,
        "two cubes meeting along one edge leave a result that is not a \
         manifold: {reached:?}"
    );
}
