//! A half ball on a flat disc whose rim passes both poles of the ball in the
//! middle of its one edge, measured at several deflections.
//!
//! `docs/decisions.md` §193.3: a `Sphere` here always has its axis along `z`.
//! This dome has its axis along `−y`, so in the kernel's chart its rim is two
//! meridians half a turn apart, and its one corner `(r, 0, 0)` is on the
//! chart's equator. The rim therefore runs through both poles with no corner
//! at either. The mesher only knew a pole when a point walked along the edge
//! happened to land on it, which the import's coarse walk did and the finer
//! walks `measure` asks for did not.

mod written_by_hand;

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use written_by_hand::a_header;

const RADIUS: f64 = 1.5;

#[test]
fn a_rim_through_both_poles_measures_half_a_ball_at_a_fine_deflection() {
    measures_half_a_ball(0.001);
}

#[test]
fn a_rim_through_both_poles_measures_half_a_ball_at_a_coarse_deflection() {
    measures_half_a_ball(0.05);
}

#[test]
fn a_rim_through_both_poles_measures_half_a_ball_at_a_middling_deflection() {
    measures_half_a_ball(0.0137);
}

#[test]
fn a_rim_through_both_poles_measures_half_a_ball_at_a_very_fine_deflection() {
    measures_half_a_ball(0.0003);
}

fn measures_half_a_ball(deflection: f64) {
    let text = a_half_ball_with_its_corner_off_the_poles(RADIUS);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let import = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a half ball reads");
    let [(_, dome)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let measured = import
        .model()
        .solid(*dome)
        .expect("this model made the key")
        .measure(
            rcad_geom::Deflection::of(deflection).expect("a positive deflection"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("a half ball measures at {deflection}, not {why:?}"));

    let space = 2.0 / 3.0 * core::f64::consts::PI * RADIUS.powi(3);
    assert!(
        (measured.volume() - space).abs() <= measured.volume_error(),
        "at {deflection}, half a ball encloses {space} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// Half a ball on a flat disc, the ball's axis `−y` and the dome on the `−y`
/// side, with no seam: the dome is bounded by the rim alone, a circle in
/// `y = 0` whose one corner is `(r, 0, 0)`, on the kernel chart's equator.
///
/// A copy of the writer in `a_dome_on_a_flat_disc_imports.rs`.
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
