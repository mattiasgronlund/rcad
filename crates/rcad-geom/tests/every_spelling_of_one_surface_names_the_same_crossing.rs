//! One surface written twice, a few ulps apart, has to name the crossing its
//! run names — the same one it names when it is written twice bit for bit.
//!
//! §219.8 asked for this file. `two_spheres_a_hair_apart_are_one_sphere_and_a_
//! second_surface.rs` is one case of it, drawn by hand: a ball written twice
//! 1e-15 apart used to slip past `which_two_surfaces_a_run_is_between`'s
//! `along[0] == along[1]` guard, so the pair either side went to
//! `where_two_balls_cut`, which answers **finitely and wrongly** — a great
//! circle of the ball's own radius through its own centre, in the plane halfway
//! between the two spellings (`docs/decisions.md` §218.6). §219.1 closed that
//! one case. This is the family.
//!
//! # The oracle is the same surface written twice bit for bit
//!
//! Two runs that must agree, and they must agree **exactly**, not within a
//! tolerance. Every arm of [`which_two_surfaces_a_run_is_between`] that fires
//! hands back one of the two faces and `and`: the first face listed where the
//! faces are flat, and where they are curved the one
//! [`Surface::how_it_sorts_against`] puts first, so that the answer does not
//! depend on which face is listed first (`docs/decisions.md` §336). So a pair
//! that reaches any of them is handed to `what_a_run_between_these_two_lies_on`
//! as the *same two surfaces* that one spelling written twice bit for bit is
//! handed, and the answer is bit for bit the same point and the same list of
//! points. A tolerance here would be admitting a difference that cannot exist
//! unless an arm was missed, which is the whole thing being tested (`testing`
//! skill, "a tolerance is not an oracle").
//!
//! The control is that spelling written **once** — `[&s, &s]`, with `s` the
//! first spelling or the second, whichever the answer was read with — which
//! always reaches the seam arm or the closed-run arm and so always names
//! `[s, and]`. That is what the run means, and it is what the re-spelt pair has
//! to agree with. That the answer was read with one of the two spellings is
//! asserted, not assumed, and where the faces are curved so is that listing
//! them the other way round gives the same answer.
//!
//! # What "a few ulps" means here, and why
//!
//! One to four steps of the `f64` number line, away from zero or towards it,
//! applied to the numbers the surface stores: a ball's centre, an axis's origin,
//! a plane's offset. Not a fixed distance — a fixed distance is a different
//! number of steps at every magnitude, and it is the *steps* that decide whether
//! two spellings are equal by bits.
//!
//! Four steps is the top because the case already measured is about one. §219.1
//! wrote its second ball 1e-15 along `x` from a centre at the origin with a
//! radius of 5; one ulp of 5.0 is 8.88e-16, so that case is a little over one
//! step. One to four brackets it, and going wider would start drawing surfaces
//! that are genuinely somewhere else rather than the one surface written twice.
//!
//! **A step from 0.0 is 5e-324**, the smallest number there is, and that is
//! kept rather than skipped: it is the harshest spelling of all, numerically
//! nothing and bitwise a difference, and it is exactly what a file that writes a
//! centre out and reads it back can produce.
//!
//! # A nudge that folds is counted, not skipped quietly
//!
//! Some nudges come back as the same surface — a plane's offset one step down
//! from 0.0 saturates at 0.0, an axis origin whose every coordinate is 0.0 and
//! whose step is inward does not move. Those draws have nothing to test, because
//! the seam arm fires for them and correctly. They are counted and reported, and
//! the floors below are on the draws that did *not* fold.
//!
//! # Which arms this reaches, and the one it cannot close
//!
//! `which_arm_the_respelt_pair_reaches` asks the public answer whether any arm
//! caught the pair — one that did not hands back the pair either side — and
//! names the arm that did by the shape of the draw, which is all that tells
//! them apart once `and` is neither face. It used to restate the function's
//! guards, because the function was private; since §221.5 it is public and the
//! copy is gone. It is a **counter and never an oracle** — nothing is asserted
//! from it — and it is here so that the report says which arms were actually
//! drawn rather than leaving it to be believed.
//!
//! What *is* asserted from the public answer is the pair itself: every draw
//! that an arm catches has to hand the kernel exactly the pair the surface
//! written once does, before either door is asked anything.
//!
//! Four arms are reached and agree. The fifth bucket is the **fallthrough**, and
//! since §220.4 all that reaches it is two flat faces whose run names a third
//! plane: a straight line, with no closed crossing to start on either way.
//!
//! Until then the fallthrough was a defect. §189.3's arm asked for a **ball**
//! by name — `both_curved && and.as_plane().is_some() && a_ball.is_some()` — so
//! a pipe, a cone or a torus written twice with a plane `and` went to the pair
//! either side, two nearly-coincident curved surfaces, and answered something
//! else entirely. `docs/decisions.md` §220.4 measured that bucket disagreeing
//! with its control 405 times out of 592 on the shipped corpus.
//! `a_pipe_written_twice_a_hair_apart_walks_the_circle_its_plane_names` below is
//! that defect as one case.
//!
//! **That measurement is also the proof this comparison is not a tautology.** A
//! property that hands back `Ok(())` whatever happens is the failure this file
//! could most easily have been. Nothing about the comparison changes between
//! the buckets — only which arm the pair reaches — and it did disagree where an
//! arm was missing, so the arms that never disagree are saying something rather
//! than being unable to say anything. Put §189.3's guard back and the property
//! fails again, on a curved face that is not a ball with a plane `and`, and so
//! does the case below.
//!
//! **The arm trusts the plane**, and that trust is earned in `rcad-step`, which
//! writes a plane beside two curved faces only for a circle it has checked lies
//! on every face (`the_surface_that_names_the_curve`). This file draws the plane
//! at random rather than as the circle's own, which is fine here: the pair
//! written once goes through the seam arm to the same `[face, plane]`, so both
//! spellings are handed the same surfaces whether or not the plane is honest.

#![forbid(unsafe_code)]

mod soak;

use std::cell::RefCell;
use std::collections::BTreeMap;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, TestRunner};
use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, EdgeRun, Error, HalfSpace, NamedRun, Picker, Plane, Point,
    Sphere, Surface, Torus, Vector, walk, where_a_closed_crossing_starts,
    which_two_surfaces_a_run_is_between,
};
use rcad_predicates::{Policy, ProvenanceLog};
use smallvec::smallvec;

/// How many draws the shipped corpus takes.
///
/// Wide, because a draw is a handful of sign decisions and a walk of at most a
/// few dozen points: the whole file runs in well under a second at this width.
const MEASURED_CASES: u32 = 8192;

/// How coarsely the walk is asked for.
///
/// Coarse on purpose: the comparison is exact either way, because both walks are
/// handed the same two surfaces, and a fine deflection only buys points to
/// compare that cost time to build.
const HOW_COARSE_THE_WALK_IS: f64 = 0.25;

/// The whole property: every spelling names the crossing the run names.
#[test]
fn a_surface_written_twice_a_few_ulps_apart_names_the_crossing_its_run_names() {
    let tally = Tally::default();
    let mut runner =
        TestRunner::new_with_rng(configured(soak::cases(MEASURED_CASES)), soak::seeded());
    let outcome = runner.run(&one_surface_written_twice(), |drawn| {
        tally.saw(drawn);
        Ok(())
    });
    if let Err(why) = outcome {
        panic!("{why}");
    }

    tally.report();
    tally.every_closed_arm_agreed();
    tally.every_closed_arm_was_reached();
    tally.the_fallthrough_is_only_a_line();
}

/// One draw: a surface both faces sit on, the second surface the run names,
/// where each sits, how the run is said, and how the second spelling is written.
#[derive(Debug, Clone, Copy)]
struct OneSurfaceWrittenTwice {
    face: WhichSurface,
    and: WhichSurface,
    face_at: [i8; 3],
    and_at: [i8; 3],
    said: HowTheRunIsSaid,
    steps: u8,
    outward: bool,
}

/// A kind of surface and the parameters that pick one of its family.
///
/// Indices rather than coordinates, so shrinking walks toward the first
/// direction and the smallest radius rather than toward a degenerate shape
/// (`testing` skill, "shrink parameters, never coordinates").
#[derive(Debug, Clone, Copy)]
enum WhichSurface {
    Flat { normal: u8 },
    Ball { radius: u8 },
    Pipe { axis: u8, radius: u8 },
    Taper { axis: u8, slope: u8 },
    Ring { axis: u8 },
}

/// The two ways a run names a second surface.
///
/// [`EdgeRun::Straight`] and [`EdgeRun::RightRoundWhereItsFacesCross`] are not
/// drawn: neither carries an `and`, so neither can say anything about a pair
/// either side that is one surface written twice.
#[derive(Debug, Clone, Copy)]
enum HowTheRunIsSaid {
    RightRoundIt,
    APieceOfIt { facing: u8 },
}

fn one_surface_written_twice() -> impl Strategy<Value = OneSurfaceWrittenTwice> {
    (
        a_surface_kind(),
        a_surface_kind(),
        a_lattice_point(),
        a_lattice_point(),
        how_a_run_is_said(),
        1u8..=4,
        any::<bool>(),
    )
        .prop_map(|(face, and, face_at, and_at, said, steps, outward)| {
            OneSurfaceWrittenTwice {
                face,
                and,
                face_at,
                and_at,
                said,
                steps,
                outward,
            }
        })
}

fn a_surface_kind() -> impl Strategy<Value = WhichSurface> {
    prop_oneof![
        (0u8..HOW_MANY_DIRECTIONS).prop_map(|normal| WhichSurface::Flat { normal }),
        (0u8..HOW_MANY_RADII).prop_map(|radius| WhichSurface::Ball { radius }),
        (0u8..HOW_MANY_DIRECTIONS, 0u8..HOW_MANY_RADII)
            .prop_map(|(axis, radius)| WhichSurface::Pipe { axis, radius }),
        (0u8..HOW_MANY_DIRECTIONS, 0u8..HOW_MANY_SLOPES)
            .prop_map(|(axis, slope)| WhichSurface::Taper { axis, slope }),
        (0u8..HOW_MANY_DIRECTIONS).prop_map(|axis| WhichSurface::Ring { axis }),
    ]
}

/// Where a surface sits.
///
/// A family's placement is part of its sample set (`testing` skill), so it is
/// drawn rather than fixed — and the two surfaces are placed independently, so
/// the pair is sometimes crossing and sometimes clear of each other.
fn a_lattice_point() -> impl Strategy<Value = [i8; 3]> {
    prop::array::uniform3(-4i8..=4)
}

fn how_a_run_is_said() -> impl Strategy<Value = HowTheRunIsSaid> {
    prop_oneof![
        Just(HowTheRunIsSaid::RightRoundIt),
        (0u8..HOW_MANY_DIRECTIONS).prop_map(|facing| HowTheRunIsSaid::APieceOfIt { facing }),
    ]
}

/// Which arm of [`which_two_surfaces_a_run_is_between`] a re-spelt pair reaches.
///
/// A counter and never an oracle. Nothing below asserts an answer from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Arm {
    /// `RightRoundWhereItCrosses { and }`, which takes `and` whatever is either
    /// side of it, so long as `and` is neither face. §216, §221.5.
    AClosedRunCarriesItsSecondSurface,
    /// `both_flat && and.as_plane().is_none()`. §176.2's two planes 1.2e-15
    /// apart with a curved `and`.
    TwoFlatFacesAndACurvedSecond,
    /// A piece of a run between two curved faces that names a plane. §189.3's
    /// ball and curved face naming a plane, widened to every curved face by
    /// §220.4.
    ACurvedFaceAndAPlane,
    /// A piece of a run between two curved faces that names a curved surface
    /// that is neither of them. §219.1's third-surface arm.
    TwoCurvedFacesAndAThirdCurvedOne,
    /// No arm catches it, so the pair either side stands — which for a pair that
    /// is one surface written twice is the wrong answer.
    NoArmCatchesIt,
}

const EVERY_ARM: [Arm; 5] = [
    Arm::AClosedRunCarriesItsSecondSurface,
    Arm::TwoFlatFacesAndACurvedSecond,
    Arm::ACurvedFaceAndAPlane,
    Arm::TwoCurvedFacesAndAThirdCurvedOne,
    Arm::NoArmCatchesIt,
];

/// The four arms that hand back one of the two faces and `and`, and so have to
/// agree with the control.
const EVERY_CLOSED_ARM: [Arm; 4] = [
    Arm::AClosedRunCarriesItsSecondSurface,
    Arm::TwoFlatFacesAndACurvedSecond,
    Arm::ACurvedFaceAndAPlane,
    Arm::TwoCurvedFacesAndAThirdCurvedOne,
];

/// What every arm saw, and the two buckets a draw can fall out into.
#[derive(Default)]
struct Tally {
    arms: RefCell<BTreeMap<Arm, WhatOneArmSaw>>,
    /// Draws whose second spelling came back as the first, bit for bit.
    the_nudge_folded: RefCell<usize>,
    /// Draws where `and` is one of the two faces bit for bit, which is the fuse
    /// saying "the honest pair stays" and is not one surface written twice.
    and_repeated_the_face: RefCell<usize>,
}

#[derive(Default, Debug, Clone, Copy)]
struct WhatOneArmSaw {
    reached: usize,
    the_control_named_a_start: usize,
    the_control_walked_points: usize,
    agreed: usize,
    disagreed: usize,
}

impl Tally {
    fn saw(&self, drawn: OneSurfaceWrittenTwice) {
        let at = a_point(drawn.face_at);
        let once = a_surface(drawn.face, at);
        let twice = the_same_surface_written_again(drawn.face, at, drawn.steps, drawn.outward);
        let and = a_surface(drawn.and, a_point(drawn.and_at));

        if once == twice {
            *self.the_nudge_folded.borrow_mut() += 1;
            return;
        }
        if and == once || and == twice {
            *self.and_repeated_the_face.borrow_mut() += 1;
            return;
        }

        let run = a_run(drawn.said, and, a_point(drawn.and_at));
        let arm = which_arm_the_respelt_pair_reaches(run.run(), drawn.said, &once, &twice, &and);
        let written_once = if EVERY_CLOSED_ARM.contains(&arm) {
            the_spelling_the_pair_is_read_with(arm, &run, &once, &twice)
        } else {
            once
        };
        let control = what_the_kernel_says(&run, [&written_once, &written_once]);
        let respelt = what_the_kernel_says(&run, [&once, &twice]);

        let mut arms = self.arms.borrow_mut();
        let saw = arms.entry(arm).or_default();
        saw.reached += 1;
        if control.start.is_ok() {
            saw.the_control_named_a_start += 1;
        }
        if walked_points(&control) {
            saw.the_control_walked_points += 1;
        }
        if control == respelt {
            saw.agreed += 1;
        } else {
            saw.disagreed += 1;
            panic!(
                "{arm:?}: one surface written twice named a different crossing from the one \
                 written once.\n  the face: {once:?}\n  written again: {twice:?}\n  and: \
                 {and:?}\n  run: {run:?}\n  written once: {control:?}\n  written twice: \
                 {respelt:?}"
            );
        }
    }

    fn report(&self) {
        for arm in EVERY_ARM {
            println!(
                "{arm:?}: {:?}",
                self.arms.borrow().get(&arm).copied().unwrap_or_default()
            );
        }
        println!(
            "{} draws whose nudge folded back to the first spelling, {} where `and` repeated a \
             face",
            self.the_nudge_folded.borrow(),
            self.and_repeated_the_face.borrow()
        );
    }

    /// The property itself, restated over the whole run so a green line cannot
    /// mean "nothing disagreed because nothing was compared".
    fn every_closed_arm_agreed(&self) {
        for arm in EVERY_CLOSED_ARM {
            let saw = self.arms.borrow().get(&arm).copied().unwrap_or_default();
            assert_eq!(
                saw.disagreed, 0,
                "{arm:?} hands back a face and `and` exactly as the seam arm does for that face, \
                 so it cannot disagree with the control: {saw:?}"
            );
        }
    }

    /// A floor per arm, because a family that stopped reaching one of them would
    /// otherwise leave this file green over four arms it never drew.
    ///
    /// The floors are on draws where the **control named a start** — a real
    /// point on a real closed crossing — rather than on draws reached, because a
    /// pair that refuses by name compares two refusals and proves very little.
    fn every_closed_arm_was_reached(&self) {
        for (arm, floor) in EVERY_ARM_AND_ITS_FLOOR {
            let saw = self.arms.borrow().get(&arm).copied().unwrap_or_default();
            assert!(
                saw.the_control_named_a_start >= floor,
                "{arm:?} was reached with an answering control only {} times, under its floor of \
                 {floor}: {saw:?}",
                saw.the_control_named_a_start
            );
        }
        let walked: usize = EVERY_CLOSED_ARM
            .iter()
            .map(|arm| {
                self.arms
                    .borrow()
                    .get(arm)
                    .copied()
                    .unwrap_or_default()
                    .the_control_walked_points
            })
            .sum();
        assert!(
            walked >= A_WALK_WITH_POINTS_IN_IT,
            "only {walked} draws walked a crossing with points on it, so the walking half of this \
             property compared nothing"
        );
    }

    /// What still falls through is a straight line, and nothing else.
    ///
    /// Since §220.4 the only draw no arm catches is two flat faces whose run
    /// names a third plane. Three planes cross in a line or nowhere, so there is
    /// no closed crossing to name a start on, whichever spelling is asked — and
    /// the pair either side standing costs nothing there. A control that named a
    /// start in this bucket would be a closed crossing that some arm is missing.
    fn the_fallthrough_is_only_a_line(&self) {
        let saw = self
            .arms
            .borrow()
            .get(&Arm::NoArmCatchesIt)
            .copied()
            .unwrap_or_default();
        assert!(
            saw.reached >= A_DRAW_NO_ARM_CATCHES,
            "only {} draws fell through every arm, so the bucket that says what is left out is \
             not being drawn: {saw:?}",
            saw.reached
        );
        assert_eq!(
            saw.the_control_named_a_start, 0,
            "a draw no arm catches named a start on a closed crossing, so an arm is missing: \
             {saw:?}"
        );
    }
}

/// What each arm was measured at, and the floor under it.
///
/// Measured on this tree, on 2026-10-09, at the fixed seed and at
/// `RCAD_SOAK_SEED` 1 through 12 — thirteen runs in all — by
///
/// ```text
/// mise exec -- just t a_surface_written_twice_a_few_ulps_apart --no-capture
/// RCAD_SOAK_SEED=<n> mise exec -- just t a_surface_written_twice_a_few_ulps_apart --no-capture
/// ```
///
/// | arm | the control named a start, min .. max over the 13 runs |
/// |---|---|
/// | a closed run carries its second surface | 838 .. 964 |
/// | two flat faces and a curved second | 274 .. 327 |
/// | a curved face and a plane | 252 .. 330 |
/// | two curved faces and a third curved one | 123 .. 149 |
///
/// The fixed seed's own run sits inside each range (915, 293, 256, 125) and
/// reached each arm 4048, 606, 591 and 2706 times, with 132 draws falling
/// through every arm; those are the single-run figures the floors used to be
/// pinned to.
///
/// The third row was a ball and a plane, at 131 and 41 on the fixed seed, until
/// §220.4 widened the arm to every curved face; the draws it gained there are
/// the ones the fourth row lost, which fell through before.
///
/// §336 restated the control for curved arms — for a curved face the spelling
/// read is sometimes the second, a few ulps from the first, and whether a
/// crossing has a start to name can turn on those ulps — which moved the fixed
/// seed's own counts for the first, third and fourth arms from 899, 250, 128 to
/// 915, 256, 125 on 2026-09-24. The draws did not move.
///
/// Each floor here is roughly three quarters of the **lowest** of the 13
/// counts, not of the fixed seed's own count: a family that stopped reaching an
/// arm has to fail on an ordinary soak seed, not only on the one the shipped
/// corpus pins — the same rule
/// `every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs` sets its three
/// floors by. The four floors below replace ones that were still three quarters
/// of the pre-§336 fixed-seed counts, 899, 293, 250 and 128.
///
/// **The gap between a reached count and a start count is mostly a refusal by
/// name**, and that is why the floor is on the second. A cone or a torus
/// against another curved surface is
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] or
/// [`Error::ACrossingBetweenATorusAndAnotherCurvedSurface`] before any
/// arithmetic happens, so a draw that answers in the third-surface arm has a
/// ball or a pipe on both sides — which is §219.1's own case and the family it
/// asked for.
const EVERY_ARM_AND_ITS_FLOOR: [(Arm, usize); 4] = [
    (Arm::AClosedRunCarriesItsSecondSurface, 628),
    (Arm::TwoFlatFacesAndACurvedSecond, 205),
    (Arm::ACurvedFaceAndAPlane, 189),
    (Arm::TwoCurvedFacesAndAThirdCurvedOne, 92),
];

/// How many draws have to walk a crossing that has points on it, across the four
/// arms that are closed.
///
/// The sum was 789 + 323 + 280 + 125 = 1517 at the fixed seed, and ranged
/// 1492 .. 1637 over the fixed seed and `RCAD_SOAK_SEED` 1 through 12 (measured
/// 2026-10-09, same command as `EVERY_ARM_AND_ITS_FLOOR`); floored at three
/// quarters of the lowest, 1492. Without it the walking half of every
/// comparison could fall to nothing behind the naming half, which answers for a
/// conic the walk then refuses to step along.
const A_WALK_WITH_POINTS_IN_IT: usize = 1119;

/// How many draws have to fall through every arm.
///
/// Measured at 132 at the fixed seed, which over the same 13 runs
/// `EVERY_ARM_AND_ITS_FLOOR` measured was also the lowest (range 132 .. 174);
/// the floor stays at three quarters of that, 99, because the fixed seed was
/// already the worst case rather than the easiest.
const A_DRAW_NO_ARM_CATCHES: usize = 99;

/// What both public doors say about a run between these two surfaces.
///
/// Both, because they read the pair through the same function and then part
/// company: one builds a conic and takes its point at angle zero, the other
/// walks it. A file that asked only one of them would leave the other's reading
/// of a re-spelt pair untested.
#[derive(Debug, PartialEq)]
struct WhatTheKernelSays {
    start: Result<Point, Error>,
    walked: Result<Vec<Point>, Error>,
}

fn walked_points(said: &WhatTheKernelSays) -> bool {
    said.walked.as_ref().is_ok_and(|points| !points.is_empty())
}

fn what_the_kernel_says(run: &NamedRun, along: [&Surface; 2]) -> WhatTheKernelSays {
    WhatTheKernelSays {
        start: where_a_closed_crossing_starts(run, along, &mut ProvenanceLog::new()),
        walked: walk(
            run,
            along,
            Deflection::of(HOW_COARSE_THE_WALK_IS).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|points| points.points().to_vec()),
    }
}

/// Whether an arm caught the pair, as the public answer says, and which one, as
/// the draw's shape says.
///
/// An arm that catches a pair hands back `[once, and]` or `[twice, and]`; none
/// catching it hands back `[once, twice]`, and `and` is never `twice` here —
/// the caller has put those draws in [`Tally::and_repeated_the_face`] and
/// returned, and a draw whose nudge folded never gets this far either. Past
/// that, `once` and `twice`
/// are one kind of surface, so which arm caught it is only whether the faces
/// are flat, whether `and` is, and how the run is said.
fn which_arm_the_respelt_pair_reaches(
    run: &EdgeRun,
    said: HowTheRunIsSaid,
    once: &Surface,
    twice: &Surface,
    and: &Surface,
) -> Arm {
    if which_two_surfaces_a_run_is_between(run, [once, twice]) == [*once, *twice] {
        return Arm::NoArmCatchesIt;
    }
    let HowTheRunIsSaid::APieceOfIt { .. } = said else {
        return Arm::AClosedRunCarriesItsSecondSurface;
    };
    match (once.as_plane().is_some(), and.as_plane().is_some()) {
        (true, _) => Arm::TwoFlatFacesAndACurvedSecond,
        (false, true) => Arm::ACurvedFaceAndAPlane,
        (false, false) => Arm::TwoCurvedFacesAndAThirdCurvedOne,
    }
}

/// Which of the two spellings an arm read the re-spelt pair with.
///
/// Asserted to be one of them — the pair has to be what that spelling written
/// twice bit for bit is read as — and, where the faces are curved, the same one
/// whichever face is listed first (`docs/decisions.md` §336). Two flat faces
/// are still read with the first face listed, so that half is not asked of them.
fn the_spelling_the_pair_is_read_with(
    arm: Arm,
    run: &NamedRun,
    once: &Surface,
    twice: &Surface,
) -> Surface {
    let read = which_two_surfaces_a_run_is_between(run.run(), [once, twice]);
    let Some(written_once) = [*once, *twice].into_iter().find(|spelling| {
        which_two_surfaces_a_run_is_between(run.run(), [spelling, spelling]) == read
    }) else {
        panic!(
            "{arm:?}: one surface written twice was read as {read:?}, which neither spelling \
             written once is read as.\n  the face: {once:?}\n  written again: {twice:?}\n  run: \
             {run:?}"
        );
    };
    if once.as_plane().is_none() {
        assert_eq!(
            read,
            which_two_surfaces_a_run_is_between(run.run(), [twice, once]),
            "{arm:?}: one curved surface written twice was read as a different pair with its \
             two spellings listed the other way round.\n  the face: {once:?}\n  written again: \
             {twice:?}\n  run: {run:?}"
        );
    }
    written_once
}

fn a_run(said: HowTheRunIsSaid, and: Surface, near: Point) -> NamedRun {
    match said {
        HowTheRunIsSaid::RightRoundIt => {
            NamedRun::the_whole_of(EdgeRun::RightRoundWhereItCrosses { and })
                .expect("a closed run carrying its own second surface needs nothing picking it")
        }
        HowTheRunIsSaid::APieceOfIt { facing } => {
            NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                and,
                picked_by: smallvec![Picker::from(
                    HalfSpace::through_a_point_facing(near, a_direction(facing))
                        .expect("a direction with a length")
                )],
            })
            .expect("a run with a half-space on it does not need ends")
        }
    }
}

fn a_surface(which: WhichSurface, at: Point) -> Surface {
    match which {
        WhichSurface::Flat { normal } => Surface::Plane(
            Plane::from_point_and_normal(at, a_direction(normal))
                .expect("a direction has a length"),
        ),
        WhichSurface::Ball { radius } => {
            Surface::Sphere(Sphere::new(at, a_radius(radius)).expect("a positive radius"))
        }
        WhichSurface::Pipe { axis, radius } => Surface::Cylinder(
            Cylinder::new(an_axis(axis, at), a_radius(radius)).expect("a positive radius"),
        ),
        WhichSurface::Taper { axis, slope } => {
            Surface::Cone(Cone::new(an_axis(axis, at), a_slope(slope)).expect("a positive slope"))
        }
        WhichSurface::Ring { axis } => Surface::Torus(
            Torus::new(an_axis(axis, at), 4.0, 1.0).expect("a ring wider than its tube"),
        ),
    }
}

/// The same surface, written again with one stored number a few ulps off.
///
/// A plane is re-spelt through its **offset** rather than through the point it
/// was built from: `from_point_and_normal` turns the point into one offset
/// number, so nudging the point sideways within the plane folds straight back
/// (`the_spellings_a_round_surface_still_has.rs` says why), and
/// [`Plane::moved_to`] is the call that hands the stored normal straight back
/// so only the offset moves. Every curved kind keeps its placement point whole,
/// so those are re-spelt there.
fn the_same_surface_written_again(
    which: WhichSurface,
    at: Point,
    steps: u8,
    outward: bool,
) -> Surface {
    let WhichSurface::Flat { .. } = which else {
        return a_surface(which, a_few_ulps_from(at, steps, outward));
    };
    let Surface::Plane(flat) = a_surface(which, at) else {
        unreachable!("a flat kind builds a plane")
    };
    Surface::Plane(
        flat.moved_to(a_few_ulps_away(flat.offset(), steps, outward))
            .expect("a finite offset"),
    )
}

fn a_few_ulps_from(at: Point, steps: u8, outward: bool) -> Point {
    Point::from(
        at.coordinates()
            .map(|one| a_few_ulps_away(one, steps, outward)),
    )
}

/// `value` moved `steps` steps along the `f64` number line, away from zero when
/// `outward` and towards it when not.
///
/// Worked on the magnitude and given the sign back, because the raw bit pattern
/// of a negative `f64` counts *away* from zero as it rises, so adding to it
/// would move a negative number outward where it moves a positive one outward
/// too — which is right, but only by accident, and reads as if it were wrong.
/// Zero stepped inward saturates and stays zero; zero stepped outward reaches
/// the smallest subnormal there is, which is the harshest second spelling a file
/// can hand over.
fn a_few_ulps_away(value: f64, steps: u8, outward: bool) -> f64 {
    let bits = value.abs().to_bits();
    let moved = if outward {
        bits + u64::from(steps)
    } else {
        bits.saturating_sub(u64::from(steps))
    };
    if value.is_sign_negative() {
        -f64::from_bits(moved)
    } else {
        f64::from_bits(moved)
    }
}

fn a_point(at: [i8; 3]) -> Point {
    Point::from(at.map(f64::from))
}

fn an_axis(which: u8, through: Point) -> Axis {
    Axis::new(through, a_direction(which)).expect("a direction has a length")
}

const DIRECTIONS: [[f64; 3]; 4] = [
    [0.0, 0.0, 1.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [1.0, 1.0, 1.0],
];
const RADII: [f64; 3] = [1.0, 2.0, 4.0];
const SLOPES: [f64; 3] = [0.5, 1.0, 2.0];

#[allow(clippy::cast_possible_truncation)]
const HOW_MANY_DIRECTIONS: u8 = DIRECTIONS.len() as u8;
#[allow(clippy::cast_possible_truncation)]
const HOW_MANY_RADII: u8 = RADII.len() as u8;
#[allow(clippy::cast_possible_truncation)]
const HOW_MANY_SLOPES: u8 = SLOPES.len() as u8;

fn a_direction(which: u8) -> Vector {
    Vector::from(DIRECTIONS[usize::from(which) % DIRECTIONS.len()])
}

fn a_radius(which: u8) -> f64 {
    RADII[usize::from(which) % RADII.len()]
}

fn a_slope(which: u8) -> f64 {
    SLOPES[usize::from(which) % SLOPES.len()]
}

fn configured(cases: u32) -> Config {
    Config {
        cases,
        source_file: Some(file!()),
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource(
            "proptest-regressions",
        ))),
        ..Config::default()
    }
}

/// **The gap, as one case.** A pipe written twice a hair apart, with a run that
/// names the plane its rim lies in.
///
/// §189.3's arm asked for a ball by name — `both_curved && and.as_plane()
/// .is_some() && a_ball.is_some()` — so a pipe, a cone or a torus written twice
/// fell through every arm and the pair either side stood. Two pipes 1e-15
/// apart are not the rim's crossing; the rim is where this pipe meets the plane.
/// §220.4 widened the arm to every curved face.
#[test]
fn a_pipe_written_twice_a_hair_apart_walks_the_circle_its_plane_names() {
    let pipe = Surface::Cylinder(
        Cylinder::new(
            Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0)).expect("z"),
            2.0,
        )
        .expect("a positive radius"),
    );
    let written_again = Surface::Cylinder(
        Cylinder::new(
            Axis::new(Point::new(1e-15, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0)).expect("z"),
            2.0,
        )
        .expect("a positive radius"),
    );
    let rims_plane = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 3.0), Vector::new(0.0, 0.0, 1.0))
            .expect("z"),
    );
    let run = NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
        and: rims_plane,
        picked_by: smallvec![Picker::from(
            HalfSpace::through_a_point_facing(
                Point::new(0.0, 0.0, 3.0),
                Vector::new(0.0, 0.0, 1.0),
            )
            .expect("z")
        )],
    })
    .expect("a run with a half-space on it does not need ends");

    assert_eq!(
        what_the_kernel_says(&run, [&pipe, &written_again]),
        what_the_kernel_says(&run, [&pipe, &pipe]),
        "one pipe written twice 1e-15 apart has to name the same rim the pipe written once names"
    );
}
