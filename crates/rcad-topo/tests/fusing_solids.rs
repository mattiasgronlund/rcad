//! Cutting two groups of solids against each other and keeping what a rule
//! asks for.
//!
//! The volumes are the oracle throughout, because they are the one thing that
//! is true whatever the answer is made of: `vol(A ∪ B) + vol(A ∩ B) =
//! vol(A) + vol(B)` holds for every pair, and for the lattice-aligned pairs
//! here every corner is exactly representable, so the arithmetic is exact and
//! the numbers can be compared with `==`.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Approximation, Cause, Policy, ProvenanceLog};
use rcad_topo::{Error, FaceKey, Model, ProfileBuilder, Rule, SolidKey, VertexKey};

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

/// The other half of criterion 7. Two towers `1e308` tall overlap, `orient3d`
/// cannot multiply coordinates that far apart without overflowing, so a decision
/// the answer depends on cannot be proved — and a certified policy refuses
/// rather than guessing.
///
/// **This test asked the same question at `1e200` and got the right answer for
/// the wrong reason.** What refused there was not the arithmetic: it was
/// `InPlane::of` stepping off a plane by a fixed distance, which vanishes into a
/// plane that far out, so the fuse never reached a predicate at all. The
/// companion below is the same pair at `1e200`, now settled and measured, and
/// `a_solid_a_long_way_from_the_origin.rs` is where that defect is written up. A
/// refusal is only worth having as a demonstration when it is the arithmetic
/// that refused, which is the distinction `docs/decisions.md` §15.8 draws
/// between a refusal and a deserved one.
#[test]
fn a_fuse_that_needs_a_decision_nothing_can_prove_refuses() {
    let mut model = a_model();
    let one = tower(&mut model, Vector::new(0.0, 0.0, 0.0), 1e308);
    let other = tower(&mut model, Vector::new(0.5, 0.5, 0.0), 1e308);

    let refused = model.fuse(
        &[one],
        &[other],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    match refused {
        Err(Error::Undecidable { records }) => {
            assert!(
                records
                    .records()
                    .iter()
                    .any(|record| record.predicate() == "orient3d"),
                "the records name the predicate that could not be settled, and it is the \
                 one whose products overflow: {records:?}"
            );
            assert!(
                records
                    .records()
                    .iter()
                    .all(|record| record.predicate() != "a point off a face's plane"),
                "the fuse got as far as the arithmetic: a plane at 1e308 is what \
                 `InPlane::of`'s second reach — the same distance towards the origin \
                 instead of away from it — is for, and this is where it runs: {records:?}"
            );
        }
        other => panic!("nothing in f64 can settle that pair: {other:?}"),
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
