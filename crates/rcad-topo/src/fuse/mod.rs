//! Cutting two groups of solids against each other, ready to be put back
//! together.
//!
//! A fuse takes two groups of solids — the arguments `A` and the tools `B` —
//! cuts every face of one group against every face of the other, and then keeps
//! the pieces one selection rule asks for. This module holds the cutting half.
//! Selection, shell reassembly and the commit are not here yet; the seam they
//! attach to is the last section below.
//!
//! # What it rests on
//!
//! - **A face *is* its plane** (`docs/decisions.md` §17.1). The stored corners
//!   are cached samples of it, so at fuse entry every corner is re-derived as
//!   the meet of the planes around it. A split face inherits its parent's plane
//!   bit for bit, so chaining fuses never degrades a plane, and a corner's
//!   rounding is always the rounding of one Cramer solve rather than a sum over
//!   the operation's history.
//! - **Two kinds of wobble, and only one of them refuses** (§17.2). The width of
//!   a re-derived corner's box is *representation wobble*: it is tagged on the
//!   corner as [`Cause::ConstructedPoint`](rcad_predicates::Cause::ConstructedPoint),
//!   it is never a reason to refuse, and it never goes in the
//!   [`ProvenanceLog`]. Only a predicate that could not decide goes through
//!   [`resolve_under_policy`](rcad_predicates::resolve_under_policy), and under
//!   [`Policy::certified`] that refuses.
//! - **Everything that matters to a face is a pair of planes** (§17.4). A face
//!   `F` on plane `p` has a boundary edge on `p ∩ Q` for the plane `Q` of the
//!   face across it, and a cut line on `p ∩ Q` for the plane `Q` of the face
//!   cutting it. So every corner of the arrangement is
//!   [`Plane::meet_of_three`], the anticlockwise order of two lines round a
//!   corner is a determinant of three stored normals, and no two-plane meet is
//!   needed anywhere — building a point and a direction on a line would round,
//!   which is §15.5's mistake by another road (§17.5).
//! - **One arrangement per face against all of its cut lines at once** (§17.4),
//!   never a sequence of pairwise splits, because a pairwise sequence computes a
//!   different partition for `fuse(A, B)` than for `fuse(B, A)` and §14.8's
//!   commutativity and associativity oracles would fail by construction.
//! - **Build in scratch, commit once** (§17.4). Nothing in this module touches
//!   the model, so any `Err` on the way out leaves it byte-identical.
//! - **No `HashMap` or `HashSet` in the working state** (§17.4). Hash order
//!   varies per process, that order would decide arena insertion order, and
//!   arena order is visible in every serialized byte.
//!
//! # The seam the rest of the fuse attaches to
//!
//! [`Scratch::split_both_groups`] is the whole of what is here. It hands back a
//! [`Scratch`], and the three things to build on it are:
//!
//! - **Selection.** [`Scratch::pieces`] is every face of both groups cut into
//!   the pieces the two groups' boundaries leave. Each [`Piece`] names the
//!   ancestor faces it was cut out of, carries their plane bit for bit, and says
//!   which way round the ancestors' solid used them. Which operand a piece was
//!   cut from, and so which group it belongs to, is [`Scratch::cut_from`] and
//!   never its ancestors, since one solid may be named twice. §17.4's selection
//!   table is a table plus one `match` on
//!   `Rule`, and classifying a piece is
//!   [`SolidView::classify_point`](crate::SolidView::classify_point) on a point
//!   inside it; the ON columns are
//!   [`Plane::agreement_with`](rcad_geom::Plane::agreement_with) between the two
//!   pieces' planes.
//! - **Reassembly.** A [`Piece`]'s wires are chains of [`FaceMeet`]s that close:
//!   each one ends where the next begins, the outer wire first and then one wire
//!   per hole, outer anticlockwise round the plane's normal and holes clockwise
//!   — the winding `Model::validate` asks for. Two pieces are neighbours when
//!   they share a [`FaceMeet`] walked once each way, which is where the
//!   manifold check of [`Error::ResultIsNotManifold`] belongs.
//! - **The commit.** A [`MeetPoint::Existing`] is an input vertex the result
//!   should keep the key of; a [`MeetPoint::OfThreeSurfaces`] is a new vertex, and
//!   the width of its box is what to tag it with, as
//!   [`Cause::ConstructedPoint`](rcad_predicates::Cause::ConstructedPoint) — not
//!   as anything a policy judges (§17.2). The commit is the first mutation of
//!   the model in the whole operation, and it must stay the only one.
//!
//! What is deliberately **not** here, so nobody goes looking: the `Rule` enum,
//! the selection table, the image maps history is derived from, and
//! `Model::fuse` itself.

use rcad_geom::{DynamicRange, Picker, Point, Surface};
use rcad_predicates::{
    Approximation, Decision, Level, Measures, Point as PredicatePoint, Policy, ProvenanceLog,
    Record, Strictness, resolve_under_policy,
};
use smallvec::SmallVec;

use crate::Model;
use crate::{EdgeKey, Error, Result, SolidKey, VertexKey};

mod arrange;
pub(crate) mod assemble;
mod bake;
mod cuts;
mod history;
mod select;

pub use history::Fused;
pub use select::Rule;

pub(crate) use arrange::name_of_surface_bits;

use arrange::{Piece, split_a_face};
use assemble::assemble;
use bake::{BakedSolid, bake};
use history::history_of;
use select::choose;

impl Model {
    /// Cut two groups of solids against each other and keep the pieces `rule`
    /// asks for.
    ///
    /// `A` is the union of `arguments` and `B` the union of `tools`.
    /// [`Rule::Union`] is `A ∪ B`, [`Rule::Intersection`] is `A ∩ B`, and
    /// [`Rule::Difference`] is `A \ B`. Either group may be empty, and the union
    /// of no solids is nothing: `fuse([A], [], Intersection)` is empty and
    /// `fuse([A], [], Difference)` is `A`.
    ///
    /// **An empty result is a success, not an error.** Two solids that do not
    /// touch have an empty intersection and the caller asked a well-formed
    /// question; the [`Fused`] that comes back simply names no solids (§17.4).
    ///
    /// The input solids are left exactly as they were, keys and all. Nothing is
    /// removed from the arena, so "deleted" in [`Fused`] means an input with no
    /// image rather than a key that has stopped resolving.
    ///
    /// # What `Ok` promises, and under which policy
    ///
    /// An answer this returns `Ok` for passes [`Model::validate`] **under
    /// `policy`, and under no other**. A fuse invents features neither input
    /// had — the sliver where two nearly-flush faces meet is as thin as the gap
    /// between the two planes, which has no lower bound short of zero — and a
    /// tolerant `policy` warns about a spread that thin where a certified one
    /// refuses it. So a tolerant fuse writes solids that a later certified
    /// caller is told are broken, and both readings are correct: "valid" is a
    /// property of a model *and a policy*, not of a model on its own.
    ///
    /// Nothing is corrupted when that happens and the two never disagree at the
    /// same strictness — a caller who tightens the policy gets a refusal, never
    /// a wrong answer. What a caller must not do is fuse tolerantly and then
    /// hand the result on as if it were certified.
    /// `a_model_only_the_policy_that_built_it_calls_valid.rs` measures it, and
    /// `docs/decisions.md` §68 says why the refusal is not moved into the fuse.
    ///
    /// # Atomic
    ///
    /// The whole answer is worked out in scratch and written in one go. Any
    /// `Err` leaves the model byte-identical (§17.4).
    ///
    /// # Errors
    ///
    /// [`Error::Undecidable`] when a decision the answer depends on could not be
    /// settled under `policy`, or when what the operation *did* have to
    /// approximate came to more than [`Policy::max_wobble`] allows.
    /// [`Error::ArgumentIntersectsItself`] when one of the input solids runs
    /// through itself. [`Error::ResultIsNotManifold`] when the answer would have
    /// an edge with other than two faces along it — two cubes meeting along one
    /// edge united, say. [`Error::Invalid`] when three of the surfaces round an
    /// input's corner place it and another surface round it is certified off
    /// that place by more than [`Model::validate`] lets a corner sit off its
    /// surface, which takes a corner with four or more surfaces round it. A
    /// corner with three is placed only where they cross, and one further from
    /// there than that room is placed by nothing: [`Error::Undecidable`], on
    /// "where three surfaces meet", and not this (§294.16). [`Error::Invalid`]
    /// also when a shell of the answer encloses no space, or when the answer's
    /// own shortest edge is too small to resolve beside its own coordinates —
    /// which is [`Invalid::DynamicRangeTooWide`] about a feature the inputs did
    /// not have and the fuse built (see `Assembly::check_dynamic_range`), or
    /// when cutting a face left a piece bounded by two straight sides, which
    /// enclose no area and so describe no face — [`Invalid::WireTooShort`], and
    /// §67 for why that is the input's fault and not the splitter's.
    ///
    /// [`Error::KeyFromAnotherModel`] when one of `arguments` or `tools` was
    /// made by a different model. It is asked of every key in both lists
    /// **before any other work**, which is the one refusal a fuse can reach
    /// without arithmetic and makes the "Atomic" promise above trivially true
    /// for it. Such a key answered `Ok` naming no solids at all until §84, and
    /// panicked between §84 and §86.
    ///
    /// [`Error::TheFuseWroteABrokenAnswer`] if the answer this built does not
    /// validate. That can be a gap here or an input that was already broken,
    /// because nothing here asks [`Model::validate`] of the inputs: in §294.2,
    /// six fuses of a bar whose corner was moved in its bytes came back as this,
    /// carrying exactly what `validate` says of the bar before any fuse. The
    /// variant's own doc has a pinned case of each. It used to be a panic — but
    /// the answer is now built in a throwaway model and swept there, so a caller
    /// who sees it has a model exactly as it was, and there is nothing corrupt
    /// for a crash to protect (`docs/decisions.md` §87, which amends §4.1 and
    /// §66.4).
    /// **The answer is swept in every build**, by twelve of [`Model::validate`]'s
    /// thirteen checks: the throwaway holds the answer and nothing else, which
    /// is what makes a whole-model sweep of it local to what this fuse wrote.
    /// The thirteenth, the model-wide coordinate range, is left out —
    /// `Assembly::check_dynamic_range` asks that one, in `assemble` and behind
    /// a guard, and
    /// asking it again over a model holding only the answer would undo the
    /// guard (§87.3). Before §66.3 the check was `debug_assertions`-gated *and*
    /// skipped whenever the model did not already validate, so a release fuse
    /// looked at nothing; §66.3 made it run always, and §87 made it a refusal.
    ///
    /// # Panics
    ///
    /// If the answer that swept clean does not survive being written a second
    /// time into the caller's model. **So the "Atomic" promise above holds up to
    /// that panic and no further** — it fires after the write, with the answer
    /// already in the model. It stays a panic because reaching it means the two
    /// writes disagreed, which is this crate corrupting its own arena (§17.4,
    /// §4.3).
    ///
    /// **An input broken in a way [`Model::validate`] rejects is not checked, in
    /// any build.** This paragraph once said such an input panicked, and then
    /// that it tripped an assert further in; nothing here has ever asked, and
    /// the one kind of break measured since does neither. A round bar whose top
    /// corner, where only the cap's plane and the wall's cylinder meet, was
    /// moved a millimetre off one of the two in the model's bytes and read back was
    /// fused under `Policy::certified` thirty times — three moves, each against
    /// ten partners and rules — and none of the thirty panicked. Each came back
    /// one of three ways: the answer the sound bar gives, where the partner cuts
    /// the moved corner away (twenty); [`Error::TheFuseWroteABrokenAnswer`],
    /// where the answer keeps it (six); or
    /// [`Error::CannotTellTheCellsOfAFaceApart`] or [`Error::Undecidable`],
    /// where a partner's face runs through where the corner should be (four).
    /// `a_broken_two_surface_corner_read_back_from_bytes.rs` pins all thirty
    /// (§294.2, §294.3), and Q419 pinned the same "no wrong `Ok`, no panic"
    /// result for a placed bar, a tolerant policy, a ball's moved pole, and a
    /// frustum's moved corner, in
    /// `a_turned_and_shifted_bar_blames_itself_for_a_corner_off_the_cap_with_the_distance_rounded`,
    /// `under_a_tolerant_policy_a_corner_moved_off_the_cap_in_a_cut_along_the_seam_touches_without_crossing`,
    /// `a_ball_whose_south_pole_is_moved_fuses_to_the_sound_balls_answer`, and
    /// `a_frustum_whose_top_corner_is_moved_gives_what_the_bar_gives`
    /// (§294.8). A box whose top corner, where three planes meet, was moved a
    /// millimetre the same way was fused against the same partners at four
    /// placements under both policies, and every fuse refused with
    /// [`Error::Undecidable`] on "where three surfaces meet", the partner cut
    /// the corner away or not, and left the model as it was
    /// (`a_broken_three_surface_corner_read_back_from_bytes.rs`, §294.16). Both
    /// kinds that paragraph once called untried are tried now. A pyramid apex,
    /// four planes round the corner, moved the same four ways against the
    /// same partners: all 352 fuses refuse the same `Undecidable` on "where
    /// three surfaces meet", no wrong `Ok`, no panic (§294.17). A corner where
    /// a cap, a round wall and a cut meet, one surface curved among three, got
    /// the same millimetre moves and refused the same way; moved onto the
    /// cut's other end it refuses `Error::Invalid(EdgeHasNoLength)` or, turned,
    /// `Undecidable`, still with no wrong `Ok` and no panic (§294.18).
    pub fn fuse(
        &mut self,
        arguments: &[SolidKey],
        tools: &[SolidKey],
        rule: Rule,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Fused> {
        for key in arguments.iter().chain(tools) {
            key.try_slot_in(self.id())?;
        }

        let mut mine = ProvenanceLog::new();
        let fused = self.fused_under(arguments, tools, rule, policy, &mut mine);
        mine.drain_into(log);
        fused
    }

    fn fused_under(
        &mut self,
        arguments: &[SolidKey],
        tools: &[SolidKey],
        rule: Rule,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Fused> {
        let scratch = Scratch::split_both_groups(self, arguments, tools, policy, log)?;
        let chosen = choose(self, &scratch, rule, policy, log)?;

        // The last question the answer itself can be asked, and the only one
        // neither input could have been asked instead: a fuse invents features
        // — the sliver where two nearly-flush faces meet — and a sliver has no
        // lower size short of zero. `assemble` asks it, once, and only when
        // this guard says the inputs fit; `Assembly::commit` deliberately
        // sweeps the throwaway with the twelve checks that are not this one,
        // so that the guard is what decides whether it is asked at all. Two of
        // the fuse's own tests are pairs whose inputs do not fit and whose
        // answer this fuse reaches exactly, and a sweep asking the question
        // again refuses both (§87.3).
        //
        // A tolerant policy's warning is held back until the wobble check
        // below has run: that check judges the decisions this fuse could not
        // prove, a range is not one, and before Q1614 the check ran ahead of
        // the range and never saw it.
        let inputs_reach = the_inputs_fit_in_f64(self, arguments, tools)?;
        let mut range_warning = ProvenanceLog::new();
        let mut assembly = assemble(
            self,
            &scratch,
            &chosen,
            inputs_reach,
            policy,
            log,
            &mut range_warning,
        )?;
        assembly.nest_the_shells(self, policy, log)?;

        // Exit criterion 7, and the one place it is easy to get wrong (§17.2).
        // What is judged is the wobble of the decisions this operation could not
        // *prove* — which is all a `ProvenanceLog` ever holds. Judging
        // `Model::worst_approximation()` here instead would be the shorter line
        // of code and would make a certified fuse refuse every input whose
        // corners are not exactly representable, which is every part that is not
        // axis-aligned on a lattice.
        //
        // Read at the working size the policy accepted each record at, which is
        // the size `resolve_under_policy` read it at. The model's
        // `Scale::nominal_size` is a second copy of that number and need not
        // agree with it. Read at a model a hundredth of the policy's size, a
        // volume comes out ten thousand times wider, and this used to refuse a
        // solid built only out of decisions the policy had accepted (§15.2,
        // §17.8).
        //
        // Every other strictness carries no working size, and `Certified` is
        // the only other there is, so the model's stands in. Its budget is
        // zero, so all this asks there is whether any record's wobble is above
        // zero. Any positive size gives the same answer to that, unless
        // dividing by it rounds a record's wobble down to zero, which takes a
        // wobble smaller than the smallest double.
        let working_size =
            if let Strictness::TolerantFallback { working_size } = policy.strictness() {
                working_size
            } else {
                self.scale().nominal_size()
            };
        if !policy.wobble_is_acceptable(log.worst_wobble(working_size)) {
            return Err(Error::Undecidable {
                records: core::mem::take(log),
            });
        }
        range_warning.drain_into(log);

        let inherited = worst_tag_of_the_inputs(self, arguments, tools);
        let written = assembly.commit(self, inherited, policy, log)?;
        Ok(history_of(self, &scratch, &assembly, &written))
    }
}

/// The inputs' largest coordinate, when the inputs' own coordinates and their
/// shortest edge fit in `f64` together, and `None` when they do not.
///
/// [`Assembly::check_dynamic_range`] measures the answer's smallest feature
/// against that coordinate as well as the answer's own, because the inputs
/// stay in the caller's model beside the answer.
///
/// **A fuse is answerable for what it builds, not for what it was handed.** A
/// frame placed at `2^40` with millimetre walls is already outside
/// [`DynamicRange::LARGEST_SUPPORTED_RATIO`] before anything is fused, and
/// `Model::validate` already says so; uniting it with a copy of itself is a
/// perfectly good answer and refusing it here would report somebody else's
/// dynamic range as this fuse's bug. It is taken over the input solids rather
/// than over the whole model so that it stays local to the entities this
/// operation touched (§4.3) — the same reason [`Assembly::commit`] sweeps a
/// throwaway model holding only the answer rather than the caller's whole one.
///
/// Chords only, where `Model::validate` also takes the radii of curved faces.
/// A ball, a pipe or a ring keeps its radius bit for bit in the answer (§17.1),
/// so were one of those the smallest thing about the inputs, measuring it here
/// would report the inputs as not fitting, which only ever makes the check
/// stand down. Leaving those radii out is therefore the stricter of the two,
/// not a hole. A cone's radius at a corner is not a number of its surface, and
/// the fuse makes corners of its own: the check measures the answer's, and
/// this leaves the inputs' out for the same reason as the others, so the
/// stricter way round again. A model whose radius is what puts it out of
/// range fails `Model::validate` going in, and its caller's next
/// `Model::validate` says so again on the way out; the sweep of the throwaway
/// answer does not, because that sweep leaves the range question to the
/// check.
///
/// # Errors
///
/// [`Error::ACrossingThisCannotBeOrderedAlong`] for an input edge whose run is a
/// kind this crate has never seen. **This one answers fit or not and cannot
/// say "skip me", which is why it refuses instead of guessing.** Read as an
/// open edge, a closed run of a kind nothing here knows contributes a chord of
/// nothing; that makes the ratio hopeless, which makes this `None`, which
/// stands the *answer's* range check down and leaves [`Assembly::commit`]'s
/// assert as the only thing between a bad answer and the model — a panic where
/// there was a refusal to be had. Read the other way it would understate the
/// smallest feature of the input. Neither reading is a measurement of anything,
/// and a fuse whose input carries a word this crate cannot read has no business
/// deciding whose fault the numbers are.
fn the_inputs_fit_in_f64(
    model: &Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
) -> Result<Option<f64>> {
    let mut largest = 0.0_f64;
    let mut shortest = f64::INFINITY;
    for &solid in arguments.iter().chain(tools) {
        for face in model.solid_at(solid.slot_in(model.id())).faces() {
            for wire in face.wires() {
                for corner in wire.points() {
                    largest = largest.max(corner.largest_coordinate_magnitude());
                }
                for coedge in wire.coedges() {
                    let edge = coedge.edge();
                    if !edge.is_closed()? {
                        shortest = shortest.min(edge.distance_between_its_ends());
                    }
                }
            }
        }
    }
    if !shortest.is_finite() || largest == 0.0 {
        return Ok(Some(largest));
    }
    Ok(DynamicRange::measure(largest, shortest)
        .is_ok_and(|range| range.is_within_supported_range())
        .then_some(largest))
}

/// The worst slop any entity of any input carries, if any of them carries one.
///
/// A fuse hands this on to everything it builds. A face cut by a face that was
/// only ever known to a millimetre is only known to a millimetre itself, and
/// §17.2 settles that a certified fuse carries an input's tag through rather
/// than refusing it: a tag on an imported vertex says how that vertex got into
/// the model, not what this operation did.
fn worst_tag_of_the_inputs(
    model: &Model,
    arguments: &[SolidKey],
    tools: &[SolidKey],
) -> Option<Approximation> {
    let mut worst: Option<Approximation> = None;
    let mut take = |tag: Option<Approximation>| {
        if let Some(tag) = tag
            && worst.is_none_or(|so_far| so_far.magnitude() < tag.magnitude())
        {
            worst = Some(tag);
        }
    };
    for &solid in arguments.iter().chain(tools) {
        for face in model.solid_at(solid.slot_in(model.id())).faces() {
            take(face.approximation());
            for wire in model.face_at(face.key().slot_in(model.id())).wires() {
                for coedge in wire.coedges() {
                    take(coedge.edge().approximation());
                    take(coedge.start().approximation());
                }
            }
        }
    }
    worst
}

/// Everything a fuse has worked out before it touches the model.
///
/// The scratch half of §17.4's "build the whole result in scratch and commit
/// once": a half-applied boolean is the corrupt solid §4.1 would rather crash
/// than emit, and here we can have neither, so nothing below this line is
/// allowed to mutate a [`Model`].
#[derive(Debug)]
pub(crate) struct Scratch {
    /// The argument solids, in world coordinates, with their cut lines.
    pub(crate) arguments: Vec<BakedSolid>,
    /// The tool solids, the same way.
    pub(crate) tools: Vec<BakedSolid>,
    /// Every face of both groups, cut into pieces. Each piece names the input
    /// faces it came from.
    pub(crate) pieces: Vec<Piece>,
    /// Which operand each of [`Scratch::pieces`] was cut from, by its place in
    /// [`Scratch::arguments`] and [`Scratch::tools`] laid end to end.
    ///
    /// **Not derivable from the faces a piece names.** One solid may be named
    /// twice — `fuse([a], [a], Union)` — and then both copies' pieces name the
    /// same faces. Looking the operand up by face credited every piece to the
    /// later copy, every piece then lay on the earlier one and was dropped as
    /// the one that came second, and `a ∪ a` came back empty.
    pub(crate) cut_from: Vec<usize>,
    /// Every point some face's arrangement found to be more than one name,
    /// from the faces of both groups, kept piece or not, and every meet a
    /// tolerant policy took as an input corner a rounding from it (§417.7).
    pub(crate) names_of_one_point: Vec<NamesOfOnePoint>,
}

impl Scratch {
    /// Bake both groups into world coordinates, work out every cut line, and
    /// split every face of both groups against all of its cut lines at once.
    ///
    /// The pieces come out in a fixed order that depends on the two groups and
    /// nothing else — not on the order the cut lines were found in, and not on
    /// anything that varies between runs.
    ///
    /// # Errors
    ///
    /// [`Error::Undecidable`] when a decision the split depends on could not be
    /// settled under `policy`, [`Error::ArgumentIntersectsItself`] when one of
    /// the input solids crosses itself, and [`Error::Invalid`] when an input's
    /// corners and its planes disagree by more than the corners' own boxes.
    pub(crate) fn split_both_groups(
        model: &Model,
        arguments: &[SolidKey],
        tools: &[SolidKey],
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Self> {
        let mut solids = bake(model, arguments, policy, log)?;
        let group_splits_here = solids.len();
        solids.extend(bake(model, tools, policy, log)?);
        cuts::gather(&mut solids, policy, log)?;
        let tools = solids.split_off(group_splits_here);
        let arguments = solids;

        let mut pieces = Vec::new();
        let mut cut_from = Vec::new();
        let mut names_of_one_point = Vec::new();
        for (operand, solid) in arguments.iter().chain(&tools).enumerate() {
            for face in &solid.faces {
                let split = split_a_face(face, policy, log)?;
                pieces.extend(split.pieces);
                names_of_one_point.extend(split.names_of_one_point);
            }
            cut_from.resize(pieces.len(), operand);
        }
        let mut scratch = Self {
            arguments,
            tools,
            pieces,
            cut_from,
            names_of_one_point,
        };
        let taken_as_input_corners = assemble::fold_each_corner_one_plane_cuts_a_rounding_from_it(
            model, &scratch, policy, log,
        );
        scratch.names_of_one_point.extend(taken_as_input_corners);
        Ok(scratch)
    }
}

/// Where two faces meet: part of the curve carried by their two surfaces.
///
/// This is §15.7's intersection result type. The two ends are **not** exact and
/// never will be, because a meet divides — §15.7 said this type "fills in
/// nothing else", which would have someone wire the enclosure to zero, and
/// §17.5 corrects it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FaceMeet {
    /// The two surfaces whose meet carries this piece of curve. Never a point
    /// and a direction: building one would round (§17.5).
    pub(crate) carried_by: [Surface; 2],
    /// Where the piece starts and where it ends, in that order.
    pub(crate) ends: [MeetPoint; 2],
    /// Which side of `carried_by[1]` the piece's material is on.
    ///
    /// [`Decision::Neg`] or [`Decision::Pos`], and never anything else. It costs
    /// no arithmetic: the walk that made this side keeps the material on its
    /// left, and left of a crossing's forward direction is always the negative
    /// side of the partner — `(∇S × ∇T) × ∇S · ∇T = −‖∇S × ∇T‖²` by
    /// Cauchy–Schwarz, everywhere, with no case analysis (§36.6).
    ///
    /// It is what lets selection say whether a point is inside a **curved**
    /// piece without winding anything: a piece bounded by whole closed curves is
    /// the part of its surface on the material side of every one of them.
    pub(crate) material_side: Decision,
    /// The input edges this runs along, when it runs along any. Empty for a
    /// piece of a cut, which is boundary the result invents, and more than one
    /// when joining coplanar input faces made two edges into one.
    pub(crate) along: SmallVec<[EdgeKey; 1]>,
    /// How far the real meet may stray from this description. Planes meet in a
    /// straight line, so this is always zero for a flat pair; M3's curved
    /// marcher is its first real producer (§15.7).
    pub(crate) approximation: Option<Approximation>,
}

/// One end of a [`FaceMeet`].
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MeetPoint {
    /// A corner the input already had. Re-derived or moved all the same
    /// (§17.1, §31.3); the key is kept so history can name it.
    Existing {
        /// The input vertex this corner is.
        vertex: VertexKey,
        /// Where it is.
        point: PredicatePoint,
    },
    /// A new point, where three surfaces meet.
    OfThreeSurfaces {
        /// The three surfaces, in a fixed order.
        surfaces: [Surface; 3],
        /// Which of their meeting points this is. **Load-bearing**: three
        /// surfaces can meet at more than one point, so the three names do not
        /// pick a corner on their own and `Corners::of` would join two different
        /// corners into one (§31.3).
        which: u8,
        /// The box their meeting point is somewhere inside.
        point: PredicatePoint,
    },
    /// The one point on a closed crossing that nothing else cuts.
    ///
    /// A wire needs a coedge, an edge needs two ends, and a wire that closes on
    /// one edge needs both ends to be one vertex — so a closed crossing with no
    /// corner on it still needs one. Where it goes depends on the two surfaces
    /// and on nothing else, which is what keeps `fuse(A, B)` and `fuse(B, A)`
    /// agreeing (§31.3, §34.4).
    ///
    /// **Two surfaces can cross in two closed curves**, and then the two
    /// surfaces do not say which one this starts: a bar down a pointed cone's
    /// axis crosses the cone's surface in a circle either side of the apex.
    /// Named by the two alone, both circles got one corner and so one edge,
    /// with four faces along it. `picked_by` is what says which.
    WhereAClosedCurveStarts {
        /// The two surfaces whose crossing this starts, in the order the face
        /// asking carries them.
        carried_by: [Surface; 2],
        /// What picks this curve out of the others the two cross in, as
        /// `intersect` named it, and empty where they cross in one closed curve.
        /// The same from both faces along the curve, in both argument orders:
        /// `intersect` builds it from the two surfaces whichever comes first.
        picked_by: SmallVec<[Picker; 1]>,
        /// Where it is.
        point: PredicatePoint,
    },
    /// The second place a closed crossing is cut, across the circle from where
    /// it starts, when where it starts is a ball's pole.
    ///
    /// A closed edge starting at a pole has no direction anyone can read, so
    /// the circle is cut in two there and each half runs between two corners
    /// (§194). Named apart from where the curve starts by the two surfaces and
    /// what picks the curve out of the others they cross in, which is what
    /// keeps both faces along it and both argument orders agreeing.
    HalfWayRoundAClosedCurve {
        /// The two surfaces whose crossing this is, in the order the face
        /// asking carries them.
        carried_by: [Surface; 2],
        /// What picks this curve out of the others the two cross in, as
        /// [`MeetPoint::WhereAClosedCurveStarts`] carries it.
        picked_by: SmallVec<[Picker; 1]>,
        /// Where it is.
        point: PredicatePoint,
    },
    /// A pole of a closed surface, where it pinches to a point.
    ///
    /// The one end of a boundary that is not where curves meet. A ball's skin
    /// is trimmed by nothing at all, so the answer has to cut it open along a
    /// seam of its own, and a seam runs pole to pole (§38.1).
    APoleOfAClosedSurface {
        /// The surface it is a pole of.
        surface: Surface,
        /// Which of the two: `true` for the one the surface's second coordinate
        /// climbs towards.
        at_the_top: bool,
        /// Where it is.
        point: PredicatePoint,
    },
}

impl MeetPoint {
    /// Where this end is.
    pub(crate) fn point(&self) -> PredicatePoint {
        match self {
            MeetPoint::Existing { point, .. }
            | MeetPoint::OfThreeSurfaces { point, .. }
            | MeetPoint::WhereAClosedCurveStarts { point, .. }
            | MeetPoint::HalfWayRoundAClosedCurve { point, .. }
            | MeetPoint::APoleOfAClosedSurface { point, .. } => *point,
        }
    }
}

/// One point that a face's arrangement found to be more than one name.
///
/// The arrangement folds a meet of three surfaces into a corner of the face,
/// or into another meet, when the two are one point within rounding, and
/// hands the survivor on as the end of every side there. A face that has
/// neither that corner nor the fold keeps the meet as a node of its own and
/// names it by its three surfaces, so the same point came into the answer as
/// two corners under two names, a few ulps apart, and an edge was left with
/// one face along it (§284.7). This carries the fold to `assemble`, which
/// makes every name here one corner before any piece is walked.
///
/// **Not a [`MeetPoint`] field.** The names travel beside the pieces, from
/// every node of every face whether or not a kept piece ends there, because
/// the face that folded may keep no piece at the point while the face that
/// did not fold keeps one.
#[derive(Clone, Debug)]
pub(crate) struct NamesOfOnePoint {
    /// The end the arrangement hands on for the point.
    pub(crate) kept: MeetPoint,
    /// The meets folded into it: their three surfaces, in
    /// `meet_of_three_surfaces`'s own fixed order, and which of their meeting
    /// points each one was, as [`MeetPoint::OfThreeSurfaces`] carries them.
    pub(crate) folded: Vec<([Surface; 3], u8)>,
    /// How far a tolerant policy moved the folded meets to put them at `kept`,
    /// when they were proved apart from it and taken as one point all the same
    /// (`docs/decisions.md` §417.7). `None` for a fold the arrangement made,
    /// where the two were one point within rounding.
    pub(crate) moved: Option<Approximation>,
}

/// Give up on a decision nothing can bound, recording that it was asked.
///
/// The enclosure is infinite because that is the truth: there is no width here
/// to compare against a budget, so no policy can accept it and none is asked.
fn gave_up(
    predicate: &'static str,
    inputs: &[PredicatePoint],
    measures: Measures,
    log: &mut ProvenanceLog,
) -> Error {
    log.push(Record::new(
        predicate,
        inputs,
        Level::Interval,
        f64::INFINITY,
        measures,
    ));
    Error::Undecidable {
        records: core::mem::take(log),
    }
}

/// The middle of a point's box: the one `f64` triple that stands for it.
///
/// A box that has collapsed onto one triple gives that triple back, so an
/// exactly representable corner is stored exactly as it was solved.
pub(crate) fn middle_of(point: PredicatePoint) -> Point {
    let box_of_it = point.enclosure();
    let axis =
        |axis: usize| box_of_it[axis].lo() + (box_of_it[axis].hi() - box_of_it[axis].lo()) / 2.0;
    Point::new(axis(0), axis(1), axis(2))
}

/// Which things have turned out to be one thing.
pub(crate) struct Sameness {
    leader: Vec<usize>,
    joined: bool,
}

impl Sameness {
    pub(crate) fn of(count: usize) -> Self {
        Self {
            leader: (0..count).collect(),
            joined: false,
        }
    }

    pub(crate) fn leader(&self, mut of: usize) -> usize {
        while self.leader[of] != of {
            of = self.leader[of];
        }
        of
    }

    pub(crate) fn join(&mut self, one: usize, other: usize) {
        let (one, other) = (self.leader(one), self.leader(other));
        if one == other {
            return;
        }
        self.leader[one.max(other)] = one.min(other);
        self.joined = true;
    }

    pub(crate) fn joined_anything(&self) -> bool {
        self.joined
    }
}

/// The same turn seen from the other side.
fn the_other_way(decision: Decision) -> Decision {
    match decision {
        Decision::Pos => Decision::Neg,
        Decision::Neg => Decision::Pos,
        Decision::Zero | Decision::Undecidable { .. } => decision,
        other => other,
    }
}

/// A box that has collapsed onto one `f64` triple *is* that triple.
///
/// The box is a certified enclosure, so a box of no width proves the true meet
/// is exactly the number in it, and saying so is a statement of what was proved
/// rather than a rounding.
///
/// It matters more than it looks. Level 3 of the cascade cannot run on a
/// constructed point, so a lattice-aligned model — where every meet lands
/// exactly on an `f64` — would otherwise come back undecided on decisions it can
/// prove, and §17.7's control family, whose expected refusal rate is zero, would
/// refuse everything.
fn as_exact_as_it_is(point: PredicatePoint) -> PredicatePoint {
    let box_of_it = point.enclosure();
    if box_of_it.iter().all(|side| side.lo() == side.hi()) {
        return PredicatePoint::from([box_of_it[0].lo(), box_of_it[1].lo(), box_of_it[2].lo()]);
    }
    point
}

/// Turn a predicate result the fuse cannot carry on without into an answer or a
/// refusal.
///
/// The name is the seam's own: everything in this module routes an undecided
/// predicate through here, so there is one place to look for what a refusal
/// means and one place a policy is read.
///
/// The same shape as `build.rs`'s `turn_of`, and for the same reason: an
/// undecided predicate is a *result* and not an error, but an operation left
/// with nothing to return is [`Error::Undecidable`] (§4.2). Under
/// [`Policy::certified`] every unproved decision lands there, and that refusal
/// rate is the number M2.5 exists to measure (§17.7), so nothing here softens
/// it.
///
/// Representation wobble must never come through here: it is a different
/// quantity, it is not a decision anybody failed to prove, and conflating the
/// two is the specific failure §17.2 guards against.
fn must_decide(
    decision: Decision,
    predicate: &'static str,
    inputs: &[PredicatePoint],
    measures: Measures,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Decision> {
    match decision {
        Decision::Neg | Decision::Zero | Decision::Pos => Ok(decision),
        Decision::Undecidable { enclosure } => {
            let record = Record::new(predicate, inputs, Level::Interval, enclosure, measures);
            match resolve_under_policy(record, policy, log) {
                settled @ (Decision::Neg | Decision::Zero | Decision::Pos) => Ok(settled),
                Decision::Undecidable { .. } => Err(Error::Undecidable {
                    records: core::mem::take(log),
                }),
                other => panic!("a policy came back with a decision nothing here knows: {other:?}"),
            }
        }
        // `Decision` grows a variant at every milestone. Whatever the new one
        // means, guessing is worse than stopping.
        other => panic!("a predicate came back with a decision nothing here knows: {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
    use rcad_predicates::{Policy, ProvenanceLog};

    use super::{MeetPoint, Piece, Rule, Scratch};
    use crate::{Error, Model, SolidKey};

    /// Two cubes of side two, the second moved by `shift` and then turned by
    /// `turn` about the origin.
    fn two_cubes(shift: Vector, turn: Option<Isometry>) -> (Model, SolidKey, SolidKey) {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let one = model.add_cube(2.0, &policy, &mut log).expect("a cube");
        let other = model
            .add_cube(2.0, &policy, &mut log)
            .expect("another cube");
        model
            .place_solid(
                other,
                &Isometry::translation(shift).expect("a finite shift"),
            )
            .expect("this model made the key");
        if let Some(turn) = turn {
            model
                .place_solid(one, &turn)
                .expect("this model made the key");
            model
                .place_solid(other, &turn)
                .expect("this model made the key");
        }
        (model, one, other)
    }

    /// Twice the area a piece covers, `Σ (p × q) · n` round each of its wires.
    ///
    /// Every corner here is exactly representable, so every product and every
    /// sum is exact in `f64` and the pieces of a face add up to that face with
    /// no tolerance anywhere.
    fn twice_the_area_of(piece: &Piece) -> f64 {
        let normal = piece.plane().normal().components();
        piece
            .wires
            .iter()
            .flatten()
            .map(|side| {
                let at = |end: &MeetPoint| {
                    end.point()
                        .exact_coordinates()
                        .expect("a lattice-aligned corner is an f64")
                };
                let (p, q) = (at(&side.ends[0]), at(&side.ends[1]));
                (p[1] * q[2] - p[2] * q[1]) * normal[0]
                    + (p[2] * q[0] - p[0] * q[2]) * normal[1]
                    + (p[0] * q[1] - p[1] * q[0]) * normal[2]
            })
            .sum()
    }

    #[test]
    fn cutting_two_cubes_that_overlap_leaves_the_pieces_of_each_face_adding_up_to_it() {
        let (model, one, other) = two_cubes(Vector::new(1.0, 1.0, 1.0), None);
        let mut log = ProvenanceLog::new();

        let scratch =
            Scratch::split_both_groups(&model, &[one], &[other], &Policy::certified(), &mut log)
                .expect("two lattice-aligned cubes leave nothing undecided");

        assert!(log.is_empty(), "nothing here had to be approximated");
        assert_eq!(scratch.arguments.len(), 1);
        assert_eq!(scratch.tools.len(), 1);

        let faces: Vec<_> = scratch
            .arguments
            .iter()
            .chain(&scratch.tools)
            .flat_map(|solid| &solid.faces)
            .collect();
        assert_eq!(faces.len(), 12, "two cubes have twelve faces between them");

        for face in faces {
            let pieces: Vec<&Piece> = scratch
                .pieces
                .iter()
                .filter(|piece| piece.ancestors == face.keys)
                .collect();
            assert!(!pieces.is_empty(), "every face comes back as something");
            let whole = 2.0 * 2.0 * 2.0;
            let summed: f64 = pieces.iter().map(|piece| twice_the_area_of(piece)).sum();
            assert_eq!(summed, whole, "the pieces of a face of side two");
        }

        let cut_in_four = scratch
            .pieces
            .iter()
            .filter(|piece| piece.plane().normal() == Vector::new(0.0, 0.0, 1.0))
            .count();
        assert_eq!(
            cut_in_four, 8,
            "each cube's top is cut into four by the two walls of the other that cross it — the \
             upper one's top included, which the upper cube is nowhere near"
        );
    }

    /// The same two cubes, turned so that nothing about them is exactly
    /// representable any more. Every corner is then a box rather than a number,
    /// which §17.1 says is the ordinary case and not a corner case.
    #[test]
    fn cutting_two_turned_cubes_that_overlap_keeps_every_piece_closed() {
        let axis = Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 2.0, 3.0))
            .expect("a direction with length");
        let turn = Isometry::rotation(&axis, 0.7).expect("a rotation");
        let (model, one, other) = two_cubes(Vector::new(1.0, 1.0, 1.0), Some(turn));
        let mut log = ProvenanceLog::new();

        let scratch =
            Scratch::split_both_groups(&model, &[one], &[other], &Policy::certified(), &mut log)
                .expect("a turned pair is decided by proof alone");

        assert!(
            log.is_empty(),
            "a turned pair still needs nothing approximated: every corner is a box, and a box is \
             not slop anybody failed to prove"
        );
        for piece in &scratch.pieces {
            for wire in &piece.wires {
                assert!(wire.len() >= 3);
                for step in 0..wire.len() {
                    assert_eq!(
                        wire[step].ends[1].point(),
                        wire[(step + 1) % wire.len()].ends[0].point(),
                        "a side of a piece has to start where the one before it ended"
                    );
                }
            }
        }
    }

    /// Building in scratch and committing once (§17.4): the cutting half touches
    /// nothing, so a model handed to it comes back the same size it went in.
    #[test]
    fn cutting_leaves_the_model_alone() {
        let (model, one, other) = two_cubes(Vector::new(1.0, 1.0, 1.0), None);
        let before = postcard::to_allocvec(&model).expect("a model writes out");

        let _ = Scratch::split_both_groups(
            &model,
            &[one],
            &[other],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("two cubes cut each other");

        assert_eq!(
            before,
            postcard::to_allocvec(&model).expect("a model writes out"),
            "cutting builds in scratch and writes nothing back"
        );
    }
    /// The generic case: a cube turned about an awkward axis cutting a straight
    /// one. Nothing is coplanar, nothing is parallel, no two lines are the same
    /// line, and every corner of every piece is a box rather than a number.
    ///
    /// A certified policy decides all of it and the log stays empty, which is
    /// §17.7's refusal rate looking at its first sample: the corners being
    /// inexact is representation wobble, and representation wobble refuses
    /// nothing.
    #[test]
    fn a_turned_cube_cutting_a_straight_one_never_has_to_ask_the_policy() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let straight = model.add_cube(2.0, &policy, &mut log).expect("a cube");
        let turned = model
            .add_cube(2.0, &policy, &mut log)
            .expect("another cube");
        let axis = Axis::new(Point::new(1.0, 1.0, 1.0), Vector::new(1.0, 2.0, 3.0))
            .expect("a direction with length");
        model
            .place_solid(turned, &Isometry::rotation(&axis, 0.7).expect("a rotation"))
            .expect("this model made the key");

        let scratch = Scratch::split_both_groups(&model, &[straight], &[turned], &policy, &mut log)
            .expect("a turned cube cutting a straight one is decided by proof alone");

        assert!(log.is_empty(), "nothing here had to be approximated");
        assert!(
            scratch.pieces.len() > 12,
            "two cubes that cut each other come apart into more than they went in as, got {}",
            scratch.pieces.len()
        );
        for piece in &scratch.pieces {
            for wire in &piece.wires {
                assert!(wire.len() >= 3);
                for step in 0..wire.len() {
                    assert_eq!(
                        wire[step].ends[1].point(),
                        wire[(step + 1) % wire.len()].ends[0].point()
                    );
                }
            }
        }
    }

    /// Roadmap §7.4's ninth planar catalogue entry, and the one that cannot sit
    /// with the other eight in `tests/degenerate_fuses.rs`: nothing in the
    /// public API builds a solid that runs through itself. `ProfileBuilder`
    /// refuses an outline that crosses itself and an extrusion of a sound
    /// profile never crosses itself either, so the input has to be wired
    /// together by hand from two overlapping cubes.
    ///
    /// The refusal is the right answer and not a limitation. §17.4 settles that
    /// the fuse notices this for free — it is already working out where every
    /// pair of faces meets — and that `Model::validate` deliberately does not
    /// look for it, since every face against every other is quadratic in the
    /// face count.
    #[test]
    fn a_solid_that_runs_through_itself_cannot_be_fused() {
        let (mut model, one, other) = two_cubes(Vector::new(1.0, 1.0, 1.0), None);
        let crossed = both_solids_as_one(&mut model, one, other);

        let refused = model.fuse(
            &[crossed],
            &[],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );

        assert!(
            matches!(refused, Err(Error::ArgumentIntersectsItself)),
            "expected a refusal, got {refused:?}"
        );
    }

    /// Two solids wired into one, sharing their shells.
    fn both_solids_as_one(model: &mut Model, one: SolidKey, other: SolidKey) -> SolidKey {
        let mut shells = model.data().solids[one.slot_in(model.id())].shells.clone();
        let theirs = model.data().solids[other.slot_in(model.id())]
            .shells
            .clone();
        shells.extend(theirs);
        let slot = model.add_solid(shells);
        model.key_for_solid(slot)
    }
}
