use crate::expansion::{expansion_sign, expansion_sum, product_is_exact, two_product};
use crate::power_of_two::{exponent_that_brings_into_range, scaled_by_a_power_of_two};
use crate::{
    Decision, Filtered, Interval, Level, Measures, NOTHING_IS_KNOWN, Point, Policy, Predicate,
    ProvenanceLog, Record, are_real_numbers, decide, resolve_under_policy,
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
/// Exact `f64` inputs get a certified answer whenever one exists. Coordinates
/// that are uniformly tiny or uniformly enormous have their products fall off
/// the bottom or the top of the `f64` range, and those are decided anyway: the
/// whole input is moved into range by an exact power of two, which multiplies
/// the determinant by a positive number and so leaves its sign alone
/// ([`decided_after_normalising`]).
///
/// What is left as [`Decision::Undecidable`] is a genuine *spread* — a model
/// whose coordinates sit far from the origin while its differences are far
/// below it, so that no single scaling brings both into range at once. That is
/// the case decisions.md §3.5's dynamic-range check is for, and being a ratio
/// is exactly what lets it see this and not the uniform cases (§16.11).
///
/// A coordinate that is not a real number at all gets [`NOTHING_IS_KNOWN`]
/// before any of that runs. [`are_real_numbers`] says why.
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
///
/// The guard reports [`Level::Interval`] because that is the level a coordinate
/// that is not a real number would have reached — it arrives there as
/// [`Interval::ENTIRE`] — and it is the level the cascade already reports for
/// the refusals it works out for itself.
///
/// A retry reports whichever level settled it, which is the level that did the
/// work. Nothing marks the answer as having been normalised, because nothing
/// about it is weaker: scaling by a power of two is exact, so a certified
/// answer here is certified in the same sense as any other (decisions.md §5).
///
/// Both public entry points funnel through here, so both get the retry.
pub fn orient2d_reporting_level(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> (Decision, Level) {
    if !are_real_numbers(a.into_iter().chain(b).chain(c)) {
        return (NOTHING_IS_KNOWN, Level::Interval);
    }
    let refused = match decide(&Orient2d { a, b, c }) {
        (Decision::Undecidable { enclosure }, level) => {
            (Decision::Undecidable { enclosure }, level)
        }
        settled => return settled,
    };
    decided_after_normalising(a, b, c).unwrap_or(refused)
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
    let Decision::Undecidable { enclosure } = decision else {
        return decision;
    };
    // A record carries [`Point`], which is three-dimensional. A flat predicate
    // has no third coordinate to report, so its inputs go in at height zero.
    let inputs = [at_height_zero(a), at_height_zero(b), at_height_zero(c)];
    let record = Record::new(ORIENT2D, &inputs, level, enclosure, Measures::Area);
    resolve_under_policy(record, policy, log)
}

fn at_height_zero(point: [f64; 2]) -> Point {
    Point::from([point[0], point[1], 0.0])
}

/// One more run of the whole cascade, on a copy of the input moved into range
/// by an exact power of two. `None` when there is no such copy, or when the
/// copy could not be settled either.
///
/// This determinant is homogeneous of degree two in the coordinates, so scaling
/// all six by `s > 0` multiplies it by `s^2` and leaves its sign alone. That is
/// what the metamorphic test next door already trusts; here it is used as a
/// technique rather than as an oracle.
///
/// Only reached from an [`Decision::Undecidable`], so the path that already
/// works pays nothing. The retry re-enters at the top rather than inside
/// [`Orient2d::on_floats`] because all three levels fail together on these
/// inputs — a determinant of `1e-400` has subnormal products at level 1,
/// interval products that collapse to zero at level 2, and an expansion that
/// declines to run at level 3.
///
/// # Why `ORIENT2D_ERROR_BOUND` carries over untouched
///
/// Rounding to nearest commutes with multiplication by an exact power of two:
/// while nothing leaves the normal range, `fl(2^k * x) = 2^k * fl(x)`, because
/// the two numbers have the same mantissa and the grid they round onto is the
/// same grid scaled. So each difference in [`Orient2d::on_floats`] comes out as
/// `s` times what it was, each product as `s^2`, and `value` as `s^2` — the
/// same mantissa bits throughout. The permanent is a sum of those same
/// degree-two products, so it scales by `s^2` as well, and
/// `value > ORIENT2D_ERROR_BOUND * permanent` compares two sides multiplied by
/// the same positive number. Level 1 is therefore scale-invariant, which is why
/// the bound needs no adjusting and gets none.
///
/// That argument is about not *losing* certifications. Soundness needs less
/// than it: the scaled copy is an ordinary exact `f64` input, and
/// `ORIENT2D_ERROR_BOUND` is a relative bound that holds for any such input
/// whose products stay normal, so it holds here on its own terms.
fn decided_after_normalising(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Option<(Decision, Level)> {
    let exponent =
        exponent_that_brings_into_range([a[0] - c[0], a[1] - c[1], b[0] - c[0], b[1] - c[1]])?;
    if exponent == 0 {
        return None;
    }

    let a = scaled_by_a_power_of_two(a, exponent)?;
    let b = scaled_by_a_power_of_two(b, exponent)?;
    let c = scaled_by_a_power_of_two(c, exponent)?;

    // An enclosure carries the units of the value it encloses (decisions.md
    // §12.4), and this one would carry them scaled by `s^2`. A sign is
    // dimensionless and survives; a width does not, so only a certified answer
    // comes back from here.
    let (decision, level) = decide(&Orient2d { a, b, c });
    decision.is_certified().then_some((decision, level))
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

        // Alone among the predicates here, this one needs no check that the two
        // products stayed out of the subnormals — see the note on
        // [`ORIENT2D_ERROR_BOUND`].
        //
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
/// # Why this one needs no subnormal guard
///
/// Every other predicate in the cascade checks that its products stayed out of
/// the subnormals, because a bound of this shape is *relative* and rounding
/// down there is not (see [`product_is_normal`](crate::product_is_normal)).
/// This one does not need to, because its *sign* cannot go wrong even when the
/// bound does.
///
/// The whole value is `fl(fl(f) - fl(b))` for two exact products `f` and `b`.
/// Rounding to nearest never reorders two numbers: if `f > b` then
/// `fl(f) >= fl(b)`, subnormals and flushes to zero included. Subtracting two
/// `f64` cannot reorder them either, and gives exactly zero only when they are
/// equal. So the sign of the value is either the true sign or zero — never the
/// opposite sign. A wrong bound can therefore only make this filter certify a
/// sign that happens to be right, and a value of zero clears no bound at all,
/// since the bound is never negative.
///
/// That argument turns on there being exactly one subtraction at the end. It
/// does not survive a third term: `orient3d` adds three such differences after
/// scaling each by a different height, and one of those scalings can swamp
/// another, which is exactly how it used to certify wrong signs.
///
/// The bound itself still holds only while nothing overflows or falls into the
/// subnormals, and [`decide`] refuses a bound that is not a normal number.
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
    let mut total = [0.0f64; DETERMINANT_ENTRIES];
    match exact_determinant(a, b, c, &mut total) {
        Some(total_len) => expansion_sign(&total[..total_len]),
        None => Decision::Undecidable {
            enclosure: f64::INFINITY,
        },
    }
}

/// The determinant itself, written into `total`, and how many entries it took.
///
/// `None` when a product overflowed or underflowed, which is the case
/// [`orient2d_exactly`] has no honest sign for.
fn exact_determinant(
    a: [f64; 2],
    b: [f64; 2],
    c: [f64; 2],
    total: &mut [f64; DETERMINANT_ENTRIES],
) -> Option<usize> {
    let (Some((ab, ab_len)), Some((ac, ac_len)), Some((bc, bc_len))) =
        (cross(a, b), cross(a, c), cross(b, c))
    else {
        return None;
    };

    let mut negated_ac = ac;
    for entry in &mut negated_ac[..ac_len] {
        *entry = -*entry;
    }

    let mut without_bc = [0.0f64; 8];
    let without_bc_len = expansion_sum(&ab[..ab_len], &negated_ac[..ac_len], &mut without_bc);
    Some(expansion_sum(
        &without_bc[..without_bc_len],
        &bc[..bc_len],
        total,
    ))
}

/// Three cross products of four entries each.
const DETERMINANT_ENTRIES: usize = 12;

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
        DETERMINANT_ENTRIES, ORIENT2D, ORIENT2D_ERROR_BOUND, Orient2d, exact_determinant, orient2d,
        orient2d_exactly, orient2d_reporting_level, orient2d_tolerantly, scaled_by_a_power_of_two,
    };
    use crate::expansion::{expansion_sum, two_diff};
    use crate::{
        Decision, Filtered, Level, Policy, Predicate, ProvenanceLog, decide, product_is_normal,
    };

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
        let (a, b, c) = spread_wider_than_f64_holds_at_once();
        let mut log = ProvenanceLog::new();
        let decision = orient2d_tolerantly(a, b, c, &Policy::certified(), &mut log);
        assert!(matches!(decision, Decision::Undecidable { .. }));
        assert_eq!(log.len(), 1);
        assert_eq!(log.records()[0].predicate(), ORIENT2D);
        assert_eq!(log.records()[0].inputs().len(), 3);
        assert_eq!(log.records()[0].level(), Level::Interval);
    }

    /// Every product here overflows, and the turn is still an ordinary
    /// anticlockwise one. Bringing all six coordinates down by `2^-998` settles
    /// it, and settles it as the same shape at unit size does.
    #[test]
    fn coordinates_big_enough_to_overflow_the_products_are_normalised_and_decided() {
        assert_eq!(
            orient2d_reporting_level([1e300, 0.0], [0.0, 1e300], [-1e300, -1e300]),
            (Decision::Pos, Level::Float)
        );
        assert_eq!(
            orient2d_reporting_level([1.0, 0.0], [0.0, 1.0], [-1.0, -1.0]),
            (Decision::Pos, Level::Float),
            "the same three points at a size f64 has no trouble with"
        );
    }

    /// The true determinant here is `1e-400`, which is positive and far below
    /// the smallest `f64`. Both products underflow where they are, and both are
    /// perfectly ordinary once the whole triangle is lifted into range.
    #[test]
    fn coordinates_small_enough_to_underflow_the_products_are_normalised_and_decided() {
        assert_eq!(
            orient2d_reporting_level([1e-200, 0.0], [0.0, 1e-200], [0.0, 0.0]),
            (Decision::Pos, Level::Float)
        );
        assert_eq!(
            orient2d_reporting_level([1.0, 0.0], [0.0, 1.0], [0.0, 0.0]),
            (Decision::Pos, Level::Float),
            "the same three points at a size f64 has no trouble with"
        );
    }

    /// Three points whose coordinates run from `2^-600` to `2^1000` and whose
    /// determinant is decided by a difference of `2^-650`.
    ///
    /// This is the refusal that has to survive. No single power of two brings
    /// the products of the raw coordinates into range: lifting `2^-600 *
    /// 2^-600` above the smallest exact product needs a factor of at least
    /// `2^115`, and keeping `2^1000 * 2^1000` finite needs one of at most
    /// `2^-488`. The two cheap levels are no help either — the value is `2^350`
    /// against a permanent of `2^2001`, which no relative bound can separate
    /// from zero at any scale.
    ///
    /// So the answer exists in the reals and this technique cannot reach it,
    /// which is what makes it a limit rather than a gap
    /// (`docs/decisions.md` §16.9). Refusing is the whole of the right answer.
    #[test]
    fn coordinates_spread_wider_than_f64_holds_at_once_are_still_undecidable() {
        let (a, b, c) = spread_wider_than_f64_holds_at_once();
        match orient2d(a, b, c) {
            Decision::Pos => panic!("the true determinant is negative, so this is a wrong sign"),
            Decision::Neg | Decision::Zero => panic!("nothing here can prove a sign"),
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
        }
    }

    /// The paragraph above is an argument that no scaling helps, and
    /// `docs/decisions.md` §16.10 is about exactly how wrong those go
    /// unchecked. So it is executed: every exponent an `f64` could carry is
    /// tried, not just the one the predicate picks.
    ///
    /// The ones that scale exactly are the only ones worth trying — a copy that
    /// lost bits describes different geometry — and not one of them certifies
    /// anything.
    #[test]
    fn no_power_of_two_at_all_rescues_that_spread() {
        let (a, b, c) = spread_wider_than_f64_holds_at_once();
        let mut tried = 0;
        for exponent in -1100..=1100 {
            let (Some(a), Some(b), Some(c)) = (
                scaled_by_a_power_of_two(a, exponent),
                scaled_by_a_power_of_two(b, exponent),
                scaled_by_a_power_of_two(c, exponent),
            ) else {
                continue;
            };
            tried += 1;
            assert!(
                !orient2d(a, b, c).is_certified(),
                "2^{exponent} settled what the comment says nothing settles"
            );
        }
        assert!(tried > 100, "only {tried} exponents scaled exactly at all");
    }

    fn spread_wider_than_f64_holds_at_once() -> ([f64; 2], [f64; 2], [f64; 2]) {
        let near = 2f64.powi(-600);
        let apart = 2f64.powi(-650);
        let far = 2f64.powi(1000);
        ([near, near], [near, near + apart], [far, far])
    }

    /// The claim [`decided_after_normalising`] rests on, executed rather than
    /// argued: an exact power of two multiplies level 1's value and its error
    /// bound by the same `2^(2k)`, so the comparison between them comes out the
    /// same way and `ORIENT2D_ERROR_BOUND` needs no adjusting.
    #[test]
    fn scaling_moves_level_one_and_its_error_bound_by_exactly_the_same_factor() {
        let mut checked = 0;
        for exponent in [-40, -7, 1, 13, 40] {
            for (a, b, c) in awkward_triangles() {
                let here = Orient2d { a, b, c }
                    .on_floats()
                    .expect("nothing is subnormal");
                let moved = Orient2d {
                    a: scaled_by_a_power_of_two(a, exponent).expect("nothing leaves the range"),
                    b: scaled_by_a_power_of_two(b, exponent).expect("nothing leaves the range"),
                    c: scaled_by_a_power_of_two(c, exponent).expect("nothing leaves the range"),
                }
                .on_floats()
                .expect("nothing is subnormal");

                let factor = 2f64.powi(2 * exponent);
                assert_eq!(moved.value, here.value * factor, "{a:?} {b:?} {c:?}");
                assert_eq!(
                    moved.error_bound,
                    here.error_bound * factor,
                    "{a:?} {b:?} {c:?}"
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 20);
    }

    /// Coordinates with mantissas full enough that the products round, which is
    /// the only case where the claim above could come apart.
    fn awkward_triangles() -> [([f64; 2], [f64; 2], [f64; 2]); 4] {
        [
            ([0.0, 0.0], [1.0, 0.0], [0.3, 1.0]),
            ([0.5, 0.25], [3.0, -1.0], [-4.0, 7.0]),
            ([1.0, 1.0], [1.0, 1.0 + 2f64.powi(-50)], [4.0, 4.0]),
            (
                [1.0 / 3.0, 7.0 / 9.0],
                [2.0 / 7.0, 11.0 / 13.0],
                [-5.0 / 3.0, 1.0 / 11.0],
            ),
        ]
    }

    /// Three points, one of which is not a point, do not turn any particular
    /// way.
    ///
    /// Before the guard this came back [`Decision::Neg`] at
    /// [`Level::Interval`]: the not-a-number reaches level 2 as an enclosure of
    /// everything, `b`'s and `c`'s heights are equal so it is multiplied by
    /// zero, and the other half of the determinant certifies the sign on its
    /// own.
    #[test]
    fn a_corner_that_is_not_a_number_gets_no_answer() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for axis in 0..2 {
                let mut broken = [0.0, 1.0];
                broken[axis] = bad;
                assert_eq!(
                    orient2d_reporting_level(broken, [1.0, 0.0], [0.0, 0.0]),
                    (
                        Decision::Undecidable {
                            enclosure: f64::INFINITY
                        },
                        Level::Interval
                    ),
                    "{bad} on axis {axis}"
                );
            }
        }
    }

    /// Whichever of the three it is.
    #[test]
    fn any_of_the_three_corners_being_no_corner_stops_the_answer() {
        let good = [[0.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
        for spoiled in 0..3 {
            let mut corners = good;
            corners[spoiled] = [f64::NAN, 0.0];
            assert!(
                !orient2d(corners[0], corners[1], corners[2]).is_certified(),
                "corner {spoiled}"
            );
        }
        assert!(orient2d(good[0], good[1], good[2]).is_certified());
    }

    /// The tolerant entry point must not launder the refusal into a `Zero`.
    #[test]
    fn a_tolerant_policy_still_refuses_a_corner_that_is_not_a_corner() {
        let mut log = ProvenanceLog::new();
        let decision = orient2d_tolerantly(
            [f64::NAN, 1.0],
            [1.0, 0.0],
            [0.0, 0.0],
            &Policy::tolerant(1.0),
            &mut log,
        );
        assert!(!decision.is_certified());
        assert!(!log.is_empty(), "a refusal has to leave a record behind");
    }

    /// `ORIENT2D_ERROR_BOUND` promises that the two products together land
    /// within `3*epsilon` of the permanent from the true determinant. Here that
    /// promise is measured rather than argued, on triangles searched out for
    /// making the gap as wide as they can.
    ///
    /// **The worst found comes to 2.96 epsilon of the permanent**, against the
    /// 3 the constant carries, so this fails as soon as the constant is cut
    /// below about 2.96 epsilon (decisions.md §15.6, §16.3).
    ///
    /// What is measured is `forwards - backwards` *before* the last rounding,
    /// because that is the quantity the constant bounds and the whole of what
    /// makes the filter sound. The value level 1 hands back is one rounding
    /// further out and that rounding is deliberately not covered — see the test
    /// below, which is why asserting on `value` here would assert something
    /// false.
    #[test]
    fn the_two_products_stay_inside_the_error_bound_where_the_rounding_is_worst() {
        let mut worst = 0.0f64;
        for corners in triangles_whose_arithmetic_rounds_as_badly_as_it_can() {
            let (products_error, _, filtered) = how_far_level_one_falls_from_the_truth(corners)
                .expect("no product here leaves the range");
            assert!(
                products_error.abs() <= filtered.error_bound,
                "the products came out {products_error:e} from the truth on {corners:?}, \
                 past the {:e} promised",
                filtered.error_bound
            );
            let permanent = filtered.error_bound / ORIENT2D_ERROR_BOUND;
            worst = worst.max(products_error.abs() / permanent / (f64::EPSILON / 2.0));
        }
        assert!(
            worst > 2.95,
            "these used to reach 2.96 epsilon of the permanent and now reach only {worst}, \
             so the net has gone slack and the search wants running again"
        );
    }

    /// The note on [`ORIENT2D_ERROR_BOUND`] argues that the final subtraction
    /// needs no epsilon of its own, because its error is a fraction of the
    /// result rather than of the permanent and can only shrink a value that
    /// already cleared the bound. That is a claim justifying an absence, so it
    /// gets a check of its own (decisions.md §16.10).
    ///
    /// On this triangle `value` really does end up 3.36 epsilon of the
    /// permanent from the truth — past the 3 the constant carries — while the
    /// two products it was built from stay inside. So the constant is not a
    /// bound on `value`, exactly as the note says, and level 1 certifies the
    /// right sign from it regardless, which is what the note says makes that
    /// safe.
    #[test]
    fn the_last_rounding_falls_outside_the_bound_and_the_sign_survives_it_anyway() {
        let corners = [
            [1.3214740756387775, -1.2097128315383816],
            [-1.9135572424091511, 1.279523460954989],
            [1.1972513460202028, 1.4140834091865018],
        ];
        let (products_error, value_error, filtered) =
            how_far_level_one_falls_from_the_truth(corners).expect("no product leaves the range");

        assert!(products_error.abs() <= filtered.error_bound);
        assert!(
            value_error.abs() > filtered.error_bound,
            "this triangle is only worth keeping while the last rounding takes the value \
             outside the constant, and it no longer does"
        );

        let [a, b, c] = corners;
        assert_eq!(
            orient2d_reporting_level(a, b, c),
            (Decision::Neg, Level::Float),
            "level 1 has to be what certifies this, and it has to get the sign right"
        );
    }

    /// Level 1 has to be what answers these, or the measurement above is of an
    /// expression nothing uses.
    ///
    /// [`decided_after_normalising`] runs only once [`decide`] has refused, and
    /// [`decide`] certifies every one of these at [`Level::Float`] — so no
    /// scaled copy is ever formed and no retry can be quietly supplying the
    /// answer (decisions.md §16.9).
    #[test]
    fn the_worst_rounding_triangles_are_settled_by_level_one_itself() {
        for corners in triangles_whose_arithmetic_rounds_as_badly_as_it_can() {
            let [a, b, c] = corners;
            let (decision, level) = decide(&Orient2d { a, b, c });
            assert!(
                decision.is_certified(),
                "{corners:?} was not settled at all"
            );
            assert_eq!(level, Level::Float, "{corners:?}");
        }
    }

    /// How far the two products together fall from the true determinant, how
    /// far `value` falls once the final subtraction has rounded as well, and
    /// the bound the constant promises for the first of those.
    ///
    /// The truth comes from [`exact_determinant`], which is level 3's own
    /// arithmetic and throws nothing away. [`two_diff`] hands back what the
    /// last subtraction lost, so adding that in recovers `forwards - backwards`
    /// exactly and the two errors come out of the same subtraction.
    ///
    /// `None` when a product left the range, where a relative bound claims
    /// nothing at all.
    fn how_far_level_one_falls_from_the_truth(
        corners: [[f64; 2]; 3],
    ) -> Option<(f64, f64, Filtered)> {
        let [a, b, c] = corners;
        let (acx, acy) = (a[0] - c[0], a[1] - c[1]);
        let (bcx, bcy) = (b[0] - c[0], b[1] - c[1]);
        if !product_is_normal(acx, bcy) || !product_is_normal(acy, bcx) {
            return None;
        }
        let filtered = Orient2d { a, b, c }.on_floats()?;
        let (value, subtraction_roundoff) = two_diff(acx * bcy, acy * bcx);
        assert_eq!(value, filtered.value, "the same expression, spelled twice");

        let mut determinant = [0.0f64; DETERMINANT_ENTRIES];
        let length = exact_determinant(a, b, c, &mut determinant)?;

        let mut value_error = [0.0f64; DETERMINANT_ENTRIES + 1];
        let value_error_length = expansion_sum(&determinant[..length], &[-value], &mut value_error);

        let mut products_error = [0.0f64; DETERMINANT_ENTRIES + 2];
        let products_error_length = expansion_sum(
            &determinant[..length],
            &[-subtraction_roundoff, -value],
            &mut products_error,
        );

        Some((
            products_error[..products_error_length].iter().sum(),
            value_error[..value_error_length].iter().sum(),
            filtered,
        ))
    }

    /// The four the search kept coming back to, worst first. Their products
    /// fall 2.96, 2.45, 2.40 and 2.39 epsilon of the permanent from the truth.
    ///
    /// They are kept as literals because that is what makes this a test rather
    /// than a second search: the coordinates are the finding, and re-deriving
    /// them on every run would cost minutes and prove nothing.
    fn triangles_whose_arithmetic_rounds_as_badly_as_it_can() -> [[[f64; 2]; 3]; 4] {
        [
            [
                [0.18202136112233935, -16.509565117185108],
                [-0.2664687296518431, -0.25278305332298173],
                [-4.289950494568721, -0.360491885299977],
            ],
            [
                [8.858639129305297, 1.698983312928278],
                [0.09737340948185011, 0.09945658874300996],
                [0.09992273072931024, -0.1299712859920983],
            ],
            [
                [1.919499316075261, 1.5232665278705688],
                [7.718968216943509, -2.5003140165964743],
                [-0.3354606432058729, -1.0544476559196],
            ],
            [
                [59.21239135734716, -79.46432295359523],
                [-0.32675796193138495, 10.834848102579858],
                [71.01141229525142, -0.019631577874157813],
            ],
        ]
    }
}
