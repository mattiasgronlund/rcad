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
//! It stops now at the disc of radius 2.5 on the flange's front, refused
//! `AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve`. These tests pin that. When the
//! disc answers, each turns back into the volume check of commit `f179ca02a`:
//! what the box and the LED share is the box's part inside the LED, and the
//! intersection and the difference add up to the LED.

use std::{env, fs, path::PathBuf};

use rcad_geom::{Isometry, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Error, FaceKey, Model, ProfileBuilder, Rule, SolidKey};

const LED: &str = "LED_D5.0mm_Horizontal_O1.27mm_Z3.0mm.step";

/// Half the side of the box put round each corner.
const HALF: f64 = 0.1;

#[test]
fn a_box_round_the_back_corner_gets_past_the_rim_and_stops_at_the_front_disc() {
    a_box_round_a_corner_stops_at_the_front_disc([-1.23, -2.27, 3.0]);
}

#[test]
fn a_box_round_the_front_corner_gets_past_the_rim_and_stops_at_the_front_disc() {
    a_box_round_a_corner_stops_at_the_front_disc([-1.23, -1.27, 3.0]);
}

/// The LED and a box of side `2 * HALF` centred on `corner`, fused under
/// intersection and under difference, each refused as having an edge of the
/// disc on the flange's front not drawn along its own curve.
fn a_box_round_a_corner_stops_at_the_front_disc(corner: [f64; 3]) {
    let (led, solid) = the_led();
    let disc = the_disc_on_the_front(&led, solid);
    for rule in [Rule::Intersection, Rule::Difference] {
        let mut model = led.clone();
        let knife = a_box_round(&mut model, corner);
        match model.fuse(
            &[solid],
            &[knife],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) {
            Err(Error::AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve { face }) => assert_eq!(
                face, disc,
                "{rule:?} with a box round {corner:?} names a face other than the front disc"
            ),
            Ok(_) => panic!(
                "{rule:?} with a box round {corner:?} answers: check its volume as f179ca02a did"
            ),
            Err(other) => panic!("{rule:?} with a box round {corner:?} refused: {other:?}"),
        }
    }
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

/// The disc on the flange's front: the flat face of the LED in the plane
/// `y = -1.27` whose every corner is within 2.5 of the axis `x = 1.27, z = 3`.
/// The rest of the flange's front reaches out to the rim, 2.9 from it.
fn the_disc_on_the_front(led: &Model, solid: SolidKey) -> FaceKey {
    let discs: Vec<FaceKey> = led
        .solid(solid)
        .expect("a solid of this model")
        .faces()
        .filter(|face| {
            let Surface::Plane(_) = face.surface() else {
                return false;
            };
            face.wires()
                .flat_map(|wire| wire.coedges())
                .map(|coedge| coedge.start().point())
                .all(|corner| {
                    (corner.y() + 1.27).abs() < 1e-9
                        && (corner.x() - 1.27).hypot(corner.z() - 3.0) < 2.5 + 1e-9
                })
        })
        .map(|face| face.key())
        .collect();
    let [disc] = discs[..] else {
        panic!("the LED has one disc of radius 2.5 on its front, not {discs:?}")
    };
    disc
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
