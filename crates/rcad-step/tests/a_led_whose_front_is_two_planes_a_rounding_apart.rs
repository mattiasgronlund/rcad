//! A KiCad LED whose front disc sits 0.4 inside the rim round it, with the
//! disc's arcs drawn between two planes a rounding apart.
//!
//! `LED_D5.0mm_Horizontal_O6.35mm_Z3.0mm.step` is a pipe of radius 2.5 about
//! the line `x = 1.27, z = 3`, with a flange in front of it whose rim is a pipe
//! of radius 2.9 about the same line. The disc on the front of the body, in the
//! plane `y = -6.35`, is bounded by three arcs of the body's pipe, and across
//! each arc is the flange's front, whose plane is the disc's own written down
//! a rounding apart: `(5.6e-17, 1, 0)` and `(-1.0e-16, 1, -1.1e-15)`.
//!
//! Every fuse of it refused `Undecidable` in the check that it does not cross
//! itself, on the rim against the disc: side of plane at the rim's corner
//! `(-1.23, -6.35, 4.4697)`, enclosure `5.3e-15` (Q2929). The disc is nowhere
//! near the rim, but the baked solid could not say so: each arc's side named
//! only the flange's front across it, a plane, and the pipe the arc runs along
//! was dropped. Bake now carries that pipe, and the check skips a flat face a
//! pipe provably holds strictly inside (Q2947).
//!
//! Every fuse here then stopped where the flange's flat, `x = -1.23`, meets
//! the body's pipe and the rim's: two pipes about one axis, a rounding apart
//! in direction, which never meet. That was refused
//! `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, and is answered now (Q2938).
//!
//! Past that, without Q2940, every fuse here stopped on the body's pipe (face
//! slot 8), refused `AFaceTrimmedByACurveThisCannotFollow`: the fuse baked
//! each arc where the dome meets the pipe against the ball across it. Bake now
//! reads such an arc against the rim's plane (Q2940), and every fuse here gets
//! past the body's pipe too.
//!
//! Each then stopped `Undecidable` on side of plane at the rim's corner on the
//! front plane, `(-1.23, -6.35, 4.4697)` in this LED, enclosure `5.3e-15`, in
//! the check that the LED does not cross itself. The flange's front drew the
//! three arcs round the disc as where its plane crosses the disc's, two
//! spellings of one plane, which is a straight line nowhere near the arcs, and
//! `place_the_corners` (fuse/arrange.rs) asked which side of the disc's plane
//! the rim's corner lies. The arrangement now draws such an arc where the
//! face's plane crosses the pipe it runs along (Q2966).
//!
//! Each then stopped `Undecidable` on side of cylinder at the front corner,
//! where the circle round the disc touches the flange's flat, against a box
//! that ran along the flat, `z` from `0.834` to `4.170` in this LED. Bake
//! solved that corner from three planes, the flat, the flange's front and the
//! disc, the last two a rounding apart, so they meet anywhere along the flat's
//! front edge. Bake now reads a corner in two fans of faces out of the model,
//! as it already did when solving one refused (Q2939).
//!
//! Each stops now on the flange's front, face slot 2, the plane `y = -6.35`
//! here, refused `CannotTellTheCellsOfAFaceApart`, the refusal Q2974 traces on
//! `LED_D3.0mm_Horizontal_O3.81mm_Z6.0mm.step`, whose front corner was read
//! out of the model since Q2928. These tests pin it, in this LED and in the
//! six others Q2929 found refusing as this one did.

use std::{env, fs, path::PathBuf};

use rcad_geom::Surface;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// Where the box that misses the LED goes.
const FAR_OFF: [f64; 3] = [10.0, 10.0, 10.0];

/// The six other LED files that refused on their rim and front disc, each with
/// the `y` of its flange's front.
const THE_OTHER_SIX: [(&str, f64); 6] = [
    ("LED_D3.0mm_Horizontal_O3.81mm_Z2.0mm.step", -3.81),
    ("LED_D3.0mm_Horizontal_O6.35mm_Z10.0mm.step", -6.35),
    ("LED_D3.0mm_Horizontal_O6.35mm_Z6.0mm.step", -6.35),
    ("LED_D5.0mm_Horizontal_O3.81mm_Z15.0mm.step", -3.81),
    ("LED_D5.0mm_Horizontal_O6.35mm_Z15.0mm.step", -6.35),
    ("LED_D5.0mm_Horizontal_O6.35mm_Z9.0mm.step", -6.35),
];

#[test]
fn an_led_whose_disc_sits_inside_its_rim_gets_past_its_front_corner_to_the_cells_of_its_front() {
    every_fuse_stops_on_the_front(
        "LED_D5.0mm_Horizontal_O6.35mm_Z3.0mm.step",
        -6.35,
        &[FAR_OFF, [-1.23, -6.35, 3.0]],
    );
}

#[test]
fn six_more_leds_get_past_their_front_corner_to_the_cells_of_their_front() {
    for (file, front) in THE_OTHER_SIX {
        every_fuse_stops_on_the_front(file, front, &[FAR_OFF]);
    }
}

/// The LED in `file` and a box of side 0.2 centred on each of `centres`, fused
/// under intersection and under difference, each refused
/// `CannotTellTheCellsOfAFaceApart` on the LED's face on the plane
/// `y = front`.
fn every_fuse_stops_on_the_front(file: &str, front: f64, centres: &[[f64; 3]]) {
    let (led, solid) = the_led(file);
    for &centre in centres {
        for rule in [Rule::Intersection, Rule::Difference] {
            let mut model = led.clone();
            let knife = a_box_round(&mut model, centre);
            match model.fuse(
                &[solid],
                &[knife],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            ) {
                Err(error) => assert!(
                    the_cells_of_the_front_cannot_be_told_apart(&model, &error, front),
                    "{file}: {rule:?} with a box round {centre:?} refused {error:?}, which is not \
                     the refusal on the front this pins"
                ),
                Ok(_) => panic!(
                    "{file}: {rule:?} with a box round {centre:?} answers: check its volume as \
                     f179ca02a did"
                ),
            }
        }
    }
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

fn a_box_round(model: &mut Model, centre: [f64; 3]) -> SolidKey {
    let side = 0.2;
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let low = rcad_geom::Vector::new(centre[0] - 0.1, centre[1] - 0.1, centre[2] - 0.1);
    model
        .add_extrusion(
            &square,
            side,
            &rcad_geom::Isometry::translation(low).expect("a finite shift"),
        )
        .expect("a box")
}
