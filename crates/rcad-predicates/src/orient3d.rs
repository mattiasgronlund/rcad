use crate::expansion::{
    expansion_sign, expansion_sum, product_is_exact, scale_expansion, two_product,
};
use crate::interval::determinant;
use crate::{
    Decision, Filtered, Interval, Level, Measures, Point, Policy, Predicate, ProvenanceLog, Record,
    decide, product_is_normal, resolve_under_policy,
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
/// Exact `f64` inputs always get a certified answer, as long as the products
/// of coordinates stay inside the normal `f64` range. If one overflows, or if
/// one is small enough that its roundoff term would be subnormal, exact
/// arithmetic stops being exact and the answer is [`Decision::Undecidable`]
/// rather than a guess. Coordinates that far apart in magnitude are what
/// decisions.md §3.5's dynamic-range check exists to reject.
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
pub fn orient3d_reporting_level(a: Point, b: Point, c: Point, d: Point) -> (Decision, Level) {
    decide(&Orient3d { a, b, c, d })
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
    if decision.is_certified() {
        return decision;
    }
    let record = Record::new(ORIENT3D, &[a, b, c, d], level, decision);
    resolve_under_policy(record, Measures::Volume, policy, log)
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
        // anything while every product keeps its relative accuracy. One of them
        // in the subnormals and this level certifies a wrong sign.
        //
        // The permanent's own three products are each at least as large as the
        // matching one here — `|x| + |y|` is never below `|x - y|` — so
        // checking these nine covers the permanent too.
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
/// Where the 7 comes from — walk the longest chain of roundings between an
/// input coordinate and `value` in [`Orient3d::on_floats`]. Each difference
/// (`bdx = b.x - d.x`) is one rounding. Each product inside a 2x2 minor
/// (`bdx * cdy`) is a second, and it multiplies two already-rounded
/// differences, so it carries two more. The minor's own subtraction is a
/// fourth, multiplying by the third rounded difference `adz` is a fifth and
/// sixth, and folding the three terms together with two additions is the
/// seventh. Seven roundings, each costing at most a factor of `(1 + epsilon)`,
/// give `(1 + epsilon)^7 - 1 <= 7*epsilon + 21*epsilon^2 + ...` times the
/// permanent. The `56*epsilon^2` covers those higher-order terms together with
/// the rounding of the permanent itself.
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
    let cannot_be_done = Decision::Undecidable {
        enclosure: f64::INFINITY,
    };
    let (Some((ab, ab_len)), Some((ac, ac_len)), Some((ad, ad_len))) =
        (minor(a, b), minor(a, c), minor(a, d))
    else {
        return cannot_be_done;
    };
    let (Some((bc, bc_len)), Some((bd, bd_len)), Some((cd, cd_len))) =
        (minor(b, c), minor(b, d), minor(c, d))
    else {
        return cannot_be_done;
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

    const CAPACITY: usize = 96;
    let mut total = [0.0f64; CAPACITY];
    let mut total_len = 0;
    let mut merged = [0.0f64; CAPACITY];
    let mut term = [0.0f64; 8];

    for (two_by_two, scale) in terms {
        if !two_by_two
            .iter()
            .all(|&entry| product_is_exact(entry, scale))
        {
            return cannot_be_done;
        }
        let term_len = scale_expansion(two_by_two, scale, &mut term);
        total_len = expansion_sum(&total[..total_len], &term[..term_len], &mut merged);
        core::mem::swap(&mut total, &mut merged);
    }

    expansion_sign(&total[..total_len])
}

/// `u.x * v.y - u.y * v.x`, exactly, or `None` if a product underflowed.
fn minor(u: [f64; 3], v: [f64; 3]) -> Option<([f64; 4], usize)> {
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
        ORIENT3D, orient3d, orient3d_exactly, orient3d_reporting_level, orient3d_tolerantly,
    };
    use crate::{Decision, Interval, Level, Point, Policy, ProvenanceLog};

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

    #[test]
    fn coordinates_big_enough_to_overflow_the_products_are_undecidable() {
        let decision = orient3d(
            [1e300, 0.0, 0.0],
            [0.0, 1e300, 0.0],
            [0.0, 0.0, 1e300],
            [-1e300, -1e300, -1e300],
        );
        assert_eq!(
            decision,
            Decision::Undecidable {
                enclosure: f64::INFINITY
            }
        );
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
    /// Nothing in `f64` can prove the true sign — level 3 needs the roundoff
    /// term of each product to be normal, and here the products themselves are
    /// not — so [`Decision::Undecidable`] is the whole of the right answer.
    #[test]
    fn a_determinant_whose_products_fall_off_the_bottom_of_the_range_is_not_certified() {
        let s = 2f64.powi(-538);
        let decision = orient3d(
            [-3.0, 0.0, 2f64.powi(1000)],
            [11.0 * s, s, 0.0],
            [9.0 * s, s, 2f64.powi(462)],
            [0.0, 0.0, 0.0],
        );
        match decision {
            Decision::Pos => panic!("the true determinant is -2^-76, so this is a wrong sign"),
            Decision::Neg | Decision::Zero => panic!("nothing here can prove a sign: {decision:?}"),
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
        }
    }

    /// The same determinant, rescaled until it fits, as an oracle for the sign
    /// the test above cannot prove.
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
}
