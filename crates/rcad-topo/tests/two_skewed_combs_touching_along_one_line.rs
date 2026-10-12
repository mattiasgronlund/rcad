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
//!
//! **Under a tolerant policy each difference is now the comb it starts from**
//! (queue row Q1710). Until Q1710 each comb less the other kept the prism as a
//! notch in its own shell, whose floor, a triangle of the other comb's bottom
//! or top, had all three corners rounded onto `y = 2` and so wound neither
//! way: refused `OuterWireWindsAgainstTheNormal`. The shell fold above takes
//! only a shell of its own. Now the fuse asks the same question of what the
//! intersection would keep before it keeps anything, and reads each piece of
//! the prism as lying on the other comb facing the other way, which is what
//! two solids touching from outside do (`fuse/select.rs`, §417's 2026-10-12
//! note). The intersection still keeps none of it, and logs the one record.

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
/// it is asked moved nothing here.
///
/// The union answers one solid, both ways round, holding what the two combs
/// hold between them, with the one record the range warns with. The
/// intersection answers no solid, and the model it leaves validates: exactly,
/// the two combs share a line, which holds no space. The prism of `3.08e−32`
/// their rounded planes close is a shell whose corners all lie within the
/// policy's wobble of one of its planes, so the fuse leaves it out and logs
/// that once, `7.9e−15`, and logs nothing else. Until Q1687 it was refused
/// `VolumeIsNotPositive { volume: 0.0 }`.
///
/// Either difference is the comb it starts from, corner for corner as a fuse
/// writes that comb less a block far away, with that one record and no other
/// (queue row Q1710). Until Q1710 each was refused as
/// an outline winding against its face: the comb less the other kept a notch
/// the shape of the prism, whose floor is a triangle of the other comb's
/// bottom or top with all three corners rounded onto `y = 2`, and so no
/// winding at all. Neither fold reached it. The prism there is no shell of
/// its own, only a notch in the comb's, and each of its corners has two meets
/// of three planes near it, not three (`docs/decisions.md` §417).
#[test]
fn under_a_tolerant_policy_the_union_answers_the_intersection_is_nothing_and_each_difference_is_the_comb_it_starts_from()
 {
    let policy = Policy::tolerant(25.0);
    let (model, combs) = the_combs().build();
    let both = volume_of(&model, &combs);
    for [mine, other] in [[0, 1], [1, 0]] {
        let [first, second] = [combs[mine], combs[other]];
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
        let mut log = ProvenanceLog::new();
        let answer = difference
            .fuse(&[first], &[second], Rule::Difference, &policy, &mut log)
            .unwrap_or_else(|why| panic!("the difference, {first:?} first: {why:?}"));
        let [left] = answer.solids() else {
            panic!("the difference, {first:?} first: {:?}", answer.solids());
        };
        difference
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("the difference, {first:?} first: {why:?}"));
        let (written, alone) = the_comb_less_a_block_far_away(mine, &policy);
        assert_eq!(
            corners_of(&difference, *left),
            corners_of(&written, alone),
            "the difference, {first:?} first"
        );
        assert_eq!(
            difference
                .solid(*left)
                .expect("a key the fuse made")
                .faces()
                .count(),
            model
                .solid(first)
                .expect("a key the combs made")
                .faces()
                .count(),
            "the difference, {first:?} first"
        );
        assert_eq!(
            volume_of(&difference, answer.solids()),
            volume_of(&written, &[alone]),
            "the difference, {first:?} first"
        );
        let predicates: Vec<&str> = log
            .records()
            .iter()
            .map(|record| record.predicate())
            .collect();
        assert_eq!(
            predicates,
            ["a shell within the wobble of one of its planes taken as flat"],
            "the difference, {first:?} first"
        );
    }
}

/// The model a difference of the comb at `index` in [`the_combs`], less a block
/// far from it, leaves, and the one solid it answers.
///
/// A fuse writes each corner where its planes meet in `f64`, not where the
/// input put it, so this is the comb as any fuse writes it. Comb 1's two
/// corners at `(−1, 2, z)` come out at `x = −1.000_000_000_000_000_4` here,
/// under either policy, with no other comb anywhere near.
fn the_comb_less_a_block_far_away(index: usize, policy: &Policy) -> (Model, SolidKey) {
    let mut configuration = the_combs();
    configuration.solids[1 - index] = Placed {
        shape: Shape::Block { width: 1, depth: 1 },
        skew: Skew { x_per_y: 0 },
        height: 1,
        at: [20, 20, 20],
    };
    let (mut model, solids) = configuration.build();
    let answer = model
        .fuse(
            &[solids[index]],
            &[solids[1 - index]],
            Rule::Difference,
            policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("comb {index} less a block far away: {why:?}"));
    let [alone] = answer.solids() else {
        panic!("comb {index} less a block far away: {:?}", answer.solids());
    };
    (model, *alone)
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

/// Every corner of `solid`'s faces, sorted, each once.
fn corners_of(model: &Model, solid: SolidKey) -> Vec<[f64; 3]> {
    let mut corners: Vec<[f64; 3]> = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .flat_map(|face| face.wires())
        .flat_map(|wire| wire.points())
        .map(|point| point.coordinates())
        .collect();
    corners.sort_by(|one, other| one.partial_cmp(other).expect("a corner is a number"));
    corners.dedup();
    corners
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
