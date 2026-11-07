//! Probe, not kept as it stands.

use rcad_geom::{Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog, side_of_a_plane_where_three_planes_meet};
use rcad_topo::{DraftStep, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

#[test]
fn probe_q2921_t_junction() {
    two_blocks(&mut a_model());
    let slopes: [(f64, f64); 10] = [
        (2.0, -1.0),
        (3.0, -1.0),
        (3.0, -2.0),
        (1.0, -2.0),
        (1.0, -3.0),
        (2.0, -3.0),
        (5.0, -2.0),
        (5.0, -3.0),
        (4.0, -3.0),
        (3.0, -4.0),
    ];
    for (dx, dy) in slopes {
        for side in 0..4 {
            let k = 0.5;
            let rising = side % 2 == 1;
            let (ex, ey) = if rising { (dx, -dy) } else { (dx, dy) };
            let p1 = [3.0 + ex * k, 3.0 + ey * k];
            let p3 = [3.0 - ex * k, 3.0 - ey * k];
            let outline = match side {
                0 => [p1, [p1[0], p3[1]], p3],
                1 => [p3, [p1[0], p3[1]], p1],
                2 => [p3, [p3[0], p1[1]], p1],
                _ => [p3, p1, [p3[0], p1[1]]],
            };
            eprintln!("PROBE side {side} outline {outline:?}");
            run_with(two_built_blocks, dx, dy, k, rising, outline);
        }
    }
}

fn two_built_blocks(model: &mut Model) -> SolidKey {
    let mut corners = Vec::new();
    let mut faces: Vec<Vec<usize>> = Vec::new();
    for ([x0, x1], [y0, y1], [z0, z1]) in [
        ([0.0, 3.0], [0.0, 3.0], [-1.0, 2.0]),
        ([3.0, 6.0], [3.0, 6.0], [0.0, 1.0]),
    ] {
        let base = corners.len();
        for k in [z0, z1] {
            for j in [y0, y1] {
                for i in [x0, x1] {
                    corners.push([i, j, k]);
                }
            }
        }
        for face in [
            [0, 2, 3, 1],
            [4, 5, 7, 6],
            [0, 4, 6, 2],
            [1, 3, 7, 5],
            [0, 1, 5, 4],
            [2, 6, 7, 3],
        ] {
            faces.push(face.iter().map(|at| base + at).collect());
        }
    }
    let faces: Vec<&[usize]> = faces.iter().map(Vec::as_slice).collect();
    let solid = a_polyhedron(model, &corners, &faces);
    eprintln!(
        "PROBE two built blocks validate {:?}",
        model.validate(&Policy::certified(), &mut ProvenanceLog::new())
    );
    solid
}

fn a_polyhedron(model: &mut Model, corners: &[[f64; 3]], faces: &[&[usize]]) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let vertices: Vec<_> = corners
        .iter()
        .map(|&[x, y, z]| builder.add_vertex(Point::new(x, y, z)).expect("a corner"))
        .collect();
    let mut edges = std::collections::BTreeMap::new();
    let mut drafted = Vec::new();
    for face in faces {
        let mut steps = Vec::new();
        for (at, &from) in face.iter().enumerate() {
            let to = face[(at + 1) % face.len()];
            let ends = (from.min(to), from.max(to));
            let edge = *edges.entry(ends).or_insert_with(|| {
                builder
                    .add_edge(vertices[ends.0], vertices[ends.1], EdgeRun::Straight)
                    .expect("an edge")
            });
            let way = if from < to {
                Orientation::Forward
            } else {
                Orientation::Reversed
            };
            steps.push(DraftStep::Along(
                builder.add_coedge(edge, way).expect("a coedge"),
            ));
        }
        let wire = builder.add_wire(&steps).expect("a wire");
        let [a, b, c] = [face[0], face[1], face[2]].map(|index| corners[index]);
        let along = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let next = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
        let outward = Vector::new(
            along[1] * next[2] - along[2] * next[1],
            along[2] * next[0] - along[0] * next[2],
            along[0] * next[1] - along[1] * next[0],
        );
        let plane = Plane::from_point_and_normal(Point::new(a[0], a[1], a[2]), outward)
            .expect("three corners not in a line");
        drafted.push(
            builder
                .add_face(Surface::Plane(plane), &[wire])
                .expect("a face"),
        );
    }
    let shell: Vec<_> = drafted
        .into_iter()
        .map(|face| (face, Orientation::Forward))
        .collect();
    builder.add_shell(&shell).expect("the faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a polyhedron")
}

fn two_blocks(model: &mut Model) -> SolidKey {
    let a = a_prism(
        model,
        vec![[0.0, 0.0], [3.0, 0.0], [3.0, 3.0], [0.0, 3.0]],
        [0.0, 0.0, -1.0],
        3.0,
    );
    let b = a_prism(
        model,
        vec![[3.0, 3.0], [6.0, 3.0], [6.0, 6.0], [3.0, 6.0]],
        [0.0, 0.0, 0.0],
        1.0,
    );
    let fused = model.fuse(
        &[a],
        &[b],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    eprintln!(
        "PROBE two fused blocks {:?}",
        fused.map(|f| f.solids().len())
    );
    a
}

#[test]
fn probe_q2921() {
    let slopes: [(f64, f64); 10] = [
        (2.0, -1.0),
        (3.0, -1.0),
        (3.0, -2.0),
        (1.0, -2.0),
        (1.0, -3.0),
        (2.0, -3.0),
        (5.0, -2.0),
        (5.0, -3.0),
        (4.0, -3.0),
        (3.0, -4.0),
    ];
    for (dx, dy) in &slopes[..1] {
        let (dx, dy) = (*dx, *dy);
        for k in [0.5] {
            for upper_left in [false] {
                // upper_left: W along (dx, -dy) through Q1 and Q3, T to its lower right.
                let (ex, ey) = if upper_left { (dx, -dy) } else { (dx, dy) };
                let p1 = [3.0 + ex * k, 3.0 + ey * k];
                let p3 = [3.0 - ex * k, 3.0 - ey * k];
                let outline = if upper_left {
                    // T below-right of the line from p3 (lower left) to p1 (upper right).
                    [p3, [p1[0], p3[1]], p1]
                } else {
                    // T above-right of the line from p1 (lower right) to p3 (upper left).
                    [p1, [p1[0], p3[1]], p3]
                };
                run_one(dx, dy, k, upper_left, outline);
            }
        }
    }
}

fn run_one(dx: f64, dy: f64, k: f64, upper_left: bool, outline: [[f64; 2]; 3]) {
    run_with(plate_less_two_squares, dx, dy, k, upper_left, outline);
}

fn plate_less_two_squares(model: &mut Model) -> SolidKey {
    let plate = a_prism(
        model,
        vec![[0.0, 0.0], [6.0, 0.0], [6.0, 6.0], [0.0, 6.0]],
        [0.0, 0.0, 0.0],
        1.0,
    );
    let holes = [
        a_prism(
            model,
            vec![[3.0, 2.0], [4.0, 2.0], [4.0, 3.0], [3.0, 3.0]],
            [0.0, 0.0, -1.0],
            3.0,
        ),
        a_prism(
            model,
            vec![[2.0, 3.0], [3.0, 3.0], [3.0, 4.0], [2.0, 4.0]],
            [0.0, 0.0, -1.0],
            3.0,
        ),
    ];
    let fused = model
        .fuse(
            &[plate],
            &holes,
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the plate less two squares");
    let &[solid] = fused.solids() else {
        panic!("not one solid");
    };
    solid
}

fn run_with(
    make: fn(&mut Model) -> SolidKey,
    dx: f64,
    dy: f64,
    k: f64,
    upper_left: bool,
    outline: [[f64; 2]; 3],
) {
    let mut model = a_model();
    let solid = make(&mut model);
    let tool = a_prism(&mut model, outline.to_vec(), [0.0, 0.0, 0.5], 1.0);
    let wall = model
        .solid(tool)
        .expect("made")
        .faces()
        .map(|face| face.plane().coefficients())
        .find(|[a, b, c, _]| *c == 0.0 && *a != 0.0 && *b != 0.0)
        .expect("a leaning wall");
    let c_side = side_of_a_plane_where_three_planes_meet(
        [
            [1.0, 0.0, 0.0, -3.0],
            [0.0, 1.0, 0.0, -3.0],
            [0.0, 0.0, 1.0, -0.5],
        ],
        wall,
    );
    let s_volume = volume(&model, &[solid]);
    let t_volume = volume(&model, &[tool]);
    let mut line = format!(
        "slope ({dx},{dy}) k {k} upper_left {upper_left} wall {wall:?} c {c_side:?} S {s_volume:?} T {t_volume:?}:"
    );
    for (one, other, rule) in [
        (solid, tool, Rule::Union),
        (solid, tool, Rule::Intersection),
        (solid, tool, Rule::Difference),
        (tool, solid, Rule::Union),
        (tool, solid, Rule::Intersection),
        (tool, solid, Rule::Difference),
    ] {
        eprintln!("PROBE fuse {rule:?} {}", one == solid);
        let mut copy = model.clone();
        let answer = copy.fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let said = match answer {
            Ok(fused) => {
                let solids = fused.solids().to_vec();
                let valid = copy.validate(&Policy::certified(), &mut ProvenanceLog::new());
                format!(
                    "{} solids {:?} valid {}",
                    solids.len(),
                    volume(&copy, &solids),
                    valid.is_ok()
                )
            }
            Err(why) => format!("refused {why:?}"),
        };
        line.push_str(&format!(
            " | {rule:?} {} {said}",
            if one == solid { "S,T" } else { "T,S" }
        ));
    }
    eprintln!("PROBE {line}");
}

fn volume(model: &Model, solids: &[SolidKey]) -> (f64, f64) {
    let (mut v, mut e) = (0.0, 0.0);
    for &solid in solids {
        let m = model
            .solid(solid)
            .expect("made")
            .measure(
                Deflection::of(0.1).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("measures");
        v += m.volume();
        e += m.volume_error();
    }
    (v, e)
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn a_prism(
    model: &mut Model,
    outline: Vec<[f64; 2]>,
    [x, y, z]: [f64; 3],
    length: f64,
) -> SolidKey {
    let outline = ProfileBuilder::new(outline)
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a simple outline");
    model
        .add_extrusion(
            &outline,
            length,
            &Isometry::translation(Vector::new(x, y, z)).expect("a finite shift"),
        )
        .expect("a positive length")
}
