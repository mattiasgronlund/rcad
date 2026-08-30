//! Where a spline's pieces meet, and how smoothly.

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// The highest degree a knot vector may carry.
///
/// The cap is not about speed. `docs/decisions.md` §15.6 rule 1 wants a
/// published error bound to be a named constant sitting next to the expression
/// it bounds, and the bound on de Boor is a bound over `p + 1` terms. With no
/// cap on the degree that constant is a function of runtime data, and a test of
/// it ends up recomputing its own tolerance — which is the tuned-until-it-passed
/// tolerance this kernel exists to avoid. A cap makes it a number.
///
/// Degree 16 is bad caller input rather than a bug, so it comes back as
/// [`Error::DegreeNotSupported`] (`docs/decisions.md` §4.1). One de Boor stage
/// needs `p + 1` homogeneous points of scratch — 512 bytes of `f64` at the
/// cap — so evaluation never has to allocate.
pub const MAX_DEGREE: usize = 15;

/// The parameter values where a spline's polynomial pieces are joined.
///
/// Carries its own degree, because almost nothing about a knot vector can be
/// checked without it: how many knots the ends need, how often an interior knot
/// may repeat, and which stretch of parameter actually names a point.
///
/// There is no tolerance anywhere in here. Two knots are equal or they are not.
/// Two knots one `f64` apart make a span too narrow to see, and that is the
/// caller's data — snapping it shut would be this kernel deciding something the
/// caller did not ask it to decide.
///
/// Clamped and non-periodic only: the first and last knots each repeat
/// `degree + 1` times, so the curve starts at its first control point and ends
/// at its last. Every periodic surface mechanical CAD cares about — cylinder,
/// sphere, cone, torus — is analytic and stays analytic in this kernel.
/// Periodic knot vectors first arrive with STEP import at M5.
///
/// Serializes as `(degree, knots)` and is checked again on the way in, so a
/// corrupt file cannot hand out a knot vector that never passed [`KnotVector::new`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "(usize, Vec<f64>)", into = "(usize, Vec<f64>)")]
pub struct KnotVector {
    degree: usize,
    knots: Vec<f64>,
}

impl KnotVector {
    /// The knot vector of the given degree, if the knots describe one.
    ///
    /// Six things have to hold, and they are checked in this order: every knot
    /// is a finite number, the knots never go backwards, the first `degree + 1`
    /// are all the same, the last `degree + 1` are all the same, no interior
    /// knot appears more than `degree` times, and at least one span inside the
    /// domain has some width.
    ///
    /// The degree and the length are checked ahead of all six, because the
    /// clamping rules have nothing to say about a vector too short to hold two
    /// clamps.
    ///
    /// # The knots also have to be close enough together to subtract
    ///
    /// This is not a seventh invariant of what a knot vector *is*. It is a
    /// limit on what this kernel can evaluate, the same kind of thing as the
    /// degree cap: a de Boor coefficient is one knot difference divided by
    /// another, and if `knots[last] - knots[0]` overflows then so can the
    /// differences below it. When it does, every coefficient comes out as zero,
    /// every stage collapses to the control point below it, and
    /// [`EVALUATION_ROUNDOFF`](crate::EVALUATION_ROUNDOFF) goes on certifying
    /// the answer — its derivation assumes the subtractions did not overflow,
    /// and nothing at evaluation time checks that. A knot vector this kernel
    /// cannot evaluate on is bad caller input, so it is refused here
    /// (`docs/decisions.md` §4.1).
    ///
    /// It is checked after the first two of the six rather than with the degree
    /// cap, because a difference of knots says nothing until the knots are
    /// known to be finite and in order.
    ///
    /// # Errors
    ///
    /// [`Error::DegreeNotSupported`] for degree zero or above [`MAX_DEGREE`],
    /// [`Error::TooFewKnots`] for fewer than `2 * (degree + 1)` knots,
    /// [`Error::NotFinite`] for a knot that is not a finite number,
    /// [`Error::KnotsGoBackwards`], [`Error::KnotsTooFarApart`],
    /// [`Error::KnotVectorNotClamped`], [`Error::KnotRepeatedTooOften`] and
    /// [`Error::KnotVectorHasNoSpan`] for the rest.
    pub fn new(degree: usize, knots: Vec<f64>) -> Result<Self> {
        if !(1..=MAX_DEGREE).contains(&degree) {
            return Err(Error::DegreeNotSupported { degree });
        }
        if knots.len() < 2 * (degree + 1) {
            return Err(Error::TooFewKnots {
                degree,
                knots: knots.len(),
            });
        }
        if knots.iter().any(|knot| !knot.is_finite()) {
            return Err(Error::NotFinite { what: "knot" });
        }
        if let Some(at) = (1..knots.len()).find(|&at| knots[at - 1] > knots[at]) {
            return Err(Error::KnotsGoBackwards { at });
        }

        let last = knots.len() - 1;
        if !(knots[last] - knots[0]).is_finite() {
            return Err(Error::KnotsTooFarApart {
                lo: knots[0],
                hi: knots[last],
            });
        }

        let start_is_clamped = knots[..=degree].iter().all(|knot| *knot == knots[0]);
        let end_is_clamped = knots[last - degree..]
            .iter()
            .all(|knot| *knot == knots[last]);
        if !(start_is_clamped && end_is_clamped) {
            return Err(Error::KnotVectorNotClamped);
        }

        for &knot in &knots[degree + 1..last - degree] {
            let times = knots.iter().filter(|other| **other == knot).count();
            if times > degree {
                return Err(Error::KnotRepeatedTooOften {
                    knot,
                    times,
                    degree,
                });
            }
        }

        if knots[degree] >= knots[last - degree] {
            return Err(Error::KnotVectorHasNoSpan);
        }

        Ok(Self { degree, knots })
    }

    /// The degree of every polynomial piece: 1 for straight, 2 for conics, 3
    /// for the usual cubic.
    #[must_use]
    pub fn degree(&self) -> usize {
        self.degree
    }

    /// The knots themselves, in the order they were given.
    #[must_use]
    pub fn knots(&self) -> &[f64] {
        &self.knots
    }

    /// How many control points a spline on this knot vector needs:
    /// `knots.len() - degree - 1`.
    #[must_use]
    pub fn control_point_count(&self) -> usize {
        self.knots.len() - self.degree - 1
    }

    /// The stretch of parameter that names a point on the curve, from
    /// `knots[degree]` to `knots[knots.len() - 1 - degree]`.
    ///
    /// Knots outside it still shape the curve; they just do not name anywhere
    /// on it.
    #[must_use]
    pub fn domain(&self) -> (f64, f64) {
        (
            self.knots[self.degree],
            self.knots[self.knots.len() - 1 - self.degree],
        )
    }

    /// Whether `u` names a point on the curve.
    ///
    /// Both ends count. A parameter that is not a number, or is infinite, fails
    /// one of the two comparisons and so is not contained.
    #[must_use]
    pub fn contains(&self, u: f64) -> bool {
        let (lo, hi) = self.domain();
        lo <= u && u <= hi
    }

    /// The span `u` falls in: the index `i` with `knots[i] <= u < knots[i+1]`.
    ///
    /// At the very top of the domain there is no such index, so the answer is
    /// held to the last span of positive width instead. That span is always the
    /// one ending at `knots[knots.len() - 1 - degree]`: a knot vector where the
    /// one before it had the same value would have that value appearing
    /// `degree + 2` times, which [`KnotVector::new`] refuses.
    ///
    /// # Panics
    ///
    /// If `u` is not in [`KnotVector::contains`].
    #[must_use]
    pub fn span_containing(&self, u: f64) -> usize {
        assert!(
            self.contains(u),
            "the parameter {u} is outside the domain {:?}",
            self.domain()
        );
        let last_span = self.knots.len() - self.degree - 2;
        if u >= self.knots[last_span] {
            return last_span;
        }
        (self.degree..last_span)
            .rfind(|&span| self.knots[span] <= u)
            .expect("the domain starts at knots[degree], so a parameter inside it has a span")
    }

    /// How many times `u` appears among the knots. Zero when it is not a knot
    /// at all.
    #[must_use]
    pub fn multiplicity_of(&self, u: f64) -> usize {
        self.knots.iter().filter(|knot| **knot == u).count()
    }
}

impl TryFrom<(usize, Vec<f64>)> for KnotVector {
    type Error = Error;

    fn try_from((degree, knots): (usize, Vec<f64>)) -> Result<Self> {
        Self::new(degree, knots)
    }
}

impl From<KnotVector> for (usize, Vec<f64>) {
    fn from(knots: KnotVector) -> Self {
        (knots.degree, knots.knots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_cubic() -> KnotVector {
        KnotVector::new(
            3,
            vec![0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0],
        )
        .expect("clamped, rising, and no interior knot past multiplicity three")
    }

    #[test]
    fn a_spline_of_degree_zero_is_not_a_spline() {
        assert_eq!(
            KnotVector::new(0, vec![0.0, 1.0]),
            Err(Error::DegreeNotSupported { degree: 0 })
        );
    }

    #[test]
    fn a_degree_one_past_the_cap_is_refused() {
        let knots = vec![0.0; 2 * (MAX_DEGREE + 2)];
        assert_eq!(
            KnotVector::new(MAX_DEGREE + 1, knots),
            Err(Error::DegreeNotSupported {
                degree: MAX_DEGREE + 1
            })
        );
    }

    #[test]
    fn a_vector_too_short_to_clamp_both_ends_is_refused() {
        assert_eq!(
            KnotVector::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0]),
            Err(Error::TooFewKnots {
                degree: 2,
                knots: 5
            })
        );
    }

    #[test]
    fn a_knot_that_is_not_a_number_is_refused() {
        assert_eq!(
            KnotVector::new(1, vec![0.0, 0.0, f64::NAN, 1.0, 1.0, 1.0]),
            Err(Error::NotFinite { what: "knot" })
        );
    }

    #[test]
    fn a_knot_smaller_than_the_one_before_it_is_refused() {
        assert_eq!(
            KnotVector::new(1, vec![0.0, 0.0, 2.0, 1.0, 3.0, 3.0]),
            Err(Error::KnotsGoBackwards { at: 3 })
        );
    }

    #[test]
    fn a_vector_that_does_not_start_with_enough_copies_of_its_first_knot_is_refused() {
        assert_eq!(
            KnotVector::new(2, vec![0.0, 0.0, 0.5, 1.0, 1.0, 1.0]),
            Err(Error::KnotVectorNotClamped)
        );
    }

    #[test]
    fn a_vector_that_does_not_end_with_enough_copies_of_its_last_knot_is_refused() {
        assert_eq!(
            KnotVector::new(2, vec![0.0, 0.0, 0.0, 0.5, 1.0, 1.0]),
            Err(Error::KnotVectorNotClamped)
        );
    }

    #[test]
    fn an_interior_knot_repeated_more_often_than_the_degree_is_refused() {
        assert_eq!(
            KnotVector::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0]),
            Err(Error::KnotRepeatedTooOften {
                knot: 1.0,
                times: 3,
                degree: 2
            })
        );
    }

    /// The clamp only says the first `degree + 1` knots match each other. One
    /// more copy on top of that is an interior knot, and it is counted across
    /// the whole vector — otherwise the first control point would have a basis
    /// function that is zero everywhere.
    #[test]
    fn a_clamp_one_knot_too_long_is_caught_as_a_repeat() {
        assert_eq!(
            KnotVector::new(2, vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0]),
            Err(Error::KnotRepeatedTooOften {
                knot: 0.0,
                times: 4,
                degree: 2
            })
        );
    }

    /// A de Boor coefficient divides `(u − below)` by `(above − below)`, and
    /// with the two clamps this far apart that denominator overflows. Every
    /// share came out as zero, every stage collapsed to the control point
    /// below it, and a curve through `(0,0,0)`, `(1,0,0)`, `(2,0,0)`, `(3,0,0)`
    /// read back `(0,0,0)` everywhere where the truth at the start of the
    /// domain is `(1.5, 0, 0)`.
    ///
    /// What makes it worth refusing at construction rather than living with is
    /// the certificate: `evaluation_roundoff_at` published a bound of `4e-14`
    /// on that answer. Its derivation assumes each subtraction is the truth
    /// times `(1 + δ)`, which overflow breaks, and nothing at evaluation time
    /// can see that the hypothesis has gone. So the check is here, where the
    /// knots arrive.
    #[test]
    fn a_knot_vector_too_wide_to_subtract_is_refused() {
        assert_eq!(
            KnotVector::new(
                3,
                vec![-9e307, -9e307, -9e307, -9e307, 9e307, 9e307, 9e307, 9e307]
            ),
            Err(Error::KnotsTooFarApart {
                lo: -9e307,
                hi: 9e307
            })
        );
    }

    /// The refusal above is exactly the range `f64` cannot subtract and not a
    /// hair more. These two clamps are the widest pair whose difference is
    /// still a number, and the kernel goes on evaluating over them.
    #[test]
    fn a_knot_vector_as_wide_as_f64_can_still_subtract_is_accepted() {
        let half_the_range = f64::MAX / 2.0;
        let widest = KnotVector::new(
            1,
            vec![
                -half_the_range,
                -half_the_range,
                half_the_range,
                half_the_range,
            ],
        )
        .expect("the two ends subtract to exactly f64::MAX, which is a number");
        assert_eq!(widest.domain(), (-half_the_range, half_the_range));
    }

    #[test]
    fn a_vector_whose_every_span_is_shut_names_no_curve() {
        assert_eq!(
            KnotVector::new(2, vec![0.0; 6]),
            Err(Error::KnotVectorHasNoSpan)
        );
    }

    #[test]
    fn a_span_one_f64_wide_is_still_a_span() {
        let next_number_up = f64::from_bits(1);
        let barely = KnotVector::new(1, vec![0.0, 0.0, next_number_up, next_number_up])
            .expect("the two ends differ, so there is a span");
        assert_eq!(barely.domain(), (0.0, next_number_up));
    }

    #[test]
    fn the_domain_runs_between_the_two_clamps() {
        assert_eq!(a_cubic().domain(), (0.0, 3.0));
    }

    #[test]
    fn there_is_one_control_point_for_every_knot_past_the_degree_and_one_more() {
        assert_eq!(a_cubic().control_point_count(), 11 - 3 - 1);
    }

    #[test]
    fn a_parameter_outside_the_domain_is_not_contained() {
        let knots = a_cubic();
        assert!(knots.contains(0.0));
        assert!(knots.contains(1.5));
        assert!(knots.contains(3.0));
        assert!(!knots.contains(-0.0001));
        assert!(!knots.contains(3.0001));
        assert!(!knots.contains(f64::NAN));
        assert!(!knots.contains(f64::INFINITY));
    }

    #[test]
    fn a_parameter_inside_a_span_lands_on_that_span() {
        let knots = a_cubic();
        assert_eq!(knots.span_containing(0.0), 3);
        assert_eq!(knots.span_containing(0.5), 3);
        assert_eq!(knots.span_containing(1.0), 4);
        assert_eq!(knots.span_containing(2.0), 6);
        assert_eq!(knots.span_containing(2.5), 6);
    }

    /// The top of the domain has no span starting at it, so it is held to the
    /// last one that has any width.
    #[test]
    fn the_very_top_of_the_domain_lands_on_the_last_span_with_width() {
        let knots = a_cubic();
        assert_eq!(knots.span_containing(3.0), 6);
        assert!(knots.knots()[6] < knots.knots()[7]);
    }

    #[test]
    #[should_panic(expected = "outside the domain")]
    fn asking_for_the_span_of_a_parameter_off_the_curve_panics() {
        let _ = a_cubic().span_containing(4.0);
    }

    #[test]
    fn a_repeated_knot_is_counted_and_one_that_is_not_there_counts_zero() {
        let knots = a_cubic();
        assert_eq!(knots.multiplicity_of(0.0), 4);
        assert_eq!(knots.multiplicity_of(2.0), 2);
        assert_eq!(knots.multiplicity_of(3.0), 4);
        assert_eq!(knots.multiplicity_of(1.5), 0);
    }
}
