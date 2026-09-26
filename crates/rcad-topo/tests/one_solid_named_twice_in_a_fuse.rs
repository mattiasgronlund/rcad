//! One solid named twice in one fuse answers as if it had been named once.
//!
//! A fuse used to work out which of its operands a piece came from by looking
//! the piece's face up in a map from face to solid, and a key given twice put
//! both copies' faces under the later copy. Every piece then lay on the earlier
//! copy, and "keep only the one that comes first" dropped every one of them:
//! `a ∪ a`, `a ∩ a` and `a` named twice among the arguments each came back with
//! no solids at all, where `a` on its own came back as itself.

#[path = "fuzzing/comparison.rs"]
mod comparison;

use rcad_geom::{Deflection, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule, SolidKey};

use comparison::{Answer, answer_of, same_answer};

#[test]
fn a_cube_united_with_itself_is_the_cube() {
    let (mut model, cube) = a_cube();
    let (answer, volume) = fused(&mut model, &[cube], &[cube], Rule::Union);

    assert_eq!(
        same_answer(&the_cube_alone(), &answer, Rule::Union, "naming it twice"),
        Ok(())
    );
    assert_eq!(volume, 8.0);
}

#[test]
fn a_cube_shared_with_itself_is_the_cube() {
    let (mut model, cube) = a_cube();
    let (answer, volume) = fused(&mut model, &[cube], &[cube], Rule::Intersection);

    assert_eq!(
        same_answer(
            &the_cube_alone(),
            &answer,
            Rule::Intersection,
            "naming it twice"
        ),
        Ok(())
    );
    assert_eq!(volume, 8.0);
}

/// This one answered before the fix as well, and for the wrong reason: every
/// piece was the tool's, so every piece was dropped. It is here so that crediting
/// each copy with its own pieces cannot start keeping one of them.
#[test]
fn a_cube_less_itself_is_nothing() {
    let (mut model, cube) = a_cube();
    let (answer, _) = fused(&mut model, &[cube], &[cube], Rule::Difference);

    assert!(
        answer.solids.is_empty(),
        "a cube less itself left {} solids",
        answer.solids.len()
    );
}

#[test]
fn a_cube_named_twice_among_the_arguments_is_the_cube() {
    let (mut model, cube) = a_cube();
    let (answer, volume) = fused(&mut model, &[cube, cube], &[], Rule::Union);

    assert_eq!(
        same_answer(&the_cube_alone(), &answer, Rule::Union, "naming it twice"),
        Ok(())
    );
    assert_eq!(volume, 8.0);
}

/// `fuse([a], [], Union)`: the cube, as the fuse hands back a solid it had no
/// reason to cut.
fn the_cube_alone() -> Answer {
    let (mut model, cube) = a_cube();
    let (answer, volume) = fused(&mut model, &[cube], &[], Rule::Union);
    assert_eq!(answer.solids.len(), 1, "a cube on its own is one solid");
    assert_eq!(volume, 8.0);
    answer
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
        .expect("a cube and itself are on the integer grid, so nothing here rounds")
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the answer is a model this kernel calls valid");
    (answer_of(model, &solids), volume_of(model, &solids))
}

/// A cube of side two, volume eight.
fn a_cube() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    (model, cube)
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
                .expect("a cube measures")
                .volume()
        })
        .sum()
}
