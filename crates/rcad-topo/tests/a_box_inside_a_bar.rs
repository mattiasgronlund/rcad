//! A box standing inside a bar, touching nothing.
//!
//! The bar has radius 5 and stands from `z = 0` to `z = 4` on the origin. The
//! box is 6 by 6 by 2, from `(−3, −3, 1)` to `(3, 3, 3)`: its upright edges are
//! `√18 ≈ 4.24` from the axis, inside the wall, and it is a unit clear of both
//! caps. That is Q28's placement.
//!
//! **A fuse cuts along surfaces, not faces**, so the box's four walls cross the
//! bar's wall in eight rulings, at `(±4, ±3)` and `(±3, ±4)`, and its floor and
//! roof cross it in the circles at `z = 1` and `z = 3`. The wall comes back as
//! 24 pieces, 8 round by 3 up, all of one pipe the same way round, and no edge
//! between them runs from the bottom rim to the top one: each line down the
//! wall is three rulings, broken at `z = 1` and `z = 3`. When a seam had to be
//! one such edge, the pieces stayed apart, and the union came back with 26
//! faces where the bar has 3. The bar less the box was refused `Undecidable`,
//! the record's predicate "one shell inside another". Q361 found both; a seam
//! that is a chain of rulings (`fuse/assemble.rs`, `a_seam_across_the_band`,
//! §224.2) answers both.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// United either way round, the box is swallowed and the answer is the bar: a
/// floor, a roof and **one** wall, holding `100π`.
#[test]
fn a_box_inside_a_bar_united_either_way_round_is_the_bar_with_one_wall() {
    for bar_first in [true, false] {
        let asked = format!("the union, the bar first {bar_first}");
        let mut model = a_model();
        let [bar, block] = the_bar_and_the_box(&mut model);
        let solid = the_one_solid(&mut model, bar, block, Rule::Union, bar_first, &asked);
        assert_holds(&model, solid, 100.0 * PI, &asked);
        assert_eq!(
            flat_and_curved_faces(&model, solid),
            [2, 1],
            "{asked}: how many flat faces and how many walls"
        );
    }
}

/// The bar less the box is the bar with a box-shaped hollow in it: one solid
/// of two shells, the bar's floor, roof and one wall round the outside and the
/// box's six faces round the hollow, holding `100π − 72`.
#[test]
fn the_bar_less_a_box_inside_it_is_the_bar_with_a_hollow() {
    let asked = "the bar less the box";
    let mut model = a_model();
    let [bar, block] = the_bar_and_the_box(&mut model);
    let solid = the_one_solid(&mut model, bar, block, Rule::Difference, true, asked);
    assert_holds(&model, solid, 100.0 * PI - 72.0, asked);
    assert_eq!(
        model
            .solid(solid)
            .expect("this model made the key")
            .shells()
            .count(),
        2,
        "{asked}: the outside and the hollow"
    );
    assert_eq!(
        flat_and_curved_faces(&model, solid),
        [8, 1],
        "{asked}: how many flat faces and how many walls"
    );
}

/// The bar with one wall can be fused again. The corners where the floor and
/// the roof crossed its seam are still corners of it, with the one wall on
/// both sides, and the box united with it again or taken out of it again
/// answers as it did the first time.
#[test]
fn the_bar_with_one_wall_takes_the_same_box_again() {
    for (rule, volume, faces) in [
        (Rule::Union, 100.0 * PI, [2, 1]),
        (Rule::Difference, 100.0 * PI - 72.0, [8, 1]),
    ] {
        let asked = format!("the union, then {rule:?} with the box again");
        let mut model = a_model();
        let [bar, block] = the_bar_and_the_box(&mut model);
        let united = the_one_solid(&mut model, bar, block, Rule::Union, true, &asked);
        let again = a_box(&mut model);
        let solid = the_one_solid(&mut model, united, again, rule, true, &asked);
        assert_holds(&model, solid, volume, &asked);
        assert_eq!(
            flat_and_curved_faces(&model, solid),
            faces,
            "{asked}: how many flat faces and how many walls"
        );
    }
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn the_bar_and_the_box(model: &mut Model) -> [SolidKey; 2] {
    let bar = model
        .add_round_bar(5.0, 4.0, &at(0.0, 0.0, 0.0))
        .expect("a bar of radius five");
    [bar, a_box(model)]
}

/// The box from `(−3, −3, 1)` to `(3, 3, 3)`.
fn a_box(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![[-3.0, -3.0], [3.0, -3.0], [3.0, 3.0], [-3.0, 3.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    model
        .add_extrusion(&square, 2.0, &at(0.0, 0.0, 1.0))
        .expect("a positive height")
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}

fn the_one_solid(
    model: &mut Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
    one_first: bool,
    asked: &str,
) -> SolidKey {
    let [first, second] = if one_first {
        [one, other]
    } else {
        [other, one]
    };
    let answer = model
        .fuse(
            &[first],
            &[second],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{asked} is refused: {why:?}"));
    let [solid] = answer.solids()[..] else {
        panic!("{asked} is one solid, not {:?}", answer.solids());
    };
    solid
}

fn assert_holds(model: &Model, solid: SolidKey, volume: f64, asked: &str) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{asked} left a model that does not validate: {why}"));
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{asked} does not measure: {why}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{asked} holds {volume} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// How many of the solid's faces are flat, and how many are on the bar's pipe.
fn flat_and_curved_faces(model: &Model, solid: SolidKey) -> [usize; 2] {
    let (flat, curved): (Vec<_>, Vec<_>) = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .partition(|face| face.surface().as_plane().is_some());
    [flat.len(), curved.len()]
}
