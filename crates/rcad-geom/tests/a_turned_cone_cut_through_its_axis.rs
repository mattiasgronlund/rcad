//! The cone of `cones::a_frustum(model, 0.5, 2, 6)` turned 0.4 about `x`, cut
//! by the plane through its axis turned the same way: why `intersect` cannot
//! name the two rulings the pair crosses in, and what would (§294.11).
//!
//! This is the pair `rcad-topo`'s
//! `the_face_along_a_turned_frustums_seam_crosses_its_cone_in_two_rulings_nothing_names`
//! finds inside a fuse, built here from the two surfaces alone. The plane runs
//! through the cone's apex at the origin and is steeper than the cone, so the two
//! cross in two rulings through the apex. Unturned, `intersect` names them;
//! turned, it refuses `CannotNameWhichPieceOfACrossingThisIs`.
//!
//! # The turned rulings run along no direction any `f64`s can write
//!
//! Turned, the axis direction is `w = (0, −b, a)` and the plane's normal is
//! `n = (0, −a, −b)`, for the two `f64`s `a` and `b` the turn came out as. The
//! plane holds `x̂` and `w`, so a direction in it is `c·x̂ + w` up to a scale,
//! and that direction is on the cone of slope `½` exactly when `c² = q/4`, with
//! `q = a² + b²` worked out exactly. So the rulings run along `(±½√q, −b, a)`,
//! and `q` is not the square of any fraction, so neither is any multiple of
//! that direction three fractions.
//!
//! Two families of fix fall to that one fact, on this pair:
//!
//! 1. **Working the direction out in another order, or at another scale.**
//!    Whatever arithmetic does it, it ends in three `f64`s, which are fractions.
//! 2. **A picker whose flat surface holds the ruling**, which is the pair of
//!    opposite half-spaces `two_lines_through_a_cones_apex` builds. A normal `p`
//!    square to `(±½√q, −b, a)` has `p_x·(±½√q) + p·w = 0`; both products are
//!    fractions times one irrational and one fraction, so `p_x = 0` and
//!    `p·w = 0`, and then `p` is along `x̂ × w`, which is `n` bit for bit. The
//!    only flat surface through the apex that holds one ruling is the cut plane,
//!    and it holds the other as well.
//!
//! # What does tell them apart: four halves in four corners
//!
//! The rulings are mirror images across the flat surface through the apex that
//! holds the axis and `n`, whose normal is `n × w`. A ruling crosses that surface
//! at the apex, so a half-space bounded by it holds **half** of each ruling and
//! not one whole one. With the flat surface through the apex square to the axis
//! as well, each of the four halves lies in its own corner of the two, strictly,
//! and neither surface needs a direction to be exact: only to pass exactly
//! through the apex, which here is the origin and offset `0`. So the four half
//! rulings can be named exactly, and the two whole ones cannot.

use rcad_geom::{
    Axis, Cone, Error, HalfSpace, Isometry, Plane, Point, Surface, SurfaceMeet, Vector,
};
use rcad_geom::{
    how_a_plane_leans_against_a_cone, intersect, where_a_cones_apex_stands_against_a_plane,
};
use rcad_predicates::{
    Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface,
};

/// The turned cone's axis direction, as `rcad-topo`'s fuse finds it (§294.10).
const THE_TURNED_AXIS: [f64; 3] = [0.0, -0.389_418_342_308_650_47, 0.921_060_994_002_885_1];

/// The turned plane's normal, as `rcad-topo`'s fuse finds it (§294.10).
const THE_TURNED_NORMAL: [f64; 3] = [0.0, -0.921_060_994_002_885_1, -0.389_418_342_308_650_47];

#[test]
fn turned_the_two_rulings_through_the_apex_are_refused_and_unturned_they_are_named() {
    for (turned, movement) in [(false, Isometry::identity()), (true, the_turn())] {
        let (flat, cone) = the_pair(&movement);
        assert_eq!(cone.apex(), Point::origin(), "turned {turned}");
        assert_eq!(
            where_a_cones_apex_stands_against_a_plane(&flat, &cone),
            Decision::Zero,
            "turned {turned}: the plane runs through the apex"
        );
        assert_eq!(
            how_a_plane_leans_against_a_cone(&flat, &cone),
            Decision::Neg,
            "turned {turned}: the plane is steeper than the cone"
        );
        let met = intersect(
            &Surface::Plane(flat),
            &Surface::Cone(cone),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        if turned {
            assert_eq!(cone.axis().direction().components(), THE_TURNED_AXIS);
            assert_eq!(flat.normal().components(), THE_TURNED_NORMAL);
            assert_eq!(flat.offset(), 0.0);
            assert!(
                matches!(met, Err(Error::CannotNameWhichPieceOfACrossingThisIs)),
                "turned, the two rulings are not named: {met:?}"
            );
        } else {
            let Ok(SurfaceMeet::Curves(rulings)) = met else {
                panic!("unturned, the two rulings are named: {met:?}");
            };
            assert_eq!(rulings.len(), 2);
        }
    }
}

/// `q = a² + b²` is not the square of a fraction, so no direction three `f64`s
/// write lies on both surfaces, however it was worked out; and the only flat
/// surface through the apex square to a ruling is the cut plane.
#[test]
fn the_turned_rulings_run_along_directions_no_fraction_writes() {
    let (flat, cone) = the_pair(&the_turn());
    let [_, minus_b, a] = cone.axis().direction().components();
    let along = cone.axis().direction();
    assert_eq!(
        Vector::new(1.0, 0.0, 0.0).cross(along),
        flat.normal(),
        "n is x̂ × w bit for bit, so the plane holds x̂ and w"
    );

    let (whole_a, a_exponent) = as_a_whole_number_times_a_power_of_two(a);
    let (whole_b, b_exponent) = as_a_whole_number_times_a_power_of_two(-minus_b);
    let lowest = a_exponent.min(b_exponent);
    let shifts = [a_exponent - lowest, b_exponent - lowest]
        .map(|shift| u32::try_from(2 * shift).expect("a's exponent is at or above the lowest"));
    assert!(
        shifts.iter().all(|shift| *shift <= 16),
        "{shifts:?} fits in u128"
    );
    let whole_q = ((whole_a * whole_a) << shifts[0]) + ((whole_b * whole_b) << shifts[1]);
    let root = whole_q.isqrt();
    assert_ne!(
        root * root,
        whole_q,
        "q is {whole_q}·4^{lowest}, and {whole_q} is not a square, so √q is irrational"
    );

    let half_root = (a * a + minus_b * minus_b).sqrt() / 2.0;
    let mut candidates = vec![0.5, half_root];
    let (mut below, mut above) = (half_root, half_root);
    for _ in 0..4 {
        below = below.next_down();
        above = above.next_up();
        candidates.extend([below, above]);
    }
    for across in candidates {
        for scale in [0.125, 1.0, 32.0] {
            for side in [1.0, -1.0] {
                let direction = [side * across, minus_b, a].map(|component| component * scale);
                let at = PredicatePoint::Exact(direction);
                assert_eq!(
                    flat.side_of_point(at),
                    Decision::Zero,
                    "{direction:?} is in the plane"
                );
                assert_ne!(
                    cone.side_of(at),
                    Decision::Zero,
                    "{direction:?} is not on the cone, and nothing three f64s write is"
                );
            }
        }
    }
}

/// The candidate the row named, checked: the flat surface through the apex
/// facing `n × w` and the one facing `w` put each half of each ruling in a
/// corner of its own, strictly, and both pass exactly through the apex.
///
/// Each half is stood for by a box round its point one step out, `±(±½√q, −b,
/// a)`, which holds a point of the ruling: the box is the segment between two
/// exact points of the plane, and the cone reads inside at one and outside at the
/// other. A flat surface through the apex reads one sign along a whole half
/// ruling once it reads it strictly at one point, so a box decided strictly is
/// that half decided.
#[test]
fn the_turned_rulings_four_halves_lie_in_four_corners_of_two_flat_surfaces_through_the_apex() {
    let (flat, cone) = the_pair(&the_turn());
    let along = cone.axis().direction();
    let [_, minus_b, a] = along.components();
    let mirror = HalfSpace::through_a_point_facing(cone.apex(), flat.normal().cross(along))
        .expect("n × w is not nothing");
    let nappe = HalfSpace::through_a_point_facing(cone.apex(), along).expect("an axis direction");
    assert_eq!([mirror.offset(), nappe.offset()], [0.0, 0.0]);

    let half_root = (a * a + minus_b * minus_b).sqrt() / 2.0;
    let (inside, outside) = (
        half_root.next_down().next_down(),
        half_root.next_up().next_up(),
    );

    let mut corners = Vec::new();
    for way in [1.0, -1.0] {
        for side in [1.0, -1.0] {
            let ends =
                [inside, outside].map(|across| [way * side * across, way * minus_b, way * a]);
            assert_eq!(
                ends.map(|end| flat.side_of_point(PredicatePoint::Exact(end))),
                [Decision::Zero; 2],
                "both ends are in the plane, so the segment between them is"
            );
            assert_eq!(
                ends.map(|end| cone.side_of(PredicatePoint::Exact(end))),
                [Decision::Neg, Decision::Pos],
                "the box round {way} × ({side} × ½√q, −b, a) holds a point of the ruling"
            );
            let [low, high] = if way * side > 0.0 {
                [ends[0][0], ends[1][0]]
            } else {
                [ends[1][0], ends[0][0]]
            };
            let the_box = PredicatePoint::Constructed([
                Interval::new(low, high).expect("low is below high"),
                Interval::exactly(way * minus_b),
                Interval::exactly(way * a),
            ]);
            let corner = [mirror.side_of_point(the_box), nappe.side_of_point(the_box)];
            assert!(
                corner
                    .iter()
                    .all(|sign| matches!(sign, Decision::Pos | Decision::Neg)),
                "{way} × ({side} × ½√q, −b, a) is strictly to one side of both: {corner:?}"
            );
            corners.push(corner);
        }
    }
    for (index, corner) in corners.iter().enumerate() {
        assert!(
            !corners[index + 1..].contains(corner),
            "four halves, four corners: {corners:?}"
        );
    }
}

/// The plane through the cone's axis and the cone of slope `½` with its apex at
/// the origin, both moved by `movement`.
fn the_pair(movement: &Isometry) -> (Plane, Cone) {
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, -1.0, 0.0))
        .expect("a plane through the origin");
    let cone = Cone::new(
        Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0)).expect("an axis"),
        0.5,
    )
    .expect("a slope above zero");
    let (Surface::Plane(flat), Surface::Cone(cone)) = (
        Surface::Plane(flat).transformed(movement),
        Surface::Cone(cone).transformed(movement),
    ) else {
        unreachable!("a movement keeps a plane a plane and a cone a cone");
    };
    (flat, cone)
}

/// A turn of 0.4 about `x`, which rounds, as `rcad-topo`'s `the_turn` is.
fn the_turn() -> Isometry {
    Isometry::rotation(&Axis::x(), 0.4).expect("a turn about x")
}

/// A positive normal `f64` as a whole number times a power of two.
fn as_a_whole_number_times_a_power_of_two(value: f64) -> (u128, i32) {
    assert!(value.is_normal() && value > 0.0, "{value}");
    let bits = value.to_bits();
    let exponent = i32::try_from((bits >> 52) & 0x7ff).expect("eleven bits") - 1075;
    (u128::from((bits & ((1 << 52) - 1)) | (1 << 52)), exponent)
}
