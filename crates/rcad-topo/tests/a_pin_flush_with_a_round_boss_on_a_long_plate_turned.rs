//! PROBE Q3267, not to be committed as is.

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

fn probe_outcome(why: &rcad_topo::Error) -> String {
    match why {
        rcad_topo::Error::Undecidable { records } => format!(
            "Undecidable {:?}",
            records
                .records()
                .iter()
                .map(|r| r.predicate())
                .collect::<Vec<_>>()
        ),
        other => format!("{other:?}").chars().take(200).collect::<String>(),
    }
}

fn probe_run(
    model: &mut Model,
    first: SolidKey,
    steps: &[(&str, SolidKey, Rule)],
    meant: f64,
) -> String {
    let mut now = vec![first];
    for &(name, next, rule) in steps {
        match model.fuse(
            &now,
            &[next],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) {
            Ok(f) => now = f.solids().to_vec(),
            Err(why) => return format!("{name}: {}", probe_outcome(&why)),
        }
    }
    if let Err(why) = model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
        return format!("answers {} solids, validate: {why:?}", now.len());
    }
    let faces: Vec<usize> = now
        .iter()
        .map(|s| model.solid(*s).expect("a solid").faces().count())
        .collect();
    let mut said = format!("answers faces {faces:?} validates");
    let all: Result<Vec<_>, _> = now
        .iter()
        .map(|&solid| {
            model.solid(solid).expect("made here").measure(
                Deflection::of(0.02).expect("a mesh"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
        })
        .collect();
    match all {
        Ok(all) => {
            let volume: f64 = all.iter().map(|m| m.volume()).sum();
            let error: f64 = all.iter().map(|m| m.volume_error()).sum();
            let within = (volume - meant).abs() <= error;
            said += &format!("; {volume:.5} +- {error:.5} against {meant} within {within}");
        }
        Err(why) => said += &format!("; measure: {}", probe_outcome(&why)),
    }
    said
}

#[test]
fn probe_q3267_flush_pin() {
    let turns: Vec<f64> = std::env::var("Q3267_TURNS").map_or_else(
        |_| vec![0.1, 0.2, 0.4, 1.0, 2.0, 4.7],
        |t| t.split(',').map(|x| x.parse().expect("a turn")).collect(),
    );
    for turn in turns {
        for plate in [5.0, 50.0] {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
            let thick = a_bar(&mut model, 2.0, 4.0, [0.0, 0.0, -2.0], turn);
            let pin = a_bar(&mut model, 1.0, 6.0, [-1.0, 0.0, -3.0], turn);
            let rectangle =
                ProfileBuilder::new(vec![[0.0, -1.5], [plate, -1.5], [plate, 1.5], [0.0, 1.5]])
                    .build(&Policy::certified(), &mut ProvenanceLog::new())
                    .expect("a rectangle");
            let block = model
                .add_extrusion(&rectangle, 4.0, &turned(turn, [0.0, 0.0, -2.0]))
                .expect("a block");
            let square =
                ProfileBuilder::new(vec![[-1.0, -0.5], [1.0, -0.5], [1.0, 0.5], [-1.0, 0.5]])
                    .build(&Policy::certified(), &mut ProvenanceLog::new())
                    .expect("a square");
            let boss = model
                .add_extrusion(&square, 2.0, &turned(turn, [plate + 1.0, 0.0, -1.0]))
                .expect("a boss");
            let both = 1.5 * 1.75_f64.sqrt() + 4.0 * 0.75_f64.asin();
            let meant = 4.0 * (4.0 * PI + plate * 3.0 - both) + 2.0 * PI + 4.0;
            let steps = [
                ("plate", block, Rule::Union),
                ("pin", pin, Rule::Union),
                ("boss", boss, Rule::Union),
            ];
            let outcome = probe_run(&mut model, thick, &steps, meant);
            println!("PROBE flush pin turn {turn} plate {plate}: {outcome}");
        }
    }
}

fn a_bar(model: &mut Model, radius: f64, height: f64, foot: [f64; 3], turn: f64) -> SolidKey {
    model
        .add_round_bar(radius, height, &turned(turn, foot))
        .expect("a bar")
}

fn turned(turn: f64, at: [f64; 3]) -> Isometry {
    Isometry::rotation(&Axis::z(), turn)
        .expect("a finite angle")
        .compose(&Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a finite move"))
}
