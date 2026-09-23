//! A plane leaned off a cone's axis cuts an **ellipse**, and that ellipse has a
//! centre.
//!
//! It is not where the axis crosses the plane, which is the answer a reader
//! guesses and the one `docs/decisions.md` §209.4 names as wrong. It is the
//! **pole** of the plane against the cone's own quadric — where the plane meets
//! the line through the apex conjugate to it — and
//! [`Cone::which_way_a_closed_crossings_centre_lies`] is the direction to walk
//! from the apex to reach it.
//!
//! Every check here comes at the centre a second way: from the two rulings the
//! plane meets in the cone's own symmetry plane, whose midpoint is the ellipse's
//! centre, or by walking out from the claimed centre until the cone's published
//! sign flips and asking whether the two walks were the same length.

use rcad_geom::{Axis, Cone, Plane, Point, Vector};
use rcad_predicates::{Decision, Interval, Point as PredicatePoint, SideOfSurface};

/// Where the centre lands, taken from the direction the cone hands back.
///
/// The direction is an enclosure rather than a triple, because forming it
/// rounds; a test may take its middle, a predicate may not.
fn the_centre(cone: &Cone, flat: &Plane) -> Option<[f64; 3]> {
    let along = cone.which_way_a_closed_crossings_centre_lies(flat)?;
    let middle = |reach: Interval| 0.5 * (reach.lo() + reach.hi());
    let along = along.map(middle);
    let normal = flat.normal().components();
    let apex = cone.apex().coordinates();
    let over: f64 = (0..3).map(|at| normal[at] * along[at]).sum();
    assert!(
        over > 0.0,
        "the closed family has a positive n·along: {over}"
    );
    let reach = (flat.offset() - (0..3).map(|at| normal[at] * apex[at]).sum::<f64>()) / over;
    Some(core::array::from_fn(|at| apex[at] + reach * along[at]))
}

/// A cone of slope one half about `+z` from the origin, cut by `z = x/4 + 4`.
///
/// In the plane `y = 0` the cone is the pair of lines `z = ±2x`, and the cutting
/// plane meets them at `x = 2/0.875` and `x = −2/1.125`. Those two points are
/// the ends of the ellipse's long axis, so the centre is their midpoint — and
/// its `x` is nowhere near the axis, which crosses the plane at `x = 0`.
#[test]
fn the_ellipse_is_centred_midway_between_the_two_rulings_the_plane_meets() {
    let cone = Cone::new(Axis::z(), 0.5).expect("a positive slope");
    let flat = Plane::from_coefficients([-0.25, 0.0, 1.0, -4.0]).expect("a plane");

    let [x, y, z] = the_centre(&cone, &flat).expect("a plane at a quarter is flatter than z = 2x");

    let (first, second) = (4.0 * 0.5 / (1.0 - 0.125), -4.0 * 0.5 / (1.0 + 0.125));
    let want = [
        0.5 * (first + second),
        0.0,
        0.5 * (first / 0.5 + -second / 0.5),
    ];
    assert!((x - want[0]).abs() < 1e-12, "{x} against {}", want[0]);
    assert!((y - want[1]).abs() < 1e-12, "{y} against {}", want[1]);
    assert!((z - want[2]).abs() < 1e-12, "{z} against {}", want[2]);
    assert!(
        (x - 0.0).abs() > 0.25,
        "and it is not where the axis crosses the plane, which is x = 0"
    );
}

/// A cone square to its cutting plane keeps the old answer: the foot of the
/// perpendicular from the apex, which for a plane square to the axis is where
/// the axis crosses it.
#[test]
fn a_plane_square_to_the_axis_is_still_centred_where_the_axis_crosses_it() {
    let cone = Cone::new(
        Axis::new(Point::new(1.0, -2.0, 3.0), Vector::new(0.0, 0.0, 1.0)).expect("a direction"),
        0.5,
    )
    .expect("a positive slope");
    let flat = Plane::from_coefficients([0.0, 0.0, 1.0, -9.0]).expect("a plane");

    let centre = the_centre(&cone, &flat).expect("square is the tidiest closed case");
    for (at, want) in centre.iter().zip([1.0, -2.0, 9.0]) {
        assert!((at - want).abs() < 1e-12, "{centre:?}");
    }
}

/// A cone moved off the origin and leaned over, cut by a plane leaned the other
/// way: the claimed centre is the middle of every chord through it.
///
/// Two directions in the cutting plane are walked out from the centre until the
/// cone's own sign flips, and the two reaches are compared. Nothing here uses
/// the pole formula a second time — the only thing asked of the cone is
/// [`Cone::side_of`], which is the published sign.
#[test]
fn a_leaning_cone_is_cut_in_an_ellipse_the_claimed_centre_halves_every_chord_of() {
    let cone = Cone::new(
        Axis::new(Point::new(1.0, -2.0, 3.0), Vector::new(0.2, -0.3, 1.0)).expect("a direction"),
        0.6,
    )
    .expect("a positive slope");
    let flat = Plane::from_coefficients([0.1, 0.25, 1.0, -12.0]).expect("a plane");

    let centre = the_centre(&cone, &flat).expect("both leans are gentler than the cone's side");
    assert_eq!(
        cone.side_of(PredicatePoint::Exact(centre)),
        Decision::Neg,
        "the centre of a closed crossing is inside the cone"
    );

    let (across, and) = flat.basis();
    for way in [across, and, across + and] {
        let out = how_far_out_the_cone_reaches(&cone, centre, way.components());
        let back = how_far_out_the_cone_reaches(&cone, centre, way.components().map(|at| -at));
        assert!(
            (out - back).abs() < 1e-9 * out,
            "a chord through the centre is halved by it: {out} one way, {back} the other"
        );
    }
}

/// How far from `from` along `way` the cone's sign flips, by bisection.
fn how_far_out_the_cone_reaches(cone: &Cone, from: [f64; 3], way: [f64; 3]) -> f64 {
    let at = |reach: f64| {
        cone.side_of(PredicatePoint::Exact(core::array::from_fn(|axis| {
            from[axis] + reach * way[axis]
        })))
    };
    let mut outside = 1.0;
    while at(outside) == Decision::Neg {
        outside *= 2.0;
        assert!(
            outside < 1e6,
            "the crossing closes, so the sign has to flip"
        );
    }
    let mut inside = 0.0;
    for _ in 0..200 {
        let middle = 0.5 * (inside + outside);
        if at(middle) == Decision::Neg {
            inside = middle;
        } else {
            outside = middle;
        }
    }
    0.5 * (inside + outside)
}

/// The three crossings that are not closed have no centre to hand back, and the
/// parabola on the boundary has none at all.
#[test]
fn a_parabola_a_hyperbola_and_an_apex_on_the_plane_have_no_centre_here() {
    let cone = Cone::new(Axis::z(), 0.5).expect("a positive slope");

    let parabola = Plane::from_coefficients([-2.0, 0.0, 1.0, -4.0]).expect("a plane");
    assert_eq!(cone.is_cut_in_a_closed_curve_by(&parabola), Some(false));
    assert_eq!(
        cone.which_way_a_closed_crossings_centre_lies(&parabola),
        None,
        "a plane parallel to a ruling cuts a parabola, which has no centre"
    );

    let hyperbola = Plane::from_coefficients([-3.0, 0.0, 1.0, -4.0]).expect("a plane");
    assert_eq!(cone.is_cut_in_a_closed_curve_by(&hyperbola), Some(false));
    assert_eq!(
        cone.which_way_a_closed_crossings_centre_lies(&hyperbola),
        None,
        "a hyperbola has a centre, and no one closed curve for a chord to split"
    );

    let through_the_apex = Plane::from_coefficients([-0.25, 0.0, 1.0, 0.0]).expect("a plane");
    assert_eq!(
        cone.is_cut_in_a_closed_curve_by(&through_the_apex),
        Some(false)
    );
    assert_eq!(
        cone.which_way_a_closed_crossings_centre_lies(&through_the_apex),
        None,
        "the apex on the plane leaves rulings and not a closed curve"
    );
}
