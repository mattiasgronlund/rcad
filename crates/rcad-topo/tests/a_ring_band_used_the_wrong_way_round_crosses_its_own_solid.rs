//! Q2017: a second public-API route to a self-intersecting solid, built face
//! by face through `SolidBuilder` rather than through `Import::read`.
//!
//! §17.4 keeps a self-intersecting solid out of `Model::validate`, for cost:
//! "the decomposition already computes every face-pair meet, so a meet
//! between two faces of the same solid that is not along their shared edge
//! is free to notice. Putting it in `validate` would make validation
//! quadratic in face count." §37.1's Q1341 note found the first such route,
//! through `Import::read` of a written model; this is the second.

use rcad_geom::{Axis, Cylinder, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Torus, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn flat_at(z: f64, facing: f64) -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, facing))
            .expect("a unit normal"),
    )
}

fn at_height(z: f64) -> Isometry {
    Isometry::translation(Vector::new(0.0, 0.0, z)).expect("a finite shift")
}

/// A copy of `a_solid_of_revolution` in
/// `a_piece_no_ray_can_place_names_its_refusal.rs` that takes each face's
/// use (its shell orientation) directly, rather than always
/// `Orientation::Forward`.
fn a_solid_of_revolution_with_each_faces_use(
    model: &mut Model,
    corners: &[(f64, f64)],
    surfaces: &[Surface],
    turns: &[Orientation],
    uses: &[Orientation],
) -> Result<SolidKey, Error> {
    let mut builder = SolidBuilder::new();
    let mut circles = Vec::with_capacity(corners.len());
    for (index, &(rho, z)) in corners.iter().enumerate() {
        let corner = builder.add_vertex(Point::new(rho, 0.0, z))?;
        let by_a_ring = matches!(surfaces[index], Surface::Torus(_))
            || matches!(surfaces[index + 1], Surface::Torus(_));
        let run = if by_a_ring {
            EdgeRun::RightRoundWhereItCrosses {
                and: flat_at(z, 1.0),
            }
        } else {
            EdgeRun::RightRoundWhereItsFacesCross
        };
        circles.push(builder.add_edge(corner, corner, run)?);
    }
    let mut shell = Vec::with_capacity(surfaces.len());
    for (index, &surface) in surfaces.iter().enumerate() {
        let mut wires = Vec::with_capacity(2);
        if index > 0 {
            let coedge = builder.add_coedge(circles[index - 1], turns[index - 1])?;
            wires.push(builder.add_wire(&[DraftStep::Along(coedge)])?);
        }
        if index < circles.len() {
            let coedge = builder.add_coedge(circles[index], turns[index].reversed())?;
            wires.push(builder.add_wire(&[DraftStep::Along(coedge)])?);
        }
        shell.push((builder.add_face(surface, &wires)?, uses[index]));
    }
    builder.add_shell(&shell)?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

const RING: f64 = 4.0;
const TUBE: f64 = 1.0;
const Z_TOP: f64 = 0.95;
const STEP: f64 = 1.2e-16;
const DISC_RADIUS: f64 = 10.0;
const Z_BOTTOM: f64 = -0.05;

/// Where the ring's outer wall (`rho = RING + sqrt(TUBE^2 - z^2)`) crosses a
/// given height.
fn outer_wall_rho(z: f64) -> f64 {
    RING + TUBE.mul_add(TUBE, -(z * z)).sqrt()
}

/// A disc of radius 10 from `z = -0.05` to `0.95`, stepped up by `STEP`
/// outside where the outer wall of a ring of 4 round a tube of 1 crosses
/// `z = 0.95`, with the ring band between the step and the unstepped flat
/// top given `Orientation::Forward` in the shell where `Reversed` is meant.
fn a_ring_band_used_the_wrong_way_round(model: &mut Model) -> Result<SolidKey, Error> {
    let z_top2 = Z_TOP + STEP;
    let rho_top = outer_wall_rho(Z_TOP);
    let rho_top2 = outer_wall_rho(z_top2);
    let ring = Surface::Torus(Torus::new(Axis::z(), RING, TUBE).expect("a ring"));
    let wall = Surface::Cylinder(Cylinder::new(Axis::z(), DISC_RADIUS).expect("a cylinder"));
    a_solid_of_revolution_with_each_faces_use(
        model,
        &[
            (DISC_RADIUS, Z_BOTTOM),
            (DISC_RADIUS, z_top2),
            (rho_top2, z_top2),
            (rho_top, Z_TOP),
        ],
        &[flat_at(Z_BOTTOM, -1.0), wall, flat_at(z_top2, 1.0), ring, flat_at(Z_TOP, 1.0)],
        &[
            Orientation::Forward,
            Orientation::Forward,
            Orientation::Reversed,
            Orientation::Reversed,
        ],
        &[
            Orientation::Forward,
            Orientation::Forward,
            Orientation::Forward,
            Orientation::Forward,
            Orientation::Forward,
        ],
    )
}

#[test]
fn probe_builds_and_measures() {
    let mut model = a_model();
    let solid = a_ring_band_used_the_wrong_way_round(&mut model).expect("it builds");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("it validates");
    let deflection = rcad_geom::Deflection::of(0.01).expect("a deflection");
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(deflection, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("it measures");
    println!(
        "PROBE volume {} +- {}",
        measured.volume(),
        measured.volume_error()
    );
}
