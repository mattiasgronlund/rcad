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
    let mut model = a_model();
    let pair = the_block_and_the_tipped_cube(&mut model, tilt);
    let answer = fuse(&mut model, pair, rule, order, policy)
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
        .sum()
}

fn refusal(tilt: [f64; 4], rule: Rule, order: Order, policy: &Policy, asked: &str) -> Error {
    let mut model = a_model();
    let pair = the_block_and_the_tipped_cube(&mut model, tilt);
    match fuse(&mut model, pair, rule, order, policy) {
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
) -> Result<Vec<SolidKey>, Error> {
    let (one, other) = match order {
        Order::BlockFirst => (block, tipped),
        Order::TippedFirst => (tipped, block),
    };
    model
        .fuse(&[one], &[other], rule, policy, &mut ProvenanceLog::new())
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

/// Temporary probe for queue row Q1528: one fuse, named by environment.
#[test]
#[ignore = "probe"]
fn probe_q1528_one() {
    let read = |name: &str| std::env::var(name).unwrap_or_else(|_| panic!("set {name}"));
    let k: u32 = read("PROBE_K").parse().expect("a tilt number");
    let further_in: f64 = std::env::var("PROBE_UP").map_or(0.0, |up| up.parse().expect("a lift"));
    let t = 0.0005 * f64::from(k);
    let tilt = [0.0, 0.0, t.sin(), t.cos()];
    let rule = match read("PROBE_RULE").as_str() {
        "union" => Rule::Union,
        "intersection" => Rule::Intersection,
        _ => Rule::Difference,
    };
    let order = match read("PROBE_ORDER").as_str() {
        "block" => Order::BlockFirst,
        _ => Order::TippedFirst,
    };
    let policy = match read("PROBE_POLICY").as_str() {
        "certified" => certified(),
        _ => tolerant(),
    };
    let mut model = a_model();
    let block = a_cube(&mut model);
    let tipped = a_cube(&mut model);
    let placement = Isometry::try_from(([5.0, 5.0, further_in], tilt)).expect("a placement");
    model.place_solid(tipped, &placement).expect("a placement");
    let mut log = ProvenanceLog::new();
    let (one, other) = match order {
        Order::BlockFirst => (block, tipped),
        Order::TippedFirst => (tipped, block),
    };
    let answer = model.fuse(&[one], &[other], rule, &policy, &mut log);
    match answer {
        Ok(answer) => {
            for &solid in answer.solids() {
                let view = model.solid(solid).expect("a solid");
                let volume = view
                    .measure(
                        Deflection::of(0.1).expect("a deflection"),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .map(|measured| measured.volume());
                eprintln!("solid: {} faces, volume {volume:?}", view.faces().count());
            }
        }
        Err(why) => eprintln!("refused: {why:?}"),
    }
    eprintln!("log: {log:?}");
    eprintln!("shared {}", what_the_two_share(tilt));
}

/// Temporary probe for queue row Q1528, taken out before the lane's last
/// commit.
#[test]
#[ignore = "probe"]
fn probe_q1528_sweep() {
    use std::collections::BTreeMap;
    let mut counts: BTreeMap<String, (usize, Vec<u32>)> = BTreeMap::new();
    for k in 1..=400_u32 {
        let t = 0.0005 * f64::from(k);
        let tilt = [0.0, 0.0, t.sin(), t.cos()];
        let shared = what_the_two_share(tilt);
        for (name, policy) in [("certified", certified()), ("tolerant", tolerant())] {
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                for order in [Order::BlockFirst, Order::TippedFirst] {
                    let worked = match rule {
                        Rule::Union => 2000.0 - shared,
                        Rule::Intersection => shared,
                        _ => 1000.0 - shared,
                    };
                    let mut model = a_model();
                    let pair = the_block_and_the_tipped_cube(&mut model, tilt);
                    let outcome = match fuse(&mut model, pair, rule, order, &policy) {
                        Ok(solids) => {
                            let valid = model.validate(&policy, &mut ProvenanceLog::new());
                            let each: Vec<(usize, f64)> = solids
                                .iter()
                                .map(|&solid| {
                                    let view = model.solid(solid).expect("a solid");
                                    let volume = view
                                        .measure(
                                            Deflection::of(0.1).expect("a deflection"),
                                            &Policy::certified(),
                                            &mut ProvenanceLog::new(),
                                        )
                                        .map_or(f64::NAN, |measured| measured.volume());
                                    (view.faces().count(), volume)
                                })
                                .collect();
                            let total: f64 = each.iter().map(|(_, volume)| volume).sum();
                            let right = (total - worked).abs() <= 1e-9 * worked;
                            if solids.len() > 1 || !right || valid.is_err() {
                                eprintln!(
                                    "k {k} {name} {rule:?} {order:?}: {} solids {each:?}, total \
                                     {total} worked {worked}, valid {:?}",
                                    solids.len(),
                                    valid.err()
                                );
                            }
                            format!(
                                "answers {} solid(s){}",
                                solids.len(),
                                if right { "" } else { " WRONG VOLUME" }
                            )
                        }
                        Err(why) => {
                            let said = format!("{why:?}");
                            let said = said
                                .strip_prefix("TheFuseWroteABrokenAnswer { broken: ")
                                .unwrap_or(&said)
                                .to_owned();
                            let said = said.strip_prefix("Invalid(").unwrap_or(&said).to_owned();
                            if said.starts_with("ResultIsNotManifold") {
                                said
                            } else {
                                said.split([' ', '(', '{']).next().unwrap_or("").to_owned()
                            }
                        }
                    };
                    let entry = counts
                        .entry(format!("{name} {rule:?} {order:?} {outcome}"))
                        .or_default();
                    entry.0 += 1;
                    entry.1.push(k);
                }
            }
        }
    }
    for (what, (count, ks)) in counts {
        eprintln!(
            "{what}: {count}, k {:?}",
            ks.iter().take(12).collect::<Vec<_>>()
        );
    }
}
