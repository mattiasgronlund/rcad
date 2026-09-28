//! What happens when the numbers run out.
//!
//! A predicate that cannot decide is a result, and point classification hands
//! it straight back. An *operation* that cannot decide has nothing to return,
//! and that is an error carrying the records that caused it (decisions.md
//! §4.2).

use rcad_geom::{DYNAMIC_RANGE_CHECK, Isometry, Point, Scale};
use rcad_predicates::{
    Approximation, Cause, Level, Measures, Point as PredicatePoint, Policy, ProvenanceLog, Record,
    Severity, orient2d,
};
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

/// The same parallelogram at `1e200`, where every product of coordinates runs
/// past what `f64` holds.
///
/// It used to be refused. Its coordinates are uniformly enormous rather than
/// spread apart, so `orient2d` brings the whole shape into range by an exact
/// power of two and reads the same turns off it — and a shape written a
/// hundred octaves further out is still the same shape.
#[test]
fn a_profile_too_big_for_f64_to_multiply_is_normalised_rather_than_refused() {
    let mut log = ProvenanceLog::new();
    let ordinary = ProfileBuilder::new(a_parallelogram(1.0))
        .build(&Policy::certified(), &mut log)
        .expect("the shape itself is sound; only its size is the problem");
    assert!(log.is_empty(), "nothing about this needed approximating");
    assert_eq!(ordinary.approximation(), None);

    let enormous = ProfileBuilder::new(a_parallelogram(1e200))
        .build(&Policy::certified(), &mut log)
        .expect("size is not the problem it used to be");

    assert_eq!(
        enormous.approximation(),
        ordinary.approximation(),
        "the two are the same shape, so neither needed approximating"
    );
    assert!(log.is_empty(), "and a certified build records nothing");
}

/// A square with one corner cut off, where the cut is bent in the middle by
/// so little that nothing in `f64` can say which way the bend turns.
///
/// The cut runs from `(1, t)` through `(0.5, 0.5)` to `(t, 1)`, with
/// `t = 1e-305`. A straight cut would pass `t / √2` further out than the
/// middle corner does, so the bend turns clockwise. The two cheap levels
/// cannot see that: the differences they form round `t` away, the two products
/// the turn is the difference of come out equal, and the interval round the
/// difference sits over zero.
///
/// The exact level multiplies raw coordinates, and the one determinant holds
/// both `1 * 1` and `t * t`. `t * t` is `1e-610`, far below the `2^-969`
/// under which a product is no longer exact. A power of two that lifts it
/// above that has to be at least `2^529`, and any power past `2^511` takes
/// `1 * 1` off the top of the range. So no scaling lets the exact level run,
/// and
/// [`no_power_of_two_lets_f64_see_which_way_the_cut_bends`] tries every one.
///
/// This used to be a square with a spike `1e-150` from the origin. The spike's
/// determinants hold only `tiny * tiny` and `tiny * 1`, never `1 * 1`, so some
/// power of two always lifts them all into range, and since §324 `orient2d`
/// finds one. A spike at the origin cannot be made small enough to stay
/// undecided that way, which is why the shape changed (§324.7).
///
/// Everything else is an ordinary shape at working size. The lowest corner —
/// the one the winding direction is read off — is settled at the cheapest
/// level, so only the bend is ever in doubt.
fn a_square_whose_cut_corner_bends_too_little_for_f64_to_see() -> ProfileBuilder {
    ProfileBuilder::new(a_cut_corner_bent_by(BEND))
}

/// How far off `1` or `0` the cut's two ends sit. Normal, and with every bit
/// of its mantissa in use.
const BEND: f64 = 1e-305;

fn a_cut_corner_bent_by(t: f64) -> Vec<[f64; 2]> {
    vec![
        [-1.0, -1.0],
        [1.0, -1.0],
        [1.0, t],
        [0.5, 0.5],
        [t, 1.0],
        [-1.0, 1.0],
    ]
}

/// The claim that no scaling helps, run rather than argued
/// (`docs/decisions.md` §16.10). Every turn the tolerant build had to
/// approximate is asked again at every power of two that moves all three of
/// its points exactly, which is every scaling that keeps the geometry the
/// same. Not one of them is decided.
#[test]
fn no_power_of_two_lets_f64_see_which_way_the_cut_bends() {
    let mut log = ProvenanceLog::new();
    a_square_whose_cut_corner_bends_too_little_for_f64_to_see()
        .build(&Policy::tolerant(100.0), &mut log)
        .expect("a tolerant policy reads the bend as straight");
    assert!(!log.is_empty());

    for record in log.records() {
        let [a, b, c] = <[[f64; 2]; 3]>::try_from(
            record
                .inputs()
                .iter()
                .map(|point| {
                    let PredicatePoint::Exact([x, y, _]) = point else {
                        panic!("a profile's corners are exact: {point:?}");
                    };
                    [*x, *y]
                })
                .collect::<Vec<_>>(),
        )
        .expect("a turn has three corners");
        let mut tried = 0;
        for exponent in -1100..=1100 {
            let (Some(a), Some(b), Some(c)) = (
                exactly_scaled(a, exponent),
                exactly_scaled(b, exponent),
                exactly_scaled(c, exponent),
            ) else {
                continue;
            };
            tried += 1;
            assert!(
                !orient2d(a, b, c).is_certified(),
                "2^{exponent} settled which way {a:?} {b:?} {c:?} turns"
            );
        }
        assert!(tried > 1000, "only {tried} exponents scaled exactly");
    }
}

/// The control: bent by `1e-290` instead, the same cut is decided, and a
/// certified build takes it. So what the test above measures is the size of
/// the bend, and not something else about the shape.
#[test]
fn a_cut_corner_bent_by_1e_290_is_decided() {
    let mut log = ProvenanceLog::new();
    let built = ProfileBuilder::new(a_cut_corner_bent_by(1e-290))
        .build(&Policy::certified(), &mut log)
        .expect("level 3 runs once the whole shape is lifted by 2^509");
    assert_eq!(built.approximation(), None);
    assert!(log.is_empty());
}

/// `point`, both coordinates multiplied by `2^exponent`, or `None` when that
/// is not exact. It is exact while every result is a normal number, and a zero
/// stays zero at any exponent.
fn exactly_scaled(point: [f64; 2], exponent: i32) -> Option<[f64; 2]> {
    let half = exponent / 2;
    let mut scaled = point;
    for value in &mut scaled {
        if *value == 0.0 {
            continue;
        }
        *value = *value * 2f64.powi(half) * 2f64.powi(exponent - half);
        if !value.is_normal() {
            return None;
        }
    }
    Some(scaled)
}

#[test]
fn a_corner_f64_cannot_place_stops_a_certified_build() {
    let mut log = ProvenanceLog::new();

    let refused = a_square_whose_cut_corner_bends_too_little_for_f64_to_see()
        .build(&Policy::certified(), &mut log);

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

    let profile = a_square_whose_cut_corner_bends_too_little_for_f64_to_see()
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

    let refused = a_square_whose_cut_corner_bends_too_little_for_f64_to_see()
        .build(&Policy::tolerant(1e-6), &mut log);

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
    let profile = a_square_whose_cut_corner_bends_too_little_for_f64_to_see()
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
    let profile: Profile = a_square_whose_cut_corner_bends_too_little_for_f64_to_see()
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

/// A unit square extruded `1e200` high. Placing a point in it used to come
/// back [`Containment::Undecidable`] with a full log — an operation reporting
/// failure through the channel `docs/decisions.md` §12.5 says stays empty, and
/// §16.11 named it as the symptom worth chasing.
///
/// The tower's coordinates are uniformly enormous, so the predicates normalise
/// and the answers are ordinary. There is nothing subtle about where these
/// three points are.
#[test]
fn a_solid_too_tall_for_f64_to_multiply_can_still_place_a_point() {
    let mut model = empty_model();
    let square = ProfileBuilder::new(a_unit_square())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a unit square");
    let tower = model
        .add_extrusion(&square, 1e200, &Isometry::identity())
        .expect("nothing about building it needs a decision");

    for (point, expected) in [
        (Point::new(0.5, 0.5, 1e199), Containment::Inside),
        (Point::new(0.5, 0.5, -1.0), Containment::Outside),
        (Point::new(2.0, 0.5, 1e199), Containment::Outside),
    ] {
        let mut log = ProvenanceLog::new();
        let answer = model
            .solid(tower)
            .expect("this model made the key")
            .classify_point(point, &Policy::certified(), &mut log);
        assert_eq!(answer, expected, "{point:?}");
        assert!(
            log.is_empty(),
            "a certified classification records nothing at all"
        );
    }
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

/// An ordinary square, small enough that `orient2d` used to be unable to tell
/// which way it turns.
///
/// The history is worth keeping because it is two mistakes deep. First the
/// builder had an `unreachable!` here, on the argument that
/// `check_nothing_crosses` runs first and rules out three points on one line at
/// the lowest, leftmost corner — which is true, and is not what a straight turn
/// means. A straight turn is whatever `orient2d_tolerantly` called `Zero`, and
/// under a tolerant policy that includes an enclosure it waved through without
/// settling. At `1e-170` a clean right angle read straight, and the builder
/// panicked on input a caller could hand it. That became
/// `Error::Undecidable` (`docs/decisions.md` §4.2).
///
/// The second mistake was to stop there. A refusal is a *limit* when the answer
/// does not exist and a *gap* when it does and the arithmetic could not reach
/// it (§16.9), and this was always a gap — the test below said so out loud,
/// by rescaling the same square by hand. `orient2d` does that rescaling itself
/// now, so the square builds, certified, with nothing approximated.
#[test]
fn a_square_too_small_for_f64_to_multiply_is_normalised_and_built() {
    let side = 1e-170;
    let corners = vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]];
    let mut log = ProvenanceLog::new();

    let built = ProfileBuilder::new(corners)
        .build(&Policy::tolerant(1.0), &mut log)
        .expect("a clean right angle has a turn whatever size it is written at");
    assert_eq!(
        built.approximation(),
        None,
        "nothing here was approximated, so nothing may be recorded as approximate"
    );
    assert!(log.is_empty(), "and a certified build records nothing");
}

/// The certified policy used to refuse this square while the tolerant one
/// crashed on it, which was the safety ordering upside down. Now neither has
/// anything to argue about: no tolerance is involved in the answer at all.
#[test]
fn the_certified_policy_builds_the_same_square_with_no_tolerance_at_all() {
    let side = 1e-170;
    let corners = vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]];
    let mut log = ProvenanceLog::new();
    assert!(
        ProfileBuilder::new(corners)
            .build(&Policy::certified(), &mut log)
            .is_ok()
    );
    assert!(log.is_empty());
}

/// The same square with every coordinate's exponent shifted by 200 — the same
/// bits, a different size. It built before this change and it builds now, and
/// that it now agrees with the unscaled square is the whole point: the answer
/// never depended on where the shape was written.
#[test]
fn the_same_square_scaled_by_a_power_of_two_builds_to_the_same_answer() {
    let side = 1e-170 * 2f64.powi(200);
    let corners = vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]];
    let mut log = ProvenanceLog::new();
    let scaled = ProfileBuilder::new(corners)
        .build(&Policy::tolerant(1.0), &mut log)
        .expect("this one always built");
    assert_eq!(scaled.approximation(), None);
    assert!(log.is_empty());
}

/// A crossing `rcad-geom` could not name is an undecided *operation* up here
/// too, and not "bad geometry".
///
/// The one variant of `rcad_geom::Error` that is not bad caller input is the one
/// added at M3's gate (§24.10). Wrapped in [`Error::Geometry`] it would be filed
/// under "the caller handed over numbers that describe no shape", which is the
/// opposite of what happened, and §17.7's breakdown of what gave up would count
/// it in the wrong column.
#[test]
fn a_crossing_that_could_not_be_named_stays_undecided_and_does_not_become_bad_geometry() {
    let mut records = ProvenanceLog::new();
    records.push(Record::new(
        "where a plane and a sphere cross",
        &[],
        Level::Interval,
        8e-14,
        Measures::Area,
    ));

    match Error::from(rcad_geom::Error::Undecidable { records }) {
        Error::Undecidable { records } => assert_eq!(records.len(), 1),
        other => panic!("an undecided operation must stay undecided: {other:?}"),
    }

    assert!(matches!(
        Error::from(rcad_geom::Error::RadiusNotPositive { radius: -1.0 }),
        Error::Geometry(rcad_geom::Error::RadiusNotPositive { .. })
    ));
}
