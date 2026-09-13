//! Arbitrary bytes into the Part 21 reader, which must never panic.
//!
//! `docs/decisions.md` §42.12 criterion 5, and CLAUDE.md's rule that bad caller
//! input is an `Err` and never a crash. A STEP file is *entirely* caller input,
//! so this is the whole surface: truncated, not UTF-8, nested a hundred
//! thousand deep, or written on purpose to break something.
//!
//! Everything a caller can reach afterwards is walked too, because a reader
//! that returns `Ok` and then panics when somebody looks at what it read has
//! only moved the crash.
//!
//! **And the mapping is walked and not only the parse.** Arbitrary bytes that
//! happen to parse become an arbitrary *entity graph*, and that graph is what
//! `Import::read` walks: a `CLOSED_SHELL` pointing at itself, an `EDGE_CURVE`
//! whose ends are a `PLANE`, a `CIRCLE` of radius `NaN`. Every one of those is
//! caller input and every one of them owes an `Err`. Whatever imports is then
//! swept with `Model::validate` under a certified policy, because §42.12's own
//! criterion 9 says an imported model that does not validate is a defect — and
//! a defect found by a fuzzer is cheaper than one found by a file.
//!
//! That paragraph was written in the present tense and was **false of the
//! checked-in corpus for as long as nothing counted**: every one of the 135
//! seeds that parsed stopped at `what_the_file_declared`, so `solid_from` and
//! the validate sweep were reached by none of 907. [`Reached`] is what makes
//! that sayable, and `a_block_that_imports` is the seed that fixes it.

// Shared between a test binary and a fuzz target, and each uses a subset.
#![allow(dead_code)]

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};

/// How far one draw of bytes got.
///
/// A count per stage and not one total, because the stages fail for entirely
/// different reasons and a single number would let the deep ones fall to
/// nothing behind the shallow ones. That is the same argument
/// `every_pair_of_surfaces_the_fuzzer_draws_is_named_honestly.rs` makes for its
/// three oracles, and the reader corpus is the case that proves it: 135 of 907
/// parsed and **none at all** imported.
#[derive(Clone, Copy, Debug, Default)]
pub struct Reached {
    /// Byte strings handed to the reader.
    pub files: usize,
    /// Byte strings that parsed into an entity graph.
    pub parsed: usize,
    /// Instances walked across every graph that parsed.
    pub instances: usize,
    /// Graphs the mapping turned into a model.
    pub imported: usize,
    /// Solids in those models.
    pub solids: usize,
    /// Graphs the mapping refused by name, having got past the parse.
    pub refused: usize,
}

impl Reached {
    /// What two draws reached between them.
    #[must_use]
    pub fn and(self, other: Self) -> Self {
        Self {
            files: self.files + other.files,
            parsed: self.parsed + other.parsed,
            instances: self.instances + other.instances,
            imported: self.imported + other.imported,
            solids: self.solids + other.solids,
            refused: self.refused + other.refused,
        }
    }
}

/// **One draw of bytes, read and then walked everywhere a caller could walk
/// it.**
///
/// The fuzz target and the suite's replay both call this and nothing narrower.
///
/// # Panics
///
/// Whenever the reader breaks a promise it made: a reference it called broken
/// that something does define, an instance not findable under its own id, a
/// type name an instance says it holds and then does not, or a model it handed
/// back that `Model::validate` rejects.
#[must_use]
pub fn everything_one_file_is_put_through(bytes: &[u8]) -> Reached {
    let mut reached = Reached {
        files: 1,
        ..Reached::default()
    };
    let Ok(file) = StepFile::read(bytes) else {
        return reached;
    };
    reached.parsed = 1;
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
        reached.instances += 1;
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
        reached.refused = 1;
        return reached;
    };
    reached.imported = 1;
    reached.solids = import.solids().len();
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
    reached
}
