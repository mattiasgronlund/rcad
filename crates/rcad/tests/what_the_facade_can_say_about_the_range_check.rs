//! A caller holding nothing but `rcad` can name the dynamic range check and
//! arrive at the number it was refused for.
//!
//! Two halves of `docs/decisions.md` §15.6, one check. A refusal carries a
//! ratio, and a caller that cannot reach [`DynamicRange`] has nothing to compare
//! that ratio against and no way to work it out for itself — it can copy the
//! digits of the bound, and a copy is right until the bound moves. A warning
//! carries a name, and a caller that cannot reach [`DYNAMIC_RANGE_CHECK`] can
//! count the records in its own log without being able to say which check any of
//! them came from.
//!
//! Both tests here are about what the *caller* can work out, not about where the
//! check fires — `coordinates_that_span_too_wide_a_range.rs` is where the check's
//! own behaviour under the three severities is pinned.
//!
//! **Nothing in this file imports anything but `rcad`**, for the reason
//! `a_cone_and_a_torus_through_the_front_door.rs` gives at its head: if a line
//! here needs `rcad_geom`, the front door is still shut.

use rcad::{
    DYNAMIC_RANGE_CHECK, DynamicRange, Error, Invalid, Isometry, Model, Policy, Profile,
    ProfileBuilder, ProvenanceLog, Scale, Severity, TopologyError,
};

/// The ratio the kernel refuses at is the ratio the caller measures, to the bit.
///
/// Not "both are over the limit" — that passes while the two numbers describe
/// different models. Bit equality is the claim worth making, because it says the
/// caller's route to the number and the kernel's are the same route, and it is
/// what goes wrong quietly if either changes what it measures.
#[test]
fn the_ratio_a_refusal_reports_is_one_the_caller_can_work_out_itself() {
    let model = a_square_with_one_hair_thin_edge();
    let measured =
        DynamicRange::measure(largest_coordinate_of(&model), smallest_feature_of(&model))
            .expect("both numbers are finite and above zero");
    assert!(
        !measured.is_within_supported_range(),
        "the caller can see the refusal coming, before it asks"
    );

    let refused: rcad::Result<()> = model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .map_err(Into::into);

    let Err(Error::Topology(TopologyError::Invalid(Invalid::DynamicRangeTooWide { ratio }))) =
        refused
    else {
        panic!("a range this wide has to be refused for being wide, and it was {refused:?}");
    };
    assert!(
        ratio.is_finite(),
        "two infinities compare equal and would make the next line say nothing"
    );
    assert_eq!(
        ratio.to_bits(),
        measured.ratio().to_bits(),
        "the kernel refused at {ratio} and the caller measured {}",
        measured.ratio()
    );
    assert!(ratio > DynamicRange::LARGEST_SUPPORTED_RATIO);
}

/// The warning is found by name and carries the slop, and a model inside the
/// bound leaves no record of that name behind.
///
/// Picking the record out by name is what a caller has to do the moment a log
/// holds two of them, and it is the thing a count cannot do: a log with one
/// record of some other check in it satisfies "there is exactly one record" and
/// says nothing about this one.
#[test]
fn the_warning_in_the_log_is_found_by_name_and_says_how_much_slop() {
    let policy = Policy::tolerant(1.0);
    assert_eq!(policy.dynamic_range_check(), Severity::Warn);

    let model = a_square_with_one_hair_thin_edge();
    let mut log = ProvenanceLog::new();
    model
        .validate(&policy, &mut log)
        .expect("a warning is not a refusal, so the sweep still passes");

    let named: Vec<_> = log
        .records()
        .iter()
        .filter(|record| record.predicate() == DYNAMIC_RANGE_CHECK)
        .collect();
    assert_eq!(named.len(), 1, "the log holds {}", log.len());
    // Nothing in a model this wide can be told from the thing next to it more
    // finely than neighbouring `f64` at its largest coordinate are apart, and
    // that is what the record hands back.
    assert_eq!(
        named[0].enclosure(),
        largest_coordinate_of(&model) * f64::EPSILON
    );

    let mut narrow_log = ProvenanceLog::new();
    let mut narrow = Model::new(Scale::millimetres(1.0).expect("a working size"));
    narrow
        .add_cube(1.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side one");
    narrow
        .validate(&policy, &mut narrow_log)
        .expect("a cube is not a hard model to work in");
    assert!(
        !narrow_log
            .records()
            .iter()
            .any(|record| record.predicate() == DYNAMIC_RANGE_CHECK),
        "a model inside the bound must not leave this name in a log"
    );
}

/// The unit square with one extra corner a trillionth of a millimetre from the
/// origin, so one edge is that short and everything else is about one.
///
/// Coordinates near one on purpose: a model whose coordinates were enormous
/// would fail the volume check before the range check ever ran.
fn a_square_with_one_hair_thin_edge() -> Model {
    let hair = 1e-12;
    let profile: Profile = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [hair, hair],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square with a nick out of one corner is a sound profile");

    let mut model = Model::new(Scale::millimetres(1.0).expect("a working size"));
    model
        .add_extrusion(&profile, 1.0, &Isometry::identity())
        .expect("it extrudes");
    model
}

fn largest_coordinate_of(model: &Model) -> f64 {
    model
        .vertices()
        .map(|vertex| vertex.point().largest_coordinate_magnitude())
        .fold(0.0, f64::max)
}

/// The shortest straight edge, which is what the smallest feature comes to for a
/// model with no curved face and no edge that runs right round one.
///
/// # Panics
///
/// If any edge is closed. A closed edge's ends are the same point, so its chord
/// is zero and stands for nothing; the kernel skips those and reads a radius off
/// the surface instead. This model has none, and the assertion is what makes the
/// bit equality above a claim about one route rather than two that happen to
/// agree here.
fn smallest_feature_of(model: &Model) -> f64 {
    let mut smallest = f64::INFINITY;
    for edge in model.edges() {
        assert!(
            !edge
                .is_closed()
                .expect("every run kind here is one we can read"),
            "this fixture is meant to have no closed edges"
        );
        smallest = smallest.min(edge.distance_between_its_ends());
    }
    assert!(smallest.is_finite(), "a model needs edges");
    smallest
}
