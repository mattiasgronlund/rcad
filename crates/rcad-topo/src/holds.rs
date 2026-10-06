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
//! A face is trimmed by the surfaces of the faces around it, or, at a circle
//! whose run names a surface neither face is on, by that one
//! ([`the_surface_an_edge_cuts`]). Where each of
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
//! **A flat face is the exception, because it has a polygon after all**: put
//! each arc back as its chord and the sides are one, and what the chord leaves
//! out is the piece between it and the arc, which is two signs to be in. So a
//! flat face whose sides are straight or arcs of a closed crossing, and which
//! nothing below follows, winds its own sides
//! ([`what_a_flat_face_of_chords_and_arcs_holds`], row Q1002), and counts a
//! round hole among them as a whole (row Q1165).
//!
//! **One family of arcs is followed now** (§167): a wedge of a ball, whose skin
//! and whose two half-discs are each bounded by half of a circle. Those faces are
//! still the part of their surface on one side of everything that trims them —
//! or, on the sphere, on one side of *any* of it — but only once that has been
//! shown of the face in hand, and the side is read off the way the wire runs
//! rather than off a corner, because every corner sits on every cut.

use rcad_geom::{
    EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Surface, SurfaceMeet, Vector,
    WHERE_A_PLANE_AND_A_PIPE_CROSS, WHERE_A_PLANE_AND_A_TORUS_CROSS,
    how_a_plane_leans_against_a_cone, intersect, where_a_cones_apex_stands_against_a_plane,
    which_two_surfaces_a_run_is_between,
};
use rcad_predicates::expansion::{
    difference_of_two_products, expansion_sign, expansion_sum, product_is_exact, scale_expansion,
    two_product,
};
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
            return what_a_face_cut_by_part_of_a_crossing_holds(self, face, point, policy, log);
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
        if is_round {
            match how_often_a_round_wire_winds_round(
                solid, face, position, wire, point, policy, log,
            )? {
                Some(count) => winding += count,
                None => return Ok(OnAFace::OnItsBoundary),
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

/// One round wire's contribution to a flat face's crossing count, or `None`
/// when `point` sits on it: `1` inside the outline, `-1` inside a hole, `0`
/// outside either. The first wire of a flat face is its outline and the rest
/// are holes, which is what `Model::validate` holds every flat face to.
///
/// # Errors
///
/// What [`what_a_round_wire_encloses`] gives.
fn how_often_a_round_wire_winds_round(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    position: usize,
    wire: WireView<'_>,
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<i64>> {
    let inwards = if position == 0 { 1 } else { -1 };
    Ok(
        match what_a_round_wire_encloses(solid, face, wire, point, policy, log)? {
            Decision::Neg => Some(inwards),
            Decision::Pos => Some(0),
            Decision::Zero => None,
            // A side nobody could settle came back as a refusal a moment ago,
            // so what is left is a `Decision` variant a later milestone added,
            // and guessing what it means is worse than stopping.
            other => panic!("a side came back with a decision nothing here knows: {other:?}"),
        },
    )
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
/// **Which circle the wire is, is read where the face is stored**, for §168's
/// reason. A turn rounds the face's plane and the ring each on its own, so a
/// plane through the ring's axis as stored lands a rounding off the turned
/// axis, and [`intersect`] then reads the two as stored once moved, rightly, as
/// a quartic with no circle to name. A movement keeps which circle a corner is
/// on, so the picker is named from the stored plane, ring and corner, and moved
/// into place; only `point` is asked where it sits, against the moved picker
/// and the moved ring, the same rounded surfaces the ring's own face is asked
/// against (row Q1168).
///
/// # Errors
///
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] when the wire is not one
/// whole closed crossing of two surfaces, or when its neighbour is a ring and
/// [`intersect`] names no one circle through the wire's corner: a plane that
/// rests on the tube and touches it in one circle has no picker to name it by.
/// The same when the ring's face is placed apart from this one, so the two
/// stored frames are not one.
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
    if partner.location() != face.location() {
        return Err(cannot());
    }
    let corner = where_it_is_stored(solid, coedge.start().key());
    let Some(this_circle) = the_picker_naming_the_circle_through(
        &face.stored_surface(),
        &partner.stored_surface(),
        PredicatePoint::from(corner.coordinates()),
        policy,
        log,
    )?
    else {
        return Err(cannot());
    };
    let this_circle = this_circle.transformed(&face.location());
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
/// `pub(crate)` so `validate.rs` tells a ring face's circles round the axis
/// apart by the same pipe this file tells its bands apart by (row Q1153).
///
/// # Errors
///
/// Whatever [`intersect`] gives for the two.
pub(crate) fn the_picker_naming_the_circle_through(
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
/// Q418 ran this with the `Zero` arm made a panic, over the whole workspace
/// and 159 points on and beside a bend's axis in
/// `a_bends_cap_holds_nothing_on_or_beside_the_rings_axis` and
/// `a_bend_whose_tube_all_but_reaches_its_axis_holds_only_its_own_side_of_it`
/// (`a_bends_cap_holds_one_of_the_two_disks.rs`): none of the 159 reached it.
/// The two workspace tests that did reach it hand it `(0, 4, 0)`, which is on
/// the plane *between* the two disks and off the cap's plane, so they test
/// the bullet above this one, not this one.
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
/// **Where no corner is off that surface, the wire says which side**, as it
/// does on a ring between two planes: a lune of a ball split down a great
/// circle has its one corner on the rim, and so has a cap cut off a ball along
/// a plane. A pointed cone's skin is read off its apex first, as before, and
/// every other such face off the way its wire walks the rim
/// ([`the_side_its_wire_keeps_along`], row Q1024).
///
/// A face with nothing trimming it is the whole surface and holds every point on
/// it — a ball's skin, whose one wire is a seam with the same face on both
/// sides.
///
/// # Errors
///
/// The three refusals [`SolidView::face_holds_the_point`] lists,
/// [`Error::CannotTellWhichWayAClosedEdgeRuns`] when the closed-edge rule
/// cannot, and whatever [`intersect`] makes of a pair of surfaces.
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
        return what_a_ring_face_between_two_planes_holds(solid, face, point, policy, log);
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
        let mut inwards = the_side_this_face_lies_on(face, &trimming)?;
        if inwards.is_none() && trimmed_once {
            inwards = the_side_the_apex_is_on(surface, &trimming)?;
        }
        if inwards.is_none() {
            inwards = the_side_its_wire_keeps_along(solid, face, &trimming)?;
        }
        let inwards = inwards
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
/// **Planes through the ring's axis are followed, and planes square to it.** A
/// plane through the axis cuts the ring in two circles round the tube,
/// each at one angle round the axis. With one of those circles from each plane
/// as its only edges, the face is one of the two stretches of ring between
/// them: the part of the ring in front of both planes when that stretch is
/// less than a half-turn, and in front of either when it is more. So:
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
///    the other is not a half. Nor is one whose two edges are both one of the
///    circles, whichever sides it keeps there, so the two corners must lie on
///    different circles ([`are_on_two_circles`]).
/// 3. **Otherwise each circle must lie strictly in front of the other plane,
///    or each strictly behind it.** A circle round the tube sits at one angle
///    round the axis, so its corner's side is the whole circle's. Both in
///    front, the stretch is less than a half-turn and is the part of the ring
///    in front of both planes. Both behind, it is more than a half-turn — a
///    three-quarter bend's wall — and is the part in front of *either* plane
///    (row Q1144). One in front and one behind, the sides kept at the two
///    circles name the two different stretches, and no face is both.
///
/// Then `point` is asked against the planes as they read once moved into
/// place. Everything else is asked of the face where it is stored, for §168's
/// reason: a movement keeps each of these facts and rounds the numbers they
/// are read from.
///
/// **A plane square to the axis that cuts the ring in two circles round the
/// axis** — the top half of a ring on `z = 0` — splits the ring less those
/// circles into two halves, one each side of it, and each half has both
/// circles as its edges. So a face whose only edges are those two circles is
/// one of the halves, and step 1's read says which, at each circle. Both
/// planes' normals must lie along the axis ([`lie_along_one_line`]), and the
/// two must be one plane, which [`intersect`] settles exactly
/// ([`are_one_plane`]), however each is stored (row Q950). Then step 2's rule
/// holds as it stands: the sides must agree once read against one normal — a
/// wire keeping one side at one circle and the other side at the other walks
/// its two circles the same way round, which no half's does — and the two
/// corners must lie on different circles. Then `point` is asked against that
/// plane alone. A plane that rests on the tube touches it along one circle,
/// and step 1 finds no way to read a side there (`docs/decisions.md` §243.8).
///
/// **Two different planes square to the axis leave two bands between them**,
/// an outer and an inner — a washer's rounded rim between `z = ±h` is the
/// outer one — so the planes' sides do not settle the face. Step 3 still
/// holds each circle in front of the other plane or each behind it, and then
/// the side of the pipe of the ring's own radius that [`intersect`] names both
/// circles by says which band ([`the_side_of_the_pipe_naming_both_circles`]).
/// Both in front, the face is that band (row Q950); both behind, it is the
/// ring less that band — a groove turned into a ring's rim leaves the ring
/// less its outer band — which is the part in front of either plane, and
/// between them the part off the band (row Q1144). `point` is asked against
/// both planes and that side of the pipe, all moved into place.
///
/// # Errors
///
/// [`Error::AFaceMissesPartOfWhereItsSurfacesCross`] when a plane neither
/// holds the ring's axis nor is square to it, when one plane is square to it
/// and the other is not, one circle is in front of the other plane and the
/// other is not, a circle is on the other plane, the wire keeps one plane's two
/// sides, both edges are along one of that plane's circles, or two different
/// planes square to the axis have one outer circle and one inner, which is what
/// the general rule gave every face of this shape. Whatever [`intersect`] gives
/// for two planes, or one plane and the
/// ring. [`Error::CannotTellWhichWayAClosedEdgeRuns`] when the closed-edge rule
/// cannot, and [`Error::Undecidable`] when a side, whether a plane holds the
/// axis, or which side of the pipe a point is on, could not be settled.
fn what_a_ring_face_between_two_planes_holds(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<OnAFace> {
    let refuse = || Error::AFaceMissesPartOfWhereItsSurfacesCross { face: face.key() };
    let ring = face.stored_surface();
    let Surface::Torus(torus) = ring else {
        panic!("only a face on a ring is sent here, and this one is on {ring:?}");
    };
    let mut cuts: Vec<Cut> = Vec::with_capacity(2);
    let mut square_to_the_axis = 0;
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
            Decision::Pos | Decision::Neg
                if lie_along_one_line(
                    flat.normal().components(),
                    torus.axis().direction().components(),
                ) =>
            {
                square_to_the_axis += 1;
            }
            Decision::Pos | Decision::Neg => return Err(refuse()),
            Decision::Undecidable { enclosure } => {
                return Err(gave_up_on_the_axis(flat, &torus, enclosure));
            }
            other => panic!("the axis came back with a decision nothing here knows: {other:?}"),
        }
        let names_it = which_two_surfaces_a_run_is_between(edge.stored_run(), [&ring, &cap]);
        if names_it != [ring, cap] {
            return Err(refuse());
        }
        let kept =
            the_side_its_wire_keeps(solid, face, across, coedge, names_it)?.ok_or_else(refuse)?;
        let corner = edge.start().key();
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
    let (Some(first), Some(second)) = (one.surface.as_plane(), other.surface.as_plane()) else {
        panic!(
            "a cut is pushed only for a plane, and these are {:?} and {:?}",
            one.surface, other.surface
        );
    };
    let one_plane = match square_to_the_axis {
        0 => the_normals_lie_along_one_line(first, second),
        2 => are_one_plane(&one.surface, &other.surface, policy, log)?,
        _ => return Err(refuse()),
    };
    if one_plane {
        if (one.kept == other.kept) != point_the_same_way(first, second)
            || !are_on_two_circles(solid, &ring, [one, other], policy, log)?
        {
            return Err(refuse());
        }
        return held_in_front_of_all(core::slice::from_ref(one), point);
    }
    let mut in_front = [false; 2];
    for (index, (circle, plane)) in [(one, other), (other, one)].into_iter().enumerate() {
        let at =
            PredicatePoint::from(where_it_is_stored(solid, circle.corners_on_it[0]).coordinates());
        in_front[index] = match side_of_surface(&plane.surface, at) {
            settled if settled == plane.kept => true,
            Decision::Pos | Decision::Neg => false,
            Decision::Zero => return Err(refuse()),
            Decision::Undecidable { enclosure } => {
                return Err(gave_up_on_a_side(&plane.surface, at, enclosure));
            }
            other => panic!("a side came back with a decision nothing here knows: {other:?}"),
        };
    }
    let in_front_of_both = match in_front {
        [true, true] => true,
        [false, false] => false,
        [true, false] | [false, true] => return Err(refuse()),
    };
    if square_to_the_axis == 0 {
        return if in_front_of_both {
            held_in_front_of_all(&cuts, point)
        } else {
            held_in_front_of_any(&cuts, point)
        };
    }
    let band = the_side_of_the_pipe_naming_both_circles(solid, &ring, [one, other], policy, log)?
        .ok_or_else(refuse)?
        .transformed(&face.location());
    if in_front_of_both {
        held_in_front_of_all_on_one_band(&cuts, &band, point)
    } else {
        held_in_front_of_either_or_off_the_band(&cuts, &band, point)
    }
}

/// Whether two planes square to one axis are one plane, stored the same way or
/// not: [`intersect`]'s own reading of two planes, which settles in exact
/// arithmetic whether their offsets stand in the ratio their normals do.
///
/// # Errors
///
/// Whatever [`intersect`] gives for the two planes.
///
/// # Panics
///
/// When the two cross in a line: both normals lie exactly along the ring's
/// axis, so they lie exactly along one line.
fn are_one_plane(
    one: &Surface,
    other: &Surface,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    match intersect(one, other, policy, log)? {
        SurfaceMeet::SameSurface => Ok(true),
        SurfaceMeet::Nothing => Ok(false),
        crossed @ (SurfaceMeet::Curves(_) | SurfaceMeet::TouchAtOnePoint { .. } | _) => {
            panic!("two planes square to one axis are parallel, and these meet in {crossed:?}")
        }
    }
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

/// Whether two cuts along one plane have their corners on the two different
/// circles that plane cuts `ring` in, and not both on one of them.
///
/// **The side each cut keeps does not settle this.** The ring less one of those
/// circles is one piece, which meets the circle from both sides, so no piece of
/// the ring keeps one side of the circle at both of two edges along it. A wire
/// that walks the circle twice, the same way round both times, keeps one side
/// at both, as a half does, and was read as a half (row Q790).
///
/// Each corner's circle is named by the picker [`intersect`] tells it from the
/// other circle by ([`the_picker_naming_the_circle_through`]). Both are named
/// on the first cut's plane as stored: the two cuts may be one plane stored
/// two ways, and two spellings of a plane can name its circles by pickers that
/// are not equal by value. `false` also when a corner is not certainly on one
/// circle, which leaves nothing to compare.
///
/// # Errors
///
/// Whatever [`intersect`] gives for the plane and the ring.
fn are_on_two_circles(
    solid: SolidView<'_>,
    ring: &Surface,
    cuts: [&Cut; 2],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    let flat = &cuts[0].surface;
    let mut circle_through = |cut: &Cut| {
        let corner = where_it_is_stored(solid, cut.corners_on_it[0]).coordinates();
        the_picker_naming_the_circle_through(flat, ring, PredicatePoint::from(corner), policy, log)
    };
    let one = circle_through(cuts[0])?;
    let other = circle_through(cuts[1])?;
    Ok(matches!((one, other), (Some(one), Some(other)) if one != other))
}

/// The side of the pipe of the ring's own radius that names the circle each of
/// two different planes square to the axis cuts a face along, when it names
/// both; `None` when it does not.
///
/// **Between two such planes the ring holds two bands**, and the pipe is what
/// tells them apart. Each plane cuts the ring in two circles round the axis, at
/// `R ± s` from it with `s > 0`, and [`intersect`] names the outer by the
/// outside of the pipe of radius `R` and the inner by its inside
/// ([`what_a_circle_round_the_axis_encloses`]). On the ring between the planes,
/// the outer band is strictly outside that pipe and the inner band strictly
/// inside it: the ring meets the pipe only at the top and the bottom of the
/// tube, and both planes cut the tube below its top and above its bottom.
///
/// So a face with one circle of each plane as its only edges, each circle in
/// front of the other plane, is the outer band when both circles are outer and
/// the inner band when both are inner. With one of each, no face keeps those
/// sides. The ring less the four circles is four pieces in a loop round the
/// tube: below both planes, the outer band, above both, the inner band. Leaving
/// an outer circle towards the other plane is entering the outer band, which
/// ends at the other plane's outer circle; past that circle is the piece beyond
/// the other plane, so the inner circle there is met from beyond the plane and
/// not from in front of it.
///
/// Each corner's circle is named on its own plane as stored. Both pickers are
/// built from the ring alone, so the same side of the pipe is equal by value
/// whichever plane named it.
///
/// # Errors
///
/// Whatever [`intersect`] gives for each plane and the ring.
fn the_side_of_the_pipe_naming_both_circles(
    solid: SolidView<'_>,
    ring: &Surface,
    cuts: [&Cut; 2],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<Picker>> {
    let mut circle_through = |cut: &Cut| {
        let corner = where_it_is_stored(solid, cut.corners_on_it[0]).coordinates();
        the_picker_naming_the_circle_through(
            &cut.surface,
            ring,
            PredicatePoint::from(corner),
            policy,
            log,
        )
    };
    let one = circle_through(cuts[0])?;
    let other = circle_through(cuts[1])?;
    Ok(one.filter(|one| other.as_ref() == Some(one)))
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
pub(crate) fn how_the_axis_lies_on(flat: &Plane, torus: &rcad_geom::Torus) -> Decision {
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

/// Which side of its trimming a curved face lies on when none of its corners
/// is off that trimming to say, read off the way its wire walks each of its
/// edges along it.
///
/// **This is a ball in two faces**: two lunes either side of a great circle,
/// or a cap cut off a ball along a plane and the rest of its skin. The rim
/// between them is written as where the ball crosses that plane, so the plane
/// is what trims each face ([`the_surface_an_edge_cuts`]), and each face's
/// only corner is on the rim. The closed-edge rule says which way a wire walks
/// a rim with its face's material on one side of the plane, so read backwards
/// it says which side the face keeps — the side the mesher draws it on, and
/// the one the rule that wrote the walk meant.
///
/// The caller has shown the face runs along every curve where its surface and
/// the trimming cross, so the face is one of the pieces those curves cut its
/// surface into, and that piece lies on one side of the trimming throughout.
/// What it keeps beside one edge it keeps everywhere, so every edge along the
/// trimming must say the same.
///
/// `None` when an edge along the trimming is where the two surfaces touch
/// rather than cross, which leaves no way round to read.
///
/// # Errors
///
/// [`Invalid::AFaceStraddlesSomethingTrimmingIt`] when two edges along the
/// trimming say different sides, which is the same broken model the corners
/// refuse when they disagree.
///
/// [`Error::CannotTellWhichWayAClosedEdgeRuns`] when the closed-edge rule
/// cannot, and whatever finding the face across an edge refuses.
fn the_side_its_wire_keeps_along(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    trimming: &Trimming,
) -> Result<Option<Decision>> {
    let mut found: Option<Decision> = None;
    for coedge in face.wires().flat_map(WireView::coedges) {
        let edge = coedge.edge();
        let Some(across) = the_face_across(solid, face, edge)? else {
            continue;
        };
        if the_surface_an_edge_cuts(face, across, edge) != trimming.surface {
            continue;
        }
        let names_it = which_two_surfaces_a_run_is_between(
            edge.stored_run(),
            [&face.stored_surface(), &across.stored_surface()],
        );
        let Some(kept) = the_side_its_wire_keeps(solid, face, across, coedge, names_it)? else {
            return Ok(None);
        };
        match found {
            None => found = Some(kept),
            Some(already) if already == kept => (),
            Some(_) => return Err(Invalid::AFaceStraddlesSomethingTrimmingIt.into()),
        }
    }
    Ok(found)
}

/// Which side of `names_it[1]` `face` keeps beside the closed edge `coedge`
/// walks, with `across` the face on its other side: the closed-edge rule
/// [`a_closed_crossing_is_walked_the_way_it_is_stored`], run backwards.
///
/// `names_it` is the pair [`which_two_surfaces_a_run_is_between`] names the
/// edge by, as `Model::validate` hands it to the rule. The side comes back
/// against `names_it[1]` as it is stored, so it is turned round when that
/// surface is the face across and the solid uses that face reversed, because
/// the rule reads the gradient each face is used with. A surface the run names
/// that no face is on is used as it is stored.
///
/// Everything is asked where the face is stored, for §168's reason: a movement
/// keeps which side a face keeps and rounds the numbers that side is read from.
///
/// `None` where the rule finds no way round to read, which is the two surfaces
/// touching rather than crossing at the edge's corner.
///
/// # Errors
///
/// [`Error::CannotTellWhichWayAClosedEdgeRuns`] when the rule cannot.
fn the_side_its_wire_keeps(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    across: FaceView<'_>,
    coedge: CoedgeView<'_>,
    names_it: [Surface; 2],
) -> Result<Option<Decision>> {
    let at = where_it_is_stored(solid, coedge.edge().start().key());
    let partner_turn = if names_it[1] == across.stored_surface() {
        across.orientation()
    } else {
        Orientation::Forward
    };
    let outward = [
        gradient_of(&face.stored_surface(), at) * face.orientation().sign(),
        gradient_of(&names_it[1], at) * partner_turn.sign(),
    ];
    let Some(forward_if_it_keeps_the_back) = a_closed_crossing_is_walked_the_way_it_is_stored(
        [&names_it[0], &names_it[1]],
        outward,
        at,
        Decision::Neg,
    )?
    else {
        return Ok(None);
    };
    // `Model::validate` hands the rule the side its corners read, turned
    // round when the partner's face is used reversed; this is that read
    // backwards.
    let walked_forward = coedge.orientation_on_its_edge() == Orientation::Forward;
    let keeps_the_back = walked_forward == forward_if_it_keeps_the_back;
    Ok(Some(if keeps_the_back != partner_turn.is_reversed() {
        Decision::Neg
    } else {
        Decision::Pos
    }))
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
/// [`every_reading_of_the_sides_of`].
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
        let asked = ends.map(|at| PredicatePoint::from(at.coordinates()));
        let Some(way) = the_way_into_an_arc(&surface, &across_surface, asked, picked_by) else {
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
///    the part in front of any of them. Or, on a cone cut by two planes through
///    its apex and by one cap, each side on one plane lies behind the other
///    plane, and the face is the part in front of the cap and of either plane:
///    three quarters of a pointed cone's skin, which
///    [`the_cap_and_the_two_cuts_a_cone_face_is_in_front_of_either_of`] argues
///    for.
///
/// Why that is enough: a face whose sides all lie in front of a neighbour
/// cannot reach behind it, because the region behind is connected, holds none of
/// the face's boundary, and borders the face's own side along that neighbour on
/// the side the face does not keep. So the face sits inside the part in front
/// of all of them; and that part is connected — a plane cut by half-planes and
/// round discs, or a sphere cut by planes through its centre or by any two
/// planes — with the face's boundary on its boundary, so the face is all of it.
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
/// it lies inside and crosses in a closed curve, a flat face that is a polygon
/// with one bite out of its edge where it lies outside a pipe
/// ([`is_a_polygon_less_one_bite_of_a_pipe`]), a sphere face whose sides are
/// arcs cut by planes through its centre or by two planes, and a cone face or a
/// pipe face cut by
/// planes as [`is_a_family_this_follows`] sets out. Each arc must be picked by a
/// half-space that tells it from its sibling ([`the_way_into_an_arc`]), which
/// asks the picker to hold both its ends exactly, or to have the arc's whole
/// curve in front of it with that curve's centre behind, or to have the middle
/// of the sibling behind it — and a cone's
/// ellipse and a pipe's circle earn that the same way a ball's circle does,
/// through [`where_a_closed_crossing_is_centred`]. Anything else is refused by
/// name, as before.
///
/// **One face is one of two pieces in front of every cut, and is told from the
/// other**: a wall of a bar with two flats, where the part of the pipe between
/// the flats is a run of rulings round each side
/// ([`one_of_two_runs_between_two_flats`]). Step 3 then holds of each piece on
/// its own, and the point is asked which half of the pipe it is in as well.
///
/// **A ruling between two patches of one pipe may be read against more than one
/// plane**, where no flat face of the solid holds it
/// ([`every_reading_of_the_sides_of`]). Each reading is taken in turn, and the
/// first whose steps prove the face to be the part in front of its cuts answers.
/// Each such proof is about the face in hand, so any reading that gets through
/// is a true one, and a reading that does not costs a refusal only when none
/// does.
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
/// # A flat face outside that family winds its own boundary instead
///
/// A flat face that keeps the outside of a pipe it crosses in a closed curve
/// can be one of two pieces in front of every cut, and then no reading of the
/// cuts tells it from the other piece: the cap of a slab with a round scoop out
/// of its top is one, and the strip past the far side of the pipe's circle is
/// in front of every cut the cap has. On a plane that question need never be
/// asked. [`what_a_flat_face_of_chords_and_arcs_holds`] winds the face's own
/// sides, with each arc read as its chord and the piece between the two added
/// or taken away, and that reads the face's boundary and not its cuts. So a
/// flat face takes it wherever the steps above say nothing
/// (`docs/decisions.md` §35, row Q1002).
///
/// **A flat face with a round wire takes it straight away** (row Q1165). Such
/// a wire, a round hole beside a tab's arc, is one whole closed curve with no
/// two ends, so it is no side the steps above can read
/// ([`every_reading_of_the_sides_of`] leaves it out), and the steps read
/// without it would take the hole for material. The winding counts it as a
/// whole, as [`what_a_flat_face_holds`] counts the round hole of a top with
/// straight sides. Before, every reading refused such a face, so no face the
/// steps answered is answered differently now.
///
/// # Errors
///
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] for a curved face outside
/// that family and for the shapes
/// [`what_a_flat_face_of_chords_and_arcs_holds`] refuses, and
/// [`Error::Undecidable`] when a sign could not be settled.
fn what_a_face_cut_by_part_of_a_crossing_holds(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<OnAFace> {
    let surface = face.stored_surface();
    let readings = every_reading_of_the_sides_of(solid, face)?;
    let with_a_round_wire = face
        .wires()
        .any(|wire| is_a_round_wire_of_a_flat_face(face, wire));
    if !with_a_round_wire {
        let mut refused = None;
        for sides in &readings {
            match what_the_part_in_front_of_its_cuts_holds(face, &surface, sides, point) {
                Ok(Some(held)) => return Ok(held),
                Ok(None) => {}
                Err(why) => {
                    refused.get_or_insert(why);
                }
            }
        }
        if let Some(why) = refused {
            return Err(why);
        }
    }
    match surface {
        Surface::Plane(flat) => {
            let [sides] = readings.as_slice() else {
                panic!(
                    "a flat face's sides are read one way, since only a ruling of a pipe is read more ways than one"
                );
            };
            what_a_flat_face_of_chords_and_arcs_holds(solid, face, &flat, sides, point, policy, log)
        }
        Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_) | _ => {
            Err(Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() })
        }
    }
}

/// Steps 1 to 3 of [`what_a_face_cut_by_part_of_a_crossing_holds`], or `None`
/// where the face is outside the family they follow.
fn what_the_part_in_front_of_its_cuts_holds(
    face: FaceView<'_>,
    surface: &Surface,
    sides: &[Side],
    point: PredicatePoint,
) -> Result<Option<OnAFace>> {
    let mut cuts: Vec<Cut> = Vec::new();
    for side in sides {
        let kept = which_side_of_its_neighbour_a_side_keeps(face, surface, side)?;
        let corners = [side.start.key, side.end.key];
        match cuts.iter_mut().find(|known| known.surface == side.across) {
            Some(known) if known.kept == kept => known.corners_on_it.extend(corners),
            Some(_) => return Ok(None),
            None => cuts.push(Cut {
                surface: side.across,
                placed: side.across_placed,
                kept,
                corners_on_it: corners.to_vec(),
            }),
        }
    }
    let one_of_two_runs = if is_a_family_this_follows(surface, &cuts) {
        None
    } else {
        let Some(run) = one_of_two_runs_between_two_flats(face, surface, sides, &cuts)? else {
            return Ok(None);
        };
        Some(run)
    };

    let placed = face.surface();
    let mut at_the_apex = false;
    if let Surface::Cone(cone) = placed {
        match is_on_the_nappe_the_face_is_on(face, &cone, point)? {
            Decision::Pos => (),
            Decision::Zero => at_the_apex = true,
            Decision::Neg => return Ok(Some(OnAFace::NotHeld)),
            Decision::Undecidable { enclosure } => {
                return Err(gave_up_on_a_side(&placed, point, enclosure));
            }
            other => panic!("a side came back with a decision nothing here knows: {other:?}"),
        }
    }
    let the_apex_is_a_corner = |held| {
        if at_the_apex && held == OnAFace::Held {
            OnAFace::OnItsBoundary
        } else {
            held
        }
    };

    if let Some(either) =
        the_cap_and_the_two_cuts_a_cone_face_is_in_front_of_either_of(surface, sides, &cuts)
    {
        let held = held_in_front_of_the_cap_and_of_either(&either, point)?;
        return Ok(Some(the_apex_is_a_corner(held)));
    }

    let mut in_front_of_all = true;
    let mut behind_all = true;
    for cut in &cuts {
        for side in sides.iter().filter(|side| side.across != cut.surface) {
            match where_a_side_lies(surface, side, cut) {
                Lies::On => {}
                Lies::Beside(found) if found == cut.kept => behind_all = false,
                Lies::Beside(_) => in_front_of_all = false,
                Lies::Across => return Ok(None),
            }
        }
    }
    if in_front_of_all {
        let held = match &one_of_two_runs {
            Some(run) => held_in_front_of_all_on_its_own_run(run, &cuts, point)?,
            None => held_in_front_of_all(&cuts, point)?,
        };
        Ok(Some(the_apex_is_a_corner(held)))
    } else if behind_all
        && (matches!(surface, Surface::Sphere(_)) || is_two_discs_taken_together(surface, &cuts))
    {
        held_in_front_of_any(&cuts, point).map(Some)
    } else {
        Ok(None)
    }
}

/// A cone face's cap, and the two planes through its apex it is in front of
/// either of ([`the_cap_and_the_two_cuts_a_cone_face_is_in_front_of_either_of`]).
struct InFrontOfEither {
    cap: Cut,
    either: [Cut; 2],
}

/// The cap and the two cuts of a cone face that is the part of its nappe in
/// front of the cap and in front of **either** of two planes through its apex;
/// `None` where the face is not shown to be that.
///
/// Three quarters of a pointed cone, cut down its axis by `x = 0` and `y = 0`
/// with the quarter `x, y > 0` taken away, have a skin like that. It keeps
/// `x ≤ 0` along its side on `x = 0` and `y ≤ 0` along its side on `y = 0`, and
/// the part of the nappe in front of both is the opposite quarter, which is not
/// the skin: its rim runs the long way round, across both planes. On a sphere
/// the way out is that every side lies behind every other neighbour, so the
/// rest of the sphere is the part in front of all of them turned round, and the
/// face the part in front of any. On a cone that is not true: past the cap the
/// rest of the nappe runs on for ever, and the sides up from the apex lie in
/// front of the cap, not behind it. So the cap is kept apart from the two
/// planes, and the face is asked to be in front of the one and of either of
/// the other two.
///
/// # What is asked
///
/// - The cuts are two planes `X` and `Y` through the apex that meet in a line
///   strictly inside the cone ([`do_two_planes_through_the_apex_meet_inside`]),
///   and one more plane, the cap, which the apex is strictly in front of and
///   which cuts the cone in a closed curve. Those are cuts
///   [`is_a_family_this_follows`] takes, so the face is on no run of
///   [`one_of_two_runs_between_two_flats`].
/// - Every side on `X` lies strictly behind `Y`, every side on `Y` strictly
///   behind `X`, and each of them in front of or on the cap.
/// - The cap carries one side, the rim: an arc with one end a corner of a side
///   on `X` and the other a corner of a side on `Y`, which leaves its end on
///   `X` into the side of `X` the face keeps
///   ([`the_side_an_arc_leaves_its_end_on`]).
///
/// # Why the face is then the part in front of the cap and of either plane
///
/// Every point of the nappe but the apex is on one ruling, and along a ruling a
/// plane through the apex keeps all of it or only the apex. Call `Q` the part of
/// the nappe strictly behind both `X` and `Y`. Turned round, the two planes
/// still meet in the same line inside the cone, so by
/// [`is_a_family_this_follows`]'s argument for two cuts through the apex the
/// rulings in `Q` are **one run**, and `Q` is that run with the apex left out:
/// one piece. The run ends at two rulings, one on each plane. `X` holds the
/// line the two meet in, strictly inside the cone, so it cuts this nappe in two
/// rulings with that line between them; inside `X`, `Y` changes sign only
/// across that line, so one of the two is behind `Y` and the other in front.
/// The run's end on `X` is the one behind `Y`, and the face's side on `X`,
/// which lies behind `Y`, lies along it. Its side on `Y` lies along the run's
/// other end the same way.
///
/// The face reaches neither past the cap nor into `Q`, by the argument
/// [`what_a_face_cut_by_part_of_a_crossing_holds`] gives for one cut: each is
/// one piece, holds none of the face's boundary, and borders a side of the face
/// on the side the face does not keep, so the face cannot reach into it without
/// filling it, and it does not fill it.
///
/// - Past the cap is one piece because the cap cuts every ruling of the nappe
///   once — it closes, with the apex in front — so that part is a band right
///   round the cone. The sides up from the apex lie in front of the cap, and the
///   rim on it, and the rim borders the band.
/// - `Q` holds no side on `X` or `Y`, which lie on those planes. It borders the
///   side on `X`: just across `X` from it is behind `X`, and behind `Y` as that
///   side is. And it holds no point of the rim, which is why the rim is asked
///   which way it leaves `X`. The cap's curve meets the rulings one for one, so
///   `Q` meets it in one open arc, ending on the two rulings `Q`'s run ends at,
///   and those are where the rim's two ends are. Between those two points the
///   curve is that arc or the rest of it; the rim leaves `X` into the face's
///   side of `X`, which is in front of it, so the rim is not the arc in `Q`, and
///   it is the rest.
///
/// So the face lies in the part `R` of the nappe on or in front of the cap and
/// on or in front of `X` or `Y`. `R` is one piece: the rulings it holds are the
/// whole circle of them less `Q`'s run, one run again, and each carries the
/// piece from the apex to the cap. The face's boundary is on `R`'s: past the
/// rim is past the cap, and just across a side on `X` or `Y` is `Q`. A face
/// inside one piece with its boundary on that piece's boundary is all of it, as
/// [`what_a_face_cut_by_part_of_a_crossing_holds`] says of the part in front of
/// every cut. Three quarters of a pointed cone are that
/// (`three_quarters_of_a_pointed_cone_hold_their_points` in
/// `tests/a_solid_carrying_a_run_along_a_cones_ruling.rs`).
///
/// # What is still refused
///
/// A second cut the apex is in front of, a cut it is behind, a third plane
/// through it, two planes that do not meet inside the cone, or more than one
/// side on the cap. Each is a gap and not a limit: none of them is shown to
/// leave the part in front of either in two pieces. The argument above leans on
/// one band past one cap and one rim across it, and a face that needs more is
/// the consumer `docs/decisions.md` §13 asks for.
fn the_cap_and_the_two_cuts_a_cone_face_is_in_front_of_either_of(
    surface: &Surface,
    sides: &[Side],
    cuts: &[Cut],
) -> Option<InFrontOfEither> {
    let Surface::Cone(cone) = surface else {
        return None;
    };
    let apex = PredicatePoint::from(cone.apex().coordinates());
    let mut through_the_apex = Vec::new();
    let mut caps = Vec::new();
    for cut in cuts {
        let flat = cut.surface.as_plane()?;
        let sits = flat.side_of_point(apex);
        if sits == Decision::Zero {
            through_the_apex.push((cut, flat));
        } else if sits == cut.kept {
            caps.push((cut, flat));
        } else {
            return None;
        }
    }
    let ([(one, one_flat), (other, other_flat)], [(cap, cap_flat)]) =
        (through_the_apex.as_slice(), caps.as_slice())
    else {
        return None;
    };
    if !do_two_planes_through_the_apex_meet_inside(cone, one_flat, other_flat)
        || cone.is_cut_in_a_closed_curve_by(cap_flat) != Some(true)
    {
        return None;
    }

    let in_front_of_or_on = |side: &Side, cut: &Cut| match where_a_side_lies(surface, side, cut) {
        Lies::On => true,
        Lies::Beside(found) => found == cut.kept,
        Lies::Across => false,
    };
    let strictly_behind = |side: &Side, cut: &Cut| match where_a_side_lies(surface, side, cut) {
        Lies::Beside(found) => found != cut.kept,
        Lies::On | Lies::Across => false,
    };
    let mut rims = Vec::new();
    for side in sides {
        let lies_as_it_must = if side.across == one.surface {
            strictly_behind(side, other) && in_front_of_or_on(side, cap)
        } else if side.across == other.surface {
            strictly_behind(side, one) && in_front_of_or_on(side, cap)
        } else if side.across == cap.surface {
            rims.push(side);
            true
        } else {
            panic!("every side is across one of the cuts it was read into");
        };
        if !lies_as_it_must {
            return None;
        }
    }

    let [rim] = rims.as_slice() else {
        return None;
    };
    let SideRun::Arc { into_the_arc } = &rim.run else {
        return None;
    };
    let on = |cut: &Cut, end: Corner| cut.corners_on_it.contains(&end.key);
    let end_on_one = if on(one, rim.start) && on(other, rim.end) {
        rim.start
    } else if on(one, rim.end) && on(other, rim.start) {
        rim.end
    } else {
        return None;
    };
    let leaves =
        the_side_an_arc_leaves_its_end_on(surface, rim, *into_the_arc, one_flat, end_on_one.at);
    (leaves == Some(one.kept)).then(|| InFrontOfEither {
        cap: (*cap).clone(),
        either: [(*one).clone(), (*other).clone()],
    })
}

/// Whether a flat face holds `point`, which is taken as being on that face's
/// plane, read off the face's own sides: each straight, or an arc of the closed
/// curve where the plane cuts a ball, a cone or a pipe, or of the parabola or
/// hyperbola where it cuts a cone.
///
/// **Each arc is read as its chord, and the piece between the two is added or
/// taken away.** The sides with every arc put back as its chord are a polygon,
/// counted as [`walk_one_outline`] counts one. What the chord leaves out is
/// the piece between the arc and its chord, and that piece is exactly the part
/// of the plane inside the curve and on the arc's side of the chord line: the
/// inside of a circle or an ellipse is convex, so the chord line meets it in the
/// chord, and a line meets the curve in two points, which are the arc's ends, so
/// the arc is all of the curve on its side of that line. So going round the arc
/// and back along the chord goes once round that piece, and the count round the
/// face's own sides is the polygon's count plus, for each arc, one round its
/// piece where the point is in it. Being in it is two signs: the round
/// surface's own, and the side of the chord line the point is on against the
/// side [`the_way_into_an_arc`] says the arc is on. Nothing is built.
///
/// **A parabola or a hyperbola's branch reads the same**, with the inside of
/// the curve the convex region `K` the branch bounds
/// ([`the_way_into_an_arc_that_does_not_close`] has why the chord line meets it
/// in the chord and the arc is all of the branch on its side). The cone's own
/// sign holds a second region on a hyperbola's plane, inside the other branch,
/// so a third sign is read there: which nappe the point is on
/// ([`is_on_the_other_nappe_from_the_arc`]).
///
/// **The arc goes round its piece the way the polygon goes round a piece on its
/// left exactly when the arc is on the chord's right.** Seen from the side the
/// plane's normal points to, going along the chord with the piece on its right
/// and back along it with the piece on its left is going round anticlockwise,
/// which is the way [`walk_one_outline`] counts `+1`. So an arc bulging out of a
/// polygon that goes round anticlockwise adds its piece, and one bulging into
/// it takes its piece away, and a hole's arcs do the same turned round.
///
/// **A round wire is counted as a whole**, through
/// [`how_often_a_round_wire_winds_round`]: once where its surface holds the
/// point, inwards for the outline and outwards for a hole, as
/// [`what_a_flat_face_holds`] counts one. `sides` leaves such wires out. The
/// count round the face is the sum of each wire's, so the two add.
///
/// **This reads the face's boundary, and never asks whether the part in front of
/// every cut is one piece.** That is why it answers the cap of a slab with a
/// round scoop out of its top, which keeps the outside of the pipe and is one of
/// two such pieces, and the two halves of a block's top a bar bored down into
/// it splits in two.
///
/// **A point on a chord is not on the face's boundary**, and the count of the
/// polygon is not defined there, though the count of the face is. Just off the
/// chord on the side away from the arc, the point is in no piece of that arc,
/// and the polygon's count there is the walk with that chord read as having the
/// point on that side of it — provided the half-line the walk runs crosses the
/// chord rather than running along it, so it is aimed at a corner off the
/// chord's line. On one side and the other of a chord the face's count is the
/// same, because the polygon's and the piece's each change by one there and
/// turned round; so that is the face's count at the point.
///
/// # Errors
///
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] when the point is on the
/// chords of two arcs at once, or on a chord with every corner of the face on
/// that chord's line: a gap, for a point the walk would need two nudges for, and
/// for a face no solid here has.
///
/// [`Error::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn`] when no point off
/// the plane can be found ([`a_point_off_the_plane`]), and [`Error::Undecidable`]
/// when a sign could not be settled.
fn what_a_flat_face_of_chords_and_arcs_holds(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    flat: &Plane,
    sides: &[Side],
    point: PredicatePoint,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<OnAFace> {
    let cannot = || Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() };
    let Surface::Plane(placed) = face.surface() else {
        unreachable!("a face stored on a plane sits on one once moved")
    };
    let target = a_point_off_the_plane(face, &placed)
        .ok_or(Error::NothingSaysWhichSideOfItsTrimmingAFaceLiesOn { face: face.key() })?;

    let mut winding = 0i64;
    for (position, wire) in face.wires().enumerate() {
        if !is_a_round_wire_of_a_flat_face(face, wire) {
            continue;
        }
        match how_often_a_round_wire_winds_round(solid, face, position, wire, point, policy, log)? {
            Some(count) => winding += count,
            None => return Ok(OnAFace::OnItsBoundary),
        }
    }
    let mut on_a_chord: Option<(usize, Decision)> = None;
    let mut along = Vec::with_capacity(sides.len());
    for (index, side) in sides.iter().enumerate() {
        let [from, to] = [side.start.placed, side.end.placed];
        let beside = decide(from, to, target, point, policy, log);
        if let Decision::Undecidable { enclosure } = beside {
            return Err(gave_up_turning_a_corner(point, enclosure));
        }
        along.push(beside);
        let SideRun::Arc { into_the_arc } = &side.run else {
            if beside == Decision::Zero {
                match lies_between(from, to, point) {
                    Ok(true) => return Ok(OnAFace::OnItsBoundary),
                    Ok(false) => (),
                    Err(overhang) => return Err(gave_up_ordering_along_a_side(point, overhang)),
                }
            }
            continue;
        };
        let on_the_left =
            is_the_arc_left_of_its_chord(flat, side, *into_the_arc).ok_or_else(cannot)?;
        let [arcs_side, away] = if on_the_left {
            [Decision::Pos, Decision::Neg]
        } else {
            [Decision::Neg, Decision::Pos]
        };
        let read = side_of_surface(&side.across_placed, point);
        let inside = if matches!(read, Decision::Neg | Decision::Zero)
            && is_on_the_other_nappe_from_the_arc(flat, side, point)?
        {
            Decision::Pos
        } else {
            read
        };
        match inside {
            Decision::Zero if beside == arcs_side || beside == Decision::Zero => {
                return Ok(OnAFace::OnItsBoundary);
            }
            Decision::Neg if beside == arcs_side => winding += if on_the_left { -1 } else { 1 },
            Decision::Neg if beside == Decision::Zero => {
                if on_a_chord.is_some() {
                    return Err(cannot());
                }
                on_a_chord = Some((index, away));
            }
            Decision::Zero | Decision::Neg | Decision::Pos => (),
            Decision::Undecidable { enclosure } => {
                return Err(gave_up_on_a_side(&side.across_placed, point, enclosure));
            }
            other => panic!("a side came back with a decision nothing here knows: {other:?}"),
        }
    }

    let mut corners = sides.iter().map(|side| side.start.placed);
    let aim = match on_a_chord {
        None => corners.next(),
        Some((chord, _)) => {
            let [from, to] = [sides[chord].start.placed, sides[chord].end.placed];
            corners.find(|corner| {
                matches!(
                    decide(
                        from,
                        to,
                        target,
                        PredicatePoint::from(corner.coordinates()),
                        policy,
                        log
                    ),
                    Decision::Pos | Decision::Neg
                )
            })
        }
    }
    .ok_or_else(cannot)?;

    for (index, side) in sides.iter().enumerate() {
        let along = match on_a_chord {
            Some((chord, away)) if chord == index => away,
            _ => along[index],
        };
        let [from, to] = [side.start.placed, side.end.placed].map(|corner| {
            if corner == aim {
                Decision::Zero
            } else {
                decide(corner, aim, target, point, policy, log)
            }
        });
        winding += crossing(from, to, along).ok_or_else(|| {
            gave_up_turning_a_corner(point, whichever_of_them_was_not_read(from, to))
        })?;
    }
    Ok(if winding == 0 {
        OnAFace::NotHeld
    } else {
        OnAFace::Held
    })
}

/// Whether an arc lies on the left of its chord, going from its start to its
/// end and looking down the normal of the plane it is on: `into_the_arc` is
/// [`the_way_into_an_arc`], which is that chord crossed with `flat`'s normal or
/// turned round, so the sign read here is that choice and no rounding can make
/// it otherwise. A movement keeps it, because a movement never mirrors. `None`
/// only when the chord has no length to read it by.
fn is_the_arc_left_of_its_chord(
    flat: &Plane,
    side: &Side,
    into_the_arc: [Interval; 3],
) -> Option<bool> {
    let [from, to] = [side.start.at, side.end.at].map(|corner| corner.coordinates());
    let chord =
        core::array::from_fn(|axis| Interval::exactly(to[axis]) - Interval::exactly(from[axis]));
    let left = cross(exactly(flat.normal().components()), chord);
    match dot(into_the_arc, left).sign() {
        Decision::Pos => Some(true),
        Decision::Neg => Some(false),
        Decision::Zero | Decision::Undecidable { .. } | _ => None,
    }
}

/// Whether `point`, on the plane `flat`, is on the other nappe of the cone
/// across `side` from the one the arc is on, where `flat` cuts that cone in a
/// curve that does not close. On a hyperbola's plane the inside of the cone is
/// a region on each nappe with nothing between them, and only the arc's own is
/// the piece between the arc and its chord
/// ([`the_way_into_an_arc_that_does_not_close`]). `false` for a side across
/// anything else, and for a point level with the apex, which is outside the
/// cone there because the plane misses the apex.
///
/// # Errors
///
/// [`Error::Undecidable`] when the point's nappe could not be settled.
fn is_on_the_other_nappe_from_the_arc(
    flat: &Plane,
    side: &Side,
    point: PredicatePoint,
) -> Result<bool> {
    let (Surface::Cone(cone), Surface::Cone(placed)) = (side.across, side.across_placed) else {
        return Ok(false);
    };
    if !matches!(
        how_a_plane_leans_against_a_cone(flat, &cone),
        Decision::Zero | Decision::Neg
    ) {
        return Ok(false);
    }
    let Some(arcs) = the_nappe_both_ends_are_on(&cone, [side.start.at, side.end.at]) else {
        return Ok(false);
    };
    let found = the_nappe_of(&placed, point.enclosure());
    if let Decision::Undecidable { enclosure } = found {
        return Err(gave_up_on_a_side(&side.across_placed, point, enclosure));
    }
    Ok(matches!(found, Decision::Pos | Decision::Neg) && found != arcs)
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
#[derive(Clone)]
struct Side {
    across: Surface,
    across_placed: Surface,
    start: Corner,
    end: Corner,
    run: SideRun,
}

/// A corner of a face, where it is stored and where it sits once moved.
#[derive(Clone, Copy)]
struct Corner {
    key: crate::VertexKey,
    at: Point,
    placed: Point,
}

/// What a [`Side`] runs along between its ends.
#[derive(Clone)]
enum SideRun {
    Straight,
    /// Part of a conic, carrying the one thing about it the steps below read:
    /// a direction square to the chord, in the conic's own plane, pointing from
    /// the chord into this arc rather than into its sibling
    /// ([`the_way_into_an_arc`]) or into the rest of its branch
    /// ([`the_way_into_an_arc_that_does_not_close`]).
    Arc {
        into_the_arc: [Interval; 3],
    },
}

/// One surface cutting a face: which side of it the face keeps, which of the
/// face's corners are on it because an edge along it ends there, and where it
/// is once moved into place, which is the only thing the point is asked against.
#[derive(Clone)]
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

/// Every way to read the sides of `face`, each one every side in the order its
/// wires walk them, seams left out, in the frame the face is stored in. One way
/// for every face but a pipe face with a ruling no flat face of the solid holds.
///
/// **A side with another face of the same surface across it is cut by
/// something else**, and that is what the side carries as `across`. The fuse
/// cuts a bar's wall along the whole of each crossing a plane of the other
/// solid makes with the pipe, so two patches of one pipe meet along an arc of
/// the circle where the block's top crosses it, or along a ruling where one of
/// its sides does, though the block is nowhere near (`docs/decisions.md` §167).
/// The pipe across says nothing about which side of that line the face keeps;
/// the plane it was cut along does, and a wire keeps its material on the left
/// of `∇S × ∇T` whatever face is across, so steps 1 to 3 of
/// [`what_a_face_cut_by_part_of_a_crossing_holds`] read it as a cut like any
/// other. An arc names that plane in its run's `and`, which the fuse writes for
/// exactly this. A ruling is stored straight and names nothing, so it is read
/// off a flat face of the solid that holds it ([`the_flat_face_a_ruling_lies_on`]).
///
/// **Where no flat face holds it, the ruling is read against each plane square
/// to a coordinate axis that does** ([`the_planes_square_to_an_axis_a_ruling_lies_on`]),
/// one reading for each. The union of a bar and a block notched into it keeps
/// none of the block's back face, `x = 2`, though the fuse cut the wall along
/// it at `y = ±√21`; the ruling there lies on `x = 2` and on `y = √21` as the
/// corners round it, and either is a plane along the axis crossing the pipe
/// through it, which is all [`the_flat_face_a_ruling_lies_on`]'s argument asks.
/// Which of the two the face is in front of all of is the steps' question:
/// tried first, `y = √21` gets six of those twelve patches through them, and
/// `x = 2` all twelve, so it takes the second reading to answer the same
/// whichever comes first. Where no such plane holds the ruling either, the side
/// is left as it was, which step 1 refuses.
///
/// **A flat face's round wires are left out too** ([`is_a_round_wire_of_a_flat_face`]):
/// a whole closed curve has no two ends to read a chord between, and
/// [`what_a_flat_face_of_chords_and_arcs_holds`] counts it as a whole instead.
fn every_reading_of_the_sides_of(
    solid: SolidView<'_>,
    face: FaceView<'_>,
) -> Result<Vec<Vec<Side>>> {
    let cannot = || Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() };
    let mut readings: Vec<Vec<Side>> = vec![Vec::new()];
    let coedges = face
        .wires()
        .filter(|&wire| !is_a_round_wire_of_a_flat_face(face, wire))
        .flat_map(WireView::coedges);
    for coedge in coedges {
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
            placed: corner.point(),
        });
        let one_surface = across.stored_surface() == face.stored_surface();
        let as_it_is = || vec![(across.stored_surface(), across.surface())];
        let read_against = match edge.stored_run() {
            EdgeRun::PartOfWhereItCrosses { and, .. } if one_surface => {
                vec![(*and, and.transformed(&face.location()))]
            }
            EdgeRun::Straight if one_surface => {
                match the_flat_face_a_ruling_lies_on(solid, face, [start.at, end.at]) {
                    Some(flat) => vec![(flat.stored_surface(), flat.surface())],
                    None => {
                        let planes =
                            the_planes_square_to_an_axis_a_ruling_lies_on(face, [start.at, end.at]);
                        if planes.is_empty() {
                            as_it_is()
                        } else {
                            planes
                                .into_iter()
                                .map(|flat| {
                                    let flat = Surface::Plane(flat);
                                    (flat, flat.transformed(&face.location()))
                                })
                                .collect()
                        }
                    }
                }
            }
            EdgeRun::Straight
            | EdgeRun::PartOfWhereItCrosses { .. }
            | EdgeRun::RightRoundWhereItsFacesCross
            | EdgeRun::RightRoundWhereItCrosses { .. }
            | _ => as_it_is(),
        };
        let mut ways = Vec::with_capacity(read_against.len());
        for (across_surface, across_placed) in read_against {
            let run = the_way_a_side_runs(solid, face, edge, [start, end], &across_surface)?;
            ways.push(Side {
                across: across_surface,
                across_placed,
                start,
                end,
                run,
            });
        }
        readings = readings
            .into_iter()
            .flat_map(|reading| {
                ways.iter().map(move |way| {
                    let mut widened = reading.clone();
                    widened.push(way.clone());
                    widened
                })
            })
            .collect();
    }
    Ok(readings)
}

/// Whether `wire` is one whole closed curve on one edge of a flat face: a round
/// hole through a plate, or a bar's round cap. Such a wire has no corner to
/// walk between, so [`what_a_flat_face_holds`] counts it through
/// [`how_often_a_round_wire_winds_round`], and so does
/// [`what_a_flat_face_of_chords_and_arcs_holds`] beside its other wires' arcs.
fn is_a_round_wire_of_a_flat_face(face: FaceView<'_>, wire: WireView<'_>) -> bool {
    face.stored_surface().as_plane().is_some() && wire.closes_on_one_edge()
}

/// What `edge`, a side of `face` between `ends` with `across` across it, runs
/// along.
///
/// # Errors
///
/// [`Error::AFaceTrimmedByACurveThisCannotFollow`] for an arc no half-space
/// tells from its sibling and that is not an arc of a cone's face along a curve
/// that does not close ([`the_way_into_an_arc_that_does_not_close`]), a whole
/// closed crossing among parts of others, or a kind of run this has not been
/// taught, and whatever [`the_face_across`] gives for the edge.
fn the_way_a_side_runs(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    edge: EdgeView<'_>,
    [start, end]: [Corner; 2],
    across: &Surface,
) -> Result<SideRun> {
    let cannot = || Error::AFaceTrimmedByACurveThisCannotFollow { face: face.key() };
    Ok(match edge.stored_run() {
        EdgeRun::Straight => SideRun::Straight,
        EdgeRun::PartOfWhereItCrosses { .. } if is_a_piece_of_a_ruling(solid, face, edge)? => {
            SideRun::Straight
        }
        EdgeRun::PartOfWhereItCrosses { picked_by, .. } if start.key != end.key => {
            let surface = face.stored_surface();
            let ends = [start.at, end.at];
            let into_the_arc = the_way_into_an_arc(
                &surface,
                across,
                ends.map(|at| PredicatePoint::from(at.coordinates())),
                picked_by,
            )
            .or_else(|| the_way_into_an_arc_that_does_not_close(&surface, across, ends));
            SideRun::Arc {
                into_the_arc: into_the_arc.ok_or_else(cannot)?,
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
    })
}

/// The way into an arc of the curve a plane cuts a cone in when that curve does
/// not close — a parabola, or one branch of a hyperbola — as
/// [`the_way_into_an_arc`] reads it of a curve that does: square to the chord,
/// in the plane, pointing from the chord into the arc. `None` unless one of
/// `surface` and `across` is the cone and the other the plane, the apex is
/// strictly off the plane, the curve does not close, and both ends are on one
/// nappe, each read exactly off the stored numbers.
///
/// **The ends alone say which arc this is.** Between two points of a curve that
/// does not close there is one arc that is bounded, and an edge is: the rest
/// of the branch is two pieces that each run off to infinity. So no picker is
/// asked, and the one the fuse writes — `z ≥ 0` for the branch on the upper
/// nappe — says only which branch, which the ends already do.
///
/// **That arc is on the side of the chord away from where the branch runs
/// off.** On the plane, the inside of the cone on the ends' nappe is a convex
/// region `K` bounded by the branch, and nowhere else does the plane meet that
/// nappe's inside. `K` runs off to infinity along the directions inside the
/// nappe that lie in the plane, and the axis direction projected into the
/// plane, `|n|²w − (n·w)n` turned towards that nappe, is one of them: of every
/// direction in the plane it is the one nearest the axis, and the plane holds
/// one within the cone's half angle of the axis because the curve does not
/// close — strictly within for a hyperbola, and for a parabola that one is the
/// only one. The line through the chord meets the branch only at the two ends,
/// a line meeting a conic in at most two points, so it crosses the branch there
/// and does not touch it, and leaves `K` in the chord alone. No direction `K`
/// runs off along lies along the chord: a hyperbola's asymptotes would meet its
/// branch once, not twice, and along any other such direction the chord would
/// run from one end into the inside of `K` and could not come out at the
/// other. So all of them, the projected axis with them, point across the
/// chord's line to one side, the side where `K` is unbounded. The two pieces
/// running off to infinity are on the boundary of `K` and off the line, so they
/// are on that side; the arc leaves each end on the other side from the piece
/// it meets there, so it is on the side away, where `K` is bounded.
///
/// The projected axis reads against `n × chord` as `|n|²·(w · (n × chord))`,
/// so the sign wanted is the sign of `w · (n × chord)` times the nappe's: one
/// determinant of stored numbers, in interval arithmetic, and a sign it cannot
/// settle is `None`.
///
/// **On a hyperbola's plane the inside of the cone is two regions**, one on
/// each nappe, and only `K` is the arc's: a flat face reads the piece between
/// an arc and its chord with the other one left out
/// ([`is_on_the_other_nappe_from_the_arc`]).
fn the_way_into_an_arc_that_does_not_close(
    surface: &Surface,
    across: &Surface,
    ends: [Point; 2],
) -> Option<[Interval; 3]> {
    let ((Surface::Cone(cone), Surface::Plane(flat)) | (Surface::Plane(flat), Surface::Cone(cone))) =
        (surface, across)
    else {
        return None;
    };
    let apex_is_off_the_plane = matches!(
        where_a_cones_apex_stands_against_a_plane(flat, cone),
        Decision::Pos | Decision::Neg
    );
    let does_not_close = matches!(
        how_a_plane_leans_against_a_cone(flat, cone),
        Decision::Zero | Decision::Neg
    );
    if !apex_is_off_the_plane || !does_not_close {
        return None;
    }
    let nappe_of_the_ends = the_nappe_both_ends_are_on(cone, ends)?;
    let [from, to] = ends.map(|end| exactly(end.coordinates()));
    let chord = core::array::from_fn(|axis| to[axis] - from[axis]);
    let square_to_the_chord = cross(exactly(flat.normal().components()), chord);
    let up = exactly(cone.axis().direction().components());
    let towards_the_axis = dot(up, square_to_the_chord).sign();
    if !matches!(towards_the_axis, Decision::Pos | Decision::Neg) {
        return None;
    }
    Some(if towards_the_axis == nappe_of_the_ends {
        square_to_the_chord.map(core::ops::Neg::neg)
    } else {
        square_to_the_chord
    })
}

/// [`Decision::Pos`] when both `ends` are strictly up `cone`'s axis from its
/// apex and [`Decision::Neg`] when both are strictly down it, read exactly off
/// the stored numbers; `None` otherwise.
fn the_nappe_both_ends_are_on(cone: &rcad_geom::Cone, ends: [Point; 2]) -> Option<Decision> {
    let [from, to] = ends.map(|end| the_nappe_of(cone, exactly(end.coordinates())));
    (matches!(from, Decision::Pos | Decision::Neg) && to == from).then_some(from)
}

/// Which way up `cone`'s axis from its apex `at` is, as a sign: the nappe it is
/// on, or [`Decision::Zero`] level with the apex.
fn the_nappe_of(cone: &rcad_geom::Cone, at: [Interval; 3]) -> Decision {
    let apex = exactly(cone.apex().coordinates());
    let up = exactly(cone.axis().direction().components());
    dot(core::array::from_fn(|axis| at[axis] - apex[axis]), up).sign()
}

/// A flat face of `solid` whose plane holds a straight side between two patches
/// of `face`'s pipe and crosses the pipe along it: a plane along the axis,
/// nearer to it than the radius, holding both `ends` exactly where they are
/// stored. `None` when `face` is not on a pipe, or no face of `solid` is that.
///
/// **Any such plane will do, and none has to be the one the fuse cut along.**
/// The side is on the pipe by topology and on the plane because its two ends
/// are, so it is part of where the two cross; the plane crosses rather than
/// touches there, so the face keeps one side of it near the side, and
/// [`which_side_of_its_neighbour_a_side_keeps`] reads which. Whether the face
/// is then the part of its pipe in front of every cut is what the rest of
/// [`what_a_face_cut_by_part_of_a_crossing_holds`] proves or refuses, for this
/// plane as for any other.
///
/// The ends are asked where they are stored, and only a plane that holds them
/// exactly is taken: the block's side at `y = −2` holds the corner at
/// `(√21, −2)` however `√21` rounds, where a plane built through the axis and
/// that corner would not.
fn the_flat_face_a_ruling_lies_on<'m>(
    solid: SolidView<'m>,
    face: FaceView<'m>,
    ends: [Point; 2],
) -> Option<FaceView<'m>> {
    let Surface::Cylinder(pipe) = face.stored_surface() else {
        return None;
    };
    solid.faces().find(|other| {
        let Surface::Plane(flat) = other.stored_surface() else {
            return false;
        };
        other.location() == face.location() && holds_a_ruling_of(&pipe, &flat, ends)
    })
}

/// Each plane square to a coordinate axis that holds a straight side between
/// two patches of `face`'s pipe as [`the_flat_face_a_ruling_lies_on`] asks a
/// face's plane to: along the axis, crossing the pipe, and holding both `ends`
/// exactly. Empty when `face` is not on a pipe.
///
/// **Each is built through the start with a normal of one `1.0` and two
/// zeros**, so its offset is the start's own coordinate and nothing rounds;
/// the end is on it exactly when it shares that coordinate, which the ends of a
/// ruling up a pipe standing square to that axis do as the fuse solves them.
/// Nothing else is built: a plane through the axis would round, and a corner a
/// hair off it is the one case the argument cannot have.
fn the_planes_square_to_an_axis_a_ruling_lies_on(
    face: FaceView<'_>,
    ends: [Point; 2],
) -> Vec<Plane> {
    let Surface::Cylinder(pipe) = face.stored_surface() else {
        return Vec::new();
    };
    [
        Vector::new(1.0, 0.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
        Vector::new(0.0, 0.0, 1.0),
    ]
    .into_iter()
    .filter_map(|normal| Plane::from_point_and_normal_as_written(ends[0], normal).ok())
    .filter(|flat| holds_a_ruling_of(&pipe, flat, ends))
    .collect()
}

/// Whether `flat` lies along `pipe`'s axis, crosses the pipe in two rulings,
/// and holds both `ends` exactly where they are stored.
fn holds_a_ruling_of(pipe: &rcad_geom::Cylinder, flat: &Plane, ends: [Point; 2]) -> bool {
    lies_along_the_axis(pipe, flat)
        && crosses_the_pipe_in_two_rulings(pipe, flat)
        && ends.iter().all(|end| {
            flat.side_of_point(PredicatePoint::from(end.coordinates())) == Decision::Zero
        })
}

/// Whether `flat`'s stored normal is exactly square to `pipe`'s stored axis.
fn lies_along_the_axis(pipe: &rcad_geom::Cylinder, flat: &Plane) -> bool {
    let (leans, _) = rcad_predicates::decide(&SquareToTheAxis {
        named: WHERE_A_PLANE_AND_A_PIPE_CROSS,
        normal: flat.normal().components(),
        along: pipe.axis().direction().components(),
    });
    leans == Decision::Zero
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
/// This is the one thing the three steps below read out of an arc, and the
/// first two ways below read it out of the picker's **normal** alone. Where the
/// picker's plane sits is not asked there, which is what lets a fused arc in:
/// the fuse lowers that plane
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
/// The picker has to settle which side, and it does so in one of three ways:
///
/// - **Its plane holds both ends**, which is §167's test and what a face built
///   by hand gives. That plane then meets the curve's plane in the chord line
///   itself, and the arc is in front, so the arc is on the side the normal
///   points to.
/// - **Both ends are on or in front of it and the curve's centre is on or
///   behind it** ([`where_the_curves_centre_sits`]), which is what a fused arc
///   gives when it is the shorter of the two.
/// - **The middle of the other arc is behind it**, which is what a fused arc
///   gives when it is the longer: the plane through its chord, lowered past both
///   ends, has the centre in front. The arc is then the one on the centre's side
///   of the chord, and [`into_the_arc_on_the_centres_side`] carries that
///   argument, which reads where the plane sits and not only its normal.
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
/// points from the chord into the arc. That sign is the only reading the first
/// two ways take of the picker; how nearly square to the chord its normal is
/// never comes into it, and neither does how far the fuse lowered its plane.
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
///
/// # Ends that are boxes
///
/// The fuse's arrangement asks this too, of the boxes it solved each end into
/// rather than of stored corners (`which_way_an_edge_runs_round` in
/// `fuse/arrange.rs`), so the two read an arc by one spelling and cannot come to
/// disagree (§23.8). Every sign above is then read over the whole of both
/// boxes, the chord and the direction square to it included, so a sign that
/// comes back certified is the sign at the true ends wherever in their boxes
/// they are, and one the boxes leave open is `None`. A stored corner is a box
/// with no width, and is read as the exact point it always was.
pub(crate) fn the_way_into_an_arc(
    surface: &Surface,
    across: &Surface,
    ends: [PredicatePoint; 2],
    picked_by: &[Picker],
) -> Option<[Interval; 3]> {
    let flat = surface.as_plane().or_else(|| across.as_plane())?;
    let centred = where_a_closed_crossing_is_centred(surface, flat)
        .or_else(|| where_a_closed_crossing_is_centred(across, flat))?;
    let boxes = ends.map(PredicatePoint::enclosure);
    let [from, to] = boxes;
    let chord = core::array::from_fn(|axis| to[axis] - from[axis]);
    let square_to_the_chord = cross(exactly(flat.normal().components()), chord);

    let mut halves = picked_by.iter().filter_map(|picker| match picker {
        Picker::InFrontOf(half) => Some(half),
        Picker::OutsideABall(_) | Picker::OutsideAPipe(_) | Picker::InsideAPipe(_) | _ => None,
    });
    let by_its_normal = halves.clone().find_map(|half| {
        if !a_picker_that_tells_the_two_arcs_apart(half, flat, &centred, ends) {
            return None;
        }
        match dot(exactly(half.normal().components()), square_to_the_chord).sign() {
            Decision::Pos => Some(square_to_the_chord),
            Decision::Neg => Some(square_to_the_chord.map(core::ops::Neg::neg)),
            Decision::Zero | Decision::Undecidable { .. } | _ => None,
        }
    });
    by_its_normal.or_else(|| {
        let round = if surface.as_plane().is_some() {
            across
        } else {
            surface
        };
        halves.find_map(|half| {
            into_the_arc_on_the_centres_side(
                half,
                flat,
                round,
                &centred,
                boxes,
                square_to_the_chord,
            )
        })
    })
}

/// Whether `half` says which of the two arcs between `ends` an edge is, by one
/// of the first two ways [`the_way_into_an_arc`] sets out.
///
/// The fuse's own arrangement asks this through [`the_way_into_an_arc`], on the
/// boxes it solved for rather than on stored corners, and so asks the third way,
/// [`into_the_arc_on_the_centres_side`], as well (§23.8, and §211.5 for the
/// three spellings of the closed-curve test that went the same way).
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

/// The way into an arc on the same side of its chord as the curve's centre,
/// when the middle of the **other** arc is certainly behind `half`; `None`
/// otherwise.
///
/// This is the third way [`the_way_into_an_arc`] reads an arc, and the one for
/// the longer of the two. The fuse picks an arc by a half-space through its
/// chord, lowered until both ends read on or in front
/// (`the_half_space_both_ends_are_on` in `fuse/assemble.rs`), and for the
/// longer arc the centre is then in front as well, so neither way above takes
/// it. What does: the arc lies wholly on or in front of `half`, because that is
/// what a picker is. A point of the curve strictly behind `half` is therefore
/// on the other arc. The part of the curve on the far side of the chord line
/// from the centre is one arc, since a line meets a closed conic in two points
/// and those are the ends. So if the curve has a point strictly behind `half`
/// on that far side, the arc picked is the one on the centre's side, and the
/// way into it is `square_to_the_chord` turned to read positive towards the
/// centre.
///
/// **The point asked is the far arc's middle**, `m = c + (M − c)/λ`: the curve
/// where the ray from the centre `c` through the chord's middle `M` leaves it.
/// `λ` is how far out `M` is, as a fraction of the way to the curve along that
/// ray. With `q` the round surface's own polynomial, negative inside, its
/// restriction to the plane is `q(c)` plus a form that is positive and has no
/// part linear in `p − c`, because `c` is the centre; so the curve is where
/// that form reaches `−q(c)`, and `λ² = (q(M) − q(c))/(−q(c))`. That holds of a
/// circle and of an ellipse alike, and nothing in it is a distance. `q(M)`
/// negative puts `M` strictly inside, so `λ < 1` and `m` is past `M`, on the far
/// side of the chord line.
///
/// With `f` the picker's own affine function, `f(m) = f(c) − (f(c) − f(M))/λ`,
/// so `f(m) < 0` follows from `Δ = f(c) − f(M) > 0` and `Δ² > λ²·f(c)²`, which
/// give `Δ > λ·|f(c)| ≥ λ·f(c)`. Every one of those is multiplied through by
/// powers of `n·along`, which is positive, so the centre is never formed and
/// no division is taken: the signs read are of polynomials in stored numbers
/// and the ends' boxes, in interval arithmetic, and an interval that straddles
/// zero is `None`.
///
/// **The arc's ends are somewhere in `ends`**: a stored corner as a box with no
/// width, the premise every other reading in this file makes of it, or the box
/// the fuse's arrangement solved a corner into. Every sign is read over the
/// whole of both boxes, so it is the sign at the true ends wherever in their
/// boxes they are. What is asked of the far arc is a long way from either end —
/// its middle — so an end off the curve by a rounding does not move it onto the
/// near arc.
fn into_the_arc_on_the_centres_side(
    half: &HalfSpace,
    flat: &Plane,
    round: &Surface,
    centred: &WhereTheCurveIsCentred,
    ends: [[Interval; 3]; 2],
    square_to_the_chord: [Interval; 3],
) -> Option<[Interval; 3]> {
    let normal = exactly(flat.normal().components());
    let from = exactly(centred.from.coordinates());
    let across = dot(normal, centred.along);
    if across.sign() != Decision::Pos {
        return None;
    }
    let up = Interval::exactly(flat.offset()) - dot(normal, from);
    let to_the_centre = centred.along.map(|step| step * up);
    let [start, end] = ends;
    let middle: [Interval; 3] =
        core::array::from_fn(|axis| (start[axis] + end[axis]) * Interval::exactly(0.5));

    let start_to_from: [Interval; 3] = core::array::from_fn(|axis| from[axis] - start[axis]);
    let into = match (dot(start_to_from, square_to_the_chord) * across
        + dot(to_the_centre, square_to_the_chord))
    .sign()
    {
        Decision::Pos => square_to_the_chord,
        Decision::Neg => square_to_the_chord.map(core::ops::Neg::neg),
        Decision::Zero | Decision::Undecidable { .. } | _ => return None,
    };

    let facing = exactly(half.normal().components());
    let offset = Interval::exactly(half.offset());
    let at_the_centre = (dot(facing, from) - offset) * across + up * dot(facing, centred.along);
    let at_the_middle = dot(facing, middle) - offset;
    let falls = at_the_centre - at_the_middle * across;
    let from_to_the_middle: [Interval; 3] = core::array::from_fn(|axis| middle[axis] - from[axis]);
    let inside_at_the_centre = how_far_outside(round, to_the_centre, across)?;
    let inside_at_the_middle = how_far_outside(round, from_to_the_middle, Interval::exactly(1.0))?;
    if falls.sign() != Decision::Pos
        || inside_at_the_centre.sign() != Decision::Neg
        || inside_at_the_middle.sign() != Decision::Neg
    {
        return None;
    }
    let behind = falls * falls * -inside_at_the_centre
        - (inside_at_the_middle * across * across - inside_at_the_centre)
            * at_the_centre
            * at_the_centre;
    (behind.sign() == Decision::Pos).then_some(into)
}

/// `k²·q(from + x/k)`, with `q` the polynomial `round`'s sign predicate reads —
/// negative inside — and `from` the point [`where_a_closed_crossing_is_centred`]
/// walks from: a ball's centre, a cone's apex, a pipe's axis origin. `None` for
/// a surface that has no closed crossing with a plane.
///
/// About that point `q` has no part linear in `x`, so this is its part of
/// degree two in `x` plus `q(from)·k²`, and `k` lets a point that is a ratio be
/// asked without dividing.
fn how_far_outside(round: &Surface, x: [Interval; 3], k: Interval) -> Option<Interval> {
    let squared = |u: [Interval; 3]| dot(u, u);
    match round {
        Surface::Sphere(ball) => {
            let radius = Interval::exactly(ball.radius());
            Some(squared(x) - radius * radius * k * k)
        }
        Surface::Cone(cone) => {
            let axis = exactly(cone.axis().direction().components());
            let slope = Interval::exactly(cone.slope());
            let up = dot(x, axis);
            Some(squared(cross(x, axis)) - slope * slope * up * up)
        }
        Surface::Cylinder(pipe) => {
            let axis = exactly(pipe.axis().direction().components());
            let radius = Interval::exactly(pipe.radius());
            Some(squared(cross(x, axis)) - radius * radius * squared(axis) * k * k)
        }
        Surface::Plane(_) | Surface::Torus(_) | _ => None,
    }
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
/// are not one closed curve for a chord to split in two. An arc of either
/// needs no centre, since its ends alone say which arc it is, and
/// [`the_way_into_an_arc_that_does_not_close`] reads it.
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
/// is not admitted by that argument: the outside of an ellipse is not convex,
/// which is the first of the cone's reasons below, and a block with a bar bored
/// down across the whole of its top keeps two pieces of that top outside the
/// bar (`a_block_with_a_bar_bored_into_it_has_each_half_of_its_top_hold_its_own_side`
/// in `tests/a_cap_trimmed_by_a_pipe_holds_its_points.rs`). It is admitted by
/// another, when the pipe's curve crosses the edge of the polygon the straight
/// cuts leave only at the two ends of the face's one arc
/// ([`is_a_polygon_less_one_bite_of_a_pipe`]): the floor of a block with a bar
/// bored through the middle of one edge. A flat face this says no to is not
/// refused for it: [`what_a_face_cut_by_part_of_a_crossing_holds`] hands it to
/// [`what_a_flat_face_of_chords_and_arcs_holds`], and both halves of that top
/// are answered there.
///
/// **Convexity does not need the crossing to close on a pipe**, where it does on
/// a cone: the inside of a pipe is every point within its radius of the axis,
/// which is convex, so every plane keeps a convex piece of it — a strip between
/// two rulings when the plane lies along the axis. Only the closed crossings are
/// admitted because only they are what the sides can be placed against:
/// [`where_a_side_lies`] reads a straight side as inside a pipe only on a plane
/// that crosses it in a closed curve, and [`the_way_into_an_arc`] reads an arc
/// only off a closed curve. A fuse stores a ruling as a straight side, so a flat
/// face along a bar's axis takes [`what_a_flat_face_holds`] instead and is
/// answered there.
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
/// **A face keeping the cone's *outside* is refused here, and that is this
/// argument's limit and no longer the kernel's gap.** `cut.kept == Decision::Neg`
/// is what says so, and deleting it moves nothing. Read this before reaching
/// for it again (`docs/decisions.md` §215.3 asked, and this is the answer):
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
///   (the case of
///   `a_corner_up_cube_of_side_six_turned_about_the_axis_round_a_pointed_cones_apex_answers_right`,
///   whose 9 326 decided grid points are the floor §215.3 named): deleting the
///   conjunct leaves 9 326, and deleting it *and* letting a straight side read
///   `Beside(Pos)` against a cone leaves 9 326 again, with all 13 187 refusals
///   moving from this gate to that `Lies::Across`.
///
/// So the family of fixes here is "loosen what this admits", and it is the only
/// one that keeps the shape of [`what_a_face_cut_by_part_of_a_crossing_holds`]'s
/// steps — they ask one question, "is `point` in front of every cut", and a
/// region in two pieces has no such answer. Following a kept outside that way
/// would mean deciding whether the leftover region is connected, which is a
/// predicate nobody here has, over where the ellipse meets the straight cuts.
///
/// **A flat face needs none of that, and since row Q1002 it does not ask.** The
/// cap of a slab with a round scoop out of its top keeps the outside of a pipe
/// and is one of two pieces in front of every cut, and it is the consumer that
/// §13 asked for. [`what_a_flat_face_of_chords_and_arcs_holds`] winds the face's
/// own sides, each arc read as its chord with the piece between the two added
/// or taken away, which is one sign of the round surface and one side of the
/// chord. It reads the face's boundary and not its cuts, so whether the part in
/// front of every cut is one piece never comes up, and it holds for the cone's
/// ellipse and the ball's circle as for the pipe's. So this still says no, which
/// is right of the question it answers, and a flat face it says no to is
/// answered that way instead: the slab `z = 6` above is one.
///
/// On a pipe it is planes, as [`leaves_a_pipe_in_one_piece`] sets out, or two
/// planes along the axis meeting inside the pipe in place of its one
/// ([`is_a_wedge_of_a_pipe`]). Two parallel planes along the axis leave two
/// pieces, so this says no to them, and the caller asks
/// [`one_of_two_runs_between_two_flats`] which piece the face is.
///
/// On a sphere, it is planes through the centre, or any two planes. Each plane
/// through the centre cuts the sphere into two halves, and any number of halves
/// meet in one piece. **Two planes leave one piece whatever they keep** (row
/// Q1162). They cut the sphere in two circles, which meet in at most two
/// points. Where they cross at two, the part in front of both has its edge on
/// the arc of each circle in front of the other plane, and those two arcs run
/// between the same two points: one closed curve, which cuts the sphere into
/// two pieces and no more. The part in front of both is one of them, since the
/// rest of either circle lies on neither. Where they do not cross, one circle
/// lies wholly on one side of the other plane, and the part in front of both is
/// a cap, a band or nothing. So a ball cut to `x ≤ 26`, `y ≤ 10`, both planes
/// well off its centre, has a skin that is the part in front of both
/// (`tests/a_rung_whose_ray_meets_a_face_nobody_follows.rs`). Turning which side
/// each keeps round does not touch that argument, so the part behind both, which
/// the sphere's "in front of any" reading needs as one piece, is one too.
/// **A third plane that misses the centre is refused, and that is a limit of
/// the argument and not a gap**: three caps lost can go right round a patch and
/// leave the part in front of all three in two pieces with nothing between
/// them (`three_planes_that_miss_a_balls_centre_leave_its_skin_in_two_pieces`
/// below), and telling the face's piece from the other wants where the circles
/// meet, which nothing here reads.
///
/// # On a cone it is planes the apex is in front of, any number of them, and one it is on or two that meet inside the cone
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
/// against each other, which is why this arm reads one sign per cut the apex is
/// in front of rather than looking for where two of them meet; only two cuts
/// through the apex are asked where they meet, below. §216.5's sketch settled a
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
/// **The same pinch happens on one nappe, and it is why two cuts through the
/// apex must meet inside the cone.** A plane through the apex has the value zero
/// there, so along each ruling it keeps all of it or only the apex, and which is
/// the sign of how fast it grows along that ruling — affine in `cos φ` and
/// `sin φ`, so one arc of the circle of rulings. One such arc is one piece. Two
/// can meet in two: keep `|x| ≤ z/10` on the cone of slope a half and the
/// rulings with `|cos φ| ≤ 1/5` are left, one run round `+y` and one round `−y`,
/// two sheets that meet only at the apex. A face on one of them is cut by both
/// planes, has every side in front of both, and the other sheet is in front of
/// both too, so `held_in_front_of_all` would hold its points — a wrong answer.
/// `two_planes_through_the_apex_leave_a_nappe_in_two_sheets` below walks those
/// rulings. Before `docs/decisions.md` §389 a cut through the apex came only
/// with a side stored as [`EdgeRun::Straight`] along a ruling, since a ruling
/// stored as part of a crossing was refused as an arc; since it, every ruling
/// side reaches this arm.
///
/// **Two cuts through the apex that meet in a line strictly inside the cone
/// keep one run, whichever wedge is kept and on either nappe.** Cut the nappe
/// square to its axis: the rulings are the rim of a disc, and a ruling carries
/// more than the apex exactly where its rim point is in front of both planes.
/// The line the planes meet in crosses that disc at a point `c` strictly inside
/// it, and each plane crosses it in a straight line through `c`, so the part in
/// front of both is a wedge with its corner at `c` and an angle under a half
/// turn — a fan of rays out of `c`. Each of those rays leaves the disc through
/// its rim exactly once, because `c` is strictly inside, so the rim points in the
/// wedge are one arc: one run of rulings. Cuts the apex is strictly in front of
/// only shorten each ruling's piece, never to nothing, so the region is still
/// star-shaped from the apex over that one run. A quarter of a pointed cone, cut
/// by `x = 0` and `y = 0` along its axis, is that
/// (`a_quarter_of_a_pointed_cone_holds_its_points` in
/// `tests/a_solid_carrying_a_run_along_a_cones_ruling.rs`).
/// [`do_two_planes_through_the_apex_meet_inside`] settles where the line is.
/// The other three quarters have their skin cut by the same planes and keep
/// the part in front of either, not of both; this says yes to their cuts too,
/// and [`the_cap_and_the_two_cuts_a_cone_face_is_in_front_of_either_of`] tells
/// that skin from the quarter's, by the wedge turned round being one run as well.
///
/// **The strip `|x| ≤ z/10` is refused because of its shape**, not for want of
/// arithmetic: its planes meet along `y`, square to the axis and outside the
/// cone, so there is no corner `c` in the disc, the two planes cross the section
/// in two lines side by side, and the strip between them cuts the rim in two
/// arcs. A line outside the cone that does cross the section crosses it outside
/// the disc, and a wedge out of a point outside a disc can cut its rim in two
/// arcs the same way. A line along a ruling, or two planes that are one, is
/// refused as well: the argument above wants `c` strictly inside.
///
/// **Two cuts through the apex beside a cut the apex is behind, and three or
/// more through it, are refused**, and that is a gap and not a limit: neither is
/// shown to be two pieces. With a third plane through the apex the part of the
/// section in front of all three is no longer a fan out of one point unless all
/// three share one line, and with a cut the apex is behind each ruling's piece no
/// longer starts at the apex, so neither argument above reaches it.
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
/// a gap even when every cut closes.** Each such cut gives its own arc and two
/// arcs can meet in two:
/// `one_plane_behind_the_apex_and_two_in_front_leave_a_nappe_in_two_pieces`
/// below is a floor that cuts a circle and two roofs that each cut an ellipse,
/// whose region is two pieces round `±90°` with nothing along `±x`. Telling that
/// apart from a three-cut region that is one piece wants the arcs' ends compared,
/// which is where two roofs meet on the cone — a part with no named consumer
/// (§13); the lens, the one face that asked, has two cuts.
///
/// **Two cuts the apex is behind are refused, and that is a gap and not a
/// limit.** There the region really can be two pieces. On the cone of slope one
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
        Surface::Plane(flat) if is_a_polygon_less_one_bite_of_a_pipe(flat, cuts) => true,
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
            let all_planes = cuts.iter().all(|cut| cut.surface.as_plane().is_some());
            all_planes
                && (cuts.len() <= 2
                    || cuts.iter().all(|cut| {
                        cut.surface
                            .as_plane()
                            .is_some_and(|flat| flat.side_of_point(centre) == Decision::Zero)
                    }))
        }
        Surface::Cone(cone) => {
            let apex = PredicatePoint::from(cone.apex().coordinates());
            let mut behind = Vec::new();
            let mut through_the_apex = Vec::new();
            for cut in cuts {
                let Some(flat) = cut.surface.as_plane() else {
                    return false;
                };
                let sits = flat.side_of_point(apex);
                if sits == Decision::Zero {
                    through_the_apex.push(flat);
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
            match (through_the_apex.as_slice(), behind.as_slice()) {
                ([] | [_], []) => true,
                ([] | [_], [flat]) => {
                    let not_side_by_side = cuts
                        .iter()
                        .filter_map(|cut| cut.surface.as_plane())
                        .filter(|other| !the_normals_lie_along_one_line(flat, other))
                        .count();
                    not_side_by_side <= 1 && cone.is_cut_in_a_closed_curve_by(flat) == Some(true)
                }
                ([one, other], []) => do_two_planes_through_the_apex_meet_inside(cone, one, other),
                ([] | [_], [_, _, ..]) | ([_, _], [_, ..]) | ([_, _, _, ..], _) => false,
            }
        }
        Surface::Cylinder(pipe) => {
            leaves_a_pipe_in_one_piece(pipe, cuts) || is_a_wedge_of_a_pipe(pipe, cuts)
        }
        Surface::Torus(_) | _ => false,
    }
}

/// Whether the line two planes through `cone`'s apex meet in lies **strictly
/// inside** the cone; `false` when it lies on it or outside it, when the two are
/// one plane, and when the sign could not be settled.
///
/// Both planes hold the apex, so the line runs through it along `m = n₁ × n₂`,
/// the cross product of the two stored normals. The cone's own sign at the
/// apex plus `m` is `|m × w|² − s²(m·w)²` for `w` the stored axis direction and
/// `s` the slope — negative inside, as [`side_of_surface`] reads a cone — and
/// asking it of `m` rather than of a point builds nothing that rounds.
/// [`WhereTwoPlanesThroughTheApexMeet`] settles it through the cascade, exactly
/// when the interval cannot. It is even in `m`, so it says nothing of which nappe
/// and needs to say nothing: the line runs through both, one ray in each.
///
/// Only [`Decision::Neg`] is followed. [`Decision::Zero`] is a line along a
/// ruling, or `m` nothing at all, which is two planes that are one;
/// [`Decision::Pos`] is a line outside the cone, the strip's two sheets in
/// `two_planes_through_the_apex_leave_a_nappe_in_two_sheets`. An
/// [`Decision::Undecidable`] sign costs a refusal, never a guess.
fn do_two_planes_through_the_apex_meet_inside(
    cone: &rcad_geom::Cone,
    one: &Plane,
    other: &Plane,
) -> bool {
    let (sign, _) = rcad_predicates::decide(&WhereTwoPlanesThroughTheApexMeet {
        one: one.normal().components(),
        other: other.normal().components(),
        along: cone.axis().direction().components(),
        slope: cone.slope(),
    });
    match sign {
        Decision::Neg => true,
        Decision::Zero | Decision::Pos | Decision::Undecidable { .. } => false,
        other => panic!("a sign came back with a decision nothing here knows: {other:?}"),
    }
}

/// The cone's own sign along the line two planes through its apex meet in:
/// `|m × w|² − (s·(m·w))²` for `m = n₁ × n₂`, the slope taken inside the
/// square as `how_a_plane_leans_against_a_cone` takes it.
///
/// Degree eight in the stored numbers. The exact level carries every degree of
/// it in expansions and gives up, as [`SquareToTheAxis`] does, where a product
/// would round or a sum overflow.
struct WhereTwoPlanesThroughTheApexMeet {
    one: [f64; 3],
    other: [f64; 3],
    along: [f64; 3],
    slope: f64,
}

impl Predicate for WhereTwoPlanesThroughTheApexMeet {
    fn name(&self) -> &'static str {
        WHERE_TWO_PLANES_THROUGH_A_CONES_APEX_MEET
    }

    fn on_floats(&self) -> Option<Filtered> {
        None
    }

    fn on_intervals(&self) -> Interval {
        let meet = cross(exactly(self.one), exactly(self.other));
        let along = exactly(self.along);
        let sideways = cross(meet, along);
        let up = Interval::exactly(self.slope) * dot(meet, along);
        dot(sideways, sideways) - up * up
    }

    fn exactly(&self) -> Option<Decision> {
        let (one, other, along) = (self.one, self.other, self.along);
        let mut meet: [Vec<f64>; 3] = Default::default();
        for (axis, component) in meet.iter_mut().enumerate() {
            let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
            let (entries, used) =
                difference_of_two_products(one[next], other[after], one[after], other[next])?;
            *component = entries[..used].to_vec();
        }
        let mut total = Vec::new();
        let mut up = Vec::new();
        for axis in 0..3 {
            let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
            let sideways = added(
                &scaled(&meet[next], along[after])?,
                &scaled(&meet[after], -along[next])?,
            );
            total = added(&total, &multiplied(&sideways, &sideways)?);
            up = added(&up, &scaled(&meet[axis], along[axis])?);
        }
        let up = scaled(&up, self.slope)?;
        let squared: Vec<f64> = multiplied(&up, &up)?.iter().map(|entry| -entry).collect();
        match expansion_sign(&added(&total, &squared)) {
            Decision::Neg => Some(Decision::Neg),
            Decision::Zero => Some(Decision::Zero),
            Decision::Pos => Some(Decision::Pos),
            Decision::Undecidable { .. } => None,
            other => {
                panic!("exact arithmetic came back with a decision nothing here knows: {other:?}")
            }
        }
    }
}

/// The name the cone's sign along where two planes through its apex meet goes
/// by in provenance records.
const WHERE_TWO_PLANES_THROUGH_A_CONES_APEX_MEET: &str =
    "where two planes through a cone's apex meet";

/// `expansion` times `factor`, exactly; `None` where one of the products would
/// round ([`product_is_exact`]).
fn scaled(expansion: &[f64], factor: f64) -> Option<Vec<f64>> {
    if !expansion
        .iter()
        .all(|&entry| product_is_exact(entry, factor))
    {
        return None;
    }
    let mut out = vec![0.0; 2 * expansion.len()];
    let used = scale_expansion(expansion, factor, &mut out);
    out.truncate(used);
    Some(out)
}

/// Two expansions multiplied out, exactly: every entry of one scaling the
/// other, all added up. `None` where a product would round.
fn multiplied(left: &[f64], right: &[f64]) -> Option<Vec<f64>> {
    right.iter().try_fold(Vec::new(), |total, &factor| {
        Some(added(&total, &scaled(left, factor)?))
    })
}

fn added(left: &[f64], right: &[f64]) -> Vec<f64> {
    let mut sum = vec![0.0; left.len() + right.len()];
    let used = expansion_sum(left, right, &mut sum);
    sum.truncate(used);
    sum
}

/// Whether the part of `pipe` in front of every cut is one piece because two
/// of the cuts are planes along the axis that meet **strictly inside** the
/// pipe, and the others are what [`leaves_a_pipe_in_one_piece`] admits beside
/// one plane along the axis. A quarter of a bar's wall is that: `x ≥ 0` and
/// `y ≥ 0` meet along the axis.
///
/// **Two such planes keep one run of rulings.** Look down the axis: the pipe is
/// a circle, each plane a line, and what both keep is a wedge whose point is
/// where the two lines meet, its angle less than a half turn. From a point
/// strictly inside a circle every ray leaves the circle exactly once, so the
/// rulings the wedge keeps are the ones whose direction from that point lies in
/// the wedge's angle: one run. One run is all [`leaves_a_pipe_in_one_piece`]
/// reads off a single plane along the axis, so its argument for the floor and
/// the roof carries over word for word, and it is asked with one of the two
/// planes left out.
///
/// **Where they meet on or outside the pipe is refused.** The wedge can then
/// keep two runs, and the face is only one of them, so "in front of every cut"
/// would hold a point on the other. Two parallel planes are the extreme case: a
/// bar with two flats, `|x| ≤ 3` across a bar of radius 5, keeps a run round
/// `+y` and a run round `−y`, and each is a wall of its own. Those are answered
/// by [`one_of_two_runs_between_two_flats`], which tells the two runs apart by
/// the plane through the axis square to the flats. Two planes that are not
/// parallel and meet outside the pipe are still refused by name. Their face's
/// answer exists too, so that is a gap, and nothing here tells their two runs
/// apart.
///
/// A cut that is a second pipe running the same way is left to
/// [`leaves_a_pipe_in_one_piece`], which counts it beside the plane kept here
/// and refuses the two.
fn is_a_wedge_of_a_pipe(pipe: &rcad_geom::Cylinder, cuts: &[Cut]) -> bool {
    let axis = pipe.axis().direction().components();
    let along_the_axis: Vec<(usize, &Plane)> = cuts
        .iter()
        .enumerate()
        .filter_map(|(at, cut)| Some((at, cut.surface.as_plane()?)))
        .filter(|(_, flat)| {
            let (leans, _) = rcad_predicates::decide(&SquareToTheAxis {
                named: WHERE_A_PLANE_AND_A_PIPE_CROSS,
                normal: flat.normal().components(),
                along: axis,
            });
            leans == Decision::Zero
        })
        .collect();
    let [(_, one_plane), (other, other_plane)] = along_the_axis[..] else {
        return false;
    };
    if !do_two_planes_along_the_axis_meet_inside(pipe, one_plane, other_plane) {
        return false;
    }
    let all_but_one: Vec<Cut> = cuts
        .iter()
        .enumerate()
        .filter(|&(at, _)| at != other)
        .map(|(_, cut)| Cut {
            surface: cut.surface,
            placed: cut.placed,
            kept: cut.kept,
            corners_on_it: cut.corners_on_it.clone(),
        })
        .collect();
    leaves_a_pipe_in_one_piece(pipe, &all_but_one)
}

/// Whether the line two planes along `pipe`'s axis meet in lies strictly inside
/// the pipe; `false` when they are parallel, and when the sign could not be
/// settled.
///
/// With `nᵢ·x = dᵢ` the planes, `o` the axis origin, `eᵢ = dᵢ − nᵢ·o` and
/// `m = n₁ × n₂`, the point `o + U/|m|²` with `U = e₁(n₂ × m) + e₂(m × n₁)` is
/// on both, because `nᵢ·U = eᵢ|m|²` by the triple product. Both normals are
/// square to the axis, so `m` lies along it and `U`, square to `m`, is square
/// to the axis too: the line is `|U|/|m|²` from the axis everywhere. So it is
/// strictly inside exactly when `|U|² < r²|m|⁴`, a sign of stored numbers with
/// no division. Parallel planes have `m = 0`, both sides read zero, and that is
/// refused.
fn do_two_planes_along_the_axis_meet_inside(
    pipe: &rcad_geom::Cylinder,
    one: &Plane,
    other: &Plane,
) -> bool {
    let origin = exactly(pipe.axis().origin().coordinates());
    let [one_normal, other_normal] = [one, other].map(|flat| exactly(flat.normal().components()));
    let [one_offset, other_offset] = [(one, one_normal), (other, other_normal)]
        .map(|(flat, normal)| Interval::exactly(flat.offset()) - dot(normal, origin));
    let meet = cross(one_normal, other_normal);
    let [towards_one, towards_other] = [cross(other_normal, meet), cross(meet, one_normal)];
    let off_the_axis: [Interval; 3] = core::array::from_fn(|axis| {
        one_offset * towards_one[axis] + other_offset * towards_other[axis]
    });
    let radius = Interval::exactly(pipe.radius());
    let squared = dot(meet, meet);
    (dot(off_the_axis, off_the_axis) - radius * radius * squared * squared).sign() == Decision::Neg
}

/// A pipe face that is one of the two runs of rulings two parallel flats keep
/// between them, moved into place, with the half of the pipe it is in.
struct OneOfTwoRuns {
    pipe: rcad_geom::Cylinder,
    flat: Plane,
    half: Decision,
}

/// Whether `face`, on the pipe `surface`, is one of the two runs of rulings that
/// two parallel planes along the axis keep between them — a wall of a bar with
/// two flats — and if it is, which half of the pipe it is in. `None` when it is
/// not, which the caller refuses by name.
///
/// **Look down the axis.** The pipe is a circle of radius `r` and each flat is a
/// line. Take `n̂` across the flats and `t̂` along them, so the ruling at angle `φ`
/// is at `r(cos φ n̂ + sin φ t̂)` from the axis. The flats keep the rulings with
/// `a ≤ r cos φ ≤ b`, and when each flat is less than `r` from the axis
/// ([`crosses_the_pipe_in_two_rulings`]) both `|a|` and `|b|` are below `r`. So
/// `cos φ` is never `±1` there, `sin φ` is never zero, and the rulings kept are
/// two runs: one where `sin φ > 0` and one where `sin φ < 0`. The plane through
/// the axis square to the flats is `t̂·(x − o) = 0`, which is `sin φ = 0`, so it
/// has one run strictly on each side of it — **the two halves of the pipe**.
///
/// **Each half holds one run**, and one run is all [`leaves_a_pipe_in_one_piece`]
/// reads off one plane along the axis, so its argument for the floor and the
/// roof carries over and it is asked with one flat left out, as
/// [`is_a_wedge_of_a_pipe`] asks it. So the part of the pipe in front of every
/// cut is two pieces, one in each half, and step 3 of
/// [`what_a_face_cut_by_part_of_a_crossing_holds`] holds of each on its own: the
/// face sits inside the part in front of every cut, it is connected so it sits
/// inside one piece, and its boundary is on that piece's boundary, so it is all
/// of that piece. Which piece is the half its corners are in. Every corner reads
/// it where it is stored, and corners that do not all say one half are refused.
///
/// **The flats must keep the strip between them** ([`keep_the_strip_between_them`]).
/// Two parallel planes both keeping `+x` keep the part beyond one of them, which
/// is one run round `−x` — and that run crosses the plane through the axis, so
/// telling halves apart would cut the face in two.
///
/// **This, and not the plane through the wall's own two rulings**, because the
/// plane through the axis is built of stored numbers only: the pipe's axis and
/// one flat's normal. The rulings' plane is only through the rulings when the
/// corners it would be built from are exactly on the pipe — on the bar of radius
/// 5 with flats at `|x| ≤ 3` they are, at `(±3, ±4)`, and with flats at
/// `|x| ≤ 2` they are at `y = ±√21`, which rounds — and it touches the wall
/// along both of them, so a point near a ruling would be asked against it where
/// its sign is zero. The plane through the axis meets neither run anywhere, so
/// nothing on it has to be exact.
///
/// The point is asked against the pipe and the flat moved into place, as every
/// cut is (§168): a movement turns `t̂` with everything else and never mirrors,
/// so the half a point is in does not change under one.
///
/// # Errors
///
/// [`Error::Undecidable`] when a corner's half could not be settled.
fn one_of_two_runs_between_two_flats(
    face: FaceView<'_>,
    surface: &Surface,
    sides: &[Side],
    cuts: &[Cut],
) -> Result<Option<OneOfTwoRuns>> {
    let Surface::Cylinder(pipe) = surface else {
        return Ok(None);
    };
    let axis = pipe.axis().direction().components();
    let along_the_axis: Vec<(usize, &Plane)> = cuts
        .iter()
        .enumerate()
        .filter_map(|(at, cut)| Some((at, cut.surface.as_plane()?)))
        .filter(|(_, flat)| {
            let (leans, _) = rcad_predicates::decide(&SquareToTheAxis {
                named: WHERE_A_PLANE_AND_A_PIPE_CROSS,
                normal: flat.normal().components(),
                along: axis,
            });
            leans == Decision::Zero
        })
        .collect();
    let [(one, one_flat), (other, other_flat)] = along_the_axis[..] else {
        return Ok(None);
    };
    let all_but_one: Vec<Cut> = cuts
        .iter()
        .enumerate()
        .filter(|&(at, _)| at != other)
        .map(|(_, cut)| cut.clone())
        .collect();
    if !the_normals_lie_along_one_line(one_flat, other_flat)
        || !keep_the_strip_between_them([(&cuts[one], one_flat), (&cuts[other], other_flat)])
        || !crosses_the_pipe_in_two_rulings(pipe, one_flat)
        || !crosses_the_pipe_in_two_rulings(pipe, other_flat)
        || !leaves_a_pipe_in_one_piece(pipe, &all_but_one)
    {
        return Ok(None);
    }

    let mut half = None;
    for corner in sides.iter().flat_map(|side| [side.start, side.end]) {
        let found = which_half_of_the_pipe(pipe, one_flat, exactly(corner.at.coordinates()));
        match found {
            Decision::Pos | Decision::Neg if half.is_none_or(|known| known == found) => {
                half = Some(found);
            }
            Decision::Pos | Decision::Neg | Decision::Zero => return Ok(None),
            Decision::Undecidable { enclosure } => {
                let at = PredicatePoint::from(corner.at.coordinates());
                return Err(gave_up_on_which_half(at, enclosure));
            }
            other => panic!("a side came back with a decision nothing here knows: {other:?}"),
        }
    }
    let half = half.expect("a face with two flats among its cuts has corners on them");
    let Surface::Cylinder(placed_pipe) = face.surface() else {
        panic!("a pipe moved into place is a pipe");
    };
    let Surface::Plane(placed_flat) = cuts[one].placed else {
        panic!("a plane moved into place is a plane");
    };
    Ok(Some(OneOfTwoRuns {
        pipe: placed_pipe,
        flat: placed_flat,
        half,
    }))
}

/// Whether two parallel planes keep the strip between them: the ways into what
/// each keeps point against each other. A plane keeps the way its normal points
/// when it keeps its positive side, so that is the sign of the dot of the two
/// normals against whether the two keep the same sign.
fn keep_the_strip_between_them(flats: [(&Cut, &Plane); 2]) -> bool {
    let [(one, one_flat), (other, other_flat)] = flats;
    let normals = [one_flat, other_flat].map(|flat| exactly(flat.normal().components()));
    match dot(normals[0], normals[1]).sign() {
        Decision::Pos => one.kept != other.kept,
        Decision::Neg => one.kept == other.kept,
        Decision::Zero | Decision::Undecidable { .. } => false,
        found => panic!("a sign came back with a decision nothing here knows: {found:?}"),
    }
}

/// Whether a plane along `pipe`'s axis is strictly less than the radius from
/// it, so it crosses the pipe in two rulings rather than touching it along one
/// or missing it: `(d − n·o)² < r²|n|²`, with `n·x = d` the plane and `o` the
/// axis origin. `false` when the sign could not be settled.
///
/// A plane touching the pipe along a ruling is what this keeps out, since the
/// run it bounds then reaches round to the plane through the axis.
/// [`which_side_of_its_neighbour_a_side_keeps`] refuses a side along a touching
/// plane, but it reads the touch at a corner, and a corner rounded off the ruling
/// it should be on reads as a crossing.
fn crosses_the_pipe_in_two_rulings(pipe: &rcad_geom::Cylinder, flat: &Plane) -> bool {
    off_the_axis_against_the_radius(pipe, flat) == Decision::Neg
}

/// Which side of the plane through `pipe`'s axis square to `flat` the box `at`
/// is on: the sign of `(at − o)·(n × w)`, with `o` the axis origin, `w` its
/// direction and `n` the flat's normal, as stored. Interval arithmetic: a sign
/// it cannot settle reads [`Decision::Undecidable`].
fn which_half_of_the_pipe(pipe: &rcad_geom::Cylinder, flat: &Plane, at: [Interval; 3]) -> Decision {
    let origin = exactly(pipe.axis().origin().coordinates());
    let across = cross(
        exactly(flat.normal().components()),
        exactly(pipe.axis().direction().components()),
    );
    dot(core::array::from_fn(|axis| at[axis] - origin[axis]), across).sign()
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

/// Whether the part of `flat` in front of every cut is one piece because it is
/// a polygon with one bite out of its edge: the cuts are planes and one pipe,
/// the face keeps the pipe's **outside**, it has one arc along that pipe, and
/// every plane either misses the pipe altogether ([`misses_the_pipe`]) or holds
/// both ends of that arc, which at least one plane does. The floor of a block
/// with a bar bored down through the middle of one edge is that: the square
/// less half a disc.
///
/// **What it is told apart from** is the top of a block too narrow for the bar,
/// `a_block_with_a_bar_bored_into_it_has_each_half_of_its_top_hold_its_own_side`
/// in `tests/a_cap_trimmed_by_a_pipe_holds_its_points.rs`. There the bar's
/// circle crosses each long side twice, so the strip outside the bar is two
/// pieces, and each piece's arc has one end on each long side. Neither long
/// side holds both ends, and neither misses the pipe, so this says no to that
/// top, and [`what_a_flat_face_of_chords_and_arcs_holds`] answers it.
///
/// **Why this is one piece.** Let `P` be the part of the plane in front of
/// every plane cut, `C` the curve the pipe cuts the plane in — a circle or an
/// ellipse, since the crossing must close — `D` the inside of `C`, and `A` and
/// `B` the arc's two ends. The region asked about is `P` less the inside of `D`.
///
/// 1. **`C` meets the edge of `P` only at `A` and `B`.** The edge of `P` lies
///    on the lines the plane cuts leave on `flat`. A plane that misses the pipe
///    leaves a line that misses `C`. A plane that holds `A` and `B` leaves the
///    line through them, and a line meets a circle or an ellipse in at most two
///    points, so in those two.
/// 2. **The arc `α` runs through the inside of `P`**, every point but its ends
///    strictly in front of every plane cut: that is what the caller's loop asks
///    of every side against every cut it does not run along, and refuses
///    otherwise.
/// 3. **The rest of `C`, call it `β`, is outside `P`.** Take a plane cut
///    holding both ends: its line meets `C` only at `A` and `B`, so `α` and `β`
///    are on opposite sides of it, and `α` is in front of it, so `β` is behind.
///    So `C` meets `P` in `α` alone.
/// 4. **So `α` cuts `P` in two**, as any arc through the inside of a convex
///    region with both ends on its edge does. Neither piece meets `C`, so each
///    lies wholly inside `D` or wholly outside it. Just beside `α` there are
///    points of both pieces, on both sides of `C`, so one piece is inside and
///    the other outside, and `P` less the inside of `D` is the outside piece
///    with `α` along its edge: one piece.
///
/// The caller's step 3 then makes the face all of it. Its loop also asks that
/// every straight side stays outside the pipe ([`where_a_side_lies`] reads it,
/// through [`a_straight_side_stays_outside_the_pipe`]), which is the part of the
/// face's boundary this argument does not already place.
///
/// **A plane that crosses the pipe beside `A` and `B` is not taken here.** A
/// block with a bar bored down through one corner keeps the square less a
/// quarter of a disc, one piece, but each of its two sides through the bite
/// holds one end of the arc and crosses the pipe again outside the square.
/// Telling that from the narrow top above wants to know whether the second
/// crossing is on the edge of `P`, which is where two of the plane cuts meet.
/// Nothing here asks it, because such a face winds its own sides instead
/// ([`what_a_flat_face_of_chords_and_arcs_holds`]), and so does one cut by a
/// leaned plane that misses `C` on `flat` but meets the pipe elsewhere:
/// [`misses_the_pipe`] asks the plane, not the line it leaves on `flat`.
fn is_a_polygon_less_one_bite_of_a_pipe(flat: &Plane, cuts: &[Cut]) -> bool {
    let (round, planes): (Vec<&Cut>, Vec<&Cut>) = cuts
        .iter()
        .partition(|cut| cut.surface.as_plane().is_none());
    let [bitten_by] = round[..] else {
        return false;
    };
    let Surface::Cylinder(pipe) = bitten_by.surface else {
        return false;
    };
    let [one_end, other_end] = bitten_by.corners_on_it[..] else {
        return false;
    };
    if bitten_by.kept != Decision::Pos
        || one_end == other_end
        || pipe.is_cut_in_a_closed_curve_by(flat) != Some(true)
    {
        return false;
    }
    let holds_both_ends =
        |cut: &Cut| cut.corners_on_it.contains(&one_end) && cut.corners_on_it.contains(&other_end);
    planes.iter().any(|cut| holds_both_ends(cut))
        && planes.iter().all(|cut| {
            holds_both_ends(cut)
                || cut
                    .surface
                    .as_plane()
                    .is_some_and(|plane| misses_the_pipe(&pipe, plane))
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
///
/// `pub(crate)` so `validate.rs`'s `a_circle_round_the_axis` can compare sides
/// at exactly the circles this file reads, rather than at a naive `f64` cross
/// product that can round to zero where this does not, or the reverse.
pub(crate) fn lie_along_one_line(u: [f64; 3], v: [f64; 3]) -> bool {
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
/// **A straight side that starts at a cone's apex is read at its end
/// instead**, because the cone has no gradient at its apex and `∇S × ∇T` is
/// nothing there for every plane, crossing or touching. Such a side is a piece
/// of a ruling, the ray from the apex through its end, and along that ray the
/// cone's gradient is the one at the end scaled by how far out it is
/// ([`gradient_at`] is linear in the point less the apex), so every point of the
/// side but the apex reads the sign the end reads. A plane touching the cone
/// along the ruling still reads [`Decision::Zero`] there and is still refused. A
/// face with a corner at the apex has one such side wherever its wire passes
/// the apex: the side that comes in ends there and is read at its start, and the
/// side that goes out starts there; a quarter of a pointed cone's skin was
/// refused at that second side until `docs/decisions.md` §389's row Q820.
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
    let starts_at_the_apex = matches!(surface, Surface::Cone(cone) if cone.apex() == side.start.at);
    let at = if starts_at_the_apex && matches!(side.run, SideRun::Straight) {
        side.end.at
    } else {
        side.start.at
    };
    let along_the_crossing =
        the_way_the_crossing_runs(surface, &side.across, at).ok_or_else(cannot)?;
    let flat = [surface, &side.across].map(|one| one.as_plane().is_some());
    let runs = match &side.run {
        SideRun::Straight => {
            let to = side.end.at.coordinates();
            let from = side.start.at.coordinates();
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
/// - A straight side of a flat face against a pipe that face's plane crosses in
///   a curve that closes, **outside** it but for an end:
///   [`a_straight_side_stays_outside_the_pipe`].
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
/// - An arc of a pipe against a plane that misses that pipe altogether
///   ([`misses_the_pipe`]), both ends on the same side of it: the arc is on the
///   pipe, so it never reaches the plane, and is on the side its ends are. The
///   floor of a block with a bar bored through one edge has its arc against the
///   three sides of the block the bar is clear of.
/// - Any other arc against a plane, both ends on the same side of it, when the
///   line the plane meets the arc's own plane in misses the arc's curve or meets
///   it only behind the chord ([`an_arc_never_reaches_the_plane`]). The skin of
///   a ball cut down to four planes through its centre has each of its four arcs
///   against the plane across from it (row Q976).
///
/// Everything else is [`Lies::Across`]: an arc with neither end on the plane
/// that the line does meet in front of its chord really does bulge across it.
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
            Surface::Cylinder(pipe)
                if surface
                    .as_plane()
                    .is_some_and(|own| pipe.is_cut_in_a_closed_curve_by(own) == Some(true))
                    && a_straight_side_stays_outside_the_pipe(&pipe, &cut.surface, side, ends) =>
            {
                Lies::Beside(Decision::Pos)
            }
            Surface::Sphere(_)
            | Surface::Cylinder(_)
            | Surface::Cone(_)
            | Surface::Torus(_)
            | _ => Lies::Across,
        },
        SideRun::Arc { into_the_arc } => {
            let arcs_own_round_surface = match (surface.as_plane(), side.across.as_plane()) {
                (Some(_), None) => Some(&side.across),
                (None, Some(_)) => Some(surface),
                (Some(_), Some(_)) | (None, None) => None,
            };
            let own_plane = surface.as_plane().or(side.across.as_plane());
            let Surface::Plane(flat) = cut.surface else {
                let on_a_pipe_running_the_same_way = arcs_own_round_surface
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
                    let Some(own_plane) = own_plane else {
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
                        && (own_plane
                            .is_some_and(|own| the_normals_lie_along_one_line(own, &flat))
                            || arcs_own_round_surface.is_some_and(|own| {
                                matches!(own, Surface::Cylinder(pipe) if misses_the_pipe(pipe, &flat))
                            })
                            || own_plane.zip(arcs_own_round_surface).is_some_and(
                                |(own, round)| {
                                    an_arc_never_reaches_the_plane(
                                        own,
                                        round,
                                        side.start.at,
                                        *into_the_arc,
                                        &flat,
                                    )
                                },
                            )) =>
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
    if the_side_an_arc_leaves_its_end_on(surface, side, into_the_arc, cut, at) == Some(far) {
        Lies::Beside(far)
    } else {
        Lies::Across
    }
}

/// The side of `cut` an arc leaves its end `at` on, as
/// [`where_an_arc_with_one_end_on_a_cut_lies`] reads it:
/// `(∇S × ∇T · into_the_arc)(∇S × ∇T · cut)`. [`Decision::Zero`] for a graze
/// and [`Decision::Undecidable`] for a sign not settled, which no caller takes
/// for a side; `None` for a surface [`gradient_at`] does not read.
fn the_side_an_arc_leaves_its_end_on(
    surface: &Surface,
    side: &Side,
    into_the_arc: [Interval; 3],
    cut: &Plane,
    at: Point,
) -> Option<Decision> {
    let along = the_way_the_crossing_runs(surface, &side.across, at)?;
    Some((dot(along, into_the_arc) * dot(along, exactly(cut.normal().components()))).sign())
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

/// Whether a straight side lies strictly outside `pipe` but for an end on it,
/// in the two shapes the floor of a block with a bar bored through one edge
/// has; `cut` is `pipe` as a surface and `ends` what [`where_a_side_lies`]
/// read at the two ends.
///
/// - **Both ends outside, along a plane that misses the pipe**
///   ([`misses_the_pipe`]): every point of such a plane is outside, and the
///   side is on it.
/// - **One end on the pipe, by topology, leaving it outwards**: the way from
///   that end to the other reads positive against the pipe's gradient there.
///   A pipe's sign `f` is convex — its part of degree two, `|w|²|x|² − (w·x)²`,
///   is never negative — so along the side `f(A + t·v) ≥ f(A) + t·∇f(A)·v`,
///   which is `t` times a positive number for every `t > 0`, `f(A)` being zero.
///   So every point past `A` is strictly outside. The far end is asked to read
///   outside as well, which it then must.
///
/// Anything else is `false`. A side with both ends outside along a plane that
/// crosses the pipe may run through it, as each long side of the narrow top in
/// `a_block_with_a_bar_bored_into_it_has_each_half_of_its_top_hold_its_own_side`
/// would if it were one side.
fn a_straight_side_stays_outside_the_pipe(
    pipe: &rcad_geom::Cylinder,
    cut: &Surface,
    side: &Side,
    ends: [Decision; 2],
) -> bool {
    let leaves_outwards = |on: Corner, off: Corner| {
        let Some(out) = gradient_at(cut, on.at) else {
            return false;
        };
        let [from, to] = [on.at, off.at].map(|corner| corner.coordinates());
        let way = core::array::from_fn(|axis| {
            Interval::exactly(to[axis]) - Interval::exactly(from[axis])
        });
        dot(way, out).sign() == Decision::Pos
    };
    if ends == [Decision::Pos, Decision::Pos] {
        side.across
            .as_plane()
            .is_some_and(|along| misses_the_pipe(pipe, along))
    } else if ends == [Decision::Zero, Decision::Pos] {
        leaves_outwards(side.start, side.end)
    } else if ends == [Decision::Pos, Decision::Zero] {
        leaves_outwards(side.end, side.start)
    } else {
        false
    }
}

/// Whether `flat` misses `pipe` altogether: it lies along the axis and further
/// than the radius from it. A plane that does not lie along the axis crosses
/// every ruling, so these are the only planes that miss a pipe. `false` when
/// either sign could not be settled.
fn misses_the_pipe(pipe: &rcad_geom::Cylinder, flat: &Plane) -> bool {
    let (leans, _) = rcad_predicates::decide(&SquareToTheAxis {
        named: WHERE_A_PLANE_AND_A_PIPE_CROSS,
        normal: flat.normal().components(),
        along: pipe.axis().direction().components(),
    });
    leans == Decision::Zero && off_the_axis_against_the_radius(pipe, flat) == Decision::Pos
}

/// The sign of `(d − n·o)² − r²|n|²`, with `n·x = d` the plane and `o` the
/// axis origin. For a plane along `pipe`'s axis that is how far it is from the
/// axis against the radius: positive when it misses the pipe, zero when it
/// touches it along a ruling, negative when it crosses it in two. Interval
/// arithmetic: a sign it cannot settle reads [`Decision::Undecidable`].
fn off_the_axis_against_the_radius(pipe: &rcad_geom::Cylinder, flat: &Plane) -> Decision {
    let normal = exactly(flat.normal().components());
    let off_the_axis =
        Interval::exactly(flat.offset()) - dot(normal, exactly(pipe.axis().origin().coordinates()));
    let radius = Interval::exactly(pipe.radius());
    (off_the_axis * off_the_axis - radius * radius * dot(normal, normal)).sign()
}

/// Whether an arc of the closed curve the plane `own` cuts `round` in never
/// reaches the plane `cut`: the line the two planes meet in misses the curve,
/// or meets it only behind the arc's chord. `start` is either end of the arc and
/// `into_the_arc` is [`the_way_into_an_arc`]. `false` when the two planes are
/// side by side, when the curve does not close, and when a sign could not be
/// settled.
///
/// **Why that is enough.** The arc lies on `own`, so it can be on `cut` only on
/// that line, and only where the line meets its curve. The arc is all of the
/// curve strictly on the `into_the_arc` side of its chord, with its two ends,
/// so a point of the curve strictly behind the chord is not on the arc. When
/// every point where the line meets the curve is behind it, nothing of the arc
/// is on `cut`, the arc is in one piece, and it stays on whichever side its two
/// ends are on. The caller asks that they are on the same side.
///
/// **How the meets are read without building them.** With `u = n × h` along
/// the line, `n·x = e` the arc's plane and `h·x = d` the cut,
/// `x₀ = (e(h × u) + d(u × n))/|u|²` is on both planes, and the line is
/// `(|u|²x₀ + t·u)/|u|²`. [`how_far_outside`] at that point, scaled by `|u|²`,
/// is `a·t² + 2b·t + c`: `a` is the round surface's part of degree two at `u`,
/// positive because the curve closes; `c` is its value at `|u|²x₀`; and `b`, the
/// cross term, is a quarter of its value a step of `u` past that point less its
/// value a step before, since the two squares cancel. The meets are
/// `t = (−b ± √(b² − ac))/a`, and `b² − ac` negative is the line missing the
/// curve. Behind the chord is `g + γ·t < 0`, with `g` how far the arc's side of
/// the chord reads at `|u|²x₀` against `start`, and `γ` how fast it grows along
/// `u`. Both meets read that when `Δ = a·g − γ·b` is negative and `Δ²` is more
/// than `γ²(b² − ac)`, which is `Δ < −|γ|·√(b² − ac)` multiplied through by `a`
/// — the same two signs [`into_the_arc_on_the_centres_side`] reads of the far
/// arc's middle, for a pair of points instead of one. Nothing is divided and no
/// root is taken: every sign is of a polynomial in stored numbers, in interval
/// arithmetic.
fn an_arc_never_reaches_the_plane(
    own: &Plane,
    round: &Surface,
    start: Point,
    into_the_arc: [Interval; 3],
    cut: &Plane,
) -> bool {
    let Some(centred) = where_a_closed_crossing_is_centred(round, own) else {
        return false;
    };
    let normal = exactly(own.normal().components());
    let facing = exactly(cut.normal().components());
    let along = cross(normal, facing);
    let scale = dot(along, along);
    let [own_offset, cut_offset] = [own.offset(), cut.offset()].map(Interval::exactly);
    let [towards_own, towards_the_cut] = [cross(facing, along), cross(along, normal)];
    let on_both: [Interval; 3] = core::array::from_fn(|axis| {
        own_offset * towards_own[axis] + cut_offset * towards_the_cut[axis]
    });
    let less = |point: Point| -> [Interval; 3] {
        let point = exactly(point.coordinates());
        core::array::from_fn(|axis| on_both[axis] - scale * point[axis])
    };
    let off_from = less(centred.from);
    let step = |by: Interval| -> [Interval; 3] {
        core::array::from_fn(|axis| off_from[axis] + by * along[axis])
    };
    let (Some(a), Some(c), Some(past), Some(before)) = (
        how_far_outside(round, along, Interval::exactly(0.0)),
        how_far_outside(round, off_from, scale),
        how_far_outside(round, step(Interval::exactly(1.0)), scale),
        how_far_outside(round, step(Interval::exactly(-1.0)), scale),
    ) else {
        return false;
    };
    if scale.sign() != Decision::Pos || a.sign() != Decision::Pos {
        return false;
    }
    let b = (past - before) * Interval::exactly(0.25);
    let meets = b * b - a * c;
    if meets.sign() == Decision::Neg {
        return true;
    }
    let grows = dot(into_the_arc, along);
    let middle = a * dot(into_the_arc, less(start)) - grows * b;
    middle.sign() == Decision::Neg
        && (middle * middle - grows * grows * meets).sign() == Decision::Pos
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

/// The face is the part of its nappe in front of its cap and in front of either
/// of two planes through its apex
/// ([`the_cap_and_the_two_cuts_a_cone_face_is_in_front_of_either_of`]).
fn held_in_front_of_the_cap_and_of_either(
    either: &InFrontOfEither,
    point: PredicatePoint,
) -> Result<OnAFace> {
    let by_the_cap = held_in_front_of_all(core::slice::from_ref(&either.cap), point)?;
    if by_the_cap == OnAFace::NotHeld {
        return Ok(OnAFace::NotHeld);
    }
    let by_either = held_in_front_of_any(&either.either, point)?;
    Ok(if by_either == OnAFace::NotHeld {
        OnAFace::NotHeld
    } else if by_the_cap == OnAFace::Held && by_either == OnAFace::Held {
        OnAFace::Held
    } else {
        OnAFace::OnItsBoundary
    })
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

/// The face is the part of its pipe in front of every cut that is in its own
/// half ([`one_of_two_runs_between_two_flats`]). The other half's piece is
/// strictly in the other half, so a point on the plane between the halves is
/// not on the face either.
fn held_in_front_of_all_on_its_own_run(
    run: &OneOfTwoRuns,
    cuts: &[Cut],
    point: PredicatePoint,
) -> Result<OnAFace> {
    let held = held_in_front_of_all(cuts, point)?;
    if held == OnAFace::NotHeld {
        return Ok(held);
    }
    match which_half_of_the_pipe(&run.pipe, &run.flat, point.enclosure()) {
        found if found == run.half => Ok(held),
        Decision::Pos | Decision::Neg | Decision::Zero => Ok(OnAFace::NotHeld),
        Decision::Undecidable { enclosure } => Err(gave_up_on_which_half(point, enclosure)),
        other => panic!("a side came back with a decision nothing here knows: {other:?}"),
    }
}

/// The face is the part of its ring in front of both planes square to its axis
/// and on `band`'s side of the pipe of the ring's own radius
/// ([`the_side_of_the_pipe_naming_both_circles`]), already moved into place.
///
/// A point on the pipe is kept, as a picker keeps what is on it. No point of
/// the ring in front of both planes is on it, so that reading is never asked of
/// one on the face.
///
/// # Errors
///
/// [`Error::Undecidable`], naming the pipe, when it cannot say.
fn held_in_front_of_all_on_one_band(
    cuts: &[Cut],
    band: &Picker,
    point: PredicatePoint,
) -> Result<OnAFace> {
    let held = held_in_front_of_all(cuts, point)?;
    if held == OnAFace::NotHeld {
        return Ok(held);
    }
    match band.side_of_point(point) {
        Decision::Pos | Decision::Zero => Ok(held),
        Decision::Neg => Ok(OnAFace::NotHeld),
        Decision::Undecidable { enclosure } => Err(gave_up(
            band.predicate_name(),
            point,
            Level::Exact,
            enclosure,
            Measures::Area,
        )),
        other => panic!("a pipe came back with a decision nothing here knows: {other:?}"),
    }
}

/// The face is its ring less the band between two planes square to its axis on
/// `band`'s side of the pipe of the ring's own radius
/// ([`the_side_of_the_pipe_naming_both_circles`]), already moved into place:
/// the part in front of either plane, and between them the part off the band.
///
/// The band's own two circles are the face's boundary, and the other band's
/// two circles are inside the face, so a point on a plane is on the boundary
/// only when it is on the band's side of the pipe. A point on the pipe is taken
/// as on that side, as a picker keeps what is on it, and so is the band's own
/// reading of it. The ring meets the pipe only at the top and the bottom of the
/// tube, which are in front of either plane, so that reading is never asked of
/// a point on the face.
///
/// # Errors
///
/// [`Error::Undecidable`], naming a plane or the pipe, when it cannot say.
fn held_in_front_of_either_or_off_the_band(
    cuts: &[Cut],
    band: &Picker,
    point: PredicatePoint,
) -> Result<OnAFace> {
    let by_the_planes = held_in_front_of_any(cuts, point)?;
    if by_the_planes == OnAFace::Held {
        return Ok(by_the_planes);
    }
    match band.side_of_point(point) {
        Decision::Neg => Ok(OnAFace::Held),
        Decision::Pos | Decision::Zero => Ok(by_the_planes),
        Decision::Undecidable { enclosure } => Err(gave_up(
            band.predicate_name(),
            point,
            Level::Exact,
            enclosure,
            Measures::Area,
        )),
        other => panic!("a pipe came back with a decision nothing here knows: {other:?}"),
    }
}

/// The face is the part of its surface in front of any of `cuts`: on a sphere,
/// whose rest is behind all of them; on the plane of two discs taken together
/// ([`is_two_discs_taken_together`]); on a cone, of the two planes through its
/// apex it is in front of either of
/// ([`held_in_front_of_the_cap_and_of_either`]); and on a ring, of two planes
/// each of whose circles the face has is behind the other
/// ([`what_a_ring_face_between_two_planes_holds`]).
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
/// What trims the face along an edge is the surface of the face across it,
/// unless the edge's run names a third surface ([`the_surface_an_edge_cuts`]).
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
            let surface = the_surface_an_edge_cuts(face, partner, edge);
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

/// The surface that cuts `face` along `edge`, with `partner` across it, moved
/// into place: the partner's own, unless the edge's run names a third surface,
/// and then that one.
///
/// **A face across a circle can be on the face's own surface.** A bar whose
/// wall is two bands on one pipe has the circle between them written as where
/// the pipe crosses a plane, because the pair either side is one pipe and names
/// no curve. Read off the partner, the band is trimmed by its own pipe, which
/// [`intersect`] calls one surface with it, and every point of the band was
/// refused `AFaceMissesPartOfWhereItsSurfacesCross` (row Q977, §325.5).
///
/// **Which two surfaces a run is between has one answer**,
/// [`which_two_surfaces_a_run_is_between`], which the mesher and
/// `Model::validate` already read the circle by. Every surface it hands back is
/// one of the two faces' or the run's own, so a surface that is neither face's
/// is the run's, and that is the surface the circle is on and the one that cuts
/// this face there. Where it names only the two faces, the partner is it.
///
/// The question is asked of the stored surfaces, as the other readers ask it,
/// so that a run naming a face's own surface bit for bit still does once both
/// are moved; and the run's surface comes back moved the way the edge is.
fn the_surface_an_edge_cuts(
    face: FaceView<'_>,
    partner: FaceView<'_>,
    edge: EdgeView<'_>,
) -> Surface {
    let (mine, theirs) = (face.stored_surface(), partner.stored_surface());
    let named = which_two_surfaces_a_run_is_between(edge.stored_run(), [&mine, &theirs]);
    if named.iter().all(|one| *one == mine || *one == theirs) {
        return partner.surface();
    }
    let run = edge.run();
    let (EdgeRun::RightRoundWhereItCrosses { and } | EdgeRun::PartOfWhereItCrosses { and, .. }) =
        &run
    else {
        panic!(
            "only a run carrying a surface names one that neither face is on, and this is {run:?}"
        );
    };
    *and
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
/// **The corners are asked first, and the wire only where they say nothing.**
/// The other place a side comes from is which way round the wire runs, read by
/// the closed-edge rule ([`the_side_its_wire_keeps_along`]), and
/// [`what_a_curved_face_holds`] reads a face with every corner on its
/// trimming that way.
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

/// The same, for which half of a pipe between two flats a point or a corner is
/// in ([`which_half_of_the_pipe`]): a length, the distance off the plane through
/// the axis scaled by the lengths of the two directions.
fn gave_up_on_which_half(point: PredicatePoint, enclosure: f64) -> Error {
    gave_up(
        WHICH_HALF_OF_A_PIPE,
        point,
        Level::Interval,
        enclosure,
        Measures::Length,
    )
}

/// The name which half of a pipe between two flats a point is in goes by in
/// provenance records.
const WHICH_HALF_OF_A_PIPE: &str = "which_half_of_a_pipe";

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

    /// A ball of radius 5 cut down to `x ≥ |y|` and `x ≥ |z|`, four planes
    /// through its centre, keeps one face of its skin, and that face holds a
    /// point of the sphere exactly when the four planes do.
    ///
    /// Each of the face's four arcs has the plane across from it as a cut with
    /// neither of its ends on it: the arc on `x = z` runs from `(a, a, a)` to
    /// `(a, −a, a)`, and `x = −z` meets its circle at `(0, ±5, 0)`, behind its
    /// chord. Until row Q976 that read [`Lies::Across`] and the face was refused.
    #[test]
    fn the_skin_of_a_ball_cut_down_to_four_planes_through_its_centre_holds_what_they_hold() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let ball = model
            .add_ball(5.0, &Isometry::identity())
            .expect("a positive radius");
        let square =
            crate::ProfileBuilder::new(vec![[0.0, 0.0], [20.0, 0.0], [20.0, 20.0], [0.0, 20.0]])
                .build(&policy, &mut log)
                .expect("a square");
        let quarter = core::f64::consts::FRAC_PI_4;
        let a_cube_turned = |model: &mut Model, about: Axis, turn: f64, shift: Vector| {
            let placed = Isometry::rotation(&about, turn)
                .expect("a finite turn")
                .compose(&Isometry::translation(shift).expect("a finite shift"));
            model
                .add_extrusion(&square, 20.0, &placed)
                .expect("a positive height")
        };
        let either_side_of_y =
            a_cube_turned(&mut model, Axis::y(), quarter, Vector::new(0.0, -10.0, 0.0));
        let either_side_of_z = a_cube_turned(
            &mut model,
            Axis::z(),
            -quarter,
            Vector::new(0.0, 0.0, -10.0),
        );
        let wedge = model
            .fuse(
                &[ball],
                &[either_side_of_y],
                crate::Rule::Intersection,
                &policy,
                &mut log,
            )
            .expect("a ball and a cube with an edge through its centre")
            .solids()
            .to_vec();
        let pyramid = model
            .fuse(
                &wedge,
                &[either_side_of_z],
                crate::Rule::Intersection,
                &policy,
                &mut log,
            )
            .expect("a wedge of a ball and a second such cube");
        assert!(log.is_empty(), "a certified fuse logs nothing: {log:?}");
        let [solid] = pyramid.solids() else {
            panic!("one solid: {:?}", pyramid.solids());
        };
        let skins: Vec<FaceKey> = model
            .solid(*solid)
            .expect("this model made the key")
            .faces()
            .filter(|face| matches!(face.surface(), Surface::Sphere(_)))
            .map(|face| face.key())
            .collect();
        let [skin] = skins[..] else {
            panic!("one face on the sphere: {skins:?}");
        };

        let mut on_the_sphere: Vec<[f64; 3]> = vec![[5.0, 0.0, 0.0], [-5.0, 0.0, 0.0]];
        for [one, other] in [[3.0, 4.0], [4.0, 3.0], [0.0, 5.0]] {
            for [x, y] in [[one, other], [-one, other], [one, -other], [-one, -other]] {
                on_the_sphere.extend([[x, y, 0.0], [x, 0.0, y], [0.0, x, y]]);
            }
        }
        let mut held = 0;
        for point in on_the_sphere {
            let [x, y, z] = point;
            let expected = if x > y.abs() && x > z.abs() {
                held += 1;
                OnAFace::Held
            } else {
                OnAFace::NotHeld
            };
            assert_eq!(
                what_it_holds(&model, *solid, skin, PredicatePoint::from(point)),
                expected,
                "{point:?}"
            );
        }
        assert_eq!(held, 5, "the points inside the four planes");
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

    /// What [`is_a_family_this_follows`] lets through on a ball's skin and what
    /// it refuses: any two planes, and no third that misses the centre, because
    /// three can leave the part of the sphere in front of them all in two pieces.
    ///
    /// A ball of radius 2 at the origin kept to `x, y, z ≤ 1.3`. Each two of the
    /// three caps it loses overlap, `2 · 1.3² < 4`, and no point is in all three,
    /// `3 · 1.3² > 4`, so the lost caps go right round `(1, 1, 1)` and leave a
    /// three-sided patch there apart from the rest. On the sphere a point with
    /// no coordinate below zero and none past `1.3` has each at least
    /// `√(4 − 2 · 1.3²) ≈ 0.787`, the other two squared adding up to no more
    /// than `2 · 1.3²`. So nothing in front of all three has its smallest
    /// coordinate between `0` and `0.787`, and a path from the patch to
    /// `(−2, 0, 0)` would have to pass one. This samples the sphere for such a
    /// point, finds none, and finds both ends of the path in front of all three.
    #[test]
    fn three_planes_that_miss_a_balls_centre_leave_its_skin_in_two_pieces() {
        let ball = Surface::Sphere(
            rcad_geom::Sphere::new(Point::origin(), 2.0).expect("a ball of radius two"),
        );
        let planes = [
            Vector::new(1.0, 0.0, 0.0),
            Vector::new(0.0, 1.0, 0.0),
            Vector::new(0.0, 0.0, 1.0),
        ]
        .map(|normal| {
            Plane::from_point_and_normal(Point::origin() + normal * 1.3, normal)
                .expect("a plane square to an axis")
        });
        let cuts: Vec<Cut> = planes
            .map(|plane| Cut {
                surface: Surface::Plane(plane),
                placed: Surface::Plane(plane),
                kept: Decision::Neg,
                corners_on_it: Vec::new(),
            })
            .into_iter()
            .collect();
        assert!(
            !is_a_family_this_follows(&ball, &cuts),
            "three planes that miss the centre are refused"
        );
        for left_out in 0..3 {
            let two: Vec<Cut> = (0..3)
                .filter(|which| *which != left_out)
                .map(|which| cuts[which].clone())
                .collect();
            assert!(
                is_a_family_this_follows(&ball, &two),
                "any two planes are followed"
            );
        }

        let in_front_of_all = |at: [f64; 3]| at.iter().all(|coordinate| *coordinate <= 1.3);
        let how_many = 1000u32;
        let mut across_the_gap = 0;
        for up in 0..=how_many {
            let latitude = core::f64::consts::PI * f64::from(up) / f64::from(how_many);
            for round in 0..2 * how_many {
                let longitude = core::f64::consts::PI * f64::from(round) / f64::from(how_many);
                let at = [
                    2.0 * latitude.sin() * longitude.cos(),
                    2.0 * latitude.sin() * longitude.sin(),
                    2.0 * latitude.cos(),
                ];
                let smallest = at.iter().copied().fold(f64::INFINITY, f64::min);
                if in_front_of_all(at) && (0.0..0.78).contains(&smallest) {
                    across_the_gap += 1;
                }
            }
        }
        assert_eq!(
            across_the_gap, 0,
            "no point in front of all three has its smallest coordinate in [0, 0.78)"
        );
        let patch = 2.0 / 3.0f64.sqrt();
        assert!(
            in_front_of_all([patch; 3]) && patch >= 0.78,
            "the patch round (1, 1, 1) is in front of all three"
        );
        assert!(
            in_front_of_all([-2.0, 0.0, 0.0]),
            "and so is the far side of the ball"
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

    /// Why a cone face beside **two** cuts through its apex is followed only when
    /// the two meet inside the cone.
    ///
    /// On the cone of slope a half about `+z`, keep `x ≤ z/10` and `−x ≤ z/10`,
    /// both through the apex, and `z ≤ 8`, which the apex is in front of. Along
    /// the ruling at `φ` a plane through the apex is zero at the apex and grows
    /// at one rate, so it keeps the whole ruling or only the apex. Both keep more
    /// than the apex exactly where `|cos φ| < 1/5`: **two** runs of rulings, round
    /// `+y` and round `−y`, two sheets of one nappe meeting only at the apex. The
    /// part in front of every cut is connected through the apex and is still not
    /// one face, and it is refused: the two planes meet along `y`, outside the
    /// cone.
    ///
    /// One plane through the apex keeps one run, and that is followed.
    ///
    /// So are two that meet along the axis, inside the cone: `x = 0` and `y = 0`
    /// keep one run whichever quarter they keep — `0 < φ < 90°` for `x ≥ 0` and
    /// `y ≥ 0`, a quarter of a pointed cone, and one quarter turn of rulings for
    /// each of the others.
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
            "two cuts through the apex meeting outside the cone are refused"
        );

        let half = [one_side(), roof()];
        assert_eq!(runs_of_rulings_carrying_more_than_the_apex(&half), 1);
        assert!(
            is_a_family_this_follows(&Surface::Cone(cone), &half),
            "one cut through the apex, and any the apex is in front of, are followed"
        );

        for (x_kept, y_kept) in [(1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
            let quarter = [
                cut([0.0; 3], [x_kept, 0.0, 0.0]),
                cut([0.0; 3], [0.0, y_kept, 0.0]),
                roof(),
            ];
            assert_eq!(
                runs_of_rulings_carrying_more_than_the_apex(&quarter),
                1,
                "the quarter facing {x_kept}, {y_kept}"
            );
            assert!(
                is_a_family_this_follows(&Surface::Cone(cone), &quarter),
                "two cuts through the apex meeting inside the cone keep one run, and are \
                 followed: the quarter facing {x_kept}, {y_kept}"
            );
        }
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
        a_bend_walked(bend, near_cap, Orientation::Reversed)
    }

    /// [`a_bend`] with its wall walking the near circle
    /// `near_circle_on_the_wall` and the far circle the other way, and each
    /// cap walking its circle against the wall. `Reversed` is [`a_bend`]'s
    /// walk; `Forward` turns every coedge on a circle over and leaves the caps
    /// and the seam as they were, so each cap is walked against the way it
    /// faces and the wall's circles say the other stretch of ring from the one
    /// its seam is on.
    fn a_bend_walked(
        bend: Bend,
        near_cap: NearCap,
        near_circle_on_the_wall: Orientation,
    ) -> (Model, SolidKey, FaceKey) {
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
        a_piece_of_pipe_walked(
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
            near_circle_on_the_wall,
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
        a_piece_of_pipe_walked(ring, near, far, seam, Orientation::Reversed)
    }

    /// [`a_piece_of_pipe`] with its wall walking the near circle
    /// `near_circle_on_the_wall` and the far one the other way, and each cap
    /// walking its circle against the wall, as [`a_bend_walked`] says.
    fn a_piece_of_pipe_walked(
        ring: rcad_geom::Torus,
        near: End,
        far: End,
        seam: Option<EdgeRun>,
        near_circle_on_the_wall: Orientation,
    ) -> (Model, SolidKey, FaceKey) {
        a_piece_of_pipe_walking_each_circle(
            ring,
            near,
            far,
            seam,
            [near_circle_on_the_wall, near_circle_on_the_wall.reversed()],
        )
    }

    /// [`a_piece_of_pipe`] with its wall walking the near circle `on_the_wall[0]`
    /// and the far one `on_the_wall[1]`, and each cap walking its circle against
    /// the wall.
    ///
    /// Two closed edges on one circle are stored running the same way round it,
    /// because the way a closed edge is stored running is read off the ring
    /// alone. So [`a_piece_of_pipe_walked`], which walks the far circle the
    /// other way from the near one, walks a circle had twice once each way
    /// round, and this is what walks it the same way round twice.
    fn a_piece_of_pipe_walking_each_circle(
        ring: rcad_geom::Torus,
        near: End,
        far: End,
        seam: Option<EdgeRun>,
        on_the_wall: [Orientation; 2],
    ) -> (Model, SolidKey, FaceKey) {
        a_piece_of_pipe_placed(
            ring,
            near,
            far,
            seam,
            on_the_wall,
            Orientation::Forward,
            Isometry::identity(),
        )
    }

    /// [`a_piece_of_pipe_walking_each_circle`] with the shell using the wall
    /// `wall_used` — `Reversed` where the material is outside the ring's tube —
    /// and the solid using the shell moved by `placed`.
    fn a_piece_of_pipe_placed(
        ring: rcad_geom::Torus,
        near: End,
        far: End,
        seam: Option<EdgeRun>,
        on_the_wall: [Orientation; 2],
        wall_used: Orientation,
        placed: Isometry,
    ) -> (Model, SolidKey, FaceKey) {
        let [near_circle_on_the_wall, far_circle_on_the_wall] = on_the_wall;
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
        let near_cap = cap(
            &mut model,
            near.cap,
            near_circle,
            near_circle_on_the_wall.reversed(),
        );
        let far_cap = cap(
            &mut model,
            far.cap,
            far_circle,
            far_circle_on_the_wall.reversed(),
        );

        let wires: SmallVec<[_; 1]> = match seam {
            Some(seam) => {
                let round_the_wall: SmallVec<[_; 4]> = [
                    (near_circle, near_circle_on_the_wall),
                    (seam, Orientation::Forward),
                    (far_circle, far_circle_on_the_wall),
                    (seam, Orientation::Reversed),
                ]
                .map(|(edge, turn)| model.add_coedge(edge, turn))
                .into_iter()
                .collect();
                smallvec::smallvec![model.add_wire(round_the_wall)]
            }
            None => [
                (near_circle, near_circle_on_the_wall),
                (far_circle, far_circle_on_the_wall),
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
                (wall, wall_used),
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
            loc: placed,
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

    /// A three-quarter bend's wall holds the three quarters of the ring
    /// between its caps, and nothing of the fourth (row Q1144).
    ///
    /// It is the part of the ring in front of *either* cap's plane, not of
    /// both: each of its circles is behind the other cap's plane. It was
    /// refused `AFaceMissesPartOfWhereItsSurfacesCross` for that, at every
    /// point asked. Read off a corner instead of the wire, both kept sides come
    /// out turned round, both circles look in front, and every point of the one
    /// quarter the wall does *not* cover comes back held (§243).
    ///
    /// The bend itself is sound, and `Model::validate` passes it. The same
    /// bend with every coedge on a circle turned over is refused
    /// `AClosedEdgeIsWalkedAgainstItsSurface`: each cap is then walked against
    /// the way it faces. Until §392 it was the other way round. The sweep read
    /// which side of each cap's plane the wall keeps off the wall's far corner,
    /// which past half the ring is the side it does not keep, so it refused
    /// the sound bend and passed the one walked the wrong way.
    #[test]
    fn a_three_quarter_bends_wall_holds_the_three_quarters_between_its_caps_and_nothing_else() {
        let (model, solid, wall) = a_bend(Bend::ThreeQuarters, NearCap::FacingOut);
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a three-quarter bend is a solid");
        let (walked_the_wrong_way, _, _) = a_bend_walked(
            Bend::ThreeQuarters,
            NearCap::FacingOut,
            Orientation::Forward,
        );
        let refused =
            walked_the_wrong_way.validate(&Policy::certified(), &mut ProvenanceLog::new());
        assert!(
            matches!(
                refused,
                Err(Error::Invalid(
                    Invalid::AClosedEdgeIsWalkedAgainstItsSurface
                ))
            ),
            "the three-quarter bend walked the wrong way came back {refused:?}"
        );
        a_three_quarter_bends_wall_holds_the_three_quarters(&model, solid, wall);
    }

    /// The same three-quarter bend with its near cap's plane stored facing
    /// into the bend and used reversed holds the same three quarters.
    #[test]
    fn a_three_quarter_bends_wall_holds_the_same_three_quarters_with_its_near_cap_stored_turned_round()
     {
        let (model, solid, wall) = a_bend(Bend::ThreeQuarters, NearCap::FacingInAndUsedReversed);
        a_three_quarter_bends_wall_holds_the_three_quarters(&model, solid, wall);
    }

    /// Every point asked is exactly on the ring of radius four and tube one:
    /// on its outer equator at a 3-4-5 angle in each quarter, at the top or
    /// the bottom of the tube, or on one of the four circles the two caps'
    /// planes cut it in. The wall runs from `+x` round through `+y` and `−x`
    /// to `−y`, so the quarter with `x > 0` and `y < 0` is the one it does not
    /// cover. Its two circles, `y = 0` at `+x` and `x = 0` at `−y`, are its
    /// boundary; the other two circles of those planes are inside it.
    fn a_three_quarter_bends_wall_holds_the_three_quarters(
        model: &Model,
        solid: SolidKey,
        wall: FaceKey,
    ) {
        let expected = [
            (at(3.0, 4.0, 0.0), OnAFace::Held),
            (at(4.0, 3.0, 0.0), OnAFace::Held),
            (at(-3.0, 4.0, 0.0), OnAFace::Held),
            (at(-4.0, -3.0, 0.0), OnAFace::Held),
            (at(-3.0, -4.0, 0.0), OnAFace::Held),
            (at(0.0, 4.0, 1.0), OnAFace::Held),
            (at(0.0, 3.0, 0.0), OnAFace::Held),
            (at(-4.0, 0.0, -1.0), OnAFace::Held),
            (at(-5.0, 0.0, 0.0), OnAFace::Held),
            (at(3.0, -4.0, 0.0), OnAFace::NotHeld),
            (at(4.0, -3.0, 0.0), OnAFace::NotHeld),
            (at(5.0, 0.0, 0.0), OnAFace::OnItsBoundary),
            (at(4.0, 0.0, 1.0), OnAFace::OnItsBoundary),
            (at(3.0, 0.0, 0.0), OnAFace::OnItsBoundary),
            (at(0.0, -5.0, 0.0), OnAFace::OnItsBoundary),
            (at(0.0, -4.0, -1.0), OnAFace::OnItsBoundary),
            (at(0.0, -3.0, 0.0), OnAFace::OnItsBoundary),
        ];
        let bend = model.solid(solid).expect("this model made the key");
        for (point, answer) in expected {
            let held = bend
                .face_holds_the_point(wall, point, &Policy::certified(), &mut ProvenanceLog::new())
                .expect("a three-quarter bend's wall is a face this can follow");
            assert_eq!(held, answer, "{point:?}");
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

    /// The same wall walked the same way round its one circle both times is
    /// refused too, and not read as a half of the ring (row Q790).
    ///
    /// The ring less the circle `y = 0` cuts at `x > 0` is one piece, which
    /// meets that circle from both sides, so no piece of the ring keeps one
    /// side of it at both of two edges along it. This wire does: walked the
    /// same way round, the circle keeps the same side of `y = 0` at both
    /// edges, and the side check passes it. Without the check that the two
    /// corners lie on different circles of the plane, all six walls were read
    /// as a half. Walked `Reversed`, as the half bend walks its near circle,
    /// each held `(3, 4, 0)` and not `(3, −4, 0)`, walked `Forward` the other
    /// way about, and either way `(−5, 0, 0)`, on the circle the wall does not
    /// have, came back on its boundary.
    #[test]
    fn a_wall_round_one_circle_twice_the_same_way_is_refused_rather_than_read_as_a_half() {
        let through_the_axis = |facing| {
            Plane::from_point_and_normal(rcad_geom::Point::origin(), facing)
                .expect("a coordinate direction is a unit normal")
        };
        for walked in [Orientation::Forward, Orientation::Reversed] {
            for (far_facing, far_used) in [
                (Vector::new(0.0, -1.0, 0.0), Orientation::Forward),
                (Vector::new(0.0, 1.0, 0.0), Orientation::Forward),
                (Vector::new(0.0, 1.0, 0.0), Orientation::Reversed),
            ] {
                let (model, solid, wall) = a_piece_of_pipe_walking_each_circle(
                    rcad_geom::Torus::new(Axis::z(), 4.0, 1.0)
                        .expect("a tube narrower than its ring"),
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
                    [walked, walked],
                );
                let bend = model.solid(solid).expect("this model made the key");
                for point in [
                    at(3.0, 4.0, 0.0),
                    at(3.0, -4.0, 0.0),
                    at(4.0, 0.0, 1.0),
                    at(-5.0, 0.0, 0.0),
                ] {
                    let answer = bend.face_holds_the_point(
                        wall,
                        point,
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    );
                    assert!(
                        matches!(answer, Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face }) if face == wall),
                        "walked {walked:?}, far cap facing {far_facing:?}, used {far_used:?}, \
                         {point:?}: {answer:?}"
                    );
                }
            }
        }
    }

    /// A wall whose two edges both run round the outer circle `z = 0` cuts the
    /// ring in, walked the same way round both times, is refused, and not read
    /// as the top or the bottom half of the ring (§243.8).
    ///
    /// The plane `z = 0` is square to the ring's axis, and the ring less its
    /// outer circle is one piece that meets that circle from above and from
    /// below. The wire keeps the same side of the plane at both edges, so the
    /// side check passes it. Without the check that the two corners lie on
    /// different circles of the plane, both walls were read as a half. Walked
    /// `Forward`, each held `(4, 0, −1)` and not `(4, 0, 1)`, walked `Reversed`
    /// the other way about, and either way `(0, −3, 0)`, on the inner circle
    /// the wall does not have, came back on its boundary.
    #[test]
    fn a_wall_round_one_circle_round_the_axis_twice_the_same_way_is_refused() {
        let flat =
            Plane::from_point_and_normal(rcad_geom::Point::origin(), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal");
        for walked in [Orientation::Forward, Orientation::Reversed] {
            let (model, solid, wall) = a_piece_of_pipe_walking_each_circle(
                rcad_geom::Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring"),
                End {
                    corner: rcad_geom::Point::new(5.0, 0.0, 0.0),
                    cap: flat,
                    used: Orientation::Forward,
                },
                End {
                    corner: rcad_geom::Point::new(-5.0, 0.0, 0.0),
                    cap: flat,
                    used: Orientation::Forward,
                },
                None,
                [walked, walked],
            );
            let ring = model.solid(solid).expect("this model made the key");
            for point in [
                at(4.0, 0.0, 1.0),
                at(4.0, 0.0, -1.0),
                at(0.0, 5.0, 0.0),
                at(0.0, -3.0, 0.0),
            ] {
                let answer = ring.face_holds_the_point(
                    wall,
                    point,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                assert!(
                    matches!(answer, Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face }) if face == wall),
                    "walked {walked:?}, {point:?}: {answer:?}"
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

    /// Every way a wall can walk its two circles.
    const EVERY_WALK: [[Orientation; 2]; 4] = [
        [Orientation::Forward, Orientation::Forward],
        [Orientation::Forward, Orientation::Reversed],
        [Orientation::Reversed, Orientation::Forward],
        [Orientation::Reversed, Orientation::Reversed],
    ];

    /// The walk of a band's two circles that `Model::validate` passes, near
    /// circle on `z = −3` first: each of the other three is refused, for the
    /// barrel and the spool alike, the two that walk both circles the same way
    /// round with `AClosedEdgeIsWalkedAgainstItsSurface`, and the one turned
    /// over at both with `FacesDoNotAgreeWhichWayTheyFace`
    /// ([`a_bands_wall_walking_both_circles_the_same_way_round_the_tube_is_refused`]).
    const THE_BANDS_WALK: [Orientation; 2] = [Orientation::Reversed, Orientation::Forward];

    /// A piece of the ring of radius ten and tube five between the planes
    /// `z = −3` and `z = 3`, each cap stored facing away from the other and
    /// used as stored, with its corners at `radius` from the axis and the whole
    /// solid moved by `placed`.
    ///
    /// Each plane cuts the ring in two circles round the axis, of radius 14 and
    /// 6. Corners at 14 make the band outside the pipe of radius ten, a barrel
    /// whose wall the shell uses as stored; corners at 6 make the band inside
    /// it, a spool, whose material is outside the tube and whose wall is used
    /// reversed.
    fn a_band(radius: f64, walk: [Orientation; 2], placed: Isometry) -> (Model, SolidKey, FaceKey) {
        let facing = |up: f64| {
            Plane::from_point_and_normal(
                rcad_geom::Point::new(0.0, 0.0, up),
                Vector::new(0.0, 0.0, up.signum()),
            )
            .expect("the z axis is a unit normal")
        };
        let wall_used = if radius > 10.0 {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        a_piece_of_pipe_placed(
            rcad_geom::Torus::new(Axis::z(), 10.0, 5.0).expect("a tube narrower than its ring"),
            End {
                corner: rcad_geom::Point::new(radius, 0.0, -3.0),
                cap: facing(-3.0),
                used: Orientation::Forward,
            },
            End {
                corner: rcad_geom::Point::new(radius, 0.0, 3.0),
                cap: facing(3.0),
                used: Orientation::Forward,
            },
            None,
            walk,
            wall_used,
            placed,
        )
    }

    /// Points of the ring of radius ten and tube five, each with the band of
    /// it between `z = ±3` that it is on: `Some(true)` for the outer band and
    /// `Some(false)` for the inner, and `None` off both, above `z = 3` or
    /// below `z = −3`. The last four are on the circles the two planes cut,
    /// which each band has as its boundary or does not hold.
    const ON_THE_RING: [(f64, f64, f64, Option<bool>); 13] = [
        (15.0, 0.0, 0.0, Some(true)),
        (9.0, 12.0, 0.0, Some(true)),
        (5.0, 0.0, 0.0, Some(false)),
        (3.0, -4.0, 0.0, Some(false)),
        (13.0, 0.0, 4.0, None),
        (7.0, 0.0, 4.0, None),
        (13.0, 0.0, -4.0, None),
        (-7.0, 0.0, -4.0, None),
        (10.0, 0.0, 5.0, None),
        (14.0, 0.0, 3.0, Some(true)),
        (0.0, 14.0, -3.0, Some(true)),
        (6.0, 0.0, 3.0, Some(false)),
        (0.0, -6.0, -3.0, Some(false)),
    ];

    /// A washer's rounded rim — the outer band of a ring between two planes
    /// square to its axis that both cut the tube — holds that band and nothing
    /// else of the ring, and a spool's waist, the inner band, holds the inner
    /// one (row Q950).
    ///
    /// The planes' sides do not settle either face: both bands are in front of
    /// both planes. Each wall was refused `AFaceMissesPartOfWhereItsSurfacesCross`
    /// at every point asked, because two planes square to the axis were
    /// followed only when they were one stored plane. The pipe of radius ten
    /// tells the bands apart, and it is the same pipe [`intersect`] names each
    /// circle by. Moved a hundred along `x`, each band holds the same points
    /// moved. Asked against the pipe where it is stored, every moved point is
    /// outside it, and the barrel held `(105, 0, 0)`, a point of the inner band.
    #[test]
    fn a_washers_rounded_rim_holds_the_outer_band_between_its_planes_and_a_spools_waist_the_inner()
    {
        let moved = Isometry::translation(Vector::new(100.0, 0.0, 0.0)).expect("a finite shift");
        for (radius, outer) in [(14.0, true), (6.0, false)] {
            for (placed, by) in [(Isometry::identity(), 0.0), (moved, 100.0)] {
                let (model, solid, wall) = a_band(radius, THE_BANDS_WALK, placed);
                model
                    .validate(&Policy::certified(), &mut ProvenanceLog::new())
                    .expect("a barrel or a spool is a solid");
                let band = model.solid(solid).expect("this model made the key");
                for (index, (x, y, z, on)) in ON_THE_RING.into_iter().enumerate() {
                    let expected = match on {
                        Some(this) if this == outer && index >= 9 => OnAFace::OnItsBoundary,
                        Some(this) if this == outer => OnAFace::Held,
                        Some(_) | None => OnAFace::NotHeld,
                    };
                    let point = at(x + by, y, z);
                    let held = band
                        .face_holds_the_point(
                            wall,
                            point,
                            &Policy::certified(),
                            &mut ProvenanceLog::new(),
                        )
                        .expect("a band between two planes square to the axis is followed");
                    assert_eq!(held, expected, "radius {radius}, {point:?}");
                }
            }
        }
    }

    /// A band's wall walked the other way round both circles keeps the far
    /// side of both planes, and holds the ring less that band (row Q1144); one
    /// walked the other way round only one circle keeps the far side of one
    /// plane only, and is refused.
    ///
    /// Turned over at both circles, each circle is behind the other plane, and
    /// the wall is the ring less the band, as a grooved ring's is. Turned over
    /// at one, one circle is in front of the other plane and the other is
    /// behind, and with both circles named by one side of the pipe no face
    /// keeps those sides.
    #[test]
    fn a_bands_wall_walked_the_other_way_round_both_circles_holds_the_rest_of_the_ring() {
        for (radius, outer) in [(14.0, true), (6.0, false)] {
            let (model, solid, wall) = a_band(
                radius,
                THE_BANDS_WALK.map(Orientation::reversed),
                Isometry::identity(),
            );
            let band = model.solid(solid).expect("this model made the key");
            for (index, (x, y, z, on)) in ON_THE_RING.into_iter().enumerate() {
                let expected = match on {
                    Some(this) if this == outer && index >= 9 => OnAFace::OnItsBoundary,
                    Some(this) if this == outer => OnAFace::NotHeld,
                    Some(_) | None => OnAFace::Held,
                };
                let point = at(x, y, z);
                let held = band
                    .face_holds_the_point(
                        wall,
                        point,
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .expect("a ring less a band between two planes is followed");
                assert_eq!(held, expected, "radius {radius}, {point:?}");
            }
            for walk in [
                [Orientation::Forward, Orientation::Forward],
                [Orientation::Reversed, Orientation::Reversed],
            ] {
                let (model, solid, wall) = a_band(radius, walk, Isometry::identity());
                let band = model.solid(solid).expect("this model made the key");
                for point in [at(15.0, 0.0, 0.0), at(5.0, 0.0, 0.0), at(13.0, 0.0, 4.0)] {
                    let answer = band.face_holds_the_point(
                        wall,
                        point,
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    );
                    assert!(
                        matches!(answer, Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face }) if face == wall),
                        "radius {radius}, walked {walk:?}, {point:?}: {answer:?}"
                    );
                }
            }
        }
    }

    /// A ring of radius ten and tube five with a groove turned into it between
    /// the planes `z = −3` and `z = 3`, the whole solid moved by `placed`.
    ///
    /// `outer` turns the groove into the ring's rim, taking the outer band
    /// between the planes down to a floor on the pipe of radius 12; otherwise
    /// it is turned into the bore, taking the inner band out to a floor on the
    /// pipe of radius 8. Either way the wall is the ring less that band, with
    /// the band's two circles as its only edges, and each side of the groove
    /// is the ring of a plane between one of those circles and the floor,
    /// stored facing into the groove.
    ///
    /// **Every walk is its twin's turned over.** The twin is the band itself,
    /// inside the tube, closed off by the same floor with the sides facing out
    /// of it, and `Model::validate` passes it at one floor walk of the four. A
    /// closed edge is stored running the way the curved surface alone says,
    /// so with each side's plane and the floor's use turned round, every walk
    /// turns over. The outer twin's wall walks [`THE_BANDS_WALK`], as the
    /// barrel's does; the inner twin's is used as stored where the spool's is
    /// used reversed, so it walks the spool's walk turned over.
    fn a_grooved_ring(outer: bool, placed: Isometry) -> (Model, SolidKey, FaceKey) {
        a_grooved_ring_walked(outer, the_grooves_walk(outer), placed)
    }

    /// The walk of a grooved ring's wall, near circle on `z = −3` first.
    fn the_grooves_walk(outer: bool) -> [Orientation; 2] {
        if outer {
            THE_BANDS_WALK.map(Orientation::reversed)
        } else {
            THE_BANDS_WALK
        }
    }

    /// [`a_grooved_ring`] with its wall walking `walk`, and the side and the
    /// floor along each circle the wall walks other than [`the_grooves_walk`]
    /// turned over to match: the side walks that circle and its floor circle
    /// the other way and faces the other way, and the floor walks its circle
    /// the other way. So each edge is still walked once each way and each side
    /// still winds the way it faces, and only the wall's walk says anything new.
    fn a_grooved_ring_walked(
        outer: bool,
        walk: [Orientation; 2],
        placed: Isometry,
    ) -> (Model, SolidKey, FaceKey) {
        let (band, floor, floor_walk, floor_used) = if outer {
            (
                14.0,
                12.0,
                [Orientation::Reversed, Orientation::Forward],
                Orientation::Forward,
            )
        } else {
            (
                6.0,
                8.0,
                [Orientation::Forward, Orientation::Reversed],
                Orientation::Reversed,
            )
        };
        let grooves = the_grooves_walk(outer);
        let turned = [walk[0] != grooves[0], walk[1] != grooves[1]];
        let floor_walk = [0, 1].map(|index| {
            if turned[index] {
                floor_walk[index].reversed()
            } else {
                floor_walk[index]
            }
        });
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let circle = |model: &mut Model, radius: f64, up: f64| {
            let corner = model.add_vertex(rcad_geom::Point::new(radius, 0.0, up));
            model.add_closed_edge(corner)
        };
        let band_circles = [
            circle(&mut model, band, -3.0),
            circle(&mut model, band, 3.0),
        ];
        let floor_circles = [
            circle(&mut model, floor, -3.0),
            circle(&mut model, floor, 3.0),
        ];
        let face = |model: &mut Model, surface, round: &[(crate::key::EdgeSlot, Orientation)]| {
            let wires: SmallVec<[_; 1]> = round
                .iter()
                .map(|&(edge, turn)| {
                    let round_it = model.add_coedge(edge, turn);
                    model.add_wire(smallvec::smallvec![round_it])
                })
                .collect();
            model.add_face(surface, wires)
        };
        let wall = face(
            &mut model,
            Surface::Torus(
                rcad_geom::Torus::new(Axis::z(), 10.0, 5.0).expect("a tube narrower than its ring"),
            ),
            &[(band_circles[0], walk[0]), (band_circles[1], walk[1])],
        );
        let sides = [(-3.0, 1.0, 0), (3.0, -1.0, 1)].map(|(up, facing, index)| {
            let facing = if turned[index] { -facing } else { facing };
            face(
                &mut model,
                Surface::Plane(
                    Plane::from_point_and_normal(
                        rcad_geom::Point::new(0.0, 0.0, up),
                        Vector::new(0.0, 0.0, facing),
                    )
                    .expect("the z axis is a unit normal"),
                ),
                &[
                    (band_circles[index], walk[index].reversed()),
                    (floor_circles[index], floor_walk[index]),
                ],
            )
        });
        let floor_face = face(
            &mut model,
            Surface::Cylinder(Cylinder::new(Axis::z(), floor).expect("a positive radius")),
            &[
                (floor_circles[0], floor_walk[0].reversed()),
                (floor_circles[1], floor_walk[1].reversed()),
            ],
        );
        let shell = model.add_shell(
            [
                (wall, Orientation::Forward),
                (sides[0], Orientation::Forward),
                (sides[1], Orientation::Forward),
                (floor_face, floor_used),
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
            loc: placed,
            orient: Orientation::Forward,
        }]));
        let key = model.key_for_solid(solid);
        let wall = model
            .solid(key)
            .expect("this model made the key")
            .faces()
            .find(|face| matches!(face.surface(), Surface::Torus(_)))
            .expect("a grooved ring has a wall")
            .key();
        (model, key, wall)
    }

    /// A groove turned into a ring's rim leaves a wall that holds the ring
    /// less its outer band between the groove's planes, and one turned into
    /// its bore a wall that holds the ring less its inner band (row Q1144).
    ///
    /// Each circle of the wall is behind the other plane, so the wall is the
    /// part of the ring in front of either plane, and between them the part
    /// off the band, by the pipe of radius ten. It was refused
    /// `AFaceMissesPartOfWhereItsSurfacesCross` at every point asked. Moved a
    /// hundred along `x`, each wall holds the same points moved.
    ///
    /// **`Model::validate` passes both grooved rings** (row Q1153). It refused
    /// them `AClosedEdgeIsWalkedAgainstItsSurface`: at a circle round the axis
    /// it read the side of the plane the wall keeps off the wall's corners, and
    /// the only corner off that plane is the other circle's, which is on the
    /// side the wall does not keep: `Pos` where the wire says `Neg`, at all
    /// four circles. It now holds the wall to leaving its two circles going
    /// opposite ways round the tube, and reads no corner there.
    #[test]
    fn a_grooved_rings_wall_holds_the_ring_less_the_band_the_groove_took() {
        let moved = Isometry::translation(Vector::new(100.0, 0.0, 0.0)).expect("a finite shift");
        for outer in [true, false] {
            for (placed, by) in [(Isometry::identity(), 0.0), (moved, 100.0)] {
                let (model, solid, wall) = a_grooved_ring(outer, placed);
                model
                    .validate(&Policy::certified(), &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| {
                        panic!("outer {outer}: a grooved ring is a solid: {why:?}")
                    });
                let ring = model.solid(solid).expect("this model made the key");
                for (index, (x, y, z, on)) in ON_THE_RING.into_iter().enumerate() {
                    let expected = match on {
                        Some(this) if this == outer && index >= 9 => OnAFace::OnItsBoundary,
                        Some(this) if this == outer => OnAFace::NotHeld,
                        Some(_) | None => OnAFace::Held,
                    };
                    let point = at(x + by, y, z);
                    let held = ring
                        .face_holds_the_point(
                            wall,
                            point,
                            &Policy::certified(),
                            &mut ProvenanceLog::new(),
                        )
                        .expect("a ring less a band between two planes is followed");
                    assert_eq!(held, expected, "outer {outer}, {point:?}");
                }
            }
        }
    }

    /// A grooved ring's wall walked any way but the groove's, with the side and
    /// the floor along each circle it walks the other way turned over to match,
    /// is refused `AClosedEdgeIsWalkedAgainstItsSurface` (row Q1153).
    ///
    /// **Walked the other way round one circle**, the wall leaves both its
    /// circles going the same way round the tube, which bounds neither piece
    /// of the ring between them, and `Model::validate` refuses it at the wall,
    /// the first face, by comparing the two circles. **Turned over at both**,
    /// the wall is the band the groove took and is a face, but the floor is
    /// still used the way the groove's is; worked by hand and not traced, the
    /// floor's own corners, off each side's plane, say it is walked against
    /// the sides that turned over. With the comparison switched off in place,
    /// all three are still refused `AClosedEdgeIsWalkedAgainstItsSurface`:
    /// here the comparison names the fault first, and a band's wall
    /// ([`a_bands_wall_walking_both_circles_the_same_way_round_the_tube_is_refused`])
    /// is where it is the only check that does.
    #[test]
    fn a_grooved_rings_wall_walked_any_other_way_is_refused() {
        for outer in [true, false] {
            for walk in EVERY_WALK {
                let (model, _, _) = a_grooved_ring_walked(outer, walk, Isometry::identity());
                let answer = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
                if walk == the_grooves_walk(outer) {
                    assert!(answer.is_ok(), "outer {outer}, walked {walk:?}: {answer:?}");
                } else {
                    assert!(
                        matches!(
                            answer,
                            Err(Error::Invalid(
                                Invalid::AClosedEdgeIsWalkedAgainstItsSurface
                            ))
                        ),
                        "outer {outer}, walked {walk:?}: {answer:?}"
                    );
                }
            }
        }
    }

    /// A band's wall walking both its circles the same way round the tube is
    /// refused `AClosedEdgeIsWalkedAgainstItsSurface`, for the barrel and the
    /// spool, and so is the wall along the outer circle of `z = −3` and the
    /// inner circle of `z = 3` walked so (row Q1154's wall). Each is a wire that
    /// leaves one circle up the outside of the tube and the other down it, and
    /// no piece of the ring does that.
    ///
    /// With the comparison of the two circles switched off in place, and the
    /// corners not read at them, the barrel walked `[Reversed, Reversed]` and
    /// the outer-and-inner wall walked `[Reversed, Reversed]` passed
    /// `Model::validate`. The sound walk of each validates, the outer-and-inner
    /// wall's being the outer band and the top of the tube, and so does the
    /// ring with its hole plugged between the planes: the ring less the inner
    /// band, used as stored, with the two caps facing out.
    ///
    /// With `validate.rs` put back to `55a27b3d`, the first shape this asks
    /// fails: the barrel turned over at both circles was refused
    /// `AClosedEdgeIsWalkedAgainstItsSurface` off the corners, where the mesh
    /// now refuses it `FacesDoNotAgreeWhichWayTheyFace`.
    #[test]
    fn a_bands_wall_walking_both_circles_the_same_way_round_the_tube_is_refused() {
        let facing = |up: f64| {
            Plane::from_point_and_normal(
                rcad_geom::Point::new(0.0, 0.0, up),
                Vector::new(0.0, 0.0, up.signum()),
            )
            .expect("the z axis is a unit normal")
        };
        let a_wall = |near: f64, far: f64, walk, wall_used| {
            a_piece_of_pipe_placed(
                rcad_geom::Torus::new(Axis::z(), 10.0, 5.0).expect("a tube narrower than its ring"),
                End {
                    corner: rcad_geom::Point::new(near, 0.0, -3.0),
                    cap: facing(-3.0),
                    used: Orientation::Forward,
                },
                End {
                    corner: rcad_geom::Point::new(far, 0.0, 3.0),
                    cap: facing(3.0),
                    used: Orientation::Forward,
                },
                None,
                walk,
                wall_used,
                Isometry::identity(),
            )
            .0
        };
        let barrel = (14.0, 14.0, Orientation::Forward);
        let spool = (6.0, 6.0, Orientation::Reversed);
        let plugged = (6.0, 6.0, Orientation::Forward);
        let outer_and_inner = (14.0, 6.0, Orientation::Forward);
        for (near, far, wall_used) in [barrel, spool, plugged, outer_and_inner] {
            for walk in EVERY_WALK {
                let answer = a_wall(near, far, walk, wall_used)
                    .validate(&Policy::certified(), &mut ProvenanceLog::new());
                let shape = format!("{near} to {far}, used {wall_used:?}, walked {walk:?}");
                match walk {
                    [Orientation::Reversed, Orientation::Forward] => {
                        assert!(answer.is_ok(), "{shape}: {answer:?}");
                    }
                    [Orientation::Forward, Orientation::Reversed] => assert!(
                        matches!(
                            answer,
                            Err(Error::Invalid(Invalid::FacesDoNotAgreeWhichWayTheyFace))
                        ),
                        "{shape}: {answer:?}"
                    ),
                    _ => assert!(
                        matches!(
                            answer,
                            Err(Error::Invalid(
                                Invalid::AClosedEdgeIsWalkedAgainstItsSurface
                            ))
                        ),
                        "{shape}: {answer:?}"
                    ),
                }
            }
        }
    }

    /// A ring face along the outer circle of `z = −3` and the inner circle of
    /// `z = 3` is refused, however it is walked and used: no face keeps the
    /// near side of both planes along those two circles.
    #[test]
    fn a_ring_face_along_an_outer_circle_of_one_plane_and_an_inner_of_another_is_refused() {
        let facing = |up: f64| {
            Plane::from_point_and_normal(
                rcad_geom::Point::new(0.0, 0.0, up),
                Vector::new(0.0, 0.0, up.signum()),
            )
            .expect("the z axis is a unit normal")
        };
        for walk in EVERY_WALK {
            for wall_used in [Orientation::Forward, Orientation::Reversed] {
                let (model, solid, wall) = a_piece_of_pipe_placed(
                    rcad_geom::Torus::new(Axis::z(), 10.0, 5.0)
                        .expect("a tube narrower than its ring"),
                    End {
                        corner: rcad_geom::Point::new(14.0, 0.0, -3.0),
                        cap: facing(-3.0),
                        used: Orientation::Forward,
                    },
                    End {
                        corner: rcad_geom::Point::new(6.0, 0.0, 3.0),
                        cap: facing(3.0),
                        used: Orientation::Forward,
                    },
                    None,
                    walk,
                    wall_used,
                    Isometry::identity(),
                );
                let ring = model.solid(solid).expect("this model made the key");
                for point in [at(15.0, 0.0, 0.0), at(5.0, 0.0, 0.0), at(10.0, 0.0, 5.0)] {
                    let answer = ring.face_holds_the_point(
                        wall,
                        point,
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    );
                    assert!(
                        matches!(answer, Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face }) if face == wall),
                        "walked {walk:?}, used {wall_used:?}, {point:?}: {answer:?}"
                    );
                }
            }
        }
    }

    /// The top half of a ring cut by `z = 0`, with the cap along its outer
    /// circle stored facing `−z` and the one along its inner circle stored
    /// facing `+z` and used reversed, holds the top half; walked the other way
    /// round both circles, it holds the bottom half (row Q950).
    ///
    /// The two caps are one plane stored two ways, so they are two trimmings
    /// and were refused `AFaceMissesPartOfWhereItsSurfacesCross`, being two
    /// planes square to the axis that are not one stored plane. [`intersect`]
    /// says they are one plane, and the rule for a half bend stored two ways
    /// answers it. The two walks that keep the same side of both stored
    /// normals keep one half at one circle and the other half at the other,
    /// and are refused.
    #[test]
    fn a_half_rings_wall_holds_its_half_with_one_circles_cap_stored_turned_round() {
        let flat = |facing: f64| {
            Plane::from_point_and_normal(rcad_geom::Point::origin(), Vector::new(0.0, 0.0, facing))
                .expect("the z axis is a unit normal")
        };
        for walk in EVERY_WALK {
            let (model, solid, wall) = a_piece_of_pipe_walking_each_circle(
                rcad_geom::Torus::new(Axis::z(), 10.0, 5.0).expect("a tube narrower than its ring"),
                End {
                    corner: rcad_geom::Point::new(15.0, 0.0, 0.0),
                    cap: flat(-1.0),
                    used: Orientation::Forward,
                },
                End {
                    corner: rcad_geom::Point::new(-5.0, 0.0, 0.0),
                    cap: flat(1.0),
                    used: Orientation::Reversed,
                },
                None,
                walk,
            );
            let half = model.solid(solid).expect("this model made the key");
            let ask = |point| {
                half.face_holds_the_point(
                    wall,
                    point,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
            };
            let up = match walk {
                [Orientation::Reversed, Orientation::Forward] => 1.0,
                [Orientation::Forward, Orientation::Reversed] => -1.0,
                _ => {
                    for point in [at(10.0, 0.0, 5.0), at(10.0, 0.0, -5.0), at(15.0, 0.0, 0.0)] {
                        let answer = ask(point);
                        assert!(
                            matches!(answer, Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face }) if face == wall),
                            "walked {walk:?}, {point:?}: {answer:?}"
                        );
                    }
                    continue;
                }
            };
            let expected = [
                (at(10.0, 0.0, 5.0 * up), OnAFace::Held),
                (at(13.0, 0.0, 4.0 * up), OnAFace::Held),
                (at(0.0, -7.0, 4.0 * up), OnAFace::Held),
                (at(10.0, 0.0, -5.0 * up), OnAFace::NotHeld),
                (at(-13.0, 0.0, -4.0 * up), OnAFace::NotHeld),
                (at(7.0, 0.0, -4.0 * up), OnAFace::NotHeld),
                (at(15.0, 0.0, 0.0), OnAFace::OnItsBoundary),
                (at(0.0, 5.0, 0.0), OnAFace::OnItsBoundary),
                (at(-9.0, -12.0, 0.0), OnAFace::OnItsBoundary),
                (at(3.0, 4.0, 0.0), OnAFace::OnItsBoundary),
            ];
            for (point, answer) in expected {
                let held = ask(point).expect("a half ring is a face this can follow");
                assert_eq!(held, answer, "walked {walk:?}, {point:?}");
            }
        }
    }
}
