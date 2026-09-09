//! A fuse near the origin, in a model that also holds a block a long way out.
//!
//! `Assembly::commit` checks what the fuse wrote, in every build, and it is the
//! one place a check like that can be *asked the wrong question*: the sweep it
//! replaced was [`Model::validate`] over the whole model, and one of the
//! sweep's thirteen checks — the coordinate range — is a fold over every solid
//! there is. Two solids that are each perfectly fine can be too far apart to
//! compute between, so a whole-model range check after a write reports where
//! **somebody else's** solid sits as this fuse's bug (`docs/decisions.md` §66.3,
//! and `Model::check_what_was_just_written` for what is asked instead).
//!
//! **The numbers, and why they are exactly these.**
//! [`DynamicRange::LARGEST_SUPPORTED_RATIO`] is `1e11`.
//!
//! - The far block starts at `1024` and is `2` wide, so its largest
//!   coordinate is `1026` and its shortest edge is `2`: a ratio of `513`.
//! - The two blocks that are fused overlap in one corner by `2^-30`, so the
//!   answer carries edges that short. Against the answer's own largest
//!   coordinate, just under `4`, that is about `4.3e9` — inside the limit,
//!   which is why `Assembly::check_dynamic_range` lets the fuse through.
//! - Together — the far block's largest coordinate, `1026`, against the
//!   answer's shortest edge, `2^-30` — the ratio is `1026 * 2^30`, exactly
//!   `1_101_659_111_424`, about `1.1e12`, which is not.
//!
//! So before this change the fuse answered, wrote, and then panicked with
//! `Invalid(DynamicRangeTooWide { ratio: 1101659111424.0 })` — for a step it cut
//! near the origin and a block it never touched.
//!
//! **Why it is not a tautology.** A fuse that quietly failed to make a `2^-30`
//! step would pass this test by doing nothing interesting, so the test does not
//! merely assert that the fuse returns `Ok`: it asserts that the model
//! afterwards is one [`Model::validate`] *refuses*, by that very name. That is
//! the whole-model check firing — which is what the commit check would have
//! done if it still asked the whole-model question — with the fuse having
//! committed its answer anyway.
//!
//! **`2^-30` is exact**, so `Policy::certified()` decides everything here and a
//! refusal from the fuse would be a bug rather than the arithmetic giving up.

use rcad_geom::{DynamicRange, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, ProfileBuilder, Rule, SolidKey};

/// How far the two blocks overlap: `2^-30`, and the shortest edge the answer
/// ends up carrying.
const A_TINY_STEP: f64 = 1.0 / 1_073_741_824.0;

/// Far enough that `1024 / 2^-30` is past [`DynamicRange::LARGEST_SUPPORTED_RATIO`],
/// near enough that the block on its own is nowhere near it.
const A_LONG_WAY_OUT: f64 = 1024.0;

#[test]
fn a_fuse_at_the_origin_is_not_blamed_for_a_block_a_long_way_out() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::millimetres(4.0).expect("four millimetres is a size"));

    a_block(
        &mut model,
        Vector::new(A_LONG_WAY_OUT, 0.0, 0.0),
        &policy,
        &mut log,
    );
    let here = a_block(&mut model, Vector::new(0.0, 0.0, 0.0), &policy, &mut log);
    let corner = overlapping_it_in_one_corner(&mut model, &policy, &mut log);

    model
        .validate(&policy, &mut log)
        .expect("three plain blocks a thousand apart are a sound model");

    let fused = model
        .fuse(&[here], &[corner], Rule::Union, &policy, &mut log)
        .expect("two blocks overlapping in one corner unite");
    assert_eq!(
        fused.solids().len(),
        1,
        "a union of two touching blocks is one solid"
    );

    let refused = model
        .validate(&policy, &mut log)
        .expect_err("the model as a whole is now too wide to compute in");
    assert!(
        matches!(
            refused,
            Error::Invalid(Invalid::DynamicRangeTooWide { ratio })
                if ratio > DynamicRange::LARGEST_SUPPORTED_RATIO
        ),
        "the whole-model range is what a commit-time sweep would have panicked on, \
         and this is it firing: {refused:?}"
    );
}

/// The same fuse with nobody else in the model, which must both answer and
/// validate.
///
/// The control for the test above: it says the answer's own coordinates and its
/// own `2^-30` step are inside what this kernel computes in, so the refusal up
/// there is the far block's doing and not the step's.
#[test]
fn the_same_fuse_with_nothing_else_in_the_model_leaves_a_sound_model() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::millimetres(4.0).expect("four millimetres is a size"));

    let here = a_block(&mut model, Vector::new(0.0, 0.0, 0.0), &policy, &mut log);
    let corner = overlapping_it_in_one_corner(&mut model, &policy, &mut log);

    model
        .fuse(&[here], &[corner], Rule::Union, &policy, &mut log)
        .expect("two blocks overlapping in one corner unite");
    model
        .validate(&policy, &mut log)
        .expect("and what they leave behind is sound on its own");
}

/// A second block set so that its near corner sits `2^-30` inside the first,
/// which is where the answer's short edges come from.
fn overlapping_it_in_one_corner(
    model: &mut Model,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> SolidKey {
    let corner = 2.0 - A_TINY_STEP;
    a_block(model, Vector::new(corner, corner, corner), policy, log)
}

fn a_block(model: &mut Model, at: Vector, policy: &Policy, log: &mut ProvenanceLog) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]])
        .build(policy, log)
        .expect("a two-millimetre square");
    model
        .add_extrusion(
            &square,
            2.0,
            &Isometry::translation(at).expect("a plain translation"),
        )
        .expect("a two-millimetre block")
}
