//! The rules a rigid movement has to obey.
//!
//! `docs/decisions.md` §3.2 puts real location composition in from day one, on
//! the grounds that every accessor written without it has to be found and fixed
//! later. These properties are what make that safe: if composition, inverse and
//! application do not agree with each other, the composed locations topology
//! hands out are quietly wrong everywhere at once.

use proptest::prelude::*;
use rcad_geom::{Axis, Isometry, Plane, Point, Vector};

/// Coordinates stay inside a 1000-unit box, so an absolute tolerance of `1e-9`
/// is about six digits looser than the rounding on the numbers themselves.
const CLOSE_ENOUGH: f64 = 1e-9;

fn coordinate() -> impl Strategy<Value = f64> {
    -1000.0f64..1000.0f64
}

fn point() -> impl Strategy<Value = Point> {
    (coordinate(), coordinate(), coordinate()).prop_map(|(x, y, z)| Point::new(x, y, z))
}

fn movement() -> impl Strategy<Value = Isometry> {
    (
        point(),
        coordinate(),
        coordinate(),
        coordinate(),
        -8.0f64..8.0f64,
        coordinate(),
        coordinate(),
        coordinate(),
    )
        .prop_filter_map(
            "the axis direction needs some length",
            |(on_axis, dx, dy, dz, angle, tx, ty, tz)| {
                let axis = Axis::new(on_axis, Vector::new(dx, dy, dz)).ok()?;
                let turn = Isometry::rotation(&axis, angle).ok()?;
                let shift = Isometry::translation(Vector::new(tx, ty, tz)).ok()?;
                Some(shift.compose(&turn))
            },
        )
}

proptest! {
    #[test]
    fn moving_a_point_and_then_moving_it_back_leaves_it_where_it_started(
        movement in movement(),
        point in point(),
    ) {
        let there_and_back = movement.inverse().apply_to_point(movement.apply_to_point(point));
        prop_assert!(there_and_back.distance_to(point) < CLOSE_ENOUGH);
    }

    #[test]
    fn it_does_not_matter_which_two_of_three_movements_you_combine_first(
        first in movement(),
        second in movement(),
        third in movement(),
        point in point(),
    ) {
        let left = third.compose(&second).compose(&first);
        let right = third.compose(&second.compose(&first));
        prop_assert!(
            left.apply_to_point(point).distance_to(right.apply_to_point(point)) < CLOSE_ENOUGH
        );
    }

    #[test]
    fn combining_with_the_movement_that_does_nothing_changes_nothing(
        movement in movement(),
        point in point(),
    ) {
        let identity = Isometry::identity();
        let moved = movement.apply_to_point(point);
        prop_assert!(
            movement.compose(&identity).apply_to_point(point).distance_to(moved) < CLOSE_ENOUGH
        );
        prop_assert!(
            identity.compose(&movement).apply_to_point(point).distance_to(moved) < CLOSE_ENOUGH
        );
    }

    #[test]
    fn a_rigid_movement_does_not_change_the_distance_between_two_points(
        movement in movement(),
        from in point(),
        to in point(),
    ) {
        let before = from.distance_to(to);
        let after = movement.apply_to_point(from).distance_to(movement.apply_to_point(to));
        prop_assert!((after - before).abs() < CLOSE_ENOUGH);
    }

    #[test]
    fn combining_two_movements_is_the_same_as_doing_them_one_after_the_other(
        outer in movement(),
        inner in movement(),
        point in point(),
    ) {
        let combined = outer.compose(&inner).apply_to_point(point);
        let one_at_a_time = outer.apply_to_point(inner.apply_to_point(point));
        prop_assert!(combined.distance_to(one_at_a_time) < CLOSE_ENOUGH);
    }

    #[test]
    fn moving_a_step_between_two_points_is_the_same_as_moving_both_points(
        movement in movement(),
        from in point(),
        to in point(),
    ) {
        let moved_step = movement.apply_to_vector(to - from);
        let step_between_moved =
            movement.apply_to_point(to) - movement.apply_to_point(from);
        prop_assert!((moved_step - step_between_moved).length() < CLOSE_ENOUGH);
    }

    #[test]
    fn undoing_a_movement_twice_gets_the_movement_back(
        movement in movement(),
        point in point(),
    ) {
        let twice_undone = movement.inverse().inverse();
        prop_assert!(
            twice_undone.apply_to_point(point).distance_to(movement.apply_to_point(point))
                < CLOSE_ENOUGH
        );
    }

    #[test]
    fn a_snapped_translation_stays_within_half_a_grid_step_of_what_was_asked_for(
        x in coordinate(),
        y in coordinate(),
        z in coordinate(),
        grid_exponent in -20i32..10i32,
    ) {
        let asked_for = Vector::new(x, y, z);
        let snapped = Isometry::snapped_translation(asked_for, grid_exponent)
            .unwrap()
            .translation_part();
        let half_a_step = f64::from(grid_exponent).exp2() / 2.0;
        for (before, after) in asked_for.components().iter().zip(snapped.components()) {
            prop_assert!((after - before).abs() <= half_a_step);
        }
    }
}

/// Moving a plane by the movement that does nothing leaves the plane alone,
/// however many times it is done.
///
/// It did not until [`Plane::transformed`] stopped rebuilding the offset as
/// `normal · movement(origin())`. `origin()` is `normal * offset`, so that round
/// trip computed `offset * (nx² + ny² + nz²)`, and for a normal only as close to
/// unit length as `f64` allows that squared length is not exactly one. The plane
/// below used to walk from `-1.0000000000000002` to `-1.0000000000000047` over
/// twenty reads, monotonically, with nothing to stop it.
///
/// **Why twenty reads and not one.** `FaceView::plane` in `rcad-topo` is
/// `surface.transformed(&location)` whatever the location is, so **reading a
/// plane out of a model must be a pure read**. The fuse reads its inputs' planes
/// through exactly that accessor, and one ulp per read is one ulp per fuse in a
/// chain — the slow leak `docs/decisions.md` §17.1 rejected
/// vertex-authoritative geometry in order to avoid, arriving through the
/// accessor instead of through the geometry.
/// `rcad-topo`'s `uniting_a_turned_solid_with_copies_of_itself_never_moves_a_plane`
/// is the same statement about a chain of fuses.
#[test]
fn moving_a_plane_by_the_identity_twenty_times_leaves_every_coefficient_alone() {
    for plane in [
        a_plane_whose_normal_is_not_quite_unit_length(),
        a_plane_whose_normal_is_not_a_fixed_point_of_scaling(),
    ] {
        let mut read_back = plane;
        for read in 1..=20 {
            read_back = read_back.transformed(&Isometry::identity());
            assert_eq!(
                read_back.coefficients(),
                plane.coefficients(),
                "reading a plane back {read} times moved it"
            );
        }
    }
}

/// The same for the line down the middle of a pipe, which `Cylinder` carries and
/// reads back out through the same accessor a face's plane goes through.
#[test]
fn moving_an_axis_by_the_identity_twenty_times_leaves_its_direction_alone() {
    let axis = Axis::new(Point::new(1.0, 2.0, 3.0), Vector::new(6.0, 3.0, 7.0))
        .expect("a direction with length");

    let mut read_back = axis;
    for read in 1..=20 {
        read_back = read_back.transformed(&Isometry::identity());
        assert_eq!(
            read_back.direction().components(),
            axis.direction().components(),
            "reading an axis back {read} times turned it"
        );
    }
}

/// Sliding a plane along a direction that lies in it leaves it where it was.
///
/// The shift is along `z` and the normal has no `z` component, so the shift
/// contributes exactly zero and the answer is the plane that went in. This is
/// the identity read again with a movement that actually does something: what
/// used to move the offset was the round trip through a point on the plane, not
/// the shift.
#[test]
fn sliding_a_plane_along_a_direction_that_lies_in_it_leaves_every_coefficient_alone() {
    let plane = a_plane_whose_normal_is_not_quite_unit_length();
    let slide = Isometry::translation(Vector::new(0.0, 0.0, 5.0)).expect("a finite shift");

    assert_eq!(
        plane.transformed(&slide).coefficients(),
        plane.coefficients()
    );
}

/// A plane the round trip through a point could not leave alone.
///
/// Its normal is as close to length one as these two numbers get: the squared
/// length is `1.0000000000000002`, whose square root is exactly `1.0`, so the
/// normal survives being scaled and only the offset was ever moved.
fn a_plane_whose_normal_is_not_quite_unit_length() -> Plane {
    Plane::from_coefficients([
        -0.634_393_284_163_645_5,
        0.773_010_453_362_737_1,
        0.0,
        -1.000_000_000_000_000_2,
    ])
    .expect("four coefficients that describe a plane")
}

/// A plane that scaling the normal a second time could not leave alone.
///
/// Scaling a vector to unit length does not always give a vector whose length
/// reads back as exactly one, and a direction as plain as `(6, 3, 7)` is one
/// where it does not: the stored normal's length is `0.9999999999999999`, so
/// dividing by it again grows all three components. That is why
/// [`Plane::transformed`] does not scale a moved normal — it was scaled once,
/// where the plane was built.
fn a_plane_whose_normal_is_not_a_fixed_point_of_scaling() -> Plane {
    Plane::from_point_and_normal(Point::new(1.0, 2.0, 3.0), Vector::new(6.0, 3.0, 7.0))
        .expect("a point and a direction with length")
}
