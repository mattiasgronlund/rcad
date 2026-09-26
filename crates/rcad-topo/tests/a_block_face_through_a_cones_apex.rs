//! A pointed cone and a block whose face `x = 0` runs through the cone's apex:
//! every fuse of the two refuses `CannotTellTheCellsOfAFaceApart` on that
//! face, and every answer exists. `docs/decisions.md` §283, found in §252.8.
//!
//! The cone stands on its apex at the origin, slope one half and eight tall,
//! so its cap is a disc of radius four at `z = 8`. The block is
//! `[0, 6] × [−6, 6] × [−2, 10]`: it holds the cone's `x > 0` half whole, and
//! the only thing that cuts the cone is the block's face on `x = 0`.
//!
//! **What that face really holds.** The plane `x = 0` cuts the cone's double
//! surface in the two rulings `y = z/2` and `y = −z/2`, which cross at the
//! apex, and the cap's plane in the line `z = 8`. On the face those three lines
//! leave seven regions: the triangle inside the cone, `(0, 0)`, `(±4, 8)` in
//! `(y, z)`, of area 32; the strip above the cap between the rulings, 18; the
//! small triangle down in the other nappe, `(0, 0)`, `(±1, −2)`, 2; and a pair
//! of 43 and a pair of 3 on either side. They add up to the face's 144. Four of
//! them meet at the apex and touch nowhere else.
//!
//! **Why they cannot be told apart.** `arrange.rs`'s `place_the_meets` skips
//! every pair of crossings with one partner, so the two rulings, which have the
//! cone as partner both, get no node where they cross. Each ruling is drawn as
//! one arc from `z = −2` to `z = 8` straight through the apex, the two arcs
//! cross between nodes, and the walks round a drawing that is not flat come out
//! wrong: one of them goes round both the triangle inside the cone and the one
//! in the other nappe, as a bow tie, and another goes along both sides of six
//! of the face's own edges. `spread_the_winding` finds that walk wound once
//! and not at all, and refuses.

use rcad_geom::{Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, FaceKey, Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::a_pointed_cone;

const SLOPE: f64 = 0.5;
const HEIGHT: f64 = 8.0;

/// Each of the six fuses refuses on the block's face `x = 0`, by name.
///
/// Every answer is one solid, and each is worked out by hand from two halves of
/// the cone, since the face cuts the cone exactly in half and the block holds
/// the `x > 0` half whole. Half the cone is `π · 4² · 8 / 6 = 64π/3`
/// `≈ 67.0206` and the block is `6 · 12 · 12 = 864`. The message of each
/// assertion is the answer the refusal stands in for.
#[test]
fn every_fuse_of_a_block_whose_face_runs_through_the_apex_is_refused_by_name() {
    let half_the_cone = core::f64::consts::PI * SLOPE * SLOPE * HEIGHT.powi(3) / 6.0;
    let block = 6.0 * 12.0 * 12.0;
    for (rule, cone_first, answer, volume) in [
        (
            Rule::Union,
            true,
            "the block with the cone's x < 0 half standing out of its face x = 0",
            block + half_the_cone,
        ),
        (
            Rule::Union,
            false,
            "the block with the cone's x < 0 half standing out of its face x = 0",
            block + half_the_cone,
        ),
        (
            Rule::Intersection,
            true,
            "the cone's x > 0 half",
            half_the_cone,
        ),
        (
            Rule::Intersection,
            false,
            "the cone's x > 0 half",
            half_the_cone,
        ),
        (
            Rule::Difference,
            true,
            "the cone's x < 0 half",
            half_the_cone,
        ),
        (
            Rule::Difference,
            false,
            "the block with a half-cone groove in its face x = 0, the groove's point at the origin",
            block - half_the_cone,
        ),
    ] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let (cone, block) = a_cone_and_a_block(&mut model, 0.0, -2.0);
        let through_the_apex = the_face_on_x_is_nought(&model, block);
        let refused = fuse(&mut model, cone, block, rule, cone_first);
        assert!(
            matches!(
                refused,
                Err(Error::CannotTellTheCellsOfAFaceApart { face }) if face == through_the_apex
            ),
            "{rule:?} (cone first: {cone_first}) should be one solid, {answer}, of volume \
             {volume}; today it is refused on the block's face x = 0. Got {refused:?}",
        );
    }
}

/// The control: the same block one unit off the apex, so its face `x = 1`
/// cuts the cone in a hyperbola and no two curves of one pair cross on it.
/// Every fuse answers, one solid each, and the model validates.
#[test]
fn the_same_block_one_unit_off_the_apex_answers_every_fuse() {
    let policy = Policy::certified();
    for (rule, cone_first) in six_fuses() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let (cone, block) = a_cone_and_a_block(&mut model, 1.0, -2.0);
        let answer = fuse(&mut model, cone, block, rule, cone_first).unwrap_or_else(|error| {
            panic!("{rule:?} (cone first: {cone_first}) was refused: {error:?}")
        });
        assert_eq!(
            answer.len(),
            1,
            "{rule:?} (cone first: {cone_first}): one solid"
        );
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|error| {
                panic!("{rule:?} (cone first: {cone_first}): the answer does not validate: {error:?}")
            });
    }
}

/// The block's floor raised from `z = −2` to `z = 1`, so the apex is off the
/// face, just below it. The two rulings still cross at the apex with no node
/// there, and all six fuses now get past the arrangement and refuse one stage
/// later, for want of a point inside a piece of that face. The record names
/// `(0, ∓6, −12)` and `(0, ±0.5, 1)`: two stretches, one along each ruling,
/// that cross at the apex.
///
/// Every answer is one solid again. The cone below `z = 1` is a tip of radius
/// one half, `π/12`, so the block holds `64π/3 − π/24 = 511π/24` of the cone,
/// and the block is `6 · 12 · 9 = 648`.
#[test]
fn the_same_block_with_the_apex_just_below_its_face_is_refused_a_stage_later() {
    let pi = core::f64::consts::PI;
    let (block, cone) = (6.0 * 12.0 * 9.0, 128.0 * pi / 3.0);
    let shared = 511.0 * pi / 24.0;
    for (rule, cone_first) in six_fuses() {
        let volume = match (rule, cone_first) {
            (Rule::Union, _) => block + cone - shared,
            (Rule::Intersection, _) => shared,
            (Rule::Difference, true) => cone - shared,
            (Rule::Difference, false) => block - shared,
            (other, _) => panic!("{other:?} is not one of the six"),
        };
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let (cone, block) = a_cone_and_a_block(&mut model, 0.0, 1.0);
        let refused = fuse(&mut model, cone, block, rule, cone_first);
        let why = match &refused {
            Err(Error::Undecidable { records }) => records
                .records()
                .last()
                .map(|record| record.predicate().to_owned()),
            _ => None,
        };
        assert_eq!(
            why.as_deref(),
            Some("a point inside a piece of a face"),
            "{rule:?} (cone first: {cone_first}) should be one solid of volume {volume}; \
             today it is refused for want of a point inside a piece. Got {refused:?}",
        );
    }
}

fn six_fuses() -> [(Rule, bool); 6] {
    [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Intersection, true),
        (Rule::Intersection, false),
        (Rule::Difference, true),
        (Rule::Difference, false),
    ]
}

/// The cone, then the block `[x, x + 6] × [−6, 6] × [floor, 10]`, in that
/// order, which is what makes the block's face on `x = 0` the model's face
/// `8v1` as §252.8 found it.
fn a_cone_and_a_block(model: &mut Model, x: f64, floor: f64) -> (SolidKey, SolidKey) {
    let cone = a_pointed_cone(model, SLOPE, HEIGHT);
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [6.0, 0.0], [6.0, 12.0], [0.0, 12.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    let placed = Isometry::translation(Vector::new(x, -6.0, floor)).expect("a finite move");
    let block = model
        .add_extrusion(&rectangle, 10.0 - floor, &placed)
        .expect("a rectangle extrudes");
    (cone, block)
}

fn the_face_on_x_is_nought(model: &Model, block: SolidKey) -> FaceKey {
    model
        .solid(block)
        .expect("this model made the key")
        .faces()
        .find(|face| {
            face.surface().as_plane().is_some_and(|plane| {
                plane.normal().components() == [-1.0, 0.0, 0.0] && plane.offset() == 0.0
            })
        })
        .expect("the block has a face on x = 0")
        .key()
}

fn fuse(
    model: &mut Model,
    cone: SolidKey,
    block: SolidKey,
    rule: Rule,
    cone_first: bool,
) -> rcad_topo::Result<Vec<SolidKey>> {
    let (arguments, tools) = if cone_first {
        ([cone], [block])
    } else {
        ([block], [cone])
    };
    model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().to_vec())
}
