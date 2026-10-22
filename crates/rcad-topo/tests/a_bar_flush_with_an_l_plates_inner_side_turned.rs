//! A round bar standing in an L plate, its pipe touching the line of one of the
//! plate's inner sides past that side's end, both turned about `z`
//! (`docs/decisions.md` §191.5, Q2505).
//!
//! # The shape
//!
//! The L is `(0,0) (4,0) (4,2) (2,2) (2,4) (0,4)`, `z` from 0 to 1. The bar is
//! of radius `1/2`, `z` from −1 to 2.5, its axis through `(1, 1.5)` or
//! `(1.5, 1)`. Through `(1, 1.5)` its pipe touches the line `y = 2` at
//! `(1, 2)`, and the inner side on that line runs from `x = 2` to `x = 4`, so
//! the pipe meets no face of the plate's sides: it goes through the plate's
//! top and floor alone. `(1.5, 1)` is the same against `x = 2`.
//!
//! # Where it was refused
//!
//! Turned a quarter turn, the side's plane is stored with normal
//! `(1, ±2⁻⁵², 0)` and the bar's axis a rounding off `x = 1.5`. Worked out
//! on those numbers the plane is a little under a radius from the axis, so it
//! cuts the pipe in two lines about `2.2e−16` apart, still off the side's
//! face. Turned `3π/2`, every certified fuse was refused `Undecidable` on the
//! order of the two places those lines cross the plate's floor and the bar's
//! floor. Turned `−π/2`, and through `(1.5, 1)` turned `π/2`, every one was
//! refused `CannotNameWhichPieceOfACrossingThisIs`, by `intersect` on the side
//! and the pipe.

use core::f64::consts::{FRAC_PI_2, PI};

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Fused, Model, ProfileBuilder, Rule, SolidKey};

#[test]
fn every_fuse_of_the_bar_and_the_l_plate_unturned_answers_the_two_as_they_stand() {
    every_fuse_answers_the_two_as_they_stand([1.0, 1.5], 0.0);
    every_fuse_answers_the_two_as_they_stand([1.5, 1.0], 0.0);
}

#[test]
fn every_fuse_of_the_bar_and_the_l_plate_turned_three_tenths_answers_the_two_as_they_stand() {
    every_fuse_answers_the_two_as_they_stand([1.0, 1.5], 0.3);
}

#[test]
fn every_fuse_of_the_bar_and_the_l_plate_turned_three_quarter_turns_answers_the_two_as_they_stand()
{
    every_fuse_answers_the_two_as_they_stand([1.0, 1.5], 1.5 * PI);
}

#[test]
fn every_fuse_of_the_bar_and_the_l_plate_turned_back_a_quarter_turn_answers_the_two_as_they_stand()
{
    every_fuse_answers_the_two_as_they_stand([1.0, 1.5], -FRAC_PI_2);
}

#[test]
fn every_fuse_of_the_bar_by_the_other_side_turned_a_quarter_turn_answers_the_two_as_they_stand() {
    every_fuse_answers_the_two_as_they_stand([1.5, 1.0], FRAC_PI_2);
}

/// Every rule with either solid first, under a certified policy, answers the
/// two as they stand, validates, and records nothing.
///
/// **The closed forms.** The plate holds `12`. The bar is `π/4` across: it
/// shares `π/4` with the plate, and sticks out `1` below it and `1.5` above.
/// So the union holds `12 + 5π/8`, the plate less the bar `12 − π/4`, and the
/// bar less the plate is two pieces, `π/4` and `3π/8`.
fn every_fuse_answers_the_two_as_they_stand(at: [f64; 2], turn: f64) {
    let shared = PI / 4.0;
    let bar = 3.5 * shared;
    let plate = 12.0;
    let policy = Policy::certified();
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for bar_first in [true, false] {
            let case = format!("bar at {at:?} turned {turn}, {rule:?}, bar first {bar_first}");
            let (mut model, the_plate, the_bar) = the_two(at, turn);
            let (first, second) = if bar_first {
                (the_bar, the_plate)
            } else {
                (the_plate, the_bar)
            };
            let mut log = ProvenanceLog::new();
            let fused = model.fuse(&[first], &[second], rule, &policy, &mut log);
            assert!(log.records().is_empty(), "{case}: {:#?}", log.records());
            let closed_forms: &[f64] = match (rule, bar_first) {
                (Rule::Union, _) => &[plate + bar - shared],
                (Rule::Intersection, _) => &[shared],
                (Rule::Difference, true) => &[shared, 1.5 * shared],
                (Rule::Difference, false) => &[plate - shared],
                _ => unreachable!("{case}: three rules are asked"),
            };
            assert_solids_holding(&model, fused, closed_forms, shared, &policy, &case);
        }
    }
}

/// Checks that `fused` is as many solids as `closed_forms`, the smallest
/// holding the first and so on, each within the error bar `measure`
/// publishes, and that the model validates. The bar must be under half of
/// `shared`, the piece the bar and the plate share, so that an answer that
/// left that piece out or counted it twice is off by more than its bar.
fn assert_solids_holding(
    model: &Model,
    fused: rcad_topo::Result<Fused>,
    closed_forms: &[f64],
    shared: f64,
    policy: &Policy,
    case: &str,
) {
    let fused = fused.unwrap_or_else(|why| panic!("{case}: {why:?}"));
    let mut measured: Vec<_> = fused
        .solids()
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("the fuse made it")
                .measure(
                    Deflection::of(0.01).expect("a positive deflection"),
                    policy,
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{case}: {why:?}"))
        })
        .collect();
    measured.sort_by(|one, other| one.volume().total_cmp(&other.volume()));
    assert_eq!(measured.len(), closed_forms.len(), "{case}: {measured:?}");
    for (measured, closed_form) in measured.iter().zip(closed_forms) {
        assert!(
            (measured.volume() - closed_form).abs() <= measured.volume_error(),
            "{case}: {closed_form} wanted, {measured:?}"
        );
        assert!(
            measured.volume_error() < shared / 2.0,
            "{case}: {shared} shared, {measured:?}"
        );
    }
    model
        .validate(policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{case}: {why:?}"));
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
        .expect("a positive height");
    let bar = model
        .add_round_bar(
            0.5,
            3.5,
            &Isometry::translation(Vector::new(at[0], at[1], -1.0)).expect("a finite shift"),
        )
        .expect("a positive radius and height");
    let turn = Isometry::rotation(&Axis::z(), turn).expect("a finite turn");
    model.place_solid(plate, &turn).expect("the model made it");
    model.place_solid(bar, &turn).expect("the model made it");
    (model, plate, bar)
}
