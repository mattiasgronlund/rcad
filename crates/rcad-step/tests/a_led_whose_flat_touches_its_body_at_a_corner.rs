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
//! It stops now where the flange's flat meets the body's pipe and the rim's,
//! two pipes about one axis a rounding apart in direction that never meet,
//! refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` (Q2938). These tests pin
//! that. When it answers, each turns back into the volume check of commit
//! `f179ca02a`: what the box and the LED share is the box's part inside the
//! LED, and the intersection and the difference add up to the LED.
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
//! `Z3.0` one did, and gets past it as that one does. Both stop where three
//! surfaces meet, refused `Undecidable` on the flange's flat and two curved
//! surfaces about one point: `x = -1.23` and two about `(1.27, -1.27, 15)` in
//! the `Z15.0` LED, its flat and two about `(1.27, -3.81, 6)` in the `D3.0`
//! one. That is the triple Q2938 names in
//! `LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step`, and Q2938 is the row whose
//! road it waits on. These tests pin that too.

use std::{env, fs, path::PathBuf};

use rcad_geom::{Isometry, Vector, WHERE_THREE_SURFACES_MEET};
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
fn a_box_round_the_back_corner_gets_past_the_front_disc_and_stops_where_the_flat_meets_two_pipes() {
    every_fuse_refuses(
        Z3,
        &a_box_round_each_of(&[[-1.23, -2.27, 3.0]]),
        where_the_flat_meets_two_pipes,
    );
}

#[test]
fn a_box_round_the_front_corner_gets_past_the_front_disc_and_stops_where_the_flat_meets_two_pipes()
{
    every_fuse_refuses(
        Z3,
        &a_box_round_each_of(&[[-1.23, -1.27, 3.0]]),
        where_the_flat_meets_two_pipes,
    );
}

/// The `Z15.0` LED gets past its body's pipe since Q2940 and stops where its
/// flange's flat `x = -1.23` meets two curved surfaces about
/// `(1.27, -1.27, 15)`, the far box included. Q2938 is the row whose road this
/// triple waits on.
#[test]
fn a_z15_led_gets_past_its_body_and_stops_where_three_surfaces_meet() {
    every_fuse_refuses(
        Z15,
        &a_box_round_each_of(&[[-1.23, -1.27, 15.0], [-1.23, -2.27, 15.0], FAR_OFF]),
        where_three_surfaces_meet_is_undecidable,
    );
}

#[test]
fn a_d3_led_gets_past_its_front_disc_and_stops_where_three_surfaces_meet() {
    let mut fuses = a_box_round_each_of(&[[-0.23, -3.81, 5.999999956464], [-0.23, -4.81, 6.0]]);
    fuses.push((FAR_OFF, Rule::Intersection));
    every_fuse_refuses(D3, &fuses, where_three_surfaces_meet_is_undecidable);
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
/// `fuses`, fused under its rule, each refused as `the_refusal` takes.
fn every_fuse_refuses(
    file: &str,
    fuses: &[([f64; 3], Rule)],
    the_refusal: impl Fn(&Error) -> bool,
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
                the_refusal(&error),
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

fn where_the_flat_meets_two_pipes(error: &Error) -> bool {
    matches!(
        error,
        Error::Geometry(rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
    )
}

fn where_three_surfaces_meet_is_undecidable(error: &Error) -> bool {
    let Error::Undecidable { records } = error else {
        return false;
    };
    records
        .records()
        .iter()
        .map(|record| record.predicate())
        .eq([WHERE_THREE_SURFACES_MEET])
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
