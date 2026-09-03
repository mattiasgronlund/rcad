use std::collections::HashMap;

use rcad_geom::{
    Deflection, DynamicRange, EdgeRun, Isometry, NEAREST_F64_POINT_ROUNDOFF, Orientation, Plane,
    Point, SIDE_OF_CONE, SIDE_OF_CYLINDER, SIDE_OF_PLANE, SIDE_OF_SPHERE, SIDE_OF_TORUS, Surface,
    Vector,
};
use rcad_predicates::expansion::{
    expansion_sign, expansion_sum, product_is_exact, scale_expansion, two_product,
};
use rcad_predicates::{
    Approximation, Decision, Filtered, Interval, Level, Measures, NOTHING_IS_KNOWN,
    Point as PredicatePoint, Policy, Predicate, ProvenanceLog, Record, SideOfSurface,
    are_real_numbers, decide,
};
use slotmap::SecondaryMap;

use crate::fuse::name_of_surface_bits;
use crate::holds::{Trimming, the_face_across, the_side_this_face_lies_on, what_trims_this_face};
use crate::key::{CoedgeSlot, EdgeSlot, FaceSlot, ShellSlot, WireSlot};
use crate::measure::signed_volume_over;
use crate::mesh::Mesh;
use crate::model::WireData;
use crate::view::{CoedgeView, FaceView, SolidView, WireView};
use crate::{EdgeView, Error, Invalid, Model, Result};

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
    /// - every stored key names something that is still there — asked first,
    ///   for all seven arenas at once, because every check after it walks the
    ///   model through views and a view over a missing slot panics rather than
    ///   refusing (`ModelData::check_keys_resolve`),
    /// - no coordinate is infinite and no edge has zero length,
    /// - every edge is used by exactly two coedges running opposite ways,
    /// - every wire closes back on itself and has enough steps to bound
    ///   something: three, or one edge that closes already, or two different
    ///   edges that are not both straight,
    /// - every face sits on a plane, with its corners on that plane, its outer
    ///   wire winding with the face normal and its holes winding against it,
    ///   so the material is on one consistent side,
    /// - nothing is orphaned and nothing is used twice where it may only be
    ///   used once,
    /// - every solid encloses a positive amount of space, which is what says
    ///   its faces point outwards — over its own fan when it is a polyhedron,
    ///   and over a coarse mesh when it is not,
    /// - every side of that mesh is used by exactly two triangles running
    ///   opposite ways, which is what says a round cap is not on back to front,
    /// - every [`Approximation`] claims a length, and
    ///   `tol(vertex) >= tol(edge) >= tol(face)` over the entities carrying one
    ///   (roadmap §5.4, decisions.md §1.2),
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
    ///
    /// Also whatever [`SolidView::mesh`] refuses, for a model holding a curved
    /// solid — a refusal about the *sweep* rather than about the model, which is
    /// why [`how_far_the_sweeps_mesh_may_stray`] asks for the coarsest mesh
    /// there is.
    pub fn validate(&self, policy: &Policy, log: &mut ProvenanceLog) -> Result<()> {
        self.data().check_keys_resolve()?;
        self.check_vertices()?;
        self.check_edges()?;
        self.check_coedges()?;
        self.check_wires()?;
        self.check_faces()?;
        self.check_which_side_of_its_trimming_each_curved_face_lies_on()?;
        self.check_closed_edges_are_walked_the_way_their_surfaces_wind()?;
        self.check_shells_and_solids(policy, log)?;
        self.check_approximation_ordering()?;
        self.check_dynamic_range(policy, log)
    }

    /// The shape checks a public method is allowed to take for granted, asked
    /// of every entity, with no arithmetic and no [`Policy`].
    ///
    /// **A key resolving is not the same as a shape being possible.** A file
    /// naming an entity that is not there is refused before it becomes a
    /// `Model` (`ModelData::check_keys_resolve`), and every key in a face's
    /// wire list resolving says nothing about the list being empty. So a face
    /// with no wires, a wire that does not close and an edge walked by three
    /// coedges all read back cleanly — and then
    /// [`FaceView::outer_wire`](crate::FaceView::outer_wire) has no wire to
    /// hand back and [`Model::place_solid`]'s after-the-mutation checks assert
    /// about a shape the mutation did not make.
    ///
    /// Which is why this is asked at the door rather than at each of those
    /// sites. Making every accessor answer `Option` would hand the invalidity
    /// to every caller for ever; refusing the bytes keeps one truth, and it is
    /// the same truth [`Model::validate`] already tells — these three checks
    /// **are** the ones `check_edges`, `check_wires` and `check_faces` make,
    /// called from here as well.
    ///
    /// The rest of the sweep stays out: a corner off its face's plane, a wire
    /// wound against its normal or a solid enclosing negative space all need
    /// predicates, a `Policy` to say how hard to try, and for a curved solid a
    /// mesh. Reading a file is not the place to spend that, and
    /// [`Model::validate`] is right there.
    ///
    /// # What belongs here
    ///
    /// **Every check that needs no [`Policy`], no arithmetic beyond
    /// `is_finite`, and no sweep over pairs** — because those are exactly the
    /// checks a public method is entitled to take for granted, and a model that
    /// fails one of them is not a model at any strictness
    /// (`docs/decisions.md` §44.5). The membership was decided one panic at a
    /// time until that was written down.
    ///
    /// [`Invalid::VertexNotFinite`] is the first check the rule moved. A corner
    /// at no point at all passes every question above it — its slot resolves,
    /// its wires close — and then every predicate downstream is asked about a
    /// coordinate that is not a number. `check_vertices` has always caught it,
    /// and `check_vertices` is in the sweep a file's own read never runs.
    ///
    /// **The rule also reaches [`Invalid::UsedMoreThanOnce`], and that check is
    /// asked one step earlier than this one.** Counting how many shells name a
    /// face is one pass and an integer per face — no policy, no arithmetic, and
    /// nothing that looks at a pair — so it belongs at the door; it is not in
    /// *this* function because what it protects is the index, and
    /// [`Model::from_parts`] builds the index before there is a `Model` to ask
    /// this of. [`ModelData::check_nothing_is_used_more_than_once`] runs beside
    /// `check_keys_resolve` instead, and says why there.
    ///
    /// # Errors
    ///
    /// [`Invalid::VertexNotFinite`], [`Invalid::FaceHasNoWires`],
    /// [`Invalid::WireTooShort`], [`Invalid::WireDoesNotClose`],
    /// [`Invalid::EdgeCoedgeCount`] or [`Invalid::CoedgesRunTheSameWay`],
    /// naming the first one that does not hold.
    pub(crate) fn check_shape_is_possible(&self) -> Result<()> {
        for vertex in self.data().vertices.values() {
            if !vertex.point.is_finite() {
                return Err(Invalid::VertexNotFinite.into());
            }
        }
        for face in self.data().faces.values() {
            if face.wires.is_empty() {
                return Err(Invalid::FaceHasNoWires.into());
            }
        }
        for wire in self.data().wires.values() {
            self.check_wire_closes(wire)?;
        }
        for slot in self.data().edges.keys() {
            self.check_edge_is_walked_by_two_opposite_coedges(slot)?;
        }
        Ok(())
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
        let rounding = self.what_writing_a_plane_down_costs();
        let writing_a_point_down = self.what_writing_a_point_down_costs();
        for (slot, edge) in &self.data().edges {
            let (Some(start), Some(end)) = (
                self.data().vertices.get(edge.start),
                self.data().vertices.get(edge.end),
            ) else {
                return Err(Invalid::DanglingKey { entity: "vertex" }.into());
            };
            // A closed edge is *supposed* to come back to where it started, so
            // its two ends being one vertex says it is a whole circle and not
            // that it has no length. Only a straight edge is refused for it.
            if edge.run == EdgeRun::Straight && (edge.start == edge.end || start.point == end.point)
            {
                return Err(Invalid::EdgeHasNoLength.into());
            }

            self.check_edge_is_walked_by_two_opposite_coedges(slot)?;

            // The other half of Q16's invariant, and the half that keeps an
            // edge's *name* honest. An edge named by a crossing has to be on
            // both surfaces of it, and inside every half-space that was used to
            // pick which piece it is. Otherwise the name describes a curve this
            // edge is not on, and nothing else in the kernel would notice.
            //
            // A plane is asked with `what_writing_a_plane_down_costs` of room,
            // which is the same allowance `check_flat_face` gives a corner
            // against the plane its face *sits* on. Asking a stored *name* more
            // strictly than the surface a face is actually carried by had it
            // backwards: it refused corners for the rounding in the name.
            if let EdgeRun::PartOfWhereItCrosses { and, picked_by } = &edge.run {
                for corner in [start, end] {
                    let at = PredicatePoint::from(corner.point.coordinates());
                    match side_of_surface(and, at) {
                        Decision::Zero => {}
                        // The same slop `check_curved_face` reads off a solved
                        // corner (§36.13), read here too. A corner where three
                        // surfaces meet is a box divided down to a point, so it
                        // lands *near* each of them and exactly on one only when
                        // the arithmetic happened to be exact.
                        _ if not_provably_off(
                            and,
                            corner.point,
                            corner.approximation,
                            writing_a_point_down,
                        ) => {}
                        _ if off_a_plane_by_no_more_than_writing_it_down_cost(
                            and,
                            corner.point,
                            rounding,
                        ) => {}
                        Decision::Pos | Decision::Neg => {
                            return Err(Invalid::VertexOffItsFaceSurface.into());
                        }
                        decision @ Decision::Undecidable { .. } => {
                            let mut records = ProvenanceLog::new();
                            records.push(Record::new(
                                name_of_surface(and),
                                &[at],
                                Level::Exact,
                                decision,
                                Measures::Area,
                            ));
                            return Err(Error::Undecidable { records });
                        }
                        other => panic!(
                            "a corner came back with a decision nothing here knows: {other:?}"
                        ),
                    }
                    if picked_by
                        .iter()
                        .any(|picker| picker.side_of_point(at) == Decision::Neg)
                    {
                        return Err(Invalid::EdgeOutsideWhatPickedIt.into());
                    }
                }
            }
        }
        Ok(())
    }

    /// Whether every face of this solid sits on a plane.
    ///
    /// The checks that sum over triangles mean a polyhedron, and only a wholly
    /// flat solid is one.
    fn solid_is_wholly_flat(&self, slot: crate::key::SolidSlot) -> bool {
        self.data().solids[slot].shells.iter().all(|used| {
            self.data().shells[used.shell]
                .faces
                .iter()
                .all(|face| self.data().faces[face.face].surface.as_plane().is_some())
        })
    }

    fn check_coedges(&self) -> Result<()> {
        let mut used: SecondaryMap<CoedgeSlot, usize> = SecondaryMap::new();
        for wire in self.data().wires.values() {
            for coedge in wire.coedges() {
                if let Some(entry) = used.entry(coedge) {
                    *entry.or_insert(0) += 1;
                }
            }
        }
        for slot in self.data().coedges.keys() {
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
            self.check_wire_closes(wire)?;

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

    /// A wire long enough to bound something, whose steps chain end to start
    /// and come back to where they set off.
    ///
    /// Three steps, or one edge that closes already, or two different edges
    /// that are not both straight —
    /// [`Model::wire_of_two_steps_bounds_a_face`] says why those two shapes and
    /// no others.
    ///
    /// Also the check [`Model::check_touched_wire`] makes after a mutation, and
    /// that is on purpose: one of the two audiences wants a refusal and the
    /// other wants a crash, and there being two of them is no reason for two
    /// answers.
    ///
    /// # Errors
    ///
    /// [`Invalid::WireTooShort`] or [`Invalid::WireDoesNotClose`].
    pub(crate) fn check_wire_closes(&self, wire: &WireData) -> Result<()> {
        if wire.steps.len() < 3
            && !self.wire_closes_on_one_edge(wire)
            && !self.wire_of_two_steps_bounds_a_face(wire)
        {
            return Err(Invalid::WireTooShort {
                count: wire.steps.len(),
            }
            .into());
        }
        // A pole is walked like any other step and simply goes nowhere, so
        // closure is one rule rather than two.
        let mut previous_end = None;
        for (position, &step) in wire.steps.iter().enumerate() {
            let (start, end) = self.step_ends(step);
            if previous_end.is_some_and(|previous| previous != start) {
                return Err(Invalid::WireDoesNotClose { position }.into());
            }
            previous_end = Some(end);
        }
        let first_start = self.step_ends(wire.steps[0]).0;
        if previous_end != Some(first_start) {
            return Err(Invalid::WireDoesNotClose {
                position: wire.steps.len(),
            }
            .into());
        }
        Ok(())
    }

    /// An edge walked by exactly two coedges, running opposite ways — which is
    /// what says the two faces meeting there agree which side the material is
    /// on.
    ///
    /// Shared with [`Model::check_touched_edge`] for the reason
    /// [`Model::check_wire_closes`] gives.
    ///
    /// # Errors
    ///
    /// [`Invalid::EdgeCoedgeCount`] or [`Invalid::CoedgesRunTheSameWay`].
    pub(crate) fn check_edge_is_walked_by_two_opposite_coedges(
        &self,
        slot: EdgeSlot,
    ) -> Result<()> {
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
        let writing_a_point_down = self.what_writing_a_point_down_costs();
        for (slot, face) in &self.data().faces {
            if face.wires.is_empty() {
                return Err(Invalid::FaceHasNoWires.into());
            }

            let view = FaceView::new(self, slot, Isometry::identity(), Orientation::Forward);
            if let Surface::Plane(plane) = &face.surface {
                check_flat_face(view, plane, allowed_wobble)?;
            } else {
                check_curved_face(view, &face.surface, writing_a_point_down)?;
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

    /// Every curved face on one side of everything that trims it.
    ///
    /// **The curved twin of the winding check**, and what is left of §31.3's
    /// "every wire keeps the face's material on its left" once you notice there
    /// is no direction rule to read a closed edge's own sense from (§30.8 item
    /// 10). What *can* be asked without one is this: a crossing that goes right
    /// round cuts a surface into exactly two pieces, so a face whose whole
    /// boundary is such crossings is one of those pieces, and one corner on each
    /// side of one of them is a model whose topology and geometry disagree.
    ///
    /// **Only asked of a face whose every side is a whole closed crossing**, and
    /// not because the rest is hard. A face trimmed by a *piece* of a crossing
    /// straddles that piece's surface quite properly — a bar with a notch cut in
    /// its wall does — so asking there would refuse sound solids.
    ///
    /// It is the invariant [`SolidView::face_holds_the_point`] rests on, and the
    /// sweep asks it of every face of every solid at once. **It is not a hoist**
    /// — the query path still asks its own face on its own account, because a
    /// model can be mutated between a sweep and a query and a check that was
    /// trusted afterwards would be a cache with no invalidation (§4.3).
    ///
    /// # Errors
    ///
    /// [`Invalid::AFaceStraddlesSomethingTrimmingIt`], and
    /// [`Error::Undecidable`] when a corner's side could not be settled.
    fn check_which_side_of_its_trimming_each_curved_face_lies_on(&self) -> Result<()> {
        for (slot, _) in &self.data().solids {
            let solid = SolidView::new(self, slot, Isometry::identity(), Orientation::Forward);
            for face in solid.faces() {
                if face.surface().as_plane().is_some() {
                    continue;
                }
                // A face trimmed by part of a curve is not one this invariant is
                // about, and saying so by name is the difference between a
                // precondition and swallowing whatever went wrong: every other
                // refusal `what_trims_this_face` can make is about a model this
                // sweep exists to catch.
                let trimming = match what_trims_this_face(solid, face) {
                    Ok(trimming) => trimming,
                    Err(Error::AFaceTrimmedByACurveThisCannotFollow { .. }) => continue,
                    Err(why) => return Err(why),
                };
                for one in &trimming {
                    the_side_this_face_lies_on(face, one)?;
                }
            }
        }
        Ok(())
    }

    /// Every closed edge walked the way the direction rule says.
    ///
    /// **This is §30.8 item 10 becoming a check rather than staying a rule.** A
    /// closed edge starts and ends at the one corner it has, so nothing in the
    /// model says which way it runs and only a rule can. The fuse follows the
    /// rule when it writes one and until now nothing read it back: a bar's wall
    /// with both its cap circles walked the same way round closes up, counts
    /// right, and is bounded by nothing at all.
    ///
    /// The rule itself is [`a_closed_crossing_is_walked_the_way_it_is_stored`],
    /// the very function the fuse chose with. What this adds is the two things
    /// it has to be asked *about*.
    ///
    /// **Which side of the partner the face's material lies on** is read off a
    /// corner of the face that is off it, which is the only place a model says it
    /// ([`the_side_this_face_lies_on`]). A face with no such corner — a bar's cap
    /// has its one corner *on* the wall — says nothing, and is skipped rather
    /// than guessed at.
    ///
    /// **Both gradients are the ones the solid uses**, which is the clause §36.6
    /// got wrong: it says the face's orientation does not come into it, and for a
    /// curved face a fuse turned round it is the whole of what carries the turn —
    /// a sphere has no reversed form, so the wire is stored walked the other way
    /// and `FaceUse::orient` is what says so. Reading the stored gradient instead
    /// would read the rule off a face pointing into its own material. The
    /// coedge's own turn is read **unturned** for the same reason: it is what the
    /// fuse stored after turning the wire round, not what a view composes back.
    ///
    /// # Errors
    ///
    /// [`Invalid::AClosedEdgeIsWalkedAgainstItsSurface`], and whatever reading a
    /// corner's side refuses.
    fn check_closed_edges_are_walked_the_way_their_surfaces_wind(&self) -> Result<()> {
        for (slot, _) in &self.data().solids {
            let solid = SolidView::new(self, slot, Isometry::identity(), Orientation::Forward);
            for face in solid.faces() {
                for coedge in face.wires().flat_map(WireView::coedges) {
                    let edge = coedge.edge();
                    if *edge.stored_run() != EdgeRun::RightRoundWhereItsFacesCross {
                        continue;
                    }
                    let Some(across) = the_face_across(solid, face, edge)? else {
                        continue;
                    };
                    let Some(material) = which_side_of_it_this_face_lies_on(solid, face, across)?
                    else {
                        continue;
                    };
                    let at = edge.start().point();
                    let outward = |used: FaceView<'_>| {
                        gradient_of(&used.surface(), at) * used.orientation().sign()
                    };
                    let Some(walks_it_forward) = a_closed_crossing_is_walked_the_way_it_is_stored(
                        [&face.surface(), &across.surface()],
                        [outward(face), outward(across)],
                        at,
                        turned_by(material, across.orientation()),
                    )?
                    else {
                        continue;
                    };
                    if walks_it_forward
                        != (coedge.orientation_on_its_edge() == Orientation::Forward)
                    {
                        return Err(Invalid::AClosedEdgeIsWalkedAgainstItsSurface.into());
                    }
                }
            }
        }
        Ok(())
    }

    fn check_shells_and_solids(&self, policy: &Policy, log: &mut ProvenanceLog) -> Result<()> {
        let mut used: SecondaryMap<ShellSlot, usize> = SecondaryMap::new();
        for solid in self.data().solids.values() {
            for shell in &solid.shells {
                if let Some(entry) = used.entry(shell.shell) {
                    *entry.or_insert(0) += 1;
                }
            }
        }
        for slot in self.data().shells.keys() {
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

        for slot in self.data().solids.keys() {
            let view = SolidView::new(self, slot, Isometry::identity(), Orientation::Forward);
            // A polyhedron's own fan *is* its volume, so nothing is meshed for
            // one. A curved solid has no fan — a round bar's caps fan into no
            // triangles at all and its wall into two with no area — and until
            // there was a mesher this check simply did not run for it.
            let volume = if self.solid_is_wholly_flat(slot) {
                view.signed_volume()
            } else {
                how_much_space_a_curved_solid_encloses(view, policy, log)?
            };
            if !volume.is_finite() {
                return Err(Error::VolumeTooLargeToMeasure);
            }
            if volume <= 0.0 {
                return Err(Invalid::VolumeIsNotPositive { volume }.into());
            }
        }
        Ok(())
    }

    /// A thing built from another may not claim less slop than what it is built
    /// from — and every amount claimed is a length to begin with.
    ///
    /// **The second half is asked first, because the first half is a `<`.**
    /// Infinity and `NaN` are not ordered by `<`, so a sweep that only compared
    /// magnitudes would file "this face is infinitely out of place" as tidy.
    /// What that magnitude really does is switch the certified check off:
    /// [`not_provably_off`] keeps any magnitude above zero and widens the corner
    /// into an interval that straddles every surface there is.
    ///
    /// **There are two ways an [`Approximation`] gets into a model and both
    /// refuse one**: [`Approximation::new`](rcad_predicates::Approximation::new)
    /// is the only constructor and it asserts, and `serde` is the only other
    /// route in and it errors
    /// (`a_file_saying_a_corner_is_infinitely_out_of_place_never_becomes_a_model`).
    /// So this arm cannot be reached today. It is here for the same reason
    /// `check_keys_resolve` is at the head of [`Model::validate`] when
    /// `Deserialize` has already run it: the sweep says what it depends on out
    /// loud rather than depending on it silently.
    fn check_approximation_ordering(&self) -> Result<()> {
        for (entity, magnitude) in self.every_amount_of_slop_claimed() {
            if !(magnitude.is_finite() && magnitude >= 0.0) {
                return Err(Invalid::ApproximationIsNotALength { entity, magnitude }.into());
            }
        }
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
                for coedge in self.data().wires[wire].coedges() {
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

    /// Every slop any entity in the model claims, named by what claims it.
    ///
    /// An entity with no tag is left out rather than reported as zero. Zero is
    /// what `magnitude_of` stands in for a missing tag when the ordering is
    /// being read, and a missing tag is not a claim about anything.
    fn every_amount_of_slop_claimed(&self) -> impl Iterator<Item = (&'static str, f64)> + '_ {
        let vertices = self
            .data()
            .vertices
            .values()
            .filter_map(|vertex| vertex.approximation)
            .map(|tag| ("vertex", tag.magnitude()));
        let edges = self
            .data()
            .edges
            .values()
            .filter_map(|edge| edge.approximation)
            .map(|tag| ("edge", tag.magnitude()));
        let faces = self
            .data()
            .faces
            .values()
            .filter_map(|face| face.approximation)
            .map(|tag| ("face", tag.magnitude()));
        vertices.chain(edges).chain(faces)
    }

    /// The coordinate magnitude is measured **where the solids actually are**,
    /// not where their vertices are stored.
    ///
    /// A solid placed a long way out keeps ordinary small numbers in the arena
    /// and carries the distance in its location, so reading the arena says a
    /// model is comfortable when every predicate that runs on it is working at
    /// `2^39`. A fuse bakes placement in (§17.3), so the same geometry read the
    /// arena's way passed this check before a fuse and failed it after — the
    /// same model, two answers, decided by where the placement happened to be
    /// written down. What the check is about is the numbers the predicates see,
    /// and those are located.
    ///
    /// Lengths are not affected by placement, so the smallest feature is read
    /// straight off the edges.
    fn check_dynamic_range(&self, policy: &Policy, log: &mut ProvenanceLog) -> Result<()> {
        let largest = self
            .solids()
            .flat_map(|solid| solid.faces())
            .flat_map(|face| face.wires())
            .flat_map(|wire| wire.points())
            .map(|corner| corner.largest_coordinate_magnitude())
            .fold(0.0, f64::max);
        // A closed edge is skipped rather than measured, because the distance
        // between its two ends is zero and that is not how small it is — it is
        // a whole circle. What stands in for it is the radius of the curved
        // surface it runs round, taken from the faces below, which is the
        // smallest thing about such an edge that anything here can name.
        let smallest_chord = self
            .edges()
            .filter(|edge| !edge.is_closed())
            .map(EdgeView::distance_between_its_ends)
            .fold(f64::INFINITY, f64::min);
        let smallest_radius = self
            .solids()
            .flat_map(|solid| solid.faces())
            .filter_map(|face| match face.surface() {
                Surface::Plane(_) => None,
                Surface::Sphere(ball) => Some(ball.radius()),
                Surface::Cylinder(pipe) => Some(pipe.radius()),
                // **A cone's apex is a vertex and a vertex has no size**, the
                // same way a cube's corner has none, so the radius *there* is
                // not a feature this is asking about. What the radius stands in
                // for is the closed edge a curved face carries, whose chord is
                // zero and whose real size is the circle it runs round — and a
                // closed edge on a cone never sits at the apex, because a circle
                // of no radius is a point. So the corners at the apex are passed
                // over and the smallest circle the face actually carries is what
                // comes back.
                Surface::Cone(cone) => {
                    let along = cone.axis().direction();
                    face.wires()
                        .flat_map(WireView::points)
                        .map(|corner| cone.radius_at(along.dot(corner - cone.apex())))
                        .filter(|radius| *radius > 0.0)
                        .fold(f64::INFINITY, f64::min)
                        .into()
                }
                // **A torus has two radii and the smaller is not always the
                // tube.** The closed curves a torus face carries are the circles
                // round the tube, of radius `r`, and the circles round the axis,
                // whose smallest is the inner equator at `R − r`. Either can be
                // the smallest feature of the solid, so both go in.
                Surface::Torus(ring) => {
                    Some((ring.ring_radius() - ring.tube_radius()).min(ring.tube_radius()))
                }
                // Every variant there is has an arm above; this is only the
                // one that does not exist yet, and a surface whose smallest
                // feature nobody has worked out is safest counted as having
                // none.
                _ => None,
            })
            .filter(|radius| radius.is_finite())
            .fold(f64::INFINITY, f64::min);
        let smallest_feature = smallest_chord.min(smallest_radius);
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
    /// [`Model::what_writing_a_plane_down_costs`].
    ///
    /// **Only the second half is allowed against a plane an edge is merely
    /// *named* by** — see that function.
    fn allowed_wobble(&self) -> f64 {
        self.scale().default_linear_tolerance() + self.what_writing_a_plane_down_costs()
    }

    /// How far off a plane a point exactly on it can read, purely because the
    /// plane was written down in `f64`.
    ///
    /// [`Plane`] scales its normal to length one and rounds its offset, so the
    /// plane the numbers *are* is a few ulps from the plane the numbers *mean*,
    /// and a point exactly on the second reads a nonzero distance to the first.
    /// That is what the plane says its own arithmetic costs
    /// ([`Plane::SIGNED_DISTANCE_ROUNDOFF`]) times the largest coordinate in the
    /// model, times [`MEASURING_SLACK`].
    ///
    /// **It is not a modelling tolerance and nothing here may turn it into
    /// one.** It is the width of the number: a corner inside it is one no `f64`
    /// could have written any nearer, and a corner outside it is off a plane
    /// somebody chose rather than off a plane that rounded.
    fn what_writing_a_plane_down_costs(&self) -> f64 {
        MEASURING_SLACK * Plane::SIGNED_DISTANCE_ROUNDOFF * self.largest_coordinate()
    }

    /// How far off a curved surface a corner exactly on it can land, purely
    /// because the corner had to be written down in `f64`.
    ///
    /// **The curved twin of [`Model::what_writing_a_plane_down_costs`], and a
    /// length rather than a value** — a ball, a pipe, a cone and a ring answer
    /// with a sign and hand no distance out, so what is budgeted here is where
    /// the corner is and [`not_provably_off`] turns it into an answer.
    /// [`NEAREST_F64_POINT_ROUNDOFF`] says a point on a surface, written as
    /// three `f64`, lands within that times the largest coordinate and no
    /// writer could have done better; [`MEASURING_SLACK`] is this check's own
    /// room on top, for the surface's numbers having rounded as well.
    ///
    /// **It is not a modelling tolerance and nothing here may turn it into
    /// one.** [`Model::allowed_wobble`] deliberately does *not* appear: a flat
    /// face may sit a `default_linear_tolerance` off its plane, and a corner of
    /// a curved face may sit off its surface only by the width of the number.
    ///
    /// The size it is measured against is the largest coordinate any corner of
    /// the model holds, which is the same size the flat check uses so that the
    /// two have one standard. It does **not** grow with a surface's own numbers,
    /// so a pipe of radius `1e9` carrying a face at the origin gets a budget
    /// smaller than its own radius rounds by. That is the safe direction and it
    /// is untested here: too small a budget refuses a sound corner, where too
    /// large a one accepts a corner nobody placed.
    fn what_writing_a_point_down_costs(&self) -> f64 {
        MEASURING_SLACK * NEAREST_F64_POINT_ROUNDOFF * self.largest_coordinate()
    }

    /// The largest coordinate magnitude any corner of the model holds.
    fn largest_coordinate(&self) -> f64 {
        self.data()
            .vertices
            .values()
            .map(|vertex| vertex.point.largest_coordinate_magnitude())
            .fold(0.0, f64::max)
    }
}

/// How much space a curved solid encloses, signed by which way its faces point,
/// and whether the two faces along each of its sides agree.
///
/// A polyhedron's own fan *is* its volume; a curved solid has no such fan — a
/// round bar's caps fan into no triangles at all and its wall into two with no
/// area (§35.4) — so it is summed over a mesh instead. Only the **sign** is read
/// off this, never the size.
///
/// # A coarse mesh that encloses exactly nothing has not answered
///
/// [`as_coarse_as_a_mesh_gets`] leans on the mesher never stepping more than a
/// quarter turn, which is what keeps a *whole* curved surface from collapsing.
/// **It says nothing about a patch that spans less than a quarter turn**, and
/// there the mesher has no reason to put a point anywhere: the patch is handed
/// to the triangulator with a budget it already meets, so it comes back as its
/// own boundary and nothing else — flat. Closed off by the flat face on the
/// other side of that boundary it encloses zero, to the last bit.
///
/// A ball cut by a plane is exactly that, and it is `docs/decisions.md` §41.6: a
/// cap under `h/r = 1 − cos(π/4) = 0.2929` spans under a quarter turn, and on a
/// ball of radius four that is a cap of 15.52 and 5.8% of the ball. **Cut square
/// to the ball's own polar axis the same cap answers**, because then it holds a
/// pole and is fanned up to it whatever the deflection — so the same two solids
/// a quarter turn apart answered one way round and refused the other, which is
/// the §19 violation §39.7 item 1 leads with.
///
/// So a zero is asked again with less room. Halving from the mesh's own width
/// rather than down from [`as_coarse_as_a_mesh_gets`] is what keeps it to a
/// handful of meshes: the bulge a flat patch is hiding is at most that width, and
/// every halving that does not find it rules out half of what is left. **The
/// finer meshes are read for a sign and thrown away**; nothing a caller measures
/// is worked out here.
///
/// **What stops it needs no argument about geometry.** Halving a finite length
/// reaches zero, and `Deflection::of` refuses a length that is not greater than
/// zero, so the loop ends whatever any mesh does. Long before that two named
/// refusals end it on a real solid — `DeflectionBelowWhatAMeshCanReach` at the
/// floor of the input's own numbers, and `MoreMeshPointsThanASolidWillHold` if a
/// surface reaches the point cap first — and either way the zero stands and the
/// caller refuses it.
///
/// # Errors
///
/// Whatever meshing the solid at its coarsest makes of it, and
/// [`Invalid::FacesDoNotAgreeWhichWayTheyFace`] when two faces along one side do
/// not.
pub(crate) fn how_much_space_a_curved_solid_encloses(
    view: SolidView<'_>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<f64> {
    let coarsest = view.mesh(as_coarse_as_a_mesh_gets(), policy, log)?;
    every_side_of_the_skin_is_used_both_ways(&coarsest)?;
    let enclosed = signed_volume_over(&coarsest);
    if enclosed != 0.0 {
        return Ok(enclosed);
    }
    let mut room = how_far_across(&coarsest) / 2.0;
    while let Ok(less) = Deflection::of(room) {
        // A finer mesh that refuses is the stopping condition and not a
        // failure: the coarsest one was built and checked, and what this is
        // looking for is a point off the flat that no mesh of this solid holds.
        // Handing the refusal on would replace a named answer about the shape
        // with one about a deflection nobody asked for.
        let Ok(finer) = view.mesh(less, policy, log) else {
            break;
        };
        let enclosed = signed_volume_over(&finer);
        if enclosed != 0.0 {
            return Ok(enclosed);
        }
        room /= 2.0;
    }
    Ok(0.0)
}

/// The longest side of the box a mesh's points sit in.
fn how_far_across(mesh: &Mesh) -> f64 {
    let mut lowest = [f64::MAX; 3];
    let mut highest = [f64::MIN; 3];
    for point in mesh.points() {
        for (axis, coordinate) in point.coordinates().into_iter().enumerate() {
            lowest[axis] = lowest[axis].min(coordinate);
            highest[axis] = highest[axis].max(coordinate);
        }
    }
    (0..3).fold(0.0, |widest: f64, axis| {
        widest.max(highest[axis] - lowest[axis])
    })
}

/// How far the mesh the sweep checks a curved solid over may stray.
///
/// **As coarse as it goes, and that is not a corner cut.** What is asked of this
/// mesh is a *sign* and a *count* — does the solid enclose a positive amount of
/// space, and do the two faces along each side agree which way they face — and
/// neither answer gets truer with more triangles. What stops a coarse mesh
/// collapsing is not the deflection: the mesher will not step more than a
/// quarter turn of a curved surface whatever it is handed, whichever way that
/// turn goes, so a bar still comes out four ways round and a ball still comes
/// out an octahedron. The sweep runs at every test boundary, and a mesh at the
/// model's own deflection would cost it a hundredfold for the same two answers.
///
/// **That is an argument about a whole surface and it does not carry to a
/// patch.** A face spanning less than a quarter turn is one step whatever the
/// deflection, and one step is flat.
/// [`how_much_space_a_curved_solid_encloses`] is where that is paid for; nothing
/// else reads a mesh this coarse.
///
/// **A number and not a fraction of the solid, because a fraction has a worst
/// case and this does not.** Half a radius sounds like the coarsest sensible
/// answer and is not: the chord a deflection allows on a pipe grows as
/// `√(3 · radius · deflection)`, so a bar of radius `1e-8` and length one —
/// which `an_untidy_radius_is_certified_onto_its_own_cylinder_too` builds —
/// would want thirty-five million points along its length for a mesh nobody
/// reads.
///
/// This used to make an exception for a surface that pinches to a point, on the
/// grounds that a ball meshed this coarsely came back with no triangles at all.
/// It did, and that was the mesher charging a step's turn to its longitude
/// alone; a pole-to-pole step turns half a turn in latitude and none in
/// longitude. The exception here would only ever have covered the one caller
/// that thought of it — `SolidView::measure` did not, and panicked — so the
/// floor went where it belongs and this went back to a constant.
pub(crate) fn as_coarse_as_a_mesh_gets() -> Deflection {
    Deflection::of(f64::MAX).expect("the largest f64 is a positive finite length")
}

/// Every side of the mesh used the same number of times each way round.
///
/// §23.4's oracle, and the only thing in this sweep that can catch a round cap
/// put on back to front. A cap's boundary is one closed edge and one corner, so
/// [`winding_of`] has no polygon to ask about; the tessellated cap has one, and
/// a cap facing the wrong way runs its triangles round the shared circle the
/// same way the wall does instead of the opposite way.
///
/// **The volume check does not catch it and cannot.** Hang a bar's sums from a
/// corner of its bottom rim: the bottom cap lies in the plane it is measured
/// from and contributes nothing whichever way it faces, and flipping the top one
/// leaves `πr²h/3` — still positive, still no complaint.
///
/// Counted as a balance rather than as two counts, which is what makes "twice
/// the same way round" a failure rather than a pass — and is also **weaker than
/// §23.4's oracle**, which says *exactly* two. A side used four times, twice
/// each way, balances to zero and passes here. The suite's own copy in
/// `meshing_a_curved_solid.rs` is the same balance, so nothing in the kernel
/// asks for the count; saying so is cheaper than a check with no input that
/// fails it.
pub(crate) fn every_side_of_the_skin_is_used_both_ways(mesh: &Mesh) -> Result<()> {
    let mut balance: HashMap<(u32, u32), i32> = HashMap::new();
    for triangle in mesh.triangles() {
        let [first, second, third] = triangle.corners();
        for (from, to) in [(first, second), (second, third), (third, first)] {
            *balance.entry((from.min(to), from.max(to))).or_default() +=
                if from < to { 1 } else { -1 };
        }
    }
    if balance.values().any(|&count| count != 0) {
        return Err(Invalid::FacesDoNotAgreeWhichWayTheyFace.into());
    }
    Ok(())
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

/// Every corner on the plane, the outer wire winding with the normal and the
/// holes against it, so the material is on one consistent side.
fn check_flat_face(view: FaceView<'_>, plane: &Plane, allowed_wobble: f64) -> Result<()> {
    for wire in view.wires() {
        for point in wire.points() {
            let distance = plane.signed_distance_to(point).abs();
            if distance > allowed_wobble {
                return Err(Invalid::VertexOffItsFacePlane { distance }.into());
            }
        }
    }
    for (position, wire) in view.wires().enumerate() {
        // Fewer than three corners is not a polygon, so there is nothing here
        // to wind. A round cap's boundary is one closed edge and one corner; a
        // face bounded by an arc and its chord has two, and two corners sweep
        // `(p × q) · n + (q × p) · n`, which cancels to exactly zero whichever
        // way round the face is. Which way such a boundary goes is a fact about
        // the curve, and the corners are all this can see.
        //
        // Which way such a face points is asked over the mesh instead, by
        // `every_side_of_the_skin_is_used_both_ways` — where it does have a
        // polygon and a neighbour to disagree with.
        if wire.points().take(3).count() < 3 {
            continue;
        }
        let winding = winding_of(wire, plane.normal())?;
        if position == 0 && winding != Decision::Pos {
            return Err(Invalid::OuterWireWindsAgainstTheNormal.into());
        }
        if position > 0 && winding != Decision::Neg {
            return Err(Invalid::HoleWindsWithTheOuterWire.into());
        }
    }
    Ok(())
}

/// Every corner **certified** on the surface the face sits on.
///
/// This is the curved twin of the two checks above, and it is one check rather
/// than two because a curved face has no `signed_distance_to` to be near and no
/// single normal to wind about. What it has instead is a sign predicate, so the
/// question is not "how far off is this corner" but "can anything prove this
/// corner is off" — and a corner that is genuinely on the surface answers
/// [`Decision::Zero`] exactly.
///
/// That is a stronger check than the flat one and it is affordable here for a
/// reason worth writing down: the builders this milestone ships put corners at
/// quarter-turn positions on an axis-aligned surface, where the coordinates are
/// exact and `x² + y² − r²` cancels to exactly zero. The day it starts coming
/// back [`Decision::Undecidable`] is the day a general-angle arc arrives, and
/// that is information rather than an obstacle (`docs/decisions.md` §16.8 on
/// why general-angle arcs were not built).
///
/// The points come from a view built on [`Isometry::identity`], so they are in
/// the face's own frame. That is not an accident: reading them through a
/// composed location walks a coordinate an ulp per read (§19.1), and this would
/// then refuse a sound model for a defect in the accessor.
///
/// # Errors
///
/// [`Invalid::VertexOffItsFaceSurface`] when a corner is certified off the
/// surface, and [`Error::Undecidable`] when nothing can prove either way.
/// Which side of `surface` this point is on, for whichever surface it is.
///
/// One place, because three callers ask it — the corners of a curved face, the
/// ends of an edge named by a crossing, and the mesher working out which half of
/// a meridian a seam is — and two spellings of the same question is how they
/// come to disagree.
///
/// # Panics
///
/// If `surface` is a kind nothing here can ask about. `Surface` grows a variant
/// at every milestone, and a surface checked by not being checked is worse than
/// a stop.
pub(crate) fn side_of_surface(surface: &Surface, point: PredicatePoint) -> Decision {
    match surface {
        Surface::Plane(flat) => flat.side_of_point(point),
        Surface::Sphere(ball) => ball.side_of(point),
        Surface::Cylinder(pipe) => pipe.side_of(point),
        Surface::Cone(cone) => cone.side_of(point),
        Surface::Torus(ring) => ring.side_of(point),
        other => panic!("a surface nothing here can ask about: {other:?}"),
    }
}

/// Whether a wire walks a closed crossing the way that edge is stored running.
///
/// **The closed-edge direction rule, in one place.** A closed edge starts and
/// ends at the one corner it has, so its ends say nothing about which way it
/// goes and only a rule can. Two facts meet.
///
/// **Which way the edge is stored running** is the way a surface's own second
/// coordinate takes the points along it, which is the mesher's order and is
/// canonical (`docs/decisions.md` §36.18). That longitude runs along `ẑ × ∇C`
/// for `ẑ` the frame axis the curved one of `pair` is wound about, and only the
/// *sign* of a dot against it is read — so nothing is normalized and no angle is
/// built.
///
/// **Which way the wire runs in space** is the direction that keeps the face's
/// own material on its left, and left of `∇S × ∇T` is always `T`'s negative side
/// — `(∇S × ∇T) × ∇S · ∇T = −‖∇S × ∇T‖²` by Cauchy–Schwarz, everywhere, with no
/// case analysis (§36.6). So the wire runs along `outward[0] × outward[1]`
/// exactly when its material lies inside the partner, which is what
/// `material_side` says.
///
/// `outward` are the two gradients **as their faces are used**: a curved surface
/// cannot be stored reversed, so a face a fuse turned round is the same surface
/// with its wires walked the other way and its use saying so, and reading the
/// stored gradient would read the rule off a face pointing into its own
/// material.
///
/// Written once because the fuse chooses with it and `Model::validate` reads the
/// answer back with it, and two spellings of one question is how those two come
/// to disagree (§23.8).
///
/// # Errors
///
/// [`Invalid::ClosedEdgeBetweenTwoFlatFaces`] if neither surface is curved. Two
/// planes cross in a straight line, which does not close, so no closed edge is
/// named by a flat pair — this is bad caller input rather than a broken
/// invariant, because `Model` derives `Deserialize` and nothing upstream of
/// this check has confirmed the edge's own claim is possible
/// (`docs/decisions.md` §39.7 item 4, not §16.11 — that section is about the
/// dynamic-range check being a ratio and says nothing about panics on a
/// caller's bytes; the rule is CLAUDE.md's).
///
/// # Panics
///
/// If `material_side` is neither [`Decision::Pos`] nor [`Decision::Neg`].
/// This one *is* a broken internal invariant and not caller input: the only
/// caller, [`Model::validate`](crate::Model::validate)'s own
/// `check_closed_edges_are_walked_the_way_their_surfaces_wind`, only ever
/// passes a value [`which_side_of_it_this_face_lies_on`] read off a real
/// corner, and that function's own `Ok(Some(_))` is never anything but
/// [`Decision::Pos`] or [`Decision::Neg`] — it filters `Zero` out and turns
/// `Undecidable` into an `Err` before it can reach here.
pub(crate) fn a_closed_crossing_is_walked_the_way_it_is_stored(
    pair: [&Surface; 2],
    outward: [Vector; 2],
    at: Point,
    material_side: Decision,
) -> Result<Option<bool>> {
    let Some(curved) = the_curved_one_of(&[*pair[0], *pair[1]]) else {
        return Err(Invalid::ClosedEdgeBetweenTwoFlatFaces.into());
    };
    let longitude = the_way_it_is_parametrised_about(&curved, at).cross(gradient_of(&curved, at));
    // **At a pole there is no longitude to compare against** (§38.3). Every
    // longitude runs through the one point there, so `ẑ × ∇C` is exactly the
    // zero vector and the sign of a dot with it is not a fact about the shape.
    // Nothing is guessed: the caller that writes an edge refuses, and the caller
    // that reads one back skips, the same way it skips a face with no corner off
    // its partner.
    if longitude == Vector::zero() {
        return Ok(None);
    }
    let runs_along_stored = outward[0].cross(outward[1]).dot(longitude) > 0.0;
    Ok(Some(match material_side {
        Decision::Neg => runs_along_stored,
        Decision::Pos => !runs_along_stored,
        other => panic!("a face with no side to its material: {other:?}"),
    }))
}

/// A side read against a surface, turned round when the face on it is used
/// reversed.
fn turned_by(side: Decision, orientation: Orientation) -> Decision {
    match (side, orientation) {
        (side, Orientation::Forward) => side,
        (Decision::Pos, Orientation::Reversed) => Decision::Neg,
        (Decision::Neg, Orientation::Reversed) => Decision::Pos,
        (other, Orientation::Reversed) => other,
    }
}

/// Which side of the face across an edge this face lies on, as its own corners
/// say.
///
/// `None` when every corner of it is on that surface and nothing says — a bar's
/// cap has its one corner on the wall, and that is the case §36.6 already names.
fn which_side_of_it_this_face_lies_on(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    across: FaceView<'_>,
) -> Result<Option<Decision>> {
    let surface = across.surface();
    let mut corners_on_it = Vec::new();
    for coedge in face.wires().flat_map(WireView::coedges) {
        let edge = coedge.edge();
        if the_face_across(solid, face, edge)?.is_some_and(|other| other.surface() == surface) {
            corners_on_it.push(edge.start().key());
            corners_on_it.push(edge.end().key());
        }
    }
    the_side_this_face_lies_on(
        face,
        &Trimming {
            surface,
            corners_on_it,
        },
    )
}

/// Which of the two surfaces either side of an edge names which way it runs.
///
/// The bits sort them so that a pair of curved surfaces picks the same one
/// whichever way round it is written.
pub(crate) fn the_curved_one_of(carried_by: &[Surface; 2]) -> Option<Surface> {
    let mut curved: Vec<Surface> = carried_by
        .iter()
        .filter(|surface| surface.as_plane().is_none())
        .copied()
        .collect();
    curved.sort_by_key(name_of_surface_bits);
    curved.first().copied()
}

/// The axis a surface's first coordinate is wound about, at `at`.
///
/// Crossed into the surface's own gradient it gives the way that coordinate
/// runs, which is what a closed edge is walked by. A sphere is parametrised
/// about the frame's own `z` — `Sphere::point_at` says so in its arithmetic —
/// and a pipe and a cone about their stored axis direction. This is read only
/// for the *sign* of a dot product, so nothing here is normalized.
///
/// # A torus's is not one direction, and that is why this takes a point
///
/// A torus's first coordinate runs round the **tube**, so the axis it is wound
/// about is the tangent to the centre circle — which turns as you go round the
/// ring. Handed the torus's own axis instead, the answer at a meridian circle's
/// corner is square to that circle and the dot product below is **exactly
/// zero**, so the rule reads a sign off nothing and a bend of pipe is refused
/// whichever way round it is described. The tangent is `ê × w` with `ê` the way
/// out from the axis at `at`, and crossed into the gradient it comes out as
/// exactly the direction `Torus::point_at`'s first coordinate takes.
///
/// Every other surface here ignores `at` and gives what it always gave.
///
/// # Panics
///
/// On a flat surface, which has no longitude, and on a kind this has not been
/// taught.
pub(crate) fn the_way_it_is_parametrised_about(surface: &Surface, at: Point) -> Vector {
    match surface {
        Surface::Sphere(_) => Vector::new(0.0, 0.0, 1.0),
        Surface::Cylinder(pipe) => pipe.axis().direction(),
        Surface::Cone(cone) => cone.axis().direction(),
        Surface::Torus(ring) => {
            let along = ring.axis().direction();
            let out = at - ring.centre();
            (out - along * (out.dot(along) / along.dot(along))).cross(along)
        }
        Surface::Plane(_) => panic!("a plane has no longitude to be wound about"),
        other => panic!("this crate cannot say what {other:?} is wound about"),
    }
}

/// Which way a surface faces at a point, as a polynomial in the numbers it
/// stores with no normalization in it (§31.2).
///
/// # Panics
///
/// On a surface kind this has not been taught.
pub(crate) fn gradient_of(surface: &Surface, at: Point) -> Vector {
    match surface {
        Surface::Plane(flat) => flat.normal(),
        Surface::Sphere(ball) => at - ball.centre(),
        Surface::Cylinder(pipe) => {
            let (origin, along) = (pipe.axis().origin(), pipe.axis().direction());
            let out = at - origin;
            out * along.dot(along) - along * along.dot(out)
        }
        // The gradient of `|x × w|² − slope²(x·w)²` at `x = at − apex`, halved:
        // `|w|²x − (1 + slope²)(x·w)w`. It points out of whichever nappe the
        // point is on, which is what the sign predicate's own evenness in `x·w`
        // asks for.
        Surface::Cone(cone) => {
            let (apex, along) = (cone.apex(), cone.axis().direction());
            let out = at - apex;
            out * along.dot(along) - along * (along.dot(out) * (1.0 + cone.slope() * cone.slope()))
        }
        // The gradient of `(w·w)(x·x + R² − r²)² − 4R²‖x × w‖²` at
        // `x = at − centre`, divided by four:
        // `(w·w)(x·x + R² − r² − 2R²)x + 2R²(x·w)w`. On the outer equator that
        // comes to `(w·w)·2Rr·x`, pointing away from the axis; on the inner
        // equator to `−(w·w)·2Rr·x`, pointing towards it — which is out of the
        // tube both times, and is the whole of what a torus's normal has that no
        // quadric's here does.
        Surface::Torus(ring) => {
            let (centre, along) = (ring.centre(), ring.axis().direction());
            let out = at - centre;
            let (big, small) = (ring.ring_radius(), ring.tube_radius());
            let bulge = out.dot(out) + big * big - small * small - 2.0 * big * big;
            out * (along.dot(along) * bulge) + along * (2.0 * big * big * along.dot(out))
        }
        other => panic!("this crate cannot say which way {other:?} faces"),
    }
}

/// What the predicate that decided a surface's sign goes by in a record.
pub(crate) fn name_of_surface(surface: &Surface) -> &'static str {
    match surface {
        Surface::Plane(_) => SIDE_OF_PLANE,
        Surface::Sphere(_) => SIDE_OF_SPHERE,
        Surface::Cylinder(_) => SIDE_OF_CYLINDER,
        Surface::Cone(_) => SIDE_OF_CONE,
        Surface::Torus(_) => SIDE_OF_TORUS,
        other => panic!("a surface nothing here can ask about: {other:?}"),
    }
}

fn check_curved_face(view: FaceView<'_>, surface: &Surface, writing_it_down: f64) -> Result<()> {
    let named = name_of_surface(surface);

    for wire in view.wires() {
        for corner in wire.coedges().map(CoedgeView::start) {
            let point = PredicatePoint::from(corner.point().coordinates());
            let decision = side_of_surface(surface, point);
            if decision != Decision::Zero
                && not_provably_off(
                    surface,
                    corner.point(),
                    corner.approximation(),
                    writing_it_down,
                )
            {
                continue;
            }
            match decision {
                Decision::Zero => {}
                Decision::Pos | Decision::Neg => {
                    return Err(Invalid::VertexOffItsFaceSurface.into());
                }
                Decision::Undecidable { .. } => {
                    let mut records = ProvenanceLog::new();
                    records.push(Record::new(
                        named,
                        &[point],
                        Level::Exact,
                        decision,
                        Measures::Area,
                    ));
                    return Err(Error::Undecidable { records });
                }
                // `Decision` grows a variant at every milestone. Whatever the
                // new one means, guessing is worse than stopping.
                other => panic!("a corner came back with a decision nothing here knows: {other:?}"),
            }
        }
    }
    Ok(())
}

/// Whether a corner that is not *exactly* on its face's surface is nonetheless
/// not provably off it, once the slop recorded on it is taken into account.
///
/// **This is §17.2, and it is what a fused curved corner needs.** A corner of a
/// curved face is asked for a *sign* and never for a distance, because a ball
/// and a pipe have a value rather than a signed length — so there is nothing to
/// compare against an allowance the way a flat face's corner is. What there is
/// instead is the width the corner already carries: a corner a builder placed on
/// a sphere carries none, and a corner a fuse solved for carries the width of
/// the box it was solved as. Whether *that box* straddles the surface is the
/// honest question, and a box that does is a corner nobody has shown to be off.
///
/// Asked only after the exact point has failed, so **an exactly placed corner is
/// asked exactly what it always was** and nothing is loosened for one.
///
/// **A corner that came in off the wire carries no slop at all, and that is the
/// second width.** A file that names a curved face writes its corners as
/// decimal text, the reader turns each into the nearest `f64`, and the nearest
/// `f64` to a point on a ball or a pipe is not on it — `x² + y² − r²` worked out
/// exactly is not zero at any `f64` near a circle. So `writing_it_down` is
/// [`Model::what_writing_a_point_down_costs`], and it is the wider of the two
/// that is spent: the recorded slop is the box a fuse solved the corner as, and
/// a solve that landed inside one rounding is no more certain than one rounding.
///
/// Asked about a face's own surface by [`check_curved_face`] and about an edge's
/// partner surface by `check_edges`. The second was an exact `Decision::Zero`
/// until §39, on the argument that a corner a fuse solved for is *exactly* on a
/// plane it was solved from. It is not: the corner is `through + t·direction`
/// over a divisor and it rounds, so a block turned relative to a bar left the
/// fuse writing a solid its own sweep called broken.
fn not_provably_off(
    surface: &Surface,
    point: Point,
    slop: Option<Approximation>,
    writing_it_down: f64,
) -> bool {
    let recorded = slop.map_or(0.0, Approximation::magnitude);
    let reach = recorded.max(writing_it_down);
    if !(reach > 0.0 && reach.is_finite()) {
        return false;
    }
    let within = PredicatePoint::Constructed(point.coordinates().map(|coordinate| {
        Interval::new(coordinate - reach, coordinate + reach)
            .expect("a positive finite reach makes a box round a finite corner")
    }));
    matches!(
        side_of_surface(surface, within),
        Decision::Zero | Decision::Undecidable { .. }
    )
}

/// Whether the surface an edge is *named* by is a plane, and the corner is off
/// it by no more than writing that plane down in `f64` cost.
///
/// **The two standards this kernel had, made one.** `check_flat_face` allows a
/// corner one [`Model::allowed_wobble`] against the plane its face is *carried
/// by*; `check_edges` allowed a corner nothing at all against the plane an arc
/// is merely *named* by, which is the weaker claim of the two asked the harder
/// way round. An arc's plane is exactly where the mismatch bites: a circle's
/// plane is built through a point with a normal that gets scaled to length one,
/// so a corner exactly on the circle reads a nonzero distance to the stored
/// plane unless the numbers happen to cancel — which they do only for an
/// axis-aligned circle whose ends share the centre's coordinate along the axis.
/// Every other arc was refused for the rounding in its own name.
///
/// `a_plane_through_a_point_does_not_put_that_point_exactly_on_itself` is that
/// rounding, demonstrated on one plane and one point rather than argued.
///
/// **A plane and nothing else, because a plane is the only one with a distance
/// to compare.** A sphere, a pipe, a cone and a ring answer with a sign and
/// hand no signed length out, so the same allowance reaches them the other way
/// round: [`not_provably_off`] widens the corner into a box of
/// [`Model::what_writing_a_point_down_costs`] and asks which side the box is on.
/// Both budgets are the width of the number and neither is a tolerance; they
/// differ only in which end of the question they are spent at.
fn off_a_plane_by_no_more_than_writing_it_down_cost(
    surface: &Surface,
    point: Point,
    rounding: f64,
) -> bool {
    surface
        .as_plane()
        .is_some_and(|flat| flat.signed_distance_to(point).abs() <= rounding)
}

/// Which way `wire` winds round `normal`.
///
/// [`Decision::Pos`] with it, [`Decision::Neg`] against it, [`Decision::Zero`]
/// when the corners sweep out no area at all.
///
/// **Zero does not mean the face is unsound**, and this is the half of §44.11
/// item 3's winding question that stayed open. It means the *corners* enclose
/// nothing, which a boundary that bends between them can do while enclosing
/// plenty: a face bounded by an arc and its chord has two corners and sweeps
/// exactly zero, and so does any face whose corners land on one line —
/// **however many of them there are**. [`check_flat_face`] keeps the first out
/// of here by not asking about a boundary with fewer than three corners.
/// Collinear corners it still asks and still refuses. That is a **gap and not a
/// limit**: the answer exists and is written down — each arc's
/// [`EdgeRun`](rcad_geom::EdgeRun) carries the half-spaces that say which way it
/// bulges — and nothing that asks a wire which way it goes reads a run today.
/// Closing it means winding something other than the stored corners.
///
/// **The consumer is now named, and it is the importer.** This used to say "no
/// file in the corpus is refused for it, so it waits for one that is", and the
/// corpus still does not refuse a file for a winding — the KiCad and NIST files
/// stop earlier, on B-splines and on corners off their curved faces. What names
/// it instead is a file written by hand:
/// `rcad-step`'s `a_file_bounded_by_two_half_circles.rs` reads a whole AP203
/// `manifold_solid_brep` for a half-round plate with a half-round rib under it,
/// whose flat top and bottom are each bounded by two half-circles closed by two
/// straights, so their four corners are four points on the `x` axis. The STEP
/// mapping takes every part of that file — its own `Walk::area_swept_by` adds
/// each arc's bulge past its chord for exactly this shape — and this rule
/// refuses it through `SolidBuilder::build`. Two windings in one pipeline, one
/// face, two answers. `a_face_bounded_by_two_half_circles.rs` is the same solid
/// built face by face, and its tests are `#[ignore]`d on this refusal.
///
/// The old sentence also named the wrong shape: a boundary of two half-circles
/// has **two** corners, not three, so it never reached this at all.
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
    use rcad_geom::{
        Axis, Cylinder, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Sphere, Surface,
        Vector,
    };
    use rcad_predicates::{Approximation, Cause, Policy, ProvenanceLog};
    use smallvec::SmallVec;

    use super::{Decision, as_coarse_as_a_mesh_gets, winding_of, winding_round};
    use crate::key::{CoedgeSlot, EdgeSlot, FaceSlot, VertexSlot, WireSlot};
    use crate::measure::signed_volume_over;
    use crate::model::{ShellData, ShellUse, SolidData, VertexData, WireStep};
    use crate::{Error, FaceView, Invalid, Model, SolidView, WireView};

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

    fn a_round_bar() -> Model {
        let mut model = an_empty_model();
        model
            .add_round_bar(3.0, 5.0, &Isometry::identity())
            .expect("a bar three across and five tall");
        model
    }

    fn a_ball() -> Model {
        let mut model = an_empty_model();
        model
            .add_ball(2.0, &Isometry::identity())
            .expect("a ball of radius two");
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

    /// **The sweep itself crashed on a stranger's model, which is the one
    /// thing it exists not to do.**
    ///
    /// A fuzz target over `postcard::from_bytes::<Model>` built this shape
    /// after 17,827,556 executions: two solids, and nothing at all in the
    /// other six arenas. The sweep reached it at
    /// `check_which_side_of_its_trimming_each_curved_face_lies_on`, which walks
    /// every solid through views, and `ShellView::faces` indexed the shell
    /// arena with a slot that was not in it — `invalid SlotMap key used`,
    /// `view.rs:388`. The solid-names-a-shell check was already written; it
    /// just ran two checks too late.
    ///
    /// Built by hand rather than read from the fuzzer's bytes, because
    /// [the format is one we intend to break](crate::serialize) and a stored
    /// corpus in it stops reproducing the day the version moves.
    #[test]
    fn a_solid_naming_a_shell_that_is_not_there_is_refused_rather_than_crashing_the_sweep() {
        let mut model = an_empty_model();
        let nowhere = model
            .data_mut()
            .shells
            .insert(ShellData { faces: Vec::new() });
        model.data_mut().shells.remove(nowhere);
        model.data_mut().solids.insert(SolidData {
            shells: SmallVec::from_buf([ShellUse {
                shell: nowhere,
                loc: Isometry::identity(),
                orient: Orientation::Forward,
            }]),
        });

        assert_eq!(refusal(&model), Invalid::DanglingKey { entity: "shell" });
    }

    /// **Every level, not just the two the earlier sweep reached.**
    ///
    /// The check was written five times, once inside the check for whichever
    /// entity happened to store the key, and only the vertex and edge levels
    /// ran early enough to work. Measured on the code before this change,
    /// taking one entity of each kind out of a cube gave: vertex and edge
    /// refused by name; **coedge panicked** `invalid SlotMap key used` at
    /// `check_edges`, which reads the index's coedge list and indexes the
    /// coedge arena with it; wire, face and shell refused for
    /// [`Invalid::Orphan`] one level down, which is the symptom and not the
    /// fact — the model's own words for what is wrong with it are that a key
    /// names nothing.
    ///
    /// One entity of each kind is the whole family in one test.
    #[test]
    fn every_level_of_the_model_is_refused_for_naming_something_that_is_gone() {
        let gone = |entity| Invalid::DanglingKey { entity };

        let mut model = a_cube();
        let vertex = model.data().vertices.keys().next().expect("a corner");
        model.data_mut().vertices.remove(vertex);
        assert_eq!(refusal(&model), gone("vertex"), "an edge's end");

        let mut model = a_cube();
        let edge = model.data().edges.keys().next().expect("an edge");
        model.data_mut().edges.remove(edge);
        assert_eq!(refusal(&model), gone("edge"), "a coedge's edge");

        let mut model = a_cube();
        let coedge = model.data().coedges.keys().next().expect("a coedge");
        model.data_mut().coedges.remove(coedge);
        assert_eq!(refusal(&model), gone("coedge"), "a step round a wire");

        let mut model = a_cube();
        let wire = model.data().wires.keys().next().expect("a wire");
        model.data_mut().wires.remove(wire);
        assert_eq!(refusal(&model), gone("wire"), "a face's boundary");

        let mut model = a_cube();
        let face = model.data().faces.keys().next().expect("a face");
        model.data_mut().faces.remove(face);
        assert_eq!(refusal(&model), gone("face"), "a face a shell is made of");

        let mut model = a_cube();
        let shell = model.data().shells.keys().next().expect("a shell");
        model.data_mut().shells.remove(shell);
        assert_eq!(refusal(&model), gone("shell"), "a solid's skin");
    }

    /// The one step round a wire that names a corner instead of an edge.
    ///
    /// A pole cannot be reached by taking a corner away, because a ball's poles
    /// are its seam's two ends and the edge check names them first. So the slot
    /// is made dangling on purpose: put a corner in, take it out again, and
    /// point the pole at it.
    #[test]
    fn a_pole_standing_on_a_corner_that_is_gone_is_caught() {
        let mut model = a_ball();
        let nowhere = model.data_mut().vertices.insert(VertexData {
            point: Point::origin(),
            approximation: None,
        });
        model.data_mut().vertices.remove(nowhere);

        let mut poles = 0;
        for wire in model.data_mut().wires.values_mut() {
            for step in &mut wire.steps {
                if let WireStep::Pole(vertex) = step {
                    *vertex = nowhere;
                    poles += 1;
                }
            }
        }
        assert!(poles > 0, "a ball's boundary stands on its poles");

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
        model.data_mut().wires[wire].steps.swap(0, 1);

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

    /// The refusing half of the curved-face check.
    ///
    /// A face is no longer refused for being curved — `add_round_bar` builds
    /// one and the sweep accepts it, which is the other half of this and lives
    /// in `tests/a_solid_with_a_curved_face.rs` (§14.7 wants both halves). What
    /// is refused is a curved face whose corners are not on the surface, and a
    /// cube's corners hung on a unit ball are as far off as it gets.
    ///
    /// Note what does *not* appear: a distance. A sign predicate says which
    /// side and never how far, so there is no tolerance here to tune.
    /// An edge named as part of a crossing has to be on **both** surfaces of
    /// that crossing, not just the one its face sits on.
    ///
    /// Move the ball's seam plane so the poles are no longer in it, and the
    /// name now describes a meridian this edge does not run along. Nothing else
    /// in the kernel would notice: the face's own surface still certifies both
    /// poles, the wire still closes, and every count is right.
    #[test]
    fn an_edge_named_by_a_crossing_it_is_not_on_is_caught() {
        let mut model = an_empty_model();
        model
            .add_ball(2.0, &Isometry::identity())
            .expect("a ball of radius two");

        let edge = model.data().edges.keys().next().expect("the seam");
        let moved =
            Plane::from_point_and_normal(Point::new(0.0, 1.0, 0.0), Vector::new(0.0, 1.0, 0.0))
                .expect("the y axis is a unit normal");
        let EdgeRun::PartOfWhereItCrosses { and, .. } = &mut model.data_mut().edges[edge].run
        else {
            panic!("a ball's seam is named by a crossing")
        };
        *and = Surface::Plane(moved);

        assert_eq!(refusal(&model), Invalid::VertexOffItsFaceSurface);
    }

    /// The rounding the allowance above exists for, on one plane and one point.
    ///
    /// `(0, −10, −8)` dotted with `(−9, −4, 5)` is `40 − 40` and the point is
    /// exactly on the plane through the origin with that normal. The stored
    /// plane is not that plane: the normal is divided by its largest component
    /// and then scaled to length one, and neither division is exact, so the
    /// point reads a fifth of a coordinate spacing off a plane it is on.
    ///
    /// **Nothing anywhere can make this number zero**, which is why the
    /// allowance is not a tolerance. Every arc a real file names is one of
    /// these: its plane is written down as a centre and an axis, and its
    /// corners are other points.
    #[test]
    fn a_plane_through_a_point_does_not_put_that_point_exactly_on_itself() {
        let named_by_an_axis =
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, -10.0, -8.0))
                .expect("a direction with length");
        let on_it = Point::new(-9.0, -4.0, 5.0);

        let off_by = named_by_an_axis.signed_distance_to(on_it).abs();
        let spacing = on_it.largest_coordinate_magnitude() * f64::EPSILON;
        assert!(
            off_by > 0.0,
            "the whole point is that writing the plane down rounds"
        );
        assert!(
            off_by < spacing,
            "{off_by} should be under one coordinate spacing {spacing}"
        );
    }

    /// A corner off the plane its arc is *named* by, by no more than writing
    /// that plane down cost, is let through.
    ///
    /// The same allowance `check_flat_face` gives a corner against the plane its
    /// face is carried by, and it is here because a name is the *weaker* claim
    /// of the two and was being asked the harder way round.
    ///
    /// The ball's seam plane is nudged along its own normal, which is what a
    /// file that wrote the plane a few ulps off does, and the poles stay exactly
    /// on the sphere so nothing else in the sweep has an opinion.
    #[test]
    fn a_corner_off_the_plane_of_its_arc_by_the_rounding_in_that_plane_is_let_through() {
        let mut model = an_empty_model();
        model
            .add_ball(2.0, &Isometry::identity())
            .expect("a ball of radius two");
        let rounding = model.what_writing_a_plane_down_costs();

        nudge_the_seam_plane(&mut model, rounding / 2.0);

        a_clean_sweep(&model);
    }

    #[test]
    fn a_corner_further_off_the_plane_of_its_arc_than_that_is_caught() {
        let mut model = an_empty_model();
        model
            .add_ball(2.0, &Isometry::identity())
            .expect("a ball of radius two");
        let rounding = model.what_writing_a_plane_down_costs();

        nudge_the_seam_plane(&mut model, rounding * 2.0);

        assert_eq!(refusal(&model), Invalid::VertexOffItsFaceSurface);
    }

    /// The curved twin of the two above, and the reason 90 corpus files were
    /// refused.
    ///
    /// A file that says a corner is on a ball writes three decimal numbers and
    /// the reader takes the nearest `f64` to each. The nearest `f64` to a point
    /// on a ball is not on it — `x² + y² + z² − r²` worked out exactly is not
    /// zero there — so the corner is provably off a surface no writer could
    /// have put it nearer to. Sliding a pole along the axis it sits on is that
    /// miss, and it leaves the seam plane and every count alone.
    #[test]
    fn a_corner_off_its_curved_face_by_no_more_than_writing_it_down_cost_is_let_through() {
        let mut model = an_empty_model();
        model
            .add_ball(2.0, &Isometry::identity())
            .expect("a ball of radius two");
        let rounding = model.what_writing_a_point_down_costs();

        nudge_a_pole_off_the_ball(&mut model, rounding / 2.0);

        a_clean_sweep(&model);
    }

    /// And the half that keeps the allowance from being a tolerance.
    ///
    /// Twice the width of the number is a corner somebody chose, and it is
    /// still refused.
    #[test]
    fn a_corner_further_off_its_curved_face_than_that_is_caught() {
        let mut model = an_empty_model();
        model
            .add_ball(2.0, &Isometry::identity())
            .expect("a ball of radius two");
        let rounding = model.what_writing_a_point_down_costs();

        nudge_a_pole_off_the_ball(&mut model, rounding * 2.0);

        assert_eq!(refusal(&model), Invalid::VertexOffItsFaceSurface);
    }

    /// Slide a ball's north pole out along the axis it sits on.
    fn nudge_a_pole_off_the_ball(model: &mut Model, by: f64) {
        let pole = model
            .data()
            .vertices
            .iter()
            .find(|(_, vertex)| vertex.point.coordinates()[2] > 0.0)
            .map(|(slot, _)| slot)
            .expect("a ball has a north pole");
        let was = model.data().vertices[pole].point.coordinates();
        model.data_mut().vertices[pole].point = Point::new(was[0], was[1], was[2] + by);
    }

    /// Slide the plane a ball's seam is named by along its own normal.
    fn nudge_the_seam_plane(model: &mut Model, by: f64) {
        let edge = model.data().edges.keys().next().expect("the seam");
        let moved =
            Plane::from_point_and_normal(Point::new(0.0, by, 0.0), Vector::new(0.0, 1.0, 0.0))
                .expect("the y axis is a unit normal");
        let EdgeRun::PartOfWhereItCrosses { and, .. } = &mut model.data_mut().edges[edge].run
        else {
            panic!("a ball's seam is named by a crossing")
        };
        *and = Surface::Plane(moved);
    }

    /// And it has to be inside the half-spaces that picked which part it is.
    ///
    /// A plane crosses a sphere in one circle and the poles cut that circle
    /// into two arcs, so without this the name says "one of those two" and
    /// stops. Turn the half-space right round and the seam is on the wrong
    /// side of it.
    #[test]
    fn an_edge_outside_the_half_space_that_picked_it_is_caught() {
        let mut model = an_empty_model();
        model
            .add_ball(2.0, &Isometry::identity())
            .expect("a ball of radius two");

        let edge = model.data().edges.keys().next().expect("the seam");
        let elsewhere = rcad_geom::HalfSpace::through_a_point_facing(
            Point::new(5.0, 0.0, 0.0),
            Vector::new(1.0, 0.0, 0.0),
        )
        .expect("the x axis has length");
        let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = &mut model.data_mut().edges[edge].run
        else {
            panic!("a ball's seam is named by a crossing")
        };
        picked_by[0] = rcad_geom::Picker::from(elsewhere);

        assert_eq!(refusal(&model), Invalid::EdgeOutsideWhatPickedIt);
    }

    /// The same check, asked about a curved half-space, which is what §24.5
    /// widened the field to hold.
    ///
    /// A ball's seam is picked out by *outside a pipe standing beside it*, which
    /// swallows the `+x` half of the meridian and leaves the `-x` half. Both
    /// poles are well clear of that pipe, so the model is sound; grow a ball
    /// about the origin past them instead and the name picks a piece the seam is
    /// not on. Nothing but this check would notice, and while `picked_by` held
    /// planes the question could not be asked at all.
    #[test]
    fn an_edge_outside_a_curved_half_space_that_picked_it_is_caught() {
        let beside_the_ball = Axis::new(Point::new(3.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
            .expect("a direction with length");
        let sound = rcad_geom::Picker::from(
            Cylinder::new(beside_the_ball, 2.0).expect("a pipe of radius two"),
        );
        let swallows_the_poles = rcad_geom::Picker::from(
            Sphere::new(Point::origin(), 5.0).expect("a ball of radius five"),
        );

        for (picker, expected) in [
            (sound, None),
            (swallows_the_poles, Some(Invalid::EdgeOutsideWhatPickedIt)),
        ] {
            let mut model = an_empty_model();
            model
                .add_ball(2.0, &Isometry::identity())
                .expect("a ball of radius two");

            let edge = model.data().edges.keys().next().expect("the seam");
            let EdgeRun::PartOfWhereItCrosses { picked_by, .. } =
                &mut model.data_mut().edges[edge].run
            else {
                panic!("a ball's seam is named by a crossing")
            };
            picked_by[0] = picker;

            match expected {
                None => model
                    .validate(&Policy::certified(), &mut ProvenanceLog::new())
                    .expect("both poles are well outside the pipe beside the ball"),
                Some(invalid) => assert_eq!(refusal(&model), invalid),
            }
        }
    }

    /// A curved face standing on both sides of something that trims it is
    /// caught.
    ///
    /// **The curved twin of `a_face_whose_outer_wire_winds_against_its_normal`**
    /// and the only part of "the material is on the left" that can be asked
    /// while a closed edge has no direction rule (§30.8 item 10).
    ///
    /// One cylinder and three flat faces cutting it at three heights, with all
    /// three circles claimed by the one curved face. Every corner of it is
    /// certified onto the cylinder and every corner of each flat face is on its
    /// own plane, so nothing before this notices; what is wrong is that the
    /// middle plane has a corner of the curved face on each side of it, and a
    /// crossing that goes right round leaves two pieces and not three. **The
    /// route is `Deserialize`**: no builder makes this and the fuse will not,
    /// but the arenas hold it and `validate` is the untrusted-import boundary
    /// (§16.11).
    #[test]
    fn a_curved_face_standing_on_both_sides_of_something_that_trims_it_is_caught() {
        let mut model = an_empty_model();
        let radius = 3.0;
        let pipe = Cylinder::new(rcad_geom::Axis::z(), radius).expect("a pipe");

        let mut round_it: SmallVec<[WireSlot; 1]> = SmallVec::new();
        let mut caps = Vec::new();
        for height in [-1.0, 0.0, 1.0] {
            let corner = model.add_vertex(Point::new(radius, 0.0, height));
            let circle = model.add_closed_edge(corner);
            let mine = model.add_coedge(circle, Orientation::Forward);
            round_it.push(model.add_wire(smallvec::smallvec![mine]));

            let flat = Plane::from_point_and_normal(
                Point::new(0.0, 0.0, height),
                Vector::new(0.0, 0.0, 1.0),
            )
            .expect("the z axis is a unit normal");
            let theirs = model.add_coedge(circle, Orientation::Reversed);
            let wire = model.add_wire(smallvec::smallvec![theirs]);
            caps.push(model.add_face(Surface::Plane(flat), SmallVec::from_buf([wire])));
        }
        let wall = model.add_face(Surface::Cylinder(pipe), round_it);

        let shell = model.add_shell(
            caps.into_iter()
                .chain(core::iter::once(wall))
                .map(|face| crate::model::FaceUse {
                    face,
                    loc: Isometry::identity(),
                    orient: Orientation::Forward,
                })
                .collect(),
        );
        model.add_solid(SmallVec::from_buf([crate::model::ShellUse {
            shell,
            loc: Isometry::identity(),
            orient: Orientation::Forward,
        }]));

        assert!(
            matches!(refusal(&model), Invalid::AFaceStraddlesSomethingTrimmingIt),
            "got {:?}",
            refusal(&model)
        );
    }

    #[test]
    fn a_corner_that_is_not_on_its_faces_curved_surface_is_caught() {
        let mut model = a_cube();
        let ball = Sphere::new(Point::origin(), 1.0).expect("a ball of radius one");
        model
            .data_mut()
            .faces
            .values_mut()
            .next()
            .expect("a cube has faces")
            .surface = Surface::Sphere(ball);

        assert_eq!(refusal(&model), Invalid::VertexOffItsFaceSurface);
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

    /// §23.9's second gap, closed. A curved solid has no fan, so nothing used to
    /// say a bar or a ball pointed outwards at all — the check was skipped
    /// rather than run against a number about a shape the solid is not.
    ///
    /// Turned inside out, both now come back with a negative meshed volume, and
    /// the control is that the sweep accepts them the right way round.
    ///
    /// **The two are caught by different things now**, and which one is which is
    /// worth the table. A bar turned inside out walks both its cap circles
    /// against their surfaces and the direction rule refuses it before anything
    /// is meshed. A ball has **no closed edge at all** — its one seam runs pole
    /// to pole — so the rule has nothing to say about it and the volume is still
    /// what catches it.
    ///
    /// §23.9's own claim is asserted for both all the same, by measuring the
    /// meshed volume here rather than waiting for the sweep to reach it.
    #[test]
    fn a_curved_solid_turned_inside_out_is_caught() {
        for (build, has_a_closed_edge) in [
            (a_round_bar as fn() -> Model, true),
            (a_ball as fn() -> Model, false),
        ] {
            let mut model = build();
            a_clean_sweep(&model);

            for shell in model.data_mut().shells.values_mut() {
                for used in &mut shell.faces {
                    used.orient = used.orient.reversed();
                }
            }
            let refused = refusal(&model);
            if has_a_closed_edge {
                assert_eq!(refused, Invalid::AClosedEdgeIsWalkedAgainstItsSurface);
            } else {
                assert!(matches!(refused, Invalid::VolumeIsNotPositive { .. }));
            }

            let solid = model.data().solids.keys().next().expect("one solid");
            let view = SolidView::new(&model, solid, Isometry::identity(), Orientation::Forward);
            let mesh = view
                .mesh(
                    as_coarse_as_a_mesh_gets(),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("an inside-out solid still meshes");
            assert!(
                signed_volume_over(&mesh) < 0.0,
                "a curved solid turned inside out holds a negative volume"
            );
        }
    }

    /// §30.8 item 10 stopped being a rule and became a check.
    ///
    /// A closed edge starts and ends at the one corner it has, so nothing in the
    /// model says which way it runs. Walk a bar's bottom cap circle the other way
    /// round — **both** of its coedges, so the two still run opposite ways and
    /// `CoedgesRunTheSameWay` has nothing to say — and every other check passes:
    /// the wire closes, both corners are on both surfaces, the mesh is watertight
    /// and the volume is right. Nothing but this notices, and §36.22 said as
    /// much while it was still a rule.
    ///
    /// The control is the row that flips **one** coedge, which is caught by the
    /// older check and says this one is not doing that job over again.
    #[test]
    fn a_closed_edge_walked_the_wrong_way_round_its_surface_is_caught() {
        for (both, expected) in [
            (true, Invalid::AClosedEdgeIsWalkedAgainstItsSurface),
            (false, Invalid::CoedgesRunTheSameWay),
        ] {
            let mut model = a_round_bar();
            a_clean_sweep(&model);

            let circle = model
                .data()
                .edges
                .iter()
                .find(|(_, edge)| edge.run == EdgeRun::RightRoundWhereItsFacesCross)
                .map(|(slot, _)| slot)
                .expect("a bar's cap is a whole circle");
            let coedges: Vec<CoedgeSlot> = model.index().coedges_of_edge(circle).to_vec();
            let turning = if both { &coedges[..] } else { &coedges[..1] };
            for &coedge in turning {
                let orientation = model.data().coedges[coedge].orientation.reversed();
                model.data_mut().coedges[coedge].orientation = orientation;
            }

            assert_eq!(refusal(&model), expected);
        }
    }

    /// Whether the coedge, walked the way its wire uses it, starts at `vertex`.
    fn coedge_starts_at(model: &Model, coedge: CoedgeSlot, vertex: VertexSlot) -> bool {
        let data = model.data();
        let edge = &data.edges[data.coedges[coedge].edge];
        let start = match data.coedges[coedge].orientation {
            Orientation::Forward => edge.start,
            Orientation::Reversed => edge.end,
        };
        start == vertex
    }

    /// `docs/decisions.md` §39.7 item 3's first demonstrated panic: a cube
    /// with one extra closed edge spliced into two of its wires, so a
    /// `RightRoundWhereItsFacesCross` edge sits between two flat faces —
    /// which no builder in this crate makes, and nothing stops `Model`'s
    /// `Deserialize` from holding it anyway.
    ///
    /// Two planes cross in a straight line, which never closes, so
    /// `a_closed_crossing_is_walked_the_way_it_is_stored` used to reach for
    /// the curved surface of the pair with `.expect(...)` and find neither —
    /// a panic on exactly the kind of bytes `Model::validate` exists to
    /// refuse instead.
    #[test]
    fn a_closed_edge_between_two_flat_faces_is_caught() {
        let mut model = a_cube();
        let corner = model
            .data()
            .vertices
            .keys()
            .next()
            .expect("a cube has corners");

        let closed_edge = model.add_closed_edge(corner);
        let forward = model.add_coedge(closed_edge, Orientation::Forward);
        let backward = model.add_coedge(closed_edge, Orientation::Reversed);

        let wires_through_corner: Vec<WireSlot> = model
            .data()
            .wires
            .keys()
            .filter(|&wire| {
                model.data().wires[wire]
                    .coedges()
                    .any(|coedge| coedge_starts_at(&model, coedge, corner))
            })
            .collect();
        assert!(
            wires_through_corner.len() >= 2,
            "a cube corner sits on at least two faces' wires, and this has {}",
            wires_through_corner.len()
        );

        for (&wire, new_coedge) in wires_through_corner.iter().zip([forward, backward]) {
            let position = model.data().wires[wire]
                .coedges()
                .position(|coedge| coedge_starts_at(&model, coedge, corner))
                .expect("just filtered the wires for this");
            model.data_mut().wires[wire]
                .steps
                .insert(position, crate::model::WireStep::Along(new_coedge));
        }
        // `data_mut` only touches the downward links; the reverse index that
        // `faces_along_edge` reads is derived and stays stale until rebuilt.
        let model = Model::from_parts(model.data().clone());

        assert_eq!(refusal(&model), Invalid::ClosedEdgeBetweenTwoFlatFaces);
    }

    /// §23.9's first gap, closed. A round cap's wire has one corner and no
    /// polygon, so `winding_of` has nothing to ask and `check_flat_face` skips
    /// it; reversing the cap's plane turns the cap round and used to leave every
    /// other check happy.
    ///
    /// Note what does *not* catch it. The corners are still on the plane, since
    /// reversing one only flips the sign of a distance this reads the size of.
    /// And the volume is still positive: hung from a corner of the bottom rim,
    /// the bottom cap contributes nothing whichever way it faces and flipping
    /// the top one leaves a third of `πr²h`. What catches it is the mesh, where
    /// the cap's triangles now run round the shared circle the same way the
    /// wall's do.
    #[test]
    fn a_round_cap_put_on_back_to_front_is_caught() {
        let mut model = a_round_bar();
        a_clean_sweep(&model);

        let cap = model
            .data()
            .faces
            .iter()
            .find(|(_, face)| face.surface.as_plane().is_some())
            .map(|(slot, _)| slot)
            .expect("a bar has two flat caps");
        let plane = *model.data().faces[cap]
            .surface
            .as_plane()
            .expect("just found it");
        model.data_mut().faces[cap].surface = Surface::Plane(plane.reversed());

        assert_eq!(refusal(&model), Invalid::FacesDoNotAgreeWhichWayTheyFace);
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

    /// Two steps that go out and come back along the *same* edge retrace the
    /// same curve, so they bound nothing whatever that curve is.
    ///
    /// §44.11 item 3 let two-step wires in, and this is one of the two shapes
    /// it did not let in. `a_face_bounded_by_two_steps.rs` asks the builder the
    /// same question; this is the sweep's own copy of the rule, which a model
    /// off the wire meets instead.
    #[test]
    fn a_wire_of_two_coedges_out_and_back_along_one_edge_is_caught() {
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

    /// The other shape §44.11 item 3 did not let in: two *different* straight
    /// edges between the same two corners, which are the same segment written
    /// twice.
    #[test]
    fn a_wire_of_two_straight_coedges_between_the_same_corners_is_caught() {
        let mut model = an_empty_model();
        let start = model.add_vertex(Point::new(0.0, 0.0, 0.0));
        let end = model.add_vertex(Point::new(1.0, 0.0, 0.0));
        let one = model.add_edge(start, end);
        let other = model.add_edge(start, end);
        a_wire_and_its_mirror(&mut model, one, other);

        assert_eq!(refusal(&model), Invalid::WireTooShort { count: 2 });
    }

    /// And the shape it did: two different edges, one of them an arc.
    ///
    /// The sweep goes on to refuse this scrap for what it really is — a face in
    /// no shell — and the point is that the *length* rule is no longer what
    /// stops it. `a_face_bounded_by_two_steps.rs` builds the whole solid.
    #[test]
    fn a_wire_of_an_arc_and_its_chord_is_long_enough_for_the_sweep() {
        let mut model = an_empty_model();
        let start = model.add_vertex(Point::new(1.0, 0.0, 0.0));
        let end = model.add_vertex(Point::new(-1.0, 0.0, 0.0));
        let arc = model.add_edge_running(
            start,
            end,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(the_ground()),
                picked_by: SmallVec::from_buf([rcad_geom::Picker::from(
                    rcad_geom::HalfSpace::through_a_point_facing(
                        Point::origin(),
                        Vector::new(0.0, 1.0, 0.0),
                    )
                    .expect("the y axis has length"),
                )]),
            },
        );
        let chord = model.add_edge(start, end);
        a_wire_and_its_mirror(&mut model, arc, chord);

        assert_eq!(refusal(&model), Invalid::Orphan { entity: "face" });
    }

    /// Two edges as a two-step wire and the same two run the other way round,
    /// on one face.
    ///
    /// The mirror is what gives each edge the two coedges running opposite ways
    /// the sweep insists on, so a two-step wire reaches the length rule instead
    /// of stopping at [`Invalid::EdgeCoedgeCount`] before it.
    fn a_wire_and_its_mirror(model: &mut Model, one: EdgeSlot, other: EdgeSlot) {
        let there = model.add_coedge(one, Orientation::Forward);
        let back = model.add_coedge(other, Orientation::Reversed);
        let outer = model.add_wire([there, back].into_iter().collect());
        let there = model.add_coedge(other, Orientation::Forward);
        let back = model.add_coedge(one, Orientation::Reversed);
        let hole = model.add_wire([there, back].into_iter().collect());
        model.add_face(
            Surface::Plane(the_ground()),
            [outer, hole].into_iter().collect(),
        );
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
