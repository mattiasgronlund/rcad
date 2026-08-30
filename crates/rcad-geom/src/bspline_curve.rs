//! Spline curves, always rational.

use serde::{Deserialize, Serialize};

use crate::knot_vector::KnotVector;
use crate::point::Point;
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
}
