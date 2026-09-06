//! How far a curved solid's measured volume sits from the one it really has.
//!
//! `Mesh::deviation`'s doc publishes a bound on that distance, and §15.6 says a
//! published bound needs a test that could fail. This is it. The true answers
//! here are known in closed form — `4πr³/3` and `4πr²` for a ball, `πr²h` and
//! `2πr(r + h)` for a bar — so nothing is checked against the kernel's own idea
//! of what it did.
//!
//! The bound is worked out **here**, from the geometry the shape was built with,
//! and not read back off the `Measurement`. Checking a kernel's answer against
//! the same kernel's `volume_error` would pass however wrong both were.
//!
//! `docs/decisions.md` §47.4 item 1 is why the first thing every check does is
//! ask whether the bound is a number. A bound of `f64::INFINITY` cannot be
//! violated, and a suite that asserts against one has measured nothing.
//!
//! # The second half of the file, and why the first half needed one
//!
//! Everything above runs on four balls and eight round bars at the origin. What
//! feeds the bound underneath is the mesher's own idea of how far a face's
//! points reach — `mesh::how_far_the_points_of_reach`, a face's wire corners
//! plus the radius its surface bends at — and **on those twelve shapes that
//! number cannot be told apart from a corrected one**. Both charges agree to the
//! last bit on every one of the twelve, so a change to what the mesher charges
//! could not move a single figure this file prints.
//!
//! §52.7 is what that costs: a bound was tightened, its own §15.6 search
//! reported a change of one ulp, and the reason was that the search's family sat
//! exactly on the subspace where the old and new formulas are algebraically the
//! same. Green for years, watching a bound the code no longer computed.
//!
//! So the file carries a second family that is *not* on that subspace —
//! [`shapes_whose_faces_run_past_their_own_corners`] — and prints both charges
//! side by side on every row of both families, which is what says the second one
//! can see what the first cannot.
//!
//! The mesher has since stopped charging the corners for a pipe and a cone, and
//! charges how far along its axis a face's **edges** run instead. Every number
//! this file prints is unchanged by that, which is the point: neither charge
//! here is read off the kernel, so the two families still measure the same gap
//! and this one is still the one that can see it.

use rcad_geom::{Axis, Deflection, Isometry, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    FaceKey, FaceView, Measurement, Mesh, MeshTriangle, Model, Rule, SolidKey, WireView,
};

const PI: f64 = core::f64::consts::PI;

/// The deflections every shape below is measured at.
///
/// Six of them over three decades, because two points fix a line through
/// anything and say nothing about whether the thing is a line.
const HOW_FINE: [f64; 6] = [1.0, 0.3, 0.1, 0.03, 0.01, 0.003];

/// The bound `Mesh::deviation` publishes: `skin · d · (R / (R − d))²`.
///
/// `skin` is the area of the triangles, `d` how far any of them may sit from the
/// surface, and `R` the smallest radius any curved face of the solid bends at.
/// Infinite when the mesh may be a whole radius out, which is the case that has
/// to be checked for before anything is asserted against it.
///
/// Every caller below passes the **deflection it asked for** rather than the
/// deviation the mesh came back with. That is the looser of the two, because the
/// deviation is never the larger under a certified policy and the bound only
/// grows with `d` — and it is the one worth checking, because a deflection is
/// what a caller has before there is a mesh to read a deviation off.
fn how_far_a_measured_volume_can_be_out(skin: f64, deflection: f64, smallest_radius: f64) -> f64 {
    let inner = smallest_radius - deflection;
    if inner <= 0.0 {
        return f64::INFINITY;
    }
    let widening = smallest_radius / inner;
    skin * deflection * widening * widening
}

/// A ball, a tall bar and a short wide one, at four sizes each.
///
/// Every one carries the volume it really has and the radius its curved faces
/// bend at, both worked out from what it was built with.
fn shapes_whose_volume_is_known() -> (Model, Vec<(String, SolidKey, f64, f64)>) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let mut shapes = Vec::new();
    for radius in [0.25, 1.0, 3.0, 10.0] {
        let ball = model
            .add_ball(radius, &Isometry::identity())
            .expect("a ball");
        shapes.push((
            format!("ball of radius {radius}"),
            ball,
            4.0 / 3.0 * PI * radius * radius * radius,
            radius,
        ));
        for height in [0.5, 5.0] {
            let bar = model
                .add_round_bar(radius, height, &Isometry::identity())
                .expect("a bar");
            shapes.push((
                format!("bar {radius} across and {height} tall"),
                bar,
                PI * radius * radius * height,
                radius,
            ));
        }
    }
    (model, shapes)
}

fn measured(model: &Model, solid: SolidKey, how_fine: f64) -> Measurement {
    model
        .solid(solid)
        .measure(
            Deflection::of(how_fine).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("measuring at a deflection of {how_fine}: {why}"))
}

/// The bound holds over every shape whose volume this suite knows in closed
/// form, and it holds with room to spare that is stated rather than hoped for.
///
/// Two numbers come out of this and both are asserted. The bound is never
/// broken, which is what makes it a bound; and somewhere in the family the real
/// error gets within a stated share of it, which is what stops it being a bound
/// nothing can violate.
#[test]
fn a_measured_volume_sits_inside_the_published_bound_over_balls_and_bars() {
    let (model, shapes) = shapes_whose_volume_is_known();

    let mut worst_share = 0.0f64;
    let mut worst_at = String::new();
    let mut slackest_share = f64::MAX;
    let mut slackest_at = String::new();
    let mut worst_share_of_the_kernels_own = 0.0f64;
    let mut too_coarse_to_bound = 0;
    let mut bounded = 0;
    for (what, solid, true_volume, radius) in shapes {
        for how_fine in HOW_FINE {
            let measured = measured(&model, solid, how_fine);
            let bound =
                how_far_a_measured_volume_can_be_out(measured.surface_area(), how_fine, radius);
            if !bound.is_finite() {
                assert!(
                    how_fine >= radius,
                    "the bound is only allowed to give up when the mesh may be a whole \
                     radius out, and a {what} at a deflection of {how_fine} is not that"
                );
                too_coarse_to_bound += 1;
                continue;
            }
            bounded += 1;

            let out_by = (true_volume - measured.volume()).abs();
            assert!(
                out_by <= bound,
                "a {what} measured at a deflection of {how_fine} came to {} against the \
                 {true_volume} it really holds, which is {out_by} out of a bound of {bound}",
                measured.volume()
            );
            let share = out_by / bound;
            if share > worst_share {
                worst_share = share;
                worst_at = format!("a {what} at a deflection of {how_fine}");
            }
            if share < slackest_share {
                slackest_share = share;
                slackest_at = format!("a {what} at a deflection of {how_fine}");
            }

            // The kernel's own bound is the tighter of the two, and the doc
            // says by how much. It is checked here rather than in `measuring.rs`
            // because this is where the comparison is published, and it is
            // checked **two-sided**: `volume_error` reads as a shortfall and is
            // a distance.
            assert!(
                measured.volume_error().is_finite() && out_by <= measured.volume_error(),
                "a {what} at a deflection of {how_fine} is {out_by} out and the kernel \
                 owned up to {}",
                measured.volume_error()
            );
            worst_share_of_the_kernels_own =
                worst_share_of_the_kernels_own.max(out_by / measured.volume_error());
        }
    }

    assert!(
        too_coarse_to_bound > 0 && bounded > 60,
        "the family has to reach both sides of the bound's own limit, and it bounded \
         {bounded} and gave up on {too_coarse_to_bound}"
    );
    // A fifth of the bound is what a ball of radius three at a deflection of
    // 0.003 really loses, and it is the number `Mesh::deviation`'s doc
    // publishes as the bound's slack. Falling under this means the bound has
    // got looser than the doc says it is, which is the failure this catches —
    // the same failure as breaking the bound, from the other side.
    assert!(
        worst_share > 0.18,
        "the bound is meant to be within about five times what really goes missing, and \
         the closest anything came was {worst_share} at {worst_at}"
    );
    // The other end, published beside it because a bound is worth having only
    // if how much slack it has is said out loud. A short wide bar is where it
    // is loosest: its flat caps carry area that the bound charges as if it
    // bent, and nothing at all hides under a flat face.
    assert!(
        slackest_share < 0.01,
        "the doc says the bound runs to a hundred times the truth on a short wide bar, \
         and the loosest here was {slackest_share} at {slackest_at}"
    );
    assert!(
        worst_share_of_the_kernels_own > 0.24,
        "the doc says `volume_error` runs at four times what really goes missing, and \
         the closest it came was {worst_share_of_the_kernels_own}"
    );
    println!("{bounded} bounded, {too_coarse_to_bound} too coarse to bound");
    println!("worst share of `volume_error` used: {worst_share_of_the_kernels_own:.4}");
    println!("worst share of the bound used: {worst_share:.4} at {worst_at}");
    println!("slackest share of the bound used: {slackest_share:.5} at {slackest_at}");
}

/// What goes missing falls **in step with** the deflection, not with its square.
///
/// Fitted, not eyeballed off two points: a least-squares line through
/// `log(missing)` against `log(deflection)` over the five finest deflections of
/// every shape in the family. A slope of one is the claim; the test is what says
/// the data supports it.
#[test]
fn the_volume_a_mesh_misses_falls_in_step_with_the_deflection() {
    let (model, shapes) = shapes_whose_volume_is_known();

    let mut steepest = f64::MIN;
    let mut shallowest = f64::MAX;
    for (what, solid, true_volume, _radius) in shapes {
        let mut points = Vec::new();
        for how_fine in [0.1, 0.03, 0.01, 0.003, 0.001] {
            let out_by = (true_volume - measured(&model, solid, how_fine).volume()).abs();
            assert!(
                out_by > 0.0,
                "a {what} at a deflection of {how_fine} claims to have been measured exactly"
            );
            points.push((how_fine.ln(), out_by.ln()));
        }
        let slope = slope_of_the_best_line_through(&points);
        assert!(
            (0.9..=1.1).contains(&slope),
            "a {what} loses volume as the deflection to the power {slope}, and the doc \
             says that power is one"
        );
        steepest = steepest.max(slope);
        shallowest = shallowest.min(slope);
    }
    println!("slopes ran from {shallowest:.3} to {steepest:.3}");
}

fn slope_of_the_best_line_through(points: &[(f64, f64)]) -> f64 {
    let count = points.len() as f64;
    let mean_across = points.iter().map(|(across, _)| across).sum::<f64>() / count;
    let mean_up = points.iter().map(|(_, up)| up).sum::<f64>() / count;
    let mut leaning = 0.0;
    let mut spread = 0.0;
    for (across, up) in points {
        leaning += (across - mean_across) * (up - mean_up);
        spread += (across - mean_across) * (across - mean_across);
    }
    leaning / spread
}

/// The bound is on the **distance**, and this is the shape that says it has to
/// be: a block with a round hole measures **bigger** than it is.
///
/// A round bar's wall bends away from its material, so its chords lie inside the
/// bar and the sum over them is short. A hole's wall bends the other way: the
/// chords cut into the material the block is made of, so the mesh solid is the
/// block minus a prism inscribed in the hole, which is more block than there
/// really is. Same shell, same thickness, opposite sign.
#[test]
fn a_block_with_a_round_hole_measures_bigger_than_it_is() {
    const RADIUS: f64 = 1.0;
    let true_volume = 512.0 - PI * RADIUS * RADIUS * 8.0;

    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let block = model
        .add_cube(8.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a block eight across");
    let bar = model
        .add_round_bar(
            RADIUS,
            12.0,
            &Isometry::translation(Vector::new(4.0, 4.0, -2.0)).expect("a shift"),
        )
        .expect("a bar through the middle of it");
    let holed = model
        .fuse(
            &[block],
            &[bar],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a block with a hole through it");
    let holed = holed.solids()[0];

    for how_fine in [0.1, 0.03, 0.01, 0.003] {
        let measured = measured(&model, holed, how_fine);
        assert!(
            measured.volume() > true_volume,
            "the mesh of a hole is a prism inside it, so at a deflection of {how_fine} the \
             block has to come out bigger than the {true_volume} it is, and it came out {}",
            measured.volume()
        );
        let bound = how_far_a_measured_volume_can_be_out(measured.surface_area(), how_fine, RADIUS);
        assert!(
            bound.is_finite(),
            "nothing is worth asserting against an infinite bound (§47.4)"
        );
        assert!(
            measured.volume() - true_volume <= bound,
            "the block is {} over the {true_volume} it holds, against a bound of {bound}",
            measured.volume() - true_volume
        );
    }
}

/// What the bound is **for**: `vol(A ∪ B) + vol(A ∩ B) = vol(A) + vol(B)` over a
/// pair with a curved face in it.
///
/// `fuses_that_must_agree.rs` asserts that law exactly, and exactly is all it
/// can do over flat solids, where every volume is a determinant. A curved
/// volume is a sum over an approximation, so the law can only be asserted to a
/// tolerance — and a tolerance picked to make the test pass is worth nothing.
/// This is the tolerance the four measurements themselves declare: each is out
/// by at most its own `volume_error`, so the four sides of the law are out by at
/// most the four added up.
///
/// A block and a bar through it, because that pair is one the fuse suite already
/// trusts.
#[test]
fn a_curved_pair_obeys_inclusion_and_exclusion_inside_the_error_the_four_declare() {
    for how_fine in [0.1, 0.03, 0.01, 0.003] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let block = model
            .add_cube(8.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a block eight across");
        let bar = model
            .add_round_bar(
                1.0,
                12.0,
                &Isometry::translation(Vector::new(4.0, 4.0, -2.0)).expect("a shift"),
            )
            .expect("a bar through the middle of it");

        let mut both_ways = Vec::new();
        for rule in [Rule::Union, Rule::Intersection] {
            let fused = model
                .fuse(
                    &[block],
                    &[bar],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("a block and a bar {rule:?}: {why}"));
            let solids = fused.solids();
            assert_eq!(
                solids.len(),
                1,
                "{rule:?} of a block and a bar is one solid"
            );
            both_ways.push(measured(&model, solids[0], how_fine));
        }
        let each_side = [
            measured(&model, block, how_fine),
            measured(&model, bar, how_fine),
        ];

        let out_by = both_ways.iter().map(|m| m.volume()).sum::<f64>()
            - each_side.iter().map(|m| m.volume()).sum::<f64>();
        let allowed = both_ways
            .iter()
            .chain(&each_side)
            .map(|m| m.volume_error())
            .sum::<f64>();
        assert!(
            allowed.is_finite(),
            "nothing is worth asserting against an infinite tolerance (§47.4)"
        );
        assert!(
            out_by.abs() <= allowed,
            "at a deflection of {how_fine} the two sides of the law differ by {out_by}, \
             and the four measurements only declared {allowed}"
        );
        // **And a hundredth of the tolerance is all it uses**, which is the
        // honest thing to say about it: the four bounds added up would still
        // pass a fuse whose volume was seventy times further out than this one
        // is. The errors on the four sides are the same shells charged twice
        // with opposite signs, so they cancel and the sum does not know it.
        // Pinning the share is what gives the check teeth; it is a pin on this
        // pair's arithmetic and it will move if the mesher does.
        assert!(
            out_by.abs() < 0.02 * allowed,
            "at a deflection of {how_fine} the law is {out_by} out, which is more of the \
             declared {allowed} than the hundredth this pair has ever used"
        );
        println!(
            "deflection {how_fine}: the law is {out_by:.6e} out of a declared {allowed:.6e}, \
             which is {:.4} of it",
            out_by.abs() / allowed
        );
    }
}

/// A mesh never strays further than the deflection it was asked for, which is
/// what lets the published bound be worked out before anything is meshed.
///
/// Only under a certified policy. A tolerant one is allowed to mesh **coarser**
/// than asked when the arithmetic cannot reach the request, and then the
/// deflection is not an upper bound on anything.
#[test]
fn a_certified_mesh_never_strays_further_than_the_deflection_it_was_asked_for() {
    let (model, shapes) = shapes_whose_volume_is_known();
    for (what, solid, _volume, _radius) in shapes {
        for how_fine in HOW_FINE {
            let deviation = model
                .solid(solid)
                .mesh(
                    Deflection::of(how_fine).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a mesh")
                .deviation()
                .magnitude();
            assert!(
                deviation <= how_fine,
                "a {what} asked for {how_fine} charged itself {deviation}"
            );
        }
    }
}

/// The bound gives up exactly where it says it does, and this is what "gives up"
/// looks like: not a large number, an infinite one.
///
/// The case that matters is a cone that ends in a point, whose radius of
/// curvature goes to zero at the apex. There is no public builder for one here,
/// so what is checked is the arithmetic of the bound itself at the radius such a
/// face reaches. §47.4 item 1 is a volume checked against exactly this number.
#[test]
fn the_bound_is_infinite_where_a_surface_bends_tighter_than_the_mesh_strays() {
    assert!(how_far_a_measured_volume_can_be_out(100.0, 0.1, 0.0).is_infinite());
    assert!(how_far_a_measured_volume_can_be_out(100.0, 0.1, 0.1).is_infinite());
    assert!(how_far_a_measured_volume_can_be_out(100.0, 0.1, 0.05).is_infinite());
    assert!(how_far_a_measured_volume_can_be_out(100.0, 0.1, 0.2).is_finite());
}

// ---------------------------------------------------------------------------
// How far a face's points reach, charged two ways and then measured
// ---------------------------------------------------------------------------

#[path = "cones/mod.rs"]
mod cones;

/// The largest coordinate a face's **wire corners** have, plus the radius its
/// surface bends at.
///
/// **What `mesh::how_far_the_points_of_reach` charged until the family below
/// made the shortfall visible**, written out here rather than called. Calling it
/// would make this test move with the thing it watches, which is §52.11's
/// mistake and the whole reason this family exists — and the mesher has since
/// moved: for a pipe and a cone it now charges how far along its axis the face's
/// **edges** run, worked out from the surfaces either side of each of them.
/// `what_a_leaning_cut_costs_the_floor_under_a_mesh.rs` is that change measured
/// from the caller's side.
///
/// So this is now a record of a charge nothing computes, kept because the
/// distance between it and [`what_the_whole_face_can_reach`] is what says this
/// family can see a change to the reach at all.
///
/// Flat faces are charged nothing: a plane's mesh is drawn through corners the
/// model already holds.
fn what_the_corners_alone_charge(face: FaceView<'_>) -> f64 {
    let radius = match face.surface() {
        Surface::Plane(_) => return 0.0,
        Surface::Sphere(ball) => ball.radius(),
        Surface::Cylinder(pipe) => pipe.radius(),
        Surface::Cone(cone) => {
            let along = cone.axis().direction();
            face.wires()
                .flat_map(WireView::points)
                .map(|corner| cone.radius_at(along.dot(corner - cone.apex())))
                .fold(0.0, f64::max)
        }
        Surface::Torus(ring) => ring.ring_radius() + ring.tube_radius(),
        other => panic!("this test cannot say how far {other:?} reaches"),
    };
    how_far_the_corners_are_from_the_origin(face) + radius
}

fn how_far_the_corners_are_from_the_origin(face: FaceView<'_>) -> f64 {
    face.wires()
        .flat_map(WireView::points)
        .map(|corner| corner.largest_coordinate_magnitude())
        .fold(0.0, f64::max)
}

/// The same reach worked out from **where the surface is placed** as well as
/// from the corners, which is what the corners alone cannot say.
///
/// A point of the face is its surface's placing point — a ball's centre, a
/// pipe's axis origin, a cone's apex — plus however far the face gets from it,
/// so `|placing point| + spread` bounds every coordinate it can have. The
/// spread is the surface's own size where the surface is bounded, and where it
/// is not, the corners say how far along the axis the face runs:
///
/// - a **ball** reaches its radius from its centre and no further, corners or
///   no corners;
/// - a **pipe** reaches its radius sideways and as far along its axis as its
///   corners do;
/// - a **cone** at `h` up its axis from the apex reaches `h` along and
///   `slope · h` out, so `(1 + slope) · h` bounds both, with `h` taken from the
///   corners;
/// - a **ring** reaches `R + r` from its centre whichever way it wraps.
///
/// Whichever of the two charges is larger is the one taken, so this is never
/// below [`what_the_corners_alone_charge`] and the ratio between them is how
/// much the corner-only charge is short by.
///
/// Nothing here is read back off the mesher. Every number comes off the shape
/// the test built.
///
/// **The pipe and the cone lean on their corners for how far along the axis the
/// face runs**, which is true of every face here — each is bounded by cap
/// circles whose points all sit at one height — and is not claimed of a face
/// this family does not build. [`how_short_the_corners_came`] asserts on every
/// row that the mesh points stayed inside this, which is what would catch a
/// shape where it does not hold.
fn what_the_whole_face_can_reach(face: FaceView<'_>) -> f64 {
    let (placed_by, spread) = match face.surface() {
        Surface::Plane(_) => return 0.0,
        Surface::Sphere(ball) => (ball.centre(), ball.radius()),
        Surface::Cylinder(pipe) => (
            pipe.axis().origin(),
            pipe.radius() + how_far_the_corners_run_along(face, pipe.axis()),
        ),
        Surface::Cone(cone) => (
            cone.apex(),
            (1.0 + cone.slope()) * how_far_the_corners_run_along(face, cone.axis()),
        ),
        Surface::Torus(ring) => (ring.centre(), ring.ring_radius() + ring.tube_radius()),
        other => panic!("this test cannot say how far {other:?} reaches"),
    };
    what_the_corners_alone_charge(face).max(placed_by.largest_coordinate_magnitude() + spread)
}

fn how_far_the_corners_run_along(face: FaceView<'_>, axis: Axis) -> f64 {
    face.wires()
        .flat_map(WireView::points)
        .map(|corner| axis.direction().dot(corner - axis.origin()).abs())
        .fold(0.0, f64::max)
}

/// The largest coordinate any of this face's **own mesh points** really has.
///
/// Measured off the mesh and not derived from anything, so it is a floor under
/// how far the face reaches and never a claim about the ceiling.
fn how_far_the_points_of_this_face_went(mesh: &Mesh, face: FaceKey) -> f64 {
    let points = mesh.points();
    mesh.triangles()
        .iter()
        .copied()
        .filter(|triangle| triangle.face() == face)
        .flat_map(MeshTriangle::corners)
        .map(|corner| points[corner as usize].largest_coordinate_magnitude())
        .fold(0.0, f64::max)
}

/// One curved face's row: what the corners charge, what the whole face can
/// reach, and where its mesh points really got to.
struct WhatOneFaceReached {
    charged: f64,
    can_reach: f64,
    went: f64,
}

/// Every curved face of `solid`, meshed at `how_fine` and charged both ways.
///
/// Flat faces are left out: both charges are zero on them and a ratio of zero
/// over zero says nothing.
fn what_each_curved_face_reached(
    model: &Model,
    solid: SolidKey,
    how_fine: f64,
) -> Vec<WhatOneFaceReached> {
    let mesh = model
        .solid(solid)
        .mesh(
            Deflection::of(how_fine).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("meshing at a deflection of {how_fine}: {why}"));
    model
        .solid(solid)
        .faces()
        .filter(|face| !matches!(face.surface(), Surface::Plane(_)))
        .map(|face| WhatOneFaceReached {
            charged: what_the_corners_alone_charge(face),
            can_reach: what_the_whole_face_can_reach(face),
            went: how_far_the_points_of_this_face_went(&mesh, face.key()),
        })
        .collect()
}

/// Print one shape's faces and hand back the worst two ratios on it.
fn how_short_the_corners_came(what: &str, faces: &[WhatOneFaceReached]) -> (f64, f64) {
    let mut worst_went = 0.0f64;
    let mut worst_can_reach = 0.0f64;
    for face in faces {
        assert!(
            face.went <= face.can_reach,
            "a face of {what} put a mesh point {} out, and the whole face was said to \
             reach no further than {}",
            face.went,
            face.can_reach
        );
        println!(
            "{what}: corners charge {:.4}, the whole face reaches {:.4} ({:.4}x), its points \
             got to {:.4} ({:.4}x)",
            face.charged,
            face.can_reach,
            face.can_reach / face.charged,
            face.went,
            face.went / face.charged
        );
        worst_went = worst_went.max(face.went / face.charged);
        worst_can_reach = worst_can_reach.max(face.can_reach / face.charged);
    }
    (worst_went, worst_can_reach)
}

/// Shapes with a curved face that runs further from the origin than its own
/// corners do.
///
/// **A second family on purpose, rather than more shapes in
/// [`shapes_whose_volume_is_known`].** That one is named in
/// `a_measured_volume_sits_inside_the_published_bound_over_balls_and_bars`, and
/// dropping a cone into it would make that test's own name false. None of these
/// has a closed-form volume for this file to check either, which is the second
/// reason they are not that family's business.
///
/// Four kinds, because one surface reaching past its corners could be a quirk of
/// that one surface:
///
/// - a **pointed cone dropped until its rim sits at the origin**. Its one corner
///   is on the rim and the face runs a whole height down to the apex. Three
///   slopes, because the shortfall here is `1 / (2 · slope)` and a ladder says
///   more than a number;
/// - a **squat frustum slid sideways** until its seam runs down the near side of
///   the axis, leaving the far side of the cone twice as far out as any corner;
/// - a **ball fused onto a block**, so all that is left of its corners is a
///   small circle near the origin while the face still reaches over the top;
/// - a **wide flat bar slid off its own axis**, which is the same trick on a
///   pipe.
///
/// Each carries its own model, because two of the four are put together by a
/// placement and one by a fuse.
fn shapes_whose_faces_run_past_their_own_corners() -> Vec<(String, Model, SolidKey, f64)> {
    let mut shapes = Vec::new();

    for slope in [0.25, 0.125, 0.0625] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let cone = cones::a_pointed_cone(&mut model, slope, 8.0);
        model.place_solid(
            cone,
            &Isometry::translation(Vector::new(0.0, 0.0, -8.0)).expect("a shift"),
        );
        shapes.push((
            format!("a cone of slope {slope} hanging below its own rim"),
            model,
            cone,
            0.1,
        ));
    }

    for (near, far) in [(0.125, 0.25), (0.25, 0.28125)] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let frustum = cones::a_frustum(&mut model, 8.0, near, far);
        model.place_solid(
            frustum,
            &Isometry::translation(Vector::new(-8.0 * far, 0.0, 0.0)).expect("a shift"),
        );
        shapes.push((
            format!("a squat frustum from {near} to {far} slid onto its own seam"),
            model,
            frustum,
            0.1,
        ));
    }

    for centre in [2.5, 2.75, 2.875] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let block = model
            .add_cube(8.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a block eight across");
        model.place_solid(
            block,
            &Isometry::translation(Vector::new(-4.0, -4.0, -8.0)).expect("a shift"),
        );
        let ball = model
            .add_ball(
                3.0,
                &Isometry::translation(Vector::new(0.0, 0.0, centre)).expect("a shift"),
            )
            .expect("a ball");
        let fused = model
            .fuse(
                &[block],
                &[ball],
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("a ball centred {centre} on a block: {why}"));
        let solid = fused.solids()[0];
        shapes.push((
            format!("a ball centred {centre} up, sunk into a block"),
            model,
            solid,
            0.1,
        ));
    }

    for height in [0.5, 0.25, 0.125] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let bar = model
            .add_round_bar(
                5.0,
                height,
                &Isometry::translation(Vector::new(-5.0, 0.0, 0.0)).expect("a shift"),
            )
            .expect("a bar");
        shapes.push((
            format!("a bar five across and {height} tall, slid onto its own seam"),
            model,
            bar,
            0.03,
        ));
    }

    shapes
}

/// A face's own mesh points can land further from the origin than the reach its
/// corners are charged for, so that charge is not an upper bound on anything.
///
/// **Both charges run side by side on every row**, which is what says the family
/// can see the difference at all —
/// [`no_ball_or_bar_the_volume_bound_measures_reaches_past_what_its_corners_charge`]
/// is the same two numbers over the family this file measures volumes on, where
/// they agree to the last bit. §52.7 is why that pair is written this way: a
/// §15.6 search whose family sits where the two formulas agree measures a bound
/// nobody computes, and stays green through the fix it was supposed to watch.
///
/// **The cones come out on a ladder that was predicted before it was run**, and
/// that is what says the mechanism is understood rather than stumbled on. A
/// cone of slope `s` hung below its own rim has one corner, on the rim, `s · far`
/// out and `radius_at` there is `s · far` too, so the corners charge `2 · s · far`
/// while the face runs the whole `far` down to the apex: `1 / (2s)`. At slopes a
/// quarter, an eighth and a sixteenth that is 2, 4 and 8, and it is 2, 4 and 8.
///
/// **Seen failing under the corrected charge**, which is what §52.9 asks of a
/// search that is meant to move: putting the surface's placing point into
/// [`what_the_corners_alone_charge`] drops the first cone's row from 2.0000 to
/// 0.8000 and this test fails on it, while every row of
/// [`no_ball_or_bar_the_volume_bound_measures_reaches_past_what_its_corners_charge`]
/// comes back bit for bit unchanged.
#[test]
fn a_cone_a_fused_ball_and_a_slid_bar_all_reach_past_what_their_corners_charge() {
    let mut worst_went = 0.0f64;
    let mut worst_went_at = String::new();
    let mut worst_can_reach = 0.0f64;
    let mut worst_can_reach_at = String::new();
    let mut short_by_half_or_more = 0;
    let shapes = shapes_whose_faces_run_past_their_own_corners();
    let how_many = shapes.len();
    for (what, model, solid, how_fine) in shapes {
        let faces = what_each_curved_face_reached(&model, solid, how_fine);
        assert!(!faces.is_empty(), "{what} has no curved face to charge");
        let (went, can_reach) = how_short_the_corners_came(&what, &faces);
        assert!(
            went > 1.0,
            "{what} is in this family because its points outrun the corner charge, and \
             they only reached {went} of it"
        );
        if can_reach >= 2.0 {
            short_by_half_or_more += 1;
        }
        if went > worst_went {
            worst_went = went;
            worst_went_at = what.clone();
        }
        if can_reach > worst_can_reach {
            worst_can_reach = can_reach;
            worst_can_reach_at = what;
        }
    }

    assert_eq!(how_many, 11, "the family is eleven shapes");
    // Eight, because a cone of slope one sixteenth hanging below its own rim is
    // charged one and its points reach eight. Re-derive it if the family moves;
    // do not relax it (§19.1).
    assert!(
        worst_went > 7.9,
        "the worst a face outran its corner charge by was {worst_went} at {worst_went_at}, \
         and this family was built to reach eight"
    );
    assert!(
        worst_can_reach > 16.0,
        "the whole-face reach was worst at {worst_can_reach} times the corner charge, at \
         {worst_can_reach_at}"
    );
    // The three cones, at 4.5, 8.5 and 16.5 times. The eight others fall short
    // by less than half — a ball and a pipe cannot be charged less than half
    // what they reach, because a corner of such a face sits on the surface and
    // so is already within one radius of the far side of it. A cone has no such
    // ceiling: its radius is charged where its corners are and nowhere else.
    assert!(
        short_by_half_or_more >= 3,
        "only {short_by_half_or_more} of the {how_many} were charged less than half what \
         the whole face reaches, and the three cones alone are that"
    );
    println!("worst a face outran its corner charge: {worst_went:.4}x at {worst_went_at}");
    println!(
        "worst the corner charge fell short of the whole face: {worst_can_reach:.4}x at \
         {worst_can_reach_at}"
    );
}

/// **The blindness itself.** Over the balls and bars this file measures volumes
/// on, the corner charge and the whole-face reach are the same number, and no
/// face's points ever reach past either.
///
/// So a change to what the mesher charges a face for cannot move a single number
/// in this file's volume tests, and a search that only ran on this family would
/// report the same figure before and after such a change — which is exactly
/// §52.7's failure. The family above is what stops that, and this test is what
/// says it was needed.
#[test]
fn no_ball_or_bar_the_volume_bound_measures_reaches_past_what_its_corners_charge() {
    let (model, shapes) = shapes_whose_volume_is_known();

    let mut worst_went = 0.0f64;
    let mut worst_can_reach = 0.0f64;
    for (what, solid, _volume, _radius) in shapes {
        let faces = what_each_curved_face_reached(&model, solid, 0.1);
        assert!(!faces.is_empty(), "a {what} has a curved face");
        let (went, can_reach) = how_short_the_corners_came(&what, &faces);
        worst_went = worst_went.max(went);
        worst_can_reach = worst_can_reach.max(can_reach);
    }

    assert!(
        worst_went <= 1.0,
        "no ball or bar in this family puts a mesh point past what its corners are \
         charged for, and one reached {worst_went} of it"
    );
    assert_eq!(
        worst_can_reach, 1.0,
        "the two charges are the same number on every one of these shapes, and the \
         furthest they came apart was {worst_can_reach}"
    );
    println!("the volume family's points reached {worst_went:.4} of what they were charged");
    println!("and the two charges never came apart at all: {worst_can_reach}");
}
