//! [`rcad::Error`] wraps the three layers underneath it, and keeps apart the
//! three different ways this kernel can fail to reach an answer
//! (`docs/decisions.md` §4.2).
//!
//! The distinction these tests exist for: a *predicate* that cannot decide
//! returns [`Decision::Undecidable`] and that is a result; a *classification*
//! that cannot decide returns [`Containment::Undecidable`] and that is a result
//! too; only an *operation* left with nothing to hand back is a failure, and
//! that is [`Error::Undecidable`] carrying the records of what gave up.

use rcad::{
    Approximation, Cause, Containment, Decision, Error, Invalid, Level, Model, Point, Policy,
    ProvenanceLog, Record, Scale,
};

#[test]
fn one_question_mark_works_on_all_three_layers_underneath() {
    fn build_a_cube_and_check_it() -> rcad::Result<f64> {
        let scale = Scale::millimetres(100.0)?;
        let mut model = Model::new(scale);
        let cube = model.add_cube(10.0)?;
        model.validate(&Policy::certified())?;
        Ok(model.solid(cube).measure().volume())
    }

    assert_eq!(
        build_a_cube_and_check_it().expect("nothing here fails"),
        1000.0
    );
}

#[test]
fn an_operation_that_could_not_decide_keeps_its_own_variant_at_the_facade() {
    let mut records = ProvenanceLog::new();
    records.push(Record::new(
        "orient3d",
        &[],
        Level::Interval,
        Decision::Undecidable { enclosure: 1e-17 },
    ));

    // Buried inside `Error::Topology` this would be worth nothing: a caller
    // would have to unwrap two layers to learn that what happened was "could
    // not decide" rather than "bad input".
    let raised = Error::from(rcad_topo::Error::Undecidable { records });

    match raised {
        Error::Undecidable { records } => {
            assert_eq!(records.len(), 1);
            assert_eq!(records.worst_enclosure(), 1e-17);
        }
        other => panic!("an undecided operation must stay undecided: {other:?}"),
    }
}

#[test]
fn a_model_that_refuses_a_tag_says_which_invariant_it_would_break() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    model.add_cube(2.0).expect("a cube of side two");
    let face = model.faces().next().expect("a cube has faces").key();

    let raised = Error::from(
        model
            .set_face_approximation(face, Approximation::new(1e-6, Cause::TolerantFallback))
            .expect_err("a face may not carry more slop than the edges it is built from"),
    );

    assert!(matches!(
        raised,
        Error::Topology(rcad_topo::Error::Invalid(
            Invalid::ApproximationOrdering { .. }
        ))
    ));
    assert!(raised.to_string().contains("slop"), "{raised}");
}

#[test]
fn a_piece_of_geometry_that_refuses_to_be_built_is_a_geometry_error() {
    let raised = Error::from(Scale::millimetres(0.0).expect_err("a model of no size"));

    assert!(matches!(raised, Error::Geometry(_)));
    assert!(raised.to_string().contains("nominal size"), "{raised}");
}

#[test]
fn a_predicate_that_refuses_its_input_is_a_predicate_error() {
    let raised = Error::from(rcad_predicates::Error::NotAnInterval { lo: 1.0, hi: 0.0 });

    assert!(matches!(raised, Error::Predicate(_)));
    assert!(raised.to_string().contains("interval"), "{raised}");
}

#[test]
fn a_classification_that_lands_on_the_skin_is_an_answer_and_not_an_error() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let cube = model.add_cube(2.0).expect("a cube of side two");
    let mut log = ProvenanceLog::new();

    let answer =
        model
            .solid(cube)
            .classify_point(Point::new(0.0, 0.0, 0.0), &Policy::certified(), &mut log);

    match answer {
        Containment::OnBoundary => {}
        Containment::Inside | Containment::Outside => {
            panic!("a corner of a cube is on its skin, not on one side of it")
        }
        Containment::Undecidable { enclosure } => {
            panic!("exact coordinates decide exactly: enclosure {enclosure}")
        }
        other => panic!("unhandled answer: {other:?}"),
    }
    assert!(log.is_empty(), "nothing had to be approximated");
}
