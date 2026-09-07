//! How far the **middle** of a mesh triangle really sits from the surface it
//! stands for, measured against each surface's own closed-form distance.
//!
//! `how_far_a_balls_mesh_points_really_sit_from_the_ball.rs` measures the mesh
//! *points*, and says in its own head note that what a deflection buys is how
//! far the middle of a triangle sits inside the surface, "which is a different
//! distance and not the one this file measures". This is that distance. Every
//! point of a curved face's mesh is on its surface by construction; the sag is
//! entirely in between them, so a file that only reads the corners cannot tell a
//! mesh that meets its deflection from one that misses by a factor of eight.
//!
//! # What it is here to catch
//!
//! `how_many_steps` in `rcad-topo`'s mesher is asked for one edge at a time. It
//! is handed that edge's two corners in the surface's own two coordinates and
//! nothing else, so it never sees the **cell** two of its steps close off, and a
//! cell's diagonal is longer than either of the sides that make it. Whether that
//! matters is not a question about the walk on its own: the triangulator
//! measures every triangle it holds and cuts the ones that stray too far, and
//! `Mesh::deviation` publishes an upper bound rather than a measurement. Both of
//! those are the kernel's account of itself. This file does not read either one
//! to decide whether the mesh is inside its deflection. It takes the triangles
//! out, samples across each one, and works out the distance to the surface with
//! the surface's own arithmetic.
//!
//! `the_cell_two_steps_close_off_fits_on_a_ball_and_is_eight_times_over_on_a_cone`
//! in `mesh.rs` is the other half, and it is the half that carries the numbers:
//! a ball's cell uses 98.1% of the deflection and a stretched cone's is charged
//! eight times over. That test also carries the argument for why a cell charged
//! eight times over cannot reach a mesh — `SolidView::mesh` is the only thing
//! that returns one, `triangulate` is its only feed, and `triangulate` returns
//! `Ok` only when every triangle it holds is inside the deflection. This file is
//! the same claim from the other end: not that the arithmetic must hold, but
//! that on real solids it does.
//!
//! # Where a sample lands
//!
//! Nine rows of barycentric weights across each triangle, so 55 points on it
//! including its three corners. The sag of a flat piece on any of these surfaces
//! is largest somewhere strictly inside, and a grid this fine finds it to well
//! within the margin every assert below leaves.

use rcad_geom::{Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Mesh, Model, SolidKey};

mod cones;
mod rings;

/// How far `point` really sits from `surface`, in the model's own unit.
///
/// The nearest point of every surface here is the one at the same two
/// coordinates the point projects to, so this is a projection and a distance —
/// **except on a cone**, where projecting drops straight in towards the axis
/// rather than perpendicular to the skin. The perpendicular distance is the
/// radial one over `√(1 + slope²)`, which is the same factor
/// `how_far_a_flat_piece_of_a_cone_strays` divides by one crate down.
fn how_far_off_the_surface(surface: &Surface, point: Point) -> f64 {
    let (u, v) = surface.coordinates_of(point);
    let straight_in = surface.point_at(u, v).distance_to(point);
    if let Surface::Cone(cone) = surface {
        return straight_in / cone.slope().hypot(1.0);
    }
    straight_in
}

/// The furthest any point of any triangle of `mesh` sits from the face it was
/// cut for.
fn how_far_inside_a_triangle_it_strays(model: &Model, solid: SolidKey, mesh: &Mesh) -> f64 {
    let surface_of: Vec<_> = model
        .solid(solid)
        .faces()
        .map(|face| (face.key(), face.surface()))
        .collect();
    let mut worst = 0.0_f64;
    for triangle in mesh.triangles() {
        let surface = surface_of
            .iter()
            .find(|(key, _)| *key == triangle.face())
            .map(|(_, surface)| surface)
            .expect("every triangle names a face of the solid it came from");
        if surface.as_plane().is_some() {
            continue;
        }
        let at = triangle
            .corners()
            .map(|corner| mesh.points()[corner as usize]);
        for one in 0..=ROWS_ACROSS_A_TRIANGLE {
            for other in 0..=(ROWS_ACROSS_A_TRIANGLE - one) {
                let weights = [
                    f64::from(ROWS_ACROSS_A_TRIANGLE - one - other),
                    f64::from(one),
                    f64::from(other),
                ]
                .map(|weight| weight / f64::from(ROWS_ACROSS_A_TRIANGLE));
                let mixed = (0..3)
                    .map(|which| (at[which] - Point::origin()) * weights[which])
                    .fold(Vector::zero(), |sum, part| sum + part);
                worst = worst.max(how_far_off_the_surface(surface, Point::origin() + mixed));
            }
        }
    }
    worst
}

/// How many rows of samples go across one triangle. See the head note.
const ROWS_ACROSS_A_TRIANGLE: u32 = 9;

/// The deflections every solid below is cut at, as a share of its own size.
const HOW_COARSE: [f64; 4] = [1e-3, 1e-2, 1e-1, 0.3];

/// One solid to measure, the model it lives in, the size the ladder above is
/// read against, and the least of its deflection it has to use.
///
/// **The floor is per shape and not one number over the family.** One over the
/// family is a floor on whichever shape happens to be worst and on nothing else:
/// the balls here use 0.44 and 0.45, so a single guard they satisfy would let
/// the stretched frustum — the shape this file was written for — slacken by
/// three and a half times with the headline number unmoved. Each shape carries
/// its own, set a little under what it measures, so a regression in any one of
/// them fails on that one's line.
struct Shape {
    what: String,
    model: Model,
    solid: SolidKey,
    size: f64,
    at_least: f64,
}

fn a_model(size: f64) -> Model {
    Model::new(Scale::millimetres(100.0 * size).expect("a working size"))
}

/// The curved solids this file measures.
///
/// **The stretched frustum is the one this file was written for.** A cone is
/// flat along its rulings, so the step up one comes back a single step however
/// long the frustum is, and the step round it is sized at the radius of the rim
/// it sits on. From two up to forty-two the top rim is twenty times as wide as
/// the bottom, which is the widest spread between the two ends of one face
/// anything here builds.
///
/// Every `at_least` is about seven eighths of what the shape measures today, so
/// ordinary wobble in the arithmetic does not trip it and a shape that stopped
/// being measured at all — an empty mesh, a face that stopped being curved, a
/// walk that started cutting three times as fine as it was asked — does.
fn the_family() -> Vec<Shape> {
    let mut family = Vec::new();

    for (radius, at_least) in [(1.0, 0.38), (7.0, 0.39)] {
        let mut model = a_model(radius);
        let ball = model
            .add_ball(radius, &Isometry::identity())
            .expect("a whole-number ball");
        family.push(Shape {
            what: format!("a ball of radius {radius}"),
            model,
            solid: ball,
            size: radius,
            at_least,
        });
    }

    let mut model = a_model(3.0);
    let bar = model
        .add_round_bar(1.0, 3.0, &Isometry::identity())
        .expect("a bar three tall");
    family.push(Shape {
        what: "a round bar of radius one and height three".to_owned(),
        model,
        solid: bar,
        size: 3.0,
        at_least: 0.16,
    });

    for (slope, near, far, at_least) in [
        (0.5, 1.0, 3.0, 0.27),
        (0.5, 2.0, 42.0, 0.24),
        (0.25, 2.0, 8.0, 0.25),
    ] {
        let mut model = a_model(far);
        let frustum = cones::a_frustum(&mut model, slope, near, far);
        family.push(Shape {
            what: format!("a frustum of slope {slope} from {near} to {far}"),
            model,
            solid: frustum,
            size: far,
            at_least,
        });
    }

    let mut model = a_model(4.0);
    let cone = cones::a_pointed_cone(&mut model, 0.5, 4.0);
    family.push(Shape {
        what: "a pointed cone of slope 0.5 up to 4".to_owned(),
        model,
        solid: cone,
        size: 4.0,
        at_least: 0.14,
    });

    let mut model = a_model(5.0);
    let bend = rings::a_quarter_bend(&mut model, 4.0, 1.0);
    family.push(Shape {
        what: "a quarter bend of ring 4 and tube 1".to_owned(),
        model,
        solid: bend,
        size: 5.0,
        at_least: 0.29,
    });

    family
}

/// No triangle of any of these meshes strays further from its own surface than
/// the deflection it was cut at.
///
/// This is the sentence a deflection is, checked where it is spent — inside a
/// triangle rather than at its corners. It would fail if a cell the walk hands
/// over ever reached the output uncut, which is what
/// `the_cell_two_steps_close_off_fits_on_a_ball_and_is_eight_times_over_on_a_cone`
/// shows a cone's cell can be charged for.
#[test]
fn no_triangle_of_a_curved_mesh_strays_further_than_the_deflection_it_was_cut_at() {
    for shape in the_family() {
        let mut worst_share = 0.0_f64;
        let mut worst_rung = 0.0_f64;
        for coarse in HOW_COARSE {
            let asked = coarse * shape.size;
            let mesh = shape
                .model
                .solid(shape.solid)
                .mesh(
                    Deflection::of(asked).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{} at {coarse} of its size: {why:?}", shape.what));
            let strays = how_far_inside_a_triangle_it_strays(&shape.model, shape.solid, &mesh);
            assert!(
                strays <= asked,
                "{} cut at {asked} strays {strays} inside a triangle",
                shape.what
            );
            if strays / asked > worst_share {
                worst_share = strays / asked;
                worst_rung = coarse;
            }
        }
        println!(
            "{:<44} uses {worst_share:.4} of its deflection, at {worst_rung} of its size",
            shape.what
        );
        assert!(
            worst_share > shape.at_least,
            "{} used {worst_share} of its deflection at every rung, under its floor of {}",
            shape.what,
            shape.at_least
        );
    }
}
