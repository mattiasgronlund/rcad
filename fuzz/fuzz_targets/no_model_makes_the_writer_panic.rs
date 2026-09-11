//! Arbitrary bytes into `rcad_step::write`, which must never panic.
//!
//! CLAUDE.md's rule that a caller's bytes may not panic, and the reader's own
//! fuzz target (`no_step_file_makes_the_reader_panic.rs`) is why this one
//! exists: that target found a real crash in its first five minutes, and the
//! writer had no target at all until this one.
//!
//! **`rcad_step::write` takes a `&Model`, not bytes, so this target has to
//! make one first.** `Model` implements `serde`'s `Deserialize`
//! (`crates/rcad-topo/src/serialize.rs`) — that is precisely the route a
//! stranger's bytes take into a `Model`, whether from a saved file or from
//! this fuzzer — so arbitrary bytes go through `postcard::from_bytes` and
//! whatever comes out, valid or not, goes straight into the writer.
//! `postcard::from_bytes` is already exercised by the ordinary suite
//! (`crates/rcad-topo/tests/writing_and_reading.rs`); this reaches the writer
//! with whatever it accepts, sound or not — a `Model` a fuzzer built is under
//! no obligation to be one `Model::validate` would let through, and the
//! writer owes an `Err` either way, never a crash.
//!
//! **A model that does come out is checked against what the door promised**,
//! not just handed on. `Deserialize` refuses a corner that is at no point and a
//! slop that is not a length (`docs/decisions.md` §44.5), and this target
//! reaches that door with more byte strings in a minute than the suite has in
//! total. Walking past a model whose corner is `NaN` would leave the promise
//! untested by the thing best placed to break it. The suite asks the same
//! question on the pinned stable toolchain, where `cargo fuzz` cannot run at
//! all, in
//! `no_mutation_of_a_model_that_still_reads_back_carries_a_number_that_is_not_real`.
#![no_main]

use libfuzzer_sys::fuzz_target;
use rcad_fuzz_timing::timed;
use rcad_topo::Model;

fuzz_target!(|bytes: &[u8]| {
    timed(env!("CARGO_BIN_NAME"), bytes, || {
        one_model_decoded_and_written(bytes);
    });
});

/// One draw of bytes, decoded the way a stranger's saved file is and written out.
fn one_model_decoded_and_written(bytes: &[u8]) {
    let Ok(model) = postcard::from_bytes::<Model>(bytes) else {
        return;
    };
    every_number_the_door_promised_is_real(&model);
    let _ = rcad_step::write(&model, "fuzz");
}

/// What `Deserialize for Model` promised about the numbers in it.
///
/// Every corner is at a point, and every amount of slop is a length. The second
/// one is the one that costs something if it is wrong: `rcad-topo`'s
/// `not_provably_off` keeps any magnitude above zero and widens the corner it
/// is asked about into an interval that straddles every surface, so an infinite
/// tag is the certified check switched off rather than loosened.
fn every_number_the_door_promised_is_real(model: &Model) {
    for vertex in model.vertices() {
        assert!(
            vertex.point().is_finite(),
            "a corner that is at no point came through the door: {:?}",
            vertex.point()
        );
        assert_it_is_a_length(vertex.approximation().map(|tag| tag.magnitude()));
    }
    for edge in model.edges() {
        assert_it_is_a_length(edge.approximation().map(|tag| tag.magnitude()));
    }
    for face in model.faces() {
        assert_it_is_a_length(face.approximation().map(|tag| tag.magnitude()));
    }
}

fn assert_it_is_a_length(magnitude: Option<f64>) {
    if let Some(magnitude) = magnitude {
        assert!(
            magnitude.is_finite() && magnitude >= 0.0,
            "a slop of {magnitude} came through the door, and it is not a length"
        );
    }
}
