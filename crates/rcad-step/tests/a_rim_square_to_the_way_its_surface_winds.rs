//! A closed edge square to the way its own surface winds still gets told apart
//! from its other half.
//!
//! A cap circle has one corner, so its ends say nothing about which way it runs
//! and only a rule can. This reader's rule reads the sign of the edge's travel
//! against the curved surface's winding — `docs/decisions.md` §115 — and that
//! sign is a *hint*. The two uses of one closed edge travel exact opposite
//! ways, so the hint normally tells them apart all by itself.
//!
//! **Unless the sign is zero, and then it does not.** The cap below is sliced
//! off a ball by an upright plane, so its rim runs up and over while a ball
//! winds round the equator, and at the rim's one corner the two are exactly
//! square. A hint that answers `Forward` at zero answers `Forward` for **both**
//! uses, and `SolidBuilder` refuses the whole solid by `both coedges of the
//! edge ... run the same way round`. That refusal is provably wrong: one of two
//! uses of an edge has to be the other way.
//!
//! This is not a shape dug up to make a test. It is what stops 26 files of the
//! STEP corpus, every one of them a bend of pipe capped by a plane, whose cap
//! circle runs round the **ring** while
//! `Surface::the_way_its_first_coordinate_runs_at` on a bend runs round the
//! **tube** — square to it at every corner of that circle.

mod written_by_hand;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use written_by_hand::{a_header, real};

/// A cap sliced off a ball by an upright plane imports, and holds what the
/// spherical-cap formula says it holds.
#[test]
fn a_cap_sliced_off_a_ball_sideways_imports() {
    let (ball, cut) = (5.0, 3.0);
    let tall = ball - cut;
    let holds = core::f64::consts::PI * tall * tall * (3.0 * ball - tall) / 3.0;
    let text = a_cap_sliced_off_a_ball(ball, cut);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let mut log = ProvenanceLog::new();
    let import = Import::read(&file, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("a cap sliced off a ball: {why}"));
    import
        .model()
        .validate(&Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("a cap sliced off a ball validates: {why}"));
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
        .unwrap_or_else(|why| panic!("a cap sliced off a ball measures: {why}"));
    assert!(
        (measured.volume() - holds).abs() <= measured.volume_error(),
        "a cap {tall} tall off a ball of radius {ball} holds {holds}, and this one says {} give \
         or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// What a ball of `ball` radius on the origin leaves on the far side of the
/// upright plane `x = cut`, in millimetres.
///
/// The rim is the circle the plane cuts, of radius `√(ball² − cut²)`, written
/// about the axis `(1, 0, 0)` from the reference direction `(0, 1, 0)`. Its one
/// `VERTEX_POINT` sits where that circle is furthest round towards `+y`, which
/// is the corner where the rim's travel and the ball's winding are square: the
/// travel is `(0, 0, ±ρ)` and the winding `ẑ × corner` is `(−ρ, cut, 0)`, and
/// nothing in that dot product is even rounded.
///
/// Neither pole is on this cap and the cap does not go right round the ball, so
/// nothing here needs a seam. The flat face faces `(−1, 0, 0)` and so walks the
/// rim against its parameter; the round face takes it the other way. Both
/// surfaces are stored facing out of the material, so `same_sense` is `.T.` on
/// both faces — a ball has no turned-round form, the same standing the cylinder
/// has in `which_way_a_cap_circle_is_walked.rs`.
fn a_cap_sliced_off_a_ball(ball: f64, cut: f64) -> String {
    let rim = (ball * ball - cut * cut).sqrt();
    let (ball, cut, rim) = (real(ball), real(cut), real(rim));
    let mut text = a_header(".MILLI.");
    text.push_str(&format!(
        "#10=CARTESIAN_POINT('',(0.,0.,0.));\n\
         #11=CARTESIAN_POINT('',({cut},0.,0.));\n\
         #12=DIRECTION('',(1.,0.,0.));\n\
         #13=DIRECTION('',(-1.,0.,0.));\n\
         #14=DIRECTION('',(0.,1.,0.));\n\
         #15=DIRECTION('',(0.,0.,1.));\n\
         #20=AXIS2_PLACEMENT_3D('',#11,#12,#14);\n\
         #21=CIRCLE('',#20,{rim});\n\
         #22=CARTESIAN_POINT('',({cut},{rim},0.));\n\
         #23=VERTEX_POINT('',#22);\n\
         #24=EDGE_CURVE('',#23,#23,#21,.T.);\n\
         #30=AXIS2_PLACEMENT_3D('',#11,#13,#14);\n\
         #31=PLANE('',#30);\n\
         #32=ORIENTED_EDGE('',*,*,#24,.F.);\n\
         #33=EDGE_LOOP('',(#32));\n\
         #34=FACE_OUTER_BOUND('',#33,.T.);\n\
         #35=ADVANCED_FACE('',(#34),#31,.T.);\n\
         #40=AXIS2_PLACEMENT_3D('',#10,#15,#12);\n\
         #41=SPHERICAL_SURFACE('',#40,{ball});\n\
         #42=ORIENTED_EDGE('',*,*,#24,.T.);\n\
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
