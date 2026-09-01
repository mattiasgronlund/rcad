//! Turning a pair of surfaces does not change what kind of thing they cross in.
//!
//! §24.18's criterion 9, and roadmap §11.2's "start here": a transform
//! invariance needs no oracle, because the question is only whether two runs of
//! the same code agree.
//!
//! # The one thing criterion 9 could not have known
//!
//! **A pair on a knife edge is allowed to change its answer, and that is the
//! honest behaviour rather than a hole in the property.** Every predicate in
//! this kernel is exact *about the numbers as stored* and says nothing about the
//! shape those numbers were rounded from (`docs/decisions.md` §17.1, and the
//! note on `Plane::side_of_point`). Turning a plane that rests exactly on a ball
//! rounds its normal and its offset, and the turned plane genuinely no longer
//! rests on the turned ball — it cuts a tiny circle or misses by a hair. A
//! property that insisted otherwise would be asking the kernel to answer about
//! the pair somebody meant.
//!
//! So the two knife edges are named, skipped and **counted**, and the counts are
//! printed. They are exactly the two configurations a rounding of the inputs can
//! move across:
//!
//! - **a tangency** — a plane resting on a ball, or two balls resting on each
//!   other — where the discriminant is exactly zero; and
//! - **two directions exactly square or exactly parallel**: a plane along a
//!   pipe's axis, or two parallel planes. The pipe one is the surprise. A plane
//!   *not* parallel to an endless pipe always cuts it in an ellipse, however far
//!   away it is, so "parallel and well clear of it" answers `Nothing` and the
//!   smallest tilt turns that into an enormous ellipse. The knife edge there is
//!   not a near miss; it is the whole of the `Nothing` answer.
//!
//! Everything else has a discriminant clear of zero by more than a rotation can
//! move it, and for those the kind is the same before and after. §28 records the
//! sharpening.

#![forbid(unsafe_code)]

mod crossings;

use std::cell::Cell;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngAlgorithm, TestRng, TestRunner};
use rcad_geom::{Error, Surface, SurfaceMeet, intersect};
use rcad_predicates::{Policy, ProvenanceLog};

use crossings::{Turn, a_lattice_crossing};

/// How many pairs this turns.
const CASES: u32 = 2048;

/// How many of them have to survive the knife-edge skip.
///
/// Three fifths. Without a floor the property would pass just as green if every
/// pair were skipped, which is §14.7's vacuity trap.
///
/// It is not higher because of what the control family *is*: every direction on
/// the lattice is a coordinate axis, so two of them are exactly parallel or
/// exactly square far more often than two directions drawn at random would be.
/// Measured on the shipped corpus: 1485 of 2048 compared, 564 skipped.
const MOST_PAIRS_MUST_BE_COMPARED: f64 = 0.6;

#[test]
fn turning_a_pair_of_surfaces_names_the_same_kind_of_crossing() {
    let mut runner = TestRunner::new_with_rng(
        Config {
            cases: CASES,
            source_file: Some(file!()),
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
                "proptest-regressions",
            ))),
            ..Config::default()
        },
        TestRng::deterministic_rng(RngAlgorithm::ChaCha),
    );

    let compared = Cell::new(0usize);
    let on_a_knife_edge = Cell::new(0usize);
    let outcome = runner.run(&(a_lattice_crossing(), a_turn()), |(crossing, turn)| {
        let still = crossing;
        let mut moved = crossing;
        moved.turn = Some(turn);

        if is_on_a_knife_edge(&still.surfaces()) {
            on_a_knife_edge.set(on_a_knife_edge.get() + 1);
            return Ok(());
        }
        let before = named(&still.surfaces())?;
        if matches!(before, SurfaceMeet::TouchAtOnePoint { .. }) {
            on_a_knife_edge.set(on_a_knife_edge.get() + 1);
            return Ok(());
        }
        let after = named(&moved.surfaces())?;

        compared.set(compared.get() + 1);
        prop_assert_eq!(
            kind_of(&before),
            kind_of(&after),
            "turning this pair changed what kind of thing it crosses in"
        );
        Ok(())
    });
    if let Err(why) = outcome {
        panic!("{why}");
    }

    println!(
        "{} of {CASES} pairs compared, {} skipped for sitting on a knife edge",
        compared.get(),
        on_a_knife_edge.get()
    );
    assert!(
        compared.get() as f64 >= MOST_PAIRS_MUST_BE_COMPARED * f64::from(CASES),
        "only {} of {CASES} pairs were off a knife edge, so this property is barely asserting \
         anything",
        compared.get()
    );
}

/// The crossing, or `Ok(())` out of the whole case where it could not be named.
///
/// A refusal is measured in `how_often_naming_a_crossing_refuses.rs` and is not
/// this file's business; what matters here is that a pair which *was* named
/// keeps the same kind of answer.
fn named(surfaces: &[Surface; 2]) -> Result<SurfaceMeet, TestCaseError> {
    match intersect(
        &surfaces[0],
        &surfaces[1],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Ok(met) => Ok(met),
        Err(Error::Undecidable { .. } | Error::CannotNameWhichPieceOfACrossingThisIs) => {
            Err(TestCaseError::reject("this pair could not be named"))
        }
        Err(why) => Err(TestCaseError::fail(format!("unexpected refusal: {why}"))),
    }
}

/// What kind of thing two surfaces cross in, with nothing in it that a rotation
/// could move.
fn kind_of(met: &SurfaceMeet) -> (&'static str, usize) {
    match met {
        SurfaceMeet::Nothing => ("nothing", 0),
        SurfaceMeet::TouchAtOnePoint { .. } => ("one point", 0),
        SurfaceMeet::Curves(runs) => ("curves", runs.len()),
        SurfaceMeet::SameSurface => ("the same surface", 0),
        other => panic!("unhandled crossing: {other:?}"),
    }
}

/// Whether a rounding of these two directions could move the answer.
///
/// Asked of the surfaces and not of the answer, because the plane-and-pipe case
/// is a knife edge even when the answer is `Nothing`. See the file's opening.
fn is_on_a_knife_edge(surfaces: &[Surface; 2]) -> bool {
    match surfaces {
        [Surface::Plane(flat), Surface::Cylinder(pipe)]
        | [Surface::Cylinder(pipe), Surface::Plane(flat)] => {
            flat.normal().dot(pipe.axis().direction()) == 0.0
        }
        [Surface::Plane(one), Surface::Plane(other)] => {
            one.normal().cross(other.normal()).length_squared() == 0.0
        }
        _ => false,
    }
}

fn a_turn() -> impl Strategy<Value = Turn> {
    ([-3i32..=3, -3i32..=3, -3i32..=3], 1u32..64).prop_map(|(direction, steps)| Turn {
        direction: if direction == [0, 0, 0] {
            [0, 0, 1]
        } else {
            direction
        },
        steps,
    })
}
