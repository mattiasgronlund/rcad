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

#![no_main]

use libfuzzer_sys::fuzz_target;
use rcad_step::StepFile;

fuzz_target!(|bytes: &[u8]| {
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
});
