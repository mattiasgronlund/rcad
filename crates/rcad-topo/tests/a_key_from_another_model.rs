//! A key made by one model, used against another.
//!
//! `slotmap`'s generation counter catches a key whose slot was freed and
//! refilled in the same model. It cannot see a key from another live model
//! reaching a live slot in this one, and slot zero is a slot every model has —
//! so the wrong answer this is about is a plausible one and not a crash. Two
//! models each holding one block: the small one's key measured the large one's
//! volume, and a fuse handed the small one's key answered `Ok` naming no solids
//! at all (`docs/decisions.md` §82.10).
//!
//! §84 gave every key a model number in every build and made those two panic.
//! §86 made them refuse: which model a key came from is data this crate did not
//! choose, so it is bad caller input, and §4.1 answers bad caller input with
//! `Err`. Every call below reads and writes nothing before it says so.
//!
//! # Why this is not a tautology
//!
//! Each test asks the same question twice — once with a stranger's key and once
//! with the model's own. **The second half is the guard.** A build in which
//! every one of these refused would pass the first half of every test here and
//! be useless; a build in which two ids compare equal whichever models they came
//! from — which is what §84 fixed — passes the second half and fails the first.
//! Neither half means anything without the other.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Approximation, Cause, Policy, ProvenanceLog};
use rcad_topo::{
    CoedgeKey, EdgeKey, Error, FaceKey, Model, Rule, ShellKey, SolidKey, VertexKey, WireKey,
};

/// Which kind of entity a foreign-key refusal named, or `None` when the call
/// was not refused at all.
///
/// Any other error is this test asking the wrong question, so it fails here
/// rather than being folded into `None` and read as an answer.
fn what_it_refused<T>(answer: Result<T, Error>) -> Option<&'static str> {
    match answer {
        Err(Error::KeyFromAnotherModel { entity }) => Some(entity),
        Ok(_) => None,
        Err(other) => panic!("expected a foreign-key refusal or an answer, got {other:?}"),
    }
}

#[test]
fn a_key_of_every_kind_is_refused_by_the_wrong_model_and_taken_by_the_right_one() {
    let (mine, small) = a_model_holding_one_cube(1.0);
    let (theirs, _) = a_model_holding_one_cube(10.0);

    for (against, refuses) in [(&theirs, true), (&mine, false)] {
        assert_eq!(
            what_it_refused(against.solid(small.solid)),
            refuses.then_some("solid")
        );
        assert_eq!(
            what_it_refused(against.shell(small.shell)),
            refuses.then_some("shell")
        );
        assert_eq!(
            what_it_refused(against.face(small.face)),
            refuses.then_some("face")
        );
        assert_eq!(
            what_it_refused(against.wire(small.wire)),
            refuses.then_some("wire")
        );
        assert_eq!(
            what_it_refused(against.coedge(small.coedge)),
            refuses.then_some("coedge")
        );
        assert_eq!(
            what_it_refused(against.edge(small.edge)),
            refuses.then_some("edge")
        );
        assert_eq!(
            what_it_refused(against.vertex(small.vertex)),
            refuses.then_some("vertex")
        );
    }
}

/// The wrong answer this replaced, measured: a one millimetre cube's key read
/// 1000 mm³ off a model holding a ten millimetre one.
#[test]
fn another_models_solid_key_does_not_measure_this_models_solid() {
    let (_mine, small) = a_model_holding_one_cube(1.0);
    let (theirs, large) = a_model_holding_one_cube(10.0);

    assert_eq!(what_it_refused(theirs.solid(small.solid)), Some("solid"));

    let its_own = theirs
        .solid(large.solid)
        .expect("this model made the key")
        .measure(
            Deflection::for_scale(theirs.scale()),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a cube is a polyhedron and measures exactly")
        .volume();
    assert_eq!(
        its_own, 1000.0,
        "the solid the stranger's key used to resolve to"
    );
}

#[test]
fn another_models_solid_key_moves_nothing_and_leaves_the_model_byte_identical() {
    let (_mine, small) = a_model_holding_one_cube(1.0);
    let (mut theirs, large) = a_model_holding_one_cube(10.0);
    let before = bytes_of(&theirs);
    let shift = Isometry::translation(Vector::new(3.0, 0.0, 0.0)).expect("a finite shift");

    let refused = theirs.place_solid(small.solid, &shift);

    assert_eq!(what_it_refused(refused), Some("solid"));
    assert_eq!(
        bytes_of(&theirs),
        before,
        "a refused placement wrote something"
    );

    theirs
        .place_solid(large.solid, &shift)
        .expect("this model made the key");
    assert_ne!(
        bytes_of(&theirs),
        before,
        "the same call with this model's own key moved nothing either, so the \
         comparison above proves nothing"
    );
}

#[test]
fn a_fuse_given_another_models_solid_refuses_and_writes_nothing() {
    let (_mine, small) = a_model_holding_one_cube(1.0);
    let (mut theirs, large) = a_model_holding_one_cube(10.0);
    let before = bytes_of(&theirs);

    let refused = theirs.fuse(
        &[small.solid],
        &[large.solid],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    assert_eq!(what_it_refused(refused), Some("solid"));
    assert_eq!(bytes_of(&theirs), before, "a refused fuse wrote something");
}

/// A stranger's key is refused where it would otherwise have been *answered*.
///
/// This is the one place the old panic was hiding a second wrong answer behind
/// the first. `images_of_face` hands back an empty slice for an input face the
/// fuse deleted, so a stranger's slot number missing from the map reads as
/// "deleted" — a definite answer about a face this fuse never saw.
#[test]
fn what_became_of_another_models_face_is_refused_rather_than_answered_deleted() {
    let (_mine, small) = a_model_holding_one_cube(1.0);
    let (mut theirs, large) = a_model_holding_one_cube(10.0);
    let overlapping = a_second_cube_half_a_side_along(&mut theirs);

    let fused = theirs
        .fuse(
            &[large.solid],
            &[overlapping],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("two overlapping cubes on the lattice unite");

    assert_eq!(
        what_it_refused(fused.images_of_face(small.face)),
        Some("face")
    );
    assert_eq!(
        what_it_refused(fused.images_of_edge(small.edge)),
        Some("edge")
    );
    assert_eq!(
        what_it_refused(fused.images_of_vertex(small.vertex)),
        Some("vertex")
    );

    assert_eq!(what_it_refused(fused.images_of_face(large.face)), None);
    assert_eq!(what_it_refused(fused.images_of_edge(large.edge)), None);
    assert_eq!(what_it_refused(fused.images_of_vertex(large.vertex)), None);
}

#[test]
fn tagging_another_models_entity_with_slop_is_refused() {
    let (_mine, small) = a_model_holding_one_cube(1.0);
    let (mut theirs, large) = a_model_holding_one_cube(10.0);
    let tag = Approximation::new(1e-9, Cause::TolerantFallback);
    let before = bytes_of(&theirs);

    assert_eq!(
        what_it_refused(theirs.set_vertex_approximation(small.vertex, tag)),
        Some("vertex")
    );
    assert_eq!(
        what_it_refused(theirs.set_edge_approximation(small.edge, tag)),
        Some("edge")
    );
    assert_eq!(
        what_it_refused(theirs.set_face_approximation(small.face, tag)),
        Some("face")
    );
    assert_eq!(
        bytes_of(&theirs),
        before,
        "a refused tag wrote something, or moved worst_approximation"
    );

    theirs
        .set_vertex_approximation(large.vertex, tag)
        .expect("this model made the key");
    assert_eq!(theirs.worst_approximation(), 1e-9);
}

/// One key of every kind, all seven off the same solid.
struct EveryKind {
    solid: SolidKey,
    shell: ShellKey,
    face: FaceKey,
    wire: WireKey,
    coedge: CoedgeKey,
    edge: EdgeKey,
    vertex: VertexKey,
}

/// A model of its own holding one cube, and a key of every kind off it.
///
/// The cube is the model's first solid, so its keys name slot zero of every
/// arena — which is a slot the other model has too. That is what makes a
/// stranger's key resolve to something real rather than to nothing.
fn a_model_holding_one_cube(side: f64) -> (Model, EveryKind) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = model
        .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");

    let keys = {
        let view = model.solid(solid).expect("this model made the key");
        let shell = view.shells().next().expect("a cube has one shell");
        let face = shell.faces().next().expect("a cube has faces");
        let wire = face.wires().next().expect("a face has an outer wire");
        let coedge = wire.coedges().next().expect("a wire has coedges");
        EveryKind {
            solid,
            shell: shell.key(),
            face: face.key(),
            wire: wire.key(),
            coedge: coedge.key(),
            edge: coedge.edge().key(),
            vertex: coedge.start().key(),
        }
    };
    (model, keys)
}

/// A second ten millimetre cube, moved five along x so that it overlaps the
/// first on the lattice rather than sitting exactly on it.
fn a_second_cube_half_a_side_along(model: &mut Model) -> SolidKey {
    let cube = model
        .add_cube(10.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a second cube of side ten");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(5.0, 0.0, 0.0)).expect("a finite shift"),
        )
        .expect("this model made the key");
    cube
}

/// The whole model as bytes, for asking whether a refused call wrote anything.
///
/// Stronger than checking the entity counts or one solid's volume: a placement
/// composed onto a shell, a tag on a vertex and a bumped
/// `worst_approximation` all move these bytes and none of them moves a count.
fn bytes_of(model: &Model) -> Vec<u8> {
    postcard::to_allocvec(model).expect("a model writes out")
}
