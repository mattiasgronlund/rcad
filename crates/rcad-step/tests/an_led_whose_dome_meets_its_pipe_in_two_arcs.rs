//! A small box fused with a KiCad LED whose round dome meets its body's pipe
//! in two arcs.
//!
//! `LED_D5.0mm-3.step` is a body standing up `z`: a pipe of radius 2.5 about
//! the line `x = 2.54, y = 0`, topped at `z = 9.1` by a dome, a ball of the same
//! radius centred on that line. The ball touches the pipe all the way round the
//! rim `z = 9.1` and never crosses it. The file draws the rim as two arcs, from
//! `(5.04, 0, 9.1)` to `(0.04, 0, 9.1)` and back, and each arc's run names the
//! rim's plane. Its one corner where two fans of faces touch is
//! `(0.04, 0, 4)`, where the flange's flat touches the pipe (§471.3).
//!
//! `LED_D3.0mm_Horizontal_O6.35mm_Z2.0mm.step` is the same kind of LED lying
//! down: a pipe of radius 1.5 about `x = 1.27, z = 2` running back along `-y`,
//! with its dome about `(1.27, -10.15, 2)` and its rim on the plane
//! `y = -10.15`. Its two corners in two fans are at `(-0.23, -6.35, 2)` and
//! `(-0.23, -7.35, 2)`.
//!
//! Every fuse of either LED tried by Q2931's lane refused
//! `AFaceTrimmedByACurveThisCannotFollow` naming face slot 2, the body's pipe,
//! the box at `(10, 10, 10)` included. The fuse baked each arc of the pipe's
//! rim against the ball across it, and the arrangement could not tell which
//! way along the rim an arc runs from two curved surfaces that only touch. Bake
//! now reads such an arc against the plane its run names, as `classify` does
//! since Q2923, and every fuse of either LED gets past the rim (Q2940).
//!
//! These tests pin where each stops now:
//!
//! - The `D5.0` LED stops where its dome is split, refused
//!   `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`: the plane `y = 0.2` or
//!   `y = -0.2`, the ball and the pipe, which touch along the rim, have no road
//!   to where the three meet (Q2941).
//! - The `D3.0` LED stopped there too, where its flange's flat meets the body's
//!   pipe and the flange's rim, a pipe of radius 1.9 about the same axis a
//!   rounding apart in direction. Q2938's road answers that triple, and the
//!   fuse now stops in the check that the LED does not cross itself, on the
//!   flange's flat `x = -0.23`: `crossing_for` (fuse/arrange.rs) finds the flat
//!   meets one of its neighbours on its own plane, and it is refused
//!   `AFaceMeetsItsNeighbourOnItsOwnSurface`.
//!
//! When either answers, its test turns into the volume check of commit
//! `f179ca02a`: what the box and the LED share is the box's part inside the
//! LED, and the intersection and the difference add up to the LED.

use std::{env, fs, path::PathBuf};

use rcad_geom::{Isometry, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

const D5: &str = "LED_D5.0mm-3.step";
const D3: &str = "LED_D3.0mm_Horizontal_O6.35mm_Z2.0mm.step";

/// Half the side of the box put round each corner.
const HALF: f64 = 0.1;

/// Where the box that misses the LED goes.
const FAR_OFF: [f64; 3] = [10.0, 10.0, 10.0];

#[test]
fn a_d5_led_gets_past_its_dome_rim_and_stops_where_its_dome_is_split() {
    every_fuse_refuses(D5, &[[0.04, 0.0, 4.0], FAR_OFF], |error| {
        matches!(
            error,
            Error::Geometry(rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
        )
    });
}

#[test]
fn a_d3_led_gets_past_where_its_flat_meets_two_pipes_and_stops_on_its_flat() {
    every_fuse_refuses(
        D3,
        &[[-0.23, -6.35, 1.999999956464], [-0.23, -7.35, 2.0], FAR_OFF],
        |error| matches!(error, Error::AFaceMeetsItsNeighbourOnItsOwnSurface { .. }),
    );
}

/// The LED in `file` and a box of side `2 * HALF` centred on each of
/// `centres`, fused under intersection and under difference, each refused as
/// `the_refusal` takes.
fn every_fuse_refuses(file: &str, centres: &[[f64; 3]], the_refusal: impl Fn(&Error) -> bool) {
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
                Err(error) if the_refusal(&error) => {}
                Err(error) => panic!(
                    "{file}: {rule:?} with a box round {centre:?} refused {error:?}, which is not \
                     the refusal this pins"
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
    let side = 2.0 * HALF;
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let low = Vector::new(centre[0] - HALF, centre[1] - HALF, centre[2] - HALF);
    model
        .add_extrusion(
            &square,
            side,
            &Isometry::translation(low).expect("a finite shift"),
        )
        .expect("a box")
}
