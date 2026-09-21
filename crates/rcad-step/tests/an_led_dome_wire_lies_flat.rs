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
use rcad_step::{Import, StepFile};

/// The smallest of the 18 files. Its dome is walked (§192.1), and since the
/// mesher cuts the dome open along its seam rather than handing the
/// triangulator a slit (§192.2), the whole LED imports.
#[test]
fn the_dome_of_an_led_is_walked_and_the_led_imports() {
    let name = "LED_D3.0mm_Horizontal_O3.81mm_Z6.0mm.step";
    let bytes = fs::read(corpus().join("kicad").join(name)).expect("the LED is in the corpus");
    let file = StepFile::read(&bytes).expect("it parses");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let import = Import::read(&file, &policy, &mut log)
        .unwrap_or_else(|refused| panic!("{name} does not import: {refused:?}"));
    import
        .model()
        .validate(&policy, &mut log)
        .expect("and the full sweep is happy with what it brought");
}

fn corpus() -> PathBuf {
    PathBuf::from(env::var_os("RCAD_STEP_CORPUS").expect("`just t` sets RCAD_STEP_CORPUS"))
}
