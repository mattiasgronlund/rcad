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
//!
//! **One family of arcs is followed now** (§167): a wedge of a ball, whose skin
//! and whose two half-discs are each bounded by half of a circle. Those faces are
//! still the part of their surface on one side of everything that trims them —
//! or, on the sphere, on one side of *any* of it — but only once that has been
//! shown of the face in hand, and the side is read off the way the wire runs
//! rather than off a corner, because every corner sits on every cut.

use rcad_geom::{
    EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Surface, SurfaceMeet,
    WHERE_A_PLANE_AND_A_PIPE_CROSS, WHERE_A_PLANE_AND_A_TORUS_CROSS, intersect,
    which_two_surfaces_a_run_is_between,
};
use rcad_predicates::expansion::{expansion_sign, expansion_sum, product_is_exact, two_product};
use rcad_predicates::{
    Decision, Filtered, Interval, Level, Measures, ORIENT3D, Point as PredicatePoint, Policy,
    Predicate, ProvenanceLog, Record, orient2d,
};

use crate::classify::decide;
use crate::validate::{
    a_closed_crossing_is_walked_the_way_it_is_stored, gradient_of, name_of_surface, side_of_surface,
};
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
        if a_side_of_it_is_part_of_a_crossing(self, face)? {
            return what_a_face_cut_by_part_of_a_crossing_holds(self, face, point);
        }
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
/// hole through a block — and so is a circle walked as two arcs from pole to
/// pole ([`is_one_circle_cut_in_two`]). See [`what_a_round_wire_encloses`].
fn what_a_flat_face_holds(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    point: PredicatePoint,
    target: Point,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<OnAFace> {
    let mut round = Vec::new();
    for wire in face.wires() {
        round.push(wire.closes_on_one_edge() || is_one_circle_cut_in_two(solid, face, wire)?);
    }
    let aim = face
        .wires()
        .zip(&round)
        .filter(|(_, is_round)| !**is_round)
        .flat_map(|(wire, _)| wire.points())
        .next();

    let mut winding = 0i64;
    for ((position, wire), is_round) in face.wires().enumerate().zip(round) {
        let inwards = if position == 0 { 1 } else { -1 };
        if is_round {
            match what_a_round_wire_encloses(solid, face, wire, point, policy, log)? {
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
        match walk_one_outline(solid, face, wire, point, aim, target, policy, log)? {
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
/// **A ring leaves two regions on a plane through its axis**, the disks of the
/// two meridian circles, so the ring's sign alone holds the other circle's disk
/// as surely as this one's (§232.2). What tells them apart is the flat
/// half-space [`intersect`] names each circle by, having proved it clears both:
/// each disk is exactly the part of the tube on its own circle's side, and the
/// wire's corner says which side that is.
///
/// **A plane square to the axis cuts two circles round it**, and neither
/// circle's disk is the tube's inside there: the tube's inside is the band
/// between them. The pipe of the ring's own radius [`intersect`] names each
/// circle by says which circle the wire is and, with the ring's sign, which
/// side of it a point is on ([`what_a_circle_round_the_axis_encloses`],
/// `docs/decisions.md` §250.9).
///
/// # Errors
///
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] when the wire is not one
/// whole closed crossing of two surfaces, or when its neighbour is a ring and
/// [`intersect`] names no one circle through the wire's corner: a plane that
/// rests on the tube and touches it in one circle has no picker to name it by.
///
/// [`Error::Undecidable`] when the half-space naming a meridian circle, or the
/// pipe naming a circle round the axis, cannot say which side of it `point` is
/// on.
///
/// Whatever [`intersect`] makes of the face's plane and a ring beside it.
fn what_a_round_wire_encloses(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    wire: WireView<'_>,
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Decision> {
    let cannot = || Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() };
    let coedge = wire.coedges().next().ok_or_else(cannot)?;
    let edge = coedge.edge();
    match edge.is_closed() {
        Ok(true) => {}
        Ok(false) if is_one_circle_cut_in_two(solid, face, wire)? => {}
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
    if let Decision::Undecidable { enclosure } = side {
        return Err(gave_up_on_a_side(&surface, point, enclosure));
    }
    if !matches!(surface, Surface::Torus(_)) {
        return Ok(side);
    }
    let corner = wire.points().next().ok_or_else(cannot)?;
    let Some(this_circle) = the_picker_naming_the_circle_through(
        &face.surface(),
        &surface,
        PredicatePoint::from(corner.coordinates()),
        policy,
        log,
    )?
    else {
        return Err(cannot());
    };
    match this_circle {
        Picker::InFrontOf(_) => {
            if side == Decision::Pos {
                return Ok(side);
            }
            // The tube holds the point, so it is in one of the two disks or on
            // one of the two circles; the half-space says which.
            Ok(if the_meridian_half_space_keeps(&this_circle, point)? {
                side
            } else {
                Decision::Pos
            })
        }
        Picker::OutsideAPipe(_) => {
            what_a_circle_round_the_axis_encloses(&this_circle, true, side, point)
        }
        Picker::InsideAPipe(_) => {
            what_a_circle_round_the_axis_encloses(&this_circle, false, side, point)
        }
        Picker::OutsideABall(_) | _ => Err(cannot()),
    }
}

/// Which side of a circle round a ring's axis `point` is on, for a circle on a
/// plane square to that axis: [`Decision::Neg`] nearer the axis than the
/// circle, [`Decision::Pos`] further, [`Decision::Zero`] on it. `side` is the
/// ring's own sign at `point`, already settled.
///
/// **Such a plane cuts the ring in two circles round the axis**, at `R ± s`
/// from it, where `R` is the ring's radius and `s` is strictly positive, and
/// [`intersect`] names the outer by the outside of the pipe of radius `R` and
/// the inner by its inside. On the plane the ring holds the band between the
/// two (`Neg`), has the two circles on its skin (`Zero`), and leaves the rest
/// out (`Pos`). So the band is inside the outer circle and outside the inner,
/// and the pipe says which circle a point the ring does not hold is on, or which
/// side of both it lies:
///
/// | circle | ring `Neg` | ring `Zero` or `Pos`, kept by the pipe | not kept |
/// |---|---|---|---|
/// | outer, `OutsideAPipe` | `Neg` | the ring's sign | `Neg`: nearer the axis than `R` |
/// | inner, `InsideAPipe` | `Pos` | the ring's sign turned round | `Pos`: further from the axis than `R` |
///
/// A point on the pipe itself is kept, as a picker keeps what is on it, and on
/// the plane such a point is in the band, where the pipe is not asked. Off the
/// plane the answer is whatever the two signs give, as everywhere in this file:
/// [`SolidView::face_holds_the_point`] takes its point as on the face.
///
/// `outer` says which of the two circles `this_circle`, the side of the pipe
/// naming it, picks.
///
/// # Errors
///
/// [`Error::Undecidable`], naming the pipe, when it cannot say.
fn what_a_circle_round_the_axis_encloses(
    this_circle: &Picker,
    outer: bool,
    side: Decision,
    point: PredicatePoint,
) -> Result<Decision> {
    if side == Decision::Neg {
        return Ok(if outer { Decision::Neg } else { Decision::Pos });
    }
    let kept = match this_circle.side_of_point(point) {
        Decision::Pos | Decision::Zero => true,
        Decision::Neg => false,
        Decision::Undecidable { enclosure } => {
            return Err(gave_up(
                this_circle.predicate_name(),
                point,
                Level::Exact,
                enclosure,
                Measures::Area,
            ));
        }
        other => panic!("a pipe came back with a decision nothing here knows: {other:?}"),
    };
    Ok(match (outer, kept, side) {
        (true, true, side) => side,
        (true, false, _) => Decision::Neg,
        (false, true, Decision::Pos) => Decision::Neg,
        (false, true, side) => side,
        (false, false, _) => Decision::Pos,
    })
}

/// The flat half-space that names the one meridian circle `flat` cuts `ring`
/// in through `corner`, or `None` when the plane does not cut the ring in
/// meridian circles or no one of them certainly keeps the corner.
///
/// `fuse/select.rs` asks it of a flat piece; [`what_a_round_wire_encloses`]
/// asks [`the_picker_naming_the_circle_through`] of a face of a solid, which is
/// this with the circles round the axis left in (`docs/decisions.md` §250.5,
/// §250.9).
///
/// **One half-space, not a list.** A run is taken only when exactly one
/// [`Picker::InFrontOf`] picks it, because that is what [`intersect`] makes of a
/// plane through a ring's axis — one half-space per circle, the one it has
/// proved clears both — and it is that proof, and nothing about a list, that
/// makes each disk the part of the tube on its own circle's side.
///
/// # Errors
///
/// Whatever [`intersect`] gives for the two.
pub(crate) fn the_half_space_naming_the_meridian_circle_through(
    flat: &Surface,
    ring: &Surface,
    corner: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<Picker>> {
    Ok(
        the_picker_naming_the_circle_through(flat, ring, corner, policy, log)?
            .filter(|picker| matches!(picker, Picker::InFrontOf(_))),
    )
}

/// The one picker [`intersect`] names the circle `flat` cuts `ring` in through
/// `corner` by, or `None` when no one circle certainly keeps the corner.
///
/// **The one place this is looked up.** A plane through the ring's axis gives
/// each meridian circle one flat half-space, and a plane square to the axis
/// gives each circle round the axis one side of the pipe of the ring's own
/// radius. A run is taken only when exactly one picker picks it.
///
/// # Errors
///
/// Whatever [`intersect`] gives for the two.
fn the_picker_naming_the_circle_through(
    flat: &Surface,
    ring: &Surface,
    corner: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<Picker>> {
    let SurfaceMeet::Curves(runs) = intersect(flat, ring, policy, log)? else {
        return Ok(None);
    };
    let mut keeping = runs.iter().filter_map(|run| match run.run() {
        EdgeRun::PartOfWhereItCrosses { picked_by, .. } => match picked_by.as_slice() {
            [only] if only.side_of_point(corner) == Decision::Pos => Some(*only),
            _ => None,
        },
        EdgeRun::Straight
        | EdgeRun::RightRoundWhereItsFacesCross
        | EdgeRun::RightRoundWhereItCrosses { .. }
        | _ => None,
    });
    Ok(match (keeping.next(), keeping.next()) {
        (Some(only), None) => Some(only),
        _ => None,
    })
}

/// Whether the half-space naming a meridian circle keeps `point`: `true` on or
/// in front of it, `false` behind it. Asked of a point the tube holds, `true`
/// puts it in this circle's disk and `false` in the other's.
///
/// **A point on the half-space's own plane is kept**, because that is what a
/// [`Picker::InFrontOf`] is: everything on or in front of a flat surface. It is
/// the reading the circle's own edge is picked by, and `fuse/select.rs` asks
/// through this function too, so the two files cannot disagree about it again.
///
/// **No point on the face's plane gets that far.** [`intersect`] names meridian
/// circles only for a plane that holds the ring's axis exactly, and only once
/// it has proved the half-space's plane clears both circles. So the tube's
/// inside on the face's plane is the two disks, each strictly on one side of
/// the half-space's plane, and a point on the half-space's plane that the tube
/// holds is off the face's plane. [`SolidView::face_holds_the_point`] does not
/// ask whether its point is on the face, so it can be handed one; a refusal
/// here would be a check for that made in this one place and nowhere else.
///
/// # Errors
///
/// [`Error::Undecidable`], naming the half-space, when it cannot say.
pub(crate) fn the_meridian_half_space_keeps(
    this_circle: &Picker,
    point: PredicatePoint,
) -> Result<bool> {
    match this_circle.side_of_point(point) {
        Decision::Pos | Decision::Zero => Ok(true),
        Decision::Neg => Ok(false),
        Decision::Undecidable { enclosure } => Err(gave_up(
            this_circle.predicate_name(),
            point,
            Level::Exact,
            enclosure,
            Measures::Length,
        )),
        other => panic!("a half-space came back with a decision nothing here knows: {other:?}"),
    }
}

/// One polygonal wire's contribution to the crossing count, or `None` when
/// `point` sits on one of its sides.
///
/// Every side must be straight between its two corners: stored as
/// [`EdgeRun::Straight`], or a piece of a cone's ruling
/// ([`is_a_piece_of_a_ruling`]), which is the same segment stored as part of a
/// crossing.
#[expect(
    clippy::too_many_arguments,
    reason = "the solid is what a ruling side's face across is found in, and every other one \
              was here before it"
)]
fn walk_one_outline(
    solid: SolidView<'_>,
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
        let edge = coedge.edge();
        if edge.stored_run() != &EdgeRun::Straight && !is_a_piece_of_a_ruling(solid, face, edge)? {
            return Err(Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() });
        }
    }

    // The corner the half-line aims at is on its line whatever `point` is: two
    // rows of that determinant are the same, so it is zero exactly. Asked, a
    // `point` that is a box gets an enclosure round zero and no sign.
    let sides: Vec<Decision> = corners
        .iter()
        .map(|&corner| {
            if corner == aim {
                Decision::Zero
            } else {
                decide(corner, aim, target, point, policy, log)
            }
        })
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
            Decision::Undecidable { enclosure } => {
                return Err(gave_up_turning_a_corner(point, enclosure));
            }
            other => panic!("a corner came back with a decision nothing here knows: {other:?}"),
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
    if let Surface::Cone(cone) = surface {
        match is_on_the_nappe_the_face_is_on(face, cone, point)? {
            Decision::Pos => (),
            Decision::Zero => on_its_boundary = true,
            Decision::Neg => return Ok(OnAFace::NotHeld),
            Decision::Undecidable { enclosure } => {
                return Err(gave_up_on_a_side(surface, point, enclosure));
            }
            other => panic!("a side came back with a decision nothing here knows: {other:?}"),
        }
    }
    let trimmings = what_trims_this_face(solid, face)?;
    if is_a_ring_face_between_two_planes(surface, &trimmings) {
        return what_a_ring_face_between_two_planes_holds(solid, face, point);
    }
    let trimmed_once = trimmings.len() == 1;
    for trimming in trimmings {
        if !every_piece_of_that_crossing_is_an_edge(
            surface,
            &trimming.surface,
            trimming.corners_on_it.len(),
            policy,
            log,
        )? {
            return Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face: face.key() });
        }
        let inwards = match the_side_this_face_lies_on(face, &trimming)? {
            Some(side) => Some(side),
            None if trimmed_once => the_side_the_apex_is_on(surface, &trimming)?,
            None => None,
        }
        .ok_or(Error::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn { face: face.key() })?;
        let side = side_of_surface(&trimming.surface, point);
        match side {
            Decision::Zero => on_its_boundary = true,
            settled if settled == inwards => (),
            Decision::Pos | Decision::Neg => return Ok(OnAFace::NotHeld),
            Decision::Undecidable { enclosure } => {
                return Err(gave_up_on_a_side(&trimming.surface, point, enclosure));
            }
            other => panic!("a side came back with a decision nothing here knows: {other:?}"),
        }
    }
    Ok(if on_its_boundary {
        OnAFace::OnItsBoundary
    } else {
        OnAFace::Held
    })
}

/// Whether a face on a ring is trimmed by two planes and has one edge along
/// each — the wall of a bend of pipe, whose two end caps each cut the ring in
/// two circles of which the wall has one.
///
/// The general rule in [`what_a_curved_face_holds`] refuses every face of that
/// shape, because a plane that cuts a ring in two circles and holds only one of
/// them as an edge of this face leaves it unable to say which of the two
/// pieces that plane cuts the ring into is the face. Answering it takes the
/// plane's own geometry, which [`what_a_ring_face_between_two_planes_holds`]
/// reads.
///
/// **Two caps stored on one plane are one trimming with both edges along it**,
/// because [`what_trims_this_face`] gathers edges by the surface across them. A
/// half bend's caps both lie on the plane through its two ends, so its wall
/// comes here as one plane with two edges. The general rule refused that too:
/// every corner of the wall is on the plane, so no corner says which side of it
/// the wall is on (`docs/decisions.md` §377).
fn is_a_ring_face_between_two_planes(surface: &Surface, trimmings: &[Trimming]) -> bool {
    matches!(surface, Surface::Torus(_))
        && trimmings
            .iter()
            .all(|trimming| trimming.surface.as_plane().is_some())
        && trimmings
            .iter()
            .map(|trimming| trimming.corners_on_it.len())
            .sum::<usize>()
            == 2
}

/// Whether a ring's face trimmed by two planes, one edge along each, holds
/// `point`, which is taken as being on the ring (`docs/decisions.md` §243).
///
/// **Only planes through the ring's axis are followed.** Such a plane cuts the
/// ring in two circles round the tube, each at one angle round the axis. With
/// one of those circles from each plane as its only edges, the face is one of
/// the two stretches of ring between them, and it is the part of the ring in
/// front of both planes exactly when that stretch is less than a half-turn.
/// So:
///
/// 1. **Which side of each plane the face keeps** is read off the way its wire
///    walks the circle on that plane, by the closed-edge rule
///    [`a_closed_crossing_is_walked_the_way_it_is_stored`] that
///    `Model::validate` checks every closed edge against. Not off a corner: a
///    three-quarter bend's wall has its far corner on the other side of the
///    first cap's plane from where the wall leaves that cap.
/// 2. **Two planes that are one plane leave half of the ring** — a half bend's
///    wall, whose two ends lie on the plane through both of them. Both hold
///    the ring's centre and their normals lie along one line, so they are one
///    plane, and it cuts the ring less its two circles into two halves, one
///    each side. A face with only that plane's circles as edges is one of the
///    halves, and the side its wire keeps says which. Every corner of it is on
///    the plane, so no corner could (`docs/decisions.md` §377). Each side is
///    read against its own stored normal, and two normals along one line can
///    point opposite ways, so the two sides must differ exactly when the
///    normals do; a wire that leaves one circle on one side and the other on
///    the other is not a half.
/// 3. **Otherwise each circle must lie strictly in front of the other plane.**
///    A circle round the tube sits at one angle round the axis, so its
///    corner's side is the whole circle's. This is what turns away a stretch
///    of more than a half-turn, which is the part of the ring in front of
///    *either* plane and not of both.
///
/// Then the stretch between the circles is the part of the ring in front of
/// both planes, and `point` is asked against the planes as they read once
/// moved into place. Everything else is asked of the face where it is stored,
/// for §168's reason: a movement keeps each of these facts and rounds the
/// numbers they are read from.
///
/// # Errors
///
/// [`Error::AFaceMissesPartOfWhereItsSurfacesCross`] when a plane does not hold
/// the ring's axis, a circle is not in front of the other plane, or the wire
/// keeps one plane's two sides, which is what the general rule gave every face
/// of this shape.
/// [`Error::CannotTellWhichWayAClosedEdgeRuns`] when the closed-edge rule
/// cannot, and [`Error::Undecidable`] when a side, or whether a plane holds the
/// axis, could not be settled.
fn what_a_ring_face_between_two_planes_holds(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    point: PredicatePoint,
) -> Result<OnAFace> {
    let refuse = || Error::AFaceMissesPartOfWhereItsSurfacesCross { face: face.key() };
    let ring = face.stored_surface();
    let Surface::Torus(torus) = ring else {
        panic!("only a face on a ring is sent here, and this one is on {ring:?}");
    };
    let mut cuts: Vec<Cut> = Vec::with_capacity(2);
    for coedge in face.wires().flat_map(WireView::coedges) {
        let edge = coedge.edge();
        let Some(across) = the_face_across(solid, face, edge)? else {
            continue;
        };
        if across.location() != face.location() {
            return Err(refuse());
        }
        let cap = across.stored_surface();
        let Some(flat) = cap.as_plane() else {
            panic!("a face this was sent is trimmed by planes only, and this one by {cap:?}");
        };
        match how_the_axis_lies_on(flat, &torus) {
            Decision::Zero => (),
            Decision::Pos | Decision::Neg => return Err(refuse()),
            Decision::Undecidable { enclosure } => {
                return Err(gave_up_on_the_axis(flat, &torus, enclosure));
            }
            other => panic!("the axis came back with a decision nothing here knows: {other:?}"),
        }
        let corner = edge.start().key();
        let at = where_it_is_stored(solid, corner);
        let names_it = which_two_surfaces_a_run_is_between(edge.stored_run(), [&ring, &cap]);
        if names_it != [ring, cap] && names_it != [cap, ring] {
            return Err(refuse());
        }
        let outward = [
            gradient_of(&ring, at) * face.orientation().sign(),
            gradient_of(&cap, at) * across.orientation().sign(),
        ];
        let Some(forward_if_it_keeps_the_back) = a_closed_crossing_is_walked_the_way_it_is_stored(
            [&names_it[0], &names_it[1]],
            outward,
            at,
            Decision::Neg,
        )?
        else {
            return Err(refuse());
        };
        // `Model::validate` hands the rule the side its corners read, turned
        // round when the plane's face is used reversed; this is that read
        // backwards.
        let walked_forward = coedge.orientation_on_its_edge() == Orientation::Forward;
        let keeps_the_back = walked_forward == forward_if_it_keeps_the_back;
        let kept = if keeps_the_back != across.orientation().is_reversed() {
            Decision::Neg
        } else {
            Decision::Pos
        };
        cuts.push(Cut {
            surface: cap,
            placed: across.surface(),
            kept,
            corners_on_it: vec![corner],
        });
    }
    let [one, other] = cuts.as_slice() else {
        panic!(
            "two planes with one edge along each leave two cuts, and {} came back",
            cuts.len()
        );
    };
    if let (Some(first), Some(second)) = (one.surface.as_plane(), other.surface.as_plane())
        && the_normals_lie_along_one_line(first, second)
    {
        if (one.kept == other.kept) != point_the_same_way(first, second) {
            return Err(refuse());
        }
        return held_in_front_of_all(core::slice::from_ref(one), point);
    }
    for (circle, plane) in [(one, other), (other, one)] {
        let at =
            PredicatePoint::from(where_it_is_stored(solid, circle.corners_on_it[0]).coordinates());
        match side_of_surface(&plane.surface, at) {
            settled if settled == plane.kept => (),
            Decision::Zero | Decision::Pos | Decision::Neg => return Err(refuse()),
            Decision::Undecidable { enclosure } => {
                return Err(gave_up_on_a_side(&plane.surface, at, enclosure));
            }
            other => panic!("a side came back with a decision nothing here knows: {other:?}"),
        }
    }
    held_in_front_of_all(&cuts, point)
}

/// Whether two planes' normals, already shown to lie along one line, point the
/// same way.
///
/// Along one line, each normal is an exact multiple of the other, so a
/// component is zero in both or in neither, and the multiple's sign is whether
/// the signs of any other component agree. Nothing is worked out, so nothing
/// rounds.
fn point_the_same_way(one: &Plane, other: &Plane) -> bool {
    let (one, other) = (one.normal().components(), other.normal().components());
    let axis = (0..3)
        .find(|&axis| one[axis] != 0.0)
        .expect("a plane's normal has a length");
    assert!(
        other[axis] != 0.0,
        "two normals along one line are zero along the same axes: {one:?} and {other:?}"
    );
    (one[axis] > 0.0) == (other[axis] > 0.0)
}

/// Whether the whole of `torus`'s axis lies on `flat`: [`Decision::Zero`] when
/// it does — the plane's normal square to the axis direction and the ring's
/// centre on the plane, both exactly — [`Decision::Pos`] or [`Decision::Neg`]
/// when it does not, and [`Decision::Undecidable`] when neither question says
/// no and one of them could not say.
///
/// The same two questions [`intersect`] asks before it names the two circles
/// round the tube, asked of the numbers as stored and settled the same way:
/// where the enclosure straddles zero, in exact arithmetic. A normal worked out
/// square to a slanted axis straddles zero every time, whether or not its
/// rounding left it square, and reading that as "no" refused every bend whose
/// axis is not along a coordinate axis (`docs/decisions.md` §243.7, §328).
///
/// A sign other than zero is the true answer and not a refusal of one: that
/// plane cuts the ring in a quartic, and [`intersect`] says so too. Which of
/// the two signs it is says nothing more.
fn how_the_axis_lies_on(flat: &Plane, torus: &rcad_geom::Torus) -> Decision {
    let (leans, _) = rcad_predicates::decide(&SquareToTheAxis {
        named: WHERE_A_PLANE_AND_A_TORUS_CROSS,
        normal: flat.normal().components(),
        along: torus.axis().direction().components(),
    });
    let centre = flat.side_of_point(PredicatePoint::from(torus.centre().coordinates()));
    match (leans, centre) {
        (Decision::Pos | Decision::Neg, _) => leans,
        (_, Decision::Pos | Decision::Neg) => centre,
        (Decision::Undecidable { .. }, _) => leans,
        (_, Decision::Undecidable { .. }) => centre,
        (Decision::Zero, Decision::Zero) => Decision::Zero,
        other => panic!("the axis came back with a decision nothing here knows: {other:?}"),
    }
}

/// Whether a plane's normal is square to a ring's or a pipe's axis: the sign of
/// the dot of the two as stored. [`the_normals_lie_along_one_line`] asks it of
/// each component of a cross product, which is the same kind of sum.
///
/// A sum of three products of `f64`s, so level 3 settles it exactly — unless
/// one of those products is not zero and is below `2^-969`, where
/// [`product_is_exact`] says its rounding can no longer be held. `intersect`
/// stops at the same place (`docs/decisions.md` §328).
struct SquareToTheAxis {
    named: &'static str,
    normal: [f64; 3],
    along: [f64; 3],
}

impl Predicate for SquareToTheAxis {
    fn name(&self) -> &'static str {
        self.named
    }

    fn on_floats(&self) -> Option<Filtered> {
        None
    }

    fn on_intervals(&self) -> Interval {
        dot(exactly(self.normal), exactly(self.along))
    }

    fn exactly(&self) -> Option<Decision> {
        let mut total = [0.0; 6];
        let mut used = 0;
        for axis in 0..3 {
            let (one, other) = (self.normal[axis], self.along[axis]);
            if !product_is_exact(one, other) {
                return None;
            }
            let (product, roundoff) = two_product(one, other);
            let mut sum = [0.0; 6];
            used = expansion_sum(&total[..used], &[roundoff, product], &mut sum);
            total = sum;
        }
        match expansion_sign(&total[..used]) {
            Decision::Neg => Some(Decision::Neg),
            Decision::Zero => Some(Decision::Zero),
            Decision::Pos => Some(Decision::Pos),
            // The sum overflowed. Exact arithmetic that is not exact has
            // nothing to offer, so this level cannot run.
            Decision::Undecidable { .. } => None,
            other => {
                panic!("exact arithmetic came back with a decision nothing here knows: {other:?}")
            }
        }
    }
}

/// Whether `point`, taken as being on `cone`, is on the half of it `face` is on:
/// [`Decision::Pos`] on the face's half, [`Decision::Neg`] on the other,
/// [`Decision::Zero`] at the apex, where the two halves meet.
///
/// **A face on a cone lies on one half of it** — one nappe — and the trimming
/// loop does not look at which. A cone's surface is both halves, so a point on
/// the other half is on the surface too, and a trimming only turns it away if
/// it lies on the far side of one. A pointed cone's only trimming is its cap,
/// and the whole of the other half lies below the cap, on the side the face
/// keeps — so without this the skin would hold all of it, and a ray from
/// inside the cone out through the other half would count two crossings that
/// are not there (`docs/decisions.md` §200.3).
///
/// Which half is read off the face's corners, as the side of a trimming is:
/// the sign of `(corner − apex) · w`, which no corner may disagree on.
///
/// # Errors
///
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] when the face has no corner
/// off the apex to read its half from, or its corners disagree — a face across
/// the apex, which no builder here makes and a file might.
///
/// [`Error::Undecidable`] when a corner's half could not be settled; `point`'s
/// own comes back as [`Decision::Undecidable`] for the caller to refuse.
fn is_on_the_nappe_the_face_is_on(
    face: FaceView<'_>,
    cone: &rcad_geom::Cone,
    point: PredicatePoint,
) -> Result<Decision> {
    let cannot = || Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() };
    let apex = exactly(cone.apex().coordinates());
    let up = exactly(cone.axis().direction().components());
    let how_far_up =
        |at: [Interval; 3]| dot(core::array::from_fn(|axis| at[axis] - apex[axis]), up).sign();

    let mut faces_half: Option<Decision> = None;
    for corner in face.wires().flat_map(WireView::points) {
        let side = how_far_up(exactly(corner.coordinates()));
        match side {
            Decision::Zero => (),
            Decision::Pos | Decision::Neg => match faces_half {
                None => faces_half = Some(side),
                Some(already) if already == side => (),
                Some(_) => return Err(cannot()),
            },
            Decision::Undecidable { enclosure } => {
                let at = PredicatePoint::from(corner.coordinates());
                return Err(gave_up_on_a_side(&Surface::Cone(*cone), at, enclosure));
            }
            other => panic!("a side came back with a decision nothing here knows: {other:?}"),
        }
    }
    let faces_half = faces_half.ok_or_else(cannot)?;
    Ok(match how_far_up(point.enclosure()) {
        Decision::Zero => Decision::Zero,
        Decision::Pos if faces_half == Decision::Pos => Decision::Pos,
        Decision::Neg if faces_half == Decision::Neg => Decision::Pos,
        Decision::Pos | Decision::Neg => Decision::Neg,
        Decision::Undecidable { enclosure } => Decision::Undecidable { enclosure },
        other => panic!("a side came back with a decision nothing here knows: {other:?}"),
    })
}

/// Which side of its one trimming a cone face lies on when none of its corners
/// is off that trimming to say — the side the apex is on.
///
/// **This is the pointed cone**: one face on one half of a cone, bounded by one
/// closed curve and nothing else, with its one corner on that curve. A plane,
/// or a pipe on the same axis, crosses one half of a cone in a closed curve
/// only by going right round the axis, and that cuts the half into two pieces:
/// the one reaching the apex, which is bounded, and the one running off to
/// infinity, which is not. A face is bounded, so it is the first, and the side
/// of the trimming it lies on is the apex's.
///
/// `None` for anything outside that family — a surface that is not a cone, a
/// trimming that is not a plane or a pipe, or an apex on the trimming itself —
/// which the caller refuses by name as before.
///
/// # Errors
///
/// [`Error::Undecidable`] when the apex's side could not be settled.
fn the_side_the_apex_is_on(surface: &Surface, trimming: &Trimming) -> Result<Option<Decision>> {
    let Surface::Cone(cone) = surface else {
        return Ok(None);
    };
    if !matches!(trimming.surface, Surface::Plane(_) | Surface::Cylinder(_)) {
        return Ok(None);
    }
    let apex = PredicatePoint::from(cone.apex().coordinates());
    match side_of_surface(&trimming.surface, apex) {
        Decision::Pos => Ok(Some(Decision::Pos)),
        Decision::Neg => Ok(Some(Decision::Neg)),
        Decision::Zero => Ok(None),
        Decision::Undecidable { enclosure } => {
            Err(gave_up_on_a_side(&trimming.surface, apex, enclosure))
        }
        other => panic!("a side came back with a decision nothing here knows: {other:?}"),
    }
}

/// Whether a side of `face` runs along only *part* of where its surface crosses
/// the face across it — half of a circle rather than the whole of it.
///
/// A seam is left out: the same face lies on both sides of it, so it trims
/// nothing, and a ball's seam is part of a crossing in exactly this sense.
///
/// So is a wire that is one whole circle walked as two arcs
/// ([`is_one_circle_cut_in_two`]): each arc is part of the crossing, but the
/// two together are all of it, and the round-wire count answers such a wire
/// where §167's family refuses a hole in a flat face.
///
/// So is a piece of a cone's ruling on a **flat** face
/// ([`is_a_piece_of_a_ruling`]). It is straight, so a flat face whose other
/// sides are straight too is a polygon and is walked as one: the face of a
/// half frustum down its axis (`docs/decisions.md` §389). On a curved face a
/// ruling is still part of a crossing, read as a straight side by
/// [`the_sides_of`].
fn a_side_of_it_is_part_of_a_crossing(solid: SolidView<'_>, face: FaceView<'_>) -> Result<bool> {
    let flat = face.stored_surface().as_plane().is_some();
    for wire in face.wires() {
        if is_one_circle_cut_in_two(solid, face, wire)? {
            continue;
        }
        for coedge in wire.coedges() {
            let edge = coedge.edge();
            let is_a_piece = matches!(edge.stored_run(), EdgeRun::PartOfWhereItCrosses { .. })
                && edge.start().key() != edge.end().key();
            if !is_a_piece || the_face_across(solid, face, edge)?.is_none() {
                continue;
            }
            if !(flat && is_a_piece_of_a_ruling(solid, face, edge)?) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Whether `wire`, on a flat face, is one whole circle walked as two arcs —
/// where the face's plane cuts a ball, cut in two because the walk started at
/// a pole (§194). `fuse/select.rs`'s `round_wire_of` reads the same wire as
/// round.
///
/// **It is proved, not read off the wire having two sides.** Both sides must
/// have the same ball across them, stored in the face's own frame, so both lie
/// on the one circle where the face's plane cuts that ball. Two sides of a
/// closed wire run between the same two corners, so they are either the
/// circle's two arcs either side of the chord between those corners or the
/// same arc twice. [`the_way_into_an_arc`] tells those apart: the two arcs of
/// one circle leave the chord in opposite directions. Anything that does not
/// settle all of that is `false`, and the face is then asked the way it was
/// before.
fn is_one_circle_cut_in_two(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    wire: WireView<'_>,
) -> Result<bool> {
    let surface = face.stored_surface();
    if surface.as_plane().is_none() {
        return Ok(false);
    }
    let coedges: Vec<CoedgeView<'_>> = wire.coedges().collect();
    let [one, other] = coedges.as_slice() else {
        return Ok(false);
    };
    if one.start().key() == one.end().key() {
        return Ok(false);
    }
    let ends = [one.start(), one.end()].map(|corner| where_it_is_stored(solid, corner.key()));
    let mut ball: Option<Surface> = None;
    let mut ways = Vec::new();
    for coedge in [one, other] {
        let edge = coedge.edge();
        let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = edge.stored_run() else {
            return Ok(false);
        };
        let Some(across) = the_face_across(solid, face, edge)? else {
            return Ok(false);
        };
        let across_surface = across.stored_surface();
        let is_the_one_ball = matches!(across_surface, Surface::Sphere(_))
            && ball.is_none_or(|known| known == across_surface);
        if across.location() != face.location() || !is_the_one_ball {
            return Ok(false);
        }
        ball = Some(across_surface);
        let Some(way) = the_way_into_an_arc(&surface, &across_surface, ends, picked_by) else {
            return Ok(false);
        };
        ways.push(way);
    }
    Ok(dot(ways[0], ways[1]).sign() == Decision::Neg)
}

/// Whether a face with part of a crossing among its sides holds `point`, which
/// is taken as being on that face's surface (`docs/decisions.md` §167).
///
/// **The face is still the part of its surface on one side of everything that
/// trims it**, as [`what_a_curved_face_holds`] reads it — but that is no longer
/// true of every face, and here it is proved of this one before it is used. A
/// quarter of a ball's skin is the part of the sphere in front of both walls; the
/// three quarters are the part in front of *either*. So:
///
/// 1. **Which side of each neighbour the face keeps** is read off the way its
///    wire runs, because nothing else in the model says it: both corners of a
///    lune sit on both walls, so no corner is off either of them.
/// 2. **Every side of the face must lie on that side of every other
///    neighbour**. Then the face is exactly the part of its surface in front of
///    all of them. Or, on a sphere, every side lies *behind* every other
///    neighbour, and then the rest of the sphere is that part, and the face is
///    the part in front of any of them.
///
/// Why that is enough: a face whose sides all lie in front of a neighbour
/// cannot reach behind it, because the region behind is connected, holds none of
/// the face's boundary, and borders the face's own side along that neighbour on
/// the side the face does not keep. So the face sits inside the part in front
/// of all of them; and that part is connected — a plane cut by half-planes and
/// round discs, or a sphere cut by planes through its centre — with the face's
/// boundary on its boundary, so the face is all of it.
///
/// **A cone leaves one convex region on a plane it cuts in a closed curve, and
/// that is the whole of why one is admitted.** Square to the axis the curve is
/// a circle and the region its disc; leaned less than the cone's own half angle
/// it is an ellipse and the region the ellipse's inside. Either is convex, so the
/// plane cut by it and by half-planes is still one piece. The sign that tells
/// those from the parabola and the hyperbola is §208.6's
/// [`Cone::is_cut_in_a_closed_curve_by`](rcad_geom::Cone::is_cut_in_a_closed_curve_by),
/// and [`is_a_family_this_follows`] reads it — it has since §211, which is why
/// the leaned lids of `tests/a_plane_face_cut_by_a_cone_it_crosses_in_an_ellipse.rs`
/// are answered on the base that §219.8 called this a gap on. **A plane leaned
/// past that is refused and stays refused**, and that is a limit: it cuts both
/// nappes in a hyperbola, and the part of the plane inside the cone is then *two*
/// regions with nothing between them, which is exactly the connectedness this
/// argument rests on.
///
/// **Only the family where each step of that is a sign is taken**: a flat face
/// whose curved sides are arcs of a ball it lies inside or of a cone or a pipe
/// it lies inside and crosses in a closed curve, a sphere face whose sides are
/// arcs cut by planes through its centre, and a cone face or a pipe face cut by
/// planes as [`is_a_family_this_follows`] sets out. Each arc must be picked by a
/// half-space that tells it from its sibling ([`the_way_into_an_arc`]), which
/// asks the picker either to hold both its ends exactly or to have the arc's
/// whole curve in front of it with that curve's centre behind — and a cone's
/// ellipse and a pipe's circle earn that the same way a ball's circle does,
/// through [`where_a_closed_crossing_is_centred`]. Anything else is refused by
/// name, as before.
///
/// **All of that is asked of the face where it is stored, and only `point` is
/// asked where it sits** (§168). Each step is a property a movement keeps —
/// a plane holding two points, a plane through a centre, which way a wire runs
/// against a cross product — but a movement *rounds*, and a turned wedge's arc
/// ends land a hair off its picker's plane, so asking them of the placed face
/// refused every turned wedge. Where the face is stored, those facts are exact.
/// The face where it sits is then the moved image of a face shown to be the part
/// in front of its cuts, so it is the part in front of the moved cuts, and
/// `point` is asked against those as they read once moved — the same rounded
/// surfaces every other face of a placed solid is asked against (§17.3). Every
/// face across a side must be stored in the face's own frame for that to hold,
/// which one shell's faces always are.
///
/// **A cone face is asked which nappe it is on before any of that is read**, the
/// same [`is_on_the_nappe_the_face_is_on`] [`what_a_curved_face_holds`] asks. It
/// is asked in both places because neither calls the other: a face with part of a
/// crossing among its sides comes here instead, and a cone's surface is both its
/// halves, so without this the part in front of every cut reaches round the apex
/// onto the far nappe and a point there would come back **held** — a wrong
/// answer and not a refusal.
/// [`is_a_family_this_follows`] carries the argument and
/// `a_cone_face_round_an_apex_holds_its_own_nappe_and_not_the_other` pins the two
/// points that used to.
///
/// # Errors
///
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] outside that family, and
/// [`Error::Undecidable`] when a sign could not be settled.
fn what_a_face_cut_by_part_of_a_crossing_holds(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    point: PredicatePoint,
) -> Result<OnAFace> {
    let cannot = || Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() };
    let surface = face.stored_surface();
    let sides = the_sides_of(solid, face)?;

    let mut cuts: Vec<Cut> = Vec::new();
    for side in &sides {
        let kept = which_side_of_its_neighbour_a_side_keeps(face, &surface, side)?;
        let corners = [side.start.key, side.end.key];
        match cuts.iter_mut().find(|known| known.surface == side.across) {
            Some(known) if known.kept == kept => known.corners_on_it.extend(corners),
            Some(_) => return Err(cannot()),
            None => cuts.push(Cut {
                surface: side.across,
                placed: side.across_placed,
                kept,
                corners_on_it: corners.to_vec(),
            }),
        }
    }
    if !is_a_family_this_follows(&surface, &cuts) {
        return Err(cannot());
    }

    let placed = face.surface();
    let mut at_the_apex = false;
    if let Surface::Cone(cone) = placed {
        match is_on_the_nappe_the_face_is_on(face, &cone, point)? {
            Decision::Pos => (),
            Decision::Zero => at_the_apex = true,
            Decision::Neg => return Ok(OnAFace::NotHeld),
            Decision::Undecidable { enclosure } => {
                return Err(gave_up_on_a_side(&placed, point, enclosure));
            }
            other => panic!("a side came back with a decision nothing here knows: {other:?}"),
        }
    }

    let mut in_front_of_all = true;
    let mut behind_all = true;
    for cut in &cuts {
        for side in sides.iter().filter(|side| side.across != cut.surface) {
            match where_a_side_lies(&surface, side, cut) {
                Lies::On => {}
                Lies::Beside(found) if found == cut.kept => behind_all = false,
                Lies::Beside(_) => in_front_of_all = false,
                Lies::Across => return Err(cannot()),
            }
        }
    }
    if in_front_of_all {
        let held = held_in_front_of_all(&cuts, point)?;
        Ok(if at_the_apex && held == OnAFace::Held {
            OnAFace::OnItsBoundary
        } else {
            held
        })
    } else if behind_all
        && (matches!(surface, Surface::Sphere(_)) || is_two_discs_taken_together(&surface, &cuts))
    {
        held_in_front_of_any(&cuts, point)
    } else {
        Err(cannot())
    }
}

/// Whether a flat face whose every side lies behind the other cut is the
/// insides of two pipes running the same way taken together, the cap of two
/// side-by-side bars fused into one (`docs/decisions.md` §331.5).
///
/// **Its sides are the rim of the two together.** Each side is an arc of one
/// curve with both ends on the other and lying outside it
/// ([`where_an_arc_between_two_points_of_a_pipe_lies`]). The two curves meet in
/// at most two points ([`is_a_pair_of_pipes_running_the_same_way`]) and cross at
/// both, so each has exactly one arc outside the other, and the sides are those
/// two arcs: one closed curve round the two insides taken together. The face
/// keeps the inside of each pipe along its arc, and next to that arc the inside
/// of that pipe is the inside of the two together. So the face is the part of
/// the plane inside either, and a point is held when either pipe holds it.
///
/// Both have to be kept inside. Kept outside, the same arcs bound the part of
/// the plane outside both, which runs on for ever and is no face of a solid.
fn is_two_discs_taken_together(surface: &Surface, cuts: &[Cut]) -> bool {
    surface.as_plane().is_some_and(|flat| {
        is_a_pair_of_pipes_running_the_same_way(flat, cuts)
            && cuts.iter().all(|cut| cut.kept == Decision::Neg)
    })
}

/// One side of a face in the frame the face is stored in, with the surface of
/// the face across it there, and that surface as it reads once moved into
/// place.
struct Side {
    across: Surface,
    across_placed: Surface,
    start: Corner,
    end: Corner,
    run: SideRun,
}

/// A corner of a face, where it is stored.
#[derive(Clone, Copy)]
struct Corner {
    key: crate::VertexKey,
    at: Point,
}

/// What a [`Side`] runs along between its ends.
enum SideRun {
    Straight,
    /// Part of a circle, carrying the one thing about it the steps below read:
    /// a direction square to the chord, in the circle's own plane, pointing from
    /// the chord into this arc rather than into its sibling
    /// ([`the_way_into_an_arc`]).
    Arc {
        into_the_arc: [Interval; 3],
    },
}

/// One surface cutting a face: which side of it the face keeps, which of the
/// face's corners are on it because an edge along it ends there, and where it
/// is once moved into place, which is the only thing the point is asked against.
struct Cut {
    surface: Surface,
    placed: Surface,
    kept: Decision,
    corners_on_it: Vec<crate::VertexKey>,
}

/// Where the inside of one side lies against a surface cutting the face.
enum Lies {
    /// Along it, so on neither side.
    On,
    /// Strictly on this side of it, ends apart.
    Beside(Decision),
    /// Across it, or somewhere nothing here can tell.
    Across,
}

/// Every side of `face` in the order its wires walk them, seams left out, in the
/// frame the face is stored in.
fn the_sides_of(solid: SolidView<'_>, face: FaceView<'_>) -> Result<Vec<Side>> {
    let cannot = || Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() };
    let mut sides = Vec::new();
    for coedge in face.wires().flat_map(WireView::coedges) {
        let edge = coedge.edge();
        let Some(across) = the_face_across(solid, face, edge)? else {
            continue;
        };
        if across.location() != face.location() {
            return Err(cannot());
        }
        let [start, end] = [coedge.start(), coedge.end()].map(|corner| Corner {
            key: corner.key(),
            at: where_it_is_stored(solid, corner.key()),
        });
        let across_surface = across.stored_surface();
        let run = match edge.stored_run() {
            EdgeRun::Straight => SideRun::Straight,
            EdgeRun::PartOfWhereItCrosses { .. } if is_a_piece_of_a_ruling(solid, face, edge)? => {
                SideRun::Straight
            }
            EdgeRun::PartOfWhereItCrosses { picked_by, .. } if start.key != end.key => {
                SideRun::Arc {
                    into_the_arc: the_way_into_an_arc(
                        &face.stored_surface(),
                        &across_surface,
                        [start.at, end.at],
                        picked_by,
                    )
                    .ok_or_else(cannot)?,
                }
            }
            // A whole closed crossing among parts of others, or a kind of run
            // this has not been taught.
            EdgeRun::PartOfWhereItCrosses { .. }
            | EdgeRun::RightRoundWhereItsFacesCross
            | EdgeRun::RightRoundWhereItCrosses { .. }
            | _ => {
                return Err(cannot());
            }
        };
        sides.push(Side {
            across: across_surface,
            across_placed: across.surface(),
            start,
            end,
            run,
        });
    }
    Ok(sides)
}

/// Whether `edge`, a side of `face`, is a piece of a straight ruling of a cone:
/// part of where a plane holding the cone's apex crosses that cone, with its two
/// ends on one line through the apex. Such a side is the straight segment
/// between its two corners.
///
/// **A plane through the apex cuts a cone in straight lines through the apex**
/// — two, or one where it touches, or none — so a piece of that crossing is
/// straight unless it turns at the apex from one line onto the other. Its two
/// ends on one line with the apex rule the turn out: a piece that left that line
/// at the apex could only come back to it at the apex, and an edge does not run
/// through one point twice.
///
/// Every number asked is stored, in the frame the face is stored in: the
/// plane's own sign at the apex, and three [`orient2d`]s, which are the three
/// components of `(start − apex) × (end − apex)`. A corner rounded off its line
/// fails the second and the side is refused as it was; the corners of a half
/// frustum built by hand down its axis pass (`docs/decisions.md` §389).
///
/// # Errors
///
/// Whatever [`the_face_across`] gives for the edge.
fn is_a_piece_of_a_ruling(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    edge: EdgeView<'_>,
) -> Result<bool> {
    if !matches!(edge.stored_run(), EdgeRun::PartOfWhereItCrosses { .. })
        || edge.start().key() == edge.end().key()
    {
        return Ok(false);
    }
    let Some(across) = the_face_across(solid, face, edge)? else {
        return Ok(false);
    };
    if across.location() != face.location() {
        return Ok(false);
    }
    let ((Surface::Plane(flat), Surface::Cone(cone)) | (Surface::Cone(cone), Surface::Plane(flat))) =
        (face.stored_surface(), across.stored_surface())
    else {
        return Ok(false);
    };
    let apex = cone.apex().coordinates();
    if flat.side_of_point(PredicatePoint::from(apex)) != Decision::Zero {
        return Ok(false);
    }
    let ends = [edge.start().key(), edge.end().key()]
        .map(|corner| where_it_is_stored(solid, corner).coordinates());
    Ok([(0, 1), (1, 2), (2, 0)].into_iter().all(|(i, j)| {
        let [from, start, end] = [apex, ends[0], ends[1]].map(|at| [at[i], at[j]]);
        orient2d(from, start, end) == Decision::Zero
    }))
}

/// Where a corner of `solid` is in the frame its faces are stored in.
///
/// Read from the arena because [`crate::view::VertexView`] only hands out
/// the corner moved into place, and moving it back would round it twice.
fn where_it_is_stored(solid: SolidView<'_>, corner: crate::VertexKey) -> Point {
    let model = solid.model();
    model.data().vertices[corner.slot_in(model.id())].point
}

/// A direction square to the chord between `ends`, lying in the plane the arc's
/// circle lies in, pointing from the chord into **this** arc and not into the
/// other one with the same two ends; `None` when nothing in `picked_by` says
/// which of the two this is.
///
/// This is the one thing the three steps below read out of an arc, and it is
/// read out of the picker's **normal** alone. Where the picker's plane sits is
/// never asked, which is what lets a fused arc in: the fuse lowers that plane
/// until the predicate says both ends are on or in front of it
/// (`the_half_space_both_ends_are_on`), so a fused arc's ends are off its
/// picker's plane where they are stored, and §167 asked them to be on it.
///
/// # What the picker has to say, and why the reading is then sound
///
/// The arc's own curve is where the flat one of the two surfaces cuts the round
/// one, and it has to be a curve that **closes**: a ball's circle, or a cone's
/// or a pipe's circle or ellipse ([`where_a_closed_crossing_is_centred`]). Its
/// chord line splits that curve in two, and the whole of this arc is strictly on
/// one side of the line, because a line meets a conic at two points and those
/// two are the arc's own ends. So "which of the two arcs" *is* "which side of
/// the chord", and `square_to_the_chord` below names the two sides — the chord
/// read off the corners as they are stored, which for a fused solid is a rounded
/// world frame and is the same frame every other step here reads (§17.3, §168).
/// The picker has to settle which side, and it does so in one of two ways:
///
/// - **Its plane holds both ends**, which is §167's test and what a face built
///   by hand gives. That plane then meets the curve's plane in the chord line
///   itself, and the arc is in front, so the arc is on the side the normal
///   points to.
/// - **Both ends are on or in front of it and the curve's centre is on or
///   behind it** ([`where_the_curves_centre_sits`]), which is what a fused arc
///   gives.
///
/// # Why the second way works, on a circle and then on an ellipse
///
/// On a **circle**: the whole arc is on or in front of the picker and the picker
/// is at or above the centre, so the whole arc is in the closed half of the
/// circle the normal points to from the centre. The arc's own midpoint is in
/// there with it, and a midpoint sits a radius from the centre square to the
/// chord, on the arc's side of it — so the normal reads positive against that
/// side.
///
/// **That last step is the circle's alone and does not carry over.** On an
/// ellipse a midpoint does not sit square to the chord from the centre, so
/// anyone repeating the sentence with "ellipse" written in has said something
/// false. What carries over is that the whole claim is **affine**:
///
/// - Everything happens inside the curve's own plane. Restricted to it the
///   picker is an affine function `f`, the hypothesis is `f ≥ 0` all along the
///   arc and `f ≤ 0` at the centre, and the conclusion is that `f` grows across
///   the chord from the sibling arc towards this one.
/// - An affine map of that plane leaves every word of both alone. A chord stays
///   a chord, the two arcs stay the two arcs, the centre of a conic stays its
///   centre, a half-plane stays a half-plane, and an affine function stays
///   affine, so which side of the chord it grows towards is unchanged. Being
///   *square* to something is not preserved — and neither the hypothesis nor the
///   conclusion mentions it.
/// - Some affine map of the plane carries any ellipse to a circle, and the
///   circle is the paragraph above.
///
/// Either way the arc is on the side of the chord the picker's normal points to,
/// so `square_to_the_chord` turned round to read positive against that normal
/// points from the chord into the arc. That sign is the only reading taken of
/// the picker; how nearly square to the chord its normal is never comes into it,
/// and neither does how far the fuse lowered its plane.
///
/// **The tangent goes the same way**, and it needs no affine argument because it
/// never mentioned the centre. Every point of the arc is off the chord on the
/// arc's own side — `(p − end) · into_the_arc > 0` for each of them — so the
/// direction the arc leaves either end in, being a limit of those, reads
/// non-negative against `into_the_arc` too. It reads zero only where the tangent
/// lies along the chord, which means the chord is tangent there; a line meets a
/// conic at two points, so a chord tangent at one of its ends has its two ends at
/// one point. That is why steps 1 and 3 can read a *direction* off this same
/// vector.
fn the_way_into_an_arc(
    surface: &Surface,
    across: &Surface,
    ends: [Point; 2],
    picked_by: &[Picker],
) -> Option<[Interval; 3]> {
    let flat = surface.as_plane().or_else(|| across.as_plane())?;
    let centred = where_a_closed_crossing_is_centred(surface, flat)
        .or_else(|| where_a_closed_crossing_is_centred(across, flat))?;
    let [from, to] = ends.map(|at| at.coordinates());
    let chord =
        core::array::from_fn(|axis| Interval::exactly(to[axis]) - Interval::exactly(from[axis]));
    let square_to_the_chord = cross(exactly(flat.normal().components()), chord);
    let asked = ends.map(|at| PredicatePoint::from(at.coordinates()));

    picked_by.iter().find_map(|picker| {
        let half = match picker {
            Picker::InFrontOf(half) => half,
            Picker::OutsideABall(_) | Picker::OutsideAPipe(_) | Picker::InsideAPipe(_) | _ => {
                return None;
            }
        };
        if !a_picker_that_tells_the_two_arcs_apart(half, flat, &centred, asked) {
            return None;
        }
        match dot(exactly(half.normal().components()), square_to_the_chord).sign() {
            Decision::Pos => Some(square_to_the_chord),
            Decision::Neg => Some(square_to_the_chord.map(core::ops::Neg::neg)),
            Decision::Zero | Decision::Undecidable { .. } | _ => None,
        }
    })
}

/// Whether `half` says which of the two arcs between `ends` an edge is, by one
/// of the two ways [`the_way_into_an_arc`] sets out.
///
/// The fuse's own arrangement asks the same question of the same two ways, on
/// the boxes it solved for rather than on stored corners, so it asks this
/// (§23.8, and §211.5 for the three spellings of the closed-curve test that
/// went the same way).
pub(crate) fn a_picker_that_tells_the_two_arcs_apart(
    half: &HalfSpace,
    flat: &Plane,
    centred: &WhereTheCurveIsCentred,
    ends: [PredicatePoint; 2],
) -> bool {
    let sides = ends.map(|at| half.side_of_point(at));
    if sides.iter().all(|side| *side == Decision::Zero) {
        return true;
    }
    if !sides
        .iter()
        .all(|side| matches!(side, Decision::Zero | Decision::Pos))
    {
        return false;
    }
    matches!(
        where_the_curves_centre_sits(half, flat, centred),
        Decision::Zero | Decision::Neg
    )
}

/// Which side of `half` the centre of the closed curve `flat` cuts sits on.
///
/// That centre is `from + ((e − n·from)/(n·along)) along`, with `n` and `e` the
/// plane's normal and offset, and the division is left out rather than taken:
/// `n·along` is positive, so the sign wanted is the sign of
///
/// ```text
/// (h·from − d)(n·along)  +  (e − n·from)(h·along)
/// ```
///
/// which is a polynomial in numbers that are stored exactly. How big the round
/// surface is never comes into it.
fn where_the_curves_centre_sits(
    half: &HalfSpace,
    flat: &Plane,
    centred: &WhereTheCurveIsCentred,
) -> Decision {
    let normal = exactly(flat.normal().components());
    let facing = exactly(half.normal().components());
    let from = exactly(centred.from.coordinates());
    let above = Interval::exactly(flat.offset()) - dot(normal, from);
    ((dot(facing, from) - Interval::exactly(half.offset())) * dot(normal, centred.along)
        + above * dot(facing, centred.along))
    .sign()
}

/// Where the closed curve `flat` cuts a round surface in is centred: walk
/// `along` from `from` until the plane is reached.
///
/// The centre itself is not formed, because forming it divides and
/// [`where_the_curves_centre_sits`] wants a sign of it (§168).
pub(crate) struct WhereTheCurveIsCentred {
    from: Point,
    along: [Interval; 3],
}

/// How the curve where `flat` cuts `surface` is centred, or `None` when the two
/// do not cross in a curve that closes.
///
/// A plane cuts a **ball** in a circle centred at the foot of the perpendicular
/// from the ball's centre, whatever the plane, so `from` is that centre and
/// `along` is the plane's own normal — every component of it stored exactly, so
/// a ball's answer here is the arithmetic it always was, to the bit.
///
/// A **cone** hands back its apex and
/// [`Cone::which_way_a_closed_crossings_centre_lies`], which carries the
/// argument. A plane square to the axis is the circle and falls out of it as
/// `along = |w|²·n`; a plane leaned off the axis by less than the cone's own
/// half angle cuts an **ellipse**, whose centre is the pole of the plane and not
/// where the axis crosses it.
///
/// **The parabola and the hyperbola are refused**, and that is a limit rather
/// than a gap: a parabola has no centre at all, and a hyperbola's two branches
/// are not one closed curve for a chord to split in two.
///
/// A **pipe** hands back its axis origin and its stored axis direction, turned
/// round where it reads negative against the plane's normal
/// ([`a_pipes_axis_turned_to_cross`]). Every component of that is stored
/// exactly too, as the ball's is.
///
/// [`Cone::which_way_a_closed_crossings_centre_lies`]: rcad_geom::Cone::which_way_a_closed_crossings_centre_lies
pub(crate) fn where_a_closed_crossing_is_centred(
    surface: &Surface,
    flat: &Plane,
) -> Option<WhereTheCurveIsCentred> {
    match surface {
        Surface::Sphere(ball) => Some(WhereTheCurveIsCentred {
            from: ball.centre(),
            along: exactly(flat.normal().components()),
        }),
        Surface::Cone(cone) => Some(WhereTheCurveIsCentred {
            from: cone.apex(),
            along: cone.which_way_a_closed_crossings_centre_lies(flat)?,
        }),
        Surface::Cylinder(pipe) => Some(WhereTheCurveIsCentred {
            from: pipe.axis().origin(),
            along: exactly(a_pipes_axis_turned_to_cross(pipe, flat)?),
        }),
        Surface::Plane(_) | Surface::Torus(_) | _ => None,
    }
}

/// `pipe`'s stored axis direction, turned round if it reads negative against
/// `flat`'s normal; `None` when the plane lies along the axis, and when that
/// could not be settled.
///
/// **Where the axis crosses the plane is the centre** of the curve the plane
/// cuts the pipe in, whether that is a circle or an ellipse. Turning space a
/// half turn about that point — every `p` to `2c − p` — carries the pipe onto
/// itself, as it does about any point of the axis, and carries the plane onto
/// itself because `c` is on it. So it carries the curve onto itself, and a
/// curve that is its own image through `c` is centred at `c`. Walking `along`
/// from the axis origin until the plane is reached lands there, and
/// [`where_the_curves_centre_sits`] wants `n·along` positive, which the turn
/// gives it.
///
/// **`n·a` of zero is refused, and that is a limit**: the plane then holds the
/// axis's direction and cuts the pipe in two lines, one, or none, and none of
/// those closes ([`rcad_geom::Cylinder::is_cut_in_a_closed_curve_by`] reads the
/// same sign). An undecided sign is refused with it, so it costs a refusal and
/// never picks a direction.
fn a_pipes_axis_turned_to_cross(pipe: &rcad_geom::Cylinder, flat: &Plane) -> Option<[f64; 3]> {
    let along = pipe.axis().direction().components();
    let (leans, _) = rcad_predicates::decide(&SquareToTheAxis {
        named: WHERE_A_PLANE_AND_A_PIPE_CROSS,
        normal: flat.normal().components(),
        along,
    });
    match leans {
        Decision::Pos => Some(along),
        Decision::Neg => Some(along.map(core::ops::Neg::neg)),
        Decision::Zero | Decision::Undecidable { .. } => None,
        other => panic!("the axis came back with a decision nothing here knows: {other:?}"),
    }
}

/// Whether the part of `surface` in front of every cut is **one piece** — the
/// thing [`what_a_face_cut_by_part_of_a_crossing_holds`]'s argument rests on.
///
/// **It says nothing about the sides, and it used to claim to.** Until 2026-09-24
/// this line promised "arcs of circles for sides" as a second thing checked here,
/// which no arm of it ever read and which was false of the arms it did have: a
/// leaned plane's crossing with a cone is an *ellipse*, admitted since §211, and
/// a cone face's own sides are conic arcs or straight rulings and never circles.
/// What the caller needs of a side is that [`where_a_side_lies`] can place it
/// against every other cut, and the refusal for a side it cannot place is that
/// function's own [`Lies::Across`], read one side at a time where the side is.
///
/// On a plane, that is half-planes, the insides of balls, and the inside of a
/// cone or a pipe whose crossing with the plane **closes** — each of those
/// leaves a convex piece of the plane, and any number of convex pieces meet in
/// one. A pipe's closed crossing is a circle or an ellipse
/// ([`rcad_geom::Cylinder::is_cut_in_a_closed_curve_by`]), whose inside is
/// convex, so it is the cone's argument word for word. Keeping a pipe's outside
/// is not admitted either: the outside of an ellipse is not convex, which is
/// the first of the cone's reasons below.
///
/// What this needs of the cone is that the region be one piece, which is
/// [`Cone::is_cut_in_a_closed_curve_by`]'s question and not the narrower circle.
/// A circle's inside is a disc and an ellipse's inside is an ellipse's inside;
/// both are convex, so both leave one piece and
/// [`what_a_face_cut_by_part_of_a_crossing_holds`]'s argument is the same
/// argument word for word with "disc" read as "convex region". Neither the
/// parabola nor the hyperbola is admitted: the parabola's inside is unbounded,
/// and the hyperbola's is two pieces with nothing between them, which is the one
/// thing this test exists to refuse. `Some(false)` and `None` are both refused,
/// so an undecided sign costs a refusal and never a wrong answer.
///
/// **A face keeping the cone's *outside* is refused, and that is a limit and not
/// a conjunct in the way.** `cut.kept == Decision::Neg` is what says so, and
/// deleting it moves nothing. Read this before reaching for it again
/// (`docs/decisions.md` §215.3 asked, and this is the answer):
///
/// - The argument above does not carry. The inside of an ellipse is convex; the
///   **outside** of one is not, so "any number of convex pieces meet in one" says
///   nothing about the region left over, and that region really can be two
///   pieces. `a_slab_the_cones_circle_splits_in_two_is_never_answered_wrong`, in
///   `tests/a_plane_cut_along_a_cones_other_nappe.rs`, builds one through the
///   public API: a slab 2 wide and 12 long across a slope-one-half cone, whose
///   top face at `z = 6` the cone's radius-3 circle runs into at one long side
///   and out at the other, leaving the part of that face outside the cone in two
///   pieces with the circle between them. Admitting it here would let
///   [`held_in_front_of_all`] call a point in the *other* piece a point on this
///   face, which is a wrong answer and not a refusal.
/// - Widening it would also move no number, because two more things sit behind
///   it. [`where_a_side_lies`] reads a straight side against a cone only as
///   [`Lies::Beside`]`(`[`Decision::Neg`]`)` — and it is right to: a segment
///   between two points *outside* a convex region can run straight through it,
///   where one between two points inside cannot. And the ellipse such a face is
///   ringed by is **closed**, so neither of its ends is on any of the straight
///   cuts, which is [`where_a_side_lies`]'s `[..] => Lies::Across` arm.
/// - Measured, on the `Union` answer of the side-6 corner-up cube centred on the
///   pointed cone's apex and turned `0.7`
///   (`a_corner_up_cube_round_a_pointed_cones_apex_answers_right`'s case, whose
///   9 326 decided grid points are the floor §215.3 named): deleting the
///   conjunct leaves 9 326, and deleting it *and* letting a straight side read
///   `Beside(Pos)` against a cone leaves 9 326 again, with all 13 187 refusals
///   moving from this gate to that `Lies::Across`.
///
/// So the family of fixes is "loosen what this admits", and it is the only one
/// that keeps the shape of [`what_a_face_cut_by_part_of_a_crossing_holds`] — it
/// asks one question, "is `point` in front of every cut", and a region in two
/// pieces has no such answer. Anything that does follow a kept outside has to
/// decide whether the leftover region is connected, which is a predicate nobody
/// here has, over a shape nobody here builds: where the ellipse meets the
/// straight cuts. That is a part with no named consumer (§13), and until one
/// appears this refuses.
///
/// On a pipe it is planes, as [`leaves_a_pipe_in_one_piece`] sets out.
///
/// On a sphere, it is planes through the centre: each cuts the sphere into two
/// halves, and any number of halves meet in one piece, where planes that miss
/// the centre can leave two caps with nothing between them.
///
/// # On a cone it is planes the apex is in front of, any number of them, and at most one it is on
///
/// Every point of one nappe but the apex sits on exactly one ray from the apex —
/// a ruling — and the whole straight segment from the apex out to that point is
/// on the same ray, so it never leaves the nappe. A plane's signed value along
/// such a ray is affine in how far along it you have gone, so "in front of this
/// cut" is a half-interval of the ray, and "in front of every cut" is the
/// intersection of those: **one interval, however many cuts there are.** A ruling
/// can never leave the region and come back, which is the strip
/// `docs/decisions.md` §217.4 worried three planes could leave, and it is settled
/// for three the same way it is for thirty.
///
/// Ask on top of that that the **apex** be on or in front of every cut, and every
/// one of those intervals starts at the apex. Then the region holds, with each of
/// its points, the whole segment back to the apex — it is star-shaped from the
/// apex, so it is one piece. Nothing in that mentions how the cut planes sit
/// against each other, which is why this arm counts planes and reads one sign per
/// cut rather than looking for where two of them meet. §216.5's sketch settled a
/// cone cut by exactly **two** planes by walking the circle of rulings; that
/// sketch is not needed, and it was not enough, because §217.4 counted every
/// cone-face refusal in `tests/a_plane_cut_along_a_cones_other_nappe.rs` and
/// found three planes every time and two not once.
///
/// **One nappe is what the argument is about, and that is not the same claim as
/// connectedness.** Both nappes' regions contain the apex, so their union is
/// connected too and this test would pass on it. What fails on the union is the
/// caller's other half — that the *face* is the whole of the part in front of
/// every cut. A cone face lies on one nappe; with the apex strictly in front of
/// every cut the region on the double cone is a neighbourhood of the apex on
/// **both**, two sheets pinched at one point, and the face is one of them. So
/// [`what_a_face_cut_by_part_of_a_crossing_holds`] puts
/// [`is_on_the_nappe_the_face_is_on`] in front of all of this, and deleting that
/// guard costs a wrong answer rather than a refusal: on the side-6 corner-up cube
/// round a pointed cone's apex, `(0.5, 0, −1)` and `(1, 0, −2)` come back
/// **held**. `a_cone_face_round_an_apex_holds_its_own_nappe_and_not_the_other`, in
/// `tests/a_plane_cut_along_a_cones_other_nappe.rs`, is those two points through
/// the public [`SolidView::face_holds_the_point`].
///
/// **The same pinch happens on one nappe, and it is why at most one cut may
/// hold the apex.** A plane through the apex has the value zero there, so along
/// each ruling it keeps all of it or only the apex, and which is the sign of how
/// fast it grows along that ruling — affine in `cos φ` and `sin φ`, so one arc of
/// the circle of rulings. One such arc is one piece. Two can meet in two: keep
/// `|x| ≤ z/10` on the cone of slope a half and the rulings with
/// `|cos φ| ≤ 1/5` are left, one run round `+y` and one round `−y`, two sheets
/// that meet only at the apex. A face on one of them is cut by both planes, has
/// every side in front of both, and the other sheet is in front of both too, so
/// `held_in_front_of_all` would hold its points — a wrong answer.
/// `two_planes_through_the_apex_leave_a_nappe_in_two_sheets` below walks those
/// rulings. Before `docs/decisions.md` §389 a cut through the apex came only
/// with a side stored as [`EdgeRun::Straight`] along a ruling, since a ruling
/// stored as part of a crossing was refused as an arc; since it, every ruling
/// side reaches this arm.
///
/// **That refuses some faces that are one piece, and it is a gap.** Two planes
/// through the apex meeting along a line inside the cone keep one arc: every
/// half-plane round that line meets the nappe in one ruling, so a wedge round it
/// holds one run. A quarter of a pointed cone, cut by `x = 0` and `y = 0` along
/// its axis, is that, and its skin is refused here
/// (`a_quarter_of_a_pointed_cone_has_its_skin_refused_by_name_though_it_is_one_piece`
/// in `tests/a_solid_carrying_a_run_along_a_cones_ruling.rs`, and the quarter's
/// cuts in `two_planes_through_the_apex_leave_a_nappe_in_two_sheets` below).
/// It was refused before §389 too, measured, so this took nothing away.
/// Telling it from two sheets wants where the two planes meet compared with the
/// cone (`docs/decisions.md` §389).
///
/// # And one cut the apex is behind, when it closes, beside at most one other
///
/// The lens of `tests/a_plane_face_cut_by_a_cone_it_crosses_in_an_ellipse.rs` is
/// a cone face between two lids with the apex behind the lower one, and the
/// argument above does not reach it. What does: let the cut the apex is behind
/// keep `g + t·a(φ) ≥ 0` along the ruling at `φ`, with `g < 0` its value at the
/// apex and `a(φ)` how fast it grows. That the plane cuts the cone in a **closed
/// curve** — §208.6's sign, [`Cone::is_cut_in_a_closed_curve_by`] — is exactly
/// that no ruling runs along it, so `a(φ)` never changes sign on a nappe, and it
/// is positive on the nappe the curve is on, which is the face's own, since a
/// side of the face runs along that curve. So every ruling of the face's nappe
/// crosses the cut once, at `α(φ) = −g/a(φ)`, and keeps everything past it.
///
/// With **no other cut**, every ruling keeps its whole half-line past `α`, and
/// the region is a band round the cone — one piece. With **one other cut**,
/// `h + t·b(φ) ≥ 0` with `h ≥ 0`, a ruling carries a point exactly when `α(φ)`
/// satisfies it, which, multiplied through by `a(φ) > 0`, is
/// `h·a(φ) − g·b(φ) ≥ 0`. Both `a` and `b` are affine in `cos φ` and `sin φ`, so
/// that is one arc of the circle of rulings, or all of it, or none; over it the
/// points `α(φ)` run along one unbroken curve, and each ruling's piece hangs off
/// that curve. One piece.
///
/// **A cut side by side with the one the apex is behind costs nothing**, however
/// many there are: its normal lies along that cut's
/// ([`the_normals_lie_along_one_line`]), so as functions of space it is `μ` times
/// that cut plus a constant, and `b(φ) = μ·a(φ)`. Then `h·a(φ) − g·b(φ)` is
/// `a(φ)·(h − g·μ)`, whose sign does not depend on `φ` at all: every ruling
/// carries a point or none does. What is left is the arc of the one other cut,
/// and each ruling's piece runs from `α(φ)` to the nearest of the others, so it
/// still hangs off the one curve. A half frustum's skin is that: the floor the
/// apex is behind, the roof square to the axis above it, and the plane down the
/// axis that cut it in half (`docs/decisions.md` §389).
///
/// **Two or more others not side by side with it is where it stops, and that is
/// a limit even when every cut closes.** Each such cut gives its own arc and two
/// arcs can meet in two:
/// `one_plane_behind_the_apex_and_two_in_front_leave_a_nappe_in_two_pieces`
/// below is a floor that cuts a circle and two roofs that each cut an ellipse,
/// whose region is two pieces round `±90°` with nothing along `±x`. Telling that
/// apart from a three-cut region that is one piece wants the arcs' ends compared,
/// which is where two roofs meet on the cone — a part with no named consumer
/// (§13); the lens, the one face that asked, has two cuts.
///
/// **Two cuts the apex is behind are refused, and that is a limit and not a
/// gap.** There the region really can be two pieces. On the cone of slope one
/// about `+z`, keep `x + z/√2 ≥ 1` and `−x + z/√2 ≥ 1`; the apex is a whole unit
/// behind each. Along the ruling at angle `φ` the first grows at
/// `cos φ + 1/√2` and the second at `−cos φ + 1/√2`, and a cut the apex is behind
/// is reachable along a ruling only where its own rate is positive — so the first
/// is reachable only for `|φ| < 135°` and the second only for `45° < φ < 315°`.
/// Those two arcs of the circle of rulings overlap in **two** arcs,
/// `45° < φ < 135°` and `225° < φ < 315°`, and every ruling between them — `φ = 0`
/// and `φ = 180°` among them — holds no point in front of both. Two pieces with
/// nothing between them, which is the one thing this test exists to refuse.
/// `two_planes_the_apex_is_behind_leave_a_nappe_in_two_pieces` below walks those
/// rulings and asks this predicate about the same two cuts, so the refusal has a
/// shape behind it and not a paragraph (§16.10). Widening to reach it would need a
/// predicate for whether the leftover is connected, over the meets of the cut
/// planes on the cone — a part with no named consumer (§13).
///
/// [`Cone::is_cut_in_a_closed_curve_by`]: rcad_geom::Cone::is_cut_in_a_closed_curve_by
fn is_a_family_this_follows(surface: &Surface, cuts: &[Cut]) -> bool {
    match surface {
        Surface::Plane(flat) if is_a_pair_of_pipes_running_the_same_way(flat, cuts) => {
            cuts.iter().any(|cut| cut.kept == Decision::Neg)
        }
        Surface::Plane(flat) => cuts.iter().all(|cut| match cut.surface {
            Surface::Plane(_) => true,
            Surface::Sphere(_) => cut.kept == Decision::Neg,
            Surface::Cone(cone) => {
                cut.kept == Decision::Neg && cone.is_cut_in_a_closed_curve_by(flat) == Some(true)
            }
            Surface::Cylinder(pipe) => {
                cut.kept == Decision::Neg && pipe.is_cut_in_a_closed_curve_by(flat) == Some(true)
            }
            Surface::Torus(_) | _ => false,
        }),
        Surface::Sphere(ball) => {
            let centre = PredicatePoint::from(ball.centre().coordinates());
            cuts.iter().all(|cut| {
                cut.surface
                    .as_plane()
                    .is_some_and(|flat| flat.side_of_point(centre) == Decision::Zero)
            })
        }
        Surface::Cone(cone) => {
            let apex = PredicatePoint::from(cone.apex().coordinates());
            let mut behind = Vec::new();
            let mut through_the_apex = 0;
            for cut in cuts {
                let Some(flat) = cut.surface.as_plane() else {
                    return false;
                };
                let sits = flat.side_of_point(apex);
                if sits == Decision::Zero {
                    through_the_apex += 1;
                    continue;
                }
                if sits == cut.kept {
                    continue;
                }
                if sits != Decision::Pos && sits != Decision::Neg {
                    return false;
                }
                behind.push(flat);
            }
            if through_the_apex > 1 {
                return false;
            }
            match behind.as_slice() {
                [] => true,
                [flat] => {
                    let not_side_by_side = cuts
                        .iter()
                        .filter_map(|cut| cut.surface.as_plane())
                        .filter(|other| !the_normals_lie_along_one_line(flat, other))
                        .count();
                    not_side_by_side <= 1 && cone.is_cut_in_a_closed_curve_by(flat) == Some(true)
                }
                [_, _, ..] => false,
            }
        }
        Surface::Cylinder(pipe) => leaves_a_pipe_in_one_piece(pipe, cuts),
        Surface::Torus(_) | _ => false,
    }
}

/// Whether `cuts` are exactly two pipes whose axes run the same way, each cut
/// by `flat` in a curve that closes: the two cap circles of two bars side by
/// side, on a cap of anything fused from them (`docs/decisions.md` §331.5).
///
/// **Two such curves are one curve moved and scaled.** A pipe's sign is
/// `|w|²|x − o|² − (w·(x − o))² − r²|w|²`, and its part of degree two,
/// `|w|²|x|² − (w·x)²`, changes only by a positive factor between two pipes
/// whose axes run the same way. So one sign less the other, scaled, is a
/// plane's sign, and three things follow on `flat`:
///
/// - **The two curves meet in at most two points**, because those are on both
///   curves and on that plane, and a line meets a closed curve in at most two
///   points — or they are the same curve, which reads as a touch
///   ([`where_an_arc_between_two_points_of_a_pipe_lies`]) and is refused.
/// - **Some affine map of `flat` makes both circles**, the one that makes the
///   first a circle. Being inside, outside, one piece and a chord are all kept
///   by such a map, so what follows is argued on circles.
/// - **The part of `flat` in front of both cuts is one piece** when the face
///   keeps the inside of at least one, which is what
///   [`is_a_family_this_follows`] asks. Inside both is two discs meeting, which
///   is convex. Inside one disc and outside the other: walk from any of its
///   points straight away from the second disc's centre. The walk stays outside
///   the second disc, since its distance from that centre only grows, and stays
///   in the first until it crosses the first circle — at a point outside the
///   second disc, so on the one arc of the first circle outside the second.
///   Every point is joined to that one arc, so the whole is one piece.
///
/// Outside both is refused. It is not a region a closed solid's face can be,
/// since it runs on for ever.
fn is_a_pair_of_pipes_running_the_same_way(flat: &Plane, cuts: &[Cut]) -> bool {
    let [one, other] = cuts else {
        return false;
    };
    are_pipes_running_the_same_way(&one.surface, &other.surface)
        && [one, other].iter().all(|cut| match cut.surface {
            Surface::Cylinder(pipe) => pipe.is_cut_in_a_closed_curve_by(flat) == Some(true),
            Surface::Plane(_) | Surface::Sphere(_) | Surface::Cone(_) | Surface::Torus(_) | _ => {
                false
            }
        })
}

/// Whether the part of `pipe` in front of every cut is one piece, for the cuts
/// this can show it of: planes, at most one of them lying along the axis, at
/// most one keeping the pipe's far end and at most one keeping its near end —
/// and with all three kinds present, those last two side by side.
///
/// **Walk the pipe along its rulings**, the straight lines up its wall, one for
/// each angle `φ` round the axis. A plane lying along the axis has the same
/// value all the way up a ruling, and that value is affine in `cos φ` and
/// `sin φ`, so it keeps one arc of the circle of rulings, or all of it, or none.
/// A plane across the axis grows at the one rate `n·a` up every ruling, so it
/// keeps each ruling from a point `α(φ)` on: up the axis if `n·a` has the sign
/// of the side it keeps (a floor), and back down if not (a roof). `α(φ)` is
/// again affine in `cos φ` and `sin φ`.
///
/// - With **at most a floor**, every ruling the cut along the axis leaves holds
///   a half-line, whose start moves smoothly with `φ`: one piece. A roof alone is
///   the same turned round.
/// - With **a floor and a roof**, a ruling holds `[α_floor, α_roof]`, which is
///   there where `α_roof − α_floor ≥ 0`. That difference is affine in `cos φ`
///   and `sin φ` too, so it holds on one arc, or all, or none. Alone, that is one
///   piece.
/// - With **both and a cut along the axis** there are two arcs, and two arcs of
///   a circle can meet in two pieces: on the pipe of radius one up `z`, keeping
///   `x ≥ −½`, `z ≥ 0` and `z ≤ ½ − x` leaves the rulings with `|x| ≤ ½`, one
///   run round `+y` and one round `−y`. So the floor and the roof are asked to be
///   **side by side**, their normals along one line
///   ([`the_normals_lie_along_one_line`]); then `α_roof − α_floor` does not
///   depend on `φ` at all, and the first arc is all or nothing. A half bar's two
///   caps are that.
///
/// **A second pipe running the same way is a cut along the axis too**, which is
/// what a wall of two bars side by side, fused, is cut by. On this pipe the
/// other one's sign is a plane's sign, the plane through the two lines the
/// pipes cross in ([`is_a_pair_of_pipes_running_the_same_way`]), and a pipe's
/// sign does not change up its own axis, so neither does this: it is the same
/// all the way up a ruling and affine in `cos φ` and `sin φ`, as a plane along
/// the axis is. Whichever side of it the face keeps, it keeps one arc of
/// rulings.
///
/// Anything else is refused, and none of it is shown to be a limit. Two cuts
/// along the axis or two floors are each two arcs, which can meet in two pieces
/// as above (a slab `|x| ≤ ½` across the same pipe), and telling that from one
/// piece wants the arcs' ends compared; a floor and a roof that are not side by
/// side beside a cut along the axis could be settled the same way. No face here
/// has asked for either (§13). A cut that is not a plane, or a sign that could
/// not be settled, is refused too, so an undecided sign costs a refusal and
/// never a wrong answer.
fn leaves_a_pipe_in_one_piece(pipe: &rcad_geom::Cylinder, cuts: &[Cut]) -> bool {
    let axis = pipe.axis().direction().components();
    let mut along_the_axis = 0;
    let mut floors = Vec::new();
    let mut roofs = Vec::new();
    for cut in cuts {
        if let Surface::Cylinder(other) = cut.surface
            && lie_along_one_line(axis, other.axis().direction().components())
        {
            along_the_axis += 1;
            continue;
        }
        let Some(flat) = cut.surface.as_plane() else {
            return false;
        };
        let (leans, _) = rcad_predicates::decide(&SquareToTheAxis {
            named: WHERE_A_PLANE_AND_A_PIPE_CROSS,
            normal: flat.normal().components(),
            along: axis,
        });
        match leans {
            Decision::Zero => along_the_axis += 1,
            Decision::Pos | Decision::Neg if leans == cut.kept => floors.push(flat),
            Decision::Pos | Decision::Neg => roofs.push(flat),
            Decision::Undecidable { .. } => return false,
            other => panic!("the axis came back with a decision nothing here knows: {other:?}"),
        }
    }
    match (along_the_axis, floors.as_slice(), roofs.as_slice()) {
        (0 | 1, [] | [_], []) | (0 | 1, [], [_]) | (0, [_], [_]) => true,
        (1, [floor], [roof]) => the_normals_lie_along_one_line(floor, roof),
        _ => false,
    }
}

/// Whether two planes' stored normals lie along one line ([`lie_along_one_line`]).
fn the_normals_lie_along_one_line(one: &Plane, other: &Plane) -> bool {
    lie_along_one_line(one.normal().components(), other.normal().components())
}

/// Whether two pipes' stored axis directions lie along one line
/// ([`lie_along_one_line`]); `false` when either is not a pipe.
fn are_pipes_running_the_same_way(one: &Surface, other: &Surface) -> bool {
    let (Surface::Cylinder(one), Surface::Cylinder(other)) = (one, other) else {
        return false;
    };
    lie_along_one_line(
        one.axis().direction().components(),
        other.axis().direction().components(),
    )
}

/// Whether `u` and `v` lie along one line, which is every component of `u × v`
/// exactly zero; `false` when any of the three could not be settled.
///
/// Each component is `u_i·v_j − u_j·v_i`, the dot of `(u_i, −u_j)` with
/// `(v_j, v_i)`, and turning a stored number round is exact, so each is the
/// same sum of stored products [`SquareToTheAxis`] settles.
fn lie_along_one_line(u: [f64; 3], v: [f64; 3]) -> bool {
    [(1, 2), (2, 0), (0, 1)].into_iter().all(|(i, j)| {
        let (component, _) = rcad_predicates::decide(&SquareToTheAxis {
            named: WHETHER_TWO_DIRECTIONS_LIE_ALONG_ONE_LINE,
            normal: [u[i], -u[j], 0.0],
            along: [v[j], v[i], 0.0],
        });
        component == Decision::Zero
    })
}

/// The name the test of whether two planes' normals, or two pipes' axes, lie
/// along one line goes by in provenance records.
const WHETHER_TWO_DIRECTIONS_LIE_ALONG_ONE_LINE: &str = "whether two directions lie along one line";

/// Which side of the surface across `side` the face keeps, read off the way the
/// wire runs.
///
/// A wire keeps its face's material on its left, and left of `∇S × ∇T` is always
/// `T`'s negative side (§36.6, `a_closed_crossing_is_walked_the_way_it_is_stored`).
/// So the face keeps `T`'s negative side exactly when the side runs along
/// `∇S × ∇T` at its start. A straight side runs from its start to its end. An
/// arc leaves either of its ends into itself, and into itself reads positive
/// against [`the_way_into_an_arc`], so the sign of that against `∇S × ∇T` is the
/// sign of the way it runs.
///
/// **`∇S` is the stored surface's own gradient, whichever way the face is
/// used**, because that is the normal the wire, as a view hands it, keeps the
/// material on the left of. The arena has the two coedges of every edge run
/// opposite ways (`Model::validate`), so a face used reversed is stored with its
/// wire walked round its *used* normal — `fuse/assemble.rs`'s `sides_of` writes
/// a curved piece that way — and [`WireView::coedges`] walks it back the other
/// way round. Turning `∇S` round for a reversed face as well, as this did until
/// `docs/decisions.md` §331.5's note of 2026-09-29, read every cut of the
/// crescent's inner wall as keeping the far side, and that wall refused.
///
/// **A straight side on a curved face is read the same way as one between two
/// planes**, which is what a half bar's wall needs: its two sides along `x = 0`
/// are rulings of the pipe. The whole segment lies on both surfaces, so it is
/// part of the curve they cross in, and that curve's direction at the start is
/// along the segment. The one way that fails is `∇S × ∇T` being nothing there,
/// the plane touching the pipe along the ruling rather than crossing it, and
/// that reads [`Decision::Zero`] and is refused.
///
/// # Errors
///
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] for an arc between two
/// planes, which is not a side of this family, or a side along which the two
/// surfaces touch, and [`Error::Undecidable`] when the sign could not be
/// settled.
fn which_side_of_its_neighbour_a_side_keeps(
    face: FaceView<'_>,
    surface: &Surface,
    side: &Side,
) -> Result<Decision> {
    let cannot = || Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() };
    let at = side.start.at;
    let along_the_crossing =
        the_way_the_crossing_runs(surface, &side.across, at).ok_or_else(cannot)?;
    let flat = [surface, &side.across].map(|one| one.as_plane().is_some());
    let runs = match &side.run {
        SideRun::Straight => {
            let to = side.end.at.coordinates();
            let from = at.coordinates();
            let step = core::array::from_fn(|axis| {
                Interval::exactly(to[axis]) - Interval::exactly(from[axis])
            });
            dot(step, along_the_crossing)
        }
        SideRun::Arc { into_the_arc } if flat == [true, false] || flat == [false, true] => {
            dot(*into_the_arc, along_the_crossing)
        }
        SideRun::Arc { .. } => return Err(cannot()),
    };
    match runs.sign() {
        Decision::Pos => Ok(Decision::Neg),
        Decision::Neg => Ok(Decision::Pos),
        Decision::Zero => Err(cannot()),
        Decision::Undecidable { enclosure } => Err(gave_up(
            WHICH_SIDE_A_SIDE_KEEPS,
            PredicatePoint::from(at.coordinates()),
            Level::Interval,
            enclosure,
            Measures::Volume,
        )),
        other => panic!("a sign came back with a decision nothing here knows: {other:?}"),
    }
}

/// The name reading which side of its neighbour a side keeps goes by in
/// provenance records.
const WHICH_SIDE_A_SIDE_KEEPS: &str = "which_side_a_side_keeps";

/// Where the inside of `side` lies against `cut`, one of the other surfaces
/// cutting the face.
///
/// **A corner of an edge along `cut` is on `cut` by topology**, and is read as
/// on it rather than asked — §35.2's rule, for §19.1's reason.
///
/// - A straight side against a plane lies where its ends do.
/// - A straight side against a ball it has both ends in or on is inside it,
///   because a ball holds every segment between two of its points.
/// - A straight side of a **flat** face against a cone or a pipe it crosses in a
///   curve that closes, both ends in or on it, is inside it too — not because the
///   cone is convex, which a double cone is not, but because what either leaves
///   on such a plane is one convex region, and a convex region holds every
///   segment between two of its points. A closed crossing is what
///   [`is_a_family_this_follows`] has already asked of every cone and pipe here.
/// - An arc against a plane that holds both its ends: that plane meets the arc's
///   own plane in the line through the two ends, which is the chord, and the arc
///   is the whole of the side of the chord [`the_way_into_an_arc`] points to. So
///   the arc is in front of the cut exactly when the cut's normal points that
///   way too, which is the sign of `(n × p) · (h × p)` — the cut's and the arc's
///   in-plane directions compared.
/// - An arc against a plane that holds **one** of its ends, which is the pie
///   slice: [`where_an_arc_with_one_end_on_a_cut_lies`].
/// - An arc against a plane side by side with the arc's own plane
///   ([`the_normals_lie_along_one_line`]), both ends on the same side of it: the
///   two planes never meet, so the whole of the arc's plane is on that side. A
///   half bar's wall is cut by its two caps, and each cap's arc is this against
///   the other cap.
///
/// Everything else is [`Lies::Across`]: an arc with neither end on the plane may
/// bulge across it, and nothing here builds the point where it would.
fn where_a_side_lies(surface: &Surface, side: &Side, cut: &Cut) -> Lies {
    let side_of_an_end = |end: Corner| {
        if cut.corners_on_it.contains(&end.key) {
            Decision::Zero
        } else {
            side_of_surface(&cut.surface, PredicatePoint::from(end.at.coordinates()))
        }
    };
    let ends = [side_of_an_end(side.start), side_of_an_end(side.end)];
    let settled = ends
        .iter()
        .all(|end| matches!(end, Decision::Zero | Decision::Pos | Decision::Neg));
    if !settled {
        return Lies::Across;
    }
    match &side.run {
        SideRun::Straight => match cut.surface {
            Surface::Plane(_) => {
                let beside: Vec<Decision> = ends
                    .into_iter()
                    .filter(|end| *end != Decision::Zero)
                    .collect();
                match beside.as_slice() {
                    [] => Lies::On,
                    [one] => Lies::Beside(*one),
                    [one, other] if one == other => Lies::Beside(*one),
                    [..] => Lies::Across,
                }
            }
            Surface::Sphere(_) if ends.iter().all(|end| *end != Decision::Pos) => {
                Lies::Beside(Decision::Neg)
            }
            Surface::Cone(cone)
                if surface
                    .as_plane()
                    .is_some_and(|own| cone.is_cut_in_a_closed_curve_by(own) == Some(true))
                    && ends.iter().all(|end| *end != Decision::Pos) =>
            {
                Lies::Beside(Decision::Neg)
            }
            Surface::Cylinder(pipe)
                if surface
                    .as_plane()
                    .is_some_and(|own| pipe.is_cut_in_a_closed_curve_by(own) == Some(true))
                    && ends.iter().all(|end| *end != Decision::Pos) =>
            {
                Lies::Beside(Decision::Neg)
            }
            Surface::Sphere(_)
            | Surface::Cylinder(_)
            | Surface::Cone(_)
            | Surface::Torus(_)
            | _ => Lies::Across,
        },
        SideRun::Arc { into_the_arc } => {
            let Surface::Plane(flat) = cut.surface else {
                let arcs_own_pipe = match (surface.as_plane(), side.across.as_plane()) {
                    (Some(_), None) => Some(&side.across),
                    (None, Some(_)) => Some(surface),
                    (Some(_), Some(_)) | (None, None) => None,
                };
                let on_a_pipe_running_the_same_way = arcs_own_pipe
                    .is_some_and(|own| are_pipes_running_the_same_way(own, &cut.surface));
                return if on_a_pipe_running_the_same_way && ends == [Decision::Zero, Decision::Zero]
                {
                    where_an_arc_between_two_points_of_a_pipe_lies(
                        surface,
                        side,
                        *into_the_arc,
                        &cut.surface,
                    )
                } else {
                    Lies::Across
                };
            };
            let on_the_cut = |end: Corner, far| {
                where_an_arc_with_one_end_on_a_cut_lies(
                    surface,
                    side,
                    *into_the_arc,
                    &flat,
                    end.at,
                    far,
                )
            };
            match ends {
                [Decision::Zero, Decision::Zero] => {
                    let Some(own_plane) = surface.as_plane().or(side.across.as_plane()) else {
                        return Lies::Across;
                    };
                    let normal = exactly(own_plane.normal().components());
                    let cut_line = cross(exactly(flat.normal().components()), normal);
                    let arc_line = cross(*into_the_arc, normal);
                    match dot(cut_line, arc_line).sign() {
                        found @ (Decision::Pos | Decision::Neg) => Lies::Beside(found),
                        Decision::Zero | Decision::Undecidable { .. } | _ => Lies::Across,
                    }
                }
                [Decision::Zero, far @ (Decision::Pos | Decision::Neg)] => {
                    on_the_cut(side.start, far)
                }
                [far @ (Decision::Pos | Decision::Neg), Decision::Zero] => {
                    on_the_cut(side.end, far)
                }
                [one @ (Decision::Pos | Decision::Neg), other]
                    if one == other
                        && surface
                            .as_plane()
                            .or(side.across.as_plane())
                            .is_some_and(|own| the_normals_lie_along_one_line(own, &flat)) =>
                {
                    Lies::Beside(one)
                }
                [..] => Lies::Across,
            }
        }
    }
}

/// Where an arc lies against a plane cutting the face, when one end of the arc
/// is on that plane and the other is not — which is what a **pie slice** asks
/// and a half-disc does not (`docs/decisions.md` §169).
///
/// The arc runs on a circle, and the cutting plane meets that circle in the end
/// it holds and in **one** other point. So the side of the plane the arc is on
/// changes nowhere but there: it leaves the end it holds on one side, and it is
/// still on that side at its far end unless it ran past the other point on the
/// way. The far end's own side says whether it did, so how far the arc runs is
/// answered without building a single point on it — which is what §168.4 left
/// open.
///
/// **Which way it leaves** is `∇S × ∇T`, the way the crossing runs, turned round
/// to point into the arc: the arc leaves either of its ends into itself, so into
/// the arc is whichever way reads positive against [`the_way_into_an_arc`]. The
/// side of the cut it leaves on is then the sign of
/// `(∇S × ∇T · into_the_arc)(∇S × ∇T · cut)`, and turning `∇S × ∇T` round turns
/// both factors round, so which of the arc's two ends this is does not matter.
///
/// [`Lies::Across`] when the two disagree, which is an arc that really does
/// cross the cut, and when either sign is [`Decision::Zero`] or could not be
/// settled. The two `Zero` readings are grazes — the cut touching the circle at
/// this end rather than crossing it, or the tangent there lying along the chord
/// — and a graze is a side that stays where it starts, so those are answers this
/// refuses rather than answers it gets wrong. A gap, left for the face that
/// needs it.
fn where_an_arc_with_one_end_on_a_cut_lies(
    surface: &Surface,
    side: &Side,
    into_the_arc: [Interval; 3],
    cut: &Plane,
    at: Point,
    far: Decision,
) -> Lies {
    let Some(along) = the_way_the_crossing_runs(surface, &side.across, at) else {
        return Lies::Across;
    };
    let leaving = dot(along, into_the_arc) * dot(along, exactly(cut.normal().components()));
    if leaving.sign() == far {
        Lies::Beside(far)
    } else {
        Lies::Across
    }
}

/// Where an arc of the curve a plane cuts one pipe in lies against another pipe
/// running the same way, when both of the arc's ends are on that pipe: on the
/// side the arc leaves its start into. The arc may be a side of the flat face or
/// of the pipe's own wall.
///
/// **It stays on that side all the way.** The arc lies on the plane, so it can
/// meet the cut only where the plane cuts the cut's pipe, and the two curves the
/// plane cuts two pipes running the same way in meet in at most two points
/// ([`is_a_pair_of_pipes_running_the_same_way`]). The arc's two ends are two of
/// them, so nothing between its ends is on the cut.
///
/// Which side it leaves into is the way the crossing runs at the start, turned
/// round to point into the arc as [`where_an_arc_with_one_end_on_a_cut_lies`]
/// turns it, read against the cut's own gradient there, which points out of
/// the pipe.
///
/// [`Lies::Across`] when that reads zero or could not be settled. Zero is the
/// arc leaving along the cut's curve — the two touching at the start, or being
/// one curve — and neither is a side the arc lies on.
fn where_an_arc_between_two_points_of_a_pipe_lies(
    surface: &Surface,
    side: &Side,
    into_the_arc: [Interval; 3],
    cut: &Surface,
) -> Lies {
    let at = side.start.at;
    let (Some(along), Some(out_of_the_cut)) = (
        the_way_the_crossing_runs(surface, &side.across, at),
        gradient_at(cut, at),
    ) else {
        return Lies::Across;
    };
    match (dot(along, into_the_arc) * dot(along, out_of_the_cut)).sign() {
        found @ (Decision::Pos | Decision::Neg) => Lies::Beside(found),
        Decision::Zero | Decision::Undecidable { .. } | _ => Lies::Across,
    }
}

/// Which way the curve two surfaces cross in runs where it passes `at`, as
/// `∇S × ∇T`; `None` for a surface [`gradient_at`] does not read.
fn the_way_the_crossing_runs(
    surface: &Surface,
    across: &Surface,
    at: Point,
) -> Option<[Interval; 3]> {
    Some(cross(gradient_at(surface, at)?, gradient_at(across, at)?))
}

/// The face is the part of its surface in front of every cut.
fn held_in_front_of_all(cuts: &[Cut], point: PredicatePoint) -> Result<OnAFace> {
    let mut on_its_boundary = false;
    for cut in cuts {
        match side_of_surface(&cut.placed, point) {
            Decision::Zero => on_its_boundary = true,
            settled if settled == cut.kept => (),
            Decision::Pos | Decision::Neg => return Ok(OnAFace::NotHeld),
            Decision::Undecidable { enclosure } => {
                return Err(gave_up_on_a_side(&cut.placed, point, enclosure));
            }
            other => panic!("a side came back with a decision nothing here knows: {other:?}"),
        }
    }
    Ok(if on_its_boundary {
        OnAFace::OnItsBoundary
    } else {
        OnAFace::Held
    })
}

/// The face is the part of its sphere in front of any cut: the rest of the
/// sphere is behind all of them.
fn held_in_front_of_any(cuts: &[Cut], point: PredicatePoint) -> Result<OnAFace> {
    let mut on_its_boundary = false;
    let mut undecided = None;
    for cut in cuts {
        match side_of_surface(&cut.placed, point) {
            settled if settled == cut.kept => return Ok(OnAFace::Held),
            Decision::Zero => on_its_boundary = true,
            Decision::Pos | Decision::Neg => (),
            Decision::Undecidable { enclosure } => {
                undecided = Some(gave_up_on_a_side(&cut.placed, point, enclosure));
            }
            other => panic!("a side came back with a decision nothing here knows: {other:?}"),
        }
    }
    if let Some(refusal) = undecided {
        return Err(refusal);
    }
    Ok(if on_its_boundary {
        OnAFace::OnItsBoundary
    } else {
        OnAFace::NotHeld
    })
}

/// The gradient of a plane, a ball, a cone or a pipe at `at`, as a box, as the
/// surface is stored; `None` for any other kind.
///
/// Only the sign of a dot against this is ever read, so a positive factor is
/// dropped where one falls out. A cone's own sign is
/// `|x|²|w|² − (1 + s²)(x·w)²` for `x = at − apex` and `w` the stored axis
/// direction, whose gradient is twice `|w|²x − (1 + s²)(x·w)w`; the two is what
/// is dropped. It points out of the cone, the way a plane's normal and a ball's
/// outward step do, because the sign grows outward.
///
/// It is zero only at the apex: away from it, `|w|²x = (1 + s²)(x·w)w` puts `x`
/// along `w`, and then `s²` times a non-zero number would have to be zero. No
/// corner of an arc this reads is the apex, because an arc has a circle of its
/// own with two different ends on it.
///
/// A pipe's sign is `|x|²|w|² − (x·w)² − r²|w|²` for `x = at − origin`, whose
/// gradient is twice `|w|²x − (x·w)w`: the part of `x` square to the axis,
/// scaled by `|w|²`. It points out of the pipe and is zero only on the axis,
/// which no point of the pipe's skin is on.
fn gradient_at(surface: &Surface, at: Point) -> Option<[Interval; 3]> {
    match surface {
        Surface::Plane(flat) => Some(exactly(flat.normal().components())),
        Surface::Sphere(ball) => {
            let at = at.coordinates();
            let centre = ball.centre().coordinates();
            Some(core::array::from_fn(|axis| {
                Interval::exactly(at[axis]) - Interval::exactly(centre[axis])
            }))
        }
        Surface::Cone(cone) => {
            let at = at.coordinates();
            let apex = cone.apex().coordinates();
            let out: [Interval; 3] = core::array::from_fn(|axis| {
                Interval::exactly(at[axis]) - Interval::exactly(apex[axis])
            });
            let along = exactly(cone.axis().direction().components());
            let slope = Interval::exactly(cone.slope());
            let steepened = (Interval::exactly(1.0) + slope * slope) * dot(out, along);
            let squared = dot(along, along);
            Some(core::array::from_fn(|axis| {
                squared * out[axis] - steepened * along[axis]
            }))
        }
        Surface::Cylinder(pipe) => {
            let at = at.coordinates();
            let origin = pipe.axis().origin().coordinates();
            let out: [Interval; 3] = core::array::from_fn(|axis| {
                Interval::exactly(at[axis]) - Interval::exactly(origin[axis])
            });
            let along = exactly(pipe.axis().direction().components());
            let how_far_along = dot(out, along);
            let squared = dot(along, along);
            Some(core::array::from_fn(|axis| {
                squared * out[axis] - how_far_along * along[axis]
            }))
        }
        Surface::Torus(_) | _ => None,
    }
}

fn exactly(value: [f64; 3]) -> [Interval; 3] {
    value.map(Interval::exactly)
}

fn cross(u: [Interval; 3], v: [Interval; 3]) -> [Interval; 3] {
    [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ]
}

fn dot(u: [Interval; 3], v: [Interval; 3]) -> Interval {
    u[0] * v[0] + u[1] * v[1] + u[2] * v[2]
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
            Decision::Undecidable { enclosure } => {
                return Err(gave_up_on_a_side(partner, at, enclosure));
            }
            other => panic!("a corner came back with a decision nothing here knows: {other:?}"),
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
/// report, and §69.3 is what happened when it reported one anyway.
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
/// for. Naming the neighbour is what §69 records, and it panicked rather than
/// lying, which is the only reason it was ever found.
///
/// [`crossing`] gives up only through [`above_the_line`], which reads every
/// certified sign, so one of the two really is undecided and the panic below
/// has nothing to fall back to.
fn whichever_of_them_was_not_read(from_side: Decision, to_side: Decision) -> f64 {
    if let Decision::Undecidable { enclosure } = from_side {
        return enclosure;
    }
    let Decision::Undecidable { enclosure } = to_side else {
        panic!(
            "the crossing count was refused with both of its ends decided: {from_side:?} and {to_side:?}"
        )
    };
    enclosure
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
fn gave_up_on_a_side(surface: &Surface, point: PredicatePoint, enclosure: f64) -> Error {
    gave_up(
        name_of_surface(surface),
        point,
        Level::Exact,
        enclosure,
        Measures::Area,
    )
}

/// The same, for whether a cap's plane holds a ring's axis, filed under the
/// question [`intersect`] asks of the same two surfaces.
fn gave_up_on_the_axis(flat: &Plane, ring: &rcad_geom::Torus, enclosure: f64) -> Error {
    let mut records = ProvenanceLog::new();
    records.push(Record::new(
        WHERE_A_PLANE_AND_A_TORUS_CROSS,
        &[
            PredicatePoint::from(flat.origin().coordinates()),
            PredicatePoint::from(ring.centre().coordinates()),
        ],
        Level::Exact,
        enclosure,
        Measures::Length,
    ));
    Error::Undecidable { records }
}

/// The same, for an `orient3d` in the outline walk.
///
/// The predicate has already put its own record in the caller's log on the way
/// past (`orient3d_tolerantly` does that for every level it could not settle),
/// so what this adds is which *question* the walk was asking when it stopped.
fn gave_up_turning_a_corner(point: PredicatePoint, enclosure: f64) -> Error {
    gave_up(ORIENT3D, point, Level::Exact, enclosure, Measures::Volume)
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
        overhang,
        Measures::Length,
    )
}

fn gave_up(
    named: &'static str,
    point: PredicatePoint,
    level: Level,
    enclosure: f64,
    measures: Measures,
) -> Error {
    let mut records = ProvenanceLog::new();
    records.push(Record::new(named, &[point], level, enclosure, measures));
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
            .expect("this model made the key")
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
            .expect("this model made the key")
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
            .expect("this model made the key")
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
        let skin = model
            .solid(ball)
            .expect("this model made the key")
            .faces()
            .next()
            .expect("one face")
            .key();

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
            .expect("this model made the key")
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
        let skin = model
            .solid(ball)
            .expect("this model made the key")
            .faces()
            .next()
            .expect("one face")
            .key();

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
            .expect("this model made the key")
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

    /// What [`is_a_family_this_follows`] refuses a cone face for when the apex is
    /// **behind** one of its cuts, and why that refusal is a limit and not a gap.
    ///
    /// Two planes across the cone of slope one about `+z`, keeping
    /// `x + z/√2 ≥ 1` and `−x + z/√2 ≥ 1`, with the apex a whole unit behind each.
    /// Along one ruling a plane's value is affine in how far out you have gone, so
    /// a ruling carries a point in front of both exactly when both planes *grow*
    /// along it — there is no third case to sample for. This walks the circle of
    /// rulings and counts the runs of rulings that carry one: **two**, with
    /// rulings carrying nothing between them, so the region is two pieces with
    /// nothing between them and there is no one answer to the caller's question.
    #[test]
    fn two_planes_the_apex_is_behind_leave_a_nappe_in_two_pieces() {
        let cone = rcad_geom::Cone::new(Axis::z(), 1.0).expect("a positive slope");
        let leaned = |towards_x: f64| {
            Plane::from_point_and_normal(
                Point::new(towards_x, 0.0, 0.0),
                Vector::new(towards_x, 0.0, 0.5f64.sqrt()),
            )
            .expect("a plane leaning off the axis")
        };
        let planes = [leaned(1.0), leaned(-1.0)];
        let cuts: Vec<Cut> = planes
            .map(|plane| Cut {
                surface: Surface::Plane(plane),
                placed: Surface::Plane(plane),
                kept: Decision::Pos,
                corners_on_it: Vec::new(),
            })
            .into_iter()
            .collect();

        let apex = PredicatePoint::from(cone.apex().coordinates());
        for cut in &cuts {
            assert_eq!(
                side_of_surface(&cut.surface, apex),
                Decision::Neg,
                "the apex has to be behind both cuts for this to be the shape"
            );
        }
        assert!(
            !is_a_family_this_follows(&Surface::Cone(cone), &cuts),
            "a cone face whose apex is behind a cut is refused"
        );

        let carries_a_point = |angle: f64| {
            let along = [angle.cos(), angle.sin(), 1.0];
            planes.iter().all(|plane| {
                let normal = plane.normal().components();
                let rate: f64 = (0..3).map(|axis| normal[axis] * along[axis]).sum();
                assert!(plane.offset() > 0.0, "the apex is behind this plane");
                rate > 0.0
            })
        };
        let how_many = 3600;
        let round_the_rulings: Vec<bool> = (0..how_many)
            .map(|step| {
                carries_a_point(core::f64::consts::TAU * f64::from(step) / f64::from(how_many))
            })
            .collect();
        let runs = round_the_rulings
            .iter()
            .zip(round_the_rulings.iter().cycle().skip(1))
            .filter(|(now, next)| !**now && **next)
            .count();
        assert_eq!(runs, 2, "the rulings that carry a point come in two runs");
        assert!(
            carries_a_point(core::f64::consts::FRAC_PI_2),
            "the ruling at a quarter turn is in one of them"
        );
        assert!(
            !carries_a_point(0.0) && !carries_a_point(core::f64::consts::PI),
            "the rulings along and against +x carry nothing, which is what parts them"
        );
    }

    /// Why one cut the apex is behind is followed only beside **one** other cut,
    /// even when every cut closes.
    ///
    /// On the cone of slope a half about `+z`, keep `z ≥ 1`, which the apex is
    /// behind and which cuts a circle, and `z ≤ 3/2 ± 1.9x`, which the apex is in
    /// front of and which each cut an ellipse — `1.9` is just under the `2` past
    /// which they would not close. Along the ruling at `φ` the first keeps
    /// `t ≥ 1` and each of the others `t ≤ 3/2 / (1 ∓ 0.95 cos φ)`, so a ruling
    /// carries a point exactly where both of those reach `1`, which is
    /// `|cos φ| ≤ 0.526…`: **two** runs of rulings, round `±90°`, with the rulings
    /// along `±x` carrying nothing. Two pieces, and no one answer.
    ///
    /// Drop either front cut and what is left is one run, which is the case
    /// [`is_a_family_this_follows`] takes: with one cut behind and one other, a
    /// ruling carries a point where a single expression affine in `cos φ` and
    /// `sin φ` is not negative, and that is one arc of the circle of rulings.
    #[test]
    fn one_plane_behind_the_apex_and_two_in_front_leave_a_nappe_in_two_pieces() {
        let cone = rcad_geom::Cone::new(Axis::z(), 0.5).expect("a positive slope");
        let cut = |through: [f64; 3], facing: [f64; 3]| {
            let plane = Plane::from_point_and_normal(
                Point::new(through[0], through[1], through[2]),
                Vector::new(facing[0], facing[1], facing[2]),
            )
            .expect("a normal with a length");
            assert_eq!(
                cone.is_cut_in_a_closed_curve_by(&plane),
                Some(true),
                "every cut here closes: {facing:?}"
            );
            Cut {
                surface: Surface::Plane(plane),
                placed: Surface::Plane(plane),
                kept: Decision::Pos,
                corners_on_it: Vec::new(),
            }
        };
        let floor = || cut([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]);
        let one_roof = || cut([0.0, 0.0, 1.5], [1.9, 0.0, -1.0]);
        let other_roof = || cut([0.0, 0.0, 1.5], [-1.9, 0.0, -1.0]);

        let apex = PredicatePoint::from(cone.apex().coordinates());
        assert_eq!(side_of_surface(&floor().surface, apex), Decision::Neg);
        assert_eq!(side_of_surface(&one_roof().surface, apex), Decision::Pos);
        assert_eq!(side_of_surface(&other_roof().surface, apex), Decision::Pos);

        let three = [floor(), one_roof(), other_roof()];
        assert!(
            !is_a_family_this_follows(&Surface::Cone(cone), &three),
            "one cut behind the apex and two others are refused"
        );
        assert_eq!(runs_of_rulings_that_carry_a_point(&three), 2);

        let two = [floor(), one_roof()];
        assert!(
            is_a_family_this_follows(&Surface::Cone(cone), &two),
            "one cut behind the apex that closes, and one other, are followed"
        );
        assert_eq!(runs_of_rulings_that_carry_a_point(&two), 1);
    }

    /// Why a cone face is followed beside at most **one** cut through its apex.
    ///
    /// On the cone of slope a half about `+z`, keep `x ≤ z/10` and `−x ≤ z/10`,
    /// both through the apex, and `z ≤ 8`, which the apex is in front of. Along
    /// the ruling at `φ` a plane through the apex is zero at the apex and grows
    /// at one rate, so it keeps the whole ruling or only the apex. Both keep more
    /// than the apex exactly where `|cos φ| < 1/5`: **two** runs of rulings, round
    /// `+y` and round `−y`, two sheets of one nappe meeting only at the apex. The
    /// part in front of every cut is connected through the apex and is still not
    /// one face, and before this refused it `[] => true` followed it.
    ///
    /// One plane through the apex keeps one run, and that is followed.
    ///
    /// **Two are refused even where they keep one run**, which is the gap
    /// `docs/decisions.md` §389 files: `x ≥ 0` and `y ≥ 0`, a quarter of a pointed
    /// cone, meet along the axis, inside the cone, and keep `0 < φ < 90°`.
    #[test]
    fn two_planes_through_the_apex_leave_a_nappe_in_two_sheets() {
        let cone = rcad_geom::Cone::new(Axis::z(), 0.5).expect("a positive slope");
        let cut = |through: [f64; 3], facing: [f64; 3]| {
            let plane = Plane::from_point_and_normal(
                Point::new(through[0], through[1], through[2]),
                Vector::new(facing[0], facing[1], facing[2]),
            )
            .expect("a normal with a length");
            Cut {
                surface: Surface::Plane(plane),
                placed: Surface::Plane(plane),
                kept: Decision::Pos,
                corners_on_it: Vec::new(),
            }
        };
        let one_side = || cut([0.0; 3], [-1.0, 0.0, 0.1]);
        let other_side = || cut([0.0; 3], [1.0, 0.0, 0.1]);
        let roof = || cut([0.0, 0.0, 8.0], [0.0, 0.0, -1.0]);

        let apex = PredicatePoint::from(cone.apex().coordinates());
        assert_eq!(side_of_surface(&one_side().surface, apex), Decision::Zero);
        assert_eq!(side_of_surface(&other_side().surface, apex), Decision::Zero);
        assert_eq!(side_of_surface(&roof().surface, apex), Decision::Pos);

        let strip = [one_side(), other_side(), roof()];
        assert_eq!(runs_of_rulings_carrying_more_than_the_apex(&strip), 2);
        assert!(
            !is_a_family_this_follows(&Surface::Cone(cone), &strip),
            "two cuts through the apex are refused"
        );

        let half = [one_side(), roof()];
        assert_eq!(runs_of_rulings_carrying_more_than_the_apex(&half), 1);
        assert!(
            is_a_family_this_follows(&Surface::Cone(cone), &half),
            "one cut through the apex, and any the apex is in front of, are followed"
        );

        let quarter = [
            cut([0.0; 3], [1.0, 0.0, 0.0]),
            cut([0.0; 3], [0.0, 1.0, 0.0]),
            roof(),
        ];
        assert_eq!(runs_of_rulings_carrying_more_than_the_apex(&quarter), 1);
        assert!(
            !is_a_family_this_follows(&Surface::Cone(cone), &quarter),
            "two cuts through the apex meeting inside the cone keep one run, and are refused \
             all the same"
        );
    }

    /// How many runs of the rulings of the slope-one-half cone about `+z` carry a
    /// point in front of every cut other than the apex, walked a tenth of a
    /// degree at a time. A cut through the apex keeps a ruling where it grows
    /// along it; [`runs_of_rulings_that_carry_a_point`] counts the apex alone as
    /// a point, which every ruling carries.
    fn runs_of_rulings_carrying_more_than_the_apex(cuts: &[Cut]) -> usize {
        let carries_more = |angle: f64| {
            let along = [0.5 * angle.cos(), 0.5 * angle.sin(), 1.0];
            let (mut from, mut to) = (0.0, f64::INFINITY);
            for cut in cuts {
                let plane = cut.surface.as_plane().expect("every cut here is a plane");
                let normal = plane.normal().components();
                let rate: f64 = (0..3).map(|axis| normal[axis] * along[axis]).sum();
                let at_the_apex = -plane.offset();
                if rate > 0.0 {
                    from = f64::max(from, -at_the_apex / rate);
                } else if rate < 0.0 {
                    to = f64::min(to, -at_the_apex / rate);
                } else if at_the_apex < 0.0 {
                    return false;
                }
            }
            from < to
        };
        let how_many = 3600;
        let round_the_rulings: Vec<bool> = (0..how_many)
            .map(|step| {
                carries_more(core::f64::consts::TAU * f64::from(step) / f64::from(how_many))
            })
            .collect();
        round_the_rulings
            .iter()
            .zip(round_the_rulings.iter().cycle().skip(1))
            .filter(|(now, next)| !**now && **next)
            .count()
    }

    /// How many runs of the rulings of the slope-one-half cone about `+z` carry a
    /// point in front of every cut, walked a tenth of a degree at a time.
    ///
    /// A plane's value along a ruling is affine in how far out you have gone, so
    /// each cut keeps a half-line of it and a ruling carries a point exactly when
    /// the half-lines meet.
    fn runs_of_rulings_that_carry_a_point(cuts: &[Cut]) -> usize {
        let carries_a_point = |angle: f64| {
            let along = [0.5 * angle.cos(), 0.5 * angle.sin(), 1.0];
            let (mut from, mut to) = (0.0, f64::INFINITY);
            for cut in cuts {
                let plane = cut.surface.as_plane().expect("every cut here is a plane");
                let normal = plane.normal().components();
                let rate: f64 = (0..3).map(|axis| normal[axis] * along[axis]).sum();
                let at_the_apex = -plane.offset();
                if rate > 0.0 {
                    from = f64::max(from, -at_the_apex / rate);
                } else if rate < 0.0 {
                    to = f64::min(to, -at_the_apex / rate);
                } else if at_the_apex < 0.0 {
                    return false;
                }
            }
            from <= to
        };
        let how_many = 3600;
        let round_the_rulings: Vec<bool> = (0..how_many)
            .map(|step| {
                carries_a_point(core::f64::consts::TAU * f64::from(step) / f64::from(how_many))
            })
            .collect();
        round_the_rulings
            .iter()
            .zip(round_the_rulings.iter().cycle().skip(1))
            .filter(|(now, next)| !**now && **next)
            .count()
    }

    /// Why a pipe face cut by a floor, a roof and a plane along the axis is
    /// followed only when the floor and the roof lie side by side.
    ///
    /// On the pipe of radius one up `+z`, keep `x ≥ −½` (along the axis),
    /// `z ≥ 0` (a floor) and `z ≤ ½ − x` (a roof). The ruling at `φ` is at
    /// `x = cos φ`: it is in front of the first where `cos φ ≥ −½`, and its
    /// stretch `0 ≤ z ≤ ½ − cos φ` is there where `cos φ ≤ ½`. So a ruling carries
    /// a point where `|cos φ| ≤ ½`: **two** runs, round `±90°`, with the rulings
    /// along `±x` carrying nothing. Two pieces, and no one answer.
    ///
    /// Level the roof to `z ≤ 2`, side by side with the floor as a half bar's two
    /// caps are, and every ruling the first cut leaves carries its whole stretch:
    /// one run, and [`leaves_a_pipe_in_one_piece`] takes it.
    #[test]
    fn a_floor_a_leaned_roof_and_a_cut_along_the_axis_leave_a_pipe_in_two_pieces() {
        let pipe = Cylinder::new(Axis::z(), 1.0).expect("a positive radius");
        let cut = |through: [f64; 3], facing: [f64; 3]| {
            let plane = Plane::from_point_and_normal(
                Point::new(through[0], through[1], through[2]),
                Vector::new(facing[0], facing[1], facing[2]),
            )
            .expect("a normal with a length");
            Cut {
                surface: Surface::Plane(plane),
                placed: Surface::Plane(plane),
                kept: Decision::Pos,
                corners_on_it: Vec::new(),
            }
        };
        let along_the_axis = || cut([-0.5, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let floor = || cut([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let leaned_roof = || cut([0.0, 0.0, 0.5], [-1.0, 0.0, -1.0]);
        let level_roof = || cut([0.0, 0.0, 2.0], [0.0, 0.0, -1.0]);

        let leaned = [along_the_axis(), floor(), leaned_roof()];
        assert!(
            !is_a_family_this_follows(&Surface::Cylinder(pipe), &leaned),
            "a floor and a roof that are not side by side, beside a cut along the axis, are refused"
        );
        assert_eq!(runs_of_a_pipes_rulings_that_carry_a_point(&leaned), 2);

        let level = [along_the_axis(), floor(), level_roof()];
        assert!(
            is_a_family_this_follows(&Surface::Cylinder(pipe), &level),
            "a floor and a roof side by side, beside a cut along the axis, are followed"
        );
        assert_eq!(runs_of_a_pipes_rulings_that_carry_a_point(&level), 1);
    }

    /// How many runs of the rulings of the pipe of radius one up `+z` carry a
    /// point in front of every cut, walked a tenth of a degree at a time.
    ///
    /// A plane's value up a ruling is affine in how far up you have gone, so each
    /// cut keeps a half-line of it, or all of it, or none, and a ruling carries a
    /// point exactly when what they keep meets.
    fn runs_of_a_pipes_rulings_that_carry_a_point(cuts: &[Cut]) -> usize {
        let carries_a_point = |angle: f64| {
            let foot = [angle.cos(), angle.sin(), 0.0];
            let (mut from, mut to) = (f64::NEG_INFINITY, f64::INFINITY);
            for cut in cuts {
                let plane = cut.surface.as_plane().expect("every cut here is a plane");
                let normal = plane.normal().components();
                let at_the_foot: f64 =
                    (0..3).map(|axis| normal[axis] * foot[axis]).sum::<f64>() - plane.offset();
                let rate = normal[2];
                if rate > 0.0 {
                    from = f64::max(from, -at_the_foot / rate);
                } else if rate < 0.0 {
                    to = f64::min(to, -at_the_foot / rate);
                } else if at_the_foot < 0.0 {
                    return false;
                }
            }
            from <= to
        };
        let how_many = 3600;
        let round_the_rulings: Vec<bool> = (0..how_many)
            .map(|step| {
                carries_a_point(core::f64::consts::TAU * f64::from(step) / f64::from(how_many))
            })
            .collect();
        round_the_rulings
            .iter()
            .zip(round_the_rulings.iter().cycle().skip(1))
            .filter(|(now, next)| !**now && **next)
            .count()
    }

    /// How far round the ring a bend goes from its cap on `y = 0` at `+x`.
    #[derive(Clone, Copy)]
    enum Bend {
        /// To a cap on `x = 0` at `+y`.
        Quarter,
        /// To a cap on `x = 0` at `−y`, the long way round.
        ThreeQuarters,
        /// To a cap on `y = 0` at `−x`, round through `+y`: both caps on one
        /// plane, facing `−y`, the way `tests/rings/mod.rs` builds its half
        /// bend.
        Half,
    }

    /// How a bend's near cap, on `y = 0`, is stored.
    #[derive(Clone, Copy)]
    enum NearCap {
        /// Facing out of the bend, `−y`, the way every builder here stores a
        /// plane.
        FacingOut,
        /// Facing into the bend, `+y`, and used reversed: the same solid.
        /// `Model::validate` refuses a flat face used reversed, and no builder
        /// or file makes one, so only a face asked about directly meets it.
        FacingInAndUsedReversed,
    }

    /// A bend of pipe about `+z`, ring four and tube one, built the way
    /// `tests/rings/mod.rs` builds its quarter bend but straight into the arena,
    /// so no sweep stands between the description and the face asked about.
    ///
    /// **The three-quarter bend's far end is the quarter bend's turned half a
    /// turn round the axis** — corner, circle and cap — and a turn about the
    /// axis keeps which way every coedge runs against its surfaces. So both
    /// bends walk every coedge the same way, and the near end is the same edge
    /// for both. That is what makes the long one a real bend and not a guess.
    fn a_bend(bend: Bend, near_cap: NearCap) -> (Model, SolidKey, FaceKey) {
        let (far_corner, far_cap_facing, the_way_round) = match bend {
            Bend::Quarter => (
                rcad_geom::Point::new(0.0, 3.0, 0.0),
                Vector::new(-1.0, 0.0, 0.0),
                HalfSpace::new(Vector::new(1.0, 1.0, 0.0), 0.0),
            ),
            Bend::ThreeQuarters => (
                rcad_geom::Point::new(0.0, -3.0, 0.0),
                Vector::new(1.0, 0.0, 0.0),
                HalfSpace::new(Vector::new(-1.0, 1.0, 0.0), -3.0),
            ),
            Bend::Half => (
                rcad_geom::Point::new(-3.0, 0.0, 0.0),
                Vector::new(0.0, -1.0, 0.0),
                HalfSpace::new(Vector::new(0.0, 1.0, 0.0), 0.0),
            ),
        };
        let (near_cap_facing, near_cap_used) = match near_cap {
            NearCap::FacingOut => (Vector::new(0.0, -1.0, 0.0), Orientation::Forward),
            NearCap::FacingInAndUsedReversed => (Vector::new(0.0, 1.0, 0.0), Orientation::Reversed),
        };
        let through_the_axis = |facing| {
            Plane::from_point_and_normal(rcad_geom::Point::origin(), facing)
                .expect("a coordinate direction is a unit normal")
        };
        let flat_through_the_middle = through_the_axis(Vector::new(0.0, 0.0, 1.0));
        a_piece_of_pipe(
            rcad_geom::Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring"),
            End {
                corner: rcad_geom::Point::new(3.0, 0.0, 0.0),
                cap: through_the_axis(near_cap_facing),
                used: near_cap_used,
            },
            End {
                corner: far_corner,
                cap: through_the_axis(far_cap_facing),
                used: Orientation::Forward,
            },
            Some(EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(flat_through_the_middle),
                picked_by: smallvec::smallvec![
                    Picker::InsideAPipe(Cylinder::new(Axis::z(), 4.0).expect("the ring's radius")),
                    Picker::from(the_way_round.expect("a normal with a length")),
                ],
            }),
        )
    }

    /// One end of a piece of pipe: a corner on the ring, the plane its cap is
    /// stored on, and which way round the shell uses that cap.
    struct End {
        corner: rcad_geom::Point,
        cap: Plane,
        used: Orientation,
    }

    /// A piece of pipe round `ring` between two flat caps, every coedge walked
    /// the way [`a_bend`] walks the quarter bend's.
    ///
    /// That walk says two things, and each caller makes them true of its own
    /// caps: the wall runs round the ring from the near cap's circle to the far
    /// one's the way `+x` turns into `+y`, and each cap, as it is used, faces
    /// away from the wall. `seam` runs along the wall from the near corner to
    /// the far one; without one the wall is two wires, one round each circle,
    /// and nothing asked of a wall here reads a seam.
    fn a_piece_of_pipe(
        ring: rcad_geom::Torus,
        near: End,
        far: End,
        seam: Option<EdgeRun>,
    ) -> (Model, SolidKey, FaceKey) {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let near_corner = model.add_vertex(near.corner);
        let far_corner = model.add_vertex(far.corner);
        let seam = seam.map(|run| model.add_edge_running(near_corner, far_corner, run));
        let near_circle = model.add_closed_edge(near_corner);
        let far_circle = model.add_closed_edge(far_corner);

        let cap = |model: &mut Model, plane: Plane, circle, turn| {
            let round_it = model.add_coedge(circle, turn);
            let wire = model.add_wire(smallvec::smallvec![round_it]);
            model.add_face(Surface::Plane(plane), smallvec::smallvec![wire])
        };
        let near_cap = cap(&mut model, near.cap, near_circle, Orientation::Forward);
        let far_cap = cap(&mut model, far.cap, far_circle, Orientation::Reversed);

        let wires: SmallVec<[_; 1]> = match seam {
            Some(seam) => {
                let round_the_wall: SmallVec<[_; 4]> = [
                    (near_circle, Orientation::Reversed),
                    (seam, Orientation::Forward),
                    (far_circle, Orientation::Forward),
                    (seam, Orientation::Reversed),
                ]
                .map(|(edge, turn)| model.add_coedge(edge, turn))
                .into_iter()
                .collect();
                smallvec::smallvec![model.add_wire(round_the_wall)]
            }
            None => [
                (near_circle, Orientation::Reversed),
                (far_circle, Orientation::Forward),
            ]
            .map(|(edge, turn)| {
                let round_it = model.add_coedge(edge, turn);
                model.add_wire(smallvec::smallvec![round_it])
            })
            .into_iter()
            .collect(),
        };
        let wall = model.add_face(Surface::Torus(ring), wires);

        let shell = model.add_shell(
            [
                (near_cap, near.used),
                (far_cap, far.used),
                (wall, Orientation::Forward),
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
        let wall = model
            .solid(key)
            .expect("this model made the key")
            .faces()
            .find(|face| matches!(face.surface(), Surface::Torus(_)))
            .expect("a piece of pipe has a wall")
            .key();
        (model, key, wall)
    }

    /// A quarter bend's wall holds the quarter of the ring between its caps,
    /// and nothing of the other three quarters (§243).
    #[test]
    fn a_quarter_bends_wall_holds_the_quarter_between_its_caps_and_nothing_else() {
        let (model, solid, wall) = a_bend(Bend::Quarter, NearCap::FacingOut);
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("the bend `tests/rings/mod.rs` builds through the sweep");
        a_quarter_bends_wall_holds_the_quarter(&model, solid, wall);
    }

    /// The same quarter bend with its near cap's plane stored facing into the
    /// bend and used reversed holds the same quarter.
    ///
    /// The side the wall keeps of that plane is read off the wire and then
    /// turned round for the reversed use, because that is what `Model::validate`
    /// does to the side it hands the closed-edge rule. Without the turn the wall
    /// keeps the back of the plane as stored, which is `y < 0`, the far circle
    /// is not in front of it, and the wall is refused.
    #[test]
    fn a_quarter_bends_wall_holds_the_same_quarter_with_its_near_cap_stored_turned_round() {
        let (model, solid, wall) = a_bend(Bend::Quarter, NearCap::FacingInAndUsedReversed);
        a_quarter_bends_wall_holds_the_quarter(&model, solid, wall);
    }

    /// Every point asked is exactly on the ring — on its outer equator at a
    /// 3-4-5 angle, or at the top of the tube — so each answer is a sign and
    /// the oracle is which quarter the point is in. `(−5, 0, 0)` is on the near
    /// cap's own plane, on the circle the wall does not have.
    fn a_quarter_bends_wall_holds_the_quarter(model: &Model, solid: SolidKey, wall: FaceKey) {
        let expected = [
            (at(3.0, 4.0, 0.0), OnAFace::Held),
            (at(4.0, 3.0, 0.0), OnAFace::Held),
            (at(5.0, 0.0, 0.0), OnAFace::OnItsBoundary),
            (at(0.0, 4.0, 1.0), OnAFace::OnItsBoundary),
            (at(-3.0, 4.0, 0.0), OnAFace::NotHeld),
            (at(-3.0, -4.0, 0.0), OnAFace::NotHeld),
            (at(3.0, -4.0, 0.0), OnAFace::NotHeld),
            (at(-5.0, 0.0, 0.0), OnAFace::NotHeld),
        ];
        let bend = model.solid(solid).expect("this model made the key");
        for (point, answer) in expected {
            let held = bend
                .face_holds_the_point(wall, point, &Policy::certified(), &mut ProvenanceLog::new())
                .expect("a quarter bend's wall is a face this can follow");
            assert_eq!(held, answer, "{point:?}");
        }
    }

    /// The rounded rim of a puck is refused, and no longer holds the inside of
    /// its ring.
    ///
    /// A disk of radius four on `z = 1`, one on `z = −1`, and between them the
    /// outer half of the ring of radius four and tube one: each disk's plane
    /// rests on the ring along one circle, and the rim has that circle as an
    /// edge. Two planes with one edge along each, like a bend's wall — but
    /// neither holds the ring's axis. The general rule read the rim as the part
    /// of the ring between the two planes, which is the inner half too, and
    /// `(3, 0, 0)` down in the hole came back held (§243). The rim's wire runs
    /// the way the bend's wall's does; nothing asked here reads it.
    #[test]
    fn a_pucks_rounded_rim_is_refused_rather_than_holding_the_inside_of_its_ring() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let top = model.add_vertex(rcad_geom::Point::new(4.0, 0.0, 1.0));
        let bottom = model.add_vertex(rcad_geom::Point::new(4.0, 0.0, -1.0));
        let upright =
            Plane::from_point_and_normal(rcad_geom::Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("the y axis is a unit normal");
        let seam = model.add_edge_running(
            bottom,
            top,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(upright),
                picked_by: smallvec::smallvec![Picker::from(
                    HalfSpace::new(Vector::new(1.0, 0.0, 0.0), 4.0)
                        .expect("a normal with a length")
                )],
            },
        );
        let round_the_top = model.add_closed_edge(top);
        let round_the_bottom = model.add_closed_edge(bottom);
        let disk = |model: &mut Model, up: f64, circle, turn| {
            let plane = Plane::from_point_and_normal(
                rcad_geom::Point::new(0.0, 0.0, up),
                Vector::new(0.0, 0.0, up),
            )
            .expect("the z axis is a unit normal");
            let round_it = model.add_coedge(circle, turn);
            let wire = model.add_wire(smallvec::smallvec![round_it]);
            model.add_face(Surface::Plane(plane), smallvec::smallvec![wire])
        };
        let lid = disk(&mut model, 1.0, round_the_top, Orientation::Reversed);
        let base = disk(&mut model, -1.0, round_the_bottom, Orientation::Forward);
        let round_the_rim: SmallVec<[_; 4]> = [
            (round_the_bottom, Orientation::Reversed),
            (seam, Orientation::Forward),
            (round_the_top, Orientation::Forward),
            (seam, Orientation::Reversed),
        ]
        .map(|(edge, turn)| model.add_coedge(edge, turn))
        .into_iter()
        .collect();
        let wire = model.add_wire(round_the_rim);
        let ring =
            rcad_geom::Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring");
        let rim = model.add_face(Surface::Torus(ring), smallvec::smallvec![wire]);
        let shell = model.add_shell(
            [
                (lid, Orientation::Forward),
                (base, Orientation::Forward),
                (rim, Orientation::Forward),
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
        let puck = model
            .solid(model.key_for_solid(solid))
            .expect("this model made the key");
        let rim = puck
            .faces()
            .find(|face| matches!(face.surface(), Surface::Torus(_)))
            .expect("a puck has a rim")
            .key();

        for point in [at(3.0, 0.0, 0.0), at(5.0, 0.0, 0.0)] {
            let answer = puck.face_holds_the_point(
                rim,
                point,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            assert!(
                matches!(answer, Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face }) if face == rim),
                "{point:?}: {answer:?}"
            );
        }
    }

    /// A puck's lid is refused rather than holding nothing but its rim.
    ///
    /// The lid is the disk of radius four on `z = 1`, and that plane rests on
    /// the ring of radius four and tube one along the lid's own circle. The
    /// ring's inside on that plane is empty, so the ring's sign put every point
    /// of the lid but its circle outside it: the lid's own centre `(0, 0, 1)`
    /// came back not held. A plane square to the axis that cuts the tube in two
    /// circles names each by a side of the pipe of the ring's own radius, and
    /// is answered (§250.9); this one rests on the tube and touches it in one
    /// circle, which `intersect` names by no picker at all, so the lid is
    /// refused (§250).
    #[test]
    fn a_pucks_lid_is_refused_rather_than_holding_nothing_but_its_rim() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let top = model.add_vertex(rcad_geom::Point::new(4.0, 0.0, 1.0));
        let bottom = model.add_vertex(rcad_geom::Point::new(4.0, 0.0, -1.0));
        let round_the_top = model.add_closed_edge(top);
        let round_the_bottom = model.add_closed_edge(bottom);
        let disk = |model: &mut Model, up: f64, circle, turn| {
            let plane = Plane::from_point_and_normal(
                rcad_geom::Point::new(0.0, 0.0, up),
                Vector::new(0.0, 0.0, up),
            )
            .expect("the z axis is a unit normal");
            let round_it = model.add_coedge(circle, turn);
            let wire = model.add_wire(smallvec::smallvec![round_it]);
            model.add_face(Surface::Plane(plane), smallvec::smallvec![wire])
        };
        let lid = disk(&mut model, 1.0, round_the_top, Orientation::Reversed);
        let base = disk(&mut model, -1.0, round_the_bottom, Orientation::Forward);
        let rim_wires: SmallVec<[_; 1]> = [
            (round_the_bottom, Orientation::Reversed),
            (round_the_top, Orientation::Forward),
        ]
        .map(|(edge, turn)| {
            let round_it = model.add_coedge(edge, turn);
            model.add_wire(smallvec::smallvec![round_it])
        })
        .into_iter()
        .collect();
        let ring =
            rcad_geom::Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring");
        let rim = model.add_face(Surface::Torus(ring), rim_wires);
        let shell = model.add_shell(
            [
                (lid, Orientation::Forward),
                (base, Orientation::Forward),
                (rim, Orientation::Forward),
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
        let puck = model
            .solid(model.key_for_solid(solid))
            .expect("this model made the key");
        let lid = puck
            .faces()
            .find(|face| {
                face.surface()
                    .as_plane()
                    .is_some_and(|flat| flat.offset() > 0.0)
            })
            .expect("a puck has a lid")
            .key();

        for point in [at(0.0, 0.0, 1.0), at(2.0, 0.0, 1.0), at(5.0, 0.0, 1.0)] {
            let answer = puck.face_holds_the_point(
                lid,
                point,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            assert!(
                matches!(answer, Err(Error::AFaceTrimmedByACurveThisCannotFollow { face }) if face == lid),
                "{point:?}: {answer:?}"
            );
        }
    }

    /// A three-quarter bend's wall is refused, and not read as a quarter.
    ///
    /// It is the part of the ring in front of *either* cap's plane, not of
    /// both, so no answer read off the two planes together is right. The
    /// refusal comes from its circles: the near one is behind the far cap's
    /// plane. Read off a corner instead of the wire, both kept sides come out
    /// turned round, both circles look in front, and every point of the one
    /// quarter the wall does *not* cover comes back held (§243).
    ///
    /// `Model::validate` refuses this bend, which is the quarter bend with its
    /// far end turned half a turn: `AClosedEdgeIsWalkedAgainstItsSurface`. It
    /// hands the closed-edge rule the side its corners read, and both of the
    /// wall's circles have the far corner on the other side of their plane.
    #[test]
    fn a_three_quarter_bends_wall_is_refused_rather_than_read_as_a_quarter() {
        let (model, solid, wall) = a_bend(Bend::ThreeQuarters, NearCap::FacingOut);
        assert!(
            matches!(
                model.validate(&Policy::certified(), &mut ProvenanceLog::new()),
                Err(Error::Invalid(
                    Invalid::AClosedEdgeIsWalkedAgainstItsSurface
                ))
            ),
            "the sweep has learned a three-quarter bend, and §243 says what that changes"
        );
        for point in [at(3.0, -4.0, 0.0), at(-3.0, 4.0, 0.0), at(3.0, 4.0, 0.0)] {
            let answer = model
                .solid(solid)
                .expect("this model made the key")
                .face_holds_the_point(wall, point, &Policy::certified(), &mut ProvenanceLog::new());
            assert!(
                matches!(answer, Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face }) if face == wall),
                "{point:?}: {answer:?}"
            );
        }
    }

    /// A half bend's wall holds the half of the ring on the side of `y = 0` it
    /// runs round through, and nothing of the other half (§377).
    ///
    /// Both caps are stored on the one plane `y = 0` facing `−y`, so the wall
    /// is trimmed once, along both of that plane's circles, and every corner of
    /// it is on the plane. The general rule found no corner to read the side
    /// off and refused with `NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`.
    #[test]
    fn a_half_bends_wall_holds_the_half_of_the_ring_its_caps_plane_leaves_on_its_side() {
        let (model, solid, wall) = a_bend(Bend::Half, NearCap::FacingOut);
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("the half bend `tests/rings/mod.rs` builds through the sweep");
        a_half_bends_wall_holds_the_half(&model, solid, wall);
    }

    /// The same half bend with its near cap stored facing `+y` and used
    /// reversed holds the same half.
    ///
    /// The two caps are now two trimmings, one plane stored two ways, and each
    /// circle is on the other's plane. §243's rule wanted each circle strictly
    /// in front of the other plane and refused with
    /// `AFaceMissesPartOfWhereItsSurfacesCross`. The two sides the wire reads
    /// differ, as the two normals do, so they are one side.
    #[test]
    fn a_half_bends_wall_holds_the_same_half_with_its_near_cap_stored_turned_round() {
        let (model, solid, wall) = a_bend(Bend::Half, NearCap::FacingInAndUsedReversed);
        a_half_bends_wall_holds_the_half(&model, solid, wall);
    }

    /// Every point asked is exactly on the ring: on its outer or inner equator
    /// at a 3-4-5 angle, at the top or bottom of the tube, or on one of the two
    /// circles `y = 0` cuts it in.
    fn a_half_bends_wall_holds_the_half(model: &Model, solid: SolidKey, wall: FaceKey) {
        let expected = [
            (at(3.0, 4.0, 0.0), OnAFace::Held),
            (at(-3.0, 4.0, 0.0), OnAFace::Held),
            (at(0.0, 4.0, 1.0), OnAFace::Held),
            (at(0.0, 4.0, -1.0), OnAFace::Held),
            (at(0.0, 3.0, 0.0), OnAFace::Held),
            (at(3.0, -4.0, 0.0), OnAFace::NotHeld),
            (at(-3.0, -4.0, 0.0), OnAFace::NotHeld),
            (at(0.0, -4.0, 1.0), OnAFace::NotHeld),
            (at(0.0, -5.0, 0.0), OnAFace::NotHeld),
            (at(5.0, 0.0, 0.0), OnAFace::OnItsBoundary),
            (at(-5.0, 0.0, 0.0), OnAFace::OnItsBoundary),
            (at(4.0, 0.0, 1.0), OnAFace::OnItsBoundary),
            (at(-4.0, 0.0, -1.0), OnAFace::OnItsBoundary),
            (at(-3.0, 0.0, 0.0), OnAFace::OnItsBoundary),
        ];
        let bend = model.solid(solid).expect("this model made the key");
        for (point, answer) in expected {
            let held = bend
                .face_holds_the_point(wall, point, &Policy::certified(), &mut ProvenanceLog::new())
                .expect("a half bend's wall is a face this can follow");
            assert_eq!(held, answer, "{point:?}");
        }
    }

    /// A wall whose two edges both run round the circle `y = 0` cuts at
    /// `x > 0` is refused, and not read as a half of the ring.
    ///
    /// The far corner is `(5, 0, 0)`, on the near circle, so the wall has one
    /// circle twice and the other circle not at all. Its wire walks that
    /// circle once each way, so it keeps one side of the plane at one edge and
    /// the other side at the other, and no half of the ring is that. Whichever
    /// way the far cap's plane is stored, and however it is used, the two
    /// sides read off the wire disagree once turned to one normal.
    #[test]
    fn a_wall_round_one_circle_twice_is_refused_rather_than_read_as_a_half() {
        let through_the_axis = |facing| {
            Plane::from_point_and_normal(rcad_geom::Point::origin(), facing)
                .expect("a coordinate direction is a unit normal")
        };
        for (far_facing, far_used) in [
            (Vector::new(0.0, -1.0, 0.0), Orientation::Forward),
            (Vector::new(0.0, 1.0, 0.0), Orientation::Forward),
            (Vector::new(0.0, 1.0, 0.0), Orientation::Reversed),
        ] {
            let (model, solid, wall) = a_piece_of_pipe(
                rcad_geom::Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring"),
                End {
                    corner: rcad_geom::Point::new(3.0, 0.0, 0.0),
                    cap: through_the_axis(Vector::new(0.0, -1.0, 0.0)),
                    used: Orientation::Forward,
                },
                End {
                    corner: rcad_geom::Point::new(5.0, 0.0, 0.0),
                    cap: through_the_axis(far_facing),
                    used: far_used,
                },
                None,
            );
            let bend = model.solid(solid).expect("this model made the key");
            for point in [at(3.0, 4.0, 0.0), at(3.0, -4.0, 0.0)] {
                let answer = bend.face_holds_the_point(
                    wall,
                    point,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                assert!(
                    matches!(answer, Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face }) if face == wall),
                    "far cap facing {far_facing:?}, used {far_used:?}, {point:?}: {answer:?}"
                );
            }
        }
    }

    /// A pipe over the top of a ring between the planes `x = 3` and `x = −3`
    /// is refused, because neither plane holds the ring's centre.
    ///
    /// Ring six, tube one, corners `(±3, 4, 0)` on the inner equator. Both
    /// planes run along the axis, so each cuts the ring in two ovals, one
    /// either side of `y = 0`, and the wall has the two at `y > 0`. The part
    /// of the ring in front of both planes is the band `−3 < x < 3`, which is
    /// this wall *and* its mirror image under `y = 0`. Asked only whether each
    /// plane runs along the axis, both edges are in front of the other plane,
    /// and `(0, −5, 0)`, on the far side of the ring, comes back held.
    #[test]
    fn a_pipe_between_two_parallel_planes_is_refused_rather_than_holding_the_far_side_of_its_ring()
    {
        let facing = |x: f64| {
            Plane::from_point_and_normal(
                rcad_geom::Point::new(x, 0.0, 0.0),
                Vector::new(x, 0.0, 0.0),
            )
            .expect("a coordinate direction is a unit normal")
        };
        let flat_through_the_middle =
            Plane::from_point_and_normal(rcad_geom::Point::origin(), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal");
        let (model, solid, wall) = a_piece_of_pipe(
            rcad_geom::Torus::new(Axis::z(), 6.0, 1.0).expect("a tube narrower than its ring"),
            End {
                corner: rcad_geom::Point::new(3.0, 4.0, 0.0),
                cap: facing(3.0),
                used: Orientation::Forward,
            },
            End {
                corner: rcad_geom::Point::new(-3.0, 4.0, 0.0),
                cap: facing(-3.0),
                used: Orientation::Forward,
            },
            Some(EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(flat_through_the_middle),
                picked_by: smallvec::smallvec![
                    Picker::InsideAPipe(Cylinder::new(Axis::z(), 6.0).expect("the ring's radius")),
                    Picker::from(
                        HalfSpace::new(Vector::new(0.0, 1.0, 0.0), 0.0)
                            .expect("a normal with a length")
                    ),
                ],
            }),
        );
        let pipe = model.solid(solid).expect("this model made the key");
        for point in [at(0.0, -5.0, 0.0), at(0.0, 5.0, 0.0)] {
            let answer = pipe.face_holds_the_point(
                wall,
                point,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            assert!(
                matches!(answer, Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face }) if face == wall),
                "{point:?}: {answer:?}"
            );
        }
    }

    /// A bend of just over half a turn whose far cap leans is refused, because
    /// that cap's plane holds the ring's centre but not its axis.
    ///
    /// Ring four, tube one. The near cap is on a plane through the axis an
    /// eighth-slope short of `+x`, facing `(−1, −8, 0)`; the far cap is on
    /// `y = z`, facing `(0, −1, 1)`, and the wall runs round from one to the
    /// other through `+y`. A leaning plane through the centre cuts the ring in
    /// two curves round the tube that each sweep through a range of angles
    /// round the axis, so a corner's side is no longer its whole curve's: the
    /// far cap's second curve, near `+x`, crosses the wall, and the part of the
    /// wall beyond it is behind the far cap's plane. Asked only whether each
    /// plane holds the centre, both corners are in front of the other plane,
    /// and `(4, 0, 1)` and `(−4, 0, −1)` — both on the wall — come back not
    /// held. No seam: nothing asked here reads one, and no curve a builder
    /// names runs from the near corner at the bottom of the tube to the far
    /// one at the top.
    #[test]
    fn a_bend_whose_far_cap_leans_is_refused_rather_than_losing_part_of_its_wall() {
        let through_the_centre = |facing| {
            Plane::from_point_and_normal(rcad_geom::Point::origin(), facing)
                .expect("a direction with a length")
        };
        let root_of_65 = 65.0_f64.sqrt();
        let (model, solid, wall) = a_piece_of_pipe(
            rcad_geom::Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring"),
            End {
                corner: rcad_geom::Point::new(32.0 / root_of_65, -4.0 / root_of_65, -1.0),
                cap: through_the_centre(Vector::new(-1.0, -8.0, 0.0)),
                used: Orientation::Forward,
            },
            End {
                corner: rcad_geom::Point::new(-(15.0_f64.sqrt()), 1.0, 1.0),
                cap: through_the_centre(Vector::new(0.0, -1.0, 1.0)),
                used: Orientation::Forward,
            },
            None,
        );
        let bend = model.solid(solid).expect("this model made the key");
        for point in [at(4.0, 0.0, 1.0), at(-4.0, 0.0, -1.0), at(0.0, 4.0, 1.0)] {
            let answer = bend.face_holds_the_point(
                wall,
                point,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            assert!(
                matches!(answer, Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face }) if face == wall),
                "{point:?}: {answer:?}"
            );
        }
    }

    /// A plane through a slanted axis holds it exactly where [`intersect`] cuts
    /// the ring in two circles, and nowhere else (§243.7, §328).
    ///
    /// Ring four and tube one round each of four slanted axes, cut by planes
    /// through the ring's centre whose normals are worked out square to the
    /// axis's stored direction `w`. For every one the interval dot of the
    /// stored normal and axis straddles zero, and for every one but the last
    /// the exact dot is zero: the normal's two slanted components are two of
    /// `w`'s swapped and one negated, and rescaling to length one rounds them
    /// the way it rounded `w`'s. `(w₂, 0, −w₀)` round `(1, 2, 3)` rescales to
    /// numbers whose products do not cancel, so that plane really does lean
    /// off the stored axis, and cuts the ring in a quartic.
    #[test]
    fn a_plane_through_a_slanted_axis_holds_it_exactly_where_intersect_names_two_circles() {
        type Square = fn([f64; 3]) -> [f64; 3];
        let across: Square = |w| [w[1], -w[0], 0.0];
        let round_x: Square = |w| [0.0, w[2], -w[1]];
        let round_y: Square = |w| [w[2], 0.0, -w[0]];
        let mut wrong = Vec::new();
        for (along, square, holds) in [
            ([0.6, 0.8, 0.0], across, Decision::Zero),
            ([1.0, 1.0, 0.0], across, Decision::Zero),
            ([0.0, 1.0, 1.0], round_x, Decision::Zero),
            ([1.0, 2.0, 3.0], across, Decision::Zero),
            ([1.0, 2.0, 3.0], round_x, Decision::Zero),
            ([1.0, 2.0, 3.0], round_y, Decision::Neg),
        ] {
            let axis = Axis::new(
                rcad_geom::Point::origin(),
                Vector::new(along[0], along[1], along[2]),
            )
            .expect("a direction with a length");
            let ring =
                rcad_geom::Torus::new(axis, 4.0, 1.0).expect("a tube narrower than its ring");
            let [x, y, z] = square(axis.direction().components());
            let flat =
                Plane::from_point_and_normal(rcad_geom::Point::origin(), Vector::new(x, y, z))
                    .expect("a normal with a length");
            let straddles = matches!(
                SquareToTheAxis {
                    named: WHERE_A_PLANE_AND_A_TORUS_CROSS,
                    normal: flat.normal().components(),
                    along: axis.direction().components(),
                }
                .on_intervals()
                .sign(),
                Decision::Undecidable { .. }
            );
            let lies = how_the_axis_lies_on(&flat, &ring);
            let crossed = intersect(
                &Surface::Plane(flat),
                &Surface::Torus(ring),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            let agrees = if holds == Decision::Zero {
                matches!(&crossed, Ok(SurfaceMeet::Curves(runs)) if runs.len() == 2)
            } else {
                matches!(
                    &crossed,
                    Err(rcad_geom::Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic)
                )
            };
            if !straddles || lies != holds || !agrees {
                wrong.push(format!(
                    "axis {along:?}, normal {:?}: straddles {straddles}, lies {lies:?}, \
                     intersect {crossed:?}",
                    flat.normal().components()
                ));
            }
        }
        assert!(wrong.is_empty(), "{wrong:#?}");
    }

    /// A bend's wall whose cap's plane is not exactly square to the ring's
    /// slanted axis is still refused, and there "no" is the true answer: that
    /// plane cuts the stored ring in a quartic, so the cap's circle is not one
    /// of its crossings (§328).
    ///
    /// The near cap is the leaning plane of the test above; the far one is a
    /// plane that does hold the axis. The corners are where the inner equator
    /// would cross each plane, to rounding, and nothing reads them before the
    /// near cap is turned away.
    #[test]
    fn a_wall_whose_cap_leans_off_a_slanted_axis_by_its_rounding_is_refused() {
        let axis = Axis::new(rcad_geom::Point::origin(), Vector::new(1.0, 2.0, 3.0))
            .expect("a direction with a length");
        let [w0, w1, w2] = axis.direction().components();
        let cap_square_to = |[x, y, z]: [f64; 3]| {
            let plane =
                Plane::from_point_and_normal(rcad_geom::Point::origin(), Vector::new(x, y, z))
                    .expect("a normal with a length");
            let [n0, n1, n2] = plane.normal().components();
            let corner = [w1 * n2 - w2 * n1, w2 * n0 - w0 * n2, w0 * n1 - w1 * n0].map(|c| 3.0 * c);
            (
                plane,
                rcad_geom::Point::new(corner[0], corner[1], corner[2]),
            )
        };
        let (leaning, near_corner) = cap_square_to([w2, 0.0, -w0]);
        let (holding, far_corner) = cap_square_to([0.0, w2, -w1]);
        let ring = rcad_geom::Torus::new(axis, 4.0, 1.0).expect("a tube narrower than its ring");
        assert_eq!(how_the_axis_lies_on(&leaning, &ring), Decision::Neg);
        assert_eq!(how_the_axis_lies_on(&holding, &ring), Decision::Zero);
        let (model, solid, wall) = a_piece_of_pipe(
            ring,
            End {
                corner: near_corner,
                cap: leaning,
                used: Orientation::Forward,
            },
            End {
                corner: far_corner,
                cap: holding,
                used: Orientation::Forward,
            },
            None,
        );
        let answer = model
            .solid(solid)
            .expect("this model made the key")
            .face_holds_the_point(
                wall,
                PredicatePoint::from(near_corner.coordinates()),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
        assert!(
            matches!(answer, Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face }) if face == wall),
            "{answer:?}"
        );
    }

    /// A normal exactly square to the axis through products below `2^-969` is
    /// refused as undecidable, where the answer is that the plane holds the
    /// axis: row Q328's family (§328).
    ///
    /// The normal is `(−0.8·2^-969, 1, 0.6·2^-969)` and the axis `(0.6, 0,
    /// 0.8)`, so the dot is `−0.8·0.6·2^-969 + 0.6·0.8·2^-969`, zero exactly.
    /// Neither product is an `f64`, so the interval straddles, and both are
    /// below `2^-969`, where [`product_is_exact`] turns them away although
    /// [`two_product`] holds each exactly: the lowest set bits of each pair of
    /// factors sit at `2^-1021` and `2^-53`, or `2^-1022` and `2^-52`, and add
    /// up to `2^-1074`. [`intersect`] refuses the same plane the same way,
    /// through `Exact::product_of`. When Q328 teaches `rcad-geom` that test,
    /// [`SquareToTheAxis`] wants it too, and this comes back
    /// [`Decision::Zero`].
    ///
    /// A tiny product that is an `f64` is no trouble: `(−2^-1000, 1, 2^-500)`
    /// against `(1, 0, 2^-500)` is settled by the interval, which rounds
    /// nothing, and never reaches the exact level.
    #[test]
    fn a_normal_square_to_the_axis_through_products_below_two_to_the_minus_969_is_refused() {
        let tiny = 2f64.powi(-969);
        let normal = [-0.8 * tiny, 1.0, 0.6 * tiny];
        let axis = Axis::new(rcad_geom::Point::origin(), Vector::new(0.6, 0.0, 0.8))
            .expect("a direction with a length");
        let flat = Plane::from_point_and_normal(
            rcad_geom::Point::origin(),
            Vector::new(normal[0], normal[1], normal[2]),
        )
        .expect("a normal with a length");
        assert_eq!(flat.normal().components(), normal, "stored as written");
        assert_eq!(
            axis.direction().components(),
            [0.6, 0.0, 0.8],
            "stored as written"
        );
        let ring = rcad_geom::Torus::new(axis, 4.0, 1.0).expect("a tube narrower than its ring");
        let lies = how_the_axis_lies_on(&flat, &ring);
        assert!(matches!(lies, Decision::Undecidable { .. }), "{lies:?}");
        let crossed = intersect(
            &Surface::Plane(flat),
            &Surface::Torus(ring),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert!(
            matches!(crossed, Err(rcad_geom::Error::Undecidable { .. })),
            "{crossed:?}"
        );

        let a_tiny_power_of_two = 2f64.powi(-500);
        let settled = how_the_axis_lies_on(
            &Plane::from_point_and_normal(
                rcad_geom::Point::origin(),
                Vector::new(
                    -(a_tiny_power_of_two * a_tiny_power_of_two),
                    1.0,
                    a_tiny_power_of_two,
                ),
            )
            .expect("a normal with a length"),
            &rcad_geom::Torus::new(
                Axis::new(
                    rcad_geom::Point::origin(),
                    Vector::new(1.0, 0.0, a_tiny_power_of_two),
                )
                .expect("a direction with a length"),
                4.0,
                1.0,
            )
            .expect("a tube narrower than its ring"),
        );
        assert_eq!(settled, Decision::Zero);
    }
}
