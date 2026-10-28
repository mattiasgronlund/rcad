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
//! Each now stops `Undecidable` on side of plane at the rim's corner on the
//! front plane, in the check that the LED does not cross itself:
//! `passes_through`, under `place_the_corners` (fuse/arrange.rs), cannot tell
//! which side of a plane that corner lies. In this LED the corner is
//! `(-1.23, -6.35, 4.4697)`, enclosure `5.3e-15`. These tests pin it, in this
//! LED and in the six others Q2929 found refusing as this one did.

use std::{env, fs, path::PathBuf};

use rcad_geom::SIDE_OF_PLANE;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// Where the box that misses the LED goes.
const FAR_OFF: [f64; 3] = [10.0, 10.0, 10.0];

/// The six other LED files that refused on their rim and front disc.
const THE_OTHER_SIX: [&str; 6] = [
    "LED_D3.0mm_Horizontal_O3.81mm_Z2.0mm.step",
    "LED_D3.0mm_Horizontal_O6.35mm_Z10.0mm.step",
    "LED_D3.0mm_Horizontal_O6.35mm_Z6.0mm.step",
    "LED_D5.0mm_Horizontal_O3.81mm_Z15.0mm.step",
    "LED_D5.0mm_Horizontal_O6.35mm_Z15.0mm.step",
    "LED_D5.0mm_Horizontal_O6.35mm_Z9.0mm.step",
];

#[test]
fn an_led_whose_front_disc_sits_inside_its_rim_gets_past_its_body_and_stops_at_the_rims_corner() {
    every_fuse_stops_at_the_rims_corner_on_the_front(
        "LED_D5.0mm_Horizontal_O6.35mm_Z3.0mm.step",
        &[FAR_OFF, [-1.23, -6.35, 3.0]],
    );
}

#[test]
fn six_more_leds_get_past_their_body_and_stop_at_the_rims_corner() {
    for file in THE_OTHER_SIX {
        every_fuse_stops_at_the_rims_corner_on_the_front(file, &[FAR_OFF]);
    }
}

/// The LED in `file` and a box of side 0.2 centred on each of `centres`, fused
/// under intersection and under difference, each refused `Undecidable` on side
/// of plane alone.
fn every_fuse_stops_at_the_rims_corner_on_the_front(file: &str, centres: &[[f64; 3]]) {
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
                Err(Error::Undecidable { records })
                    if records
                        .records()
                        .iter()
                        .map(|record| record.predicate())
                        .eq([SIDE_OF_PLANE]) => {}
                Err(error) => panic!(
                    "{file}: {rule:?} with a box round {centre:?} refused {error:?}, which is not \
                     the refusal at the rim's corner this pins"
                ),
                Ok(_) => panic!(
                    "{file}: {rule:?} with a box round {centre:?} answers: check its volume as \
                     f179ca02a did"
                ),
            }
        }
    }
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
