//! Probe, taken out before commit.

mod cones;
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

use pairs::{Configuration, Elsewhere, Placed, Shape, Skew};

fn the_combs() -> Configuration {
    Configuration {
        solids: vec![
            Placed {
                shape: Shape::Comb {
                    teeth: 2,
                    tooth_height: 2,
                },
                skew: Skew { x_per_y: -2 },
                height: 2,
                at: [0, 0, 0],
            },
            Placed {
                shape: Shape::Comb {
                    teeth: 1,
                    tooth_height: 1,
                },
                skew: Skew { x_per_y: 2 },
                height: 2,
                at: [-5, 2, 1],
            },
        ],
        turn: None,
        elsewhere: Elsewhere::here(),
    }
}

fn say(model: &Model, arguments: SolidKey, tools: SolidKey, rule: Rule, policy: &Policy) -> String {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    match model.fuse(&[arguments], &[tools], rule, policy, &mut log) {
        Ok(fused) => {
            let volumes: Vec<f64> = fused
                .solids()
                .iter()
                .map(|&solid| {
                    model
                        .solid(solid)
                        .expect("key")
                        .measure(
                            Deflection::of(0.1).expect("length"),
                            &Policy::certified(),
                            &mut ProvenanceLog::new(),
                        )
                        .map_or(f64::NAN, |measure| measure.volume())
                })
                .collect();
            let valid = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
            format!(
                "Ok {volumes:?} log {} validate {:?}",
                log.records().len(),
                valid.err()
            )
        }
        Err(Error::Undecidable { records }) => {
            format!("Undecidable ({} records)", records.records().len())
        }
        Err(other) => format!("{other:?} log {}", log.records().len()),
    }
}

#[test]
fn probe_q1614_the_combs() {
    let (model, keys) = the_combs().build();
    for (name, policy) in [
        ("certified", Policy::certified()),
        ("tolerant 25", Policy::tolerant(25.0)),
    ] {
        for (a, b) in [(0, 1), (1, 0)] {
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                println!(
                    "PROBE {name} {a}{b} {rule:?}: {}",
                    say(&model, keys[a], keys[b], rule, &policy)
                );
            }
        }
    }
}
