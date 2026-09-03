//! Whether two solids fill any of the same space.
//!
//! `docs/roadmap.md` §7.4 names two of its degenerate cases as the ones that
//! matter here: a solid **fully contained** in another, and two solids
//! **touching but not overlapping**. The first has to be refused, the second
//! must not be, and a box test calls them the same thing — the boxes overlap
//! either way. That is the whole difficulty, and it is why this is not
//! `Bounds::overlap` with a longer name.
//!
//! # A pass of its own, not a check inside the sweep
//!
//! [`Model::validate`] sweeps one solid at a time. Two solids sharing space is
//! a question about a *pair*, and putting it in the sweep would make every
//! validation quadratic in the number of solids for a question most callers do
//! not ask — an import of five hundred parts is a legitimate model whose parts
//! are meant to be laid out and cut, not a broken one. So it is
//! [`Model::check_no_two_solids_share_space`], called when the caller wants it.
//!
//! # What is proved, and what is only searched for
//!
//! Two answers here are proofs and one is not, and reading the third as a proof
//! is the mistake this module is most likely to cause.
//!
//! - [`SharedSpace::Apart`] is a proof. The boxes round the two solids miss, so
//!   no point is in both.
//! - [`SharedSpace::Shared`] is a proof. Either a corner of one solid is
//!   strictly inside the other — a corner is on its own solid's skin, and every
//!   neighbourhood of a skin point holds material, so material meets material —
//!   or two faces sit on one plane with their material on the same side of it
//!   and a place was found in the middle of both.
//! - [`SharedSpace::NothingFound`] is **not** a proof that they share nothing.
//!   It says the search above turned nothing up. Two bars crossing in a plus,
//!   with no corner of either inside the other and no shared plane, land here
//!   and really do share space —
//!   `two_bars_crossing_in_a_plus_share_space_and_this_pass_does_not_see_it`
//!   is that, run, rather than said. Closing it needs an edge of one solid
//!   tested against a face of the other, which is boolean-grade work and is not
//!   done here.
//!
//! Two identical balls also land in [`SharedSpace::NothingFound`]: the
//! coincident-face rule reads a plane off each face and a ball has none.
//! `two_balls_at_the_same_place_share_space_and_this_pass_does_not_see_it` is
//! that one run too.
//!
//! Both proofs take the two solids to be sound — faces wound so that the
//! outward normal points out of the material, which is what
//! [`Model::validate`] is for. Handed a solid whose skin points inwards, the
//! coincident-face rule reads its material on the wrong side of the plane.
//!
//! # Cost
//!
//! Over a model: one box test per pair of solids. Inside a pair whose boxes
//! meet: one point classification per corner of each solid against the other,
//! and one plane comparison per pair of faces. The plane comparison is the
//! quadratic one.
//!
//! The *fully contained* case on its own is cheaper than what runs: a solid
//! inside another and clear of its skin has **every** corner strictly inside, so
//! one corner would settle it. Every corner is asked because the same loop then
//! also catches an overlap that is not containment, and because a contained
//! solid that touches its host's skin can have corners on it.

use std::collections::HashSet;

use rcad_geom::{Plane, PlaneAgreement, Point};
use rcad_predicates::{Interval, Point as PredicatePoint, Policy, ProvenanceLog};

use crate::view::WireView;
use crate::{Containment, Error, FaceView, Model, OnAFace, Result, SolidView};

/// What a search for space two solids both fill turned up.
///
/// Only two of these are proofs; the module header says which and why.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SharedSpace {
    /// A place was found that is in the material of both solids.
    Shared,
    /// The boxes round the two solids miss, so nothing can be in both.
    Apart,
    /// Nothing was found, which is not the same as there being nothing to find.
    NothingFound,
    /// Something the search depends on could not be decided.
    Undecidable {
        /// How wide the widest enclosure that straddled zero was.
        ///
        /// Zero when nothing straddled and the geometry was still unreadable —
        /// two planes so far apart in size that no minor of theirs can be given
        /// a sign.
        enclosure: f64,
    },
}

impl Model {
    /// Refuse a model in which two solids fill any of the same space.
    ///
    /// A pass of its own. [`Model::validate`] sweeps one solid at a time and
    /// deliberately still does; the module header says why.
    ///
    /// # Errors
    ///
    /// [`Error::TwoSolidsShareSpace`] when a place was found that is in the
    /// material of both of two solids.
    ///
    /// [`Error::CannotTellWhetherTwoSolidsShareSpace`] when the search could not
    /// be finished under `policy`. Not being able to tell is an answer at the
    /// predicate level and a refusal here, which is the split `docs/decisions.md`
    /// §4.2 draws.
    pub fn check_no_two_solids_share_space(
        &self,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        let solids: Vec<SolidView<'_>> = self.solids().collect();
        for (position, one) in solids.iter().enumerate() {
            for other in &solids[position + 1..] {
                match one.shares_space_with(*other, policy, log) {
                    SharedSpace::Shared => {
                        return Err(Error::TwoSolidsShareSpace {
                            one: one.key(),
                            other: other.key(),
                        });
                    }
                    SharedSpace::Undecidable { enclosure } => {
                        return Err(Error::CannotTellWhetherTwoSolidsShareSpace {
                            one: one.key(),
                            other: other.key(),
                            enclosure,
                        });
                    }
                    SharedSpace::Apart | SharedSpace::NothingFound => (),
                }
            }
        }
        Ok(())
    }
}

impl<'m> SolidView<'m> {
    /// Whether this solid and `other` fill any of the same space.
    ///
    /// Read [`SharedSpace`] before acting on the answer: one of its four is a
    /// search that came up empty and not a proof of anything.
    ///
    /// # Errors
    ///
    /// Nothing here fails; not being able to tell is an answer, not a failure.
    #[must_use]
    pub fn shares_space_with(
        self,
        other: SolidView<'m>,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> SharedSpace {
        // No box means there was nothing of the solid to measure — no face with
        // any reach at all — and a solid that fills nothing shares nothing.
        // `Model::validate` refuses one, so this is what a model that never went
        // through the door gets rather than a case the kernel makes.
        let (Some(mine), Some(theirs)) = (self.box_around_it(), other.box_around_it()) else {
            return SharedSpace::Apart;
        };
        if boxes_miss(mine, theirs) {
            return SharedSpace::Apart;
        }

        let mut widest: Option<f64> = None;
        for [one, two] in [[self, other], [other, self]] {
            match a_corner_of_one_is_inside_the_other(one, two, policy, log) {
                Found::Yes => return SharedSpace::Shared,
                Found::No => (),
                Found::Undecided { enclosure } => widen(&mut widest, enclosure),
            }
        }
        match two_faces_keep_their_material_on_one_side(self, other, policy, log) {
            Found::Yes => return SharedSpace::Shared,
            Found::No => (),
            Found::Undecided { enclosure } => widen(&mut widest, enclosure),
        }

        match widest {
            Some(enclosure) => SharedSpace::Undecidable { enclosure },
            None => SharedSpace::NothingFound,
        }
    }
}

/// Whether the two boxes have no point in common.
///
/// A proof and not a hint, and plain `f64` is honest for the same reason
/// `Bounds::overlap` gives: every point of every face is inside the box
/// [`SolidView::box_around_it`] draws, so two boxes that miss hold no common
/// point at all.
fn boxes_miss(mine: (Point, Point), theirs: (Point, Point)) -> bool {
    let (my_low, my_high) = (mine.0.coordinates(), mine.1.coordinates());
    let (their_low, their_high) = (theirs.0.coordinates(), theirs.1.coordinates());
    (0..3).any(|axis| my_high[axis] < their_low[axis] || their_high[axis] < my_low[axis])
}

/// What one step of the search found.
///
/// Exhaustive on purpose: a search either found its witness, ran out of places
/// to look, or could not read one of them.
#[derive(Clone, Copy, Debug)]
enum Found {
    Yes,
    No,
    Undecided { enclosure: f64 },
}

/// Whether a corner of `one` is strictly inside `two`.
///
/// A corner sits on its own solid's skin, and every neighbourhood of a skin
/// point holds material of that solid. So a corner strictly inside the other
/// solid — which is an open region — puts material of both in one place. That
/// is the whole proof, and it is why a corner on the *boundary* of `two` proves
/// nothing.
fn a_corner_of_one_is_inside_the_other(
    one: SolidView<'_>,
    two: SolidView<'_>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Found {
    let mut widest: Option<f64> = None;
    for corner in corners_of(one) {
        match two.classify_point(corner, policy, log) {
            Containment::Inside => return Found::Yes,
            Containment::OnBoundary | Containment::Outside => (),
            Containment::Undecidable { enclosure } => widen(&mut widest, enclosure),
            // The question was never put rather than asked and left open, so it
            // is not something a looser policy reaches (§4.2). It leaves the
            // search empty-handed, which `SharedSpace::NothingFound` already
            // says it might be.
            Containment::CannotFollowAFacesTrimming { .. }
            | Containment::CannotFireARayAtAFacesSurface { .. } => (),
        }
    }
    match widest {
        Some(enclosure) => Found::Undecided { enclosure },
        None => Found::No,
    }
}

/// Every corner of the solid, once each.
///
/// A corner of a cube is reached six times walking faces, wires and coedges,
/// and classifying it six times would cost six times what it proves.
fn corners_of(solid: SolidView<'_>) -> Vec<Point> {
    let mut seen = HashSet::new();
    let mut corners = Vec::new();
    for face in solid.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let corner = coedge.start();
                if seen.insert(corner.key()) {
                    corners.push(corner.point());
                }
            }
        }
    }
    corners
}

/// Whether a face of each solid sits on one plane with both solids' material on
/// the same side of it, sharing more than an edge of that plane.
///
/// This is the case no corner test can see. Two copies of one cube have every
/// corner of each on the other's skin and nothing strictly inside, so the only
/// thing left that separates them from two cubes stacked face to face is which
/// way the two shared faces point out of their own material. Stacked, they point
/// away from each other and the cubes only touch; on top of each other, they
/// point the same way and the material is in the same place.
///
/// "Sharing more than an edge" is what [`a_place_both_faces_hold`] settles.
/// Without it, two faces on one plane whose outlines are side by side would be
/// read as an overlap.
fn two_faces_keep_their_material_on_one_side(
    one: SolidView<'_>,
    two: SolidView<'_>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Found {
    let mut widest: Option<f64> = None;
    for mine in one.faces() {
        let Some(my_plane) = mine.surface().as_plane().copied() else {
            continue;
        };
        for theirs in two.faces() {
            let Some(their_plane) = theirs.surface().as_plane().copied() else {
                continue;
            };
            match the_same_way_out(&my_plane, mine, &their_plane, theirs) {
                Sides::TheSame => (),
                Sides::Opposite | Sides::NotOnePlane => continue,
                Sides::CannotTell => {
                    widen(&mut widest, 0.0);
                    continue;
                }
            }
            match a_place_both_faces_hold(one, mine, two, theirs, policy, log) {
                Found::Yes => return Found::Yes,
                Found::No => (),
                Found::Undecided { enclosure } => widen(&mut widest, enclosure),
            }
        }
    }
    match widest {
        Some(enclosure) => Found::Undecided { enclosure },
        None => Found::No,
    }
}

/// Where two faces keep their material relative to one another.
///
/// Exhaustive on purpose: two faces are on one plane or they are not, and if
/// they are, their material is on one side of it or on both.
#[derive(Clone, Copy, Debug)]
enum Sides {
    TheSame,
    Opposite,
    NotOnePlane,
    CannotTell,
}

/// Whether two faces on one plane point out of their material the same way.
///
/// [`Plane::agreement_with`] answers about the two *stored* planes and
/// [`FaceView::orientation`] carries the turn the shell put on each, so the two
/// together are the outward directions. Reading either alone gets the answer
/// backwards for a face used reversed.
///
/// The four answers [`PlaneAgreement`] has today are all named. It is
/// `#[non_exhaustive]` and a fifth would arrive meaning something this does not
/// know, so anything else is taken as not being one plane — the answer that
/// proves nothing, which is the safe way for an unknown to fall.
fn the_same_way_out(
    my_plane: &Plane,
    mine: FaceView<'_>,
    their_plane: &Plane,
    theirs: FaceView<'_>,
) -> Sides {
    let agreement = my_plane.agreement_with(their_plane);
    if agreement == PlaneAgreement::CannotTell {
        return Sides::CannotTell;
    }
    let stored_planes_agree = if agreement == PlaneAgreement::SameWay {
        true
    } else if agreement == PlaneAgreement::OppositeWay {
        false
    } else {
        return Sides::NotOnePlane;
    };
    let faces_are_turned_alike =
        mine.orientation().is_reversed() == theirs.orientation().is_reversed();
    if stored_planes_agree == faces_are_turned_alike {
        Sides::TheSame
    } else {
        Sides::Opposite
    }
}

/// A place in the middle of both faces, if the corners of either lead to one.
///
/// The places tried are the corners of each face and the midpoints of every pair
/// of them. A corner is exact; a midpoint is a sum rounded outwards and then
/// halved, which is exact, so the box handed on really does hold the true
/// midpoint.
///
/// Corners alone are not enough and that is the point of the midpoints: two
/// copies of one cube share a face exactly, so every corner of one is a corner
/// of the other and sits on its boundary rather than in its middle. The
/// midpoint of the two ends of a diagonal is in the middle of both.
fn a_place_both_faces_hold(
    one: SolidView<'_>,
    mine: FaceView<'_>,
    two: SolidView<'_>,
    theirs: FaceView<'_>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Found {
    let mut undecided = false;
    for place in places_to_try_on(mine)
        .into_iter()
        .chain(places_to_try_on(theirs))
    {
        let mut held_by_both = true;
        for (solid, face) in [(one, mine), (two, theirs)] {
            match does_this_face_hold_it(solid, face, place, policy, log) {
                Found::Yes => (),
                Found::No => held_by_both = false,
                Found::Undecided { .. } => {
                    undecided = true;
                    held_by_both = false;
                }
            }
        }
        if held_by_both {
            return Found::Yes;
        }
    }
    if undecided {
        // A face's outline is walked with exact corners and one constructed
        // point, so what could not be settled is the point's own box and its
        // width is what the caller of `face_holds_the_point` already put in the
        // log.
        Found::Undecided { enclosure: 0.0 }
    } else {
        Found::No
    }
}

/// The corners of a face and the midpoint of every pair of them.
///
/// A face has a handful of corners, so the pairs are a handful squared. The
/// solid-level loops are the ones that grow.
fn places_to_try_on(face: FaceView<'_>) -> Vec<PredicatePoint> {
    let corners: Vec<Point> = face.wires().flat_map(WireView::points).collect();
    let mut places: Vec<PredicatePoint> = corners
        .iter()
        .map(|corner| PredicatePoint::from(corner.coordinates()))
        .collect();
    for (position, corner) in corners.iter().enumerate() {
        for other in &corners[position + 1..] {
            places.push(midpoint_of(*corner, *other));
        }
    }
    places
}

/// The point halfway between two corners, as a box that holds it.
///
/// The sum is rounded outwards by the interval arithmetic and halving is exact
/// in `f64`, so the true midpoint is inside the box that comes back. Both
/// corners are on the face's plane, so the true midpoint is too, which is what
/// [`SolidView::face_holds_the_point`] takes as given about what it is handed.
fn midpoint_of(one: Point, other: Point) -> PredicatePoint {
    let mine = one.coordinates();
    let theirs = other.coordinates();
    let half = Interval::exactly(0.5);
    PredicatePoint::Constructed(std::array::from_fn(|axis| {
        (Interval::exactly(mine[axis]) + Interval::exactly(theirs[axis])) * half
    }))
}

/// Whether one face of one solid holds a place in its middle.
///
/// A place on the face's boundary is [`Found::No`] here on purpose: the proof
/// needs material either side of the plane at that place, and a boundary point
/// of the face has some of the plane around it that the face does not cover.
fn does_this_face_hold_it(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    place: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Found {
    match solid.face_holds_the_point(face.key(), place, policy, log) {
        Ok(OnAFace::Held) => Found::Yes,
        Ok(OnAFace::OnItsBoundary) | Ok(OnAFace::NotHeld) => Found::No,
        Err(refusal) => {
            if let Error::Undecidable { mut records } = refusal {
                records.drain_into(log);
                return Found::Undecided { enclosure: 0.0 };
            }
            // The face is trimmed in a way nothing here can follow. That is a
            // fact about the shape and no policy loosens it, so it is one fewer
            // place to look and not something left open.
            Found::No
        }
    }
}

/// Keep the widest enclosure that straddled, and remember that one did.
fn widen(widest: &mut Option<f64>, enclosure: f64) {
    *widest = Some(widest.unwrap_or(0.0).max(enclosure));
}
