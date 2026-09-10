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
//! wrong-model comparison in `slot_in` was always true in release and a key from
//! another live model resolved to whatever now sat in that slot. A test for
//! that written anywhere else in this crate would pass under `just gate` for
//! the wrong reason. The two tests below are that test, and the id is a real
//! counter in every build now (`docs/decisions.md` §82.10).
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

use rcad_geom::{Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// One millimetre cube, and a block a thousand times its size.
///
/// Both are the first solid their own model holds, so both keys name slot zero.
/// That is what makes the wrong answer a *plausible* one rather than a crash:
/// the small cube's key resolves, against the other model, to a solid that is
/// really there.
const SMALL: f64 = 1.0;
const LARGE: f64 = 10.0;

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

/// Reading a solid out of a model with another model's key panics.
///
/// Both keys name slot zero of their own model, so before `ModelId` became a
/// real number in every build this call did not fail. It handed back the ten
/// millimetre block: `SolidView::measure` on it answered 1000 mm³ where the
/// caller's key named a one millimetre cube of 1 mm³. Nothing said so.
#[test]
#[should_panic(expected = "this solid key was made by a different model")]
fn a_solid_key_from_another_model_names_a_stranger_and_must_not_be_taken() {
    let (_mine, cube) = a_model_holding_one_block(SMALL);
    let (theirs, _block) = a_model_holding_one_block(LARGE);

    let _ = theirs.solid(cube);
}

/// A fuse handed another model's solid panics, rather than answering `Ok`.
///
/// This is the one that mattered. `fuse` takes its arguments by key, so the
/// zero-sized id let a caller ask for the union of a stranger's cube with this
/// model's block and get **`Ok`** back. What it had really done was fuse the
/// block against itself and hand back a [`Fused`](rcad_topo::Fused) naming no
/// solids at all; the cube was never looked at. A wrong answer wearing `Ok` is
/// the shape `CLAUDE.md`'s "crash > corrupt" is about.
#[test]
#[should_panic(expected = "this solid key was made by a different model")]
fn a_fuse_given_another_models_solid_must_not_answer_ok() {
    let (_mine, cube) = a_model_holding_one_block(SMALL);
    let (mut theirs, block) = a_model_holding_one_block(LARGE);

    let _ = theirs.fuse(
        &[cube],
        &[block],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
}

/// A model of its own, holding one square block `side` millimetres on a side.
fn a_model_holding_one_block(side: f64) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let block = model
        .add_extrusion(
            &square,
            side,
            &Isometry::translation(Vector::new(0.0, 0.0, 0.0)).expect("a finite placement"),
        )
        .expect("building a box needs no decision");
    (model, block)
}
