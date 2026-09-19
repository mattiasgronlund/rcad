//! §145 closed: the step from one pipe's axis across to the other is worked out
//! scaled by `w·w`, so it no longer rounds away to nothing.
//!
//! `two_lines_between_two_pipes` used to take that step as it reads —
//! `(c − o) − w·((c − o)·w / w·w)` — and that is a subtraction of two numbers
//! that agree to everything the answer is made of. Put the second pipe's origin
//! on `w` scaled a long way out and rounded onto the `f64` grid, and it cancels
//! **exactly**: the step is `[0, 0, 0]`, the length everything below divides by
//! is the zero-width box at zero, and a pair of ordinary pipes that cross in two
//! straight runs comes back unnameable both ways round.
//!
//! The same step multiplied by `w·w` is `(w × (c − o)) × w`, a polynomial in the
//! stored numbers with nothing to cancel, which `Exact` holds outright. The
//! scale comes back off with one division per component — a division by a number
//! near one, not a cancellation — and what is handed on is the true step to
//! within an ulp or two of itself.
//!
//! # What moved
//!
//! Eighteen reaches from `1` to `1e17`, each with the second origin at `w` times
//! the reach rounded onto the grid and then nudged along `z` by −2, −1, 0, +1
//! and +2 ulps, each pair asked both ways round — ninety answers. **Sixteen of
//! them changed and no others did.** Every one of the sixteen is a pair whose
//! plain step rounds to exactly zero, and every one went from
//! `CannotNameWhichPieceOfACrossingThisIs` both ways round to two straight runs
//! both ways round — which is what the pairs one and two ulps either side of it
//! answered all along, before the change as well as after.
//!
//! The pair at reach `1e16` nudged `+2` answers two runs one way round and
//! refuses the other. It reads the same before the change and after it, so it is
//! not this one's; the two pipes there are a hair off touching, which is a
//! different question from this one.

#![forbid(unsafe_code)]

use rcad_geom::{Axis, Cylinder, Error, Point, Surface, SurfaceMeet, Vector, intersect};
use rcad_predicates::{Policy, ProvenanceLog};

/// Which way both pipes run, before it is scaled to length one.
const WAY: [f64; 3] = [0.3, 0.5, 0.7];

/// Both pipes are this wide, so a pair whose axes could not be told apart would
/// come back as one surface rather than as a refusal.
const WIDE: f64 = 1.0;

/// How far out along the shared direction §145's own pair puts the second axis.
const REACH: f64 = 1e10;

#[test]
fn two_pipes_whose_step_across_rounds_away_are_named_in_two_runs() {
    let pair = two_pipes_reaching(REACH, 0);
    assert_eq!(
        the_step_the_plain_way(&pair),
        Vector::zero(),
        "the plain subtraction has to cancel, or this pair is not the one §145 \
         is about"
    );
    for first in [0, 1] {
        assert_eq!(named_runs_in(&pair, first), 2, "pipe {first} asked first");
    }
}

#[test]
fn every_reach_whose_step_rounds_away_is_named_in_two_runs() {
    let mut rounded_away = 0;
    for power in 1..=17 {
        let pair = two_pipes_reaching(10f64.powi(power), 0);
        if the_step_the_plain_way(&pair) != Vector::zero() {
            continue;
        }
        rounded_away += 1;
        for first in [0, 1] {
            assert_eq!(
                named_runs_in(&pair, first),
                2,
                "1e{power} out, pipe {first} asked first"
            );
        }
    }
    assert!(
        rounded_away >= 12,
        "only {rounded_away} of the seventeen reaches cancelled to nothing, so \
         this test has stopped asking about §145's case at all"
    );
}

#[test]
fn a_nudge_of_an_ulp_either_way_no_longer_changes_the_answer() {
    for nudge in [-2, -1, 0, 1, 2] {
        let pair = two_pipes_reaching(REACH, nudge);
        for first in [0, 1] {
            assert_eq!(
                named_runs_in(&pair, first),
                2,
                "{nudge} ulps along, pipe {first} asked first"
            );
        }
    }
}

/// Two pipes side by side with nothing awkward about them, so that the reach
/// sweep above cannot be the only thing holding the new step up.
///
/// **It does not pin which way the step points**, which is what it was written
/// to do. Turning the step round turns `w × step` with it, and the picking
/// half-space leans along `w × step` while `up` *is* `w × step` — so both flip
/// together, the two runs come back in the other order, and every test here
/// stays green. Taking the gap the other way round in
/// `the_step_across_to_the_other_axis` and running this file is the measurement.
#[test]
fn two_ordinary_pipes_side_by_side_are_still_named_in_two_runs() {
    let pair = [Point::origin(), Point::new(0.0, 1.5, 0.0)].map(|origin| {
        Surface::Cylinder(
            Cylinder::new(
                Axis::new(origin, Vector::new(1.0, 0.0, 0.0)).expect("an axis"),
                WIDE,
            )
            .expect("a pipe"),
        )
    });
    for first in [0, 1] {
        assert_eq!(named_runs_in(&pair, first), 2, "pipe {first} asked first");
    }
}

/// The pair, with the second axis origin `reach` along the shared direction,
/// rounded onto the grid, and then nudged along `z` by `nudge` ulps.
fn two_pipes_reaching(reach: f64, nudge: i64) -> [Surface; 2] {
    let way = Vector::new(WAY[0], WAY[1], WAY[2]);
    let here = Point::origin();
    let [x, y, z] = (Axis::new(here, way).expect("an axis").direction() * reach).components();
    let z = f64::from_bits(z.to_bits().wrapping_add_signed(nudge));
    [here, Point::new(x, y, z)].map(|origin| {
        Surface::Cylinder(
            Cylinder::new(Axis::new(origin, way).expect("an axis"), WIDE).expect("a pipe"),
        )
    })
}

/// The step across to the other axis, worked out the way that cancels.
fn the_step_the_plain_way(pair: &[Surface; 2]) -> Vector {
    let (Surface::Cylinder(one), Surface::Cylinder(other)) = (&pair[0], &pair[1]) else {
        panic!("two pipes");
    };
    let along = one.axis().direction();
    let between = other.axis().origin() - one.axis().origin();
    between - along * (between.dot(along) / along.length_squared())
}

/// How many straight runs the pair is named in, with `first` written first.
fn named_runs_in(pair: &[Surface; 2], first: usize) -> usize {
    let met: Result<SurfaceMeet, Error> = intersect(
        &pair[first],
        &pair[1 - first],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    let Ok(SurfaceMeet::Curves(runs)) = met else {
        panic!("not named in runs at all: {met:?}");
    };
    runs.len()
}
