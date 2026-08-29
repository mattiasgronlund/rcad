use rcad_geom::{Isometry, Orientation, Plane, Point, Segment, Surface, Vector};
use rcad_predicates::Approximation;

use crate::key::{CoedgeSlot, EdgeSlot, FaceSlot, ShellSlot, SolidSlot, VertexSlot, WireSlot};
use crate::model::{CoedgeData, EdgeData, FaceData, ShellData, SolidData, VertexData, WireData};
use crate::{CoedgeKey, EdgeKey, FaceKey, Model, ShellKey, SolidKey, VertexKey, WireKey};

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

    /// The smallest axis-aligned box holding every vertex of this solid.
    ///
    /// `None` when the solid has no faces at all.
    #[must_use]
    pub fn bounding_box(self) -> Option<(Point, Point)> {
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

    fn data(self) -> &'m SolidData {
        &self.model.data().solids[self.slot]
    }
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
    /// If the face sits on something other than a plane. M0 builds planar
    /// solids only (decisions.md §3.1) and [`Model::validate`] rejects anything
    /// else, so reaching this means the model was already broken.
    #[must_use]
    pub fn plane(self) -> Plane {
        *self
            .surface()
            .as_plane()
            .expect("M0 faces sit on planes; validate() refuses anything else")
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

    /// How many coedges the chain has.
    #[must_use]
    pub fn coedge_count(self) -> usize {
        self.data().coedges.len()
    }

    /// The chain of coedges, in the order this wire is being used.
    ///
    /// A wire reached through a face that is used reversed comes out walked the
    /// other way round, so following it always keeps the material on the same
    /// side. That is the whole point of orientation living on the reference
    /// rather than the node (roadmap §5.2).
    pub fn coedges(self) -> impl Iterator<Item = CoedgeView<'m>> {
        let (model, loc, orient) = (self.model, self.loc, self.orient);
        let coedges = &self.data().coedges;
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

    /// The corner points, in the stored order, already located.
    ///
    /// One point per coedge: the point that coedge starts at. Following them
    /// round and back to the first one traces the wire.
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

    /// The straight line this edge follows, already located and running the way
    /// the edge is being used.
    ///
    /// # Panics
    ///
    /// If the two ends are at the same point. [`Model::validate`] rejects that,
    /// so reaching it means the model was already broken.
    #[must_use]
    pub fn segment(self) -> Segment {
        Segment::new(self.start().point(), self.end().point())
            .expect("an edge of a valid model has two different ends")
    }

    /// How long the edge is.
    #[must_use]
    pub fn length(self) -> f64 {
        self.start().point().distance_to(self.end().point())
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
