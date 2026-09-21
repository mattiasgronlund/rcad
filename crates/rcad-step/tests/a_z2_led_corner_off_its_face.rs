//! The four `LED_D3.0mm_Horizontal_O1.27mm_Z2.0mm*` files are refused for a
//! corner off the curved face they put it on, and that is a limit: the file
//! disagrees with itself about the radius, so no reading of it puts the corner
//! on its face.
//!
//! `docs/decisions.md` §193.3 left them refused after the dome's slit was cut
//! open. This file is the demonstration §16.10 asks for before calling a
//! refusal a limit rather than a gap.

use std::{env, fs, path::PathBuf};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{EntityId, Import, Refusal, StepFile};

/// In `LED_D3.0mm_Horizontal_O1.27mm_Z2.0mm.step` the face `#302` is the LED's
/// neck, a `CYLINDRICAL_SURFACE` `#324` of radius `1.5` about the line through
/// `(1.27, -1.27, 2.)` along `-y`. Its loop has six edges:
///
/// - three quarter circles `#183`, `#165`, `#174` of radius `1.5`, centred on
///   that axis at `y = -2.27`,
/// - the seam `#307`, a line from `(1.27, -2.27, 3.5)` along `-y`, which lies on
///   the cylinder,
/// - the rim `#315`, a whole circle of radius **`1.499999999999`** centred on
///   the axis at `y = -5.07`, beginning and ending at the corner
///   `#308 = (1.27, -5.07, 3.499999999999)`.
///
/// The corner is exactly on the rim the file gives it, and the rim is
/// `1e-12` inside the cylinder. The seam, the same corner's other edge, runs at
/// `z = 3.5` and so misses the corner by the same `1e-12`. So the file names
/// **two radii about one axis for one face**, and the two cannot both be the
/// cylinder's.
///
/// **Why this is a limit, by the family of fixes it rules out.** A reader could
/// have picked the wrong surface, the wrong axis, the wrong sense, the wrong
/// unit or the wrong placement. The face has one surface. The distance of a
/// point from a line does not depend on which way the line points. A unit
/// scales both radii alike, and a placement moves corner and surface together,
/// and neither changes a distance ratio. The axis's `-5.55e-17` tilt in `x`
/// shifts it `2e-16` sideways at the rim, across the miss rather than along it,
/// which changes the corner's distance by about `1e-32`; worked out exactly on
/// the file's `f64`s, the distance is `1.499999999999` with or without the
/// tilt. And the decimal text is not the
/// rounding of a point on the cylinder: `3.499999999999` has thirteen
/// significant digits, so the widest reading of it is `3.4999999999995`, still
/// `5e-13` short of the `3.5` the seam and the cylinder both need.
///
/// What would make it import is reading `1e-12` as zero, which is what a
/// reader with a vertex tolerance does — OpenCASCADE's default is `1e-7` — and
/// that is the healing §42.9 declines by name.
///
/// **The siblings are clean.** `…_Z6.0mm.step` and `…_Z10.0mm.step` write every
/// circle and the dome at `1.5`; only the `Z2.0mm` exporter left the rim at
/// `1.499999999999` and the dome's ball at `1.499999999998`. Those two import,
/// which `the_clean_siblings_import` pins, so the mapping is not what refuses.
#[test]
fn a_z2_led_rim_is_a_hair_narrower_than_the_neck_it_bounds() {
    for (name, face, corner) in [
        ("LED_D3.0mm_Horizontal_O1.27mm_Z2.0mm.step", 302, 308),
        ("LED_D3.0mm_Horizontal_O1.27mm_Z2.0mm_Clear.step", 17, 22),
        ("LED_D3.0mm_Horizontal_O1.27mm_Z2.0mm_IRBlack.step", 17, 22),
        ("LED_D3.0mm_Horizontal_O1.27mm_Z2.0mm_IRGrey.step", 17, 22),
    ] {
        let Err(why) = import(name) else {
            panic!("{name} imports, so this limit has moved and this test with it");
        };
        let Refusal::ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn {
            face: refused_face,
            corner: refused_corner,
            off_by,
            coordinate_spacing,
            ..
        } = why
        else {
            panic!("{name} is refused for a corner off its curved face, and got: {why}");
        };
        assert_eq!(
            (refused_face, refused_corner),
            (EntityId(face), EntityId(corner)),
            "{name}"
        );

        let rim_radius = 1.499_999_999_999_f64;
        let neck_radius = 1.5_f64;
        let corner_from_axis = 3.499_999_999_999_f64 - 2.0;
        assert_eq!(corner_from_axis, rim_radius, "the corner is on its own rim");
        let rim_inside_neck = neck_radius - rim_radius;
        assert!(
            (rim_inside_neck - off_by).abs() < off_by * 1e-3,
            "{name}: the rim is {rim_inside_neck} inside the neck and the refusal says {off_by}, \
             so this test is no longer reading the corner the refusal names"
        );

        let widest_reading_of_the_text = 3.499_999_999_999_5_f64;
        assert!(
            3.5 - widest_reading_of_the_text > 400.0 * coordinate_spacing,
            "{name}: no rounding of thirteen written digits reaches the neck"
        );
        assert!(
            off_by > 800.0 * coordinate_spacing,
            "{name}: {off_by} against a coordinate spacing of {coordinate_spacing}"
        );
    }
}

#[test]
fn the_clean_siblings_import() {
    for name in [
        "LED_D3.0mm_Horizontal_O1.27mm_Z6.0mm.step",
        "LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step",
    ] {
        if let Err(why) = import(name) {
            panic!("{name} writes every radius as 1.5 and should import: {why}");
        }
    }
}

fn import(name: &str) -> Result<(), Refusal> {
    let bytes = fs::read(corpus().join("kicad").join(name)).expect("the LED is in the corpus");
    let file = StepFile::read(&bytes).expect("it parses");
    let mut log = ProvenanceLog::new();
    Import::read(&file, &Policy::certified(), &mut log).map(|_| ())
}

fn corpus() -> PathBuf {
    PathBuf::from(env::var_os("RCAD_STEP_CORPUS").expect("`just t` sets RCAD_STEP_CORPUS"))
}
