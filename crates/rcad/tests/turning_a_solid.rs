//! Volume, area and centroid under a rigid movement
//! (`docs/decisions.md` §9.1, roadmap §11.2).
//!
//! Kernels overfit badly to shapes that sit square-on to the axes and centred
//! on the origin. An arbitrary rotation finds that straight away and needs no
//! oracle at all: whatever the right answer is, turning the part cannot change
//! it. Roadmap §11.2 says start here, and it costs nothing.
//!
//! # How tight the comparisons are, and why
//!
//! An exact `==` cannot hold through a general rotation and asserting one would
//! only be testing `nalgebra`'s quaternion arithmetic. Two different bounds are
//! used, and both are worked out rather than guessed.
//!
//! **[`SLACK_AFTER_A_GENERAL_TURN`]** covers an arbitrary axis and angle.
//! Rotating a coordinate is a three-term dot product, so every coordinate comes
//! back with a few roundings on it. Volume is degree three in the coordinates,
//! so each term of the divergence-theorem sum carries roughly nine of those.
//! The sum itself cancels heavily — the tetrahedra reach out to the origin —
//! which multiplies the whole thing by the triangle count, fewer than a hundred
//! here, so call it two hundred. `200 * 9 * f64::EPSILON` is about `4e-13`, and
//! the constant is `1e-12`. The worst seen over several thousand runs is
//! `1.6e-13`.
//!
//! **[`SLACK_AFTER_A_QUARTER_TURN`]** covers a multiple of a quarter turn about
//! a coordinate axis, which is the case that costs nothing and catches the
//! most. Such a rotation only permutes coordinates and flips signs, both of
//! which are exact — except that the matrix is built from a quaternion, so
//! `cos(pi/2)` arrives as `6.1e-17` rather than zero and the entries land
//! within a rounding or two of `0` and `±1`. So the result *should* be very
//! nearly bit-exact, and `64 * f64::EPSILON` says how nearly. The worst seen is
//! `1.1e-15`; a real mistake in the location composition would be nowhere near
//! this small.

mod shapes;

use core::f64::consts::FRAC_PI_2;

use proptest::prelude::*;
use rcad::{Axis, Isometry, Model, Point, Policy, ProvenanceLog, Scale, SolidKey};
use shapes::{a_notched_polygon_with_holes, an_arbitrary_movement, extruded};

/// Room for two hundred roundings of a quantity that is cubic in the
/// coordinates. See the module docs.
const SLACK_AFTER_A_GENERAL_TURN: f64 = 1e-12;

/// Room for a rounding or two, and nothing more. See the module docs.
const SLACK_AFTER_A_QUARTER_TURN: f64 = 64.0 * f64::EPSILON;

#[test]
fn a_cube_turned_a_quarter_of_the_way_round_the_z_axis_is_still_the_same_cube() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let cube = model
        .add_cube(10.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side ten");
    let before = model
        .solid(cube)
        .measure()
        .expect("a solid this size measures");
    assert_eq!(before.volume(), 1000.0);

    let quarter = Isometry::rotation(&Axis::z(), FRAC_PI_2).expect("a quarter turn");
    model.place_solid(cube, &quarter);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("turning a cube leaves it a cube");
    let after = model
        .solid(cube)
        .measure()
        .expect("a solid this size measures");

    assert_eq!(after.volume(), before.volume());
    assert_eq!(after.surface_area(), before.surface_area());
    assert!(
        after
            .centroid()
            .distance_to(quarter.apply_to_point(before.centroid()))
            <= SLACK_AFTER_A_QUARTER_TURN * 10.0
    );
}

proptest! {
    #[test]
    fn turning_a_solid_any_which_way_leaves_its_volume_and_its_area_alone(
        profile in a_notched_polygon_with_holes(),
        height in 0.5f64..=6.0,
        movement in an_arbitrary_movement(),
    ) {
        let (mut model, solid) = extruded(&profile, height);
        let before = model.solid(solid).measure().expect("a solid this size measures");

        model.place_solid(solid, &movement);
        model.validate(&Policy::certified(), &mut ProvenanceLog::new())?;
        let after = model.solid(solid).measure().expect("a solid this size measures");

        prop_assert!(
            drift(after.volume(), before.volume()) <= SLACK_AFTER_A_GENERAL_TURN,
            "volume went from {} to {}",
            before.volume(),
            after.volume()
        );
        prop_assert!(
            drift(after.surface_area(), before.surface_area()) <= SLACK_AFTER_A_GENERAL_TURN,
            "surface area went from {} to {}",
            before.surface_area(),
            after.surface_area()
        );
    }

    #[test]
    fn the_centroid_of_a_turned_solid_lands_where_the_movement_says_it_should(
        profile in a_notched_polygon_with_holes(),
        height in 0.5f64..=6.0,
        movement in an_arbitrary_movement(),
    ) {
        let (mut model, solid) = extruded(&profile, height);
        let before = model.solid(solid).measure().expect("a solid this size measures").centroid();

        model.place_solid(solid, &movement);
        let after = model.solid(solid).measure().expect("a solid this size measures").centroid();

        // The centroid is a moment divided by the volume, so it carries the
        // volume's relative error, and it is a position rather than a ratio —
        // so the bound has to be scaled by how far from the origin the solid
        // has been put.
        let allowed = SLACK_AFTER_A_GENERAL_TURN * reach_of(&model, solid);
        let expected = movement.apply_to_point(before);
        prop_assert!(
            after.distance_to(expected) <= allowed,
            "centroid landed at {after:?}, expected {expected:?}"
        );
    }

    #[test]
    fn a_solid_turned_a_whole_number_of_quarter_turns_about_an_axis_barely_moves_at_all(
        profile in a_notched_polygon_with_holes(),
        height in 0.5f64..=6.0,
        which_axis in 0usize..3,
        quarters in 1u32..4,
    ) {
        let axis = [Axis::x(), Axis::y(), Axis::z()][which_axis];
        let turn = Isometry::rotation(&axis, FRAC_PI_2 * f64::from(quarters))
            .expect("a whole number of quarter turns");

        let (mut model, solid) = extruded(&profile, height);
        let before = model.solid(solid).measure().expect("a solid this size measures");

        model.place_solid(solid, &turn);
        model.validate(&Policy::certified(), &mut ProvenanceLog::new())?;
        let after = model.solid(solid).measure().expect("a solid this size measures");

        prop_assert!(
            drift(after.volume(), before.volume()) <= SLACK_AFTER_A_QUARTER_TURN,
            "volume went from {} to {}",
            before.volume(),
            after.volume()
        );
        prop_assert!(
            drift(after.surface_area(), before.surface_area()) <= SLACK_AFTER_A_QUARTER_TURN,
            "surface area went from {} to {}",
            before.surface_area(),
            after.surface_area()
        );
        prop_assert!(
            after
                .centroid()
                .distance_to(turn.apply_to_point(before.centroid()))
                <= SLACK_AFTER_A_QUARTER_TURN * reach_of(&model, solid),
            "centroid went from {:?} to {:?}",
            before.centroid(),
            after.centroid()
        );
    }
}

/// How far apart two numbers are, as a fraction of how big they are.
fn drift(measured: f64, expected: f64) -> f64 {
    (measured - expected).abs() / expected.abs()
}

/// How far from the origin the solid reaches, which is what an absolute
/// position tolerance has to be measured against.
///
/// # Panics
///
/// If the solid has no faces. Every solid here is an extrusion, so it has.
fn reach_of(model: &Model, solid: SolidKey) -> f64 {
    let (low, high) = model
        .solid(solid)
        .bounding_box()
        .expect("an extrusion has faces");
    largest_coordinate(low).max(largest_coordinate(high))
}

fn largest_coordinate(point: Point) -> f64 {
    point
        .coordinates()
        .iter()
        .fold(0.0f64, |worst, value| worst.max(value.abs()))
}
