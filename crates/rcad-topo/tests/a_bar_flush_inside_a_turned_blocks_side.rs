//! A round bar flush inside the side of a turned block: the side touches the
//! bar's wall along one line, and where the block's top crosses that line the
//! side's crossing with the top and the wall's run the same way
//! (`docs/decisions.md` §244.8, Q1854).
//!
//! # The shape
//!
//! A round bar of radius `1` up `z`, 10 tall, its axis through `(−16, 0, 0)`,
//! and a block on the square `x ∈ [−17, −13]`, `y ∈ [−6, 6]`, 4 tall, moved
//! down 2 and turned `−0.5` about `x`. The turn leaves `x` alone, so the
//! block's side is exactly `x = −17`, one radius from the axis and parallel to
//! it: it touches the wall along the line `x = −17`, `y = 0`. It is §244.8's
//! shape with the bar `2⁻⁵¹` thinner.
//!
//! # Where it was refused
//!
//! On the block's top, the side cuts a line and the wall an ellipse, and at the
//! place they meet both run along `n × (−1, 0, 0)`, for the top's normal `n`.
//! The turn between them is `det(n, ∇side, ∇wall)`, and the side's and the
//! wall's gradients there are both `(−1, 0, 0)`: two rows the same, so the turn
//! is nothing. Worked out over the box the place is known in, the wall's
//! gradient is `(−1, ±0, ±8.9e-16)` and the determinant straddles nothing by
//! `8.5e-16`, so every certified fuse was refused `Undecidable` at `orient3d`,
//! and every tolerant one read it as nothing and wrote five records.
//!
//! It is a **gap**, not a limit: the side rests along the wall for the stored
//! numbers, which `intersect` proves under a certified policy, and any place
//! on both is on the line they touch along, where the two gradients are
//! parallel. `turns_anticlockwise` in `fuse/arrange.rs` reads it so.

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Scale, Surface, SurfaceMeet, Vector, intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Fused, Model, ProfileBuilder, Rule, SolidKey};

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The block's side and the bar's wall touch along one line, proved for the
/// stored numbers: `intersect` under a certified policy, which reads nothing
/// as nothing only where it proved it, answers one straight line.
#[test]
fn the_blocks_side_touches_the_bars_wall_along_one_line() {
    let (model, bar, block) = the_bar_and_the_block();
    let side = model
        .solid(block)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .find(|surface| {
            matches!(surface, Surface::Plane(flat) if flat.normal().components() == [-1.0, 0.0, 0.0])
        })
        .expect("the block has a side facing -x");
    let wall = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .find(|surface| matches!(surface, Surface::Cylinder(_)))
        .expect("a round bar has a wall");
    let mut log = ProvenanceLog::new();
    let meet = intersect(&side, &wall, &Policy::certified(), &mut log).expect("certified");
    let SurfaceMeet::Curves(runs) = meet else {
        panic!("a plane resting along a pipe touches it along a line: {meet:?}")
    };
    let [line] = &runs[..] else {
        panic!("one line: {runs:?}")
    };
    assert!(matches!(line.run(), EdgeRun::Straight), "{line:?}");
    assert!(log.records().is_empty(), "{:#?}", log.records());
}

/// Under either policy, every rule with either solid first answers one solid
/// that validates and holds its closed form, with nothing recorded.
///
/// **The closed forms.** Within the bar, `|y| ≤ 1`, the block's bottom is
/// below `z = 0` and its top crosses the bar at height `4 / cos ½ − 2 − y tan ½`,
/// whose mean over the disk is its value at the axis. So the two share
/// `π (4 / cos ½ − 2)`, the bar holds `10 π` and the block `4 · 12 · 4 = 192`.
///
/// Before the turn was read as nothing every certified fuse was refused
/// `Undecidable` at `orient3d`, `Interval`, enclosure `8.516308344827248e-16`,
/// and every tolerant one answered one solid with five `orient3d` records.
#[test]
fn every_fuse_of_a_bar_flush_inside_the_turned_blocks_side_answers_its_closed_form() {
    let shared = std::f64::consts::PI * (4.0 / 0.5_f64.cos() - 2.0);
    let bar = std::f64::consts::PI * 10.0;
    let block = 4.0 * 12.0 * 4.0;
    for policy in [Policy::certified(), Policy::tolerant(25.0)] {
        for rule in EVERY_RULE {
            for bar_first in [true, false] {
                let case = format!("{policy:?} {rule:?}, bar first {bar_first}");
                let mut log = ProvenanceLog::new();
                let (mut model, the_bar, the_block) = the_bar_and_the_block();
                let fused = if bar_first {
                    model.fuse(&[the_bar], &[the_block], rule, &policy, &mut log)
                } else {
                    model.fuse(&[the_block], &[the_bar], rule, &policy, &mut log)
                };
                assert!(log.records().is_empty(), "{case}: {:#?}", log.records());
                let closed_form = match (rule, bar_first) {
                    (Rule::Union, _) => block + bar - shared,
                    (Rule::Intersection, _) => shared,
                    (Rule::Difference, true) => bar - shared,
                    (Rule::Difference, false) => block - shared,
                    _ => unreachable!("{case}: EVERY_RULE names three rules"),
                };
                assert_one_solid_holding(&model, fused, closed_form, shared, &policy, &case);
            }
        }
    }
}

/// Checks that `fused` is one solid holding `closed_form` within the error bar
/// `measure` publishes, and that the model validates. The bar must be under
/// half of `shared`, the piece the bar and the block share, so that an answer
/// that left that piece out or counted it twice is off by more than its bar.
fn assert_one_solid_holding(
    model: &Model,
    fused: rcad_topo::Result<Fused>,
    closed_form: f64,
    shared: f64,
    policy: &Policy,
    case: &str,
) {
    let fused = fused.unwrap_or_else(|why| panic!("{case}: {why:?}"));
    let [solid] = fused.solids()[..] else {
        panic!("{case}: {:?}", fused.solids())
    };
    let measured = model
        .solid(solid)
        .expect("the fuse made it")
        .measure(
            Deflection::of(0.01).expect("a positive deflection"),
            policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{case}: {why:?}"));
    assert!(
        (measured.volume() - closed_form).abs() <= measured.volume_error(),
        "{case}: {closed_form} wanted, {measured:?}"
    );
    assert!(
        measured.volume_error() < shared / 2.0,
        "{case}: {shared} shared, {measured:?}"
    );
    model
        .validate(policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{case}: {why:?}"));
}

fn the_bar_and_the_block() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let bar = model
        .add_round_bar(
            1.0,
            10.0,
            &Isometry::translation(Vector::new(-16.0, 0.0, 0.0)).expect("a finite shift"),
        )
        .expect("a positive radius and height");
    let square = ProfileBuilder::new(vec![
        [-17.0, -6.0],
        [-13.0, -6.0],
        [-13.0, 6.0],
        [-17.0, 6.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    let placement = Isometry::translation(Vector::new(0.0, 0.0, -2.0))
        .expect("a finite shift")
        .compose(&Isometry::rotation(&Axis::x(), -0.5).expect("a finite turn"));
    let block = model
        .add_extrusion(&square, 4.0, &placement)
        .expect("a positive height");
    (model, bar, block)
}
