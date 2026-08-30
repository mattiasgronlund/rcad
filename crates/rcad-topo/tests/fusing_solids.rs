//! Cutting two groups of solids against each other and keeping what a rule
//! asks for.
//!
//! The volumes are the oracle throughout, because they are the one thing that
//! is true whatever the answer is made of: `vol(A ∪ B) + vol(A ∩ B) =
//! vol(A) + vol(B)` holds for every pair, and for the lattice-aligned pairs
//! here every corner is exactly representable, so the arithmetic is exact and
//! the numbers can be compared with `==`.

use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
use rcad_predicates::{Approximation, Cause, Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey, VertexKey};

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// A cube of the given side, with its low corner at `at`.
fn cube(model: &mut Model, side: f64, at: Vector) -> SolidKey {
    let cube = model
        .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    model.place_solid(
        cube,
        &Isometry::translation(at).expect("a finite placement"),
    );
    cube
}

/// A unit square extruded `1e200` high and put down at `at`. The coordinates
/// then run from one to `1e200`, which is far past where `f64` can multiply two
/// of them and still have an exact answer to fall back on.
fn tower(model: &mut Model, at: Vector) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a unit square");
    model
        .add_extrusion(
            &square,
            1e200,
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
        .map(|&solid| model.solid(solid).measure().volume())
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
    let solid = model.solid(hollow[0]);
    assert_eq!(
        solid.shells().count(),
        2,
        "the outside of the block and the skin of the hole it now has"
    );
    assert_eq!(solid.measure().volume(), 64.0 - 8.0);
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
    let images: usize = model.solid(fused.solids()[0]).faces().count();
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
    for face in model.solid(fused.solids()[0]).faces() {
        assert_eq!(
            model.face(face.key()).approximation(),
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
        for face in model.solid(solid).faces() {
            let images = fused.images_of_face(face.key());
            assert_eq!(images.len(), 1);
            assert_ne!(images[0], face.key());
            assert_eq!(model.face(images[0]).plane(), face.plane());
        }
    }
    assert_eq!(fused.deleted_faces().count(), 0);
    assert_eq!(fused.modified_faces().count(), 12);
}

/// A face that is cut comes through as the pieces it was cut into, and every
/// face of the answer is one of them.
///
/// The block's faces are cut by the bubble's six planes, which reach past the
/// bubble itself — a cut is by a plane, not by the patch of it a face covers.
/// The bubble's own faces are cut by nothing, since none of the block's planes
/// passes through the middle of the block where the bubble sits.
#[test]
fn a_face_that_is_cut_comes_through_as_the_pieces_it_was_cut_into() {
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
    for face in model.solid(block).faces() {
        let mine = fused.images_of_face(face.key()).len();
        assert!(mine > 1, "the bubble's planes cut every side of the block");
        images += mine;
    }
    for face in model.solid(bubble).faces() {
        assert_eq!(
            fused.images_of_face(face.key()).len(),
            1,
            "every face of the bubble becomes one face of the hole's skin"
        );
        images += 1;
    }
    assert_eq!(
        images,
        model.solid(fused.solids()[0]).faces().count(),
        "every face of the answer is the image of exactly one input face"
    );
    assert_eq!(fused.deleted_faces().count(), 0);

    for vertex in corners_of(&model, bubble) {
        assert_eq!(fused.images_of_vertex(vertex).len(), 1);
    }
}

/// Every corner of a solid, once each.
fn corners_of(model: &Model, solid: SolidKey) -> Vec<VertexKey> {
    let mut corners = Vec::new();
    for face in model.solid(solid).faces() {
        for wire in model.face(face.key()).wires() {
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

    for face in model.solid(bubble).faces() {
        assert!(
            fused.images_of_face(face.key()).is_empty(),
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
    model.place_solid(one, &turn);
    model.place_solid(other, &turn);

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

/// The other half of criterion 7. Two towers `1e200` tall overlap, and nothing
/// in `f64` can multiply coordinates that far apart without overflowing, so a
/// decision the answer depends on cannot be proved — and a certified policy
/// refuses rather than guessing.
#[test]
fn a_fuse_that_needs_a_decision_nothing_can_prove_refuses() {
    let mut model = a_model();
    let one = tower(&mut model, Vector::new(0.0, 0.0, 0.0));
    let other = tower(&mut model, Vector::new(0.5, 0.5, 0.0));

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
                !records.is_empty(),
                "the records say what could not be settled"
            );
        }
        other => panic!("nothing in f64 can settle that pair: {other:?}"),
    }
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
    model.place_solid(one, &turn);
    model.place_solid(other, &turn);

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
