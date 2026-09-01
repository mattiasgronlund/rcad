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
//! has to be wired together from two cubes inside the crate. It still gets a row
//! in [`the_nine_planar_entries_as_a_table`], naming where it is discharged, so
//! that the table is nine entries and not eight.

use rcad_geom::{Deflection, Isometry, PlaneAgreement, Point, Scale, Segment, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Fused, Model, ProfileBuilder, Rule, SolidKey};

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

/// The two sides of a fuse: what is being cut, and what is cutting it.
///
/// Every entry of the catalogue is built by one of these, and both the case's
/// own test and [`the_nine_planar_entries_as_a_table`] go through it. So the
/// table §17.7 carries cannot describe a pair different from the one the
/// assertions are about.
struct Pair {
    arguments: Vec<SolidKey>,
    tools: Vec<SolidKey>,
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
        .map(|&solid| {
            model
                .solid(solid)
                .measure(
                    a_deflection(),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a solid this size measures")
                .volume()
        })
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

/// Every edge of a solid, once each.
fn edges_of(model: &Model, solid: SolidKey) -> Vec<Segment> {
    let mut seen = Vec::new();
    let mut edges = Vec::new();
    for face in model.solid(solid).faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if !seen.contains(&edge.key()) {
                    seen.push(edge.key());
                    edges.push(edge.segment());
                }
            }
        }
    }
    edges
}

/// The edges of a solid that run through a point, ends included.
fn edges_through(model: &Model, solid: SolidKey, point: Point) -> Vec<Segment> {
    edges_of(model, solid)
        .into_iter()
        .filter(|edge| edge.closest_point(point) == point)
        .collect()
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
                        edge.distance_between_its_ends() > 0.0,
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

/// No face of the answer is half of a face.
///
/// Two faces on one plane, facing one way, with an edge between them are one
/// flat region cut in two, and §17.4's amendment settles that the commit joins
/// them back up. That is what leaves three planes round every corner of the
/// answer, which is what lets §17.1 re-derive that corner when the answer is
/// fused again — so this is the property the whole partition rests on, and
/// nothing else here looks at it directly.
///
/// A face that runs up against *itself* along an edge is still bounded there,
/// so only faces that are two different faces are compared.
fn no_face_of_it_is_half_of_another(model: &Model, solids: &[SolidKey]) {
    for &solid in solids {
        for face in model.solid(solid).faces() {
            for wire in face.wires() {
                for coedge in wire.coedges() {
                    for neighbour in coedge.edge().neighbouring_faces() {
                        if neighbour == face.key() {
                            continue;
                        }
                        assert_ne!(
                            face.plane().agreement_with(&model.face(neighbour).plane()),
                            PlaneAgreement::SameWay,
                            "two faces sharing an edge lie on one plane facing one way, \
                             so they are one face and should have been joined"
                        );
                    }
                }
            }
        }
    }
}

// 1. Coplanar faces, sharing the whole face.

/// Two cubes of side two, one straight on top of the other. The face they share
/// is the same square in both inputs, corner for corner.
fn two_cubes_sharing_a_whole_face(model: &mut Model) -> Pair {
    Pair {
        arguments: vec![cube(model, 2.0, Vector::new(0.0, 0.0, 0.0))],
        tools: vec![cube(model, 2.0, Vector::new(0.0, 0.0, 2.0))],
    }
}

/// The tops and bottoms that meet point opposite ways, so the union drops both
/// and the answer is a plain 2×2×4 box. Each of its four walls arrives in two
/// halves, one from each cube, and the two halves are one plane facing one way
/// with an edge between them — which §17.4's amendment joins into the one wall
/// they are. Ten faces was the count of surviving *pieces*, which described the
/// splitter's leftovers; the shape the test is named for has six.
#[test]
fn two_cubes_sharing_a_whole_face_unite_into_one_box() {
    let mut model = a_model();
    let pair = two_cubes_sharing_a_whole_face(&mut model);

    let united = fused(&mut model, &pair.arguments, &pair.tools, Rule::Union);

    assert_eq!(united.solids().len(), 1);
    let solid = united.solids()[0];
    assert_eq!(model.solid(solid).shells().count(), 1);
    assert_eq!(model.solid(solid).faces().count(), 6, "a box has six faces");
    let mut areas = areas_of(&model, solid);
    areas.sort_by(f64::total_cmp);
    assert_eq!(
        areas,
        vec![4.0, 4.0, 8.0, 8.0, 8.0, 8.0],
        "two 2×2 ends and four walls that are 2×4 each, not eight halves of 2×2"
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
    let pair = two_cubes_sharing_a_whole_face(&mut model);

    let shared = fused(&mut model, &pair.arguments, &pair.tools, Rule::Intersection);

    assert!(shared.solids().is_empty());
}

/// Subtracting one from the other takes nothing away. The shared face is ON,
/// facing opposite ways, and difference keeps that once — the argument's own
/// copy — so the lower cube comes back whole.
#[test]
fn taking_a_cube_off_the_face_of_another_takes_nothing_away() {
    let mut model = a_model();
    let pair = two_cubes_sharing_a_whole_face(&mut model);

    let bitten = fused(&mut model, &pair.arguments, &pair.tools, Rule::Difference);

    assert_eq!(bitten.solids().len(), 1);
    assert_eq!(model.solid(bitten.solids()[0]).faces().count(), 6);
    assert_eq!(volume_of(&model, bitten.solids()), 8.0);
}

// 1. Coplanar faces, overlapping only in part.

/// Two 4×4×2 slabs, the upper one slid halfway along. The faces that meet are
/// on the same plane and share only the strip `2 ≤ x ≤ 4`: the rest of each
/// stays on the outside of the answer.
fn two_slabs_whose_faces_half_overlap(model: &mut Model) -> Pair {
    Pair {
        arguments: vec![block(model, [4.0, 4.0, 2.0], Vector::new(0.0, 0.0, 0.0))],
        tools: vec![block(model, [4.0, 4.0, 2.0], Vector::new(2.0, 0.0, 2.0))],
    }
}

#[test]
fn two_slabs_whose_faces_are_coplanar_and_half_overlap_unite() {
    let mut model = a_model();
    let pair = two_slabs_whose_faces_half_overlap(&mut model);

    let united = fused(&mut model, &pair.arguments, &pair.tools, Rule::Union);

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
    let pair = two_slabs_whose_faces_half_overlap(&mut model);

    let shared = fused(&mut model, &pair.arguments, &pair.tools, Rule::Intersection);

    assert!(shared.solids().is_empty());
    let bitten = fused(&mut model, &pair.arguments, &pair.tools, Rule::Difference);
    assert_eq!(volume_of(&model, bitten.solids()), 32.0);
}

// 2. Coincident edges.

/// Two cubes catty-corner from each other, sharing one whole edge and nothing
/// else.
fn two_cubes_sharing_one_whole_edge(model: &mut Model) -> Pair {
    Pair {
        arguments: vec![cube(model, 2.0, Vector::new(0.0, 0.0, 0.0))],
        tools: vec![cube(model, 2.0, Vector::new(2.0, 2.0, 0.0))],
    }
}

/// Four faces would run along that shared edge in the union, and a `Model` means
/// an edge is used by exactly two — relaxing that changes what a model *is*,
/// which is not work a boolean gets to do on its way past (§17.4).
///
/// This is a refusal that is right rather than a bug: the union of those two
/// cubes genuinely is not a solid this kernel can hold.
#[test]
fn two_cubes_sharing_one_whole_edge_cannot_be_united() {
    let mut model = a_model();
    let pair = two_cubes_sharing_one_whole_edge(&mut model);

    match refusal(&mut model, &pair.arguments, &pair.tools, Rule::Union) {
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
    let pair = two_cubes_sharing_one_whole_edge(&mut model);

    let shared = fused(&mut model, &pair.arguments, &pair.tools, Rule::Intersection);
    assert!(shared.solids().is_empty());

    let bitten = fused(&mut model, &pair.arguments, &pair.tools, Rule::Difference);
    assert_eq!(bitten.solids().len(), 1);
    assert_eq!(model.solid(bitten.solids()[0]).faces().count(), 6);
    assert_eq!(volume_of(&model, bitten.solids()), 8.0);
}

// 3. Vertex on face.

/// Two cubes of side four, the second slid two along in `x` and two in `y`. The
/// corners `(2, 2, 0)` and `(2, 2, 4)` of the second land exactly in the middle
/// of the first one's bottom and top faces — not on an edge of them, in the
/// interior.
fn two_cubes_offset_by_a_corner(model: &mut Model) -> Pair {
    Pair {
        arguments: vec![cube(model, 4.0, Vector::new(0.0, 0.0, 0.0))],
        tools: vec![cube(model, 4.0, Vector::new(2.0, 2.0, 0.0))],
    }
}

/// Whether the fuse *copes* with that is the catalogue entry; whether the point
/// stays a corner is a different question, and the answer to it is no. Under
/// §17.1 a face is its plane, and the two bottom squares are one flat region
/// with one plane over it, so the answer's bottom is one face and the landed
/// point is in the middle of it. Keeping it as a corner would put four coplanar
/// faces round one point, and a point like that has two planes round it and not
/// three, so the *next* fuse over the answer could not re-derive it at all
/// (§17.4, amended).
///
/// So the claim is that the coincidence was decided and proved, the volume is
/// exactly right, the answer is a model in good standing, and the corner is
/// absorbed rather than lost: still on the skin of the answer, on none of its
/// edges and none of its corners, which places it strictly inside a face.
#[test]
fn a_corner_landing_inside_a_face_is_absorbed_into_that_face() {
    let mut model = a_model();
    let pair = two_cubes_offset_by_a_corner(&mut model);

    let united = fused(&mut model, &pair.arguments, &pair.tools, Rule::Union);

    assert_eq!(united.solids().len(), 1);
    let solid = united.solids()[0];
    assert_eq!(model.solid(solid).shells().count(), 1);
    assert_eq!(volume_of(&model, united.solids()), 112.0);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse writes a sound model or writes nothing");

    let corners = corners_of(&model, solid);
    for landed in [Point::new(2.0, 2.0, 0.0), Point::new(2.0, 2.0, 4.0)] {
        assert_eq!(
            model.solid(solid).classify_point(
                landed,
                &Policy::certified(),
                &mut ProvenanceLog::new()
            ),
            Containment::OnBoundary,
            "the corner that landed on a face is still on the skin of the answer"
        );
        assert!(
            !corners.contains(&landed),
            "a corner in the middle of a flat region is not a corner of anything"
        );
        assert!(
            edges_through(&model, solid, landed).is_empty(),
            "it is not on the boundary of that face either, it is inside it"
        );
    }
    no_face_of_it_is_half_of_another(&model, united.solids());
    nothing_in_it_is_degenerate(&model, united.solids());
}

/// What the two of them share is the 2×2×4 column standing on the corner that
/// landed: a plain box, six faces, nothing left over.
#[test]
fn what_two_cubes_offset_by_a_corner_share_is_the_column_under_it() {
    let mut model = a_model();
    let pair = two_cubes_offset_by_a_corner(&mut model);

    let shared = fused(&mut model, &pair.arguments, &pair.tools, Rule::Intersection);

    assert_eq!(shared.solids().len(), 1);
    let solid = model.solid(shared.solids()[0]);
    assert_eq!(solid.shells().count(), 1);
    assert_eq!(solid.faces().count(), 6, "the shared part is a box");
    assert_eq!(volume_of(&model, shared.solids()), 16.0);

    let bitten = fused(&mut model, &pair.arguments, &pair.tools, Rule::Difference);
    assert_eq!(volume_of(&model, bitten.solids()), 48.0);
}

// 4. Vertex on edge.

/// A 3×3×4 box sitting in the corner of a cube of side four and poking out
/// through the top. Its corner `(4, 4, 2)` lands exactly partway along the
/// cube's vertical edge at `x = y = 4`.
fn a_box_poking_out_of_the_top_of_a_cube(model: &mut Model) -> Pair {
    Pair {
        arguments: vec![cube(model, 4.0, Vector::new(0.0, 0.0, 0.0))],
        tools: vec![block(model, [3.0, 3.0, 4.0], Vector::new(1.0, 1.0, 2.0))],
    }
}

/// The two solids are flush on `x = 4` and flush again on `y = 4`, so each of
/// those two pairs of pieces is one plane facing one way and joins into one
/// face. The edge between the two joined faces then runs the whole way from
/// `(4, 4, 0)` up to `(4, 4, 6)`, straight, with the landed point in the middle
/// of it — a point with two faces round it, which §17.4's amendment drops,
/// making the two edges either side of it one edge.
///
/// So the claim is that the coincidence was decided and proved, the volume is
/// exactly right, the answer is a model in good standing, and the corner is
/// absorbed into that one edge rather than lost.
#[test]
fn a_corner_landing_partway_along_an_edge_is_absorbed_into_one_straight_edge() {
    let mut model = a_model();
    let pair = a_box_poking_out_of_the_top_of_a_cube(&mut model);

    let united = fused(&mut model, &pair.arguments, &pair.tools, Rule::Union);

    assert_eq!(united.solids().len(), 1);
    let solid = united.solids()[0];
    assert_eq!(volume_of(&model, united.solids()), 64.0 + 36.0 - 18.0);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse writes a sound model or writes nothing");

    let landed = Point::new(4.0, 4.0, 2.0);
    assert_eq!(
        model
            .solid(solid)
            .classify_point(landed, &Policy::certified(), &mut ProvenanceLog::new()),
        Containment::OnBoundary,
        "the corner that landed on an edge is still on the skin of the answer"
    );
    assert!(
        !corners_of(&model, solid).contains(&landed),
        "a corner in the middle of a straight edge is not a corner of anything"
    );
    let through = edges_through(&model, solid, landed);
    assert_eq!(
        through.len(),
        1,
        "one edge runs through it, not two meeting"
    );
    let (bottom, top) = (Point::new(4.0, 4.0, 0.0), Point::new(4.0, 4.0, 6.0));
    let ends = [through[0].start(), through[0].end()];
    assert!(
        ends == [bottom, top] || ends == [top, bottom],
        "that edge is the whole corner of the answer, {bottom:?} to {top:?}, but it runs {ends:?}"
    );
    nothing_in_it_is_degenerate(&model, united.solids());
}

/// The same pair under the other two rules. Both are ordinary answers — the
/// corner sitting on the edge changes the topology, not the arithmetic.
#[test]
fn a_corner_on_an_edge_still_intersects_and_subtracts() {
    let mut model = a_model();
    let pair = a_box_poking_out_of_the_top_of_a_cube(&mut model);

    let shared = fused(&mut model, &pair.arguments, &pair.tools, Rule::Intersection);
    assert_eq!(shared.solids().len(), 1);
    assert_eq!(volume_of(&model, shared.solids()), 18.0);

    let bitten = fused(&mut model, &pair.arguments, &pair.tools, Rule::Difference);
    assert_eq!(bitten.solids().len(), 1);
    assert_eq!(volume_of(&model, bitten.solids()), 46.0);
}

// 5. Zero-length edges.

/// Two tools with walls on the very same planes, cutting one cube. Every face
/// of the cube is handed the plane `x = 3` twice over, once from each tool, and
/// the same for `x = 7`, `y = 1` and `y = 3`.
fn a_cube_and_two_bars_on_the_same_planes(model: &mut Model) -> Pair {
    Pair {
        arguments: vec![cube(model, 6.0, Vector::new(0.0, 0.0, 0.0))],
        tools: vec![
            block(model, [4.0, 2.0, 1.0], Vector::new(3.0, 1.0, 1.0)),
            block(model, [4.0, 2.0, 1.0], Vector::new(3.0, 1.0, 4.0)),
        ],
    }
}

/// A splitter that cut by each line in turn would put two corners at the same
/// point on every one of those faces and join them with an edge of no length.
/// §17.4 settles that this is prevented rather than filtered: the coincidence is
/// decided and the two points merge. So what this asserts is that the merge
/// happened — no edge of the answer has no length, and no face of it covers no
/// area — and not that a filter ran afterwards.
#[test]
fn two_tools_cutting_along_the_same_planes_merge_instead_of_making_an_edge_of_no_length() {
    let mut model = a_model();
    let pair = a_cube_and_two_bars_on_the_same_planes(&mut model);

    let united = fused(&mut model, &pair.arguments, &pair.tools, Rule::Union);

    assert_eq!(united.solids().len(), 1);
    assert_eq!(
        volume_of(&model, united.solids()),
        216.0 + 2.0 + 2.0,
        "the cube, plus the one-by-two-by-one of each bar that pokes out of it"
    );
    nothing_in_it_is_degenerate(&model, united.solids());
}

// 6. Sliver faces.

/// How thick the slab a sliver cut leaves is.
///
/// A power of two, so the thin coordinate is exactly representable and the
/// answer stays a lattice case: this is a sliver, not a rounding.
const A_SLIVER: f64 = 1.0 / (1 << 20) as f64;

/// A cut a millionth of a millimetre in from a face of a cube of side four.
/// What is left in both is a slab [`A_SLIVER`] thick, and four of its six faces
/// are slivers that thin and four long.
fn a_cube_and_a_knife_a_sliver_in(model: &mut Model) -> Pair {
    Pair {
        arguments: vec![cube(model, 4.0, Vector::new(0.0, 0.0, 0.0))],
        tools: vec![block(
            model,
            [4.0, 6.0, 6.0],
            Vector::new(4.0 - A_SLIVER, -1.0, -1.0),
        )],
    }
}

/// §12.3 is what this is here to stop happening again: M0 twice refused a
/// sliver that was certifiably not degenerate, and it took an exhaustive search
/// over 160,179 rings to find out. So this asserts the slivers are **kept** and
/// that each of them covers a positive area, rather than asserting anything was
/// cleaned up.
#[test]
fn a_cut_a_millionth_of_the_way_in_keeps_the_sliver_it_leaves() {
    let mut model = a_model();
    let pair = a_cube_and_a_knife_a_sliver_in(&mut model);

    let shared = fused(&mut model, &pair.arguments, &pair.tools, Rule::Intersection);

    assert_eq!(shared.solids().len(), 1);
    let slab = shared.solids()[0];
    assert_eq!(model.solid(slab).faces().count(), 6);
    assert_eq!(volume_of(&model, shared.solids()), 16.0 * A_SLIVER);

    let mut areas = areas_of(&model, slab);
    areas.sort_by(f64::total_cmp);
    assert_eq!(
        areas,
        vec![
            4.0 * A_SLIVER,
            4.0 * A_SLIVER,
            4.0 * A_SLIVER,
            4.0 * A_SLIVER,
            16.0,
            16.0
        ],
        "four slivers, kept, each with an area that is small and not zero"
    );
    nothing_in_it_is_degenerate(&model, shared.solids());
}

// 7. Fully contained solids.

/// A cube of side two in the middle of a cube of side four.
fn a_cube_wholly_inside_another(model: &mut Model) -> Pair {
    Pair {
        arguments: vec![cube(model, 4.0, Vector::new(0.0, 0.0, 0.0))],
        tools: vec![cube(model, 2.0, Vector::new(1.0, 1.0, 1.0))],
    }
}

/// The union is the outer cube and the inner one's faces have no image at all;
/// the intersection is the inner cube; the difference is the outer one with a
/// bubble in it, which is one solid with two shells.
#[test]
fn a_cube_wholly_inside_another_under_all_three_rules() {
    let mut model = a_model();
    let pair = a_cube_wholly_inside_another(&mut model);

    let united = fused(&mut model, &pair.arguments, &pair.tools, Rule::Union);
    assert_eq!(united.solids().len(), 1);
    assert_eq!(model.solid(united.solids()[0]).shells().count(), 1);
    assert_eq!(
        model.solid(united.solids()[0]).faces().count(),
        6,
        "a cut is by a plane, so the inner cube's four walls do cross all six faces of the outer \
         one and each comes back in nine pieces — but a cut where the solid it belongs to is \
         nowhere near is not boundary of anything, and §17.4's join puts all nine back together"
    );
    assert_eq!(
        areas_of(&model, united.solids()[0]),
        vec![16.0; 6],
        "the six faces are the outer cube's own, whole"
    );
    assert_eq!(volume_of(&model, united.solids()), 64.0);
    assert_eq!(
        united.deleted_faces().count(),
        6,
        "every face of the inner cube is swallowed"
    );

    let shared = fused(&mut model, &pair.arguments, &pair.tools, Rule::Intersection);
    assert_eq!(shared.solids().len(), 1);
    assert_eq!(model.solid(shared.solids()[0]).shells().count(), 1);
    assert_eq!(model.solid(shared.solids()[0]).faces().count(), 6);
    assert_eq!(volume_of(&model, shared.solids()), 8.0);

    let hollow = fused(&mut model, &pair.arguments, &pair.tools, Rule::Difference);
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
    let pair = a_cube_wholly_inside_another(&mut model);

    let nothing = fused(&mut model, &pair.tools, &pair.arguments, Rule::Difference);

    assert!(nothing.solids().is_empty());
}

// 8. Touching but not overlapping — on a face, on an edge, at a corner.

/// A small cube standing on the top face of a big one, its whole footprint
/// inside that face. They share skin and no space.
fn two_cubes_touching_on_a_face(model: &mut Model) -> Pair {
    Pair {
        arguments: vec![cube(model, 4.0, Vector::new(0.0, 0.0, 0.0))],
        tools: vec![cube(model, 2.0, Vector::new(1.0, 1.0, 4.0))],
    }
}

#[test]
fn two_cubes_that_only_touch_on_a_face() {
    let mut model = a_model();
    let pair = two_cubes_touching_on_a_face(&mut model);

    let united = fused(&mut model, &pair.arguments, &pair.tools, Rule::Union);
    assert_eq!(
        united.solids().len(),
        1,
        "a shared face joins them into one solid"
    );
    assert_eq!(model.solid(united.solids()[0]).shells().count(), 1);
    assert_eq!(volume_of(&model, united.solids()), 72.0);
    nothing_in_it_is_degenerate(&model, united.solids());

    let shared = fused(&mut model, &pair.arguments, &pair.tools, Rule::Intersection);
    assert!(shared.solids().is_empty());

    let bitten = fused(&mut model, &pair.arguments, &pair.tools, Rule::Difference);
    assert_eq!(volume_of(&model, bitten.solids()), 64.0);
}

/// A small cube touching a big one along part of one vertical edge, and nowhere
/// else.
fn two_cubes_touching_along_part_of_an_edge(model: &mut Model) -> Pair {
    Pair {
        arguments: vec![cube(model, 4.0, Vector::new(0.0, 0.0, 0.0))],
        tools: vec![cube(model, 2.0, Vector::new(4.0, 4.0, 1.0))],
    }
}

/// Four faces run along that stretch, so the union is refused for the same
/// reason as the two catty-corner cubes above — and this time the shared stretch
/// is only part of the big cube's edge, so the refusal does not depend on the
/// two edges being the same edge.
#[test]
fn two_cubes_that_only_touch_along_part_of_an_edge() {
    let mut model = a_model();
    let pair = two_cubes_touching_along_part_of_an_edge(&mut model);

    match refusal(&mut model, &pair.arguments, &pair.tools, Rule::Union) {
        Error::ResultIsNotManifold { count } => assert_eq!(count, 4),
        other => panic!("two solids joined along an edge are not one solid: {other:?}"),
    }

    let shared = fused(&mut model, &pair.arguments, &pair.tools, Rule::Intersection);
    assert!(shared.solids().is_empty());

    let bitten = fused(&mut model, &pair.arguments, &pair.tools, Rule::Difference);
    assert_eq!(volume_of(&model, bitten.solids()), 64.0);
}

/// Two cubes that meet at exactly one point.
fn two_cubes_touching_at_a_corner(model: &mut Model) -> Pair {
    Pair {
        arguments: vec![cube(model, 2.0, Vector::new(0.0, 0.0, 0.0))],
        tools: vec![cube(model, 2.0, Vector::new(2.0, 2.0, 2.0))],
    }
}

/// No edge of either is claimed more than twice, so the manifold check has
/// nothing to say; the two never share an edge either, so the walk that groups
/// faces into shells never joins them, and the answer is two solids that happen
/// to touch.
#[test]
fn two_cubes_that_only_touch_at_a_corner() {
    let mut model = a_model();
    let pair = two_cubes_touching_at_a_corner(&mut model);

    let united = fused(&mut model, &pair.arguments, &pair.tools, Rule::Union);
    assert_eq!(
        united.solids().len(),
        2,
        "one point of contact joins no faces, so they stay two"
    );
    assert_eq!(volume_of(&model, united.solids()), 16.0);
    nothing_in_it_is_degenerate(&model, united.solids());

    let shared = fused(&mut model, &pair.arguments, &pair.tools, Rule::Intersection);
    assert!(shared.solids().is_empty());

    let bitten = fused(&mut model, &pair.arguments, &pair.tools, Rule::Difference);
    assert_eq!(volume_of(&model, bitten.solids()), 8.0);
}

// §17.7's catalogue family, gathered into the one thing it can honestly be
// reported as.

/// The nine planar entries, side by side, as the table §17.7 carries.
///
/// Nine hand-written cases are not a sample, so this family gets a table and not
/// a rate: each entry either passes or refuses, and a refusal is right or wrong
/// on its own terms rather than by how many of them there are. The rest of this
/// file asserts each entry's volumes, counts and areas; what this adds is the
/// summary, checked as it is printed, so the record in §17.7 cannot drift away
/// from what the kernel does without something going red.
///
/// The pairs are the very ones the assertions above are about, because both go
/// through the same builder.
#[test]
fn the_nine_planar_entries_as_a_table() {
    println!("| §7.4 entry | the pair | rule | outcome |");
    println!("|---|---|---|---|");
    for entry in catalogue() {
        let said = match entry.case {
            Case::Fused {
                build,
                rule,
                outcome,
            } => {
                assert_eq!(
                    what_happened(build, rule),
                    outcome,
                    "{}, {}: the catalogue table in §17.7 says otherwise",
                    entry.degeneracy,
                    entry.pair
                );
                format!("{rule:?} | {}", outcome.said())
            }
            Case::InsideTheCrate {
                rule,
                refusal,
                test,
            } => format!("{rule:?} | refused, `{refusal}`, by `{test}`"),
        };
        println!("| {} | {} | {said} |", entry.degeneracy, entry.pair);
    }
}

/// One row of the catalogue table.
struct Entry {
    /// Which of roadmap §7.4's nine planar degeneracies this is a case of.
    degeneracy: &'static str,
    /// The pair, in a few words.
    pair: &'static str,
    case: Case,
}

/// How a row is discharged.
enum Case {
    /// Build the pair here and ask `rule` of it.
    Fused {
        build: fn(&mut Model) -> Pair,
        rule: Rule,
        outcome: Outcome,
    },
    /// Nothing in the public API can build the input, so the case lives inside
    /// the crate and the row only names what discharges it.
    InsideTheCrate {
        rule: Rule,
        /// The `Error` variant it comes back with.
        refusal: &'static str,
        /// The test that asserts it, in `crates/rcad-topo/src/fuse/mod.rs`.
        test: &'static str,
    },
}

/// What a certified fuse of a catalogue pair came back with.
///
/// There is deliberately no variant for `Error::Undecidable`. Every pair here is
/// exactly representable, so a certified policy has to decide all of it, and one
/// arriving would be §17.7's control claim broken rather than a row of the table
/// reading differently.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Outcome {
    /// An answer, with every decision in it proved.
    Answered,
    /// A refusal, because the answer would use an edge more than twice and is
    /// not a solid a `Model` can hold. Right rather than a bug (§17.4).
    NotManifold,
}

impl Outcome {
    fn said(self) -> &'static str {
        match self {
            Outcome::Answered => "answered",
            Outcome::NotManifold => "refused, `ResultIsNotManifold`",
        }
    }
}

fn catalogue() -> Vec<Entry> {
    vec![
        Entry {
            degeneracy: "1. coplanar faces",
            pair: "two cubes sharing a whole face",
            case: Case::Fused {
                build: two_cubes_sharing_a_whole_face,
                rule: Rule::Union,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "1. coplanar faces",
            pair: "two slabs whose faces half overlap",
            case: Case::Fused {
                build: two_slabs_whose_faces_half_overlap,
                rule: Rule::Union,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "2. coincident edges",
            pair: "two cubes catty-corner, sharing one whole edge",
            case: Case::Fused {
                build: two_cubes_sharing_one_whole_edge,
                rule: Rule::Union,
                outcome: Outcome::NotManifold,
            },
        },
        Entry {
            degeneracy: "2. coincident edges",
            pair: "the same two, asked what they share",
            case: Case::Fused {
                build: two_cubes_sharing_one_whole_edge,
                rule: Rule::Intersection,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "3. vertex on face",
            pair: "two cubes offset by a corner",
            case: Case::Fused {
                build: two_cubes_offset_by_a_corner,
                rule: Rule::Union,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "4. vertex on edge",
            pair: "a box poking out of the top of a cube",
            case: Case::Fused {
                build: a_box_poking_out_of_the_top_of_a_cube,
                rule: Rule::Union,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "5. zero-length edges",
            pair: "a cube and two bars whose walls are on the same planes",
            case: Case::Fused {
                build: a_cube_and_two_bars_on_the_same_planes,
                rule: Rule::Union,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "6. sliver faces",
            pair: "a cube and a knife a millionth of the way in",
            case: Case::Fused {
                build: a_cube_and_a_knife_a_sliver_in,
                rule: Rule::Intersection,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "7. fully contained solids",
            pair: "a cube in the middle of another, hollowed out",
            case: Case::Fused {
                build: a_cube_wholly_inside_another,
                rule: Rule::Difference,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "8. touching, not overlapping",
            pair: "two cubes touching on a face",
            case: Case::Fused {
                build: two_cubes_touching_on_a_face,
                rule: Rule::Union,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "8. touching, not overlapping",
            pair: "two cubes touching along part of an edge",
            case: Case::Fused {
                build: two_cubes_touching_along_part_of_an_edge,
                rule: Rule::Union,
                outcome: Outcome::NotManifold,
            },
        },
        Entry {
            degeneracy: "8. touching, not overlapping",
            pair: "two cubes touching at a corner",
            case: Case::Fused {
                build: two_cubes_touching_at_a_corner,
                rule: Rule::Union,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "9. self-intersecting input",
            pair: "two cubes wired into one solid that runs through itself",
            case: Case::InsideTheCrate {
                rule: Rule::Union,
                refusal: "ArgumentIntersectsItself",
                test: "a_solid_that_runs_through_itself_cannot_be_fused",
            },
        },
    ]
}

/// Build a catalogue pair, fuse it under a certified policy, and say what came
/// back.
fn what_happened(build: fn(&mut Model) -> Pair, rule: Rule) -> Outcome {
    let mut model = a_model();
    let pair = build(&mut model);
    let mut log = ProvenanceLog::new();

    match model.fuse(
        &pair.arguments,
        &pair.tools,
        rule,
        &Policy::certified(),
        &mut log,
    ) {
        Ok(_) => {
            assert!(
                log.is_empty(),
                "a certified fuse of an exactly representable pair records nothing at all"
            );
            Outcome::Answered
        }
        Err(Error::ResultIsNotManifold { .. }) => Outcome::NotManifold,
        Err(why) => panic!("a lattice-aligned {rule:?} is decided by proof alone: {why}"),
    }
}

/// The deflection the measurements here are asked for.
///
/// Every solid in this file is a polyhedron, and a polyhedron ignores it: its
/// own fan is the answer rather than a picture of one (`docs/decisions.md`
/// §25.4).
fn a_deflection() -> Deflection {
    Deflection::of(0.1).expect("a tenth of a millimetre is a length")
}
