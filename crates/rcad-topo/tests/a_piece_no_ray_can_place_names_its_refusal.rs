//! The shapes row Q1125 tried, looking for a fuse with a piece that no point
//! placed because no ray could be fired at a ring (`docs/decisions.md` §35.6).
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
//! and that for every point offered inside the piece, with no side, corner or
//! plane placing it after. A piece of a different surface is cut along where
//! it meets the solid's skin, so its points sit clear of that skin, and a
//! piece on the same surface is caught as lying on the face, or shown off it.
//! What is left is a piece a rounding from a face of the solid, and every such
//! family here was refused before a point was put to a ray: while the pieces
//! were cut, or when no point was found inside a piece.
//!
//! Run with `settle_against_each`'s arm made to print, all 1755 tests of this
//! crate's suite, every fuse here among them: it printed nine times, all in
//! `a_bend_placed_a_quarter_turn_round_from_another_is_refused_before_a_ray`,
//! whose pieces are cut since Q1392. Each piece whose point it printed for was
//! placed by some other point or rule, and every one of those fuses is refused
//! at a later piece no point was found inside. One test of that run,
//! `a_half_bend_and_two_quarter_bars_spelling_a_plane_through_its_axis_an_ulp_apart_are_refused`,
//! stopped at its first fuse; run again with all 24 of its fuses let through,
//! it printed none.

use core::f64::consts::{FRAC_PI_2, PI};

use rcad_geom::{Axis, Isometry, Scale, Vector};
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
/// `Undecidable` at `side of torus`; at `1e-13`, `3e-14` and `1e-14` no
/// point is found inside a piece at all, `a point inside a piece of a face`.
/// At `3e-14` and `1e-14` the pieces were not cut, `side of plane`, until a
/// point on the ring and on a plane through its axis was put on a side of
/// another such plane by the planes' own numbers (Q1392). A box round a
/// point of a piece of the wall is about `256 ε · 5`, `3e-13` across, so the
/// hair would have to be thinner than about `6e-14` of a turn.
#[test]
fn a_bend_placed_a_hair_round_from_another_answers_or_is_refused_before_a_ray() {
    for (turn, said) in [
        (1e-9, None),
        (1e-12, Some("side of torus")),
        (1e-13, Some("a point inside a piece of a face")),
        (3e-14, Some("a point inside a piece of a face")),
        (1e-14, Some("a point inside a piece of a face")),
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

/// A quarter bend and a copy placed a quarter turn either way round, or half a
/// turn, by `place_solid` rather than written out: `cos(π/2)` and `sin(π)` are
/// not nought, so the copy's caps lie on planes through the ring's axis a
/// rounding off the first bend's, the shape two cubes a rounding apart are
/// (`tests/two_turned_cubes_a_rounding_apart.rs`).
///
/// The pieces are cut: a point of one bend's cap rim, a rounding off the
/// other's cap plane, is put on its side of that plane by the two planes' own
/// numbers (Q1392). But each such plane cuts the ring in a second circle too,
/// a rounding round from the other bend's own cap rim, and the band of that
/// bend's wall between the two holds no point the search offers. Every rule
/// both ways round is refused `Undecidable` at `a point inside a piece of a
/// face`, though the same two bends written out answer
/// (`two_bends_turned_by_whole_quarter_turns_answer_every_rule`).
#[test]
fn a_bend_placed_a_quarter_turn_round_from_another_is_refused_before_a_ray() {
    for turn in [FRAC_PI_2, -FRAC_PI_2, PI] {
        for rule in EVERY_RULE {
            for the_first_first in [true, false] {
                let what = format!("{turn} round, {rule:?}, the first first {the_first_first}");
                let mut model = a_model();
                let first = a_quarter_bend(&mut model, 4.0, 1.0);
                let placed = a_quarter_bend_placed_round_by(&mut model, turn);
                let refused = fused_or_refused(&mut model, first, placed, rule, the_first_first);
                expect(refused, Some("a point inside a piece of a face"), &what);
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

#[test]
fn q1393_probe() {
    let turns = [FRAC_PI_2, -FRAC_PI_2, PI];
    for (ring, tube) in [(4.0, 1.0), (1.0, 0.5), (2.0, 1.0)] {
        for turn in turns {
            for rule in EVERY_RULE {
                for first_first in [true, false] {
                    let what = format!(
                        "F1 bends ring {ring} tube {tube} turn {turn} {rule:?} first {first_first}"
                    );
                    eprintln!("PROBE {what}");
                    let mut model = a_model();
                    let first = a_quarter_bend(&mut model, ring, tube);
                    let placed = a_quarter_bend(&mut model, ring, tube);
                    model
                        .place_solid(placed, &Isometry::rotation(&Axis::z(), turn).unwrap())
                        .unwrap();
                    let got = fused_or_refused(&mut model, first, placed, rule, first_first);
                    eprintln!("PROBE-RESULT {what}: {}", short(&got));
                }
            }
        }
    }
    for turn in [0.0, FRAC_PI_2, -FRAC_PI_2, PI] {
        for rule in EVERY_RULE {
            for first_first in [true, false] {
                let what = format!("F2 sector turn {turn} {rule:?} bend first {first_first}");
                eprintln!("PROBE {what}");
                let mut model = a_model();
                let bend = a_quarter_bend(&mut model, 4.0, 1.0);
                let sector = match a_sector(&mut model, 6.0, turn) {
                    Ok(sector) => sector,
                    Err(why) => {
                        eprintln!("PROBE-RESULT {what}: sector {why:?}");
                        continue;
                    }
                };
                let got = fused_or_refused(&mut model, bend, sector, rule, first_first);
                eprintln!("PROBE-RESULT {what}: {}", short(&got));
            }
        }
    }
}

#[test]
fn q1393_probe_f3() {
    for (label, bottom, height) in [
        ("exact", -1.5, 0.0),
        ("overlap", -1.5, 1.110_223_024_625_156_5e-16),
        ("gap", -1.5, -5.551_115_123_125_783e-17),
        ("overlap by 1e-12", -1.5, 1e-12),
        ("overlap by a quarter", -1.5, 0.25),
        ("gap of a quarter", -1.5, -0.25),
        ("exact moved down and up", -1.5, 1e300),
    ] {
        for rule in EVERY_RULE {
            for ring_first in [true, false] {
                let what = format!("F3 {label} {rule:?} ring first {ring_first}");
                eprintln!("PROBE {what}");
                let mut model = a_model();
                let ring = rings::the_top_half_of_a_ring(&mut model, 4.0, 1.0, false)
                    .expect("a half ring");
                let sector = match an_annulus_sector(&mut model, bottom, height) {
                    Ok(sector) => sector,
                    Err(why) => {
                        eprintln!("PROBE-RESULT {what}: sector {why:?}");
                        continue;
                    }
                };
                let got = fused_or_refused(&mut model, ring, sector, rule, ring_first);
                eprintln!("PROBE-RESULT {what}: {}", short(&got));
            }
        }
    }
}

#[test]
fn q1393_probe_f4() {
    let moves = [
        ("up 1e-15", at_height(1e-15)),
        ("up 1e-12", at_height(1e-12)),
        (
            "round x by 2pi",
            Isometry::rotation(&Axis::x(), 2.0 * PI).unwrap(),
        ),
        (
            "round z by 2pi",
            Isometry::rotation(&Axis::z(), 2.0 * PI).unwrap(),
        ),
    ];
    for (label, movement) in &moves {
        for rule in EVERY_RULE {
            for first_first in [true, false] {
                let what = format!("F4 bend {label} {rule:?} first {first_first}");
                let mut model = a_model();
                let first = a_quarter_bend(&mut model, 4.0, 1.0);
                let moved = a_quarter_bend(&mut model, 4.0, 1.0);
                model.place_solid(moved, movement).unwrap();
                let got = fused_or_refused(&mut model, first, moved, rule, first_first);
                eprintln!("PROBE-RESULT {what}: {}", short(&got));
            }
        }
    }
}

/// The quarter of a thick tube about `z`, radii 3.5 to 4.5, in `x, y >= 0`,
/// from `bottom` up by `height`.
fn an_annulus_sector(model: &mut Model, bottom: f64, hair: f64) -> Result<SolidKey, Error> {
    let policy = Policy::certified();
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [0.0, 10.0]])
        .build(&policy, &mut ProvenanceLog::new())
        .expect("a triangle");
    let prism = model
        .add_extrusion(&triangle, -bottom, &at_height(bottom))
        .expect("a prism");
    let outer = model
        .add_round_bar(6.0, 6.0, &at_height(-3.0))
        .expect("a round bar");
    let inner = model
        .add_round_bar(2.0, 6.0, &at_height(-3.0))
        .expect("a round bar");
    let answer = model.fuse(
        &[prism],
        &[outer],
        Rule::Intersection,
        &policy,
        &mut ProvenanceLog::new(),
    )?;
    let &[sector] = answer.solids() else {
        panic!("one sector, got {:?}", answer.solids());
    };
    let answer = model.fuse(
        &[sector],
        &[inner],
        Rule::Difference,
        &policy,
        &mut ProvenanceLog::new(),
    )?;
    let &[sector] = answer.solids() else {
        panic!("one sector, got {:?}", answer.solids());
    };
    if hair == 1e300 {
        model.place_solid(sector, &at_height(-1.0)).unwrap();
        model.place_solid(sector, &at_height(1.0)).unwrap();
    } else if hair != 0.0 {
        model.place_solid(sector, &at_height(hair)).unwrap();
    }
    let top = model
        .solid(sector)
        .unwrap()
        .faces()
        .filter_map(|face| face.surface().as_plane().map(|plane| format!("{plane:?}")))
        .collect::<Vec<_>>();
    eprintln!("PROBE sector planes {top:?}");
    Ok(sector)
}

fn short(got: &Result<usize, Error>) -> String {
    match got {
        Ok(n) => format!("Ok({n})"),
        Err(Error::Undecidable { records }) => format!(
            "Undecidable at {:?} {:?}",
            records.records().last().map(|record| record.predicate()),
            records.records().last().map(|record| record.inputs())
        ),
        Err(other) => format!("{other:?}"),
    }
}

/// The quarter of a round bar of radius `radius` about `z` in `x, y >= 0`,
/// `z` in `[-2, 2]`, placed turned by `turn` about `z` when `turn` is not 0.
fn a_sector(model: &mut Model, radius: f64, turn: f64) -> Result<SolidKey, Error> {
    let policy = Policy::certified();
    let far = 2.0 * radius;
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [far, 0.0], [0.0, far]])
        .build(&policy, &mut ProvenanceLog::new())
        .expect("a triangle");
    let prism = model
        .add_extrusion(&triangle, 4.0, &at_height(-2.0))
        .expect("a prism");
    let bar = model
        .add_round_bar(radius, 6.0, &at_height(-3.0))
        .expect("a round bar");
    let answer = model.fuse(
        &[prism],
        &[bar],
        Rule::Intersection,
        &policy,
        &mut ProvenanceLog::new(),
    )?;
    let &[sector] = answer.solids() else {
        panic!("one sector, got {:?}", answer.solids());
    };
    if turn != 0.0 {
        model
            .place_solid(sector, &Isometry::rotation(&Axis::z(), turn).unwrap())
            .unwrap();
    }
    Ok(sector)
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
