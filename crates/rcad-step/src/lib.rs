//! Reads an ISO 10303-21 file — a STEP "Part 21" file — into an entity graph,
//! and that graph's solids into a [`Model`](rcad_topo::Model). [`write`] goes
//! back the other way.
//!
//! **This reader maps solids bounded by planes, spheres, cylinders, cones and
//! tori, whose edges run along straight lines, circles and ellipses. Every
//! other surface and curve kind is refused by the name the file gives it. A
//! model that imports is internally sound and is not checked for interference
//! with anything else in the file.** That is §42.11's opening sentence, and it
//! is the first thing to know.
//!
//! # Two halves
//!
//! [`StepFile::read`] turns bytes into entities. It does not know what an
//! `ADVANCED_FACE` means, does not build geometry, and never mentions a
//! `Model`: what it hands back is exactly what the file said, a header, a list
//! of instances in file order, and an index from `#123` to the instance it
//! names.
//!
//! [`Import::read`] walks that graph — `manifold_solid_brep`, `closed_shell`,
//! `advanced_face`, `face_bound`, `edge_loop`, `oriented_edge`, `edge_curve`,
//! `vertex_point` — into one `Model`, or refuses it by a [`Refusal`] that says
//! which `#id` stopped it and why.
//!
//! # Writing, and the oracle it exists for
//!
//! [`write`] turns a `Model`'s solids back into those same entities, or refuses
//! by a [`CannotWrite`] that says what it could not put in the file. What it is
//! *for* is a check the reader could never make on its own: a solid this kernel
//! built, written out and read back, has to measure the same bit for bit and
//! fuse the same against a third solid, piece for piece
//! (`crates/rcad-step/tests/a_solid_written_out_and_read_back_is_the_same_solid.rs`).
//!
//! **A round trip checks a writer against the reader beside it**, so a
//! misunderstanding of AP203 the two share cancels out and passes green. Two
//! things stand against that: `tests/written_by_hand/` holds files nothing in
//! this crate produced, and `step-io` — a dev-dependency only — reads the bytes
//! this writer produces in
//! `crates/rcad-step/tests/a_second_reader_reads_what_this_writer_wrote.rs`.
//!
//! **Nothing rounds on the way out.** A coordinate is written with `{:?}`,
//! which is the shortest decimal text that reads back as the same `f64`, so
//! there is no conversion cost to publish a bound for — there is no conversion.
//! `every_number_this_writer_writes_reads_back_bit_for_bit` is that claim as a
//! test, through this crate's own reader.
//!
//! # How often a real file imports
//!
//! Measured, per corpus, never blended, by
//! `crates/rcad-step/tests/how_often_a_real_file_imports.rs`:
//!
//! | | files | imported |
//! |---|---:|---:|
//! | files this kernel wrote itself (the control) | 88 | **88** |
//! | `kicad/`, KiCad's own footprint library | 323 | **165** |
//! | `nist/`, ASME Y14.5 GD&T certification parts | 44 | **1** |
//!
//! The two real corpora are different populations and are never added together.
//! What refuses the rest is grouped and counted in that test's own output, and
//! the two biggest are not about this reader: a face bounded by two steps,
//! which this kernel has no wire for, and a corner the file puts on a curved
//! face that an exact predicate says is not on it.
//!
//! # Nothing is converted and nothing is healed
//!
//! A file written in inches comes back in inches, with the numbers the file
//! wrote. [`Scale`](rcad_geom::Scale) carries a unit and never multiplies by
//! one, so there is no conversion to round (decisions.md §3.5); a unit this
//! kernel has no name for is refused rather than scaled into one it has.
//! The declared uncertainty is read and published by
//! [`Import::declared_uncertainty`] and **never acted on**: no predicate
//! anywhere is asked a different question because of it.
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

mod names;

mod units;

mod brep;

mod import;

mod refusal;

mod write;

pub use error::{Error, Problem};
pub use graph::{
    BrokenReference, EntityId, Header, Instance, InstancesNamed, Record, StepFile, Value,
};
pub use import::{Import, ProbeReport, probe};
pub use refusal::Refusal;
pub use write::{CannotWrite, write};

/// How deep lists, and values wearing type names, may nest.
///
/// The reader walks nested values with the call stack, so this is what stands
/// between a file full of `(((((` and a stack overflow. The deepest nesting any
/// of the 367 files in M5's corpus reaches is 3, measured and asserted to be
/// clear of this limit by
/// `crates/rcad-step/tests/every_file_in_the_corpus_reads.rs`.
pub const DEEPEST_NESTING: usize = 64;
