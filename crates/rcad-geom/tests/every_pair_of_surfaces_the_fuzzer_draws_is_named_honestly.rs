//! §24.18's criterion 10, and the part of it that runs in CI.
//!
//! # What runs where
//!
//! `fuzz/fuzz_targets/where_two_surfaces_cross.rs` is the `cargo-fuzz` target,
//! and it needs a nightly compiler for its sanitizer. `rust-toolchain.toml`
//! pins a stable one (§8.1), so **`just gate` does not run it** — `just fuzz`
//! does, on a toolchain a person asked for. `docs/decisions.md` §29 records that
//! plainly rather than leaving it to be discovered.
//!
//! What runs on every commit is this file, and it is the same property over the
//! same decoder: `crates/rcad-geom/tests/fuzzing/mod.rs` holds both, and the
//! fuzz target reaches across to that file rather than copying it.
//!
//! Three rows, and the two in this section are the ones that read bytes. The
//! **corpus** is what is checked in beside the target: a handful
//! of seeds written out by hand and named for the configuration each one is,
//! plus everything a coverage-guided run kept, minimised. Every one is replayed,
//! so a case a fuzzing run once found can never come back unnoticed — and one of
//! them is a case it did find, `two_balls_of_almost_one_size_almost_in_one_place`
//! (§29).
//!
//! The **soak** is a couple of hundred thousand pseudo-random byte strings,
//! which is not coverage-guided and does not pretend to be. What it buys is that
//! the property is exercised over the whole decoded space on a machine that has
//! no nightly compiler on it.
//!
//! **Not every seed decodes to a pair of surfaces, and that is right.** A
//! coverage-guided run keeps an input because it reached somewhere new, and the
//! decoder's own refusals — a radius of nothing, a direction of nothing — are
//! branches like any other.
//!
//! # A third row, because the bytes cannot draw a cone or a torus
//!
//! [`fuzzing::two_surfaces_from`] reads two bits per surface and asks for
//! `which % 3`, so the only kinds it ever names are a plane, a ball and a pipe.
//! `Surface` has had a `Cone` and a `Torus` since M4. Left there, the two
//! relation oracles would be green over a family that stopped growing when the
//! kernel did not — §48.2's failure exactly, and §50.9 sent this lane to avoid
//! it.
//!
//! The byte layout is not widened to fix that, and **not because a repack would
//! reinterpret the whole corpus — measured, it would not.** Over the 830
//! minimised seeds, promoting the two unread bits 4 and 5 to a third kind bit
//! apiece and reading five kinds instead of three gives a different kind to at
//! least one side of **92** seeds, 67 of them among the 749 that decode to a
//! pair. Handing the already-redundant kind value 3 to `Cone` and leaving the
//! layout alone moves **42**, 35 of them decoding. A seed whose two kind fields
//! come out unchanged reads the same bytes into the same pair, so those counts
//! are the whole of what either repack disturbs.
//!
//! The two counts are 2026-09-04's and did not move when §85's seed was added:
//! its kinds byte is `0x0a`, so both sides read kind 2 today, both read kind 2
//! under the wider layout — bits 4 and 5 are clear — and neither side is the
//! value 3 that the other repack rewrites. §88's seed is `0x0a` too, so the same
//! three sentences carry it, and the counts are unmoved again. So is §89's slow
//! unit, promoted 2026-09-13, and so are both of §91's, promoted the same day,
//! and so is §93's crash, promoted with the fix for it — which makes six seeds
//! the paragraph covers and none that it has had to be rewritten for.
//!
//! Two things settle it instead. **A minimised seed is worth keeping only for
//! the branch it reached**, so ninety-odd of them quietly meaning something else
//! is a real cost against a corpus of 830. And **the repack does not buy the
//! coverage it is for**: a torus reads eight numbers where a pipe reads seven,
//! so a torus against a cone, a pipe or another torus needs 76 bytes, and the
//! longest seed checked in is 71. Those pairs would decode to nothing whatever
//! the kind bits said. Widening the number budget as well moves every seed's
//! bytes and not just its first one.
//!
//! So the cones and tori come from
//! a **second family, reported separately and never blended into the byte rows**
//! (§17.7): a small table of shapes on the integer lattice, every pair of which
//! has a cone or a torus in it.
//!
//! # Where the floors below come from
//!
//! Every one was measured on this tree, on 2026-09-04, by
//!
//! ```text
//! cargo nextest run -p rcad-geom \
//!     every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly:: \
//!     --no-capture --no-fail-fast
//! ```
//!
//! | row | pairs | points | orders | scalings |
//! |---|---|---|---|---|
//! | corpus | 749 of 830 seeds | 4235 | 549 | 1097 |
//! | soak | 199 943 of 200 000 draws | 400 412 | 178 631 | 355 291 |
//! | cones and tori | 136 | 696 | 39 | 78 |
//!
//! The corpus row is 2026-09-13's, six seeds later than the cone-and-torus
//! one: §85's crash was promoted into the corpus, then §88's — the pair of
//! leaning pipes that named one crossing forwards and refused backwards — then
//! §89's slow unit, then the two §91 rescued, and last §93's crash. §88's decodes
//! to a pair and is decided both ways round now, which is one order and two
//! scalings of what the row gained.
//!
//! **§93 moved this row twice over, and only one of the two is its seed.** The
//! fix took orders from 547 to 548 and scalings from 1093 to 1095 with no seed
//! added at all: one pair already in the corpus was a side-by-side pipe pair at a
//! radius ratio that used to be named one way round and refused the other, and
//! closing that reaches it. Then §93's own seed — the crash the night of
//! 2026-09-12 found and the night after reported as nothing found — takes them to
//! 549 and 1097. Measured the way the others were, by adding it and taking it
//! away again, it brings one seed, one pair, one order, two scalings and **no
//! points**, because two endless straight lines are named rather than walked. It
//! costs no measurable time: 63.15 s against 63.61 s. Both of those numbers are
//! the slow seed two paragraphs down rather than this one, and §94 has since
//! taken them off the row — they are left as measured because what they were
//! measuring is the *difference* between them.
//!
//! **The second of §91's two is the cheap one, and it is why the points column
//! moved.** It is a slow unit written 2026-09-09 that was still sitting as its
//! only copy in an unlocked worktree four days later, and measured the same way
//! — added, then taken away again — it costs about 0.1 s and brings 18 points,
//! one order and two scalings. A seed that buys coverage for a tenth of a second
//! needs no argument beyond that it was one sweep from being deleted.
//!
//! **The first of §91's two cost 63 of this row's 65 seconds, and §94 took them
//! back.** Two near-perpendicular pipes, radius 2147482112 against radius
//! `4.3601508761683463e-106` — a ratio near 5e114 — crossing in two loops each as
//! wide as the thin pipe, sitting where the numbers are `2.38e-7` apart. The
//! marcher used to take the seed anyway and crawl to its million-point cap, which
//! took this row from 0.072 s to 63.2 s and brought one order, two scalings and
//! **no points at all**. It now turns the seed away in twenty microseconds and
//! the row reads 0.069 s.
//!
//! **The seed still brings no points, and it was never going to.** §91.3 called
//! this a gap and said the row would drop "when §91.3's gap is closed"; §94
//! measured it a **limit** instead — each loop is narrower than one step of the
//! grid it sits on by ninety-nine orders of magnitude, so no point that can be
//! written down is on both pipes, and there is no answer to reach. What the seed
//! earns its place with is the refusal being cheap and named:
//! `how_small_a_crossing_can_be_before_no_point_sits_on_it.rs` fails without §94
//! with `TooManyPointsAlongOneCrossing` after 35 seconds.
//!
//! That prediction said "§91's gap" until §93, which closed §91.2's and moved the
//! timing by 0.46 s — inside the noise. §91 recorded two gaps and this seed
//! witnesses the leaning row's, so naming the subsection was the difference
//! between a prediction and a guess — and the named prediction then turned out
//! right about the timing and wrong about the classification.
//!
//! **Three of the six were promoted for their time rather than their answer**,
//! and §89's is the one that shows what a closed gap looks like afterwards: two
//! crossed pipes, radius 2048 against radius `0.0152740478515625`, which before
//! §89 took 5.93 s to produce no points at all and now takes 227 µs to produce
//! ten. Those are the ten points, one order and two scalings between the
//! 2026-09-11 row and the one `b7bda36` published.
//! A seed that costs a fifth of a millisecond is worth keeping for the six
//! seconds it used to cost, and §89's own regression test is a claim with no
//! input behind it until this file is checked in beside it.
//!
//! **The points column then more than doubled, and that is §89.** The marcher
//! used to jump clean across a crossing when the budget handed to it was larger
//! than the crossing was — which is what the first of the two walks here does on
//! purpose — and publish a bound so wide that the second walk was asked for
//! something absurd and gave up. Corpus points went `1993` to `4207` and the
//! soak's `400 342` to `400 412`; no row lost any. The floors are left where they
//! were, which is now nearer a half of the corpus row than three quarters.
//!
//! **Not one of them is `rcad-fuzz`'s number.** That branch published three
//! floors measured before M4 and §33 moved the refusal rates under them, and
//! §50.9 records that everything it published is stale by 5× to 40×. Each floor
//! here was set at roughly three quarters of what was measured, so it catches an
//! oracle falling silent without failing on ordinary drift.
//!
//! # The cone-and-torus row moved, exactly as it was written to
//!
//! Naming the coaxial cone and pipe landed, and the row is the seven pairs
//! better for it: 97 of its 136 pairs are refused both ways round where 104 were
//! before, points 696 where they were 435, orders 39 where they were 32,
//! scalings 78 where they were 64. Seven cone-and-pipe pairs in the table share
//! an axis — two cones about `z` against two pipes about `z`, the raised cone
//! against the same two, and the cone about `x` against the pipe about `x` — and
//! each contributes one order and two scalings, which is the whole of the move.
//!
//! **The three floors stay where they were, and only one of them is still three
//! quarters of measured**: orders at 30 of 39, scalings at 48 of 78, points at
//! 300 of 696. They are left low on purpose. A cone against a ball or another
//! cone, and a torus against anything curved, are still refused, and naming any
//! of them turns refusals into answers and lifts all three counts again. A floor
//! re-cut to three quarters of today would have to be re-cut on each of those
//! landings, and what it is for — noticing an oracle that has gone silent — it
//! does at 300 exactly as well as at 522.

#![forbid(unsafe_code)]

mod fuzzing;

use std::path::PathBuf;

use fuzzing::{Asked, MOST_BYTES_A_PAIR_READS, every_oracle_holds_of, two_surfaces_from};
use rcad_geom::{Axis, Cone, Cylinder, Plane, Point, Sphere, Surface, Torus, Vector};

/// How many pseudo-random byte strings the soak draws.
const SOAK: usize = 200_000;

/// Where the checked-in corpus lives.
fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/corpus/where_two_surfaces_cross")
}

/// Every seed checked in beside the fuzz target, replayed.
#[test]
fn every_seed_in_the_checked_in_corpus_names_its_crossing_honestly() {
    let mut seeds = 0;
    let mut named = 0;
    let mut asked = Asked::default();
    let listing = std::fs::read_dir(corpus()).expect("the corpus is checked in beside the target");
    for entry in listing {
        let path = entry.expect("a readable directory entry").path();
        if !path.is_file() {
            continue;
        }
        let bytes = std::fs::read(&path).expect("a readable seed");
        seeds += 1;
        if let Some(pair) = two_surfaces_from(&bytes) {
            named += 1;
            asked = asked.and(every_oracle_holds_of(pair));
        }
    }
    println!("{seeds} seeds in the corpus, {named} of them a pair of surfaces, {asked:?}");
    assert!(seeds >= 100, "only {seeds} seeds are checked in");
    assert!(
        named * 2 > seeds,
        "only {named} of {seeds} seeds decoded to a pair, so the corpus has drifted away from \
         the thing it is a corpus of"
    );
    every_oracle_was_reached(asked, 1_400, 400, 800);
}

/// The soak, over the whole decoded space.
///
/// A cheap deterministic generator rather than a proptest strategy, because what
/// is being explored is the *bytes* and a strategy would put a shape on them
/// that the fuzzer's own mutations do not have.
#[test]
fn a_soak_over_pseudo_random_bytes_names_every_crossing_honestly() {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut drawn = 0;
    let mut named = 0;
    let mut asked = Asked::default();
    let mut bytes = vec![0u8; MOST_BYTES_A_PAIR_READS];

    for _ in 0..SOAK {
        for byte in &mut bytes {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *byte = state as u8;
        }
        drawn += 1;
        if let Some(pair) = two_surfaces_from(&bytes) {
            named += 1;
            asked = asked.and(every_oracle_holds_of(pair));
        }
    }

    println!("{drawn} byte strings drawn, {named} of them a pair of surfaces, {asked:?}");
    assert!(
        named * 4 > drawn,
        "only {named} of {drawn} draws named a pair, so the decoder is refusing most of the \
         space rather than exploring it"
    );
    every_oracle_was_reached(asked, 300_000, 130_000, 260_000);
}

/// **The cones and the tori, which no byte string reaches.**
///
/// The kinds are hard-wired rather than decoded, so this row grows when
/// `Surface` grows and not when the byte layout does.
///
/// Most of it refuses, and the refusals are the reason it is here: a cone
/// against a cone or a torus is a curve of degree four or eight that nothing
/// names yet, and a refusal hands the point oracle nothing to walk. The order
/// oracle is the one that still says something across a refusal, because it
/// compares a refusal against the other way round.
#[test]
fn every_pair_in_the_cone_and_torus_table_is_named_the_same_way_round_and_at_two_scales() {
    let mut asked = Asked::default();
    let mut pairs = 0;
    for pair in every_pair_the_table_makes_with_a_cone_or_a_torus_in_it() {
        pairs += 1;
        asked = asked.and(every_oracle_holds_of(pair));
    }
    println!("{pairs} pairs with a cone or a torus in them, {asked:?}");
    assert!(
        pairs >= 100,
        "only {pairs} pairs were built, so the family has shrunk"
    );
    every_oracle_was_reached(asked, 300, 30, 48);
}

/// Every pair the table below makes that has a cone or a torus on one side.
///
/// Whole numbers and powers of two throughout, so scaling by `2^±1` is exact and
/// the scale oracle is comparing the same geometry rather than a near copy of
/// it.
fn every_pair_the_table_makes_with_a_cone_or_a_torus_in_it() -> Vec<[Surface; 2]> {
    let mut out = Vec::new();
    for one in the_cones_and_the_tori() {
        for other in the_cones_and_the_tori()
            .into_iter()
            .chain(the_shapes_the_bytes_already_draw())
        {
            out.push([one, other]);
        }
    }
    out
}

fn the_cones_and_the_tori() -> Vec<Surface> {
    let z = Axis::new(Point::from([0.0, 0.0, 0.0]), Vector::from([0.0, 0.0, 1.0]))
        .expect("z is a direction");
    let raised = Axis::new(Point::from([0.0, 0.0, 4.0]), Vector::from([0.0, 0.0, 1.0]))
        .expect("z is a direction");
    let leaning = Axis::new(Point::from([0.0, 0.0, 0.0]), Vector::from([1.0, 1.0, 1.0]))
        .expect("a diagonal is a direction");
    let x = Axis::new(Point::from([0.0, 0.0, 0.0]), Vector::from([1.0, 0.0, 0.0]))
        .expect("x is a direction");
    vec![
        Surface::Cone(Cone::new(z, 1.0).expect("a positive slope")),
        Surface::Cone(Cone::new(z, 0.5).expect("a positive slope")),
        Surface::Cone(Cone::new(raised, 2.0).expect("a positive slope")),
        Surface::Cone(Cone::new(leaning, 1.0).expect("a positive slope")),
        Surface::Cone(Cone::new(x, 1.0).expect("a positive slope")),
        Surface::Torus(Torus::new(z, 4.0, 1.0).expect("a ring wider than its tube")),
        Surface::Torus(Torus::new(raised, 8.0, 2.0).expect("a ring wider than its tube")),
        Surface::Torus(Torus::new(x, 4.0, 2.0).expect("a ring wider than its tube")),
    ]
}

fn the_shapes_the_bytes_already_draw() -> Vec<Surface> {
    let z = Axis::new(Point::from([0.0, 0.0, 0.0]), Vector::from([0.0, 0.0, 1.0]))
        .expect("z is a direction");
    let x = Axis::new(Point::from([0.0, 0.0, 0.0]), Vector::from([1.0, 0.0, 0.0]))
        .expect("x is a direction");
    vec![
        Surface::Plane(Plane::from_coefficients([0.0, 0.0, 1.0, 0.0]).expect("a normal")),
        Surface::Plane(Plane::from_coefficients([0.0, 0.0, 1.0, -2.0]).expect("a normal")),
        Surface::Plane(Plane::from_coefficients([1.0, 0.0, 0.0, 0.0]).expect("a normal")),
        Surface::Plane(Plane::from_coefficients([1.0, 1.0, 1.0, 0.0]).expect("a normal")),
        Surface::Sphere(Sphere::new(Point::from([0.0, 0.0, 0.0]), 4.0).expect("a radius")),
        Surface::Sphere(Sphere::new(Point::from([0.0, 0.0, 4.0]), 2.0).expect("a radius")),
        Surface::Cylinder(Cylinder::new(z, 2.0).expect("a radius")),
        Surface::Cylinder(Cylinder::new(z, 4.0).expect("a radius")),
        Surface::Cylinder(Cylinder::new(x, 2.0).expect("a radius")),
    ]
}

/// Insist that each oracle was reached often enough to have been asserting
/// anything.
///
/// A floor apiece and not one shared floor, because the three skip for entirely
/// different reasons: a walk that refuses costs the point oracle nothing to
/// skip, the order oracle counts a pair that *either* order named, and the scale
/// oracle needs *both* scales decided and drops any pair already near the ends
/// of the range. A single count would let one of them fall to nothing behind
/// the other two.
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
