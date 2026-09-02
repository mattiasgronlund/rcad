use rcad_geom::{EdgeRun, Isometry, Orientation, Plane, Point, Segment, Surface, Vector};
use rcad_predicates::Approximation;
use smallvec::SmallVec;

use crate::key::{CoedgeSlot, EdgeSlot, FaceSlot, ShellSlot, SolidSlot, VertexSlot, WireSlot};
use crate::model::{
    CoedgeData, EdgeData, FaceData, ShellData, SolidData, VertexData, WireData, WireStep,
};
use crate::{CoedgeKey, EdgeKey, FaceKey, Model, Result, ShellKey, SolidKey, VertexKey, WireKey};

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

    /// An axis-aligned box the whole solid fits inside, or `None` when it has no
    /// faces at all.
    ///
    /// **This is the one [`SolidView::box_around_its_corners`] is not**, and the
    /// difference is what a ray caster needs: a point outside this box is
    /// outside the solid with nothing left over, which is a proof and not a
    /// shortcut, and a ray target picked from it is clear of every face.
    ///
    /// A flat face is the box round its own corners, exactly as before. The two
    /// curved ones bulge away from theirs and each is bounded by an argument
    /// rather than by a fudge factor:
    ///
    /// - A **ball**'s skin is inside the box round its centre a radius each way,
    ///   whatever the face is trimmed down to. Nothing tighter is available
    ///   without knowing where the trimming runs, and a ball's face may reach
    ///   right round.
    /// - A **pipe**'s skin is a radius from its axis, and how far *along* that
    ///   axis a face reaches is settled by its corners: the distance along the
    ///   axis grows steadily everywhere on a pipe, so it has no high or low spot
    ///   in the middle of a face and reaches its own furthest on the boundary.
    ///   So the box round the corners' shadows on the axis, opened out by the
    ///   radius, holds the face.
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
        let mut round_it = box_the_surface_bounds(face);
        if face.surface().as_plane().is_none() {
            return round_it;
        }
        for coedge in face.wires().flat_map(WireView::coedges) {
            let edge = coedge.edge();
            if *edge.run() == EdgeRun::Straight {
                continue;
            }
            let Some(across) = edge
                .neighbouring_faces()
                .find(|&other| other != face.key())
                .and_then(|other| self.faces().find(|used| used.key() == other))
            else {
                continue;
            };
            if let Some(theirs) = box_the_surface_bounds(across) {
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
/// - A **pipe**'s skin is a radius from its axis, and how far *along* that axis
///   a face reaches is settled by its corners: the distance along an axis grows
///   steadily everywhere on a pipe, so a face has no high or low spot in the
///   middle and reaches its own furthest on the boundary.
/// - A **plane** bounds nothing on its own, so this is the box round the
///   corners, which holds the face only while every side of it is straight.
///   [`SolidView::box_around_one_face`] is what opens that out.
fn box_the_surface_bounds(face: FaceView<'_>) -> Option<(Point, Point)> {
    let mut low = [f64::INFINITY; 3];
    let mut high = [f64::NEG_INFINITY; 3];
    let mut seen = false;
    let mut open_out = |at: [f64; 3], by: f64| {
        seen = true;
        for axis in 0..3 {
            low[axis] = low[axis].min(at[axis] - by);
            high[axis] = high[axis].max(at[axis] + by);
        }
    };
    match face.surface() {
        Surface::Plane(_) => {
            for corner in face.wires().flat_map(WireView::points) {
                open_out(corner.coordinates(), 0.0);
            }
        }
        Surface::Sphere(ball) => open_out(ball.centre().coordinates(), ball.radius()),
        Surface::Cylinder(pipe) => {
            let (origin, along) = (pipe.axis().origin(), pipe.axis().direction());
            for corner in face.wires().flat_map(WireView::points) {
                let shadow = origin + along * along.dot(corner - origin);
                open_out(shadow.coordinates(), pipe.radius());
            }
        }
        other => panic!("this crate cannot say how far {other:?} reaches"),
    }
    seen.then(|| (Point::from(low), Point::from(high)))
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
    /// If the face has no wires at all.
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
    #[must_use]
    pub fn is_closed(self) -> bool {
        self.data().run == EdgeRun::RightRoundWhereItsFacesCross
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
    pub fn segment(self) -> Result<Option<Segment>> {
        if self.is_closed() {
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
    #[must_use]
    pub fn distance_between_its_ends(self) -> f64 {
        self.start().point().distance_to(self.end().point())
    }

    /// What this edge runs along between its two ends.
    ///
    /// Not public. What an edge runs along is how the *model* names it, and
    /// there is no consumer outside this crate — a caller that wants the shape
    /// of a crossing asks the intersector, which hands back its own
    /// [`NamedRun`](rcad_geom::NamedRun).
    pub(crate) fn run(self) -> &'m EdgeRun {
        &self.data().run
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
