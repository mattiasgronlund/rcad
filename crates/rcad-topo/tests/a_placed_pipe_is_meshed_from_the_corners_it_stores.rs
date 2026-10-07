//! Whether a placed pipe's flat end — a ring between two circles, not a bar's
//! whole disc — is cut into the unplaced pipe's triangles, under the same
//! twelve placements `a_placed_bar_is_meshed_from_the_corners_it_stores.rs`
//! uses.
//!
//! Q797 measured it first, when §379.5's rule on corner order was asked only
//! for a flat patch with one loop and a ring was left to spade's near-ties:
//! nine of the twelve placements moved 2 of a ring's triangles onto other
//! points, on the bottom ring, the top ring, or (once) both. Since Q1109 a
//! ring whose two rims the walls across read at one height is cut by the
//! order of its corners too, and every placement is cut into the unplaced
//! pipe's triangles, corner for corner and in order. `docs/decisions.d/379-*.md`.
//!
//! The pipe is a bar of radius 3 and height 5 less a smaller coaxial bar that
//! pokes through both ends, so each flat end is a ring. It is built through
//! `Model::fuse`, since no builder makes a pipe without one.

use std::collections::HashSet;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{FaceKey, Mesh, Model, Rule, SolidKey};

const OUTER_RADIUS: f64 = 3.0;
const HEIGHT: f64 = 5.0;
const INNER_RADIUS: f64 = 1.0;
const INNER_HEIGHT: f64 = HEIGHT + 2.0;

/// Each flat end of the pipe, under each of the twelve placements, is cut into
/// the unplaced pipe's triangles: the same three corners in the same order, in
/// the same order down the mesh. Prints how many of each ring's triangles join
/// other points, which was 2 on one ring or both in nine placements before
/// the rings were cut by corner order (§379.5, Q797).
#[test]
fn a_placed_pipes_flat_ends_are_cut_into_the_unplaced_pipes_triangles() {
    let (unplaced, unplaced_bottom, unplaced_top) =
        a_pipe_meshed_with_its_ends(&Isometry::identity());
    let unplaced_bottom_cut = the_triangles_of(&unplaced, unplaced_bottom);
    let unplaced_top_cut = the_triangles_of(&unplaced, unplaced_top);
    assert!(
        !unplaced_bottom_cut.is_empty() && !unplaced_top_cut.is_empty(),
        "the unplaced pipe's flat ends have no triangles to compare"
    );
    let unplaced_as_sets: HashSet<[u32; 3]> = unplaced_bottom_cut
        .iter()
        .chain(&unplaced_top_cut)
        .map(|&corners| as_a_set(corners))
        .collect();
    let off = |cut: &[[u32; 3]]| {
        cut.iter()
            .filter(|&&corners| !unplaced_as_sets.contains(&as_a_set(corners)))
            .count()
    };
    let mut differing = Vec::new();

    for (name, placement) in &some_turns_with_and_without_a_shift() {
        let (placed, bottom, top) = a_pipe_meshed_with_its_ends(placement);
        let bottom_cut = the_triangles_of(&placed, bottom);
        let top_cut = the_triangles_of(&placed, top);
        let (bottom_off, top_off) = (off(&bottom_cut), off(&top_cut));
        println!(
            "{name}: of {} triangles, {bottom_off} on the bottom ring and {top_off} on the top \
             ring join other points",
            placed.triangles().len()
        );
        if bottom_cut != unplaced_bottom_cut || top_cut != unplaced_top_cut {
            differing.push((name.clone(), bottom_off, top_off));
        }
    }

    assert!(
        differing.is_empty(),
        "a placed pipe's flat ends are not cut into the unplaced pipe's triangles; (placement, \
         bottom ring and top ring triangles joining other points): {differing:?}"
    );
}

/// The triangles of `face`, each as its three corners in the order the mesh
/// holds them.
fn the_triangles_of(mesh: &Mesh, face: FaceKey) -> Vec<[u32; 3]> {
    mesh.triangles()
        .iter()
        .filter(|triangle| triangle.face() == face)
        .map(|triangle| triangle.corners())
        .collect()
}

fn as_a_set(mut corners: [u32; 3]) -> [u32; 3] {
    corners.sort_unstable();
    corners
}

/// The mesh of the pipe placed by `placement`, and the keys of its bottom
/// and top rings.
///
/// **The two ring keys are told apart before `placement` is applied.** The
/// outer and inner bars that make the pipe are both built at no placement, so
/// the ring nearer `z = 0` in the stored frame is the bottom one and the
/// other is the top one, whichever `placement` later moves them to.
fn a_pipe_meshed_with_its_ends(placement: &Isometry) -> (Mesh, FaceKey, FaceKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let outer = model
        .add_round_bar(OUTER_RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar of radius three");
    let through = model
        .add_round_bar(
            INNER_RADIUS,
            INNER_HEIGHT,
            &shifted_by(Vector::new(0.0, 0.0, -1.0)),
        )
        .expect("a bar of radius one, poking through both ends");
    let pipe = the_one_solid(&mut model, outer, through, Rule::Difference);
    let (bottom, top) = the_two_rings_of(&model, pipe);
    model
        .place_solid(pipe, placement)
        .expect("this model made the key");
    let solid = model.solid(pipe).expect("this model made the key");
    let mesh = solid
        .mesh(
            Deflection::of(0.005).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a pipe meshes at five thousandths wherever it is put");
    (mesh, bottom, top)
}

/// The pipe's two flat faces, told apart by which sits nearer `z = 0` before
/// any placement.
fn the_two_rings_of(model: &Model, pipe: SolidKey) -> (FaceKey, FaceKey) {
    let solid = model.solid(pipe).expect("this model made the key");
    let mut rings: Vec<(FaceKey, f64)> = solid
        .faces()
        .filter_map(|face| {
            let surface = face.surface();
            surface
                .as_plane()
                .map(|plane| (face.key(), plane.origin().coordinates()[2]))
        })
        .collect();
    assert_eq!(
        rings.len(),
        2,
        "the pipe has other than two flat faces: {rings:?}"
    );
    rings.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("finite heights"));
    (rings[0].0, rings[1].0)
}

fn the_one_solid(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let answer = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{rule:?} refused: {why}"));
    let [solid] = answer.solids() else {
        panic!("{rule:?} is one solid, not {:?}", answer.solids());
    };
    *solid
}

/// §249.3's eighth about `z`, and turns about two leaning axes, each alone
/// and then shifted — the same twelve placements
/// `a_placed_bar_is_meshed_from_the_corners_it_stores.rs` uses.
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
