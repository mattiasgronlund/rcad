//! The rules a rigid movement has to obey.
//!
//! `docs/decisions.md` §3.2 puts real location composition in from day one, on
//! the grounds that every accessor written without it has to be found and fixed
//! later. These properties are what make that safe: if composition, inverse and
//! application do not agree with each other, the composed locations topology
//! hands out are quietly wrong everywhere at once.

use proptest::prelude::*;
use rcad_geom::{Axis, Isometry, Point, Vector};

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
