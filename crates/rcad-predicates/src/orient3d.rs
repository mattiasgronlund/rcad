use crate::expansion::{
    difference_of_two_products, expansion_sign, expansion_sum, product_is_exact, scale_expansion,
};
use crate::interval::determinant;
use crate::power_of_two::{
    each_once_and_never_zero, exponent_that_brings_into_range,
    exponent_that_lifts_every_product_to, exponent_that_lifts_the_largest_to,
    scaled_by_a_power_of_two,
};
use crate::{
    Decision, Filtered, Interval, Level, Measures, NOTHING_IS_KNOWN, Point, Policy, Predicate,
    ProvenanceLog, Record, decide, is_a_real_point, product_is_normal, resolve_under_policy,
};

/// The name this predicate goes by in provenance records.
pub const ORIENT3D: &str = "orient3d";

/// Which side of the plane through `a`, `b` and `c` the point `d` lies on.
///
/// [`Decision::Pos`] when `d` is below the plane, where "below" means `a`, `b`
/// and `c` run counterclockwise seen from above it. [`Decision::Neg`] when `d`
/// is above. [`Decision::Zero`] when all four are coplanar. This is the same
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
/// high as the top of the range allows.
///
/// What is left as [`Decision::Undecidable`] is a genuine *spread* — a model
/// whose coordinates sit far from the origin while its differences are far
/// below it, so that no single scaling brings both into range at once. That is
/// the case decisions.md §3.5's dynamic-range check is for, and being a ratio
/// is exactly what lets it see this and not the uniform cases (§16.11).
///
/// A coordinate that is not a real number at all gets [`NOTHING_IS_KNOWN`]
/// before any of that runs. [`is_a_real_point`] says why.
///
/// ```
/// # use rcad_predicates::{orient3d, Decision};
/// let flat = orient3d([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]);
/// assert_eq!(flat, Decision::Zero);
/// ```
pub fn orient3d(a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) -> Decision {
    orient3d_of_points(a.into(), b.into(), c.into(), d.into())
}

/// [`orient3d`] where any argument may be a constructed point.
///
/// A constructed point is only known to lie inside a box, so level 3 of the
/// cascade cannot run: exact arithmetic on an inexact input answers the wrong
/// question. When the interval straddles zero the answer is
/// [`Decision::Undecidable`], and no amount of precision will change that
/// (decisions.md §2.5).
pub fn orient3d_of_points(a: Point, b: Point, c: Point, d: Point) -> Decision {
    orient3d_reporting_level(a, b, c, d).0
}

/// [`orient3d_of_points`], and which tier of the cascade settled it.
///
/// A diagnostic. Tests and benchmarks use it to check that the cheap levels
/// are carrying the load they are supposed to.
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
/// All three public entry points funnel through here, so all three get the
/// retry.
pub fn orient3d_reporting_level(a: Point, b: Point, c: Point, d: Point) -> (Decision, Level) {
    if ![a, b, c, d].into_iter().all(is_a_real_point) {
        return (NOTHING_IS_KNOWN, Level::Interval);
    }
    let refused = match decide(&Orient3d { a, b, c, d }) {
        (Decision::Undecidable { enclosure }, level) => {
            (Decision::Undecidable { enclosure }, level)
        }
        settled => return settled,
    };
    decided_after_normalising(a, b, c, d).unwrap_or(refused)
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
pub fn orient3d_tolerantly(
    a: Point,
    b: Point,
    c: Point,
    d: Point,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Decision {
    let (decision, level) = orient3d_reporting_level(a, b, c, d);
    let Decision::Undecidable { enclosure } = decision else {
        return decision;
    };
    let record = Record::new(ORIENT3D, &[a, b, c, d], level, enclosure, Measures::Volume);
    resolve_under_policy(record, policy, log)
}

/// Up to three more runs of the whole cascade, on a copy of the input moved
/// into range by an exact power of two. `None` when there is no such copy, or
/// when no copy could be settled either.
///
/// This determinant is homogeneous of degree three in the coordinates, so
/// scaling all twelve by `s > 0` multiplies it by `s^3` and leaves its sign
/// alone — and a determinant of exactly zero stays exactly zero. That is what
/// the metamorphic test next door already trusts; here it is used as a
/// technique rather than as an oracle. The copy has to be exact for any of
/// that to mean something, and [`scaled_by_a_power_of_two`] refuses one that
/// is not.
///
/// # Three exponents, tried in turn
///
/// The first brings the largest *difference* into `[1, 2)`, which is what
/// levels 1 and 2 multiply, and settles coordinates that are uniformly tiny or
/// uniformly enormous.
///
/// It does nothing for four points near the origin whose differences run from
/// `2` down to `1e-300`: the largest difference is already in `[1, 2)`, so the
/// exponent is zero. Level 3 is what can settle those, and it refuses only
/// because products near `1e-300` sit below
/// [`product_is_exact`]'s floor — even where, as in `docs/decisions.md` §324,
/// the product is a power of two times the coordinate and is exact anyway, and
/// the determinant is exactly zero, which only level 3 can certify. So
/// the second exponent lifts the largest *coordinate* into
/// [`HIGHEST_BINADE_LEVEL_THREE_TAKES`], which lifts every product level 3
/// forms as far off the bottom as the top of the range allows
/// ([`exponent_that_lifts_the_largest_to`]).
///
/// The first is tried first so that nothing it already settled moves: the
/// second only ever turns a refusal into an answer.
///
/// The third lifts the products level 3 forms as high as they go, for a spike
/// at the origin that the second leaves under that floor;
/// [`exponents_to_try`] says why.
///
/// A constructed point has no coordinates to scale, and its refusal is
/// decisions.md §2.5's limit rather than a gap — no arithmetic recovers a
/// number that was never there — so this stops at the first one it meets.
///
/// Only reached from an [`Decision::Undecidable`], so the path that already
/// works pays nothing. The retry re-enters at the top rather than inside
/// [`Orient3d::on_floats`] because all three levels fail together on these
/// inputs: subnormal products at level 1, interval products that collapse to
/// zero at level 2, and an expansion that declines to run at level 3.
///
/// # Why `ORIENT3D_ERROR_BOUND` carries over untouched
///
/// Rounding to nearest commutes with multiplication by an exact power of two:
/// while nothing leaves the normal range, `fl(2^k * x) = 2^k * fl(x)`, because
/// the two numbers have the same mantissa and the grid they round onto is the
/// same grid scaled. So each difference in [`Orient3d::on_floats`] comes out as
/// `s` times what it was, each 2x2 minor as `s^2`, each of the three terms as
/// `s^3`, and `value` as `s^3` — the same mantissa bits throughout. The
/// permanent is a sum of those same degree-three products, so it scales by
/// `s^3` as well, and `value > ORIENT3D_ERROR_BOUND * permanent` compares two
/// sides multiplied by the same positive number. Level 1 is therefore
/// scale-invariant, which is why the bound needs no adjusting and gets none.
///
/// That argument is about not *losing* certifications. Soundness needs less
/// than it: the scaled copy is an ordinary exact `f64` input, and
/// `ORIENT3D_ERROR_BOUND` is a relative bound that holds for any such input
/// whose products stay normal, so it holds here on its own terms.
fn decided_after_normalising(a: Point, b: Point, c: Point, d: Point) -> Option<(Decision, Level)> {
    let [a, b, c, d] = [a, b, c, d].map(Point::exact_coordinates);
    let (a, b, c, d) = (a?, b?, c?, d?);

    each_once_and_never_zero(exponents_to_try([a, b, c, d]))
        .find_map(|exponent| decided_at_scale(a, b, c, d, exponent))
}

/// The exponents [`decided_after_normalising`] tries, in the order it tries
/// them.
///
/// The third is for a spike at the origin under a unit cube, whose
/// determinants hold `tiny * tiny * tiny` and `tiny * tiny * 1` but never
/// `1 * 1 * 1` (`docs/decisions.md` §324.7). The second lifts the largest
/// coordinate only as far as `1 * 1 * 1` could go, which leaves
/// `tiny * tiny * tiny` under [`product_is_exact`]'s floor below
/// `tiny = 2^-661`. The third reads the products level 3 forms one by one —
/// the two in each minor, and those two times each `z` the minor is scaled by
/// — and lifts until the one closest to the top reaches
/// [`LARGEST_SUM_OF_TWO_FACTOR_BINADES`] or
/// [`LARGEST_SUM_OF_THREE_FACTOR_BINADES`]. It never lands below the second,
/// because no product is larger than the largest coordinate to its own power.
/// It comes last so that nothing the first two settle can move.
///
/// Levels 1 and 2 multiply differences, and at the third exponent those can
/// overflow. That costs them their answer and nothing else: an infinite level 1
/// bound is not normal and [`decide`] skips it, and an interval that overflows
/// widens to take in infinity.
/// `levels_one_and_two_overflowing_at_the_third_exponent_still_agree_with_level_three`
/// runs that.
fn exponents_to_try(points: [[f64; 3]; 4]) -> [Option<i32>; 3] {
    let [a, b, c, d] = points;
    let coordinates = points.into_iter().flatten();
    [
        exponent_that_brings_into_range([
            a[0] - d[0],
            a[1] - d[1],
            a[2] - d[2],
            b[0] - d[0],
            b[1] - d[1],
            b[2] - d[2],
            c[0] - d[0],
            c[1] - d[1],
            c[2] - d[2],
        ]),
        exponent_that_lifts_the_largest_to(coordinates.clone(), HIGHEST_BINADE_LEVEL_THREE_TAKES),
        exponent_that_lifts_every_product_to(
            coordinates.clone(),
            products_in_the_minors(points),
            LARGEST_SUM_OF_TWO_FACTOR_BINADES,
        )
        .zip(exponent_that_lifts_every_product_to(
            coordinates,
            products_in_the_terms(points),
            LARGEST_SUM_OF_THREE_FACTOR_BINADES,
        ))
        .map(|(minors, terms)| minors.min(terms)),
    ]
}

/// The pairs of points whose minor [`exact_determinant`] forms.
const MINORS: [(usize, usize); 6] = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];

/// The two products in each minor `u.x * v.y - u.y * v.x`.
fn products_in_the_minors(points: [[f64; 3]; 4]) -> impl Iterator<Item = [f64; 2]> {
    MINORS.into_iter().flat_map(move |(u, v)| {
        let (u, v) = (points[u], points[v]);
        [[u[0], v[1]], [u[1], v[0]]]
    })
}

/// The same products times the `z` of each of the two other points, which is
/// what [`exact_determinant`] scales each minor by.
fn products_in_the_terms(points: [[f64; 3]; 4]) -> impl Iterator<Item = [f64; 3]> {
    MINORS.into_iter().flat_map(move |(u, v)| {
        (0..4)
            .filter(move |&other| other != u && other != v)
            .flat_map(move |other| {
                let (u, v, z) = (points[u], points[v], points[other][2]);
                [[u[0], v[1], z], [u[1], v[0], z]]
            })
    })
}

/// How high the binades of the two factors in one of a minor's products may add
/// up to at the third exponent, for level 3 to overflow nothing.
///
/// Measured rather than argued: with every x, y and z at the top of a binade of
/// its own, which makes every product as large as this allows at once, none of
/// the 4096 ways to sign the twelve overflows at `1021`, and 3584 do at
/// `1022`.
/// `the_highest_sums_of_factor_binades_overflow_nothing_in_level_three` runs
/// it, together with [`LARGEST_SUM_OF_THREE_FACTOR_BINADES`].
///
/// Like [`HIGHEST_BINADE_LEVEL_THREE_TAKES`], this decides reach and not
/// soundness: an overflowed product is refused by [`product_is_exact`], and an
/// overflowed sum by [`expansion_sign`].
const LARGEST_SUM_OF_TWO_FACTOR_BINADES: i32 = 1021;

/// The same for the three factors of a product times the `z` that scales it,
/// where none of the 4096 overflows at `1017` and 336 do at `1018`.
const LARGEST_SUM_OF_THREE_FACTOR_BINADES: i32 = 1017;

/// The binade [`decided_after_normalising`]'s second exponent lifts the largest
/// coordinate into: `[2^338, 2^339)`, the highest in which no level of the
/// cascade can overflow.
///
/// Every coordinate is then below `2^339`. Level 3 multiplies two coordinates
/// twice in each of six minors, so each minor is below `2^679`, and scales
/// each by a third coordinate: twelve terms below `2^1018` each, summing to
/// below `12 * 2^1018 < 2^1022`. Levels 1 and 2 work on differences, which are
/// at most `2^340`, so each product is at most `2^680`, each minor `2^681`,
/// each of the three terms `2^1021`, and the value and the permanent below
/// `3 * 2^1021 < 2^1023`. One binade higher and twelve terms at the top could
/// pass `2^1024`.
/// `the_highest_binade_the_retry_lifts_into_overflows_nothing` runs it.
///
/// Going past it would only cost reach, never soundness — an overflowed level 1
/// bound is refused by [`decide`] and an overflowed product by
/// [`product_is_exact`] — so the one thing this number decides is how far the
/// retry gets.
const HIGHEST_BINADE_LEVEL_THREE_TAKES: i32 = 338;

/// [`decide`] on the four points scaled by `2^exponent`, if that is exact and
/// the answer is certified.
fn decided_at_scale(
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
    d: [f64; 3],
    exponent: i32,
) -> Option<(Decision, Level)> {
    let a = scaled_by_a_power_of_two(a, exponent)?;
    let b = scaled_by_a_power_of_two(b, exponent)?;
    let c = scaled_by_a_power_of_two(c, exponent)?;
    let d = scaled_by_a_power_of_two(d, exponent)?;

    // An enclosure carries the units of the value it encloses (decisions.md
    // §12.4), and this one would carry them scaled by `s^3`. A sign is
    // dimensionless and survives; a width does not, so only a certified answer
    // comes back from here.
    let (decision, level) = decide(&Orient3d {
        a: a.into(),
        b: b.into(),
        c: c.into(),
        d: d.into(),
    });
    decision.is_certified().then_some((decision, level))
}

struct Orient3d {
    a: Point,
    b: Point,
    c: Point,
    d: Point,
}

impl Predicate for Orient3d {
    fn name(&self) -> &'static str {
        ORIENT3D
    }

    fn on_floats(&self) -> Option<Filtered> {
        let [a, b, c, d] = self.exact_coordinates()?;

        let (adx, ady, adz) = (a[0] - d[0], a[1] - d[1], a[2] - d[2]);
        let (bdx, bdy, bdz) = (b[0] - d[0], b[1] - d[1], b[2] - d[2]);
        let (cdx, cdy, cdz) = (c[0] - d[0], c[1] - d[1], c[2] - d[2]);

        let bdxcdy = bdx * cdy;
        let cdxbdy = cdx * bdy;
        let cdxady = cdx * ady;
        let adxcdy = adx * cdy;
        let adxbdy = adx * bdy;
        let bdxady = bdx * ady;

        let (first, second, third) = (bdxcdy - cdxbdy, cdxady - adxcdy, adxbdy - bdxady);

        // The bound below is a fraction of the permanent, which only bounds
        // anything while every product here keeps its relative accuracy. One
        // of them in the subnormals and this level can certify a wrong sign
        // (decisions.md §12.2).
        //
        // The permanent's own three products are each at least as large as the
        // matching one here — `|x| + |y|` is never below `|x - y|` — so for
        // each of `first`, `second` and `third` that is not zero, checking
        // these nine covers its product in the permanent too. A minor that is
        // zero passes its check on the zero, and the permanent's matching
        // product can then fall into the subnormals or to zero: `bdxcdy ==
        // cdxbdy == 2^-1000` with `adz == 2^-100` make it `2^-1099`, which
        // rounds to zero. The sign level 1 certifies is still right there:
        // Gappa proves it with any one or any two of the three minors at zero,
        // and with all three `value` is zero, which certifies nothing
        // (decisions.md §464.7).
        let stays_accurate = product_is_normal(bdx, cdy)
            && product_is_normal(cdx, bdy)
            && product_is_normal(cdx, ady)
            && product_is_normal(adx, cdy)
            && product_is_normal(adx, bdy)
            && product_is_normal(bdx, ady)
            && product_is_normal(adz, first)
            && product_is_normal(bdz, second)
            && product_is_normal(cdz, third);
        if !stays_accurate {
            return None;
        }

        let value = adz * first + bdz * second + cdz * third;

        // The bound has to allow for cancellation inside each 2x2 minor, so it
        // sums the magnitudes of the two products rather than the magnitude of
        // their difference. Shewchuk calls this the permanent of the matrix.
        let permanent = (bdxcdy.abs() + cdxbdy.abs()) * adz.abs()
            + (cdxady.abs() + adxcdy.abs()) * bdz.abs()
            + (adxbdy.abs() + bdxady.abs()) * cdz.abs();

        Some(Filtered {
            value,
            error_bound: ORIENT3D_ERROR_BOUND * permanent,
        })
    }

    fn on_intervals(&self) -> Interval {
        let a = self.a.enclosure();
        let b = self.b.enclosure();
        let c = self.c.enclosure();
        let d = self.d.enclosure();

        determinant([
            [a[0] - d[0], a[1] - d[1], a[2] - d[2]],
            [b[0] - d[0], b[1] - d[1], b[2] - d[2]],
            [c[0] - d[0], c[1] - d[1], c[2] - d[2]],
        ])
    }

    fn exactly(&self) -> Option<Decision> {
        let [a, b, c, d] = self.exact_coordinates()?;
        match orient3d_exactly(a, b, c, d) {
            Decision::Neg => Some(Decision::Neg),
            Decision::Zero => Some(Decision::Zero),
            Decision::Pos => Some(Decision::Pos),
            // A product overflowed or underflowed. Exact arithmetic that is
            // not exact has nothing to offer, so this level cannot run.
            Decision::Undecidable { .. } => None,
        }
    }
}

impl Orient3d {
    fn exact_coordinates(&self) -> Option<[[f64; 3]; 4]> {
        Some([
            self.a.exact_coordinates()?,
            self.b.exact_coordinates()?,
            self.c.exact_coordinates()?,
            self.d.exact_coordinates()?,
        ])
    }
}

/// Level 1's forward error bound, as a multiple of the permanent.
///
/// `epsilon` below is the `f64` unit roundoff, 2⁻⁵³: half an ulp of 1, and so
/// the most a single rounding can cost in relative terms.
///
/// **It bounds the sum `value` is rounded from, not `value`.** In
/// [`Orient3d::on_floats`], `value = adz * first + bdz * second + cdz * third`
/// ends with one addition; call what that addition comes to before it rounds
/// `y`. Where the 7 comes from — walk the longest chain of roundings between
/// the input coordinates and `y`. The three differences that meet in one term
/// (`adz`, `bdx`, `cdy`) are a rounding each, since their errors multiply into
/// it. The product inside the 2x2 minor (`bdx * cdy`) is a fourth, the
/// minor's own subtraction a fifth, multiplying it by `adz` a sixth, and adding
/// it to the second term the seventh. Seven roundings, each costing at most a
/// factor of `(1 + epsilon)`, give `(1 + epsilon)^7 - 1 <= 7*epsilon +
/// 21*epsilon^2 + ...` times the permanent. The `56*epsilon^2` covers those
/// higher-order terms together with the rounding of the permanent itself.
/// Gappa proves `y` within this bound of the true determinant (decisions.md
/// §464.4, and §464.7 for a minor that is zero).
///
/// That is what the sign needs. [`decide`] certifies `Pos` when `value >
/// error_bound`, and `error_bound` is a double, so rounding `y` to `value`
/// cannot carry it past `error_bound` unless `y` was past it already: `y` at
/// most `error_bound` rounds to at most `error_bound`. And `y` above
/// `error_bound` and within it of the truth makes the truth positive. `Neg` is
/// the same with the signs turned. `value` itself takes an eighth rounding,
/// and can land further from the truth than `error_bound`:
/// `level_one_can_land_past_its_error_bound_and_still_certify_the_right_sign`
/// builds a tetrahedron where it lands 8/7 as far (decisions.md §464.7).
///
/// The value is Shewchuk's `o3derrboundA`, from "Adaptive Precision
/// Floating-Point Arithmetic and Fast Robust Geometric Predicates" (1997)
/// §4.4 and the `exactinit` routine of his `predicates.c`. The `robust` crate
/// carries the same constant, which is one more reason the differential test
/// is worth having.
///
/// Every step of that argument is a *relative* one, so the whole bound holds
/// only while nothing overflows or falls into the subnormals.
/// [`Orient3d::on_floats`] enforces the bottom end product by product through
/// [`product_is_normal`], and [`decide`] the top end by refusing a bound that
/// is not itself a normal number. Without those two the filter certifies wrong
/// signs, which is what `robust` does with the same constant.
const ORIENT3D_ERROR_BOUND: f64 = {
    let epsilon = f64::EPSILON / 2.0;
    (7.0 + 56.0 * epsilon) * epsilon
};

/// The exact sign of the determinant, in expansion arithmetic.
///
/// The value wanted is the determinant of the rows `a - d`, `b - d`, `c - d`.
/// Those differences are not `f64`, so the determinant is expanded over the
/// original coordinates instead. Multilinearity in the rows gives
///
/// ```text
/// det[a-d, b-d, c-d] = det[a,b,c] - det[a,b,d] + det[a,c,d] - det[b,c,d]
/// ```
///
/// and expanding each three by three determinant along its third column, with
/// `M(u,v) = u.x*v.y - u.y*v.x`, gives
///
/// ```text
/// det[p,q,r] = M(p,q)*r.z + M(q,r)*p.z - M(p,r)*q.z
/// ```
///
/// Collecting the four determinants leaves twelve terms, each one of the six
/// distinct `M` values scaled by a single coordinate. Every `M` is exact in
/// four entries and every scaling at most doubles that, so twelve terms need
/// at most 96 entries — the capacity below.
///
/// Comes back [`Decision::Undecidable`] when a product overflowed or
/// underflowed, because then the arithmetic is not exact and there is nothing
/// honest to report. Additions never need that check: they are exact for every
/// pair of `f64` that does not overflow.
fn orient3d_exactly(a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) -> Decision {
    let mut total = [0.0f64; DETERMINANT_ENTRIES];
    match exact_determinant(a, b, c, d, &mut total) {
        Some(total_len) => expansion_sign(&total[..total_len]),
        None => Decision::Undecidable {
            enclosure: f64::INFINITY,
        },
    }
}

/// The determinant itself, written into `total`, and how many entries it took.
///
/// `None` when a product overflowed or underflowed, which is the case
/// [`orient3d_exactly`] has no honest sign for.
fn exact_determinant(
    a: [f64; 3],
    b: [f64; 3],
    c: [f64; 3],
    d: [f64; 3],
    total: &mut [f64; DETERMINANT_ENTRIES],
) -> Option<usize> {
    let (Some((ab, ab_len)), Some((ac, ac_len)), Some((ad, ad_len))) =
        (minor(a, b), minor(a, c), minor(a, d))
    else {
        return None;
    };
    let (Some((bc, bc_len)), Some((bd, bd_len)), Some((cd, cd_len))) =
        (minor(b, c), minor(b, d), minor(c, d))
    else {
        return None;
    };

    let terms: [(&[f64], f64); 12] = [
        (&ab[..ab_len], c[2]),
        (&ab[..ab_len], -d[2]),
        (&bc[..bc_len], a[2]),
        (&bc[..bc_len], -d[2]),
        (&ac[..ac_len], -b[2]),
        (&ac[..ac_len], d[2]),
        (&bd[..bd_len], -a[2]),
        (&bd[..bd_len], c[2]),
        (&ad[..ad_len], b[2]),
        (&ad[..ad_len], -c[2]),
        (&cd[..cd_len], a[2]),
        (&cd[..cd_len], -b[2]),
    ];

    let mut total_len = 0;
    let mut merged = [0.0f64; DETERMINANT_ENTRIES];
    let mut term = [0.0f64; 8];

    for (two_by_two, scale) in terms {
        if !two_by_two
            .iter()
            .all(|&entry| product_is_exact(entry, scale))
        {
            return None;
        }
        let term_len = scale_expansion(two_by_two, scale, &mut term);
        total_len = expansion_sum(&total[..total_len], &term[..term_len], &mut merged);
        core::mem::swap(total, &mut merged);
    }

    Some(total_len)
}

/// The twelve terms of the expansion above need at most eight entries each.
const DETERMINANT_ENTRIES: usize = 96;

/// `u.x * v.y - u.y * v.x`, exactly, or `None` if a product underflowed.
fn minor(u: [f64; 3], v: [f64; 3]) -> Option<([f64; 4], usize)> {
    difference_of_two_products(u[0], v[1], u[1], v[0])
}

#[cfg(test)]
mod tests {
    use super::{
        DETERMINANT_ENTRIES, HIGHEST_BINADE_LEVEL_THREE_TAKES, LARGEST_SUM_OF_THREE_FACTOR_BINADES,
        LARGEST_SUM_OF_TWO_FACTOR_BINADES, ORIENT3D, ORIENT3D_ERROR_BOUND, Orient3d,
        decided_at_scale, each_once_and_never_zero, exact_determinant, exponents_to_try, orient3d,
        orient3d_exactly, orient3d_of_points, orient3d_reporting_level, orient3d_tolerantly,
        scaled_by_a_power_of_two,
    };
    use crate::expansion::expansion_sum;
    use crate::{
        Decision, Filtered, Interval, Level, Point, Policy, Predicate, ProvenanceLog, decide,
    };
    use core::cmp::Ordering;

    /// The naive `f64` determinant. Exact when the coordinates are small
    /// integers, which is what makes it a usable oracle for the sign
    /// convention.
    fn naive_orient3d(a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) -> f64 {
        let r = [
            [a[0] - d[0], a[1] - d[1], a[2] - d[2]],
            [b[0] - d[0], b[1] - d[1], b[2] - d[2]],
            [c[0] - d[0], c[1] - d[1], c[2] - d[2]],
        ];
        r[0][0] * (r[1][1] * r[2][2] - r[1][2] * r[2][1])
            - r[0][1] * (r[1][0] * r[2][2] - r[1][2] * r[2][0])
            + r[0][2] * (r[1][0] * r[2][1] - r[1][1] * r[2][0])
    }

    #[test]
    fn exact_arithmetic_agrees_with_the_plain_determinant_on_small_integers() {
        let mut checked = 0;
        for ax in -2..=2 {
            for by in -2..=2 {
                for cz in -2..=2 {
                    for dz in -2..=2 {
                        let a = [f64::from(ax), 1.0, 2.0];
                        let b = [3.0, f64::from(by), -1.0];
                        let c = [-2.0, 4.0, f64::from(cz)];
                        let d = [1.0, -3.0, f64::from(dz)];
                        let plain = naive_orient3d(a, b, c, d);
                        let expected = if plain > 0.0 {
                            Decision::Pos
                        } else if plain < 0.0 {
                            Decision::Neg
                        } else {
                            Decision::Zero
                        };
                        assert_eq!(
                            orient3d_exactly(a, b, c, d),
                            expected,
                            "{a:?} {b:?} {c:?} {d:?}"
                        );
                        checked += 1;
                    }
                }
            }
        }
        assert_eq!(checked, 625);
    }

    #[test]
    fn a_point_below_a_counterclockwise_triangle_is_positive() {
        assert_eq!(
            orient3d(
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, -1.0]
            ),
            Decision::Pos
        );
        assert_eq!(
            orient3d(
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0]
            ),
            Decision::Neg
        );
    }

    #[test]
    fn a_point_far_off_the_plane_is_settled_by_the_cheapest_level() {
        let (decision, level) = orient3d_reporting_level(
            Point::from([0.0, 0.0, 0.0]),
            Point::from([1.0, 0.0, 0.0]),
            Point::from([0.0, 1.0, 0.0]),
            Point::from([0.3, 0.3, -1.0]),
        );
        assert_eq!(decision, Decision::Pos);
        assert_eq!(level, Level::Float);
    }

    #[test]
    fn a_repeated_point_makes_the_determinant_zero() {
        let a = [1.5, -2.25, 0.125];
        let b = [0.0, 3.0, 7.0];
        assert_eq!(orient3d(a, b, a, [4.0, 5.0, 6.0]), Decision::Zero);
        assert_eq!(orient3d(a, b, [4.0, 5.0, 6.0], a), Decision::Zero);
    }

    #[test]
    fn swapping_two_points_flips_the_answer() {
        let (a, b, c, d) = (
            [0.5, 0.25, 0.125],
            [3.0, -1.0, 2.0],
            [-4.0, 7.0, 0.5],
            [1.0, 1.0, 9.0],
        );
        assert_eq!(orient3d(a, b, c, d), Decision::Neg);
        assert_eq!(orient3d(b, a, c, d), Decision::Pos);
    }

    #[test]
    fn a_certified_answer_leaves_the_log_empty() {
        let mut log = ProvenanceLog::new();
        let decision = orient3d_tolerantly(
            Point::from([0.0, 0.0, 0.0]),
            Point::from([1.0, 0.0, 0.0]),
            Point::from([0.0, 1.0, 0.0]),
            Point::from([0.3, 0.3, -1.0]),
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
        let sliver = Interval::new(-1e-18, 1e-18).unwrap();
        let mut log = ProvenanceLog::new();
        let decision = orient3d_tolerantly(
            Point::from([0.0, 0.0, 0.0]),
            Point::from([1.0, 0.0, 0.0]),
            Point::from([0.0, 1.0, 0.0]),
            Point::Constructed([Interval::exactly(0.25), Interval::exactly(0.25), sliver]),
            &Policy::certified(),
            &mut log,
        );
        assert!(matches!(decision, Decision::Undecidable { .. }));
        assert_eq!(log.len(), 1);
        assert_eq!(log.records()[0].predicate(), ORIENT3D);
        assert_eq!(log.records()[0].inputs().len(), 4);
        assert_eq!(log.records()[0].level(), Level::Interval);
    }

    /// Every product here overflows, and the four points still make an
    /// ordinary tetrahedron. Bringing all twelve coordinates down settles it,
    /// and settles it as the same tetrahedron at unit size does.
    #[test]
    fn coordinates_big_enough_to_overflow_the_products_are_normalised_and_decided() {
        assert_eq!(
            orient3d(
                [1e300, 0.0, 0.0],
                [0.0, 1e300, 0.0],
                [0.0, 0.0, 1e300],
                [-1e300, -1e300, -1e300],
            ),
            Decision::Pos
        );
        assert_eq!(
            orient3d(
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [-1.0, -1.0, -1.0],
            ),
            Decision::Pos,
            "the same four points at a size f64 has no trouble with"
        );
    }

    /// And the other end of the range, where every product falls off the
    /// bottom instead.
    #[test]
    fn coordinates_small_enough_to_underflow_the_products_are_normalised_and_decided() {
        assert_eq!(
            orient3d_reporting_level(
                Point::from([1e-200, 0.0, 0.0]),
                Point::from([0.0, 1e-200, 0.0]),
                Point::from([0.0, 0.0, 1e-200]),
                Point::from([0.0, 0.0, 0.0]),
            ),
            (Decision::Pos, Level::Float)
        );
        assert_eq!(
            orient3d_reporting_level(
                Point::from([1.0, 0.0, 0.0]),
                Point::from([0.0, 1.0, 0.0]),
                Point::from([0.0, 0.0, 1.0]),
                Point::from([0.0, 0.0, 0.0]),
            ),
            (Decision::Pos, Level::Float),
            "the same four points at a size f64 has no trouble with"
        );
    }

    /// The claim [`decided_after_normalising`] rests on, executed rather than
    /// argued: an exact power of two multiplies level 1's value and its error
    /// bound by the same `2^(3k)`, so the comparison between them comes out the
    /// same way and `ORIENT3D_ERROR_BOUND` needs no adjusting.
    #[test]
    fn scaling_moves_level_one_and_its_error_bound_by_exactly_the_same_factor() {
        let mut checked = 0;
        for exponent in [-40, -7, 1, 13, 40] {
            for [a, b, c, d] in awkward_tetrahedra() {
                let here = Orient3d {
                    a: a.into(),
                    b: b.into(),
                    c: c.into(),
                    d: d.into(),
                }
                .on_floats()
                .expect("nothing is subnormal");
                let moved = Orient3d {
                    a: moved_by(a, exponent),
                    b: moved_by(b, exponent),
                    c: moved_by(c, exponent),
                    d: moved_by(d, exponent),
                }
                .on_floats()
                .expect("nothing is subnormal");

                let factor = 2f64.powi(3 * exponent);
                assert_eq!(moved.value, here.value * factor, "{a:?} {b:?} {c:?} {d:?}");
                assert_eq!(
                    moved.error_bound,
                    here.error_bound * factor,
                    "{a:?} {b:?} {c:?} {d:?}"
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 15);
    }

    fn moved_by(point: [f64; 3], exponent: i32) -> Point {
        Point::from(scaled_by_a_power_of_two(point, exponent).expect("nothing leaves the range"))
    }

    /// Coordinates with mantissas full enough that the products round, which is
    /// the only case where the claim above could come apart.
    fn awkward_tetrahedra() -> [[[f64; 3]; 4]; 3] {
        [
            [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.3, 0.3, -1.0],
            ],
            [
                [0.5, 0.25, 0.125],
                [3.0, -1.0, 2.0],
                [-4.0, 7.0, 0.5],
                [1.0, 1.0, 9.0],
            ],
            [
                [1.0 / 3.0, 7.0 / 9.0, -2.0 / 11.0],
                [2.0 / 7.0, 11.0 / 13.0, 5.0 / 17.0],
                [-5.0 / 3.0, 1.0 / 11.0, 3.0 / 7.0],
                [1.0 / 19.0, -4.0 / 23.0, 8.0 / 29.0],
            ],
        ]
    }

    /// The true determinant here is `-2^-76`, and level 1 used to certify it
    /// as positive.
    ///
    /// Work it out by hand. With `s = 2^-538` and `d` at the origin the rows
    /// are `a`, `b`, `c` themselves, and only two of the six products are
    /// non-zero, so
    ///
    /// ```text
    /// det = adz * (bdx*cdy - cdx*bdy) + cdz * (adx*bdy)
    ///     = 2^1000 * (11 - 9) * 2^-1076  +  2^462 * (-3 * 2^-538)
    ///     = 2^-75 - 3 * 2^-76
    ///     = -2^-76
    /// ```
    ///
    /// In `f64` the two products land in the subnormals: `11*2^-1076` is
    /// `2.75 * 2^-1074` and rounds to `3 * 2^-1074`, and `9*2^-1076` is
    /// `2.25 * 2^-1074` and rounds to `2 * 2^-1074`. Their difference comes
    /// out `1 * 2^-1074` where the truth is `0.5 * 2^-1074` — twice too big —
    /// so the first term becomes `2^-74` instead of `2^-75` and the sum flips
    /// to `+2^-76`. The permanent is built from those same rounded products,
    /// so the level 1 bound comes to a perfectly ordinary `2.4e-37` and
    /// happily certifies the wrong sign.
    ///
    /// What must never come back is [`Decision::Pos`]. Level 3 needs the
    /// roundoff term of each product to be normal and here the products
    /// themselves are not, so it declines, and the two cheap levels have
    /// nothing left to prove a sign with.
    ///
    /// The first two exponents of [`decided_after_normalising`] do not rescue
    /// it. The first is read off the largest difference, which is `2^1000`, so
    /// it would divide by about that much — and that sends `2^-538` below the
    /// smallest normal number, which the self-check refuses. The second takes
    /// the largest coordinate, the same `2^1000`, down to `2^338`, and is
    /// refused for the same reason. The third lifts instead: no product level 3
    /// forms is near the top, so it stops only where `2^1000` reaches the top
    /// binade, at `2^23`, and there the sign is the true one.
    #[test]
    fn a_determinant_whose_products_fall_off_the_bottom_of_the_range_is_lifted_to_its_true_sign() {
        let s = 2f64.powi(-538);
        let points = [
            [-3.0, 0.0, 2f64.powi(1000)],
            [11.0 * s, s, 0.0],
            [9.0 * s, s, 2f64.powi(462)],
            [0.0, 0.0, 0.0],
        ];
        assert!(refused_before_the_third_exponent(points));
        assert_eq!(exponents_to_try(points)[2], Some(23));
        let [a, b, c, d] = points;
        assert_eq!(orient3d(a, b, c, d), Decision::Neg);
    }

    /// The same determinant, rescaled until it fits, as an oracle for the sign
    /// the test above has to find.
    ///
    /// `d` is the origin, so the rows of the determinant are `b` and `c`
    /// themselves. Multiplying a row by a positive number multiplies the
    /// determinant by that same positive number, so the sign survives. `b`
    /// goes up by `2^600` and `c` by `2^300` — different factors because `c`
    /// already carries a `2^462` that must not overflow — which multiplies the
    /// determinant by `2^900` and leaves `-2^-76 * 2^900 = -2^824`.
    ///
    /// That scaling lifts both products clear of the subnormals, so every
    /// level of the cascade means what it says here, and the answer is
    /// [`Decision::Neg`].
    #[test]
    fn the_same_determinant_rescaled_to_fit_comes_out_negative() {
        let decision = orient3d(
            [-3.0, 0.0, 2f64.powi(1000)],
            [11.0 * 2f64.powi(62), 2f64.powi(62), 0.0],
            [9.0 * 2f64.powi(-238), 2f64.powi(-238), 2f64.powi(762)],
            [0.0, 0.0, 0.0],
        );
        assert_eq!(decision, Decision::Neg);
    }

    /// `HIGHEST_BINADE_LEVEL_THREE_TAKES`'s arithmetic, run rather than argued:
    /// every coordinate at the top of that binade, under every one of the 4096
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

    /// Of the 4096 ways to sign twelve coordinates all at the top of `binade`,
    /// how many overflow level 1 or level 3.
    fn sign_patterns_that_overflow(binade: i32) -> usize {
        let top = 2f64.powi(binade + 1).next_down();
        (0..1u32 << 12)
            .filter(|signs| {
                let coordinate = |bit: u32| if signs & (1 << bit) == 0 { top } else { -top };
                let point = |first: u32| [0, 1, 2].map(|axis| coordinate(first + axis));
                let [a, b, c, d] = [point(0), point(3), point(6), point(9)];
                let level_one = Orient3d {
                    a: a.into(),
                    b: b.into(),
                    c: c.into(),
                    d: d.into(),
                }
                .on_floats()
                .expect("no product this large is subnormal");
                let mut total = [0.0; DETERMINANT_ENTRIES];
                let level_three = exact_determinant(a, b, c, d, &mut total);
                !level_one.value.is_finite()
                    || !level_one.error_bound.is_finite()
                    || level_three
                        .is_none_or(|length| !total[..length].iter().all(|e| e.is_finite()))
            })
            .count()
    }

    /// A spike at the origin under a unit cube is decided on every turn it
    /// has, with the sign whole-number arithmetic gives, at `2^-700` and at
    /// `2^-994` (`docs/decisions.md` §324.7).
    ///
    /// The turns that need the third exponent are counted, so the test cannot
    /// pass by the first two settling everything: at `2^-700` they are the 84
    /// with three tiny corners that are not the origin, whose
    /// `tiny * tiny * tiny` the second exponent leaves under level 3's floor,
    /// and at `2^-994` there are 1920 of them.
    #[test]
    fn a_spike_at_the_origin_under_a_unit_cube_is_decided_on_every_turn() {
        let corners = a_spike_at_the_origin_under_a_unit_cube();
        for (t, needing_the_third) in [(2f64.powi(-700), 84), (2f64.powi(-994), 1920)] {
            assert!(t > 0.0 && t <= 2f64.powi(-41), "{t:e}");
            let mut only_the_third_exponent_settles = 0;
            for turn in every_turn(corners.len()) {
                let points = turn.map(|corner| corners[corner].at(t));
                let [a, b, c, d] = points.map(Point::from);
                assert_eq!(
                    orient3d_of_points(a, b, c, d),
                    the_sign_at_a_tiny_t(turn.map(|corner| corners[corner])),
                    "t = {t:e}, corners {turn:?}"
                );
                if refused_before_the_third_exponent(points) {
                    only_the_third_exponent_settles += 1;
                }
            }
            assert_eq!(
                only_the_third_exponent_settles, needing_the_third,
                "t = {t:e}"
            );
        }
    }

    /// Two tiny corners that are not the origin, `(t, 3t, 0)` and
    /// `(t, t, 2t)`, and two corners of the cube, at `t = 2^-995`: level 3
    /// needs their `t * t` lifted to `2^-969` or more, so by `2^511` or more,
    /// and `1 * 1` lifted by `2^512` is `2^1024`. So `2^511` is the only power
    /// of two that decides it.
    ///
    /// The third exponent stops one short, at `2^510`, because it reads
    /// binades: two factors in binade `511` could be as large as `2^1024`
    /// between them, and a minor of two such products overflows. Here the
    /// factors are exactly `2^511` and nothing overflows, but a rule that reads
    /// only binades cannot tell that. So this is a gap the rule leaves, pinned
    /// with the power that closes it.
    #[test]
    fn a_spike_2_to_the_minus_995_wide_is_refused_on_turns_only_2_to_the_511_decides() {
        let corners = a_spike_at_the_origin_under_a_unit_cube();
        let t = 2f64.powi(-995);
        let turn = [1, 3, 4, 5];
        let points = turn.map(|corner| corners[corner].at(t));
        let [a, b, c, d] = points;
        let expected = the_sign_at_a_tiny_t(turn.map(|corner| corners[corner]));
        assert!(expected.is_certified());

        assert_eq!(exponents_to_try(points)[2], Some(510));
        assert!(
            !orient3d(a, b, c, d).is_certified(),
            "the third exponent reaches this now, so the gap below is closed"
        );
        let decided_by: Vec<i32> = (-1100..=1100)
            .filter(|&exponent| {
                let [a, b, c, d] = points.map(|point| scaled_by_a_power_of_two(point, exponent));
                let (Some(a), Some(b), Some(c), Some(d)) = (a, b, c, d) else {
                    return false;
                };
                let decision = orient3d(a, b, c, d);
                assert!(
                    !decision.is_certified() || decision == expected,
                    "2^{exponent} gave {decision:?}"
                );
                decision.is_certified()
            })
            .collect();
        assert_eq!(decided_by, [511]);
    }

    /// The same two tiny corners and two corners of the cube, at `2^-1000` and
    /// at the smallest `f64`, are a limit and not a gap: no power of two that
    /// moves all four points exactly decides them, because lifting `t * t` to
    /// level 3's floor takes `1 * 1` past the top of the range.
    #[test]
    fn below_2_to_the_minus_995_no_power_of_two_decides_those_turns() {
        let corners = a_spike_at_the_origin_under_a_unit_cube();
        for t in [2f64.powi(-1000), f64::from_bits(1)] {
            let points = [1, 3, 4, 5].map(|corner| corners[corner].at(t));
            let mut tried = 0;
            for exponent in -1100..=1100 {
                let [a, b, c, d] = points.map(|point| scaled_by_a_power_of_two(point, exponent));
                let (Some(a), Some(b), Some(c), Some(d)) = (a, b, c, d) else {
                    continue;
                };
                tried += 1;
                assert!(
                    !orient3d(a, b, c, d).is_certified(),
                    "2^{exponent} settled what this says nothing settles, at t = {t:e}"
                );
            }
            assert!(tried > 900, "only {tried} exponents scaled exactly at all");
        }
    }

    fn every_turn(corners: usize) -> impl Iterator<Item = [usize; 4]> {
        (0..corners.pow(4))
            .map(move |index| [0, 1, 2, 3].map(|place| index / corners.pow(place) % corners))
            .filter(|turn| (0..4).all(|i| (i + 1..4).all(|j| turn[i] != turn[j])))
    }

    fn refused_before_the_third_exponent(points: [[f64; 3]; 4]) -> bool {
        let [a, b, c, d] = points;
        let [first, second, _] = exponents_to_try(points);
        !decide(&Orient3d {
            a: a.into(),
            b: b.into(),
            c: c.into(),
            d: d.into(),
        })
        .0
        .is_certified()
            && each_once_and_never_zero([first, second])
                .all(|exponent| decided_at_scale(a, b, c, d, exponent).is_none())
    }

    /// A corner whose coordinates are `unit + tiny * t`, as whole numbers, and
    /// never both on one axis, so that each is an `f64` for every `t`.
    #[derive(Clone, Copy)]
    struct SpikeCorner {
        unit: [i32; 3],
        tiny: [i32; 3],
    }

    impl SpikeCorner {
        fn at(self, t: f64) -> [f64; 3] {
            [0, 1, 2].map(|axis| {
                let (unit, tiny) = (self.unit[axis], self.tiny[axis]);
                assert!(unit == 0 || tiny == 0, "{unit} + {tiny} t is not an f64");
                f64::from(unit) + f64::from(tiny) * t
            })
        }
    }

    /// The corners of `[-1, 1]^3` and four tiny points at the origin, one of
    /// them the origin itself.
    fn a_spike_at_the_origin_under_a_unit_cube() -> [SpikeCorner; 12] {
        let tiny = |tiny| SpikeCorner {
            unit: [0, 0, 0],
            tiny,
        };
        let unit = |unit| SpikeCorner {
            unit,
            tiny: [0, 0, 0],
        };
        [
            tiny([0, 0, 0]),
            tiny([1, 3, 0]),
            tiny([2, 0, 1]),
            tiny([1, 1, 2]),
            unit([-1, -1, -1]),
            unit([-1, -1, 1]),
            unit([-1, 1, -1]),
            unit([-1, 1, 1]),
            unit([1, -1, -1]),
            unit([1, -1, 1]),
            unit([1, 1, -1]),
            unit([1, 1, 1]),
        ]
    }

    /// The sign of `det[a - d, b - d, c - d]` for a positive `t` too small to
    /// matter against whole numbers.
    ///
    /// The determinant is `A + B t + C t^2 + D t^3` with whole `A` to `D`. The
    /// first of them that is not zero is at least `1` times its power of `t`,
    /// and while all four are below `2^40` the rest come to less than `2^41`
    /// times the next power, so for `t <= 2^-41` the first one's sign is the
    /// determinant's.
    fn the_sign_at_a_tiny_t(corners: [SpikeCorner; 4]) -> Decision {
        let [a, b, c, d] = corners;
        let row = |p: SpikeCorner| {
            [0, 1, 2].map(|axis| {
                [
                    i64::from(p.unit[axis] - d.unit[axis]),
                    i64::from(p.tiny[axis] - d.tiny[axis]),
                    0,
                    0,
                ]
            })
        };
        let [r, s, u] = [row(a), row(b), row(c)];
        let minor = |i: usize, j: usize| minus(times(s[i], u[j]), times(s[j], u[i]));
        let coefficients = plus(
            minus(times(r[0], minor(1, 2)), times(r[1], minor(0, 2))),
            times(r[2], minor(0, 1)),
        );
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

    /// Two polynomials in `t` of degree at most three, multiplied, where the
    /// product has to stay within degree three.
    fn times(p: [i64; 4], q: [i64; 4]) -> [i64; 4] {
        let mut product = [0; 4];
        for (i, &x) in p.iter().enumerate() {
            for (j, &y) in q.iter().enumerate() {
                if x * y != 0 {
                    assert!(i + j < 4, "a determinant of three rows is a cubic");
                    product[i + j] += x * y;
                }
            }
        }
        product
    }

    fn plus(p: [i64; 4], q: [i64; 4]) -> [i64; 4] {
        [0, 1, 2, 3].map(|power| p[power] + q[power])
    }

    fn minus(p: [i64; 4], q: [i64; 4]) -> [i64; 4] {
        [0, 1, 2, 3].map(|power| p[power] - q[power])
    }

    /// The two ceilings' arithmetic, run rather than argued: every x, y and z
    /// at the top of a binade of its own, with the x and y binades adding up
    /// to [`LARGEST_SUM_OF_TWO_FACTOR_BINADES`] and all three to
    /// [`LARGEST_SUM_OF_THREE_FACTOR_BINADES`], under all 4096 ways to sign
    /// the twelve, and level 3 overflows nothing. One more on either sum and
    /// some of them do.
    #[test]
    fn the_highest_sums_of_factor_binades_overflow_nothing_in_level_three() {
        let (two, three) = (
            LARGEST_SUM_OF_TWO_FACTOR_BINADES,
            LARGEST_SUM_OF_THREE_FACTOR_BINADES,
        );
        let x = two / 2;
        assert_eq!(
            sign_patterns_that_overflow_level_three(x, two - x, three - two),
            0
        );
        assert!(sign_patterns_that_overflow_level_three(x, two + 1 - x, three - two - 1) > 0);
        assert!(sign_patterns_that_overflow_level_three(x, two - x, three + 1 - two) > 0);
    }

    /// At the third exponent levels 1 and 2 can overflow, and that has to cost
    /// them their answer and nothing else. Every turn of the spike above that
    /// the third exponent is what settles, scaled by that exponent, gets level
    /// 3's answer from the whole cascade, and that answer is the whole-number
    /// one — while level 1's bound and level 2's interval run off the top on
    /// some of them.
    ///
    /// `orient3d` on the turn unscaled gets checked against the same
    /// `exactly`, so this fails if `orient3d` ever stops trying the third
    /// exponent at its own call site. Scaling the points by hand and calling
    /// `decide` on the result, as the rest of this test does, would not
    /// notice that at all.
    #[test]
    fn levels_one_and_two_overflowing_at_the_third_exponent_still_agree_with_level_three() {
        let corners = a_spike_at_the_origin_under_a_unit_cube();
        let (mut level_one_overflowed, mut level_two_overflowed) = (0, 0);
        for t in [2f64.powi(-700), 2f64.powi(-994)] {
            for turn in every_turn(corners.len()) {
                let points = turn.map(|corner| corners[corner].at(t));
                if !refused_before_the_third_exponent(points) {
                    continue;
                }
                let third = exponents_to_try(points)[2].expect("a corner is not zero");
                let [a, b, c, d] = points.map(|point| {
                    scaled_by_a_power_of_two(point, third)
                        .expect("the third exponent keeps every coordinate an f64")
                });
                let exactly = orient3d_exactly(a, b, c, d);
                assert_eq!(
                    exactly,
                    the_sign_at_a_tiny_t(turn.map(|corner| corners[corner]))
                );
                let [unscaled_a, unscaled_b, unscaled_c, unscaled_d] = points;
                assert_eq!(
                    orient3d(unscaled_a, unscaled_b, unscaled_c, unscaled_d),
                    exactly,
                    "corners {turn:?}, t = {t:e}"
                );
                let predicate = Orient3d {
                    a: a.into(),
                    b: b.into(),
                    c: c.into(),
                    d: d.into(),
                };
                assert_eq!(decide(&predicate).0, exactly, "corners {turn:?}");

                if predicate
                    .on_floats()
                    .is_some_and(|level_one| !level_one.error_bound.is_finite())
                {
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
    }

    /// Of the 4096 ways to sign twelve coordinates, with every x at the top of
    /// `x_binade`, every y at the top of `y_binade` and every z at the top of
    /// `z_binade`, how many overflow level 3.
    fn sign_patterns_that_overflow_level_three(
        x_binade: i32,
        y_binade: i32,
        z_binade: i32,
    ) -> usize {
        let top_of = |binade: i32| 2f64.powi(binade + 1).next_down();
        let magnitudes = [top_of(x_binade), top_of(y_binade), top_of(z_binade)];
        (0..1u32 << 12)
            .filter(|&signs| {
                let signed = |bit: u32, magnitude: f64| {
                    if signs & (1 << bit) == 0 {
                        magnitude
                    } else {
                        -magnitude
                    }
                };
                let [a, b, c, d] = [0, 1, 2, 3].map(|point| {
                    [0, 1, 2].map(|axis| signed(3 * point + axis, magnitudes[axis as usize]))
                });
                let mut total = [0.0; DETERMINANT_ENTRIES];
                exact_determinant(a, b, c, d, &mut total)
                    .is_none_or(|length| !total[..length].iter().all(|e| e.is_finite()))
            })
            .count()
    }

    /// The `x` of the first point is not a number, so there is no first point,
    /// and there is no fact about which side of a plane it falls on.
    ///
    /// Before the guard this came back [`Decision::Neg`] at
    /// [`Level::Interval`]: the not-a-number arrives at level 2 as
    /// [`Interval::ENTIRE`], both places it appears are multiplied by zero, and
    /// what survives is `-1`. Every other axis of the same point is checked
    /// too, so a guard that only looked at one coordinate would fail here.
    #[test]
    fn a_point_with_a_coordinate_that_is_not_a_number_gets_no_answer() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for axis in 0..3 {
                let mut broken = [0.0, 1.0, 0.0];
                broken[axis] = bad;
                assert_eq!(
                    orient3d_reporting_level(
                        Point::from(broken),
                        Point::from([1.0, 0.0, 1.0]),
                        Point::from([2.0, 0.0, 3.0]),
                        Point::from([0.0, 0.0, 0.0]),
                    ),
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

    /// Whichever of the four it is, and however far down the argument list.
    #[test]
    fn any_of_the_four_points_being_no_point_stops_the_answer() {
        let good = [
            Point::from([0.0, 1.0, 0.0]),
            Point::from([1.0, 0.0, 1.0]),
            Point::from([2.0, 0.0, 3.0]),
            Point::from([0.0, 0.0, 0.0]),
        ];
        for spoiled in 0..4 {
            let mut points = good;
            points[spoiled] = Point::from([f64::NAN, 0.0, 0.0]);
            assert!(
                !orient3d_of_points(points[0], points[1], points[2], points[3]).is_certified(),
                "point {spoiled}"
            );
        }
        assert!(orient3d_of_points(good[0], good[1], good[2], good[3]).is_certified());
    }

    /// The tolerant entry point must not launder the refusal into a `Zero`.
    /// An enclosure of infinity is wider than any working size a policy can
    /// carry, so the fallback has nothing to fall back on and says so.
    #[test]
    fn a_tolerant_policy_still_refuses_a_point_that_is_not_a_point() {
        let mut log = ProvenanceLog::new();
        let decision = orient3d_tolerantly(
            Point::from([f64::NAN, 1.0, 0.0]),
            Point::from([1.0, 0.0, 1.0]),
            Point::from([2.0, 0.0, 3.0]),
            Point::from([0.0, 0.0, 0.0]),
            &Policy::tolerant(1.0),
            &mut log,
        );
        assert!(!decision.is_certified());
        assert!(!log.is_empty(), "a refusal has to leave a record behind");
    }

    /// A box is a set of real numbers however wide it is, so a sign proved over
    /// the whole of it is proved for every point in it. That is a theorem, not
    /// the defect above, and the guard leaves it alone.
    #[test]
    fn a_constructed_point_whose_box_is_everything_can_still_be_placed() {
        let (decision, level) = orient3d_reporting_level(
            Point::Constructed([
                Interval::ENTIRE,
                Interval::exactly(1.0),
                Interval::exactly(0.0),
            ]),
            Point::from([1.0, 0.0, 1.0]),
            Point::from([2.0, 0.0, 3.0]),
            Point::from([0.0, 0.0, 0.0]),
        );
        assert_eq!((decision, level), (Decision::Neg, Level::Interval));
    }

    /// Level 1's `value` against its error bound, measured on four tetrahedra
    /// a search found rounding badly. This is a tripwire on the constant, not a
    /// bound that holds on every input (decisions.md §16.3, §18.5): the
    /// constant bounds the sum `value` is rounded from, and
    /// `level_one_can_land_past_its_error_bound_and_still_certify_the_right_sign`
    /// builds a tetrahedron whose `value` lands past it.
    ///
    /// **The worst found comes to 6.29 epsilon of the permanent**, against the
    /// 7 the constant carries. So this fails as soon as the constant is cut
    /// below about 6.29 epsilon, and it says nothing at all between there and
    /// 7: a bound of `6.5*epsilon` would be false and everything here would
    /// still pass. The derivation is what makes the constant a bound; this is
    /// what stops [`Orient3d::on_floats`] drifting out from under the
    /// derivation (decisions.md §15.6, §16.3).
    ///
    /// The tetrahedra came from random coordinates followed by hill climbing on
    /// that ratio, over some hundred million candidates, and the best of them
    /// would not improve under a further six million. The best it found is not
    /// a near-degenerate one — cancellation shrinks the error and the permanent
    /// together — but an ordinary tetrahedron whose twelve coordinates happen
    /// to round the same way step after step. The one built to round worse, to
    /// 8 epsilon, is an ordinary tetrahedron too.
    #[test]
    fn level_one_stays_inside_its_error_bound_on_the_tetrahedra_a_search_found() {
        let mut worst = 0.0f64;
        for points in tetrahedra_a_search_found_rounding_badly() {
            let (error, filtered) = how_far_level_one_falls_from_the_truth(points)
                .expect("no product here leaves the range");
            assert!(
                error.abs() <= filtered.error_bound,
                "level 1 came out {error:e} from the truth on {points:?}, past the {:e} \
                 it promised",
                filtered.error_bound
            );
            let permanent = filtered.error_bound / ORIENT3D_ERROR_BOUND;
            worst = worst.max(error.abs() / permanent / (f64::EPSILON / 2.0));
        }
        assert!(
            worst > 6.25,
            "these used to reach 6.29 epsilon of the permanent and now reach only {worst}, \
             so the net has gone slack and the search wants running again"
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
    fn the_searched_tetrahedra_are_settled_by_level_one_itself() {
        for points in tetrahedra_a_search_found_rounding_badly() {
            let [a, b, c, d] = points;
            let (decision, level) = decide(&Orient3d {
                a: a.into(),
                b: b.into(),
                c: c.into(),
                d: d.into(),
            });
            assert!(decision.is_certified(), "{points:?} was not settled at all");
            assert_eq!(level, Level::Float, "{points:?}");
        }
    }

    /// `value`, the double level 1 compares, can land further from the true
    /// determinant than `error_bound`, and the sign level 1 certifies from it
    /// is still right.
    ///
    /// `ORIENT3D_ERROR_BOUND` bounds the sum `value` is rounded from, which is
    /// at most seven roundings from the inputs; `value` is an eighth.
    /// This tetrahedron is built so that all eight go the same way by nearly a
    /// whole `epsilon`, along the one term that is nearly all of the
    /// permanent: `adz`, `bdx` and `cdy`, the product `bdx * cdy`, the minor
    /// `first`, `adz * first`, the sum of the first two terms, and `value`.
    /// That puts `value` 8 epsilon of the permanent from the truth where the
    /// constant carries 7. The sign holds because `error_bound` is a double:
    /// rounding cannot carry the sum past it unless the sum was past it
    /// already (decisions.md §464.7).
    #[test]
    fn level_one_can_land_past_its_error_bound_and_still_certify_the_right_sign() {
        let a = [7.450580596923828e-9, 1.0, 1.0000000049670539];
        let b = [1.0000000149011614, 1.0, -1.4901160971803051e-8];
        let c = [
            -8.478601278668534e-22,
            1.0000000074505806,
            -8.231416447182961e-25,
        ];
        let d = [1.1102230163533504e-16; 3];

        let (error, filtered) = how_far_level_one_falls_from_the_truth([a, b, c, d])
            .expect("no product here leaves the range");
        assert!(
            error.abs() > filtered.error_bound,
            "level 1 came out {error:e} from the truth, inside the {:e} it carries, so this \
             no longer shows what it is for",
            filtered.error_bound
        );
        let permanent = filtered.error_bound / ORIENT3D_ERROR_BOUND;
        let in_epsilon = error.abs() / permanent / (f64::EPSILON / 2.0);
        assert!(
            in_epsilon > 7.99,
            "this used to land 8 epsilon of the permanent from the truth and now lands \
             {in_epsilon}"
        );

        let decided = decide(&Orient3d {
            a: a.into(),
            b: b.into(),
            c: c.into(),
            d: d.into(),
        });
        assert_eq!(decided, (Decision::Pos, Level::Float));
    }

    /// How far level 1's `value` falls from the true determinant, and the bound
    /// it promised to stay inside.
    ///
    /// The truth comes from [`exact_determinant`], which is level 3's own
    /// arithmetic and throws nothing away, so subtracting `value` from it in
    /// that same arithmetic leaves the rounding error and nothing else. Only
    /// the final `sum` rounds, and it rounds a quantity the callers compare
    /// against a bound many orders of magnitude larger.
    ///
    /// `None` when a product left the range, where a relative bound claims
    /// nothing and level 1 declines to run anyway.
    fn how_far_level_one_falls_from_the_truth(points: [[f64; 3]; 4]) -> Option<(f64, Filtered)> {
        let [a, b, c, d] = points;
        let filtered = Orient3d {
            a: a.into(),
            b: b.into(),
            c: c.into(),
            d: d.into(),
        }
        .on_floats()?;

        let mut determinant = [0.0f64; DETERMINANT_ENTRIES];
        let length = exact_determinant(a, b, c, d, &mut determinant)?;
        let mut error = [0.0f64; DETERMINANT_ENTRIES + 1];
        let error_length = expansion_sum(&determinant[..length], &[-filtered.value], &mut error);

        Some((error[..error_length].iter().sum(), filtered))
    }

    /// The four the search kept coming back to, worst first. Their errors are
    /// 6.29, 6.15, 5.99 and 5.60 epsilon of the permanent.
    ///
    /// They are kept as literals because that is what makes this a test rather
    /// than a second search: the coordinates are the finding, and re-deriving
    /// them on every run would cost minutes and prove nothing.
    fn tetrahedra_a_search_found_rounding_badly() -> [[[f64; 3]; 4]; 4] {
        [
            [
                [-0.01787677771316376, 215.14081658155717, 2.2116568307902695],
                [1.5854223489664379, -26.608260632339537, 216.38579693428494],
                [82.33595501635082, 43.210471131708566, 4.62763999917739],
                [
                    -1.9079816382710177,
                    -21.061093436043066,
                    -0.9008993180197393,
                ],
            ],
            [
                [
                    -0.00012008530581342215,
                    8.793630593117996e-5,
                    7.597255455493723e-5,
                ],
                [
                    -68130.92238356236,
                    0.00039377780730831405,
                    -1.7605642836007228e-6,
                ],
                [
                    0.13751870201048225,
                    -306398.9613431073,
                    0.010549140241096213,
                ],
                [-6.324454820867686, 1905.829219415685, -1804.7603652528232],
            ],
            [
                [
                    0.24257214066419674,
                    -2.427681550466581e-5,
                    0.0004490449705293907,
                ],
                [
                    1.65255177808632e-6,
                    -0.0018356836401014578,
                    -297.8742145146856,
                ],
                [-0.0001490787451421975, 987948.3441143868, 294731.1906442909],
                [
                    -8.651997195832852,
                    1.2201615793518594e-5,
                    -0.00036001088861326025,
                ],
            ],
            [
                [1.1599817779136417, -1.7744964357133617, -1.065988351401675],
                [-1.5668515693620335, -1.7966017740217959, 1.0353304930432294],
                [-1.6301099710905995, 1.254301350558929, 1.5853495175344006],
                [-1.9223785638829012, -1.9601812446295237, -1.162024325924673],
            ],
        ]
    }
}
