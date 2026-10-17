//! A stepped shaft cut in two on a plane beside or through its axis, both
//! halves turned about that axis, and the halves fused again by every rule
//! (`docs/decisions.md` §191.5, row Q2228).
//!
//! **The shape.** The shaft is a bar of radius 2 from `z = 0` to `z = 1`
//! united with a bar of radius 1 from `z = 1` to `z = 2`, so `5π` in all. One
//! copy less the block `x < c` is the half cut off, and another shared with
//! that block is the half kept, so the two lie flush along the plane `x = c`.
//! Both are then turned by the same angle about the shaft's axis, `z`.
//!
//! **What it holds, by hand.** The part of a disk of radius `r` on the side
//! `x < c` of a chord is `r² acos(−c / r) + c √(r² − c²)`, so the kept half is
//! that for radius 2 and for radius 1, one unit high each, and the half cut off
//! is the rest of `5π`. The halves only touch, so they unite into one solid of
//! both volumes, share nothing, and either less the other is itself.
//!
//! **What was wrong before Q2228.** Cutting the shaft splits its bottom face
//! where the narrow bar's pipe would cross it, and the fuse joins the two
//! pieces back into one face. That face's one ring runs round an arc of radius
//! 2 and back along the chord, through four corners all on the chord's line.
//! Unturned, those corners are exactly on one line, the polygon through them
//! sweeps nothing, and the ring, the only one, was taken as the outline.
//! Turned, the turn rounds them off the line, the polygon sweeps a sliver
//! whose sign is the rounding's, and where that came out against the plane
//! the ring was taken as a hole. With no outline left, `rings_round` in
//! `fuse/assemble.rs` panicked "none of its rings winds with the plane or
//! bounds a region holding the others", under both policies. Now a ring with
//! a curved side is not placed by its corners alone: where every ring winds
//! against the plane, the outline is among those with a curved side, and the
//! one such ring is it.

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// The turns about `z` the halves are placed with: unturned, and the three
/// the row found panicking.
const TURNS: [f64; 4] = [0.0, 0.4, 1.0, 2.5];

/// The stepped shaft cut on `x = 0.5`, beside its axis and through both bars.
#[test]
fn a_stepped_shaft_cut_beside_its_axis_and_turned_fuses_with_its_other_half() {
    for turn in TURNS {
        every_fuse_of_the_halves_answers(
            &format!("cut at x = 0.5, turned {turn}"),
            0.5,
            CutOffFrom::TheSteppedShaft,
            &[Isometry::rotation(&Axis::z(), turn).expect("a finite turn")],
        );
    }
}

/// The stepped shaft cut through its axis on `x = 0`, then shifted by
/// `(7, −5, 2.5)` and turned about `z`, so it is turned about a line it no
/// longer stands on.
#[test]
fn a_stepped_shaft_cut_through_its_axis_shifted_and_turned_fuses_with_its_other_half() {
    for turn in TURNS {
        every_fuse_of_the_halves_answers(
            &format!("cut at x = 0, shifted, turned {turn}"),
            0.0,
            CutOffFrom::TheSteppedShaft,
            &[
                Isometry::translation(Vector::new(7.0, -5.0, 2.5)).expect("a finite shift"),
                Isometry::rotation(&Axis::z(), turn).expect("a finite turn"),
            ],
        );
    }
}

/// The half cut off is of a plain bar of radius 2 from `z = 0` to `z = 2`, and
/// the half kept is of the stepped shaft, both cut on `x = 0.5`.
#[test]
fn a_bars_half_and_a_stepped_shafts_half_turned_fuse_by_every_rule() {
    for turn in TURNS {
        every_fuse_of_the_halves_answers(
            &format!("a bar's half against the shaft's, turned {turn}"),
            0.5,
            CutOffFrom::AWideBar,
            &[Isometry::rotation(&Axis::z(), turn).expect("a finite turn")],
        );
    }
}

/// What the half cut off is cut from.
#[derive(Clone, Copy, Debug)]
enum CutOffFrom {
    TheSteppedShaft,
    /// A bar of radius 2 from `z = 0` to `z = 2`.
    AWideBar,
}

impl CutOffFrom {
    fn build(self, model: &mut Model) -> SolidKey {
        match self {
            CutOffFrom::TheSteppedShaft => a_stepped_shaft(model),
            CutOffFrom::AWideBar => a_bar(model, 2.0, 2.0, 0.0),
        }
    }

    /// The volume of the half cut off, on `x > cut`, by hand.
    fn volume_right_of(self, cut: f64) -> f64 {
        match self {
            CutOffFrom::TheSteppedShaft => 5.0 * PI - a_stepped_shafts_part_left_of(cut),
            CutOffFrom::AWideBar => 2.0 * (4.0 * PI - a_bars_part_left_of(2.0, cut)),
        }
    }
}

/// The half of `cut_off_from` on `x > cut` and the stepped shaft's half on
/// `x < cut`, each placed by every movement in `moves` in turn, fused by every
/// rule either way round under both policies: one solid of the volume worked
/// out by hand, or nothing where the two only touch, and a certified fuse logs
/// nothing.
fn every_fuse_of_the_halves_answers(
    route: &str,
    cut: f64,
    cut_off_from: CutOffFrom,
    moves: &[Isometry],
) {
    let kept = a_stepped_shafts_part_left_of(cut);
    let cut_off = cut_off_from.volume_right_of(cut);
    for (policy_name, policy) in [
        ("certified", Policy::certified()),
        ("tolerant", Policy::tolerant(100.0)),
    ] {
        for (rule, kept_first, volume) in [
            (Rule::Union, true, Some(kept + cut_off)),
            (Rule::Union, false, Some(kept + cut_off)),
            (Rule::Intersection, true, None),
            (Rule::Intersection, false, None),
            (Rule::Difference, true, Some(kept)),
            (Rule::Difference, false, Some(cut_off)),
        ] {
            let asked =
                format!("{route}, {rule:?}, the kept half first {kept_first}, {policy_name}");
            let mut model = a_model();
            let halves = the_two_halves(&mut model, cut, cut_off_from);
            for movement in moves {
                for half in halves {
                    model
                        .place_solid(half, movement)
                        .expect("this model made the key");
                }
            }
            let [kept_half, cut_off_half] = halves;
            let [one, other] = if kept_first {
                [kept_half, cut_off_half]
            } else {
                [cut_off_half, kept_half]
            };
            let mut log = ProvenanceLog::new();
            let fused = model
                .fuse(&[one], &[other], rule, &policy, &mut log)
                .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
            if policy_name == "certified" {
                assert!(
                    log.is_empty(),
                    "{asked}: a certified fuse logs nothing: {log:?}"
                );
            }
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|broken| panic!("{asked}: {broken:?}"));
            match volume {
                Some(volume) => {
                    let [solid] = fused.solids() else {
                        panic!("{asked}: one solid, not {:?}", fused.solids());
                    };
                    holds(&model, *solid, volume, &asked);
                }
                None => assert!(
                    fused.solids().is_empty(),
                    "{asked}: nothing, not {:?}",
                    fused.solids()
                ),
            }
        }
    }
}

/// The stepped shaft's volume on `x < cut`.
fn a_stepped_shafts_part_left_of(cut: f64) -> f64 {
    a_bars_part_left_of(2.0, cut) + a_bars_part_left_of(1.0, cut)
}

/// The area of a disk of `radius` about the origin on the side `x < cut`, the
/// volume of that part of a bar one unit high.
fn a_bars_part_left_of(radius: f64, cut: f64) -> f64 {
    radius * radius * (-cut / radius).acos() + cut * (radius * radius - cut * cut).sqrt()
}

/// The stepped shaft's part on `x < cut`, then `cut_off_from`'s on `x > cut`.
fn the_two_halves(model: &mut Model, cut: f64, cut_off_from: CutOffFrom) -> [SolidKey; 2] {
    let below = [[-10.0, cut], [-10.0, 10.0], [-10.0, 10.0]];
    let [one, other] = [cut_off_from.build(model), a_stepped_shaft(model)];
    let [block, its_copy] = [a_box(model, below), a_box(model, below)];
    [
        the_one_solid(model, other, its_copy, Rule::Intersection),
        the_one_solid(model, one, block, Rule::Difference),
    ]
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a working size"))
}

/// A bar of radius 2 from `z = 0` to `z = 1` united with one of radius 1 from
/// `z = 1` to `z = 2`.
fn a_stepped_shaft(model: &mut Model) -> SolidKey {
    let wide = a_bar(model, 2.0, 1.0, 0.0);
    let narrow = a_bar(model, 1.0, 1.0, 1.0);
    the_one_solid(model, wide, narrow, Rule::Union)
}

fn a_bar(model: &mut Model, radius: f64, height: f64, bottom: f64) -> SolidKey {
    model
        .add_round_bar(
            radius,
            height,
            &Isometry::translation(Vector::new(0.0, 0.0, bottom)).expect("a finite shift"),
        )
        .expect("a bar of positive size")
}

fn a_box(model: &mut Model, [x, y, z]: [[f64; 2]; 3]) -> SolidKey {
    let outline = ProfileBuilder::new(vec![[x[0], y[0]], [x[1], y[0]], [x[1], y[1]], [x[0], y[1]]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("an outline");
    model
        .add_extrusion(
            &outline,
            z[1] - z[0],
            &Isometry::translation(Vector::new(0.0, 0.0, z[0])).expect("a finite shift"),
        )
        .expect("a positive height")
}

fn the_one_solid(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let fused = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{rule:?}: {refused:?}"));
    let [solid] = fused.solids() else {
        panic!("{rule:?} is one solid: {:?}", fused.solids());
    };
    *solid
}

fn holds(model: &Model, solid: SolidKey, volume: f64, asked: &str) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{asked}: {} ± {} against {volume}",
        measured.volume(),
        measured.volume_error()
    );
}
