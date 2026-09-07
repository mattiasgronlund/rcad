//! Probe, not yet a test.

mod cones;

use cones::{a_frustum, a_pointed_cone};
use rcad_geom::{Deflection, Point, Scale, Surface};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, SolidKey};

#[test]
fn probe() {
    for near in [1.0, 1e-3, 1e-6, 1e-10, 1e-14, 1e-15] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let solid = a_frustum(&mut model, 0.5, near, 8.0);
        report(&model, solid, &format!("frustum near={near}"));
    }
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let solid = a_pointed_cone(&mut model, 0.5, 8.0);
    report(&model, solid, "pointed cone");
}

fn report(model: &Model, solid: SolidKey, label: &str) {
    let apex = model
        .solid(solid)
        .faces()
        .find_map(|face| match face.surface() {
            Surface::Cone(cone) => Some(cone.apex()),
            _ => None,
        })
        .expect("a cone face");
    let mesh = model.solid(solid).mesh(
        Deflection::of(0.01).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    match mesh {
        Err(error) => println!("{label}: apex {:?}, mesh refused: {error:?}", apex.coordinates()),
        Ok(mesh) => {
            let mut distances: Vec<f64> = mesh
                .points()
                .iter()
                .map(|point| point.distance_to(apex))
                .collect();
            distances.sort_by(f64::total_cmp);
            println!(
                "{label}: apex {:?}, deviation {:?}, {} points, nearest {:?}",
                apex.coordinates(),
                mesh.deviation(),
                mesh.points().len(),
                &distances[..distances.len().min(4)]
            );
        }
    }
    let _ = Point::origin();
}
