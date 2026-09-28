//! Which way a torus face's normal points, asked of a bend of pipe written in a
//! file and of the same bend built the way this kernel builds one
//! (`docs/decisions.md` §329).
//!
//! **Every answer is checked against the shape and not against the kernel.** A
//! quarter bend of ring four and tube one about `+z`, from a cap on `y = 0` to
//! a cap on `x = 0`, encloses `2π²` by Pappus — the tube's section `π` carried
//! a quarter of the way round a circle of radius four. A point in the tube
//! between the caps is inside it; a point past a cap, in the hole, or beyond
//! the outer equator is outside.
//!
//! # Two ways to write one bend
//!
//! **As the standard says.** ISO 10303-42 parametrises a `TOROIDAL_SURFACE` as
//! `C + (R + r cos v)((cos u)x + (sin u)y) + r(sin v)z`, round the ring first,
//! and its `σ_u × σ_v` points out of the tube; Open CASCADE's
//! `Geom_ToroidalSurface` says the same in words. So a bend's wall is written
//! `same_sense .T.`, and every loop — both caps and the wall — winds the way a
//! right hand turns about its own face's outward normal.
//!
//! **As this kernel builds one** — `tests/rings/mod.rs`'s `a_quarter_bend`,
//! read in here by path so that it is the kernel's own description and not a
//! copy of it. Its wall is used `Reversed` and every one of its loops runs the
//! other way. Written as a file that is `same_sense .F.` and every
//! `ORIENTED_EDGE` flag turned over, and that file is read in here too: it
//! answers everything the built bend answers, which is what shows the reader
//! adds nothing of its own.
//!
//! # What this kernel says today, and which pin moves
//!
//! `Model::validate`'s closed-edge rule, `classify_point` and the file reader
//! all take a torus's own normal **out of the tube**. The mesher alone takes it
//! **into** the tube, because `Torus::point_at` puts the tube angle first and
//! `hand_over` turns a face's triangles by the chart's own `∂first × ∂second`.
//! So:
//!
//! - the bend written as the standard says passes the closed-edge rule and is
//!   refused by the mesh, `FacesDoNotAgreeWhichWayTheyFace`, and so are three
//!   KiCad relays in the corpus;
//! - the bend built the kernel's way passes the mesh, because its wall is
//!   turned round, and passes the closed-edge rule, because its loops are
//!   turned round too; it measures `2π²`, and `classify_point` puts a point
//!   past either cap **inside** it.
//!
//! When `mesh.rs` turns a torus face's triangles against its chart, the pins
//! marked *today* in the first and last tests move to what the shape says, and
//! the kernel-built bend is refused until `tests/rings/mod.rs` builds it the
//! standard way — at which point the middle test's wrong placements move too.
//! §329.5 lists the rows.

mod written_by_hand;

#[path = "../../rcad-topo/tests/rings/mod.rs"]
mod rings;

use std::path::PathBuf;

use rcad_geom::{Deflection, Point, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};
use rcad_topo::{Containment, Error, Invalid, Model, SolidKey};
use written_by_hand::a_header;

/// The bend written as the standard says is refused by the mesh today, after
/// passing the closed-edge rule that runs before it.
///
/// **Today's pin, and the one a fix to `mesh.rs` moves.** What the shape says
/// is that it validates, encloses `2π²`, and places every point in [`POINTS`]
/// as listed there. With the mesher turning a torus face's
/// triangles against its chart, this very file validates, measures `19.706`
/// give or take `0.392`, and places all eight exact points right (§329.3).
#[test]
fn a_bend_written_as_the_standard_says_is_refused_by_the_mesh_today() {
    let text = a_quarter_bend_written(Written::AsTheStandardSays);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let refused = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
        .err()
        .expect("today the mesh refuses the bend a file writes");
    assert!(
        is_faces_do_not_agree(&refused),
        "the mesh takes a torus's own normal into the tube and the file out of it, and the \
         refusal says so by name: {refused:?}"
    );
}

/// The bend built the way this kernel builds one, and the same bend written as
/// a file the kernel's way, validate and enclose `2π²` — and put a point past
/// either cap inside.
///
/// **The two placements past the caps are today's pins and are wrong.** A ray
/// from `(−1, 3.5, 0)` goes in through the cap on `x = 0` and out through the
/// wall; the cap counts `−1` and the turned-round wall counts its leaving
/// `−1`, and `−2` is read as inside. Everything else here matches the shape.
#[test]
fn a_bend_built_the_kernels_way_validates_and_puts_a_point_past_its_cap_inside() {
    let policy = Policy::certified();
    let mut built = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let key = rings::a_quarter_bend(&mut built, 4.0, 1.0);

    let text = a_quarter_bend_written(Written::AsThisKernelBuildsIt);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let read = Import::read(&file, &policy, &mut ProvenanceLog::new())
        .expect("the kernel's own bend written out as a file imports");
    let [(_, read_key)] = read.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };

    for (spelling, model, key) in [
        ("built", &built, key),
        ("written the kernel's way", read.model(), *read_key),
    ] {
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("the bend {spelling} validates: {why}"));
        let solid = model.solid(key).expect("this model made the key");
        encloses_what_the_shape_does(model, key, spelling);
        for (at, the_shape_says) in POINTS {
            let placed = solid.classify_point(Point::from(at), &policy, &mut ProvenanceLog::new());
            let today = if at == PAST_THE_FAR_CAP || at == PAST_THE_NEAR_CAP {
                Containment::Inside
            } else {
                the_shape_says
            };
            assert_eq!(placed, today, "{at:?} against the bend {spelling}");
        }
    }
}

/// Three KiCad relays in the corpus, each with a torus face used forwards, are
/// refused by the same name as the bend written as the standard says.
///
/// **Ground truth the kernel did not make.** With the mesher turning a torus
/// face's triangles against its chart and nothing else changed, all three
/// import — which is to say they validate — and no file in the corpus with a
/// torus in it is refused that was not refused before (§329.4). Today's pin; it
/// moves with the first test's.
#[test]
fn three_relays_in_the_corpus_are_refused_by_the_mesh_today() {
    let root = PathBuf::from(std::env::var_os("RCAD_STEP_CORPUS").expect("`just t` sets it"));
    for name in [
        "Relay_1-Form-A_Schrack-RYII_RM5mm.step",
        "Relay_1-Form-B_Schrack-RYII_RM5mm.step",
        "Relay_SPDT_Finder_34.51_Vertical.step",
    ] {
        let path = root.join("kicad").join(name);
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|why| panic!("{} is in the corpus: {why}", path.display()));
        let file = StepFile::read(&bytes).expect("a corpus file reads");
        let refused = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
            .err()
            .unwrap_or_else(|| panic!("today the mesh refuses {name}"));
        assert!(is_faces_do_not_agree(&refused), "{name}: {refused:?}");
    }
}

/// Exact points against a quarter bend from `+x` to `+y`, and where the shape
/// puts each.
///
/// Every one has exact coordinates, and so does its step to the first ray
/// target, which is what lets a ray be fired at a ring at all.
const POINTS: [([f64; 3], Containment); 8] = [
    ([4.0, 1.0, 0.0], Containment::Inside),
    ([3.5, 3.5, 0.0], Containment::Inside),
    ([1.0, 4.0, 0.5], Containment::Inside),
    (PAST_THE_FAR_CAP, Containment::Outside),
    (PAST_THE_NEAR_CAP, Containment::Outside),
    ([4.0, 4.0, 0.0], Containment::Outside),
    ([5.5, 1.0, 0.0], Containment::Outside),
    ([1.0, 1.0, 0.0], Containment::Outside),
];

/// In the tube the bend would have if it went on past its cap on `x = 0`.
const PAST_THE_FAR_CAP: [f64; 3] = [-1.0, 3.5, 0.0];

/// The same past the cap on `y = 0`.
const PAST_THE_NEAR_CAP: [f64; 3] = [3.5, -1.0, 0.0];

fn is_faces_do_not_agree(refused: &Refusal) -> bool {
    matches!(
        refused,
        Refusal::TheSolidTheFileDescribesIsNotOne { why, .. }
            if matches!(**why, Error::Invalid(Invalid::FacesDoNotAgreeWhichWayTheyFace))
    )
}

/// The measured volume is `2π²` to within the measurement's own error.
fn encloses_what_the_shape_does(model: &Model, key: SolidKey, spelling: &str) {
    let holds = 2.0 * core::f64::consts::PI * core::f64::consts::PI;
    let measured = model
        .solid(key)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a hundredth of the tube"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the bend {spelling} measures: {why}"));
    assert!(
        (measured.volume() - holds).abs() <= measured.volume_error(),
        "a quarter bend of ring four and tube one encloses {holds}, and the one {spelling} \
         measures {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

#[derive(Clone, Copy)]
enum Written {
    /// Wall `same_sense .T.`, every loop winding right-handed about its face's
    /// outward normal.
    AsTheStandardSays,
    /// Wall `same_sense .F.` and every `ORIENTED_EDGE` turned over: the bend
    /// `tests/rings/mod.rs` builds.
    AsThisKernelBuildsIt,
}

/// A quarter bend of ring four and tube one about `+z`, in millimetres.
///
/// Both cap circles are closed edges with one corner each on the inner
/// equator, `(3, 0, 0)` and `(0, 3, 0)`, and the wall is bounded by the two of
/// them and has no seam. Each `CIRCLE` turns right-handed about its cap's
/// outward normal — `−y` for the near cap, `−x` for the far one — so a cap
/// walks its circle with the curve and the wall against it, as the standard
/// asks. [`Written::AsThisKernelBuildsIt`] turns every one of those flags over.
fn a_quarter_bend_written(written: Written) -> String {
    let (cap, wall, same_sense) = match written {
        Written::AsTheStandardSays => (".T.", ".F.", ".T."),
        Written::AsThisKernelBuildsIt => (".F.", ".T.", ".F."),
    };
    let mut text = a_header(".MILLI.");
    text.push_str(&format!(
        "#10=CARTESIAN_POINT('',(0.,0.,0.));\n\
         #11=DIRECTION('',(0.,0.,1.));\n\
         #12=DIRECTION('',(1.,0.,0.));\n\
         #13=DIRECTION('',(0.,-1.,0.));\n\
         #14=DIRECTION('',(-1.,0.,0.));\n\
         #15=DIRECTION('',(0.,1.,0.));\n\
         #20=CARTESIAN_POINT('',(4.,0.,0.));\n\
         #21=AXIS2_PLACEMENT_3D('',#20,#13,#14);\n\
         #22=CIRCLE('',#21,1.);\n\
         #23=CARTESIAN_POINT('',(3.,0.,0.));\n\
         #24=VERTEX_POINT('',#23);\n\
         #25=EDGE_CURVE('',#24,#24,#22,.T.);\n\
         #30=CARTESIAN_POINT('',(0.,4.,0.));\n\
         #31=AXIS2_PLACEMENT_3D('',#30,#14,#13);\n\
         #32=CIRCLE('',#31,1.);\n\
         #33=CARTESIAN_POINT('',(0.,3.,0.));\n\
         #34=VERTEX_POINT('',#33);\n\
         #35=EDGE_CURVE('',#34,#34,#32,.T.);\n\
         #40=AXIS2_PLACEMENT_3D('',#10,#13,#12);\n\
         #41=PLANE('',#40);\n\
         #42=ORIENTED_EDGE('',*,*,#25,{cap});\n\
         #43=EDGE_LOOP('',(#42));\n\
         #44=FACE_OUTER_BOUND('',#43,.T.);\n\
         #45=ADVANCED_FACE('',(#44),#41,.T.);\n\
         #50=AXIS2_PLACEMENT_3D('',#10,#14,#15);\n\
         #51=PLANE('',#50);\n\
         #52=ORIENTED_EDGE('',*,*,#35,{cap});\n\
         #53=EDGE_LOOP('',(#52));\n\
         #54=FACE_OUTER_BOUND('',#53,.T.);\n\
         #55=ADVANCED_FACE('',(#54),#51,.T.);\n\
         #60=AXIS2_PLACEMENT_3D('',#10,#11,#12);\n\
         #61=TOROIDAL_SURFACE('',#60,4.,1.);\n\
         #62=ORIENTED_EDGE('',*,*,#25,{wall});\n\
         #63=EDGE_LOOP('',(#62));\n\
         #64=FACE_OUTER_BOUND('',#63,.T.);\n\
         #65=ORIENTED_EDGE('',*,*,#35,{wall});\n\
         #66=EDGE_LOOP('',(#65));\n\
         #67=FACE_BOUND('',#66,.T.);\n\
         #68=ADVANCED_FACE('',(#64,#67),#61,{same_sense});\n\
         #70=CLOSED_SHELL('',(#45,#55,#68));\n\
         #71=MANIFOLD_SOLID_BREP('',#70);\n\
         ENDSEC;\n\
         END-ISO-10303-21;\n"
    ));
    text
}
