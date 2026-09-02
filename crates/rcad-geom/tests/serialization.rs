//! Writing geometry out and reading it back.
//!
//! `postcard` is the format `docs/decisions.md` §6 picked, explicitly unstable.
//! What is tested here is not the byte layout — that is free to change — but
//! that a value survives the trip, and that the checks the constructors make
//! are made again on the way in. A file is untrusted input; a type whose
//! invariants only hold on the way out is a type that cannot be loaded safely.

use core::f64::consts::FRAC_PI_3;

use rcad_geom::{
    Axis, BSplineCurve, BSplineSurface, Cylinder, Isometry, KnotVector, Orientation, Plane, Point,
    Scale, Segment, Sphere, Surface, Unit, Vector,
};

fn there_and_back<T>(value: T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let bytes = postcard::to_allocvec(&value).expect("geometry serializes");
    postcard::from_bytes(&bytes).expect("what we just wrote must read back")
}

#[test]
fn a_point_and_a_vector_come_back_exactly_as_they_went_out() {
    let point = Point::new(1.5, -2.5, 3.5);
    assert_eq!(there_and_back(point), point);

    let vector = Vector::new(0.1, 0.2, 0.3);
    assert_eq!(there_and_back(vector), vector);
}

#[test]
fn an_axis_comes_back_pointing_the_same_way() {
    let axis = Axis::new(Point::new(1.0, 2.0, 3.0), Vector::new(0.0, 0.0, 4.0)).unwrap();
    assert_eq!(there_and_back(axis), axis);
}

#[test]
fn an_orientation_comes_back_the_same_way_round() {
    for orientation in [Orientation::Forward, Orientation::Reversed] {
        assert_eq!(there_and_back(orientation), orientation);
    }
}

/// Not bit for bit: the stored rotation is scaled back to length one when it is
/// read, and scaling something already very nearly unit length can move the
/// last bit. What has to survive is where the movement puts things.
#[test]
fn a_movement_comes_back_moving_points_to_the_same_places() {
    let movement = Isometry::rotation(&Axis::z(), FRAC_PI_3)
        .unwrap()
        .compose(&Isometry::translation(Vector::new(10.0, 20.0, 30.0)).unwrap());
    let recovered = there_and_back(movement);

    for corner in [
        Point::origin(),
        Point::new(100.0, 0.0, 0.0),
        Point::new(-3.0, 7.5, 42.0),
    ] {
        assert!(
            recovered
                .apply_to_point(corner)
                .distance_to(movement.apply_to_point(corner))
                < 1e-13
        );
    }
}

#[test]
fn a_movement_about_a_coordinate_axis_comes_back_bit_for_bit() {
    let quarter_turn = Isometry::rotation(&Axis::z(), core::f64::consts::FRAC_PI_2).unwrap();
    assert_eq!(there_and_back(quarter_turn), quarter_turn);
}

#[test]
fn the_three_surfaces_come_back_as_the_surfaces_they_were() {
    let plane = Plane::from_point_and_normal(Point::new(0.0, 0.0, 2.0), Vector::new(0.0, 0.0, 1.0))
        .unwrap();
    let ball = Sphere::new(Point::new(1.0, 2.0, 3.0), 4.0).unwrap();
    let pipe = Cylinder::new(Axis::x(), 0.5).unwrap();

    assert_eq!(there_and_back(plane), plane);
    assert_eq!(there_and_back(ball), ball);
    assert_eq!(there_and_back(pipe), pipe);

    for surface in [
        Surface::from(plane),
        Surface::from(ball),
        Surface::from(pipe),
    ] {
        assert_eq!(there_and_back(surface), surface);
    }
}

#[test]
fn an_edge_comes_back_running_between_the_same_two_points() {
    let segment = Segment::new(Point::origin(), Point::new(3.0, 4.0, 0.0)).unwrap();
    let recovered = there_and_back(segment);
    assert_eq!(recovered, segment);
    assert_eq!(recovered.length(), 5.0);
}

#[test]
fn a_scale_comes_back_with_its_unit_and_its_tolerance() {
    for unit in [
        Unit::Micrometre,
        Unit::Millimetre,
        Unit::Centimetre,
        Unit::Metre,
        Unit::Inch,
        Unit::Foot,
    ] {
        let scale = Scale::new(unit, 100.0).unwrap();
        let recovered = there_and_back(scale);
        assert_eq!(recovered, scale);
        assert_eq!(
            recovered.default_linear_tolerance(),
            scale.default_linear_tolerance()
        );
    }
}

#[test]
fn an_axis_stored_without_a_direction_is_refused_on_the_way_in() {
    let bytes = postcard::to_allocvec(&(Point::origin(), Vector::zero())).unwrap();
    assert!(postcard::from_bytes::<Axis>(&bytes).is_err());
}

#[test]
fn a_movement_stored_with_no_rotation_at_all_is_refused_on_the_way_in() {
    let bytes = postcard::to_allocvec(&([0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0])).unwrap();
    assert!(postcard::from_bytes::<Isometry>(&bytes).is_err());
}

#[test]
fn a_movement_stored_with_a_stretched_rotation_is_scaled_back_to_a_rigid_one() {
    let bytes = postcard::to_allocvec(&([0.0, 0.0, 0.0], [2.0, 0.0, 0.0, 0.0])).unwrap();
    let recovered: Isometry = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(recovered, Isometry::identity());
}

#[test]
fn a_plane_stored_with_no_normal_is_refused_on_the_way_in() {
    let bytes = postcard::to_allocvec(&[0.0f64, 0.0, 0.0, 1.0]).unwrap();
    assert!(postcard::from_bytes::<Plane>(&bytes).is_err());
}

#[test]
fn a_ball_stored_with_no_radius_is_refused_on_the_way_in() {
    let bytes = postcard::to_allocvec(&(Point::origin(), 0.0f64)).unwrap();
    assert!(postcard::from_bytes::<Sphere>(&bytes).is_err());
}

#[test]
fn a_pipe_stored_with_a_negative_radius_is_refused_on_the_way_in() {
    let bytes = postcard::to_allocvec(&(Axis::z(), -1.0f64)).unwrap();
    assert!(postcard::from_bytes::<Cylinder>(&bytes).is_err());
}

/// `Cylinder::basis` panics if the stored axis has no direction to give, and
/// says the constructors do not allow that. `Axis` is the type that carries
/// that check, and it carries it on the way in too: a `Cylinder` can only be
/// built around an already-checked `Axis` — `#[serde(try_from = "(Axis, f64)")]`
/// makes it the only route, on the wire or in code — so bytes that describe a
/// direction with no length are refused here, before a `Cylinder`, let alone
/// its `basis`, ever exists to be asked.
#[test]
fn a_pipe_stored_with_a_zero_length_axis_direction_is_refused_on_the_way_in() {
    let bytes = postcard::to_allocvec(&((Point::origin(), Vector::zero()), 3.0f64)).unwrap();
    assert!(postcard::from_bytes::<Cylinder>(&bytes).is_err());
}

#[test]
fn an_edge_stored_with_both_ends_in_the_same_place_is_refused_on_the_way_in() {
    let bytes = postcard::to_allocvec(&(Point::origin(), Point::origin())).unwrap();
    assert!(postcard::from_bytes::<Segment>(&bytes).is_err());
}

#[test]
fn a_scale_stored_with_no_size_is_refused_on_the_way_in() {
    let bytes = postcard::to_allocvec(&(Unit::Millimetre, 0.0f64)).unwrap();
    assert!(postcard::from_bytes::<Scale>(&bytes).is_err());
}

/// Splines are the one place where the round trip has to be bit for bit rather
/// than close enough: they are stored in the same homogeneous coordinates they
/// are worked in, so nothing is scaled on the way in or out.
#[test]
fn a_knot_vector_comes_back_knot_for_knot() {
    let knots = KnotVector::new(
        3,
        vec![0.0, 0.0, 0.0, 0.0, 0.1, 0.2, 0.2, 1.0, 1.0, 1.0, 1.0],
    )
    .unwrap();
    let recovered = there_and_back(knots.clone());
    assert_eq!(recovered, knots);
    assert_eq!(recovered.knots(), knots.knots());
}

#[test]
fn a_spline_curve_comes_back_with_every_stored_number_untouched() {
    let curve = a_weighted_arc();
    let recovered = there_and_back(curve.clone());
    assert_eq!(recovered, curve);
    for i in 0..curve.control_point_count() {
        assert_eq!(
            recovered.homogeneous_control_point(i),
            curve.homogeneous_control_point(i)
        );
    }
}

#[test]
fn a_spline_surface_comes_back_with_every_stored_number_untouched() {
    let surface = a_weighted_wall();
    let recovered = there_and_back(surface.clone());
    assert_eq!(recovered, surface);
    for i in 0..3 {
        for j in 0..2 {
            assert_eq!(
                recovered.homogeneous_control_point(i, j),
                surface.homogeneous_control_point(i, j)
            );
        }
    }
}

#[test]
fn a_knot_vector_stored_without_its_clamps_is_refused_on_the_way_in() {
    let bytes = postcard::to_allocvec(&(2usize, vec![0.0f64, 0.0, 0.5, 1.0, 1.0, 1.0])).unwrap();
    assert!(postcard::from_bytes::<KnotVector>(&bytes).is_err());
}

#[test]
fn a_knot_vector_stored_with_its_knots_out_of_order_is_refused_on_the_way_in() {
    let bytes = postcard::to_allocvec(&(1usize, vec![0.0f64, 0.0, 2.0, 1.0, 3.0, 3.0])).unwrap();
    assert!(postcard::from_bytes::<KnotVector>(&bytes).is_err());
}

#[test]
fn a_knot_vector_stored_at_a_degree_this_kernel_does_not_carry_is_refused_on_the_way_in() {
    let bytes = postcard::to_allocvec(&(0usize, vec![0.0f64, 1.0])).unwrap();
    assert!(postcard::from_bytes::<KnotVector>(&bytes).is_err());
}

/// The stored net is homogeneous, so a weight of zero is a control point that
/// cannot be divided back out — and a stored curve carrying one would panic
/// during evaluation instead of failing here.
#[test]
fn a_curve_stored_with_a_weight_of_zero_is_refused_on_the_way_in() {
    let knots = KnotVector::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]).unwrap();
    let net = vec![
        [1.0f64, 0.0, 0.0, 1.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 1.0],
    ];
    let bytes = postcard::to_allocvec(&(knots, net)).unwrap();
    assert!(postcard::from_bytes::<BSplineCurve>(&bytes).is_err());
}

#[test]
fn a_curve_stored_with_the_wrong_number_of_control_points_is_refused_on_the_way_in() {
    let knots = KnotVector::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]).unwrap();
    let net = vec![[1.0f64, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]];
    let bytes = postcard::to_allocvec(&(knots, net)).unwrap();
    assert!(postcard::from_bytes::<BSplineCurve>(&bytes).is_err());
}

#[test]
fn a_surface_stored_with_the_wrong_sized_grid_is_refused_on_the_way_in() {
    let surface = a_weighted_wall();
    let (u_knots, v_knots, mut net) = <(KnotVector, KnotVector, Vec<[f64; 4]>)>::from(surface);
    net.pop();
    let bytes = postcard::to_allocvec(&(u_knots, v_knots, net)).unwrap();
    assert!(postcard::from_bytes::<BSplineSurface>(&bytes).is_err());
}

/// Weights that are not one, and coordinates that do not survive being
/// multiplied by them and divided back out.
fn a_weighted_arc() -> BSplineCurve {
    let knots = KnotVector::new(2, vec![0.0, 0.0, 0.0, 0.25, 1.0, 1.0, 1.0]).unwrap();
    let corners = [
        Point::new(0.1, 0.0, 0.0),
        Point::new(1.0, 1.0, 0.0),
        Point::new(0.0, 1.0, 0.3),
        Point::new(-2.0, 1.0, 0.0),
    ];
    BSplineCurve::new(knots, &corners, &[1.0, 3.0, 0.5, 7.0]).unwrap()
}

fn a_weighted_wall() -> BSplineSurface {
    let u_knots = KnotVector::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]).unwrap();
    let v_knots = KnotVector::new(1, vec![0.0, 0.0, 1.0, 1.0]).unwrap();
    let corners = [
        Point::new(1.0, 0.0, 0.0),
        Point::new(1.0, 0.0, 1.0),
        Point::new(1.0, 1.0, 0.0),
        Point::new(1.0, 1.0, 1.0),
        Point::new(0.0, 1.0, 0.0),
        Point::new(0.0, 1.0, 1.0),
    ];
    BSplineSurface::new(u_knots, v_knots, &corners, &[1.0, 1.0, 3.0, 3.0, 2.0, 2.0]).unwrap()
}

#[test]
fn a_bare_point_carries_no_invariant_so_even_a_broken_one_survives_the_trip() {
    let broken = Point::new(f64::NAN, 0.0, 0.0);
    assert!(there_and_back(broken).x().is_nan());
}

#[test]
fn a_coordinate_that_is_not_a_number_is_refused_by_everything_built_on_points() {
    let broken = Point::new(f64::NAN, 0.0, 0.0);

    let as_an_axis = postcard::to_allocvec(&(broken, Vector::new(0.0, 0.0, 1.0))).unwrap();
    assert!(postcard::from_bytes::<Axis>(&as_an_axis).is_err());

    let as_an_edge = postcard::to_allocvec(&(broken, Point::new(1.0, 0.0, 0.0))).unwrap();
    assert!(postcard::from_bytes::<Segment>(&as_an_edge).is_err());

    let as_a_ball = postcard::to_allocvec(&(broken, 1.0f64)).unwrap();
    assert!(postcard::from_bytes::<Sphere>(&as_a_ball).is_err());
}
