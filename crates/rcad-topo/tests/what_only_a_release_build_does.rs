//! The tests that are about what a **release** build does, and nothing else.
//!
//! `just gate` builds everything with `debug_assertions` on, so the code that
//! actually ships is never run by anything that gates. That is not a speed
//! question — the two profiles measured within 3% of each other on the same
//! workload — it is that every `debug_assert!` and every
//! `cfg!(debug_assertions)` branch is present in all 1820 gated tests and gone
//! in the build a user gets.
//!
//! `ModelId` in `crates/rcad-topo/src/key.rs` is the case that asked for this
//! file: a counter in a debug build and a zero-sized type in release, so the
//! wrong-model comparison in `slot_in` is always true in release and a key from
//! another live model resolves to whatever now sits in that slot. A test for
//! that written anywhere else in this crate would pass under `just gate` for
//! the wrong reason.
//!
//! # Adding a test here
//!
//! Write it in this file. `just check-release-build` names this file and this
//! crate, so nothing else has to change, and the file-level `cfg` below keeps
//! it out of the ordinary dev-profile suite.
//!
//! A later test that has to live in another crate needs one more `cargo
//! nextest run` line in that recipe, naming that crate and a file of this same
//! name. Reaching for `--workspace` instead would build every crate twice for a
//! handful of tests, which is the cost this arrangement exists to avoid.
//!
//! # Why the whole file is behind one `cfg`
//!
//! Under the dev profile this strips to an empty binary, so `just test` never
//! runs these and never has to know they exist. `check-release-build` passes
//! `--no-tests fail`, so that same emptiness is what makes the recipe fail
//! loudly if it ever stops building in release, or if its filter stops
//! selecting anything: an empty selection otherwise looks exactly like a clean
//! pass.
#![cfg(not(debug_assertions))]

/// The arm's own selftest: this file must be built with the assertions off.
///
/// Redundant with the file-level `cfg` on the day it is written, and that is
/// the point — it is what makes deleting that `cfg` a red `just test` instead
/// of a release arm that silently re-runs the dev suite. `docs/decisions.md`
/// §77.6 collects the checks that passed for a property of the environment
/// they ran in rather than of the tree; this is the guard against that shape.
#[test]
fn these_tests_are_built_with_the_debug_assertions_off() {
    assert!(
        !cfg!(debug_assertions),
        "this file ran with debug_assertions on, so it proves nothing about a \
         release build — see the recipe check-release-build"
    );
}
