//! Arbitrary bytes into `rcad_step::write`, which must never panic, and every
//! sound model it writes must read back as the same model.
//!
//! Everything this does is in `crates/rcad-step/tests/fuzzing/writer.rs`, and
//! this target **reaches across to that file** rather than copying it — so the
//! property `cargo fuzz` explores and the property the ordinary test suite
//! replays are the same lines of code. `docs/decisions.md` §29.9 records which
//! of the two runs in CI and which does not, and §95 records this target
//! joining the arrangement.

#![no_main]

use libfuzzer_sys::fuzz_target;

// `#[path]` and not `include!`, because the shared file opens with a module doc
// comment and an inner attribute, and neither of those can be pasted into the
// middle of another file.
#[path = "../../crates/rcad-step/tests/fuzzing/writer.rs"]
mod writer;

use rcad_fuzz_timing::timed;
use writer::one_model_decoded_and_written;

fuzz_target!(|bytes: &[u8]| {
    timed(env!("CARGO_BIN_NAME"), bytes, || {
        let _ = one_model_decoded_and_written(bytes);
    });
});
