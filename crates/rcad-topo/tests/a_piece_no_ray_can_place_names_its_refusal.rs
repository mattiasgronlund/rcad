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
//! face of its own kind. A quarter bend joined to a ball or a cone on its axis
//! is refused at `order along a crossing` while the two are cut, so that ring
//! solid is never built. A ball or a cone joined to a ring, built face by face
//! with no seam, is built; but against a piece of its own kind on it, it is
//! refused `ResultIsNotManifold`, or `Undecidable` at `side of sphere` or
//! `side of cone`, and moved off the axis, the piece is refused when the two
//! are crossed. So neither kind is reached.
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

/// A quarter bend of ring 4 and tube 1 placed round by `0.3`, and a copy
/// placed round by `0.3 + π/2`, by `place_solid`. The first bend's end cap and
/// the copy's start cap then lie on planes through the ring's axis whose
/// normals differ by an ulp, so the two bends touch across a gap no `f64` lies
/// in, and the first bend's end cap disk lies strictly in front of the copy's
/// start cap plane (Q1948, Q2056).
///
/// Every rule both ways round answers and validates: the union the two bends
/// apart, the intersection nothing and the difference the first bend, each
/// against `π² R r² / 2` a bend.
#[test]
fn a_bend_placed_round_by_0_3_and_a_copy_a_quarter_turn_on_answer_every_rule() {
    let one_bend = PI * PI * 4.0 / 2.0;
    let mut wrong = Vec::new();
    for rule in EVERY_RULE {
        for the_first_first in [true, false] {
            let what = format!("{rule:?}, the first first {the_first_first}");
            let mut model = a_model();
            let first = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, 0.3);
            let copy = a_quarter_bend_placed_round_by(&mut model, 4.0, 1.0, 0.3 + FRAC_PI_2);
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
                Ok(measured) => wrong.push(format!("{what}: {measured:?}, a bend {one_bend}")),
                Err(why) => wrong.push(format!("{what}: {}", the_refusal(&Err(why)))),
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
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
/// `5.6e-17` down, for `1e-12`, and for a quarter either way, which no rounding
/// is in and which is a refusal of its own.
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
        0.25,
        -0.25,
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

/// A quarter bend of ring 4 round a tube of 1, and a ball about the origin of
/// 3.5 or 4.5, or the frustum of the cone `ρ = (z + 8) / 2` from `z = −2` to
/// `2`: a ball or a cone on the ring's axis, crossing its tube. Joined, they
/// would be a ring solid with a ball or a cone face, for a piece of a ball or
/// a cone a hair off that face to be put to a ray against.
///
/// They are never joined. Every rule both ways round is refused `Undecidable`
/// at `order along a crossing` while the pieces are cut. For the union with
/// the ball of 3.5, the bend named first, it is at the two points where the
/// circle the ball crosses the ring in
/// meets the bend's two caps, `(3.406, 0, −0.805)` and `(0, 3.406, −0.805)`.
/// The ball and the frustum are turned `5π/4` round the axis, so the ball's
/// seam and the frustum's straight edge are clear of the bend.
#[test]
fn a_bend_and_a_ball_or_a_cone_on_its_axis_are_refused_while_they_are_cut() {
    let a_ball_of_3_5: fn(&mut Model) -> SolidKey = |model| {
        model
            .add_ball(3.5, &turned(5.0 * PI / 4.0))
            .expect("a ball")
    };
    let a_ball_of_4_5: fn(&mut Model) -> SolidKey = |model| {
        model
            .add_ball(4.5, &turned(5.0 * PI / 4.0))
            .expect("a ball")
    };
    let a_frustum: fn(&mut Model) -> SolidKey =
        |model| a_frustum_placed(model, 0.5, 6.0, 10.0, -8.0, 5.0 * PI / 4.0);
    let mut not_as_pinned = Vec::new();
    for (named, made) in [
        ("a ball of 3.5", a_ball_of_3_5),
        ("a ball of 4.5", a_ball_of_4_5),
        ("a frustum", a_frustum),
    ] {
        for rule in EVERY_RULE {
            for the_bend_first in [true, false] {
                let mut model = a_model();
                let bend = a_quarter_bend(&mut model, 4.0, 1.0);
                let other = made(&mut model);
                let fused = fused_or_refused(&mut model, bend, other, rule, the_bend_first);
                let said = the_refusal(&fused);
                if said != "Undecidable at order along a crossing" {
                    not_as_pinned.push(format!(
                        "{named}, {rule:?}, the bend first {the_bend_first}: {said}"
                    ));
                }
            }
        }
    }
    assert_eq!(not_as_pinned, Vec::<String>::new());
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
            [(7, 5), (4, 3), (5, 1)],
            "ACrossingBetweenATorusAndAnotherCurvedSurface",
        ),
        (
            "cone",
            a_cone,
            a_seamless_frustum,
            "Undecidable at side of cone",
            &["up 1e-13", "down 1e-13"][..],
            [(3, 3), (3, 3), (3, 1)],
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

/// PROBE (Q1951): a round bar of radius 4 standing a rounding tall — its one
/// curved wall face is a band between its two cap rims, a rounding apart,
/// exactly the shape `a_band_the_chart_reads_as_one_line` zips on a torus but
/// restricts to `Surface::Torus(_)`. Not committed.
#[test]
fn probe_q1951_thin_band_on_a_pipe() {
    let mut model = a_model();
    let bar = model
        .add_round_bar(4.0, 4.0 * f64::EPSILON, &Isometry::identity())
        .expect("a thin bar");
    let mesh = model.solid(bar).expect("this model made the key").mesh(
        rcad_geom::Deflection::of(1e-6).expect("a length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    println!("PROBE pipe: {mesh:?}");
}

/// PROBE (Q1951): a ball of radius 4, cut by a thin slab box a rounding tall
/// straddling its equator. Unlike `probe_q1951_thin_band_on_a_pipe` above, the
/// cut here comes from a real fuse (`Rule::Intersection`), so the two latitude
/// circles bounding the kept band are new rims the fuse's own plane-crosses-
/// sphere arithmetic placed a rounding apart, not two declared vertices on a
/// bare primitive. Not committed.
#[test]
fn probe_q1951_thin_band_on_a_ball_cut_by_a_fuse() {
    let mut model = a_model();
    let ball = model.add_ball(4.0, &Isometry::identity()).expect("a ball");
    let rectangle = ProfileBuilder::new(vec![[-5.0, -5.0], [5.0, -5.0], [5.0, 5.0], [-5.0, 5.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let slab = model
        .add_extrusion(
            &rectangle,
            4.0 * f64::EPSILON,
            &at_height(-2.0 * f64::EPSILON),
        )
        .expect("a thin slab");
    let fused = model.fuse(
        &[ball],
        &[slab],
        Rule::Intersection,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    match fused {
        Ok(answer) => {
            let solids: Vec<SolidKey> = answer.solids().to_vec();
            println!("PROBE ball x slab: Ok, {} solid(s)", solids.len());
            for solid in solids {
                let mesh = model.solid(solid).expect("this model made the key").mesh(
                    rcad_geom::Deflection::of(1e-6).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                println!("PROBE ball x slab mesh: {mesh:?}");
            }
        }
        Err(why) => println!("PROBE ball x slab: refused {:?}", the_refusal(&Err(why))),
    }
}

/// PROBE (Q1951): two coaxial bars, radius 4 and a rounding wider, the
/// narrower one a rounding taller — the step at their shared top leaves the
/// narrower bar's own wall between the step and its own cap rim, a rounding
/// apart, on a `Surface::Cylinder` rather than a `Surface::Torus`.
///
/// Printing `a_band_the_chart_reads_as_one_line`'s own locals for this face
/// (instrumented by hand, not committed) showed every condition but the
/// `Surface::Torus(_)` match already true: `goes_right_round` on both rims,
/// neither `goes_right_round_the_second_way`, `climbed` opposite in sign
/// (`-6.283185307179586` and `6.283185307179586`), and `highest - lowest` at
/// `9.9e-14`, inside `HOW_FAR_A_READ_LONGITUDE_CAN_BE_OFF`'s `8.9e-5`. So the
/// doc comment's "for want of a solid that reaches them" is wrong: this solid
/// reaches it. But widening the match to `Surface::Cylinder(_)` too, so the
/// face is handed to `zip_up_a_band` instead of cut and triangulated, changed
/// nothing: both ways round gave the identical
/// `CannotMeshAFace { why: BoundaryCrossesItself { at: 10261 }, .. }`, same
/// face, same index. So whatever `BoundaryCrossesItself` is tracking here
/// sits upstream of the `HowToCut` choice, not inside `cut_a_band_right_round_the_ring`
/// or `zip_up_a_band` themselves, and widening the predicate alone would not
/// have zipped this band up clean the way it does a torus's. Not committed,
/// and not run yet with an ordinary radius/height step rather than this
/// `1e-13` one, so neither a class nor a fix is claimed.
#[test]
fn probe_q1951_thin_band_on_two_coaxial_bars() {
    let mut model = a_model();
    let narrower = model
        .add_round_bar(4.0, 10.0, &Isometry::identity())
        .expect("a bar");
    let wider = model
        .add_round_bar(4.0 + 1e-13, 10.0 - 1e-13, &Isometry::identity())
        .expect("a bar");
    let fused = model.fuse(
        &[narrower],
        &[wider],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    match fused {
        Ok(answer) => {
            let solids: Vec<SolidKey> = answer.solids().to_vec();
            println!("PROBE two bars: Ok, {} solid(s)", solids.len());
            for solid in solids {
                let mesh = model.solid(solid).expect("this model made the key").mesh(
                    rcad_geom::Deflection::of(1e-6).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                println!(
                    "PROBE two bars mesh: {}",
                    match &mesh {
                        Ok(mesh) => format!("Ok({} points)", mesh.points().len()),
                        Err(why) => format!("{why:?}"),
                    }
                );
            }
        }
        Err(why) => println!("PROBE two bars: refused {:?}", the_refusal(&Err(why))),
    }
}
