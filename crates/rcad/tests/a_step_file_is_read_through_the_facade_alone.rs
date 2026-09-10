//! The measured defect this test closes: before it, `rcad_step::Import` and
//! its `Refusal` were reachable only by adding `rcad-step` to a caller's own
//! `Cargo.toml` and matching a second, unbridged error type. Every test below
//! names `rcad::` and nothing else — no `rcad_step::` anywhere in this file —
//! which is what "a caller who wants to read a STEP file can do it through
//! `rcad`" means as code rather than as a sentence.

use rcad::{Deflection, Error, Import, Model, Policy, ProvenanceLog, Refusal, Scale, StepFile};

#[test]
fn a_cube_this_kernel_wrote_is_read_back_through_the_facade_alone() {
    let scale = Scale::millimetres(10.0).expect("a working scale");
    let mut model = Model::new(scale);
    let mut log = ProvenanceLog::new();
    model
        .add_cube(10.0, &Policy::certified(), &mut log)
        .expect("a cube of side ten");

    let text = rcad::write(&model, "a cube round-tripped through rcad alone")
        .expect("a sound model writes");
    let file = StepFile::read(text.as_bytes()).expect("a file this crate wrote reads");
    let import =
        Import::read(&file, &Policy::certified(), &mut log).expect("its one solid imports");

    let [(_, key)] = import.solids() else {
        panic!("one solid written is one solid read back");
    };
    let measured = import
        .model()
        .solid(*key)
        .expect("this model made the key")
        .measure(Deflection::for_scale(scale), &Policy::certified(), &mut log)
        .expect("a cube measures");
    assert_eq!(
        measured.volume(),
        1000.0,
        "the cube's own volume, not a copy of it"
    );
}

/// One `?` from bytes with no `MANIFOLD_SOLID_BREP` in them to a
/// [`rcad::Error`], never naming `rcad_step::Refusal` directly.
#[test]
fn a_file_with_no_solid_in_it_refuses_as_a_rcad_error() {
    fn read_a_solid_out_of(bytes: &[u8]) -> rcad::Result<()> {
        let file = StepFile::read(bytes)?;
        Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())?;
        Ok(())
    }

    let no_solid = b"ISO-10303-21;\n\
        HEADER;\n\
        FILE_DESCRIPTION((''),'2;1');\n\
        FILE_NAME('','',(''),(''),'','','');\n\
        FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));\n\
        ENDSEC;\n\
        DATA;\n\
        #1=(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT(.MILLI.,.METRE.));\n\
        #2=(NAMED_UNIT(*)PLANE_ANGLE_UNIT()SI_UNIT($,.RADIAN.));\n\
        #3=(NAMED_UNIT(*)SOLID_ANGLE_UNIT()SI_UNIT($,.STERADIAN.));\n\
        #4=(GEOMETRIC_REPRESENTATION_CONTEXT(3)\
        GLOBAL_UNIT_ASSIGNED_CONTEXT((#1,#2,#3))REPRESENTATION_CONTEXT('',''));\n\
        #5=CARTESIAN_POINT('',(1.,1.,1.));\n\
        #6=VERTEX_POINT('',#5);\n\
        ENDSEC;\n\
        END-ISO-10303-21;\n";

    let failure = read_a_solid_out_of(no_solid).expect_err("a lone point is not a solid");
    assert!(
        matches!(failure, Error::StepImport(Refusal::TheFileHoldsNoSolid)),
        "expected the facade's own StepImport variant, got {failure:?}"
    );
}

/// The other half of the reader: bytes that are not Part 21 at all, still
/// reached through one `?` into [`rcad::Error`].
#[test]
fn bytes_that_are_not_part_21_at_all_refuse_as_a_rcad_error() {
    fn read_a_solid_out_of(bytes: &[u8]) -> rcad::Result<()> {
        let file = StepFile::read(bytes)?;
        Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())?;
        Ok(())
    }

    let failure = read_a_solid_out_of(b"not a step file").expect_err("no ISO-10303-21; header");
    assert!(
        matches!(failure, Error::StepSyntax(_)),
        "expected the facade's own StepSyntax variant, got {failure:?}"
    );
}
