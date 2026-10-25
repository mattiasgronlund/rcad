//! Round pipe surfaces.

use rcad_predicates::expansion::{
    expansion_sign, expansion_sum, product_is_exact, scale_expansion, two_product,
};
use rcad_predicates::{
    Decision, Filtered, Interval, NOTHING_IS_KNOWN, Point as PredicatePoint, Predicate,
    SideOfSurface, are_real_numbers, decide, is_a_real_point, product_is_normal,
};
use serde::{Deserialize, Serialize};

use crate::enclosure::{dot_enclosing, exactly};
use crate::exact::{Exact, difference, square_of_expansion};
use crate::intersect::{Question, WHERE_A_PLANE_AND_A_PIPE_CROSS};
use crate::isometry::Isometry;
use crate::plane::Plane;
use crate::point::{Axis, Point, Vector};
use crate::sphere::{largest_magnitude, squared, without_overflow};
use crate::{Error, Result};

/// The name the cylinder's sign predicate goes by in provenance records.
pub const SIDE_OF_CYLINDER: &str = "side of cylinder";

/// An endless round pipe's skin: every point exactly `radius` away from `axis`.
///
/// Ships as a type and a sign predicate, for the same reason as
/// [`Sphere`](crate::Sphere): nothing in M0 builds a face on one, so
/// [`SideOfSurface::side_of`] is the whole of what it does. There is
/// deliberately no `signed_distance_to`.
///
/// Serializes as `(axis, radius)` and is re-checked on the way in.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "(Axis, f64)", into = "(Axis, f64)")]
pub struct Cylinder {
    axis: Axis,
    radius: f64,
}

impl Cylinder {
    /// The pipe of the given radius around `axis`.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `axis` is not made of finite numbers, and
    /// [`Error::RadiusNotPositive`] if the radius is zero, negative, or not a
    /// finite number.
    pub fn new(axis: Axis, radius: f64) -> Result<Self> {
        if !axis.origin().is_finite() || !axis.direction().is_finite() {
            return Err(Error::NotFinite {
                what: "cylinder axis",
            });
        }
        if !(radius.is_finite() && radius > 0.0) {
            return Err(Error::RadiusNotPositive { radius });
        }
        Ok(Self { axis, radius })
    }

    /// The line down the middle of the pipe.
    #[must_use]
    pub fn axis(&self) -> Axis {
        self.axis
    }

    /// How far the surface is from the axis. Always greater than zero.
    #[must_use]
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// The same pipe after `movement` is applied. A rigid movement cannot
    /// change the radius.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        Self {
            axis: self.axis.transformed(movement),
            radius: self.radius,
        }
    }

    /// Two directions across the pipe, at right angles to each other and to the
    /// axis, with the first crossed into the second giving the axis back.
    ///
    /// The choice is arbitrary but fixed, so the same pipe always gives the
    /// same pair. The first one is where `u = 0` sits, and for a pipe on the
    /// `z` axis it is `+x` — which is where a round bar puts its seam, so that
    /// face's `(u, v)` rectangle has the seam down its edge rather than
    /// somewhere across the middle of it.
    ///
    /// Exact when the axis is a coordinate direction, which is what every
    /// generator in this milestone draws. Off-axis it is a normalize like any
    /// other and rounds (`docs/decisions.md` §16.8).
    ///
    /// # Panics
    ///
    /// If the stored axis direction has no length. That never happens:
    /// [`Axis::new`] checks it, [`Axis::as_written`] keeps a direction only
    /// when its length is within a hair of one and hands any other to
    /// [`Axis::new`], and [`Axis::x`]/[`Axis::y`]/[`Axis::z`] each build one
    /// straight from a literal unit vector, so no public constructor can hand
    /// back a zero direction. `Deserialize` goes through [`Axis::as_written`] —
    /// `#[serde(try_from = "(Point, Vector)")]` on `Axis` makes it the route
    /// in on the wire.
    /// `a_pipe_stored_with_a_zero_length_axis_direction_is_refused_on_the_way_in`
    /// (`crates/rcad-geom/tests/serialization.rs`) shows the bytes refused
    /// before a `Cylinder` exists to ask.
    ///
    /// **There is one more route in and it is not a constructor.**
    /// [`Cylinder::transformed`] goes through `Axis::transformed`, which does
    /// not re-normalize — deliberately, so a direction that was exactly square
    /// to something stays exactly square to it (`docs/decisions.md` §32.3). What
    /// keeps that safe is two types away: an [`Isometry`] turns by a quaternion
    /// scaled to length one, whatever scale its four coefficients arrived at, so
    /// it never shrinks a direction to nothing.
    /// `a_rotation_written_at_any_scale_is_the_same_rotation` is that
    /// demonstration, and §41.4 is why an unreachability claim has to name every
    /// route rather than the constructors it thought of.
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
        // Taking the part of that axis lying across the pipe, rather than
        // crossing with it, is what puts `u = 0` on `+x` for a pipe on `z`. A
        // cross product would land it on `+y`, and then the seam of a bar would
        // not be on the edge of its own rectangle.
        let across = least_aligned_axis - along * along.dot(least_aligned_axis);
        let u = across
            .normalized_as("direction across a cylinder")
            .expect("a unit axis is never parallel to its least aligned coordinate axis");
        (u, along.cross(u))
    }

    /// The point at angle `round` radians from [`Cylinder::basis`]'s first
    /// direction, `along` up the axis from its origin.
    ///
    /// Only exactly on the pipe where the sine and cosine are exact, which in
    /// practice means `round = 0`. Everywhere else this is a point *near* the
    /// surface, which is why no corner of a face is ever made this way —
    /// corners come from exact constructions and get certified. This is for
    /// filling in a mesh between them, where being near is the whole idea and
    /// the deviation is what gets bounded.
    #[must_use]
    pub fn point_at(&self, round: f64, along: f64) -> Point {
        let (u, v) = self.basis();
        self.axis.origin()
            + (u * round.cos() + v * round.sin()) * self.radius
            + self.axis.direction() * along
    }

    /// Which angle round and how far along `point` sits at —
    /// [`Cylinder::point_at`] the other way round.
    ///
    /// Only the *direction* across the pipe is read, so a point off the skin
    /// gets the coordinates of the point on the skin nearest it. How far from
    /// the axis it was is thrown away; how far along the axis it sits is not,
    /// and comes back the same for a point off the skin as for the one on it.
    ///
    /// The angle comes back on `atan2`'s branch, `(-π, π]`, and how close it
    /// gets is [`Surface::ROUND_TRIP_ROUNDOFF`](crate::Surface::ROUND_TRIP_ROUNDOFF).
    ///
    /// **A point exactly on the seam has one answer here and not two.** The
    /// seam is where [`Cylinder::basis`] puts `u = 0`, and a point on it comes
    /// back `0` and never `2π`, because `atan2(+0, positive)` is `+0`. A wire
    /// that goes all the way round wants its far seam corner at `2π`, and
    /// putting it there is the job of whoever knows the wire is a loop —
    /// carrying a previous angle in here would make a pipe's answer depend on
    /// the order it was asked in (`docs/decisions.md` §25.2).
    ///
    /// A point on the axis has no nearest point on the skin, and its angle is
    /// not determined by the geometry at all. Zero is what `atan2(0, 0)` hands
    /// over, without a branch.
    #[must_use]
    pub fn coordinates_of(&self, point: Point) -> (f64, f64) {
        let (across, and) = self.basis();
        let from_origin = point - self.axis.origin();
        (
            from_origin.dot(and).atan2(from_origin.dot(across)),
            from_origin.dot(self.axis.direction()),
        )
    }

    /// Whether `flat` cuts this pipe in a curve that **closes**; [`None`] when
    /// the sign cannot be proved.
    ///
    /// [`Cone::is_cut_in_a_closed_curve_by`](crate::Cone::is_cut_in_a_closed_curve_by)'s
    /// question asked of a pipe, and it is the easier of the two. A pipe has no
    /// apex to dispose of first and no nappe on the far side, so there is one
    /// sign and nothing else: the plane's stored normal `n` against the stored
    /// axis direction `a`.
    ///
    /// ```text
    /// n·a ≠ 0
    /// ```
    ///
    /// Above or below zero the plane cuts across the pipe and leaves one closed
    /// curve — an ellipse in general, a circle when the plane is square to the
    /// axis, and neither is drawn so neither needs telling apart (§24.5).
    /// Exactly zero and the plane holds the axis's direction, so what it leaves
    /// is two endless lines, one, or nothing, and none of those closes.
    ///
    /// Three products of stored `f64`s summed with nothing thrown away, so no
    /// vector is normalised and nothing is rounded on the way. `n·a` rounding
    /// to nothing is not a proof that it is nothing, and
    /// `arrange.rs`'s `a_plane_lies_along_a_pipes_axis` carries what a rounded
    /// version of this same sign cost.
    #[must_use]
    pub fn is_cut_in_a_closed_curve_by(&self, flat: &Plane) -> Option<bool> {
        let normal = flat.normal().components();
        let along = self.axis.direction().components();
        let leaning = Question {
            named: WHERE_A_PLANE_AND_A_PIPE_CROSS,
            enclosing: move || dot_enclosing(exactly(normal), exactly(along)),
            exactly: move || Exact::dot_of(normal, along),
        };
        match decide(&leaning).0 {
            Decision::Pos | Decision::Neg => Some(true),
            Decision::Zero => Some(false),
            Decision::Undecidable { .. } | _ => None,
        }
    }
}

/// Inside, on, or outside, decided through the cascade.
///
/// The sign of `|(point - axis origin) × axis direction|² - radius²·|axis
/// direction|²`: [`Decision::Neg`] inside the pipe, [`Decision::Pos`] outside,
/// [`Decision::Zero`] exactly on the skin.
///
/// A point with exact `f64` coordinates always gets a certified answer, as long
/// as no product overflows or falls into the subnormals. A constructed point is
/// only known to lie inside a box, so when that box straddles the skin the
/// answer is [`Decision::Undecidable`] (`docs/decisions.md` §2.5).
///
/// # What is certified here
///
/// The pipe of radius exactly `radius`, whatever the stored direction's length
/// turned out to be.
///
/// Both sides of the comparison are degree two in the direction, so scaling the
/// direction by any number at all scales the whole value by the square of it and
/// leaves every sign alone. That matters because the direction is *not* a unit
/// vector: [`Axis::new`] divides by a length and the division rounds, and
/// [`Cylinder::transformed`] does not re-normalize at all (§32.3). Until
/// `docs/decisions.md` §118.10 this compared against a bare `radius²`, which is
/// degree zero, so the surface certified was the pipe of radius
/// `radius / length(stored direction)` — and two pipes written with the same
/// radius literal were then the same width only when their directions happened
/// to round to the same length.
///
/// Every answer below is still only exact about the numbers as stored, and a
/// plane's offset was measured against a normal that had already rounded — so
/// [`Plane::side_of_point`](crate::Plane::side_of_point) keeps its own version
/// of the caveat, which this one no longer shares.
///
/// # Input that is not made of real numbers
///
/// Refused with [`NOTHING_IS_KNOWN`], and both halves of the input can need it.
/// A point whose coordinate along the axis is not a real number reached a
/// certified answer, because that coordinate only ever enters the cross product
/// multiplied by a direction component, and for an axis-aligned pipe two of
/// those are zero. So did a pipe whose own axis origin had overflowed —
/// [`Cylinder::transformed`] cannot fail and can overflow, so a cylinder built
/// from finite numbers can be holding an infinite origin by the time it gets
/// here.
impl SideOfSurface for Cylinder {
    fn side_of(&self, point: PredicatePoint) -> Decision {
        let pipe_is_real = are_real_numbers(self.axis.origin().coordinates())
            && are_real_numbers(self.axis.direction().components())
            && are_real_numbers([self.radius]);
        if !(is_a_real_point(point) && pipe_is_real) {
            return NOTHING_IS_KNOWN;
        }
        decide(&HowFarFromTheSkin {
            cylinder: *self,
            point,
        })
        .0
    }
}

/// `|(point - origin) × direction|² - radius²·|direction|²`, evaluated three
/// ways.
struct HowFarFromTheSkin {
    cylinder: Cylinder,
    point: PredicatePoint,
}

impl Predicate for HowFarFromTheSkin {
    fn name(&self) -> &'static str {
        SIDE_OF_CYLINDER
    }

    fn on_floats(&self) -> Option<Filtered> {
        let point = self.point.exact_coordinates()?;
        let origin = self.cylinder.axis.origin().coordinates();
        let direction = self.cylinder.axis.direction().components();

        let from_origin = [0, 1, 2].map(|axis| point[axis] - origin[axis]);
        let radius = self.cylinder.radius;

        // The bound below is a fraction of the permanent, which only bounds
        // anything while every product keeps its relative accuracy. One of them
        // in the subnormals and this level certifies a wrong sign.
        //
        // The permanent squares `|left| + |right|` where the value squares
        // `left - right`, and the first is never below the second, so where a
        // component does not cancel, checking it here covers its reach square
        // too. One that cancels to zero, `left == right`, passes the check,
        // since `product_is_normal` passes a zero, and its reach square
        // `4 left²` is checked by nothing and can fall below the normals or
        // round to zero. The bound holds there all the same
        // (`docs/decisions.md` §464.15), so a wrong sign does not follow from
        // a reach square in the subnormals. Whether one follows from a product
        // checked here has not been run.
        let halves = cross_product_halves_if_normal(from_origin, direction)?;
        let components = halves.map(|(left, right)| left - right);
        let direction_squared =
            direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2];
        let stays_accurate = components
            .iter()
            .all(|&component| product_is_normal(component, component))
            && direction
                .iter()
                .all(|&along| product_is_normal(along, along))
            && product_is_normal(radius, radius)
            && product_is_normal(radius * radius, direction_squared);
        if !stays_accurate {
            return None;
        }

        let skin = radius * radius * direction_squared;
        let squares = components.map(|component| component * component);

        // Each cross-product component is a difference, so it can cancel and
        // its own magnitude says nothing about the rounding it carries. The
        // permanent has to use the sum of the two halves instead, squared
        // because the component is squared. The radius term is a sum of
        // positives, so it stands for its own size.
        let spread = halves.map(|(left, right)| {
            let reach = left.abs() + right.abs();
            reach * reach
        });
        let permanent = spread[0] + spread[1] + spread[2] + skin;

        Some(Filtered {
            value: squares[0] + squares[1] + squares[2] - skin,
            error_bound: SIDE_OF_CYLINDER_ERROR_BOUND * permanent,
        })
    }

    fn on_intervals(&self) -> Interval {
        let point = self.point.enclosure();
        let origin = self.cylinder.axis.origin().coordinates();
        let direction = self.cylinder.axis.direction().components();

        let largest = largest_magnitude(&point, &[origin, [self.cylinder.radius, 0.0, 0.0]]);
        let along = direction.map(Interval::exactly);
        let direction_squared = squared(along[0]) + squared(along[1]) + squared(along[2]);

        without_overflow(largest, |scale| {
            let from_origin =
                [0, 1, 2].map(|axis| point[axis] * scale - Interval::exactly(origin[axis]) * scale);
            let halves = cross_product_halves(from_origin, along);
            let squared_component = |axis: usize| {
                let (left, right) = halves[axis];
                squared(left - right)
            };
            let radius = Interval::exactly(self.cylinder.radius) * scale;
            squared_component(0) + squared_component(1) + squared_component(2)
                - squared(radius) * direction_squared
        })
    }

    fn exactly(&self) -> Option<Decision> {
        let point = self.point.exact_coordinates()?;
        let origin = self.cylinder.axis.origin().coordinates();
        let direction = self.cylinder.axis.direction().components();
        let from_origin = [0, 1, 2].map(|axis| difference(point[axis], origin[axis]));

        let mut total = [0.0f64; LONGEST_CYLINDER_TOTAL];
        let mut total_len = 0;
        let mut merged = [0.0f64; LONGEST_CYLINDER_TOTAL];
        let mut component = [0.0f64; LONGEST_CROSS_COMPONENT];
        let mut squared = [0.0f64; LONGEST_SQUARED_COMPONENT];

        for axis in 0..3 {
            let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
            let component_len = cross_component(
                from_origin[next],
                direction[after],
                from_origin[after],
                direction[next],
                &mut component,
            )?;
            let squared_len = square_of_expansion(&component[..component_len], &mut squared)?;
            total_len = expansion_sum(&total[..total_len], &squared[..squared_len], &mut merged);
            core::mem::swap(&mut total, &mut merged);
        }

        // `r²‖w‖²`, exactly: the three squares of the stored direction summed,
        // then scaled by each half of `r²` in turn. Every step declines rather
        // than rounds.
        let mut direction_squared = [0.0f64; LONGEST_DIRECTION_SQUARED];
        let mut direction_squared_len = 0;
        let mut direction_merged = [0.0f64; LONGEST_DIRECTION_SQUARED];
        for along in direction {
            if !product_is_exact(along, along) {
                return None;
            }
            let (square, square_roundoff) = two_product(along, along);
            direction_squared_len = expansion_sum(
                &direction_squared[..direction_squared_len],
                &[square_roundoff, square],
                &mut direction_merged,
            );
            core::mem::swap(&mut direction_squared, &mut direction_merged);
        }

        let radius = self.cylinder.radius;
        if !product_is_exact(radius, radius) {
            return None;
        }
        let (radius_squared, radius_roundoff) = two_product(radius, radius);
        if !direction_squared[..direction_squared_len]
            .iter()
            .all(|&entry| {
                product_is_exact(entry, radius_squared) && product_is_exact(entry, radius_roundoff)
            })
        {
            return None;
        }
        let mut by_square = [0.0f64; 2 * LONGEST_DIRECTION_SQUARED];
        let by_square_len = scale_expansion(
            &direction_squared[..direction_squared_len],
            -radius_squared,
            &mut by_square,
        );
        let mut by_roundoff = [0.0f64; 2 * LONGEST_DIRECTION_SQUARED];
        let by_roundoff_len = scale_expansion(
            &direction_squared[..direction_squared_len],
            -radius_roundoff,
            &mut by_roundoff,
        );
        let mut skin = [0.0f64; LONGEST_RADIUS_TERM];
        let skin_len = expansion_sum(
            &by_square[..by_square_len],
            &by_roundoff[..by_roundoff_len],
            &mut skin,
        );

        total_len = expansion_sum(&total[..total_len], &skin[..skin_len], &mut merged);

        match expansion_sign(&merged[..total_len]) {
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

/// [`cross_product_halves`] in `f64`, or `None` when one of the six products
/// fell into the subnormals and stopped being relatively accurate.
fn cross_product_halves_if_normal(
    from_origin: [f64; 3],
    along: [f64; 3],
) -> Option<[(f64, f64); 3]> {
    for axis in 0..3 {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        if !product_is_normal(from_origin[next], along[after])
            || !product_is_normal(from_origin[after], along[next])
        {
            return None;
        }
    }
    Some(cross_product_halves(from_origin, along))
}

/// The two halves of each component of `from_origin × along`, left half first.
///
/// Kept apart rather than subtracted so that level 1 can size its error bound
/// against both of them; the difference on its own hides the cancellation.
fn cross_product_halves<T: Copy + core::ops::Mul<Output = T>>(
    from_origin: [T; 3],
    along: [T; 3],
) -> [(T, T); 3] {
    [
        (from_origin[1] * along[2], from_origin[2] * along[1]),
        (from_origin[2] * along[0], from_origin[0] * along[2]),
        (from_origin[0] * along[1], from_origin[1] * along[0]),
    ]
}

/// `first * first_factor - second * second_factor`, exactly.
///
/// `None` when one of the products would round, which is what
/// [`product_is_exact`] is for.
fn cross_component(
    first: [f64; 2],
    first_factor: f64,
    second: [f64; 2],
    second_factor: f64,
    out: &mut [f64],
) -> Option<usize> {
    let exact = first
        .iter()
        .all(|&entry| product_is_exact(entry, first_factor))
        && second
            .iter()
            .all(|&entry| product_is_exact(entry, second_factor));
    if !exact {
        return None;
    }

    let mut scaled_first = [0.0f64; 4];
    let first_len = scale_expansion(&first, first_factor, &mut scaled_first);
    let mut scaled_second = [0.0f64; 4];
    let second_len = scale_expansion(&second, -second_factor, &mut scaled_second);
    Some(expansion_sum(
        &scaled_first[..first_len],
        &scaled_second[..second_len],
        out,
    ))
}

/// Level 1's forward error bound, as a multiple of the permanent.
///
/// `epsilon` is the `f64` unit roundoff, 2⁻⁵³.
///
/// Where the 10 comes from: follow the longest chain of roundings from an input
/// coordinate to `value` in [`HowFarFromTheSkin::on_floats`]. The step
/// `point - origin` is one rounding. Multiplying it by a direction component is
/// a second, and it carries the first along, so the half is at `(1 + epsilon)²`.
/// Subtracting the two halves is a third. Squaring that component is a fourth,
/// and it multiplies two copies of a value already at `(1 + epsilon)³`, so the
/// square sits at `(1 + epsilon)⁷`. Two additions to fold the three squares
/// together make eight and nine, and subtracting the radius term makes ten.
/// `(1 + epsilon)^10 - 1 <= 10*epsilon + 45*epsilon^2 + ...` times the
/// permanent.
///
/// The radius term `radius² * |direction|²` sits at `(1 + epsilon)^6`, so the
/// ten covers it too. Squaring a direction component is one rounding and the two
/// sums that fold the three squares together are a second and third, putting the
/// squared length at `(1 + epsilon)³`; `radius * radius` is one rounding on its
/// own; multiplying those two carries both and adds a rounding, which is five;
/// and the subtraction that made ten on the cross-product side makes six here.
///
/// The `80*epsilon^2` was meant to cover the higher-order terms and the
/// rounding of the permanent itself, and by that count it falls short. The
/// higher-order terms start at `45*epsilon^2`. The permanent's own roundings
/// can put it up to about `6*epsilon` under its true value, which costs
/// `60*epsilon^2` against the `10*epsilon`, and multiplying it by this constant
/// rounds once more, which costs `epsilon` of the bound and so `10*epsilon^2`
/// of the permanent: `45 + 60 + 10 = 115`, above the `80`.
///
/// That count is loose, and the constant holds where `decide` needs it: on the
/// sum before `value`'s last rounding, whose longest chain is nine roundings
/// rather than ten. Gappa proves the bound there at this constant
/// (`docs/decisions.md` §464.15), whether or not a component of the cross
/// product cancels, and with none cancelling the least constant it proves is
/// `9*epsilon + 70*epsilon^2`. On `value` itself the two forms tried prove the
/// bound only at
/// `10*epsilon + 82*epsilon^2`, two `epsilon^2` above this constant, and nobody
/// has looked for an input where `value` lands past it. Neither proof covers a
/// zero difference, direction component or radius, which have not been run.
///
/// Every step of that argument is a *relative* one, so as with every bound of
/// this shape it holds only while nothing overflows or falls into the
/// subnormals. [`HowFarFromTheSkin::on_floats`] enforces the bottom end product
/// by product through [`product_is_normal`], and `decide` the top end by
/// refusing a bound that is not itself a normal number. The permanent's reach
/// squares are not checked one by one, and where a component cancels its reach
/// square can fall below the normals or round to zero; the bound on the sum
/// holds there all the same (§464.15).
const SIDE_OF_CYLINDER_ERROR_BOUND: f64 = {
    let epsilon = f64::EPSILON / 2.0;
    (10.0 + 80.0 * epsilon) * epsilon
};

/// Two two-entry steps, each scaled by an `f64` into four, then subtracted.
const LONGEST_CROSS_COMPONENT: usize = 8;

/// The square of one of those.
const LONGEST_SQUARED_COMPONENT: usize = 2 * LONGEST_CROSS_COMPONENT * LONGEST_CROSS_COMPONENT;

/// Three two-entry squares of the stored direction components, summed: two
/// entries, then four, then six.
const LONGEST_DIRECTION_SQUARED: usize = 6;

/// `radius²·|direction|²`: the squared length scaled by each of `r²`'s two
/// entries, which doubles its length each time, and the two results summed.
const LONGEST_RADIUS_TERM: usize = 4 * LONGEST_DIRECTION_SQUARED;

/// Three squared cross components and the radius term, exactly.
const LONGEST_CYLINDER_TOTAL: usize = 3 * LONGEST_SQUARED_COMPONENT + LONGEST_RADIUS_TERM;

impl TryFrom<(Axis, f64)> for Cylinder {
    type Error = Error;

    fn try_from((axis, radius): (Axis, f64)) -> Result<Self> {
        Self::new(axis, radius)
    }
}

impl From<Cylinder> for (Axis, f64) {
    fn from(cylinder: Cylinder) -> Self {
        (cylinder.axis, cylinder.radius)
    }
}

#[cfg(test)]
mod tests {
    use rcad_predicates::{Level, decide, meet_of_three_planes};

    use super::*;
    use crate::{Point, Vector};

    fn unit_pipe_about_z() -> Cylinder {
        Cylinder::new(Axis::z(), 1.0).unwrap()
    }

    #[test]
    fn a_pipe_with_no_radius_is_not_a_pipe() {
        assert_eq!(
            Cylinder::new(Axis::z(), 0.0),
            Err(Error::RadiusNotPositive { radius: 0.0 })
        );
    }

    #[test]
    fn moving_a_pipe_moves_its_axis_and_leaves_its_radius_alone() {
        let pipe = unit_pipe_about_z();
        let shift = Isometry::translation(Vector::new(4.0, 0.0, 0.0)).unwrap();
        let moved = pipe.transformed(&shift);
        assert_eq!(moved.axis().origin(), Point::new(4.0, 0.0, 0.0));
        assert_eq!(moved.axis().direction(), Vector::new(0.0, 0.0, 1.0));
        assert_eq!(moved.radius(), 1.0);
    }

    #[test]
    fn a_point_well_outside_is_positive_and_one_well_inside_is_negative() {
        let pipe = unit_pipe_about_z();
        assert_eq!(
            pipe.side_of(PredicatePoint::from([3.0, 0.0, 100.0])),
            Decision::Pos
        );
        assert_eq!(
            pipe.side_of(PredicatePoint::from([0.25, 0.25, -100.0])),
            Decision::Neg
        );
    }

    #[test]
    fn a_point_clear_of_the_skin_is_settled_by_the_cheapest_level() {
        let (decision, level) = decide(&HowFarFromTheSkin {
            cylinder: unit_pipe_about_z(),
            point: PredicatePoint::from([3.0, 0.0, 100.0]),
        });
        assert_eq!(decision, Decision::Pos);
        assert_eq!(level, Level::Float);
    }

    #[test]
    fn sliding_along_the_axis_does_not_change_which_side_a_point_is_on() {
        let pipe = unit_pipe_about_z();
        for height in [-1e6, 0.0, 1e6] {
            assert_eq!(
                pipe.side_of(PredicatePoint::from([1.0, 0.0, height])),
                Decision::Zero
            );
        }
    }

    /// Legs and hypotenuse of a Pythagorean triple big enough that the squares
    /// run past 53 bits, so only exact arithmetic can see the point is on the
    /// skin.
    #[test]
    fn a_point_on_a_pipe_too_wide_to_see_in_f64_still_comes_out_exactly_on_it() {
        let pipe = Cylinder::new(Axis::z(), 198_489_546.0).unwrap();
        let on_the_skin = PredicatePoint::from([106_308_504.0, 167_620_410.0, 12_345.0]);

        let (decision, level) = decide(&HowFarFromTheSkin {
            cylinder: pipe,
            point: on_the_skin,
        });
        assert_eq!(decision, Decision::Zero);
        assert_eq!(level, Level::Exact);
    }

    /// The three planes meet at (1/3, 1/3, 1/3), which is not an `f64`, and the
    /// pipe through it has radius sqrt(2)/3, which is not one either.
    #[test]
    fn a_constructed_point_sitting_on_the_skin_cannot_be_placed() {
        let meet = meet_of_three_planes(
            [1.0, 1.0, 1.0, -1.0],
            [1.0, -1.0, 0.0, 0.0],
            [0.0, 1.0, -1.0, 0.0],
        )
        .unwrap();
        let pipe = Cylinder::new(Axis::z(), (2.0f64 / 9.0).sqrt()).unwrap();

        match pipe.side_of(meet) {
            Decision::Neg | Decision::Zero | Decision::Pos => {
                panic!("neither the box nor the radius is exact enough to place this")
            }
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
            other => panic!("unhandled decision: {other:?}"),
        }
    }

    /// A point just outside a very thin pipe, and level 1 used to place it
    /// inside.
    ///
    /// With `g = 1.25 * 2^-538` and a radius of `3 * 2^-539 = 1.5 * 2^-538`,
    /// the point `(g, g, 0)` is `sqrt(2) * g = 1.7678 * 2^-538` from the z
    /// axis against a radius of `1.5 * 2^-538`, so it is outside. Exactly:
    ///
    /// ```text
    /// 2*g² - (1.5 * 2^-538)² = (3.125 - 2.25) * 2^-1076 = 7/32 of a u,
    /// ```
    ///
    /// where `u = 2^-1074` is the smallest number `f64` has. The truthful
    /// answer is [`Decision::Pos`].
    ///
    /// Both squared cross-product components and the squared radius are
    /// fractions of one `u`, so `f64` rounds each to a whole number of them:
    /// the two squares `0.390625u` each go to `0`, and the squared radius
    /// `0.5625u` goes to `1u`. That leaves `0 - 1u`, negative where the truth
    /// is positive, and the bound `1.1e-15 * 1u` rounds to zero, so it
    /// certifies.
    #[test]
    fn a_point_whose_squares_fall_off_the_bottom_of_the_range_is_not_certified() {
        let across = 1.25 * 2f64.powi(-538);
        let pipe = Cylinder::new(Axis::z(), 3.0 * 2f64.powi(-539)).unwrap();

        let decision = pipe.side_of(PredicatePoint::from([across, across, 0.0]));
        match decision {
            Decision::Neg => panic!("this point is outside the pipe, not inside it"),
            Decision::Pos | Decision::Zero => panic!("nothing here can prove a sign: {decision:?}"),
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
            other => panic!("unhandled decision: {other:?}"),
        }
    }

    /// The same pipe and point scaled up by `2^600`, as an oracle for the test
    /// above.
    ///
    /// The axis runs through the origin, so scaling the radius and the point
    /// together by a positive number multiplies the whole value by the square
    /// of it — positive, so the sign is untouched. At `2^600` the squares are
    /// ordinary normal numbers and the answer is the [`Decision::Pos`] the
    /// test above cannot prove.
    #[test]
    fn the_same_pipe_and_point_scaled_up_to_fit_put_the_point_outside() {
        let across = 1.25 * 2f64.powi(-538 + 600);
        let pipe = Cylinder::new(Axis::z(), 3.0 * 2f64.powi(-539 + 600)).unwrap();
        assert_eq!(
            pipe.side_of(PredicatePoint::from([across, across, 0.0])),
            Decision::Pos
        );
    }

    #[test]
    fn a_constructed_point_well_inside_is_still_decided() {
        let meet = meet_of_three_planes(
            [1.0, 1.0, 1.0, -1.0],
            [1.0, -1.0, 0.0, 0.0],
            [0.0, 1.0, -1.0, 0.0],
        )
        .unwrap();

        match unit_pipe_about_z().side_of(meet) {
            Decision::Neg => (),
            other => panic!("a box around (1/3, 1/3, 1/3) is inside the unit pipe: {other:?}"),
        }
    }

    #[test]
    fn a_pipe_about_a_slanted_axis_still_places_points_on_its_skin() {
        let slanted = Axis::new(Point::origin(), Vector::new(1.0, 1.0, 0.0)).unwrap();
        let pipe = Cylinder::new(slanted, 1.0).unwrap();

        assert_eq!(
            pipe.side_of(PredicatePoint::from([0.0, 0.0, 5.0])),
            Decision::Pos,
            "the axis lies in the xy plane, so five up is five away from it"
        );
        assert_eq!(
            pipe.side_of(PredicatePoint::from([1.0, 1.0, 0.0])),
            Decision::Neg,
            "this point is on the axis itself"
        );
    }

    /// `1/√2` is not an `f64`, so a 45° axis stores a direction whose squared
    /// length is `1 - 1.6e-16` and not one.
    ///
    /// Before the comparison was made homogeneous in the direction
    /// (`docs/decisions.md` §118.10) the surface certified here was the pipe of
    /// radius `radius / length(stored direction)`, and a point exactly one
    /// radius out from this axis came back [`Decision::Neg`] — an exact answer
    /// about a pipe nobody asked for.
    #[test]
    fn a_pipe_about_a_forty_five_degree_axis_is_exactly_its_own_radius_wide() {
        let slanted = Axis::new(Point::origin(), Vector::new(1.0, 1.0, 0.0)).unwrap();
        let stored = slanted.direction().components();
        assert_ne!(
            stored[0] * stored[0] + stored[1] * stored[1],
            1.0,
            "the normalize rounded, and this difference from one is exactly what the old \
             comparison against a bare radius² certified as Neg"
        );

        let pipe = Cylinder::new(slanted, 1.0).unwrap();
        assert_eq!(
            pipe.side_of(PredicatePoint::from([0.0, 0.0, 1.0])),
            Decision::Zero,
            "straight up from the axis, one radius out"
        );
    }

    /// A point that is not a point is neither in the pipe nor out of it.
    ///
    /// Before the guard, a pipe about the `z` axis answered [`Decision::Pos`]
    /// for a point whose height was not a number: the height only enters the
    /// cross product multiplied by a direction component, and this direction
    /// has two of those at zero, so the unknown was multiplied away.
    #[test]
    fn a_point_that_is_not_a_point_is_neither_in_nor_out() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for axis in 0..3 {
                let mut broken = [5.0, 0.0, 0.0];
                broken[axis] = bad;
                assert_eq!(
                    unit_pipe_about_z().side_of(PredicatePoint::from(broken)),
                    Decision::Undecidable {
                        enclosure: f64::INFINITY
                    },
                    "{bad} on axis {axis}"
                );
            }
        }
    }

    /// [`Cylinder::transformed`] cannot fail and can overflow, so a pipe built
    /// from finite numbers can be holding an infinite axis origin. Before the
    /// guard such a pipe answered [`Decision::Pos`] about a perfectly ordinary
    /// point, which is a confident answer about a surface that is not a
    /// surface.
    #[test]
    fn a_pipe_shoved_past_the_end_of_the_number_line_has_no_inside() {
        let far = 1e308;
        let shove = Isometry::translation(Vector::new(far, 0.0, 0.0)).unwrap();
        let axis = Axis::new(Point::new(far, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0)).unwrap();
        let shoved = Cylinder::new(axis, 1.0).unwrap().transformed(&shove);

        assert!(
            !shoved.axis().origin().is_finite(),
            "the axis origin overflowed"
        );
        assert_eq!(
            shoved.side_of(PredicatePoint::from([0.0, 5.0, 0.0])),
            Decision::Undecidable {
                enclosure: f64::INFINITY
            }
        );
    }

    /// A plane square to the axis, one leaned off it, and one holding it.
    #[test]
    fn a_plane_closes_round_a_pipe_unless_it_holds_the_axis_direction() {
        let pipe = unit_pipe_about_z();
        for normal in [Vector::new(0.0, 0.0, 1.0), Vector::new(0.0, 1.0, 1.0)] {
            let flat = Plane::from_point_and_normal(Point::origin(), normal).unwrap();
            assert_eq!(pipe.is_cut_in_a_closed_curve_by(&flat), Some(true));
        }
        let along_it = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
            .expect("a plane holding the z axis");
        assert_eq!(pipe.is_cut_in_a_closed_curve_by(&along_it), Some(false));
    }

    /// A pair whose `f64` dot product is exactly nothing and whose real one is
    /// not: the crossing closes, and a rounded spelling of this would say it
    /// does not.
    ///
    /// The two directions are `arrange.rs`'s
    /// `a_plane_a_rounded_dot_calls_flush_with_an_axis_is_not`, where the same
    /// sign decides whether a closed ellipse gets ordered along a straight line.
    /// Here it decides whether the bulge of that ellipse is counted, and reading
    /// the rounded dot would cost a refusal rather than a wrong answer — but a
    /// refusal on a pair nothing has run out of arithmetic on.
    #[test]
    fn a_plane_a_rounded_dot_calls_flush_with_a_pipes_axis_still_closes_round_it() {
        let normal = Vector::new(
            0.082_988_092_356_092_5,
            -0.977_481_762_337_168_9,
            0.194_016_444_729_097_85,
        );
        let along = Vector::new(
            0.292_520_127_841_236_6,
            -0.162_218_223_338_325_75,
            -0.942_399_714_996_085_8,
        );
        let flat = Plane::from_point_and_normal(Point::origin(), normal).unwrap();
        let pipe = Cylinder::new(Axis::new(Point::origin(), along).unwrap(), 1.0).unwrap();
        assert_eq!(flat.normal(), normal, "the plane stores what it was given");
        assert_eq!(pipe.axis().direction(), along, "and so does the axis");
        assert_eq!(
            flat.normal().dot(pipe.axis().direction()),
            0.0,
            "the `f64` dot is exactly nothing, which is what a rounded test would read"
        );

        assert_eq!(pipe.is_cut_in_a_closed_curve_by(&flat), Some(true));
    }
}
