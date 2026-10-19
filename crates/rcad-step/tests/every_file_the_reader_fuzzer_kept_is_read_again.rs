//! The reader's checked-in corpus, replayed on every commit.
//!
//! # What runs where
//!
//! `fuzz/fuzz_targets/no_step_file_makes_the_reader_panic.rs` is the
//! `cargo-fuzz` target and it needs a nightly compiler for its sanitizer;
//! `rust-toolchain.toml` pins a stable one, so **`just gate` does not run it**
//! — `just fuzz` does. What runs on every commit is this file, over the same
//! harness: `tests/fuzzing/reader.rs` holds it, and the target reaches across
//! to that file rather than copying it.
//!
//! Replaying the corpus means a case a fuzzing run once found can never come
//! back unnoticed. It also means the corpus can be *asked what it covers*, and
//! that is the reason this file exists at all.
//!
//! # The corpus reached nothing past the first line of the mapping
//!
//! Measured 2026-09-13, before the two control seeds below were added: of 907
//! seeds, **135 parsed and 0 imported**, and all 135 stopped at the same place
//! — `what_the_file_declared` at `crates/rcad-step/src/import.rs:73`, refusing
//! `TheFileDeclaresNoLengthUnit`. Not one reached
//! `nothing_is_placed_somewhere_this_reader_cannot_move_it`, the
//! `MANIFOLD_SOLID_BREP` walk, `solid_from`, or the `Model::validate` sweep the
//! harness ends with.
//!
//! That is a corpus that cannot say what its target's own doc comment says it
//! says, and it was invisible because nothing counted. A coverage-guided run
//! keeps an input because it reached somewhere new, and byte mutation was never
//! going to build a `GEOMETRIC_REPRESENTATION_CONTEXT` with a `LENGTH_UNIT`
//! hanging off it from nothing. Even the hand-written `a_small_whole_file`
//! declares its `LENGTH_UNIT` as a free-standing complex instance that no
//! context references, so it refuses too.
//!
//! # The two control seeds
//!
//! `a_block_that_imports` is `written_by_hand::a_block([2.0, 4.0, 8.0],
//! ".MILLI.")` — the same block three other test files measure — and it is the
//! corpus's only seed that reaches `solid_from` and the sweep.
//! `a_block_with_no_solid_in_it` is that file with its one
//! `MANIFOLD_SOLID_BREP` line deleted, so it gets past the unit and the
//! placements and is then refused `TheFileHoldsNoSolid`. One either side, which
//! is what keeps a floor of "at least one imports" from being met by a seed
//! that only ever refuses and a floor of "at least one refuses" from being met
//! by the 772 that do not even parse.
//!
//! **The seeds are worth more to the fuzzer than to this file.** An overnight
//! run now has something to mutate from, which is the half of the problem a
//! replay test cannot fix.
//!
//! # Where the floors below come from
//!
//! Measured on this tree, 2026-09-13, by
//!
//! ```text
//! just t -p rcad-step every_file_the_reader_fuzzer_kept_is_read_again:: \
//!     --no-capture
//! ```
//!
//! which reported, with the two control seeds in:
//!
//! ```text
//! Reached { files: 909, parsed: 137, instances: 547, imported: 1, solids: 1, refused: 136 }
//! ```
//!
//! in 22 ms. Each floor is set at roughly three quarters of that, so it catches
//! a stage falling silent without failing on ordinary drift — the same rule
//! `every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs` uses. The two
//! that are not three quarters are the deep ones: `imported` and the solid
//! under it come from one seed, so their floor is 1 and the control seed is
//! what holds it up.

#![forbid(unsafe_code)]

mod fuzzing;

use std::path::PathBuf;

use fuzzing::reader::{Reached, everything_one_file_is_put_through};

/// Where the checked-in corpus lives.
fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fuzz/corpus/no_step_file_makes_the_reader_panic")
}

#[test]
fn every_seed_in_the_checked_in_corpus_is_read_without_panicking() {
    let mut reached = Reached::default();
    let listing = std::fs::read_dir(corpus()).expect("the corpus is checked in beside the target");
    for entry in listing {
        let path = entry.expect("a readable directory entry").path();
        if !path.is_file() {
            continue;
        }
        let bytes = std::fs::read(&path).expect("a readable seed");
        reached = reached.and(everything_one_file_is_put_through(&bytes));
    }

    println!("{reached:?}");
    assert!(
        reached.files >= 700,
        "only {} seeds are checked in, so the corpus has shrunk",
        reached.files
    );
    assert!(
        reached.parsed >= 100,
        "only {} seeds parsed, so the corpus has drifted away from being files",
        reached.parsed
    );
    assert!(
        reached.instances >= 400,
        "only {} instances were walked, so the graphs have gone empty",
        reached.instances
    );
    assert!(
        reached.refused >= 100,
        "only {} seeds were refused by the mapping, so the refusal paths are \
         no longer reached",
        reached.refused
    );
    assert!(
        reached.imported >= 1,
        "no seed imported, so `solid_from` and the validate sweep are reached \
         by nothing in this corpus — which is exactly what was true before \
         `a_block_that_imports` was checked in"
    );
    assert!(
        reached.solids >= 1,
        "no solid was imported, so nothing walked a `MANIFOLD_SOLID_BREP`"
    );
}

/// The two named seeds, each asserted to be the thing it was checked in as.
///
/// A floor is a count and a count can be met by the wrong seeds. These two are
/// named, so what they are for is asserted by name rather than left to the
/// arithmetic above.
#[test]
fn the_two_named_control_seeds_still_land_either_side_of_the_mapping() {
    let imports = std::fs::read(corpus().join("a_block_that_imports"))
        .expect("the control seed is checked in");
    let reached = everything_one_file_is_put_through(&imports);
    assert_eq!(reached.imported, 1, "{reached:?}");
    assert_eq!(reached.solids, 1, "{reached:?}");

    let gutted = std::fs::read(corpus().join("a_block_with_no_solid_in_it"))
        .expect("the control seed is checked in");
    let reached = everything_one_file_is_put_through(&gutted);
    assert_eq!(reached.parsed, 1, "{reached:?}");
    assert_eq!(
        reached.refused, 1,
        "this seed gets past the unit and the placements and is then refused \
         for holding no solid: {reached:?}"
    );
}
