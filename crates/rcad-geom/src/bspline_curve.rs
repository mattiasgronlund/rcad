//! Spline curves, always rational.

use rcad_predicates::Point as PredicatePoint;
use serde::{Deserialize, Serialize};

use crate::de_boor::{
    Coefficient, EVALUATION_ROUNDOFF, after_two_passes, blended, de_boor, derived_points, divided,
    enclosed, rational_derivative, roundoff_of_a_divided_point,
};
use crate::knot_vector::{KnotVector, MAX_DEGREE};
use crate::point::{Point, Vector};
use crate::{Error, Result};

/// A curve made of polynomial pieces joined at the knots, with a weight on
/// every control point.
///
/// # One always-rational type
///
/// There is no separate non-rational variant. A weight of exactly `1.0` divides
/// out exactly, so a polynomial curve costs nothing extra to carry as a
/// rational one, and every algorithm downstream is written once.
///
/// # Homogeneous storage, affine constructors
///
/// What is stored per control point is `[w*x, w*y, w*z, w]`. That is what makes
/// "knot insertion leaves everything outside the affected span alone, bit for
/// bit" true by construction rather than by special case: Boehm's algorithm is
/// an affine combination of the stored numbers, so an untouched control point
/// is copied and not recomputed.
///
/// The price is that `w * P` rounds on the way in and the divide rounds on the
/// way back out, so [`BSplineCurve::control_point`] is not in general
/// bit-identical to the point handed to [`BSplineCurve::new`].
/// [`BSplineCurve::weight`] is, being the stored fourth number itself.
///
/// Serializes as `(knots, homogeneous net)` — the stored numbers, so the round
/// trip is exact. Writing the affine points out instead would round twice.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    try_from = "(KnotVector, Vec<[f64; 4]>)",
    into = "(KnotVector, Vec<[f64; 4]>)"
)]
pub struct BSplineCurve {
    knots: KnotVector,
    net: Vec<[f64; 4]>,
}

/// How far evaluating a refined curve can be from the true point of the curve
/// it was refined from, as a fraction of the largest homogeneous control
/// coordinate of the original net.
///
/// # Where the number comes from
///
/// Refining and then evaluating is two de Boor passes one after the other, so
/// it is the same composition a surface makes along `v` and then along `u`, and
/// it reuses the same step.
///
/// One Boehm stage replaces two stored points by `(1 − a)·P + a·Q`, with the
/// same coefficient shape and the same three roundings as one de Boor stage —
/// they share the code that does it — so [`EVALUATION_ROUNDOFF`]'s count of
/// nine roundings per stage covers an insertion stage too. A control point of
/// the refined net is at most `degree` stages deep, and `degree` is capped at
/// [`MAX_DEGREE`], so the refined net sits within `K·M` of the exact refined
/// net, where `K` is [`EVALUATION_ROUNDOFF`] and `M` is the largest magnitude
/// among the four homogeneous coordinates of the original net.
///
/// Every stage is a convex combination, so the refined points lie inside the
/// original net's convex hull and are no bigger than `M` in truth. Evaluating
/// them is a second pass starting from numbers already `K·M` off, worth
/// `K·M·(1 + K) + K·M = M·(2K + K²)`.
///
/// # The precondition, which is not optional
///
/// `M` is taken over the **whole** original net rather than the span the
/// evaluation touches. A refined control point is a blend of original ones from
/// a wider stretch than the span it ends up serving, so charging the narrower
/// figure would give a number that is not a bound.
///
/// The stage count assumes at most [`MAX_DEGREE`] insertions.
/// [`BSplineCurve::with_knot_inserted`] does one and
/// [`BSplineCurve::split_at`] does at most `degree`, so both satisfy it by
/// construction. [`BSplineCurve::bezier_segments`] does not: a curve with many
/// interior knots blends deeper than the cap, and this constant says nothing
/// about it.
pub const REFINEMENT_ROUNDOFF: f64 = after_two_passes(EVALUATION_ROUNDOFF);

impl BSplineCurve {
    /// The curve through the given control net.
    ///
    /// `control_points` and `weights` both have
    /// [`KnotVector::control_point_count`] entries, and every weight is a
    /// normal number greater than zero.
    ///
    /// # Why weights have to be normal, not merely positive
    ///
    /// A de Boor stage blends two weights as `(1 − a)·w₀ + a·w₁`. Down among the
    /// subnormals both products can round away to nothing at once — two weights
    /// of `5e-324` at the middle of a span give exactly zero — and a curve that
    /// evaluates at four parameters and dies at the fifth is not one this kernel
    /// should have accepted.
    ///
    /// Normal weights rule it out with room to spare. Rounding `(1 − a)·w₀` to
    /// zero needs `1 − a` below `2⁻⁵³`, and rounding `a·w₁` to zero needs `a`
    /// below `2⁻⁵³`; but `a` that small makes `1 − a` round to exactly `1`. The
    /// margin is a factor of `2⁵³`, so the few ulps a stage can drift are
    /// nowhere near it.
    ///
    /// A rational spline depends on its weights only through their ratios, so
    /// multiplying every weight by a power of two hands back the identical
    /// curve with no bits lost — but only while the weights span less than
    /// `f64`'s normal range. Weights of `5e-324` and `1e300` have no power of
    /// two that brings both into that range: `2⁵²` is needed to make `5e-324`
    /// normal, and `1e300 · 2⁵²` is past `f64::MAX`.
    ///
    /// # Errors
    ///
    /// [`Error::WrongNumberOfControlPoints`] if either list is the wrong
    /// length, [`Error::NotFinite`] if a control point has a coordinate that is
    /// not a finite number or if weighting it overflows,
    /// [`Error::WeightNotPositive`] for a weight that is zero, negative, or not
    /// a finite number, and [`Error::WeightTooSmallToBlend`] for a positive
    /// weight among the subnormals.
    pub fn new(knots: KnotVector, control_points: &[Point], weights: &[f64]) -> Result<Self> {
        let expected = knots.control_point_count();
        for given in [control_points.len(), weights.len()] {
            if given != expected {
                return Err(Error::WrongNumberOfControlPoints { expected, given });
            }
        }
        if control_points.iter().any(|point| !point.is_finite()) {
            return Err(Error::NotFinite {
                what: "spline control point",
            });
        }

        let net = control_points
            .iter()
            .zip(weights)
            .map(|(point, &weight)| weigh(*point, weight))
            .collect();
        Self::from_homogeneous_net(knots, net)
    }

    /// The knot vector the curve is built on.
    #[must_use]
    pub fn knots(&self) -> &KnotVector {
        &self.knots
    }

    /// The degree of every polynomial piece.
    #[must_use]
    pub fn degree(&self) -> usize {
        self.knots.degree()
    }

    /// How many control points the curve has.
    #[must_use]
    pub fn control_point_count(&self) -> usize {
        self.net.len()
    }

    /// The stretch of parameter that names a point on the curve.
    #[must_use]
    pub fn domain(&self) -> (f64, f64) {
        self.knots.domain()
    }

    /// The `i`th control point, with its weight divided back out.
    ///
    /// Not in general bit-identical to what was handed to
    /// [`BSplineCurve::new`] — see the note on [`BSplineCurve`]. It is exact
    /// when the weight is `1.0`, because dividing by one rounds nothing.
    ///
    /// # Panics
    ///
    /// If `i` is past the last control point.
    #[must_use]
    pub fn control_point(&self, i: usize) -> Point {
        let [x, y, z, weight] = self.homogeneous_control_point(i);
        Point::new(x / weight, y / weight, z / weight)
    }

    /// The `i`th weight, exactly as it was handed over.
    ///
    /// # Panics
    ///
    /// If `i` is past the last control point.
    #[must_use]
    pub fn weight(&self, i: usize) -> f64 {
        self.homogeneous_control_point(i)[3]
    }

    /// The `i`th control point as it is stored: `[w*x, w*y, w*z, w]`.
    ///
    /// # Panics
    ///
    /// If `i` is past the last control point.
    #[must_use]
    pub fn homogeneous_control_point(&self, i: usize) -> [f64; 4] {
        self.net[i]
    }

    /// The point on the curve at `u`.
    ///
    /// Ordinary `f64`. Nothing may branch on it near a decision — see the crate
    /// doc's "three kinds of answer". [`BSplineCurve::evaluation_roundoff_at`]
    /// says how far off it can be.
    ///
    /// # Errors
    ///
    /// [`Error::ParameterOutsideDomain`] if `u` is not a finite number inside
    /// [`BSplineCurve::domain`]. That is the only way this can fail.
    ///
    /// # Panics
    ///
    /// If the weight at `u` does not come out above zero — which means the
    /// stored net is not what construction promised. A broken invariant is a
    /// crash and a corrupt point is worse than a crash (`docs/decisions.md`
    /// §4.1). This is a deliberate departure from roadmap §4.5's blanket "never
    /// panic", which is about bad caller input; bad caller input here is the
    /// `Err` above.
    ///
    /// This used to say that a positive stored net could not produce it,
    /// because de Boor mixes weights with non-negative coefficients that sum to
    /// one. Positive was not enough: two subnormal weights blend to exactly
    /// zero at the middle of a span, and the constructor accepted them, so the
    /// panic fired on ordinary caller input. [`BSplineCurve::new`] now requires
    /// weights to be **normal**, which does rule it out — see the argument and
    /// its `2⁵³` margin there.
    pub fn evaluate(&self, u: f64) -> Result<Point> {
        Ok(divided(self.homogeneous_evaluate(u)?))
    }

    /// The first derivative of the curve in `u`.
    ///
    /// The derivative of a B-spline is a B-spline of degree `p − 1` on a
    /// derived control net, so this is the same de Boor machinery through
    /// another door rather than a separate part of the kernel.
    ///
    /// The curve is rational, and that is where the textbook trips: with
    /// `C = A / w`, the derivative is `(A' − w'·C) / w`. Differentiating the
    /// affine control points instead gives `A'` alone, which is only the same
    /// thing when every weight is equal.
    ///
    /// # Errors
    ///
    /// [`Error::ParameterOutsideDomain`], as [`BSplineCurve::evaluate`].
    ///
    /// [`Error::NotFinite`] if the true tangent is itself past what `f64` can
    /// hold. A knot span one `f64` step wide under ordinary coordinates does
    /// that, and [`KnotVector::new`] accepts such a span on purpose — two knots
    /// a step apart are the caller's data, not something to snap. So this is a
    /// refusal about the numbers asked for, not about the arithmetic used, and
    /// §4.1 makes it an `Err`. Handing back the `NaN` would be the silent
    /// corrupt value §4.1 ranks below a crash.
    ///
    /// The derived net no longer loses a representable answer: where its
    /// subtraction would run off the end of the range, both coordinates are
    /// halved first and the factor comes back in the scale — see the note on
    /// `de_boor::derived_points`.
    ///
    /// `rational_derivative`'s `(A' − w'·C) / w` has its own two overflow cases,
    /// both covered and both tested — see `de_boor::one_axis_of_the_rate`.
    ///
    /// **None of this proves every representable tangent comes back.** An
    /// earlier version of this sentence came close to claiming it, and a curve
    /// whose true tangent is about `-1e307` was refused behind the claim. What
    /// holds is narrower: each fix enlarges the set of inputs that get an
    /// answer, the cases named are tested, and whatever is left refuses rather
    /// than returning a `NaN`.
    ///
    /// # Panics
    ///
    /// For the same broken invariant [`BSplineCurve::evaluate`] panics on.
    pub fn tangent(&self, u: f64) -> Result<Vector> {
        let (value, rate) = self.homogeneous_value_and_derivative(u)?;
        rational_derivative(value, rate)
    }

    /// A box the true point at `u` is somewhere inside, ready for the predicate
    /// cascade.
    ///
    /// The same de Boor recursion as [`BSplineCurve::evaluate`], run over
    /// intervals instead of `f64`, with the perspective divide done the same
    /// way at the end. **This is the enclosure, and it is the only one.**
    /// [`EVALUATION_ROUNDOFF`] is tested against it and is not a second source:
    /// the two are never intersected. An empty intersection would be a free
    /// cross-check, and it would make a published bound load-bearing at
    /// runtime, which is exactly where `docs/decisions.md` §12.2's mistake
    /// lives. The cross-check is a test instead.
    ///
    /// Comes back [`PredicatePoint::Exact`] when all three sides have zero
    /// width, and [`PredicatePoint::Constructed`] otherwise. Zero width says
    /// the true coordinate *is* that `f64`, so exact is sound — and it is
    /// strictly stronger, because only an exact point runs level 3 of the
    /// cascade. That is what makes a degree-1 curve through exact control
    /// points, read at an exact parameter, decidable at all.
    ///
    /// # Errors
    ///
    /// [`Error::ParameterOutsideDomain`] if `u` is not a finite number inside
    /// [`BSplineCurve::domain`] — the same one failure
    /// [`BSplineCurve::evaluate`] has, and no other.
    ///
    /// # Panics
    ///
    /// For the same broken invariant [`BSplineCurve::evaluate`] panics on. A
    /// weight whose *enclosure* holds zero is not that, and does not panic; see
    /// the note on the divide in `de_boor.rs`.
    pub fn evaluate_with_enclosure(&self, u: f64) -> Result<PredicatePoint> {
        Ok(enclosed(self.homogeneous_evaluate(u)?))
    }

    /// How far [`BSplineCurve::evaluate`] at `u` can be from the true point,
    /// per coordinate.
    ///
    /// `None` when [`EVALUATION_ROUNDOFF`]'s precondition does not hold —
    /// something in the touched net is not a normal number, or the weight at
    /// `u` is not. A bound whose model does not hold is worse than no bound,
    /// because a caller believes it (`docs/decisions.md` §12.2).
    ///
    /// # Errors
    ///
    /// [`Error::ParameterOutsideDomain`], as [`BSplineCurve::evaluate`].
    pub fn evaluation_roundoff_at(&self, u: f64) -> Result<Option<f64>> {
        let span = self.span_of(u)?;
        let homogeneous = self.homogeneous_evaluate(u)?;
        Ok(roundoff_of_a_divided_point(
            EVALUATION_ROUNDOFF,
            self.largest_active_coordinates(span),
            homogeneous,
        ))
    }

    /// The same curve with one more copy of `u` in its knot vector.
    ///
    /// Boehm's algorithm. The span `u` falls in reaches `degree + 1` control
    /// points; they are replaced by blends of consecutive pairs, and **every
    /// other control point is copied**. The curve is the same shape as before —
    /// what changes is how it is written down, and there is one more control
    /// point to write it with.
    ///
    /// # Why this runs on the stored homogeneous net
    ///
    /// A blend here is an affine combination of the stored numbers
    /// `[w*x, w*y, w*z, w]`, so a control point outside the affected span is
    /// copied rather than recomputed and comes out bit-identical in all four
    /// coordinates — by construction, not by a special case that checks for it.
    /// Going through affine points and weights instead would divide each weight
    /// out and multiply it back in, rounding twice and throwing that property
    /// away. It is the whole reason this crate stores the weighted net.
    ///
    /// # Errors
    ///
    /// [`Error::ParameterOutsideDomain`] if `u` is not a finite number inside
    /// [`BSplineCurve::domain`], the same refusal
    /// [`BSplineCurve::evaluate`] makes. [`Error::CannotRepeatKnotAgain`] if
    /// `u` already appears `degree` times or more — one more copy would leave a
    /// control point with no basis function of its own, and
    /// [`KnotVector::new`] would refuse the result. Both ends of the domain
    /// appear `degree + 1` times, so inserting at an end is refused this way.
    ///
    /// [`Error::WeightNotPositive`] if a blend of two weights underflows to
    /// zero. That needs weights down in the subnormals, and a curve carrying
    /// them cannot be refined into one this kernel would accept.
    pub fn with_knot_inserted(&self, u: f64) -> Result<Self> {
        let span = self.span_of(u)?;
        let degree = self.degree();
        let times = self.knots.multiplicity_of(u);
        if times >= degree {
            return Err(Error::CannotRepeatKnotAgain {
                knot: u,
                times,
                degree,
            });
        }

        let knots = self.knots.knots();
        let mut grown = Vec::with_capacity(knots.len() + 1);
        grown.extend_from_slice(&knots[..=span]);
        grown.push(u);
        grown.extend_from_slice(&knots[span + 1..]);

        let mut net = Vec::with_capacity(self.net.len() + 1);
        net.extend_from_slice(&self.net[..=span - degree]);
        for i in span - degree + 1..=span - times {
            let below = knots[i];
            let above = knots[i + degree];
            assert!(
                below < above,
                "the insertion span from {below} to {above} is shut, which the knot vector's own \
                 invariants rule out"
            );
            let share = (u - below) / (above - below);
            net.push(blended(1.0 - share, self.net[i - 1], share, self.net[i]));
        }
        net.extend_from_slice(&self.net[span - times..]);

        let grown = KnotVector::new(degree, grown).expect(
            "one more copy of an interior knot still below the degree keeps every invariant",
        );
        Self::from_homogeneous_net(grown, net)
    }

    /// The two halves of the curve on either side of `u`, each a curve in its
    /// own right.
    ///
    /// `u` is raised to a multiplicity of `degree` by repeated insertion and
    /// the curve is cut there. At that multiplicity the curve passes exactly
    /// through one control point, and the two halves share it — the same four
    /// stored numbers, copied into both, so the seam matches bit for bit rather
    /// than to within something.
    ///
    /// # Errors
    ///
    /// [`Error::ParameterOutsideDomain`] if `u` is not a finite number inside
    /// [`BSplineCurve::domain`].
    ///
    /// [`Error::NothingOnOneSideOfTheSplit`] at either end of the domain. An
    /// empty half is not a curve: it would need a knot vector whose every span
    /// is shut, which [`KnotVector::new`] refuses and should. Returning the
    /// whole curve and a nothing, or the whole curve twice, would both be
    /// answers to a question the caller did not ask.
    ///
    /// [`Error::WeightNotPositive`], for the reason
    /// [`BSplineCurve::with_knot_inserted`] gives.
    pub fn split_at(&self, u: f64) -> Result<(Self, Self)> {
        self.span_of(u)?;
        let (lo, hi) = self.domain();
        if u == lo || u == hi {
            return Err(Error::NothingOnOneSideOfTheSplit {
                parameter: u,
                lo,
                hi,
            });
        }

        let degree = self.degree();
        let mut raised = self.clone();
        while raised.knots.multiplicity_of(u) < degree {
            raised = raised.with_knot_inserted(u)?;
        }
        Ok(raised.cut_at(u))
    }

    /// One Bezier curve per span of positive width, in order.
    ///
    /// Every interior knot is raised to a multiplicity of `degree`, which
    /// leaves the net as one run of `degree + 1` control points per span with
    /// neighbouring runs sharing an end. Each run is handed out over the knot
    /// vector `[a, …, a, b, …, b]`, so the last control point of one segment
    /// and the first of the next are the same four stored numbers and the
    /// segments join bit for bit.
    ///
    /// A curve that is already a single Bezier has no interior knots, so
    /// nothing is inserted and the one segment that comes back is this curve
    /// unchanged.
    ///
    /// Roadmap §6.2's Bezier clipping is what this is for.
    ///
    /// # Panics
    ///
    /// If a knot the curve already carries cannot be inserted into it. Every
    /// interior knot is inside the domain and appears fewer than `degree` times
    /// until this raises it, so that is a broken invariant rather than bad
    /// caller input.
    #[must_use]
    pub fn bezier_segments(&self) -> Vec<Self> {
        let degree = self.degree();
        let interior = self.distinct_interior_knots();

        let mut refined = self.clone();
        for &knot in &interior {
            while refined.knots.multiplicity_of(knot) < degree {
                refined = refined
                    .with_knot_inserted(knot)
                    .expect("an interior knot below full multiplicity can always take one more");
            }
        }

        let (lo, hi) = self.domain();
        let mut breaks = Vec::with_capacity(interior.len() + 2);
        breaks.push(lo);
        breaks.extend_from_slice(&interior);
        breaks.push(hi);

        assert_eq!(
            refined.net.len(),
            degree + 1 + interior.len() * degree,
            "a fully refined curve holds one run of {} control points per span, sharing ends",
            degree + 1
        );

        breaks
            .windows(2)
            .enumerate()
            .map(|(segment, ends)| {
                let mut knots = vec![ends[0]; degree + 1];
                knots.extend(core::iter::repeat_n(ends[1], degree + 1));
                let start = segment * degree;
                Self::a_piece(degree, knots, &refined.net[start..=start + degree])
            })
            .collect()
    }

    /// How far [`BSplineCurve::evaluate`] on this curve can be from the true
    /// point of the curve it was refined from, per coordinate.
    ///
    /// `self` is a curve that came out of [`BSplineCurve::with_knot_inserted`]
    /// or [`BSplineCurve::split_at`] and `original` is what it came from. See
    /// [`REFINEMENT_ROUNDOFF`] for what that costs and for the one case it does
    /// not cover.
    ///
    /// `None` when the precondition does not hold, exactly as
    /// [`BSplineCurve::evaluation_roundoff_at`] returns `None`.
    ///
    /// # Errors
    ///
    /// [`Error::ParameterOutsideDomain`] if `u` is outside **this** curve's
    /// domain — a half of a split covers only half of the original.
    pub fn refinement_roundoff_at(&self, original: &Self, u: f64) -> Result<Option<f64>> {
        let homogeneous = self.homogeneous_evaluate(u)?;
        Ok(roundoff_of_a_divided_point(
            REFINEMENT_ROUNDOFF,
            largest_coordinates(&original.net),
            homogeneous,
        ))
    }

    /// The point at `u` before the weight is divided back out, in whichever
    /// arithmetic the caller asked for.
    ///
    /// One body serves [`BSplineCurve::evaluate`] and
    /// [`BSplineCurve::evaluate_with_enclosure`], so "the enclosure runs the
    /// same recursion" is true of the same code and not of two copies that
    /// agree today.
    ///
    /// The fourth coordinate is `Σ Nᵢ,ₚ(u) · wᵢ`, which is what the
    /// partition-of-unity oracle reads.
    pub(crate) fn homogeneous_evaluate<C: Coefficient>(&self, u: f64) -> Result<[C; 4]> {
        let span = self.span_of(u)?;
        let degree = self.degree();
        let mut active = self.active_net(span);
        Ok(de_boor(
            degree,
            self.knots.knots(),
            span,
            u,
            &mut active[..=degree],
        ))
    }

    /// The homogeneous point and the homogeneous derivative at `u`.
    ///
    /// The derived net is evaluated as a bare slice on the knots with both ends
    /// dropped. It must not go through [`KnotVector`]: a degree-1 curve derives
    /// to degree 0, and `KnotVector::new` refuses degree zero.
    fn homogeneous_value_and_derivative(&self, u: f64) -> Result<([f64; 4], [f64; 4])> {
        let span = self.span_of(u)?;
        let degree = self.degree();
        let knots = self.knots.knots();
        let mut active = self.active_net(span);

        let mut derived = [[0.0f64; 4]; MAX_DEGREE + 1];
        derived_points(
            degree,
            knots,
            span,
            &active[..=degree],
            &mut derived[..degree],
        );
        let inner = &knots[1..knots.len() - 1];
        let rate = de_boor(degree - 1, inner, span - 1, u, &mut derived[..degree]);

        let value = de_boor(degree, knots, span, u, &mut active[..=degree]);
        Ok((value, rate))
    }

    /// The span `u` falls in, or the one failure evaluation has.
    fn span_of(&self, u: f64) -> Result<usize> {
        if !self.knots.contains(u) {
            let (lo, hi) = self.domain();
            return Err(Error::ParameterOutsideDomain {
                parameter: u,
                lo,
                hi,
            });
        }
        Ok(self.knots.span_containing(u))
    }

    /// The `degree + 1` control points the span reaches, ready for de Boor.
    fn active_net<C: Coefficient>(&self, span: usize) -> [[C; 4]; MAX_DEGREE + 1] {
        let degree = self.degree();
        let mut active = [[C::exactly(0.0); 4]; MAX_DEGREE + 1];
        for (j, point) in active[..=degree].iter_mut().enumerate() {
            *point = self.net[j + span - degree].map(C::exactly);
        }
        active
    }

    /// The biggest magnitude in each of the four homogeneous coordinates,
    /// across the control points the span reaches.
    fn largest_active_coordinates(&self, span: usize) -> [f64; 4] {
        largest_coordinates(&self.net[span - self.degree()..=span])
    }

    /// The two halves of a curve whose seam already appears `degree` times.
    ///
    /// The seam's `degree` copies sit at `first ..= first + degree - 1`. The
    /// lower half keeps the knots before them and clamps with `degree + 1`
    /// copies of its own; the upper half opens with `degree + 1` copies and
    /// keeps the knots after them. Control point `first - 1` goes to both, and
    /// it is the point the curve passes exactly through.
    fn cut_at(&self, seam: f64) -> (Self, Self) {
        let degree = self.degree();
        let knots = self.knots.knots();
        let first = knots
            .iter()
            .position(|knot| *knot == seam)
            .expect("the seam was raised to a multiplicity of degree before the cut");

        let mut below = knots[..first].to_vec();
        below.extend(core::iter::repeat_n(seam, degree + 1));
        let mut above = vec![seam; degree + 1];
        above.extend_from_slice(&knots[first + degree..]);

        (
            Self::a_piece(degree, below, &self.net[..first]),
            Self::a_piece(degree, above, &self.net[first - 1..]),
        )
    }

    /// The interior knot values, each once, in order.
    fn distinct_interior_knots(&self) -> Vec<f64> {
        let degree = self.degree();
        let knots = self.knots.knots();
        let mut distinct: Vec<f64> = Vec::new();
        for &knot in &knots[degree + 1..knots.len() - degree - 1] {
            if distinct.last() != Some(&knot) {
                distinct.push(knot);
            }
        }
        distinct
    }

    /// A stretch cut out of a refined curve, checked the way any other curve is.
    ///
    /// # Panics
    ///
    /// If the kernel would not accept it. Every piece cut here is clamped at
    /// both ends by construction, carries knots that already rose, repeats no
    /// interior knot more often than it did in the curve it came from, and has
    /// a span of positive width because the cut is never at an end of the
    /// domain. Its control points are copied from a net whose weights were
    /// already checked. A refusal is a broken invariant, not bad caller input.
    fn a_piece(degree: usize, knots: Vec<f64>, net: &[[f64; 4]]) -> Self {
        let knots = KnotVector::new(degree, knots)
            .expect("a piece of a refined curve is clamped, rising, and has a span of its own");
        Self::from_homogeneous_net(knots, net.to_vec())
            .expect("the piece carries control points the whole curve already had")
    }

    /// The curve from a net that is already weighted, checking everything a
    /// stored net has to satisfy.
    ///
    /// This is the way in for `serde`, which must not go through
    /// [`BSplineCurve::new`]: dividing the stored net back to affine points and
    /// weighting them again would round twice and hand back a different curve.
    fn from_homogeneous_net(knots: KnotVector, net: Vec<[f64; 4]>) -> Result<Self> {
        let expected = knots.control_point_count();
        if net.len() != expected {
            return Err(Error::WrongNumberOfControlPoints {
                expected,
                given: net.len(),
            });
        }
        for &[x, y, z, weight] in &net {
            if !(weight.is_finite() && weight > 0.0) {
                return Err(Error::WeightNotPositive { weight });
            }
            if weight < f64::MIN_POSITIVE {
                return Err(Error::WeightTooSmallToBlend { weight });
            }
            if !(x.is_finite() && y.is_finite() && z.is_finite()) {
                return Err(Error::NotFinite {
                    what: "weighted spline control point",
                });
            }
        }
        Ok(Self { knots, net })
    }
}

/// The biggest magnitude in each of the four homogeneous coordinates, across
/// the given control points.
fn largest_coordinates(net: &[[f64; 4]]) -> [f64; 4] {
    let mut largest = [0.0f64; 4];
    for point in net {
        for (biggest, coordinate) in largest.iter_mut().zip(point) {
            *biggest = biggest.max(coordinate.abs());
        }
    }
    largest
}

/// A control point moved into homogeneous coordinates.
pub(crate) fn weigh(point: Point, weight: f64) -> [f64; 4] {
    let [x, y, z] = point.coordinates();
    [weight * x, weight * y, weight * z, weight]
}

impl TryFrom<(KnotVector, Vec<[f64; 4]>)> for BSplineCurve {
    type Error = Error;

    fn try_from((knots, net): (KnotVector, Vec<[f64; 4]>)) -> Result<Self> {
        Self::from_homogeneous_net(knots, net)
    }
}

impl From<BSplineCurve> for (KnotVector, Vec<[f64; 4]>) {
    fn from(curve: BSplineCurve) -> Self {
        (curve.knots, curve.net)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn three_points() -> Vec<Point> {
        vec![
            Point::new(1.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
            Point::new(0.0, 1.0, 0.0),
        ]
    }

    fn a_quadratic() -> KnotVector {
        KnotVector::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]).expect("a single Bezier span")
    }

    #[test]
    fn a_curve_carries_its_knot_vectors_degree_and_domain() {
        let curve = BSplineCurve::new(a_quadratic(), &three_points(), &[1.0, 1.0, 1.0]).unwrap();
        assert_eq!(curve.degree(), 2);
        assert_eq!(curve.control_point_count(), 3);
        assert_eq!(curve.domain(), (0.0, 1.0));
        assert_eq!(curve.knots().knots(), &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn a_net_of_the_wrong_length_is_refused() {
        assert_eq!(
            BSplineCurve::new(a_quadratic(), &three_points()[..2], &[1.0, 1.0]),
            Err(Error::WrongNumberOfControlPoints {
                expected: 3,
                given: 2
            })
        );
    }

    #[test]
    fn a_weight_for_every_control_point_but_one_is_refused() {
        assert_eq!(
            BSplineCurve::new(a_quadratic(), &three_points(), &[1.0, 1.0]),
            Err(Error::WrongNumberOfControlPoints {
                expected: 3,
                given: 2
            })
        );
    }

    #[test]
    fn a_control_point_that_is_not_a_number_is_refused() {
        let mut points = three_points();
        points[1] = Point::new(f64::NAN, 0.0, 0.0);
        assert_eq!(
            BSplineCurve::new(a_quadratic(), &points, &[1.0, 1.0, 1.0]),
            Err(Error::NotFinite {
                what: "spline control point"
            })
        );
    }

    #[test]
    fn a_weight_of_zero_leaves_nothing_to_divide_by() {
        assert_eq!(
            BSplineCurve::new(a_quadratic(), &three_points(), &[1.0, 0.0, 1.0]),
            Err(Error::WeightNotPositive { weight: 0.0 })
        );
    }

    #[test]
    fn a_negative_weight_is_refused() {
        assert_eq!(
            BSplineCurve::new(a_quadratic(), &three_points(), &[1.0, -2.0, 1.0]),
            Err(Error::WeightNotPositive { weight: -2.0 })
        );
    }

    /// The refused weight is compared bit for bit, because a `NaN` handed back
    /// as a `NaN` does not compare equal to itself.
    #[test]
    fn a_weight_that_is_not_a_number_is_refused() {
        for weight in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            match BSplineCurve::new(a_quadratic(), &three_points(), &[1.0, weight, 1.0]) {
                Err(Error::WeightNotPositive { weight: refused }) => {
                    assert_eq!(refused.to_bits(), weight.to_bits());
                }
                other => panic!("a weight of {weight} cannot be divided back out: {other:?}"),
            }
        }
    }

    /// A perfectly good weight and a perfectly good point whose product is not
    /// a number any more. Storing it would give a curve that cannot be
    /// evaluated, so it is refused at the door.
    #[test]
    fn a_control_point_that_overflows_when_it_is_weighted_is_refused() {
        let mut points = three_points();
        points[1] = Point::new(1e300, 0.0, 0.0);
        assert_eq!(
            BSplineCurve::new(a_quadratic(), &points, &[1.0, 1e300, 1.0]),
            Err(Error::NotFinite {
                what: "weighted spline control point"
            })
        );
    }

    #[test]
    fn a_control_point_with_weight_one_comes_back_exactly_as_it_went_in() {
        let points = vec![
            Point::new(0.1, -0.2, 0.3),
            Point::new(1e-7, 12_345.678, -9.0),
            Point::new(0.0, 0.0, 0.0),
        ];
        let curve = BSplineCurve::new(a_quadratic(), &points, &[1.0, 1.0, 1.0]).unwrap();
        for (i, point) in points.iter().enumerate() {
            assert_eq!(curve.control_point(i), *point);
        }
    }

    /// The weight is stored as it arrived, so it survives whatever the
    /// weighting did to the coordinates.
    #[test]
    fn a_weight_survives_even_when_its_control_point_does_not_come_back() {
        let points = vec![
            Point::new(0.1, 0.0, 0.0),
            Point::new(0.0, 0.0, 0.0),
            Point::new(0.0, 0.0, 0.0),
        ];
        let weights = [3.0, 7.0, 1.0];
        let curve = BSplineCurve::new(a_quadratic(), &points, &weights).unwrap();

        assert_ne!(
            curve.control_point(0),
            points[0],
            "3.0 * 0.1 / 3.0 is not 0.1, which is the point of this test"
        );
        for (i, weight) in weights.iter().enumerate() {
            assert_eq!(curve.weight(i), *weight);
        }
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn asking_for_a_control_point_that_is_not_there_panics() {
        let curve = BSplineCurve::new(a_quadratic(), &three_points(), &[1.0, 1.0, 1.0]).unwrap();
        let _ = curve.control_point(3);
    }

    /// Degrees one to five, uniform spans and repeated interior knots, so the
    /// basis-function oracles below are not all looking at the same shape.
    fn a_spread_of_knot_vectors() -> Vec<KnotVector> {
        let sound = "clamped at both ends, rising, and no interior repeat past the degree";
        vec![
            KnotVector::new(1, vec![0.0, 0.0, 1.0, 2.0, 3.0, 3.0]).expect(sound),
            KnotVector::new(2, vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0]).expect(sound),
            KnotVector::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 2.0]).expect(sound),
            KnotVector::new(
                3,
                vec![0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0],
            )
            .expect(sound),
            KnotVector::new(
                4,
                vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0, 3.0, 3.0],
            )
            .expect(sound),
            KnotVector::new(
                5,
                vec![
                    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0,
                ],
            )
            .expect(sound),
        ]
    }

    /// Thirty-three evenly spaced parameters plus every knot, so the knots
    /// themselves — where the coefficients go to zero and one — are covered.
    fn parameters_across(knots: &KnotVector) -> Vec<f64> {
        let (lo, hi) = knots.domain();
        let mut all: Vec<f64> = (0..=32)
            .map(|step| lo + (hi - lo) * f64::from(step) / 32.0)
            .collect();
        all.extend(knots.knots());
        all.retain(|u| knots.contains(*u));
        all
    }

    /// Every weight one and every control point the origin, so the fourth
    /// homogeneous coordinate is `Σ Nᵢ,ₚ(u)` and nothing else.
    ///
    /// The sum has `p + 1` terms and `p` is capped, so the bound is
    /// [`EVALUATION_ROUNDOFF`] with the largest touched coordinate equal to
    /// one. It is derived, not tuned.
    #[test]
    fn the_basis_functions_at_any_parameter_add_up_to_one() {
        for knots in a_spread_of_knot_vectors() {
            let origins = vec![Point::origin(); knots.control_point_count()];
            let weights = vec![1.0; origins.len()];
            let curve = BSplineCurve::new(knots.clone(), &origins, &weights).unwrap();

            for u in parameters_across(&knots) {
                let total = curve.homogeneous_evaluate::<f64>(u).unwrap()[3];
                assert!(
                    (total - 1.0).abs() <= EVALUATION_ROUNDOFF,
                    "degree {} at {u}: the basis functions add up to {total}",
                    knots.degree()
                );
            }
        }
    }

    /// Read off the evaluator that actually ships rather than a second
    /// Cox–de Boor implementation: one author and one set of conventions makes
    /// a misread convention appear in both (`docs/decisions.md` §14.6). A net
    /// whose `i`th control point is `(1, 0, 0)` and whose others are the origin,
    /// all weights one, has `Nᵢ,ₚ(u)` as its first homogeneous coordinate.
    ///
    /// Zero is a hard floor rather than a bounded one: every number the
    /// recursion multiplies or adds here is non-negative, so nothing can come
    /// out below it. The ceiling needs the bound, because the sum of all of
    /// them is only one to within [`EVALUATION_ROUNDOFF`].
    #[test]
    fn no_basis_function_is_ever_negative_or_above_one() {
        for knots in a_spread_of_knot_vectors() {
            let count = knots.control_point_count();
            let weights = vec![1.0; count];
            for i in 0..count {
                let mut points = vec![Point::origin(); count];
                points[i] = Point::new(1.0, 0.0, 0.0);
                let curve = BSplineCurve::new(knots.clone(), &points, &weights).unwrap();

                for u in parameters_across(&knots) {
                    let basis = curve.homogeneous_evaluate(u).unwrap()[0];
                    assert!(
                        (0.0..=1.0 + EVALUATION_ROUNDOFF).contains(&basis),
                        "degree {}, control point {i} at {u}: basis function {basis}",
                        knots.degree()
                    );
                }
            }
        }
    }

    /// At the bottom of the domain every coefficient is exactly zero and at the
    /// top exactly one, so the ends are not approximately the end control
    /// points — they are those points, bit for bit.
    #[test]
    fn a_clamped_curve_starts_exactly_at_its_first_control_point_and_ends_at_its_last() {
        for knots in a_spread_of_knot_vectors() {
            let count = knots.control_point_count();
            let points: Vec<Point> = (0..count)
                .map(|i| Point::new(i as f64 * 0.1, 7.25 - i as f64, 1.0 / (i as f64 + 3.0)))
                .collect();
            let weights: Vec<f64> = (0..count).map(|i| 1.0 + i as f64 * 0.5).collect();
            let curve = BSplineCurve::new(knots.clone(), &points, &weights).unwrap();

            let (lo, hi) = curve.domain();
            assert_eq!(curve.evaluate(lo).unwrap(), curve.control_point(0));
            assert_eq!(curve.evaluate(hi).unwrap(), curve.control_point(count - 1));
        }
    }

    /// Knots `[0, 0, 1, 2, 2]` and dyadic control points, read at the middle of
    /// each span: the answer is the exact midpoint of the two points the
    /// parameter sits between, with nothing to round.
    fn a_straight_run() -> BSplineCurve {
        let knots = KnotVector::new(1, vec![0.0, 0.0, 1.0, 2.0, 2.0]).expect("two straight spans");
        let corners = [
            Point::new(0.0, 0.0, 0.0),
            Point::new(2.0, 4.0, 8.0),
            Point::new(6.0, 0.0, -4.0),
        ];
        BSplineCurve::new(knots, &corners, &[1.0, 1.0, 1.0]).expect("three points, three weights")
    }

    #[test]
    fn a_degree_one_curve_is_exactly_the_line_between_the_two_points_it_sits_between() {
        let curve = a_straight_run();
        assert_eq!(curve.evaluate(0.5).unwrap(), Point::new(1.0, 2.0, 4.0));
        assert_eq!(curve.evaluate(1.5).unwrap(), Point::new(4.0, 2.0, 2.0));
    }

    /// The derived net of a degree-1 curve has degree 0, which `KnotVector::new`
    /// refuses — so the derivative must never build one. It is constant along
    /// each span and steps at the knot.
    #[test]
    fn a_degree_one_curve_has_one_exact_tangent_per_span() {
        let curve = a_straight_run();
        for u in [0.0, 0.25, 0.5, 0.75] {
            assert_eq!(curve.tangent(u).unwrap(), Vector::new(2.0, 4.0, 8.0));
        }
        for u in [1.0, 1.5, 2.0] {
            assert_eq!(curve.tangent(u).unwrap(), Vector::new(4.0, -4.0, -12.0));
        }
    }

    /// Control points `(1,0)`, `(1,1)`, `(0,1)` with weights `1, 1, 2` on
    /// `[0,0,0,1,1,1]` is the quarter of the unit circle, exactly:
    /// `C(u) = ((1−u²)/(1+u²), 2u/(1+u²))`.
    fn the_quarter_circle() -> BSplineCurve {
        BSplineCurve::new(a_quadratic(), &three_points(), &[1.0, 1.0, 2.0])
            .expect("three points, three positive weights")
    }

    /// `C' = (A' − w'·C) / w`. Differentiating the affine control points gives
    /// `A' / w`, and at the top of this arc that is `(−1, 1, 0)` where the true
    /// tangent is `(−1, 0, 0)` — so a missing `w'·C` term cannot hide here.
    /// Both ends of this arc are dyadic, so the assertion needs no bound.
    #[test]
    fn the_tangent_of_a_weighted_curve_is_not_the_derivative_of_its_numerator() {
        let arc = the_quarter_circle();
        assert_eq!(arc.tangent(0.0).unwrap(), Vector::new(0.0, 2.0, 0.0));
        assert_eq!(arc.tangent(1.0).unwrap(), Vector::new(-1.0, 0.0, 0.0));
    }

    /// `(3/5, 4/5)` is exactly on the unit circle and neither coordinate is an
    /// `f64`, so the check is against the published bound rather than against
    /// an equality that cannot hold.
    #[test]
    fn the_quarter_circle_passes_through_three_fifths_four_fifths_at_the_middle() {
        let arc = the_quarter_circle();
        let middle = arc.evaluate(0.5).unwrap();
        let bound = arc
            .evaluation_roundoff_at(0.5)
            .unwrap()
            .expect("every number in this arc is a small whole one");

        assert!((middle.x() - 0.6).abs() <= bound);
        assert!((middle.y() - 0.8).abs() <= bound);
        assert_eq!(middle.z(), 0.0, "a flat arc has an exact z");
    }

    /// The same point as the test above, with no bound anywhere in the
    /// assertion. Three fifths is not an `f64`, so the box has to have width,
    /// and `z` is exactly zero, which is what stops this from being a box around
    /// everything.
    ///
    /// The box is asserted to hold `0.6` strictly, which is the `f64` nearest
    /// three fifths rather than three fifths itself. That settles the real
    /// claim: the true value is within half a step of `0.6`, and a box with
    /// `0.6` strictly inside it holds everything within half a step of it.
    #[test]
    fn the_enclosure_at_the_middle_of_the_quarter_circle_holds_three_fifths_and_four_fifths() {
        let arc = the_quarter_circle();
        let [x, y, z] = match arc.evaluate_with_enclosure(0.5).unwrap() {
            PredicatePoint::Constructed(sides) => sides,
            other => panic!("three fifths is not an f64, so this cannot be exact: {other:?}"),
        };

        assert!(
            x.lo() < 0.6 && 0.6 < x.hi(),
            "{x:?} does not hold three fifths"
        );
        assert!(
            y.lo() < 0.8 && 0.8 < y.hi(),
            "{y:?} does not hold four fifths"
        );
        assert_eq!(z.width(), 0.0, "a flat arc has an exact z");
        assert_eq!(z.lo(), 0.0);
    }

    /// `KnotVector::new` refuses a vector whose first and last knot cannot be
    /// subtracted, and the argument that one check is enough is that every
    /// difference either recursion forms lies between those two knots — de
    /// Boor's `knots[right] − knots[left]` and `u − knots[left]`, and the
    /// derived net's `knots[i+p+1] − knots[i+1]`. The knots rise, so each of
    /// those differences is no wider in truth than the whole range, and
    /// rounding to nearest keeps the order.
    ///
    /// This is that argument made to run. The knots here are the widest pair
    /// the kernel accepts — they subtract to exactly `f64::MAX` — and both
    /// recursions are driven across them. The clamp is the sharp part: with the
    /// coefficients collapsing to zero, *every* parameter read back as control
    /// point 0, so the top of the domain coming back as control point 3 is the
    /// assertion the overflow could not have passed.
    #[test]
    fn the_widest_knot_vector_the_kernel_takes_still_evaluates_and_differentiates() {
        let half_the_range = f64::MAX / 2.0;
        let knots = KnotVector::new(
            2,
            vec![
                -half_the_range,
                -half_the_range,
                -half_the_range,
                0.0,
                half_the_range,
                half_the_range,
                half_the_range,
            ],
        )
        .expect("clamped, rising, one interior knot, and a range f64 can subtract");
        let corners: Vec<Point> = (0..4).map(|i| Point::new(f64::from(i), 0.0, 0.0)).collect();
        let curve = BSplineCurve::new(knots, &corners, &[1.0; 4]).expect("four small points");

        let (lo, hi) = curve.domain();
        assert_eq!(curve.evaluate(lo).unwrap(), Point::new(0.0, 0.0, 0.0));
        assert_eq!(curve.evaluate(hi).unwrap(), Point::new(3.0, 0.0, 0.0));

        let middle = curve.evaluate(0.0).unwrap().x();
        assert!(
            0.0 < middle && middle < 3.0,
            "the interior point {middle} is outside the control net it is a blend of"
        );
        assert!(curve.tangent(0.0).unwrap().is_finite());
    }

    /// A span `1e300` wide under control coordinates around `5e307`. The
    /// derived point is `p · (P[i+1] − P[i]) / width`, and forming the product
    /// first overflowed while the answer itself, `2e8`, fits in an `f64` three
    /// hundred orders of magnitude over. `tangent` handed back a `NaN` from a
    /// method that documents one failure and does not panic — a silent corrupt
    /// value, which `docs/decisions.md` §4.1 ranks below a crash.
    ///
    /// Dividing before scaling is what makes this exact rather than merely
    /// finite: `1e308 / 1e300` is `1e8` with nothing rounded, and the de Boor
    /// coefficient at this parameter is exactly a quarter.
    #[test]
    fn a_tangent_across_a_span_too_wide_to_scale_before_dividing_is_exact() {
        let knots = KnotVector::new(2, vec![0.0, 0.0, 0.0, 1e300, 1e300, 1e300])
            .expect("a clamped single-span quadratic");
        let corners = [
            Point::new(-5e307, 0.0, 0.0),
            Point::new(5e307, 0.0, 0.0),
            Point::new(-5e307, 0.0, 0.0),
        ];
        let curve = BSplineCurve::new(knots, &corners, &[1.0; 3]).expect("three finite points");

        let a_quarter_along = 2.5e299;
        assert_eq!(
            curve.evaluate(a_quarter_along).unwrap(),
            Point::new(-1.25e307, 0.0, 0.0)
        );
        assert_eq!(
            curve.tangent(a_quarter_along).unwrap(),
            Vector::new(1e8, 0.0, 0.0)
        );
    }

    /// The two control points are a whole `f64` range apart, so `P[1] − P[0]`
    /// runs off the end of the range before any division can help. Reordering
    /// cannot reach this one — every route to the derived point needs that
    /// subtraction — but halving both first does, and the factor of two comes
    /// back in the scale.
    ///
    /// The true tangent is exactly `5e307`, which fits with room to spare, so
    /// refusing it was a gap and not a limit. Asserted exactly: halving is a
    /// power of two and rounds nothing here.
    #[test]
    fn a_curve_whose_two_control_points_are_a_whole_f64_range_apart_still_has_its_tangent() {
        let knots = KnotVector::new(1, vec![0.0, 0.0, 4.0, 4.0]).expect("one straight span");
        let ends = [Point::new(-1e308, 0.0, 0.0), Point::new(1e308, 0.0, 0.0)];
        let curve = BSplineCurve::new(knots, &ends, &[1.0; 2]).expect("two finite points");

        assert_eq!(curve.evaluate(2.0).unwrap(), Point::origin());
        assert_eq!(curve.tangent(2.0).unwrap(), Vector::new(5e307, 0.0, 0.0));
    }

    /// The rational combination `(A' − w'·C) / w` overflows at `w'·C` here
    /// while the answer fits easily, because the weight is large enough to
    /// bring the product back.
    ///
    /// Homogeneous `A` is the same `2^664` at both control points, so `A'` is
    /// exactly zero and the whole tangent is the `w'·C` term. `w'` is `2^500`
    /// against a point at about `2^600`, so their product passes `2^1024` and
    /// is lost — but divided by the weight first, `w'/w` is `2^400` and
    /// nothing leaves the range. The true tangent is `-4.76226047638341e300`,
    /// which exact rational arithmetic confirms to every digit `f64` carries.
    #[test]
    fn a_weight_derivative_that_overflows_against_the_point_still_gives_a_tangent() {
        let knots = KnotVector::new(1, vec![0.0, 0.0, 2f64.powi(-400), 2f64.powi(-400)])
            .expect("one very narrow span");
        let ends = [
            Point::new(2f64.powi(600), 0.0, 0.0),
            Point::new(2f64.powi(599), 0.0, 0.0),
        ];
        let weights = [2f64.powi(100), 2f64.powi(101)];
        let curve = BSplineCurve::new(knots, &ends, &weights).expect("two finite points");

        let halfway = 2f64.powi(-401);
        assert_eq!(
            curve.tangent(halfway).unwrap(),
            Vector::new(-4.762_260_476_383_41e300, 0.0, 0.0)
        );
    }

    /// The second way `w'·C` leaves the range, which dividing first cannot fix.
    ///
    /// Here `w` is exactly `1`, so `A'/w − (w'/w)·C` is the plain form written
    /// differently and overflows identically. Both terms sit near the end of the
    /// range while their difference is small, and only scaling helps. The true
    /// tangent is about `-1e307`; this was refused until halving was added
    /// alongside the divide.
    #[test]
    fn a_tangent_whose_two_terms_are_both_near_the_end_of_the_range_still_arrives() {
        let hair = 1e-300;
        let knots = KnotVector::new(1, vec![0.0, 0.0, hair, hair]).expect("one hair of a span");
        let ends = [
            Point::new(1.9e8, 0.0, 0.0),
            Point::new(2.65e8 / 1.5, 0.0, 0.0),
        ];
        let curve = BSplineCurve::new(knots, &ends, &[0.5, 1.5]).expect("two normal weights");

        let tangent = curve.tangent(hair / 2.0).expect("a representable tangent");
        let truth = -1.0e307;
        assert!(
            ((tangent.x() - truth) / truth).abs() < 1e-12,
            "the true tangent is about {truth:e} and this came back {:e}",
            tangent.x()
        );
    }

    /// Two of the smallest subnormal weights blend to exactly zero at the
    /// middle of a span, because `0.5 * 5e-324` rounds half-to-even down to
    /// nothing. The constructor used to accept these — "finite and above zero"
    /// was its whole rule — and then `evaluate` panicked on the way back out.
    ///
    /// It is refused at construction now. Weights count only through their
    /// ratios, so scaling every one of them up by a power of two gives the
    /// identical curve exactly — here it does, since `2¹⁰⁰⁰` leaves the scaled
    /// weight well inside `f64`'s normal range. That scaling does not always
    /// recover the curve: weights of `5e-324` and `1e300` have no power of two
    /// that brings both into the normal range at once.
    #[test]
    fn weights_small_enough_to_blend_away_to_nothing_are_refused_up_front() {
        let smallest = f64::from_bits(1);
        assert_eq!(0.5 * smallest, 0.0, "this is what the constructor prevents");

        let knots = KnotVector::new(1, vec![0.0, 0.0, 1.0, 1.0]).expect("one straight span");
        let ends = [Point::origin(), Point::new(2.0, 0.0, 0.0)];
        assert_eq!(
            BSplineCurve::new(knots.clone(), &ends, &[smallest; 2]),
            Err(Error::WeightTooSmallToBlend { weight: smallest })
        );

        let scaled_up = smallest * 2f64.powi(1000);
        let curve = BSplineCurve::new(knots, &ends, &[scaled_up; 2])
            .expect("the same curve, with the weights carried somewhere f64 can blend them");
        assert_eq!(curve.evaluate(0.5).unwrap(), Point::new(1.0, 0.0, 0.0));
    }

    /// Scaling every weight by the same power of two only ever recovers the
    /// curve when the whole group of weights still fits `f64`'s normal range
    /// afterwards. `5e-324` and `1e300` together do not: `2⁵²` is the only
    /// power that lifts `5e-324` into the normal range, and at that power
    /// `1e300` is already past `f64::MAX`.
    #[test]
    fn no_power_of_two_rescues_both_a_subnormal_weight_and_an_astronomical_one() {
        let subnormal = f64::from_bits(1);
        let huge = 1e300;

        let normal_at = |exponent: i32| (subnormal * 2f64.powi(exponent)).is_normal();
        let fits_at = |exponent: i32| (huge * 2f64.powi(exponent)).is_finite();

        assert!(normal_at(52), "2^52 is the power that normalizes 5e-324");
        assert!(
            !fits_at(52),
            "and at that power 1e300 is already past f64::MAX"
        );

        for exponent in -1074..=1023 {
            assert!(
                !(normal_at(exponent) && fits_at(exponent)),
                "exponent {exponent} would bring both weights into range at once"
            );
        }
    }

    /// The old message promised scaling always recovers the curve. It does
    /// not, and the message says so now.
    #[test]
    fn weight_too_small_to_blend_message_says_scaling_is_not_always_enough() {
        let message = Error::WeightTooSmallToBlend {
            weight: f64::from_bits(1),
        }
        .to_string();
        assert!(
            message.contains("but only if the largest weight still has room to grow"),
            "message was: {message}"
        );
    }

    /// The same refusal reaches the routes that went through `.expect()` rather
    /// than through a `Result`, which is what made this worth fixing at the
    /// constructor instead of at each caller: `bezier_segments` hands back a
    /// `Vec` and has no error channel at all.
    #[test]
    fn a_net_that_cannot_be_blended_never_reaches_bezier_extraction() {
        let knots =
            KnotVector::new(2, vec![0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0]).expect("one interior knot");
        let points: Vec<Point> = (0..4).map(|i| Point::new(f64::from(i), 0.0, 0.0)).collect();
        assert_eq!(
            BSplineCurve::new(knots, &points, &[f64::from_bits(1); 4]),
            Err(Error::WeightTooSmallToBlend {
                weight: f64::from_bits(1)
            })
        );
    }

    /// The refusal that is left, and it is about the numbers asked for rather
    /// than the arithmetic used.
    ///
    /// The span is one `f64` step wide at the bottom of the range while the
    /// control points are ordinary, so the true derivative is past `f64::MAX`
    /// and no amount of scaling reaches a number that does not exist.
    /// [`KnotVector::new`] takes a span like this on purpose — two knots a step
    /// apart are the caller's data, not something to snap — so §4.1 makes this
    /// an `Err` and not a panic. The curve still evaluates perfectly well.
    #[test]
    fn a_span_a_single_f64_step_wide_has_a_tangent_too_big_for_f64() {
        let a_step = f64::from_bits(1);
        let knots = KnotVector::new(1, vec![0.0, 0.0, a_step, a_step]).expect("one hair of a span");
        let ends = [Point::new(-0.5, 0.0, 0.0), Point::new(0.5, 0.0, 0.0)];
        let curve = BSplineCurve::new(knots, &ends, &[1.0; 2]).expect("two finite points");

        assert_eq!(curve.evaluate(0.0).unwrap(), Point::new(-0.5, 0.0, 0.0));
        assert_eq!(
            curve.tangent(0.0),
            Err(Error::NotFinite {
                what: "spline derivative"
            })
        );
    }

    /// Knots `[0, 0, 1, 2, 2]`, so inserting at a half blends the first two
    /// control points in equal shares — an exact answer with nothing rounded,
    /// and every other point copied.
    #[test]
    fn inserting_a_knot_halfway_along_a_straight_run_is_the_midpoint_and_three_copies() {
        let refined = a_straight_run().with_knot_inserted(0.5).unwrap();

        assert_eq!(refined.knots().knots(), &[0.0, 0.0, 0.5, 1.0, 2.0, 2.0]);
        assert_eq!(refined.control_point_count(), 4);
        assert_eq!(refined.control_point(0), Point::new(0.0, 0.0, 0.0));
        assert_eq!(refined.control_point(1), Point::new(1.0, 2.0, 4.0));
        assert_eq!(refined.control_point(2), Point::new(2.0, 4.0, 8.0));
        assert_eq!(refined.control_point(3), Point::new(6.0, 0.0, -4.0));
    }

    /// A quadratic whose last three spans are single `f64` steps apart, so the
    /// knot differences an insertion divides by are as small as `f64` allows
    /// without shutting.
    ///
    /// The tail clamp sits two steps above the interior knots, which is what
    /// makes the coefficient's denominator two steps wide.
    fn a_curve_whose_spans_are_a_hair_wide() -> BSplineCurve {
        let half = 0.5f64;
        let step_up = half.next_up();
        let two_steps_up = step_up.next_up();
        let knots = KnotVector::new(
            2,
            vec![
                0.0,
                0.0,
                0.0,
                half,
                half,
                step_up,
                two_steps_up,
                two_steps_up,
                two_steps_up,
            ],
        )
        .expect("rising, clamped, and no interior knot past multiplicity two");

        let points: Vec<Point> = (0..6)
            .map(|i| Point::new(i as f64, 2.0 * i as f64, 4.0 * i as f64))
            .collect();
        BSplineCurve::new(knots, &points, &[1.0; 6]).expect("six dyadic points, six weights")
    }

    /// The case a reviewer looks for: a span so narrow that a de Boor
    /// coefficient might go to `0/0` or run away. It does neither, and this
    /// says what it does instead rather than that it is fine.
    ///
    /// Inserting the middle interior knot divides `(u − knots[i])` by
    /// `(knots[i+2] − knots[i])`. Both are one and two `f64` steps of a half,
    /// and both subtractions are exact — the two numbers are within a factor of
    /// two of each other, so nothing is lost. The quotient of `2⁻⁵³` by `2⁻⁵²`
    /// is exactly a half, so the new control point is the exact midpoint of the
    /// two it came from, with nothing rounded anywhere.
    ///
    /// The denominator can never be zero: the two knots it subtracts always sit
    /// on opposite sides of a span of positive width, which is what the
    /// assertion inside the insertion states.
    #[test]
    fn inserting_into_a_span_two_f64_steps_wide_lands_on_the_exact_midpoint() {
        let curve = a_curve_whose_spans_are_a_hair_wide();
        let barely_above_a_half = 0.5f64.next_up();
        let refined = curve.with_knot_inserted(barely_above_a_half).unwrap();

        assert_eq!(refined.control_point_count(), 7);
        assert_eq!(
            refined.homogeneous_control_point(4),
            [3.5, 7.0, 14.0, 1.0],
            "a coefficient of exactly a half puts the new point halfway between points 3 and 4"
        );
        for i in 0..4 {
            assert_eq!(
                refined.homogeneous_control_point(i),
                curve.homogeneous_control_point(i)
            );
        }
        for i in 5..7 {
            assert_eq!(
                refined.homogeneous_control_point(i),
                curve.homogeneous_control_point(i - 1)
            );
        }
    }

    /// Splitting the same curve at the same hair-wide knot has to give two
    /// curves the kernel accepts, and it is the case where a coefficient of
    /// exactly one and a coefficient of exactly zero both turn up.
    #[test]
    fn a_curve_whose_spans_are_a_hair_wide_still_splits_into_two_curves() {
        let curve = a_curve_whose_spans_are_a_hair_wide();
        let barely_above_a_half = 0.5f64.next_up();
        let (below, above) = curve.split_at(barely_above_a_half).unwrap();

        assert_eq!(below.domain(), (0.0, barely_above_a_half));
        assert_eq!(
            above.domain(),
            (barely_above_a_half, 0.5f64.next_up().next_up())
        );
        assert_eq!(
            below.homogeneous_control_point(below.control_point_count() - 1),
            above.homogeneous_control_point(0)
        );
    }

    #[test]
    fn inserting_a_knot_at_either_end_of_the_domain_is_refused() {
        let curve = the_quarter_circle();
        for u in [0.0, 1.0] {
            assert_eq!(
                curve.with_knot_inserted(u),
                Err(Error::CannotRepeatKnotAgain {
                    knot: u,
                    times: 3,
                    degree: 2
                }),
                "a clamped end already appears one more time than the degree"
            );
        }
    }

    #[test]
    fn inserting_an_interior_knot_that_already_repeats_as_often_as_the_degree_is_refused() {
        let knots = KnotVector::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 2.0])
            .expect("an interior knot at multiplicity two is allowed at degree two");
        let points: Vec<Point> = (0..5).map(|i| Point::new(i as f64, 0.0, 0.0)).collect();
        let curve = BSplineCurve::new(knots, &points, &[1.0; 5]).unwrap();

        assert_eq!(
            curve.with_knot_inserted(1.0),
            Err(Error::CannotRepeatKnotAgain {
                knot: 1.0,
                times: 2,
                degree: 2
            })
        );
    }

    #[test]
    fn splitting_at_either_end_of_the_domain_leaves_nothing_on_one_side() {
        let curve = the_quarter_circle();
        for u in [0.0, 1.0] {
            assert_eq!(
                curve.split_at(u),
                Err(Error::NothingOnOneSideOfTheSplit {
                    parameter: u,
                    lo: 0.0,
                    hi: 1.0
                })
            );
        }
    }

    #[test]
    fn a_curve_that_is_already_one_bezier_gives_back_itself_and_nothing_else() {
        let curve = the_quarter_circle();
        let segments = curve.bezier_segments();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], curve);
    }

    #[test]
    fn a_parameter_off_the_curve_is_refused_by_everything_that_takes_one() {
        let curve = the_quarter_circle();
        for u in [-0.25, 1.5, f64::NAN, f64::INFINITY] {
            let refusals = [
                curve.evaluate(u).err(),
                curve.tangent(u).err(),
                curve.evaluate_with_enclosure(u).err(),
                curve.evaluation_roundoff_at(u).err(),
                curve.with_knot_inserted(u).err(),
                curve.split_at(u).err(),
            ];
            for refusal in refusals {
                match refusal {
                    // Compared bit for bit, because a `NaN` handed back as a
                    // `NaN` does not compare equal to itself.
                    Some(Error::ParameterOutsideDomain { parameter, lo, hi }) => {
                        assert_eq!(parameter.to_bits(), u.to_bits());
                        assert_eq!((lo, hi), (0.0, 1.0));
                    }
                    other => panic!("{u} does not name a point on this arc: {other:?}"),
                }
            }
        }
    }
}
