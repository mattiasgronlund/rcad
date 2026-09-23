//! A new kind of [`EdgeRun`] must not arrive in silence either.
//!
//! # The fourth kind arrived, and this file is what made it loud
//!
//! §216 added [`EdgeRun::RightRoundWhereItCrosses`] — a closed rim that carries
//! the surface making the cut, for a frustum's wall split in two by a leaning
//! plane. It is the arrival this file was written against, and it went the way
//! the file says it should: the list below went red, and naming the kind at
//! every site that reads one was what turned it green again. Four sites in
//! `rcad-topo` had to be told about it —
//! [`EdgeView::is_closed`](rcad_topo::EdgeView::is_closed),
//! `the_ends_are_the_whole_of_it` in `builder.rs`,
//! `the_curved_surface_a_run_names` and
//! `check_closed_edges_are_walked_the_way_their_surfaces_wind` in `validate.rs`
//! — and the fourth of those is the one that caught it at run time: the sweep
//! refused the new rim under `ACrossingThisCannotBeOrderedAlong` with no
//! compile error anywhere, exactly as the paragraphs below predicted.
//!
//! So the numbers in the tables further down are the measurements that were
//! made on a clone before that happened, and they still say what they said.
//! What a **fifth** kind gets is what this file now asks about.
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
//!   one point walked in through the public door, and `check_edges` — which
//!   reads the same list — did not ask again.
//! - `fuse::cuts::cuts_between` panicked, crashing a fuse over two solids that
//!   were as sound as they had ever been.
//! - `fuse::cuts::check_it_does_not_cross_itself` **skipped the pair without a
//!   word**, which is the worse of the two: the solid came back sound and
//!   nobody had looked at it.
//!
//! Those three refuse now, under two names: `ADraftEdgeHasNoLength` at the
//! door, and `ACrossingThisCannotBeOrderedAlong` in the fuse — the name its
//! twin in `fuse/arrange.rs` already used for the same arrival. Three sites
//! carry two names because the last two are one function now,
//! `fuse::cuts::how_two_faces_meet`.
//!
//! **The first bullet is about `check_edges` and not about the sweep.** A
//! zero-length edge that gets past the length rule is still refused by
//! `check_dynamic_range`, which measures every edge that is not
//! `RightRoundWhereItsFacesCross` end to end and will not take a chord of
//! nothing for a feature size —
//! `tests/what_the_whole_sweep_says_about_a_two_step_wire.rs` hands it a whole
//! model and reads back
//! `Error::Geometry(FeatureSizeNotPositive { size: 0.0 })`. What the door's
//! refusal buys is the name: it points at the edge, where a feature size points
//! at a number.
//!
//! # A fourth site answers the same arrival a third way, on purpose
//!
//! `pickers_of` in `fuse/arrange.rs` reads a run too, and it **panics** rather
//! than refusing. That is right for the reason it gives: empty is what says
//! "this pair names one curve", so handing empty back for a kind that carries
//! pickers does not refuse — it silently picks the wrong curve of two, which is
//! a wrong answer and not a refusal.
//!
//! So a new word in `rcad-geom` is not answered one way here. It is refused
//! where there is something whole to refuse — an edge being described, a
//! crossing being ordered — and crashed where answering would put a different
//! curve where the caller asked for this one. Both calls are right and
//! **neither site says so about the other**, four hundred lines apart. What
//! they need is that rule written down once — `docs/decisions.md` is where it
//! belongs — with `pickers_of`'s `# Panics` and `crossing_for`'s refusal each
//! naming it, so the next site to read a run has something to be consistent
//! with instead of two precedents. This file is the pin that names both in one
//! place until then.
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
//! no list to ask it for and no test of this shape to write for it.
//!
//! `just check-wildcards` covers that half, and covers this one too. Clippy says
//! nothing about a wildcard that covers no variant that exists, so a match
//! naming every kind there is today, with the `_` §40 asks for underneath, is
//! silent now and loud the day the enum grows. Measured on the same clone —
//! `cargo clippy -p rcad-topo --all-targets -- -W clippy::wildcard_enum_match_arm`
//! warns **56** times against this tree and **59** against the clone, and the
//! three that arrive are `builder.rs` for the fourth run kind and `fuse/cuts.rs`
//! and `fuse/arrange.rs` for the fifth crossing shape. Those are the three sites
//! this file is about, and none of them would say a word if the lists were
//! written as `matches!`, which the lint never looks inside.

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
fn the_four_kinds_of_run_this_crate_was_written_against() {
    let name: StrDeserializer<ReadError> = "a fifth kind".into_deserializer();
    let refused =
        EdgeRun::deserialize(name).expect_err("`EdgeRun` read back a kind that does not exist");

    assert_eq!(
        refused.to_string(),
        "unknown variant `a fifth kind`, expected one of `Straight`, \
         `RightRoundWhereItsFacesCross`, `RightRoundWhereItCrosses`, \
         `PartOfWhereItCrosses`",
        "the kinds of `EdgeRun` changed. Every place in this crate that reads a \
         run now has a kind it has never seen, and the two that matter most are \
         `the_ends_are_the_whole_of_it` in `builder.rs` and `check_edges` in \
         `validate.rs`: both refuse a kind they do not know rather than take it \
         for a loop, so a new closed kind is refused until it is named in both \
         (docs/decisions.md §40)"
    );
}

/// The three runs that are allowed to bring an edge back to the corner it left.
///
/// All three are named, and everything else — [`EdgeRun::Straight`] today, a
/// fifth kind tomorrow — is refused. Trimming this list, or widening it to
/// "anything that is not straight", is what this pins.
#[test]
fn only_a_run_that_names_a_curve_may_start_and_finish_at_one_corner() {
    for run in [
        EdgeRun::RightRoundWhereItsFacesCross,
        EdgeRun::RightRoundWhereItCrosses { and: a_plane() },
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
