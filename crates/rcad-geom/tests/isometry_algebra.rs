//! The rules a rigid movement has to obey.
//!
//! `docs/decisions.md` §3.2 puts real location composition in from day one, on
//! the grounds that every accessor written without it has to be found and fixed
//! later. These properties are what make that safe: if composition, inverse and
//! application do not agree with each other, the composed locations topology
//! hands out are quietly wrong everywhere at once.

use core::f64::consts::TAU;

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

fn a_plane() -> impl Strategy<Value = Plane> {
    (point(), coordinate(), coordinate(), coordinate())
        .prop_filter_map("the normal needs some length", |(through, x, y, z)| {
            Plane::from_point_and_normal(through, Vector::new(x, y, z)).ok()
        })
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

    /// Two faces that touch stay touching, exactly, when **one** movement moves
    /// both of them.
    ///
    /// The other half of
    /// `a_contact_between_two_solids_at_two_placements_does_not_survive_every_turn`,
    /// and what makes that one a gap rather than a limit: the answer exists and
    /// one arrangement reaches it every time. Two faces of a contact are one
    /// plane and its [`Plane::reversed`] — `(n, d)` and `(−n, −d)` — and
    /// [`Plane::transformed`] carries that through whatever the movement is,
    /// because it is a pure function of the two and because negating a vector
    /// negates each product and each sum of an `f64` dot product exactly.
    ///
    /// So a pair moved **as a unit** obeys `docs/decisions.md` §19 bit for bit.
    /// What breaks it is the two solids carrying different placements, which is
    /// what `add_extrusion` leaves behind for every solid built anywhere but the
    /// origin. §41.13.
    #[test]
    fn two_faces_that_touch_stay_touching_when_one_movement_moves_both(
        movement in movement(),
        plane in a_plane(),
    ) {
        let facing_it = plane.transformed(&movement);
        let facing_back = plane.reversed().transformed(&movement);
        prop_assert_eq!(facing_it.coefficients(), facing_back.reversed().coefficients());
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

/// Two solids at two placements, turned by one turn, stop touching.
///
/// This is the arithmetic under `rcad-topo`'s
/// `a_pair_that_touches_face_to_face_can_stop_touching_when_the_pair_is_turned`,
/// measured over every turn that suite can draw, and `docs/decisions.md` §41.13
/// is what it is recorded for.
///
/// The two planes are a solid's downward face on `z = 0` and another solid's
/// upward face on the very same `z = 0`, each written the way `add_extrusion`
/// writes it: the geometry in the solid's own frame and the lift kept as a
/// placement. Turning both by one turn is `place_solid`, which **composes**, so
/// the second one's placement becomes `(R, fl(R·(0,0,−3)))` — and
/// `fl(R·(0,0,−3))` is not `−3·fl(R·(0,0,1))`, because the quaternion sandwich
/// is not exactly homogeneous in the vector it turns.
/// [`Plane::transformed`] then works the moved offset out as `d + n·t`, whose
/// two terms are both about 3 and nearly cancel, and what is left over is the
/// gap between two faces that were touching.
///
/// **The gap is real and not a summation artefact**: summed exactly rather than
/// in `f64` it is `−5.195804564346705e-16` for the turn this pins, so the
/// composed placement genuinely puts the second solid that far off. It is
/// `0.78·f64::EPSILON·L` with `L = 3`, well inside
/// [`Isometry::POINT_ROUNDOFF`], which is why nothing downstream is entitled to
/// call it a mistake — and why a fuse, which reads the four numbers as exact,
/// answers that the two solids do not touch.
///
/// **Both halves are asserted, and the second is what stops this going quiet.**
/// A turn that keeps every contact would make the count meaningless, so the
/// test insists that a real share of the turns keep it too.
#[test]
fn a_contact_between_two_solids_at_two_placements_does_not_survive_every_turn() {
    let mut kept = 0;
    let mut lost = 0;
    for direction in every_whole_number_direction() {
        for steps in 1..64 {
            let axis = Axis::new(Point::origin(), direction).expect("a direction with length");
            let turn = Isometry::rotation(&axis, TAU * f64::from(steps) / 64.0)
                .expect("a finite rotation");
            if a_touching_pair_still_touches(&turn) {
                kept += 1;
            } else {
                lost += 1;
            }
        }
    }

    println!(
        "of {} turns, {kept} keep the contact and {lost} lose it",
        kept + lost
    );
    assert_eq!(
        (kept, lost),
        (10446, 11100),
        "the two moved planes agree for this many of the turns the fuse suites draw"
    );
}

/// The turn this suite's `rcad-topo` sibling pins, and the one the shrinker
/// found.
#[test]
fn the_turn_that_lost_a_contact_lost_it_by_less_than_one_rounding_of_the_placement() {
    let axis = Axis::new(Point::origin(), Vector::new(2.0, 3.0, -2.0)).expect("a direction");
    let turn = Isometry::rotation(&axis, TAU * 44.0 / 64.0).expect("a finite rotation");

    let (below, above) = a_touching_pair_after(&turn);
    assert_eq!(
        below.normal().components().map(|component| -component),
        above.normal().components(),
        "the two normals are still exactly opposite; only the offsets moved"
    );

    let gap = (below.offset() + above.offset()).abs();
    assert_ne!(gap, 0.0, "this is the turn that loses the contact");
    assert!(
        gap < Isometry::POINT_ROUNDOFF * 3.0,
        "a gap of {gap} is further than placing the solid is allowed to move it"
    );
}

/// The two faces of the contact, each read the way a fuse reads it.
fn a_touching_pair_after(turn: &Isometry) -> (Plane, Plane) {
    let below = Plane::from_coefficients([0.0, 0.0, -1.0, -0.0]).expect("a plane");
    let above = Plane::from_coefficients([0.0, 0.0, 1.0, -3.0]).expect("a plane");
    let lift = Isometry::translation(Vector::new(0.0, 0.0, -3.0)).expect("a finite shift");
    (
        below.transformed(turn),
        above.transformed(&turn.compose(&lift)),
    )
}

fn a_touching_pair_still_touches(turn: &Isometry) -> bool {
    let (below, above) = a_touching_pair_after(turn);
    let (mine, theirs) = (below.coefficients(), above.coefficients());
    (0..3).all(|axis| mine[axis] == -theirs[axis]) && mine[3] + theirs[3] == 0.0
}

/// Every direction `pairs::a_turn` can draw.
fn every_whole_number_direction() -> impl Iterator<Item = Vector> {
    (-3..=3).flat_map(|x| {
        (-3..=3).flat_map(move |y| {
            (-3..=3)
                .filter(move |&z: &i32| [x, y, z] != [0, 0, 0])
                .map(move |z| Vector::new(f64::from(x), f64::from(y), f64::from(z)))
        })
    })
}
