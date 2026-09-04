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
//! - **a tangency** — a plane resting on a ball, two balls resting on each
//!   other, or a ball resting all the way round the inside of a pipe — where the
//!   discriminant is exactly zero; and
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
//!
//! # Both families, and why that had to be said out loud
//!
//! There are two properties here and there used to be one. The four pairs whose
//! crossing lies in a plane are `a_lattice_crossing`'s; a ball crossing a pipe
//! and a pipe crossing a pipe are `a_quartic_lattice_crossing`'s, and nothing
//! was handing the second strategy to this file. Criterion 9 read as discharged
//! and covered two thirds of the pair table. The claim audit at M3's closing
//! gate is what found it, and it found it by enumerating what the generator
//! ranges over rather than by reading the property — which is the whole of
//! §16.10's method (§30).
//!
//! Measured: **1485 of 2048 planar pairs compared and 564 skipped, 1619 of 2048
//! quartic pairs compared and 430 skipped.**

#![forbid(unsafe_code)]

mod crossings;
mod soak;

use std::cell::Cell;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, TestRunner};
use rcad_geom::{Error, Surface, SurfaceMeet, intersect};
use rcad_predicates::{Policy, ProvenanceLog};

use crossings::{Crossing, Elsewhere, Piece, Turn, a_lattice_crossing, a_quartic_lattice_crossing};

/// How many pairs this turns on the shipped corpus, which `soak::cases`
/// multiplies and `soak::seeded` reseeds.
const CASES: u32 = 2048;

/// How many of the planar pairs have to survive the knife-edge skip.
///
/// Three fifths. Without a floor the property would pass just as green if every
/// pair were skipped, which is §14.7's vacuity trap.
///
/// It is not higher because of what the control family *is*: every direction on
/// the lattice is a coordinate axis, so two of them are exactly parallel or
/// exactly square far more often than two directions drawn at random would be.
/// Measured on the shipped corpus: 1485 of 2048 compared, 564 skipped.
const MOST_PLANAR_PAIRS_MUST_BE_COMPARED: f64 = 0.6;

/// How many of the quartic pairs have to survive it.
///
/// Half. The generator draws radii up to five and places them up to six apart in
/// whole numbers, so a lattice pipe pair is exactly tangent surprisingly often —
/// five apart with radii of two and three is one — and every one of those is a
/// knife edge a turn moves off (§29.5). Measured on the shipped corpus: 1619 of
/// 2048 compared, 430 skipped.
const MOST_QUARTIC_PAIRS_MUST_BE_COMPARED: f64 = 0.5;

/// The four pairs whose crossing lies in a plane.
#[test]
fn turning_a_pair_of_surfaces_names_the_same_kind_of_crossing() {
    the_same_kind_before_and_after(
        a_lattice_crossing().boxed(),
        MOST_PLANAR_PAIRS_MUST_BE_COMPARED,
    );
}

/// **The two rows M3's second stage added**, which this file did not draw until
/// the claim audit at the closing gate asked what criterion 9 was covering.
///
/// It was covering the four planar pairs: `a_lattice_crossing`'s own doc says a
/// ball crossing a pipe and a pipe crossing a pipe are drawn by
/// `a_quartic_pair` instead, and nothing was passing that strategy to this
/// property. So criterion 9 was discharged for stage one and silently not for
/// stage two — a sufficiency claim broken by what the generator did not draw
/// rather than by anything the code did (§30).
///
/// The quartic pairs have one knife edge the planar ones do not: a crossing that
/// **runs through itself** is two surfaces tangent at a point of the curve, so a
/// turn moves it off exactly as it moves a tangency off. It is refused by name
/// rather than answered, so it is skipped by [`named`] alongside the other
/// refusals.
#[test]
fn turning_a_quartic_pair_of_surfaces_names_the_same_kind_of_crossing() {
    the_same_kind_before_and_after(
        a_quartic_lattice_crossing().boxed(),
        MOST_QUARTIC_PAIRS_MUST_BE_COMPARED,
    );
}

/// The knife edge a ball and a pipe have, which [`is_on_a_knife_edge`] had no
/// arm for until this case was written.
///
/// A ball of radius one whose middle sits on the axis of a pipe of radius one
/// rests against the pipe all the way round a single circle. That circle is
/// where `r² − ρ²` is exactly zero, so it is the same kind of edge as a plane
/// resting on a ball: turning the pair rounds both radii, and whichever way the
/// last bits fall the answer changes. A sixty-fourth of a turn about `(1, 1, 1)`
/// makes the ball miss the pipe altogether; twenty sixty-fourths makes it cut
/// two circles.
///
/// **Both turned answers are honest**, which is why the pair has to be skipped
/// rather than fixed: the turned surfaces genuinely no longer rest on each
/// other. Without the arm, the property above draws this pair — the generator
/// places a ball anywhere and gives both it and a pipe a radius from one to
/// five — compares it, and fails.
#[test]
fn a_ball_resting_along_a_whole_circle_inside_a_pipe_is_a_knife_edge() {
    let resting = Crossing {
        first: Piece::Ball {
            centre: [0, 0, 0],
            radius: 1,
        },
        second: Piece::Pipe {
            along: [0, 0, 1],
            through: [0, 0, 0],
            radius: 1,
        },
        turn: None,
        elsewhere: Elsewhere::here(),
        hair: None,
    };
    let turned_by = |steps| Crossing {
        turn: Some(Turn {
            direction: [1, 1, 1],
            steps,
        }),
        ..resting
    };

    assert_eq!(
        kind_of(&named(&resting.surfaces()).expect("a ball resting inside a pipe is named")),
        ("curves", 1),
        "the ball touches the pipe along one whole circle"
    );
    assert_eq!(
        kind_of(&named(&turned_by(1).surfaces()).expect("the turned pair is named")),
        ("nothing", 0),
        "a sixty-fourth of a turn rounds the ball just clear of the pipe"
    );
    assert_eq!(
        kind_of(&named(&turned_by(20).surfaces()).expect("the turned pair is named")),
        ("curves", 2),
        "twenty sixty-fourths rounds it the other way and the circle splits in two"
    );

    assert!(
        is_on_a_knife_edge(&resting.surfaces()),
        "a pair whose kind two different turns change in two different directions is a knife \
         edge, and the property above compares it unless this says so"
    );
}

fn the_same_kind_before_and_after(family: BoxedStrategy<Crossing>, floor: f64) {
    let cases = soak::cases(CASES);
    let mut runner = TestRunner::new_with_rng(
        Config {
            cases,
            source_file: Some(file!()),
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
                "proptest-regressions",
            ))),
            ..Config::default()
        },
        soak::seeded(),
    );

    let compared = Cell::new(0usize);
    let on_a_knife_edge = Cell::new(0usize);
    let outcome = runner.run(&(family, a_turn()), |(crossing, turn)| {
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
        "{} of {cases} pairs compared, {} skipped for sitting on a knife edge",
        compared.get(),
        on_a_knife_edge.get()
    );
    assert!(
        compared.get() as f64 >= floor * f64::from(cases),
        "only {} of {cases} pairs were off a knife edge, so this property is barely asserting \
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
        Err(
            Error::Undecidable { .. }
            | Error::CannotNameWhichPieceOfACrossingThisIs
            | Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo,
        ) => Err(TestCaseError::reject("this pair could not be named")),
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
        // A ball whose middle is on a pipe's axis and whose radius is the pipe's
        // rests against the pipe along a whole circle. `r² − ρ²` is exactly zero
        // there, so a rounding of either radius either lifts the ball clear or
        // splits the circle in two —
        // `a_ball_resting_along_a_whole_circle_inside_a_pipe_is_a_knife_edge`
        // shows one turn doing each.
        [Surface::Sphere(ball), Surface::Cylinder(pipe)]
        | [Surface::Cylinder(pipe), Surface::Sphere(ball)] => {
            ball.radius() == pipe.radius()
                && (ball.centre() - pipe.axis().origin())
                    .cross(pipe.axis().direction())
                    .length_squared()
                    == 0.0
        }
        // Two pipes down one direction cross in straight lines and a lean turns
        // that into two closed loops, so this is the quartic family's version of
        // the plane-and-pipe surprise above.
        [Surface::Cylinder(one), Surface::Cylinder(other)] => {
            one.axis()
                .direction()
                .cross(other.axis().direction())
                .length_squared()
                == 0.0
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
