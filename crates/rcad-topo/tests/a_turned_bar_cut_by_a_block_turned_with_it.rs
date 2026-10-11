//! Probe.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Surface, Unit, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

fn turns() -> Vec<(&'static str, Isometry)> {
    let diagonal = Axis::new(Point::origin(), Vector::new(1.0, 1.0, 1.0)).expect("a finite axis");
    vec![
        ("none", Isometry::identity()),
        (
            "half about y",
            Isometry::rotation(&Axis::y(), std::f64::consts::PI).expect("a turn"),
        ),
        (
            "exact half about y",
            Isometry::quarter_turns(&Axis::y(), 2).expect("a turn"),
        ),
        (
            "one radian about 111",
            Isometry::rotation(&diagonal, 1.0).expect("a turn"),
        ),
    ]
}

#[test]
fn probe() {
    let tilt_axis = Axis::new(Point::new(3.0, 0.0, 2.0), Vector::new(1.0, 2.0, 0.5)).unwrap();
    let tilt = Isometry::rotation(&tilt_axis, 0.3).unwrap();
    let blocks: Vec<(&str, [f64; 3], Vector, Isometry)> = vec![
        (
            "seam notch",
            [2.0, 2.0, 2.0],
            Vector::new(2.0, -1.0, 1.0),
            Isometry::identity(),
        ),
        (
            "wall notch",
            [2.0, 2.0, 2.0],
            Vector::new(-1.0, 2.0, 1.0),
            Isometry::identity(),
        ),
        (
            "slab, sides outside",
            [8.0, 8.0, 2.0],
            Vector::new(-4.0, -4.0, 1.0),
            Isometry::identity(),
        ),
        (
            "top plug, sides inside",
            [2.0, 2.0, 2.0],
            Vector::new(-1.0, -1.0, 4.0),
            Isometry::identity(),
        ),
        (
            "seam notch tilted",
            [2.0, 2.0, 2.0],
            Vector::new(2.0, -1.0, 1.0),
            tilt,
        ),
    ];
    for (name, turn) in turns() {
        for (block_name, size, at, extra) in &blocks {
            let mut model = Model::new(Scale::new(Unit::Millimetre, 5.0).expect("a working size"));
            let bar = model
                .add_round_bar(3.0, 5.0, &Isometry::identity())
                .expect("a bar");
            model.place_solid(bar, &turn).expect("turned");
            let block = a_block(&mut model, *size, *at);
            model
                .place_solid(block, &turn.compose(extra))
                .expect("turned");
            let pipe = model
                .solid(bar)
                .unwrap()
                .faces()
                .find_map(|f| match f.surface() {
                    Surface::Cylinder(c) => Some(c),
                    _ => None,
                })
                .unwrap();
            let leans: Vec<String> = model
                .solid(block)
                .unwrap()
                .faces()
                .map(|f| match f.surface() {
                    Surface::Plane(p) => format!(
                        "{:?}",
                        rcad_geom::how_a_plane_leans_against_a_pipe(&p, &pipe)
                    ),
                    other => format!("{other:?}"),
                })
                .collect();
            println!("{name} / {block_name}: leans {leans:?}");
            for tolerant in [false, true] {
                let rule = Rule::Difference;
                let mut model = model.clone();
                let policy = if tolerant {
                    Policy::tolerant(model.scale().nominal_size())
                } else {
                    Policy::certified()
                };
                let mut log = ProvenanceLog::new();
                let fused = model.fuse(&[bar], &[block], rule, &policy, &mut log);
                match fused {
                    Ok(fused) => {
                        let volumes: Vec<f64> = fused
                            .solids()
                            .iter()
                            .map(|&s| {
                                model
                                    .solid(s)
                                    .unwrap()
                                    .measure(
                                        Deflection::of(0.01).unwrap(),
                                        &Policy::certified(),
                                        &mut ProvenanceLog::new(),
                                    )
                                    .map(|m| m.volume())
                                    .unwrap_or(f64::NAN)
                            })
                            .collect();
                        println!(
                            "  tolerant {tolerant}: OK {volumes:?} log {}",
                            log.records().len()
                        );
                    }
                    Err(Error::Undecidable { records }) => {
                        println!("  tolerant {tolerant}: UNDECIDABLE");
                        for r in records.records() {
                            println!(
                                "    {} {:?} {:?} {:?}",
                                r.predicate(),
                                r.inputs(),
                                r.level(),
                                r.outcome()
                            );
                        }
                    }
                    Err(other) => {
                        println!("  tolerant {tolerant}: ERR {other:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn probe_start() {
    let diagonal = Axis::new(Point::origin(), Vector::new(1.0, 1.0, 1.0)).expect("a finite axis");
    let turn = Isometry::rotation(&diagonal, 1.0).unwrap();
    let mut model = Model::new(Scale::new(Unit::Millimetre, 5.0).expect("a working size"));
    let bar = model
        .add_round_bar(3.0, 5.0, &Isometry::identity())
        .unwrap();
    model.place_solid(bar, &turn).unwrap();
    let block = a_block(&mut model, [2.0, 2.0, 2.0], Vector::new(2.0, -1.0, 1.0));
    model.place_solid(block, &turn).unwrap();
    let pipe = model
        .solid(bar)
        .unwrap()
        .faces()
        .map(|f| f.surface())
        .find(|s| matches!(s, Surface::Cylinder(_)))
        .unwrap();
    let back = turn.inverse();
    for f in model.solid(block).unwrap().faces() {
        let flat = f.surface();
        let Surface::Plane(p) = flat else { panic!() };
        let normal_back = back.apply_to_vector(p.normal());
        let meet = rcad_geom::intersect(
            &flat,
            &pipe,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        match meet {
            Ok(rcad_geom::SurfaceMeet::Curves(runs)) => {
                for run in &runs {
                    let start = rcad_geom::where_a_closed_crossing_starts(
                        run,
                        [&flat, &pipe],
                        &mut ProvenanceLog::new(),
                    );
                    println!(
                        "side with normal {normal_back:?} offset {}: {:?} start {:?}",
                        p.offset(),
                        run.run(),
                        start.map(|s| (s, back.apply_to_point(s)))
                    );
                }
            }
            other => println!("side with normal {normal_back:?}: {other:?}"),
        }
    }
}

fn a_block(model: &mut Model, [x, y, z]: [f64; 3], at: Vector) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [x, 0.0], [x, y], [0.0, y]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            z,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("a block")
}
