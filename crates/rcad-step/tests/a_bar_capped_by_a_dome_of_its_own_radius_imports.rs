//! A round bar whose end is a dome of the bar's own radius, centred on its axis.
//!
//! `docs/decisions.md` §189.3: the body of
//! `kicad/LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step` meets its dome along a
//! circle where the two surfaces touch rather than cross. The file names that
//! circle as a closed edge with one corner.
//!
//! The marcher cannot follow two surfaces that touch, and no closed form
//! certified by exact signs can draw the circle either: the body's axis is
//! written `(-5.551115123126E-17, -1, 0)` and the dome's centre sits `3.8` along
//! it, so as stored the centre is off the axis by about `2.1e-16`. But the file
//! names the circle, so the answer exists. The reader now checks the file's
//! circle lies on both the body and the dome, and then names the run by the
//! circle's plane, which the walk crosses with the dome to draw it.
//!
//! **None of the nine LEDs imports yet.** The two `_O1.27mm` ones get past the
//! body and the dome and stop on their flat face with the pin holes. The seven
//! siblings §188.3 counted as "50 shortenings" get past the body too, and stop
//! on the dome's own wire, which the mesher will not lay flat.

use std::{env, fs, path::PathBuf};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};

/// Both `_O1.27mm` LEDs now stop on entity `#223`, the flat face with the pin
/// holes, whose boundary crosses itself. The taller one used to stop on the
/// marcher following the body's circle.
///
/// `CannotTriangulate` is `rcad-mesh`'s, which this crate does not depend on,
/// so its variant is read off its name.
#[test]
fn the_leds_body_and_dome_no_longer_stop_them() {
    for name in [
        "LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step",
        "LED_D3.0mm_Horizontal_O1.27mm_Z6.0mm.step",
    ] {
        let (entity, why) = why_it_does_not_import(name);
        assert!(
            entity == Some(223)
                && matches!(
                    &why,
                    rcad_topo::Error::ADraftFaceCouldNotBeMeshed { why, .. }
                        if format!("{why:?}").starts_with("BoundaryCrossesItself")
                ),
            "{name} stops somewhere other than a boundary crossing itself on #223: \
             {entity:?} {why:?}"
        );
    }
}

/// One of the seven §188.3 files whose body used to stop the marcher after 50
/// shortenings on entity `#317`. The body is walked now, and what stops it is
/// the dome's wire.
#[test]
fn a_sibling_led_gets_past_its_body_and_stops_on_its_dome() {
    let (_, why) = why_it_does_not_import("LED_D3.0mm_Horizontal_O3.81mm_Z10.0mm.step");
    assert!(
        matches!(why, rcad_topo::Error::AWireWillNotLieFlatOnItsFace { .. }),
        "the sibling LED stops somewhere other than its dome's wire: {why:?}"
    );
}

/// The file entity a refusal names, if it names one, and why.
fn why_it_does_not_import(name: &str) -> (Option<u64>, rcad_topo::Error) {
    let bytes = fs::read(corpus().join("kicad").join(name)).expect("the LED is in the corpus");
    let file = StepFile::read(&bytes).expect("it parses");
    let refused = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
        .expect_err("the LED does not import yet");
    if let Refusal::TheSolidTheFileDescribesIsNotOne { why, .. } = refused {
        return (None, *why);
    }
    let Refusal::ThisEntityOfTheSolidWasRefused { entity, why, .. } = refused else {
        panic!("{name} is refused before it reaches the model: {refused:?}");
    };
    (Some(entity.0), *why)
}

fn corpus() -> PathBuf {
    PathBuf::from(env::var_os("RCAD_STEP_CORPUS").expect("`just t` sets RCAD_STEP_CORPUS"))
}
