//! Two combs leaning opposite ways whose teeth touch along one line, and share
//! a sliver a rounding thin (queue rows Q1612 and Q1614).
//!
//! A comb of two teeth two high, leaning back two along `x` for each step along
//! `y`, standing two high at the origin, and a comb of one tooth one high,
//! leaning forward two, standing two high at `(−5, 2, 1)`. Q554's soak of
//! `fuses_that_must_agree.rs` found them (seed 1).
//!
//! Exactly, the two share a line and no space. Their stored planes, rounded,
//! close a three-sided prism of `3.08e−32` between them, whose tip is
//! `1.24e−16` above `y = 2` and rounds onto it (Q1612's trace, at `63bc1064`).
//! A sliver that thin beside corners five from the origin is a range `f64`
//! cannot carry: a limit, since a length over a length is the same at every
//! scale (`Assembly::check_dynamic_range`).
//!
//! **Until Q1614 the intersection was refused for the wrong reason.** Its
//! answer is the prism alone, whose corners all round onto one plane, and
//! nesting the shells measured it as enclosing no space before the range was
//! asked: `Invalid(VolumeIsNotPositive { volume: 0.0 })`, either way round.
//! The fuse now asks the range before it nests the shells, so all six
//! certified fuses name the limit (`docs/decisions.md` §87.3).
//!
//! **Under a tolerant policy the intersection is now no solid** (queue row
//! Q1687). The range is only a warning there, so until Q1687 the prism went on
//! to be nested and was refused `VolumeIsNotPositive { volume: 0.0 }`. A
//! tolerant fuse now leaves out a shell of flat faces whose corners all lie
//! within the policy's wobble of the plane of one of its own faces
//! (`leave_out_each_shell_thinner_than_the_wobble` in `fuse/assemble.rs`,
//! §417's 2026-10-11 note). The prism's six corners all round onto `y = 2`, so
//! it is left out, and nothing is left. That is the exact answer: the two
//! combs share a line, and a line holds no space.

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, Rule, SolidKey};

use pairs::{Configuration, Elsewhere, Placed, Shape, Skew};

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// Every rule, either comb first, is refused for the range, at one ratio.
///
/// The union and both differences were refused the same way before Q1614;
/// both intersections were `VolumeIsNotPositive { volume: 0.0 }`.
#[test]
fn every_certified_fuse_of_the_two_combs_is_refused_for_the_range() {
    let (model, combs) = the_combs().build();
    for rule in EVERY_RULE {
        for [first, second] in [[combs[0], combs[1]], [combs[1], combs[0]]] {
            let mut model = model.clone();
            let mut log = ProvenanceLog::new();
            let fused = model.fuse(&[first], &[second], rule, &Policy::certified(), &mut log);
            let Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) = fused else {
                panic!("{rule:?}, {first:?} first: {fused:?}");
            };
            assert_eq!(
                ratio, 2.702_159_776_422_297_6e16,
                "{rule:?}, {first:?} first"
            );
            assert!(
                log.records().is_empty(),
                "{rule:?}, {first:?} first: a certified fuse logs nothing: {:#?}",
                log.records()
            );
        }
    }
}

/// Under `Policy::tolerant(25.0)` the range is only a warning, so moving where
/// it is asked moved nothing here. Queue row Q1615 owns the refusals.
///
/// The union answers one solid, both ways round, holding what the two combs
/// hold between them, with the one record the range warns with. The
/// intersection answers no solid, and the model it leaves validates: exactly,
/// the two combs share a line, which holds no space. The prism of `3.08e−32`
/// their rounded planes close is a shell whose corners all lie within the
/// policy's wobble of one of its planes, so the fuse leaves it out and logs
/// that once, `7.9e−15`, and logs nothing else. Until Q1687 it was refused
/// `VolumeIsNotPositive { volume: 0.0 }`. Either difference is refused as an
/// outline winding against its face, once the answer is written with its
/// corners rounded.
#[test]
fn under_a_tolerant_policy_the_union_answers_the_intersection_is_nothing_and_each_difference_is_refused()
 {
    let policy = Policy::tolerant(25.0);
    let (model, combs) = the_combs().build();
    let both = volume_of(&model, &combs);
    for [first, second] in [[combs[0], combs[1]], [combs[1], combs[0]]] {
        let mut union = model.clone();
        let mut log = ProvenanceLog::new();
        let answer = union
            .fuse(&[first], &[second], Rule::Union, &policy, &mut log)
            .unwrap_or_else(|why| panic!("the union, {first:?} first: {why:?}"));
        assert_eq!(log.records().len(), 1, "the union, {first:?} first");
        assert_eq!(answer.solids().len(), 1, "the union, {first:?} first");
        let held = volume_of(&union, answer.solids());
        assert!(
            (held - both).abs() <= 1e-9 * both,
            "the union, {first:?} first, holds {held}, and the two combs {both}"
        );

        let mut intersection = model.clone();
        let mut log = ProvenanceLog::new();
        let answer = intersection
            .fuse(&[first], &[second], Rule::Intersection, &policy, &mut log)
            .unwrap_or_else(|why| panic!("the intersection, {first:?} first: {why:?}"));
        assert!(
            answer.solids().is_empty(),
            "the intersection, {first:?} first: {:?}",
            answer.solids()
        );
        intersection
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("the intersection, {first:?} first: {why:?}"));
        let predicates: Vec<&str> = log
            .records()
            .iter()
            .map(|record| record.predicate())
            .collect();
        assert_eq!(
            predicates,
            ["a shell within the wobble of one of its planes taken as flat"],
            "the intersection, {first:?} first"
        );

        let mut difference = model.clone();
        let refused = difference.fuse(
            &[first],
            &[second],
            Rule::Difference,
            &policy,
            &mut ProvenanceLog::new(),
        );
        assert!(
            matches!(
                refused,
                Err(Error::TheFuseWroteABrokenAnswer {
                    broken: Invalid::OuterWireWindsAgainstTheNormal
                })
            ),
            "the difference, {first:?} first: {refused:?}"
        );
    }
}

fn the_combs() -> Configuration {
    Configuration {
        solids: vec![
            Placed {
                shape: Shape::Comb {
                    teeth: 2,
                    tooth_height: 2,
                },
                skew: Skew { x_per_y: -2 },
                height: 2,
                at: [0, 0, 0],
            },
            Placed {
                shape: Shape::Comb {
                    teeth: 1,
                    tooth_height: 1,
                },
                skew: Skew { x_per_y: 2 },
                height: 2,
                at: [-5, 2, 1],
            },
        ],
        turn: None,
        elsewhere: Elsewhere::here(),
    }
}

fn volume_of(model: &Model, solids: &[SolidKey]) -> f64 {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.1).expect("a deflection"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a solid of flat faces measures")
                .volume()
        })
        .sum()
}
