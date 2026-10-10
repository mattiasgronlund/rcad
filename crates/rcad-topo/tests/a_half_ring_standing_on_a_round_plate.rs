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

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

mod rings;

/// How fine a mesh every volume here is read off.
const DEFLECTION: f64 = 0.01;

/// Half of a ring of four round a tube of one.
const THE_HALF_RING: f64 = 4.0 * PI * PI;

/// The plate with its top at `z = 0`.
const THE_LOW_PLATE: f64 = 72.0 * PI;

/// The plate with its top at `z = 1/2`.
const THE_HIGH_PLATE: f64 = 90.0 * PI;

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
        [
            vec![THE_HIGH_PLATE + THE_HALF_RING - shared],
            vec![shared],
            vec![THE_HALF_RING - shared],
            vec![THE_HIGH_PLATE - shared],
        ],
    );
}

/// Every rule, with the half ring as the argument and as the tool, on a plate
/// whose top is at `top`. `truths` are the union, the intersection, the half
/// ring less the plate and the plate less the half ring.
fn every_fuse_answers(a_seam: bool, top: f64, truths: [Vec<f64>; 4]) {
    let [union, intersection, ring_less_plate, plate_less_ring] = truths;
    for (rule, ring_first, truths) in [
        (Rule::Union, true, &union),
        (Rule::Union, false, &union),
        (Rule::Intersection, true, &intersection),
        (Rule::Intersection, false, &intersection),
        (Rule::Difference, true, &ring_less_plate),
        (Rule::Difference, false, &plate_less_ring),
    ] {
        let what = format!("{rule:?}, seam {a_seam}, top {top}, half ring first {ring_first}");
        let mut model = Model::new(Scale::millimetres(50.0).expect("a positive size"));
        let ring = rings::the_top_half_of_a_ring(&mut model, 4.0, 1.0, a_seam)
            .expect("the top half of a ring described this way is a solid");
        let plate = a_plate(&mut model, top);
        let (arguments, tools) = if ring_first {
            ([ring], [plate])
        } else {
            ([plate], [ring])
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

#[test]
fn probe_q1580() {
    for (with_plate, bar_top) in [(true, 1.0), (false, 1.0), (true, 2.0), (false, 2.0)] {
        for a_seam in [true, false] {
            for (rule, ring_first) in [
                (Rule::Union, true),
                (Rule::Union, false),
                (Rule::Intersection, true),
                (Rule::Intersection, false),
                (Rule::Difference, true),
                (Rule::Difference, false),
            ] {
                let mut model = Model::new(Scale::millimetres(50.0).expect("a positive size"));
                let ring = rings::the_top_half_of_a_ring(&mut model, 4.0, 1.0, a_seam)
                    .expect("the top half of a ring described this way is a solid");
                let bar = model
                    .add_round_bar(
                        23.0 / 8.0,
                        bar_top + 1.0,
                        &Isometry::translation(Vector::new(0.0, 0.0, -1.0))
                            .expect("a finite shift"),
                    )
                    .expect("a bar");
                let (arguments, tools): (Vec<SolidKey>, Vec<SolidKey>) = if with_plate {
                    let plate = a_plate(&mut model, 0.0);
                    if ring_first {
                        (vec![ring, bar], vec![plate])
                    } else {
                        (vec![plate], vec![ring, bar])
                    }
                } else if ring_first {
                    (vec![ring], vec![bar])
                } else {
                    (vec![bar], vec![ring])
                };
                let answer = model.fuse(
                    &arguments,
                    &tools,
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                let said = match answer {
                    Ok(fused) => {
                        let volumes: Vec<f64> = fused
                            .solids()
                            .iter()
                            .map(|&solid| {
                                model
                                    .solid(solid)
                                    .expect("made here")
                                    .measure(
                                        Deflection::of(DEFLECTION).expect("positive"),
                                        &Policy::certified(),
                                        &mut ProvenanceLog::new(),
                                    )
                                    .map_or(f64::NAN, |measured| measured.volume())
                            })
                            .collect();
                        format!("answers {volumes:?}")
                    }
                    Err(rcad_topo::Error::Undecidable { records }) => format!(
                        "Undecidable {:?}",
                        records.records().last().map(|record| format!("{record:?}"))
                    ),
                    Err(other) => format!("{other:?}"),
                };
                let valid = model
                    .validate(&Policy::certified(), &mut ProvenanceLog::new())
                    .map_or_else(|why| format!("{why:?}"), |()| "valid".to_owned());
                eprintln!(
                    "PROBE plate {with_plate} top {bar_top} seam {a_seam} {rule:?} ring first {ring_first}: {said} {valid}"
                );
            }
        }
    }
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
