//! The same circle drawn on a flat face as §176.2's, written as **one closed
//! edge** rather than four quarter arcs, keeps its centre and its radius too.
//!
//! `docs/decisions.md` §218.6 and §218.7: `round_run` answered every full
//! `CIRCLE` or `ELLIPSE` whose two ends are one vertex with
//! `EdgeRun::RightRoundWhereItsFacesCross`, which carries nothing and leaves the
//! reader to ask the two faces where the edge goes. Where those two faces are
//! the one flat surface they cross in a straight line, which never closes, so
//! nothing said where the circle was — and `Model::validate` refused the model
//! the reader had just built, by `ClosedEdgeBetweenTwoFlatFaces`. Four KiCad
//! files were stopped there.
//!
//! The two tests below are the two arms of `the_faces_along_it_name_no_crossing`:
//! the disc's plane written as the very entity the face round it sits on, and
//! written again from its own placement so the numbers come out a hair apart —
//! which is what the corpus does, and is why the reader cannot ask `==` alone.
//! The second one goes as far as this reader can take it and is then stopped by
//! a check in `rcad-topo` that reads the two faces and never the run.

mod written_by_hand;

use rcad_geom::{Deflection, EdgeRun, Surface};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use written_by_hand::{a_header, real};

const CENTRE: [f64; 2] = [5.0, 5.0];
const RADIUS: f64 = 2.0;
const TOP: f64 = 2.0;

/// The disc and the face round it sit on one `PLANE` entity, so the two
/// surfaces along the rim are equal bit for bit.
#[test]
fn a_closed_circle_between_two_faces_on_the_one_plane_names_its_cylinder() {
    a_closed_circle_reaches_the_mesh_round(TheDiscsPlane::TheOneTheFaceRoundItSitsOn);
}

/// The same circle with the disc on a `PLANE` of its own, whose direction is a
/// hair off the other's, gets the same run out of this reader — and `rcad-topo`
/// refuses it anyway, one check further on.
///
/// This is the corpus's own shape, and the reason the reader cannot ask `==`
/// alone: on `SW_Tactile_SPST_Angled_PTS645Vx31-2LFS.step` the planes either
/// side of entity `#1234`'s circle differ by `5.2e-16` in one component of the
/// normal and `1.6e-15` in the offset, which is one plane written twice.
///
/// **What the refusal is now is not what it was.** It used to come from
/// `check_something_curved_says_where_this_edge_bends`, because the run named
/// nothing; that check passes now — the run carries the circle's own cylinder,
/// measured, and a cylinder is not a plane. It comes from
/// `check_closed_edges_are_walked_the_way_their_surfaces_wind` instead, whose
/// `a_closed_crossing_is_walked_the_way_it_is_stored` reads the two **face**
/// surfaces and never the run's `and`, so it finds no curved surface to read a
/// winding off and refuses under the older name. The bit-identical pair above
/// slips past it only because no corner of either face is off the other, so the
/// check has nothing to ask about.
///
/// That is §218.6's gap one crate up, in `crates/rcad-topo/src/validate.rs`,
/// and it is a **gap** and not a limit under §16.10: the answer exists — the
/// run names the cylinder and the cylinder says which way the circle winds —
/// and the check simply does not look there. It is outside this lane's write
/// set; this test is the demonstration §16.10 asks for in place of a paragraph,
/// and it turns into the one above the day that check reads the run.
#[test]
fn two_spellings_of_one_plane_stop_at_a_check_that_reads_the_faces_and_not_the_run() {
    let text = a_slab_with_a_circle_drawn_round_on_top(TheDiscsPlane::ItsOwnAHairOff);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let mut log = ProvenanceLog::new();
    let refused = Import::read(&file, &Policy::certified(), &mut log)
        .expect_err("the winding check still refuses a closed edge between two flat faces");
    let named = format!("{refused:?}");
    assert!(
        named.contains("ClosedEdgeBetweenTwoFlatFaces"),
        "the one refusal left on this shape is the winding check's, and this says {named}"
    );
}

#[derive(Clone, Copy)]
enum TheDiscsPlane {
    TheOneTheFaceRoundItSitsOn,
    ItsOwnAHairOff,
}

fn a_closed_circle_reaches_the_mesh_round(disc: TheDiscsPlane) {
    let text = a_slab_with_a_circle_drawn_round_on_top(disc);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let import =
        Import::read(&file, &policy, &mut log).expect("a slab with a closed circle on top reads");
    import
        .model()
        .validate(&policy, &mut log)
        .expect("and the model it makes passes the full sweep");
    let [(_, slab)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let model = import.model();
    let solid = model.solid(*slab).expect("this model made the key");

    let mut rims = Vec::new();
    for face in solid.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if edge.start().key() == edge.end().key() && !rims.contains(&edge.key()) {
                    rims.push(edge.key());
                    let EdgeRun::RightRoundWhereItCrosses { and } = edge.run() else {
                        panic!(
                            "the circle's two faces name no crossing, so its run has to carry \
                             the surface that does; it says {:?}",
                            edge.run()
                        );
                    };
                    let Surface::Cylinder(round) = and else {
                        panic!("a circle on flat faces is named by its own cylinder, not {and:?}");
                    };
                    assert_eq!(
                        round.radius(),
                        RADIUS,
                        "the cylinder is the one of the circle's own radius"
                    );
                }
            }
        }
    }
    assert_eq!(rims.len(), 1, "the circle is one closed edge");

    let mesh = solid
        .mesh(Deflection::of(0.01).expect("a length"), &policy, &mut log)
        .expect("the slab meshes");
    let allowed = mesh.deviation().magnitude();
    let along = mesh.points_along_edge(rims[0]);
    assert!(
        along.len() > 8,
        "a whole circle of radius two at a hundredth came back with {} points",
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

fn how_far_off_the_circle([x, y, z]: [f64; 3]) -> f64 {
    let across = ((x - CENTRE[0]).powi(2) + (y - CENTRE[1]).powi(2)).sqrt();
    (across - RADIUS).abs().max((z - TOP).abs())
}

/// A 10 × 10 × 2 slab whose top is two faces on one plane: a disc of radius
/// two about `(5, 5)` and the square round it, with the circle between them
/// written as one `EDGE_CURVE` that starts and ends at the same `VERTEX_POINT`.
///
/// Everything but the circle is §176.2's slab, so that what the two files
/// differ by is the one thing under test.
fn a_slab_with_a_circle_drawn_round_on_top(disc: TheDiscsPlane) -> String {
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

    let mut text = a_header(".MILLI.");
    for (which, corner) in corners.iter().enumerate() {
        text += &format!(
            "#{}=CARTESIAN_POINT('',{});\n",
            100 + which,
            triple(*corner)
        );
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 110 + which, 100 + which);
    }
    text += &format!(
        "#120=CARTESIAN_POINT('',{});\n",
        triple([x + RADIUS, y, TOP])
    );
    text += "#125=VERTEX_POINT('',#120);\n";
    for (which, [from, to]) in edges.iter().enumerate() {
        let way: Vec<f64> = (0..3)
            .map(|axis| {
                let step = corners[*to][axis] - corners[*from][axis];
                if step == 0.0 { 0.0 } else { step.signum() }
            })
            .collect();
        text += &format!(
            "#{}=DIRECTION('',{});\n",
            340 + which,
            triple([way[0], way[1], way[2]])
        );
        text += &format!(
            "#{}=VECTOR('',#{},{});\n",
            320 + which,
            340 + which,
            real(1.0)
        );
        text += &format!(
            "#{}=LINE('',#{},#{});\n",
            300 + which,
            100 + from,
            320 + which
        );
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
    text += "#610=EDGE_CURVE('',#125,#125,#604,.T.);\n";

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
        text += &format!(
            "#{}=FACE_OUTER_BOUND('',#{},.T.);\n",
            440 + which,
            430 + which
        );
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

    text += "#700=ORIENTED_EDGE('',*,*,#610,.F.);\n";
    text += "#710=EDGE_LOOP('',(#700));\n";
    text += "#711=FACE_BOUND('',#710,.T.);\n";
    text += "#720=ORIENTED_EDGE('',*,*,#610,.T.);\n";
    text += "#730=EDGE_LOOP('',(#720));\n";
    text += "#731=FACE_OUTER_BOUND('',#730,.T.);\n";
    text += &match disc {
        TheDiscsPlane::TheOneTheFaceRoundItSitsOn => "#732=PLANE('',#461);\n".to_owned(),
        TheDiscsPlane::ItsOwnAHairOff => {
            let mut own = format!("#734=CARTESIAN_POINT('',{});\n", triple([x, y, TOP]));
            own += &format!("#735=DIRECTION('',{});\n", triple([1e-17, 0.0, 1.0]));
            own += "#736=AXIS2_PLACEMENT_3D('',#734,#735,$);\n";
            own + "#732=PLANE('',#736);\n"
        }
    };
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
