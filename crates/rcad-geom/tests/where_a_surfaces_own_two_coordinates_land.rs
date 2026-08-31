//! What `Surface::point_at` promises, and the one bound a mesher sizes by.
//!
//! Two claims are load-bearing here and both are checked rather than asserted
//! in a comment. That `u = 0` is where a bar's seam is, so the seam sits on the
//! edge of the face's rectangle instead of across the middle of it. And that
//! `furthest_a_unit_step_goes` is genuinely a ceiling, because a mesher that
//! sized a step by a number the surface could exceed would place points further
//! apart than it meant to and quietly break its own deviation bound.

use rcad_geom::{Axis, Cylinder, Point, Sphere, Surface, Vector};
use rcad_predicates::{Decision, SideOfSurface};

/// `u = 0` on a pipe about `z` is `+x`, exactly, which is where a round bar
/// puts its seam.
///
/// If this ever moves, a cylindrical face's `(u, v)` rectangle stops having its
/// seam down one edge and the mesher has to cut the domain in two.
#[test]
fn the_start_of_the_way_round_a_pipe_is_where_a_bars_seam_is() {
    let pipe = Cylinder::new(Axis::z(), 3.0).expect("a pipe of radius three");
    assert_eq!(pipe.point_at(0.0, 0.0), Point::new(3.0, 0.0, 0.0));
    assert_eq!(pipe.point_at(0.0, 5.0), Point::new(3.0, 0.0, 5.0));

    let (across, and) = pipe.basis();
    assert_eq!(across, Vector::new(1.0, 0.0, 0.0));
    assert_eq!(and, Vector::new(0.0, 1.0, 0.0), "and the two cross to +z");
}

/// The same for a ball: longitude zero on the equator is `+x`.
#[test]
fn the_start_of_the_way_round_a_ball_is_also_x() {
    let ball = Sphere::new(Point::origin(), 3.0).expect("a ball of radius three");
    assert_eq!(ball.point_at(0.0, 0.0), Point::new(3.0, 0.0, 0.0));
}

/// Where the trigonometry is exact, the point is *certified* onto the surface —
/// not near it.
#[test]
fn a_point_where_the_arithmetic_is_exact_is_certified_onto_its_surface() {
    for radius in [1.0, 3.0, 0.1, 1e8] {
        let pipe = Cylinder::new(Axis::z(), radius).expect("a pipe");
        for along in [0.0, 1.0, -4.0] {
            let at = pipe.point_at(0.0, along);
            assert_eq!(
                pipe.side_of(at.coordinates().into()),
                Decision::Zero,
                "the seam of a pipe of radius {radius} at height {along}"
            );
        }

        let ball = Sphere::new(Point::origin(), radius).expect("a ball");
        assert_eq!(
            ball.side_of(ball.point_at(0.0, 0.0).coordinates().into()),
            Decision::Zero,
            "longitude zero on the equator of a ball of radius {radius}"
        );
    }
}

/// The axial coordinate of a pipe comes back exactly, whatever the angle.
///
/// The two directions across the pipe have no component along the axis, so the
/// round trip is one multiplication and nothing to cancel.
#[test]
fn going_round_a_pipe_never_moves_you_along_it() {
    let pipe = Cylinder::new(Axis::z(), 3.0).expect("a pipe of radius three");
    for along in [0.0, 1.0, -4.0, 1e6] {
        for round in [0.0, 0.5, 1.0, 2.0, 3.0, 6.0, -2.5] {
            assert_eq!(
                pipe.point_at(round, along).coordinates()[2],
                along,
                "at angle {round}"
            );
        }
    }
}

/// The bound, on the surface it is exactly right for.
///
/// A cylinder unrolls flat, so one unit of `u` is exactly `radius` of arc. The
/// straight line between two points that far apart is the chord, which is
/// shorter than the arc — so the bound holds with room to spare, and the room
/// shrinks to nothing as the step does.
#[test]
fn no_step_round_a_pipe_goes_further_than_the_bound_says() {
    let pipe = Cylinder::new(Axis::z(), 3.0).expect("a pipe of radius three");
    let surface = Surface::from(pipe);
    let (per_round, per_along) = surface.furthest_a_unit_step_goes();
    assert_eq!(per_round, 3.0, "one radian carries you one radius");
    assert_eq!(per_along, 1.0, "and a unit along the axis is a unit");

    for step in [1.0, 0.5, 0.1, 0.01, 1e-4] {
        for from in [0.0, 1.0, 2.5, -3.0] {
            let moved = pipe
                .point_at(from, 0.0)
                .distance_to(pipe.point_at(from + step, 0.0));
            assert!(
                moved <= per_round * step,
                "a step of {step} from {from} moved {moved}, past the bound of {}",
                per_round * step
            );
        }
    }
}

/// And on the surface it is only a ceiling for.
///
/// A ball does not unroll. A step in longitude covers a whole radius at the
/// equator and nothing at all at a pole, so the bound is right at one latitude
/// and generous everywhere else. That is the cost of a domain that cannot be
/// made faithful, and it is paid in triangles.
#[test]
fn no_step_round_a_ball_goes_further_than_the_bound_says_either() {
    let ball = Sphere::new(Point::origin(), 3.0).expect("a ball of radius three");
    let (per_round, per_up) = Surface::from(ball).furthest_a_unit_step_goes();
    assert_eq!((per_round, per_up), (3.0, 3.0));

    for step in [1.0, 0.1, 0.01] {
        for up in [0.0, 0.5, 1.0, 1.5, -1.2] {
            let round_the_ball = ball.point_at(0.0, up).distance_to(ball.point_at(step, up));
            assert!(
                round_the_ball <= per_round * step,
                "a longitude step of {step} at latitude {up} moved {round_the_ball}"
            );

            let over_the_top = ball
                .point_at(0.0, up)
                .distance_to(ball.point_at(0.0, up + step));
            assert!(
                over_the_top <= per_up * step,
                "a latitude step of {step} from {up} moved {over_the_top}"
            );
        }
    }
}

/// The generosity is real and worth a number, because it is what the ball pays
/// in triangles near its poles.
#[test]
fn a_step_near_a_balls_pole_covers_almost_none_of_what_the_bound_allows() {
    let ball = Sphere::new(Point::origin(), 1.0).expect("a ball of radius one");
    let (per_round, _) = Surface::from(ball).furthest_a_unit_step_goes();

    let at_the_equator = ball
        .point_at(0.0, 0.0)
        .distance_to(ball.point_at(0.01, 0.0));
    let near_the_pole = ball
        .point_at(0.0, 1.56)
        .distance_to(ball.point_at(0.01, 1.56));

    assert!(
        at_the_equator > 0.99 * per_round * 0.01,
        "at the equator the bound is nearly tight, and this covered {at_the_equator}"
    );
    assert!(
        near_the_pole < 0.02 * per_round * 0.01,
        "a hair from the pole the same step covers almost nothing, and this covered \
         {near_the_pole}"
    );
}

/// A plane's two coordinates are already distances, so its bound is one.
#[test]
fn a_step_across_a_plane_is_the_distance_it_says_it_is() {
    let flat = rcad_geom::Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
        .expect("the z axis is a unit normal");
    let surface = Surface::from(flat);
    assert_eq!(surface.furthest_a_unit_step_goes(), (1.0, 1.0));

    for step in [1.0, 0.5, 7.0] {
        let moved = surface
            .point_at(0.0, 0.0)
            .distance_to(surface.point_at(step, 0.0));
        assert_eq!(moved, step, "a plane's coordinates are already lengths");
    }
}
