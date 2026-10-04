//! A comb's tooth corner lying on an ell's edge, turned, fuses the same both
//! ways round (`docs/decisions.md` §351.5, queue row Q711).
//!
//! The comb has three teeth one high and stands at the origin. The ell is four
//! by three with a one-by-one bite, three high, at `(2, 0, 0)`. Both are turned
//! together by 27/64 of a turn about `(2, 1, 2)`. This is configuration 52 of
//! `swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit`.
//!
//! The comb's input corner `(2, 2, 0)` lies on the ell's edge between its floor
//! and its wall at `x = 2`. Both solids store those two planes with the same
//! bits. Selection keeps the floor and the wall of only one of the two solids,
//! whichever came first. In one order the corner then came from the ell's
//! pieces, named after the three planes meeting there, and from the comb's
//! tooth wall at `y = 2`, named after the comb's input vertex. The two names
//! did not match. The two points were 4 ulps apart in `y`, so the coordinates
//! did not join them either. Four edges got one face each, and the fuse
//! refused with `ResultIsNotManifold { count: 1 }`: the union with the comb
//! first and the intersection with the ell first.
//!
//! Both answers exist. The other order gave each of them.

// `pairs` describes its frustum through `cones::a_frustum`.
#[path = "fuzzing/comparison.rs"]
mod comparison;
mod cones;
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

use comparison::{answer_of, same_answer};
use pairs::{Configuration, Elsewhere, Placed, Shape, Skew, Turn};

#[test]
fn the_comb_and_the_ell_unite_to_one_solid_whichever_comes_first() {
    both_ways_round_give_one_solid_of(Rule::Union, 36.0);
}

#[test]
fn the_comb_and_the_ell_share_one_solid_whichever_comes_first() {
    both_ways_round_give_one_solid_of(Rule::Intersection, 5.0);
}

/// Both orders give one solid of `volume`, and the two are the same solid bit
/// for bit.
///
/// The comb takes up 8 and the ell 33. They share the comb's base from `x = 2`
/// to `x = 5` and the two teeth standing on it, which is 5, so their union is
/// 36.
fn both_ways_round_give_one_solid_of(rule: Rule, volume: f64) {
    let (model, keys) = the_comb_and_the_ell().build();
    let forwards = fused(&model, keys[0], keys[1], rule);
    let backwards = fused(&model, keys[1], keys[0], rule);
    for ((model, solids), order) in [(&forwards, "comb first"), (&backwards, "ell first")] {
        assert_eq!(
            solids.len(),
            1,
            "the {rule:?} with the {order} is one solid"
        );
        let measured = volume_of(model, solids);
        assert!(
            (measured - volume).abs() < 1e-9,
            "the {rule:?} with the {order} takes up {measured}, not {volume}"
        );
    }
    if let Err(why) = same_answer(
        &answer_of(&forwards.0, &forwards.1),
        &answer_of(&backwards.0, &backwards.1),
        rule,
        "swapping the comb and the ell",
    ) {
        panic!("{why}");
    }
}

fn the_comb_and_the_ell() -> Configuration {
    Configuration {
        solids: vec![
            Placed {
                shape: Shape::Comb {
                    teeth: 3,
                    tooth_height: 1,
                },
                skew: Skew::none(),
                height: 1,
                at: [0, 0, 0],
            },
            Placed {
                shape: Shape::Ell {
                    width: 4,
                    depth: 3,
                    bite_width: 1,
                    bite_depth: 1,
                },
                skew: Skew::none(),
                height: 3,
                at: [2, 0, 0],
            },
        ],
        turn: Some(Turn {
            direction: [2, 1, 2],
            steps: 27,
        }),
        elsewhere: Elsewhere::here(),
    }
}

/// Fuse a copy of `model` under a certified policy, and check that what came
/// back validates.
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
        Err(why) => panic!("the {rule:?} of {arguments:?} against {tools:?} refused: {why:?}"),
    };
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse's answer validates");
    (model, solids)
}

fn volume_of(model: &Model, solids: &[SolidKey]) -> f64 {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.1).expect("a tenth of a millimetre is a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a solid this size measures")
                .volume()
        })
        .sum()
}
