//! §145.6's third item: **there is no such pair, and the answer that comes out
//! of the annihilation is never unearned.**
//!
//! `quotient_or_everything` (`enclosure.rs`) answers `Interval::ENTIRE` where
//! its divisor touches zero, and promises that the nothing-is-known box "travels
//! through the arithmetic above it and turns into a refusal". `two_lines_told_
//! apart` is the one place it does not: it forms `off_the_picker + leaning *
//! how_far`, and `[0, 0] * ENTIRE` is `[0, 0]`, so an exactly-zero `leaning`
//! swallows a `how_far` that certified nothing. §145 found the one pair known to
//! reach that, §148 removed the route it took, and §148.6 left the question
//! standing: **is there a pair that reaches the annihilation with no second
//! route, so that a sign is certified off a term nothing is known about?**
//!
//! The hypothesis this file was written to attack was that the two are always
//! tied together — that every caller handing over an exact-zero `leaning` also
//! hands over an `up` built by dividing by the same vanished length, so the
//! second route always exists. **That is not the thing that makes the site
//! safe**, and it is stated one step too strongly. What makes it safe is
//! narrower and stronger, and this file demonstrates it in two halves.
//!
//! # The swallow is wrong only where the true `how_far` is not a finite real
//!
//! `leaning` being exactly zero is a rigorous certificate about the two stored
//! triples it dots, so the true `middle` is `off_the_picker + 0 · how_far_true`.
//! Where `how_far_true` is a finite real that product really is zero and `[0,0]`
//! encloses it — the box was wide because the arithmetic gave up, not because
//! the answer is unknown. The swallow can only be wrong where `how_far_true` is
//! **not** a finite real, and `how_far` is a quotient by a stored length, so
//! that means a divisor that is truly zero and not merely a box around zero.
//!
//! So the question is not "when is `how_far` everything" but "when is the length
//! it divides by truly nothing", and there are only two callers to ask.
//!
//! - `two_lines_along_a_pipe` divides by a [`Plane`] normal's squared length,
//!   and a `Plane` has no zero normal to offer
//!   ([`a_plane_normal_never_squares_to_nothing`], §145.4's demonstration re-run
//!   here because §148 did not touch that caller and its argument still holds).
//! - `two_lines_between_two_pipes` divides by `‖towards‖`, the step across to
//!   the other axis, and that is truly zero exactly when `towards` is
//!   `[0, 0, 0]` — at which point `apart` is the zero-width box at zero, every
//!   component of `sideways` is everything, and
//!   [`a_step_across_that_is_nothing_leaves_the_picking_normal_reading_nothing`]
//!   shows the picking normal cannot avoid reading an `up` that is everything.
//!   The second route fires.
//!
//! That closes it over the public API rather than over today's pairs: it is a
//! statement about what the true quotient is, not about which pairs anybody has
//! drawn.
//!
//! # And the annihilation cannot even be reached with `up` left tight
//!
//! The weaker question — reaching `ENTIRE` at all with a finite `up` — was
//! attacked too, and the two routes that work in the arithmetic are both shut
//! at the door. Both need `how_far` emptied by something other than a vanished
//! step, which in `two_lines_between_two_pipes` means emptying `across`'s
//! numerator `apart² + r₁² − r₂²` or overflowing its quotient:
//!
//! - **An overflow in the numerator.** `r² > f64::MAX` makes one of the squares
//!   `[MAX, inf]` and the subtraction everything, with `apart` and `sideways`
//!   and `up` all perfectly tight. But the same overflow empties `E² −
//!   4r₁²r₂²W₁²`, the question that has to be *certified negative* before this
//!   caller is reached at all, so the pair never arrives.
//!   [`a_pipe_too_fat_for_its_own_squared_radius_never_reaches_the_caller`].
//! - **An overflow in the quotient.** Two pipes of the same width whose axes are
//!   almost touching leave `apart² + r₁² − r₂²` straddling zero by one ulp of
//!   `r²`, and dividing that by a very small `apart` could reach out past
//!   `f64::MAX` both ways. It cannot get there: the crossing question needs `E²`
//!   inside `f64`, which caps `r₁² + r₂²` at `√f64::MAX`, and `apart` missing
//!   zero floors the step at about `2.2e-162`.
//!   [`the_widest_numerator_the_crossing_question_allows_cannot_overflow_its_quotient`]
//!   runs the two numbers together: `5.0e299` against `f64::MAX`'s `1.8e308`.
//!
//! Eight decades is a margin and not a proof for all time, which is why the
//! paragraph above it — the one about the true quotient — is the answer and this
//! one is the supporting measurement. If a later change raises what the crossing
//! question can certify, this second half is what would have to be re-run; the
//! first half would not.

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cylinder, Error, HalfSpace, Plane, Point, Surface, SurfaceMeet, Vector, intersect,
};
use rcad_predicates::{Interval, Policy, ProvenanceLog};

/// `[0, 0] * ENTIRE` is `[0, 0]`, which is the whole of what this file is about.
#[test]
fn an_exact_zero_times_a_nothing_is_known_box_swallows_it() {
    let swallowed = Interval::exactly(0.0) * Interval::ENTIRE;
    assert_eq!(
        (swallowed.lo(), swallowed.hi()),
        (0.0, 0.0),
        "if this ever stops being a zero there is nothing left to ask about"
    );

    let merely_containing_zero =
        Interval::new(-1.0, 1.0).expect("a box around zero") * Interval::ENTIRE;
    assert!(
        merely_containing_zero.lo().is_infinite() && merely_containing_zero.hi().is_infinite(),
        "a box that merely contains zero still refuses, so the annihilation \
         needs `leaning` to be an exact zero and nothing wider"
    );
}

/// The second route, demonstrated for every picking normal the kernel can build.
///
/// When the step across to the other axis is exactly `[0, 0, 0]` — the only way
/// the true `how_far` stops being a finite real — `apart` is the zero-width box
/// at zero and every component of `sideways` comes back as everything. What
/// `two_lines_told_apart` reads next is `dot(normal, up)` with
/// `up = (along × sideways) / ‖along‖`, and **that is everything for every
/// normal the picker can hold**, because a normal exactly square to `along`
/// cannot sit on the one axis where the cross product annihilates.
#[test]
fn a_step_across_that_is_nothing_leaves_the_picking_normal_reading_nothing() {
    let mut pickers = 0;
    for along in some_directions() {
        for towards in some_directions().into_iter().chain([Vector::zero()]) {
            let Ok(picker) =
                HalfSpace::through_a_point_square_to(Point::origin(), along, along.cross(towards))
            else {
                continue;
            };
            pickers += 1;
            let reading = what_the_normal_reads_off_up(picker, along);
            assert!(
                reading.lo().is_infinite() && reading.hi().is_infinite(),
                "along {:?}, towards {:?}, normal {:?}: `up` has to come back as \
                 everything wherever the normal reads it, or the annihilation \
                 above it would be signing a term nothing is known about. Read \
                 [{}, {}]",
                along.components(),
                towards.components(),
                picker.normal().components(),
                reading.lo(),
                reading.hi()
            );
        }
    }
    assert!(pickers >= 156, "only {pickers} pickers were built");
}

/// §145.4 re-run: the other caller's divisor is a plane normal's squared length,
/// which never touches zero, so its `how_far` is a quotient by something real
/// and the swallow there is a zero times a finite number. §148 changed only
/// `two_lines_between_two_pipes`, so this argument reads the same after it.
#[test]
fn a_plane_normal_never_squares_to_nothing() {
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
                let squared = squared_length_of(flat.normal());
                assert!(
                    !(squared.lo() <= 0.0 && squared.hi() >= 0.0),
                    "[{a:e}, {b:e}, {c:e}] gave [{}, {}]",
                    squared.lo(),
                    squared.hi()
                );
                planes += 1;
            }
        }
    }
    assert!(planes > 1000, "only {planes} planes were built");
}

/// The first route to an emptied `how_far` with a tight `up`: make a squared
/// radius overflow, and `apart² + r₁² − r₂²` is everything while `apart`,
/// `sideways` and `up` stay exact. The same overflow empties the crossing
/// question, which has to be certified negative first, so no such pair arrives.
#[test]
fn a_pipe_too_fat_for_its_own_squared_radius_never_reaches_the_caller() {
    for power in [155, 160, 200, 250, 300] {
        let radius = 10f64.powi(power);
        let numerator = the_numerator_across_divides(1.0, radius, radius);
        assert!(
            numerator.lo().is_infinite() && numerator.hi().is_infinite(),
            "1e{power} wide: the numerator was supposed to overflow to everything"
        );
        for gap in [1.0, radius, radius * 1.5] {
            let named = what_two_pipes_of_one_width_cross_in(radius, gap);
            assert!(
                matches!(named, Err(Error::Undecidable { .. })),
                "1e{power} wide, axes {gap:e} apart: a pair whose squared radius \
                 overflows must not get past the crossing question, or it would \
                 reach `two_lines_told_apart` with `how_far` empty and `up` \
                 tight. Got {named:?}"
            );
        }
    }
}

/// The widest pipes the crossing question still names, kept as the measurement
/// the bound below is read against.
#[test]
fn the_crossing_question_stops_naming_pipes_between_1e77_and_1e78_wide() {
    let named = what_two_pipes_of_one_width_cross_in(1e77, 1e77);
    assert!(
        matches!(named, Ok(SurfaceMeet::Curves(ref runs)) if runs.len() == 2),
        "1e77 wide, axes 1e77 apart, was expected to be named in two runs: {named:?}"
    );
    let refused = what_two_pipes_of_one_width_cross_in(1e78, 1e78);
    assert!(
        matches!(refused, Err(Error::Undecidable { .. })),
        "1e78 wide was expected to be past what the crossing question can \
         certify: {refused:?}"
    );
}

/// The second route: a numerator that straddles zero by one ulp of `r²`,
/// divided by a step small enough to throw that past `f64::MAX` both ways.
///
/// Two things have to hold at once and they pull apart. The crossing question
/// has to certify `E² − 4r₁²r₂²W₁²` negative before this caller is reached, and
/// `E` is about `r₁² + r₂²` when the axes are nearly touching — so `E²` staying
/// inside `f64` caps `r₁² + r₂²` at `√f64::MAX`, which caps how wide the
/// numerator can be. And `apart` has to miss zero, which floors the step at the
/// smallest length whose square is still a number, about `2.2e-162`. The
/// quotient of the two comes out at `5.0e299` against `f64::MAX`'s `1.8e308` —
/// eight and a half decades of room.
#[test]
fn the_widest_numerator_the_crossing_question_allows_cannot_overflow_its_quotient() {
    let step = the_smallest_step_whose_length_misses_zero();
    assert!(
        step < 1e-161 && step > 1e-163,
        "the smallest step that keeps `apart` off zero moved: {step:e}"
    );

    let widest = f64::MAX.sqrt();
    let radius = (widest / 2.0).sqrt();
    let numerator = the_numerator_across_divides(step, radius, radius);
    let width = numerator.hi() - numerator.lo();
    assert!(
        numerator.lo() < 0.0 && numerator.hi() > 0.0,
        "two pipes of one width whose axes nearly touch leave the numerator \
         straddling zero, or there is no overflow to chase: [{}, {}]",
        numerator.lo(),
        numerator.hi()
    );

    let reach = width / (2.0 * step);
    assert!(
        reach < f64::MAX / 1e8,
        "the widest numerator the crossing question allows, over the smallest \
         step that keeps `apart` off zero, reaches {reach:e} — within eight \
         decades of f64::MAX, so `across` may now overflow to everything with \
         `up` left tight and this file's second half no longer holds"
    );

    let mut worst: f64 = 0.0;
    for shrink in 0..8 {
        for stretch in 0..8 {
            let radius = radius / 10f64.powi(shrink * 3);
            let step = step * 10f64.powi(stretch * 20);
            let numerator = the_numerator_across_divides(step, radius, radius);
            let reach = (numerator.hi() - numerator.lo()) / (2.0 * step);
            assert!(
                reach.is_finite(),
                "{radius:e} wide, axes {step:e} apart: the quotient reached \
                 {reach:e}"
            );
            worst = worst.max(reach);
        }
    }
    assert!(
        worst <= width / (2.0 * step),
        "the widest pipes with the smallest step were supposed to be the worst \
         case, and {worst:e} beats it"
    );
}

/// An ordinary pair, so that the file cannot pass by never reaching the caller
/// at all.
#[test]
fn two_ordinary_pipes_side_by_side_are_still_named_in_two_runs() {
    let named = what_two_pipes_of_one_width_cross_in(1.0, 1.5);
    assert!(
        matches!(named, Ok(SurfaceMeet::Curves(ref runs)) if runs.len() == 2),
        "two ordinary pipes: {named:?}"
    );
}

/// What `two_lines_told_apart` reads off `up` when every component of
/// `sideways` came back as everything, mirroring the caller one line at a time.
fn what_the_normal_reads_off_up(picker: HalfSpace, along: Vector) -> Interval {
    let sideways = [Interval::ENTIRE; 3];
    let squared_direction = squared_length_of(along);
    let length = root_of(squared_direction);
    let up: [Interval; 3] = core::array::from_fn(|axis| {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        let part = Interval::exactly(along.components()[next]) * sideways[after]
            - Interval::exactly(along.components()[after]) * sideways[next];
        quotient_or_everything(part, length)
    });
    let normal = picker.normal().components();
    (0..3).fold(Interval::exactly(0.0), |so_far, axis| {
        so_far + Interval::exactly(normal[axis]) * up[axis]
    })
}

/// `over / under`, or everything where `under` touches zero — mirroring
/// `enclosure.rs`. Only two numerators can reach it here, an exact zero and
/// everything, and neither needs a division to be worked out.
fn quotient_or_everything(over: Interval, under: Interval) -> Interval {
    if under.lo() <= 0.0 && under.hi() >= 0.0 {
        return Interval::ENTIRE;
    }
    if over.lo().is_infinite() && over.hi().is_infinite() {
        return Interval::ENTIRE;
    }
    assert!(
        over.lo() == 0.0 && over.hi() == 0.0,
        "a cross product of an exact direction with three unknowns is either \
         everything or an exact zero, and this is neither: [{}, {}]",
        over.lo(),
        over.hi()
    );
    over
}

/// `apart² + r₁² − r₂²`, the numerator `across` divides by twice the step.
fn the_numerator_across_divides(step: f64, mine: f64, theirs: f64) -> Interval {
    let apart = root_of(squared_length_of(Vector::new(step, 0.0, 0.0)));
    apart * apart + Interval::exactly(mine) * Interval::exactly(mine)
        - Interval::exactly(theirs) * Interval::exactly(theirs)
}

/// The smallest step along one axis whose length still comes back as a box that
/// misses zero, which is the floor on `apart` in every pair that reaches the
/// caller with `up` intact.
fn the_smallest_step_whose_length_misses_zero() -> f64 {
    let mut step = 1.0;
    loop {
        let shorter = step / 2.0;
        let length = root_of(squared_length_of(Vector::new(shorter, 0.0, 0.0)));
        if length.lo() <= 0.0 || shorter == 0.0 {
            return step;
        }
        step = shorter;
    }
}

fn squared_length_of(what: Vector) -> Interval {
    what.components()
        .iter()
        .fold(Interval::exactly(0.0), |so_far, part| {
            so_far + Interval::exactly(*part) * Interval::exactly(*part)
        })
}

fn root_of(squared: Interval) -> Interval {
    Interval::new(squared.lo().max(0.0).sqrt(), squared.hi().sqrt()).expect("a root")
}

/// Directions worth asking about: the three coordinate axes, which are where the
/// cross product annihilates a component, and a few that lean.
fn some_directions() -> Vec<Vector> {
    let mut directions = Vec::new();
    for parts in [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [-1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.3, 0.5, 0.7],
        [1e-8, 1.0, 0.0],
        [1e160, 1.0, 1e-160],
        [1.0, -2.0, 3.0],
    ] {
        directions.push(Vector::new(parts[0], parts[1], parts[2]));
    }
    directions
}

/// Two pipes of the same width, axes along `z`, `gap` apart.
fn what_two_pipes_of_one_width_cross_in(radius: f64, gap: f64) -> Result<SurfaceMeet, Error> {
    let pair = [Point::origin(), Point::new(gap, 0.0, 0.0)].map(|origin| {
        Surface::Cylinder(
            Cylinder::new(
                Axis::new(origin, Vector::new(0.0, 0.0, 1.0)).expect("an axis"),
                radius,
            )
            .expect("a pipe"),
        )
    });
    intersect(
        &pair[0],
        &pair[1],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}
