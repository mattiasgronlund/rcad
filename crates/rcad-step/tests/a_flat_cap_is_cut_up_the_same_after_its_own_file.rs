//! A flat face of a placed solid is cut into the same triangles as the same
//! face read back out of the solid's own STEP file (§362).
//!
//! A round bar of radius 3 and height 5, moved by `(0.1, 0.2, 0.3)` and nothing
//! else, comes back from its file with every mesh point the same bits (§249.4
//! item 2). Its two caps used to come back cut differently all the same: 20 of
//! each cap's triangles joined other points. The points round a cap circle are
//! points of one circle, so four in a row are a true tie for the Delaunay
//! triangulator, and the tie was decided on different numbers on the two sides.
//! The bar going in was cut up in the frame it is stored in, from its mesh
//! points moved back there by a rounded subtraction; the bar coming back has no
//! placement and was cut up where it sits, from the mesh points themselves.

use std::collections::BTreeMap;
use std::f64::consts::FRAC_PI_4;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Measurement, Mesh, Model, SolidView};

#[test]
fn a_bar_moved_a_tenth_has_its_caps_cut_up_the_same_after_its_own_file() {
    let here = a_bar_moved_by(Vector::new(0.1, 0.2, 0.3));
    let there = read_back(&here);
    let deflection = Deflection::for_scale(there.scale());
    let solid = the_one_solid(&here);
    let solid_read_back = the_one_solid(&there);
    let mesh = meshed(solid, deflection);
    let mesh_read_back = meshed(solid_read_back, deflection);

    assert_eq!(
        the_bits_of_every_point(&mesh),
        the_bits_of_every_point(&mesh_read_back),
        "the mesh points no longer come back the same bits, so the triangles below \
         cannot be compared by which points they join"
    );
    let caps = each_flat_face_cut_up(solid, &mesh);
    assert_eq!(caps.len(), 2, "a round bar has two flat caps");
    let caps_read_back = each_flat_face_cut_up(solid_read_back, &mesh_read_back);
    for (plane, triangles) in &caps {
        let read_back = caps_read_back.get(plane).unwrap_or_else(|| {
            panic!(
                "no cap on the plane {:?} came back",
                plane.map(f64::from_bits)
            )
        });
        let differing = triangles
            .iter()
            .zip(read_back)
            .filter(|(going_in, coming_back)| going_in != coming_back)
            .count();
        assert!(
            triangles == read_back,
            "the cap on the plane {:?} is cut into {} triangles going in and {} coming back, \
             and {differing} of them join other points",
            plane.map(f64::from_bits),
            triangles.len(),
            read_back.len(),
        );
    }
}

/// The volume was one ulp off while the caps were cut differently (§249.4
/// item 2). The wall is still cut up in a different frame on each side — the
/// bar's stored frame going in, and a shift that loses nothing coming back —
/// but for this bar its points land on the same bits in both (§362.2), so the
/// caps were all there was to tell the two apart.
///
/// The volume, the area and the centroid, and not the error bounds printed
/// beside them: those are charged for how far each side's frame can round, and
/// a placed wall is charged differently from a shifted one.
#[test]
fn a_bar_moved_a_tenth_measures_the_same_bits_after_its_own_file() {
    let here = a_bar_moved_by(Vector::new(0.1, 0.2, 0.3));
    let there = read_back(&here);
    let deflection = Deflection::for_scale(there.scale());
    let [going_in, coming_back] = [&here, &there].map(|model| {
        let measured = measured(the_one_solid(model), deflection);
        format!(
            "volume {:?}, area {:?}, centroid {:?}",
            measured.volume(),
            measured.surface_area(),
            measured.centroid()
        )
    });
    assert_eq!(
        going_in, coming_back,
        "the bar measures differently after its own file"
    );
}

/// A cube's face is a square, and a square's four corners are a tie for the
/// Delaunay test, so which diagonal it is cut along is the last bits' to say.
/// Turned and moved, a cube comes back from its file with every corner the same
/// bits, and 17 of these 21 turns used to cut from one to four of its faces
/// along the other diagonal (§362.4).
#[test]
fn a_turned_cube_has_every_face_cut_up_the_same_after_its_own_file() {
    let mut differing = Vec::new();
    for axis in [
        Vector::new(0.0, 0.0, 1.0),
        Vector::new(1.0, 2.0, 3.0),
        Vector::new(-2.0, 0.5, 1.0),
    ] {
        for eighths in 1..8 {
            let turn = Isometry::rotation(
                &Axis::new(Point::origin(), axis).expect("a direction that is not zero"),
                f64::from(eighths) * FRAC_PI_4,
            )
            .expect("a finite angle about a unit axis");
            let here = a_cube_turned_and_moved(&turn);
            let there = read_back(&here);
            let deflection = Deflection::for_scale(there.scale());
            let [going_in, coming_back] = [&here, &there].map(|model| {
                let solid = the_one_solid(model);
                let mesh = meshed(solid, deflection);
                (the_bits_of_every_point(&mesh), each_flat_face_cut_up(solid, &mesh))
            });
            assert_eq!(
                going_in.0, coming_back.0,
                "{eighths} eighths about {axis:?}: the cube's corners no longer come back the \
                 same bits, so its triangles cannot be compared by which corners they join"
            );
            let faces = going_in
                .1
                .iter()
                .filter(|(plane, triangles)| coming_back.1.get(*plane) != Some(triangles))
                .count();
            if faces > 0 {
                differing.push(format!("{eighths} eighths about {axis:?}: {faces} faces"));
            }
        }
    }
    assert!(
        differing.is_empty(),
        "turned cubes cut up differently after their own file: {differing:?}"
    );
}

/// A cube of side 2 at the origin, turned and then moved by `(3, −5, 7)`.
fn a_cube_turned_and_moved(turn: &Isometry) -> Model {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side 2 fits in a model this size");
    let shift = Isometry::translation(Vector::new(3.0, -5.0, 7.0)).expect("a finite shift");
    model
        .place_solid(solid, turn)
        .expect("the key came from this model");
    model
        .place_solid(solid, &shift)
        .expect("the key came from this model");
    model
}

fn the_bits_of_every_point(mesh: &Mesh) -> Vec<[u64; 3]> {
    mesh.points()
        .iter()
        .map(|point| point.coordinates().map(f64::to_bits))
        .collect()
}

/// A round bar of radius 3 and height 5, built at the origin and then moved,
/// the order the writer's fuzz target builds one in.
fn a_bar_moved_by(shift: Vector) -> Model {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = model
        .add_round_bar(3.0, 5.0, &Isometry::identity())
        .expect("a round bar of radius 3 fits in a model this size");
    let movement = Isometry::translation(shift).expect("a finite shift");
    model
        .place_solid(solid, &movement)
        .expect("the key came from this model");
    model
}

fn read_back(model: &Model) -> Model {
    let text = rcad_step::write(model, "a_flat_cap_is_cut_up_the_same_after_its_own_file")
        .expect("a sound bar is written");
    let file = StepFile::read(text.as_bytes()).expect("a file this writer wrote parses");
    Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a file this writer wrote is read")
        .into_model()
}

fn the_one_solid(model: &Model) -> SolidView<'_> {
    let mut solids = model.solids();
    let solid = solids.next().expect("the model holds a solid");
    assert!(solids.next().is_none(), "the model holds one solid");
    solid
}

fn meshed(solid: SolidView<'_>, deflection: Deflection) -> Mesh {
    solid
        .mesh(deflection, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound bar is meshed")
}

fn measured(solid: SolidView<'_>, deflection: Deflection) -> Measurement {
    solid
        .measure(deflection, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound bar measures")
}

/// Every flat face's triangles, as the mesh points they join and in the order
/// the mesh holds them, under the bits of the plane the face sits on.
fn each_flat_face_cut_up(solid: SolidView<'_>, mesh: &Mesh) -> BTreeMap<[u64; 4], Vec<[u32; 3]>> {
    solid
        .faces()
        .filter_map(|face| {
            let plane = face.surface().as_plane()?.coefficients().map(f64::to_bits);
            let triangles = mesh
                .triangles()
                .iter()
                .filter(|triangle| triangle.face() == face.key())
                .map(|triangle| triangle.corners())
                .collect();
            Some((plane, triangles))
        })
        .collect()
}
