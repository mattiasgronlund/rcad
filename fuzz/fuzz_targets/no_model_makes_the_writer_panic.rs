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
#![no_main]

use libfuzzer_sys::fuzz_target;
use rcad_topo::Model;

fuzz_target!(|bytes: &[u8]| {
    let Ok(model) = postcard::from_bytes::<Model>(bytes) else {
        return;
    };
    let _ = rcad_step::write(&model, "fuzz");
});
