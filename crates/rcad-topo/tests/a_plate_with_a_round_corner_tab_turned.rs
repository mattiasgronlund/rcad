//! A plate with a round tab at its corner and a block on it, all turned about
//! `z` (Q2359, was Q2230).
//!
//! At `Scale::millimetres(100.0)` the plate is the square `[0, 10]²` extruded
//! 2 high, united with a round bar of radius 3 and height 2 at `(10, 10, 0)`,
//! the two placed with the same turn before they are united. The block is `x`
//! 6 to 12, `y` 6 to 12, `z` 2 to 3, placed with the same turn.
//!
//! The bar's seam corner `(13, 10, 0)` is on the line `y = 10` the square's
//! wall stands on, which crosses the tab's circle there. Unturned, the meet of
//! that wall, the plate's floor and the bar's wall comes out as the corner,
//! bit for bit. Turned, it is a box round the corner, and every fuse was
//! refused on which of the two comes first round the circle, with an
//! enclosure of `9.6e-15` at a turn of `0.05`.

use rcad_geom::{Axis, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// The turns about `z`, in steps of `0.05`, at which the plate itself unites on
/// the base `88298a67`. At the other steps up to `2.95` the union that builds
/// it is refused at side of plane by the seam corner, which this file does not
/// pin yet.
const STEPS_THE_PLATE_UNITES_AT: [u8; 19] = [
    1, 2, 3, 4, 7, 8, 9, 10, 15, 17, 18, 21, 22, 33, 38, 45, 47, 50, 54,
];

#[test]
fn a_turned_plate_with_a_round_corner_tab_and_a_block_on_it_answer_every_rule_both_ways_round() {
    let mut refused = Vec::new();
    let mut turns: Vec<(Axis, f64)> = STEPS_THE_PLATE_UNITES_AT
        .iter()
        .map(|&step| (Axis::z(), f64::from(step) * 0.05))
        .collect();
    turns.push((Axis::x(), 0.4));
    for (axis, angle) in &turns {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for plate_first in [true, false] {
                if let Err(why) = fused(axis, *angle, rule, plate_first) {
                    refused.push(format!(
                        "turned {angle} about {:?}, {rule:?}, plate first {plate_first}: {why:?}",
                        axis.direction()
                    ));
                }
            }
        }
    }
    assert!(
        refused.is_empty(),
        "{} of {} fuses were refused:\n{}",
        refused.len(),
        6 * turns.len(),
        refused.join("\n")
    );
}

/// The plate and the block, turned by `angle` about `axis`, fused under `rule`
/// and certified; the model validates once it answers.
fn fused(axis: &Axis, angle: f64, rule: Rule, plate_first: bool) -> rcad_topo::Result<()> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let turn = Isometry::rotation(axis, angle).expect("a finite angle");
    let plate = a_plate_with_a_round_corner_tab(&mut model, &turn);
    let block = a_block(&mut model, [6.0, 6.0, 2.0], [12.0, 12.0, 3.0]);
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

fn a_plate_with_a_round_corner_tab(model: &mut Model, turn: &Isometry) -> SolidKey {
    let square = a_block(model, [0.0, 0.0, 0.0], [10.0, 10.0, 2.0]);
    let tab = model
        .add_round_bar(
            3.0,
            2.0,
            &Isometry::translation(Vector::new(10.0, 10.0, 0.0)).expect("a finite shift"),
        )
        .expect("a bar");
    model
        .place_solid(square, turn)
        .expect("this model made the key");
    model
        .place_solid(tab, turn)
        .expect("this model made the key");
    let fused = model
        .fuse(
            &[square],
            &[tab],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the square and the tab unite at these turns");
    let [plate] = fused.solids() else {
        panic!("the plate is one solid, got {}", fused.solids().len());
    };
    *plate
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
