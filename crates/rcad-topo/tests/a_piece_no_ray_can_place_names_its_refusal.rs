//! The shapes row Q1125 tried, looking for a fuse with a piece that no point
//! placed because no ray could be fired at a ring (`docs/decisions.md` §35.6).
//!
//! `select.rs`'s `settle_against_each` folds
//! `Containment::CannotFireARayAtAFacesSurface` into the undecided points, and
//! `Error::CannotFireARayAtAFacesSurface` exists to name it instead. Changing
//! that needs a fuse which reaches it, and **none of these does**. Each test
//! below is one family, with the step that stopped it pinned as it stands.
//!
//! What reaching it takes: a point proved inside a piece, put to a ray against
//! a solid with a ring face, whose own ray has no exact line, **and** whose box
//! grown to take in the exact point beside it meets a face of that solid, for
//! all three targets (`classify.rs`'s `what_a_point_beside_it_says`, §323) —
//! and that for every point offered inside the piece, with no side, corner or
//! plane placing it after. A piece of a different surface is cut along where
//! it meets the solid's skin, so its points sit clear of that skin, and a
//! piece on the same surface is caught as lying on the face, or shown off it.
//! What is left is a piece a rounding from a face of the solid, and every such
//! family here was refused while the pieces were cut, before a point was put
//! to a ray.
//!
//! Run with `settle_against_each`'s arm made to print, all 1663 tests of this
//! crate's suite and every fuse here: nothing printed.

use core::f64::consts::{FRAC_PI_2, PI};

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

mod rings;

use rings::{a_bend_turned_and_written_out, a_quarter_bend};

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
/// `Undecidable` at `side of torus`; at `1e-13` no point is found inside a
/// piece at all, `a point inside a piece of a face`; at `3e-14` and `1e-14`
/// the pieces are not cut, `side of plane`. A box round a point of a piece of
/// the wall is about `256 ε · 5`, `3e-13` across, so the hair would have to
/// be thinner than about `6e-14` of a turn.
#[test]
fn a_bend_placed_a_hair_round_from_another_answers_or_is_refused_before_a_ray() {
    for (turn, said) in [
        (1e-9, None),
        (1e-12, Some("side of torus")),
        (1e-13, Some("a point inside a piece of a face")),
        (3e-14, Some("side of plane")),
        (1e-14, Some("side of plane")),
    ] {
        for rule in EVERY_RULE {
            for the_first_first in [true, false] {
                let what =
                    format!("{turn:e} of a turn, {rule:?}, the first first {the_first_first}");
                let mut model = a_model();
                let first = a_quarter_bend(&mut model, 4.0, 1.0);
                let placed = a_quarter_bend_placed_round_by(&mut model, turn);
                let fused = fused_or_refused(&mut model, first, placed, rule, the_first_first);
                expect(fused, said, &what);
            }
        }
    }
}

/// A quarter bend and a copy placed half a turn round by `place_solid` rather
/// than written out: the copy's caps lie on planes through the ring's axis a
/// rounding off the first bend's, `sin(π)` not being nought. The two bends are
/// a quarter turn apart either way round, so every rule, both ways round,
/// answers as the same two bends written out do
/// (`two_bends_turned_by_whole_quarter_turns_answer_every_rule`), and each
/// solid it answers with holds a quarter of the ring, `2π²`, within what
/// `measure` says it can be out by.
#[test]
fn a_bend_placed_half_a_turn_round_from_another_answers_every_rule() {
    for rule in EVERY_RULE {
        for the_first_first in [true, false] {
            let what = format!("half a turn round, {rule:?}, the first first {the_first_first}");
            let mut model = a_model();
            let first = a_quarter_bend(&mut model, 4.0, 1.0);
            let placed = a_quarter_bend_placed_round_by(&mut model, PI);
            let (arguments, tools) = if the_first_first {
                ([first], [placed])
            } else {
                ([placed], [first])
            };
            let answer = model
                .fuse(
                    &arguments,
                    &tools,
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what}: {why:?}"));
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: {why:?}"));
            let expected = match rule {
                Rule::Union => 2,
                Rule::Difference => 1,
                Rule::Intersection => 0,
                other => panic!("not a rule this tries: {other:?}"),
            };
            assert_eq!(answer.solids().len(), expected, "{what}");
            for &solid in answer.solids() {
                holds_a_quarter_of_the_ring(&model, solid, &what);
            }
        }
    }
}

/// A quarter bend and a copy placed a quarter turn either way round by
/// `place_solid` rather than written out: `cos(π/2)` is not nought, so the
/// copy's cap is a rounding off the first bend's cap plane, the shape two
/// cubes a rounding apart are (`tests/two_turned_cubes_a_rounding_apart.rs`).
/// Every rule both ways round is refused while the pieces are cut,
/// `Undecidable` at `side of plane`, though the same two bends written out
/// answer (`two_bends_turned_by_whole_quarter_turns_answer_every_rule`).
#[test]
fn a_bend_placed_a_quarter_turn_round_from_another_is_refused_before_a_ray() {
    for turn in [FRAC_PI_2, -FRAC_PI_2] {
        for rule in EVERY_RULE {
            for the_first_first in [true, false] {
                let what = format!("{turn} round, {rule:?}, the first first {the_first_first}");
                let mut model = a_model();
                let first = a_quarter_bend(&mut model, 4.0, 1.0);
                let placed = a_quarter_bend_placed_round_by(&mut model, turn);
                let refused = fused_or_refused(&mut model, first, placed, rule, the_first_first);
                expect(refused, Some("side of plane"), &what);
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

fn a_quarter_bend_placed_round_by(model: &mut Model, turn: f64) -> SolidKey {
    let bend = a_quarter_bend(model, 4.0, 1.0);
    model
        .place_solid(
            bend,
            &Isometry::rotation(&Axis::z(), turn).expect("a real angle"),
        )
        .expect("this model made the key");
    bend
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

/// That `solid` measures a quarter of a ring of 4 and tube 1, `2π²` by Pappus,
/// within what `measure` says it can be out by.
fn holds_a_quarter_of_the_ring(model: &Model, solid: SolidKey, what: &str) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a hundredth of the tube"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the answer does not measure: {why:?}"));
    let pappus = 2.0 * PI * PI;
    assert!(
        (measured.volume() - pappus).abs() <= measured.volume_error(),
        "{what}: measures {} give or take {}, against {pappus}",
        measured.volume(),
        measured.volume_error()
    );
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
