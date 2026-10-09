//! A half bar turned about the axis of a half bend, fused with it every way.
//!
//! **The half bar** is a round bar of radius six about `z`, from `z = −2` to
//! `z = 2`, cut back to `x ≥ 0` by taking a block away, and then turned about
//! `z`. Its two caps are half disks, each bounded by one straight side on the
//! flat face through the axis and one arc of the bar's wall: half a circle,
//! whose chord runs through the circle's centre.
//!
//! **The half bend** is the tube of radius one round the ring of radius four
//! about `z`, from `+x` round through `+y` to `−x`
//! (`rings::a_bend_turned_and_written_out(model, 4, 1, 2, 0)`). The bar's wall,
//! six from the axis, and its caps, two above and below the ring's plane, are
//! clear of the tube, which reaches five from the axis and one off its plane, so
//! the bar's flat face through the axis is the only face of the bar that meets
//! the bend.
//!
//! # What each fuse is held to
//!
//! By Pappus, a disk of area `π` swept round a ring of radius four holds `4π` a
//! radian. The half bend is `π` radians of it, `4π²`, and the half bar holds
//! `36π · 4 / 2 = 72π`. The half bar keeps the side of its flat face that
//! `(cos θ, sin θ)` points to once turned by `θ`, a turn right-handed about `z`
//! taking `+x` towards `+y`: the ring from `θ − π/2` to `θ + π/2` round the
//! axis. The bend runs from `0` to `π`, so for `0 ≤ θ < π/2` the two share the
//! ring from `0` to `θ + π/2`, which holds `4π(θ + π/2)`. Unturned that is
//! `2π²`.
//!
//! The union is the two less what they share, the intersection is what they
//! share, and each difference is its first argument less what they share. Each
//! answer is one solid: the bend's cap at `+x` is inside the bar, and it leaves
//! the bar through the bar's flat face, so the bar less the bend is the bar
//! with one blind round hole in it, and the bend less the bar is one piece of
//! tube.
//!
//! Every volume is held to the error its measurement gives, since a ring's wall
//! is not a chord-for-chord wall.
//!
//! # Why a turned half bar was refused, and what answers it
//!
//! Unturned, every fuse here answered already. Turned, every one was refused
//! `AFaceTrimmedByACurveThisCannotFollow` on the bar's cap until Q1496: the
//! fuse's arrangement could not say which half of the cap's circle the cap's
//! arc is (`which_way_an_edge_runs_round` in `fuse/arrange.rs`). The arc is
//! picked by the plane of the bar's flat face, and that plane runs through the
//! circle's centre and through the boxes the fuse solves both ends of the arc
//! into. Each of the first four ways `the_way_into_an_arc` in `holds.rs` reads
//! an arc needs an end clearly on or in front of that plane, or the centre
//! clearly off the chord or clearly behind the plane. The fifth asks where the
//! line through the centre, square to the chord, meets the circle instead: one
//! of the two points is clearly behind the plane, so the arc is the half on the
//! other side (§330.3).

use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

mod rings;

/// How fine a mesh every volume here is read off.
const DEFLECTION: f64 = 0.01;

/// What the half bar holds: half of a bar of radius six and length four.
const THE_HALF_BAR: f64 = 72.0 * PI;

/// What the half bend holds: `π` radians of a disk of area `π` swept round a
/// ring of radius four.
const THE_HALF_BEND: f64 = 4.0 * PI * PI;

/// The two turns of Q555 that put the bar's face `x = 0` on `x + y = 0`
/// spelled with `c = 0.707 106 781 186 547 5`.
const TO_THE_SHORTER_SPELLING: [f64; 2] = [0.29, 0.495_398_163_397_448_4];

/// The two turns of Q555 that put it there spelled with
/// `c′ = 0.707 106 781 186 547 6`.
const TO_THE_LONGER_SPELLING: [f64; 2] = [0.45, 0.335_398_163_397_448_05];

#[test]
fn unturned_every_fuse_answers() {
    every_fuse_answers(&[]);
}

#[test]
fn turned_by_three_tenths_every_fuse_answers() {
    every_fuse_answers(&[0.3]);
}

#[test]
fn turned_by_a_quarter_of_pi_every_fuse_answers() {
    every_fuse_answers(&[FRAC_PI_4]);
}

#[test]
fn turned_by_0_7853_every_fuse_answers() {
    every_fuse_answers(&[0.7853]);
}

#[test]
fn turned_by_one_radian_every_fuse_answers() {
    every_fuse_answers(&[1.0]);
}

#[test]
fn turned_in_two_parts_onto_x_plus_y_equals_nought_every_fuse_answers() {
    every_fuse_answers(&TO_THE_SHORTER_SPELLING);
    every_fuse_answers(&TO_THE_LONGER_SPELLING);
}

/// Fuse the half bar turned by each of `turns` in order with the half bend,
/// under every rule and whichever is named first, and hold each answer to its
/// closed form.
fn every_fuse_answers(turns: &[f64]) {
    let angle: f64 = turns.iter().sum();
    assert!(
        (0.0..FRAC_PI_2).contains(&angle),
        "the closed forms here are for a turn from nought to a quarter"
    );
    let shared = 4.0 * PI * (angle + FRAC_PI_2);

    let mut model = a_model();
    let bar = a_half_bar(&mut model, turns);
    keeps_the_side_turned_to(&model, bar, angle);

    for bar_first in [true, false] {
        let what = format!("turned by {turns:?}, bar first {bar_first}");
        let fuse = |rule| {
            let mut model = a_model();
            let bend = rings::a_bend_turned_and_written_out(&mut model, 4.0, 1.0, 2, 0)
                .expect("a half bend described this way is a solid");
            let bar = a_half_bar(&mut model, turns);
            let answer = fused(&mut model, bar_first, bar, bend, rule, &what);
            assert_eq!(answer.len(), 1, "{rule:?}, {what}: one solid");
            volume_and_error_of(&model, &answer)
        };

        within_its_error(
            fuse(Rule::Union),
            THE_HALF_BAR + THE_HALF_BEND - shared,
            &format!("union, {what}"),
        );
        within_its_error(
            fuse(Rule::Intersection),
            shared,
            &format!("intersection, {what}"),
        );
        let first = if bar_first {
            THE_HALF_BAR
        } else {
            THE_HALF_BEND
        };
        within_its_error(
            fuse(Rule::Difference),
            first - shared,
            &format!("difference, {what}"),
        );
    }
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(50.0).expect("a positive size"))
}

/// A bar of radius six about `z` from `z = −2` to `z = 2`, cut back to `x ≥ 0`
/// by taking a block away, and then turned about `z` by each of `turns`.
fn a_half_bar(model: &mut Model, turns: &[f64]) -> SolidKey {
    let bar = model
        .add_round_bar(
            6.0,
            4.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -2.0)).expect("a finite placement"),
        )
        .expect("a bar of radius six");
    let x_below_nought = ProfileBuilder::new(vec![
        [-20.0, -20.0],
        [0.0, -20.0],
        [0.0, 20.0],
        [-20.0, 20.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    let cut = model
        .add_extrusion(
            &x_below_nought,
            40.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -20.0)).expect("a finite placement"),
        )
        .expect("building a box needs no decision");
    let answer = model
        .fuse(
            &[bar],
            &[cut],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar less a block cut square about its axis is answered");
    let &[half] = answer.solids() else {
        panic!("half a bar is one solid, not {:?}", answer.solids());
    };
    for &angle in turns {
        model
            .place_solid(
                half,
                &Isometry::rotation(&Axis::z(), angle).expect("a finite turn"),
            )
            .expect("this model made the key");
    }
    half
}

/// That the half bar's flat face through the axis faces `−(cos θ, sin θ, 0)`
/// once turned by `angle`, so the bar keeps the side that `(cos θ, sin θ)`
/// points to: the turn the closed forms are worked out for.
fn keeps_the_side_turned_to(model: &Model, bar: SolidKey, angle: f64) {
    let normals: Vec<[f64; 3]> = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .filter_map(|face| face.surface().as_plane().copied())
        .filter(|plane| plane.offset() == 0.0)
        .map(|plane| plane.normal().components())
        .collect();
    let [[x, y, z]] = normals.as_slice() else {
        panic!("a half bar has one flat face through its axis, not {normals:?}");
    };
    let length = x.hypot(*y);
    assert!(
        *z == 0.0
            && (x / length + angle.cos()).abs() < 1e-12
            && (y / length + angle.sin()).abs() < 1e-12,
        "the flat face of a half bar turned by {angle} faces {normals:?}"
    );
}

/// Fuse `bar` with `bend`, `bar` first when `bar_first`, under a certified
/// policy, and insist the model it leaves is valid.
fn fused(
    model: &mut Model,
    bar_first: bool,
    bar: SolidKey,
    bend: SolidKey,
    rule: Rule,
    what: &str,
) -> Vec<SolidKey> {
    let (arguments, tools) = if bar_first {
        ([bar], [bend])
    } else {
        ([bend], [bar])
    };
    let answer = model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{rule:?}, {what}: refused {why:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse writes a sound model or writes nothing");
    answer
}

/// What `solids` hold together at [`DEFLECTION`], and how far off their
/// measurements say that can be.
fn volume_and_error_of(model: &Model, solids: &[SolidKey]) -> (f64, f64) {
    solids
        .iter()
        .map(|&solid| {
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(DEFLECTION).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("the answer measures");
            (measured.volume(), measured.volume_error())
        })
        .fold((0.0, 0.0), |(volume, error), (one, its)| {
            (volume + one, error + its)
        })
}

/// Insist a volume is off its closed form by no more than the error its
/// measurement gives.
fn within_its_error((found, error): (f64, f64), closed_form: f64, what: &str) {
    assert!(
        (found - closed_form).abs() <= error,
        "{what}: measured {found} give or take {error}, closed form {closed_form}"
    );
}
