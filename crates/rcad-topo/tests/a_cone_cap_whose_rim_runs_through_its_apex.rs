//! A cone face whose rim has a corner at the cone's apex, and what
//! `which_pole_this_cap_reaches` in `mesh.rs` makes of it (Q578,
//! `docs/decisions.md` §199.4).
//!
//! **Unfinished: no test here yet.** Measured so far, with a scratch print
//! where `which_pole_this_cap_reaches` answers `None` or is handed a rim with
//! a corner on a pinch: across every test of the workspace and the slow tier,
//! it never answers `None`, and it is never handed a cone's rim at all with a
//! corner at the apex. Every rim it is handed with a corner on a pinch is a
//! ball's, and each of those reaches the other pole.
//!
//! The faces still to build here are a pointed cone whose skin carries a seam
//! up one ruling from the apex, a block whose face lies on a plane touching
//! the cone along one ruling, and a cone fused with a bar whose wall runs
//! through the apex.
