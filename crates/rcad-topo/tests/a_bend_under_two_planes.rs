//! Probe file, being written.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

mod rings;

#[test]
fn probe_the_bend_and_the_bar_of_its_own_radius() {
    for (ring, tube) in [(4.0, 1.0), (6.0, 2.0)] {
        let bend = PI * PI * ring * tube * tube / 2.0;
        let bar = PI * ring * ring * 4.0 * tube;
        let shared = (PI * PI * ring * tube * tube - 4.0 * PI * tube * tube * tube / 3.0) / 4.0;
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for bend_first in [true, false] {
                let mut model = Model::new(Scale::millimetres(100.0).expect("a size"));
                let b = rings::a_quarter_bend(&mut model, ring, tube);
                let r = model
                    .add_round_bar(ring, 4.0 * tube, &at(0.0, 0.0, -2.0 * tube))
                    .expect("bar");
                let [one, other] = if bend_first { [b, r] } else { [r, b] };
                let expected = match (rule, bend_first) {
                    (Rule::Union, _) => bend + bar - shared,
                    (Rule::Intersection, _) => shared,
                    (Rule::Difference, true) => bend - shared,
                    (Rule::Difference, false) => bar - shared,
                    _ => unreachable!(),
                };
                report(
                    &mut model,
                    one,
                    other,
                    rule,
                    expected,
                    &format!("bar ring {ring} tube {tube} {rule:?} bend first {bend_first}"),
                );
            }
        }
    }
}

#[test]
fn probe_a_half_bend_under_a_half_bar() {
    let (ring, tube, t) = (4.0, 1.25, 0.75);
    let s = (tube * tube - t * t as f64).sqrt();
    let segment = tube * tube * (t / tube).acos() - t * s;
    let shared = segment * ring * PI / 2.0;
    let bend = PI * PI * ring * tube * tube;
    let half_bar = PI * 100.0 / 2.0 * (5.0 - t);
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for bend_first in [true, false] {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a size"));
            let b = rings::a_bend_turned_and_written_out(&mut model, ring, tube, 2, 0)
                .expect("a half bend");
            let bar = model
                .add_round_bar(10.0, 5.0 - t, &at(0.0, 0.0, t))
                .expect("bar");
            let square =
                ProfileBuilder::new(vec![[0.0, -20.0], [20.0, -20.0], [20.0, 20.0], [0.0, 20.0]])
                    .build(&Policy::certified(), &mut ProvenanceLog::new())
                    .expect("a square");
            let block = model
                .add_extrusion(&square, 7.0, &at(0.0, 0.0, -1.0))
                .expect("block");
            let fused = model
                .fuse(
                    &[bar],
                    &[block],
                    Rule::Difference,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("half bar");
            let [h] = fused.solids() else { panic!() };
            let h = *h;
            let [one, other] = if bend_first { [b, h] } else { [h, b] };
            let expected = match (rule, bend_first) {
                (Rule::Union, _) => bend + half_bar - shared,
                (Rule::Intersection, _) => shared,
                (Rule::Difference, true) => bend - shared,
                (Rule::Difference, false) => half_bar - shared,
                _ => unreachable!(),
            };
            report(
                &mut model,
                one,
                other,
                rule,
                expected,
                &format!("half bar {rule:?} bend first {bend_first}"),
            );
        }
    }
}

fn report(
    model: &mut Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
    expected: f64,
    what: &str,
) {
    let fused = model.fuse(
        &[one],
        &[other],
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    let fused = match fused {
        Ok(fused) => fused,
        Err(why) => {
            eprintln!("PROBE {what}: refused {why:?}");
            return;
        }
    };
    let valid = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
    let mut measured = 0.0;
    let mut allowed = 0.0;
    for solid in fused.solids() {
        match model.solid(*solid).expect("key").measure(
            Deflection::of(0.01).expect("d"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) {
            Ok(held) => {
                measured += held.volume();
                allowed += held.volume_error();
            }
            Err(why) => {
                eprintln!("PROBE {what}: does not measure {why:?}");
                return;
            }
        }
    }
    eprintln!(
        "PROBE {what}: {} solids, valid {:?}, measured {measured} expected {expected} allowed {allowed} ok {}",
        fused.solids().len(),
        valid.map(|_| ()),
        (measured - expected).abs() <= allowed
    );
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a shift")
}
