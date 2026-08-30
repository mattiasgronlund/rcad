//! [`rcad::Error`] wraps the three layers underneath it, and keeps apart the
//! three different ways this kernel can fail to reach an answer
//! (`docs/decisions.md` §4.2).
//!
//! The distinction these tests exist for: a *predicate* that cannot decide
//! returns [`Decision::Undecidable`] and that is a result; a *classification*
//! that cannot decide returns [`Containment::Undecidable`] and that is a result
//! too; only an *operation* left with nothing to hand back is a failure, and
//! that is [`Error::Undecidable`] carrying the records of what gave up.
//!
//! Every path through these tests names `rcad::` and nothing else, inner
//! variants included. That is the point of re-exporting [`TopologyError`],
//! [`GeometryError`] and [`PredicateError`]: a caller who matches on a failure
//! should never have to depend on a crate the facade exists to hide.

use rcad::{
    Approximation, Cause, Containment, Decision, Error, GeometryError, Invalid, Level, Model,
    Point, Policy, PredicateError, ProfileBuilder, ProvenanceLog, Record, Scale, TopologyError,
};

#[test]
fn one_question_mark_works_on_all_three_layers_underneath() {
    fn build_a_cube_and_check_it() -> rcad::Result<f64> {
        let scale = Scale::millimetres(100.0)?;
        let mut model = Model::new(scale);
        let cube = model.add_cube(10.0)?;
        model.validate(&Policy::certified(), &mut ProvenanceLog::new())?;
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
    let raised = Error::from(TopologyError::Undecidable { records });

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
        Error::Topology(TopologyError::Invalid(
            Invalid::ApproximationOrdering { .. }
        ))
    ));
    assert!(raised.to_string().contains("slop"), "{raised}");
}

#[test]
fn a_piece_of_geometry_that_refuses_to_be_built_is_a_geometry_error() {
    let raised = Error::from(Scale::millimetres(0.0).expect_err("a model of no size"));

    assert!(matches!(
        raised,
        Error::Geometry(GeometryError::NominalSizeNotPositive { .. })
    ));
    assert!(raised.to_string().contains("nominal size"), "{raised}");
}

#[test]
fn a_predicate_that_refuses_its_input_is_a_predicate_error() {
    let raised = Error::from(PredicateError::NotAnInterval { lo: 1.0, hi: 0.0 });

    assert!(matches!(
        raised,
        Error::Predicate(PredicateError::NotAnInterval { lo: 1.0, hi: 0.0 })
    ));
    assert!(raised.to_string().contains("interval"), "{raised}");
}

/// The whole point of the three re-exports: a caller reads a failure down to
/// the leaf without ever naming `rcad_topo`, and gets at the numbers inside it.
#[test]
fn a_caller_can_match_a_refused_profile_down_to_its_innermost_variant() {
    let raised: Error = ProfileBuilder::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
        .with_hole(vec![[8.0, 8.0], [9.0, 8.0], [9.0, 9.0], [8.0, 9.0]])
        .build(&Policy::certified())
        .expect_err("a hole nowhere near the outline is not a hole in it")
        .into();

    let Error::Topology(TopologyError::HoleNotInsideOutline { hole }) = raised else {
        panic!("a profile the builder refuses must arrive as a topology error: {raised:?}");
    };
    assert_eq!(hole, 0, "the facade hands back the number in the variant");
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
