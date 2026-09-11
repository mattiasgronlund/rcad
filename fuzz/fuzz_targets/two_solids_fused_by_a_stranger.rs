//! Arbitrary bytes into two [`Model`]s, fused under all three rules.
//!
//! `Model::fuse` is the one public entry point a stranger's bytes reach that
//! nothing has fuzzed yet: every existing target either reads a single
//! `Model` (`no_step_file_makes_the_reader_panic.rs`,
//! `no_model_makes_the_writer_panic.rs`) or a pair of surfaces
//! (`where_two_surfaces_cross.rs`), and none of them ever call `fuse` at all.
//!
//! # Why two `Model`s and not one cross-fused pair
//!
//! `Model::fuse` takes `arguments` and `tools` as `&[SolidKey]` against
//! `&mut self` — both groups live in the *same* arena. There is no public way
//! to move a solid from one deserialized `Model` into another's arena, and
//! there should not be: `crates/rcad-topo/src/key.rs` gives every key type a
//! debug-only `ModelId` for exactly this reason, and using a key against a
//! model that did not mint it is asserted against by name
//! (`"this solid key was made by a different model"`). Splitting the input in
//! two, deserializing a `SolidKey` from one half and feeding it to the other
//! half's `Model::fuse`, would therefore do nothing but rediscover that
//! assert on nearly every run — a documented contract, not a finding
//! (CLAUDE.md's `docs/decisions.md` §16.10 is about claims of impossibility,
//! and the corresponding waste here is a claim of novelty).
//!
//! So this target decodes **two independent `Model`s** from the two halves of
//! the input — doubling how much of every corpus entry's bytes turn into a
//! model, and giving two independent (rule, policy, split) draws per run —
//! and for each one that decodes, fuses a byte-chosen split of *that model's
//! own* solids against each other. That is still `fuse` reached by a
//! stranger's bytes on both of its arguments; it just never needs a foreign
//! key to do it.
//!
//! # Validate first, or let a panic on unsound input be the finding?
//!
//! **Validates first.** `Model::fuse` documents that it panics if an input
//! solid is broken in a way `Model::validate` rejects — that is contract, not
//! accident, and a target that skips validation would mostly rediscover that
//! documented panic rather than find anything new. A deserialized `Model` is
//! key-sound and shape-possible (`Deserialize` already refused a dangling key
//! or an impossible shape) but **not** geometrically sound — `validate`
//! cannot see geometric coincidence at all. So the question this target asks
//! is the one left open: can a model that *passes* `validate` still crash
//! `fuse`?
//!
//! **What the door promised is asserted before `validate` runs, not assumed.**
//! A corner at no point and a slop that is not a length are both refused on
//! the way in (`docs/decisions.md` §44.5), and the infinite slop is the one
//! that matters here: `not_provably_off` keeps any magnitude above zero and
//! widens the corner into an interval straddling every surface, so a model
//! carrying one *passes* `validate` and reaches `fuse` with its corners
//! nowhere near its faces. Checking it here rather than after `validate` is
//! deliberate — `validate` is the thing that would be fooled.
//!
//! `Policy::tolerant`'s scale always comes from the model's own
//! `Scale::nominal_size()`, never from fuzzed bytes directly, because a bad
//! numeric parameter there is an already-known, already-reported panic
//! (`Policy::tolerant` asserts its scale is finite and positive) and drawing
//! it from the model sidesteps rediscovering that one on every run.
#![no_main]

use libfuzzer_sys::fuzz_target;
use rcad_fuzz_timing::timed;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule};

fuzz_target!(|data: &[u8]| {
    timed(env!("CARGO_BIN_NAME"), data, || {
        two_models_split_and_fused(data);
    });
});

/// One draw of bytes, cut into a header and two `Model`s.
fn two_models_split_and_fused(data: &[u8]) {
    let Some((&header, rest)) = data.split_first() else {
        return;
    };
    let half = rest.len() / 2;
    let (first_half, second_half) = rest.split_at(half);

    fuse_a_split_of_its_own_solids(first_half, header);
    fuse_a_split_of_its_own_solids(second_half, header.rotate_left(4));
}

/// Decode one `Model`, and — if it passes `validate` — fuse a `header`-chosen
/// split of its own solids against each other under a `header`-chosen rule
/// and policy.
fn fuse_a_split_of_its_own_solids(bytes: &[u8], header: u8) {
    let Ok(mut model) = postcard::from_bytes::<Model>(bytes) else {
        return;
    };
    every_number_the_door_promised_is_real(&model);
    if model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .is_err()
    {
        return;
    }

    let solids: Vec<_> = model.solids().map(|solid| solid.key()).collect();
    if solids.is_empty() {
        return;
    }
    let split = usize::from(header) % (solids.len() + 1);
    let (arguments, tools) = solids.split_at(split);

    let rule = match header % 3 {
        0 => Rule::Union,
        1 => Rule::Intersection,
        _ => Rule::Difference,
    };
    let policy = if header & 0x80 == 0 {
        Policy::certified()
    } else {
        Policy::tolerant(model.scale().nominal_size())
    };

    let _ = model.fuse(arguments, tools, rule, &policy, &mut ProvenanceLog::new());
}

/// What `Deserialize for Model` promised about the numbers in it.
///
/// Every corner is at a point, and every amount of slop is a length.
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
