//! Whether a placed pipe's flat end — a ring between two circles, not a bar's
//! whole disc — is cut into other triangles than the unplaced pipe's, under
//! the same twelve placements `a_placed_bar_is_meshed_from_the_corners_it_stores.rs`
//! uses.
//!
//! §379.5's rule on corner order is asked only for a flat patch with one
//! loop. A ring has two loops — an outline and a hole — so the triangulator
//! leaves it to spade whatever is asked, the same as a ball's lid. Whether
//! spade's near-ties on a ring go the same way wherever the pipe is put was
//! not measured (Q784 lane report, remaining); this is that measurement.
//!
//! **It does not.** Nine of the twelve placements move 2 of a ring's
//! triangles onto other points, on the bottom ring, the top ring, or (once)
//! both. The counts are pinned below and recorded, dated, in §379.5; a row is
//! filed for the defect. `docs/decisions.d/379-*.md`.
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

/// The count, pinned per placement, of each flat end's triangles that join
/// points the unplaced pipe's mesh does not join them with: (bottom ring,
/// top ring), in `some_turns_with_and_without_a_shift`'s order.
///
/// Measured 2026-10-06 on `9e6da7e1` and recorded in §379.5. Every entry but
/// two is `(2, 0)`, `(0, 2)` or `(0, 0)`; one is `(2, 2)`. A change in either
/// direction, on either ring, is a change worth re-reading §379.5 for.
const PINNED_OFF: [(usize, usize); 12] = [
    (2, 0),
    (0, 2),
    (2, 0),
    (2, 0),
    (2, 2),
    (2, 0),
    (0, 0),
    (0, 0),
    (2, 0),
    (2, 0),
    (0, 2),
    (0, 0),
];

/// Pins how many of each flat end's triangles, under each of the twelve
/// placements, join points the unplaced pipe's mesh does not. §379.5: a ring
/// has two loops, so the corner-order rule that keeps a bar's whole discs at
/// `0` leaves a ring to spade's near-ties, and this is not `0` everywhere.
#[test]
fn how_many_of_a_placed_pipes_flat_end_triangles_join_other_points() {
    let (unplaced, unplaced_bottom, unplaced_top) =
        a_pipe_meshed_with_its_ends(&Isometry::identity());
    let unplaced_bottom_triangles = corner_sets_of(&unplaced, unplaced_bottom);
    let unplaced_top_triangles = corner_sets_of(&unplaced, unplaced_top);
    assert!(
        !unplaced_bottom_triangles.is_empty() && !unplaced_top_triangles.is_empty(),
        "the unplaced pipe's flat ends have no triangles to compare"
    );
    let placements = some_turns_with_and_without_a_shift();
    assert_eq!(
        placements.len(),
        PINNED_OFF.len(),
        "the pinned table has a different length than the placements it is meant to cover"
    );
    let mut measured = Vec::new();
    let mut moved = Vec::new();

    for ((name, placement), &pinned) in placements.iter().zip(PINNED_OFF.iter()) {
        let (placed, bottom, top) = a_pipe_meshed_with_its_ends(placement);
        let bottom_off = count_joining_other_points(&placed, bottom, &unplaced_bottom_triangles);
        let top_off = count_joining_other_points(&placed, top, &unplaced_top_triangles);
        println!(
            "{name}: of {} triangles, {bottom_off} on the bottom ring and {top_off} on the top \
             ring join other points",
            placed.triangles().len()
        );
        measured.push((bottom_off, top_off));
        if (bottom_off, top_off) != pinned {
            moved.push((name.clone(), pinned, (bottom_off, top_off)));
        }
    }

    assert!(
        moved.is_empty(),
        "a placed pipe's flat end counts moved from what §379.5 pinned; (placement, pinned \
         (bottom, top), measured (bottom, top)): {moved:?}; full table: {measured:?}"
    );
}

/// The triangles of `face`, each as its three corner indices sorted.
fn corner_sets_of(mesh: &Mesh, face: FaceKey) -> HashSet<[u32; 3]> {
    mesh.triangles()
        .iter()
        .filter(|triangle| triangle.face() == face)
        .map(|triangle| as_a_set(triangle.corners()))
        .collect()
}

/// How many of `face`'s triangles, sorted, are not among `unplaced`.
fn count_joining_other_points(mesh: &Mesh, face: FaceKey, unplaced: &HashSet<[u32; 3]>) -> usize {
    mesh.triangles()
        .iter()
        .filter(|triangle| triangle.face() == face)
        .filter(|triangle| !unplaced.contains(&as_a_set(triangle.corners())))
        .count()
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
