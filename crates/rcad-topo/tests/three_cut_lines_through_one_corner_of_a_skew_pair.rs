//! Three pairs from `how_often_a_certified_fuse_of_a_skew_pair_refuses`
//! whose fuses stopped on a flat piece with one side a rounding long (queue
//! rows Q1069 and Q1145, `docs/decisions.md` §365.7).
//!
//! Three cut lines meet at one point on such a piece, by the way the pair is
//! put together: two walls of one solid and a wall of the other, one of them
//! leaning. The leaning wall's plane is stored rounded, so its two meets with
//! the other two lines come out as two corners a rounding apart, and the side
//! between them is a rounding long. Every fan from every corner of the piece
//! then holds a triangle on that side, and no point is turned about it either
//! way. A point well inside the piece is outside each of those triangles all
//! the same, as two of their sides turn round it opposite ways, and since
//! Q1145 that is enough. Two of the pairs answer now. The third gets past
//! its top and stopped on a wall until Q1279; now its intersection and
//! difference answer, and its union is refused as a limit, for an edge a
//! rounding long.

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::{Decision, Policy, ProvenanceLog, side_of_a_plane_where_three_planes_meet};
use rcad_topo::{Error, Invalid, Model, Rule, SolidKey};

use pairs::{Configuration, Elsewhere, Placed, Shape, Skew};

/// A comb of two teeth one high, standing two high at the origin, and a frame
/// three by four, leaning one along `x` for each step along `y`, standing on
/// the comb's top at `(5, −3, 2)`. The frame's floor met the comb's planes
/// in a strip between its leaning wall and its hole, with two corners a
/// rounding from `(3, −2, 2)`.
///
/// They share no space, so the union holds 40, which is the comb's 10 and the
/// frame's 30, the intersection holds nothing, and the difference is the
/// whole comb.
#[test]
fn a_comb_and_a_leaning_frame_standing_on_its_top_share_no_space() {
    the_two_share_no_space(
        Configuration {
            solids: vec![
                Placed {
                    shape: Shape::Comb {
                        teeth: 2,
                        tooth_height: 1,
                    },
                    skew: Skew::none(),
                    height: 2,
                    at: [0, 0, 0],
                },
                Placed {
                    shape: Shape::Frame { width: 3, depth: 4 },
                    skew: Skew { x_per_y: 1 },
                    height: 3,
                    at: [5, -3, 2],
                },
            ],
            turn: None,
            elsewhere: Elsewhere::here(),
        },
        [10.0, 30.0],
    );
}

/// A comb of one tooth two high, leaning two along `x` for each step along
/// `y` and standing two high at the origin, and a frame five by five leaning
/// one, standing three high at `(2, −4, −1)`. The comb's top met the frame's
/// planes in a piece with two corners a rounding from `(6, 3, 2)`.
///
/// They share no space either: the union holds 54, the comb's 6 and the
/// frame's 48.
#[test]
fn a_leaning_tooth_and_a_leaning_frame_around_it_share_no_space() {
    the_two_share_no_space(
        Configuration {
            solids: vec![
                Placed {
                    shape: Shape::Comb {
                        teeth: 1,
                        tooth_height: 2,
                    },
                    skew: Skew { x_per_y: 2 },
                    height: 2,
                    at: [0, 0, 0],
                },
                Placed {
                    shape: Shape::Frame { width: 5, depth: 5 },
                    skew: Skew { x_per_y: 1 },
                    height: 3,
                    at: [2, -4, -1],
                },
            ],
            turn: None,
            elsewhere: Elsewhere::here(),
        },
        [6.0, 48.0],
    );
}

/// A block two by two leaning one along `x` for each step along `y`, two high
/// at the origin, and a block two by three leaning two the other way, two
/// high at `(5, 1, 1)`. The second block's corner edge at `(3, 1)` stands on
/// the first block's leaning wall `x − y = 2`, so that wall and the second
/// block's walls `y = 1` and `x + 2y = 5` meet in one upright line.
///
/// Before Q1145 every rule stopped on the first block's top, `z = 2`, on a
/// piece with corners at `(3, 1)` twice, `(5/3, 5/3)` and `(1, 1)`, though
/// `(7/3, 4/3)` is well inside it. It got past that and stopped on the first
/// block's leaning wall instead, on a piece between `z = 0` and `z = 1` whose
/// four corners are all a rounding from the upright line through `(3, 1)`.
/// Since Q1279 a plane of the second block with every corner of the piece on
/// or in front of it, and one strictly, places it.
///
/// So the intersection answers, the parallelogram the two share between
/// `y = 1` and `y = 2`, area `7/6`, one high from `z = 1` to `z = 2`, and the
/// first less the second holds `8 − 7/6 = 41/6`.
///
/// **The union is a limit**, refused [`Invalid::DynamicRangeTooWide`]. The
/// strip of the second block's wall `x + 2y = 5` from `z = 1` to `z = 2` is
/// placed outside the first, and its end runs along that block's floor edge,
/// from its corner on `y = 1` to where the stored plane of the first block's
/// wall `x − y = 2` crosses it. Those two points are not one, the planes being
/// stored rounded (the test below shows it exactly), and they round to
/// `(3.0000000000000004, 1, 1)` and `(3, 1, 1)`: an edge `2^-51` long, beside
/// a largest coordinate of 5, a spread of `5 · 2^51` against the `1e11` a model
/// supports (`docs/decisions.md` §3.5). It is a length over a length, so no
/// scaling of the answer fits the two together (`Assembly::check_dynamic_range`
/// in `fuse/assemble.rs`).
#[test]
fn two_leaning_blocks_meeting_on_one_upright_line_answer_but_their_union_needs_an_edge_too_short() {
    let (model, keys) = two_leaning_blocks_meeting_on_one_upright_line().build();
    holds_exactly(&model, &keys[..1], 8.0, "the first");
    holds_exactly(&model, &keys[1..], 12.0, "the second");

    let (both, solids) = fused(&model, keys[0], keys[1], Rule::Intersection);
    holds_exactly(&both, &solids, 7.0 / 6.0, "the intersection");

    let (less, solids) = fused(&model, keys[0], keys[1], Rule::Difference);
    holds_exactly(&less, &solids, 41.0 / 6.0, "the difference");

    let mut model = model.clone();
    let union = model.fuse(
        &[keys[0]],
        &[keys[1]],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    let Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) = union else {
        panic!("the union should be refused for an edge too short: {union:?}");
    };
    assert_eq!(ratio, 5.0 * 2.0_f64.powi(51));
}

/// **The second block's corner is a rounding off the first block's wall.**
/// The corner is the meet of the second block's floor `z = 1`, its wall
/// `y = 1` and its wall `x + 2y = 5`, and asked exactly of the planes the model
/// stores, it is strictly in front of the first block's wall `x − y = 2`,
/// outside that block, though `(3, 1, 1)` is on that wall. So the edge of the
/// union's answer from that corner along the floor to the wall has a length,
/// which is what the test above finds too short.
#[test]
fn the_second_blocks_corner_is_a_rounding_outside_the_first_blocks_wall() {
    let (model, keys) = two_leaning_blocks_meeting_on_one_upright_line().build();
    let planes_of = |solid: SolidKey| -> Vec<[f64; 4]> {
        model
            .solid(solid)
            .expect("this model made the key")
            .faces()
            .map(|face| face.plane().coefficients())
            .collect()
    };
    let one_of = |planes: &[[f64; 4]], which: &dyn Fn(&[f64; 4]) -> bool| -> [f64; 4] {
        let found: Vec<[f64; 4]> = planes
            .iter()
            .copied()
            .filter(|plane| which(plane))
            .collect();
        let [plane] = found[..] else {
            panic!("one plane, not {found:?}");
        };
        plane
    };
    let first = planes_of(keys[0]);
    let second = planes_of(keys[1]);
    let leaning_out_along_x = one_of(&first, &|plane| plane[0] > 0.0 && plane[1] < 0.0);
    let corner = [
        one_of(&second, &|plane| plane[2] < 0.0),
        one_of(&second, &|plane| plane[1] < 0.0 && plane[0] == 0.0),
        one_of(&second, &|plane| plane[0] < 0.0 && plane[1] < 0.0),
    ];
    assert_eq!(
        side_of_a_plane_where_three_planes_meet(corner, leaning_out_along_x),
        Ok(Decision::Pos),
        "{corner:?} against {leaning_out_along_x:?}"
    );
}

fn two_leaning_blocks_meeting_on_one_upright_line() -> Configuration {
    Configuration {
        solids: vec![
            Placed {
                shape: Shape::Block { width: 2, depth: 2 },
                skew: Skew { x_per_y: 1 },
                height: 2,
                at: [0, 0, 0],
            },
            Placed {
                shape: Shape::Block { width: 2, depth: 3 },
                skew: Skew { x_per_y: -2 },
                height: 2,
                at: [5, 1, 1],
            },
        ],
        turn: None,
        elsewhere: Elsewhere::here(),
    }
}

/// The union of the two holds what each holds, the intersection is no solid
/// at all, and the first less the second is the first.
fn the_two_share_no_space(pair: Configuration, [first, second]: [f64; 2]) {
    let (model, keys) = pair.build();
    holds_exactly(&model, &keys[..1], first, "the first");
    holds_exactly(&model, &keys[1..], second, "the second");

    let (united, solids) = fused(&model, keys[0], keys[1], Rule::Union);
    holds_exactly(&united, &solids, first + second, "the union");

    let (_, solids) = fused(&model, keys[0], keys[1], Rule::Intersection);
    assert_eq!(solids, [], "the intersection");

    let (less, solids) = fused(&model, keys[0], keys[1], Rule::Difference);
    holds_exactly(&less, &solids, first, "the difference");
}

/// Fuse a copy of `model` under a certified policy, and check that what came
/// back validates and that nothing was left unproved.
fn fused(
    model: &Model,
    arguments: SolidKey,
    tools: SolidKey,
    rule: Rule,
) -> (Model, Vec<SolidKey>) {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    let solids = match model.fuse(&[arguments], &[tools], rule, &Policy::certified(), &mut log) {
        Ok(fused) => fused.solids().to_vec(),
        Err(why) => panic!("the {rule:?} refused: {why:?}"),
    };
    assert!(log.is_empty(), "a certified {rule:?} logged {log:?}");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse's answer validates");
    (model, solids)
}

/// The solids hold `volume` together, to within the rounding their measures
/// own up to.
fn holds_exactly(model: &Model, solids: &[SolidKey], volume: f64, what: &str) {
    assert!(!solids.is_empty(), "{what} is no solid at all");
    let (mut measured, mut error) = (0.0, 0.0);
    for &solid in solids {
        let measure = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.1).expect("a tenth of a millimetre is a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a flat solid this size measures");
        measured += measure.volume();
        error += measure.volume_error();
    }
    assert!(
        (measured - volume).abs() <= error,
        "{what} holds {measured} give or take {error}, not {volume}"
    );
}
