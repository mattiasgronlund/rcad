//! The shapes rows Q1125, Q1393 and Q1581 tried, looking for a fuse with a
//! piece that no point placed because no ray could be fired at a ring
//! (`docs/decisions.md` §35.6).
//!
//! `select.rs`'s `settle_against_each` folds
//! `Containment::CannotFireARayAtAFacesSurface` into the undecided points, and
//! `Error::CannotFireARayAtAFacesSurface` exists to name it instead. Changing
//! that needs a fuse which is refused for it, and **none of these is**. Each
//! test below is one family, with the step that stopped it pinned as it stands.
//!
//! What reaching it takes: a point proved inside a piece, put to a ray against
//! a solid with a ring face, whose own ray has no exact line, **and** whose box
//! grown to take in the exact point beside it meets a face of that solid, for
//! all three targets (`classify.rs`'s `what_a_point_beside_it_says`, §323) —
//! and that for every point offered inside the piece, with no side, corner,
//! plane or piece across placing it after. So every point offered lies within
//! about one box, `3e-13`, of that solid's skin. A piece of a surface that
//! crosses the skin is cut along it, so its points sit clear of it; a piece on
//! the same surface is caught as lying on the face, or shown off it; and a
//! sliver thinner than a box has no point found inside it at all. What is
//! left is a piece a rounding off a face of the ring solid.
//!
//! Every surface in the fuse is one the ring is crossed with: a plane through
//! the ring's axis or square to it, a ball, a pipe or a cone on the axis, or
//! the ring itself. Any other is refused when the two are crossed
//! (`rcad_geom::Error`'s two torus refusals), and that holds for the ring
//! solid's own planes too: a block a bend grows out of is refused for its
//! sides, under a quarter of a tube whose every surface the ring is crossed
//! with. So the piece, and the face it is a rounding off, lie on such
//! surfaces, and Q1393 tried each kind it could build:
//!
//! - a plane through the axis a rounding off a cap, placed there by a turn
//!   whose cosine or sine rounds: two bends, and a quarter of a bar over a
//!   bend;
//! - a plane square to the axis a hair off a flat face: a quarter of a tube
//!   under half a ring, and over the block a bend grows out of;
//! - a pipe on the axis a hair off a pipe face: a bar on the inner wall, or a
//!   tube round the outer wall, of half a ring joined to a quarter of a tube.
//!   That solid is refused with each of them, and with a bar in its hole
//!   touching nothing too, so this kind is not reached;
//! - a torus a rounding off a torus: a bend and a copy of it moved a hair.
//!
//! Q1581 tried the two kinds left, a ball or a cone on the axis a hair off a
//! face of its own kind. A quarter bend joined to a cone on its axis is
//! refused at `order along a crossing` while the two are cut, so that ring
//! solid is never built. One joined to a ball on its axis was refused the same
//! way until Q2118, and answers now; a piece of a ball a hair off that union's
//! ball face has not been tried. A ball or a cone joined to a ring, built face
//! by face with no seam, is built; but against a piece of its own kind on it,
//! it is refused `ResultIsNotManifold`, or `Undecidable` at `side of sphere` or
//! `side of cone`, and moved off the axis, the piece is refused when the two
//! are crossed. So neither kind is reached by what was tried.
//!
//! Run with `settle_against_each`'s arm made to print, every fuse here, only
//! the bends placed a quarter turn round reached it: 6 of their 36 fuses,
//! ring 4 and tube 1 turned `−π/2`, every rule both ways round, 18 prints in
//! all. Each time the piece was the placed bend's cap on the plane whose
//! normal is `(−2.2e-16, 1, 0)`, and its three points, at `(4, 8.9e-16, 0)`,
//! `(4, 6.3e-16, 0)` and `(4, 8.9e-16, 0)`, could not be fired at the first
//! bend's ring. Each of those pieces was then placed by some other point or
//! rule, and since Q1537 each of the six fuses answers. Q1581's fuses, run
//! with the arm made to print, printed nothing. The ball and the cone joined
//! to a ring printed nothing again at Q1757, after Q1566 let them past the
//! meet of three that refused them before. The quarter of a bar placed round
//! a bend got past the cut at Q1817 and has not been run with the arm made to
//! print since; none of its fuses is refused `Undecidable`, so every piece of
//! each was placed.
//!
//! Q1125's run, of all 1755 tests of this crate's suite with the arm made to
//! print, printed nine times, all in the placed bends' test, and each piece it
//! printed for was placed by some other point or rule too. One test of that
//! run, `a_half_bend_and_two_quarter_bars_spelling_a_plane_through_its_axis_an_ulp_apart_are_refused`,
//! stopped at its first fuse; run again with all 24 of its fuses let through,
//! it printed none.
//!
//! Q1950 tried bends written out at a ring angle that is not a multiple of
//! `π/2`, the one case `a_bend_placed_a_quarter_turn_round_from_another_answers_every_rule`
//! and `a_bend_placed_half_a_turn_round_from_another_places_every_piece` leave
//! untried, since each starts its first bend's wall at angle 0. Tried over
//! `0.1` to `1.4` radians, a copy a quarter turn backward or half a turn
//! round, ring 4 and tube 1: most angles are refused before a ray, each at a
//! different step — `Undecidable` at `a point inside a piece of a face` or at
//! `side of torus`, `ACurvedFaceGoesRightRoundWithNoSeam`, or
//! `ResultIsNotManifold` — never at `CannotFireARayAtAFacesSurface`, so this
//! family is not reached either. `0.9` radians answers cleanly for every rule,
//! both turns and all three rings in `PLACED_RINGS`, and is the one pinned in
//! `a_bend_written_out_at_another_angle_and_a_copy_turned_round_answer_every_rule`.

use core::f64::consts::{FRAC_PI_2, PI};
use std::collections::BTreeMap;

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale,
    Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};
use smallvec::smallvec;

mod rings;

use rings::{
    a_bend_turned_and_written_out, a_bend_written_out_at_an_angle, a_quarter_bend,
    the_top_half_of_a_ring,
};

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// A quarter bend and a second one written out turned by whole quarter turns,
/// on one torus: every rule, both ways round, answers and validates, for four
/// rings, the one whose tube all but reaches its axis among them.
///
/// Every piece of one bend's wall that is off the other bend lies on the same
/// torus, so it is either held by the other's wall or shown off it, and a
/// point of it is fired from beside it. So every piece is placed, which is
/// what an answer says, and no fuse here can reach the refusal.
#[test]
fn two_bends_turned_by_whole_quarter_turns_answer_every_rule() {
    let a_hair = 2.0_f64.powi(-30);
    for (ring, tube) in [(1.0, 1.0 - a_hair), (4.0, 1.0), (1.0, 0.5), (2.0, 1.0)] {
        for (quarters, turns) in [(1, 1), (1, 2), (1, 3), (2, 1), (2, 2)] {
            for rule in EVERY_RULE {
                for the_quarter_first in [true, false] {
                    let what = format!(
                        "ring {ring}, tube {tube}, {quarters} quarters turned {turns}, {rule:?}, \
                         the quarter first {the_quarter_first}"
                    );
                    let mut model = a_model();
                    let quarter = a_bend_turned_and_written_out(&mut model, ring, tube, 1, 0)
                        .expect("a quarter bend");
                    let turned =
                        a_bend_turned_and_written_out(&mut model, ring, tube, quarters, turns)
                            .expect("a bend turned by whole quarter turns");
                    let solids = fused(&mut model, quarter, turned, rule, the_quarter_first, &what);
                    let apart = quarters == 1 && turns == 2;
                    let expected = match rule {
                        Rule::Union if apart => 2,
                        Rule::Union | Rule::Difference => 1,
                        Rule::Intersection => 0,
                        other => panic!("not a rule this tries: {other:?}"),
                    };
                    assert_eq!(solids, expected, "{what}");
                }
            }
        }
    }
}

/// A quarter bend and a copy placed a hair round the ring from it. The copy's
/// wall then runs past the first bend's cap by that hair, and a point of that
/// sliver is as near the first bend's cap as the hair: once the hair is
/// thinner than the box round such a point, no point beside it is clear of
/// the cap, which is the refusal's shape.
///
/// The hair never gets that thin with the fuse still running. At `1e-9` of a
/// turn every rule answers. At `1e-12` the pieces are not cut,
/// `Undecidable` at `side of torus`; at `1e-13`, `3e-14` and `1e-14` no
/// point is found inside a piece at all, `a point inside a piece of a face`.
/// At `3e-14` and `1e-14` the pieces were not cut, `side of plane`, until a
/// point on the ring and on a plane through its axis was put on a side of
/// another such plane by the planes' own numbers (Q1392). A box round a
/// point of a piece of the wall is about `256 ε · 5`, `3e-13` across, so the
/// hair would have to be thinner than about `6e-14` of a turn.
///
/// A whole turn, `2π`, is such a hair: its sine rounds to `−2.4e-16`, so the
/// copy sits that far back round the ring. It is refused the same way, before
/// a ray.
///
/// Run with a print where no rule placed a piece (Q1537), each refused fuse
/// stops at a band of one bend's wall a hair wide, outside the other bend.
/// With the first bend named first it is the first bend's wall between its
/// own start cap rim and the copy's start cap plane; with the copy first, the
/// copy's wall between the first bend's end cap plane and its own end cap
/// rim. Neither rule Q1537 added places it:
///
/// - Its cut is also a side of a piece of the other bend's wall, with the
///   same partner, but its material is on the other side of that partner, so
///   it is not a piece lying on the other bend's skin.
/// - The piece of its own wall across that cut lies on the other bend's wall,
///   and the cut runs along the other bend's cap rim, so the cut does not
///   start clear of that cap.
/// - Its other side is its own cap rim, and no piece of its own wall lies
///   across that.
///
/// Not tried: placing it by which side of the other bend's cap plane its
/// material is on, outside in front of the cap's outward plane, as
/// `by_a_side_from_an_edge_it_starts_on` places a flat piece beyond an edge.
#[test]
fn a_bend_placed_a_hair_round_from_another_answers_or_is_refused_before_a_ray() {
    for (turn, said) in [
        (1e-9, None),
        (1e-12, Some("side of torus")),
        (1e-13, Some("a point inside a piece of a face")),
        (3e-14, Some("a point inside a piece of a face")),
        (1e-14, Some("a point inside a piece of a face")),
        (2.0 * PI, Some("a point inside a piece of a face")),
    ] {
        for rule in EVERY_RULE {
            for the_first_first in [true, false] {
                let what =
                    format!("{turn:e} of a turn, {rule:?}, the first first {the_first_first}");
                let mut model = a_model();
                let first = a_quarter_bend(&mut model, 4.0, 1.0);
                let placed = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, turn);
                let fused = fused_or_refused(&mut model, first, placed, rule, the_first_first);
                expect(fused, said, &what);
            }
        }
    }
}

/// The three rings the bends placed round by a float turn are tried on.
const PLACED_RINGS: [(f64, f64); 3] = [(4.0, 1.0), (1.0, 0.5), (2.0, 1.0)];

/// A quarter bend and a copy placed a quarter turn either way round, by
/// `place_solid` rather than written out, for three rings: `cos(π/2)` is not
/// nought, so the copy's cap lies on a plane through the ring's axis a
/// rounding off the first bend's, the shape two cubes a rounding apart are
/// (`tests/two_turned_cubes_a_rounding_apart.rs`), and the two bends overlap
/// in a sliver of ring or touch across a gap no `f64` lies in.
///
/// The pieces are cut: a point of one bend's cap rim, a rounding off the
/// other's cap plane, is put on its side of that plane by the two planes' own
/// numbers (Q1392). Each such plane cuts the other bend's wall in a circle a
/// rounding round from that bend's own cap rim, and the band of wall between
/// the two holds no point the search offers. The band shares its rim with the
/// band of the other bend's wall that the other cap plane cut, on the same
/// ring and on the same side of the same plane, so it lies on that bend's
/// skin (Q1537). Until then every rule both ways round was refused
/// `Undecidable` at `a point inside a piece of a face`.
///
/// Every rule both ways round now answers one solid that validates: the union
/// two bends' worth of ring, the difference one, and the intersection a sliver
/// under a millionth of a millionth of a bend, each against `π² R r² / 2` a
/// bend, within the measure's own error. Ring 4 and tube 1 turned `+π/2` was
/// refused `CannotMeshAFace` for the union with the placed bend named first
/// and for the intersection both ways round: run with a print where the shells
/// are nested, the face not meshed was each time a band of ring wall between
/// two meridian circles a rounding apart, a whole circle each, cut across and
/// refused `CornersCoincide`. Such a band is now zipped up from rim to rim
/// rather than cut across (Q1734).
#[test]
fn a_bend_placed_a_quarter_turn_round_from_another_answers_every_rule() {
    for (ring, tube) in PLACED_RINGS {
        let one_bend = PI * PI * ring * tube * tube / 2.0;
        for turn in [FRAC_PI_2, -FRAC_PI_2] {
            for rule in EVERY_RULE {
                for the_first_first in [true, false] {
                    let what = format!(
                        "ring {ring}, tube {tube}, {turn} round, {rule:?}, \
                         the first first {the_first_first}"
                    );
                    let mut model = a_model();
                    let first = a_quarter_bend(&mut model, ring, tube);
                    let placed = a_quarter_bend_placed_round_by(&mut model, ring, tube, turn);
                    let measured =
                        fused_and_measured(&mut model, first, placed, rule, the_first_first)
                            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
                    let [(volume, error)] = measured[..] else {
                        panic!("{what}: one solid, not {measured:?}");
                    };
                    let near = |expected: f64| (volume - expected).abs() <= error;
                    let answered = match rule {
                        Rule::Union => near(2.0 * one_bend),
                        Rule::Difference => near(one_bend),
                        Rule::Intersection => volume < one_bend * 1e-12,
                        other => panic!("not a rule this tries: {other:?}"),
                    };
                    assert!(answered, "{what}: {volume} ± {error}, a bend {one_bend}");
                }
            }
        }
    }
}

/// The same two bends with the copy placed half a turn round: `sin(π)` is not
/// nought, so each bend's cap plane cuts the other bend's wall in a circle a
/// rounding round from that bend's own cap rim, half a ring away from the
/// first, and the band of wall between the two holds no point the search
/// offers. The start of the cut is clear of every face of the other bend, so
/// the band is placed as the rest of its own wall across the cut is: outside
/// the other bend (Q1537). That start lies on the other bend's cap plane and
/// on its ring, and is shown off its cap by the half-space that names the
/// cap's circle, half a ring away. Until then every rule both ways round was refused
/// `Undecidable` at `a point inside a piece of a face`.
///
/// The intersection answers nothing, the union the two bends apart and the
/// difference the first bend, each against `π² R r² / 2` a bend, and each
/// validates. The union and the difference keep each band beside the rest of
/// its wall, and were refused `CannotMeshAFace` until Q1734: run with a print
/// where the shells are nested, the face not meshed was each time a band of
/// ring wall between two meridian circles a rounding apart, a whole circle
/// each, refused `CornersCoincide`.
#[test]
fn a_bend_placed_half_a_turn_round_from_another_places_every_piece() {
    for (ring, tube) in PLACED_RINGS {
        let one_bend = PI * PI * ring * tube * tube / 2.0;
        for rule in EVERY_RULE {
            for the_first_first in [true, false] {
                let what = format!(
                    "ring {ring}, tube {tube}, {rule:?}, the first first {the_first_first}"
                );
                let mut model = a_model();
                let first = a_quarter_bend(&mut model, ring, tube);
                let placed = a_quarter_bend_placed_round_by(&mut model, ring, tube, PI);
                let measured = fused_and_measured(&mut model, first, placed, rule, the_first_first)
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"));
                let bends = match rule {
                    Rule::Union => 2,
                    Rule::Difference => 1,
                    Rule::Intersection => 0,
                    other => panic!("not a rule this tries: {other:?}"),
                };
                assert_eq!(measured.len(), bends, "{what}: {measured:?}");
                for (volume, error) in measured {
                    assert!(
                        (volume - one_bend).abs() <= error,
                        "{what}: {volume} ± {error}, a bend {one_bend}"
                    );
                }
            }
        }
    }
}

/// The two tests above place the first bend's wall from angle 0, so the band
/// of wall a rounding wide between a cap rim and the other bend's meridian
/// sits at a ring angle of 0 or `π/2`, each a multiple of `π/2`. Placing a
/// copy by `Isometry::rotation` cannot try another angle either: a placed
/// face is meshed in the frame it is stored in, unturned, so it never reaches
/// the branch `atan2` cuts the ring angle on.
///
/// Here both bends are written out instead, one at `0.9` radians — not a
/// multiple of `π/2` — and the other a quarter turn backward or half a turn
/// round from it, so the band this time sits at a ring angle of `0.9` or
/// `0.9 − π/2`. Every rule both ways round answers and validates, for all
/// three rings, each solid's volume against `π² R r² / 2` a bend. Turned a
/// quarter backward the two bends meet only at their shared cap plane and
/// share no material there, so the intersection answers no solid at all —
/// unlike `a_bend_placed_a_quarter_turn_round_from_another_answers_every_rule`,
/// whose exact axis angles leave a floating-point sliver of overlap instead.
///
/// Not every angle answers this way: run with the arm made to print over a
/// spread of angles (row Q1950), `0.3`, `0.7` and most others are refused
/// `Undecidable`, some at `a point inside a piece of a face` and some at
/// `side of torus`, and a few are refused `ACurvedFaceGoesRightRoundWithNoSeam`
/// or `ResultIsNotManifold` instead — each a different band than the one
/// `a_bend_placed_a_quarter_turn_round_from_another_answers_every_rule` and
/// `a_bend_placed_half_a_turn_round_from_another_places_every_piece` already
/// place, at a ring angle that is not a multiple of `π/2`. `0.9` is the one
/// tried here that answers for every ring, both turns and every rule.
#[test]
fn a_bend_written_out_at_another_angle_and_a_copy_turned_round_answer_every_rule() {
    let angle = 0.9;
    for (ring, tube) in PLACED_RINGS {
        let one_bend = PI * PI * ring * tube * tube / 2.0;
        for turn in [-FRAC_PI_2, PI] {
            let half_a_turn = turn == PI;
            for rule in EVERY_RULE {
                for the_first_first in [true, false] {
                    let what = format!(
                        "ring {ring}, tube {tube}, {angle} then {turn} round, {rule:?}, \
                         the first first {the_first_first}"
                    );
                    let mut model = a_model();
                    let first = a_bend_written_out_at_an_angle(&mut model, ring, tube, 1, angle)
                        .expect("a bend written out at another angle");
                    let other =
                        a_bend_written_out_at_an_angle(&mut model, ring, tube, 1, angle + turn)
                            .expect("a bend written out at another angle, turned round");
                    let measured =
                        fused_and_measured(&mut model, first, other, rule, the_first_first)
                            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
                    let bends = match (rule, half_a_turn) {
                        (Rule::Union, true) => 2,
                        (Rule::Union, false) => 1,
                        (Rule::Difference, _) => 1,
                        (Rule::Intersection, _) => 0,
                        (other, _) => panic!("not a rule this tries: {other:?}"),
                    };
                    assert_eq!(measured.len(), bends, "{what}: {measured:?}");
                    let expected = if rule == Rule::Union && !half_a_turn {
                        2.0 * one_bend
                    } else {
                        one_bend
                    };
                    for (volume, error) in measured {
                        assert!(
                            (volume - expected).abs() <= error,
                            "{what}: {volume} ± {error}, expected {expected}"
                        );
                    }
                }
            }
        }
    }
}

/// A quarter bend of ring 4 and tube 1 placed round by a start, and a copy
/// placed round by that start and a turn on, both by `place_solid` (Q1948).
/// Placed round by `1`, `0.3` or `−2` and half a turn on, or by `0.3` and a
/// quarter turn on, every rule both ways round answers and validates: the
/// union the two bends apart, the intersection nothing and the difference the
/// first bend, each against `π² R r² / 2` a bend.
///
/// Placed round by `0.3` and a quarter turn on, the first bend's end cap and
/// the copy's start cap lie on planes through the ring's axis whose normals
/// differ by an ulp, and the first bend's end cap disk lies strictly in front
/// of the copy's start cap plane. So the two bends touch across a gap no `f64`
/// lies in, neither plane cuts the other bend's wall, and the disk shares no
/// side with any piece. Every rule both ways round was refused `Undecidable`
/// at `a point in a piece that every solid could place` for that disk, until
/// `select.rs`'s `by_a_meridian_disk_beside_it` placed it by the copy's whole
/// start cap (Q2056).
///
/// Each of those fuses then stopped at a later piece, at `a point inside a
/// piece of a face`, and so did each placed round by `0.3` or `−2` and half a
/// turn on. Run with a print where no rule placed a piece, the piece placed
/// round by `0.3` and a quarter turn on was a band of the copy's wall a
/// rounding wide, between its end cap rim and the first bend's start cap
/// plane, which runs on through the axis to it; half a turn on, the band was
/// of the same kind (Q2056). `by_the_piece_across_a_side_clear_of_every_face`
/// places such a band from the start of its cut, which needs `holds.rs`'s
/// `held_in_front_of_all` to say the first bend's wall does not hold that
/// start. That start's box straddles the first bend's start cap plane, and
/// until Q2059 the loop gave up there, before it asked the end cap's plane,
/// which puts the start strictly off the wall.
#[test]
fn a_bend_placed_round_by_a_float_and_a_copy_turned_on_answers_every_rule() {
    let one_bend = PI * PI * 4.0 / 2.0;
    let mut not_the_bends = Vec::new();
    for (start, turn) in [(1.0, PI), (0.3, FRAC_PI_2), (0.3, PI), (-2.0, PI)] {
        for rule in EVERY_RULE {
            for the_first_first in [true, false] {
                let what =
                    format!("{start} and {turn} on, {rule:?}, the first first {the_first_first}");
                let mut model = a_model();
                let first = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, start);
                let copy = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, start + turn);
                let bends = match rule {
                    Rule::Union => 2,
                    Rule::Difference => 1,
                    Rule::Intersection => 0,
                    other => panic!("not a rule this tries: {other:?}"),
                };
                match fused_and_measured(&mut model, first, copy, rule, the_first_first) {
                    Ok(measured)
                        if measured.len() == bends
                            && measured
                                .iter()
                                .all(|&(volume, error)| (volume - one_bend).abs() <= error) => {}
                    Ok(measured) => {
                        not_the_bends.push(format!("{what}: {measured:?}, a bend {one_bend}"));
                    }
                    Err(why) => not_the_bends.push(format!("{what}: {}", the_refusal(&Err(why)))),
                }
            }
        }
    }
    assert_eq!(not_the_bends, Vec::<String>::new());
}

/// The two tests above measure every answer, and a band of wall a rounding
/// wide encloses no volume whichever way its triangles turn: drawn inside out,
/// it would still measure. So every side of each answer's mesh, ring 4 and
/// tube 1 placed each of the three ways round, is checked to be used once each
/// way, which a band turned the wrong way fails along both its rims.
#[test]
fn every_side_of_a_bend_placed_round_from_another_is_used_once_each_way() {
    let policy = Policy::certified();
    let deflection = rcad_geom::Deflection::of(0.01).expect("a deflection");
    for turn in [FRAC_PI_2, -FRAC_PI_2, PI] {
        for rule in EVERY_RULE {
            for the_first_first in [true, false] {
                let what = format!("{turn} round, {rule:?}, the first first {the_first_first}");
                let mut model = a_model();
                let first = a_quarter_bend(&mut model, 4.0, 1.0);
                let placed = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, turn);
                let (arguments, tools) = if the_first_first {
                    ([first], [placed])
                } else {
                    ([placed], [first])
                };
                let answer = model
                    .fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"));
                for &solid in answer.solids() {
                    let mesh = model
                        .solid(solid)
                        .and_then(|solid| {
                            solid.mesh(deflection, &policy, &mut ProvenanceLog::new())
                        })
                        .unwrap_or_else(|why| panic!("{what}: {why:?}"));
                    let mut used: BTreeMap<(u32, u32), u32> = BTreeMap::new();
                    for triangle in mesh.triangles() {
                        let [one, two, three] = triangle.corners();
                        for side in [(one, two), (two, three), (three, one)] {
                            *used.entry(side).or_default() += 1;
                        }
                    }
                    for (&(from, to), &times) in &used {
                        assert_eq!(
                            (times, used.get(&(to, from))),
                            (1, Some(&1)),
                            "{what}: the side from {from} to {to}"
                        );
                    }
                }
            }
        }
    }
}

/// A quarter bend of ring 4 and tube 1 placed round by `−2`, and a copy placed
/// round by `−2 − π/2`, a quarter turn back. The copy's end cap and the first
/// bend's start cap lie on planes through the ring's axis a rounding apart,
/// and each cap's disk lies inside the other bend by about `1.7e-17` at the
/// ring, so the two bends overlap in a sliver. Every rule both ways round was
/// refused `ResultIsNotManifold`, with 3 faces along the first bend's start
/// rim for the union and 1 for the intersection and the difference (Q1949). A
/// point `8e-16` off the first bend's start cap, beyond the copy's end cap
/// plane, was held by that cap, and the ray put it outside the copy: the union
/// and the difference kept the cap and the intersection dropped it.
///
/// `select.rs` now drops what the ray says of a point of a flat piece when a
/// face of the solid it is fired at may run between the point and the piece's
/// plane, as the copy's end cap does here. Each cap is then placed inside the
/// other bend by a piece it shares a side with. So the union and the
/// difference both ways round answer one solid that validates: the union two
/// bends' worth of ring and the difference one, each within the measure's own
/// error of `π² R r² / 2` a bend.
///
/// The intersection is the sliver, a wedge of the ring as wide as the angle
/// between the two caps, `4.3e-18`, so it holds `4π` times that angle,
/// `5.4e-17`. Its mesh's caps lie up to `4.3e-16` off their planes, and the
/// space between those triangles and their planes moves the mesh's sum
/// anywhere from `−4.9e-16` to `3.2e-16` over deflections from `0.01` to
/// `5e-5` (Q2087). `Model::validate` charged a flat face nothing for that
/// space, and passed the sliver on a sum that answered by luck. Charged, no
/// mesh says which way its faces point, and from Q2089 until Q2266 the fuse
/// refused it [`Error::CannotTellWhichWayASolidsFacesPoint`]. Read again with
/// its caps' points moved onto their planes, the finest mesh says it, and the
/// fuse answers the sliver
/// (`two_bends_overlapping_in_a_sliver_thinner_than_a_mesh_can_tell_are_answered`).
/// The measure charges a flat face that space too (Q2077, Q2078), and a
/// sliver a mesh can tell is measured against its wedge in
/// `two_bends_overlapping_in_a_sliver_a_mesh_can_tell_measure_its_wedge`.
#[test]
fn a_bend_placed_round_by_minus_two_and_a_copy_a_quarter_turn_back_overlap_in_a_sliver() {
    let one_bend = PI * PI * 4.0 / 2.0;
    for (rule, expected) in [(Rule::Union, 2.0 * one_bend), (Rule::Difference, one_bend)] {
        for the_first_first in [true, false] {
            let what = format!("{rule:?}, the first first {the_first_first}");
            let mut model = a_model();
            let first = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, -2.0);
            let copy = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, -2.0 - FRAC_PI_2);
            let [(volume, error)] =
                fused_and_measured(&mut model, first, copy, rule, the_first_first)
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"))[..]
            else {
                panic!("{what}: not one solid");
            };
            assert!(
                (volume - expected).abs() <= error,
                "{what}: {volume} ± {error}, expected {expected}"
            );
        }
    }
}

/// The same two bends placed round by `−2`, `−1.984`, `−2.4645`, `2.7745`,
/// `−3.1 + 0.0155 · 117` or `−3.1 + 0.0155 · 58` and a quarter turn back, where
/// the first bend's start cap and the copy's end cap overlap in a wedge `4π`
/// times the angle between them (Q2089, Q2137).
///
/// The wedge is thinner than the space any plain mesh of it can leave out, so
/// no rung of the ladder says which way its faces point: the closest sum is
/// below zero, `−3.3e-16` at `−2`, against a wedge of `5.4e-17`, and the mesh
/// can leave out `6.1e-15`. Almost all of that is its caps, whose points lie
/// up to `4.3e-16` off their planes. The fuse refused the intersection at
/// `−1.984` [`Invalid::VolumeIsNotPositive`] on such a sum, both ways round,
/// which says the faces point inwards; they point out. Until Q2266 every start
/// here was refused [`Error::CannotTellWhichWayASolidsFacesPoint`].
///
/// `Model::validate` now reads the finest mesh again with each point of a cap
/// moved onto the cap's plane, and charges only what that moved mesh can leave
/// out. So the intersection both ways round answers one solid, and
/// `Model::validate` passes it. `−3.1 + 0.0155 · 117` is pinned as that value
/// and not as the literal `−1.2865`, which answers no solid.
///
/// The measure at a deflection of `0.01` still charges the caps for how far
/// their points stray off their planes, so its error, `1.4e-15` to `3.1e-15`
/// over these starts, is wider than the wedge, and the volume it reads is
/// below zero at every one of them. It holds the wedge, but cannot tell it
/// from nothing.
#[test]
fn two_bends_overlapping_in_a_sliver_thinner_than_a_mesh_can_tell_are_answered() {
    for start in [
        -2.0,
        -1.984,
        -2.4645,
        2.7745,
        -3.1 + 0.0155 * 117.0,
        -3.1 + 0.0155 * 58.0,
    ] {
        for the_first_first in [true, false] {
            let what = format!("placed round by {start}, the first first {the_first_first}");
            let mut model = a_model();
            let first = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, start);
            let copy = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, start - FRAC_PI_2);
            let wedge = 4.0 * PI * the_angle_where_the_caps_overlap(&model, [first, copy], start);
            let measured =
                fused_and_measured(&mut model, first, copy, Rule::Intersection, the_first_first)
                    .unwrap_or_else(|why| panic!("{what}: {why:?}, a wedge of {wedge}"));
            let [(volume, error)] = measured[..] else {
                panic!("{what}: {measured:?}, a wedge of {wedge}");
            };
            assert!(
                0.0 < wedge && (volume - wedge).abs() <= error,
                "{what}: {volume} ± {error}, a wedge of {wedge}"
            );
        }
    }
}

/// The same two bends placed round by `−1.75` and a quarter turn back overlap
/// in a wedge of `2.5e-15`, thick enough that a mesh of it says which way its
/// faces point. The intersection both ways round answers it, `Model::validate`
/// passes it, and the measure holds it against the wedge at deflections of
/// `0.01`, `0.001` and `1e-4`, its caps charged for how far the points the
/// mesher places on them stray off their planes (Q2077, Q2078).
#[test]
fn two_bends_overlapping_in_a_sliver_a_mesh_can_tell_measure_its_wedge() {
    let start = -1.75;
    let policy = Policy::certified();
    for the_first_first in [true, false] {
        let what = format!("the first first {the_first_first}");
        let mut model = a_model();
        let first = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, start);
        let copy = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, start - FRAC_PI_2);
        let wedge = 4.0 * PI * the_angle_where_the_caps_overlap(&model, [first, copy], start);
        let (arguments, tools) = if the_first_first {
            ([first], [copy])
        } else {
            ([copy], [first])
        };
        let answer = model
            .fuse(
                &arguments,
                &tools,
                Rule::Intersection,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let [solid] = answer.solids()[..] else {
            panic!("{what}: one solid, not {:?}", answer.solids());
        };
        for size in [0.01, 0.001, 1e-4] {
            let deflection = rcad_geom::Deflection::of(size).expect("a deflection");
            let (volume, error) = model
                .solid(solid)
                .and_then(|solid| solid.measure(deflection, &policy, &mut ProvenanceLog::new()))
                .map(|measured| (measured.volume(), measured.volume_error()))
                .unwrap_or_else(|why| panic!("{what}, deflection {size}: {why:?}"));
            assert!(
                0.0 < wedge && (volume - wedge).abs() <= error,
                "{what}, deflection {size}: {volume} ± {error}, a wedge of {wedge}"
            );
        }
    }
}

/// The angle by which the start cap of the first of `bends` reaches past the
/// end cap of the second, both on planes through the ring's axis at the angle
/// `start` round it: the angle between the two caps of their sliver.
///
/// Each cap faces out of the sliver, so their outward normals all but cancel,
/// and what is left of their sum points in along the radius by that angle.
/// Each cap's plane runs through the axis, so the wedge between them holds the
/// tube's area times the angle times the ring, by Pappus.
fn the_angle_where_the_caps_overlap(model: &Model, bends: [SolidKey; 2], start: f64) -> f64 {
    let [first, second] = bends.map(|bend| {
        model
            .solid(bend)
            .expect("this model made the key")
            .faces()
            .filter(|face| face.surface().as_plane().is_some())
            .map(|face| face.outward_normal())
            .collect::<Vec<_>>()
    });
    let left_over = first
        .iter()
        .flat_map(|&one| second.iter().map(move |&two| one + two))
        .min_by(|one, two| one.length().total_cmp(&two.length()))
        .expect("each bend has two caps");
    -(left_over.x() * start.cos() + left_over.y() * start.sin())
}

/// A quarter of a round bar of radius 6, `z` from −2 to 2, holds a quarter
/// bend of ring 4 and tube 1 whole, its two flat faces on the planes of the
/// bend's two caps. Where it sits, every rule both ways round answers and
/// validates.
///
/// Placed round a quarter turn either way, or half a turn, its flat faces lie
/// on planes through the ring's axis a rounding off the bend's caps, as the
/// placed bend's caps do in
/// `a_bend_placed_a_quarter_turn_round_from_another_answers_every_rule`. Until
/// Q1817 every rule both ways round was then refused `Undecidable` at `order
/// along a crossing` while the pieces were cut. Q1817 orders those crossings
/// by their angle round the pipe, so every fuse now gets past the cut.
///
/// Turned a quarter either way, the intersection answers a sliver, and turned
/// half a turn it answers nothing. Turned any of the three ways, the bend less
/// the quarter answers the bend, `π² R r² / 2`, validating; half a turn round it
/// was refused `CannotMeshAFace` until a band the chart reads as one line was
/// zipped up rather than cut across (Q1734). The union and the quarter less the bend are refused
/// `DynamicRangeTooWide`: the answer's largest coordinate over its shortest
/// edge is `3.7e15` a quarter turn round and `8.2e15` half a turn round, an
/// edge a few roundings long (row Q1865).
#[test]
fn a_quarter_of_a_bar_placed_round_from_a_bend_in_it_answers_or_keeps_an_edge_a_rounding_long() {
    for rule in EVERY_RULE {
        for the_bend_first in [true, false] {
            let what = format!("where it sits, {rule:?}, the bend first {the_bend_first}");
            let mut model = a_model();
            let bend = a_quarter_bend(&mut model, 4.0, 1.0);
            let quarter = a_quarter_of_a_bar(&mut model);
            let solids = fused(&mut model, bend, quarter, rule, the_bend_first, &what);
            let expected = match rule {
                Rule::Union | Rule::Intersection => 1,
                Rule::Difference if the_bend_first => 0,
                Rule::Difference => 1,
                other => panic!("not a rule this tries: {other:?}"),
            };
            assert_eq!(solids, expected, "{what}");
        }
    }
    let one_bend = PI * PI * 4.0 / 2.0;
    for turn in [FRAC_PI_2, -FRAC_PI_2, PI] {
        let half_a_turn = turn == PI;
        let too_wide = if half_a_turn {
            8_165_619_676_597_685.0
        } else {
            3_747_221_395_533_424.5
        };
        for rule in EVERY_RULE {
            for the_bend_first in [true, false] {
                let what = format!("{turn} round, {rule:?}, the bend first {the_bend_first}");
                let mut model = a_model();
                let bend = a_quarter_bend(&mut model, 4.0, 1.0);
                let quarter = a_quarter_of_a_bar(&mut model);
                model
                    .place_solid(
                        quarter,
                        &Isometry::rotation(&Axis::z(), turn).expect("a real angle"),
                    )
                    .expect("this model made the key");
                let measured = fused_and_measured(&mut model, bend, quarter, rule, the_bend_first);
                match (rule, the_bend_first, half_a_turn) {
                    (Rule::Intersection, _, true) => assert!(
                        matches!(&measured, Ok(solids) if solids.is_empty()),
                        "{what}: {measured:?}"
                    ),
                    (Rule::Intersection, _, false) => {
                        let measured = measured.unwrap_or_else(|why| panic!("{what}: {why:?}"));
                        let [(volume, _)] = measured[..] else {
                            panic!("{what}: one solid, not {measured:?}");
                        };
                        assert!(volume < one_bend * 1e-12, "{what}: {volume}");
                    }
                    (Rule::Difference, true, _) => {
                        let measured = measured.unwrap_or_else(|why| panic!("{what}: {why:?}"));
                        let [(volume, error)] = measured[..] else {
                            panic!("{what}: one solid, not {measured:?}");
                        };
                        assert!(
                            (volume - one_bend).abs() <= error,
                            "{what}: {volume} ± {error}, a bend {one_bend}"
                        );
                    }
                    (Rule::Union | Rule::Difference, _, _) => assert!(
                        matches!(
                            measured,
                            Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio }))
                                if ratio == too_wide
                        ),
                        "{what}: {measured:?}"
                    ),
                    (other, _, _) => panic!("not a rule this tries: {other:?}"),
                }
            }
        }
    }
}

/// A quarter of a thick tube about `z`, radii 2 to 6, `z` from −1.5 to 0, under
/// the top half of a ring of 4 round a tube of 1: the quarter's top lies on the
/// half ring's flat face. Where it sits, and moved down 1 and back up 1, every
/// rule both ways round answers and validates.
///
/// Moved up or down by a hair, its top is a plane square to the ring's axis a
/// hair off that flat face. Every rule both ways round is then refused
/// `Undecidable` at `order along a crossing` while the pieces are cut, before
/// any point is put to a ray: for hairs of a rounding, `1.1e-16` up and
/// `5.6e-17` down, and for `1e-12`. Moved a quarter either way, which no
/// rounding is in, it answers, in
/// `a_quarter_of_a_tube_a_quarter_off_half_a_ring_answers_with_its_volume`.
#[test]
fn a_quarter_of_a_tube_a_hair_under_half_a_ring_is_refused_before_a_ray() {
    for moves in [&[][..], &[-1.0, 1.0]] {
        for rule in EVERY_RULE {
            for the_ring_first in [true, false] {
                let what = format!("moved by {moves:?}, {rule:?}, the ring first {the_ring_first}");
                let mut model = a_model();
                let ring =
                    the_top_half_of_a_ring(&mut model, 4.0, 1.0, false).expect("a half ring");
                let quarter = a_quarter_of_a_tube_moved_by(&mut model, moves);
                let solids = fused(&mut model, ring, quarter, rule, the_ring_first, &what);
                let expected = match rule {
                    Rule::Union | Rule::Difference => 1,
                    Rule::Intersection => 0,
                    other => panic!("not a rule this tries: {other:?}"),
                };
                assert_eq!(solids, expected, "{what}");
            }
        }
    }
    for hair in [
        1.110_223_024_625_156_5e-16,
        -5.551_115_123_125_783e-17,
        1e-12,
    ] {
        for rule in EVERY_RULE {
            for the_ring_first in [true, false] {
                let what = format!("moved by {hair:e}, {rule:?}, the ring first {the_ring_first}");
                let mut model = a_model();
                let ring =
                    the_top_half_of_a_ring(&mut model, 4.0, 1.0, false).expect("a half ring");
                let quarter = a_quarter_of_a_tube_moved_by(&mut model, &[hair]);
                let refused = fused_or_refused(&mut model, ring, quarter, rule, the_ring_first);
                expect(refused, Some("order along a crossing"), &what);
            }
        }
    }
}

/// The quarter of a tube in
/// `a_quarter_of_a_tube_a_hair_under_half_a_ring_is_refused_before_a_ray`,
/// `12π`, moved up or down a quarter against the same half ring, `4π²`.
///
/// Moved up, the quarter's top cuts the half ring a quarter above its flat
/// face, and the two share a quarter of the slice of the ring's tube from its
/// middle up a quarter: `2π (h √(1 − h²) + asin h)` at `h = 1/4`, by Pappus,
/// since that slice is a quarter turn of a piece of the tube's disc whose
/// middle is on the ring's circle. Every rule both ways round answers one
/// solid. Moved down, the two are a quarter apart: the union is the two of
/// them, the intersection nothing, and each difference the first named.
///
/// Until Q2118 every one of these was refused `Undecidable` at `order along a
/// crossing` (row Q1577).
#[test]
fn a_quarter_of_a_tube_a_quarter_off_half_a_ring_answers_with_its_volume() {
    let quarter = 12.0 * PI;
    let half_a_ring = 4.0 * PI * PI;
    let h: f64 = 0.25;
    let shared = 2.0 * PI * (h * (1.0 - h * h).sqrt() + h.asin());
    for up in [h, -h] {
        for rule in EVERY_RULE {
            for the_ring_first in [true, false] {
                let what = format!("moved by {up}, {rule:?}, the ring first {the_ring_first}");
                let mut model = a_model();
                let ring =
                    the_top_half_of_a_ring(&mut model, 4.0, 1.0, false).expect("a half ring");
                let tube = a_quarter_of_a_tube_moved_by(&mut model, &[up]);
                let measured = fused_and_measured(&mut model, ring, tube, rule, the_ring_first)
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"));
                let expected = match (rule, up > 0.0, the_ring_first) {
                    (Rule::Union, true, _) => vec![half_a_ring + quarter - shared],
                    (Rule::Union, false, _) => vec![quarter, half_a_ring],
                    (Rule::Intersection, true, _) => vec![shared],
                    (Rule::Intersection, false, _) => vec![],
                    (Rule::Difference, true, true) => vec![half_a_ring - shared],
                    (Rule::Difference, true, false) => vec![quarter - shared],
                    (Rule::Difference, false, true) => vec![half_a_ring],
                    (Rule::Difference, false, false) => vec![quarter],
                    (other, _, _) => panic!("not a rule this tries: {other:?}"),
                };
                each_within_its_error(measured, &expected, &what);
            }
        }
    }
}

/// The block a bend grows out of in `tests/a_point_beside_a_turned_flat_face.rs`
/// under a quarter of a thick tube about `z`, radii 1.5 to 7, in `x >= 0,
/// y <= 0`, `z` from 2 to 3.5: the quarter's bottom lies on the block's top.
/// The quarter's surfaces are two planes through the ring's axis, two square
/// to it and two pipes on it, every one a surface the ring is crossed with.
///
/// Where it sits, and moved up or down `z` by a rounding at 2, by `1e-15` or
/// by `1e-13`, every rule both ways round is refused
/// `ACrossingBetweenAPlaneAndATorusThatIsAQuartic`. No plane of the quarter is
/// such a plane, so the plane refused is one of the block's own sides that are
/// parallel to the ring's axis and not through it.
#[test]
fn a_quarter_of_a_tube_over_a_block_a_bend_grows_out_of_is_refused_for_the_blocks_sides() {
    let a_rounding_at_2 = 4.440_892_098_500_626e-16;
    for moves in [
        &[][..],
        &[a_rounding_at_2],
        &[-a_rounding_at_2],
        &[1e-15],
        &[-1e-15],
        &[1e-13],
        &[-1e-13],
    ] {
        for rule in EVERY_RULE {
            for the_block_first in [true, false] {
                let what =
                    format!("moved by {moves:?}, {rule:?}, the block first {the_block_first}");
                let mut model = a_model();
                let block = a_bend_out_of_a_block(&mut model)
                    .expect("a bend growing out of a block's side");
                let quarter = a_quarter_of_a_tube(
                    &mut model,
                    [[0.0, 0.0], [0.0, -10.0], [10.0, 0.0]],
                    (1.5, 7.0),
                    2.0,
                    moves,
                );
                let refused = fused_or_refused(&mut model, block, quarter, rule, the_block_first);
                assert!(
                    matches!(
                        refused,
                        Err(Error::Geometry(
                            rcad_geom::Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic
                        ))
                    ),
                    "{what}: {refused:?}"
                );
            }
        }
    }
}

/// The top half of a ring of 4 round a tube of 1 joined to the quarter of a
/// tube under it in
/// `a_quarter_of_a_tube_a_hair_under_half_a_ring_is_refused_before_a_ray`,
/// fused with something about `z` from −1.4 to −1.2, which reaches only the
/// quarter: a round bar of radius 0.5, in the hole and touching nothing; one
/// of 2, on the quarter's inner wall, or a rounding or `1e-13` off it; one of
/// 2.5, through that wall; or a bar of 9 less one of 6, round the quarter's
/// outer wall, or a rounding off it.
///
/// Every rule both ways round is refused `CannotTellTheCellsOfAFaceApart`, the
/// bar touching nothing too, so a pipe a hair off a pipe face of this solid is
/// refused before any piece is placed.
#[test]
fn half_a_ring_joined_to_a_quarter_of_a_tube_is_refused_with_a_bar_in_it_or_a_tube_round_it() {
    let a_rounding_at_2 = 4.440_892_098_500_626e-16;
    let a_rounding_at_6 = 8.881_784_197_001_252e-16;
    let a_bar: fn(&mut Model, f64) -> SolidKey = a_thin_bar;
    let a_tube: fn(&mut Model, f64) -> SolidKey = a_thin_tube_with_a_hole_of;
    for (radius, made) in [
        (0.5, a_bar),
        (2.0, a_bar),
        (2.0 + a_rounding_at_2, a_bar),
        (2.0 - a_rounding_at_2 / 2.0, a_bar),
        (2.0 + 1e-13, a_bar),
        (2.5, a_bar),
        (6.0, a_tube),
        (6.0 - a_rounding_at_6, a_tube),
        (6.0 + a_rounding_at_6, a_tube),
    ] {
        for rule in EVERY_RULE {
            for the_ring_first in [true, false] {
                let what = format!("radius {radius:e}, {rule:?}, the ring first {the_ring_first}");
                let mut model = a_model();
                let ring = half_a_ring_joined_to_a_quarter_of_a_tube(&mut model);
                let round_it = made(&mut model, radius);
                let refused = fused_or_refused(&mut model, ring, round_it, rule, the_ring_first);
                assert!(
                    matches!(refused, Err(Error::CannotTellTheCellsOfAFaceApart { .. })),
                    "{what}: {refused:?}"
                );
            }
        }
    }
}

/// A quarter bend and a copy moved a rounding off it, so the copy's ring is a
/// rounding off the first's: refused when the two are crossed, before anything
/// is cut. Moved up the axis by `1e-15` or `1e-12`, the two rings are two
/// curved surfaces nothing here crosses,
/// `ACrossingBetweenATorusAndAnotherCurvedSurface`. Tipped round `x` by `2π`,
/// the copy's axis leans a rounding off `z`, so neither bend's caps lie on a
/// plane through the other's axis, `ACrossingBetweenAPlaneAndATorusThatIsAQuartic`.
/// Turned round `z` by `2π` is a hair round the ring, in
/// `a_bend_placed_a_hair_round_from_another_answers_or_is_refused_before_a_ray`.
#[test]
fn a_bend_and_a_copy_moved_a_rounding_off_it_are_refused_when_crossed() {
    for (moved, refusal) in [
        (
            at_height(1e-15),
            rcad_geom::Error::ACrossingBetweenATorusAndAnotherCurvedSurface,
        ),
        (
            at_height(1e-12),
            rcad_geom::Error::ACrossingBetweenATorusAndAnotherCurvedSurface,
        ),
        (
            Isometry::rotation(&Axis::x(), 2.0 * PI).expect("a real angle"),
            rcad_geom::Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic,
        ),
    ] {
        for rule in EVERY_RULE {
            for the_first_first in [true, false] {
                let what = format!("{moved:?}, {rule:?}, the first first {the_first_first}");
                let mut model = a_model();
                let first = a_quarter_bend(&mut model, 4.0, 1.0);
                let copy = a_quarter_bend(&mut model, 4.0, 1.0);
                model
                    .place_solid(copy, &moved)
                    .expect("this model made the key");
                let refused = fused_or_refused(&mut model, first, copy, rule, the_first_first);
                assert!(
                    matches!(&refused, Err(Error::Geometry(why)) if *why == refusal),
                    "{what}: expected {refusal:?}, got {refused:?}"
                );
            }
        }
    }
}

/// A wedge of a round bar, between a plane through the bend's axis a hair
/// below its cap `y = 0` and the plane `y = −x`, closed by the bar's wall of
/// radius 6 and the planes `z = ±2`: every surface one the ring is crossed
/// with, so the wedge's face a hair off the cap would be the piece whose
/// points sit a rounding off the bend.
///
/// The wedge itself, the prism less what is outside the bar, validates and
/// measures `18π − 72 atan(hair / 10)` within the measure's own error for
/// every hair from `1e-3` down to `1e-20`; until Q1817 it was refused
/// `Undecidable` at `order along a crossing` from `1e-15` down. The wedge's
/// face is not stored on a plane through the axis, so the fuse is refused
/// `ACrossingBetweenAPlaneAndATorusThatIsAQuartic` at every hair, at a reach
/// of ten.
#[test]
fn a_wedge_a_hair_below_a_bends_cap_is_refused_before_a_ray() {
    let policy = Policy::certified();
    for hair in [1e-3, 1e-9, 1e-12, 1e-14, 1e-15, 1e-16, 1e-20] {
        let mut model = a_model();
        let wedge = a_wedge(&mut model, hair)
            .unwrap_or_else(|why| panic!("a hair of {hair:e}: the wedge {why:?}"));
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("a hair of {hair:e}: {why:?}"));
        let measured = model
            .solid(wedge)
            .expect("this model made the key")
            .measure(
                rcad_geom::Deflection::of(0.01).expect("a deflection"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("a hair of {hair:e}: {why:?}"));
        let expected = 18.0 * PI - 72.0 * (hair / 10.0).atan();
        assert!(
            (measured.volume() - expected).abs() <= measured.volume_error(),
            "a hair of {hair:e}: {} ± {}, against {expected}",
            measured.volume(),
            measured.volume_error()
        );
        for rule in EVERY_RULE {
            for the_bend_first in [true, false] {
                let what = format!("a hair of {hair:e}, {rule:?}, the bend first {the_bend_first}");
                let mut model = a_model();
                let bend = a_quarter_bend(&mut model, 4.0, 1.0);
                let wedge = a_wedge(&mut model, hair)
                    .unwrap_or_else(|why| panic!("{what}: the wedge {why:?}"));
                let refused = fused_or_refused(&mut model, bend, wedge, rule, the_bend_first);
                assert!(
                    matches!(
                        refused,
                        Err(Error::Geometry(
                            rcad_geom::Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic
                        ))
                    ),
                    "{what}: {refused:?}"
                );
            }
        }
    }
}

/// A quarter bend of ring 4 round a tube of 1, and the frustum of the cone
/// `ρ = (z + 8) / 2` from `z = −2` to `2`: a cone on the ring's axis, crossing
/// its tube. Joined, they would be a ring solid with a cone face, for a piece
/// of a cone a hair off that face to be put to a ray against.
///
/// They are never joined. Every rule both ways round is refused `Undecidable`
/// at `order along a crossing` while the pieces are cut. The frustum is turned
/// `5π/4` round the axis, so its straight edge is clear of the bend.
///
/// Until Q2118 this pinned a ball about the origin of 3.5 or 4.5 too, as
/// `a_bend_and_a_ball_or_a_cone_on_its_axis_are_refused_while_they_are_cut`;
/// those answer now, in
/// `a_bend_and_a_ball_on_its_axis_answer_every_rule_with_their_volumes`.
#[test]
fn a_bend_and_a_cone_on_its_axis_are_refused_while_they_are_cut() {
    let mut not_as_pinned = Vec::new();
    for rule in EVERY_RULE {
        for the_bend_first in [true, false] {
            let mut model = a_model();
            let bend = a_quarter_bend(&mut model, 4.0, 1.0);
            let frustum = a_frustum_placed(&mut model, 0.5, 6.0, 10.0, -8.0, 5.0 * PI / 4.0);
            let fused = fused_or_refused(&mut model, bend, frustum, rule, the_bend_first);
            let said = the_refusal(&fused);
            if said != "Undecidable at order along a crossing" {
                not_as_pinned.push(format!("{rule:?}, the bend first {the_bend_first}: {said}"));
            }
        }
    }
    assert_eq!(not_as_pinned, Vec::<String>::new());
}

/// A quarter bend of ring 4 round a tube of 1, `2π²`, and a ball about the
/// origin of 3.5 or 4.5, turned `5π/4` round the axis so its seam is clear of
/// the bend: a ball on the ring's axis, crossing its tube. Every rule both
/// ways round answers one solid, whose volume is the closed form's, with the
/// part of the bend inside the ball from `how_much_of_the_bend_is_in_a_ball`.
///
/// Until Q2118 every one was refused `Undecidable` at `order along a crossing`
/// while the pieces were cut (row Q1584): for the union with the ball of 3.5,
/// the bend named first, at the two points where the circle the ball crosses
/// the ring in meets the bend's two caps, `(3.406, 0, −0.805)` and
/// `(0, 3.406, −0.805)`.
#[test]
fn a_bend_and_a_ball_on_its_axis_answer_every_rule_with_their_volumes() {
    let bend = 2.0 * PI * PI;
    for radius in [3.5, 4.5] {
        let inside = how_much_of_the_bend_is_in_a_ball(radius);
        let by_slices = how_much_of_the_bend_is_in_a_ball_by_slices(radius);
        assert!(
            (inside - by_slices).abs() <= inside * 1e-6,
            "a ball of {radius}: {inside} closed, {by_slices} by slices"
        );
        let ball = 4.0 * PI * radius.powi(3) / 3.0;
        for rule in EVERY_RULE {
            for the_bend_first in [true, false] {
                let what = format!("a ball of {radius}, {rule:?}, the bend first {the_bend_first}");
                let mut model = a_model();
                let quarter = a_quarter_bend(&mut model, 4.0, 1.0);
                let round = model
                    .add_ball(radius, &turned(5.0 * PI / 4.0))
                    .expect("a ball");
                let measured = fused_and_measured(&mut model, quarter, round, rule, the_bend_first)
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"));
                let expected = match (rule, the_bend_first) {
                    (Rule::Union, _) => bend + ball - inside,
                    (Rule::Intersection, _) => inside,
                    (Rule::Difference, true) => bend - inside,
                    (Rule::Difference, false) => ball - inside,
                    (other, _) => panic!("not a rule this tries: {other:?}"),
                };
                each_within_its_error(measured, &[expected], &what);
            }
        }
    }
}

/// The volume of the quarter bend of ring 4 round a tube of 1 that is inside a
/// ball about the origin of `radius`, from 3 to 5.
///
/// In `(ρ, z)` the tube is the disc of radius 1 about `(4, 0)`, and the ball's
/// skin crosses it at `ρ = 4 + c`, `c = (radius² − 17) / 8`, an angle `φ =
/// acos c` round the disc from its outer side. The part of the disc inside the
/// ball, each piece of it times its `ρ`, sums to `4 (c sin φ + π − φ)`, and a
/// quarter turn of it is `π/2` of that, by Pappus.
fn how_much_of_the_bend_is_in_a_ball(radius: f64) -> f64 {
    assert!(
        (3.0..=5.0).contains(&radius),
        "a ball whose skin crosses the tube: {radius}"
    );
    let c = (radius * radius - 17.0) / 8.0;
    let phi = c.acos();
    2.0 * PI * (c * phi.sin() + PI - phi)
}

/// `how_much_of_the_bend_is_in_a_ball` summed over thin slices across `z`, as
/// a check on its closed form.
fn how_much_of_the_bend_is_in_a_ball_by_slices(radius: f64) -> f64 {
    let slices = 200_000_u32;
    let thick = 2.0 / f64::from(slices);
    let each_piece_times_its_reach: f64 = (0..slices)
        .map(|slice| {
            let z = -1.0 + (f64::from(slice) + 0.5) * thick;
            let half_across = (1.0 - z * z).sqrt();
            let inner = 4.0 - half_across;
            let outer = (4.0 + half_across).min((radius * radius - z * z).sqrt());
            (outer * outer - inner * inner).max(0.0) / 2.0 * thick
        })
        .sum();
    FRAC_PI_2 * each_piece_times_its_reach
}

/// A ball of 5 joined to a ring round it, and a frustum of a cone joined to a
/// ring, both about `z` and built face by face with no seam: every edge is a
/// whole circle, so neither has a plane through its axis for a ball or a cone
/// to be crossed with. Against each, a piece of its own kind on it: the ball of
/// 5 above `z = 3`, whose cap lies on the ring solid's upper ball face, and the
/// frustum of the same cone from `z = 2` to `5`, whose wall lies on the ring
/// solid's upper cone face. Each piece's bottom rim is inside the ring's tube.
///
/// None reaches a ray. Where the piece sits, every rule both ways round is
/// refused `ResultIsNotManifold`, and so is the cone's piece moved up or down
/// `z` by a rounding; the counts are pinned, the ring solid named first and
/// then second (row Q1758). The ball's piece moved by a rounding, and either
/// piece moved by `1e-13`, is refused `Undecidable` at `side of sphere` or
/// `side of cone`; with the ring solid named second it is at the piece's
/// bottom rim (row Q1759). Moved `1e-13` off the axis along `x`, the piece's
/// ball is one the ring is not crossed with, and its cone one the ring solid's
/// cone is not crossed with.
///
/// Until Q1566, every case on the axis but the ones moved `1e-13` with the ring
/// solid named second was refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`:
/// a plane square to a ring's axis, the ring, and a ball or a cone on that
/// axis was a meet of three `rcad-geom` had no road for.
/// `what_a_plane_square_to_a_rings_axis_leaves` is that road now (§428.5), and
/// with it alone, the join in `the_name_each_closed_curve_goes_by` narrowed
/// back to flat-picked circles, these are refused as pinned here. Run again
/// with `settle_against_each`'s arm made to print, they printed nothing.
///
/// A whole ball about the origin, of 5 and a rounding or `5 ± 1e-13`, against
/// the ball joined to a ring is refused `Undecidable` at `side of sphere` by
/// every rule both ways round, at `(0, 3, −4)` on the lower circle the ball
/// and the ring cross in.
#[test]
fn a_ball_or_a_cone_joined_to_a_ring_with_a_piece_of_its_own_kind_on_it_is_refused_before_a_ray() {
    let a_rounding = 8.881_784_197_001_252e-16;
    let a_ball: fn(&mut Model) -> Result<SolidKey, Error> = a_ball_joined_to_a_ring;
    let a_cone: fn(&mut Model) -> Result<SolidKey, Error> = a_frustum_joined_to_a_ring;
    let mut not_as_pinned = Vec::new();
    for (kind, ring_solid, piece, its_side, moved_to_its_side, not_a_manifold, off_the_axis) in [
        (
            "ball",
            a_ball,
            a_ball_segment as fn(&mut Model) -> Result<SolidKey, Error>,
            "Undecidable at side of sphere",
            &["up a rounding", "down a rounding", "up 1e-13", "down 1e-13"][..],
            [(3, 1), (4, 3), (4, 1)],
            "ACrossingBetweenATorusAndAnotherCurvedSurface",
        ),
        (
            "cone",
            a_cone,
            a_seamless_frustum,
            "Undecidable at side of cone",
            &["up 1e-13", "down 1e-13"][..],
            [(1, 1), (3, 3), (3, 1)],
            "ACrossingBetweenAConeAndAnotherCurvedSurface",
        ),
    ] {
        for (moved, by) in [
            ("where it sits", Isometry::identity()),
            ("up a rounding", at_height(a_rounding)),
            ("down a rounding", at_height(-a_rounding)),
            ("up 1e-13", at_height(1e-13)),
            ("down 1e-13", at_height(-1e-13)),
            (
                "off the axis",
                Isometry::translation(Vector::new(1e-13, 0.0, 0.0)).expect("a finite shift"),
            ),
        ] {
            for (rule, (counted_ring_first, counted_ring_second)) in
                EVERY_RULE.into_iter().zip(not_a_manifold)
            {
                for the_ring_first in [true, false] {
                    let mut model = a_model();
                    let ring = ring_solid(&mut model).expect("a ring solid");
                    let on_it = piece(&mut model).expect("a piece of its own kind");
                    model
                        .place_solid(on_it, &by)
                        .expect("this model made the key");
                    let fused = fused_or_refused(&mut model, ring, on_it, rule, the_ring_first);
                    let said = the_refusal(&fused);
                    let count = if the_ring_first {
                        counted_ring_first
                    } else {
                        counted_ring_second
                    };
                    let pinned = if moved == "off the axis" {
                        off_the_axis.to_owned()
                    } else if moved_to_its_side.contains(&moved) {
                        its_side.to_owned()
                    } else {
                        format!("Err(ResultIsNotManifold {{ count: {count} }})")
                    };
                    if said != pinned {
                        not_as_pinned.push(format!(
                            "{kind} {moved}, {rule:?}, the ring first {the_ring_first}: {said}"
                        ));
                    }
                }
            }
        }
    }
    for radius in [5.0 + a_rounding, 5.0 - a_rounding, 5.0 + 1e-13, 5.0 - 1e-13] {
        for rule in EVERY_RULE {
            for the_ring_first in [true, false] {
                let mut model = a_model();
                let ring = a_ball_joined_to_a_ring(&mut model).expect("a ring solid");
                let ball = model
                    .add_ball(radius, &Isometry::identity())
                    .expect("a ball");
                let fused = fused_or_refused(&mut model, ring, ball, rule, the_ring_first);
                let said = the_refusal(&fused);
                if said != "Undecidable at side of sphere" {
                    not_as_pinned.push(format!(
                        "a whole ball of {radius:e}, {rule:?}, the ring first {the_ring_first}: \
                         {said}"
                    ));
                }
            }
        }
    }
    assert_eq!(not_as_pinned, Vec::<String>::new());
}

/// A ball of 5 about the origin joined to a ring of 6 round a tube of 5 about
/// `z`: they cross in the circles through `(3, 0, ±4)`, both exact.
///
/// Its two circles are walked `Forward` by the face above them, and it
/// measures `3060 ± 35` at `0.03` against the profile's `3063`. Walked
/// `Reversed` it also builds and validates, but measures `941.6`.
fn a_ball_joined_to_a_ring(model: &mut Model) -> Result<SolidKey, Error> {
    let ball = Surface::Sphere(rcad_geom::Sphere::new(Point::origin(), 5.0).expect("a ball"));
    let ring = Surface::Torus(Torus::new(Axis::z(), 6.0, 5.0).expect("a ring"));
    a_solid_of_revolution(
        model,
        &[(3.0, -4.0), (3.0, 4.0)],
        &[ball, ring, ball],
        &[Orientation::Forward, Orientation::Forward],
    )
}

/// The cone `ρ = 9 − z` from `z = −6` to `7`, closed by two flat faces, joined
/// to a ring of 10 round a tube of 5 about `z`: the cone crosses the ring in
/// the circles through `(13, 0, −4)` and `(6, 0, 3)`.
fn a_frustum_joined_to_a_ring(model: &mut Model) -> Result<SolidKey, Error> {
    let cone = the_cone_down_from_9();
    let ring = Surface::Torus(Torus::new(Axis::z(), 10.0, 5.0).expect("a ring"));
    a_solid_of_revolution(
        model,
        &[(15.0, -6.0), (13.0, -4.0), (6.0, 3.0), (2.0, 7.0)],
        &[flat_at(-6.0, -1.0), cone, ring, cone, flat_at(7.0, 1.0)],
        &[Orientation::Reversed; 4],
    )
}

fn the_cone_down_from_9() -> Surface {
    Surface::Cone(
        rcad_geom::Cone::new(
            Axis::new(Point::new(0.0, 0.0, 9.0), Vector::new(0.0, 0.0, -1.0)).expect("an axis"),
            1.0,
        )
        .expect("a cone"),
    )
}

/// The ball of 5 about the origin above `z = 3`.
fn a_ball_segment(model: &mut Model) -> Result<SolidKey, Error> {
    let ball = Surface::Sphere(rcad_geom::Sphere::new(Point::origin(), 5.0).expect("a ball"));
    a_solid_of_revolution(
        model,
        &[(4.0, 3.0)],
        &[flat_at(3.0, -1.0), ball],
        &[Orientation::Forward],
    )
}

/// The cone `ρ = 9 − z` from `z = 2` to `5`.
fn a_seamless_frustum(model: &mut Model) -> Result<SolidKey, Error> {
    a_solid_of_revolution(
        model,
        &[(7.0, 2.0), (4.0, 5.0)],
        &[
            flat_at(2.0, -1.0),
            the_cone_down_from_9(),
            flat_at(5.0, 1.0),
        ],
        &[Orientation::Reversed, Orientation::Reversed],
    )
}

fn flat_at(z: f64, facing: f64) -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, facing))
            .expect("a unit normal"),
    )
}

/// A solid of revolution about `z` with no seam: `corners` are its profile's
/// corners `(ρ, z)` from the bottom of the axis to the top, each one circle,
/// and `surfaces` the faces between them, one more than the corners. `turns`
/// is how the face above each circle walks it, and the face below walks it the
/// other way.
///
/// Which walk builds was found by trying each: the ball's circles beside the
/// ring `Forward`, and every circle on the cone `ρ = 9 − z` `Reversed`, the
/// cone's circles beside the ring as well as those beside its flat faces.
fn a_solid_of_revolution(
    model: &mut Model,
    corners: &[(f64, f64)],
    surfaces: &[Surface],
    turns: &[Orientation],
) -> Result<SolidKey, Error> {
    let mut builder = SolidBuilder::new();
    let mut circles = Vec::with_capacity(corners.len());
    for (index, &(rho, z)) in corners.iter().enumerate() {
        let corner = builder.add_vertex(Point::new(rho, 0.0, z))?;
        let by_a_ring = matches!(surfaces[index], Surface::Torus(_))
            || matches!(surfaces[index + 1], Surface::Torus(_));
        let run = if by_a_ring {
            EdgeRun::RightRoundWhereItCrosses {
                and: flat_at(z, 1.0),
            }
        } else {
            EdgeRun::RightRoundWhereItsFacesCross
        };
        circles.push(builder.add_edge(corner, corner, run)?);
    }
    let mut shell = Vec::with_capacity(surfaces.len());
    for (index, &surface) in surfaces.iter().enumerate() {
        let mut wires = Vec::with_capacity(2);
        if index > 0 {
            let coedge = builder.add_coedge(circles[index - 1], turns[index - 1])?;
            wires.push(builder.add_wire(&[DraftStep::Along(coedge)])?);
        }
        if index < circles.len() {
            let coedge = builder.add_coedge(circles[index], turns[index].reversed())?;
            wires.push(builder.add_wire(&[DraftStep::Along(coedge)])?);
        }
        shell.push((builder.add_face(surface, &wires)?, Orientation::Forward));
    }
    builder.add_shell(&shell)?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// What a fuse was refused: the last record's predicate for `Undecidable`, the
/// refusal's own name for a geometry one.
fn the_refusal(fused: &Result<usize, Error>) -> String {
    match fused {
        Err(Error::Undecidable { records }) => format!(
            "Undecidable at {}",
            records
                .records()
                .last()
                .map_or("no record", |record| record.predicate())
        ),
        Err(Error::Geometry(why)) => format!("{why:?}"),
        other => format!("{other:?}"),
    }
}

fn turned(turn: f64) -> Isometry {
    Isometry::rotation(&Axis::z(), turn).expect("a real angle")
}

fn a_frustum_placed(
    model: &mut Model,
    slope: f64,
    near: f64,
    far: f64,
    down: f64,
    turn: f64,
) -> SolidKey {
    let skin = rcad_geom::Cone::new(Axis::z(), slope).expect("a positive slope");
    let mut builder = SolidBuilder::new();
    let at = |height: f64| Point::new(slope * height, 0.0, height);
    let bottom_corner = builder.add_vertex(at(near)).expect("a finite point");
    let top_corner = builder.add_vertex(at(far)).expect("a finite point");
    let seam = builder
        .add_edge(bottom_corner, top_corner, EdgeRun::Straight)
        .expect("a ruling");
    let bottom_circle = builder
        .add_edge(
            bottom_corner,
            bottom_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a circle");
    let top_circle = builder
        .add_edge(
            top_corner,
            top_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a circle");
    let cap = |builder: &mut SolidBuilder, at: f64, outward: f64, edge, turn| {
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, at), Vector::new(0.0, 0.0, outward))
                .expect("a unit normal");
        let coedge = builder.add_coedge(edge, turn).expect("an edge");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("a wire");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a cap")
    };
    let bottom = cap(
        &mut builder,
        near,
        -1.0,
        bottom_circle,
        Orientation::Reversed,
    );
    let top = cap(&mut builder, far, 1.0, top_circle, Orientation::Forward);
    let round_the_wall = [
        (bottom_circle, Orientation::Forward),
        (seam, Orientation::Forward),
        (top_circle, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ]
    .map(|(edge, turn)| DraftStep::Along(builder.add_coedge(edge, turn).expect("an edge")));
    let wire = builder.add_wire(&round_the_wall).expect("a wire");
    let side = builder
        .add_face(Surface::Cone(skin), &[wire])
        .expect("a face on the cone");
    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("three faces close");
    let frustum = builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a frustum");
    model
        .place_solid(frustum, &turned(turn))
        .expect("this model made the key");
    model
        .place_solid(frustum, &at_height(down))
        .expect("this model made the key");
    frustum
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn a_quarter_bend_placed_round_by(model: &mut Model, ring: f64, tube: f64, turn: f64) -> SolidKey {
    let bend = a_quarter_bend(model, ring, tube);
    model
        .place_solid(
            bend,
            &Isometry::rotation(&Axis::z(), turn).expect("a real angle"),
        )
        .expect("this model made the key");
    bend
}

/// The quarter of a round bar of radius 6 about `z` in `x, y >= 0`, `z` from
/// −2 to 2: the prism over the triangle `(0, 0)`, `(12, 0)`, `(0, 12)` less
/// what is outside the bar.
fn a_quarter_of_a_bar(model: &mut Model) -> SolidKey {
    let policy = Policy::certified();
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [12.0, 0.0], [0.0, 12.0]])
        .build(&policy, &mut ProvenanceLog::new())
        .expect("a triangle");
    let prism = model
        .add_extrusion(&triangle, 4.0, &at_height(-2.0))
        .expect("a prism");
    let bar = model
        .add_round_bar(6.0, 6.0, &at_height(-3.0))
        .expect("a round bar");
    let answer = model
        .fuse(
            &[prism],
            &[bar],
            Rule::Intersection,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a prism and a bar on its corner answer");
    let &[quarter] = answer.solids() else {
        panic!("one quarter, got {:?}", answer.solids());
    };
    quarter
}

/// The top half of a ring of 4 round a tube of 1, joined to the quarter of a
/// tube under it whose top lies on the half ring's flat face.
fn half_a_ring_joined_to_a_quarter_of_a_tube(model: &mut Model) -> SolidKey {
    let half = the_top_half_of_a_ring(model, 4.0, 1.0, false).expect("a half ring");
    let quarter = a_quarter_of_a_tube_moved_by(model, &[]);
    let answer = model
        .fuse(
            &[half],
            &[quarter],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a half ring and the quarter of a tube under it answer");
    let &[ring] = answer.solids() else {
        panic!("one solid, got {:?}", answer.solids());
    };
    ring
}

/// The quarter of a thick tube about `z`, radii 2 to 6, in `x, y >= 0`, `z`
/// from −1.5 to 0, then placed up `z` by each of `moves` in turn.
///
/// The quarter is built where it sits and moved after: a prism built a hair
/// off `z = 0`, with its bottom at −0.3 or −0.5, was refused `order along a
/// crossing` while the quarter itself was cut.
fn a_quarter_of_a_tube_moved_by(model: &mut Model, moves: &[f64]) -> SolidKey {
    a_quarter_of_a_tube(
        model,
        [[0.0, 0.0], [10.0, 0.0], [0.0, 10.0]],
        (2.0, 6.0),
        -1.5,
        moves,
    )
}

/// The prism over `triangle`, with its corner on the `z` axis, from `bottom`
/// to 1.5 above it, less what is inside a round bar about `z` of radius
/// `inside` and what is outside one of radius `outside`, then placed up `z` by
/// each of `moves` in turn.
fn a_quarter_of_a_tube(
    model: &mut Model,
    triangle: [[f64; 2]; 3],
    (inside, outside): (f64, f64),
    bottom: f64,
    moves: &[f64],
) -> SolidKey {
    let policy = Policy::certified();
    let triangle = ProfileBuilder::new(triangle.to_vec())
        .build(&policy, &mut ProvenanceLog::new())
        .expect("a triangle");
    let prism = model
        .add_extrusion(&triangle, 1.5, &at_height(bottom))
        .expect("a prism");
    let outer = model
        .add_round_bar(outside, 6.0, &at_height(bottom - 1.5))
        .expect("a round bar");
    let inner = model
        .add_round_bar(inside, 6.0, &at_height(bottom - 1.5))
        .expect("a round bar");
    let answer = model
        .fuse(
            &[prism],
            &[outer],
            Rule::Intersection,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a prism and a bar on its corner answer");
    let &[quarter] = answer.solids() else {
        panic!("one quarter of a bar, got {:?}", answer.solids());
    };
    let answer = model
        .fuse(
            &[quarter],
            &[inner],
            Rule::Difference,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a quarter of a bar less a thinner bar answers");
    let &[quarter] = answer.solids() else {
        panic!("one quarter of a tube, got {:?}", answer.solids());
    };
    for &up in moves {
        model
            .place_solid(quarter, &at_height(up))
            .expect("this model made the key");
    }
    quarter
}

/// The block of `tests/a_point_beside_a_turned_flat_face.rs`, `[2, 6] × [−2, 0]
/// × [−2, 2]`, with a quarter bend of ring four and tube one growing out of its
/// side `y = 0`, unturned. That side holds the bend's first cap circle as a
/// hole, walked the way round that builds; the bend's wall walks it the other
/// way. The other five sides are rectangles.
fn a_bend_out_of_a_block(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let mut corners = BTreeMap::new();
    for x in [2, 6] {
        for y in [-2, 0] {
            for z in [-2, 2] {
                let at = Point::new(f64::from(x), f64::from(y), f64::from(z));
                corners.insert([x, y, z], builder.add_vertex(at)?);
            }
        }
    }
    let start_corner = builder.add_vertex(Point::new(3.0, 0.0, 0.0))?;
    let end_corner = builder.add_vertex(Point::new(0.0, 3.0, 0.0))?;
    let flat_through_the_middle =
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let seam = builder.add_edge(
        start_corner,
        end_corner,
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(flat_through_the_middle),
            picked_by: smallvec![
                Picker::InsideAPipe(Cylinder::new(Axis::z(), 4.0).expect("the ring's radius")),
                Picker::from(
                    HalfSpace::new(Vector::new(1.0, 1.0, 0.0), 0.0)
                        .expect("a normal with a length")
                ),
            ],
        },
    )?;
    let cap_circle = builder.add_edge(
        start_corner,
        start_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let end_circle = builder.add_edge(
        end_corner,
        end_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;

    let mut straight = BTreeMap::new();
    let mut shell = Vec::with_capacity(8);
    for (normal, round) in [
        ([0, 1, 0], [[2, 0, -2], [2, 0, 2], [6, 0, 2], [6, 0, -2]]),
        (
            [0, -1, 0],
            [[2, -2, -2], [6, -2, -2], [6, -2, 2], [2, -2, 2]],
        ),
        ([1, 0, 0], [[6, -2, -2], [6, 0, -2], [6, 0, 2], [6, -2, 2]]),
        ([-1, 0, 0], [[2, -2, -2], [2, -2, 2], [2, 0, 2], [2, 0, -2]]),
        ([0, 0, 1], [[2, -2, 2], [6, -2, 2], [6, 0, 2], [2, 0, 2]]),
        (
            [0, 0, -1],
            [[2, -2, -2], [2, 0, -2], [6, 0, -2], [6, -2, -2]],
        ),
    ] {
        let mut steps = Vec::with_capacity(4);
        for position in 0..4 {
            let from = round[position];
            let to = round[(position + 1) % 4];
            let (edge, turn) = if let Some(&edge) = straight.get(&(to, from)) {
                (edge, Orientation::Reversed)
            } else {
                let edge = builder.add_edge(corners[&from], corners[&to], EdgeRun::Straight)?;
                straight.insert((from, to), edge);
                (edge, Orientation::Forward)
            };
            steps.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
        }
        let mut wires = vec![builder.add_wire(&steps)?];
        if normal == [0, 1, 0] {
            let round_the_cap = builder.add_coedge(cap_circle, Orientation::Forward)?;
            wires.push(builder.add_wire(&[DraftStep::Along(round_the_cap)])?);
        }
        let [x, y, z] = normal.map(f64::from);
        let on_it = round[0].map(f64::from);
        let plane = Plane::from_point_and_normal(
            Point::new(on_it[0], on_it[1], on_it[2]),
            Vector::new(x, y, z),
        )
        .expect("a coordinate direction is a unit normal");
        shell.push((
            builder.add_face(Surface::Plane(plane), &wires)?,
            Orientation::Forward,
        ));
    }

    let end_cap = {
        let plane = Plane::from_point_and_normal(Point::origin(), Vector::new(-1.0, 0.0, 0.0))
            .expect("a coordinate direction is a unit normal");
        let coedge = builder.add_coedge(end_circle, Orientation::Reversed)?;
        let wire = builder.add_wire(&[DraftStep::Along(coedge)])?;
        builder.add_face(Surface::Plane(plane), &[wire])?
    };
    let mut round_the_wall = Vec::with_capacity(4);
    for (edge, turn) in [
        (cap_circle, Orientation::Reversed),
        (seam, Orientation::Forward),
        (end_circle, Orientation::Forward),
        (seam, Orientation::Reversed),
    ] {
        round_the_wall.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
    }
    let wire = builder.add_wire(&round_the_wall)?;
    let ring = Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring");
    let wall = builder.add_face(Surface::Torus(ring), &[wire])?;
    shell.push((end_cap, Orientation::Forward));
    shell.push((wall, Orientation::Forward));
    builder.add_shell(&shell)?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// A round bar about `z` of radius 9, `z` from −1.4 to −1.2, less one of
/// radius `hole`.
fn a_thin_tube_with_a_hole_of(model: &mut Model, hole: f64) -> SolidKey {
    let bar = a_thin_bar(model, 9.0);
    let core = model
        .add_round_bar(hole, 1.0, &at_height(-2.0))
        .expect("a round bar");
    let answer = model
        .fuse(
            &[bar],
            &[core],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar less a thinner, taller bar answers");
    let &[tube] = answer.solids() else {
        panic!("one tube, got {:?}", answer.solids());
    };
    tube
}

/// A round bar about `z` of `radius`, `z` from −1.4 to −1.2.
fn a_thin_bar(model: &mut Model, radius: f64) -> SolidKey {
    model
        .add_round_bar(radius, 0.2, &at_height(-1.4))
        .expect("a round bar")
}

/// The prism over the triangle `(0, 0)`, `(10, −10)`, `(10, −hair)` from
/// `z = −2` to `2`, less what is outside a round bar of radius 6 on the `z`
/// axis.
fn a_wedge(model: &mut Model, hair: f64) -> Result<SolidKey, Error> {
    let policy = Policy::certified();
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, -10.0], [10.0, -hair]])
        .build(&policy, &mut ProvenanceLog::new())
        .expect("a triangle");
    let prism = model
        .add_extrusion(&triangle, 4.0, &at_height(-2.0))
        .expect("a prism");
    let bar = model
        .add_round_bar(6.0, 6.0, &at_height(-3.0))
        .expect("a round bar");
    let answer = model.fuse(
        &[prism],
        &[bar],
        Rule::Intersection,
        &policy,
        &mut ProvenanceLog::new(),
    )?;
    let &[wedge] = answer.solids() else {
        panic!("one wedge, got {:?}", answer.solids());
    };
    Ok(wedge)
}

fn at_height(z: f64) -> Isometry {
    Isometry::translation(Vector::new(0.0, 0.0, z)).expect("a finite shift")
}

/// How many solids the fuse answers with, once the model it leaves validates.
fn fused(
    model: &mut Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
    one_first: bool,
    what: &str,
) -> usize {
    let count = fused_or_refused(model, one, other, rule, one_first)
        .unwrap_or_else(|why| panic!("{what}: {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: {why:?}"));
    count
}

fn fused_or_refused(
    model: &mut Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
    one_first: bool,
) -> Result<usize, Error> {
    let (arguments, tools) = if one_first {
        ([one], [other])
    } else {
        ([other], [one])
    };
    model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().len())
}

/// The volume of each solid the fuse answers with and the measure's own error
/// on it, once the model it leaves validates, or the fuse's refusal.
fn fused_and_measured(
    model: &mut Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
    one_first: bool,
) -> Result<Vec<(f64, f64)>, Error> {
    let (arguments, tools) = if one_first {
        ([one], [other])
    } else {
        ([other], [one])
    };
    let policy = Policy::certified();
    let answer = model.fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())?;
    model.validate(&policy, &mut ProvenanceLog::new())?;
    let deflection = rcad_geom::Deflection::of(0.01).expect("a deflection");
    answer
        .solids()
        .iter()
        .map(|&solid| {
            let measured =
                model
                    .solid(solid)?
                    .measure(deflection, &policy, &mut ProvenanceLog::new())?;
            Ok((measured.volume(), measured.volume_error()))
        })
        .collect()
}

/// That the fuse answered one solid for each of `expected`, and that each
/// volume, smallest first, is within the measure's own error of the one of
/// `expected` in the same place, smallest first.
fn each_within_its_error(mut measured: Vec<(f64, f64)>, expected: &[f64], what: &str) {
    assert_eq!(
        measured.len(),
        expected.len(),
        "{what}: {measured:?} against {expected:?}"
    );
    measured.sort_by(|one, other| one.0.total_cmp(&other.0));
    let mut expected = expected.to_vec();
    expected.sort_by(f64::total_cmp);
    for (&(volume, error), &closed) in measured.iter().zip(&expected) {
        assert!(
            (volume - closed).abs() <= error,
            "{what}: {volume} ± {error}, against {closed}"
        );
    }
}

/// The same two bends placed round by `−3.1 + 0.0155 · step` and a quarter
/// turn back overlap in a wedge under `1e-15`. Their intersection's torus face
/// lies between two meridian circles a rounding apart round the ring. Until
/// Q2265 the mesher read the circles' middles inverted at some deflections,
/// moved the second circle a whole turn, and cut the face the long way round:
/// at steps 58, 173 and 196 the fuse answered, `Model::validate` passed, and
/// the measure gave about `78.9`, a whole ring, for the wedge (Q2138, Q2149).
/// The mesher now orders the two circles round the ring by the planes through
/// the axis they lie on, exactly.
///
/// At steps 173 and 196 the intersection both ways round answers the sliver,
/// and the measure holds it against its wedge at every deflection from `0.32`
/// to `1e-3`. From Q2265 until Q2272 the fuse refused step 196
/// [`Invalid::VolumeIsNotPositive`] on a sum of `−5.2e-16`, where the caps,
/// read exactly, overlap in a wedge of `5.9e-19`. The rung that passed that
/// sum left a flat face's own corners out of what it charged for points off
/// the face's plane, and the copy's corner lies `6.4e-16` off its own cap's
/// plane. Charged for it, no rung answers, and the sum moved onto the planes
/// does. Step 58 was refused [`Error::CannotTellWhichWayASolidsFacesPoint`]
/// from Q2265 until Q2266, and is answered in
/// `two_bends_overlapping_in_a_sliver_thinner_than_a_mesh_can_tell_are_answered`.
#[test]
fn two_bends_overlapping_in_a_sliver_are_never_meshed_right_round_the_ring() {
    let policy = Policy::certified();
    let at = |step: f64| -3.1 + 0.0155 * step;
    for (step, the_first_first) in [(173.0, true), (173.0, false), (196.0, true), (196.0, false)] {
        let what = format!("step {step}, the first first {the_first_first}");
        let start = at(step);
        let mut model = a_model();
        let first = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, start);
        let copy = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, start - FRAC_PI_2);
        let wedge = 4.0 * PI * the_angle_where_the_caps_overlap(&model, [first, copy], start);
        let (arguments, tools) = if the_first_first {
            ([first], [copy])
        } else {
            ([copy], [first])
        };
        let answer = model
            .fuse(
                &arguments,
                &tools,
                Rule::Intersection,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let [solid] = answer.solids()[..] else {
            panic!("{what}: one solid, not {:?}", answer.solids());
        };
        for size in [
            0.32, 0.16, 0.08, 0.04, 0.02, 0.01, 5e-3, 2.5e-3, 1.25e-3, 1e-3,
        ] {
            let deflection = rcad_geom::Deflection::of(size).expect("a deflection");
            let (volume, error) = model
                .solid(solid)
                .and_then(|solid| solid.measure(deflection, &policy, &mut ProvenanceLog::new()))
                .map(|measured| (measured.volume(), measured.volume_error()))
                .unwrap_or_else(|why| panic!("{what}, deflection {size}: {why:?}"));
            assert!(
                0.0 < wedge && (volume - wedge).abs() <= error,
                "{what}, deflection {size}: {volume} ± {error}, a wedge of {wedge}"
            );
        }
    }
}

/// The step-196 sliver of
/// `two_bends_overlapping_in_a_sliver_are_never_meshed_right_round_the_ring`
/// answers and validates both ways round, and `SolidView::measure` holds its
/// wedge of `5.9e-19` at deflections `0.5`, `0.25` and `0.125` too.
///
/// Until Q2316 the measure refused it [`Invalid::VolumeIsNotPositive`] there,
/// on sums of `−5.15e-16`, `−4.81e-16` and `−4.51e-16` against errors of
/// `5.10e-16`, `2.69e-16` and `3.05e-16`. The copy's cap is one circle with
/// one corner on it, and that corner lies `6.4e-16` off the cap's plane: the
/// slab between the cap's triangles and its plane is `−5.6e-16` at `0.5`, and
/// `−5.2e-16` of it is the triangles that corner tilts. The measure left a
/// flat face's own corners out of what it charged for points off its plane.
#[test]
fn the_sliver_at_step_196_is_measured_within_its_error_at_coarse_deflections() {
    let policy = Policy::certified();
    let start = -3.1 + 0.0155 * 196.0;
    for the_first_first in [true, false] {
        let what = format!("the first first {the_first_first}");
        let mut model = a_model();
        let first = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, start);
        let copy = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, start - FRAC_PI_2);
        let wedge = 4.0 * PI * the_angle_where_the_caps_overlap(&model, [first, copy], start);
        let (arguments, tools) = if the_first_first {
            ([first], [copy])
        } else {
            ([copy], [first])
        };
        let answer = model
            .fuse(
                &arguments,
                &tools,
                Rule::Intersection,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let [solid] = answer.solids()[..] else {
            panic!("{what}: one solid, not {:?}", answer.solids());
        };
        for size in [0.5, 0.25, 0.125] {
            let deflection = rcad_geom::Deflection::of(size).expect("a deflection");
            let (volume, error) = model
                .solid(solid)
                .and_then(|solid| solid.measure(deflection, &policy, &mut ProvenanceLog::new()))
                .map(|measured| (measured.volume(), measured.volume_error()))
                .unwrap_or_else(|why| {
                    panic!("{what}, deflection {size}: {why:?}, a wedge of {wedge}")
                });
            assert!(
                0.0 < wedge && (volume - wedge).abs() <= error,
                "{what}, deflection {size}: {volume} ± {error}, a wedge of {wedge}"
            );
        }
    }
}

/// That the fuse answered when `said` is `None`, and was refused `Undecidable`
/// with `said` the last record's predicate otherwise.
fn expect(fused: Result<usize, Error>, said: Option<&str>, what: &str) {
    match (&fused, said) {
        (Ok(_), None) => {}
        (Err(Error::Undecidable { records }), Some(said)) => assert_eq!(
            records.records().last().map(|record| record.predicate()),
            Some(said),
            "{what}"
        ),
        (_, _) => panic!("{what}: expected {said:?}, got {fused:?}"),
    }
}

/// The same two bends placed round by `−0.3565`, and by the grid value
/// `−3.1 + 0.0155 · 177`, which is `−0.35650000000000004`, and a quarter turn
/// back. Their touching caps lie apart: read exactly off the two stored
/// planes, both through the ring's axis, the copy's end cap is behind the
/// first bend's start cap, so the bends share no point. Every rule both ways
/// round was refused `Undecidable` at `side of torus` (Q2090, Q2149, Q2267).
///
/// The fuse places every piece. The copy's end cap is placed outside the first
/// bend by a point a rounding inside its rim, at the ring's inner equator,
/// whose first ray met the first bend's start cap on that cap's rim, where the
/// sign of the ring cannot be told. That ray was thrown away and the next
/// placed the point, but the thrown-away ray's two `side of torus` records
/// stayed in the fuse's log, and a certified policy refuses any record. The
/// next point the search offers places the cap the same way and asks nothing
/// undecided.
///
/// So the union answers both bends, the difference the first, and the
/// intersection nothing, each bend within the measure's own error of
/// `π² R r² / 2`.
#[test]
fn two_bends_whose_caps_lie_apart_answer_every_rule() {
    let one_bend = PI * PI * 4.0 / 2.0;
    for start in [-0.3565, -3.1 + 0.0155 * 177.0] {
        for rule in EVERY_RULE {
            for the_first_first in [true, false] {
                let what = format!("{start:?}, {rule:?}, the first first {the_first_first}");
                let mut model = a_model();
                let first = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, start);
                let copy = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, start - FRAC_PI_2);
                let bends = match rule {
                    Rule::Union => 2,
                    Rule::Difference => 1,
                    Rule::Intersection => 0,
                    other => panic!("not a rule this tries: {other:?}"),
                };
                let measured = fused_and_measured(&mut model, first, copy, rule, the_first_first)
                    .unwrap_or_else(|why| panic!("{what}: {}", the_refusal(&Err(why))));
                each_within_its_error(measured, &vec![one_bend; bends], &what);
            }
        }
    }
}
