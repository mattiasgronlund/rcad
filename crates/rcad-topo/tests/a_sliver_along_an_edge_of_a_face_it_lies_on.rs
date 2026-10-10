//! Four pairs from `how_often_a_certified_fuse_of_a_skew_pair_refuses` that
//! each stopped on a sliver a rounding across, lying on the plane of a face of
//! the other solid along one of that face's edges (queue rows Q1034 and Q1606,
//! `docs/decisions.md` §365.8). Before Q1606 every rule of all four refused
//! `a point inside a piece of a face`. Two now answer every rule, and two stop
//! further on.

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::{Point, Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

use pairs::{Configuration, Elsewhere, Placed, Shape, Skew};

const INSIDE_A_PIECE: &str = "a point inside a piece of a face";

/// A frame four by three, leaning back one along `x` for each step along `y`,
/// and a comb of three teeth two high standing at `(−2, −2, 0)`, both one high.
///
/// The comb's middle tooth and its right one each reach one square into the
/// frame's near wall, so the intersection is two unit cubes. The right tooth's
/// top corner `(2, 1)` is the corner of the frame's hole, where its leaning
/// wall `x + y = 3` meets its wall `y = 1`. That leaning wall is stored
/// rounded, so the comb's wall `x = 2` and it cut a sliver a rounding across
/// out of the frame's wall `y = 1`, holding no `f64` point, along the edge of
/// the tooth's top. Q1606's rule places it by that edge.
///
/// Neither the union nor the frame less the comb is a solid: each has two
/// parts that touch only along a line, with four faces along it. In the frame
/// less the comb, the near wall left of the right tooth and the side wall above
/// it touch along `x = 2, y = 1`. In the union, the comb's left tooth touches
/// the frame's leaning outer wall along `x = −1, y = 1`.
#[test]
fn a_leaning_frame_and_a_comb_meet_in_two_squares() {
    let (model, keys) = frame_and_comb().build();
    holds_exactly(&model, &keys[..1], 10.0, "the frame");
    holds_exactly(&model, &keys[1..], 11.0, "the comb");

    let (met, solids) = fused(&model, keys[0], keys[1], Rule::Intersection);
    assert_eq!(solids.len(), 2, "the intersection is two squares");
    holds_exactly(&met, &solids, 2.0, "the intersection");

    is_not_a_solid(&model, keys[0], keys[1], Rule::Union);
    is_not_a_solid(&model, keys[0], keys[1], Rule::Difference);
}

/// The same two with the comb first: the comb less the frame holds 9, the
/// comb's 11 less the 2 they share, and the union is still not a solid.
#[test]
fn a_comb_and_a_leaning_frame_meet_in_two_squares() {
    let (model, keys) = frame_and_comb().build();

    let (met, solids) = fused(&model, keys[1], keys[0], Rule::Intersection);
    assert_eq!(solids.len(), 2, "the intersection is two squares");
    holds_exactly(&met, &solids, 2.0, "the intersection");

    let (less, solids) = fused(&model, keys[1], keys[0], Rule::Difference);
    holds_exactly(&less, &solids, 9.0, "the difference");

    is_not_a_solid(&model, keys[1], keys[0], Rule::Union);
}

/// A comb of two teeth three high, leaning forward one along `x` for each step
/// along `y`, one high at the origin, and an ell five by four with a bite one
/// by three, leaning back two, three high on the comb's top at `(1, 1, 1)`.
///
/// They touch only across `z = 1`, so the intersection holds nothing and each
/// less the other is itself: the comb's 9 and the ell's 51. The union is not a
/// solid: the ell's near wall `y = 1` runs along the bottom of the comb's gap
/// between its teeth, from `x = 2` to `3`, the ell above it and the comb below,
/// so four faces meet along that line.
#[test]
fn a_leaning_comb_and_a_leaning_ell_standing_on_it_share_no_space() {
    let (model, keys) = comb_and_ell().build();
    holds_exactly(&model, &keys[..1], 9.0, "the comb");
    holds_exactly(&model, &keys[1..], 51.0, "the ell");

    let (_, solids) = fused(&model, keys[0], keys[1], Rule::Intersection);
    assert_eq!(solids, [], "the intersection");

    let (less, solids) = fused(&model, keys[0], keys[1], Rule::Difference);
    holds_exactly(&less, &solids, 9.0, "the difference");

    is_not_a_solid(&model, keys[0], keys[1], Rule::Union);
}

/// The same two with the ell first: the intersection still holds nothing, the
/// ell less the comb is the ell, and the union is still not a solid.
#[test]
fn a_leaning_ell_standing_on_a_leaning_comb_shares_no_space_with_it() {
    let (model, keys) = comb_and_ell().build();

    let (_, solids) = fused(&model, keys[1], keys[0], Rule::Intersection);
    assert_eq!(solids, [], "the intersection");

    let (less, solids) = fused(&model, keys[1], keys[0], Rule::Difference);
    holds_exactly(&less, &solids, 51.0, "the difference");

    is_not_a_solid(&model, keys[1], keys[0], Rule::Union);
}

/// A frame three by three, leaning forward one, two high at the origin, and an
/// ell four by four with a bite three by one, leaning back two, one high at
/// `(3, 1, 1)`. They hold 16 and 13.
///
/// Every rule, in either order, still stops on a triangle a rounding across at
/// the frame's hole corner `(3, 2, 1)`, on the ell's floor, refused `a point
/// inside a piece of a face`. Q1606's rule does not place it.
#[test]
fn a_leaning_frame_and_a_leaning_ell_still_stop_at_the_frames_hole_corner() {
    let (model, keys) = frame_and_ell().build();
    holds_exactly(&model, &keys[..1], 16.0, "the frame");
    holds_exactly(&model, &keys[1..], 13.0, "the ell");
    for [first, second] in [[keys[0], keys[1]], [keys[1], keys[0]]] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            stops_at(&model, first, second, rule, [3.0, 2.0, 1.0]);
        }
    }
}

/// A comb of three teeth two high, leaning forward two, two high at the
/// origin, and a comb of two teeth two high, leaning back two, one high at
/// `(3, 0, 1)`. They hold 22 and 7.
///
/// Before Q1606 the first comb first stopped at `(4, 1, 2)`. That sliver is
/// placed now, and every rule, in either order, stops at `(4, 1, 1)` instead,
/// refused `a point inside a piece of a face`.
#[test]
fn two_combs_leaning_opposite_ways_still_stop_at_one_corner() {
    let (model, keys) = comb_and_comb().build();
    holds_exactly(&model, &keys[..1], 22.0, "the first comb");
    holds_exactly(&model, &keys[1..], 7.0, "the second comb");
    for [first, second] in [[keys[0], keys[1]], [keys[1], keys[0]]] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            stops_at(&model, first, second, rule, [4.0, 1.0, 1.0]);
        }
    }
}

fn configuration(first: Placed, second: Placed) -> Configuration {
    Configuration {
        solids: vec![first, second],
        turn: None,
        elsewhere: Elsewhere::here(),
    }
}

fn frame_and_comb() -> Configuration {
    configuration(
        Placed {
            shape: Shape::Frame { width: 4, depth: 3 },
            skew: Skew { x_per_y: -1 },
            height: 1,
            at: [0, 0, 0],
        },
        Placed {
            shape: Shape::Comb {
                teeth: 3,
                tooth_height: 2,
            },
            skew: Skew::none(),
            height: 1,
            at: [-2, -2, 0],
        },
    )
}

fn comb_and_ell() -> Configuration {
    configuration(
        Placed {
            shape: Shape::Comb {
                teeth: 2,
                tooth_height: 3,
            },
            skew: Skew { x_per_y: 1 },
            height: 1,
            at: [0, 0, 0],
        },
        Placed {
            shape: Shape::Ell {
                width: 5,
                depth: 4,
                bite_width: 1,
                bite_depth: 3,
            },
            skew: Skew { x_per_y: -2 },
            height: 3,
            at: [1, 1, 1],
        },
    )
}

fn frame_and_ell() -> Configuration {
    configuration(
        Placed {
            shape: Shape::Frame { width: 3, depth: 3 },
            skew: Skew { x_per_y: 1 },
            height: 2,
            at: [0, 0, 0],
        },
        Placed {
            shape: Shape::Ell {
                width: 4,
                depth: 4,
                bite_width: 3,
                bite_depth: 1,
            },
            skew: Skew { x_per_y: -2 },
            height: 1,
            at: [3, 1, 1],
        },
    )
}

fn comb_and_comb() -> Configuration {
    configuration(
        Placed {
            shape: Shape::Comb {
                teeth: 3,
                tooth_height: 2,
            },
            skew: Skew { x_per_y: 2 },
            height: 2,
            at: [0, 0, 0],
        },
        Placed {
            shape: Shape::Comb {
                teeth: 2,
                tooth_height: 2,
            },
            skew: Skew { x_per_y: -2 },
            height: 1,
            at: [3, 0, 1],
        },
    )
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

/// The fuse's answer has four faces along one edge, so it is not a solid.
fn is_not_a_solid(model: &Model, arguments: SolidKey, tools: SolidKey, rule: Rule) {
    let mut model = model.clone();
    let answer = model
        .fuse(
            &[arguments],
            &[tools],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|fused| fused.solids().to_vec());
    assert!(
        matches!(answer, Err(Error::ResultIsNotManifold { count: 4 })),
        "the {rule:?} gave {answer:?}"
    );
}

/// The fuse refuses with one record, naming `a point inside a piece of a face`
/// and the exact point `corner` among its inputs.
fn stops_at(model: &Model, arguments: SolidKey, tools: SolidKey, rule: Rule, corner: [f64; 3]) {
    let mut model = model.clone();
    let answer = model.fuse(
        &[arguments],
        &[tools],
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    let records = match answer {
        Err(Error::Undecidable { records }) => records,
        other => panic!(
            "the {rule:?} gave {:?}",
            other.map(|fused| fused.solids().to_vec())
        ),
    };
    let [record] = records.records() else {
        panic!("the {rule:?} refused with {records:#?}");
    };
    assert_eq!(
        record.predicate(),
        INSIDE_A_PIECE,
        "the {rule:?}: {record:#?}"
    );
    assert!(
        record.inputs().contains(&Point::Exact(corner)),
        "the {rule:?} stopped somewhere else: {record:#?}"
    );
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
