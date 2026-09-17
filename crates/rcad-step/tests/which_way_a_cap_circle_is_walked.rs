//! A cap circle is walked the way this kernel's rule says, and not the way the
//! file's own flag happens to read.
//!
//! `ORIENTED_EDGE.orientation` says which way an edge runs **between its two
//! corners**. A cap circle has one corner, so the flag says nothing about it,
//! and this kernel gives [`Orientation`](rcad_geom::Orientation) its other
//! meaning there: the way the curved surface either side of the edge winds
//! (`docs/decisions.md` §115). The two are different questions and the file
//! answers only the first.
//!
//! **The bar below is written twice and the two files describe one solid.** The
//! only difference between them is which way round the wall's own axis is
//! spelled, and a cylinder is the same surface either way — the same point set,
//! the same outward normal, the same material inside it. So both files have to
//! import and both have to enclose `πr²h`.
//!
//! What makes that a test rather than a tautology is that the kernel's rule
//! *does* turn round with the wall's axis, while every flag in the file stays
//! put. A reader that hands `ORIENTED_EDGE.orientation` straight through gives
//! both spellings the same answer, so it has to be wrong about one of them —
//! and `Model::validate` catches it by name, as
//! `a closed edge is walked against its surface`.
//!
//! **The two volumes are asked to agree within the measurement's own error and
//! not bit for bit.** They came out two ulps apart when this was written, which
//! is the mesher summing one bar's faces in the order one spelling puts them in
//! and the other's in the other. Two ulps is not a fact about the bar, and a
//! test that pinned them would be measuring the summation order.

mod written_by_hand;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use written_by_hand::{a_header, real};

/// A round bar imports whichever way round its wall's axis is written, and both
/// spellings enclose the same bar.
#[test]
fn a_round_bar_imports_whichever_way_round_its_wall_axis_is_written() {
    let (radius, height) = (3.0, 8.0);
    let holds = core::f64::consts::PI * radius * radius * height;
    let mut both = Vec::new();
    for the_wall_axis_points_up in [true, false] {
        let spelling = if the_wall_axis_points_up {
            "with its wall axis up"
        } else {
            "with its wall axis down"
        };
        let text = a_round_bar(radius, height, the_wall_axis_points_up);
        let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
        let mut log = ProvenanceLog::new();
        let import = Import::read(&file, &Policy::certified(), &mut log)
            .unwrap_or_else(|why| panic!("a round bar {spelling}: {why}"));
        import
            .model()
            .validate(&Policy::certified(), &mut log)
            .unwrap_or_else(|why| panic!("a round bar {spelling} validates: {why}"));
        let [(_, key)] = import.solids() else {
            panic!("one MANIFOLD_SOLID_BREP is one solid");
        };
        let measured = import
            .model()
            .solid(*key)
            .expect("this model made the key")
            .measure(
                Deflection::for_scale(import.model().scale()),
                &Policy::certified(),
                &mut log,
            )
            .unwrap_or_else(|why| panic!("a round bar {spelling} measures: {why}"));
        assert!(
            (measured.volume() - holds).abs() <= measured.volume_error(),
            "a round bar of radius {radius} and height {height} encloses {holds}, and the one \
             {spelling} says {} give or take {}",
            measured.volume(),
            measured.volume_error()
        );
        both.push((measured.volume(), measured.volume_error()));
    }
    let ((one, room), (other, _)) = (both[0], both[1]);
    assert!(
        (one - other).abs() <= room,
        "one bar written two ways is one bar, and these two measure {one} and {other}"
    );
}

/// A round bar of `radius` standing `height` tall on the origin: a disc at each
/// end and a cylindrical wall between them, in millimetres.
///
/// Both cap circles are **closed edges** — one `VERTEX_POINT` each, at
/// `(r, 0, 0)` and `(r, 0, h)` — which is the shape this file is written for.
///
/// `the_wall_axis_points_up` is the whole of what varies. A
/// `CYLINDRICAL_SURFACE` written about `(0, 0, −1)` is the same surface as one
/// written about `(0, 0, 1)`: a cylinder's outward normal is radial and carries
/// no sign of its axis, so the face still faces out of the bar and
/// `same_sense` stays `.T.` in both. Nothing else in the file moves.
///
/// Every loop below winds the way a right hand turns about its own face's
/// outward normal, which is what ISO 10303-42 asks of an outer bound. The
/// bottom cap faces `(0, 0, −1)` and so walks its circle **against** the
/// parameter, the top cap faces `(0, 0, 1)` and walks its own **with** it, and
/// the wall takes each the opposite way from the cap that shares it.
fn a_round_bar(radius: f64, height: f64, the_wall_axis_points_up: bool) -> String {
    let (r, h) = (real(radius), real(height));
    let wall_axis = if the_wall_axis_points_up {
        "#11"
    } else {
        "#13"
    };
    let mut text = a_header(".MILLI.");
    text.push_str(&format!(
        "#10=CARTESIAN_POINT('',(0.,0.,0.));\n\
         #11=DIRECTION('',(0.,0.,1.));\n\
         #12=DIRECTION('',(1.,0.,0.));\n\
         #13=DIRECTION('',(0.,0.,-1.));\n\
         #20=AXIS2_PLACEMENT_3D('',#10,#11,#12);\n\
         #21=CIRCLE('',#20,{r});\n\
         #22=CARTESIAN_POINT('',({r},0.,0.));\n\
         #23=VERTEX_POINT('',#22);\n\
         #24=EDGE_CURVE('',#23,#23,#21,.T.);\n\
         #30=CARTESIAN_POINT('',(0.,0.,{h}));\n\
         #31=AXIS2_PLACEMENT_3D('',#30,#11,#12);\n\
         #32=CIRCLE('',#31,{r});\n\
         #33=CARTESIAN_POINT('',({r},0.,{h}));\n\
         #34=VERTEX_POINT('',#33);\n\
         #35=EDGE_CURVE('',#34,#34,#32,.T.);\n\
         #40=AXIS2_PLACEMENT_3D('',#10,#13,#12);\n\
         #41=PLANE('',#40);\n\
         #42=ORIENTED_EDGE('',*,*,#24,.F.);\n\
         #43=EDGE_LOOP('',(#42));\n\
         #44=FACE_OUTER_BOUND('',#43,.T.);\n\
         #45=ADVANCED_FACE('',(#44),#41,.T.);\n\
         #50=AXIS2_PLACEMENT_3D('',#30,#11,#12);\n\
         #51=PLANE('',#50);\n\
         #52=ORIENTED_EDGE('',*,*,#35,.T.);\n\
         #53=EDGE_LOOP('',(#52));\n\
         #54=FACE_OUTER_BOUND('',#53,.T.);\n\
         #55=ADVANCED_FACE('',(#54),#51,.T.);\n\
         #60=AXIS2_PLACEMENT_3D('',#10,{wall_axis},#12);\n\
         #61=CYLINDRICAL_SURFACE('',#60,{r});\n\
         #62=ORIENTED_EDGE('',*,*,#24,.T.);\n\
         #63=EDGE_LOOP('',(#62));\n\
         #64=FACE_OUTER_BOUND('',#63,.T.);\n\
         #65=ORIENTED_EDGE('',*,*,#35,.F.);\n\
         #66=EDGE_LOOP('',(#65));\n\
         #67=FACE_BOUND('',#66,.T.);\n\
         #68=ADVANCED_FACE('',(#64,#67),#61,.T.);\n\
         #70=CLOSED_SHELL('',(#45,#55,#68));\n\
         #71=MANIFOLD_SOLID_BREP('',#70);\n\
         ENDSEC;\n\
         END-ISO-10303-21;\n"
    ));
    text
}
