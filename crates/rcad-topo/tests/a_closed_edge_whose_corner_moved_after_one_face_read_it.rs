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
    let upright_first = answer(Rule::Intersection, true, shared);
    let leaning_first = answer(Rule::Intersection, false, shared);
    the_same_volume(upright_first, leaning_first, "the intersection");
}

/// The two bars unite to the same solid whichever is handed in first.
#[test]
fn two_bars_unite_to_the_same_solid_either_way_round() {
    let whole = UPRIGHT + LEANING - what_the_two_bars_share(1000);
    let upright_first = answer(Rule::Union, true, whole);
    let leaning_first = answer(Rule::Union, false, whole);
    the_same_volume(upright_first, leaning_first, "the union");
}

/// Each bar less the other answers, and measures its own volume less what
/// the two share.
#[test]
fn each_bar_less_the_other_answers() {
    let shared = what_the_two_bars_share(1000);
    answer(Rule::Difference, true, UPRIGHT - shared);
    answer(Rule::Difference, false, LEANING - shared);
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
/// it answers one solid, that the model validates, and that the solid measures
/// `volume` to within its mesh's own error. Hands back what it measured.
fn answer(rule: Rule, upright_first: bool, volume: f64) -> Measured {
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
