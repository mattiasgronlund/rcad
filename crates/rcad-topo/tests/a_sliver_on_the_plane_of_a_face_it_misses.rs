//! A pair from `how_often_a_certified_fuse_of_a_skew_pair_refuses` that
//! stopped on a triangle a rounding across, lying in the plane of a face of
//! the other solid and well off that face (queue row Q1357,
//! `docs/decisions.md` §365.9).

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

use pairs::{Configuration, Elsewhere, Placed, Shape, Skew};

/// A comb of two teeth two high, standing three high at the origin, and an
/// ell two by five with a bite one by four, leaning back one along `x` for
/// each step along `y`, standing one high on the comb's top at `(1, −3, 3)`.
///
/// The comb's walls `x = 2` and `y = 1` and the ell's wall `x + y = 3` all
/// pass through `(2, 1, 3)`, the comb's corner at the foot of its right tooth.
/// The ell's wall is stored rounded, so the three cut a triangle a rounding
/// across out of the comb's top, which holds no `f64` point. It lies in the
/// plane of the ell's floor, nowhere near the floor itself, which at `y = 1`
/// runs from `x = 0` to `1`. Every rule stopped on it, refused `a point
/// inside a piece of a face`.
///
/// They share no space, so the union holds 27, the comb's 21 and the ell's 6,
/// the intersection holds nothing, and the comb less the ell is the comb.
#[test]
fn a_comb_and_a_leaning_ell_standing_on_its_top_share_no_space() {
    let (model, keys) = a_comb_and_a_leaning_ell_standing_on_its_top().build();
    holds_exactly(&model, &keys[..1], 21.0, "the comb");
    holds_exactly(&model, &keys[1..], 6.0, "the ell");

    let (united, solids) = fused(&model, keys[0], keys[1], Rule::Union);
    holds_exactly(&united, &solids, 27.0, "the union");

    let (_, solids) = fused(&model, keys[0], keys[1], Rule::Intersection);
    assert_eq!(solids, [], "the intersection");

    let (less, solids) = fused(&model, keys[0], keys[1], Rule::Difference);
    holds_exactly(&less, &solids, 21.0, "the difference");
}

/// The same two with the ell first: the union still holds 27, the
/// intersection nothing, and the ell less the comb is the ell.
#[test]
fn a_leaning_ell_standing_on_a_combs_top_shares_no_space_with_it() {
    let (model, keys) = a_comb_and_a_leaning_ell_standing_on_its_top().build();

    let (united, solids) = fused(&model, keys[1], keys[0], Rule::Union);
    holds_exactly(&united, &solids, 27.0, "the union");

    let (_, solids) = fused(&model, keys[1], keys[0], Rule::Intersection);
    assert_eq!(solids, [], "the intersection");

    let (less, solids) = fused(&model, keys[1], keys[0], Rule::Difference);
    holds_exactly(&less, &solids, 6.0, "the difference");
}

fn a_comb_and_a_leaning_ell_standing_on_its_top() -> Configuration {
    Configuration {
        solids: vec![
            Placed {
                shape: Shape::Comb {
                    teeth: 2,
                    tooth_height: 2,
                },
                skew: Skew::none(),
                height: 3,
                at: [0, 0, 0],
            },
            Placed {
                shape: Shape::Ell {
                    width: 2,
                    depth: 5,
                    bite_width: 4,
                    bite_depth: 4,
                },
                skew: Skew { x_per_y: -1 },
                height: 1,
                at: [1, -3, 3],
            },
        ],
        turn: None,
        elsewhere: Elsewhere::here(),
    }
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
