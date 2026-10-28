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
//! planes and a pipe that touch rather than cross there (§471.3).

use std::{env, fs, path::PathBuf};

use rcad_geom::{Deflection, Isometry, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

const LED: &str = "LED_D5.0mm_Horizontal_O1.27mm_Z3.0mm.step";

/// Half the side of the box put round each corner.
const HALF: f64 = 0.1;

#[test]
fn a_box_round_the_back_corner_where_the_body_touches_the_flat_is_cut_out() {
    let back = [-1.23, -2.27, 3.0];
    let radius = 2.5_f64;
    let flange_in_the_box = HALF * HALF * (2.0 * HALF);
    let body_in_the_box = HALF
        * (HALF * (radius * radius - HALF * HALF).sqrt()
            + radius * radius * (HALF / radius).asin()
            - 2.0 * HALF * (radius - HALF));
    a_box_round_a_corner_is_cut_out(back, flange_in_the_box + body_in_the_box);
}

#[test]
fn a_box_round_the_front_corner_where_a_circle_touches_the_flat_is_cut_out() {
    let front = [-1.23, -1.27, 3.0];
    a_box_round_a_corner_is_cut_out(front, HALF * HALF * (2.0 * HALF));
}

/// The LED and a box of side `2 * HALF` centred on `corner` are fused under
/// intersection and under difference. Each answers with a model `validate`
/// accepts, what they share is `shared` to within what measuring costs, and the
/// two answers add up to the LED.
fn a_box_round_a_corner_is_cut_out(corner: [f64; 3], shared: f64) {
    let (led, solid) = the_led();
    let (whole, whole_error) = volume_of(&led, &[solid]);
    let mut volumes = Vec::new();
    for rule in [Rule::Intersection, Rule::Difference] {
        let mut model = led.clone();
        let knife = a_box_round(&mut model, corner);
        let fused = model
            .fuse(
                &[solid],
                &[knife],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{rule:?} with a box round {corner:?} refused: {why:?}"));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| {
                panic!("{rule:?} round {corner:?} is not a valid model: {why:?}")
            });
        volumes.push(volume_of(&model, fused.solids()));
    }
    let [(common, common_error), (rest, rest_error)] = volumes[..] else {
        unreachable!("two rules were fused")
    };
    assert!(
        (common - shared).abs() <= common_error + 1e-12,
        "the LED and the box round {corner:?} share {common} ± {common_error}, and {shared} is right"
    );
    assert!(
        (common + rest - whole).abs() <= common_error + rest_error + whole_error + 1e-12,
        "round {corner:?}, {common} shared and {rest} left do not add up to the LED's {whole}"
    );
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

/// The volume of `solids` together, and how far measuring it may be off.
fn volume_of(model: &Model, solids: &[SolidKey]) -> (f64, f64) {
    solids.iter().fold((0.0, 0.0), |(volume, error), &solid| {
        let measured = model
            .solid(solid)
            .expect("a solid of this model")
            .measure(
                Deflection::for_scale(model.scale()),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("it measures");
        (volume + measured.volume(), error + measured.volume_error())
    })
}
