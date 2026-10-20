//! A plate with a block united onto it, and a round bar standing through the
//! plate against the block's wall, all turned a quarter turn back about `z`.
//!
//! At `Scale::millimetres(100.0)` the plate is `x` and `y` 0 to 4, `z` 0 to 1;
//! the block on it is `x` 0.5 to 1.5, `y` 1.5 to 3.5, `z` 1 to 2, united with
//! it first, so the foot of the block's wall `x = 1.5` is a side of the
//! plate's top. The bar, of radius `0.5`, stands at `(2, 2.5)` from `z = −1`
//! to `2.5`, so its pipe touches that wall along the line `(1.5, 2.5)`.
//!
//! The bar's pipe cuts the plate's top in a circle that crosses nothing else
//! drawn there, a lump of the drawing of its own, and its one node is where
//! the circle starts: the circle's point along the first direction of the
//! top's plane, `+y`, wherever the turn puts the bar. Turned by `−π/2`, `+y`
//! is the way to the block's wall, so the node is the point where the pipe
//! touches the wall's foot, a rounding from that side of the top. The count
//! of how often the top's rim winds round the node could not be read there,
//! and the top was refused `CannotTellTheCellsOfAFaceApart` under every rule
//! both ways round (Q2353).

use core::f64::consts::{FRAC_PI_2, PI};

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

const PLATE_AND_BLOCK: f64 = 18.0;
const BAR: f64 = 0.875 * PI;
const BAR_IN_THE_PLATE: f64 = 0.25 * PI;
const BAR_BELOW_THE_PLATE: f64 = 0.25 * PI;
const BAR_ABOVE_THE_PLATE: f64 = 0.375 * PI;

#[test]
fn a_bar_against_a_blocks_wall_turned_a_quarter_turn_back_is_united_and_cut_both_ways_round() {
    for plate_first in [true, false] {
        holds(
            Rule::Union,
            plate_first,
            &[PLATE_AND_BLOCK + BAR - BAR_IN_THE_PLATE],
        );
    }
    holds(
        Rule::Difference,
        true,
        &[PLATE_AND_BLOCK - BAR_IN_THE_PLATE],
    );
    holds(
        Rule::Difference,
        false,
        &[BAR_BELOW_THE_PLATE, BAR_ABOVE_THE_PLATE],
    );
}

/// Fuses the plate and the bar turned by `−π/2` under `rule` and checks the
/// answer is one solid for each of `volumes`, measuring them in order of size.
fn holds(rule: Rule, plate_first: bool, volumes: &[f64]) {
    let asked = format!("{rule:?}, plate first {plate_first}");
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let plate = a_plate_with_a_block_on_it(&mut model);
    let bar = model
        .add_round_bar(
            0.5,
            3.5,
            &Isometry::translation(Vector::new(2.0, 2.5, -1.0)).expect("a finite placement"),
        )
        .expect("a bar");
    let turn = Isometry::rotation(&Axis::z(), -FRAC_PI_2).expect("a finite angle");
    model
        .place_solid(plate, &turn)
        .expect("this model made the key");
    model
        .place_solid(bar, &turn)
        .expect("this model made the key");
    let (first, second) = if plate_first {
        (plate, bar)
    } else {
        (bar, plate)
    };
    let fused = model
        .fuse(
            &[first],
            &[second],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{asked} does not validate: {why}"));
    let mut measured: Vec<(f64, f64)> = fused
        .solids()
        .iter()
        .map(|&solid| {
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.001).expect("a fine mesh"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
            (measured.volume(), measured.volume_error())
        })
        .collect();
    measured.sort_by(|one, other| one.0.total_cmp(&other.0));
    assert_eq!(
        measured.len(),
        volumes.len(),
        "{asked}: {measured:?} against {volumes:?}"
    );
    for (&(volume, error), &meant) in measured.iter().zip(volumes) {
        assert!(
            (volume - meant).abs() <= error,
            "{asked}: {volume} ± {error} against {meant}"
        );
    }
}

fn a_plate_with_a_block_on_it(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let plate = model
        .add_extrusion(&square, 1.0, &Isometry::identity())
        .expect("building a plate needs no decision");
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 2.0], [0.0, 2.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let block = model
        .add_extrusion(
            &rectangle,
            1.0,
            &Isometry::translation(Vector::new(0.5, 1.5, 1.0)).expect("a finite placement"),
        )
        .expect("building a box needs no decision");
    let united = model
        .fuse(
            &[plate],
            &[block],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a block standing on a plate unites with it");
    let [one] = united.solids() else {
        panic!(
            "a block on a plate unites into one solid, got {}",
            united.solids().len()
        );
    };
    *one
}
