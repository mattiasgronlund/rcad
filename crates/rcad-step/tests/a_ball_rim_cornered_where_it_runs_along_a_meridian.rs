//! A closed edge square to the way its own surface winds, where the surface
//! is a ball cut through its own axis rather than across it.
//!
//! `a_rim_square_to_the_way_its_surface_winds.rs` is the cap case: a plane
//! across the ball's axis, so the rim is a line of latitude and the hint goes
//! square only at the one corner it happens to land on. Cut instead *through*
//! the axis and the rim is a whole meridian, lying entirely in a plane that
//! holds the world's `z` axis. A ball's longitude direction is always
//! `z × (point − centre)` (`Surface::the_way_its_first_coordinate_runs_at`),
//! which has no `z` component, while every tangent to a meridian stays inside
//! the plane that holds `z` — so the two are square **everywhere** on this
//! rim, corner included, and not just by where the one `VERTEX_POINT` sits.
//!
//! `which_way_round_a_closed_edge_is_stored` then tries
//! `the_way_a_closed_edge_runs_where_its_longitude_says_nothing`, which answers
//! only when the curved surface is a torus (`docs/decisions.md` §390) — a
//! sphere is not, so that also answers nothing, and the reader falls all the
//! way back to the file's own `ORIENTED_EDGE` flags. This file pins what the
//! reader does with both ways those flags can be written, each read under
//! `Policy::certified`, against the half-ball's closed form.

mod written_by_hand;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};
use rcad_topo::{Error, Invalid};
use written_by_hand::{a_header, real};

/// A half ball, flat face stored running forward and the round face
/// backward, imports and holds half the ball's volume.
#[test]
fn a_half_ball_imports_flat_face_forward() {
    let (ball, flat_runs_forward, round_runs_forward) = (4.0, true, false);
    let holds = 2.0 * core::f64::consts::PI * ball * ball * ball / 3.0;
    let text = a_half_ball_split_on_a_meridian(ball, flat_runs_forward, round_runs_forward);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let mut log = ProvenanceLog::new();
    let import = Import::read(&file, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("a half ball split on a meridian: {why}"));
    import
        .model()
        .validate(&Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("a half ball split on a meridian validates: {why}"));
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
        .unwrap_or_else(|why| panic!("a half ball split on a meridian measures: {why}"));
    assert!(
        (measured.volume() - holds).abs() <= measured.volume_error(),
        "a half ball of radius {ball} holds {holds}, and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The same half ball with the two faces' flags swapped is refused, not
/// silently read inside out.
///
/// A single closed edge with one corner has exactly two consistent ways to
/// split between its two uses, one for each face. The one that reads is
/// [`a_half_ball_imports_flat_face_forward`]'s; the other has every
/// `ORIENTED_EDGE` flag the file can write, each one legal on its own loop,
/// and still comes out with the two faces facing opposite ways round their
/// shared edge — caught, not corrupted: `Import::read` refuses the whole
/// solid rather than hand back one with a flipped face in it.
#[test]
fn a_half_ball_refuses_flat_face_backward() {
    let (ball, flat_runs_forward, round_runs_forward) = (4.0, false, true);
    let text = a_half_ball_split_on_a_meridian(ball, flat_runs_forward, round_runs_forward);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let mut log = ProvenanceLog::new();
    let Err(Refusal::TheSolidTheFileDescribesIsNotOne { why, .. }) =
        Import::read(&file, &Policy::certified(), &mut log)
    else {
        panic!("a half ball with its flags swapped reads two faces facing opposite ways");
    };
    assert!(
        matches!(
            *why,
            Error::Invalid(Invalid::FacesDoNotAgreeWhichWayTheyFace)
        ),
        "the flipped half ball was refused, but not by the two faces disagreeing which way \
         they face: {why}"
    );
}

/// A ball of `ball` radius on the origin, cut by the plane through its own
/// axis, keeping the half on the `y ≥ 0` side, in millimetres.
///
/// The ball's own placement puts its axis along the world's `z`, which is
/// what `Surface::the_way_its_first_coordinate_runs_at` always winds a
/// sphere's longitude about — so this is the axis the cutting plane has to
/// hold for the rim to be a meridian and not a parallel. The rim's one
/// `VERTEX_POINT` sits at `(ball, 0, 0)`, a quarter turn round from the
/// sphere's own reference direction `(0, 1, 0)` — as far round the longitude
/// as a point can sit before the two reference meridians either side of it
/// claim it instead.
///
/// `flat_runs_forward` and `round_runs_forward` set the `ORIENTED_EDGE` flag
/// on the flat face and the round face in turn. The one shared `EDGE_CURVE`
/// has to run opposite ways on its two uses for the solid to close, so the
/// two flags always come in here as opposites — this just lets a caller pick
/// which face is told to run which way, to check the reader takes either.
fn a_half_ball_split_on_a_meridian(
    ball: f64,
    flat_runs_forward: bool,
    round_runs_forward: bool,
) -> String {
    let ball = real(ball);
    let flat_flag = if flat_runs_forward { ".T." } else { ".F." };
    let round_flag = if round_runs_forward { ".T." } else { ".F." };
    let mut text = a_header(".MILLI.");
    text.push_str(&format!(
        "#10=CARTESIAN_POINT('',(0.,0.,0.));\n\
         #11=DIRECTION('',(0.,0.,1.));\n\
         #12=DIRECTION('',(0.,1.,0.));\n\
         #13=DIRECTION('',(0.,-1.,0.));\n\
         #14=DIRECTION('',(1.,0.,0.));\n\
         #20=AXIS2_PLACEMENT_3D('',#10,#12,#14);\n\
         #21=CIRCLE('',#20,{ball});\n\
         #22=CARTESIAN_POINT('',({ball},0.,0.));\n\
         #23=VERTEX_POINT('',#22);\n\
         #24=EDGE_CURVE('',#23,#23,#21,.T.);\n\
         #30=AXIS2_PLACEMENT_3D('',#10,#13,#14);\n\
         #31=PLANE('',#30);\n\
         #32=ORIENTED_EDGE('',*,*,#24,{flat_flag});\n\
         #33=EDGE_LOOP('',(#32));\n\
         #34=FACE_OUTER_BOUND('',#33,.T.);\n\
         #35=ADVANCED_FACE('',(#34),#31,.T.);\n\
         #40=AXIS2_PLACEMENT_3D('',#10,#11,#12);\n\
         #41=SPHERICAL_SURFACE('',#40,{ball});\n\
         #42=ORIENTED_EDGE('',*,*,#24,{round_flag});\n\
         #43=EDGE_LOOP('',(#42));\n\
         #44=FACE_OUTER_BOUND('',#43,.T.);\n\
         #45=ADVANCED_FACE('',(#44),#41,.T.);\n\
         #50=CLOSED_SHELL('',(#35,#45));\n\
         #51=MANIFOLD_SOLID_BREP('',#50);\n\
         ENDSEC;\n\
         END-ISO-10303-21;\n"
    ));
    text
}
