//! The search behind [`NEAREST_F64_POINT_ROUNDOFF`] (`docs/decisions.md`
//! §15.6).
//!
//! The bound says a point that truly lies on a curved surface, written down as
//! three `f64`, lands within `NEAREST_F64_POINT_ROUNDOFF` times its largest
//! coordinate of that surface. What would catch it being too small is a search
//! for the worst case, and the worst case is constructed rather than guessed:
//! step one coordinate through neighbouring `f64` until the sign the surface
//! gives flips. The surface passes between those two, so **both of them are
//! within one coordinate spacing of it and no `f64` on that line is nearer** —
//! which is exactly what a corpus file's corner is when the writer rounded
//! correctly.
//!
//! Four things are asserted per bracket:
//!
//! - **The pair really is off the surface.** Handed over as exact `f64`
//!   triples, both come back with a certified sign. Without that this file
//!   would be checking a budget nothing needed.
//! - **The bound covers them.** Widened into a box of the published half-width,
//!   neither can be certified off any more.
//! - **It does not cover a corner somebody chose.** The same box around a point
//!   a thousand spacings further out still certifies a side, so the number is a
//!   width and not a tolerance.
//! - **The gap the surface fell in fits inside the budget**, which is the
//!   derivation itself, and the run prints what share of it a spacing filled.
//!   The worst is between a half and a whole, so halving the published number
//!   would fail this file rather than quietly refusing corpus files again.
//!
//! Together those keep it from passing for a silly reason: a bound big enough
//! to swallow everything fails the third, a bound of zero fails the second, and
//! a bound many times wider than a spacing fails the fourth.

#![forbid(unsafe_code)]

use rcad_geom::{Axis, Cone, Cylinder, NEAREST_F64_POINT_ROUNDOFF, Point, Sphere, Torus, Vector};
use rcad_predicates::{Decision, Interval, Point as PredicatePoint, SideOfSurface};

/// How many spacings out a corner has to be before this file calls it a corner
/// somebody chose rather than one that rounded.
///
/// Well clear of the bound, so the refusing half is not measuring the same edge
/// twice, and still far below the smallest modelling tolerance a real file
/// carries — `refusal.rs` puts that tail in the thousands of spacings.
const A_MISS_NOBODY_COULD_BLAME_ON_ROUNDING: f64 = 1000.0;

/// How much of the published budget one coordinate spacing has to fill.
///
/// The other side of §15.6: a bound nothing can get near is untested. The
/// derivation is that a coordinate spacing fits inside the budget, so the
/// spacing as a share of the budget is the figure to watch — over one and the
/// number is too small, near zero and it is far wider than the rounding it is
/// meant to be and the search has stopped being evidence for it.
const THE_SHARE_A_SPACING_HAS_TO_FILL: f64 = 0.25;

#[test]
fn the_nearest_f64_to_a_point_on_a_ball_is_never_certified_off_it() {
    let mut needed = Vec::new();
    for scale in scales() {
        let ball = Sphere::new(
            Point::new(0.3 * scale, -1.7 * scale, 2.9 * scale),
            4.75 * scale,
        )
        .expect("a ball with a positive radius");
        let along = |height: f64| PredicatePoint::from([1.534 * scale, 0.645 * scale, height]);
        needed.extend(check_the_bracket_across(
            &ball,
            &along,
            2.9 * scale,
            20.0 * scale,
        ));
    }
    report("a ball", needed);
}

#[test]
fn the_nearest_f64_to_a_point_on_a_pipe_is_never_certified_off_it() {
    let mut needed = Vec::new();
    for scale in scales() {
        let pipe = Cylinder::new(
            Axis::new(
                Point::new(1.1 * scale, -2.2 * scale, 0.5 * scale),
                Vector::new(1.0, 2.0, 3.0),
            )
            .expect("an axis with a direction"),
            3.25 * scale,
        )
        .expect("a pipe with a positive radius");
        let along = |across: f64| PredicatePoint::from([across, -2.2 * scale, 0.5 * scale]);
        needed.extend(check_the_bracket_across(
            &pipe,
            &along,
            1.1 * scale,
            30.0 * scale,
        ));
    }
    report("a pipe", needed);
}

#[test]
fn the_nearest_f64_to_a_point_on_a_cone_is_never_certified_off_it() {
    let mut needed = Vec::new();
    for scale in scales() {
        let cone = Cone::new(
            Axis::new(
                Point::new(0.5 * scale, 0.5 * scale, 0.5 * scale),
                Vector::new(0.0, 0.0, 1.0),
            )
            .expect("an axis with a direction"),
            0.7,
        )
        .expect("a cone with a positive slope");
        // Far enough up the axis that the cone is wide there, which is where
        // the value the predicate decides is largest and a fixed budget in the
        // predicate's own units would have been worst.
        let along = |across: f64| PredicatePoint::from([across, 0.5 * scale, 9.5 * scale]);
        needed.extend(check_the_bracket_across(
            &cone,
            &along,
            0.5 * scale,
            30.0 * scale,
        ));
    }
    report("a cone", needed);
}

#[test]
fn the_nearest_f64_to_a_point_on_a_ring_is_never_certified_off_it() {
    let mut needed = Vec::new();
    for scale in scales() {
        let ring = Torus::new(
            Axis::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
                .expect("an axis with a direction"),
            5.3 * scale,
            1.7 * scale,
        )
        .expect("a ring wider than its tube");
        let along = |across: f64| PredicatePoint::from([across, 0.0, 0.4 * scale]);
        needed.extend(check_the_bracket_across(
            &ring,
            &along,
            5.3 * scale,
            20.0 * scale,
        ));
    }
    report("a ring", needed);
}

/// The sizes the same shape is asked at.
///
/// None is a power of two, so nothing in the arithmetic cancels for free, and
/// they span nine orders of magnitude — the bound is relative, so a size it
/// held at only one of them would be a bound about that size and not about
/// `f64`.
fn scales() -> Vec<f64> {
    vec![1.0e-3, 0.375, 1.0, 6.7, 254.0, 1.0e6]
}

/// Walk `along` from `inside` outwards, find the two neighbouring `f64` the
/// surface passes between, and put the three questions of the file header to
/// them.
///
/// Hands back what share of the budget the coordinate spacing at the crossing
/// filled, and what share the cascade actually had to be given before it
/// stopped certifying, or nothing when the walk found no crossing.
fn check_the_bracket_across(
    surface: &impl SideOfSurface,
    along: &impl Fn(f64) -> PredicatePoint,
    inside: f64,
    outside: f64,
) -> Option<(f64, f64)> {
    let (nearer, further) = where_the_surface_crosses(surface, along, inside, outside)?;

    let mut asked_for: f64 = 0.0;
    for reading in [nearer, further] {
        let at = along(reading);
        let certified = matches!(surface.side_of(at), Decision::Pos | Decision::Neg);
        assert!(
            certified,
            "the pair bracketing the surface has to be off it for this file to be \
             checking anything, and {reading} was not"
        );

        let budget = NEAREST_F64_POINT_ROUNDOFF * largest_coordinate(at);
        let decision = surface.side_of(widened_by(at, budget));
        assert!(
            !matches!(decision, Decision::Pos | Decision::Neg),
            "the surface passes between two neighbouring f64 and {reading} is one of \
             them, so no writer could have put a corner nearer; a box of {budget} \
             round it still came back {decision:?}"
        );
        asked_for = asked_for.max(the_share_of_the_budget_it_asked_for(surface, at, budget));
    }

    let at = along(nearer);
    let budget = NEAREST_F64_POINT_ROUNDOFF * largest_coordinate(at);
    let spacing = spacing_at(nearer);
    let chosen = along(nearer + A_MISS_NOBODY_COULD_BLAME_ON_ROUNDING * spacing);
    let decision = surface.side_of(widened_by(chosen, budget));
    assert!(
        matches!(decision, Decision::Pos | Decision::Neg),
        "a corner {A_MISS_NOBODY_COULD_BLAME_ON_ROUNDING} spacings off the surface is \
         one somebody chose, and the box round it has to still say which side: \
         {decision:?}"
    );

    assert!(
        spacing <= budget,
        "the surface fell in a gap of {spacing} between two neighbouring f64, and the \
         budget of {budget} is meant to cover a whole one"
    );
    Some((spacing / budget, asked_for))
}

/// What the four tests print and check.
///
/// The first figure is the derivation: one coordinate spacing as a share of the
/// budget, which has to be under one for the bound to hold and well above zero
/// for the search to be evidence for it. The second is how much of the budget
/// the cascade actually asked for, and it is printed rather than checked —
/// widening a corner at all costs a pipe, a cone and a ring their exact level,
/// and the interval arithmetic that is left already loses about a spacing on
/// these shapes, so for three of the four the budget is a floor rather than the
/// binding constraint.
fn report(surface: &str, measured: Vec<(f64, f64)>) {
    let worst = measured
        .iter()
        .map(|&(spacing, _)| spacing)
        .fold(0.0, f64::max);
    let asked_for = measured
        .iter()
        .map(|&(_, asked_for)| asked_for)
        .fold(0.0, f64::max);
    println!(
        "{surface}: {} brackets, a spacing filled up to {worst:.3} of the budget, and the \
         cascade asked for up to {asked_for:.3} of it",
        measured.len()
    );
    assert_eq!(
        measured.len(),
        scales().len(),
        "a scale gave no bracket, so this run checked less than it says"
    );
    assert!(
        worst >= THE_SHARE_A_SPACING_HAS_TO_FILL,
        "a spacing filled only {worst:.3} of the budget, so the budget is far wider than \
         the rounding it is meant to be and this search is no longer evidence for it"
    );
}

/// The smallest share of `budget` that stops `surface` certifying a side of
/// `at`, to within a part in a thousand billion.
fn the_share_of_the_budget_it_asked_for(
    surface: &impl SideOfSurface,
    at: PredicatePoint,
    budget: f64,
) -> f64 {
    let (mut too_small, mut enough) = (0.0f64, 1.0f64);
    for _ in 0..40 {
        let middle = f64::midpoint(too_small, enough);
        let decision = surface.side_of(widened_by(at, middle * budget));
        if matches!(decision, Decision::Pos | Decision::Neg) {
            too_small = middle;
        } else {
            enough = middle;
        }
    }
    enough
}

/// The two neighbouring `f64` readings the surface passes between, nearer end
/// of the walk first.
///
/// `inside` and `outside` have to come back with opposite certified signs;
/// bisecting on the bit pattern then closes on a pair with nothing between them.
fn where_the_surface_crosses(
    surface: &impl SideOfSurface,
    along: &impl Fn(f64) -> PredicatePoint,
    inside: f64,
    outside: f64,
) -> Option<(f64, f64)> {
    let side = |reading: f64| surface.side_of(along(reading));
    let (low, high) = (side(inside), side(outside));
    if !matches!(
        (low, high),
        (Decision::Neg, Decision::Pos) | (Decision::Pos, Decision::Neg)
    ) {
        return None;
    }

    let (mut low_key, mut high_key) = (in_reading_order(inside), in_reading_order(outside));
    while high_key - low_key > 1 {
        let middle = low_key + (high_key - low_key) / 2;
        if side(from_reading_order(middle)) == low {
            low_key = middle;
        } else {
            high_key = middle;
        }
    }
    Some((from_reading_order(low_key), from_reading_order(high_key)))
}

/// The gap between `reading` and the next `f64` above it.
fn spacing_at(reading: f64) -> f64 {
    from_reading_order(in_reading_order(reading) + 1) - reading
}

/// `f64` bit patterns rearranged so that comparing them as integers orders the
/// numbers themselves, which is what lets the bisection above step one `f64` at
/// a time.
fn in_reading_order(reading: f64) -> i64 {
    let bits = reading.to_bits() as i64;
    if bits < 0 {
        i64::MIN.wrapping_sub(bits)
    } else {
        bits
    }
}

fn from_reading_order(key: i64) -> f64 {
    let bits = if key < 0 {
        i64::MIN.wrapping_sub(key)
    } else {
        key
    };
    f64::from_bits(bits as u64)
}

fn largest_coordinate(at: PredicatePoint) -> f64 {
    at.enclosure()
        .iter()
        .map(|coordinate| coordinate.lo().abs().max(coordinate.hi().abs()))
        .fold(0.0, f64::max)
}

fn widened_by(at: PredicatePoint, reach: f64) -> PredicatePoint {
    PredicatePoint::Constructed(at.enclosure().map(|coordinate| {
        Interval::new(coordinate.lo() - reach, coordinate.hi() + reach)
            .expect("a positive reach round a finite coordinate")
    }))
}
