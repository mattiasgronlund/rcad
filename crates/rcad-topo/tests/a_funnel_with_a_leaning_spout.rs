//! Probe for Q2667.

use rcad_geom::Isometry;
use rcad_geom::{Axis, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_pointed_cone};

#[test]
fn probe_q2667() {
    for (name, shape) in [
        (
            "funnel and spout",
            funnel_and_spout as fn(&mut Model) -> (SolidKey, SolidKey),
        ),
        ("two pointed cones", two_pointed_cones),
    ] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for first in [true, false] {
                let policy = Policy::certified();
                let mut model =
                    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
                let (one, other) = shape(&mut model);
                let (a, b) = if first { (one, other) } else { (other, one) };
                let said = model
                    .fuse(&[a], &[b], rule, &policy, &mut ProvenanceLog::new())
                    .map(|fused| fused.solids().len());
                println!("PROBE {name} {rule:?} first={first}: {said:?}");
            }
        }
    }
}

fn funnel_and_spout(model: &mut Model) -> (SolidKey, SolidKey) {
    let funnel = a_frustum(model, 0.5, 2.0, 8.0);
    let spout = a_frustum(model, 0.125, 2.0, 12.0);
    let through =
        Axis::new(Point::new(0.0, 0.0, 4.0), Vector::new(1.0, 0.0, 0.0)).expect("an axis");
    model
        .place_solid(spout, &Isometry::rotation(&through, 0.8).expect("a turn"))
        .expect("this model made the key");
    (funnel, spout)
}

fn two_pointed_cones(model: &mut Model) -> (SolidKey, SolidKey) {
    let upright = a_pointed_cone(model, 0.5, 8.0);
    let leaning = a_pointed_cone(model, 0.5, 8.0);
    let through =
        Axis::new(Point::new(0.0, 0.0, 4.0), Vector::new(1.0, 0.0, 0.0)).expect("an axis");
    model
        .place_solid(leaning, &Isometry::rotation(&through, 0.3).expect("a turn"))
        .expect("this model made the key");
    (upright, leaning)
}
