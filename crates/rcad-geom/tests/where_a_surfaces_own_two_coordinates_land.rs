//! What `Surface::point_at` promises, and the one bound a mesher sizes by.
//!
//! Three claims are load-bearing here and all three are checked rather than
//! asserted in a comment. That `u = 0` is where a bar's seam is, so the seam
//! sits on the edge of the face's rectangle instead of across the middle of it.
//! That `furthest_a_unit_step_goes` is genuinely a ceiling, because a mesher
//! that sized a step by a number the surface could exceed would place points
//! further apart than it meant to and quietly break its own deviation bound.
//! And which way `∂u × ∂v` points on each of the five surfaces, because a
//! caller reading a normal off the two coordinates gets the material's outside
//! on four of them and its inside on the fifth.

use rcad_geom::{Axis, Cone, Cylinder, Plane, Point, Sphere, Surface, Torus, Vector};
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
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
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

/// The table `docs/consumers.md` publishes: `∂u × ∂v` of `point_at` points out
/// of the material on a plane, a ball, a pipe and a cone, and into it on a
/// torus.
///
/// **This is the one to read when it fails.** A failure here says one surface's
/// handedness moved, so every normal a caller read off that parameterization is
/// now inverted. Which surface is in the message.
///
/// The measurement never asks the parameterization which way is out — that would
/// be the same question twice. It differences `point_at` itself for the two
/// directions, crosses them, and hands a short step either way to
/// `SideOfSurface::side_of` and `Plane::side_of_point`, which know nothing about
/// `(u, v)`: forwards must land [`Decision::Pos`] and backwards
/// [`Decision::Neg`], or the surface faces the other way.
///
/// A torus is the odd one out because its two coordinates are ordered
/// tube-first, so a bend of pipe charts like a straight bar — `Torus::point_at`
/// says why under "Round the tube comes first, and that is not arbitrary", and
/// this test does not re-argue it. Crossing the tube into the ring is the
/// opposite hand from crossing the ring into the tube, and the ordering is worth
/// more than the hand.
///
/// # Where it does not sample, and why
///
/// **A ball's two poles**, at latitude `±π/2`, where every longitude names one
/// point and the cross product shrinks to nothing. Sampled to `±1.2` radians,
/// which is a fifth of a turn clear of them. **A cone's apex**, at `along = 0`,
/// where the cross product is exactly zero for the same reason. Sampled from
/// `±0.75` outwards, on both nappes, because a cone has two and the second is
/// where a sign that came from `along` rather than from the geometry would show
/// up. A pipe and a torus pinch nowhere, so those two are sampled right round.
#[test]
fn every_surface_but_a_torus_crosses_its_two_coordinates_out_of_the_material() {
    let [flat, ball, pipe, taper, ring] = the_five_surfaces();

    let checked = every_sample_faces(&flat, Facing::OutOfTheMaterial)
        + every_sample_faces(&ball, Facing::OutOfTheMaterial)
        + every_sample_faces(&pipe, Facing::OutOfTheMaterial)
        + every_sample_faces(&taper, Facing::OutOfTheMaterial)
        + every_sample_faces(&ring, Facing::IntoTheMaterial);

    assert_eq!(
        checked, 94,
        "a spread that quietly emptied would agree with the table on nothing at all"
    );
}

/// The five do not all cross the same way, which is the single bit a caller
/// differentiating a parameterization has to carry a per-surface sign for.
///
/// **This is a separate test because a failure here is not a regression.** It
/// says the five have come into agreement, and the news is that anyone carrying
/// a per-surface sign to cope with the odd one out can drop it — a consumer's
/// decision to reopen, not a bug to fix. Inside the table above that message
/// would be buried and read as an inverted normal.
///
/// It asserts strictly less than the table does, on purpose, and it says nothing
/// about which surface is the odd one out.
#[test]
fn the_five_surfaces_do_not_all_cross_their_two_coordinates_the_same_way() {
    let facings = the_five_surfaces().map(|(surface, spots)| {
        let (u, v) = spots[0];
        which_way_the_two_coordinates_cross(&surface, u, v)
    });

    assert!(
        facings.iter().any(|facing| *facing != facings[0]),
        "all five surfaces now cross {:?}, so a per-surface sign has stopped earning its keep",
        facings[0]
    );
}

/// A plane, a ball, a pipe, a cone and a torus, each with the `(u, v)` to sample
/// it at.
fn the_five_surfaces() -> [(Surface, Vec<(f64, f64)>); 5] {
    let flat = Plane::from_point_and_normal(Point::new(0.0, 0.0, 2.0), Vector::new(0.0, 0.0, 1.0))
        .expect("the z axis is a unit normal");
    let ball = Sphere::new(Point::new(1.0, -2.0, 0.5), 3.0).expect("a ball of radius three");
    let pipe = Cylinder::new(Axis::z(), 3.0).expect("a pipe of radius three");
    let taper = Cone::new(Axis::z(), 0.5).expect("a cone of slope one half");
    let ring =
        Torus::new(Axis::z(), 5.0, 1.5).expect("a ring with a tube of radius one and a half");

    let right_round = [0.0, 0.9, 2.1, -2.7, 3.0];
    let clear_of_a_balls_poles = [-1.2, -0.4, 0.0, 0.4, 1.2];
    let clear_of_a_cones_apex = [-2.0, -0.75, 0.75, 2.0];

    [
        (
            Surface::from(flat),
            spread(&[-4.0, 0.0, 7.0], &[-1.0, 0.0, 2.5]),
        ),
        (
            Surface::from(ball),
            spread(&right_round, &clear_of_a_balls_poles),
        ),
        (Surface::from(pipe), spread(&right_round, &[-6.0, 0.0, 4.0])),
        (
            Surface::from(taper),
            spread(&right_round, &clear_of_a_cones_apex),
        ),
        (Surface::from(ring), spread(&right_round, &right_round)),
    ]
}

/// Every pairing of one list with the other.
fn spread(us: &[f64], vs: &[f64]) -> Vec<(f64, f64)> {
    us.iter()
        .flat_map(|&u| vs.iter().map(move |&v| (u, v)))
        .collect()
}

/// Checks every spot of one surface and hands back how many it checked, so the
/// caller can insist the run was not empty.
fn every_sample_faces((surface, spots): &(Surface, Vec<(f64, f64)>), expected: Facing) -> usize {
    for &(u, v) in spots {
        assert_eq!(
            which_way_the_two_coordinates_cross(surface, u, v),
            expected,
            "at ({u}, {v}) on {surface:?}"
        );
    }
    spots.len()
}

/// Which side of the surface a step along `∂u × ∂v` lands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Facing {
    OutOfTheMaterial,
    IntoTheMaterial,
}

/// `∂u × ∂v` by differencing `point_at`, then a step either way from the point
/// to ask a predicate which side that direction leads to.
///
/// The two half-steps are `1e-5` of a coordinate and the step off the surface is
/// `1e-6` of a length. A central difference is out by the square of its step, so
/// the direction is good to about a part in `1e10` and the sideways slip that
/// costs is `1e-16` of a length — ten orders of magnitude short of undoing the
/// step it is added to. `point_at` itself lands within a rounding of the skin,
/// which is smaller again.
fn which_way_the_two_coordinates_cross(surface: &Surface, u: f64, v: f64) -> Facing {
    let half_step = 1e-5;
    let along_u = surface.point_at(u + half_step, v) - surface.point_at(u - half_step, v);
    let along_v = surface.point_at(u, v + half_step) - surface.point_at(u, v - half_step);
    let crossed = along_u
        .cross(along_v)
        .normalized()
        .expect("neither a pole nor an apex is sampled, so the two directions are not parallel");

    let from = surface.point_at(u, v);
    let off_the_surface = 1e-6;
    let forwards = side_of(surface, from + crossed * off_the_surface);
    let backwards = side_of(surface, from - crossed * off_the_surface);
    match (forwards, backwards) {
        (Decision::Pos, Decision::Neg) => Facing::OutOfTheMaterial,
        (Decision::Neg, Decision::Pos) => Facing::IntoTheMaterial,
        (forwards, backwards) => panic!(
            "a step either way from ({u}, {v}) on {surface:?} came back {forwards:?} and \
             {backwards:?}, so the oracle decided nothing and no side was measured"
        ),
    }
}

/// The independent oracle: inside is [`Decision::Neg`], outside
/// [`Decision::Pos`], and none of these five ever look at `(u, v)`.
fn side_of(surface: &Surface, point: Point) -> Decision {
    let at: rcad_predicates::Point = point.coordinates().into();
    match surface {
        Surface::Plane(flat) => flat.side_of_point(at),
        Surface::Sphere(ball) => ball.side_of(at),
        Surface::Cylinder(pipe) => pipe.side_of(at),
        Surface::Cone(taper) => taper.side_of(at),
        Surface::Torus(ring) => ring.side_of(at),
        _ => panic!("a surface this test has never been shown: {surface:?}"),
    }
}
