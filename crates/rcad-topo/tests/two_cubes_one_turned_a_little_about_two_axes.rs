//! Probe for Q3195.

use rcad_geom::{Deflection, Isometry, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

fn turns() -> [(&'static str, [f64; 4]); 2] {
    let (cz, sz) = ((0.0043f64 / 2.0).cos(), (0.0043f64 / 2.0).sin());
    let (cx, sx) = ((0.0081f64 / 2.0).cos(), (0.0081f64 / 2.0).sin());
    let z_then_x = [cz * cx, cz * sx, sz * sx, sz * cx];
    let x_then_z = [cx * cz, sx * cz, -sx * sz, cx * sz];
    [("qz*qx", z_then_x), ("qx*qz", x_then_z)]
}

#[test]
fn probe_q3195_planes() {
    for (name, q) in turns() {
        for tx in [5.0, 5.000_000_1] {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a scale"));
            let block = a_cube(&mut model);
            let turned = a_cube(&mut model);
            let placement = Isometry::try_from(([tx, 0.0, 0.0], q)).expect("a placement");
            model.place_solid(turned, &placement).expect("a placement");
            println!("CASE {name} {tx:?} q={q:?}");
            for (which, solid) in [("block", block), ("turned", turned)] {
                for face in model.solid(solid).unwrap().faces() {
                    let plane = face.plane();
                    let sign = face.orientation().sign();
                    let [x, y, z] = plane.normal().components();
                    println!(
                        "PLANE {which} {:?} {:?} {:?} {:?} {:?}",
                        x,
                        y,
                        z,
                        plane.offset(),
                        sign
                    );
                }
            }
        }
    }
}

#[test]
fn probe_q3195_every_fuse() {
    let mut refused = 0;
    for (name, q) in turns() {
        for tx in [5.0, 5.000_000_1] {
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                for turned_first in [false, true] {
                    for (policy_name, policy) in [
                        ("certified", Policy::certified()),
                        ("tolerant", Policy::tolerant(100.0)),
                    ] {
                        eprintln!(
                            "START {name} tx={tx} {rule:?} turned_first={turned_first} {policy_name}"
                        );
                        let mut model = Model::new(Scale::millimetres(100.0).expect("a scale"));
                        let block = a_cube(&mut model);
                        let turned = a_cube(&mut model);
                        let placement =
                            Isometry::try_from(([tx, 0.0, 0.0], q)).expect("a placement");
                        model.place_solid(turned, &placement).expect("a placement");
                        let (a, b): (&[SolidKey], &[SolidKey]) = if turned_first {
                            (&[turned], &[block])
                        } else {
                            (&[block], &[turned])
                        };
                        let got = model.fuse(a, b, rule, &policy, &mut ProvenanceLog::new());
                        let said = match got {
                            Ok(answer) => {
                                for &s in answer.solids() {
                                    for face in model.solid(s).unwrap().faces() {
                                        let plane = face.plane();
                                        for wire in face.wires() {
                                            let near: Vec<_> = wire
                                                .points()
                                                .filter(|p| {
                                                    (p.x() - tx).abs() < 1e-9
                                                        && p.y().abs() < 1e-9
                                                        && p.z().abs() < 1e-9
                                                })
                                                .collect();
                                            if !near.is_empty() {
                                                eprintln!(
                                                    "NEAR face {:?} {:?} corners {near:?} of {}",
                                                    plane.normal(),
                                                    plane.offset(),
                                                    wire.points().count()
                                                );
                                            }
                                        }
                                    }
                                }
                                let valid = model.validate(&policy, &mut ProvenanceLog::new());
                                let volume: f64 = answer
                                    .solids()
                                    .iter()
                                    .map(|&s| {
                                        model
                                            .solid(s)
                                            .unwrap()
                                            .measure(
                                                Deflection::of(0.1).unwrap(),
                                                &Policy::certified(),
                                                &mut ProvenanceLog::new(),
                                            )
                                            .map(|m| m.volume())
                                            .unwrap_or(f64::NAN)
                                    })
                                    .sum();
                                format!("ok valid={:?} volume={volume}", valid.is_ok())
                            }
                            Err(why) => {
                                if policy_name == "certified" {
                                    refused += 1;
                                }
                                format!("ERR {why:?}")
                            }
                        };
                        eprintln!(
                            "PROBE {name} tx={tx} {rule:?} turned_first={turned_first} {policy_name} : {said}"
                        );
                    }
                }
            }
        }
    }
    eprintln!("PROBE certified refused {refused} of 24");
}

fn a_cube(model: &mut Model) -> SolidKey {
    model
        .add_cube(10.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube")
}
