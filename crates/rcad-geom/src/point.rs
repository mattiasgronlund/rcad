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
    ///
    /// Two points that are not the same point never measure zero apart, however
    /// close together they are, and two points that are very far apart measure
    /// a number rather than an infinity — see [`Vector::length`], which is where
    /// both of those used to go wrong.
    ///
    /// What it cannot do is measure a separation the coordinates never held:
    /// two points a hair apart at `1e300` are the same `f64` triple, because the
    /// hair was lost when the coordinates were written down.
    #[must_use]
    pub fn distance_to(&self, other: Point) -> f64 {
        (other - *self).length()
    }

    /// The square of [`Point::distance_to`], which avoids a square root.
    ///
    /// The square of a distance is a different quantity with a different range:
    /// squaring a separation of `1e-200` gives zero and squaring one of `1e200`
    /// gives infinity, and no arithmetic recovers either, because neither answer
    /// is an `f64`. Compare two of these only when both distances are ordinary
    /// sizes; otherwise take [`Point::distance_to`] and compare that.
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
    ///
    /// Adding up the squares and taking the root is the whole of this in the
    /// ordinary range, and outside it the squares are the problem rather than
    /// the answer. A vector `3.8e-211` long squares to `1.4e-421`, under the
    /// smallest `f64` there is, so it came back **exactly zero** — a distinct
    /// pair of points reported as one point, which is the confident wrong
    /// answer §4.1 ranks below both crashing and refusing. A vector `1e200`
    /// long squares to infinity and came back infinite. Both answers are
    /// ordinary `f64`s that the intermediate could not hold, so both are
    /// **gaps** rather than limits (§16.10).
    ///
    /// So when the largest component is too big or too small for its own square
    /// to be a normal `f64`, every component is scaled by a power of two first
    /// and the answer is scaled back by the same power afterwards. Both scalings
    /// are exact, which is what makes the answer the one the unscaled arithmetic
    /// would have given if `f64` had the range for it.
    ///
    /// Inside the ordinary range nothing is scaled and every bit is what it
    /// always was. That is why this is not `f64::hypot`, which is the obvious
    /// instrument and would also have worked: `hypot` is a *different* rounding
    /// of the same real number, up to two ulps from this one over the unit cube,
    /// so it would have moved lengths everywhere in the kernel to fix a range
    /// nothing in the ordinary range was in. It is also a libm call whose
    /// accuracy is the platform's business, and this is three multiplications,
    /// two additions and a root whose accuracy is IEEE 754's.
    ///
    /// The answer is within about two ulps of the true length, which is what the
    /// unscaled arithmetic was always worth and is unchanged by the scaling.
    #[must_use]
    pub fn length(&self) -> f64 {
        match self.power_of_two_that_keeps_the_squares_in_range() {
            None => self.0.norm(),
            Some(scale) => (*self * scale).0.norm() / scale,
        }
    }

    /// A power of two to measure this vector at, or `None` to measure it where
    /// it is.
    ///
    /// The square of the largest component has to be a normal `f64`, which puts
    /// the largest component in `[2^-511, 2^512)`. Outside that, six hundred
    /// doublings either way land it well inside: a largest component of `2^-1074`
    /// becomes `2^-474` and one of `2^1023` becomes `2^423`, and the squares of
    /// both are normal. Components far smaller than the largest may lose bits or
    /// vanish under the scaling down, and cannot matter — one that vanishes was
    /// more than `2^474` times smaller than the largest, so its square is more
    /// than `2^948` times smaller than the largest square.
    ///
    /// `powi` on a base of two is exact: every intermediate in its squaring
    /// chain is a power of two comfortably inside the `f64` range.
    fn power_of_two_that_keeps_the_squares_in_range(&self) -> Option<f64> {
        let widest = self.x().abs().max(self.y().abs()).max(self.z().abs());
        let squared = widest * widest;
        if widest == 0.0 || squared.is_normal() {
            return None;
        }
        if squared.is_finite() {
            Some(2.0f64.powi(600))
        } else {
            Some(2.0f64.powi(-600))
        }
    }

    /// The square of [`Vector::length`], which avoids a square root.
    ///
    /// Unlike [`Vector::length`] this cannot be rescued outside the ordinary
    /// range, and it is not a shortcoming of the arithmetic: the square of a
    /// length of `1e-200` is not an `f64` at all. Use it to compare two lengths
    /// that are both ordinary sizes, and [`Vector::length`] otherwise.
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

    /// The same direction, scaled to length one, and the same bits for every
    /// vector along one line however long it was.
    ///
    /// This divides by the largest component first and then normalizes what is
    /// left. Every component is then in `[-1, 1]` with the largest exactly `±1`,
    /// so the length is between one and the root of three.
    ///
    /// It once earned its place by keeping the squares out of the subnormals,
    /// which [`Vector::length`] now does for every caller. What it still does
    /// and plain normalizing still does not is the part below.
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

    /// Dividing by the length is done at the scale the length was measured at,
    /// not back where the vector started.
    ///
    /// A vector `5e-324` long has a length that is one bit wide, so dividing by
    /// it hands back a "unit" vector of length `√2`. Scaling first and dividing
    /// there gives the direction that does exist. In the ordinary range the
    /// scale is one and nothing moves; outside it, numerator and denominator are
    /// scaled by the same exact power of two, which a division does not notice.
    pub(crate) fn normalized_as(&self, what: &'static str) -> Result<Vector> {
        if !self.is_finite() {
            return Err(Error::NotFinite { what });
        }
        let measured_at = match self.power_of_two_that_keeps_the_squares_in_range() {
            None => *self,
            Some(scale) => *self * scale,
        };
        let length = measured_at.0.norm();
        if length == 0.0 {
            return Err(Error::ZeroLength { what });
        }
        Ok(Self(measured_at.0 / length))
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
///
/// # No stored number is `-0.0`
///
/// Two faces that are the same pipe, cone or bend hold the same six numbers,
/// and the fuse rests on it. [`Axis::holding`] turns every stored zero the
/// right way up and every constructor here goes through it; see it for what a
/// single negative zero costs and for the three ways one arrives.
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
        Ok(Self::holding(
            origin,
            direction.scaled_to_length_one_from_anywhere("axis direction")?,
        ))
    }

    /// The first coordinate axis: through the origin, pointing along `x`.
    #[must_use]
    pub fn x() -> Self {
        Self::holding(Point::origin(), Vector::new(1.0, 0.0, 0.0))
    }

    /// The second coordinate axis: through the origin, pointing along `y`.
    #[must_use]
    pub fn y() -> Self {
        Self::holding(Point::origin(), Vector::new(0.0, 1.0, 0.0))
    }

    /// The third coordinate axis: through the origin, pointing along `z`.
    #[must_use]
    pub fn z() -> Self {
        Self::holding(Point::origin(), Vector::new(0.0, 0.0, 1.0))
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
    ///
    /// **An involution on the bits**, which [`Axis::holding`] is what gives it:
    /// negating alone turns `0.0` into `-0.0`, so a pipe about `z` turned round
    /// would hold other bits than the same pipe named pointing down, and turning
    /// it round twice would not come back to what it started as.
    #[must_use]
    pub fn reversed(&self) -> Self {
        Self::holding(self.origin, -self.direction)
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
        Self::holding(
            movement.apply_to_point(self.origin),
            movement.apply_to_vector(self.direction),
        )
    }

    /// The same line through another point, pointing along the same bits.
    ///
    /// **Scaling is what this is for.** Multiplying a model by `2^k` moves an
    /// origin and leaves a direction alone, because a direction is not a length
    /// — so there is nothing left to work out and nothing that should round.
    /// Going back through [`Axis::new`] instead scales an already-scaled
    /// direction, and that is not the direction that went in: **24 750 of
    /// 100 000** random directions come back with other bits when an axis is
    /// rebuilt from its own. [`Axis::transformed`] refuses the same second
    /// scaling and says more about what it costs.
    ///
    /// That is a footgun and not a broken promise. [`Axis::new`] keeps what it
    /// claims — two vectors *exactly* along one line give the same axis bit for
    /// bit — and a stored direction is the rounded normalizing of whatever
    /// built it, so it lies exactly along nothing.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `origin` has a coordinate that is not a finite
    /// number.
    pub fn moved_to(&self, origin: Point) -> Result<Self> {
        if !origin.is_finite() {
            return Err(Error::NotFinite {
                what: "axis origin",
            });
        }
        Ok(Self::holding(origin, self.direction))
    }

    /// The only place an `Axis` is built, so that no axis anywhere stores
    /// `-0.0`.
    ///
    /// Two faces that are the same pipe, cone or bend have to hold the same
    /// numbers, and four places rest on it: `Surface::how_it_sorts_against` puts
    /// a triple in a fixed order on the axis origin and direction, `rcad-topo`'s
    /// `name_of_surface_bits` names a corner by them, `canonical_cut_surfaces`
    /// drops a repeat by them, and `docs/decisions.md` §32.5 is the rule itself.
    /// All four order with [`f64::total_cmp`](f64::total_cmp), which sorts
    /// `-0.0` before `0.0`, so a single negative zero anywhere in an axis makes
    /// two spellings of one surface two surfaces.
    ///
    /// Three ways in, none of them hypothetical: [`Axis::new`] is public and
    /// [`Vector::scaled_to_length_one_from_anywhere`] divides by two positive
    /// numbers, so a `DIRECTION` written `(-0.,0.,1.)` — which 93 of the checked
    /// corpus's files hold — reaches storage through `rcad-step`'s `geometry.rs`
    /// as it came; [`Axis::reversed`] negates, and negating `0.0` gives `-0.0`;
    /// and a turn can hand back `-0.0` for a component that was `0.0`.
    ///
    /// Adding zero is what turns one back the right way up: `-0.0 + 0.0` is
    /// `0.0` and `x + 0.0` is `x` for every other finite `x`, so this rounds
    /// nothing and moves no axis. [`Plane::holding`](crate::Plane) folds a plane
    /// the same way for the same reason.
    fn holding(origin: Point, direction: Vector) -> Self {
        let [x, y, z] = origin.coordinates();
        let [u, v, w] = direction.components();
        Self {
            origin: Point::new(x + 0.0, y + 0.0, z + 0.0),
            direction: Vector::new(u + 0.0, v + 0.0, w + 0.0),
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
    use core::cmp::Ordering;

    use super::*;
    use crate::{Cone, Cylinder, Surface, Torus};

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

    /// The separation a fuzzer found by fusing two boxes one of them
    /// `3.80218313259032e-211` mm up `y`. That is too small to move the top face
    /// and just big enough to move the bottom one, so the union really does have
    /// a step one hair high, and the edge along it really is that long. Squaring
    /// the hair gives `1.4e-421`, which is no `f64`, so the length came back
    /// zero and the kernel was told a real edge had no length.
    #[test]
    fn two_points_a_hair_apart_are_not_reported_as_the_same_point() {
        let hair = 3.80218313259032e-211;
        assert_eq!(
            Point::origin().distance_to(Point::new(0.0, hair, 0.0)),
            hair
        );
    }

    /// The other end of the same defect. Whole numbers times a power of two, so
    /// the true answer is exactly representable and the test needs no tolerance:
    /// squaring `4 · 2^600` overflows, and `5 · 2^600` is an ordinary `f64`.
    #[test]
    fn two_points_too_far_apart_to_square_are_a_finite_distance_apart() {
        let far = 2.0f64.powi(600);
        assert_eq!(
            Point::origin().distance_to(Point::new(3.0 * far, 4.0 * far, 0.0)),
            5.0 * far
        );
    }

    #[test]
    fn two_points_too_close_together_to_square_are_still_apart() {
        let close = 2.0f64.powi(-600);
        assert_eq!(
            Point::origin().distance_to(Point::new(3.0 * close, 4.0 * close, 0.0)),
            5.0 * close
        );
    }

    /// A vector whose length underflowed used to have no direction either,
    /// because `normalized` divides by that length and refuses when it is zero.
    #[test]
    fn a_vector_far_too_short_to_square_still_has_a_direction() {
        let close = 2.0f64.powi(-600);
        assert_eq!(
            Vector::new(0.0, 3.0 * close, 4.0 * close)
                .normalized()
                .unwrap(),
            Vector::new(0.0, 0.6, 0.8)
        );
    }

    #[test]
    fn a_vector_far_too_long_to_square_still_has_a_direction() {
        let far = 2.0f64.powi(600);
        assert_eq!(
            Vector::new(0.0, 3.0 * far, 4.0 * far).normalized().unwrap(),
            Vector::new(0.0, 0.6, 0.8)
        );
    }

    /// The very bottom of the range, where the length is one bit wide.
    ///
    /// `√2 · 2^-1074` rounds to `2^-1074`, so dividing the components by the
    /// length where they stand gives `(1, 1, 0)` — a "unit" vector of length
    /// `√2`. Measuring and dividing at the same scale gives the direction that
    /// does exist down there.
    #[test]
    fn a_direction_written_in_the_smallest_subnormals_is_still_a_direction() {
        let smallest = 5e-324;
        assert_eq!(
            Vector::new(smallest, smallest, 0.0).normalized().unwrap(),
            Vector::new(1.0, 1.0, 0.0).normalized().unwrap()
        );
    }

    /// The claim that pays for the rescale being a threshold rather than
    /// `f64::hypot`: no length an ordinary model asks for goes down the new
    /// path, so no length an ordinary model asks for changed by a bit.
    #[test]
    fn nothing_of_an_ordinary_size_is_rescaled_to_be_measured() {
        for magnitude in [
            2.0f64.powi(-511),
            1e-150,
            1e-9,
            1.0,
            1e9,
            1e150,
            2.0f64.powi(511),
        ] {
            assert_eq!(
                Vector::new(magnitude, -magnitude, magnitude)
                    .power_of_two_that_keeps_the_squares_in_range(),
                None,
                "a vector around {magnitude} is measured where it stands"
            );
        }
        assert_eq!(
            Vector::zero().power_of_two_that_keeps_the_squares_in_range(),
            None
        );
    }

    #[test]
    fn a_length_that_cannot_exist_is_still_an_infinity() {
        assert_eq!(
            Vector::new(f64::MAX, f64::MAX, f64::MAX).length(),
            f64::INFINITY
        );
        assert!(Vector::new(1.0, f64::NAN, 0.0).length().is_nan());
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

    /// **Rebuilding an axis from its own direction moves it, and
    /// [`Axis::moved_to`] is how a scaling avoids that.**
    ///
    /// The test above is about two vectors that lie *exactly* along one line,
    /// and that promise holds. A stored direction is not such a vector: it is
    /// the rounded normalizing of whatever built the axis, so dividing it by
    /// its own largest component and normalizing again rounds a second time and
    /// lands somewhere else. The pair of pipes the night of 2026-09-14 crashed
    /// on turned on exactly one such bit — one spelling crossed in a closed loop
    /// and the other in nothing, and both were right about the pipes they were
    /// given.
    #[test]
    fn rebuilding_an_axis_from_its_own_direction_moves_it_and_moving_its_origin_does_not() {
        let mut seed = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            f64::from(u32::try_from(seed & 0xffff_ffff).expect("thirty-two bits"))
                / f64::from(u32::MAX)
                - 0.5
        };

        let mut drawn = 0usize;
        let mut rebuilding_moved_it = 0usize;
        let mut moving_the_origin_moved_it = 0usize;
        for _ in 0..100_000 {
            let Ok(axis) = Axis::new(Point::origin(), Vector::new(next(), next(), next())) else {
                continue;
            };
            drawn += 1;
            let rebuilt = Axis::new(Point::origin(), axis.direction()).expect("a direction");
            if rebuilt.direction() != axis.direction() {
                rebuilding_moved_it += 1;
            }
            let moved = axis
                .moved_to(Point::new(2.0, -3.0, 5.0))
                .expect("a finite origin");
            if moved.direction() != axis.direction() {
                moving_the_origin_moved_it += 1;
            }
        }

        println!(
            "{drawn} directions: rebuilding the axis from its own direction stored other bits for \
             {rebuilding_moved_it} of them, moving the origin for {moving_the_origin_moved_it}"
        );
        assert!(
            drawn >= 99_000,
            "only {drawn} of 100 000 draws named a direction, so this is measuring the generator"
        );
        assert!(
            rebuilding_moved_it * 100 > drawn,
            "going back through Axis::new is meant to move a stored direction often enough to be \
             worth a method of its own, and here it moved {rebuilding_moved_it} of {drawn}"
        );
        assert_eq!(
            moving_the_origin_moved_it, 0,
            "a scaling moves an origin and leaves a direction alone, so moved_to has nothing to \
             work out and must round nothing"
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

    fn bits_of(axis: Axis) -> [u64; 6] {
        let [x, y, z] = axis.origin().coordinates();
        let [u, v, w] = axis.direction().components();
        [x, y, z, u, v, w].map(f64::to_bits)
    }

    /// Every way of signing the zeros of one axis, through the door `rcad-step`
    /// opens.
    ///
    /// A STEP `DIRECTION` may be written `(-0.,0.,1.)` and `brep.rs` hands the
    /// three numbers to [`Axis::new`] as they came. `-0.0` is the same number as
    /// `0.0` and a different `f64`, and everything that asks whether two faces
    /// are the same surface compares with [`f64::total_cmp`](f64::total_cmp),
    /// which sorts `-0.0` first.
    ///
    /// The scaling turns none of these back up, and the count says so rather
    /// than a sentence: [`Vector::scaled_to_length_one_from_anywhere`] divides
    /// by the largest magnitude and then by a length, and neither divisor is
    /// ever negative, so every sign that goes in comes out.
    #[test]
    fn an_axis_holds_the_same_bits_however_its_zeros_were_signed() {
        let plain = Axis::new(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 0.0, 1.0)).unwrap();
        let mut spellings = 0usize;
        let mut the_scaling_kept_the_sign = 0usize;

        for origin_x in [0.0, -0.0] {
            for origin_y in [0.0, -0.0] {
                for direction_x in [0.0, -0.0] {
                    for direction_y in [0.0, -0.0] {
                        let direction = Vector::new(direction_x, direction_y, 1.0);
                        spellings += 1;
                        let scaled = direction
                            .scaled_to_length_one_from_anywhere("axis direction")
                            .unwrap();
                        if scaled.components().map(f64::to_bits)
                            != plain.direction().components().map(f64::to_bits)
                        {
                            the_scaling_kept_the_sign += 1;
                        }
                        let spelt =
                            Axis::new(Point::new(origin_x, origin_y, 4.0), direction).unwrap();
                        assert_eq!(
                            bits_of(spelt),
                            bits_of(plain),
                            "an axis spelled with origin ({origin_x}, {origin_y}, 4) and \
                             direction ({direction_x}, {direction_y}, 1) holds other bits than \
                             the same axis spelled with every zero positive"
                        );
                    }
                }
            }
        }

        println!(
            "{the_scaling_kept_the_sign} of {spellings} spellings still hold a negative zero \
             after the scaling alone"
        );
        assert_eq!(
            the_scaling_kept_the_sign, 12,
            "the scaling divides by two positive numbers, so it turns no signed zero back up and \
             all twelve spellings with one in the direction should survive it"
        );
    }

    /// The bits an axis holds are the bits a pipe, a cone and a bend sort on.
    ///
    /// `Surface::how_it_sorts_against` reads the axis origin and direction
    /// straight out, and so does `rcad-topo`'s `name_of_surface_bits`, which
    /// names a corner by them one crate away. A cone's apex and a torus's centre
    /// are the axis origin, so this pair covers what all three write down.
    ///
    /// [`Axis::reversed`] is the second spelling worth naming: negating `0.0`
    /// gives `-0.0`, so a pipe turned round used to hold other bits than the
    /// same pipe named pointing down in the first place.
    #[test]
    fn one_axis_spelled_two_ways_builds_surfaces_that_sort_as_one() {
        let up = Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0)).unwrap();
        let down = Axis::new(Point::origin(), Vector::new(0.0, 0.0, -1.0)).unwrap();

        for (one, other, how_it_was_spelled) in [
            (
                up,
                Axis::new(Point::new(-0.0, 0.0, 0.0), Vector::new(-0.0, 0.0, 1.0)).unwrap(),
                "a negative zero as it was written in the file",
            ),
            (
                down,
                up.reversed(),
                "turned round rather than named downward",
            ),
        ] {
            let surfaces = |axis: Axis| {
                [
                    Surface::Cylinder(Cylinder::new(axis, 2.0).unwrap()),
                    Surface::Cone(Cone::new(axis, 0.5).unwrap()),
                    Surface::Torus(Torus::new(axis, 3.0, 1.0).unwrap()),
                ]
            };
            for (mine, theirs) in surfaces(one).into_iter().zip(surfaces(other)) {
                assert_eq!(
                    mine.how_it_sorts_against(&theirs),
                    Ordering::Equal,
                    "{mine:?} and the same surface, {how_it_was_spelled}, sort apart"
                );
            }
        }
    }

    /// §32.5's trap, on the type it was written about.
    ///
    /// Turning an axis by nothing has to give back the numbers that were stored,
    /// or `Cylinder::transformed` — which is how a pipe is read out of a model
    /// at whatever location its face carries — moves a bit on every read.
    #[test]
    fn moving_an_axis_by_the_identity_gives_back_the_bits_it_held() {
        for axis in [
            Axis::x(),
            Axis::y(),
            Axis::z(),
            Axis::z().reversed(),
            Axis::new(Point::new(1.0, -2.5, 3.0), Vector::new(1.0, 2.0, 3.0)).unwrap(),
        ] {
            assert_eq!(
                bits_of(axis.transformed(&Isometry::identity())),
                bits_of(axis)
            );
        }
    }
}
