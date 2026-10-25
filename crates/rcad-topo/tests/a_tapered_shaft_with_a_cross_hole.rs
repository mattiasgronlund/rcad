//! Probe.

use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_pointed_cone};

fn shaft_and_pin(model: &mut Model) -> (SolidKey, SolidKey) {
    let shaft = a_frustum(model, 0.25, 4.0, 16.0);
    let along_y = Isometry::translation(Vector::new(0.0, -6.0, 10.0))
        .unwrap()
        .compose(&Isometry::quarter_turns(&Axis::x(), -1).unwrap());
    let pin = model.add_round_bar(0.5, 12.0, &along_y).unwrap();
    (shaft, pin)
}

fn cone_and_leaning_bar(model: &mut Model) -> (SolidKey, SolidKey) {
    let cone = a_pointed_cone(model, 0.5, 8.0);
    let through = Axis::new(Point::new(0.0, 0.0, 4.0), Vector::new(1.0, 0.0, 0.0)).unwrap();
    let placement = Isometry::rotation(&through, 0.3)
        .unwrap()
        .compose(&Isometry::translation(Vector::new(0.0, 0.0, -2.0)).unwrap());
    let bar = model.add_round_bar(1.0, 12.0, &placement).unwrap();
    (cone, bar)
}

fn cone_and_ball(model: &mut Model) -> (SolidKey, SolidKey) {
    let cone = a_pointed_cone(model, 0.5, 8.0);
    let ball = model
        .add_ball(
            2.0,
            &Isometry::translation(Vector::new(1.0, 0.0, 5.0)).unwrap(),
        )
        .unwrap();
    (cone, ball)
}

#[test]
fn probe_q2666_base() {
    for (name, shape) in [
        (
            "shaft and pin",
            shaft_and_pin as fn(&mut Model) -> (SolidKey, SolidKey),
        ),
        ("cone and leaning bar", cone_and_leaning_bar),
        ("cone and ball", cone_and_ball),
    ] {
        let mut model = Model::new(Scale::millimetres(100.0).unwrap());
        let (one, two) = shape(&mut model);
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for (argument, tool) in [(one, two), (two, one)] {
                let mut answer = model.clone();
                let fused = answer.fuse(
                    &[argument],
                    &[tool],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                println!(
                    "PROBE {name} {rule:?} cone-first={}: {:?}",
                    argument == one,
                    fused.as_ref().err()
                );
            }
        }
    }
}
