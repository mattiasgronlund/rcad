//! What decides whether the two lines two side-by-side pipes cut out of each
//! other can be told apart, and where that gives up.
//!
//! §142.9's fourth site was the `across` term in `two_lines_between_two_pipes`,
//! and the bound its doc comment published: a thin pipe on a fat one's skin
//! losing the answer "past `r₁ ≈ 2²⁶·r₂` — a 67 m pipe against a 1 m one".
//! §15.6 asks a published bound for a test. Measuring it found **the bound was
//! about a line of code that is not there, and the kernel gave up thirty million
//! times sooner than it said, for a different reason entirely**.
//!
//! # What the published bound is right about
//!
//! The arithmetic, and only about the formula the sentence explains the
//! *absence* of. `r₁² − across²` really is `r₂²` reached by subtracting two
//! numbers of size `r₁²`, and the gap between neighbouring `f64` at `r₁²` grows
//! to the whole of `r₂²` at exactly `r₁ = 2²⁶·r₂`. That is the first test below.
//! `two_lines_between_two_pipes` takes `X` from the discriminant instead, so
//! nothing about the kernel's own behaviour follows from it — and the very pair
//! the sentence describes is named at `2²⁶` and on past `2⁵⁰`.
//!
//! # What was really deciding it, and the fix
//!
//! The picking plane's normal. `two_lines_between_two_pipes` asks
//! [`HalfSpace::through_a_point_square_to`] for one along `along × towards`,
//! which separates the two lines perfectly — but that vector is only *exactly*
//! square to `along` when the pipes run along a coordinate direction. Otherwise
//! one of three fallbacks is taken, and the best of those can be 55° off (its own
//! doc comment publishes that bound). The two lines sit at `(across, ±X)` in the
//! plane square to the pipes, so a plane anchored on the **first pipe's axis**
//! and leaning `θ` off the separating direction holds both of them on one side
//! once `across/X` passes `cot θ` — a radius ratio of `2.03` for pipes along
//! `[0.3, 0.5, 0.7]`, and `1.32` for the worst of the thirty directions swept
//! below. Nineteen of those thirty refused somewhere under a ratio of 100.
//!
//! It was §16.10's gap and not a limit, and §150's shape exactly: at the refusing
//! ratio the two lines read `−1.75` and `−0.0033` against a plane reading zero,
//! a window `1.75` wide on pipes a couple of units across, so every `f64` between
//! them separated them and one stored `f64` was the whole of what could not reach
//! it.
//!
//! The fix is to anchor the plane **between the two lines** instead of on the
//! axis, which removes the `across·sin θ` term rather than bounding it. Three
//! families were weighed, per §16.10:
//!
//! - **Reorder or rescale the arithmetic.** Refuted by the numbers: at the
//!   refusing ratio the margin the kernel was short of was `3.3e-3`, ten trillion
//!   times any rounding in play, and the axis-aligned control decided at `2⁵⁰`
//!   where every enclosure is at its widest. Nothing was cancelling.
//! - **Let the normal lean, and bound the lean.** Refused by
//!   `two_lines_told_apart`'s own doc comment, and rightly: a line is endless, so
//!   a normal off square by any rounding reads a different number far enough
//!   along it and the half-space holds nothing.
//! - **Move the anchor.** Keep the exactly-square normal and pass the plane
//!   through the point midway between the lines, which the caller already has
//!   the pieces of. This is the one that works.
//!
//! `docs/decisions.md` §164.
//!
//! # Where it gives up now
//!
//! Nowhere under a radius ratio of 100, for any of the thirty directions, either
//! way round — where nineteen of thirty refused before. The first refusal is at
//! `2⁵⁰`, fifteen decimal orders further out, and past `2⁵⁴` the pairs stop being
//! the pairs that were asked for: the thin pipe's axis is built at `fat` along a
//! unit step, and once one `f64` step at `fat` grows past the thin radius that
//! point is no longer on the fat pipe's skin. The axis-aligned control, whose
//! step is exact, keeps being named to `2⁷⁰` and is the demonstration.

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cylinder, EdgeRun, Error, HalfSpace, Picker, Point, Surface, SurfaceMeet, Vector,
    intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// The thin pipe is one wide, so a fat radius is also the ratio between them.
const THIN: f64 = 1.0;

/// Which way both pipes run in the tilted pair, before it is scaled to one.
const TILTED: [f64; 3] = [0.3, 0.5, 0.7];

/// Which way the step from one axis to the other leans, before it is made
/// square to the pipes and scaled.
const OFFSET_LEANS: [f64; 3] = [0.1, 0.2, 1.3];

/// The two ratios either side of `2.2029765594779596`, where the tilted pair used
/// to stop being told apart one way round and not the other.
const UNDER_THE_OLD_FLIP: f64 = 2.20;
const OVER_THE_OLD_FLIP: f64 = 2.21;

/// The published sentence is arithmetic about `r₁² − across²`, and the
/// arithmetic holds up: `2²⁶` is exactly where the gap between neighbouring
/// `f64` at `r₁²` grows to the whole of the answer.
///
/// `across` is `r₁ − r₂²/(2r₁)` for a thin pipe on a fat one's skin, so
/// `r₁² − across²` is `r₂²·(1 − r₂²/(4r₁²))` — two numbers of size `r₁²`
/// subtracted to leave `r₂²`. An enclosure of `r₁²` is a step of `r₁²` wide
/// whatever else happens, and this is the ratio at which that step swallows the
/// answer whole.
///
/// What the next test says is that no line of the kernel does this subtraction.
#[test]
fn one_step_of_the_fat_radius_squared_grows_to_the_whole_answer_at_a_ratio_of_two_to_the_26() {
    let answer = THIN * THIN;

    let fat = 2.0_f64.powi(26);
    assert_eq!(fat, 67_108_864.0, "the 67 m pipe the doc comment names");
    assert_eq!(
        one_step_at(fat * fat),
        answer,
        "one step at r₁² is the whole of r₂², so an enclosure of the difference \
         is as wide as the difference and straddles zero"
    );

    let half_as_fat = 2.0_f64.powi(25);
    assert_eq!(
        one_step_at(half_as_fat * half_as_fat),
        answer / 4.0,
        "one binary order down the step is a quarter of the answer, because \
         halving the ratio quarters r₁²"
    );
}

/// **The published bound is not where the kernel gives up.** The pair the
/// sentence describes, at the ratio it names and on past `2⁵⁰`, is named in two
/// straight runs both ways round.
///
/// `two_lines_between_two_pipes` takes `X` from the discriminant — the rest of
/// the same doc comment says so — and never subtracts `across²` from anything.
/// So `2²⁶` bounds a line of code that is not there.
#[test]
fn the_kernel_names_that_very_pair_at_two_to_the_26_and_on_past_two_to_the_50() {
    for power in [26, 30, 40, 50] {
        let fat = 2.0_f64.powi(power);
        let pair = the_pair(Vector::new(0.0, 0.0, 1.0), fat);
        for first in [0, 1] {
            assert_eq!(
                how_it_is_named(&pair, first),
                Ok(2),
                "a pipe 2^{power} wide against one 1 wide on its skin, \
                 {} written first",
                if first == 0 {
                    "the fat one"
                } else {
                    "the thin one"
                }
            );
        }
    }
}

/// The pair that used to flip between `2.20` and `2.21` is named at both, both
/// ways round.
///
/// This is the fix, at the ratio the defect was measured at. Before the picking
/// plane was anchored between the lines, `2.21` with the fat pipe written first
/// came back [`Error::CannotNameWhichPieceOfACrossingThisIs`] while the same pair
/// written the other way round was named — §140's shape, which is a bug and not a
/// taste.
#[test]
fn the_tilted_pair_that_used_to_flip_at_a_ratio_of_2_2_is_named_at_both_ratios_both_ways_round() {
    for ratio in [UNDER_THE_OLD_FLIP, OVER_THE_OLD_FLIP] {
        let pair = the_pair(the_tilted_way(), ratio);
        for first in [0, 1] {
            assert_eq!(
                how_it_is_named(&pair, first),
                Ok(2),
                "at a ratio of {ratio}, {} written first",
                if first == 0 {
                    "the fat one"
                } else {
                    "the thin one"
                }
            );
        }
    }
}

/// No direction gives up under a radius ratio of 100 any more, where nineteen of
/// these thirty did.
///
/// Ten pipe directions against three directions for the step between the axes,
/// swept from a ratio of `0.6` — below which the thin pipe swallows the fat one
/// and the pair is not two lines at all — up to `100`, both ways round. The
/// lowest refusal before the anchor moved was `1.3225450275326378`.
#[test]
fn every_direction_is_told_apart_at_every_ratio_up_to_a_hundred_both_ways_round() {
    let mut pairs = 0;
    for way in WAYS.map(|w| Vector::new(w[0], w[1], w[2])) {
        for leaning in LEANINGS.map(|s| Vector::new(s[0], s[1], s[2])) {
            for step in 0..200 {
                let ratio = 0.6 + f64::from(step) * 0.5;
                let pair = the_pair_leaning(way, leaning, ratio);
                for first in [0, 1] {
                    assert_eq!(
                        how_it_is_named(&pair, first),
                        Ok(2),
                        "{way:?} stepping along {leaning:?} at a ratio of \
                         {ratio}, {} written first",
                        if first == 0 {
                            "the fat one"
                        } else {
                            "the thin one"
                        }
                    );
                    pairs += 1;
                }
            }
        }
    }
    assert_eq!(
        pairs, 12_000,
        "thirty directions by two hundred ratios by two orders"
    );
}

/// The normal still leans `26.24°` off the direction that separates the two
/// lines. **That is no longer what decides the answer**, and this test is here to
/// say the lean did not go away — the anchor did the work.
///
/// `through_a_point_square_to` falls back to `along × x̂` here, exactly as
/// before, and the stored normal is unchanged. What changed is the offset: it
/// used to be `normal · here`, the first pipe's axis, which reads zero.
#[test]
fn the_picking_plane_still_leans_26_degrees_and_no_longer_decides_the_answer() {
    let picker = the_picking_plane_the_fat_pipe_first_used(UNDER_THE_OLD_FLIP);
    let [_, dy, dz] = the_tilted_way_scaled_to_one().components();
    assert_eq!(
        picker.normal(),
        Vector::new(0.0, dz, -dy),
        "the first of the three fallbacks, stored unchanged"
    );
    assert_ne!(
        picker.offset(),
        0.0,
        "and no longer anchored through the first pipe's axis"
    );

    let normal = picker.normal().normalized().expect("a normal has length");
    let (sideways, up) = the_frame_square_to_the_pipes();
    let leans_along_the_step = normal.dot(sideways).abs();
    let leans_along_the_separator = normal.dot(up).abs();
    assert!(
        (leans_along_the_step - 0.442_084_942_084_942_1).abs() < 1e-12
            && (leans_along_the_separator - 0.896_973_190_224_631_8).abs() < 1e-12,
        "the normal reads {leans_along_the_step} along the step between the axes \
         and {leans_along_the_separator} along the direction that separates the \
         two lines"
    );

    let cot = leans_along_the_separator / leans_along_the_step;
    assert!(
        (cot - 2.028_961_190_115_105_7).abs() < 1e-12,
        "so an axis-anchored plane would stop separating them at a ratio of {cot}"
    );
    assert!(
        the_ratio_across_over_x(UNDER_THE_OLD_FLIP) < cot
            && cot < the_ratio_across_over_x(OVER_THE_OLD_FLIP),
        "and `across/X` crosses {cot} between the two ratios the test above \
         names, {} then {} — which is what used to separate a named pair from a \
         refused one and now separates nothing",
        the_ratio_across_over_x(UNDER_THE_OLD_FLIP),
        the_ratio_across_over_x(OVER_THE_OLD_FLIP)
    );
}

/// The stored offset lands between the two lines' own readings, which is the
/// whole of what the fix does.
///
/// At a ratio the old anchor could not tell apart, the two lines read `−1.75` and
/// `−0.0033` against the stored normal — both under an axis-anchored plane's
/// reading of zero. The plane the kernel builds now reads between them.
#[test]
fn the_stored_offset_lands_between_the_two_lines_readings() {
    let picker = the_picking_plane_the_fat_pipe_first_used(OVER_THE_OLD_FLIP);
    let stored = picker.normal();

    let [under, over] = where_the_two_lines_read_against(stored, OVER_THE_OLD_FLIP);
    assert!(
        under < picker.offset() && picker.offset() < over,
        "the two lines read {under} and {over} against the stored normal, and \
         the plane is anchored at {}",
        picker.offset()
    );

    assert!(
        over < 0.0,
        "where the old anchor, the fat pipe's axis through the origin, read \
         zero and held both of them on one side"
    );
}

/// Past `2⁵⁴` the pair stops being the pair that was asked for, and the
/// axis-aligned control is the demonstration.
///
/// The thin pipe's axis is placed at `fat` along a step of length one. Once one
/// `f64` step at `fat` grows past the thin radius, the rounded point is no longer
/// on the fat pipe's skin and the two pipes genuinely miss each other — the
/// kernel answering [`SurfaceMeet::Nothing`] is right about the numbers it was
/// given. Where the step runs along a coordinate direction that placement is
/// exact, nothing rounds, and the pair keeps being named far past it.
#[test]
fn past_two_to_the_54_it_is_the_asked_for_pair_that_is_gone_not_the_answer() {
    let fat = 2.0_f64.powi(54);
    assert!(
        one_step_at(fat) > THIN,
        "one f64 step at {fat} is {}, wider than the thin pipe",
        one_step_at(fat)
    );

    let tilted = the_pair_leaning(the_tilted_way(), the_offset_leaning(), fat);
    assert_eq!(
        how_it_is_named(&tilted, 0),
        Ok(0),
        "so the tilted pair's axes are no longer a fat radius apart and the \
         pipes miss"
    );

    for power in [54, 60, 70] {
        let exact_step = the_pair_leaning(
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(0.0, 1.0, 0.0),
            2.0_f64.powi(power),
        );
        for first in [0, 1] {
            assert_eq!(
                how_it_is_named(&exact_step, first),
                Ok(2),
                "while a step along a coordinate direction rounds nothing and \
                 2^{power} is still named"
            );
        }
    }
}

/// The ten pipe directions swept.
const WAYS: [[f64; 3]; 10] = [
    [0.0, 0.0, 1.0],
    TILTED,
    [1.0, 1.0, 1.0],
    [1.0, 2.0, 3.0],
    [1.0, 0.0, 1.0],
    [7.0, 1.0, 1.0],
    [1.0, 7.0, 13.0],
    [0.1, 0.9, 0.05],
    [3.0, 4.0, 5.0],
    [11.0, 13.0, 17.0],
];

/// The three directions the step between the axes is made to lean along.
const LEANINGS: [[f64; 3]; 3] = [OFFSET_LEANS, [1.0, 0.0, 0.0], [0.7, 0.7, 0.1]];

/// A fat pipe of radius `fat` through the origin, and a pipe one wide whose axis
/// lies on its skin — `δ = r₁`, the pair the doc comment's bound is about.
///
/// The fat pipe is written first.
fn the_pair(way: Vector, fat: f64) -> [Surface; 2] {
    the_pair_leaning(way, the_offset_leaning(), fat)
}

fn the_pair_leaning(way: Vector, leaning: Vector, fat: f64) -> [Surface; 2] {
    let [x, y, z] = (the_step_square_to(way, leaning) * fat).components();
    [(Point::origin(), fat), (Point::new(x, y, z), THIN)].map(|(origin, radius)| {
        Surface::Cylinder(
            Cylinder::new(Axis::new(origin, way).expect("an axis"), radius).expect("a pipe"),
        )
    })
}

/// How many straight runs the pair is named in, with `first` written first — or
/// what it was refused with.
fn how_it_is_named(pair: &[Surface; 2], first: usize) -> Result<usize, Error> {
    match intersect(
        &pair[first],
        &pair[1 - first],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Ok(SurfaceMeet::Curves(runs)) => Ok(runs.len()),
        Ok(SurfaceMeet::Nothing) => Ok(0),
        Ok(other) => panic!("named as something else entirely: {other:?}"),
        Err(refused) => Err(refused),
    }
}

/// The half-space the fat pipe written first picked its first run with, read
/// straight out of the answer.
fn the_picking_plane_the_fat_pipe_first_used(fat: f64) -> HalfSpace {
    let pair = the_pair(the_tilted_way(), fat);
    let Ok(SurfaceMeet::Curves(runs)) = intersect(
        &pair[0],
        &pair[1],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) else {
        panic!("the fat pipe written first names the crossing at {fat}");
    };
    let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = runs[0].run() else {
        panic!("a picked piece of a crossing");
    };
    let [Picker::InFrontOf(picker)] = picked_by[..] else {
        panic!("one flat picker");
    };
    picker
}

/// Where the two lines read against `normal`, measured from the origin, smaller
/// first.
///
/// The two lines sit at `across` along the step between the axes and `±X` along
/// the direction square to both, which for a pipe on a fat one's skin is
/// `r₁ − r₂²/(2r₁)` and `√(r₂² − r₂⁴/(4r₁²))`. Neither subtracts two nearly
/// equal large numbers, so these are the true readings to a rounding or two.
///
/// The fat pipe's axis passes through the origin, so a reading against `normal`
/// is directly comparable with a half-space's stored offset.
fn where_the_two_lines_read_against(normal: Vector, fat: f64) -> [f64; 2] {
    let (sideways, up) = the_frame_square_to_the_pipes();
    let middle = normal.dot(sideways) * (fat - THIN * THIN / (2.0 * fat));
    let either_way = normal.dot(up) * the_half_chord(fat);
    let [one, other] = [middle + either_way, middle - either_way];
    if one <= other {
        [one, other]
    } else {
        [other, one]
    }
}

/// Half the distance between the two lines: `√(r₁² − across²)`, taken the way
/// that does not cancel.
fn the_half_chord(fat: f64) -> f64 {
    (THIN * THIN - THIN * THIN * THIN * THIN / (4.0 * fat * fat)).sqrt()
}

/// How far out along the step the middle of the two lines sits, against half the
/// distance between them — the ratio an axis-anchored plane had to keep under
/// `cot θ`.
fn the_ratio_across_over_x(fat: f64) -> f64 {
    (fat - THIN * THIN / (2.0 * fat)) / the_half_chord(fat)
}

/// The two directions square to the tilted pipes: along the step between the
/// axes, and the one that separates the two lines.
fn the_frame_square_to_the_pipes() -> (Vector, Vector) {
    let sideways = the_step_square_to(the_tilted_way(), the_offset_leaning());
    (
        sideways,
        the_tilted_way_scaled_to_one()
            .cross(sideways)
            .normalized()
            .expect("square to both"),
    )
}

/// `leaning` made square to `way` and scaled to length one.
fn the_step_square_to(way: Vector, leaning: Vector) -> Vector {
    Axis::new(Point::origin(), way)
        .expect("an axis")
        .direction()
        .cross(leaning)
        .normalized()
        .expect("the step leans off the pipes")
}

/// The gap between `size` and the next `f64` above it.
fn one_step_at(size: f64) -> f64 {
    f64::from_bits(size.to_bits() + 1) - size
}

fn the_tilted_way() -> Vector {
    Vector::new(TILTED[0], TILTED[1], TILTED[2])
}

fn the_tilted_way_scaled_to_one() -> Vector {
    Axis::new(Point::origin(), the_tilted_way())
        .expect("an axis")
        .direction()
}

fn the_offset_leaning() -> Vector {
    Vector::new(OFFSET_LEANS[0], OFFSET_LEANS[1], OFFSET_LEANS[2])
}
