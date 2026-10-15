//! A round bar `add_round_bar(r, 2r)` and a block over `x ∈ [0.6r, 2r]`,
//! `y ∈ [−2r, 0.8r]`, `z ∈ [−r, 3r]`, whose corner edge `(0.6r, 0.8r)` stands
//! on the bar's wall, under a tolerant policy (rows Q2028, Q2068, Q2081).
//!
//! **Q2081: the union, bar first, did not come back** at radius `24.32` and
//! working size `6.4`, nor at its power-of-two image, radius `0.38` and size
//! `0.1`, where every other fuse of the family takes milliseconds. The block's
//! faces `y = 0.8r` and `x = 0.6r` cut the pipe in two rulings a hair apart,
//! since the corner `(0.6r, 0.8r)`, rounded, is a hair outside the bar, and
//! each cap has an arc between their feet two roundings long. Its two ends
//! are stored with one `y` though the circle runs 37 degrees off `x` there.
//! The first wrong step was facing that arc:
//! `Edge::the_way_across_the_chord_into_the_arc` in `fuse/assemble.rs` read
//! its side off the stored chord, and on the bottom cap faced it towards the
//! bar's axis, so its picker held the other 286 degrees of the circle. The
//! volume search of `nest_the_shells` then meshed that cap the long way round,
//! every rung of `the_coarsest_mesh_whose_sides_do_not_cross` in `validate.rs`
//! was refused `BoundaryCrossesItself`, and the ladder walked down towards
//! half the arc's length, each rung costing about twice the last. The arc is
//! now faced by which way along its chord it sets off, as for any arc of a
//! closed curve whose ends' rounding chose its side.
//!
//! **What the union comes back as now is a refusal, and a gap**: the answer
//! exists. The mesher finds two corners of a cap at one stored point,
//! `(0.6r, 0.8r)` rounded: the foot of the ruling where `x = 0.6r` cuts the
//! pipe, and the foot of the block's corner edge, which in exact arithmetic is
//! `1.78e−15` from it, outside the bar.
//! The row Q2081 filed for it follows Q2079, which also writes this file, and
//! a probe of Q2079's change at this branch's tip left these two unions
//! refused the same way.

#![forbid(unsafe_code)]

use std::sync::mpsc;
use std::time::Duration;

use rcad_geom::{Isometry, Scale, Vector};
use rcad_mesh::CannotTriangulate;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule};

#[test]
fn the_union_at_radius_24_32_comes_back_within_ten_seconds() {
    assert!(
        the_union_within_ten_seconds(24.32, 6.4).is_some(),
        "the union of the bar and the block cornered on its wall did not come back in 10 s"
    );
}

/// The exact image of the case above under a scale by `2⁻⁶`, which ran past
/// 290 s before Q2081.
#[test]
fn the_union_at_radius_0_38_comes_back_within_ten_seconds() {
    assert!(
        the_union_within_ten_seconds(0.38, 0.1).is_some(),
        "the union of the bar and the block cornered on its wall did not come back in 10 s"
    );
}

/// Both unions are refused by the mesher, for two corners of a cap that are
/// stored at one point. This pins the gap the module's doc describes, so that
/// the row which answers it moves this test.
#[test]
fn both_unions_are_refused_for_two_corners_of_a_cap_stored_at_one_point() {
    for (radius, size) in [(24.32, 6.4), (0.38, 0.1)] {
        let answer = the_union_within_ten_seconds(radius, size);
        let Some(Err(Error::CannotMeshAFace {
            why: CannotTriangulate::CornersCoincide { .. },
            ..
        })) = answer
        else {
            panic!("radius {radius}, working size {size}: {answer:?}");
        };
    }
}

/// The union, bar first, or `None` when it has not come back within ten
/// seconds. Every other fuse of the family takes milliseconds, so ten seconds
/// is room and not a race.
fn the_union_within_ten_seconds(radius: f64, size: f64) -> Option<rcad_topo::Result<usize>> {
    let (send, recv) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = send.send(the_union_of_the_bar_and_the_block(radius, size));
    });
    recv.recv_timeout(Duration::from_secs(10)).ok()
}

/// What the union, bar first, comes back as: how many solids it holds, or the
/// refusal.
fn the_union_of_the_bar_and_the_block(radius: f64, size: f64) -> rcad_topo::Result<usize> {
    let mut model = Model::new(Scale::millimetres(size).expect("a working size"));
    let bar = model
        .add_round_bar(radius, 2.0 * radius, &Isometry::identity())
        .expect("a bar");
    let notch = ProfileBuilder::new(vec![
        [0.6 * radius, -2.0 * radius],
        [2.0 * radius, -2.0 * radius],
        [2.0 * radius, 0.8 * radius],
        [0.6 * radius, 0.8 * radius],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    let block = model
        .add_extrusion(
            &notch,
            4.0 * radius,
            &Isometry::translation(Vector::new(0.0, 0.0, -radius)).expect("a shift"),
        )
        .expect("a block");
    model
        .fuse(
            &[bar],
            &[block],
            Rule::Union,
            &Policy::tolerant(size),
            &mut ProvenanceLog::new(),
        )
        .map(|fused| fused.solids().len())
}
