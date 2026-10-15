//! A round bar `add_round_bar(r, 2r)` and a block over `x ∈ [0.6r, 2r]`,
//! `y ∈ [−2r, 0.8r]`, `z ∈ [−r, 3r]`, whose corner edge `(0.6r, 0.8r)` stands
//! on the bar's wall, under a tolerant policy (rows Q2028, Q2068, Q2081).
#![forbid(unsafe_code)]

use std::sync::mpsc;
use std::time::Duration;

use rcad_geom::{Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule};

/// Row Q2081. At radius `24.32` and working size `6.4` the union, bar first,
/// keeps the feet of the two rulings on each cap `3.55e-15` apart, and the
/// volume search of `nest_the_shells` then halved its mesh room from `48.64`
/// towards half that gap, every rung refused `BoundaryCrossesItself` on the
/// bottom cap and each costing about twice the last: no answer after 60 s.
/// Every other fuse of the family takes milliseconds, so ten seconds is room
/// and not a race.
#[test]
fn the_union_at_radius_24_32_comes_back_within_ten_seconds() {
    let (send, recv) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = send.send(the_union_of_the_bar_and_the_block(24.32, 6.4));
    });
    let answer = recv.recv_timeout(Duration::from_secs(10));
    assert!(
        answer.is_ok(),
        "the union of the bar and the block cornered on its wall did not come back in 10 s"
    );
}

/// What the union, bar first, comes back as: `Ok` with how many solids it
/// holds, or the refusal's `Debug`.
fn the_union_of_the_bar_and_the_block(radius: f64, size: f64) -> Result<usize, String> {
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
        .map_err(|refused| format!("{refused:?}"))
}
