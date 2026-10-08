//! Writing geometry out and reading it back.
//!
//! `postcard` is the format `docs/decisions.md` §6 picked, explicitly unstable.
//! What is tested here is not the byte layout — that is free to change — but
//! that a value survives the trip, and that the checks the constructors make
//! are made again on the way in. A file is untrusted input; a type whose
//! invariants only hold on the way out is a type that cannot be loaded safely.

use core::f64::consts::FRAC_PI_3;

use rcad_geom::{
    Axis, BSplineCurve, BSplineSurface, Cone, Cylinder, Isometry, KnotVector, Orientation, Plane,
    Point, Scale, Segment, Sphere, Surface, Torus, Unit, Vector,
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

/// An axis a turn left an ulp off length one comes back from its bytes with
/// the bits it was stored with, because the way in is `Axis::as_written` and
/// not `Axis::new`, which scales the direction again.
///
/// The three coordinate axes, each turned about `x`, `y`, `z` and `(1, 2, 3)`
/// by every tenth of a radian up to a whole turn, and moved off the origin:
/// `Axis::new` would have moved the direction of 175 of the 744, and the
/// bytes move none. The ring of the turned bend in `rcad-step`'s
/// `how_often_a_turned_bend_comes_back_from_its_own_file` is one of them,
/// `z` turned about `x` by `2.5` (`docs/decisions.md` §355.6).
#[test]
fn an_axis_a_turn_left_comes_back_from_its_bytes_with_the_bits_it_was_stored_with() {
    let bits = |axis: Axis| {
        let [x, y, z] = axis.origin().coordinates();
        let [u, v, w] = axis.direction().components();
        [x, y, z, u, v, w].map(f64::to_bits)
    };
    let about = [
        Axis::x(),
        Axis::y(),
        Axis::z(),
        Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("a direction"),
    ];
    let (mut turned, mut new_would_move, mut the_bytes_moved) = (0, 0, 0);
    for axis in [Axis::x(), Axis::y(), Axis::z()] {
        for spin in &about {
            for tenths in 1..63 {
                let turn = Isometry::translation(Vector::new(3.0, 5.0, 10.0))
                    .expect("a finite offset")
                    .compose(
                        &Isometry::rotation(spin, f64::from(tenths) * 0.1)
                            .expect("a finite angle about a unit axis"),
                    );
                let stored = axis.transformed(&turn);
                turned += 1;
                let scaled = Axis::new(stored.origin(), stored.direction()).expect("a finite axis");
                if bits(scaled) != bits(stored) {
                    new_would_move += 1;
                }
                if bits(there_and_back(stored)) != bits(stored) {
                    the_bytes_moved += 1;
                }
            }
        }
    }
    println!(
        "{turned} turned axes: Axis::new would move {new_would_move}, the bytes moved \
         {the_bytes_moved}"
    );
    assert_eq!((turned, new_would_move, the_bytes_moved), (744, 175, 0));
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

/// A cone comes back bit for bit, apex, axis and slope.
///
/// **The slope and not a half-angle, which is why this is an `assert_eq` and
/// could not be one otherwise.** A `Cone` stores `tan` of its half-angle so that
/// its sign predicate is a polynomial in stored numbers; if it stored the angle
/// and took the tangent on the way in, the round trip would depend on `tan`
/// being the same function on both machines, which Rust does not promise.
#[test]
fn a_cone_comes_back_with_the_same_apex_axis_and_slope() {
    let cone = Cone::new(
        Axis::new(Point::new(1.0, -2.0, 3.0), Vector::new(0.0, 0.0, 1.0))
            .expect("a direction with length"),
        0.75,
    )
    .expect("a positive slope");
    assert_eq!(there_and_back(cone), cone);
    assert_eq!(there_and_back(Surface::from(cone)), Surface::from(cone));
}

/// Every way a cone's numbers can be a cone's numbers and not name a cone, off
/// the wire.
///
/// `Model` derives `Deserialize`, so a cone arrives from a stranger's bytes and
/// every `# Panics` a cone can reach has to be a shape the arenas cannot hold.
/// `Cone::basis`, `Cone::point_at` and `Cone::coordinates_of` all divide by or
/// multiply through the axis direction and the slope, and
/// `#[serde(try_from = "(Axis, f64)")]` makes `Cone::new` the only route in, on
/// the wire as in code, and through it `Axis::as_written`, which keeps a
/// direction only when its length is within a hair of one and hands any other
/// to `Axis::new`'s checks.
#[test]
fn a_cone_stored_with_a_spread_that_is_not_a_spread_is_refused_on_the_way_in() {
    for slope in [
        0.0f64,
        -1.0,
        -0.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        let bytes = postcard::to_allocvec(&(Axis::z(), slope)).unwrap();
        assert!(
            postcard::from_bytes::<Cone>(&bytes).is_err(),
            "a slope of {slope} is not a cone"
        );
    }
    let no_direction = postcard::to_allocvec(&((Point::origin(), Vector::zero()), 1.0f64)).unwrap();
    assert!(postcard::from_bytes::<Cone>(&no_direction).is_err());

    let apex_at_infinity = postcard::to_allocvec(&(
        (
            Point::new(f64::INFINITY, 0.0, 0.0),
            Vector::new(0.0, 0.0, 1.0),
        ),
        1.0f64,
    ))
    .unwrap();
    assert!(postcard::from_bytes::<Cone>(&apex_at_infinity).is_err());
}

/// **`FORMAT_VERSION` does not move for the cone, and this is the reason.**
///
/// The two times it did move (§24.5, §33) a *field* changed shape, so a file
/// written at the old shape parsed cleanly at the new one and came back a
/// different solid — the one outcome §4.1 ranks below a crash. A fourth variant
/// on the end of `Surface` cannot do that: `postcard` writes an enum as its
/// variant index and then its payload, so `Plane`, `Sphere` and `Cylinder` keep
/// the indices they had and every byte of an old file still says what it said.
/// A *new* file holding a cone read by an older build meets an index that build
/// has no arm for, which `postcard` refuses rather than guesses.
///
/// The assertion is on the tag bytes because that is the whole of the claim.
/// `a_model_holding_a_cone_reads_back_at_the_version_that_was_already_there` in
/// `rcad-topo` is the other half, where the version number itself lives.
#[test]
fn a_cone_takes_the_next_tag_and_moves_nobody() {
    let plane = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
        .expect("the z axis is a unit normal");
    let ball = Sphere::new(Point::origin(), 1.0).expect("a radius");
    let pipe = Cylinder::new(Axis::z(), 1.0).expect("a radius");
    let cone = Cone::new(Axis::z(), 1.0).expect("a slope");

    let tag = |surface: Surface| postcard::to_allocvec(&surface).expect("a surface writes out")[0];
    assert_eq!(tag(Surface::from(plane)), 0);
    assert_eq!(tag(Surface::from(ball)), 1);
    assert_eq!(tag(Surface::from(pipe)), 2);
    assert_eq!(tag(Surface::from(cone)), 3);
}

/// A torus comes back bit for bit, centre, axis and both radii.
#[test]
fn a_torus_comes_back_with_the_same_centre_axis_and_two_radii() {
    let ring = Torus::new(
        Axis::new(Point::new(1.0, -2.0, 3.0), Vector::new(0.0, 0.0, 1.0))
            .expect("a direction with length"),
        4.0,
        1.5,
    )
    .expect("a tube narrower than its ring");
    assert_eq!(there_and_back(ring), ring);
    assert_eq!(there_and_back(Surface::from(ring)), Surface::from(ring));
}

/// Every way a torus's numbers can be a torus's numbers and not name a torus,
/// off the wire.
///
/// `Model` derives `Deserialize`, so a torus arrives from a stranger's bytes and
/// every `# Panics` a torus can reach has to be a shape the arenas cannot hold.
/// `Torus::basis` panics on an axis direction with no length, and
/// `#[serde(try_from = "(Axis, f64, f64)")]` makes `Torus::new` the only route
/// in, on the wire as in code, and through it `Axis::as_written`, which keeps a
/// direction only when its length is within a hair of one and hands any other
/// to `Axis::new`'s checks.
///
/// **The tube-against-ring row is the one no other surface has.** A ball, a pipe
/// and a cone each carry one size and it is either a size or it is not; a torus
/// carries two whose *relation* decides whether the numbers name a surface at
/// all, and `a_tube_wider_than_its_ring_would_certify_the_wrong_side` in
/// `torus.rs` is why letting one through would be worse than refusing it.
#[test]
fn a_torus_stored_with_radii_that_do_not_name_a_ring_is_refused_on_the_way_in() {
    for radius in [
        0.0f64,
        -1.0,
        -0.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        for pair in [(radius, 1.0f64), (4.0f64, radius)] {
            let bytes = postcard::to_allocvec(&(Axis::z(), pair.0, pair.1)).unwrap();
            assert!(
                postcard::from_bytes::<Torus>(&bytes).is_err(),
                "{pair:?} is not a pair of radii"
            );
        }
    }
    for (ring, tube) in [(1.0f64, 1.0f64), (1.0, 2.0)] {
        let bytes = postcard::to_allocvec(&(Axis::z(), ring, tube)).unwrap();
        assert!(
            postcard::from_bytes::<Torus>(&bytes).is_err(),
            "a tube of {tube} on a ring of {ring} runs through itself"
        );
    }

    let no_direction =
        postcard::to_allocvec(&((Point::origin(), Vector::zero()), 4.0f64, 1.0f64)).unwrap();
    assert!(postcard::from_bytes::<Torus>(&no_direction).is_err());

    let centre_at_infinity = postcard::to_allocvec(&(
        (
            Point::new(f64::INFINITY, 0.0, 0.0),
            Vector::new(0.0, 0.0, 1.0),
        ),
        4.0f64,
        1.0f64,
    ))
    .unwrap();
    assert!(postcard::from_bytes::<Torus>(&centre_at_infinity).is_err());
}

/// **`FORMAT_VERSION` does not move for the torus either**, and the reason is
/// the cone's word for word: a fifth variant on the end of `Surface` takes the
/// next `postcard` index and moves nobody.
///
/// The four tags that were there are asserted again beside the new one, because
/// what would break a file is one of *them* moving and not the new one arriving.
#[test]
fn a_torus_takes_the_next_tag_and_moves_nobody() {
    let plane = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
        .expect("the z axis is a unit normal");
    let ball = Sphere::new(Point::origin(), 1.0).expect("a radius");
    let pipe = Cylinder::new(Axis::z(), 1.0).expect("a radius");
    let cone = Cone::new(Axis::z(), 1.0).expect("a slope");
    let ring = Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring");

    let tag = |surface: Surface| postcard::to_allocvec(&surface).expect("a surface writes out")[0];
    assert_eq!(tag(Surface::from(plane)), 0);
    assert_eq!(tag(Surface::from(ball)), 1);
    assert_eq!(tag(Surface::from(pipe)), 2);
    assert_eq!(tag(Surface::from(cone)), 3);
    assert_eq!(tag(Surface::from(ring)), 4);
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
