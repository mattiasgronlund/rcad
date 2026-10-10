//! A cube turned so its corner points into another cube, with that corner
//! placed exactly on the other cube's face.
//!
//! The block is `add_cube(10.0)`, `[0, 10]³`. The turned cube is a second
//! `add_cube(10.0)` turned 1.5 about the axis through the origin along
//! `(1, 1, 0)`, which leans all three of its edges at that corner towards
//! `+x`, and moved so the corner is at `(0, 3, 3)`, on the block's `x = 0`
//! face. Its three planes there do not meet on the face: worked out in
//! rationals they meet at `x = −1.64e−16`, a rounding outside it, so the
//! three edges cross `x = 0` a rounding from the corner and from each other,
//! and the arrangement cuts that tiny triangle out of the block's face.
//!
//! **Six of the twelve fuses here panicked** in `rings_round`
//! (`fuse/assemble.rs`) under both policies: the union both ways round and
//! the block less the turned cube (Q1287). The block's `x = 0` face, joined
//! back up round the triangle, has the triangle as a hole, and the hole's
//! stored corners, a rounding apart, wind the same way as the outline. Two
//! rings claiming to be the outline was an assert; it is now
//! `TheFuseWroteABrokenAnswer { broken: HoleWindsWithTheOuterWire }`, which is
//! what `Model::validate` would say of that face if it were written. The
//! other six were refused by name already and still are. Every test below
//! pins one fuse to its refusal.
//!
//! **Under `Policy::certified()` the refusal is a limit, but not for the
//! reason the windings suggest.** Rounding each coordinate of the tip and of
//! the three crossings down or up, 256 ways (one coordinate is exact), 78 make
//! the hole and the three triangles of the tip all wind the right way, so the
//! windings alone are a gap. But in every one of the 78 the shortest edge is
//! at most `4.7e−16`, beside a corner of the block at 10: a range of at least
//! `2.1e16` against the `1e11` a certified fuse fails on
//! (`DynamicRange::LARGEST_SUPPORTED_RATIO`). `Assembly::check_dynamic_range`
//! says why that refusal is a limit — a length over a length is the same at
//! every scale — and it is what the intersection here is refused for, and
//! since Q1614 the turned cube less the block too. The same count on Q1208's
//! shape below: 160 of 512, every one with a range of at least `1.4e17`.
//!
//! **Under `Policy::tolerant(100.0)` it is a gap.** The answer meant is the
//! block with its `x = 0` face whole, with the turned cube touching it from
//! inside; it exists, and nothing here reaches it yet. The three crossings are
//! proved apart, exactly, so no decision is left for a policy to resolve, and
//! reaching it means taking three crossings a rounding apart as one: the
//! second half of `docs/decisions.md` §417, not built. The six tolerant pins
//! below stay refused until it is.
//!
//! **Where the same turn's planes meet a rounding inside the face, every fuse
//! answers under both policies** (Q1288). With the corner at `(0, 2, 4)` they
//! meet at `x = +1.03e−16`. The three crossings still cut a triangle a
//! rounding across out of the block's face, but now it is the part of the
//! face in front of all three planes, outside the turned cube, and the
//! answer exists: the block's face whole. No point could be proved inside the
//! triangle, and it was refused `Undecidable` on "a point inside a piece of a
//! face" until `select.rs`'s `by_a_plane` took a piece with every corner on or
//! in front of a plane of the solid, and one strictly in front, as outside it:
//! two corners of the triangle are on one of the turned cube's planes, and
//! the third is in front of it. With the corner at `(0, 2, 5)`
//! they meet exactly on the face, the crossings are one point and nothing is
//! cut. [`with_its_planes_meeting_a_rounding_inside_every_fuse_answers_certified`]
//! and [`with_its_planes_meeting_exactly_on_the_face_every_fuse_answers`] pin
//! both against volumes.
//!
//! Q1208's 400 random turns split the same way. Run once over all 400 by
//! Q1288, the planes meet inside the face in 201 and outside it in 199; the
//! union of each of the 201 was refused `Undecidable` under both policies,
//! and every rule both ways round now answers, the volumes agreeing as
//! [`every_fuse_answers_and_the_volumes_agree`] asks.
//! [`random_turns_whose_planes_meet_inside_the_block_answer`] keeps the first
//! 60.
//!
//! **Where the planes meet outside the face, the triangle is inside the turned
//! cube** (Q1353, `docs/decisions.md` §417): it is the cube's tip cut off by
//! the face, each of its corners on two of the cube's planes and strictly
//! behind the third. `by_a_plane` now places a piece with every corner on or
//! behind every plane of a solid with only flat faces, all on or behind them,
//! and one corner strictly behind each, inside it. Of the 199, the union under
//! `Policy::tolerant(100.0)` with the block first answered 51 and was refused
//! in 148 before: 74 `Undecidable` on the triangle, 58 a hole winding with its
//! outline and 16 an outline winding against its face. It answered 62 after
//! that, 68 after Q1442 left out an edge only wires enclosing nothing walked,
//! and answers 104 since Q1506 left out the flat ring another plane walks
//! back. The intersection of 102 of the 104 answers too, and the two hold 2000
//! between them; the intersection of turns 13 and 314 is refused as an
//! outline winding against its face, and their union is 1000 more than the
//! turned cube less the block. None is refused `Undecidable`, and the 95 left
//! are refused by name once the answer is written with its corners rounded:
//! 63 a hole winding with its outline, 16 an outline winding against its face
//! and 16 an edge with four faces. The 34 shells enclosing nothing of before
//! Q1506 are gone. The rule is exact, so it runs under `Policy::certified()`
//! too, where the union of 16 of the 199 answered either way round before
//! Q1506. 10 answered after Q1353: each validates and holds, within `1e−6`,
//! what the turned cube moved `2⁻³⁰` further in gives, and how the tip is
//! written in those 10 was not traced. The other 6 are Q1442's, whose three
//! crossings round to one point
//! ([`six_random_turns_whose_crossings_round_to_one_corner_answer_every_fuse`]).
//! Since Q1506 the certified union answers 53 either way round, and is refused
//! 95 for the range, 35 a hole winding with its outline and 16 an edge with
//! four faces; with the block first each of the 53 validates and measures what
//! the tolerant union measures. Since Q1614 the certified turned cube less the
//! block answers the same 74 and is refused for the range in the other 125,
//! where 60 of them were refused as a shell enclosing nothing
//! ([`the_turned_cube_less_the_block_over_the_199_turns_whose_planes_meet_outside`]).
//! `docs/decisions.md` §417.3 has the counts.
//! [`random_turns_whose_planes_meet_outside_the_block_answer_or_are_refused_by_name_tolerant`]
//! keeps the first 60, 31 of which meet outside.
//!
//! With the corner a thousandth inside the face, every fuse answers, and the
//! union and the intersection hold both cubes between them:
//! [`with_the_corner_a_thousandth_inside_the_union_and_intersection_hold_both_cubes`].

use std::collections::BTreeMap;

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Decision, Policy, ProvenanceLog, side_of_a_plane_where_three_planes_meet};
use rcad_topo::{DraftStep, Error, Invalid, Model, Rule, SolidBuilder, SolidKey};

/// The corner of the turned cube, on the block's `x = 0` face.
const ON_THE_FACE: [f64; 3] = [0.0, 3.0, 3.0];

/// A corner on the face where the turned cube's three planes meet at
/// `x = +1.03e−16`, a rounding inside it.
const WHERE_ITS_PLANES_MEET_INSIDE: [f64; 3] = [0.0, 2.0, 4.0];

/// A corner on the face where the turned cube's three planes meet exactly on
/// it.
const WHERE_ITS_PLANES_MEET_ON_THE_FACE: [f64; 3] = [0.0, 2.0, 5.0];

#[test]
fn the_block_united_with_the_turned_cube_is_refused_certified() {
    assert_refused(
        Rule::Union,
        Order::BlockFirst,
        &certified(),
        Refused::AHoleWindingWithItsOutline,
    );
}

#[test]
fn the_block_united_with_the_turned_cube_is_refused_tolerant() {
    assert_refused(
        Rule::Union,
        Order::BlockFirst,
        &tolerant(),
        Refused::AHoleWindingWithItsOutline,
    );
}

#[test]
fn the_turned_cube_united_with_the_block_is_refused_certified() {
    assert_refused(
        Rule::Union,
        Order::TurnedFirst,
        &certified(),
        Refused::AHoleWindingWithItsOutline,
    );
}

#[test]
fn the_turned_cube_united_with_the_block_is_refused_tolerant() {
    assert_refused(
        Rule::Union,
        Order::TurnedFirst,
        &tolerant(),
        Refused::AHoleWindingWithItsOutline,
    );
}

#[test]
fn the_block_less_the_turned_cube_is_refused_certified() {
    assert_refused(
        Rule::Difference,
        Order::BlockFirst,
        &certified(),
        Refused::AHoleWindingWithItsOutline,
    );
}

#[test]
fn the_block_less_the_turned_cube_is_refused_tolerant() {
    assert_refused(
        Rule::Difference,
        Order::BlockFirst,
        &tolerant(),
        Refused::AHoleWindingWithItsOutline,
    );
}

/// The answer is the tip outside the face, a rounding across beside corners at
/// 10, and it is refused for the range.
///
/// **Q1614 moved it.** Until then the fuse nested the answer's shells before it
/// asked the range, and the tip, its corners rounded, measured as a shell
/// enclosing less than nothing, `VolumeIsNotPositive`, which is still what the
/// tolerant fuse is refused for (`docs/decisions.md` §87.3).
#[test]
fn the_turned_cube_less_the_block_is_refused_certified() {
    assert_refused(
        Rule::Difference,
        Order::TurnedFirst,
        &certified(),
        Refused::TooWideARange,
    );
}

#[test]
fn the_turned_cube_less_the_block_is_refused_tolerant() {
    assert_refused(
        Rule::Difference,
        Order::TurnedFirst,
        &tolerant(),
        Refused::NoSpaceInside,
    );
}

#[test]
fn the_block_and_the_turned_cube_intersected_are_refused_certified() {
    assert_refused(
        Rule::Intersection,
        Order::BlockFirst,
        &certified(),
        Refused::TooWideARange,
    );
}

#[test]
fn the_block_and_the_turned_cube_intersected_are_refused_tolerant() {
    assert_refused(
        Rule::Intersection,
        Order::BlockFirst,
        &tolerant(),
        Refused::AnOutlineWindingAgainstItsFace,
    );
}

#[test]
fn the_turned_cube_and_the_block_intersected_are_refused_certified() {
    assert_refused(
        Rule::Intersection,
        Order::TurnedFirst,
        &certified(),
        Refused::TooWideARange,
    );
}

#[test]
fn the_turned_cube_and_the_block_intersected_are_refused_tolerant() {
    assert_refused(
        Rule::Intersection,
        Order::TurnedFirst,
        &tolerant(),
        Refused::AnOutlineWindingAgainstItsFace,
    );
}

/// The same pair with the corner a thousandth inside the block's face, where
/// nothing is a rounding from anything else.
///
/// The faces of the union and of the intersection, taken together, are the
/// faces of the two cubes cut up, so the two volumes add up to 2000. They add
/// up to it but for the corners the fuse made, which are rounded: moving a
/// corner by `d` moves a volume by at most `√3·d` times the faces round it,
/// the faces of both answers come to the cubes' 1200, and every corner here is
/// within 17 of the origin, so allowing each corner four roundings at 16 is
/// `1200 · √3 · 4 · 16ε` — about `3e−11`, and a missing or doubled piece of
/// either answer is many orders more.
#[test]
fn with_the_corner_a_thousandth_inside_the_union_and_intersection_hold_both_cubes() {
    let inside = [1e-3, ON_THE_FACE[1], ON_THE_FACE[2]];
    let allowed = 1200.0 * 3.0_f64.sqrt() * 4.0 * 16.0 * f64::EPSILON;
    for policy in [certified(), tolerant()] {
        for order in [Order::BlockFirst, Order::TurnedFirst] {
            let asked = format!("{order:?}, {policy:?}");
            let union = volume_of_the_answer(inside, Rule::Union, order, &policy, &asked);
            let intersection =
                volume_of_the_answer(inside, Rule::Intersection, order, &policy, &asked);
            assert!(
                (union + intersection - 2000.0).abs() <= allowed,
                "{asked}: the union {union} and the intersection {intersection} hold \
                 {} between them, not 2000",
                union + intersection
            );
        }
    }
}

/// The same turn with its corner at `(0, 2, 4)`, where its planes meet a
/// rounding inside the block, under `Policy::certified()`. Before Q1288 every
/// fuse but the turned cube less the block was refused `Undecidable` on the
/// triangle the crossings cut out of the block's face.
#[test]
fn with_its_planes_meeting_a_rounding_inside_every_fuse_answers_certified() {
    every_fuse_answers_and_the_volumes_agree(
        &the_turn(),
        WHERE_ITS_PLANES_MEET_INSIDE,
        &certified(),
    );
}

/// [`with_its_planes_meeting_a_rounding_inside_every_fuse_answers_certified`]
/// under `Policy::tolerant(100.0)`.
#[test]
fn with_its_planes_meeting_a_rounding_inside_every_fuse_answers_tolerant() {
    every_fuse_answers_and_the_volumes_agree(
        &the_turn(),
        WHERE_ITS_PLANES_MEET_INSIDE,
        &tolerant(),
    );
}

/// The same turn with its corner at `(0, 2, 5)`, where its planes meet
/// exactly on the block's face, so the three crossings are one point.
#[test]
fn with_its_planes_meeting_exactly_on_the_face_every_fuse_answers() {
    for policy in [certified(), tolerant()] {
        every_fuse_answers_and_the_volumes_agree(
            &the_turn(),
            WHERE_ITS_PLANES_MEET_ON_THE_FACE,
            &policy,
        );
    }
}

/// Six of Q1208's 400 random turns whose planes meet outside the block, and
/// whose three crossings of `x = 0` all round to one `f64` triple, with the
/// tip, where the three planes meet, a rounding outside the face: turns 190,
/// 228, 273, 283, 294 and 329. Every fuse answers under both policies, and the
/// volumes agree as [`every_fuse_answers_and_the_volumes_agree`] asks.
///
/// **The union either way round and the turned cube less the block were
/// refused** `ResultIsNotManifold { count: 0 }` until queue row Q1442. Each
/// of the cube's three faces at the corner keeps a triangle of the tip, from
/// the tip to two crossings. With the crossings one corner, its side between
/// them has no length and is left out, and what is left walks from the tip to
/// that corner and back, which encloses nothing and is left out too
/// (`docs/decisions.md` §365.7). The block's face keeps no hole, since every
/// side of the hole has no length. The edge from the tip to the crossings was
/// still made, though only those left-out wires walked it, and it was an edge
/// with no face along it. It is now left out with them, and the tip with it,
/// so the answer is written as if the tip were on the face.
#[test]
fn six_random_turns_whose_crossings_round_to_one_corner_answer_every_fuse() {
    let turns = q1208s_random_turns(400);
    for number in [190, 228, 273, 283, 294, 329] {
        let (turn, at) = turns[number - 1];
        assert_eq!(
            where_the_planes_at_the_corner_meet(&turn, at),
            Decision::Neg,
            "turn {number} has its planes meet outside the block"
        );
        for policy in [certified(), tolerant()] {
            every_fuse_answers_and_the_volumes_agree(&turn, at, &policy);
        }
    }
}

/// The first 60 of Q1208's random turns, sorted by where the turned cube's
/// three planes at the corner meet, asked exactly: each whose planes meet
/// inside the block answers the union and the intersection under both
/// policies, and the two hold both cubes between them. Before Q1288 the
/// union of each was refused `Undecidable`. 29 of the 60 meet inside, which
/// is what the same planes give worked out in rationals.
#[test]
fn random_turns_whose_planes_meet_inside_the_block_answer() {
    let allowed = 1200.0 * 3.0_f64.sqrt() * 4.0 * 16.0 * f64::EPSILON;
    let mut inside = 0;
    for (index, (turn, at)) in q1208s_random_turns(60).into_iter().enumerate() {
        if where_the_planes_at_the_corner_meet(&turn, at) != Decision::Pos {
            continue;
        }
        inside += 1;
        for policy in [certified(), tolerant()] {
            let asked = format!("turn {}, {policy:?}", index + 1);
            let union =
                volume_of_a_fuse(&turn, at, Rule::Union, Order::BlockFirst, &policy, &asked);
            let intersection = volume_of_a_fuse(
                &turn,
                at,
                Rule::Intersection,
                Order::BlockFirst,
                &policy,
                &asked,
            );
            assert!(
                (union + intersection - 2000.0).abs() <= allowed,
                "{asked}: the union {union} and the intersection {intersection} hold {} \
                 between them, not 2000",
                union + intersection
            );
        }
    }
    assert_eq!(inside, 29, "how many of the 60 meet inside the block");
}

/// The same first 60 of Q1208's random turns, now each whose turned cube's
/// three planes at the corner meet outside the block, under
/// `Policy::tolerant(100.0)`: none of the 31 is refused for want of a point
/// inside a piece of a face any more (Q1353). The triangle the crossings cut
/// out of the block's face is inside the turned cube, and `select.rs`'s
/// `by_a_plane` now places it there: every corner of it is on or behind every
/// plane of the turned cube, and strictly behind one. 18 answer the union and
/// the intersection, and the two hold both cubes between them; turns 47 and 48
/// were refused `Undecidable` on the triangle before, and turns 8, 21 and 49
/// `NoSpaceInside` until Q1506's rule left out the flat ring another plane
/// walks back.
///
/// Turn 13 answers its union and is refused its intersection, as
/// `ONLY_THE_UNION_ANSWERS` says; its union was refused as a hole winding with
/// its outline until Q1506. With no intersection to hold it to, its union is
/// held to the turned cube less the block, which answers: that is one cube
/// less what they share, so the union is `2000 − (1000 −` it`)`, within the
/// rounding the other checks allow. The other 12 are refused their union by
/// name, as `REFUSED` says, once the answer is written with its corners
/// rounded; what is left to reach them is the second half of
/// `docs/decisions.md` §417.
#[test]
fn random_turns_whose_planes_meet_outside_the_block_answer_or_are_refused_by_name_tolerant() {
    const REFUSED: [(usize, Refused); 12] = [
        (1, Refused::AHoleWindingWithItsOutline),
        (5, Refused::AHoleWindingWithItsOutline),
        (9, Refused::AnEdgeWithFourFaces),
        (14, Refused::AnOutlineWindingAgainstItsFace),
        (18, Refused::AnOutlineWindingAgainstItsFace),
        (25, Refused::AHoleWindingWithItsOutline),
        (29, Refused::AHoleWindingWithItsOutline),
        (34, Refused::AHoleWindingWithItsOutline),
        (39, Refused::AnOutlineWindingAgainstItsFace),
        (51, Refused::AHoleWindingWithItsOutline),
        (55, Refused::AnEdgeWithFourFaces),
        (57, Refused::AHoleWindingWithItsOutline),
    ];
    const ONLY_THE_UNION_ANSWERS: [(usize, Refused); 1] =
        [(13, Refused::AnOutlineWindingAgainstItsFace)];
    let allowed = 1200.0 * 3.0_f64.sqrt() * 4.0 * 16.0 * f64::EPSILON;
    let mut outside = 0;
    let mut both_answer = 0;
    let mut only_the_union_answers = 0;
    for (index, (turn, at)) in q1208s_random_turns(60).into_iter().enumerate() {
        if where_the_planes_at_the_corner_meet(&turn, at) != Decision::Neg {
            continue;
        }
        outside += 1;
        let number = index + 1;
        let asked = format!("turn {number}");
        if let Some(&(_, refused)) = REFUSED.iter().find(|&&(which, _)| which == number) {
            let mut model = a_model();
            let pair = a_block_and_a_cube_turned(&mut model, &turn, at);
            refused.assert_is(
                fuse(
                    &mut model,
                    pair,
                    Rule::Union,
                    Order::BlockFirst,
                    &tolerant(),
                ),
                &asked,
            );
            continue;
        }
        let union = volume_of_a_fuse(
            &turn,
            at,
            Rule::Union,
            Order::BlockFirst,
            &tolerant(),
            &asked,
        );
        if let Some(&(_, refused)) = ONLY_THE_UNION_ANSWERS
            .iter()
            .find(|&&(which, _)| which == number)
        {
            only_the_union_answers += 1;
            let mut model = a_model();
            let pair = a_block_and_a_cube_turned(&mut model, &turn, at);
            refused.assert_is(
                fuse(
                    &mut model,
                    pair,
                    Rule::Intersection,
                    Order::BlockFirst,
                    &tolerant(),
                ),
                &format!("{asked}, the intersection"),
            );
            let turned_less_the_block = volume_of_a_fuse(
                &turn,
                at,
                Rule::Difference,
                Order::TurnedFirst,
                &tolerant(),
                &asked,
            );
            let shared = 1000.0 - turned_less_the_block;
            assert!(
                (union - (2000.0 - shared)).abs() <= allowed,
                "{asked}: the union is {union}, and the turned cube less the block \
                 {turned_less_the_block} says {}",
                2000.0 - shared
            );
            continue;
        }
        both_answer += 1;
        let intersection = volume_of_a_fuse(
            &turn,
            at,
            Rule::Intersection,
            Order::BlockFirst,
            &tolerant(),
            &asked,
        );
        assert!(
            (union + intersection - 2000.0).abs() <= allowed,
            "{asked}: the union {union} and the intersection {intersection} hold {} between \
             them, not 2000",
            union + intersection
        );
    }
    assert_eq!(outside, 31, "how many of the 60 meet outside the block");
    assert_eq!(
        (both_answer, only_the_union_answers),
        (18, ONLY_THE_UNION_ANSWERS.len()),
        "how many of the 31 answer both rules, and how many only the union"
    );
}

/// The turned cube less the block, over the 199 of Q1208's 400 turns whose
/// planes meet outside the block. Under `Policy::certified()` it answers 74
/// and is refused for the range in the other 125. Under
/// `Policy::tolerant(100.0)`, where the range only warns, it answers 124 and is
/// refused 60 as a shell enclosing nothing and 15 as an outline winding against
/// its face. The answers are counted, not measured.
///
/// **Q1614 moved the certified 125.** Until then the fuse nested the answer's
/// shells before it asked the range, and 60 of them were refused as a shell
/// enclosing nothing, as many as the tolerant fuse still is
/// (`docs/decisions.md` §87.3, §417.3). The 74 answers did not move.
#[test]
fn the_turned_cube_less_the_block_over_the_199_turns_whose_planes_meet_outside() {
    let outside: Vec<_> = q1208s_random_turns(400)
        .into_iter()
        .filter(|&(turn, at)| where_the_planes_at_the_corner_meet(&turn, at) == Decision::Neg)
        .collect();
    assert_eq!(outside.len(), 199, "how many of the 400 meet outside");
    let certified_split = [
        ("one solid", 66),
        ("two solids", 6),
        ("three solids", 2),
        ("DynamicRangeTooWide", 125),
    ];
    let tolerant_split = [
        ("one solid", 66),
        ("two solids", 47),
        ("three solids", 11),
        ("OuterWireWindsAgainstTheNormal", 15),
        ("VolumeIsNotPositive", 60),
    ];
    for (policy, expected) in [
        (certified(), certified_split.as_slice()),
        (tolerant(), tolerant_split.as_slice()),
    ] {
        let mut split: BTreeMap<String, usize> = BTreeMap::new();
        for &(turn, at) in &outside {
            let mut model = a_model();
            let pair = a_block_and_a_cube_turned(&mut model, &turn, at);
            let answer = fuse(
                &mut model,
                pair,
                Rule::Difference,
                Order::TurnedFirst,
                &policy,
            );
            *split.entry(what_a_fuse_gives(answer)).or_default() += 1;
        }
        let expected: BTreeMap<String, usize> = expected
            .iter()
            .map(|&(what, count)| (what.to_owned(), count))
            .collect();
        assert_eq!(split, expected, "{policy:?}");
    }
}

fn what_a_fuse_gives(answer: Result<Vec<SolidKey>, Error>) -> String {
    match answer {
        Ok(solids) => match solids.len() {
            1 => "one solid".to_owned(),
            2 => "two solids".to_owned(),
            3 => "three solids".to_owned(),
            more => format!("{more} solids"),
        },
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { .. })) => {
            "DynamicRangeTooWide".to_owned()
        }
        Err(Error::Invalid(Invalid::VolumeIsNotPositive { .. })) => {
            "VolumeIsNotPositive".to_owned()
        }
        Err(Error::TheFuseWroteABrokenAnswer {
            broken: Invalid::OuterWireWindsAgainstTheNormal,
        }) => "OuterWireWindsAgainstTheNormal".to_owned(),
        Err(other) => format!("{other:?}"),
    }
}

/// Every rule both ways round answers one or more solids that validate, and
/// their volumes agree with each other and with the turned cube moved a
/// little further in.
///
/// The union is both cubes less what they share, so with the intersection it
/// holds 2000; either difference is one cube less what they share, `1000`
/// less the intersection. Those hold whatever the intersection is, so the
/// intersection is held to the one with the turned cube moved `2⁻³⁰` further
/// in, where nothing is a rounding from anything else. Moving a solid by `δ`
/// changes what it shares with another by at most `δ` times its shadow, and a
/// cube of side 10 casts at most `100·√3`. The rounding allowed is
/// [`with_the_corner_a_thousandth_inside_the_union_and_intersection_hold_both_cubes`]'s.
fn every_fuse_answers_and_the_volumes_agree(turn: &Isometry, at: [f64; 3], policy: &Policy) {
    let allowed = 1200.0 * 3.0_f64.sqrt() * 4.0 * 16.0 * f64::EPSILON;
    let further_in = 2.0_f64.powi(-30);
    let reference = volume_of_a_fuse(
        turn,
        [at[0] + further_in, at[1], at[2]],
        Rule::Intersection,
        Order::BlockFirst,
        policy,
        &format!("{further_in} further in, {policy:?}"),
    );
    for order in [Order::BlockFirst, Order::TurnedFirst] {
        let asked = format!("{at:?}, {order:?}, {policy:?}");
        let union = volume_of_a_fuse(turn, at, Rule::Union, order, policy, &asked);
        let intersection = volume_of_a_fuse(turn, at, Rule::Intersection, order, policy, &asked);
        let difference = volume_of_a_fuse(turn, at, Rule::Difference, order, policy, &asked);
        assert!(
            (union + intersection - 2000.0).abs() <= allowed,
            "{asked}: the union {union} and the intersection {intersection} hold {} between \
             them, not 2000",
            union + intersection
        );
        assert!(
            (difference + intersection - 1000.0).abs() <= allowed,
            "{asked}: the difference {difference} and the intersection {intersection} hold {} \
             between them, not 1000",
            difference + intersection
        );
        assert!(
            (intersection - reference).abs() <= further_in * 100.0 * 3.0_f64.sqrt() + allowed,
            "{asked}: the intersection is {intersection}, and {reference} with the turned cube \
             {further_in} further in"
        );
    }
}

/// Which side of the block's `x = 0` face the turned cube's three planes at
/// its corner meet on, asked of the planes and exact: `Pos` is inside the
/// block.
fn where_the_planes_at_the_corner_meet(turn: &Isometry, at: [f64; 3]) -> Decision {
    let mut model = a_model();
    let [_, turned] = a_block_and_a_cube_turned(&mut model, turn, at);
    let at_the_corner: Vec<[f64; 4]> = model
        .solid(turned)
        .expect("this model made the key")
        .faces()
        .map(|face| face.plane().coefficients())
        .filter(|plane| {
            (plane[0] * at[0] + plane[1] * at[1] + plane[2] * at[2] + plane[3]).abs() < 1e-9
        })
        .collect();
    let three: [[f64; 4]; 3] = at_the_corner
        .try_into()
        .expect("three faces of a cube meet at its corner");
    side_of_a_plane_where_three_planes_meet(three, [1.0, 0.0, 0.0, 0.0])
        .expect("three faces of a cube meet in a point")
}

/// Q1208's crash, the turned cube of `crash-45d4e616…` of the night of
/// 2026-10-06, read off the sample and described through `SolidBuilder`: its
/// eight corners as stored, with the one at `(2.44e−149, 5, 7.63e−5)` put at
/// `x = 0`, and its six planes through `Plane::from_coefficients`, which
/// moves four of them by an ulp. Its three planes at that corner meet at
/// `x = −6.8e−17`. Every fuse is refused just as the turned cube above is.
///
/// **Q1614 moved the certified turned cube less the block** from a shell
/// enclosing less than nothing to the range, as it moved the turned cube's
/// above.
///
/// The same cube made by turning `add_cube(10.0)` with the turn its corners
/// say, about `(0.0381, 0.0379, −1.098)` by 0.582, is not this shape: its
/// planes differ in the last bits, and none of its fuses panicked.
#[test]
fn the_cube_of_q1208s_crash_is_refused_by_name() {
    let table = [
        (
            Rule::Union,
            Order::BlockFirst,
            true,
            Refused::AHoleWindingWithItsOutline,
        ),
        (
            Rule::Union,
            Order::BlockFirst,
            false,
            Refused::AHoleWindingWithItsOutline,
        ),
        (
            Rule::Union,
            Order::TurnedFirst,
            true,
            Refused::AHoleWindingWithItsOutline,
        ),
        (
            Rule::Union,
            Order::TurnedFirst,
            false,
            Refused::AHoleWindingWithItsOutline,
        ),
        (
            Rule::Difference,
            Order::BlockFirst,
            true,
            Refused::AHoleWindingWithItsOutline,
        ),
        (
            Rule::Difference,
            Order::BlockFirst,
            false,
            Refused::AHoleWindingWithItsOutline,
        ),
        (
            Rule::Difference,
            Order::TurnedFirst,
            true,
            Refused::TooWideARange,
        ),
        (
            Rule::Difference,
            Order::TurnedFirst,
            false,
            Refused::NoSpaceInside,
        ),
        (
            Rule::Intersection,
            Order::BlockFirst,
            true,
            Refused::TooWideARange,
        ),
        (
            Rule::Intersection,
            Order::BlockFirst,
            false,
            Refused::AnOutlineWindingAgainstItsFace,
        ),
        (
            Rule::Intersection,
            Order::TurnedFirst,
            true,
            Refused::TooWideARange,
        ),
        (
            Rule::Intersection,
            Order::TurnedFirst,
            false,
            Refused::AnOutlineWindingAgainstItsFace,
        ),
    ];
    for (rule, order, is_certified, refused) in table {
        let policy = if is_certified {
            certified()
        } else {
            tolerant()
        };
        let mut model = a_model();
        let block = a_block(&mut model);
        let turned = the_cube_of_q1208s_crash(&mut model);
        let asked = format!("Q1208's cube, {rule:?}, {order:?}, {policy:?}");
        refused.assert_is(
            fuse(&mut model, [block, turned], rule, order, &policy),
            &asked,
        );
    }
}

/// The 30 of Q1208's 400 random turns whose union panicked.
///
/// Q1208's sweep drew a turn about a random axis by a random angle, kept it
/// when all three edges at the corner lean to `+x`, and put the corner at
/// `(0, y, z)` with `y` and `z` in 1 to 9. Of 400, each policy refused 370 by
/// name and answered or panicked on the rest; the 30 here panicked under both.
/// The whole sweep takes eight seconds, so only these are kept. Each is now
/// refused by name under both policies, and with the corner a thousandth
/// inside, each answers one solid that validates, so the turned cube is sound
/// and the refusal is about where its corner is.
#[test]
fn thirty_random_turns_that_panicked_are_refused_by_name() {
    const PANICKED: [usize; 30] = [
        1, 29, 34, 57, 69, 75, 76, 109, 127, 164, 169, 171, 179, 183, 189, 193, 216, 218, 222, 261,
        269, 270, 275, 286, 290, 366, 370, 378, 388, 400,
    ];
    let turns = q1208s_random_turns(400);
    for index in PANICKED {
        let (turn, at) = turns[index - 1];
        for policy in [certified(), tolerant()] {
            let asked = format!("turn {index}, {policy:?}");
            let mut model = a_model();
            let pair = a_block_and_a_cube_turned(&mut model, &turn, at);
            Refused::AHoleWindingWithItsOutline.assert_is(
                fuse(&mut model, pair, Rule::Union, Order::BlockFirst, &policy),
                &asked,
            );
        }
        let mut model = a_model();
        let pair = a_block_and_a_cube_turned(&mut model, &turn, [1e-3, at[1], at[2]]);
        let answer = fuse(
            &mut model,
            pair,
            Rule::Union,
            Order::BlockFirst,
            &tolerant(),
        )
        .unwrap_or_else(|why| panic!("turn {index}, the corner inside: {why}"));
        assert_eq!(answer.len(), 1, "turn {index}, the corner inside");
        model
            .validate(&tolerant(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("turn {index}, the corner inside: {why}"));
    }
}

/// What a fuse here is refused for.
#[derive(Clone, Copy, Debug)]
enum Refused {
    /// The block's `x = 0` face joined round the tiny triangle has the
    /// triangle as a hole winding with its outline, refused in `rings_round`
    /// before anything is written.
    AHoleWindingWithItsOutline,
    /// The intersection's cap on `x = 0` is the tiny triangle, and its stored
    /// corners wind against the face, refused by the sweep of the answer.
    AnOutlineWindingAgainstItsFace,
    /// The answer's shortest edge is a rounding long beside corners at 10.
    TooWideARange,
    /// A shell of the answer is the tip outside the face, which encloses
    /// nothing, or less than nothing, once its corners are rounded: all of the
    /// turned cube less the block, under a tolerant policy only. A certified
    /// one is refused for the range first (Q1614).
    NoSpaceInside,
    /// An edge of the answer has four faces along it, where the tip's sides
    /// meet the block's face.
    AnEdgeWithFourFaces,
}

impl Refused {
    fn assert_is(self, answer: Result<Vec<SolidKey>, Error>, asked: &str) {
        let why = match answer {
            Ok(solids) => panic!("{asked} answers {solids:?}, not {self:?}"),
            Err(why) => why,
        };
        let matched = match self {
            Self::AHoleWindingWithItsOutline => matches!(
                why,
                Error::TheFuseWroteABrokenAnswer {
                    broken: Invalid::HoleWindsWithTheOuterWire
                }
            ),
            Self::AnOutlineWindingAgainstItsFace => matches!(
                why,
                Error::TheFuseWroteABrokenAnswer {
                    broken: Invalid::OuterWireWindsAgainstTheNormal
                }
            ),
            Self::TooWideARange => {
                matches!(why, Error::Invalid(Invalid::DynamicRangeTooWide { .. }))
            }
            Self::NoSpaceInside => matches!(
                why,
                Error::Invalid(Invalid::VolumeIsNotPositive { volume }) if volume <= 0.0
            ),
            Self::AnEdgeWithFourFaces => {
                matches!(why, Error::ResultIsNotManifold { count: 4 })
            }
        };
        assert!(matched, "{asked} is refused {why:?}, not {self:?}");
    }
}

#[derive(Clone, Copy, Debug)]
enum Order {
    BlockFirst,
    TurnedFirst,
}

fn assert_refused(rule: Rule, order: Order, policy: &Policy, refused: Refused) {
    let mut model = a_model();
    let pair = the_block_and_the_turned_cube(&mut model, ON_THE_FACE);
    let asked = format!("{rule:?}, {order:?}, {policy:?}");
    refused.assert_is(fuse(&mut model, pair, rule, order, policy), &asked);
}

fn volume_of_the_answer(
    at: [f64; 3],
    rule: Rule,
    order: Order,
    policy: &Policy,
    asked: &str,
) -> f64 {
    volume_of_a_fuse(&the_turn(), at, rule, order, policy, asked)
}

fn volume_of_a_fuse(
    turn: &Isometry,
    at: [f64; 3],
    rule: Rule,
    order: Order,
    policy: &Policy,
    asked: &str,
) -> f64 {
    let mut model = a_model();
    let pair = a_block_and_a_cube_turned(&mut model, turn, at);
    let answer = fuse(&mut model, pair, rule, order, policy)
        .unwrap_or_else(|why| panic!("{asked}, {rule:?}: {why}"));
    model
        .validate(policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{asked}, {rule:?}: {why}"));
    answer
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.1).expect("a deflection"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{asked}, {rule:?} does not measure: {why}"))
                .volume()
        })
        .sum()
}

fn fuse(
    model: &mut Model,
    [block, turned]: [SolidKey; 2],
    rule: Rule,
    order: Order,
    policy: &Policy,
) -> Result<Vec<SolidKey>, Error> {
    let (one, other) = match order {
        Order::BlockFirst => (block, turned),
        Order::TurnedFirst => (turned, block),
    };
    model
        .fuse(&[one], &[other], rule, policy, &mut ProvenanceLog::new())
        .map(|answer| answer.solids().to_vec())
}

fn certified() -> Policy {
    Policy::certified()
}

fn tolerant() -> Policy {
    Policy::tolerant(100.0)
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a scale"))
}

fn a_block(model: &mut Model) -> SolidKey {
    model
        .add_cube(10.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube")
}

/// The block, and the cube turned 1.5 about `(1, 1, 0)` with its corner at `at`.
fn the_block_and_the_turned_cube(model: &mut Model, at: [f64; 3]) -> [SolidKey; 2] {
    a_block_and_a_cube_turned(model, &the_turn(), at)
}

/// A turn by 1.5 about the axis through the origin along `(1, 1, 0)`.
fn the_turn() -> Isometry {
    let axis = Axis::new(Point::origin(), Vector::new(1.0, 1.0, 0.0)).expect("an axis");
    Isometry::rotation(&axis, 1.5).expect("a turn")
}

fn a_block_and_a_cube_turned(model: &mut Model, turn: &Isometry, at: [f64; 3]) -> [SolidKey; 2] {
    let block = a_block(model);
    let turned = a_block(model);
    let shift = Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a shift");
    model
        .place_solid(turned, &shift.compose(turn))
        .expect("a placement");
    [block, turned]
}

/// Q1208's first `count` turns, each with where it puts the corner: drawn
/// from a seeded generator, keeping each turn that leans all three edges at
/// the corner towards `+x` by more than 0.05.
fn q1208s_random_turns(count: usize) -> Vec<(Isometry, [f64; 3])> {
    let mut seed: u64 = 1208;
    let mut next = || {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        #[expect(clippy::cast_precision_loss, reason = "53 bits fit an f64 exactly")]
        let drawn = (seed >> 11) as f64 / (1_u64 << 53) as f64;
        drawn
    };
    let mut turns = Vec::new();
    while turns.len() < count {
        let axis = Vector::new(next() - 0.5, next() - 0.5, next() - 0.5);
        let angle = next() * 3.0;
        let turn = Isometry::rotation(&Axis::new(Point::origin(), axis).expect("an axis"), angle)
            .expect("a turn");
        let leans_to_plus_x = [
            Vector::new(1.0, 0.0, 0.0),
            Vector::new(0.0, 1.0, 0.0),
            Vector::new(0.0, 0.0, 1.0),
        ]
        .into_iter()
        .all(|edge| turn.apply_to_vector(edge).components()[0] > 0.05);
        if leans_to_plus_x {
            let at = [0.0, 1.0 + 8.0 * next(), 1.0 + 8.0 * next()];
            turns.push((turn, at));
        }
    }
    turns
}

/// The corners of the turned cube in Q1208's sample, as stored.
const CORNERS_OF_Q1208S_CUBE: [[f64; 3]; 8] = [
    [0.0, 5.0, 7.629_394_531_25e-5],
    [
        0.132_229_525_990_410_93,
        4.752_663_665_461_631,
        9.996_142_524_721_972,
    ],
    [
        5.622_430_786_809_978_5,
        13.109_678_230_140_135,
        10.130_297_962_355_264,
    ],
    [
        5.490_201_260_819_568,
        13.357_014_564_678_504,
        0.134_231_731_578_605_46,
    ],
    [
        8.357_045_259_429_41,
        -0.486_267_611_351_861_36,
        -0.246_220_739_085_831_75,
    ],
    [
        13.847_246_520_248_977,
        7.870_746_953_326_642,
        -0.112_065_301_452_538_79,
    ],
    [
        13.979_476_046_239_39,
        7.623_410_618_788_274,
        9.884_000_929_324_118,
    ],
    [
        8.489_274_785_419_823,
        -0.733_603_945_890_230_6,
        9.749_845_491_690_827,
    ],
];

/// Its faces: the outward normal and offset the sample stores, and the
/// corners round it, walked the way the sample walks them.
const FACES_OF_Q1208S_CUBE: [([f64; 3], f64, [usize; 4]); 6] = [
    (
        [
            -0.835_704_525_942_941,
            0.548_626_761_135_186_2,
            0.024_629_703_303_114_425,
        ],
        2.743_135_684_773_167_6,
        [0, 1, 2, 3],
    ),
    (
        [
            0.835_704_525_942_941,
            -0.548_626_761_135_186_2,
            -0.024_629_703_303_114_425,
        ],
        7.256_864_315_226_832,
        [4, 5, 6, 7],
    ),
    (
        [
            -0.013_222_952_599_041_16,
            0.024_733_633_453_837_056,
            -0.999_606_623_077_665_8,
        ],
        0.123_591_903_336_150_18,
        [0, 3, 5, 4],
    ),
    (
        [
            0.549_020_126_081_956_7,
            0.835_701_456_467_850_4,
            0.013_415_543_763_329_263,
        ],
        14.178_508_305_864_014,
        [3, 2, 6, 5],
    ),
    (
        [
            0.013_222_952_599_041_16,
            -0.024_733_633_453_837_056,
            0.999_606_623_077_665_8,
        ],
        9.876_408_096_663_85,
        [2, 1, 7, 6],
    ),
    (
        [
            -0.549_020_126_081_956_7,
            -0.835_701_456_467_850_4,
            -0.013_415_543_763_329_263,
        ],
        -4.178_508_305_864_014,
        [1, 0, 4, 7],
    ),
];

fn the_cube_of_q1208s_crash(model: &mut Model) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let corners: Vec<_> = CORNERS_OF_Q1208S_CUBE
        .iter()
        .map(|&[x, y, z]| {
            builder
                .add_vertex(Point::new(x, y, z))
                .expect("a finite corner")
        })
        .collect();
    let sides = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (3, 5),
        (2, 6),
        (1, 7),
    ];
    let edges: Vec<_> = sides
        .iter()
        .map(|&(from, to)| {
            builder
                .add_edge(corners[from], corners[to], EdgeRun::Straight)
                .expect("an edge with length")
        })
        .collect();
    let mut faces = Vec::new();
    for (normal, offset, ring) in FACES_OF_Q1208S_CUBE {
        let steps: Vec<DraftStep> = (0..4)
            .map(|step| {
                let (from, to) = (ring[step], ring[(step + 1) % 4]);
                let (edge, way) = match sides.iter().position(|&side| side == (from, to)) {
                    Some(edge) => (edge, Orientation::Forward),
                    None => (
                        sides
                            .iter()
                            .position(|&side| side == (to, from))
                            .expect("every side of a face is an edge"),
                        Orientation::Reversed,
                    ),
                };
                DraftStep::Along(
                    builder
                        .add_coedge(edges[edge], way)
                        .expect("an edge of this builder"),
                )
            })
            .collect();
        let wire = builder.add_wire(&steps).expect("a ring of four");
        let plane =
            Plane::from_coefficients([normal[0], normal[1], normal[2], -offset]).expect("a plane");
        let face = builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a face");
        faces.push((face, Orientation::Forward));
    }
    builder.add_shell(&faces).expect("one shell");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("Q1208's cube is a sound solid")
}
