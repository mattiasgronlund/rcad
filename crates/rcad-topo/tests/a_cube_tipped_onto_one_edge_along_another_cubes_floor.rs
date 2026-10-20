//! A cube tipped onto one of its edges, standing with that edge along another
//! cube's floor.
//!
//! The block is `add_cube(10.0)`, `[0, 10]³`. The tipped cube is a second
//! `add_cube(10.0)` placed by `Isometry::try_from(([5, 5, 0], [0, 0, s, c]))`
//! with `s = sin t` and `c = cos t`: a half turn about the axis through the
//! origin along `(0, s, c)`, which leans `t` from `z` towards `y`, then a move
//! by `(5, 5, 0)`. That stands it on its edge `y = 5`, `z = 0`,
//! `−5 ≤ x ≤ 5`, tipped by `2t` about `x`, so the edge lies along the block's
//! floor `z = 0` from `x = 0` to `x = 5` and crosses the floor's own edge at
//! `(0, 5, 0)`. Two tilts, `t = 0.038` and `t = 0.081`, as queue row Q1506
//! found them among 400 tilts from `0.0005` to `0.2`.
//!
//! **The union panicked** in `edges_after_joining` (`fuse/assemble.rs`, "a
//! piece of a face has a straight side with no length") under both policies,
//! both ways round, at both tilts. The floor cuts the tipped cube's bottom
//! face along a line one rounding off the cube's own edge, and the meet of the
//! floor, the bottom face and the cube's side `x = 5` was not folded into the
//! cube's corner beside it. The union kept a sliver triangle of the bottom
//! face, `(0, 5, 0)`, `(4.999999999999999, 5, 0)` and
//! `(5, 5.000000000000001, 0)`, each of whose three sides had the floor on its
//! other side. The floor, joined, walks the same three corners back, and the
//! fuse now leaves both out (`leave_out_each_flat_ring_another_plane_walks_back`
//! in `fuse/assemble.rs`).
//!
//! **The union was refused** `ResultIsNotManifold { count: 4 }` at 115 of
//! those 400 tilts, under both policies, both ways round (queue row Q1528;
//! five of them are [`TILTS_THE_UNION_WAS_REFUSED_AT`]). There the cube's
//! corner at `x = 5` itself rounds onto the floor, at
//! `(4.999999999999999, 5, 0)`, and the union kept a sliver of the cube's
//! bottom face and a sliver of its side `x = 5`, each the triangle of that
//! corner and the floor's `(0, 5, 0)` and `(5, 5, 0)`. The two slivers walk
//! one ring both ways, so the edge from `(0, 5, 0)` to `(5, 5, 0)` had four
//! faces along it, and joining the floor's pieces refused before Q1506's rule
//! could ask. The fuse now leaves such a pair out before it joins anything
//! (`leave_out_each_pair_of_flat_faces_walking_one_ring_both_ways` in
//! `fuse/assemble.rs`).
//!
//! **The block less the tipped cube was refused** `ResultIsNotManifold {
//! count: 4 }` at those five tilts, under a tolerant policy, until queue row
//! Q1901. There the block keeps a thin wedge under the cube's bottom face and
//! a wide one beside its side, and the two touch along the edge the cube
//! stands on: one solid touching itself along a line. That is an answer
//! (`docs/decisions.md` §39.4), and the fuse parts the line into one edge for
//! each wedge.
//!
//! **The tipped cube less the block was two solids**, under a tolerant
//! policy, at the four tilts of
//! [`TILTS_A_TETRAHEDRON_WAS_LEFT_BELOW_THE_FLOOR_AT`] (queue row Q1687).
//! There the cube's corner at `x = 5` is stored a rounding below the floor,
//! and the wedge between the cube's edge and the floor, under the block,
//! closed into a tetrahedron of `8.2e−32` to `3.3e−31` touching the rest at
//! `(0, 5, 0)`, or at `t = 0.142` into a shell flat on the floor that
//! measured `0.0` and refused the fuse. The same cube `2⁻³⁰` higher or lower
//! is one solid. A tolerant fuse now leaves out a shell whose corners all lie
//! within the policy's wobble of one of its own planes
//! (`leave_out_each_shell_thinner_than_the_wobble` in `fuse/assemble.rs`).
//!
//! **The union and the intersection were refused**, under a tolerant policy,
//! either way round, at the four tilts of
//! [`TILTS_THE_UNION_AND_THE_INTERSECTION_WERE_REFUSED_AT`], and the union at
//! `t = 0.142`, as a face whose outline winds against its normal (queue rows
//! Q1686 and Q1711). There the cube's bottom and its side meet in an edge
//! parallel to the floor and a rounding below it, so the floor cuts both
//! faces in lines a rounding either side of the edge, and the strip between
//! them winds no way. A tolerant fuse refused otherwise now takes the two
//! meets where the floor cuts those lines at each end of the edge as that
//! end, and the strip is one edge walked out and back
//! (`fold_each_end_of_an_edge_one_plane_runs_beside_a_rounding_from_it` in
//! `fuse/assemble.rs`, `docs/decisions.md` §417.9). The union took that same
//! fold until queue row Q2270: since then it takes the tipped cube's thin
//! shell below the floor as flat first, the same fold the difference above
//! uses, so the floor-fold's record now comes from the intersection alone.
//!
//! What the two cubes share is the cube's square cross-section, clipped to
//! the block, along the five of its ten that lie over the block:
//! [`what_the_two_share`] works it out from the tilt.

use rcad_geom::{Deflection, Isometry, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, Rule, SolidKey};

/// The two tilts, as the quaternion `[w, i, j, k]` that `Isometry::try_from`
/// reads: `t = 0.038` and `t = 0.081`, each written as Q1506 printed it.
const TILTS: [[f64; 4]; 2] = [
    [0.0, 0.0, 0.037_990_855_326_937_03, 0.999_278_086_876_484_9],
    [0.0, 0.0, 0.080_911_455_551_998_05, 0.996_721_293_221_157_7],
];

/// Five of the 115 tilts, of 400, at which the union was refused
/// `ResultIsNotManifold { count: 4 }` under both policies, both ways round
/// (queue row Q1528): `t = 0.002`, `0.003`, `0.008`, `0.01` and `0.015`, as
/// `f64::sin` and `f64::cos` give them.
const TILTS_THE_UNION_WAS_REFUSED_AT: [[f64; 4]; 5] = [
    [
        0.0,
        0.0,
        0.001_999_998_666_666_933_3,
        0.999_998_000_000_666_6,
    ],
    [0.0, 0.0, 0.002_999_995_500_002_025, 0.999_995_500_003_375],
    [0.0, 0.0, 0.007_999_914_666_939_733, 0.999_968_000_170_666_3],
    [0.0, 0.0, 0.009_999_833_334_166_664, 0.999_950_000_416_665_3],
    [0.0, 0.0, 0.014_999_437_506_328_09, 0.999_887_502_109_359_2],
];

/// Four tilts at which the tipped cube less the block, under a tolerant
/// policy, was not one solid (queue row Q1687): `t = 0.075`, `0.0885` and
/// `0.1315`, where it answered a second solid, a tetrahedron of `8.2e−32` to
/// `3.3e−31` below the floor, and `t = 0.142`, where it was refused
/// `VolumeIsNotPositive { volume: 0.0 }`. As `f64::sin` and `f64::cos` give
/// them.
const TILTS_A_TETRAHEDRON_WAS_LEFT_BELOW_THE_FLOOR_AT: [[f64; 4]; 4] = [
    [0.0, 0.0, 0.074_929_707_272_742_34, 0.997_188_818_112_207_5],
    [0.0, 0.0, 0.088_384_519_545_367_51, 0.996_086_430_338_62],
    [0.0, 0.0, 0.131_121_339_064_130_45, 0.991_366_327_066_856_5],
    [0.0, 0.0, 0.141_523_266_230_237_75, 0.989_934_929_738_073_8],
];

/// Four tilts at which the union and the intersection, under a tolerant
/// policy, either way round, were refused `TheFuseWroteABrokenAnswer {
/// broken: OuterWireWindsAgainstTheNormal }` (queue rows Q1686 and Q1711):
/// `t = 0.024`, `0.0955`, `0.1015` and `0.154`, as `f64::sin` and `f64::cos`
/// give them. The tipped cube's bottom and its side `y = 5` meet in an edge
/// parallel to the floor, `3.7e−18` below it at `t = 0.024`, so the floor
/// cuts the cube in a strip `7.8e−17` wide whose corners all round onto
/// `y = 5`, and the strip, a face of the intersection, winds `Zero`. Both
/// answer since Q1711, with the floor taken through both ends of the edge.
const TILTS_THE_UNION_AND_THE_INTERSECTION_WERE_REFUSED_AT: [[f64; 4]; 4] = [
    [0.0, 0.0, 0.023_997_696_066_354_29, 0.999_712_013_823_734_5],
    [0.0, 0.0, 0.095_354_902_203_049_7, 0.995_443_339_736_545_1],
    [0.0, 0.0, 0.101_325_810_022_482_75, 0.994_853_295_829_735_8],
    [0.0, 0.0, 0.153_392_010_734_994_53, 0.988_165_416_842_076_7],
];

#[test]
fn the_union_is_both_cubes_less_what_they_share_where_it_was_refused() {
    for tilt in TILTS_THE_UNION_WAS_REFUSED_AT {
        for policy in [certified(), tolerant()] {
            for order in [Order::BlockFirst, Order::TippedFirst] {
                let asked = format!("{tilt:?}, {order:?}, {policy:?}");
                let volume = volume_of_a_fuse(tilt, Rule::Union, order, &policy, &asked);
                assert_about(volume, 2000.0 - what_the_two_share(tilt), &asked);
            }
        }
    }
}

/// Refused `ResultIsNotManifold { count: 4 }` under a tolerant policy until
/// queue row Q1901: the block touches itself along the cube's edge there, and
/// a solid touching itself along a line is an answer (`docs/decisions.md`
/// §39.4). One solid under either policy since.
#[test]
fn the_block_less_the_tipped_cube_is_the_block_less_what_they_share_where_the_union_was_refused() {
    for tilt in TILTS_THE_UNION_WAS_REFUSED_AT {
        for policy in [certified(), tolerant()] {
            let asked = format!("{tilt:?}, {policy:?}");
            let volumes = volumes_of_a_fuse(
                tilt,
                Rule::Difference,
                Order::BlockFirst,
                &policy,
                &mut ProvenanceLog::new(),
                &asked,
            );
            assert_eq!(volumes.len(), 1, "{asked}: {volumes:?}");
            assert_about(volumes[0], 1000.0 - what_the_two_share(tilt), &asked);
        }
    }
}

#[test]
fn the_union_is_both_cubes_less_what_they_share_tolerant() {
    for tilt in TILTS {
        for order in [Order::BlockFirst, Order::TippedFirst] {
            let asked = format!("{tilt:?}, {order:?}");
            let volume = volume_of_a_fuse(tilt, Rule::Union, order, &tolerant(), &asked);
            assert_about(volume, 2000.0 - what_the_two_share(tilt), &asked);
        }
    }
}

#[test]
fn the_union_is_both_cubes_less_what_they_share_certified() {
    for tilt in TILTS {
        for order in [Order::BlockFirst, Order::TippedFirst] {
            let asked = format!("{tilt:?}, {order:?}");
            let volume = volume_of_a_fuse(tilt, Rule::Union, order, &certified(), &asked);
            assert_about(volume, 2000.0 - what_the_two_share(tilt), &asked);
        }
    }
}

#[test]
fn the_intersection_is_what_they_share_tolerant() {
    for tilt in TILTS {
        for order in [Order::BlockFirst, Order::TippedFirst] {
            let asked = format!("{tilt:?}, {order:?}");
            let volume = volume_of_a_fuse(tilt, Rule::Intersection, order, &tolerant(), &asked);
            assert_about(volume, what_the_two_share(tilt), &asked);
        }
    }
}

/// Not Q1506's: refused before and after it, for corners a rounding apart
/// beside corners ten long.
#[test]
fn the_intersection_is_refused_for_the_range_certified() {
    for tilt in TILTS {
        for order in [Order::BlockFirst, Order::TippedFirst] {
            let asked = format!("{tilt:?}, {order:?}");
            let why = refusal(tilt, Rule::Intersection, order, &certified(), &asked);
            assert!(
                matches!(why, Error::Invalid(Invalid::DynamicRangeTooWide { .. })),
                "{asked}: {why:?}"
            );
        }
    }
}

#[test]
fn the_block_less_the_tipped_cube_is_the_block_less_what_they_share_tolerant() {
    for tilt in TILTS {
        let asked = format!("{tilt:?}");
        let volume = volume_of_a_fuse(
            tilt,
            Rule::Difference,
            Order::BlockFirst,
            &tolerant(),
            &asked,
        );
        assert_about(volume, 1000.0 - what_the_two_share(tilt), &asked);
    }
}

/// Not Q1506's, as the certified intersection.
#[test]
fn the_block_less_the_tipped_cube_is_refused_for_the_range_certified() {
    for tilt in TILTS {
        let asked = format!("{tilt:?}");
        let why = refusal(
            tilt,
            Rule::Difference,
            Order::BlockFirst,
            &certified(),
            &asked,
        );
        assert!(
            matches!(why, Error::Invalid(Invalid::DynamicRangeTooWide { .. })),
            "{asked}: {why:?}"
        );
    }
}

/// Refused `VolumeIsNotPositive` with a volume of `0.0` under both policies,
/// both tilts, until the sliver the union kept was left out of every answer.
#[test]
fn the_tipped_cube_less_the_block_is_the_cube_less_what_they_share() {
    for tilt in TILTS {
        for policy in [certified(), tolerant()] {
            let asked = format!("{tilt:?}, {policy:?}");
            let volume =
                volume_of_a_fuse(tilt, Rule::Difference, Order::TippedFirst, &policy, &asked);
            assert_about(volume, 1000.0 - what_the_two_share(tilt), &asked);
        }
    }
}

#[test]
fn the_tipped_cube_less_the_block_is_one_solid_where_its_corner_rounds_below_the_floor_tolerant() {
    for tilt in TILTS_A_TETRAHEDRON_WAS_LEFT_BELOW_THE_FLOOR_AT {
        let asked = format!("{tilt:?}");
        let volumes = volumes_of_a_fuse(
            tilt,
            Rule::Difference,
            Order::TippedFirst,
            &tolerant(),
            &mut ProvenanceLog::new(),
            &asked,
        );
        assert_eq!(volumes.len(), 1, "{asked}: {volumes:?}");
        assert_about(volumes[0], 1000.0 - what_the_two_share(tilt), &asked);
    }
}

#[test]
fn the_union_is_both_cubes_less_what_they_share_where_the_cubes_edge_rounds_below_the_floor_tolerant()
 {
    for tilt in TILTS_THE_UNION_AND_THE_INTERSECTION_WERE_REFUSED_AT {
        for order in [Order::BlockFirst, Order::TippedFirst] {
            let asked = format!("{tilt:?}, {order:?}");
            let volume = volume_of_a_fuse(tilt, Rule::Union, order, &tolerant(), &asked);
            assert_about(volume, 2000.0 - what_the_two_share(tilt), &asked);
        }
    }
}

#[test]
fn the_intersection_is_what_they_share_where_the_cubes_edge_rounds_below_the_floor_tolerant() {
    for tilt in TILTS_THE_UNION_AND_THE_INTERSECTION_WERE_REFUSED_AT {
        for order in [Order::BlockFirst, Order::TippedFirst] {
            let asked = format!("{tilt:?}, {order:?}");
            let volume = volume_of_a_fuse(tilt, Rule::Intersection, order, &tolerant(), &asked);
            assert_about(volume, what_the_two_share(tilt), &asked);
        }
    }
}

/// Answered one solid since queue row Q1687, `741.0000015943825` at
/// `t = 0.024` against `741.0000015943831` worked by hand.
#[test]
fn the_tipped_cube_less_the_block_is_one_solid_where_the_cubes_edge_rounds_below_the_floor_tolerant()
 {
    for tilt in TILTS_THE_UNION_AND_THE_INTERSECTION_WERE_REFUSED_AT {
        let asked = format!("{tilt:?}");
        let volumes = volumes_of_a_fuse(
            tilt,
            Rule::Difference,
            Order::TippedFirst,
            &tolerant(),
            &mut ProvenanceLog::new(),
            &asked,
        );
        assert_eq!(volumes.len(), 1, "{asked}: {volumes:?}");
        let worked = 1000.0 - what_the_two_share(tilt);
        assert!(
            (volumes[0] - worked).abs() <= 1e-12,
            "{asked}: measured {}, worked by hand {worked}",
            volumes[0]
        );
    }
}

#[test]
fn the_block_less_the_tipped_cube_is_one_solid_where_the_cubes_edge_rounds_below_the_floor_tolerant()
 {
    for tilt in TILTS_THE_UNION_AND_THE_INTERSECTION_WERE_REFUSED_AT {
        let asked = format!("{tilt:?}");
        let volumes = volumes_of_a_fuse(
            tilt,
            Rule::Difference,
            Order::BlockFirst,
            &tolerant(),
            &mut ProvenanceLog::new(),
            &asked,
        );
        assert_eq!(volumes.len(), 1, "{asked}: {volumes:?}");
        assert_about(volumes[0], 1000.0 - what_the_two_share(tilt), &asked);
    }
}

/// Every tilt of this file, every rule, either way round, under a tolerant
/// policy: one solid, within §417.8's allowance of the volume worked by hand.
///
/// **The block less the tipped cube was refused `ResultIsNotManifold { count:
/// 4 }` at the five [`TILTS_THE_UNION_WAS_REFUSED_AT`]** until queue row
/// Q1901, with the fold tried and refused too. There the block touches itself
/// along the cube's edge, and a solid touching itself along a line is an
/// answer (`docs/decisions.md` §39.4).
///
/// **The floor is taken as running through the ends of the cube's edge only
/// where the intersection was refused without it** (queue row Q1711,
/// `docs/decisions.md` §417.9): at
/// [`TILTS_THE_UNION_AND_THE_INTERSECTION_WERE_REFUSED_AT`]. It logs two
/// records, one at each end of the edge the cube stands on, `(−5, 5, 0)` and
/// `(5, 5, 0)`: the floor's plane cuts the cube's bottom and side the edge's
/// whole length, past the block at `x = −5` too. Every other intersection
/// logs none.
///
/// **The union never needs that fold** (queue row Q2270): it takes the
/// tipped cube's thin shell below the floor as flat before anything is kept,
/// the same fold
/// [`the_tipped_cube_less_the_block_is_one_solid_where_its_corner_rounds_below_the_floor_tolerant`]
/// pins for the difference, and logs one "a shell within the wobble of one of
/// its planes taken as flat" record at the eight tilts of
/// [`TILTS_A_TETRAHEDRON_WAS_LEFT_BELOW_THE_FLOOR_AT`] and
/// [`TILTS_THE_UNION_AND_THE_INTERSECTION_WERE_REFUSED_AT`], both orders, and
/// none at the rest.
///
/// The allowance is the one `a_turned_cube_with_its_corner_on_a_face.rs`
/// works out for its own pair of cubes: four roundings at 16 for each corner,
/// over faces of no more than 1200.
#[test]
fn every_tolerant_fuse_is_one_solid_and_takes_the_floor_through_the_edges_ends_only_where_refused_without()
 {
    let allowed = 1200.0 * 3.0_f64.sqrt() * 4.0 * 16.0 * f64::EPSILON;
    for tilt in every_tilt() {
        let share = what_the_two_share(tilt);
        let refused_without = TILTS_THE_UNION_AND_THE_INTERSECTION_WERE_REFUSED_AT.contains(&tilt);
        let a_shell_was_taken_as_flat =
            refused_without || TILTS_A_TETRAHEDRON_WAS_LEFT_BELOW_THE_FLOOR_AT.contains(&tilt);
        for order in [Order::BlockFirst, Order::TippedFirst] {
            for (rule, worked, folded) in [
                (Rule::Union, 2000.0 - share, false),
                (Rule::Intersection, share, refused_without),
                (Rule::Difference, 1000.0 - share, false),
            ] {
                let asked = format!("{tilt:?}, {rule:?}, {order:?}");
                let mut log = ProvenanceLog::new();
                let volumes = volumes_of_a_fuse(tilt, rule, order, &tolerant(), &mut log, &asked);
                assert_eq!(volumes.len(), 1, "{asked}: {volumes:?}");
                assert!(
                    (volumes[0] - worked).abs() <= allowed,
                    "{asked}: measured {}, worked by hand {worked}",
                    volumes[0]
                );
                let ends = the_ends_the_floor_was_taken_through(&log);
                let wanted: &[[f64; 3]] = if folded {
                    &[[-5.0, 5.0, 0.0], [5.0, 5.0, 0.0]]
                } else {
                    &[]
                };
                assert_eq!(ends.len(), wanted.len(), "{asked}: {ends:?}");
                for (end, corner) in ends.iter().zip(wanted) {
                    let off = (0..3)
                        .map(|axis| (end[axis] - corner[axis]).abs())
                        .fold(0.0, f64::max);
                    assert!(
                        off <= 1e-12,
                        "{asked}: taken through {end:?}, not {corner:?}"
                    );
                }
                if rule == Rule::Union {
                    let flat = how_many_shells_were_taken_as_flat(&log);
                    let wanted_flat = usize::from(a_shell_was_taken_as_flat);
                    assert_eq!(flat, wanted_flat, "{asked}: {flat} shells taken as flat");
                }
            }
        }
    }
}

/// [`what_the_two_share`], which every pin here measures the kernel against,
/// worked again by hand in `T = tan t`, so those pins do not rest on its five
/// corners alone.
///
/// With `φ = 2t`, `sin φ = 2T / (1 + T²)` and `cos φ = (1 − T²) / (1 + T²)`,
/// the shoelace sum over its corners comes to `25 tan φ − 100 − 200T`, and
/// `tan φ = 2T / (1 − T²)`. So the two share
/// `5 · (100 + 200T − 25 tan φ) / 2 = 250 + 500T − 125T / (1 − T²)`. At
/// `t = 0` that is the block's half of the upright cube, `5 · 10 · 5 = 250`.
/// In `t` it is `250 + 375t − 200t⁵` and terms smaller still, so at
/// `t = 0.01` the two share `253.75` less about `2e−8`, and the block less the
/// tipped cube is `746.25` and about as much more.
#[test]
fn what_the_two_share_is_250_and_500_tan_t_less_125_tan_t_over_one_less_its_square() {
    for tilt in TILTS_THE_UNION_WAS_REFUSED_AT {
        let [_, _, s, c] = tilt;
        let tan = s / c;
        let worked = 250.0 + 500.0 * tan - 125.0 * tan / (1.0 - tan * tan);
        let shared = what_the_two_share(tilt);
        assert!(
            (shared - worked).abs() <= 1e-12 * worked,
            "{tilt:?}: the five corners give {shared}, the tangent {worked}"
        );
    }
    let at_a_hundredth = what_the_two_share(TILTS_THE_UNION_WAS_REFUSED_AT[3]);
    assert!(
        (at_a_hundredth - 253.75).abs() <= 1e-7,
        "at t = 0.01 the two share {at_a_hundredth}"
    );
}

/// The volume the block and the tipped cube share.
///
/// The tipped cube's square cross-section, in `(y, z)`, has its corner at
/// `(5, 0)` and its sides leaning by `φ = 2t` from the axes: its bottom runs
/// towards `−y` and rises, `(−cos φ, sin φ)`, and its side runs up and
/// towards `+y`, `(sin φ, cos φ)`. Clipped to the block's `[0, 10]²` it is
/// the five-cornered shape from `(5, 0)` up its bottom to `y = 0` at
/// `z = 5 tan φ`, up `y = 0` to the block's top, along the top to where the
/// cube's top face crosses it, and down to the cube's far corner of that side,
/// `(5 + 10 sin φ, 10 cos φ)`. Its area by the shoelace formula, times the
/// length `5` of the cube that lies over the block, `0 ≤ x ≤ 5`.
fn what_the_two_share(tilt: [f64; 4]) -> f64 {
    let [_, _, s, c] = tilt;
    let (sin, cos) = (2.0 * s * c, c * c - s * s);
    let along_the_top = 10.0 * (1.0 - cos) / sin;
    let corners = [
        [5.0, 0.0],
        [0.0, 5.0 * sin / cos],
        [0.0, 10.0],
        [5.0 - along_the_top * cos + 10.0 * sin, 10.0],
        [5.0 + 10.0 * sin, 10.0 * cos],
    ];
    let twice_the_area: f64 = (0..corners.len())
        .map(|at| {
            let ([y0, z0], [y1, z1]) = (corners[at], corners[(at + 1) % corners.len()]);
            y0 * z1 - y1 * z0
        })
        .sum();
    5.0 * twice_the_area.abs() / 2.0
}

fn assert_about(got: f64, worked: f64, asked: &str) {
    assert!(
        (got - worked).abs() <= 1e-9 * worked,
        "{asked}: measured {got}, worked by hand {worked}"
    );
}

#[derive(Clone, Copy, Debug)]
enum Order {
    BlockFirst,
    TippedFirst,
}

fn volume_of_a_fuse(tilt: [f64; 4], rule: Rule, order: Order, policy: &Policy, asked: &str) -> f64 {
    volumes_of_a_fuse(tilt, rule, order, policy, &mut ProvenanceLog::new(), asked)
        .iter()
        .sum()
}

/// The volume of each solid of the answer, once the model it is written into
/// validates. The fuse's records go into `log`.
fn volumes_of_a_fuse(
    tilt: [f64; 4],
    rule: Rule,
    order: Order,
    policy: &Policy,
    log: &mut ProvenanceLog,
    asked: &str,
) -> Vec<f64> {
    let mut model = a_model();
    let pair = the_block_and_the_tipped_cube(&mut model, tilt);
    let answer = fuse(&mut model, pair, rule, order, policy, log)
        .unwrap_or_else(|why| panic!("{asked}, {rule:?}: {why}"));
    model
        .validate(policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{asked}, {rule:?}: {why}"));
    answer
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.1).expect("a deflection"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{asked}, {rule:?} does not measure: {why}"))
                .volume()
        })
        .collect()
}

/// Every tilt of this file, each once.
fn every_tilt() -> impl Iterator<Item = [f64; 4]> {
    TILTS
        .into_iter()
        .chain(TILTS_THE_UNION_WAS_REFUSED_AT)
        .chain(TILTS_A_TETRAHEDRON_WAS_LEFT_BELOW_THE_FLOOR_AT)
        .chain(TILTS_THE_UNION_AND_THE_INTERSECTION_WERE_REFUSED_AT)
}

/// The middle of each end of an edge `log` says the fuse took as one point
/// with the two meets of a plane running beside the edge, along `x`.
fn the_ends_the_floor_was_taken_through(log: &ProvenanceLog) -> Vec<[f64; 3]> {
    let mut ends: Vec<[f64; 3]> = log
        .records()
        .iter()
        .filter(|record| {
            record.predicate() == "two meets of one plane beside an edge taken as its end"
        })
        .map(|record| {
            record
                .inputs()
                .first()
                .expect("the record names the end first")
                .enclosure()
                .map(|side| f64::midpoint(side.lo(), side.hi()))
        })
        .collect();
    ends.sort_by(|one, other| one[0].total_cmp(&other[0]));
    ends
}

/// How many shells `log` took as flat before anything was kept
/// (`takes_as_flat` in `fuse/assemble.rs`).
fn how_many_shells_were_taken_as_flat(log: &ProvenanceLog) -> usize {
    log.records()
        .iter()
        .filter(|record| {
            record.predicate() == "a shell within the wobble of one of its planes taken as flat"
        })
        .count()
}

fn refusal(tilt: [f64; 4], rule: Rule, order: Order, policy: &Policy, asked: &str) -> Error {
    let mut model = a_model();
    let pair = the_block_and_the_tipped_cube(&mut model, tilt);
    match fuse(
        &mut model,
        pair,
        rule,
        order,
        policy,
        &mut ProvenanceLog::new(),
    ) {
        Ok(solids) => panic!("{asked}, {rule:?} answers {} solids", solids.len()),
        Err(why) => why,
    }
}

fn fuse(
    model: &mut Model,
    [block, tipped]: [SolidKey; 2],
    rule: Rule,
    order: Order,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Vec<SolidKey>, Error> {
    let (one, other) = match order {
        Order::BlockFirst => (block, tipped),
        Order::TippedFirst => (tipped, block),
    };
    model
        .fuse(&[one], &[other], rule, policy, log)
        .map(|answer| answer.solids().to_vec())
}

fn certified() -> Policy {
    Policy::certified()
}

fn tolerant() -> Policy {
    Policy::tolerant(100.0)
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a scale"))
}

fn the_block_and_the_tipped_cube(model: &mut Model, tilt: [f64; 4]) -> [SolidKey; 2] {
    let block = a_cube(model);
    let tipped = a_cube(model);
    let placement = Isometry::try_from(([5.0, 5.0, 0.0], tilt)).expect("a placement");
    model.place_solid(tipped, &placement).expect("a placement");
    [block, tipped]
}

fn a_cube(model: &mut Model) -> SolidKey {
    model
        .add_cube(10.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube")
}
