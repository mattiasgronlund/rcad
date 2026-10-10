//! The top half of a ring standing on a round plate, fused every way (row
//! Q1514).
//!
//! **The half ring** is `rings::the_top_half_of_a_ring(model, 4, 1, seam)`: the
//! tube of radius one round the ring of radius four about `z`, cut by `z = 0`
//! and keeping `z ≥ 0`. By Pappus it holds half of `2π² · 4 · 1`, so `4π²`.
//!
//! **The plate** is a round bar of radius six about `z`, from `z = −2` up to
//! its top. With its top at `z = 0` the half ring stands on it, its flat face
//! lying on the plate's top, and the plate holds `72π`. With its top at
//! `z = 1/2` it holds `90π`, and its top cuts the half ring in two whole
//! circles round the axis, of radius `4 ± √3/2`.
//!
//! What the half ring and the taller plate share is the tube's cross-section
//! between `z = 0` and `z = 1/2`, `√3/4 + π/6`, swept round the ring's circle
//! of length `8π`: `4π²/3 + 2π√3`.
//!
//! **The bar** (row Q1580) is a round bar of radius `23/8` about `z`, from
//! `z = −1` up to its top, standing in the half ring's hole and touching
//! nothing of it, fused with the half ring as one group. Each unit of its
//! height holds `529π/64`, and one unit of it is inside the low plate.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Point, Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

mod rings;

/// How fine a mesh every volume here is read off.
const DEFLECTION: f64 = 0.01;

/// Half of a ring of four round a tube of one.
const THE_HALF_RING: f64 = 4.0 * PI * PI;

/// The plate with its top at `z = 0`.
const THE_LOW_PLATE: f64 = 72.0 * PI;

/// The plate with its top at `z = 1/2`.
const THE_HIGH_PLATE: f64 = 90.0 * PI;

/// One unit of the bar's height.
const A_UNIT_OF_THE_BAR: f64 = 529.0 * PI / 64.0;

#[test]
fn standing_on_the_plate_with_a_seam_every_fuse_answers() {
    standing_on_the_plate_every_fuse_answers(true);
}

#[test]
fn standing_on_the_plate_with_no_seam_every_fuse_answers() {
    standing_on_the_plate_every_fuse_answers(false);
}

/// The half ring's flat face lies on the plate's top, so the two touch and
/// share nothing: the union is one solid of both, the intersection nothing,
/// and each difference its first argument.
fn standing_on_the_plate_every_fuse_answers(a_seam: bool) {
    every_fuse_answers(
        a_seam,
        0.0,
        None,
        [
            vec![THE_LOW_PLATE + THE_HALF_RING],
            Vec::new(),
            vec![THE_HALF_RING],
            vec![THE_LOW_PLATE],
        ],
    );
}

#[test]
fn cut_by_the_plate_with_a_seam_every_fuse_answers() {
    cut_by_the_plate_every_fuse_answers(true);
}

#[test]
fn cut_by_the_plate_with_no_seam_every_fuse_answers() {
    cut_by_the_plate_every_fuse_answers(false);
}

/// The plate's top at `z = 1/2` cuts the half ring in two circles, so every
/// answer is one solid: the union, the band of the tube below `z = 1/2`, the
/// cap of the tube above it, and the plate with that band taken out.
fn cut_by_the_plate_every_fuse_answers(a_seam: bool) {
    let shared = 4.0 * PI * PI / 3.0 + 2.0 * PI * 3f64.sqrt();
    every_fuse_answers(
        a_seam,
        0.5,
        None,
        [
            vec![THE_HIGH_PLATE + THE_HALF_RING - shared],
            vec![shared],
            vec![THE_HALF_RING - shared],
            vec![THE_HIGH_PLATE - shared],
        ],
    );
}

#[test]
fn with_a_bar_in_its_hole_standing_on_the_plate_with_a_seam_every_fuse_answers() {
    with_a_bar_in_its_hole_standing_on_the_plate_every_fuse_answers(true);
}

#[test]
fn with_a_bar_in_its_hole_standing_on_the_plate_with_no_seam_every_fuse_answers() {
    with_a_bar_in_its_hole_standing_on_the_plate_every_fuse_answers(false);
}

/// The half ring and the bar up to `z = 2`, as one group, on the low plate:
/// the half ring touches the plate and the bar goes one unit into it, so the
/// union is one solid, the intersection the bar's unit in the plate, the group
/// less the plate the half ring and the bar's two units above it, and the
/// plate less the group the plate with the bar's hole in it.
///
/// The plate's top between the bar's circle and the half ring's inner circle
/// is a band whose piece comes with the bar's circle as its first wire. Read
/// as the band's outer wire, it wound every point of the band zero times, and
/// fuses with the half ring first and with the plate first were refused 'a
/// point inside a piece of a face' at `(0, 23/8, 0)` and `(3, 0, 0)` (row
/// Q1580).
fn with_a_bar_in_its_hole_standing_on_the_plate_every_fuse_answers(a_seam: bool) {
    every_fuse_answers(
        a_seam,
        0.0,
        Some(2.0),
        [
            vec![THE_LOW_PLATE + THE_HALF_RING + 2.0 * A_UNIT_OF_THE_BAR],
            vec![A_UNIT_OF_THE_BAR],
            vec![THE_HALF_RING, 2.0 * A_UNIT_OF_THE_BAR],
            vec![THE_LOW_PLATE - A_UNIT_OF_THE_BAR],
        ],
    );
}

/// With the bar's top at `z = 1`, level with the top of the tube, the bar's
/// top plane touches the half ring along the whole circle of radius four
/// there. Every fuse of the half ring and the bar, on the low plate (row
/// Q1580) or alone (row Q1515), is refused at the one corner `(4, 0, 1)` of a
/// piece of the half ring's face on the far side of that circle from the
/// tube, which holds no point. The answers exist: on the plate they are those
/// of [`with_a_bar_in_its_hole_standing_on_the_plate_every_fuse_answers`] with
/// one unit of the bar less above the plate, and alone the two solids apart.
#[test]
fn with_a_bar_level_with_the_top_of_the_tube_every_fuse_is_refused_there() {
    for with_the_plate in [true, false] {
        for a_seam in [true, false] {
            for (rule, ring_first) in EVERY_RULE_BOTH_WAYS {
                let what = format!(
                    "{rule:?}, seam {a_seam}, plate {with_the_plate}, half ring first {ring_first}"
                );
                let mut model = Model::new(Scale::millimetres(50.0).expect("a positive size"));
                let ring = rings::the_top_half_of_a_ring(&mut model, 4.0, 1.0, a_seam)
                    .expect("the top half of a ring described this way is a solid");
                let bar = a_bar_in_the_hole(&mut model, 1.0);
                let (group, other) = if with_the_plate {
                    (vec![ring, bar], vec![a_plate(&mut model, 0.0)])
                } else {
                    (vec![ring], vec![bar])
                };
                let (arguments, tools) = if ring_first {
                    (group, other)
                } else {
                    (other, group)
                };
                let fused = model.fuse(
                    &arguments,
                    &tools,
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                let Err(Error::Undecidable { records }) = fused else {
                    panic!("{what}: {fused:?}");
                };
                let last = records
                    .records()
                    .last()
                    .expect("a refusal names its decision");
                assert_eq!(
                    (last.predicate(), last.inputs()),
                    (
                        "a point inside a piece of a face",
                        &[Point::Exact([4.0, 0.0, 1.0])][..]
                    ),
                    "{what}"
                );
            }
        }
    }
}

/// Every rule, with the group as the argument and as the tool.
const EVERY_RULE_BOTH_WAYS: [(Rule, bool); 6] = [
    (Rule::Union, true),
    (Rule::Union, false),
    (Rule::Intersection, true),
    (Rule::Intersection, false),
    (Rule::Difference, true),
    (Rule::Difference, false),
];

/// Every rule, with the half ring as the argument and as the tool, on a plate
/// whose top is at `top`, and with the bar beside the half ring in one group
/// where `bar` gives its top. `truths` are the union, the intersection, the
/// group less the plate and the plate less the group.
fn every_fuse_answers(a_seam: bool, top: f64, bar: Option<f64>, truths: [Vec<f64>; 4]) {
    let [union, intersection, ring_less_plate, plate_less_ring] = truths;
    for (rule, ring_first, truths) in [
        (Rule::Union, true, &union),
        (Rule::Union, false, &union),
        (Rule::Intersection, true, &intersection),
        (Rule::Intersection, false, &intersection),
        (Rule::Difference, true, &ring_less_plate),
        (Rule::Difference, false, &plate_less_ring),
    ] {
        let what = format!(
            "{rule:?}, seam {a_seam}, top {top}, bar {bar:?}, half ring first {ring_first}"
        );
        let mut model = Model::new(Scale::millimetres(50.0).expect("a positive size"));
        let mut group = vec![
            rings::the_top_half_of_a_ring(&mut model, 4.0, 1.0, a_seam)
                .expect("the top half of a ring described this way is a solid"),
        ];
        group.extend(bar.map(|bar_top| a_bar_in_the_hole(&mut model, bar_top)));
        let plate = vec![a_plate(&mut model, top)];
        let (arguments, tools) = if ring_first {
            (group, plate)
        } else {
            (plate, group)
        };
        let answer = model
            .fuse(
                &arguments,
                &tools,
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: refused {why:?}"))
            .solids()
            .to_vec();
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        the_solids_measure(&model, &answer, truths, &what);
    }
}

/// A round bar of radius `23/8` about `z`, from `z = −1` up to `top`.
fn a_bar_in_the_hole(model: &mut Model, top: f64) -> SolidKey {
    model
        .add_round_bar(
            23.0 / 8.0,
            top + 1.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -1.0)).expect("a finite shift"),
        )
        .expect("a bar of radius 23/8")
}

/// A round bar of radius six about `z`, from `z = −2` up to `top`.
fn a_plate(model: &mut Model, top: f64) -> SolidKey {
    model
        .add_round_bar(
            6.0,
            top + 2.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -2.0)).expect("a finite shift"),
        )
        .expect("a bar of radius 6")
}

/// One solid for each of `truths`, each measuring as its own one inside the
/// error its measurement gives.
fn the_solids_measure(model: &Model, answer: &[SolidKey], truths: &[f64], what: &str) {
    assert_eq!(
        answer.len(),
        truths.len(),
        "{what}: {answer:?} against {truths:?}"
    );
    let mut left = truths.to_vec();
    for &solid in answer {
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(DEFLECTION).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: the solid measures: {why:?}"));
        let at = left
            .iter()
            .position(|&truth| (measured.volume() - truth).abs() <= measured.volume_error())
            .unwrap_or_else(|| {
                panic!(
                    "{what}: {} measured, {} off, against {left:?}",
                    measured.volume(),
                    measured.volume_error()
                )
            });
        left.remove(at);
    }
}
