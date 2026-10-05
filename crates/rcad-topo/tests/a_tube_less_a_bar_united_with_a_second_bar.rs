//! A tube with a bar standing across its wall, fused every way.
//!
//! The tube is a bar of radius 3 and height 4 less a bar of radius 1 standing
//! through it from `z = −1` to `5`. The second bar has radius 1.5 and stands at
//! `(2, 0)`, so its circle crosses the tube's outer circle at
//! `(2.6875, ±1.3332)` and its inner circle at `(0.6875, ±0.7262)`.
//!
//! **Each cap of the tube less the bar's disk is a C**, with those four corners
//! and the hole's own corner, `(0, ∓1)`. Its outer arc runs the long way round,
//! so half way from that arc's chord's middle, `(2.6875, 0)`, to where the line
//! square to the chord crosses the circle, `(−3, 0)`, is `(−0.15625, 0)`: in the
//! hole. Every other point the search offered was in the hole or in the bar's
//! disk, and all eighteen fuses below were refused `a point inside a piece of a
//! face` naming those five corners (queue rows Q603 and Q897,
//! `docs/decisions.md` §365.2). The C holds plenty of doubles, so that was a gap
//! in the search. The search now also offers rungs nearer each crossing
//! (`rungs_towards_each_crossing` in `fuse/select.rs`, §365.6).
//!
//! The volumes, by hand. The tube is `π (3² − 1²) · 4 = 32π`, the bar
//! `π · 1.5² = 2.25π` for each unit of its height, and they share, for each
//! unit of height they share, the bar's disk inside the circle of radius 3 less
//! its disk inside the circle of radius 1.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

/// Both caps of the bar flush with the tube's. The bar less the tube is two
/// solids: the part of the bar inside the hole and the part outside the tube.
#[test]
fn a_bar_flush_with_both_of_the_tubes_caps_fuses_every_way() {
    fuse_every_way(0.0, 4.0);
}

/// The bar's top flush with the tube's, its foot a unit below the tube's.
#[test]
fn a_bar_a_unit_below_the_tubes_foot_fuses_every_way() {
    fuse_every_way(-1.0, 4.0);
}

/// The bar a unit below the tube altogether.
#[test]
fn a_bar_a_unit_below_the_whole_tube_fuses_every_way() {
    fuse_every_way(-1.0, 3.0);
}

fn fuse_every_way(bottom: f64, top: f64) {
    let tube = 32.0 * PI;
    let bar = 2.25 * PI * (top - bottom);
    let shared = (top.min(4.0) - bottom.max(0.0)) * (bar_shares_with(3.0) - bar_shares_with(1.0));
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for bar_first in [false, true] {
            let what = format!("{rule:?}, bar from {bottom} to {top}, bar first: {bar_first}");
            let (volume, how_many) = match (rule, bar_first) {
                (Rule::Union, _) => (tube + bar - shared, 1),
                (Rule::Intersection, _) => (shared, 1),
                (Rule::Difference, false) => (tube - shared, 1),
                (Rule::Difference, true) => (bar - shared, if bottom == 0.0 { 2 } else { 1 }),
                (other, _) => panic!("not a rule this fuses: {other:?}"),
            };
            let (mut model, tube_key, bar_key) = a_tube_and_a_bar(bottom, top);
            let [one, other] = if bar_first {
                [bar_key, tube_key]
            } else {
                [tube_key, bar_key]
            };
            let solids = fused(&mut model, one, other, rule, &what);
            assert_holds(&model, &solids, how_many, volume, &what);
        }
    }
}

/// The area the bar's disk shares with a disk of `radius` about the tube's
/// axis: the two circular segments either side of the chord between the places
/// the two circles cross.
fn bar_shares_with(radius: f64) -> f64 {
    let (bar, apart) = (1.5_f64, 2.0_f64);
    let segment = |mine: f64, theirs: f64| {
        let half_angle = ((apart * apart + mine * mine - theirs * theirs) / (2.0 * apart * mine))
            .clamp(-1.0, 1.0)
            .acos();
        mine * mine * (half_angle - half_angle.sin() * half_angle.cos())
    };
    segment(radius, bar) + segment(bar, radius)
}

fn a_tube_and_a_bar(bottom: f64, top: f64) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let outer = model
        .add_round_bar(3.0, 4.0, &Isometry::identity())
        .expect("a bar of radius three");
    let through = model
        .add_round_bar(1.0, 6.0, &at(0.0, 0.0, -1.0))
        .expect("a bar of radius one");
    let [tube] = fused(&mut model, outer, through, Rule::Difference, "the tube")[..] else {
        panic!("the tube is one solid");
    };
    let bar = model
        .add_round_bar(1.5, top - bottom, &at(2.0, 0.0, bottom))
        .expect("a bar of radius one and a half");
    (model, tube, bar)
}

fn fused(
    model: &mut Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
    what: &str,
) -> Vec<SolidKey> {
    model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} refused: {why:?}"))
        .solids()
        .to_vec()
}

fn assert_holds(model: &Model, solids: &[SolidKey], how_many: usize, volume: f64, what: &str) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} left a model that does not validate: {why}"));
    assert_eq!(solids.len(), how_many, "{what}: how many solids");
    let (mut measured, mut error) = (0.0, 0.0);
    for &solid in solids {
        let measure = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.001).expect("a fine mesh"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} does not measure: {why}"));
        measured += measure.volume();
        error += measure.volume_error();
    }
    assert!(
        (measured - volume).abs() <= error,
        "{what} holds {volume} and this one says {measured} give or take {error}"
    );
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift")
}
