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
//!   other, two pipes of one width whose axes meet, whose walls touch at two
//!   points, or a thin pipe resting on the inside of a fat one, or a pipe on
//!   the inside of a wider ball, whose walls touch at one — where the
//!   discriminant is exactly zero; and
//! - **two directions exactly square or exactly parallel**: a plane along a
//!   pipe's axis, or two parallel planes. The pipe one is the surprise. A plane
//!   *not* parallel to an endless pipe always cuts it in an ellipse, however far
//!   away it is, so "parallel and well clear of it" answers `Nothing` and the
//!   smallest tilt turns that into an enormous ellipse. The knife edge there is
//!   not a near miss; it is the whole of the `Nothing` answer.
//!
//! A ball resting all the way round the inside of a pipe used to be a third
//! one, and §118.11 took it off the list: both its radii survive a turn
//! untouched, so the resting does too. [`is_on_a_knife_edge`] says what the arm
//! was and why it went, and
//! [`a_ball_resting_along_a_whole_circle_inside_a_pipe_answers_one_circle_however_it_is_turned`]
//! counts what the old rule answered instead.
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
//! Measured: **1486 of 2048 planar pairs compared and 564 skipped, 1560 of 2048
//! quartic pairs compared and 490 skipped**.
//! [`the_490_skipped_quartic_pairs_split_by_which_clause_skips_them`] pins
//! what the 490 are a mix of: 332 pairs down one direction, 10 two pipes of
//! one width whose axes meet (§347.9), 57 a thin pipe resting on the inside
//! of a fat one, 6 a ball resting on the far inside of a pipe, 31 a ball and
//! a pipe touching at one point, and 54 two
//! pipes of different widths resting against each other from the outside —
//! a tangency `is_on_a_knife_edge` has no clause for at all (§347), so those
//! 54 are skipped by the soak's own second check and never asked whether
//! turning them would change their answer.

#![forbid(unsafe_code)]

mod crossings;
mod soak;

use std::cell::Cell;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, TestRunner};
use rcad_geom::{Cylinder, Error, Sphere, Surface, SurfaceMeet, intersect};
use rcad_predicates::expansion::{
    difference_of_two_products, expansion_sign, expansion_sum, scale_expansion, two_diff,
    two_product,
};
use rcad_predicates::{Decision, Policy, ProvenanceLog};

use crossings::{Crossing, Elsewhere, Piece, Turn, a_lattice_crossing, a_quartic_lattice_crossing};

/// How many pairs this turns on the shipped corpus, which `soak::cases`
/// multiplies and `soak::seeded` reseeds.
const CASES: u32 = 2048;

/// How many pairs the runner may throw away per pair it keeps.
///
/// A pair neither side could name is rejected and redrawn, and proptest's own
/// ceiling on that is a flat 1024 however many cases are asked for — four per
/// case at its default corpus, and one per two cases at this file's. So a soak
/// at any real factor runs out of them and aborts on *"Too many global
/// rejects"*, which is a red run about the runner's bookkeeping rather than
/// about geometry. Four per case is proptest's own ratio, kept as the corpus
/// grows.
///
/// **It is not the vacuity guard.** That is the two floors below, which count
/// what was compared rather than what was thrown away.
const REJECTS_ALLOWED_PER_CASE: u32 = 4;

/// How many of the planar pairs have to survive the knife-edge skip.
///
/// Three fifths. Without a floor the property would pass just as green if every
/// pair were skipped, which is §14.7's vacuity trap.
///
/// It is not higher because of what the control family *is*: every direction on
/// the lattice is a coordinate axis, so two of them are exactly parallel or
/// exactly square far more often than two directions drawn at random would be.
/// Measured on the shipped corpus: 1486 of 2048 compared, 564 skipped.
const MOST_PLANAR_PAIRS_MUST_BE_COMPARED: f64 = 0.6;

/// How many of the quartic pairs have to survive it.
///
/// Half. The generator draws radii up to five and places them up to six apart in
/// whole numbers, so a lattice pipe pair is exactly tangent surprisingly often —
/// five apart with radii of two and three is one — and every one of those is a
/// knife edge a turn moves off (§29.5). Measured on the shipped corpus: 1560 of
/// 2048 compared, 490 skipped.
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
/// turn moves it off exactly as it moves a tangency off. Two pipes of one width
/// whose axes meet are named, as four half ellipses, and skipped by
/// [`is_on_a_knife_edge`] (see
/// [`two_pipes_of_one_width_whose_axes_meet_cross_in_one_curve_once_a_turn_parts_the_axes`]),
/// and so are a thin pipe resting on the inside of a fat one and a ball
/// resting on the far inside of a pipe, each named as two loops that
/// meet where the walls touch (see
/// [`a_thin_pipe_resting_inside_a_fat_one_crosses_it_in_one_loop_once_a_turn_moves_the_axes`]
/// and
/// [`a_ball_resting_on_the_far_inside_of_a_pipe_crosses_it_in_one_loop_once_a_turn_moves_it`]).
#[test]
fn turning_a_quartic_pair_of_surfaces_names_the_same_kind_of_crossing() {
    the_same_kind_before_and_after(
        a_quartic_lattice_crossing().boxed(),
        MOST_QUARTIC_PAIRS_MUST_BE_COMPARED,
    );
}

/// The 490 pairs the test above skips, split by which clause skips each one.
///
/// Runs the same soak, the same seed, the same 2048 cases, so this counts the
/// same 490 pairs rather than a different draw of them. Four of the six
/// buckets are [`is_on_a_knife_edge`]'s own clauses, the first three its
/// cylinder-cylinder ones in the order that function tries them, since `||`
/// in Rust stops at the first `true` and a pair that satisfies two clauses at
/// once is counted under whichever one the function would have stopped at:
///
/// - **down one direction** — the two axes are parallel, so the function
///   never reaches the other two clauses even when one of them also holds;
/// - **two pipes of one width whose axes meet** — the Steinmetz tangency;
/// - **a thin pipe resting inside a fat one** — the other tangency of two
///   pipes [`is_on_a_knife_edge`] names;
/// - **a ball resting on the far inside of a pipe** — the ball-against-pipe
///   clause, the one tangency of a ball and a pipe at a point of a curve they
///   cross in.
///
/// The other two buckets are pairs [`is_on_a_knife_edge`] says are on no
/// knife edge at all, skipped anyway on the separate check at the soak's
/// call site that catches a [`SurfaceMeet::TouchAtOnePoint`] [`named`]
/// returns before any knife-edge check of the *moved* pair runs. That is the
/// gap the soak's doc comment points at: [`is_on_a_knife_edge`] is asked of
/// the *still* surfaces only, so neither of these two kinds is ever asked
/// whether turning it would in fact change its answer.
///
/// - **a ball and a pipe touching at one point** — from outside, or from
///   inside a pipe wider than the ball, and named as that one point.
///   [`is_on_a_knife_edge`]'s ball-against-pipe clause asks only after a
///   ball wider than the pipe, whose walls touch at a point of a curve.
/// - **two pipes of different widths resting against each other from
///   outside** — the tangency [`is_on_a_knife_edge`] does not name at all.
///   Its thin-pipe clause asks whether the axes are `|r₁ − r₂|` apart, which
///   is the one way two *differently sized* pipes can touch from the
///   *inside*; it never asks whether they are `r₁ + r₂` apart, which is how
///   two pipes of any two widths touch resting against each other from the
///   *outside* — the same tangency [`a_crossing_a_hair_from_the_knife_edge`]
///   draws on purpose as "two pipes leaning". A pair in this bucket is
///   genuinely on a knife edge; [`is_on_a_knife_edge`] is missing the clause
///   for it. §347 is where this gap is recorded, and the fix is a follow-up
///   row rather than this one, which only has to count what the soak skips
///   today without silently trusting [`is_on_a_knife_edge`]'s own account of
///   why.
///
/// Measured on the shipped corpus: 332 down one direction, 10 two pipes of
/// one width whose axes meet, 57 a thin pipe resting inside a fat one, 6 a
/// ball resting on the far inside of a pipe, 31 a ball and a pipe touching at
/// one point, and 54 two pipes of different widths resting against each other
/// from outside — 490 in all, which is every pair
/// `turning_a_quartic_pair_of_surfaces_names_the_same_kind_of_crossing` skips.
/// The 6 were refused by [`named`] and drawn again until Q2649 named them, so
/// naming them moved the draws after them too.
#[test]
fn the_490_skipped_quartic_pairs_split_by_which_clause_skips_them() {
    let cases = soak::cases(CASES);
    let mut runner = TestRunner::new_with_rng(
        Config {
            cases,
            max_global_rejects: cases.saturating_mul(REJECTS_ALLOWED_PER_CASE),
            source_file: Some(file!()),
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
                "proptest-regressions",
            ))),
            ..Config::default()
        },
        soak::seeded(),
    );

    let compared = Cell::new(0usize);
    let down_one_direction = Cell::new(0usize);
    let two_pipes_whose_axes_meet = Cell::new(0usize);
    let a_thin_pipe_inside_a_fat_one = Cell::new(0usize);
    let a_ball_resting_on_the_far_inside_of_a_pipe = Cell::new(0usize);
    let a_ball_and_a_pipe_touching_at_one_point = Cell::new(0usize);
    let two_pipes_resting_from_outside_on_no_named_knife_edge = Cell::new(0usize);

    let outcome = runner.run(
        &(a_quartic_lattice_crossing().boxed(), a_turn()),
        |(crossing, turn)| {
            let still = crossing;
            let mut moved = crossing;
            moved.turn = Some(turn);
            let surfaces = still.surfaces();

            if is_on_a_knife_edge(&surfaces) {
                match &surfaces {
                    [Surface::Cylinder(one), Surface::Cylinder(other)] => {
                        if one
                            .axis()
                            .direction()
                            .cross(other.axis().direction())
                            .length_squared()
                            == 0.0
                        {
                            down_one_direction.set(down_one_direction.get() + 1);
                        } else if one.radius() == other.radius() {
                            two_pipes_whose_axes_meet.set(two_pipes_whose_axes_meet.get() + 1);
                        } else {
                            a_thin_pipe_inside_a_fat_one
                                .set(a_thin_pipe_inside_a_fat_one.get() + 1);
                        }
                    }
                    [Surface::Sphere(_), Surface::Cylinder(_)]
                    | [Surface::Cylinder(_), Surface::Sphere(_)] => {
                        a_ball_resting_on_the_far_inside_of_a_pipe
                            .set(a_ball_resting_on_the_far_inside_of_a_pipe.get() + 1);
                    }
                    other => panic!(
                        "the quartic family draws only ball-against-pipe and \
                         pipe-against-pipe: {other:?}"
                    ),
                }
                return Ok(());
            }
            let before = named(&surfaces)?;
            if matches!(before, SurfaceMeet::TouchAtOnePoint { .. }) {
                if let [Surface::Cylinder(_), Surface::Cylinder(_)] = surfaces {
                    two_pipes_resting_from_outside_on_no_named_knife_edge
                        .set(two_pipes_resting_from_outside_on_no_named_knife_edge.get() + 1);
                } else {
                    let ([Surface::Sphere(_), Surface::Cylinder(_)]
                    | [Surface::Cylinder(_), Surface::Sphere(_)]) = surfaces
                    else {
                        panic!(
                            "the quartic family draws only ball-against-pipe and \
                             pipe-against-pipe"
                        );
                    };
                    a_ball_and_a_pipe_touching_at_one_point
                        .set(a_ball_and_a_pipe_touching_at_one_point.get() + 1);
                }
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
        },
    );
    if let Err(why) = outcome {
        panic!("{why}");
    }

    let on_a_knife_edge = down_one_direction.get()
        + two_pipes_whose_axes_meet.get()
        + a_thin_pipe_inside_a_fat_one.get()
        + a_ball_resting_on_the_far_inside_of_a_pipe.get();
    let on_no_named_knife_edge = a_ball_and_a_pipe_touching_at_one_point.get()
        + two_pipes_resting_from_outside_on_no_named_knife_edge.get();
    let skipped = on_a_knife_edge + on_no_named_knife_edge;
    println!(
        "{} of {cases} quartic pairs compared, {skipped} skipped: {} down one \
         direction, {} two pipes of one width whose axes meet, {} a thin pipe \
         resting inside a fat one, {} a ball resting on the far inside of a \
         pipe, {} a ball and a pipe touching at one point, {} two pipes of \
         different widths resting against each other from outside",
        compared.get(),
        down_one_direction.get(),
        two_pipes_whose_axes_meet.get(),
        a_thin_pipe_inside_a_fat_one.get(),
        a_ball_resting_on_the_far_inside_of_a_pipe.get(),
        a_ball_and_a_pipe_touching_at_one_point.get(),
        two_pipes_resting_from_outside_on_no_named_knife_edge.get(),
    );

    assert_eq!(
        down_one_direction.get()
            + two_pipes_whose_axes_meet.get()
            + a_thin_pipe_inside_a_fat_one.get()
            + a_ball_resting_on_the_far_inside_of_a_pipe.get()
            + a_ball_and_a_pipe_touching_at_one_point.get()
            + two_pipes_resting_from_outside_on_no_named_knife_edge.get(),
        skipped,
        "every skip is counted in exactly one of the six buckets"
    );
    assert_eq!(
        compared.get(),
        1560,
        "matches the module doc's published count"
    );
    assert_eq!(skipped, 490, "matches the module doc's published count");
    assert_eq!(down_one_direction.get(), 332);
    assert_eq!(two_pipes_whose_axes_meet.get(), 10);
    assert_eq!(a_thin_pipe_inside_a_fat_one.get(), 57);
    assert_eq!(a_ball_resting_on_the_far_inside_of_a_pipe.get(), 6);
    assert_eq!(a_ball_and_a_pipe_touching_at_one_point.get(), 31);
    assert_eq!(
        two_pipes_resting_from_outside_on_no_named_knife_edge.get(),
        54,
        "a knife edge is_on_a_knife_edge has no clause for (§347)"
    );
}

/// Why the four half ellipses of two pipes of one width whose axes meet are a
/// knife edge and the one curve the same pair turned is named is not a wrong
/// answer.
///
/// The pair is the one the quartic property above first drew when the four
/// halves were named: pipes of radius 3 along `y` through `(6, 0, 0)` and up
/// `z` through `(6, 0, 1)`, whose axes meet at `(6, 0, 0)`, turned one step
/// about `y`. The turn rounds both stored axes, and the turned axes miss each
/// other. Two pipes of one width whose axes miss by any distance short of two
/// radii each bite into the other, and neither takes the other's whole
/// cross-section, so they cross in one closed curve; that is what is named.
#[test]
fn two_pipes_of_one_width_whose_axes_meet_cross_in_one_curve_once_a_turn_parts_the_axes() {
    let meeting = Crossing {
        first: Piece::Pipe {
            along: [0, 1, 0],
            through: [6, 0, 0],
            radius: 3,
        },
        second: Piece::Pipe {
            along: [0, 0, 1],
            through: [6, 0, 1],
            radius: 3,
        },
        turn: None,
        elsewhere: Elsewhere::here(),
        hair: None,
    };
    let turned = Crossing {
        turn: Some(Turn {
            direction: [0, 1, 0],
            steps: 1,
        }),
        ..meeting
    };

    let still = meeting.surfaces();
    assert_eq!(
        kind_of(&named(&still).expect("two pipes of one width whose axes meet are named")),
        ("curves", 4),
        "two ellipses, each cut in two where the walls touch"
    );
    assert!(is_on_a_knife_edge(&still), "and that is a knife edge");

    let moved = turned.surfaces();
    let [Surface::Cylinder(one), Surface::Cylinder(other)] = moved else {
        panic!("this pair is two pipes");
    };
    assert_eq!(
        one.radius(),
        other.radius(),
        "a turn leaves both radii alone"
    );
    assert!(
        !the_axes_meet_exactly(&one, &other),
        "the turn rounds the two axes apart"
    );
    assert_eq!(
        kind_of(&named(&moved).expect("the turned pair is named")),
        ("curves", 1),
        "two pipes of one width whose axes miss cross in one curve"
    );
}

/// Why the two loops of a thin pipe resting on the inside of a fat one are a
/// knife edge, and the one loop the same pair turned is named is not a wrong
/// answer.
///
/// The pair is the one the quartic property above first drew once the two
/// loops were named (Q2636): a pipe of radius 3 up `z` through `(−2, 0, 0)`
/// and a pipe of radius 2 along `y` through `(−3, 0, 0)`, whose axes are one
/// apart, which is what the widths differ by. The thin pipe touches the fat
/// one from inside at `(−5, 0, 0)`, and its two loops meet there. The turn
/// rounds both axes, and the turned axes are a few last bits nearer or further
/// apart than one. Nearer, the two loops part; further, the thin pipe pokes
/// out through the fat one's wall and the two cross in one loop, and that is
/// what this turn gives.
#[test]
fn a_thin_pipe_resting_inside_a_fat_one_crosses_it_in_one_loop_once_a_turn_moves_the_axes() {
    let resting = Crossing {
        first: Piece::Pipe {
            along: [0, 0, 1],
            through: [-2, 0, 0],
            radius: 3,
        },
        second: Piece::Pipe {
            along: [0, 1, 0],
            through: [-3, 0, 0],
            radius: 2,
        },
        turn: None,
        elsewhere: Elsewhere::here(),
        hair: None,
    };
    let turned = Crossing {
        turn: Some(Turn {
            direction: [0, -2, -1],
            steps: 3,
        }),
        ..resting
    };

    let still = resting.surfaces();
    assert_eq!(
        kind_of(&named(&still).expect("a thin pipe resting inside a fat one is named")),
        ("curves", 2),
        "two loops, which meet where the walls touch"
    );
    assert!(is_on_a_knife_edge(&still), "and that is a knife edge");

    let moved = turned.surfaces();
    let [Surface::Cylinder(one), Surface::Cylinder(other)] = moved else {
        panic!("this pair is two pipes");
    };
    assert_eq!(
        (one.radius(), other.radius()),
        (3.0, 2.0),
        "a turn leaves both radii alone"
    );
    assert_eq!(
        how_far_apart_the_axes_are_against_what_the_widths_differ_by(&one, &other),
        Decision::Pos,
        "the turn rounds the two axes further apart than the widths differ"
    );
    assert_eq!(
        kind_of(&named(&moved).expect("the turned pair is named")),
        ("curves", 1),
        "the turned axes are further apart than the widths differ, so the thin pipe pokes out \
         and the two cross in one loop"
    );
}

/// Why the two loops of a ball resting on the far inside of a pipe are a knife
/// edge, and the one loop the same pair turned is named is not a wrong answer.
///
/// The pair is the one the quartic property above first drew once the two
/// loops were named (Q2649): a ball of radius 5 round `(0, −2, −2)` and a pipe
/// of radius 4 along `x` through `(0, −1, −2)`, whose axis is one from the
/// ball's centre, which is what the radii differ by. The pipe's wall touches
/// the ball's from inside at `(0, 3, −2)`, and the two loops meet there. The
/// turn rounds the centre and the axis, and the turned centre is a few last
/// bits nearer or further than one from the turned axis. Nearer, the ball
/// swallows a whole cross-section and the two loops part; further, that
/// cross-section pokes out through the ball and the two cross in one loop, and
/// that is what this turn gives.
#[test]
fn a_ball_resting_on_the_far_inside_of_a_pipe_crosses_it_in_one_loop_once_a_turn_moves_it() {
    let resting = Crossing {
        first: Piece::Ball {
            centre: [0, -2, -2],
            radius: 5,
        },
        second: Piece::Pipe {
            along: [1, 0, 0],
            through: [0, -1, -2],
            radius: 4,
        },
        turn: None,
        elsewhere: Elsewhere::here(),
        hair: None,
    };
    let turned = Crossing {
        turn: Some(Turn {
            direction: [1, 0, 0],
            steps: 21,
        }),
        ..resting
    };

    let still = resting.surfaces();
    assert_eq!(
        kind_of(&named(&still).expect("a ball resting on the far inside of a pipe is named")),
        ("curves", 2),
        "two loops, which meet where the walls touch"
    );
    assert!(is_on_a_knife_edge(&still), "and that is a knife edge");

    let moved = turned.surfaces();
    let [Surface::Sphere(ball), Surface::Cylinder(pipe)] = moved else {
        panic!("this pair is a ball and a pipe, in that order");
    };
    assert_eq!(
        (ball.radius(), pipe.radius()),
        (5.0, 4.0),
        "a turn leaves both radii alone"
    );
    assert_eq!(
        how_far_off_the_axis_the_centre_is_against_what_the_radii_differ_by(&ball, &pipe),
        Decision::Pos,
        "the turn rounds the centre further off the axis than the radii differ"
    );
    assert!(
        !is_on_a_knife_edge(&moved),
        "so the turned pair is on no knife edge"
    );
    assert_eq!(
        kind_of(&named(&moved).expect("the turned pair is named")),
        ("curves", 1),
        "the turned centre is further off the axis than the radii differ, so the pipe's \
         cross-section pokes out through the ball and the two cross in one loop"
    );
}

/// The pair that used to be this file's third knife edge, and is not one any
/// more.
///
/// A ball of radius one whose middle sits **on the origin the pipe's axis runs
/// through** rests against a pipe of radius one all the way round a single
/// circle. Turning the pair leaves both radii alone — a radius is a scalar and a
/// rotation does not touch it — and carries the ball's middle and the axis'
/// origin to the same place, because they are the same place. So the resting is
/// exact after every turn, and one circle is the truthful answer to all
/// sixty-four of them.
///
/// **It did not used to be.** Before §118.11 a pipe was every point `r/‖w‖` from
/// its axis, so the question "does this ball rest inside this pipe" came out as
/// `R²‖w‖² − r²`, which for `R = r = 1` is the sign of `‖w‖² − 1` — the stored
/// direction's own rounding and nothing else about the geometry, which is why
/// the assertion this test used to carry read `nothing` at one sixty-fourth and
/// `two circles` at twenty.
///
/// This counts that sign over the sixty-three turns below, in exact arithmetic
/// because the answer is a handful of last bits from zero. **Measured: two
/// circles 24 times, nothing at all 39 times, and the truthful answer not
/// once.** The two turns the old assertion named are pinned inside the loop, so
/// what is counted is the rule that really ran and not a rule that resembles it.
#[test]
fn a_ball_resting_along_a_whole_circle_inside_a_pipe_answers_one_circle_however_it_is_turned() {
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

    let mut the_old_rule_said_two_circles = 0;
    let mut the_old_rule_said_nothing = 0;
    let mut the_old_rule_agreed = 0;
    for steps in 1..64 {
        let surfaces = turned_by(steps).surfaces();
        assert_eq!(
            kind_of(&named(&surfaces).expect("the turned pair is named")),
            ("curves", 1),
            "a turn moves the resting circle and does not undo the resting, so every one of \
             these is still one circle"
        );

        let [Surface::Sphere(ball), Surface::Cylinder(pipe)] = surfaces else {
            panic!("this pair is a ball and a pipe, in that order");
        };
        assert_eq!(ball.radius(), 1.0, "a rotation does not touch a radius");
        assert_eq!(pipe.radius(), 1.0, "nor this one");
        assert_eq!(
            (ball.centre() - pipe.axis().origin())
                .cross(pipe.axis().direction())
                .length_squared(),
            0.0,
            "the ball's middle and the axis' origin are the same place, so a rotation carries \
             them to the same place"
        );

        let old = how_a_stored_directions_squared_length_compares_with_one(
            pipe.axis().direction().components(),
        );
        // Two turns that witness the old rule going each way, pinning the rule
        // rebuilt here to the one that actually ran. The second was twenty
        // sixty-fourths until §120's correctly rounded normalize moved every
        // turned axis; twenty reads `nothing` now and two reads `two circles`.
        if steps == 1 {
            assert_eq!(
                old,
                Decision::Neg,
                "the old test read `nothing` at one sixty-fourth"
            );
        }
        if steps == 2 {
            assert_eq!(old, Decision::Pos, "and `two circles` at two sixty-fourths");
        }
        if old == Decision::Pos {
            the_old_rule_said_two_circles += 1;
        } else if old == Decision::Neg {
            the_old_rule_said_nothing += 1;
        } else {
            assert_eq!(old, Decision::Zero, "a sum of exact squares has a sign");
            the_old_rule_agreed += 1;
        }
    }

    println!(
        "of 63 turns of one resting ball, the pre-§118.11 rule said two circles for \
         {the_old_rule_said_two_circles}, nothing at all for {the_old_rule_said_nothing}, and \
         one circle for {the_old_rule_agreed}"
    );
    assert!(
        the_old_rule_said_two_circles > 0 && the_old_rule_said_nothing > 0,
        "this test is a demonstration that the old answer was a rounding rather than the \
         geometry, and it demonstrates nothing unless that rounding went both ways"
    );
    assert_eq!(
        the_old_rule_agreed, 0,
        "and the sharper half of it: of sixty-three turns of one resting ball the old rule \
         reached the truthful answer not once"
    );

    assert!(
        !is_on_a_knife_edge(&resting.surfaces()),
        "a pair every turn above answers the same way is not a knife edge, and the properties \
         above skip it if this says it is"
    );
}

fn the_same_kind_before_and_after(family: BoxedStrategy<Crossing>, floor: f64) {
    let cases = soak::cases(CASES);
    let mut runner = TestRunner::new_with_rng(
        Config {
            cases,
            max_global_rejects: cases.saturating_mul(REJECTS_ALLOWED_PER_CASE),
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
///
/// # The arm §118.11 deleted
///
/// A ball resting all the way round the inside of a pipe exactly as wide as it
/// is used to be a third knife edge here, and it is not one any more. A turn
/// leaves both radii alone, `R² − r²` is exactly zero after it as well as
/// before, and a ball no wider than the pipe can never swallow a whole
/// cross-section of it however far off the axis a turn nudges its middle. The
/// arm was there because the old question carried the stored direction's
/// squared length on the ball's side, and `‖w‖²R² − r²` is that direction's
/// rounding wearing a discriminant's clothes.
///
/// A ball **wider** than the pipe resting on its far inside is a knife edge,
/// and has its own arm: there the centre's distance off the axis, which a turn
/// does round, is the whole of the question.
fn is_on_a_knife_edge(surfaces: &[Surface; 2]) -> bool {
    match surfaces {
        [Surface::Plane(flat), Surface::Cylinder(pipe)]
        | [Surface::Cylinder(pipe), Surface::Plane(flat)] => {
            flat.normal().dot(pipe.axis().direction()) == 0.0
        }
        [Surface::Plane(one), Surface::Plane(other)] => {
            one.normal().cross(other.normal()).length_squared() == 0.0
        }
        // Two pipes down one direction cross in straight lines and a lean turns
        // that into two closed loops, so this is the quartic family's version of
        // the plane-and-pipe surprise above.
        [Surface::Cylinder(one), Surface::Cylinder(other)] => {
            let down_one_direction = one
                .axis()
                .direction()
                .cross(other.axis().direction())
                .length_squared()
                == 0.0;
            down_one_direction
                || (one.radius() == other.radius() && the_axes_meet_exactly(one, other))
                || (one.radius() != other.radius()
                    && how_far_apart_the_axes_are_against_what_the_widths_differ_by(one, other)
                        == Decision::Zero)
        }
        [Surface::Sphere(ball), Surface::Cylinder(pipe)]
        | [Surface::Cylinder(pipe), Surface::Sphere(ball)] => {
            ball.radius() > pipe.radius()
                && how_far_off_the_axis_the_centre_is_against_what_the_radii_differ_by(ball, pipe)
                    == Decision::Zero
        }
        _ => false,
    }
}

/// Whether the two stored axes meet, asked exactly: `(o₂ − o₁)·(d₁ × d₂)` is
/// nothing.
///
/// **Two pipes of one width whose axes meet are a tangency**: their walls
/// touch at the two places the four half ellipses meet. The distance between
/// the axes is the whole of the difference between that and one curve, and a
/// turn rounds both axes, so a turned pair's distance is a few last bits from
/// nothing on one side or the other — and on either side it is one curve,
/// because two pipes of one width whose axes miss each bite into the other
/// and neither takes the other's whole cross-section. So this is asked
/// exactly, and of nothing wider than the pair that names four halves.
///
/// # Panics
///
/// If a product leaves the range of `f64`, which no pair drawn here comes near.
fn the_axes_meet_exactly(one: &Cylinder, other: &Cylinder) -> bool {
    let (gap, square) = the_gap_and_the_square_to_both_axes(one, other);
    the_sign_of(&dot_of(&gap, &square)) == Decision::Zero
}

/// How far apart the two stored axes are against what the two widths differ
/// by, asked exactly: the sign of `((o₂ − o₁)·(d₁ × d₂))² − (r₁ − r₂)²|d₁ × d₂|²`,
/// which is `δ² − (r₁ − r₂)²` times `|d₁ × d₂|²`, for `δ` how far apart the
/// axes are.
///
/// **A thin pipe resting on the inside of a fat one is a tangency**: where `δ`
/// is what the widths differ by, the walls touch at one point, and the thin
/// pipe's two loops meet there. A turn rounds both axes, so a turned pair's
/// `δ` is a few last bits from that on one side or the other: nearer, the two
/// loops part, and further, the thin pipe pokes out through the fat one's wall
/// and the two cross in one loop. So this is asked exactly, as
/// [`the_axes_meet_exactly`] asks the same of two pipes of one width, for
/// which `r₁ − r₂` is nothing.
///
/// # Panics
///
/// If a product leaves the range of `f64`, which no pair drawn here comes near.
fn how_far_apart_the_axes_are_against_what_the_widths_differ_by(
    one: &Cylinder,
    other: &Cylinder,
) -> Decision {
    let (gap, square) = the_gap_and_the_square_to_both_axes(one, other);
    let reach = dot_of(&gap, &square);
    let (difference, roundoff) = two_diff(one.radius(), other.radius());
    let narrowing = [roundoff, difference];
    let room = product_of(
        &product_of(&narrowing, &narrowing),
        &dot_of(&square, &square),
    );
    let less_room = room.iter().map(|part| -part).collect::<Vec<_>>();
    the_sign_of(&sum_of(&product_of(&reach, &reach), &less_room))
}

/// How far the ball's centre is off the pipe's stored axis against what the
/// two radii differ by, asked exactly: the sign of
/// `‖(c − o) × w‖² − (R − r)²‖w‖²`, which is `a² − (R − r)²` times `‖w‖²`, for
/// `a` how far off the axis the centre is.
///
/// **A ball resting on the far inside of a pipe is a tangency**: where the
/// ball is the wider and `a` is `R − r`, the pipe's wall touches the ball's at
/// the point of the pipe across its axis from the centre, and the two loops the
/// ball leaves meet there. A turn rounds the centre and the axis, so a turned
/// pair's `a` is a few last bits from that on one side or the other: nearer,
/// the ball swallows a whole cross-section of the pipe and the two loops part,
/// and further, that cross-section pokes out through the ball and the two
/// cross in one loop. So this is asked exactly, as
/// [`how_far_apart_the_axes_are_against_what_the_widths_differ_by`] asks the
/// same of two pipes. It is `intersect`'s own `V < 0, Δ = 0`: `Δ = 0` with
/// `V < 0` is `√A = (R − r)√W`.
///
/// # Panics
///
/// If a product leaves the range of `f64`, which no pair drawn here comes near.
fn how_far_off_the_axis_the_centre_is_against_what_the_radii_differ_by(
    ball: &Sphere,
    pipe: &Cylinder,
) -> Decision {
    let centre = ball.centre().coordinates();
    let origin = pipe.axis().origin().coordinates();
    let along = pipe.axis().direction().components();
    let gap = [0, 1, 2].map(|axis| {
        let (gap, roundoff) = two_diff(centre[axis], origin[axis]);
        vec![roundoff, gap]
    });
    let square_to_the_axis = [0, 1, 2].map(|axis| {
        let (next, last) = ((axis + 1) % 3, (axis + 2) % 3);
        sum_of(
            &product_of(&gap[next], &[along[last]]),
            &negated(&product_of(&gap[last], &[along[next]])),
        )
    });
    let stored = along.map(|step| vec![step]);
    let (difference, roundoff) = two_diff(ball.radius(), pipe.radius());
    let narrowing = [roundoff, difference];
    let room = product_of(
        &product_of(&narrowing, &narrowing),
        &dot_of(&stored, &stored),
    );
    the_sign_of(&sum_of(
        &dot_of(&square_to_the_axis, &square_to_the_axis),
        &negated(&room),
    ))
}

fn negated(exact: &[f64]) -> Vec<f64> {
    exact.iter().map(|part| -part).collect()
}

/// `o₂ − o₁`, from one stored axis's origin to the other's, and `d₁ × d₂`,
/// square to both, each coordinate an exact expansion.
fn the_gap_and_the_square_to_both_axes(
    one: &Cylinder,
    other: &Cylinder,
) -> ([Vec<f64>; 3], [Vec<f64>; 3]) {
    let [from, to] = [one, other].map(|pipe| pipe.axis().origin().coordinates());
    let [a, b] = [one, other].map(|pipe| pipe.axis().direction().components());
    let gap = [0, 1, 2].map(|axis| {
        let (gap, roundoff) = two_diff(to[axis], from[axis]);
        vec![roundoff, gap]
    });
    let square = [0, 1, 2].map(|axis| {
        let (next, last) = ((axis + 1) % 3, (axis + 2) % 3);
        let (across, used) = difference_of_two_products(a[next], b[last], a[last], b[next])
            .expect("a stored direction's products are exact");
        across[..used].to_vec()
    });
    (gap, square)
}

fn dot_of(one: &[Vec<f64>; 3], other: &[Vec<f64>; 3]) -> Vec<f64> {
    (0..3).fold(Vec::new(), |total, axis| {
        sum_of(&total, &product_of(&one[axis], &other[axis]))
    })
}

fn product_of(one: &[f64], other: &[f64]) -> Vec<f64> {
    other.iter().fold(Vec::new(), |total, &part| {
        let mut scaled = vec![0.0; 2 * one.len()];
        let used = scale_expansion(one, part, &mut scaled);
        sum_of(&total, &scaled[..used])
    })
}

fn sum_of(one: &[f64], other: &[f64]) -> Vec<f64> {
    let mut summed = vec![0.0; one.len() + other.len()];
    let used = expansion_sum(one, other, &mut summed);
    summed.truncate(used);
    summed
}

fn the_sign_of(exact: &[f64]) -> Decision {
    match expansion_sign(exact) {
        sign @ (Decision::Pos | Decision::Zero | Decision::Neg) => sign,
        Decision::Undecidable { .. } | _ => panic!("a sum of exact products has a sign"),
    }
}

/// The sign of `w·w − 1`, worked out in exact arithmetic.
///
/// This is the whole of the rule a ball-and-pipe crossing used to run on when
/// both radii are one: `dot(w, w)·R² − r²` with `R = r = 1`. It has to be exact
/// to be the same rule — the answer is a handful of last bits from zero, and
/// `Vector::length_squared` rounds three times on the way there and says `Zero`
/// where the truth is either side of it.
fn how_a_stored_directions_squared_length_compares_with_one(along: [f64; 3]) -> Decision {
    let squares = along.map(|step| {
        let (product, roundoff) = two_product(step, step);
        [roundoff, product]
    });
    let mut two_of_them = [0.0; 4];
    let used = expansion_sum(&squares[0], &squares[1], &mut two_of_them);
    let mut all_three = [0.0; 6];
    let used = expansion_sum(&two_of_them[..used], &squares[2], &mut all_three);
    let mut less_one = [0.0; 7];
    let used = expansion_sum(&all_three[..used], &[-1.0], &mut less_one);
    expansion_sign(&less_one[..used])
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
