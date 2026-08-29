//! Writing geometry out and reading it back.
//!
//! `postcard` is the format `docs/decisions.md` §6 picked, explicitly unstable.
//! What is tested here is not the byte layout — that is free to change — but
//! that a value survives the trip, and that the checks the constructors make
//! are made again on the way in. A file is untrusted input; a type whose
//! invariants only hold on the way out is a type that cannot be loaded safely.

use core::f64::consts::FRAC_PI_3;

use rcad_geom::{
    Axis, Cylinder, Isometry, Orientation, Plane, Point, Scale, Segment, Sphere, Surface, Unit,
    Vector,
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
