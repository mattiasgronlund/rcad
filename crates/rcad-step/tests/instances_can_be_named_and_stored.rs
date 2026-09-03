//! `StepFile::instances` and `StepFile::instances_named` used to hand back
//! `impl Iterator`. A `let` binding could still hold one, by inference -- what
//! a caller could not do is *write* the type anywhere: a struct field, a
//! function's own return type, a generic bound named rather than elided.
//! §44.9's third cheap closure narrows both to types a caller can name.
//!
//! This is the demonstration and not just the doc comment: `Held` below is a
//! struct built entirely from types this crate exports, and `held_from` hands
//! one back from a function of its own -- exactly what `impl Iterator` could
//! not be put in either place for. `Instance` was already nameable and
//! exported before this closure; what changed is `instances()`'s own return
//! type, now the standard library's `std::slice::Iter`, and `InstancesNamed`,
//! which did not exist to write at all.

use rcad_step::{Instance, InstancesNamed, StepFile};

/// A struct holding both kinds of iterator `StepFile` hands back, by name.
struct Held<'a> {
    all: std::slice::Iter<'a, Instance>,
    breps: InstancesNamed<'a>,
}

fn held_from(file: &StepFile) -> Held<'_> {
    Held {
        all: file.instances(),
        breps: file.instances_named("MANIFOLD_SOLID_BREP"),
    }
}

#[test]
fn both_iterators_can_be_named_stored_and_still_count_right() {
    let bytes = b"ISO-10303-21;
HEADER;
FILE_DESCRIPTION((''),'2;1');
FILE_NAME('a','2026-09-03T00:00:00',(''),(''),'','','');
FILE_SCHEMA(('AUTOMOTIVE_DESIGN { 1 0 10303 214 1 1 1 1 }'));
ENDSEC;
DATA;
#1=CARTESIAN_POINT('',(0.,0.,0.));
#2=MANIFOLD_SOLID_BREP('',#3);
#3=CLOSED_SHELL('',());
#4=MANIFOLD_SOLID_BREP('',#3);
ENDSEC;
END-ISO-10303-21;
";
    let file = StepFile::read(bytes).expect("this file is well formed");
    let held = held_from(&file);

    assert_eq!(held.all.len(), 4, "the file wrote four instances");
    assert_eq!(
        held.breps.count(),
        2,
        "two of the four are a MANIFOLD_SOLID_BREP"
    );
}
