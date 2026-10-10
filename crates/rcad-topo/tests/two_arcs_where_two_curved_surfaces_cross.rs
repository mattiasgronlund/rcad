//! Probe.

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

fn what_each_rule_gives(
    build: impl Fn(&mut Model) -> [SolidKey; 2],
) -> Vec<Result<Vec<(f64, f64)>, Error>> {
    every_rule_both_ways()
        .map(|(rule, swapped)| {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let [first, second] = build(&mut model);
            let [one, other] = if swapped {
                [second, first]
            } else {
                [first, second]
            };
            let answer = model.fuse(
                &[one],
                &[other],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )?;
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("an answer validates");
            let mut volumes: Vec<(f64, f64)> = answer
                .solids()
                .iter()
                .map(|&solid| {
                    let measured = model
                        .solid(solid)
                        .expect("this model made the key")
                        .measure(
                            Deflection::of(0.01).expect("a fine mesh"),
                            &Policy::certified(),
                            &mut ProvenanceLog::new(),
                        );
                    match measured {
                        Ok(measured) => (measured.volume(), measured.volume_error()),
                        Err(why) => {
                            println!("    does not measure: {why:?}");
                            (f64::NAN, f64::NAN)
                        }
                    }
                })
                .collect();
            volumes.sort_by(|one, other| one.0.total_cmp(&other.0));
            Ok(volumes)
        })
        .collect()
}

fn every_rule_both_ways() -> impl Iterator<Item = (Rule, bool)> {
    [Rule::Union, Rule::Intersection, Rule::Difference]
        .into_iter()
        .flat_map(|rule| [false, true].map(move |swapped| (rule, swapped)))
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}

fn block(model: &mut Model, low: [f64; 3], high: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [low[0], low[1]],
        [high[0], low[1]],
        [high[0], high[1]],
        [low[0], high[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(&rectangle, high[2] - low[2], &at(0.0, 0.0, low[2]))
        .expect("a block")
}

fn cut(model: &mut Model, solid: SolidKey, low: [f64; 3], high: [f64; 3]) -> SolidKey {
    let block = block(model, low, high);
    let answer = model
        .fuse(
            &[solid],
            &[block],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("cutting: {why:?}"))
        .solids()
        .to_vec();
    let [solid] = answer[..] else {
        panic!("one solid, got {answer:?}");
    };
    solid
}

fn bar(model: &mut Model, radius: f64, low: f64, high: f64, seam: i32) -> SolidKey {
    model
        .add_round_bar(
            radius,
            high - low,
            &at(0.0, 0.0, low)
                .compose(&Isometry::quarter_turns(&Axis::z(), seam).expect("z is an axis")),
        )
        .expect("a bar")
}

fn ball(model: &mut Model, radius: f64, centre: [f64; 3]) -> SolidKey {
    model
        .add_ball(radius, &at(centre[0], centre[1], centre[2]))
        .expect("a ball")
}

fn thin_bar_along_x(model: &mut Model, off: f64, seam: i32) -> SolidKey {
    let placement = at(-10.0, off, 5.0)
        .compose(&Isometry::quarter_turns(&Axis::y(), 1).expect("y is an axis"))
        .compose(&Isometry::quarter_turns(&Axis::z(), seam).expect("z is an axis"));
    model
        .add_round_bar(1.0, 20.0, &placement)
        .expect("a thin bar")
}

const X_POS: ([f64; 3], [f64; 3]) = ([0.0, -20.0, -20.0], [20.0, 20.0, 20.0]);
const Y_POS: ([f64; 3], [f64; 3]) = ([-20.0, 0.0, -20.0], [20.0, 20.0, 20.0]);

#[test]
fn probe() {
    let placements: Vec<(&str, Box<dyn Fn(&mut Model) -> [SolidKey; 2]>)> = vec![
        (
            "half bar r4 x>=0, ball r5 on axis",
            Box::new(|model| {
                let bar = bar(model, 4.0, -1.0, 11.0, 2);
                [
                    cut(model, bar, X_POS.0, X_POS.1),
                    ball(model, 5.0, [0.0, 0.0, 5.0]),
                ]
            }),
        ),
        (
            "half ball r5 x>=0, bar r4 on axis",
            Box::new(|model| {
                let ball = ball(model, 5.0, [0.0, 0.0, 5.0]);
                [
                    cut(model, ball, X_POS.0, X_POS.1),
                    bar(model, 4.0, -1.0, 11.0, 2),
                ]
            }),
        ),
        (
            "half bar r2 x>=0, ball r3 off axis at (0,3,5)",
            Box::new(|model| {
                let bar = bar(model, 2.0, -1.0, 11.0, 2);
                [
                    cut(model, bar, X_POS.0, X_POS.1),
                    ball(model, 3.0, [0.0, 3.0, 5.0]),
                ]
            }),
        ),
        (
            "short bar r2 z<=5, ball r3 off axis at (0,3,5)",
            Box::new(|model| {
                [
                    bar(model, 2.0, -1.0, 5.0, 2),
                    ball(model, 3.0, [0.0, 3.0, 5.0]),
                ]
            }),
        ),
        (
            "half bar r2 x>=0, ball r5 off axis at (0,1,5): two loops",
            Box::new(|model| {
                let bar = bar(model, 2.0, -1.0, 11.0, 2);
                [
                    cut(model, bar, X_POS.0, X_POS.1),
                    ball(model, 5.0, [0.0, 1.0, 5.0]),
                ]
            }),
        ),
        (
            "fat bar seam 0, thin bar off 0 seam 0",
            Box::new(|model| {
                [
                    bar(model, 4.0, 0.0, 10.0, 0),
                    thin_bar_along_x(model, 0.0, 0),
                ]
            }),
        ),
        (
            "fat bar, half thin bar y>=0",
            Box::new(|model| {
                let thin = thin_bar_along_x(model, 0.0, 0);
                [
                    bar(model, 4.0, 0.0, 10.0, 1),
                    cut(model, thin, Y_POS.0, Y_POS.1),
                ]
            }),
        ),
        (
            "whole bar r2 seam +y, ball r3 off axis at (0,3,5)",
            Box::new(|model| {
                [
                    bar(model, 2.0, -1.0, 11.0, 1),
                    ball(model, 3.0, [0.0, 3.0, 5.0]),
                ]
            }),
        ),
        (
            "whole bar r2 seam +y, ball r5 off axis at (0,1,5): two loops",
            Box::new(|model| {
                [
                    bar(model, 2.0, -1.0, 11.0, 1),
                    ball(model, 5.0, [0.0, 1.0, 5.0]),
                ]
            }),
        ),
        (
            "whole bar r4 seam +x, ball r5 on axis",
            Box::new(|model| {
                [
                    bar(model, 4.0, -1.0, 11.0, 0),
                    ball(model, 5.0, [0.0, 0.0, 5.0]),
                ]
            }),
        ),
        (
            "fat bar, thin bar off 0 cut at x <= 3.9",
            Box::new(|model| {
                let thin = thin_bar_along_x(model, 0.0, 0);
                [
                    bar(model, 4.0, 0.0, 10.0, 1),
                    cut(model, thin, [-20.0, -20.0, -20.0], [3.9, 20.0, 20.0]),
                ]
            }),
        ),
        (
            "fat bar cut at z <= 5.5, thin bar off 0",
            Box::new(|model| {
                [
                    bar(model, 4.0, 0.0, 5.5, 1),
                    thin_bar_along_x(model, 0.0, 0),
                ]
            }),
        ),
        (
            "two bars r2 square, axes meeting",
            Box::new(|model| {
                let along_x = model
                    .add_round_bar(
                        2.0,
                        20.0,
                        &at(-10.0, 0.0, 5.0)
                            .compose(&Isometry::quarter_turns(&Axis::y(), 1).expect("y")),
                    )
                    .expect("a bar");
                [bar(model, 2.0, 0.0, 10.0, 1), along_x]
            }),
        ),
        (
            "bar r4 seam 0, ball r4 on axis (inscribed)",
            Box::new(|model| {
                [
                    bar(model, 4.0, -1.0, 11.0, 0),
                    ball(model, 4.0, [0.0, 0.0, 5.0]),
                ]
            }),
        ),
        (
            "half bar r4 x>=0, ball r4 on axis (inscribed)",
            Box::new(|model| {
                let bar = bar(model, 4.0, -1.0, 11.0, 2);
                [
                    cut(model, bar, X_POS.0, X_POS.1),
                    ball(model, 4.0, [0.0, 0.0, 5.0]),
                ]
            }),
        ),
    ];
    for (name, build) in placements {
        println!("{name}");
        for ((rule, swapped), gives) in every_rule_both_ways().zip(what_each_rule_gives(build)) {
            println!("  {rule:?} swapped {swapped}: {gives:?}");
        }
    }
}
