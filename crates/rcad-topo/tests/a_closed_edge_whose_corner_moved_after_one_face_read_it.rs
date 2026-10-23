//! A round bar of radius 3 leaning across a round bar of radius 2 whose seam
//! it presses into, fused either way round (row Q2571).
//!
//! # The shape
//!
//! The upright bar is radius 2, up `z` from −1 to 11, turned a quarter round
//! `z` so that its seam runs up `(0, 2, z)`. The leaning bar is radius 3 and
//! 20 long, its axis through `(0, 4, 5)` along `(cos 0.3, 0, −sin 0.3)`. Their
//! axes pass 4 apart, inside the 5 their radii add to, so the leaning bar
//! presses 1 into the upright one and the two pipes cross in one closed loop.
//! The upright bar's seam crosses that loop twice, at `(0, 2, 5 ± √5/cos 0.3)`,
//! and at both corners the loop runs exactly along the leaning pipe's ruling.
//!
//! # Why one order was refused
//!
//! With the upright bar first, the intersection was refused
//! `TheFuseWroteABrokenAnswer { broken: CoedgesRunTheSameWay }` and the union
//! and the difference `AWireWillNotLieFlatOnItsFace`; the leaning bar first
//! answered all three. `Edges::of` in `fuse/assemble.rs` read which way a
//! closed side is walked at its corner as the corner stood when the first face
//! asked for it, and `Corners::of` moved that corner afterwards, to the lower
//! rounding the other piece's meet gave it. The upright pipe's side was read at
//! `x = 4.695e-16` and its edge written at `x = 5.869e-17`. The way is the
//! sign of a dot product that is zero at the true corner, since the loop runs
//! along the ruling there, and its sign differed between the two roundings
//! (`+4.4e-16` and `−2.7e-15`), so the two faces walked the one edge the same
//! way. The corners are now all settled before any closed side is read.
//!
//! The fuses run certified, in a model of 100 mm.

use std::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule};

/// How far the leaning bar's axis leans down from `x`, in radians.
const LEAN: f64 = 0.3;

/// The two bars share the same piece whichever is handed in first, and it
/// measures what they share.
#[test]
fn the_piece_two_bars_share_is_the_same_either_way_round() {
    let shared = what_the_two_bars_share(1000);
    let upright_first = answer(Rule::Intersection, true, shared, 2);
    let leaning_first = answer(Rule::Intersection, false, shared, 2);
    the_same_volume(upright_first, leaning_first, "the intersection");
}

/// The two bars unite to the same solid whichever is handed in first: each
/// pipe with a hole, and the four caps.
#[test]
fn two_bars_unite_to_the_same_solid_either_way_round() {
    let whole = UPRIGHT + LEANING - what_the_two_bars_share(1000);
    let upright_first = answer(Rule::Union, true, whole, 6);
    let leaning_first = answer(Rule::Union, false, whole, 6);
    the_same_volume(upright_first, leaning_first, "the union");
}

/// Each bar less the other answers, and measures its own volume less what
/// the two share: its pipe with a hole, its two caps, and the dent.
#[test]
fn each_bar_less_the_other_answers() {
    let shared = what_the_two_bars_share(1000);
    answer(Rule::Difference, true, UPRIGHT - shared, 4);
    answer(Rule::Difference, false, LEANING - shared, 4);
}

/// The upright bar's volume, `π · 2² · 12`.
const UPRIGHT: f64 = 48.0 * PI;

/// The leaning bar's volume, `π · 3² · 20`.
const LEANING: f64 = 180.0 * PI;

/// What a measured volume is, and how far its mesh may miss it.
#[derive(Clone, Copy, Debug)]
struct Measured {
    volume: f64,
    error: f64,
}

/// The fuse `rule` of the two bars, the upright one first or not: assert that
/// it answers one solid of `faces` faces, that the model validates, and that
/// the solid measures `volume` to within its mesh's own error. Hands back what
/// it measured.
fn answer(rule: Rule, upright_first: bool, volume: f64, faces: usize) -> Measured {
    let what = format!("{rule:?}, upright bar first: {upright_first}");
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let upright = model
        .add_round_bar(2.0, 12.0, &seamed_up_y())
        .expect("a bar");
    let leaning = model
        .add_round_bar(3.0, 20.0, &leaning_across())
        .expect("a bar");
    let (arguments, tools) = if upright_first {
        ([upright], [leaning])
    } else {
        ([leaning], [upright])
    };
    let solids = model
        .fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: refused: {why:?}"))
        .solids()
        .to_vec();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: the answer does not validate: {why:?}"));
    let [solid] = solids[..] else {
        panic!("{what}: one solid, got {solids:?}");
    };
    let measure = model
        .solid(solid)
        .expect("a solid the fuse made")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the answer does not measure: {why:?}"));
    let measured = Measured {
        volume: measure.volume(),
        error: measure.volume_error(),
    };
    assert_eq!(
        model
            .solid(solid)
            .expect("a solid the fuse made")
            .faces()
            .count(),
        faces,
        "{what}: faces"
    );
    assert!(
        (measured.volume - volume).abs() <= measured.error,
        "{what}: measured {measured:?} against {volume}"
    );
    measured
}

/// Assert that the two orders' answers measure the same, to within the two
/// meshes' errors added.
fn the_same_volume(upright_first: Measured, leaning_first: Measured, what: &str) {
    assert!(
        (upright_first.volume - leaning_first.volume).abs()
            <= upright_first.error + leaning_first.error,
        "{what}: upright bar first {upright_first:?}, leaning bar first {leaning_first:?}"
    );
}

/// The upright bar, up `z` from −1 to 11, turned a quarter round `z` so that
/// its seam runs up `(0, 2, z)`.
fn seamed_up_y() -> Isometry {
    moved_to([0.0, 0.0, -1.0])
        .compose(&Isometry::quarter_turns(&Axis::z(), 1).expect("a turn about z"))
}

/// The leaning bar, its axis through `(0, 4, 5)` along `(cos 0.3, 0, −sin
/// 0.3)`, reaching 10 each way from there.
fn leaning_across() -> Isometry {
    let the_y_axis = Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("an axis");
    moved_to([0.0, 4.0, 5.0])
        .compose(&Isometry::rotation(&the_y_axis, PI / 2.0 + LEAN).expect("a turn"))
        .compose(&moved_to([0.0, 0.0, -10.0]))
}

fn moved_to(at: [f64; 3]) -> Isometry {
    Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a finite move")
}

/// What the two bars share, by a midpoint rule of `steps` steps.
///
/// A line up `z` through `(x, y)` inside the upright bar is inside the leaning
/// one where `(y − 4)² + (x·sin 0.3 + (z − 5)·cos 0.3)² ≤ 9`, a length of
/// `2√(9 − (y − 4)²)/cos 0.3`, for `y` from 1 to 2, well inside both bars'
/// ends. Across `x` the upright bar is `2√(4 − y²)` wide, so the volume is
/// `(4/cos 0.3) ∫₁² √((2 − y)(y − 1)) √((2 + y)(7 − y)) dy`. Put
/// `y = 1.5 − cos(t)/2`: the first root is `sin(t)/2` and `dy` is `sin(t)/2 dt`,
/// which leaves a smooth integrand over `t` from 0 to π.
fn what_the_two_bars_share(steps: u32) -> f64 {
    let step = PI / f64::from(steps);
    let sum: f64 = (0..steps)
        .map(|at| {
            let t = step * (f64::from(at) + 0.5);
            let y = 1.5 - t.cos() / 2.0;
            t.sin().powi(2) / 4.0 * ((2.0 + y) * (7.0 - y)).sqrt()
        })
        .sum();
    4.0 / LEAN.cos() * sum * step
}

#[derive(Clone, Copy, Debug)]
struct ProbeShape {
    upright_radius: f64,
    leaning_radius: f64,
    apart: f64,
    lean: f64,
    seam_at: f64,
    roll: f64,
    height: f64,
}

fn probe_shared(shape: ProbeShape) -> f64 {
    let steps = 4000u32;
    let (a, b) = (shape.apart - shape.leaning_radius, shape.upright_radius);
    let (m, w) = ((a + b) / 2.0, (b - a) / 2.0);
    let step = PI / f64::from(steps);
    let sum: f64 = (0..steps)
        .map(|at| {
            let t = step * (f64::from(at) + 0.5);
            let y = m - w * t.cos();
            w * w
                * t.sin().powi(2)
                * ((shape.upright_radius + y) * (shape.apart + shape.leaning_radius - y)).sqrt()
        })
        .sum();
    4.0 / shape.lean.cos() * sum * step
}

fn probe_one(shape: ProbeShape, rule: Rule, upright_first: bool) -> String {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let z = Axis::z();
    let upright_placement =
        moved_to([0.0, 0.0, -1.0]).compose(&Isometry::rotation(&z, shape.seam_at).expect("a turn"));
    let the_y_axis = Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("an axis");
    let (c, s) = (shape.seam_at.cos(), shape.seam_at.sin());
    let leaning_placement = moved_to([shape.apart * c, shape.apart * s, shape.height])
        .compose(&Isometry::rotation(&z, shape.seam_at - PI / 2.0).expect("a turn"))
        .compose(&Isometry::rotation(&the_y_axis, PI / 2.0 + shape.lean).expect("a turn"))
        .compose(&moved_to([0.0, 0.0, -10.0]))
        .compose(&Isometry::rotation(&z, shape.roll).expect("a turn"));
    let upright = model
        .add_round_bar(shape.upright_radius, 12.0, &upright_placement)
        .expect("a bar");
    let leaning = model
        .add_round_bar(shape.leaning_radius, 20.0, &leaning_placement)
        .expect("a bar");
    let (arguments, tools) = if upright_first {
        ([upright], [leaning])
    } else {
        ([leaning], [upright])
    };
    let shared = probe_shared(shape);
    let r1 = shape.upright_radius;
    let r2 = shape.leaning_radius;
    let expected = match rule {
        Rule::Intersection => shared,
        Rule::Union => PI * r1 * r1 * 12.0 + PI * r2 * r2 * 20.0 - shared,
        Rule::Difference if upright_first => PI * r1 * r1 * 12.0 - shared,
        Rule::Difference => PI * r2 * r2 * 20.0 - shared,
        _ => unreachable!(),
    };
    let fused = match model.fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new()) {
        Ok(fused) => fused.solids().to_vec(),
        Err(why) => return format!("REFUSED {why:?}"),
    };
    if let Err(why) = model.validate(&policy, &mut ProvenanceLog::new()) {
        return format!("INVALID {why:?}");
    }
    let [solid] = fused[..] else {
        return format!("SOLIDS {}", fused.len());
    };
    let view = model.solid(solid).expect("a solid");
    let faces = view.faces().count();
    match view.measure(
        Deflection::of(0.01).expect("a positive length"),
        &policy,
        &mut ProvenanceLog::new(),
    ) {
        Ok(measure) => {
            let off = measure.volume() - expected;
            let verdict = if off.abs() <= measure.volume_error() {
                "ok"
            } else {
                "WRONG"
            };
            format!(
                "{verdict} faces {faces} volume {:.6} expected {expected:.6} error {:.2e}",
                measure.volume(),
                measure.volume_error()
            )
        }
        Err(why) => format!("UNMEASURED faces {faces} {why:?}"),
    }
}

#[test]
fn probe_q2590_sweep() {
    let mut shapes = Vec::new();
    for &lean in &[0.1, 0.2, 0.3, 0.45, 0.6] {
        for &seam_at in &[PI / 2.0, 0.0, 0.3, 1.0, 2.0, 2.9, 4.0, 5.5] {
            for &roll in &[0.0, 1.3] {
                shapes.push(ProbeShape {
                    upright_radius: 2.0,
                    leaning_radius: 3.0,
                    apart: 4.0,
                    lean,
                    seam_at,
                    roll,
                    height: 5.0,
                });
            }
        }
    }
    for &(r1, r2, apart) in &[(2.0, 2.5, 3.5), (1.5, 3.0, 3.75), (2.0, 3.0, 4.5)] {
        for &seam_at in &[PI / 2.0, 1.0, 4.0] {
            shapes.push(ProbeShape {
                upright_radius: r1,
                leaning_radius: r2,
                apart,
                lean: 0.3,
                seam_at,
                roll: 0.0,
                height: 5.0,
            });
        }
    }
    let mut bad = 0;
    for shape in shapes {
        for rule in [Rule::Intersection, Rule::Union, Rule::Difference] {
            for upright_first in [true, false] {
                let said = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    probe_one(shape, rule, upright_first)
                }))
                .unwrap_or_else(|_| "PANICKED".to_owned());
                if !said.starts_with("ok") {
                    bad += 1;
                }
                println!("{shape:?} {rule:?} upright_first {upright_first}: {said}");
            }
        }
    }
    println!("bad: {bad}");
}

fn probe_gradient(surface: &rcad_geom::Surface, at: Point) -> Vector {
    match surface {
        rcad_geom::Surface::Cylinder(pipe) => {
            let (origin, along) = (pipe.axis().origin(), pipe.axis().direction());
            let out = at - origin;
            out * along.dot(along) - along * along.dot(out)
        }
        other => panic!("{other:?}"),
    }
}

fn probe_inspect(shape: ProbeShape, rule: Rule, upright_first: bool) {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let z = Axis::z();
    let upright_placement =
        moved_to([0.0, 0.0, -1.0]).compose(&Isometry::rotation(&z, shape.seam_at).expect("a turn"));
    let the_y_axis = Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("an axis");
    let (c, s) = (shape.seam_at.cos(), shape.seam_at.sin());
    let leaning_placement = moved_to([shape.apart * c, shape.apart * s, shape.height])
        .compose(&Isometry::rotation(&z, shape.seam_at - PI / 2.0).expect("a turn"))
        .compose(&Isometry::rotation(&the_y_axis, PI / 2.0 + shape.lean).expect("a turn"))
        .compose(&moved_to([0.0, 0.0, -10.0]))
        .compose(&Isometry::rotation(&z, shape.roll).expect("a turn"));
    let upright = model
        .add_round_bar(shape.upright_radius, 12.0, &upright_placement)
        .expect("a bar");
    let leaning = model
        .add_round_bar(shape.leaning_radius, 20.0, &leaning_placement)
        .expect("a bar");
    let (arguments, tools) = if upright_first {
        ([upright], [leaning])
    } else {
        ([leaning], [upright])
    };
    let solids = match model.fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new()) {
        Ok(fused) => fused.solids().to_vec(),
        Err(why) => {
            println!("{shape:?} {rule:?} {upright_first}: refused {why:?}");
            return;
        }
    };
    let view = model.solid(solids[0]).expect("a solid");
    let mut seen = Vec::new();
    for face in view.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if !edge.is_closed().expect("closed") || seen.contains(&edge.key()) {
                    continue;
                }
                seen.push(edge.key());
                let at = edge.start().point();
                let surfaces: Vec<rcad_geom::Surface> = edge
                    .neighbouring_faces()
                    .map(|key| model.face(key).expect("a face").surface())
                    .collect();
                let pipes: Vec<rcad_geom::Surface> = surfaces
                    .iter()
                    .copied()
                    .filter(|s| matches!(s, rcad_geom::Surface::Cylinder(_)))
                    .collect();
                if pipes.len() != 2 {
                    continue;
                }
                let is_upright = |s: &rcad_geom::Surface| match s {
                    rcad_geom::Surface::Cylinder(pipe) => {
                        pipe.axis().direction().components()[2].abs() > 0.999
                    }
                    _ => false,
                };
                let (up, lean) = if is_upright(&pipes[0]) {
                    (pipes[0], pipes[1])
                } else {
                    (pipes[1], pipes[0])
                };
                let origin_x = |s: &rcad_geom::Surface| match s {
                    rcad_geom::Surface::Cylinder(pipe) => pipe.axis().origin().coordinates(),
                    _ => unreachable!(),
                };
                let lean_axis = match lean {
                    rcad_geom::Surface::Cylinder(pipe) => pipe.axis().direction(),
                    _ => unreachable!(),
                };
                let (g_up, g_lean) = (probe_gradient(&up, at), probe_gradient(&lean, at));
                let walked = g_up.cross(g_lean);
                let longitude = lean_axis.cross(g_lean);
                let along = walked.dot(longitude);
                let latitude = walked.dot(g_lean.cross(longitude));
                let rule_says =
                    rcad_geom::which_way_a_closed_crossing_turns([g_up, g_lean], g_lean, lean_axis);
                let p = at.coordinates();
                let off_seam = p[1].atan2(p[0]) - shape.seam_at;
                println!(
                    "{shape:?} {rule:?} {upright_first}: corner {p:?} off seam {off_seam:.3e} up origin {:?} lean origin {:?} lean longitude f64 {along:.3e} latitude {latitude:.3e} rule {rule_says:?}",
                    origin_x(&up),
                    origin_x(&lean)
                );
            }
        }
    }
}

#[test]
fn probe_q2590_inspect() {
    for &lean in &[0.1, 0.3, 0.6] {
        for &seam_at in &[PI / 2.0, 1.0, 2.0, 2.9, 4.0] {
            for &roll in &[0.0, 1.3] {
                let shape = ProbeShape {
                    upright_radius: 2.0,
                    leaning_radius: 3.0,
                    apart: 4.0,
                    lean,
                    seam_at,
                    roll,
                    height: 5.0,
                };
                for upright_first in [true, false] {
                    probe_inspect(shape, Rule::Intersection, upright_first);
                }
            }
        }
    }
}
