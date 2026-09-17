//! A pipe of radius `r` about a line is the points exactly `r` from that line,
//! and which of the infinitely many vectors along the line someone wrote down
//! cannot move it.
//!
//! **This kernel has that property since §118.11**, and `docs/decisions.md`
//! §118.10 is the argument that got it there. `Cylinder::side_of` takes the
//! sign of `|(p − o) × w|² − r²‖w‖²`, degree two in `w` on both sides, so the
//! length the direction was stored at cancels and the surface certified is the
//! pipe the caller named.
//!
//! It compared against a bare `r²` until then, degree zero on the right, so what
//! it certified was the pipe of radius `r / ‖w‖` — and `‖w‖` is only one to
//! within a rounding. `Axis::new` scales a direction to length one and that
//! division rounds; `Axis::transformed`, which `Cylinder::transformed` goes
//! through, does not scale it again at all (§32.3). So two pipes written with
//! the same radius literal were the same width only when their two directions
//! happened to round to the same `‖w‖²`, which is why the 45° pair out of
//! `kicad/Relay_SPDT_Finder_32.21-x000.step` was refused where the 90° pairs
//! were not.
//!
//! **This file was written red**, deliberately, against a kernel that had not
//! been fixed yet. It went green when §118.10's option 1 landed, and it stays
//! here as what pins the property.
//!
//! # The one point every test here asks about, and why it has teeth
//!
//! Every pipe below runs through the coordinate origin along a line lying flat
//! in the `xy` plane, and every question is asked about `(0, 0, r)`.
//!
//! That point is **exactly** `r` from every one of those lines, as real
//! arithmetic and not as a rounding: the `z` axis is square to the `xy` plane,
//! so the distance is the height, whatever direction the line runs. The
//! numbers going in are exact too — the origin is `(0, 0, 0)`, so `p − o` is
//! `(0, 0, r)` down to the bit, with nothing for the subtraction to round.
//! Write it out and the predicate's value is
//!
//! ```text
//! |(0, 0, r) × (wx, wy, 0)|²  =  r²(wx² + wy²)  =  r²‖w‖²,
//! ```
//!
//! exactly, and that is the right-hand side the predicate now subtracts. The two
//! cancel to bit zero, so the answer is `Zero` for every direction and every
//! length it was written at. Against the old bare `r²` the value was
//! `r²(‖w‖² − 1)`, just as exactly, and the cascade reached that sign exactly —
//! so the answer was the sign of `‖w‖² − 1` and **nothing else**, with no
//! rounding in this test for a failure to be blamed on.
//!
//! The teeth are in how tight `Zero` is, and the margin is under one ulp.
//! `one_ulp_either_side_of_the_skin_is_outside_and_inside` shows the
//! neighbouring two `f64` heights are decided `Pos` and `Neg`, so nothing
//! looser than the skin itself is left to ask about. At `r = 0.3` the 45°
//! direction below stores a `‖w‖²` that is `1.8e-16` short of one, which put the
//! surface the kernel certified `r(1/‖w‖ − 1) = 2.7e-17` outside the one that
//! was named — half of the `5.6e-17` ulp. So the skin was the only height of the
//! three whose answer the defect could move, and it moved it. A probe a whole
//! ulp out would sit on the same side of both surfaces and prove nothing at all.
//!
//! # What would make these pass for the wrong reason
//!
//! - **A direction with a `z` component.** Then `(0, 0, r)` is nearer the line
//!   than `r` and `Neg` is the truthful answer, so the test would fail while
//!   agreeing with the geometry. Every pipe here asserts its stored `z` is
//!   zero before it asks anything.
//! - **Checking two spellings against each other instead of against the
//!   geometry.** Scaling a written direction by a small whole number does not
//!   move the stored bits at all — `Vector::scaled_to_length_one_from_anywhere`
//!   divides by the largest component first, and `3·(1, 2, 0) / 6` is exactly
//!   `(1, 2, 0) / 2` — so two spellings can agree perfectly and both be wrong
//!   by the same `1e-16`. Every assertion here is against `Decision::Zero`,
//!   which is what the geometry says, and never against the other spelling.
//! - **A probe point that is not where it is meant to be.** A failure then
//!   looks identical from the outside. Two tests here are the control for it:
//!   `a_pipe_about_an_exactly_unit_direction_already_holds_the_probe_point`
//!   asks the same question of the one direction that stores exactly, and
//!   `one_ulp_either_side_of_the_skin_is_outside_and_inside` brackets the
//!   height. Both passed while the other three were red, so the point is where
//!   it is claimed to be and the only thing that varied in those three was
//!   `‖w‖`.
//! - **An inexact probe point.** At the pipes' own origins in that STEP file,
//!   `10.4 + 0.3` is not an `f64`, so `p − o` would carry an error of `8.9e-16`
//!   — sixteen times the effect being measured, and enough to drown it either
//!   way. The two relay pipes are slid to the coordinate origin with
//!   `Axis::moved_to`, which is a rigid move and keeps the direction bits, and
//!   `the_two_finder_relay_pipes_written_with_one_radius_are_one_width` checks
//!   the bits survived the slide.
//!
//! # A better normalize is not the fix, and the 45° case says so out loud
//!
//! The twelve digits that file writes normalize onto
//! [`f64::consts::FRAC_1_SQRT_2`] — the nearest `f64` there is to `1/√2`, with
//! nothing between it and the real number. Its square doubled is still
//! `1 − 1.8e-16`, because no `f64` squares to a half. So there is no direction
//! for this axis that would store better, and rounding the normalize more
//! carefully could not have closed this. Only making the predicate homogeneous
//! could, and §118.11 did.

// The literals here are a STEP file's own digits and the `f64` they land on,
// and which bits get stored is the whole subject. `FRAC_1_SQRT_2` is named
// where it is the answer rather than a stand-in for one.
#![allow(clippy::approx_constant)]

use core::f64::consts::FRAC_1_SQRT_2;

use rcad_geom::{Axis, Cylinder, Isometry, Point, Vector};
use rcad_predicates::{Decision, Point as PredicatePoint, SideOfSurface};

/// The radius both pipes of §118.10's pair are written with, and the height the
/// probe point sits at.
const RADIUS: f64 = 0.3;

/// The control: a pipe whose stored direction is exactly a unit vector holds
/// the probe point on its skin today.
///
/// `(0, 1, 0)` normalizes to itself, so `‖w‖²` is exactly one and the factor
/// that used to be missing is the number one. This is what told the reds below
/// apart from a probe point built somewhere it was not meant to be: the same
/// point, the same radius and the same arithmetic came out `Zero` here while
/// they failed, so the only thing left varying in the three tests that follow is
/// the length the direction got stored at.
#[test]
fn a_pipe_about_an_exactly_unit_direction_already_holds_the_probe_point() {
    let pipe = a_pipe_about_a_flat_line_through_the_origin(Vector::new(0.0, 1.0, 0.0));
    assert_eq!(
        pipe.axis().direction().components(),
        [0.0, 1.0, 0.0],
        "nothing rounded on the way in",
    );
    assert_eq!(
        pipe.side_of(the_point_one_radius_above_the_origin()),
        Decision::Zero,
    );
}

/// Scaling the vector an axis direction is written with must not move the skin
/// the pipe certifies.
///
/// Whether a factor changes the stored bits at all is the normalize's business
/// — small whole numbers do not, large and fractional ones do — and this test
/// deliberately does not care which happened. Each spelling is asked whether it
/// still holds the point that is exactly a radius from its own axis, and every
/// one of them must say yes.
#[test]
fn a_pipes_skin_does_not_move_when_its_axis_direction_is_written_longer() {
    let flat_lines = [
        Vector::new(0.0, 1.0, 0.0),
        Vector::new(1.0, 2.0, 0.0),
        Vector::new(7.0, 9.0, 0.0),
        Vector::new(0.707_106_781_187, -0.707_106_781_187, 0.0),
    ];
    let lengths = [1.0, 2.0, 3.0, 7.0, 1024.0, 1e-7, 1e12];

    for line in flat_lines {
        for factor in lengths {
            let pipe = a_pipe_about_a_flat_line_through_the_origin(line * factor);
            assert_eq!(
                pipe.side_of(the_point_one_radius_above_the_origin()),
                Decision::Zero,
                "{line:?} written {factor} times longer stores {:?}, and the point a \
                 radius above the origin is on that pipe's skin",
                pipe.axis().direction(),
            );
        }
    }
}

/// §118.10's pair, with the numbers out of
/// `kicad/Relay_SPDT_Finder_32.21-x000.step`.
///
/// `#1146` runs along `(0, 1, 0)`, which is exactly a unit vector, and `#1318`
/// along `(0.707106781187, -0.707106781187, 0)`, which is not and does not
/// normalize to one either. Both are written `0.3` wide. Two pipes written one
/// width are one width, and this kernel makes them two widths a rounding apart
/// — which is why the fuse cannot recognise them as a pair on crossing axes and
/// falls through to the quartic.
///
/// The origins are the file's, and then both pipes are slid to the coordinate
/// origin so the probe point is exact. A translation cannot change how wide a
/// pipe is, and `Axis::moved_to` carries the direction across unchanged — which
/// is asserted here rather than assumed, because the whole question is which
/// bits the direction is stored in.
#[test]
fn the_two_finder_relay_pipes_written_with_one_radius_are_one_width() {
    let square_on =
        Axis::new(Point::new(18.3, -1.19, 10.4), Vector::new(0.0, 1.0, 0.0)).expect("#1146's axis");
    let at_forty_five_degrees = Axis::new(
        Point::new(17.227_867_965_644, 8.597_867_965_644, 10.4),
        Vector::new(0.707_106_781_187, -0.707_106_781_187, 0.0),
    )
    .expect("#1318's axis");

    // The normalize does **not** land on the nearest `f64` to `1/√2`: it comes
    // back one ulp below `FRAC_1_SQRT_2`. That is worth pinning both ways round,
    // because the interesting claim is the second one — a better normalize would
    // not help either, since neither candidate doubles its square to one. So the
    // gap this test is about survives any rounding of the direction, and nobody
    // reading it should reach for a better normalize.
    let stored = at_forty_five_degrees.direction().components();
    assert_eq!(
        stored,
        [FRAC_1_SQRT_2.next_down(), -FRAC_1_SQRT_2.next_down(), 0.0],
        "the normalize lands one ulp below the nearest f64 to 1/√2",
    );
    for candidate in [stored[0], FRAC_1_SQRT_2] {
        assert_ne!(
            candidate * candidate + candidate * candidate,
            1.0,
            "neither the direction stored nor the nearest f64 to 1/√2 doubles \
             its square to one, so rounding the normalize better would move \
             this gap rather than close it: {candidate:?}",
        );
    }

    for axis in [square_on, at_forty_five_degrees] {
        let slid = axis.moved_to(Point::origin()).expect("slid to the origin");
        assert_eq!(
            slid.direction(),
            axis.direction(),
            "sliding a pipe sideways must not touch the bits its direction is stored in",
        );
        let pipe = a_pipe_about_a_flat_line_through_the_origin(slid.direction());
        assert_eq!(
            pipe.side_of(the_point_one_radius_above_the_origin()),
            Decision::Zero,
            "a pipe written {RADIUS} wide about {:?} is {RADIUS} wide",
            axis.direction(),
        );
    }
}

/// Turning a pipe about the `z` axis, and turning it back, must leave its skin
/// where it was.
///
/// `Cylinder::transformed` goes through `Axis::transformed`, which does not
/// scale the turned direction back to length one — deliberately, so that a
/// direction square to something stays square to it (§32.3). So every turn
/// leaves the stored length wherever the turn's arithmetic put it, and a second
/// turn starts from that rather than from one.
///
/// The probe point sits on the `z` axis, which is the line being turned about,
/// so the turn holds it exactly still while carrying the pipe round it — and
/// the pipe's axis stays flat in the `xy` plane, so the point stays exactly a
/// radius from it. Both of those are checked below rather than trusted: the
/// turned origin is still the coordinate origin, and the turned direction still
/// has no `z`.
#[test]
fn turning_a_pipe_and_turning_it_back_leaves_its_skin_where_it_was() {
    let mut pipe = a_pipe_about_a_flat_line_through_the_origin(Vector::new(0.0, 1.0, 0.0));
    assert_eq!(
        pipe.side_of(the_point_one_radius_above_the_origin()),
        Decision::Zero,
        "the pipe starts with an exactly unit direction, so it starts out right",
    );

    for turn in [0.1_f64, -0.1, 0.4, -0.4] {
        let by = Isometry::rotation(&Axis::z(), turn).expect("a turn about z");
        pipe = pipe.transformed(&by);

        assert_eq!(
            pipe.axis().origin(),
            Point::origin(),
            "a turn about z holds the coordinate origin still",
        );
        assert_eq!(
            pipe.axis().direction().components()[2],
            0.0,
            "a turn about z keeps a flat direction flat",
        );
        assert_eq!(
            pipe.side_of(the_point_one_radius_above_the_origin()),
            Decision::Zero,
            "after turning by {turn} the pipe stores {:?} and is still {RADIUS} wide",
            pipe.axis().direction(),
        );
    }
}

/// How tight the `Zero` above is: the two `f64` heights either side of it are
/// already `Pos` and `Neg`.
///
/// This is the evidence that the probe point has teeth. One ulp at `0.3` is
/// `5.6e-17`, and the surface the un-homogeneous predicate certified for the
/// 45° direction was `2.7e-17` off the named one — half an ulp, so the skin was
/// the only height of the three whose answer the defect could change, and it
/// changed it. A probe further out would agree with both surfaces and prove
/// nothing.
///
/// This one passed while the other three were red, and it is here so that a
/// later reader can tell their real green from a green bought by moving the
/// probe somewhere slack.
#[test]
fn one_ulp_either_side_of_the_skin_is_outside_and_inside() {
    let just_outside = f64::from_bits(RADIUS.to_bits() + 1);
    let just_inside = f64::from_bits(RADIUS.to_bits() - 1);

    for line in [
        Vector::new(0.0, 1.0, 0.0),
        Vector::new(0.707_106_781_187, -0.707_106_781_187, 0.0),
    ] {
        let pipe = a_pipe_about_a_flat_line_through_the_origin(line);
        assert_eq!(
            pipe.side_of(PredicatePoint::from([0.0, 0.0, just_outside])),
            Decision::Pos,
            "one ulp above the skin of the pipe about {line:?}",
        );
        assert_eq!(
            pipe.side_of(PredicatePoint::from([0.0, 0.0, just_inside])),
            Decision::Neg,
            "one ulp below the skin of the pipe about {line:?}",
        );
    }
}

/// The point that is exactly [`RADIUS`] away from every line this file builds a
/// pipe about.
fn the_point_one_radius_above_the_origin() -> PredicatePoint {
    PredicatePoint::from([0.0, 0.0, RADIUS])
}

/// A pipe [`RADIUS`] wide about the line through the coordinate origin running
/// along `written`, which has to lie flat in the `xy` plane.
///
/// The flatness is what makes `(0, 0, RADIUS)` exactly a radius from the axis,
/// so it is asserted on the **stored** direction and not on the written one: a
/// normalize that turned an exact zero into something else would otherwise move
/// the skin under the test without saying so.
fn a_pipe_about_a_flat_line_through_the_origin(written: Vector) -> Cylinder {
    let axis = Axis::new(Point::origin(), written).expect("a direction with a length");
    assert_eq!(
        axis.direction().components()[2],
        0.0,
        "{written:?} was meant to store a direction lying flat in the xy plane",
    );
    Cylinder::new(axis, RADIUS).expect("a positive radius")
}
