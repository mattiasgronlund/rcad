//! A sixth kind of [`Surface`] must not arrive in silence.
//!
//! Five times this milestone a hand-kept list of surface kinds did not learn
//! about a new one, and the worst of them told every capped bend of pipe that
//! its crossing could not be ordered along — a refusal that was not true.
//!
//! Three doc comments in this crate said the guard against that was the
//! compiler: name every kind, leave the `_` for the one that does not exist
//! yet, and a sixth kind is a compile error. **In this crate that is false.**
//! [`Surface`] is `#[non_exhaustive]` and lives in `rcad-geom`, so every match
//! over it here is *required* to carry a wildcard and can never be exhaustive.
//! Adding a sixth variant in a clone of this tree and running
//! `cargo check -p rcad-topo` produces zero errors, both before and after the
//! lists were completed. The one mechanism that would do it,
//! `non_exhaustive_omitted_patterns`, is unstable on the pinned 1.96
//! (rust-lang/rust#89554). Inside `rcad-geom` the claim *is* true, which is why
//! `rcad_geom::walk` can make it — exhaustiveness bites in the crate that owns
//! the enum.
//!
//! `just check-wildcards` does catch a sixth kind at most sites, and loudly:
//! twenty matches in this crate that name all five today start warning the
//! moment there is a sixth, measured by running
//! `cargo clippy -p rcad-topo --all-targets -- -W clippy::wildcard_enum_match_arm`
//! against the same clone — thirty sites become fifty, none stop.
//!
//! **The three it does not catch are the three lists of surface pairs**, and
//! they are where all five silent drifts happened. Clippy asks only what the
//! scrutinee's own type is, and theirs is a tuple or an array of two
//! `Surface`, so it never looks inside; the composite scan behind the other
//! half of `check-wildcards` counts sites and not variants, so its number does
//! not move either. Naming all twenty-five pairs at each of them does not
//! change that — it puts every pair in front of a reader, which is worth
//! having, and it is not a guard.
//!
//! So this test is the guard. It is not a tautology: it asks the `Deserialize`
//! derive, which is generated from the variant list itself, to name every kind
//! there is. A sixth kind changes that list and this fails, which is the notice
//! the five silent drifts never got.

use rcad_geom::Surface;
use serde::Deserialize;
use serde::de::IntoDeserializer;
use serde::de::value::{Error, StrDeserializer};

/// The kinds this crate's hand-kept lists of surface pairs were written
/// against.
///
/// `a_pair_that_crosses_in_a_conic` and `a_plane_along_a_pipes_axis` in
/// `fuse/arrange.rs` and `the_crossing_of_these_runs_straight` in
/// `fuse/assemble.rs` each name all twenty-five ordered pairs of these five and
/// send everything else to a `_`. A sixth kind falls into those wildcards and
/// gets whatever the wildcard says, which for all three is `false`.
#[test]
fn the_five_kinds_the_fuses_lists_of_surface_pairs_were_written_against() {
    let name: StrDeserializer<Error> = "a sixth kind".into_deserializer();
    let refused =
        Surface::deserialize(name).expect_err("`Surface` read back a kind that does not exist");

    assert_eq!(
        refused.to_string(),
        "unknown variant `a sixth kind`, \
         expected one of `Plane`, `Sphere`, `Cylinder`, `Cone`, `Torus`",
        "the kinds of `Surface` changed. Every `_` arm over a `Surface` in this \
         crate now covers a kind that exists rather than one that does not, and \
         `cargo clippy -p rcad-topo --all-targets -- -W clippy::wildcard_enum_match_arm` \
         lists them. Visit each before updating this test (docs/decisions.md §40)"
    );
}
