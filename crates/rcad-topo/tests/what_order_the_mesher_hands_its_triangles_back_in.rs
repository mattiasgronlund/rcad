//! The order [`Mesh::triangles`] hands its triangles back in, which is published.
//!
//! `Mesh::triangles` promises two things and only the second was guarded:
//! *"Faces in the solid's stored order, and within a face whatever the
//! triangulator gives back for a patch built the same way — so two runs produce
//! the same triangles in the same order"* (`docs/decisions.md` §25.7). A
//! published order stands where a published bound stands (§15.6), and a consumer
//! outside this repo has deleted a defensive sort on the strength of the first
//! half.
//!
//! Nothing bit it before. `meshing_a_curved_solid.rs`'s
//! `every_triangle_names_the_face_it_came_from` collects the faces into a
//! `HashSet`, which stays green under every permutation of them, and
//! `measure.rs` leans on the order only for two runs agreeing — which a
//! stable-but-different order still does. So a reordering would leave rcad's own
//! sums bit for bit correct and break the promise anyway.
//!
//! # Two claims, not one
//!
//! *Together* is that a face's triangles arrive in one block and that face never
//! comes back afterwards. *In stored order* is which block comes first, and it
//! is strictly the stronger of the two. They are two tests here because they
//! break differently and a reader has to be able to tell which one did (§51.10):
//! "the blocks moved" is a bug a consumer reading spans survives, and "one
//! face's triangles are now mixed into another's" is not.
//!
//! **Nothing here compares sets and nothing here sorts.** What is checked is the
//! run of faces the triangles arrive in, read off in arrival order.
//!
//! **And the stored-order test does not ask `SolidView::faces` twice.** Both
//! `SolidView::mesh` and "the solid's stored order" walk that one iterator, so a
//! test comparing only those two would move with it and stay green while the
//! promise broke. So it also reads the arrival order off the **geometry**: a
//! bar's blocks are a plane facing down, a plane facing up and then a face that
//! is no plane at all, and a cube's are its six exact outward normals. That is a
//! claim about the shape, and an iterator cannot satisfy it by agreeing with
//! itself.

use rcad_geom::{Deflection, Isometry, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{FaceKey, Mesh, Model, SolidKey};

const RADIUS: f64 = 3.0;
const HEIGHT: f64 = 5.0;
const SIDE: f64 = 2.0;

/// The weaker half: a face is cut up once, and once it is done with it is done.
///
/// This is what a consumer walking the triangles and starting a new span
/// whenever the face changes actually needs. It says nothing about which face
/// comes first, so a permutation of whole blocks leaves it green — which is the
/// point of it being its own test.
#[test]
fn a_faces_triangles_arrive_in_one_block_and_that_face_never_comes_back() {
    for (what, model, solid) in [a_round_bar(), a_cube()] {
        let arrived = the_faces_in_the_order_their_triangles_arrive(&meshed(&model, solid));
        assert!(
            arrived.len() > 1,
            "{what}: one block is in one block whatever the mesher does, so a mesh \
             with one block cannot tell anybody anything"
        );
        for (block, face) in arrived.iter().enumerate() {
            assert!(
                !arrived[..block].contains(face),
                "{what}: {face:?} was cut up, another face's triangles followed, and \
                 then {face:?}'s came back again at block {block} — a consumer \
                 reading spans is handed that face twice"
            );
        }
    }
}

/// The stronger half, for a bar: the blocks arrive bottom cap, top cap, wall.
///
/// A bar is the smallest solid whose faces cannot be mistaken for one another —
/// two planes facing opposite ways and a cylinder — so any reordering of the
/// three moves one of them past another and both halves of this see it.
#[test]
fn a_bars_blocks_arrive_in_the_order_the_solid_stores_its_faces() {
    let (what, model, bar) = a_round_bar();
    let arrived = the_faces_in_the_order_their_triangles_arrive(&meshed(&model, bar));
    let stored: Vec<FaceKey> = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .map(|face| face.key())
        .collect();

    assert_eq!(stored.len(), 3, "{what} has two caps and a wall");
    assert_eq!(
        arrived, stored,
        "{what}: the blocks of triangles must arrive in the solid's stored face order"
    );
    assert_eq!(
        which_way_each_of_them_faces(&model, &arrived),
        vec![Some([0.0, 0.0, -1.0]), Some([0.0, 0.0, 1.0]), None],
        "{what}: the blocks arrive bottom cap, then top cap, then the wall"
    );
}

/// The stronger half, for a cube: six faces, so there are 719 ways to be wrong.
///
/// A different builder from the bar's — an extrusion of a square — and six
/// planes whose outward normals are exact, which is what lets the arrival order
/// be named in full with no tolerance anywhere.
#[test]
fn a_cubes_blocks_arrive_in_the_order_the_solid_stores_its_faces() {
    let (what, model, cube) = a_cube();
    let arrived = the_faces_in_the_order_their_triangles_arrive(&meshed(&model, cube));
    let stored: Vec<FaceKey> = model
        .solid(cube)
        .expect("this model made the key")
        .faces()
        .map(|face| face.key())
        .collect();

    assert_eq!(stored.len(), 6, "{what} has six sides");
    assert_eq!(
        arrived, stored,
        "{what}: the blocks of triangles must arrive in the solid's stored face order"
    );
    assert_eq!(
        which_way_each_of_them_faces(&model, &arrived),
        vec![
            Some([0.0, 0.0, -1.0]),
            Some([0.0, 0.0, 1.0]),
            Some([0.0, -1.0, 0.0]),
            Some([1.0, 0.0, 0.0]),
            Some([0.0, 1.0, 0.0]),
            Some([-1.0, 0.0, 0.0]),
        ],
        "{what}: the blocks arrive bottom, top, then the four walls the way the \
         square was written down"
    );
}

fn a_round_bar() -> (&'static str, Model, SolidKey) {
    let mut model = a_model();
    let bar = model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar three across and five tall");
    ("a bar", model, bar)
}

fn a_cube() -> (&'static str, Model, SolidKey) {
    let mut model = a_model();
    let cube = model
        .add_cube(SIDE, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    ("a cube", model, cube)
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn meshed(model: &Model, solid: SolidKey) -> Mesh {
    let mesh = model
        .solid(solid)
        .expect("this model made the key")
        .mesh(
            Deflection::of(0.1).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("meshing at a tenth of a millimetre");
    assert!(
        !mesh.triangles().is_empty(),
        "a mesh with no triangles cannot say anything about their order"
    );
    mesh
}

/// The faces the triangles name, in arrival order, with a face listed again only
/// when its triangles were actually broken apart by another face's.
///
/// So the length of this is the number of *blocks* and not the number of faces,
/// and that is what makes both tests able to see interleaving at all.
fn the_faces_in_the_order_their_triangles_arrive(mesh: &Mesh) -> Vec<FaceKey> {
    let mut blocks: Vec<FaceKey> = Vec::new();
    for triangle in mesh.triangles() {
        if blocks.last() != Some(&triangle.face()) {
            blocks.push(triangle.face());
        }
    }
    blocks
}

/// Each face's outward normal where it sits on a plane, and nothing where it
/// does not.
fn which_way_each_of_them_faces(model: &Model, faces: &[FaceKey]) -> Vec<Option<[f64; 3]>> {
    faces
        .iter()
        .map(|&face| {
            model
                .face(face)
                .expect("this model made the key")
                .surface()
                .as_plane()
                .map(|plane| plane.normal().components())
        })
        .collect()
}
