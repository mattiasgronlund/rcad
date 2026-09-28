//! Two half bars stacked end to end on one pipe, each a bar less a block over
//! `x < 0`, and what their fuses should answer.
//!
//! # Why nothing here pins a refusal
//!
//! Every fuse of the two half bars refuses, all six of them, with one spelling
//! of the wall or two: `AFaceTrimmedByACurveThisCannotFollow`, raised in
//! `Arrangement::which_way_an_edge_runs_round` in `fuse/arrange.rs`. A half
//! bar's wall and its two half-disc caps have a half circle for a side, and
//! reading which half of the circle that is needs the circle's centre, which
//! `where_a_closed_crossing_is_centred` in `holds.rs` gives for a ball and a
//! cone and not for a pipe. `SolidView::face_holds_the_point` refuses on those
//! three faces for the same reason, through `the_sides_of` in `holds.rs`.
//!
//! Those refusals are **gaps** and not limits (§16.10): the answers exist, and
//! the kernel reaches them when the block is cut off *after* the whole bars are
//! fused, which is the same set by plain set algebra. That is what this file
//! pins, so a fix has the answer to check against. Pinning the refusal itself
//! would keep a gap as if it were settled. `docs/decisions.md` §325 has the
//! measurements, and the second refusal, which the same route meets once the
//! wall is spelled twice.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface,
    Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, DraftStep, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

const RADIUS: f64 = 2.0;

/// Each bar is two tall, the lower one on `z = 0` and the upper one on it.
const HEIGHT: f64 = 2.0;

/// `(A − B) ∪ (U − B)`, `(A − B) ∩ (U − B)` and `(A − B) − (U − B)` are
/// `(A ∪ U) − B`, `(A ∩ U) − B` and `(A − U) − B`, so fusing the whole bars
/// first and cutting the block off afterwards is the answer the fuse of the
/// two half bars should give.
///
/// - The union is a half bar four tall: `π·2²·4 / 2 = 8π`.
/// - The difference, either way round, is one half bar two tall: `4π`.
/// - The intersection is nothing. The bars only touch, on the disc at `z = 2`.
#[test]
fn cutting_the_block_off_after_fusing_the_whole_bars_answers_what_the_half_bars_should() {
    let policy = Policy::certified();
    for (rule, by_hand) in [
        (Rule::Union, Some(8.0 * PI)),
        (Rule::Intersection, None),
        (Rule::Difference, Some(4.0 * PI)),
    ] {
        for lower_first in [true, false] {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
            let lower = a_bar_on_the_one_pipe(&mut model, 0.0);
            let upper = a_bar_on_the_one_pipe(&mut model, HEIGHT);
            let [argument, tool] = if lower_first {
                [lower, upper]
            } else {
                [upper, lower]
            };
            let whole = model
                .fuse(
                    &[argument],
                    &[tool],
                    rule,
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|refused| panic!("{rule:?} of the whole bars: {refused:?}"));

            let mut halves = Vec::new();
            for solid in whole.solids().to_vec() {
                let block = a_block_over_negative_x(&mut model);
                let cut = model
                    .fuse(
                        &[solid],
                        &[block],
                        Rule::Difference,
                        &policy,
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|refused| panic!("{rule:?}, then less the block: {refused:?}"));
                halves.extend_from_slice(cut.solids());
            }
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|broken| panic!("{rule:?} came back broken: {broken:?}"));

            let Some(by_hand) = by_hand else {
                assert!(halves.is_empty(), "{rule:?}: {halves:?}");
                continue;
            };
            let [half] = halves.as_slice() else {
                panic!("{rule:?}, lower first {lower_first}: {halves:?}");
            };
            let measured = model
                .solid(*half)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.01).expect("a positive length"),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .expect("a half bar measures");
            assert!(
                (measured.volume() - by_hand).abs() <= measured.volume_error(),
                "{rule:?}, lower first {lower_first}: {} ± {} against {by_hand}",
                measured.volume(),
                measured.volume_error()
            );
        }
    }
}

/// A point the half bar's own faces cannot place has an answer: the bar holds
/// it and the block that was cut off does not.
#[test]
fn a_point_a_half_bar_cannot_place_is_inside_the_bar_and_outside_the_block() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let bar = a_bar_on_the_one_pipe(&mut model, 0.0);
    let block = a_block_over_negative_x(&mut model);
    let point = Point::new(1.0, 0.5, 0.5);

    let in_the_bar = model
        .solid(bar)
        .expect("this model made the key")
        .classify_point(point, &policy, &mut ProvenanceLog::new());
    let in_the_block = model
        .solid(block)
        .expect("this model made the key")
        .classify_point(point, &policy, &mut ProvenanceLog::new());
    assert_eq!(in_the_bar, Containment::Inside);
    assert_eq!(in_the_block, Containment::Outside);
}

/// A block over `x < 0` reaching well past both bars, above, below and to the
/// sides, so the only face of it the bars meet is `x = 0`.
fn a_block_over_negative_x(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![
        [-10.0, -10.0],
        [0.0, -10.0],
        [0.0, 10.0],
        [-10.0, 10.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    model
        .add_extrusion(
            &square,
            10.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -3.0)).expect("a finite shift"),
        )
        .expect("a positive height")
}

/// A round bar from `bottom` up [`HEIGHT`], whose wall names its axis at the
/// origin whichever bar it is.
///
/// Built face by face rather than with `Model::add_round_bar`, which places the
/// whole solid and so moves the wall's axis origin with it: the fuse would then
/// read the two walls as two spellings of one pipe, and the whole bars' union
/// would keep its wall as two faces (§231, §247).
fn a_bar_on_the_one_pipe(model: &mut Model, bottom: f64) -> SolidKey {
    let top = bottom + HEIGHT;
    let mut builder = SolidBuilder::new();
    let low = builder
        .add_vertex(Point::new(RADIUS, 0.0, bottom))
        .expect("a finite corner");
    let high = builder
        .add_vertex(Point::new(RADIUS, 0.0, top))
        .expect("a finite corner");
    let seam = builder
        .add_edge(low, high, EdgeRun::Straight)
        .expect("the seam up the wall");
    let low_circle = builder
        .add_edge(low, low, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the bottom cap circle");
    let high_circle = builder
        .add_edge(high, high, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the top cap circle");

    let cap = |builder: &mut SolidBuilder, z: f64, outward: f64, edge, turn| {
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, outward))
                .expect("the z axis is a unit normal");
        let coedge = builder.add_coedge(edge, turn).expect("one use of the rim");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("a cap is bounded by one whole circle");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a flat cap")
    };
    let bottom_cap = cap(
        &mut builder,
        bottom,
        -1.0,
        low_circle,
        Orientation::Reversed,
    );
    let top_cap = cap(&mut builder, top, 1.0, high_circle, Orientation::Forward);

    let round_the_wall = [
        (low_circle, Orientation::Forward),
        (seam, Orientation::Forward),
        (high_circle, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ]
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("a step round the wall"),
        )
    });
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("the wall's one wire");
    let pipe = Cylinder::new(
        Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0)).expect("a direction with a length"),
        RADIUS,
    )
    .expect("a positive radius");
    let wall = builder
        .add_face(Surface::Cylinder(pipe), &[wire])
        .expect("the curved wall");

    builder
        .add_shell(&[
            (bottom_cap, Orientation::Forward),
            (top_cap, Orientation::Forward),
            (wall, Orientation::Forward),
        ])
        .expect("a closed skin");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar described face by face")
}
