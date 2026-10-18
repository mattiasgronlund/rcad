//! An L-shaped plate with a round hole, and a block resting on it, both turned
//! about `z`.
//!
//! At `Scale::millimetres(100.0)` the plate is the extrusion of `(0, 0)`
//! `(4, 0)` `(4, 2)` `(2, 2)` `(2, 4)` `(0, 4)` one high, less a round bar of
//! radius `0.5` standing at `(3, 1, −1)`; the block is `x` 0.5 to 1.5, `y` 1.5
//! to 3.5, `z` 1 to 2. The block's wall `y = 1.5`, drawn on across the plate's
//! top, touches the hole at `(3, 1.5)`, which is where the hole's one corner
//! is. Turned, that wall's line misses the hole's circle by a rounding, so the
//! hole is a lump of the drawing of its own, and its one corner is the only
//! point the arrangement places it by: a rounding from the line, so neither
//! the count of how often a walk winds round it nor its side of the line can
//! be read, and the top face was refused `CannotTellTheCellsOfAFaceApart`
//! under every rule both ways round (Q2229, Q2351).

use rcad_geom::{Axis, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

#[test]
fn a_holed_plate_and_a_block_on_it_turned_answer_every_rule_both_ways_round() {
    let mut refused = Vec::new();
    for step in 0..60 {
        let angle = f64::from(step) * 0.05;
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for plate_first in [true, false] {
                if let Err(why) = fused(angle, rule, plate_first) {
                    refused.push(format!(
                        "turned {angle}, {rule:?}, plate first {plate_first}: {why:?}"
                    ));
                }
            }
        }
    }
    assert!(
        refused.is_empty(),
        "{} of 360 fuses were refused:\n{}",
        refused.len(),
        refused.join("\n")
    );
}

/// The plate and the block, both turned by `angle` about `z`, fused under
/// `rule` and certified; the model validates once it answers.
fn fused(angle: f64, rule: Rule, plate_first: bool) -> rcad_topo::Result<()> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let plate = a_holed_plate(&mut model);
    let block = a_block_on_it(&mut model);
    let turn = Isometry::rotation(&Axis::z(), angle).expect("a finite angle");
    model
        .place_solid(plate, &turn)
        .expect("this model made the key");
    model
        .place_solid(block, &turn)
        .expect("this model made the key");
    let (first, second) = if plate_first {
        (plate, block)
    } else {
        (block, plate)
    };
    model.fuse(
        &[first],
        &[second],
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )?;
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the {rule:?} turned {angle} does not validate: {why}"));
    Ok(())
}

fn a_holed_plate(model: &mut Model) -> SolidKey {
    let outline = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 2.0],
        [2.0, 2.0],
        [2.0, 4.0],
        [0.0, 4.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("an L");
    let plate = model
        .add_extrusion(&outline, 1.0, &Isometry::identity())
        .expect("building a plate needs no decision");
    let bar = model
        .add_round_bar(
            0.5,
            3.0,
            &Isometry::translation(Vector::new(3.0, 1.0, -1.0)).expect("a finite placement"),
        )
        .expect("a bar");
    let holed = model
        .fuse(
            &[plate],
            &[bar],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar through a plate holes it");
    let [one] = holed.solids() else {
        panic!("a holed plate is one solid, got {}", holed.solids().len());
    };
    *one
}

fn a_block_on_it(model: &mut Model) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 2.0], [0.0, 2.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            1.0,
            &Isometry::translation(Vector::new(0.5, 1.5, 1.0)).expect("a finite placement"),
        )
        .expect("building a box needs no decision")
}
