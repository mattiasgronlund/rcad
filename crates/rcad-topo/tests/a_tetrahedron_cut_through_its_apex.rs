//! Q1839 probe, not committed.
use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftEdge, DraftStep, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

#[test]
fn q1839_probe_the_tetrahedron_cut_through_its_apex() {
    let shift = Isometry::translation(Vector::new(7.0, -5.0, 2.5)).expect("a plain translation");
    let turn = Isometry::rotation(&Axis::x(), 0.4).expect("a turn about x");
    let placements = [
        ("none", Isometry::identity()),
        ("shift", shift),
        ("turn", turn),
        ("turn then shift", shift.compose(&turn)),
    ];
    for (name, placement) in placements {
        for rule in [Rule::Difference, Rule::Intersection] {
            for (policy_name, policy) in [
                ("certified", Policy::certified()),
                ("tolerant(100)", Policy::tolerant(100.0)),
            ] {
                let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm"));
                let tetrahedron = a_tetrahedron(&mut model);
                model
                    .place_solid(tetrahedron, &placement)
                    .expect("this model made the key");
                let block = a_block(&mut model, &placement);
                let mut log = ProvenanceLog::new();
                match model.fuse(&[tetrahedron], &[block], rule, &policy, &mut log) {
                    Ok(answer) => {
                        let volume: f64 = answer
                            .solids()
                            .iter()
                            .map(|&solid| {
                                model
                                    .solid(solid)
                                    .expect("a key")
                                    .measure(
                                        Deflection::of(0.01).expect("a deflection"),
                                        &Policy::certified(),
                                        &mut ProvenanceLog::new(),
                                    )
                                    .expect("a measure")
                                    .volume()
                            })
                            .sum();
                        println!(
                            "Q1839 {name} {rule:?} {policy_name}: Ok {} solids volume {volume} records {}",
                            answer.solids().len(),
                            log.records().len()
                        );
                    }
                    Err(error) => println!("Q1839 {name} {rule:?} {policy_name}: Err {error:?}"),
                }
            }
        }
    }
}

#[test]
fn q1839_probe_what_each_answer_is() {
    let shift = Isometry::translation(Vector::new(7.0, -5.0, 2.5)).expect("a plain translation");
    let turn = Isometry::rotation(&Axis::x(), 0.4).expect("a turn about x");
    let placements = [
        ("none", Isometry::identity()),
        ("shift", shift),
        ("turn", turn),
        ("turn then shift", shift.compose(&turn)),
    ];
    let policy = Policy::certified().with_dynamic_range_check(rcad_predicates::Severity::Warn);
    for (name, placement) in placements {
        for rule in [Rule::Difference, Rule::Intersection] {
            let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm"));
            let tetrahedron = a_tetrahedron(&mut model);
            model
                .place_solid(tetrahedron, &placement)
                .expect("this model made the key");
            let block = a_block(&mut model, &placement);
            let mut log = ProvenanceLog::new();
            match model.fuse(&[tetrahedron], &[block], rule, &policy, &mut log) {
                Ok(answer) => {
                    println!(
                        "Q1839W {name} {rule:?}: Ok, records {}",
                        log.records().len()
                    );
                    for &solid in answer.solids() {
                        for face in model.solid(solid).expect("a key").faces() {
                            println!("Q1839W   face {:?}", face.plane().coefficients());
                            for wire in face.wires() {
                                for coedge in wire.coedges() {
                                    let edge = coedge.edge();
                                    println!(
                                        "Q1839W     {:?} -> {:?} len {:e}",
                                        coedge.start().point().coordinates(),
                                        coedge.end().point().coordinates(),
                                        edge.distance_between_its_ends()
                                    );
                                }
                            }
                        }
                    }
                }
                Err(error) => println!("Q1839W {name} {rule:?}: Err {error:?}"),
            }
        }
    }
}

fn a_block(model: &mut Model, placement: &Isometry) -> SolidKey {
    let footprint = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 5.0], [0.0, 5.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let own = Isometry::translation(Vector::new(-5.0, 0.0, -1.0)).expect("a plain translation");
    model
        .add_extrusion(&footprint, 12.0, &placement.compose(&own))
        .expect("a block")
}

fn a_tetrahedron(model: &mut Model) -> SolidKey {
    let points = [
        [0.0, 0.0, 10.0],
        [-3.0, -3.0, 0.0],
        [3.0, -3.0, 0.0],
        [0.0, 3.0, 0.0],
    ]
    .map(|[x, y, z]| Point::new(x, y, z));
    let faces: [&[usize]; 4] = [&[0, 1, 2], &[0, 2, 3], &[0, 3, 1], &[1, 3, 2]];

    let mut builder = SolidBuilder::new();
    let vertices = points.map(|point| builder.add_vertex(point).expect("a finite corner"));
    let mut edges: Vec<([usize; 2], DraftEdge)> = Vec::new();
    let drafted = faces.map(|face| {
        let steps: Vec<DraftStep> = (0..face.len())
            .map(|position| {
                let (from, to) = (face[position], face[(position + 1) % face.len()]);
                let (edge, orientation) = match edges.iter().find(|(ends, _)| *ends == [to, from]) {
                    Some(&(_, edge)) => (edge, Orientation::Reversed),
                    None => {
                        let edge = builder
                            .add_edge(vertices[from], vertices[to], EdgeRun::Straight)
                            .expect("a side between two corners");
                        edges.push(([from, to], edge));
                        (edge, Orientation::Forward)
                    }
                };
                DraftStep::Along(
                    builder
                        .add_coedge(edge, orientation)
                        .expect("one use of a side"),
                )
            })
            .collect();
        let wire = builder.add_wire(&steps).expect("a closed boundary");
        let plane = Plane::through_three_points(points[face[0]], points[face[1]], points[face[2]])
            .expect("three corners of a face not in a line");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a face of a tetrahedron")
    });
    builder
        .add_shell(&drafted.map(|face| (face, Orientation::Forward)))
        .expect("four faces close a tetrahedron");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a tetrahedron is a sound solid")
}
