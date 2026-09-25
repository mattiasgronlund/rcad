//! A face with one tool pressed against it from each side is inside the tools,
//! whichever tool is named first.
//!
//! Three cubes of side two, every corner on the integer grid:
//!
//! - `s0` is `[0, 2]³`;
//! - `s1` is `[2, 4] × [0, 2] × [0, 2]`, touching `s0` on `x = 2` from outside;
//! - `s2` is `[0, 2] × [1, 3] × [0, 2]`, overlapping `s0` in its top half along
//!   `y`, with its own `x = 2` face flush with `s0`'s.
//!
//! The part of `s0`'s `x = 2` face with `y ∈ [1, 2]` lies on `s1`, which faces
//! it, and on `s2`, which faces the same way it does. So the tools' union has
//! material on both sides of that piece and it is inside them. A fuse used to
//! take only the first tool the piece lay on, and so read it as "against `s1`"
//! or "flush with `s2`" by the order the tools were named in: `s0 \ (s1 ∪ s2)`
//! was refused `ResultIsNotManifold { count: 3 }` and `s0 \ (s2 ∪ s1)` answered,
//! and `s0 ∩ (s1 ∪ s2)` was refused `ResultIsNotManifold { count: 1 }`.

#[path = "fuzzing/comparison.rs"]
mod comparison;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

use comparison::{Answer, answer_of, same_answer};

/// `s0 \ (s1 ∪ s2)` is `[0, 2] × [0, 1] × [0, 2]`, volume four, both ways
/// round.
#[test]
fn a_cube_less_two_it_touches_answers_the_same_whichever_tool_comes_first() {
    let (mut model, [s0, s1, s2]) = three_cubes();
    let (s1_first, volume) = fused(&mut model, &[s0], &[s1, s2], Rule::Difference);
    assert_eq!(volume, 4.0);

    let (mut model, [s0, s1, s2]) = three_cubes();
    let (s2_first, volume) = fused(&mut model, &[s0], &[s2, s1], Rule::Difference);
    assert_eq!(volume, 4.0);

    assert_eq!(
        same_answer(
            &s2_first,
            &s1_first,
            Rule::Difference,
            "naming the tools the other way round"
        ),
        Ok(())
    );
}

/// `s1` meets `s0` in nothing but a face, so `s0 ∩ (s1 ∪ s2)` is `s0 ∩ s2`,
/// `[0, 2] × [1, 2] × [0, 2]`, volume four.
#[test]
fn a_cube_shared_with_two_it_touches_is_its_share_of_the_one_it_overlaps() {
    let (mut model, [s0, _, s2]) = three_cubes();
    let (expected, volume) = fused(&mut model, &[s0], &[s2], Rule::Intersection);
    assert_eq!(volume, 4.0);

    for tools_in_order in [[1, 2], [2, 1]] {
        let (mut model, cubes) = three_cubes();
        let tools = tools_in_order.map(|which| cubes[which]);
        let (answer, volume) = fused(&mut model, &[cubes[0]], &tools, Rule::Intersection);
        assert_eq!(volume, 4.0);
        assert_eq!(
            same_answer(
                &expected,
                &answer,
                Rule::Intersection,
                "adding a tool that only touches"
            ),
            Ok(()),
            "tools named in the order {tools_in_order:?}"
        );
    }
}

fn three_cubes() -> (Model, [SolidKey; 3]) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let s0 = cube_at(&mut model, Vector::new(0.0, 0.0, 0.0));
    let s1 = cube_at(&mut model, Vector::new(2.0, 0.0, 0.0));
    let s2 = cube_at(&mut model, Vector::new(0.0, 1.0, 0.0));
    (model, [s0, s1, s2])
}

/// A cube of side two with its low corner at `shift`.
fn cube_at(model: &mut Model, shift: Vector) -> SolidKey {
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(cube, &Isometry::translation(shift).expect("a finite shift"))
        .expect("this model made the key");
    cube
}

/// The answer's geometry and its volume, once it has been checked to validate.
fn fused(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> (Answer, f64) {
    let solids = model
        .fuse(
            arguments,
            tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("three cubes on the integer grid leave nothing to round")
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the answer is a model this kernel calls valid");
    (answer_of(model, &solids), volume_of(model, &solids))
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
                .expect("a box measures")
                .volume()
        })
        .sum()
}
