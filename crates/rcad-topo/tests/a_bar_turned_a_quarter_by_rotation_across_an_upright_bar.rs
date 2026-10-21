//! A round bar of radius 3 lying level across a round bar of radius 2 whose
//! seam it presses into, the level bar handed in first (rows Q2572, Q2573 and
//! Q2581).
//!
//! # The shape
//!
//! The upright bar is radius 2, up `z` from −1 to 11, turned a quarter round
//! `z` so that its seam runs up `(0, 2, z)`. The level bar is radius 3 and 20
//! long, turned onto `x` by `Isometry::rotation` about `y` by `π/2`, which
//! writes its axis as `(1, 0, 6.1e-17)`, and moved so that the axis runs
//! through `(0, y, 5)` for `y` of 4 or 4.5. Their axes pass `y` apart, inside
//! the 5 their radii add to, so the level bar presses into the upright one and
//! the two pipes cross in one closed loop. That loop crosses the upright bar's
//! seam at `x = 0`, and there it runs along the level pipe's ruling and round
//! the upright pipe at one height.
//!
//! The fuses run certified, in a model of 100 mm.

use std::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule};

/// The level bar through `y = 4.5` shares the same piece with the upright bar
/// whichever is handed in first, and it measures what they share.
#[test]
fn the_piece_two_bars_share_four_and_a_half_apart_is_the_same_either_way_round() {
    let shared = what_the_two_bars_share(4.5, 1000);
    let level_first = answer(Rule::Intersection, 4.5, true, shared, 2);
    let upright_first = answer(Rule::Intersection, 4.5, false, shared, 2);
    the_same_volume(level_first, upright_first, "the intersection at y 4.5");
}

/// The level bar through `y = 4` shares the same piece with the upright bar
/// whichever is handed in first, and it measures what they share.
#[test]
fn the_piece_two_bars_share_four_apart_is_the_same_either_way_round() {
    let shared = what_the_two_bars_share(4.0, 1000);
    let level_first = answer(Rule::Intersection, 4.0, true, shared, 2);
    let upright_first = answer(Rule::Intersection, 4.0, false, shared, 2);
    the_same_volume(level_first, upright_first, "the intersection at y 4");
}

/// The level bar through `y = 4` less the upright bar answers: its pipe with
/// a hole, its two caps, and the dent. It measures the level bar less what the
/// two share, and less what the upright bar first measured them to share.
#[test]
fn the_level_bar_four_apart_less_the_upright_bar_answers() {
    let shared = what_the_two_bars_share(4.0, 1000);
    let less = answer(Rule::Difference, 4.0, true, LEVEL - shared, 4);
    let upright_first = answer(Rule::Intersection, 4.0, false, shared, 2);
    the_same_volume(
        less,
        Measured {
            volume: LEVEL - upright_first.volume,
            error: upright_first.error,
        },
        "the difference at y 4",
    );
}

/// The level bar's volume, `π · 3² · 20`.
const LEVEL: f64 = 180.0 * PI;

/// What a measured volume is, and how far its mesh may miss it.
#[derive(Clone, Copy, Debug)]
struct Measured {
    volume: f64,
    error: f64,
}

/// The fuse `rule` of the upright bar and the level bar through `(0, y, 5)`,
/// the level one first or not: assert that it answers one solid of `faces`
/// faces, that the model validates, and that the solid measures `volume` to
/// within its mesh's own error. Hands back what it measured.
fn answer(rule: Rule, y: f64, level_first: bool, volume: f64, faces: usize) -> Measured {
    let what = format!("{rule:?} at y {y}, level bar first: {level_first}");
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let upright = model
        .add_round_bar(2.0, 12.0, &seamed_up_y())
        .expect("a bar");
    let level = model
        .add_round_bar(3.0, 20.0, &level_across(y))
        .expect("a bar");
    let (arguments, tools) = if level_first {
        ([level], [upright])
    } else {
        ([upright], [level])
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

/// Assert that two answers measure the same, to within the two meshes'
/// errors added.
fn the_same_volume(one: Measured, other: Measured, what: &str) {
    assert!(
        (one.volume - other.volume).abs() <= one.error + other.error,
        "{what}: one way {one:?}, the other {other:?}"
    );
}

/// The upright bar, up `z` from −1 to 11, turned a quarter round `z` so that
/// its seam runs up `(0, 2, z)`.
fn seamed_up_y() -> Isometry {
    moved_to([0.0, 0.0, -1.0])
        .compose(&Isometry::quarter_turns(&Axis::z(), 1).expect("a turn about z"))
}

/// The level bar, turned onto `x` by a rotation about `y` by `π/2`, its axis
/// through `(0, y, 5)`, reaching 10 each way from there.
fn level_across(y: f64) -> Isometry {
    let the_y_axis = Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("an axis");
    moved_to([0.0, y, 5.0])
        .compose(&Isometry::rotation(&the_y_axis, PI / 2.0).expect("a turn"))
        .compose(&moved_to([0.0, 0.0, -10.0]))
}

fn moved_to(at: [f64; 3]) -> Isometry {
    Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a finite move")
}

/// What the upright bar shares with the level bar through `(0, y, 5)`, by a
/// midpoint rule of `steps` steps.
///
/// At height `t` across `y`, from `y − 3` to 2, the upright bar is
/// `2√(4 − t²)` wide across `x` and the level bar is `2√(9 − (t − y)²)` tall
/// up `z`, well inside both bars' ends. Write `a` for `y − 3`: the product is
/// `4 √((2 − t)(t − a)) √((2 + t)(3 + y − t))`. Put `t = m − h cos(s)` with
/// `m` and `h` the middle and half the length of `a` to 2: the first root is
/// `h sin(s)` and `dt` is `h sin(s) ds`, which leaves a smooth integrand over
/// `s` from 0 to π.
fn what_the_two_bars_share(y: f64, steps: u32) -> f64 {
    let a = y - 3.0;
    let middle = (a + 2.0) / 2.0;
    let half = (2.0 - a) / 2.0;
    let step = PI / f64::from(steps);
    let sum: f64 = (0..steps)
        .map(|at| {
            let s = step * (f64::from(at) + 0.5);
            let t = middle - half * s.cos();
            (half * s.sin()).powi(2) * ((2.0 + t) * (3.0 + y - t)).sqrt()
        })
        .sum();
    4.0 * sum * step
}
