//! Arbitrary bytes into two `Model`s, fused under all three rules.
//!
//! Everything this does is in `crates/rcad-topo/tests/fuzzing/fuse.rs`, and
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
#[path = "../../crates/rcad-topo/tests/fuzzing/fuse.rs"]
mod fuse;

use fuse::two_models_split_and_fused;
use rcad_fuzz_timing::timed;

fuzz_target!(|data: &[u8]| {
    timed(env!("CARGO_BIN_NAME"), data, || {
        let _ = two_models_split_and_fused(data);
    });
});
