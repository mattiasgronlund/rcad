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
//!
//! # What the answer is asked, beyond whether it crashed
//!
//! Three checks, and each one is a promise made somewhere else in the tree
//! (§95.6):
//!
//! - **A refusal leaves the model byte-identical.** §17.4's atomicity row, and
//!   `Model::fuse`'s own "Atomic" heading. Nothing else in the tree asked it:
//!   the nearest thing is `fusing_two_bars_side_by_side.rs`, which re-validates
//!   after a refusal and so would pass a model that had been half written and
//!   was still sound.
//! - **Every solid handed back resolves**, through `Model::solid`, in the model
//!   that minted it.
//! - **An `Ok` answer passes the sweep under the policy the fuse ran under** —
//!   `Model::fuse`'s "What `Ok` promises, and under which policy". §66.1
//!   measured this once over 1060 answers with a harness built for the day;
//!   here it stops being a one-off.
//!
//! What the bytes are compared with is `postcard`, which is the same door the
//! input came through, so this compares what a saved file would hold and not
//! the arena's own bookkeeping. `Reached::ok_that_wrote_something` is what says
//! that comparison can see a fuse's writes at all.
//!
//! # The `Ok` sweep is sound here because of the sweep above it
//!
//! **`Model::fuse` answering `Ok` does not mean the caller's model validates,
//! in general.** §87.3 leaves that case open on purpose: when the inputs fail
//! `the_inputs_fit_in_f64` the fuse skips `Assembly::check_dynamic_range` and
//! hands back the answer the arithmetic reached exactly, and `Model::validate`'s
//! thirteenth check — the coordinate range, the one check that folds over the
//! whole model — then refuses it.
//! `a_frame_a_long_way_from_the_origin_encloses_what_a_frame_encloses` is such a
//! fuse and it passes on `main`.
//!
//! It cannot arise here, because the model was swept under `Policy::certified()`
//! before it was fused. `the_inputs_fit_in_f64` folds over the input solids
//! alone and takes chords alone, so its ratio is bounded by the whole-model
//! ratio that sweep has already accepted: no coordinate it sees is larger and no
//! edge it sees is shorter. So a model that reaches the fuse here fits, the
//! range check runs, and a fuse that would have needed the carve-out refuses
//! rather than answering.
//!
//! **Which makes that sweep load-bearing twice over.** Moving it to the fuse's
//! own policy, or taking it out, does not merely change what reaches the fuse —
//! it makes the assertion below start firing on good answers.
//!
//! # A union or an intersection is asked both ways round
//!
//! A fourth check, and the first of `fuses_that_must_agree.rs`'s laws to reach
//! a stranger's solids (§129.5 item 3): a two-sided union or intersection is
//! fused again with the two sides swapped, on a copy of the model as it was,
//! under the same policy, and the two answers are compared **the way that
//! file's `both_ways_round_of` compares them**:
//!
//! - a refusal on either side is not held to anything, because the two orders
//!   genuinely take different paths through the arithmetic;
//! - one order answering and the other handing back an answer no `Model` can
//!   hold is counted when every input face is flat and a failure when one is
//!   curved — `both_agree_about_whether_there_is_an_answer`'s split between its
//!   flat and its round families;
//! - two answers must be the same bit for bit, arena order aside —
//!   `same_answer` over `answer_of(..).canonical()`.
//!
//! Difference is not asked, because `A − B` is not `B − A`. Nor is a one-sided
//! draw: the source only ever swaps one solid for one solid.
//!
//! **This used to be a copy of that comparison; now it is the comparison.**
//! `Answer`, `Face`, `Step`, `Run`, `Picking`, `WhatItSitsOn`, `answer_of`,
//! `turned_to_its_smallest`, `bits` and `same_answer` live in
//! `tests/fuzzing/comparison.rs`, reached from here and from
//! `fuses_that_must_agree.rs` by `#[path]`, so there is one copy of the
//! comparison rather than two that can drift apart. `same_answer`'s error is a
//! plain `String`, which that file wraps into proptest's `TestCaseError` and
//! this one leaves alone.

// Shared between a test binary and a fuzz target, and each uses a subset.
#![allow(dead_code)]

#[path = "comparison.rs"]
mod comparison;
#[path = "the_door.rs"]
mod the_door;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

use comparison::Answer;
use the_door::every_number_the_door_promised_is_real;

/// How far one draw of bytes got.
///
/// A count per stage and not one total: a byte string that does not decode, a
/// model that does not validate and a fuse that refuses fail for entirely
/// different reasons, and one number would let the deep stages fall to nothing
/// behind the shallow ones.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
    /// Solids handed back across every `Ok`, each one a key checked to resolve.
    ///
    /// An empty answer is a real answer (§17.4), so this is not `ok` again: it
    /// is how many keys the resolve check actually had to look at.
    pub solids_made: usize,
    /// `Ok` fuses after which the model's bytes are different.
    ///
    /// **The control on the byte comparison the refusal arm makes.** Were
    /// serializing a `Model` blind to what a fuse writes into it, that
    /// comparison would pass on a half-written model and nothing would say so —
    /// and this number would be zero.
    pub ok_that_wrote_something: usize,
    /// Two-sided unions and intersections fused a second time with the two
    /// sides swapped.
    pub asked_both_ways_round: usize,
    /// Of those, the ones where both orders answered and the two answers were
    /// compared and found the same.
    ///
    /// **The control on the comparison**: were both orders to refuse every
    /// time, the check would pass for free and this would be zero.
    pub the_same_both_ways_round: usize,
    /// Of those, the ones over flat solids where one order answered and the
    /// other said the answer is not a solid — counted, not failed, exactly as
    /// `fuses_that_must_agree.rs` counts it on its flat families (§48.2 item 5).
    pub a_solid_one_way_round_only: usize,
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
            solids_made: self.solids_made + other.solids_made,
            ok_that_wrote_something: self.ok_that_wrote_something + other.ok_that_wrote_something,
            asked_both_ways_round: self.asked_both_ways_round + other.asked_both_ways_round,
            the_same_both_ways_round: self.the_same_both_ways_round
                + other.the_same_both_ways_round,
            a_solid_one_way_round_only: self.a_solid_one_way_round_only
                + other.a_solid_one_way_round_only,
        }
    }
}

/// **One draw of bytes, cut into a header and two `Model`s, both fused.**
///
/// The fuzz target and the suite's replay both call this and nothing narrower.
#[must_use]
pub fn two_models_split_and_fused(data: &[u8]) -> Reached {
    two_models_split_and_fused_keeping_the_provenance(data, &mut ProvenanceLog::new())
}

/// **The same draw, handing the provenance back instead of dropping it.**
///
/// The fuzz target calls [`two_models_split_and_fused`] and throws all three
/// logs away — the validate sweep's, the fuse's, and the second sweep's.
/// `rcad-topo/benches/what_the_provenance_log_costs_a_fuzz_night.rs` goes
/// through here to count what a draw recorded, and this is the body both share
/// so that the draw being measured is the draw being fuzzed (§144.9).
///
/// **One log where the fuzz path keeps three**, which merges the counts and
/// changes no record that is made.
///
/// # Panics
///
/// As [`two_models_split_and_fused`] does.
#[must_use]
pub fn two_models_split_and_fused_keeping_the_provenance(
    data: &[u8],
    log: &mut ProvenanceLog,
) -> Reached {
    let Some((&header, rest)) = data.split_first() else {
        return Reached::default();
    };
    let half = rest.len() / 2;
    let (first_half, second_half) = rest.split_at(half);

    how_one_model_is_fused_keeping_the_provenance(first_half, header, log).and(
        how_one_model_is_fused_keeping_the_provenance(second_half, header.rotate_left(4), log),
    )
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
    how_one_model_is_fused_keeping_the_provenance(bytes, header, &mut ProvenanceLog::new())
}

/// The same draw of one model, recording into the caller's log rather than its
/// own three.
///
/// Why this door exists is
/// [`two_models_split_and_fused_keeping_the_provenance`]'s doc comment.
///
/// # Panics
///
/// As [`how_one_model_is_fused`] does.
#[must_use]
pub fn how_one_model_is_fused_keeping_the_provenance(
    bytes: &[u8],
    header: u8,
    log: &mut ProvenanceLog,
) -> Reached {
    let mut reached = Reached {
        halves: 1,
        ..Reached::default()
    };
    let Ok(mut model) = postcard::from_bytes::<Model>(bytes) else {
        return reached;
    };
    reached.decoded = 1;
    every_number_the_door_promised_is_real(&model);
    if model.validate(&Policy::certified(), log).is_err() {
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

    let asked_both_ways_round = !one_sided && matches!(rule, Rule::Union | Rule::Intersection);
    let unfused = asked_both_ways_round.then(|| model.clone());

    reached.fused = 1;
    let before = postcard::to_allocvec(&model).expect("a model that decoded serializes again");
    let answer = model.fuse(arguments, tools, rule, &policy, log);
    let after = postcard::to_allocvec(&model).expect("a fused model serializes");

    let forwards = match answer {
        Ok(fused) => {
            reached.ok = 1;
            reached.solids_made = fused.solids().len();
            if before != after {
                reached.ok_that_wrote_something = 1;
            }
            for &solid in fused.solids() {
                model
                    .solid(solid)
                    .expect("a solid the fuse handed back resolves in the model that made it");
            }
            model
                .validate(&policy, log)
                .expect("an answer the fuse called Ok passes the sweep under its own policy");
            WhatCameBack::Answered(the_answer_written_out(&model, fused.solids()))
        }
        Err(why) => {
            reached.refused = 1;
            assert!(
                before == after,
                "a refused fuse left the model changed: {} bytes where it had {}",
                after.len(),
                before.len()
            );
            WhatCameBack::from_refusal(&why)
        }
    };

    if let Some(mut unfused) = unfused {
        reached.asked_both_ways_round = 1;
        let every_face_is_flat = arguments
            .iter()
            .chain(tools)
            .all(|&solid| every_face_of_it_is_flat(&unfused, solid));
        let backwards = match unfused.fuse(tools, arguments, rule, &policy, log) {
            Ok(fused) => WhatCameBack::Answered(the_answer_written_out(&unfused, fused.solids())),
            Err(why) => WhatCameBack::from_refusal(&why),
        };
        match both_ways_round(&forwards, &backwards, every_face_is_flat, rule) {
            Ok(Agreed::TheSameAnswer) => reached.the_same_both_ways_round = 1,
            Ok(Agreed::ASolidOneWayRoundOnly) => reached.a_solid_one_way_round_only = 1,
            Ok(Agreed::NothingToCompare) => {}
            Err(why) => panic!("{why}"),
        }
    }
    reached
}

/// What one order of a fuse came back with, in the three shapes
/// `fuses_that_must_agree.rs`'s `Fused` tells apart.
#[derive(Debug)]
pub enum WhatCameBack {
    /// An answer, read by [`the_answer_written_out`].
    Answered(Answer),
    /// A refusal of any kind but the one below.
    Refused,
    /// [`Error::ResultIsNotManifold`]: the answer is not something a `Model`
    /// can hold.
    NotASolid,
}

impl WhatCameBack {
    /// A yes-or-no question and not a match over every error: whatever
    /// variant `Error` grows next, it is a refusal (§40).
    fn from_refusal(why: &Error) -> Self {
        if matches!(why, Error::ResultIsNotManifold { .. }) {
            WhatCameBack::NotASolid
        } else {
            WhatCameBack::Refused
        }
    }
}

/// What asking both ways round found, when it did not find a disagreement.
#[derive(Debug, PartialEq, Eq)]
pub enum Agreed {
    /// Both orders answered, and with the same answer bit for bit.
    TheSameAnswer,
    /// Flat solids only: one order answered and the other said the answer is
    /// not a solid. Counted, not failed.
    ASolidOneWayRoundOnly,
    /// At least one order refused, so there was nothing to hold them to.
    NothingToCompare,
}

/// **The two orders of one fuse, held to each other.**
///
/// A mirror of `both_ways_round_of` in `tests/fuses_that_must_agree.rs`, which
/// the module doc says why this cannot call:
/// `both_agree_about_whether_there_is_an_answer` first, then `same_answer`.
///
/// # Errors
///
/// The disagreement, in words, when the two answers differ, or when solids with
/// a curved face answer one way round and are not a solid the other.
pub fn both_ways_round(
    forwards: &WhatCameBack,
    backwards: &WhatCameBack,
    every_face_is_flat: bool,
    rule: Rule,
) -> Result<Agreed, String> {
    match (forwards, backwards) {
        (WhatCameBack::Answered(one), WhatCameBack::Answered(other)) => {
            comparison::same_answer(one, other, rule, "swapping the two sides")
                .map(|()| Agreed::TheSameAnswer)
        }
        (WhatCameBack::NotASolid, WhatCameBack::Answered(_))
        | (WhatCameBack::Answered(_), WhatCameBack::NotASolid) => {
            if every_face_is_flat {
                Ok(Agreed::ASolidOneWayRoundOnly)
            } else {
                Err(format!(
                    "swapping the two sides turned a {rule:?} of solids with a curved face whose \
                     answer is a solid into one whose answer no `Model` can hold, which cannot \
                     depend on the path taken"
                ))
            }
        }
        _ => Ok(Agreed::NothingToCompare),
    }
}

/// `comparison::answer_of`, named for what this fuzzer's callers already call
/// it: `every_model_the_fuse_fuzzer_kept_is_fused_again.rs` reaches this name
/// directly, so it stays even though the body is now one line.
#[must_use]
pub fn the_answer_written_out(model: &Model, solids: &[SolidKey]) -> Answer {
    comparison::answer_of(model, solids)
}

fn every_face_of_it_is_flat(model: &Model, solid: SolidKey) -> bool {
    model
        .solid(solid)
        .expect("the draw's own solid resolves")
        .faces()
        .all(|face| face.surface().as_plane().is_some())
}
