//! The de Boor recursion, written once.
//!
//! Curves and surfaces share it, and so does the interval arithmetic that
//! carries the enclosure: everything here is generic over [`Coefficient`], a
//! scalar that can be built from an exact `f64`, added, subtracted, multiplied
//! and divided. `f64` and [`Interval`] are the two that implement it, and the
//! recursion cannot tell them apart.
//!
//! Everything runs on the **homogeneous** net — `[w*x, w*y, w*z, w]` — and the
//! divide back to a point happens once, at the very end. That is what lets a
//! surface be the curve evaluator applied twice, and it is what makes the
//! scaling oracle exactly true: every step is homogeneous of degree one in the
//! control points, so scaling them by a power of two scales the answer by the
//! same factor, bit for bit.
//!
//! Nothing here allocates. One de Boor stage needs `degree + 1` homogeneous
//! points and [`MAX_DEGREE`] is 15, so the callers keep the scratch on the
//! stack.

use core::ops::{Add, Mul, Sub};

use rcad_predicates::expansion::{product_is_exact, two_product};
use rcad_predicates::{Interval, Point as PredicatePoint, product_is_normal};

use crate::knot_vector::MAX_DEGREE;
use crate::point::{Point, Vector};

/// How much of the largest control coordinate one de Boor pass can lose to
/// rounding, as a fraction of it.
///
/// The bound is **absolute in the size of the input, not relative to the
/// answer**. de Boor mixes control points with non-negative weights that sum to
/// one, but the control points themselves have signs, so `(+1) + (−1)` can come
/// out as zero with a rounding error attached. Nothing relative to the answer
/// survives that. What does survive is a bound in terms of `M`, the largest
/// magnitude among the four homogeneous coordinates of the control points the
/// evaluation actually touches: each coordinate of the homogeneous result is
/// within `EVALUATION_ROUNDOFF * M` of the true one.
///
/// # Where the number comes from
///
/// Write `u = f64::EPSILON / 2` for the unit roundoff — half an ulp of 1, and
/// so the most a single rounding costs in relative terms. One stage replaces
/// `d[j]` by `(1 − a)·d[j−1] + a·d[j]`, where
/// `a = (x − knots[left]) / (knots[right] − knots[left])` and `a` is in `[0, 1]`.
///
/// - `a` costs three roundings: the two subtractions and the divide. Its
///   operands are stored numbers, so each subtraction is exactly `(1 + δ)`
///   times the truth, and `|a_computed − a| ≤ 3u·a ≤ 3u`.
/// - `1 − a` inherits that error and adds one rounding of its own, so
///   `|b_computed − b| ≤ 4u`.
/// - The two products and the sum are three more roundings, worth `2u` on the
///   two products' magnitudes plus `u` on the sum — `(1 + u)² − 1 ≤ 2u + u²`
///   applied to a total that is at most `(a + b)` times the largest running
///   value.
///
/// The naive reading of that list is seven roundings per stage. **It is nine.**
/// The coefficient errors do not travel down one chain; both of them are
/// charged at full size against the running values, because `|δa|·|d[j]|` does
/// not shrink when `a` is small. Adding them up: `4u` for `b`, `3u` for `a`,
/// `2u` for the three arithmetic roundings, so
/// `E_r ≤ E_{r−1}·(1 + 9u) + 9u·M`, and after `p` stages
/// `E_p ≤ M·((1 + 9u)^p − 1)`.
///
/// With `p` capped at [`MAX_DEGREE`] that is 15 stages and 135 roundings.
/// Using `(1 + x)^p − 1 ≤ p·x + p²·x²` with `x = 9u`, the constant is
/// `135u + (9·15)²·u² = 135u + 18225u²`.
///
/// The binomial's own second-order term is only `C(15,2)·9² = 8505`, so 18225
/// leaves better than a factor of two spare. It is spent on two real things:
/// the second-order terms dropped from the per-stage argument above, worth
/// another 165 or so; and the fact that `f64` cannot write `135u + 8670u²`
/// down that precisely — the second term is worth about 78 ulps of the first,
/// so the sum's own rounding costs more than a percent of it. A bound that its
/// own arithmetic rounded below the truth is not a bound. This is the slack
/// `SIDE_OF_SPHERE_ERROR_BOUND`'s 48-over-15 leaves, for the same reason.
///
/// A surface is two passes, `v` then `u`, so it needs
/// [`after_two_passes`] instead.
///
/// # The precondition, which is not optional
///
/// Every step of the argument above is *relative*, so it holds only while
/// nothing overflows and `M` is a normal number. [`roundoff_of_a_divided_point`]
/// checks that and hands back `None` rather than certify a bound whose model
/// does not hold — the same shape as `product_is_normal` guarding the level-1
/// filters, and for the same reason (`docs/decisions.md` §12.2).
///
/// Subnormal *intermediates* need no separate guard, and that is worth saying
/// because it is not obvious. A product landing in the subnormals is wrong by
/// at most `2⁻¹⁰⁷⁵`, and this bound already charges `u·M` for it, which is at
/// least `2⁻¹⁰⁷⁵` as soon as `M` is normal. The subtractions need nothing
/// either: by Hauser's lemma a sum or difference that lands in the subnormals
/// is exact.
pub const EVALUATION_ROUNDOFF: f64 = {
    let unit_roundoff = f64::EPSILON / 2.0;
    let roundings = 9.0 * MAX_DEGREE as f64;
    (roundings + roundings * roundings * unit_roundoff) * unit_roundoff
};

/// A number the recursion can run on.
///
/// `f64` for the ordinary answer and [`Interval`] for the enclosure. They
/// implement the same three operations and the same two named ones, which is
/// why the recursion below never mentions either except to read a stored knot.
pub(crate) trait Coefficient:
    Copy + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self>
{
    /// The stored number itself, with nothing added to it.
    fn exactly(value: f64) -> Self;

    /// Division, which is not one of the `core::ops` this trait already has
    /// because an interval cannot divide by a range that straddles zero.
    fn divided_by(self, divisor: Self) -> Self;
}

impl Coefficient for f64 {
    fn exactly(value: f64) -> Self {
        value
    }

    fn divided_by(self, divisor: Self) -> Self {
        self / divisor
    }
}

/// The arithmetic the enclosure is made of.
///
/// de Boor suits it unusually well: every coefficient the recursion forms lies
/// in `[0, 1]`, so a stage is a weighted average of two numbers the true answer
/// already sits between, and there is no catastrophic cancellation anywhere for
/// a width to blow up on. What width there is comes from the knot differences
/// and from the perspective divide, and both are one division each.
impl Coefficient for Interval {
    fn exactly(value: f64) -> Self {
        Interval::exactly(value)
    }

    fn divided_by(self, divisor: Self) -> Self {
        quotient_enclosing(self, divisor)
    }
}

/// The point on the spline at `parameter`, in homogeneous coordinates.
///
/// `points` holds the `degree + 1` control points the span reaches, in order:
/// `points[j]` is control point `j + span − degree`. It is written over as the
/// recursion runs.
///
/// # Panics
///
/// If `points` is not `degree + 1` long, or if a de Boor coefficient would
/// divide by zero. The second cannot happen: the coefficient at stage `r`
/// divides by `knots[j + 1 + span − r] − knots[j + span − degree]`, and since
/// `j ≥ r` and `j ≤ degree` those two indices always sit on opposite sides of
/// the span — at or below `span` on the left, at or above `span + 1` on the
/// right. `span` is a span of positive width, so the difference is at least its
/// width. The assertion is there because that argument is about the knot
/// vector's invariants and not about anything the type system knows.
pub(crate) fn de_boor<C: Coefficient>(
    degree: usize,
    knots: &[f64],
    span: usize,
    parameter: f64,
    points: &mut [[C; 4]],
) -> [C; 4] {
    assert_eq!(
        points.len(),
        degree + 1,
        "a degree {degree} de Boor stage works on exactly {} points",
        degree + 1
    );

    let one = C::exactly(1.0);
    let at = C::exactly(parameter);
    for round in 1..=degree {
        for j in (round..=degree).rev() {
            let below = knots[j + span - degree];
            let above = knots[j + 1 + span - round];
            assert!(
                below < above,
                "the de Boor span from {below} to {above} is shut, which the knot vector's \
                 own invariants rule out"
            );
            let share = (at - C::exactly(below)).divided_by(C::exactly(above) - C::exactly(below));
            points[j] = blended(one - share, points[j - 1], share, points[j]);
        }
    }
    points[degree]
}

/// The control points of the derivative spline that an evaluation at `span`
/// reaches, in homogeneous coordinates.
///
/// The derivative of a B-spline is a B-spline of degree `p − 1` on the derived
/// net `Q[i] = p · (P[i+1] − P[i]) / (knots[i+p+1] − knots[i+1])`, over the knot
/// vector with the first and last knot dropped. That is the same de Boor
/// machinery through another door, which is why derivatives ship in the same
/// stage as evaluation rather than as a part of their own.
///
/// `active` holds the `degree + 1` points the span reaches and `out` receives
/// the `degree` derived points below them: `out[j]` is `Q[j + span − degree]`.
/// Evaluating `out` needs degree `degree − 1`, the knots with both ends
/// dropped, and span `span − 1`.
///
/// **The caller must not build a [`KnotVector`](crate::KnotVector) for that.**
/// A degree-1 curve derives to degree 0, and `KnotVector::new` refuses degree
/// zero — rightly, since a degree-0 vector describes no curve. The derived net
/// is evaluated as a bare slice instead, and at degree 0 [`de_boor`] runs no
/// stages and hands back the one point it was given.
///
/// # Panics
///
/// If the slices are the wrong length, or if a derived span is shut. The second
/// cannot happen either, and for the same reason as in [`de_boor`]: the
/// reachable `i` run from `span − degree` to `span − 1`, so `i + 1 ≤ span` and
/// `i + degree + 1 ≥ span + 1`, and the knots at those two indices straddle a
/// span of positive width. Note this needs no appeal to the multiplicity rule —
/// unreachable `i` may well have a shut derived span, and are never asked for.
pub(crate) fn derived_points<C: Coefficient>(
    degree: usize,
    knots: &[f64],
    span: usize,
    active: &[[C; 4]],
    out: &mut [[C; 4]],
) {
    assert_eq!(
        active.len(),
        degree + 1,
        "one point per active control point"
    );
    assert_eq!(
        out.len(),
        degree,
        "one derived point fewer than active ones"
    );

    let rise = C::exactly(degree as f64);
    for (j, derived) in out.iter_mut().enumerate() {
        let i = j + span - degree;
        let below = knots[i + 1];
        let above = knots[i + degree + 1];
        assert!(
            below < above,
            "the derived span from {below} to {above} is shut, which an evaluation at this \
             span cannot reach"
        );
        let width = C::exactly(above) - C::exactly(below);
        *derived = core::array::from_fn(|axis| {
            (rise * (active[j + 1][axis] - active[j][axis])).divided_by(width)
        });
    }
}

/// The homogeneous point brought back to an ordinary one.
///
/// # Panics
///
/// If the weight is not above zero. Every stored weight is checked to be above
/// zero at construction and de Boor mixes them with non-negative coefficients
/// that sum to one, so a weight of zero here means the net is not what
/// construction promised. That is a broken invariant, not bad caller input, and
/// a corrupt point is worse than a crash (`docs/decisions.md` §4.1).
pub(crate) fn divided(homogeneous: [f64; 4]) -> Point {
    let [x, y, z, weight] = homogeneous;
    assert!(
        weight > 0.0,
        "the weight at this parameter came out as {weight}, which a net of positive weights \
         cannot produce"
    );
    Point::new(x / weight, y / weight, z / weight)
}

/// The homogeneous enclosure brought back to a point the cascade will take.
///
/// # Why an exact point comes out of interval arithmetic
///
/// A side of zero width says the true coordinate is that `f64` and nothing
/// else, so [`PredicatePoint::Exact`] is sound. It is also strictly stronger
/// than handing back a zero-width box: only an exact point can run level 3 of
/// the cascade, and that is what makes a degree-1 curve through exact control
/// points, read at an exact parameter, decidable at all. Always saying
/// [`PredicatePoint::Constructed`] would be safe in the sense of never
/// certifying a wrong sign, and it would refuse cases whose answer is there to
/// be had — a timid filter, which is the thing the acceptance oracle exists to
/// catch (`docs/decisions.md` §14.7).
///
/// # A weight that straddles zero
///
/// Every stored weight is above zero, so the true weight here is too. A wide
/// enough enclosure can still hold zero anyway — a coefficient one rounding
/// outside `[0, 1]` against control weights sixteen orders of magnitude apart
/// does it — and then the true point really could be anywhere on the line.
/// Unlike [`divided`], that is not a broken invariant; it is the arithmetic
/// saying it does not know, and every real number is the honest answer. It can
/// never certify anything, which is exactly what an enclosure that knows
/// nothing should do.
pub(crate) fn enclosed(homogeneous: [Interval; 4]) -> PredicatePoint {
    let weight = homogeneous[3];
    if weight.contains_zero() {
        return PredicatePoint::Constructed([Interval::ENTIRE; 3]);
    }

    let sides: [Interval; 3] =
        core::array::from_fn(|axis| quotient_enclosing(homogeneous[axis], weight));
    if sides.iter().all(|side| side.width() == 0.0) {
        return PredicatePoint::Exact(sides.map(Interval::lo));
    }
    PredicatePoint::Constructed(sides)
}

/// The first derivative of the rational curve or surface, from the homogeneous
/// value and the homogeneous derivative.
///
/// A rational spline is `C = A / w`, so `C' = (A' − w'·C) / w`. The affine
/// control points cannot be differentiated directly — that would give the
/// derivative of the numerator and quietly drop the `w'·C` term, which is zero
/// only when every weight is the same.
///
/// # Panics
///
/// If the weight is not above zero, for the reason [`divided`] gives.
pub(crate) fn rational_derivative(value: [f64; 4], derivative: [f64; 4]) -> Vector {
    let point = divided(value);
    let weight = value[3];
    let weight_rate = derivative[3];
    let [x, y, z] = point.coordinates();
    Vector::new(
        (derivative[0] - weight_rate * x) / weight,
        (derivative[1] - weight_rate * y) / weight,
        (derivative[2] - weight_rate * z) / weight,
    )
}

/// How far a divided point can be from the true one, per coordinate, or `None`
/// when [`EVALUATION_ROUNDOFF`]'s precondition does not hold.
///
/// `relative` is [`EVALUATION_ROUNDOFF`] for one de Boor pass and
/// [`after_two_passes`] for a surface's two. `largest` is the biggest magnitude
/// among the homogeneous coordinates of the control points the evaluation
/// touched, and `homogeneous` is the result it produced.
///
/// # The divide
///
/// The recursion's bound is on the homogeneous numbers. Writing the computed
/// pair as `X = X_true + e₁` and `W = W_true + e₂` with `|e₁|, |e₂| ≤ E`:
///
/// ```text
/// X/W − X_true/W_true = (e₁ − (X/W)·e₂) / (W − e₂)
/// ```
///
/// so the divide contributes `(E_x + |X/W|·E_w) / (W − E_w)`, and rounding the
/// divide itself contributes `u·|X/W|` on top. That last term scales with the
/// answer rather than with `largest`, which is why it is applied here and not
/// folded into the published constant.
///
/// # One largest per coordinate, not one for the point
///
/// `largest` carries four numbers because the four homogeneous coordinates
/// never mix: the recursion runs the same coefficients down each of them
/// separately. Charging the weight's error against the largest of all four
/// would be sound but useless — a curve with coordinates around a kilometre and
/// weights around a thousandth would get a bound a million times wider than the
/// truth, which is the whole of the wide-dynamic-range family.
///
/// A coordinate whose touched control points are all exactly zero gets a bound
/// of exactly zero, and that is right rather than optimistic: `0·x` and `0 + 0`
/// are exact, so a flat curve's `z` really is exact.
///
/// The bound is nudged up by `8·f64::EPSILON` at the end. Forming it is itself
/// half a dozen `f64` operations worth at most `3·f64::EPSILON` between them,
/// and handing back a bound that its own arithmetic rounded below the truth
/// would defeat the point of publishing one.
pub(crate) fn roundoff_of_a_divided_point(
    relative: f64,
    largest: [f64; 4],
    homogeneous: [f64; 4],
) -> Option<f64> {
    let mut spread = [0.0f64; 4];
    for (axis, &size) in largest.iter().enumerate() {
        if !(size.is_finite() && size <= f64::MAX / 2.0) {
            return None;
        }
        if size == 0.0 {
            continue;
        }
        if !(size >= f64::MIN_POSITIVE && product_is_normal(relative, size)) {
            return None;
        }
        spread[axis] = relative * size;
    }

    let weight = homogeneous[3];
    if !(weight.is_finite() && weight >= f64::MIN_POSITIVE) {
        return None;
    }
    let room = weight - spread[3];
    if room < f64::MIN_POSITIVE {
        return None;
    }

    let mut worst: f64 = 0.0;
    for axis in 0..3 {
        let divided = (homogeneous[axis] / weight).abs();
        if !divided.is_finite() {
            return None;
        }
        // 1/(1 − u) ≤ 1 + 2u, and the multiplication rounds once more.
        let above = divided * (1.0 + 2.0 * f64::EPSILON);
        let bound = (spread[axis] + above * spread[3]) / room + (f64::EPSILON / 2.0) * above;
        if !bound.is_finite() {
            return None;
        }
        worst = worst.max(bound);
    }
    Some(worst * (1.0 + 8.0 * f64::EPSILON))
}

/// The same fraction of `largest` after two de Boor passes instead of one.
///
/// A surface collapses each active row along `v` and then collapses the results
/// along `u`. The second pass starts from numbers already `K·M` off and no
/// bigger than `M` in truth, so `E ≤ K·M·(1 + K) + K·M = M·(2K + K²)`. Written
/// as `2K + K²` rather than `(1 + K)² − 1` because the second form cancels away
/// the `K²` term it is there to keep.
pub(crate) fn after_two_passes(one_pass: f64) -> f64 {
    (2.0 * one_pass + one_pass * one_pass) * (1.0 + f64::EPSILON)
}

/// `rest·before + share·after`, on all four homogeneous coordinates.
fn blended<C: Coefficient>(rest: C, before: [C; 4], share: C, after: [C; 4]) -> [C; 4] {
    core::array::from_fn(|axis| rest * before[axis] + share * after[axis])
}

/// A range holding every quotient of a number in `over` by a number in `under`.
///
/// [`Interval`] has no public division and `docs/decisions.md` §15.9 freezes
/// the crate it lives in, so this is built out of `Interval::new`, `lo` and
/// `hi` rather than added there.
///
/// # Why the four corners and a step enclose
///
/// `under` keeps away from zero, so the quotient is monotone in each argument
/// across the whole box and its extremes sit at the corners. Each corner is one
/// `f64` division, and IEEE division rounds to nearest: the true quotient of
/// two `f64` is therefore somewhere between the computed one's two neighbours.
/// Widening the smallest corner down a step and the largest up a step covers
/// all four, since `next_down` and `next_up` both keep the order.
///
/// That is at most one step wider at each end than a directed-rounding divide
/// would give, which is the price of keeping the freeze.
///
/// # Why an exact corner is not widened
///
/// A quotient that came out exactly — [`quotient_is_exact`] proves it, rather
/// than assuming a `1.0` divisor is the only case — is widened by nothing. This
/// is not an optimisation. Widen it and *every* enclosure has width, a
/// degree-1 curve through dyadic control points at a dyadic parameter stops
/// being an exact point, level 3 of the cascade never runs on a spline point,
/// and the acceptance half of the oracle fails.
///
/// # Panics
///
/// If `under` touches zero. Dividing by a range that straddles zero has no
/// enclosure to hand back, and every caller here has already checked: the knot
/// differences are positive by the knot vector's own invariants, and
/// [`enclosed`] tests the weight before it divides by it.
fn quotient_enclosing(over: Interval, under: Interval) -> Interval {
    assert!(
        !under.contains_zero(),
        "dividing by {under:?}, which touches zero, has no enclosure"
    );

    let mut lowest = f64::INFINITY;
    let mut highest = f64::NEG_INFINITY;
    for top in [over.lo(), over.hi()] {
        for bottom in [under.lo(), under.hi()] {
            let corner = top / bottom;
            let (below, above) = if quotient_is_exact(top, bottom, corner) {
                (corner, corner)
            } else {
                (corner.next_down(), corner.next_up())
            };
            lowest = lowest.min(below);
            highest = highest.max(above);
        }
    }

    // A corner is only ever a NaN if an infinite range was divided by an
    // infinite one, and a divisor with an infinite end cannot reach here from a
    // net of finite control points.
    Interval::new(lowest, highest)
        .expect("no corner is a NaN, so the smallest is below the largest")
}

/// Whether `top` divided by `bottom` is `corner` with nothing rounded away.
///
/// `corner * bottom` is worked out to the last bit and compared with `top`. The
/// [`product_is_exact`] guard is what makes that comparison mean something: a
/// product down in the subnormals loses its low half, and without the guard a
/// quotient off by a fraction of the smallest `f64` would be certified exact
/// and handed to the cascade as a point.
fn quotient_is_exact(top: f64, bottom: f64, corner: f64) -> bool {
    if !product_is_exact(corner, bottom) {
        return false;
    }
    let (high, low) = two_product(corner, bottom);
    high == top && low == 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A typo in the count is the cheapest way for this constant to go wrong,
    /// and it is the one thing about it a test can see directly.
    #[test]
    fn the_published_bound_is_a_hundred_and_thirty_five_roundings_and_a_hair() {
        let unit_roundoff = f64::EPSILON / 2.0;
        assert!(EVALUATION_ROUNDOFF > 135.0 * unit_roundoff);
        assert!(EVALUATION_ROUNDOFF < 136.0 * unit_roundoff);
        assert!(EVALUATION_ROUNDOFF.is_normal());
    }

    #[test]
    fn two_passes_cost_more_than_twice_one() {
        let one = EVALUATION_ROUNDOFF;
        assert!(after_two_passes(one) > 2.0 * one);
        assert!(after_two_passes(one) < 2.1 * one);
    }

    /// The precondition is the whole reason this returns an `Option`. A weight
    /// down in the subnormals is exactly the case `docs/decisions.md` §12.2
    /// describes: the model stops holding and the honest answer is that there
    /// is no bound to publish.
    #[test]
    fn a_weight_too_small_to_be_normal_gets_no_bound() {
        let subnormal = f64::MIN_POSITIVE / 2.0;
        assert_eq!(
            roundoff_of_a_divided_point(
                EVALUATION_ROUNDOFF,
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0, subnormal]
            ),
            None
        );
    }

    #[test]
    fn a_control_net_too_small_to_be_normal_gets_no_bound() {
        assert_eq!(
            roundoff_of_a_divided_point(
                EVALUATION_ROUNDOFF,
                [f64::MIN_POSITIVE / 2.0, 0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0, 1.0]
            ),
            None
        );
    }

    /// `EVALUATION_ROUNDOFF * 1e-300` is itself subnormal, so the bound would
    /// flush to zero and certify perfection. That is §12.2's bug verbatim.
    #[test]
    fn a_control_net_whose_bound_would_flush_to_zero_gets_no_bound() {
        assert_eq!(
            roundoff_of_a_divided_point(
                EVALUATION_ROUNDOFF,
                [1e-300, 0.0, 0.0, 1e-300],
                [1e-300, 0.0, 0.0, 1e-300]
            ),
            None
        );
    }

    #[test]
    fn an_ordinary_net_gets_a_bound_a_few_roundings_wide() {
        let bound = roundoff_of_a_divided_point(
            EVALUATION_ROUNDOFF,
            [4.0, 0.0, 0.0, 1.0],
            [2.0, 0.0, 0.0, 1.0],
        )
        .expect("nothing here is anywhere near the ends of the f64 range");
        assert!(bound > 0.0);
        assert!(
            bound < 1e-13,
            "a bound of {bound} on numbers this size is absurd"
        );
    }

    /// A coordinate every touched control point leaves at zero is computed
    /// exactly, and the bound has to say so — otherwise a flat curve carries a
    /// made-up uncertainty in `z` for the rest of the cascade to widen.
    #[test]
    fn a_coordinate_that_is_zero_all_the_way_along_is_bounded_by_nothing_at_all() {
        let bound = roundoff_of_a_divided_point(
            EVALUATION_ROUNDOFF,
            [0.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        )
        .expect("a net of weights alone is as ordinary as it gets");
        assert_eq!(bound, 0.0);
    }

    /// One third is not an `f64`, so the enclosure of `1 / 3` has to hold the
    /// true third strictly inside it, one step below the nearest `f64` and one
    /// above.
    ///
    /// The check is exact integer arithmetic rather than a comparison against
    /// `1.0 / 3.0`, which is the very number under test. Both bounds land
    /// between a quarter and a half, where every `f64` is a whole multiple of
    /// `2⁻⁵⁴`, so multiplying by `2⁵⁴` is exact and comparing `3 * k` with
    /// `2⁵⁴` settles `k · 2⁻⁵⁴ < 1/3` with nothing rounded away.
    #[test]
    fn the_enclosure_of_a_third_holds_the_true_third_strictly_inside_it() {
        let third = quotient_enclosing(Interval::exactly(1.0), Interval::exactly(3.0));
        let whole_multiples_of = 2f64.powi(54);
        for bound in [third.lo(), third.hi()] {
            assert!(
                (0.25..0.5).contains(&bound),
                "{bound} is not where a third is, so the exact check below says nothing"
            );
        }

        let low = (third.lo() * whole_multiples_of) as i128;
        let high = (third.hi() * whole_multiples_of) as i128;
        assert!(
            3 * low < 1i128 << 54,
            "{:?} starts at or above a third",
            third
        );
        assert!(
            3 * high > 1i128 << 54,
            "{:?} ends at or below a third",
            third
        );
    }

    /// The number the ordinary evaluator would produce is inside the enclosure
    /// of the same divide, which is the whole claim in one line.
    #[test]
    fn the_f64_quotient_is_inside_the_enclosure_of_the_same_divide() {
        for (top, bottom) in [(1.0, 3.0), (-7.0, 9.0), (1e-6, 1e6), (2.5, 0.1)] {
            let enclosure = quotient_enclosing(Interval::exactly(top), Interval::exactly(bottom));
            assert!((enclosure.lo()..=enclosure.hi()).contains(&(top / bottom)));
        }
    }

    /// A divide that came out exactly is not widened. Every enclosure having
    /// width would cost the cascade its exact level on spline points, which is
    /// the acceptance half of `docs/decisions.md` §14.7.
    #[test]
    fn a_divide_that_rounded_nothing_stays_a_single_number() {
        for (top, bottom, quotient) in [(1.0, 4.0, 0.25), (3.0, 1.5, 2.0), (0.0, 7.0, 0.0)] {
            let enclosure = quotient_enclosing(Interval::exactly(top), Interval::exactly(bottom));
            assert_eq!(
                enclosure.width(),
                0.0,
                "{top} / {bottom} is exactly {quotient}"
            );
            assert_eq!(enclosure.lo(), quotient);
        }
    }

    #[test]
    #[should_panic(expected = "touches zero")]
    fn dividing_by_a_range_that_straddles_zero_has_no_answer() {
        let _ = quotient_enclosing(Interval::exactly(1.0), Interval::new(-1.0, 1.0).unwrap());
    }

    #[test]
    fn a_homogeneous_point_that_divides_exactly_comes_back_as_an_exact_point() {
        let exactly = |value: f64| Interval::exactly(value);
        let point = enclosed([exactly(3.0), exactly(-2.0), exactly(0.0), exactly(4.0)]);
        assert_eq!(point, PredicatePoint::Exact([0.75, -0.5, 0.0]));
    }

    /// Three fifths is not an `f64`, so this point is a box — and its `z`, which
    /// divided exactly, is still a single number inside that box.
    #[test]
    fn a_homogeneous_point_with_one_rounded_coordinate_comes_back_as_a_box() {
        let exactly = |value: f64| Interval::exactly(value);
        let point = enclosed([exactly(3.0), exactly(0.0), exactly(0.0), exactly(5.0)]);
        match point {
            PredicatePoint::Constructed([x, y, z]) => {
                assert!(x.width() > 0.0);
                assert_eq!((y.width(), z.width()), (0.0, 0.0));
            }
            other => panic!("three fifths is not an f64: {other:?}"),
        }
    }

    /// Not a broken invariant — the stored weights are all above zero, but an
    /// enclosure wide enough to hold zero leaves the true point anywhere at all,
    /// and saying so is the only sound answer.
    #[test]
    fn a_weight_whose_enclosure_holds_zero_puts_the_point_nowhere_in_particular() {
        let straddling = Interval::new(-1e-30, 1.0).unwrap();
        let point = enclosed([
            Interval::exactly(1.0),
            Interval::exactly(1.0),
            Interval::exactly(1.0),
            straddling,
        ]);
        assert_eq!(point, PredicatePoint::Constructed([Interval::ENTIRE; 3]));
    }

    /// Coordinates around a kilometre with weights around a thousandth. Sharing
    /// one magnitude across all four homogeneous coordinates would charge the
    /// weight's error at the size of `x` and hand back a bound a million times
    /// too wide.
    #[test]
    fn a_curve_whose_weights_and_coordinates_live_at_different_scales_gets_a_useful_bound() {
        let bound = roundoff_of_a_divided_point(
            EVALUATION_ROUNDOFF,
            [1e3, 0.0, 0.0, 1e-3],
            [1e3, 0.0, 0.0, 1e-3],
        )
        .expect("a thousandth is a perfectly normal number");
        assert!(
            bound < 1e-7,
            "a coordinate of 1e6 is worth about 1.5e-8 of rounding, not {bound}"
        );
    }
}
