//! The door §144.9's measurement goes through still sees what a draw records.
//!
//! `rcad-topo/benches/what_the_provenance_log_costs_a_fuzz_night.rs` weighs the
//! provenance a fuzz draw makes against the draw itself, and it can only do that
//! because [`every_oracle_holds_of_keeping_the_provenance`] hands the log back
//! where the fuzz target drops it.
//!
//! **Take the log out of that door and the bench reads zero percent** — which is
//! the answer it went looking for, arrived at by measuring nothing. That is
//! §77.6's shape exactly: a check that passes for a property of how it was run
//! rather than of the tree, and the way to catch one is to take away what the
//! running supplied and look again. So this pins both halves: the door asks what
//! the front asks, and the log it fills is not empty.
//!
//! [`every_oracle_holds_of_keeping_the_provenance`]: fuzzing::every_oracle_holds_of_keeping_the_provenance

#![forbid(unsafe_code)]

mod fuzzing;

use fuzzing::{
    bytes_for, every_oracle_holds_of, every_oracle_holds_of_keeping_the_provenance,
    two_surfaces_from,
};
use rcad_geom::Surface;
use rcad_predicates::ProvenanceLog;

#[test]
fn a_fuzz_draw_hands_its_provenance_back_through_the_door() {
    let pair = the_corpus_draw_that_records_the_most();
    let mut log = ProvenanceLog::new();

    let through_the_door = every_oracle_holds_of_keeping_the_provenance(pair, &mut log);

    assert_eq!(
        through_the_door,
        every_oracle_holds_of(pair),
        "the door reached a different set of oracles than the front door does, so the two are \
         no longer the same draw"
    );
    assert!(
        !log.is_empty(),
        "this draw records four decisions and the door came back with none, so a measurement \
         taken through it would read as free whatever the kernel did"
    );
}

/// The seed in `fuzz/corpus/where_two_surfaces_cross/` that records more
/// decisions than any other, which is four of them.
///
/// A ball and a pipe written out at `4e138`, where a radius squared is most of
/// the way to overflowing — so the pair crosses in two runs and the walk along
/// them cannot settle every point's side. The bench above found it and wrote it
/// to `target/provenance-cost/`; these are the numbers `rcad-fuzz-explain`
/// decoded it into, which is what it exists for.
///
/// **The numbers and not the 56 bytes**, because a hex blob in a test says
/// nothing about what is being asked. `bytes_for` writes them back into the
/// layout the decoder reads, so this is still the fuzzer's own draw.
fn the_corpus_draw_that_records_the_most() -> [Surface; 2] {
    let ball_then_pipe = [1, 2];
    let out_there = 4.0162086526409553e138;
    let bytes = bytes_for(
        ball_then_pipe,
        &[
            4.00052033852302e138,
            out_there,
            out_there,
            out_there,
            out_there,
            out_there,
            out_there,
            out_there,
            2.6875,
            1073741824.0,
            -1090519034.0,
        ],
    );
    two_surfaces_from(&bytes).expect("a ball and a pipe")
}
