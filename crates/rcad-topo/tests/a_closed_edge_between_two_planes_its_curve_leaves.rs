//! A round bar whose flat lid is split by a circle into an annulus and a disc,
//! with the disc tilted off the lid through the circle's one corner.
//!
//! The circle is a closed edge between two flat faces, and its run names the
//! pipe of radius 1 to say where it goes. That pipe crosses the lid in a
//! circle and the tilted disc's plane in an ellipse, so the two faces put the
//! edge in two different places. The kernel drew whichever the solid listed
//! first: Q386's probe measured `113.003` with the annulus first and `111.644`
//! with the disc first, each `± 0.376`, for a bar that is `36π`. Q574 refuses
//! the edge instead, in `SolidBuilder::build` and in `Model::validate`.
//!
//! The same bar with the disc on the lid's own plane, or on it written a hair
//! apart, is one plane spelt twice and still builds.

use rcad_geom::{Axis, Cylinder, EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftFace, DraftStep, DraftWire, Error, Invalid, Model, Result, SolidBuilder,
    SolidKey,
};

const F: Orientation = Orientation::Forward;
const R: Orientation = Orientation::Reversed;

#[test]
fn a_disc_tilted_off_the_lid_is_refused_whichever_face_is_listed_first() {
    for disc_first in [false, true] {
        let (_, built) = a_bar_with_its_lid_split_by_a_disc(tilted_through_the_corner(), disc_first);

        assert!(
            matches!(
                built,
                Err(Error::Invalid(
                    Invalid::ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes
                ))
            ),
            "disc first {disc_first}: the builder gave {built:?}"
        );
    }
}

/// The same refusal from the sweep, for a model that never went through a
/// builder: the sound bar written out, its disc's plane swapped for the tilted
/// one in the bytes, and read back. `Deserialize` asks nothing about geometry,
/// so this is the route a model that holds the shape takes.
#[test]
fn the_sweep_refuses_the_tilted_disc_read_back_from_bytes() {
    for disc_first in [false, true] {
        let (model, built) = a_bar_with_its_lid_split_by_a_disc(the_lid(), disc_first);
        built.expect("the disc on the lid's own plane builds");
        let tilted = with_the_discs_plane_swapped(&model, disc_first, tilted_through_the_corner());

        let swept = tilted.validate(&Policy::certified(), &mut ProvenanceLog::new());

        assert!(
            matches!(
                swept,
                Err(Error::Invalid(
                    Invalid::ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes
                ))
            ),
            "disc first {disc_first}: the sweep gave {swept:?}"
        );
    }
}

#[test]
fn a_disc_on_the_lids_own_plane_still_builds_and_validates() {
    let one_ulp_higher = the_lid()
        .moved_to(f64::from_bits(the_lid().offset().to_bits() + 1))
        .expect("an offset");
    for disc in [the_lid(), one_ulp_higher] {
        for disc_first in [false, true] {
            let (model, built) = a_bar_with_its_lid_split_by_a_disc(disc, disc_first);

            built.unwrap_or_else(|why| {
                panic!("disc {disc:?}, disc first {disc_first}: refused with {why:?}")
            });
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("and the model it went into passes the sweep");
        }
    }
}

/// The plane the disc is tilted onto: through the circle's corner `(1, 0, 4)`,
/// leaning a tenth along `x`.
fn tilted_through_the_corner() -> Plane {
    Plane::from_point_and_normal(Point::new(1.0, 0.0, 4.0), Vector::new(-0.1, 0.0, 1.0))
        .expect("a plane")
}

fn the_lid() -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), up()).expect("a plane")
}

fn up() -> Vector {
    Vector::new(0.0, 0.0, 1.0)
}

/// A bar of radius 3 up `z` from 0 to 4, its lid split by a circle of radius
/// 1 into an annulus on the lid's plane and a disc on `disc`. The circle names
/// the pipe of radius 1 about the bar's axis. Every edge is walked the way
/// Q386's probe found the sound bar wants.
fn a_bar_with_its_lid_split_by_a_disc(disc: Plane, disc_first: bool) -> (Model, Result<SolidKey>) {
    let pipe = |radius| {
        Surface::Cylinder(
            Cylinder::new(Axis::new(Point::origin(), up()).expect("an axis"), radius)
                .expect("a pipe"),
        )
    };
    let base = Plane::from_point_and_normal(Point::origin(), -up()).expect("a plane");
    let mut builder = SolidBuilder::new();
    let at_the_bottom = builder.add_vertex(Point::new(3.0, 0.0, 0.0)).expect("a corner");
    let at_the_top = builder.add_vertex(Point::new(3.0, 0.0, 4.0)).expect("a corner");
    let on_the_circle = builder.add_vertex(Point::new(1.0, 0.0, 4.0)).expect("a corner");
    let bottom = builder
        .add_edge(at_the_bottom, at_the_bottom, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("an edge");
    let top = builder
        .add_edge(at_the_top, at_the_top, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("an edge");
    let circle = builder
        .add_edge(
            on_the_circle,
            on_the_circle,
            EdgeRun::RightRoundWhereItCrosses { and: pipe(1.0) },
        )
        .expect("an edge");
    let seam = builder.add_edge(at_the_bottom, at_the_top, EdgeRun::Straight).expect("an edge");

    let steps = [(bottom, F), (seam, F), (top, R), (seam, R)]
        .map(|(edge, way)| DraftStep::Along(builder.add_coedge(edge, way).expect("a coedge")));
    let round = builder.add_wire(&steps).expect("a wire");
    let wall = builder.add_face(pipe(3.0), &[round]).expect("a face");
    let annulus = |builder: &mut SolidBuilder| {
        let outer = one_step_round(builder, top, F);
        let hole = one_step_round(builder, circle, R);
        builder
            .add_face(Surface::Plane(the_lid()), &[outer, hole])
            .expect("a face")
    };
    let disc = |builder: &mut SolidBuilder| {
        let round = one_step_round(builder, circle, F);
        builder.add_face(Surface::Plane(disc), &[round]).expect("a face")
    };
    let (first, second) = if disc_first {
        let listed_first = disc(&mut builder);
        (listed_first, annulus(&mut builder))
    } else {
        let listed_first = annulus(&mut builder);
        (listed_first, disc(&mut builder))
    };
    let floor_round = one_step_round(&mut builder, bottom, R);
    let floor = builder
        .add_face(Surface::Plane(base), &[floor_round])
        .expect("a face");
    let faces: [DraftFace; 4] = [wall, first, second, floor];
    builder
        .add_shell(&faces.map(|face| (face, F)))
        .expect("a shell");
    let mut model = Model::new(Scale::millimetres(10.0).expect("a size"));
    let built = builder.build(&mut model, &Policy::certified(), &mut ProvenanceLog::new());
    (model, built)
}

fn one_step_round(builder: &mut SolidBuilder, edge: DraftEdge, way: Orientation) -> DraftWire {
    let coedge = builder.add_coedge(edge, way).expect("a coedge");
    builder.add_wire(&[DraftStep::Along(coedge)]).expect("a wire")
}

/// `model` written out with postcard, the disc's plane overwritten with
/// `tilted`, and read back.
///
/// The lid's plane is written twice, once for the annulus and once for the
/// disc, in the order the faces were described; which of the two is the disc
/// is `disc_first`.
fn with_the_discs_plane_swapped(model: &Model, disc_first: bool, tilted: Plane) -> Model {
    let mut bytes = postcard::to_allocvec(model).expect("a model writes out");
    let stored = the_bytes_of(the_lid());
    let places: Vec<usize> = bytes
        .windows(stored.len())
        .enumerate()
        .filter(|(_, window)| *window == stored.as_slice())
        .map(|(at, _)| at)
        .collect();
    let [annulus_or_disc, disc_or_annulus] = places[..] else {
        panic!("the lid's plane is written once per face on it, and it was found {places:?}");
    };
    let at = if disc_first { annulus_or_disc } else { disc_or_annulus };
    bytes[at..at + stored.len()].copy_from_slice(&the_bytes_of(tilted));
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
