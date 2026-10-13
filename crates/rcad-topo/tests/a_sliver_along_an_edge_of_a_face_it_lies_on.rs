//! Four pairs from `how_often_a_certified_fuse_of_a_skew_pair_refuses` that
//! each stopped on a sliver a rounding across (`docs/decisions.md` §365.8).
//! Before Q1606 every rule of all four refused `a point inside a piece of a
//! face`. In two the sliver lies on the plane of a face of the other solid,
//! along one of that face's edges, and Q1606's rule places it (queue rows
//! Q1034 and Q1606). In the other two it lies on one solid's floor, with a
//! side starting inside an upright edge of the other solid, and the rule of
//! Q1608 and Q1609 places it by that edge. Every rule of three of them answers
//! now. Of the two combs, every rule answers but the certified first comb less
//! the second, which as written keeps an edge `2^-51` long beside a coordinate
//! of 11 and is refused by that name. Their union is written without a notch a
//! rounding across that holds no `f64` point, and with the first comb's corner
//! `(3, 1, 0)` one rounding off (`docs/decisions.md` §431).

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::expansion::{compress, expansion_sum, product_is_exact, scale_expansion};
use rcad_predicates::{
    Decision, Policy, ProvenanceLog, meet_of_three_planes, side_of_a_plane_where_three_planes_meet,
};
use rcad_topo::{Error, Invalid, Model, Rule, SolidKey};

use pairs::{Configuration, Elsewhere, Placed, Shape, Skew};

/// A frame four by three, leaning back one along `x` for each step along `y`,
/// and a comb of three teeth two high standing at `(−2, −2, 0)`, both one high.
///
/// The comb's middle tooth and its right one each reach one square into the
/// frame's near wall, so the intersection is two unit cubes. The right tooth's
/// top corner `(2, 1)` is the corner of the frame's hole, where its leaning
/// wall `x + y = 3` meets its wall `y = 1`. That leaning wall is stored
/// rounded, so the comb's wall `x = 2` and it cut a sliver a rounding across
/// out of the frame's wall `y = 1`, holding no `f64` point, along the edge of
/// the tooth's top. Q1606's rule places it by that edge.
///
/// The union and the frame less the comb each touch themselves along a line,
/// with four faces along it. In the union, the comb's left tooth touches the
/// frame's leaning outer wall along `x = −1, y = 1`, an edge of the comb and
/// not of the frame. Since queue row Q1901 that is an answer
/// (`docs/decisions.md` §39.4): one solid holding the frame's 10 and the
/// comb's 11 less the 2 they share, 19. Until then it was refused
/// `ResultIsNotManifold { count: 4 }`.
///
/// In the frame less the comb, the near wall left of the right tooth and the
/// side wall above it touch along `x = 2, y = 1`. That line is an edge of both,
/// the corner of the frame's hole and of the tooth's top, so the four faces
/// along it are four faces of the two inputs: the hole's walls `y = 1` and
/// `x + y = 3`, and the tooth's top `y = 1` and its wall `x = 2`. It was
/// refused `ResultIsNotManifold { count: 4 }` until queue row Q1913. The four
/// faces are one shell without the line, so the answer is one solid touching
/// itself, `10 − 2`, and not the two touching solids §80.9 leaves unbuilt
/// (`docs/decisions.md` §39.4).
#[test]
fn a_leaning_frame_and_a_comb_meet_in_two_squares() {
    let (model, keys) = frame_and_comb().build();
    holds_exactly(&model, &keys[..1], 10.0, "the frame");
    holds_exactly(&model, &keys[1..], 11.0, "the comb");

    let (met, solids) = fused(&model, keys[0], keys[1], Rule::Intersection);
    assert_eq!(solids.len(), 2, "the intersection is two squares");
    holds_exactly(&met, &solids, 2.0, "the intersection");

    let (either, solids) = fused(&model, keys[0], keys[1], Rule::Union);
    assert_eq!(solids.len(), 1, "the union is one solid");
    holds_exactly(&either, &solids, 10.0 + 11.0 - 2.0, "the union");

    let (less, solids) = fused(&model, keys[0], keys[1], Rule::Difference);
    assert_eq!(solids.len(), 1, "the frame less the comb is one solid");
    holds_exactly(&less, &solids, 10.0 - 2.0, "the difference");
}

/// The same two with the comb first: the comb less the frame holds 9, the
/// comb's 11 less the 2 they share, and the union is one solid of 19 touching
/// itself along `x = −1, y = 1`, as with the frame first, since queue row
/// Q1901.
#[test]
fn a_comb_and_a_leaning_frame_meet_in_two_squares() {
    let (model, keys) = frame_and_comb().build();

    let (met, solids) = fused(&model, keys[1], keys[0], Rule::Intersection);
    assert_eq!(solids.len(), 2, "the intersection is two squares");
    holds_exactly(&met, &solids, 2.0, "the intersection");

    let (less, solids) = fused(&model, keys[1], keys[0], Rule::Difference);
    holds_exactly(&less, &solids, 9.0, "the difference");

    let (either, solids) = fused(&model, keys[1], keys[0], Rule::Union);
    assert_eq!(solids.len(), 1, "the union is one solid");
    holds_exactly(&either, &solids, 10.0 + 11.0 - 2.0, "the union");
}

/// A comb of two teeth three high, leaning forward one along `x` for each step
/// along `y`, one high at the origin, and an ell five by four with a bite one
/// by three, leaning back two, three high on the comb's top at `(1, 1, 1)`.
///
/// They touch only across `z = 1`, so the intersection holds nothing and each
/// less the other is itself: the comb's 9 and the ell's 51. The union is not a
/// solid: the ell's near wall `y = 1` runs along the bottom of the comb's gap
/// between its teeth, from `x = 2` to `3`, the ell above it and the comb below,
/// so four faces meet along that line.
#[test]
fn a_leaning_comb_and_a_leaning_ell_standing_on_it_share_no_space() {
    let (model, keys) = comb_and_ell().build();
    holds_exactly(&model, &keys[..1], 9.0, "the comb");
    holds_exactly(&model, &keys[1..], 51.0, "the ell");

    let (_, solids) = fused(&model, keys[0], keys[1], Rule::Intersection);
    assert_eq!(solids, [], "the intersection");

    let (less, solids) = fused(&model, keys[0], keys[1], Rule::Difference);
    holds_exactly(&less, &solids, 9.0, "the difference");

    is_not_a_solid(&model, keys[0], keys[1], Rule::Union);
}

/// The same two with the ell first: the intersection still holds nothing, the
/// ell less the comb is the ell, and the union is still not a solid.
#[test]
fn a_leaning_ell_standing_on_a_leaning_comb_shares_no_space_with_it() {
    let (model, keys) = comb_and_ell().build();

    let (_, solids) = fused(&model, keys[1], keys[0], Rule::Intersection);
    assert_eq!(solids, [], "the intersection");

    let (less, solids) = fused(&model, keys[1], keys[0], Rule::Difference);
    holds_exactly(&less, &solids, 51.0, "the difference");

    is_not_a_solid(&model, keys[1], keys[0], Rule::Union);
}

/// A frame three by three, leaning forward one, two high at the origin, and an
/// ell four by four with a bite three by one, leaning back two, one high at
/// `(3, 1, 1)`. They hold 16 and 13.
///
/// The ell's wall `x + 2y = 7` runs through the frame's hole corner `(3, 2)`,
/// where the hole's walls `y = 2` and `x − y = 1` meet. That wall is stored
/// rounded, so the three cut a triangle a rounding across out of the ell's
/// floor `z = 1`, holding no `f64` point, in the frame above the hole. Its side
/// on `y = 2` starts inside the frame's upright edge at the corner and runs
/// along the hole's wall, and Q1608's rule places it by that edge.
///
/// They share `z = 1` to `2` over two parts: a four-sided one in the frame's
/// right wall, `(3, 1)`, `(4, 1)`, `(13/3, 4/3)`, `(11/3, 5/3)`, holding a
/// half, and `(1, 1)`, `(2, 1)`, `(3, 2)`, `(7/3, 7/3)` in its left wall and
/// above the hole, holding seven sixths: five thirds in all. No two parts of
/// any answer touch along a line, so the union holds `16 + 13 − 5/3` and the
/// frame less the ell `16 − 5/3`.
#[test]
fn a_leaning_frame_and_a_leaning_ell_meet_in_two_parts() {
    let (model, keys) = frame_and_ell().build();
    holds_exactly(&model, &keys[..1], 16.0, "the frame");
    holds_exactly(&model, &keys[1..], 13.0, "the ell");

    let (met, solids) = fused(&model, keys[0], keys[1], Rule::Intersection);
    assert_eq!(solids.len(), 2, "the intersection is two parts");
    holds_exactly(&met, &solids, 5.0 / 3.0, "the intersection");

    let (either, solids) = fused(&model, keys[0], keys[1], Rule::Union);
    holds_exactly(&either, &solids, 29.0 - 5.0 / 3.0, "the union");

    let (less, solids) = fused(&model, keys[0], keys[1], Rule::Difference);
    holds_exactly(&less, &solids, 16.0 - 5.0 / 3.0, "the difference");
}

/// The same two with the ell first. The ell less the frame is three parts:
/// what lies left of the frame and behind it, what lies in the hole, and a
/// triangle `(4, 1)`, `(5, 1)`, `(13/3, 4/3)` right of it, `13 − 5/3` in all.
///
/// That one is not held to `13 − 5/3`. It measures `11.33333333333333` give
/// or take `2.4e-15`, `3.0e-15` under `13 − 5/3`, and the ell's `13` less the
/// intersection's `1.6666666666666667`, each measured exactly, is `2.9e-15`
/// more than that, worked out exactly. No piece is lost
/// ([`no_answer_of_a_leaning_frame_and_a_leaning_ell_loses_a_piece`]), and a
/// throwaway probe found every part of all six answers within its own error
/// of its own triangles (§365.8's note on Q1608 and Q1609). Two things take
/// it past its error. Its largest part measures `2.4e-15` under its own
/// triangles, nearly all the error it owns up to. And it moves the ell's
/// corner `(−1, 4)` by two roundings, so its triangles and the
/// intersection's hold `2.6e-16` less than the ell's. So the volume is pinned
/// bit for bit, so that a move is seen.
#[test]
fn a_leaning_ell_and_a_leaning_frame_meet_in_two_parts() {
    let (model, keys) = frame_and_ell().build();

    let (met, solids) = fused(&model, keys[1], keys[0], Rule::Intersection);
    assert_eq!(solids.len(), 2, "the intersection is two parts");
    holds_exactly(&met, &solids, 5.0 / 3.0, "the intersection");

    let (either, solids) = fused(&model, keys[1], keys[0], Rule::Union);
    holds_exactly(&either, &solids, 29.0 - 5.0 / 3.0, "the union");

    let (less, solids) = fused(&model, keys[1], keys[0], Rule::Difference);
    assert_eq!(solids.len(), 3, "the ell less the frame is three parts");
    let (left, _) = measured(&less, &solids);
    assert_eq!(
        left.to_bits(),
        11.333_333_333_333_33_f64.to_bits(),
        "the ell less the frame holds {left}"
    );
}

/// No answer of the frame and the ell loses a piece, however their corners
/// round. Over the triangles the answers hand out, the union is the skin of
/// each solid outside the other, the intersection the skin of each inside the
/// other, and each difference the skin of one outside the other with the
/// other's skin inside it turned round. So six times the union's volume less
/// the intersection's and both differences' is nothing, worked out exactly,
/// when all four are made of the same pieces, and a piece one of them lost
/// would show as what that piece closed off.
///
/// What the triangles hold is not what the solids drawn in whole numbers
/// hold. The ell less the frame and the union write the ell's corner
/// `(−1, 4)`, where its wall `x + 2y = 7` meets `y = 4`, at
/// `(−0.9999999999999998, 4)`, though the frame does not reach `y = 4`. Its
/// stored wall crosses `y = 4` at `−1 + 1.24e-16`, worked out exactly from
/// its stored coefficients, which rounds to `−0.9999999999999999`.
#[test]
fn no_answer_of_a_leaning_frame_and_a_leaning_ell_loses_a_piece() {
    let (model, keys) = frame_and_ell().build();
    let (frame, ell) = (keys[0], keys[1]);
    let six_times = |rule: Rule, arguments: SolidKey, tools: SolidKey| {
        let (answer, solids) = fused(&model, arguments, tools, rule);
        solids.iter().fold(Vec::new(), |total, &solid| {
            plus(&total, &six_times_the_volume(&answer, solid))
        })
    };
    let ell_less_the_frame = six_times(Rule::Difference, ell, frame);
    let frame_less_the_ell = six_times(Rule::Difference, frame, ell);
    for (first, second) in [(ell, frame), (frame, ell)] {
        let left = plus(
            &six_times(Rule::Union, first, second),
            &negated(&plus(
                &six_times(Rule::Intersection, first, second),
                &plus(&ell_less_the_frame, &frame_less_the_ell),
            )),
        );
        assert!(
            left.iter().all(|&entry| entry == 0.0),
            "the union less the rest, {first:?} first, leaves {left:?}"
        );
    }

    let (less, solids) = fused(&model, ell, frame, Rule::Difference);
    let drawn = [-1.0, 4.0, 1.0];
    let written = [-0.999_999_999_999_999_8, 4.0, 1.0];
    assert!(has_a_corner_at(&model, ell, drawn), "the ell's own corner");
    for (at, held) in [(drawn, false), (written, true)] {
        assert_eq!(
            solids
                .iter()
                .any(|&solid| has_a_corner_at(&less, solid, at)),
            held,
            "the ell less the frame and a corner at {at:?}"
        );
    }
}

/// A comb of three teeth two high, leaning forward two, two high at the
/// origin, and a comb of two teeth two high, leaning back two, one high at
/// `(3, 0, 1)`. They hold 22 and 7: the first is a base five by one and three
/// teeth one by two, eleven across, two high, and the second a base three by
/// one and two teeth one by two, seven across, one high. Leaning keeps what a
/// slice holds.
///
/// The second comb's wall `x + 2y = 6` runs through the first comb's corner
/// `(4, 1)`, where the floor `y = 1` of its gap meets its tooth's wall
/// `x − 2y = 2`. That wall is stored rounded, so the three cut a triangle a
/// rounding across out of the second comb's floor `z = 1`, holding no `f64`
/// point, in that tooth. Its side on the tooth's wall `x − 2y = 2` starts
/// inside the first comb's upright edge at the corner and runs into the tooth,
/// and Q1609's rule places it by that edge. Before Q1606 the first comb first
/// stopped at `(4, 1, 2)`.
///
/// They share `z = 1` to `2` over the bases, eleven fourths, and a four-sided
/// part where the first comb's left tooth and the second's right one cross,
/// `(3, 1)`, `(3.5, 1.25)`, `(3, 1.5)`, `(2.5, 1.25)`, a fourth: three in all.
/// Drawn in whole numbers that part touches the bases along `x = 3, y = 1`.
/// Stored, it does not: the second comb's wall `x + 2y = 5` crosses `y = 1`
/// at `3 + 3.7e-16` and meets the first comb's wall `x − 2y = 1` at
/// `9.3e-17` above `y = 1`, worked out exactly from the stored coefficients,
/// so the part stands that far off the bases and the intersection is two
/// parts.
///
/// Until Q1608 and Q1609 the intersection was refused with one face along an
/// edge. The strip of the first comb's gap floor `y = 1` from `x = 3` to
/// `3 + 3.7e-16` lies on the second comb's gap floor, and §365.3's rule read
/// its side on `x + 2y = 5` against a piece of that wall cut along the
/// upright line through the meet above, whose ends were named the same
/// corners, and put the strip outside. Now that rule does not place it, and
/// Q1606's rule places it by an edge of the second comb's gap floor, on that
/// floor and facing its way, in all three fuses.
///
/// Stored, the union and the first less the second each have a part a
/// rounding across at `(3, 1)` from `z = 1` to `2`: the union a three-sided
/// notch, which neither comb fills, between `(3, 1)`, the crossing of
/// `x + 2y = 5` with `y = 1`, and the meet of `x + 2y = 5` with `x − 2y = 1`,
/// and the difference the side of the first comb's tooth from `(3, 1)` to that
/// meet. Those are three points within `4e-16` of `(3, 1)`, and no `f64` point
/// lies inside the notch
/// ([`where_the_two_combs_cross_three_corners_lie_a_rounding_apart`]).
///
/// **The union is written without the notch, one solid of 26 faces holding
/// `22 + 7 − 3`**, under each policy and with the second comb first too. The
/// boxes of the crossing and of the meet have the same middle,
/// `(3.0000000000000004, 1)`, so `Corners::of` in `fuse/assemble.rs` makes
/// them one corner. The notch's wall on `x + 2y = 5` then has two sides of no
/// length and walks out and back along one edge, and is left out
/// (`docs/decisions.md` §365.7). Its two other walls, the strip of the floor
/// `y = 1` and the strip of the tooth's wall `x − 2y = 1`, walk one ring of the
/// same four edges, each the other way round, and enclose nothing as they are
/// written. The plane of the second comb's wall also cuts the floor and the
/// tooth's wall from `z = 0` to `1`, below that wall, into two more such
/// strips. Until Q1528 four faces met along an edge of each ring and the union
/// was refused `ResultIsNotManifold { count: 4 }`; now each pair is left out
/// together (§431), and the first comb's corner `(3, 1, 0)` is written where
/// the crossing and the meet are, one rounding off
/// ([`the_union_leaves_out_a_notch_whose_three_corners_round_to_two_points`]).
///
/// **The first less the second is a limit, refused by the limit's name**
/// (queue rows Q1634 and Q1658). It would hold `22 − 3`. Exactly, its side of
/// the tooth from `(3, 1)` to the meet is the top of the tooth's wall, which
/// runs on below `z = 1`. As written, the strip of that wall below `z = 1` is
/// left out with its strip of the floor, as in the union, so from `z = 1` to
/// `2` the side is a face of its own. The floor strip beside it is the top of
/// the second comb's base and not in the difference, so no face walks the
/// side's ring back and it stays. Its edges at `z = 1` and `2` are written
/// `2^-51` long beside the first comb's `11` across. That is a length over a
/// length, so no scaling fits the two together (§3.5): refused
/// [`Invalid::DynamicRangeTooWide`] at `11 · 2^51`, under a certified policy.
/// A tolerant one writes those edges and answers 25 faces holding `22 − 3`.
#[test]
fn two_combs_leaning_opposite_ways_meet_in_two_parts_a_rounding_apart() {
    let (model, keys) = comb_and_comb().build();
    holds_exactly(&model, &keys[..1], 22.0, "the first comb");
    holds_exactly(&model, &keys[1..], 7.0, "the second comb");

    let (met, solids) = fused(&model, keys[0], keys[1], Rule::Intersection);
    assert_eq!(solids.len(), 2, "the intersection is two parts");
    assert_eq!(
        face_counts(&met, &solids),
        [8, 6],
        "the intersection's faces"
    );
    holds_exactly(&met, &solids, 3.0, "the intersection");

    for (either, solids) in [
        fused(&model, keys[0], keys[1], Rule::Union),
        fused_tolerantly(&model, keys[0], keys[1], Rule::Union),
    ] {
        assert_eq!(face_counts(&either, &solids), [26], "the union's faces");
        holds_exactly(&either, &solids, 22.0 + 7.0 - 3.0, "the union");
    }

    let mut less = model.clone();
    let answer = less
        .fuse(
            &[keys[0]],
            &[keys[1]],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|fused| fused.solids().to_vec());
    assert!(
        matches!(
            answer,
            Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) if ratio == 11.0 * 2f64.powi(51)
        ),
        "the certified difference gave {answer:?}"
    );
    let (less, solids) = fused_tolerantly(&model, keys[0], keys[1], Rule::Difference);
    assert_eq!(
        face_counts(&less, &solids),
        [25],
        "the tolerant difference's faces"
    );
    holds_exactly(&less, &solids, 22.0 - 3.0, "the tolerant difference");
}

/// The same two with the second comb first. The intersection is the same two
/// parts. The second less the first is four parts, `7 − 3` in all: its left
/// tooth with the end of its base under it, the other end of its base, the top
/// of its right tooth above where the first comb's tooth crosses it, and the
/// triangle `(3, 1)`, `(4, 1)`, `(3.5, 1.25)` of that tooth in the first
/// comb's gap.
///
/// Until Q1633 the intersection was refused with four faces along the upright
/// line through `(3 + 3.7e-16, 1)`, where the two parts touch: on `y = 1`, the
/// second comb's gap floor, which reaches that far past `x = 3`, and the first
/// comb's gap floor beyond it, and resting on them the walls `x + 2y = 5` and
/// `x − 2y = 1` of the four-sided part. Two flat faces on one plane facing one
/// way, with the other two on its outer side, are two parts touching along a
/// line, and the line is now parted into one edge per part, as it already was
/// where the two faces on the plane were pieces of one input face.
///
/// The union is the one written with the first comb first, one solid of 26
/// faces without the notch a rounding across, under each policy. Until Q1528
/// it was refused with four faces along an edge, as that one was.
#[test]
fn two_combs_leaning_opposite_ways_the_other_way_round_meet_in_two_parts_a_rounding_apart() {
    let (model, keys) = comb_and_comb().build();

    let (met, solids) = fused(&model, keys[1], keys[0], Rule::Intersection);
    assert_eq!(solids.len(), 2, "the intersection is two parts");
    assert_eq!(
        face_counts(&met, &solids),
        [8, 6],
        "the intersection's faces"
    );
    holds_exactly(&met, &solids, 3.0, "the intersection");

    for (either, solids) in [
        fused(&model, keys[1], keys[0], Rule::Union),
        fused_tolerantly(&model, keys[1], keys[0], Rule::Union),
    ] {
        assert_eq!(face_counts(&either, &solids), [26], "the union's faces");
        holds_exactly(&either, &solids, 22.0 + 7.0 - 3.0, "the union");
    }

    let (less, solids) = fused(&model, keys[1], keys[0], Rule::Difference);
    assert_eq!(
        solids.len(),
        4,
        "the second comb less the first is four parts"
    );
    assert_eq!(
        face_counts(&less, &solids),
        [6, 6, 5, 5],
        "the difference's faces"
    );
    holds_exactly(&less, &solids, 7.0 - 3.0, "the difference");
}

/// **Where the two combs cross `(3, 1)`, three corners lie a rounding apart.**
/// From `z = 1` to `2`, the planes of three faces through that line meet two at
/// a time in three upright lines, asked exactly of the planes the model
/// stores. The first comb's corner, where its gap floor `y = 1` meets its
/// tooth's wall `x − 2y = 1`, is `(3, 1)`. The crossing of the second comb's
/// wall `x + 2y = 5` with its gap floor, the same plane `y = 1`, is strictly
/// in front of the tooth's wall, outside the first comb. The meet of the two
/// walls is strictly in front of the floor, inside both gaps. So the three are
/// three points, and the notch between them, and the side of the tooth from
/// the corner to the meet, are a rounding across: the edges the exact union
/// and first comb less the second have there. The difference keeps the side
/// and is refused for it, the limit the first combs test pins. The union is
/// written without the notch
/// ([`the_union_leaves_out_a_notch_whose_three_corners_round_to_two_points`]).
#[test]
fn where_the_two_combs_cross_three_corners_lie_a_rounding_apart() {
    let (model, keys) = comb_and_comb().build();
    let [tooth_wall, floor, wall, bottom] = the_planes_through_three_one(&model, &keys);

    let crossing = [bottom, floor, wall];
    let meet = [bottom, wall, tooth_wall];
    assert_eq!(
        side_of_a_plane_where_three_planes_meet(crossing, tooth_wall),
        Ok(Decision::Pos),
        "the crossing against the tooth's wall"
    );
    assert_eq!(
        side_of_a_plane_where_three_planes_meet(meet, floor),
        Ok(Decision::Pos),
        "the meet against the floor"
    );
    for [one, other, last] in [crossing, meet] {
        let point =
            meet_of_three_planes(one, other, last).expect("three planes meeting at a point");
        for (along, at) in point.enclosure().into_iter().zip([3.0, 1.0, 1.0]) {
            assert!(
                (along.lo() - at).abs() < 2e-15 && (along.hi() - at).abs() < 2e-15,
                "{along:?} is not within 2e-15 of {at}"
            );
        }
    }
}

/// **The union is written without the notch, and with the first comb's corner
/// `(3, 1)` where the crossing and the meet are written.** The tooth's wall is
/// stored `a·x − 2a·y − a = 0`, which is `x − 2y = 1` exactly, the gap floor
/// `y = 1` exactly, and the second comb's wall `−a·x − 2a·y + d = 0`, which is
/// `x + 2y = 5 + δ` with `δ = (d − 5a)/a` between `2^-52` and `2^-51`. So the
/// first comb's corner A is `(3, 1)`, the crossing B is `(3 + δ, 1)` and the
/// meet C is `(3 + δ/2, 1 + δ/4)`, and the notch, the triangle between them
/// from `z = 1` to `2`, holds `δ²/8`, `1.7e-32`.
///
/// A's box is the point itself, so `Corners::of` in `fuse/assemble.rs` would
/// write it there. B's and C's boxes have the one middle W,
/// `(3.0000000000000004, 1)`, so it writes them there as one corner. Both of
/// those boxes hold A. As written, the floor and the tooth's wall then each
/// have a strip from A to W, from `z = 0` to `1`, where the plane of the second
/// comb's wall cuts them below that wall, and from `z = 1` to `2`, the notch's.
/// Each pair is one ring walked both ways, and `docs/decisions.md` §431's rule
/// leaves it out. So the union has no corner at `(3, 1, z)`: the first comb's
/// `(3, 1, 0)` is written at W, one rounding off, and its `(3, 1, 2)` and the
/// second comb's `(3, 1, 1)` and `(3, 1, 2)` go with the notch.
#[test]
fn the_union_leaves_out_a_notch_whose_three_corners_round_to_two_points() {
    let (model, keys) = comb_and_comb().build();
    let [tooth_wall, floor, wall, bottom] = the_planes_through_three_one(&model, &keys);
    let (a, d) = (tooth_wall[0], wall[3]);
    assert_eq!(tooth_wall, [a, -2.0 * a, 0.0, -a], "the tooth's wall");
    assert_eq!(floor, [0.0, 1.0, 0.0, -1.0], "the gap floor");
    assert_eq!(wall, [-a, -2.0 * a, 0.0, d], "the second comb's wall");

    let delta = plus(&[d], &negated(&times(5.0, a))).iter().sum::<f64>() / a;
    assert!(
        2f64.powi(-52) < delta && delta < 2f64.powi(-51),
        "the second comb's wall is x + 2y = 5 + {delta:e}"
    );
    let notch = delta * delta / 8.0;
    assert!(
        (notch / 1.733_336_949_948_512_3e-32 - 1.0).abs() < 1e-14,
        "the notch holds {notch:e}"
    );

    let w = [3.000_000_000_000_000_4, 1.0, 1.0];
    for (planes, written, what) in [
        (
            [bottom, floor, tooth_wall],
            [3.0, 1.0, 1.0],
            "the first comb's corner",
        ),
        ([bottom, floor, wall], w, "the crossing"),
        ([bottom, wall, tooth_wall], w, "the meet"),
    ] {
        let [one, other, last] = planes;
        let box_of_it = meet_of_three_planes(one, other, last)
            .expect("three planes meeting at a point")
            .enclosure();
        assert_eq!(
            box_of_it.map(|along| along.lo() + (along.hi() - along.lo()) / 2.0),
            written,
            "where {what} is written"
        );
        assert!(
            box_of_it
                .iter()
                .zip([3.0, 1.0, 1.0])
                .all(|(along, at)| along.lo() <= at && at <= along.hi()),
            "{what}'s box {box_of_it:?} holds the first comb's corner"
        );
    }

    assert_eq!(
        corners_near_three_one(&model, keys[0]),
        [[3.0, 1.0, 0.0], [3.0, 1.0, 2.0]],
        "the first comb's corners"
    );
    assert_eq!(
        corners_near_three_one(&model, keys[1]),
        [[3.0, 1.0, 1.0], [3.0, 1.0, 2.0]],
        "the second comb's corners"
    );
    let (either, solids) = fused(&model, keys[0], keys[1], Rule::Union);
    let [solid] = solids[..] else {
        panic!("the union is {solids:?}");
    };
    assert_eq!(
        corners_near_three_one(&either, solid),
        [[w[0], 1.0, 0.0], w],
        "the union's corners"
    );
}

fn configuration(first: Placed, second: Placed) -> Configuration {
    Configuration {
        solids: vec![first, second],
        turn: None,
        elsewhere: Elsewhere::here(),
    }
}

fn frame_and_comb() -> Configuration {
    configuration(
        Placed {
            shape: Shape::Frame { width: 4, depth: 3 },
            skew: Skew { x_per_y: -1 },
            height: 1,
            at: [0, 0, 0],
        },
        Placed {
            shape: Shape::Comb {
                teeth: 3,
                tooth_height: 2,
            },
            skew: Skew::none(),
            height: 1,
            at: [-2, -2, 0],
        },
    )
}

fn comb_and_ell() -> Configuration {
    configuration(
        Placed {
            shape: Shape::Comb {
                teeth: 2,
                tooth_height: 3,
            },
            skew: Skew { x_per_y: 1 },
            height: 1,
            at: [0, 0, 0],
        },
        Placed {
            shape: Shape::Ell {
                width: 5,
                depth: 4,
                bite_width: 1,
                bite_depth: 3,
            },
            skew: Skew { x_per_y: -2 },
            height: 3,
            at: [1, 1, 1],
        },
    )
}

fn frame_and_ell() -> Configuration {
    configuration(
        Placed {
            shape: Shape::Frame { width: 3, depth: 3 },
            skew: Skew { x_per_y: 1 },
            height: 2,
            at: [0, 0, 0],
        },
        Placed {
            shape: Shape::Ell {
                width: 4,
                depth: 4,
                bite_width: 3,
                bite_depth: 1,
            },
            skew: Skew { x_per_y: -2 },
            height: 1,
            at: [3, 1, 1],
        },
    )
}

fn comb_and_comb() -> Configuration {
    configuration(
        Placed {
            shape: Shape::Comb {
                teeth: 3,
                tooth_height: 2,
            },
            skew: Skew { x_per_y: 2 },
            height: 2,
            at: [0, 0, 0],
        },
        Placed {
            shape: Shape::Comb {
                teeth: 2,
                tooth_height: 2,
            },
            skew: Skew { x_per_y: -2 },
            height: 1,
            at: [3, 0, 1],
        },
    )
}

/// Fuse a copy of `model` under a certified policy, and check that what came
/// back validates and that nothing was left unproved.
fn fused(
    model: &Model,
    arguments: SolidKey,
    tools: SolidKey,
    rule: Rule,
) -> (Model, Vec<SolidKey>) {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    let solids = match model.fuse(&[arguments], &[tools], rule, &Policy::certified(), &mut log) {
        Ok(fused) => fused.solids().to_vec(),
        Err(why) => panic!("the {rule:?} refused: {why:?}"),
    };
    assert!(log.is_empty(), "a certified {rule:?} logged {log:?}");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse's answer validates");
    (model, solids)
}

/// Fuse a copy of `model` under a tolerant policy, and check that what came
/// back validates under it.
fn fused_tolerantly(
    model: &Model,
    arguments: SolidKey,
    tools: SolidKey,
    rule: Rule,
) -> (Model, Vec<SolidKey>) {
    let policy = Policy::tolerant(100.0);
    let mut model = model.clone();
    let solids = match model.fuse(
        &[arguments],
        &[tools],
        rule,
        &policy,
        &mut ProvenanceLog::new(),
    ) {
        Ok(fused) => fused.solids().to_vec(),
        Err(why) => panic!("the tolerant {rule:?} refused: {why:?}"),
    };
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("a tolerant fuse's answer validates under its policy");
    (model, solids)
}

/// The fuse's answer has four faces along one edge, so it is not a solid.
fn is_not_a_solid(model: &Model, arguments: SolidKey, tools: SolidKey, rule: Rule) {
    let mut model = model.clone();
    let answer = model
        .fuse(
            &[arguments],
            &[tools],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|fused| fused.solids().to_vec());
    assert!(
        matches!(answer, Err(Error::ResultIsNotManifold { count: 4 })),
        "the {rule:?} gave {answer:?}"
    );
}

/// The solids hold `volume` together, to within the rounding their measures
/// own up to.
fn holds_exactly(model: &Model, solids: &[SolidKey], volume: f64, what: &str) {
    let (measured, error) = measured(model, solids);
    assert!(
        (measured - volume).abs() <= error,
        "{what} holds {measured} give or take {error}, not {volume}"
    );
}

/// What the solids hold together, and the rounding the measures own up to.
fn measured(model: &Model, solids: &[SolidKey]) -> (f64, f64) {
    assert!(!solids.is_empty(), "no solid at all");
    let (mut measured, mut error) = (0.0, 0.0);
    for &solid in solids {
        let measure = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.1).expect("a tenth of a millimetre is a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a flat solid this size measures");
        measured += measure.volume();
        error += measure.volume_error();
    }
    (measured, error)
}

/// How many faces each solid has, most first.
fn face_counts(model: &Model, solids: &[SolidKey]) -> Vec<usize> {
    let mut counts: Vec<usize> = solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .faces()
                .count()
        })
        .collect();
    counts.sort_unstable_by(|one, other| other.cmp(one));
    counts
}

/// The planes the two combs store through `(3, 1, 1)`, as `[a, b, c, d]`: the
/// first comb's tooth wall `x − 2y = 1`, the gap floor `y = 1` both combs
/// share, the second comb's wall `x + 2y = 5` and its bottom `z = 1`.
fn the_planes_through_three_one(model: &Model, keys: &[SolidKey]) -> [[f64; 4]; 4] {
    let first = planes_through_three_one_one(model, keys[0]);
    let second = planes_through_three_one_one(model, keys[1]);
    let tooth_wall = one_plane_of(&first, |&[a, b, _, _]| a > 0.0 && b < 0.0);
    let first_floor = one_plane_of(&first, |&[a, b, _, _]| a == 0.0 && b > 0.0);
    let floor = one_plane_of(&second, |&[a, b, _, _]| a == 0.0 && b > 0.0);
    let wall = one_plane_of(&second, |&[a, b, _, _]| a < 0.0 && b < 0.0);
    let bottom = one_plane_of(&second, |&[_, _, c, _]| c < 0.0);
    assert_eq!(first_floor, floor, "the two gap floors are one plane");
    [tooth_wall, floor, wall, bottom]
}

/// The one plane of `planes` that `which` picks.
fn one_plane_of(planes: &[[f64; 4]], which: impl Fn(&[f64; 4]) -> bool) -> [f64; 4] {
    let found: Vec<[f64; 4]> = planes
        .iter()
        .copied()
        .filter(|plane| which(plane))
        .collect();
    let [plane] = found[..] else {
        panic!("one plane, not {found:?}");
    };
    plane
}

/// The planes of `solid`'s faces that pass within a rounding of `(3, 1, 1)`,
/// each once, as `[a, b, c, d]`.
fn planes_through_three_one_one(model: &Model, solid: SolidKey) -> Vec<[f64; 4]> {
    let mut planes: Vec<[f64; 4]> = Vec::new();
    for face in model.solid(solid).expect("this model made the key").faces() {
        let plane = face.plane().coefficients();
        let [a, b, c, d] = plane;
        if (3.0 * a + b + c + d).abs() < 1e-12 && !planes.contains(&plane) {
            planes.push(plane);
        }
    }
    planes
}

fn has_a_corner_at(model: &Model, solid: SolidKey, at: [f64; 3]) -> bool {
    model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .any(|face| {
            face.outer_wire()
                .points()
                .any(|corner| corner.coordinates() == at)
        })
}

/// The corners of `solid`'s faces within `1e-12` of the line `x = 3, y = 1`,
/// each once, lowest first.
fn corners_near_three_one(model: &Model, solid: SolidKey) -> Vec<[f64; 3]> {
    let mut corners: Vec<[f64; 3]> = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .flat_map(|face| face.wires().flat_map(|wire| wire.points()))
        .map(|corner| corner.coordinates())
        .filter(|&[x, y, _]| (x - 3.0).abs() < 1e-12 && (y - 1.0).abs() < 1e-12)
        .collect();
    corners.sort_by(|one, other| one.partial_cmp(other).expect("no corner is NaN"));
    corners.dedup();
    corners
}

/// Six times the volume the solid's triangles enclose, exactly: the sum over
/// every triangle of `a · (b × c)`, hung from the world origin, as
/// `a_flat_solid_publishes_the_rounding_of_its_measure.rs` works it out.
fn six_times_the_volume(model: &Model, solid: SolidKey) -> Vec<f64> {
    let mut total = Vec::new();
    for face in model.solid(solid).expect("this model made the key").faces() {
        face.for_each_triangle(|corners| {
            total = plus(
                &total,
                &six_times_the_volume_under(corners.map(|corner| corner.coordinates())),
            );
        });
    }
    total
}

/// `a · (b × c)`, exactly.
fn six_times_the_volume_under([a, b, c]: [[f64; 3]; 3]) -> Vec<f64> {
    let mut total = Vec::new();
    for (first, second, third) in [(0, 1, 2), (1, 2, 0), (2, 0, 1)] {
        let across = plus(
            &times(b[second], c[third]),
            &negated(&times(b[third], c[second])),
        );
        total = plus(&total, &times_expansion(&across, a[first]));
    }
    total
}

fn times(left: f64, right: f64) -> Vec<f64> {
    times_expansion(&[left], right)
}

fn times_expansion(expansion: &[f64], factor: f64) -> Vec<f64> {
    assert!(
        expansion
            .iter()
            .all(|&entry| product_is_exact(entry, factor)),
        "corners this size lose nothing below the subnormals"
    );
    let mut product = vec![0.0; 2 * expansion.len()];
    let used = scale_expansion(expansion, factor, &mut product);
    product.truncate(used);
    product
}

fn plus(left: &[f64], right: &[f64]) -> Vec<f64> {
    let mut sum = vec![0.0; left.len() + right.len()];
    let used = expansion_sum(left, right, &mut sum);
    let mut packed = vec![0.0; used];
    let kept = compress(&sum[..used], &mut packed);
    packed.truncate(kept);
    packed
}

fn negated(expansion: &[f64]) -> Vec<f64> {
    expansion.iter().map(|entry| -entry).collect()
}
