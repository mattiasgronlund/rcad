//! probe

use std::panic::{AssertUnwindSafe, catch_unwind};

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

#[test]
fn probe_every_shape() {
    let shapes: [(&str, f64, f64, f64, f64); 6] = [
        ("inside pocket", 1.0, 0.0, -1.0, 6.0),
        ("inside rising", 1.0, 0.0, 1.0, 5.0),
        ("corner pocket", 1.0, -3.0, -1.0, 6.0),
        ("corner rising", 1.0, -3.0, 1.0, 5.0),
        ("whole chord pocket", 4.0, 0.0, -1.0, 6.0),
        ("whole chord rising", 4.0, 0.0, 1.0, 5.0),
    ];
    for (name, radius, y, bottom, height) in shapes {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for left_first in [true, false] {
                let outcome = catch_unwind(AssertUnwindSafe(|| {
                    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
                    let left = a_left_d(&mut model);
                    let bite = a_half_bar_bulging_left(&mut model, radius, y, bottom, height);
                    let pocket = bottom < 0.0;
                    let left = fuse(
                        &mut model,
                        left,
                        bite,
                        if pocket { Rule::Difference } else { Rule::Union },
                    )
                    .map_err(|why| format!("building the left side: {why:?}"))?;
                    let [left] = left[..] else {
                        return Err(format!("the left side is {left:?}"));
                    };
                    let right = a_right_d(&mut model);
                    let [one, other] = if left_first { [left, right] } else { [right, left] };
                    let answer = fuse(&mut model, one, other, rule).map_err(|why| format!("{why:?}"))?;
                    let valid = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
                    let volumes: Vec<String> = answer
                        .iter()
                        .map(|&solid| {
                            match model.solid(solid).expect("a key").measure(
                                Deflection::of(0.001).expect("fine"),
                                &Policy::certified(),
                                &mut ProvenanceLog::new(),
                            ) {
                                Ok(m) => format!("{} ± {}", m.volume(), m.volume_error()),
                                Err(e) => format!("unmeasured {e:?}"),
                            }
                        })
                        .collect();
                    Ok(format!("{} solids {volumes:?}, validate {valid:?}", answer.len()))
                }));
                let said = match outcome {
                    Ok(Ok(s)) => s,
                    Ok(Err(s)) => format!("refused {s}"),
                    Err(p) => format!(
                        "PANIC {}",
                        p.downcast_ref::<String>()
                            .cloned()
                            .or_else(|| p.downcast_ref::<&str>().map(|s| (*s).to_string()))
                            .unwrap_or_default()
                    ),
                };
                println!("{name}, {rule:?}, left first {left_first}: {said}");
            }
        }
    }
}

fn a_left_d(model: &mut Model) -> SolidKey {
    let bar = model.add_round_bar(5.0, 4.0, &at(-3.0, 0.0, 0.0)).expect("a bar");
    let block = a_block(model, [0.0, -20.0], [20.0, 20.0]);
    let [d] = fuse(model, bar, block, Rule::Difference).expect("a D")[..] else {
        panic!("one D")
    };
    d
}

fn a_right_d(model: &mut Model) -> SolidKey {
    let bar = model.add_round_bar(5.0, 4.0, &at(3.0, 0.0, 0.0)).expect("a bar");
    let block = a_block(model, [-20.0, -20.0], [0.0, 20.0]);
    let [d] = fuse(model, bar, block, Rule::Difference).expect("a D")[..] else {
        panic!("one D")
    };
    d
}

fn a_half_bar_bulging_left(model: &mut Model, radius: f64, y: f64, bottom: f64, height: f64) -> SolidKey {
    let bar = model.add_round_bar(radius, height, &at(0.0, y, bottom)).expect("a bar");
    let block = a_block(model, [0.0, -20.0], [20.0, 20.0]);
    let [d] = fuse(model, bar, block, Rule::Difference).expect("a half bar")[..] else {
        panic!("one half bar")
    };
    d
}

fn fuse(
    model: &mut Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
) -> rcad_topo::Result<Vec<SolidKey>> {
    model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().to_vec())
}

fn a_block(model: &mut Model, low: [f64; 2], high: [f64; 2]) -> SolidKey {
    let square = ProfileBuilder::new(vec![
        [low[0], low[1]],
        [high[0], low[1]],
        [high[0], high[1]],
        [low[0], high[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(&square, 20.0, &at(0.0, 0.0, -5.0))
        .expect("a positive height")
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift")
}
