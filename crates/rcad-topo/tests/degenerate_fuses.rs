//! Roadmap §7.4's degeneracy catalogue, the nine entries a planar fuse can
//! reach, hand-written one case at a time (decisions.md §1.7, criterion 5).
//!
//! Every case either passes — with the volume and the counts arithmetic says
//! are right, and an empty log, because a certified fuse of an exactly
//! representable pair proves everything it decides — or refuses with a named
//! error and a comment saying why that refusal is the right answer.
//!
//! # The degeneracy is built, not approached
//!
//! Every pair here shares coordinates between the two solids, so the
//! coincidence is exact: the corner value that lands on the other solid's face
//! is the same `f64` in both inputs (§17.7). Nudging a clean pair by a
//! billionth would generate the *near* case and never the *on* case, and the on
//! case is the whole catalogue. Everything is on a small integer lattice, so
//! every plane is axis-aligned, every meet of three planes divides exactly, and
//! `Policy::certified()` can settle all of it.
//!
//! One consequence is worth stating, because it is why several entries below
//! look alike: on an axis-aligned lattice a corner of a box can only land on
//! another box's face plane if the two boxes share that plane, so a face
//! coincidence comes along with every vertex coincidence. The entries differ in
//! what else is true — whether a corner lands *inside* a face patch, or partway
//! along an edge — and that is what each one asserts.
//!
//! # The four entries that are not here
//!
//! §17.6 defers them to M4, and each needs geometry a planar fuse does not
//! have:
//!
//! - **Tangent spheres** — needs two curved surfaces touching along something.
//! - **Tangent cylinder–plane** — the tangency is a curve on a curved surface.
//! - **Seam-crossing intersections** — a seam is where a periodic surface's
//!   parameter domain joins up, and a plane has no seam.
//! - **Closed periodic surfaces** — a plane is neither closed nor periodic.
//!
//! # The ninth entry, and why it is not in this file
//!
//! Self-intersecting inputs are covered by
//! `a_solid_that_runs_through_itself_cannot_be_fused` in
//! `crates/rcad-topo/src/fuse/mod.rs`. Nothing in the public API can build a
//! solid that runs through itself — `ProfileBuilder` refuses a profile that
//! crosses itself and an extrusion of a sound profile never does — so the input
//! has to be wired together from two cubes inside the crate.

use rcad_geom::{Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule, SolidKey};

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// A cube of the given side, with its low corner at `at`.
fn cube(model: &mut Model, side: f64, at: Vector) -> SolidKey {
    block(model, [side; 3], at)
}

/// A box of the given lengths, with its low corner at `at`.
fn block(model: &mut Model, [along_x, along_y, along_z]: [f64; 3], at: Vector) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            along_z,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("building a box needs no decision")
}

/// Fuse under a certified policy and insist nothing had to be approximated.
///
/// Every pair in this file is exactly representable, so a refusal here is a
/// timid filter rather than a fact about the geometry — §17.7's control family,
/// whose expected refusal rate is zero.
fn fused(model: &mut Model, arguments: &[SolidKey], tools: &[SolidKey], rule: Rule) -> Fused {
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(arguments, tools, rule, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| {
            panic!("a lattice-aligned {rule:?} is decided by proof alone: {why}")
        });
    assert!(
        log.is_empty(),
        "a certified fuse of an exactly representable pair records nothing at all"
    );
    answer
}

/// The refusal a fuse came back with, or a panic naming what it did instead.
fn refusal(model: &mut Model, arguments: &[SolidKey], tools: &[SolidKey], rule: Rule) -> Error {
    match model.fuse(
        arguments,
        tools,
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Err(why) => why,
        Ok(fused) => panic!("expected a refusal, got {} solids", fused.solids().len()),
    }
}

/// How much space the answer takes up, all of its solids together.
fn volume_of(model: &Model, solids: &[SolidKey]) -> f64 {
    solids
        .iter()
        .map(|&solid| model.solid(solid).measure().volume())
        .sum()
}

/// Every corner of a solid, once each.
fn corners_of(model: &Model, solid: SolidKey) -> Vec<Point> {
    let mut corners = Vec::new();
    for face in model.solid(solid).faces() {
        for wire in face.wires() {
            for corner in wire.points() {
                if !corners.contains(&corner) {
                    corners.push(corner);
                }
            }
        }
    }
    corners
}

/// How much skin each face of a solid has, face by face.
fn areas_of(model: &Model, solid: SolidKey) -> Vec<f64> {
    model
        .solid(solid)
        .faces()
        .map(|face| {
            let mut vector_area = Vector::zero();
            face.for_each_triangle(|[a, b, c]| {
                vector_area = vector_area + (b - a).cross(c - a) * 0.5;
            });
            vector_area.length()
        })
        .collect()
}

/// Nothing in the answer is a piece of geometry that is not there.
///
/// §17.4 settles that degenerate pieces are *prevented* rather than filtered
/// out: a decided coincidence merges its two points into one, so an edge of no
/// length is never made in the first place, and a face of no area reaching the
/// commit is a broken invariant that panics.
fn nothing_in_it_is_degenerate(model: &Model, solids: &[SolidKey]) {
    for &solid in solids {
        for face in model.solid(solid).faces() {
            for wire in face.wires() {
                for coedge in wire.coedges() {
                    let edge = coedge.edge();
                    assert!(
                        edge.length() > 0.0,
                        "an edge from {:?} to {:?} has no length",
                        edge.start().point(),
                        edge.end().point()
                    );
                }
            }
        }
        for area in areas_of(model, solid) {
            assert!(area > 0.0, "a face of the answer covers no area at all");
        }
    }
}

// 1. Coplanar faces, sharing the whole face.

/// Two cubes of side two, one straight on top of the other. The face they share
/// is the same square in both inputs, corner for corner.
///
/// The tops and bottoms that meet point opposite ways, so the union drops both
/// and the answer is a plain 2×2×4 box: five faces from each cube, and no face
/// at all on the plane they shared.
#[test]
fn two_cubes_sharing_a_whole_face_unite_into_one_box() {
    let mut model = a_model();
    let lower = cube(&mut model, 2.0, Vector::new(0.0, 0.0, 0.0));
    let upper = cube(&mut model, 2.0, Vector::new(0.0, 0.0, 2.0));

    let united = fused(&mut model, &[lower], &[upper], Rule::Union);

    assert_eq!(united.solids().len(), 1);
    let solid = model.solid(united.solids()[0]);
    assert_eq!(solid.shells().count(), 1);
    assert_eq!(
        solid.faces().count(),
        10,
        "a bottom, a top, and four walls in two halves each"
    );
    assert_eq!(volume_of(&model, united.solids()), 16.0);
    nothing_in_it_is_degenerate(&model, united.solids());
}

/// The same pair intersected. They share a square of skin and no space at all,
/// and an empty answer is a success (§17.4) — the caller asked a well-formed
/// question and the answer is that there is nothing in both.
#[test]
fn two_cubes_sharing_a_whole_face_have_nothing_in_both() {
    let mut model = a_model();
    let lower = cube(&mut model, 2.0, Vector::new(0.0, 0.0, 0.0));
    let upper = cube(&mut model, 2.0, Vector::new(0.0, 0.0, 2.0));

    let shared = fused(&mut model, &[lower], &[upper], Rule::Intersection);

    assert!(shared.solids().is_empty());
}

/// Subtracting one from the other takes nothing away. The shared face is ON,
/// facing opposite ways, and difference keeps that once — the argument's own
/// copy — so the lower cube comes back whole.
#[test]
fn taking_a_cube_off_the_face_of_another_takes_nothing_away() {
    let mut model = a_model();
    let lower = cube(&mut model, 2.0, Vector::new(0.0, 0.0, 0.0));
    let upper = cube(&mut model, 2.0, Vector::new(0.0, 0.0, 2.0));

    let bitten = fused(&mut model, &[lower], &[upper], Rule::Difference);

    assert_eq!(bitten.solids().len(), 1);
    assert_eq!(model.solid(bitten.solids()[0]).faces().count(), 6);
    assert_eq!(volume_of(&model, bitten.solids()), 8.0);
}

// 1. Coplanar faces, overlapping only in part.

/// Two 4×4×2 slabs, the upper one slid halfway along. The faces that meet are
/// on the same plane and share only the strip `2 ≤ x ≤ 4`: the rest of each
/// stays on the outside of the answer.
#[test]
fn two_slabs_whose_faces_are_coplanar_and_half_overlap_unite() {
    let mut model = a_model();
    let lower = block(&mut model, [4.0, 4.0, 2.0], Vector::new(0.0, 0.0, 0.0));
    let upper = block(&mut model, [4.0, 4.0, 2.0], Vector::new(2.0, 0.0, 2.0));

    let united = fused(&mut model, &[lower], &[upper], Rule::Union);

    assert_eq!(united.solids().len(), 1);
    let solid = model.solid(united.solids()[0]);
    assert_eq!(solid.shells().count(), 1);
    assert_eq!(volume_of(&model, united.solids()), 64.0);
    nothing_in_it_is_degenerate(&model, united.solids());
}

/// The same pair, and again they enclose no space between them.
#[test]
fn two_slabs_that_only_share_part_of_a_face_have_nothing_in_both() {
    let mut model = a_model();
    let lower = block(&mut model, [4.0, 4.0, 2.0], Vector::new(0.0, 0.0, 0.0));
    let upper = block(&mut model, [4.0, 4.0, 2.0], Vector::new(2.0, 0.0, 2.0));

    let shared = fused(&mut model, &[lower], &[upper], Rule::Intersection);

    assert!(shared.solids().is_empty());
    let bitten = fused(&mut model, &[lower], &[upper], Rule::Difference);
    assert_eq!(volume_of(&model, bitten.solids()), 32.0);
}

// 2. Coincident edges.

/// Two cubes catty-corner from each other, sharing one whole edge and nothing
/// else. Four faces would run along that edge in the union, and a `Model` means
/// an edge is used by exactly two — relaxing that changes what a model *is*,
/// which is not work a boolean gets to do on its way past (§17.4).
///
/// This is a refusal that is right rather than a bug: the union of those two
/// cubes genuinely is not a solid this kernel can hold.
#[test]
fn two_cubes_sharing_one_whole_edge_cannot_be_united() {
    let mut model = a_model();
    let one = cube(&mut model, 2.0, Vector::new(0.0, 0.0, 0.0));
    let other = cube(&mut model, 2.0, Vector::new(2.0, 2.0, 0.0));

    match refusal(&mut model, &[one], &[other], Rule::Union) {
        Error::ResultIsNotManifold { count } => {
            assert_eq!(count, 4, "both cubes claim that edge with two faces each")
        }
        other => panic!("that union is not a solid this kernel can hold: {other:?}"),
    }
}

/// The other two rules over the same pair answer without trouble: an edge is
/// not any space, so there is nothing in both, and subtracting takes nothing
/// away.
#[test]
fn two_cubes_sharing_one_whole_edge_still_intersect_and_subtract() {
    let mut model = a_model();
    let one = cube(&mut model, 2.0, Vector::new(0.0, 0.0, 0.0));
    let other = cube(&mut model, 2.0, Vector::new(2.0, 2.0, 0.0));

    let shared = fused(&mut model, &[one], &[other], Rule::Intersection);
    assert!(shared.solids().is_empty());

    let bitten = fused(&mut model, &[one], &[other], Rule::Difference);
    assert_eq!(bitten.solids().len(), 1);
    assert_eq!(model.solid(bitten.solids()[0]).faces().count(), 6);
    assert_eq!(volume_of(&model, bitten.solids()), 8.0);
}

// 3. Vertex on face.

/// Two cubes of side four, the second slid two along in `x` and two in `y`. The
/// corners `(2, 2, 0)` and `(2, 2, 4)` of the second land exactly in the middle
/// of the first one's bottom and top faces — not on an edge of them, in the
/// interior — and the answer has to carry those points as corners of its own.
#[test]
fn a_corner_landing_inside_a_face_survives_as_a_corner_of_the_answer() {
    let mut model = a_model();
    let one = cube(&mut model, 4.0, Vector::new(0.0, 0.0, 0.0));
    let other = cube(&mut model, 4.0, Vector::new(2.0, 2.0, 0.0));

    let united = fused(&mut model, &[one], &[other], Rule::Union);

    assert_eq!(united.solids().len(), 1);
    assert_eq!(volume_of(&model, united.solids()), 112.0);
    let corners = corners_of(&model, united.solids()[0]);
    for landed in [Point::new(2.0, 2.0, 0.0), Point::new(2.0, 2.0, 4.0)] {
        assert!(
            corners.contains(&landed),
            "the corner that landed in the middle of a face is a corner of the answer too"
        );
    }
    nothing_in_it_is_degenerate(&model, united.solids());
}

/// What the two of them share is the 2×2×4 column standing on the corner that
/// landed: a plain box, six faces, nothing left over.
#[test]
fn what_two_cubes_offset_by_a_corner_share_is_the_column_under_it() {
    let mut model = a_model();
    let one = cube(&mut model, 4.0, Vector::new(0.0, 0.0, 0.0));
    let other = cube(&mut model, 4.0, Vector::new(2.0, 2.0, 0.0));

    let shared = fused(&mut model, &[one], &[other], Rule::Intersection);

    assert_eq!(shared.solids().len(), 1);
    let solid = model.solid(shared.solids()[0]);
    assert_eq!(solid.shells().count(), 1);
    assert_eq!(solid.faces().count(), 6, "the shared part is a box");
    assert_eq!(volume_of(&model, shared.solids()), 16.0);

    let bitten = fused(&mut model, &[one], &[other], Rule::Difference);
    assert_eq!(volume_of(&model, bitten.solids()), 48.0);
}

// 4. Vertex on edge.

/// A 3×3×4 box sitting in the corner of a cube of side four and poking out
/// through the top. Its corner `(4, 4, 2)` lands exactly partway along the
/// cube's vertical edge at `x = y = 4`, which the answer has to split in two.
#[test]
fn a_corner_landing_partway_along_an_edge_splits_that_edge_in_two() {
    let mut model = a_model();
    let one = cube(&mut model, 4.0, Vector::new(0.0, 0.0, 0.0));
    let other = block(&mut model, [3.0, 3.0, 4.0], Vector::new(1.0, 1.0, 2.0));

    let united = fused(&mut model, &[one], &[other], Rule::Union);

    assert_eq!(united.solids().len(), 1);
    assert_eq!(volume_of(&model, united.solids()), 64.0 + 36.0 - 18.0);
    let corners = corners_of(&model, united.solids()[0]);
    assert!(
        corners.contains(&Point::new(4.0, 4.0, 2.0)),
        "the corner that landed on the edge is a corner of the answer"
    );
    nothing_in_it_is_degenerate(&model, united.solids());
}

/// The same pair under the other two rules. Both are ordinary answers — the
/// corner sitting on the edge changes the topology, not the arithmetic.
#[test]
fn a_corner_on_an_edge_still_intersects_and_subtracts() {
    let mut model = a_model();
    let one = cube(&mut model, 4.0, Vector::new(0.0, 0.0, 0.0));
    let other = block(&mut model, [3.0, 3.0, 4.0], Vector::new(1.0, 1.0, 2.0));

    let shared = fused(&mut model, &[one], &[other], Rule::Intersection);
    assert_eq!(shared.solids().len(), 1);
    assert_eq!(volume_of(&model, shared.solids()), 18.0);

    let bitten = fused(&mut model, &[one], &[other], Rule::Difference);
    assert_eq!(bitten.solids().len(), 1);
    assert_eq!(volume_of(&model, bitten.solids()), 46.0);
}

// 5. Zero-length edges.

/// Two tools with walls on the very same planes, cutting one cube. Every face
/// of the cube is handed the plane `x = 3` twice over, once from each tool, and
/// the same for `x = 7`, `y = 1` and `y = 3`.
///
/// A splitter that cut by each line in turn would put two corners at the same
/// point on every one of those faces and join them with an edge of no length.
/// §17.4 settles that this is prevented rather than filtered: the coincidence is
/// decided and the two points merge. So what this asserts is that the merge
/// happened — no edge of the answer has no length, and no face of it covers no
/// area — and not that a filter ran afterwards.
#[test]
fn two_tools_cutting_along_the_same_planes_merge_instead_of_making_an_edge_of_no_length() {
    let mut model = a_model();
    let cut = cube(&mut model, 6.0, Vector::new(0.0, 0.0, 0.0));
    let lower = block(&mut model, [4.0, 2.0, 1.0], Vector::new(3.0, 1.0, 1.0));
    let upper = block(&mut model, [4.0, 2.0, 1.0], Vector::new(3.0, 1.0, 4.0));

    let united = fused(&mut model, &[cut], &[lower, upper], Rule::Union);

    assert_eq!(united.solids().len(), 1);
    assert_eq!(
        volume_of(&model, united.solids()),
        216.0 + 2.0 + 2.0,
        "the cube, plus the one-by-two-by-one of each bar that pokes out of it"
    );
    nothing_in_it_is_degenerate(&model, united.solids());
}

// 6. Sliver faces.

/// A cut a millionth of a millimetre in from a face of a cube of side four.
/// What is left in both is a slab 2⁻²⁰ thick, and four of its six faces are
/// slivers 2⁻²⁰ across and four long.
///
/// A power of two, so the thin coordinate is exactly representable and the
/// answer stays a lattice case: this is a sliver, not a rounding.
///
/// §12.3 is what this is here to stop happening again: M0 twice refused a
/// sliver that was certifiably not degenerate, and it took an exhaustive search
/// over 160,179 rings to find out. So this asserts the slivers are **kept** and
/// that each of them covers a positive area, rather than asserting anything was
/// cleaned up.
#[test]
fn a_cut_a_millionth_of_the_way_in_keeps_the_sliver_it_leaves() {
    let a_sliver = 2.0f64.powi(-20);
    let mut model = a_model();
    let solid = cube(&mut model, 4.0, Vector::new(0.0, 0.0, 0.0));
    let knife = block(
        &mut model,
        [4.0, 6.0, 6.0],
        Vector::new(4.0 - a_sliver, -1.0, -1.0),
    );

    let shared = fused(&mut model, &[solid], &[knife], Rule::Intersection);

    assert_eq!(shared.solids().len(), 1);
    let slab = shared.solids()[0];
    assert_eq!(model.solid(slab).faces().count(), 6);
    assert_eq!(volume_of(&model, shared.solids()), 16.0 * a_sliver);

    let mut areas = areas_of(&model, slab);
    areas.sort_by(f64::total_cmp);
    assert_eq!(
        areas,
        vec![
            4.0 * a_sliver,
            4.0 * a_sliver,
            4.0 * a_sliver,
            4.0 * a_sliver,
            16.0,
            16.0
        ],
        "four slivers, kept, each with an area that is small and not zero"
    );
    nothing_in_it_is_degenerate(&model, shared.solids());
}

// 7. Fully contained solids.

/// A cube of side two in the middle of a cube of side four, under all three
/// rules. The union is the outer cube and the inner one's faces have no image
/// at all; the intersection is the inner cube; the difference is the outer one
/// with a bubble in it, which is one solid with two shells.
#[test]
fn a_cube_wholly_inside_another_under_all_three_rules() {
    let mut model = a_model();
    let outer = cube(&mut model, 4.0, Vector::new(0.0, 0.0, 0.0));
    let inner = cube(&mut model, 2.0, Vector::new(1.0, 1.0, 1.0));

    let united = fused(&mut model, &[outer], &[inner], Rule::Union);
    assert_eq!(united.solids().len(), 1);
    assert_eq!(model.solid(united.solids()[0]).shells().count(), 1);
    assert_eq!(
        model.solid(united.solids()[0]).faces().count(),
        54,
        "a cut is by a plane, so the inner cube's four walls cross all six faces of the outer one \
         and each comes back in nine pieces"
    );
    assert_eq!(volume_of(&model, united.solids()), 64.0);
    assert_eq!(
        united.deleted_faces().count(),
        6,
        "every face of the inner cube is swallowed"
    );

    let shared = fused(&mut model, &[outer], &[inner], Rule::Intersection);
    assert_eq!(shared.solids().len(), 1);
    assert_eq!(model.solid(shared.solids()[0]).shells().count(), 1);
    assert_eq!(model.solid(shared.solids()[0]).faces().count(), 6);
    assert_eq!(volume_of(&model, shared.solids()), 8.0);

    let hollow = fused(&mut model, &[outer], &[inner], Rule::Difference);
    assert_eq!(hollow.solids().len(), 1);
    assert_eq!(
        model.solid(hollow.solids()[0]).shells().count(),
        2,
        "the skin of the block and the skin of the bubble in it"
    );
    assert_eq!(volume_of(&model, hollow.solids()), 56.0);
    nothing_in_it_is_degenerate(&model, hollow.solids());
}

/// The containment the other way round. Taking the block away from the bubble
/// leaves nothing, and nothing is an answer.
#[test]
fn taking_a_block_away_from_a_cube_inside_it_leaves_nothing() {
    let mut model = a_model();
    let outer = cube(&mut model, 4.0, Vector::new(0.0, 0.0, 0.0));
    let inner = cube(&mut model, 2.0, Vector::new(1.0, 1.0, 1.0));

    let nothing = fused(&mut model, &[inner], &[outer], Rule::Difference);

    assert!(nothing.solids().is_empty());
}

// 8. Touching but not overlapping — on a face, on an edge, at a corner.

/// A small cube standing on the top face of a big one, its whole footprint
/// inside that face. They share skin and no space.
#[test]
fn two_cubes_that_only_touch_on_a_face() {
    let mut model = a_model();
    let big = cube(&mut model, 4.0, Vector::new(0.0, 0.0, 0.0));
    let small = cube(&mut model, 2.0, Vector::new(1.0, 1.0, 4.0));

    let united = fused(&mut model, &[big], &[small], Rule::Union);
    assert_eq!(
        united.solids().len(),
        1,
        "a shared face joins them into one solid"
    );
    assert_eq!(model.solid(united.solids()[0]).shells().count(), 1);
    assert_eq!(volume_of(&model, united.solids()), 72.0);
    nothing_in_it_is_degenerate(&model, united.solids());

    let shared = fused(&mut model, &[big], &[small], Rule::Intersection);
    assert!(shared.solids().is_empty());

    let bitten = fused(&mut model, &[big], &[small], Rule::Difference);
    assert_eq!(volume_of(&model, bitten.solids()), 64.0);
}

/// A small cube touching a big one along part of one vertical edge, and nowhere
/// else. Four faces run along that stretch, so the union is refused for the
/// same reason as the two catty-corner cubes above — and this time the shared
/// stretch is only part of the big cube's edge, so the refusal does not depend
/// on the two edges being the same edge.
#[test]
fn two_cubes_that_only_touch_along_part_of_an_edge() {
    let mut model = a_model();
    let big = cube(&mut model, 4.0, Vector::new(0.0, 0.0, 0.0));
    let small = cube(&mut model, 2.0, Vector::new(4.0, 4.0, 1.0));

    match refusal(&mut model, &[big], &[small], Rule::Union) {
        Error::ResultIsNotManifold { count } => assert_eq!(count, 4),
        other => panic!("two solids joined along an edge are not one solid: {other:?}"),
    }

    let shared = fused(&mut model, &[big], &[small], Rule::Intersection);
    assert!(shared.solids().is_empty());

    let bitten = fused(&mut model, &[big], &[small], Rule::Difference);
    assert_eq!(volume_of(&model, bitten.solids()), 64.0);
}

/// Two cubes that meet at exactly one point. No edge of either is claimed more
/// than twice, so the manifold check has nothing to say; the two never share an
/// edge either, so the walk that groups faces into shells never joins them, and
/// the answer is two solids that happen to touch.
#[test]
fn two_cubes_that_only_touch_at_a_corner() {
    let mut model = a_model();
    let one = cube(&mut model, 2.0, Vector::new(0.0, 0.0, 0.0));
    let other = cube(&mut model, 2.0, Vector::new(2.0, 2.0, 2.0));

    let united = fused(&mut model, &[one], &[other], Rule::Union);
    assert_eq!(
        united.solids().len(),
        2,
        "one point of contact joins no faces, so they stay two"
    );
    assert_eq!(volume_of(&model, united.solids()), 16.0);
    nothing_in_it_is_degenerate(&model, united.solids());

    let shared = fused(&mut model, &[one], &[other], Rule::Intersection);
    assert!(shared.solids().is_empty());

    let bitten = fused(&mut model, &[one], &[other], Rule::Difference);
    assert_eq!(volume_of(&model, bitten.solids()), 8.0);
}
