//! §24.18's criterion 10, and the part of it that runs in CI.
//!
//! # What runs where
//!
//! `fuzz/fuzz_targets/where_two_surfaces_cross.rs` is the `cargo-fuzz` target,
//! and it needs a nightly compiler for its sanitizer. `rust-toolchain.toml`
//! pins a stable one (§8.1), so **`just default` does not run it** — `just fuzz`
//! does, on a toolchain a person asked for. `docs/decisions.md` §29 records that
//! plainly rather than leaving it to be discovered.
//!
//! What runs on every commit is this file, and it is the same property over the
//! same decoder: `crates/rcad-geom/tests/fuzzing/mod.rs` holds both, and the
//! fuzz target reaches across to that file rather than copying it.
//!
//! Two rows. The **corpus** is what is checked in beside the target: a handful
//! of seeds written out by hand and named for the configuration each one is,
//! plus everything a coverage-guided run kept, minimised. Every one is replayed,
//! so a case a fuzzing run once found can never come back unnoticed — and one of
//! them is a case it did find, `two_balls_of_almost_one_size_almost_in_one_place`
//! (§29).
//!
//! The **soak** is a couple of hundred thousand pseudo-random byte strings,
//! which is not coverage-guided and does not pretend to be. What it buys is that
//! the property is exercised over the whole decoded space on a machine that has
//! no nightly compiler on it.
//!
//! **Not every seed decodes to a pair of surfaces, and that is right.** A
//! coverage-guided run keeps an input because it reached somewhere new, and the
//! decoder's own refusals — a radius of nothing, a direction of nothing — are
//! branches like any other.

#![forbid(unsafe_code)]

mod fuzzing;

use std::path::PathBuf;

use fuzzing::{
    MOST_BYTES_A_PAIR_READS, every_point_of_every_crossing_is_on_both_surfaces, two_surfaces_from,
};

/// How many pseudo-random byte strings the soak draws.
const SOAK: usize = 200_000;

/// Where the checked-in corpus lives.
fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/corpus/where_two_surfaces_cross")
}

/// Every seed checked in beside the fuzz target, replayed.
#[test]
fn every_seed_in_the_checked_in_corpus_names_its_crossing_honestly() {
    let mut seeds = 0;
    let mut named = 0;
    let mut points = 0;
    let listing = std::fs::read_dir(corpus()).expect("the corpus is checked in beside the target");
    for entry in listing {
        let path = entry.expect("a readable directory entry").path();
        if !path.is_file() {
            continue;
        }
        let bytes = std::fs::read(&path).expect("a readable seed");
        seeds += 1;
        if let Some(pair) = two_surfaces_from(&bytes) {
            named += 1;
            points += every_point_of_every_crossing_is_on_both_surfaces(pair);
        }
    }
    println!(
        "{seeds} seeds in the corpus, {named} of them a pair of surfaces, {points} points asked \
         about"
    );
    assert!(seeds >= 100, "only {seeds} seeds are checked in");
    assert!(
        named * 2 > seeds,
        "only {named} of {seeds} seeds decoded to a pair, so the corpus has drifted away from \
         the thing it is a corpus of"
    );
    assert!(
        points > 50,
        "the corpus asked about only {points} points, so it is barely exercising the oracle"
    );
}

/// The soak, over the whole decoded space.
///
/// A cheap deterministic generator rather than a proptest strategy, because what
/// is being explored is the *bytes* and a strategy would put a shape on them
/// that the fuzzer's own mutations do not have.
#[test]
fn a_soak_over_pseudo_random_bytes_names_every_crossing_honestly() {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut drawn = 0;
    let mut named = 0;
    let mut points = 0;
    let mut bytes = vec![0u8; MOST_BYTES_A_PAIR_READS];

    for _ in 0..SOAK {
        for byte in &mut bytes {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *byte = state as u8;
        }
        drawn += 1;
        if let Some(pair) = two_surfaces_from(&bytes) {
            named += 1;
            points += every_point_of_every_crossing_is_on_both_surfaces(pair);
        }
    }

    println!(
        "{drawn} byte strings drawn, {named} of them a pair of surfaces, {points} points asked \
         about"
    );
    assert!(
        named * 4 > drawn,
        "only {named} of {drawn} draws named a pair, so the decoder is refusing most of the \
         space rather than exploring it"
    );
    assert!(
        points > 10_000,
        "the soak asked about only {points} points, so it is barely exercising the oracle"
    );
}
