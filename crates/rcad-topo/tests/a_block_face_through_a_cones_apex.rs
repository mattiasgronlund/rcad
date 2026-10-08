//! A pointed cone and a block whose face `x = 0` runs through the cone's apex.
//! `docs/decisions.md` §283 found every fuse of the two refused on that face,
//! and §347 is the fix: every fuse answers, right.
//!
//! The cone stands on its apex at the origin, slope one half and eight tall,
//! so its cap is a disc of radius four at `z = 8`. The block is
//! `[0, 6] × [−6, 6] × [−2, 10]`: it holds the cone's `x > 0` half whole, and
//! the only thing that cuts the cone is the block's face on `x = 0`.
//!
//! **What that face holds.** The plane `x = 0` cuts the cone's double surface
//! in the two rulings `y = z/2` and `y = −z/2`, which cross at the apex, and
//! the cap's plane in the line `z = 8`. On the face those three lines leave
//! seven regions: the triangle inside the cone, `(0, 0)`, `(±4, 8)` in
//! `(y, z)`, of area 32; the strip above the cap between the rulings, 18; the
//! small triangle down in the other nappe, `(0, 0)`, `(±1, −2)`, 2; and a pair
//! of 43 and a pair of 3 on either side. They add up to the face's 144. Four of
//! them meet at the apex and touch nowhere else.
//!
//! **What changed.** `arrange.rs` used to put no node where two curves of one
//! pair cross, so the two rulings crossed at the apex between nodes and the
//! walks round the face came out as a bow tie. There is a node there now. A
//! ruling runs along its own line rather than along `∇S × ∇Q`, which is nothing
//! at the apex and turns round there. On the cone's own face the apex is one
//! node per nappe, since the rulings run on through it into the nappe the
//! solid does not have.
//!
//! **And the apex stays a corner** (§347.4, Q862). Where only the two rulings
//! leave the apex, `Assembly::what_is_left` in `fuse/assemble.rs` read the
//! apex as a point walked straight through, because two edges leave it, and
//! folded the two rulings into one `Straight` edge between their far ends.
//! With the floor at `z = −2` that edge ran from `(0, 4, 8)` to `(0, −4, 8)`,
//! beside the cap's chord between the same two corners, and the mesher that
//! measures a curved answer before it is written was right to refuse what it
//! was handed: the unions, and the block less the cone, as `CornersCoincide`
//! or `TooFewCorners { count: 2 }`. A corner that is the apex of a cone an
//! edge there is carried by, bit for bit, now stays a corner.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::a_pointed_cone;

const SLOPE: f64 = 0.5;
const HEIGHT: f64 = 8.0;

/// Every fuse answers one solid of the volume worked out by hand, with the
/// faces it should have, and the model validates.
///
/// The face cuts the cone exactly in half and the block holds the `x > 0` half
/// whole. Half the cone is `π · 4² · 8 / 6 = 64π/3 ≈ 67.0206` and the block is
/// `6 · 12 · 12 = 864`. A half cone is its half of the skin, its half of the
/// cap, and the triangle on `x = 0`. The block with the other half standing
/// out of it, or with this half cut out of it, is the block's six faces, the
/// face `x = 0` with a triangular hole whose point is the apex, and the half
/// cone's skin and cap.
#[test]
fn a_block_face_through_the_apex_cuts_the_cone_in_half_and_every_fuse_answers() {
    let half_the_cone = core::f64::consts::PI * SLOPE * SLOPE * HEIGHT.powi(3) / 6.0;
    let block = 6.0 * 12.0 * 12.0;
    for (rule, cone_first, answer, volume, faces) in [
        (
            Rule::Union,
            true,
            "the block with the cone's x < 0 half standing out of its face x = 0",
            block + half_the_cone,
            8,
        ),
        (
            Rule::Union,
            false,
            "the block with the cone's x < 0 half standing out of its face x = 0",
            block + half_the_cone,
            8,
        ),
        (
            Rule::Intersection,
            true,
            "the cone's x > 0 half",
            half_the_cone,
            3,
        ),
        (
            Rule::Intersection,
            false,
            "the cone's x > 0 half",
            half_the_cone,
            3,
        ),
        (
            Rule::Difference,
            true,
            "the cone's x < 0 half",
            half_the_cone,
            3,
        ),
        (
            Rule::Difference,
            false,
            "the block with a half-cone groove in its face x = 0, the groove's point at the origin",
            block - half_the_cone,
            8,
        ),
    ] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let (cone, block) = a_cone_and_a_block(&mut model, 0.0, -2.0);
        let said = fuse(&mut model, cone, block, rule, cone_first);
        let what = format!(
            "{rule:?} (cone first: {cone_first}), {answer} of volume {volume} and {faces} faces"
        );
        answers(&mut model, said, volume, faces, &what);
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
                panic!(
                    "{rule:?} (cone first: {cone_first}): the answer does not validate: {error:?}"
                )
            });
    }
}

/// The block's floor raised from `z = −2` to `z = 1`, so the apex is off the
/// face, just below it. The two rulings still cross at the apex, off the face,
/// and a node is there as well. Every fuse answers, right.
///
/// §283 pinned all six refused one stage after the arrangement, for want of a
/// point inside a piece of the face. That was the missing node and the
/// rulings' direction together: with the node alone the cone's skin refuses in
/// the arrangement, and with both the fuses get through selection. The unions
/// were then refused by the mesher, for the apex folded away below the block.
///
/// The cone below `z = 1` is a tip of radius one half, `π/12`, so the block
/// holds `64π/3 − π/24 = 511π/24` of the cone, and the block is
/// `6 · 12 · 9 = 648`. What the block holds is four faces: the cone's skin,
/// its cap and the trapezium on `x = 0`, each cut to `x > 0`, and the half
/// disc on the floor. The block less the cone is eight: the block's six, the
/// floor and the face `x = 0` each notched where the cone runs through them,
/// and the skin and the cap the other way round. The cone less the block is
/// four: the skin outside the block, the cap's `x < 0` half, and the
/// trapezium on `x = 0` and the half disc on the floor turned round. The union
/// is eight: the block's six with the floor and the face `x = 0` notched, the
/// skin outside the block and the cap's `x < 0` half.
///
/// **The skin outside the block is one face** (§347.4, Q989). The cut along
/// the rulings below the floor and round the cone at `z = 1` for `x < 0`
/// leaves three pieces of the one cone: the tip either side of `x = 0`, and
/// the `x < 0` half above the floor. Joined, they are one face whose one ring,
/// of four sides, goes right round the apex, and
/// `Assembly::edges_a_curved_face_keeps` in `fuse/assemble.rs` left them
/// apart, since it only looked for a seam across a band between two rings:
/// ten faces for the unions and six for the cone less the block. One ring
/// right round a cone and nothing else is a cap round its apex, which needs
/// no seam, and the mesher cuts it up to the apex.
#[test]
fn the_same_block_with_the_apex_just_below_its_face_answers_every_fuse() {
    let pi = core::f64::consts::PI;
    let (block, cone) = (6.0 * 12.0 * 9.0, 128.0 * pi / 3.0);
    let shared = 511.0 * pi / 24.0;
    for (rule, cone_first) in six_fuses() {
        let (volume, faces) = match (rule, cone_first) {
            (Rule::Union, _) => (block + cone - shared, 8),
            (Rule::Intersection, _) => (shared, 4),
            (Rule::Difference, true) => (cone - shared, 4),
            (Rule::Difference, false) => (block - shared, 8),
            (other, _) => panic!("{other:?} is not one of the six"),
        };
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let (cone, block) = a_cone_and_a_block(&mut model, 0.0, 1.0);
        let said = fuse(&mut model, cone, block, rule, cone_first);
        let what = format!(
            "{rule:?} (cone first: {cone_first}), one solid of volume {volume} and {faces} faces"
        );
        answers(&mut model, said, volume, faces, &what);
    }
}

/// Assert that `said` is one solid with `faces` faces, that the model
/// validates, and that the solid measures `volume` to within the mesh's own
/// error. `what` is the right answer, which is what every message carries.
fn answers(
    model: &mut Model,
    said: rcad_topo::Result<Vec<SolidKey>>,
    volume: f64,
    faces: usize,
    what: &str,
) {
    let policy = Policy::certified();
    let solids = said.unwrap_or_else(|why| panic!("{what}: refused: {why:?}"));
    let [solid] = solids[..] else {
        panic!("{what}: {} solids", solids.len())
    };
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: the answer does not validate: {why:?}"));
    let view = model.solid(solid).expect("a solid the fuse made");
    assert_eq!(view.faces().count(), faces, "{what}: faces");
    let measured = view
        .measure(
            Deflection::of(0.001).expect("a positive length"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("the answer measures");
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{what}: measured {} and the mesh may miss by {}",
        measured.volume(),
        measured.volume_error()
    );
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
