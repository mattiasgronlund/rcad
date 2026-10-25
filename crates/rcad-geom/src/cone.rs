//! Cone surfaces.

use rcad_predicates::expansion::{
    expansion_sign, expansion_sum, product_is_exact, scale_expansion, two_product,
};
use rcad_predicates::{
    Decision, Filtered, Interval, NOTHING_IS_KNOWN, Point as PredicatePoint, Predicate,
    SideOfSurface, are_real_numbers, decide, is_a_real_point, product_is_normal,
};
use serde::{Deserialize, Serialize};

use crate::exact::{difference, square_of_expansion};
use crate::intersect::{
    how_a_plane_leans_against_a_cone, where_a_cones_apex_stands_against_a_plane,
};
use crate::isometry::Isometry;
use crate::plane::Plane;
use crate::point::{Axis, Point, Vector};
use crate::sphere::{largest_magnitude, squared, without_overflow};
use crate::{Error, Result};

/// The name the cone's sign predicate goes by in provenance records.
pub const SIDE_OF_CONE: &str = "side of cone";

/// A double cone's skin: every point whose distance from the axis is `slope`
/// times how far along the axis it sits.
///
/// # Two nappes and not one, said out loud
///
/// `slope` is `tan` of the half-angle, and the equation
/// `|(p − apex) × w|² = slope²·((p − apex)·w)²` is even in `(p − apex)·w`. So
/// this is the **whole double cone**, both halves, meeting at the apex — which
/// is what a quadric is and what keeps the sign predicate a polynomial. A STEP
/// `conical_surface` is the same set: its own parametrization lets the radius go
/// negative, which walks onto the other nappe.
///
/// A *face* lives on one nappe, and which one is said by where its boundary is,
/// exactly as which half of a pipe a face is on is said by its boundary and not
/// by the pipe.
///
/// # Why the tangent is stored and not the angle
///
/// A predicate has to be exact about the numbers a surface holds, and `tan` is
/// not an exact function — Rust does not say how accurate it is, so two builds
/// could disagree about which side of a cone a point is on. Storing the tangent
/// makes every question below a polynomial in stored `f64`, decided exactly by
/// level 3 of the cascade. [`Cone::from_half_angle`] does the one rounding, at
/// the door, where a caller can see it.
///
/// # What is certified
///
/// **The slope exactly, and the axis to a rounding.** The expression here
/// divides through by `|w|²` cleanly: `|x × w|²/|w|² − slope²·((x·w)/|w|)²` is
/// `|x_⊥|² − slope²x_∥²` whatever `|w|` is. So the surface the predicate is
/// certified about is the cone of **exactly** this slope about the line the
/// stored [`Axis`] names. What rounds is where that line is, which is
/// [`Axis::new`]'s rounding and not this type's.
///
/// This used to be the sentence that set a cone apart from a
/// [`Cylinder`](crate::Cylinder), whose certified radius was once
/// `radius / length(stored direction)`. §118.11 put `‖w‖²` on both sides of the
/// pipe's own comparison, so a pipe reads the same way now and there is no
/// surface left here whose stored size means something other than the number
/// the caller wrote.
///
/// Serializes as `(axis, slope)` and is re-checked on the way in.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "(Axis, f64)", into = "(Axis, f64)")]
pub struct Cone {
    axis: Axis,
    slope: f64,
}

impl Cone {
    /// The cone whose apex is `axis.origin()` and which widens by `slope` for
    /// every unit along `axis.direction()`.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `axis` is not made of finite numbers, and
    /// [`Error::SlopeNotPositive`] if the slope is zero, negative, or not a
    /// finite number. A slope of zero is a line and a negative one is the same
    /// cone written backwards; both are refused so that one cone has one
    /// spelling, which is what [`Surface::how_it_sorts_against`] rests on.
    ///
    /// [`Surface::how_it_sorts_against`]: crate::Surface
    pub fn new(axis: Axis, slope: f64) -> Result<Self> {
        if !axis.origin().is_finite() || !axis.direction().is_finite() {
            return Err(Error::NotFinite { what: "cone axis" });
        }
        if !(slope.is_finite() && slope > 0.0) {
            return Err(Error::SlopeNotPositive { slope });
        }
        Ok(Self { axis, slope })
    }

    /// The same cone named by its half-angle instead of its slope.
    ///
    /// **This is where the one rounding is**, and it is here rather than inside
    /// the predicate on purpose: a STEP file names a `semi_angle`, and turning
    /// that into a slope is a conversion at the door with a caller watching,
    /// not an arithmetic step buried under a sign question.
    ///
    /// # Errors
    ///
    /// [`Error::HalfAngleIsNotACone`] if `half_angle` is not strictly between
    /// zero and a right angle. At zero the cone is a line and at a right angle
    /// it is a plane, and neither is a cone; outside that range the tangent
    /// names some other cone.
    ///
    /// [`Error::NotFinite`] and [`Error::SlopeNotPositive`] as
    /// [`Cone::new`] gives them — the second is reachable when the tangent of a
    /// half-angle a hair under a right angle overflows.
    pub fn from_half_angle(axis: Axis, half_angle: f64) -> Result<Self> {
        if !(half_angle.is_finite()
            && half_angle > 0.0
            && half_angle < core::f64::consts::FRAC_PI_2)
        {
            return Err(Error::HalfAngleIsNotACone { half_angle });
        }
        Self::new(axis, half_angle.tan())
    }

    /// The line from the apex up the middle of the cone. Its origin is the apex.
    #[must_use]
    pub fn axis(&self) -> Axis {
        self.axis
    }

    /// The one point of the cone where it has no surface normal.
    #[must_use]
    pub fn apex(&self) -> Point {
        self.axis.origin()
    }

    /// How far out the cone spreads for every unit along its axis: the tangent
    /// of its half-angle. Always greater than zero.
    #[must_use]
    pub fn slope(&self) -> f64 {
        self.slope
    }

    /// How far out the cone reaches `along` units up its axis from the apex.
    ///
    /// One multiplication, so it rounds once. Negative `along` is on the other
    /// nappe and the answer is the radius there, which is a distance and so is
    /// taken positive.
    #[must_use]
    pub fn radius_at(&self, along: f64) -> f64 {
        (self.slope * along).abs()
    }

    /// The same cone after `movement` is applied. A rigid movement cannot change
    /// the half-angle, so the slope goes over untouched.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        Self {
            axis: self.axis.transformed(movement),
            slope: self.slope,
        }
    }

    /// Two directions across the cone, at right angles to each other and to the
    /// axis, with the first crossed into the second giving the axis back.
    ///
    /// The same choice a [`Cylinder`](crate::Cylinder) makes and for the same
    /// reason: `u = 0` lands on `+x` for a cone about `z`, which is where a
    /// frustum puts its seam, so that face's `(u, v)` rectangle has the seam
    /// down its edge rather than across the middle of it.
    ///
    /// # Panics
    ///
    /// If the stored axis direction has no length. No public constructor can
    /// hand back one — [`Axis::new`] checks it, and [`Axis::as_written`], which
    /// `Deserialize` goes through, hands it any direction whose length is not
    /// within a hair of one — and [`Cone::transformed`] cannot shrink one,
    /// for the reason [`Cylinder::basis`](crate::Cylinder::basis) spells out at
    /// length.
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
            .normalized_as("direction across a cone")
            .expect("a unit axis is never parallel to its least aligned coordinate axis");
        (u, along.cross(u))
    }

    /// The point at angle `round` radians from [`Cone::basis`]'s first
    /// direction, `along` up the axis from the apex.
    ///
    /// `along = 0` is the apex whatever `round` is, which is the cone's version
    /// of a ball's pole: the second coordinate pinches the first away to
    /// nothing. Negative `along` walks onto the other nappe.
    ///
    /// **On either nappe `round` names the same direction across the axis**,
    /// because the step across is [`Cone::radius_at`], which is never negative.
    /// That is what makes this undo [`Cone::coordinates_of`], which reads the
    /// angle off the direction across whichever nappe the point is on. Scaling
    /// by `slope · along` instead put every point with a negative `along` half a
    /// turn round from where it was read, and a frustum whose wall names its
    /// cone about the axis pointing away from it meshed folded
    /// (`docs/decisions.md` §357.3). Reading the angle the other way round on
    /// that nappe instead would move the seam there to `round = π`, and §25.2
    /// fixes it at zero.
    ///
    /// Only exactly on the cone where the sine and cosine are exact, which in
    /// practice means `round = 0`. No corner of a face is ever made this way,
    /// for the reason [`Cylinder::point_at`](crate::Cylinder::point_at) gives.
    #[must_use]
    pub fn point_at(&self, round: f64, along: f64) -> Point {
        let (u, v) = self.basis();
        self.axis.origin()
            + (u * round.cos() + v * round.sin()) * self.radius_at(along)
            + self.axis.direction() * along
    }

    /// Which angle round and how far along `point` sits at —
    /// [`Cone::point_at`] the other way round.
    ///
    /// **The angle is read off the direction across the axis and the distance
    /// along it off the axis, exactly as a pipe does**, so a point off the skin
    /// gets the coordinates of the point on the same ruling rather than of the
    /// nearest point on the cone. That is deliberate and it is what
    /// `point_at(coordinates_of(p))` needs to come back to `p` for a `p` that is
    /// on the cone: the along-coordinate has to be the one the ruling is
    /// parametrised by.
    ///
    /// The angle comes back on `atan2`'s branch, `(−π, π]`.
    ///
    /// **The apex has no angle at all** and this hands back zero for it, which
    /// is what `atan2(0, 0)` gives without a branch — the same answer a pipe
    /// gives for a point on its axis, and honest for the same reason: the
    /// geometry does not determine it.
    #[must_use]
    pub fn coordinates_of(&self, point: Point) -> (f64, f64) {
        let (across, and) = self.basis();
        let from_apex = point - self.axis.origin();
        (
            from_apex.dot(and).atan2(from_apex.dot(across)),
            from_apex.dot(self.axis.direction()),
        )
    }

    /// Whether `flat` cuts this cone in a curve that closes — a circle or an
    /// ellipse, on one nappe and bounded — with `None` when the arithmetic
    /// could not settle it.
    ///
    /// This is the one place the question is asked. Three callers in
    /// `rcad-topo` used to spell it three ways, two of them by comparing the
    /// stored normal against the stored axis direction and so answering only
    /// about the circle; `docs/decisions.md` §23.8 is why that is how two
    /// spellings come to disagree.
    ///
    /// # The test, with nothing normalised
    ///
    /// With `n` the plane's normal, `a` the axis direction and `s` the slope,
    /// the curve closes exactly when the angle between `n` and `a` is small
    /// enough that `cos²θ > sin²α` for `tan α = s`. Cleared of divisions that
    /// is
    ///
    /// ```text
    /// (n·a)²·(1 + s²)  −  s²·|n|²·|a|²  >  0
    /// ```
    ///
    /// which is [`how_a_plane_leans_against_a_cone`]'s
    /// `E = (n·a)² − s²‖n × a‖²` written another way, because
    /// `‖n × a‖² = |n|²|a|² − (n·a)²`. Every term is a polynomial in the stored
    /// `f64`, so no vector is normalised and nothing is rounded on the way.
    ///
    /// **The comparison is strict at both ends.** Equality is a parabola and
    /// below it is a hyperbola's two branches; neither closes, and both come
    /// back `Some(false)`.
    ///
    /// A plane **square to the axis** is the tidiest member of the closed
    /// family rather than a case of its own: `n × a` is nothing there, so
    /// `E = (n·a)² = (a·a)² > 0` for any direction a cone can hold.
    ///
    /// # The apex has to be off the plane
    ///
    /// With the apex on it there is no closed curve to be had at all — the two
    /// share a ruling, a pair of rulings, or the apex alone — so that is
    /// `Some(false)` before the lean is asked at all.
    ///
    /// [`how_a_plane_leans_against_a_cone`]: crate::how_a_plane_leans_against_a_cone
    #[must_use]
    pub fn is_cut_in_a_closed_curve_by(&self, flat: &Plane) -> Option<bool> {
        match where_a_cones_apex_stands_against_a_plane(flat, self) {
            Decision::Pos | Decision::Neg => {}
            Decision::Zero => return Some(false),
            Decision::Undecidable { .. } | _ => return None,
        }
        match how_a_plane_leans_against_a_cone(flat, self) {
            Decision::Pos => Some(true),
            Decision::Zero | Decision::Neg => Some(false),
            Decision::Undecidable { .. } | _ => None,
        }
    }

    /// Which way the centre of the closed curve `flat` cuts this cone in lies
    /// from the apex, as a direction that is neither normalised nor rounded;
    /// `None` when the crossing does not close.
    ///
    /// The centre itself is
    ///
    /// ```text
    /// apex + ((e − n·apex) / (n·along)) · along
    /// ```
    ///
    /// with `n` and `e` the plane's normal and offset, and `n·along` is
    /// **positive** whenever this answers at all. A direction and a point come
    /// back rather than the centre because taking that division would round it,
    /// and a caller that wants a sign of the centre against something can clear
    /// the division instead — multiply through by `n·along` and every term is a
    /// polynomial in numbers that are stored exactly.
    ///
    /// A **ball**'s answer to the same question is its own centre with
    /// `along = n`, so one formula serves both.
    ///
    /// # Where the direction comes from
    ///
    /// Write `v = x − apex`. A point is on this cone exactly when
    /// `(1 + s²)(v·w)² − |w|²|v|² = 0` for `w` the stored axis direction and
    /// `s` the slope, so the quadric's matrix is `A = (1 + s²)·w wᵀ − |w|²·I`.
    ///
    /// A conic's centre is the **pole** of the plane that cuts it: the point
    /// `c` on the plane with `A(c − apex)` running along `n`. There the
    /// quadratic form has no linear part in the plane's own two directions, so
    /// the curve is symmetric about `c`, which is what being the centre means.
    ///
    /// Expanding `A` against
    ///
    /// ```text
    /// along = (1 + s²)(n·w)·w  −  s²|w|²·n
    /// ```
    ///
    /// gives `w·along = (n·w)|w|²` and then `A·along = s²|w|⁴·n`, so `along`
    /// runs the way `A⁻¹n` does and walking it from the apex to the plane lands
    /// on the pole.
    ///
    /// # Why `n·along` positive is exactly the closed family
    ///
    /// ```text
    /// n·along = (1 + s²)(n·w)² − s²|n|²|w|²
    /// ```
    ///
    /// which is [`Cone::is_cut_in_a_closed_curve_by`]'s test written out — so
    /// the circle and the ellipse have it positive, the parabola has it zero
    /// and has no centre at all, and the hyperbola has it negative. The
    /// hyperbola does have a centre, and this still refuses it: its two
    /// branches are not one closed curve for a chord to split.
    ///
    /// A plane **square to the axis** falls out as `along = |w|²·n`, so the
    /// centre is the foot of the perpendicular from the apex — the circle, and
    /// the case this is a widening of.
    #[must_use]
    pub fn which_way_a_closed_crossings_centre_lies(&self, flat: &Plane) -> Option<[Interval; 3]> {
        if self.is_cut_in_a_closed_curve_by(flat) != Some(true) {
            return None;
        }
        let normal = flat.normal().components().map(Interval::exactly);
        let axis = self.axis.direction().components().map(Interval::exactly);
        let dot = |u: [Interval; 3], v: [Interval; 3]| u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
        let squared_slope = Interval::exactly(self.slope) * Interval::exactly(self.slope);
        let leaning = (Interval::exactly(1.0) + squared_slope) * dot(normal, axis);
        let spread = squared_slope * dot(axis, axis);
        Some(core::array::from_fn(|at| {
            leaning * axis[at] - spread * normal[at]
        }))
    }
}

/// Inside, on, or outside, decided through the cascade.
///
/// The sign of `|(point − apex) × w|² − slope²·((point − apex)·w)²`:
/// [`Decision::Neg`] inside the cone, [`Decision::Pos`] outside,
/// [`Decision::Zero`] exactly on the skin.
///
/// **Inside means inside either nappe**, because the expression is even in
/// `(point − apex)·w` and a quadric cone has two halves. See [`Cone`].
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
/// and [`Cone::transformed`] cannot fail and can overflow.
impl SideOfSurface for Cone {
    fn side_of(&self, point: PredicatePoint) -> Decision {
        let cone_is_real = are_real_numbers(self.axis.origin().coordinates())
            && are_real_numbers(self.axis.direction().components())
            && are_real_numbers([self.slope]);
        if !(is_a_real_point(point) && cone_is_real) {
            return NOTHING_IS_KNOWN;
        }
        decide(&HowFarFromTheSkin { cone: *self, point }).0
    }
}

/// `|(point − apex) × w|² − slope²·((point − apex)·w)²`, evaluated three ways.
struct HowFarFromTheSkin {
    cone: Cone,
    point: PredicatePoint,
}

impl Predicate for HowFarFromTheSkin {
    fn name(&self) -> &'static str {
        SIDE_OF_CONE
    }

    fn on_floats(&self) -> Option<Filtered> {
        let point = self.point.exact_coordinates()?;
        let apex = self.cone.axis.origin().coordinates();
        let direction = self.cone.axis.direction().components();

        let from_apex = [0, 1, 2].map(|axis| point[axis] - apex[axis]);
        let slope = self.cone.slope;

        // The bound below is a fraction of the permanent, which only bounds
        // anything while every product keeps its relative accuracy. One of them
        // in the subnormals and this level certifies a wrong sign.
        let halves = cross_product_halves_if_normal(from_apex, direction)?;
        let components = halves.map(|(left, right)| left - right);
        let up_halves: [f64; 3] = core::array::from_fn(|axis| from_apex[axis] * direction[axis]);
        let up = up_halves[0] + up_halves[1] + up_halves[2];
        let stays_accurate = components
            .iter()
            .all(|&component| product_is_normal(component, component))
            && (0..3).all(|axis| product_is_normal(from_apex[axis], direction[axis]))
            && product_is_normal(slope, slope)
            && product_is_normal(up, up)
            && product_is_normal(slope * slope, up * up);
        if !stays_accurate {
            return None;
        }

        let squares = components.map(|component| component * component);
        let along = slope * slope * (up * up);

        // Each cross-product component is a difference, so it can cancel and
        // its own magnitude says nothing about the rounding it carries. The
        // permanent has to use the sum of the two halves instead, squared
        // because the component is squared. The same goes for the dot product
        // down the axis.
        let spread = halves.map(|(left, right)| {
            let reach = left.abs() + right.abs();
            reach * reach
        });
        let up_reach = up_halves[0].abs() + up_halves[1].abs() + up_halves[2].abs();
        let permanent = spread[0] + spread[1] + spread[2] + slope * slope * (up_reach * up_reach);

        Some(Filtered {
            value: squares[0] + squares[1] + squares[2] - along,
            error_bound: SIDE_OF_CONE_ERROR_BOUND * permanent,
        })
    }

    fn on_intervals(&self) -> Interval {
        let point = self.point.enclosure();
        let apex = self.cone.axis.origin().coordinates();
        let direction = self.cone.axis.direction().components();

        let largest = largest_magnitude(&point, &[apex]);
        let along = direction.map(Interval::exactly);
        let slope_squared = squared(Interval::exactly(self.cone.slope));

        without_overflow(largest, |scale| {
            let from_apex =
                [0, 1, 2].map(|axis| point[axis] * scale - Interval::exactly(apex[axis]) * scale);
            let halves = cross_product_halves(from_apex, along);
            let squared_component = |axis: usize| {
                let (left, right) = halves[axis];
                squared(left - right)
            };
            let up = (0..3).fold(Interval::exactly(0.0), |so_far, axis| {
                so_far + from_apex[axis] * along[axis]
            });
            squared_component(0) + squared_component(1) + squared_component(2)
                - slope_squared * squared(up)
        })
    }

    fn exactly(&self) -> Option<Decision> {
        let point = self.point.exact_coordinates()?;
        let apex = self.cone.axis.origin().coordinates();
        let direction = self.cone.axis.direction().components();
        let from_apex = [0, 1, 2].map(|axis| difference(point[axis], apex[axis]));

        let mut total = [0.0f64; LONGEST_CONE_TOTAL];
        let mut total_len = 0;
        let mut merged = [0.0f64; LONGEST_CONE_TOTAL];
        let mut component = [0.0f64; LONGEST_CROSS_COMPONENT];
        let mut squared = [0.0f64; LONGEST_SQUARED_COMPONENT];

        for axis in 0..3 {
            let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
            let component_len = cross_component(
                from_apex[next],
                direction[after],
                from_apex[after],
                direction[next],
                &mut component,
            )?;
            let squared_len = square_of_expansion(&component[..component_len], &mut squared)?;
            total_len = expansion_sum(&total[..total_len], &squared[..squared_len], &mut merged);
            core::mem::swap(&mut total, &mut merged);
        }

        // `slope²·(x·w)²`, exactly: the dot product first, then its square,
        // then twice by the slope. Every step declines rather than rounds.
        let mut up = [0.0f64; LONGEST_DOT];
        let mut up_len = 0;
        let mut up_merged = [0.0f64; LONGEST_DOT];
        for axis in 0..3 {
            if !from_apex[axis]
                .iter()
                .all(|&entry| product_is_exact(entry, direction[axis]))
            {
                return None;
            }
            let mut term = [0.0f64; 4];
            let term_len = scale_expansion(&from_apex[axis], direction[axis], &mut term);
            up_len = expansion_sum(&up[..up_len], &term[..term_len], &mut up_merged);
            core::mem::swap(&mut up, &mut up_merged);
        }

        let mut up_squared = [0.0f64; LONGEST_SQUARED_DOT];
        let up_squared_len = square_of_expansion(&up[..up_len], &mut up_squared)?;
        let slope = self.cone.slope;
        if !product_is_exact(slope, slope) {
            return None;
        }
        let (slope_squared, slope_roundoff) = two_product(slope, slope);
        let mut by_slope = [0.0f64; LONGEST_CONE_TOTAL];
        let mut by_roundoff = [0.0f64; LONGEST_CONE_TOTAL];
        if !up_squared[..up_squared_len].iter().all(|&entry| {
            product_is_exact(entry, slope_squared) && product_is_exact(entry, slope_roundoff)
        }) {
            return None;
        }
        let by_slope_len =
            scale_expansion(&up_squared[..up_squared_len], -slope_squared, &mut by_slope);
        let by_roundoff_len = scale_expansion(
            &up_squared[..up_squared_len],
            -slope_roundoff,
            &mut by_roundoff,
        );
        let mut axial = [0.0f64; LONGEST_CONE_TOTAL];
        let axial_len = expansion_sum(
            &by_slope[..by_slope_len],
            &by_roundoff[..by_roundoff_len],
            &mut axial,
        );

        total_len = expansion_sum(&total[..total_len], &axial[..axial_len], &mut merged);

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
fn cross_product_halves_if_normal(from_apex: [f64; 3], along: [f64; 3]) -> Option<[(f64, f64); 3]> {
    for axis in 0..3 {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        if !product_is_normal(from_apex[next], along[after])
            || !product_is_normal(from_apex[after], along[next])
        {
            return None;
        }
    }
    Some(cross_product_halves(from_apex, along))
}

/// The two halves of each component of `from_apex × along`, left half first.
///
/// Kept apart rather than subtracted so that level 1 can size its error bound
/// against both of them; the difference on its own hides the cancellation.
fn cross_product_halves<T: Copy + core::ops::Mul<Output = T>>(
    from_apex: [T; 3],
    along: [T; 3],
) -> [(T, T); 3] {
    [
        (from_apex[1] * along[2], from_apex[2] * along[1]),
        (from_apex[2] * along[0], from_apex[0] * along[2]),
        (from_apex[0] * along[1], from_apex[1] * along[0]),
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
/// The cross-product half of this is
/// [`Cylinder`](crate::Cylinder)'s word for word and reaches `(1 + epsilon)^10`.
/// The axial half is one step longer: `point − apex` is one rounding, a product
/// with a direction component is a second, two sums to fold the three terms are
/// a third and fourth, squaring that is a fifth and doubles what it carries so
/// it sits at `(1 + epsilon)^9`, and the two multiplications by the slope are a
/// tenth and eleventh. Subtracting it from the cross-product sum is a twelfth.
/// `(1 + epsilon)^12 − 1 ≤ 12·epsilon + 66·epsilon² + …` times the permanent.
///
/// The `96·epsilon²` was meant to cover the higher-order terms and the
/// rounding of the permanent itself, and by that count it falls short. The
/// higher-order terms start at `66·epsilon²`. The permanent's own roundings can
/// put it up to about `7·epsilon` under its true value, which costs
/// `84·epsilon²` against the `12·epsilon`, and multiplying it by this constant
/// rounds once more, which costs `epsilon` of the bound and so `12·epsilon²` of
/// the permanent: `66 + 84 + 12 = 162`, above the `96`.
///
/// That count is loose, and the constant holds where `decide` needs it: on the
/// sum before `value`'s last rounding, whose longest chain is eleven roundings
/// rather than twelve. Gappa proves the bound there at this constant
/// (`docs/decisions.md` §464.16), whether or not a component of the cross
/// product or the dot product down the axis cancels, but for the one case
/// below. With nothing cancelling the least constant it proves is
/// `11·epsilon + 99·epsilon²`. Nobody has tried the claim on `value` itself.
/// The proof covers no zero difference or direction component, which have not
/// been run.
///
/// Every step of that argument is a *relative* one, so as with every bound of
/// this shape it holds only while nothing overflows or falls into the
/// subnormals. `decide` keeps out the top end by refusing a bound that is not
/// itself a normal number. [`HowFarFromTheSkin::on_floats`] checks the bottom
/// end through [`product_is_normal`] for the products of `value`, but for none
/// of the permanent's own: not the reach squares, not `up_reach` squared, and
/// not `slope²` times that. A check passes on a zero, so a component of the
/// cross product that cancels to zero lets its reach square fall below the
/// normals or round to zero, and where the dot product down the axis cancels to
/// zero, `up_reach` squared and `slope²` times it can too. The bound on the sum
/// holds in each of those but one: where `up_reach` squared rounds to zero,
/// level 1 certifies a wrong sign. A probe at a slope of `2⁵¹¹` shows one, and
/// by hand the shape needs a slope of about `2⁷⁸` or more (§464.16's 6).
const SIDE_OF_CONE_ERROR_BOUND: f64 = {
    let epsilon = f64::EPSILON / 2.0;
    (12.0 + 96.0 * epsilon) * epsilon
};

/// Two two-entry steps, each scaled by an `f64` into four, then subtracted.
const LONGEST_CROSS_COMPONENT: usize = 8;

/// The square of one of those.
const LONGEST_SQUARED_COMPONENT: usize = 2 * LONGEST_CROSS_COMPONENT * LONGEST_CROSS_COMPONENT;

/// Three two-entry steps each scaled by an `f64` into four, summed.
const LONGEST_DOT: usize = 12;

/// The square of one of those.
const LONGEST_SQUARED_DOT: usize = 2 * LONGEST_DOT * LONGEST_DOT;

/// Three squared cross components and the axial term twice over, exactly.
const LONGEST_CONE_TOTAL: usize = 3 * LONGEST_SQUARED_COMPONENT + 4 * LONGEST_SQUARED_DOT;

impl TryFrom<(Axis, f64)> for Cone {
    type Error = Error;

    fn try_from((axis, slope): (Axis, f64)) -> Result<Self> {
        Self::new(axis, slope)
    }
}

impl From<Cone> for (Axis, f64) {
    fn from(cone: Cone) -> Self {
        (cone.axis, cone.slope)
    }
}

#[cfg(test)]
mod tests {
    use rcad_predicates::{Level, decide, meet_of_three_planes};

    use super::*;
    use crate::{Point, Vector};

    fn a_right_angled_cone() -> Cone {
        Cone::new(Axis::z(), 1.0).expect("a slope of one is 45 degrees")
    }

    #[test]
    fn a_cone_with_no_spread_is_not_a_cone() {
        for slope in [0.0, -1.0, -0.0, f64::NAN, f64::INFINITY] {
            match Cone::new(Axis::z(), slope) {
                Err(Error::SlopeNotPositive { slope: refused }) => {
                    assert_eq!(refused.to_bits(), slope.to_bits());
                }
                other => panic!("a slope of {slope} does not name a cone: {other:?}"),
            }
        }
    }

    /// Zero and a right angle are a line and a plane, and neither is a cone.
    #[test]
    fn a_half_angle_at_or_past_a_right_angle_is_not_a_cone() {
        for half_angle in [
            0.0,
            -0.5,
            core::f64::consts::FRAC_PI_2,
            2.0,
            f64::NAN,
            f64::INFINITY,
        ] {
            match Cone::from_half_angle(Axis::z(), half_angle) {
                Err(Error::HalfAngleIsNotACone {
                    half_angle: refused,
                }) => {
                    assert_eq!(refused.to_bits(), half_angle.to_bits());
                }
                other => panic!("a half-angle of {half_angle} does not name a cone: {other:?}"),
            }
        }
        assert_eq!(
            Cone::from_half_angle(Axis::z(), core::f64::consts::FRAC_PI_4)
                .expect("45 degrees is a cone")
                .slope(),
            core::f64::consts::FRAC_PI_4.tan()
        );
    }

    #[test]
    fn moving_a_cone_moves_its_apex_and_leaves_its_slope_alone() {
        let cone = a_right_angled_cone();
        let shift = Isometry::translation(Vector::new(4.0, 0.0, 0.0)).expect("a finite shift");
        let moved = cone.transformed(&shift);
        assert_eq!(moved.apex(), Point::new(4.0, 0.0, 0.0));
        assert_eq!(moved.axis().direction(), Vector::new(0.0, 0.0, 1.0));
        assert_eq!(moved.slope(), 1.0);
    }

    #[test]
    fn a_point_well_outside_is_positive_and_one_well_inside_is_negative() {
        let cone = a_right_angled_cone();
        assert_eq!(
            cone.side_of(PredicatePoint::from([3.0, 0.0, 1.0])),
            Decision::Pos
        );
        assert_eq!(
            cone.side_of(PredicatePoint::from([0.25, 0.25, 100.0])),
            Decision::Neg
        );
    }

    /// The whole double cone, said as a test rather than as a sentence: the
    /// point that is inside at `z = 100` is inside at `z = −100` too.
    #[test]
    fn a_cone_has_two_halves_and_a_point_is_inside_either_of_them() {
        let cone = a_right_angled_cone();
        for height in [100.0, -100.0] {
            assert_eq!(
                cone.side_of(PredicatePoint::from([0.25, 0.25, height])),
                Decision::Neg,
                "at {height}"
            );
            assert_eq!(
                cone.side_of(PredicatePoint::from([height, 0.0, height])),
                Decision::Zero,
                "on the skin at {height}"
            );
        }
    }

    /// The apex is on the cone, and the only point of it that is.
    #[test]
    fn the_apex_is_on_the_cone() {
        let cone = Cone::new(
            Axis::new(Point::new(2.0, -3.0, 5.0), Vector::new(0.0, 0.0, 1.0))
                .expect("a direction with length"),
            0.5,
        )
        .expect("a positive slope");
        assert_eq!(
            cone.side_of(PredicatePoint::from([2.0, -3.0, 5.0])),
            Decision::Zero
        );
    }

    #[test]
    fn a_point_clear_of_the_skin_is_settled_by_the_cheapest_level() {
        let (decision, level) = decide(&HowFarFromTheSkin {
            cone: a_right_angled_cone(),
            point: PredicatePoint::from([3.0, 0.0, 1.0]),
        });
        assert_eq!(decision, Decision::Pos);
        assert_eq!(level, Level::Float);
    }

    /// Legs and hypotenuse of a Pythagorean triple big enough that the squares
    /// run past 53 bits, so only exact arithmetic can see the point is on the
    /// skin. The cone has slope 1, so the height is the radius.
    #[test]
    fn a_point_on_a_cone_too_wide_to_see_in_f64_still_comes_out_exactly_on_it() {
        let cone = a_right_angled_cone();
        let on_the_skin = PredicatePoint::from([106_308_504.0, 167_620_410.0, 198_489_546.0]);

        let (decision, level) = decide(&HowFarFromTheSkin {
            cone,
            point: on_the_skin,
        });
        assert_eq!(decision, Decision::Zero);
        assert_eq!(level, Level::Exact);
    }

    /// The three planes meet at (1/3, 1/3, 1/3), which is not an `f64`, and the
    /// cone through it has slope `√2`, which is not one either.
    #[test]
    fn a_constructed_point_sitting_on_the_skin_cannot_be_placed() {
        let meet = meet_of_three_planes(
            [1.0, 1.0, 1.0, -1.0],
            [1.0, -1.0, 0.0, 0.0],
            [0.0, 1.0, -1.0, 0.0],
        )
        .expect("three planes that meet at a point");
        let cone = Cone::new(Axis::z(), 2.0f64.sqrt()).expect("a positive slope");

        match cone.side_of(meet) {
            Decision::Neg | Decision::Zero | Decision::Pos => {
                panic!("neither the box nor the slope is exact enough to place this")
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

        // The point sits `√(2)/3 ≈ 0.471` off the `z` axis at a height of `1/3`,
        // so a cone of slope two reaches `0.667` there and swallows it.
        let cone = Cone::new(Axis::z(), 2.0).expect("a positive slope");
        match cone.side_of(meet) {
            Decision::Neg => (),
            other => {
                panic!("a box around (1/3, 1/3, 1/3) is inside a cone of slope two: {other:?}")
            }
        }
    }

    #[test]
    fn a_cone_about_a_slanted_axis_still_places_points_on_its_skin() {
        let slanted = Axis::new(Point::origin(), Vector::new(1.0, 1.0, 0.0))
            .expect("a direction with length");
        let cone = Cone::new(slanted, 1.0).expect("a positive slope");

        assert_eq!(
            cone.side_of(PredicatePoint::from([0.0, 0.0, 5.0])),
            Decision::Pos,
            "the axis lies in the xy plane, so a point five up is five off it and the cone \
             through the origin reaches nothing there"
        );
        assert_eq!(
            cone.side_of(PredicatePoint::from([5.0, 5.0, 0.0])),
            Decision::Neg,
            "this point is on the axis itself"
        );
    }

    /// A point that is not a point is neither in the cone nor out of it.
    #[test]
    fn a_point_that_is_not_a_point_is_neither_in_nor_out() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for axis in 0..3 {
                let mut broken = [5.0, 0.0, 1.0];
                broken[axis] = bad;
                assert_eq!(
                    a_right_angled_cone().side_of(PredicatePoint::from(broken)),
                    Decision::Undecidable {
                        enclosure: f64::INFINITY
                    },
                    "{bad} on axis {axis}"
                );
            }
        }
    }

    /// [`Cone::transformed`] cannot fail and can overflow, so a cone built from
    /// finite numbers can be holding an infinite apex by the time it gets here.
    #[test]
    fn a_cone_shoved_past_the_end_of_the_number_line_has_no_inside() {
        let far = 1e308;
        let shove = Isometry::translation(Vector::new(far, 0.0, 0.0)).expect("a finite shift");
        let axis = Axis::new(Point::new(far, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
            .expect("a direction with length");
        let shoved = Cone::new(axis, 1.0)
            .expect("a positive slope")
            .transformed(&shove);

        assert!(!shoved.apex().is_finite(), "the apex overflowed");
        assert_eq!(
            shoved.side_of(PredicatePoint::from([0.0, 5.0, 0.0])),
            Decision::Undecidable {
                enclosure: f64::INFINITY
            }
        );
    }

    /// The two coordinates come back to the point they were read from, for a
    /// cone about a coordinate axis where the trigonometry lands exactly.
    #[test]
    fn a_point_on_a_ruling_reads_back_to_itself() {
        let cone = Cone::new(Axis::z(), 2.0).expect("a positive slope");
        let on_it = Point::new(6.0, 0.0, 3.0);
        let (round, along) = cone.coordinates_of(on_it);
        assert_eq!((round, along), (0.0, 3.0));
        assert_eq!(cone.point_at(round, along), on_it);
    }

    /// The apex has no angle, and asking for one gives zero rather than a
    /// division by nothing.
    #[test]
    fn the_apex_has_no_angle_and_says_so_with_a_zero() {
        let cone = a_right_angled_cone();
        assert_eq!(cone.coordinates_of(Point::origin()), (0.0, 0.0));
        for round in [0.0, 1.0, -2.5] {
            assert_eq!(cone.point_at(round, 0.0), Point::origin());
        }
    }
}
