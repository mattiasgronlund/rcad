//! A round bar standing in an L plate, its pipe touching the line of one of the
//! plate's inner sides past that side's end, both turned about `z`
//! (`docs/decisions.md` §191.5, Q2505, Q2629, Q2654).
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
//! # A quarter turn off an axis
//!
//! Turned a quarter turn, the side's plane is stored with a normal a rounding
//! off an axis, `±(1, 2⁻⁵², 0)` or `(2⁻⁵², 1, 0)`. Worked out on the stored
//! numbers it is a little under a radius from the bar's axis, so it cuts the
//! pipe in two lines about `2.2e−16` apart, still off the side's face.
//!
//! - **Through `(1, 1.5)` turned `3π/2` or `−π/2`**, every fuse answers the
//!   two as they stand. The side's plane cuts the circles of the plate's
//!   floor and top at two places `2⁻⁵³` or `3·2⁻⁵³` apart. The pipe is joined
//!   back with one of its two lines as its seam. Until Q2654 the other line's
//!   ends were kept as corners, so each circle came back as an arc that short
//!   and an arc the rest of the way round, and every fuse was refused
//!   `DynamicRangeTooWide`, at `2⁵⁵` and `2⁵⁵/3`. Now each circle, which
//!   leaves the seam's corner and comes back round to it on faces that hold
//!   it whole, is folded back into one closed edge on that corner
//!   (`where_curved_pieces_met_off_a_run_that_closes_up` in
//!   `fuse/assemble.rs`).
//! - **Through `(1.5, 1)` turned `π/2`** the test below asks every fuse to
//!   answer, and until Q2655 lands each is refused. The bar's seam lands on
//!   the side's line, `1.09e−47` past the side's plane, between the two lines
//!   it cuts the pipe in. With the bar first every fuse is refused
//!   `Undecidable` "side of plane", for whether the seam's corner on the
//!   bar's floor is on that plane. With the plate first it is "order along a
//!   crossing", for the order of the two meets of the side's line with the
//!   top's circle, going round from the seam. Both read a box `1.1e−13` wide
//!   round the seam, and that box holds both lines.
//!
//! Turned `−π/2`, every fuse was refused on the order of the two meets of the
//! side's plane and the bar's floor, whose normals' cross product has both
//! signs, until Q2629 read the way the line runs off the axis `rcad-geom`
//! divides by.

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
            let mut log = ProvenanceLog::new();
            let (model, fused) = the_two_fused(at, turn, rule, bar_first, &mut log);
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

/// The plate and the bar of [`the_two`], fused under `rule` and a certified
/// policy.
fn the_two_fused(
    at: [f64; 2],
    turn: f64,
    rule: Rule,
    bar_first: bool,
    log: &mut ProvenanceLog,
) -> (Model, rcad_topo::Result<Fused>) {
    let (mut model, the_plate, the_bar) = the_two(at, turn);
    let (first, second) = if bar_first {
        (the_bar, the_plate)
    } else {
        (the_plate, the_bar)
    };
    let fused = model.fuse(&[first], &[second], rule, &Policy::certified(), log);
    (model, fused)
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
