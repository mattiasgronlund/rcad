//! A thin bar standing wholly inside the wall of a tube, touching nothing.
//!
//! The tube is a bar of radius 3 and height 4 less a bar of radius 1 and height
//! 6 standing at `z = −1`, so its caps are rings between two whole circles. The
//! thin bar has radius ½ and height 2 and stands at `(2, 0, 1)`: in the middle
//! of the wall, a unit clear of both caps and half a unit clear of both pipes.
//!
//! The thin bar stops short of the caps, but its pipe crosses a cap's plane in a
//! circle all the same — a fuse cuts along surfaces, not faces — so the cap
//! comes out as two pieces, the small disk inside that circle and the ring round
//! it, and is joined up again. What is left round the joined cap is an outline
//! and a hole that are each **a whole circle with one corner**. One corner
//! encloses no area, so the corners cannot say which circle is the outline.
//! Before the fix that was a panic in `fuse/assemble.rs` under the union both
//! ways round and under the tube less the bar: *"2 rings enclose no area to say
//! which it is"*. Q28 found it; §191.5 is the family.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

/// The union is the tube as it stood, `32π`, whichever solid goes first.
#[test]
fn a_bar_inside_a_tubes_wall_united_with_it_is_the_tube() {
    for thin_first in [false, true] {
        let (mut model, tube, thin) = a_tube_and_a_bar_inside_its_wall();
        let [one, other] = if thin_first {
            [thin, tube]
        } else {
            [tube, thin]
        };
        let answer = the_one_solid(&mut model, one, other, Rule::Union);
        assert_holds(&model, answer, 32.0 * PI, "the union");
    }
}

/// The tube less the bar is the tube with a bar-shaped hole sealed inside its
/// wall, `32π − π/2`.
#[test]
fn a_tube_less_a_bar_inside_its_wall_is_hollowed_by_the_bar() {
    let (mut model, tube, thin) = a_tube_and_a_bar_inside_its_wall();
    let answer = the_one_solid(&mut model, tube, thin, Rule::Difference);
    assert_holds(&model, answer, 32.0 * PI - PI / 2.0, "the tube less the bar");
}

fn a_tube_and_a_bar_inside_its_wall() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let outer = model
        .add_round_bar(3.0, 4.0, &Isometry::identity())
        .expect("a bar of radius three");
    let through = model
        .add_round_bar(1.0, 6.0, &at(0.0, 0.0, -1.0))
        .expect("a bar of radius one");
    let tube = the_one_solid(&mut model, outer, through, Rule::Difference);
    let thin = model
        .add_round_bar(0.5, 2.0, &at(2.0, 0.0, 1.0))
        .expect("a bar of radius a half");
    (model, tube, thin)
}

fn the_one_solid(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let answer = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{rule:?} refused: {why}"));
    let [solid] = answer.solids() else {
        panic!("{rule:?} is one solid, not {:?}", answer.solids());
    };
    *solid
}

fn assert_holds(model: &Model, solid: SolidKey, volume: f64, what: &str) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} left a model that does not validate: {why}"));
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} does not measure: {why}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{what} holds {volume} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift")
}
