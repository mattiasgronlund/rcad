//! §24.18's criterion 10 again, for the two curved kinds the first fuzz target
//! cannot draw.
//!
//! # Why there are two byte formats and not one wider one
//!
//! `every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs` is the sibling
//! of this file, and its header measures what widening the first format would
//! cost: 830 checked-in seeds, of which a repack gives a different kind to at
//! least one side of 92. A minimised seed is worth keeping only for the branch it
//! reached, so ninety-odd of them quietly meaning something else is a real cost.
//!
//! So [`fuzzing::two_surfaces_with_a_cone_or_a_torus_from`] is a **second**
//! format with a corpus of its own, and the first corpus keeps meaning what it
//! meant because nothing reads it differently. The cost of that is this file:
//! the same three oracles, replayed over the other decoder.
//!
//! # What this covers that the hand-written table does not
//!
//! The table in the sibling file is 136 pairs of fixed `Surface` values. It
//! reaches cones and tori, and **a fuzzer cannot mutate a Rust literal**. These
//! are bytes, so `cargo fuzz` can bend any of them and keep what reaches
//! somewhere new — which is the whole argument for a corpus over a table, and
//! why the table's row and this file's rows are reported apart rather than added
//! up (§17.7).
//!
//! Every seed here was hand-written but two, and those two are the first
//! keepers a coverage-guided run sent back: `a_plane_square_to_a_huge_cones_axis`
//! and `a_plane_square_to_a_flatter_huge_cones_axis`, from the nights of
//! 2026-09-17 and 2026-09-16. Both are the same defect, which was in the
//! harness's own `scaled` rather than in the kernel — see the note on
//! [`fuzzing`]'s `scaled`. More keepers land in the same directory and these rows
//! move; the floors below are set low enough that they do not have to move with
//! them.
//!
//! # Counted by category, and not only in total
//!
//! A total says the corpus is large. It does not say the corpus still has a cone
//! against a ball in it. So the corpus row buckets every seed three ways and
//! puts a floor on each bucket:
//!
//! - **the nine kind pairs** a cone or a torus can be half of, each of which must
//!   be there at all;
//! - **the size band** the pair is written at, which is what puts the range
//!   refusals §28.2 measures inside a mutation's reach;
//! - **the pairs sharing one axis line**, which is where the degenerate answers
//!   live — a pipe touching a ring along its inner circle, a cone and a pipe
//!   about one axis.
//!
//! The near-tangent seeds are in here too — a plane on a cone's one line, a ball
//! inside a ring's inner circle — and they are counted through the kind-pair
//! bucket rather than a bucket of their own. Tangency is a property of the
//! answer and not of the pair, and most of these pairs are refused, so a
//! tangency bucket would be counting refusals rather than tangencies.
//!
//! # Where the floors come from
//!
//! Measured on this tree on 2026-09-14, and the corpus row again on 2026-09-17
//! with the two keepers in it, by
//!
//! ```text
//! cargo nextest run -p rcad-geom \
//!     every_cone_or_torus_pair_the_fuzzer_draws_is_named_honestly:: \
//!     --no-capture --no-fail-fast
//! ```
//!
//! | row | pairs | points | orders | scalings |
//! |---|---|---|---|---|
//! | corpus | 34 of 38 seeds | 377 | 14 | 28 |
//! | soak | 94 060 of 200 000 draws | 830 | 11 435 | 22 770 |
//!
//! Each floor is roughly three quarters of what was measured, the convention the
//! sibling file set, so it catches an oracle that has gone silent without failing
//! on ordinary drift.
//!
//! **Fourteen of the corpus's thirty-four pairs were named both ways round, and
//! the other twenty were refused both ways.** That is the shape the sibling file's
//! table row has too, and for the same reason: a cone against a ball or another
//! cone, and a ring against anything curved, are curves of degree four and eight
//! that nothing names yet. The order oracle is the one that still says something
//! across a refusal, because it compares a refusal against the other way round.
//!
//! # What the soak actually draws, measured rather than assumed
//!
//! Of the 188 120 surfaces in the soak's decoded pairs: 106 667 cones, 22 950
//! planes, 22 908 balls, 22 678 pipes and 12 917 rings. The cone's share is the
//! forced slot showing up — it is the forced kind on half of all draws, and it
//! almost never refuses, needing only a direction and a slope that are not zero.
//!
//! **The ring is the kind that refuses**, which is why it is last even though the
//! kinds byte asks for it about as often as for a pipe: [`rcad_geom::Torus::new`]
//! wants both radii positive and the tube narrower than the ring, and numbers
//! drawn out of random bytes rarely oblige. That is a refusal and not a repair,
//! and [`fuzzing::two_surfaces_with_a_cone_or_a_torus_from`] says why it is left
//! that way.
//!
//! 47 % of draws decode at all, against 99.97 % for the first format. The
//! difference is the ring, and it is the price of a forced slot that can be one.

#![forbid(unsafe_code)]

mod fuzzing;

use std::collections::BTreeMap;
use std::path::PathBuf;

use fuzzing::{
    Asked, MOST_BYTES_A_CONE_OR_TORUS_PAIR_READS, bytes_for_a_cone_or_torus_pair,
    every_oracle_holds_of, two_surfaces_with_a_cone_or_a_torus_from,
};
use rcad_geom::{Axis, Surface};

/// How many pseudo-random byte strings the soak draws.
const SOAK: usize = 200_000;

/// **A seed written by hand comes back as the numbers it was asked for.**
///
/// Every seed in the corpus was written through
/// [`fuzzing::bytes_for_a_cone_or_torus_pair`], so a silent slip in it is a
/// corpus of near-misses dressed up as round numbers — and there was one. The
/// exponent came out a step too low for every number whose `log2` is `31 + 5k`,
/// the whole number landed on exactly `2^31`, and `as i32` saturated one below
/// it. A radius asked for as `2.0` was written out as `1.9999999990686774`, and
/// a pipe meant to touch a ring along its inner circle missed it by an ulp.
///
/// `2.0` is in the sweep below at `k = 1`, and so is every other power of two the
/// corpus is written from.
#[test]
fn every_power_of_two_written_into_a_seed_decodes_back_to_itself() {
    for k in -120..=120 {
        let radius = 2f64.powi(k);
        let bytes = bytes_for_a_cone_or_torus_pair(
            [CONE, PIPE],
            &[
                0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, radius,
            ],
        );
        let decoded = two_surfaces_with_a_cone_or_a_torus_from(&bytes);
        let Some([Surface::Cone(cone), Surface::Cylinder(pipe)]) = decoded else {
            panic!("2^{k} as a radius decoded to {decoded:?}");
        };
        assert_eq!(pipe.radius(), radius, "2^{k} did not come back as itself");
        assert_eq!(cone.slope(), 1.0, "the slope beside 2^{k} did not either");
    }
}

/// What the kinds byte calls a pipe and a cone.
const PIPE: u8 = 2;
const CONE: u8 = 3;

/// Where the checked-in corpus lives.
fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fuzz/corpus/where_a_cone_or_a_torus_crosses")
}

/// Every seed checked in beside the fuzz target, replayed and bucketed.
#[test]
fn every_seed_in_the_checked_in_corpus_names_its_crossing_honestly() {
    let mut seeds = 0;
    let mut named = 0;
    let mut asked = Asked::default();
    let mut kind_pairs: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut bands: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut sharing_an_axis = 0;

    let listing = std::fs::read_dir(corpus()).expect("the corpus is checked in beside the target");
    for entry in listing {
        let path = entry.expect("a readable directory entry").path();
        if !path.is_file() {
            continue;
        }
        let bytes = std::fs::read(&path).expect("a readable seed");
        seeds += 1;
        let Some(pair) = two_surfaces_with_a_cone_or_a_torus_from(&bytes) else {
            continue;
        };
        named += 1;
        assert!(
            pair.iter()
                .any(|surface| matches!(surface, Surface::Cone(_) | Surface::Torus(_))),
            "{} decoded to a pair with neither a cone nor a torus in it: {pair:?}",
            path.display()
        );
        *kind_pairs.entry(what_pair_this_is(&pair)).or_default() += 1;
        *bands
            .entry(what_band_this_is_written_at(&pair))
            .or_default() += 1;
        sharing_an_axis += usize::from(the_two_share_one_axis_line(&pair));
        asked = asked.and(every_oracle_holds_of(pair));
    }

    println!("{seeds} seeds in the corpus, {named} of them a pair of surfaces, {asked:?}");
    println!("  kind pairs: {kind_pairs:?}");
    println!("  bands: {bands:?}");
    println!("  sharing one axis line: {sharing_an_axis}");

    assert!(seeds >= 30, "only {seeds} seeds are checked in");
    assert!(
        named * 2 > seeds,
        "only {named} of {seeds} seeds decoded to a pair, so the corpus has drifted away from \
         the thing it is a corpus of"
    );
    for wanted in EVERY_KIND_PAIR_WITH_A_CONE_OR_A_TORUS_IN_IT {
        assert!(
            kind_pairs.contains_key(wanted),
            "no seed in the corpus is a {wanted}: {kind_pairs:?}"
        );
    }
    for (band, floor) in [("tiny", 3), ("ordinary", 18), ("huge", 3)] {
        let counted = bands.get(band).copied().unwrap_or_default();
        assert!(
            counted >= floor,
            "only {counted} seeds are written at {band} sizes: {bands:?}"
        );
    }
    assert!(
        sharing_an_axis >= 7,
        "only {sharing_an_axis} seeds put both surfaces on one axis line"
    );
    every_oracle_was_reached(asked, 264, 9, 18);
}

/// The soak, over the whole decoded space.
///
/// A cheap deterministic generator rather than a proptest strategy, for the
/// reason the sibling file gives: what is being explored is the *bytes*, and a
/// strategy would put a shape on them that the fuzzer's own mutations do not
/// have.
#[test]
fn a_soak_over_pseudo_random_bytes_names_every_crossing_honestly() {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut drawn = 0;
    let mut named = 0;
    let mut asked = Asked::default();
    let mut kinds: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut bytes = vec![0u8; MOST_BYTES_A_CONE_OR_TORUS_PAIR_READS];

    for _ in 0..SOAK {
        for byte in &mut bytes {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *byte = state as u8;
        }
        drawn += 1;
        if let Some(pair) = two_surfaces_with_a_cone_or_a_torus_from(&bytes) {
            named += 1;
            assert!(
                pair.iter()
                    .any(|surface| matches!(surface, Surface::Cone(_) | Surface::Torus(_))),
                "these bytes decoded to a pair with neither a cone nor a torus in it.\n  \
                 bytes {bytes:?}\n  pair {pair:?}"
            );
            for surface in &pair {
                *kinds.entry(what_kind_this_is(surface)).or_default() += 1;
            }
            asked = asked.and(every_oracle_holds_of(pair));
        }
    }

    println!("{drawn} byte strings drawn, {named} of them a pair of surfaces, {asked:?}");
    println!("  surfaces drawn: {kinds:?}");
    assert!(
        named * 3 > drawn,
        "only {named} of {drawn} draws named a pair, so the decoder is refusing more of the \
         space than the ring alone accounts for"
    );
    every_oracle_was_reached(asked, 620, 8_500, 17_000);
}

/// The nine unordered kind pairs a cone or a torus can be half of.
///
/// Written out rather than generated, so that a kind pair going missing from the
/// corpus fails by name.
const EVERY_KIND_PAIR_WITH_A_CONE_OR_A_TORUS_IN_IT: [&str; 9] = [
    "ball and cone",
    "ball and ring",
    "cone and cone",
    "cone and pipe",
    "cone and plane",
    "cone and ring",
    "pipe and ring",
    "plane and ring",
    "ring and ring",
];

/// Which of the nine kind pairs this is, named the same way round however the
/// pair was drawn.
fn what_pair_this_is(pair: &[Surface; 2]) -> &'static str {
    let mut names = [what_kind_this_is(&pair[0]), what_kind_this_is(&pair[1])];
    names.sort_unstable();
    let [first, second] = names;
    EVERY_KIND_PAIR_WITH_A_CONE_OR_A_TORUS_IN_IT
        .into_iter()
        .find(|wanted| *wanted == format!("{first} and {second}"))
        .unwrap_or_else(|| panic!("{first} and {second} has no cone and no torus in it"))
}

/// `Surface` is `#[non_exhaustive]` and this is another crate, so the wildcard is
/// forced. All five arms that exist are named above it, and the wildcard panics:
/// a sixth kind quietly bucketed as one of these five is how a category count
/// stops counting anything (§48.2).
fn what_kind_this_is(surface: &Surface) -> &'static str {
    match surface {
        Surface::Plane(_) => "plane",
        Surface::Sphere(_) => "ball",
        Surface::Cylinder(_) => "pipe",
        Surface::Cone(_) => "cone",
        Surface::Torus(_) => "ring",
        other => panic!("nothing here can name {other:?}"),
    }
}

/// Whether the pair is written tiny, at ordinary sizes, or huge.
///
/// The band is the biggest length either surface is written with. **Only
/// lengths**: a plane's normal and a cone's slope are ratios, and scaling them
/// tilts the shape instead of resizing it, which is the same distinction
/// [`fuzzing`]'s own scale oracle draws.
///
/// A pair written entirely out of zeroes — a cone at the origin against a plane
/// through it — has no size at all, and is counted as ordinary rather than as
/// the smallest thing there is.
fn what_band_this_is_written_at(pair: &[Surface; 2]) -> &'static str {
    let biggest = pair
        .iter()
        .flat_map(every_length_of)
        .map(f64::abs)
        .filter(|length| *length > 0.0)
        .fold(0.0_f64, f64::max);
    if biggest == 0.0 {
        return "ordinary";
    }
    if biggest < 2f64.powi(-200) {
        "tiny"
    } else if biggest > 2f64.powi(200) {
        "huge"
    } else {
        "ordinary"
    }
}

fn every_length_of(surface: &Surface) -> Vec<f64> {
    match surface {
        Surface::Plane(flat) => vec![flat.coefficients()[3]],
        Surface::Sphere(ball) => {
            let mut out = ball.centre().coordinates().to_vec();
            out.push(ball.radius());
            out
        }
        Surface::Cylinder(pipe) => {
            let mut out = pipe.axis().origin().coordinates().to_vec();
            out.push(pipe.radius());
            out
        }
        Surface::Cone(cone) => cone.axis().origin().coordinates().to_vec(),
        Surface::Torus(ring) => {
            let mut out = ring.axis().origin().coordinates().to_vec();
            out.push(ring.ring_radius());
            out.push(ring.tube_radius());
            out
        }
        other => panic!("nothing here can measure {other:?}"),
    }
}

/// Whether both surfaces are drawn about one and the same line.
///
/// Both directions equal — [`Axis::new`] scales every direction to length one, so
/// two axes written parallel come back identical — and the step from one origin
/// to the other along that line, which is what a zero cross product says.
fn the_two_share_one_axis_line(pair: &[Surface; 2]) -> bool {
    let (Some(one), Some(other)) = (axis_of(&pair[0]), axis_of(&pair[1])) else {
        return false;
    };
    if one.direction() != other.direction() {
        return false;
    }
    let [x, y, z] = one.origin().coordinates();
    let [a, b, c] = other.origin().coordinates();
    let step = rcad_geom::Vector::from([a - x, b - y, c - z]);
    step.cross(one.direction()).components() == [0.0, 0.0, 0.0]
}

fn axis_of(surface: &Surface) -> Option<Axis> {
    match surface {
        Surface::Cylinder(pipe) => Some(pipe.axis()),
        Surface::Cone(cone) => Some(cone.axis()),
        Surface::Torus(ring) => Some(ring.axis()),
        Surface::Plane(_) | Surface::Sphere(_) => None,
        other => panic!("nothing here knows whether {other:?} has an axis"),
    }
}

/// Insist that each oracle was reached often enough to have been asserting
/// anything.
///
/// A floor apiece and not one shared floor, for the reason the sibling file
/// gives: the three skip for entirely different reasons, and a single count
/// would let one of them fall to nothing behind the other two.
fn every_oracle_was_reached(asked: Asked, points: usize, orders: usize, scalings: usize) {
    assert!(
        asked.points >= points,
        "only {} points were asked about: {asked:?}",
        asked.points
    );
    assert!(
        asked.orders >= orders,
        "only {} pairs were named both ways round: {asked:?}",
        asked.orders
    );
    assert!(
        asked.scalings >= scalings,
        "only {} pairs were named at two scales: {asked:?}",
        asked.scalings
    );
}
