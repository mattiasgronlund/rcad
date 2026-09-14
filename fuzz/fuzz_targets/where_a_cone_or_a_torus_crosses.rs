//! §24.18's criterion 3, asked only of pairs with a cone or a torus in them.
//!
//! The sibling target `where_two_surfaces_cross.rs` draws a plane, a ball or a
//! pipe and can draw nothing else, so the two curved kinds `Surface` has had
//! since M4 were reachable only from a hand-written table. This is the second
//! byte format that closes that, and it reaches across to the same shared file
//! rather than copying it — the oracles are one set of lines either way.
//!
//! **A second format and not a wider one.** The 830 seeds beside the other
//! target keep meaning what they meant, because nothing reads them differently.

#![no_main]

use libfuzzer_sys::fuzz_target;

// `#[path]` and not `include!`, because the shared file opens with a module doc
// comment and an inner attribute, and neither of those can be pasted into the
// middle of another file.
#[path = "../../crates/rcad-geom/tests/fuzzing/mod.rs"]
mod fuzzing;

use fuzzing::{every_oracle_holds_of, two_surfaces_with_a_cone_or_a_torus_from};
use rcad_fuzz_timing::timed;

fuzz_target!(|bytes: &[u8]| {
    timed(env!("CARGO_BIN_NAME"), bytes, || {
        if let Some(pair) = two_surfaces_with_a_cone_or_a_torus_from(bytes) {
            let _ = every_oracle_holds_of(pair);
        }
    });
});
