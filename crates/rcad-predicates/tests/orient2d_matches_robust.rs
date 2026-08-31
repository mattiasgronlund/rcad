//! `orient2d` against the `robust` crate, which is always exact.
//!
//! `robust` fuses the float filter and exact arithmetic internally and never
//! comes back undecided, so for exact `f64` inputs our answer must equal its
//! sign on every single input. Zero disagreements, or the cascade is wrong.
//!
//! The one place the two are allowed to differ is down among the subnormals,
//! where `robust` omits the underflow guard and answers confidently anyway.
//! That case has a test of its own at the bottom.

use proptest::prelude::*;
use rcad_predicates::{Decision, Level, orient2d_reporting_level};

/// Checks one triple against `robust` and reports which level answered.
///
/// An exact `f64` input that comes back undecided is a failure in itself:
/// level 3 always decides as long as the products stay in the normal `f64`
/// range, and every case here keeps them there.
fn agrees_with_robust(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Level {
    let expected = robust_decision(a, b, c);
    let (ours, level) = orient2d_reporting_level(a, b, c);

    match ours {
        Decision::Neg | Decision::Zero | Decision::Pos => assert_eq!(
            ours, expected,
            "disagreed with robust on {a:?} {b:?} {c:?} at {level:?}"
        ),
        Decision::Undecidable { enclosure } => panic!(
            "exact f64 inputs must always be decided, but {a:?} {b:?} {c:?} \
             came back undecided with an enclosure {enclosure} wide"
        ),
        other => panic!("unhandled decision {other:?}"),
    }
    level
}

fn robust_decision(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Decision {
    let value = robust::orient2d(coord(a), coord(b), coord(c));
    if value > 0.0 {
        Decision::Pos
    } else if value < 0.0 {
        Decision::Neg
    } else {
        Decision::Zero
    }
}

fn coord(p: [f64; 2]) -> robust::Coord<f64> {
    robust::Coord { x: p[0], y: p[1] }
}

/// Points on the line `3x + 5y = 0`. Whole-number multiples keep both
/// coordinates exact, so the points stay exactly collinear however large the
/// multiples get.
fn on_a_shared_line(along: f64) -> [f64; 2] {
    [5.0 * along, -3.0 * along]
}

/// Moves one coordinate by `ulps` representable steps.
///
/// Goes through the same sign-magnitude to two's-complement flip that
/// `f64::total_cmp` uses, so stepping works across zero and through the
/// negatives instead of landing on a NaN.
fn nudged(point: [f64; 2], axis: usize, ulps: i64) -> [f64; 2] {
    let mut moved = point;
    moved[axis] = from_in_order_position(in_order_position(point[axis]) + ulps);
    assert!(
        moved[axis].is_finite(),
        "nudged off the end of the f64 range"
    );
    moved
}

fn in_order_position(value: f64) -> i64 {
    let bits = value.to_bits() as i64;
    bits ^ (((bits >> 63) as u64) >> 1) as i64
}

fn from_in_order_position(position: i64) -> f64 {
    let bits = position ^ (((position >> 63) as u64) >> 1) as i64;
    f64::from_bits(bits as u64)
}

type ThreePoints = ([f64; 2], [f64; 2], [f64; 2]);

/// Triples chosen to break things: exactly collinear, one ulp off collinear,
/// coincident, spread over sixteen orders of magnitude, and large enough that
/// the products are within a few decades of overflowing.
fn the_awkward_cases() -> Vec<ThreePoints> {
    let line = on_a_shared_line;
    let mut cases = vec![
        // The simplest anticlockwise corner there is.
        ([0.0, 0.0], [1.0, 0.0], [0.0, 1.0]),
        // Exactly collinear on a line that is not axis-aligned, spread far
        // enough apart that the products no longer fit in an f64.
        (line(1.0), line(0.0), line(1000000007.0)),
        (line(1000003.0), line(999983.0), line(2147483647.0)),
        (line(-1234567.0), line(1048576.0), line(999999999.0)),
        // Collinear and axis-aligned, where the differences cancel exactly.
        ([0.0, 0.0], [1.0, 2.0], [2.0, 4.0]),
        ([1.5, 1.5], [-1.5, -1.5], [3.0, 3.0]),
        // The same point more than once.
        ([0.25, -0.5], [1.0, 2.0], [0.25, -0.5]),
        ([0.25, -0.5], [0.25, -0.5], [4.0, 5.0]),
        ([1.0, 1.0], [1.0, 1.0], [1.0, 1.0]),
        // Sixteen orders of magnitude inside one triple.
        ([1e-8, 0.0], [0.0, 1e8], [1e8, 1e-8]),
        ([1e8, 1e-8], [1e-8, 1e8], [0.0, 0.0]),
        // Large enough that the products reach 1e300 and stop a few decades
        // short of overflowing.
        ([1e150, 0.0], [0.0, 1e150], [-1e150, -1e150]),
        ([1e150, 1e150], [-1e150, 1e150], [1e-150, 1e-150]),
    ];

    // Each case again with one coordinate moved by a handful of ulps in each
    // direction. This is where a filter that trusts itself too far gets the
    // sign backwards.
    //
    // Nudging a zero lands in the subnormals, where expansion arithmetic is no
    // longer exact and `robust` is no longer a valid oracle; that case has a
    // test of its own below.
    let unnudged = cases.clone();
    for (a, b, c) in unnudged {
        for axis in 0..2 {
            for ulps in [-8i64, -2, -1, 1, 2, 8] {
                let moved_c = nudged(c, axis, ulps);
                if is_normal_or_zero(moved_c[axis]) {
                    cases.push((a, b, moved_c));
                }
                let moved_b = nudged(b, axis, ulps);
                if is_normal_or_zero(moved_b[axis]) {
                    cases.push((a, moved_b, c));
                }
            }
        }
    }
    cases
}

fn is_normal_or_zero(value: f64) -> bool {
    value == 0.0 || value.is_normal()
}

#[test]
fn every_awkward_case_agrees_with_robust() {
    let cases = the_awkward_cases();
    assert_eq!(cases.len(), 285);
    for (a, b, c) in cases {
        agrees_with_robust(a, b, c);
    }
}

/// The same three points, with the last one pushed off the shared line a little
/// further each time. Every level of the cascade has to answer somewhere on
/// this ladder, or the two cheap levels are dead code and the test suite is
/// only ever exercising exact arithmetic.
#[test]
fn pushing_a_point_further_off_the_line_walks_back_down_the_cascade() {
    let a = on_a_shared_line(1000003.0);
    let b = on_a_shared_line(999983.0);
    let on_the_line = on_a_shared_line(2147483647.0);

    let level_for = |ulps| agrees_with_robust(a, b, nudged(on_the_line, 1, ulps));

    assert_eq!(
        level_for(0),
        Level::Exact,
        "only exact arithmetic can prove a determinant is zero once the \
         products start rounding"
    );
    assert_eq!(level_for(1), Level::Exact);
    assert_eq!(level_for(-1), Level::Exact);
    assert_eq!(
        level_for(1 << 28),
        Level::Interval,
        "the interval is sharper here than the static error bound is"
    );
    assert_eq!(level_for(-(1 << 28)), Level::Interval);
    assert_eq!(
        level_for(1 << 29),
        Level::Float,
        "well clear of the line, the error bound settles it on its own"
    );
    assert_eq!(level_for(-(1 << 29)), Level::Float);
}

/// `robust` leaves out the underflow guard on purpose, so on a triple whose
/// determinant is a positive number smaller than the smallest `f64` it answers
/// zero — that is, it says three points that make a corner are in a line.
///
/// This is where the two implementations part company for good, and it is the
/// one place in this file where matching `robust` would be the wrong outcome.
/// The corner is a perfectly ordinary right angle that happens to be written
/// down in tiny numbers, so its sign is a fact: lift all six coordinates by an
/// exact power of two and every level of the cascade can see it.
#[test]
fn coordinates_down_among_the_subnormals_are_normalised_rather_than_guessed() {
    let a = [1e-200, 0.0];
    let b = [0.0, 1e-200];
    let c = [0.0, 0.0];

    assert_eq!(
        robust::orient2d(coord(a), coord(b), coord(c)),
        0.0,
        "the case is only interesting because robust gets it wrong"
    );

    let (ours, level) = orient2d_reporting_level(a, b, c);
    match ours {
        Decision::Pos => (),
        Decision::Zero => panic!("these three points are not in a line"),
        Decision::Neg => panic!("they turn the same way as the same corner at unit size"),
        Decision::Undecidable { .. } => panic!("the sign exists and a power of two reaches it"),
        other => panic!("unhandled decision {other:?}"),
    }
    assert_eq!(
        level,
        Level::Float,
        "once it is in range this is the cheapest kind of turn there is"
    );
    assert_eq!(
        orient2d_reporting_level([1.0, 0.0], [0.0, 1.0], [0.0, 0.0]),
        (ours, level),
        "the same corner at a size f64 has no trouble with"
    );
}

/// One coordinate. Mixes exact small values, where degeneracies actually
/// happen, with full-mantissa values over a wide span of exponents.
fn a_coordinate() -> impl Strategy<Value = f64> {
    prop_oneof![
        (-8i32..=8).prop_map(f64::from),
        (-1_000_000i32..=1_000_000).prop_map(|whole| f64::from(whole) / 64.0),
        (-1.0f64..1.0, -60i32..60).prop_map(|(mantissa, exponent)| mantissa * 2f64.powi(exponent)),
        (-1.0f64..1.0, -8i32..=8).prop_map(|(mantissa, exponent)| mantissa * 10f64.powi(exponent)),
    ]
}

fn a_point() -> impl Strategy<Value = [f64; 2]> {
    [a_coordinate(), a_coordinate()]
}

/// A triple that is exactly collinear however large the multiples get, so the
/// generator actually reaches the degenerate case instead of missing it with
/// probability one.
fn a_collinear_triple() -> impl Strategy<Value = [[f64; 2]; 3]> {
    proptest::array::uniform3(-2_000_000_000i32..2_000_000_000).prop_map(|m| {
        [
            on_a_shared_line(f64::from(m[0])),
            on_a_shared_line(f64::from(m[1])),
            on_a_shared_line(f64::from(m[2])),
        ]
    })
}

proptest! {
    // These four are the reason this file exists, and one predicate call is a
    // few dozen flops. Run more of them than the default 256.
    #![proptest_config(ProptestConfig { cases: 4096, ..ProptestConfig::default() })]

    #[test]
    fn arbitrary_points_agree_with_robust(a in a_point(), b in a_point(), c in a_point()) {
        agrees_with_robust(a, b, c);
    }

    #[test]
    fn collinear_points_agree_with_robust(triple in a_collinear_triple()) {
        let [a, b, c] = triple;
        agrees_with_robust(a, b, c);
    }

    #[test]
    fn collinear_points_moved_by_a_few_ulps_agree_with_robust(
        triple in a_collinear_triple(),
        axis in 0usize..2,
        ulps in -16i64..16,
    ) {
        let [a, b, c] = triple;
        agrees_with_robust(a, b, nudged(c, axis, ulps));
    }

    /// Swapping two arguments flips the sign of a determinant. Anything that
    /// gets this wrong has an orientation bug that a collinear test cannot see.
    #[test]
    fn swapping_two_points_flips_the_answer(a in a_point(), b in a_point(), c in a_point()) {
        let forwards = orient2d_reporting_level(a, b, c).0;
        let backwards = orient2d_reporting_level(b, a, c).0;
        let expected = match forwards {
            Decision::Neg => Decision::Pos,
            Decision::Pos => Decision::Neg,
            Decision::Zero => Decision::Zero,
            Decision::Undecidable { enclosure } => panic!(
                "exact f64 inputs must always be decided, got an enclosure {enclosure} wide"),
            other => panic!("unhandled decision {other:?}"),
        };
        prop_assert_eq!(backwards, expected);
    }
}
