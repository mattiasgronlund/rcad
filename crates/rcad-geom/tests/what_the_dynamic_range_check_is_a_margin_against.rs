//! Where a certified answer really stops meaning what the caller asked, and how
//! far below that [`DynamicRange::LARGEST_SUPPORTED_RATIO`] sits.
//!
//! `docs/decisions.md` §15.6 says a published bound needs a test, and §3.5
//! publishes this one: a model's spread is `largest coordinate / smallest
//! feature`, and the check *"fails loudly above roughly 1e11"*. §3.5 defers to
//! the roadmap for why `1e11`, and the roadmap is a plan rather than a
//! description of this codebase (`CLAUDE.md`), so nothing measured held the
//! number up. This file measures it.
//!
//! # What it found
//!
//! **The constant is a margin, not a cliff.** Nothing goes wrong at `1e11`. The
//! spread where a feature a caller writes actually disappears is `2^53`, about
//! `9.0e15` — four and a half decades further out, and
//! [`the_spread_a_written_feature_survives_to`] is what says so.
//!
//! **What goes wrong first is a constructed point, at `2^49`**, and that is
//! still 2 800 times the widest spread the check allows.
//!
//! **The margin is there for conditioning, which a ratio cannot see.** This is
//! the part worth having measured. A construction of conditioning `t` divides
//! the feature by `t` inside its own intermediates, so what matters is not the
//! model's spread but `spread / t` — and the feature is gone once that reaches
//! `2^52`, exactly, at every spread tried.
//! [`conditioning_multiplies_the_spread_and_the_ratio_cannot_see_it`] pins that
//! law. So at the widest spread the check allows, a construction conditioned
//! worse than about `2.2e-5` loses the feature anyway and the check still says
//! yes. §16.11 already says the ratio is blind to things like this; this is the
//! quantity it is blind by.
//!
//! # The failure is a wrong answer, not a refusal
//!
//! Worth saying plainly, because it is the reason the margin is wanted rather
//! than merely tidy. Past the cliff the kernel does not refuse: the offset
//! rounds away, the points it was handed really are coplanar, and `orient3d`
//! answers `Zero` about them — correctly, and about a model that is not the one
//! asked for. A refusal would be safe. This is not, so the check has to fire
//! well before it, and the only question this file settles is how well before.

use rcad_geom::DynamicRange;
use rcad_predicates::{Decision, Point, meet_of_three_planes, orient3d_of_points};

/// Coordinate magnitudes to put the wedge at, so a boundary that is really about
/// the spread cannot pass by being about one place in the number line.
const COORDINATES: [f64; 4] = [1.0, 1024.0, 1048576.0, 1099511627776.0];

/// `2^exponent`, built from the bits so no `powi` rounding enters the sweep.
fn two_to_the(exponent: i32) -> f64 {
    assert!(
        (-1022..=1023).contains(&exponent),
        "2^{exponent} is not a normal f64"
    );
    f64::from_bits(((1023_i64 + i64::from(exponent)) as u64) << 52)
}

/// A wedge at `c` whose three edges off the first corner are each `feature`
/// long, so its `orient3d` is `feature^3` and its true sign never changes.
fn a_wedge(c: f64, feature: f64) -> [[f64; 3]; 4] {
    [
        [c, c, c],
        [c + feature, c, c],
        [c, c + feature, c],
        [c, c, c + feature],
    ]
}

/// The sign the wedge has whenever its apex is still off the base plane.
///
/// Which of the two it is depends on the corner order and is not the claim; that
/// it is the same definite sign at every spread until the feature vanishes is.
const OFF_THE_BASE: Decision = Decision::Neg;

/// The spread at which a feature written straight into the coordinates stops
/// being there at all.
///
/// This is the number `scale.rs` describes when it says the feature becomes
/// smaller than the rounding error on the coordinate, and it is `2^53`: at that
/// spread `c + feature` is `c`, all four corners collapse onto one plane, and
/// the answer flips from a definite sign to `Zero` for good.
const WRITTEN_FEATURE_SURVIVES_TO: i32 = 52;

/// The spread at which a well-conditioned constructed apex stops being
/// decidable, which is earlier than a written one and is the first thing to go.
const CONSTRUCTED_APEX_DECIDES_TO: i32 = 48;

/// A constructed point is supposed to be the first thing to go. If someone
/// re-measures these and that stops being true, the file's argument is upside
/// down and this refuses to build rather than passing quietly.
const _: () = assert!(CONSTRUCTED_APEX_DECIDES_TO < WRITTEN_FEATURE_SURVIVES_TO);

/// The effective spread at which a feature is lost, once conditioning is folded
/// in. See [`conditioning_multiplies_the_spread_and_the_ratio_cannot_see_it`].
const THE_SPREAD_A_FEATURE_DIES_AT: i32 = 52;

#[test]
fn the_spread_a_written_feature_survives_to() {
    let mut still_there = 0;
    let mut gone = 0;

    for spread_exponent in 40..=58 {
        for c in COORDINATES {
            let feature = c / two_to_the(spread_exponent);
            let [a, b, third, apex] = a_wedge(c, feature);
            let decision = rcad_predicates::orient3d(a, b, third, apex);

            if spread_exponent <= WRITTEN_FEATURE_SURVIVES_TO {
                assert_eq!(
                    decision, OFF_THE_BASE,
                    "a wedge at {c} with a feature of {feature}, spread 2^{spread_exponent}, \
                     should still have its apex off the base"
                );
                still_there += 1;
            } else {
                assert_eq!(
                    decision,
                    Decision::Zero,
                    "a wedge at {c} with a feature of {feature}, spread 2^{spread_exponent}, \
                     kept an apex the coordinates cannot hold: {} is {}",
                    c + feature,
                    if c + feature == c { "c" } else { "not c" }
                );
                gone += 1;
            }
        }
    }

    println!(
        "a written feature survives to a spread of 2^{WRITTEN_FEATURE_SURVIVES_TO} and is gone \
         by 2^{}; {still_there} cases either side of it, {gone} past it",
        WRITTEN_FEATURE_SURVIVES_TO + 1
    );
    assert!(
        still_there > 0 && gone > 0,
        "the sweep saw {still_there} surviving and {gone} lost, so it did not cross the boundary \
         it claims to measure"
    );
    assert!(
        DynamicRange::LARGEST_SUPPORTED_RATIO < two_to_the(WRITTEN_FEATURE_SURVIVES_TO),
        "the check allows a spread of {}, which is past where a written feature survives",
        DynamicRange::LARGEST_SUPPORTED_RATIO
    );
}

/// The wedge's apex rebuilt as the meet of three planes, tilted so the division
/// in Cramer's rule is a real one.
fn decide_with_a_constructed_apex(
    c: f64,
    spread_exponent: i32,
    normals: [[f64; 3]; 3],
) -> Decision {
    let feature = c / two_to_the(spread_exponent);
    let [a, b, third, apex] = a_wedge(c, feature);
    let planes = normals.map(|n| {
        let d = -(n[0] * apex[0] + n[1] * apex[1] + n[2] * apex[2]);
        [n[0], n[1], n[2], d]
    });
    let built = meet_of_three_planes(planes[0], planes[1], planes[2])
        .expect("three planes at right angles to nothing meet in a point");
    orient3d_of_points(Point::Exact(a), Point::Exact(b), Point::Exact(third), built)
}

/// A point that arrived through a construction gives up before a written one.
///
/// Level 3 of the cascade is exact for exact inputs and a constructed point is
/// not one (§2.5), so this is the branch that can genuinely refuse — and it
/// refuses `2^4` earlier than the coordinates stop holding the feature. It is
/// still 2 800 times the widest spread the check allows, which is the point.
#[test]
fn a_constructed_apex_gives_up_before_a_written_one_does() {
    const WELL_CONDITIONED: [[f64; 3]; 3] = [[1.0, 0.3, 0.1], [0.2, 1.0, 0.4], [0.15, 0.25, 1.0]];

    let mut decided = 0;
    let mut refused = 0;

    for spread_exponent in 40..=56 {
        for c in COORDINATES {
            let decision = decide_with_a_constructed_apex(c, spread_exponent, WELL_CONDITIONED);
            if spread_exponent <= CONSTRUCTED_APEX_DECIDES_TO {
                assert_eq!(
                    decision, OFF_THE_BASE,
                    "a constructed apex at {c}, spread 2^{spread_exponent}, should still decide"
                );
                decided += 1;
            } else {
                assert!(
                    matches!(decision, Decision::Undecidable { .. }),
                    "a constructed apex at {c}, spread 2^{spread_exponent}, answered {decision:?} \
                     where the box should have straddled the base plane"
                );
                refused += 1;
            }
        }
    }

    println!(
        "a constructed apex decides to a spread of 2^{CONSTRUCTED_APEX_DECIDES_TO} and refuses \
         from 2^{}; {decided} decided, {refused} refused",
        CONSTRUCTED_APEX_DECIDES_TO + 1
    );
    assert!(
        decided > 0 && refused > 0,
        "the sweep saw {decided} decided and {refused} refused, so it did not cross the boundary \
         it claims to measure"
    );
    assert!(
        DynamicRange::LARGEST_SUPPORTED_RATIO < two_to_the(CONSTRUCTED_APEX_DECIDES_TO),
        "the check allows a spread of {}, which is past where a constructed apex still decides",
        DynamicRange::LARGEST_SUPPORTED_RATIO
    );
}

/// What the margin is for: a ratio cannot see how well conditioned a
/// construction is, and conditioning divides straight into the spread.
///
/// The third plane is tilted by `t` away from sharing a line with the other two.
/// Building its offset costs the feature a factor of `t` — the coefficient holds
/// a coordinate-sized sum and the feature's share of it is `t` times smaller —
/// and dividing by `t` afterwards cannot bring back a bit that is gone. So the
/// spread that decides the answer is `spread / t`, and the feature dies once
/// that reaches `2^52`.
///
/// That is checked as an exact prediction rather than a trend: for each spread,
/// the first tilt that loses the apex must be `2^(spread - 52)` on the nose.
#[test]
fn conditioning_multiplies_the_spread_and_the_ratio_cannot_see_it() {
    let mut crossed = 0;

    for spread_exponent in [10, 20, 30, 36] {
        let mut first_lost = None;
        for tilt_exponent in (-53..=0).rev() {
            let tilted = [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, two_to_the(tilt_exponent)],
            ];
            let decision = decide_with_a_constructed_apex(1024.0, spread_exponent, tilted);
            if decision != OFF_THE_BASE {
                first_lost = Some(tilt_exponent);
                break;
            }
        }

        let lost_at = first_lost.unwrap_or_else(|| {
            panic!("spread 2^{spread_exponent} never lost its apex, so this measures nothing")
        });
        assert_eq!(
            lost_at,
            spread_exponent - THE_SPREAD_A_FEATURE_DIES_AT,
            "at a spread of 2^{spread_exponent} the apex went at a tilt of 2^{lost_at}, so the \
             effective spread it died at was 2^{}, not 2^{THE_SPREAD_A_FEATURE_DIES_AT}",
            spread_exponent - lost_at
        );
        crossed += 1;
    }

    assert_eq!(crossed, 4, "only {crossed} spreads were measured");

    let worst_conditioning_the_check_still_covers =
        DynamicRange::LARGEST_SUPPORTED_RATIO / two_to_the(THE_SPREAD_A_FEATURE_DIES_AT);
    println!(
        "at the widest spread the check allows, a construction conditioned worse than \
         {worst_conditioning_the_check_still_covers:e} loses the feature anyway"
    );
    assert!(
        (1e-6..1e-4).contains(&worst_conditioning_the_check_still_covers),
        "the margin covers conditioning down to {worst_conditioning_the_check_still_covers:e}, \
         which is not what scale.rs's doc comment says it is"
    );
}
