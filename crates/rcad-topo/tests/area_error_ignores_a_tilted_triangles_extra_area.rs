//! A flat face whose mesh points stand off its own plane, so that its triangles
//! lean, and lean triangles have more area than their shadow on the plane.
//!
//! **The shape is the tilted half disc of
//! `an_edge_between_two_planes_its_curve_leaves.rs`, leaning further.** A round
//! bar of radius 3 up `z` from 0 to 4, its lid split by the half of the circle
//! of radius 1 at `x ≥ 0` and the chord `x = 0`, the half disc then put on a
//! plane through the chord leaning `0.3` along `x`. The arc names the pipe of
//! radius 1, which crosses the half disc's plane in half an ellipse, and with
//! the half disc listed first the mesher draws the arc there, up to `0.3` above
//! the lid. The rest of the lid shares those points, so its triangles lean up
//! to them.
//!
//! The arc's points are on the pipe, so their shadow on the lid is the lid's
//! own half circle: the lid's triangles cast the shadow its own outline would,
//! and every face's true area is known. The lid is `9π − π/2`, the half disc
//! `π/2 · √(1 + 0.3²)`, the wall `24π` and the floor `9π`.
//!
//! **A model only bytes can carry.** `Model::validate` refuses it
//! `AnArcBetweenTwoFlatFacesOnDifferentPlanes`, and the builder will not build
//! it; it is the sound bar written out with postcard, the half disc's plane
//! swapped in the bytes, and read back. `SolidView::measure` asks nothing of
//! where a face is, and answers it.

use std::f64::consts::PI;

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Scale,
    Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftEdge, DraftStep, DraftWire, Model, Result, SolidBuilder, SolidKey};

const F: Orientation = Orientation::Forward;
const R: Orientation = Orientation::Reversed;

const LEAN: f64 = 0.3;

/// Measured on the base: `26.84129` against `9π − π/2 = 26.70354`, `0.138`
/// over, with an area error of `0.0120` — the deviation times the lid's rim and
/// nothing for the lean.
#[test]
fn a_lid_whose_triangles_lean_up_to_an_arc_off_its_plane_measures_within_its_area_error() {
    let model = the_bar_with_its_half_disc_leaning();
    let solid = model.solids().next().expect("the bar");
    let lid = solid
        .faces()
        .find(|face| face.surface().as_plane() == Some(&the_lid()))
        .expect("the rest of the lid");

    let measured = solid
        .measure_one_face(
            lid.key(),
            deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the lid measures");

    let truth = 9.0 * PI - PI / 2.0;
    let miss = (measured.area() - truth).abs();
    assert!(
        miss <= measured.area_error(),
        "the lid measured {} ± {}, {miss} from its true {truth}",
        measured.area(),
        measured.area_error()
    );
}

/// Measured on the base: `132.14949` against `132.01605`, `0.133` over, with an
/// area error of `0.0492`.
#[test]
fn a_bar_whose_lid_leans_up_to_an_arc_off_its_plane_measures_within_its_area_error() {
    let model = the_bar_with_its_half_disc_leaning();
    let solid = model.solids().next().expect("the bar");

    let measured = solid
        .measure(
            deflection(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the bar measures");

    let half_disc = PI / 2.0 * (1.0 + LEAN * LEAN).sqrt();
    let truth = 24.0 * PI + 9.0 * PI + (9.0 * PI - PI / 2.0) + half_disc;
    let miss = (measured.surface_area() - truth).abs();
    assert!(
        miss <= measured.area_error(),
        "the bar measured {} ± {}, {miss} from its true {truth}",
        measured.surface_area(),
        measured.area_error()
    );
}

fn deflection() -> Deflection {
    Deflection::of(0.001).expect("a length")
}

/// The bar with the half disc listed first, on the lid's own plane, which
/// builds; written out, the half disc's plane swapped for one through the
/// chord leaning [`LEAN`] along `x`, and read back.
fn the_bar_with_its_half_disc_leaning() -> Model {
    let (model, built) = a_bar_with_its_lid_split_by_a_half_disc();
    built.expect("the half disc on the lid's own plane builds");
    let leaning =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(-LEAN, 0.0, 1.0))
            .expect("a plane");
    let mut bytes = postcard::to_allocvec(&model).expect("a model writes out");
    let stored = the_bytes_of(the_lid());
    let places: Vec<usize> = bytes
        .windows(stored.len())
        .enumerate()
        .filter(|(_, window)| *window == stored.as_slice())
        .map(|(at, _)| at)
        .collect();
    let [half_disc, _] = places[..] else {
        panic!("the lid's plane is written once per face on it, and it was found {places:?}");
    };
    bytes[half_disc..half_disc + stored.len()].copy_from_slice(&the_bytes_of(leaning));
    postcard::from_bytes(&bytes).expect("reading back does not ask where a face is")
}

/// A plane the way postcard writes one: its four coefficients, little-endian.
fn the_bytes_of(plane: Plane) -> Vec<u8> {
    plane
        .coefficients()
        .into_iter()
        .flat_map(f64::to_le_bytes)
        .collect()
}

fn the_lid() -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), up()).expect("a plane")
}

fn up() -> Vector {
    Vector::new(0.0, 0.0, 1.0)
}

/// A pipe of `radius` up the bar's axis.
fn pipe(radius: f64) -> Surface {
    Surface::Cylinder(
        Cylinder::new(Axis::new(Point::origin(), up()).expect("an axis"), radius).expect("a pipe"),
    )
}

/// The bar of radius 3 up `z` from 0 to 4, its lid split by the half of the
/// circle of radius 1 at `x ≥ 0` and the chord `x = 0`, the half disc on the
/// lid's own plane and listed before the rest of the lid. Every edge is walked
/// the way `an_edge_between_two_planes_its_curve_leaves.rs` walks it.
fn a_bar_with_its_lid_split_by_a_half_disc() -> (Model, Result<SolidKey>) {
    let base = Plane::from_point_and_normal(Point::origin(), -up()).expect("a plane");
    let mut builder = SolidBuilder::new();
    let at_the_bottom = builder
        .add_vertex(Point::new(3.0, 0.0, 0.0))
        .expect("a corner");
    let at_the_top = builder
        .add_vertex(Point::new(3.0, 0.0, 4.0))
        .expect("a corner");
    let bottom = builder
        .add_edge(
            at_the_bottom,
            at_the_bottom,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("an edge");
    let top = builder
        .add_edge(
            at_the_top,
            at_the_top,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("an edge");
    let seam = builder
        .add_edge(at_the_bottom, at_the_top, EdgeRun::Straight)
        .expect("an edge");
    let steps = [(bottom, F), (seam, F), (top, R), (seam, R)]
        .map(|(edge, way)| DraftStep::Along(builder.add_coedge(edge, way).expect("a coedge")));
    let round = builder.add_wire(&steps).expect("a wire");
    let wall = builder.add_face(pipe(3.0), &[round]).expect("a face");

    let east = HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
        .expect("a half-space");
    let run = EdgeRun::PartOfWhereItCrosses {
        and: pipe(1.0),
        picked_by: smallvec::smallvec![Picker::from(east)],
    };
    let south = builder
        .add_vertex(Point::new(0.0, -1.0, 4.0))
        .expect("a corner");
    let north = builder
        .add_vertex(Point::new(0.0, 1.0, 4.0))
        .expect("a corner");
    let arc = builder.add_edge(south, north, run).expect("an edge");
    let chord = builder
        .add_edge(north, south, EdgeRun::Straight)
        .expect("an edge");
    let round_the_half = |builder: &mut SolidBuilder, way| {
        let steps = [arc, chord]
            .map(|edge| DraftStep::Along(builder.add_coedge(edge, way).expect("a coedge")));
        builder.add_wire(&steps).expect("a wire")
    };
    let hole = round_the_half(&mut builder, R);
    let round_the_half_disc = round_the_half(&mut builder, F);

    let half_disc = builder
        .add_face(Surface::Plane(the_lid()), &[round_the_half_disc])
        .expect("a face");
    let outer = one_step_round(&mut builder, top, F);
    let rest_of_the_lid = builder
        .add_face(Surface::Plane(the_lid()), &[outer, hole])
        .expect("a face");
    let floor_round = one_step_round(&mut builder, bottom, R);
    let floor = builder
        .add_face(Surface::Plane(base), &[floor_round])
        .expect("a face");
    builder
        .add_shell(&[wall, half_disc, rest_of_the_lid, floor].map(|face| (face, F)))
        .expect("a shell");
    let mut model = Model::new(Scale::millimetres(10.0).expect("a size"));
    let built = builder.build(&mut model, &Policy::certified(), &mut ProvenanceLog::new());
    (model, built)
}

fn one_step_round(builder: &mut SolidBuilder, edge: DraftEdge, way: Orientation) -> DraftWire {
    let coedge = builder.add_coedge(edge, way).expect("a coedge");
    builder
        .add_wire(&[DraftStep::Along(coedge)])
        .expect("a wire")
}
