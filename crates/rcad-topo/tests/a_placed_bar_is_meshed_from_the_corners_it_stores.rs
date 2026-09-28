//! A placed bar's mesh points are the unplaced bar's mesh points moved out,
//! bit for bit.
//!
//! A placed face is meshed in the frame it is stored in, and its points are
//! then moved out by the placement. So the mesh of a bar placed by some
//! movement should be the mesh of the same bar at no placement, with each point
//! moved by that movement: the same walk, on the same numbers, and one movement
//! out.
//!
//! **It was not, and the corners were why.** `BuildingAMesh::walk` handed the
//! walk each end of an edge as the located corner moved back into the stored
//! frame. That is the placement and then its inverse, and it rounds:
//! `(3, 0, 0)` turned an eighth about `z` and back is `(3, −4.4e−16, 0)`. A
//! walk that starts from a different corner puts its points somewhere else, and
//! for the eighth turn 75 of the 252 points differed (`docs/decisions.md`
//! §249.3, which found it). The walk now reads the stored corner out of the
//! arena, the way `the_surface_this_face_is_meshed_on` already read the surface.
//!
//! # What stops this passing for the wrong reason
//!
//! A placement whose round trip happens to give the corners back exactly
//! cannot tell the old walk from the new one. So each placement is also asked
//! whether moving the bar's two corners out and back changes them, and a real
//! share of them must.
//!
//! # What is not compared
//!
//! **The triangles.** `BuildingAMesh::cut_up` still reads each point of a wire
//! from the point table, where it sits moved out, and moves it back in. That
//! is the same double rounding one step later, and it is enough to change how
//! a cap is split: for the eighth turn, 174 of 500 triangles join points the
//! unplaced bar's triangles do not. Reading those points in the frame they
//! were walked in instead would make every triangle match as well. It would
//! also turn the unit ball at `x = 1.0808639105689192e16` of
//! `how_far_out_a_ball_can_sit_and_still_mesh.rs` from a refusal into a flat
//! mesh that encloses nothing, which `docs/decisions.md` §333.6 measured to be
//! a wrong answer, so that is left for a change that keeps the refusal.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Mesh, Model};

const RADIUS: f64 = 3.0;
const HEIGHT: f64 = 5.0;

#[test]
fn a_placed_bar_meshes_to_the_unplaced_bars_mesh_moved_out_point_for_point() {
    let unplaced = a_bar_meshed(&Isometry::identity());
    let placements = some_turns_with_and_without_a_shift();
    let mut the_round_trip_moves_a_corner = 0;
    let mut differing = Vec::new();

    for (name, placement) in &placements {
        let placed = a_bar_meshed(placement);
        assert_eq!(
            placed.points().len(),
            unplaced.points().len(),
            "{name}: the placed bar has a different number of mesh points, so there is \
             nothing to compare point by point"
        );
        let moved_out = unplaced
            .points()
            .iter()
            .map(|&point| placement.apply_to_point(point));
        let off = placed
            .points()
            .iter()
            .zip(moved_out)
            .filter(|(placed, moved)| bits(**placed) != bits(*moved))
            .count();
        if off > 0 {
            differing.push((name.as_str(), off));
        }
        if the_round_trip_moves_a_corner_of(placement) {
            the_round_trip_moves_a_corner += 1;
        }
    }

    println!(
        "{} placements, {the_round_trip_moves_a_corner} of them ones where moving a corner out \
         and back changes it",
        placements.len()
    );
    assert!(
        differing.is_empty(),
        "the placed bar's mesh is not the unplaced bar's mesh moved out; (placement, points \
         differing): {differing:?}"
    );
    assert!(
        the_round_trip_moves_a_corner * 2 >= placements.len(),
        "only {the_round_trip_moves_a_corner} of {} placements change a corner when it is \
         moved out and back, so this barely tells a walk from the stored corners from a walk \
         from the moved ones",
        placements.len()
    );
}

fn a_bar_meshed(placement: &Isometry) -> Mesh {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar of radius three and height five");
    model
        .place_solid(bar, placement)
        .expect("this model made the key");
    model
        .solid(bar)
        .expect("this model made the key")
        .mesh(
            Deflection::of(0.005).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar meshes at five thousandths wherever it is put")
}

/// §249.3's eighth about `z`, and turns about two leaning axes, each alone and
/// then shifted.
fn some_turns_with_and_without_a_shift() -> Vec<(String, Isometry)> {
    let eighth = Isometry::rotation(&Axis::z(), core::f64::consts::FRAC_PI_4).expect("finite");
    let mut placements = vec![("an eighth about z".to_owned(), eighth)];
    for shift in [[3.0, -5.0, 7.0], [0.25, -0.5, 0.125], [0.1, 0.2, 0.3]] {
        placements.push((
            format!("an eighth about z, then {shift:?}"),
            shifted_by(Vector::new(shift[0], shift[1], shift[2])).compose(&eighth),
        ));
    }
    for about in [[1.0, 2.0, 3.0], [-3.0, 1.0, 0.5]] {
        let axis = Axis::new(
            Point::origin(),
            Vector::new(about[0], about[1], about[2])
                .normalized()
                .expect("a direction with length"),
        )
        .expect("a unit direction is an axis");
        for turn in [0.7, 2.5] {
            let turned = Isometry::rotation(&axis, turn).expect("a finite angle");
            placements.push((format!("{turn} about {about:?}"), turned));
            placements.push((
                format!("{turn} about {about:?}, then [3, 5, 7]"),
                shifted_by(Vector::new(3.0, 5.0, 7.0)).compose(&turned),
            ));
        }
    }
    placements
}

fn shifted_by(shift: Vector) -> Isometry {
    Isometry::translation(shift).expect("a finite shift")
}

/// Whether moving the bar's two stored corners out by `placement` and back by
/// its inverse — the route the walk used to take — gives either of them back
/// changed.
fn the_round_trip_moves_a_corner_of(placement: &Isometry) -> bool {
    let back = placement.inverse();
    [
        Point::new(RADIUS, 0.0, 0.0),
        Point::new(RADIUS, 0.0, HEIGHT),
    ]
    .into_iter()
    .any(|corner| bits(back.apply_to_point(placement.apply_to_point(corner))) != bits(corner))
}

fn bits(point: Point) -> [u64; 3] {
    point.coordinates().map(f64::to_bits)
}
