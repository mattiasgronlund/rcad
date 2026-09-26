//! §282.4's probe, rebuilt, and the sweep §282.3 asked for before `arrange.rs`
//! touches the glue.
//!
//! A plate 10 × 10 × 2, a bar of radius 1 at `(3, 5)` through it (a hole with
//! no wall of its own touching the plate's edge), and a block over
//! `x ∈ [6, 16]`, `y ∈ [2, 8]` through it (a cut whose three walls join the
//! plate's own boundary, since the block runs past `x = 10`). §282.4 measured
//! each tool alone answering cleanly and both tools together refusing
//! `ResultIsNotManifold`, then deleted the probe. It is rebuilt below as
//! [`each_rule_alone_with_the_block_answers_one_solid`] and
//! [`both_tools_together_refuse_on_the_same_glued_cells`], with the block
//! standing in for "one tool alone" because it keeps every answer a single
//! solid in both argument orders — the bar alone does not, since `bar \ plate`
//! is two disjoint stubs, one above the plate and one below.
//!
//! # The sweep
//!
//! §282.3 says the arrange.rs row (Q204) needs to know, before it touches the
//! glue, whether it can produce a **wrong answer that still validates** rather
//! than only ever refusing. Swept: 0 to 3 holes (bars of radius 1, positioned
//! clear of every block variant below and of each other), three ways the
//! block's cut meets the plate's edge — one side (`x ∈ [6, 16]`, `y ∈ [2, 8]`,
//! as above), a corner (`x ∈ [6, 16]`, `y ∈ [6, 16]`), and two opposite sides
//! (`x ∈ [-5, 15]`, `y ∈ [4, 6]`, a slot straight through) — all three rules,
//! and both argument orders: 72 cases. Each was checked against the volume
//! worked out by hand and against the same tools fused one at a time (§282.4's
//! two independent checks), under `catch_unwind` for a panic.
//!
//! **No wrong answer and no panic turned up.** Every case with at least one
//! hole refused — always `ResultIsNotManifold` (count 1 or 3) or, in a few
//! cases, `Undecidable` on a `side of plane` predicate whose enclosure was
//! `1.7e-13` wide, a near-coincidence of the sweep's own numbers and not this
//! glue. Every case with no hole (the block alone, on any of the three edges)
//! answered and was right. So on this family of shapes the same-lump glue is
//! a **refusal**, not a silent wrong answer, and Q204 can rely on that: fixing
//! `group_the_cycles_into_cells` here is a correctness fix with no known case
//! of a wrong answer to also chase down, not a bug hiding behind a refusal.
//! This does not prove no shape anywhere gives a wrong answer — a family this
//! size cannot — but it is the family §282.4 named and the one this row was
//! asked to check first.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

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
/// top and bottom faces, the block's cut leaves the boundary lump in five
/// walks (§282.4), and `group_the_cycles_into_cells` glues all five to the
/// circle's outer walk into one piece with no point in it.
///
/// `Union` and `Difference` (plate as the argument) refuse with three faces
/// on the bad edge; `Intersection` and `Difference` (block and bar as the
/// argument) refuse with one. Whichever tool the fuse builds the glued piece
/// from names a different edge first, and the count follows.
#[test]
fn both_tools_together_refuse_on_the_same_glued_cells() {
    let policy = Policy::certified();

    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let plate = a_plate(&mut model);
    let bar = a_bar_through(&mut model, 3.0, 5.0);
    let block = a_block_through(&mut model, 6.0, 16.0, 2.0, 8.0);

    let cases = [
        (Rule::Union, vec![plate], vec![bar, block], 3),
        (Rule::Intersection, vec![plate], vec![bar, block], 1),
        (Rule::Difference, vec![plate], vec![bar, block], 3),
        (Rule::Difference, vec![bar, block], vec![plate], 1),
    ];
    for (rule, arguments, tools, expected_count) in cases {
        let mut model = model.clone();
        let refused = model.fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new());
        let Err(Error::ResultIsNotManifold { count }) = refused else {
            panic!(
                "{rule:?} of the plate, the bar and the block: expected ResultIsNotManifold, got {refused:?}"
            );
        };
        assert_eq!(
            count, expected_count,
            "{rule:?}: how many faces the bad edge has"
        );
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .expect("a refused fuse leaves the model exactly as it was");
    }
}
