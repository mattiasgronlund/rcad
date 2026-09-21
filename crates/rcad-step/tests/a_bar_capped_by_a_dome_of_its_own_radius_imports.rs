//! A round bar whose end is a dome of the bar's own radius, centred on its axis.
//!
//! `docs/decisions.md` §189.3: the body of
//! `kicad/LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step` meets its dome along a
//! circle where the two surfaces touch rather than cross. The file names that
//! circle as a closed edge with one corner, and two things went wrong walking
//! it: the marcher's points began a quarter turn from that corner, and nothing
//! said which way round to run them, because where two surfaces touch their
//! crossing has no direction.
//!
//! **It does not import yet**, and what stops it now is measured: the marcher
//! itself, which cannot follow two surfaces that touch all the way round at
//! every deflection `Model::validate` asks of it. The body's axis is written
//! `(-5.551115123126E-17, -1, 0)` and the dome's centre sits `3.8` along it, so
//! as stored the centre is off the axis by `3.8 × 5.55e-17` — about `2.1e-16` —
//! and the kernel's exact arithmetic says so. The pair as stored does not meet
//! in the file's circle, which is why no closed form certified by exact signs
//! can draw it.

use std::{env, fs, path::PathBuf};

use rcad_geom::Error as Geometry;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};

/// Both LEDs used to stop on the body, face 8, as a wire that "will not lie
/// flat". Now neither does. The taller one stops on the marcher following the
/// body's circle; the shorter gets past the body altogether and stops on its
/// flat face with the pin holes, whose boundary crosses itself.
#[test]
fn the_leds_body_no_longer_stops_them_where_it_meets_its_dome() {
    let taller = why_it_does_not_import("LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step");
    assert!(
        matches!(
            &taller,
            rcad_topo::Error::ADraftEdgeHasACrossingThatCannotBeFollowed { why, .. }
                if matches!(**why, Geometry::CannotFollowACrossingBecauseNoStepStaysOnIt { .. })
        ),
        "the taller LED stops somewhere other than the marcher on the body's circle: {taller:?}"
    );

    // `CannotTriangulate` is `rcad-mesh`'s, which this crate does not depend on,
    // so its variant is read off its name.
    let shorter = why_it_does_not_import("LED_D3.0mm_Horizontal_O1.27mm_Z6.0mm.step");
    assert!(
        matches!(
            &shorter,
            rcad_topo::Error::ADraftFaceCouldNotBeMeshed { why, .. }
                if format!("{why:?}").starts_with("BoundaryCrossesItself")
        ),
        "the shorter LED stops somewhere other than a boundary crossing itself: {shorter:?}"
    );
}

fn why_it_does_not_import(name: &str) -> rcad_topo::Error {
    let bytes = fs::read(corpus().join("kicad").join(name)).expect("the LED is in the corpus");
    let file = StepFile::read(&bytes).expect("it parses");
    let refused = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
        .expect_err("the LED does not import yet");
    let (Refusal::ThisEntityOfTheSolidWasRefused { why, .. }
    | Refusal::TheSolidTheFileDescribesIsNotOne { why, .. }) = refused
    else {
        panic!("{name} is refused before it reaches the model: {refused:?}");
    };
    *why
}

fn corpus() -> PathBuf {
    PathBuf::from(env::var_os("RCAD_STEP_CORPUS").expect("`just t` sets RCAD_STEP_CORPUS"))
}
