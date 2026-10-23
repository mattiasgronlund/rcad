//! Two round bars of the same radius leaning across each other, fused either
//! way round (row Q2609).
//!
//! # The shape
//!
//! The upright bar is radius 2, up `z` from −1 to 11, turned a quarter round
//! `z` so that its seam runs up `(0, 2, z)`. The leaning bar is radius 2 and
//! 20 long, its axis through `(0, 3.5, 5)` along `(cos 0.5, 0, −sin 0.5)`.
//!
//! # Why this shape has its own pin
//!
//! Q2581 first asked for two bars of one radius, and its pins give the two
//! bars radius 3 against radius 2, so they do not pin the one-radius case.
//! With `march.rs` from before Q2581 (`ee1826eb^1`), the upright bar first was
//! refused `CannotFollowACrossingWhereItsSurfacesAreTangent { at:
//! BoundingHowFarOffTheCrossingAPointIs }` by the intersection, the union and
//! the difference alike. Since Q2581 all three answer, in both orders.
//!
//! The fuses run certified, in a model of 100 mm.

use std::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule};

/// How far the leaning bar's axis leans down from `x`, in radians.
const LEAN: f64 = 0.5;

/// The two bars share the same piece whichever is handed in first, and it
/// measures what they share.
#[test]
fn the_piece_two_bars_of_one_radius_share_is_the_same_either_way_round() {
    let shared = what_the_two_bars_share(1000);
    let upright_first = answer(Rule::Intersection, true, shared, 2);
    let leaning_first = answer(Rule::Intersection, false, shared, 2);
    the_same_volume(upright_first, leaning_first, "the intersection");
}

/// The two bars unite to the same solid whichever is handed in first.
#[test]
fn two_bars_of_one_radius_unite_to_the_same_solid_either_way_round() {
    let whole = UPRIGHT + LEANING - what_the_two_bars_share(1000);
    let upright_first = answer(Rule::Union, true, whole, 8);
    let leaning_first = answer(Rule::Union, false, whole, 8);
    the_same_volume(upright_first, leaning_first, "the union");
}

/// Each bar less the other answers, and measures its own volume less what the
/// two share. Unlike the radius-3-against-2 pair, the two differences do not
/// come back with the same face count: the upright bar less the leaning one
/// keeps 4 faces, the leaning bar less the upright one keeps 6.
#[test]
fn each_bar_of_one_radius_less_the_other_answers() {
    let shared = what_the_two_bars_share(1000);
    answer(Rule::Difference, true, UPRIGHT - shared, 4);
    answer(Rule::Difference, false, LEANING - shared, 6);
}

/// The upright bar's volume, `π · 2² · 12`.
const UPRIGHT: f64 = 48.0 * PI;

/// The leaning bar's volume, `π · 2² · 20`.
const LEANING: f64 = 80.0 * PI;

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
        .add_round_bar(2.0, 20.0, &leaning_across())
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

/// The leaning bar, its axis through `(0, 3.5, 5)` along `(cos 0.5, 0, −sin
/// 0.5)`, reaching 10 each way from there.
fn leaning_across() -> Isometry {
    let the_y_axis = Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("an axis");
    moved_to([0.0, 3.5, 5.0])
        .compose(&Isometry::rotation(&the_y_axis, PI / 2.0 + LEAN).expect("a turn"))
        .compose(&moved_to([0.0, 0.0, -10.0]))
}

fn moved_to(at: [f64; 3]) -> Isometry {
    Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a finite move")
}

/// What the two bars share, by a midpoint rule of `steps` steps: `(1/cos 0.5)`
/// times the integral over `y` from 1.5 to 2 of `2·√(4 − y²) · 2·√(4 − (y −
/// 3.5)²)`.
fn what_the_two_bars_share(steps: u32) -> f64 {
    let step = 0.5 / f64::from(steps);
    let sum: f64 = (0..steps)
        .map(|at| {
            let y = 1.5 + step * (f64::from(at) + 0.5);
            2.0 * (4.0 - y * y).sqrt() * 2.0 * (4.0 - (y - 3.5) * (y - 3.5)).sqrt()
        })
        .sum();
    sum * step / LEAN.cos()
}
