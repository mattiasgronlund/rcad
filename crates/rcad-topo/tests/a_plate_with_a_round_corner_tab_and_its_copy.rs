//! A plate with a round tab at its corner, fused with a copy of itself, and
//! with a small block inside the tab (Q2405, from Q2231 and Q2036).
//!
//! At `Scale::millimetres(100.0)` the plate is the square `[0, 10]²` extruded
//! 2 high and united with `add_round_bar(r, 2)` at `(x, y, 0)`. With the tab's
//! centre at the square's corner, or near it, the union's wall on the pipe is
//! more than half of it: the band between the two caps, less the part inside
//! the square, which is the part in front of **either** of the square's two
//! walls the tab crosses and not the part in front of both. No step that reads
//! a face as the part in front of its cuts takes such a band, so every point
//! asked of that wall was refused `AFaceTrimmedByACurveThisCannotFollow`, and
//! so was every rule of the plate and its copy, both ways round. A tab at the
//! middle of a side leaves half a pipe, in front of every cut, and answered.
//!
//! The wall is now counted along the ruling through the point, as a pipe face
//! crossed by a pipe of its width already was: a ruling meets the arc of each
//! cap once or not at all, so how many it meets above the point says whether
//! the point is on the face.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// Each tab as `(radius, centre)`: at the corner, smaller at the corner, and
/// off the corner either way, all of which were refused; then at the middle
/// of a side, which answered.
const TABS: [(f64, [f64; 2]); 5] = [
    (3.0, [10.0, 10.0]),
    (2.7, [10.0, 10.0]),
    (3.0, [9.0, 10.5]),
    (3.0, [11.0, 9.0]),
    (2.5, [10.0, 5.0]),
];

/// `A ∪ A` and `A ∩ A` are `A`, and `A \ A` is nothing, whichever copy comes
/// first. Each answer is held to the volume the plate itself measures, within
/// what the two meshes may miss by.
#[test]
fn a_plate_with_a_round_tab_and_its_copy_answer_every_rule_both_ways_round() {
    let policy = Policy::certified();
    let mut failed = Vec::new();
    for (radius, centre) in TABS {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let one = a_plate_with_a_tab(&mut model, radius, centre);
        let two = a_plate_with_a_tab(&mut model, radius, centre);
        let (volume, error) = measured(&model, &[one], &policy);
        for (rule, first, second) in [
            (Rule::Union, one, two),
            (Rule::Union, two, one),
            (Rule::Intersection, one, two),
            (Rule::Intersection, two, one),
            (Rule::Difference, one, two),
            (Rule::Difference, two, one),
        ] {
            let mut model = model.clone();
            let fused = match model.fuse(
                &[first],
                &[second],
                rule,
                &policy,
                &mut ProvenanceLog::new(),
            ) {
                Ok(fused) => fused,
                Err(why) => {
                    failed.push(format!(
                        "tab {radius} at {centre:?}, {rule:?}: refused {why:?}"
                    ));
                    continue;
                }
            };
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|broken| {
                    panic!("tab {radius} at {centre:?}, {rule:?}: {broken:?}")
                });
            let solids = fused.solids();
            if rule == Rule::Difference {
                if !solids.is_empty() {
                    failed.push(format!(
                        "tab {radius} at {centre:?}, {rule:?}: {} solids",
                        solids.len()
                    ));
                }
                continue;
            }
            let [_] = solids else {
                failed.push(format!(
                    "tab {radius} at {centre:?}, {rule:?}: {} solids",
                    solids.len()
                ));
                continue;
            };
            let (answer, its_error) = measured(&model, solids, &policy);
            if (answer - volume).abs() > error + its_error {
                failed.push(format!(
                    "tab {radius} at {centre:?}, {rule:?}: {answer} ± {its_error} against the \
                     plate's {volume} ± {error}"
                ));
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

/// A block `0.6 × 0.6` inside the tab at the corner, standing on the plate's
/// floor or hanging from its top, half the plate's height: the plate with it
/// taken away, with it added, and with the two shared, both ways round. The
/// block is inside the plate, so the union is the plate, the intersection is
/// the block, and `plate \ block` is the plate less it.
#[test]
fn a_block_inside_the_round_corner_tab_answers_every_rule_both_ways_round() {
    let policy = Policy::certified();
    let mut failed = Vec::new();
    for (floor, x, y) in [
        (0.0, 11.2, 8.2),
        (1.0, 11.2, 8.2),
        (0.0, 10.5, 11.0),
        (1.0, 11.5, 10.5),
    ] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let plate = a_plate_with_a_tab(&mut model, 3.0, [10.0, 10.0]);
        let block = a_block(&mut model, [x, y, floor], [x + 0.6, y + 0.6, floor + 1.0]);
        let (whole, error) = measured(&model, &[plate], &policy);
        let small = 0.6 * 0.6;
        for (rule, first, second, by_hand) in [
            (Rule::Union, plate, block, Some(whole)),
            (Rule::Union, block, plate, Some(whole)),
            (Rule::Intersection, plate, block, Some(small)),
            (Rule::Intersection, block, plate, Some(small)),
            (Rule::Difference, plate, block, Some(whole - small)),
            (Rule::Difference, block, plate, None),
        ] {
            let mut model = model.clone();
            let fused = match model.fuse(
                &[first],
                &[second],
                rule,
                &policy,
                &mut ProvenanceLog::new(),
            ) {
                Ok(fused) => fused,
                Err(why) => {
                    failed.push(format!(
                        "block at {x}, {y}, {floor}, {rule:?}: refused {why:?}"
                    ));
                    continue;
                }
            };
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|broken| {
                    panic!("block at {x}, {y}, {floor}, {rule:?}: {broken:?}")
                });
            let solids = fused.solids();
            let Some(by_hand) = by_hand else {
                if !solids.is_empty() {
                    failed.push(format!(
                        "block at {x}, {y}, {floor}, {rule:?}: {} solids",
                        solids.len()
                    ));
                }
                continue;
            };
            let [_] = solids else {
                failed.push(format!(
                    "block at {x}, {y}, {floor}, {rule:?}: {} solids",
                    solids.len()
                ));
                continue;
            };
            let (answer, its_error) = measured(&model, solids, &policy);
            if (answer - by_hand).abs() > error + its_error {
                failed.push(format!(
                    "block at {x}, {y}, {floor}, {rule:?}: {answer} ± {its_error} against \
                     {by_hand} ± {error}"
                ));
            }
        }
    }
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}

fn a_plate_with_a_tab(model: &mut Model, radius: f64, [x, y]: [f64; 2]) -> SolidKey {
    let square = a_block(model, [0.0, 0.0, 0.0], [10.0, 10.0, 2.0]);
    let tab = model
        .add_round_bar(
            radius,
            2.0,
            &Isometry::translation(Vector::new(x, y, 0.0)).expect("a finite shift"),
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
        .expect("a plate and its tab unite");
    let [plate] = fused.solids() else {
        panic!("a plate and its tab are {} solids", fused.solids().len());
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

/// The volume of `solids` together, and what the mesh may miss it by.
fn measured(model: &Model, solids: &[SolidKey], policy: &Policy) -> (f64, f64) {
    let (mut volume, mut error) = (0.0, 0.0);
    for &solid in solids {
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                policy,
                &mut ProvenanceLog::new(),
            )
            .expect("a solid measures");
        volume += measured.volume();
        error += measured.volume_error();
    }
    (volume, error)
}
