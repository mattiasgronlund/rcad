//! A small box cut out of a KiCad LED where the flat side of its flange
//! touches the round body, and where it touches the round disc on its front.
//!
//! `LED_D5.0mm_Horizontal_O1.27mm_Z3.0mm.step` is a pipe of radius 2.5 about
//! the line `x = 1.27, z = 3` running back along `-y` from `y = -2.27`, with a
//! flange in front of it from `y = -2.27` to `y = -1.27`: a disc of radius 2.9
//! cut flat at `x = -1.23`, which is 2.5 from the axis, so the flat touches the
//! pipe along the line `z = 3`. The file keeps two corners on that line, and
//! each holds two fans of faces (`docs/decisions.md` §471.1):
//!
//! - `#123` at `(-1.23, -2.27, 3)`, on the flange's back: the flange's fan, its
//!   flat and its back, and the body's fan, its pipe and the flange's back.
//! - `#74` at `(-1.23, -1.27, 3)`, on the flange's front, where a circle of
//!   radius 2.5 cut into the front face touches the flat.
//!
//! Every fuse of the LED tried by Q2911's lane refused `Undecidable` on where
//! three surfaces meet, at `#123`, handed the flat, the back and the pipe: two
//! planes and a pipe that touch rather than cross there (§471.3). Bake now
//! reads such a corner out of the model, and every fuse of the LED gets past
//! it and stops later, in the check that the LED does not cross itself.
//!
//! It stopped first on the upper half of the flange's rim, a pipe of radius
//! 2.9 that no box here touches, refused `AFaceTrimmedByACurveThisCannotFollow`.
//! The rim's edge along the flat is a `LINE` in the file, so nothing picks it,
//! and the flat's normal is `3.6e-32` off square to the rim's axis, so the two
//! surfaces cross in an ellipse about `1.6e32` long with that edge on it. The
//! arrangement now tells which of the ellipse's two arcs the edge is by the
//! rim's box, which the other arc leaves (Q2926).
//!
//! It stopped next at the disc of radius 2.5 on the flange's front, refused
//! `AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve`, from the rim against the disc:
//! the disc's arcs are drawn between two planes a rounding apart, and nothing
//! baked said they ran along the body's pipe. The check now skips a flat face
//! a pipe holds strictly inside, and the disc is 0.4 inside the rim (Q2947).
//!
//! It stopped next where the flange's flat meets the body's pipe and the
//! rim's, two pipes about one axis a rounding apart in direction that never
//! meet, refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`. Q2938's road
//! answers that triple.
//!
//! It stopped next on the flange's front, face slot 2, the plane `y = -1.27`,
//! refused `AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve` by
//! `cut_the_crossings_into_arcs` (fuse/arrange.rs) in the check that the LED
//! does not cross itself. The front drew the arcs round the disc as where its
//! plane crosses the disc's, two spellings of one plane, which is a straight
//! line and not the circle the arcs run along. The arrangement now draws such
//! an arc where the face's plane crosses the pipe it runs along (Q2966).
//!
//! It stops now `Undecidable` on side of cylinder at `#74`, the front corner:
//! bake solves it from the front, the flat and the disc, three planes two of
//! which are a rounding apart, and the box it gets runs along the flat, `z`
//! from `2.50` to `3.27` (Q2939). These tests pin that. When it answers, each
//! turns back into the volume check of commit `f179ca02a`: what the box and
//! the LED share is the box's part inside the LED, and the intersection and
//! the difference add up to the LED.
//!
//! `LED_D5.0mm_Horizontal_O1.27mm_Z15.0mm.step` is the same LED with its axis
//! at `z = 15`, and `LED_D3.0mm_Horizontal_O3.81mm_Z6.0mm.step` a smaller one,
//! its body a pipe of radius 1.5 about `x = 1.27, z = 6` with its flange's
//! flat at `x = -0.23` and its front at `y = -3.81`. Every fuse of either that
//! Q2928's row tried refused `Undecidable` on where three surfaces meet, the
//! far box included, and it was neither at the back corner: it was at the
//! front one, `(-1.23, -1.27, 15)` and `(-0.23, -3.81, 5.999999956464)`. There
//! the flange's flat meets its front and the body's front disc, and the
//! front's plane and the disc's are one plane written down twice, a rounding
//! apart: `(0, 1, -1.15e-15)` and `(5.6e-17, 1, 0)`. So the corner has three
//! planes round it that share a line, solving it finds a box along that line
//! that misses the stored corner, and bake would not read a corner with three
//! planes round it out of the model. It now does, and asks every plane about
//! it as it asks a solved corner.
//!
//! Past that corner, the `Z15.0` LED stopped at its body's pipe, refused
//! `AFaceTrimmedByACurveThisCannotFollow`: the fuse baked each arc where its
//! dome meets the pipe against the ball across it, and two curved surfaces
//! that only touch cannot say which way along the rim an arc runs. Bake now
//! reads such an arc against the rim's plane (Q2940), and the `Z15.0` LED gets
//! past its body's pipe. The `D3.0` LED stopped at its front disc as the
//! `Z3.0` one did, and gets past it as that one does.
//!
//! Both stopped next where three surfaces meet, refused `Undecidable` on the
//! flange's flat and two curved surfaces about one point: `x = -1.23` and two
//! about `(1.27, -1.27, 15)` in the `Z15.0` LED, its flat and two about
//! `(1.27, -3.81, 6)` in the `D3.0` one. The two curved surfaces are the
//! body's pipe and the rim's, about one axis a rounding apart in direction,
//! the pair Q2938 names. The flat's normal there is exactly `(-1, 0, 0)`, and
//! the road along a pipe ran out of range writing the lines the flat cuts. It
//! now hands the triple on to Q2938's road for a plane that cuts both in
//! conics, which answers it as it answers the `Z3.0` LED's flat that leans
//! off `x` by a rounding.
//!
//! Both stopped next at the front corner, refused `Undecidable` with the one
//! record `order along a crossing`, the far box included: in the `Z15.0` LED
//! between the corner `(-1.23, -1.27, 15)` and a point solved near
//! `(-1.23, -1.27, 15.075)` in a box `4.4e-16` wide, in the `D3.0` one between
//! `(-0.23, -3.81, 5.999999956464)` and a point near `z = 6.147`. Those points
//! were on the straight line where the front's plane crosses the disc's, which
//! the front drew its arcs round the disc as until Q2966.
//!
//! The `Z15.0` LED stopped next on `order along a crossing` between two points
//! both within `1e-14` of the front corner, enclosure `1.8e-14`, in the check
//! that the LED does not cross itself. They were the two places where the
//! flat's line on the front meets the circle the body's pipe cuts in it, and
//! the line touches that circle at the corner. Their boxes share a point and
//! both lie inside the room writing the corner down cost, so nothing proves
//! them two points, and the corner now stands for both (Q2959).
//!
//! Both LEDs stop now on the flange's front, face slot 8 in the `Z15.0` LED
//! and slot 2 in the `D3.0` one, refused `CannotTellTheCellsOfAFaceApart` in
//! the same check, where the windings of the front's cells are spread (Q2974).
//! These tests pin both.

use std::{env, fs, path::PathBuf};

use rcad_geom::{Isometry, SIDE_OF_CYLINDER, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

const Z3: &str = "LED_D5.0mm_Horizontal_O1.27mm_Z3.0mm.step";
const Z15: &str = "LED_D5.0mm_Horizontal_O1.27mm_Z15.0mm.step";
const D3: &str = "LED_D3.0mm_Horizontal_O3.81mm_Z6.0mm.step";

/// Half the side of the box put round each corner.
const HALF: f64 = 0.1;

/// Where the box that misses the LED goes.
const FAR_OFF: [f64; 3] = [10.0, 10.0, 10.0];

#[test]
fn a_box_round_the_back_corner_gets_past_the_arcs_round_the_disc_and_stops_at_the_front_corner() {
    every_fuse_refuses(
        Z3,
        &a_box_round_each_of(&[[-1.23, -2.27, 3.0]]),
        the_side_of_a_pipe_is_undecidable,
    );
}

#[test]
fn a_box_round_the_front_corner_gets_past_the_arcs_round_the_disc_and_stops_at_it() {
    every_fuse_refuses(
        Z3,
        &a_box_round_each_of(&[[-1.23, -1.27, 3.0]]),
        the_side_of_a_pipe_is_undecidable,
    );
}

/// The `Z15.0` LED gets past its body's pipe since Q2940, and past where its
/// flange's flat `x = -1.23` meets two curved surfaces about
/// `(1.27, -1.27, 15)` since Q2938's road answers a flat whose normal is
/// exactly `(-1, 0, 0)`. Since Q2966 it gets past the point solved near
/// `(-1.23, -1.27, 15.075)` on the straight line the front drew its arcs round
/// the disc as. Since Q2959 the two places where the flat's line on the front
/// meets the disc's circle, both within `1e-14` of the front corner and not
/// proved apart, are the corner, and it stops where the `D3.0` LED does, the
/// far box included.
#[test]
fn a_z15_led_gets_past_the_two_meets_at_its_front_corner_and_stops_telling_the_cells_of_its_front_apart()
 {
    every_fuse_refuses(
        Z15,
        &a_box_round_each_of(&[[-1.23, -1.27, 15.0], [-1.23, -2.27, 15.0], FAR_OFF]),
        |model, error| the_cells_of_the_front_cannot_be_told_apart(model, error, -1.27),
    );
}

#[test]
fn a_d3_led_gets_past_the_arcs_round_the_disc_and_stops_telling_the_cells_of_its_front_apart() {
    let mut fuses = a_box_round_each_of(&[[-0.23, -3.81, 5.999999956464], [-0.23, -4.81, 6.0]]);
    fuses.push((FAR_OFF, Rule::Intersection));
    every_fuse_refuses(D3, &fuses, |model, error| {
        the_cells_of_the_front_cannot_be_told_apart(model, error, -3.81)
    });
}

/// A box round each corner, to be fused with the LED under intersection and
/// under difference.
fn a_box_round_each_of(corners: &[[f64; 3]]) -> Vec<([f64; 3], Rule)> {
    corners
        .iter()
        .flat_map(|&corner| [Rule::Intersection, Rule::Difference].map(|rule| (corner, rule)))
        .collect()
}

/// The LED in `file` and a box of side `2 * HALF` centred on each corner of
/// `fuses`, fused under its rule, each refused as `the_refusal` takes, handed
/// the model it was refused in.
fn every_fuse_refuses(
    file: &str,
    fuses: &[([f64; 3], Rule)],
    the_refusal: impl Fn(&Model, &Error) -> bool,
) {
    let (led, solid) = the_led(file);
    for &(corner, rule) in fuses {
        let mut model = led.clone();
        let knife = a_box_round(&mut model, corner);
        match model.fuse(
            &[solid],
            &[knife],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) {
            Err(error) => assert!(
                the_refusal(&model, &error),
                "{file}: {rule:?} with a box round {corner:?} refused {error:?}, which is not \
                 the refusal this pins"
            ),
            Ok(_) => panic!(
                "{file}: {rule:?} with a box round {corner:?} answers: check its volume as \
                 f179ca02a did"
            ),
        }
    }
}

fn the_side_of_a_pipe_is_undecidable(_: &Model, error: &Error) -> bool {
    let Error::Undecidable { records } = error else {
        return false;
    };
    records
        .records()
        .iter()
        .map(|record| record.predicate())
        .eq([SIDE_OF_CYLINDER])
}

/// Refused `CannotTellTheCellsOfAFaceApart` on the flange's front, the face
/// of the LED on the plane `y = front`.
fn the_cells_of_the_front_cannot_be_told_apart(model: &Model, error: &Error, front: f64) -> bool {
    let Error::CannotTellTheCellsOfAFaceApart { face } = error else {
        return false;
    };
    let surface = model
        .face(*face)
        .expect("the refusal names a face of the LED")
        .surface();
    let Surface::Plane(plane) = surface else {
        return false;
    };
    [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
        .into_iter()
        .all(|[x, z]| {
            plane
                .signed_distance_to(rcad_geom::Point::new(x, front, z))
                .abs()
                < 1e-9
        })
}

fn the_led(file: &str) -> (Model, SolidKey) {
    let path =
        PathBuf::from(env::var_os("RCAD_STEP_CORPUS").expect("`just t` sets RCAD_STEP_CORPUS"))
            .join("kicad")
            .join(file);
    let bytes = fs::read(&path).expect("the LED is in the corpus");
    let step = StepFile::read(&bytes).expect("it parses");
    let import = Import::read(&step, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the LED imports");
    let [(_, solid)] = import.solids()[..] else {
        panic!("{file} is one solid, not {}", import.solids().len())
    };
    (import.model().clone(), solid)
}

fn a_box_round(model: &mut Model, corner: [f64; 3]) -> SolidKey {
    let side = 2.0 * HALF;
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let low = Vector::new(corner[0] - HALF, corner[1] - HALF, corner[2] - HALF);
    model
        .add_extrusion(
            &square,
            side,
            &Isometry::translation(low).expect("a finite shift"),
        )
        .expect("a box")
}
