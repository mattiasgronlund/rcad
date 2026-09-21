//! A circle a file draws on a flat face, with that face's plane on both sides
//! of it, reaches the model with its centre and its radius.
//!
//! `docs/decisions.md` §176.2: the reader used to name each arc of such a
//! circle by the circle's own plane, which for a circle drawn on a flat face is
//! the face's plane again — so the run said "where this plane crosses itself",
//! nothing in the model knew where the circle was, and the mesher drew the
//! triangle on its corners. The arc is named by the cylinder of the circle's
//! radius about its axis now, which crosses the face in exactly the circle.
//!
//! [`a_slab_with_a_circle_drawn_on_top`] is the shape written out by hand, with
//! nothing else on the face to get in the way; the KiCad LED the finding came
//! from is checked for how far it now gets.

mod written_by_hand;

use std::{env, fs, path::PathBuf};

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};
use written_by_hand::{a_header, real};

const CENTRE: [f64; 2] = [5.0, 5.0];
const RADIUS: f64 = 2.0;
const TOP: f64 = 2.0;

#[test]
fn a_circle_drawn_on_a_slabs_top_is_meshed_round_and_not_as_a_square() {
    let text = a_slab_with_a_circle_drawn_on_top();
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let import = Import::read(&file, &policy, &mut log).expect("a slab with a circle on top reads");
    let [(_, slab)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let model = import.model();
    let solid = model.solid(*slab).expect("this model made the key");
    let mesh = solid
        .mesh(Deflection::of(0.01).expect("a length"), &policy, &mut log)
        .expect("the slab meshes");
    let allowed = mesh.deviation().magnitude();

    let mut arcs = Vec::new();
    for face in solid.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if how_far_off_the_circle(edge.start().point().coordinates()) < 1e-12
                    && !arcs.contains(&edge.key())
                {
                    arcs.push(edge.key());
                }
            }
        }
    }
    assert_eq!(arcs.len(), 4, "the circle is four quarter arcs");
    for arc in arcs {
        let along = mesh.points_along_edge(arc);
        assert!(
            along.len() > 4,
            "a quarter circle of radius two at a hundredth came back with {} points",
            along.len()
        );
        for &at in along {
            let point = mesh.points()[at as usize].coordinates();
            let off = how_far_off_the_circle(point);
            assert!(
                off <= allowed,
                "{point:?} is {off} off the circle, past the published {allowed}"
            );
        }
    }
}

/// The LED is no longer stopped for want of a curve to place its circle by.
///
/// **It does not import yet**, and what stops it is measured rather than
/// guessed: `Model::validate` meshes the solid as coarsely as a mesh goes to
/// read the sign of its volume, which walks each arc of the circle in one step
/// of a quarter turn, and the disc's two pin holes sit closer to the circle
/// than that step's chord. The boundary crosses a hole there. Asked at a
/// hundredth instead, the disc meshes and the file stops on a later face.
#[test]
fn the_led_the_finding_came_from_gets_past_the_circle_on_its_face() {
    let bytes = fs::read(corpus().join("kicad").join(THE_LED)).expect("the LED is in the corpus");
    let file = StepFile::read(&bytes).expect("it parses");
    let refused = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
        .expect_err("the LED does not import yet");
    let nothing_names_the_curve = if let Refusal::ThisEntityOfTheSolidWasRefused { why, .. }
    | Refusal::TheSolidTheFileDescribesIsNotOne { why, .. } = &refused
    {
        matches!(
            **why,
            rcad_topo::Error::AnEdgeBendsButEveryFaceAlongItIsFlat { .. }
        )
    } else {
        false
    };
    assert!(
        !nothing_names_the_curve,
        "the circle reached the model with nothing naming its curve: {refused:?}"
    );
}

const THE_LED: &str = "LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step";

fn how_far_off_the_circle([x, y, z]: [f64; 3]) -> f64 {
    let across = ((x - CENTRE[0]).powi(2) + (y - CENTRE[1]).powi(2)).sqrt();
    (across - RADIUS).abs().max((z - TOP).abs())
}

fn corpus() -> PathBuf {
    PathBuf::from(env::var_os("RCAD_STEP_CORPUS").expect("`just t` sets RCAD_STEP_CORPUS"))
}

/// A 10 × 10 × 2 slab whose top is two faces on one plane: a disc of radius
/// two about `(5, 5)` and the square round it, with the circle between them
/// written as four quarter `CIRCLE` arcs.
fn a_slab_with_a_circle_drawn_on_top() -> String {
    let corners = [
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [0.0, 10.0, 0.0],
        [0.0, 0.0, TOP],
        [10.0, 0.0, TOP],
        [10.0, 10.0, TOP],
        [0.0, 10.0, TOP],
    ];
    let edges: [[usize; 2]; 12] = [
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0],
        [4, 5],
        [5, 6],
        [6, 7],
        [7, 4],
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7],
    ];
    let faces: [([usize; 4], [f64; 3], usize); 6] = [
        ([0, 3, 2, 1], [0.0, 0.0, -1.0], 0),
        ([4, 5, 6, 7], [0.0, 0.0, 1.0], 4),
        ([0, 1, 5, 4], [0.0, -1.0, 0.0], 0),
        ([3, 7, 6, 2], [0.0, 1.0, 0.0], 3),
        ([0, 4, 7, 3], [-1.0, 0.0, 0.0], 0),
        ([1, 2, 6, 5], [1.0, 0.0, 0.0], 1),
    ];
    let [x, y] = CENTRE;
    let on_the_circle = [
        [x + RADIUS, y, TOP],
        [x, y + RADIUS, TOP],
        [x - RADIUS, y, TOP],
        [x, y - RADIUS, TOP],
    ];

    let mut text = a_header(".MILLI.");
    for (which, corner) in corners.iter().enumerate() {
        text += &format!("#{}=CARTESIAN_POINT('',{});\n", 100 + which, triple(*corner));
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 110 + which, 100 + which);
    }
    for (which, corner) in on_the_circle.iter().enumerate() {
        text += &format!("#{}=CARTESIAN_POINT('',{});\n", 120 + which, triple(*corner));
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 125 + which, 120 + which);
    }
    for (which, [from, to]) in edges.iter().enumerate() {
        let way = (0..3).map(|axis| {
            let step = corners[*to][axis] - corners[*from][axis];
            if step == 0.0 { 0.0 } else { step.signum() }
        });
        let way: Vec<f64> = way.collect();
        text += &format!(
            "#{}=DIRECTION('',{});\n",
            340 + which,
            triple([way[0], way[1], way[2]])
        );
        text += &format!("#{}=VECTOR('',#{},{});\n", 320 + which, 340 + which, real(1.0));
        text += &format!("#{}=LINE('',#{},#{});\n", 300 + which, 100 + from, 320 + which);
        text += &format!(
            "#{}=EDGE_CURVE('',#{},#{},#{},.T.);\n",
            380 + which,
            110 + from,
            110 + to,
            300 + which
        );
    }
    text += &format!("#600=CARTESIAN_POINT('',{});\n", triple([x, y, TOP]));
    text += "#601=DIRECTION('',(0.,0.,1.));\n";
    text += "#602=DIRECTION('',(1.,0.,0.));\n";
    text += "#603=AXIS2_PLACEMENT_3D('',#600,#601,#602);\n";
    text += &format!("#604=CIRCLE('',#603,{});\n", real(RADIUS));
    for quarter in 0..4 {
        text += &format!(
            "#{}=EDGE_CURVE('',#{},#{},#604,.T.);\n",
            610 + quarter,
            125 + quarter,
            125 + (quarter + 1) % 4
        );
    }

    for (which, (round, normal, origin)) in faces.iter().enumerate() {
        for step in 0..4 {
            let (from, to) = (round[step], round[(step + 1) % 4]);
            let (edge, forwards) = which_edge(&edges, from, to);
            text += &format!(
                "#{}=ORIENTED_EDGE('',*,*,#{},{});\n",
                400 + which * 4 + step,
                380 + edge,
                if forwards { ".T." } else { ".F." }
            );
        }
        let steps: Vec<String> = (0..4)
            .map(|step| format!("#{}", 400 + which * 4 + step))
            .collect();
        text += &format!("#{}=EDGE_LOOP('',({}));\n", 430 + which, steps.join(","));
        text += &format!("#{}=FACE_OUTER_BOUND('',#{},.T.);\n", 440 + which, 430 + which);
        text += &format!(
            "#{}=CARTESIAN_POINT('',{});\n",
            470 + which,
            triple(corners[*origin])
        );
        text += &format!("#{}=DIRECTION('',{});\n", 480 + which, triple(*normal));
        text += &format!(
            "#{}=AXIS2_PLACEMENT_3D('',#{},#{},$);\n",
            460 + which,
            470 + which,
            480 + which
        );
        text += &format!("#{}=PLANE('',#{});\n", 450 + which, 460 + which);
        let bounds = if which == 1 {
            format!("#{},#711", 440 + which)
        } else {
            format!("#{}", 440 + which)
        };
        text += &format!(
            "#{}=ADVANCED_FACE('',({bounds}),#{},.T.);\n",
            490 + which,
            450 + which
        );
    }

    for (step, quarter) in (0..4).rev().enumerate() {
        text += &format!("#{}=ORIENTED_EDGE('',*,*,#{},.F.);\n", 700 + step, 610 + quarter);
        text += &format!("#{}=ORIENTED_EDGE('',*,*,#{},.T.);\n", 720 + quarter, 610 + quarter);
    }
    text += "#710=EDGE_LOOP('',(#700,#701,#702,#703));\n";
    text += "#711=FACE_BOUND('',#710,.T.);\n";
    text += "#730=EDGE_LOOP('',(#720,#721,#722,#723));\n";
    text += "#731=FACE_OUTER_BOUND('',#730,.T.);\n";
    text += "#732=PLANE('',#461);\n";
    text += "#733=ADVANCED_FACE('',(#731),#732,.T.);\n";

    text += "#500=CLOSED_SHELL('',(#490,#491,#492,#493,#494,#495,#733));\n";
    text += "#501=MANIFOLD_SOLID_BREP('',#500);\n";
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}

fn which_edge(edges: &[[usize; 2]; 12], from: usize, to: usize) -> (usize, bool) {
    for (which, ends) in edges.iter().enumerate() {
        if *ends == [from, to] {
            return (which, true);
        }
        if *ends == [to, from] {
            return (which, false);
        }
    }
    panic!("the slab's loops walk only its own edges, and {from} to {to} is not one");
}

fn triple([x, y, z]: [f64; 3]) -> String {
    format!("({},{},{})", real(x), real(y), real(z))
}
