//! Which of its four ways out the walked-point oracle takes when it comes back
//! `points: 0`, and whether that zero is its fault, the walk's, or nobody's.
//!
//! The overnight runs of 2026-09-17 and 2026-09-18 left five slow units on
//! `where_two_surfaces_cross`. All five reach `SurfaceMeet::Curves` with two
//! runs; one comes back `Asked { points: 10, .. }` and four come back
//! `Asked { points: 0, .. }`. A zero there had four possible meanings, because
//! `every_point_of_every_crossing_is_on_both_surfaces` counts one number and the
//! loop that feeds it can end four ways: the coarse sizing walk refusing, the
//! size it hands back not being a length, the fine walk refusing, or every
//! walked point losing the box around it in `somewhere_within`. The last of
//! those is the quiet one — `checked += 1` sits after that `continue`, so a
//! point whose box collapses is invisible to the counter.
//!
//! **It was the first exit, on every run of every one of the four, and never
//! any of the other three.** `how_every_crossing_was_walked` says so per run
//! instead of per pair, and the tests below pin what it says. Since Q1598 that
//! holds of the two hair-thin pairs only: the two tilted ones walk now, and
//! every point they walk is checked.
//!
//! # What that makes the zero
//!
//! Two different things, and the refusal the sizing walk gives says which:
//!
//! **A defect in the walk, for `1bd43cf4` and `d8b562bb`, closed by Q1598.**
//! Both refused `TooManyPointsAlongOneCrossing { most: 1048576 }`, and both are
//! the pair behind the seeded `7e62ebe6` with a y-component of `3.85e-122` or
//! `1.15e-129` put on the thin pipe's axis. That was §16.10's *gap*: the answer
//! exists and the arithmetic reaches it. Every ordinary ask does —
//! [`a_hair_of_tilt_does_not_stop_these_pipes_being_walked_at_an_ordinary_ask`]
//! walks the very same runs at `0.01`, `1.0`, `1e10` and `1e16` and gets points
//! back every time. What did not reach it was the harness's *sizing* ask of
//! `f64::MAX / 4.0`, where the march kept a first trial that had settled back
//! onto its start and then crawled from there to the point cap. The tilt was
//! not a second bug. It is what took this pair from *just* surviving that ask
//! to not surviving it, which the untilted `7e62ebe6` showed by walking ten
//! points through the same code. Q1598 tries such a trial again shorter rather
//! than keeping it (§347.9), and now both runs of both pairs walk at the
//! harness's fine ask, every point they draw checked against both pipes in a
//! box reaching `8.94e-8` each way, round a loop of radius `1.375`:
//! [`the_pipe_tilted_by_a_hundred_and_twenty_two_orders_has_every_walked_point_checked`]
//! and its sibling at a hundred and twenty-nine orders pin it.
//!
//! **A refusal the kernel is right to make, for `5dddf2a9` and `3a4a8231`.**
//! Their loops are `4.34e-19` across where one step of the `f64` grid at those
//! coordinates is `5.82e-11`, so no `f64` point sits on both pipes and there is
//! no answer for any ask to reach. That is §16.10's *limit*, and
//! `what_makes_a_huge_pipe_against_a_thin_one_slow_to_walk.rs` demonstrates the
//! limit half rather than arguing it: nothing changes but the thin radius, and
//! at `1e-7` the same call walks the pair. Here the oracle skipping is correct.
//! [`the_hair_thin_pipes_are_refused_at_the_finest_ask_too`] pins that it is not
//! the size of the harness's ask that turns them away.
//!
//! # So the oracle is not what is wrong
//!
//! The counter is honest. The fourth exit, the one that could have made a zero
//! mean "checked nothing while looking like it walked", never fires on any of
//! the five: on the one pair that does produce points every point keeps its box,
//! and [`every_point_this_oracle_walks_keeps_the_box_around_it`] pins that.
//!
//! # The seeded `5dddf2a9`, whose walked-point oracle checks nothing
//!
//! It stays, and "it asserts nothing" would be false. Replayed from the corpus
//! it asserts three things: that `intersect` names the pair `Curves` rather than
//! refusing, that naming it the other way round agrees (`orders: 1`), and that
//! writing it at another power of two does not change it (`scalings: 2`). What
//! it does not assert is that any point sits on both pipes — and that is not an
//! assertion anyone can make about this pair, because no such point exists. A
//! seed that pins a refusal is pinning something; dropping it would lose the one
//! corpus member that drives the tangent refusal at both a fine ask and a coarse
//! one (until Q1598 the coarse one marched to the point cap instead), and §91
//! says an artifact not in the corpus is lost.
//!
//! # How every number here was measured
//!
//! `mise exec -- just t --release --no-capture -E 'binary(...)' --run-ignored
//! all`, in a release build, on a throwaway version of this file. The five pairs
//! were decoded with `mise exec -- cargo run --release -q -p rcad-fuzz-explain`,
//! and the number lists below are the `rebuild:` lines it printed, so what runs
//! here is what the fuzzer drew and not a retyping of it.

#![forbid(unsafe_code)]

use fuzzing::{WhereARunWent, bytes_for, how_every_crossing_was_walked, two_surfaces_from};
use rcad_geom::{
    Deflection, Error, NamedRun, POINT_ALONG_A_CROSSING_ROUNDOFF, Surface, SurfaceMeet, intersect,
    walk,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// **The fourth exit never fires**, which is what clears the oracle itself.
///
/// `7e62ebe6` is the one of the five that walks, and every point it walks keeps
/// the box `somewhere_within` puts around it. Had `drawn` and `checked` differed
/// here, a zero elsewhere could have meant points walked and silently dropped;
/// they do not, so it cannot.
#[test]
fn every_point_this_oracle_walks_keeps_the_box_around_it() {
    let went = how_every_crossing_was_walked(the_big_pipe_and_the_thin_one_with_no_tilt());
    assert_eq!(went.len(), 2, "a pipe punched through another leaves two");
    for run in &went {
        let WhereARunWent::Walked { drawn, checked } = run else {
            panic!("this pair walks; it went {run:?}");
        };
        assert!(*drawn > 0, "the walk drew no points at all");
        assert_eq!(
            drawn,
            checked,
            "{} of {drawn} walked points lost their box and went uncounted",
            drawn - checked
        );
    }
}

/// **A hair of tilt is not a second bug**, and this is the §16.10 demonstration
/// that the answer exists for the two night artifacts the harness gives up on.
///
/// `1bd43cf4` and `d8b562bb` differ from the pair above in the thin pipe's axis
/// y-component and in nothing else, by `3.85e-122` and `1.15e-129` — far too
/// little to move where a pipe of radius `1.68e8` and one of radius `1.375`
/// cross. Every ordinary ask walks their runs, and every point comes back far
/// closer to the crossing than the thin pipe is wide. Only the harness's
/// `f64::MAX / 4.0` sizing ask does not, which makes the zero a gap in the walk
/// rather than a property of the pair.
#[test]
fn a_hair_of_tilt_does_not_stop_these_pipes_being_walked_at_an_ordinary_ask() {
    for pair in [
        the_big_pipe_and_the_thin_one_tilted_by_3e_122(),
        the_big_pipe_and_the_thin_one_tilted_by_1e_129(),
    ] {
        let runs = the_runs_of(&pair);
        assert_eq!(runs.len(), 2, "a pipe punched through another leaves two");
        for run in &runs {
            for ask in [0.01, 1.0, 1e10, 1e16] {
                let (drawn, off) = walk_it(&pair, run, ask)
                    .unwrap_or_else(|why| panic!("asked for {ask:e} of room and got {why:?}"));
                assert!(drawn > 0, "asking for {ask:e} of room drew no points");
                assert!(
                    off < 1.375,
                    "the points may sit {off:e} off the loop, which is wider than the thin pipe"
                );
            }
        }
    }
}

/// **The hair-thin pair is turned away at the finest ask as well**, so its zero
/// is not the size of the harness's ask.
///
/// A hundredth of a loop that is `4.34e-19` across is as small an ask as this
/// harness ever makes, and both runs of both pairs still refuse — in
/// microseconds, because the tangent guard fires there. That is the limit half
/// of §16.10 for these two: there is no answer to reach, and the kernel says so.
#[test]
fn the_hair_thin_pipes_are_refused_at_the_finest_ask_too() {
    for pair in [
        the_huge_pipe_and_the_thread_through_it(),
        the_same_thread_moved_and_the_huge_pipe_tilted_further(),
    ] {
        for run in &the_runs_of(&pair) {
            let why = walk_it(&pair, run, 0.01).expect_err("no f64 point sits on both these pipes");
            assert!(
                matches!(
                    why,
                    Error::CannotFollowACrossingWhereItsSurfacesAreTangent { .. }
                ),
                "the marcher stopped for {why:?} rather than calling the pair tangent"
            );
        }
    }
}

/// **Every zero is the sizing walk refusing**, for the seeded hair-thin pair.
///
/// One test per pair rather than a loop over them, because until Q1598 each one
/// marched to the million-point cap and took seconds. Since then the sizing
/// walk is refused as tangent, at bounding how far off the crossing a point is,
/// in hundredths of a second.
#[test]
fn the_zero_on_the_seeded_hair_thin_pair_is_the_sizing_walk_refusing() {
    only_the_sizing_walk_gives_up_on(the_huge_pipe_and_the_thread_through_it());
}

/// The same, for `3a4a8231`.
#[test]
fn the_zero_on_the_moved_hair_thin_pair_is_the_sizing_walk_refusing() {
    only_the_sizing_walk_gives_up_on(the_same_thread_moved_and_the_huge_pipe_tilted_further());
}

/// Both runs of `pair` left the oracle at its first exit and at no other.
fn only_the_sizing_walk_gives_up_on(pair: [Surface; 2]) {
    let went = how_every_crossing_was_walked(pair);
    assert_eq!(went.len(), 2, "each of these crosses in two runs");
    for run in &went {
        assert!(
            matches!(run, WhereARunWent::TheSizingWalkRefused(_)),
            "this run left the oracle by {run:?} and not by the sizing walk"
        );
    }
}

/// **The pipe tilted by `3.85e-122` walks, and every point it walks is
/// checked**, for `1bd43cf4`.
///
/// Until Q1598 its sizing walk crawled to the point cap and the oracle checked
/// nothing. Now both runs walk at the harness's own asks.
#[test]
fn the_pipe_tilted_by_a_hundred_and_twenty_two_orders_has_every_walked_point_checked() {
    every_walked_point_is_checked_in_a_box_far_smaller_than_the_loop(
        the_big_pipe_and_the_thin_one_tilted_by_3e_122(),
    );
}

/// The same, for `d8b562bb`, tilted by `1.15e-129`.
#[test]
fn the_pipe_tilted_by_a_hundred_and_twenty_nine_orders_has_every_walked_point_checked() {
    every_walked_point_is_checked_in_a_box_far_smaller_than_the_loop(
        the_big_pipe_and_the_thin_one_tilted_by_1e_129(),
    );
}

/// Both runs of `pair` reached the oracle's last exit with points, and every one
/// of them kept its box, so every one was asked about both pipes.
///
/// `how_every_crossing_was_walked` panics on a point whose box is certified off
/// either pipe, so reaching the assertions here says each point is on both
/// within that box. The box is the fine walk's own deviation, so this also
/// walks each run at the asks the harness makes — the coarsest a walk can be
/// asked for, then a hundredth of the size that gave back — and holds that box
/// far under the thin pipe's radius of `1.375`. A box as wide as the loop would
/// make the check say nothing.
fn every_walked_point_is_checked_in_a_box_far_smaller_than_the_loop(pair: [Surface; 2]) {
    let went = how_every_crossing_was_walked(pair);
    assert_eq!(went.len(), 2, "each of these crosses in two runs");
    for run in &went {
        let WhereARunWent::Walked { drawn, checked } = run else {
            panic!("this run left the oracle by {run:?} and walked no points");
        };
        assert!(*drawn > 0, "the walk drew no points at all");
        assert_eq!(
            drawn,
            checked,
            "{} of {drawn} walked points lost their box and went uncounted",
            drawn - checked
        );
    }
    for run in &the_runs_of(&pair) {
        let (_, sized) = walk_it(&pair, run, f64::MAX / 4.0).expect("the sizing walk answers");
        let how_big = sized / POINT_ALONG_A_CROSSING_ROUNDOFF;
        let (_, box_size) = walk_it(&pair, run, how_big * 1e-2).expect("the fine walk answers");
        assert!(
            box_size < 1e-6,
            "the oracle checked each point in a box reaching {box_size:e} each way, round a loop of radius 1.375"
        );
    }
}

/// `7e62ebe6`, the seed of the five that walks: a pipe of radius `167774208`
/// lying along `x` with one of radius `1.375` standing along `z` through it.
fn the_big_pipe_and_the_thin_one_with_no_tilt() -> [Surface; 2] {
    the_pair_the_fuzzer_drew(&[
        0.0,
        0.0,
        805306368.0,
        64.0,
        0.0,
        0.0,
        167774208.0,
        0.0,
        32.0,
        1.0,
        0.0,
        0.0,
        1.0390772379261495e138,
        1.375,
    ])
}

/// `1bd43cf4`: the pair above with the thin pipe's axis y-component at
/// `3.8466290314169993e-122` instead of exactly zero.
fn the_big_pipe_and_the_thin_one_tilted_by_3e_122() -> [Surface; 2] {
    the_pair_the_fuzzer_drew(&[
        0.0,
        0.0,
        805306368.0,
        64.0,
        0.0,
        0.0,
        167774208.0,
        0.0,
        32.0,
        1.0,
        0.0,
        1191182336.0,
        3.0966914830391094e130,
        1.375,
    ])
}

/// `d8b562bb`: the same again at `1.1463847850015757e-129`.
fn the_big_pipe_and_the_thin_one_tilted_by_1e_129() -> [Surface; 2] {
    the_pair_the_fuzzer_drew(&[
        0.0,
        0.0,
        805306368.0,
        64.0,
        0.0,
        0.0,
        167774208.0,
        0.0,
        32.0,
        1.0,
        0.0,
        1191182336.0,
        1.0390772379261495e138,
        1.375,
    ])
}

/// `5dddf2a9`, the other seed: a pipe of radius `50397183` with one of radius
/// `4.336808689942018e-19` through it.
fn the_huge_pipe_and_the_thread_through_it() -> [Surface; 2] {
    the_pair_the_fuzzer_drew(&[
        0.0,
        -0.25,
        -393344.03125,
        8970.0,
        -393352.0,
        5.667849711660012e22,
        50397183.0,
        -393352.0,
        8970.0,
        12800.0,
        0.0,
        1.806844170501043e19,
        0.125,
        -4.336808689942018e-19,
    ])
}

/// `3a4a8231`: the pair above with the thread moved from `z = 12800` to
/// `z = 13312` and the huge pipe's axis tilted from `-6.94e-18` to `-6.24e-17`.
fn the_same_thread_moved_and_the_huge_pipe_tilted_further() -> [Surface; 2] {
    the_pair_the_fuzzer_drew(&[
        0.0,
        -0.25,
        -393344.03125,
        8970.0,
        -3539080.0,
        5.667849711660012e22,
        50397183.0,
        -393352.0,
        8970.0,
        13312.0,
        0.0,
        1.806844170501043e19,
        0.125,
        -4.336808689942018e-19,
    ])
}

/// Two pipes, decoded from the numbers `rcad-fuzz-explain` printed on its
/// `rebuild:` line, so that what is tested is what the fuzzer drew.
fn the_pair_the_fuzzer_drew(numbers: &[f64]) -> [Surface; 2] {
    two_surfaces_from(&bytes_for([2, 2], numbers)).expect("these bytes name a pair of pipes")
}

/// The runs a pair of pipes crosses in, under a certified policy.
fn the_runs_of(pair: &[Surface; 2]) -> Vec<NamedRun> {
    let named = intersect(
        &pair[0],
        &pair[1],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("these pipes cross");
    let SurfaceMeet::Curves(runs) = named else {
        panic!("these pipes cross in curves, not in {named:?}");
    };
    runs
}

/// How many points one walked run came back with and how far off the crossing
/// the furthest of them may sit, or why the walk refused.
fn walk_it(pair: &[Surface; 2], run: &NamedRun, room: f64) -> Result<(usize, f64), Error> {
    let walked = walk(
        run,
        [&pair[0], &pair[1]],
        Deflection::of(room).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )?;
    Ok((walked.points().len(), walked.deviation().magnitude()))
}

mod fuzzing;
