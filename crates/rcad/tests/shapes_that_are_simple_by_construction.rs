//! A shape that is simple by construction has to be accepted.
//!
//! This is the direction the profile checks failed in, and the direction the
//! rest of the suite had no property for. A self-crossing check that read "these
//! two bounding boxes overlap" as "these two edges cross" refused a plain right
//! triangle, and every property built on the notched star passed anyway: the
//! star's corners sit at strictly increasing angles about the origin, so no two
//! of its edges ever have boxes lying on top of each other, and neither do the
//! axis-aligned rectangles the rest of the tests use.
//!
//! The shapes generated here do. See `shapes/mod.rs` for each family and for why
//! it is simple whatever its parameters come out as.

mod shapes;

use proptest::prelude::*;
use rcad::{Deflection, Policy, ProfileBuilder, ProvenanceLog};
use shapes::{a_simple_polygon, area_of, extruded, perimeter_of};

/// How far apart two numbers that should be equal are allowed to be, as a
/// fraction of how big they are. The same figure and the same reasoning as
/// `extruded_polygons_with_holes.rs`, which has the arithmetic.
const SLACK: f64 = 1e-13;

proptest! {
    #[test]
    fn a_polygon_that_is_simple_by_construction_is_always_accepted(
        outline in a_simple_polygon(),
    ) {
        let built = ProfileBuilder::new(outline.clone()).build(&Policy::certified(), &mut ProvenanceLog::new());

        prop_assert!(
            built.is_ok(),
            "{outline:?} is simple by construction, and was refused: {:?}",
            built.err()
        );
    }

    #[test]
    fn a_polygon_that_is_simple_by_construction_extrudes_into_a_sound_solid(
        outline in a_simple_polygon(),
        height in 0.5f64..=6.0,
    ) {
        let profile = ProfileBuilder::new(outline).build(&Policy::certified(), &mut ProvenanceLog::new())?;
        let (model, solid) = extruded(&profile, height);

        model.validate(&Policy::certified(), &mut ProvenanceLog::new())?;

        let flat_area = area_of(&profile);
        let measured = model.solid(solid)?.measure(a_deflection(), &Policy::certified(), &mut ProvenanceLog::new()).expect("a solid this size measures");

        let expected_volume = flat_area * height;
        prop_assert!(
            (measured.volume() - expected_volume).abs() <= SLACK * expected_volume,
            "volume {} against a shoelace area times height of {expected_volume}",
            measured.volume()
        );

        let expected_skin = 2.0 * flat_area + perimeter_of(&profile) * height;
        prop_assert!(
            (measured.surface_area() - expected_skin).abs() <= SLACK * expected_skin,
            "surface area {} against two caps plus walls of {expected_skin}",
            measured.surface_area()
        );
    }
}

/// The deflection the measurements here are asked for.
///
/// Every solid in this file is a polyhedron, and a polyhedron ignores it: its
/// own fan is the answer rather than a picture of one (`docs/decisions.md`
/// §25.4).
fn a_deflection() -> Deflection {
    Deflection::of(0.1).expect("a tenth of a millimetre is a length")
}
