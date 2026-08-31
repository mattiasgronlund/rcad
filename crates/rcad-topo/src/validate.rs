use rcad_geom::{DynamicRange, Isometry, Orientation, Plane, Vector};
use rcad_predicates::expansion::{
    expansion_sign, expansion_sum, product_is_exact, scale_expansion, two_product,
};
use rcad_predicates::{
    Approximation, Decision, Filtered, Interval, Level, Measures, NOTHING_IS_KNOWN,
    Point as PredicatePoint, Policy, Predicate, ProvenanceLog, Record, are_real_numbers, decide,
};
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
    /// that does not hold, [`Error::Geometry`](crate::Error::Geometry) if the
    /// coordinate range could not even be measured, or
    /// [`Error::Undecidable`](crate::Error::Undecidable) if a face's wire is
    /// wound so far past what `f64` can hold that the cascade cannot say which
    /// way round it goes.
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
                let winding = winding_of(wire, plane.normal())?;
                if position == 0 && winding != Decision::Pos {
                    return Err(Invalid::OuterWireWindsAgainstTheNormal.into());
                }
                if position > 0 && winding != Decision::Neg {
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
    /// Two separate things added together. The first is how far off the model
    /// itself is allowed to be — a face built here is exactly planar in the
    /// model's own frame, so this is only taking up the slack in fitting a
    /// plane through corners that arrived from somewhere else. The second is
    /// how far off our own *measurement* of it could be: what the plane says
    /// its own arithmetic costs, times the largest coordinate in the model,
    /// times [`MEASURING_SLACK`].
    fn allowed_wobble(&self) -> f64 {
        let largest = self
            .data()
            .vertices
            .values()
            .map(|vertex| vertex.point.largest_coordinate_magnitude())
            .fold(0.0, f64::max);
        self.scale().default_linear_tolerance()
            + MEASURING_SLACK * Plane::SIGNED_DISTANCE_ROUNDOFF * largest
    }
}

/// How much slack to allow on top of what reading the distance costs.
///
/// [`Plane::SIGNED_DISTANCE_ROUNDOFF`] is what one reading rounds away, and it
/// is derived where the arithmetic lives. This is the part that is not derived,
/// and it is here rather than there because it is a judgement about how much
/// room this particular check wants, not a fact about the expression
/// (decisions.md §15.6).
///
/// A corner is not only measured with rounding; the plane it is measured
/// against was fitted through corners that had already been rounded, and the
/// constructors scale a normal to unit length on the way in. Sixteen is wide
/// enough that ordinary construction never trips the check and narrow enough
/// that a corner genuinely off its plane still does.
const MEASURING_SLACK: f64 = 16.0;

/// Which way `wire` winds round `normal`.
///
/// [`Decision::Pos`] with it, [`Decision::Neg`] against it, [`Decision::Zero`]
/// when the wire sweeps out no area at all, which no sound face has.
///
/// The quantity decided is the wire's vector area dotted with the normal. That
/// sum cancels — a wire far from the origin adds and subtracts numbers far
/// larger than the area they are measuring — so reading the sign off an `f64`
/// total is a coin toss near the point where the sign turns over. A sweep that
/// got this wrong would pass a solid with its material on the wrong side, or
/// refuse a sound one, so it goes through the cascade like every other
/// geometric decision here (decisions.md §2.1).
///
/// # Errors
///
/// [`Error::Undecidable`] when the cascade cannot settle the sign. For the
/// exact `f64` coordinates a model stores that only happens when a coordinate
/// product overflows or falls into the subnormals.
///
/// This used to add "which is what the dynamic-range check exists to reject",
/// and that check does not reject it: it is a **ratio** test and is blind to
/// absolute magnitude, so a uniformly tiny model passes with a ratio of one.
/// This matters more here than elsewhere, because `Model` is `Deserialize` and
/// `validate` is the untrusted-import boundary — which is exactly where such
/// coordinates arrive. `docs/decisions.md` §16.11.
fn winding_of(wire: WireView<'_>, normal: Vector) -> Result<Decision> {
    let corners: Vec<[f64; 3]> = wire.points().map(|corner| corner.coordinates()).collect();
    winding_round(&corners, normal.components())
}

/// The same question asked of a ring of corners that is not in the model yet.
///
/// The fuse joins coplanar pieces into one face before it commits anything, and
/// then has to say which of the rings it ended up with is the outline and which
/// are holes. That is this decision, on corners the model has never seen.
///
/// A corner or a normal component that is not a real number is refused before
/// the cascade runs, and comes back as [`Error::Undecidable`] like any other
/// refusal. Without that, a corner whose height is not a number still reached a
/// certified winding: an axis-aligned normal has two zero components, and a
/// zero coefficient multiplies the unknown clean out of the sum.
///
/// [`Error::Undecidable`]: crate::Error::Undecidable
pub(crate) fn winding_round(corners: &[[f64; 3]], normal: [f64; 3]) -> Result<Decision> {
    let swept = AreaSweptRoundTheNormal {
        corners: corners.to_vec(),
        normal,
    };
    let every_number_is_real = are_real_numbers(corners.iter().flatten().copied().chain(normal));
    let (decision, level) = if every_number_is_real {
        decide(&swept)
    } else {
        (NOTHING_IS_KNOWN, Level::Interval)
    };
    match decision {
        Decision::Neg | Decision::Zero | Decision::Pos => Ok(decision),
        Decision::Undecidable { .. } => {
            let corners: Vec<PredicatePoint> = swept
                .corners
                .iter()
                .map(|&corner| PredicatePoint::from(corner))
                .collect();
            let mut records = ProvenanceLog::new();
            records.push(Record::new(
                SWEPT_AREA,
                &corners,
                level,
                decision,
                Measures::Area,
            ));
            Err(crate::Error::Undecidable { records })
        }
        // `Decision` grows a variant at every milestone. Whatever the new one
        // means, guessing is worse than stopping.
        other => panic!("the swept area came back with a decision nothing here knows: {other:?}"),
    }
}

/// The name this predicate goes by in provenance records.
const SWEPT_AREA: &str = "area swept by a wire round a normal";

/// Twice the area a closed wire sweeps out, dotted with a normal.
///
/// Summed as `Σ (p × q) · n` over the wire's edges rather than fanned from one
/// corner. Both telescope, so both are right whatever shape the wire is, but
/// this one is a sum of plain products of stored coordinates — which is what
/// lets level 3 work in exact arithmetic over the numbers the model actually
/// holds, instead of over differences it would have to round first.
struct AreaSweptRoundTheNormal {
    corners: Vec<[f64; 3]>,
    normal: [f64; 3],
}

/// Which two coordinates each component of a cross product is built from.
///
/// `(u × v)_x = u_y*v_z − u_z*v_y`, and the other two follow it round.
const CROSS_PRODUCT_AXES: [(usize, usize); 3] = [(1, 2), (2, 0), (0, 1)];

impl Predicate for AreaSweptRoundTheNormal {
    fn name(&self) -> &'static str {
        SWEPT_AREA
    }

    /// There is no `f64` filter for this one.
    ///
    /// A forward error bound has to be derived for a fixed expression, and this
    /// expression grows with the wire: the number of roundings depends on how
    /// many corners there are. Writing a bound that holds for every wire length
    /// means writing one that is loose for short wires and easy to get wrong,
    /// and a bound that is wrong certifies the wrong sign. Level 2 is rigorous
    /// by construction and settles everything a filter would have, so the
    /// filter buys speed this sweep does not need.
    fn on_floats(&self) -> Option<Filtered> {
        None
    }

    fn on_intervals(&self) -> Interval {
        let mut total = Interval::exactly(0.0);
        for (start, end) in self.edges() {
            for (axis, (first, second)) in CROSS_PRODUCT_AXES.into_iter().enumerate() {
                let minor = Interval::exactly(start[first]) * Interval::exactly(end[second])
                    - Interval::exactly(start[second]) * Interval::exactly(end[first]);
                total = total + Interval::exactly(self.normal[axis]) * minor;
            }
        }
        total
    }

    fn exactly(&self) -> Option<Decision> {
        let mut total: Vec<f64> = Vec::new();
        let mut merged: Vec<f64> = Vec::new();
        let mut scaled = [0.0f64; 8];

        for (start, end) in self.edges() {
            for (axis, (first, second)) in CROSS_PRODUCT_AXES.into_iter().enumerate() {
                let along_the_normal = self.normal[axis];
                let (minor, minor_length) =
                    exact_minor(start[first], end[second], start[second], end[first])?;
                let minor = &minor[..minor_length];
                if !minor
                    .iter()
                    .all(|&entry| product_is_exact(entry, along_the_normal))
                {
                    return None;
                }
                let scaled_length = scale_expansion(minor, along_the_normal, &mut scaled);
                merged.resize(total.len() + scaled_length, 0.0);
                let total_length = expansion_sum(&total, &scaled[..scaled_length], &mut merged);
                merged.truncate(total_length);
                core::mem::swap(&mut total, &mut merged);
            }
        }

        match expansion_sign(&total) {
            Decision::Neg => Some(Decision::Neg),
            Decision::Zero => Some(Decision::Zero),
            Decision::Pos => Some(Decision::Pos),
            // A product overflowed. Exact arithmetic that is not exact has
            // nothing to offer, so this level cannot run.
            Decision::Undecidable { .. } => None,
            other => {
                panic!("an expansion came back with a decision nothing here knows: {other:?}")
            }
        }
    }
}

impl AreaSweptRoundTheNormal {
    /// Every corner paired with the one after it, the last with the first.
    fn edges(&self) -> impl Iterator<Item = ([f64; 3], [f64; 3])> + '_ {
        let count = self.corners.len();
        (0..count).map(move |index| (self.corners[index], self.corners[(index + 1) % count]))
    }
}

/// `first * second - third * fourth`, exactly.
///
/// `None` when either product is small enough that its own roundoff term would
/// be subnormal, because then [`two_product`] cannot hand the term back whole
/// and the result stops being exact.
fn exact_minor(first: f64, second: f64, third: f64, fourth: f64) -> Option<([f64; 4], usize)> {
    if !product_is_exact(first, second) || !product_is_exact(third, fourth) {
        return None;
    }
    let (forwards, forwards_roundoff) = two_product(first, second);
    let (backwards, backwards_roundoff) = two_product(third, fourth);
    let mut out = [0.0; 4];
    let length = expansion_sum(
        &[forwards_roundoff, forwards],
        &[-backwards_roundoff, -backwards],
        &mut out,
    );
    Some((out, length))
}

fn magnitude_of(tag: Option<Approximation>) -> f64 {
    tag.map_or(0.0, Approximation::magnitude)
}

#[cfg(test)]
mod tests {
    use rcad_geom::{Isometry, Orientation, Plane, Point, Scale, Sphere, Surface, Vector};
    use rcad_predicates::{Approximation, Cause, Policy, ProvenanceLog};
    use smallvec::SmallVec;

    use super::{Decision, winding_of, winding_round};
    use crate::key::{CoedgeSlot, EdgeSlot, FaceSlot, VertexSlot, WireSlot};
    use crate::{Error, FaceView, Invalid, Model, WireView};

    fn an_empty_model() -> Model {
        Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
    }

    fn a_cube() -> Model {
        let mut model = an_empty_model();
        model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube of side two");
        model
    }

    fn a_plate_with_a_hole() -> Model {
        let mut model = an_empty_model();
        let plate =
            crate::ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
                .with_hole(vec![[3.0, 3.0], [6.0, 3.0], [6.0, 6.0], [3.0, 6.0]])
                .build(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("a plate with a hole");
        model
            .add_extrusion(&plate, 2.0, &Isometry::identity())
            .expect("a plate two thick");
        model
    }

    /// The plane `z = 0`, facing up.
    fn the_ground() -> Plane {
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal")
    }

    fn a_clean_sweep(model: &Model) {
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("this model is sound");
    }

    /// The one invariant the sweep refused this model for.
    fn refusal(model: &Model) -> Invalid {
        match model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
            Err(Error::Invalid(broken)) => broken,
            other => panic!("the sweep should have refused this model, and gave {other:?}"),
        }
    }

    #[test]
    fn an_edge_naming_a_corner_that_is_gone_is_caught() {
        let mut model = a_cube();
        let corner = model
            .data()
            .vertices
            .keys()
            .next()
            .expect("a cube has corners");
        model.data_mut().vertices.remove(corner);

        assert_eq!(refusal(&model), Invalid::DanglingKey { entity: "vertex" });
    }

    #[test]
    fn a_corner_at_a_coordinate_that_is_not_a_number_is_caught() {
        let mut model = a_cube();
        model
            .data_mut()
            .vertices
            .values_mut()
            .next()
            .expect("a cube has corners")
            .point = Point::new(f64::NAN, 0.0, 0.0);

        assert_eq!(refusal(&model), Invalid::VertexNotFinite);
    }

    #[test]
    fn an_edge_that_starts_and_ends_at_the_same_corner_is_caught() {
        let mut model = a_cube();
        let edge = model.data().edges.keys().next().expect("a cube has edges");
        let start = model.data().edges[edge].start;
        model.data_mut().edges[edge].end = start;

        assert_eq!(refusal(&model), Invalid::EdgeHasNoLength);
    }

    #[test]
    fn an_edge_that_has_grown_a_third_coedge_is_caught() {
        let mut model = a_cube();
        let edge = model.data().edges.keys().next().expect("a cube has edges");
        model.add_coedge(edge, Orientation::Forward);

        assert_eq!(refusal(&model), Invalid::EdgeCoedgeCount { count: 3 });
    }

    #[test]
    fn both_coedges_of_an_edge_running_the_same_way_is_caught() {
        let mut model = a_cube();
        let edge = model.data().edges.keys().next().expect("a cube has edges");
        let coedge = model.index().coedges_of_edge(edge)[0];
        let turned = model.data().coedges[coedge].orientation.reversed();
        model.data_mut().coedges[coedge].orientation = turned;

        assert_eq!(refusal(&model), Invalid::CoedgesRunTheSameWay);
    }

    #[test]
    fn a_wire_whose_coedges_no_longer_join_up_is_caught() {
        let mut model = a_cube();
        let wire = model.data().wires.keys().next().expect("a cube has wires");
        model.data_mut().wires[wire].coedges.swap(0, 1);

        assert_eq!(refusal(&model), Invalid::WireDoesNotClose { position: 1 });
    }

    #[test]
    fn a_corner_no_edge_uses_is_caught() {
        let mut model = a_cube();
        model.add_vertex(Point::new(9.0, 9.0, 9.0));

        assert_eq!(refusal(&model), Invalid::Orphan { entity: "vertex" });
    }

    #[test]
    fn a_shell_that_lists_the_same_face_twice_is_caught() {
        let mut model = a_cube();
        let shell = model
            .data()
            .shells
            .keys()
            .next()
            .expect("a cube has a shell");
        let again = model.data().shells[shell].faces[0];
        model.data_mut().shells[shell].faces.push(again);

        assert_eq!(
            refusal(&model),
            Invalid::UsedMoreThanOnce {
                entity: "face",
                count: 2
            }
        );
    }

    #[test]
    fn a_face_that_is_not_on_a_plane_is_caught() {
        let mut model = a_cube();
        let ball = Sphere::new(Point::origin(), 1.0).expect("a ball of radius one");
        model
            .data_mut()
            .faces
            .values_mut()
            .next()
            .expect("a cube has faces")
            .surface = Surface::Sphere(ball);

        assert_eq!(refusal(&model), Invalid::FaceIsNotPlanar);
    }

    #[test]
    fn a_face_whose_outer_wire_winds_against_its_normal_is_caught() {
        let mut model = a_cube();
        let face = model
            .data_mut()
            .faces
            .values_mut()
            .next()
            .expect("a cube has faces");
        let plane = *face
            .surface
            .as_plane()
            .expect("a cube's faces sit on planes");
        face.surface = Surface::Plane(plane.reversed());

        assert_eq!(refusal(&model), Invalid::OuterWireWindsAgainstTheNormal);
    }

    #[test]
    fn a_solid_turned_inside_out_is_caught() {
        let mut model = a_cube();
        for shell in model.data_mut().shells.values_mut() {
            for used in &mut shell.faces {
                used.orient = used.orient.reversed();
            }
        }

        assert!(matches!(
            refusal(&model),
            Invalid::VolumeIsNotPositive { .. }
        ));
    }

    #[test]
    fn an_edge_carrying_more_slop_than_its_corners_is_caught() {
        let mut model = a_cube();
        model
            .data_mut()
            .edges
            .values_mut()
            .next()
            .expect("a cube has edges")
            .approximation = Some(Approximation::new(1e-6, Cause::ImportedUncertainty));

        assert!(matches!(
            refusal(&model),
            Invalid::ApproximationOrdering { .. }
        ));
    }

    /// Lift the corner the first face's wire starts from straight up.
    ///
    /// Straight up, because the walls of an extrusion stand on vertical planes:
    /// moving a corner along `z` takes it off the cap it sits on and leaves it
    /// exactly on every wall that meets there, so the cap is the only thing
    /// this can break.
    fn lift_the_first_corner(model: &mut Model, by: f64) {
        let corner = model
            .data_mut()
            .vertices
            .values_mut()
            .next()
            .expect("a cube has corners");
        corner.point = Point::new(corner.point.x(), corner.point.y(), corner.point.z() + by);
    }

    #[test]
    fn a_corner_within_the_allowed_wobble_of_its_face_plane_is_let_through() {
        let mut model = a_cube();
        let allowed = model.allowed_wobble();
        lift_the_first_corner(&mut model, allowed / 2.0);

        a_clean_sweep(&model);
    }

    #[test]
    fn a_corner_further_off_its_face_plane_than_that_is_caught() {
        let mut model = a_cube();
        let allowed = model.allowed_wobble();
        lift_the_first_corner(&mut model, allowed * 2.0);

        match refusal(&model) {
            Invalid::VertexOffItsFacePlane { distance } => {
                assert!(distance > allowed, "{distance} should be over {allowed}");
            }
            other => panic!("expected a corner off its plane, got {other:?}"),
        }
    }

    #[test]
    fn a_wire_of_only_two_coedges_is_caught() {
        let mut model = an_empty_model();
        let start = model.add_vertex(Point::new(0.0, 0.0, 0.0));
        let end = model.add_vertex(Point::new(1.0, 0.0, 0.0));
        let edge = model.add_edge(start, end);
        let there = model.add_coedge(edge, Orientation::Forward);
        let back = model.add_coedge(edge, Orientation::Reversed);
        let wire = model.add_wire([there, back].into_iter().collect());
        model.add_face(Surface::Plane(the_ground()), SmallVec::from_buf([wire]));

        assert_eq!(refusal(&model), Invalid::WireTooShort { count: 2 });
    }

    #[test]
    fn a_face_with_no_wires_at_all_is_caught() {
        let mut model = an_empty_model();
        model.add_face(Surface::Plane(the_ground()), SmallVec::new());

        assert_eq!(refusal(&model), Invalid::FaceHasNoWires);
    }

    /// A triangle in the ground plane and its mirror image, as two wires.
    ///
    /// The mirror runs the same edges the other way round, which is what gives
    /// every edge the two coedges running opposite ways that the sweep insists
    /// on. Hand it back so a face can hold both.
    fn a_triangle_and_its_mirror(
        model: &mut Model,
        corners: [[f64; 2]; 3],
    ) -> (WireSlot, WireSlot) {
        let vertices: Vec<VertexSlot> = corners
            .iter()
            .map(|corner| model.add_vertex(Point::new(corner[0], corner[1], 0.0)))
            .collect();
        let edges: Vec<EdgeSlot> = (0..3)
            .map(|index| model.add_edge(vertices[index], vertices[(index + 1) % 3]))
            .collect();
        let anticlockwise: SmallVec<[CoedgeSlot; 4]> = edges
            .iter()
            .map(|&edge| model.add_coedge(edge, Orientation::Forward))
            .collect();
        let clockwise: SmallVec<[CoedgeSlot; 4]> = edges
            .iter()
            .rev()
            .map(|&edge| model.add_coedge(edge, Orientation::Reversed))
            .collect();
        (model.add_wire(anticlockwise), model.add_wire(clockwise))
    }

    #[test]
    fn a_hole_winding_the_same_way_as_the_wire_round_it_is_caught() {
        let mut model = an_empty_model();
        let outline = a_triangle_and_its_mirror(&mut model, [[0.0, 0.0], [4.0, 0.0], [0.0, 4.0]]);
        let hole = a_triangle_and_its_mirror(&mut model, [[1.0, 1.0], [2.0, 1.0], [1.0, 2.0]]);
        model.add_face(
            Surface::Plane(the_ground()),
            // The outline anticlockwise and then the hole *also* anticlockwise,
            // which is the one thing wrong here. The two mirrors ride along at
            // the back so every edge still has its second coedge.
            [outline.0, hole.0, outline.1, hole.1].into_iter().collect(),
        );

        assert_eq!(refusal(&model), Invalid::HoleWindsWithTheOuterWire);
    }

    /// A triangle of area one half, sitting where `f64` cannot see it.
    ///
    /// Both cross products come to `10000000200000001` and `10000000200000000`,
    /// and the first of those is one above a number `f64` can hold at that size,
    /// so it rounds down onto the second. The difference an `f64` sum reports is
    /// zero; the difference is one.
    fn a_sliver_triangle_f64_cannot_measure(model: &mut Model) -> FaceSlot {
        let corners = [
            [0.0, 0.0],
            [100_000_001.0, 100_000_000.0],
            [100_000_002.0, 100_000_001.0],
        ];
        let (anticlockwise, clockwise) = a_triangle_and_its_mirror(model, corners);
        model.add_face(
            Surface::Plane(the_ground()),
            [anticlockwise, clockwise].into_iter().collect(),
        )
    }

    /// The `f64` fan the sweep used to read this sign off, kept here so a test
    /// can show it getting the answer wrong.
    fn fanned_in_f64(wire: WireView<'_>, normal: Vector) -> f64 {
        let mut total = Vector::zero();
        let mut fan_from: Option<Point> = None;
        let mut previous: Option<Point> = None;
        for point in wire.points() {
            if let (Some(apex), Some(start)) = (fan_from, previous) {
                total = total + (start - apex).cross(point - apex);
            } else {
                fan_from = Some(point);
            }
            previous = Some(point);
        }
        (total * 0.5).dot(normal)
    }

    #[test]
    fn a_sliver_winds_with_its_normal_even_where_adding_its_area_up_in_f64_says_it_does_not() {
        let mut model = an_empty_model();
        let face = a_sliver_triangle_f64_cannot_measure(&mut model);
        let view = FaceView::new(&model, face, Isometry::identity(), Orientation::Forward);

        assert_eq!(
            fanned_in_f64(view.outer_wire(), the_ground().normal()),
            0.0,
            "the f64 fan has to lose this sign for the test to mean anything"
        );
        assert_eq!(
            winding_of(view.outer_wire(), the_ground().normal()).expect("the cascade settles this"),
            Decision::Pos,
            "the exact sign is there to be had, and a zero would have the sweep \
             refuse this face for winding the wrong way"
        );
    }

    #[test]
    fn a_hole_in_a_plate_winds_against_the_face_it_is_cut_from() {
        let model = a_plate_with_a_hole();
        a_clean_sweep(&model);

        let cap = model.faces().next().expect("a plate has caps");
        let mut wires = cap.wires();
        let outline = wires.next().expect("the outline");
        let hole = wires.next().expect("and the hole");
        let normal = cap.plane().normal();

        assert_eq!(
            winding_of(outline, normal).expect("the cascade settles this"),
            Decision::Pos
        );
        assert_eq!(
            winding_of(hole, normal).expect("the cascade settles this"),
            Decision::Neg
        );
    }

    /// A ring with a corner that is not a corner does not wind either way.
    ///
    /// Before the guard this came back `Ok(Decision::Pos)`. The normal points
    /// straight up, so its other two components are zero, and the two minors
    /// that the broken height appears in were multiplied away — leaving the
    /// flat square to certify a winding for a ring that has no fourth corner.
    #[test]
    fn a_ring_with_a_corner_that_is_not_a_corner_does_not_wind() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for axis in 0..3 {
                let mut broken = [0.0, 0.0, 0.0];
                broken[axis] = bad;
                let ring = [broken, [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
                assert!(
                    winding_round(&ring, [0.0, 0.0, 1.0]).is_err(),
                    "{bad} on axis {axis}"
                );
            }
        }
    }

    /// The normal is the other half of the input, and it reached the same
    /// wrong answer: a component that is not a number sits beside a minor that
    /// the sound components decide on their own.
    #[test]
    fn a_ring_measured_round_a_normal_that_is_not_a_normal_does_not_wind() {
        let square = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for axis in 0..3 {
                let mut normal = [0.0, 0.0, 1.0];
                normal[axis] = bad;
                assert!(
                    winding_round(&square, normal).is_err(),
                    "{bad} on axis {axis}"
                );
            }
        }
        assert_eq!(
            winding_round(&square, [0.0, 0.0, 1.0]).expect("a flat square winds"),
            Decision::Pos
        );
    }
}
