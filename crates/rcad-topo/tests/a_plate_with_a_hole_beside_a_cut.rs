//! §282.4's probe, rebuilt: a flat face whose sign rule glued five cells into
//! one, and what it answers now that it does not.
//!
//! A plate 10 × 10 × 2, a bar of radius 1 at `(3, 5)` through it (a hole with
//! no wall of its own touching the plate's edge), and a block over
//! `x ∈ [6, 16]`, `y ∈ [2, 8]` through it (a cut whose three walls join the
//! plate's own boundary, since the block runs past `x = 10`). §282.4 measured
//! each tool alone answering cleanly and both tools together refusing
//! `ResultIsNotManifold`, then deleted the probe. It is rebuilt below as
//! [`each_rule_alone_with_the_block_answers_one_solid`] and
//! [`both_tools_together_answer_the_volumes_worked_out_by_hand`], with the
//! block standing in for "one tool alone" because it keeps every answer a
//! single solid in both argument orders — the bar alone does not, since
//! `bar \ plate` is two disjoint stubs, one above the plate and one below.
//!
//! # The sweep, and what it did not see
//!
//! Queue row Q203 swept this family before `arrange.rs` was touched: 0 to 3
//! holes, three ways the block's cut meets the plate's edge, all three rules and
//! both argument orders, 72 cases, under `catch_unwind`. Every case with a hole
//! refused — `ResultIsNotManifold` (count 1 or 3) or, in a few, `Undecidable` on
//! a `side of plane` predicate whose enclosure was `1.7e-13` wide — and every
//! case without one answered right. This file used to say from that that **no
//! panic turned up**, and that the glue was a refusal and never worse.
//!
//! **True of those 72 and false beyond them.** Queue row Q20 found the same
//! glue making `Model::fuse` panic in `assemble.rs` ("a face joined out of
//! coplanar pieces has one outline, and this one has two") wherever every
//! region of the glued piece is kept: a block holding a bar and a small box
//! whose walls cut its floor in nine, and a block round a lens of a cone whose
//! floor a lid's plane cuts in two. The sweep never kept every region of a
//! glued piece, so it could not see that. Those two are pinned in
//! `a_floor_holding_a_circle_cut_in_two.rs`, and this family now answers.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

fn a_plate(model: &mut Model) -> SolidKey {
    let policy = Policy::certified();
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .build(&policy, &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    model
        .add_extrusion(&square, 2.0, &Isometry::identity())
        .expect("a plate extrudes")
}

/// A bar through the plate, sticking out one unit on each side so it cuts a
/// clean hole rather than stopping flush with a face.
fn a_bar_through(model: &mut Model, x: f64, y: f64) -> SolidKey {
    model
        .add_round_bar(
            1.0,
            4.0,
            &Isometry::translation(Vector::new(x, y, -1.0)).expect("a finite move"),
        )
        .expect("a bar extrudes")
}

/// A block through the plate, over `[x0, x1] × [y0, y1]`, sticking out the
/// same way the bar does.
fn a_block_through(model: &mut Model, x0: f64, x1: f64, y0: f64, y1: f64) -> SolidKey {
    let policy = Policy::certified();
    let rectangle = ProfileBuilder::new(vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]])
        .build(&policy, &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    model
        .add_extrusion(
            &rectangle,
            4.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -1.0)).expect("a finite move"),
        )
        .expect("a block extrudes")
}

fn one_solid(model: &Model, solids: &[SolidKey], policy: &Policy) -> f64 {
    let [solid] = solids else {
        panic!("expected one solid, got {solids:?}");
    };
    model
        .solid(*solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a solid measures")
        .volume()
}

/// The block's footprint on the plate is `[6, 10] × [2, 8]`, `4 × 6 = 24`
/// wide, since the plate stops at `x = 10` and the block runs on to `16`. Both
/// tools protrude one unit above and below the plate, so the shared depth is
/// the plate's own `2`: the overlap is `24 × 2 = 48`.
///
/// - `plate ∪ block = 200 + 240 − 48 = 392` (the block alone is
///   `10 × 6 × 4 = 240`).
/// - `plate ∩ block = 48`.
/// - `plate \ block = 200 − 48 = 152` (the plate with the corner notched out).
/// - `block \ plate = 240 − 48 = 192` (a bracket: the part of the block past
///   `x = 10` plus the flanges that stick out above and below the plate).
///
/// `Difference` is the one rule where the two argument orders answer
/// different solids, so both are pinned; `Union` and `Intersection` do not
/// depend on which group is which and are pinned once.
#[test]
fn each_rule_alone_with_the_block_answers_one_solid() {
    let policy = Policy::certified();

    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let plate = a_plate(&mut model);
    let block = a_block_through(&mut model, 6.0, 16.0, 2.0, 8.0);

    let cases = [
        (Rule::Union, vec![plate], vec![block], 392.0),
        (Rule::Intersection, vec![plate], vec![block], 48.0),
        (Rule::Difference, vec![plate], vec![block], 152.0),
        (Rule::Difference, vec![block], vec![plate], 192.0),
    ];
    for (rule, arguments, tools, by_hand) in cases {
        let mut model = model.clone();
        let fused = model
            .fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|refused| panic!("{rule:?} of the plate and the block: {refused:?}"));
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|broken| panic!("{rule:?} came back broken: {broken:?}"));
        let measured = one_solid(&model, fused.solids(), &policy);
        assert_eq!(
            measured, by_hand,
            "{rule:?} of the plate and the block, all flat faces: measured {measured} against \
             {by_hand} by hand"
        );
    }
}

/// The same two tools together: the bar's circle is one lump on the plate's
/// top and bottom faces, and the block's cut leaves the boundary lump in five
/// walks (§282.4). The circle's outer walk agrees with all five about every
/// closed curve, and `group_the_cycles_into_cells` used to glue all five to it
/// into one piece with no point in it, refusing `ResultIsNotManifold`. It is
/// placed now by which of the five its corners wind once round, and every rule
/// answers.
///
/// **The volumes, by hand.** The bar is `π · 1² · 4 = 4π` long and shares
/// `2π` with the plate; the block is `240` and shares `48` (the test above).
/// The two tools miss each other.
///
/// - `plate ∪ tools = 200 + 4π + 240 − 2π − 48 = 392 + 2π`, one solid.
/// - `plate ∩ tools = 2π + 48`, two solids: the plug of bar and the block's
///   bite.
/// - `plate \ tools = 200 − 2π − 48 = 152 − 2π`, one solid.
/// - `tools \ plate = 2π + 192`, three solids: the bar's two stubs and the
///   bracket of block.
///
/// The bar's wall is meshed, so each is held to its volume within what the
/// mesh says it may miss by.
#[test]
fn both_tools_together_answer_the_volumes_worked_out_by_hand() {
    let policy = Policy::certified();
    let round = 2.0 * core::f64::consts::PI;

    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let plate = a_plate(&mut model);
    let bar = a_bar_through(&mut model, 3.0, 5.0);
    let block = a_block_through(&mut model, 6.0, 16.0, 2.0, 8.0);

    let cases = [
        (Rule::Union, vec![plate], vec![bar, block], 1, 392.0 + round),
        (
            Rule::Intersection,
            vec![plate],
            vec![bar, block],
            2,
            48.0 + round,
        ),
        (
            Rule::Difference,
            vec![plate],
            vec![bar, block],
            1,
            152.0 - round,
        ),
        (
            Rule::Difference,
            vec![bar, block],
            vec![plate],
            3,
            192.0 + round,
        ),
    ];
    for (rule, arguments, tools, solids, by_hand) in cases {
        let mut model = model.clone();
        let fused = model
            .fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|refused| panic!("{rule:?} of the plate and both tools: {refused:?}"));
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|broken| panic!("{rule:?} came back broken: {broken:?}"));
        assert_eq!(fused.solids().len(), solids, "{rule:?}: how many solids");
        let (mut volume, mut error) = (0.0, 0.0);
        for &solid in fused.solids() {
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.01).expect("a positive length"),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .expect("a solid measures");
            volume += measured.volume();
            error += measured.volume_error();
        }
        assert!(
            (volume - by_hand).abs() <= error,
            "{rule:?} of the plate and both tools: measured {volume} against {by_hand} by hand, \
             and the mesh may miss by {error}"
        );
    }
}
