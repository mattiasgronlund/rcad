//! `Surface::coordinates_of` undoes `Surface::point_at`, and what "undoes"
//! costs.
//!
//! The sibling of `where_a_surfaces_own_two_coordinates_land.rs`, which is
//! about the way out. This is about the way back.
//!
//! Answering the four questions of `docs/decisions.md` §14.1:
//!
//! 1. **Exactly true, or bounded?** Bounded, and
//!    `Surface::ROUND_TRIP_ROUNDOFF` is the bound under test. Nothing here is
//!    exact: a point goes through an `atan2` and comes back through a `sin` and
//!    a `cos`, and none of the three is exact anywhere interesting.
//! 2. **Do the two runs take different paths?** The oracle is the point itself,
//!    which never went through either function. And the points are made two
//!    ways — by `point_at`, and by evaluating the spline patches in `support`,
//!    which reach the same skins through de Boor and share not one line with
//!    the trigonometry.
//! 3. **What family of inputs?** Balls and pipes whose radius runs over
//!    thirteen orders of magnitude, placed anywhere out to `1e10` and turned
//!    any way, with points drawn all over them; plus the awkward places one at
//!    a time — a seam, a pole, an axis, a centre, and a point nowhere near the
//!    surface at all.
//! 4. **Which gap would this have caught?** §12.6's fourth — a constant
//!    published with a derivation and nothing checking it — in the shape §19
//!    keeps finding it. A bound stated in `radius · ε` and nothing else reads
//!    perfectly well and is false: the angle a pipe reads off a point is a
//!    difference of terms the size of the whole model, so a pipe of radius one
//!    a long way out loses the model's worth of bits to cancellation and not
//!    one radius' worth.
//!
//! What this cannot do is show the constant is tight. Roundings cancel, so a
//! drawn case lands well inside the worst case; the derivation beside the
//! constant is what makes it a bound, and
//! `the_worst_round_trip_this_suite_can_find_is_well_inside_the_bound` says how
//! much room the drawn ones leave.

use proptest::prelude::*;
use rcad_geom::{Axis, Cylinder, Isometry, Plane, Point, Sphere, Surface, Vector};

mod support;

/// How far a round trip moved `point`, as a fraction of what
/// [`Surface::ROUND_TRIP_ROUNDOFF`] allows at that size.
///
/// One is the bound. Anything above it is a broken promise.
fn how_much_of_the_bound_a_round_trip_used(surface: &Surface, point: Point) -> f64 {
    let (u, v) = surface.coordinates_of(point);
    let allowed = Surface::ROUND_TRIP_ROUNDOFF * the_largest_number_involved(surface, point);
    assert!(allowed > 0.0, "every surface here has a size");
    point.distance_to(surface.point_at(u, v)) / allowed
}

/// The size the bound is stated against: the largest magnitude among the
/// point's coordinates, the numbers the surface places itself with, and the
/// radius.
fn the_largest_number_involved(surface: &Surface, point: Point) -> f64 {
    let of_the_surface = match surface {
        Surface::Plane(flat) => flat.offset().abs(),
        Surface::Sphere(ball) => ball
            .centre()
            .largest_coordinate_magnitude()
            .max(ball.radius()),
        Surface::Cylinder(pipe) => pipe
            .axis()
            .origin()
            .largest_coordinate_magnitude()
            .max(pipe.radius()),
        Surface::Cone(cone) => cone.apex().largest_coordinate_magnitude().max(cone.slope()),
        other => panic!("this suite does not know how to size {other:?}"),
    };
    of_the_surface.max(point.largest_coordinate_magnitude())
}

fn assert_the_round_trip_stayed_inside_the_bound(surface: &Surface, point: Point, what: &str) {
    let used = how_much_of_the_bound_a_round_trip_used(surface, point);
    assert!(
        used <= 1.0,
        "{what} moved {used} times what the bound allows"
    );
}

/// A point on a pipe goes into the two coordinates and comes back out.
///
/// Every angle, including the two where a sine or a cosine is at its worst, and
/// heights far enough up the axis that the angle is read out of numbers a
/// million times bigger than the radius.
#[test]
fn a_point_on_a_pipe_comes_back_to_where_it_was() {
    for radius in [1.0, 3.0, 0.001, 1e6] {
        let pipe = Cylinder::new(Axis::z(), radius).expect("a pipe");
        let surface = Surface::from(pipe);
        for round in [0.0, 0.3, 1.0, core::f64::consts::FRAC_PI_2, 3.0, -2.5, 3.1] {
            for along in [0.0, 1.0, -4.0, 1e6] {
                assert_the_round_trip_stayed_inside_the_bound(
                    &surface,
                    pipe.point_at(round, along),
                    &format!("a pipe of radius {radius} at ({round}, {along})"),
                );
            }
        }
    }
}

/// The same for a ball, poles included.
#[test]
fn a_point_on_a_ball_comes_back_to_where_it_was() {
    for radius in [1.0, 3.0, 0.001, 1e6] {
        let ball = Sphere::new(Point::new(1.0, -2.0, 0.5), radius).expect("a ball");
        let surface = Surface::from(ball);
        for round in [0.0, 0.3, 1.0, 3.0, -2.5, 3.1] {
            for up in [0.0, 0.7, -1.2, core::f64::consts::FRAC_PI_2] {
                assert_the_round_trip_stayed_inside_the_bound(
                    &surface,
                    ball.point_at(round, up),
                    &format!("a ball of radius {radius} at ({round}, {up})"),
                );
            }
        }
    }
}

/// Each arm hands the question to the shape underneath it, bit for bit.
///
/// The plane's arm is `Plane::plane_coordinates_of`, which was already there
/// and keeps its name — delegation rather than a second implementation, so
/// there is one answer to the question and not two
/// (`docs/decisions.md` §25.2). The round trip on the plane is checked here
/// too, on points put onto it first: a plane throws away how far off it a point
/// was, so a point that was never on it does not come back.
#[test]
fn a_surface_hands_the_question_to_the_shape_underneath_it() {
    let flat = Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(1.0, 2.0, 3.0))
        .expect("a plane with a normal to point along");
    let ball = Sphere::new(Point::new(1.0, 2.0, 3.0), 5.0).expect("a ball of radius five");
    let pipe = Cylinder::new(
        Axis::new(Point::new(-4.0, 0.0, 1.0), Vector::new(1.0, 1.0, 2.0)).expect("an axis"),
        0.5,
    )
    .expect("a pipe of radius a half");

    for anywhere in [
        Point::origin(),
        Point::new(1.0, 2.0, 3.0),
        Point::new(-1e7, 4.0, 0.25),
    ] {
        assert_eq!(
            bits_of(Surface::from(flat).coordinates_of(anywhere)),
            bits_of(flat.plane_coordinates_of(anywhere))
        );
        assert_eq!(
            bits_of(Surface::from(ball).coordinates_of(anywhere)),
            bits_of(ball.coordinates_of(anywhere))
        );
        assert_eq!(
            bits_of(Surface::from(pipe).coordinates_of(anywhere)),
            bits_of(pipe.coordinates_of(anywhere))
        );

        assert_the_round_trip_stayed_inside_the_bound(
            &Surface::from(flat),
            flat.project(anywhere),
            "a point on a plane",
        );
    }
}

/// Two coordinates as the bits they are stored in, so `-0.0` and `+0.0` are not
/// called the same number. This suite claims a delegation copies an answer, and
/// that is stronger than the two being equal.
fn bits_of((u, v): (f64, f64)) -> (u64, u64) {
    (u.to_bits(), v.to_bits())
}

/// The other direction: a coordinate handed out and read back in.
///
/// The bound is a distance, so it is converted into the coordinate by dividing
/// by how far a step in it carries you *there* — the radius, going round a
/// pipe. The conversion is sound because for a turn this small the arc and the
/// chord agree to well past the digits being compared.
#[test]
fn an_angle_round_a_pipe_comes_back_as_the_angle_it_was() {
    let pipe = Cylinder::new(Axis::z(), 3.0).expect("a pipe of radius three");
    let surface = Surface::from(pipe);

    for round in [0.0, 0.3, 1.0, 2.0, 3.0, -0.4, -3.1] {
        for along in [0.0, 5.0, -400.0] {
            let point = pipe.point_at(round, along);
            let (came_back, and_along) = pipe.coordinates_of(point);
            let allowed = Surface::ROUND_TRIP_ROUNDOFF
                * the_largest_number_involved(&surface, point)
                / pipe.radius();
            assert!(
                (came_back - round).abs() <= allowed,
                "the angle {round} came back as {came_back}, past the {allowed} allowed"
            );
            assert_eq!(
                and_along, along,
                "a pipe about z reads its own height back exactly"
            );
        }
    }
}

/// The same for a ball, where going round is worth less and less as you climb.
///
/// The conversion divides by the radius of the ring the point sits on, not by
/// the ball's radius: a longitude near a pole is pinned down by almost nothing,
/// and this is where that is a number rather than a remark.
#[test]
fn a_longitude_and_a_latitude_on_a_ball_come_back_as_they_went_in() {
    let ball = Sphere::new(Point::new(10.0, 0.0, -3.0), 2.0).expect("a ball of radius two");
    let surface = Surface::from(ball);

    for round in [0.0, 0.3, 1.0, 2.0, 3.0, -0.4, -3.1] {
        for up in [0.0, 0.5, -1.0, 1.5] {
            let point = ball.point_at(round, up);
            let (round_again, up_again) = ball.coordinates_of(point);
            let reach = Surface::ROUND_TRIP_ROUNDOFF * the_largest_number_involved(&surface, point);

            let ring = ball.radius() * up.cos();
            assert!(
                (round_again - round).abs() <= reach / ring,
                "the longitude {round} at latitude {up} came back as {round_again}"
            );
            assert!(
                (up_again - up).abs() <= reach / ball.radius(),
                "the latitude {up} came back as {up_again}"
            );
        }
    }
}

/// A point exactly on a bar's seam has one answer here and not two.
///
/// `atan2(+0, positive)` is `+0`, so the seam comes back at the start of the
/// branch and never at the end of it. A hair the other way round is a small
/// *negative* angle rather than a hair under `2π`, which is the whole reason
/// `docs/decisions.md` §25.2 puts the unwrapping in `rcad-topo`: only something
/// that knows it is walking a loop can tell the two apart.
#[test]
fn a_point_on_a_bars_seam_gets_one_answer_and_not_two() {
    let pipe = Cylinder::new(Axis::z(), 3.0).expect("a pipe of radius three");

    for along in [0.0, 2.0, -7.5] {
        let on_the_seam = Point::new(3.0, 0.0, along);
        assert_eq!(pipe.coordinates_of(on_the_seam), (0.0, along));
    }

    let (a_hair_back, _) = pipe.coordinates_of(pipe.point_at(-0.001, 0.0));
    assert!(
        a_hair_back < 0.0,
        "a hair short of the seam is a small negative angle, not one just under a turn, \
         and this came back {a_hair_back}"
    );
}

/// An angle handed out past the end of the branch does not come back as
/// itself, and the round trip in 3D holds anyway.
///
/// This is the half of the contract that is a refusal: `coordinates_of` is the
/// inverse of `point_at` as a map between *points*, and only inside `(-π, π]`
/// as a map between numbers. A whole turn is the same place on a pipe, so
/// nothing has gone wrong — but a caller walking a wire round a bar and
/// expecting `2π` back is the reason §25.2 puts the unwrap in `rcad-topo`, and
/// this is that expectation being wrong on purpose.
#[test]
fn an_angle_past_the_end_of_the_branch_comes_back_as_the_one_inside_it() {
    let pipe = Cylinder::new(Axis::z(), 3.0).expect("a pipe of radius three");
    let a_whole_turn = 2.0 * core::f64::consts::PI;

    for round in [0.0, 0.6, -2.0] {
        let (came_back, _) = pipe.coordinates_of(pipe.point_at(round + a_whole_turn, 0.0));
        assert!(
            (came_back - round).abs() < 1e-12,
            "a turn further round is the same place, and {} came back as {came_back}",
            round + a_whole_turn
        );
        assert_the_round_trip_stayed_inside_the_bound(
            &Surface::from(pipe),
            pipe.point_at(round + a_whole_turn, 0.0),
            "a point named by an angle past the end of the branch",
        );
    }
}

/// The far side of the seam is `π`, and `-π` only for a coordinate that arrives
/// as negative zero.
///
/// §25.2 calls the branch `(-π, π]`. `atan2`'s is closed at both ends, and
/// `-0.0` is the one input that reaches the open one — a real difference and a
/// small one, because a caller has to hand over a negative zero to see it.
///
/// It is written down here rather than papered over. Forcing `-π` up to `+π`
/// would mean comparing a coordinate against zero, which is the kind of test
/// this kernel does not make, and it would buy nothing: the two answers name
/// the same point, and the only thing downstream that can tell them apart is
/// §25.2's unwrap, which sees a jump of a whole turn and takes it out either
/// way.
#[test]
fn a_point_arriving_with_a_negative_zero_lands_on_the_far_end_of_the_branch() {
    let ball = Sphere::new(Point::origin(), 1.0).expect("a ball of radius one");

    let (from_above, _) = ball.coordinates_of(Point::new(-1.0, 0.0, 0.0));
    assert_eq!(from_above, core::f64::consts::PI);

    let (from_below, _) = ball.coordinates_of(Point::new(-1.0, -0.0, 0.0));
    assert_eq!(from_below, -core::f64::consts::PI);
}

/// At a pole no longitude is more right than any other, and zero is the one
/// that comes back.
///
/// Right in the only sense available: putting it back through `point_at` lands
/// on the pole, and so does every other longitude. The latitude is `±π/2`
/// exactly, which is what lets a caller *tell* it is at a pole.
///
/// `cos(π/2)` is `6.1e-17` rather than zero — the note on `Sphere::point_at`
/// says so — so a pole named through `point_at` sits on a ring about
/// `0.28 · f64::EPSILON · radius` across, and two longitudes are at most twice
/// that apart. Four is that with room for the roundings on the way.
#[test]
fn a_balls_pole_has_no_longitude_and_zero_is_as_right_as_anything_else() {
    let ball = Sphere::new(Point::new(2.0, -1.0, 4.0), 3.0).expect("a ball of radius three");

    for (pole, up) in [
        (Point::new(2.0, -1.0, 7.0), core::f64::consts::FRAC_PI_2),
        (Point::new(2.0, -1.0, 1.0), -core::f64::consts::FRAC_PI_2),
    ] {
        assert_eq!(ball.coordinates_of(pole), (0.0, up));

        let allowed = 4.0 * f64::EPSILON * ball.radius();
        for any_longitude in [0.0, 1.0, -2.5, 3.0] {
            assert!(
                pole.distance_to(ball.point_at(any_longitude, up)) <= allowed,
                "longitude {any_longitude} names the same pole as zero does"
            );
        }
    }
}

/// A point off the surface is answered about the point on the surface nearest
/// it, which is `docs/decisions.md` §25.2's one rule rather than two.
///
/// Straight out from the centre of a ball and straight out from the axis of a
/// pipe: every point along that ray has the same two coordinates, whether it is
/// half a radius out or a thousand. The multipliers are powers of two so that
/// stepping along the ray moves an exponent and nothing else, and the claim can
/// be equality rather than nearness.
#[test]
fn a_point_off_the_surface_is_answered_about_the_nearest_point_on_it() {
    let ball = Sphere::new(Point::origin(), 2.0).expect("a ball of radius two");
    let on_the_skin = ball.point_at(0.7, -0.3);
    for how_far in [0.5, 1.0, 4.0, 1024.0] {
        let further_out = Point::origin() + (on_the_skin - Point::origin()) * how_far;
        assert_eq!(
            ball.coordinates_of(further_out),
            ball.coordinates_of(on_the_skin),
            "{how_far} of the way out is the same direction from the centre"
        );
    }

    let pipe = Cylinder::new(Axis::z(), 2.0).expect("a pipe of radius two");
    let far_out = pipe.coordinates_of(Point::new(6.0, 6.0, 5.0));
    let well_inside = pipe.coordinates_of(Point::new(0.25, 0.25, 5.0));
    assert_eq!(far_out, well_inside);
    assert_eq!(
        far_out.1, 5.0,
        "how far along the axis is kept, not thrown away with the distance"
    );

    let flat = Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 0.0, 1.0))
        .expect("a plane facing up");
    let above = Point::new(3.0, -1.0, 900.0);
    assert_eq!(
        flat.plane_coordinates_of(above),
        flat.plane_coordinates_of(flat.project(above)),
        "a plane throws away how far off it a point was, and always did"
    );
}

/// A pipe's axis has no nearest point on the skin. Its angle is not determined
/// by anything and comes back zero; how far along it is *is* determined, and is
/// not thrown away with it.
///
/// §25.2 says the answer where there is no nearest point is `(0, 0)`, and gives
/// the reason: no branch, just what the arithmetic hands over. Follow the
/// reason and a pipe keeps its second coordinate, because zeroing it would need
/// exactly the branch the sentence rules out — a test of two `f64` against
/// zero — and would throw away a number that is perfectly well determined. The
/// centre of a ball, where neither coordinate means anything, really is
/// `(0, 0)`.
#[test]
fn a_point_on_a_pipes_axis_gives_up_its_angle_and_keeps_how_far_along_it_is() {
    let pipe = Cylinder::new(Axis::z(), 2.0).expect("a pipe of radius two");
    assert_eq!(pipe.coordinates_of(Point::origin()), (0.0, 0.0));
    assert_eq!(pipe.coordinates_of(Point::new(0.0, 0.0, 9.0)), (0.0, 9.0));

    let ball = Sphere::new(Point::new(3.0, 4.0, 5.0), 2.0).expect("a ball of radius two");
    assert_eq!(ball.coordinates_of(ball.centre()), (0.0, 0.0));
}

/// The bound holds out to `2^60` from the origin, which is the point of stating
/// it against the largest number involved rather than against the radius.
///
/// `docs/decisions.md` §19 is the same claim about reading a plane off a model
/// that has been moved. A pipe of radius one at `2^60` reads its angle out of
/// terms the size of `2^60`, and what survives that is a fixed fraction of
/// `2^60` — not of the radius.
#[test]
fn a_surface_a_long_way_from_the_origin_holds_the_same_bound() {
    for exponent in [0, 10, 30, 60, -30] {
        let far = 2f64.powi(exponent);
        let shift = Vector::new(far, -far, far * 0.5);

        let pipe = Cylinder::new(
            Axis::new(Point::origin() + shift, Vector::new(0.0, 0.0, 1.0)).expect("an axis"),
            1.0,
        )
        .expect("a pipe of radius one, a long way out");
        for round in [0.0, 1.0, -2.5, 3.1] {
            assert_the_round_trip_stayed_inside_the_bound(
                &Surface::from(pipe),
                pipe.point_at(round, far),
                &format!("a pipe at 2^{exponent}, at angle {round}"),
            );
        }

        let ball = Sphere::new(Point::origin() + shift, 1.0).expect("a ball a long way out");
        for round in [0.0, 1.0, -2.5, 3.1] {
            for up in [0.0, 1.0, -1.4] {
                assert_the_round_trip_stayed_inside_the_bound(
                    &Surface::from(ball),
                    ball.point_at(round, up),
                    &format!("a ball at 2^{exponent}, at ({round}, {up})"),
                );
            }
        }
    }
}

/// And across `2^-60` to `2^60` of size — thirty-six orders of magnitude —
/// which is what `docs/decisions.md` §18 asks of anything stated as a multiple
/// of a scale.
#[test]
fn a_surface_of_any_size_holds_the_same_bound() {
    for exponent in [-60, -20, 0, 20, 60] {
        let radius = 2f64.powi(exponent);

        let pipe = Cylinder::new(Axis::z(), radius).expect("a pipe");
        let ball = Sphere::new(Point::origin(), radius).expect("a ball");
        for round in [0.0, 1.0, -2.5, 3.1] {
            assert_the_round_trip_stayed_inside_the_bound(
                &Surface::from(pipe),
                pipe.point_at(round, radius * 3.0),
                &format!("a pipe of radius 2^{exponent}"),
            );
            assert_the_round_trip_stayed_inside_the_bound(
                &Surface::from(ball),
                ball.point_at(round, 0.9),
                &format!("a ball of radius 2^{exponent}"),
            );
        }
    }
}

/// A surface reaching this function has already had its location composed
/// (`docs/decisions.md` §24.8), so turned and shifted is the ordinary case and
/// axis-aligned is the special one.
///
/// A turned pipe is the case with something to go wrong: its two directions
/// across come out of a `normalized`, so the angle is read off a frame that is
/// only at right angles to itself to within a rounding.
#[test]
fn a_turned_and_shifted_pipe_puts_its_points_back_where_it_found_them() {
    let turn = Isometry::rotation(
        &Axis::new(Point::new(1.0, 1.0, 1.0), Vector::new(1.0, 2.0, -0.5)).expect("an axis"),
        0.9,
    )
    .expect("a turn about a real axis");
    let shift = Isometry::translation(Vector::new(-30.0, 12.0, 4.0)).expect("a finite shift");
    let movement = shift.compose(&turn);

    let pipe = Cylinder::new(Axis::z(), 2.0)
        .expect("a pipe of radius two")
        .transformed(&movement);
    let surface = Surface::from(pipe);
    for round in [0.0, 0.9, 2.2, -1.7, 3.1] {
        for along in [0.0, 6.0, -11.0] {
            assert_the_round_trip_stayed_inside_the_bound(
                &surface,
                pipe.point_at(round, along),
                &format!("a turned pipe at ({round}, {along})"),
            );
        }
    }
}

/// Turning a ball does not turn its coordinates, and this is what that means
/// for the way back.
///
/// A ball is the same ball however it is turned, so `Sphere::point_at` takes
/// its frame from the world and `coordinates_of` reads it back out of the
/// world. A turned ball therefore answers about the *unturned* longitude of a
/// point, and a caller expecting the turn to have carried the parameterization
/// with it would be wrong in a way no round-trip test can see. So it is said
/// here directly.
#[test]
fn turning_a_ball_does_not_turn_which_longitude_a_point_is_at() {
    let ball = Sphere::new(Point::origin(), 3.0).expect("a ball of radius three");
    let turn = Isometry::rotation(&Axis::z(), 1.0).expect("a turn about z");
    let turned = ball.transformed(&turn);

    assert_eq!(
        turned, ball,
        "turning a ball about its centre changes nothing"
    );

    let point = Point::new(3.0, 0.0, 0.0);
    assert_eq!(turned.coordinates_of(point), ball.coordinates_of(point));
    assert_the_round_trip_stayed_inside_the_bound(
        &Surface::from(turned),
        point,
        "a point on a turned ball",
    );
}

/// The measurement behind the constant, printed so the room it leaves is a
/// figure and not a feeling.
///
/// It sweeps the ratio the derivation is really about: a surface placed at
/// `2^placed` whose radius is `2^wide`, so the angle is read out of terms up to
/// `2^40` times bigger than the radius they have to resolve. Run it with
/// `cargo test --release -- --nocapture` to read the number.
#[test]
fn the_worst_round_trip_this_suite_can_find_is_well_inside_the_bound() {
    let turn = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(1.0, 2.0, -0.5)).expect("an axis"),
        0.9,
    )
    .expect("a turn about a real axis");

    let mut worst = 0.0f64;
    for placed in [-40, -10, 0, 10, 40] {
        for wide in [placed - 40, placed - 20, placed, placed + 20] {
            let at = Point::new(2f64.powi(placed), -2f64.powi(placed), 2f64.powi(placed));
            let radius = 2f64.powi(wide);

            let pipe = Cylinder::new(
                Axis::new(at, Vector::new(0.0, 0.0, 1.0)).expect("an axis"),
                radius,
            )
            .expect("a pipe")
            .transformed(&turn);
            let ball = Sphere::new(at, radius).expect("a ball");

            for step in 0..400 {
                let fraction = f64::from(step) / 400.0;
                let round = core::f64::consts::PI * (2.0 * fraction - 1.0);
                let up = 3.0 * fraction - 1.5;
                worst = worst.max(how_much_of_the_bound_a_round_trip_used(
                    &Surface::from(pipe),
                    pipe.point_at(round, radius * 100.0),
                ));
                worst = worst.max(how_much_of_the_bound_a_round_trip_used(
                    &Surface::from(ball),
                    ball.point_at(round, up),
                ));
            }
        }
    }

    println!("the worst round trip found used {worst} of the bound");
    assert!(worst <= 1.0, "the bound was broken, by a factor of {worst}");
    assert!(
        worst > 0.0,
        "a suite that never moved a point at all is not measuring anything"
    );
}

/// A ball of any size, anywhere, turned any way — which for a ball is only
/// anywhere, and `turning_a_ball_does_not_turn_which_longitude_a_point_is_at`
/// says why.
fn a_ball_anywhere() -> impl Strategy<Value = Sphere> {
    (a_size(-6..6), a_place()).prop_map(|(radius, centre)| {
        Sphere::new(centre, radius).expect("a positive radius and a finite centre")
    })
}

/// A pipe of any size, anywhere, about any direction.
///
/// The direction is built from two angles rather than drawn as three numbers,
/// so it always has a length to point along and `Axis::new` never has anything
/// to refuse — valid by construction, which is what `support`'s families do and
/// for the same reason.
fn a_pipe_anywhere() -> impl Strategy<Value = Cylinder> {
    (a_size(-6..6), a_place(), -3.2f64..3.2, -1.5f64..1.5).prop_map(
        |(radius, origin, round, up)| {
            let direction = Vector::new(up.cos() * round.cos(), up.cos() * round.sin(), up.sin());
            let axis = Axis::new(origin, direction).expect("two angles always name a direction");
            Cylinder::new(axis, radius).expect("a positive radius and a finite axis")
        },
    )
}

/// Somewhere from the origin out to a thousand kilometres, in millimetres.
fn a_place() -> impl Strategy<Value = Point> {
    proptest::collection::vec(a_coordinate(), 3).prop_map(|at| Point::new(at[0], at[1], at[2]))
}

fn a_coordinate() -> impl Strategy<Value = f64> {
    prop_oneof![
        Just(0.0),
        (a_size(-3..9), any::<bool>()).prop_map(|(size, below)| if below { -size } else { size }),
    ]
}

/// A number between `10^exponents.start` and `10^exponents.end`, every order of
/// magnitude in between getting equal billing.
fn a_size(exponents: core::ops::Range<i32>) -> impl Strategy<Value = f64> {
    (exponents, 1.0f64..10.0).prop_map(|(exponent, mantissa)| mantissa * 10f64.powi(exponent))
}

proptest! {
    #[test]
    fn every_point_on_a_ball_comes_back_from_its_two_coordinates(
        ball in a_ball_anywhere(),
        round in -3.2f64..3.2,
        up in -1.6f64..1.6,
    ) {
        let surface = Surface::from(ball);
        let used = how_much_of_the_bound_a_round_trip_used(&surface, ball.point_at(round, up));
        prop_assert!(used <= 1.0, "used {} of the bound", used);
    }

    #[test]
    fn every_point_on_a_pipe_comes_back_from_its_two_coordinates(
        pipe in a_pipe_anywhere(),
        round in -3.2f64..3.2,
        along in -1e6f64..1e6,
    ) {
        let surface = Surface::from(pipe);
        let used = how_much_of_the_bound_a_round_trip_used(&surface, pipe.point_at(round, along));
        prop_assert!(used <= 1.0, "used {} of the bound", used);
    }

    /// The same claim about points that never went through the trigonometry.
    ///
    /// `support::the_octant_of_the_unit_sphere` is the octant of the unit ball
    /// as an exact rational patch, so de Boor puts a point on the same skin
    /// `Sphere` describes, by a route that shares nothing with `point_at`.
    /// Scaling by a power of two only moves an exponent, so the scaled point is
    /// exactly on the scaled ball.
    #[test]
    fn a_point_the_splines_put_on_a_ball_comes_back_too(
        u in 0.0f64..1.0,
        v in 0.0f64..1.0,
        exponent in -40i32..40,
    ) {
        let scale = 2f64.powi(exponent);
        let on_the_unit_ball = support::the_octant_of_the_unit_sphere()
            .evaluate(u, v)
            .expect("a parameter from the patch's own domain");
        let point = Point::new(
            on_the_unit_ball.x() * scale,
            on_the_unit_ball.y() * scale,
            on_the_unit_ball.z() * scale,
        );

        let ball = Sphere::new(Point::origin(), scale).expect("a positive radius");
        let used = how_much_of_the_bound_a_round_trip_used(&Surface::from(ball), point);
        prop_assert!(used <= 1.0, "used {} of the bound", used);
    }

    /// And on a pipe, from the quarter circle pulled a unit along `z`.
    #[test]
    fn a_point_the_splines_put_on_a_pipe_comes_back_too(
        u in 0.0f64..1.0,
        v in 0.0f64..1.0,
        exponent in -40i32..40,
    ) {
        let scale = 2f64.powi(exponent);
        let on_the_unit_pipe = support::the_quarter_circle_extruded_one_unit_in_z()
            .evaluate(u, v)
            .expect("a parameter from the patch's own domain");
        let point = Point::new(
            on_the_unit_pipe.x() * scale,
            on_the_unit_pipe.y() * scale,
            on_the_unit_pipe.z() * scale,
        );

        let pipe = Cylinder::new(Axis::z(), scale).expect("a positive radius");
        let used = how_much_of_the_bound_a_round_trip_used(&Surface::from(pipe), point);
        prop_assert!(used <= 1.0, "used {} of the bound", used);
    }
}
