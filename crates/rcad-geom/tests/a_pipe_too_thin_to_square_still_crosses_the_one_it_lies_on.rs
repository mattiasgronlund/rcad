//! A hair-thin pipe lying along a vastly fatter pipe's skin, swept down 66
//! decades of thinness — `docs/decisions.md` §143's refusal band.
//!
//! §140 fixed the top sliver of this ladder, where the two orders disagreed.
//! Below it sat a band from about `3.04e-139` to about `1.28e-73` where the true
//! answer is two straight runs and **both orders refused**, with an enclosure of
//! everything. That is what this file is about.
//!
//! # The geometry, and why the answer exists
//!
//! Both axes run along `x`, so the whole question is two circles in `(y, z)`:
//! one of radius `R1` at the origin, one of radius `r2` at `(R1, Z)`. They cut
//! each other twice exactly when `(R1 − r2)² < d² < (R1 + r2)²` for
//! `d² = R1² + Z²`. Take `R1²` off both sides and that is
//!
//! ```text
//! Z² + 2·R1·r2 − r2² > 0     and     2·R1·r2 + r2² − Z² > 0.
//! ```
//!
//! The first holds for every positive `r2`. The second is the whole story: the
//! thin pipe reaches the fat one's skin exactly when `2·R1·r2` out-reaches `Z²`,
//! which for this `R1` and `Z` is `r2` past `Z²/(2·R1) = 1.712674258532773e-139`.
//! Above that the truth is two straight runs, below it the two pipes miss each
//! other entirely, and [`the_thin_pipe_stops_reaching_the_fat_one_where_the_arithmetic_says_it_does`]
//! pins the kernel's flip to that number within one step of a six-bit mantissa.
//!
//! # Where the refusal came from
//!
//! The shape question is the sign of `E² − 4r₁²r₂²W₁²` for
//! `E = A − W₁(r₁² + r₂²)`, and `A` is `2.5478723449713562e307` here while the
//! term taken off it is the same number to fifteen digits. The true `E` is
//! `Z² − r2²`, about `1.7e15`; as an interval it came back `±4.99e291`, and
//! **squaring that overflows to everything**. The exact arm had the right `E`
//! all along — `A`'s `r₁²` cancels against the one being subtracted — and threw
//! it away because *its* own next step, `E·E` as an expansion, needs every
//! pairwise product to clear `f64::MIN_POSITIVE · 2⁵³`, and the smallest entry
//! squared is `r₂⁴`. So the two arms failed for two unrelated reasons and the
//! answer nobody could reach was sitting in the middle.
//!
//! The fix is to let the interval arm read `E` off the exact expansion, which is
//! what `two_lines_between_two_pipes` already does twenty lines away in the same
//! file. An exact `E` that encloses to a fraction of an ulp squares without
//! trouble; `r₂⁴` never has to exist.
//!
//! # What is left, and it is not this band
//!
//! [`the_squares_whose_roundoff_has_nowhere_to_go_are_what_still_refuses`]
//! measures the residue and names its cause. Forming `E` exactly needs
//! `r₂²` *and its roundoff half* to survive `Exact::times`, and a roundoff half
//! that lands between zero and `2.004168360008973e-292` does not. That is a
//! floor in `Exact::times`, not in this expression, and it takes about one
//! radius in fifty across the band.

#![forbid(unsafe_code)]

use rcad_geom::{Axis, Cylinder, Point, Surface, SurfaceMeet, Vector, intersect};
use rcad_predicates::{Policy, ProvenanceLog};

/// The fat pipe's radius, and how far the thin pipe's axis sits off the fat
/// axis across it. §140's pair drew both.
const R1: f64 = 5.047645337156084e153;

/// How far the thin pipe's axis sits along the fat axis' own direction.
const Z: f64 = 41581179.0;

/// The thinnest radius in the band, and §143's measured bottom edge: the first
/// step of the sweep above `Z²/(2·R1)`, where the truth turns from "they miss"
/// to "two straight runs".
const THINNEST_IN_THE_BAND: f64 = 3.044037611241337e-139;

/// Two pipes 66 decades apart in width still cross in two straight runs, and
/// the kernel says so whichever of them is asked about first.
///
/// Every radius here is `5.4016709327697754e-8` — §140's thin pipe — halved
/// some number of times, so each one squares exactly and the residue
/// [`the_squares_whose_roundoff_has_nowhere_to_go_are_what_still_refuses`]
/// measures is out of the way. `docs/decisions.md` §143.
#[test]
fn a_pipe_far_too_thin_to_square_still_crosses_the_one_it_lies_on() {
    let mut thinnest = f64::INFINITY;
    let mut fattest: f64 = 0.0;
    for halvings in 0..=436 {
        let thin = 5.4016709327697754e-8 * 2.0f64.powi(-halvings);
        if thin < THINNEST_IN_THE_BAND {
            continue;
        }
        assert_eq!(
            thin.mul_add(thin, -(thin * thin)),
            0.0,
            "this radius does not square exactly, so it is not one of the ones this test is about"
        );
        thinnest = thinnest.min(thin);
        fattest = fattest.max(thin);
        for first in 0..2 {
            let named = name_the_crossing(&two_pipes_side_by_side(thin), first);
            assert_eq!(
                named, "curves 2",
                "a pipe {thin:e} wide on a pipe {R1:e} wide, asked about pipe {first} first"
            );
        }
    }
    assert_eq!(thinnest, THINNEST_IN_THE_BAND);
    assert_eq!(fattest, 5.4016709327697754e-8);
    assert!(
        fattest / thinnest > 1e66,
        "the sweep should cover the whole 66-decade band, and covers {:e}",
        fattest / thinnest
    );
}

/// Below `Z²/(2·R1)` the thin pipe no longer reaches the fat one's skin, and
/// the kernel's answer turns over within one step of a six-bit mantissa of
/// where the exact cross-section says it should.
///
/// The margin either side of the last step is about `1e13` against quantities of
/// about `1.7e15`, so which side of the turn each of these radii falls on is not
/// a rounding — it is a hundredth of the numbers involved. `r2²` is `2.9e-278`
/// here and changes nothing.
#[test]
fn the_thin_pipe_stops_reaching_the_fat_one_where_the_arithmetic_says_it_does() {
    for sixty_fourths in 64..=192 {
        let thin = f64::from(sixty_fourths) * 2.0f64.powi(-462 - 6);
        let reach = 2.0 * R1 * thin - Z * Z;
        let wanted = if reach > 0.0 { "curves 2" } else { "Nothing" };
        for first in 0..2 {
            assert_eq!(
                name_the_crossing(&two_pipes_side_by_side(thin), first),
                wanted,
                "a pipe {thin:e} wide reaches past the fat pipe's far side by {reach:e}, \
                 asked about pipe {first} first"
            );
        }
    }
}

/// The same band on a pair whose stored direction does not have squared length
/// one, which is what every direction off the coordinate axes rounds to.
///
/// `(1, 1, 0)` stores as `(0.7071067811865476, 0.7071067811865476, 0)`, whose
/// squared length is `1` and two ulps. The thin axis is put at `(0, 0, R1)`,
/// square to that direction and exactly `R1` away from it, so the thin pipe's
/// axis lies exactly on the fat pipe's skin and the two cross in two runs for
/// every radius under `2·R1`.
///
/// The band's bottom edge moves up with the direction: `W₁`'s own roundoff entry
/// has to survive being multiplied by `r₂²`, so the thinnest pipe this pair
/// reaches is a few decades fatter than the one the `x` axis reaches.
#[test]
fn a_direction_whose_squared_length_is_not_one_crosses_the_same_band() {
    let leaning = Axis::new(Point::origin(), Vector::new(1.0, 1.0, 0.0)).expect("a direction");
    let [a, b, c] = leaning.direction().components();
    assert_ne!(
        a * a + b * b + c * c,
        1.0,
        "this test is about a direction whose squared length is not one"
    );

    for halvings in 0..=433 {
        let thin = 5.4016709327697754e-8 * 2.0f64.powi(-halvings);
        if thin < 2.4352300889930697e-138 {
            continue;
        }
        let fat = Cylinder::new(leaning, R1).expect("a fat pipe");
        let thin_pipe = Cylinder::new(
            Axis::new(Point::new(0.0, 0.0, R1), Vector::new(1.0, 1.0, 0.0)).expect("a direction"),
            thin,
        )
        .expect("a thin pipe");
        let pair = [Surface::Cylinder(fat), Surface::Cylinder(thin_pipe)];
        for first in 0..2 {
            assert_eq!(
                name_the_crossing(&pair, first),
                "curves 2",
                "a pipe {thin:e} wide on the skin of a pipe {R1:e} wide, \
                 asked about pipe {first} first"
            );
        }
    }
}

/// What is left of the band, and the one thing that causes it.
///
/// Reading `E` off the exact expansion only helps where the exact expansion can
/// be built, and building it multiplies `r₁² + r₂²` by the squared direction.
/// `Exact::times` refuses a pairwise product under
/// `f64::MIN_POSITIVE · 2⁵³ = 2.004168360008973e-292`, and the smallest entry of
/// `r₁² + r₂²` is the roundoff half of `r₂²` — which for a radius around
/// `1e-138` is about `1e-292`, right on that floor. Where it falls under,
/// nothing can be read off the exact arm and the answer is the refusal this
/// kernel gave before.
///
/// **That is a limit of `Exact::times` and not of this expression**, so it is
/// measured and named here rather than left as an unexplained rate. The
/// assertion that matters is not the size of the residue but that every refusal
/// has that cause: a refusal from any other cause is a new defect.
#[test]
fn the_squares_whose_roundoff_has_nowhere_to_go_are_what_still_refuses() {
    const SMALLEST_EXACT_PRODUCT: f64 = f64::MIN_POSITIVE * 9007199254740992.0;
    let top_of_the_band = 1.2822983070604231e-73;

    let mut asked = 0u32;
    let mut refused = 0u32;
    let mut seed = 0x2545_F491_4F6C_DD1Du64;
    for _ in 0..2000 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let share = (seed >> 11) as f64 / 9007199254740992.0;
        let thin = THINNEST_IN_THE_BAND * (top_of_the_band / THINNEST_IN_THE_BAND).powf(share);
        let roundoff = thin.mul_add(thin, -(thin * thin));
        let can_be_carried = roundoff == 0.0 || roundoff.abs() >= SMALLEST_EXACT_PRODUCT;
        let pair = two_pipes_side_by_side(thin);
        for first in 0..2 {
            asked += 1;
            let named = name_the_crossing(&pair, first);
            if named == "refused" {
                refused += 1;
                assert!(
                    !can_be_carried,
                    "a pipe {thin:e} wide refused for some reason other than a roundoff half \
                     of {roundoff:e} that `Exact::times` cannot carry"
                );
            } else {
                assert_eq!(named, "curves 2", "a pipe {thin:e} wide was named wrongly");
            }
        }
    }

    assert!(
        refused * 10 < asked,
        "{refused} of {asked} refused, which is more of the band than the roundoff floor \
         alone can account for"
    );
}

/// The pair every test here but one asks about: a pipe `R1` wide about the `x`
/// axis, and a pipe `thin` wide whose axis sits `R1` across and `Z` along.
fn two_pipes_side_by_side(thin: f64) -> [Surface; 2] {
    let along = Vector::new(1.0, 0.0, 0.0);
    let fat = Cylinder::new(Axis::new(Point::origin(), along).expect("a direction"), R1)
        .expect("a fat pipe");
    let thin = Cylinder::new(
        Axis::new(Point::new(0.0, R1, Z), along).expect("a direction"),
        thin,
    )
    .expect("a thin pipe");
    [Surface::Cylinder(fat), Surface::Cylinder(thin)]
}

/// What the kernel calls the crossing, as a short word a failure can print.
fn name_the_crossing(pair: &[Surface; 2], first: usize) -> String {
    let met = intersect(
        &pair[first],
        &pair[1 - first],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    match &met {
        Ok(SurfaceMeet::Curves(runs)) => format!("curves {}", runs.len()),
        Ok(other) => format!("{other:?}"),
        Err(_) => "refused".to_owned(),
    }
}
