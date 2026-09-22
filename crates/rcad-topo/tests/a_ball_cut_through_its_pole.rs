//! Probe.

use core::f64::consts::PI;
use std::collections::BTreeMap;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

#[test]
fn probe() {
    let mut tally: BTreeMap<String, u32> = BTreeMap::new();
    let one = std::env::var("PROBE_ONE").is_ok();
    for radius in if one { vec![1.0] } else { vec![0.5, 1.0, 2.0] } {
        for lean in if one { vec![0.7] } else { vec![0.3, 0.7, 1.1] } {
            for turn in if one { vec![0.0] } else { vec![0.0, 0.4, 1.3, PI / 2.0, 2.5, PI, -2.0] } {
                let mut model = Model::new(Scale::millimetres(25.0 * radius).unwrap());
                let ball = model.add_ball(radius, &Isometry::identity()).unwrap();
                let difference = std::env::var("PROBE_DIFF").is_ok();
                let other_side = std::env::var("PROBE_OTHER").is_ok();
                let knife = a_block_on_the_north_side(
                    &mut model,
                    radius,
                    if difference != other_side { PI + lean } else { lean },
                    turn,
                );
                let outcome = model.fuse(
                    &[ball],
                    &[knife],
                    if difference { Rule::Difference } else { Rule::Intersection },
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                let name = match outcome {
                    Ok(fused) => {
                        model
                            .validate(&Policy::certified(), &mut ProvenanceLog::new())
                            .unwrap();
                        let solids = fused.solids().to_vec();
                        let deflection = 0.001 * radius;
                        let found: f64 = solids
                            .iter()
                            .map(|&s| {
                                model
                                    .solid(s)
                                    .unwrap()
                                    .measure(
                                        Deflection::of(deflection).unwrap(),
                                        &Policy::certified(),
                                        &mut ProvenanceLog::new(),
                                    )
                                    .unwrap()
                                    .volume()
                            })
                            .sum();
                        let h = radius + radius * lean.cos();
                        let h = if other_side { 2.0 * radius - h } else { h };
                        let closed = PI * h * h * (3.0 * radius - h) / 3.0;
                        let under = (closed - found) / closed;
                        format!(
                            "answered {} solids, under {under:.2e} within {}",
                            solids.len(),
                            (0.0..3.0 * deflection / radius).contains(&under)
                        )
                    }
                    Err(why) => {
                        let text = format!("{why:?}");
                        text.split([' ', '{']).next().unwrap().to_owned()
                    }
                };
                eprintln!("radius {radius} lean {lean} turn {turn}: {name}");
                *tally
                    .entry(name.split(',').next().unwrap().to_owned())
                    .or_default() += 1;
            }
        }
    }
    eprintln!("{tally:?}");
}

fn a_block_on_the_north_side(model: &mut Model, radius: f64, lean: f64, turn: f64) -> SolidKey {
    let wide = 3.0 * radius;
    let square = ProfileBuilder::new(vec![[-wide, -wide], [wide, -wide], [wide, wide], [-wide, wide]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap();
    let lean_it = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(turn.sin(), -turn.cos(), 0.0)).unwrap(),
        lean,
    )
    .unwrap();
    let placement = Isometry::translation(Vector::new(0.0, 0.0, -radius))
        .unwrap()
        .compose(&lean_it);
    model.add_extrusion(&square, wide, &placement).unwrap()
}
