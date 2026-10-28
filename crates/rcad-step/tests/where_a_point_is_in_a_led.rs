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
//! it, not the pipe its run names, and two planes cross in no arc. Past that,
//! it refused face slot 8, the body's pipe, the same way: the dome is a ball of
//! the pipe's radius, the rim where they meet is two half circles, and each
//! was read against the ball across it, not the rim's plane its run names.
//! Both are now read against the surface the run names.

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

/// A grid through the LED behind its front, where the leads are not, from the
/// dome's far side to past the flange's rim. Its steps are odd numbers so that
/// few points fall on a face, and those within `1e-3` of one are left out.
#[test]
fn every_point_of_a_grid_behind_the_leds_front_is_where_its_shape_puts_it() {
    let (led, solid) = the_led();
    let led = led.solid(solid).expect("a solid of this model");
    let mut asked = 0;
    let mut misses = Vec::new();
    for i in 0..12_i32 {
        for j in 0..20_i32 {
            for k in 0..12_i32 {
                let point = [
                    -2.1 + 0.6113 * f64::from(i),
                    -10.6 + 0.4807 * f64::from(j),
                    -0.6 + 0.6517 * f64::from(k),
                ];
                let Some(inside) = whether_the_leds_shape_holds(point) else {
                    continue;
                };
                asked += 1;
                let want = if inside {
                    Containment::Inside
                } else {
                    Containment::Outside
                };
                let mut log = ProvenanceLog::new();
                let got = led.classify_point(
                    Point::new(point[0], point[1], point[2]),
                    &Policy::certified(),
                    &mut log,
                );
                if got != want || !log.is_empty() {
                    misses.push((point, got, log.len()));
                }
            }
        }
    }
    assert!(asked > 2500, "only {asked} points were asked");
    assert!(
        misses.is_empty(),
        "{} of {asked} points missed: {misses:?}",
        misses.len()
    );
}

/// Whether the LED as drawn holds `point`, from its numbers: the body's pipe
/// of radius 2.5 from its front back to the dome's rim, the flange of radius
/// 2.9 cut flat at `x = -1.23`, and the dome, a ball of radius 2.5 centred on
/// the rim. `None` within `1e-3` of a face, or in front of `y = -1.3`, where
/// the leads are.
fn whether_the_leds_shape_holds([x, y, z]: [f64; 3]) -> Option<bool> {
    let off_the_axis = (x - 1.27).hypot(z - 3.0);
    let off_the_domes_centre = off_the_axis.hypot(y + 7.37);
    let to_each_face = [
        off_the_axis - 2.5,
        off_the_axis - 2.9,
        x + 1.23,
        y + 1.27,
        y + 2.27,
        y + 7.37,
        off_the_domes_centre - 2.5,
    ];
    if y > -1.3 || to_each_face.iter().any(|apart| apart.abs() < 1e-3) {
        return None;
    }
    let in_the_body = (-7.37..-1.27).contains(&y) && off_the_axis < 2.5;
    let in_the_flange = (-2.27..-1.27).contains(&y) && off_the_axis < 2.9 && x > -1.23;
    let in_the_dome = y < -7.37 && off_the_domes_centre < 2.5;
    Some(in_the_body || in_the_flange || in_the_dome)
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
