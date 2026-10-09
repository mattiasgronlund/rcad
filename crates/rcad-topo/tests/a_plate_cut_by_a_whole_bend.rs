//! PROBE (Q1305): a half disk plate, a whole ring and a bar on its axis.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

const RING: f64 = 4.0;
const TUBE: f64 = 1.0;
const PLATE: f64 = 5.5;
const PLATE_TOP: f64 = 0.5;
const PLATE_BOTTOM: f64 = -2.0;
const BAR: f64 = 23.0 / 8.0;
const BAR_BOTTOM: f64 = 0.25;
const BAR_TOP: f64 = 0.75;

#[test]
fn probe() {
    for which in 0..3 {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for plate_first in [true, false] {
                let mut model = a_model();
                let [plate, ring, bar] = the_three(&mut model);
                let (first, second): (Vec<SolidKey>, Vec<SolidKey>) = match which {
                    0 => (vec![plate], vec![ring]),
                    1 => (vec![bar], vec![ring]),
                    _ => (vec![plate], vec![bar]),
                };
                let (first, second) = if plate_first {
                    (first, second)
                } else {
                    (second, first)
                };
                print!("{which} ");
                let answer = model
                    .fuse(
                        &first,
                        &second,
                        rule,
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .map(|fused| fused.solids().to_vec());
                match answer {
                    Ok(solids) => {
                        let total: f64 = solids.iter().map(|&s| volume_of(&model, s)).sum();
                        println!(
                            "{rule:?} plate first {plate_first}: {} solids, {total}",
                            solids.len()
                        );
                    }
                    Err(why) => println!("{rule:?} plate first {plate_first}: {why:?}"),
                }
            }
        }
    }
    let pi = PI;
    let plate = pi * PLATE * PLATE / 2.0 * (PLATE_TOP - PLATE_BOTTOM);
    let ring = 8.0 * pi * pi;
    let bar = pi * BAR * BAR * (BAR_TOP - BAR_BOTTOM);
    let plate_and_ring = 8.0 * pi * pi / 3.0 + pi * 3.0_f64.sqrt();
    let plate_and_bar = pi * BAR * BAR / 2.0 * (PLATE_TOP - BAR_BOTTOM);
    let both = plate_and_ring + plate_and_bar;
    println!(
        "union {} intersection {} plate less {} others less {}",
        plate + ring + bar - both,
        both,
        plate - both,
        ring + bar - both
    );
    panic!("probe");
}

fn volume_of(model: &Model, solid: SolidKey) -> f64 {
    model
        .solid(solid)
        .expect("the fuse made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|measured| measured.volume())
        .unwrap_or(f64::NAN)
}

fn the_three(model: &mut Model) -> [SolidKey; 3] {
    let plate = the_half_disk_plate(model);
    let ring = a_whole_ring_cut_along_its_equators(model);
    let bar = a_bar_on_the_axis(model, BAR, BAR_BOTTOM, BAR_TOP);
    [plate, ring, bar]
}

fn the_half_disk_plate(model: &mut Model) -> SolidKey {
    let disk = a_bar_on_the_axis(model, PLATE, PLATE_BOTTOM, PLATE_TOP);
    let block = a_block(model, [0.0, -6.0, -3.0], [6.0, 6.0, 1.0]);
    let fused = model
        .fuse(
            &[disk],
            &[block],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar and a block on its axis");
    let &[plate] = fused.solids() else {
        panic!("{fused:?}");
    };
    plate
}

fn a_bar_on_the_axis(model: &mut Model, radius: f64, bottom: f64, top: f64) -> SolidKey {
    let lift = Isometry::translation(Vector::new(0.0, 0.0, bottom)).expect("a finite shift");
    model
        .add_round_bar(radius, top - bottom, &lift)
        .expect("a bar")
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a working size"))
}

fn a_block(model: &mut Model, low: [f64; 3], high: [f64; 3]) -> SolidKey {
    let [x0, y0, z0] = low;
    let [x1, y1, z1] = high;
    let square = ProfileBuilder::new(vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    model
        .add_extrusion(
            &square,
            z1 - z0,
            &Isometry::translation(Vector::new(0.0, 0.0, z0)).expect("a finite shift"),
        )
        .expect("a block")
}

fn a_whole_ring_cut_along_its_equators(model: &mut Model) -> SolidKey {
    let cut = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let corners = [
        Point::new(-(RING + TUBE), 0.0, 0.0),
        Point::new(-(RING - TUBE), 0.0, 0.0),
    ];
    let skin = Surface::Torus(Torus::new(Axis::z(), RING, TUBE).expect("a tube narrower"));
    let mut builder = SolidBuilder::new();
    let [one, other] = corners.map(|corner| {
        let corner = builder
            .add_vertex(corner)
            .expect("a corner on the ring and on the plane");
        builder
            .add_edge(
                corner,
                corner,
                EdgeRun::RightRoundWhereItCrosses {
                    and: Surface::Plane(cut),
                },
            )
            .expect("a rim is a whole closed curve, so one corner twice is an edge")
    });
    let face = |builder: &mut SolidBuilder, walks: [(_, Orientation); 2]| {
        let wires = walks.map(|(edge, turn)| {
            let coedge = builder
                .add_coedge(edge, turn)
                .expect("an edge of this build");
            builder
                .add_wire(&[DraftStep::Along(coedge)])
                .expect("one closed edge is a wire")
        });
        builder
            .add_face(skin, &wires)
            .expect("a band between two circles")
    };
    let first = face(
        &mut builder,
        [(one, Orientation::Reversed), (other, Orientation::Forward)],
    );
    let second = face(
        &mut builder,
        [(one, Orientation::Forward), (other, Orientation::Reversed)],
    );
    builder
        .add_shell(&[
            (first, Orientation::Forward),
            (second, Orientation::Forward),
        ])
        .expect("two halves close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a whole ring in two faces is a solid")
}
