//! Whether one face of a solid holds a point.
//!
//! # One question, three callers, one place
//!
//! Ray casting asks it of the point where a ray met a face's surface, the fuse
//! asks it of a witness point inside a piece it is placing, and the sweep asks
//! it of a corner. Two spellings of it is how they come to disagree
//! (`docs/decisions.md` §23.8), so there is one, and this is it.
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
    Decision, Level, Measures, Point as PredicatePoint, Policy, ProvenanceLog, Record,
};

use crate::classify::decide;
use crate::validate::side_of_surface;
use crate::view::{EdgeView, FaceView, WireView};
use crate::{Error, FaceKey, Result, SolidView};

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
        match view.surface() {
            Surface::Plane(plane) => {
                let aim = a_point_off_the_plane(view, &plane)
                    .ok_or(Error::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn { face })?;
                what_a_flat_face_holds(self, view, point, aim, policy, log)
            }
            curved => what_a_curved_face_holds(self, view, &curved, point, policy, log),
        }
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
/// `None` when doubling sixty times never gets off the plane, which needs a
/// normal that is not a real number: a plane built by a constructor here has a
/// unit one, and sixty doublings of it clears any finite coordinate.
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
                other => return Err(gave_up_on_a_side(point, other)),
            }
            continue;
        }
        let Some(aim) = aim else {
            return Err(Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() });
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
    if edge.run() != &EdgeRun::RightRoundWhereItsFacesCross {
        return Err(cannot());
    }
    let Across::Face(partner) = the_face_across(solid, face, edge)? else {
        return Err(cannot());
    };
    let surface = partner.surface();
    if surface.as_plane().is_some() {
        return Err(cannot());
    }
    let side = side_of_surface(&surface, point);
    if let Decision::Undecidable { .. } = side {
        return Err(gave_up_on_a_side(point, side));
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
        if coedge.edge().run() != &EdgeRun::Straight {
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
                Some(true) => return Ok(None),
                Some(false) => (),
                None => return Err(gave_up_on_a_side(point, along)),
            },
            Decision::Pos | Decision::Neg => (),
            other => return Err(gave_up_on_a_side(point, other)),
        }
        match crossing(sides[index], sides[next], along) {
            Some(count) => winding += count,
            None => return Err(gave_up_on_a_side(point, sides[index])),
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
    for (partner, edges) in what_trims_this_face(solid, face)? {
        if !every_piece_of_that_crossing_is_an_edge(surface, &partner, edges, policy, log)? {
            return Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face: face.key() });
        }
        let inwards = the_side_this_face_lies_on(face, &partner)?;
        let side = side_of_surface(&partner, point);
        match side {
            Decision::Zero => on_its_boundary = true,
            Decision::Pos | Decision::Neg if side == inwards => (),
            Decision::Pos | Decision::Neg => return Ok(OnAFace::NotHeld),
            other => return Err(gave_up_on_a_side(point, other)),
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
fn what_trims_this_face(solid: SolidView<'_>, face: FaceView<'_>) -> Result<Vec<(Surface, usize)>> {
    let mut trimming: Vec<(Surface, usize)> = Vec::new();
    for wire in face.wires() {
        for coedge in wire.coedges() {
            let edge = coedge.edge();
            let Across::Face(partner) = the_face_across(solid, face, edge)? else {
                continue;
            };
            if edge.start().key() != edge.end().key() {
                return Err(Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() });
            }
            let surface = partner.surface();
            match trimming.iter_mut().find(|(known, _)| *known == surface) {
                Some((_, count)) => *count += 1,
                None => trimming.push((surface, 1)),
            }
        }
    }
    Ok(trimming)
}

/// What is on the other side of one edge of a face.
enum Across<'m> {
    /// This same face, so the edge is a seam and trims nothing.
    Seam,
    /// Another face of the same solid.
    Face(FaceView<'m>),
}

/// The face on the other side of `edge`, already moved into place.
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
fn the_face_across<'m>(
    solid: SolidView<'m>,
    face: FaceView<'m>,
    edge: EdgeView<'m>,
) -> Result<Across<'m>> {
    let Some(other) = edge.neighbouring_faces().find(|&key| key != face.key()) else {
        return Ok(Across::Seam);
    };
    solid
        .faces()
        .find(|used| used.key() == other)
        .map(Across::Face)
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
        _ => Ok(false),
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
/// [`Error::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`] when no corner is off
/// `partner`, or two corners are off it on opposite sides — which is a face that
/// straddles something trimming it, and then it is not one of the two pieces at
/// all.
///
/// [`Error::Undecidable`] when a corner's side could not be settled.
fn the_side_this_face_lies_on(face: FaceView<'_>, partner: &Surface) -> Result<Decision> {
    let mut found: Option<Decision> = None;
    for corner in face.wires().flat_map(WireView::points) {
        let at = PredicatePoint::from(corner.coordinates());
        let side = side_of_surface(partner, at);
        match side {
            Decision::Zero => (),
            Decision::Pos | Decision::Neg => match found {
                None => found = Some(side),
                Some(already) if already == side => (),
                Some(_) => {
                    return Err(Error::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn {
                        face: face.key(),
                    });
                }
            },
            other => return Err(gave_up_on_a_side(at, other)),
        }
    }
    found.ok_or(Error::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn { face: face.key() })
}

/// Whether `point`, already shown to be on the line through `from` and `to`, is
/// on the piece between them. `None` when its box straddles an end.
///
/// The comparison runs along whichever axis the side reaches furthest, which is
/// `docs/decisions.md` §17.4's rule for ordering points along a line and needs
/// no predicate of its own.
fn lies_between(from: Point, to: Point, point: PredicatePoint) -> Option<bool> {
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
        Some(true)
    } else if span.hi() < low || span.lo() > high {
        Some(false)
    } else {
        None
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

/// The name a side of a face's own surface goes by in a provenance record.
const WHICH_SIDE_OF_A_FACE: &str = "which side of a face a point is on";

/// What to hand back when a sign this needed could not be settled.
///
/// **The caller's log is left alone**, and the one record travels with the
/// refusal in a fresh one. Whoever asked may well carry on — a ray that grazes
/// one face tries the next target — and emptying their log on the way past would
/// take the records of every decision that *was* settled with it.
fn gave_up_on_a_side(point: PredicatePoint, decision: Decision) -> Error {
    let mut records = ProvenanceLog::new();
    records.push(Record::new(
        WHICH_SIDE_OF_A_FACE,
        &[point],
        Level::Exact,
        decision,
        Measures::Volume,
    ));
    Error::Undecidable { records }
}
