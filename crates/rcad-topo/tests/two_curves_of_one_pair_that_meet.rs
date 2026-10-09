//! Probe.

use rcad_geom::{Axis, Deflection, Isometry, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

mod cones;
mod rings;

use std::f64::consts::PI;

type Pair = fn(&mut Model) -> [SolidKey; 2];

#[test]
fn probe() {
    let shapes: [(&str, Pair, [f64; 3]); 15] = [
        (
            "p1 block along a bar",
            p1,
            [16.0 * PI, 80.0, 2.0 * (4.0 * PI / 3.0 - 3f64.sqrt())],
        ),
        (
            "p2 two bars side by side",
            p2,
            [
                16.0 * PI,
                8.0 * PI,
                2.0 * (8.0 * PI / 3.0 - 2.0 * 3f64.sqrt()),
            ],
        ),
        (
            "p3 ball on a bar's axis",
            p3,
            [
                32.0 * PI / 3.0,
                6.0 * PI,
                32.0 * PI / 3.0 - 4.0 * 3f64.sqrt() * PI,
            ],
        ),
        (
            "p4 cone and bar on one axis",
            p4,
            [64.0 * PI / 3.0, 8.0 * PI, 10.0 * PI / 3.0],
        ),
        (
            "p7 cone and block off the apex",
            p7,
            [64.0 * PI / 3.0, 5.0 * 12.0 * 8.0, cone_beyond(1.0, 4.0)],
        ),
        ("p8 viviani", p8, [32.0 * PI / 3.0, 6.0 * PI, f64::NAN]),
        (
            "p8b ball and bar two loops",
            p8b,
            [32.0 * PI / 3.0, 6.0 * PI, ball_and_bar(2.0, 1.0, 0.5)],
        ),
        ("p9 steinmetz", p9, [4.0 * PI, 4.0 * PI, 16.0 / 3.0]),
        (
            "p9b thin bar through a thick one",
            p9b,
            [16.0 * PI, 8.0 * PI, bar_through_bar(2.0, 1.0, 0.5)],
        ),
        (
            "p5 half ring and coaxial bar",
            p5,
            [
                4.0 * PI * PI,
                36.0 * PI * 2.5,
                4.0 * PI * PI - 8.0 * PI * (PI / 3.0 - 3f64.sqrt() / 4.0),
            ],
        ),
        (
            "p5b half ring and coaxial bar of 4",
            p5b,
            [4.0 * PI * PI, 16.0 * PI * 2.5, f64::NAN],
        ),
        (
            "p8c ball and bar at (0, 1/2)",
            p8c,
            [32.0 * PI / 3.0, 6.0 * PI, ball_and_bar(2.0, 1.0, 0.5)],
        ),
        (
            "p8d ball and thin bar clear of the poles",
            p8d,
            [32.0 * PI / 3.0, 1.5 * PI, ball_and_bar(2.0, 0.5, 1.0)],
        ),
        (
            "p8e ball turned, bar at x=1/2",
            p8e,
            [32.0 * PI / 3.0, 6.0 * PI, ball_and_bar(2.0, 1.0, 0.5)],
        ),
        (
            "p9c thin bar resting inside a thick one",
            p9c,
            [16.0 * PI, 8.0 * PI, f64::NAN],
        ),
    ];
    for (what, build, [first, second, shared]) in shapes {
        for (rule, first_first) in six_fuses() {
            let mut model = Model::new(rcad_geom::Scale::millimetres(100.0).unwrap());
            let [one, other] = build(&mut model);
            let (arguments, tools) = if first_first {
                ([one], [other])
            } else {
                ([other], [one])
            };
            let start = std::time::Instant::now();
            let said = model.fuse(
                &arguments,
                &tools,
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            let took = start.elapsed();
            let expected = match (rule, first_first) {
                (Rule::Union, _) => first + second - shared,
                (Rule::Intersection, _) => shared,
                (Rule::Difference, true) => first - shared,
                (Rule::Difference, false) => second - shared,
                _ => unreachable!(),
            };
            match said {
                Err(rcad_topo::Error::CannotTellTheCellsOfAFaceApart { face }) => {
                    let surface = model.face(face).map(|view| view.surface());
                    eprintln!(
                        "{what} {rule:?} {first_first}: refused cells on {surface:?} ({took:?})"
                    );
                }
                Err(why) => eprintln!("{what} {rule:?} {first_first}: refused {why:?} ({took:?})"),
                Ok(fused) => {
                    let solids = fused.solids().to_vec();
                    let valid = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
                    let mut total = 0.0;
                    let mut error = 0.0;
                    let mut faces = Vec::new();
                    for solid in &solids {
                        let view = model.solid(*solid).unwrap();
                        faces.push(view.faces().count());
                        match view.measure(
                            Deflection::of(0.01).unwrap(),
                            &Policy::certified(),
                            &mut ProvenanceLog::new(),
                        ) {
                            Ok(m) => {
                                total += m.volume();
                                error += m.volume_error();
                            }
                            Err(why) => eprintln!("   measure refused {why:?}"),
                        }
                    }
                    eprintln!(
                        "{what} {rule:?} {first_first}: {} solids faces {faces:?} valid {:?} volume {total} expected {expected} off {} error {error} ({took:?})",
                        solids.len(),
                        valid.is_ok(),
                        (total - expected).abs()
                    );
                }
            }
        }
    }
}

fn six_fuses() -> [(Rule, bool); 6] {
    [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Intersection, true),
        (Rule::Intersection, false),
        (Rule::Difference, true),
        (Rule::Difference, false),
    ]
}

fn a_block(model: &mut Model, from: [f64; 3], to: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [to[0] - from[0], 0.0],
        [to[0] - from[0], to[1] - from[1]],
        [0.0, to[1] - from[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle is a profile");
    let placed =
        Isometry::translation(Vector::new(from[0], from[1], from[2])).expect("a finite move");
    model
        .add_extrusion(&rectangle, to[2] - from[2], &placed)
        .expect("a rectangle extrudes")
}

fn a_bar(model: &mut Model, radius: f64, at: [f64; 3], height: f64) -> SolidKey {
    let placed = Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a finite move");
    model.add_round_bar(radius, height, &placed).expect("a bar")
}

fn a_bar_along_x(model: &mut Model, radius: f64, at: [f64; 3], length: f64) -> SolidKey {
    let turned = Isometry::quarter_turns(&Axis::y(), 1).expect("a turn about y");
    let placed = Isometry::translation(Vector::new(at[0], at[1], at[2]))
        .expect("a finite move")
        .compose(&turned);
    model.add_round_bar(radius, length, &placed).expect("a bar")
}

fn p1(model: &mut Model) -> [SolidKey; 2] {
    [
        a_bar(model, 2.0, [0.0, 0.0, 0.0], 4.0),
        a_block(model, [1.0, -5.0, 1.0], [5.0, 5.0, 3.0]),
    ]
}

fn p2(model: &mut Model) -> [SolidKey; 2] {
    [
        a_bar(model, 2.0, [0.0, 0.0, 0.0], 4.0),
        a_bar(model, 2.0, [2.0, 0.0, 1.0], 2.0),
    ]
}

fn p3(model: &mut Model) -> [SolidKey; 2] {
    let ball = model.add_ball(2.0, &Isometry::identity()).expect("a ball");
    [ball, a_bar(model, 1.0, [0.0, 0.0, -3.0], 6.0)]
}

fn p4(model: &mut Model) -> [SolidKey; 2] {
    let cone = cones::a_pointed_cone(model, 1.0, 4.0);
    [cone, a_bar(model, 1.0, [0.0, 0.0, -2.0], 8.0)]
}

fn p5(model: &mut Model) -> [SolidKey; 2] {
    let ring = rings::the_top_half_of_a_ring(model, 4.0, 1.0, false).expect("a half ring");
    [ring, a_bar(model, 6.0, [0.0, 0.0, -2.0], 2.5)]
}

fn p5b(model: &mut Model) -> [SolidKey; 2] {
    let ring = rings::the_top_half_of_a_ring(model, 4.0, 1.0, false).expect("a half ring");
    [ring, a_bar(model, 4.0, [0.0, 0.0, -2.0], 2.5)]
}

fn p8c(model: &mut Model) -> [SolidKey; 2] {
    let ball = model.add_ball(2.0, &Isometry::identity()).expect("a ball");
    [ball, a_bar(model, 1.0, [0.0, 0.5, -3.0], 6.0)]
}

fn p8d(model: &mut Model) -> [SolidKey; 2] {
    let ball = model.add_ball(2.0, &Isometry::identity()).expect("a ball");
    [ball, a_bar(model, 0.5, [0.0, 1.0, -3.0], 6.0)]
}

fn p8e(model: &mut Model) -> [SolidKey; 2] {
    let turned = Isometry::quarter_turns(&Axis::z(), 1).expect("a turn about z");
    let ball = model.add_ball(2.0, &turned).expect("a ball");
    [ball, a_bar(model, 1.0, [0.5, 0.0, -3.0], 6.0)]
}

fn p9c(model: &mut Model) -> [SolidKey; 2] {
    [
        a_bar(model, 2.0, [0.0, 0.0, -2.0], 4.0),
        a_bar_along_x(model, 1.0, [-4.0, 1.0, 0.0], 8.0),
    ]
}

fn p7(model: &mut Model) -> [SolidKey; 2] {
    let cone = cones::a_pointed_cone(model, 1.0, 4.0);
    [cone, a_block(model, [1.0, -6.0, -2.0], [6.0, 6.0, 6.0])]
}

fn p8(model: &mut Model) -> [SolidKey; 2] {
    let ball = model.add_ball(2.0, &Isometry::identity()).expect("a ball");
    [ball, a_bar(model, 1.0, [1.0, 0.0, -3.0], 6.0)]
}

fn p8b(model: &mut Model) -> [SolidKey; 2] {
    let ball = model.add_ball(2.0, &Isometry::identity()).expect("a ball");
    [ball, a_bar(model, 1.0, [0.5, 0.0, -3.0], 6.0)]
}

fn p9(model: &mut Model) -> [SolidKey; 2] {
    [
        a_bar(model, 1.0, [0.0, 0.0, -2.0], 4.0),
        a_bar_along_x(model, 1.0, [-2.0, 0.0, 0.0], 4.0),
    ]
}

fn p9b(model: &mut Model) -> [SolidKey; 2] {
    [
        a_bar(model, 2.0, [0.0, 0.0, -2.0], 4.0),
        a_bar_along_x(model, 1.0, [-4.0, 0.5, 0.0], 8.0),
    ]
}

/// Volume of the cone `ρ ≤ z`, `0 ≤ z ≤ far`, with `x ≥ d`.
fn cone_beyond(d: f64, far: f64) -> f64 {
    let root = (far * far - d * d).sqrt();
    let log = ((far + root) / d).ln();
    far.powi(3) / 3.0 * (d / far).acos()
        - d / 6.0 * (far * root + d * d * log)
        - d / 2.0 * (far * root - d * d * log)
}

/// Volume of the ball of `big` about the origin inside the bar of `small` about `x = off` along z.
fn ball_and_bar(big: f64, small: f64, off: f64) -> f64 {
    let steps = 20000;
    let mut sum = 0.0;
    for step in 0..steps {
        let theta = -PI / 2.0 + PI * (f64::from(step) + 0.5) / f64::from(steps);
        let x = off + small * theta.sin();
        let half = small * theta.cos();
        let a2 = big * big - x * x;
        let a = a2.sqrt();
        let across = 2.0 * (half * (a2 - half * half).sqrt() + a2 * (half / a).asin());
        sum += across * small * theta.cos() * PI / f64::from(steps);
    }
    sum
}

/// Volume shared by the bar of `big` along z and the bar of `small` along x at `y = off`.
fn bar_through_bar(big: f64, small: f64, off: f64) -> f64 {
    let steps = 20000;
    let mut sum = 0.0;
    for step in 0..steps {
        let theta = -PI / 2.0 + PI * (f64::from(step) + 0.5) / f64::from(steps);
        let y = off + small * theta.sin();
        let across = 2.0 * (big * big - y * y).sqrt() * 2.0 * small * theta.cos();
        sum += across * small * theta.cos() * PI / f64::from(steps);
    }
    sum
}
