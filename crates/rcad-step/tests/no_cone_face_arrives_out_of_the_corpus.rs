//! No cone face reaches a consumer through import, and this is the tripwire on
//! the day that changes.
//!
//! **This is not a census and its four numbers are not the point.** §56.7
//! records that 41 corpus files name `CONICAL_SURFACE`, that none of them
//! import, and that the faces which do arrive are planes, cylinders and a single
//! sphere. Those move whenever the corpus grows or the reader improves, so none
//! of them is asserted here. What is asserted is the **claim** they support: a
//! consumer of this kernel has never yet been handed a cone face out of a real
//! file.
//!
//! # Why that claim is worth a test rather than a paragraph
//!
//! Two things in two repositories are written against it today.
//!
//! §56.7 says nobody ships against the unswept unplaced cone path, and gives
//! that as its reason for leaving it an open gap rather than fixing it now.
//! rscene special-cases a cone's apex on exact equality and has sized its guards
//! for the cones it can actually receive. Both sentences stop being true on the
//! same day, and it is a day nobody is watching for: **the cone is not what
//! refuses those 41 files.** They die upstream of it, overwhelmingly on
//! `B_SPLINE_CURVE_WITH_KNOTS` on an edge, and `CONICAL_SURFACE` itself maps
//! fine. So the day a B-spline edge lands — work with nothing to do with cones,
//! by someone with no reason to think about them — cone faces start arriving and
//! two documents quietly become wrong.
//!
//! Without this test that day produces no signal at all. §56.7's numbers were
//! taken by a probe that was deleted, so nothing re-takes them and nothing goes
//! red.
//!
//! # What to do when it fires
//!
//! Nothing is broken and this is not a defect. It is the named day. Re-read
//! §56.7, which is written on the assumption this test now denies: the unplaced
//! cone path has acquired a consumer, so it is no longer a gap that can wait,
//! and rscene has to be told that cone faces now arrive out of files rather than
//! only out of its own builders.
//!
//! # What stops it passing for the wrong reason
//!
//! An assertion that a count is zero is the easiest vacuous test there is: it
//! passes when the corpus is missing, when nothing imports, when the census
//! reads the wrong thing, and — the one that would matter most — when the reader
//! could never yield a cone face at all, in which case the day above can never
//! come and this watches for nothing forever.
//!
//! So two controls run in the same test, before the claim.
//!
//! 1. **The census sees a cone.** It is run over a cone-faced solid built here
//!    through the public builder and required to find it. That is what says a
//!    later count of zero is a fact about the corpus rather than about
//!    `Surface::Cone` being read wrongly.
//! 2. **The cone is not what refuses those files.** Every refused file that
//!    names `CONICAL_SURFACE` is required *not* to have been refused for a
//!    surface this kernel does not hold. That is what says the zero is caused
//!    upstream and will flip when upstream clears, rather than being a permanent
//!    property of the reader.
//!
//! **The control this test would rather have run is not available**, and the
//! reason is worth recording where the next person to try it will find it: this
//! kernel cannot write a cone-faced solid out. `rcad_step::write` refuses a
//! pointed cone with `ACurveTheFacesEitherSideOfItDoNotName { first: "plane",
//! second: "cone", closed: true }`, so the write-it-out-and-read-it-back control
//! that `how_often_a_real_file_imports` uses cannot be built for a cone today.
//! That is a **writer** limitation and not a reader one — `brep.rs:506` hands
//! back `EdgeRun::RightRoundWhereItsFacesCross` with no restriction on which
//! surfaces meet — so it says nothing about whether an imported cone would
//! arrive. When the writer learns that pair, the stronger control belongs here.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rcad_geom::{Axis, Cone, EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};
use rcad_topo::{DraftStep, Model, SolidBuilder};

#[test]
fn no_cone_face_arrives_out_of_the_corpus() {
    the_census_sees_a_cone_when_there_is_one();

    let root = corpus();
    let (mut files, mut imported, mut faces) = (0usize, 0usize, 0usize);
    let mut names_a_cone_surface = 0usize;
    let mut arrived: BTreeSet<String> = BTreeSet::new();
    let mut refused_for_a_surface: BTreeMap<String, String> = BTreeMap::new();

    for source in ["nist", "kicad"] {
        for path in files_under(&root.join(source)) {
            files += 1;
            let bytes = fs::read(&path).expect("a file the walk just listed");
            // A byte scan rather than a parse, so what the file *says* does not
            // depend on what this reader can take from it.
            let names_a_cone = String::from_utf8_lossy(&bytes).contains("CONICAL_SURFACE");
            if names_a_cone {
                names_a_cone_surface += 1;
            }
            let Ok(file) = StepFile::read(&bytes) else {
                continue;
            };
            match Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new()) {
                Ok(import) => {
                    imported += 1;
                    faces += import.model().faces().count();
                    if cone_faces_in(import.model()) > 0 {
                        arrived.insert(name_of(&path));
                    }
                }
                Err(why) => {
                    if names_a_cone && matches!(why, Refusal::ASurfaceThisKernelDoesNotHold { .. })
                    {
                        refused_for_a_surface.insert(name_of(&path), format!("{why}"));
                    }
                }
            }
        }
    }

    assert!(
        files > 0,
        "the walk found no STEP files, so this test did nothing"
    );
    assert!(
        imported > 0,
        "no corpus file imported at all, so finding no cone face among them says nothing"
    );
    assert!(
        faces > 0,
        "{imported} files imported holding no faces at all, so a face census over them is vacuous"
    );
    assert!(
        names_a_cone_surface > 0,
        "no corpus file names CONICAL_SURFACE any more, so this test watches for a day that can \
         no longer come. The corpus has changed out from under §56.7 and that paragraph's numbers \
         need re-taking — do not relax this assertion instead."
    );
    assert!(
        refused_for_a_surface.is_empty(),
        "a file naming CONICAL_SURFACE is now refused for a surface this kernel does not hold: \
         {refused_for_a_surface:?}. That breaks this test's second control — the claim below is \
         only interesting while the cone is *not* what blocks these files, because that is what \
         makes the zero temporary. Find out whether the reader has stopped taking conical \
         surfaces before reading anything into the assertion after this one."
    );

    assert!(
        arrived.is_empty(),
        "a cone face now reaches a consumer out of the corpus, from {arrived:?}. Nothing is broken \
         and this is not a defect: it is the day §56.7 names. Two things written on the assumption \
         it had not come are now wrong and have to be re-read — §56.7's reason for leaving the \
         unplaced cone path an open gap was that nothing ships against it, and rscene's cone \
         handling is sized for the cones its own builders make. Tell rscene, and re-take §56.7's \
         numbers rather than editing this test to agree with the new state. (Context, asserted \
         nowhere: {files} files, {imported} imported, {faces} faces, {names_a_cone_surface} \
         naming CONICAL_SURFACE.)"
    );

    println!(
        "no cone face out of {imported} imported files ({faces} faces; {names_a_cone_surface} of \
         {files} name CONICAL_SURFACE and none of them import)"
    );
}

/// How many of a model's faces stand on a cone.
fn cone_faces_in(model: &Model) -> usize {
    model
        .faces()
        .filter(|face| matches!(face.surface(), Surface::Cone(_)))
        .count()
}

/// The first control: the census finds a cone when the model holds one.
///
/// Run inside the same test rather than beside it, because what it defends
/// against is the corpus half reporting an absence it could never have reported
/// a presence for.
fn the_census_sees_a_cone_when_there_is_one() {
    assert_eq!(
        cone_faces_in(&a_cone_faced_model()),
        1,
        "the census read no cone face off a solid built with exactly one, so a count of zero over \
         the corpus would say nothing about the corpus"
    );
}

/// A pointed cone described through the public builder.
fn a_cone_faced_model() -> Model {
    const SLOPE: f64 = 0.5;
    const FAR: f64 = 8.0;

    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let skin = Cone::new(Axis::z(), SLOPE).expect("a positive slope");
    let mut builder = SolidBuilder::new();

    let rim = builder
        .add_vertex(Point::new(SLOPE * FAR, 0.0, FAR))
        .expect("a finite point");
    let circle = builder
        .add_edge(rim, rim, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the cap plane cuts the cone in one closed curve");

    let plane = Plane::from_point_and_normal(Point::new(0.0, 0.0, FAR), Vector::new(0.0, 0.0, 1.0))
        .expect("the z axis is a unit normal");
    let cap_coedge = builder
        .add_coedge(circle, Orientation::Forward)
        .expect("an edge of this build");
    let cap_wire = builder
        .add_wire(&[DraftStep::Along(cap_coedge)])
        .expect("one closed edge is a wire");
    let cap = builder
        .add_face(Surface::Plane(plane), &[cap_wire])
        .expect("a cap is a plane trimmed by its circle");

    let side_coedge = builder
        .add_coedge(circle, Orientation::Reversed)
        .expect("an edge of this build");
    let side_wire = builder
        .add_wire(&[DraftStep::Along(side_coedge)])
        .expect("an edge of this build");
    let side = builder
        .add_face(Surface::Cone(skin), &[side_wire])
        .expect("a face on the cone, bounded by its base circle and pinched at the apex");

    builder
        .add_shell(&[(cap, Orientation::Forward), (side, Orientation::Forward)])
        .expect("two faces close");
    builder
        .build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a pointed cone described this way is a solid");
    model
}

fn corpus() -> PathBuf {
    let root = env::var_os("RCAD_STEP_CORPUS").unwrap_or_else(|| {
        panic!(
            "RCAD_STEP_CORPUS is not set. Point it at a directory holding nist/ and kicad/ — this \
             test does not skip on a missing corpus."
        )
    });
    let root = PathBuf::from(root);
    assert!(
        root.is_dir(),
        "RCAD_STEP_CORPUS points at {}, which is not a directory",
        root.display()
    );
    root
}

fn files_under(directory: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files_under(&path));
            continue;
        }
        let is_step = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("stp") || extension.eq_ignore_ascii_case("step")
            });
        if is_step {
            found.push(path);
        }
    }
    found
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("a file with no name")
        .to_owned()
}
