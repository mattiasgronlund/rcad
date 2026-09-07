//! A frustum whose small end is a speck beside its height: refused for a
//! certified caller, built for one that asked for the check to warn instead.
//!
//! The downstream sentence this comes from is in
//! `where_the_certified_dynamic_range_stops.rs` — a truncated cone on a real
//! ruling "needs a *tolerant* caller because rcad's certified range check
//! refuses that solid outright". That file pins **where** the frontier is, to
//! the ulp, by bisecting on the rim radius. It never names a second policy, so
//! the second half of the sentence — that a caller who downgrades the check
//! gets the solid, and gets told — is nowhere in the suite.
//!
//! This file is that half. One shape, two policies, and the only difference
//! between the two is [`Policy::with_dynamic_range_check`]. Everything else
//! stays [`Policy::certified`], which is what makes the refusal attributable:
//! the solid is sound, the arithmetic on it is exact, and the range check is
//! the single thing standing between the caller and it.
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
use rcad_predicates::{Decision, Policy, ProvenanceLog, Severity};
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

/// Height over rim radius: what the kernel measures the spread of this frustum
/// to be, worked out here rather than copied from a run.
const THE_SPREAD: f64 = FAR / (SLOPE * NEAR);

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
}

/// Downgrade that one check to a warning and the same description builds, with
/// the slop written into the provenance log.
///
/// This is what pins the refusal on the range check alone. If any other part of
/// a certified sweep also objected to this frustum, this test would go red and
/// the one above would be measuring two things at once.
#[test]
fn warning_instead_of_failing_is_the_only_change_this_frustum_needs() {
    let asked_to_warn = Policy::certified().with_dynamic_range_check(Severity::Warn);
    assert_eq!(
        Policy::certified().dynamic_range_check(),
        Severity::Fail,
        "the two policies differ in this and in nothing else"
    );

    let mut model = Model::new(a_working_size());
    let mut log = ProvenanceLog::new();
    let solid = cones::a_frustum_under(&mut model, SLOPE, NEAR, FAR, &asked_to_warn, &mut log)
        .expect("a caller who asked for a warning gets the solid");
    assert_eq!(model.solid(solid).faces().count(), 3, "two caps and a wall");

    let spread_was_recorded = log
        .records()
        .iter()
        .find(|record| record.predicate() == DYNAMIC_RANGE_CHECK)
        .expect("a caller who is handed a model this wide is told so");
    assert!(
        matches!(spread_was_recorded.outcome(), Decision::Undecidable { .. }),
        "the record says every predicate on this model is uncertain, which is what the spread \
         means: {:?}",
        spread_was_recorded.outcome()
    );
    assert!(
        spread_was_recorded.enclosure() > 0.0,
        "and it carries how much slop that is"
    );
}

fn a_working_size() -> Scale {
    Scale::millimetres(FAR).expect("a frustum's height is a positive length")
}
