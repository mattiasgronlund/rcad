//! §142.7's annihilation, drawn: two pipes whose step from one axis to the
//! other rounds away to nothing while the axes stay certifiably apart.
//!
//! `quotient_or_everything` (`enclosure.rs`) answers `Interval::ENTIRE` where
//! its divisor touches zero, and its doc comment used to promise that the
//! nothing-is-known box "travels through the arithmetic above it and turns into
//! a refusal". §140's fix built the one place where it does not:
//! `two_lines_told_apart` forms `off_the_picker + leaning * how_far`, and
//! `[0, 0] * ENTIRE` is `[0, 0]` rather than `ENTIRE`. §142.7 wrote the
//! measurement down and said what it could not say — whether any pair a caller
//! can draw arrives with `how_far` being `ENTIRE` and `leaning` an exact zero at
//! the same time. This pair does.
//!
//! # How a pair reaches it
//!
//! `two_lines_between_two_pipes` steps from its own axis to the other one with
//!
//! ```text
//! towards = between − along · (between·along / along·along)
//! ```
//!
//! in plain `f64`, and everything it hands `two_lines_told_apart` is built from
//! that step: `how_far` divides by its length, and `leaning` dots the picking
//! normal against it. Put the second pipe's origin on a point the first axis
//! nearly passes through — `along` scaled by `1e10` and rounded componentwise —
//! and the subtraction cancels **exactly**. The step is `[0, 0, 0]`, so
//!
//! - its length is the zero-width box at zero, which `quotient_or_everything`
//!   reads as a divisor touching zero, so `how_far` comes back as everything;
//!   and
//! - `leaning` is a dot product against `[0, 0, 0]`, which the exact level sums
//!   to an exact zero whatever the picking normal turned out to be.
//!
//! The axes are still apart. Rounding `along · 1e10` onto the `f64` grid leaves
//! `between × along` at `[-4.76837158203125e-7, 4.76837158203125e-7, 0]`, so the
//! two axes are `6.743495761743046e-7` apart — which the exact level of the
//! `WHERE_TWO_PIPES_CROSS` question sees and certifies positive — the pair would
//! come back as one surface rather than as a refusal if it did not, because both
//! pipes are one width. That is the asymmetry §142.7 asked about: the exact arm
//! keeps the separation and the enclosure arm loses it.
//!
//! # And it changes nothing
//!
//! The annihilation is reached and the answer is still a refusal, by a second
//! route rather than the promised one. The same zero-length step that empties
//! `how_far` is also what the sideways direction is scaled by, so `up` is
//! everything, so the either-side term is everything, and neither side of the
//! picking plane can be signed. In this caller `how_far` being everything
//! **always** drags `up` with it, because both come from dividing by the length
//! of the one step.
//!
//! # The refusal is a gap and not a limit
//!
//! Move the second axis one ulp along `z` either way and the same pair is named
//! in two straight runs, which is what
//! [`a_pair_of_pipes_one_ulp_either_side_of_it_is_named_in_two_runs`] holds. The
//! answer exists and the arithmetic could not reach it, which is §16.10's test
//! for a gap. Reported, not fixed — `intersect.rs` is not this lane's to write.

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cylinder, Error, HalfSpace, Plane, Point, Surface, SurfaceMeet, Vector, intersect,
};
use rcad_predicates::{Interval, Policy, ProvenanceLog};

/// Which way both pipes run, before it is scaled to length one.
const WAY: [f64; 3] = [0.3, 0.5, 0.7];

/// The second pipe's axis origin: the first pipe's direction times `1e10`,
/// rounded onto the `f64` grid one component at a time.
const THERE: [f64; 3] = [3292927799.69071, 5488212999.484517, 7683498199.278325];

/// Both pipes are this wide, so a pair whose axes could not be told apart would
/// come back as one surface rather than as a refusal.
const WIDE: f64 = 1.0;

#[test]
fn the_step_between_two_pipe_axes_can_round_to_nothing_while_the_axes_stay_apart() {
    let pair = two_pipes_nudged_by(0);
    let (Surface::Cylinder(one), Surface::Cylinder(other)) = (&pair[0], &pair[1]) else {
        panic!("two pipes");
    };

    let here = one.axis().origin();
    let along = one.axis().direction();
    let between = other.axis().origin() - here;
    let towards = between - along * (between.dot(along) / along.length_squared());
    assert_eq!(
        towards,
        Vector::zero(),
        "the step from one axis to the other has to round away, or `leaning` is \
         not an exact zero and `how_far` is not everything"
    );
    assert_ne!(
        between.cross(along),
        Vector::zero(),
        "the axes have to stay apart, or the pair is one surface and never \
         reaches the product at all"
    );

    let picker =
        HalfSpace::through_a_point_square_to(here, along, along.cross(towards)).expect("a picker");
    let [x, y, z] = along.components();
    let exactly_square = [
        Vector::new(0.0, z, -y),
        Vector::new(-z, 0.0, x),
        Vector::new(y, -x, 0.0),
    ];
    assert!(
        exactly_square.contains(&picker.normal()),
        "the picking normal is one of the three that are exactly square to the \
         axis, so `two_lines_told_apart` gets past its first gate and reaches \
         the product: {:?}",
        picker.normal()
    );

    let annihilated = Interval::exactly(0.0) * Interval::ENTIRE;
    assert_eq!(
        (annihilated.lo(), annihilated.hi()),
        (0.0, 0.0),
        "an exact zero times a nothing-is-known box is a zero, which is what \
         `leaning * how_far` does here"
    );

    for first in [0, 1] {
        let met = what_the_pair_crosses_in(&pair, first);
        assert!(
            matches!(met, Err(Error::CannotNameWhichPieceOfACrossingThisIs)),
            "pipe {first} asked first: two pipes of one width whose axes could \
             not be told apart would be one surface, and a pair that never \
             reached `two_lines_told_apart` would be named or empty — so this \
             refusal is what says the product was formed, and that it certified \
             nothing: {met:?}"
        );
    }
}

#[test]
fn a_pair_of_pipes_one_ulp_either_side_of_it_is_named_in_two_runs() {
    for steps in [-2, -1, 1, 2] {
        let pair = two_pipes_nudged_by(steps);
        for first in [0, 1] {
            let met = what_the_pair_crosses_in(&pair, first);
            let Ok(SurfaceMeet::Curves(runs)) = met else {
                panic!("{steps} ulps along, pipe {first} first, was not named: {met:?}");
            };
            assert_eq!(runs.len(), 2, "{steps} ulps along, pipe {first} first");
        }
    }
}

#[test]
fn a_plane_and_a_pipe_never_reach_the_product_because_a_plane_normal_is_one_long() {
    let wild = [
        0.0,
        1.0,
        -1.0,
        3.0,
        1e-8,
        1e-160,
        1e160,
        1e-300,
        1e300,
        f64::MIN_POSITIVE,
        5e-324,
    ];
    let mut planes = 0;
    for a in wild {
        for b in wild {
            for c in wild {
                let Ok(flat) = Plane::from_coefficients([a, b, c, 1.0]) else {
                    continue;
                };
                let [x, y, z] = flat.normal().components();
                let squared = Interval::exactly(0.0)
                    + Interval::exactly(x) * Interval::exactly(x)
                    + Interval::exactly(y) * Interval::exactly(y)
                    + Interval::exactly(z) * Interval::exactly(z);
                assert!(
                    !squared.contains_zero(),
                    "a plane's normal squares to a box away from zero, which is \
                     what stops `two_lines_along_a_pipe` dividing by nothing: \
                     [{a:e}, {b:e}, {c:e}] gave [{}, {}]",
                    squared.lo(),
                    squared.hi()
                );
                planes += 1;
            }
        }
    }
    assert!(planes > 1000, "only {planes} planes were built");
}

/// The pair, with the second axis origin nudged along `z` by `steps` ulps.
fn two_pipes_nudged_by(steps: i64) -> [Surface; 2] {
    let way = Vector::new(WAY[0], WAY[1], WAY[2]);
    let here = Point::origin();
    let z = f64::from_bits(THERE[2].to_bits().wrapping_add_signed(steps));
    let there = Point::new(THERE[0], THERE[1], z);
    [
        Surface::Cylinder(
            Cylinder::new(Axis::new(here, way).expect("an axis"), WIDE).expect("a pipe"),
        ),
        Surface::Cylinder(
            Cylinder::new(Axis::new(there, way).expect("an axis"), WIDE).expect("a pipe"),
        ),
    ]
}

/// What the kernel makes of the pair, asked with `first` written first.
fn what_the_pair_crosses_in(pair: &[Surface; 2], first: usize) -> Result<SurfaceMeet, Error> {
    intersect(
        &pair[first],
        &pair[1 - first],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}
