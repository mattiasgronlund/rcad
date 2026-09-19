//! The fuse's half of the door §144.9's measurement goes through.
//!
//! `benches/what_the_provenance_log_costs_a_fuzz_night.rs` weighs the provenance
//! a fuzz draw makes against the draw itself, and it can only do that because
//! [`two_models_split_and_fused_keeping_the_provenance`] hands back the three
//! logs the fuzz target drops — the validate sweep's, the fuse's, and the sweep
//! that checks the answer.
//!
//! **Take the log out of that door and the bench reads zero percent**, which is
//! the answer it went looking for, arrived at by measuring nothing. §77.6 is the
//! shape, and `rcad-geom/tests/a_fuzz_draw_hands_its_provenance_back_through_the_door.rs`
//! is the same pin on the surface targets' door.
//!
//! **The corpus and not one pinned draw, which is the difference from that
//! file.** A recording pair of surfaces is eleven numbers and reads as a shape;
//! the fuse's worst draw is four and a half kilobytes of `Model`, and a hex blob
//! that long in a test says nothing about what is being asked. What is checked
//! instead is the property the bench depends on: somewhere in the corpus the
//! door comes back with records, and everywhere in it the door answers what the
//! front answers.
//!
//! # And where the records come from, which is §162's account
//!
//! Walking the whole corpus with the log in hand is also the only place that can
//! say *which* draws record, so it says it. Measured on 2026-09-19: of 1751
//! seeds, the 1216 that never got a model past `validate` recorded **4 decisions
//! between them**, and the 535 that reached the fuse recorded the other 16 032 —
//! about thirty apiece.
//!
//! That is the half of §144.9 that held and the half that did not. A draw that
//! goes deep does record, so provenance is not free on the path a night takes;
//! and thirty records is nothing against the fuse that made them, which is why
//! `just provenance-cost` reads a tenth of a percent rather than a fifth of the
//! run. The assertion below is on the concentration and not on either count: the
//! numbers move whenever the corpus does, and what must not change is that the
//! records belong to the deep draws.
//!
//! [`two_models_split_and_fused_keeping_the_provenance`]: fuzzing::fuse::two_models_split_and_fused_keeping_the_provenance

#![forbid(unsafe_code)]

mod fuzzing;

use std::path::PathBuf;

use fuzzing::fuse::{
    two_models_split_and_fused, two_models_split_and_fused_keeping_the_provenance,
};
use rcad_predicates::ProvenanceLog;

#[test]
fn a_fused_draw_hands_its_provenance_back_through_the_door() {
    let mut seeds_that_recorded = 0;
    let mut records = 0;
    let mut stopped_before_validate = (0, 0);
    let mut stopped_at_validate = (0, 0);
    let mut fused = (0, 0);
    for bytes in every_seed() {
        let mut log = ProvenanceLog::new();
        let through_the_door = two_models_split_and_fused_keeping_the_provenance(&bytes, &mut log);

        assert_eq!(
            through_the_door,
            two_models_split_and_fused(&bytes),
            "the door reached somewhere different than the front door does, so the two are no \
             longer the same draw"
        );
        if !log.is_empty() {
            seeds_that_recorded += 1;
            records += log.len();
        }
        let how_far = if through_the_door.fused > 0 {
            &mut fused
        } else if through_the_door.validated > 0 {
            &mut stopped_at_validate
        } else {
            &mut stopped_before_validate
        };
        how_far.0 += 1;
        how_far.1 += log.len();
    }

    println!("{seeds_that_recorded} seeds recorded {records} decisions between them");
    println!("  never decoded or validated: {stopped_before_validate:?}  (seeds, records)");
    println!("  validated but never fused:   {stopped_at_validate:?}");
    println!("  reached the fuse:            {fused:?}");
    assert!(
        seeds_that_recorded > 0,
        "not one seed of the corpus recorded a decision through the door, so a measurement \
         taken through it would read as free whatever the kernel did"
    );
    assert!(
        stopped_before_validate.1 * 100 < fused.1,
        "{} of the {} records came from the {} draws that never reached the fuse, so provenance \
         is no longer the deep draws' cost and §162's account of where it comes from is wrong",
        stopped_before_validate.1,
        records,
        stopped_before_validate.0,
    );
}

/// Every seed checked in beside the fuzz target.
fn every_seed() -> impl Iterator<Item = Vec<u8>> {
    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fuzz/corpus/two_solids_fused_by_a_stranger");
    let listing = std::fs::read_dir(corpus).expect("the corpus is checked in beside the target");
    listing.filter_map(|entry| {
        let path = entry.expect("a readable directory entry").path();
        path.is_file()
            .then(|| std::fs::read(&path).expect("a readable seed"))
    })
}
