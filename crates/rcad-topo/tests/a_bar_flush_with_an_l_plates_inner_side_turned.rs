//! Probe for Q2505, unfinished: it prints what each fuse of the five cases
//! answers and asserts nothing. Replace it with the pins the row asks for.
//! Run one case with `./target/probe.sh '(1,1.5) 3pi/2' plate` (or `bar`).

use core::f64::consts::{FRAC_PI_2, PI};

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

#[test]
fn probe_q2505() {
    let cases = [
        ("(1,1.5) unturned", [1.0, 1.5], 0.0),
        ("(1.5,1) unturned", [1.5, 1.0], 0.0),
        ("(1,1.5) 0.3", [1.0, 1.5], 0.3),
        ("(1,1.5) 3pi/2", [1.0, 1.5], 1.5 * PI),
        ("(1,1.5) -pi/2", [1.0, 1.5], -FRAC_PI_2),
        ("(1.5,1) pi/2", [1.5, 1.0], FRAC_PI_2),
    ];
    for (name, at, turn) in cases {
        if std::env::var("PROBE_CASE").is_ok_and(|wanted| wanted != name) {
            continue;
        }
        {
            let (model, plate, bar) = the_two(at, turn);
            let plate: Vec<_> = model
                .solid(plate)
                .unwrap()
                .faces()
                .map(|f| f.surface())
                .collect();
            let bar: Vec<_> = model
                .solid(bar)
                .unwrap()
                .faces()
                .map(|f| f.surface())
                .collect();
            for one in &plate {
                for other in &bar {
                    let met = rcad_geom::intersect(
                        one,
                        other,
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    );
                    if !matches!(met, Ok(rcad_geom::SurfaceMeet::Nothing)) {
                        println!("PROBE meet {name}: {one:?} x {other:?} -> {met:?}");
                    }
                }
            }
        }
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for plate_first in [true, false] {
                if let Ok(one) = std::env::var("PROBE_ONE")
                    && (format!("{rule:?}") != "Union" || (one == "plate") != plate_first)
                {
                    continue;
                }
                let (mut model, plate, bar) = the_two(at, turn);
                let (first, second) = if plate_first {
                    (plate, bar)
                } else {
                    (bar, plate)
                };
                let mut log = ProvenanceLog::new();
                let fused = model.fuse(&[first], &[second], rule, &Policy::certified(), &mut log);
                let what = match fused {
                    Ok(fused) => {
                        let volumes: Vec<f64> = fused
                            .solids()
                            .iter()
                            .map(|&solid| {
                                model
                                    .solid(solid)
                                    .unwrap()
                                    .measure(
                                        Deflection::of(0.001).unwrap(),
                                        &Policy::certified(),
                                        &mut ProvenanceLog::new(),
                                    )
                                    .map(|m| m.volume())
                                    .unwrap_or(f64::NAN)
                            })
                            .collect();
                        format!("Ok {volumes:?}")
                    }
                    Err(why) => format!("Err {why:?}"),
                };
                println!("PROBE {name} {rule:?} plate_first={plate_first}: {what}");
            }
        }
    }
}

fn the_two(at: [f64; 2], turn: f64) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let l = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 2.0],
        [2.0, 2.0],
        [2.0, 4.0],
        [0.0, 4.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("an L");
    let plate = model
        .add_extrusion(&l, 1.0, &Isometry::identity())
        .expect("a plate");
    let bar = model
        .add_round_bar(
            0.5,
            3.5,
            &Isometry::translation(Vector::new(at[0], at[1], -1.0)).expect("a finite placement"),
        )
        .expect("a bar");
    let turn = Isometry::rotation(&Axis::z(), turn).expect("a finite angle");
    model.place_solid(plate, &turn).unwrap();
    model.place_solid(bar, &turn).unwrap();
    (model, plate, bar)
}
