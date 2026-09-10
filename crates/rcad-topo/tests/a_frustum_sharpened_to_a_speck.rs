//! A frustum whose small end is a speck beside its height: refused for a
//! certified caller, built for one that asked for the check to warn instead.
//!
//! The downstream sentence this comes from is in
//! `where_the_certified_dynamic_range_stops.rs` — a truncated cone on a real
//! ruling "needs a *tolerant* caller because rcad's certified range check
//! refuses that solid outright". That file pins **where** the frontier is, to
//! the ulp, by bisecting on the rim radius, and it never names a second policy.
//!
//! # What is already covered, and what is not
//!
//! "A warning reaches the caller" is **not** new.
//! `when_it_cannot_tell.rs`'s `a_policy_that_only_warns_about_the_range_says_so_in_the_log`
//! says it already, with a tighter oracle than this file's: an exact record
//! count, an exact enclosure and an exact `approximation()`.
//!
//! Three things about it are different here, and each is deliberate.
//!
//! - **It downgrades one field and nothing else.** That test uses
//!   `Policy::tolerant(100.0)`, which is [`Strictness::TolerantFallback`] as
//!   well as [`Severity::Warn`], so a solid it accepts may have been accepted
//!   by the fallback rather than by the warning. Here the policy is
//!   [`Policy::certified`] with [`Policy::with_dynamic_range_check`] laid over
//!   it, so a pass attributes to the one field that moved.
//! - **The shape is a cone.** That test extrudes a unit square into a tower, so
//!   its smallest feature is an edge. A cone's is a *radius*, read off
//!   `Surface::Cone`'s own arm of `Model::check_dynamic_range` — the arm the
//!   neighbouring file records as droppable with the suite staying green.
//! - **The door is [`SolidBuilder::build`]**, which is where an importer knocks,
//!   rather than `Model::validate` on a model that is already in the arena.
//!
//! # Why this needed a helper that can refuse
//!
//! `cones::a_frustum` used to hardwire `Policy::certified()` and `expect` the
//! build, so a refusal arrived as a panic and no test could name it as the
//! expected outcome. `cones::a_frustum_under` takes the policy and hands back
//! the [`Result`], which is the shape the rest of the kernel already uses:
//! `Undecidable` is one of a predicate's four answers, and a refused operation
//! is an [`Err`] a caller reads, not a crash.

mod cones;

use rcad_geom::{DYNAMIC_RANGE_CHECK, DynamicRange, Scale};
use rcad_predicates::{Decision, Policy, ProvenanceLog, Severity, Strictness};
use rcad_topo::{Error, Invalid, Model};

/// Out per unit up. A power of two, so `slope * near` and `slope * far` are
/// exact and every corner is certified on its own cone — `tests/cones/mod.rs`
/// says why that matters.
const SLOPE: f64 = 0.5;

/// The near cap sits here, a hair above the apex, so its circle has a radius of
/// `SLOPE * NEAR` and that is the smallest feature in the model.
const NEAR: f64 = 1e-12;

/// The far cap sits here, which is also the largest coordinate in the model.
const FAR: f64 = 8.0;

/// The spread the kernel measures this frustum to be: its height over its rim,
/// `8 / 5e-13`, **written out rather than computed**.
///
/// The second copy is the point, and it is the rule the neighbouring file
/// states for the bound itself: `FAR / (SLOPE * NEAR)` is the same operator on
/// the same two operands `DynamicRange::measure` uses, so an assertion against
/// it would move with the code and pin nothing. Written out, it also pins
/// *which* two quantities the kernel picked — the seam chord would give `0.894`
/// and the far rim `2.0`, neither of which is this number.
const THE_SPREAD: f64 = 1.6e13;

/// A certified caller is refused this frustum, and refused it **for its
/// dynamic range** rather than for anything else about it.
///
/// The ratio on the refusal is the model's own height over its own rim, which
/// says the check measured the shape that was described and not some other one.
#[test]
fn a_certified_caller_is_refused_a_frustum_whose_near_cap_is_a_speck() {
    const {
        assert!(
            THE_SPREAD > DynamicRange::LARGEST_SUPPORTED_RATIO,
            "this frustum has to spread wider than the kernel supports for there to be a refusal \
             at all"
        );
    }

    let mut model = Model::new(a_working_size());
    let mut log = ProvenanceLog::new();
    let refusal =
        cones::a_frustum_under(&mut model, SLOPE, NEAR, FAR, &Policy::certified(), &mut log);

    match refusal {
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => assert_eq!(
            ratio, THE_SPREAD,
            "the refusal names this frustum's own height over its own rim"
        ),
        other => panic!(
            "a frustum {FAR} tall on a rim of {} spreads {THE_SPREAD} wide and a certified caller \
             has to be refused it, but got {other:?}",
            SLOPE * NEAR
        ),
    }
    assert!(
        log.is_empty(),
        "a check that fails returns the refusal rather than recording it: {:?}",
        log.records()
    );
}

/// Downgrade that one check to a warning and the same description builds, with
/// the slop written into the provenance log.
///
/// This is what pins the refusal on the range check alone. If any other part of
/// a certified sweep also objected to this frustum, this test would go red and
/// the one above would be measuring two things at once.
#[test]
fn warning_instead_of_failing_is_the_only_change_this_frustum_needs() {
    let certified = Policy::certified();
    let asked_to_warn = certified.with_dynamic_range_check(Severity::Warn);
    assert_eq!(certified.dynamic_range_check(), Severity::Fail);
    assert_eq!(asked_to_warn.dynamic_range_check(), Severity::Warn);
    assert_eq!(
        asked_to_warn.strictness(),
        Strictness::Certified,
        "still certified: an undecided sign stays undecided, so nothing below can be let \
         through by a tolerant fallback instead"
    );
    assert_eq!(
        asked_to_warn.symbolic_perturbation(),
        certified.symbolic_perturbation(),
        "and nothing is being perturbed either"
    );

    let mut model = Model::new(a_working_size());
    let mut log = ProvenanceLog::new();
    let solid = cones::a_frustum_under(&mut model, SLOPE, NEAR, FAR, &asked_to_warn, &mut log)
        .expect("a caller who asked for a warning gets the solid");
    assert_eq!(
        model
            .solid(solid)
            .expect("this model made the key")
            .faces()
            .count(),
        3,
        "two caps and a wall"
    );

    assert_eq!(
        log.len(),
        1,
        "the spread is the only thing about this frustum that was not certified, so it is the \
         only record: {:?}",
        log.records()
    );
    let record = &log.records()[0];
    assert_eq!(record.predicate(), DYNAMIC_RANGE_CHECK);
    assert_eq!(
        record.outcome(),
        Decision::Undecidable {
            enclosure: ONE_STEP_AT_THE_FAR_CAP
        },
        "every predicate on a model this wide is uncertain by at least the gap between \
         neighbouring coordinates at its far end"
    );
    assert_eq!(
        record.approximation(),
        None,
        "nothing resolved an undecided sign here; the slop is the range itself"
    );
}

/// How far apart neighbouring `f64` are at [`FAR`], the largest coordinate in
/// the model, which is the finest anything in it can be told apart.
///
/// Asked of the float type — where is the next one up — rather than retyped as
/// the `largest * f64::EPSILON` that `DynamicRange::coordinate_spacing`
/// computes. `FAR` is a power of two, so this is `2⁻⁴⁹`.
const ONE_STEP_AT_THE_FAR_CAP: f64 = FAR.next_up() - FAR;

fn a_working_size() -> Scale {
    Scale::millimetres(FAR).expect("a frustum's height is a positive length")
}
