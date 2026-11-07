//! Probe, not kept as it stands.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog, side_of_a_plane_where_three_planes_meet};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

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
    let mut model = a_model();
    let plate = a_prism(
        &mut model,
        vec![[0.0, 0.0], [6.0, 0.0], [6.0, 6.0], [0.0, 6.0]],
        [0.0, 0.0, 0.0],
        1.0,
    );
    let holes = [
        a_prism(
            &mut model,
            vec![[3.0, 2.0], [4.0, 2.0], [4.0, 3.0], [3.0, 3.0]],
            [0.0, 0.0, -1.0],
            3.0,
        ),
        a_prism(
            &mut model,
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
