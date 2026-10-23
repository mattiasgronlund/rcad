//! `docs/decisions.md` §316.1's table, decoded again rather than taken on
//! trust (Q312). Each of the three corpus seeds splits into two halves, each
//! half a `Model` holding two cubes of side `s = 2.3611832414348226e21` on the
//! same six axis planes; one solid's stored corner or corners sit off where
//! its planes cross, by an amount no plane can see (§316.2).
//!
//! Measured here with
//!
//! ```text
//! just t -p rcad-topo --test merged \
//!     -E 'test(/^the_three_flat_seeds_and_the_corners_they_move::/)'
//! ```
//!
//! Every half that decodes is checked below, not only the half §316.1
//! happened to print: `2e9c4138…`'s first half refused both ways round and so
//! was never read before this, and `65452b22…`'s first half does not decode at
//! all. Both are pinned here too.
//!
//! **One correction to §316.1's opening claim.** "Both cubes sit on the same
//! six axis planes, bit for bit... the second cube differs only in one or two
//! stored corners" holds for five of the six halves, but not
//! `2e9c4138…`'s first half: there **solid 1** also carries two corners off
//! the canonical `{0, s}` cube, both different from either of solid 2's two
//! moved corners. `the_first_half_of_2e9c4138_moves_a_corner_of_both_solids`
//! pins that.

#![forbid(unsafe_code)]

use std::path::PathBuf;

use rcad_geom::Point;
use rcad_topo::{Model, Rule, SolidKey};

/// Where the checked-in corpus lives, the same join every other replay in
/// this crate uses.
fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fuzz/corpus/two_solids_fused_by_a_stranger")
}

/// One seed's header, and both halves decoded into a `Model` where they can
/// be. The split is the one `fuzzing::fuse::two_models_split_and_fused` makes
/// — a header byte, then the rest cut in half, the second half read under the
/// header rotated by one nibble — redone here because that door hands back
/// only a `Reached` count of what happened and this wants the `Model` itself,
/// to read its corners.
fn the_two_halves(name: &str) -> [(u8, Option<Model>); 2] {
    let bytes = std::fs::read(corpus().join(name)).expect("the seed is checked in");
    let (&header, rest) = bytes
        .split_first()
        .expect("a seed carries at least a header");
    let half = rest.len() / 2;
    let (first, second) = rest.split_at(half);
    [
        (header, postcard::from_bytes(first).ok()),
        (header.rotate_left(4), postcard::from_bytes(second).ok()),
    ]
}

/// `(header >> 4) % 3`, the rule a half's header draws.
fn rule_of(header: u8) -> Rule {
    match (header >> 4) % 3 {
        0 => Rule::Union,
        1 => Rule::Intersection,
        _ => Rule::Difference,
    }
}

/// `header & 0x80`, whether a half's header draws the tolerant policy.
fn is_tolerant(header: u8) -> bool {
    header & 0x80 != 0
}

/// Every stored corner of one solid, in the model's own frame, deduplicated
/// bit for bit. Walked from the solid down through its faces' wires' coedges
/// rather than read off the model's whole vertex arena, so a vertex no face of
/// this solid uses is never counted.
fn corners_of(model: &Model, solid: SolidKey) -> Vec<(Point, u64, u64, u64)> {
    let mut corners: Vec<(Point, u64, u64, u64)> = Vec::new();
    for face in model.solid(solid).expect("the model's own key").faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let point = coedge.start().point();
                let bits = (
                    point.x().to_bits(),
                    point.y().to_bits(),
                    point.z().to_bits(),
                );
                if !corners.iter().any(|&(_, x, y, z)| (x, y, z) == bits) {
                    corners.push((point, bits.0, bits.1, bits.2));
                }
            }
        }
    }
    corners
}

/// The two solids of a decoded half, in the fixed order `Model::solids`
/// walks them.
fn the_two_solids(model: &Model) -> (SolidKey, SolidKey) {
    let solids: Vec<_> = model.solids().map(|s| s.key()).collect();
    let [one, two] = solids[..] else {
        panic!("two solids, not {}: {solids:?}", solids.len());
    };
    (one, two)
}

/// Every stored corner of the second solid that is not among the first
/// solid's corners, bit for bit — §316.1's "stored corner(s) of solid 2 that
/// are off", read rather than quoted.
fn corners_of_solid_two_that_differ_from_solid_one(model: &Model) -> Vec<(Point, u64, u64, u64)> {
    let (one, two) = the_two_solids(model);
    let first = corners_of(model, one);
    corners_of(model, two)
        .into_iter()
        .filter(|&(_, x, y, z)| {
            !first
                .iter()
                .any(|&(_, fx, fy, fz)| (fx, fy, fz) == (x, y, z))
        })
        .collect()
}

/// `2e9c4138…`'s first half: never read before (§316.1 says so outright), and
/// the one half of the six where it is **solid 1**, not only solid 2, that
/// carries a corner off the canonical cube.
#[test]
fn the_first_half_of_2e9c4138_moves_a_corner_of_both_solids() {
    let [(header, model), _] = the_two_halves("2e9c4138d49472eaa947d5b75e38557823efa072");
    assert_eq!(header, 0x94, "the seed's own header byte");
    assert_eq!(rule_of(header), Rule::Union);
    assert!(is_tolerant(header), "0x94 & 0x80 is set");
    let model = model.expect("this half decodes");

    let (one, _two) = the_two_solids(&model);
    let s: f64 = 2.361_183_241_434_822_6e21;
    let canonical: Vec<(u64, u64, u64)> = [0.0, s]
        .into_iter()
        .flat_map(|x| [0.0, s].into_iter().map(move |y| (x, y)))
        .flat_map(|(x, y)| [0.0, s].into_iter().map(move |z| (x, y, z)))
        .map(|(x, y, z): (f64, f64, f64)| (x.to_bits(), y.to_bits(), z.to_bits()))
        .collect();

    let off_canonical = |solid: SolidKey| -> Vec<(Point, u64, u64, u64)> {
        corners_of(&model, solid)
            .into_iter()
            .filter(|&(_, x, y, z)| !canonical.contains(&(x, y, z)))
            .collect()
    };
    let solid_one_off = off_canonical(one);
    println!("solid 1's own off-canonical corners: {solid_one_off:?}");
    assert_eq!(
        solid_one_off,
        vec![
            (
                Point::new(3.2379e-319, 2.1123e-319, 0.0),
                0x10000,
                0xa701,
                0x0
            ),
            (
                Point::new(1.61895e-319, s, 0.0),
                0x8000,
                0x4460000000000000,
                0x0
            ),
        ],
        "solid 1 itself is not the clean cube §316.1's opening paragraph assumes every half holds"
    );

    let moved = corners_of_solid_two_that_differ_from_solid_one(&model);
    println!("solid 2's corners that differ from solid 1's: {moved:?}");
    assert_eq!(
        moved,
        vec![
            (
                Point::new(0.0, 1.060_997_895_5e-314, 3.2379e-319),
                0x0,
                0x80000000,
                0x10000
            ),
            // Solid 2's own exact (0, s, 0) is "off" here only because solid
            // 1's near-(0, s, 0) corner (above) has itself moved away from
            // it — not a third moved corner of solid 2 in its own right.
            (Point::new(0.0, s, 0.0), 0x0, 0x4460000000000000, 0x0),
            (
                Point::new(6.3e-322, s, s),
                0x80,
                0x4460000000000000,
                0x4460000000000000
            ),
        ]
    );
}

/// `2e9c4138…`'s second half: §316.1's own row, header `0x49`, a certified
/// intersection. Solid 1 is the clean cube here; solid 2 carries the two
/// moved corners the table lists.
#[test]
fn the_second_half_of_2e9c4138_moves_two_corners_of_its_second_solid() {
    let [_, (header, model)] = the_two_halves("2e9c4138d49472eaa947d5b75e38557823efa072");
    assert_eq!(header, 0x49);
    assert_eq!(rule_of(header), Rule::Intersection);
    assert!(!is_tolerant(header), "0x49 & 0x80 is clear: certified");
    let model = model.expect("this half decodes");

    let s = 2.361_183_241_434_822_6e21;
    let moved = corners_of_solid_two_that_differ_from_solid_one(&model);
    println!("solid 2's corners that differ from solid 1's: {moved:?}");
    assert_eq!(
        moved,
        vec![
            (
                Point::new(0.0, 1.358_077_306_22e-312, 0.0),
                0x0,
                0x4000000000,
                0x0
            ),
            (
                Point::new(s, 4.021_529_366_771_898e-87, 0.0),
                0x4460000000000000,
                0x2e00000000000000,
                0x0
            ),
        ]
    );
}

/// `65452b22…`'s first half does not decode at all — the `halves_that_decode:
/// 1` the replay in `every_model_the_fuse_fuzzer_kept_is_fused_again.rs`
/// already asserts for this seed, read directly here rather than taken on
/// trust.
#[test]
fn the_first_half_of_65452b22_does_not_decode() {
    let [(header, model), _] = the_two_halves("65452b22c98030899498f124502a4f41d1659162");
    assert_eq!(header, 0x94);
    assert!(model.is_none(), "postcard refuses this half's bytes");
}

/// `65452b22…`'s second half: header `0x49`, a certified intersection, one
/// moved corner on its second solid.
#[test]
fn the_second_half_of_65452b22_moves_one_corner_of_its_second_solid() {
    let [_, (header, model)] = the_two_halves("65452b22c98030899498f124502a4f41d1659162");
    assert_eq!(header, 0x49);
    assert_eq!(rule_of(header), Rule::Intersection);
    assert!(!is_tolerant(header));
    let model = model.expect("this half decodes");

    let moved = corners_of_solid_two_that_differ_from_solid_one(&model);
    println!("solid 2's corners that differ from solid 1's: {moved:?}");
    assert_eq!(
        moved,
        vec![(
            Point::new(0.0, 2.716_154_612_436e-312, 0.0),
            0x0,
            0x8000000000,
            0x0
        )]
    );
}

/// `1422656e…`'s **first** half: header `0x94`, a tolerant union, one moved
/// corner on its second solid. §316.1 already names this as the half that is
/// counted `a_solid_one_way_round_only`.
#[test]
fn the_first_half_of_1422656e_moves_one_corner_of_its_second_solid() {
    let [(header, model), _] = the_two_halves("1422656e67fcad8172a7e15528cf7ed5ace87f81");
    assert_eq!(header, 0x94);
    assert_eq!(rule_of(header), Rule::Union);
    assert!(is_tolerant(header));
    let model = model.expect("this half decodes");

    let moved = corners_of_solid_two_that_differ_from_solid_one(&model);
    println!("solid 2's corners that differ from solid 1's: {moved:?}");
    assert_eq!(
        moved,
        vec![(Point::new(0.0, 0.0, 3.2379e-319), 0x0, 0x0, 0x10000)]
    );
}

/// `1422656e…`'s **second** half: header `0x49`, a certified intersection.
/// Its one moved corner, `(s, 4.02e-87, 0)`, is bit-for-bit the same value as
/// `2e9c4138…`'s second half gives its *own* second moved corner — a genuine
/// coincidence between two corpus files, confirmed by decoding both, not a
/// copy-paste error in the record (Q312).
#[test]
fn the_second_half_of_1422656e_moves_one_corner_of_its_second_solid() {
    let [_, (header, model)] = the_two_halves("1422656e67fcad8172a7e15528cf7ed5ace87f81");
    assert_eq!(header, 0x49);
    assert_eq!(rule_of(header), Rule::Intersection);
    assert!(!is_tolerant(header));
    let model = model.expect("this half decodes");

    let s = 2.361_183_241_434_822_6e21;
    let moved = corners_of_solid_two_that_differ_from_solid_one(&model);
    println!("solid 2's corners that differ from solid 1's: {moved:?}");
    assert_eq!(
        moved,
        vec![(
            Point::new(s, 4.021_529_366_771_898e-87, 0.0),
            0x4460000000000000,
            0x2e00000000000000,
            0x0
        )]
    );
}
