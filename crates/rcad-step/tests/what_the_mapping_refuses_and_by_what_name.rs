//! Every refusal the mapping can make, one test each, reached from a file that
//! works by breaking exactly one thing.
//!
//! Same shape as `what_the_reader_refuses_and_by_what_name.rs` one layer down.
//! Each test starts from [`a_block`] or [`a_quarter_disc_plate`] — both of which
//! import, which `a_block_imports` and the control in
//! `how_often_a_real_file_imports.rs` prove — cuts one line out of the text, and
//! checks the refusal names the entity that was cut.
//!
//! **Whether each is a limit or a gap is on the variant's own doc comment**
//! (§16.10) and is not repeated here. What is here is that every one of them is
//! reachable from bytes, which is the claim a table in a report cannot make.

mod written_by_hand;

use std::path::{Path, PathBuf};
use std::{env, fs};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{EntityId, Import, Refusal, StepFile};
use written_by_hand::{a_block, a_plate_bounded_by_an_arc_and_its_chord, a_quarter_disc_plate};

/// A block of sides two, four and eight, in millimetres.
fn a_working_block() -> String {
    a_block([2.0, 4.0, 8.0], ".MILLI.")
}

/// A quarter-disc plate of radius two, four tall, in millimetres.
fn a_working_plate() -> String {
    a_quarter_disc_plate(2.0, 4.0, ".MILLI.")
}

/// What the mapping made of a file, or what it refused it for.
fn read(text: &str) -> Result<Import, Refusal> {
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let mut log = ProvenanceLog::new();
    Import::read(&file, &Policy::certified(), &mut log)
}

/// The refusal a file earns, and a panic if it imports instead.
fn refusal(text: &str) -> Refusal {
    match read(text) {
        Ok(import) => panic!(
            "this was supposed to be refused, and instead it imported {} solids",
            import.solids().len()
        ),
        Err(why) => why,
    }
}

/// The starting point, so that every test below breaks something that mattered.
#[test]
fn a_block_and_a_plate_both_import() {
    assert_eq!(
        read(&a_working_block())
            .expect("a block imports")
            .solids()
            .len(),
        1
    );
    assert_eq!(
        read(&a_working_plate())
            .expect("a plate imports")
            .solids()
            .len(),
        1
    );
}

#[test]
fn a_file_with_no_solid_in_it_is_refused_by_name() {
    let no_solid = a_working_block().replace("#501=MANIFOLD_SOLID_BREP('',#500);\n", "");
    assert!(matches!(refusal(&no_solid), Refusal::TheFileHoldsNoSolid));
}

#[test]
fn a_solid_with_a_void_in_it_is_refused_by_name() {
    let hollow = a_working_block().replace(
        "#501=MANIFOLD_SOLID_BREP('',#500);",
        "#501=BREP_WITH_VOIDS('',#500,(#500));",
    );
    assert!(matches!(
        refusal(&hollow),
        Refusal::ASolidWithAVoidInIt { id: EntityId(501) }
    ));
}

#[test]
fn a_surface_kind_this_kernel_does_not_hold_is_refused_by_the_name_the_file_gives_it() {
    let splined = a_working_block().replace(
        "#450=PLANE('',#460);",
        "#450=B_SPLINE_SURFACE_WITH_KNOTS('',#460);",
    );
    let Refusal::ASurfaceThisKernelDoesNotHold {
        surface, keyword, ..
    } = refusal(&splined)
    else {
        panic!("a B-spline patch is refused by its own name");
    };
    assert_eq!(surface, EntityId(450));
    assert_eq!(keyword, "B_SPLINE_SURFACE_WITH_KNOTS");
}

#[test]
fn a_curve_kind_this_kernel_does_not_hold_is_refused_by_the_name_the_file_gives_it() {
    let splined = a_working_block().replace(
        "#300=LINE('',#100,#320);",
        "#300=B_SPLINE_CURVE_WITH_KNOTS('',#100,#320);",
    );
    let Refusal::ACurveThisKernelDoesNotHold { curve, keyword, .. } = refusal(&splined) else {
        panic!("a B-spline edge is refused by its own name");
    };
    assert_eq!(curve, EntityId(300));
    assert_eq!(keyword, "B_SPLINE_CURVE_WITH_KNOTS");
}

#[test]
fn a_loop_that_is_not_an_edge_loop_is_refused_by_name() {
    let standing_still = a_working_block().replace(
        "#430=EDGE_LOOP('',(#400,#401,#402,#403));",
        "#430=VERTEX_LOOP('',#110);",
    );
    let Refusal::ALoopThisReaderDoesNotWalk { id, keyword, .. } = refusal(&standing_still) else {
        panic!("a vertex loop is refused by its own name");
    };
    assert_eq!(id, EntityId(430));
    assert_eq!(keyword, "VERTEX_LOOP");
}

#[test]
fn a_face_that_does_not_say_which_bound_is_the_outer_one_is_refused_by_name() {
    let ambiguous = a_working_block()
        .replace(
            "#440=FACE_OUTER_BOUND('',#430,.T.);",
            "#440=FACE_BOUND('',#430,.T.);",
        )
        .replace(
            "#490=ADVANCED_FACE('',(#440),#450,.T.);",
            "#490=ADVANCED_FACE('',(#440,#440),#450,.T.);",
        );
    assert!(matches!(
        refusal(&ambiguous),
        Refusal::AFaceThatDoesNotSayWhichBoundIsTheOuterOne {
            face: EntityId(490),
            bounds: 2
        }
    ));
}

#[test]
fn an_arc_whose_two_ends_are_the_same_point_is_refused_by_name() {
    // The far end of the bottom arc, moved onto the near end's own coordinates.
    // Two `VERTEX_POINT`s at one place stay two corners — nothing here merges
    // them — so what the arc names is neither of the two pieces of its circle.
    let pinched =
        a_working_plate().replace("#612=VERTEX_POINT('',#602);", "#612=VERTEX_POINT('',#600);");
    assert!(matches!(
        refusal(&pinched),
        Refusal::AnArcThatStartsAndEndsInTheSamePlace {
            edge: EntityId(690)
        }
    ));
}

#[test]
fn a_unit_the_kernel_does_not_have_is_refused_rather_than_scaled() {
    let nanometres = a_working_block().replace(".MILLI.,.METRE.", ".NANO.,.METRE.");
    let Refusal::ALengthUnitThisKernelDoesNotHave { id, described } = refusal(&nanometres) else {
        panic!("this kernel has no nanometre");
    };
    assert_eq!(id, EntityId(1));
    assert_eq!(described, "NANO metre");
}

#[test]
fn a_file_that_declares_no_length_unit_is_refused_by_name() {
    let weightless =
        a_working_block().replace("LENGTH_UNIT()NAMED_UNIT(*)", "MASS_UNIT()NAMED_UNIT(*)");
    assert!(matches!(
        refusal(&weightless),
        Refusal::TheFileDeclaresNoLengthUnit
    ));
}

#[test]
fn a_file_that_declares_two_different_length_units_is_refused_by_name() {
    let both = a_working_block().replace(
        "#501=MANIFOLD_SOLID_BREP('',#500);",
        "#900=(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT($,.METRE.));\n\
         #901=(GEOMETRIC_REPRESENTATION_CONTEXT(3)\
         GLOBAL_UNIT_ASSIGNED_CONTEXT((#900))REPRESENTATION_CONTEXT('',''));\n\
         #501=MANIFOLD_SOLID_BREP('',#500);",
    );
    assert!(matches!(
        refusal(&both),
        Refusal::TheFileDeclaresTwoDifferentLengthUnits { .. }
    ));
}

#[test]
fn an_assembly_occurrence_is_refused_rather_than_imported_in_the_wrong_place() {
    let assembled = a_working_block().replace(
        "#501=MANIFOLD_SOLID_BREP('',#500);",
        "#900=NEXT_ASSEMBLY_USAGE_OCCURRENCE('','','',#1,#1,$);\n\
         #501=MANIFOLD_SOLID_BREP('',#500);",
    );
    assert!(matches!(
        refusal(&assembled),
        Refusal::APlacementThisReaderCannotApply {
            id: EntityId(900),
            keyword: "NEXT_ASSEMBLY_USAGE_OCCURRENCE"
        }
    ));
}

#[test]
fn an_entity_that_is_not_the_kind_the_schema_wants_is_refused_by_name() {
    let muddled = a_working_block().replace("#450=PLANE('',#460);", "#450=PLANE('',#100);");
    let Refusal::AnEntityIsNotTheKindTheSchemaWants { id, wanted, found } = refusal(&muddled)
    else {
        panic!("a plane sits on a placement and not on a point");
    };
    assert_eq!(id, EntityId(100));
    assert_eq!(wanted, "AXIS2_PLACEMENT_3D");
    assert_eq!(found, "CARTESIAN_POINT");
}

#[test]
fn an_attribute_that_is_not_what_the_schema_wants_is_refused_by_name() {
    let senseless = a_working_block().replace(
        "#490=ADVANCED_FACE('',(#440),#450,.T.);",
        "#490=ADVANCED_FACE('',(#440),#450,$);",
    );
    assert!(matches!(
        refusal(&senseless),
        Refusal::AnAttributeIsNotWhatTheSchemaWants {
            id: EntityId(490),
            keyword: "ADVANCED_FACE",
            position: 3,
            ..
        }
    ));
}

#[test]
fn an_integer_too_big_to_be_the_real_it_stands_for_is_refused_by_name() {
    // One past `2^53`, which is the first whole number an `f64` cannot hold.
    let coarse = a_working_block().replace(
        "#100=CARTESIAN_POINT('',(0.0,0.0,0.0));",
        "#100=CARTESIAN_POINT('',(9007199254740993,0.0,0.0));",
    );
    assert!(matches!(
        refusal(&coarse),
        Refusal::AnIntegerTooBigToBeTheRealItStandsFor {
            id: EntityId(100),
            value: 9_007_199_254_740_993,
            ..
        }
    ));
}

#[test]
fn an_integer_the_schema_wants_as_a_real_is_read_as_one_and_counted() {
    let whole = a_working_block().replace(
        "#100=CARTESIAN_POINT('',(0.0,0.0,0.0));",
        "#100=CARTESIAN_POINT('',(0,0,0));",
    );
    let import = read(&whole).expect("nought and nought point nought are the same number");
    assert_eq!(
        import.integers_read_as_reals(),
        3,
        "three integers stood in for three reals, and a reader that did that quietly \
         would be a reader that heals"
    );
}

#[test]
fn a_reference_that_points_nowhere_is_refused_by_name() {
    let dangling = a_working_block().replace(
        "#490=ADVANCED_FACE('',(#440),#450,.T.);",
        "#490=ADVANCED_FACE('',(#440),#999,.T.);",
    );
    assert!(matches!(
        refusal(&dangling),
        Refusal::AReferencePointsNowhere {
            from: EntityId(490),
            to: EntityId(999)
        }
    ));
}

#[test]
fn geometry_that_is_not_a_working_shape_is_refused_by_name() {
    let flat = a_working_block().replace(
        "#480=DIRECTION('',(0.0,0.0,-1.0));",
        "#480=DIRECTION('',(0.0,0.0,0.0));",
    );
    assert!(matches!(
        refusal(&flat),
        Refusal::TheGeometryIsNotAWorkingShape {
            id: EntityId(450),
            ..
        }
    ));
}

/// A two-step `EDGE_LOOP` is no longer the mapping's to refuse, and what stops
/// this file now is a mesh that cannot be drawn rather than a loop that cannot
/// be held.
///
/// **What changed.** `SolidBuilder::add_wire` and `Model::validate` both wanted
/// three steps, so this file died at entity 740 with
/// `ADraftWireIsTooShort { steps: 2 }` and the mapping named the loop. Both now
/// take two different edges that are not both straight, so the loop goes
/// through, four faces are built, and the refusal comes back from the sweep with
/// the whole solid's name on it.
///
/// **The defect this test was written around is fixed, and this test is what
/// says so.** It used to be green for a bad reason:
/// [`a_plate_bounded_by_an_arc_and_its_chord`] writes a **quarter** arc, and
/// `rcad_geom::walk` never took fewer than one step — so at
/// `as_coarse_as_a_mesh_gets`, which is what the sweep meshes with, that arc got
/// no point between its ends and the flat face fanned into two corners
/// `rcad-mesh` will not take as a loop. The old rustdoc here named its own
/// undoing: *"closing it — a floor on how coarsely a curved boundary may be
/// walked — changes this test again."* It was closed, the floor is two pieces
/// because one piece shows **none** of a bend at any deflection, and the file
/// imports.
///
/// **Letting the loop through found a second thing, and it was in front rather
/// than behind.** The file's chord face read
/// `Face::Flat([1.0, 1.0, 0.0], [radius, 0.0, 0.0])`, and that module documents
/// and emits the first vector as the **outward** normal with `same_sense` true.
/// A segment's material is the arc side, `x + y >= r`, so out of it is towards
/// the axis and the file had the normal back to front. Nothing had caught it
/// because no face of this file had ever been oriented: the old rule stopped the
/// mapping at the loop, and a face is built from a loop. With two steps allowed
/// the sweep got as far as the faces and said
/// `OuterWireWindsAgainstTheNormal`, which is how it was found. It is fixed in
/// `written_by_hand`, and this test's refusal moving from the winding to the
/// mesh is the test of that fix.
///
/// **So this is where the two-step rule is proved from bytes**, which is the
/// thing the old note said nothing did. `a_face_bounded_by_two_steps.rs` builds
/// the shape face by face and sweeps it with nothing approximated; this reads it
/// out of a file, which is the path a real part takes.
#[test]
fn a_two_step_loop_imports() {
    let segment = a_plate_bounded_by_an_arc_and_its_chord(2.0, 4.0, ".MILLI.");
    let import = read(&segment).expect("a plate bounded by an arc and its chord imports");
    assert_eq!(
        import.solids().len(),
        1,
        "a segment plate is one solid, and the two-step loop is now held all the way through"
    );
}

#[test]
fn a_solid_the_sweep_will_not_have_is_refused_with_the_sweeps_own_name() {
    // One face's plane turned to face along `y`, which leaves its own corners
    // off it. Nothing about one entity is wrong; the solid is.
    let leaning = a_working_block().replace(
        "#480=DIRECTION('',(0.0,0.0,-1.0));",
        "#480=DIRECTION('',(0.0,1.0,0.0));",
    );
    let Refusal::TheSolidTheFileDescribesIsNotOne { solid, .. } = refusal(&leaning) else {
        panic!("a corner off its face's plane is the sweep's refusal and not one entity's");
    };
    assert_eq!(solid, EntityId(501));
}

#[test]
fn a_corner_that_is_not_on_its_curved_face_is_refused_with_how_far_off_it_is() {
    // The pipe made a hair wider than the corners the file puts on it.
    let widened = a_working_plate().replace(
        "#774=CYLINDRICAL_SURFACE('',#768,2.0);",
        "#774=CYLINDRICAL_SURFACE('',#768,2.000000001);",
    );
    let Refusal::ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn {
        face,
        off_by,
        coordinate_spacing,
        ..
    } = refusal(&widened)
    else {
        panic!("a corner a nanometre off a pipe is off it");
    };
    assert_eq!(face, EntityId(780));
    assert!(
        off_by > coordinate_spacing,
        "a nanometre on a two millimetre pipe is not a rounding: {off_by} against {coordinate_spacing}"
    );
}

/// The corner of a real file, worked out from the file's own decimal text, so
/// that "this refusal is a limit and not a gap" is drawn rather than argued.
///
/// `kicad/LED_D3.0mm.step` puts the corner `#118` at
/// `(-0.23, -1.16619037897, 3.)` on `#140`, a cylinder of radius `1.9` about
/// `(1.27, 0., 3.)`. The corner is exactly `1.5` from the axis along `x`, so the
/// `y` the cylinder asks for is `√(1.9² − 1.5²) = 1.16619037896906…` and the
/// file wrote **twelve significant digits of it**. Those digits are short by
/// `9.4e-13`, which is `5.8e-13` of radius, and that is the number the refusal
/// hands back.
///
/// **An `f64` would have been two thousand times closer**, which is the whole
/// claim. `NEAREST_F64_POINT_ROUNDOFF` times the coordinate is `2.6e-16`, and
/// `crates/rcad-geom/tests/how_far_off_a_curved_surface_the_nearest_f64_can_sit.rs`
/// is the search that says no point of a curved surface written as three `f64`
/// lands further off than that. So this corner is not the rounding of any point
/// of this cylinder: the exporter chose twelve digits, the answer the file needs
/// is not among its own numbers, and no filter, no exact level and no rescale
/// reaches a corner that was never written down. A sweep that accepted it would
/// be accepting a corner nobody placed.
///
/// Measured on 2026-09-20, this is the shape of **75 of the 76** corpus files
/// whose first refusal is this one: the smallest miss among the other 74 is 56
/// coordinate spacings and the largest 20 777, where one spacing is all a
/// rounding can cost. The remaining file is `SW_Slide_SPDT_Angled_…`, whose
/// named corner is inside its own spacing and is not what the sweep stopped at —
/// the looseness the variant's own doc comment declares.
#[test]
fn the_corpus_corner_this_refusal_names_was_written_short_and_not_rounded() {
    let text = fs::read(corpus().join("kicad").join("LED_D3.0mm.step"))
        .expect("LED_D3.0mm.step is in the pinned corpus");
    let file = StepFile::read(&text).expect("every file in the corpus parses");
    let mut log = ProvenanceLog::new();
    let Err(why) = Import::read(&file, &Policy::certified(), &mut log) else {
        panic!("LED_D3.0mm.step does not import, and this test is about why not");
    };
    let Refusal::ACornerIsNotOnTheCurvedFaceTheFileSaysItIsOn {
        face,
        corner,
        off_by,
        coordinate_spacing,
        ..
    } = why
    else {
        panic!("LED_D3.0mm.step is refused for a corner off a curved face, and got: {why}");
    };
    assert_eq!((face, corner), (EntityId(113), EntityId(118)));

    let written = -1.166_190_378_97_f64;
    let asked_for = (1.9_f64 * 1.9 - 1.5 * 1.5).sqrt();
    let short_by = (written.abs() - asked_for).abs();
    let missed_by = ((1.5_f64 * 1.5 + written * written).sqrt() - 1.9).abs();
    assert!(
        (missed_by - off_by).abs() < off_by * 1e-3,
        "the file's own decimal text misses the cylinder by {missed_by} and the refusal says \
         {off_by}, so this test is no longer reading the corner the refusal names"
    );

    let a_rounding_of_that_coordinate = asked_for * rcad_geom::NEAREST_F64_POINT_ROUNDOFF;
    assert!(
        short_by > 2000.0 * a_rounding_of_that_coordinate,
        "writing that coordinate down costs {a_rounding_of_that_coordinate} and the file is \
         {short_by} out, so this is no longer a number the exporter shortened"
    );
    assert!(
        off_by > 800.0 * coordinate_spacing,
        "{off_by} against a coordinate spacing of {coordinate_spacing}"
    );
}

#[test]
fn a_corner_that_is_not_on_the_plane_of_its_own_arc_is_refused_with_how_far_off_it_is() {
    // The bottom arc's circle lifted a nanometre off the corners it ends at.
    let lifted = a_working_plate().replace(
        "#620=CARTESIAN_POINT('',(0.0,0.0,0.0));",
        "#620=CARTESIAN_POINT('',(0.0,0.0,1.E-9));",
    );
    let Refusal::ACornerIsNotOnThePlaneOfTheArcItEndsOn {
        edge,
        off_by,
        coordinate_spacing,
        ..
    } = refusal(&lifted)
    else {
        panic!("a corner a nanometre off the plane its arc lies in is off it");
    };
    assert_eq!(edge, EntityId(690));
    assert!(
        off_by > coordinate_spacing,
        "{off_by} against {coordinate_spacing}"
    );
}

#[test]
fn an_arc_whose_own_numbers_overflow_a_picking_half_space_is_refused_by_name() {
    // A plate `1e200` across. The half-space that says which of the two arcs an
    // edge is stands on the chord, and its offset is a dot product of numbers
    // that size — which is not a number an `f64` holds. **This is the one
    // refusal a real file has never reached**, and it is here because the
    // alternative to a named refusal is a picker that refuses the very corners
    // it was built from.
    let enormous = a_quarter_disc_plate(1e200, 1e200, ".MILLI.");
    assert!(matches!(
        refusal(&enormous),
        Refusal::AnArcThisReaderCouldNotName {
            edge: EntityId(690)
        }
    ));
}

#[test]
fn a_file_whose_every_coordinate_is_zero_is_refused_by_name() {
    let nothing = a_working_block()
        .replace("(2.0,", "(0.0,")
        .replace(",4.0,", ",0.0,")
        .replace(",8.0)", ",0.0)");
    let flattened = nothing
        .replace("(2.0,0.0,0.0)", "(0.0,0.0,0.0)")
        .replace("2.0", "0.0")
        .replace("4.0", "0.0")
        .replace("8.0", "0.0")
        .replace("1.0", "0.0")
        .replace("1.E-7", "0.0");
    assert!(matches!(refusal(&flattened), Refusal::TheFileHasNoSize));
}

/// A file the mesher stops on names the face it could not cut into triangles,
/// not the whole solid.
///
/// `docs/decisions.md` §123.5's half of §123: the mapping's own
/// `ADraftEdgeHasACrossingThatCannotBeFollowed` already turned a crossing
/// refusal's key into the position an edge was described at
/// (`a_crossing_the_marcher_gave_up_on_names_its_edge.rs`), and
/// `ADraftFaceCouldNotBeMeshed` is the same move for a face the sweep's own
/// mesher gives up on — `builder::tests::a_face_the_sweep_cannot_mesh_names_its_own_position`
/// in `rcad-topo` shows the same refusal from a hand-built solid; this shows
/// it reachable from bytes.
///
/// Same shape as `a_crossing_the_marcher_gave_up_on_names_its_edge.rs`: a
/// property over whichever corpus files bottom out here, and not a named
/// file, because a rescue can empty the set the way §112 emptied the
/// tangency wall's (that file's own note on why it names none).
#[test]
fn a_face_the_mesher_cannot_cut_up_names_its_own_advanced_face() {
    let root = corpus();
    let mut stopped_there = Vec::new();

    for family in ["nist", "kicad"] {
        for path in files_under(&root.join(family)) {
            let bytes =
                fs::read(&path).unwrap_or_else(|why| panic!("reading {}: {why}", path.display()));
            let Ok(file) = StepFile::read(&bytes) else {
                continue;
            };
            let mut log = ProvenanceLog::new();
            let Err(refusal) = Import::read(&file, &Policy::certified(), &mut log) else {
                continue;
            };
            let Refusal::ThisEntityOfTheSolidWasRefused { entity, why, .. } = &refusal else {
                continue;
            };
            if !matches!(**why, rcad_topo::Error::ADraftFaceCouldNotBeMeshed { .. }) {
                continue;
            }
            let name = path.display().to_string();
            stopped_there.push(name.clone());

            assert_eq!(
                keyword_of(&file, *entity),
                "ADVANCED_FACE",
                "{name} refused about {entity}, and a mesher that cannot cut a face into \
                 triangles is refusing that face: {why}"
            );
        }
    }

    println!(
        "files stopping at the mesher, on a face: {}\n  {}",
        stopped_there.len(),
        stopped_there.join("\n  ")
    );
    assert!(
        !stopped_there.is_empty(),
        "no corpus file stops at the mesher over a face any more, so this proved nothing. That \
         is either very good news, or the mapping stopped naming the face. Find out which, then \
         retire this test or fix what broke."
    );
}

fn keyword_of(file: &StepFile, id: EntityId) -> &str {
    file.instance(id)
        .and_then(|instance| instance.parts().first())
        .map_or("<unknown>", |record| record.keyword())
}

fn files_under(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|kind| kind == "step"))
        .collect();
    found.sort();
    found
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
