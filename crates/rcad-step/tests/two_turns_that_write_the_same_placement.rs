//! Two placements of the same round bar write the same placement and measure
//! differently — the limit §249.5 found, and the reason
//! `read_back_and_compared` (`crates/rcad-step/tests/fuzzing/writer.rs`) now
//! checks located numbers rather than the measurement for a solid like this
//! one (§249.6, §249.7, amending §129.5 item 3).
//!
//! The pair is the eighth turn about `z`
//! (`Isometry::rotation(&Axis::z(), FRAC_PI_4)`) and the very same turn with
//! its quaternion's `w` nudged up by one ulp, then renormalised through
//! `Isometry::try_from` the way a placement read out of a saved file is.
//! Both are unit quaternions a model can hold, and turning the bar's own
//! three axes by either one lands on the same nine numbers, bit for bit —
//! §249.5's "same three columns".
//!
//! **What "write the same placement" means, checked against the actual
//! file.** A round bar's every `AXIS2_PLACEMENT_3D` and `DIRECTION` is built
//! from those same three axes, so it is identical between the two files too
//! — checked below line by line, not assumed from the three-column fact
//! alone. What is *not* the same is the one `CARTESIAN_POINT` that names the
//! seam's corner: that number comes from turning `(3, 0, 0)`, not one of the
//! three unit axes, and turning a longer vector does not reduce to scaling a
//! shorter one, bit for bit, in the arithmetic the kernel actually runs. So
//! the two files differ by exactly that one point (in two lines — the corner
//! is named once on each cap) and agree everywhere else. That is enough for
//! the two bars to measure differently, which is the limit this file pins.

use std::f64::consts::FRAC_PI_4;

use rcad_geom::{Axis, Deflection, Isometry, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::Model;

/// The eighth turn about `z`, as `Isometry::rotation` builds it — `[w, i, j,
/// k]`.
const EIGHTH_TURN_ABOUT_Z: [f64; 4] = [0.9238795325112867, 0.0, 0.0, 0.3826834323650898];

/// The same turn with `w` one ulp higher — the exact pair §249.5 found.
const ITS_TWIN_ONE_ULP_HIGHER_IN_W: [f64; 4] = [0.9238795325112868, 0.0, 0.0, 0.3826834323650898];

#[test]
fn the_eighth_turn_and_its_one_ulp_twin_write_the_same_placement_and_measure_differently() {
    let turn = Isometry::rotation(&Axis::z(), FRAC_PI_4).expect("a finite angle about a unit axis");
    assert_eq!(
        turn.rotation_quaternion(),
        EIGHTH_TURN_ABOUT_Z,
        "the eighth turn about z is not the quaternion §249.5 measured; this test would be \
         pinning a different pair"
    );
    let twin = Isometry::try_from((
        turn.translation_part().components(),
        ITS_TWIN_ONE_ULP_HIGHER_IN_W,
    ))
    .expect(
        "a quaternion of length one, nudged by one ulp and renormalised, is still a quaternion \
         of length one",
    );
    assert_ne!(
        turn.rotation_quaternion(),
        twin.rotation_quaternion(),
        "the twin must be a different quaternion for this test to mean anything"
    );

    let bar = a_bar_placed_by(&turn);
    let twin_bar = a_bar_placed_by(&twin);

    let name = "two_turns_that_write_the_same_placement";
    let text =
        rcad_step::write(&bar, name).expect("a round bar the writer already knows how to write");
    let twin_text = rcad_step::write(&twin_bar, name).expect("the same shape, placed by the twin");
    assert_placement_lines_match_and_only_the_corner_differs(&text, &twin_text);

    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let deflection = Deflection::for_scale(bar.scale());
    let measured = bar
        .solids()
        .next()
        .expect("a_bar_placed_by built one solid")
        .measure(deflection, &policy, &mut log)
        .expect("a sound bar measures");
    let twin_measured = twin_bar
        .solids()
        .next()
        .expect("a_bar_placed_by built one solid")
        .measure(deflection, &policy, &mut log)
        .expect("a sound bar measures");
    assert_ne!(
        (
            measured.volume(),
            measured.surface_area(),
            measured.centroid()
        ),
        (
            twin_measured.volume(),
            twin_measured.surface_area(),
            twin_measured.centroid()
        ),
        "the limit this test pins: two placements that write the same placement still measure \
         differently — {measured:?} against {twin_measured:?}"
    );
}

/// Every line of `text` and `twin_text` must match except the lines naming
/// the seam's corner (a `CARTESIAN_POINT`), and at least one line must
/// differ — otherwise this is not the pair it claims to be.
fn assert_placement_lines_match_and_only_the_corner_differs(text: &str, twin_text: &str) {
    let lines: Vec<&str> = text.lines().collect();
    let twin_lines: Vec<&str> = twin_text.lines().collect();
    assert_eq!(
        lines.len(),
        twin_lines.len(),
        "the two placements wrote a different number of lines"
    );
    let differing: Vec<(&str, &str)> = lines
        .iter()
        .zip(&twin_lines)
        .filter(|(one, other)| one != other)
        .map(|(&one, &other)| (one, other))
        .collect();
    assert!(
        !differing.is_empty(),
        "the twin wrote an identical file; this pair does not pin the limit it claims to"
    );
    assert!(
        differing
            .iter()
            .all(|(one, _)| one.contains("CARTESIAN_POINT")),
        "a line other than the seam's own corner differs between the two placements' files: \
         {differing:?}"
    );
}

/// A round bar of radius 3, height 5, built at the origin and then placed by
/// `turn` — the mutator's own shape and order
/// (`crates/rcad-step/tests/fuzzing/writer.rs`'s `add_round_bar` then
/// `place_solid`, which is where §249.2 found the placement composed on).
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
