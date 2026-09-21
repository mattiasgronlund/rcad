//! A KiCad LED's dome is a half ball whose rim is a circle through both of the
//! ball's poles, and its wire has to lie flat in the ball's own coordinates.
//!
//! `docs/decisions.md` §190.2 left 18 `LED_D*_Horizontal` files refused on the
//! dome by `AWireWillNotLieFlatOnItsFace`. The rim, read in the ball's chart,
//! is two meridians half a turn apart, one pole a corner of the file and the
//! other in the middle of a piece of the rim. The mesher took the pole side at
//! each pole the short way round, and at exactly half a turn that goes the
//! wrong way, so the walk came back a whole turn from where it set off.

use std::{env, fs, path::PathBuf};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};

/// The smallest of the 18 files. Its dome is walked now, and what stops it is
/// `#223`, the flat face with the pin holes, whose boundary crosses itself —
/// the same place the two `_O1.27mm` `D3.0` files stop (§190.2).
///
/// `CannotTriangulate` is `rcad-mesh`'s, which this crate does not depend on,
/// so its variant is read off its name.
#[test]
fn the_dome_of_an_led_is_walked_and_the_led_stops_on_its_pin_face() {
    let name = "LED_D3.0mm_Horizontal_O3.81mm_Z6.0mm.step";
    let bytes = fs::read(corpus().join("kicad").join(name)).expect("the LED is in the corpus");
    let file = StepFile::read(&bytes).expect("it parses");
    let refused = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
        .expect_err("the LED does not import yet");
    let Refusal::ThisEntityOfTheSolidWasRefused { entity, why, .. } = refused else {
        panic!("{name} is refused somewhere other than one of its faces: {refused:?}");
    };
    assert!(
        entity.0 == 223
            && matches!(
                &*why,
                rcad_topo::Error::ADraftFaceCouldNotBeMeshed { why, .. }
                    if format!("{why:?}").starts_with("BoundaryCrossesItself")
            ),
        "{name} stops somewhere other than a boundary crossing itself on #223: \
         #{} {why:?}",
        entity.0
    );
}

fn corpus() -> PathBuf {
    PathBuf::from(env::var_os("RCAD_STEP_CORPUS").expect("`just t` sets RCAD_STEP_CORPUS"))
}
