//! Why a huge pipe crossed by a thin one took seconds, and which half of it was
//! a gap. Since Q1598 neither half crawls.
//!
//! The overnight fuzzer found four slow units on `where_two_surfaces_cross`
//! across 2026-09-12 to 2026-09-14. All four decode to two pipes, one huge and
//! one ordinary or thinner. **They are two cases and not one**, and until
//! 2026-09-17 the counter said so the same way for every one of them: all four
//! came back `Asked { points: 0, orders: 1, scalings: 2 }` — named both ways
//! round, scaled both ways, and not one point walked, `points: 0` meaning every
//! walk refused.
//!
//! **Two of the four stopped refusing on 2026-09-17, and this file said
//! otherwise for a day.** `1f1da0bf` made a refused trial retry at the square
//! root of the ask rather than at half of it, and `262c17e4` and `7e62ebe6` —
//! the same pair drawn at two places along its own axis — have come back
//! `Asked { points: 10, … }` ever since. §125.7 has the bisect that names it.
//! Every `points: 0` reading below describes the tree before that commit; what
//! is written about `5dddf2a9` is untouched by it and still holds.
//!
//! Naming is not what costs. Measured in release on the pair behind `262c17e4`
//! **and before `1f1da0bf`**, `intersect` takes 40.6µs one way round and 10.9µs
//! the other, and the two coarse walks that followed took **7.30 seconds**.
//! Every one of those seconds was inside [`walk`], ended by
//! [`MOST_POINTS_ALONG_ONE_CROSSING`](rcad_geom::MOST_POINTS_ALONG_ONE_CROSSING)
//! — 1 048 576 steps and a `TooManyPointsAlongOneCrossing`. That crawl is gone
//! for this pair:
//! [`a_huge_pipe_and_a_thin_one_are_walked_even_when_asked_for_all_the_room_there_is`]
//! walks both pairs, on both runs, at `f64::MAX / 4.0`, in **7ms together**. It
//! was not gone for the thread below until Q1598.
//!
//! # The first case is the sizing ask, and it is a gap
//!
//! `262c17e4` and `7e62ebe6` are one pair moved along its own axis: a pipe of
//! radius `167774208` lying along `x` with a pipe of radius `1.375` standing
//! along `z` punched through it, crossing in two loops about `1.375` across at
//! `z ≈ 6.5e8` and `z ≈ 9.9e8`. One step of the grid there is `1.19e-7`, so a
//! loop spans about twelve million of them — there is nothing small about this
//! crossing but the ratio.
//!
//! [`the_two_loops_a_thin_pipe_cuts_through_a_huge_one_are_walked`] is the
//! demonstration §16.10 asks for: the very same run, walked at any ask from
//! `0.01` to `1e16`, comes back in **21 to 26 microseconds** with five to
//! sixty-five points sitting `8.94e-8` off. The arithmetic reaches the answer.
//! What does not is the harness's *sizing* walk in
//! `crates/rcad-geom/tests/fuzzing/mod.rs`, which asks `f64::MAX / 4.0` on
//! purpose to find out how big a crossing is before asking for the real one.
//! Against an ask that large the marcher's step has no ceiling worth the name:
//! `march.rs`'s `MOST_A_STEP_MAY_GROW_TO` is a million times the ask, and
//! `SAGITTA_MAY_TAKE` is measured against the ask too, so both go vacuous
//! exactly where they were needed.
//!
//! **That paragraph now describes a pair that survives the ask anyway.** Both
//! ceilings are still the caller's ask in disguise; what `1f1da0bf` changed is
//! how fast a refused trial shortens, which is a third knob and enough for this
//! pair. It was not enough for the same pair with a hair of tilt on it, so
//! §125.2 carried the gap on, on `1bd43cf4` and `d8b562bb`. Those two walk the
//! sizing ask too since Q1598
//! (`why_the_walked_point_oracle_checks_nothing_on_the_slow_pipes.rs`), but the
//! ceilings are still the caller's ask, and §125.6 keeps that open.
//!
//! **It is the ask and not the pair.** Swept over the budget, the same run gives
//! five points at `1e1`, `1e4`, `1e8`, `1e10` … `1e16`, crawls to the cap at
//! `1e20`, walks again in six points at `1e30`, and crawls at `1e100` and at
//! `f64::MAX / 4.0`. Non-monotone, which is what a wild first step landing
//! wherever the settling drops it looks like.
//!
//! **It is not the magnitude.** The whole pair written at `2^-8`, `2^-16`,
//! `2^-24`, `2^-32`, `2^8` and `2^16` — every ratio held fixed — crawls to the
//! same cap in the same 3.7 seconds at every one of them.
//!
//! **It is the ratio.** Holding the huge pipe at `167774208` and fattening the
//! thin one: `1.375` crawls, `16` refuses as tangent in 57.7µs, and `1024`,
//! `65536`, `1.6777e6` and `1.6777e7` all walk in three to six points in under
//! 29µs. The crawl lives below a ratio of about `1e-5`.
//!
//! The family of fixes, per §16.10, and why it is one family: the step needs a
//! ceiling that does not come from the caller's ask. Either
//! `MOST_A_STEP_MAY_GROW_TO` is measured against something absolute — the
//! crossing's own size, which `walk` already works out for the conic case — or
//! the first ask is clamped to it. Raising the point cap is not in the family:
//! a walk that is not following the curve does not start following it with more
//! points. Nor is changing the harness to stop asking `f64::MAX / 4.0` — §16.10
//! says to check the **public API**, and [`Deflection`](rcad_geom::Deflection)
//! takes any positive length from any caller.
//!
//! # The second case is §94's family again, and the answer does not exist
//!
//! `5dddf2a9` is a pipe of radius `50397183` with a pipe of radius
//! `4.336808689942018e-19` through it, crossing near `x = -393352`. One step of
//! the grid there is `5.82e-11`, so the loop is about **`7.5e-9` of one grid
//! step** wide. No `f64` point sits on both pipes, exactly as in
//! `how_small_a_crossing_can_be_before_no_point_sits_on_it.rs`, and that is a
//! limit and not a gap.
//!
//! What was a gap is how long saying so took. §94's guard,
//! `the_settling_did_not_land_on_both`, turns `b9068e9e` away in **7.1µs**; it
//! does not fire here, and this pair crawled to the point cap in **2.81s at
//! `f64::MAX / 4.0`, 2.76s at `1e10` and 2.89s at `1.0`** before refusing for
//! the wrong reason. **Since Q1598 it does not crawl.** The marcher no longer
//! keeps a trial that came back short and no further from its start than the
//! two ends' own bounds, so the trials that settle back near the seed are tried
//! again shorter, and the first step it keeps is one step of the grid, whose
//! far end it cannot bound. The walk is refused there, after one step and in
//! milliseconds, by the same name the finest ask already got
//! (`why_the_walked_point_oracle_checks_nothing_on_the_slow_pipes.rs`).
//! [`a_crossing_thinner_than_one_step_of_the_grid_is_refused_at_its_first_step`]
//! pins that.
//!
//! [`the_same_pair_is_walked_as_soon_as_its_thin_pipe_spans_a_grid_step`] is the
//! limit half: nothing changes but the thin radius, and at `1e-7` — about 1 718
//! grid steps — the same call walks it in six points in 30.4µs.
//!
//! # How every number above was measured
//!
//! In a release build, `mise exec -- just t --release --no-capture -E
//! 'test(<name>)' --run-ignored all`, on the throwaway measurement file this
//! lane replaced. The debug figure of 3m47s quoted in the lane's brief is not
//! repeated here because it was not run here.

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cylinder, Deflection, Error, NamedRun, Point, Surface, SurfaceMeet, Vector,
    WhereFollowingACrossingGaveUp, intersect, walk,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// The pair behind `262c17e4`: a huge pipe lying along `x` and a thin one
/// standing along `z` punched through it.
///
/// Written out rather than decoded from the fuzzer's bytes so that what it is
/// can be read. `7e62ebe6` is the same pair with the huge pipe's axis moved from
/// `822083584` to `805306368`, which is the same shape drawn somewhere else
/// along its own axis, and the two behave identically at every ask below.
fn the_huge_pipe_and_the_thin_one_through_it(huge_pipe_sits_at: f64) -> [Surface; 2] {
    [
        pipe([0.0, 0.0, huge_pipe_sits_at], [1.0, 0.0, 0.0], 167774208.0),
        pipe([0.0, 32.0, 1.0], [0.0, 0.0, 1.0], 1.375),
    ]
}

/// **The answer exists and the arithmetic reaches it**, which is what makes the
/// crawl above a gap rather than a limit (§16.10).
///
/// Every ask here is an ordinary one — from a hundredth of the loop's own
/// radius up to `1e16`, which is already seven orders of magnitude past the
/// coordinates the loop sits at. All of them come back with points, and every
/// point sits far closer to the crossing than the thin pipe is wide, so they
/// are on the loop and not merely somewhere near the pair.
#[test]
fn the_two_loops_a_thin_pipe_cuts_through_a_huge_one_are_walked() {
    for huge_pipe_sits_at in [822083584.0, 805306368.0] {
        let pair = the_huge_pipe_and_the_thin_one_through_it(huge_pipe_sits_at);
        let runs = the_runs_of(&pair);
        assert_eq!(runs.len(), 2, "a pipe punched through another leaves two");
        for run in &runs {
            for ask in [0.01, 1.0, 1e10, 1e16] {
                let (drawn, off) = walk_it(&pair, run, ask).unwrap_or_else(|why| {
                    panic!("asked for {ask:e} of room and got {why:?} at {huge_pipe_sits_at:e}")
                });
                assert!(
                    drawn > 0,
                    "asking for {ask:e} of room drew no points at all"
                );
                assert!(
                    off < 1.375,
                    "the points may sit {off:e} off the loop, which is wider than the thin pipe"
                );
            }
        }
    }
}

/// **The enormous ask is walked too, and that is new.** Pinned because nothing
/// else here would catch it moving back.
///
/// The harness's sizing walk asks `f64::MAX / 4.0` before it asks for anything
/// real, and until 2026-09-17 this pair refused exactly that ask — which was the
/// whole of the `Asked { points: 0 }` the fuzz oracle reported (§125.1).
/// `1f1da0bf` made a refused trial retry at the square root of the ask rather
/// than half of it, and both pairs have walked it since (§125.7).
///
/// Every ask in [`the_two_loops_a_thin_pipe_cuts_through_a_huge_one_are_walked`]
/// is an ordinary one, and an ordinary ask was never the one that refused. So
/// that test passed on both sides of the change and would pass again if it went
/// away.
#[test]
fn a_huge_pipe_and_a_thin_one_are_walked_even_when_asked_for_all_the_room_there_is() {
    for huge_pipe_sits_at in [822083584.0, 805306368.0] {
        let pair = the_huge_pipe_and_the_thin_one_through_it(huge_pipe_sits_at);
        let runs = the_runs_of(&pair);
        assert_eq!(runs.len(), 2, "a pipe punched through another leaves two");
        for run in &runs {
            let (drawn, off) = walk_it(&pair, run, f64::MAX / 4.0).unwrap_or_else(|why| {
                panic!("the sizing ask got {why:?} at {huge_pipe_sits_at:e}")
            });
            assert!(drawn > 0, "the sizing ask drew no points at all");
            assert!(
                off < 1.375,
                "the points may sit {off:e} off the loop, which is wider than the thin pipe"
            );
        }
    }
}

/// **A crossing narrower than one step of the grid it sits on is a limit, and
/// it is refused after one step.**
///
/// The loop is `4.34e-19` across where one step of the grid is `5.82e-11`, so
/// there is no answer to reach — the same sentence
/// `how_small_a_crossing_can_be_before_no_point_sits_on_it.rs` makes about
/// `b9068e9e`, and [`the_same_pair_is_walked_as_soon_as_its_thin_pipe_spans_a_grid_step`]
/// demonstrates it (§16.10). §94's seed check turns that one away in
/// microseconds and does not fire on this one.
///
/// Until Q1598 the refusal came a million marched points later, as
/// `TooManyPointsAlongOneCrossing`, and cost seconds. Now the first step takes
/// 33 trials, from `0.5` down. Eight of them settle `7.45e-9` from the seed,
/// no further than the two ends' own bounds reach, and the marcher no longer
/// keeps such a trial; the other 24 are refused where the far end cannot be
/// bounded. The one it keeps is `1.16e-10`, a chord of `5.82e-11`, one step of
/// the grid, and the angle between the two pipes at that far end cannot be told
/// from nought, so the walk is refused there as tangent, while bounding how far
/// off that point sits. The pipes are not tangent: at a point one grid step
/// from a thread `4.34e-19` wide, the thread's normal is all rounding, and
/// that is what the refusal reports. Measured with prints in `march.rs`'s
/// `one_step`, not kept.
#[test]
fn a_crossing_thinner_than_one_step_of_the_grid_is_refused_at_its_first_step() {
    let pair = the_huge_pipe_and_the_thread_through_it(4.336808689942018e-19);
    let runs = the_runs_of(&pair);
    let why = walk_it(&pair, &runs[0], 1.0).expect_err("nothing can be walked here");
    assert!(
        matches!(
            why,
            Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
                at: WhereFollowingACrossingGaveUp::BoundingHowFarOffTheCrossingAPointIs,
                ..
            }
        ),
        "the marcher stopped for {why:?} rather than failing to bound how far off a point sits"
    );
}

/// The limit half of the pair above, demonstrated rather than argued (§16.10).
///
/// Nothing changes but the thin pipe's radius. At `1e-7` the loop finally spans
/// about 1 718 steps of the grid it sits on, and the very same call walks it.
#[test]
fn the_same_pair_is_walked_as_soon_as_its_thin_pipe_spans_a_grid_step() {
    let one_step_of_the_grid = (-393352.0_f64).next_up() + 393352.0;
    let pair = the_huge_pipe_and_the_thread_through_it(1e-7);
    assert!(
        1e-7 > one_step_of_the_grid * 1000.0,
        "a thin pipe of 1e-7 has to be many grid steps wide for this to prove anything"
    );
    let runs = the_runs_of(&pair);
    let (drawn, off) = walk_it(&pair, &runs[0], 1.0).expect("a loop a thousand grid steps across");
    assert!(drawn > 0, "the walk drew no points at all");
    assert!(
        off < 1e-7,
        "the points may sit {off:e} off the loop, which is wider than the thin pipe"
    );
}

/// The pair behind `5dddf2a9`, with the thin pipe's radius left open so that the
/// two tests above differ in that and in nothing else.
fn the_huge_pipe_and_the_thread_through_it(thin: f64) -> [Surface; 2] {
    [
        pipe(
            [0.0, -0.25, -393344.03125],
            [1.582610770632598e-19, -6.940056988292907e-18, 1.0],
            50397183.0,
        ),
        pipe(
            [-393352.0, 8970.0, 12800.0],
            [0.0, 1.0, 6.918139485450875e-21],
            thin,
        ),
    ]
}

/// The runs a pair of pipes crosses in, under a certified policy.
///
/// A `let`-else rather than a `match` with a wildcard in it, because every other
/// answer [`SurfaceMeet`] can give is wrong here for the same reason and there is
/// nothing for a new variant to be quietly folded into (§40).
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

fn pipe(origin: [f64; 3], direction: [f64; 3], radius: f64) -> Surface {
    Surface::Cylinder(
        Cylinder::new(
            Axis::new(Point::from(origin), Vector::from(direction)).expect("an axis"),
            radius,
        )
        .expect("a pipe"),
    )
}
