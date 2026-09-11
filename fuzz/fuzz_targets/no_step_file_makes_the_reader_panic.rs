//! Arbitrary bytes into the Part 21 reader, which must never panic.
//!
//! decisions.md §42.12 criterion 5, and CLAUDE.md's rule that bad caller input
//! is an `Err` and never a crash. A STEP file is *entirely* caller input, so
//! this is the whole surface: truncated, not UTF-8, nested a hundred thousand
//! deep, or written on purpose to break something.
//!
//! Everything a caller can reach afterwards is walked too, because a reader
//! that returns `Ok` and then panics when somebody looks at what it read has
//! only moved the crash.
//!
//! **And the mapping is fuzzed and not only the parse.** Arbitrary bytes that
//! happen to parse become an arbitrary *entity graph*, and that graph is what
//! `Import::read` walks: a `CLOSED_SHELL` pointing at itself, an `EDGE_CURVE`
//! whose ends are a `PLANE`, a `CIRCLE` of radius `NaN`. Every one of those is
//! caller input and every one of them owes an `Err`. Whatever imports is then
//! swept with `Model::validate` under a certified policy, because §42.12's own
//! criterion 9 says an imported model that does not validate is a defect —
//! and a defect found by a fuzzer is cheaper than one found by a file.

#![no_main]

use libfuzzer_sys::fuzz_target;
use rcad_fuzz_timing::timed;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};

fuzz_target!(|bytes: &[u8]| {
    timed(env!("CARGO_BIN_NAME"), bytes, || {
        everything_one_file_is_put_through(bytes);
    });
});

/// One draw of bytes, read and then walked everywhere a caller could walk it.
fn everything_one_file_is_put_through(bytes: &[u8]) {
    let Ok(file) = StepFile::read(bytes) else {
        return;
    };
    let _ = file.header().schemas();
    let _ = file.header().record("FILE_NAME");
    let _ = file.how_many_of_each_type();
    let _ = file.every_reference_resolves();
    for broken in file.references_that_point_nowhere() {
        assert!(
            file.instance(broken.to).is_none(),
            "a reference called broken must be one nothing defines"
        );
    }
    for instance in file.instances() {
        assert_eq!(
            file.instance(instance.id()).map(rcad_step::Instance::id),
            Some(instance.id()),
            "every instance has to be findable by the id it was written under"
        );
        for part in instance.parts() {
            assert!(
                instance.part(part.keyword()).is_some(),
                "an instance holds the type names it says it holds"
            );
            let _ = file.instances_named(part.keyword()).count();
        }
    }

    let mut log = ProvenanceLog::new();
    let Ok(import) = Import::read(&file, &Policy::certified(), &mut log) else {
        return;
    };
    import
        .model()
        .validate(&Policy::certified(), &mut log)
        .expect("a model this reader handed back has already been through the sweep");
    let _ = import.declared_unit();
    let _ = import.declared_uncertainty();
    let _ = import.asks_for_more_precision_than_its_numbers_carry();
    let _ = import.shapes_this_reader_did_not_take();
    for (_, solid) in import.solids() {
        let _ = import
            .model()
            .solid(*solid)
            .expect("a key this import handed out is this import's model's own")
            .box_around_it();
    }
}
