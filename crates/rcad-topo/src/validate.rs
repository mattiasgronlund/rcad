use rcad_geom::{DynamicRange, Isometry, Orientation, Point, Vector};
use rcad_predicates::{Approximation, Policy, ProvenanceLog};
use slotmap::SecondaryMap;

use crate::key::{CoedgeSlot, FaceSlot, ShellSlot, WireSlot};
use crate::view::{FaceView, SolidView, WireView};
use crate::{EdgeView, Invalid, Model, Result};

impl Model {
    /// Check every invariant in the model, from top to bottom.
    ///
    /// This is the full sweep, and it costs a pass over everything. It belongs
    /// at test boundaries and wherever untrusted data comes in — not after
    /// every operation, which would make a test that performs *n* operations
    /// cost O(n²) (decisions.md §4.3). The cheap per-entity checks that do run
    /// after every mutation are the ones that panic; these return an error,
    /// because a file arriving broken is data, not a bug.
    ///
    /// What it checks:
    ///
    /// - every stored key names something that is still there,
    /// - no coordinate is infinite and no edge has zero length,
    /// - every edge is used by exactly two coedges running opposite ways,
    /// - every wire has at least three coedges and closes back on itself,
    /// - every face sits on a plane, with its corners on that plane, its outer
    ///   wire winding with the face normal and its holes winding against it,
    ///   so the material is on one consistent side,
    /// - nothing is orphaned and nothing is used twice where it may only be
    ///   used once,
    /// - every solid encloses a positive amount of space, which is what says
    ///   its faces point outwards,
    /// - `tol(vertex) >= tol(edge) >= tol(face)` over the entities carrying an
    ///   [`Approximation`] (roadmap §5.4, decisions.md §1.2),
    /// - the coordinate range is narrow enough to compute in, as loudly as
    ///   `policy` asks for (decisions.md §3.5).
    ///
    /// A sweep that certifies everything leaves `log` untouched. The one check
    /// here that can come back non-certified is the coordinate range under
    /// [`Severity::Warn`](rcad_predicates::Severity::Warn), and that is the
    /// only thing that ever lands in `log`. An empty log means the model was
    /// checked with nothing approximated (decisions.md §5).
    ///
    /// # Errors
    ///
    /// [`Error::Invalid`](crate::Error::Invalid) naming the first invariant
    /// that does not hold, or [`Error::Geometry`](crate::Error::Geometry) if
    /// the coordinate range could not even be measured.
    pub fn validate(&self, policy: &Policy, log: &mut ProvenanceLog) -> Result<()> {
        self.check_vertices()?;
        self.check_edges()?;
        self.check_coedges()?;
        self.check_wires()?;
        self.check_faces()?;
        self.check_shells_and_solids()?;
        self.check_approximation_ordering()?;
        self.check_dynamic_range(policy, log)
    }

    fn check_vertices(&self) -> Result<()> {
        for (slot, vertex) in &self.data().vertices {
            if !vertex.point.is_finite() {
                return Err(Invalid::VertexNotFinite.into());
            }
            if self.index().edges_of_vertex(slot).is_empty() {
                return Err(Invalid::Orphan { entity: "vertex" }.into());
            }
        }
        Ok(())
    }

    fn check_edges(&self) -> Result<()> {
        for (slot, edge) in &self.data().edges {
            let (Some(start), Some(end)) = (
                self.data().vertices.get(edge.start),
                self.data().vertices.get(edge.end),
            ) else {
                return Err(Invalid::DanglingKey { entity: "vertex" }.into());
            };
            if edge.start == edge.end || start.point == end.point {
                return Err(Invalid::EdgeHasNoLength.into());
            }

            let coedges = self.index().coedges_of_edge(slot);
            if coedges.len() != 2 {
                return Err(Invalid::EdgeCoedgeCount {
                    count: coedges.len(),
                }
                .into());
            }
            if self.data().coedges[coedges[0]].orientation
                == self.data().coedges[coedges[1]].orientation
            {
                return Err(Invalid::CoedgesRunTheSameWay.into());
            }
        }
        Ok(())
    }

    fn check_coedges(&self) -> Result<()> {
        let mut used: SecondaryMap<CoedgeSlot, usize> = SecondaryMap::new();
        for wire in self.data().wires.values() {
            for &coedge in &wire.coedges {
                if let Some(entry) = used.entry(coedge) {
                    *entry.or_insert(0) += 1;
                }
            }
        }
        for (slot, coedge) in &self.data().coedges {
            if !self.data().edges.contains_key(coedge.edge) {
                return Err(Invalid::DanglingKey { entity: "edge" }.into());
            }
            match used.get(slot).copied().unwrap_or(0) {
                0 => return Err(Invalid::Orphan { entity: "coedge" }.into()),
                1 => {}
                count => {
                    return Err(Invalid::UsedMoreThanOnce {
                        entity: "coedge",
                        count,
                    }
                    .into());
                }
            }
        }
        Ok(())
    }

    fn check_wires(&self) -> Result<()> {
        let mut used: SecondaryMap<WireSlot, usize> = SecondaryMap::new();
        for face in self.data().faces.values() {
            for &wire in &face.wires {
                if let Some(entry) = used.entry(wire) {
                    *entry.or_insert(0) += 1;
                }
            }
        }
        for (slot, wire) in &self.data().wires {
            if wire.coedges.len() < 3 {
                return Err(Invalid::WireTooShort {
                    count: wire.coedges.len(),
                }
                .into());
            }
            for &coedge in &wire.coedges {
                if !self.data().coedges.contains_key(coedge) {
                    return Err(Invalid::DanglingKey { entity: "coedge" }.into());
                }
            }

            let mut previous_end = None;
            for (position, &coedge) in wire.coedges.iter().enumerate() {
                let (start, end) = self.coedge_ends(coedge);
                if previous_end.is_some_and(|previous| previous != start) {
                    return Err(Invalid::WireDoesNotClose { position }.into());
                }
                previous_end = Some(end);
            }
            let first_start = self.coedge_ends(wire.coedges[0]).0;
            if previous_end != Some(first_start) {
                return Err(Invalid::WireDoesNotClose {
                    position: wire.coedges.len(),
                }
                .into());
            }

            match used.get(slot).copied().unwrap_or(0) {
                0 => return Err(Invalid::Orphan { entity: "wire" }.into()),
                1 => {}
                count => {
                    return Err(Invalid::UsedMoreThanOnce {
                        entity: "wire",
                        count,
                    }
                    .into());
                }
            }
        }
        Ok(())
    }

    fn check_faces(&self) -> Result<()> {
        let mut used: SecondaryMap<FaceSlot, usize> = SecondaryMap::new();
        for shell in self.data().shells.values() {
            for face in &shell.faces {
                if let Some(entry) = used.entry(face.face) {
                    *entry.or_insert(0) += 1;
                }
            }
        }
        let allowed_wobble = self.allowed_wobble();
        for (slot, face) in &self.data().faces {
            let Some(plane) = face.surface.as_plane() else {
                return Err(Invalid::FaceIsNotPlanar.into());
            };
            if face.wires.is_empty() {
                return Err(Invalid::FaceHasNoWires.into());
            }
            for &wire in &face.wires {
                if !self.data().wires.contains_key(wire) {
                    return Err(Invalid::DanglingKey { entity: "wire" }.into());
                }
            }

            let view = FaceView::new(self, slot, Isometry::identity(), Orientation::Forward);
            for wire in view.wires() {
                for point in wire.points() {
                    let distance = plane.signed_distance_to(point).abs();
                    if distance > allowed_wobble {
                        return Err(Invalid::VertexOffItsFacePlane { distance }.into());
                    }
                }
            }
            for (position, wire) in view.wires().enumerate() {
                let with_the_normal = vector_area(wire).dot(plane.normal());
                if position == 0 && with_the_normal <= 0.0 {
                    return Err(Invalid::OuterWireWindsAgainstTheNormal.into());
                }
                if position > 0 && with_the_normal >= 0.0 {
                    return Err(Invalid::HoleWindsWithTheOuterWire.into());
                }
            }

            match used.get(slot).copied().unwrap_or(0) {
                0 => return Err(Invalid::Orphan { entity: "face" }.into()),
                1 => {}
                count => {
                    return Err(Invalid::UsedMoreThanOnce {
                        entity: "face",
                        count,
                    }
                    .into());
                }
            }
        }
        Ok(())
    }

    fn check_shells_and_solids(&self) -> Result<()> {
        let mut used: SecondaryMap<ShellSlot, usize> = SecondaryMap::new();
        for solid in self.data().solids.values() {
            for shell in &solid.shells {
                if let Some(entry) = used.entry(shell.shell) {
                    *entry.or_insert(0) += 1;
                }
            }
        }
        for (slot, shell) in &self.data().shells {
            for face in &shell.faces {
                if !self.data().faces.contains_key(face.face) {
                    return Err(Invalid::DanglingKey { entity: "face" }.into());
                }
            }
            match used.get(slot).copied().unwrap_or(0) {
                0 => return Err(Invalid::Orphan { entity: "shell" }.into()),
                1 => {}
                count => {
                    return Err(Invalid::UsedMoreThanOnce {
                        entity: "shell",
                        count,
                    }
                    .into());
                }
            }
        }

        for (slot, solid) in &self.data().solids {
            for shell in &solid.shells {
                if !self.data().shells.contains_key(shell.shell) {
                    return Err(Invalid::DanglingKey { entity: "shell" }.into());
                }
            }
            let volume = SolidView::new(self, slot, Isometry::identity(), Orientation::Forward)
                .signed_volume();
            if !(volume.is_finite() && volume > 0.0) {
                return Err(Invalid::VolumeIsNotPositive { volume }.into());
            }
        }
        Ok(())
    }

    fn check_approximation_ordering(&self) -> Result<()> {
        for edge in self.data().edges.values() {
            let on_edge = magnitude_of(edge.approximation);
            for end in [edge.start, edge.end] {
                let on_vertex = magnitude_of(self.data().vertices[end].approximation);
                if on_vertex < on_edge {
                    return Err(Invalid::ApproximationOrdering {
                        outer: "edge",
                        outer_magnitude: on_edge,
                        inner: "vertex",
                        inner_magnitude: on_vertex,
                    }
                    .into());
                }
            }
        }
        for face in self.data().faces.values() {
            let on_face = magnitude_of(face.approximation);
            for &wire in &face.wires {
                for &coedge in &self.data().wires[wire].coedges {
                    let edge = self.data().coedges[coedge].edge;
                    let on_edge = magnitude_of(self.data().edges[edge].approximation);
                    if on_edge < on_face {
                        return Err(Invalid::ApproximationOrdering {
                            outer: "face",
                            outer_magnitude: on_face,
                            inner: "edge",
                            inner_magnitude: on_edge,
                        }
                        .into());
                    }
                }
            }
        }
        Ok(())
    }

    fn check_dynamic_range(&self, policy: &Policy, log: &mut ProvenanceLog) -> Result<()> {
        let largest = self
            .data()
            .vertices
            .values()
            .map(|vertex| vertex.point.largest_coordinate_magnitude())
            .fold(0.0, f64::max);
        let smallest_feature = self
            .edges()
            .map(EdgeView::length)
            .fold(f64::INFINITY, f64::min);
        if !smallest_feature.is_finite() || largest == 0.0 {
            return Ok(());
        }

        let range = DynamicRange::measure(largest, smallest_feature)?;
        match range.check_under_policy(policy, log) {
            // The tag says how much slop a range this wide puts into anything
            // built from the model. A sweep builds nothing, so there is no
            // entity to hang it on; the record it pushed into `log` is the part
            // a caller can act on.
            Ok(_) => Ok(()),
            // Both layers already have a name for this. Keep reporting the
            // topology one, since the sweep is what found it.
            Err(rcad_geom::Error::DynamicRangeTooWide { ratio }) => {
                Err(Invalid::DynamicRangeTooWide { ratio }.into())
            }
            Err(other) => Err(other.into()),
        }
    }

    /// How far off its own plane a face's corner is allowed to sit.
    ///
    /// A face built here is exactly planar in the model's own frame, so this is
    /// only ever taking up the slack in fitting a plane through the corners.
    fn allowed_wobble(&self) -> f64 {
        let largest = self
            .data()
            .vertices
            .values()
            .map(|vertex| vertex.point.largest_coordinate_magnitude())
            .fold(0.0, f64::max);
        self.scale().default_linear_tolerance() + 64.0 * f64::EPSILON * largest
    }
}

/// Twice the area the wire sweeps out, as a vector along the wire's own normal.
///
/// Fanned from the wire's first corner, which telescopes: the extra triangles
/// on a wire that is not convex cancel exactly, so this is right whatever shape
/// the wire is.
fn vector_area(wire: WireView<'_>) -> Vector {
    let mut total = Vector::zero();
    let mut corner: Option<Point> = None;
    let mut previous: Option<Point> = None;
    for point in wire.points() {
        if let (Some(apex), Some(start)) = (corner, previous) {
            total = total + (start - apex).cross(point - apex);
        } else {
            corner = Some(point);
        }
        previous = Some(point);
    }
    total * 0.5
}

fn magnitude_of(tag: Option<Approximation>) -> f64 {
    tag.map_or(0.0, Approximation::magnitude)
}
