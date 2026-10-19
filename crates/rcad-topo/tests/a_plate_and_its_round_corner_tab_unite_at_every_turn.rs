//! A plate and the round tab at its corner, united at every turn about `z`
//! (Q2374, from Q2370).
//!
//! At `Scale::millimetres(100.0)` the plate is the square `[0, 10]²` extruded
//! 2 high, and the tab is `add_round_bar(3, 2)` at `(10, 10, 0)`. Both are
//! placed with the same turn about `z` and then united. Three quarters of the
//! tab's disk lie outside the square, so the union holds `200 + 13.5π`.
//!
//! The tab's bottom seam corner `(3, 0, 0)`, moved to `(13, 10, 0)`, is on the
//! line `y = 10` the square's wall stands on. Turned, the tab's moved seam plane
//! and the wall have the same normal bit for bit, but at 40 of the 59 turns the
//! tab's offset is 1 to 3 ulps off the wall's, since the tab's shift composed
//! with the turn rounds. There the union was refused `Undecidable` at side of
//! plane, asking a box round the corner which side of the wall it is on.

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// The turns are `0.05 · step` for each step here, up to `2.95`.
const STEPS: core::ops::RangeInclusive<u8> = 1..=59;

#[test]
fn a_plate_and_its_round_corner_tab_unite_into_one_solid_at_every_turn_about_z() {
    let expected = 200.0 + 13.5 * PI;
    let mut failed = Vec::new();
    for step in STEPS {
        let angle = f64::from(step) * 0.05;
        if let Err(why) = one_solid_of_the_right_volume(angle, expected) {
            failed.push(format!("step {step}, turned {angle}: {why}"));
        }
    }
    assert!(
        failed.is_empty(),
        "{} of {} turns did not unite:\n{}",
        failed.len(),
        STEPS.count(),
        failed.join("\n")
    );
}

/// The union at `angle`, as one solid that validates and holds `expected`
/// within what the mesh can say; the reason it is not, otherwise.
fn one_solid_of_the_right_volume(angle: f64, expected: f64) -> Result<(), String> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let turn = Isometry::rotation(&Axis::z(), angle).expect("a finite angle");
    let square = a_block(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 2.0]);
    let tab = model
        .add_round_bar(
            3.0,
            2.0,
            &Isometry::translation(Vector::new(10.0, 10.0, 0.0)).expect("a finite shift"),
        )
        .expect("a bar");
    model
        .place_solid(square, &turn)
        .expect("this model made the key");
    model
        .place_solid(tab, &turn)
        .expect("this model made the key");
    let fused = model
        .fuse(
            &[square],
            &[tab],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map_err(|why| format!("refused: {why:?}"))?;
    let [plate] = fused.solids() else {
        return Err(format!("{} solids", fused.solids().len()));
    };
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .map_err(|why| format!("does not validate: {why}"))?;
    let measured = model
        .solid(*plate)
        .expect("the fuse made the key")
        .measure(
            Deflection::for_scale(model.scale()),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map_err(|why| format!("cannot be measured: {why:?}"))?;
    let (volume, error) = (measured.volume(), measured.volume_error());
    if (volume - expected).abs() <= error {
        Ok(())
    } else {
        Err(format!("measured {volume} ± {error}, by hand {expected}"))
    }
}

fn a_block(model: &mut Model, [x0, y0, z0]: [f64; 3], [x1, y1, z1]: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    model
        .add_extrusion(
            &rectangle,
            z1 - z0,
            &Isometry::translation(Vector::new(0.0, 0.0, z0)).expect("a finite shift"),
        )
        .expect("a rectangle extrudes")
}
