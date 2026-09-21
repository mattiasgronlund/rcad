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
//! **These LEDs import now.** They got past the body with §189.3 and past the
//! dome's wire with §192.1. What stopped them after that was the dome's seam,
//! which in this kernel's chart of the ball is a slit, and the mesher now cuts
//! the dome open along it (§192.2). They were reported on their flat face with
//! the pin holes, `#223`, because that is the coarsest refusal; the finer
//! deflections were all refused on the dome.

use std::{env, fs, path::PathBuf};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};

/// Both `_O1.27mm` LEDs, which used to stop on `#223`. The taller one used to
/// stop on the marcher following the body's circle before that.
#[test]
fn the_leds_body_and_dome_no_longer_stop_them() {
    for name in [
        "LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step",
        "LED_D3.0mm_Horizontal_O1.27mm_Z6.0mm.step",
    ] {
        imports_and_sweeps_clean(name);
    }
}

/// One of the seven §188.3 files whose body used to stop the marcher after 50
/// shortenings on entity `#317`. The body is walked now, and so are the dome's
/// wire (§192.1) and its seam (§192.2).
#[test]
fn a_sibling_led_gets_past_its_body_and_its_dome() {
    imports_and_sweeps_clean("LED_D3.0mm_Horizontal_O3.81mm_Z10.0mm.step");
}

fn imports_and_sweeps_clean(name: &str) {
    let bytes = fs::read(corpus().join("kicad").join(name)).expect("the LED is in the corpus");
    let file = StepFile::read(&bytes).expect("it parses");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let import = Import::read(&file, &policy, &mut log)
        .unwrap_or_else(|refused| panic!("{name} does not import: {refused:?}"));
    import
        .model()
        .validate(&policy, &mut log)
        .unwrap_or_else(|why| panic!("{name} imports but the full sweep refuses it: {why:?}"));
}

fn corpus() -> PathBuf {
    PathBuf::from(env::var_os("RCAD_STEP_CORPUS").expect("`just t` sets RCAD_STEP_CORPUS"))
}
