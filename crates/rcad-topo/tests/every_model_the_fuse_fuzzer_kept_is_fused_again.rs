//! The fuse's checked-in corpus, replayed on every commit.
//!
//! `fuzz/fuzz_targets/two_solids_fused_by_a_stranger.rs` is the `cargo-fuzz`
//! target, needs nightly, and does not run in `just gate`. This file does, over
//! the same harness — `tests/fuzzing/fuse.rs` — which the target reaches across
//! to rather than copying.
//!
//! # This is the row that was already worth replaying
//!
//! Measured 2026-09-13, before the two control seeds below were added: 1740
//! seeds, 3480 halves, **910 of them decoded into a `Model`, 669 reached
//! `Model::fuse`, 255 came back with a model and 414 refused**. The models are
//! real — 7 and 12 faces are the common shapes — and the refusals are the
//! interesting half: `ResultIsNotManifold`, and `Undecidable` raised by *where
//! three surfaces meet*, *a point inside a piece of a face* and *where two
//! planes cross*. That is 669 fuses a commit can break, for 0.79 s.
//!
//! It is also the only one of the three corpora that was healthy, and the
//! reason is worth writing down: this target splits its input in two and
//! decodes a `Model` from each half, so a run gets two draws per seed off the
//! same `postcard` door that leaves the writer's corpus at 16 models out of
//! 1180.
//!
//! # The split and the rule used to be the same number
//!
//! For a model with two solids `header % (solids.len() + 1)` and `header % 3`
//! are the same expression, so only three of the nine (split, rule) pairs could
//! be drawn and **a two-sided union or difference could not be drawn at all**.
//! §95.4 measured that rather than arguing it: over the corpus as it stood, the
//! 678 fuses landed on 23 distinct `(sidedness, rule, outcome)` triples and not
//! one of them was a two-sided union or a two-sided difference.
//!
//! Taking the rule off the high nibble instead — `(header >> 4) % 3` — costs no
//! seed its model, because the byte the input is cut at does not move. It is 28
//! triples now, 9 gained and 4 lost, and `two_sided_unions` and
//! `two_sided_differences` below are the two counters that were flatly zero
//! before it. §95.4 has the whole table, including why a second header byte was
//! declined: it moves the payload, and 678 fuses become 336 or 80.
//!
//! **It buys coverage and that is measured, not argued.** The same 1751 files
//! through the `cargo-fuzz` binary reach 14 788 to 14 793 edges under the new
//! layout against 14 765 to 14 766 under the old — three runs and two, because
//! the count is not deterministic, and the ranges do not overlap.
//!
//! **`one_sided` did not move, and that is the point of keeping it.** One-sided
//! draws come from the split's own modulus having two degenerate outcomes of
//! three, which is a separate matter from the rule and is left alone; §95.4 says
//! why.
//!
//! # The four control seeds
//!
//! Every one is `[header] ++ bytes ++ bytes`, so both halves decode to the same
//! model and the second header is the first one rotated by four.
//!
//! | seed | header | what it draws |
//! |---|---|---|
//! | `two_cubes_that_fuse` | `0x01` | two cubes of side 2 overlapping at `(1, 1, 1)`, unioned then intersected, both `Ok` |
//! | `two_cubes_touching_along_one_edge` | `0x02` | two cubes meeting along one edge at `(2, 2, 0)`, refused `ResultIsNotManifold { count: 4 }` |
//! | `two_cubes_one_taken_out_of_the_other` | `0x22` | the overlapping cubes again, a **two-sided difference** both halves, both `Ok` |
//! | `a_cube_with_a_unit_ball_nearly_inside_it` | `0xa0` | a cube and a ball nearly inside it, a **two-sided intersection** then a union, both `Ok` |
//!
//! One either side of the fuse, named, so the floors below cannot be met by a
//! corpus that has quietly stopped doing one of them. The last two are the
//! rule's move: the difference seed holds up the thinnest floor here, and the
//! curved-face seed was the **one branch the move would otherwise have
//! dropped** — exactly one half in the corpus reached
//! `CurvedFaceWithAHoleGoesTooFarRound`, at header `0xf4`, and under the new
//! layout that half draws a union instead. `0xa0` draws the same tolerant
//! two-sided intersection on the same model, so the branch was kept by name
//! rather than by luck.
//!
//! **That branch is no longer what the seed reaches, and its name is now
//! wrong.** The ball's skin inside the cube is one face of two wires, a circle
//! round its top pole and an island at its equator. Since `951c28a7` refused a
//! wire that winds beside one that does not in `cut_up`, the first half was
//! refused `AWireWillNotLieFlatOnItsFace` there; the pin counted refusals and
//! not their names, so nothing said so. Since Q920 that face is cut up to its
//! pole with the island in it (`docs/decisions.md` §95.4), and the first half
//! answers. One half of the corpus, `29bf58fc…`'s first, still reaches
//! `CurvedFaceWithAHoleGoesTooFarRound`, by luck again.
//!
//! # Where the floors below come from
//!
//! Measured on this tree, 2026-09-13, by
//!
//! ```text
//! just t -p rcad-topo \
//!     --test every_model_the_fuse_fuzzer_kept_is_fused_again --no-capture
//! ```
//!
//! which reported, with all four control seeds in:
//!
//! ```text
//! Reached { halves: 3502, decoded: 926, validated: 682, fused: 682,
//!           one_sided: 569, two_sided_unions: 69, two_sided_differences: 7,
//!           ok: 265, refused: 417, solids_made: 266,
//!           ok_that_wrote_something: 249 }
//! ```
//!
//! in 0.82 s, and each floor is set at roughly three quarters of that.
//!
//! **The refusal floor was set again on 2026-10-04**, when `bake.rs` grew a
//! face's box by what writing a corner down costs and refusals fell from 310 to
//! 262 (`docs/decisions.md` §340.10). That tree read `ok: 428, refused: 262,
//! solids_made: 447, ok_that_wrote_something: 411, asked_both_ways_round: 108,
//! the_same_both_ways_round: 40`, so the floor is three quarters of 262, and
//! every other floor is still at most three quarters of its own count.
//!
//! **The refusal floor was set again on 2026-10-18**, when
//! `by_a_step_towards_a_corner_of_the_box_round_it` gave the point search one
//! more way to place a curved piece (`docs/decisions.md` §35.6, Q2116). That
//! tree read `ok: 504, refused: 187, solids_made: 521,
//! ok_that_wrote_something: 484, asked_both_ways_round: 109,
//! the_same_both_ways_round: 63`, so the floor is three quarters of 187, and
//! every other floor is still at most three quarters of its own count.
//!
//! # The last two counters are the ones that keep the checks honest
//!
//! §95.6 added three correctness checks to the harness — a refusal leaves the
//! model byte-identical, an `Ok` answer passes the sweep under its own policy,
//! and every solid handed back resolves — and a check nothing reaches passes for
//! free. `solids_made: 266` is how many keys the resolve check looked at across
//! 265 answers, so it is not 265 empty ones; `ok_that_wrote_something: 249` is
//! how many `Ok` fuses moved the bytes, which is what says the byte comparison
//! on the refusal side can see a fuse's writes at all. The 16 that did not move
//! them are the empty answers.
//!
//! The three cost 0.01 s over 682 fuses — the extra `postcard` round per fuse,
//! which is the same door the seeds came through.
//!
//! Before the rule moved off the split the same run read `halves: 3498,
//! decoded: 922, validated: 678, fused: 678, one_sided: 569, ok: 260,
//! refused: 418` — so the move itself cost no half its model, and the four extra
//! halves are the two seeds added with it.
//!
//! **The seven crash seeds §92 rescued are in that number and cost nothing.**
//! They landed on `main` while this file was being written, and replaying them
//! here moves halves 3484 → 3498, fuses 673 → 678, `ok` 257 → 260 and refusals
//! 416 → 418, for no measurable time at all. Seven seeds a night found, five
//! more fuses a commit can break.
//!
//! The `one_sided` assertion is not a floor and is written the other way round:
//! it fails when the number *drops*, because a drop means the split's modulus
//! has been changed and §95.4 is out of date. It survived the rule moving off
//! the split untouched, which is the evidence that the two are separate defects.
//!
//! # Both ways round
//!
//! Every two-sided union and intersection is fused again with the sides
//! swapped and held to the first answer the way `fuses_that_must_agree.rs`'s
//! commutativity is (`tests/fuzzing/fuse.rs` says how, and why it is a copy).
//! Measured 2026-09-25 on `0c1346f9` plus this change: `asked_both_ways_round:
//! 106, the_same_both_ways_round: 19, a_solid_one_way_round_only: 3`, and no
//! disagreement. The floors are three quarters of the first two.
//!
//! **The three were one defect, not a class to count** (`docs/decisions.md`
//! §316). They were the second halves of `2e9c4138…` and `65452b22…`, header
//! `0x49`, each a certified intersection, and the *first* half of
//! `1422656e…`, header `0x94`, a tolerant union. Each holds two cubes on the
//! same six planes, one with a stored corner moved by an amount no plane can
//! see, and selection read a face through triangles fanned from its stored
//! corners. §351 fixed that: on `665d42d2` the replay read
//! `asked_both_ways_round: 108, the_same_both_ways_round: 20,
//! a_solid_one_way_round_only: 3`, and with the fix `108, 24, 0`. The harness
//! still counts a flat pair that is a solid one way round only rather than
//! failing it, as `fuses_that_must_agree.rs` does on its flat families, but
//! this replay holds that count at zero.

#![forbid(unsafe_code)]

mod fuzzing;

use core::f64::consts::PI;
use std::path::PathBuf;

use fuzzing::fuse::{
    Agreed, Reached, WhatCameBack, both_ways_round, the_answer_written_out,
    two_models_split_and_fused,
};
use rcad_geom::{Deflection, Point, Surface};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule};

/// Where the checked-in corpus lives.
fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fuzz/corpus/two_solids_fused_by_a_stranger")
}

/// Every seed checked in beside the fuzz target, replayed.
fn every_seed() -> impl Iterator<Item = Vec<u8>> {
    let listing = std::fs::read_dir(corpus()).expect("the corpus is checked in beside the target");
    listing.filter_map(|entry| {
        let path = entry.expect("a readable directory entry").path();
        path.is_file()
            .then(|| std::fs::read(&path).expect("a readable seed"))
    })
}

#[test]
fn every_seed_in_the_checked_in_corpus_is_fused_without_panicking() {
    let mut reached = Reached::default();
    for bytes in every_seed() {
        reached = reached.and(two_models_split_and_fused(&bytes));
    }

    println!("{reached:?}");
    assert!(
        reached.halves >= 2_600,
        "only {} halves are checked in, so the corpus has shrunk",
        reached.halves
    );
    assert!(
        reached.decoded >= 680,
        "only {} halves decoded into a model, so the door has moved under the \
         corpus",
        reached.decoded
    );
    assert!(
        reached.validated >= 500,
        "only {} models passed the certified sweep, so the corpus is decoding \
         models the sweep turns away",
        reached.validated
    );
    assert!(
        reached.fused >= 500,
        "only {} fuses were reached, so the models that pass the sweep now \
         hold no solid to fuse",
        reached.fused
    );
    assert!(
        reached.ok >= 190,
        "only {} fuses came back with a model, so the corpus now refuses \
         nearly everything and this replay is asserting far less than it looks \
         like it is",
        reached.ok
    );
    // Three quarters of the 187 refusals left once
    // `by_a_step_towards_a_corner_of_the_box_round_it` gave the point search
    // one more way to place a curved piece (`docs/decisions.md` §35.6, Q2116).
    assert!(
        reached.refused >= 140,
        "only {} fuses refused, so the refusal paths are no longer reached",
        reached.refused
    );
    assert!(
        reached.solids_made >= 190,
        "only {} solids came back across every answer, so the check that every \
         key handed back resolves is looking at almost nothing (§95.6)",
        reached.solids_made
    );
    assert!(
        reached.ok_that_wrote_something >= 180,
        "only {} answered fuses changed the model's bytes, so `postcard` has \
         stopped seeing what a fuse writes — and the byte comparison on every \
         refusal is passing for free (§95.6)",
        reached.ok_that_wrote_something
    );
    assert!(
        reached.two_sided_unions >= 50,
        "only {} fuses were a union with something on both sides, and there \
         were none at all until the rule moved off `header % 3` — so the split \
         and the rule are the same number again (§95.4)",
        reached.two_sided_unions
    );
    assert!(
        reached.two_sided_differences >= 5,
        "only {} fuses were a difference with something on both sides, and \
         there were none at all until the rule moved off `header % 3` — so the \
         split and the rule are the same number again (§95.4). Two of them are \
         `two_cubes_one_taken_out_of_the_other`, so reaching zero here means \
         that seed stopped decoding too",
        reached.two_sided_differences
    );
    assert!(
        reached.asked_both_ways_round >= 80,
        "only {} unions and intersections were asked the other way round too, so \
         the commutativity check is reaching almost nothing (§129.5 item 3)",
        reached.asked_both_ways_round
    );
    assert!(
        reached.the_same_both_ways_round >= 14,
        "only {} fuses answered both ways round and were compared, so the \
         comparison is passing for free",
        reached.the_same_both_ways_round
    );
    assert_eq!(
        reached.a_solid_one_way_round_only, 0,
        "a union or intersection of flat solids answered one way round and was \
         not a solid the other. The three the corpus held were one defect in \
         selection (§316, §351), so a fourth is a defect too"
    );
    assert!(
        reached.one_sided * 2 > reached.fused,
        "fewer than half of {} fuses had a side with nothing in it, which \
         would mean the split's own modulus has been changed — move this \
         assertion and §95.4 rather than deleting them",
        reached.fused
    );
}

/// **The both-ways-round check tells two different answers apart**, and holds
/// "not a solid" against an answer the way `fuses_that_must_agree.rs` does.
///
/// The union and the intersection of the two overlapping cubes are two real
/// answers of the same model, so handing them over as the two orders of one
/// fuse is a disagreement the check has to refuse. Without it the replay above
/// would pass whatever the two orders answered.
#[test]
fn the_check_both_ways_round_refuses_two_different_answers() {
    let bytes = std::fs::read(corpus().join("two_cubes_that_fuse")).expect("a control seed");
    let rest = &bytes[1..];
    let model: Model = postcard::from_bytes(&rest[..rest.len() / 2]).expect("two cubes decode");
    let cubes: Vec<_> = model.solids().map(|solid| solid.key()).collect();
    let answered = |rule| {
        let mut model = model.clone();
        let fused = model
            .fuse(
                &cubes[..1],
                &cubes[1..],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("two overlapping cubes fuse");
        WhatCameBack::Answered(the_answer_written_out(&model, fused.solids()))
    };
    let (united, shared) = (answered(Rule::Union), answered(Rule::Intersection));

    assert_eq!(
        both_ways_round(&united, &answered(Rule::Union), true, Rule::Union),
        Ok(Agreed::TheSameAnswer),
        "the same fuse asked twice"
    );
    assert!(
        both_ways_round(&united, &shared, true, Rule::Union).is_err(),
        "a union and an intersection of two overlapping cubes passed for the same answer"
    );
    assert_eq!(
        both_ways_round(&WhatCameBack::NotASolid, &shared, true, Rule::Intersection),
        Ok(Agreed::ASolidOneWayRoundOnly),
        "flat solids: counted, as §48.2 item 5 is on the flat families"
    );
    assert!(
        both_ways_round(&shared, &WhatCameBack::NotASolid, false, Rule::Intersection).is_err(),
        "a curved solid that is a solid one way round and not the other passed"
    );
    assert_eq!(
        both_ways_round(&WhatCameBack::Refused, &shared, false, Rule::Intersection),
        Ok(Agreed::NothingToCompare),
        "a refusal on one side is held to nothing"
    );
}

/// **The seed of a ball by a block clear of it is fused and written down.**
///
/// The smallest of the 24 crash files of 2026-09-26 and 2026-09-27, all one
/// shape: a block and a ball clear of each other. Its first half is their union
/// under a certified policy and its second half does not decode. The fuse cut
/// the ball in two along one of the block's planes and wrote a rim naming that
/// plane, and until the comparison had a shape for the rim it panicked writing
/// the answer down. Since `docs/decisions.md` §400 the fuse proves the two
/// apart and leaves the ball whole, so this seed no longer reaches the rim;
/// `a_rim_that_names_its_cutting_surface.rs` reaches it with a ball that
/// touches its block.
#[test]
fn a_ball_by_a_block_clear_of_it_is_fused_and_written_down() {
    let reached = control_seed("a_ball_cut_by_the_plane_of_a_block_clear_of_it");
    assert_eq!(reached.decoded, 1, "{reached:?}");
    assert_eq!(reached.ok, 1, "{reached:?}");
    assert_eq!(
        reached.solids_made, 2,
        "the block and the ball do not touch, so their union is two solids: {reached:?}"
    );
}

/// **The three seeds that answered one way round only answer both ways now,
/// and with the same cube.**
///
/// Each holds two cubes on the same six planes, one with a stored corner moved
/// by an amount no plane can see. `docs/decisions.md` §316 traced them to
/// selection reading a face through triangles fanned from its stored corners,
/// and §351 fixed that. Every half of the three that decodes is asked here,
/// `2e9c4138`'s first half among them, which refused both ways round before and
/// so was never counted.
#[test]
fn the_three_seeds_that_answered_one_way_round_only_answer_both_ways() {
    for (name, halves_that_decode) in [
        ("2e9c4138d49472eaa947d5b75e38557823efa072", 2),
        ("65452b22c98030899498f124502a4f41d1659162", 1),
        ("1422656e67fcad8172a7e15528cf7ed5ace87f81", 2),
    ] {
        let reached = control_seed(name);
        assert_eq!(reached.fused, halves_that_decode, "{name}: {reached:?}");
        assert_eq!(reached.ok, halves_that_decode, "{name}: {reached:?}");
        assert_eq!(
            reached.the_same_both_ways_round, halves_that_decode,
            "{name}: {reached:?}"
        );
        assert_eq!(reached.a_solid_one_way_round_only, 0, "{name}: {reached:?}");
    }
}

/// One named seed, replayed.
fn control_seed(name: &str) -> Reached {
    let bytes = std::fs::read(corpus().join(name)).expect("the control seed is checked in");
    two_models_split_and_fused(&bytes)
}

/// The four named seeds, each asserted to be the thing it was checked in as.
#[test]
fn the_named_control_seeds_still_land_either_side_of_the_fuse() {
    let reached = control_seed("two_cubes_that_fuse");
    assert_eq!(reached.halves, 2, "{reached:?}");
    assert_eq!(reached.fused, 2, "{reached:?}");
    assert_eq!(reached.ok, 2, "both halves are the same model: {reached:?}");
    assert_eq!(
        reached.solids_made, 2,
        "a union of two overlapping cubes is one solid and so is their \
         intersection, and both keys were resolved: {reached:?}"
    );
    assert_eq!(
        reached.ok_that_wrote_something, 2,
        "neither answer is empty, so both wrote into the model: {reached:?}"
    );
    assert_eq!(
        reached.one_sided, 0,
        "header 0x01 cuts between the two cubes on both halves: {reached:?}"
    );
    assert_eq!(
        reached.two_sided_unions, 1,
        "the first half unions the two cubes, which nothing could draw before \
         the rule moved off the split: {reached:?}"
    );

    assert_eq!(
        reached.the_same_both_ways_round, 2,
        "a union and an intersection of two overlapping cubes, each the same \
         solid bit for bit whichever cube goes first: {reached:?}"
    );

    let reached = control_seed("two_cubes_touching_along_one_edge");
    assert_eq!(reached.fused, 2, "{reached:?}");
    assert_eq!(
        reached.refused, 2,
        "two cubes meeting along one edge leave a result that is not a \
         manifold: {reached:?}"
    );

    let reached = control_seed("two_cubes_one_taken_out_of_the_other");
    assert_eq!(reached.fused, 2, "{reached:?}");
    assert_eq!(
        reached.two_sided_differences, 2,
        "header 0x22 is its own nibble swap, so both halves take one cube out \
         of the other: {reached:?}"
    );
    assert_eq!(reached.ok, 2, "{reached:?}");

    let reached = control_seed("a_cube_with_a_unit_ball_nearly_inside_it");
    assert_eq!(reached.fused, 2, "{reached:?}");
    assert_eq!(
        reached.one_sided, 0,
        "both halves cut between the solids: {reached:?}"
    );
    assert_eq!(
        (reached.ok, reached.refused),
        (2, 0),
        "the first half, the tolerant two-sided intersection, was refused until \
         a cap on a ball took islands (§95.4), and the second half's union \
         answered already: {reached:?}"
    );
    assert_eq!(
        reached.the_same_both_ways_round, 2,
        "both halves are two-sided, so both were asked the other way round: {reached:?}"
    );
}

/// **The curved-face seed's first half is the ball, nearly whole.**
///
/// It draws a cube `[0, 2]³` and a unit ball round `(1 + 2.4e-7, 1, 1 +
/// 2.3e-13)`, so the ball pokes through `x = 2` and `z = 2` by those two
/// amounts and touches `y = 0` and `y = 2`. Their intersection is the ball less
/// two slivers of `1.8e-13` between them, far under what the measure can see,
/// and the ball's skin in it is one face of two wires: the `z = 2` circle round
/// its top pole and the `x = 2` circle as an island at its equator. That face
/// was refused until a cap on a ball took islands (`docs/decisions.md` §95.4).
#[test]
fn the_curved_seeds_intersection_measures_as_the_ball() {
    let bytes = std::fs::read(corpus().join("a_cube_with_a_unit_ball_nearly_inside_it"))
        .expect("the control seed is checked in");
    let rest = &bytes[1..];
    let mut model: Model =
        postcard::from_bytes(&rest[..rest.len() / 2]).expect("the first half decodes");
    let solids: Vec<_> = model.solids().map(|solid| solid.key()).collect();
    let &[cube, ball] = &solids[..] else {
        panic!("a cube and a ball, not {} solids", solids.len());
    };
    let skins: Vec<_> = model
        .solid(ball)
        .expect("the seed's own key")
        .faces()
        .map(|face| face.surface())
        .collect();
    let [Surface::Sphere(skin)] = &skins[..] else {
        panic!("the second solid is a ball: {skins:?}");
    };
    assert!(
        (skin.centre() - Point::new(1.0, 1.0, 1.0)).length() < 1e-6 && skin.radius() == 1.0,
        "a unit ball nearly in the middle of the cube: {skin:?}"
    );

    let answer = model
        .fuse(
            &[cube],
            &[ball],
            Rule::Intersection,
            &Policy::tolerant(model.scale().nominal_size()),
            &mut ProvenanceLog::new(),
        )
        .expect("the seed's own draw answers")
        .solids()
        .to_vec();
    let [only] = answer[..] else {
        panic!("one solid, not {}", answer.len());
    };
    let answered = model.solid(only).expect("the fuse's own key");
    let wires_on_the_ball: Vec<usize> = answered
        .faces()
        .filter(|face| matches!(face.surface(), Surface::Sphere(_)))
        .map(|face| face.wire_count())
        .collect();
    assert_eq!(
        wires_on_the_ball,
        [2],
        "one face on the ball, a ring round its pole and an island"
    );
    let measured = answered
        .measure(
            Deflection::of(0.002).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the answer measures");
    let ball = 4.0 * PI / 3.0;
    assert!(
        (measured.volume() - ball).abs() <= measured.volume_error(),
        "{} measured, {} off at most, against {ball}",
        measured.volume(),
        measured.volume_error()
    );
}
