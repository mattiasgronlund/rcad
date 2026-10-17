//! A plate with a round tab, and a block whose corner `(12.4, 8.2)` sits on
//! the tab's rim (Q2232).
//!
//! The plate is the square `[0, 10]²` extruded 2 high, united with a round bar
//! of radius 3 and height 2 at `(10, 10, 0)`. `2.4² + 1.8² = 3²`, so the corner
//! is on the rim, and in `f64` it is a rounding outside it. The block's walls
//! `x = 12.4` and `y = 8.2` cross the rim's circle at two points a rounding
//! apart, and every fuse was refused on which of the two comes first round the
//! circle. The walls and the plate's top or floor meet at the corner, an `f64`
//! triple, and `fuse/arrange.rs` now reads the order off them where the policy
//! would refuse it.
//!
//! Under a certified policy the fuses still refuse further on, at the sliver of
//! the block's face between the corner and the rim, which holds no `f64`
//! point; that refusal is in `fuse/select.rs`, and this file waits for its fix
//! to pin every certified fuse's answer and volume. A tolerant policy reads
//! the two crossings as one point, and there the fuses answer.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// Each block `[x0, y0]`–`[12.4, y1]`, set on the plate's top or under its
/// floor, fused with the plate by every rule, plate first and second.
#[test]
fn a_block_cornered_on_a_round_tabs_rim_is_never_refused_on_which_rim_crossing_comes_first() {
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
                    ) && format!("{why:?}").contains("order along a crossing")
                    {
                        refused.push(format!(
                            "x {x0} to 12.4, y {y0} to {y1}, z {z0} to {z1}, {rule:?}, \
                             plate first {plate_first}: {why:?}"
                        ));
                    }
                }
            }
        }
    }
    assert!(
        refused.is_empty(),
        "refused on which rim crossing comes first:\n{}",
        refused.join("\n")
    );
}

/// Under a tolerant policy the two rim crossings a rounding apart are one
/// point, the corner is on the rim, and each block that has no other corner
/// on it answers with its volume. The block touches the plate along a face,
/// so the union holds both, the difference the first, and the intersection
/// nothing.
///
/// The block `x 7.6 to 12.4` by `y 8.2 to 11.8` is left out: its corner
/// `(7.6, 11.8)` is on the rim too, and is refused at a point inside a piece of
/// a face under either policy.
#[test]
fn a_block_cornered_on_a_round_tabs_rim_answers_with_its_volume_when_the_corner_is_read_as_on_it() {
    let policy = Policy::tolerant(100.0);
    let mut wrong = Vec::new();
    for [x0, y0, y1] in [[11.4, 8.2, 9.2], [11.9, 8.2, 8.7], [11.2, 8.2, 9.4]] {
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
                    let plate_holds = 2.0 * (100.0 + 6.75 * core::f64::consts::PI);
                    let block_holds = (12.4 - x0) * (y1 - y0) * (z1 - z0);
                    let should_hold = match (rule, plate_first) {
                        (Rule::Union, _) => Some(plate_holds + block_holds),
                        (Rule::Intersection, _) => None,
                        (Rule::Difference, true) => Some(plate_holds),
                        (Rule::Difference, false) => Some(block_holds),
                        (other, _) => panic!("no volume for {other:?}"),
                    };
                    let what = model
                        .fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())
                        .map_err(|why| format!("refused {why:?}"))
                        .and_then(|fused| {
                            what_it_holds(&model, fused.solids(), should_hold, &policy)
                        });
                    if let Err(why) = what {
                        wrong.push(format!(
                            "x {x0} to 12.4, y {y0} to {y1}, z {z0} to {z1}, {rule:?}, \
                             plate first {plate_first}: {why}"
                        ));
                    }
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Whether `solids` is one solid that validates and measures `should_hold`
/// within its error, or none where `should_hold` is `None`.
fn what_it_holds(
    model: &Model,
    solids: &[SolidKey],
    should_hold: Option<f64>,
    policy: &Policy,
) -> Result<(), String> {
    model
        .validate(policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("does not validate: {why:?}"))?;
    let (&[solid], Some(should_hold)) = (solids, should_hold) else {
        return match (solids.len(), should_hold) {
            (0, None) => Ok(()),
            (count, _) => Err(format!("{count} solids")),
        };
    };
    let measured = model
        .solid(solid)
        .expect("the fuse made the key")
        .measure(
            Deflection::of(0.01).expect("a length"),
            policy,
            &mut ProvenanceLog::new(),
        )
        .map_err(|why| format!("does not measure: {why:?}"))?;
    if (measured.volume() - should_hold).abs() > measured.volume_error() {
        return Err(format!(
            "measured {} ± {}, and it should hold {should_hold}",
            measured.volume(),
            measured.volume_error()
        ));
    }
    Ok(())
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
