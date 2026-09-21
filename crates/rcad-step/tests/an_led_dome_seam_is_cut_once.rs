//! A half ball whose file cut it open along a seam that, in this kernel's own
//! coordinates on the ball, is a slit.
//!
//! `docs/decisions.md` §192.2: every KiCad `LED_D*_Horizontal` dome is a half
//! ball written on a `SPHERICAL_SURFACE` whose axis points out of the dome, and
//! its wire is the rim, then a seam from the rim to the top of the dome, then
//! the same seam back. In the file's own chart the seam runs from the equator
//! to a pole and its two uses are a whole turn apart. A `Sphere` here always
//! has its axis along `z`, so the top of the dome is an ordinary point on the
//! equator and the seam is a line into the face that the wire goes out along
//! and straight back: the same points twice. The triangulator refused that
//! with `CornersCoincide`.
//!
//! The file below is that dome with nothing else: the LED's dome and its rim,
//! at the origin, closed by a flat disc.

mod written_by_hand;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use written_by_hand::a_header;

const RADIUS: f64 = 1.5;
const LENGTH: f64 = 2.0;

/// The dome imports, and the solid encloses a bar and half a ball. The volume
/// is what says the two pieces the dome is cut into cover it once each, rather
/// than overlapping or leaving a gap.
#[test]
fn a_dome_cut_open_along_a_slit_imports_and_measures_half_a_ball() {
    let text = a_half_ball_on_a_bar(RADIUS, LENGTH);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();

    let import = Import::read(&file, &policy, &mut log).expect("a domed bar reads");
    let [(_, dome)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let measured = import
        .model()
        .solid(*dome)
        .expect("this model made the key")
        .measure(
            rcad_geom::Deflection::of(0.001).expect("a fine mesh"),
            &policy,
            &mut log,
        )
        .expect("a domed bar measures");

    let space = core::f64::consts::PI * RADIUS * RADIUS * (2.0 / 3.0 * RADIUS + LENGTH);
    assert!(
        (measured.volume() - space).abs() <= measured.volume_error(),
        "a bar and half a ball enclose {space} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The dome of `kicad/LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step` (`#553`) moved
/// to the origin, on a round bar of its own radius `length` long, closed by a
/// flat disc.
///
/// The ball's axis is `−y` and the dome is on the `−y` side, so the top of the
/// dome is `(0, −r, 0)`. The rim is a circle in `y = 0` with one corner at
/// `(0, 0, r)`, and the seam is a quarter circle in `x = 0` from that corner to
/// the top of the dome. The bar is there because the LED's is: the rim is
/// where the bar and the ball touch, and a rim the reader checks that way is
/// the one it gives its second corner at the far pole (§192.1).
fn a_half_ball_on_a_bar(radius: f64, length: f64) -> String {
    let r = radius;
    let mut text = a_header(".MILLI.");
    text += &format!(
        "#10=CARTESIAN_POINT('',(0.,0.,0.));\n\
         #11=DIRECTION('',(0.,-1.,0.));\n\
         #12=DIRECTION('',(0.,0.,1.));\n\
         #13=DIRECTION('',(1.,0.,0.));\n\
         #14=DIRECTION('',(0.,1.,0.));\n\
         #20=CARTESIAN_POINT('',(0.,0.,{r:?}));\n\
         #21=VERTEX_POINT('',#20);\n\
         #22=CARTESIAN_POINT('',(0.,{top:?},0.));\n\
         #23=VERTEX_POINT('',#22);\n\
         #30=AXIS2_PLACEMENT_3D('',#10,#11,#12);\n\
         #31=CIRCLE('',#30,{r:?});\n\
         #32=EDGE_CURVE('',#21,#21,#31,.T.);\n\
         #40=AXIS2_PLACEMENT_3D('',#10,#13,#14);\n\
         #41=CIRCLE('',#40,{r:?});\n\
         #42=EDGE_CURVE('',#21,#23,#41,.T.);\n\
         #50=ORIENTED_EDGE('',*,*,#32,.T.);\n\
         #51=ORIENTED_EDGE('',*,*,#42,.T.);\n\
         #52=ORIENTED_EDGE('',*,*,#42,.F.);\n\
         #53=EDGE_LOOP('',(#50,#51,#52));\n\
         #54=FACE_BOUND('',#53,.T.);\n\
         #55=AXIS2_PLACEMENT_3D('',#10,#11,#12);\n\
         #56=SPHERICAL_SURFACE('',#55,{r:?});\n\
         #57=ADVANCED_FACE('',(#54),#56,.T.);\n\
         #15=CARTESIAN_POINT('',(0.,{length:?},0.));\n\
         #24=CARTESIAN_POINT('',(0.,{length:?},{r:?}));\n\
         #25=VERTEX_POINT('',#24);\n\
         #33=VECTOR('',#14,1.);\n\
         #34=LINE('',#20,#33);\n\
         #35=EDGE_CURVE('',#21,#25,#34,.T.);\n\
         #36=AXIS2_PLACEMENT_3D('',#15,#11,#12);\n\
         #37=CIRCLE('',#36,{r:?});\n\
         #38=EDGE_CURVE('',#25,#25,#37,.T.);\n\
         #60=ORIENTED_EDGE('',*,*,#32,.F.);\n\
         #61=ORIENTED_EDGE('',*,*,#35,.T.);\n\
         #62=ORIENTED_EDGE('',*,*,#38,.T.);\n\
         #63=ORIENTED_EDGE('',*,*,#35,.F.);\n\
         #64=EDGE_LOOP('',(#60,#61,#62,#63));\n\
         #65=FACE_BOUND('',#64,.T.);\n\
         #66=AXIS2_PLACEMENT_3D('',#10,#11,#12);\n\
         #67=CYLINDRICAL_SURFACE('',#66,{r:?});\n\
         #68=ADVANCED_FACE('',(#65),#67,.T.);\n\
         #80=ORIENTED_EDGE('',*,*,#38,.F.);\n\
         #81=EDGE_LOOP('',(#80));\n\
         #82=FACE_BOUND('',#81,.T.);\n\
         #83=AXIS2_PLACEMENT_3D('',#15,#14,#13);\n\
         #84=PLANE('',#83);\n\
         #85=ADVANCED_FACE('',(#82),#84,.T.);\n\
         #90=CLOSED_SHELL('',(#57,#68,#85));\n\
         #91=MANIFOLD_SOLID_BREP('',#90);\n",
        top = -r
    );
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}
