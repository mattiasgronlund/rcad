//! How far a ball's mesh points really sit from the ball, measured against the
//! centre and radius the ball was built with.
//!
//! `Mesh::deviation` publishes a distance and a certified mesh promises never to
//! stray further than the deflection it was asked for. Both of those are the
//! kernel's own account of itself. This file does not read either one to decide
//! whether it is true: it takes the points out of the mesh, works out
//! `|p - centre| - radius` for each of them, and compares that to what was
//! promised.
//!
//! Where a part sits, every mesh point of a ball is *on* the sphere by
//! construction — a chord cuts inside the arc, but the chord's **ends** do not.
//! So the number measured here is a roundoff, not a chord sag, and what a
//! deflection buys is how far the middle of a triangle sits inside the surface,
//! which is a different distance and not the one this file measures.
//!
//! **"By construction" runs out, and this file does not cover where.** It holds
//! the sentence over the radii and the two placements below — out to ten
//! thousand — and not past them. A ball smaller than the spacing of the `f64`
//! grid it is placed on has no representable point on its own surface at all, so
//! far enough out the points come off the sphere by a share of the radius that
//! grows with the distance, and the roundoff this file measures stops being a
//! roundoff. `how_far_a_balls_mesh_strays_when_the_ball_sits_far_from_the_origin.rs`
//! is what measures that end, out to `1e13`, and carries the figures.

use rcad_geom::{Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Mesh, Model};

/// The deflections every ball below is cut at.
const HOW_FINE: [f64; 5] = [1.0, 0.3, 0.1, 0.03, 0.01];

/// The radii of the balls.
const RADII: [f64; 4] = [0.25, 1.0, 3.0, 10.0];

/// The furthest any point of `mesh` sits from the sphere of `radius` about
/// `centre`, as a fraction of the radius and as a length.
fn how_far_off_the_sphere(mesh: &Mesh, centre: Point, radius: f64) -> (f64, f64) {
    let worst = mesh
        .points()
        .iter()
        .map(|p| (centre.distance_to(*p) - radius).abs())
        .fold(0.0, f64::max);
    (worst, worst / radius)
}

/// Where the balls are put, because a ball a long way out is carried into place
/// by an arithmetic that rounds against how far out it sits.
///
/// A ball at the origin is the easy case, and it is not the case a consumer
/// draws. The far one is a thousand times the largest radius here, so its points
/// round against a coordinate of that size and not against the radius.
fn where_the_balls_go() -> [(&'static str, Isometry); 2] {
    [
        ("at the origin", Isometry::identity()),
        (
            "ten thousand out",
            Isometry::translation(Vector::new(10_000.0, -5_000.0, 2_500.0)).expect("a movement"),
        ),
    ]
}

/// Where a part sits, a certified ball's mesh points sit on the sphere to within
/// roundoff, and the log says nothing because nothing was fudged.
///
/// The qualifier is load-bearing and is [`where_the_balls_go`]: this holds at the
/// origin and ten thousand out, which is where a part sits, and it is false far
/// enough past that. `Mesh::deviation`'s doc cites this test by name for the
/// first half of that, so the range here is what that sentence means.
///
/// The claim under test is the one a downstream consumer disputed: that a
/// certified ball comes back with points a quarter of a radius off, refusing
/// nothing and logging nothing. Both halves are checked here from the ball's own
/// centre and radius rather than from anything the mesh says about itself.
///
/// The tolerance is a fixed handful of ulps of **how far out the ball's surface
/// reaches**, which is its centre plus its radius, and not of the radius alone.
/// That is what the arithmetic rounds against: a ball of radius a quarter
/// sitting ten thousand out has its points built from coordinates of ten
/// thousand, and nothing recovers bits the placement never had. It is still a
/// hundred million million times smaller than the quarter of a radius under
/// dispute.
#[test]
fn a_certified_balls_mesh_points_sit_on_the_sphere_and_the_log_stays_empty() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    for (where_it_is, placement) in where_the_balls_go() {
        let centre = placement.apply_to_point(Point::origin());
        for radius in RADII {
            let room = 8.0 * f64::EPSILON * (centre.largest_coordinate_magnitude() + radius);
            let ball = model.add_ball(radius, &placement).expect("a ball");
            for how_fine in HOW_FINE {
                let mut log = ProvenanceLog::new();
                let mesh = model
                    .solid(ball)
                    .mesh(
                        Deflection::of(how_fine).expect("a length"),
                        &Policy::certified(),
                        &mut log,
                    )
                    .expect("a mesh");
                let (off, share) = how_far_off_the_sphere(&mesh, centre, radius);
                println!(
                    "radius {radius} {where_it_is} at deflection {how_fine}: {} points, worst off \
                     the sphere {off:.3e} = {share:.3e} of the radius, deviation {:.3e}, log {} \
                     records",
                    mesh.points().len(),
                    mesh.deviation().magnitude(),
                    log.len()
                );
                assert!(
                    log.is_empty(),
                    "a certified ball of radius {radius} {where_it_is} at deflection {how_fine} \
                     logged {} records",
                    log.len()
                );
                assert!(
                    off <= room,
                    "a ball of radius {radius} {where_it_is} at deflection {how_fine} has a mesh \
                     point {off} off the sphere, which is {share} of the radius and past the \
                     {room} its placement rounds by"
                );
            }
        }
    }
}

/// The distance a caller can actually see is the sag in the middle of a
/// triangle, it is a large share of a small ball's radius, and it is inside
/// what the mesh said it would be.
///
/// This is the number a consumer measuring "how far is this mesh from a ball"
/// gets if it samples the surface of the mesh rather than only its corners. It
/// is bounded by [`Mesh::deviation`] and by the deflection asked for, and on a
/// ball whose radius is a few deflections across it is a quarter of the radius
/// or more — which is the deflection doing what it was asked to do and not a
/// deformation.
#[test]
fn the_sag_in_the_middle_of_a_triangle_is_large_but_never_past_the_stated_deviation() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    for radius in RADII {
        let ball = model
            .add_ball(radius, &Isometry::identity())
            .expect("a ball");
        for how_fine in HOW_FINE {
            let mut log = ProvenanceLog::new();
            let mesh = model
                .solid(ball)
                .mesh(
                    Deflection::of(how_fine).expect("a length"),
                    &Policy::certified(),
                    &mut log,
                )
                .expect("a mesh");
            let sag = worst_sag_across_the_triangles(&mesh, Point::origin(), radius);
            let stated = mesh.deviation().magnitude();
            println!(
                "radius {radius} at deflection {how_fine}: worst sag {sag:.4e} = {:.2}% of the \
                 radius, stated deviation {stated:.4e} = {:.2}% of the radius",
                100.0 * sag / radius,
                100.0 * stated / radius
            );
            assert!(
                sag <= stated,
                "a ball of radius {radius} at deflection {how_fine} sags {sag} in the middle of a \
                 triangle, past the {stated} it said it would keep to"
            );
            assert!(
                stated <= how_fine,
                "a certified ball of radius {radius} charged itself {stated} for a deflection of \
                 {how_fine}"
            );
        }
    }
}

/// A sag of about a quarter of the radius is what a deflection of about three
/// quarters of the radius buys, and it is ordinary rather than a deformation.
///
/// The sweep is here because the sag does not slide with the deflection: it
/// **steps**, since how many rows of triangles a ball is cut into is a whole
/// number. So no deflection sags a unit ball exactly a quarter of its radius —
/// the steps around there are what the sweep prints — and a consumer reporting
/// "up to a quarter" is reporting one of those steps, reached by asking for a
/// deflection near the ball's own size.
///
/// Nothing in the sweep strays past what the mesh said it would keep to, and
/// nothing strays past the deflection asked for. Those two are the promise; the
/// size of the sag is not one.
#[test]
fn a_quarter_of_a_radius_of_sag_is_what_a_deflection_near_the_radius_buys() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ball = model.add_ball(1.0, &Isometry::identity()).expect("a ball");
    let mut a_quarter_ish_was_seen = false;
    for step in 1..=20 {
        let how_fine = f64::from(step) * 0.05;
        let mesh = model
            .solid(ball)
            .mesh(
                Deflection::of(how_fine).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a mesh");
        let sag = worst_sag_across_the_triangles(&mesh, Point::origin(), 1.0);
        let stated = mesh.deviation().magnitude();
        println!(
            "deflection {how_fine:.2} of a unit ball: sag {:.2}% of the radius, stated {:.2}%",
            100.0 * sag,
            100.0 * stated
        );
        assert!(
            sag <= stated && stated <= how_fine,
            "a unit ball asked for {how_fine} said {stated} and sagged {sag}"
        );
        if (0.20..0.28).contains(&sag) {
            a_quarter_ish_was_seen = true;
        }
    }
    assert!(
        a_quarter_ish_was_seen,
        "no deflection between a twentieth and a whole radius sags a unit ball about a quarter of \
         its radius, so a report of a quarter cannot be a ball meshed this way"
    );
}

/// The furthest any point *on* a triangle of `mesh` sits from the sphere.
///
/// Sampled on a barycentric grid rather than at the centroid alone: a triangle
/// standing in for a piece of sphere sags most somewhere inside it, and where
/// that is depends on the triangle's shape. Twenty-one samples per triangle is
/// enough to see a quarter-radius sag; it is not a proof of the worst case and
/// is not used as one — the assertion above is that the sag stays *under* the
/// stated deviation, and undersampling can only make that easier to pass, which
/// is why the sag is also printed and is large.
fn worst_sag_across_the_triangles(mesh: &Mesh, centre: Point, radius: f64) -> f64 {
    const STEPS: usize = 5;
    let mut worst: f64 = 0.0;
    for triangle in mesh.triangles() {
        let [a, b, c] = triangle.corners().map(|i| mesh.points()[i as usize]);
        for i in 0..=STEPS {
            for j in 0..=(STEPS - i) {
                let (u, v) = (i as f64 / STEPS as f64, j as f64 / STEPS as f64);
                let w = 1.0 - u - v;
                let inside = Point::new(
                    u * a.x() + v * b.x() + w * c.x(),
                    u * a.y() + v * b.y() + w * c.y(),
                    u * a.z() + v * b.z() + w * c.z(),
                );
                worst = worst.max((centre.distance_to(inside) - radius).abs());
            }
        }
    }
    worst
}
