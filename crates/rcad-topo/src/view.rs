use std::collections::HashMap;

use rcad_geom::{
    EdgeRun, Isometry, Orientation, Plane, PlaneAgreement, Point, Segment, Surface, Vector,
};
use rcad_predicates::{Approximation, Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use smallvec::SmallVec;

use crate::key::{CoedgeSlot, EdgeSlot, FaceSlot, ShellSlot, SolidSlot, VertexSlot, WireSlot};
use crate::model::{
    CoedgeData, EdgeData, FaceData, ShellData, SolidData, VertexData, WireData, WireStep,
};
use crate::{
    CoedgeKey, EdgeKey, Error, FaceKey, Model, OnAFace, Result, ShellKey, SolidKey, VertexKey,
    WireKey,
};

/// A solid, with the location and orientation it is being looked at through.
///
/// Views are how anything reads geometry out of a [`Model`]. They carry the
/// borrow *and* the transform composed on the way down, so a plane read out of
/// a face has already been moved into place. There is deliberately no way to
/// get at an entity's stored data instead (decisions.md §3.3).
#[derive(Clone, Copy, Debug)]
pub struct SolidView<'m> {
    model: &'m Model,
    slot: SolidSlot,
    loc: Isometry,
    orient: Orientation,
}

impl<'m> SolidView<'m> {
    pub(crate) fn new(
        model: &'m Model,
        slot: SolidSlot,
        loc: Isometry,
        orient: Orientation,
    ) -> Self {
        Self {
            model,
            slot,
            loc,
            orient,
        }
    }

    /// A key naming this solid.
    #[must_use]
    pub fn key(self) -> SolidKey {
        SolidKey::new(self.slot, self.model.id())
    }

    /// The model this solid lives in.
    #[must_use]
    pub fn model(self) -> &'m Model {
        self.model
    }

    /// Where this solid has been placed.
    #[must_use]
    pub fn location(self) -> Isometry {
        self.loc
    }

    /// Which way round this solid is being used.
    #[must_use]
    pub fn orientation(self) -> Orientation {
        self.orient
    }

    /// The solid's shells, each carrying this solid's placement composed with
    /// its own.
    pub fn shells(self) -> impl Iterator<Item = ShellView<'m>> {
        let (model, loc, orient) = (self.model, self.loc, self.orient);
        self.data().shells.iter().map(move |used| {
            ShellView::new(
                model,
                used.shell,
                loc.compose(&used.loc),
                used.orient.composed_with(orient),
            )
        })
    }

    /// Every face of every shell, already located and oriented.
    pub fn faces(self) -> impl Iterator<Item = FaceView<'m>> {
        self.shells().flat_map(ShellView::faces)
    }

    /// Whether every face of this solid sits on a plane.
    ///
    /// The one question every operation that only knows how to handle flat
    /// faces asks first, and it is one function rather than one per operation
    /// on purpose: two independently written guards is the seam between two
    /// units that `docs/decisions.md` §13 is about, and the fuse and the
    /// classifier are two doors into the same assumption.
    ///
    /// Asked over **every** face before anything is examined, so which answer a
    /// caller gets cannot depend on the order the faces happened to be walked.
    /// The commutativity properties compare answers across exactly that
    /// reordering.
    #[must_use]
    pub fn is_wholly_flat(self) -> bool {
        self.faces().all(|face| face.surface().as_plane().is_some())
    }

    /// The smallest axis-aligned box holding every corner of this solid.
    ///
    /// **Not a box round the solid**, and the name says so because for a curved
    /// solid the two are different things. A round bar has two corners, both on
    /// its seam, so the box round them is a line — while the bar bulges a
    /// radius away from it on every side. A ball has two corners, its poles,
    /// and the box round those misses the whole equator.
    ///
    /// This is the shape of mistake `docs/decisions.md` §23.8 records four
    /// times over: a helper that meant one thing until a new kind of input
    /// arrived. It used to be called a bounding box, and it was one for every
    /// solid that existed when it was written. A box round a *curved* solid
    /// needs the mesh, and §25.9 hands it to the lane that builds one.
    ///
    /// `None` when the solid fans into no triangles at all — which for a
    /// polyhedron means it has no faces, and which a **ball** also manages: its
    /// one face has two corners, and two corners make no triangle.
    #[must_use]
    pub fn box_around_its_corners(self) -> Option<(Point, Point)> {
        let mut low = [f64::INFINITY; 3];
        let mut high = [f64::NEG_INFINITY; 3];
        let mut seen = false;
        for face in self.faces() {
            face.for_each_triangle(|corners| {
                seen = true;
                for corner in corners {
                    let coordinates = corner.coordinates();
                    for axis in 0..3 {
                        low[axis] = low[axis].min(coordinates[axis]);
                        high[axis] = high[axis].max(coordinates[axis]);
                    }
                }
            });
        }
        seen.then(|| (Point::from(low), Point::from(high)))
    }

    /// An axis-aligned box round every stored corner of the solid and every
    /// curved face whole, or `None` when it has no faces at all.
    ///
    /// **This is the one [`SolidView::box_around_its_corners`] is not**: it
    /// holds how far a curved face bulges past its corners, and a ray target
    /// picked from it is clear of every face.
    ///
    /// **It is not a box the whole solid fits inside, so a point past it is not
    /// proved outside by it alone.** A face is its stored plane
    /// (`docs/decisions.md` §17.1), and a turn leaves a corner a rounding off
    /// where its planes meet, so the solid those planes bound can reach past
    /// this box at its outermost corner. A cube of side 2 turned `2.5` about
    /// `(1, 1, 1)` holds a point three spacings above this box that every one of
    /// its six planes puts inside, and forty turned cubes hold 3897 points past
    /// theirs (`tests/a_point_a_rounding_past_a_turned_solids_corner.rs`,
    /// Q1157, Q1331). A proof that a point, or another solid, is clear of this
    /// one reads this box opened out to where the planes meet
    /// (`SolidView::box_opened_out_to_where_its_planes_meet`).
    ///
    /// A flat face is the box round its own corners, exactly as before. The two
    /// curved ones bulge away from theirs and each is bounded by an argument
    /// rather than by a fudge factor:
    ///
    /// - A **ball**'s skin is inside the box round its centre a radius each way,
    ///   whatever the face is trimmed down to. Nothing tighter is available
    ///   without knowing where the trimming runs, and a ball's face may reach
    ///   right round.
    /// - A **pipe** or a **cone** is bounded by how far along its axis the face
    ///   reaches, opened out by how far from the axis the surface is there.
    ///   How far along is *not* settled by the corners alone — a side that is a
    ///   whole ellipse has one corner — and [`box_the_surface_bounds`] gives the
    ///   argument for what does settle it.
    ///
    /// # Panics
    ///
    /// On a surface this has not been taught. [`Surface`] grows a variant at
    /// every milestone, and a solid measured by not being measured is worse than
    /// a stop.
    #[must_use]
    pub fn box_around_it(self) -> Option<(Point, Point)> {
        let mut round_them_all: Option<(Point, Point)> = None;
        for face in self.faces() {
            let Some(round_this_one) = self.box_around_one_face(face) else {
                continue;
            };
            round_them_all = Some(match round_them_all {
                None => round_this_one,
                Some(so_far) => both_boxes(so_far, round_this_one),
            });
        }
        round_them_all
    }

    /// The box from `low` to `high`, opened out to hold every corner of a flat
    /// face where that face's stored plane meets the stored planes across its
    /// two straight sides there; `None` where three such planes cannot be
    /// shown to meet in a point.
    ///
    /// Handed [`SolidView::box_around_it`], this is the box a proof that
    /// something is clear of the solid reads, for the reason that one's doc
    /// gives: the solid the stored planes bound reaches a rounding past the
    /// stored corners at a turned corner.
    ///
    /// Each straight side of a flat face's outline runs along where its plane
    /// meets the plane across that side, from one meeting of three planes to
    /// the next, and a flat outline lies inside the box round its corners. So
    /// this box holds every flat face along its straight sides between flat
    /// faces, and the stored box holds the rest as it did. A corner where four
    /// or more faces meet is placed once from each face's side, and each of
    /// those places is taken in. A corner whose two sides run along one plane
    /// is no corner of the outline, which runs straight on through it, and is
    /// passed over.
    ///
    /// **A corner beside a curved face is not opened out here**: the stored
    /// box holds a face on a pipe or a cone by the discs across its axis, and
    /// a corner on that face is inside those discs. The stored box holds a
    /// tipped disc's whole reach itself, rounded outward
    /// ([`box_round_a_disc`], Q1363), so no caller has to open that out.
    /// Where a plane and a curved surface meet a rounding past the discs is
    /// not asked here either way.
    ///
    /// Asked only where the stored box has already answered, since a solid of a
    /// thousand faces is a thousand meetings worked out.
    pub(crate) fn box_opened_out_to_where_its_planes_meet(
        self,
        low: Point,
        high: Point,
    ) -> Option<(Point, Point)> {
        let planes: HashMap<FaceKey, Plane> = self
            .faces()
            .filter_map(|face| Some((face.key(), *face.surface().as_plane()?)))
            .collect();
        let (mut low, mut high) = (low.coordinates(), high.coordinates());
        for face in self.faces() {
            let Some(flat) = planes.get(&face.key()) else {
                continue;
            };
            for wire in face.wires() {
                let across: Vec<Option<&Plane>> = wire
                    .coedges()
                    .map(|coedge| {
                        let edge = coedge.edge();
                        if *edge.stored_run() != EdgeRun::Straight {
                            return None;
                        }
                        edge.neighbouring_faces()
                            .find(|&other| other != face.key())
                            .and_then(|other| planes.get(&other))
                    })
                    .collect();
                let next_round = across.iter().cycle().skip(1);
                for (into, out_of) in across.iter().zip(next_round) {
                    let (Some(into), Some(out_of)) = (into, out_of) else {
                        continue;
                    };
                    if matches!(
                        into.agreement_with(out_of),
                        PlaneAgreement::SameWay | PlaneAgreement::OppositeWay
                    ) {
                        continue;
                    }
                    let meet = Plane::meet_of_three(flat, into, out_of).ok()?;
                    for (axis, side) in meet.enclosure().into_iter().enumerate() {
                        if !(side.lo().is_finite() && side.hi().is_finite()) {
                            return None;
                        }
                        low[axis] = low[axis].min(side.lo());
                        high[axis] = high[axis].max(side.hi());
                    }
                }
            }
        }
        Some((Point::from(low), Point::from(high)))
    }

    /// An axis-aligned box one face of this solid fits inside, or `None` when
    /// there is nothing of it to measure.
    ///
    /// **Not the box round the face's corners**, and for a curved face the two
    /// are nothing like each other: a round bar's wall has two corners, both on
    /// its seam, and the wall bulges a whole radius away from the line between
    /// them. A round cap has one corner and fans into no triangles at all, so a
    /// box read off [`FaceView::for_each_triangle`] is empty. A broad phase
    /// reading either would skip a pair of faces that really do cross.
    ///
    /// A **flat** face is the box round its corners, plus the box round each
    /// face across a side of it that is not straight. A bar's cap is bounded by
    /// a circle with one corner on it, and what holds that circle is the wall it
    /// is shared with. The two faces along an edge both hold it, so borrowing
    /// the neighbour's box is a fact about the model rather than a fudge. It is
    /// a loose box and deliberately so — a cap of a bar comes out as tall as the
    /// bar. Tightening it means bounding where a *curve* runs, and a box that
    /// only ever decides which pairs to look at does not need one.
    ///
    /// It hangs off the solid rather than off the face for the reason
    /// `docs/decisions.md` §3.2 gives about every neighbour question: a face
    /// read out of the arena by key carries no location, and the box round the
    /// stored surface is not the box round the one this solid is using.
    ///
    /// # Panics
    ///
    /// On a surface this has not been taught. [`Surface`] grows a variant at
    /// every milestone, and a face measured by not being measured is worse than
    /// a stop.
    #[must_use]
    pub fn box_around_one_face(self, face: FaceView<'m>) -> Option<(Point, Point)> {
        let mut round_it = box_the_surface_bounds(self, face);
        if face.surface().as_plane().is_none() {
            return round_it;
        }
        for coedge in face.wires().flat_map(WireView::coedges) {
            let edge = coedge.edge();
            if *edge.stored_run() == EdgeRun::Straight {
                continue;
            }
            let Some(across) = edge
                .neighbouring_faces()
                .find(|&other| other != face.key())
                .and_then(|other| self.faces().find(|used| used.key() == other))
            else {
                continue;
            };
            let side = box_round_a_side_on_an_axis(face.surface(), across.surface(), edge);
            if let Some(theirs) = side.or_else(|| box_the_surface_bounds(self, across)) {
                round_it = Some(round_it.map_or(theirs, |mine| both_boxes(mine, theirs)));
            }
        }
        round_it
    }

    fn data(self) -> &'m SolidData {
        &self.model.data().solids[self.slot]
    }
}

/// A box holding everything the face's own surface can reach, given where its
/// corners are.
///
/// - A **ball**'s skin is inside the box round its centre a radius each way,
///   whatever the face is trimmed down to. Nothing tighter is available without
///   knowing where the trimming runs, and a ball's face may reach right round.
/// - A **pipe**'s skin is a radius from its axis, so the face is inside the box
///   round the discs at the least and the most it reaches along the axis.
/// - A **cone**'s skin is `slope · |up|` from its axis at `up` along it, so the
///   face is inside the box round the discs at the least and the most it
///   reaches along the axis, together with the apex: every point of a cone
///   between two heights on one half of it lies on a straight ruling between
///   the circle at the higher one and the apex, so inside the hull of that disc
///   and the apex.
/// - A **plane** bounds nothing on its own, so this is the box round the
///   corners, which holds the face only while every side of it is straight.
///   [`SolidView::box_around_one_face`] is what opens that out.
///
/// # How far along its axis a face on a pipe or a cone reaches
///
/// Measured as `up`, the distance along the axis from its origin (the apex, on
/// a cone). **A face has no high or low spot in `up` anywhere inside it except
/// at a cone's apex**: through every other point of either surface runs a
/// straight ruling, and `up` changes at a steady non-zero rate along it. So the
/// least and the most `up` on a face are on its boundary, or at the apex.
///
/// The apex is taken in unless the face is shown not to hold it
/// ([`the_face_may_reach_its_apex`]). A pointed cone needs it: its only corner
/// is on its cap, so without the apex the box stopped at the cap's own height
/// less a radius, and `classify_point` answered `Outside` for everything below
/// it (`docs/decisions.md` §200.3). A frustum must not have it: its side stops
/// at its small cap, and a box reaching down to the apex handed the fuse pairs
/// of faces that never meet (§202.4).
///
/// **The boundary is not settled by the corners.** A straight side is a ruling,
/// along which `up` runs steadily from one end to the other, so its ends — two
/// corners — are the whole of its reach. A curved side reaches past its ends
/// wherever `up` turns along it, and a side that is one whole closed curve has
/// one corner and turns twice: a pipe cut by a tilted plane has an ellipse
/// there, its one corner may be the lowest point, and a box read off corners
/// stopped there (`a_box_holds_every_curved_face.rs`). So each curved side adds
/// every `up` at which the crossing it runs along can turn
/// ([`where_up_can_turn_along_a_crossing`]); an arc that does not reach one of
/// those only makes the box wider, never wrong.
///
/// **A curved side whose other surface is a pipe or a cone adds nothing**, and
/// that is a gap and not a proof: those crossings are degree four and nothing
/// here yet says where `up` turns along one. Such a side is bounded by its
/// corners as before.
fn box_the_surface_bounds(solid: SolidView<'_>, face: FaceView<'_>) -> Option<(Point, Point)> {
    let mut low = [f64::INFINITY; 3];
    let mut high = [f64::NEG_INFINITY; 3];
    let mut seen = false;
    let mut take_in = |(from, to): (Point, Point)| {
        seen = true;
        let (from, to) = (from.coordinates(), to.coordinates());
        for axis in 0..3 {
            low[axis] = low[axis].min(from[axis]);
            high[axis] = high[axis].max(to[axis]);
        }
    };
    let every_way_round = |centre: Point, by: f64| {
        let centre = centre.coordinates();
        (
            Point::from(centre.map(|at| at - by)),
            Point::from(centre.map(|at| at + by)),
        )
    };
    match face.surface() {
        Surface::Plane(_) => {
            for corner in face.wires().flat_map(WireView::points) {
                take_in((corner, corner));
            }
        }
        Surface::Sphere(ball) => take_in(every_way_round(ball.centre(), ball.radius())),
        Surface::Cylinder(pipe) => {
            let round = RoundAnAxis::Pipe(pipe);
            for up in how_far_along_its_axis_a_face_reaches(solid, face, round) {
                take_in(box_round_a_disc(round, up));
            }
        }
        Surface::Cone(cone) => {
            let round = RoundAnAxis::Cone(cone);
            let reach = how_far_along_its_axis_a_face_reaches(solid, face, round);
            let apex_too = the_face_may_reach_its_apex(solid, face, &reach).then_some(0.0);
            for up in apex_too.into_iter().chain(reach) {
                take_in(box_round_a_disc(round, up));
            }
        }
        // **The whole torus and not the corners' own reach**, unlike a pipe's
        // arm above. A pipe's face cannot leave the tube of its own radius round
        // the stretch of axis its corners span, so the shadows bound it. A torus
        // has *two* ways round and a face that goes right round either of them
        // reaches places no corner of it shadows — so what is bounded here is
        // the surface, which is inside `ring + tube` of its centre whichever way
        // the face wraps.
        Surface::Torus(ring) => take_in(every_way_round(
            ring.centre(),
            ring.ring_radius() + ring.tube_radius(),
        )),
        other => panic!("this crate cannot say how far {other:?} reaches"),
    }
    seen.then(|| (Point::from(low), Point::from(high)))
}

/// Whether a face on a cone may hold the cone's apex, so that its box has to.
///
/// `false` only where the face is **shown** not to: its boundary reaches only
/// one side of the apex along the axis, and the trimming test
/// ([`SolidView::what_this_face_holds`]) answers that the apex is off the face.
/// A frustum's side is the case: its small cap's plane has the apex on the far
/// side from the big cap. A pointed cone's side answers that it holds it.
///
/// Anything else takes the apex in — a boundary on both sides of it, or a
/// trimming test that refuses — because a box only has to hold the face, and a
/// refusal costs width and never correctness. The test is run under a
/// certified policy and what it records is dropped, since it is a bound and
/// not an answer: a stricter or looser policy could only move the box between
/// two sizes that both hold the face.
fn the_face_may_reach_its_apex(solid: SolidView<'_>, face: FaceView<'_>, reach: &[f64]) -> bool {
    let (Some(least), Some(most)) = (
        reach.iter().copied().reduce(f64::min),
        reach.iter().copied().reduce(f64::max),
    ) else {
        return true;
    };
    if least <= 0.0 && most >= 0.0 {
        return true;
    }
    let Surface::Cone(cone) = face.surface() else {
        panic!("only a face on a cone has an apex to reach");
    };
    let apex = PredicatePoint::from(cone.apex().coordinates());
    !matches!(
        solid.what_this_face_holds(face, apex, &Policy::certified(), &mut ProvenanceLog::new()),
        Ok(OnAFace::NotHeld)
    )
}

/// A pipe or a cone: the two surfaces a face on which is measured along an
/// axis.
///
/// Exhaustive on purpose: it names the two arms of [`box_the_surface_bounds`]
/// that share [`how_far_along_its_axis_a_face_reaches`], and nothing else.
#[derive(Clone, Copy)]
enum RoundAnAxis {
    Pipe(rcad_geom::Cylinder),
    Cone(rcad_geom::Cone),
}

impl RoundAnAxis {
    /// Where `up` is measured from: a point on a pipe's axis, a cone's apex.
    fn origin(self) -> Point {
        match self {
            Self::Pipe(pipe) => pipe.axis().origin(),
            Self::Cone(cone) => cone.apex(),
        }
    }

    fn along(self) -> Vector {
        match self {
            Self::Pipe(pipe) => pipe.axis().direction(),
            Self::Cone(cone) => cone.axis().direction(),
        }
    }

    fn up_of(self, point: Point) -> f64 {
        self.along().dot(point - self.origin())
    }
}

/// Every `up` the face's boundary can reach its least or its most at: each
/// corner, and each place a curved side can turn — the argument is on
/// [`box_the_surface_bounds`].
///
/// A curved side lies on this face's surface and on each surface across it: the
/// face on its other side, and for a piece of a crossing the surface named with
/// it. Any one of those that says where `up` can turn is enough.
fn how_far_along_its_axis_a_face_reaches(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    round: RoundAnAxis,
) -> SmallVec<[f64; 8]> {
    let mut reach: SmallVec<[f64; 8]> = face
        .wires()
        .flat_map(WireView::points)
        .map(|corner| round.up_of(corner))
        .collect();
    for coedge in face.wires().flat_map(WireView::coedges) {
        let edge = coedge.edge();
        if *edge.stored_run() == EdgeRun::Straight {
            continue;
        }
        let mut across: SmallVec<[Surface; 2]> = edge
            .neighbouring_faces()
            .filter(|&other| other != face.key())
            .filter_map(|other| solid.faces().find(|used| used.key() == other))
            .map(FaceView::surface)
            .collect();
        if let EdgeRun::PartOfWhereItCrosses { and, .. } = edge.run() {
            across.push(and);
        }
        if let Some(turns) = across
            .iter()
            .find_map(|other| where_up_can_turn_along_a_crossing(round, other))
        {
            reach.extend(turns);
        }
    }
    reach
}

/// Every `up` at which the crossing of `round` with `other` can turn, or
/// `None` when this cannot say.
///
/// - **A plane** `n · x = n · p`. Write a point of a pipe of radius `r` as
///   `origin + up · a + r · w`, with `w` a unit direction square to the axis
///   `a`; on the plane, `up · (n · a) + r · (n · w) = d`, where
///   `d = n · (p − origin)`. As `w` goes round, `s = n · w` runs between `±m`,
///   `m = √(1 − (n · a)²)`, and `up = (d − r · s) / (n · a)` turns only where
///   `s` does — at `s = ±m`. A plane along the axis (`n · a = 0`) crosses a
///   pipe in lines along it, on which `up` never turns. A cone of slope `k` is
///   the same with `r = k · |up|`: `up = d / (n · a ± k · s)`, the sign by the
///   half of the cone, which turns only at `s = ±m` too, giving
///   `d / (n · a ± k · m)` on either half. Where that divisor is zero the turn
///   is at infinity, and not on any side.
/// - **A ball or a ring**, whose whole surface lies within a known reach of its
///   centre, so the crossing does too.
fn where_up_can_turn_along_a_crossing(
    round: RoundAnAxis,
    other: &Surface,
) -> Option<SmallVec<[f64; 2]>> {
    let reach_round = |centre: Point, reach: f64| {
        let middle = round.up_of(centre);
        [middle - reach, middle + reach].into_iter().collect()
    };
    // `if let` rather than a `match`: `Surface` is `#[non_exhaustive]`, and a
    // kind this does not name is the `None` the caller already handles, so a
    // wildcard arm would only restate that.
    if let Surface::Sphere(ball) = other {
        return Some(reach_round(ball.centre(), ball.radius()));
    }
    if let Surface::Torus(ring) = other {
        return Some(reach_round(
            ring.centre(),
            ring.ring_radius() + ring.tube_radius(),
        ));
    }
    let flat = other.as_plane()?;
    let across = flat.normal().dot(round.along());
    let d = -flat.signed_distance_to(round.origin());
    let most = (1.0 - across * across).max(0.0).sqrt();
    let turns: SmallVec<[f64; 2]> = match round {
        RoundAnAxis::Pipe(_) if across == 0.0 => SmallVec::new(),
        RoundAnAxis::Pipe(pipe) => [-most, most]
            .into_iter()
            .map(|s| (d - pipe.radius() * s) / across)
            .collect(),
        RoundAnAxis::Cone(cone) => [across + cone.slope() * most, across - cone.slope() * most]
            .into_iter()
            .filter(|&divisor| divisor != 0.0)
            .map(|divisor| d / divisor)
            .collect(),
    };
    // The formula rounds a handful of times, and a slanted bar's ellipse top at
    // `z = 2` came back as `1.9999999999999998`. Every turn is taken a little
    // either way — `2⁻⁴⁰` of the sizes it was worked out from — so the rounding
    // widens the box rather than cutting the face. That margin is measured, not
    // derived: a divisor itself near rounding is not covered by it.
    let sizes = round.origin().largest_coordinate_magnitude() + flat.offset().abs() + 1.0;
    Some(
        turns
            .into_iter()
            .flat_map(|up| {
                let margin = (up.abs() + sizes) * 2f64.powi(-40);
                [up - margin, up + margin]
            })
            .collect(),
    )
}

/// A box round one curved side of a flat face, where the face across it is on
/// a pipe or a cone, or `None` when it is on anything else.
///
/// The side lies where the plane crosses that surface, so how far along the
/// surface's axis it reaches is settled by its two ends and the places `up`
/// can turn along the crossing ([`where_up_can_turn_along_a_crossing`]), and
/// the box round the discs there holds it. A flat face lies inside the hull of
/// its own boundary, so these boxes and its corners hold the whole face —
/// which is tighter than the whole of the face across, and a pointed cone's cap
/// is why that matters: borrowing the skin's box stretched the cap down to the
/// apex, and the fuse then cut the cap along planes it never reaches.
///
/// No apex is needed for a cone: a plane that misses the apex crosses the cone
/// where `up` is never zero, so one side stays on one half, between the discs
/// at its least and its most `up`. A plane through the apex turns at `up = 0`,
/// which puts the apex in.
fn box_round_a_side_on_an_axis(
    flat: Surface,
    across: Surface,
    edge: EdgeView<'_>,
) -> Option<(Point, Point)> {
    let round = if let Surface::Cylinder(pipe) = across {
        RoundAnAxis::Pipe(pipe)
    } else if let Surface::Cone(cone) = across {
        RoundAnAxis::Cone(cone)
    } else {
        return None;
    };
    let turns = where_up_can_turn_along_a_crossing(round, &flat)?;
    let ends = [edge.start().point(), edge.end().point()].map(|end| round.up_of(end));
    let mut round_it: Option<(Point, Point)> = None;
    for up in ends.into_iter().chain(turns) {
        let disc = box_round_a_disc(round, up);
        round_it = Some(round_it.map_or(disc, |so_far| both_boxes(so_far, disc)));
    }
    round_it
}

/// The box round the disc across `round`'s axis at `up` along it, rounded
/// outward so that it holds every point of the disc.
///
/// **A disc of radius `r` square to the stored axis `a` reaches
/// `r · √(b² + c²)/|a|` along a coordinate axis**, `b` and `c` being the
/// axis's two parts across that coordinate axis. It was worked out as
/// `r · √(1 − a²)` from the part along it until Q1363, and where that part is
/// near one the subtraction keeps nothing but rounding. A bar tipped `1e-9` off
/// `z` stores an axis whose `z` part is exactly one, so its discs reached
/// nothing in `z` by that sum where they truly reach a billionth of the radius,
/// and the stored surfaces put points a billionth past the box inside
/// (`tests/a_point_a_rounding_past_a_turned_solids_corner.rs`, Q1332, Q1363).
/// A stored axis is of length one only to within a turn, so `|a|` is carried
/// rather than taken as one.
///
/// The disc's centre is `origin + a · up/|a|²` on either surface. A cone's
/// radius there is `slope · |up|/|a|`, so its reach is
/// `slope · |up| · √(b² + c²)/|a|²`.
///
/// **Every step rounds outward**: the squares and the sums and products in
/// interval arithmetic, the quotients and the root to nearest and then a step
/// outward unless the rounding is shown to be exact ([`quotient_of`],
/// [`root_rounded_up`]). So a bar along a coordinate axis whose numbers are
/// whole gets the box it always had, bit for bit, and a tipped one a rounding
/// wider where it needs one. What is still rounded to nearest is `up` itself
/// ([`RoundAnAxis::up_of`]), which the caller works out.
///
/// An `up` that is not a finite number is a disc nobody can place, and the box
/// round it is every point.
fn box_round_a_disc(round: RoundAnAxis, up: f64) -> (Point, Point) {
    if !up.is_finite() {
        return (
            Point::from([f64::NEG_INFINITY; 3]),
            Point::from([f64::INFINITY; 3]),
        );
    }
    let parts = round.along().components();
    let squares = parts.map(|part| Interval::exactly(part) * Interval::exactly(part));
    let length = squares[0] + squares[1] + squares[2];
    assert!(
        length.lo() > 0.0,
        "an axis of length one to within a turn has a length: {parts:?}"
    );
    let [over_least, over_most] = [length.lo(), length.hi()].map(|bottom| quotient_of(up, bottom));
    let along_it = Interval::new(
        over_least.lo().min(over_most.lo()),
        over_least.hi().max(over_most.hi()),
    )
    .expect("two quotients of one finite number are a range");
    let (radius, over) = match round {
        RoundAnAxis::Pipe(pipe) => (Interval::exactly(pipe.radius()), length),
        RoundAnAxis::Cone(cone) => (
            Interval::exactly(cone.slope()) * Interval::exactly(up.abs()),
            length * length,
        ),
    };
    let origin = round.origin().coordinates();
    let ends = [0, 1, 2].map(|axis| {
        let across = squares[(axis + 1) % 3] + squares[(axis + 2) % 3];
        let spread = root_rounded_up(quotient_of(across.hi(), over.lo()).hi());
        let reach = Interval::exactly((radius * Interval::exactly(spread)).hi());
        let centre = Interval::exactly(origin[axis]) + Interval::exactly(parts[axis]) * along_it;
        ((centre - reach).lo(), (centre + reach).hi())
    });
    (
        Point::from(ends.map(|(low, _)| low)),
        Point::from(ends.map(|(_, high)| high)),
    )
}

/// `top / bottom` for a `bottom` above zero, as a range that holds it: the
/// quotient rounded to nearest, and the next `f64` on whichever side the
/// rounding went, or neither where it was exact.
///
/// Which side is read off `top − quotient · bottom`, which one fused
/// multiply-add works out exactly: the remainder of a division rounded to
/// nearest is itself an `f64` while nothing is near the subnormals. Where
/// something is, both sides are taken.
fn quotient_of(top: f64, bottom: f64) -> Interval {
    debug_assert!(bottom > 0.0, "a quotient over {bottom}");
    if top == 0.0 {
        return Interval::exactly(0.0);
    }
    let quotient = top / bottom;
    if !quotient.is_finite() {
        return Interval::ENTIRE;
    }
    let near_the_subnormals = [top, bottom, quotient]
        .iter()
        .any(|value| value.abs() < 1e-270);
    let (low, high) = if near_the_subnormals {
        (quotient.next_down(), quotient.next_up())
    } else {
        let left_over = (-quotient).mul_add(bottom, top);
        if left_over > 0.0 {
            (quotient, quotient.next_up())
        } else if left_over < 0.0 {
            (quotient.next_down(), quotient)
        } else {
            (quotient, quotient)
        }
    };
    Interval::new(low, high).expect("a finite quotient and its neighbours are a range")
}

/// The square root of `square`, which is at least zero, rounded up: the root
/// rounded to nearest, or the next `f64` above it where that came out short.
///
/// Whether it came out short is the sign of `root² − square`, which one fused
/// multiply-add gets right while nothing is near the subnormals.
fn root_rounded_up(square: f64) -> f64 {
    debug_assert!(square >= 0.0, "a root of {square}");
    if square == 0.0 {
        return 0.0;
    }
    let root = square.sqrt();
    if square < 1e-270 || root.mul_add(root, -square) < 0.0 {
        root.next_up()
    } else {
        root
    }
}

/// The smallest box holding both of two boxes.
fn both_boxes(one: (Point, Point), other: (Point, Point)) -> (Point, Point) {
    let (low, high) = (
        [one.0.coordinates(), other.0.coordinates()],
        [one.1.coordinates(), other.1.coordinates()],
    );
    (
        Point::from([0, 1, 2].map(|axis| low[0][axis].min(low[1][axis]))),
        Point::from([0, 1, 2].map(|axis| high[0][axis].max(high[1][axis]))),
    )
}

/// A shell, with the location and orientation it is being looked at through.
#[derive(Clone, Copy, Debug)]
pub struct ShellView<'m> {
    model: &'m Model,
    slot: ShellSlot,
    loc: Isometry,
    orient: Orientation,
}

impl<'m> ShellView<'m> {
    pub(crate) fn new(
        model: &'m Model,
        slot: ShellSlot,
        loc: Isometry,
        orient: Orientation,
    ) -> Self {
        Self {
            model,
            slot,
            loc,
            orient,
        }
    }

    /// A key naming this shell.
    #[must_use]
    pub fn key(self) -> ShellKey {
        ShellKey::new(self.slot, self.model.id())
    }

    /// Where this shell has been placed.
    #[must_use]
    pub fn location(self) -> Isometry {
        self.loc
    }

    /// Which way round this shell is being used.
    #[must_use]
    pub fn orientation(self) -> Orientation {
        self.orient
    }

    /// How many faces make up the skin.
    #[must_use]
    pub fn face_count(self) -> usize {
        self.data().faces.len()
    }

    /// The shell's faces, each carrying this shell's placement composed with
    /// its own.
    pub fn faces(self) -> impl Iterator<Item = FaceView<'m>> {
        let (model, loc, orient) = (self.model, self.loc, self.orient);
        self.data().faces.iter().map(move |used| {
            FaceView::new(
                model,
                used.face,
                loc.compose(&used.loc),
                used.orient.composed_with(orient),
            )
        })
    }

    /// The same faces as [`ShellView::faces`], but as the raw references that
    /// carry the placement — the shape decisions.md §3.2 names.
    ///
    /// The location on a [`FaceRef`] is the one stored on the shell, *not* the
    /// composed one. [`ShellView::faces`] is what you want unless you are
    /// looking at the reference itself.
    pub fn face_refs(self) -> impl Iterator<Item = FaceRef> {
        let model = self.model;
        self.data().faces.iter().map(move |used| {
            FaceRef::new(FaceKey::new(used.face, model.id()), used.loc, used.orient)
        })
    }

    fn data(self) -> &'m ShellData {
        &self.model.data().shells[self.slot]
    }
}

/// A reference to a face: which face, where it sits, and which way round.
///
/// This is the split OCCT hides behind `TopoDS_Shape`/`TopoDS_TShape`, written
/// out in the types. The [`FaceKey`] is the shared node; this is one use of it.
/// Because geometry is only reachable through a view built from one of these,
/// there is no way to read a face's plane without composing its location
/// (decisions.md §3.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FaceRef {
    face: FaceKey,
    loc: Isometry,
    orient: Orientation,
}

impl FaceRef {
    /// Use `face` at `loc`, turned `orient`.
    #[must_use]
    pub fn new(face: FaceKey, loc: Isometry, orient: Orientation) -> Self {
        Self { face, loc, orient }
    }

    /// Which face is being used.
    #[must_use]
    pub fn face(self) -> FaceKey {
        self.face
    }

    /// Where it is used.
    #[must_use]
    pub fn location(self) -> Isometry {
        self.loc
    }

    /// Which way round it is used.
    #[must_use]
    pub fn orientation(self) -> Orientation {
        self.orient
    }
}

/// A face, with the location and orientation it is being looked at through.
#[derive(Clone, Copy, Debug)]
pub struct FaceView<'m> {
    model: &'m Model,
    slot: FaceSlot,
    loc: Isometry,
    orient: Orientation,
}

impl<'m> FaceView<'m> {
    pub(crate) fn new(
        model: &'m Model,
        slot: FaceSlot,
        loc: Isometry,
        orient: Orientation,
    ) -> Self {
        Self {
            model,
            slot,
            loc,
            orient,
        }
    }

    /// A key naming this face.
    #[must_use]
    pub fn key(self) -> FaceKey {
        FaceKey::new(self.slot, self.model.id())
    }

    /// Where this face has been placed.
    #[must_use]
    pub fn location(self) -> Isometry {
        self.loc
    }

    /// Which way round this face is being used.
    #[must_use]
    pub fn orientation(self) -> Orientation {
        self.orient
    }

    /// The surface this face sits on, already moved into place.
    #[must_use]
    pub fn surface(self) -> Surface {
        self.data().surface.transformed(&self.loc)
    }

    /// The surface in the frame this face is **stored** in, with this view's
    /// location left off.
    ///
    /// Not public, for [`EdgeView::stored_run`]'s reason: it is the raw arena
    /// payload and un-located geometry is what the views exist to stop being
    /// handed out. The mesher wants it because the frame it meshes a placed face
    /// in *is* the frame the face is stored in, so moving the placed surface
    /// there is a movement composed with its own inverse — the same surface,
    /// rounded twice for nothing.
    pub(crate) fn stored_surface(self) -> Surface {
        self.data().surface
    }

    /// The plane this face sits on, already moved into place.
    ///
    /// # Panics
    ///
    /// If the face sits on something other than a plane.
    ///
    /// `validate` used to be what made that unreachable, and **it is not any
    /// more**: a curved face is a sound face now, and `add_round_bar` builds
    /// one. What keeps every *in-crate* caller sound is
    /// [`SolidView::is_wholly_flat`], asked by every operation that only handles
    /// flat faces before it walks anything, so reaching this from inside the
    /// kernel is a bug in this crate.
    ///
    /// **From outside it is three lines and no bug at all.** This is `pub`,
    /// [`Model::face`](crate::Model::face) hands out a [`FaceView`] for any key,
    /// and a bar's wall is curved. That is a documented panic on a stated
    /// precondition and not §16.11's kind — [`FaceView::surface`] is right here
    /// and answers for every face — but "rather than bad input" was not true and
    /// is gone (§39).
    #[must_use]
    pub fn plane(self) -> Plane {
        *self.surface().as_plane().expect(
            "this face is curved, and whichever operation asked for its plane \
             did not check is_wholly_flat() first",
        )
    }

    /// The direction pointing out of the material at this face.
    ///
    /// This is the face's plane normal turned round when the face is used
    /// reversed, which is the whole reason orientation lives on the reference
    /// and not on the face (roadmap §5.2).
    ///
    /// # Panics
    ///
    /// If the face does not sit on a plane.
    #[must_use]
    pub fn outward_normal(self) -> Vector {
        self.plane().normal() * self.orient.sign()
    }

    /// How much slop a decision put into this face, if any.
    #[must_use]
    pub fn approximation(self) -> Option<Approximation> {
        self.data().approximation
    }

    /// How many wires trim this face: one outer wire plus one per hole.
    #[must_use]
    pub fn wire_count(self) -> usize {
        self.data().wires.len()
    }

    /// The face's wires. The first is the outer one; the rest are holes.
    pub fn wires(self) -> impl Iterator<Item = WireView<'m>> {
        let (model, loc, orient) = (self.model, self.loc, self.orient);
        self.data()
            .wires
            .iter()
            .map(move |&wire| WireView::new(model, wire, loc, orient))
    }

    /// The outer wire.
    ///
    /// # Panics
    ///
    /// If the face has no wires at all, which is a bug in this crate rather
    /// than something a caller can hand in.
    ///
    /// **That was not true until the shape check went on the door.** An empty
    /// wire list holds no bad keys, so a file carrying one used to read back
    /// cleanly and panic here, and nothing forces a caller through
    /// [`Model::validate`](crate::Model::validate) first. Now
    /// `Model::check_shape_is_possible` refuses those bytes —
    /// `a_file_whose_shape_is_impossible_never_becomes_a_model` is the
    /// demonstration — and the only other routes into the arenas are
    /// `Model::from_parts`, which is `pub(crate)` and only reached through that
    /// same check, `data_mut`, which is `#[cfg(test)]`, and the builders, which
    /// end in `Model::check_touched_solid` and refuse a face with no wires
    /// themselves.
    ///
    /// So handing back an `Option` would say a face with no wires is a state
    /// worth carrying, at every call site, for ever. It is not one.
    #[must_use]
    pub fn outer_wire(self) -> WireView<'m> {
        self.wires().next().expect("a face has at least one wire")
    }

    /// Break the face into triangles and hand each one to `f`, corners in
    /// outward order.
    ///
    /// Every wire is fanned from one apex — the first corner of the outer wire.
    /// The triangles overlap for a face that is not convex, and the ones over a
    /// hole come back wound the other way. That is on purpose: the signed sums
    /// that measurement and point classification are built on cancel exactly,
    /// so no ear clipping is needed and there is nothing to get wrong when a
    /// wire doubles back.
    ///
    /// Put together, the triangles of every face of a closed solid form a
    /// closed surface: the spokes of each fan cancel in pairs and what is left
    /// is the wires themselves, which are shared with the neighbouring faces
    /// the other way round.
    ///
    /// The order is fixed, so two runs produce the same triangles in the same
    /// order and any float sum over them comes out bit for bit the same.
    pub fn for_each_triangle(self, mut f: impl FnMut([Point; 3])) {
        let Some(apex) = self.outer_wire().points().next() else {
            return;
        };
        for (position, wire) in self.wires().enumerate() {
            if position == 0 {
                // The apex is this wire's own first corner, so the two
                // triangles that would touch it twice have no area. Start one
                // corner in and stop before coming back round.
                let mut corner = wire.points();
                corner.next();
                let mut previous = corner.next();
                for point in corner {
                    if let Some(start) = previous {
                        f([apex, start, point]);
                    }
                    previous = Some(point);
                }
            } else {
                let mut first = None;
                let mut previous: Option<Point> = None;
                for point in wire.points() {
                    if let Some(start) = previous {
                        f([apex, start, point]);
                    } else {
                        first = Some(point);
                    }
                    previous = Some(point);
                }
                if let (Some(start), Some(end)) = (previous, first) {
                    f([apex, start, end]);
                }
            }
        }
    }

    fn data(self) -> &'m FaceData {
        &self.model.data().faces[self.slot]
    }
}

/// A wire, with the location and orientation it is being looked at through.
#[derive(Clone, Copy, Debug)]
pub struct WireView<'m> {
    model: &'m Model,
    slot: WireSlot,
    loc: Isometry,
    orient: Orientation,
}

impl<'m> WireView<'m> {
    pub(crate) fn new(
        model: &'m Model,
        slot: WireSlot,
        loc: Isometry,
        orient: Orientation,
    ) -> Self {
        Self {
            model,
            slot,
            loc,
            orient,
        }
    }

    /// A key naming this wire.
    #[must_use]
    pub fn key(self) -> WireKey {
        WireKey::new(self.slot, self.model.id())
    }

    /// Where this wire has been placed.
    #[must_use]
    pub fn location(self) -> Isometry {
        self.loc
    }

    /// Which way round this wire is being used.
    #[must_use]
    pub fn orientation(self) -> Orientation {
        self.orient
    }

    /// How many coedges the chain has, not counting poles.
    #[must_use]
    pub fn coedge_count(self) -> usize {
        self.data().coedges().count()
    }

    /// How many steps the chain has, counting poles.
    ///
    /// Different from [`WireView::coedge_count`] only for a wire round a
    /// surface that pinches to a point — a ball's one face has four steps and
    /// two coedges, because two of the four go nowhere.
    #[must_use]
    pub fn step_count(self) -> usize {
        self.data().steps.len()
    }

    /// Whether this is one coedge on an edge that already comes back to where
    /// it started — a round cap's boundary, which is one whole circle.
    ///
    /// Such a wire has one corner and no polygon at all, so anything that reads
    /// its corners as a ring of straight sides has to ask this first.
    #[must_use]
    pub fn closes_on_one_edge(self) -> bool {
        self.model.wire_closes_on_one_edge(self.data())
    }

    /// The chain of coedges, in the order this wire is being used.
    ///
    /// A wire reached through a face that is used reversed comes out walked the
    /// other way round, so following it always keeps the material on the same
    /// side. That is the whole point of orientation living on the reference
    /// rather than the node (roadmap §5.2).
    /// Poles are skipped, because a pole is not a use of an edge. Use
    /// [`WireView::step_count`] when the difference matters.
    pub fn coedges(self) -> impl Iterator<Item = CoedgeView<'m>> {
        let (model, loc, orient) = (self.model, self.loc, self.orient);
        let coedges: SmallVec<[CoedgeSlot; 4]> = self.data().coedges().collect();
        let count = coedges.len();
        (0..count).map(move |step| {
            let position = if orient.is_reversed() {
                count - 1 - step
            } else {
                step
            };
            CoedgeView::new(model, coedges[position], loc, orient)
        })
    }

    /// Every step round the chain, in the order this wire is being used, poles
    /// and all.
    ///
    /// [`WireView::coedges`] is the same walk with the poles left out, and that
    /// is what almost everything wants. This is for the callers that have to
    /// see where the boundary stands still: a mesher, which needs the pole to
    /// know that the side of the patch it is standing on is a whole turn wide
    /// (`docs/decisions.md` §25.6), and anything taking a solid apart to
    /// describe it again through a [`SolidBuilder`](crate::SolidBuilder) — a
    /// ball's one wire is four steps and two coedges, and reading it as two
    /// describes a shape that is not there.
    ///
    /// Nothing un-located comes out: both arms carry a view, and a view's
    /// geometry has already been moved into place.
    pub fn steps(self) -> impl Iterator<Item = WireStepView<'m>> {
        let (model, loc, orient) = (self.model, self.loc, self.orient);
        let steps: SmallVec<[WireStep; 4]> = self.data().steps.clone();
        let count = steps.len();
        (0..count).map(move |step| {
            let position = if orient.is_reversed() {
                count - 1 - step
            } else {
                step
            };
            match steps[position] {
                WireStep::Along(coedge) => {
                    WireStepView::Along(CoedgeView::new(model, coedge, loc, orient))
                }
                WireStep::Pole(vertex) => WireStepView::Pole(VertexView::new(model, vertex, loc)),
            }
        })
    }

    /// The corner points, in the stored order, already located.
    ///
    /// One point per coedge: the point that coedge starts at. Following them
    /// round and back to the first one traces the wire.
    ///
    /// A pole adds no point of its own, because it stands still on the corner
    /// the coedge either side of it already named.
    pub fn points(self) -> impl Iterator<Item = Point> {
        self.coedges().map(|coedge| coedge.start().point())
    }

    /// Which face this wire trims, if any.
    ///
    /// A key rather than a view: this looks *up* the graph, and the location on
    /// the way down to the wire is not necessarily the one the face is used at.
    #[must_use]
    pub fn face(self) -> Option<FaceKey> {
        self.model
            .index()
            .face_of_wire(self.slot)
            .map(|face| FaceKey::new(face, self.model.id()))
    }

    fn data(self) -> &'m WireData {
        &self.model.data().wires[self.slot]
    }
}

/// One step round a wire, as that wire is being used.
///
/// A wire is almost always a chain of coedges, and was only that until a ball
/// arrived. What a ball adds is the pole: the surface pinches to a point, the
/// boundary has to go somewhere between arriving there and leaving, and where
/// it goes is nowhere at all.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum WireStepView<'m> {
    /// Along one use of an edge.
    Along(CoedgeView<'m>),
    /// Standing still at a pole.
    Pole(VertexView<'m>),
}

/// A coedge, with the location and orientation it is being looked at through.
#[derive(Clone, Copy, Debug)]
pub struct CoedgeView<'m> {
    model: &'m Model,
    slot: CoedgeSlot,
    loc: Isometry,
    orient: Orientation,
}

impl<'m> CoedgeView<'m> {
    pub(crate) fn new(
        model: &'m Model,
        slot: CoedgeSlot,
        loc: Isometry,
        orient: Orientation,
    ) -> Self {
        Self {
            model,
            slot,
            loc,
            orient,
        }
    }

    /// A key naming this coedge.
    #[must_use]
    pub fn key(self) -> CoedgeKey {
        CoedgeKey::new(self.slot, self.model.id())
    }

    /// The edge this coedge uses, turned the way this coedge uses it.
    #[must_use]
    pub fn edge(self) -> EdgeView<'m> {
        let data = self.data();
        EdgeView::new(
            self.model,
            data.edge,
            self.loc,
            data.orientation.composed_with(self.orient),
        )
    }

    /// The vertex this coedge starts at.
    #[must_use]
    pub fn start(self) -> VertexView<'m> {
        self.edge().start()
    }

    /// The vertex this coedge ends at.
    #[must_use]
    pub fn end(self) -> VertexView<'m> {
        self.edge().end()
    }

    /// Which way round this coedge uses its edge, before any outer orientation
    /// is composed in.
    #[must_use]
    pub fn orientation_on_its_edge(self) -> Orientation {
        self.data().orientation
    }

    /// Which wire this coedge belongs to, if any.
    ///
    /// A key rather than a view, for the same reason as [`WireView::face`].
    #[must_use]
    pub fn wire(self) -> Option<WireKey> {
        self.model
            .index()
            .wire_of_coedge(self.slot)
            .map(|wire| WireKey::new(wire, self.model.id()))
    }

    fn data(self) -> &'m CoedgeData {
        &self.model.data().coedges[self.slot]
    }
}

/// An edge, with the location and orientation it is being looked at through.
#[derive(Clone, Copy, Debug)]
pub struct EdgeView<'m> {
    model: &'m Model,
    slot: EdgeSlot,
    loc: Isometry,
    orient: Orientation,
}

impl<'m> EdgeView<'m> {
    pub(crate) fn new(
        model: &'m Model,
        slot: EdgeSlot,
        loc: Isometry,
        orient: Orientation,
    ) -> Self {
        Self {
            model,
            slot,
            loc,
            orient,
        }
    }

    /// A key naming this edge.
    #[must_use]
    pub fn key(self) -> EdgeKey {
        EdgeKey::new(self.slot, self.model.id())
    }

    /// Which way round this edge is being used.
    #[must_use]
    pub fn orientation(self) -> Orientation {
        self.orient
    }

    /// The vertex this edge runs from, the way it is being used.
    #[must_use]
    pub fn start(self) -> VertexView<'m> {
        let data = self.data();
        let vertex = if self.orient.is_reversed() {
            data.end
        } else {
            data.start
        };
        VertexView::new(self.model, vertex, self.loc)
    }

    /// The vertex this edge runs to, the way it is being used.
    #[must_use]
    pub fn end(self) -> VertexView<'m> {
        let data = self.data();
        let vertex = if self.orient.is_reversed() {
            data.start
        } else {
            data.end
        };
        VertexView::new(self.model, vertex, self.loc)
    }

    /// Whether this edge comes back to the corner it left, because it runs
    /// right round a closed curve.
    ///
    /// A round cap's boundary is one of these. Anything that reads an edge as a
    /// straight side between two different corners has to ask this first.
    ///
    /// **This is the one list of run kinds that close, and every site that
    /// needs to know reads it here rather than asking on its own.** The
    /// mesher's direction rule, each of the fuse's own checks, the winding
    /// count on a round wire, [`EdgeView::segment`] below, `check_dynamic_range`
    /// in [`Model::validate`](crate::Model::validate) and `rcad-step`'s writer
    /// all ask this and nothing else. Naming the kinds again at every one of
    /// them would be one copy of the rule per site, which is the drift
    /// [`the_ends_are_the_whole_of_it`](crate::SolidBuilder) was pulled back
    /// into one list to stop (§44.11).
    ///
    /// **Two kinds close, and the second one arrived here rather than at any
    /// of those sites.** [`EdgeRun::RightRoundWhereItCrosses`] is a closed
    /// curve that carries the surface making the cut, for a rim whose two
    /// faces are the one surface — a frustum's wall split in two by a leaning
    /// plane. It is closed for exactly the reason the first one is, so it is
    /// one word on this line and nothing anywhere else, which is the whole of
    /// what pulling the rule together bought. `docs/decisions.md` §216.
    ///
    /// # Why this refuses rather than answering `false`
    ///
    /// It used to be `run == EdgeRun::RightRoundWhereItsFacesCross`, a bare `==`
    /// with no wildcard for `just check-wildcards` to count and nothing for
    /// clippy to warn about, so a fourth kind of [`EdgeRun`] — the enum is
    /// `#[non_exhaustive]` and lives in `rcad-geom`, and this milestone did grow
    /// its vocabulary — arrived at every site reading this as *not closed*, with
    /// zero compile errors. `tests/a_fourth_kind_of_run.rs` is what says a
    /// fourth kind arrived.
    ///
    /// A [`Result`] and not a three-way answer of its own. The kernel's
    /// three-way answer is
    /// [`Decision`](rcad_predicates::Decision), and its third value is a
    /// *measurement* that the arithmetic could not settle — a fact about the
    /// numbers, which a caller may reasonably carry on from. This third value is
    /// nothing about the edge at all: it is this crate not knowing a word, and
    /// there is no more arithmetic that would settle it. That is a refusal, and
    /// it refuses under [`Error::ACrossingThisCannotBeOrderedAlong`] — the name
    /// `check_closed_edges_are_walked_the_way_their_surfaces_wind` and the fuse
    /// already give a run kind they cannot follow, so a new kind of run refuses
    /// under one name across the kernel rather than under a fresh one per site.
    ///
    /// **What each site then does is not the same**, and each says so where
    /// it asks: most refuse under the shared name, and two rename it to their
    /// own — `rcad-step`'s writer, to `CannotWrite::ARunKindThisWriterHasNoCurveFor`,
    /// and the winding count, because its refusal points at the face.
    ///
    /// # Errors
    ///
    /// [`Error::ACrossingThisCannotBeOrderedAlong`] for a kind of run this crate
    /// has never seen.
    pub fn is_closed(self) -> Result<bool> {
        match self.data().run {
            EdgeRun::RightRoundWhereItsFacesCross | EdgeRun::RightRoundWhereItCrosses { .. } => {
                Ok(true)
            }
            EdgeRun::Straight | EdgeRun::PartOfWhereItCrosses { .. } => Ok(false),
            _ => Err(Error::ACrossingThisCannotBeOrderedAlong),
        }
    }

    /// The straight line this edge follows, already located and running the way
    /// the edge is being used.
    ///
    /// `None` for a closed edge: its two ends are one corner by definition, so
    /// there is no two-different-point segment to hand back. [`EdgeView::is_closed`]
    /// says so up front, or read this `None`.
    ///
    /// # Errors
    ///
    /// Whatever [`Segment::new`] refuses — a coordinate that is not a finite
    /// number, or two ends at the same point on an edge that is not closed.
    /// The doc this replaced said `Model::validate` rules the second one out,
    /// which is true only of `EdgeRun::Straight`: `check_edges` explicitly
    /// only refuses same-point ends when the run is straight, precisely
    /// because a closed edge is supposed to have them. `Model` derives
    /// `Deserialize`, and this is called on models `validate` never saw, so a
    /// `Straight` edge with two equal ends is a real caller-input case and
    /// not a broken invariant.
    ///
    /// **A kind of run this crate cannot read gets neither answer.** `None` says
    /// the two ends are one corner and a segment says they are the whole of the
    /// edge; a kind nothing here knows has not earned either, and handing back a
    /// chord for one would put a straight line where a curve is. So
    /// [`EdgeView::is_closed`]'s refusal travels out of here unchanged.
    pub fn segment(self) -> Result<Option<Segment>> {
        if self.is_closed()? {
            return Ok(None);
        }
        Ok(Some(Segment::new(
            self.start().point(),
            self.end().point(),
        )?))
    }

    /// The straight-line distance between the edge's two ends.
    ///
    /// **Not the length of the edge**, and the difference is the whole reason
    /// this is not called that. For a straight edge the two are the same
    /// number. For a closed edge the two ends are one corner, so this is
    /// exactly zero while the edge itself is a whole circle — and a caller that
    /// read the zero as a length would conclude the circle was not there.
    /// [`EdgeView::is_closed`] is how to tell.
    ///
    /// Zero here therefore means the two ends are the *same corner*, and never
    /// that they were too close together to tell apart — see [`how_far_apart`].
    #[must_use]
    pub fn distance_between_its_ends(self) -> f64 {
        how_far_apart(self.start().point(), self.end().point())
    }

    /// What this edge runs along between its two ends, already moved into
    /// place.
    ///
    /// By value, because a located run is built here rather than stored
    /// anywhere — the same shape as [`FaceView::surface`] and for the same
    /// reason. A stored run carries a [`Surface`] and
    /// [`Picker`](rcad_geom::Picker)s written in the frame the face is stored
    /// in, and this view has a location composed onto it, so handing the stored
    /// one out is what CLAUDE.md's rule about views forbids.
    ///
    /// This is what anything taking a solid apart to describe it again reads:
    /// an importer or a writer has to hand an [`EdgeRun`] back, and for an arc
    /// it cannot work one out from [`EdgeView::is_closed`] alone. Working in
    /// the frame the face is stored in instead wants
    /// [`EdgeView::stored_run`].
    ///
    /// **What this cannot promise is the exact squareness of a picker on a
    /// piece that runs on for ever.** A movement with no turn keeps every
    /// picking normal bit for bit, so a placed-but-unturned solid's runs are
    /// the stored ones shifted; a turn takes that squareness away, and
    /// [`EdgeRun::transformed`](rcad_geom::EdgeRun::transformed) names the
    /// three families of fix that were weighed and which one is left open.
    #[must_use]
    pub fn run(self) -> EdgeRun {
        self.data().run.transformed(&self.loc)
    }

    /// This edge's run in the frame its face is stored in, with this view's
    /// location left off.
    ///
    /// Not public: it is the raw arena payload, and handing that out is exactly
    /// what [`EdgeView::run`] exists to stop. Two kinds of caller in here want
    /// it anyway. The mesher shifts a face's run into the frame it meshes that
    /// face in, and §41.2's shift is worked out for a face at no placement at
    /// all, so a located run would land somewhere else. And every comparison
    /// against [`EdgeRun::Straight`] or
    /// [`EdgeRun::RightRoundWhereItsFacesCross`] takes either, since neither
    /// carries geometry — reading this one saves building a run to throw away.
    pub(crate) fn stored_run(self) -> &'m EdgeRun {
        &self.data().run
    }

    /// This edge's two ends in the frame its face is stored in, in the order
    /// the edge is stored, with this view's location left off.
    ///
    /// Not public, for [`EdgeView::stored_run`]'s reason. [`EdgeView::length`]
    /// wants it because a movement changes no length, so the stored numbers
    /// give a placed edge the same answer bit for bit as one at the origin —
    /// where the located ends would carry the placement's rounding into it.
    pub(crate) fn stored_ends(self) -> [Point; 2] {
        let data = self.data();
        let vertices = &self.model.data().vertices;
        [vertices[data.start].point, vertices[data.end].point]
    }

    /// The surfaces of the faces either side of this edge, in the frame they are
    /// stored in.
    ///
    /// A seam has one face on both sides and it comes back twice, which is what
    /// tells [`rcad_geom::which_two_surfaces_a_run_is_between`] to read the
    /// second surface off the run; an edge only one face uses is read the same
    /// way.
    ///
    /// # Errors
    ///
    /// [`Error::AnEdgeWithoutTwoFacesAlongItCannotBeWalked`] when no face or
    /// three or more run along it, so no pair of them says where it runs. The
    /// mesher's refusal and for its reason: [`Model`] is `Deserialize`, and a
    /// model that never went through [`Model::validate`] reaches here first.
    pub(crate) fn stored_surfaces_either_side(self) -> Result<[Surface; 2]> {
        let mut faces = self.model.faces_along_edge(self.slot);
        let sides = match (faces.next(), faces.next(), faces.next()) {
            (Some(only), None, None) => [only, only],
            (Some(one), Some(other), None) => [one, other],
            _ => {
                return Err(Error::AnEdgeWithoutTwoFacesAlongItCannotBeWalked { edge: self.key() });
            }
        };
        Ok(sides.map(|face| self.model.face_at(face).stored_surface()))
    }

    /// How much slop a decision put into this edge, if any.
    #[must_use]
    pub fn approximation(self) -> Option<Approximation> {
        self.data().approximation
    }

    /// The coedges that use this edge. A closed solid has exactly two.
    pub fn coedges(self) -> impl Iterator<Item = CoedgeKey> + 'm {
        let model = self.model;
        model
            .index()
            .coedges_of_edge(self.slot)
            .iter()
            .map(move |&coedge| CoedgeKey::new(coedge, model.id()))
    }

    /// The faces that meet along this edge.
    ///
    /// Keys rather than views: this looks *up* the graph, and a face may be
    /// used somewhere other than where this edge was reached from.
    pub fn neighbouring_faces(self) -> impl Iterator<Item = FaceKey> + 'm {
        let model = self.model;
        model
            .faces_along_edge(self.slot)
            .map(move |face| FaceKey::new(face, model.id()))
    }

    fn data(self) -> &'m EdgeData {
        &self.model.data().edges[self.slot]
    }
}

/// How far apart two points are, with no square in the middle of it.
///
/// [`Point::distance_to`](rcad_geom::Point::distance_to) adds up the squares of
/// the three differences and takes the root, and a difference below about
/// `1.5e-162` squares to zero: the root of zero is zero, so a real separation
/// comes back as no separation at all. That is a **gap** and not a limit — the
/// answer is an ordinary `f64` and the arithmetic threw it away — and the two
/// callers here are the two that have to tell a very short edge from no edge:
/// [`EdgeView::distance_between_its_ends`], which is what
/// `Model::check_dynamic_range` measures the smallest feature of a model with,
/// and the fuse's own check on the answer it is about to write.
///
/// `f64::hypot` is the same distance computed without the intermediate square,
/// so the whole subnormal range comes back as itself.
pub(crate) fn how_far_apart(one: Point, other: Point) -> f64 {
    let apart = other - one;
    apart.x().hypot(apart.y()).hypot(apart.z())
}

/// A vertex, with the location it is being looked at through.
#[derive(Clone, Copy, Debug)]
pub struct VertexView<'m> {
    model: &'m Model,
    slot: VertexSlot,
    loc: Isometry,
}

impl<'m> VertexView<'m> {
    pub(crate) fn new(model: &'m Model, slot: VertexSlot, loc: Isometry) -> Self {
        Self { model, slot, loc }
    }

    /// A key naming this vertex.
    #[must_use]
    pub fn key(self) -> VertexKey {
        VertexKey::new(self.slot, self.model.id())
    }

    /// Where the vertex is, already moved into place.
    #[must_use]
    pub fn point(self) -> Point {
        self.loc.apply_to_point(self.data().point)
    }

    /// Where this vertex has been placed.
    #[must_use]
    pub fn location(self) -> Isometry {
        self.loc
    }

    /// Where the vertex is in the frame it is stored in, with this view's
    /// location left off.
    ///
    /// Not public, for [`EdgeView::stored_run`]'s reason. The mesher wants it
    /// because it works a placed face out in the frame that face is stored in,
    /// and [`VertexView::point`] moved back there by the placement's inverse is
    /// this point rounded twice.
    pub(crate) fn stored_point(self) -> Point {
        self.data().point
    }

    /// How much slop a decision put into this vertex, if any.
    #[must_use]
    pub fn approximation(self) -> Option<Approximation> {
        self.data().approximation
    }

    /// The edges that meet at this vertex.
    ///
    /// Keys rather than views, for the same reason as
    /// [`EdgeView::neighbouring_faces`].
    pub fn edges(self) -> impl Iterator<Item = EdgeKey> + 'm {
        let model = self.model;
        model
            .index()
            .edges_of_vertex(self.slot)
            .iter()
            .map(move |&edge| EdgeKey::new(edge, model.id()))
    }

    fn data(self) -> &'m VertexData {
        &self.model.data().vertices[self.slot]
    }
}
