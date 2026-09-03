//! The search behind [`NEAREST_F64_POINT_ROUNDOFF`] (`docs/decisions.md`
//! §15.6).
//!
//! The bound says a point that truly lies on a curved surface, written down as
//! three `f64`, lands within `NEAREST_F64_POINT_ROUNDOFF` times its largest
//! coordinate of that surface. Two families put that to a real surface.
//!
//! **A bracket walk**, which needs nothing but the surface's own sign. Step one
//! coordinate through neighbouring `f64` until the sign flips. The surface
//! passes between those two, so **both of them are within one coordinate
//! spacing of it and no `f64` on that line is nearer** — which is exactly what
//! a corpus file's corner is when the writer rounded correctly.
//!
//! **Points that are exactly on the surface and then round**, which the walk
//! cannot build. A walk moves one coordinate; the `√3` in the derivation is
//! about three coordinates rounding at once, and until this family existed the
//! published number was never searched in the configuration it was derived for.
//! A Pythagorean quadruple puts a point exactly on a ball of whole radius with
//! no arithmetic at all, and one IEEE division per coordinate then rounds it to
//! the nearest `f64` triple — correctly rounded, so no other triple is nearer.
//! The same trick puts exact points on a cone and on a ring. A pipe takes only
//! two of the three, and that is a fact about pipes rather than a hole in the
//! search: a pipe is the same all the way along its axis, so the rounding of
//! the coordinate up the axis slides the point along the surface and cannot
//! take it off.
//!
//! # What this file proves, and what it only records
//!
//! Not every assertion below can fail, and a file presenting an arithmetic
//! identity as one of several independent checks is the §15.6 failure in
//! miniature. So, plainly.
//!
//! **These test the size of the constant:**
//!
//! - *Halving it fails this.* How far the rounding really moved the point, in
//!   plain distance, against the budget meant to cover it. The true point is a
//!   fraction of whole numbers, so the miss is worked out from those and no
//!   surface, cascade or filter is consulted — this is the derivation's own
//!   claim and nothing else. The worst each family finds is over half the
//!   budget, so a constant half the published one fails all four.
//! - *Too small would also fail this.* Widened into a box of the published
//!   half-width, a point the surface passes within one rounding of can no
//!   longer be certified off it. That is the bound as its consumer spends it.
//!   It has far more room in it than the assertion above, for a reason the last
//!   section gives.
//! - *Too large would fail this.* The same box around a corner a thousand
//!   spacings further out still certifies a side, so the number is a width and
//!   not a tolerance.
//!
//! **These keep the run from passing by doing nothing:** the bracketing pair
//! has to come back certified off the surface; an exactly-representable point
//! has to be *decided* rather than refused, because level 3 of the cascade is
//! exact on exact input and a refusal there is a filter bug; every scale has to
//! give a bracket; and a floor says how many rounded points landed off the
//! surface at all.
//!
//! **This one records and cannot fail at the published value:** `spacing <=
//! budget`. `spacing` is the gap to the next `f64` at the walked coordinate `v`
//! and `budget` is `NEAREST_F64_POINT_ROUNDOFF · max|coord|`, which at the
//! published value is at least `f64::EPSILON · |v|`, which is at least that
//! gap — for any point at all, naming no surface. It is worth keeping only
//! because it fails the moment the constant is set below `f64::EPSILON`.
//! [`THE_SHARE_A_SPACING_HAS_TO_FILL`] is the same kind of thing pointed the
//! other way: it fails if the constant is set far above `f64::EPSILON`. Neither
//! is a search and neither says anything about a ball.
//!
//! # Why the widened box cannot pin the constant, and what does
//!
//! The run prints, per surface, the smallest share of the budget the cascade
//! had to be given before it stopped certifying a side. That share is the
//! widened box's headroom: a constant that many times smaller would still pass
//! *that* assertion, and mostly it is near zero. That looks alarming and is
//! not. The moment a corner is widened at all it stops being an exact `f64`
//! triple, the cascade loses its exact level, and the interval arithmetic left
//! over already over-estimates by about a coordinate spacing on these shapes —
//! so the box goes from certain to undecided almost as soon as it has any
//! width, and where it does that says more about interval arithmetic than about
//! the size of this constant. **The widened box is how the bound is spent, not
//! what fixes its size.**
//!
//! What fixes its size is the distance measurement, which asks the plain
//! question the doc comment's derivation asks: a point exactly on the surface,
//! written down as the nearest three `f64` — how far did it move, against
//! `NEAREST_F64_POINT_ROUNDOFF · L`? No cascade, no filter, nowhere to hide.
//! The worst found is over half the budget on every one of the four surfaces,
//! so **the published constant would fail this file if it were halved**, and
//! [`HOW_MUCH_OF_THE_BUDGET_A_ROUNDING_HAS_TO_USE`] is the assertion that says
//! so. What is left between that and the derivation's own `√3/2 ≈ 0.87` needs
//! all three coordinates the same size and all three rounding the wrong way at
//! once; a search over whole-number fractions gets close to that without
//! landing on it, and the run prints how close.

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
/// The other side of §15.6: a bound nothing can get near is untested. **This is
/// bookkeeping about the constant's size and not a search over surfaces** — the
/// share is `2^floor(log2 v) / max|coord|` scaled by `f64::EPSILON /
/// NEAREST_F64_POINT_ROUNDOFF`, so no surface enters it. It fails if the
/// constant is set far above `f64::EPSILON`, and the `spacing <= budget`
/// assertion fails if it is set below. Between them they hold the constant
/// inside a small factor either side of a coordinate spacing without saying
/// anything about a ball, a pipe, a cone or a ring — how far a rounding really
/// moves a point on one of those is what the exact families measure.
const THE_SHARE_A_SPACING_HAS_TO_FILL: f64 = 0.25;

/// How much of the published budget the worst rounding a family finds has to
/// use, before that family stops being evidence about the size of the bound.
///
/// The derivation's own worst case is `√3/2 ≈ 0.87` of the budget and needs all
/// three coordinates the same size and all three rounding the wrong way at
/// once. Half is the number that matters: a family whose worst rounding uses
/// more than half the budget is a family that **fails if the constant is
/// halved**, which is the whole point of searching for a worst case. All four
/// are over it, and the run prints by how much.
const HOW_MUCH_OF_THE_BUDGET_A_ROUNDING_HAS_TO_USE: f64 = 0.5;

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

#[test]
fn a_ball_point_with_all_three_coordinates_rounded_is_never_certified_off_it() {
    let mut tally = Tally::new("a ball");
    for [across, up, along, whole] in QUADRUPLES_THAT_SQUARE_TO_A_SQUARE {
        for radius in WHOLE_LENGTHS {
            for two_to_the in POWERS_OF_TWO {
                let size = size_of(two_to_the);
                let ball = Sphere::new(Point::origin(), f64::from(radius) * size)
                    .expect("a ball with a positive radius");
                let (at, moved) = round_a_point(
                    [
                        [radius * across, whole],
                        [radius * up, whole],
                        [radius * along, whole],
                    ],
                    size,
                );
                tally.saw(moved, check_a_rounded_surface_point(&ball, at));
            }
        }
    }
    tally.report_what_it_checked();
}

/// A pipe is the same all the way along its axis, so the third coordinate's
/// rounding slides the point along the surface and cannot take it off. Only
/// the two across the axis can move a point off a pipe, and that is a fact
/// about pipes rather than a hole in the search. The third is rounded here all
/// the same, because the published bound is about **how far the point moved**
/// and not about how far off the surface it ended up — the derivation adds up
/// three coordinates, so the measurement does too.
#[test]
fn a_pipe_point_with_its_two_across_coordinates_rounded_is_never_certified_off_it() {
    let mut tally = Tally::new("a pipe");
    for [across, up, whole] in TRIPLES_THAT_SQUARE_TO_A_SQUARE {
        for radius in WHOLE_LENGTHS {
            for two_to_the in POWERS_OF_TWO {
                let size = size_of(two_to_the);
                let pipe = Cylinder::new(straight_up(), f64::from(radius) * size)
                    .expect("a pipe with a positive radius");
                let (at, moved) = round_a_point(
                    [
                        [radius * across, whole],
                        [radius * up, whole],
                        [radius * (across + up), 2 * whole],
                    ],
                    size,
                );
                tally.saw(moved, check_a_rounded_surface_point(&pipe, at));
            }
        }
    }
    tally.report_what_it_checked();
}

/// A cone of slope `rise / 4` passes exactly through `(rise·m·a, rise·m·b,
/// 4·m·h) / (k·h)` whenever `a² + b² = h²`: the point is `rise·m/k` out from
/// the axis and `4·m/(k)` up it, and `slope · height` is `rise·m/k` too.
/// Dividing all three through by the same odd `k` keeps it on the cone, because
/// a cone through the origin is the same cone at every size, and leaves every
/// coordinate with a remainder to round.
#[test]
fn a_cone_point_with_all_three_coordinates_rounded_is_never_certified_off_it() {
    let mut tally = Tally::new("a cone");
    for [across, up, whole] in TRIPLES_THAT_SQUARE_TO_A_SQUARE {
        for rise in [1, 3, 5, 7] {
            for height in WHOLE_LENGTHS {
                for two_to_the in POWERS_OF_TWO {
                    let size = size_of(two_to_the);
                    let cone = Cone::new(straight_up(), f64::from(rise) / 4.0)
                        .expect("a cone with a positive slope");
                    let odd = 13;
                    let (at, moved) = round_a_point(
                        [
                            [rise * height * across, odd * whole],
                            [rise * height * up, odd * whole],
                            [4 * height, odd],
                        ],
                        size,
                    );
                    tally.saw(moved, check_a_rounded_surface_point(&cone, at));
                }
            }
        }
    }
    tally.report_what_it_checked();
}

/// A ring of radii `R` and `r` passes exactly through `((R·w + r·u)·a / (w·h),
/// (R·w + r·u)·b / (w·h), r·v / w)` whenever `a² + b² = h²` and `u² + v² = w²`:
/// the
/// point is `(R·w + r·u)/w` out from the axis, which is `r·u/w` past the ring's
/// own circle, and `r·v/w` up — and those two are the legs of a right triangle
/// with `r` as its hypotenuse, which is the ring's tube.
#[test]
fn a_ring_point_with_all_three_coordinates_rounded_is_never_certified_off_it() {
    let mut tally = Tally::new("a ring");
    for [across, up, whole] in TRIPLES_THAT_SQUARE_TO_A_SQUARE {
        for [out, over, round_the_tube] in TRIPLES_THAT_SQUARE_TO_A_SQUARE {
            for [ring, tube] in [[25, 5], [65, 13], [169, 17], [4001, 3]] {
                for two_to_the in POWERS_OF_TWO {
                    let size = size_of(two_to_the);
                    let torus = Torus::new(
                        straight_up(),
                        f64::from(ring) * size,
                        f64::from(tube) * size,
                    )
                    .expect("a ring wider than its tube");
                    let out_from_the_axis = ring * round_the_tube + tube * out;
                    let (at, moved) = round_a_point(
                        [
                            [out_from_the_axis * across, round_the_tube * whole],
                            [out_from_the_axis * up, round_the_tube * whole],
                            [tube * over, round_the_tube],
                        ],
                        size,
                    );
                    tally.saw(moved, check_a_rounded_surface_point(&torus, at));
                }
            }
        }
    }
    tally.report_what_it_checked();
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
/// surface passes between, and put the file header's questions to them.
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
        // Anti-vacuity, not the bound: without it a widened box that refuses
        // would be refusing a point nothing ever certified in the first place.
        assert!(
            certified,
            "the pair bracketing the surface has to be off it for this file to be \
             checking anything, and {reading} was not"
        );

        let budget = NEAREST_F64_POINT_ROUNDOFF * largest_coordinate(at);
        let decision = surface.side_of(widened_by(at, budget));
        // The bound's own claim, and the assertion a constant set too small
        // fails. `the_share_of_the_budget_it_asked_for` below measures by how
        // much it passes.
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
    // The other size question: a constant set too large fails this.
    assert!(
        matches!(decision, Decision::Pos | Decision::Neg),
        "a corner {A_MISS_NOBODY_COULD_BLAME_ON_ROUNDING} spacings off the surface is \
         one somebody chose, and the box round it has to still say which side: \
         {decision:?}"
    );

    // Bookkeeping, not a search: `spacing` is the gap at the walked coordinate
    // `v` and `budget` is at least `f64::EPSILON · |v|`, which is at least that
    // gap, for any point and any surface. It can only fail if the published
    // constant is set below `f64::EPSILON`.
    assert!(
        spacing <= budget,
        "the surface fell in a gap of {spacing} between two neighbouring f64, and the \
         budget of {budget} is meant to cover a whole one"
    );
    Some((spacing / budget, asked_for))
}

/// What the four walking tests print and check.
///
/// The first figure is bookkeeping: one coordinate spacing as a share of the
/// budget, which is under one whatever surface is passed and only says the
/// published constant sits near `f64::EPSILON`.
///
/// The second is the one worth reading. It is how much of the budget the
/// cascade had to be given before it stopped certifying, so it is the headroom:
/// a constant that many times smaller would still pass every assertion in this
/// file. It is printed and not asserted against, because on a pipe, a cone and
/// a ring it is near zero — widening a corner at all costs those three their
/// exact level, and the interval arithmetic left over already loses about a
/// spacing on these shapes, so for them the budget is a floor under the reach
/// rather than what decides.
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
        "{surface}, walking one coordinate: {} brackets, a spacing filled up to {worst:.3} of \
         the budget, and the cascade asked for up to {asked_for:.3} of it — so this family \
         would still pass with the constant {} times smaller",
        measured.len(),
        headroom(asked_for),
    );
    assert_eq!(
        measured.len(),
        scales().len(),
        "a scale gave no bracket, so this run checked less than it says"
    );
    // The other side of the bookkeeping assert: this fails if the constant is
    // set far above `f64::EPSILON`.
    assert!(
        worst >= THE_SHARE_A_SPACING_HAS_TO_FILL,
        "a spacing filled only {worst:.3} of the budget, so the budget is far wider than \
         the rounding it is meant to be and this search is no longer evidence for it"
    );
}

/// How many times smaller the published constant could be set before the run
/// that measured `asked_for` would start failing.
fn headroom(asked_for: f64) -> String {
    if asked_for > 0.0 {
        format!("{:.2}", 1.0 / asked_for)
    } else {
        "any number of".to_owned()
    }
}

/// Whole numbers whose squares add up to a square, so that dividing a whole
/// radius by the last of them puts a point **exactly** on a ball with nothing
/// rounded on the way.
const QUADRUPLES_THAT_SQUARE_TO_A_SQUARE: [[i32; 4]; 19] = [
    [12, 15, 16, 25],
    [12, 16, 21, 29],
    [10, 10, 23, 27],
    [1, 2, 2, 3],
    [2, 3, 6, 7],
    [1, 4, 8, 9],
    [4, 4, 7, 9],
    [2, 6, 9, 11],
    [6, 6, 7, 11],
    [3, 4, 12, 13],
    [2, 5, 14, 15],
    [2, 10, 11, 15],
    [1, 12, 12, 17],
    [8, 9, 12, 17],
    [6, 10, 15, 19],
    [4, 5, 20, 21],
    [4, 13, 16, 21],
    [6, 13, 18, 23],
    [9, 12, 20, 25],
];

/// The same for two numbers, which is what a pipe, a cone and a ring are built
/// from.
const TRIPLES_THAT_SQUARE_TO_A_SQUARE: [[i32; 3]; 12] = [
    [3, 4, 5],
    [5, 12, 13],
    [8, 15, 17],
    [7, 24, 25],
    [20, 21, 29],
    [12, 35, 37],
    [9, 40, 41],
    [28, 45, 53],
    [11, 60, 61],
    [33, 56, 65],
    [16, 63, 65],
    [48, 55, 73],
];

/// The whole lengths the exact families are asked at.
///
/// All odd, so dividing by the quadruple's last number leaves a remainder to
/// round; a length that shared a factor with it would divide out and hand the
/// surface a point already exactly on it, which searches nothing. There are a
/// lot of them because the worst rounding is the point of the exercise, and the
/// only way to find one that catches all three coordinates near half a spacing
/// at once is to try a great many remainders.
const WHOLE_LENGTHS: [i32; 11] = [1, 3, 7, 17, 33, 101, 255, 1001, 12_345, 100_001, 999_983];

/// The powers of two the exact families are then scaled by.
///
/// Scaling by a power of two moves no bits: the rounded triple stays the
/// nearest triple to the scaled true point, and the surface stays exactly the
/// surface. So this asks the relative bound over eighteen orders of magnitude
/// without spending a rounding to get there — which the walking family, whose
/// scales are deliberately not powers of two, cannot do.
const POWERS_OF_TWO: [i32; 5] = [-30, -8, 0, 8, 30];

fn size_of(two_to_the: i32) -> f64 {
    2.0_f64.powi(two_to_the)
}

/// The nearest `f64` triple to the point whose three coordinates are the exact
/// fractions `parts`, scaled by the power of two `size`, together with how far
/// the rounding moved it as a share of the published budget.
///
/// **The share is what the derivation itself claims.** Rounding moves each
/// coordinate by at most half a spacing, so three of them move the point by at
/// most `√3 · f64::EPSILON / 2 ≈ 0.87` times the largest coordinate — under
/// [`NEAREST_F64_POINT_ROUNDOFF · L`], which is the whole argument. This
/// measures the distance the rounding really moved, so a share over one means
/// the constant is too small. It needs no surface and no cascade: the true
/// point is a fraction of whole numbers and the miss is worked out from those.
///
/// `mul_add` gives `p · denominator − numerator` with one rounding of the exact
/// value, and that value is a handful of bits wide, so it is exact. Rather than
/// leave that as a sentence, every slip is checked against half a coordinate
/// spacing — which is what correct rounding means and what a `mul_add` that had
/// lost a bit would miss.
fn round_a_point(parts: [[i32; 2]; 3], size: f64) -> (PredicatePoint, f64) {
    let rounded = parts.map(|[numerator, denominator]| nearest_f64_to(numerator, denominator));
    let largest = rounded
        .iter()
        .fold(0.0_f64, |so_far, coordinate| so_far.max(coordinate.abs()));

    let mut moved_squared = 0.0;
    for (&coordinate, [numerator, denominator]) in rounded.iter().zip(parts) {
        let slip = coordinate.mul_add(f64::from(denominator), -f64::from(numerator))
            / f64::from(denominator);
        assert!(
            2.0 * slip.abs() <= spacing_at(coordinate.abs()),
            "{numerator}/{denominator} rounded to {coordinate}, which this makes out to be \
             {slip} away — further than half a spacing, so either the division was not \
             correctly rounded or this is not measuring what it says"
        );
        moved_squared += slip * slip;
    }

    let scaled = rounded.map(|coordinate| coordinate * size);
    for coordinate in scaled {
        // Scaling by a power of two is exact only while the answer stays a
        // normal number, and the claim that the rounded triple is still the
        // nearest one rests on that.
        assert!(
            coordinate.is_normal(),
            "{coordinate} is not a normal number, so scaling by {size} spent bits and this \
             triple is no longer the nearest one to anything"
        );
    }

    (
        PredicatePoint::from(scaled),
        moved_squared.sqrt() / (NEAREST_F64_POINT_ROUNDOFF * largest),
    )
}

/// The `f64` nearest to `numerator / denominator`.
///
/// One IEEE division of two exactly-represented whole numbers, so the answer is
/// correctly rounded and **no other `f64` is nearer the true fraction** — which
/// is the whole claim [`NEAREST_F64_POINT_ROUNDOFF`] is about.
fn nearest_f64_to(numerator: i32, denominator: i32) -> f64 {
    f64::from(numerator) / f64::from(denominator)
}

fn straight_up() -> Axis {
    Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0)).expect("an axis with a direction")
}

/// Put the file header's size questions to `at`, the nearest `f64` triple to a
/// point that lies exactly on `surface`.
///
/// Hands back the share of the budget the cascade had to be given before it
/// stopped certifying a side, or nothing when the triple landed exactly on the
/// surface and there was no rounding for the budget to cover.
fn check_a_rounded_surface_point(surface: &impl SideOfSurface, at: PredicatePoint) -> Option<f64> {
    let where_it_landed = surface.side_of(at);
    if matches!(where_it_landed, Decision::Zero) {
        return None;
    }
    // Anti-vacuity, and a filter check as well: every coordinate here is an
    // exact `f64` and level 3 of the cascade is exact on those, so a refusal is
    // a bug in our filters and not a fact about the surface.
    assert!(
        matches!(where_it_landed, Decision::Pos | Decision::Neg),
        "a point written as three exact f64 has to be decided, and this one came back \
         {where_it_landed:?}"
    );

    let budget = NEAREST_F64_POINT_ROUNDOFF * largest_coordinate(at);
    let decision = surface.side_of(widened_by(at, budget));
    // The bound's own claim, in the configuration its `√3` is about.
    assert!(
        !matches!(decision, Decision::Pos | Decision::Neg),
        "this triple is the nearest three f64 to a point exactly on the surface, so no \
         writer could have put a corner nearer; a box of {budget} round it still came \
         back {decision:?}"
    );
    Some(the_share_of_the_budget_it_asked_for(surface, at, budget))
}

/// What the exact families count, so that a run which stopped landing off the
/// surface says so instead of passing.
struct Tally {
    surface: &'static str,
    tried: u32,
    landed_off_the_surface: u32,
    asked_for: f64,
    moved: f64,
}

impl Tally {
    fn new(surface: &'static str) -> Self {
        Self {
            surface,
            tried: 0,
            landed_off_the_surface: 0,
            asked_for: 0.0,
            moved: 0.0,
        }
    }

    fn saw(&mut self, moved: f64, share: Option<f64>) {
        self.tried += 1;
        self.moved = self.moved.max(moved);
        if let Some(share) = share {
            self.landed_off_the_surface += 1;
            self.asked_for = self.asked_for.max(share);
        }
    }

    fn report_what_it_checked(self) {
        println!(
            "{}, rounding every coordinate at once: {} of {} triples landed off the \
             surface. The rounding moved a point up to {:.3} of the budget, and the \
             cascade asked for up to {:.3} of it — so the widened box alone would still \
             pass with the constant {} times smaller",
            self.surface,
            self.landed_off_the_surface,
            self.tried,
            self.moved,
            self.asked_for,
            headroom(self.asked_for),
        );

        // A triple that divided out exactly is on the surface and asks the
        // budget for nothing, so a family where most of them did that would be
        // green without having searched.
        assert!(
            self.landed_off_the_surface * 2 >= self.tried,
            "only {} of {} rounded triples landed off the surface at all, so most of this \
             family was checking a budget nothing needed",
            self.landed_off_the_surface,
            self.tried
        );
        // The derivation's own claim, and the assertion that fails when the
        // constant is set too small. The cascade is not consulted: this is the
        // distance from a point exactly on the surface to the nearest triple
        // there is, against the budget meant to cover it.
        assert!(
            self.moved <= 1.0,
            "writing a point of {} down as three f64 moved it {} of the budget, so \
             NEAREST_F64_POINT_ROUNDOFF does not cover a rounding",
            self.surface,
            self.moved
        );
        assert!(
            self.moved >= HOW_MUCH_OF_THE_BUDGET_A_ROUNDING_HAS_TO_USE,
            "the worst rounding this family found used only {} of the budget, so it never \
             got near the bound and is no longer evidence about its size",
            self.moved
        );
    }
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
