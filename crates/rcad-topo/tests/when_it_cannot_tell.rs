//! What happens when the numbers run out.
//!
//! A predicate that cannot decide is a result, and point classification hands
//! it straight back. An *operation* that cannot decide has nothing to return,
//! and that is an error carrying the records that caused it (decisions.md
//! §4.2).

use rcad_geom::{DYNAMIC_RANGE_CHECK, Isometry, Point, Scale};
use rcad_predicates::{Policy, ProvenanceLog, Severity};
use rcad_topo::{Containment, Error, Invalid, Model, ProfileBuilder};

fn empty_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn a_unit_square() -> Vec<[f64; 2]> {
    vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
}

/// The same four corners at unit size make a perfectly ordinary trapezoid.
fn a_trapezoid(size: f64) -> Vec<[f64; 2]> {
    vec![[0.0, 0.0], [0.0, size], [size, size], [2.0 * size, 0.0]]
}

#[test]
fn a_profile_too_big_for_f64_to_multiply_is_refused_rather_than_guessed() {
    ProfileBuilder::new(a_trapezoid(1.0))
        .build(&Policy::certified())
        .expect("the shape itself is sound; only its size is the problem");

    let refused = ProfileBuilder::new(a_trapezoid(1e200)).build(&Policy::certified());

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

/// A unit square stretched into a tower `1e30` high: the coordinates reach
/// `1e30` and the smallest edge is still one, so the spread is far past the
/// `1e11` where `f64` runs out of digits.
fn a_tower_too_tall_to_resolve_its_own_base() -> Model {
    let mut model = empty_model();
    let square = ProfileBuilder::new(a_unit_square())
        .build(&Policy::certified())
        .expect("a unit square");
    model
        .add_extrusion(&square, 1e30, &Isometry::identity())
        .expect("building it needs no decision");
    model
}

#[test]
fn a_solid_spanning_far_too_wide_a_range_fails_the_range_check() {
    let model = a_tower_too_tall_to_resolve_its_own_base();

    let refused = model.validate(&Policy::certified(), &mut ProvenanceLog::new());

    assert!(matches!(
        refused,
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { .. }))
    ));
}

/// A warning that nobody can read is not a warning. Under [`Severity::Warn`]
/// the sweep passes, and the record it leaves behind is the only way a caller
/// learns the model is too wide to compute in.
#[test]
fn a_policy_that_only_warns_about_the_range_says_so_in_the_log() {
    let model = a_tower_too_tall_to_resolve_its_own_base();
    let policy = Policy::tolerant(100.0);
    assert_eq!(policy.dynamic_range_check(), Severity::Warn);

    let mut log = ProvenanceLog::new();
    model
        .validate(&policy, &mut log)
        .expect("a warning is not a refusal, so the sweep still passes");

    assert_eq!(log.len(), 1, "a warning is a record, not silence");
    let record = &log.records()[0];
    assert_eq!(record.predicate(), DYNAMIC_RANGE_CHECK);
    // Nothing in a model this wide can be resolved more finely than the gap
    // between neighbouring `f64` at its largest coordinate, which is the
    // magnitude of the `Approximation` the same check hands back.
    assert_eq!(record.enclosure(), 1e30 * f64::EPSILON);
    assert_eq!(
        record.approximation(),
        None,
        "nothing resolved an undecided sign here; the slop is the range itself"
    );
}

#[test]
fn a_policy_that_ignores_the_range_records_nothing_at_all() {
    let model = a_tower_too_tall_to_resolve_its_own_base();
    let policy = Policy::tolerant(100.0).with_dynamic_range_check(Severity::Ignore);

    let mut log = ProvenanceLog::new();
    model
        .validate(&policy, &mut log)
        .expect("ignoring the check cannot fail the sweep");

    assert!(log.is_empty(), "an ignored check has nothing to say");
}
