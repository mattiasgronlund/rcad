//! A round plate with a round hole through it, off its centre.
//!
//! The plate is a bar of radius 10 at `(−6, 0, 0)`, height 4; the hole is a bar
//! of radius 5 at `(−3, 0, −1)`, height 6, standing wholly inside it and out
//! through both caps. Each cap of the plate is cut into a disk inside the hole
//! and the ring round it, and the ring's outline and hole are each **a whole
//! circle with one corner**: `(4, 0, 0)` on the plate's rim and `(−3, −5, 0)` on
//! the hole's.
//!
//! Every fuse of the two refused, under every rule and both orders, with
//! `a point inside a piece of a face` naming those two corners
//! (`docs/decisions.md` §365.2, queue rows Q637 and Q714). Nothing the search
//! offered was inside the ring: a one-corner wire's corners blend to the corner
//! itself, the one blend across the two wires, `(0.5, −2.5, 0)`, is in the hole,
//! and so are the middles of both circles, `(−6, 0, 0)` and `(−3, 0, 0)`. The
//! ring holds plenty of doubles, `(−14, 0, 0)` among them, so this was a gap in
//! the search and not a piece without a point. The search now also offers the
//! points between the two circles along a line through each one's middle, and
//! `(−12, 0, 0)` is the first of them.
//!
//! The volumes, by hand. The plate is `π · 10² · 4 = 400π`, the hole's bar
//! `π · 5² · 6 = 150π`, and they share `π · 5² · 4 = 100π`.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

/// The plate less the hole's bar is the plate with its hole, `300π`, one solid.
#[test]
fn a_plate_less_an_off_centre_bar_is_the_plate_with_its_hole() {
    let (mut model, plate, hole) = a_plate_and_the_bar_through_it();
    let solids = fused(&mut model, plate, hole, Rule::Difference);
    assert_holds(&model, &solids, 1, 300.0 * PI, "the plate less the bar");
}

/// The bar less the plate is its two stubs, one below the plate and one above,
/// `2 · π · 5² · 1 = 50π`.
#[test]
fn the_bar_less_the_plate_is_its_two_stubs() {
    let (mut model, plate, hole) = a_plate_and_the_bar_through_it();
    let solids = fused(&mut model, hole, plate, Rule::Difference);
    assert_holds(&model, &solids, 2, 50.0 * PI, "the bar less the plate");
}

/// The union is `400π + 150π − 100π = 450π`, and the intersection is the plug
/// of bar inside the plate, `100π`, whichever solid goes first.
#[test]
fn the_union_and_the_intersection_answer_either_way_round() {
    for (rule, volume) in [(Rule::Union, 450.0 * PI), (Rule::Intersection, 100.0 * PI)] {
        for plate_first in [true, false] {
            let (mut model, plate, hole) = a_plate_and_the_bar_through_it();
            let [one, other] = if plate_first {
                [plate, hole]
            } else {
                [hole, plate]
            };
            let solids = fused(&mut model, one, other, rule);
            assert_holds(
                &model,
                &solids,
                1,
                volume,
                &format!("{rule:?} with the plate first: {plate_first}"),
            );
        }
    }
}

/// Q715's replay of Q602, which nothing pinned before (`docs/decisions.md`
/// §365.2): a bar of radius 39, height 4, and a bar of radius 25 standing
/// through it from `z = −1` to `5`, on the same axis. The 39 less the 25 is
/// `6084π − 2500π = 3584π`, one solid; the 25 less the 39 is
/// `3750π − 2500π = 1250π`, as its two stubs.
#[test]
fn a_bar_of_39_less_a_bar_of_25_through_it_is_3584pi_as_one_solid() {
    let (mut model, outer, through) = a_39_bar_and_a_25_bar_through_it();
    let solids = fused(&mut model, outer, through, Rule::Difference);
    assert_holds(
        &model,
        &solids,
        1,
        3584.0 * PI,
        "the 39 bar less the 25 bar",
    );
}

#[test]
fn a_bar_of_25_less_a_bar_of_39_through_it_is_1250pi_as_two_solids() {
    let (mut model, outer, through) = a_39_bar_and_a_25_bar_through_it();
    let solids = fused(&mut model, through, outer, Rule::Difference);
    assert_holds(
        &model,
        &solids,
        2,
        1250.0 * PI,
        "the 25 bar less the 39 bar",
    );
}

/// The union is `6084π + 3750π − 2500π = 7334π`, and the intersection is the
/// 25 bar's share of the 39 bar's height, `2500π`, whichever goes first.
#[test]
fn the_39_and_25_bars_union_and_intersect_either_way_round() {
    for (rule, volume) in [
        (Rule::Union, 7334.0 * PI),
        (Rule::Intersection, 2500.0 * PI),
    ] {
        for outer_first in [true, false] {
            let (mut model, outer, through) = a_39_bar_and_a_25_bar_through_it();
            let [one, other] = if outer_first {
                [outer, through]
            } else {
                [through, outer]
            };
            let solids = fused(&mut model, one, other, rule);
            assert_holds(
                &model,
                &solids,
                1,
                volume,
                &format!("{rule:?} with the 39 bar first: {outer_first}"),
            );
        }
    }
}

fn a_39_bar_and_a_25_bar_through_it() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let outer = model
        .add_round_bar(39.0, 4.0, &Isometry::identity())
        .expect("a bar of radius thirty-nine");
    let through = model
        .add_round_bar(25.0, 6.0, &at(0.0, 0.0, -1.0))
        .expect("a bar of radius twenty-five");
    (model, outer, through)
}

fn a_plate_and_the_bar_through_it() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let plate = model
        .add_round_bar(10.0, 4.0, &at(-6.0, 0.0, 0.0))
        .expect("a bar of radius ten");
    let hole = model
        .add_round_bar(5.0, 6.0, &at(-3.0, 0.0, -1.0))
        .expect("a bar of radius five");
    (model, plate, hole)
}

fn fused(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> Vec<SolidKey> {
    model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{rule:?} refused: {why:?}"))
        .solids()
        .to_vec()
}

fn assert_holds(model: &Model, solids: &[SolidKey], how_many: usize, volume: f64, what: &str) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} left a model that does not validate: {why}"));
    assert_eq!(solids.len(), how_many, "{what}: how many solids");
    let (mut measured, mut error) = (0.0, 0.0);
    for &solid in solids {
        let measure = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.001).expect("a fine mesh"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} does not measure: {why}"));
        measured += measure.volume();
        error += measure.volume_error();
    }
    assert!(
        (measured - volume).abs() <= error,
        "{what} holds {volume} and this one says {measured} give or take {error}"
    );
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift")
}
