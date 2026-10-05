//! Points, vectors and axes.
//!
//! These are newtypes over `nalgebra`, which is used freely inside them and
//! never appears in a signature. `nalgebra` is at 0.35 and breaks its API
//! regularly; if `nalgebra::Point3<f64>` were part of our public API then every
//! bump of it would be a breaking release for everyone downstream
//! (`docs/decisions.md` §3.4).

use core::ops::{Add, Mul, Neg, Sub};

use nalgebra as na;
use rcad_predicates::expansion::{product_is_exact, two_diff, two_product, two_sum};
use serde::{Deserialize, Serialize};

use crate::exact::{binary_exponent, multiplied_by_a_power_of_two};
use crate::isometry::Isometry;
use crate::plane::has_length_one_to_within_a_turn;
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

    /// The dot product with the whole sum rounded once instead of five times.
    ///
    /// Three products and two additions each round in [`Vector::dot`], so the
    /// answer can sit several ulps from the nearest `f64` to the real dot
    /// product. Here every product is kept exactly and the three of them are
    /// added to about a hundred bits, so the only place the answer rounds is
    /// the last add. `docs/decisions.md` §132 is why a plane's offset wants
    /// that.
    pub(crate) fn dot_rounded_once(&self, other: Vector) -> f64 {
        let mut total = (0.0, 0.0);
        for (mine, theirs) in self.components().into_iter().zip(other.components()) {
            total = sum_of(total, product_to_a_hundred_bits(mine, theirs));
        }
        total.0
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
    /// **Every component above `2^-1012` comes back correctly rounded** — the
    /// nearest `f64` there is to the exact ratio between that component and the
    /// exact length. Below `2^-1012` it can be one ulp out, which is the last
    /// ten binades before the subnormals and no direction anyone draws.
    /// `a_unit_component_is_the_nearest_f64_to_the_exact_ratio` is the test
    /// §15.6 asks for, and
    /// `a_tiny_third_component_comes_back_untouched_into_the_subnormals` pins
    /// the floor of the bound and everything under it.
    ///
    /// Two steps get there.
    ///
    /// The vector is first multiplied by the exact power of two that puts its
    /// largest component in `[1, 2)`. A power of two moves no mantissa bit, so
    /// what comes out is the *same ray*, not a rounding of it, and the answer
    /// then depends on the ratios between the components and on nothing else:
    /// two vectors along one line give the same direction however long each of
    /// them was, which is what [`Axis::new`] needs and §22.6 is about. A
    /// component more than about a thousand binades below the largest does lose
    /// bits on the way down, and cannot matter: its share of the unit vector is
    /// smaller than the smallest `f64`.
    ///
    /// Then the length of the scaled vector is worked out to about a hundred
    /// bits, held as two `f64` that sum to it, and each scaled component is
    /// divided by that pair. A hundred bits is fifty more than the answer keeps,
    /// so the fifty-third is already settled when the division rounds it.
    ///
    /// **Dividing by the largest component and then normalizing is what this
    /// used to do**, and it was out by up to three ulp. The divide was not what
    /// cost the bit — `(1, 1, 0)` has a largest component of one, so there it is
    /// the identity — rounding the square root and then rounding the division
    /// was. `docs/decisions.md` §120.
    ///
    /// It is not [`Vector::normalized`] itself, because that one is called on
    /// vectors that are already directions and rounding them again moves them.
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
        let up_to_one = -binary_exponent(widest);
        let scaled = [
            multiplied_by_a_power_of_two(x, up_to_one),
            multiplied_by_a_power_of_two(y, up_to_one),
            multiplied_by_a_power_of_two(z, up_to_one),
        ];
        let length = square_root_to_a_hundred_bits(sum_of_the_squares_to_a_hundred_bits(scaled));
        Ok(Vector::new(
            divided_by_a_hundred_bit_number(scaled[0], length),
            divided_by_a_hundred_bit_number(scaled[1], length),
            divided_by_a_hundred_bit_number(scaled[2], length),
        ))
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

/// A number held as two `f64` that sum to it, the high one carrying its size
/// and the low one the bits the high one had no room for.
///
/// One `f64` holds fifty-three bits of a number; a pair of them holds about a
/// hundred, which is what [`Vector::scaled_to_length_one_from_anywhere`] needs
/// to round the fifty-third bit of its answer the right way. The pair is always
/// kept so that adding the two together gives the high one back — that is what
/// [`two_sum`] returns and why every one of these ends with a call to it.
pub(crate) type AHundredBits = (f64, f64);

/// `x² + y² + z²`, to a hundred bits.
///
/// A square too small to square exactly is added with a low half of zero. The
/// bits that throws away are below `2^-969` while the total is at least one,
/// because the largest component is in `[1, 2)` by the time this is called —
/// eight hundred binades below the last bit the pair carries.
fn sum_of_the_squares_to_a_hundred_bits(components: [f64; 3]) -> AHundredBits {
    let mut total = (0.0, 0.0);
    for component in components {
        total = sum_of(total, product_to_a_hundred_bits(component, component));
    }
    total
}

/// The square root of a hundred-bit number, to a hundred bits.
///
/// One Newton step away from the `f64` root: `√v ≈ r + (v - r²) / 2r`. A Newton
/// step doubles the digits that are right and the `f64` root already has
/// fifty-three of them, so the pair that comes out has the hundred. The caller
/// always passes a value of at least one, so the root is never zero.
fn square_root_to_a_hundred_bits(value: AHundredBits) -> AHundredBits {
    let root = value.0.sqrt();
    let gap = difference_of(value, two_product(root, root));
    two_sum(root, (gap.0 + gap.1) / (2.0 * root))
}

/// `numerator / length`, rounded once rather than twice.
///
/// The first division is the `f64` answer, good to fifty-three bits. What is
/// left over after taking it back out is worked out against the whole hundred-
/// bit length, so the correction carries the bits the first division dropped,
/// and the single add at the end is the only place the answer rounds.
fn divided_by_a_hundred_bit_number(numerator: f64, length: AHundredBits) -> f64 {
    let first = numerator / length.0;
    if first == 0.0 {
        return first;
    }
    let taken_back_out = product_to_a_hundred_bits(first, length.0);
    let left_over = difference_of(
        (numerator, 0.0),
        sum_of(taken_back_out, (first * length.1, 0.0)),
    );
    first + (left_over.0 + left_over.1) / length.0
}

/// A hundred-bit numerator over an `f64`, rounded once rather than twice.
///
/// The mirror of [`divided_by_a_hundred_bit_number`], for the caller that has
/// the bits in the numerator instead: [`Plane::offset_of`](crate::Plane) forms
/// `d · unitᵢ` exactly and then divides it by a coefficient. Same shape — the
/// `f64` quotient, then what is left over after taking it back out of the whole
/// hundred-bit numerator, and one add.
///
/// **Keeping the product exact is what keeps two proportional coefficient sets
/// agreeing** (`docs/decisions.md` §17.1, §22.6, §132). Nothing rounds before
/// the division, so scaling the plane scales the numerator and the denominator
/// by exactly the same real number and the quotient is the same real number —
/// which the old spelling bought by dividing first and paid for with a second
/// rounding.
pub(crate) fn a_hundred_bit_number_divided_by(numerator: AHundredBits, divisor: f64) -> f64 {
    let first = numerator.0 / divisor;
    if first == 0.0 {
        return first;
    }
    let left_over = difference_of(numerator, product_to_a_hundred_bits(first, divisor));
    first + (left_over.0 + left_over.1) / divisor
}

/// `left · right` exactly when that product is exact, and rounded when it is
/// not — the pair every hundred-bit sum here starts from.
pub(crate) fn product_to_a_hundred_bits(left: f64, right: f64) -> AHundredBits {
    if product_is_exact(left, right) {
        two_product(left, right)
    } else {
        (left * right, 0.0)
    }
}

/// Two hundred-bit numbers added, to a hundred bits.
fn sum_of(left: AHundredBits, right: AHundredBits) -> AHundredBits {
    let (high, carried) = two_sum(left.0, right.0);
    two_sum(high, carried + left.1 + right.1)
}

/// Two hundred-bit numbers subtracted, to a hundred bits.
fn difference_of(left: AHundredBits, right: AHundredBits) -> AHundredBits {
    let (high, carried) = two_diff(left.0, right.0);
    two_sum(high, carried + left.1 - right.1)
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
/// the turn, and [`Axis::as_written`] keeps a direction that is that close
/// rather than scale it again.
///
/// Serializes as `(origin, direction)` and is re-checked on the way back in
/// through [`Axis::as_written`], so a file cannot hand us an axis without a
/// direction, and an axis that was stored comes back with the bits it was
/// stored with.
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
    /// an exact power-of-two rescale before the division by the length, which is
    /// what [`Vector::scaled_to_length_one_from_anywhere`] does and what plain
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

    /// The line through `origin` along `direction`, **storing `direction`'s
    /// own bits** when its squared length is within
    /// [`Plane::NORMAL_LENGTH_ROUNDOFF`](crate::Plane::NORMAL_LENGTH_ROUNDOFF)
    /// of one, and scaling it as [`Axis::new`] does when it is not.
    ///
    /// This is for a direction somebody already wrote down as one: a STEP
    /// file's pipe, cone or ring, which carries the axis its writer's model
    /// stored, and an axis read back from saved bytes. A moved axis points
    /// along a turn of a unit direction, an ulp or two off length one, which
    /// [`Axis::transformed`] keeps rather than scale again. [`Axis::new`]
    /// scales it again and can move its last bit: the axis `z` turned about
    /// `x` by `2.5` is `(0, −0.5984721441039565, −0.8011436155469336)`, and
    /// [`Axis::new`] makes it `(…, −0.8011436155469337)`. A ring written about
    /// it came back an ulp off the plane its seam was cut from, which kept
    /// that direction, and the bend was refused (`docs/decisions.md` §355.6).
    ///
    /// The check is [`Plane::from_point_and_normal_as_written`]'s own, exact
    /// and shared, so a plane and an axis written along one direction keep it
    /// or scale it alike. A direction past the bound is scaled and not
    /// refused, for the reasons that constructor gives: STEP does not ask for
    /// a direction of length one, and both answers are the line the caller
    /// named.
    ///
    /// **Two lengths of one direction are not one axis here**, where
    /// [`Axis::new`] makes them one: `(s, 0, s)` for the `f64` just below
    /// `1/√2` is kept, and `(1, 0, 1)` is scaled to the `f64` just above.
    ///
    /// [`Plane::from_point_and_normal_as_written`]: crate::Plane::from_point_and_normal_as_written
    ///
    /// # Errors
    ///
    /// The same as [`Axis::new`]'s.
    pub fn as_written(origin: Point, direction: Vector) -> Result<Self> {
        if !has_length_one_to_within_a_turn(direction) {
            return Self::new(origin, direction);
        }
        if !origin.is_finite() {
            return Err(Error::NotFinite {
                what: "axis origin",
            });
        }
        Ok(Self::holding(origin, direction))
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
    /// [`Vector::scaled_to_length_one_from_anywhere`] multiplies by a positive
    /// power of two and divides by a positive length, so neither step turns a
    /// signed zero back up and a `DIRECTION` written `(-0.,0.,1.)` — which 93 of the checked
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
        Self::as_written(origin, direction)
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

    use rcad_predicates::Decision;

    use super::*;
    use crate::exact::Exact;
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

    /// The claim [`Vector::dot_rounded_once`] makes, on three terms where the
    /// ordinary dot product loses all of the answer.
    ///
    /// `2^60 + 1 − 2^60` is one. Added left to right in `f64` the one falls off
    /// the end of `2^60 + 1` and what comes back is zero — not a rounding of
    /// the answer, the whole of it. Every product here is exact, so the
    /// hundred-bit sum carries the one and the single rounding at the end has
    /// nothing left to throw away.
    #[test]
    fn a_dot_product_rounded_once_keeps_a_term_three_products_and_two_adds_lose() {
        let huge = 2f64.powi(60);
        let ones = Vector::new(1.0, 1.0, 1.0);
        let cancelling = Vector::new(huge, 1.0, -huge);

        assert_eq!(ones.dot(cancelling), 0.0, "in order, the one is lost");
        assert_eq!(ones.dot_rounded_once(cancelling), 1.0, "and it is an f64");
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
    /// Scaling by an exact power of two first makes what comes out depend on
    /// the ratios between the components and on nothing else, and the ratios of
    /// two multiples of one vector are the same real numbers, so they round the
    /// same way. Normalizing alone divides by two lengths that are not multiples
    /// of each other in `f64`, and **38.1% of these pairs land on two different
    /// directions** because of it.
    ///
    /// This is §22.6's same-ray property, and it is checked here rather than
    /// argued from the correct rounding of §120: a correctly rounded answer is a
    /// function of the exact ray and so keeps the property by construction, but
    /// "by construction" is the kind of sentence that is true right up until the
    /// scaling loses a bit.
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
            "scaling by a power of two makes the answer a function of the ratios, and a ratio \
             does not care how long the vector was"
        );
    }

    /// **Every component of a direction above `2^-1012` is the `f64` nearest to
    /// the exact ratio it stands for**, which is the bound
    /// [`Vector::scaled_to_length_one_from_anywhere`] publishes and the test
    /// §15.6 asks for.
    ///
    /// Nothing here is decided in `f64`. An answer is right when it is at least
    /// as close to the true `c / √Q` as each of its two neighbours is, and
    /// "closer to `answer` than to `neighbour`" is `2c` against
    /// `(answer + neighbour)·√Q`. Squaring both sides turns the root into `Q`
    /// and leaves `(2c)² - (answer + neighbour)²·Q`, whose sign an expansion
    /// names exactly.
    ///
    /// The old arithmetic is measured beside it rather than remembered: divide
    /// by the largest component and then normalize is three lines below, and it
    /// misses the nearest `f64` on nearly one component in five. That is what
    /// makes this a test the change was needed for and not a test of whatever
    /// the code happens to do.
    ///
    /// **How far down this reaches is measured too, and printed.** The exact
    /// reference is [`ExactFrame`], which lifts the smallest component of a
    /// triple to `2^-400` and gives up on a lift past 500, so it carries a
    /// triple 900 binades wide and no wider — asserted below at 900 and 901
    /// rather than read off the constant. Hand-written directions `(1, 2^-e, 0)`
    /// claim every one of those binades; the drawn directions alone stopped
    /// about a hundred short of them, because their exponents come out of a
    /// range 800 wide. So the sweep pins the bound from the top down to `2^-900`
    /// and the last 112 binades of `2^-1012` rest on §120.1's integer-triple
    /// measurement instead. `a_tiny_third_component_comes_back_untouched_into
    /// _the_subnormals` pins the floor itself by a route that needs no expansion
    /// arithmetic at all.
    #[test]
    fn a_unit_component_is_the_nearest_f64_to_the_exact_ratio() {
        let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };

        let mut directions = vec![
            Vector::new(1.0, 1.0, 0.0),
            Vector::new(1.0, 1.0, 1.0),
            Vector::new(3.0, 4.0, 5.0),
            Vector::new(2f64.powi(-483), 2f64.powi(516), 0.0),
        ];
        for x in -6i32..=6 {
            for y in -6i32..=6 {
                for z in -6i32..=6 {
                    let triple = Vector::new(f64::from(x), f64::from(y), f64::from(z));
                    if triple != Vector::zero() {
                        directions.push(triple);
                    }
                }
            }
        }
        // A drawn triple spans at most eight hundred binades, which leaves the
        // bottom hundred of what the exact reference can carry unclaimed. These
        // claim them: the second component of each is the smallest thing any
        // direction here is checked at.
        for spread in 1..=900 {
            directions.push(Vector::new(
                1.0,
                multiplied_by_a_power_of_two(1.0, -spread),
                0.0,
            ));
        }
        // Mantissas out of the generator and exponents spread as wide as four
        // hundred binades, so the drawn directions reach well past anything a
        // lattice of small whole numbers holds.
        for _ in 0..3_000 {
            let mut drawn = || {
                let bits = next();
                let mantissa = f64::from(u32::try_from(bits >> 32).expect("thirty-two bits"))
                    / f64::from(u32::MAX)
                    - 0.5;
                let exponent = i32::try_from(bits % 801).expect("under a thousand") - 400;
                multiplied_by_a_power_of_two(mantissa, exponent)
            };
            let triple = Vector::new(drawn(), drawn(), drawn());
            if triple != Vector::zero() {
                directions.push(triple);
            }
        }

        // Divide by the largest component, then normalize: what `Axis::new`
        // called until §120, kept here so the comparison is a measurement.
        let the_old_way = |direction: &Vector| {
            let [x, y, z] = direction.components();
            let widest = x.abs().max(y.abs()).max(z.abs());
            Vector::new(x / widest, y / widest, z / widest).normalized()
        };

        let mut checked = 0usize;
        let mut wrong = 0usize;
        let mut wrong_the_old_way = 0usize;
        let mut too_wide_to_square = 0usize;
        let mut below_the_bound = 0usize;
        let mut smallest_checked = f64::INFINITY;

        for direction in directions {
            let Ok(unit) = direction.scaled_to_length_one_from_anywhere("direction") else {
                continue;
            };
            let Ok(old_unit) = the_old_way(&direction) else {
                continue;
            };
            let Some(frame) = ExactFrame::around(direction) else {
                too_wide_to_square += 1;
                continue;
            };
            for axis in 0..3 {
                let answer = unit.components()[axis];
                let component = frame.components[axis];
                assert_eq!(
                    answer == 0.0,
                    component == 0.0 || answer.abs() < f64::MIN_POSITIVE,
                    "a zero unit component and a zero input component go together, and {direction:?} \
                     has one without the other on axis {axis}"
                );
                if answer == 0.0 {
                    below_the_bound += usize::from(component != 0.0);
                    continue;
                }
                assert_eq!(
                    answer < 0.0,
                    component < 0.0,
                    "{direction:?} came back pointing the other way on axis {axis}"
                );
                if answer.abs() < 2f64.powi(-1012) {
                    below_the_bound += 1;
                    continue;
                }
                let Some(nearest) = frame.is_the_nearest(answer, component) else {
                    too_wide_to_square += 1;
                    continue;
                };
                checked += 1;
                smallest_checked = smallest_checked.min(answer.abs());
                if !nearest {
                    wrong += 1;
                    println!(
                        "{direction:?} rounds axis {axis} to {answer:e}, which is not nearest"
                    );
                }
                let old_answer = old_unit.components()[axis];
                if old_answer == 0.0 || frame.is_the_nearest(old_answer, component) != Some(true) {
                    wrong_the_old_way += 1;
                }
            }
        }

        println!(
            "{checked} components checked against an exact reference: {wrong} not correctly \
             rounded, against {wrong_the_old_way} ({:.2}%) for dividing by the largest component \
             and normalizing. {below_the_bound} components below 2^-1012 and \
             {too_wide_to_square} comparisons too wide to square. The smallest \
             component checked was 2^{}.",
            100.0 * wrong_the_old_way as f64 / checked as f64,
            binary_exponent(smallest_checked)
        );
        // Where the exact reference stops, measured rather than read off
        // `ExactFrame::around`'s two constants.
        let spread_of =
            |binades: i32| Vector::new(1.0, multiplied_by_a_power_of_two(1.0, -binades), 0.0);
        assert!(
            ExactFrame::around(spread_of(900)).is_some(),
            "the exact reference is meant to carry a triple nine hundred binades wide"
        );
        assert!(
            ExactFrame::around(spread_of(901)).is_none(),
            "a triple wider than nine hundred binades has no scale that carries the comparison, \
             so the reference must refuse it rather than decide it in f64"
        );
        assert!(
            binary_exponent(smallest_checked) <= -900,
            "the sweep is meant to reach as far down as the exact reference carries, and it \
             stopped at 2^{}",
            binary_exponent(smallest_checked)
        );
        assert!(
            checked >= 10_000,
            "only {checked} components reached the exact reference, which is not the measurement \
             this claims to be"
        );
        assert_eq!(
            wrong, 0,
            "every component above 2^-1012 is meant to be the nearest f64 to its exact ratio"
        );
        assert!(
            wrong_the_old_way * 50 > checked,
            "the old arithmetic is what this change replaced, and if it misses the nearest f64 \
             only {wrong_the_old_way} times out of {checked} then this test is not the one §15.6 \
             asks for"
        );
    }

    /// **`(1, 0, t)` with a tiny `t` is already its own unit vector**, so this
    /// pins the floor of the published bound and everything under it, right down
    /// to the smallest number an `f64` can hold.
    ///
    /// The true length is `√(1 + t²)`, which is `1` out by a relative `t²/2`.
    /// At `t = 2^-e` that is `2^(-2e-1)`, and both components are scaled by it:
    /// the true unit is `(1 - 2^(-2e-1), 0, t·(1 - 2^(-2e-1)))` to leading order.
    /// A `f64` keeps 53 bits, so the nearest one to each of those is what it
    /// started as the moment `2^(-2e-1)` is below half an ulp, `2^-54` — that is
    /// `e ≥ 27`, and this starts at 600. So the correctly rounded answer here is
    /// `(1.0, 0.0, t)` exactly, with nothing to round.
    ///
    /// **The premise is checked and not asserted.** Wherever the exact reference
    /// of the test above can still carry the comparison — a triple 900 binades
    /// wide and no wider — this asks it, and it agrees. Past 900 binades the
    /// reference refuses, and the reasoning above is what carries the rest.
    ///
    /// **This does not fail without §120's change.** Divide by the largest
    /// component and then normalize hands back `(1, 0, t)` here too, and the
    /// assertion below says so rather than leaving it to be assumed: the length
    /// it computes rounds to `1.0` and dividing by one moves nothing. So this
    /// test is the half of §15.6 that says the published bound has a test, and
    /// the sweep above is the half that says the bound is the new arithmetic's
    /// and not the old one's.
    #[test]
    fn a_tiny_third_component_comes_back_untouched_into_the_subnormals() {
        let mut agreed_with_the_exact_reference = 0usize;
        for e in 600..=1074 {
            let tiny = multiplied_by_a_power_of_two(1.0, -e);
            let direction = Vector::new(1.0, 0.0, tiny);
            let unit = direction
                .scaled_to_length_one_from_anywhere("direction")
                .expect("a direction with a component of one has a length");
            assert_eq!(
                unit,
                Vector::new(1.0, 0.0, tiny),
                "(1, 0, 2^-{e}) is already the nearest f64 triple to its own unit vector"
            );
            let old = Vector::new(1.0, 0.0, tiny)
                .normalized()
                .expect("a direction with a component of one has a length");
            assert_eq!(
                old, unit,
                "(1, 0, 2^-{e}) is where the old arithmetic and the new one agree, and this test \
                 does not discriminate between them"
            );
            if let Some(frame) = ExactFrame::around(direction) {
                assert_eq!(
                    frame.is_the_nearest(tiny, frame.components[2]),
                    Some(true),
                    "the exact reference can still carry 2^-{e} and must agree that it rounds to \
                     itself"
                );
                agreed_with_the_exact_reference += 1;
            }
        }
        assert_eq!(
            agreed_with_the_exact_reference, 301,
            "the exact reference carries a triple 900 binades wide, so it is meant to check the \
             first 301 of these and refuse the rest"
        );
    }

    /// One direction, scaled so that this crate's exact arithmetic can carry
    /// the whole comparison.
    ///
    /// Two scalings, and both are multiplications by a power of two, which move
    /// no mantissa bit: the ratios the answer is a function of are the same
    /// before and after. The first puts the largest component in `[1, 2)`, so
    /// the length is between one and the root of three. The second lifts the
    /// smallest component up to `2^-400`, which is not where an [`Exact`] stops
    /// squaring — that is `2^-484` — but where the *product* of two of them
    /// stops being exact. The comparison multiplies the squared length by a
    /// three-entry square: the squared length's lowest entry sits at about
    /// `2^(2·smallest − 53)` and the square's at about `2^-104`, and their
    /// product has to stay above the `2^-969` [`product_is_exact`] wants.
    ///
    /// So a triple spread wider than nine hundred binades has no scale that
    /// carries this comparison, and comes back `None` rather than being decided
    /// in `f64`. Nine hundred is the `lift > 500` below against a lift target of
    /// `2^-400`, and it is this struct's own wall — `exact::WIDEST_SQUARABLE
    /// _SPREAD` is a different number for a different question, 995 binades of
    /// spread inside one expansion, and nothing here consults it.
    struct ExactFrame {
        components: [f64; 3],
        lift: i32,
        squared_length: Exact,
    }

    impl ExactFrame {
        fn around(direction: Vector) -> Option<Self> {
            let [x, y, z] = direction.components();
            let widest = x.abs().max(y.abs()).max(z.abs());
            if widest == 0.0 {
                return None;
            }
            let up_to_one = -binary_exponent(widest);
            let components = [
                multiplied_by_a_power_of_two(x, up_to_one),
                multiplied_by_a_power_of_two(y, up_to_one),
                multiplied_by_a_power_of_two(z, up_to_one),
            ];
            let smallest = components
                .iter()
                .filter(|component| **component != 0.0)
                .map(|component| binary_exponent(*component))
                .min()?;
            let lift = (-400 - smallest).max(0);
            if lift > 500 {
                return None;
            }
            let lifted = [
                multiplied_by_a_power_of_two(components[0], lift),
                multiplied_by_a_power_of_two(components[1], lift),
                multiplied_by_a_power_of_two(components[2], lift),
            ];
            Some(Self {
                components,
                lift,
                squared_length: Exact::dot_of(lifted, lifted)?,
            })
        }

        /// Whether `answer` is at least as close to `component / √Q` as either
        /// of its neighbours, or `None` when the exact arithmetic could not
        /// carry the comparison.
        ///
        /// The answer and the component are lifted together by the power of two
        /// that puts the answer in `[1, 2)`; the component then lands near the
        /// length, which is between one and the root of three. Both sides of the
        /// comparison carry the same power of two, so the sign is the sign of
        /// the unlifted question.
        fn is_the_nearest(&self, answer: f64, component: f64) -> Option<bool> {
            let answer = answer.abs();
            let component = component.abs();
            let to_one = -binary_exponent(answer);
            let twice_the_component = multiplied_by_a_power_of_two(
                multiplied_by_a_power_of_two(component, to_one),
                self.lift + 1,
            );
            let lifted_answer = multiplied_by_a_power_of_two(answer, to_one);
            let gap_from = |neighbour: f64| {
                let lifted = multiplied_by_a_power_of_two(neighbour, to_one);
                let both = Exact::product_of(lifted_answer, lifted_answer)?
                    .plus(&Exact::product_of(lifted_answer, lifted)?)
                    .plus(&Exact::product_of(lifted_answer, lifted)?)
                    .plus(&Exact::product_of(lifted, lifted)?);
                Some(
                    Exact::product_of(twice_the_component, twice_the_component)?
                        .minus(&both.times(&self.squared_length)?)
                        .sign(),
                )
            };
            let below = gap_from(answer.next_down())?;
            let above = gap_from(answer.next_up())?;
            Some(
                matches!(below, Decision::Pos | Decision::Zero)
                    && matches!(above, Decision::Neg | Decision::Zero),
            )
        }
    }

    /// **Rebuilding an axis from its own direction moves it, and
    /// [`Axis::moved_to`] is how a scaling avoids that.**
    ///
    /// The test above is about two vectors that lie *exactly* along one line,
    /// and that promise holds. A stored direction is not such a vector: it is
    /// the rounded normalizing of whatever built the axis, so normalizing it
    /// again rounds a second time and can land somewhere else. The pair of pipes
    /// the night of 2026-09-14 crashed on turned on exactly one such bit — one
    /// spelling crossed in a closed loop and the other in nothing, and both were
    /// right about the pipes they were given.
    ///
    /// §120's correctly rounded normalize made this much rarer without making it
    /// go away: a stored direction is now the nearest `f64` to its own ratio, and
    /// normalizing such a triple usually hands the same triple back. **992 of
    /// 100 000 still move**, which is why the threshold below is half a percent
    /// and not the one percent it was.
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
            rebuilding_moved_it * 200 > drawn,
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
    /// than a sentence: [`Vector::scaled_to_length_one_from_anywhere`]
    /// multiplies by a power of two and then divides by a length, and neither a
    /// power of two nor a length is ever negative, so every sign that goes in
    /// comes out.
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
            "the scaling multiplies by a positive power of two and divides by a positive length, \
             so it turns no signed zero back up and all twelve spellings with one in the \
             direction should survive it"
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
