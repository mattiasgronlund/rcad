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
//! # The split and the rule are the same number, and that is a defect
//!
//! [`how_one_model_is_fused`] reads `header % (solids.len() + 1)` for the split
//! and `header % 3` for the rule. For a model with **two** solids those are the
//! same expression, so of the nine (split, rule) pairs only three are
//! reachable: an empty `arguments` under `Union`, one against one under
//! `Intersection`, and an empty `tools` under `Difference`. A two-sided union
//! and a two-sided difference cannot be drawn at all. `docs/decisions.md` §95
//! records what that costs over the checked-in corpus and why the layout is
//! left alone for now rather than reinterpreting 1740 seeds in passing.

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
    /// **A measurement of the defect the file header names**, not a stage. For
    /// a two-solid model the header cannot draw a two-sided union or a
    /// two-sided difference at all, and this is what that costs. A change to
    /// the byte layout that claims to fix it has to move this number to prove
    /// it.
    pub one_sided: usize,
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
    if arguments.is_empty() || tools.is_empty() {
        reached.one_sided = 1;
    }

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

    reached.fused = 1;
    match model.fuse(arguments, tools, rule, &policy, &mut ProvenanceLog::new()) {
        Ok(_) => reached.ok = 1,
        Err(_) => reached.refused = 1,
    }
    reached
}
