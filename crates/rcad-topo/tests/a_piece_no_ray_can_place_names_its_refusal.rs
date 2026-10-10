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
//! refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, or `Undecidable` at
//! `side of sphere` or `side of cone`, and moved off the axis, the piece is
//! refused when the two are crossed. So neither kind is reached.
//!
//! Run with `settle_against_each`'s arm made to print, every fuse here, only
//! the placed bends reached it: 3 of their 54 fuses, ring 4 and tube 1 turned
//! `−π/2`, the first bend named first, one fuse per rule. Each time the piece
//! was the placed bend's cap on the plane whose normal is `(−2.2e-16, 1, 0)`,
//! and its three points, at `(4, 8.9e-16, 0)`, `(4, 6.3e-16, 0)` and
//! `(4, 8.9e-16, 0)`, could not be fired at the first bend's ring. Each of
//! those pieces was then placed inside by a side, a corner, a plane or a piece
//! across, and each fuse was refused at a later piece no point was found
//! inside. Q1581's fuses, run with the arm made to print, printed nothing.
//!
//! Q1125's run, of all 1755 tests of this crate's suite with the arm made to
//! print, printed nine times, all in the placed bends' test, and each piece it
//! printed for was placed by some other point or rule too. One test of that
//! run, `a_half_bend_and_two_quarter_bars_spelling_a_plane_through_its_axis_an_ulp_apart_are_refused`,
//! stopped at its first fuse; run again with all 24 of its fuses let through,
//! it printed none.

use core::f64::consts::{FRAC_PI_2, PI};
use std::collections::BTreeMap;

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale,
    Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};
use smallvec::smallvec;

mod rings;

use rings::{a_bend_turned_and_written_out, a_quarter_bend, the_top_half_of_a_ring};

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

/// A quarter bend and a copy placed a quarter turn either way round, or half a
/// turn, by `place_solid` rather than written out, for three rings:
/// `cos(π/2)` and `sin(π)` are not nought, so the copy's caps lie on planes
/// through the ring's axis a rounding off the first bend's, the shape two
/// cubes a rounding apart are (`tests/two_turned_cubes_a_rounding_apart.rs`).
///
/// The pieces are cut: a point of one bend's cap rim, a rounding off the
/// other's cap plane, is put on its side of that plane by the two planes' own
/// numbers (Q1392). But each such plane cuts the ring in a second circle too,
/// a rounding round from the other bend's own cap rim, and the band of that
/// bend's wall between the two holds no point the search offers. Every rule
/// both ways round is refused `Undecidable` at `a point inside a piece of a
/// face`, though the same two bends written out answer
/// (`two_bends_turned_by_whole_quarter_turns_answer_every_rule`). These are
/// the only fuses here whose points reach the arm that folds the refusal; the
/// header says which.
#[test]
fn a_bend_placed_a_quarter_turn_round_from_another_is_refused_before_a_ray() {
    for (ring, tube) in [(4.0, 1.0), (1.0, 0.5), (2.0, 1.0)] {
        for turn in [FRAC_PI_2, -FRAC_PI_2, PI] {
            for rule in EVERY_RULE {
                for the_first_first in [true, false] {
                    let what = format!(
                        "ring {ring}, tube {tube}, {turn} round, {rule:?}, \
                         the first first {the_first_first}"
                    );
                    let mut model = a_model();
                    let first = a_quarter_bend(&mut model, ring, tube);
                    let placed = a_quarter_bend_placed_round_by(&mut model, ring, tube, turn);
                    let refused =
                        fused_or_refused(&mut model, first, placed, rule, the_first_first);
                    expect(refused, Some("a point inside a piece of a face"), &what);
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
/// `a_bend_placed_a_quarter_turn_round_from_another_is_refused_before_a_ray`.
/// Every rule both ways round is then refused `Undecidable` at `order along a
/// crossing` while the pieces are cut, before any point is put to a ray.
#[test]
fn a_quarter_of_a_bar_placed_round_from_a_bend_in_it_is_refused_before_a_ray() {
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
    for turn in [FRAC_PI_2, -FRAC_PI_2, PI] {
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
                let refused = fused_or_refused(&mut model, bend, quarter, rule, the_bend_first);
                expect(refused, Some("order along a crossing"), &what);
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
/// The wedge's face is not stored on a plane through the axis, so the fuse is
/// refused `ACrossingBetweenAPlaneAndATorusThatIsAQuartic` from a hair of
/// `1e-3` down to `1e-14` at a reach of ten; at `1e-15` and below the wedge
/// itself, the prism less what is outside the bar, is refused `Undecidable`
/// at `order along a crossing`.
#[test]
fn a_wedge_a_hair_below_a_bends_cap_is_refused_before_a_ray() {
    for hair in [1e-3, 1e-9, 1e-12, 1e-14] {
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
    for hair in [1e-15, 1e-16, 1e-20] {
        let mut model = a_model();
        let refused = a_wedge(&mut model, hair).map(|_| ());
        let said = match &refused {
            Err(Error::Undecidable { records }) => {
                records.records().last().map(|record| record.predicate())
            }
            _ => None,
        };
        assert_eq!(
            said,
            Some("order along a crossing"),
            "a hair of {hair:e}: {refused:?}"
        );
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
/// None reaches a ray. Where the piece sits, and moved up or down `z` by a
/// rounding, every rule both ways round is refused
/// `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`. Moved by `1e-13`, it is refused
/// so with the ring solid named first, and `Undecidable` at `side of sphere`
/// or `side of cone` with it named second, at the piece's bottom rim. Moved
/// `1e-13` off the axis along `x`, the piece's ball is one the ring is not
/// crossed with, and its cone one the ring solid's cone is not crossed with.
///
/// A whole ball about the origin, of 5 and a rounding or `5 ± 1e-13`, against
/// the ball joined to a ring is refused `Undecidable` at `side of sphere` by
/// every rule both ways round, at `(0, 3, −4)` on the lower circle the ball
/// and the ring cross in.
#[test]
fn a_ball_or_a_cone_joined_to_a_ring_with_a_piece_of_its_own_kind_on_it_is_refused_before_a_ray() {
    let a_rounding = 8.881_784_197_001_252e-16;
    let three_surfaces = "WhereThreeSurfacesMeetNeedsTwoOfThemFlat";
    let a_ball: fn(&mut Model) -> Result<SolidKey, Error> = a_ball_joined_to_a_ring;
    let a_cone: fn(&mut Model) -> Result<SolidKey, Error> = a_frustum_joined_to_a_ring;
    let mut not_as_pinned = Vec::new();
    for (kind, ring_solid, piece, its_side, off_the_axis) in [
        (
            "ball",
            a_ball,
            a_ball_segment as fn(&mut Model) -> Result<SolidKey, Error>,
            "Undecidable at side of sphere",
            "ACrossingBetweenATorusAndAnotherCurvedSurface",
        ),
        (
            "cone",
            a_cone,
            a_seamless_frustum,
            "Undecidable at side of cone",
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
            for rule in EVERY_RULE {
                for the_ring_first in [true, false] {
                    let mut model = a_model();
                    let ring = ring_solid(&mut model).expect("a ring solid");
                    let on_it = piece(&mut model).expect("a piece of its own kind");
                    model
                        .place_solid(on_it, &by)
                        .expect("this model made the key");
                    let fused = fused_or_refused(&mut model, ring, on_it, rule, the_ring_first);
                    let said = the_refusal(&fused);
                    let pinned = match moved {
                        "off the axis" => off_the_axis,
                        "up 1e-13" | "down 1e-13" if !the_ring_first => its_side,
                        _ => three_surfaces,
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
        &[ball.clone(), ring, ball],
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
        &[
            flat_at(-6.0, -1.0),
            cone.clone(),
            ring,
            cone,
            flat_at(7.0, 1.0),
        ],
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
    for (index, surface) in surfaces.iter().enumerate() {
        let mut wires = Vec::with_capacity(2);
        if index > 0 {
            let coedge = builder.add_coedge(circles[index - 1], turns[index - 1])?;
            wires.push(builder.add_wire(&[DraftStep::Along(coedge)])?);
        }
        if index < circles.len() {
            let coedge = builder.add_coedge(circles[index], turns[index].reversed())?;
            wires.push(builder.add_wire(&[DraftStep::Along(coedge)])?);
        }
        shell.push((
            builder.add_face(surface.clone(), &wires)?,
            Orientation::Forward,
        ));
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
