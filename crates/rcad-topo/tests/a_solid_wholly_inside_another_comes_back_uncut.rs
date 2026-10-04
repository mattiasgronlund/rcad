//! Probe: every fuse of a solid wholly inside another, printed.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_pointed_cone};

#[derive(Clone, Copy, Debug)]
enum Pair {
    AFrustumInABar,
    APointedConeInABar,
    ABarInABall,
    ABoxInABall,
}

impl Pair {
    fn built(self, model: &mut Model) -> [SolidKey; 2] {
        match self {
            Self::AFrustumInABar => {
                let bar = model
                    .add_round_bar(4.0, 6.0, &at(0.0, 0.0, 1.0))
                    .expect("a bar");
                [bar, a_frustum(model, 0.5, 2.0, 6.0)]
            }
            Self::APointedConeInABar => {
                let bar = model
                    .add_round_bar(3.0, 6.0, &at(0.0, 0.0, -1.0))
                    .expect("a bar");
                [bar, a_pointed_cone(model, 0.5, 4.0)]
            }
            Self::ABarInABall => {
                let ball = model.add_ball(5.0, &at(0.0, 0.0, 0.0)).expect("a ball");
                let bar = model
                    .add_round_bar(3.0, 4.0, &at(0.0, 0.0, -2.0))
                    .expect("a bar");
                [ball, bar]
            }
            Self::ABoxInABall => {
                let ball = model.add_ball(3.0, &at(0.0, 0.0, 0.0)).expect("a ball");
                let cube = model
                    .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
                    .expect("a cube");
                model
                    .place_solid(cube, &at(-1.0, -1.0, -1.0))
                    .expect("this model made the key");
                [ball, cube]
            }
        }
    }

    fn volumes(self) -> [f64; 2] {
        match self {
            Self::AFrustumInABar => [96.0 * PI, 52.0 * PI / 3.0],
            Self::APointedConeInABar => [54.0 * PI, 16.0 * PI / 3.0],
            Self::ABarInABall => [500.0 * PI / 3.0, 36.0 * PI],
            Self::ABoxInABall => [36.0 * PI, 8.0],
        }
    }
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}

#[test]
fn every_fuse_printed() {
    for pair in [
        Pair::AFrustumInABar,
        Pair::APointedConeInABar,
        Pair::ABarInABall,
        Pair::ABoxInABall,
    ] {
        let [outer_volume, inner_volume] = pair.volumes();
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for outer_first in [true, false] {
                let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
                let [outer, inner] = pair.built(&mut model);
                let [first, second] = if outer_first {
                    [outer, inner]
                } else {
                    [inner, outer]
                };
                let answer = model.fuse(
                    &[first],
                    &[second],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                let line = match answer {
                    Err(why) => format!("refused {why:?}"),
                    Ok(fused) => {
                        let mut said = String::new();
                        for &solid in fused.solids() {
                            let view = model.solid(solid).expect("this model made the key");
                            let (flat, curved): (Vec<_>, Vec<_>) = view
                                .faces()
                                .partition(|face| face.surface().as_plane().is_some());
                            let measured = view.measure(
                                Deflection::of(0.01).expect("a length"),
                                &Policy::certified(),
                                &mut ProvenanceLog::new(),
                            );
                            let volume = match measured {
                                Ok(measured) => format!(
                                    "{:.3} give or take {:.3}",
                                    measured.volume(),
                                    measured.volume_error()
                                ),
                                Err(why) => format!("does not measure {why:?}"),
                            };
                            said.push_str(&format!(
                                "[{} shells, {} flat, {} curved, {volume}] ",
                                view.shells().count(),
                                flat.len(),
                                curved.len()
                            ));
                        }
                        format!("{} solids {said}", fused.solids().len())
                    }
                };
                println!(
                    "{pair:?} {rule:?} outer first {outer_first}: {line} (outer {outer_volume:.3}, \
                     inner {inner_volume:.3}, outer less inner {:.3})",
                    outer_volume - inner_volume
                );
            }
        }
    }
}
