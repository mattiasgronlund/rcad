//! Q541: a run between two different curved faces used to store as its second
//! surface the ball if there was one and otherwise `faces[0]` — the first face
//! the file's shell happened to list. `a_face_where_curved_faces_meet` now
//! picks with `the_curved_one_of`, which sorts on the faces' own numbers
//! instead, so the same shape read with its faces listed either way stores the
//! same run.
//!
//! The shape: a pipe of radius 3 flaring into a cone, its rim cut into two
//! half circles so the STEP file writes it as two closed `EDGE_CURVE`s rather
//! than one arc — the same construction `a_flared_bar_reads_whichever_way_its_pipe_points.rs`
//! uses whole. The rim's `CIRCLE` is written `3.000000000001`, a hair off the
//! pipe's exact radius, so the plane the rim's run could otherwise carry is
//! refused (`Walk::every_face_holds_the_curve`) and the run falls back to
//! naming one of its own two faces — the fallback this row changed.

mod written_by_hand;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use written_by_hand::{a_header, real};

/// The rim's two corners, where the cone, listed first or second, must not
/// change which run the model stores for either half.
const RIM: [[f64; 3]; 2] = [[3.0, 0.0, 4.0], [-3.0, 0.0, 4.0]];

#[test]
fn a_flared_bars_rim_runs_are_the_same_cone_first_or_pipe_first() {
    let cone_first = read_it(&a_flared_bar_with_a_split_rim(true));
    let pipe_first = read_it(&a_flared_bar_with_a_split_rim(false));
    assert_eq!(
        cone_first, pipe_first,
        "the rim's two runs should not depend on which face the file lists first"
    );
}

/// Both rim edges' runs, by their endpoints so the order the model happens to
/// walk them in does not matter.
fn read_it(text: &str) -> Vec<(String, String)> {
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let import = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a flared bar with a split rim reads");
    let model = import.model();
    let mut runs: Vec<(String, String)> = model
        .edges()
        .filter_map(|edge| {
            let ends = [
                edge.start().point().coordinates(),
                edge.end().point().coordinates(),
            ];
            let at_the_rim = (ends[0] == RIM[0] && ends[1] == RIM[1])
                || (ends[0] == RIM[1] && ends[1] == RIM[0]);
            at_the_rim.then(|| (format!("{ends:?}"), format!("{:?}", edge.run())))
        })
        .collect();
    assert_eq!(
        runs.len(),
        2,
        "the rim is cut into exactly two half circles"
    );
    runs.sort();
    runs
}

/// A pipe of radius 3 from `z = 0` to `z = 4`, flaring into a cone to radius 6
/// at `z = 8`, with the pipe-to-cone rim cut into two half circles at
/// `(3,0,4)` and `(-3,0,4)`.
///
/// The rim's `CIRCLE` is written `3.000000000001`, off the pipe's exact
/// radius `3`, so `Walk::every_face_holds_the_curve` refuses the plane and the
/// run falls back to naming one of the cone or the pipe — the pick
/// `a_face_where_curved_faces_meet` makes.
fn a_flared_bar_with_a_split_rim(cone_first: bool) -> String {
    let mut t = a_header(".MILLI.");
    t += "#20=DIRECTION('',(0.,0.,1.));\n";
    t += "#21=DIRECTION('',(1.,0.,0.));\n";
    t += "#22=DIRECTION('',(0.,0.,-1.));\n";
    t += "#24=DIRECTION('',(3.,0.,4.));\n";
    t += &format!("#23=VECTOR('',#24,{});\n", real(1.0));
    t += &format!("#26=VECTOR('',#20,{});\n", real(1.0));
    t += "#100=CARTESIAN_POINT('',(3.,0.,4.));\n";
    t += "#101=CARTESIAN_POINT('',(-3.,0.,4.));\n";
    t += "#102=CARTESIAN_POINT('',(6.,0.,8.));\n";
    t += "#103=CARTESIAN_POINT('',(3.,0.,0.));\n";
    t += "#110=VERTEX_POINT('',#100);\n";
    t += "#111=VERTEX_POINT('',#101);\n";
    t += "#112=VERTEX_POINT('',#102);\n";
    t += "#113=VERTEX_POINT('',#103);\n";
    t += "#120=CARTESIAN_POINT('',(0.,0.,4.));\n";
    t += "#121=CARTESIAN_POINT('',(0.,0.,8.));\n";
    t += "#122=CARTESIAN_POINT('',(0.,0.,0.));\n";
    t += "#130=AXIS2_PLACEMENT_3D('',#120,#20,#21);\n";
    t += "#131=AXIS2_PLACEMENT_3D('',#121,#20,#21);\n";
    t += "#132=AXIS2_PLACEMENT_3D('',#122,#20,#21);\n";
    t += "#140=CIRCLE('',#130,3.000000000001);\n";
    t += "#141=CIRCLE('',#131,6.);\n";
    t += "#142=CIRCLE('',#132,3.);\n";
    t += "#150=EDGE_CURVE('',#110,#111,#140,.T.);\n";
    t += "#151=EDGE_CURVE('',#111,#110,#140,.T.);\n";
    t += "#152=EDGE_CURVE('',#112,#112,#141,.T.);\n";
    t += "#153=EDGE_CURVE('',#113,#113,#142,.T.);\n";
    t += "#160=LINE('',#100,#23);\n";
    t += "#170=EDGE_CURVE('',#110,#112,#160,.T.);\n";
    t += "#161=LINE('',#103,#26);\n";
    t += "#171=EDGE_CURVE('',#113,#110,#161,.T.);\n";

    t += &format!(
        "#210=CONICAL_SURFACE('',#130,3.,{});\n",
        real(0.75_f64.atan())
    );
    t += "#220=ORIENTED_EDGE('',*,*,#150,.T.);\n";
    t += "#221=ORIENTED_EDGE('',*,*,#151,.T.);\n";
    t += "#222=ORIENTED_EDGE('',*,*,#170,.T.);\n";
    t += "#223=ORIENTED_EDGE('',*,*,#152,.F.);\n";
    t += "#224=ORIENTED_EDGE('',*,*,#170,.F.);\n";
    t += "#230=EDGE_LOOP('',(#220,#221,#222,#223,#224));\n";
    t += "#235=FACE_OUTER_BOUND('',#230,.T.);\n";
    t += "#240=ADVANCED_FACE('',(#235),#210,.T.);\n";

    t += "#133=AXIS2_PLACEMENT_3D('',#122,#20,#21);\n";
    t += "#211=CYLINDRICAL_SURFACE('',#133,3.);\n";
    t += "#241=ORIENTED_EDGE('',*,*,#153,.T.);\n";
    t += "#242=ORIENTED_EDGE('',*,*,#171,.T.);\n";
    t += "#243=ORIENTED_EDGE('',*,*,#151,.F.);\n";
    t += "#244=ORIENTED_EDGE('',*,*,#150,.F.);\n";
    t += "#245=ORIENTED_EDGE('',*,*,#171,.F.);\n";
    t += "#246=EDGE_LOOP('',(#241,#242,#243,#244,#245));\n";
    t += "#247=FACE_OUTER_BOUND('',#246,.T.);\n";
    t += "#253=ADVANCED_FACE('',(#247),#211,.T.);\n";

    t += "#261=PLANE('',#131);\n";
    t += "#262=ORIENTED_EDGE('',*,*,#152,.T.);\n";
    t += "#263=EDGE_LOOP('',(#262));\n";
    t += "#264=FACE_OUTER_BOUND('',#263,.T.);\n";
    t += "#265=ADVANCED_FACE('',(#264),#261,.T.);\n";

    t += "#134=AXIS2_PLACEMENT_3D('',#122,#22,#21);\n";
    t += "#266=PLANE('',#134);\n";
    t += "#267=ORIENTED_EDGE('',*,*,#153,.F.);\n";
    t += "#268=EDGE_LOOP('',(#267));\n";
    t += "#269=FACE_OUTER_BOUND('',#268,.T.);\n";
    t += "#270=ADVANCED_FACE('',(#269),#266,.T.);\n";

    t += if cone_first {
        "#500=CLOSED_SHELL('',(#240,#253,#265,#270));\n"
    } else {
        "#500=CLOSED_SHELL('',(#253,#240,#265,#270));\n"
    };
    t += "#501=MANIFOLD_SOLID_BREP('',#500);\n";
    t + "ENDSEC;\nEND-ISO-10303-21;\n"
}
