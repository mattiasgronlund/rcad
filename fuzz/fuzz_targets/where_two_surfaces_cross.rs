//! §24.18's criterion 10: a pair of surfaces in, and criterion 3 asserted.
//!
//! Everything this does is in `crates/rcad-geom/tests/fuzzing/mod.rs`, and this
//! target **reaches across to that file** rather than copying it — so the
//! property `cargo fuzz` explores and the property the ordinary test suite
//! replays are the same lines of code. `docs/decisions.md` §29 records which of
//! the two runs in CI and which does not.

#![no_main]

use libfuzzer_sys::fuzz_target;

// `#[path]` and not `include!`, because the shared file opens with a module doc
// comment and an inner attribute, and neither of those can be pasted into the
// middle of another file.
#[path = "../../crates/rcad-geom/tests/fuzzing/mod.rs"]
mod fuzzing;

use fuzzing::{every_point_of_every_crossing_is_on_both_surfaces, two_surfaces_from};

fuzz_target!(|bytes: &[u8]| {
    if let Some(pair) = two_surfaces_from(bytes) {
        let _ = every_point_of_every_crossing_is_on_both_surfaces(pair);
    }
});
