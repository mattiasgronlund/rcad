//! A block with a bore through it, cut across by a second fuse.
//!
//! The first fuse is a square block, 20 by 20, 10 tall, less a round bar bored
//! right through it: the bar stands from `z = −1` to `11`, a unit past both of
//! the block's own caps, so the bore's own ends never meet the block's. What is
//! left is a ring, the square less a circle of radius 5, ten tall, and the
//! bore's wall is **used reversed**: the material is outside the cylinder, the
//! same turn as a tube's inner wall or a crescent's inner wall in
//! `fusing_two_bars_side_by_side.rs`.
//!
//! The second fuse is a block 6 wide in `x`, the full 20 in `y`, standing from
//! `z = 5` to `15`: a crossbar through the top half of the bore. Its two side
//! walls, at `x = ±3`, cut the bore's circle at `(±3, ±4, ·)` — a 3-4-5
//! triangle, so every number below is exact — leaving the bore's wall a band
//! with a seam on each side, not a face that goes right round with none. Its
//! floor also cuts the wall at `z = 5`. That is the same kind of cut as
//! `a_block_over_the_top_half_of_a_crescent_cuts_it_as_cutting_each_bar_first_does`
//! in `fusing_two_bars_side_by_side.rs` met on a round outer wall: `bake.rs`
//! used to hand the arrangement that wall walked the way it is stored, and the
//! arrangement took the material on the wrong side of it (`docs/decisions.md`
//! §331.6). Nothing before this row crossed a bore this way: the tube in
//! `a_tube_less_a_bar_united_with_a_second_bar.rs` fuses its hole's wall with a
//! bar standing beside it, not a block cutting across the wall's own height,
//! and a block whose crossbar swallows the whole circle (no side wall meets
//! the bore) does not meet it either — tried first, on a worktree of
//! `2c349f2b`, with a probe since discarded, and it answered there already.
//!
//! **At `2c349f2b`, before the fix, three of the six below refused**: the
//! union both ways round and the difference with the ring first, each
//! `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 1, used: 1 }`. The other
//! three — both orders of the intersection and the difference with the block
//! first — already answered there, at the same volumes this fuses to now.
//! **At this row's base, an ancestor of the fix (`035a4a56`), all six answer
//! and validate**, each held to its volume by hand.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Profile, ProfileBuilder, Rule, SolidKey};

/// Half the ring's square footprint.
const HALF: f64 = 10.0;
/// How tall the block, and the crossbar, each stand.
const HEIGHT: f64 = 10.0;
/// The bore's radius: well inside the block's half-width, 10.
const BORE_RADIUS: f64 = 5.0;
/// How far the bore's own ends stand past the block's caps.
const BORE_PAST: f64 = 1.0;
/// Half the crossbar's width in `x`.
const CROSSBAR_HALF: f64 = 3.0;

/// **A block with a bore right through it, fused across the bore by every
/// rule and both ways round, each answer checked against its volume by hand
/// and validated.**
///
/// The ring is the block less the bore, `20·20·10 − π·5²·10`. The crossbar is
/// `6·20·10`. They share the part of the crossbar's own footprint, in the
/// ring's top half, that is not the bore's circle: the crossbar's rectangle,
/// `6·20`, less the band of the circle between `x = −3` and `x = 3` — the
/// circle less the two caps beyond `x = ±3`, each a circular segment,
/// `5²·acos(3/5) − 3·4` — all of it five units tall, the height the ring and
/// the crossbar share.
///
/// The intersection is two solids, the band split in two by the hole; every
/// other rule gives one.
#[test]
fn a_block_bored_through_fuses_across_the_bore_every_rule_both_ways_round() {
    let square = (2.0 * HALF) * (2.0 * HALF);
    let block = square * HEIGHT;
    let bore = core::f64::consts::PI * BORE_RADIUS * BORE_RADIUS * HEIGHT;
    let ring = block - bore;
    let crossbar = (2.0 * CROSSBAR_HALF) * (2.0 * HALF) * HEIGHT;
    let chord_half = (BORE_RADIUS * BORE_RADIUS - CROSSBAR_HALF * CROSSBAR_HALF).sqrt();
    let cap = BORE_RADIUS * BORE_RADIUS * (CROSSBAR_HALF / BORE_RADIUS).acos()
        - CROSSBAR_HALF * chord_half;
    let band = core::f64::consts::PI * BORE_RADIUS * BORE_RADIUS - 2.0 * cap;
    let crossbar_footprint = (2.0 * CROSSBAR_HALF) * (2.0 * HALF);
    let shared = (crossbar_footprint - band) * (HEIGHT / 2.0);

    fuse_and_check(
        true,
        Rule::Union,
        ring + crossbar - shared,
        1,
        "Union, ring first",
    );
    fuse_and_check(
        false,
        Rule::Union,
        ring + crossbar - shared,
        1,
        "Union, crossbar first",
    );
    fuse_and_check(
        true,
        Rule::Intersection,
        shared,
        2,
        "Intersection, ring first",
    );
    fuse_and_check(
        false,
        Rule::Intersection,
        shared,
        2,
        "Intersection, crossbar first",
    );
    fuse_and_check(
        true,
        Rule::Difference,
        ring - shared,
        1,
        "Difference, ring first",
    );
    fuse_and_check(
        false,
        Rule::Difference,
        crossbar - shared,
        1,
        "Difference, crossbar first",
    );
}

/// Builds the ring and the crossbar fresh, fuses them by `rule` with the ring
/// first when `ring_first`, and checks the `how_many` solids that come back,
/// summed, against `by_hand`.
fn fuse_and_check(ring_first: bool, rule: Rule, by_hand: f64, how_many: usize, what: &str) {
    let policy = Policy::certified();
    let mut model = a_model();
    let ring = the_ring(&mut model);
    let crossbar = the_crossbar(&mut model);
    let [first, second] = if ring_first {
        [ring, crossbar]
    } else {
        [crossbar, ring]
    };
    let fused = model
        .fuse(
            &[first],
            &[second],
            rule,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: {why:?}"));
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} left a model that does not validate: {why}"));
    let solids = fused.solids();
    assert_eq!(solids.len(), how_many, "{what}: how many solids");
    let (mut measured, mut error) = (0.0, 0.0);
    for &solid in solids {
        let measure = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a fine mesh"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} does not measure: {why}"));
        measured += measure.volume();
        error += measure.volume_error();
    }
    assert!(
        (measured - by_hand).abs() <= error,
        "{what} holds {by_hand} and this one says {measured} give or take {error}"
    );
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// The square block, `20 × 20`, standing from `z = 0` to `HEIGHT`, less a bore
/// of radius `BORE_RADIUS` standing from `z = −BORE_PAST` to `HEIGHT +
/// BORE_PAST`, a unit past both of the block's own caps.
fn the_ring(model: &mut Model) -> SolidKey {
    let block = model
        .add_extrusion(&the_square(), HEIGHT, &at(0.0, 0.0, 0.0))
        .expect("the square extrudes");
    let bore = model
        .add_round_bar(
            BORE_RADIUS,
            HEIGHT + 2.0 * BORE_PAST,
            &at(0.0, 0.0, -BORE_PAST),
        )
        .expect("the bore");
    let fused = model
        .fuse(
            &[block],
            &[bore],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a block less a bore right through it");
    let [ring] = fused.solids() else {
        panic!("the block less the bore is one solid: {:?}", fused.solids());
    };
    *ring
}

/// `2·CROSSBAR_HALF` wide in `x`, the ring's own `20` in `y`, standing from
/// `z = HEIGHT / 2` up to `1.5 · HEIGHT`: across the top half of the bore,
/// its side walls cutting the bore's circle rather than swallowing it whole.
fn the_crossbar(model: &mut Model) -> SolidKey {
    let narrow = ProfileBuilder::new(vec![
        [-CROSSBAR_HALF, -HALF],
        [CROSSBAR_HALF, -HALF],
        [CROSSBAR_HALF, HALF],
        [-CROSSBAR_HALF, HALF],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a narrow rectangle");
    model
        .add_extrusion(&narrow, HEIGHT, &at(0.0, 0.0, HEIGHT / 2.0))
        .expect("the crossbar extrudes")
}

fn the_square() -> Profile {
    ProfileBuilder::new(vec![
        [-HALF, -HALF],
        [HALF, -HALF],
        [HALF, HALF],
        [-HALF, HALF],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square")
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}
