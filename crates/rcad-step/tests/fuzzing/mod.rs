//! What the two `rcad-step` fuzz targets do with a stranger's bytes, held here
//! so that the suite replays the same lines the fuzzer explores.
//!
//! `fuzz/fuzz_targets/no_step_file_makes_the_reader_panic.rs` and
//! `no_model_makes_the_writer_panic.rs` reach across to [`reader`] and
//! [`writer`] with `#[path]` rather than copying them, which is the shape
//! `where_two_surfaces_cross.rs` already uses for `rcad-geom` (§29.9). A
//! property added to one of those files is explored by `cargo fuzz` and
//! replayed on every commit without either side being edited.

pub mod reader;
pub mod writer;
