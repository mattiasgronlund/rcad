//! Properties of solids made by extruding a polygon that is not convex and has
//! holes in it (`docs/decisions.md` §9.1, M0 exit criterion 9).
//!
//! The shapes and the flat-polygon arithmetic that acts as the oracle live in
//! `shapes/mod.rs`, together with the explanation of what shrinks and toward
//! what.
//!
//! # How tight the comparisons are, and why
//!
//! Volume comes out of a sum of signed tetrahedron volumes, one per triangle of
//! every face. The individual terms are far bigger than the total they add up
//! to — the tetrahedra reach out to the origin and then cancel — so the
//! relative error grows with the triangle count rather than staying at a single
//! rounding. A shape here has at most sixteen outline corners and three
//! four-cornered holes, which is fewer than a hundred triangles, so the sum
//! carries on the order of two hundred roundings: `200 * f64::EPSILON`, about
//! `4e-14`.
//!
//! [`SLACK`] is `1e-13`, that bound with room to spare. The worst seen over
//! several thousand runs is `2e-15`.

mod shapes;

use proptest::prelude::*;
use rcad::{Containment, Model, Point, Policy, ProvenanceLog};
use shapes::{
    Material, a_notched_polygon_with_holes, area_of, extruded, perimeter_of, where_the_point_sits,
};

/// How far apart two numbers that should be equal are allowed to be, as a
/// fraction of how big they are. See the module docs for where it comes from.
const SLACK: f64 = 1e-13;

/// How close to an edge a sample point may sit before the flat oracle declines
/// to say which side of it the point is on.
///
/// Six orders of magnitude above the `f64` spacing at these coordinates, and
/// eight below the smallest hole. It exists so that a disagreement is never
/// this file's floating-point arithmetic being wrong about a close call.
const NOT_A_CLOSE_CALL: f64 = 1e-9;

proptest! {
    #[test]
    fn an_extruded_polygon_with_holes_is_always_a_sound_solid(
        profile in a_notched_polygon_with_holes(),
        height in 0.5f64..=6.0,
    ) {
        let (model, solid) = extruded(&profile, height);

        model.validate(&Policy::certified())?;

        let measured = model.solid(solid).measure();
        prop_assert!(measured.volume() > 0.0, "volume {}", measured.volume());
        prop_assert!(measured.surface_area() > 0.0);
        prop_assert_eq!(model.solid(solid).shells().count(), 1);
    }

    #[test]
    fn the_volume_is_the_flat_area_times_the_height(
        profile in a_notched_polygon_with_holes(),
        height in 0.5f64..=6.0,
    ) {
        let (model, solid) = extruded(&profile, height);

        let expected = area_of(&profile) * height;
        prop_assert!(expected > 0.0, "the shoelace area came out {expected}");
        let measured = model.solid(solid).measure().volume();
        prop_assert!(
            (measured - expected).abs() <= SLACK * expected,
            "volume {measured} against a shoelace area times height of {expected}"
        );
    }

    #[test]
    fn the_surface_area_is_the_two_caps_plus_the_walls(
        profile in a_notched_polygon_with_holes(),
        height in 0.5f64..=6.0,
    ) {
        let (model, solid) = extruded(&profile, height);

        // The walls stand on the holes as well as on the outline, which is why
        // the perimeter is the total round every loop.
        let expected = 2.0 * area_of(&profile) + perimeter_of(&profile) * height;
        let measured = model.solid(solid).measure().surface_area();
        prop_assert!(
            (measured - expected).abs() <= SLACK * expected,
            "surface area {measured} against two caps plus walls of {expected}"
        );
    }

    #[test]
    fn a_solid_written_out_and_read_back_is_the_same_solid(
        profile in a_notched_polygon_with_holes(),
        height in 0.5f64..=6.0,
    ) {
        let (model, solid) = extruded(&profile, height);
        let before = model.solid(solid).measure();

        let bytes = postcard::to_allocvec(&model)?;
        let read_back: Model = postcard::from_bytes(&bytes)?;

        read_back.validate(&Policy::certified())?;
        let same = read_back.solids().next().expect("the solid came back");
        // Exactly equal, not nearly. Coordinates go out and come back as the
        // same bits and the measurement adds them up in the same order, so
        // anything less than equality would mean something moved.
        prop_assert_eq!(same.measure(), before);
    }

    #[test]
    fn the_centroid_of_an_extruded_polygon_is_inside_it(
        profile in a_notched_polygon_with_holes(),
        height in 0.5f64..=6.0,
    ) {
        let (model, solid) = extruded(&profile, height);
        let middle = model.solid(solid).measure().centroid();

        let mut log = ProvenanceLog::new();
        let sits = model
            .solid(solid)
            .classify_point(middle, &Policy::certified(), &mut log);

        match sits {
            Containment::Inside => {}
            Containment::OnBoundary => {
                prop_assert!(false, "the centroid landed exactly on the skin")
            }
            Containment::Outside => prop_assert!(false, "the centroid landed outside the solid"),
            Containment::Undecidable { enclosure } => {
                prop_assert!(false, "the centroid could not be placed: enclosure {enclosure}")
            }
            other => prop_assert!(false, "unhandled answer: {other:?}"),
        }
        prop_assert!(log.is_empty(), "an exact question needed no approximation");
    }
}

proptest! {
    // Every sample walks every triangle of every face with exact predicates, so
    // this one property costs more than the other five together. Fewer cases,
    // each of them worth much more.
    #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

    #[test]
    fn a_grid_of_points_is_classified_the_way_the_flat_outline_says(
        profile in a_notched_polygon_with_holes(),
        height in 0.5f64..=6.0,
    ) {
        let (model, solid) = extruded(&profile, height);
        let policy = Policy::certified();

        for step_across in 0..5 {
            for step_up in 0..5 {
                // Offsets that are not round numbers, so no sample lands on the
                // shape's own symmetry lines.
                let flat = [
                    -14.0 + 7.1 * f64::from(step_across),
                    -13.0 + 6.7 * f64::from(step_up),
                ];
                let Some(material) = where_the_point_sits(&profile, flat, NOT_A_CLOSE_CALL) else {
                    continue;
                };

                for fraction in [0.3, 0.7] {
                    let sample = Point::new(flat[0], flat[1], height * fraction);
                    let mut log = ProvenanceLog::new();
                    let sits = model.solid(solid).classify_point(sample, &policy, &mut log);
                    prop_assert!(log.is_empty(), "an exact question needed no approximation");

                    match (material, sits) {
                        (Material::Solid, Containment::Inside)
                        | (Material::Empty, Containment::Outside) => {}
                        (_, Containment::OnBoundary) => prop_assert!(
                            false,
                            "{sample:?} is {NOT_A_CLOSE_CALL} clear of every edge, so it is not on the skin"
                        ),
                        (Material::Solid, Containment::Outside) => prop_assert!(
                            false,
                            "{sample:?} is in the material but came back outside"
                        ),
                        (Material::Empty, Containment::Inside) => prop_assert!(
                            false,
                            "{sample:?} is outside the outline or inside a hole, but came back inside"
                        ),
                        (_, Containment::Undecidable { enclosure }) => prop_assert!(
                            false,
                            "{sample:?} could not be placed: enclosure {enclosure}"
                        ),
                        (_, other) => prop_assert!(false, "unhandled answer: {other:?}"),
                    }
                }
            }
        }
    }
}
