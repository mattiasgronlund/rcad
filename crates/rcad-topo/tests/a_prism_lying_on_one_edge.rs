//! A prism lying along one of its edges on a block's top face, the edge inside
//! the face and the insides apart.
//!
//! The block is eight wide, three deep and ten long, `−4 ≤ x ≤ 4`,
//! `−3 ≤ y ≤ 0`, `0 ≤ z ≤ 10`, volume 240. Its top face is `y = 0`: an
//! extrusion runs along `z`, and a quarter turn to lay one on its side is not
//! exact in `f64`, so the block is stood on its side instead. The prism is the
//! triangle `(0, 0)`, `(3, 2)`, `(−1, 2)`, area 4, its walls leaning away from
//! the corner at the origin on either side, so it touches the top face along
//! that one edge and nowhere else. It is laid two ways: from `z = 2` to `z = 8`
//! with its edge at `x = 0`, volume 24, and from `z = 2.5` to `z = 7.25` with
//! its edge at `x = −1`, volume 19. Either way the edge is strictly inside the
//! top face, so the line is on no edge of the block.
//!
//! **The union used to be refused `ResultIsNotManifold { count: 4 }`** (queue
//! row Q803). Along the edge there were four darts: the top face's two pieces
//! walking it opposite ways, and the prism's two faces either side of its own
//! edge, two input faces. `fuse/assemble.rs` parted only a line whose two
//! pairs are each pieces of one input face — a bar lying along a wall (Q760) —
//! and now parts off the top face's pair too, so the union is the two solids as
//! they stood.
//!
//! Two shapes that also reach four darts that way are pinned refused, because
//! their answer is one solid pinched along the line rather than two lumps:
//! a block less a prism lying inside it along its edge, and a block united
//! with a solid lying on it along one edge and reaching down into it somewhere
//! else. Both validate if they are parted, which is why the parting asks.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule, SolidKey};

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The block's volume: eight by three by ten.
const BLOCK: f64 = 240.0;

/// The prism's triangle, its corner at the origin, both walls leaning up.
const RESTING: [[f64; 2]; 3] = [[0.0, 0.0], [3.0, 2.0], [-1.0, 2.0]];

/// Where each prism is moved to, and how long it is.
const LAID: [([f64; 3], f64); 2] = [([0.0, 0.0, 2.0], 6.0), ([-1.0, 0.0, 2.5], 4.75)];

/// The regression: every rule both ways round, checked against volumes. The
/// union is the block and the prism as they stood, two lumps; the
/// intersection is no solid; each difference is the first solid whole.
#[test]
fn a_prism_lying_on_its_edge_on_a_blocks_top_answers_every_rule_both_ways_round() {
    let mut wrong = Vec::new();
    for (at, length) in LAID {
        let prism = 4.0 * length;
        for swapped in [false, true] {
            for rule in EVERY_RULE {
                let what =
                    format!("prism at {at:?} {length} long, {rule:?}, prism first {swapped}");
                let (model, answer) = fuse(swapped, rule, |model| {
                    [a_block(model), a_prism(model, RESTING, at, length)]
                });
                let should: &[f64] = match (rule, swapped) {
                    (Rule::Union, _) => &[prism, BLOCK],
                    (Rule::Difference, false) => &[BLOCK],
                    (Rule::Difference, true) => &[prism],
                    _ => &[],
                };
                if let Some(why) = differs(&model, answer, should) {
                    wrong.push(format!("{what}: {why}"));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The same prism with its edge at `x = −1.5`, from `z = 2` to `z = 8`, is
/// refused under every rule both ways round, `Undecidable` for want of *a
/// point inside a piece of a face*; at `x = 0` and `x = −1` no fuse refuses.
/// With the block first the piece is a sliver of the block's top along
/// `x = −1.5` from `z = 0` to `z = 2`, the prism's edge run back to the block's
/// end where no face of the prism reaches: corners `(−1.5, 0, 0)` and
/// `(−1.5, 0, 2)` exact, and two boxes a rounding wide in `x` beside them. Not
/// traced further. That refusal is the selection's, ahead of the assembly, and
/// the intersection and both differences reach it too. The answers exist, so
/// it is a gap; this pin fails the day it closes, and then the placement
/// belongs in [`LAID`].
#[test]
fn the_prism_with_its_edge_at_x_minus_one_and_a_half_is_refused_for_want_of_a_point_inside_a_piece()
{
    let mut wrong = Vec::new();
    for swapped in [false, true] {
        for rule in EVERY_RULE {
            let (_, answer) = fuse(swapped, rule, |model| {
                [
                    a_block(model),
                    a_prism(model, RESTING, [-1.5, 0.0, 2.0], 6.0),
                ]
            });
            match answer {
                Err(Error::Undecidable { records })
                    if !records.records().is_empty()
                        && records.records().iter().all(|record| {
                            record.predicate() == "a point inside a piece of a face"
                        }) => {}
                other => wrong.push(format!("{rule:?}, prism first {swapped}: {other:?}")),
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The prism turned over and moved into the block, its edge on the top face
/// from below: the triangle `(0, 0)`, `(−1, −2)`, `(3, −2)`, from `z = 2` to
/// `z = 8`. The union is the block, the intersection the prism, and the prism
/// less the block nothing.
///
/// The block less the prism has a prism-shaped hollow whose edge touches the
/// top face, which is one solid pinched along the line, and it stays refused.
/// The four darts are the same two pairs as on top, but the prism's walls,
/// turned round, lie on the inner side of the top face; parted, the answer
/// came back as one solid of two shells, 216, that validates.
#[test]
fn a_block_less_a_prism_lying_inside_it_along_its_edge_is_refused_and_the_rest_answer() {
    let mut wrong = Vec::new();
    let inside = [[0.0, 0.0], [-1.0, -2.0], [3.0, -2.0]];
    for swapped in [false, true] {
        for rule in EVERY_RULE {
            let what = format!("{rule:?}, prism first {swapped}");
            let (model, answer) = fuse(swapped, rule, |model| {
                [a_block(model), a_prism(model, inside, [0.0, 0.0, 2.0], 6.0)]
            });
            let outcome = match (rule, swapped) {
                (Rule::Difference, false) => refused_as_four_faces_along_one_edge(answer),
                (Rule::Union, _) => differs(&model, answer, &[BLOCK]),
                (Rule::Intersection, _) => differs(&model, answer, &[24.0]),
                _ => differs(&model, answer, &[]),
            };
            if let Some(why) = outcome {
                wrong.push(format!("{what}: {why}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A solid lying on the block's top along one edge, as the prism does, and
/// reaching down into the block a little further along: the outline `(0, 0)`,
/// `(2, 1)`, `(2, −1)`, `(3, −1)`, `(3, 2)`, `(−1, 2)`, area 7, from `z = 2` to
/// `z = 8`, volume 42, of which `1 × 1 × 6` is inside the block.
///
/// The union is one solid that touches itself along the line, and it stays
/// refused. Parted, it came back as one shell of 276 that validates. The
/// intersection is the bit inside, 6, and the differences are 234 and 36.
#[test]
fn a_solid_lying_on_a_block_along_an_edge_and_reaching_into_it_is_refused_united() {
    let mut wrong = Vec::new();
    for swapped in [false, true] {
        for rule in EVERY_RULE {
            let what = format!("{rule:?}, hook first {swapped}");
            let (model, answer) = fuse(swapped, rule, |model| [a_block(model), a_hook(model)]);
            let outcome = match (rule, swapped) {
                (Rule::Union, _) => refused_as_four_faces_along_one_edge(answer),
                (Rule::Intersection, _) => differs(&model, answer, &[6.0]),
                (Rule::Difference, false) => differs(&model, answer, &[234.0]),
                _ => differs(&model, answer, &[36.0]),
            };
            if let Some(why) = outcome {
                wrong.push(format!("{what}: {why}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The block and the other solid fused, the block first unless `swapped`.
fn fuse(
    swapped: bool,
    rule: Rule,
    build: impl FnOnce(&mut Model) -> [SolidKey; 2],
) -> (Model, rcad_topo::Result<Fused>) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let [block, other] = build(&mut model);
    let [first, second] = if swapped {
        [other, block]
    } else {
        [block, other]
    };
    let answer = model.fuse(
        &[first],
        &[second],
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    (model, answer)
}

/// What is wrong with an answer that should be one solid per volume in
/// `should`, in any order, each measured exactly; `None` when nothing is.
fn differs(model: &Model, answer: rcad_topo::Result<Fused>, should: &[f64]) -> Option<String> {
    let fused = match answer {
        Ok(fused) => fused,
        Err(why) => return Some(format!("refused {why:?}")),
    };
    if let Err(why) = model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
        return Some(format!("wrote a model that does not validate: {why}"));
    }
    let mut found: Vec<f64> = fused
        .solids()
        .iter()
        .map(|&solid| {
            let measured = model
                .solid(solid)
                .expect("the fuse made it")
                .measure(
                    Deflection::of(0.01).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a flat solid this size measures");
            assert_eq!(
                measured.volume_error(),
                0.0,
                "a flat solid on these whole and half numbers measures exactly"
            );
            measured.volume()
        })
        .collect();
    found.sort_by(f64::total_cmp);
    let mut should = should.to_vec();
    should.sort_by(f64::total_cmp);
    (found != should).then(|| format!("solids of {found:?}, and the answer is {should:?}"))
}

fn refused_as_four_faces_along_one_edge(answer: rcad_topo::Result<Fused>) -> Option<String> {
    match answer {
        Err(Error::ResultIsNotManifold { count: 4 }) => None,
        Err(why) => Some(format!("refused {why:?}, not as four faces along one edge")),
        Ok(fused) => Some(format!(
            "answered {:?}, which is one solid pinched along a line",
            fused.solids()
        )),
    }
}

fn a_block(model: &mut Model) -> SolidKey {
    a_prism(
        model,
        [[-4.0, -3.0], [4.0, -3.0], [4.0, 0.0], [-4.0, 0.0]],
        [0.0, 0.0, 0.0],
        10.0,
    )
}

fn a_hook(model: &mut Model) -> SolidKey {
    a_prism(
        model,
        [
            [0.0, 0.0],
            [2.0, 1.0],
            [2.0, -1.0],
            [3.0, -1.0],
            [3.0, 2.0],
            [-1.0, 2.0],
        ],
        [0.0, 0.0, 2.0],
        6.0,
    )
}

/// An outline stood `length` long along `z`, moved by `at`.
fn a_prism<const N: usize>(
    model: &mut Model,
    outline: [[f64; 2]; N],
    [x, y, z]: [f64; 3],
    length: f64,
) -> SolidKey {
    let outline = ProfileBuilder::new(outline.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a simple outline");
    model
        .add_extrusion(
            &outline,
            length,
            &Isometry::translation(Vector::new(x, y, z)).expect("a finite shift"),
        )
        .expect("a positive length")
}
