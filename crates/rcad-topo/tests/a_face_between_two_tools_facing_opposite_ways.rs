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
//!
//! A curved piece is asked about coincidence on a branch of its own, before any
//! ray, so it has cases of its own: a cube of side four with a cavity a unit
//! ball fills exactly, that ball, and a second unit ball built in the same
//! place. The second ball's skin lies on the cavity's wall, which faces it, and
//! on the first ball, which faces the same way — inside the pair, as the flat
//! piece is inside `s1 ∪ s2`.

#[path = "fuzzing/comparison.rs"]
mod comparison;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Measurement, Model, Rule, SolidKey};

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

/// The cavity and the first ball together fill the cube, so nothing of the
/// second ball is outside them.
#[test]
fn a_ball_less_the_cavity_it_fills_and_a_ball_in_its_place_is_nothing() {
    for tools_in_order in [[0, 1], [1, 0]] {
        let (mut model, [cavity, first, ball]) = a_cavity_and_two_balls_that_fill_it();
        let tools = tools_in_order.map(|which| [cavity, first][which]);
        let solids = fused_and_validated(&mut model, &[ball], &tools, Rule::Difference);
        assert!(
            solids.is_empty(),
            "tools named in the order {tools_in_order:?}: {} solids",
            solids.len()
        );
    }
}

/// All of the second ball is inside the cavity and the first ball together.
#[test]
fn a_ball_shared_with_the_cavity_it_fills_and_a_ball_in_its_place_is_the_ball() {
    for tools_in_order in [[0, 1], [1, 0]] {
        let (mut model, [cavity, first, ball]) = a_cavity_and_two_balls_that_fill_it();
        let tools = tools_in_order.map(|which| [cavity, first][which]);
        let solids = fused_and_validated(&mut model, &[ball], &tools, Rule::Intersection);
        assert_eq!(
            solids.len(),
            1,
            "tools named in the order {tools_in_order:?}"
        );
        assert_measures_alike(&model, solids[0], ball);
    }
}

/// The cavity, the first ball and the second ball make the cube whole, however
/// they are split into arguments and tools and in whatever order.
#[test]
fn a_cavity_united_with_the_ball_that_fills_it_and_a_ball_in_its_place_is_the_whole_cube() {
    let splits: [(&[usize], &[usize]); 5] = [
        (&[2], &[0, 1]),
        (&[2], &[1, 0]),
        (&[0, 1, 2], &[]),
        (&[2, 0, 1], &[]),
        (&[1, 2, 0], &[]),
    ];
    for (arguments, tools) in splits {
        let (mut model, solids) = a_cavity_and_two_balls_that_fill_it();
        let answer = fused_and_validated(
            &mut model,
            &arguments
                .iter()
                .map(|&which| solids[which])
                .collect::<Vec<_>>(),
            &tools.iter().map(|&which| solids[which]).collect::<Vec<_>>(),
            Rule::Union,
        );
        assert_eq!(answer.len(), 1, "{arguments:?} with {tools:?}");
        assert_eq!(
            measured(&model, answer[0]).volume(),
            64.0,
            "{arguments:?} with {tools:?}: the whole cube is flat, so its volume is exact"
        );
    }
}

/// Taking the second ball out of the cavity and the first ball leaves the
/// cavity.
#[test]
fn the_cavity_and_the_ball_that_fills_it_less_a_ball_in_its_place_is_the_cavity() {
    for arguments_in_order in [[0, 1], [1, 0]] {
        let (mut model, [cavity, first, ball]) = a_cavity_and_two_balls_that_fill_it();
        let arguments = arguments_in_order.map(|which| [cavity, first][which]);
        let solids = fused_and_validated(&mut model, &arguments, &[ball], Rule::Difference);
        assert_eq!(
            solids.len(),
            1,
            "arguments named in the order {arguments_in_order:?}"
        );
        assert_measures_alike(&model, solids[0], cavity);
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

/// `[cavity, first, ball]`: a cube of side four centred on the origin less a
/// unit ball there, then two more unit balls in the same place.
fn a_cavity_and_two_balls_that_fill_it() -> (Model, [SolidKey; 3]) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cube = cube_of_side_four_centred_on_the_origin(&mut model);
    let hole = a_unit_ball(&mut model);
    let cavity = fused_and_validated(&mut model, &[cube], &[hole], Rule::Difference);
    assert_eq!(cavity.len(), 1, "a ball inside a cube leaves one solid");
    let first = a_unit_ball(&mut model);
    let ball = a_unit_ball(&mut model);
    (model, [cavity[0], first, ball])
}

fn cube_of_side_four_centred_on_the_origin(model: &mut Model) -> SolidKey {
    let cube = model
        .add_cube(4.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(-2.0, -2.0, -2.0)).expect("a finite shift"),
        )
        .expect("this model made the key");
    cube
}

fn a_unit_ball(model: &mut Model) -> SolidKey {
    model
        .add_ball(1.0, &Isometry::identity())
        .expect("a unit ball")
}

/// The answer's solids, once the model holding them has been checked to
/// validate: a wrong answer here used to validate, so this alone is not enough.
fn fused_and_validated(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> Vec<SolidKey> {
    let solids = model
        .fuse(
            arguments,
            tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a ball and a cavity it fills are answered")
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the answer is a model this kernel calls valid");
    solids
}

/// Two solids that should be the same shape measure within their two published
/// errors of each other: each error bounds how far its volume is from the truth.
fn assert_measures_alike(model: &Model, answer: SolidKey, expected: SolidKey) {
    let answer = measured(model, answer);
    let expected = measured(model, expected);
    assert!(
        (answer.volume() - expected.volume()).abs()
            <= answer.volume_error() + expected.volume_error(),
        "volume {} ± {} against {} ± {}",
        answer.volume(),
        answer.volume_error(),
        expected.volume(),
        expected.volume_error()
    );
}

fn measured(model: &Model, solid: SolidKey) -> Measurement {
    model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a hundredth of a millimetre is a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a ball and a cube with a cavity measure")
}
