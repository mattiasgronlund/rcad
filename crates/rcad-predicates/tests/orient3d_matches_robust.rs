//! `orient3d` against the `robust` crate, which is always exact.
//!
//! `robust` fuses the float filter and exact arithmetic internally and never
//! comes back undecided, so for exact `f64` inputs our answer must equal its
//! sign on every single input. Zero disagreements, or the cascade is wrong.
//! This is the highest-value test M0 has: a known-correct implementation
//! validating the framework before anything depends on it (decisions.md §2.2,
//! §9.1).

use proptest::prelude::*;
use rcad_predicates::{Decision, Level, Point, orient3d_reporting_level};

/// Checks one quadruple against `robust` and reports which level answered.
///
/// An exact `f64` input that comes back undecided is a failure in itself:
/// level 3 always decides as long as the products stay in the normal `f64`
/// range, and every case here keeps them there.
fn agrees_with_robust(a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) -> Level {
    let expected = robust_decision(a, b, c, d);
    let (ours, level) = orient3d_reporting_level(
        Point::from(a),
        Point::from(b),
        Point::from(c),
        Point::from(d),
    );

    match ours {
        Decision::Neg | Decision::Zero | Decision::Pos => assert_eq!(
            ours, expected,
            "disagreed with robust on {a:?} {b:?} {c:?} {d:?} at {level:?}"
        ),
        Decision::Undecidable { enclosure } => panic!(
            "exact f64 inputs must always be decided, but {a:?} {b:?} {c:?} {d:?} \
             came back undecided with an enclosure {enclosure} wide"
        ),
        other => panic!("unhandled decision {other:?}"),
    }
    level
}

fn robust_decision(a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) -> Decision {
    let value = robust::orient3d(coord(a), coord(b), coord(c), coord(d));
    if value > 0.0 {
        Decision::Pos
    } else if value < 0.0 {
        Decision::Neg
    } else {
        Decision::Zero
    }
}

fn coord(p: [f64; 3]) -> robust::Coord3D<f64> {
    robust::Coord3D {
        x: p[0],
        y: p[1],
        z: p[2],
    }
}

/// Points on the plane `3x + 5y + 7z = 0`, built from two directions that lie
/// in it. Whole-number multiples keep every coordinate exact, so the four
/// points are exactly coplanar however large the multiples get.
fn on_a_shared_plane(along: f64, across: f64) -> [f64; 3] {
    [5.0 * along, -3.0 * along + 7.0 * across, -5.0 * across]
}

/// Moves one coordinate by `ulps` representable steps.
///
/// Goes through the same sign-magnitude to two's-complement flip that
/// `f64::total_cmp` uses, so stepping works across zero and through the
/// negatives instead of landing on a NaN.
fn nudged(point: [f64; 3], axis: usize, ulps: i64) -> [f64; 3] {
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

type FourPoints = ([f64; 3], [f64; 3], [f64; 3], [f64; 3]);

/// Quadruples chosen to break things: exactly coplanar, one ulp off coplanar,
/// collinear, coincident, spread over sixteen orders of magnitude, and large
/// enough that the products are within a few decades of overflowing.
fn the_awkward_cases() -> Vec<FourPoints> {
    let plane = |along, across| on_a_shared_plane(along, across);
    let mut cases = vec![
        // Flat on the z = 0 plane, in the simplest possible arrangement.
        (
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ),
        // Exactly coplanar on a plane that is not axis-aligned, with enough
        // significant bits that the products round.
        (
            plane(1.0, 0.0),
            plane(0.0, 1.0),
            plane(1000003.0, 999983.0),
            plane(1234567.0, 765432.0),
        ),
        (
            plane(1.0, 0.0),
            plane(0.0, 1.0),
            plane(5000015.0 / 5.0, 999983.0),
            plane(2097152.0, 1048576.0),
        ),
        // Three points on one line: coplanar with anything at all.
        (
            [0.0, 0.0, 0.0],
            [1.0, 2.0, 3.0],
            [2.0, 4.0, 6.0],
            [7.0, -1.0, 0.5],
        ),
        (
            [1.5, 1.5, 1.5],
            [-1.5, -1.5, -1.5],
            [3.0, 3.0, 3.0],
            [0.0, 1.0, 0.0],
        ),
        // The same point more than once.
        (
            [0.25, -0.5, 8.0],
            [1.0, 2.0, 3.0],
            [0.25, -0.5, 8.0],
            [7.0, -1.0, 0.5],
        ),
        (
            [0.25, -0.5, 8.0],
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
            [0.25, -0.5, 8.0],
        ),
        (
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
        ),
        // Sixteen orders of magnitude inside one tuple.
        (
            [1e-8, 0.0, 0.0],
            [0.0, 1e8, 0.0],
            [0.0, 0.0, 1e-8],
            [1e8, 1e-8, 1e8],
        ),
        (
            [1e8, 1e8, 1e-8],
            [1e-8, 1e8, 1e8],
            [1e8, 1e-8, 1e8],
            [0.0, 0.0, 0.0],
        ),
        // Large enough that the triple products reach 1e300 and stop a few
        // decades short of overflowing.
        (
            [1e100, 0.0, 0.0],
            [0.0, 1e100, 0.0],
            [0.0, 0.0, 1e100],
            [-1e100, -1e100, -1e100],
        ),
        (
            [1e100, 1e100, 1e100],
            [-1e100, 1e100, -1e100],
            [1e100, -1e100, -1e100],
            [1e-100, 1e-100, 1e-100],
        ),
    ];

    // Each case again with one coordinate moved by a handful of ulps in each
    // direction. This is where a filter that trusts itself too far gets the
    // sign backwards.
    //
    // Nudging a zero lands in the subnormals, where expansion arithmetic is no
    // longer exact and `robust` is no longer a valid oracle; that case has a
    // test of its own below.
    let flat_cases = cases.clone();
    for (a, b, c, d) in flat_cases {
        for axis in 0..3 {
            for ulps in [-8i64, -2, -1, 1, 2, 8] {
                let moved_d = nudged(d, axis, ulps);
                if is_normal_or_zero(moved_d[axis]) {
                    cases.push((a, b, c, moved_d));
                }
                let moved_c = nudged(c, axis, ulps);
                if is_normal_or_zero(moved_c[axis]) {
                    cases.push((a, b, moved_c, d));
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
    assert_eq!(cases.len(), 384);
    for (a, b, c, d) in cases {
        agrees_with_robust(a, b, c, d);
    }
}

/// The same four points, lifted off the shared plane a little further each
/// time. Every level of the cascade has to answer somewhere on this ladder, or
/// the two cheap levels are dead code and the test suite is only ever
/// exercising exact arithmetic.
#[test]
fn lifting_a_point_further_off_the_plane_walks_back_down_the_cascade() {
    let a = on_a_shared_plane(1.0, 0.0);
    let b = on_a_shared_plane(0.0, 1.0);
    let c = on_a_shared_plane(1000003.0, 999983.0);
    let on_the_plane = on_a_shared_plane(1234567.0, 765432.0);

    let level_for = |ulps| agrees_with_robust(a, b, c, nudged(on_the_plane, 2, ulps));

    assert_eq!(
        level_for(0),
        Level::Exact,
        "only exact arithmetic can prove a determinant is zero once the \
         products start rounding"
    );
    assert_eq!(level_for(1), Level::Exact);
    assert_eq!(level_for(-1), Level::Exact);
    assert_eq!(
        level_for(1 << 18),
        Level::Interval,
        "the interval is sharper here than the static error bound is"
    );
    assert_eq!(level_for(-(1 << 18)), Level::Interval);
    assert_eq!(
        level_for(1 << 22),
        Level::Float,
        "well clear of the plane, the error bound settles it on its own"
    );
    assert_eq!(level_for(-(1 << 22)), Level::Float);
}

#[test]
fn a_point_well_clear_of_the_plane_is_settled_by_the_float_filter() {
    let level = agrees_with_robust(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.3, 0.3, -1.0],
    );
    assert_eq!(level, Level::Float);
}

/// Three points on one line are coplanar with every fourth point, so this
/// determinant is exactly zero whatever `d` is. Down among the subnormals the
/// roundoff term of a product no longer fits in an `f64`, so expansion
/// arithmetic stops being exact — `robust` answers `-1.5e-323` here, which is
/// simply wrong.
///
/// This used to be refused, and the refusal was a gap rather than a limit
/// (`docs/decisions.md` §324): every coordinate here is exact, the largest is
/// `3`, and multiplying all twelve by `2^337` lifts `-3.5e-323` to a normal
/// number without losing a bit. On that copy exact arithmetic runs, and it says
/// zero. What must never come back is a sign.
#[test]
fn coordinates_down_among_the_subnormals_are_lifted_and_decided_rather_than_guessed() {
    let a = [1.5, 1.5, 1.5];
    let b = [-1.5, -1.5, -1.5];
    let c = [3.0, 3.0, 3.0];
    let d = [-3.5e-323, 1.0, 0.0];

    assert_ne!(
        robust::orient3d(coord(a), coord(b), coord(c), coord(d)),
        0.0,
        "the case is only interesting because robust gets it wrong"
    );

    let (ours, level) = orient3d_reporting_level(
        Point::from(a),
        Point::from(b),
        Point::from(c),
        Point::from(d),
    );
    match ours {
        Decision::Neg | Decision::Pos => {
            panic!("claimed a sign for a determinant that is exactly zero")
        }
        Decision::Zero => (),
        Decision::Undecidable { enclosure } => {
            panic!("refused at {level:?} with {enclosure}, though a lifted copy decides it")
        }
        other => panic!("unhandled decision {other:?}"),
    }
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

fn a_point() -> impl Strategy<Value = [f64; 3]> {
    [a_coordinate(), a_coordinate(), a_coordinate()]
}

/// A quadruple that is exactly coplanar however large the multiples get, so
/// the generator actually reaches the degenerate case instead of missing it
/// with probability one.
fn a_coplanar_quadruple() -> impl Strategy<Value = [[f64; 3]; 4]> {
    proptest::array::uniform8(-4_000_000i32..4_000_000).prop_map(|m| {
        [
            on_a_shared_plane(f64::from(m[0]), f64::from(m[1])),
            on_a_shared_plane(f64::from(m[2]), f64::from(m[3])),
            on_a_shared_plane(f64::from(m[4]), f64::from(m[5])),
            on_a_shared_plane(f64::from(m[6]), f64::from(m[7])),
        ]
    })
}

proptest! {
    // These four are the reason this file exists, and one predicate call is a
    // few dozen flops. Run more of them than the default 256.
    #![proptest_config(ProptestConfig { cases: 4096, ..ProptestConfig::default() })]

    #[test]
    fn arbitrary_points_agree_with_robust(
        a in a_point(), b in a_point(), c in a_point(), d in a_point()
    ) {
        agrees_with_robust(a, b, c, d);
    }

    #[test]
    fn coplanar_points_agree_with_robust(quadruple in a_coplanar_quadruple()) {
        let [a, b, c, d] = quadruple;
        agrees_with_robust(a, b, c, d);
    }

    #[test]
    fn coplanar_points_moved_by_a_few_ulps_agree_with_robust(
        quadruple in a_coplanar_quadruple(),
        axis in 0usize..3,
        ulps in -16i64..16,
    ) {
        let [a, b, c, d] = quadruple;
        agrees_with_robust(a, b, c, nudged(d, axis, ulps));
    }

    /// Swapping two arguments flips the sign of a determinant. Anything that
    /// gets this wrong has an orientation bug that a coplanar test cannot see.
    #[test]
    fn swapping_two_points_flips_the_answer(
        a in a_point(), b in a_point(), c in a_point(), d in a_point()
    ) {
        let forwards = orient3d_reporting_level(
            Point::from(a), Point::from(b), Point::from(c), Point::from(d)).0;
        let backwards = orient3d_reporting_level(
            Point::from(b), Point::from(a), Point::from(c), Point::from(d)).0;
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
