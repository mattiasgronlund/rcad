//! What the `rcad-topo` fuzz target does with a stranger's bytes, held here so
//! that the suite replays the same lines the fuzzer explores.
//!
//! `fuzz/fuzz_targets/two_solids_fused_by_a_stranger.rs` reaches across to
//! [`fuse`] with `#[path]` rather than copying it, which is the shape
//! `where_two_surfaces_cross.rs` already uses for `rcad-geom` (§29.9).
//!
//! `the_door.rs` sits beside it and is declared by nothing here on purpose:
//! `fuse.rs` and `crates/rcad-step/tests/fuzzing/writer.rs` each reach it by
//! `#[path]`, so it is compiled once per binary that wants it and there is one
//! copy of it in the tree.

pub mod fuse;
