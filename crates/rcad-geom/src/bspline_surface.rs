//! Spline surfaces, always rational.

use serde::{Deserialize, Serialize};

use crate::bspline_curve::weigh;
use crate::knot_vector::KnotVector;
use crate::point::Point;
use crate::{Error, Result};

/// A tensor-product spline surface: a knot vector in `u`, a knot vector in `v`,
/// and a grid of weighted control points.
///
/// The grid is stored one row at a time, so the control point at `(i, j)` is
/// entry `i * v_count + j`, where `i` runs along `u` and `j` along `v`.
///
/// Everything the note on [`BSplineCurve`](crate::BSplineCurve) says about
/// homogeneous storage applies here word for word: the stored numbers are
/// `[w*x, w*y, w*z, w]`, [`BSplineSurface::weight`] gives back exactly what
/// arrived, and [`BSplineSurface::control_point`] is only exact when the weight
/// is one.
///
/// Serializes as `(u knots, v knots, homogeneous net)` and is checked again on
/// the way in.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    try_from = "(KnotVector, KnotVector, Vec<[f64; 4]>)",
    into = "(KnotVector, KnotVector, Vec<[f64; 4]>)"
)]
pub struct BSplineSurface {
    u_knots: KnotVector,
    v_knots: KnotVector,
    net: Vec<[f64; 4]>,
}

impl BSplineSurface {
    /// The surface through the given grid of control points.
    ///
    /// `control_points` and `weights` both hold one entry per grid position,
    /// row by row: `u_knots.control_point_count()` rows of
    /// `v_knots.control_point_count()` each. Every weight is a finite number
    /// greater than zero.
    ///
    /// # Errors
    ///
    /// [`Error::WrongNumberOfControlPoints`] if either list is the wrong
    /// length, [`Error::NotFinite`] if a control point has a coordinate that is
    /// not a finite number or if weighting it overflows, and
    /// [`Error::WeightNotPositive`] for a weight that is zero, negative, or not
    /// a finite number.
    pub fn new(
        u_knots: KnotVector,
        v_knots: KnotVector,
        control_points: &[Point],
        weights: &[f64],
    ) -> Result<Self> {
        let expected = u_knots.control_point_count() * v_knots.control_point_count();
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
        Self::from_homogeneous_net(u_knots, v_knots, net)
    }

    /// The knot vector along `u`, the direction the rows run down.
    #[must_use]
    pub fn u_knots(&self) -> &KnotVector {
        &self.u_knots
    }

    /// The knot vector along `v`, the direction a row runs along.
    #[must_use]
    pub fn v_knots(&self) -> &KnotVector {
        &self.v_knots
    }

    /// The control point at `(i, j)`, with its weight divided back out.
    ///
    /// Exact only when the weight is `1.0` — see [`BSplineSurface`].
    ///
    /// # Panics
    ///
    /// If `i` or `j` is past the edge of the grid.
    #[must_use]
    pub fn control_point(&self, i: usize, j: usize) -> Point {
        let [x, y, z, weight] = self.homogeneous_control_point(i, j);
        Point::new(x / weight, y / weight, z / weight)
    }

    /// The weight at `(i, j)`, exactly as it was handed over.
    ///
    /// # Panics
    ///
    /// If `i` or `j` is past the edge of the grid.
    #[must_use]
    pub fn weight(&self, i: usize, j: usize) -> f64 {
        self.homogeneous_control_point(i, j)[3]
    }

    /// The control point at `(i, j)` as it is stored: `[w*x, w*y, w*z, w]`.
    ///
    /// # Panics
    ///
    /// If `i` or `j` is past the edge of the grid.
    #[must_use]
    pub fn homogeneous_control_point(&self, i: usize, j: usize) -> [f64; 4] {
        let across = self.v_knots.control_point_count();
        assert!(j < across, "column {j} is past the {across} in the grid");
        self.net[i * across + j]
    }

    /// The surface from a net that is already weighted, checking everything a
    /// stored net has to satisfy.
    ///
    /// This is the way in for `serde`, which must not go through
    /// [`BSplineSurface::new`]: dividing the stored net back to affine points
    /// and weighting them again would round twice.
    fn from_homogeneous_net(
        u_knots: KnotVector,
        v_knots: KnotVector,
        net: Vec<[f64; 4]>,
    ) -> Result<Self> {
        let expected = u_knots.control_point_count() * v_knots.control_point_count();
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
        Ok(Self {
            u_knots,
            v_knots,
            net,
        })
    }
}

impl TryFrom<(KnotVector, KnotVector, Vec<[f64; 4]>)> for BSplineSurface {
    type Error = Error;

    fn try_from((u_knots, v_knots, net): (KnotVector, KnotVector, Vec<[f64; 4]>)) -> Result<Self> {
        Self::from_homogeneous_net(u_knots, v_knots, net)
    }
}

impl From<BSplineSurface> for (KnotVector, KnotVector, Vec<[f64; 4]>) {
    fn from(surface: BSplineSurface) -> Self {
        (surface.u_knots, surface.v_knots, surface.net)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_quadratic() -> KnotVector {
        KnotVector::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]).expect("a single Bezier span")
    }

    fn a_straight_run() -> KnotVector {
        KnotVector::new(1, vec![0.0, 0.0, 1.0, 1.0]).expect("one straight span")
    }

    /// Three control points along `u`, two along `v`, in row-major order.
    fn a_six_point_grid() -> Vec<Point> {
        vec![
            Point::new(1.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 1.0),
            Point::new(1.0, 1.0, 0.0),
            Point::new(1.0, 1.0, 1.0),
            Point::new(0.0, 1.0, 0.0),
            Point::new(0.0, 1.0, 1.0),
        ]
    }

    #[test]
    fn a_surface_keeps_both_knot_vectors_and_reads_its_grid_row_by_row() {
        let surface = BSplineSurface::new(
            a_quadratic(),
            a_straight_run(),
            &a_six_point_grid(),
            &[1.0; 6],
        )
        .unwrap();

        assert_eq!(surface.u_knots().degree(), 2);
        assert_eq!(surface.v_knots().degree(), 1);
        for i in 0..3 {
            for j in 0..2 {
                assert_eq!(surface.control_point(i, j), a_six_point_grid()[i * 2 + j]);
            }
        }
    }

    #[test]
    fn a_grid_of_the_wrong_size_is_refused() {
        assert_eq!(
            BSplineSurface::new(
                a_quadratic(),
                a_straight_run(),
                &a_six_point_grid()[..5],
                &[1.0; 5]
            ),
            Err(Error::WrongNumberOfControlPoints {
                expected: 6,
                given: 5
            })
        );
    }

    #[test]
    fn a_weight_of_zero_leaves_nothing_to_divide_by() {
        let mut weights = [1.0; 6];
        weights[4] = 0.0;
        assert_eq!(
            BSplineSurface::new(
                a_quadratic(),
                a_straight_run(),
                &a_six_point_grid(),
                &weights
            ),
            Err(Error::WeightNotPositive { weight: 0.0 })
        );
    }

    #[test]
    fn a_control_point_that_is_not_a_number_is_refused() {
        let mut grid = a_six_point_grid();
        grid[2] = Point::new(0.0, f64::INFINITY, 0.0);
        assert_eq!(
            BSplineSurface::new(a_quadratic(), a_straight_run(), &grid, &[1.0; 6]),
            Err(Error::NotFinite {
                what: "spline control point"
            })
        );
    }

    /// Without the column check, `(0, 2)` would silently read `(1, 0)` — the
    /// first entry of the next row.
    #[test]
    #[should_panic(expected = "past the 2 in the grid")]
    fn reading_past_the_end_of_a_row_panics_instead_of_wrapping_onto_the_next() {
        let surface = BSplineSurface::new(
            a_quadratic(),
            a_straight_run(),
            &a_six_point_grid(),
            &[1.0; 6],
        )
        .unwrap();
        let _ = surface.control_point(0, 2);
    }
}
