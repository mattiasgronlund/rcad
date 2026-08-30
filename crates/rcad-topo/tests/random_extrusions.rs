//! Random extruded polygons, with holes.
//!
//! Not random convex hulls. Those are always valid and always convex, so they
//! never exercise a wire that turns back on itself or an inner wire, which is
//! exactly where the winding invariants bite (decisions.md §9.1).
//!
//! The outlines here are star-shaped about the origin with radii that jump
//! about, so they are simple by construction and almost never convex.

use proptest::prelude::*;
use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Model, Profile, ProfileBuilder};

/// A star-shaped outline: evenly spaced directions, wildly varying radii.
fn a_lumpy_outline() -> impl Strategy<Value = Vec<[f64; 2]>> {
    (3usize..12).prop_flat_map(|corners| {
        proptest::collection::vec(1u32..=40, corners).prop_map(move |radii| {
            radii
                .into_iter()
                .enumerate()
                .map(|(index, radius)| {
                    let angle = core::f64::consts::TAU * index as f64 / corners as f64;
                    let radius = f64::from(radius) / 4.0;
                    [radius * angle.cos(), radius * angle.sin()]
                })
                .collect()
        })
    })
}

/// The same, with a small square hole punched at the middle when there is room
/// for one.
///
/// # Panics
///
/// If the builder refuses one, which would be a bug in the kernel rather than
/// an input to be skipped. Every corner sits at its own angle about the origin
/// with a positive radius, so each edge stays inside a wedge narrower than half
/// a turn: adjacent edges share their corner and nothing else, and no two other
/// edges come near each other at all. The hole only goes in when the outline
/// clears the unit circle it fits inside.
///
/// Discarding a refusal here instead would hide exactly the failure this suite
/// exists to catch — a sound profile being turned away.
fn a_lumpy_profile() -> impl Strategy<Value = Profile> {
    (a_lumpy_outline(), any::<bool>()).prop_map(|(outline, wants_a_hole)| {
        let closest = outline
            .iter()
            .map(|point| point[0].hypot(point[1]))
            .fold(f64::INFINITY, f64::min);
        let mut builder = ProfileBuilder::new(outline);
        if wants_a_hole && closest > 1.0 {
            builder = builder.with_hole(vec![
                [-0.25, -0.25],
                [0.25, -0.25],
                [0.25, 0.25],
                [-0.25, 0.25],
            ]);
        }
        builder
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("every lumpy outline is a sound profile")
    })
}

fn empty_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

proptest! {
    #[test]
    fn an_extruded_lumpy_polygon_is_always_a_sound_solid(profile in a_lumpy_profile(), height in 0.1f64..10.0) {
        let mut model = empty_model();
        let solid = model.add_extrusion(&profile, height, &Isometry::identity())?;

        model.validate(&Policy::certified(), &mut ProvenanceLog::new())?;

        let measured = model.solid(solid).measure();
        prop_assert!(measured.volume() > 0.0);
        prop_assert!(measured.surface_area() > 0.0);
    }

    #[test]
    fn turning_an_extruded_lumpy_polygon_does_not_change_its_size(
        profile in a_lumpy_profile(),
        height in 0.1f64..10.0,
        angle in 0.0f64..core::f64::consts::TAU,
    ) {
        let mut model = empty_model();
        let solid = model.add_extrusion(&profile, height, &Isometry::identity())?;
        let before = model.solid(solid).measure();

        let axis = Axis::new(Point::new(0.0, 0.0, 0.0), Vector::new(1.0, 2.0, 3.0))
            .expect("a direction with length");
        model.place_solid(solid, &Isometry::rotation(&axis, angle).expect("a rotation"));
        let after = model.solid(solid).measure();

        model.validate(&Policy::certified(), &mut ProvenanceLog::new())?;
        let slack = 1e-9 * before.volume().max(before.surface_area());
        prop_assert!((after.volume() - before.volume()).abs() <= slack);
        prop_assert!((after.surface_area() - before.surface_area()).abs() <= slack);
    }

    #[test]
    fn the_middle_of_an_extruded_lumpy_polygon_is_inside_it(
        profile in a_lumpy_profile(),
        height in 0.1f64..10.0,
    ) {
        let mut model = empty_model();
        let solid = model.add_extrusion(&profile, height, &Isometry::identity())?;

        // Star-shaped about the origin, so the middle at half height is inside
        // the material — unless the hole is there, which is punched round it.
        let sample = Point::new(0.0, 0.0, height / 2.0);
        let mut log = ProvenanceLog::new();
        let answer = model
            .solid(solid)
            .classify_point(sample, &Policy::certified(), &mut log);

        let expected = if profile.holes().is_empty() {
            Containment::Inside
        } else {
            Containment::Outside
        };
        prop_assert_eq!(answer, expected);
        prop_assert!(log.is_empty());
    }
}
