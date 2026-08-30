//! What happens when the numbers run out.
//!
//! A predicate that cannot decide is a result, and point classification hands
//! it straight back. An *operation* that cannot decide has nothing to return,
//! and that is an error carrying the records that caused it (decisions.md
//! §4.2).

use rcad_geom::{DYNAMIC_RANGE_CHECK, Isometry, Point, Scale};
use rcad_predicates::{Approximation, Cause, Policy, ProvenanceLog, Severity};
use rcad_topo::{Containment, Error, Invalid, Model, Profile, ProfileBuilder};

fn empty_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn a_unit_square() -> Vec<[f64; 2]> {
    vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
}

/// The same four corners at unit size make a perfectly ordinary parallelogram.
fn a_parallelogram(size: f64) -> Vec<[f64; 2]> {
    vec![[0.0, 0.0], [size, size], [size, 2.0 * size], [0.0, size]]
}

#[test]
fn a_profile_too_big_for_f64_to_multiply_is_refused_rather_than_guessed() {
    let mut log = ProvenanceLog::new();
    let ordinary = ProfileBuilder::new(a_parallelogram(1.0))
        .build(&Policy::certified(), &mut log)
        .expect("the shape itself is sound; only its size is the problem");
    assert!(log.is_empty(), "nothing about this needed approximating");
    assert_eq!(ordinary.approximation(), None);

    let refused = ProfileBuilder::new(a_parallelogram(1e200)).build(&Policy::certified(), &mut log);

    match refused {
        Err(Error::Undecidable { records }) => {
            assert!(!records.is_empty(), "the records say which decision failed");
            assert!(records.worst_wobble(1.0) > 0.0);
        }
        other => panic!("this cannot be settled in f64: {other:?}"),
    }
    assert!(
        log.is_empty(),
        "a failed build reports its records in the error, not in both places"
    );
}

/// A square with a spike so close to the origin that nothing in `f64` can say
/// which way the spike turns when it is seen from the square's far corners.
///
/// The exact level multiplies raw coordinates, and `1e-150 * 1e-150` is
/// `1e-300` — too small for the product to still be exact — so it declines to
/// run. The two cheap levels cannot help either: from a corner a whole unit
/// away, the two products the turn is the difference of are equal to within a
/// few of the last bits either one has, so the interval round the difference
/// sits over zero.
///
/// Everything else is an ordinary shape at working size. The lowest corner —
/// the one the winding direction is read off — is settled at the cheapest
/// level, so only the spike is ever in doubt.
fn a_square_with_a_spike_f64_cannot_place() -> ProfileBuilder {
    let tiny = 1e-150;
    ProfileBuilder::new(vec![
        [0.0, 0.0],
        [tiny, 3.0 * tiny],
        [2.0 * tiny, 0.0],
        [1.0, -1.0],
        [1.0, 1.0],
        [-1.0, 1.0],
        [-1.0, -1.0],
    ])
}

#[test]
fn a_corner_f64_cannot_place_stops_a_certified_build() {
    let mut log = ProvenanceLog::new();

    let refused = a_square_with_a_spike_f64_cannot_place().build(&Policy::certified(), &mut log);

    match refused {
        Err(Error::Undecidable { records }) => {
            assert_eq!(records.len(), 1, "the build stops at the first one");
            let record = &records.records()[0];
            assert_eq!(record.predicate(), "orient2d");
            assert!(record.enclosure() > 0.0);
            assert_eq!(
                record.approximation(),
                None,
                "a certified policy resolves nothing, so there is no slop to report"
            );
        }
        other => panic!("nothing in f64 can settle that corner: {other:?}"),
    }
    assert!(log.is_empty());
}

#[test]
fn a_tolerant_policy_says_in_the_log_what_it_had_to_approximate() {
    let mut log = ProvenanceLog::new();

    let profile = a_square_with_a_spike_f64_cannot_place()
        .build(&Policy::tolerant(100.0), &mut log)
        .expect("a tolerant policy reads those corners' enclosures as zero");

    assert!(
        !log.is_empty(),
        "the caller has to be able to learn what was approximated"
    );
    let slop = profile
        .approximation()
        .expect("the profile carries what it cost to accept it");
    assert_eq!(slop.cause(), Cause::UndecidableResolved);
    assert_eq!(slop.magnitude(), log.worst_wobble(100.0));
    // The turns in doubt are read off corners a unit away, so the widest
    // interval that straddled zero is a handful of the last bits of one. What
    // lands on the profile is not that interval though — `orient2d` measures an
    // area, so at this working size the distance a corner could be out by is a
    // hundredth of it (decisions.md §15.2).
    assert!(slop.magnitude() > 0.0 && slop.magnitude() < 16.0 * f64::EPSILON / 100.0);
    for record in log.records() {
        assert_eq!(record.predicate(), "orient2d");
        let resolved = record
            .approximation()
            .expect("every record here was resolved rather than given up on");
        assert_eq!(resolved.cause(), Cause::UndecidableResolved);
        // What goes on the entity is a distance, and what the cascade saw is
        // an area. At this working size they are a hundred apart, and the tag
        // is the one a caller can compare against a tolerance.
        assert_eq!(resolved.magnitude(), record.wobble(100.0));
        assert_ne!(resolved.magnitude(), record.enclosure());
        assert!(resolved.magnitude() <= slop.magnitude());
    }
}

/// Tolerance is a budget, not a shrug. The same loops under a policy with far
/// less room to spend stay refused.
#[test]
fn a_tolerant_policy_with_too_small_a_budget_still_refuses() {
    let mut log = ProvenanceLog::new();

    let refused = a_square_with_a_spike_f64_cannot_place().build(&Policy::tolerant(1e-6), &mut log);

    match refused {
        Err(Error::Undecidable { records }) => {
            assert!(records.worst_wobble(1e-6) > 0.0);
            assert_eq!(records.records()[0].approximation(), None);
        }
        other => panic!("that budget cannot cover this corner: {other:?}"),
    }
}

#[test]
fn slop_a_tolerant_profile_carried_lands_on_the_solid_built_from_it() {
    let mut model = empty_model();
    let profile = a_square_with_a_spike_f64_cannot_place()
        .build(&Policy::tolerant(100.0), &mut ProvenanceLog::new())
        .expect("a tolerant policy settles those corners");
    let slop = profile.approximation().expect("the profile is approximate");

    model
        .add_extrusion(&profile, 1.0, &Isometry::identity())
        .expect("an approximated profile still extrudes");

    assert_eq!(model.worst_approximation(), slop.magnitude());
    for vertex in model.vertices() {
        assert_eq!(vertex.approximation(), Some(slop));
    }
    for edge in model.edges() {
        assert_eq!(edge.approximation(), Some(slop));
    }
    for face in model.faces() {
        assert_eq!(face.approximation(), Some(slop));
    }
    model
        .validate(&Policy::tolerant(100.0), &mut ProvenanceLog::new())
        .expect("tagging everything equally keeps tol(vertex) >= tol(edge) >= tol(face)");
}

#[test]
fn a_certified_cube_leaves_nothing_approximate_anywhere() {
    let mut model = empty_model();
    let mut log = ProvenanceLog::new();

    model
        .add_cube(2.0, &Policy::certified(), &mut log)
        .expect("a cube of side two");

    assert!(
        log.is_empty(),
        "a fully certified operation records nothing at all"
    );
    assert_eq!(model.worst_approximation(), 0.0);
    let tagged: Vec<Option<Approximation>> = model
        .vertices()
        .map(|vertex| vertex.approximation())
        .chain(model.edges().map(|edge| edge.approximation()))
        .chain(model.faces().map(|face| face.approximation()))
        .filter(Option::is_some)
        .collect();
    assert!(
        tagged.is_empty(),
        "nothing here was approximated: {tagged:?}"
    );
}

/// Extruding the same loops twice must not make the model any more approximate
/// the second time round. The check that catches tolerance inflation is the one
/// that would have to be loosened to let that happen (decisions.md §4.3).
#[test]
fn extruding_an_approximated_profile_twice_does_not_inflate_the_slop() {
    let mut model = empty_model();
    let profile: Profile = a_square_with_a_spike_f64_cannot_place()
        .build(&Policy::tolerant(100.0), &mut ProvenanceLog::new())
        .expect("a tolerant policy settles those corners");
    let slop = profile.approximation().expect("the profile is approximate");

    model
        .add_extrusion(&profile, 1.0, &Isometry::identity())
        .expect("the first one");
    model
        .add_extrusion(&profile, 2.0, &Isometry::identity())
        .expect("the second one");

    assert_eq!(model.worst_approximation(), slop.magnitude());
}

#[test]
fn a_solid_too_tall_for_f64_to_multiply_says_it_cannot_place_a_point() {
    let mut model = empty_model();
    let square = ProfileBuilder::new(a_unit_square())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
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
        .build(&Policy::certified(), &mut ProvenanceLog::new())
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
