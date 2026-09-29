//! A flat face whose outline is two arcs meeting at two corners, with a hole
//! that is half a disk whose straight side lies on the line through those two
//! corners. Every corner of both rings is on one line, so neither ring winds
//! round any area, and `rings_round` in `fuse/assemble.rs` has to ask
//! `holds_the_other_rings` which one is the outline (§191.5). Q576 built the
//! same question with rings of arcs only; this is the family with a straight
//! side, which it argued about and did not build (Q598).
//!
//! Every shape here is whole numbers. The outline comes from bars of radius 5
//! standing on `z = 0`, four tall, at `(−3, 0)` and `(3, 0)`, whose circles
//! cross at `(0, ±4)`. The hole comes from a half bar of radius 1: a round bar
//! less a block over `x > 0`, so it bulges towards `−x` and its flat side is on
//! `x = 0`, the line through `(0, ±4)`. It stands from `z = 1` to `z = 6`, so
//! it rises out of the cap at `z = 4`.
//!
//! **What answers.** A bar at `(−3, 0)` fused with the half bar at the origin
//! answers by every rule, both ways round. The fuse runs the line `x = 0` right
//! across the bar's cap, so the cap is cut at `(0, ±4)` and joined back up: its
//! outline is two arcs of the one pipe meeting there, and its hole is the half
//! disk from `(0, −1)` to `(0, 1)`. That is this family with both arcs on one
//! surface.
//!
//! Run under gdb on base `04a469ad`, with a breakpoint on the first line of
//! `holds_the_other_rings`, the three fuses that keep the cap each call it
//! twice: the ring along the one pipe answers that it holds the other, and the
//! half disk, through the count of turns round a corner, that it holds nothing.
//!
//! **What refuses.** With the two arcs on two pipes — a real lens, where the
//! bars at `(−3, 0)` and `(3, 0)` meet — the plane `x = 0` crosses each pipe
//! along the vertical line through a corner of the lens, which is where the two
//! walls meet. Every route tried is refused in the arrangement, before any face
//! is joined and so before `rings_round`: `TwoCurvesTouchWithoutCrossing` on a
//! wall bounded by those lines, which is §38.2's two curves along one ruling,
//! or `AFaceMeetsItsNeighbourOnItsOwnSurface` where a first fuse cut a wall
//! along them and did not join it back. A half bar whose corner is the lens's
//! corner, four surfaces round one vertical line, is refused the same way or
//! with `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`. Under the same breakpoint
//! the two tests of those routes stop twelve times and none: six times two,
//! the one-bar-and-half-bar union one route builds on the way.
//!
//! None of the forty-eight fuses here panics. Past those refusals the lens
//! would not reach the panic either, by `holds_the_other_rings` as it reads on
//! that base rather than by a run: a half disk inside the lens with its corners
//! strictly between the lens's two has both of the lens's corners outside its
//! circle, so it is placed as holding nothing and the lens is the one ring left
//! unplaced, which `rings_round` takes for the outline.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

const RULES: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// A way to build the two solids a fuse is asked about, what it is called, and
/// the refusal every fuse of them is expected to give.
type Route = (
    &'static str,
    fn(&mut Model) -> [SolidKey; 2],
    fn(&Error) -> bool,
);

/// A bar of radius 5 at `(−3, 0)` and the half bar rising out of its cap, by
/// every rule and both ways round.
///
/// - The union is the bar and the half bar's two units above the cap:
///   `100π + π = 101π`.
/// - The intersection is the half bar from `z = 1` to the cap: `1.5π`.
/// - The bar less the half bar has a pocket three deep: `98.5π`.
/// - The half bar less the bar is its two units above the cap: `π`.
///
/// Where the answer keeps the bar's cap, the cap is the face this file is
/// about: an outline with corners at `(0, ±4)` and a hole with corners at
/// `(0, ±1)`, four corners on one line.
#[test]
fn a_half_bar_rising_out_of_a_bars_cap_is_fused_by_every_rule_either_way_round() {
    for (rule, bar_first, volume, keeps_the_cap) in [
        (Rule::Union, true, 101.0 * PI, true),
        (Rule::Union, false, 101.0 * PI, true),
        (Rule::Intersection, true, 1.5 * PI, false),
        (Rule::Intersection, false, 1.5 * PI, false),
        (Rule::Difference, true, 98.5 * PI, true),
        (Rule::Difference, false, PI, false),
    ] {
        let asked = format!("{rule:?}, the bar first {bar_first}");
        let mut model = a_model();
        let bar = a_bar_at(&mut model, -3.0);
        let half = a_half_bar_bulging_left(&mut model, 1.0, 0.0);
        let [one, other] = if bar_first { [bar, half] } else { [half, bar] };
        let answer =
            fuse(&mut model, one, other, rule).unwrap_or_else(|why| panic!("{asked}: {why}"));
        let [solid] = answer[..] else {
            panic!("{asked} is one solid, not {answer:?}");
        };
        assert_holds(&model, solid, volume, &asked);
        if keeps_the_cap {
            assert_eq!(
                the_rings_of_the_cap(&model, solid),
                [[[0.0, -4.0], [0.0, 4.0]], [[0.0, -1.0], [0.0, 1.0]]],
                "{asked}: the corners of the cap's outline and of its hole"
            );
        }
    }
}

/// A lens of two pipes with the half bar standing on its chord, built three
/// ways, is refused by every rule and both ways round before any face is
/// joined.
///
/// - The bars at `(−3, 0)` and `(3, 0)` united first, then fused with the half
///   bar: the half bar's plane `x = 0` crosses each wall along the ruling where
///   the two walls meet.
/// - Where the two bars overlap, fused with the half bar: the same rulings are
///   the lens's two edges up its side.
/// - The bar at `(−3, 0)` united with the half bar first, then fused with the
///   bar at `(3, 0)`: the first union answers, but leaves the bar's wall in
///   four pieces cut along `x = 0` and `z = 1`, and the second fuse meets the
///   edge between two of them.
/// - The bars cut down to the D on each side of `x = 0`, the left one with
///   the half disk taken out of its flat side: the two flat sides meet along
///   the same rulings.
#[test]
fn a_lens_of_two_pipes_with_a_half_bar_on_its_chord_is_refused_by_every_rule() {
    let routes: [Route; 4] = [
        (
            "the two bars united, then the half bar",
            |model| {
                let bars = two_bars_united(model);
                [bars, a_half_bar_bulging_left(model, 1.0, 0.0)]
            },
            |why| matches!(why, Error::TwoCurvesTouchWithoutCrossing { .. }),
        ),
        (
            "where the two bars overlap, then the half bar",
            |model| {
                let lens = where_two_bars_overlap(model);
                [lens, a_half_bar_bulging_left(model, 1.0, 0.0)]
            },
            |why| matches!(why, Error::TwoCurvesTouchWithoutCrossing { .. }),
        ),
        (
            "one bar united with the half bar, then the other bar",
            |model| {
                let bar = a_bar_at(model, -3.0);
                let half = a_half_bar_bulging_left(model, 1.0, 0.0);
                let [both] =
                    fuse(model, bar, half, Rule::Union).expect("one bar and the half bar")[..]
                else {
                    panic!("one bar and the half bar are one solid");
                };
                [both, a_bar_at(model, 3.0)]
            },
            |why| matches!(why, Error::AFaceMeetsItsNeighbourOnItsOwnSurface { .. }),
        ),
        (
            "the D on each side, the left one bitten",
            |model| {
                let left = a_d_of_the_bar_at(model, -3.0);
                let bite = a_half_bar_bulging_left(model, 1.0, 0.0);
                let [bitten] = fuse(model, left, bite, Rule::Difference).expect("a bitten D")[..]
                else {
                    panic!("a bitten D is one solid");
                };
                [bitten, a_d_of_the_bar_at(model, 3.0)]
            },
            |why| matches!(why, Error::TwoCurvesTouchWithoutCrossing { .. }),
        ),
    ];
    for (route, build, refused_as_expected) in routes {
        assert_every_fuse_refuses(route, build, refused_as_expected);
    }
}

/// The half bar moved to `(0, −3)`, so its flat side runs from the lens's
/// corner `(0, −4)` to `(0, −2)`: four surfaces round the one vertical line
/// through `(0, −4)`. Every fuse that would join a cap round it is refused, by
/// every rule and both ways round.
#[test]
fn a_half_bar_whose_corner_is_the_lens_corner_is_refused_by_every_rule() {
    let routes: [Route; 3] = [
        (
            "one bar and the half bar",
            |model| {
                let bar = a_bar_at(model, -3.0);
                [bar, a_half_bar_bulging_left(model, 1.0, -3.0)]
            },
            |why| matches!(why, Error::TwoCurvesTouchWithoutCrossing { .. }),
        ),
        (
            "the two bars united, then the half bar",
            |model| {
                let bars = two_bars_united(model);
                [bars, a_half_bar_bulging_left(model, 1.0, -3.0)]
            },
            the_three_surfaces_need_two_flat,
        ),
        (
            "where the two bars overlap, then the half bar",
            |model| {
                let lens = where_two_bars_overlap(model);
                [lens, a_half_bar_bulging_left(model, 1.0, -3.0)]
            },
            the_three_surfaces_need_two_flat,
        ),
    ];
    for (route, build, refused_as_expected) in routes {
        assert_every_fuse_refuses(route, build, refused_as_expected);
    }
}

fn the_three_surfaces_need_two_flat(why: &Error) -> bool {
    matches!(
        why,
        Error::Geometry(rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
    )
}

fn assert_every_fuse_refuses(
    route: &str,
    build: fn(&mut Model) -> [SolidKey; 2],
    refused_as_expected: fn(&Error) -> bool,
) {
    for rule in RULES {
        for first_first in [true, false] {
            let asked = format!("{route}, {rule:?}, the first first {first_first}");
            let mut model = a_model();
            let [first, second] = build(&mut model);
            let [one, other] = if first_first {
                [first, second]
            } else {
                [second, first]
            };
            match fuse(&mut model, one, other, rule) {
                Err(why) if refused_as_expected(&why) => (),
                Err(why) => panic!("{asked} is refused another way: {why:?}"),
                Ok(answer) => panic!("{asked} answers {answer:?}; pin what it answers"),
            }
        }
    }
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// A bar of radius 5 at `(x, 0)`, from `z = 0` to `z = 4`.
fn a_bar_at(model: &mut Model, x: f64) -> SolidKey {
    model
        .add_round_bar(5.0, 4.0, &at(x, 0.0, 0.0))
        .expect("a bar of radius five")
}

/// A half bar of `radius` about `(0, y)`, from `z = 1` to `z = 6`, over
/// `x ≤ 0`.
fn a_half_bar_bulging_left(model: &mut Model, radius: f64, y: f64) -> SolidKey {
    let bar = model
        .add_round_bar(radius, 5.0, &at(0.0, y, 1.0))
        .expect("a bar");
    let block = a_block_over(model, [0.0, -20.0], [20.0, 20.0]);
    the_one_solid(model, bar, block, Rule::Difference)
}

/// The bars at `(−3, 0)` and `(3, 0)` united.
fn two_bars_united(model: &mut Model) -> SolidKey {
    let left = a_bar_at(model, -3.0);
    let right = a_bar_at(model, 3.0);
    the_one_solid(model, left, right, Rule::Union)
}

/// Where the bars at `(−3, 0)` and `(3, 0)` overlap.
fn where_two_bars_overlap(model: &mut Model) -> SolidKey {
    let left = a_bar_at(model, -3.0);
    let right = a_bar_at(model, 3.0);
    the_one_solid(model, left, right, Rule::Intersection)
}

/// The part of the bar at `(x, 0)` on its own side of `x = 0`.
fn a_d_of_the_bar_at(model: &mut Model, x: f64) -> SolidKey {
    let bar = a_bar_at(model, x);
    let block = if x < 0.0 {
        a_block_over(model, [0.0, -20.0], [20.0, 20.0])
    } else {
        a_block_over(model, [-20.0, -20.0], [0.0, 20.0])
    };
    the_one_solid(model, bar, block, Rule::Difference)
}

/// A block over the rectangle from `low` to `high`, from `z = −5` to
/// `z = 15`, well past everything else here.
fn a_block_over(model: &mut Model, low: [f64; 2], high: [f64; 2]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [low[0], low[1]],
        [high[0], low[1]],
        [high[0], high[1]],
        [low[0], high[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(&rectangle, 20.0, &at(0.0, 0.0, -5.0))
        .expect("a positive height")
}

fn the_one_solid(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let answer = fuse(model, one, other, rule).unwrap_or_else(|why| panic!("{rule:?}: {why}"));
    let [solid] = answer[..] else {
        panic!("{rule:?} is one solid, not {answer:?}");
    };
    solid
}

fn fuse(
    model: &mut Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
) -> Result<Vec<SolidKey>, Error> {
    model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().to_vec())
}

fn assert_holds(model: &Model, solid: SolidKey, volume: f64, what: &str) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} left a model that does not validate: {why}"));
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} does not measure: {why}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{what} holds {volume} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The corners of each ring of the face at `z = 4` facing up, each ring's
/// corners in order of `y` and the rings in order of their lowest corner.
fn the_rings_of_the_cap(model: &Model, solid: SolidKey) -> Vec<Vec<[f64; 2]>> {
    let solid = model.solid(solid).expect("this model made the key");
    let [cap] = solid
        .faces()
        .filter(|face| {
            face.surface()
                .as_plane()
                .is_some_and(|plane| plane.normal().components() == [0.0, 0.0, 1.0])
                && face
                    .wires()
                    .flat_map(|wire| wire.points())
                    .all(|point| point.coordinates()[2] == 4.0)
        })
        .collect::<Vec<_>>()[..]
    else {
        panic!("one face at z = 4 faces up");
    };
    let mut rings: Vec<Vec<[f64; 2]>> = cap
        .wires()
        .map(|wire| {
            let mut corners: Vec<[f64; 2]> = wire
                .points()
                .map(|point| {
                    let [x, y, _] = point.coordinates();
                    [x, y]
                })
                .collect();
            corners.sort_by(|one, other| one[1].total_cmp(&other[1]));
            corners
        })
        .collect();
    rings.sort_by(|one, other| one[0][1].total_cmp(&other[0][1]));
    rings
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift")
}
