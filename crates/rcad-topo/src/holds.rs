//! Whether one face of a solid holds a point.
//!
//! # One question, two callers, one place
//!
//! Ray casting asks it of the point where a ray met a face's surface, and the
//! sweep asks it of the corners of a curved face. Two spellings of it is how
//! they come to disagree (`docs/decisions.md` §23.8), so there is one.
//!
//! **The fuse is not on that list yet and should be**, which is a thing to know
//! rather than a thing to gloss. `fuse/select.rs` still winds a point round a
//! piece's outline itself, and it cannot call this one as written: its outlines
//! are made of [`PredicatePoint`]s the arrangement solved for, and the walk
//! below takes corners the model holds. §35 files that as still open.
//!
//! # What a flat face does, and why it does not carry over
//!
//! A flat face is decided by walking its outline: a half-line runs from the
//! point towards the first corner, the sides crossing it are counted, and the
//! count is non-zero exactly on the material. Every test is an
//! [`orient3d`](rcad_predicates::orient3d_tolerantly) against a fourth point off
//! the plane, so nothing is projected into the plane's own two coordinates
//! (§15.5). It works because a flat face is bounded and the far end of the
//! half-line is off it — the count has somewhere known to start from.
//!
//! **A curved face has no such place**, and that is the whole of why row 10 of
//! §31.2 is the milestone's schedule risk. A ball's skin is closed: there is no
//! far end, and "outline and holes" is an order the face stores rather than a
//! fact about the geometry (§31.3). §35 records the three routes that were
//! weighed and what chose between them.
//!
//! # What is decided here instead
//!
//! A face is trimmed by the surfaces of the faces around it. Where each of
//! those surfaces meets this one in **whole closed curves that are all edges of
//! this face**, the face is one of the two pieces that curve cuts its surface
//! into, and which piece is a sign — [`side_of_surface`], the predicate that
//! already exists. So the answer is a handful of exact signs with no point
//! constructed, no curve followed, no ordering along one, and no meet of three
//! surfaces.
//!
//! Where a face is trimmed by something else — an *arc* of a crossing, or one
//! loop of a crossing that has two — this refuses by name rather than guessing.
//! Both refusals are gaps and §35 names the consumer that calls each due.

use rcad_geom::{EdgeRun, Plane, Point, Surface, SurfaceMeet, intersect};
use rcad_predicates::{
    Decision, Level, Measures, ORIENT3D, Point as PredicatePoint, Policy, ProvenanceLog, Record,
};

use crate::classify::decide;
use crate::validate::{name_of_surface, side_of_surface};
use crate::view::{CoedgeView, EdgeView, FaceView, WireView};
use crate::{Error, FaceKey, Invalid, Result, SolidView};

/// Where a point sits relative to one face of a solid.
///
/// The point is taken as being on the face's surface already. Nothing here asks
/// whether it is: a ray casting caller has just worked the point out *from* that
/// surface, and asking again would be a second opinion about a fact the caller
/// already holds.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnAFace {
    /// Inside the face's material, clear of its boundary.
    Held,
    /// Exactly on one of the face's boundary edges or corners.
    OnItsBoundary,
    /// Off the face.
    NotHeld,
}

impl<'m> SolidView<'m> {
    /// Whether one face of this solid holds `point`.
    ///
    /// `face` has to be a face of this solid, because what trims it is the
    /// surfaces of the faces around it and those are only reachable — already
    /// moved into place — from the solid they sit in.
    ///
    /// # Errors
    ///
    /// [`Error::NoSuchFaceInThisSolid`] when `face` names a face this solid does
    /// not use.
    ///
    /// [`Error::AFaceTrimmedByACurveThisCannotFollow`],
    /// [`Error::AFaceMissesPartOfWhereItsSurfacesCross`] and
    /// [`Error::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`] when the face is
    /// trimmed in a way this cannot follow. All three are about the shape of the
    /// face and not about the arithmetic, so no policy loosens them.
    ///
    /// [`Error::Undecidable`] when a sign could not be settled — which for a
    /// point that is a box is the honest answer and not a shortcoming (§2.5).
    ///
    /// Whatever [`intersect`] makes of the two surfaces either side of an edge.
    pub fn face_holds_the_point(
        self,
        face: FaceKey,
        point: PredicatePoint,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<OnAFace> {
        let view = self
            .faces()
            .find(|used| used.key() == face)
            .ok_or(Error::NoSuchFaceInThisSolid { face })?;
        self.what_this_face_holds(view, point, policy, log)
    }

    /// The same question, asked with the face already in hand.
    ///
    /// The ray caster walks `self.faces()` and holds a located [`FaceView`]
    /// already; looking the same one up again by key would walk every face of
    /// every shell and compose an [`Isometry`](rcad_geom::Isometry) per step, for
    /// every face of every ray. The public entry point above is the only place
    /// that pays for the lookup, and it pays once.
    pub(crate) fn what_this_face_holds(
        self,
        face: FaceView<'_>,
        point: PredicatePoint,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<OnAFace> {
        let Surface::Plane(plane) = face.surface() else {
            return what_a_curved_face_holds(self, face, &face.surface(), point, policy, log);
        };
        let aim = a_point_off_the_plane(face, &plane)
            .ok_or(Error::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn { face: face.key() })?;
        what_a_flat_face_holds(self, face, point, aim, policy, log)
    }
}

/// A point far enough off `plane` that the plane itself certifies it is off it.
///
/// The winding walk turns three points on a plane about a fourth off it, and the
/// sign it reads is only the in-plane one while that fourth point really is off
/// (§15.5). A unit normal added to a corner a long way from the origin rounds
/// straight back onto the plane, so the step grows until the plane's own
/// predicate says [`Decision::Pos`] about where it landed.
///
/// `None` when doubling sixty times never gets off the plane. The sentence that
/// used to stand here said that needs a normal that is not a real number, and
/// named the wrong end of the range: the step *starts* at the corner's own
/// magnitude, so a corner at `f64::MAX` on a plane whose normal is not along an
/// axis overflows on the very first try and every doubling after it, and the
/// answer is `None` with a perfectly good answer available at a *smaller* step.
/// A **gap**, and it is the flat second cause of
/// [`Error::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`] that §35.11 records
/// only the curved one of. Nothing a builder here makes reaches it — every
/// constructor refuses a model whose dynamic range is that wide — and
/// [`Model`](crate::Model) is `Deserialize`, so it is a refusal rather than a
/// wrong answer and §39 hands it on with the other three the audit found.
fn a_point_off_the_plane(face: FaceView<'_>, plane: &Plane) -> Option<Point> {
    let from = face
        .wires()
        .flat_map(WireView::points)
        .next()
        .unwrap_or_else(Point::origin);
    let mut step = from.largest_coordinate_magnitude().max(1.0);
    for _ in 0..60 {
        let target = from + plane.normal() * step;
        if plane.side_of_point(PredicatePoint::from(target.coordinates())) == Decision::Pos {
            return Some(target);
        }
        step *= 2.0;
    }
    None
}

/// Whether the material of a flat face holds `point`, which is taken as being on
/// that face's plane.
///
/// A half-line is run from `point` towards the first corner of the first wire
/// that has corners, and the sides crossing it are counted, `+1` for one way
/// round and `-1` for the other; a hole subtracts itself because it runs the
/// other way, and the count is non-zero exactly on the material. Which side of
/// that half-line's line each corner falls is read as a half-open range — a
/// corner exactly on the line counts as below it — and that is what leaves no
/// tie to break and no second try to make. The one case a crossing count cannot
/// see is the point sitting on a side, and that is looked for as we go.
///
/// [`FaceView::for_each_triangle`] cannot answer this. Its triangles overlap
/// wherever the face is not convex and its spokes are sides of nothing, so one
/// triangle holding the point says nothing about the face: the fan over a frame
/// covers the frame's own hole twice, once each way, and the two only cancel
/// once they are added up.
///
/// **A wire that is one whole closed curve has no polygon to walk**, and it is
/// counted rather than walked. A round bar's cap is one, and so is the round
/// hole through a block. See [`what_a_round_wire_encloses`].
fn what_a_flat_face_holds(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    point: PredicatePoint,
    target: Point,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<OnAFace> {
    let aim = face
        .wires()
        .filter(|wire| !wire.closes_on_one_edge())
        .flat_map(WireView::points)
        .next();

    let mut winding = 0i64;
    for (position, wire) in face.wires().enumerate() {
        let inwards = if position == 0 { 1 } else { -1 };
        if wire.closes_on_one_edge() {
            match what_a_round_wire_encloses(solid, face, wire, point)? {
                Decision::Neg => winding += inwards,
                Decision::Pos => (),
                Decision::Zero => return Ok(OnAFace::OnItsBoundary),
                // A side nobody could settle came back as a refusal a moment
                // ago, so what is left is a `Decision` variant a later milestone
                // added, and guessing what it means is worse than stopping.
                other => panic!("a side came back with a decision nothing here knows: {other:?}"),
            }
            continue;
        }
        let Some(aim) = aim else {
            // The only wires left are round ones, which the arm above took, so
            // an aim is missing only for a wire with no corners at all.
            continue;
        };
        match walk_one_outline(face, wire, point, aim, target, policy, log)? {
            Some(count) => winding += count,
            None => return Ok(OnAFace::OnItsBoundary),
        }
    }
    Ok(if winding == 0 {
        OnAFace::NotHeld
    } else {
        OnAFace::Held
    })
}

/// Which side of the surface a round wire is cut from `point` is on:
/// [`Decision::Neg`] inside the wire, [`Decision::Pos`] outside it,
/// [`Decision::Zero`] exactly on it.
///
/// **The inside of such a wire is the inside of the ball or pipe that cut it**,
/// and that is a fact rather than a convention: a closed curve cut on a plane by
/// a ball or a pipe bounds the part of the plane inside that ball or pipe, which
/// is the bounded one of the two. A plane cuts a plane in a straight line, which
/// is not closed, so a round wire whose neighbour is flat is a face this cannot
/// follow rather than a case with an answer.
///
/// # Errors
///
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] when the wire is not one
/// whole closed crossing of two surfaces.
fn what_a_round_wire_encloses(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    wire: WireView<'_>,
    point: PredicatePoint,
) -> Result<Decision> {
    let cannot = || Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() };
    let coedge = wire.coedges().next().ok_or_else(cannot)?;
    let edge = coedge.edge();
    match edge.is_closed() {
        Ok(true) => {}
        // The only site of the six that keeps its own name for this. A run kind
        // nothing here knows is not one whole closed crossing as far as this can
        // tell, which is the very thing this refusal is for, and its refusal
        // points at the *face* the caller has in its hand where
        // `ACrossingThisCannotBeOrderedAlong` names no entity at all.
        Ok(false) | Err(_) => return Err(cannot()),
    }
    let Some(partner) = the_face_across(solid, face, edge)? else {
        return Err(cannot());
    };
    let surface = partner.surface();
    if surface.as_plane().is_some() {
        return Err(cannot());
    }
    let side = side_of_surface(&surface, point);
    if let Decision::Undecidable { .. } = side {
        return Err(gave_up_on_a_side(&surface, point, side));
    }
    Ok(side)
}

/// One polygonal wire's contribution to the crossing count, or `None` when
/// `point` sits on one of its sides.
fn walk_one_outline(
    face: FaceView<'_>,
    wire: WireView<'_>,
    point: PredicatePoint,
    aim: Point,
    target: Point,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<i64>> {
    let corners: Vec<Point> = wire.points().collect();
    if corners.is_empty() {
        return Ok(Some(0));
    }
    for coedge in wire.coedges() {
        if coedge.edge().stored_run() != &EdgeRun::Straight {
            return Err(Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() });
        }
    }

    let sides: Vec<Decision> = corners
        .iter()
        .map(|&corner| decide(corner, aim, target, point, policy, log))
        .collect();
    let mut winding = 0i64;
    for (index, &from) in corners.iter().enumerate() {
        let next = (index + 1) % corners.len();
        let along = decide(from, corners[next], target, point, policy, log);
        match along {
            Decision::Zero => match lies_between(from, corners[next], point) {
                Ok(true) => return Ok(None),
                Ok(false) => (),
                Err(overhang) => return Err(gave_up_ordering_along_a_side(point, overhang)),
            },
            Decision::Pos | Decision::Neg => (),
            other => return Err(gave_up_turning_a_corner(point, other)),
        }
        match crossing(sides[index], sides[next], along) {
            Some(count) => winding += count,
            None => {
                return Err(gave_up_turning_a_corner(
                    point,
                    whichever_of_them_was_not_read(sides[index], sides[next]),
                ));
            }
        }
    }
    Ok(Some(winding))
}

/// Whether the material of a curved face holds `point`, which is taken as being
/// on that face's surface.
///
/// **The face is the piece of its surface on one side of everything that trims
/// it**, and that is only true under a condition this checks rather than
/// assumes: each surface that trims the face must meet this one in whole closed
/// curves, every one of which is an edge of this face. Then that surface cuts
/// this one into exactly two pieces, the face is one of them, and which one is
/// the sign every corner of the face agrees on.
///
/// A face with nothing trimming it is the whole surface and holds every point on
/// it — a ball's skin, whose one wire is a seam with the same face on both
/// sides.
///
/// # Errors
///
/// The three refusals [`SolidView::face_holds_the_point`] lists, and
/// whatever [`intersect`] makes of a pair of surfaces.
fn what_a_curved_face_holds(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    surface: &Surface,
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<OnAFace> {
    let mut on_its_boundary = false;
    for trimming in what_trims_this_face(solid, face)? {
        if !every_piece_of_that_crossing_is_an_edge(
            surface,
            &trimming.surface,
            trimming.corners_on_it.len(),
            policy,
            log,
        )? {
            return Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face: face.key() });
        }
        let inwards = the_side_this_face_lies_on(face, &trimming)?
            .ok_or(Error::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn { face: face.key() })?;
        let side = side_of_surface(&trimming.surface, point);
        match side {
            Decision::Zero => on_its_boundary = true,
            settled if settled == inwards => (),
            Decision::Pos | Decision::Neg => return Ok(OnAFace::NotHeld),
            other => return Err(gave_up_on_a_side(&trimming.surface, point, other)),
        }
    }
    Ok(if on_its_boundary {
        OnAFace::OnItsBoundary
    } else {
        OnAFace::Held
    })
}

/// The surfaces that trim `face`, each with how many edges of this face run
/// along it.
///
/// A **seam** trims nothing and is left out: both its sides are this same face,
/// so it is where the surface's own two coordinates were cut open and not where
/// the face stops.
///
/// # Errors
///
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] when an edge of the face is
/// not a whole closed curve. An edge with two different ends is a piece of a
/// crossing rather than the whole of it, and which piece is a question about
/// where a curve runs rather than about a sign.
pub(crate) fn what_trims_this_face(
    solid: SolidView<'_>,
    face: FaceView<'_>,
) -> Result<Vec<Trimming>> {
    let mut trimming: Vec<Trimming> = Vec::new();
    for wire in face.wires() {
        for coedge in wire.coedges() {
            let edge = coedge.edge();
            let Some(partner) = the_face_across(solid, face, edge)? else {
                continue;
            };
            if edge.start().key() != edge.end().key() {
                return Err(Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() });
            }
            let surface = partner.surface();
            let corner = edge.start().key();
            match trimming.iter_mut().find(|known| known.surface == surface) {
                Some(known) => known.corners_on_it.push(corner),
                None => trimming.push(Trimming {
                    surface,
                    corners_on_it: vec![corner],
                }),
            }
        }
    }
    Ok(trimming)
}

/// One surface trimming a face: which surface, how many edges of the face run
/// along it, and which corners of the face those edges hold.
///
/// **The corners are the load-bearing part, and it is topology and not
/// arithmetic.** A corner of one of those edges is on that surface because it is
/// on that edge, which no rounding can take away. Asking a predicate instead
/// works on a solid stored where it sits and fails on one that has been placed:
/// the corner is carried there by an [`Isometry`](rcad_geom::Isometry) that
/// rounds, so it lands a hair off the surface it is on and the sign that comes
/// back is whichever way the rounding went (§19.1).
pub(crate) struct Trimming {
    /// The surface, already moved into place.
    pub surface: Surface,
    /// The corner each of those edges starts at, which are on it by
    /// construction. One per edge, so how many there are is how many edges run
    /// along this surface.
    pub corners_on_it: Vec<crate::VertexKey>,
}

/// The face on the other side of `edge`, already moved into place, or `None`
/// when the edge is a **seam** — this same face on both sides, which trims
/// nothing.
///
/// **A key would not do**, and that is [`Model`](crate::Model)'s whole shape: a
/// face read out of the arena by key carries no location, and the surface read
/// off it would be the stored one rather than the one the solid is using
/// (§3.2). So the face is found among the ones this solid is already handing
/// out, which are located.
///
/// # Errors
///
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] when the face across the edge
/// is not a face of this solid, which a closed solid never has.
pub(crate) fn the_face_across<'m>(
    solid: SolidView<'m>,
    face: FaceView<'m>,
    edge: EdgeView<'m>,
) -> Result<Option<FaceView<'m>>> {
    let Some(other) = edge.neighbouring_faces().find(|&key| key != face.key()) else {
        return Ok(None);
    };
    solid
        .faces()
        .find(|used| used.key() == other)
        .map(Some)
        .ok_or(Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() })
}

/// Whether every curve the two surfaces cross in is one of this face's `edges`.
///
/// [`intersect`] is the one authority on how many curves a pair of surfaces
/// meet in (§24.3), so this counts rather than works it out again. A face
/// holding one of two loops — one of the two disks a pipe leaves in a ball's
/// skin — is exactly the case where reading the face off a sign would name both
/// disks at once.
fn every_piece_of_that_crossing_is_an_edge(
    surface: &Surface,
    partner: &Surface,
    edges: usize,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    match intersect(surface, partner, policy, log)? {
        SurfaceMeet::Curves(runs) => Ok(runs.len() == edges),
        // Nothing, a touch and one surface used twice all leave a face with an
        // edge along a curve the pair does not have. `SurfaceMeet` grows a
        // variant at every milestone, and whatever a new one means, a face
        // trimmed by it is one this has not been taught.
        SurfaceMeet::Nothing
        | SurfaceMeet::TouchAtOnePoint { .. }
        | SurfaceMeet::SameSurface
        | _ => Ok(false),
    }
}

/// Which side of `partner` this face lies on, read off its own corners.
///
/// Every corner is either on `partner` — the corners of the very edge that runs
/// along it are — or off it on the side the face lies on. So one corner off it
/// answers, and every other corner off it has to agree.
///
/// **This is the only place the model says which side**, and that is worth
/// saying plainly. The other place it could come from is which way round the
/// wire runs, and a closed edge has no direction rule to read it from (§30.8
/// item 10, still open at §31.3). §35 says what lane 4 has to give it.
///
/// # Errors
///
/// [`Invalid::AFaceStraddlesSomethingTrimmingIt`] when two corners are off
/// `partner` on opposite sides. A crossing that goes right round cuts a surface
/// into exactly two pieces, so a face bounded by the whole of one and standing
/// on both sides of it is a model whose topology and geometry disagree — which
/// arrives from a file and not from a builder (§16.11).
///
/// `Ok(None)` when no corner is off `partner` at all. That is not a broken model
/// — a disk cut out of a ball has its one corner on the pipe that cut it — it is
/// a face this cannot read the side of, and the caller says so.
///
/// [`Error::Undecidable`] when a corner's side could not be settled.
pub(crate) fn the_side_this_face_lies_on(
    face: FaceView<'_>,
    trimming: &Trimming,
) -> Result<Option<Decision>> {
    let partner = &trimming.surface;
    let mut found: Option<Decision> = None;
    for corner in face
        .wires()
        .flat_map(WireView::coedges)
        .map(CoedgeView::start)
    {
        if trimming.corners_on_it.contains(&corner.key()) {
            continue;
        }
        let at = PredicatePoint::from(corner.point().coordinates());
        let side = side_of_surface(partner, at);
        match side {
            Decision::Zero => (),
            Decision::Pos | Decision::Neg => match found {
                None => found = Some(side),
                Some(already) if already == side => (),
                Some(_) => return Err(Invalid::AFaceStraddlesSomethingTrimmingIt.into()),
            },
            other => return Err(gave_up_on_a_side(partner, at, other)),
        }
    }
    Ok(found)
}

/// Whether `point`, already shown to be on the line through `from` and `to`, is
/// on the piece between them. `Err` when its box reaches past an end, carrying
/// how far past.
///
/// The comparison runs along whichever axis the side reaches furthest, which is
/// `docs/decisions.md` §17.4's rule for ordering points along a line and needs
/// no predicate of its own.
///
/// **The overhang is handed back rather than thrown away** because it is the
/// only measurement of this refusal there is. Nothing was decided badly here —
/// every `orient3d` certified — so the caller has no predicate's enclosure to
/// report, and §64.3 is what happened when it reported one anyway.
fn lies_between(from: Point, to: Point, point: PredicatePoint) -> Result<bool, f64> {
    let (from, to) = (from.coordinates(), to.coordinates());
    let axis = (0..3)
        .max_by(|&one, &other| {
            (to[one] - from[one])
                .abs()
                .total_cmp(&(to[other] - from[other]).abs())
        })
        .expect("three axes");
    let (low, high) = (from[axis].min(to[axis]), from[axis].max(to[axis]));
    let span = point.enclosure()[axis];
    if span.lo() >= low && span.hi() <= high {
        Ok(true)
    } else if span.hi() < low || span.lo() > high {
        Ok(false)
    } else {
        Err((low - span.lo()).max(span.hi() - high).max(0.0))
    }
}

/// `1` or `-1` when the side crosses the half-line, signed by which way it
/// crosses, and `0` when it does not.
///
/// `from_side` and `to_side` say which side of the half-line's line each end of
/// the side falls on, and `along` says which side of it the point falls on —
/// which is what tells a crossing ahead of the point from one behind it.
fn crossing(from_side: Decision, to_side: Decision, along: Decision) -> Option<i64> {
    Some(
        match (above_the_line(from_side)?, above_the_line(to_side)?, along) {
            (false, true, Decision::Pos) => 1,
            (true, false, Decision::Neg) => -1,
            _ => 0,
        },
    )
}

/// Whether a corner is above the half-line's line, counting a corner exactly on
/// it as below.
///
/// Half-open on purpose. It is what gives every side two ends on definite sides
/// and leaves the crossing count with no tie to break, however many corners the
/// line happens to run through.
fn above_the_line(side: Decision) -> Option<bool> {
    match side {
        Decision::Pos => Some(true),
        Decision::Neg | Decision::Zero => Some(false),
        Decision::Undecidable { .. } => None,
        other => panic!("orient3d came back with a decision nothing here knows: {other:?}"),
    }
}

/// Which of a side's two ends is the one [`crossing`] could not read.
///
/// The refusal has to name that one and not its neighbour. Both are in hand and
/// only one of them was measured: a certified sign has no enclosure, so a record
/// made about it would have to invent the number a reader of the refusal came
/// for. Naming the neighbour is what §64 records, and it panicked rather than
/// lying, which is the only reason it was ever found.
///
/// [`crossing`] gives up only through [`above_the_line`], which reads every
/// certified sign, so one of the two really is undecided and the assert below
/// has nothing to fall back to.
fn whichever_of_them_was_not_read(from_side: Decision, to_side: Decision) -> Decision {
    if let Decision::Undecidable { .. } = from_side {
        return from_side;
    }
    assert!(
        matches!(to_side, Decision::Undecidable { .. }),
        "the crossing count was refused with both of its ends decided: {from_side:?} and {to_side:?}"
    );
    to_side
}

/// What to hand back when the side of a surface could not be settled.
///
/// **The name and the units are the sweep's**, not a second set: `check_curved_face`
/// records the same predicate about the same kind of point, and a log that
/// called it one thing when the sweep asked and another when a ray did would be
/// §23.8's two spellings arriving in the provenance rather than in the code.
///
/// **The caller's log is left alone**, and the one record travels with the
/// refusal in a fresh one. Whoever asked may well carry on — a ray that grazes
/// one face tries the next target — and emptying their log on the way past would
/// take the records of every decision that *was* settled with it.
fn gave_up_on_a_side(surface: &Surface, point: PredicatePoint, decision: Decision) -> Error {
    gave_up(
        name_of_surface(surface),
        point,
        Level::Exact,
        decision,
        Measures::Area,
    )
}

/// The same, for an `orient3d` in the outline walk.
///
/// The predicate has already put its own record in the caller's log on the way
/// past (`orient3d_tolerantly` does that for every level it could not settle),
/// so what this adds is which *question* the walk was asking when it stopped.
fn gave_up_turning_a_corner(point: PredicatePoint, decision: Decision) -> Error {
    gave_up(ORIENT3D, point, Level::Exact, decision, Measures::Volume)
}

/// The name the ordering of a point along a side goes by in provenance records.
const BETWEEN_TWO_CORNERS: &str = "between_two_corners";

/// What to hand back when the point is on a side's line and its box reaches
/// `overhang` past an end of it.
///
/// **Not [`gave_up_turning_a_corner`]**, and the difference is what is being
/// refused rather than how it is worded. No predicate gave up here: every
/// `orient3d` certified, and `along` came back [`Decision::Zero`] because the
/// point really is on the line through the two corners. What could not be
/// settled is where it sits *along* that line, which §17.4 answers by comparing
/// a box against the ends on one axis and asks no predicate at all.
///
/// So the record names the ordering, and its enclosure is the overhang — **a
/// length**, which is what that comparison measures. Reaching for `orient3d`
/// and [`Measures::Volume`] here would file a refusal under a predicate that
/// decided, with a width in units nothing here worked in, and §15.3 is why a
/// number whose units cannot be recovered is worse than no number.
fn gave_up_ordering_along_a_side(point: PredicatePoint, overhang: f64) -> Error {
    gave_up(
        BETWEEN_TWO_CORNERS,
        point,
        Level::Interval,
        Decision::Undecidable {
            enclosure: overhang,
        },
        Measures::Length,
    )
}

fn gave_up(
    named: &'static str,
    point: PredicatePoint,
    level: Level,
    decision: Decision,
    measures: Measures,
) -> Error {
    let mut records = ProvenanceLog::new();
    records.push(Record::new(named, &[point], level, decision, measures));
    Error::Undecidable { records }
}

#[cfg(test)]
mod tests {
    use rcad_geom::{Axis, Cylinder, Isometry, Orientation, Scale, Vector};
    use smallvec::SmallVec;

    use super::*;
    use crate::model::{FaceUse, ShellUse};
    use crate::{Model, SolidKey};

    const OUTER: f64 = 3.0;
    const INNER: f64 = 1.0;
    const HEIGHT: f64 = 5.0;

    /// A washer: a bar with a round hole straight through it.
    ///
    /// **Nothing in this kernel builds one yet**, and that is why it is here.
    /// It is the shape §31.7's criterion 11 asks the fuse for — a block with a
    /// round hole through it is the same solid with a square outline — and it is
    /// the first face with a round *hole* in it, which is a case
    /// [`FaceView::for_each_triangle`] gets wrong and this file has to get
    /// right. Built by hand because the fuse cannot make one until lane 5.
    ///
    /// Four faces: the wall outside, the wall inside the hole used the other way
    /// round, and two flat caps, each an outline circle with a circle cut out of
    /// it.
    fn a_washer() -> (Model, SolidKey) {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let corner = |model: &mut Model, across: f64, up: f64| {
            model.add_vertex(rcad_geom::Point::new(across, 0.0, up))
        };
        let outer_bottom = corner(&mut model, OUTER, 0.0);
        let outer_top = corner(&mut model, OUTER, HEIGHT);
        let inner_bottom = corner(&mut model, INNER, 0.0);
        let inner_top = corner(&mut model, INNER, HEIGHT);

        let outer_seam = model.add_edge(outer_bottom, outer_top);
        let inner_seam = model.add_edge(inner_bottom, inner_top);
        let round_outer_bottom = model.add_closed_edge(outer_bottom);
        let round_outer_top = model.add_closed_edge(outer_top);
        let round_inner_bottom = model.add_closed_edge(inner_bottom);
        let round_inner_top = model.add_closed_edge(inner_top);

        // `turn` is `Reversed` for the hole, and that is the direction rule and
        // not a taste: the hole's wall faces the axis, so the stretch of pipe it
        // covers is on its left only when its two cap circles are walked the
        // other way round from the outer wall's (§36.24).
        let wall = |model: &mut Model, radius: f64, bottom, seam, top, turn: Orientation| {
            let round_it: SmallVec<[_; 4]> = SmallVec::from_buf([
                model.add_coedge(bottom, turn),
                model.add_coedge(seam, Orientation::Forward),
                model.add_coedge(top, turn.reversed()),
                model.add_coedge(seam, Orientation::Reversed),
            ]);
            let wire = model.add_wire(round_it);
            let pipe = Cylinder::new(Axis::z(), radius).expect("a pipe");
            model.add_face(Surface::Cylinder(pipe), SmallVec::from_buf([wire]))
        };
        let outside = wall(
            &mut model,
            OUTER,
            round_outer_bottom,
            outer_seam,
            round_outer_top,
            Orientation::Forward,
        );
        let hole = wall(
            &mut model,
            INNER,
            round_inner_bottom,
            inner_seam,
            round_inner_top,
            Orientation::Reversed,
        );

        let cap = |model: &mut Model, up: f64, outward: f64, outline, inner, turn| {
            let plane = Plane::from_point_and_normal(
                rcad_geom::Point::new(0.0, 0.0, up),
                Vector::new(0.0, 0.0, outward),
            )
            .expect("the z axis is a unit normal");
            let round_it = model.add_coedge(outline, turn);
            // The opposite turn from the outline, because a hole runs the other
            // way round a face than its outline does — and on a wire that is one
            // closed curve nothing but the direction rule says so, there being no
            // polygon to wind (§36.24).
            let round_the_hole = model.add_coedge(inner, turn.reversed());
            let outer = model.add_wire(smallvec::smallvec![round_it]);
            let cut_out = model.add_wire(smallvec::smallvec![round_the_hole]);
            model.add_face(
                Surface::Plane(plane),
                SmallVec::from_vec(vec![outer, cut_out]),
            )
        };
        let bottom = cap(
            &mut model,
            0.0,
            -1.0,
            round_outer_bottom,
            round_inner_bottom,
            Orientation::Reversed,
        );
        let top = cap(
            &mut model,
            HEIGHT,
            1.0,
            round_outer_top,
            round_inner_top,
            Orientation::Forward,
        );

        // The hole's wall faces inwards, towards the axis, because the material
        // is outside it; everything else faces out.
        let shell = model.add_shell(
            [
                (bottom, Orientation::Forward),
                (top, Orientation::Forward),
                (outside, Orientation::Forward),
                (hole, Orientation::Reversed),
            ]
            .into_iter()
            .map(|(face, orient)| FaceUse {
                face,
                loc: Isometry::identity(),
                orient,
            })
            .collect(),
        );
        let solid = model.add_solid(SmallVec::from_buf([ShellUse {
            shell,
            loc: Isometry::identity(),
            orient: Orientation::Forward,
        }]));
        let key = model.key_for_solid(solid);
        (model, key)
    }

    fn at(x: f64, y: f64, z: f64) -> PredicatePoint {
        PredicatePoint::from([x, y, z])
    }

    fn what_it_holds(
        model: &Model,
        solid: SolidKey,
        face: FaceKey,
        point: PredicatePoint,
    ) -> OnAFace {
        model
            .solid(solid)
            .face_holds_the_point(face, point, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a washer is a shape this can follow")
    }

    /// A flat face with a round hole in it holds the ring and not the hole.
    ///
    /// The one thing `for_each_triangle` cannot say. Its fan over the hole's
    /// single corner has no area, so the outline's own fan covers the hole and
    /// every point in it comes back as material.
    #[test]
    fn a_flat_face_with_a_round_hole_in_it_does_not_hold_the_hole() {
        let (model, washer) = a_washer();
        let cap = model
            .solid(washer)
            .faces()
            .find(|face| face.surface().as_plane().is_some())
            .expect("a washer has two caps")
            .key();

        assert_eq!(
            what_it_holds(&model, washer, cap, at(2.0, 0.0, 0.0)),
            OnAFace::Held,
            "between the two circles"
        );
        assert_eq!(
            what_it_holds(&model, washer, cap, at(0.5, 0.0, 0.0)),
            OnAFace::NotHeld,
            "inside the hole"
        );
        assert_eq!(
            what_it_holds(&model, washer, cap, at(4.0, 0.0, 0.0)),
            OnAFace::NotHeld,
            "outside the outline"
        );
        for on_a_circle in [INNER, OUTER] {
            assert_eq!(
                what_it_holds(&model, washer, cap, at(0.0, on_a_circle, 0.0)),
                OnAFace::OnItsBoundary,
                "on the circle of radius {on_a_circle}"
            );
        }
    }

    /// The hole's own wall is trimmed by the same two cap planes as the outer
    /// one, and holds the same stretch of its own cylinder.
    #[test]
    fn the_wall_of_a_round_hole_holds_the_stretch_between_the_caps() {
        let (model, washer) = a_washer();
        let hole = model
            .solid(washer)
            .faces()
            .find(|face| match face.surface() {
                Surface::Cylinder(pipe) => pipe.radius() == INNER,
                Surface::Plane(_) | Surface::Sphere(_) | _ => false,
            })
            .expect("a washer has a hole")
            .key();

        assert_eq!(
            what_it_holds(&model, washer, hole, at(0.0, INNER, HEIGHT / 2.0)),
            OnAFace::Held
        );
        assert_eq!(
            what_it_holds(&model, washer, hole, at(0.0, INNER, -1.0)),
            OnAFace::NotHeld
        );
        assert_eq!(
            what_it_holds(&model, washer, hole, at(0.0, INNER, 0.0)),
            OnAFace::OnItsBoundary
        );
    }

    /// A washer is a sound solid, and the sweep says so.
    ///
    /// Worth asserting rather than assuming: it is the first solid here with a
    /// hole in a flat face and a curved face facing inwards, and `validate`
    /// meshes every curved solid it is handed. §35 records what that turned up.
    #[test]
    fn a_washer_passes_the_full_sweep() {
        let (model, _) = a_washer();
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a washer is a sound solid");
    }

    /// A ball is one face with nothing trimming it, so it holds every point on
    /// its sphere — including both poles, which are its only two corners.
    #[test]
    fn a_balls_one_face_holds_every_point_on_its_sphere() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let ball = model
            .add_ball(OUTER, &Isometry::identity())
            .expect("a ball");
        let skin = model.solid(ball).faces().next().expect("one face").key();

        for point in [
            at(OUTER, 0.0, 0.0),
            at(0.0, 0.0, OUTER),
            at(0.0, 0.0, -OUTER),
            at(0.0, -OUTER, 0.0),
        ] {
            assert_eq!(what_it_holds(&model, ball, skin, point), OnAFace::Held);
        }
    }

    /// A face trimmed by *part* of a curve is refused by name.
    ///
    /// An edge with two different ends runs along a piece of a crossing and not
    /// the whole of one, so which side of that partner surface a point is on
    /// says nothing about whether the face holds it. Which piece a point is on is
    /// a question about where a curve runs, and §35 says why that is not this
    /// lane's to answer.
    ///
    /// **Two faces and one edge, and it is deliberately not a closed solid.**
    /// Nothing here validates it, because what is being asked is a question about
    /// one face and its neighbour, and the smallest thing that has one of those
    /// is two faces. The fuse is what will make the real shape, at lane 4.
    #[test]
    fn a_face_trimmed_by_part_of_a_curve_is_refused_by_name() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bottom = model.add_vertex(rcad_geom::Point::new(OUTER, 0.0, 0.0));
        let top = model.add_vertex(rcad_geom::Point::new(OUTER, 0.0, HEIGHT));
        let up_the_side = model.add_edge(bottom, top);

        let mine = model.add_coedge(up_the_side, Orientation::Forward);
        let round_it = model.add_wire(smallvec::smallvec![mine]);
        let pipe = Cylinder::new(Axis::z(), OUTER).expect("a pipe");
        let wall = model.add_face(Surface::Cylinder(pipe), SmallVec::from_buf([round_it]));

        let theirs = model.add_coedge(up_the_side, Orientation::Reversed);
        let round_that = model.add_wire(smallvec::smallvec![theirs]);
        let flat =
            Plane::from_point_and_normal(rcad_geom::Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("the y axis is a unit normal");
        let beside_it = model.add_face(Surface::Plane(flat), SmallVec::from_buf([round_that]));

        let shell = model.add_shell(
            [wall, beside_it]
                .into_iter()
                .map(|face| FaceUse {
                    face,
                    loc: Isometry::identity(),
                    orient: Orientation::Forward,
                })
                .collect(),
        );
        let solid = model.add_solid(SmallVec::from_buf([ShellUse {
            shell,
            loc: Isometry::identity(),
            orient: Orientation::Forward,
        }]));
        let key = model.key_for_solid(solid);

        let refusal = model
            .solid(key)
            .face_holds_the_point(
                crate::FaceKey::new(wall, model.id()),
                at(0.0, OUTER, HEIGHT / 2.0),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("a side that is a piece of a curve cannot be followed");
        assert!(
            matches!(refusal, Error::AFaceTrimmedByACurveThisCannotFollow { .. }),
            "got {refusal:?}"
        );
    }

    /// A face that runs along one of two loops is refused rather than named as
    /// both.
    ///
    /// One of the two disks a pipe leaves in a ball's skin. Its one boundary is a
    /// whole closed curve and its one corner is on the pipe, so both of the two
    /// things this reads — which side the face lies on, and whether the face has
    /// every loop — have to catch it. Built by hand: a ball whose face is trimmed
    /// by a pipe through it is lane 4's.
    #[test]
    fn a_face_along_one_of_two_loops_is_refused_rather_than_named_as_both() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let ball = model
            .add_ball(OUTER, &Isometry::identity())
            .expect("a ball");
        let skin = model.solid(ball).faces().next().expect("one face").key();

        // A pipe up the z axis crosses this ball in two circles, one either side
        // of the equator. One corner on the upper one, and one closed edge round
        // it, standing in for the boundary of the cap the pipe cuts out.
        let across = (OUTER * OUTER - INNER * INNER).sqrt();
        let on_the_circle = model.add_vertex(rcad_geom::Point::new(INNER, 0.0, across));
        let loop_edge = model.add_closed_edge(on_the_circle);
        let mine = model.add_coedge(loop_edge, Orientation::Forward);
        let theirs = model.add_coedge(loop_edge, Orientation::Reversed);
        let cut = model.add_wire(smallvec::smallvec![mine]);
        let round_it = model.add_wire(smallvec::smallvec![theirs]);
        let at_skin = skin.slot_in(model.id());
        model.data_mut().faces[at_skin].wires.push(cut);
        let pipe = Cylinder::new(Axis::z(), INNER).expect("a pipe");
        let wall = model.add_face(Surface::Cylinder(pipe), SmallVec::from_buf([round_it]));
        model
            .data_mut()
            .shells
            .values_mut()
            .next()
            .expect("a shell")
            .faces
            .push(FaceUse {
                face: wall,
                loc: Isometry::identity(),
                orient: Orientation::Forward,
            });

        let refusal = model
            .solid(ball)
            .face_holds_the_point(
                skin,
                at(0.0, 0.0, -OUTER),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("a face along one of two loops cannot be read off a sign");
        assert!(
            matches!(
                refusal,
                Error::AFaceMissesPartOfWhereItsSurfacesCross { .. }
            ),
            "got {refusal:?}"
        );
    }
}
