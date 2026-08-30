//! `Plane::SIGNED_DISTANCE_ROUNDOFF` says what rounding can cost
//! [`Plane::signed_distance_to`]. This checks it is telling the truth.
//!
//! Answering the four questions of `docs/decisions.md` §14.1:
//!
//! 1. **Exactly true, or bounded?** Bounded, and the bound is the thing under
//!    test. The oracle is exact, so nothing here is tuned until it passes: the
//!    same expression is evaluated again in Shewchuk expansion arithmetic,
//!    where every product and sum is kept whole, and the two are subtracted.
//!    What is left is exactly what rounding cost.
//! 2. **Do the two runs take different paths?** Yes — one is six `f64`
//!    operations, the other never rounds at all. They share the expression and
//!    nothing else.
//! 3. **What family of inputs?** Planes whose normals point every which way,
//!    over coordinate magnitudes from a millimetre to a kilometre, plus
//!    hand-written cases aimed at the two places the derivation is loosest: a
//!    normal spread equally over all three axes, which is where `|n₁| + |n₂| +
//!    |n₃|` reaches `√3`, and a large offset, which is the term the derivation
//!    charges least to.
//! 4. **Which gap would this have caught?** §12.6's fourth. The figure used to
//!    live in `rcad-topo` as `64 * f64::EPSILON` with a comment deriving `3ε`
//!    and calling the rest a guess — and `3ε` is not in fact a bound, because
//!    counting halves of an ulp per term ignores that a rounding costs half an
//!    ulp of the running sum. Nothing anywhere checked it.
//!
//! What this cannot do is show the constant is *tight*. Rounding errors partly
//! cancel, so a random case lands well inside the worst case and would pass
//! against a constant that is too small. The derivation next to the constant is
//! what makes it a bound; this is what stops the expression drifting away from
//! it.

use proptest::prelude::*;
use rcad_geom::{Plane, Point, Vector};
use rcad_predicates::expansion::{expansion_sum, product_is_exact, two_product};

/// How far `plane.signed_distance_to(point)` landed from the true value of the
/// expression it evaluates, and the largest quantity that went into it.
fn rounding_cost_and_scale(plane: &Plane, point: Point) -> (f64, f64) {
    let normal = plane.normal().components();
    let coordinates = point.coordinates();
    let offset = plane.offset();
    let computed = plane.signed_distance_to(point);

    let mut terms = Vec::with_capacity(8);
    for axis in 0..3 {
        assert!(
            product_is_exact(normal[axis], coordinates[axis]),
            "this test needs coordinates whose products stay in the normal range"
        );
        let (high, low) = two_product(normal[axis], coordinates[axis]);
        terms.push(low);
        terms.push(high);
    }
    terms.push(-offset);
    terms.push(-computed);

    let scale = coordinates
        .iter()
        .map(|coordinate| coordinate.abs())
        .chain(core::iter::once(offset.abs()))
        .fold(0.0, f64::max);
    (exact_sum(&terms).abs(), scale)
}

/// The exact sum of `terms`, folded one at a time so nothing is ever rounded
/// away, then collapsed to the one `f64` nearest it.
fn exact_sum(terms: &[f64]) -> f64 {
    let mut total = [0.0f64; 32];
    let mut length = 0;
    let mut merged = [0.0f64; 32];
    for &term in terms {
        length = expansion_sum(&total[..length], &[term], &mut merged);
        core::mem::swap(&mut total, &mut merged);
    }
    total[..length].iter().sum()
}

fn assert_within_the_bound(plane: &Plane, point: Point) {
    let (cost, scale) = rounding_cost_and_scale(plane, point);
    let allowed = Plane::SIGNED_DISTANCE_ROUNDOFF * scale;
    assert!(
        cost <= allowed,
        "reading a distance cost {cost}, over the {allowed} the bound promises \
         at a scale of {scale}"
    );
}

/// The normal spread equally over all three axes is where the derivation's
/// `|n₁| + |n₂| + |n₃| ≤ √3` is an equality, so it is where the bound is
/// closest to being reached.
#[test]
fn a_normal_spread_evenly_over_three_axes_stays_inside_the_bound() {
    let plane =
        Plane::from_point_and_normal(Point::new(300.0, -700.0, 900.0), Vector::new(1.0, 1.0, 1.0))
            .expect("a slanted plane");

    for coordinate in [1.0, 97.0, 1234.5, 999_999.5] {
        let point = Point::new(coordinate, -coordinate, coordinate * 0.5);
        assert_within_the_bound(&plane, point);
    }
}

/// The offset is the term the derivation charges least to — one rounding — so a
/// plane far from the origin is where undercharging it would show.
#[test]
fn a_plane_far_from_the_origin_stays_inside_the_bound() {
    let plane =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 800_000.0), Vector::new(1.0, 2.0, 3.0))
            .expect("a slanted plane far away");

    for height in [0.0, 1.0, 799_999.0, 1_600_000.0] {
        assert_within_the_bound(&plane, Point::new(1.0, -1.0, height));
    }
}

/// A point exactly on the plane is the case the consumer in `rcad-topo`
/// actually asks about, and the one where every bit of the answer is rounding.
#[test]
fn a_corner_of_a_face_reads_as_being_on_its_own_plane_to_within_the_bound() {
    let plane = Plane::through_three_points(
        Point::new(1.0, 0.0, 0.0),
        Point::new(0.0, 1.0, 0.0),
        Point::new(0.0, 0.0, 1.0),
    )
    .expect("a plane through three corners");

    for corner in [
        Point::new(1.0, 0.0, 0.0),
        Point::new(0.0, 1.0, 0.0),
        Point::new(0.0, 0.0, 1.0),
    ] {
        assert_within_the_bound(&plane, corner);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test]
    fn no_plane_and_point_costs_more_than_the_bound_allows(
        normal in prop::array::uniform3(-1.0f64..1.0),
        through in prop::array::uniform3(-1000.0f64..1000.0),
        at in prop::array::uniform3(-1000.0f64..1000.0),
    ) {
        let direction = Vector::new(normal[0], normal[1], normal[2]);
        prop_assume!(direction.length() > 1e-3);

        let plane = Plane::from_point_and_normal(
            Point::new(through[0], through[1], through[2]),
            direction,
        )
        .expect("a direction with length makes a plane");

        assert_within_the_bound(&plane, Point::new(at[0], at[1], at[2]));
    }
}
