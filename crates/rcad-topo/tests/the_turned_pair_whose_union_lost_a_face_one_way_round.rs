//! §126.6's counterexample, recovered — and the kernel never had anything to do
//! with it.
//!
//! §126.5 saw `swapping_the_two_sides_of_a_turned_pair_changes_not_one_bit`
//! fail while §126 was merged onto `ef787559`, put the tree back to make `main`
//! safe, and threw the persisted seed away with it. §126.6 worked out what had
//! happened and left the counterexample unreproduced: a union whose answer
//! differed by a face depending on which solid came first.
//!
//! Here it is. An **ell** two by two with a one-by-one bite, and a **comb** of
//! two teeth three high at `(-1, -4, 0)`, the pair turned together by
//! thirty-two sixty-fourths — half a turn — about `(0, 3, 2)`. On `ae35781c`,
//! the tree that failed, that pair fails the property on its own, with no seed
//! and no search.
//!
//! **And the two answers are the same solid.** The union has a face whose ring
//! goes through the origin twice — the origin is the one point the turn holds
//! still, and both solids have an input corner there. `fuses_that_must_agree`
//! starts a ring at the *first* corner equal to the smallest one, so two walks
//! that reach the origin from different sides write the same ring down as two
//! different lists. Compare the ring by its smallest **rotation** instead and
//! the two orders agree bit for bit, on that tree and on this one.
//!
//! So what §126.5 saw was a false positive in the oracle: starting a ring at
//! the first corner equal to the smallest one is not a canonical form for a
//! ring that repeats a corner. **§151 fixed that** — `fuses_that_must_agree`
//! takes the smallest rotation now, and its own test holds the rule.
//!
//! What is left here is the pair itself, which is worth keeping: the suite
//! draws it about once in 2 300 runs, and these tests ask it every time.

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
mod pairs;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

use pairs::{Configuration, Elsewhere, Placed, Shape, Skew, Turn};

#[test]
fn an_ell_and_a_comb_turned_half_a_turn_unite_to_the_same_solid_either_way_round() {
    both_ways_round_agree(Rule::Union);
}

#[test]
fn an_ell_and_a_comb_turned_half_a_turn_intersect_to_the_same_solid_either_way_round() {
    both_ways_round_agree(Rule::Intersection);
}

/// The condition the suite's rule needs and this answer breaks: the union has a
/// face whose ring goes through one corner twice.
///
/// The origin is what the turn holds still, and both solids have an input corner
/// there, so the turned pair meets itself at exactly one point of `f64`.
#[test]
fn the_union_of_this_pair_has_a_ring_that_goes_through_one_corner_twice() {
    let (model, keys) = the_pair_that_disagreed().build();
    assert!(
        a_ring_visits_one_corner_twice(&model, &[keys[0]], &[keys[1]], Rule::Union),
        "the union no longer has a ring that repeats a corner, so this pair no \
         longer demonstrates anything"
    );
    assert!(
        !a_ring_visits_one_corner_twice(&model, &[keys[0]], &[keys[1]], Rule::Intersection),
        "the intersection has grown one too, and it is the control"
    );
}

fn both_ways_round_agree(rule: Rule) {
    let (model, keys) = the_pair_that_disagreed().build();
    let (one, other) = (keys[0], keys[1]);

    let forwards = fused(&model, &[one], &[other], rule).expect("the ell against the comb answers");
    let backwards =
        fused(&model, &[other], &[one], rule).expect("the comb against the ell answers");

    if forwards == backwards {
        return;
    }
    panic!(
        "{rule:?} depends on which of the two comes first: {} solids and {} faces one way, \
         {} solids and {} faces the other; first difference at {}",
        forwards.len(),
        forwards.iter().map(Vec::len).sum::<usize>(),
        backwards.len(),
        backwards.iter().map(Vec::len).sum::<usize>(),
        where_they_part(&forwards, &backwards),
    );
}

/// Which face of which solid is the first the two orders do not share.
fn where_they_part(forwards: &[Vec<Face>], backwards: &[Vec<Face>]) -> String {
    for (solid, (one, other)) in forwards.iter().zip(backwards).enumerate() {
        for (face, (mine, theirs)) in one.iter().zip(other).enumerate() {
            if mine != theirs {
                return format!("solid {solid} face {face}:\n  {mine:?}\nagainst\n  {theirs:?}");
            }
        }
    }
    "no face in common position differs, so the counts do".to_owned()
}

/// The configuration proptest shrank the failure to, copied out of the seed it
/// persisted before the tree was put back.
fn the_pair_that_disagreed() -> Configuration {
    Configuration {
        solids: vec![
            Placed {
                shape: Shape::Ell {
                    width: 2,
                    depth: 2,
                    bite_width: 1,
                    bite_depth: 1,
                },
                skew: Skew::none(),
                height: 1,
                at: [0, 0, 0],
            },
            Placed {
                shape: Shape::Comb {
                    teeth: 2,
                    tooth_height: 3,
                },
                skew: Skew::none(),
                height: 1,
                at: [-1, -4, 0],
            },
        ],
        turn: Some(Turn {
            direction: [0, 3, 2],
            steps: 32,
        }),
        elsewhere: Elsewhere::here(),
    }
}

/// One face written down so that two runs compare: what it sits on and which
/// way round its solid uses it, then its wires as rings of corner bits.
type Face = (String, Vec<Vec<u64>>);

/// Fuse a copy of `model` and write the answer down so that two of them compare.
///
/// `None` covers both ways there is no answer — a refusal, and an answer no
/// `Model` can hold. Neither happens here, and the callers say so by name.
fn fused(
    model: &Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> Option<Vec<Vec<Face>>> {
    let mut model = model.clone();
    let solids = model
        .fuse(
            arguments,
            tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .ok()?
        .solids()
        .to_vec();
    Some(written_out(&model, &solids))
}

/// Whether any ring of the answer comes back to a corner it has already been to.
fn a_ring_visits_one_corner_twice(
    model: &Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> bool {
    let Some(answer) = fused(model, arguments, tools, rule) else {
        return false;
    };
    answer.iter().flatten().any(|(_, wires)| {
        wires.iter().any(|wire| {
            let corners: Vec<&[u64]> = wire.chunks(3).collect();
            (0..corners.len()).any(|one| (0..one).any(|other| corners[one] == corners[other]))
        })
    })
}

/// Every face of every solid, in an order neither argument chose.
fn written_out(model: &Model, solids: &[SolidKey]) -> Vec<Vec<Face>> {
    let mut answer: Vec<Vec<Face>> = solids
        .iter()
        .map(|&solid| {
            let mut faces: Vec<Face> = model
                .solid(solid)
                .expect("this model made the key")
                .faces()
                .map(|face| {
                    let sits = format!("{:?} {:?}", face.surface(), face.orientation());
                    let mut wires: Vec<Vec<u64>> = face
                        .wires()
                        .map(|wire| {
                            turned_to_its_smallest(
                                &wire.points().map(|at| at.coordinates()).collect::<Vec<_>>(),
                            )
                        })
                        .collect();
                    wires.sort_unstable();
                    (sits, wires)
                })
                .collect();
            faces.sort_unstable();
            faces
        })
        .collect();
    answer.sort_unstable();
    answer
}

/// A ring starts wherever the walk started, so start it where the whole ring
/// reads smallest instead.
///
/// **The smallest rotation, not the smallest corner.** `fuses_that_must_agree`'s
/// copy of this takes the first corner equal to the smallest one, which is a
/// canonical form only while no ring visits one corner twice. This pair's union
/// has a face whose outer ring passes through the origin twice, so the two
/// orders pick different starting corners for the same ring and the comparison
/// reports a difference that is not there.
fn turned_to_its_smallest(wire: &[[f64; 3]]) -> Vec<u64> {
    let corners: Vec<[u64; 3]> = wire.iter().map(|&at| at.map(bits)).collect();
    (0..corners.len())
        .map(|start| written_from(&corners, start))
        .min()
        .unwrap_or_default()
}

/// The same ring as one list of numbers, read from `start`.
fn written_from(corners: &[[u64; 3]], start: usize) -> Vec<u64> {
    (0..corners.len())
        .flat_map(|step| corners[(start + step) % corners.len()])
        .collect()
}

/// `-0.0` and `0.0` are the same corner, and their bits are not.
fn bits(coordinate: f64) -> u64 {
    if coordinate == 0.0 {
        0
    } else {
        coordinate.to_bits()
    }
}
