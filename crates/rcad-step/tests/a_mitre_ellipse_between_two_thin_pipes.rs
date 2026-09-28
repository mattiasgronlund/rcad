//! A closed mitre `ELLIPSE` where two pipes of radius `0.1` meet at a right
//! angle, written with the numbers of mitre `#691` in
//! `kicad/Relay_DPDT_Fujitsu_FTR-F1C.step`, is named by its own plane.
//!
//! The file writes the long size as `0.141421356237`, twelve digits of
//! `0.1·√2` and `3.1e-13` short of it. Holding the file's ellipse to the
//! allowance `f64` roundings give put it 61 to 66 roundings off each pipe,
//! so the run was named by its two pipes, and two equal pipes at a right
//! angle cross in two ellipses: the relay is refused
//! `CannotNameWhichPieceOfACrossingThisIs`, and this elbow a band whose wire
//! will not close. The sizes are now worked out from
//! the pipe, and the file's own are held to what twelve digits cost
//! (`docs/decisions.md` §229.5 for where this was found, §335 for the fix).
//!
//! The controls hold the other side: a reference direction turned a
//! millionth of a radian, a size wrong in its fourth digit, and a size
//! written in eleven digits all keep the run named by its pipes.

mod written_by_hand;

use rcad_geom::{Deflection, EdgeRun, Point, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};
use written_by_hand::a_header;

/// The mitre's reference direction as the relay writes it: along its long
/// axis, down the vertical pipe's outside.
const DOWN_THE_LONG_AXIS: &str = "(0.707106781187,-2.590652556828E-015,-0.707106781187)";

/// Where the mitre, both seams and nothing else meet.
const CORNER: [f64; 3] = [-0.95, 1.9, 16.05];

/// The level pipe's placement origin and radius, as the relay writes them.
const THE_RELAYS_LEVEL_PIPE: [&str; 2] = ["(4.125,1.9,16.15)", "0.1"];

#[test]
fn a_mitre_written_in_twelve_digits_is_named_by_its_plane() {
    the_mitre_is_named_by_its_plane(DOWN_THE_LONG_AXIS, "0.141421356237,0.1");
}

#[test]
fn a_mitre_written_to_the_last_bit_is_named_by_its_plane() {
    the_mitre_is_named_by_its_plane(DOWN_THE_LONG_AXIS, "0.1414213562373095,0.1");
}

/// The schema lets the first size be the short one, so a file may put its
/// reference direction across the pipe. Each size is worked out along its own
/// way, not assumed to be the long one.
#[test]
fn a_mitre_whose_reference_runs_along_its_short_axis_is_named_by_its_plane() {
    the_mitre_is_named_by_its_plane("(0.E+000,1.,0.E+000)", "0.1,0.141421356237");
}

/// Turning the reference direction about the normal leaves the plane where it
/// was, so the plane would still cut the pipes in the right curve. The file's
/// curve is the one checked, and it is off both pipes by about `1e-8`.
#[test]
fn a_mitre_whose_reference_is_turned_a_millionth_of_a_radian_is_named_by_its_pipes() {
    the_mitre_is_named_by_its_pipes(
        "(0.707106781187,1.E-006,-0.707106781187)",
        "0.141421356237,0.1",
        THE_RELAYS_LEVEL_PIPE,
    );
}

/// A file whose sizes are plainly wrong is not read as the curve the plane
/// cuts, though the plane would cut the pipes in the right one.
#[test]
fn a_mitre_whose_size_is_wrong_in_its_fourth_digit_is_named_by_its_pipes() {
    the_mitre_is_named_by_its_pipes(DOWN_THE_LONG_AXIS, "0.1415,0.1", THE_RELAYS_LEVEL_PIPE);
}

/// The edge of the allowance for the file's sizes (`docs/decisions.md`
/// §15.6): `0.14142135624` is `0.1·√2` rounded to eleven digits, `2.7e-12`
/// long, which is past five parts in `10¹²` of the size.
#[test]
fn a_mitre_whose_size_is_written_in_eleven_digits_is_named_by_its_pipes() {
    the_mitre_is_named_by_its_pipes(
        DOWN_THE_LONG_AXIS,
        "0.14142135624,0.1",
        THE_RELAYS_LEVEL_PIPE,
    );
}

/// The level pipe `1e-13` wider and `1e-13` higher, so its underside, both
/// seams and every corner stay where they were, and the plane cuts it `2e-13`
/// above the upright pipe's cut at the mitre's top. The file's ellipse is
/// inside what its twelve-digit size is allowed of both pipes; the ellipse
/// the upright pipe gives is not on the level one, and that is what keeps
/// the run named by its pipes.
#[test]
fn a_mitre_between_pipes_a_hair_apart_is_named_by_its_pipes() {
    the_mitre_is_named_by_its_pipes(
        DOWN_THE_LONG_AXIS,
        "0.141421356237,0.1",
        ["(4.125,1.9,16.1500000000001)", "0.1000000000001"],
    );
}

fn the_mitre_is_named_by_its_plane(reference: &str, sizes: &str) {
    let text = an_elbow(reference, sizes, THE_RELAYS_LEVEL_PIPE);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let import = match Import::read(&file, &policy, &mut log) {
        Ok(import) => import,
        Err(refused) => panic!(
            "an elbow of two pipes with its mitre written {sizes} is refused: {refused:?}"
        ),
    };
    let [(_, elbow)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let model = import.model();
    let solid = model.solid(*elbow).expect("this model made the key");
    let leaning = Vector::new(1.0, 0.0, 1.0);

    let mut mitre = None;
    for face in solid.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if edge.start().key() != edge.end().key()
                    || edge.start().point().coordinates() != CORNER
                {
                    continue;
                }
                mitre = Some(edge.key());
                let EdgeRun::RightRoundWhereItCrosses { and } = edge.run() else {
                    panic!(
                        "the mitre has been checked to lie on both pipes, so it is named by \
                         the surface the file draws it on; it says {:?}",
                        edge.run()
                    );
                };
                let Surface::Plane(flat) = and else {
                    panic!("a mitre between two pipes is named by its own plane, not {and:?}");
                };
                assert!(
                    flat.normal().cross(leaning).length() < 1e-14,
                    "the plane is the mitre's own, not one square to {:?}",
                    flat.normal()
                );
                assert!(
                    flat.signed_distance_to(Point::new(-1.05, 1.9, 16.15)).abs() < 1e-14,
                    "the plane runs through the mitre's centre"
                );
            }
        }
    }
    let mitre = mitre.expect("the elbow has a closed mitre between its two pipes");

    model
        .validate(&policy, &mut log)
        .expect("the model the reader built passes the full sweep");
    let mesh = solid
        .mesh(Deflection::of(0.001).expect("a length"), &policy, &mut log)
        .expect("the elbow meshes");
    let allowed = mesh.deviation().magnitude();
    let along = mesh.points_along_edge(mitre);
    assert!(
        along.len() > 8,
        "a whole mitre round a pipe of radius a tenth at a thousandth came back with {} points",
        along.len()
    );
    for &at in along {
        let [x, y, z] = mesh.points()[at as usize].coordinates();
        let off_the_upright = ((x + 1.05).hypot(y - 1.9) - 0.1).abs();
        let off_the_level = ((y - 1.9).hypot(z - 16.15) - 0.1).abs();
        assert!(
            off_the_upright.max(off_the_level) <= allowed,
            "{:?} is {off_the_upright} off the upright pipe and {off_the_level} off the level \
             one, past the published {allowed}",
            [x, y, z]
        );
    }
}

/// Named by its two pipes, which cross in two ellipses at a right angle, so
/// a band's wire does not come back to where it set off in the band's own two
/// coordinates. That is the refusal the reader gave the twelve-digit mitre
/// before `docs/decisions.md` §335.
fn the_mitre_is_named_by_its_pipes(reference: &str, sizes: &str, level_pipe: [&str; 2]) {
    let text = an_elbow(reference, sizes, level_pipe);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let refused = match Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new()) {
        Ok(_) => panic!(
            "a mitre written with reference {reference} and sizes {sizes}, beside a level \
             pipe at {level_pipe:?}, is not on both pipes, and was read as the curve its \
             plane cuts"
        ),
        Err(refused) => refused,
    };
    assert!(
        matches!(
            &refused,
            Refusal::TheSolidTheFileDescribesIsNotOne { why, .. }
                if matches!(**why, rcad_topo::Error::AWireWillNotLieFlatOnItsFace { .. })
        ),
        "a mitre named by its two pipes leaves a band's wire unable to close, not {refused:?}"
    );
}

/// An elbow of two pipes of radius `0.1`: one upright on `x = −1.05`,
/// `y = 1.9`, closed by a disc at `z = 15.15`, the other level through
/// `(4.125, 1.9, 16.15)` and leaning `3.7e-15` the way the relay's does,
/// closed by a disc at `x = 4.125`. They meet in one closed `ELLIPSE` round
/// `(−1.05, 1.9, 16.15)` square to `(1, 0, 1)`, with one corner at
/// `(−0.95, 1.9, 16.05)` where both seams end.
///
/// Every number but the mitre's reference direction and sizes, and the level
/// pipe's placement origin and radius, is the relay's own text.
fn an_elbow(reference: &str, sizes: &str, level_pipe: [&str; 2]) -> String {
    let [level_origin, level_radius] = level_pipe;
    let mut text = a_header(".MILLI.");
    text += "#20=DIRECTION('',(0.E+000,0.E+000,1.));\n";
    text += "#21=DIRECTION('',(1.,0.E+000,0.E+000));\n";
    text += "#22=DIRECTION('',(0.E+000,0.E+000,-1.));\n";
    text += "#23=DIRECTION('',(1.,-3.663735981263E-015,0.E+000));\n";
    text += "#24=DIRECTION('',(0.707106781187,-2.590652556828E-015,0.707106781187));\n";
    text += &format!("#25=DIRECTION('',{reference});\n");
    text += "#26=VECTOR('',#20,1.);\n";
    text += "#27=VECTOR('',#23,1.);\n";

    text += "#100=CARTESIAN_POINT('',(-0.95,1.9,15.15));\n";
    text += "#101=CARTESIAN_POINT('',(-0.95,1.9,16.05));\n";
    text += "#102=CARTESIAN_POINT('',(4.125,1.9,16.05));\n";
    for corner in 0..3 {
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 110 + corner, 100 + corner);
    }
    text += "#120=CARTESIAN_POINT('',(-1.05,1.9,15.15));\n";
    text += "#121=CARTESIAN_POINT('',(-1.05,1.9,16.15));\n";
    text += "#122=CARTESIAN_POINT('',(4.125,1.9,16.15));\n";
    text += &format!("#123=CARTESIAN_POINT('',{level_origin});\n");

    text += "#130=AXIS2_PLACEMENT_3D('',#120,#20,#21);\n";
    text += "#131=AXIS2_PLACEMENT_3D('',#121,#24,#25);\n";
    text += "#132=AXIS2_PLACEMENT_3D('',#122,#23,#22);\n";
    text += "#133=AXIS2_PLACEMENT_3D('',#123,#23,#22);\n";
    text += "#135=AXIS2_PLACEMENT_3D('',#120,#22,#21);\n";

    text += "#140=CIRCLE('',#130,0.1);\n";
    text += &format!("#141=ELLIPSE('',#131,{sizes});\n");
    text += "#142=CIRCLE('',#132,0.1);\n";
    text += "#143=LINE('',#100,#26);\n";
    text += "#144=LINE('',#102,#27);\n";
    text += "#150=EDGE_CURVE('',#110,#110,#140,.T.);\n";
    text += "#151=EDGE_CURVE('',#111,#111,#141,.T.);\n";
    text += "#152=EDGE_CURVE('',#112,#112,#142,.T.);\n";
    text += "#153=EDGE_CURVE('',#110,#111,#143,.T.);\n";
    text += "#154=EDGE_CURVE('',#111,#112,#144,.T.);\n";

    text += "#160=CYLINDRICAL_SURFACE('',#130,0.1);\n";
    text += &format!("#161=CYLINDRICAL_SURFACE('',#133,{level_radius});\n");
    text += "#162=PLANE('',#135);\n";
    text += "#163=PLANE('',#132);\n";

    let faces: [(&str, &[(&str, &str)]); 4] = [
        (
            "#160",
            &[("#150", ".T."), ("#153", ".T."), ("#151", ".F."), ("#153", ".F.")],
        ),
        (
            "#161",
            &[("#151", ".T."), ("#154", ".T."), ("#152", ".F."), ("#154", ".F.")],
        ),
        ("#162", &[("#150", ".F.")]),
        ("#163", &[("#152", ".T.")]),
    ];
    for (which, (surface, walked)) in faces.iter().enumerate() {
        let mut oriented = Vec::new();
        for (step, (edge, sense)) in walked.iter().enumerate() {
            let id = 200 + which * 10 + step;
            text += &format!("#{id}=ORIENTED_EDGE('',*,*,{edge},{sense});\n");
            oriented.push(format!("#{id}"));
        }
        let (bound, face) = (240 + which * 2, 260 + which);
        text += &format!("#{}=EDGE_LOOP('',({}));\n", bound, oriented.join(","));
        text += &format!("#{}=FACE_OUTER_BOUND('',#{bound},.T.);\n", bound + 1);
        text += &format!(
            "#{face}=ADVANCED_FACE('',(#{}),{surface},.T.);\n",
            bound + 1
        );
    }
    text += "#300=CLOSED_SHELL('',(#260,#261,#262,#263));\n";
    text += "#301=MANIFOLD_SOLID_BREP('',#300);\n";
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}
