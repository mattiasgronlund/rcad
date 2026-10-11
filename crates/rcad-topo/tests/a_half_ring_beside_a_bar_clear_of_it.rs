//! The top half of a ring beside a round bar that stands clear of it, fused
//! every way (row Q1513).
//!
//! **The half ring** is `rings::the_top_half_of_a_ring(model, 4, 1, seam)`: the
//! tube of radius one round the ring of radius four about `z`, cut by `z = 0`
//! and keeping `z ≥ 0`. It reaches out to `x = 5`. By Pappus it holds half of
//! `2π² · 4 · 1`, so `4π²`.
//!
//! **The bar** is `add_round_bar(23/8, 2)` moved by `(20, 0, −1)`: radius
//! `23/8`, from `z = −1` to `z = 1`, about the line `x = 20, y = 0`. It holds
//! `π · (23/8)² · 2 = 529π/32`, and it reaches in only to `x = 20 − 23/8`.
//!
//! The two never meet, so the union is the two solids as they stand, the
//! intersection is nothing, and each difference is its first argument.
//!
//! The last test stands the half ring on a bar of its own outer radius, whose
//! top rim is the half ring's outer equator, and fuses that every way too.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

mod rings;

/// How fine a mesh every volume here is read off.
const DEFLECTION: f64 = 0.01;

/// Half of a ring of four round a tube of one.
const THE_HALF_RING: f64 = 4.0 * PI * PI;

/// A bar of radius `23/8` and height two.
const THE_BAR: f64 = 529.0 * PI / 32.0;

/// A bar of radius five and height two, the half ring's outer radius.
const THE_BAR_UNDER_IT: f64 = 50.0 * PI;

#[test]
fn with_a_seam_every_fuse_is_the_solids_as_they_stand() {
    every_fuse_is_the_solids_as_they_stand(true);
}

#[test]
fn with_no_seam_every_fuse_is_the_solids_as_they_stand() {
    every_fuse_is_the_solids_as_they_stand(false);
}

/// Every rule, with the half ring as the argument and as the tool, and the
/// volumes each answer is when the two never meet.
fn every_fuse_is_the_solids_as_they_stand(a_seam: bool) {
    for (rule, ring_first, truths) in [
        (Rule::Union, true, vec![THE_HALF_RING, THE_BAR]),
        (Rule::Union, false, vec![THE_HALF_RING, THE_BAR]),
        (Rule::Intersection, true, Vec::new()),
        (Rule::Intersection, false, Vec::new()),
        (Rule::Difference, true, vec![THE_HALF_RING]),
        (Rule::Difference, false, vec![THE_BAR]),
    ] {
        let what = format!("{rule:?}, seam {a_seam}, half ring first {ring_first}");
        let mut model = Model::new(Scale::millimetres(50.0).expect("a positive size"));
        let ring = rings::the_top_half_of_a_ring(&mut model, 4.0, 1.0, a_seam)
            .expect("the top half of a ring described this way is a solid");
        let bar = model
            .add_round_bar(
                23.0 / 8.0,
                2.0,
                &Isometry::translation(Vector::new(20.0, 0.0, -1.0)).expect("a finite shift"),
            )
            .expect("a bar of radius 23/8");
        let (arguments, tools) = if ring_first {
            ([ring], [bar])
        } else {
            ([bar], [ring])
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
        the_solids_measure(&model, &answer, &truths, &what);
    }
}

/// The half ring standing on a bar of its own outer radius, `add_round_bar(5,
/// 2)` moved by `(0, 0, −2)`, fused every way, both ways round, with a seam
/// and without.
///
/// The bar's top face holds the half ring's flat ring, and its top rim is the
/// half ring's outer equator, where the bar's wall, `z = 0` and the ring meet
/// along a whole circle. The union is one solid of `50π + 4π²`, the
/// intersection nothing, the half ring less the bar `4π²` and the bar less the
/// half ring `50π`.
///
/// Until Q1566 every one was refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`,
/// since `rcad-geom` had no road for `z = 0`, the bar's wall and the ring.
/// With that road and no more, the union was refused as not a manifold, with
/// one face along each of two rims: the bar's wall reaches the rim cut by
/// `z = 0` and the pipe, and the ring's face the outer equator, which `z = 0`
/// and the ring name by the outside of the pipe of the ring's own radius.
/// `fuse/assemble.rs`'s `the_name_each_closed_curve_goes_by` joins the two now
/// (§428.5).
#[test]
fn on_a_bar_of_its_outer_radius_every_fuse_answers() {
    for a_seam in [true, false] {
        for (rule, ring_first, truths) in [
            (Rule::Union, true, vec![THE_HALF_RING + THE_BAR_UNDER_IT]),
            (Rule::Union, false, vec![THE_HALF_RING + THE_BAR_UNDER_IT]),
            (Rule::Intersection, true, Vec::new()),
            (Rule::Intersection, false, Vec::new()),
            (Rule::Difference, true, vec![THE_HALF_RING]),
            (Rule::Difference, false, vec![THE_BAR_UNDER_IT]),
        ] {
            let what = format!("{rule:?}, seam {a_seam}, half ring first {ring_first}");
            let mut model = Model::new(Scale::millimetres(50.0).expect("a positive size"));
            let ring = rings::the_top_half_of_a_ring(&mut model, 4.0, 1.0, a_seam)
                .expect("the top half of a ring described this way is a solid");
            let bar = model
                .add_round_bar(
                    5.0,
                    2.0,
                    &Isometry::translation(Vector::new(0.0, 0.0, -2.0)).expect("a finite shift"),
                )
                .expect("a bar of radius 5");
            let (arguments, tools) = if ring_first {
                ([ring], [bar])
            } else {
                ([bar], [ring])
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
            the_solids_measure(&model, &answer, &truths, &what);
        }
    }
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
