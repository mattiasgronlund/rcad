//! A pointed cone and a wedge thin enough that both of its walls hold the
//! cone's own axis exactly, fused all six ways.
//!
//! From Q1562 (`46f9f834`) on, every one of these six fuses refused
//! `DynamicRangeTooWide`. By `46b6a7ca` all six answer one solid each, and
//! the model validates after every one — Q2402 closed done at that base,
//! with no run saying which change fixed them. This file pins the six
//! answers by name, against the volumes worked out by hand.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::a_pointed_cone;

/// `a_pointed_cone(0.5, 8)`'s volume: a third of pi times its base radius
/// squared times its height, the base radius being its slope times 8.
const CONE: f64 = 128.0 * PI / 3.0;

/// The wedge's volume: its triangle's area, 0.3, times its 12-high extrusion.
const WEDGE: f64 = 3.6;

/// The part the cone and the wedge share, by hand: both wedge walls hold the
/// cone's axis exactly, so the shared part is a wedge of the cone itself, cut
/// by the plane `y = x / 60` the wedge's slanted wall lies in.
fn shared() -> f64 {
    64.0 * f64::atan(1.0 / 60.0) / 3.0
}

/// All six fuses of the cone and the wedge, both orders, under
/// `Policy::certified()`: each answers one solid, the model validates after,
/// and the answer's face count and volume hold to the hand values above.
#[test]
fn a_thin_wedge_against_a_cone_answers_and_validates_under_all_six_fuses() {
    for (rule, cone_first) in [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Intersection, true),
        (Rule::Intersection, false),
        (Rule::Difference, true),
        (Rule::Difference, false),
    ] {
        let what = format!("{rule:?}, cone first {cone_first}");
        let (truth, faces) = match (rule, cone_first) {
            (Rule::Union, _) => (CONE + WEDGE - shared(), 7),
            (Rule::Intersection, _) => (shared(), 4),
            (Rule::Difference, true) => (CONE - shared(), 4),
            (Rule::Difference, false) => (WEDGE - shared(), 7),
            (other, _) => panic!("no closed form written for {other:?}"),
        };
        let (model, solid) = fused(rule, cone_first, &what);
        assert_measures(&model, solid, truth, faces, &what);
    }
}

/// The cone and the wedge, fused under `rule`, the cone first among the
/// fuse's arguments when `cone_first` holds and last otherwise.
fn fused(rule: Rule, cone_first: bool, what: &str) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_pointed_cone(&mut model, 0.5, 8.0);
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [6.0, 0.0], [6.0, 0.1]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a triangle is a profile");
    let placed = Isometry::translation(Vector::new(0.0, 0.0, -2.0)).expect("a finite move");
    let wedge = model
        .add_extrusion(&triangle, 12.0, &placed)
        .expect("a triangle extrudes");
    let (arguments, tools) = if cone_first {
        ([cone], [wedge])
    } else {
        ([wedge], [cone])
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
    let &[solid] = answer.as_slice() else {
        panic!("{what}: one solid, got {answer:?}");
    };
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: {why:?}"));
    (model, solid)
}

/// The answer's face count matches `faces` exactly, and its volume lands
/// within the measure's own error of `truth`.
fn assert_measures(model: &Model, solid: SolidKey, truth: f64, faces: usize, what: &str) {
    let on_the_answer = model.solid(solid).expect("the fuse's own key");
    assert_eq!(on_the_answer.faces().count(), faces, "{what}: face count");
    let measured = model
        .solid(solid)
        .expect("the fuse's own key")
        .measure(
            Deflection::of(0.001).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the answer measures: {why:?}"));
    let (volume, error) = (measured.volume(), measured.volume_error());
    assert!(
        (volume - truth).abs() <= error,
        "{what}: {volume} measured, {error} off at most, against {truth}"
    );
}
