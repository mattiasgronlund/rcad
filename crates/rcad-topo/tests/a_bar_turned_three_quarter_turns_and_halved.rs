//! Probe for Q1504.

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

const RULES: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

#[test]
fn starts() {
    use rcad_geom::{
        Cylinder, EdgeRun, NamedRun, Plane, Point, Surface, where_a_closed_crossing_starts,
    };
    let wall = Surface::Cylinder(
        Cylinder::new(
            Axis::new(Point::new(0.0, 0.0, -2.0), Vector::new(0.0, 0.0, 1.0)).unwrap(),
            5.5,
        )
        .unwrap(),
    );
    for (z, n) in [(-2.0, -1.0), (0.5, 1.0)] {
        let cap = Surface::Plane(
            Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, n))
                .unwrap(),
        );
        let run = NamedRun::new(
            EdgeRun::RightRoundWhereItsFacesCross,
            [rcad_predicates::Point::from([5.5, 0.0, z]); 2],
        )
        .unwrap();
        for along in [[&cap, &wall], [&wall, &cap]] {
            println!(
                "cap z = {z}: starts at {:?}",
                where_a_closed_crossing_starts(&run, along, &mut ProvenanceLog::new())
            );
        }
    }
    panic!("starts");
}

#[test]
fn probe() {
    for (name, quarters, low, high) in [
        ("turned, y>=0", 3, [-6.0, 0.0, -3.0], [6.0, 6.0, 1.0]),
        ("unturned, y>=0", 0, [-6.0, 0.0, -3.0], [6.0, 6.0, 1.0]),
        ("unturned, x<=0", 0, [-6.0, -6.0, -3.0], [0.0, 6.0, 1.0]),
        ("unturned, x>=0", 0, [0.0, -6.0, -3.0], [6.0, 6.0, 1.0]),
        ("one turn, y>=0", 1, [-6.0, 0.0, -3.0], [6.0, 6.0, 1.0]),
        ("two turns, y>=0", 2, [-6.0, 0.0, -3.0], [6.0, 6.0, 1.0]),
    ] {
        for tolerant in [false, true] {
            for rule in RULES {
                for bar_first in [true, false] {
                    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
                    let policy = if tolerant {
                        Policy::tolerant(model.scale().nominal_size())
                    } else {
                        Policy::certified()
                    };
                    let turn = Isometry::quarter_turns(&Axis::z(), quarters).expect("about z");
                    let lift =
                        Isometry::translation(Vector::new(0.0, 0.0, -2.0)).expect("a finite shift");
                    let bar = model
                        .add_round_bar(5.5, 2.5, &lift.compose(&turn))
                        .expect("a bar");
                    let block = a_block(&mut model, low, high);
                    let (first, second) = if bar_first {
                        (bar, block)
                    } else {
                        (block, bar)
                    };
                    let mut log = ProvenanceLog::new();
                    let answer = model.fuse(&[first], &[second], rule, &policy, &mut log);
                    match answer {
                        Ok(fused) => {
                            let solids = fused.solids().to_vec();
                            let total: Vec<(f64, f64)> = solids
                                .iter()
                                .map(|&s| {
                                    let m = model
                                        .solid(s)
                                        .unwrap()
                                        .measure(
                                            Deflection::for_scale(model.scale()),
                                            &Policy::certified(),
                                            &mut ProvenanceLog::new(),
                                        )
                                        .unwrap();
                                    (m.volume(), m.volume_error())
                                })
                                .collect();
                            println!(
                                "{name} tolerant {tolerant} {rule:?} bar first {bar_first}: {total:?} log {}",
                                log.records().len()
                            );
                        }
                        Err(why) => println!(
                            "{name} tolerant {tolerant} {rule:?} bar first {bar_first}: {why:?}"
                        ),
                    }
                }
            }
        }
    }
    panic!("probe");
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
