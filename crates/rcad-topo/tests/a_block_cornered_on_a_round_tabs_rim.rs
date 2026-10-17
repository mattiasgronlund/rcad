//! A plate with a round tab, and a block whose corner `(12.4, 8.2)` sits on
//! the tab's rim (Q2232).
//!
//! The plate is the square `[0, 10]²` extruded 2 high, united with a round bar
//! of radius 3 and height 2 at `(10, 10, 0)`. `2.4² + 1.8² = 3²`, so the corner
//! is on the rim, and in `f64` it is a rounding outside it. The block's walls
//! `x = 12.4` and `y = 8.2` cross the rim's circle at two points a rounding
//! apart, and every fuse was refused on which of the two comes first round the
//! circle.
//!
//! Work in progress: this pins only that every fuse answers. The next lane adds
//! the volumes and the fix in `fuse/arrange.rs`.

use rcad_geom::{Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// Each block `[x0, y0]`–`[12.4, y1]`, set on the plate's top or under its
/// floor, fused with the plate by every rule, plate first and second.
#[test]
fn a_block_cornered_on_a_round_tabs_rim_fuses_by_every_rule_both_ways_round() {
    let mut refused = Vec::new();
    for [x0, y0, y1] in [
        [11.4, 8.2, 9.2],
        [7.6, 8.2, 11.8],
        [11.9, 8.2, 8.7],
        [11.2, 8.2, 9.4],
    ] {
        for [z0, z1] in [[2.0, 3.0], [-1.0, 0.0]] {
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                for plate_first in [true, false] {
                    let mut model =
                        Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
                    let plate = a_plate_with_a_round_tab(&mut model);
                    let block = a_block(&mut model, [x0, y0, z0], [12.4, y1, z1]);
                    let (arguments, tools) = if plate_first {
                        ([plate], [block])
                    } else {
                        ([block], [plate])
                    };
                    if let Err(why) = model.fuse(
                        &arguments,
                        &tools,
                        rule,
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    ) {
                        refused.push(format!(
                            "x {x0} to 12.4, y {y0} to {y1}, z {z0} to {z1}, {rule:?}, \
                             plate first {plate_first}: {why:?}"
                        ));
                    }
                }
            }
        }
    }
    assert!(refused.is_empty(), "refused:\n{}", refused.join("\n"));
}

fn a_plate_with_a_round_tab(model: &mut Model) -> SolidKey {
    let square = a_block(model, [0.0, 0.0, 0.0], [10.0, 10.0, 2.0]);
    let tab = model
        .add_round_bar(
            3.0,
            2.0,
            &Isometry::translation(Vector::new(10.0, 10.0, 0.0)).expect("a finite shift"),
        )
        .expect("a bar");
    let fused = model
        .fuse(
            &[square],
            &[tab],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the square and the tab unite");
    assert_eq!(fused.solids().len(), 1, "the plate is one solid");
    fused.solids()[0]
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
