//! The dynamic-range check (`docs/decisions.md` §3.5, M0 exit criterion 9).
//!
//! Past about `1e11` the biggest coordinate in a model and the smallest feature
//! it is asked to resolve no longer fit in an `f64` together: the feature is
//! narrower than the gap between neighbouring numbers at that coordinate, and
//! every predicate near it is answering a question about numbers that have
//! already lost the detail. Roadmap §2.2 names this as the case to detect and
//! report rather than quietly degrade on.
//!
//! How loud to be is [`Policy`]'s to say, and the two constructors disagree on
//! purpose: [`Policy::certified`] fails, [`Policy::tolerant`] warns. Both
//! halves are visible from the facade alone: a refusal is an [`Error`], and a
//! warning is a [`Record`] in the [`ProvenanceLog`] the sweep is handed.

use rcad::{
    Error, Invalid, Isometry, Model, Policy, Profile, ProfileBuilder, ProvenanceLog, Scale,
    Severity, TopologyError,
};

/// The ratio above which `f64` has run out of digits.
const WIDEST_THAT_STILL_WORKS: f64 = 1e11;

#[test]
fn a_model_with_a_feature_a_million_times_finer_than_the_part_is_fine() {
    let mut model = Model::new(Scale::millimetres(1.0).expect("a working size"));
    model.add_cube(1.0).expect("a cube of side one");
    assert!(range_of(&model) < WIDEST_THAT_STILL_WORKS);

    let mut log = ProvenanceLog::new();
    model
        .validate(&Policy::tolerant(1.0), &mut log)
        .expect("a cube is not a hard model to work in");

    assert!(log.is_empty(), "certified outcomes are not recorded");
}

#[test]
fn a_strict_policy_refuses_a_model_whose_smallest_edge_is_a_trillionth_of_the_part() {
    let model = a_square_with_one_hair_thin_edge();
    assert!(range_of(&model) > WIDEST_THAT_STILL_WORKS);

    let refused: rcad::Result<()> = model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .map_err(Into::into);

    let Err(Error::Topology(TopologyError::Invalid(Invalid::DynamicRangeTooWide { ratio }))) =
        refused
    else {
        panic!("a range this wide has to be refused for being wide, and it was {refused:?}");
    };
    assert!(ratio > WIDEST_THAT_STILL_WORKS);
}

#[test]
fn a_tolerant_policy_carries_on_and_says_how_much_slop_it_accepted() {
    let model = a_square_with_one_hair_thin_edge();
    let policy = Policy::tolerant(1.0);
    assert_eq!(policy.dynamic_range_check(), Severity::Warn);

    let mut log = ProvenanceLog::new();
    model
        .validate(&policy, &mut log)
        .expect("a warning is not a refusal, so the sweep still passes");

    assert_eq!(log.len(), 1, "a warning is a record, not silence");
    // Nothing in a model this wide can be resolved more finely than the gap
    // between neighbouring `f64` at its largest coordinate.
    assert_eq!(
        log.worst_enclosure(),
        largest_coordinate_of(&model) * f64::EPSILON
    );
}

#[test]
fn a_policy_that_ignores_the_check_leaves_nothing_behind_to_read() {
    let model = a_square_with_one_hair_thin_edge();
    let policy = Policy::tolerant(1.0).with_dynamic_range_check(Severity::Ignore);

    let mut log = ProvenanceLog::new();
    model
        .validate(&policy, &mut log)
        .expect("ignoring the check cannot fail the sweep");

    assert!(log.is_empty(), "an ignored check has nothing to say");
}

/// The unit square with one extra corner a trillionth of a millimetre from the
/// origin, so that one edge is that short and everything else is about one.
///
/// The point is to get past `1e11` without wrecking anything else: a model
/// whose *coordinates* are enormous would fail the volume check first, because
/// summing tetrahedra that reach out to the origin cancels away every digit it
/// has. Keeping the coordinates near one and making a single feature tiny puts
/// the range where it needs to be and leaves the rest of the sweep working.
fn a_square_with_one_hair_thin_edge() -> Model {
    let hair = 1e-12;
    let profile: Profile = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [hair, hair],
    ])
    .build(&Policy::certified())
    .expect("a square with a nick out of one corner is a sound profile");

    let mut model = Model::new(Scale::millimetres(1.0).expect("a working size"));
    model
        .add_extrusion(&profile, 1.0, &Isometry::identity())
        .expect("it extrudes");
    model
}

/// The spread the model's own vertices and edges come to: largest coordinate
/// divided by shortest edge, which is what the sweep measures.
///
/// # Panics
///
/// If the model has no edges.
fn range_of(model: &Model) -> f64 {
    let smallest_feature = model
        .edges()
        .map(|edge| edge.length())
        .fold(f64::INFINITY, f64::min);
    assert!(smallest_feature.is_finite(), "a model needs edges");
    largest_coordinate_of(model) / smallest_feature
}

fn largest_coordinate_of(model: &Model) -> f64 {
    model
        .vertices()
        .map(|vertex| vertex.point().largest_coordinate_magnitude())
        .fold(0.0, f64::max)
}
