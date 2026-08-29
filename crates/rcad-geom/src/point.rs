//! Points, vectors and axes.
//!
//! These are newtypes over `nalgebra`, which is used freely inside them and
//! never appears in a signature. `nalgebra` is at 0.35 and breaks its API
//! regularly; if `nalgebra::Point3<f64>` were part of our public API then every
//! bump of it would be a breaking release for everyone downstream
//! (`docs/decisions.md` §3.4).

use core::ops::{Add, Mul, Neg, Sub};

use nalgebra as na;
use serde::{Deserialize, Serialize};

use crate::isometry::Isometry;
use crate::{Error, Result};

/// A position in space, in the model's declared [`Unit`](crate::Unit).
///
/// Plain data. A point with a non-finite coordinate is constructible on
/// purpose — rejecting it here would put a `Result` on the most common call in
/// the kernel. The types that build real geometry out of points check
/// [`Point::is_finite`] and refuse.
///
/// Serializes as `[x, y, z]`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(from = "[f64; 3]", into = "[f64; 3]")]
pub struct Point(na::Point3<f64>);

impl Point {
    /// The point at the given coordinates.
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(na::Point3::new(x, y, z))
    }

    /// The point where all three coordinates are zero.
    #[must_use]
    pub fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// The first coordinate.
    #[must_use]
    pub fn x(&self) -> f64 {
        self.0.x
    }

    /// The second coordinate.
    #[must_use]
    pub fn y(&self) -> f64 {
        self.0.y
    }

    /// The third coordinate.
    #[must_use]
    pub fn z(&self) -> f64 {
        self.0.z
    }

    /// All three coordinates, in order.
    #[must_use]
    pub fn coordinates(&self) -> [f64; 3] {
        [self.x(), self.y(), self.z()]
    }

    /// Whether every coordinate is a finite number.
    #[must_use]
    pub fn is_finite(&self) -> bool {
        self.x().is_finite() && self.y().is_finite() && self.z().is_finite()
    }

    /// The largest of the three coordinates, ignoring sign.
    ///
    /// This is the numerator of the dynamic-range check,
    /// [`DynamicRange`](crate::DynamicRange).
    #[must_use]
    pub fn largest_coordinate_magnitude(&self) -> f64 {
        self.x().abs().max(self.y().abs()).max(self.z().abs())
    }

    /// How far it is from here to `other`.
    #[must_use]
    pub fn distance_to(&self, other: Point) -> f64 {
        (other - *self).length()
    }

    /// The square of [`Point::distance_to`], which avoids a square root.
    #[must_use]
    pub fn distance_squared_to(&self, other: Point) -> f64 {
        (other - *self).length_squared()
    }
}

impl Sub<Point> for Point {
    type Output = Vector;

    fn sub(self, start: Point) -> Vector {
        Vector(self.0 - start.0)
    }
}

impl Add<Vector> for Point {
    type Output = Point;

    fn add(self, offset: Vector) -> Point {
        Point(self.0 + offset.0)
    }
}

impl Sub<Vector> for Point {
    type Output = Point;

    fn sub(self, offset: Vector) -> Point {
        Point(self.0 - offset.0)
    }
}

impl From<[f64; 3]> for Point {
    fn from([x, y, z]: [f64; 3]) -> Self {
        Self::new(x, y, z)
    }
}

impl From<Point> for [f64; 3] {
    fn from(point: Point) -> Self {
        point.coordinates()
    }
}

/// A point read out of a model is exact as written, so the predicates get to
/// use all three levels of the cascade on it. Only a point that came out of a
/// construction is a box, and those are built by the predicate crate itself.
impl From<Point> for rcad_predicates::Point {
    fn from(point: Point) -> Self {
        rcad_predicates::Point::Exact(point.coordinates())
    }
}

impl From<na::Point3<f64>> for Point {
    fn from(point: na::Point3<f64>) -> Self {
        Self(point)
    }
}

impl From<Point> for na::Point3<f64> {
    fn from(point: Point) -> Self {
        point.0
    }
}

/// A direction and a length: the difference between two [`Point`]s.
///
/// Plain data, like [`Point`]. Serializes as `[x, y, z]`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(from = "[f64; 3]", into = "[f64; 3]")]
pub struct Vector(na::Vector3<f64>);

impl Vector {
    /// The vector with the given components.
    #[must_use]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self(na::Vector3::new(x, y, z))
    }

    /// The vector of length zero.
    #[must_use]
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// The first component.
    #[must_use]
    pub fn x(&self) -> f64 {
        self.0.x
    }

    /// The second component.
    #[must_use]
    pub fn y(&self) -> f64 {
        self.0.y
    }

    /// The third component.
    #[must_use]
    pub fn z(&self) -> f64 {
        self.0.z
    }

    /// All three components, in order.
    #[must_use]
    pub fn components(&self) -> [f64; 3] {
        [self.x(), self.y(), self.z()]
    }

    /// Whether every component is a finite number.
    #[must_use]
    pub fn is_finite(&self) -> bool {
        self.x().is_finite() && self.y().is_finite() && self.z().is_finite()
    }

    /// How long the vector is.
    #[must_use]
    pub fn length(&self) -> f64 {
        self.0.norm()
    }

    /// The square of [`Vector::length`], which avoids a square root.
    #[must_use]
    pub fn length_squared(&self) -> f64 {
        self.0.norm_squared()
    }

    /// The dot product: `length(self) * length(other) * cos(angle between)`.
    #[must_use]
    pub fn dot(&self, other: Vector) -> f64 {
        self.0.dot(&other.0)
    }

    /// The cross product: a vector at right angles to both, as long as the area
    /// of the parallelogram the two of them span.
    #[must_use]
    pub fn cross(&self, other: Vector) -> Vector {
        Self(self.0.cross(&other.0))
    }

    /// The same direction, scaled to length one.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if any component is infinite or not a number, and
    /// [`Error::ZeroLength`] if the vector is too short to say which way it
    /// points — including the case where the length underflows to zero even
    /// though no single component is exactly zero.
    pub fn normalized(&self) -> Result<Vector> {
        self.normalized_as("vector")
    }

    pub(crate) fn normalized_as(&self, what: &'static str) -> Result<Vector> {
        if !self.is_finite() {
            return Err(Error::NotFinite { what });
        }
        let length = self.length();
        if length == 0.0 {
            return Err(Error::ZeroLength { what });
        }
        Ok(Self(self.0 / length))
    }

    /// The same vector with each component rounded to the nearest multiple of
    /// two raised to `grid_exponent`.
    ///
    /// Dividing and multiplying by a power of two is exact, so the components
    /// that come out have zeros in their low mantissa bits. That is what makes
    /// a snapped translation cheap to undo without drift.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if any component is infinite or not a number, if
    /// the grid spacing itself overflows or underflows the `f64` range, or if
    /// rounding pushes a component past the largest `f64`.
    pub fn snapped_to_power_of_two_grid(&self, grid_exponent: i32) -> Result<Vector> {
        if !self.is_finite() {
            return Err(Error::NotFinite {
                what: "vector being snapped to a grid",
            });
        }
        let spacing = f64::from(grid_exponent).exp2();
        if !spacing.is_finite() || spacing == 0.0 {
            return Err(Error::NotFinite {
                what: "power-of-two grid spacing",
            });
        }
        let snapped = Self::new(
            (self.x() / spacing).round() * spacing,
            (self.y() / spacing).round() * spacing,
            (self.z() / spacing).round() * spacing,
        );
        if snapped.is_finite() {
            Ok(snapped)
        } else {
            Err(Error::NotFinite {
                what: "vector snapped to a grid",
            })
        }
    }
}

impl Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Vector {
        Self(self.0 + other.0)
    }
}

impl Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Vector {
        Self(self.0 - other.0)
    }
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Vector {
        Self(-self.0)
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, factor: f64) -> Vector {
        Self(self.0 * factor)
    }
}

impl From<[f64; 3]> for Vector {
    fn from([x, y, z]: [f64; 3]) -> Self {
        Self::new(x, y, z)
    }
}

impl From<Vector> for [f64; 3] {
    fn from(vector: Vector) -> Self {
        vector.components()
    }
}

impl From<na::Vector3<f64>> for Vector {
    fn from(vector: na::Vector3<f64>) -> Self {
        Self(vector)
    }
}

impl From<Vector> for na::Vector3<f64> {
    fn from(vector: Vector) -> Self {
        vector.0
    }
}

/// A straight line through a point, with a direction along it.
///
/// The direction always has length one; the constructor scales it and rejects
/// what it cannot scale. Serializes as `(origin, direction)` and is re-checked
/// on the way back in, so a file cannot hand us an axis without a direction.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "(Point, Vector)", into = "(Point, Vector)")]
pub struct Axis {
    origin: Point,
    direction: Vector,
}

impl Axis {
    /// The line through `origin` pointing along `direction`.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `origin` or `direction` has a coordinate that is
    /// not a finite number, and [`Error::ZeroLength`] if `direction` has no
    /// length to take a direction from.
    pub fn new(origin: Point, direction: Vector) -> Result<Self> {
        if !origin.is_finite() {
            return Err(Error::NotFinite {
                what: "axis origin",
            });
        }
        Ok(Self {
            origin,
            direction: direction.normalized_as("axis direction")?,
        })
    }

    /// The first coordinate axis: through the origin, pointing along `x`.
    #[must_use]
    pub fn x() -> Self {
        Self {
            origin: Point::origin(),
            direction: Vector::new(1.0, 0.0, 0.0),
        }
    }

    /// The second coordinate axis: through the origin, pointing along `y`.
    #[must_use]
    pub fn y() -> Self {
        Self {
            origin: Point::origin(),
            direction: Vector::new(0.0, 1.0, 0.0),
        }
    }

    /// The third coordinate axis: through the origin, pointing along `z`.
    #[must_use]
    pub fn z() -> Self {
        Self {
            origin: Point::origin(),
            direction: Vector::new(0.0, 0.0, 1.0),
        }
    }

    /// A point the line passes through.
    #[must_use]
    pub fn origin(&self) -> Point {
        self.origin
    }

    /// Which way the line points. Always has length one.
    #[must_use]
    pub fn direction(&self) -> Vector {
        self.direction
    }

    /// The same line pointing the other way.
    #[must_use]
    pub fn reversed(&self) -> Self {
        Self {
            origin: self.origin,
            direction: -self.direction,
        }
    }

    /// The point on the line nearest to `point`.
    #[must_use]
    pub fn closest_point(&self, point: Point) -> Point {
        self.origin + self.direction * self.direction.dot(point - self.origin)
    }

    /// How far `point` is from the line.
    #[must_use]
    pub fn distance_to_point(&self, point: Point) -> f64 {
        self.closest_point(point).distance_to(point)
    }

    /// The same line after `movement` is applied.
    ///
    /// # Panics
    ///
    /// If the moved direction no longer has any length. A rotation cannot do
    /// that, so reaching this means the axis or the isometry was already broken.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        Self {
            origin: movement.apply_to_point(self.origin),
            direction: movement
                .apply_to_vector(self.direction)
                .normalized_as("moved axis direction")
                .expect("a rotation cannot turn a direction into a zero-length vector"),
        }
    }
}

impl TryFrom<(Point, Vector)> for Axis {
    type Error = Error;

    fn try_from((origin, direction): (Point, Vector)) -> Result<Self> {
        Self::new(origin, direction)
    }
}

impl From<Axis> for (Point, Vector) {
    fn from(axis: Axis) -> Self {
        (axis.origin, axis.direction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtracting_two_points_gives_the_step_from_one_to_the_other() {
        let step = Point::new(4.0, 6.0, 9.0) - Point::new(1.0, 2.0, 3.0);
        assert_eq!(step, Vector::new(3.0, 4.0, 6.0));
    }

    #[test]
    fn adding_the_step_back_onto_the_start_lands_on_the_end() {
        let start = Point::new(1.0, 2.0, 3.0);
        let end = Point::new(4.0, 6.0, 9.0);
        assert_eq!(start + (end - start), end);
    }

    #[test]
    fn the_cross_product_is_at_right_angles_to_both_inputs() {
        let a = Vector::new(1.0, 2.0, 3.0);
        let b = Vector::new(-4.0, 5.0, 6.0);
        let across = a.cross(b);
        assert!(across.dot(a).abs() < 1e-12);
        assert!(across.dot(b).abs() < 1e-12);
    }

    #[test]
    fn a_vector_with_no_length_has_no_direction_to_report() {
        assert_eq!(
            Vector::zero().normalized(),
            Err(Error::ZeroLength { what: "vector" })
        );
    }

    #[test]
    fn a_vector_with_a_component_that_is_not_a_number_cannot_be_normalized() {
        assert_eq!(
            Vector::new(1.0, f64::NAN, 0.0).normalized(),
            Err(Error::NotFinite { what: "vector" })
        );
    }

    #[test]
    fn normalizing_keeps_the_direction_and_sets_the_length_to_one() {
        let normalized = Vector::new(0.0, 3.0, 4.0).normalized().unwrap();
        assert!((normalized.length() - 1.0).abs() < 1e-15);
        assert_eq!(normalized, Vector::new(0.0, 0.6, 0.8));
    }

    #[test]
    fn snapping_to_a_grid_of_one_quarter_rounds_to_the_nearest_quarter() {
        let snapped = Vector::new(1.3, -2.6, 0.1)
            .snapped_to_power_of_two_grid(-2)
            .unwrap();
        assert_eq!(snapped, Vector::new(1.25, -2.5, 0.0));
    }

    #[test]
    fn a_value_already_on_the_grid_is_left_exactly_where_it_is() {
        let already_there = Vector::new(1024.0, -0.5, 3.75);
        assert_eq!(
            already_there.snapped_to_power_of_two_grid(-2).unwrap(),
            already_there
        );
    }

    #[test]
    fn the_largest_coordinate_ignores_sign() {
        assert_eq!(
            Point::new(1.0, -7.5, 3.0).largest_coordinate_magnitude(),
            7.5
        );
    }

    #[test]
    fn an_axis_needs_a_direction_that_has_some_length() {
        assert_eq!(
            Axis::new(Point::origin(), Vector::zero()),
            Err(Error::ZeroLength {
                what: "axis direction"
            })
        );
    }

    #[test]
    fn an_axis_scales_the_direction_it_is_given_to_length_one() {
        let axis = Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(0.0, 0.0, 5.0)).unwrap();
        assert_eq!(axis.direction(), Vector::new(0.0, 0.0, 1.0));
        assert_eq!(axis.origin(), Point::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn the_closest_point_on_an_axis_is_the_foot_of_the_perpendicular() {
        let axis = Axis::z();
        let point = Point::new(3.0, 4.0, 7.0);
        assert_eq!(axis.closest_point(point), Point::new(0.0, 0.0, 7.0));
        assert!((axis.distance_to_point(point) - 5.0).abs() < 1e-12);
    }

    #[test]
    fn reversing_an_axis_flips_its_direction_and_keeps_its_origin() {
        let axis = Axis::new(Point::new(1.0, 2.0, 3.0), Vector::new(1.0, 0.0, 0.0)).unwrap();
        let back = axis.reversed();
        assert_eq!(back.direction(), Vector::new(-1.0, 0.0, 0.0));
        assert_eq!(back.origin(), axis.origin());
    }

    #[test]
    fn an_axis_rebuilt_from_stored_parts_without_a_direction_is_refused() {
        let stored = (Point::origin(), Vector::zero());
        assert!(Axis::try_from(stored).is_err());
    }
}
