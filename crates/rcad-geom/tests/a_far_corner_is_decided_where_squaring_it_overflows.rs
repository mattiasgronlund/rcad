//! A quadric's side-of-surface expression is a sum of squares, and level 2
//! used to square an interval as `gap * gap`. Interval multiplication takes its
//! two factors as independent numbers, so a box side that straddles zero
//! squared to a range reaching below zero, where no square is — and for a huge
//! box, down to `-inf`, which made the whole sum an enclosure of everything.
//!
//! Now each square is a square, and an evaluation that still overflows is run
//! again with every length scaled down by one exact power of two
//! (`docs/decisions.md` §189.1). A refusal from either was a gap in the
//! arithmetic, not a limit: the answers below exist and are now reached.

#![forbid(unsafe_code)]

use rcad_geom::{Axis, Cone, Cylinder, Point, Sphere, Torus};
use rcad_predicates::{Decision, Interval, Point as PredicatePoint, SideOfSurface};

/// Fuse seed `3482a32e` half 0: a ball of radius 5 by the origin, and a corner
/// at `x = 4.7e226` widened into the room the sweep gives it,
/// `16 · ε · 4.7e226 ≈ 1.7e212`. The `y` and `z` sides of that box straddle
/// zero, and squaring them with `gap * gap` gave `[-inf, inf]`.
#[test]
fn the_seeds_far_corner_is_decided_off_its_ball() {
    let ball = Sphere::new(Point::new(1.113623391098575e-308, 0.0, 0.0), 5.0).unwrap();
    let corner = [4.737909217226285e226, 1.727233711018889e-77, 5.0];
    let room = 16.0 * f64::EPSILON * corner[0];
    let widened = PredicatePoint::Constructed(
        corner.map(|coordinate| Interval::around(coordinate, room).unwrap()),
    );
    assert_eq!(ball.side_of(PredicatePoint::from(corner)), Decision::Pos);
    assert_eq!(ball.side_of(widened), Decision::Pos);
}

/// A ball of radius `1e200` and a corner exactly on it. Every square
/// overflows, so plain, the answer is undecided with an enclosure of `inf` —
/// the same answer a corner nowhere near the ball would get from an overflow.
/// Scaled down, the straddle is a few ulps wide: the corner is honestly
/// undecided because it is on the skin, and a caller can tell that apart from
/// "nothing is known".
#[test]
fn a_corner_on_a_ball_of_radius_1e200_is_undecided_by_a_finite_width() {
    let ball = Sphere::new(Point::origin(), 1e200).unwrap();
    let on_it = [1e200, 0.0, 0.0];
    let room = 16.0 * f64::EPSILON * 1e200;
    let widened =
        PredicatePoint::Constructed(on_it.map(|coordinate| Interval::around(coordinate, room).unwrap()));
    for point in [PredicatePoint::from(on_it), widened] {
        let Decision::Undecidable { enclosure } = ball.side_of(point) else {
            panic!("a corner on the ball is on the skin, so neither side can be certified");
        };
        assert!(enclosure.is_finite(), "{point:?} came back with {enclosure}");
    }
}

/// The same for the other three: a corner exactly on a pipe of radius `1e200`,
/// on a cone `1e200` up its axis, and on a ring whose quartic overflows from
/// `1e77` on. Each overflowed to `inf` plain.
#[test]
fn a_corner_on_a_huge_pipe_cone_or_ring_is_undecided_by_a_finite_width() {
    let pipe = Cylinder::new(Axis::z(), 1e200).unwrap();
    let cone = Cone::new(Axis::z(), 1.0).unwrap();
    let ring = Torus::new(Axis::z(), 1e100, 5e99).unwrap();
    let cases: [(&dyn SideOfSurface, [f64; 3]); 3] = [
        (&pipe, [1e200, 0.0, 0.0]),
        (&cone, [1e200, 0.0, 1e200]),
        (&ring, [1.5e100, 0.0, 0.0]),
    ];
    let not_finite: Vec<_> = cases
        .into_iter()
        .map(|(surface, on_it)| (on_it, surface.side_of(PredicatePoint::from(on_it))))
        .filter(|(_, decision)| {
            let Decision::Undecidable { enclosure } = decision else {
                panic!("a corner on the skin came back {decision:?}");
            };
            !enclosure.is_finite()
        })
        .collect();
    assert_eq!(not_finite, []);
}

/// Where the scaling costs something, as `without_overflow` in `sphere.rs`
/// says: the corner `(2⁶⁰⁰, 2⁻⁵⁰⁰, 0)` on a ball of radius `2⁶⁰⁰` is outside
/// it by `2⁻¹⁰⁰⁰` in the squared distance, and scaled down by `2⁻⁶⁰⁰` that
/// `2⁻⁵⁰⁰` falls under the smallest `f64`. The box rounds outward, so the
/// answer is undecided rather than wrong.
#[test]
fn a_length_that_underflows_under_the_scaling_leaves_the_answer_undecided_not_wrong() {
    let big = 2f64.powi(600);
    let ball = Sphere::new(Point::origin(), big).unwrap();
    let decision = ball.side_of(PredicatePoint::from([big, 2f64.powi(-500), 0.0]));
    let Decision::Undecidable { enclosure } = decision else {
        panic!("came back {decision:?}");
    };
    assert!(enclosure.is_finite(), "{enclosure}");
}

/// A box two units either side of `x = 0`, one and a half units out along
/// `y`: at least `2.25` from the axis squared, against a radius of 1. The `x`
/// side squared as `[-2, 2] * [-2, 2]` is `[-4, 4]`, and that alone used to
/// make each of these undecided. Squared as a square it is `[0, 4]`.
fn a_box_straddling_x() -> PredicatePoint {
    PredicatePoint::Constructed([
        Interval::new(-2.0, 2.0).unwrap(),
        Interval::exactly(1.5),
        Interval::exactly(0.0),
    ])
}

#[test]
fn a_box_straddling_the_middle_of_a_ball_is_decided_outside_it() {
    let ball = Sphere::new(Point::origin(), 1.0).unwrap();
    assert_eq!(ball.side_of(a_box_straddling_x()), Decision::Pos);
}

#[test]
fn a_box_straddling_a_pipes_axis_is_decided_outside_it() {
    let pipe = Cylinder::new(Axis::z(), 1.0).unwrap();
    assert_eq!(pipe.side_of(a_box_straddling_x()), Decision::Pos);
}

/// On the plane of the apex, where the cone is only its apex, so every point
/// but the apex is outside it.
#[test]
fn a_box_straddling_a_cones_axis_is_decided_outside_it() {
    let cone = Cone::new(Axis::z(), 1.0).unwrap();
    assert_eq!(cone.side_of(a_box_straddling_x()), Decision::Pos);
}

/// A ring of radius 2 with a tube of radius 1, and a box three units above
/// its centre, two and a half either side of the axis in `x`. The box's
/// nearest point to the tube is `(2.5, 0, 3)`, outside it. Squared as a
/// square, the quartic's lower end is `12² - 16 · 2.5² = 44`; with
/// `[-2.5, 2.5] * [-2.5, 2.5]` it was `(12 - 6.25)² - 16 · 6.25`, below zero.
#[test]
fn a_box_straddling_a_rings_axis_is_decided_outside_it() {
    let ring = Torus::new(Axis::z(), 2.0, 1.0).unwrap();
    let above = PredicatePoint::Constructed([
        Interval::new(-2.5, 2.5).unwrap(),
        Interval::exactly(0.0),
        Interval::exactly(3.0),
    ]);
    assert_eq!(ring.side_of(above), Decision::Pos);
}
