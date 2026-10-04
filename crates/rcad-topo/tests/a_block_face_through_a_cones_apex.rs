//! A pointed cone and a block whose face `x = 0` runs through the cone's apex.
//! `docs/decisions.md` §283 found every fuse of the two refused on that face,
//! and §347 is the fix: three of the six now answer, right, and the other
//! three get as far as the mesher and are refused there.
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
//! **What still refuses is the mesher, and the mesher is right to.** Every
//! fuse that keeps a piece of the block's face `x = 0` with the apex on its
//! edge, or the cone's skin cut down to the apex, is refused by the check a
//! fuse makes by meshing a curved answer before it writes it — `nest_the_shells`
//! in `fuse/assemble.rs`, measuring each shell. What it is handed is broken.
//! Where only the two rulings leave the apex, `Assembly::what_is_left` reads
//! the apex as a point walked straight through, because two edges leave it,
//! and folds the two rulings into one `Straight` edge between their far ends.
//! With the floor at `z = −2` that edge runs from `(0, 4, 8)` to
//! `(0, −4, 8)`, beside the cap's chord between the same two corners. The
//! face `x = 0` then has a hole of two sides (`TooFewCorners { count: 2 }`
//! when it is meshed first), and the cone's skin is an arc and a straight side
//! that the mesher reads as a loop right round the cone, cuts up to the apex
//! as a cap, and gets one piece with the apex twice at one point of its chart
//! (`CornersCoincide { position: 2 }` when the skin is meshed first). With the
//! floor at `z = 1` it is the tip below the block, cut along the rulings into
//! two pieces of one cone that are kept as two faces. No change to `mesh.rs`
//! answers these: meshing that answer would be measuring a solid that is not
//! the one asked for.
//!
//! **A probe of the fix.** Keeping, in `what_is_left`, every corner that is
//! the apex of a cone an edge there is carried by — not committed, since
//! `fuse/assemble.rs` was outside this row's write set — answered all twelve
//! fuses of the two tests below, one solid each, and the model validated: the
//! unions `931.0165 ± 0.0374` (floor `z = −2`, `864 + 64π/3 ≈ 931.0206`) and
//! `715.1469 ± 0.0571` (floor `z = 1`, `648 + 128π/3 − 511π/24 ≈ 715.1519`),
//! and the block less the cone `796.9835 ± 0.0374` (`864 − 64π/3 ≈ 796.9794`);
//! the other eight as in the tables of §347.3.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_mesh::{CannotTriangulate, NotALoop};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::a_pointed_cone;

const SLOPE: f64 = 0.5;
const HEIGHT: f64 = 8.0;

/// What one fuse comes back as today.
enum Today {
    /// One solid of this volume, which is the right one.
    Answers(f64),
    /// The mesher cannot triangulate a face of the answer: two of its corners
    /// land on one point of the face's chart. The face is a piece of the
    /// cone's skin whose two rulings were folded into one straight side.
    TwoCornersOnOnePointOfTheChart,
    /// A wire of a face of the answer fans into two corners, which is no loop:
    /// the hole in the face `x = 0`, the cap's chord and the two rulings
    /// folded into one straight side between the same two corners.
    AWireOfTwoCorners,
}

/// The intersection and the cone less the block answer, one solid each, the
/// model validates, and the volume is the one worked out by hand. The union
/// and the block less the cone are refused by the mesher.
///
/// The face cuts the cone exactly in half and the block holds the `x > 0` half
/// whole. Half the cone is `π · 4² · 8 / 6 = 64π/3 ≈ 67.0206` and the block is
/// `6 · 12 · 12 = 864`.
#[test]
fn a_block_face_through_the_apex_cuts_the_cone_in_half_and_the_mesher_refuses_the_rest() {
    let half_the_cone = core::f64::consts::PI * SLOPE * SLOPE * HEIGHT.powi(3) / 6.0;
    let block = 6.0 * 12.0 * 12.0;
    for (rule, cone_first, answer, volume, today) in [
        (
            Rule::Union,
            true,
            "the block with the cone's x < 0 half standing out of its face x = 0",
            block + half_the_cone,
            Today::TwoCornersOnOnePointOfTheChart,
        ),
        (
            Rule::Union,
            false,
            "the block with the cone's x < 0 half standing out of its face x = 0",
            block + half_the_cone,
            Today::AWireOfTwoCorners,
        ),
        (
            Rule::Intersection,
            true,
            "the cone's x > 0 half",
            half_the_cone,
            Today::Answers(half_the_cone),
        ),
        (
            Rule::Intersection,
            false,
            "the cone's x > 0 half",
            half_the_cone,
            Today::Answers(half_the_cone),
        ),
        (
            Rule::Difference,
            true,
            "the cone's x < 0 half",
            half_the_cone,
            Today::Answers(half_the_cone),
        ),
        (
            Rule::Difference,
            false,
            "the block with a half-cone groove in its face x = 0, the groove's point at the origin",
            block - half_the_cone,
            Today::AWireOfTwoCorners,
        ),
    ] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let (cone, block) = a_cone_and_a_block(&mut model, 0.0, -2.0);
        let said = fuse(&mut model, cone, block, rule, cone_first);
        let what = format!("{rule:?} (cone first: {cone_first}), {answer} of volume {volume}");
        is_what_it_is_today(&mut model, said, &today, &what);
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
/// and a node is there now as well. Both intersections and both differences
/// answer, right; the unions are refused by the mesher.
///
/// §283 pinned all six refused one stage after the arrangement, for want of a
/// point inside a piece of the face. That was the missing node and the
/// rulings' direction together: with the node alone the cone's skin refuses in
/// the arrangement, and with both the fuses get through selection.
///
/// The cone below `z = 1` is a tip of radius one half, `π/12`, so the block
/// holds `64π/3 − π/24 = 511π/24` of the cone, and the block is
/// `6 · 12 · 9 = 648`.
#[test]
fn the_same_block_with_the_apex_just_below_its_face_answers_all_but_the_unions() {
    let pi = core::f64::consts::PI;
    let (block, cone) = (6.0 * 12.0 * 9.0, 128.0 * pi / 3.0);
    let shared = 511.0 * pi / 24.0;
    for (rule, cone_first) in six_fuses() {
        let (volume, today) = match (rule, cone_first) {
            (Rule::Union, _) => (block + cone - shared, Today::TwoCornersOnOnePointOfTheChart),
            (Rule::Intersection, _) => (shared, Today::Answers(shared)),
            (Rule::Difference, true) => (cone - shared, Today::Answers(cone - shared)),
            (Rule::Difference, false) => (block - shared, Today::Answers(block - shared)),
            (other, _) => panic!("{other:?} is not one of the six"),
        };
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let (cone, block) = a_cone_and_a_block(&mut model, 0.0, 1.0);
        let said = fuse(&mut model, cone, block, rule, cone_first);
        let what = format!("{rule:?} (cone first: {cone_first}), one solid of volume {volume}");
        is_what_it_is_today(&mut model, said, &today, &what);
    }
}

/// Assert that `said` is what `today` says it is: the answer, measured and
/// validated, or the mesher's refusal by name. `what` is the right answer,
/// which is what every message carries.
fn is_what_it_is_today(
    model: &mut Model,
    said: rcad_topo::Result<Vec<SolidKey>>,
    today: &Today,
    what: &str,
) {
    let policy = Policy::certified();
    match today {
        Today::Answers(volume) => {
            let solids = said.unwrap_or_else(|why| panic!("{what}: refused: {why:?}"));
            let [solid] = solids[..] else {
                panic!("{what}: {} solids", solids.len())
            };
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: the answer does not validate: {why:?}"));
            let measured = model
                .solid(solid)
                .expect("a solid the fuse made")
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
        Today::TwoCornersOnOnePointOfTheChart => assert!(
            matches!(
                said,
                Err(Error::CannotMeshAFace {
                    why: CannotTriangulate::CornersCoincide { .. },
                    ..
                })
            ),
            "{what}: today the mesher finds two corners of a face on one point of its chart. \
             Got {said:?}"
        ),
        Today::AWireOfTwoCorners => assert!(
            matches!(
                said,
                Err(Error::AWireDoesNotFanIntoALoopToMesh {
                    why: NotALoop::TooFewCorners { count: 2 },
                    ..
                })
            ),
            "{what}: today a wire of a face fans into two corners. Got {said:?}"
        ),
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
