//! A quaternion names a rotation by the *direction* of its four coefficients,
//! so multiplying all four by one nonzero number names the very same rotation.
//! Multiplying them by a power of two costs no bits at all while the results
//! stay normal, so a movement read back off the wire must not depend on the
//! scale its quaternion happened to be written at.
//!
//! **It did.** `docs/decisions.md` §41.5: the sum of the four squares spans
//! twice the range the coefficients do, so written high enough it overflows,
//! `norm()` is infinity, dividing by it gives four zeros, and four zeros
//! normalize to the **identity** — a rotation the bytes did not name. Written
//! low enough the same sum underflows to zero and the rotation is refused for
//! having no length. §4.1 ranks the first of those below both crashing and
//! refusing, and `Model` is `Deserialize`, so it is reachable at every placed
//! face of a file.
//!
//! The oracle is the strongest one there is: the answer at one scale, bit for
//! bit. No tolerance appears in this file.

use core::f64::consts::FRAC_PI_3;

use rcad_geom::{Axis, Isometry, Point, Vector};

/// `coefficient * 2^exponent`, in two steps so the factor itself never leaves
/// the range — `2f64.powi(1074)` is infinity and would throw away scales that
/// are perfectly ordinary numbers.
fn scaled_by(coefficient: f64, exponent: i32) -> f64 {
    let half = exponent / 2;
    coefficient * 2f64.powi(half) * 2f64.powi(exponent - half)
}

fn rotation_read_back(quaternion: [f64; 4]) -> Option<[f64; 4]> {
    Isometry::try_from(([0.0, 0.0, 0.0], quaternion))
        .ok()
        .map(|movement| movement.rotation_quaternion())
}

/// Every scale of `written_at_one` that every coefficient survives exactly, and
/// what the kernel makes of it.
///
/// Exact is asked rather than argued: a scale counts only when scaling it back
/// down recovers every coefficient bit for bit. Past the top that fails on
/// infinity and below the bottom on the subnormal grid, and either way the
/// scaled array stands for a different rotation and proves nothing either way.
fn every_exact_scale_of(written_at_one: [f64; 4]) -> Vec<(i32, Option<[f64; 4]>)> {
    (-1074..=1023)
        .filter_map(|exponent| {
            let scaled = written_at_one.map(|c| scaled_by(c, exponent));
            let recovered = scaled.map(|c| scaled_by(c, -exponent));
            (recovered == written_at_one).then(|| (exponent, rotation_read_back(scaled)))
        })
        .collect()
}

/// `[2^k, 2^k, 0, 0]` is a quarter turn about `x` at every one of the 2098
/// scales an `f64` exponent field can hold, and a power of two times a power of
/// two is exact all the way down into the subnormals — so this family reaches
/// both ends of the range with nothing rounded anywhere.
#[test]
fn a_quarter_turn_written_at_every_power_of_two_scale_is_the_same_quarter_turn() {
    let scales = every_exact_scale_of([1.0, 1.0, 0.0, 0.0]);
    assert_eq!(scales.len(), 2098, "every exponent an f64 holds");

    let at_one = rotation_read_back([1.0, 1.0, 0.0, 0.0]).expect("a quarter turn about x");
    assert_ne!(
        at_one,
        Isometry::identity().rotation_quaternion(),
        "if the reference were the identity this test would pass by agreeing with the bug"
    );

    let wrong: Vec<_> = scales
        .iter()
        .filter(|(_, read_back)| *read_back != Some(at_one))
        .collect();
    assert!(
        wrong.is_empty(),
        "{} of {} scales of one quarter turn are not that quarter turn, from 2^{} to 2^{}",
        wrong.len(),
        scales.len(),
        wrong.first().expect("checked non-empty").0,
        wrong.last().expect("checked non-empty").0,
    );
}

/// The same thing for a turn whose coefficients are not powers of two and are
/// not all the same size, so the property is about scaling and not about the
/// two tidy numbers above.
#[test]
fn a_third_of_a_turn_about_a_leaning_axis_is_the_same_at_every_exact_scale() {
    let leaning = Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("an axis");
    let written = Isometry::rotation(&leaning, FRAC_PI_3)
        .expect("a third of a turn")
        .rotation_quaternion();

    let scales = every_exact_scale_of(written);
    assert!(
        scales.len() > 2000,
        "only {} scales survived, which is too few to be reaching either end",
        scales.len()
    );

    let at_one = rotation_read_back(written).expect("a third of a turn");
    let wrong: Vec<_> = scales
        .iter()
        .filter(|(_, read_back)| *read_back != Some(at_one))
        .collect();
    assert!(
        wrong.is_empty(),
        "{} of {} scales of one turn are not that turn, from 2^{} to 2^{}",
        wrong.len(),
        scales.len(),
        wrong.first().expect("checked non-empty").0,
        wrong.last().expect("checked non-empty").0,
    );
}

/// §41.5's own two numbers. `2·(1e153)² < f64::MAX` and `2·(1e154)² > f64::MAX`,
/// which is the whole of the boundary that used to sit here.
#[test]
fn a_quarter_turn_whose_squares_overflow_is_still_a_quarter_turn() {
    let straight_up = Point::new(0.0, 0.0, 1.0);
    for coefficient in [1e153, 1e154, f64::MAX] {
        let movement = Isometry::try_from(([0.0, 0.0, 0.0], [coefficient, coefficient, 0.0, 0.0]))
            .expect("a quarter turn about x, however it is written");
        let moved = movement.apply_to_point(straight_up);
        assert!(
            moved.y() < -0.999 && moved.y() > -1.001,
            "z should turn onto -y, but {coefficient:e} sent it to {moved:?}"
        );
    }
}

/// The other end, which was a refusal rather than a wrong answer: the sum of
/// squares underflows to zero and the rotation reads as having no length.
/// `1e-200` is the same number §39.7 item 7 refused three planes at.
#[test]
fn a_quarter_turn_whose_squares_underflow_is_still_a_quarter_turn() {
    let straight_up = Point::new(0.0, 0.0, 1.0);
    for coefficient in [1e-200, 1e-320, f64::MIN_POSITIVE / 4.0] {
        let movement = Isometry::try_from(([0.0, 0.0, 0.0], [coefficient, coefficient, 0.0, 0.0]))
            .expect("a quarter turn about x, however it is written");
        let moved = movement.apply_to_point(straight_up);
        assert!(
            moved.y() < -0.999 && moved.y() > -1.001,
            "z should turn onto -y, but {coefficient:e} sent it to {moved:?}"
        );
    }
}

/// What stays refused, and both are limits: bytes that name no rotation at all.
/// Four zeros have no direction to scale, and a coefficient that is not a
/// number names nothing to scale either.
#[test]
fn a_rotation_with_no_direction_at_all_is_still_refused() {
    assert!(Isometry::try_from(([0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0])).is_err());
    assert!(Isometry::try_from(([0.0, 0.0, 0.0], [0.0, -0.0, 0.0, -0.0])).is_err());
    assert!(Isometry::try_from(([0.0, 0.0, 0.0], [f64::INFINITY, 1.0, 0.0, 0.0])).is_err());
    assert!(Isometry::try_from(([0.0, 0.0, 0.0], [f64::NAN, 1.0, 0.0, 0.0])).is_err());
}
