//! Arbitrary bytes into the Part 21 reader, which must never panic.
//!
//! Everything this does is in `crates/rcad-step/tests/fuzzing/reader.rs`, and
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
#[path = "../../crates/rcad-step/tests/fuzzing/reader.rs"]
mod reader;

use rcad_fuzz_timing::timed;
use reader::everything_one_file_is_put_through;

fuzz_target!(|bytes: &[u8]| {
    timed(env!("CARGO_BIN_NAME"), bytes, || {
        let _ = everything_one_file_is_put_through(bytes);
    });
});
