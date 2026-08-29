//! What happens when the numbers run out.
//!
//! A predicate that cannot decide is a result, and point classification hands
//! it straight back. An *operation* that cannot decide has nothing to return,
//! and that is an error carrying the records that caused it (decisions.md
//! §4.2).

use rcad_geom::{Isometry, Point, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Invalid, Model, ProfileBuilder};

fn empty_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn a_unit_square() -> Vec<[f64; 2]> {
    vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
}

#[test]
fn a_profile_too_big_for_f64_to_multiply_is_refused_rather_than_guessed() {
    let huge = 1e200;
    let refused = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [huge, huge],
        [huge, 2.0 * huge],
        [0.0, huge],
    ])
    .build(&Policy::certified());

    match refused {
        Err(Error::Undecidable { records }) => {
            assert!(!records.is_empty(), "the records say which decision failed");
            assert!(records.worst_enclosure() > 0.0);
        }
        other => panic!("this cannot be settled in f64: {other:?}"),
    }
}

#[test]
fn a_solid_too_tall_for_f64_to_multiply_says_it_cannot_place_a_point() {
    let mut model = empty_model();
    let square = ProfileBuilder::new(a_unit_square())
        .build(&Policy::certified())
        .expect("a unit square");
    let tower = model
        .add_extrusion(&square, 1e200, &Isometry::identity())
        .expect("nothing about building it needs a decision");

    let mut log = ProvenanceLog::new();
    let answer = model.solid(tower).classify_point(
        Point::new(0.5, 0.5, 1e199),
        &Policy::certified(),
        &mut log,
    );

    match answer {
        Containment::Undecidable { enclosure } => assert!(enclosure > 0.0),
        Containment::Inside | Containment::OnBoundary | Containment::Outside => {
            panic!("nothing in f64 can settle this")
        }
        other => panic!("unhandled answer: {other:?}"),
    }
    assert!(!log.is_empty(), "the log says what could not be settled");
}

#[test]
fn a_solid_spanning_far_too_wide_a_range_fails_the_range_check() {
    let mut model = empty_model();
    let square = ProfileBuilder::new(a_unit_square())
        .build(&Policy::certified())
        .expect("a unit square");
    model
        .add_extrusion(&square, 1e30, &Isometry::identity())
        .expect("building it needs no decision");

    let refused = model.validate(&Policy::certified());

    assert!(matches!(
        refused,
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { .. }))
    ));
}

#[test]
fn a_policy_that_only_warns_about_the_range_lets_the_model_through() {
    let mut model = empty_model();
    let square = ProfileBuilder::new(a_unit_square())
        .build(&Policy::certified())
        .expect("a unit square");
    model
        .add_extrusion(&square, 1e30, &Isometry::identity())
        .expect("building it needs no decision");

    model
        .validate(&Policy::tolerant(100.0))
        .expect("a tolerant policy only warns about the range");
}
