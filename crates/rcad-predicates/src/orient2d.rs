use crate::expansion::{expansion_sign, expansion_sum, product_is_exact, two_product};
use crate::power_of_two::{
    each_once_and_never_zero, exponent_that_brings_into_range,
    exponent_that_lifts_every_product_to, exponent_that_lifts_the_largest_to,
    scaled_by_a_power_of_two,
};
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
/// ([`decided_after_normalising`]). So are points near the origin whose
/// differences reach from ordinary sizes down to `1e-300`, lifted instead as
/// high as the top of the range allows, and a spike at the origin under a unit
/// square, down to a spike the smallest `f64` wide. And the exact level keeps a
/// product that falls off the bottom, so a turn is decided wherever the
/// products that stayed in range outweigh what the rest lost
/// ([`sign_clear_of_what_fell_off`]).
///
/// What is left as [`Decision::Undecidable`] is a determinant that, at every
/// scale that moves the input exactly, is either within what the products
/// falling off the bottom may have lost or has a product running off the top.
/// The cut corner `(1, -t)`, `(0.5, 0.5)`, `(t, 1)` with `t` a subnormal is
/// one: its answer is `-t^2`, and the `1 * 1` in the same sum caps the
/// scaling well before `t^2` could show. Reaching it needs a wider exponent
/// than `f64` has, which is the case decisions.md §3.5's dynamic-range check is
/// for (§358).
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

/// Up to three more runs of the whole cascade, on a copy of the input moved
/// into range by an exact power of two. `None` when there is no such copy, or
/// when no copy could be settled either.
///
/// This determinant is homogeneous of degree two in the coordinates, so scaling
/// all six by `s > 0` multiplies it by `s^2` and leaves its sign alone — and a
/// determinant of exactly zero stays exactly zero. That is what the metamorphic
/// test next door already trusts; here it is used as a technique rather than as
/// an oracle.
///
/// The first two exponents are `orient3d`'s, for the same reasons
/// (`docs/decisions.md` §324): the largest difference into `[1, 2)` first, and
/// then the largest coordinate into [`HIGHEST_BINADE_LEVEL_THREE_TAKES`], for
/// three points near the origin whose differences reach down to `1e-300` —
/// where the first exponent is zero and a determinant that is exactly zero
/// needs every product above [`product_is_exact`]'s floor. The third lifts
/// the products level 3 forms as high as they go, for a turn the second leaves
/// with nothing but products under that floor; [`exponents_to_try`] says why.
///
/// Only reached from an [`Decision::Undecidable`], so the path that already
/// works pays nothing. The retry re-enters at the top rather than inside
/// [`Orient2d::on_floats`] because all three levels fail together on these
/// inputs — a determinant of `1e-400` has subnormal products at level 1,
/// interval products that collapse to zero at level 2, and at level 3 a held
/// sum of zero with a product that fell off the bottom.
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
    each_once_and_never_zero(exponents_to_try(a, b, c))
        .find_map(|exponent| decided_at_scale(a, b, c, exponent))
}

/// The exponents [`decided_after_normalising`] tries, in the order it tries
/// them.
///
/// The third is for a determinant that holds `tiny * tiny` and `tiny * 1` and
/// never `1 * 1` (`docs/decisions.md` §324.7). The second lifts the largest
/// coordinate only as far as `1 * 1` could go, which leaves `tiny * tiny`
/// under [`product_is_exact`]'s floor below `tiny = 2^-994`. Since §358 level 3
/// keeps such a product and decides whenever the `tiny * 1` terms outweigh
/// what it lost, which covers every turn of a spike at the origin under a unit
/// square. What is left for the third is a turn whose `tiny * 1` terms cancel,
/// such as `(1, 0)`, `(0, t)`, `(t, t)`. The third reads the six products level
/// 3 forms one by one and lifts until the one closest to the top reaches
/// [`LARGEST_SUM_OF_FACTOR_BINADES`]. It never lands below the second, because
/// no product is larger than the largest coordinate squared. It comes last so
/// that nothing the first two settle can move.
///
/// Levels 1 and 2 multiply differences, and at the third exponent those can
/// overflow. That costs them their answer and nothing else: an infinite level 1
/// bound is not normal and [`decide`] skips it, and an interval that overflows
/// widens to take in infinity.
/// `levels_one_and_two_overflowing_at_the_third_exponent_still_agree_with_level_three`
/// runs that.
fn exponents_to_try(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> [Option<i32>; 3] {
    let coordinates = [a, b, c].into_iter().flatten();
    [
        exponent_that_brings_into_range([a[0] - c[0], a[1] - c[1], b[0] - c[0], b[1] - c[1]]),
        exponent_that_lifts_the_largest_to(coordinates.clone(), HIGHEST_BINADE_LEVEL_THREE_TAKES),
        exponent_that_lifts_every_product_to(
            coordinates,
            [
                [a[0], b[1]],
                [a[1], b[0]],
                [a[0], c[1]],
                [a[1], c[0]],
                [b[0], c[1]],
                [b[1], c[0]],
            ],
            LARGEST_SUM_OF_FACTOR_BINADES,
        ),
    ]
}

/// How high the binades of a product's two factors may add up to at the third
/// exponent, for level 3 to overflow nothing.
///
/// Each factor is then below `2^(binade + 1)`, so each of the six products
/// [`exact_determinant`] forms is below `2^1022`. Measured rather than argued:
/// with every x at the top of one binade and every y at the top of another,
/// which makes all six products as large as this allows at once, none of the
/// 64 ways to sign them overflows at `1020`, and 24 do at `1021`.
/// `the_highest_sum_of_factor_binades_overflows_nothing_in_level_three` runs it.
///
/// Like [`HIGHEST_BINADE_LEVEL_THREE_TAKES`], this decides reach and not
/// soundness: an overflowed product is refused by [`cross`], and an overflowed
/// sum by [`expansion_sign`].
const LARGEST_SUM_OF_FACTOR_BINADES: i32 = 1020;

/// The binade [`decided_after_normalising`]'s second exponent lifts the largest
/// coordinate into: `[2^509, 2^510)`, the highest in which no level of the
/// cascade can overflow.
///
/// Every coordinate is then below `2^510`. Level 3 forms three cross products
/// of coordinates, each below `2^1021`, summing to below `3 * 2^1021 < 2^1023`.
/// Levels 1 and 2 work on differences, which are at most `2^511`, so each of
/// the two products is at most `2^1022` and the permanent at most `2^1023`.
/// One binade higher and the permanent could pass `2^1024`.
/// `the_highest_binade_the_retry_lifts_into_overflows_nothing` runs it.
///
/// Going past it would only cost reach, never soundness — an overflowed level 1
/// bound is refused by [`decide`] and an overflowed product by [`cross`] — so
/// the one thing this number decides is how far the retry gets.
const HIGHEST_BINADE_LEVEL_THREE_TAKES: i32 = 509;

/// [`decide`] on the three points scaled by `2^exponent`, if that is exact and
/// the answer is certified.
fn decided_at_scale(
    a: [f64; 2],
    b: [f64; 2],
    c: [f64; 2],
    exponent: i32,
) -> Option<(Decision, Level)> {
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
            // A product overflowed, or what fell off the bottom could be
            // worth more than what was held. Either way no sign is proved.
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
/// Comes back [`Decision::Undecidable`] when a product overflowed, because then
/// there is nothing honest to report, and when products that fell off the
/// bottom of the range lost more than the rest can outweigh
/// ([`sign_clear_of_what_fell_off`]).
fn orient2d_exactly(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Decision {
    let mut held = [0.0f64; DETERMINANT_ENTRIES];
    match exact_determinant(a, b, c, &mut held) {
        Some((held_len, fell_off)) => sign_clear_of_what_fell_off(&held[..held_len], fell_off),
        None => Decision::Undecidable {
            enclosure: f64::INFINITY,
        },
    }
}

/// The sign of the true determinant, given `held`, which misses it by less
/// than one smallest subnormal for each of the `fell_off` products that failed
/// [`product_is_exact`].
///
/// Why less than one: [`two_product`]'s high half is the product rounded, and
/// the fused multiply-add that makes its low half rounds only when that low
/// half lands among the subnormals, onto a grid `2^-1074` apart — so the pair
/// misses by at most `2^-1075`. That covers a product flushed all the way to
/// zero too, whose low half is then the product rounded onto the same grid.
/// [`expansion_sum`] loses nothing, subnormal entries included, so the misses
/// only add up. `rcad-geom`'s `Exact::times_holding_on` rests on the same
/// bound (`docs/decisions.md` §322.3), and
/// `the_box_round_what_level_three_held_holds_the_true_determinant` checks it
/// against the same triangles lifted to where nothing falls off.
///
/// So the true value is inside `held ± fell_off * 2^-1074`, and a sign that
/// holds at both ends of that box is the sign. [`Decision::Zero`] needs every
/// product exact, since a box of any width round zero holds both signs.
fn sign_clear_of_what_fell_off(held: &[f64], fell_off: u32) -> Decision {
    if fell_off == 0 {
        return expansion_sign(held);
    }
    let lost = f64::from(fell_off) * SMALLEST_SUBNORMAL;
    let [low, high] = [-lost, lost].map(|end| {
        let mut moved = [0.0f64; DETERMINANT_ENTRIES + 1];
        let len = expansion_sum(held, &[end], &mut moved);
        expansion_sign(&moved[..len])
    });
    // The two ends are `2 * lost` apart, exactly, so they are never both zero:
    // two ends that agree on a certified sign agree on `Pos` or on `Neg`.
    if low == high && low.is_certified() {
        low
    } else {
        Decision::Undecidable {
            enclosure: f64::INFINITY,
        }
    }
}

/// `2^-1074`, the grid the subnormals sit on.
const SMALLEST_SUBNORMAL: f64 = f64::from_bits(1);

/// The determinant itself, written into `total`, how many entries it took, and
/// how many of its six products failed [`product_is_exact`] on the way.
///
/// `None` when a product overflowed, which is the case [`orient2d_exactly`] has
/// no honest sign for. A product that fell off the bottom is kept, and
/// [`sign_clear_of_what_fell_off`] says what that costs.
fn exact_determinant(
    a: [f64; 2],
    b: [f64; 2],
    c: [f64; 2],
    total: &mut [f64; DETERMINANT_ENTRIES],
) -> Option<(usize, u32)> {
    let (Some((ab, ab_len, ab_fell)), Some((ac, ac_len, ac_fell)), Some((bc, bc_len, bc_fell))) =
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
    let total_len = expansion_sum(&without_bc[..without_bc_len], &bc[..bc_len], total);
    Some((total_len, ab_fell + ac_fell + bc_fell))
}

/// Three cross products of four entries each.
const DETERMINANT_ENTRIES: usize = 12;

/// `u.x * v.y - u.y * v.x`, how many entries it took, and how many of its two
/// products failed [`product_is_exact`] — or `None` if a product overflowed.
fn cross(u: [f64; 2], v: [f64; 2]) -> Option<([f64; 4], usize, u32)> {
    let factors = [(u[0], v[1]), (u[1], v[0])];
    if factors.iter().any(|&(x, y)| !(x * y).is_finite()) {
        return None;
    }
    let fell_off = factors
        .iter()
        .map(|&(x, y)| u32::from(!product_is_exact(x, y)))
        .sum();
    let [
        (forwards, forwards_roundoff),
        (backwards, backwards_roundoff),
    ] = factors.map(|(x, y)| two_product(x, y));
    let mut out = [0.0; 4];
    let len = expansion_sum(
        &[forwards_roundoff, forwards],
        &[-backwards_roundoff, -backwards],
        &mut out,
    );
    Some((out, len, fell_off))
}

#[cfg(test)]
mod tests {
    use super::{
        DETERMINANT_ENTRIES, HIGHEST_BINADE_LEVEL_THREE_TAKES, LARGEST_SUM_OF_FACTOR_BINADES,
        ORIENT2D, ORIENT2D_ERROR_BOUND, Orient2d, SMALLEST_SUBNORMAL, decided_at_scale,
        each_once_and_never_zero, exact_determinant, exponents_to_try, orient2d, orient2d_exactly,
        orient2d_reporting_level, orient2d_tolerantly, scaled_by_a_power_of_two,
    };
    use crate::expansion::{expansion_sign, expansion_sum, two_diff, two_sum};
    use crate::{
        Decision, Filtered, Level, Policy, Predicate, ProvenanceLog, decide, product_is_normal,
    };
    use core::cmp::Ordering;

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

    /// The turn in the middle of the cut corner `when_it_cannot_tell.rs`
    /// pinned as undecidable until `docs/decisions.md` §358.
    ///
    /// The determinant is `-t (1 - t)`, about `-1e-305`: an ordinary normal
    /// number, and negative, so the turn is clockwise. Level 3 used to decline
    /// only because one of its six products, `t * t`, falls below the range.
    /// That product is worth `1e-610`, and what it loses is less than the
    /// smallest subnormal, so the other five settle the sign on their own.
    #[test]
    fn a_cut_corner_bent_by_1e_305_turns_clockwise() {
        let t = 1e-305;
        assert_eq!(
            orient2d_reporting_level([1.0, t], [0.5, 0.5], [t, 1.0]),
            (Decision::Neg, Level::Exact)
        );
    }

    /// The box [`sign_clear_of_what_fell_off`] draws round what level 3 held,
    /// run rather than argued.
    ///
    /// Every coordinate is zero or sits between `2^-612` and `2^-400`, so the
    /// products level 3 forms run from `2^-1224` to `2^-800` and many fall off
    /// the bottom. The same triangle lifted by `2^600` is exact, and there nothing
    /// falls off: its determinant is exactly `2^1200` times the true one, and
    /// that is the truth here. The held sum and the box round it, lifted the
    /// same way — exact, since lifting only moves exponents up — have to
    /// contain it, and every sign read at the bottom has to be the truth's.
    ///
    /// Half the mantissas are small whole numbers, so that some determinants
    /// cancel down to nearly nothing and the box is tested where it is tight.
    #[test]
    fn the_box_round_what_level_three_held_holds_the_true_determinant() {
        const LIFT: i32 = 600;
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut roll = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let mut coordinate = move || {
            let bits = roll();
            let mantissa = if bits & 2 == 0 {
                (bits >> 11) as f64
            } else {
                ((bits >> 11) % 8) as f64
            };
            let exponent = -612 + ((bits >> 2) % 160) as i32;
            let sign = if bits & 1 == 0 { 1.0 } else { -1.0 };
            sign * mantissa * 2f64.powi(exponent)
        };

        let (mut something_fell_off, mut decided_all_the_same) = (0, 0);
        for _ in 0..20_000 {
            let corners: [[f64; 2]; 3] = [(); 3].map(|()| [coordinate(), coordinate()]);
            let [a, b, c] = corners;

            let mut held = [0.0; DETERMINANT_ENTRIES];
            let (held_len, fell_off) =
                exact_determinant(a, b, c, &mut held).expect("nothing here can overflow");
            let [la, lb, lc] = corners.map(|corner| {
                scaled_by_a_power_of_two(corner, LIFT).expect("every corner lifts exactly")
            });
            let mut truth = [0.0; DETERMINANT_ENTRIES];
            let Some((truth_len, 0)) = exact_determinant(la, lb, lc, &mut truth) else {
                panic!("lifted by 2^{LIFT}, nothing falls off: {corners:?}");
            };

            let lifted = |value: f64| value * 2f64.powi(LIFT) * 2f64.powi(LIFT);
            let held_lifted: Vec<f64> = held[..held_len].iter().map(|&v| lifted(v)).collect();
            let lost = lifted(f64::from(fell_off) * SMALLEST_SUBNORMAL);
            let truth_minus_held = subtracted(&truth[..truth_len], &held_lifted);
            let sign_of_truth_minus_held_plus = |extra: f64| {
                let mut out = vec![0.0; truth_minus_held.len() + 1];
                let len = expansion_sum(&truth_minus_held, &[extra], &mut out);
                expansion_sign(&out[..len])
            };
            assert!(
                matches!(
                    sign_of_truth_minus_held_plus(lost),
                    Decision::Pos | Decision::Zero
                ) && matches!(
                    sign_of_truth_minus_held_plus(-lost),
                    Decision::Neg | Decision::Zero
                ),
                "the truth is outside the box on {corners:?}"
            );

            let at_the_bottom = orient2d_exactly(a, b, c);
            if at_the_bottom.is_certified() {
                assert_eq!(
                    at_the_bottom,
                    expansion_sign(&truth[..truth_len]),
                    "{corners:?}"
                );
            }
            if fell_off > 0 {
                something_fell_off += 1;
                if at_the_bottom.is_certified() {
                    decided_all_the_same += 1;
                }
            }
        }
        assert!(
            something_fell_off > 10_000 && decided_all_the_same > 5_000,
            "{something_fell_off} triangles dropped a product and {decided_all_the_same} of \
             them were decided, so the box is not being tested where it matters"
        );
    }

    /// `left - right`, exactly.
    fn subtracted(left: &[f64], right: &[f64]) -> Vec<f64> {
        let negated: Vec<f64> = right.iter().map(|entry| -entry).collect();
        let mut out = vec![0.0; left.len() + negated.len()];
        let len = expansion_sum(left, &negated, &mut out);
        out.truncate(len);
        out
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
        let (a, b, c) = a_turn_smaller_than_the_bottom_of_the_range_loses();
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

    /// Three points whose coordinates run from `2^-600` to `2^1000`, which
    /// this file used to keep as the refusal that had to survive: no power of
    /// two lifts `2^-600 * 2^-600` above [`product_is_exact`]'s floor without
    /// taking `2^1000` off the top.
    ///
    /// That was a gap (`docs/decisions.md` §358). Level 3 never multiplies
    /// `2^1000` by itself here. Its determinant is `2^-1250 - 2^350`: the two
    /// products `2^-600 * 2^-600` fall off the bottom and lose less than two
    /// smallest subnormals between them, and the `2^400` products cancel
    /// exactly, leaving `-2^350` to settle the sign where it stands.
    ///
    /// [`product_is_exact`]: crate::expansion::product_is_exact
    #[test]
    fn a_spread_whose_far_products_cancel_is_decided_by_what_level_three_holds() {
        let near = 2f64.powi(-600);
        let apart = 2f64.powi(-650);
        let far = 2f64.powi(1000);
        assert_eq!(
            orient2d_reporting_level([near, near], [near, near + apart], [far, far]),
            (Decision::Neg, Level::Exact)
        );
    }

    /// A turn that is still refused, and has to be: the middle corner of a
    /// cut `(1, -t)`, `(0.5, 0.5)`, `(t, 1)` with `t = 1e-318`, a subnormal.
    ///
    /// Its determinant is exactly `-t^2`, so it turns clockwise. Level 3 forms
    /// `1 * 1` and `t * t` in the one sum, and multiplying every coordinate by
    /// `2^k` multiplies every product by `2^2k`. `1 * 1` runs off the top past
    /// `k = 511`, and at `k = 511` the whole answer is `2^1022 * t^2`, about
    /// `4.5e-329` — below half the smallest subnormal, which is the least one
    /// product falling off the bottom may lose. So at every scale the answer
    /// sits inside what the arithmetic cannot account for.
    ///
    /// The answer exists and no `f64` scaling reaches it; what would is a wider
    /// exponent, which is a limit of `f64` itself and the territory of §3.5's
    /// dynamic-range check (`docs/decisions.md` §358).
    #[test]
    fn a_turn_smaller_than_the_bottom_of_the_range_loses_is_still_undecidable() {
        let (a, b, c) = a_turn_smaller_than_the_bottom_of_the_range_loses();
        match orient2d(a, b, c) {
            Decision::Pos => panic!("the true determinant is -t^2, so this is a wrong sign"),
            Decision::Neg | Decision::Zero => panic!("nothing here can prove a sign"),
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
        }
    }

    /// The paragraph above is an argument that no scaling helps, and
    /// `docs/decisions.md` §16.10 is about exactly how wrong those go
    /// unchecked. So it is executed: every exponent an `f64` could carry is
    /// tried, not just the ones the predicate picks.
    ///
    /// The ones that scale exactly are the only ones worth trying — a copy that
    /// lost bits describes different geometry — and not one of them certifies
    /// anything. `t` is a subnormal, so exactness is asked by moving back and
    /// comparing rather than by asking for a normal number, which would leave
    /// out the input itself.
    #[test]
    fn no_power_of_two_at_all_rescues_that_turn() {
        let (a, b, c) = a_turn_smaller_than_the_bottom_of_the_range_loses();
        let mut tried = 0;
        for exponent in -1100..=1100 {
            let (Some(a), Some(b), Some(c)) = (
                moved_exactly(a, exponent),
                moved_exactly(b, exponent),
                moved_exactly(c, exponent),
            ) else {
                continue;
            };
            tried += 1;
            assert!(
                !orient2d(a, b, c).is_certified(),
                "2^{exponent} settled what the comment says nothing settles"
            );
        }
        assert_eq!(
            tried, 1025,
            "every exponent from -1 to 1023 moves it exactly"
        );
    }

    /// The same turn with x scaled by `2^i` and y by `2^j`, every pair of
    /// exponents from `-1100` to `1100`, which is the first of the two families
    /// `docs/decisions.md` §358.4 used to argue without running. Every product
    /// level 3 forms is one x times one y, so each moves by `2^(i + j)`, the
    /// answer and `1 * 1` together; an odd `i + j` is all this adds to the walk
    /// above. At `2^1023`, the most `1 * 1` allows, the answer is about
    /// `9e-329`, still below half the smallest subnormal.
    ///
    /// Each axis holds the same three sizes, `1`, `0.5` and `t`, so each moves
    /// exactly at the walk above's 1025 exponents, `-1` to `1023`: `1` runs off
    /// the top past `2^1023`, and `t = 101201 * 2^-1073` loses its last bit
    /// below `2^-1`. Neither axis limits the other, so `1025 * 1025` pairs move
    /// all three points exactly.
    #[test]
    fn no_power_of_two_per_axis_rescues_that_turn() {
        let (a, b, c) = a_turn_smaller_than_the_bottom_of_the_range_loses();
        let mut tried = 0;
        for i in -1100..=1100 {
            for j in -1100..=1100 {
                let (Some(a), Some(b), Some(c)) = (
                    moved_exactly_per_axis(a, [i, j]),
                    moved_exactly_per_axis(b, [i, j]),
                    moved_exactly_per_axis(c, [i, j]),
                ) else {
                    continue;
                };
                tried += 1;
                assert!(
                    !orient2d(a, b, c).is_certified(),
                    "x by 2^{i} and y by 2^{j} settled what the comment says nothing settles"
                );
            }
        }
        assert_eq!(
            tried,
            1025 * 1025,
            "each axis moves exactly from 2^-1 to 2^1023, whatever the other does"
        );
    }

    /// The same turn with every point moved by `(dx, dy)`, which keeps the
    /// determinant: the second family §358.4 used to argue without running.
    /// Each axis holds both `1` and `t`, and only `dx = 0` keeps `1 + dx` and
    /// `t + dx` both exact.
    ///
    /// The family is every `f64`, which no test walks, so these moves stand for
    /// it. `1 + dx` is exact only when `dx` is `0` or has no bit below
    /// `2^-53`, and then `t + dx` would need its 53 bits to reach from `2^-1073`,
    /// `t`'s lowest bit, up to about `dx` — unless the sum cancels, which needs
    /// `dx` close to minus a coordinate. So the moves tried put the lowest bit
    /// of `dx` at every place an `f64` has one, with every power of two and its
    /// negative, the ulps of `1` and of `t` among them; and they try every
    /// cancellation, one ulp either side of minus each size the turn holds.
    ///
    /// Exactness is asked of the rounding itself, and not by moving back as
    /// [`moved_exactly`] does: `1 + 2^-1074` rounds to `1`, and taking `2^-1074`
    /// away again gives `1` back, so moving back would call a move exact that
    /// shifted `t` and left `1` where it was.
    #[test]
    fn only_moving_that_turn_by_nothing_keeps_it_exact() {
        let (a, b, c) = a_turn_smaller_than_the_bottom_of_the_range_loses();
        let t = c[0];
        let ulp_of_t = SMALLEST_SUBNORMAL;
        assert_eq!((1.0 + ulp_of_t) - ulp_of_t, 1.0);
        assert_eq!(translated_exactly(1.0, ulp_of_t), None);

        let moves = moves_to_try([1.0, 0.5, t]);
        let exact_along = |axis: usize| -> Vec<f64> {
            moves
                .iter()
                .copied()
                .filter(|&by| {
                    [a, b, c]
                        .iter()
                        .all(|point| translated_exactly(point[axis], by).is_some())
                })
                .collect()
        };
        let (dxs, dys) = (exact_along(0), exact_along(1));

        let mut tried = 0;
        for &dx in &dxs {
            for &dy in &dys {
                tried += 1;
                let [a, b, c] = [a, b, c].map(|point| [point[0] + dx, point[1] + dy]);
                assert!(
                    !orient2d(a, b, c).is_certified(),
                    "moving by ({dx:e}, {dy:e}) settled what the comment says nothing settles"
                );
            }
        }
        assert_eq!(
            (dxs.as_slice(), dys.as_slice()),
            ([0.0].as_slice(), [0.0].as_slice()),
            "the moves that keep every x exact, and every y"
        );
        assert_eq!(tried, 1);
        for size in [1.0, t] {
            assert!(
                moves
                    .iter()
                    .any(|&by| by != 0.0 && translated_exactly(size, by).is_some()),
                "{size:e} alone has exact moves, so the check refuses something real"
            );
        }
    }

    fn a_turn_smaller_than_the_bottom_of_the_range_loses() -> ([f64; 2], [f64; 2], [f64; 2]) {
        let t = 1e-318f64;
        assert!(t > 0.0 && !t.is_normal());
        ([1.0, -t], [0.5, 0.5], [t, 1.0])
    }

    /// `point` multiplied by `2^exponent`, or `None` when that is not exact —
    /// which is when moving it back does not give `point` again.
    fn moved_exactly(point: [f64; 2], exponent: i32) -> Option<[f64; 2]> {
        moved_exactly_per_axis(point, [exponent, exponent])
    }

    /// `point` with x multiplied by `2^exponents[0]` and y by `2^exponents[1]`,
    /// or `None` when that is not exact.
    ///
    /// Moving back answers that soundly here, though not for a sum: a move that
    /// multiplies by `2^k` with `k > 0` rounds only by overflowing, and the
    /// way back from infinity is infinity; with `k < 0` it is the way back that
    /// is exact, so getting `point` again means the move was.
    fn moved_exactly_per_axis(point: [f64; 2], exponents: [i32; 2]) -> Option<[f64; 2]> {
        let moved = [0, 1].map(|axis| times_two_to_the(point[axis], exponents[axis]));
        let back = [0, 1].map(|axis| times_two_to_the(moved[axis], -exponents[axis]));
        (back == point).then_some(moved)
    }

    /// `value * 2^exponent`, in two halves so that neither power of two leaves
    /// the range on its own.
    fn times_two_to_the(value: f64, exponent: i32) -> f64 {
        let half = exponent / 2;
        value * 2f64.powi(half) * 2f64.powi(exponent - half)
    }

    /// Nothing, every power of two from the smallest subnormal to the largest
    /// and its negative, and minus each of `sizes` with the values one ulp
    /// either side of it, and their negatives.
    fn moves_to_try(sizes: [f64; 3]) -> Vec<f64> {
        let powers = (-1074..=1023).map(|exponent| times_two_to_the(1.0, exponent));
        let near_each_size = sizes
            .into_iter()
            .flat_map(|size| [size, -size])
            .flat_map(|size| [size.next_down(), size, size.next_up()]);
        core::iter::once(0.0)
            .chain(powers.flat_map(|power| [power, -power]))
            .chain(near_each_size)
            .collect()
    }

    /// `value + by`, or `None` when that rounds.
    fn translated_exactly(value: f64, by: f64) -> Option<f64> {
        let (sum, lost) = two_sum(value, by);
        (sum.is_finite() && lost == 0.0).then_some(sum)
    }

    /// `HIGHEST_BINADE_LEVEL_THREE_TAKES`'s arithmetic, run rather than argued:
    /// every coordinate at the top of that binade, under every one of the 64
    /// ways to sign them, and neither level 1 nor level 3 overflows. One binade
    /// higher, some of them do, so the constant is the highest that holds.
    #[test]
    fn the_highest_binade_the_retry_lifts_into_overflows_nothing() {
        assert_eq!(
            sign_patterns_that_overflow(HIGHEST_BINADE_LEVEL_THREE_TAKES),
            0
        );
        assert!(sign_patterns_that_overflow(HIGHEST_BINADE_LEVEL_THREE_TAKES + 1) > 0);
    }

    /// Of the 64 ways to sign six coordinates all at the top of `binade`, how
    /// many overflow level 1 or level 3.
    fn sign_patterns_that_overflow(binade: i32) -> usize {
        let top = 2f64.powi(binade + 1).next_down();
        (0..1u32 << 6)
            .filter(|signs| {
                let coordinate = |bit: u32| if signs & (1 << bit) == 0 { top } else { -top };
                let point = |first: u32| [coordinate(first), coordinate(first + 1)];
                let [a, b, c] = [point(0), point(2), point(4)];
                let level_one = Orient2d { a, b, c }
                    .on_floats()
                    .expect("level 1 always runs here");
                let mut total = [0.0; DETERMINANT_ENTRIES];
                let level_three = exact_determinant(a, b, c, &mut total);
                !level_one.value.is_finite()
                    || !level_one.error_bound.is_finite()
                    || level_three
                        .is_none_or(|(length, _)| !total[..length].iter().all(|e| e.is_finite()))
            })
            .count()
    }

    /// A spike at the origin under a unit square is decided on every turn it
    /// has, with the sign whole-number arithmetic gives, from a spike `2^-995`
    /// wide down to one the smallest `f64` wide (`docs/decisions.md` §324.7).
    ///
    /// Eight turns — the two tiny corners that are not the origin, in either
    /// order, with each corner of the square — have determinants that hold
    /// `tiny * tiny`, which the second exponent leaves under level 3's floor.
    /// They used to need the third exponent. Since `docs/decisions.md` §358
    /// level 3 keeps a product that falls off the bottom, and their `4 tiny`
    /// outweighs what it lost at the input's own scale or at the second
    /// exponent, so none of them does any longer. The turn that still needs
    /// the third exponent is the next test's.
    #[test]
    fn a_spike_at_the_origin_under_a_unit_square_is_decided_on_every_turn() {
        let corners = a_spike_at_the_origin_under_a_unit_square();
        for t in [2f64.powi(-995), 2f64.powi(-1000), f64::from_bits(1)] {
            assert!(t > 0.0 && t <= 2f64.powi(-41), "{t:e}");
            for (i, j, k) in every_turn(corners.len()) {
                let (a, b, c) = (corners[i].at(t), corners[j].at(t), corners[k].at(t));
                assert_eq!(
                    orient2d(a, b, c),
                    the_sign_at_a_tiny_t(corners[i], corners[j], corners[k]),
                    "t = {t:e}, corners {i} {j} {k}"
                );
                assert!(
                    !refused_before_the_third_exponent(a, b, c),
                    "t = {t:e}, corners {i} {j} {k}"
                );
            }
        }
    }

    /// A turn whose `tiny * 1` products cancel exactly, leaving only
    /// `tiny * tiny`: `(1, 0)`, `(0, t)`, `(t, t)`, whose determinant is
    /// `-t^2`. At `t` the smallest subnormal, the second exponent lifts `t`
    /// to `2^-565`, and `t * t` is still `2^-1130`, below anything a product
    /// can keep, with nothing else left over to outweigh it. The third lifts
    /// every coordinate as high as `f64` goes, since no product here is
    /// `1 * 1`, and there `t * t` is `2^-102`.
    #[test]
    fn a_turn_left_with_nothing_but_tiny_times_tiny_still_needs_the_third_exponent() {
        let t = f64::from_bits(1);
        let (a, b, c) = ([1.0, 0.0], [0.0, t], [t, t]);
        assert!(refused_before_the_third_exponent(a, b, c));
        assert_eq!(orient2d(a, b, c), Decision::Neg);
    }

    fn every_turn(corners: usize) -> impl Iterator<Item = (usize, usize, usize)> {
        (0..corners).flat_map(move |i| {
            (0..corners).flat_map(move |j| {
                (0..corners)
                    .filter(move |&k| i != j && j != k && i != k)
                    .map(move |k| (i, j, k))
            })
        })
    }

    fn refused_before_the_third_exponent(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
        let [first, second, _] = exponents_to_try(a, b, c);
        !decide(&Orient2d { a, b, c }).0.is_certified()
            && each_once_and_never_zero([first, second])
                .all(|exponent| decided_at_scale(a, b, c, exponent).is_none())
    }

    /// A corner whose coordinates are `unit + tiny * t`, as whole numbers, and
    /// never both on one axis, so that each is an `f64` for every `t`.
    #[derive(Clone, Copy)]
    struct SpikeCorner {
        unit: [i32; 2],
        tiny: [i32; 2],
    }

    impl SpikeCorner {
        fn at(self, t: f64) -> [f64; 2] {
            [0, 1].map(|axis| {
                let (unit, tiny) = (self.unit[axis], self.tiny[axis]);
                assert!(unit == 0 || tiny == 0, "{unit} + {tiny} t is not an f64");
                f64::from(unit) + f64::from(tiny) * t
            })
        }
    }

    /// The spike §324.7 measured, which `when_it_cannot_tell.rs` used to pin
    /// as undecidable: the unit square with a spike `t` wide and `3t` tall at
    /// the origin.
    fn a_spike_at_the_origin_under_a_unit_square() -> [SpikeCorner; 7] {
        let tiny = |tiny| SpikeCorner { unit: [0, 0], tiny };
        let unit = |unit| SpikeCorner { unit, tiny: [0, 0] };
        [
            tiny([0, 0]),
            tiny([1, 3]),
            tiny([2, 0]),
            unit([1, -1]),
            unit([1, 1]),
            unit([-1, 1]),
            unit([-1, -1]),
        ]
    }

    /// The sign of `det[a - c, b - c]` for a positive `t` too small to matter
    /// against whole numbers.
    ///
    /// The determinant is `A + B t + C t^2` with whole `A`, `B` and `C`. The
    /// first of them that is not zero is at least `1` times its power of `t`,
    /// and while all three are below `2^40` the rest come to less than
    /// `2^41` times the next power, so for `t <= 2^-41` the first one's sign is
    /// the determinant's.
    fn the_sign_at_a_tiny_t(a: SpikeCorner, b: SpikeCorner, c: SpikeCorner) -> Decision {
        let difference = |p: SpikeCorner, q: SpikeCorner, axis: usize| {
            [
                i64::from(p.unit[axis] - q.unit[axis]),
                i64::from(p.tiny[axis] - q.tiny[axis]),
            ]
        };
        let times =
            |u: [i64; 2], v: [i64; 2]| [u[0] * v[0], u[0] * v[1] + u[1] * v[0], u[1] * v[1]];
        let forwards = times(difference(a, c, 0), difference(b, c, 1));
        let backwards = times(difference(a, c, 1), difference(b, c, 0));
        let coefficients = [0, 1, 2].map(|power| forwards[power] - backwards[power]);
        assert!(
            coefficients.iter().all(|c| c.abs() < 1 << 40),
            "{coefficients:?} is too large for t to be negligible"
        );
        match coefficients
            .into_iter()
            .find(|&c| c != 0)
            .unwrap_or(0)
            .cmp(&0)
        {
            Ordering::Greater => Decision::Pos,
            Ordering::Less => Decision::Neg,
            Ordering::Equal => Decision::Zero,
        }
    }

    /// `LARGEST_SUM_OF_FACTOR_BINADES`'s arithmetic, run rather than argued:
    /// every x at the top of one binade and every y at the top of another, the
    /// two adding up to the constant, under all 64 ways to sign the six, and
    /// level 3 overflows nothing. One more and some of them do. The rule reads
    /// only the sum, so the sum is split three ways, from lopsided to even.
    #[test]
    fn the_highest_sum_of_factor_binades_overflows_nothing_in_level_three() {
        for sum in [
            LARGEST_SUM_OF_FACTOR_BINADES,
            LARGEST_SUM_OF_FACTOR_BINADES + 1,
        ] {
            for x_binade in [sum - 1023, sum / 2, 1023] {
                let overflowed = sign_patterns_that_overflow_level_three(x_binade, sum - x_binade);
                if sum == LARGEST_SUM_OF_FACTOR_BINADES {
                    assert_eq!(overflowed, 0, "x in binade {x_binade}");
                } else {
                    assert!(overflowed > 0, "x in binade {x_binade}");
                }
            }
        }
    }

    fn sign_patterns_that_overflow_level_three(x_binade: i32, y_binade: i32) -> usize {
        (0..1u32 << 6)
            .filter(|&signs| {
                let [a, b, c] = signed_corners(signs, top_of(x_binade), top_of(y_binade));
                let mut total = [0.0; DETERMINANT_ENTRIES];
                exact_determinant(a, b, c, &mut total)
                    .is_none_or(|(length, _)| !total[..length].iter().all(|e| e.is_finite()))
            })
            .count()
    }

    /// At the third exponent levels 1 and 2 can overflow, and that has to cost
    /// them their answer and nothing else. Every turn of the spike above,
    /// scaled by its own third exponent, gets level 3's answer from the whole
    /// cascade, and that answer is the whole-number one — while level 1's
    /// bound and level 2's interval run off the top on some of them.
    ///
    /// None of the spike's own turns need the third exponent to decide
    /// `orient2d` itself —
    /// `a_spike_at_the_origin_under_a_unit_square_is_decided_on_every_turn`
    /// shows every one of them settled before it — so this test also walks
    /// `a_turn_left_with_nothing_but_tiny_times_tiny_still_needs_the_third_exponent`'s
    /// turn, the one turn here that does, and calls `orient2d` on it
    /// unscaled. That is what makes this test fail if `orient2d` ever stops
    /// trying the third exponent at its own call site; scaling the points by
    /// hand and calling `decide` on the result, as the rest of this test
    /// does, would not notice that at all.
    #[test]
    fn levels_one_and_two_overflowing_at_the_third_exponent_still_agree_with_level_three() {
        let corners = a_spike_at_the_origin_under_a_unit_square();
        let (mut level_one_overflowed, mut level_two_overflowed) = (0, 0);
        for t in [2f64.powi(-995), 2f64.powi(-1000), f64::from_bits(1)] {
            for (i, j, k) in every_turn(corners.len()) {
                let (unscaled_a, unscaled_b, unscaled_c) =
                    (corners[i].at(t), corners[j].at(t), corners[k].at(t));
                let [_, _, Some(third)] = exponents_to_try(unscaled_a, unscaled_b, unscaled_c)
                else {
                    panic!("no corner is far enough from zero to lift");
                };
                let [a, b, c] = [unscaled_a, unscaled_b, unscaled_c].map(|point| {
                    scaled_by_a_power_of_two(point, third)
                        .expect("the third exponent keeps every coordinate an f64")
                });
                let exactly = orient2d_exactly(a, b, c);
                assert_eq!(
                    exactly,
                    the_sign_at_a_tiny_t(corners[i], corners[j], corners[k])
                );
                assert_eq!(
                    decide(&Orient2d { a, b, c }).0,
                    exactly,
                    "corners {i} {j} {k}"
                );
                assert_eq!(
                    orient2d(unscaled_a, unscaled_b, unscaled_c),
                    exactly,
                    "corners {i} {j} {k}, t = {t:e}"
                );

                let predicate = Orient2d { a, b, c };
                let level_one = predicate.on_floats().expect("level 1 always runs");
                if !level_one.error_bound.is_finite() {
                    level_one_overflowed += 1;
                }
                let level_two = predicate.on_intervals();
                if !level_two.lo().is_finite() || !level_two.hi().is_finite() {
                    level_two_overflowed += 1;
                }
            }
        }
        assert!(
            level_one_overflowed > 0 && level_two_overflowed > 0,
            "level 1 overflowed on {level_one_overflowed} and level 2 on {level_two_overflowed}, \
             so this no longer shows an overflow costing nothing"
        );

        let t = f64::from_bits(1);
        let (a, b, c) = ([1.0, 0.0], [0.0, t], [t, t]);
        assert!(refused_before_the_third_exponent(a, b, c));
        assert_eq!(orient2d(a, b, c), Decision::Neg);
    }

    fn top_of(binade: i32) -> f64 {
        2f64.powi(binade + 1).next_down()
    }

    /// Three corners with every x of magnitude `x` and every y of magnitude
    /// `y`, signed by the six bits of `signs`.
    fn signed_corners(signs: u32, x: f64, y: f64) -> [[f64; 2]; 3] {
        let signed = |bit: u32, magnitude: f64| {
            if signs & (1 << bit) == 0 {
                magnitude
            } else {
                -magnitude
            }
        };
        [0, 1, 2].map(|corner| [signed(2 * corner, x), signed(2 * corner + 1, y)])
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
        let Some((length, 0)) = exact_determinant(a, b, c, &mut determinant) else {
            return None;
        };

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
