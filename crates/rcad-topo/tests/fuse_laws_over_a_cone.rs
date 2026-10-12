//! Associativity, idempotence and absorption, run over solids with a cone face.
//!
//! §221.5's third bullet: nothing had run these three laws over a cone.
//! `fuses_that_must_agree.rs` runs commutativity over a frustum up through a
//! block, and `what_the_answered_block_fuses_actually_say.rs` checks inclusion,
//! exclusion and the two ways round by volume — but neither asks whether
//! `(A ∪ B) ∪ C` is `A ∪ (B ∪ C)`, whether `A ∪ A` is `A`, or whether
//! `A ∪ (A ∩ B)` is `A`, with a cone in any of them.
//!
//! # The shapes
//!
//! Three families, each built the way the file that owns it builds it:
//!
//! - frustums up through a block's floor, from §49.6's grid
//!   (`pairs::FrustumAndBlock`);
//! - the sixteen frustum-and-block rows of
//!   `what_the_answered_block_fuses_actually_say.rs`, drawn with the same seed;
//! - a cone that ends in a point (`cones::a_pointed_cone`) through a block's
//!   floor, out through its roof, and swallowed whole by it.
//!
//! # What "the same answer" means here
//!
//! Two answers agree when they have the same number of solids, the same corners
//! **bit for bit**, and volumes within the sum of `measure`'s own bounds.
//!
//! The corners are compared as a set and not as a list, for the reason
//! `what_the_answered_block_fuses_actually_say.rs` gives at length: the fuse
//! drops the seam of a curved wall it never touched, so a frustum goes in with
//! its two seam corners listed twice each and comes out with them listed once.
//! Same points, same solid.
//!
//! The volumes are compared within a bound rather than bit for bit because two
//! ways of reaching the same solid can mesh its cone from different places — a
//! frustum with its seam and one without are the same solid and different
//! meshes. How many compared volumes came out bit-identical anyway is printed.
//!
//! # A refusal is counted, not failed
//!
//! A fuse that refuses on either side is tallied by name and printed, and is not
//! a failure — except where the two sides of one law disagree about whether
//! there is an answer at all. That is an asymmetry and it fails: whether a
//! union of three solids exists cannot depend on which two were united first.
//!
//! **Most of what is asked here is refused**, and the counts are printed so that
//! stays visible. Every law but associativity makes the two sides share faces —
//! `A ∩ B` shares its skin with `A` — and two copies of one curved face is
//! §36.16's row, which `fuses_that_must_agree.rs` pins for a bar. So each test
//! asserts a floor on what it compared, measured on the day it was written, and
//! a drop in it is a regression rather than a draw (§19.1).
//!
//! The three `#[ignore]`d tests at the bottom are the asymmetries this found
//! that still stand; the far cube's went when pieces of one cone were joined
//! (§224.2), and the frustum-beside-a-block's went too, un-diagnosed, by Q1102.

mod cones;
mod pairs;

use std::collections::BTreeMap;

use rcad_geom::{Deflection, Isometry, Scale, Surface, Vector};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Model, OnAFace, ProfileBuilder, Rule, SolidKey};

use cones::{a_frustum, a_pointed_cone};
use pairs::{FrustumAndBlock, WORKING_SIZE};

/// What two copies of one cone face come to today, whatever the law asks.
const TWO_COPIES_OF_ONE_CURVED_FACE: &str =
    "Undecidable on a point in a piece that every solid could place";

/// `A ∪ A'` and `A ∩ A'`, where `A'` is a frustum or a pointed cone built again
/// from the same numbers.
///
/// **Every one of them is refused, on two copies of one curved face**, the same
/// refusal `fuses_that_must_agree.rs` pins for `A \ A` over a bar. A pointed cone
/// has no seam, and a ball — which also has none — answers all three questions
/// there, so this says §36.16's row is wider than "a surface that closes round a
/// seam": a seamless cone refuses too.
///
/// The law is still checked on anything that answers, and a refusal of any other
/// kind fails, so the day these answer is the day the law is asserted.
#[test]
fn a_cone_with_a_copy_of_itself_is_refused_rather_than_united_or_shared() {
    let report = idempotence_of_the_cone_alone(&every_pair());
    for name in report.refused.keys() {
        assert_eq!(
            name, TWO_COPIES_OF_ONE_CURVED_FACE,
            "a cone and a copy of it refused for a new reason"
        );
    }
    report.print_and_check("idempotence, a cone and a copy of it", 0);
}

/// `U ∪ U'` and `U ∩ U'` are both `U`, where `U` is a block united with a cone
/// and `U'` is the same union made again in the same model.
///
/// What answers is the pointed cone the block swallows, where `U` is the block
/// and has no cone face left; every `U` that keeps one refuses.
#[test]
fn a_block_and_cone_united_with_a_copy_of_that_union_is_that_union() {
    idempotence_of_a_union_with_a_cone(&every_pair())
        .print_and_check("idempotence, a block-and-cone union and a copy of it", 2);
}

/// `A ∪ (A ∩ B)` is `A`, with `A` the block.
#[test]
fn a_block_united_with_what_it_shares_with_a_cone_is_the_block() {
    absorption(
        &every_pair(),
        Which::TheBlock,
        Rule::Intersection,
        Rule::Union,
    )
    .print_and_check("absorption A ∪ (A ∩ B), A the block", 5);
}

/// `A ∪ (A ∩ B)` is `A`, with `A` the cone, over every pair but the rows of
/// [`UNITED_WITH_THE_SHARE_IN_PIECES`], which
/// [`a_frustum_beside_a_block_comes_back_from_absorption_with_its_wall_in_pieces`]
/// pins.
///
/// Only the rows whose block misses the frustum answered until a cone face came
/// to be read off its sides seen down the axis (row Q1227), so `A ∩ B` was
/// empty and this was `A ∪ ∅` — the corners and the volume still have to come
/// back. Every row answers now.
#[test]
fn a_cone_united_with_what_it_shares_with_a_block_is_the_cone() {
    let pairs: Vec<Pair> = every_pair()
        .into_iter()
        .filter(|pair| !is_one_of(*pair, &UNITED_WITH_THE_SHARE_IN_PIECES))
        .collect();
    absorption(&pairs, Which::TheCone, Rule::Intersection, Rule::Union)
        .print_and_check("absorption A ∪ (A ∩ B), A the cone", 12);
}

/// `A ∩ (A ∪ B)` is `A`, with `A` the block.
#[test]
fn a_block_shared_with_its_union_with_a_cone_is_the_block() {
    absorption(
        &every_pair(),
        Which::TheBlock,
        Rule::Union,
        Rule::Intersection,
    )
    .print_and_check("absorption A ∩ (A ∪ B), A the block", 4);
}

/// `A ∩ (A ∪ B)` is `A`, with `A` the cone, over every pair but the three of
/// [`FRUSTUMS_WHOSE_SHARE_KEEPS_THE_CIRCLE`], which the next test pins as the
/// wrong answers they are, and the rows of [`SHARED_WITH_THE_UNION_IN_PIECES`],
/// which [`a_frustum_beside_a_block_comes_back_from_absorption_with_its_wall_in_pieces`]
/// pins.
///
/// Three of those rows, `(0.5, 1.0, 6.0)`, `(0.25, 3.0, 10.0)` and
/// `(0.25, 3.0, 6.0)`, answer since the half-plane fix of Q1660 (commit
/// `ca916de9`); before it they were refused `CannotTellTheCellsOfAFaceApart`.
#[test]
fn a_cone_shared_with_its_union_with_a_block_is_the_cone() {
    let pairs: Vec<Pair> = every_pair()
        .into_iter()
        .filter(|pair| {
            !its_share_keeps_the_circle(*pair)
                && !is_one_of(*pair, &SHARED_WITH_THE_UNION_IN_PIECES)
        })
        .collect();
    absorption(&pairs, Which::TheCone, Rule::Union, Rule::Intersection)
        .print_and_check("absorption A ∩ (A ∪ B), A the cone", 8);
}

/// `A ∪ (A ∩ B)` and `A ∩ (A ∪ B)`, with `A` a frustum and `B` a block beside
/// it, in the rows where each answers with the frustum's wall in pieces. **A
/// known wrong B-rep, pinned as it is and not as right.**
///
/// Each was refused `AFaceTrimmedByACurveThisCannotFollow` until a cone face
/// came to be read off its sides seen down the axis (row Q1227). Each answer
/// now validates and measures what `A` measures within `measure`'s own bounds,
/// so it is the solid `A`; but its wall comes back cut along the whole of where
/// the block's walls `x = 1` and `y = 1` cross the cone, into pieces of one
/// cone that the fuse does not join, so it has corners `A` never had, and
/// lacks the corners of `A`'s seam. The day the fuse joins those pieces this
/// fails, and the rows go back to the two laws.
///
/// `(0.5, 1.0, 6.0)`, `(0.25, 3.0, 10.0)` and `(0.25, 3.0, 6.0)` answer in
/// both this test's rows and `SHARED_WITH_THE_UNION_IN_PIECES`'s since the
/// half-plane fix of Q1660 (commit `ca916de9`); before it they were refused
/// `CannotTellTheCellsOfAFaceApart`.
#[test]
fn a_frustum_beside_a_block_comes_back_from_absorption_with_its_wall_in_pieces() {
    let policy = Policy::certified();
    let laws = [
        (
            &UNITED_WITH_THE_SHARE_IN_PIECES[..],
            Rule::Intersection,
            Rule::Union,
        ),
        (
            &SHARED_WITH_THE_UNION_IN_PIECES[..],
            Rule::Union,
            Rule::Intersection,
        ),
    ];
    for (rows, inner, outer) in laws {
        for &(slope, near, far) in rows {
            let pair = Pair::BesideABlock { slope, near, far };
            let law = format!("{pair:?}: A {outer:?} (A {inner:?} B)");
            let mut model = a_model();
            let (block, cone) = pair.build_into(&mut model);
            let (after_inner, inner_pieces) = fused(&model, &[cone], &[block], inner)
                .unwrap_or_else(|why| panic!("{law}: the inner fuse refused at {why}"));
            let (answer, pieces) = fused(&after_inner, &[cone], &inner_pieces, outer)
                .unwrap_or_else(|why| panic!("{law}: refused at {why}"));
            answer
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{law}: not a sound solid: {why:?}"));
            assert_eq!(pieces.len(), 1, "{law}: one solid");
            assert_ne!(
                corners_of(&answer, &pieces),
                corners_of(&model, &[cone]),
                "{law}: the wall came back whole — move the row back to the law"
            );
            let found = volume_of(&answer, &pieces);
            let want = volume_of(&model, &[cone]);
            assert!(
                (found.volume - want.volume).abs() <= found.bound + want.bound,
                "{law}: measures {} where A measures {}",
                found.volume,
                want.volume
            );
        }
    }
}

/// `A ∩ (A ∪ B)` with `A` the frustum, in the three rows where it answers with
/// a face, an edge and a corner `A` never had. **A known wrong B-rep, pinned as
/// it is and not as right**: the row filed from Q977 is to leave the circle out
/// (`docs/decisions.md` §325.5).
///
/// The block stands beside the frustum with their two boxes touching, so
/// `A ∪ B` cuts the frustum's wall where the block's floor plane
/// `z = near + 1` crosses it, and hands the wall back as two bands with a
/// circle between them. `a_rim_a_fuse_cut_round_one_cone.rs` pins that circle
/// on the first of these rows. `A ∩ (A ∪ B)` has the same block among its
/// tools, is cut by the same plane, and keeps the circle too: four faces where
/// `A` has three, and the corner the circle starts at. It validates, and
/// measures what `A` measures within `measure`'s own bounds.
///
/// The first of them refused `AFaceMissesPartOfWhereItsSurfacesCross` until Q977 let a
/// band of `A ∪ B` read the circle by the plane its run names. Leaving the circle out
/// in `assemble.rs`, where a band's rim in two arcs of one closed input edge
/// counts as the one closed side it becomes, gave all three `A` back and took
/// the circle out of `A ∪ B` as well, which is the circle the rim file pins.
///
/// The day the fuse leaves the circle out, this fails, and the three rows go
/// back to [`a_cone_shared_with_its_union_with_a_block_is_the_cone`].
#[test]
fn a_frustum_shared_with_its_union_with_a_block_beside_it_keeps_the_circle_the_union_cut() {
    let policy = Policy::certified();
    for (near, circle_starts_at) in FRUSTUMS_WHOSE_SHARE_KEEPS_THE_CIRCLE {
        let pair = Pair::BesideABlock {
            slope: 0.125,
            near,
            far: 8.0,
        };
        let mut model = a_model();
        let (block, cone) = pair.build_into(&mut model);
        let (united, union) = fused(&model, &[cone], &[block], Rule::Union)
            .unwrap_or_else(|why| panic!("{pair:?}: A ∪ B refused at {why}"));
        let (answer, pieces) = fused(&united, &[cone], &union, Rule::Intersection)
            .unwrap_or_else(|why| panic!("{pair:?}: A ∩ (A ∪ B) refused at {why}"));
        answer
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{pair:?}: A ∩ (A ∪ B) is not a sound solid: {why:?}"));
        assert_eq!(pieces.len(), 1, "{pair:?}: A ∩ (A ∪ B) is one solid");

        let mut want = corners_of(&model, &[cone]);
        want.push(circle_starts_at.map(f64::to_bits));
        want.sort_unstable();
        assert_eq!(
            corners_of(&answer, &pieces),
            want,
            "{pair:?}: A ∩ (A ∪ B) has A's corners and the one the circle at z = {} starts at",
            near + 1.0
        );
        let faces_of = |model: &Model, solid: SolidKey| {
            model
                .solid(solid)
                .expect("this model made the key")
                .faces()
                .count()
        };
        assert_eq!(
            (faces_of(&answer, pieces[0]), faces_of(&model, cone)),
            (4, 3),
            "{pair:?}: A ∩ (A ∪ B) has A's wall in two bands"
        );

        let found = volume_of(&answer, &pieces);
        let want = volume_of(&model, &[cone]);
        println!(
            "{pair:?}: A ∩ (A ∪ B) measures {} ± {}, A {} ± {}",
            found.volume, found.bound, want.volume, want.bound
        );
        assert!(
            (found.volume - want.volume).abs() <= found.bound + want.bound,
            "{pair:?}: A ∩ (A ∪ B) measures {} where A measures {}",
            found.volume,
            want.volume
        );
    }
}

/// `(A ∪ B) ∪ C` is `A ∪ (B ∪ C)`, with `A` a block, `B` a cone and `C` a
/// second cube, over every trio where the two groupings agree about whether
/// there is an answer.
#[test]
fn uniting_a_block_a_cone_and_a_cube_does_not_depend_on_which_two_go_first() {
    associativity(&TRIOS_THAT_AGREE_ABOUT_ANSWERING)
        .print_and_check("associativity, a block, a cone and a cube", 6);
}

/// A frustum up through a block's floor, and a small cube wholly inside the
/// block, above the frustum.
///
/// `(block ∪ frustum) ∪ cube` answers. `block ∪ (frustum ∪ cube)` unites the
/// frustum and the cube, which never touch, into two pieces, and then that with
/// the block came back [`rcad_topo::Error::ResultIsNotManifold`] (§222.3, item
/// 1).
///
/// The cone had nothing to do with it. The next test is the same refusal with a
/// box where the frustum was, and says where it came from: two tools whose
/// boxes miss each other were never cut against each other, though the block
/// reaches both.
#[test]
fn a_frustum_a_block_and_a_cube_inside_the_block_unite_one_way_and_not_the_other() {
    associativity(&[Trio {
        pair: on_the_grid(9, 2, [3, 5]),
        third_side: 3.0,
        third_at: [0.5, 0.5, 4.5],
    }])
    .print_and_check("associativity, a cube inside the block", 1);
}

/// A block of nine, a box of two up through its floor, and a box of three
/// inside the block that touches neither: `block ∪ [poke, inside]`, with the two
/// boxes handed over together as the tools, is `block ∪ poke`.
///
/// Every one of the three rules came back
/// [`rcad_topo::Error::ResultIsNotManifold`] here. The two boxes' own boxes miss
/// each other, so they were never cut against each other; but the inside box's
/// wall `x = 3.5` cuts the block's floor, and that floor meets the poking box's
/// walls `y = 4` and `y = 6` along its footprint. The floor side of those two
/// edges was split at `x = 3.5` and the wall side was not, so each half had one
/// face along it.
#[test]
fn a_block_with_one_box_through_its_floor_and_one_inside_it_unites_to_the_block_and_the_first() {
    let mut model = a_model();
    let block = a_cube_at(&mut model, 9.0, [0.0, 0.0, 0.0]);
    let poke = a_cube_at(&mut model, 2.0, [2.0, 4.0, -1.0]);
    let inside = a_cube_at(&mut model, 3.0, [0.5, 0.5, 4.5]);

    let mut report = Report::default();
    let want = fused(&model, &[block], &[poke], Rule::Union).expect("a block and a box answer");
    match fused(&model, &[block], &[poke, inside], Rule::Union) {
        Ok(found) => report.compare(
            "block ∪ [poke, inside] against block ∪ poke",
            (&found.0, &found.1),
            (&want.0, &want.1),
        ),
        Err(name) => report.refused(&name),
    }
    for (rule, solids) in [(Rule::Intersection, 2), (Rule::Difference, 1)] {
        match fused(&model, &[block], &[poke, inside], rule) {
            Ok((_, pieces)) => assert_eq!(
                pieces.len(),
                solids,
                "{rule:?}: the poking box's part inside the block, and the inside box as \
                 itself or as a hollow"
            ),
            Err(name) => report.refused(&format!("{rule:?}: {name}")),
        }
    }
    assert!(report.refused.is_empty(), "refused: {:?}", report.refused);
    report.print_and_check(
        "a block with one box through its floor and one inside it",
        1,
    );
}

/// A pointed cone the block swallows whole, and a cube cutting through both.
///
/// `(block ∪ cone) ∪ cube` answers, since `block ∪ cone` is the block.
///
/// `cone ∪ cube` used to be refused, undecided on a point inside a piece of a
/// face, and answers now:
/// [`a_cube_wall_parallel_to_a_pointed_cones_axis_cuts_it_and_both_ways_answer`].
/// What still refuses is uniting that answer with the block, with
/// [`rcad_topo::Error::AFaceTrimmedByACurveThisCannotFollow`] on the answer's
/// cone face — the part with `x < 3.5`, round the apex, whose one wire is a long
/// arc of the cap circle and the hyperbola arc the wall cut, between
/// `(3.5, 1.064, 5)` and `(3.5, 4.936, 5)`. That is items 4 and 5 of §222.3
/// again: the fuse cannot take back in a face of its own answer bounded by a
/// hyperbola arc.
#[test]
#[ignore = "associativity: pointed cone inside a block, cube cutting both — (cone ∪ cube) ∪ block \
            is AFaceTrimmedByACurveThisCannotFollow on the answer's cone face, bounded by a \
            hyperbola arc; the other grouping answers"]
fn a_swallowed_pointed_cone_and_a_cube_through_it_unite_one_way_and_not_the_other() {
    associativity(&[Trio {
        pair: a_pointed_cone_in_a_block_six_across(1.0),
        third_side: 6.0,
        third_at: [3.5, 0.5, 0.5],
    }])
    .print_and_check("associativity, a cube through a swallowed cone", 1);
}

/// The pointed cone of the two tests either side, and the cube whose wall
/// `x = 3.5` runs half a unit off the cone's axis, parallel to it.
///
/// That wall cuts the cone in a hyperbola, and the part of the wall inside the
/// cone is a piece bounded by one arc of it and the chord the cap's plane cuts
/// across — two corners, and until `select.rs` counted the bulge of a
/// hyperbola arc on the arc's own half of the cone, no point inside it, so
/// `cone ∪ cube` was Undecidable on a point inside a piece of a face.
///
/// Union and intersection both answer now, and what they measure adds up to
/// what the two inputs measure, within `measure`'s own bounds.
#[test]
fn a_cube_wall_parallel_to_a_pointed_cones_axis_cuts_it_and_both_ways_answer() {
    let mut model = a_model();
    let (_, cone) = a_pointed_cone_in_a_block_six_across(1.0).build_into(&mut model);
    let cube = model
        .add_cube(6.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a whole-number cube");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(3.5, 0.5, 0.5)).expect("a finite shift"),
        )
        .expect("this model made the key");

    let (united_model, united) =
        fused(&model, &[cone], &[cube], Rule::Union).expect("cone ∪ cube answers");
    let (shared_model, shared) =
        fused(&model, &[cone], &[cube], Rule::Intersection).expect("cone ∩ cube answers");
    assert_eq!((united.len(), shared.len()), (1, 1));

    let where_the_cap_crosses_the_wall =
        [1.063_508_326_896_291_5, 4.936_491_673_103_708].map(|y| [3.5, y, 5.0].map(f64::to_bits));
    let shared_corners = corners_of(&shared_model, &shared);
    for corner in where_the_cap_crosses_the_wall {
        assert!(
            shared_corners.contains(&corner),
            "the intersection's corners {shared_corners:?} miss {corner:?}"
        );
    }

    let [united, shared, cone, cube] = [
        volume_of(&united_model, &united),
        volume_of(&shared_model, &shared),
        volume_of(&model, &[cone]),
        volume_of(&model, &[cube]),
    ];
    let off_by = (united.volume + shared.volume) - (cone.volume + cube.volume);
    let bound = united.bound + shared.bound + cone.bound + cube.bound;
    assert!(
        off_by.abs() <= bound,
        "union {} and intersection {} against cone {} and cube {}: off by {off_by:e} beyond {bound:e}",
        united.volume,
        shared.volume,
        cone.volume,
        cube.volume
    );
}

/// A pointed cone the block swallows whole, and a small cube grazing the cone's
/// wall near the block's corner.
///
/// `(block ∪ cone) ∪ cube` answers. `cone ∪ cube` is refused with
/// [`rcad_topo::Error::ACurvedFaceGoesRightRoundWithNoSeam`], and the refusal is
/// right to stop there, though not for the reason its name gives.
///
/// The cube's three planes through its corner `(2.5, 2.5, 2.5)` cut the cone's
/// face **past the cube** as well: the plane `x = 2.5` along a whole hyperbola
/// branch through `(2.5, 3, 2)`, `y = 2.5` likewise, and `z = 2.5` right round.
/// The union keeps every piece outside the cube as a face of its own, so one of
/// them is the patch round the apex bounded by those three arcs, with corners
/// `(2.5, 3.559, 2.5)`, `(3.559, 2.5, 2.5)` and `(2.5, 2.5, 2.414)`, and every
/// edge of it has the cone on both sides. Measuring the answer meshes that
/// patch, and `mesh.rs`'s `which_pole_this_cap_reaches` looks for the surface
/// across its first edge among the faces' surfaces, finds only the cone, and
/// gives up — which is where the refusal is raised.
///
/// Reading that surface off the edge's run instead (`EdgeRun::PartOfWhereItCrosses`
/// names the plane) meshes it, and the union then answers with those extra
/// corners on it — and uniting that answer with the block is refused with
/// `AFaceMeetsItsNeighbourOnItsOwnSurface`, and `A ∪ (A ∩ B)` over the cone
/// whose apex is at `z = 3` answers with a corner `(3, 4.5, 6)` the cone does
/// not have. So the mesher is not the fault: the answer should not carry pieces
/// of one cone as separate faces, and until the fuse joins them the refusal is
/// the better of the two outcomes.
#[test]
#[ignore = "associativity: pointed cone inside a block, small cube grazing it — cone ∪ cube keeps \
            the cone's face cut into pieces by the cube's planes past the cube, and the piece \
            round the apex cannot be meshed (ACurvedFaceGoesRightRoundWithNoSeam)"]
fn a_swallowed_pointed_cone_and_a_cube_grazing_it_unite_one_way_and_not_the_other() {
    associativity(&[Trio {
        pair: a_pointed_cone_in_a_block_six_across(1.0),
        third_side: 2.0,
        third_at: [0.5, 0.5, 0.5],
    }])
    .print_and_check("associativity, a cube grazing a swallowed cone", 1);
}

/// A frustum and the block that cuts a piece out of it, and a cube far from
/// both.
///
/// `block ∪ (frustum ∪ cube)` answers. `(block ∪ frustum) ∪ cube` refuses at
/// the second union with
/// [`rcad_topo::Error::AFaceMeetsItsNeighbourOnItsOwnSurface`], even with a cube
/// that touches nothing.
///
/// **It used to refuse one step earlier, on a circle.** Until §223.4's lane the
/// refusal was [`rcad_topo::Error::AFaceTrimmedByACurveThisCannotFollow`],
/// `fuse/arrange.rs`'s
/// `which_way_an_edge_runs_round`, reached from `cuts::gather` checking that the
/// answer does not cross itself, on the answer's block floor at `z = 5` and its
/// arc of the cone's radius-2.5 circle between `(√5.25, 1, 5)` and
/// `(1, √5.25, 5)`. The first union lowered that arc's picker until the corners
/// *as it stored them* read on or in front of it; the second union re-solves
/// each corner from the wall, the floor and the cone into a box one ulp wide,
/// and the picker's plane runs through both boxes, so neither end is on or in
/// front and nothing says which arc it is. `fuse/assemble.rs`'s
/// `the_half_space_both_ends_are_on` now lowers the picker against each corner
/// grown by the room `fuse/bake.rs` grants a re-solved corner, while the
/// circle's centre stays behind it (`pieces_of_one_curved_surface.rs`).
///
/// **What stands now is about the hyperbola**, and it is
/// [`rcad_topo::Error::AFaceMeetsItsNeighbourOnItsOwnSurface`]: the
/// answer keeps the frustum's cone as several faces, cut apart by the block's
/// walls where they run on below the block, so a cone face with corners
/// `(−√3, 1, 4)`, `(1, −√3, 4)`, `(1, −√5.25, 5)` and `(−√5.25, 1, 5)` has the
/// same cone across three of its four sides. Nothing folds curved pieces of one
/// surface back into one face the way flat ones are folded, so the fuse hands
/// back an answer it cannot read. Also a gap: the one face those pieces make up
/// exists.
///
/// Both groupings answer and agree since the fuse joins pieces of one cone
/// (§224.2) and a wall's corner on the cone stopped counting both branches of
/// the hyperbola its plane cuts the cone in.
#[test]
fn a_frustum_cut_by_a_block_and_a_far_cube_unite_one_way_and_not_the_other() {
    associativity(&[Trio {
        pair: Pair::BesideABlock {
            slope: 0.5,
            near: 4.0,
            far: 9.0,
        },
        third_side: 2.0,
        third_at: [20.0, 20.0, 20.0],
    }])
    .print_and_check("associativity, a cube far from a cut frustum", 1);
}

/// The same frustum and block, and a small cube standing in the frustum.
///
/// `block ∪ (cone ∪ third)` answers and `(block ∪ cone) ∪ third` refuses at
/// the second union: `Undecidable` on a point in a piece that every solid
/// could place. It no longer stops on unjoined cone pieces, which the far cube
/// stopped refusing on once the fuse joined pieces of one cone (§224.2). Q102
/// answered this refusal for a bend, not for this trio.
#[test]
#[ignore = "associativity: frustum cut by a block, cube inside the frustum — (block ∪ cone) ∪ \
            third refuses at the second union: Undecidable on a point in a piece that every \
            solid could place"]
fn a_frustum_cut_by_a_block_and_a_cube_in_the_frustum_unite_one_way_and_not_the_other() {
    associativity(&[Trio {
        pair: Pair::BesideABlock {
            slope: 0.5,
            near: 4.0,
            far: 9.0,
        },
        third_side: 2.0,
        third_at: [3.5, 3.5, 0.5],
    }])
    .print_and_check("associativity, a cube in a cut frustum", 1);
}

/// A frustum up through a block's floor, and a cube as big as the block beside
/// it, overlapping it and clear of the frustum.
///
/// `block ∪ (frustum ∪ cube)` answers. `(block ∪ frustum) ∪ cube` refuses at
/// the first union, undecided on a point inside a piece of a face — the same
/// refusal as the swallowed pointed cone's `cone ∪ cube`, this time on a block
/// and a frustum alone.
///
/// Until §222.3's first case was fixed both groupings refused, so this sat
/// among the trios that agree: the grouping that answers now was the
/// frustum and the cube handed to the block as two tools, which is that case.
///
/// **Un-ignored by Q1102: both groupings now answer and agree.** Found
/// passing by Q929; which change of the several since this was ignored fixed
/// it was not traced.
#[test]
fn a_frustum_a_block_and_a_cube_beside_the_frustum_unite_one_way_and_not_the_other() {
    associativity(&[Trio {
        pair: on_the_grid(8, 2, [4, 4]),
        third_side: 8.0,
        third_at: [5.0, 0.5, 2.5],
    }])
    .print_and_check("associativity, a cube beside the frustum", 1);
}

/// The skin of `cone ∪ cube` for the frustum of
/// [`TRIOS_THAT_AGREE_ABOUT_ANSWERING`]'s fifth trio, asked through the public
/// API whether it holds a point on each side of the notch the cube's corner
/// cuts out of it.
///
/// The skin is the band between the caps `z = −1` and `z = 1` less the part
/// with `x ≥ 3.5`, `y ≤ 4.5` and `z ≤ 0.5`. Its five cuts are no family
/// `holds.rs` follows, and every point was refused
/// `AFaceTrimmedByACurveThisCannotFollow` until the face came to be read off
/// its sides seen down the cone's axis (row Q1227). `(4.5, 3, 0)` is where a ray
/// from the block's floor leaves the cone in `(cone ∪ cube) ∪ block`; it is in
/// the notch. `(1.5, 3, 0)` is across the axis from it, and `(4.875, 3, 0.75)`
/// above the cube's roof, both on the skin. `(4, 3, −5)` is on the cone's
/// other nappe, below the apex.
#[test]
fn the_skin_of_a_frustum_notched_by_a_cubes_corner_holds_the_points_off_the_notch() {
    let mut model = a_model();
    let (_, cone) = on_the_grid(6, 2, [3, 3]).build_into(&mut model);
    let cube = a_cube_at(&mut model, 4.0, [3.5, 0.5, -3.5]);
    let (united, pieces) =
        fused(&model, &[cone], &[cube], Rule::Union).expect("cone ∪ cube answers");
    let solid = united.solid(pieces[0]).expect("this model made the key");
    let skins: Vec<_> = solid
        .faces()
        .filter(|face| matches!(face.surface(), Surface::Cone(_)))
        .map(|face| face.key())
        .collect();
    let [skin] = skins[..] else {
        panic!("cone ∪ cube has one face on the cone, not {}", skins.len());
    };
    for (at, want) in [
        ([4.5, 3.0, 0.0], OnAFace::NotHeld),
        ([1.5, 3.0, 0.0], OnAFace::Held),
        ([4.875, 3.0, 0.75], OnAFace::Held),
        ([4.0, 3.0, -5.0], OnAFace::NotHeld),
    ] {
        let found = solid
            .face_holds_the_point(
                skin,
                PredicatePoint::from(at),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("the notched skin at {at:?} refused: {why:?}"));
        assert_eq!(found, want, "the notched skin at {at:?}");
    }
}

/// The skin of `cone ∪ cube` for the frustum of
/// [`TRIOS_THAT_AGREE_ABOUT_ANSWERING`]'s last trio, whose cube's wall `x = 5`
/// touches the frustum's bottom circle at one point.
///
/// The circle is radius one at `z = −1` round `(4, 4)`, so the wall cuts the
/// skin in a branch of a hyperbola whose tip is `(5, 4, −1)`, on the circle.
/// The skin is the band between the caps less the part with `x > 5`, and its
/// one wire runs right round the circle from the tip back to it, up the branch,
/// round the top cap's arc and down the branch again. Every point was refused
/// `AFaceTrimmedByACurveThisCannotFollow` at that circle, a side with no two
/// ends, until the face came to read it whole (row Q1246). `(5.5, 4, 0)` is
/// where a ray from the block's floor leaves the cone in `(cone ∪ cube) ∪
/// block`; it is past the wall. `(4.5, 4, −2)` is on the cone below the bottom
/// cap and is seen inside every side but the circle, so only the circle turns it
/// away. `(5, 4, −5)` is on the other nappe.
#[test]
fn the_skin_of_a_frustum_whose_bottom_circle_a_wall_touches_holds_the_band_short_of_the_wall() {
    let mut model = a_model();
    let (_, cone) = on_the_grid(8, 2, [4, 4]).build_into(&mut model);
    let cube = a_cube_at(&mut model, 8.0, [5.0, 0.5, -2.5]);
    let (united, pieces) =
        fused(&model, &[cone], &[cube], Rule::Union).expect("cone ∪ cube answers");
    let solid = united.solid(pieces[0]).expect("this model made the key");
    let skins: Vec<_> = solid
        .faces()
        .filter(|face| matches!(face.surface(), Surface::Cone(_)))
        .map(|face| face.key())
        .collect();
    let [skin] = skins[..] else {
        panic!("cone ∪ cube has one face on the cone, not {}", skins.len());
    };
    for (at, want) in [
        ([5.5, 4.0, 0.0], OnAFace::NotHeld),
        ([6.0, 4.0, 1.0], OnAFace::NotHeld),
        ([4.0, 5.5, 0.0], OnAFace::Held),
        ([2.5, 4.0, 0.0], OnAFace::Held),
        ([4.0, 2.5, 0.0], OnAFace::Held),
        ([4.0, 3.0, -1.0], OnAFace::OnItsBoundary),
        ([4.5, 4.0, -2.0], OnAFace::NotHeld),
        ([5.0, 4.0, -5.0], OnAFace::NotHeld),
    ] {
        let found = solid
            .face_holds_the_point(
                skin,
                PredicatePoint::from(at),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("the skin at {at:?} refused: {why:?}"));
        assert_eq!(found, want, "the skin at {at:?}");
    }
}

/// A cube and a cone-faced solid.
#[derive(Clone, Copy, Debug)]
enum Pair {
    /// `pairs::FrustumAndBlock`, built the way that family builds it.
    Frustum(FrustumAndBlock),
    /// One row of `what_the_answered_block_fuses_actually_say.rs`'s family: a
    /// frustum about `z` with its apex at the origin, and beside it a box
    /// `slope · far` square and `far` tall with its low corner at
    /// `(1, 1, near + 1)`.
    BesideABlock { slope: f64, near: f64, far: f64 },
    /// A cube of `side` at the origin, and a cone that ends in a point with its
    /// apex at `apex`, widening by `slope` per unit up to a cap `far` above it.
    Pointed {
        side: f64,
        slope: f64,
        far: f64,
        apex: [f64; 3],
    },
}

/// A pair and a third cube, for associativity.
#[derive(Clone, Copy, Debug)]
struct Trio {
    pair: Pair,
    third_side: f64,
    third_at: [f64; 3],
}

/// Frustums up through a block's floor, three slopes, and one off the diagonal.
const FRUSTUMS: [Pair; 4] = [
    on_the_grid(6, 2, [3, 3]),
    on_the_grid(8, 4, [4, 4]),
    on_the_grid(10, 8, [5, 5]),
    on_the_grid(9, 2, [3, 5]),
];

/// A pointed cone with its apex below the block's floor, poking up through it;
/// one with its apex inside the block, poking out through the roof; and one
/// the block swallows whole.
const POINTED: [Pair; 3] = [
    a_pointed_cone_in_a_block_six_across(-1.0),
    a_pointed_cone_in_a_block_six_across(3.0),
    a_pointed_cone_in_a_block_six_across(1.0),
];

/// Trios on which the two groupings both answer: a far cube with a frustum, a
/// cube on the block's roof with a frustum, a far cube with a swallowed
/// pointed cone, and a far cube with a pointed cone up through the block's
/// floor.
///
/// Three more used to be here refusing both ways, so that a change making one
/// grouping answer would be seen. Counting the bulge of a hyperbola arc was
/// that change, and two of them came to answer one way round only, which a
/// test pinned by where the other grouping stopped. The third is the fourth one
/// here: the tip of its cone hangs under the floor, and `(block ∪ cone) ∪
/// third` turned that tip inside out until the circle it hangs from said which
/// side of the floor it is on (`docs/decisions.md` §340.9).
///
/// The fifth, a cube of four notching the frustum's skin at its corner, came
/// back from answering one way round by row Q1227. `(cone ∪ third)
/// ∪ block` used to stop on `AFaceTrimmedByACurveThisCannotFollow`: the skin of
/// `cone ∪ third` is the band between the caps `z = −1` and `z = 1` less the
/// cube's corner `x ≥ 3.5, y ≤ 4.5, z ≤ 0.5`, with a seam up `x = 3.5` from the
/// cube's roof to the top cap, and `select.rs`'s `place_the_piece` asks it
/// whether it holds `(4.5, 3, 0)`, where a ray from the piece of the block's
/// floor inside the cube leaves the cone (Q1200). Its five cuts are no family
/// `holds.rs` follows, and `holds.rs` now reads such a face off its own sides
/// seen down the cone's axis instead. Before that, sharing the arc picker
/// between `holds.rs` and `arrange.rs` (Q1033) had moved the refusal to
/// `CannotTellTheCellsOfAFaceApart` on the same face, and holding a lump on the
/// other nappe from the whole rim outside the face (Q1112) moved it back.
///
/// The last, a cube of eight whose wall `x = 5` touches the frustum's bottom
/// circle at one point, was the other of the two that answered one way round.
/// Its first union stopped on *side of plane* until a corner moved by a shift
/// that rounds nothing was handed on exact (Q509); then `(cone ∪ third) ∪
/// block` was `Undecidable` on *a point in a piece that every solid could
/// place* until a point whose ray met a face nobody follows named that face
/// (Q882), and stopped on `AFaceTrimmedByACurveThisCannotFollow` after it: the
/// skin of `cone ∪ third` has the whole bottom circle as a side from the tip of
/// the wall's hyperbola back to it, and `holds.rs` read no side with no two
/// ends until row Q1246.
const TRIOS_THAT_AGREE_ABOUT_ANSWERING: [Trio; 6] = [
    Trio {
        pair: on_the_grid(6, 2, [3, 3]),
        third_side: 2.0,
        third_at: [20.0, 20.0, 20.0],
    },
    Trio {
        pair: on_the_grid(6, 2, [3, 3]),
        third_side: 6.0,
        third_at: [0.0, 0.0, 6.0],
    },
    Trio {
        pair: a_pointed_cone_in_a_block_six_across(1.0),
        third_side: 2.0,
        third_at: [20.0, 20.0, 20.0],
    },
    Trio {
        pair: a_pointed_cone_in_a_block_six_across(-1.0),
        third_side: 2.0,
        third_at: [20.0, 20.0, 20.0],
    },
    Trio {
        pair: on_the_grid(6, 2, [3, 3]),
        third_side: 4.0,
        third_at: [3.5, 0.5, -3.5],
    },
    Trio {
        pair: on_the_grid(8, 2, [4, 4]),
        third_side: 8.0,
        third_at: [5.0, 0.5, -2.5],
    },
];

/// A frustum of slope `1 / up_per_unit_wider` from a cap of radius one a unit
/// below the floor of a block `side` across, to a cap of radius two, with its
/// axis at `(at, at)`: one row of §49.6's eighty-four-configuration grid.
const fn on_the_grid(side: u32, up_per_unit_wider: u32, at: [u32; 2]) -> Pair {
    Pair::Frustum(FrustumAndBlock {
        side,
        narrow: 1,
        wide: 2,
        up_per_unit_wider,
        at,
    })
}

/// A pointed cone of slope one half and height four, standing in a block six
/// across with its axis at `(3, 3)` and its apex at `z`.
const fn a_pointed_cone_in_a_block_six_across(z: f64) -> Pair {
    Pair::Pointed {
        side: 6.0,
        slope: 0.5,
        far: 4.0,
        apex: [3.0, 3.0, z],
    }
}

/// The three rows of [`the_rows_the_answered_block_family_draws`], all of slope
/// one eighth and `far` eight, whose `A ∩ (A ∪ B)` keeps the circle `A ∪ B` cut
/// round the frustum's wall: each row's `near`, and the corner the circle
/// starts at, at `z = near + 1`.
const FRUSTUMS_WHOSE_SHARE_KEEPS_THE_CIRCLE: [(f64, [f64; 3]); 3] = [
    (1.0, [0.0, -0.25, 2.0]),
    (2.0, [0.0, -0.375, 3.0]),
    (4.0, [0.0, -0.625, 5.0]),
];

/// The rows of [`the_rows_the_answered_block_family_draws`], as `(slope, near,
/// far)`, whose `A ∪ (A ∩ B)`, with `A` the cone, comes back with the frustum's
/// wall in pieces.
const UNITED_WITH_THE_SHARE_IN_PIECES: [(f64, f64, f64); 10] = [
    (0.5, 4.0, 9.0),
    (0.5, 2.0, 10.0),
    (0.5, 3.0, 11.0),
    (0.5, 2.0, 11.0),
    (0.5, 1.0, 6.0),
    (0.25, 3.0, 10.0),
    (0.25, 3.0, 6.0),
    (0.25, 2.0, 12.0),
    (0.25, 1.0, 8.0),
    (0.25, 1.0, 12.0),
];

/// The same for `A ∩ (A ∪ B)`.
const SHARED_WITH_THE_UNION_IN_PIECES: [(f64, f64, f64); 7] = [
    (0.5, 4.0, 9.0),
    (0.5, 2.0, 10.0),
    (0.5, 3.0, 11.0),
    (0.5, 2.0, 11.0),
    (0.5, 1.0, 6.0),
    (0.25, 3.0, 10.0),
    (0.25, 3.0, 6.0),
];

/// Whether `pair` is a frustum beside a block with one of `rows`' numbers.
fn is_one_of(pair: Pair, rows: &[(f64, f64, f64)]) -> bool {
    let Pair::BesideABlock { slope, near, far } = pair else {
        return false;
    };
    let bits = |row: (f64, f64, f64)| [row.0, row.1, row.2].map(f64::to_bits);
    rows.iter()
        .any(|&row| bits(row) == bits((slope, near, far)))
}

/// Whether `pair` is one of [`FRUSTUMS_WHOSE_SHARE_KEEPS_THE_CIRCLE`].
fn its_share_keeps_the_circle(pair: Pair) -> bool {
    let Pair::BesideABlock { slope, near, far } = pair else {
        return false;
    };
    slope.to_bits() == 0.125_f64.to_bits()
        && far.to_bits() == 8.0_f64.to_bits()
        && FRUSTUMS_WHOSE_SHARE_KEEPS_THE_CIRCLE
            .iter()
            .any(|(row, _)| row.to_bits() == near.to_bits())
}

/// Every pair: the grid frustums, the pointed cones, and the sixteen rows
/// `what_the_answered_block_fuses_actually_say.rs` draws.
fn every_pair() -> Vec<Pair> {
    let mut every: Vec<Pair> = FRUSTUMS.iter().chain(POINTED.iter()).copied().collect();
    every.extend(the_rows_the_answered_block_family_draws());
    every
}

/// Copied from `what_the_answered_block_fuses_actually_say.rs` with its seed, so
/// these are that family's rows and not a family of this file's own.
fn the_rows_the_answered_block_family_draws() -> Vec<Pair> {
    let mut draw = Draw(0x5EED);
    (0..16)
        .map(|_| {
            let slope = 2.0_f64.powi(i32::try_from(draw.next() % 3).expect("a small number") - 3);
            let far = draw.whole(6, 12);
            let near = draw.whole(1, 4);
            Pair::BesideABlock { slope, near, far }
        })
        .collect()
}

struct Draw(u64);

impl Draw {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        self.0 >> 33
    }

    fn whole(&mut self, low: i64, high: i64) -> f64 {
        let span = u64::try_from(high - low + 1).expect("a range that runs upwards");
        let picked = i64::try_from(self.next() % span).expect("inside the range") + low;
        picked as f64
    }
}

/// What one law over one family saw.
#[derive(Default)]
struct Report {
    compared: usize,
    volumes_bit_identical: usize,
    refused: BTreeMap<String, usize>,
    findings: Vec<String>,
}

impl Report {
    fn print_and_check(self, law: &str, at_least: usize) {
        println!(
            "{law}: {} compared, {} with bit-identical volumes, refusals {:?}",
            self.compared, self.volumes_bit_identical, self.refused
        );
        for finding in &self.findings {
            println!("  finding: {finding}");
        }
        assert!(
            self.findings.is_empty(),
            "{law}: {} findings, printed above",
            self.findings.len()
        );
        assert!(
            self.compared >= at_least,
            "{law}: {} comparisons where {at_least} were made the day this was written — a \
             fall is a regression; re-derive it rather than relax it (§19.1)",
            self.compared
        );
    }

    fn refused(&mut self, name: &str) {
        *self.refused.entry(name.to_owned()).or_default() += 1;
    }

    /// Compare an answer against what the law says it is, and write down any
    /// difference as a finding.
    fn compare(&mut self, what: &str, found: (&Model, &[SolidKey]), want: (&Model, &[SolidKey])) {
        self.compared += 1;
        let (found_model, found_pieces) = found;
        let (want_model, want_pieces) = want;
        if found_pieces.len() != want_pieces.len() {
            self.findings.push(format!(
                "{what}: {} solids where the law says {}",
                found_pieces.len(),
                want_pieces.len()
            ));
            return;
        }
        let found_corners = corners_of(found_model, found_pieces);
        let want_corners = corners_of(want_model, want_pieces);
        if found_corners != want_corners {
            let extra: Vec<[f64; 3]> = found_corners
                .iter()
                .filter(|it| !want_corners.contains(it))
                .map(|it| it.map(f64::from_bits))
                .collect();
            let missing: Vec<[f64; 3]> = want_corners
                .iter()
                .filter(|it| !found_corners.contains(it))
                .map(|it| it.map(f64::from_bits))
                .collect();
            self.findings.push(format!(
                "{what}: corners differ — {extra:?} extra and {missing:?} missing"
            ));
        }
        let found_volume = volume_of(found_model, found_pieces);
        let want_volume = volume_of(want_model, want_pieces);
        if found_volume.volume.to_bits() == want_volume.volume.to_bits() {
            self.volumes_bit_identical += 1;
        }
        let off_by = found_volume.volume - want_volume.volume;
        let bound = found_volume.bound + want_volume.bound;
        if off_by.abs() > bound {
            self.findings.push(format!(
                "{what}: volume {} against {}, off by {off_by:e} beyond the bound {bound:e}",
                found_volume.volume, want_volume.volume
            ));
        }
    }
}

/// `A ∪ A'` and `A ∩ A'` are both `A` — and so answer or refuse together.
fn idempotence_of_the_cone_alone(pairs: &[Pair]) -> Report {
    let mut report = Report::default();
    for &pair in pairs {
        let (model, one, copy) = pair.the_cone_and_a_copy_of_it();
        let mut answered = Vec::new();
        for rule in [Rule::Union, Rule::Intersection] {
            match fused(&model, &[one], &[copy], rule) {
                Ok((answer, pieces)) => {
                    report.compare(
                        &format!("{pair:?}: {rule:?} of the cone and a copy of it"),
                        (&answer, &pieces),
                        (&model, &[one]),
                    );
                    answered.push(true);
                }
                Err(name) => {
                    report.refused(&name);
                    answered.push(false);
                }
            }
        }
        if answered[0] != answered[1] {
            report.findings.push(format!(
                "{pair:?}: asymmetry — the union of the cone and a copy of it answered: {}, the \
                 intersection answered: {}",
                answered[0], answered[1]
            ));
        }
    }
    report
}

/// `U ∪ U'` and `U ∩ U'` are both `U` — and so answer or refuse together.
fn idempotence_of_a_union_with_a_cone(pairs: &[Pair]) -> Report {
    let mut report = Report::default();
    for &pair in pairs {
        let mut model = a_model();
        let (block, cone) = pair.build_into(&mut model);
        let (block_again, cone_again) = pair.build_into(&mut model);
        let (model, united) = match fused(&model, &[block], &[cone], Rule::Union) {
            Ok(answer) => answer,
            Err(name) => {
                report.refused(&format!("making U: {name}"));
                continue;
            }
        };
        let Ok((model, united_again)) = fused(&model, &[block_again], &[cone_again], Rule::Union)
        else {
            report.findings.push(format!(
                "{pair:?}: asymmetry — the same union answered once and refused the second time"
            ));
            continue;
        };
        let mut answered = Vec::new();
        for rule in [Rule::Union, Rule::Intersection] {
            match fused(&model, &united, &united_again, rule) {
                Ok((answer, pieces)) => {
                    report.compare(
                        &format!("{pair:?}: {rule:?} of a block-and-cone union and a copy of it"),
                        (&answer, &pieces),
                        (&model, &united),
                    );
                    answered.push(true);
                }
                Err(name) => {
                    report.refused(&name);
                    answered.push(false);
                }
            }
        }
        if answered[0] != answered[1] {
            report.findings.push(format!(
                "{pair:?}: asymmetry — the union of U and a copy of it answered: {}, the \
                 intersection answered: {}",
                answered[0], answered[1]
            ));
        }
    }
    report
}

/// Which of the pair is `A` in an absorption law.
#[derive(Clone, Copy, Debug)]
enum Which {
    TheBlock,
    TheCone,
}

/// `A ∪ (A ∩ B)` is `A` when `inner` is intersection and `outer` union, and
/// `A ∩ (A ∪ B)` is `A` the other way round.
fn absorption(pairs: &[Pair], which: Which, inner: Rule, outer: Rule) -> Report {
    let mut report = Report::default();
    for &pair in pairs {
        let mut model = a_model();
        let (block, cone) = pair.build_into(&mut model);
        let (a, b) = match which {
            Which::TheBlock => (block, cone),
            Which::TheCone => (cone, block),
        };
        let (after_inner, inner_pieces) = match fused(&model, &[a], &[b], inner) {
            Ok(answer) => answer,
            Err(name) => {
                report.refused(&format!("A {inner:?} B: {name}"));
                continue;
            }
        };
        match fused(&after_inner, &[a], &inner_pieces, outer) {
            Ok((answer, pieces)) => report.compare(
                &format!("{pair:?}: A = {which:?}, A {outer:?} (A {inner:?} B)"),
                (&answer, &pieces),
                (&model, &[a]),
            ),
            Err(name) => report.refused(&name),
        }
    }
    report
}

/// `(A ∪ B) ∪ C` is `A ∪ (B ∪ C)`, with `A` the block, `B` the cone and `C` a
/// second cube.
fn associativity(trios: &[Trio]) -> Report {
    let mut report = Report::default();
    for &trio in trios {
        match both_groupings(trio) {
            (Ok((left_model, left_pieces)), Ok((right_model, right_pieces))) => report.compare(
                &format!("{trio:?}: block ∪ (cone ∪ third) against (block ∪ cone) ∪ third"),
                (&right_model, &right_pieces),
                (&left_model, &left_pieces),
            ),
            (Err(left), Err(right)) => {
                report.refused(&left);
                report.refused(&right);
            }
            (Ok(_), Err(right)) => report.findings.push(format!(
                "{trio:?}: asymmetry — (block ∪ cone) ∪ third answered and block ∪ (cone ∪ \
                 third) refused at {right}"
            )),
            (Err(left), Ok(_)) => report.findings.push(format!(
                "{trio:?}: asymmetry — block ∪ (cone ∪ third) answered and (block ∪ cone) ∪ \
                 third refused at {left}"
            )),
        }
    }
    report
}

/// Two unions in turn: the model and solids they left, or where they stopped.
type United = Result<(Model, Vec<SolidKey>), String>;

/// `(block ∪ cone) ∪ third` and `(cone ∪ third) ∪ block`, in that order, each
/// built into a fresh model.
fn both_groupings(trio: Trio) -> (United, United) {
    let mut model = a_model();
    let (block, cone) = trio.pair.build_into(&mut model);
    let third = model
        .add_cube(
            trio.third_side,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a whole-number cube");
    let [x, y, z] = trio.third_at;
    model
        .place_solid(
            third,
            &Isometry::translation(Vector::new(x, y, z)).expect("a finite shift"),
        )
        .expect("this model made the key");
    (
        united_in_turn(&model, &[block], &[cone], &[third]),
        united_in_turn(&model, &[cone], &[third], &[block]),
    )
}

/// Unite the first two, then unite that with the third; a refusal at either
/// step comes back named.
fn united_in_turn(
    model: &Model,
    first: &[SolidKey],
    second: &[SolidKey],
    third: &[SolidKey],
) -> United {
    let (model, two) = match fused(model, first, second, Rule::Union) {
        Ok(answer) => answer,
        Err(name) => return Err(format!("the first union: {name}")),
    };
    match fused(&model, &two, third, Rule::Union) {
        Ok(answer) => Ok(answer),
        Err(name) => Err(format!("the second union: {name}")),
    }
}

impl Pair {
    /// Build the block and then the cone into `model`, and hand back both.
    fn build_into(self, model: &mut Model) -> (SolidKey, SolidKey) {
        let policy = Policy::certified();
        match self {
            Pair::Frustum(pair) => {
                let block = model
                    .add_cube(f64::from(pair.side), &policy, &mut ProvenanceLog::new())
                    .expect("a whole-number cube");
                let frustum = a_frustum(model, pair.slope(), pair.near(), pair.far());
                let drop = Vector::new(
                    f64::from(pair.at[0]),
                    f64::from(pair.at[1]),
                    -1.0 - pair.near(),
                );
                model
                    .place_solid(
                        frustum,
                        &Isometry::translation(drop).expect("a whole-number shift"),
                    )
                    .expect("this model made the key");
                (block, frustum)
            }
            Pair::BesideABlock { slope, near, far } => {
                let frustum = a_frustum(model, slope, near, far);
                let side = slope * far;
                let rectangle =
                    ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
                        .build(&policy, &mut ProvenanceLog::new())
                        .expect("a rectangle with four corners");
                let block = model
                    .add_extrusion(
                        &rectangle,
                        far,
                        &Isometry::translation(Vector::new(1.0, 1.0, near + 1.0))
                            .expect("a finite shift"),
                    )
                    .expect("a box");
                (block, frustum)
            }
            Pair::Pointed {
                side,
                slope,
                far,
                apex,
            } => {
                let block = model
                    .add_cube(side, &policy, &mut ProvenanceLog::new())
                    .expect("a whole-number cube");
                let cone = a_pointed_cone(model, slope, far);
                model
                    .place_solid(
                        cone,
                        &Isometry::translation(Vector::new(apex[0], apex[1], apex[2]))
                            .expect("a whole-number shift"),
                    )
                    .expect("this model made the key");
                (block, cone)
            }
        }
    }

    /// The cone-faced solid alone, built twice into one fresh model.
    fn the_cone_and_a_copy_of_it(self) -> (Model, SolidKey, SolidKey) {
        let mut model = a_model();
        let (one, copy) = match self {
            Pair::Frustum(pair) => (
                a_frustum(&mut model, pair.slope(), pair.near(), pair.far()),
                a_frustum(&mut model, pair.slope(), pair.near(), pair.far()),
            ),
            Pair::BesideABlock { slope, near, far } => (
                a_frustum(&mut model, slope, near, far),
                a_frustum(&mut model, slope, near, far),
            ),
            Pair::Pointed { slope, far, .. } => (
                a_pointed_cone(&mut model, slope, far),
                a_pointed_cone(&mut model, slope, far),
            ),
        };
        (model, one, copy)
    }
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"))
}

fn a_cube_at(model: &mut Model, side: f64, at: [f64; 3]) -> SolidKey {
    let cube = model
        .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a whole-number cube");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a finite shift"),
        )
        .expect("this model made the key");
    cube
}

/// The model an answer is in and the answer's solids, or the name of why there
/// was no answer.
type Side = Result<(Model, Vec<SolidKey>), String>;

/// Fuse a copy of `model`. A certified answer that recorded a decision it could
/// not prove is a failure, as it is in `fuses_that_must_agree.rs`.
fn fused(model: &Model, arguments: &[SolidKey], tools: &[SolidKey], rule: Rule) -> Side {
    let mut model = model.clone();
    let mut log = ProvenanceLog::new();
    match model.fuse(arguments, tools, rule, &Policy::certified(), &mut log) {
        Ok(answer) => {
            assert!(
                log.is_empty(),
                "a certified {rule:?} recorded {} decisions it could not prove",
                log.len()
            );
            let solids = answer.solids().to_vec();
            Ok((model, solids))
        }
        Err(why) => Err(the_name_of(&why)),
    }
}

/// The error's variant without its fields, so refusals tally by kind — and for
/// an `Undecidable`, the decision it gave up on.
fn the_name_of(why: &rcad_topo::Error) -> String {
    if let rcad_topo::Error::Undecidable { records } = why {
        let last = records
            .records()
            .last()
            .map_or("nothing named", |record| record.predicate());
        return format!("Undecidable on {last}");
    }
    let written = format!("{why:?}");
    written
        .split(['{', '('])
        .next()
        .expect("split always hands back one piece")
        .trim()
        .to_owned()
}

/// Every corner of every face of every piece, as bits, sorted and without
/// repeats.
fn corners_of(model: &Model, pieces: &[SolidKey]) -> Vec<[u64; 3]> {
    let mut corners: Vec<[u64; 3]> = Vec::new();
    for &piece in pieces {
        for face in model.solid(piece).expect("this model made the key").faces() {
            for wire in face.wires() {
                for at in wire.points() {
                    corners.push(at.coordinates().map(f64::to_bits));
                }
            }
        }
    }
    corners.sort_unstable();
    corners.dedup();
    corners
}

/// A volume `measure` gave, all pieces together, and the bound it published.
struct Volume {
    volume: f64,
    bound: f64,
}

fn volume_of(model: &Model, pieces: &[SolidKey]) -> Volume {
    let how_fine = Deflection::of(0.01).expect("a positive length");
    let mut total = Volume {
        volume: 0.0,
        bound: 0.0,
    };
    for &piece in pieces {
        let it = model
            .solid(piece)
            .expect("this model made the key")
            .measure(how_fine, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("every piece of these answers measures");
        assert!(
            it.volume_error().is_finite(),
            "a bound of {} on the volume checks nothing",
            it.volume_error()
        );
        total.volume += it.volume();
        total.bound += it.volume_error();
    }
    total
}
