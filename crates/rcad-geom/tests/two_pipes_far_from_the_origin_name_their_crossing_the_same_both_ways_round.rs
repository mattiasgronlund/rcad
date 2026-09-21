//! §181's crossing that disagrees with itself: §140's pair — a thin pipe lying
//! along the skin of a pipe 153 decades fatter — moved away from the origin.
//!
//! The night of 2026-09-20 drew four of these and
//! `naming_a_pair_does_not_depend_on_which_way_round_it_is_asked` caught them.
//! Asking the fat pipe first names two straight runs; asking the thin pipe
//! first refuses with `CannotNameWhichPieceOfACrossingThisIs`.
//!
//! # Why the answer exists
//!
//! Both axes run along `x`, so the question is two circles in `(y, z)`: one of
//! radius `R1`, one of radius `R2` whose centre sits `R1` away in `y` and `Δz`
//! along the axis. The true axis separation is `√(R1² + Δz²) = R1 + Δz²/(2·R1)`,
//! and that excess is `1.4e-111` against `R2 = 1.984375`. So the thin pipe cuts
//! the fat pipe's skin in two endless straight lines, by 111 decades — not
//! tangentially and not at a knife edge.
//!
//! # What is different from §140, and it is not a radius
//!
//! §140's pair sits at the origin, so its picking plane passes through the
//! origin and `normal · anchor` is exactly `0` — an offset with no rounding to
//! lose. This pair sits `3.7e21` out, where the offset is `1.87e175` and one
//! ulp of it places the plane on a grid of `6.9e5`, while the two lines it has
//! to separate stand `3.97` apart. `docs/decisions.md` §181.
//!
//! # Why it now answers, and what the answer is checked against
//!
//! §182 made both orders agree by making both **refuse**, and this file used to
//! assert only that they agree, because the order that answered then was wrong:
//! its picking plane held both lines. §183 measured that a plane which does
//! separate them exists here — `z = 41581179`, one of the three normals `f64`
//! can hold exactly square to the axis, through the thin pipe's stored axis —
//! and §184 reaches it by trying each such plane in turn and proving the
//! separation rather than assuming it.
//!
//! So the second test asserts the answer, and it does not stop at counting
//! runs: a count of two is what the wrong answer §182 removed also gave. Each
//! run's picker is asked which side of it two points on the true lines sit, and
//! each run has to hold exactly one of them, a different one from the other run.
//! The two points are the thin pipe's axis moved `±R2` along `z`. The true lines
//! sit within about `1.4e-111` of them — the excess worked out above — while
//! the two points stand `2·R2` apart, so a plane that puts them on opposite
//! sides puts the lines on opposite sides too.
//!
//! The agreement test stays: it is the invariant the fuzzer caught, and it holds
//! whichever way a later change moves the answer.

#![forbid(unsafe_code)]

mod fuzzing;

use fuzzing::{bytes_for, two_surfaces_from};
use rcad_geom::{EdgeRun, Picker, SurfaceMeet, intersect};
use rcad_predicates::{Decision, Policy, Point as PredicatePoint, ProvenanceLog};

/// The fat pipe's radius, and the thin pipe's offset off the fat axis.
const R1: f64 = 5.047645337156084e153;

/// The thin pipe's radius. `127/64`, so `R2²` is exact.
const R2: f64 = 1.984375;

/// How far the thin pipe's axis sits along the fat axis' own direction.
const Z: f64 = 41581179.0;

/// How far the fat pipe's own axis sits from the origin along `z`.
const FAR: f64 = 3.705489715806406e21;

/// The fourteen numbers the fuzzer drew, as the decoder reads them: a place and
/// a direction and a radius apiece.
const SEED: [f64; 14] = [
    100663296.0,
    0.0,
    FAR,
    127.0,
    0.0,
    0.0,
    R1,
    R1,
    R1,
    Z,
    167772415.0,
    0.0,
    0.0,
    R2,
];

/// Two side-by-side pipes far from the origin name the same crossing whichever
/// is asked about first.
///
/// `docs/decisions.md` §181.
#[test]
fn a_pipe_on_a_fat_pipes_skin_is_named_the_same_way_round_far_from_the_origin() {
    let pair = two_surfaces_from(&bytes_for([2, 2], &SEED)).expect("two pipes");
    let named = [[0, 1], [1, 0]].map(|[first, second]| {
        how_it_was_named(intersect(
            &pair[first],
            &pair[second],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ))
    });
    assert_eq!(
        named[0], named[1],
        "the same pair of pipes was named differently by which one was asked about first"
    );
}

/// Two side-by-side pipes far from the origin cross in two straight lines
/// whichever is asked about first, and each run holds one of them.
///
/// `docs/decisions.md` §184.
#[test]
fn a_pipe_on_a_fat_pipes_skin_far_from_the_origin_crosses_it_in_two_lines_either_way_round() {
    let pair = two_surfaces_from(&bytes_for([2, 2], &SEED)).expect("two pipes");
    let on_the_lines = [Z + R2, Z - R2].map(|z| PredicatePoint::from([0.0, R1, z]));

    for [first, second] in [[0, 1], [1, 0]] {
        let met = intersect(
            &pair[first],
            &pair[second],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let Ok(SurfaceMeet::Curves(runs)) = met else {
            panic!("asking pipe {first} first did not name the crossing: {met:?}");
        };
        assert_eq!(runs.len(), 2, "asking pipe {first} first");

        let held: Vec<[Decision; 2]> = runs
            .iter()
            .map(|run| {
                let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
                    panic!("one of two lines is a piece of the crossing: {run:?}");
                };
                let [Picker::InFrontOf(half_space)] = picked_by.as_slice() else {
                    panic!("one flat picker per line: {picked_by:?}");
                };
                on_the_lines.map(|point| half_space.side_of_point(point))
            })
            .collect();
        let [upper, lower] = [
            [Decision::Pos, Decision::Neg],
            [Decision::Neg, Decision::Pos],
        ];
        assert!(
            held == [upper, lower] || held == [lower, upper],
            "asking pipe {first} first: each run must hold exactly one line, and not the same \
             one — got {held:?}"
        );
    }
}

/// What a crossing was named, coarsely enough that the two orders' answers are
/// comparable.
///
/// The two orders build the same plane with negated normals, so the runs
/// themselves differ bit for bit even where the two orders agree completely.
fn how_it_was_named(met: rcad_geom::Result<SurfaceMeet>) -> String {
    match met {
        Ok(SurfaceMeet::Curves(runs)) => format!("{} run(s)", runs.len()),
        Ok(SurfaceMeet::Nothing) => "nothing".to_owned(),
        Ok(SurfaceMeet::TouchAtOnePoint { .. }) => "one touching point".to_owned(),
        Ok(SurfaceMeet::SameSurface) => "the same surface".to_owned(),
        Ok(_) => "an arm that did not exist when this test was written".to_owned(),
        Err(why) => format!("refused: {why:?}"),
    }
}
