//! A quarter of a pointed cone's face, cut up to the apex the way `rcad-topo`
//! cuts it, meshed at deflections that used to refuse.
//!
//! # What this is about
//!
//! `docs/decisions.md` §51.7 recorded a pointed cone of slope one up to two
//! refusing `RanOutOfRoomBeforeMeetingTheDeflection` at a thousandth of its own
//! size with meshes on both sides of it — six refusals interleaved with nine
//! finer successes over `1e-4 … 1e-2`, so refusals were not monotone in
//! deflection and no ladder of coarser retries could be trusted.
//!
//! It was a **gap and not a limit**: the answer existed and the bound could not
//! see it (§16.10). Cutting a cone's face up to its apex leaves the apex in the
//! patch at **two longitudes a quarter turn apart**, so every triangulation of
//! it has one triangle with two corners on the apex. In 3D that triangle is the
//! segment from the apex to its third corner — a ruling, which lies *exactly on*
//! the cone and strays by nothing. The old bound collapsed the piece into one
//! chord measured on the circle of its widest corner, which charged the apex
//! corners as though they sat out there too, and the refinement then chased a
//! deflection it could never meet by fanning ever-thinner triangles at the apex
//! until the packing floor declined the next point.
//!
//! `how_far_a_flat_piece_of_a_cone_strays` now stops one step earlier in the
//! same derivation and keeps each corner's own radius. This file is the
//! demonstration: the patch that refused meshes, and every triangle of it is
//! measured against the **real cone** rather than against the bound.
//!
//! # Why the patch is built here rather than meshed through a solid
//!
//! `rcad-mesh` is shown one patch at a time and knows no topology, so the shape
//! that provoked this is described here in the surface's own two coordinates —
//! the same shape `cut_a_cap_up_to_its_pole` hands over, with the apex twice.

use std::f64::consts::FRAC_PI_2;

use rcad_geom::{Axis, Cone, Deflection, Surface};
use rcad_mesh::{Loop, TrimmedDomain, triangulate};

const SLOPE: f64 = 1.0;
const FAR: f64 = 2.0;

/// The deflections §51.7 swept, as a share of the cone's size — a tenth of a
/// decade at a time from a ten-thousandth to a hundredth.
///
/// Six of these refused before, at `4e-4`, `8e-4`, `1e-3`, `1.3e-3`, `2e-3` and
/// `2.5e-3` of the size, with nine meshes sitting finer than the coarsest of
/// them. The list is swept whole rather than at the six, because what was wrong
/// was the shape of the answer over the stretch and not six points of it.
fn the_stretch_that_was_ragged() -> Vec<f64> {
    (0..21)
        .map(|step| 1e-4 * 10f64.powf(f64::from(step) / 10.0))
        .collect()
}

/// Every deflection over that stretch meshes, and every triangle of every mesh
/// really is within it.
///
/// Two claims and the second is the one that costs something. A mesh that came
/// back is worth nothing if it came back by giving up on the deflection, so the
/// distance from each triangle to the cone is measured **exactly** — a cone is
/// one of the few surfaces whose distance function is closed form, and for a
/// point inside the nappe it is `(t·z − √(x² + y²))/√(1 + t²)`.
#[test]
fn a_cone_cut_up_to_its_apex_meshes_at_every_deflection_over_the_stretch() {
    let cone = Cone::new(Axis::z(), SLOPE).expect("a positive slope");
    let surface = Surface::from(cone);
    let mut worst_share = 0.0_f64;

    for share in the_stretch_that_was_ragged() {
        let asked = share * FAR;
        let domain = a_quarter_of_a_pointed_cones_face(surface.clone(), asked);
        let patch = match triangulate(&domain) {
            Ok(patch) => patch,
            Err(why) => panic!("a quarter cone at {share} of its size refused: {why}"),
        };
        assert!(
            !patch.triangles().is_empty(),
            "a mesh with no triangles in it is not a mesh"
        );
        assert!(
            patch.deviation().magnitude() <= asked,
            "the patch says it strays {} and it was asked for {asked}",
            patch.deviation().magnitude()
        );

        let really = how_far_the_mesh_really_strays(&cone, &patch);
        assert!(
            really <= patch.deviation().magnitude(),
            "at {share} of its size the mesh really strays {really}, and the patch \
             claims {}",
            patch.deviation().magnitude()
        );
        worst_share = worst_share.max(really / asked);
    }

    println!(
        "the worst mesh over the stretch used {worst_share} of the deflection it was asked for"
    );
}

/// The one triangle the old bound could not see, on its own.
///
/// A patch cut up to the apex has the apex at two longitudes, so some triangle
/// has both of them. In 3D it is a ruling and strays by nothing; the old bound
/// charged it `t·v/√(1 + t²)` for its third corner's height `v`, and refused
/// this very deflection with `RanOutOfRoomBeforeMeetingTheDeflection` after
/// fanning that corner down towards the apex until the packing floor stopped it.
///
/// The share is the finest of [`the_stretch_that_was_ragged`] that this patch
/// refused at before, so a mesh coming back at all is the claim.
#[test]
fn the_triangle_with_two_corners_on_the_apex_is_not_charged_for_a_quarter_turn() {
    let cone = Cone::new(Axis::z(), SLOPE).expect("a positive slope");
    let surface = Surface::from(cone);
    let asked = the_stretch_that_was_ragged()[3] * FAR;

    let domain = a_quarter_of_a_pointed_cones_face(surface, asked);
    let patch = triangulate(&domain).expect("a quarter of a pointed cone's face meshes");

    let touching_the_apex = patch
        .triangles()
        .iter()
        .filter(|triangle| {
            triangle
                .iter()
                .filter(|&&corner| patch.corners()[corner as usize][1] == 0.0)
                .count()
                >= 2
        })
        .count();
    assert!(
        touching_the_apex > 0,
        "the shape this is about is a triangle with two corners on the apex, and \
         this mesh has none"
    );
    assert!(
        how_far_the_mesh_really_strays(&cone, &patch) <= asked,
        "and it is within the deflection like every other triangle"
    );
}

/// A quarter of a pointed cone's face, in the cone's own two coordinates.
///
/// The shape `cut_a_cap_up_to_its_pole` hands over: a quarter of the base circle
/// at `FAR`, a cut down one ruling to the apex, **the apex twice** at the two
/// longitudes the cuts arrive at, and the other cut back up. The rim is walked
/// finely enough that no side of it strays further than the deflection, because
/// this crate refuses a loop it was handed too coarse and refining one is the
/// caller's job (`docs/decisions.md` §25.1).
fn a_quarter_of_a_pointed_cones_face(surface: Surface, deflection: f64) -> TrimmedDomain {
    let mut corners = Vec::new();
    for step in 0..=how_many_steps_the_rim_needs(deflection) {
        let along = f64::from(step) / f64::from(how_many_steps_the_rim_needs(deflection));
        corners.push([along * FRAC_PI_2, FAR]);
    }
    for step in 1..DOWN_THE_CUT {
        corners.push([FRAC_PI_2, FAR * f64::from(DOWN_THE_CUT - step) / f64::from(DOWN_THE_CUT)]);
    }
    corners.push([FRAC_PI_2, 0.0]);
    corners.push([0.0, 0.0]);
    for step in 1..DOWN_THE_CUT {
        corners.push([0.0, FAR * f64::from(step) / f64::from(DOWN_THE_CUT)]);
    }

    TrimmedDomain::new(
        surface,
        Loop::through(corners).expect("a closed quarter of a cone"),
        Vec::new(),
        Deflection::of(deflection).expect("a positive length"),
    )
}

/// How many corners a quarter of the base circle needs.
///
/// A side of the rim runs between two corners at the same height `FAR`, so what
/// it strays is `R·(1 − cos Δ)/√(1 + t²)` with `R = t·FAR`. Turned round, that
/// is the widest turn one side may take, and the quarter is cut into that many
/// pieces and one more.
fn how_many_steps_the_rim_needs(deflection: f64) -> u16 {
    let radius = SLOPE * FAR;
    let widest_turn = (1.0 - deflection * SLOPE.hypot(1.0) / radius).acos();
    let steps = (FRAC_PI_2 / widest_turn).ceil() + 1.0;
    u16::try_from(steps as u64).expect("a quarter turn does not need that many corners")
}

/// How many corners each cut from the rim down to the apex is walked into.
///
/// A cut runs straight up a ruling, so every one of its sides strays by nothing
/// whatever this is, and a handful is enough to give the triangulation something
/// to hang the first row of triangles on.
const DOWN_THE_CUT: u16 = 4;

/// How far the furthest point of any triangle really sits off the cone.
///
/// Exact rather than sampled against a surface: for a point inside the nappe the
/// distance to the cone is `(t·z − √(x² + y²))/√(1 + t²)`, so each triangle is
/// walked on a barycentric grid and the worst is kept.
fn how_far_the_mesh_really_strays(cone: &Cone, patch: &rcad_mesh::Patch) -> f64 {
    const STEPS: usize = 12;
    let mut furthest = 0.0_f64;
    for triangle in patch.triangles() {
        let at = triangle.map(|corner| {
            let [round, along] = patch.corners()[corner as usize];
            cone.point_at(round, along)
        });
        for first in 0..=STEPS {
            for second in 0..=(STEPS - first) {
                let third = STEPS - first - second;
                let weights = [first, second, third].map(|share| share as f64 / STEPS as f64);
                let up: f64 = (0..3).map(|which| weights[which] * at[which].z()).sum();
                let x: f64 = (0..3).map(|which| weights[which] * at[which].x()).sum();
                let y: f64 = (0..3).map(|which| weights[which] * at[which].y()).sum();
                furthest =
                    furthest.max((cone.slope() * up - x.hypot(y)) / cone.slope().hypot(1.0));
            }
        }
    }
    furthest
}
