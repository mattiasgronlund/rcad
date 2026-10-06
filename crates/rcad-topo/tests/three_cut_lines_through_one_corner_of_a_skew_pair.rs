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
//! its top and stops on a wall.

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

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
/// `(7/3, 4/3)` is well inside it. It gets past that now and stops on the
/// first block's leaning wall instead, on a piece between `z = 0` and `z = 1`
/// whose four corners are all a rounding from the upright line through
/// `(3, 1)`: the cut lines of `y = 1` and `x + 2y = 5` on that wall are one
/// line, and the piece between them holds nothing. This pins that stop, so a
/// fix of it moves this test.
#[test]
fn two_leaning_blocks_meeting_on_one_upright_line_stop_on_a_piece_of_wall_holding_nothing() {
    let (model, keys) = Configuration {
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
    .build();
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        let mut model = model.clone();
        let refusal = model.fuse(
            &[keys[0]],
            &[keys[1]],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let Err(Error::Undecidable { records }) = refusal else {
            panic!("the {rule:?} should still refuse as undecidable: {refusal:?}");
        };
        let [record] = records.records() else {
            panic!("the {rule:?} records one refusal: {records:?}");
        };
        assert_eq!(
            record.predicate(),
            "a point inside a piece of a face",
            "the {rule:?}"
        );
        let heights: Vec<Option<f64>> = record
            .inputs()
            .iter()
            .map(|corner| {
                let holds = |axis: usize, value: f64| {
                    let side = corner.enclosure()[axis];
                    side.lo() <= value && value <= side.hi()
                };
                assert!(
                    holds(0, 3.0) && holds(1, 1.0),
                    "the {rule:?} stops on a corner off the line through (3, 1): {corner:?}"
                );
                [0.0, 1.0].into_iter().find(|&height| holds(2, height))
            })
            .collect();
        assert_eq!(
            heights,
            [Some(0.0), Some(0.0), Some(1.0), Some(1.0)],
            "the {rule:?}"
        );
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
