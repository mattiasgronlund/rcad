//! A half ball closed by a flat disc, whose rim runs through the poles of the
//! ball.
//!
//! `docs/decisions.md` §192.1 and §193.3: a `Sphere` here always has its axis
//! along `z`. Every dome below has its axis along `−y`, so in the kernel's
//! chart its rim is two meridians half a turn apart and passes through both
//! poles. When the rim is also the edge of a bar, the reader gives it a second
//! corner at the opposite point and the mesher meets each pole as a corner.
//! Closed by a flat disc, the rim keeps the one corner the file gave it.
//!
//! Three files, because the flat disc stops on two different things:
//!
//! - the LED's own rim, one corner on the north pole: no rule reads which way
//!   a closed edge runs at a pole, so the rim is walked in whatever order the
//!   walker produced it, and here that is backwards;
//! - the same rim cut in two at the south pole, which is what the reader does
//!   against a bar: imports and measures half a ball;
//! - a rim with its one corner on the chart's equator, which says which way it
//!   runs and passes both poles in the middle of the edge.

mod written_by_hand;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};
use written_by_hand::a_header;

const RADIUS: f64 = 1.5;

/// The half ball the LED dome would be on a flat disc is still refused. What
/// is missing is a direction, not a pole: [`a_rim_cut_in_two_at_the_far_pole_imports_and_measures_half_a_ball`]
/// is the same shape with the rim written as two halves, and it imports.
///
/// The rim's one corner is the ball's pole, where `∇S × ∇T` is square to
/// both the longitude and the latitude, and the reader's hint reads nothing
/// there either and falls back to the file's flag. A closed edge between a
/// plane and a ball carries no surface that names its direction, so the model
/// holds none for this rim, and the mesher walks it the wrong way round. The
/// fix is the reader cutting the rim at the opposite point, as it already does
/// for a rim it checks against a bar (§193.3).
#[test]
fn a_rim_with_its_one_corner_on_a_pole_is_refused_for_want_of_a_direction() {
    let text = a_half_ball_on_a_flat_disc(RADIUS);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();

    let Err(Refusal::TheSolidTheFileDescribesIsNotOne { why, .. }) =
        Import::read(&file, &policy, &mut log)
    else {
        panic!("the rim with one corner on the pole has no direction the mesher can read");
    };
    assert!(
        matches!(*why, rcad_topo::Error::AWireWillNotLieFlatOnItsFace { .. }),
        "the dome's wire walked the wrong way round does not close, and not {why:?}"
    );
}

/// The rim cut in two at the south pole: the direction comes from the two
/// corners, and both poles are corners.
#[test]
fn a_rim_cut_in_two_at_the_far_pole_imports_and_measures_half_a_ball() {
    let import = imports(&a_half_ball_with_a_corner_at_each_pole(RADIUS));
    measures_half_a_ball(&import);
}

/// A rim whose one corner is off the poles passes both in the middle of its
/// one edge. When a point walked along the edge lands on a pole, the walk
/// opens that pole's side the way it opens a corner's, so the dome's wire lies
/// flat and the import's own mesh closes. Before, the walk took each pole the
/// short way round, and at exactly half a turn that came back a whole turn
/// off: refused by `ACurvedFaceGoesRightRoundWithNoSeam`.
///
/// Only the import is asserted. The finer walk `measure` asks for steps past
/// both poles without landing on either, which this does not reach (§193.3).
#[test]
fn a_rim_that_passes_both_poles_in_the_middle_of_its_edge_imports() {
    imports(&a_half_ball_with_its_corner_off_the_poles(RADIUS));
}

fn imports(text: &str) -> Import {
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new()).expect("a half ball reads")
}

fn measures_half_a_ball(import: &Import) {
    let [(_, dome)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let measured = import
        .model()
        .solid(*dome)
        .expect("this model made the key")
        .measure(
            rcad_geom::Deflection::of(0.001).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a half ball measures");

    let space = 2.0 / 3.0 * core::f64::consts::PI * RADIUS.powi(3);
    assert!(
        (measured.volume() - space).abs() <= measured.volume_error(),
        "half a ball encloses {space} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The dome of `an_led_dome_seam_is_cut_once.rs` with the bar taken away: the
/// ball's axis is `−y`, the dome is on the `−y` side, the rim is a circle in
/// `y = 0` with its one corner at `(0, 0, r)`, and the seam is a quarter
/// circle in `x = 0` from that corner to the top of the dome. The rim also
/// bounds a flat disc in `y = 0` facing `+y`.
fn a_half_ball_on_a_flat_disc(radius: f64) -> String {
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
         #80=ORIENTED_EDGE('',*,*,#32,.F.);\n\
         #81=EDGE_LOOP('',(#80));\n\
         #82=FACE_BOUND('',#81,.T.);\n\
         #83=AXIS2_PLACEMENT_3D('',#10,#14,#13);\n\
         #84=PLANE('',#83);\n\
         #85=ADVANCED_FACE('',(#82),#84,.T.);\n\
         #90=CLOSED_SHELL('',(#57,#85));\n\
         #91=MANIFOLD_SOLID_BREP('',#90);\n",
        top = -r
    );
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}

/// [`a_half_ball_on_a_flat_disc`] with the rim cut at `(0, 0, −r)` into two
/// edges on the same circle.
fn a_half_ball_with_a_corner_at_each_pole(radius: f64) -> String {
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
         #24=CARTESIAN_POINT('',(0.,0.,{bottom:?}));\n\
         #25=VERTEX_POINT('',#24);\n\
         #30=AXIS2_PLACEMENT_3D('',#10,#11,#12);\n\
         #31=CIRCLE('',#30,{r:?});\n\
         #32=EDGE_CURVE('',#21,#25,#31,.T.);\n\
         #33=EDGE_CURVE('',#25,#21,#31,.T.);\n\
         #40=AXIS2_PLACEMENT_3D('',#10,#13,#14);\n\
         #41=CIRCLE('',#40,{r:?});\n\
         #42=EDGE_CURVE('',#21,#23,#41,.T.);\n\
         #50=ORIENTED_EDGE('',*,*,#32,.T.);\n\
         #49=ORIENTED_EDGE('',*,*,#33,.T.);\n\
         #51=ORIENTED_EDGE('',*,*,#42,.T.);\n\
         #52=ORIENTED_EDGE('',*,*,#42,.F.);\n\
         #53=EDGE_LOOP('',(#50,#49,#51,#52));\n\
         #54=FACE_BOUND('',#53,.T.);\n\
         #55=AXIS2_PLACEMENT_3D('',#10,#11,#12);\n\
         #56=SPHERICAL_SURFACE('',#55,{r:?});\n\
         #57=ADVANCED_FACE('',(#54),#56,.T.);\n\
         #80=ORIENTED_EDGE('',*,*,#33,.F.);\n\
         #79=ORIENTED_EDGE('',*,*,#32,.F.);\n\
         #81=EDGE_LOOP('',(#80,#79));\n\
         #82=FACE_BOUND('',#81,.T.);\n\
         #83=AXIS2_PLACEMENT_3D('',#10,#14,#13);\n\
         #84=PLANE('',#83);\n\
         #85=ADVANCED_FACE('',(#82),#84,.T.);\n\
         #90=CLOSED_SHELL('',(#57,#85));\n\
         #91=MANIFOLD_SOLID_BREP('',#90);\n",
        top = -r,
        bottom = -r
    );
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}

/// Half a ball on a flat disc, the ball's axis `−y` and the dome on the `−y`
/// side, with no seam: the dome is bounded by the rim alone, a circle in
/// `y = 0` whose one corner is `(r, 0, 0)`, on the kernel chart's equator.
fn a_half_ball_with_its_corner_off_the_poles(radius: f64) -> String {
    let r = radius;
    let mut text = a_header(".MILLI.");
    text += &format!(
        "#10=CARTESIAN_POINT('',(0.,0.,0.));\n\
         #11=DIRECTION('',(0.,-1.,0.));\n\
         #12=DIRECTION('',(1.,0.,0.));\n\
         #14=DIRECTION('',(0.,1.,0.));\n\
         #20=CARTESIAN_POINT('',({r:?},0.,0.));\n\
         #21=VERTEX_POINT('',#20);\n\
         #30=AXIS2_PLACEMENT_3D('',#10,#11,#12);\n\
         #31=CIRCLE('',#30,{r:?});\n\
         #32=EDGE_CURVE('',#21,#21,#31,.T.);\n\
         #50=ORIENTED_EDGE('',*,*,#32,.T.);\n\
         #53=EDGE_LOOP('',(#50));\n\
         #54=FACE_BOUND('',#53,.T.);\n\
         #55=AXIS2_PLACEMENT_3D('',#10,#11,#12);\n\
         #56=SPHERICAL_SURFACE('',#55,{r:?});\n\
         #57=ADVANCED_FACE('',(#54),#56,.T.);\n\
         #80=ORIENTED_EDGE('',*,*,#32,.F.);\n\
         #81=EDGE_LOOP('',(#80));\n\
         #82=FACE_BOUND('',#81,.T.);\n\
         #83=AXIS2_PLACEMENT_3D('',#10,#14,#12);\n\
         #84=PLANE('',#83);\n\
         #85=ADVANCED_FACE('',(#82),#84,.T.);\n\
         #90=CLOSED_SHELL('',(#57,#85));\n\
         #91=MANIFOLD_SOLID_BREP('',#90);\n"
    );
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}
