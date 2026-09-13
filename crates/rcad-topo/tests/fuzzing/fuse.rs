//! Arbitrary bytes into two [`Model`]s, fused under all three rules.
//!
//! `Model::fuse` is the one public entry point a stranger's bytes reach that
//! nothing else fuzzes: every other target either reads a single `Model` or a
//! pair of surfaces, and none of them call `fuse` at all.
//!
//! # Why two `Model`s and not one cross-fused pair
//!
//! `Model::fuse` takes `arguments` and `tools` as `&[SolidKey]` against
//! `&mut self` — both groups live in the *same* arena. There is no public way
//! to move a solid from one deserialized `Model` into another's arena, and
//! there should not be: `crates/rcad-topo/src/key.rs` gives every key type a
//! debug-only `ModelId` for exactly this reason, and using a key against a
//! model that did not mint it is asserted against by name. Splitting the input
//! in two, deserializing a `SolidKey` from one half and feeding it to the other
//! half's `Model::fuse`, would therefore do nothing but rediscover that assert
//! on nearly every run — a documented contract, not a finding.
//!
//! So this decodes **two independent `Model`s** from the two halves of the
//! input — doubling how much of every corpus entry's bytes turn into a model,
//! and giving two independent (rule, policy, split) draws per run — and for
//! each one that decodes, fuses a byte-chosen split of *that model's own*
//! solids against each other.
//!
//! # Validate first, or let a panic on unsound input be the finding?
//!
//! **Validates first.** `Model::fuse` documents that it panics if an input
//! solid is broken in a way `Model::validate` rejects — that is contract, not
//! accident, and a target that skips validation would mostly rediscover that
//! documented panic rather than find anything new. A deserialized `Model` is
//! key-sound and shape-possible but **not** geometrically sound. So the
//! question this asks is the one left open: can a model that *passes*
//! `validate` still crash `fuse`?
//!
//! `Policy::tolerant`'s scale always comes from the model's own
//! `Scale::nominal_size()`, never from fuzzed bytes directly, because a bad
//! numeric parameter there is an already-known, already-reported panic and
//! drawing it from the model sidesteps rediscovering that one on every run.
//!
//! # The rule comes off the high nibble, and the split off the whole byte
//!
//! [`how_one_model_is_fused`] reads `header % (solids.len() + 1)` for the split
//! and `(header >> 4) % 3` for the rule. Until 2026-09-13 the rule read
//! `header % 3`, which for a model with **two** solids is the same expression as
//! the split: only three of the nine (split, rule) pairs could be drawn, and a
//! two-sided union or a two-sided difference could not be drawn at all.
//!
//! **The high nibble is the version of that fix which moves no model payload.**
//! Every seed still cuts at the same byte and decodes to the model it always
//! did. Giving the rule a second header byte instead — in front of the halves or
//! taken off the end — shifts or truncates them, and a minimised seed is
//! byte-tight: `docs/decisions.md` §95.4 measured 678 fuses falling to 336 and
//! to 80. Only the operation a seed draws moves, on 269 of 3498 halves.
//!
//! Because a half's header is the other half's nibble swap, the two halves read
//! each other's nibble for the rule, which is what [`two_models_split_and_fused`]
//! already used `rotate_left(4)` for.
//!
//! **A one-sided draw is still two thirds of what a two-solid model can draw,
//! and that is the split's own modulus rather than this.** `Reached::one_sided`
//! counts it and §95.4 says why it is left alone.

// Shared between a test binary and a fuzz target, and each uses a subset.
#![allow(dead_code)]

#[path = "the_door.rs"]
mod the_door;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule};

use the_door::every_number_the_door_promised_is_real;

/// How far one draw of bytes got.
///
/// A count per stage and not one total: a byte string that does not decode, a
/// model that does not validate and a fuse that refuses fail for entirely
/// different reasons, and one number would let the deep stages fall to nothing
/// behind the shallow ones.
#[derive(Clone, Copy, Debug, Default)]
pub struct Reached {
    /// Halves handed to the decoder.
    pub halves: usize,
    /// Halves that decoded into a model.
    pub decoded: usize,
    /// Models that passed the certified sweep.
    pub validated: usize,
    /// Fuses actually run.
    pub fused: usize,
    /// Fuses run with nothing on one of the two sides.
    ///
    /// Not a stage, and **not a measurement of the rule-and-split defect** —
    /// taking the rule off the high nibble left this number exactly where it
    /// was, because one-sidedness comes from `header % (solids.len() + 1)`
    /// having two degenerate outcomes of three. A one-solid model has no
    /// two-sided draw at all.
    pub one_sided: usize,
    /// Two-sided fuses under `Union`, which could not be drawn before.
    ///
    /// Zero for every seed checked in until the rule moved off the split, and
    /// that is what the replay asserts against.
    pub two_sided_unions: usize,
    /// Two-sided fuses under `Difference`, which could not be drawn before.
    pub two_sided_differences: usize,
    /// Fuses that came back with a model.
    pub ok: usize,
    /// Fuses that refused by name.
    pub refused: usize,
}

impl Reached {
    /// What two draws reached between them.
    #[must_use]
    pub fn and(self, other: Self) -> Self {
        Self {
            halves: self.halves + other.halves,
            decoded: self.decoded + other.decoded,
            validated: self.validated + other.validated,
            fused: self.fused + other.fused,
            one_sided: self.one_sided + other.one_sided,
            two_sided_unions: self.two_sided_unions + other.two_sided_unions,
            two_sided_differences: self.two_sided_differences + other.two_sided_differences,
            ok: self.ok + other.ok,
            refused: self.refused + other.refused,
        }
    }
}

/// **One draw of bytes, cut into a header and two `Model`s, both fused.**
///
/// The fuzz target and the suite's replay both call this and nothing narrower.
#[must_use]
pub fn two_models_split_and_fused(data: &[u8]) -> Reached {
    let Some((&header, rest)) = data.split_first() else {
        return Reached::default();
    };
    let half = rest.len() / 2;
    let (first_half, second_half) = rest.split_at(half);

    how_one_model_is_fused(first_half, header)
        .and(how_one_model_is_fused(second_half, header.rotate_left(4)))
}

/// Decode one `Model`, and — if it passes `validate` — fuse a `header`-chosen
/// split of its own solids against each other under a `header`-chosen rule and
/// policy.
///
/// # Panics
///
/// If a model came through `Deserialize` carrying a number the door promised it
/// would refuse, or if `fuse` panics, which is the whole point.
#[must_use]
pub fn how_one_model_is_fused(bytes: &[u8], header: u8) -> Reached {
    let mut reached = Reached {
        halves: 1,
        ..Reached::default()
    };
    let Ok(mut model) = postcard::from_bytes::<Model>(bytes) else {
        return reached;
    };
    reached.decoded = 1;
    every_number_the_door_promised_is_real(&model);
    if model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .is_err()
    {
        return reached;
    }
    reached.validated = 1;

    let solids: Vec<_> = model.solids().map(|solid| solid.key()).collect();
    if solids.is_empty() {
        return reached;
    }
    let split = usize::from(header) % (solids.len() + 1);
    let (arguments, tools) = solids.split_at(split);
    let one_sided = arguments.is_empty() || tools.is_empty();
    if one_sided {
        reached.one_sided = 1;
    }

    let rule = match (header >> 4) % 3 {
        0 => Rule::Union,
        1 => Rule::Intersection,
        _ => Rule::Difference,
    };
    if !one_sided {
        match rule {
            Rule::Union => reached.two_sided_unions = 1,
            Rule::Difference => reached.two_sided_differences = 1,
            Rule::Intersection => {}
            // Imprint and symmetric difference, when they arrive, are neither.
            _ => {}
        }
    }
    let policy = if header & 0x80 == 0 {
        Policy::certified()
    } else {
        Policy::tolerant(model.scale().nominal_size())
    };

    reached.fused = 1;
    match model.fuse(arguments, tools, rule, &policy, &mut ProvenanceLog::new()) {
        Ok(_) => reached.ok = 1,
        Err(_) => reached.refused = 1,
    }
    reached
}
