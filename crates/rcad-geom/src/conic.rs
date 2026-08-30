//! Circles and arcs written down with nothing rounded away.
//!
//! # Why a circle fits in `f64` at all
//!
//! A rational quadratic Bezier with control points `P0, P1, P2` and weights
//! `(1, cos θ, 1)` is a circular arc, as long as `P1` is where the tangents at
//! `P0` and `P2` meet and the sweep is `2θ`. For a quarter circle `θ` is 45
//! degrees, `cos θ` is `√2/2`, and that is not an `f64` — so written that way
//! the quarter circle cannot be stored at all.
//!
//! Weights `(w0, w1, w2)` and `(w0·α², w1·α, w2)` draw the same set of points,
//! traced under a different parametrization, because a rational curve depends on
//! its weights only through their ratios. Taking `α = √2` turns `(1, √2/2, 1)`
//! into `(2, 1, 1)`, and reading that from the other end gives `(1, 1, 2)` —
//! every number a whole one. So the quarter circle of radius `r` with control
//! points `(r,0)`, `(r,r)`, `(0,r)` and weights `1, 1, 2` **is** the arc, not an
//! approximation of it. At `u = 1/2` the Bernstein values are `1/4, 1/2, 1/4`,
//! the numerator is `(3r/4, r)`, the denominator `5/4`, and the point is
//! `(3r/5, 4r/5)` — exactly on the circle of radius `r`, and for `r = 1` neither
//! coordinate is an `f64`. That last part is what makes these curves the
//! acceptance oracle's load-bearing case (`docs/decisions.md` §14.4, §14.7).
//!
//! # Four of them make one curve
//!
//! Consecutive quarters can be rescaled to agree on the weight of the control
//! point they share, so the whole circle is **one** clamped degree-2 curve:
//!
//! - nine control points `(r,0)`, `(r,r)`, `(0,r)`, `(-r,r)`, `(-r,0)`,
//!   `(-r,-r)`, `(0,-r)`, `(r,-r)`, `(r,0)`, the last the same place as the
//!   first,
//! - weights `1, 1, 2, 2, 4, 4, 8, 8, 16`,
//! - knots `[0,0,0, 1,1, 2,2, 3,3, 4,4,4]`.
//!
//! Every interior knot repeats exactly `degree` times, so each span reaches only
//! its own three control points, and on each span the three weights are a
//! multiple of `1, 1, 2`. The climb to sixteen is how the parametrization is
//! written, not a loss of accuracy.
//!
//! # It is not periodic, and must not be built as if it were
//!
//! Both clamps are `degree + 1` long and the two ends merely happen to land on
//! the same place. Nothing here closes the curve up, and
//! [`KnotVector`](crate::KnotVector) would refuse a periodic knot vector anyway
//! — periodic NURBS first arrive with a STEP file at M5.
//!
//! # Where the exactness stops
//!
//! Two things can round, and neither is hidden:
//!
//! 1. **The plane the circle is drawn in.** The two radii at right angles are
//!    worked out from the axis direction with a cross product and a division by
//!    a length. When the axis points exactly along a coordinate direction those
//!    come out as `±1` and `0` with nothing rounded, so the control points are
//!    exactly `centre ± r` and `centre`. For any other orientation the division
//!    rounds, and the result is a curve a rounding away from the circle that was
//!    asked for.
//! 2. **The centre.** A control point is `centre ± r`, and that addition rounds
//!    like any other. It does not round when the centre is the origin, which is
//!    the case the oracles are built on.
//!
//! So: exact for an axis along `x`, `y` or `z` through a centre that adding the
//! radius to costs nothing, and correct-but-rounded otherwise. What holds in
//! every case, whatever the orientation, is that the curve passes through its
//! own control points bit for bit at the four quarter joins, because there the
//! de Boor coefficients are exactly zero and one.
//!
//! # Arcs of a general angle are not here
//!
//! `cos θ` is an `f64` for almost no angle, so a general arc constructor cannot
//! be exact, and an inexact one has no consumer in this milestone
//! (`docs/decisions.md` §13).

use crate::bspline_curve::BSplineCurve;
use crate::knot_vector::KnotVector;
use crate::point::{Axis, Vector};
use crate::{Error, Result};

impl BSplineCurve {
    /// The whole circle of the given radius around `axis.origin()`, lying in the
    /// plane `axis` is normal to.
    ///
    /// One clamped degree-2 curve over `[0, 4]`, one unit of parameter per
    /// quarter, running right-handed about the axis. Where it starts is this
    /// constructor's choice: `+x` for an axis along `z`, `+y` for one along `x`,
    /// and `+x` for one along `y`. The module note says what is exact about the
    /// result and what is not.
    ///
    /// # Errors
    ///
    /// [`Error::RadiusNotPositive`] if `radius` is zero, negative, or not a
    /// finite number, and [`Error::NotFinite`] if the centre and the radius are
    /// each finite but adding them overflows.
    pub fn circle(axis: Axis, radius: f64) -> Result<Self> {
        let (to_start, a_quarter_on) = two_radii_at_right_angles(axis, radius)?;
        let centre = axis.origin();
        let corners = [
            centre + to_start,
            centre + to_start + a_quarter_on,
            centre + a_quarter_on,
            centre - to_start + a_quarter_on,
            centre - to_start,
            centre - to_start - a_quarter_on,
            centre - a_quarter_on,
            centre + to_start - a_quarter_on,
            centre + to_start,
        ];

        let knots = KnotVector::new(
            2,
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0],
        )
        .expect("clamped at both ends and every interior knot repeated exactly twice");
        Self::new(
            knots,
            &corners,
            &[1.0, 1.0, 2.0, 2.0, 4.0, 4.0, 8.0, 8.0, 16.0],
        )
    }

    /// The first quarter of [`BSplineCurve::circle`], on its own, over `[0, 1]`.
    ///
    /// # Errors
    ///
    /// The same two [`BSplineCurve::circle`] has.
    pub fn quarter_arc(axis: Axis, radius: f64) -> Result<Self> {
        let (to_start, a_quarter_on) = two_radii_at_right_angles(axis, radius)?;
        let centre = axis.origin();
        let corners = [
            centre + to_start,
            centre + to_start + a_quarter_on,
            centre + a_quarter_on,
        ];

        let knots = KnotVector::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0])
            .expect("a clamped single-span quadratic");
        Self::new(knots, &corners, &[1.0, 1.0, 2.0])
    }
}

/// The radius out to where the circle starts, and the radius a quarter turn on
/// from it, both already scaled by `radius`.
///
/// The two turn right-handed about the axis: the first crossed into the second
/// gives the axis direction back.
///
/// Which direction the circle starts in is this function's to choose, and it
/// chooses so that the common cases cost nothing. The coordinate direction the
/// axis leans on least is crossed into the axis and the result scaled to length
/// one; for an axis along `z` that is `+x` and `+y`, along `x` it is `+y` and
/// `+z`, and along `y` it is `+x` and `−z`. In each of those the cross product
/// already has length exactly one, so the scaling divides by `1.0` and rounds
/// nothing.
///
/// # Errors
///
/// [`Error::RadiusNotPositive`] for a radius that is zero, negative, or not a
/// finite number. A circle of no radius is a point, and every control point of
/// it would sit on the centre — a curve the kernel would happily store and
/// nothing could use.
///
/// # Panics
///
/// If the cross product has no length to scale. The axis direction has length
/// one and the coordinate direction chosen is the one it leans on least, so the
/// two are never closer than 55 degrees apart and the cross product is never
/// shorter than `√(2/3)`. Reaching this means the axis was already broken.
fn two_radii_at_right_angles(axis: Axis, radius: f64) -> Result<(Vector, Vector)> {
    if !(radius.is_finite() && radius > 0.0) {
        return Err(Error::RadiusNotPositive { radius });
    }

    let direction = axis.direction();
    let a_quarter_on = direction
        .cross(the_coordinate_direction_it_leans_on_least(direction))
        .normalized_as("circle plane")
        .expect("a unit axis is never parallel to the coordinate direction it leans on least");
    let to_start = a_quarter_on.cross(direction);

    Ok((to_start * radius, a_quarter_on * radius))
}

/// Whichever of `x`, `y` and `z` `direction` has the smallest component along,
/// ties going to the earlier one.
fn the_coordinate_direction_it_leans_on_least(direction: Vector) -> Vector {
    let [along_x, along_y, along_z] = direction.components().map(f64::abs);
    if along_x <= along_y && along_x <= along_z {
        Vector::new(1.0, 0.0, 0.0)
    } else if along_y <= along_z {
        Vector::new(0.0, 1.0, 0.0)
    } else {
        Vector::new(0.0, 0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point::Point;

    /// The point at the far end of a radius, for writing an expected net out.
    fn from_centre(centre: Point, offset: [f64; 3]) -> Point {
        let [x, y, z] = offset;
        Point::new(centre.x() + x, centre.y() + y, centre.z() + z)
    }

    #[test]
    fn a_radius_that_is_not_a_positive_number_is_refused() {
        for radius in [0.0, -1.0, -0.0] {
            assert_eq!(
                BSplineCurve::circle(Axis::z(), radius),
                Err(Error::RadiusNotPositive { radius })
            );
            assert_eq!(
                BSplineCurve::quarter_arc(Axis::z(), radius),
                Err(Error::RadiusNotPositive { radius })
            );
        }
    }

    /// The refused radius is compared bit for bit, because a `NaN` handed back
    /// as a `NaN` does not compare equal to itself.
    #[test]
    fn a_radius_that_is_not_a_finite_number_is_refused() {
        for radius in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for built in [
                BSplineCurve::circle(Axis::z(), radius),
                BSplineCurve::quarter_arc(Axis::z(), radius),
            ] {
                match built {
                    Err(Error::RadiusNotPositive { radius: refused }) => {
                        assert_eq!(refused.to_bits(), radius.to_bits());
                    }
                    other => panic!("a radius of {radius} does not name a circle: {other:?}"),
                }
            }
        }
    }

    /// A centre and a radius that are each finite and whose sum is not. The
    /// control points would be infinite, so the curve is refused rather than
    /// stored.
    #[test]
    fn a_circle_too_far_out_to_add_its_radius_to_is_refused() {
        let far_out = Axis::new(Point::new(1e308, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
            .expect("a finite origin and a direction with length");
        assert_eq!(
            BSplineCurve::circle(far_out, 1e308),
            Err(Error::NotFinite {
                what: "spline control point"
            })
        );
    }

    #[test]
    fn a_circle_is_one_clamped_quadratic_over_four_units_of_parameter() {
        let circle = BSplineCurve::circle(Axis::z(), 3.0).unwrap();
        assert_eq!(circle.degree(), 2);
        assert_eq!(circle.control_point_count(), 9);
        assert_eq!(circle.domain(), (0.0, 4.0));
        assert_eq!(
            circle.knots().knots(),
            &[0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0]
        );
        for knot in [1.0, 2.0, 3.0] {
            assert_eq!(
                circle.knots().multiplicity_of(knot),
                2,
                "each quarter join repeats exactly as often as the degree"
            );
        }
    }

    /// The whole net, written out. Every weight is a power of two, so dividing
    /// it back out of the control point is exact and the affine points can be
    /// asserted directly.
    #[test]
    fn a_circle_about_z_has_the_net_the_module_note_describes() {
        let radius = 3.0;
        let circle = BSplineCurve::circle(Axis::z(), radius).unwrap();
        let corners = [
            [radius, 0.0, 0.0],
            [radius, radius, 0.0],
            [0.0, radius, 0.0],
            [-radius, radius, 0.0],
            [-radius, 0.0, 0.0],
            [-radius, -radius, 0.0],
            [0.0, -radius, 0.0],
            [radius, -radius, 0.0],
            [radius, 0.0, 0.0],
        ];
        let weights = [1.0, 1.0, 2.0, 2.0, 4.0, 4.0, 8.0, 8.0, 16.0];

        for (i, corner) in corners.iter().enumerate() {
            assert_eq!(
                circle.control_point(i),
                from_centre(Point::origin(), *corner)
            );
            assert_eq!(circle.weight(i), weights[i]);
        }
    }

    /// The three axes and the three reversed ones. Each is a coordinate
    /// direction, so the plane costs no rounding and the whole net is whole
    /// numbers — which is what `assert_eq` on the control points is claiming.
    #[test]
    fn a_circle_about_any_coordinate_axis_has_an_exact_net() {
        let radius = 5.0;
        for axis in [
            Axis::x(),
            Axis::y(),
            Axis::z(),
            Axis::x().reversed(),
            Axis::y().reversed(),
            Axis::z().reversed(),
        ] {
            let circle = BSplineCurve::circle(axis, radius).unwrap();
            for i in 0..circle.control_point_count() {
                let corner = circle.control_point(i);
                for coordinate in corner.coordinates() {
                    assert!(
                        [0.0, radius, -radius].contains(&coordinate),
                        "about {axis:?} control point {i} is at {corner:?}, which is not made of \
                         zero and the radius"
                    );
                }
            }
        }
    }

    /// The circle runs the way a right hand does about the axis: out to the
    /// start, then a quarter turn on, crosses back to the axis direction.
    #[test]
    fn a_circle_turns_right_handed_about_its_axis() {
        for axis in [Axis::x(), Axis::y(), Axis::z(), Axis::z().reversed()] {
            let (to_start, a_quarter_on) = two_radii_at_right_angles(axis, 1.0).unwrap();
            assert_eq!(to_start.cross(a_quarter_on), axis.direction());
        }
    }

    /// A slanted axis is not one of the exact cases, and the module note says
    /// so. What still holds is that the curve is built and that the two radii
    /// are the same length and at right angles to the axis to within the last
    /// few bits — asserted against the size of the numbers, not against a
    /// tolerance anyone chose.
    #[test]
    fn a_slanted_axis_still_gives_two_radii_of_the_same_length_at_right_angles() {
        let slanted = Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0))
            .expect("a direction with length");
        let (to_start, a_quarter_on) = two_radii_at_right_angles(slanted, 1.0).unwrap();

        let a_few_last_bits = 8.0 * f64::EPSILON;
        assert!((to_start.length() - 1.0).abs() <= a_few_last_bits);
        assert!((a_quarter_on.length() - 1.0).abs() <= a_few_last_bits);
        assert!(to_start.dot(slanted.direction()).abs() <= a_few_last_bits);
        assert!(a_quarter_on.dot(slanted.direction()).abs() <= a_few_last_bits);
        assert!(to_start.dot(a_quarter_on).abs() <= a_few_last_bits);
    }

    #[test]
    fn a_quarter_arc_is_the_first_quarter_of_the_whole_circle() {
        let arc = BSplineCurve::quarter_arc(Axis::z(), 2.0).unwrap();
        let circle = BSplineCurve::circle(Axis::z(), 2.0).unwrap();

        assert_eq!(arc.domain(), (0.0, 1.0));
        assert_eq!(arc.control_point_count(), 3);
        for i in 0..3 {
            let [x, y, z, weight] = arc.homogeneous_control_point(i);
            let [cx, cy, cz, cw] = circle.homogeneous_control_point(i);
            assert_eq!([x, y, z, weight], [cx, cy, cz, cw]);
        }
    }

    /// The centre away from the origin: the plane is still exact, and the
    /// control points are still whole numbers because two and three add to a
    /// whole number. The module note is careful not to promise this for a
    /// centre that rounds.
    #[test]
    fn a_circle_around_a_centre_off_the_origin_sits_around_that_centre() {
        let centre = Point::new(2.0, -4.0, 7.0);
        let axis = Axis::new(centre, Vector::new(0.0, 0.0, 1.0)).expect("a direction with length");
        let circle = BSplineCurve::circle(axis, 3.0).unwrap();

        assert_eq!(
            circle.control_point(0),
            from_centre(centre, [3.0, 0.0, 0.0])
        );
        assert_eq!(
            circle.control_point(2),
            from_centre(centre, [0.0, 3.0, 0.0])
        );
        assert_eq!(
            circle.control_point(4),
            from_centre(centre, [-3.0, 0.0, 0.0])
        );
        assert_eq!(
            circle.control_point(6),
            from_centre(centre, [0.0, -3.0, 0.0])
        );
    }
}
