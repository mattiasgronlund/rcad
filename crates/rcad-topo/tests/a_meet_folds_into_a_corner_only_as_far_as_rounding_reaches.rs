//! A meet of three surfaces is folded into a corner of the face only when its
//! box is no further from the corner than writing the corner down cost (§253),
//! and a meet of three planes is asked that with its box narrowed by one
//! checked step (§281.2, §345).
//!
//! `place_the_meets` used to fold a meet into any corner on both of its
//! crossings whose box it touched, without asking the policy, however wide the
//! meet's box was. Across every test in the workspace, one fold's box reached
//! further than the allowance `bake.rs` grants a stored corner: a draw in the
//! fuse target's corpus, a tolerant union of two blocks that do not meet. Two
//! of the meet's three planes are about `1.5e-6` off parallel, so Cramer's box
//! round it is `2.9e-10` wide, while the point it holds is `1.1e-33` from the
//! corner (§281.1, worked in exact rationals). The old fold gave two solids,
//! and that answer was right. From §253 until §345 the meet was kept as a node
//! and the fuse refused on "order along a crossing": a right answer refused,
//! not a wrong one prevented.
//!
//! **The fold now asks whether the narrowed box lies inside the corner's
//! room**, and the narrowed box is `8.9e-16` wide and holds the meet. So the
//! meet folds for the reason §253 gives, every point of its box within the
//! rounding of the corner, and not because a box was let past the allowance.
//! The node the fuse keeps when it does not fold still has Cramer's box.
//!
//! The draw is a copy of `fuzz/corpus/two_solids_fused_by_a_stranger/a45aa71b…`
//! beside this file, so that minimising the corpus cannot drop it. It is named
//! rather than rebuilt here because it is four and a half kilobytes of `Model`
//! and reads as nothing, the reason
//! `a_fused_draw_hands_its_provenance_back_through_the_door.rs` gives for the
//! same choice.
//!
//! **Since `docs/decisions.md` §400 this draw no longer reaches the fold.** The
//! fuse now cuts two solids against each other only where it cannot prove they
//! do not meet, and it proves these two blocks apart, so neither is cut by the
//! other's planes and the meet is never asked about. The answer below is the
//! same two blocks; what holds the fold now is
//! `a_meet_is_folded_when_its_tighter_box_lies_inside_the_corners_room_however_far_cramers_reaches`
//! in `src/fuse/arrange.rs`'s `tests` module, which asks `a_corner_is_this_place`
//! directly.
//!
//! **§400.4 asks for a second draw that does reach the fold through a real
//! fuse**, and `two_blocks_far_from_the_origin_meeting_at_a_rounded_corner`,
//! a new entry in the checked-in corpus, is one: two solids a long way from
//! the origin, split and united under a tolerant policy, where three of the
//! planes `place_the_meets` asks about are a few ulps off parallel. Cramer's
//! box reaches `974228228964.1372` wide along one axis, far past the room
//! round the corner there, which is about `67108864` wide on every axis; the
//! box narrowed by the checked step (§281.2, §345) is `0.00030517578125` wide
//! along that axis and lies inside the room. So the meet folds for the
//! reason §253 gives and not because a wide box was let past the allowance —
//! read with a temporary log in `a_corner_is_this_place`, taken back out
//! (§345.3's method). The fold then leaves two regions of one face the fuse
//! cannot tell apart, so the union refuses by name rather than silently
//! answering wrong.

#![forbid(unsafe_code)]

mod fuzzing;

use std::path::PathBuf;

use fuzzing::fuse::two_models_split_and_fused_noting_refusals;
use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

const THE_DRAW: &[u8] =
    include_bytes!("a_meet_folds_into_a_corner_only_as_far_as_rounding_reaches.draw");

#[test]
fn a_tolerant_union_of_two_blocks_apart_answers_with_the_two_of_them() {
    let mut refusals = Vec::new();
    let reached = two_models_split_and_fused_noting_refusals(THE_DRAW, &mut |why, _| {
        refusals.push(why.to_string());
    });
    assert_eq!(
        (reached.fused, reached.one_sided),
        (1, 1),
        "the draw no longer reaches the one fuse it is kept for: {reached:?}"
    );
    assert_eq!(
        (reached.ok, reached.refused, reached.solids_made),
        (1, 0, 2),
        "two blocks that do not meet unite as the two of them: {reached:?}, refused with \
         {refusals:?}"
    );
}

/// The same fuse, asked here rather than through the fuzz harness, so that the
/// answer can be read: two solids whose volumes add up to the two blocks', and
/// a sweep of the model under `Policy::certified()` that passes, though the
/// fuse ran under a tolerant one.
#[test]
fn the_two_blocks_unite_with_their_volumes_and_pass_a_certified_sweep() {
    let (mut model, policy, arguments, tools, rule) = the_fuse_the_draw_asks_for();
    assert_eq!(rule, Rule::Union, "the draw is kept for a union");
    assert!(
        policy.max_wobble() > 0.0,
        "the draw is kept for a tolerant fuse"
    );
    let blocks: Vec<SolidKey> = arguments.iter().chain(&tools).copied().collect();
    assert_eq!(blocks.len(), 2, "the draw unites two blocks");
    let before = volume_of(&model, &blocks);

    let fused = model
        .fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("two blocks that do not meet were refused: {why}"));
    assert_eq!(
        fused.solids().len(),
        2,
        "two blocks that do not meet unite as the two of them"
    );
    let after = volume_of(&model, fused.solids());
    assert!(
        (after - before).abs() <= 1e-9 * before,
        "two blocks of {before} in all united to {after}"
    );
    let mut swept = ProvenanceLog::new();
    model
        .validate(&Policy::certified(), &mut swept)
        .unwrap_or_else(|why| panic!("the union fails a certified sweep: {why}"));
    assert!(swept.is_empty(), "a certified sweep recorded {swept:?}");
}

/// §400.4: a draw where two solids genuinely meet, so the fuse's own
/// arrangement reaches `place_the_meets` and folds a meet whose narrowed box
/// lies inside the corner's room while Cramer's box alone reaches past it.
///
/// The module doc comment has the two boxes and the room, measured with a
/// temporary log. What is left to check here, without that log, is that the
/// draw still reaches one fuse and that the fuse still refuses the one way:
/// two regions of one face the fold brought together that the policy cannot
/// tell apart.
#[test]
fn a_meet_folds_by_its_narrowed_box_though_cramers_misses_the_corners_room() {
    let bytes = std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../fuzz/corpus/two_solids_fused_by_a_stranger/\
             two_blocks_far_from_the_origin_meeting_at_a_rounded_corner",
    ))
    .expect("the draw pinned for §400.4 is checked in beside the corpus");
    let mut refusals = Vec::new();
    let reached = two_models_split_and_fused_noting_refusals(&bytes, &mut |why, _| {
        refusals.push(why.to_string());
    });
    assert_eq!(
        (reached.fused, reached.refused),
        (1, 1),
        "the draw no longer reaches the one fuse it is kept for: {reached:?}, refused with \
         {refusals:?}"
    );
    assert!(
        !refusals.is_empty()
            && refusals
                .iter()
                .all(|why| why == "two regions of one face could not be told apart"),
        "the fold should bring together two regions of one face the policy cannot tell apart, \
         and nothing else: {refusals:?}"
    );
}

/// The model, policy, two sides and rule that
/// `fuzzing::fuse::two_models_split_and_fused` reads out of [`THE_DRAW`] for
/// the one half that decodes and validates, read the same way.
fn the_fuse_the_draw_asks_for() -> (Model, Policy, Vec<SolidKey>, Vec<SolidKey>, Rule) {
    let (&header, rest) = THE_DRAW.split_first().expect("the draw is not empty");
    let (first_half, second_half) = rest.split_at(rest.len() / 2);
    let mut fusing = [(first_half, header), (second_half, header.rotate_left(4))]
        .into_iter()
        .filter_map(|(bytes, header)| {
            let model = postcard::from_bytes::<Model>(bytes).ok()?;
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .ok()?;
            Some((model, header))
        });
    let (model, header) = fusing
        .next()
        .expect("one half of the draw is a valid model");
    assert!(fusing.next().is_none(), "only one half of the draw fuses");

    let solids: Vec<SolidKey> = model.solids().map(|solid| solid.key()).collect();
    let (arguments, tools) = solids.split_at(usize::from(header) % (solids.len() + 1));
    let rule = match (header >> 4) % 3 {
        0 => Rule::Union,
        1 => Rule::Intersection,
        _ => Rule::Difference,
    };
    let policy = if header & 0x80 == 0 {
        Policy::certified()
    } else {
        Policy::tolerant(model.scale().nominal_size())
    };
    (model, policy, arguments.to_vec(), tools.to_vec(), rule)
}

fn volume_of(model: &Model, solids: &[SolidKey]) -> f64 {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.1).expect("a tenth of a millimetre is a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a flat solid measures")
                .volume()
        })
        .sum()
}
