//! Probe, to be replaced by pins.

mod cones;
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

use pairs::{Configuration, Elsewhere, Placed, Shape, Skew};

#[test]
fn probe() {
    for (name, pair) in [
        ("frame 4x3 / comb 3", frame_and_comb()),
        ("comb 2 / ell 5x4", comb_and_ell()),
        ("frame 3x3 / ell 4x4", frame_and_ell()),
        ("comb 3 / comb 2", comb_and_comb()),
    ] {
        let (model, keys) = pair.build();
        println!(
            "{name}: A {} B {}",
            volume(&model, &keys[..1]),
            volume(&model, &keys[1..])
        );
        for (first, second) in [(0, 1), (1, 0)] {
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                let mut copy = model.clone();
                let mut log = ProvenanceLog::new();
                match copy.fuse(
                    &[keys[first]],
                    &[keys[second]],
                    rule,
                    &Policy::certified(),
                    &mut log,
                ) {
                    Ok(fused) => {
                        let solids = fused.solids().to_vec();
                        let valid = copy
                            .validate(&Policy::certified(), &mut ProvenanceLog::new())
                            .map_err(|why| format!("{why:?}"));
                        println!(
                            "  {first}{second} {rule:?}: {} solids, volume {}, log {}, valid {valid:?}",
                            solids.len(),
                            volume(&copy, &solids),
                            log.len()
                        );
                    }
                    Err(Error::Undecidable { records }) => {
                        println!("  {first}{second} {rule:?}: undecidable");
                        for record in records.records() {
                            println!("    {} at {:?}", record.predicate(), record.inputs());
                        }
                    }
                    Err(why) => println!("  {first}{second} {rule:?}: {why:?}"),
                }
            }
        }
    }
}

fn volume(model: &Model, solids: &[SolidKey]) -> f64 {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.1).expect("a tenth of a millimetre is a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a flat solid this size measures")
                .volume()
        })
        .sum()
}

fn configuration(first: Placed, second: Placed) -> Configuration {
    Configuration {
        solids: vec![first, second],
        turn: None,
        elsewhere: Elsewhere::here(),
    }
}

fn frame_and_comb() -> Configuration {
    configuration(
        Placed {
            shape: Shape::Frame { width: 4, depth: 3 },
            skew: Skew { x_per_y: -1 },
            height: 1,
            at: [0, 0, 0],
        },
        Placed {
            shape: Shape::Comb {
                teeth: 3,
                tooth_height: 2,
            },
            skew: Skew::none(),
            height: 1,
            at: [-2, -2, 0],
        },
    )
}

fn comb_and_ell() -> Configuration {
    configuration(
        Placed {
            shape: Shape::Comb {
                teeth: 2,
                tooth_height: 3,
            },
            skew: Skew { x_per_y: 1 },
            height: 1,
            at: [0, 0, 0],
        },
        Placed {
            shape: Shape::Ell {
                width: 5,
                depth: 4,
                bite_width: 1,
                bite_depth: 3,
            },
            skew: Skew { x_per_y: -2 },
            height: 3,
            at: [1, 1, 1],
        },
    )
}

fn frame_and_ell() -> Configuration {
    configuration(
        Placed {
            shape: Shape::Frame { width: 3, depth: 3 },
            skew: Skew { x_per_y: 1 },
            height: 2,
            at: [0, 0, 0],
        },
        Placed {
            shape: Shape::Ell {
                width: 4,
                depth: 4,
                bite_width: 3,
                bite_depth: 1,
            },
            skew: Skew { x_per_y: -2 },
            height: 1,
            at: [3, 1, 1],
        },
    )
}

fn comb_and_comb() -> Configuration {
    configuration(
        Placed {
            shape: Shape::Comb {
                teeth: 3,
                tooth_height: 2,
            },
            skew: Skew { x_per_y: 2 },
            height: 2,
            at: [0, 0, 0],
        },
        Placed {
            shape: Shape::Comb {
                teeth: 2,
                tooth_height: 2,
            },
            skew: Skew { x_per_y: -2 },
            height: 1,
            at: [3, 0, 1],
        },
    )
}
