//! What the numbers mean, and how big the part is.

use rcad_predicates::{
    Approximation, Cause, Level, Measures, Policy, ProvenanceLog, Record, Severity,
};
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// The length unit the coordinates are written in.
///
/// Carried so that an importer or exporter knows what it is looking at (STEP,
/// roadmap §9.1). It **never takes part in arithmetic**: no method on this type
/// returns a conversion factor, and nothing in the kernel multiplies by one.
/// Coordinates are plain `f64` in whichever unit the model declared, and two
/// models in different units are two different models
/// (`docs/decisions.md` §3.5).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Unit {
    /// A thousandth of a millimetre.
    Micrometre,
    /// A thousandth of a metre. The usual choice for mechanical parts.
    Millimetre,
    /// A hundredth of a metre.
    Centimetre,
    /// The SI unit of length.
    Metre,
    /// 25.4 millimetres.
    Inch,
    /// Twelve inches.
    Foot,
}

impl Unit {
    /// The short name, for messages and for interop.
    #[must_use]
    pub fn symbol(self) -> &'static str {
        match self {
            Unit::Micrometre => "um",
            Unit::Millimetre => "mm",
            Unit::Centimetre => "cm",
            Unit::Metre => "m",
            Unit::Inch => "in",
            Unit::Foot => "ft",
        }
    }
}

/// How big the model is and what its numbers mean.
///
/// `nominal_size` is roughly how large the part is, in `unit` — a 100 mm
/// bracket has a nominal size of 100. Default tolerances come from it, so
/// getting it roughly right matters and getting it exactly right does not.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "(Unit, f64)", into = "(Unit, f64)")]
pub struct Scale {
    unit: Unit,
    nominal_size: f64,
}

impl Scale {
    /// How small a length has to be, relative to the nominal size, before the
    /// kernel treats it as nothing.
    ///
    /// `f64` resolves about one part in `2.2e-16`. Leaving seven decimal digits
    /// of headroom on top of that gives rounding somewhere to accumulate over a
    /// long chain of operations without eating the tolerance.
    pub const RELATIVE_LINEAR_TOLERANCE: f64 = 1e-9;

    /// A model roughly `nominal_size` across, measured in `unit`.
    ///
    /// # Errors
    ///
    /// [`Error::NominalSizeNotPositive`] if the size is zero, negative, or not
    /// a finite number.
    pub fn new(unit: Unit, nominal_size: f64) -> Result<Self> {
        if !(nominal_size.is_finite() && nominal_size > 0.0) {
            return Err(Error::NominalSizeNotPositive { nominal_size });
        }
        Ok(Self { unit, nominal_size })
    }

    /// A model roughly `nominal_size` millimetres across.
    ///
    /// # Errors
    ///
    /// As [`Scale::new`].
    pub fn millimetres(nominal_size: f64) -> Result<Self> {
        Self::new(Unit::Millimetre, nominal_size)
    }

    /// The unit the coordinates are written in.
    #[must_use]
    pub fn unit(&self) -> Unit {
        self.unit
    }

    /// Roughly how large the part is, in [`Scale::unit`].
    #[must_use]
    pub fn nominal_size(&self) -> f64 {
        self.nominal_size
    }

    /// The default length tolerance for a model this size.
    ///
    /// [`Scale::nominal_size`] times [`Scale::RELATIVE_LINEAR_TOLERANCE`]. For a
    /// 100 mm part that is `1e-7` mm.
    #[must_use]
    pub fn default_linear_tolerance(&self) -> f64 {
        self.nominal_size * Self::RELATIVE_LINEAR_TOLERANCE
    }
}

impl TryFrom<(Unit, f64)> for Scale {
    type Error = Error;

    fn try_from((unit, nominal_size): (Unit, f64)) -> Result<Self> {
        Self::new(unit, nominal_size)
    }
}

impl From<Scale> for (Unit, f64) {
    fn from(scale: Scale) -> Self {
        (scale.unit, scale.nominal_size)
    }
}

/// The spread between the biggest coordinate in a model and the smallest
/// feature it is asked to resolve.
///
/// The roadmap §2.2 names a spread too wide for `f64` as the case to detect and
/// report rather than quietly degrade on, and it costs one division.
///
/// **`1e11` is a margin and not the edge of anything.** Nothing goes wrong
/// there. A feature written straight into the coordinates is gone somewhere
/// between a spread of `2^53` and `2^54`, around `1e16`; the first thing that
/// actually fails is a constructed point, which stops being decidable at
/// `2^49`, and that is still 2 800 times the widest spread this allows.
/// `what_the_dynamic_range_check_is_a_margin_against.rs` measures all three.
///
/// Which of `2^53` and `2^54` is one bit of the coordinate: at `2^53` the
/// feature is exactly half an ulp, so `c + feature` is a tie and rounds to
/// even. A coordinate with its last mantissa bit clear swallows the feature
/// there; one with it set keeps it a binade longer. Quote `2^53` when it has to
/// be a single number, because it is the worse of the two.
///
/// **What the margin is for is conditioning, which a ratio cannot see.** A
/// construction of conditioning `t` divides the feature by `t` inside its own
/// intermediates, so the spread that decides the answer is `spread / t` and the
/// feature dies once that reaches the same tie — exactly, at every spread and
/// both parities measured. At `1e11` that covers constructions conditioned down
/// to about `2.2e-5` and no further, taking the worse side of the tie. §16.11
/// says this check is blind to what a ratio cannot express; `2.2e-5` is how
/// blind.
///
/// Past the cliff the kernel does not refuse. The offset rounds away, the points
/// really are coplanar, and a predicate answers about them correctly and about a
/// model that is not the one asked for. A refusal would be safe and this is not,
/// which is why the number is a margin rather than a tight bound.
///
/// [`DynamicRange::check`] fails on its own. [`DynamicRange::check_under_policy`]
/// is the same test with [`Policy::dynamic_range_check`] deciding how loud to be.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DynamicRange {
    largest_coordinate_magnitude: f64,
    ratio: f64,
}

impl DynamicRange {
    /// The widest spread this kernel will work at.
    ///
    /// Not the widest `f64` can carry — that is `2^53` or `2^54` depending on a
    /// bit of the coordinate, four and a half decades further out, and
    /// [`DynamicRange`]'s own doc says what the gap between the two is for.
    /// Moving this number is moving a safety margin, so move it against a
    /// measurement.
    pub const LARGEST_SUPPORTED_RATIO: f64 = 1e11;

    /// The spread of a model whose coordinates reach
    /// `largest_coordinate_magnitude` and whose smallest interesting feature is
    /// `smallest_feature_size` across.
    ///
    /// Use [`Point::largest_coordinate_magnitude`](crate::Point::largest_coordinate_magnitude)
    /// over the model's vertices for the first argument.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if the coordinate magnitude is negative or not a
    /// finite number, and [`Error::FeatureSizeNotPositive`] if the feature size
    /// is zero, negative, or not a finite number.
    pub fn measure(largest_coordinate_magnitude: f64, smallest_feature_size: f64) -> Result<Self> {
        if !(largest_coordinate_magnitude.is_finite() && largest_coordinate_magnitude >= 0.0) {
            return Err(Error::NotFinite {
                what: "largest coordinate magnitude",
            });
        }
        if !(smallest_feature_size.is_finite() && smallest_feature_size > 0.0) {
            return Err(Error::FeatureSizeNotPositive {
                size: smallest_feature_size,
            });
        }
        Ok(Self {
            largest_coordinate_magnitude,
            ratio: largest_coordinate_magnitude / smallest_feature_size,
        })
    }

    /// The spread itself: coordinate magnitude divided by feature size.
    #[must_use]
    pub fn ratio(&self) -> f64 {
        self.ratio
    }

    /// Whether the model still fits in `f64`.
    #[must_use]
    pub fn is_within_supported_range(&self) -> bool {
        self.ratio <= Self::LARGEST_SUPPORTED_RATIO
    }

    /// Loud version of [`DynamicRange::is_within_supported_range`].
    ///
    /// # Errors
    ///
    /// [`Error::DynamicRangeTooWide`] when the model asks `f64` for more digits
    /// than it has.
    pub fn check(&self) -> Result<()> {
        if self.is_within_supported_range() {
            Ok(())
        } else {
            Err(Error::DynamicRangeTooWide { ratio: self.ratio })
        }
    }

    /// [`DynamicRange::check`] with [`Policy::dynamic_range_check`] saying how
    /// loud to be about a failure.
    ///
    /// A model inside the supported range returns `Ok(None)` and leaves `log`
    /// alone whatever the policy says — a check that passed is a certified
    /// outcome, and certified outcomes are not recorded
    /// (`docs/decisions.md` §5).
    ///
    /// Outside it:
    ///
    /// - [`Severity::Ignore`] returns `Ok(None)` and records nothing, the same
    ///   as a model that was within range to begin with.
    /// - [`Severity::Warn`] pushes a record onto `log` and hands back an
    ///   [`Approximation`] for the caller to hang on whatever it builds. Every
    ///   coordinate in a model this wide carries at least one coordinate
    ///   spacing of slop, and that spacing is the magnitude on the tag.
    /// - [`Severity::Fail`] returns [`Error::DynamicRangeTooWide`].
    ///
    /// # Errors
    ///
    /// [`Error::DynamicRangeTooWide`] under [`Severity::Fail`].
    ///
    /// # Panics
    ///
    /// If `policy` carries a severity this crate has never heard of. Both
    /// crates ship together, so that is our own bug, not a caller's.
    pub fn check_under_policy(
        &self,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Option<Approximation>> {
        if self.is_within_supported_range() {
            return Ok(None);
        }

        match policy.dynamic_range_check() {
            Severity::Ignore => Ok(None),
            Severity::Warn => {
                let slop = self.coordinate_spacing();
                // A range check is not a predicate and has no sign to be
                // undecided about. `Undecidable` is still the honest outcome to
                // record: it says every predicate on this model is uncertain by
                // at least this much, which is exactly what a spacing wider
                // than the smallest feature means.
                log.push(Record::new(
                    DYNAMIC_RANGE_CHECK,
                    &[],
                    Level::Float,
                    slop,
                    // The spacing between neighbouring coordinates is already a
                    // distance, so this record needs no converting to be read
                    // as one (decisions.md §15.2).
                    Measures::Length,
                ));
                Ok(Some(Approximation::new(slop, Cause::TolerantFallback)))
            }
            Severity::Fail => Err(Error::DynamicRangeTooWide { ratio: self.ratio }),
            other => unreachable!("unhandled severity for the dynamic range check: {other:?}"),
        }
    }

    /// How far apart neighbouring `f64` are at the model's largest coordinate.
    ///
    /// No feature smaller than this can be told apart from the one next to it,
    /// wherever it sits in the model.
    fn coordinate_spacing(&self) -> f64 {
        self.largest_coordinate_magnitude * f64::EPSILON
    }
}

/// The name the dynamic range check goes by in provenance records.
pub const DYNAMIC_RANGE_CHECK: &str = "dynamic range check";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_model_of_no_size_is_refused() {
        assert_eq!(
            Scale::millimetres(0.0),
            Err(Error::NominalSizeNotPositive { nominal_size: 0.0 })
        );
    }

    #[test]
    fn a_model_of_negative_size_is_refused() {
        assert_eq!(
            Scale::millimetres(-5.0),
            Err(Error::NominalSizeNotPositive { nominal_size: -5.0 })
        );
    }

    #[test]
    fn a_hundred_millimetre_part_gets_a_tolerance_of_a_ten_millionth_of_a_millimetre() {
        let scale = Scale::millimetres(100.0).unwrap();
        assert!((scale.default_linear_tolerance() - 1e-7).abs() < 1e-20);
        assert_eq!(scale.unit(), Unit::Millimetre);
        assert_eq!(scale.nominal_size(), 100.0);
    }

    #[test]
    fn a_bigger_part_gets_a_bigger_default_tolerance() {
        let small = Scale::millimetres(1.0).unwrap();
        let large = Scale::millimetres(1000.0).unwrap();
        assert!(large.default_linear_tolerance() > small.default_linear_tolerance());
    }

    #[test]
    fn a_hundred_millimetre_part_with_micron_features_is_well_inside_the_range() {
        let range = DynamicRange::measure(100.0, 0.001).unwrap();
        assert_eq!(range.ratio(), 100_000.0);
        assert!(range.is_within_supported_range());
        assert_eq!(range.check(), Ok(()));
    }

    #[test]
    fn a_kilometre_of_model_with_nanometre_features_is_too_much_for_f64() {
        let range = DynamicRange::measure(1.0e6, 1.0e-6).unwrap();
        assert_eq!(range.ratio(), 1.0e12);
        assert!(!range.is_within_supported_range());
        assert_eq!(
            range.check(),
            Err(Error::DynamicRangeTooWide { ratio: 1.0e12 })
        );
    }

    #[test]
    fn a_spread_exactly_at_the_limit_is_still_allowed() {
        let range = DynamicRange::measure(1.0e11, 1.0).unwrap();
        assert_eq!(range.check(), Ok(()));
    }

    #[test]
    fn a_feature_size_of_zero_would_divide_by_zero_and_is_refused_instead() {
        assert_eq!(
            DynamicRange::measure(1.0, 0.0),
            Err(Error::FeatureSizeNotPositive { size: 0.0 })
        );
    }

    #[test]
    fn a_coordinate_magnitude_that_is_not_a_number_is_refused() {
        assert_eq!(
            DynamicRange::measure(f64::INFINITY, 1.0),
            Err(Error::NotFinite {
                what: "largest coordinate magnitude"
            })
        );
    }

    #[test]
    fn a_model_that_fits_records_nothing_however_strict_the_policy_is() {
        let range = DynamicRange::measure(100.0, 0.001).unwrap();
        for policy in [Policy::certified(), Policy::tolerant(100.0)] {
            let mut log = ProvenanceLog::new();
            assert_eq!(range.check_under_policy(&policy, &mut log), Ok(None));
            assert!(log.is_empty());
        }
    }

    #[test]
    fn a_certified_policy_refuses_a_model_that_is_too_wide_to_work_in() {
        let range = DynamicRange::measure(1.0e6, 1.0e-6).unwrap();
        let mut log = ProvenanceLog::new();
        assert_eq!(
            range.check_under_policy(&Policy::certified(), &mut log),
            Err(Error::DynamicRangeTooWide { ratio: 1.0e12 })
        );
        assert!(log.is_empty(), "a refusal is the report; nothing to log");
    }

    #[test]
    fn a_model_far_beyond_the_range_limit_passes_when_the_policy_says_ignore() {
        let range = DynamicRange::measure(1.0e6, 1.0e-6).unwrap();
        let policy = Policy::certified().with_dynamic_range_check(Severity::Ignore);
        let mut log = ProvenanceLog::new();

        assert_eq!(range.check_under_policy(&policy, &mut log), Ok(None));
        assert!(log.is_empty(), "ignore means nothing gets recorded either");
    }

    #[test]
    fn a_tolerant_policy_carries_on_and_hands_back_the_slop_it_accepted() {
        let range = DynamicRange::measure(1.0e6, 1.0e-6).unwrap();
        let mut log = ProvenanceLog::new();

        let tag = range
            .check_under_policy(&Policy::tolerant(1.0e6), &mut log)
            .unwrap()
            .expect("a model this wide puts slop into everything built from it");

        assert_eq!(tag.magnitude(), 1.0e6 * f64::EPSILON);
        assert_eq!(tag.cause(), Cause::TolerantFallback);
        assert_eq!(log.len(), 1);
        assert_eq!(log.records()[0].predicate(), DYNAMIC_RANGE_CHECK);
        assert_eq!(log.worst_wobble(1.0e6), tag.magnitude());
    }

    #[test]
    fn every_unit_has_a_short_name() {
        assert_eq!(Unit::Millimetre.symbol(), "mm");
        assert_eq!(Unit::Inch.symbol(), "in");
    }
}
