//! Spline surfaces, always rational.

use rcad_predicates::Point as PredicatePoint;
use serde::{Deserialize, Serialize};

use crate::bspline_curve::weigh;
use crate::de_boor::{
    Coefficient, EVALUATION_ROUNDOFF, after_two_passes, de_boor, derived_points, divided, enclosed,
    rational_derivative, roundoff_of_a_divided_point,
};
use crate::knot_vector::{KnotVector, MAX_DEGREE};
use crate::point::{Point, Vector};
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

    /// The point on the surface at `(u, v)`.
    ///
    /// Tensor-product evaluation is the curve evaluator applied twice: de Boor
    /// in `v` down each of the `degree_u + 1` active rows, then de Boor in `u`
    /// across the results, all in homogeneous coordinates with the divide
    /// happening once at the end.
    ///
    /// Ordinary `f64` — see the crate doc's "three kinds of answer".
    ///
    /// # Errors
    ///
    /// [`Error::ParameterOutsideDomain`] if `u` or `v` is not a finite number
    /// inside its knot vector's domain. That is the only way this can fail.
    ///
    /// # Panics
    ///
    /// If the weight at `(u, v)` does not come out above zero, which means the
    /// stored net is not what construction promised — see
    /// [`BSplineCurve::evaluate`](crate::BSplineCurve::evaluate).
    pub fn evaluate(&self, u: f64, v: f64) -> Result<Point> {
        Ok(divided(self.homogeneous_evaluate(u, v)?))
    }

    /// The first derivative of the surface along `u`.
    ///
    /// Collapsing every active row along `v` first leaves a curve in `u` whose
    /// control points are those collapsed rows, so the `u` derivative is the
    /// curve derivative of that — the derived net over the `u` knots. The
    /// rational correction `(A' − w'·S) / w` is the same one the curve needs.
    ///
    /// # Errors
    ///
    /// [`Error::ParameterOutsideDomain`], as [`BSplineSurface::evaluate`].
    ///
    /// # Panics
    ///
    /// For the same broken invariant [`BSplineSurface::evaluate`] panics on.
    pub fn partial_u(&self, u: f64, v: f64) -> Result<Vector> {
        let (u_span, v_span) = self.spans_of(u, v)?;
        let rows = self.rows_collapsed_along_v(u_span, v_span, v);
        let degree = self.u_knots.degree();

        let rate = self.derivative_across(self.u_knots.knots(), u_span, u, &rows[..=degree]);
        let value = self.homogeneous_evaluate(u, v)?;
        Ok(rational_derivative(value, rate))
    }

    /// The first derivative of the surface along `v`.
    ///
    /// The mirror of [`BSplineSurface::partial_u`]: collapse every active
    /// column along `u` first, and the `v` derivative is the curve derivative
    /// of what is left.
    ///
    /// # Errors
    ///
    /// [`Error::ParameterOutsideDomain`], as [`BSplineSurface::evaluate`].
    ///
    /// # Panics
    ///
    /// For the same broken invariant [`BSplineSurface::evaluate`] panics on.
    pub fn partial_v(&self, u: f64, v: f64) -> Result<Vector> {
        let (u_span, v_span) = self.spans_of(u, v)?;
        let columns = self.columns_collapsed_along_u(u_span, v_span, u);
        let degree = self.v_knots.degree();

        let rate = self.derivative_across(self.v_knots.knots(), v_span, v, &columns[..=degree]);
        let value = self.homogeneous_evaluate(u, v)?;
        Ok(rational_derivative(value, rate))
    }

    /// A box the true point at `(u, v)` is somewhere inside, ready for the
    /// predicate cascade.
    ///
    /// Both de Boor passes over intervals instead of `f64`, then the same
    /// perspective divide — see
    /// [`BSplineCurve::evaluate_with_enclosure`](crate::BSplineCurve::evaluate_with_enclosure)
    /// for why this is the only enclosure, why it is never intersected with
    /// [`EVALUATION_ROUNDOFF`], and when an exact point comes back out of
    /// interval arithmetic.
    ///
    /// # Errors
    ///
    /// [`Error::ParameterOutsideDomain`], as [`BSplineSurface::evaluate`] —
    /// the same one failure and no other.
    ///
    /// # Panics
    ///
    /// For the same broken invariant [`BSplineSurface::evaluate`] panics on.
    pub fn evaluate_with_enclosure(&self, u: f64, v: f64) -> Result<PredicatePoint> {
        Ok(enclosed(self.homogeneous_evaluate(u, v)?))
    }

    /// How far [`BSplineSurface::evaluate`] at `(u, v)` can be from the true
    /// point, per coordinate.
    ///
    /// `None` when [`EVALUATION_ROUNDOFF`]'s precondition does not hold. A
    /// surface is two de Boor passes rather than one, so the fraction it uses
    /// is [`after_two_passes`] of the published constant.
    ///
    /// # Errors
    ///
    /// [`Error::ParameterOutsideDomain`], as [`BSplineSurface::evaluate`].
    pub fn evaluation_roundoff_at(&self, u: f64, v: f64) -> Result<Option<f64>> {
        let (u_span, v_span) = self.spans_of(u, v)?;
        let homogeneous = self.homogeneous_evaluate(u, v)?;
        Ok(roundoff_of_a_divided_point(
            after_two_passes(EVALUATION_ROUNDOFF),
            self.largest_active_coordinates(u_span, v_span),
            homogeneous,
        ))
    }

    /// The point at `(u, v)` before the weight is divided back out, in
    /// whichever arithmetic the caller asked for.
    ///
    /// One body serves [`BSplineSurface::evaluate`] and
    /// [`BSplineSurface::evaluate_with_enclosure`], for the reason the curve's
    /// own version of this gives.
    pub(crate) fn homogeneous_evaluate<C: Coefficient>(&self, u: f64, v: f64) -> Result<[C; 4]> {
        let (u_span, v_span) = self.spans_of(u, v)?;
        let degree = self.u_knots.degree();
        let mut rows = self.rows_collapsed_along_v(u_span, v_span, v);
        Ok(de_boor(
            degree,
            self.u_knots.knots(),
            u_span,
            u,
            &mut rows[..=degree],
        ))
    }

    /// The homogeneous derivative of a curve whose control points are already
    /// collapsed, over `knots` at `span`.
    fn derivative_across(
        &self,
        knots: &[f64],
        span: usize,
        parameter: f64,
        active: &[[f64; 4]],
    ) -> [f64; 4] {
        let degree = active.len() - 1;
        let mut derived = [[0.0f64; 4]; MAX_DEGREE + 1];
        derived_points(degree, knots, span, active, &mut derived[..degree]);
        let inner = &knots[1..knots.len() - 1];
        de_boor(
            degree - 1,
            inner,
            span - 1,
            parameter,
            &mut derived[..degree],
        )
    }

    /// Each of the `degree_u + 1` active rows, collapsed to one homogeneous
    /// point by de Boor in `v`.
    fn rows_collapsed_along_v<C: Coefficient>(
        &self,
        u_span: usize,
        v_span: usize,
        v: f64,
    ) -> [[C; 4]; MAX_DEGREE + 1] {
        let u_degree = self.u_knots.degree();
        let v_degree = self.v_knots.degree();
        let across = self.v_knots.control_point_count();

        let mut rows = [[C::exactly(0.0); 4]; MAX_DEGREE + 1];
        for (offset, row) in rows[..=u_degree].iter_mut().enumerate() {
            let i = offset + u_span - u_degree;
            let mut along = [[C::exactly(0.0); 4]; MAX_DEGREE + 1];
            for (step, point) in along[..=v_degree].iter_mut().enumerate() {
                *point = self.net[i * across + step + v_span - v_degree].map(C::exactly);
            }
            *row = de_boor(
                v_degree,
                self.v_knots.knots(),
                v_span,
                v,
                &mut along[..=v_degree],
            );
        }
        rows
    }

    /// Each of the `degree_v + 1` active columns, collapsed to one homogeneous
    /// point by de Boor in `u`.
    fn columns_collapsed_along_u(
        &self,
        u_span: usize,
        v_span: usize,
        u: f64,
    ) -> [[f64; 4]; MAX_DEGREE + 1] {
        let u_degree = self.u_knots.degree();
        let v_degree = self.v_knots.degree();
        let across = self.v_knots.control_point_count();

        let mut columns = [[0.0f64; 4]; MAX_DEGREE + 1];
        for (offset, column) in columns[..=v_degree].iter_mut().enumerate() {
            let j = offset + v_span - v_degree;
            let mut along = [[0.0f64; 4]; MAX_DEGREE + 1];
            for (step, point) in along[..=u_degree].iter_mut().enumerate() {
                *point = self.net[(step + u_span - u_degree) * across + j];
            }
            *column = de_boor(
                u_degree,
                self.u_knots.knots(),
                u_span,
                u,
                &mut along[..=u_degree],
            );
        }
        columns
    }

    /// The two spans the parameters fall in, or the one failure evaluation has.
    fn spans_of(&self, u: f64, v: f64) -> Result<(usize, usize)> {
        for (knots, parameter) in [(&self.u_knots, u), (&self.v_knots, v)] {
            if !knots.contains(parameter) {
                let (lo, hi) = knots.domain();
                return Err(Error::ParameterOutsideDomain { parameter, lo, hi });
            }
        }
        Ok((
            self.u_knots.span_containing(u),
            self.v_knots.span_containing(v),
        ))
    }

    /// The biggest magnitude in each of the four homogeneous coordinates,
    /// across the patch of control points the two spans reach.
    fn largest_active_coordinates(&self, u_span: usize, v_span: usize) -> [f64; 4] {
        let u_degree = self.u_knots.degree();
        let v_degree = self.v_knots.degree();
        let across = self.v_knots.control_point_count();

        let mut largest = [0.0f64; 4];
        for i in u_span - u_degree..=u_span {
            for j in v_span - v_degree..=v_span {
                for (biggest, coordinate) in largest.iter_mut().zip(&self.net[i * across + j]) {
                    *biggest = biggest.max(coordinate.abs());
                }
            }
        }
        largest
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
    use crate::bspline_curve::BSplineCurve;

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

    /// Degree two across `u` and one across `v`, with a different weight at
    /// every grid position so nothing about it is symmetric.
    fn a_lumpy_patch() -> BSplineSurface {
        BSplineSurface::new(
            a_quadratic(),
            a_straight_run(),
            &a_six_point_grid(),
            &[1.0, 2.0, 0.5, 3.0, 1.5, 4.0],
        )
        .expect("six points, six positive weights")
    }

    /// Both knot vectors are clamped, so each corner of the parameter domain is
    /// the corner control point itself, bit for bit through both passes.
    #[test]
    fn each_corner_of_the_domain_is_exactly_the_corner_control_point() {
        let patch = a_lumpy_patch();
        for (u, i) in [(0.0, 0), (1.0, 2)] {
            for (v, j) in [(0.0, 0), (1.0, 1)] {
                assert_eq!(patch.evaluate(u, v).unwrap(), patch.control_point(i, j));
            }
        }
    }

    /// A grid whose two columns are the same points cannot change along `v`, so
    /// the two collapsed columns are the same numbers and the derived point
    /// between them is exactly zero.
    #[test]
    fn a_surface_that_does_not_change_along_v_has_an_exactly_zero_v_derivative() {
        let flat = swept_along_v(&the_quarter_arc(), 0.0);
        for u in [0.0, 0.125, 0.5, 0.875, 1.0] {
            for v in [0.0, 0.25, 1.0] {
                assert_eq!(flat.partial_v(u, v).unwrap(), Vector::zero());
            }
        }
    }

    /// At the bottom of the `v` domain the first pass collapses each row to
    /// that row's first control point exactly, so what is left is the curve
    /// itself — the same numbers through the same recursion.
    #[test]
    fn a_swept_curve_read_at_the_bottom_of_the_sweep_is_that_curve_bit_for_bit() {
        let arc = the_quarter_arc();
        let swept = swept_along_v(&arc, 3.0);
        for u in [0.0, 0.125, 0.5, 0.875, 1.0] {
            assert_eq!(swept.evaluate(u, 0.0).unwrap(), arc.evaluate(u).unwrap());
            assert_eq!(swept.partial_u(u, 0.0).unwrap(), arc.tangent(u).unwrap());
        }
    }

    /// Swapping the two directions and transposing the grid describes the same
    /// surface, but the passes run in the other order — so this is not bit-for-
    /// bit and the published bound is what says how far apart they may be. A
    /// row-major index read as column-major fails it by miles.
    #[test]
    fn transposing_a_patch_and_swapping_its_parameters_gives_the_same_point() {
        let patch = a_lumpy_patch();
        let turned = transposed(&patch);

        for u in [0.0, 0.25, 0.5, 1.0] {
            for v in [0.0, 0.5, 1.0] {
                let here = patch.evaluate(u, v).unwrap();
                let there = turned.evaluate(v, u).unwrap();
                let bound = patch
                    .evaluation_roundoff_at(u, v)
                    .unwrap()
                    .expect("every number in this patch is a small whole one");
                for (mine, yours) in here.coordinates().iter().zip(there.coordinates()) {
                    assert!(
                        (mine - yours).abs() <= 2.0 * bound,
                        "at ({u}, {v}) the two ways round differ by more than both bounds"
                    );
                }
            }
        }
    }

    /// A corner of a clamped patch is the corner control point through both
    /// passes with nothing rounded, so the enclosure there is a point and not a
    /// box. That is what lets the exact level of the cascade run on a surface
    /// point, and always answering with a box would lose it.
    #[test]
    fn a_corner_of_a_patch_encloses_its_control_point_with_no_width_at_all() {
        let patch = a_lumpy_patch();
        for (u, i) in [(0.0, 0), (1.0, 2)] {
            for (v, j) in [(0.0, 0), (1.0, 1)] {
                assert_eq!(
                    patch.evaluate_with_enclosure(u, v).unwrap(),
                    PredicatePoint::Exact(patch.control_point(i, j).coordinates())
                );
            }
        }
    }

    #[test]
    fn a_parameter_off_the_surface_is_refused_in_whichever_direction_it_is_off() {
        let patch = a_lumpy_patch();
        for (u, v) in [(-1.0, 0.5), (0.5, 2.0), (f64::NAN, 0.5), (0.5, f64::NAN)] {
            let refusals = [
                patch.evaluate(u, v).err(),
                patch.partial_u(u, v).err(),
                patch.partial_v(u, v).err(),
                patch.evaluate_with_enclosure(u, v).err(),
                patch.evaluation_roundoff_at(u, v).err(),
            ];
            for refusal in refusals {
                assert!(
                    matches!(refusal, Some(Error::ParameterOutsideDomain { .. })),
                    "({u}, {v}) does not name a point on this patch: {refusal:?}"
                );
            }
        }
    }

    /// The quarter of the unit circle: `(1,0)`, `(1,1)`, `(0,1)` with weights
    /// `1, 1, 2`.
    fn the_quarter_arc() -> BSplineCurve {
        let corners = [
            Point::new(1.0, 0.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
            Point::new(0.0, 1.0, 0.0),
        ];
        BSplineCurve::new(a_quadratic(), &corners, &[1.0, 1.0, 2.0])
            .expect("three points, three positive weights")
    }

    /// `arc` pulled `height` along `z`, as a surface of degree one in `v`.
    fn swept_along_v(arc: &BSplineCurve, height: f64) -> BSplineSurface {
        let mut net = Vec::new();
        for i in 0..arc.control_point_count() {
            let [x, y, z, weight] = arc.homogeneous_control_point(i);
            net.push([x, y, z, weight]);
            net.push([x, y, z + weight * height, weight]);
        }
        BSplineSurface::try_from((arc.knots().clone(), a_straight_run(), net))
            .expect("a sweep of a valid curve keeps every weight where it was")
    }

    /// The same surface with its two directions swapped and its grid turned on
    /// its side.
    fn transposed(surface: &BSplineSurface) -> BSplineSurface {
        let rows = surface.u_knots().control_point_count();
        let columns = surface.v_knots().control_point_count();
        let mut net = Vec::with_capacity(rows * columns);
        for j in 0..columns {
            for i in 0..rows {
                net.push(surface.homogeneous_control_point(i, j));
            }
        }
        BSplineSurface::try_from((surface.v_knots().clone(), surface.u_knots().clone(), net))
            .expect("turning a grid on its side keeps every number in it")
    }
}
