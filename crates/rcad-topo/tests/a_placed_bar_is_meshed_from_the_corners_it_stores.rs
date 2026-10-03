//! A placed bar's mesh points are the unplaced bar's mesh points moved out,
//! bit for bit, and its wall is cut into the unplaced wall's triangles.
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
//! # The wall's triangles
//!
//! `BuildingAMesh::walk_a_wire_round` reads each point of a placed curved
//! face where it was made, not moved out and back, so the wall is handed to
//! the triangulator as the same numbers as the unplaced wall and cut into the
//! same triangles. Moved out and back, the same double rounding one step
//! later, 12 to 120 of the wall's triangles join other points under these
//! twelve placements (`docs/decisions.md` §379.5). Reading points where they
//! were made can hand back a mesh that is sound there and flat where it
//! lands, and the unit ball at `x = 1.0808639105689192e16` of
//! `how_far_out_a_ball_can_sit_and_still_mesh.rs` is refused for it, as
//! `PlacingItTookAwayTheSpaceItsMeshEncloses`.
//!
//! # What is not compared
//!
//! **The caps' triangles.** A flat face is cut into triangles on its plane
//! where the solid sits, so that a placed cap is cut like the same cap read
//! back from its STEP file, which has no placement (§362). The points round a
//! circle are a tie for the triangulator's Delaunay test, and a tie goes
//! whichever way the last bits push it, so the placed caps are not cut like
//! the unplaced ones: 141 to 199 of the bar's 500 triangles join other points,
//! every one of them on a cap. Cut where they are stored, the caps match the
//! unplaced bar's in every row and the read-back's are lost (§379.5).

use std::collections::HashSet;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{FaceKey, Mesh, Model};

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

/// The wall's triangles, corner for corner and in the same order. The caps'
/// are printed and not compared, for the reason the module doc gives.
#[test]
fn a_placed_bars_wall_is_cut_into_the_unplaced_walls_triangles() {
    let (unplaced, unplaced_caps) = a_bar_meshed_with_its_caps(&Isometry::identity());
    let unplaced_wall = the_wall_of(&unplaced, &unplaced_caps);
    let unplaced_as_sets: HashSet<[u32; 3]> = unplaced
        .triangles()
        .iter()
        .map(|triangle| as_a_set(triangle.corners()))
        .collect();
    let joins_other_points = |corners: [u32; 3]| !unplaced_as_sets.contains(&as_a_set(corners));
    assert!(
        !unplaced_wall.is_empty() && unplaced_wall.len() < unplaced.triangles().len(),
        "the bar's {} triangles split into {} on the wall and the rest on the caps, so there is \
         no wall to compare",
        unplaced.triangles().len(),
        unplaced_wall.len()
    );
    let mut differing = Vec::new();

    for (name, placement) in &some_turns_with_and_without_a_shift() {
        let (placed, caps) = a_bar_meshed_with_its_caps(placement);
        let wall = the_wall_of(&placed, &caps);
        let off_on_the_wall = wall
            .iter()
            .filter(|&&corners| joins_other_points(corners))
            .count();
        let off_on_the_caps = placed
            .triangles()
            .iter()
            .filter(|triangle| caps.contains(&triangle.face()))
            .filter(|triangle| joins_other_points(triangle.corners()))
            .count();
        println!(
            "{name}: of {} triangles, {off_on_the_caps} on the caps and {off_on_the_wall} on \
             the wall join other points",
            placed.triangles().len()
        );
        if wall != unplaced_wall {
            differing.push((name.clone(), off_on_the_wall));
        }
    }

    assert!(
        differing.is_empty(),
        "a placed bar's wall is not cut into the unplaced wall's triangles; (placement, wall \
         triangles joining other points): {differing:?}"
    );
}

/// Each triangle not on a cap, as its three corners in the order the mesh
/// holds them.
fn the_wall_of(mesh: &Mesh, caps: &HashSet<FaceKey>) -> Vec<[u32; 3]> {
    mesh.triangles()
        .iter()
        .filter(|triangle| !caps.contains(&triangle.face()))
        .map(|triangle| triangle.corners())
        .collect()
}

fn as_a_set(mut corners: [u32; 3]) -> [u32; 3] {
    corners.sort_unstable();
    corners
}

fn a_bar_meshed(placement: &Isometry) -> Mesh {
    let (mesh, _) = a_bar_meshed_with_its_caps(placement);
    mesh
}

/// The mesh of the bar placed by `placement`, and the keys of its two flat
/// faces.
///
/// **The keys come out of the same model as the mesh.** A key names the model
/// it belongs to, and every `Model::new` is a model of its own, so a cap's key
/// read off a second bar built beside this one never matches a triangle of this
/// mesh: every triangle then counts as the wall's.
fn a_bar_meshed_with_its_caps(placement: &Isometry) -> (Mesh, HashSet<FaceKey>) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar of radius three and height five");
    model
        .place_solid(bar, placement)
        .expect("this model made the key");
    let solid = model.solid(bar).expect("this model made the key");
    let caps = solid
        .faces()
        .filter(|face| face.surface().as_plane().is_some())
        .map(|face| face.key())
        .collect();
    let mesh = solid
        .mesh(
            Deflection::of(0.005).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar meshes at five thousandths wherever it is put");
    (mesh, caps)
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
