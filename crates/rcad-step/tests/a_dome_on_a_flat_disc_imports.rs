//! A half ball closed by a flat disc, whose rim runs through the poles of the
//! ball.
//!
//! `docs/decisions.md` §192.1 and §193.3: a `Sphere` here always has its axis
//! along `z`. Every dome below has its axis along `−y`, so in the kernel's
//! chart its rim is two meridians half a turn apart and passes through both
//! poles. When the rim is also the edge of a bar, the reader gives it a second
//! corner at the opposite point and the mesher meets each pole as a corner.
//! Closed by a flat disc, the rim gets the same cut, because its one corner is
//! a pole and no rule reads which way a closed edge runs there.
//!
//! Three files:
//!
//! - the LED's own rim, one corner on the north pole: the reader cuts it at
//!   the south pole, and it imports and measures half a ball;
//! - the same rim written already cut in two at the south pole: imports and
//!   measures half a ball;
//! - a rim with its one corner on the chart's equator, which says which way it
//!   runs and passes both poles in the middle of the edge: imports and
//!   measures half a ball.

mod written_by_hand;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use written_by_hand::a_header;

const RADIUS: f64 = 1.5;

/// The half ball the LED dome would be on a flat disc. The rim's one corner is
/// the ball's pole, where the longitude has no direction, so a closed edge
/// there says nothing about which way it runs and the mesher walked it
/// backwards (`AWireWillNotLieFlatOnItsFace`). The reader now cuts such a rim
/// in two at the opposite point, the way
/// [`a_rim_cut_in_two_at_the_far_pole_imports_and_measures_half_a_ball`]
/// writes it by hand (§193.3).
#[test]
fn a_rim_with_its_one_corner_on_a_pole_imports_and_measures_half_a_ball() {
    let import = imports(&a_half_ball_on_a_flat_disc(RADIUS));
    measures_half_a_ball(&import);
}

/// The rim cut in two at the south pole: the direction comes from the two
/// corners, and both poles are corners.
#[test]
fn a_rim_cut_in_two_at_the_far_pole_imports_and_measures_half_a_ball() {
    let import = imports(&a_half_ball_with_a_corner_at_each_pole(RADIUS));
    measures_half_a_ball(&import);
}

/// A rim whose one corner is off the poles passes both in the middle of its
/// one edge. The edge's walk now holds each pole as one of its points, and
/// the wire walk opens that pole's side the way it opens a corner's. Before,
/// the walk took each pole the short way round, and at exactly half a turn
/// that came back a whole turn off: refused by
/// `ACurvedFaceGoesRightRoundWithNoSeam`. `a_rim_through_both_poles_measures.rs`
/// measures the same half ball at four deflections (§194).
#[test]
fn a_rim_that_passes_both_poles_in_the_middle_of_its_edge_imports_and_measures_half_a_ball() {
    let import = imports(&a_half_ball_with_its_corner_off_the_poles(RADIUS));
    measures_half_a_ball(&import);
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
