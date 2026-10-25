//! Torus surfaces.

use rcad_predicates::{
    Decision, Filtered, Interval, NOTHING_IS_KNOWN, Point as PredicatePoint, Predicate,
    SideOfSurface, are_real_numbers, decide, is_a_real_point, product_is_normal,
};
use serde::{Deserialize, Serialize};

use crate::exact::Exact;
use crate::isometry::Isometry;
use crate::point::{Axis, Point, Vector};
use crate::sphere::{largest_magnitude, squared, without_overflow};
use crate::{Error, Result};

/// The name the torus's sign predicate goes by in provenance records.
pub const SIDE_OF_TORUS: &str = "side of torus";

/// A ring's skin: every point exactly `tube_radius` away from the circle of
/// `ring_radius` drawn round `axis`.
///
/// # A quartic, and that is the whole of what is different
///
/// A plane, a ball, a pipe and a cone are all **quadrics** — their sign is a
/// polynomial of degree two, a straight line meets them twice, and a plane cuts
/// them in a conic. A torus is none of those. Its sign,
///
/// ```text
/// ((p − centre)·(p − centre) + R² − r²)²  −  4R²·(distance from the axis)²
/// ```
///
/// is degree **four**, a straight line meets it four times, and a plane cuts it
/// in a quartic that is a pair of circles only in two families and something
/// with no name in this kernel everywhere else. Every refusal a torus produces
/// traces back to that one sentence, and each of them says so.
///
/// # What is certified: both radii exactly
///
/// Multiply the expression above through by `w·w` and it becomes
///
/// ```text
/// (w·w)·(x·x + R² − r²)²  −  4R²·‖x × w‖²      with x = p − centre
/// ```
///
/// which is a polynomial in the numbers as stored, with no division and no
/// square root. Its zero set is **exactly** the torus of radii `R` and `r` about
/// the line the stored [`Axis`] names, whatever length that axis's direction
/// came out at — the `w·w` cancels the same way the cone's `|w|²` does. So both
/// of a torus's radii are the numbers the caller wrote. What rounds is where the
/// axis line is, which is [`Axis::new`]'s rounding and not this type's.
///
/// A [`Cylinder`](crate::Cylinder) was the exception to that for a while, its
/// certified radius being `radius / length(stored direction)`. §118.11 put
/// `‖w‖²` on both sides of the pipe's comparison too, so every surface in this
/// crate now means the sizes its caller wrote.
///
/// # Why a tube wider than its ring is refused rather than repaired
///
/// With `r ≥ R` the surface runs through itself — it has a pinch on the axis at
/// `r = R` and a whole self-intersection curve past it — and the sign above
/// **stops meaning inside and outside**. The argument that makes it a sign
/// predicate at all is
///
/// ```text
/// A = x·x + R² − r²   is positive everywhere when R > r,
/// so sign(A² − 4R²s²) = sign(A − 2Rs) = sign((s − R)² + a² − r²)
/// ```
///
/// with `s` the distance from the axis and `a` the distance along it. That first
/// line is where `R > r` is spent: at `r ≥ R` a point near the centre has
/// `A ≤ 0`, `A² − 4R²s²` comes out **positive** for a point that is plainly
/// inside the tube, and the predicate certifies the wrong side.
/// `a_tube_wider_than_its_ring_would_certify_the_wrong_side` builds that point
/// and shows the sign, which is why this is an [`Error`] at the door and not a
/// note in a doc comment.
///
/// It is caller input and never a broken invariant: these two numbers arrive
/// from a stranger's bytes through `Deserialize` as readily as from a call.
///
/// Serializes as `(axis, ring_radius, tube_radius)` and is re-checked on the way
/// in.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "(Axis, f64, f64)", into = "(Axis, f64, f64)")]
pub struct Torus {
    axis: Axis,
    ring_radius: f64,
    tube_radius: f64,
}

impl Torus {
    /// The torus whose centre is `axis.origin()`, whose centre circle has
    /// radius `ring_radius` about `axis`, and whose tube has radius
    /// `tube_radius`.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `axis` is not made of finite numbers,
    /// [`Error::RadiusNotPositive`] if either radius is zero, negative, or not a
    /// finite number, and [`Error::ATorusThatRunsThroughItself`] if the tube is
    /// as wide as the ring or wider. See [`Torus`] for why the last one is a
    /// refusal rather than a repair.
    pub fn new(axis: Axis, ring_radius: f64, tube_radius: f64) -> Result<Self> {
        if !axis.origin().is_finite() || !axis.direction().is_finite() {
            return Err(Error::NotFinite { what: "torus axis" });
        }
        for radius in [ring_radius, tube_radius] {
            if !(radius.is_finite() && radius > 0.0) {
                return Err(Error::RadiusNotPositive { radius });
            }
        }
        if tube_radius >= ring_radius {
            return Err(Error::ATorusThatRunsThroughItself {
                ring_radius,
                tube_radius,
            });
        }
        Ok(Self {
            axis,
            ring_radius,
            tube_radius,
        })
    }

    /// The line the ring is drawn round. Its origin is the middle of the hole.
    #[must_use]
    pub fn axis(&self) -> Axis {
        self.axis
    }

    /// The middle of the hole — the point the centre circle is drawn round.
    #[must_use]
    pub fn centre(&self) -> Point {
        self.axis.origin()
    }

    /// How far the middle of the tube runs from the axis. Always greater than
    /// [`Torus::tube_radius`].
    #[must_use]
    pub fn ring_radius(&self) -> f64 {
        self.ring_radius
    }

    /// How thick the tube is. Always greater than zero and always less than
    /// [`Torus::ring_radius`].
    #[must_use]
    pub fn tube_radius(&self) -> f64 {
        self.tube_radius
    }

    /// The same torus after `movement` is applied. A rigid movement cannot
    /// change either radius, so both go over untouched.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        Self {
            axis: self.axis.transformed(movement),
            ring_radius: self.ring_radius,
            tube_radius: self.tube_radius,
        }
    }

    /// Two directions across the axis, at right angles to each other and to it,
    /// with the first crossed into the second giving the axis back.
    ///
    /// The same choice a [`Cylinder`](crate::Cylinder) makes and for the same
    /// reason: `u = 0` lands on `+x` for a torus about `z`, so a ring's seam
    /// runs down the edge of its own `(u, v)` rectangle rather than across the
    /// middle of it.
    ///
    /// # Panics
    ///
    /// If the stored axis direction has no length. No public constructor can
    /// hand back one — [`Axis::new`] checks it, and [`Axis::as_written`], which
    /// `Deserialize` goes through by way of [`Torus::new`], hands it any
    /// direction whose length is not within a hair of one — and [`Torus::transformed`]
    /// cannot shrink one, for the reason
    /// [`Cylinder::basis`](crate::Cylinder::basis) spells out at length.
    #[must_use]
    pub fn basis(&self) -> (Vector, Vector) {
        let along = self.axis.direction();
        let [x, y, z] = along.components().map(f64::abs);
        let least_aligned_axis = if x <= y && x <= z {
            Vector::new(1.0, 0.0, 0.0)
        } else if y <= z {
            Vector::new(0.0, 1.0, 0.0)
        } else {
            Vector::new(0.0, 0.0, 1.0)
        };
        let across = least_aligned_axis - along * along.dot(least_aligned_axis);
        let u = across
            .normalized_as("direction across a torus")
            .expect("a unit axis is never parallel to its least aligned coordinate axis");
        (u, along.cross(u))
    }

    /// The point at `over` radians round the tube and `round` radians round the
    /// ring.
    ///
    /// `over = 0` is the outer equator, furthest from the axis, and `over = π`
    /// is the inner one, down in the hole. `round = 0` is [`Torus::basis`]'s
    /// first direction.
    ///
    /// # Round the tube comes first, and that is not arbitrary
    ///
    /// A mesher cuts a face open along the coordinate that **wraps**, and it is
    /// the first one it looks at — a pipe's angle wraps and its height does not,
    /// a ball's longitude wraps and its latitude does not. A bend of pipe is a
    /// piece of a torus that goes right round the **tube** and part way round
    /// the ring, so putting the tube first makes a bend the same shape in its
    /// chart as a straight bar: one wrapping coordinate, one running from cap to
    /// cap, one seam.
    ///
    /// **Both coordinates run right round, and neither ever pinches.** That is
    /// the whole of what makes a torus's chart different from a ball's and from
    /// a cone's: a ball's longitude collapses at its two poles and a cone's
    /// angle collapses at its apex, so a face on either has a place where one
    /// coordinate stops meaning anything. A torus has none — a unit of `over`
    /// carries `tube_radius` everywhere, and a unit of `round` carries
    /// `ring_radius + tube_radius·cos(over)`, which is at least
    /// `ring_radius − tube_radius` and so is never nothing. What a torus has
    /// instead is **two seams**, one for each coordinate, and the second of them
    /// is what a face going right round the ring as well would need.
    ///
    /// Only exactly on the torus where the sines and cosines are exact, which in
    /// practice means both angles zero. No corner of a face is ever made this
    /// way, for the reason [`Cylinder::point_at`](crate::Cylinder::point_at)
    /// gives.
    #[must_use]
    pub fn point_at(&self, over: f64, round: f64) -> Point {
        let (u, v) = self.basis();
        let out = u * round.cos() + v * round.sin();
        self.axis.origin()
            + out * (self.ring_radius + self.tube_radius * over.cos())
            + self.axis.direction() * (self.tube_radius * over.sin())
    }

    /// How far from the axis the surface reaches at `over` radians round the
    /// tube.
    ///
    /// One product and one sum, so it rounds twice. Always at least
    /// `ring_radius − tube_radius`, which is greater than zero.
    #[must_use]
    pub fn distance_from_the_axis_at(&self, over: f64) -> f64 {
        self.ring_radius + self.tube_radius * over.cos()
    }

    /// Which angle round the tube and which round the ring `point` sits at —
    /// [`Torus::point_at`] the other way round.
    ///
    /// Both angles come back on `atan2`'s branch, `(−π, π]`.
    ///
    /// **It answers about the nearest point on the torus**, unlike a cone's,
    /// which answers about the same ruling. A point off the skin is turned into
    /// the pair `(distance from the axis, distance along it)` first, and that
    /// pair's angle round the middle of the tube is the second coordinate — so
    /// the nearest point on the surface and this point share both coordinates.
    ///
    /// **Two places have no answer and neither is a pole.** A point *on the
    /// axis* has no angle round the ring, and zero is what `atan2(0, 0)` hands
    /// back without a branch — the same answer a pipe gives for a point on its
    /// own axis. A point on the **centre circle** is the same distance from
    /// every point of the tube round it, so it has no angle round the tube
    /// either, and again the arithmetic gives zero without a branch. Neither is
    /// a pinch of the surface: both are points the surface does not pass
    /// through.
    #[must_use]
    pub fn coordinates_of(&self, point: Point) -> (f64, f64) {
        let (across, and) = self.basis();
        let from_centre = point - self.axis.origin();
        let out = from_centre.dot(across);
        let sideways = from_centre.dot(and);
        (
            from_centre
                .dot(self.axis.direction())
                .atan2(out.hypot(sideways) - self.ring_radius),
            sideways.atan2(out),
        )
    }
}

/// Inside, on, or outside, decided through the cascade.
///
/// The sign of `(w·w)(x·x + R² − r²)² − 4R²‖x × w‖²` with `x = point − centre`:
/// [`Decision::Neg`] inside the tube, [`Decision::Pos`] outside,
/// [`Decision::Zero`] exactly on the skin.
///
/// **Why that quartic's sign is the tube's inside**, which is not obvious and is
/// where [`Torus::new`]'s refusal of a fat tube is spent: write `s` for the
/// distance from the axis and `a` for the distance along it, so
/// `x·x = s² + a²`. Then `A = x·x + R² − r²` is at least `R² − r² > 0`, and
///
/// ```text
/// A² − 4R²s² = (A − 2Rs)(A + 2Rs),   A + 2Rs > 0,
/// A − 2Rs = s² − 2Rs + R² + a² − r² = (s − R)² + a² − r²
/// ```
///
/// which is exactly negative inside the tube, zero on it and positive outside.
/// The `w·w` in front only scales a positive number by a positive number.
///
/// A point with exact `f64` coordinates always gets a certified answer, as long
/// as no product overflows or falls into the subnormals. A constructed point is
/// only known to lie inside a box, so when that box straddles the skin the
/// answer is [`Decision::Undecidable`] (`docs/decisions.md` §2.5).
///
/// # Input that is not made of real numbers
///
/// Refused with [`NOTHING_IS_KNOWN`], both halves of it, for the reason
/// [`Cylinder`](crate::Cylinder)'s own predicate gives: a coordinate multiplied
/// by a zero direction component reaches a certified answer it has not earned,
/// and [`Torus::transformed`] cannot fail and can overflow.
impl SideOfSurface for Torus {
    fn side_of(&self, point: PredicatePoint) -> Decision {
        let torus_is_real = are_real_numbers(self.axis.origin().coordinates())
            && are_real_numbers(self.axis.direction().components())
            && are_real_numbers([self.ring_radius, self.tube_radius]);
        if !(is_a_real_point(point) && torus_is_real) {
            return NOTHING_IS_KNOWN;
        }
        decide(&HowFarFromTheRing {
            torus: *self,
            point,
        })
        .0
    }
}

/// `(w·w)(x·x + R² − r²)² − 4R²‖x × w‖²`, evaluated three ways.
struct HowFarFromTheRing {
    torus: Torus,
    point: PredicatePoint,
}

impl Predicate for HowFarFromTheRing {
    fn name(&self) -> &'static str {
        SIDE_OF_TORUS
    }

    fn on_floats(&self) -> Option<Filtered> {
        let point = self.point.exact_coordinates()?;
        let centre = self.torus.axis.origin().coordinates();
        let direction = self.torus.axis.direction().components();
        let ring = self.torus.ring_radius;
        let tube = self.torus.tube_radius;

        let from_centre = [0, 1, 2].map(|axis| point[axis] - centre[axis]);
        let halves = cross_product_halves(from_centre, direction);
        let sideways = halves.map(|(left, right)| left - right);

        // The bound below is a fraction of the permanent, which only bounds
        // anything while every product keeps its relative accuracy. One of them
        // in the subnormals and this level certifies a wrong sign.
        let stays_accurate = (0..3).all(|axis| {
            product_is_normal(from_centre[axis], from_centre[axis])
                && product_is_normal(direction[axis], direction[axis])
                && product_is_normal(sideways[axis], sideways[axis])
        }) && (0..3).all(|axis| {
            let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
            product_is_normal(from_centre[next], direction[after])
                && product_is_normal(from_centre[after], direction[next])
        }) && product_is_normal(ring, ring)
            && product_is_normal(tube, tube);
        if !stays_accurate {
            return None;
        }

        let squares = from_centre.map(|component| component * component);
        let out_and_back = squares[0] + squares[1] + squares[2];
        let hollow = ring * ring - tube * tube;
        let bulge = out_and_back + hollow;
        let axis_squared =
            direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2];
        let across =
            sideways[0] * sideways[0] + sideways[1] * sideways[1] + sideways[2] * sideways[2];
        let four_ring_squared = 4.0 * (ring * ring);
        if !(product_is_normal(bulge, bulge)
            && product_is_normal(axis_squared, bulge * bulge)
            && product_is_normal(four_ring_squared, across))
        {
            return None;
        }

        // Each cross-product component is a difference, so it can cancel and its
        // own magnitude says nothing about the rounding it carries. The
        // permanent has to use the sum of the two halves instead, squared
        // because the component is squared.
        let spread: f64 = halves
            .iter()
            .map(|(left, right)| {
                let reach = left.abs() + right.abs();
                reach * reach
            })
            .sum();
        let bulge_reach = out_and_back + ring * ring + tube * tube;
        let permanent = axis_squared * (bulge_reach * bulge_reach) + four_ring_squared * spread;

        Some(Filtered {
            value: axis_squared * (bulge * bulge) - four_ring_squared * across,
            error_bound: SIDE_OF_TORUS_ERROR_BOUND * permanent,
        })
    }

    fn on_intervals(&self) -> Interval {
        let point = self.point.enclosure();
        let centre = self.torus.axis.origin().coordinates();
        let direction = self.torus.axis.direction().components();

        let ring = self.torus.ring_radius;
        let tube = self.torus.tube_radius;
        let largest = largest_magnitude(&point, &[centre, [ring, tube, 0.0]]);
        let along = direction.map(Interval::exactly);
        let squared_length =
            |value: [Interval; 3]| squared(value[0]) + squared(value[1]) + squared(value[2]);
        let axis_squared = squared_length(along);

        without_overflow(largest, |scale| {
            let from_centre =
                [0, 1, 2].map(|axis| point[axis] * scale - Interval::exactly(centre[axis]) * scale);
            let sideways =
                cross_product_halves(from_centre, along).map(|(left, right)| left - right);
            let ring_squared = squared(Interval::exactly(ring) * scale);
            let tube_squared = squared(Interval::exactly(tube) * scale);
            let bulge = squared_length(from_centre) + ring_squared - tube_squared;
            axis_squared * squared(bulge)
                - Interval::exactly(4.0) * ring_squared * squared_length(sideways)
        })
    }

    fn exactly(&self) -> Option<Decision> {
        let point = self.point.exact_coordinates()?;
        let centre = self.torus.axis.origin().coordinates();
        let direction = self.torus.axis.direction().components();

        let from_centre = Exact::gap_between(centre, point);
        let bulge = Exact::squared_length_of(&from_centre)?
            .plus(&Exact::product_of(
                self.torus.ring_radius,
                self.torus.ring_radius,
            )?)
            .minus(&Exact::product_of(
                self.torus.tube_radius,
                self.torus.tube_radius,
            )?);
        let axis_squared = Exact::dot_of(direction, direction)?;
        let across = Exact::squared_length_of(&Exact::cross_of(&from_centre, direction)?)?;

        // `(w·w)·A² − 4R²·C` has the sign this predicate wants, and multiplying
        // it by the positive `w·w` turns it into a difference of a square and
        // something else — which is what `Exact::square_minus` takes, and the
        // only operation here that slides its arguments into the window where
        // squaring stays exact (`docs/decisions.md` §29.6).
        let squared_bulge = axis_squared.times(&bulge)?;
        let already_squared = across
            .times(&axis_squared)?
            .times(&Exact::product_of(
                self.torus.ring_radius,
                self.torus.ring_radius,
            )?)?
            .times(&Exact::of(4.0))?;

        match squared_bulge.square_minus(&already_squared)?.sign() {
            Decision::Neg => Some(Decision::Neg),
            Decision::Zero => Some(Decision::Zero),
            Decision::Pos => Some(Decision::Pos),
            // A product overflowed. Exact arithmetic that is not exact has
            // nothing to offer, so this level cannot run.
            Decision::Undecidable { .. } => None,
            other => unreachable!("unhandled decision from exact arithmetic: {other:?}"),
        }
    }
}

/// The two halves of each component of `from_centre × along`, left half first.
///
/// Kept apart rather than subtracted so that level 1 can size its error bound
/// against both of them; the difference on its own hides the cancellation.
fn cross_product_halves<T: Copy + core::ops::Mul<Output = T>>(
    from_centre: [T; 3],
    along: [T; 3],
) -> [(T, T); 3] {
    [
        (from_centre[1] * along[2], from_centre[2] * along[1]),
        (from_centre[2] * along[0], from_centre[0] * along[2]),
        (from_centre[0] * along[1], from_centre[1] * along[0]),
    ]
}

/// Level 1's forward error bound, as a multiple of the permanent.
///
/// `epsilon` is the `f64` unit roundoff, 2⁻⁵³.
///
/// The chain, counted step by step. `point − centre` is one rounding; squaring a
/// component carries it twice and rounds, a third; the two sums that fold the
/// three squares are a fourth and fifth. `R²`, `r²` and their difference run
/// beside that sum rather than after it, so adding them on is a sixth. Squaring
/// the total doubles what it carries and rounds, so it sits at
/// `(1 + epsilon)^13`. `w·w` is three roundings of its own, and multiplying it
/// in carries those three and rounds once more, a seventeenth. The other side
/// is shorter — a cross-product component is three roundings, its square a
/// seventh, the two sums a ninth, and `4R²` times it an eleventh, `R²` carrying
/// one and the `4` exact — so the final subtraction, the eighteenth, is what
/// the whole thing is charged at.
///
/// `(1 + epsilon)^18 − 1 ≤ 18·epsilon + 153·epsilon² + …` times the permanent.
/// The `220·epsilon²`, which the constant's own sum rounds up to
/// `224·epsilon²`, was meant to cover the higher-order terms and the rounding of
/// the permanent itself, and by that count it falls short. The higher-order
/// terms start at `153·epsilon²`. The permanent's own roundings can put it up
/// to about `7·epsilon` under its true value, which costs `126·epsilon²` against
/// the `18·epsilon`, and multiplying it by this constant rounds once more, which
/// costs `epsilon` of the bound and so `18·epsilon²` of the permanent:
/// `153 + 126 + 18 = 297`, above the `224`.
///
/// That count is loose, and the constant holds where `decide` needs it: on the
/// sum before `value`'s last rounding, seventeen roundings deep. Gappa proves
/// the bound there at this constant (`docs/decisions.md` §464.17), whether or
/// not a component of the cross product cancels, whatever its reach square
/// rounds to, and whether `R² − r²` is normal or below the normals. That sum is
/// within `17·epsilon + 95·epsilon²` of the permanent with no rounding of its
/// own, and with nothing cancelling the least constant it proves is
/// `17·epsilon + 232·epsilon²`. Nobody has tried the claim on `value` itself.
/// The proof covers no zero difference or direction component, which have not
/// been run; a point on the axis of a torus whose axis is a coordinate axis is
/// one.
///
/// Every step of that argument is a *relative* one, so as with every bound of
/// this shape it holds only while nothing overflows or falls into the
/// subnormals. `decide` keeps out the top end by refusing a bound that is not
/// itself a normal number. [`HowFarFromTheRing::on_floats`] checks the bottom
/// end through [`product_is_normal`] for the products of `value`, but for none
/// of the permanent's own: not the reach squares, not `bulge_reach` squared,
/// and neither of the two products the permanent adds. `bulge_reach` squared
/// and `w·w` times it stay normal all the same. But a check passes on a zero,
/// so a component that cancels to zero lets its reach square fall below the
/// normals or round to zero, and where all three cancel, `4R²` times their sum
/// can too. The bound on the sum holds in each of those (§464.17's 6).
const SIDE_OF_TORUS_ERROR_BOUND: f64 = {
    let epsilon = f64::EPSILON / 2.0;
    (18.0 + 220.0 * epsilon) * epsilon
};

impl TryFrom<(Axis, f64, f64)> for Torus {
    type Error = Error;

    fn try_from((axis, ring_radius, tube_radius): (Axis, f64, f64)) -> Result<Self> {
        Self::new(axis, ring_radius, tube_radius)
    }
}

impl From<Torus> for (Axis, f64, f64) {
    fn from(torus: Torus) -> Self {
        (torus.axis, torus.ring_radius, torus.tube_radius)
    }
}

#[cfg(test)]
mod tests {
    use rcad_predicates::{Level, decide, meet_of_three_planes};

    use super::*;
    use crate::{Point, Vector};

    fn a_ring_of_four_and_one() -> Torus {
        Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring")
    }

    #[test]
    fn a_torus_needs_two_radii_that_are_sizes() {
        for radius in [0.0, -1.0, -0.0, f64::NAN, f64::INFINITY] {
            match Torus::new(Axis::z(), radius, 1.0) {
                Err(Error::RadiusNotPositive { radius: refused }) => {
                    assert_eq!(refused.to_bits(), radius.to_bits());
                }
                other => panic!("a ring radius of {radius} is not a size: {other:?}"),
            }
            match Torus::new(Axis::z(), 4.0, radius) {
                Err(Error::RadiusNotPositive { radius: refused }) => {
                    assert_eq!(refused.to_bits(), radius.to_bits());
                }
                other => panic!("a tube radius of {radius} is not a size: {other:?}"),
            }
        }
    }

    /// A tube as wide as its ring pinches on the axis and a wider one runs
    /// through itself. Both are refused, and the equal case is refused too.
    #[test]
    fn a_tube_as_wide_as_its_ring_or_wider_is_refused() {
        for tube in [4.0, 5.0] {
            match Torus::new(Axis::z(), 4.0, tube) {
                Err(Error::ATorusThatRunsThroughItself {
                    ring_radius,
                    tube_radius,
                }) => assert_eq!((ring_radius, tube_radius), (4.0, tube)),
                other => panic!("a tube of {tube} on a ring of 4 runs through itself: {other:?}"),
            }
        }
    }

    /// **The demonstration behind the refusal above**, and the reason it is a
    /// refusal rather than a note.
    ///
    /// The quartic is written out here by hand for a torus that
    /// [`Torus::new`] will not build: ring 1, tube 2, about the `z` axis. The
    /// point `(0, 0, 0)` is plainly inside that tube — it is one unit from the
    /// centre circle and the tube is two wide — and the quartic comes out
    /// **positive** there, which every level of the cascade would certify as
    /// *outside*. That is a confident wrong answer, which `docs/decisions.md`
    /// §4.1 ranks below a crash, and it is what the constructor's `Err` exists
    /// to stop.
    #[test]
    fn a_tube_wider_than_its_ring_would_certify_the_wrong_side() {
        let (ring, tube) = (1.0f64, 2.0f64);
        assert!(Torus::new(Axis::z(), ring, tube).is_err());

        // The centre of the hole, written out through the same expression the
        // predicate uses: x = 0, so ‖x × w‖² = 0 and x·x = 0.
        let bulge = 0.0 + ring * ring - tube * tube;
        let quartic = 1.0 * (bulge * bulge) - 4.0 * ring * ring * 0.0;
        assert!(
            quartic > 0.0,
            "the quartic says outside for a point one unit from a centre circle whose tube is \
             two wide"
        );
    }

    #[test]
    fn moving_a_torus_moves_its_centre_and_leaves_both_radii_alone() {
        let torus = a_ring_of_four_and_one();
        let shift = Isometry::translation(Vector::new(4.0, 0.0, 0.0)).expect("a finite shift");
        let moved = torus.transformed(&shift);
        assert_eq!(moved.centre(), Point::new(4.0, 0.0, 0.0));
        assert_eq!(moved.axis().direction(), Vector::new(0.0, 0.0, 1.0));
        assert_eq!((moved.ring_radius(), moved.tube_radius()), (4.0, 1.0));
    }

    /// The four places the `x` axis meets a ring of four and one, and the three
    /// gaps between them. A straight line meets a torus **four** times, which is
    /// the whole of what makes it not a quadric.
    #[test]
    fn a_line_through_a_ring_meets_it_four_times_and_a_quadric_twice() {
        let torus = a_ring_of_four_and_one();
        let on_the_skin = [-5.0, -3.0, 3.0, 5.0];
        for x in on_the_skin {
            assert_eq!(
                torus.side_of(PredicatePoint::from([x, 0.0, 0.0])),
                Decision::Zero,
                "at x = {x}"
            );
        }
        for (x, side) in [
            (-6.0, Decision::Pos),
            (-4.0, Decision::Neg),
            (0.0, Decision::Pos),
            (4.0, Decision::Neg),
            (6.0, Decision::Pos),
        ] {
            assert_eq!(
                torus.side_of(PredicatePoint::from([x, 0.0, 0.0])),
                side,
                "at x = {x}"
            );
        }
    }

    /// The middle of the hole is outside the material, and so is a point far
    /// away up the axis. A ball and a pipe have one inside; a torus's is a ring.
    #[test]
    fn the_hole_and_the_outside_are_both_outside() {
        let torus = a_ring_of_four_and_one();
        for point in [[0.0, 0.0, 0.0], [0.0, 0.0, 100.0], [100.0, 0.0, 0.0]] {
            assert_eq!(
                torus.side_of(PredicatePoint::from(point)),
                Decision::Pos,
                "at {point:?}"
            );
        }
    }

    #[test]
    fn a_point_clear_of_the_skin_is_settled_by_the_cheapest_level() {
        let (decision, level) = decide(&HowFarFromTheRing {
            torus: a_ring_of_four_and_one(),
            point: PredicatePoint::from([100.0, 0.0, 0.0]),
        });
        assert_eq!(decision, Decision::Pos);
        assert_eq!(level, Level::Float);
    }

    /// **Two** Pythagorean triples stacked, and a scale that is not a power of
    /// two, so the quartic runs well past 53 bits and only exact arithmetic can
    /// see the point is on the skin.
    ///
    /// `(6t, 8t, 4t)` sits `10t` from the `z` axis — the `3-4-5` triple —
    /// which is `3t` out from a centre circle of radius `7t`, and `3t` out and
    /// `4t` up is `5t` away, which is the tube. The scale `t` is odd, so nothing
    /// here is a power of two and the fourth power of it is around `2^93`.
    #[test]
    fn a_point_on_a_ring_too_big_to_see_in_f64_still_comes_out_exactly_on_it() {
        let scale = 1_234_567.0f64;
        let torus =
            Torus::new(Axis::z(), 7.0 * scale, 5.0 * scale).expect("a tube narrower than its ring");
        let (decision, level) = decide(&HowFarFromTheRing {
            torus,
            point: PredicatePoint::from([6.0 * scale, 8.0 * scale, 4.0 * scale]),
        });
        assert_eq!(decision, Decision::Zero);
        assert_eq!(level, Level::Exact);
    }

    /// The three planes meet at `(1/3, 1/3, 1/3)`, which is not an `f64`, and no
    /// box around it is narrow enough to be placed against a surface it sits on.
    #[test]
    fn a_constructed_point_sitting_on_the_skin_cannot_be_placed() {
        let meet = meet_of_three_planes(
            [1.0, 1.0, 1.0, -1.0],
            [1.0, -1.0, 0.0, 0.0],
            [0.0, 1.0, -1.0, 0.0],
        )
        .expect("three planes that meet at a point");
        // The point sits √2/3 from the `z` axis and 1/3 up it, so the torus
        // through it has a tube radius of exactly that distance from the centre
        // circle of radius √2/3 — which is 1/3.
        let torus = Torus::new(Axis::z(), 2.0f64.sqrt() / 3.0, 1.0 / 3.0)
            .expect("a tube narrower than its ring");

        match torus.side_of(meet) {
            Decision::Neg | Decision::Zero | Decision::Pos => {
                panic!("neither the box nor the radii are exact enough to place this")
            }
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
            other => panic!("unhandled decision: {other:?}"),
        }
    }

    #[test]
    fn a_constructed_point_well_inside_is_still_decided() {
        let meet = meet_of_three_planes(
            [1.0, 1.0, 1.0, -1.0],
            [1.0, -1.0, 0.0, 0.0],
            [0.0, 1.0, -1.0, 0.0],
        )
        .expect("three planes that meet at a point");

        // The point sits `√2/3 ≈ 0.471` off the `z` axis at a height of `1/3`,
        // so it is `0.334` from a centre circle of radius `0.5`, and a tube of
        // `0.4` swallows it with room to spare.
        let torus = Torus::new(Axis::z(), 0.5, 0.4).expect("a tube narrower than its ring");
        match torus.side_of(meet) {
            Decision::Neg => (),
            other => panic!("a box around (1/3, 1/3, 1/3) is inside this tube: {other:?}"),
        }
    }

    #[test]
    fn a_torus_about_a_slanted_axis_still_places_points_on_its_skin() {
        let slanted = Axis::new(Point::origin(), Vector::new(0.0, 0.0, 2.0))
            .expect("a direction with length");
        let torus = Torus::new(slanted, 4.0, 1.0).expect("a tube narrower than its ring");
        assert_eq!(
            torus.side_of(PredicatePoint::from([5.0, 0.0, 0.0])),
            Decision::Zero,
            "the stored direction is scaled to length one, so this is the z axis again"
        );
    }

    /// A point that is not a point is neither in the tube nor out of it.
    #[test]
    fn a_point_that_is_not_a_point_is_neither_in_nor_out() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for axis in 0..3 {
                let mut broken = [5.0, 0.0, 1.0];
                broken[axis] = bad;
                assert_eq!(
                    a_ring_of_four_and_one().side_of(PredicatePoint::from(broken)),
                    Decision::Undecidable {
                        enclosure: f64::INFINITY
                    },
                    "{bad} on axis {axis}"
                );
            }
        }
    }

    /// [`Torus::transformed`] cannot fail and can overflow, so a torus built
    /// from finite numbers can be holding an infinite centre by the time it gets
    /// here.
    #[test]
    fn a_torus_shoved_past_the_end_of_the_number_line_has_no_inside() {
        let far = 1e308;
        let shove = Isometry::translation(Vector::new(far, 0.0, 0.0)).expect("a finite shift");
        let axis = Axis::new(Point::new(far, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
            .expect("a direction with length");
        let shoved = Torus::new(axis, 4.0, 1.0)
            .expect("a tube narrower than its ring")
            .transformed(&shove);

        assert!(!shoved.centre().is_finite(), "the centre overflowed");
        assert_eq!(
            shoved.side_of(PredicatePoint::from([0.0, 5.0, 0.0])),
            Decision::Undecidable {
                enclosure: f64::INFINITY
            }
        );
    }

    /// The two coordinates come back to the point they were read from, where
    /// the trigonometry lands exactly.
    #[test]
    fn a_point_on_the_outer_equator_reads_back_to_itself() {
        let torus = a_ring_of_four_and_one();
        let on_it = Point::new(5.0, 0.0, 0.0);
        let (over, round) = torus.coordinates_of(on_it);
        assert_eq!((over, round), (0.0, 0.0));
        assert_eq!(torus.point_at(over, round), on_it);
    }

    /// Round the tube is the first coordinate and round the ring the second,
    /// which is what makes a bend of pipe the same shape in its chart as a
    /// straight bar. The inner equator is half a turn round the tube and no way
    /// round the ring.
    #[test]
    fn the_first_coordinate_is_the_one_that_goes_round_the_tube() {
        let torus = a_ring_of_four_and_one();
        let (over, round) = torus.coordinates_of(Point::new(3.0, 0.0, 0.0));
        assert_eq!((over, round), (core::f64::consts::PI, 0.0));
        let (over, round) = torus.coordinates_of(Point::new(0.0, 5.0, 0.0));
        assert_eq!((over, round), (0.0, core::f64::consts::FRAC_PI_2));
    }

    /// Neither coordinate ever pinches, which is what a ball's and a cone's do.
    /// The step a unit of the first takes is never less than the gap between the
    /// two radii.
    #[test]
    fn a_step_round_the_ring_never_shrinks_to_nothing() {
        let torus = a_ring_of_four_and_one();
        for over in [0.0, 1.0, core::f64::consts::PI, -2.0] {
            assert!(
                torus.distance_from_the_axis_at(over) >= torus.ring_radius() - torus.tube_radius(),
                "at {over}"
            );
        }
        assert_eq!(torus.distance_from_the_axis_at(0.0), 5.0);
        assert_eq!(torus.distance_from_the_axis_at(core::f64::consts::PI), 3.0);
    }
}
