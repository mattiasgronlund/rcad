//! [`EdgeView::is_closed`] is the one list of run kinds that close, and it
//! refuses a kind it cannot read.
//!
//! `a_fourth_kind_of_run.rs` is this file's parent and says the argument in
//! full: [`EdgeRun`] is `#[non_exhaustive]` and lives in `rcad-geom`, so a
//! fourth kind arrives in this crate with no compile error at all, and this
//! milestone did grow the vocabulary.
//!
//! # The site this closes, and why it was the last of its kind
//!
//! `is_closed` used to be `run == EdgeRun::RightRoundWhereItsFacesCross`. A bare
//! `==` has no wildcard for `just check-wildcards` to count and nothing for
//! clippy's `wildcard_enum_match_arm` to warn about — not now, and **not the day
//! the enum grows** — and six production sites read it, every one of them taking
//! a kind nothing here knows for an open edge:
//!
//! | site | what it does with a run kind it cannot read |
//! | --- | --- |
//! | `view.rs`'s own `segment` | refuses; a chord is not a curve and `None` is not one either |
//! | `fuse/mod.rs`, the shortest chord of the inputs | refuses; it answers `bool` and cannot skip |
//! | `mesh.rs`, the closed-edge direction rule | refuses; the alternative is half the caps walked backwards |
//! | `holds.rs`, the winding of a round wire | refuses under its own name, which points at the face |
//! | `validate.rs`, `check_dynamic_range` | refuses; a chord of nothing names a number and no entity |
//! | `rcad-step`'s writer | refuses; a `LINE` where the model holds a curve is a file about another shape |
//!
//! Six copies of one list is the drift `the_ends_are_the_whole_of_it` was pulled
//! back into one list to stop (§44.11), so the list is in `is_closed` and the
//! answer is a [`Result`]. Not a three-way answer of its own:
//! [`Decision`](rcad_predicates::Decision)'s third value is a measurement the
//! arithmetic could not settle, and this one is not about the edge at all — it
//! is this crate not knowing a word, with no further arithmetic that would
//! settle it.
//!
//! # What this file can reach, and what it cannot
//!
//! **A fourth run kind cannot be forged from outside `rcad-geom`**, so nothing
//! here can drive those six refusals. What is pinned below is the list itself,
//! in both directions and through the public door, and the *shape* of the
//! answer: a `Result` binding that stops compiling the day somebody hands back a
//! bare `bool` again.
//!
//! The refusals themselves were measured on a clone of this tree carrying a
//! fourth `EdgeRun` variant, with `Model::add_closed_edge` writing it on every
//! closed edge — so a plain round bar's two cap circles carry it. `cargo check
//! --workspace` on that clone produces **zero errors**, which is what "arrives
//! in silence" means.
//!
//! A closed kind has to be named at three doors before anything can be built out
//! of it at all: `the_ends_are_the_whole_of_it`, `Model::wire_closes_on_one_edge`
//! and `SolidBuilder::closes_on_one_edge`. Unnamed, the bar is refused at the
//! last two as `ADraftWireIsTooShort` — a wire of one edge that does not close is
//! no wire. **Those last two are bare `==` on the same list**, one in `model.rs`
//! and one in `builder.rs`, and they are the next two copies of it to pull in;
//! they fail safe today, which is why this lane left them. Those three words were
//! written in the clone, which is what the day this happens for real looks like,
//! and then:
//!
//! | the bar's fate | `is_closed` as `==` | `is_closed` as this list |
//! | --- | --- | --- |
//! | `Model::fuse` with a second bar | **`Ok(fused)`** | `Err(ACrossingThisCannotBeOrderedAlong)` |
//! | `rcad_step::write` | `Err(ACurveTheFacesEitherSideOfItDoNotName { first: "plane", second: "pipe", closed: false })` | `Err(ARunKindThisWriterHasNoCurveFor)` |
//! | `SolidView::mesh` | `Err(AWireWillNotLieFlatOnItsFace { face })` | `Err(ACrossingThisCannotBeOrderedAlong)` |
//! | `Model::validate` | `Err(ACrossingThisCannotBeOrderedAlong)` | unchanged |
//!
//! **The first row is the whole lane in one line**: a fuse of two solids whose
//! edges run along a curve nothing here can name came back with an answer, and
//! committed it. Putting `unwrap_or(false)` back at that one site — and nowhere
//! else — turns the refusal back into `Ok(fused)`, so it is that site and not
//! another.
//!
//! The middle two rows are the same fate under a better name: both refused
//! before, one of them while saying a plane and a pipe do not name their
//! crossing, which they do. The last row does not move at all, because
//! `check_closed_edges_are_walked_the_way_their_surfaces_wind` refuses the same
//! arrival earlier in the sweep; `check_dynamic_range` asking properly is depth
//! behind that check rather than a fate this changes.

use rcad_geom::{EdgeRun, Isometry, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Result};

/// The three kinds of run, in both directions, through the public door.
///
/// A round bar is straight sides and two cap circles; a ball's one edge is its
/// seam, which is a [`EdgeRun::PartOfWhereItCrosses`] and **open**. The seam is
/// what stops this list being written as "anything that is not straight closes",
/// which is the other way to get it wrong.
#[test]
fn the_kinds_of_run_that_bring_an_edge_back_to_the_corner_it_left() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    model
        .add_round_bar(3.0, 5.0, &Isometry::identity())
        .expect("a bar three across and five tall");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a round bar this kernel built is sound");

    for edge in model.edges() {
        let closes = edge
            .is_closed()
            .expect("a bar's runs are kinds this crate names");
        assert_eq!(
            closes,
            edge.run() == EdgeRun::RightRoundWhereItsFacesCross,
            "a bar's cap circles close and its straight sides do not"
        );
    }

    let mut ball = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    ball.add_ball(2.0, &Isometry::identity())
        .expect("a ball two across");

    for edge in ball.edges() {
        assert!(
            matches!(edge.run(), EdgeRun::PartOfWhereItCrosses { .. }),
            "a ball's one edge is its seam"
        );
        assert!(
            !edge.is_closed().expect("a seam is a kind this crate names"),
            "a seam runs pole to pole between two different corners, so a run that is not \
             straight is not thereby closed"
        );
    }
}

/// The answer is a [`Result`] and not a bare `bool`.
///
/// This is a compile-time pin and it is the only kind there is: a fourth run
/// kind cannot be forged from outside `rcad-geom`, so nothing here can reach the
/// refusal. What it stops is the shape going back to the one that could not say
/// *"a kind I do not know"* at all.
#[test]
fn a_reader_is_handed_an_answer_that_can_refuse() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    model
        .add_round_bar(3.0, 5.0, &Isometry::identity())
        .expect("a bar three across and five tall");

    let edge = model.edges().next().expect("a bar has edges");
    let answer: Result<bool> = edge.is_closed();

    assert!(answer.is_ok(), "every kind of run a bar carries is named");
}
