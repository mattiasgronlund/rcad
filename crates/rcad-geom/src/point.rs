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

    /// The same direction, scaled to length one, for a vector whose own squared
    /// length is not a number `f64` can hold.
    ///
    /// [`Vector::normalized`] divides by `√(x² + y² + z²)`, and that squaring is
    /// where it stops working: a direction whose components are around `1e-180`
    /// squares into the subnormals and back out again as noise, so what comes
    /// back is not a unit vector at all — it is merely near one, and the caller
    /// has no way to tell.
    ///
    /// This divides by the largest component first and then normalizes what is
    /// left. Every component is then in `[-1, 1]` with the largest exactly `±1`,
    /// so the squared length is between one and three and nothing underflows. A
    /// **gap** closed rather than a limit met (`docs/decisions.md` §16.10).
    ///
    /// **The first division is not exact**, and the doc here used to say it was.
    /// `a/|m|` is one correctly rounded division and lands on a different real
    /// number than `a` did; what is exact is that the largest component comes
    /// out as `±1`. What that first division *does* buy is that the answer
    /// depends on the ratios between the components and on nothing else, so two
    /// vectors along one line give the same direction however long each of them
    /// was — which is what [`Axis::new`] needs and §22.6 is about.
    ///
    /// It is not [`Vector::normalized`] itself, because that one is called on
    /// vectors that are already directions and dividing twice rounds differently
    /// from dividing once.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if any component is infinite or not a number, and
    /// [`Error::ZeroLength`] if every component is exactly zero.
    pub(crate) fn scaled_to_length_one_from_anywhere(&self, what: &'static str) -> Result<Vector> {
        if !self.is_finite() {
            return Err(Error::NotFinite { what });
        }
        let [x, y, z] = self.components();
        let widest = x.abs().max(y.abs()).max(z.abs());
        if widest == 0.0 {
            return Err(Error::ZeroLength { what });
        }
        Vector::new(x / widest, y / widest, z / widest).normalized_as(what)
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
/// The constructor scales the direction to length one and rejects what it
/// cannot scale; [`Axis::transformed`] turns it without scaling it again, so a
/// moved axis points along a direction of length one to within the rounding of
/// the turn. Serializes as `(origin, direction)` and is re-checked on the way
/// back in, so a file cannot hand us an axis without a direction.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "(Point, Vector)", into = "(Point, Vector)")]
pub struct Axis {
    origin: Point,
    direction: Vector,
}

impl Axis {
    /// The line through `origin` pointing along `direction`.
    ///
    /// Only the direction of `direction` is read: two vectors along the same
    /// line give the same axis bit for bit however long each one is. That takes
    /// dividing by the largest component before normalizing, which is what
    /// [`Vector::scaled_to_length_one_from_anywhere`] does and what plain
    /// normalizing does not — `docs/decisions.md` §22.6 recorded the same defect
    /// for `Plane` and this one went unfixed because nothing built the same pipe
    /// two ways. A curved fuse does, the moment two solids are flush against
    /// each other on one cylinder, and then a pair of surfaces that are the same
    /// pipe comes back as two pipes crossing.
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
            direction: direction.scaled_to_length_one_from_anywhere("axis direction")?,
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

    /// Which way the line points. Scaled to length one when the axis was built.
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
    /// The moved direction is not scaled back to unit length. It arrived unit
    /// from [`Axis::new`] and a turn does not change a length, so scaling it
    /// again is the second scaling of an already-scaled vector — and a stored
    /// direction whose computed length is `0.9999999999999999` would grow a
    /// little on every read. `Cylinder::transformed` is how a pipe is read out
    /// of a model, at whatever location the face carries, so a read that moves
    /// bits is a read nobody can do twice. `Plane::transformed` says the same
    /// thing at more length.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        Self {
            origin: movement.apply_to_point(self.origin),
            direction: movement.apply_to_vector(self.direction),
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

    /// §22.6's defect, at the type it was left on.
    ///
    /// Four directions along one line, none of them a power-of-two multiple of
    /// the next, and the axis has to store the same numbers every time. `Plane`
    /// got this in §22.6 and `Axis` did not, because nothing built the same pipe
    /// two ways until a curved fuse did.
    #[test]
    fn an_axis_is_the_same_however_long_the_direction_naming_it_was() {
        let axis_along = |x: f64, y: f64, z: f64| {
            Axis::new(Point::origin(), Vector::new(x, y, z))
                .expect("a direction with length")
                .direction()
        };

        // Whole numbers and powers of two, so that the longer triple really is
        // the shorter one times the multiple and not a rounding of it. A
        // multiple like `1e-17` is not one: `2 * 1e-17` rounds, so the pair
        // would name two directions and this would be testing the generator.
        for multiple in [2.0, 3.0, 5.0, 7.0, 11.0, 2f64.powi(60), 2f64.powi(-60)] {
            assert_eq!(
                axis_along(1.0, 2.0, 3.0),
                axis_along(multiple, 2.0 * multiple, 3.0 * multiple),
                "the direction 1:2:3, named {multiple} times over"
            );
        }
    }

    /// The measurement behind that fix, over every direction a small lattice
    /// holds.
    ///
    /// Both arithmetics are still here, which is what makes this a comparison
    /// rather than a memory: [`Vector::normalized`] is what [`Axis::new`] used
    /// to call and [`Vector::scaled_to_length_one_from_anywhere`] is what it
    /// calls now. Every integer triple in `-6..=6` that has a length, against
    /// every whole-number multiple of it from two to ninety-seven — two hundred
    /// and ten thousand pairs that name one direction twice.
    ///
    /// Dividing by the largest component first makes what comes out depend on
    /// the ratios between the components and on nothing else, and the ratios of
    /// two multiples of one vector are the same real numbers, so they round the
    /// same way. Normalizing alone divides by two lengths that are not multiples
    /// of each other in `f64`, and **38.1% of these pairs land on two different
    /// directions** because of it.
    #[test]
    fn normalizing_alone_stores_a_different_direction_and_reducing_first_does_not() {
        let mut drawn = 0usize;
        let mut normalizing_alone_differed = 0usize;
        let mut reducing_first_differed = 0usize;

        for x in -6i32..=6 {
            for y in -6i32..=6 {
                for z in -6i32..=6 {
                    let short = Vector::new(f64::from(x), f64::from(y), f64::from(z));
                    if short == Vector::zero() {
                        continue;
                    }
                    for scale in 2..=97 {
                        let long = short * f64::from(scale);
                        drawn += 1;
                        if short.normalized() != long.normalized() {
                            normalizing_alone_differed += 1;
                        }
                        if short.scaled_to_length_one_from_anywhere("direction")
                            != long.scaled_to_length_one_from_anywhere("direction")
                        {
                            reducing_first_differed += 1;
                        }
                    }
                }
            }
        }

        println!(
            "{drawn} pairs of proportional directions: normalizing alone stored different bits \
             for {normalizing_alone_differed} of them ({:.1}%), reducing first for \
             {reducing_first_differed}",
            100.0 * normalizing_alone_differed as f64 / drawn as f64
        );
        assert!(
            drawn >= 200_000,
            "only {drawn} pairs, which is not the measurement this claims to be"
        );
        assert!(
            normalizing_alone_differed * 4 > drawn,
            "normalizing alone is meant to store a different direction for a large share of \
             these, and here it was a different one only {normalizing_alone_differed} times out \
             of {drawn}"
        );
        assert_eq!(
            reducing_first_differed, 0,
            "reducing by the largest component makes the answer a function of the ratios, and a \
             ratio does not care how long the vector was"
        );
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
