//! A mitre `ELLIPSE` whose centre the file writes off one of its two pipes
//! is named by its pipes, and its sibling whose centre is on both keeps its
//! plane.

mod written_by_hand;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use written_by_hand::a_header;

#[test]
fn probe_338() {
    let text = an_elbow(&MITRE_338);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let got = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new());
    match got {
        Ok(import) => println!("338 imports: {:?}", import.solids().len()),
        Err(refused) => println!("338 refused: {refused:?}"),
    }
}

#[test]
fn probe_1769() {
    let text = an_elbow(&MITRE_1769);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let got = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new());
    match got {
        Ok(import) => println!("1769 imports: {:?}", import.solids().len()),
        Err(refused) => println!("1769 refused: {refused:?}"),
    }
}

struct ABend {
    upright_origin: &'static str,
    upright_reference: &'static str,
    leaning_origin: &'static str,
    leaning_direction: &'static str,
    leaning_reference: &'static str,
    mitre_centre: &'static str,
    mitre_normal: &'static str,
    mitre_reference: &'static str,
    mitre_senses: [&'static str; 2],
    corner: &'static str,
    foot: &'static str,
    bottom_centre: &'static str,
    onward: &'static str,
    far_corner: &'static str,
    far_centre: &'static str,
}

const MITRE_338: ABend = ABend {
    upright_origin: "(16.471428571429,4.4,1.)",
    upright_reference: "(-1.,0.E+000,0.E+000)",
    leaning_origin: "(16.354271283903,4.4,9.931443001811)",
    leaning_direction: "(0.707106781187,0.E+000,0.707106781187)",
    leaning_reference: "(4.252662132327E-032,1.,-4.252662132327E-032)",
    mitre_centre: "(16.471428571429,4.4,10.048600289336)",
    mitre_normal: "(-0.382683432365,0.E+000,-0.923879532511)",
    mitre_reference: "(-0.923879532511,0.E+000,0.382683432365)",
    mitre_senses: [".T.", ".F."],
    corner: "(16.471428571429,4.8,10.048600289336)",
    foot: "(16.471428571429,4.8,1.)",
    bottom_centre: "(16.471428571429,4.4,1.)",
    onward: "(0.707106781187,0.E+000,0.707106781187)",
    far_corner: "(17.471428571429,4.8,11.048600289336)",
    far_centre: "(17.4714285714285,4.4,11.0486002893365)",
};

const MITRE_1769: ABend = ABend {
    upright_origin: "(22.1,4.4,3.455212154016E-032)",
    upright_reference: "(1.,0.E+000,0.E+000)",
    leaning_origin: "(20.931443001811,4.4,11.217157287525)",
    leaning_direction: "(0.707106781187,0.E+000,-0.707106781187)",
    leaning_reference: "(-4.385368252713E-032,1.,-4.385368252713E-032)",
    mitre_centre: "(22.1,4.4,10.048600289336)",
    mitre_normal: "(-0.382683432365,0.E+000,0.923879532511)",
    mitre_reference: "(0.923879532511,0.E+000,0.382683432365)",
    mitre_senses: [".F.", ".T."],
    corner: "(22.1,4.8,10.048600289336)",
    foot: "(22.1,4.8,1.)",
    bottom_centre: "(22.1,4.4,1.)",
    onward: "(-0.707106781187,0.E+000,0.707106781187)",
    far_corner: "(21.1,4.8,11.048600289336)",
    far_centre: "(21.1,4.4,11.048600289336)",
};

fn an_elbow(bend: &ABend) -> String {
    let mut text = a_header(".MILLI.");
    text += "#20=DIRECTION('',(0.E+000,0.E+000,1.));\n";
    text += &format!("#21=DIRECTION('',{});\n", bend.upright_reference);
    text += "#22=DIRECTION('',(0.E+000,0.E+000,-1.));\n";
    text += &format!("#23=DIRECTION('',{});\n", bend.leaning_direction);
    text += &format!("#24=DIRECTION('',{});\n", bend.leaning_reference);
    text += &format!("#25=DIRECTION('',{});\n", bend.mitre_normal);
    text += &format!("#26=DIRECTION('',{});\n", bend.mitre_reference);
    text += &format!("#27=DIRECTION('',{});\n", bend.onward);
    text += "#28=DIRECTION('',(0.E+000,1.,0.E+000));\n";
    text += "#29=DIRECTION('',(1.,0.E+000,0.E+000));\n";
    text += "#30=VECTOR('',#20,1.);\n";
    text += "#31=VECTOR('',#27,1.);\n";

    text += &format!("#100=CARTESIAN_POINT('',{});\n", bend.foot);
    text += &format!("#101=CARTESIAN_POINT('',{});\n", bend.corner);
    text += &format!("#102=CARTESIAN_POINT('',{});\n", bend.far_corner);
    for corner in 0..3 {
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 110 + corner, 100 + corner);
    }
    text += &format!("#120=CARTESIAN_POINT('',{});\n", bend.upright_origin);
    text += &format!("#121=CARTESIAN_POINT('',{});\n", bend.mitre_centre);
    text += &format!("#122=CARTESIAN_POINT('',{});\n", bend.leaning_origin);
    text += &format!("#123=CARTESIAN_POINT('',{});\n", bend.bottom_centre);
    text += &format!("#124=CARTESIAN_POINT('',{});\n", bend.far_centre);

    text += "#130=AXIS2_PLACEMENT_3D('',#120,#20,#21);\n";
    text += "#131=AXIS2_PLACEMENT_3D('',#121,#25,#26);\n";
    text += "#132=AXIS2_PLACEMENT_3D('',#122,#23,#24);\n";
    text += "#133=AXIS2_PLACEMENT_3D('',#123,#20,#29);\n";
    text += "#134=AXIS2_PLACEMENT_3D('',#123,#22,#29);\n";
    text += "#135=AXIS2_PLACEMENT_3D('',#124,#27,#28);\n";

    text += "#140=CIRCLE('',#133,0.4);\n";
    text += "#141=ELLIPSE('',#131,0.432956880117,0.4);\n";
    text += "#142=CIRCLE('',#135,0.4);\n";
    text += "#143=LINE('',#100,#30);\n";
    text += "#144=LINE('',#101,#31);\n";
    text += "#150=EDGE_CURVE('',#110,#110,#140,.T.);\n";
    text += "#151=EDGE_CURVE('',#111,#111,#141,.T.);\n";
    text += "#152=EDGE_CURVE('',#112,#112,#142,.T.);\n";
    text += "#153=EDGE_CURVE('',#110,#111,#143,.T.);\n";
    text += "#154=EDGE_CURVE('',#111,#112,#144,.T.);\n";

    text += "#160=CYLINDRICAL_SURFACE('',#130,0.4);\n";
    text += "#161=CYLINDRICAL_SURFACE('',#132,0.4);\n";
    text += "#162=PLANE('',#134);\n";
    text += "#163=PLANE('',#135);\n";

    let [on_the_upright, on_the_leaning] = bend.mitre_senses;
    let faces: [(&str, &[(&str, &str)]); 4] = [
        (
            "#160",
            &[
                ("#150", ".T."),
                ("#153", ".T."),
                ("#151", on_the_upright),
                ("#153", ".F."),
            ],
        ),
        (
            "#161",
            &[
                ("#151", on_the_leaning),
                ("#154", ".T."),
                ("#152", ".F."),
                ("#154", ".F."),
            ],
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
