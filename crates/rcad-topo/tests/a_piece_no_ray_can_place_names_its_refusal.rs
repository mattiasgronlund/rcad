//! The shapes rows Q1125 and Q1393 tried, looking for a fuse with a piece that
//! no point placed because no ray could be fired at a ring
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
//! left is a piece a rounding off a face of the ring solid, on a surface the
//! ring is crossed with, since any other is refused when the two are crossed
//! (`rcad_geom::Error`'s two torus refusals). Q1393 tried the three ways a
//! face can be that, and pins each below:
//!
//! - a plane through the ring's axis a rounding off a cap, placed there by a
//!   turn whose cosine or sine rounds: two bends, and a quarter of a bar over
//!   a bend;
//! - a plane square to the axis a hair off a flat face: a quarter of a tube
//!   under half a ring;
//! - a torus a rounding off a torus: a bend and a copy of it moved a hair.
//!
//! Run with `settle_against_each`'s arm made to print, only the placed bends
//! reached it: 3 of their 54 fuses, ring 4 and tube 1 turned `−π/2`, the first
//! bend named first, one fuse per rule. Each time the piece was the placed
//! bend's cap on the plane whose normal is `(−2.2e-16, 1, 0)`, and its three
//! points, at `(4, 8.9e-16, 0)`, `(4, 6.3e-16, 0)` and `(4, 8.9e-16, 0)`, could
//! not be fired at the first bend's ring. Each of those pieces was then placed
//! inside by a side, a corner, a plane or a piece across, and each fuse was
//! refused at a later piece no point was found inside.
//!
//! Q1125's run, of all 1755 tests of this crate's suite with the arm made to
//! print, printed nine times, all in the placed bends' test, and each piece it
//! printed for was placed by some other point or rule too. One test of that
//! run, `a_half_bend_and_two_quarter_bars_spelling_a_plane_through_its_axis_an_ulp_apart_are_refused`,
//! stopped at its first fuse; run again with all 24 of its fuses let through,
//! it printed none.

use core::f64::consts::{FRAC_PI_2, PI};

use rcad_geom::{Axis, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

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

/// The quarter of a thick tube about `z`, radii 2 to 6, in `x, y >= 0`, `z`
/// from −1.5 to 0, then placed up `z` by each of `moves` in turn.
///
/// The quarter is built where it sits and moved after: a prism built a hair
/// off `z = 0`, with its bottom at −0.3 or −0.5, was refused `order along a
/// crossing` while the quarter itself was cut.
fn a_quarter_of_a_tube_moved_by(model: &mut Model, moves: &[f64]) -> SolidKey {
    let policy = Policy::certified();
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [0.0, 10.0]])
        .build(&policy, &mut ProvenanceLog::new())
        .expect("a triangle");
    let prism = model
        .add_extrusion(&triangle, 1.5, &at_height(-1.5))
        .expect("a prism");
    let outer = model
        .add_round_bar(6.0, 6.0, &at_height(-3.0))
        .expect("a round bar");
    let inner = model
        .add_round_bar(2.0, 6.0, &at_height(-3.0))
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
