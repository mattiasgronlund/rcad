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
//! **And the two answers were the same solid.** The union had a face whose ring
//! went through the origin twice — the origin is the one point the turn holds
//! still, and both solids have an input corner there. `fuses_that_must_agree`
//! started a ring at the *first* corner equal to the smallest one, so two walks
//! that reached the origin from different sides wrote the same ring down as two
//! different lists. Compared by its smallest **rotation** instead, the two
//! orders agreed bit for bit.
//!
//! So what §126.5 saw was a false positive in the oracle: starting a ring at
//! the first corner equal to the smallest one is not a canonical form for a
//! ring that repeats a corner. **§151 fixed that** — `fuses_that_must_agree`
//! takes the smallest rotation now, and its own test holds the rule.
//!
//! # The union is one solid touching itself along an edge
//!
//! Upright, the comb's right tooth sits flat against the ell's front and its
//! left tooth meets the ell along one edge only, from the origin to
//! `(0, 0, 1)`, the ell on one side of that line and the tooth on the other.
//! That line is an edge of both, with four faces along it, and the union was
//! refused there, `ResultIsNotManifold { count: 4 }` both ways round, as a
//! line §80.9 leaves refused. Turned, the union used to answer, with the ring
//! through the origin twice, and it was refused the same way as the upright
//! pair since an input corner with exactly three planes round it is named by
//! those planes and put where they put it (`docs/decisions.md` §351.6).
//!
//! But the two sides of the line are one solid, joined through the right
//! tooth, and since queue row Q1913 that is an answer (§39.4): one solid of
//! `3 + 9 = 12`, one shell of 14 faces, upright and turned, the two orders
//! the same bit for bit. Its top's ring goes through `(0, 0, 1)` twice and its
//! bottom's through the origin twice, so the ring §126.5 tripped over is back
//! in this pair's union. The intersection still answers, the same solid both
//! ways round.
//!
//! # A ring through one corner twice, from another pair
//!
//! An ell and a frame make such a ring too, and it is not something a turn
//! makes. The ell is five by four with a three-by-one bite, three high; the
//! frame is four by four, one high, at `(-3, -3, 0)`. The ell covers the
//! frame's far corner, where two of its walls meet, so the frame's top face is
//! left a C whose two ends touch at `(0, 0, 1)`, a corner of its hole. That
//! face's ring goes through `(0, 0, 1)` twice upright, and through where the
//! turn puts it when turned.
//!
//! §151.5 called §149's pair the only ring with a repeated corner anyone had
//! produced. The suite's lattice, turned and skew families all draw them,
//! upright ones included (`docs/decisions.md` §351.8).

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
mod pairs;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

use pairs::{Configuration, Elsewhere, Placed, Shape, Skew, Turn};

/// The union is one solid of `3 + 9 = 12`, with one shell of 14 faces, upright
/// and turned, and the two orders give it bit for bit.
///
/// The ell has 8 faces and the comb 10. The comb's right tooth sits flat against
/// the ell's front, so the tooth's top goes inside, which leaves 17. The comb's
/// wall `x = 2` and the ell's run on from each other, and so do their tops and
/// their bottoms, and each of those three pairs is one face, which leaves 14.
/// The left tooth touches the ell along `x = 0, y = 0` only, the ell on one
/// side of that line and the tooth on the other, so the top's ring goes
/// through `(0, 0, 1)` twice and the bottom's through the origin twice.
///
/// Turned, the corners are rounded, so what the solid holds is 12 only to
/// within that rounding: about `10^-14` off either way round.
#[test]
fn an_ell_and_a_comb_unite_to_one_solid_touching_itself_along_an_edge_turned_or_not() {
    for turn in [None, Some(half_a_turn())] {
        let pair = the_pair_that_disagreed(turn);
        let union = both_ways_round_agree(&pair, Rule::Union);
        assert_eq!(
            union.iter().map(Vec::len).collect::<Vec<_>>(),
            [14],
            "turned {}: the faces of each solid of the union",
            turn.is_some()
        );
        assert!(
            a_ring_visits_one_corner_twice(&union),
            "turned {}: no ring of the union goes through a corner twice, so the \
             line the ell and the comb touch along is no longer in it",
            turn.is_some()
        );

        let rounding = if turn.is_some() { 1e-12 } else { 0.0 };
        let (model, keys) = pair.build();
        for (one, other) in [(keys[0], keys[1]), (keys[1], keys[0])] {
            let (shells, volume) =
                shells_and_volume_of_the_one_solid_of_the_union(&model, one, other);
            assert_eq!(shells, 1, "turned {}: the union's shells", turn.is_some());
            assert!(
                (volume - 12.0).abs() <= rounding,
                "turned {}: the union holds {volume}, not 12",
                turn.is_some()
            );
        }
    }
}

#[test]
fn an_ell_and_a_comb_turned_half_a_turn_intersect_to_the_same_solid_either_way_round() {
    both_ways_round_agree(
        &the_pair_that_disagreed(Some(half_a_turn())),
        Rule::Intersection,
    );
}

/// The condition the suite's ring rule needs: an answer with a face whose ring
/// goes through one corner twice, which two orders reach from different sides.
///
/// Upright as well as turned, so the ring is the pair's and not the turn's. The
/// intersection is the control, with no such ring.
#[test]
fn an_ell_and_a_frame_unite_to_a_face_whose_ring_goes_through_one_corner_twice() {
    for turn in [None, Some(half_a_turn())] {
        let pair = an_ell_and_a_frame(turn);
        assert!(
            a_ring_visits_one_corner_twice(&both_ways_round_agree(&pair, Rule::Union)),
            "turned {}: the union has no ring that repeats a corner, so this pair \
             no longer demonstrates anything",
            turn.is_some()
        );
        assert!(
            !a_ring_visits_one_corner_twice(&both_ways_round_agree(&pair, Rule::Intersection)),
            "turned {}: the intersection has grown one too, and it is the control",
            turn.is_some()
        );
    }
}

/// The answer, after checking the two orders give it bit for bit.
fn both_ways_round_agree(pair: &Configuration, rule: Rule) -> Vec<Vec<Face>> {
    let (model, keys) = pair.build();
    let (one, other) = (keys[0], keys[1]);

    let forwards = fused(&model, &[one], &[other], rule)
        .unwrap_or_else(|refusal| panic!("{rule:?}, first against second: {refusal:?}"));
    let backwards = fused(&model, &[other], &[one], rule)
        .unwrap_or_else(|refusal| panic!("{rule:?}, second against first: {refusal:?}"));

    if forwards == backwards {
        return forwards;
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

/// Unite a copy of `model`, check the answer is one solid that validates, and
/// say how many shells it has and what it holds.
fn shells_and_volume_of_the_one_solid_of_the_union(
    model: &Model,
    arguments: SolidKey,
    tools: SolidKey,
) -> (usize, f64) {
    let mut model = model.clone();
    let solids = model
        .fuse(
            &[arguments],
            &[tools],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refusal| panic!("the union refused: {refusal:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse's answer validates");
    let [solid] = solids[..] else {
        panic!("the union is {} solids, not one", solids.len());
    };
    let solid = model.solid(solid).expect("this model made the key");
    let measure = solid
        .measure(
            Deflection::of(0.1).expect("a tenth of a millimetre is a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a flat solid this size measures");
    (solid.shells().count(), measure.volume())
}

/// The turn the pair that disagreed was drawn with.
fn half_a_turn() -> Turn {
    Turn {
        direction: [0, 3, 2],
        steps: 32,
    }
}

/// The configuration proptest shrank the failure to, copied out of the seed it
/// persisted before the tree was put back, with its turn left to the caller.
fn the_pair_that_disagreed(turn: Option<Turn>) -> Configuration {
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
        turn,
        elsewhere: Elsewhere::here(),
    }
}

/// An ell covering a frame's far corner, a pair the suite's turned family drew.
fn an_ell_and_a_frame(turn: Option<Turn>) -> Configuration {
    Configuration {
        solids: vec![
            Placed {
                shape: Shape::Ell {
                    width: 5,
                    depth: 4,
                    bite_width: 3,
                    bite_depth: 1,
                },
                skew: Skew::none(),
                height: 3,
                at: [0, 0, 0],
            },
            Placed {
                shape: Shape::Frame { width: 4, depth: 4 },
                skew: Skew::none(),
                height: 1,
                at: [-3, -3, 0],
            },
        ],
        turn,
        elsewhere: Elsewhere::here(),
    }
}

/// One face written down so that two runs compare: what it sits on and which
/// way round its solid uses it, then its wires as rings of corner bits.
type Face = (String, Vec<Vec<u64>>);

/// Fuse a copy of `model` and write the answer down so that two of them compare.
fn fused(
    model: &Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> Result<Vec<Vec<Face>>, Error> {
    let mut model = model.clone();
    let solids = model
        .fuse(
            arguments,
            tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )?
        .solids()
        .to_vec();
    Ok(written_out(&model, &solids))
}

/// Whether any ring of the answer comes back to a corner it has already been to.
fn a_ring_visits_one_corner_twice(answer: &[Vec<Face>]) -> bool {
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
/// **The smallest rotation, not the smallest corner.** Taking the first corner
/// equal to the smallest one is a canonical form only while no ring visits one
/// corner twice. The ell and the frame's union has a face whose ring passes
/// through one corner twice, so the two orders could pick different
/// starting corners for the same ring and the comparison would report a
/// difference that is not there.
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
