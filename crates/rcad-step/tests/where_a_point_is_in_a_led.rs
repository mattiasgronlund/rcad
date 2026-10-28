//! Where a point is in a KiCad LED whose front is two flat faces on one plane.
//!
//! `LED_D5.0mm_Horizontal_O1.27mm_Z3.0mm.step` is a pipe of radius 2.5 about
//! the line `x = 1.27, z = 3` running back along `-y` from `y = -1.27` to a
//! dome, with a flange round it from `y = -2.27` to `y = -1.27`: a disc of
//! radius 2.9 cut flat at `x = -1.23`. Its front, `y = -1.27`, is two faces:
//! the flange's front (face slot 2), the disc less a circle of radius 2.5, and
//! the body's front (face slot 7), that circle's disc. The file stores the two
//! planes a rounding apart, and the circle between them as three arcs, each
//! where the front crosses a pipe of radius 2.5.
//!
//! `classify_point` answered `CannotFollowAFacesTrimming` naming face slot 2
//! at every point, near the LED or far off (`docs/decisions.md` §471.3, row
//! Q2923): what holds the flange's front read the arc from `(1.27, -1.27,
//! 5.5)` to `(-1.23, -1.27, 3)` against the plane of the body's front across
//! it, not the pipe its run names, and two planes cross in no arc.

use std::{env, fs, path::PathBuf};

use rcad_geom::Point;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Containment, Model, SolidKey};

const LED: &str = "LED_D5.0mm_Horizontal_O1.27mm_Z3.0mm.step";

#[test]
fn the_middle_of_the_leds_body_is_inside_it() {
    assert_eq!(where_it_is([1.27, -5.0, 3.0]), Containment::Inside);
}

#[test]
fn a_point_on_the_leds_axis_inside_its_flange_is_inside_it() {
    assert_eq!(where_it_is([1.27, -1.77, 3.0]), Containment::Inside);
}

#[test]
fn a_point_in_the_flange_past_the_body_is_inside_it() {
    assert_eq!(where_it_is([1.27, -1.77, 5.7]), Containment::Inside);
}

#[test]
fn a_point_past_the_flanges_rim_is_outside_it() {
    assert_eq!(where_it_is([1.27, -1.77, 6.5]), Containment::Outside);
}

#[test]
fn a_point_far_off_is_outside_it() {
    assert_eq!(where_it_is([100.0, 100.0, 100.0]), Containment::Outside);
}

/// Where `point` is in the LED under a certified policy, with an empty log.
fn where_it_is(point: [f64; 3]) -> Containment {
    let (led, solid) = the_led();
    let mut log = ProvenanceLog::new();
    let answer = led
        .solid(solid)
        .expect("a solid of this model")
        .classify_point(
            Point::new(point[0], point[1], point[2]),
            &Policy::certified(),
            &mut log,
        );
    assert!(log.is_empty(), "{point:?} left records: {log:?}");
    answer
}

fn the_led() -> (Model, SolidKey) {
    let path =
        PathBuf::from(env::var_os("RCAD_STEP_CORPUS").expect("`just t` sets RCAD_STEP_CORPUS"))
            .join("kicad")
            .join(LED);
    let bytes = fs::read(&path).expect("the LED is in the corpus");
    let file = StepFile::read(&bytes).expect("it parses");
    let import = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the LED imports");
    let [(_, solid)] = import.solids()[..] else {
        panic!("the LED is one solid, not {}", import.solids().len())
    };
    (import.model().clone(), solid)
}
