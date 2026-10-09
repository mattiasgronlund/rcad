//! Cutting two groups of solids against each other and keeping what a rule
//! asks for.
//!
//! The volumes are the oracle throughout, because they are the one thing that
//! is true whatever the answer is made of: `vol(A ∪ B) + vol(A ∩ B) =
//! vol(A) + vol(B)` holds for every pair, and for the lattice-aligned pairs
//! here every corner is exactly representable, so the arithmetic is exact and
//! the numbers can be compared with `==`.

use rcad_geom::{Axis, Deflection, DynamicRange, Isometry, Point, Scale, Vector};
use rcad_predicates::{
    Approximation, Cause, Decision, Point as PredicatePoint, Policy, ProvenanceLog,
    meet_of_three_planes, side_of_a_plane_where_three_planes_meet,
};
use rcad_topo::{
    Containment, Error, FaceKey, Invalid, Model, ProfileBuilder, Rule, SolidKey, VertexKey,
};

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// A cube of the given side, with its low corner at `at`.
fn cube(model: &mut Model, side: f64, at: Vector) -> SolidKey {
    let cube = model
        .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model
        .place_solid(
            cube,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("this model made the key");
    cube
}

/// A unit square extruded `height` high and put down at `at`.
fn tower(model: &mut Model, at: Vector, height: f64) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a unit square");
    model
        .add_extrusion(
            &square,
            height,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("building it needs no decision")
}

/// Two cubes of side two, the second one moved by `shift`.
fn two_cubes(shift: Vector) -> (Model, SolidKey, SolidKey) {
    let mut model = a_model();
    let one = cube(&mut model, 2.0, Vector::new(0.0, 0.0, 0.0));
    let other = cube(&mut model, 2.0, shift);
    (model, one, other)
}

/// How much space the answer takes up, all of its solids together.
fn volume_of(model: &Model, solids: &[SolidKey]) -> f64 {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
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

/// Fuse under a certified policy and insist nothing had to be approximated.
fn fuse(
    model: &mut Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
    rule: Rule,
) -> Vec<SolidKey> {
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(arguments, tools, rule, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| {
            panic!("a lattice-aligned {rule:?} is decided by proof alone: {why}")
        });
    assert!(
        log.is_empty(),
        "a certified fuse of lattice-aligned cubes records nothing at all"
    );
    fused.solids().to_vec()
}

#[test]
fn two_overlapping_cubes_add_up_the_way_inclusion_and_exclusion_says_they_must() {
    let shift = Vector::new(1.0, 1.0, 1.0);
    let overlap = 1.0;

    let (mut model, one, other) = two_cubes(shift);
    let united = fuse(&mut model, &[one], &[other], Rule::Union);
    assert_eq!(united.len(), 1, "two cubes that overlap unite into one");
    assert_eq!(volume_of(&model, &united), 8.0 + 8.0 - overlap);

    let (mut model, one, other) = two_cubes(shift);
    let shared = fuse(&mut model, &[one], &[other], Rule::Intersection);
    assert_eq!(shared.len(), 1);
    assert_eq!(volume_of(&model, &shared), overlap);

    let (mut model, one, other) = two_cubes(shift);
    let bitten = fuse(&mut model, &[one], &[other], Rule::Difference);
    assert_eq!(bitten.len(), 1);
    assert_eq!(volume_of(&model, &bitten), 8.0 - overlap);
}

#[test]
fn two_cubes_that_do_not_touch_unite_into_two_solids_and_share_nothing() {
    let apart = Vector::new(10.0, 0.0, 0.0);

    let (mut model, one, other) = two_cubes(apart);
    let united = fuse(&mut model, &[one], &[other], Rule::Union);
    assert_eq!(united.len(), 2, "nothing joined them, so they stay two");
    assert_eq!(volume_of(&model, &united), 16.0);

    let (mut model, one, other) = two_cubes(apart);
    let shared = fuse(&mut model, &[one], &[other], Rule::Intersection);
    assert!(
        shared.is_empty(),
        "an empty answer is a success, not a failure"
    );

    let (mut model, one, other) = two_cubes(apart);
    let bitten = fuse(&mut model, &[one], &[other], Rule::Difference);
    assert_eq!(bitten.len(), 1);
    assert_eq!(volume_of(&model, &bitten), 8.0);
}

#[test]
fn a_cube_with_a_smaller_one_taken_out_of_the_middle_is_one_solid_with_a_void() {
    let mut model = a_model();
    let block = cube(&mut model, 4.0, Vector::new(0.0, 0.0, 0.0));
    let bubble = cube(&mut model, 2.0, Vector::new(1.0, 1.0, 1.0));

    let hollow = fuse(&mut model, &[block], &[bubble], Rule::Difference);

    assert_eq!(hollow.len(), 1);
    let solid = model.solid(hollow[0]).expect("this model made the key");
    assert_eq!(
        solid.shells().count(),
        2,
        "the outside of the block and the skin of the hole it now has"
    );
    assert_eq!(
        solid
            .measure(
                a_deflection(),
                &Policy::certified(),
                &mut ProvenanceLog::new()
            )
            .expect("a solid this size measures")
            .volume(),
        64.0 - 8.0
    );
}

/// `A` is the union of the arguments, so two solids handed over on the same
/// side have to be decomposed against each other like any other pair. Without
/// that the halves of each cube buried inside the other survive, and the answer
/// is a self-overlapping shell that measures far too much.
#[test]
fn two_overlapping_cubes_on_the_same_side_are_united_with_each_other() {
    let mut model = a_model();
    let one = cube(&mut model, 2.0, Vector::new(0.0, 0.0, 0.0));
    let other = cube(&mut model, 2.0, Vector::new(1.0, 1.0, 1.0));

    let united = fuse(&mut model, &[one, other], &[], Rule::Union);

    assert_eq!(united.len(), 1);
    assert_eq!(volume_of(&model, &united), 8.0 + 8.0 - 1.0);
}

/// Two cubes that share exactly one edge and nothing else. United, that edge
/// would have four faces along it, and a `Model` means an edge with two
/// (decisions.md §17.4).
#[test]
fn two_cubes_meeting_along_one_edge_cannot_be_united() {
    let mut model = a_model();
    let one = cube(&mut model, 2.0, Vector::new(0.0, 0.0, 0.0));
    let other = cube(&mut model, 2.0, Vector::new(2.0, 2.0, 0.0));

    let refused = model.fuse(
        &[one],
        &[other],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    match refused {
        Err(Error::ResultIsNotManifold { count }) => assert!(
            count != 2,
            "the refusal has to name an edge that is not used twice"
        ),
        other => panic!("that union is not a solid this kernel can hold: {other:?}"),
    }
}

/// Atomicity (§17.4). The refusal chosen is the non-manifold one above: it is
/// reached at the very last moment, after every piece has been cut, classified
/// and joined up, so anything the fuse might have written would already have
/// been written by the time it fails.
#[test]
fn a_fuse_that_refuses_leaves_the_model_byte_for_byte_as_it_was() {
    let mut model = a_model();
    let one = cube(&mut model, 2.0, Vector::new(0.0, 0.0, 0.0));
    let other = cube(&mut model, 2.0, Vector::new(2.0, 2.0, 0.0));
    let before = postcard::to_allocvec(&model).expect("a model writes out");

    let refused = model.fuse(
        &[one],
        &[other],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    assert!(refused.is_err());
    assert_eq!(
        before,
        postcard::to_allocvec(&model).expect("a model writes out"),
        "a fuse that refuses writes nothing at all"
    );
}

/// Determinism (§17.4). This is the test that fails the day someone reaches for
/// a `HashMap` in the fuse's working state, and nothing else in the suite would
/// notice: hash order varies per process, that order decides the order entities
/// go into the arena, and arena order is in every serialized byte.
#[test]
fn the_same_fuse_twice_over_writes_the_same_bytes_twice() {
    let mut written = Vec::new();
    for _ in 0..2 {
        let (mut model, one, other) = two_cubes(Vector::new(1.0, 1.0, 1.0));
        fuse(&mut model, &[one], &[other], Rule::Union);
        written.push(postcard::to_allocvec(&model).expect("a model writes out"));
    }

    assert_eq!(written[0], written[1]);
}

/// Every face of the answer has exactly one input face behind it, and none has
/// none. Free to check, and it panics if the splitter ever invents geometry.
///
/// Deliberately removed at M3: a curved intersection can produce a face that is
/// no input face's child, and then this stops being true on purpose.
#[test]
fn every_face_of_the_answer_has_exactly_one_input_face_behind_it() {
    let (mut model, one, other) = two_cubes(Vector::new(1.0, 1.0, 1.0));
    let faces_before = model.face_count();

    let fused = model
        .fuse(
            &[one],
            &[other],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("two lattice-aligned cubes unite");

    assert_eq!(fused.generated_faces().count(), 0);
    let images: usize = model
        .solid(fused.solids()[0])
        .expect("this model made the key")
        .faces()
        .count();
    assert_eq!(images, model.face_count() - faces_before);
    assert!(images > 0);
}

/// The guard against re-conflating the two kinds of wobble (§17.2). The input
/// carries a tag, a certified policy is a promise about *this* operation's
/// decisions, and the two have nothing to do with each other: the fuse succeeds
/// and the tag comes through.
#[test]
fn a_certified_fuse_over_a_tagged_input_succeeds_and_the_answer_keeps_the_tag() {
    let (mut model, one, other) = two_cubes(Vector::new(1.0, 1.0, 1.0));
    let tag = Approximation::new(1e-6, Cause::ImportedUncertainty);
    for vertex in model
        .vertices()
        .map(|vertex| vertex.key())
        .collect::<Vec<_>>()
    {
        model
            .set_vertex_approximation(vertex, tag)
            .expect("an imported corner may carry slop");
    }
    assert_eq!(model.worst_approximation(), 1e-6);

    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(
            &[one],
            &[other],
            Rule::Union,
            &Policy::certified(),
            &mut log,
        )
        .expect("an inherited tag is not a decision this operation failed to prove");

    assert!(log.is_empty(), "nothing here had to be approximated");
    assert!(model.worst_approximation() >= 1e-6);
    for face in model
        .solid(fused.solids()[0])
        .expect("this model made the key")
        .faces()
    {
        assert_eq!(
            model
                .face(face.key())
                .expect("this model made the key")
                .approximation(),
            Some(tag),
            "a face cut by a face known only to a millionth is known no better"
        );
    }
}

/// The input keys still resolve afterwards, and the image maps say what became
/// of each one. A face nothing reaches comes through as one face that is not
/// itself — the answer's faces are new faces, because the inputs are still in
/// the model using the old ones and a face may only be used once.
#[test]
fn the_input_keys_still_work_and_a_face_nothing_reaches_comes_through_once() {
    let (mut model, one, other) = two_cubes(Vector::new(10.0, 0.0, 0.0));

    let fused = model
        .fuse(
            &[one],
            &[other],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("two cubes that are nowhere near each other");

    for solid in [one, other] {
        for face in model.solid(solid).expect("this model made the key").faces() {
            let images = fused
                .images_of_face(face.key())
                .expect("this model made the key");
            assert_eq!(images.len(), 1);
            assert_ne!(images[0], face.key());
            assert_eq!(
                model
                    .face(images[0])
                    .expect("this model made the key")
                    .plane(),
                face.plane()
            );
        }
    }
    assert_eq!(fused.deleted_faces().count(), 0);
    assert_eq!(fused.modified_faces().count(), 12);
}

/// Every face of the answer says which input face it was cut out of, and every
/// input face that survived says which faces of the answer it became.
///
/// The block's faces are cut by the bubble's six planes, which reach past the
/// bubble itself — a cut is by a plane, not by the patch of it a face covers.
/// Nothing on any of those sides is dropped, though, so the nine pieces each
/// side comes apart into are joined straight back into the one face they were
/// cut from (§17.4, amended 2026-08-30), and each side of the block has one
/// image. The bubble's own faces are cut by nothing, since none of the block's
/// planes passes through the middle of the block where the bubble sits.
#[test]
fn every_face_of_an_answer_names_the_input_face_it_was_cut_from() {
    let mut model = a_model();
    let block = cube(&mut model, 4.0, Vector::new(0.0, 0.0, 0.0));
    let bubble = cube(&mut model, 2.0, Vector::new(1.0, 1.0, 1.0));

    let fused = model
        .fuse(
            &[block],
            &[bubble],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a block with a bubble taken out of it");

    let mut images = 0;
    for face in model.solid(block).expect("this model made the key").faces() {
        let mine = fused
            .images_of_face(face.key())
            .expect("this model made the key")
            .len();
        assert_eq!(
            mine, 1,
            "the pieces the bubble's planes cut this side into are all still one face"
        );
        images += mine;
    }
    for face in model
        .solid(bubble)
        .expect("this model made the key")
        .faces()
    {
        assert_eq!(
            fused
                .images_of_face(face.key())
                .expect("this model made the key")
                .len(),
            1,
            "every face of the bubble becomes one face of the hole's skin"
        );
        images += 1;
    }
    assert_eq!(
        images,
        model
            .solid(fused.solids()[0])
            .expect("this model made the key")
            .faces()
            .count(),
        "every face of the answer is the image of exactly one input face"
    );
    assert_eq!(fused.deleted_faces().count(), 0);

    for vertex in corners_of(&model, bubble) {
        assert_eq!(
            fused
                .images_of_vertex(vertex)
                .expect("this model made the key")
                .len(),
            1
        );
    }
}

/// Every corner of a solid, once each.
fn corners_of(model: &Model, solid: SolidKey) -> Vec<VertexKey> {
    let mut corners = Vec::new();
    for face in model.solid(solid).expect("this model made the key").faces() {
        for wire in model
            .face(face.key())
            .expect("this model made the key")
            .wires()
        {
            for coedge in wire.coedges() {
                let corner = coedge.start().key();
                if !corners.contains(&corner) {
                    corners.push(corner);
                }
            }
        }
    }
    corners
}

/// A face that is swallowed whole has no image at all, which is what "deleted"
/// means: the input key still resolves and still names the face it always did.
#[test]
fn a_face_that_is_swallowed_whole_has_no_image() {
    let mut model = a_model();
    let block = cube(&mut model, 4.0, Vector::new(0.0, 0.0, 0.0));
    let bubble = cube(&mut model, 2.0, Vector::new(1.0, 1.0, 1.0));

    let fused = model
        .fuse(
            &[block],
            &[bubble],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bubble inside a block adds nothing to it");

    for face in model
        .solid(bubble)
        .expect("this model made the key")
        .faces()
    {
        assert!(
            fused
                .images_of_face(face.key())
                .expect("this model made the key")
                .is_empty(),
            "a face inside the other solid is not on the boundary of the union"
        );
    }
    assert_eq!(fused.deleted_faces().count(), 6);
    assert_eq!(volume_of(&model, fused.solids()), 64.0);
}

/// Exit criterion 7, both halves (§17.2). A pair of cubes under a general
/// rotation has not one exactly representable corner between them, so every
/// meet the fuse solves is a box — and that is *representation* wobble, which
/// is reported and never refused on. A certified fuse of them succeeds and its
/// log stays empty.
#[test]
fn a_turned_pair_is_certified_because_a_box_is_not_a_decision_anybody_failed_to_prove() {
    let (mut model, one, other) = two_cubes(Vector::new(1.0, 1.0, 1.0));
    let axis = Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 2.0, 3.0))
        .expect("a direction with length");
    let turn = Isometry::rotation(&axis, 0.7).expect("a rotation");
    model
        .place_solid(one, &turn)
        .expect("this model made the key");
    model
        .place_solid(other, &turn)
        .expect("this model made the key");

    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(
            &[one],
            &[other],
            Rule::Union,
            &Policy::certified(),
            &mut log,
        )
        .expect("a turned pair is decided by proof alone");

    assert!(
        log.is_empty(),
        "every corner being a box is not slop anybody failed to prove"
    );
    assert_eq!(fused.solids().len(), 1);
    let volume = volume_of(&model, fused.solids());
    assert!(
        (volume - 15.0).abs() < 1e-9,
        "turning a pair of cubes does not change how much space they take up, got {volume}"
    );
    assert!(
        model.worst_approximation() > 0.0,
        "the corners the fuse solved are boxes, and the model says so"
    );
}

/// Two turned cubes overlap by less than a double can see. A certified fuse
/// refuses what they share as `Invalid::DynamicRangeTooWide`, and it is right
/// to: **no solid a `Model` can hold is the answer**. That makes it §3.5's
/// limit, reached by name (`docs/decisions.md` §417.3).
///
/// **It is not exit criterion 7's refusing half** (§17.2), though it stood as
/// that half from 2026-10-05 until Q1353. Criterion 7 is the judgment of
/// `Policy::max_wobble` against `ProvenanceLog::worst_wobble`, and it refuses
/// `Error::Undecidable`. The fuse asks the range check only after that
/// judgment has passed, so a refusal by the range check says nothing about
/// the judgment (§353.5).
///
/// The pair is the one `two_turned_cubes_a_rounding_apart.rs` turns by `0.3`.
/// Two cubes of side two, the second moved by `(2, 0.5, 0.25)`, both turned
/// about the axis through `(1, 0, 0)` along `(1, 2, 3)`. The first cube's face
/// `x = 2` and the second's face `x = 0` then face exactly opposite ways, and
/// their offsets add up to `2^-52`, which that file pins. So what the two
/// share is a slab `1.5` by `1.75` across and about `2^-52` thick.
///
/// **Why no model holds the slab, shown here and not argued.** It is convex,
/// so its corners are the points where three of the twelve planes meet and
/// every other plane has the point behind it, and the planes decide that
/// exactly. There are eight, each on exactly three planes, so two corners on
/// two common planes are the two ends of an edge, and there are twelve. Four
/// of the edges cross the slab. `meet_of_three_planes` puts a box round each
/// corner, and the two boxes of the shortest edge lie within `2.75e-15` of each
/// other while some corner's box lies `2.178` or more from the origin. However
/// an answer places its corners inside their boxes, its largest coordinate over
/// its shortest edge is past `7.9e14`. `Model::validate` refuses a model past
/// `1e11` (`docs/decisions.md` §3.5). The fuse asks its own answer the same
/// before it writes it, because its inputs are inside that range, which the
/// first lines check. So this refusal is not a gap the arithmetic failed to
/// reach (§16.10). There is no answer to reach. §17's exit table says what was
/// tried for criterion 7's refusing half, and that none is known.
///
/// **Where the fuse stops is now the range check, not selection.** The
/// slab's face on the second cube's bottom used to be a strip `2^-52` wide
/// that no candidate point could be shown to lie inside, so the fuse refused
/// it `Undecidable` at *a point inside a piece of a face*. Q1353's
/// `by_a_plane` places a piece inside a solid when every plane of the solid
/// has every corner of the piece on or behind it and one strictly behind
/// (`docs/decisions.md` §417), which that strip has, so selection now writes
/// the slab's sliver and the range check refuses it by name instead, for the
/// edge `2.2e-16` long at a coordinate past `2.18` that it would need.
#[test]
fn what_two_turned_cubes_a_rounding_deep_in_each_other_share_is_too_thin_to_hold_and_is_refused() {
    let (mut model, one, other) = two_turned_cubes_a_rounding_deep_in_each_other();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("each cube on its own is a sound solid");

    let planes: Vec<[f64; 4]> = [one, other]
        .iter()
        .flat_map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .faces()
                .map(|face| face.plane().coefficients())
                .collect::<Vec<_>>()
        })
        .collect();
    let corners = corners_of_what_every_plane_has_behind_it(&planes);
    assert_eq!(corners.len(), 8, "the slab is a box: {corners:?}");
    let mut edges = Vec::new();
    for (at, (three, end)) in corners.iter().enumerate() {
        for (other_three, other_end) in &corners[at + 1..] {
            let common = three
                .iter()
                .filter(|plane| other_three.contains(plane))
                .count();
            if common == 2 {
                edges.push(the_farthest_apart_two_points_of(*end, *other_end));
            }
        }
    }
    assert_eq!(edges.len(), 12, "a box has twelve edges");
    let shortest = edges.into_iter().fold(f64::INFINITY, f64::min);
    let largest = corners
        .iter()
        .map(|(_, corner)| {
            corner
                .enclosure()
                .iter()
                .map(|side| side.lo().abs().min(side.hi().abs()))
                .fold(0.0, f64::max)
        })
        .fold(0.0, f64::max);
    let spread =
        DynamicRange::measure(largest, shortest).expect("a coordinate and an edge are lengths");
    assert!(
        !spread.is_within_supported_range() && spread.ratio() > 7.9e14,
        "an edge of at most {shortest:e} at a coordinate of at least {largest} is a spread of {:e}",
        spread.ratio()
    );

    let refused = model.fuse(
        &[other],
        &[one],
        Rule::Intersection,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    assert!(
        matches!(
            refused,
            Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) if ratio > 1e16
        ),
        "a slab no model can hold is refused for the edge it would need: {refused:?}"
    );
}

/// [`what_two_turned_cubes_a_rounding_deep_in_each_other_share_is_too_thin_to_hold_and_is_refused`]'s
/// pair: built the way `two_turned_cubes_a_rounding_apart.rs` builds it, at a
/// turn of `0.3`.
fn two_turned_cubes_a_rounding_deep_in_each_other() -> (Model, SolidKey, SolidKey) {
    let mut model = a_model();
    let one = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let other = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let turn = Isometry::rotation(
        &Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 2.0, 3.0))
            .expect("a direction with length"),
        0.3,
    )
    .expect("a rotation");
    model
        .place_solid(one, &turn)
        .expect("this model made the key");
    model
        .place_solid(
            other,
            &turn.compose(&Isometry::translation(Vector::new(2.0, 0.5, 0.25)).expect("a shift")),
        )
        .expect("this model made the key");
    (model, one, other)
}

/// Every point where three of `planes` meet and every other one of them has
/// the point strictly behind it, with the three it lies on.
///
/// Strictly, so that each corner found lies on exactly three planes. Asked of
/// the planes themselves with `side_of_a_plane_where_three_planes_meet`, which
/// is exact, and never of the box round the point.
fn corners_of_what_every_plane_has_behind_it(
    planes: &[[f64; 4]],
) -> Vec<([usize; 3], PredicatePoint)> {
    let mut corners = Vec::new();
    for first in 0..planes.len() {
        for second in first + 1..planes.len() {
            for third in second + 1..planes.len() {
                let three = [first, second, third];
                let meet = three.map(|at| planes[at]);
                let everything_else_has_it_behind = planes
                    .iter()
                    .enumerate()
                    .filter(|(at, _)| !three.contains(at))
                    .all(|(_, plane)| {
                        side_of_a_plane_where_three_planes_meet(meet, *plane) == Ok(Decision::Neg)
                    });
                if everything_else_has_it_behind {
                    let corner = meet_of_three_planes(meet[0], meet[1], meet[2])
                        .expect("three planes with a point behind every other plane meet");
                    corners.push((three, corner));
                }
            }
        }
    }
    corners
}

/// The furthest apart any point of one box and any point of the other can be.
fn the_farthest_apart_two_points_of(one: PredicatePoint, other: PredicatePoint) -> f64 {
    let (one, other) = (one.enclosure(), other.enclosure());
    let across =
        |axis: usize| one[axis].hi().max(other[axis].hi()) - one[axis].lo().min(other[axis].lo());
    across(0).hypot(across(1)).hypot(across(2))
}

/// Two blocks two metres on a side, turned the same way and meeting corner to
/// corner, on a model whose working size is 100 mm. Every rule, both ways
/// round, gives the same answer under `Policy::tolerant` at `1e4` and `1e6` as
/// it does under a certified policy and under `Policy::tolerant` at `100` and
/// `1e9`: the two blocks for a union, nothing for an intersection, and the
/// first block for a difference.
///
/// Until Q1467 the two in the middle were refused `Undecidable`, and that was
/// a gap (`docs/decisions.md` §16.10, §17.8): the answer exists, and the
/// certified fuse gave it. The policy could not prove two `orient3d`
/// decisions, each about `2.1e-4` wide. `resolve_under_policy` read each one
/// as a wobble at the policy's working size, found it within
/// `Policy::max_wobble`, and accepted it. The fuse's judgment (§17.2) then read
/// the same log at the model's working size, `100`, where a volume is divided
/// by `100²` instead of by `1e4²` and comes out ten thousand times bigger. So
/// it refused a solid built only out of decisions the policy had accepted one
/// at a time, which `Policy::wobble_is_acceptable`'s own doc says its one
/// budget is there to stop. The judgment reads the log at the policy's working
/// size now (§15.2).
///
/// The log is what says this test still reaches the judgment. Under `1e4` and
/// `1e6` every fuse records decisions, each carries an `Approximation`, so
/// the policy accepted it, and read at the model's working size they are past
/// the budget.
#[test]
fn a_tolerant_policy_bigger_than_the_model_fuses_two_turned_blocks_meeting_at_a_corner_as_a_certified_one_does()
 {
    let block = 2000.0_f64.powi(3);
    let bigger_than_the_model = [Policy::tolerant(1e4), Policy::tolerant(1e6)];
    let policies = [
        Policy::certified(),
        Policy::tolerant(100.0),
        Policy::tolerant(1e9),
    ]
    .into_iter()
    .chain(bigger_than_the_model);
    for policy in policies {
        for (rule, solids, volume) in [
            (Rule::Union, 2, 2.0 * block),
            (Rule::Intersection, 0, 0.0),
            (Rule::Difference, 1, block),
        ] {
            for one_first in [true, false] {
                let (mut model, one, other) = two_turned_blocks_meeting_at_a_corner();
                let (first, second) = if one_first {
                    (one, other)
                } else {
                    (other, one)
                };
                let case = format!("{policy:?}, {rule:?}, first block first: {one_first}");
                let mut log = ProvenanceLog::new();
                let fused = model
                    .fuse(&[first], &[second], rule, &policy, &mut log)
                    .unwrap_or_else(|why| panic!("{case}: the blocks have an answer: {why}"));
                model
                    .validate(&policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{case}: the answer is a valid solid: {why}"));
                assert_eq!(fused.solids().len(), solids, "{case}");
                let measured = volume_of(&model, fused.solids());
                assert!(
                    (measured - volume).abs() <= 1e-9 * block,
                    "{case}: blocks meeting only at a corner share no volume, so the \
                     answer is {volume:e}, got {measured:e}"
                );

                assert!(
                    log.records()
                        .iter()
                        .all(|record| record.approximation().is_some()),
                    "{case}: every decision the policy could not prove, it accepted: {log:?}"
                );
                if bigger_than_the_model.contains(&policy) {
                    assert!(
                        !log.is_empty()
                            && !policy.wobble_is_acceptable(
                                log.worst_wobble(model.scale().nominal_size())
                            ),
                        "{case}: read at the model's working size, the log is past the budget: \
                         {log:?}"
                    );
                } else if policy == Policy::certified() {
                    assert!(
                        log.is_empty(),
                        "{case}: a certified fuse records nothing: {log:?}"
                    );
                }
            }
        }
    }
}

/// [`a_tolerant_policy_bigger_than_the_model_fuses_two_turned_blocks_meeting_at_a_corner_as_a_certified_one_does`]'s
/// pair: two cubes of side `2000` on a 100 mm model, the second moved by
/// `(2000, 2000, 2000)`, both turned by the same turn. The turn's axis and
/// angle are one draw of Q1457's probe, written out to the last bit.
fn two_turned_blocks_meeting_at_a_corner() -> (Model, SolidKey, SolidKey) {
    let mut model = a_model();
    let one = model
        .add_cube(2000.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side 2000");
    let other = model
        .add_cube(2000.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side 2000");
    let turn = Isometry::rotation(
        &Axis::new(
            Point::new(1000.0, 0.0, 0.0),
            Vector::new(
                0.225_943_046_113_692_2,
                -0.047_950_555_307_198_495,
                -0.297_768_573_555_231_8,
            ),
        )
        .expect("a direction with length"),
        1.374_160_163_811_960_3,
    )
    .expect("a rotation");
    model
        .place_solid(one, &turn)
        .expect("this model made the key");
    model
        .place_solid(
            other,
            &turn.compose(
                &Isometry::translation(Vector::new(2000.0, 2000.0, 2000.0)).expect("a shift"),
            ),
        )
        .expect("this model made the key");
    (model, one, other)
}

/// A certified union of a block and a wedge is refused `Error::Undecidable`
/// by criterion 7's judgment, over two records left by a ray that placed
/// nothing, and the union exists.
///
/// The wedge's floor runs through the block's edge from `(1, 1, 0)` to `(1, 1,
/// 1)` and on through the first point `ray_targets` aims at from the block.
/// Selection asks the block where a point on that floor is. The point is a
/// box, and the box straddles the plane through the edge and the target, so the
/// first ray grazes the edge. Its two `orient3d` calls cannot be decided, and
/// `resolve_under_policy` writes each into the log it was handed, under a
/// certified policy too. `classify_uncertain_point` keeps that log and tries
/// the next target, which places the point inside. Selection then drains the
/// log into the fuse's, and the judgment refuses a union it had already built.
///
/// **A gap, not criterion 7's refusing half.** `tolerant(100)` builds the same
/// union, and it validates and measures what the closed form says. When each
/// ray keeps a log of its own and only the one that decided is kept, this
/// answers, and this test is moved to the answer (`docs/decisions.md` §17.8).
/// The floor is aimed at `ray_targets`' first point on purpose: if those
/// targets move, aim the floor at the new one rather than moving this pin.
#[test]
fn a_certified_union_is_refused_over_the_records_of_a_ray_it_threw_away() {
    for wedge_first in [true, false] {
        let case = format!("wedge first: {wedge_first}");
        let (mut model, wedge, block) =
            a_wedge_whose_floor_runs_through_a_blocks_edge_and_the_first_ray_target(100.0);
        let (first, second) = if wedge_first {
            (wedge, block)
        } else {
            (block, wedge)
        };
        let records = match model.fuse(
            &[first],
            &[second],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) {
            Err(Error::Undecidable { records }) => records,
            Err(other) => panic!("{case}: refused by the judgment, not by {other}"),
            Ok(_) => panic!("{case}: refused by the judgment, and it answered"),
        };
        assert_they_are_the_two_records_of_the_ray_that_grazed_the_edge(&case, &records);

        let mut ray = ProvenanceLog::new();
        let placed = model
            .solid(block)
            .expect("this model made the key")
            .classify_uncertain_point(
                records.records()[0].inputs()[3],
                &Policy::certified(),
                &mut ray,
            );
        assert_eq!(
            placed,
            Containment::Inside,
            "{case}: the point the ray was fired from is placed"
        );
        assert_eq!(
            ray, records,
            "{case}: placing it leaves the ray's records behind"
        );

        let (mut model, wedge, block) =
            a_wedge_whose_floor_runs_through_a_blocks_edge_and_the_first_ray_target(100.0);
        let (first, second) = if wedge_first {
            (wedge, block)
        } else {
            (block, wedge)
        };
        let tolerant = Policy::tolerant(100.0);
        let fused = model
            .fuse(
                &[first],
                &[second],
                Rule::Union,
                &tolerant,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{case}: tolerant(100) unites the pair: {why}"));
        model
            .validate(&tolerant, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{case}: the union is a valid solid: {why}"));
        assert_eq!(fused.solids().len(), 1, "{case}");
        let measured = volume_of(&model, fused.solids());
        assert!(
            (measured - the_pairs_union()).abs() <= 1e-12,
            "{case}: the union is {:e}, got {measured:e}",
            the_pairs_union()
        );
    }
}

/// [`a_certified_union_is_refused_over_the_records_of_a_ray_it_threw_away`]'s
/// pair on a model declared `1e160` across: a certified policy takes the
/// union, with the same two records still in the log.
///
/// Under a certified policy the judgment reads the log at the model's
/// `Scale::nominal_size`. A record of a volume is divided by that size twice
/// (§15.2), and an enclosure of about `1.7e-15` divided by `1e160` twice rounds
/// to zero, which a budget of zero accepts. So the same decisions refused on a
/// 100 mm model are passed unproved on a big one. The test pins where that
/// starts: still above zero at `2.5e154`, zero at `2.7e154`.
///
/// **Wrong, and only reachable this way while a grazed ray's records stay in
/// the log.** When each ray keeps a log of its own, this pair stops reaching
/// it, and the judgment's reading wants a pin that hands it a log directly
/// (`docs/decisions.md` §17.8).
#[test]
fn a_model_declared_1e160_across_takes_the_same_union_with_the_rays_records_still_in_its_log() {
    for wedge_first in [true, false] {
        let case = format!("wedge first: {wedge_first}");
        let (mut model, wedge, block) =
            a_wedge_whose_floor_runs_through_a_blocks_edge_and_the_first_ray_target(1e160);
        let (first, second) = if wedge_first {
            (wedge, block)
        } else {
            (block, wedge)
        };
        let mut log = ProvenanceLog::new();
        let fused = model
            .fuse(
                &[first],
                &[second],
                Rule::Union,
                &Policy::certified(),
                &mut log,
            )
            .unwrap_or_else(|why| panic!("{case}: the judgment passes the union: {why}"));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{case}: the union is a valid solid: {why}"));
        assert_eq!(fused.solids().len(), 1, "{case}");
        let measured = volume_of(&model, fused.solids());
        assert!(
            (measured - the_pairs_union()).abs() <= 1e-12,
            "{case}: the union is {:e}, got {measured:e}",
            the_pairs_union()
        );
        assert_they_are_the_two_records_of_the_ray_that_grazed_the_edge(&case, &log);

        assert!(
            log.worst_wobble(100.0) > 0.0,
            "{case}: read at 100 the records are past a budget of zero"
        );
        assert!(
            log.worst_wobble(2.5e154) > 0.0,
            "{case}: read at 2.5e154 they still are"
        );
        assert_eq!(
            log.worst_wobble(2.7e154),
            0.0,
            "{case}: read at 2.7e154 they round to nothing"
        );
        assert_eq!(log.worst_wobble(1e160), 0.0, "{case}");
    }
}

/// The two `orient3d` records the block's first ray leaves when it grazes the
/// edge from `(1, 1, 0)` to `(1, 1, 1)`: neither proved, the edge's two ends
/// and the ray's target first, and the box the ray was fired from last.
fn assert_they_are_the_two_records_of_the_ray_that_grazed_the_edge(
    case: &str,
    records: &ProvenanceLog,
) {
    let the_edge = [
        PredicatePoint::Exact([1.0, 1.0, 0.0]),
        PredicatePoint::Exact([1.0, 1.0, 1.0]),
    ];
    let the_first_ray_target = PredicatePoint::Exact([
        2.0,
        1.0 + 0.618_033_988_749_895,
        1.0 + 0.414_213_562_373_095_1,
    ]);
    assert_eq!(records.len(), 2, "{case}: {records:?}");
    for record in records.records() {
        assert_eq!(record.predicate(), "orient3d", "{case}: {record:?}");
        assert!(
            record.approximation().is_none() && record.enclosure() > 0.0,
            "{case}: nothing proved it and no policy accepted it: {record:?}"
        );
        let inputs = record.inputs();
        assert_eq!(inputs.len(), 4, "{case}: {record:?}");
        assert!(
            the_edge.iter().all(|end| inputs[..2].contains(end)),
            "{case}: {record:?}"
        );
        assert_eq!(inputs[2], the_first_ray_target, "{case}: {record:?}");
        assert!(
            matches!(inputs[3], PredicatePoint::Constructed(_)),
            "{case}: {record:?}"
        );
    }
}

/// [`a_certified_union_is_refused_over_the_records_of_a_ray_it_threw_away`]'s
/// pair, on a model declared `size` across: a block `[0.3, 1] × [0, 1]`
/// extruded `1` high, and a wedge whose floor rises at [`SLOPE`] through the
/// block's edge at `x = 1, y = 1`, extruded `0.5` high from `z = 0.25`. The
/// block's box is `[0.3, 1] × [0, 1] × [0, 1]`, so `ray_targets`' first point
/// is `(2, 1 + 0.618…, 1 + 0.414…)`, and at `x = 2` the floor is at `1 +
/// SLOPE`, the same double.
fn a_wedge_whose_floor_runs_through_a_blocks_edge_and_the_first_ray_target(
    size: f64,
) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(size).expect("a working size"));
    let block = ProfileBuilder::new(vec![[0.3, 0.0], [1.0, 0.0], [1.0, 1.0], [0.3, 1.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let block = model
        .add_extrusion(&block, 1.0, &Isometry::identity())
        .expect("building it needs no decision");
    let wedge = ProfileBuilder::new(vec![
        [-1.0, 1.0 - 2.0 * SLOPE],
        [2.0, 1.0 + SLOPE],
        [2.0, 3.0],
        [-1.0, 3.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a quadrilateral");
    let wedge = model
        .add_extrusion(
            &wedge,
            0.5,
            &Isometry::translation(Vector::new(0.0, 0.0, 0.25)).expect("a finite placement"),
        )
        .expect("building it needs no decision");
    (model, wedge, block)
}

/// The slope of the wedge's floor, written so that `1 + SLOPE` is the double
/// `ray_targets` computes for its first target's `y`.
const SLOPE: f64 = (1.0 + 0.618_033_988_749_895) - 1.0;

/// What the pair's union takes up: the wedge, `3 + 0.75 SLOPE`, and the block,
/// `0.7`, less the prism they share, `0.1225 SLOPE`.
fn the_pairs_union() -> f64 {
    3.7 + 0.6275 * SLOPE
}

/// Two towers `1e308` tall, a certified fuse settles them, and nothing it
/// decided is left in the log.
///
/// **This was criterion 7's refusing half, and three times the refusal was a
/// gap.** At `1e200` it was `InPlane::of` stepping off the towers' top by a
/// fixed distance, which vanishes into a plane that far out (§19.5). At `1e308`
/// it was `orient3d`'s products overflowing, until its retry learned a second
/// power of two (§324.6). Then it was "a point inside a piece of a face", on the
/// quarter of the first tower's top that the second tower's wall cuts off: every
/// candidate there is half way between two corners at `z = 1e308`, and adding
/// the two first came to `2e308`, so each candidate was a box reaching from
/// `9e307` to infinity and no winding could place one. The middle is `1e308`,
/// a double, and halving first reaches it (`docs/decisions.md` §353). No
/// refusal by criterion 7's judgment whose answer does not exist is known
/// since. §17's exit table names what was tried.
///
/// An answer at all also says `InPlane::of`'s second reach ran: the first,
/// `1e308` further out along the top's normal, overflows, and without the
/// second the fuse refuses on "a point off a face's plane" before it reaches
/// any arithmetic.
///
/// The volume is not the oracle here, as it is for the companion below: the
/// walls' area adds up past the largest `f64`, and `measure` refuses with
/// `SurfaceAreaTooLargeToMeasure`. The top's outline is, and so is where a
/// point is — though not a point *on* the top, which `holds.rs`'s own step off
/// a plane at `1e308` cannot place yet (§353).
#[test]
fn two_towers_1e308_tall_are_united_by_proof_alone() {
    let mut model = a_model();
    let one = tower(&mut model, Vector::new(0.0, 0.0, 0.0), 1e308);
    let other = tower(&mut model, Vector::new(0.5, 0.5, 0.0), 1e308);

    let mut log = ProvenanceLog::new();
    let united = model
        .fuse(
            &[one],
            &[other],
            Rule::Union,
            &Policy::certified(),
            &mut log,
        )
        .expect("every decision the union of two towers needs is one f64 can prove");

    assert!(
        log.is_empty(),
        "a fuse of two lattice-aligned towers proves everything it decides: {log:?}"
    );
    let [solid] = united.solids() else {
        panic!("two towers that overlap unite into one: {united:?}");
    };
    let solid = model.solid(*solid).expect("this model made the key");
    assert_eq!(
        solid.faces().count(),
        10,
        "eight walls round the outline of two overlapping squares, a top and a bottom"
    );

    let tops: Vec<Vec<Vec<Point>>> = solid
        .faces()
        .map(|face| face.wires().map(|wire| wire.points().collect()).collect())
        .filter(|wires: &Vec<Vec<Point>>| {
            wires
                .iter()
                .flatten()
                .all(|corner| corner.coordinates()[2] == 1e308)
        })
        .collect();
    let [top] = tops.as_slice() else {
        panic!("one face across the top of both towers: {tops:?}");
    };
    let [outline] = top.as_slice() else {
        panic!("the top has no hole in it: {top:?}");
    };
    let mut corners: Vec<[f64; 2]> = outline
        .iter()
        .map(|corner| [corner.coordinates()[0], corner.coordinates()[1]])
        .collect();
    corners.sort_by(|one, other| one.partial_cmp(other).expect("corners are numbers"));
    assert_eq!(
        corners,
        [
            [0.0, 0.0],
            [0.0, 1.0],
            [0.5, 1.0],
            [0.5, 1.5],
            [1.0, 0.0],
            [1.0, 0.5],
            [1.5, 0.5],
            [1.5, 1.5],
        ],
        "the top keeps the quarter of the first tower's top outside the second, \
         the piece that used to refuse"
    );

    for (x, y, z, expected) in [
        (0.25, 0.25, 5e307, Containment::Inside),
        (0.75, 0.25, 5e307, Containment::Inside),
        (0.75, 0.75, 5e307, Containment::Inside),
        (1.25, 1.25, 5e307, Containment::Inside),
        (1.25, 0.25, 5e307, Containment::Outside),
        (0.25, 1.25, 5e307, Containment::Outside),
        (0.75, 0.25, 1.5e308, Containment::Outside),
        (0.75, 0.25, 0.0, Containment::OnBoundary),
    ] {
        let mut log = ProvenanceLog::new();
        assert_eq!(
            solid.classify_point(Point::new(x, y, z), &Policy::certified(), &mut log),
            expected,
            "({x}, {y}, {z:e})"
        );
        assert!(log.is_empty(), "({x}, {y}, {z:e}): {log:?}");
    }
}

/// The same pair a hundred powers of ten shorter, which a certified fuse settles
/// and used to refuse.
///
/// Two unit squares half a unit apart, each `1e200` tall: the union is two
/// towers' worth of space less the quarter they share, and every decision it
/// needs is one `orient3d` can prove. `1e200` squared is past the largest `f64`,
/// so the cascade's first two levels have nothing to say — but its exact level
/// does not multiply anything, and this is what it is for.
#[test]
fn a_pair_of_towers_a_hundred_powers_of_ten_shorter_is_settled_and_measured() {
    let mut model = a_model();
    let one = tower(&mut model, Vector::new(0.0, 0.0, 0.0), 1e200);
    let other = tower(&mut model, Vector::new(0.5, 0.5, 0.0), 1e200);

    let united = model
        .fuse(
            &[one],
            &[other],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("two overlapping towers are a union f64 can prove");

    assert_eq!(united.solids().len(), 1);
    let volume = volume_of(&model, united.solids());
    let two_towers_less_what_they_share = 1.75e200;
    assert!(
        (volume - two_towers_less_what_they_share).abs() < 1e-12 * two_towers_less_what_they_share,
        "two towers overlapping in a quarter of their base enclose {two_towers_less_what_they_share:e}, got {volume:e}"
    );
}

/// A frame, and a plug that fills one corner of its hole (§17.7's control).
///
/// Every coordinate here is a whole number, so a certified fuse has to decide
/// it. It did not. The plug's bottom face lies in the very plane the frame's
/// bottom face lies in, but over the frame's *hole*, where the frame has no
/// material — and classification read one triangle of that face's fan as proof
/// the point was on the face. The fan overlaps itself and covers the hole twice,
/// once each way, so only the whole outline can say; the outer half alone always
/// says yes. Every point of this piece is over the hole, so no choice of
/// candidate could have got round it.
#[test]
fn a_plug_filling_the_corner_of_a_frames_hole_unites_with_the_frame() {
    let mut model = a_model();
    let frame = frame(&mut model, 4.0, 5.0, 1.0);
    let plug = plug(&mut model, Vector::new(1.0, 1.0, 0.0));

    let mut log = ProvenanceLog::new();
    let united = model
        .fuse(
            &[frame],
            &[plug],
            Rule::Union,
            &Policy::certified(),
            &mut log,
        )
        .expect("every corner of a frame and a plug on the integer grid is exactly representable");

    assert!(log.is_empty(), "nothing here needed a policy to accept it");
    assert_eq!(united.solids().len(), 1);
    assert_eq!(
        volume_of(&model, united.solids()),
        4.0 * 5.0 - 2.0 * 3.0 + 1.0,
        "the frame's walls plus the plug that fills a corner of its hole"
    );
}

/// A frame with a rectangular hole one wall thick, extruded.
fn frame(model: &mut Model, width: f64, depth: f64, height: f64) -> SolidKey {
    let profile = ProfileBuilder::new(vec![[0.0, 0.0], [width, 0.0], [width, depth], [0.0, depth]])
        .with_hole(vec![
            [1.0, 1.0],
            [width - 1.0, 1.0],
            [width - 1.0, depth - 1.0],
            [1.0, depth - 1.0],
        ])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a frame");
    model
        .add_extrusion(
            &profile,
            height,
            &Isometry::translation(Vector::new(0.0, 0.0, 0.0)).expect("a finite placement"),
        )
        .expect("a frame extrudes")
}

/// A one by one by one block, put down at `at`.
fn plug(model: &mut Model, at: Vector) -> SolidKey {
    let profile = ProfileBuilder::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a unit square");
    model
        .add_extrusion(
            &profile,
            1.0,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("a plug extrudes")
}

/// A turned solid with a bit-identical copy of itself taken away.
///
/// `A \ A = ∅` is not a hard question and §17.1 makes it an easy one on purpose:
/// both solids are built from the same numbers and placed by the same
/// `Isometry`, so their stored planes agree bit for bit however the rotation
/// rounded. What used to refuse was the *ray*. Turned, every corner the fuse
/// solves for is a box, so a candidate inside a piece is a box too, and a box
/// sitting on the plane it was solved from straddles that plane — no ray can be
/// cast from it and classification honestly says so. The ray was never what
/// settles `ON`: two coincident faces are proved coincident from their planes'
/// coefficients, which no box widens.
#[test]
fn a_turned_solid_with_a_bit_identical_copy_taken_away_leaves_nothing() {
    let mut model = a_model();
    let one = plug(&mut model, Vector::new(0.0, 0.0, 0.0));
    let other = plug(&mut model, Vector::new(0.0, 0.0, 0.0));
    let axis =
        Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("a direction with length");
    let turn = Isometry::rotation(&axis, core::f64::consts::TAU / 64.0).expect("a rotation");
    model
        .place_solid(one, &turn)
        .expect("this model made the key");
    model
        .place_solid(other, &turn)
        .expect("this model made the key");

    let nothing = model
        .fuse(
            &[one],
            &[other],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid and a copy of it share every plane, turned or not");

    assert!(
        nothing.solids().is_empty(),
        "a solid with itself taken away leaves nothing, got {} solids",
        nothing.solids().len()
    );
}

/// Two ells flush against each other along their whole thickness, united both
/// ways round.
///
/// The tops and the bottoms of the two are coplanar and overlap, so the region
/// they share is cut out of both and kept once. Which of the two copies is kept
/// is whichever came first, and the two are cut by different sets of lines —
/// the argument's face never sees the lines the tool's own corners put on it —
/// so keeping one rather than the other used to give a different partition each
/// way round: twenty-three faces for `A ∪ B` and twenty-six for `B ∪ A`, both of
/// them the same solid.
///
/// Joining the coplanar pieces back into one face is what settles it. The answer
/// is then the eight faces the shape has, whichever way round it was asked for.
#[test]
fn uniting_two_ells_flush_against_each_other_gives_the_same_faces_either_way_round() {
    let (mut model, one, other) = two_ells();
    let forwards = fuse(&mut model, &[one], &[other], Rule::Union);
    assert_eq!(forwards.len(), 1);
    assert_eq!(
        model
            .solid(forwards[0])
            .expect("this model made the key")
            .faces()
            .count(),
        8,
        "an ell of one thickness has a top, a bottom and six walls"
    );
    let forwards = faces_of(&model, &forwards);

    let (mut model, one, other) = two_ells();
    let united = fuse(&mut model, &[other], &[one], Rule::Union);
    let backwards = faces_of(&model, &united);

    assert_eq!(
        forwards, backwards,
        "which side of the fuse a solid is on does not change the answer"
    );
}

/// An ell of two by three with a one by one bite out of its far corner, one
/// thick, and the same shape again one further along `y`.
fn two_ells() -> (Model, SolidKey, SolidKey) {
    let mut model = a_model();
    let one = ell(&mut model, [0.0, 0.0, 0.0], 1.0);
    let other = ell(&mut model, [0.0, 1.0, 0.0], 1.0);
    (model, one, other)
}

/// An ell of two by three, one thick, with a one by `bite_depth` bite out of its
/// far corner, put down at `at`.
fn ell(model: &mut Model, at: [f64; 3], bite_depth: f64) -> SolidKey {
    let outline = [
        [0.0, 0.0],
        [2.0, 0.0],
        [2.0, 3.0 - bite_depth],
        [1.0, 3.0 - bite_depth],
        [1.0, 3.0],
        [0.0, 3.0],
    ]
    .map(|[x, y]| [x + at[0], y + at[1]])
    .to_vec();
    let profile = ProfileBuilder::new(outline)
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound outline");
    let lift = Vector::new(0.0, 0.0, at[2]);
    model
        .add_extrusion(
            &profile,
            1.0,
            &Isometry::translation(lift).expect("a finite lift"),
        )
        .expect("an ell extrudes")
}

/// One face written out: the four numbers of its plane, then its rings of
/// corners, every number as its bits.
type WrittenFace = (Vec<u64>, Vec<Vec<[u64; 3]>>);

/// Every face of an answer, written out so that two answers can be compared
/// whatever order the arena put them in.
fn faces_of(model: &Model, solids: &[SolidKey]) -> Vec<WrittenFace> {
    let mut written: Vec<WrittenFace> = solids
        .iter()
        .flat_map(|&solid| model.solid(solid).expect("this model made the key").faces())
        .map(|face| {
            let plane = face
                .plane()
                .coefficients()
                .iter()
                .map(|&it| it.to_bits())
                .collect();
            let mut wires: Vec<Vec<[u64; 3]>> = face
                .wires()
                .map(|wire| {
                    let mut ring: Vec<[u64; 3]> = wire
                        .points()
                        .map(|at| at.coordinates().map(f64::to_bits))
                        .collect();
                    let smallest = ring
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, corner)| **corner)
                        .map_or(0, |(position, _)| position);
                    ring.rotate_left(smallest);
                    ring
                })
                .collect();
            wires.sort_unstable();
            (plane, wires)
        })
        .collect();
    written.sort_unstable();
    written
}

/// The answer to a fuse can be fused again, which it could not when two of its
/// faces sat on one plane.
///
/// Three boxes of side two: the first at the origin, the second overlapping it
/// along `x`, the third overlapping what the two of them make. Uniting the first
/// two leaves a three by two by two box, and the corners at `x = 1` and `x = 2`
/// along its bottom edges used to have four faces round them and only two
/// planes — a top and a bottom cut into halves that were never joined back up.
/// §17.1 re-derives every corner as the meet of three planes at the next fuse's
/// entry, so those corners could not be placed and the second union came back
/// `PlanesDoNotMeetInAPoint` with a determinant of exactly zero.
#[test]
fn a_union_can_be_united_again_because_every_corner_of_it_still_has_three_planes() {
    let mut model = a_model();
    let one = cube(&mut model, 2.0, Vector::new(0.0, 0.0, 0.0));
    let other = cube(&mut model, 2.0, Vector::new(1.0, 0.0, 0.0));
    let third = cube(&mut model, 2.0, Vector::new(1.0, 1.0, 1.0));

    let first = fuse(&mut model, &[one], &[other], Rule::Union);
    assert_eq!(volume_of(&model, &first), 12.0);

    let again = fuse(&mut model, &first, &[third], Rule::Union);
    assert_eq!(
        model
            .solid(first[0])
            .expect("this model made the key")
            .faces()
            .count(),
        6,
        "two boxes overlapping along one axis unite into a box"
    );
    assert_eq!(again.len(), 1);
    assert_eq!(volume_of(&model, &again), 12.0 + 8.0 - 2.0);
}

/// A hole of a face that no cut line touches has to be put in the piece that
/// surrounds it, and one corner of it having nothing to see is not the end of
/// the search.
///
/// A frame three by five with a one by three hole, an ell standing on top of one
/// side of it, and a block a long way above that touches neither. Uniting the
/// frame and the ell joins their tops into one face, and that face still has the
/// frame's hole in it. The block is far enough away that nothing cuts that face,
/// so the hole is a ring the drawing does not join to anything and the piece
/// round it has to be found by shooting a ray. From the first corner of the hole
/// every ray runs into the end of an arc and says nothing; from another corner
/// of the same hole one of them answers straight away.
#[test]
fn a_hole_nothing_cuts_is_placed_from_whichever_of_its_corners_can_see_out() {
    let mut model = a_model();
    let frame = frame(&mut model, 3.0, 5.0, 2.0);
    let ell = ell(&mut model, [2.0, 3.0, 1.0], 2.0);
    let far_away = cube(&mut model, 1.0, Vector::new(0.0, 0.0, 3.0));

    let with_the_ell = fuse(&mut model, &[frame], &[ell], Rule::Union);
    let frame_and_ell = 24.0 + 4.0 - 2.0;
    assert_eq!(volume_of(&model, &with_the_ell), frame_and_ell);

    let with_the_block = fuse(&mut model, &with_the_ell, &[far_away], Rule::Union);
    assert_eq!(
        with_the_block.len(),
        2,
        "the block touches nothing, so it stays a solid of its own"
    );
    assert_eq!(volume_of(&model, &with_the_block), frame_and_ell + 1.0);
}

/// A ray looking for the piece a hole sits in is stopped by an arc it grazes
/// the end of, but only by one that is ahead of it.
///
/// Two combs of three teeth, the second turned so its teeth point at the first
/// one's base and slid so two of them land on it. The gap between those two
/// teeth is then closed off on all four sides, so the top and bottom of the
/// answer each have a hole in them that no cut line touches. Uniting that with a
/// block sitting well above it — which cuts nothing and touches nothing — has to
/// place the hole in the piece around it, and the only ray that can do it grazes
/// the corner of a tooth two rows behind where it starts.
#[test]
fn a_hole_is_placed_by_a_ray_that_grazes_an_arc_behind_it() {
    let mut model = a_model();
    let one = comb(&mut model, [0.0, 0.0]);
    let other = comb(&mut model, [-2.0, -2.0]);
    let far_away = cube(&mut model, 1.0, Vector::new(0.0, 0.0, 2.0));

    let meshed = fuse(&mut model, &[one, other], &[], Rule::Union);
    assert_eq!(meshed.len(), 1, "two teeth of one comb land on the other");

    let with_the_block = fuse(&mut model, &[meshed[0], far_away], &[], Rule::Union);
    assert_eq!(
        with_the_block.len(),
        2,
        "the block touches nothing, so it stays a solid of its own"
    );
    assert_eq!(
        volume_of(&model, &with_the_block),
        volume_of(&model, &meshed) + 1.0
    );
}

/// Three square teeth one high standing on a base one thick, five across,
/// extruded by one and put down at `at`.
fn comb(model: &mut Model, at: [f64; 2]) -> SolidKey {
    let outline = [
        [0.0, 0.0],
        [5.0, 0.0],
        [5.0, 2.0],
        [4.0, 2.0],
        [4.0, 1.0],
        [3.0, 1.0],
        [3.0, 2.0],
        [2.0, 2.0],
        [2.0, 1.0],
        [1.0, 1.0],
        [1.0, 2.0],
        [0.0, 2.0],
    ]
    .map(|[x, y]| [x + at[0], y + at[1]])
    .to_vec();
    let profile = ProfileBuilder::new(outline)
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound comb");
    model
        .add_extrusion(&profile, 1.0, &Isometry::identity())
        .expect("a comb extrudes")
}

/// A 2×2 square extruded by one, with an extra profile point halfway along the
/// bottom wall and another halfway along the top, put down at `at`.
///
/// `ProfileBuilder` accepts three points in a row and `Model::validate` calls
/// the extrusion sound, so this is a solid any caller can build and hand to the
/// fuse. Its two bottom side faces sit on one plane and share an edge, and the
/// corner between them has two planes round it and not three — which is what
/// §17.1's re-derivation cannot solve, and why the fuse joins such faces on the
/// way in as well as on the way out.
fn a_box_with_a_point_in_the_middle_of_two_of_its_walls(
    model: &mut Model,
    at: [f64; 2],
) -> SolidKey {
    let outline = [
        [0.0, 0.0],
        [1.0, 0.0],
        [2.0, 0.0],
        [2.0, 2.0],
        [1.0, 2.0],
        [0.0, 2.0],
    ]
    .map(|[x, y]| [x + at[0], y + at[1]])
    .to_vec();
    let profile = ProfileBuilder::new(outline)
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square with a point halfway along two of its walls");
    model
        .add_extrusion(&profile, 1.0, &Isometry::identity())
        .expect("it extrudes")
}

/// The pair does not touch at all, so there is nothing here for the fuse to
/// work out — and it still refused every rule before the input was joined,
/// because the refusal is about one solid's own corners and not about the pair.
#[test]
fn a_solid_with_a_profile_point_in_the_middle_of_a_wall_fuses_with_one_it_does_not_touch() {
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        let mut model = a_model();
        let lumpy = a_box_with_a_point_in_the_middle_of_two_of_its_walls(&mut model, [0.0, 0.0]);
        let plain = cube(&mut model, 1.0, Vector::new(5.0, 5.0, 0.0));

        let answer = fuse(&mut model, &[lumpy], &[plain], rule);

        match rule {
            Rule::Union => {
                assert_eq!(answer.len(), 2, "nothing joined them, so they stay two");
                assert_eq!(volume_of(&model, &answer), 4.0 + 1.0);
                let walls: usize = model
                    .solid(answer[0])
                    .expect("this model made the key")
                    .faces()
                    .count();
                assert_eq!(
                    walls, 6,
                    "the two side faces on one plane came through as the one face they are"
                );
            }
            Rule::Intersection => assert!(answer.is_empty()),
            Rule::Difference => {
                assert_eq!(answer.len(), 1);
                assert_eq!(volume_of(&model, &answer), 4.0);
            }
            other => panic!("a rule nothing here knows: {other:?}"),
        }
    }
}

/// The same solid with a cube driven through the middle of its joined wall, so
/// the face the joining made is cut in its turn. Everything the joining dropped
/// — a corner and an edge — sat exactly where the cut lands.
#[test]
fn a_solid_with_a_profile_point_in_the_middle_of_a_wall_can_be_cut_across_that_wall() {
    let straddling = Vector::new(0.5, -0.5, 0.0);
    let overlap = 1.0 * 0.5 * 1.0;

    let mut model = a_model();
    let lumpy = a_box_with_a_point_in_the_middle_of_two_of_its_walls(&mut model, [0.0, 0.0]);
    let plain = cube(&mut model, 1.0, straddling);
    let united = fuse(&mut model, &[lumpy], &[plain], Rule::Union);
    assert_eq!(united.len(), 1);
    assert_eq!(volume_of(&model, &united), 4.0 + 1.0 - overlap);

    let mut model = a_model();
    let lumpy = a_box_with_a_point_in_the_middle_of_two_of_its_walls(&mut model, [0.0, 0.0]);
    let plain = cube(&mut model, 1.0, straddling);
    let shared = fuse(&mut model, &[lumpy], &[plain], Rule::Intersection);
    assert_eq!(volume_of(&model, &shared), overlap);

    let mut model = a_model();
    let lumpy = a_box_with_a_point_in_the_middle_of_two_of_its_walls(&mut model, [0.0, 0.0]);
    let plain = cube(&mut model, 1.0, straddling);
    let bitten = fuse(&mut model, &[lumpy], &[plain], Rule::Difference);
    assert_eq!(volume_of(&model, &bitten), 4.0 - overlap);
}

/// Both sides of the fuse have a point in the middle of a wall, so both get
/// joined on the way in and neither is a plain box the other could lean on.
#[test]
fn two_solids_that_both_have_a_profile_point_in_the_middle_of_a_wall_unite() {
    let overlap = 1.5 * 1.5 * 1.0;

    let mut model = a_model();
    let one = a_box_with_a_point_in_the_middle_of_two_of_its_walls(&mut model, [0.0, 0.0]);
    let other = a_box_with_a_point_in_the_middle_of_two_of_its_walls(&mut model, [0.5, 0.5]);

    let united = fuse(&mut model, &[one], &[other], Rule::Union);
    assert_eq!(united.len(), 1);
    assert_eq!(volume_of(&model, &united), 4.0 + 4.0 - overlap);

    let mut model = a_model();
    let one = a_box_with_a_point_in_the_middle_of_two_of_its_walls(&mut model, [0.0, 0.0]);
    let other = a_box_with_a_point_in_the_middle_of_two_of_its_walls(&mut model, [0.5, 0.5]);

    let shared = fuse(&mut model, &[one], &[other], Rule::Intersection);
    assert_eq!(volume_of(&model, &shared), overlap);
}

/// §17.4's amendment of 2026-08-30 already made every *answer* free of two
/// faces on one plane sharing an edge. Fusing such a solid and then fusing the
/// answer again is what pins that inputs and answers now meet the same
/// standard: the second fuse joins nothing, because there is nothing left to
/// join.
#[test]
fn uniting_a_solid_with_a_point_in_the_middle_of_a_wall_gives_an_answer_that_unites_again() {
    let mut model = a_model();
    let lumpy = a_box_with_a_point_in_the_middle_of_two_of_its_walls(&mut model, [0.0, 0.0]);
    let straddling = cube(&mut model, 1.0, Vector::new(0.5, -0.5, 0.0));
    let corner = cube(&mut model, 1.0, Vector::new(1.5, 1.5, 0.5));

    let once = fuse(&mut model, &[lumpy], &[straddling], Rule::Union);
    assert_eq!(once.len(), 1);
    assert_eq!(volume_of(&model, &once), 4.0 + 1.0 - 0.5);

    let twice = fuse(&mut model, &once, &[corner], Rule::Union);
    assert_eq!(twice.len(), 1);
    assert_eq!(volume_of(&model, &twice), 4.5 + 1.0 - 0.125);
}

/// Joining two input faces into one must not lose either of them from the
/// history. Both survive, and both name the single face the answer has where
/// the two of them were.
#[test]
fn both_input_faces_that_were_joined_on_the_way_in_name_the_face_they_became() {
    let mut model = a_model();
    let lumpy = a_box_with_a_point_in_the_middle_of_two_of_its_walls(&mut model, [0.0, 0.0]);
    let plain = cube(&mut model, 1.0, Vector::new(5.0, 5.0, 0.0));
    let walls: Vec<FaceKey> = model
        .solid(lumpy)
        .expect("this model made the key")
        .faces()
        .map(|face| face.key())
        .collect();
    assert_eq!(
        walls.len(),
        8,
        "six walls, two of which are halves of one, plus a top and a bottom"
    );

    let fused = model
        .fuse(
            &[lumpy],
            &[plain],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a pair that does not touch");

    let mut faces_of_the_answer: Vec<FaceKey> = Vec::new();
    for wall in walls {
        let images = fused.images_of_face(wall).expect("this model made the key");
        assert_eq!(images.len(), 1, "no wall of the input was lost");
        if !faces_of_the_answer.contains(&images[0]) {
            faces_of_the_answer.push(images[0]);
        }
    }
    assert_eq!(
        faces_of_the_answer.len(),
        6,
        "the eight walls came through as the six faces the shape actually has"
    );
}

/// The deflection the measurements here are asked for.
///
/// Every solid in this file is a polyhedron, and a polyhedron ignores it: its
/// own fan is the answer rather than a picture of one (`docs/decisions.md`
/// §25.4).
fn a_deflection() -> Deflection {
    Deflection::of(0.1).expect("a tenth of a millimetre is a length")
}
