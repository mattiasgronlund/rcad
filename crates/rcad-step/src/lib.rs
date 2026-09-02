//! Reads an ISO 10303-21 file — a STEP "Part 21" file — into an entity graph.
//!
//! This is the half of a STEP importer that turns bytes into entities. It does
//! not know what an `ADVANCED_FACE` means, does not build geometry, and never
//! mentions a `Model`. What it hands back is exactly what the file said: a
//! header, a list of instances in file order, and an index from `#123` to the
//! instance it names (decisions.md §42.10, lane 3's first half).
//!
//! ```
//! use rcad_step::{StepFile, Value};
//!
//! let file = b"ISO-10303-21;
//! HEADER;
//! FILE_DESCRIPTION((''),'2;1');
//! FILE_NAME('','',(''),(''),'','','');
//! FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));
//! ENDSEC;
//! DATA;
//! #1=CARTESIAN_POINT('origin',(0.,0.,0.));
//! ENDSEC;
//! END-ISO-10303-21;
//! ";
//!
//! let read = StepFile::read(file).unwrap();
//! assert_eq!(read.header().schemas(), ["AUTOMOTIVE_DESIGN"]);
//! let point = read.instances().next().unwrap();
//! assert_eq!(point.parts()[0].keyword(), "CARTESIAN_POINT");
//! let coordinates = point.parts()[0].attribute(1).unwrap().as_list().unwrap();
//! assert_eq!(coordinates[0], Value::Real(0.0));
//! ```
//!
//! # Nothing is healed
//!
//! What this reader cannot read, it names — with the entity id where there is
//! one — and refuses. It never repairs, never guesses and never puts a default
//! in place of a value the file got wrong (decisions.md §42.9, which chose
//! option A for M5). Two consequences worth knowing before you use it:
//!
//! - `1` and `1.` stay an [`Value::Integer`] and a [`Value::Real`]. Files
//!   written for a schema wanting a real do sometimes write the integer, and
//!   deciding what to do about that belongs to whoever knows the schema.
//! - A `#123` that nothing defines is not dropped and does not stop the read.
//!   Every one of them is listed by
//!   [`StepFile::references_that_point_nowhere`], and
//!   [`StepFile::every_reference_resolves`] turns the list into a refusal for a
//!   caller who wants one. A reader that stopped would refuse a sound b-rep for
//!   a dangling reference in a tolerance annotation nobody was going to read.
//!
//! # A file is somebody else's bytes
//!
//! [`StepFile::read`] takes `&[u8]`, and no sequence of bytes makes it panic —
//! not a truncated file, not one that is not UTF-8, not a hostile one
//! (CLAUDE.md's rule that bad caller input is an `Err`; decisions.md §42.12
//! criterion 5). Values nested deeper than [`DEEPEST_NESTING`] are refused by
//! name, because the alternative is a stack overflow, and a crash is not an
//! `Err`.
//!
//! # What it does not read
//!
//! Named here rather than discovered later. Each is refused by a variant of
//! [`Problem`], never skipped.
//!
//! - Binary values, `"0F3A"` ([`Problem::BinaryValueThisReaderDoesNotRead`]).
//!   None of the 367 files in M5's corpus holds one and no entity a b-rep
//!   walks carries one.
//! - Part 21 edition 3's `ANCHOR`, `REFERENCE` and `SIGNATURE` sections, and a
//!   `DATA` section written with parameters
//!   ([`Problem::SectionThisReaderDoesNotRead`], and
//!   [`Problem::NotWhatWasExpected`] for the second). None of the 367 appear.
//! - `&SCOPE` … `ENDSCOPE`, which no file in the corpus uses either. It is
//!   refused as an unexpected byte rather than by its own name.
//! - Alphabet pages other than ISO 8859-1
//!   ([`Problem::AlphabetThisReaderDoesNotHave`]).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod graph;

mod read;

mod token;

mod text;

mod error;

mod geometry;

mod units;

mod brep;

mod import;

mod refusal;

pub use error::{Error, Problem};
pub use graph::{BrokenReference, EntityId, Header, Instance, Record, StepFile, Value};
pub use import::Import;
pub use refusal::Refusal;

/// How deep lists, and values wearing type names, may nest.
///
/// The reader walks nested values with the call stack, so this is what stands
/// between a file full of `(((((` and a stack overflow. The deepest nesting any
/// of the 367 files in M5's corpus reaches is 3, measured and asserted to be
/// clear of this limit by
/// `crates/rcad-step/tests/every_file_in_the_corpus_reads.rs`.
pub const DEEPEST_NESTING: usize = 64;
