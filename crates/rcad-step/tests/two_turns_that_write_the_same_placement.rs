//! Two turns that a STEP placement cannot tell apart, and that measure a round
//! bar differently: the limit §249.5 found. It is why `read_back_and_compared`
//! (`tests/fuzzing/writer.rs`) checks the measurement of a curved solid only
//! when that solid has no placement (§249.6, amending §129.5 item 3).
//!
//! The pair is the eighth turn about `z`, as `Isometry::rotation` builds it, and
//! the same quaternion with `w` one ulp higher, built by `Isometry::try_from`
//! (the route a saved model's placement takes in).
//!
//! **"The same placement" means the placement as STEP writes one.** STEP has no
//! quaternion. It writes a placement as turned axes: an axis and a reference
//! direction in an `AXIS2_PLACEMENT_3D`, or all three in a
//! `CARTESIAN_TRANSFORMATION_OPERATOR_3D`. Both turns take the three unit axes
//! to the same nine numbers, bit for bit. So a writer that wrote the bar in its
//! stored frame with the placement beside it (§249.5's third family of fixes)
//! would write one file for both turns. Whichever turn a reader rebuilt from
//! it, one of the two bars would come back measuring differently.
//!
//! **Today's writer is not that writer.** It turns every corner into the file
//! (§191.2), and the corner at `(3, 0, 0)` turns one ulp apart under the two.
//! So its two files differ at that corner. That is no way out of the limit:
//! the reader bakes either file into a bar with no placement, and that bar is
//! cut into triangles in a different frame from the bar that was written.

use std::f64::consts::FRAC_PI_4;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::Model;

/// The eighth turn about `z`, as `Isometry::rotation` builds it, as `[w, i, j,
/// k]`.
const EIGHTH_TURN_ABOUT_Z: [f64; 4] = [0.9238795325112867, 0.0, 0.0, 0.3826834323650898];

/// The same turn with `w` one ulp higher: the pair §249.5 found.
const ITS_TWIN_ONE_ULP_HIGHER_IN_W: [f64; 4] = [0.9238795325112868, 0.0, 0.0, 0.3826834323650898];

#[test]
fn the_eighth_turn_about_z_and_its_twin_turn_the_three_axes_to_the_same_nine_numbers() {
    let (turn, twin) = the_eighth_turn_and_its_twin();
    for axis in [
        Vector::new(1.0, 0.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
        Vector::new(0.0, 0.0, 1.0),
    ] {
        let turned = turn.apply_to_vector(axis);
        let turned_by_the_twin = twin.apply_to_vector(axis);
        assert_eq!(
            turned.components().map(f64::to_bits),
            turned_by_the_twin.components().map(f64::to_bits),
            "{axis:?} turns to {turned:?} and to {turned_by_the_twin:?}, so a STEP placement \
             could tell these two turns apart"
        );
    }
}

#[test]
fn a_bar_placed_by_each_of_the_two_turns_measures_differently() {
    let (turn, twin) = the_eighth_turn_and_its_twin();
    let measured = the_one_solid_measured(&a_bar_placed_by(&turn));
    let measured_by_the_twin = the_one_solid_measured(&a_bar_placed_by(&twin));
    assert_ne!(
        (
            measured.volume(),
            measured.surface_area(),
            measured.centroid()
        ),
        (
            measured_by_the_twin.volume(),
            measured_by_the_twin.surface_area(),
            measured_by_the_twin.centroid()
        ),
        "the two bars measure the same, so this pair no longer shows that a placement STEP \
         writes the same can leave a bar measuring differently"
    );
}

#[test]
fn todays_writer_writes_the_two_bars_differently_at_their_corner() {
    let (turn, twin) = the_eighth_turn_and_its_twin();
    let corner = Point::new(3.0, 0.0, 0.0);
    assert_ne!(
        turn.apply_to_point(corner),
        twin.apply_to_point(corner),
        "the bar's corner turns to the same place under both turns"
    );

    let name = "two_turns_that_write_the_same_placement";
    let text = rcad_step::write(&a_bar_placed_by(&turn), name).expect("a round bar is written");
    let text_of_the_twin =
        rcad_step::write(&a_bar_placed_by(&twin), name).expect("a round bar is written");
    assert_eq!(
        text.lines().count(),
        text_of_the_twin.lines().count(),
        "the two bars were written with a different number of lines"
    );
    let differing: Vec<(&str, &str)> = text
        .lines()
        .zip(text_of_the_twin.lines())
        .filter(|(line, twin_line)| line != twin_line)
        .collect();
    assert!(
        !differing.is_empty(),
        "today's writer wrote the two bars as one file"
    );
    assert!(
        differing
            .iter()
            .all(|(line, _)| line.contains("CARTESIAN_POINT")),
        "the two bars' files differ somewhere other than at a point: {differing:?}"
    );
}

/// The eighth turn about `z`, and the same turn with `w` one ulp higher.
fn the_eighth_turn_and_its_twin() -> (Isometry, Isometry) {
    let turn = Isometry::rotation(&Axis::z(), FRAC_PI_4).expect("a finite angle about a unit axis");
    assert_eq!(
        turn.rotation_quaternion(),
        EIGHTH_TURN_ABOUT_Z,
        "the eighth turn about z is not the quaternion §249.5 measured"
    );
    let twin = Isometry::try_from((
        turn.translation_part().components(),
        ITS_TWIN_ONE_ULP_HIGHER_IN_W,
    ))
    .expect("four finite numbers, not all zero");
    assert_eq!(
        twin.rotation_quaternion(),
        ITS_TWIN_ONE_ULP_HIGHER_IN_W,
        "scaling the twin to length one moved it, so it is no longer the pair §249.5 measured"
    );
    (turn, twin)
}

/// A round bar of radius 3 and height 5, built at the origin and then placed
/// by `turn`, the order the writer's fuzz target builds one in.
fn a_bar_placed_by(turn: &Isometry) -> Model {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = model
        .add_round_bar(3.0, 5.0, &Isometry::identity())
        .expect("a round bar of radius 3 fits in a model this size");
    model
        .place_solid(solid, turn)
        .expect("the key came from this model");
    model
}

fn the_one_solid_measured(model: &Model) -> rcad_topo::Measurement {
    model
        .solids()
        .next()
        .expect("a_bar_placed_by built one solid")
        .measure(
            Deflection::for_scale(model.scale()),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a sound bar measures")
}
