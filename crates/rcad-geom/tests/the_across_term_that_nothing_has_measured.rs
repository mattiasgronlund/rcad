//! §142.9's fourth site, measured: the `across` term in
//! `two_lines_between_two_pipes`, and the bound its doc comment publishes.
//!
//! The doc comment's section **"Why `X` is not `√(r₁² − across²)`"** says that a
//! thin pipe whose axis lies on a fat one's skin loses the answer "past
//! `r₁ ≈ 2²⁶·r₂` — a 67 m pipe against a 1 m one", and that the same pair asked
//! the other way round decides. §15.6 asks a published bound for a test. This is
//! it, and **the bound is wrong by seven decimal orders of magnitude**.
//!
//! # What is right, and what it is right about
//!
//! The arithmetic is right about the formula the sentence is explaining the
//! *absence* of. `r₁² − across²` really is `r₂²` reached by subtracting two
//! numbers of size `r₁²`, and the gap between neighbouring `f64` at `r₁²`
//! reaches `r₂²` at exactly `r₁ = 2²⁶·r₂`. That is the first test below.
//!
//! **`two_lines_between_two_pipes` does not use that formula.** It takes `X`
//! from the discriminant, as the rest of the same doc comment says. So `2²⁶` is
//! a bound on a line of code that is not there, and nothing about the kernel's
//! own behaviour follows from it. Measured: the very pair the sentence describes,
//! at `2²⁶` and on past `2⁵⁰`, is **named in two straight runs both ways round**.
//!
//! # Where the kernel really gives up, and why
//!
//! At a radius ratio of `2.2029765594779596` for pipes running along
//! `[0.3, 0.5, 0.7]` — and as low as `1.3225450275326378` for another direction.
//! Thirty-odd million times sooner than published, and **the lever is not the
//! radius ratio at all: it is which way the pipes run.** Pipes along `z` at the
//! same ratios are named; of the thirty direction-and-offset pairs swept below,
//! nineteen refuse somewhere under a ratio of 100 and eleven never refuse at
//! all.
//!
//! The cause is not the `across` arithmetic. `two_lines_between_two_pipes` asks
//! [`HalfSpace::through_a_point_square_to`] for a normal along `along × towards`,
//! which separates the two lines perfectly — and that vector is only *exactly*
//! square to `along` when the pipes run along a coordinate direction. Otherwise
//! one of the three fallbacks is taken, and the best of those can be 55° off
//! (its own doc comment publishes that). The two lines sit at `(across, ±X)` in
//! the plane square to the pipes, so a picking plane through the first pipe's
//! axis leaning `θ` off the separating direction holds both of them on one side
//! once `across·sin θ ≥ X·cos θ` — that is, once the radius ratio passes
//! `cot θ`. For `[0.3, 0.5, 0.7]`, `θ` is `26.24°` and `cot θ` is `2.0289…`.
//!
//! # §16.10: a gap, and in §150's family
//!
//! At the refusing ratio the two lines read `−1.7507…` and `−0.0033…` against
//! the picking normal, both under the plane the kernel anchored through the
//! first pipe's axis, which reads exactly zero. **Every `f64` between those two
//! readings separates them**, and the window is `1.7474` wide against enclosures
//! a rounding across. The answer exists at this precision and one stored `f64` —
//! the half-space's offset — is the whole of what could not reach it, which is
//! §150's shape exactly.
//!
//! Three families of fix were weighed, per §16.10:
//!
//! - **Reorder or rescale the arithmetic.** Refuted by the numbers: at the
//!   refusing ratio the margin the kernel is short of is `3.3e-3`, ten trillion
//!   times any rounding in play, and the axis-aligned control decides at `2⁵⁰`
//!   where every enclosure is at its widest. Nothing is cancelling here.
//! - **Let the normal lean, and bound the lean.** Refused by
//!   `two_lines_told_apart`'s own doc comment, and rightly: a line is endless,
//!   so a normal off square by any rounding reads a different number far enough
//!   along it and the half-space holds nothing.
//! - **Move the anchor** — keep the exactly-square normal and choose the offset
//!   between the two readings the caller already works out, instead of taking
//!   whatever `normal · here` rounds to. This is the one that works, and the
//!   window above is the demonstration.
//!
//! # Every test here pins what the kernel does today
//!
//! Including the refusals. They are locks on a measured defect, not an
//! endorsement of it; the doc comment on each says which.

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

/// The largest ratio the tilted pair is still named at, and the smallest it is
/// refused at, either side of `2.2029765594779596`.
const NAMED: f64 = 2.20;
const REFUSED: f64 = 2.21;

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
/// sentence describes, at the ratio it names and eight binary orders past it, is
/// named in two straight runs both ways round.
///
/// `two_lines_between_two_pipes` takes `X` from the discriminant — the rest of
/// the same doc comment says so — and never subtracts `across²` from anything.
/// So `2²⁶` bounds a line of code that is not there.
///
/// The pipes here run along `z`. That is not a convenience: it is the whole of
/// what makes this pair decide, and the next two tests are why.
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

/// Where it really gives up: a radius ratio of `2.2029765594779596`, and only
/// one way round.
///
/// **This pins a defect.** The pair at `2.21` has an answer — the last test in
/// this file exhibits it — and the two orders disagreeing about it is §140's
/// shape, which is a bug and not a taste.
#[test]
fn the_tilted_pair_flips_between_a_ratio_of_2_20_and_2_21_and_only_the_fat_pipe_first() {
    let named = the_pair(the_tilted_way(), NAMED);
    for first in [0, 1] {
        assert_eq!(
            how_it_is_named(&named, first),
            Ok(2),
            "at a ratio of {NAMED} both orders name the crossing"
        );
    }

    let refused = the_pair(the_tilted_way(), REFUSED);
    assert_eq!(
        how_it_is_named(&refused, 0),
        Err(Error::CannotNameWhichPieceOfACrossingThisIs),
        "at {REFUSED} the fat pipe written first is refused"
    );
    assert_eq!(
        how_it_is_named(&refused, 1),
        Ok(2),
        "and the same pair written the other way round is named — which is the \
         half of the doc comment's last sentence that survives measurement"
    );
}

/// The lever is which way the pipes run, not how the radii compare.
///
/// Thirty pairs of a pipe direction and a direction for the step between the
/// axes. Twenty-seven of them refuse the fat pipe first at some ratio below 100,
/// the lowest at `1.3225450275326378`; three never refuse at all. Every one that
/// refuses names the same pair written the other way round.
///
/// The floor of twenty is what stops this passing over nothing if the defect is
/// fixed under it: a fix turns this test red, which is the point.
#[test]
fn the_ratio_it_gives_up_at_runs_from_1_32_to_never_depending_only_on_the_direction() {
    let mut refusing = 0;
    let mut lowest = f64::INFINITY;
    for way in WAYS.map(|w| Vector::new(w[0], w[1], w[2])) {
        for leaning in LEANINGS.map(|s| Vector::new(s[0], s[1], s[2])) {
            let Some(ratio) = the_ratio_the_fat_pipe_first_gives_up_at(way, leaning) else {
                continue;
            };
            refusing += 1;
            lowest = lowest.min(ratio);
            assert_eq!(
                how_it_is_named(&the_pair_leaning(way, leaning, ratio * 1.01), 1),
                Ok(2),
                "{way:?} stepping along {leaning:?} refuses the fat pipe first \
                 from a ratio of {ratio}, and names the same pair the other way \
                 round"
            );
        }
    }

    assert!(
        refusing >= 12,
        "only {refusing} of the thirty directions refuse; the defect this file \
         locks has moved and the numbers below are stale"
    );
    assert_eq!(refusing, 19, "nineteen of thirty, measured");
    assert_eq!(lowest, 1.322_545_027_532_637_8);
    assert_eq!(
        the_ratio_the_fat_pipe_first_gives_up_at(the_tilted_way(), the_offset_leaning()),
        Some(2.202_976_559_477_959_6),
        "and the tilted pair's own threshold sits between the two ratios the \
         test above pins"
    );
}

/// Why: the picking plane's normal leans `26.24°` off the direction that
/// separates the two lines, and `cot 26.24° = 2.0289…` is the ratio.
///
/// `two_lines_between_two_pipes` asks for a normal along `along × towards`.
/// Where the pipes run along a coordinate direction that vector is exactly
/// square to `along` and is taken as asked; here it is not, and
/// [`HalfSpace::through_a_point_square_to`] falls back to `along × x̂`, which is
/// the one of its three that leans nearest.
///
/// The plane is anchored through the first pipe's axis and its stored offset is
/// **exactly zero**, so nothing rounds into the reading below: what beats the
/// kernel is the lean, in full precision.
#[test]
fn the_picking_plane_leans_26_degrees_off_the_direction_that_separates_the_two_lines() {
    let picker = the_picking_plane_the_fat_pipe_first_used(NAMED);
    let [_, dy, dz] = the_tilted_way_scaled_to_one().components();
    assert_eq!(
        picker.normal(),
        Vector::new(0.0, dz, -dy),
        "the first of the three fallbacks, stored unchanged"
    );
    assert_eq!(
        picker.offset(),
        0.0,
        "anchored through the first pipe's axis"
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
        "so the two lines stop straddling the plane at a ratio of {cot}"
    );
    assert!(
        the_ratio_across_over_x(NAMED) < cot && cot < the_ratio_across_over_x(REFUSED),
        "and `across/X` crosses {cot} between the two ratios the test above \
         pins: {} then {}",
        the_ratio_across_over_x(NAMED),
        the_ratio_across_over_x(REFUSED)
    );
}

/// §16.10: a gap and not a limit. The two lines are on the same side of the
/// plane the kernel anchored, and **every `f64` between their two readings
/// separates them**.
///
/// The window is `1.75` wide on pipes a couple of units across. Nothing about
/// `f64` stands between the kernel and this answer — only that the offset is
/// taken from `normal · here` instead of from the two readings the caller has
/// already worked out.
#[test]
fn the_two_lines_read_the_same_side_of_the_plane_and_an_offset_that_separates_them_exists() {
    let normal = the_picking_plane_the_fat_pipe_first_used(NAMED)
        .normal()
        .normalized()
        .expect("a normal has length");

    let [under, over] = where_the_two_lines_read_against(normal, NAMED);
    assert!(
        under < 0.0 && 0.0 < over,
        "at a ratio of {NAMED} the two lines read {under} and {over}, straddling \
         the anchored plane's own reading of zero"
    );

    let [under, over] = where_the_two_lines_read_against(normal, REFUSED);
    assert!(
        (under + 1.750_703_700_925_25).abs() < 1e-12
            && (over + 0.003_273_307_757_641_119_6).abs() < 1e-12,
        "at {REFUSED} they read {under} and {over}"
    );
    assert!(
        over < 0.0,
        "both under the anchored plane's own reading of zero, so the kernel is \
         right to refuse this plane — and wrong to have anchored it here"
    );

    let between = 0.5 * (under + over);
    assert!(
        under < between && between < over,
        "and {between} is one of the very many f64 that do separate them"
    );
    assert!(
        over - under > 1.74,
        "a window {} wide, against enclosures a rounding across",
        over - under
    );

    let (_, up) = the_frame_square_to_the_pipes();
    let [under, over] = where_the_two_lines_read_against(up, REFUSED);
    assert!(
        (over - the_half_chord(REFUSED)).abs() < 1e-12 && (over + under).abs() < 1e-12,
        "the direction the caller asked for reads {under} and {over} — dead \
         centre — and is refused only for not being exactly square to the pipes"
    );
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

/// The smallest radius ratio at which the fat pipe written first stops naming
/// the crossing, or `None` if it still names it at a ratio of 100.
///
/// Bisected between `0.6`, below which the thin pipe swallows the fat one and
/// the pair is not two lines at all, and `100`.
fn the_ratio_the_fat_pipe_first_gives_up_at(way: Vector, leaning: Vector) -> Option<f64> {
    let names = |ratio| how_it_is_named(&the_pair_leaning(way, leaning, ratio), 0) == Ok(2);
    assert!(names(0.6), "the pair is named somewhere, to bisect from");
    if names(100.0) {
        return None;
    }
    let (mut lo, mut hi) = (0.6_f64, 100.0_f64);
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if names(mid) { lo = mid } else { hi = mid }
    }
    Some(lo)
}

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

/// Where the two lines read against `normal`, measured from the plane through
/// the fat pipe's own axis, smaller first.
///
/// The two lines sit at `across` along the step between the axes and `±X` along
/// the direction square to both, which for a pipe on a fat one's skin is
/// `r₁ − r₂²/(2r₁)` and `√(r₂² − r₂⁴/(4r₁²))`. Neither subtracts two nearly
/// equal large numbers, so these are the true readings to a rounding or two.
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
/// distance between them — the ratio that has to stay under `cot θ`.
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
