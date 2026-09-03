//! A fourth kind of [`EdgeRun`] must not arrive in silence either.
//!
//! `a_sixth_kind_of_surface.rs` is this test's twin and says the argument in
//! full: [`Surface`](rcad_geom::Surface) is `#[non_exhaustive]` and lives in
//! `rcad-geom`, so every match over it in this crate is *required* to carry a
//! wildcard and a sixth kind is not a compile error here. The same is true of
//! [`EdgeRun`], and of [`SurfaceMeet`](rcad_geom::SurfaceMeet) beside it — and
//! the vocabulary of crossings did grow this milestone, so this is not a
//! rehearsal.
//!
//! # What a run kind that arrived unnoticed used to get
//!
//! Three sites in this crate read a run or a crossing and used to draw a
//! conclusion from one they did not know:
//!
//! - `SolidBuilder::add_edge` skipped the zero-length check, because an
//!   unrecognised run reads as *not straight*. An edge whose two corners are
//!   one point walked in through the public door, and
//!   `Model::validate` never asked again — it reads the same list.
//! - `fuse::cuts::cuts_between` panicked, crashing a fuse over two solids that
//!   were as sound as they had ever been.
//! - `fuse::cuts::check_it_does_not_cross_itself` **skipped the pair without a
//!   word**, which is the worse of the two: the solid came back sound and
//!   nobody had looked at it.
//!
//! All three now refuse by name — `ADraftEdgeHasNoLength` at the door, and
//! `ACrossingThisCannotBeOrderedAlong` in the fuse, the name its twin in
//! `fuse/arrange.rs` already used for the same arrival. A new word in
//! `rcad-geom` is neither bad caller input nor a broken invariant here, and
//! CLAUDE.md's rule leaves a refusal as the only honest answer.
//!
//! **Measured, not argued.** A clone of this tree grew a fourth `EdgeRun`
//! variant and a fifth `SurfaceMeet` variant, four arms inside `rcad-geom` were
//! filled in to make that
//! clone build at all — which is exhaustiveness biting in the crate that owns
//! the enum, and the whole of what does not happen here — and each site was run
//! both ways:
//!
//! | site | before | after |
//! | --- | --- | --- |
//! | `add_edge`, both corners one point | `Ok(DraftEdge)` | `Err(ADraftEdgeHasNoLength)` |
//! | two balls fused | panic at `cuts.rs:128` | `Err(ACrossingThisCannotBeOrderedAlong)` |
//! | one solid of two overlapping balls | `Ok(())` | `Err(ACrossingThisCannotBeOrderedAlong)` |
//!
//! The third row is the one worth staring at: a solid nobody had looked at came
//! back sound.
//!
//! # What this test can reach, and what it cannot
//!
//! **A fourth run kind cannot be forged from outside `rcad-geom`**, so nothing
//! here can drive those three refusals — the clone above is the only way to,
//! and a clone is not a test. That is the whole difficulty: the paths exist for
//! a day when the enum grows, and on that day the compiler says nothing.
//! `cargo check -p rcad-topo --all-targets` on that same clone produces **zero
//! errors and zero warnings**, and the test below is the only thing in the crate
//! that goes red.
//!
//! So the guard is the same one the surfaces got: the `Deserialize` derive is
//! generated from the variant list itself, and asking it to name every kind
//! there is fails the moment there is another. It is not a tautology and it is
//! not a substitute for the refusals — it is the notice that they now matter.
//!
//! [`SurfaceMeet`](rcad_geom::SurfaceMeet) has **no such derive**, so there is
//! no list to ask it for and no guard of this shape to write. Nor does
//! `just check-wildcards` move for it: a match over a `SurfaceMeet` out here
//! cannot be exhaustive, the compiler makes every one carry a wildcard, and a
//! wildcard already there is already counted — so a fifth shape moves no number.
//! For the two sites in `fuse/cuts.rs` the refusal *is* the guard, which is why
//! they were worth changing before anything could reach them.

use rcad_geom::{EdgeRun, Plane, Point, Surface, Vector};
use rcad_topo::{Error, SolidBuilder};
use serde::Deserialize;
use serde::de::IntoDeserializer;
use serde::de::value::{Error as ReadError, StrDeserializer};

/// The kinds of run every hand-kept list in this crate was written against.
///
/// `the_ends_are_the_whole_of_it` in `builder.rs` and `check_edges` in
/// `validate.rs` are the two that decide whether an edge with both corners in
/// one place is a closed loop or an edge with no length, and they must agree:
/// the builder writes its description into a throwaway model and runs the sweep
/// over it, so a sweep that refused more would turn a named builder error into
/// a different one, and a sweep that refused less would let the difference
/// through both doors.
#[test]
fn the_three_kinds_of_run_this_crate_was_written_against() {
    let name: StrDeserializer<ReadError> = "a fourth kind".into_deserializer();
    let refused =
        EdgeRun::deserialize(name).expect_err("`EdgeRun` read back a kind that does not exist");

    assert_eq!(
        refused.to_string(),
        "unknown variant `a fourth kind`, expected one of `Straight`, \
         `RightRoundWhereItsFacesCross`, `PartOfWhereItCrosses`",
        "the kinds of `EdgeRun` changed. Every place in this crate that reads a \
         run now has a kind it has never seen, and the two that matter most are \
         `the_ends_are_the_whole_of_it` in `builder.rs` and `check_edges` in \
         `validate.rs`: both refuse a kind they do not know rather than take it \
         for a loop, so a new closed kind is refused until it is named in both \
         (docs/decisions.md §40)"
    );
}

/// The two runs that are allowed to bring an edge back to the corner it left.
///
/// Both are named, and everything else — [`EdgeRun::Straight`] today, a fourth
/// kind tomorrow — is refused. Trimming this list to one, or widening it to
/// "anything that is not straight", is what this pins.
#[test]
fn only_a_run_that_names_a_curve_may_start_and_finish_at_one_corner() {
    for run in [
        EdgeRun::RightRoundWhereItsFacesCross,
        EdgeRun::PartOfWhereItCrosses {
            and: a_plane(),
            picked_by: smallvec::SmallVec::new(),
        },
    ] {
        let mut builder = SolidBuilder::new();
        let corner = builder
            .add_vertex(Point::origin())
            .expect("the origin is finite");

        builder
            .add_edge(corner, corner, run.clone())
            .unwrap_or_else(|refused| {
                panic!("{run:?} names a curve its corner sits on: {refused:?}")
            });
    }

    let mut builder = SolidBuilder::new();
    let corner = builder
        .add_vertex(Point::origin())
        .expect("the origin is finite");

    let refused = builder.add_edge(corner, corner, EdgeRun::Straight);

    assert!(
        matches!(refused, Err(Error::ADraftEdgeHasNoLength { edge: 0 })),
        "a straight edge is the whole of what its two ends say, so one corner \
         twice is no edge at all: got {refused:?}"
    );
}

fn a_plane() -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("a normal with a length is a plane"),
    )
}
