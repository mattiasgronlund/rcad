//! Spline curves, always rational.

use rcad_predicates::Point as PredicatePoint;
use serde::{Deserialize, Serialize};

use crate::de_boor::{
    Coefficient, EVALUATION_ROUNDOFF, de_boor, derived_points, divided, enclosed,
    rational_derivative, roundoff_of_a_divided_point,
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

impl BSplineCurve {
    /// The curve through the given control net.
    ///
    /// `control_points` and `weights` both have
    /// [`KnotVector::control_point_count`] entries, and every weight is a
    /// finite number greater than zero.
    ///
    /// # Errors
    ///
    /// [`Error::WrongNumberOfControlPoints`] if either list is the wrong
    /// length, [`Error::NotFinite`] if a control point has a coordinate that is
    /// not a finite number or if weighting it overflows, and
    /// [`Error::WeightNotPositive`] for a weight that is zero, negative, or not
    /// a finite number.
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
    /// If the weight at `u` does not come out above zero. Every stored weight
    /// is checked to be above zero at construction and de Boor mixes them with
    /// non-negative coefficients that sum to one, so nothing else can produce
    /// that — it means the stored net is not what construction promised. A
    /// broken invariant is a crash and a corrupt point is worse than a crash
    /// (`docs/decisions.md` §4.1). This is a deliberate departure from roadmap
    /// §4.5's blanket "never panic", which is about bad caller input; bad
    /// caller input here is the `Err` above.
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
    /// # Panics
    ///
    /// For the same broken invariant [`BSplineCurve::evaluate`] panics on.
    pub fn tangent(&self, u: f64) -> Result<Vector> {
        let (value, rate) = self.homogeneous_value_and_derivative(u)?;
        Ok(rational_derivative(value, rate))
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
        let mut largest = [0.0f64; 4];
        for point in &self.net[span - self.degree()..=span] {
            for (biggest, coordinate) in largest.iter_mut().zip(point) {
                *biggest = biggest.max(coordinate.abs());
            }
        }
        largest
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
            if !(x.is_finite() && y.is_finite() && z.is_finite()) {
                return Err(Error::NotFinite {
                    what: "weighted spline control point",
                });
            }
        }
        Ok(Self { knots, net })
    }
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

    #[test]
    fn a_parameter_off_the_curve_is_refused_by_everything_that_takes_one() {
        let curve = the_quarter_circle();
        for u in [-0.25, 1.5, f64::NAN, f64::INFINITY] {
            let refusals = [
                curve.evaluate(u).err(),
                curve.tangent(u).err(),
                curve.evaluate_with_enclosure(u).err(),
                curve.evaluation_roundoff_at(u).err(),
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
