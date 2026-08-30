use crate::expansion::{expansion_sign, expansion_sum, product_is_exact, two_product};
use crate::{
    Decision, Filtered, Interval, Level, Point, Policy, Predicate, ProvenanceLog, Record, decide,
    resolve_under_policy,
};

/// The name this predicate goes by in provenance records.
pub const ORIENT2D: &str = "orient2d";

/// Which way the three points `a`, `b`, `c` turn.
///
/// [`Decision::Pos`] when they run anticlockwise, so `c` lies to the left of
/// the line from `a` to `b`. [`Decision::Neg`] when they run clockwise.
/// [`Decision::Zero`] when all three sit on one straight line. This is the same
/// sign convention as the `robust` crate, which the differential test relies
/// on.
///
/// Exact `f64` inputs always get a certified answer, as long as the products of
/// coordinates stay inside the normal `f64` range. If one overflows, or if one
/// is small enough that its roundoff term would be subnormal, exact arithmetic
/// stops being exact and the answer is [`Decision::Undecidable`] rather than a
/// guess. Coordinates that far apart in magnitude are what decisions.md §3.5's
/// dynamic-range check exists to reject.
///
/// ```
/// # use rcad_predicates::{orient2d, Decision};
/// let anticlockwise = orient2d([0.0, 0.0], [1.0, 0.0], [0.0, 1.0]);
/// assert_eq!(anticlockwise, Decision::Pos);
/// assert_eq!(orient2d([0.0, 0.0], [1.0, 1.0], [2.0, 2.0]), Decision::Zero);
/// ```
pub fn orient2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Decision {
    orient2d_reporting_level(a, b, c).0
}

/// [`orient2d`], and which tier of the cascade settled it.
///
/// A diagnostic. Tests and benchmarks use it to check that the cheap levels are
/// carrying the load they are supposed to.
pub fn orient2d_reporting_level(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> (Decision, Level) {
    decide(&Orient2d { a, b, c })
}

/// The tolerant entry point, separately named so nobody reaches it by accident.
///
/// Runs the cascade, then hands anything it could not certify to the policy.
/// Under [`Strictness::TolerantFallback`] a narrow enough enclosure is read as
/// [`Decision::Zero`] and `log` picks up a record saying so.
///
/// A certified answer leaves `log` untouched — an empty log is how a caller
/// knows nothing was approximated (decisions.md §5).
///
/// [`Strictness::TolerantFallback`]: crate::Strictness::TolerantFallback
pub fn orient2d_tolerantly(
    a: [f64; 2],
    b: [f64; 2],
    c: [f64; 2],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Decision {
    let (decision, level) = orient2d_reporting_level(a, b, c);
    if decision.is_certified() {
        return decision;
    }
    // A record carries [`Point`], which is three-dimensional. A flat predicate
    // has no third coordinate to report, so its inputs go in at height zero.
    let inputs = [at_height_zero(a), at_height_zero(b), at_height_zero(c)];
    let record = Record::new(ORIENT2D, &inputs, level, decision);
    resolve_under_policy(record, policy, log)
}

fn at_height_zero(point: [f64; 2]) -> Point {
    Point::from([point[0], point[1], 0.0])
}

struct Orient2d {
    a: [f64; 2],
    b: [f64; 2],
    c: [f64; 2],
}

impl Predicate for Orient2d {
    fn name(&self) -> &'static str {
        ORIENT2D
    }

    fn on_floats(&self) -> Option<Filtered> {
        let (acx, acy) = (self.a[0] - self.c[0], self.a[1] - self.c[1]);
        let (bcx, bcy) = (self.b[0] - self.c[0], self.b[1] - self.c[1]);

        let forwards = acx * bcy;
        let backwards = acy * bcx;

        // The bound has to allow for the two products cancelling, so it sums
        // their magnitudes rather than taking the magnitude of the difference.
        // Shewchuk calls this the permanent of the matrix.
        Some(Filtered {
            value: forwards - backwards,
            error_bound: ORIENT2D_ERROR_BOUND * (forwards.abs() + backwards.abs()),
        })
    }

    fn on_intervals(&self) -> Interval {
        let [ax, ay] = self.a.map(Interval::exactly);
        let [bx, by] = self.b.map(Interval::exactly);
        let [cx, cy] = self.c.map(Interval::exactly);

        (ax - cx) * (by - cy) - (ay - cy) * (bx - cx)
    }

    fn exactly(&self) -> Option<Decision> {
        match orient2d_exactly(self.a, self.b, self.c) {
            Decision::Neg => Some(Decision::Neg),
            Decision::Zero => Some(Decision::Zero),
            Decision::Pos => Some(Decision::Pos),
            // A product overflowed or underflowed. Exact arithmetic that is
            // not exact has nothing to offer, so this level cannot run.
            Decision::Undecidable { .. } => None,
        }
    }
}

/// Level 1's forward error bound, as a multiple of the permanent.
///
/// `epsilon` below is the `f64` unit roundoff, 2⁻⁵³: half an ulp of 1, and so
/// the most a single rounding can cost in relative terms.
///
/// Where the 3 comes from — walk the longest chain of roundings between an
/// input coordinate and one of the two products in [`Orient2d::on_floats`].
/// Each difference (`acx = a.x - c.x`) is one rounding. Multiplying two of them
/// (`acx * bcy`) is a second, and it multiplies two already-rounded
/// differences, so the two roundings they carry come along: three in all. Each
/// costs at most a factor of `(1 + epsilon)`, so each product is within
/// `(1 + epsilon)^3 - 1 <= 3*epsilon + 3*epsilon^2 + epsilon^3` of the true
/// product, and the two together are within that fraction of the permanent.
///
/// The final subtraction `forwards - backwards` is a fourth rounding, but it
/// does not add a fourth `epsilon` to the count: its error is at most `epsilon`
/// times the *result*, so it can only shrink a value that already clears the
/// bound, by a factor of `(1 + epsilon)`. That, the higher-order terms above,
/// and the rounding of the permanent itself are what the `16*epsilon^2` covers.
///
/// The value is Shewchuk's `ccwerrboundA`, from "Adaptive Precision
/// Floating-Point Arithmetic and Fast Robust Geometric Predicates" (1997) §4.3
/// and the `exactinit` routine of his `predicates.c`.
///
/// The bound holds only when nothing overflows or falls into the subnormals.
/// Overflow makes the permanent infinite, which certifies nothing, and level 3
/// then reports that it cannot run either.
const ORIENT2D_ERROR_BOUND: f64 = {
    let epsilon = f64::EPSILON / 2.0;
    (3.0 + 16.0 * epsilon) * epsilon
};

/// The exact sign of the determinant, in expansion arithmetic.
///
/// The value wanted is the determinant of the rows `a - c`, `b - c`. Those
/// differences are not `f64`, so the determinant is expanded over the original
/// coordinates instead. Multilinearity in the rows gives
///
/// ```text
/// det[a-c, b-c] = det[a,b] - det[a,c] - det[c,b] = det[a,b] - det[a,c] + det[b,c]
/// ```
///
/// and each of those is one cross product `M(u,v) = u.x*v.y - u.y*v.x`, exact
/// in four entries. Three of them sum to at most twelve — the capacity below.
///
/// Comes back [`Decision::Undecidable`] when a product overflowed or
/// underflowed, because then the arithmetic is not exact and there is nothing
/// honest to report. Additions never need that check: they are exact for every
/// pair of `f64` that does not overflow.
fn orient2d_exactly(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Decision {
    let cannot_be_done = Decision::Undecidable {
        enclosure: f64::INFINITY,
    };
    let (Some((ab, ab_len)), Some((ac, ac_len)), Some((bc, bc_len))) =
        (cross(a, b), cross(a, c), cross(b, c))
    else {
        return cannot_be_done;
    };

    let mut negated_ac = ac;
    for entry in &mut negated_ac[..ac_len] {
        *entry = -*entry;
    }

    let mut without_bc = [0.0f64; 8];
    let without_bc_len = expansion_sum(&ab[..ab_len], &negated_ac[..ac_len], &mut without_bc);
    let mut total = [0.0f64; 12];
    let total_len = expansion_sum(&without_bc[..without_bc_len], &bc[..bc_len], &mut total);

    expansion_sign(&total[..total_len])
}

/// `u.x * v.y - u.y * v.x`, exactly, or `None` if a product underflowed.
fn cross(u: [f64; 2], v: [f64; 2]) -> Option<([f64; 4], usize)> {
    if !product_is_exact(u[0], v[1]) || !product_is_exact(u[1], v[0]) {
        return None;
    }
    let (forwards, forwards_roundoff) = two_product(u[0], v[1]);
    let (backwards, backwards_roundoff) = two_product(u[1], v[0]);
    let mut out = [0.0; 4];
    let len = expansion_sum(
        &[forwards_roundoff, forwards],
        &[-backwards_roundoff, -backwards],
        &mut out,
    );
    Some((out, len))
}

#[cfg(test)]
mod tests {
    use super::{
        ORIENT2D, orient2d, orient2d_exactly, orient2d_reporting_level, orient2d_tolerantly,
    };
    use crate::{Decision, Level, Policy, ProvenanceLog};

    /// The naive `f64` determinant. Exact when the coordinates are small
    /// integers, which is what makes it a usable oracle for the sign
    /// convention.
    fn naive_orient2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
        (a[0] - c[0]) * (b[1] - c[1]) - (a[1] - c[1]) * (b[0] - c[0])
    }

    #[test]
    fn exact_arithmetic_agrees_with_the_plain_determinant_on_small_integers() {
        let mut checked = 0;
        for ax in -2..=2 {
            for by in -2..=2 {
                for cx in -2..=2 {
                    for cy in -2..=2 {
                        let a = [f64::from(ax), 1.0];
                        let b = [3.0, f64::from(by)];
                        let c = [f64::from(cx), f64::from(cy)];
                        let plain = naive_orient2d(a, b, c);
                        let expected = if plain > 0.0 {
                            Decision::Pos
                        } else if plain < 0.0 {
                            Decision::Neg
                        } else {
                            Decision::Zero
                        };
                        assert_eq!(orient2d_exactly(a, b, c), expected, "{a:?} {b:?} {c:?}");
                        checked += 1;
                    }
                }
            }
        }
        assert_eq!(checked, 625);
    }

    #[test]
    fn three_points_turning_left_are_positive() {
        assert_eq!(orient2d([0.0, 0.0], [1.0, 0.0], [0.0, 1.0]), Decision::Pos);
        assert_eq!(orient2d([0.0, 0.0], [1.0, 0.0], [0.0, -1.0]), Decision::Neg);
    }

    /// Three points on the line `3x + 5y = 0`, spread far enough apart that the
    /// two products no longer fit in an `f64`. Once they round, only exact
    /// arithmetic can prove the determinant is zero.
    #[test]
    fn three_points_in_a_line_are_decided_exactly() {
        let (decision, level) = orient2d_reporting_level([5.0, -3.0], [0.0, 0.0], [5e9, -3e9]);
        assert_eq!(decision, Decision::Zero);
        assert_eq!(level, Level::Exact);
    }

    #[test]
    fn a_point_far_off_the_line_is_settled_by_the_cheapest_level() {
        let (decision, level) = orient2d_reporting_level([0.0, 0.0], [1.0, 0.0], [0.3, 1.0]);
        assert_eq!(decision, Decision::Pos);
        assert_eq!(level, Level::Float);
    }

    #[test]
    fn a_repeated_point_makes_the_determinant_zero() {
        let a = [1.5, -2.25];
        assert_eq!(orient2d(a, [0.0, 3.0], a), Decision::Zero);
        assert_eq!(orient2d(a, a, [4.0, 5.0]), Decision::Zero);
    }

    #[test]
    fn swapping_two_points_flips_the_answer() {
        let (a, b, c) = ([0.5, 0.25], [3.0, -1.0], [-4.0, 7.0]);
        assert_eq!(orient2d(a, b, c), Decision::Pos);
        assert_eq!(orient2d(b, a, c), Decision::Neg);
    }

    #[test]
    fn a_certified_answer_leaves_the_log_empty() {
        let mut log = ProvenanceLog::new();
        let decision = orient2d_tolerantly(
            [0.0, 0.0],
            [1.0, 0.0],
            [0.3, 1.0],
            &Policy::tolerant(1.0),
            &mut log,
        );
        assert_eq!(decision, Decision::Pos);
        assert!(
            log.is_empty(),
            "a certified operation must record nothing at all"
        );
    }

    #[test]
    fn an_undecided_answer_names_the_predicate_in_the_log() {
        let mut log = ProvenanceLog::new();
        let decision = orient2d_tolerantly(
            [1e-200, 0.0],
            [0.0, 1e-200],
            [0.0, 0.0],
            &Policy::certified(),
            &mut log,
        );
        assert!(matches!(decision, Decision::Undecidable { .. }));
        assert_eq!(log.len(), 1);
        assert_eq!(log.records()[0].predicate(), ORIENT2D);
        assert_eq!(log.records()[0].inputs().len(), 3);
        assert_eq!(log.records()[0].level(), Level::Interval);
    }

    #[test]
    fn coordinates_big_enough_to_overflow_the_products_are_undecidable() {
        let decision = orient2d([1e300, 0.0], [0.0, 1e300], [-1e300, -1e300]);
        assert_eq!(
            decision,
            Decision::Undecidable {
                enclosure: f64::INFINITY
            }
        );
    }

    /// The true determinant here is `1e-400`, which is positive and far below
    /// the smallest `f64`. Both products underflow, so exact arithmetic cannot
    /// run and there is no honest sign to report.
    #[test]
    fn coordinates_small_enough_to_underflow_the_products_are_undecidable() {
        let (decision, level) = orient2d_reporting_level([1e-200, 0.0], [0.0, 1e-200], [0.0, 0.0]);
        match decision {
            Decision::Pos | Decision::Neg => panic!("nothing here can prove a sign"),
            Decision::Zero => panic!("these three points are not on one line"),
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
        }
        assert_eq!(
            level,
            Level::Interval,
            "exact arithmetic must decline to run"
        );
    }
}
