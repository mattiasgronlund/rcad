use std::collections::HashMap;

use rcad_geom::{
    Cone, Deflection, DynamicRange, EdgeRun, HalfSpace, Isometry, NEAREST_F64_POINT_ROUNDOFF,
    NamedRun, Orientation, Picker, Plane, PlaneAgreement, Point, SIDE_OF_A_HALF_SPACE,
    SIDE_OF_CONE, SIDE_OF_CYLINDER, SIDE_OF_PLANE, SIDE_OF_SPHERE, SIDE_OF_TORUS, Scale, Surface,
    Vector, how_a_plane_leans_against_a_cone, where_a_cones_apex_stands_against_a_plane,
    which_two_surfaces_a_run_is_between, which_way_a_closed_crossing_turns,
};
use rcad_predicates::expansion::{
    expansion_sign, expansion_sum, product_is_exact, scale_expansion, two_product,
};
use rcad_predicates::{
    Approximation, Decision, Filtered, Interval, Level, Measures, NOTHING_IS_KNOWN, ORIENT2D,
    Point as PredicatePoint, Policy, Predicate, ProvenanceLog, Record, SideOfSurface,
    are_real_numbers, decide,
};
use slotmap::SecondaryMap;

use crate::builder::the_ends_are_the_whole_of_it;
use crate::error::walking_along_this_edge_refused;
use crate::fuse::name_of_surface_bits;
use crate::holds::{
    Trimming, how_the_axis_lies_on, lie_along_one_line, the_face_across,
    the_picker_naming_the_circle_through, the_side_this_face_lies_on, what_trims_this_face,
};
use crate::key::{CoedgeSlot, EdgeSlot, FaceSlot, ShellSlot, SolidSlot, VertexSlot, WireSlot};
use crate::measure::signed_volume_over_and_how_far_it_rounded;
use crate::mesh::Mesh;
use crate::model::{ModelData, WireData, WireStep};
use crate::view::{CoedgeView, FaceView, SolidView, WireStepView, WireView};
use crate::{EdgeView, Error, FaceKey, Invalid, Model, Result, SolidKey};

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
    ///   refusing ([`check_keys_resolve_in`]),
    /// - no coordinate is infinite and no edge has zero length,
    /// - every edge is used by exactly two coedges running opposite ways, both
    ///   from the one shell,
    /// - every wire closes back on itself and has enough steps to bound
    ///   something: three, or one edge that closes already, or two different
    ///   edges that are not both straight,
    /// - every face sits on a plane, with its corners on that plane, its outer
    ///   wire winding with the face normal and its holes winding against it,
    ///   so the material is on one consistent side — over the boundary the wire
    ///   traces, arcs walked and all, and not over its corners alone
    ///   ([`winding_of`]),
    /// - nothing is orphaned and nothing is used twice where it may only be
    ///   used once,
    /// - no face sitting on a plane is used reversed — a plane that faces the
    ///   other way is stored turned round instead — which is what says a wall
    ///   turned round does not tear the skin,
    /// - no shell places a face anywhere but where that face is stored, which is
    ///   what says the two faces along an edge agree where that edge is,
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
    ///
    /// [`Model::validate_solid`] asks all of this of one solid instead, and runs
    /// the same code to do it.
    pub fn validate(&self, policy: &Policy, log: &mut ProvenanceLog) -> Result<()> {
        let everything = Scope::everything(self.data());
        self.check_everything_but_the_coordinate_range_in(&everything, policy, log)?;
        self.check_dynamic_range(&everything, policy, log)
    }

    /// Check every invariant [`Model::validate`] checks, over one solid and what
    /// it is built from — its shells, faces, wires, coedges and poles, edges and
    /// corners — and over nothing else in the model.
    ///
    /// **It answers what [`Model::validate`] answers for a model holding this
    /// solid alone.** That is how this kernel already judges a solid when it is
    /// made: [`SolidBuilder::build`](crate::SolidBuilder::build) sweeps a scratch
    /// model holding only the new solid, and a fuse sweeps a throwaway model
    /// holding only its answer. So a solid is asked here the question it was
    /// asked when it came into being, and a broken solid somewhere else in the
    /// model does not change the answer.
    ///
    /// **It is not a second validator.** Every check takes the set of entities
    /// it is to look at, and `validate` hands over every entity in the model
    /// where this hands over what the solid reaches. Nothing is checked here
    /// that `validate` does not check, and nothing is checked differently.
    ///
    /// # What "alone" means for the checks that read the whole model
    ///
    /// - **The rounding room is this solid's own.** How far a corner may sit off
    ///   the plane of its face, or off its curved surface, grows with the
    ///   largest coordinate in the model. Here that is the largest coordinate
    ///   among this solid's own corners, read where they are stored or where the
    ///   solid's placement puts them, whichever is smaller
    ///   ([`Model::largest_coordinate_in`]). Keeping the whole model's would let
    ///   a solid a long way off widen the room a small cube's corners get, and
    ///   accept a corner off its plane because of a neighbour. Accepting a
    ///   broken solid is the worse of the two ways to be wrong.
    /// - **The coordinate range is this solid's own**, largest coordinate
    ///   against smallest feature, both read off this solid.
    /// - **Sharing is still counted over the whole model.** A coedge, wire, face
    ///   or shell this solid reaches that some other owner names too is refused
    ///   as [`Invalid::UsedMoreThanOnce`], and an edge walked by a coedge of
    ///   another solid is counted with it and refused as
    ///   [`Invalid::AnEdgeIsWalkedOnceFromEachOfTwoShells`]. Ignoring the other
    ///   owner would pass a solid that a change to its neighbour silently
    ///   changes.
    /// - **Orphans are not asked about.** Everything this looks at is reached
    ///   through what owns it, so none of it can be an orphan. A coedge, wire,
    ///   face or shell that nothing owns belongs to no solid, and whether a model
    ///   holds one is a question about the whole model, which `validate` asks.
    ///
    /// # Neither answer says what the other one will be
    ///
    /// - **`validate` can pass a model this refuses one of its solids in.** A
    ///   solid far out makes the rounding room `validate` gives every corner
    ///   wider, so a small cube's corner a little off its plane passes there and
    ///   is refused here
    ///   (`a_corner_the_far_solid_makes_room_for_is_refused_on_its_own`).
    /// - **`validate` can refuse a model whose every solid this passes.** The
    ///   coordinate range across two solids is wider than either one's own: a
    ///   solid with a tiny step near the origin beside a block far out is too
    ///   wide to compute in as a model, and each is fine
    ///   (`two_solids_each_fine_on_its_own_can_be_too_far_apart_together`).
    ///
    /// # Errors
    ///
    /// [`Error::KeyFromAnotherModel`] if `solid` was made by a different model,
    /// and otherwise whatever [`Model::validate`] does, asked of this solid
    /// alone — less [`Invalid::Orphan`] for a coedge, wire, face or shell.
    ///
    /// # Panics
    ///
    /// If the key is this model's own and names nothing, the same way and for
    /// the same reason as [`Model::solid`].
    pub fn validate_solid(
        &self,
        solid: SolidKey,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        let slot = solid.try_slot_in(self.id())?;
        assert!(self.data().solids.contains_key(slot), "no such solid");
        let reached = Scope::what_this_solid_reaches(self.data(), slot);
        self.check_everything_but_the_coordinate_range_in(&reached, policy, log)?;
        self.check_dynamic_range(&reached, policy, log)
    }

    /// Twelve of the sweep's thirteen checks: every one that is about an entity,
    /// or about two entities that touch, rather than about the model as a whole.
    ///
    /// **The one left out is the coordinate range**, which is the only check
    /// here that folds over everything in the model at once and so is the only
    /// one whose answer depends on what else is in the model.
    ///
    /// `Assembly::commit` sweeps its throwaway
    /// model with this rather than with [`Model::validate`]. The throwaway holds
    /// the fuse's answer and nothing else, so a whole-model sweep of it is local
    /// to what the fuse wrote — but for the range that cuts both ways: a
    /// whole-model fold over a model holding only the answer *is* the answer's
    /// own range, and asking it there asks a question the fuse has already
    /// decided not to ask. `Assembly::check_dynamic_range` asks it behind a
    /// guard, so that a caller whose own solids are further apart than `f64` can
    /// resolve still gets the answer that arithmetic reached exactly
    /// (`docs/decisions.md` §87.3).
    ///
    /// # Errors
    ///
    /// Whatever [`Model::validate`] does, less
    /// [`Invalid::DynamicRangeTooWide`] and less the [`Error::Geometry`] of a
    /// range that could not be measured.
    pub(crate) fn check_everything_but_the_coordinate_range(
        &self,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        self.check_everything_but_the_coordinate_range_in(
            &Scope::everything(self.data()),
            policy,
            log,
        )
    }

    /// The twelve checks, asked of what `scope` holds.
    ///
    /// Each check walks the scope's arenas in slot order, the order `validate`
    /// has always walked the model's, so when the one solid a scope holds is the
    /// only broken thing in a model, the first refusal it meets is the one
    /// `validate` meets.
    fn check_everything_but_the_coordinate_range_in(
        &self,
        scope: &Scope,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        check_keys_resolve_in(self.data(), scope)?;
        self.check_vertices(scope)?;
        self.check_edges(scope)?;
        self.check_coedges(scope)?;
        self.check_wires(scope)?;
        self.check_faces(scope, policy, log)?;
        self.check_which_side_of_its_trimming_each_curved_face_lies_on(scope)?;
        self.check_closed_edges_are_walked_the_way_their_surfaces_wind(scope, policy, log)?;
        self.check_no_flat_face_is_used_reversed(scope)?;
        self.check_no_face_use_carries_a_placement(scope)?;
        self.check_shells_and_solids(scope, policy, log)?;
        self.check_approximation_ordering(scope)
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
    /// **[`Invalid::AFlatFaceIsUsedReversed`] and
    /// [`Invalid::AFaceUseCarriesAPlacement`] are the rule's next two members**,
    /// and they qualify for the same three reasons: one pass over the face uses,
    /// an `Orientation` or an `Isometry` compared against a constant, and
    /// nothing about a pair. Both are ways for a shell to describe a skin that
    /// does not close, neither is a solid at any strictness, and what is
    /// downstream of both is a panic rather than a wrong answer — `Model::fuse`
    /// walks the skin and trips `Model::check_touched_edge`, in every build
    /// since §66.3, which is exactly what "a public method is entitled to take
    /// this for granted" means.
    ///
    /// # Errors
    ///
    /// [`Invalid::VertexNotFinite`], [`Invalid::FaceHasNoWires`],
    /// [`Invalid::WireTooShort`], [`Invalid::WireDoesNotClose`],
    /// [`Invalid::EdgeCoedgeCount`], [`Invalid::CoedgesRunTheSameWay`],
    /// [`Invalid::AnEdgeIsWalkedOnceFromEachOfTwoShells`],
    /// [`Invalid::AFlatFaceIsUsedReversed`] or
    /// [`Invalid::AFaceUseCarriesAPlacement`], naming the first one that does
    /// not hold.
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
        let everything = Scope::everything(self.data());
        self.check_no_flat_face_is_used_reversed(&everything)?;
        self.check_no_face_use_carries_a_placement(&everything)
    }

    fn check_vertices(&self, scope: &Scope) -> Result<()> {
        for slot in scope.vertices.keys() {
            self.check_vertex(slot)?;
        }
        Ok(())
    }

    fn check_vertex(&self, slot: VertexSlot) -> Result<()> {
        if !self.data().vertices[slot].point.is_finite() {
            return Err(Invalid::VertexNotFinite.into());
        }
        if self.index().edges_of_vertex(slot).is_empty() {
            return Err(Invalid::Orphan { entity: "vertex" }.into());
        }
        Ok(())
    }

    fn check_edges(&self, scope: &Scope) -> Result<()> {
        let largest_coordinate = self.largest_coordinate_in(scope);
        let rounding = self.what_writing_a_plane_down_costs(largest_coordinate);
        let writing_a_point_down = self.what_writing_a_point_down_costs(largest_coordinate);
        for slot in scope.edges.keys() {
            self.check_edge(slot, rounding, writing_a_point_down)?;
        }
        Ok(())
    }

    /// One edge: it has some length, two coedges walk it opposite ways, and if
    /// it is named by a crossing its ends really are on that crossing.
    ///
    /// `rounding` is [`Model::what_writing_a_plane_down_costs`] and
    /// `writing_a_point_down` is [`Model::what_writing_a_point_down_costs`],
    /// handed in because both fold over every vertex in the scope and neither
    /// changes between edges.
    fn check_edge(&self, slot: EdgeSlot, rounding: f64, writing_a_point_down: f64) -> Result<()> {
        let edge = &self.data().edges[slot];
        let (Some(start), Some(end)) = (
            self.data().vertices.get(edge.start),
            self.data().vertices.get(edge.end),
        ) else {
            return Err(Invalid::DanglingKey { entity: "vertex" }.into());
        };
        // A closed edge is *supposed* to come back to where it started, so
        // its two ends being one vertex says it is a whole circle and not
        // that it has no length. What says which is the run, and the list of
        // runs that put an edge somewhere its ends do not is
        // `the_ends_are_the_whole_of_it` — the same list `SolidBuilder`'s
        // door reads, because a builder writes its description into a
        // scratch model and runs this sweep over it, so two lists would mean
        // one door refusing what the other keeps. This used to ask
        // `run == Straight`, which read a kind it did not know as *not
        // straight* and skipped the check.
        let both_ends_in_one_place = edge.start == edge.end || start.point == end.point;
        if both_ends_in_one_place && the_ends_are_the_whole_of_it(&edge.run) {
            return Err(Invalid::EdgeHasNoLength.into());
        }

        self.check_edge_is_walked_by_two_opposite_coedges(slot)?;
        self.check_something_curved_says_where_this_edge_bends(slot)?;
        self.check_flat_faces_put_an_edge_in_one_place(slot, [start.point, end.point], rounding)?;

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
        //
        // **Both kinds of run that name a second surface are asked**, and they
        // are asked the same thing. A closed rim carries an `and` and no
        // pickers — it is the whole of the curve rather than a piece of one, so
        // there is nothing that had to pick it — and this used to read only the
        // piece kind, which left a rim's named surface stored and never looked
        // at. Sound for every rim the fuse writes, whose two ends are the one
        // corner of a closed curve and on both surfaces of the crossing by
        // construction, and unsound for an imported or hand-written solid,
        // which can name any plane there at all. `docs/decisions.md` §217.5.
        //
        // A kind of run this crate has never seen names nothing it can read, so
        // there is nothing to ask; `check_closed_edges_are_walked_the_way_their
        // _surfaces_wind` is what refuses it, under
        // `ACrossingThisCannotBeOrderedAlong`.
        let names_a_second_surface = match &edge.run {
            EdgeRun::PartOfWhereItCrosses { and, picked_by } => Some((and, picked_by.as_slice())),
            EdgeRun::RightRoundWhereItCrosses { and } => Some((and, [].as_slice())),
            EdgeRun::Straight | EdgeRun::RightRoundWhereItsFacesCross => None,
            _ => None,
        };
        if let Some((and, picked_by)) = names_a_second_surface {
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
                    Decision::Undecidable { enclosure } => {
                        let mut records = ProvenanceLog::new();
                        records.push(Record::new(
                            name_of_surface(and),
                            &[at],
                            Level::Exact,
                            enclosure,
                            Measures::Area,
                        ));
                        return Err(Error::Undecidable { records });
                    }
                    other => {
                        panic!("a corner came back with a decision nothing here knows: {other:?}")
                    }
                }
                if picked_by
                    .iter()
                    .any(|picker| picker.side_of_point(at) == Decision::Neg)
                {
                    return Err(Invalid::EdgeOutsideWhatPickedIt.into());
                }
            }
        }
        Ok(())
    }

    /// An edge that bends has a curved face along it, or a run naming a curved
    /// surface, to say where it goes.
    ///
    /// [`nothing_curved_says_where_it_bends`] is the rule, and it is the one
    /// [`SolidView::mesh`] refuses by: a solid this passes, the mesher can
    /// place every edge of.
    ///
    /// An edge with no face found along it is left alone: that is a coedge no
    /// wire holds, which `check_coedges` refuses under its own name.
    ///
    /// Shared with [`Model::check_touched_edge`] for the reason
    /// [`Model::check_wire_closes`] gives. It is a few index lookups per edge, so
    /// it stays local to what an operation touched.
    ///
    /// # Errors
    ///
    /// [`Invalid::ClosedEdgeBetweenTwoFlatFaces`] for either kind of closed
    /// edge, which has that name already, and
    /// [`Invalid::AnEdgeBendsButNothingAlongItIsCurved`] for any other. A
    /// closed run carrying a flat `and` — a closed `ELLIPSE` drawn on a flat
    /// face, which is §219.6's limit — used to take the second name, the one
    /// meant for open edges.
    pub(crate) fn check_something_curved_says_where_this_edge_bends(
        &self,
        slot: EdgeSlot,
    ) -> Result<()> {
        let run = &self.data().edges[slot].run;
        let mut faces = self.faces_along_edge(slot).peekable();
        if faces.peek().is_none() {
            return Ok(());
        }
        let surfaces = faces.map(|face| self.data().faces[face].surface);
        if !nothing_curved_says_where_it_bends(run, surfaces) {
            return Ok(());
        }
        Err(match run {
            EdgeRun::RightRoundWhereItsFacesCross | EdgeRun::RightRoundWhereItCrosses { .. } => {
                Invalid::ClosedEdgeBetweenTwoFlatFaces
            }
            EdgeRun::Straight | EdgeRun::PartOfWhereItCrosses { .. } | _ => {
                Invalid::AnEdgeBendsButNothingAlongItIsCurved
            }
        }
        .into())
    }

    /// An edge whose every face is flat, and whose run names a curved surface
    /// to say where it goes, is put in one place by all of those faces.
    ///
    /// The rule above has refused a bent edge between flat faces whose run
    /// names nothing curved, so this is every such edge that is not straight.
    /// Its curve is where the named surface crosses a face's plane, and which
    /// face's plane was the first face's (Q386, Q574). A curve that lies in two
    /// planes lies on the line they cross in, so the curve one face puts the
    /// edge on leaves every other face's plane unless it is that line — and
    /// then which face the solid listed first changes the shape. That holds
    /// for a closed edge and for an open arc alike: a bar whose lid is split by
    /// a half disc tilted along its chord measured `113.088` with the lid
    /// listed first and `113.488` with the half disc first (Q604).
    ///
    /// **One plane spelt two ways is one plane.** [`Plane::agreement_with`]
    /// answers the exact question first, which takes a plane turned round or
    /// scaled. Two planes it proves different may still be one plane a file
    /// wrote twice a hair apart — a `PLANE` of the face's own, whose normal
    /// comes out `1e-17` or `5.2e-16` off the other face's
    /// (`docs/decisions.md` §219.6) — so those are then asked how far the
    /// curve on each plane can reach off the other, against `rounding`, the
    /// room [`Model::what_writing_a_plane_down_costs`] gives a corner against
    /// its own face's plane. A pipe, a ball and a torus have that bound, and a
    /// cone where the plane crosses it in an ellipse (Q612), in straight lines
    /// through its apex ([`how_far_a_crossing_reaches_off_another_plane`],
    /// Q613), or, for an arc, in a parabola or a hyperbola (Q1086); an edge whose
    /// crossing has none between two planes proved different is refused, which
    /// is the safe way for a missing bound to fall.
    ///
    /// **Not shared with [`Model::check_touched_edge`]**, unlike the two checks
    /// before it in [`Model::check_edge`]. A mutation that ends there either
    /// writes no edge this refuses or asks this first in a model of its own, and
    /// the one other way such an edge reaches it is in a file the caller read
    /// in, which is the caller's input and not a panic (Q614). That doc names
    /// the mutations, and the tests that hold each.
    ///
    /// # Errors
    ///
    /// [`Invalid::ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes`] for a closed
    /// edge and [`Invalid::AnArcBetweenTwoFlatFacesOnDifferentPlanes`] for an
    /// open one, and [`Error::Undecidable`] where [`Plane::agreement_with`]
    /// cannot tell — an answer nobody could reach is not taken for "one plane".
    fn check_flat_faces_put_an_edge_in_one_place(
        &self,
        slot: EdgeSlot,
        ends: [Point; 2],
        rounding: f64,
    ) -> Result<()> {
        let run = &self.data().edges[slot].run;
        let in_two_places = match run {
            EdgeRun::RightRoundWhereItCrosses { .. } => {
                Invalid::ClosedEdgeBetweenTwoFlatFacesOnDifferentPlanes
            }
            EdgeRun::PartOfWhereItCrosses { .. } => {
                Invalid::AnArcBetweenTwoFlatFacesOnDifferentPlanes
            }
            EdgeRun::Straight | EdgeRun::RightRoundWhereItsFacesCross | _ => return Ok(()),
        };
        let Some(planes) = self
            .faces_along_edge(slot)
            .map(|face| self.data().faces[face].surface.as_plane().copied())
            .collect::<Option<Vec<Plane>>>()
        else {
            return Ok(());
        };
        let Some((first, others)) = planes.split_first() else {
            return Ok(());
        };
        for other in others {
            match first.agreement_with(other) {
                PlaneAgreement::SameWay | PlaneAgreement::OppositeWay => continue,
                PlaneAgreement::Different => {}
                PlaneAgreement::CannotTell => {
                    let mut records = ProvenanceLog::new();
                    records.push(Record::new(
                        ORIENT2D,
                        &[
                            PredicatePoint::from(first.normal().components()),
                            PredicatePoint::from(other.normal().components()),
                        ],
                        Level::Interval,
                        f64::INFINITY,
                        Measures::Area,
                    ));
                    return Err(Error::Undecidable { records });
                }
                unknown => {
                    panic!(
                        "plane agreement came back with an answer nothing here knows: {unknown:?}"
                    )
                }
            }
            let within = |on: &Plane, off: &Plane| {
                how_far_a_crossing_reaches_off_another_plane(on, off, run, ends)
                    .is_some_and(|reach| reach <= rounding)
            };
            if !(within(first, other) && within(other, first)) {
                return Err(in_two_places.into());
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

    fn check_coedges(&self, scope: &Scope) -> Result<()> {
        used_exactly_once(
            "coedge",
            scope.coedges.keys(),
            self.data().wires.values().flat_map(WireData::coedges),
        )
    }

    fn check_wires(&self, scope: &Scope) -> Result<()> {
        for slot in scope.wires.keys() {
            self.check_wire_closes(&self.data().wires[slot])?;
        }
        used_exactly_once(
            "wire",
            scope.wires.keys(),
            self.data()
                .faces
                .values()
                .flat_map(|face| face.wires.iter().copied()),
        )
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
    /// on — and both of them from the one shell, which is what says that shell
    /// closes along this edge on its own.
    ///
    /// **Counting coedges over the whole model is not enough on its own.** A
    /// skin shared out between two shells still has two coedges running
    /// opposite ways along every edge of the cut, one from each side, so the
    /// count passed a cube whose six faces were split between two solids
    /// (`docs/decisions.md` §342). The shell is read off the index, one lookup
    /// per level, so this stays a question about one edge and costs no sweep.
    ///
    /// A coedge that no wire walks, a wire on no face, or a face in no shell
    /// has no shell to compare, and is left to the checks that refuse an
    /// orphan by name. `Deserialize` has refused all three before it asks this.
    ///
    /// Shared with [`Model::check_touched_edge`] for the reason
    /// [`Model::check_wire_closes`] gives.
    ///
    /// # Errors
    ///
    /// [`Invalid::EdgeCoedgeCount`], [`Invalid::CoedgesRunTheSameWay`] or
    /// [`Invalid::AnEdgeIsWalkedOnceFromEachOfTwoShells`].
    pub(crate) fn check_edge_is_walked_by_two_opposite_coedges(
        &self,
        slot: EdgeSlot,
    ) -> Result<()> {
        let &[one, other] = self.index().coedges_of_edge(slot) else {
            return Err(Invalid::EdgeCoedgeCount {
                count: self.index().coedges_of_edge(slot).len(),
            }
            .into());
        };
        if self.data().coedges[one].orientation == self.data().coedges[other].orientation {
            return Err(Invalid::CoedgesRunTheSameWay.into());
        }
        if let (Some(one), Some(other)) = (self.shell_of_coedge(one), self.shell_of_coedge(other))
            && one != other
        {
            return Err(Invalid::AnEdgeIsWalkedOnceFromEachOfTwoShells.into());
        }
        Ok(())
    }

    /// The shell whose skin this coedge is part of, through its wire and that
    /// wire's face, or `None` where one of those links is missing.
    fn shell_of_coedge(&self, coedge: CoedgeSlot) -> Option<ShellSlot> {
        let wire = self.index().wire_of_coedge(coedge)?;
        let face = self.index().face_of_wire(wire)?;
        self.index().shell_of_face(face)
    }

    /// Every face on the surface it says it is on, wound the way its wires say,
    /// and used by exactly one shell.
    ///
    /// **The `policy` is here for the winding, and only for the winding.** A
    /// flat face whose boundary bends has to have that boundary walked before
    /// it can be wound ([`check_flat_face`]), and walking a crossing is the one
    /// thing in this sweep that asks how hard to try. Nothing else here has
    /// grown an opinion about strictness.
    fn check_faces(&self, scope: &Scope, policy: &Policy, log: &mut ProvenanceLog) -> Result<()> {
        let largest_coordinate = self.largest_coordinate_in(scope);
        let allowed_wobble = self.allowed_wobble(largest_coordinate);
        let writing_a_point_down = self.what_writing_a_point_down_costs(largest_coordinate);
        for slot in scope.faces.keys() {
            self.check_face(slot, allowed_wobble, writing_a_point_down, policy, log)?;
        }
        used_exactly_once(
            "face",
            scope.faces.keys(),
            self.data()
                .shells
                .values()
                .flat_map(|shell| shell.faces.iter().map(|used| used.face)),
        )
    }

    /// One face: it has a boundary, and it sits on the surface it says it does
    /// with its wires wound the way that surface points.
    ///
    /// `allowed_wobble` and `writing_a_point_down` both fold over every vertex
    /// in the scope and neither changes between faces, so both are worked out
    /// once by the caller and handed in.
    fn check_face(
        &self,
        slot: FaceSlot,
        allowed_wobble: f64,
        writing_a_point_down: f64,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        let face = &self.data().faces[slot];
        if face.wires.is_empty() {
            return Err(Invalid::FaceHasNoWires.into());
        }

        let view = FaceView::new(self, slot, Isometry::identity(), Orientation::Forward);
        if let Surface::Plane(plane) = &face.surface {
            check_flat_face(self, view, plane, allowed_wobble, policy, log)?;
        } else {
            check_curved_face(view, &face.surface, writing_a_point_down)?;
        }
        self.check_every_pole_stands_where_the_surface_pinches(slot, writing_a_point_down)
    }

    /// Every pole step of this face stands where its surface pinches to a
    /// point, as far as anyone can prove otherwise.
    ///
    /// **Asked after the corners are asked onto the surface**, so that a pole
    /// slid off a ball is still refused as a corner off its face — the plainer
    /// of the two things wrong with it. What is left for this check is a pole
    /// on the surface, or within room of it, that is not at a pinch. What goes
    /// wrong then is §188.2: the mesher squashes a whole side of the face into
    /// that corner and draws a mesh folded over itself.
    ///
    /// The room is [`not_provably_off`]'s, spent the other way round: the
    /// corner is widened into a box of the slop recorded on it or of
    /// `writing_it_down`, whichever is wider, and refused only when that box
    /// misses every pinch point. **A pole is not asked exactly**, because the
    /// kernel's own balls read back from the corpus put their poles a few ulps
    /// off the pinch.
    ///
    /// # Errors
    ///
    /// [`Invalid::APoleStandsWhereTheSurfaceDoesNotPinch`].
    fn check_every_pole_stands_where_the_surface_pinches(
        &self,
        slot: FaceSlot,
        writing_it_down: f64,
    ) -> Result<()> {
        let face = &self.data().faces[slot];
        for &wire in &face.wires {
            for &step in &self.data().wires[wire].steps {
                let WireStep::Pole(vertex) = step else {
                    continue;
                };
                if !self.stands_where_it_pinches(vertex, &face.surface, writing_it_down) {
                    let at = self.data().vertices[vertex].point;
                    return Err(Invalid::APoleStandsWhereTheSurfaceDoesNotPinch {
                        at,
                        pinch: nearest(at, &where_it_pinches(&face.surface)),
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
    fn check_which_side_of_its_trimming_each_curved_face_lies_on(
        &self,
        scope: &Scope,
    ) -> Result<()> {
        for slot in scope.solids.keys() {
            self.check_which_side_of_its_trimming_each_curved_face_of_this_solid_lies_on(slot)?;
        }
        Ok(())
    }

    fn check_which_side_of_its_trimming_each_curved_face_of_this_solid_lies_on(
        &self,
        slot: SolidSlot,
    ) -> Result<()> {
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
    /// corner of the face that is off it ([`the_side_this_face_lies_on`]). A
    /// face with no such corner — a bar's cap has its one corner *on* the wall —
    /// says nothing, and is skipped rather than guessed at. A corner past the
    /// half-space picking the edge's curve out of two says the side a step from
    /// the curve towards it meets, since the face may cross the other curve on
    /// the way ([`which_side_of_it_this_face_lies_on`], row Q1190).
    ///
    /// # A circle round a ring's tube is not read off a corner
    ///
    /// A plane through a ring's axis cuts it in two circles round the tube, and
    /// a band of the ring running from one of them past the other comes back to
    /// the plane's other side: a three-quarter bend's far corner is on the side
    /// of the near cap's plane the wall does not leave it on. Both bands between
    /// two such circles have the same corners, so no corner can say which one a
    /// wall is, and read off one the sweep refused every bend past half a turn
    /// and passed one walked the wrong way round (`docs/decisions.md` §392).
    /// So at such a circle:
    ///
    /// - **The ring's face is not asked the rule.** Which band it is, is which
    ///   way it walks its circles, so what is checked is that it walks every
    ///   other circle round the tube the other way from this one
    ///   ([`the_other_circles_round_the_tube_are_walked_the_other_way`]). Two
    ///   walked the same way bound no band.
    /// - **A flat face whose outer wire is that circle alone lies inside the
    ///   tube** next to it, since the disc a circle round the tube bounds is the
    ///   tube's own cross-section. That side is fact, not read, so the rule
    ///   holds the cap's walk to the way it faces, and the two coedges of the
    ///   edge running opposite ways carry that onto the ring's face.
    ///
    /// # Nor does a ring's face say at its circles round the axis
    ///
    /// A plane square to a ring's axis cuts it in two circles round the axis,
    /// and the ring less those two is two pieces, one each side of the plane,
    /// each with both circles as its whole edge (`docs/decisions.md` §243.8). A
    /// half-ring's flat face has every corner on the ring and its wall every
    /// corner on the plane, so no corner says which side either keeps, and the
    /// sweep asked nothing of a half-ring whose flat face walks its hole the
    /// same way round as its outline. Nor did the mesh notice: it drew the flat
    /// face by its plane and its wires' places in the face, and the wall as the
    /// half its outer circle's walk gives, which are the half-ring's own
    /// triangles.
    ///
    /// So at a circle round the axis ([`a_circle_round_the_axis`]), the side of
    /// the plane the ring's face keeps is read off the way its wire walks the
    /// circle ([`the_side_its_wire_keeps`]). The plane is the face across, or
    /// the circle's own plane where the run names one beside a pipe, cone or
    /// ball across: a ring with its rim turned down to a pipe has the same two
    /// pieces with the same corners (row Q1173). **With two such circles, along one
    /// plane or two**, the face is one of the two pieces of the ring between
    /// them, and each leaves its two circles going opposite ways round the
    /// tube: that is what the face is held to, by each circle's side and the
    /// side of the pipe of the ring's own radius that names it
    /// ([`its_circles_round_the_axis_bound_a_band_of_the_tube`]), and no corner
    /// is read there. Two planes leave the band between them or the ring less
    /// it, a grooved ring's wall, and both have the same corners: read off the
    /// other circle's, the grooved ring was refused (row Q1153). Where that
    /// cannot be read, the side must be the side read at every other circle
    /// along the same stored plane, and the corners are read as before.
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
    /// # A run kind this does not know is refused, not passed over
    ///
    /// Which edges this asks about used to be `run != RightRoundWhereItsFacesCross`,
    /// which reads a fourth kind of [`EdgeRun`] as an open one and asks nothing
    /// further. `tests/a_fourth_kind_of_run.rs` names that exact shape — a solid
    /// that came back sound with nobody having looked at it — and three other
    /// sites were closed against it before this one. The three kinds are named
    /// now and anything else is [`Error::ACrossingThisCannotBeOrderedAlong`],
    /// the name the fuse already gives a run it cannot follow.
    ///
    /// **[`EdgeView::is_closed`] asks it the same way now.** It used to be a
    /// bare `==` with no wildcard for `just check-wildcards` to count, read by
    /// six production sites — `fuse/mod.rs`, `mesh.rs`, `holds.rs`, `view.rs`'s
    /// own `segment`, `rcad-step`'s writer, and `check_dynamic_range` below.
    /// Closing them one at a time would have made six copies of one list, which
    /// is the drift the collapsed-edge check was pulled back into one list to
    /// stop, so the list lives in `is_closed` and it hands back a [`Result`]:
    /// five of the six refuse with it, and `holds.rs` refuses under its own name
    /// because that one points at the face.
    ///
    /// # Errors
    ///
    /// [`Invalid::AClosedEdgeIsWalkedAgainstItsSurface`],
    /// [`Error::ACrossingThisCannotBeOrderedAlong`] for a run kind this does not
    /// know, and whatever reading a corner's side refuses.
    fn check_closed_edges_are_walked_the_way_their_surfaces_wind(
        &self,
        scope: &Scope,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        for slot in scope.solids.keys() {
            self.check_this_solids_closed_edges_are_walked_the_way_their_surfaces_wind(
                slot, policy, log,
            )?;
        }
        Ok(())
    }

    /// One solid's closed edges, each read against the second surface its run
    /// is between.
    ///
    /// **That surface is the partner, and it is not always the face across.**
    /// Where the two faces really do cross in the edge the run names the face
    /// across and the two are one. Where both faces sit on one surface, or on
    /// two spellings of it, the run names a third surface, and the face across
    /// says nothing about where the edge goes. §229.4's cone written twice, one
    /// ulp apart, is that: two bands with a closed circle between them, named
    /// by the circle's own plane. Read against the other band's cone, the two
    /// gradients at the corner cross to `(0, 1.07e-14, 0)` and the band's far
    /// corner is `-7.1e-15` inside that cone. Both signs are rounding, and
    /// together they said the lower band walks the circle the way it does not.
    /// Read against the plane, the answer is the band's.
    ///
    /// A partner that is no face of this solid has no use to turn it round, so
    /// its own gradient goes in and the face's side is read against that same
    /// surface, off the corners of the face that are not on it.
    ///
    /// # The rule is read where the faces are stored
    ///
    /// The two surfaces, the run and the corner the rule is asked at all come
    /// unmoved, which is the frame the fuse wrote the edge in and the frame the
    /// mesher and `holds.rs` read it back in. **A ball is why it has to be.** A
    /// sphere stores no axis: it is wound about the `z` of whatever frame it is
    /// read in ([`the_way_it_is_parametrised_about`]), so once
    /// [`Model::place_solid`] turns a solid, its ball read where the solid sits
    /// is wound about a different line from the one its edges were walked by. A
    /// pipe or a cone carries its axis with it and did not notice. Read where
    /// the solid sits, a ball-and-block union that validates upright was
    /// refused [`Invalid::AClosedEdgeIsWalkedAgainstItsSurface`] after a half
    /// turn about `x`, and turning a model cannot break it.
    ///
    /// Which side of the partner the face's material lies on is still read
    /// where the solid sits, because that is where
    /// [`the_side_this_face_lies_on`] reads the corners, and a movement keeps
    /// which side a corner is on. Which surface the partner *is* is settled in
    /// the stored frame first and then moved into place for that.
    ///
    /// # Panics
    ///
    /// If the face across an edge is placed anywhere but where this face is.
    /// That is a broken invariant and not a caller's bytes: the door refuses an
    /// edge walked from two shells
    /// ([`Invalid::AnEdgeIsWalkedOnceFromEachOfTwoShells`]) and a face use that
    /// carries a placement ([`Invalid::AFaceUseCarriesAPlacement`]), so the two
    /// faces along an edge are one shell's and sit where that shell puts them.
    /// Reading them in their one stored frame is reading them in one frame.
    fn check_this_solids_closed_edges_are_walked_the_way_their_surfaces_wind(
        &self,
        slot: SolidSlot,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        let solid = SolidView::new(self, slot, Isometry::identity(), Orientation::Forward);
        for face in solid.faces() {
            let bounds_a_band_of_the_tube =
                its_circles_round_the_axis_bound_a_band_of_the_tube(solid, face, policy, log)?;
            let mut sides_kept: Vec<SideKept> = Vec::new();
            for coedge in face.wires().flat_map(WireView::coedges) {
                let edge = coedge.edge();
                match edge.stored_run() {
                    EdgeRun::RightRoundWhereItsFacesCross
                    | EdgeRun::RightRoundWhereItCrosses { .. } => {}
                    EdgeRun::Straight | EdgeRun::PartOfWhereItCrosses { .. } => continue,
                    // A kind of run this crate has never seen may well come
                    // back to the corner it left, and taking it for an open
                    // one skips the only check there is that a closed edge
                    // is walked the way its surfaces wind. That is
                    // `a_fourth_kind_of_run.rs`'s worse shape: the solid
                    // comes back sound and nobody has looked at it.
                    _ => return Err(Error::ACrossingThisCannotBeOrderedAlong),
                }
                let Some(across) = the_face_across(solid, face, edge)? else {
                    continue;
                };
                assert!(
                    across.location() == face.location(),
                    "two faces along one edge are placed apart, which the door refuses"
                );
                let asked = what_the_rule_is_asked_at(face, across, edge);
                let names_it = asked.names_it;
                let partner = names_it[1];
                if a_circle_round_the_axis(face.stored_surface(), names_it).is_some() {
                    if bounds_a_band_of_the_tube {
                        continue;
                    }
                    if let Some(kept) = the_side_its_wire_keeps(&asked, coedge)? {
                        the_same_side_as_at_its_other_circles(&mut sides_kept, partner, kept)?;
                    }
                }
                let read = match a_circle_round_the_tube(names_it) {
                    Some(RoundTheTube::OnTheRing) => {
                        the_other_circles_round_the_tube_are_walked_the_other_way(
                            solid, face, coedge,
                        )?;
                        continue;
                    }
                    Some(RoundTheTube::OnThePlane)
                        if is_the_whole_of_its_outer_wire(face, coedge) =>
                    {
                        Some(Decision::Neg)
                    }
                    Some(RoundTheTube::OnThePlane) | None if partner == across.stored_surface() => {
                        which_side_of_it_this_face_lies_on(
                            solid, face, across, asked.at, policy, log,
                        )?
                    }
                    Some(RoundTheTube::OnThePlane) | None => {
                        which_side_of_a_surface_a_run_names_this_face_lies_on(solid, face, partner)?
                    }
                };
                let Some(material) = read else {
                    continue;
                };
                let Some(walks_it_forward) = a_closed_crossing_is_walked_the_way_it_is_stored(
                    [&names_it[0], &names_it[1]],
                    asked.outward,
                    asked.at,
                    turned_by(material, asked.partner_turn),
                )?
                else {
                    continue;
                };
                if walks_it_forward != (coedge.orientation_on_its_edge() == Orientation::Forward) {
                    return Err(Invalid::AClosedEdgeIsWalkedAgainstItsSurface.into());
                }
            }
        }
        Ok(())
    }

    /// No face that sits on a plane is used reversed.
    ///
    /// **This is the table two production sites already keep, read back for the
    /// first time.** `Assembly::write_into` writes it as
    ///
    /// ```text
    /// (Some(flat), true)         => (Surface::Plane(flat.reversed()), Forward)
    /// (Some(_) | None, false)    => (piece.surface,                   Forward)
    /// (None, true)               => (piece.surface,                   Reversed)
    /// ```
    ///
    /// and `rcad-step`'s `describe_the_face` writes the same three rows for a
    /// STEP `same_sense`. A plane welds "which surface" to "which way", so a
    /// plane that faces the other way is [`Plane::reversed`] and is then used
    /// forwards; a curved surface has no turned-round form — a sphere is a
    /// sphere — so it is stored as it is and the shell says
    /// [`Orientation::Reversed`] (`docs/decisions.md` §31.3, §36.24).
    ///
    /// **What it costs to leave the rule unread is a torn skin that every other
    /// check passes.** Turning one wall of a box round changes no coedge in the
    /// model, so
    /// [`Model::check_edge_is_walked_by_two_opposite_coedges`] — which reads the
    /// turns as stored — sees nothing; the wall's own winding is unchanged too,
    /// because reversing a face turns its normal *and* walks its wire the other
    /// way (`WireView::coedges`), and `check_flat_face` reads the raw face at
    /// [`Isometry::identity`] and never sees how a shell uses it at all. What is
    /// left is the volume, and flipping one wall of a `10³` box leaves the
    /// divergence sum at `333.3` — still positive, still no complaint. The fuse
    /// then walks the model and trips `Model::check_touched_edge`, which is an
    /// assert where a refusal belonged (§38.2).
    ///
    /// **A release build used to see none of that, and now sees all of it.**
    /// `check_touched_edge` returned early under `cfg!(debug_assertions)` and
    /// `Assembly::commit`'s post-write sweep was gated the same way *and* stood
    /// down whenever the model did not already validate — which a model with a
    /// turned-round wall does not — so a release fuse of two boxes with one wall
    /// turned round answered and committed a model that did not validate. §66.3
    /// took both gates away: the touched checks run in every build, and the
    /// fuse asks this very question of what it wrote, in every build. A caller
    /// now sees a refusal and not a wrong answer — §87 turned the fuse's own
    /// panic into [`Error::TheFuseWroteABrokenAnswer`], because the answer is
    /// built in a throwaway model and never reaches theirs.
    ///
    /// **Not reachable through the public API, and a file cannot carry it in
    /// either — every door this paragraph used to say was open is shut.**
    /// [`SolidBuilder::add_shell`](crate::SolidBuilder::add_shell) takes an
    /// [`Orientation`] per face and documents `Reversed` as a STEP
    /// `same_sense = false`, and its own door reads the stored turns too, so a
    /// box described face by face with any one wall turned round used to build
    /// and validate. `SolidBuilder::build` refuses it by name now, which
    /// `building_a_solid_out_of_faces.rs` asserts. A file cannot carry the same
    /// shape in either: `Deserialize` runs
    /// [`Model::check_shape_is_possible`], which calls this very check, before
    /// the bytes become a `Model` at all — this file's own
    /// `a_file_with_a_wall_turned_round_never_becomes_a_model` measures that
    /// refusal directly. Nor can a solid be turned round after it is built:
    /// the only public way to move one is
    /// [`Model::place_solid`](crate::Model::place_solid), which takes an
    /// `Isometry` — never a scale and never a mirror — and the rest of the
    /// public mutations set approximation tags. `Model::add_shell` and
    /// `Model::add_solid` are `pub(crate)`, so building and reading a file
    /// are the whole surface. So this is a
    /// hardening exercise and not §16.11's kind: no caller reaches it any
    /// more. What still earns the check its place is the paragraph above — a
    /// bug in the fuse itself, not a caller, could still slip a turned-round
    /// wall past all three doors, and the sweep
    /// [`Assembly::commit`](crate::fuse::assemble::Assembly::commit) runs over
    /// its throwaway answer is what turns that into a refusal rather than a
    /// model that commits without validating.
    ///
    /// # Why this and not a count over the skin
    ///
    /// The tempting check is that the two faces along an edge walk it opposite
    /// ways *as their shell uses them*, and it is wrong: a curved face turned
    /// round is the same surface with its wire stored walked the other way, so a
    /// bar's wall used reversed walks its open edges the same way as the planes
    /// notching it and is perfectly sound. `a_washer`'s hole says the same thing
    /// about closed edges. Only a plane is welded to a direction, so only a
    /// plane can be asked this without arithmetic.
    ///
    /// # Composed, because a shell can be used reversed too
    ///
    /// Every site in this crate that builds a `ShellUse` writes
    /// [`Orientation::Forward`], but `Deserialize` will read a reversed one, and
    /// it turns every face of that shell round including the flat ones. So the
    /// turn asked about is the one the *solid* ends up using, not the one
    /// written on the face.
    ///
    /// A shell no solid uses is not walked here. That is
    /// [`Invalid::Orphan`]'s to refuse, it is refused in the sweep, and a face
    /// no solid reaches is a face no operation can be handed.
    ///
    /// # Panics
    ///
    /// If a solid names a shell that is not there, or a shell a face that is
    /// not — `ModelData::check_keys_resolve` runs ahead of this on both paths
    /// into it, so a key that does not resolve here is this crate having broken
    /// its own arena.
    ///
    /// # Errors
    ///
    /// [`Invalid::AFlatFaceIsUsedReversed`].
    fn check_no_flat_face_is_used_reversed(&self, scope: &Scope) -> Result<()> {
        for slot in scope.solids.keys() {
            self.check_no_flat_face_of_this_solid_is_used_reversed(slot)?;
        }
        Ok(())
    }

    fn check_no_flat_face_of_this_solid_is_used_reversed(&self, slot: SolidSlot) -> Result<()> {
        for used in &self.data().solids[slot].shells {
            for face in &self.data().shells[used.shell].faces {
                if face.orient.composed_with(used.orient) == Orientation::Reversed
                    && self.data().faces[face.face].surface.as_plane().is_some()
                {
                    return Err(Invalid::AFlatFaceIsUsedReversed.into());
                }
            }
        }
        Ok(())
    }

    /// No shell places a face anywhere but where that face is stored.
    ///
    /// **The placement a solid carries lives on its shells**, which is where
    /// every builder writes it (§44.8) and what [`Model::place_solid`] composes
    /// onto. Every site in this crate that builds a `FaceUse` writes
    /// [`Isometry::identity`], so only a file can carry anything else, and there
    /// is nothing for one to serve: a face is used by exactly one shell
    /// ([`Invalid::UsedMoreThanOnce`]), so the sharing that a per-use placement
    /// exists for elsewhere is the sharing §42.7 says this data model does not
    /// have.
    ///
    /// **What one does is tear the skin without moving a corner.** [`FaceView`]
    /// hands its own location down to its wires, so a displaced face reads its
    /// surface *and* its corners at the same place and stays perfectly
    /// self-consistent — `check_flat_face` has no complaint, the wire still
    /// winds the right way, the volume still comes out positive. What is no
    /// longer true is that the two faces along an edge agree where that edge is.
    /// The mesh cannot see it either: the mesher names a corner by its vertex,
    /// and both faces name the same vertex, so a wall moved a thousand units
    /// away and a wall nudged a quarter of one both used to validate.
    ///
    /// # Why the placement is refused rather than the disagreement measured
    ///
    /// The other shape this could take is to check that the faces along each
    /// edge place it identically, which is what a check would have to do if a
    /// per-face placement were a thing this kernel meant to hold. It does not:
    /// nothing writes one, nothing reads one, and §13's rule is that a part with
    /// no named consumer is not built. Refusing the placement is one comparison
    /// per face use and needs no arithmetic; measuring the disagreement needs a
    /// pass over pairs and a decision about how near is near enough, which is a
    /// tolerance this kernel exists to avoid.
    ///
    /// The third shape is to take the field away. That is a change to the file
    /// format and to §3.2's composition, and it is not needed to close this: an
    /// identity that must stay an identity costs one comparison and says the
    /// same thing out loud.
    ///
    /// # Errors
    ///
    /// [`Invalid::AFaceUseCarriesAPlacement`].
    fn check_no_face_use_carries_a_placement(&self, scope: &Scope) -> Result<()> {
        for slot in scope.shells.keys() {
            self.check_no_face_of_this_shell_carries_a_placement(slot)?;
        }
        Ok(())
    }

    fn check_no_face_of_this_shell_carries_a_placement(&self, slot: ShellSlot) -> Result<()> {
        for used in &self.data().shells[slot].faces {
            if used.loc != Isometry::identity() {
                return Err(Invalid::AFaceUseCarriesAPlacement.into());
            }
        }
        Ok(())
    }

    fn check_shells_and_solids(
        &self,
        scope: &Scope,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        used_exactly_once(
            "shell",
            scope.shells.keys(),
            self.data()
                .solids
                .values()
                .flat_map(|solid| solid.shells.iter().map(|used| used.shell)),
        )?;

        for slot in scope.solids.keys() {
            self.check_this_solid_encloses_space(slot, policy, log)?;
        }
        Ok(())
    }

    fn check_this_solid_encloses_space(
        &self,
        slot: SolidSlot,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
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
    fn check_approximation_ordering(&self, scope: &Scope) -> Result<()> {
        for slot in scope.vertices.keys() {
            slop_is_a_length("vertex", self.data().vertices[slot].approximation)?;
        }
        for slot in scope.edges.keys() {
            slop_is_a_length("edge", self.data().edges[slot].approximation)?;
        }
        for slot in scope.faces.keys() {
            slop_is_a_length("face", self.data().faces[slot].approximation)?;
        }
        for slot in scope.edges.keys() {
            self.check_this_edge_claims_no_less_slop_than_its_corners(slot)?;
        }
        for slot in scope.faces.keys() {
            self.check_this_face_claims_no_less_slop_than_its_edges(slot)?;
        }
        Ok(())
    }

    fn check_this_edge_claims_no_less_slop_than_its_corners(&self, slot: EdgeSlot) -> Result<()> {
        let edge = &self.data().edges[slot];
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
        Ok(())
    }

    fn check_this_face_claims_no_less_slop_than_its_edges(&self, slot: FaceSlot) -> Result<()> {
        let face = &self.data().faces[slot];
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
        Ok(())
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
    ///
    /// Both ends of the range are read off `scope` alone. Over the whole model
    /// that is a fold across every solid, so two solids each narrow enough on
    /// their own can be too far apart together — see
    /// [`Model::validate_solid`].
    fn check_dynamic_range(
        &self,
        scope: &Scope,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        let solids = || {
            scope
                .solids
                .keys()
                .map(|slot| SolidView::new(self, slot, Isometry::identity(), Orientation::Forward))
        };
        let largest = solids()
            .flat_map(|solid| solid.faces())
            .flat_map(|face| face.wires())
            .flat_map(|wire| wire.points())
            .map(|corner| corner.largest_coordinate_magnitude())
            .fold(0.0, f64::max);
        let writing_it_down =
            self.what_writing_a_point_down_costs(self.largest_coordinate_in(scope));
        // A closed edge is skipped rather than measured, because the distance
        // between its two ends is zero and that is not how small it is — it is
        // a whole circle. What stands in for it is the radius of the curved
        // surface it runs round, taken from the faces below, which is the
        // smallest thing about such an edge that anything here can name.
        //
        // A run kind this crate cannot read is refused rather than measured. Its
        // chord is either nothing, and then the smallest feature of the model is
        // reported as zero and the refusal names a number instead of the edge,
        // or it is a real length that stands in for a curve nobody here can
        // measure. `EdgeView::is_closed` refuses under the name
        // `check_closed_edges_are_walked_the_way_their_surfaces_wind` already
        // gives the same arrival, ten checks up this file.
        let mut smallest_chord = f64::INFINITY;
        for slot in scope.edges.keys() {
            let edge = EdgeView::new(self, slot, Isometry::identity(), Orientation::Forward);
            if !edge.is_closed()? {
                smallest_chord = smallest_chord.min(edge.distance_between_its_ends());
            }
        }
        let smallest_radius = solids()
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
                //
                // **A pole is at the apex exactly when the pole check says it
                // is**, so it is asked that check's own question, of the same
                // stored corner and surface. A pole a few ulps up the ruling is
                // one the pole check keeps, and its radius of `2^-51` counted
                // here refused a sound cone as a range of `1.8e16` (§190.5).
                Surface::Cone(cone) => {
                    let along = cone.axis().direction();
                    let stored = &self.data().faces[face.key().slot_in(self.id())].surface;
                    let at_the_apex: Vec<VertexSlot> = face
                        .wires()
                        .flat_map(WireView::steps)
                        .filter_map(|step| match step {
                            WireStepView::Pole(corner) => Some(corner.key().slot_in(self.id())),
                            WireStepView::Along(_) => None,
                        })
                        .filter(|&pole| self.stands_where_it_pinches(pole, stored, writing_it_down))
                        .collect();
                    face.wires()
                        .flat_map(WireView::coedges)
                        .map(CoedgeView::start)
                        .filter(|corner| !at_the_apex.contains(&corner.key().slot_in(self.id())))
                        .map(|corner| cone.radius_at(along.dot(corner.point() - cone.apex())))
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

    /// Whether a pole's corner is where `surface` pinches, as far as anyone can
    /// prove otherwise.
    ///
    /// **The one answer to "is this corner the apex"**, asked by the pole check
    /// to refuse a pole anywhere else and by the dynamic range check to pass the
    /// apex over. Two copies of the question could draw the line in two places,
    /// and a pole between them would be kept by one and counted by the other.
    ///
    /// The corner is widened by the slop recorded on it or by
    /// `writing_it_down`, whichever is wider — see
    /// [`Model::check_every_pole_stands_where_the_surface_pinches`] for why a
    /// pole is not asked exactly.
    fn stands_where_it_pinches(
        &self,
        vertex: VertexSlot,
        surface: &Surface,
        writing_it_down: f64,
    ) -> bool {
        let corner = &self.data().vertices[vertex];
        let reach = corner
            .approximation
            .map_or(0.0, Approximation::magnitude)
            .max(writing_it_down);
        !provably_away_from_every_pinch(corner.point, reach, &where_it_pinches(surface))
    }

    /// How far off its surfaces [`Model::validate_solid`] lets a corner of
    /// `solid` sit: [`Model::allowed_wobble`] off a plane and
    /// [`Model::what_writing_a_point_down_costs`] off a curved surface, both at
    /// the size that sweep measures `solid` against.
    ///
    /// **The fuse grants a corner it solves again this room**, read from here
    /// and not copied, because a solid the sweep accepts is one the fuse has to
    /// be able to bake (`docs/decisions.md` §75.6, §368) and two copies of a
    /// formula drift.
    ///
    /// It is `validate_solid`'s size and not `validate`'s. A solid far out in
    /// the same model widens the room `validate` gives every corner, and a
    /// corner that passes only because of a neighbour is one `validate_solid`
    /// refuses, for the reason its own doc comment gives.
    pub(crate) fn room_a_corner_of_this_solid_gets(&self, solid: SolidSlot) -> CornerRoom {
        let largest_coordinate =
            self.largest_coordinate_in(&Scope::what_this_solid_reaches(self.data(), solid));
        CornerRoom {
            off_a_plane: self.allowed_wobble(largest_coordinate),
            off_a_curved_surface: self.what_writing_a_point_down_costs(largest_coordinate),
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
    ///
    /// # The first half takes the smaller of what the model says and what it is
    ///
    /// [`Scale`] arrives with the file, so a tolerance read off it alone lets
    /// the file decide how closely the file will be checked. Bytes declaring a
    /// part `1.8446744073709552e21` mm across — two hundred light-years — bought
    /// `1.8e12` mm of room and walked two ten-millimetre blocks whose walls sat
    /// twenty and ten millimetres off their own planes straight through here.
    /// `docs/decisions.md` §70.
    ///
    /// [`Model::largest_coordinate_in`] is measured rather than declared, and a
    /// part that really is that big has corners to match, so taking the smaller
    /// of the two costs a legitimate large model nothing and leaves a model that
    /// overstates itself with the tolerance its own corners earn.
    ///
    /// The largest coordinate is handed in rather than folded for, because
    /// every caller wants two or three of these numbers and the fold is
    /// [`Model::largest_coordinate_in`]'s pass over every vertex in the scope.
    fn allowed_wobble(&self, largest_coordinate: f64) -> f64 {
        let what_the_model_says = self.scale().default_linear_tolerance();
        let what_its_corners_show = Scale::RELATIVE_LINEAR_TOLERANCE * largest_coordinate;
        what_the_model_says.min(what_its_corners_show)
            + self.what_writing_a_plane_down_costs(largest_coordinate)
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
    fn what_writing_a_plane_down_costs(&self, largest_coordinate: f64) -> f64 {
        MEASURING_SLACK * Plane::SIGNED_DISTANCE_ROUNDOFF * largest_coordinate
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
    /// The size it is measured against is [`Model::largest_coordinate_in`], the
    /// same size the flat check uses so that the two have one standard. It
    /// does **not** grow with a surface's own numbers,
    /// so a pipe of radius `1e9` carrying a face at the origin gets a budget
    /// smaller than its own radius rounds by. That is the safe direction and it
    /// is untested here: too small a budget refuses a sound corner, where too
    /// large a one accepts a corner nobody placed.
    fn what_writing_a_point_down_costs(&self, largest_coordinate: f64) -> f64 {
        MEASURING_SLACK * NEAREST_F64_POINT_ROUNDOFF * largest_coordinate
    }

    /// The size every room in this sweep is measured against: the largest
    /// coordinate magnitude any corner in `scope` holds where it is stored, or
    /// where its solid's placement puts it, **whichever is smaller**.
    ///
    /// **A corner has two places, and it has to fit the room of both.** The
    /// checks read every face at [`Isometry::identity`], so they measure how far
    /// a corner is off its surface where the corner is stored, and that is where
    /// its numbers were written down. Everything outside this crate finds the
    /// corner where the placement puts it instead: a view composes the placement
    /// in, the writer writes the corner there, and the reader builds the solid
    /// there and sizes its room from there. How far the corner is off its
    /// surface is the same in both places, up to the rounding in moving it, but
    /// the room is not, because the room grows with the coordinates.
    ///
    /// Sized from where the corners are stored alone, a solid moved toward the
    /// origin got more room here than the reader gives the same solid out of
    /// its own file, and a model this passed was written as a file the reader
    /// refuses (`docs/decisions.md` §363). Sized from where the placement puts
    /// them alone, a placement read from a file could widen the room without
    /// limit: a translation of `5.5e303` makes a corner `4.7e226` off its ball
    /// read as on it. Taking the smaller of the two means a placement can
    /// narrow the room and never widen it, so this passes nothing it passed
    /// before it looked at placements.
    fn largest_coordinate_in(&self, scope: &Scope) -> f64 {
        let stored = scope
            .vertices
            .keys()
            .map(|slot| {
                self.data().vertices[slot]
                    .point
                    .largest_coordinate_magnitude()
            })
            .fold(0.0, f64::max);
        stored.min(self.largest_placed_coordinate_in(scope))
    }

    /// The largest coordinate magnitude any corner in `scope` holds, read where
    /// its solid's placement puts it.
    ///
    /// A corner that no solid in `scope` reaches has no placement to read, so
    /// it is read where it is stored. Only [`Model::validate`] meets one, and
    /// it goes on to refuse the model for it.
    fn largest_placed_coordinate_in(&self, scope: &Scope) -> f64 {
        let mut placed: SecondaryMap<VertexSlot, f64> = SecondaryMap::new();
        for slot in scope.solids.keys() {
            let solid = SolidView::new(self, slot, Isometry::identity(), Orientation::Forward);
            for step in solid
                .faces()
                .flat_map(FaceView::wires)
                .flat_map(WireView::steps)
            {
                let corners = match step {
                    WireStepView::Along(coedge) => [coedge.start(), coedge.end()],
                    WireStepView::Pole(corner) => [corner, corner],
                };
                for corner in corners {
                    let size = corner.point().largest_coordinate_magnitude();
                    let slot = corner.key().slot_in(self.id());
                    let widest = placed.get(slot).map_or(size, |&before| before.max(size));
                    placed.insert(slot, widest);
                }
            }
        }
        scope
            .vertices
            .keys()
            .map(|slot| {
                placed.get(slot).copied().unwrap_or_else(|| {
                    self.data().vertices[slot]
                        .point
                        .largest_coordinate_magnitude()
                })
            })
            .fold(0.0, f64::max)
    }
}

/// How far off each kind of surface the sweep lets one solid's corner sit
/// ([`Model::room_a_corner_of_this_solid_gets`]).
///
/// Two numbers and not one, because the sweep grants a flat face a modelling
/// tolerance and a curved face only the width of the number
/// ([`Model::what_writing_a_point_down_costs`] says why).
#[derive(Clone, Copy, Debug)]
pub(crate) struct CornerRoom {
    /// [`Model::allowed_wobble`].
    pub(crate) off_a_plane: f64,
    /// [`Model::what_writing_a_point_down_costs`].
    pub(crate) off_a_curved_surface: f64,
}

impl CornerRoom {
    /// The room the sweep grants a corner off `surface`.
    pub(crate) fn off(self, surface: &Surface) -> f64 {
        if surface.as_plane().is_some() {
            self.off_a_plane
        } else {
            self.off_a_curved_surface
        }
    }
}

/// The entities one sweep asks about: every one in the model, for
/// [`Model::validate`], or what one solid reaches, for
/// [`Model::validate_solid`].
///
/// One map per arena, and every check walks its arena's map in place of the
/// arena. A `SecondaryMap` walks in slot order, the order a `SlotMap` walks
/// in, so a scope holding every slot walks exactly as `validate` always did.
struct Scope {
    vertices: SecondaryMap<VertexSlot, ()>,
    edges: SecondaryMap<EdgeSlot, ()>,
    coedges: SecondaryMap<CoedgeSlot, ()>,
    wires: SecondaryMap<WireSlot, ()>,
    faces: SecondaryMap<FaceSlot, ()>,
    shells: SecondaryMap<ShellSlot, ()>,
    solids: SecondaryMap<SolidSlot, ()>,
}

impl Scope {
    fn everything(data: &ModelData) -> Self {
        Self {
            vertices: data.vertices.keys().map(|slot| (slot, ())).collect(),
            edges: data.edges.keys().map(|slot| (slot, ())).collect(),
            coedges: data.coedges.keys().map(|slot| (slot, ())).collect(),
            wires: data.wires.keys().map(|slot| (slot, ())).collect(),
            faces: data.faces.keys().map(|slot| (slot, ())).collect(),
            shells: data.shells.keys().map(|slot| (slot, ())).collect(),
            solids: data.solids.keys().map(|slot| (slot, ())).collect(),
        }
    }

    /// The solid, and everything reached walking down from it.
    ///
    /// **A key that names nothing is stepped over rather than followed**, so
    /// that the walk cannot panic on a model `validate` would refuse by name.
    /// What stepped over it is still in the scope, and [`check_keys_resolve_in`]
    /// refuses it first thing.
    fn what_this_solid_reaches(data: &ModelData, solid: SolidSlot) -> Self {
        let mut reached = Self {
            vertices: SecondaryMap::new(),
            edges: SecondaryMap::new(),
            coedges: SecondaryMap::new(),
            wires: SecondaryMap::new(),
            faces: SecondaryMap::new(),
            shells: SecondaryMap::new(),
            solids: SecondaryMap::new(),
        };
        reached.solids.insert(solid, ());
        for used in &data.solids[solid].shells {
            let Some(shell) = data.shells.get(used.shell) else {
                continue;
            };
            reached.shells.insert(used.shell, ());
            for used in &shell.faces {
                let Some(face) = data.faces.get(used.face) else {
                    continue;
                };
                reached.faces.insert(used.face, ());
                for &slot in &face.wires {
                    let Some(wire) = data.wires.get(slot) else {
                        continue;
                    };
                    reached.wires.insert(slot, ());
                    for &step in &wire.steps {
                        reached.add_what_a_step_reaches(data, step);
                    }
                }
            }
        }
        reached
    }

    fn add_what_a_step_reaches(&mut self, data: &ModelData, step: WireStep) {
        let corners = match step {
            WireStep::Along(slot) => {
                let Some(coedge) = data.coedges.get(slot) else {
                    return;
                };
                self.coedges.insert(slot, ());
                let Some(edge) = data.edges.get(coedge.edge) else {
                    return;
                };
                self.edges.insert(coedge.edge, ());
                [edge.start, edge.end]
            }
            WireStep::Pole(corner) => [corner, corner],
        };
        for corner in corners {
            if data.vertices.contains_key(corner) {
                self.vertices.insert(corner, ());
            }
        }
    }
}

/// Every slot stored in an entity of `scope` names an entity the arenas hold.
///
/// `ModelData::check_keys_resolve`'s question and its order — edges, coedges,
/// wires, faces, shells, solids — asked of the scope rather than of every
/// arena, so that the first dangling key `validate` names is the first one
/// [`Model::validate_solid`] names when it is in that solid.
///
/// # Errors
///
/// [`Invalid::DanglingKey`] naming the kind of entity the first slot that
/// resolves to nothing was reaching for.
fn check_keys_resolve_in(data: &ModelData, scope: &Scope) -> Result<()> {
    for slot in scope.edges.keys() {
        let edge = &data.edges[slot];
        for end in [edge.start, edge.end] {
            if !data.vertices.contains_key(end) {
                return Err(Invalid::DanglingKey { entity: "vertex" }.into());
            }
        }
    }
    for slot in scope.coedges.keys() {
        if !data.edges.contains_key(data.coedges[slot].edge) {
            return Err(Invalid::DanglingKey { entity: "edge" }.into());
        }
    }
    for slot in scope.wires.keys() {
        let wire = &data.wires[slot];
        for coedge in wire.coedges() {
            if !data.coedges.contains_key(coedge) {
                return Err(Invalid::DanglingKey { entity: "coedge" }.into());
            }
        }
        for &step in &wire.steps {
            if let WireStep::Pole(vertex) = step
                && !data.vertices.contains_key(vertex)
            {
                return Err(Invalid::DanglingKey { entity: "vertex" }.into());
            }
        }
    }
    for slot in scope.faces.keys() {
        for &wire in &data.faces[slot].wires {
            if !data.wires.contains_key(wire) {
                return Err(Invalid::DanglingKey { entity: "wire" }.into());
            }
        }
    }
    for slot in scope.shells.keys() {
        for used in &data.shells[slot].faces {
            if !data.faces.contains_key(used.face) {
                return Err(Invalid::DanglingKey { entity: "face" }.into());
            }
        }
    }
    for slot in scope.solids.keys() {
        for used in &data.solids[slot].shells {
            if !data.shells.contains_key(used.shell) {
                return Err(Invalid::DanglingKey { entity: "shell" }.into());
            }
        }
    }
    Ok(())
}

/// Whether an edge bends, every face along it sits on a plane, and its run names
/// no curved surface — so nothing in the model says where it goes.
///
/// A plane cannot say where a curve drawn on it bends. A run naming a plane
/// names where two planes cross, which is a straight line and not this edge; a
/// whole closed curve names no surface at all; and a kind of run this crate
/// does not know names nothing it can read.
///
/// **This is the one copy of the rule, and both of its askers call it.**
/// [`Model::check_something_curved_says_where_this_edge_bends`] asks it in the
/// sweep, and `SolidView::mesh` asks it before walking an edge with only flat
/// faces along it and refuses by it as
/// [`Error::AnEdgeBendsButEveryFaceAlongItIsFlat`]. The mesher used to keep its
/// own copy, and two copies of a rule that must agree is the drift §44.11 took
/// out of the length rule (`docs/decisions.md` §187.1).
pub(crate) fn nothing_curved_says_where_it_bends(
    run: &EdgeRun,
    faces_along_it: impl IntoIterator<Item = Surface>,
) -> bool {
    *run != EdgeRun::Straight
        && the_curved_surface_a_run_names(run).is_none()
        && faces_along_it
            .into_iter()
            .all(|surface| surface.as_plane().is_some())
}

/// The curved surface a run names as the second surface of its crossing, if it
/// names one.
///
/// For an edge whose every face is flat this is what says where it goes — the
/// circle's own cylinder, for a circle a file imprinted on a flat face.
pub(crate) fn the_curved_surface_a_run_names(run: &EdgeRun) -> Option<Surface> {
    match run {
        EdgeRun::PartOfWhereItCrosses { and, .. } | EdgeRun::RightRoundWhereItCrosses { and }
            if and.as_plane().is_none() =>
        {
            Some(*and)
        }
        EdgeRun::Straight
        | EdgeRun::RightRoundWhereItsFacesCross
        | EdgeRun::RightRoundWhereItCrosses { .. }
        | EdgeRun::PartOfWhereItCrosses { .. }
        | _ => None,
    }
}

/// The furthest any point of the curve `run` puts between `ends` on `on` can
/// be from `off`; `None` where the surface `run` names is one this has no
/// bound for.
///
/// Write `n` and `m` for the two unit normals and `X` for a point of the
/// curve, which is on `on` exactly, and `V` for the first end projected onto
/// `on`. Then `off`'s distance at `X` is its distance at that end, plus at
/// most the end's distance from `on` (the step from the end to `V` is along
/// `n`, and `|m·n| ≤ 1`), plus `m·(X − V)` — and `X − V` lies in `on`, so
/// only the part of `m` square to `n` reads it, which is `|n × m|` long.
/// What is left is how far apart two points of the crossing can be, which is
/// the crossing's width and is all each surface below has to say. What the
/// end misses the crossing by is left out: it is multiplied by `|n × m|`,
/// which is a rounding wherever the answer is near the room it is held to.
///
/// **A pipe** of radius `R` about the unit axis `a` is crossed by `on` in an
/// ellipse whose longest half-width is `R / |n·a|`, so `|X − V|` is at most
/// twice that, for the whole ellipse and so for any arc of it.
///
/// **A plane along the axis** crosses the pipe in two lines, and an arc there
/// is drawn straight between its ends — `n·a == 0.0` is the very test the walk
/// draws it straight by — so every point of it is as close to `off` as the
/// further end. Two faces that cross in one of the pipe's own lines are sound,
/// and this is what keeps them. A closed run there names no curve the walk
/// can draw; `R / 0` is infinite, and the answer is a NaN or infinite, which
/// no room holds.
///
/// **A plane through a cone's apex, leaning at or past the cone's own angle**,
/// crosses it in one or two of the straight lines through the apex, and an arc
/// there is drawn straight between its ends just the same
/// ([`the_walk_draws_it_straight`]), so two faces that cross in one of those
/// lines are kept the same way (Q613). A closed run there is left to the
/// width, which has none for it.
///
/// **A plane that misses a cone's apex, leaning at or past the cone's own
/// angle**, crosses it in a parabola or a hyperbola, which runs on for ever and
/// has no width. An arc of one gets the width of the arc instead, read off its
/// two ends: [`how_wide_an_arc_of_one_branch_is`] (Q1086). A closed run there
/// is left to the width, which has none for it, and the walk refuses it too.
///
/// **A ball** of radius `r` is crossed by any plane in a circle no wider than
/// the ball, so the width is `2r`.
///
/// **A torus** lies inside the ball of radius `R + r` about its centre, for a
/// ring of `R` and a tube of `r`, so whatever a plane crosses it in is no wider
/// than `2(R + r)` — and a plane square to the axis through the centre crosses
/// it in a circle that wide, so `R + r` alone would be short by half. The
/// circles the walk draws for it are inside that ball too: round the axis they
/// are at most `R + r` from it, and round the tube they are `r` across.
///
/// **A cone** is [`how_wide_a_plane_crosses_a_cone`].
///
/// Worked in plain `f64` as [`off_a_plane_by_no_more_than_writing_it_down_cost`]
/// is: the distances are what [`Plane::signed_distance_to`] rounds, and the
/// caller's room is sixteen times that. The cone's two widths are the
/// exception, and say why.
fn how_far_a_crossing_reaches_off_another_plane(
    on: &Plane,
    off: &Plane,
    run: &EdgeRun,
    [start, end]: [Point; 2],
) -> Option<f64> {
    let (EdgeRun::RightRoundWhereItCrosses { and } | EdgeRun::PartOfWhereItCrosses { and, .. }) =
        run
    else {
        return None;
    };
    let an_arc = matches!(run, EdgeRun::PartOfWhereItCrosses { .. });
    if an_arc && the_walk_draws_it_straight(on, and) {
        return Some(
            off.signed_distance_to(start)
                .abs()
                .max(off.signed_distance_to(end).abs()),
        );
    }
    let widest = if let Surface::Cone(cone) = and
        && an_arc
        && the_walk_draws_one_branch_of_an_endless_bend(on, cone)
    {
        how_wide_an_arc_of_one_branch_is(cone, [start, end])
    } else {
        how_wide_a_plane_crosses(on, and)?
    };
    let tilt = on.normal().cross(off.normal()).length();
    Some(off.signed_distance_to(start).abs() + on.signed_distance_to(start).abs() + widest * tilt)
}

/// Whether `on` crosses `and` in straight lines by the very tests the walk
/// draws a run between them straight by, so that an arc there is the straight
/// line between its two ends.
///
/// For a pipe that is `n·a == 0.0`, which is the walk's own test written out
/// again. For a cone it is two proved signs, the apex on `on` and `E` at or
/// below nothing, asked of `rcad-geom`'s own two functions for them. The walk
/// asks the same two signs: it works both numbers out exactly, rounds each
/// to the nearest `f64`, which keeps its sign, and refuses where it cannot work
/// them out, so it never draws a curve where these say straight lines.
fn the_walk_draws_it_straight(on: &Plane, and: &Surface) -> bool {
    if let Surface::Cylinder(pipe) = and {
        return on.normal().dot(pipe.axis().direction()) == 0.0;
    }
    if let Surface::Cone(cone) = and {
        return where_a_cones_apex_stands_against_a_plane(on, cone) == Decision::Zero
            && matches!(
                how_a_plane_leans_against_a_cone(on, cone),
                Decision::Zero | Decision::Neg
            );
    }
    false
}

/// Whether the walk draws a run between `on` and `cone` along a parabola or one
/// branch of a hyperbola: the apex off `on`, and `E` at or below nothing.
///
/// The same two proved signs [`the_walk_draws_it_straight`] asks, with the
/// apex's the other way round, and for the same reason: the walk turns on the
/// signs of those same two numbers. A sign that could not be proved is neither
/// shape, and leaves the arc to [`how_wide_a_plane_crosses_a_cone`], which has
/// no width for it.
fn the_walk_draws_one_branch_of_an_endless_bend(on: &Plane, cone: &Cone) -> bool {
    matches!(
        where_a_cones_apex_stands_against_a_plane(on, cone),
        Decision::Pos | Decision::Neg
    ) && matches!(
        how_a_plane_leans_against_a_cone(on, cone),
        Decision::Zero | Decision::Neg
    )
}

/// How far apart two points of the arc between `ends` can be, where a plane
/// that misses `cone`'s apex crosses it in a parabola or in a hyperbola: twice
/// `H·√(1 + t²)`, for `t` the slope and `H` the larger of the two ends' heights
/// `|(end − apex)·w|` up the unit axis `w`.
///
/// **An arc of one branch is highest at one of its ends.** Take two directions
/// in the plane: `across`, square to the axis as well, and `down`, square to
/// `across`. A step along `across` does not change the height `(X − apex)·w`,
/// so along the branch the height changes only with how far along `down` a
/// point is. The branch is `vertex + across·u + down·c·u²` for a parabola and
/// `centre + down·c·cosh σ + across·b·sinh σ` for one branch of a hyperbola —
/// the two `where_a_plane_cuts_a_cone` in `rcad-geom` derives and walks — so
/// the height is `h₀ + k·u²` or `h₀ + k·cosh σ`. A cone holds one point at
/// height nought, its apex, which the plane misses, so the height keeps one
/// sign along the branch; and it grows without end, since `|X − apex|` does
/// and is `|height|·√(1 + t²)` on a cone. So `k` has the height's sign, and
/// `|height|` is `|h₀| + |k|·u²` or `|h₀| + |k|·cosh σ`. Both are least at the
/// vertex and grow either side of it, and on any stretch of the parameter they
/// are largest at one end of it. The walk draws a branch between the
/// parameters of the run's two ends (`walk_an_endless_bend`), so an arc is one
/// such stretch, whether it lies to one side of the vertex or runs through it.
///
/// **So the arc lies in the ball of radius `H·√(1 + t²)` about the apex**, and
/// no two of its points are further apart than that ball is across.
/// `every_point_of_an_arc_of_one_branch_is_within_the_reach_of_its_ends` walks
/// arcs of both shapes, through the vertex and to one side of it, and finds
/// none outside.
///
/// **No arc runs from one nappe to the other.** A parabola lies wholly on one;
/// a hyperbola's two branches lie one on each, and the walk refuses a run whose
/// two ends it cannot prove are on one side of the apex
/// (`which_branch_of_a_hyperbola_this_run_is`), so no curve between two ends on
/// two nappes is ever drawn for this to bound. The test file
/// `an_edge_between_two_planes_its_curve_leaves.rs` builds one, in
/// `an_arc_from_one_branch_of_a_hyperbola_to_the_other_is_refused`.
///
/// What an end misses the branch by is left out, as it is for every width
/// here: it moves `H` by no more than itself, and the width is multiplied by
/// `|n × m|`. The heights are worked in intervals and taken at the most they
/// can be, and `‖w‖` at the least; the products and the root at the end round
/// by an ulp or two of the width, as the ellipse's do.
fn how_wide_an_arc_of_one_branch_is(cone: &Cone, ends: [Point; 2]) -> f64 {
    let along = cone.axis().direction().components().map(Interval::exactly);
    let apex = cone.apex().coordinates().map(Interval::exactly);
    let [one, other] = ends.map(|end| {
        let at = end.coordinates().map(Interval::exactly);
        let height = dot([0, 1, 2].map(|axis| at[axis] - apex[axis]), along);
        height.lo().abs().max(height.hi().abs())
    });
    let slope = Interval::exactly(cone.slope());
    // `height = (X − apex)·w/‖w‖`, and `|X − apex|` is `|height|·√(1 + t²)`.
    let reach_per_height = (Interval::exactly(1.0) + slope * slope).hi() / dot(along, along).lo();
    2.0 * one.max(other) * reach_per_height.sqrt()
}

/// How far apart two points of the curve `on` crosses `and` in can be, for the
/// four curved surfaces [`how_far_a_crossing_reaches_off_another_plane`] says
/// why of; `None` for any other.
///
/// Asked one surface at a time rather than by a `match`, which would need a
/// wildcard arm for the kinds [`Surface`] has yet to grow (`docs/decisions.md`
/// §40). A flat `and` between flat faces says nothing about where the edge
/// bends, and `check_something_curved_says_where_this_edge_bends` has refused it
/// before this is asked.
fn how_wide_a_plane_crosses(on: &Plane, and: &Surface) -> Option<f64> {
    if let Surface::Sphere(ball) = and {
        return Some(2.0 * ball.radius());
    }
    if let Surface::Torus(ring) = and {
        return Some(2.0 * (ring.ring_radius() + ring.tube_radius()));
    }
    if let Surface::Cone(cone) = and {
        return how_wide_a_plane_crosses_a_cone(on, cone);
    }
    let Surface::Cylinder(pipe) = and else {
        return None;
    };
    Some(2.0 * pipe.radius() / on.normal().dot(pipe.axis().direction()).abs())
}

/// How far apart two points of the ellipse `on` crosses `cone` in can be, or
/// `None` where the crossing is not proved to be an ellipse.
///
/// For `n` and `w` the plane's normal and the cone's axis at length one, `t`
/// the slope and `h` the apex's distance from `on`, the crossing is an ellipse
/// wherever `E = (n·w)² − t²|n × w|²` is above zero, and its two half-widths
/// are `|h|t/E` and `|h|t/√E` — `where_a_plane_cuts_a_cone` in `rcad-geom`
/// derives both, and draws that ellipse. `E` is at most `(n·w)²`, which is at
/// most one, so the first is the longer, and no two points of an ellipse are
/// further apart than twice its longer half-width: the width is `2|h|t/E`.
///
/// **That is the whole ellipse, on whichever nappe it lies.** Only `|h|` enters
/// it, so the apex may stand on either side of `on`; and a frustum is a piece of
/// the same double cone [`Cone`] stores by its apex and slope, so its rim is a
/// piece of this same ellipse. A plane through the apex that cuts across the
/// cone meets it there and nowhere else, and gets a width of nothing.
///
/// **Worked in intervals, unlike the rest.** `E` is a small difference of two
/// near-equal numbers wherever the plane leans at nearly the cone's own angle,
/// and `f64` can round it up by more than it is, which would shrink the width.
/// So `E` is taken at the least it can be and `h` at the most; the division and
/// the root at the end round by an ulp or two of the width, which is a rounding
/// of a term already held to sixteen times the rounding.
///
/// **A plane whose `E` is not proved above zero gets no width**: a parabola or
/// a hyperbola runs on for ever, and a closed run on one names nothing the walk
/// draws. An arc of one gets no width here either, and needs none: the caller
/// has already bounded it by its two ends, along a straight line through the
/// apex (Q613) and along a parabola or a hyperbola alike
/// ([`how_wide_an_arc_of_one_branch_is`], Q1086).
fn how_wide_a_plane_crosses_a_cone(on: &Plane, cone: &Cone) -> Option<f64> {
    let normal = on.normal().components().map(Interval::exactly);
    let along = cone.axis().direction().components().map(Interval::exactly);
    let apex = cone.apex().coordinates().map(Interval::exactly);
    let slope = Interval::exactly(cone.slope());
    let square_to_both = [
        normal[1] * along[2] - normal[2] * along[1],
        normal[2] * along[0] - normal[0] * along[2],
        normal[0] * along[1] - normal[1] * along[0],
    ];
    let up = dot(normal, along);
    let leaning = up * up - slope * slope * dot(square_to_both, square_to_both);
    if leaning.lo() <= 0.0 {
        return None;
    }
    let reach = dot(normal, apex) - Interval::exactly(on.offset());
    let furthest_reach = reach.lo().abs().max(reach.hi().abs());
    // `h = reach/‖n‖` and `E = leaning/(‖n‖²‖w‖²)`, put into `2|h|t/E`.
    Some(
        2.0 * furthest_reach
            * cone.slope()
            * dot(normal, normal).hi().sqrt()
            * dot(along, along).hi()
            / leaning.lo(),
    )
}

/// `one·other`, with every rounding on the way taken outward.
fn dot(one: [Interval; 3], other: [Interval; 3]) -> Interval {
    one[0] * other[0] + one[1] * other[1] + one[2] * other[2]
}

/// Every one of `held` named exactly once by `named`, and none of them left out.
///
/// The four arenas that sit under something else — coedges under wires, wires
/// under faces, faces under shells, shells under solids — are all asked this,
/// with a different pair of lists each time rather than with different code.
fn used_exactly_once<K: slotmap::Key>(
    entity: &'static str,
    held: impl Iterator<Item = K>,
    named: impl Iterator<Item = K>,
) -> Result<()> {
    let mut count: SecondaryMap<K, usize> = SecondaryMap::new();
    for key in named {
        if let Some(entry) = count.entry(key) {
            *entry.or_insert(0) += 1;
        }
    }
    for key in held {
        match count.get(key).copied().unwrap_or(0) {
            0 => return Err(Invalid::Orphan { entity }.into()),
            1 => {}
            count => return Err(Invalid::UsedMoreThanOnce { entity, count }.into()),
        }
    }
    Ok(())
}

/// An amount of slop an entity claims is a length, or it is not a claim about
/// anything.
///
/// An entity with no tag claims nothing and is not asked. Zero is what
/// [`magnitude_of`] stands in for a missing tag when the *ordering* is being
/// read, and that is a different question.
fn slop_is_a_length(entity: &'static str, claimed: Option<Approximation>) -> Result<()> {
    let Some(tag) = claimed else {
        return Ok(());
    };
    let magnitude = tag.magnitude();
    if !(magnitude.is_finite() && magnitude >= 0.0) {
        return Err(Invalid::ApproximationIsNotALength { entity, magnitude }.into());
    }
    Ok(())
}

/// How much space a curved solid encloses, signed by which way its faces point,
/// and whether the two faces along each of its sides agree.
///
/// A polyhedron's own fan *is* its volume; a curved solid has no such fan — a
/// round bar's caps fan into no triangles at all and its wall into two with no
/// area (§35.4) — so it is summed over a mesh instead. The caller reads only
/// the **sign** of what comes back; the size of a mesh's sum is weighed here,
/// below, and nowhere else.
///
/// # A coarse mesh that lies flat has not answered
///
/// [`as_coarse_as_a_mesh_gets`] leans on the mesher never stepping more than a
/// quarter turn, which is what keeps a *whole* curved surface from collapsing.
/// **It says nothing about a patch that spans less than a quarter turn**, and
/// there the mesher has no reason to put a point anywhere: the patch is handed
/// to the triangulator with a budget it already meets, so it comes back as its
/// own boundary and nothing else — flat. Closed off by the flat face on the
/// other side of that boundary it encloses nothing, and [`it_lies_flat`] says
/// why that sum is no answer.
///
/// **Square to the axes that nothing is zero to the last bit; turned, it is a
/// rounding of either sign.** A cap of a ball of radius 13, one high, its rim
/// written as three arcs, is handed over as its rim's six points at 63 of the
/// 156 turns `a_cap_whose_rim_is_three_arcs_lies_flat_at_every_turn_and_validates`
/// puts it through, and the six land a rounding off one plane: the flat mesh
/// encloses a little below zero at 34 of them and a little above at 29. Only a
/// zero used to be asked again, so turned `1/40` of a turn about `(3, −1, 2)`
/// the cap enclosed `−1.4e-15` and a sound solid was refused
/// [`Invalid::VolumeIsNotPositive`]
/// (`a_cap_whose_rim_is_three_arcs_validates_turned_off_the_axes`, row Q1849);
/// and the same cap turned inside out encloses a rounding above zero at 28 of
/// the turns, and a solid whose faces all point into it passed.
///
/// A whole ring turned `3/40` of a turn about `(1, 2, 3)` came back meshed as
/// its two equators alone, enclosed a little under zero, and was refused the
/// same way (row Q1632, `a_half_ring_standing_on_a_round_plate.rs`). That flat
/// mesh was the mesher's to mend: it had walked half a turn round the tube in
/// one step, against the quarter-turn rule above, and walks it in two now.
///
/// A ball cut by a plane is exactly that, and it is `docs/decisions.md` §41.6: a
/// cap under `h/r = 1 − cos(π/4) = 0.2929` spans under a quarter turn, and on a
/// ball of radius four that is a cap of 15.52 and 5.8% of the ball. **Cut square
/// to the ball's own polar axis the same cap answers**, because then it holds a
/// pole and is fanned up to it whatever the deflection — so the same two solids
/// a quarter turn apart answered one way round and refused the other, which is
/// the §19 violation §39.7 item 1 leads with.
///
/// So a flat mesh is asked again with less room. Halving from the mesh's own
/// width rather than down from [`as_coarse_as_a_mesh_gets`] is what keeps it to
/// a handful of meshes: the bulge a flat patch is hiding is at most that width,
/// and every halving that does not find it rules out half of what is left.
/// **The finer meshes are read and thrown away**; nothing a caller measures is
/// worked out here.
///
/// # A sum answers once it is larger than what a mesh can leave out
///
/// The solid's volume is within [`how_much_space_hides_under`] of what the
/// mesh encloses exactly, and that is within the second number
/// [`signed_volume_over_and_how_far_it_rounded`] hands back of the sum. A sum
/// larger than the two together has the solid's own sign, and answers; a sum
/// that is not has said nothing, **whether or not the mesh lies flat**, and
/// the next rung is asked for.
///
/// **A mesh that does not lie flat used to answer at once, on its sign.** But
/// each face is cut into triangles on its own, and where two curved faces meet
/// in a rim whose corners do not lie in one plane, each can cross the rim by a
/// different diagonal. Between the two, the mesh is a thin solid of either
/// sign. The lens a ball makes poking `1/16` into a pipe, built turned `8/40`
/// of a turn about `x`, has a coarsest mesh of twelve points that does not lie
/// flat and encloses `−0.041`, with `2.2` of space it can leave out. That sum
/// was read as the lens's volume: the lens was refused
/// [`Invalid::VolumeIsNotPositive`], and the same lens turned inside out passed
/// (`a_lens_where_a_ball_pokes_into_a_pipe_validates_at_every_turn`, row
/// Q1890). Weighed, it is asked again, and at room `0.0297` the lens's mesh
/// encloses `0.188`, which answers.
///
/// What that costs is the rungs down to a room where a mesh's charge is under
/// the solid's volume, for every curved solid. A probe over
/// `just t -p rcad-topo` counted 112 593 sums asked for, 39 071 answered by
/// the coarsest mesh as before, and 260 184 meshes more than the old rule
/// asked for. **That cost is not settled** (row Q1890's handoff): one test
/// went from `0.022` s to `9.1` s.
///
/// A sound solid a rounding thick, such as the sliver a fuse leaves where two
/// curved faces almost meet, lies flat in every mesh of it, so asking again
/// walked it down to the floor, minutes for one test, before its sum was
/// weighed. That is what ends the slivers of
/// `a_bend_placed_a_quarter_turn_round_from_another_answers_every_rule` and
/// `a_quarter_of_a_bar_placed_round_from_a_bend_in_it_answers_or_keeps_an_edge_a_rounding_long`,
/// at rooms `0.5` and `0.25` when the ladder halved from the width alone: a
/// probe saw one sum `2.51e-15` against `1.67e-15` hidden. The same probe, run
/// over the whole `rcad-topo` suite, saw every flat mesh summed exactly, off by
/// `3e-28` at most, and charging that changed no answer; it is charged because
/// nothing else bounds a sum kept rounded.
///
/// **A face of a cone cannot be charged yet** (see
/// [`how_much_space_hides_under`]), so a solid with one keeps the old rule: a
/// mesh that does not lie flat answers on its sign, and a flat one is asked
/// again, so a cone sliver walks to the floor as it did before. Row Q1883 gives
/// cones measure.rs's charge.
///
/// **When no rung answers, the finest sum that does not lie flat comes back**,
/// as the closest to the solid's volume this saw, and the coarse sum, the zero
/// or the rounding it was, when every mesh lay flat, so a solid flat at every
/// mesh is judged as it was before this ladder ran for it (row Q1891).
///
/// # A fold is retried; every other refusal but one still stops it
///
/// [`Error::TwoPatchesOfTheMeshDrewTheSameSide`] is the one refusal a finer
/// deflection is *expected* to cure, and it is cured the way a crossing is in
/// [`the_coarsest_mesh_whose_sides_do_not_cross`]: two patches fold onto each
/// other when a cut one of them drew lands on a side the other drew, and where
/// they fall is the mesher's own sampling choice and not the solid's. §206.3 is
/// that case with both halves in one shape — a sliver of a ball whose coarse
/// mesh came back flat and whose next mesh folded, so the bulge a third mesh
/// would have found was never asked for.
///
/// **Every other refusal but one is still the stopping condition and not a
/// failure**: the coarsest mesh was built and checked, and what this is looking
/// for is a point off the flat that no mesh of this solid holds. Handing the
/// refusal on would replace a named answer about the shape with one about a
/// deflection nobody asked for.
///
/// # Placing it took away the space is handed on
///
/// [`Error::PlacingItTookAwayTheSpaceItsMeshEncloses`] is the one refusal that
/// *is* the answer this is looking for. A finer mesh refused that way found the
/// point off the flat: it encloses space where the solid was made. It is only
/// where the placement puts it that the space is gone. Stopping there would
/// return the flat sum, and the caller would then call a sound solid
/// [`Invalid::VolumeIsNotPositive`], "its faces do not point outwards", which
/// is the misreport `docs/decisions.md` §333.6 objects to. So it is handed on,
/// and the caller gets the name [`SolidView::mesh`] gives the same solid.
///
/// **No solid is known to reach that arm, so it is a guard** (§379.5, Q783).
/// To reach it the coarsest mesh has to lie flat where the solid lands, and a
/// finer one has to enclose space where it was made and none where it lands.
/// A cap whose rim is **two** half circles was built to look for that, 0.25
/// high and put out at `2^52` where the ulp is 1. Its coarsest mesh encloses
/// `912.6666666666666` where it was made, against `0` where it lands, so it is
/// refused by name before this loop is reached
/// (`a_thin_cap_placed_where_its_height_rounds_away_is_refused_by_name`). With
/// a probe logging every ladder this loop started, the whole `rcad-topo` suite
/// started 59, in 9 tests, before the three-arc sweep was written: the
/// three-arc cap, the two slivers above, three pieces of a cone, and a model of
/// the fuzz corpus whose first rung was refused
/// `DeflectionBelowWhatAMeshCanReach`, its floor at `1.2e217`. None was
/// refused that way. That is a search, not a proof, so the arm stays.
///
/// # What stops it
///
/// **The first half needs no argument about geometry.** Halving a finite length
/// reaches zero, and `Deflection::of` refuses a length that is not greater than
/// zero, so the loop ends whatever any mesh does — a fold retried for ever is
/// still a fold retried on a deflection walking down to nothing.
///
/// **Retrying a fold does not loosen the second half either.** Both refusals
/// that end it on a real solid are raised *before* a triangle is drawn —
/// `DeflectionBelowWhatAMeshCanReach` when the mesher is asked for a deflection
/// finer than the walk under this solid's own numbers will take, and
/// `MoreMeshPointsThanASolidWillHold` while the points are being laid down —
/// and a fold is only found by `finish`, once the whole mesh is built. So a
/// solid that folds at every deflection walks the same ladder it would have
/// walked lying flat at every deflection, and stops on the same rung; what the
/// retry costs is the meshes between the first fold and that rung, and the
/// coarse sum stands.
///
/// # Errors
///
/// Whatever [`the_coarsest_mesh_whose_sides_do_not_cross`] makes of the solid,
/// [`Invalid::FacesDoNotAgreeWhichWayTheyFace`] when two faces along one
/// side do not, and [`Error::PlacingItTookAwayTheSpaceItsMeshEncloses`] when a
/// finer mesh is refused that way.
pub(crate) fn how_much_space_a_curved_solid_encloses(
    view: SolidView<'_>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<f64> {
    let coarsest = the_coarsest_mesh_whose_sides_do_not_cross(view, policy, log)?;
    every_side_of_the_skin_is_used_both_ways(&coarsest)?;
    let (enclosed, rounding) = signed_volume_over_and_how_far_it_rounded(&coarsest);
    if the_sum_answers(view, &coarsest, enclosed, rounding) {
        return Ok(enclosed);
    }
    let mut finest_that_is_not_flat =
        (enclosed != 0.0 && !it_lies_flat(&coarsest)).then_some(enclosed);
    let mut room = how_far_across(&coarsest) / 2.0;
    while let Ok(less) = Deflection::of(room) {
        let mut this_rung = ProvenanceLog::new();
        match view.mesh(less, policy, &mut this_rung) {
            Ok(finer) => {
                let (finer_encloses, rounding) = signed_volume_over_and_how_far_it_rounded(&finer);
                if the_sum_answers(view, &finer, finer_encloses, rounding) {
                    this_rung.drain_into(log);
                    return Ok(finer_encloses);
                }
                if finer_encloses != 0.0 && !it_lies_flat(&finer) {
                    finest_that_is_not_flat = Some(finer_encloses);
                }
            }
            Err(Error::TwoPatchesOfTheMeshDrewTheSameSide { .. }) => {}
            Err(why) => {
                let handed_on = hand_on_a_refusal_that_found_the_space(why);
                if handed_on.is_err() {
                    this_rung.drain_into(log);
                }
                handed_on?;
                break;
            }
        }
        room /= 2.0;
    }
    Ok(finest_that_is_not_flat.unwrap_or(enclosed))
}

/// Whether what a mesh of this solid encloses says which way the solid's faces
/// point: it is not zero, and the sum, less `rounding`, how far it can be from
/// what the mesh's triangles enclose exactly, is still larger than all the
/// space the mesh can leave out. A solid with a face that space cannot be
/// charged for answers, as it did before that space was weighed, whenever the
/// mesh does not lie flat.
///
/// **The rounding is charged because the size is read, not only the sign.**
/// [`signed_volume_over_and_how_far_it_rounded`] keeps a rounded sum whenever
/// it is further from zero than its own slack, so a sum just past that slack
/// can be off by nearly all of itself, and a mesh whose exact volume is under
/// the hidden space would then pass.
fn the_sum_answers(view: SolidView<'_>, mesh: &Mesh, enclosed: f64, rounding: f64) -> bool {
    if enclosed == 0.0 {
        return false;
    }
    match how_much_space_hides_under(view, mesh) {
        Some(hidden) => enclosed.abs() > (hidden + rounding).next_up(),
        None => !it_lies_flat(mesh),
    }
}

/// The most space there can be between a mesh of this solid and the solid,
/// on either side.
///
/// Each curved face is charged `area · d · (r / (r − d))²`, with `area` its
/// triangles' area, `d` the mesh's [`Mesh::deviation`] and `r` the smallest
/// radius the face bends at, and a flat face nothing. That is
/// `Measurement::volume_error`'s charge loosened: `Mesh::deviation`'s doc
/// shows `(r³ − (r − d)³) / (3(r − d)²) ≤ d · (r / (r − d))²`.
///
/// `None` when a triangle with area stands in for a face this cannot charge,
/// and `f64::INFINITY` when the mesh strays a whole radius of a curved face or
/// more, which a finer mesh can still bring under it.
///
/// **A cone cannot be charged.** Its radius runs down to nothing at the
/// apex, and how near a face comes to it takes a walk over the mesh that
/// `measure::bend_of_each_face` already makes; a second copy of that walk is
/// not worth what it buys here. So a solid with a cone face is answered by a
/// mesh that does not lie flat, as before. Row Q1883 puts measure.rs's own
/// charge here in place of this copy, and that one reaches the tip.
fn how_much_space_hides_under(view: SolidView<'_>, mesh: &Mesh) -> Option<f64> {
    let deviation = mesh.deviation().magnitude();
    let radius_of: HashMap<FaceKey, Option<f64>> = view
        .faces()
        .map(|face| (face.key(), the_smallest_radius_it_bends_at(&face.surface())))
        .collect();
    let mut hidden = 0.0;
    for triangle in mesh.triangles() {
        let Some(radius) = radius_of[&triangle.face()] else {
            continue;
        };
        let [a, b, c] = triangle
            .corners()
            .map(|corner| mesh.points()[corner as usize]);
        let area = (b - a).cross(c - a).length() / 2.0;
        if area == 0.0 {
            continue;
        }
        if radius == 0.0 {
            return None;
        }
        let inner = radius - deviation;
        if inner <= 0.0 {
            return Some(f64::INFINITY);
        }
        hidden += area * deviation * (radius / inner) * (radius / inner);
    }
    Some(hidden)
}

/// `None` for a plane, which does not bend, and `Some(0.0)` for a surface
/// whose bend this has not been taught, which
/// [`how_much_space_hides_under`] cannot charge at all.
fn the_smallest_radius_it_bends_at(surface: &Surface) -> Option<f64> {
    match surface {
        Surface::Plane(_) => None,
        Surface::Sphere(ball) => Some(ball.radius()),
        Surface::Cylinder(pipe) => Some(pipe.radius()),
        Surface::Torus(ring) => Some(ring.tube_radius()),
        Surface::Cone(_) => Some(0.0),
        _ => Some(0.0),
    }
}

/// Whether every point of the mesh lies within a rounding of one plane, so
/// that what it encloses is a rounding too and says nothing about the solid.
///
/// **Why a flat mesh's sum is no answer.** Turn the frame so the plane is
/// `z = h`. By the divergence theorem over the field `(0, 0, z)`, a closed mesh
/// encloses `Σ Aₜ · z̄ₜ`, exactly: each triangle's area seen from above, signed
/// by which way it faces, times the mean height of its three corners. A closed
/// mesh's areas seen from above add to zero, so `h` drops out: a mesh whose
/// points stray at most `δ` from a plane encloses at most `δ` times its area.
///
/// The rounding is `64` ulps of the largest coordinate in the mesh. Placing a
/// solid rounds every coordinate of it, so points that lay on one plane land
/// an ulp or two off it. Turned through 156 turns
/// (`a_cap_whose_rim_is_three_arcs_lies_flat_at_every_turn_and_validates`), the
/// three-arc cap's coarsest mesh is its rim's six points at 63, and they stray
/// at most `2.1` ulps from one plane; at the other 93 it has a point off the
/// rim, and strays at least `4e14`. The `64` sits far from both. A mesh
/// answers by its size, flat or not, unless the solid has a face
/// [`how_much_space_hides_under`] cannot charge; read flat, such a mesh is
/// only asked again. So a bend under `64` ulps taken for flat costs rungs, and
/// at worst comes back as its coarse sum, which is what it answered before.
fn it_lies_flat(mesh: &Mesh) -> bool {
    const ROUNDINGS_OFF_THE_PLANE: f64 = 64.0;
    how_far_it_strays_from_one_plane(mesh)
        <= ROUNDINGS_OFF_THE_PLANE * f64::EPSILON * largest_coordinate_of(mesh)
}

fn largest_coordinate_of(mesh: &Mesh) -> f64 {
    mesh.points()
        .iter()
        .flat_map(|point| point.coordinates())
        .fold(0.0, |largest: f64, coordinate| {
            largest.max(coordinate.abs())
        })
}

fn how_far_it_strays_from_one_plane(mesh: &Mesh) -> f64 {
    let points = mesh.points();
    let Some(&start) = points.first() else {
        return 0.0;
    };
    let farthest_from = |from: Point| {
        points
            .iter()
            .copied()
            .max_by(|one, other| (*one - from).length().total_cmp(&(*other - from).length()))
            .unwrap_or(from)
    };
    let one_end = farthest_from(start);
    let other_end = farthest_from(one_end);
    let along = other_end - one_end;
    let off_the_line = |point: Point| along.cross(point - one_end).length();
    let third = points
        .iter()
        .copied()
        .max_by(|one, other| off_the_line(*one).total_cmp(&off_the_line(*other)))
        .unwrap_or(one_end);
    let normal = along.cross(third - one_end);
    let length = normal.length();
    if length == 0.0 {
        return 0.0;
    }
    points
        .iter()
        .map(|point| (normal.dot(*point - one_end) / length).abs())
        .fold(0.0, f64::max)
}

/// `Err(why)` when a finer mesh of the zero ladder was refused because it found
/// the space and the placement took it away, and `Ok(())`, which stops the
/// ladder, for any other refusal.
///
/// The doc comment of [`how_much_space_a_curved_solid_encloses`] says why.
fn hand_on_a_refusal_that_found_the_space(why: Error) -> Result<()> {
    if matches!(why, Error::PlacingItTookAwayTheSpaceItsMeshEncloses { .. }) {
        return Err(why);
    }
    Ok(())
}

/// The solid meshed as coarsely as a mesh gets, or — when that mesh's own sides
/// cross, or two of its patches fold onto one side — as coarsely as it can be
/// meshed without either.
///
/// **A chord the sweep chose is not a side of the solid.** At
/// [`as_coarse_as_a_mesh_gets`] an arc is stepped a quarter turn at a time, and
/// the chord of a quarter turn runs up to `0.29` of a radius inside it. A hole
/// sitting in that room is cut in two by it, and the flat face's boundary
/// crosses itself on the way to the triangulator. That is the sweep refusing a
/// sound solid for a sampling choice of its own: twenty KiCad LED files, a disc
/// with its pin holes near the rim, stopped here (`docs/decisions.md` §187.2).
///
/// So a crossing is asked again with less room, halving from the width of the
/// box round the solid — the largest a sag can usefully be — down. A chord at
/// deflection `d` strays at most `d` from its curve, so two pieces of boundary
/// the solid keeps apart by more than twice that cannot be made to cross by the
/// mesh; halving reaches such a `d` for any gap the solid really has.
///
/// # A fold is asked again the same way
///
/// [`Error::TwoPatchesOfTheMeshDrewTheSameSide`] is the sweep's sampling too.
/// A frustum and a block whose steep face crosses it in two arcs keep a piece of
/// each face between those arcs, and at the coarsest each arc is two chords
/// with one point between them. Both faces then draw the side between the two
/// middle points, and four triangles use it. A finer mesh gives each arc more
/// points and the two faces stop drawing the same side (`docs/decisions.md`
/// §285.4). The zero ladder in [`how_much_space_a_curved_solid_encloses`]
/// already retried a fold one step on (§208.3); this is the same retry at the
/// first mesh. It is asked down to a floor of its own, for the reasons under
/// "What stops a fold".
///
/// # Why this is not a fixed finer deflection
///
/// Any constant has a solid its chords still cut across — a hole half as far
/// from the rim — and a fixed fine one taxes every solid at every test boundary
/// for the handful that need it. This costs a solid that meshes at the coarsest
/// nothing at all, which is every solid the gate built before this was written.
///
/// # What stops it, and why stopping is safe
///
/// **Only a mesh the triangulator took is ever used**, so where the halving
/// stops decides what it costs and never what it lets through: a solid that
/// stops early is refused, never passed.
///
/// A crossing stops below half the distance between the two closest corners of
/// the solid that are not in the same place — the finest feature the sweep can read
/// without meshing. That is **a cost bound and not a proof that finer would not
/// help**: a wall thinner than half its nearest corner pair, between two arcs
/// whose corners sit far apart, would still be refused, as a crossing. The model's own linear tolerance was the first floor tried, and it is
/// the one with an argument behind it — two pieces of boundary closer than it
/// are one place — but a solid whose boundary really does cross, which is what
/// the sweep is for, then walks some thirty halvings down to meshes of millions
/// of triangles: `builder.rs`'s tube with a hole as wide as its wall took
/// 16.6 seconds to be refused, and by the triangle cap rather than by its own
/// crossing. At the corner floor the same test takes 0.010 seconds. `Deflection::of` refusing a length that is not positive,
/// [`Error::DeflectionBelowWhatAMeshCanReach`],
/// [`Error::MoreMeshPointsThanASolidWillHold`] and a patch with more triangles
/// than the triangulator will hold each end it too.
///
/// **So does a touch where the rung before touched** (Q1866). Two loops of one
/// face refused as touching at one place at two rooms in a row, the coarsest
/// mesh counting as the rung before the top, are taken to touch there at every
/// room, and the finer refusal is handed back. A corner of the solid is where
/// it is at every room, and so is a chord point every halving keeps, so a touch
/// at one of them comes back. The place is compared, never the corner and side
/// numbers, which count the points along the edges and grow every rung.
/// `builder.rs`'s tube with a hole as wide as its wall walks each cap's two
/// circles the same way round from the seam's foot, and touches there at every
/// room. Built `2⁻³⁵` tall, its corner floor is some thirty-seven halvings
/// down, and the walk had not ended after a minute and a half: the rung at
/// room `4.8e-7` alone took 36 seconds. Now it is refused at the top rung.
///
/// That is again **a cost bound and not a proof**: a touch a third, finer mesh
/// would clear is refused. A throwaway probe of this ladder over
/// `just t -p rcad-topo`, the rest of the workspace and `just test-slow`, at
/// `bd9d330f`, saw 56 ladders with a rung refused for loops touching. The 51 a
/// finer mesh cleared were each cleared by the very next rung, never after two
/// touches in a row; in the 5 refused, every two touches in a row were at one
/// place on one face. Of a solid outside those suites it says nothing. Nor does
/// it catch a touch that moves: the same tube wound as a sound tube is, each
/// hole walked against its rim, touches at a chord point that creeps towards
/// the seam's foot as the room halves, and still walks towards its floor.
///
/// # What stops a fold
///
/// **The corner floor is argued for chords that cross, and a fold is not
/// that.** A chord at deflection `d` strays at most `d`, so a crossing goes once
/// `d` is under half the gap: linear in the gap. A fold is one side two patches
/// both draw through their insides. A point a patch puts inside is its own, so
/// both ends of that side are points the two faces share, on the edges between
/// them, and where those are two curves apart the side spans the gap between
/// them. It goes once one of the two patches will not draw a side that long and
/// puts a point inside instead. `rcad-mesh` takes a point inside wherever a
/// triangle's charge is over `d`, and charges a side `L` long across a curved
/// surface at least `L²/(3ρ)`, with `ρ` the radius
/// [`how_flat_a_curved_face_gets`] reads for it; so such a side is gone once
/// `d < L²/(3ρ)`. **Quadratic in the gap**, where the corner
/// floor is linear: the bulge a curved face has over a gap `L` is about
/// `L²/(8ρ)`, which is small beside `L` whenever the face is wide beside the
/// gap.
///
/// The ball less a pointed cone in `a_cone_hollowed_along_its_axis.rs` is that
/// case: a ring of the ball's band and the cone's band between two circles 2
/// apart, on a ball of radius 3. Both bands drew the same sides from circle to
/// circle at every deflection down to the corner floor of 1 and were refused;
/// the ball takes a point inside below `4/9`, and at 0.4375 the ring measures
/// its closed form.
///
/// So a fold is asked down to `closest²/(6ρ)` — half of `closest²/(3ρ)`, so
/// that the halving lands on a rung below it — with `closest` standing in for
/// the gap the way it does for a crossing, and `ρ` the flattest radius any
/// curved face of the solid bends at ([`how_flat_a_curved_face_gets`]). Never
/// above the corner floor, and the corner floor alone when a face is on a
/// surface that function has not been taught. That is again **a cost bound and
/// not a proof**: two curves closer than any two corners still fold past it.
/// What it costs is a mesh whose sides across the flattest face are about
/// `closest / √2`, which is the size of the finest feature the corners show.
///
/// **And never more than [`HALVINGS_A_FOLD_IS_ASKED_DOWN_TO`] halvings below
/// the top**, whatever `closest` says. The bar `2⁻⁴⁰` wider than `1`, less
/// the tilted block, in `an_ellipse_start_between_two_places.rs` leaves a fin
/// outside `x = −17` that is `2⁻⁴⁰` thick and `2.7e-6` across. The block's
/// strip of side between the fin's two corners has only straight edges, and the
/// bar's wall draws the same side over it, from `(−17, −1.35e-6, 0)` to
/// `(−17, 1.35e-6, 2.558)`. The floor above put the last rung some 53 halvings
/// below the top. The fold was the same at every rung from 5 down to `1.2e-3`
/// and 24672 points, each rung cost 2.5 to 5 times the last, and both policies
/// were still meshing after 465 s (Q1869). With the cap the last rung is
/// `6.1e-4`, and both are refused within 1.8 s.
///
/// The cap is **a cost bound and not a proof**: a fold that only a mesh finer
/// than the fourteenth rung would split is refused, and nothing here says no
/// solid has one. It was set from two suites and no more. A throwaway probe of
/// this ladder over `just t -p rcad-topo`, 2087 tests, and `just test-slow`,
/// 73, saw it clear a fold 469 and 28 times, never past the ninth rung — room
/// `7.8e-3` under a top room of `2`, in
/// `every_answer_over_a_block_turned_against_a_bar_is_a_solid_that_validates`
/// — at `1d3cd744`, and the same again on `bb8f67d6` with the cap merged in,
/// where the cap ended a ladder on the fin and nowhere else. Thirteen halvings
/// is five rungs past the ninth, so the cap refuses no fold those two suites
/// saw cleared. Of a solid outside them it says nothing.
///
/// **A stop on a fold no finer mesh can split was tried first, and not taken**
/// (Q1869). It refused a fold at once when every triangle using the side had
/// corners no finer mesh moves, and was charged no more than the least room
/// any face of the solid is cut in: a triangle charged under its room is not
/// split, and a flat face with straight edges is cut the same at every
/// deflection. On the fin that does not hold. The wall's triangle from
/// `(−17, 1.35e-6, 2.558)` to `(−17, −1.35e-6, 0)` to `(−17, −1.35e-6, 2.558)`
/// is charged `2.5e-12`, under that least room of `3.6e-12`, but the bar's long
/// top ellipse runs between its two upper corners: three points along it at
/// the coarsest, and a walk of it at the least room is refused for too many
/// points. The triangle's circle in the wall's chart has a radius near `1.46`,
/// so a point a finer mesh adds along that ellipse may land inside it and flip
/// the triangle away by Delaunay with no charge changing, and the stop had no
/// way to rule that out. Nor did it cover the chart's own scaling
/// (`rcad-mesh`'s `how_this_domain_is_scaled`), or a cut placed elsewhere in
/// the face. It could not show the fold stays, and in both runs above it never
/// fired.
///
/// **A fold alone ends it in at most fourteen rungs.** A solid that folds at
/// every deflection walks the halving from half the box's width down to the
/// higher of the two floors, about `log₂(3ρ·width/closest²)` rungs or
/// fourteen, whichever is fewer, and is refused with the finest fold. The floor
/// of the rung after a crossing is the corner floor, so a solid that crosses at
/// some deflections and folds at others can go further. The three refusals that
/// end a crossing's ladder before its floor end this one too, and so does
/// `Deflection::of` refusing a room that is not above zero, as on a box with no
/// width.
///
/// Whichever ends it, **the finest mesh's crossing or fold is what is handed
/// back**. A face that meshed at a finer deflection was not what stopped the
/// solid, so a coarse refusal can name a face whose only fault was the sweep's
/// own chords.
/// That happened: on `LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step` the coarse
/// mesh named the disc `#223`, which meshes from 0.869 down (§192.2).
///
/// # Not a self-crossing check, before or after
///
/// A mesh whose sides do not cross says the chords do not cross; it does not
/// say the solid's own boundary does not. That was as true of the coarse mesh
/// before this retried, and nothing here claims more.
///
/// # Errors
///
/// The finest mesh's crossing or fold when no mesh down to its floor clears it,
/// or when two meshes in a row have one face's loops touch at one place — and
/// each floor, and that stop, is a cost bound, so a finer mesh might have
/// cleared it — and a finer mesh's refusal when it is about something else,
/// which is the solid's next blocker and the one worth naming.
fn the_coarsest_mesh_whose_sides_do_not_cross(
    view: SolidView<'_>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Mesh> {
    let mut finest_refusal = match view.mesh(as_coarse_as_a_mesh_gets(), policy, log) {
        Err(why) if a_finer_mesh_may_clear(&why) => why,
        answer => return answer,
    };
    let Some((low, high)) = view.box_around_it() else {
        return Err(finest_refusal);
    };
    let Some(closest) = how_close_two_corners_come(view) else {
        return Err(finest_refusal);
    };
    let top = {
        let (low, high) = (low.coordinates(), high.coordinates());
        (0..3).fold(0.0, |widest: f64, axis| widest.max(high[axis] - low[axis])) / 2.0
    };
    let finest_for_a_crossing = closest / 2.0;
    let finest_for_a_fold = match how_flat_a_curved_face_gets(view, [low, high]) {
        Some(flattest) if flattest > 0.0 => {
            finest_for_a_crossing.min(closest * closest / (6.0 * flattest))
        }
        _ => finest_for_a_crossing,
    }
    .max(top / f64::powi(2.0, HALVINGS_A_FOLD_IS_ASKED_DOWN_TO));
    let finest = |refusal: &Error| {
        if its_sides_cross(refusal) {
            finest_for_a_crossing
        } else {
            finest_for_a_fold
        }
    };
    let mut room = top;
    while room >= finest(&finest_refusal) {
        let Ok(less) = Deflection::of(room) else {
            break;
        };
        match view.mesh(less, policy, log) {
            Ok(mesh) => return Ok(mesh),
            Err(why) if they_touch_where_they_touched_before(&finest_refusal, &why) => {
                finest_refusal = why;
                break;
            }
            Err(why) if a_finer_mesh_may_clear(&why) => finest_refusal = why,
            Err(
                Error::DeflectionBelowWhatAMeshCanReach { .. }
                | Error::MoreMeshPointsThanASolidWillHold { .. }
                | Error::CannotMeshAFace {
                    why: rcad_mesh::CannotTriangulate::MoreTrianglesThanAPatchWillHold { .. },
                    ..
                },
            ) => break,
            Err(why) => return Err(why),
        }
        room /= 2.0;
    }
    Err(finest_refusal)
}

/// How many times [`the_coarsest_mesh_whose_sides_do_not_cross`] halves its
/// room below half the box's width before it stops asking a fold again: a fold
/// is asked at that top room and at most thirteen rungs under it, and refused
/// with the finest fold when every one folds. A cost bound and not a proof;
/// that function's doc says what it was set from, under "What stops a fold".
const HALVINGS_A_FOLD_IS_ASKED_DOWN_TO: i32 = 13;

/// Whether a finer mesh may clear what refused this one, because it is about
/// where the mesh's own chords and cuts fell and not about the solid: its sides
/// crossing, or two of its patches drawing one side through their insides.
fn a_finer_mesh_may_clear(why: &Error) -> bool {
    its_sides_cross(why) || matches!(why, Error::TwoPatchesOfTheMeshDrewTheSameSide { .. })
}

/// Whether two meshes, one finer than the other, were both refused for two
/// loops of one face touching at one place.
///
/// The place, and never the corner or side numbers, which count the points
/// each mesh put along the edges and grow with every rung.
fn they_touch_where_they_touched_before(before: &Error, now: &Error) -> bool {
    matches!(
        (before, now),
        (
            Error::CannotMeshAFace {
                face,
                why: rcad_mesh::CannotTriangulate::LoopsTouch { at, .. },
            },
            Error::CannotMeshAFace {
                face: face_now,
                why: rcad_mesh::CannotTriangulate::LoopsTouch { at: at_now, .. },
            },
        ) if face == face_now && at == at_now
    )
}

/// The distance between the two closest corners of the solid that are not in
/// the same place, or `None` when it has fewer than two such corners.
///
/// Corners in the same place are counted once, whichever vertex holds them:
/// two vertices on one point are one place, and a distance of zero would stop
/// the halving before it started for a reason that is not a feature.
fn how_close_two_corners_come(view: SolidView<'_>) -> Option<f64> {
    let mut corners: Vec<[f64; 3]> = view
        .faces()
        .flat_map(|face| {
            face.wires()
                .flat_map(WireView::points)
                .map(|point| point.coordinates())
                .collect::<Vec<_>>()
        })
        .collect();
    corners.sort_by(|one, other| {
        (0..3)
            .map(|axis| one[axis].total_cmp(&other[axis]))
            .find(|order| order.is_ne())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    corners.dedup();
    let mut closest: Option<f64> = None;
    for (position, one) in corners.iter().enumerate() {
        for other in &corners[position + 1..] {
            let apart = (0..3)
                .map(|axis| (one[axis] - other[axis]).powi(2))
                .sum::<f64>()
                .sqrt();
            closest = Some(closest.map_or(apart, |so_far| so_far.min(apart)));
        }
    }
    closest
}

/// The largest `ρ` such that `rcad-mesh` charges a side `L` long across any
/// curved face of the solid at least `L²/(3ρ)`, or `None` when a face's
/// surface is one this has not been taught. Zero when every face is flat.
///
/// Each is read off how `rcad-mesh`'s `how_far_a_flat_piece_strays` charges
/// a triangle, which `a_step_that_stays_within` turns round:
///
/// - **A ball or a pipe** is charged `L²/(3r)`, `L` the whole chord on a ball
///   and the chord across the axis on a pipe, so `ρ` is its radius.
/// - **A cone** is charged `L²/(2·w·√(1 + t²))` across a ruling, `w` the widest
///   the patch is from the axis, so `ρ = w·√(1 + t²)` covers it with room. The
///   face lies inside `within`, and a point of the cone is its slope times how
///   far up the axis it is from the axis, which over a box is largest at one of
///   its corners; so the widest corner of `within` bounds `w`.
/// - **A torus** is charged `a²/(3r) + b²/(2S)` for a side `a` round the tube
///   and `b` round the ring, `S ≤ R + r`. Its chord is at most `a + b`, so
///   `L² ≤ 2(a² + b²)` and the charge is at least `L²/(6(R + r))`: `ρ` is
///   `2(R + r)`.
fn how_flat_a_curved_face_gets(view: SolidView<'_>, within: [Point; 2]) -> Option<f64> {
    let [low, high] = within.map(|corner| corner.coordinates());
    let corners_of_the_box = (0..8).map(|which: usize| {
        let pick = |axis: usize| {
            if which >> axis & 1 == 0 {
                low[axis]
            } else {
                high[axis]
            }
        };
        Point::new(pick(0), pick(1), pick(2))
    });
    let mut flattest = 0.0_f64;
    for face in view.faces() {
        let radius = match face.surface() {
            Surface::Plane(_) => continue,
            Surface::Sphere(ball) => ball.radius(),
            Surface::Cylinder(pipe) => pipe.radius(),
            Surface::Torus(ring) => 2.0 * (ring.ring_radius() + ring.tube_radius()),
            Surface::Cone(cone) => {
                let up = cone.axis().direction();
                let widest = corners_of_the_box
                    .clone()
                    .map(|corner| cone.radius_at(up.dot(corner - cone.apex())))
                    .fold(0.0, f64::max);
                widest * cone.slope().hypot(1.0)
            }
            _ => return None,
        };
        flattest = flattest.max(radius);
    }
    Some(flattest)
}

/// Whether the triangulator refused a face because two pieces of its boundary
/// met where they should not — the refusals a chord cutting across a hole, or
/// across another chord, comes back as.
///
/// **`CornersCoincide` is not one of them**, because a finer deflection cannot
/// cure it. Every corner of a loop sits on that loop's own edges, so two of them
/// on one point of the chart means the wire meets itself there, or the chart
/// sends two places to one — a seam used twice, two poles rounding onto the
/// centre. A wire that sets off from one corner of the model twice, and is two
/// loops touching there, no longer gets this far: the mesher cuts it apart
/// first (`docs/decisions.md` §364). What meets itself here and is still
/// refused is a slit, a loop that passes a pole, or one whose two visits are
/// not one point of the chart. All of it is fixed by the solid, not by where
/// the chords fall. The third
/// way, two chord points of different edges landing on the same double by
/// accident, needs those edges within a rounding of each other, far below the
/// halving's floor of half the closest corner pair, so the halving could only
/// cure it by luck. Measured before §193.1 cut the LED domes open: of the 1364
/// finer meshes the whole test suite asked for, the 886 that refused
/// `CornersCoincide` never gave way to a mesh; and all 24 sweeps in
/// `rcad-step`'s import tests that reached the floor refused it again at 0.005,
/// some forty times finer, only at a later corner (§193.2).
fn its_sides_cross(why: &Error) -> bool {
    matches!(
        why,
        Error::CannotMeshAFace {
            why: rcad_mesh::CannotTriangulate::BoundaryCrossesItself { .. }
                | rcad_mesh::CannotTriangulate::LoopsTouch { .. },
            ..
        }
    )
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
///
/// The fuse's [`the_place_the_model_points_at`](crate::fuse) reads this same
/// number rather than copying it, because the two checks have to agree: a solid
/// this file lets through is one the fuse has to be able to bake (§75.6).
pub(crate) const MEASURING_SLACK: f64 = 16.0;

/// Every corner on the plane, the outer wire winding with the normal and the
/// holes against it, so the material is on one consistent side.
fn check_flat_face(
    model: &Model,
    view: FaceView<'_>,
    plane: &Plane,
    allowed_wobble: f64,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<()> {
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
        let winding = winding_of(model, view, wire, plane.normal(), policy, log)?;
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
/// # Where the arithmetic is
///
/// The two references it takes — the longitude first and the latitude where the
/// longitude says exactly nothing — and why one of them was not enough are
/// [`which_way_a_closed_crossing_turns`]'s, along with why it runs in interval
/// arithmetic rather than against `0.0` and what its exact level under that is
/// for (§122). All this function does with the answer is read the material's side
/// off it.
///
/// # Errors
///
/// [`Error::CannotTellWhichWayAClosedEdgeRuns`] when that rule comes back
/// [`Decision::Undecidable`]. A predicate that cannot tell is a result and not a
/// failure; naming which way an edge runs is an operation, and an operation with
/// nothing left to return is a failure (§2.1, §4.1). It is what a hemisphere
/// sitting on a pipe of the same radius and very nearly the same axis gives:
/// the wire round that crossing is shorter than the rounding in the cross
/// product that built it, so there is no sign there to read (§111).
///
/// [`Invalid::ClosedEdgeBetweenTwoFlatFaces`] if neither surface is curved. Two
/// planes cross in a straight line, which does not close, so no closed edge is
/// named by a flat pair. A run that carries the curved surface itself is not a
/// flat pair at all, and [`which_two_surfaces_a_run_is_between`] is where the
/// caller swaps it in — this is bad caller input rather than a broken
/// invariant, because `Model` derives `Deserialize` and nothing upstream of
/// this check has confirmed the edge's own claim is possible
/// (`docs/decisions.md` §39.7 item 4, not §16.11 — that section is about the
/// dynamic-range check being a ratio and says nothing about panics on a
/// caller's bytes; the rule is CLAUDE.md's).
///
/// # Panics
///
/// If `material_side` is neither [`Decision::Pos`] nor [`Decision::Neg`].
/// This one *is* a broken internal invariant and not caller input:
/// [`Model::validate`](crate::Model::validate)'s own
/// `check_closed_edges_are_walked_the_way_their_surfaces_wind` passes
/// [`Decision::Neg`] when it reads a side off a wire or knows a cap's, and
/// otherwise a value [`which_side_of_it_this_face_lies_on`] or
/// [`which_side_of_a_surface_a_run_names_this_face_lies_on`] read off a real
/// corner, and both hand back [`the_side_this_face_lies_on`] or
/// [`the_side_this_face_lies_on_with_a_picker`], whose own `Ok(Some(_))` is
/// never anything but [`Decision::Pos`] or [`Decision::Neg`] — each filters
/// `Zero` out and turns `Undecidable` into an `Err` before it can reach here.
///
/// If [`which_way_a_closed_crossing_turns`] grows a fifth answer. [`Decision`]
/// is `#[non_exhaustive]`, so a last arm is forced; a fifth kind of answer to
/// "which way does it run" is this crate not having been taught something, which
/// is a broken invariant and not a caller's bytes.
pub(crate) fn a_closed_crossing_is_walked_the_way_it_is_stored(
    pair: [&Surface; 2],
    outward: [Vector; 2],
    at: Point,
    material_side: Decision,
) -> Result<Option<bool>> {
    let Some(curved) = the_curved_one_of(&[*pair[0], *pair[1]]) else {
        return Err(Invalid::ClosedEdgeBetweenTwoFlatFaces.into());
    };
    let turning = which_way_a_closed_crossing_turns(
        outward,
        gradient_of(&curved, at),
        the_way_it_is_parametrised_about(&curved, at),
    );
    let runs_along_stored = match turning {
        Decision::Pos => true,
        Decision::Neg => false,
        // The wire has no length at all, which is the two surfaces touching
        // rather than crossing at `at` — and at a pole the longitude is itself
        // exactly zero, so the latitude is too and this is the answer there for
        // the same reason (§38.3). Nothing is guessed: the caller that writes an
        // edge refuses, and the caller that reads one back skips, the same way it
        // skips a face with no corner off its partner.
        Decision::Zero => return Ok(None),
        Decision::Undecidable { enclosure } => {
            return Err(Error::CannotTellWhichWayAClosedEdgeRuns { enclosure });
        }
        other => panic!("a fifth answer out of the closed-edge rule: {other:?}"),
    };
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

/// Which side of the face across a closed edge this face lies on next to that
/// edge, as its own corners say. `at` is the edge's corner where it is stored.
///
/// **A corner's own sign is that side only where the partner does not cross
/// the face again between them.** Where the two surfaces cross in two closed
/// curves, one each side of a flat half-space facing along their axis, a face
/// bounded by one of them can run on across the other, which is no edge of it.
/// A bar's wall from below a pointed cone's apex up to the cone's own nappe
/// crosses the other nappe on the way: its corner at the bottom is inside the
/// cone, and next to the edge the wall is outside it. Read off that corner, the
/// bar and the cone's union was refused
/// [`Invalid::AClosedEdgeIsWalkedAgainstItsSurface`] (row Q1190).
///
/// So a corner past the half-space picking the edge's curve out says the side a
/// step from the curve towards that half-space meets
/// ([`what_a_corner_past_the_picker_says`]), and every other corner says its own
/// sign, as before. That holds for any face on the surface bounded by the
/// curve, whatever else of the partner it crosses: in front of the half-space
/// the partner crosses the surface in this curve alone, so a corner there is
/// on the side it reads, and everything past it is on the curve's side towards
/// it. `arrange.rs`'s `whether_past_the_picker` reads a point of the fuse's
/// arrangement the same way.
///
/// `None` when every corner of it is on that surface and nothing says — a bar's
/// cap has its one corner on the wall, and that is the case §36.6 already names.
fn which_side_of_it_this_face_lies_on(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    across: FaceView<'_>,
    at: Point,
    policy: &Policy,
    log: &mut ProvenanceLog,
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
    let trimming = Trimming {
        surface,
        corners_on_it,
    };
    let Some(past) = what_a_corner_past_the_picker_says(
        &face.stored_surface(),
        &across.stored_surface(),
        at,
        policy,
        log,
    )?
    else {
        return the_side_this_face_lies_on(face, &trimming);
    };
    the_side_this_face_lies_on_with_a_picker(
        face,
        &trimming,
        &past.moved_into_place(&face.location()),
    )
}

/// The flat half-space picking a closed edge's curve out of the two its
/// surfaces cross in, and the side of the partner a step from the curve
/// towards that half-space's flat surface meets.
struct PastThePicker {
    /// Everything on or in front of it holds the edge's curve and not the
    /// other one.
    picker: HalfSpace,
    /// The partner's sign a step past the curve towards the flat surface:
    /// [`Decision::Zero`] where the two surfaces touch rather than cross there.
    side: Decision,
}

impl PastThePicker {
    /// The same, with the half-space moved where the face sits. A movement
    /// keeps which side of the curve a step goes to, so the side comes along
    /// unchanged.
    fn moved_into_place(&self, movement: &Isometry) -> Self {
        Self {
            picker: self.picker.transformed(movement),
            side: self.side,
        }
    }
}

/// What a corner past the half-space picking out the curve through `at` says,
/// for a face on `surface` with `partner` across, all three where stored; `None`
/// where every corner says its own sign.
///
/// **Only where both curves go right round one axis and the half-space faces
/// exactly along it**, which are the pairs `arrange.rs`'s
/// `whether_past_the_picker` names: a pipe and anything [`intersect`] cuts it in
/// two such curves by, the axis being the pipe's; a cone and a pipe, a second
/// cone or a ball on its axis, the axis being the cone's; and a ball and a cone
/// whose axis runs through its centre, the axis again being the cone's. Each
/// curve then cuts the surface in two, the half-space holds the edge's curve
/// and not the other, and everything of the surface past the half-space's flat
/// surface is on the curve's side towards it: on a cone through the apex, which
/// joins the two nappes, and on a ball as the cap past the flat surface.
///
/// The half-space is the one [`intersect`] names the curve by, found the one
/// way that is looked up ([`the_picker_naming_the_circle_through`]). A pair
/// `intersect` refuses to name, a cone leaning off a pipe's axis as a
/// countersink on a turned bore comes back from a file, has no half-space, and
/// its corners say their own signs as they always did.
///
/// The side is the partner's gradient against the step from `at` towards the
/// flat surface, laid onto the surface: `|g|²·d − (g·d)·g` for the surface's
/// gradient `g` and `d` the half-space's normal turned round, and `d` itself on
/// a pipe, whose gradient is square to its axis. On a cone that runs along the
/// ruling, and on a ball along the line of longitude.
///
/// # Errors
///
/// [`Error::Undecidable`] when [`intersect`] cannot settle the two surfaces.
fn what_a_corner_past_the_picker_says(
    surface: &Surface,
    partner: &Surface,
    at: Point,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<PastThePicker>> {
    if !matches!(
        partner,
        Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_)
    ) {
        return Ok(None);
    }
    let axis = if let Surface::Cylinder(pipe) = surface {
        pipe.axis()
    } else if let (Surface::Cone(cone), _) | (Surface::Sphere(_), Surface::Cone(cone)) =
        (surface, partner)
    {
        cone.axis()
    } else {
        return Ok(None);
    };
    let named = match the_picker_naming_the_circle_through(
        surface,
        partner,
        PredicatePoint::from(at.coordinates()),
        policy,
        log,
    ) {
        Ok(named) => named,
        Err(Error::Geometry(_)) => return Ok(None),
        Err(other) => return Err(other),
    };
    let Some(Picker::InFrontOf(picker)) = named else {
        return Ok(None);
    };
    if !lie_along_one_line(picker.normal().components(), axis.direction().components()) {
        return Ok(None);
    }
    let down = picker
        .normal()
        .components()
        .map(|along| Interval::exactly(-along));
    let step = if matches!(surface, Surface::Cylinder(_)) {
        down
    } else {
        let across = the_way_out_of(surface, at);
        let (squared, leaning) = (dot(across, across), dot(across, down));
        core::array::from_fn(|index| squared * down[index] - leaning * across[index])
    };
    Ok(Some(PastThePicker {
        picker,
        side: dot(the_way_out_of(partner, at), step).sign(),
    }))
}

/// [`the_side_this_face_lies_on`], with a corner past `past`'s half-space
/// saying `past`'s side rather than its own sign.
///
/// # Errors
///
/// [`Invalid::AFaceStraddlesSomethingTrimmingIt`] when two corners say
/// opposite sides, and [`Error::Undecidable`] when a corner's side of the
/// half-space or of the partner, or the side `past` says, could not be settled.
fn the_side_this_face_lies_on_with_a_picker(
    face: FaceView<'_>,
    trimming: &Trimming,
    past: &PastThePicker,
) -> Result<Option<Decision>> {
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
        let against_the_picker = past.picker.side_of_point(at);
        if let Decision::Undecidable { enclosure } = against_the_picker {
            return Err(gave_up_at(SIDE_OF_A_HALF_SPACE, at, enclosure));
        }
        let side = if against_the_picker == Decision::Neg {
            past.side
        } else {
            assert!(
                matches!(against_the_picker, Decision::Pos | Decision::Zero),
                "a corner came back with a decision nothing here knows: {against_the_picker:?}"
            );
            side_of_surface(&trimming.surface, at)
        };
        if let Decision::Undecidable { enclosure } = side {
            return Err(gave_up_at(
                name_of_surface(&trimming.surface),
                at,
                enclosure,
            ));
        }
        if side == Decision::Zero {
            continue;
        }
        assert!(
            matches!(side, Decision::Pos | Decision::Neg),
            "a corner came back with a decision nothing here knows: {side:?}"
        );
        match found {
            None => found = Some(side),
            Some(already) if already == side => {}
            Some(_) => return Err(Invalid::AFaceStraddlesSomethingTrimmingIt.into()),
        }
    }
    Ok(found)
}

/// The refusal for a sign `named` could not settle at `at`.
fn gave_up_at(named: &'static str, at: PredicatePoint, enclosure: f64) -> Error {
    let mut records = ProvenanceLog::new();
    records.push(Record::new(
        named,
        &[at],
        Level::Exact,
        enclosure,
        Measures::Area,
    ));
    Error::Undecidable { records }
}

/// Which way a ball, a pipe or a cone faces outwards at `at`, as a box, from
/// the numbers it stores with no normalization in it (§31.2): `x` for a ball
/// and `|w|²·x − k·(x·w)·w` for the other two, `x` running from the centre,
/// the axis's origin or the apex, `w` the axis direction, and `k` one for a
/// pipe and `1 + slope²` for a cone.
///
/// # Panics
///
/// On any other kind, which [`what_a_corner_past_the_picker_says`] never hands
/// it.
fn the_way_out_of(surface: &Surface, at: Point) -> [Interval; 3] {
    let from_it = |to: Point| -> [Interval; 3] {
        let (at, to) = (at.coordinates(), to.coordinates());
        core::array::from_fn(|axis| Interval::exactly(at[axis]) - Interval::exactly(to[axis]))
    };
    if let Surface::Sphere(ball) = surface {
        return from_it(ball.centre());
    }
    let (out, along, steepness) = if let Surface::Cylinder(pipe) = surface {
        (
            from_it(pipe.axis().origin()),
            pipe.axis().direction(),
            Interval::exactly(1.0),
        )
    } else if let Surface::Cone(cone) = surface {
        let slope = Interval::exactly(cone.slope());
        (
            from_it(cone.apex()),
            cone.axis().direction(),
            Interval::exactly(1.0) + slope * slope,
        )
    } else {
        panic!("only a ball, a pipe or a cone is read past a picker, not {surface:?}");
    };
    let along = along.components().map(Interval::exactly);
    let (squared, shadow) = (dot(along, along), steepness * dot(out, along));
    core::array::from_fn(|axis| squared * out[axis] - shadow * along[axis])
}

/// Which side of a surface that a run names, and that no face of the solid
/// sits on, this face lies on, as its own corners say.
///
/// The corners on it are the ends of every edge of this face whose run is
/// between something and that surface. They are on it because the run says
/// so, which no rounding can take away ([`Trimming`]). Every other corner is
/// read.
///
/// `named` is the surface **where the face is stored**, and which runs name it
/// is asked there too, of the stored runs and surfaces, because that is the
/// frame the caller found it in. Only the side is read where the face sits,
/// against `named` moved into place, since that is where
/// [`the_side_this_face_lies_on`] reads the corners.
///
/// `None` when every corner of the face is on it, as
/// [`which_side_of_it_this_face_lies_on`] gives for a face across.
fn which_side_of_a_surface_a_run_names_this_face_lies_on(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    named: Surface,
) -> Result<Option<Decision>> {
    let mut corners_on_it = Vec::new();
    for coedge in face.wires().flat_map(WireView::coedges) {
        let edge = coedge.edge();
        let beyond = the_face_across(solid, face, edge)?.unwrap_or(face);
        if which_two_surfaces_a_run_is_between(
            edge.stored_run(),
            [&face.stored_surface(), &beyond.stored_surface()],
        )
        .contains(&named)
        {
            corners_on_it.push(edge.start().key());
            corners_on_it.push(edge.end().key());
        }
    }
    the_side_this_face_lies_on(
        face,
        &Trimming {
            surface: named.transformed(&face.location()),
            corners_on_it,
        },
    )
}

/// Which of the two surfaces a closed edge is between is a ring, when the
/// other is a plane holding the ring's axis: the edge is then one of the two
/// circles round the tube that the plane cuts the ring in.
///
/// Private and asked of nothing but this one pair, so it has no variant to
/// grow. Exhaustive on purpose.
enum RoundTheTube {
    /// The face's own surface, the first of the two, is the ring.
    OnTheRing,
    /// The face's own surface is the plane.
    OnThePlane,
}

/// Whether `names_it`, the face's own surface and the one across a closed
/// edge, are a ring and a plane through its axis, and which is the face's.
fn a_circle_round_the_tube(names_it: [Surface; 2]) -> Option<RoundTheTube> {
    if let [Surface::Torus(ring), Surface::Plane(flat)] = names_it
        && holds_the_axis_of(&flat, &ring)
    {
        return Some(RoundTheTube::OnTheRing);
    }
    if let [Surface::Plane(flat), Surface::Torus(ring)] = names_it
        && holds_the_axis_of(&flat, &ring)
    {
        return Some(RoundTheTube::OnThePlane);
    }
    None
}

/// Whether the whole of `ring`'s axis lies on `flat`, as `holds.rs` settles
/// it: the plane's normal square to the axis direction and the ring's centre
/// on the plane, both exactly as stored.
///
/// `false` when the plane crosses the axis, and `false` when that cannot be
/// settled, which leaves the edge to be read off the face's corners as every
/// other closed edge is.
fn holds_the_axis_of(flat: &Plane, ring: &rcad_geom::Torus) -> bool {
    match how_the_axis_lies_on(flat, ring) {
        Decision::Zero => true,
        Decision::Pos | Decision::Neg | Decision::Undecidable { .. } => false,
        other => panic!("the axis came back with a decision nothing here knows: {other:?}"),
    }
}

/// Whether `coedge` is the one coedge of `face`'s first wire, which is the
/// wire a flat face's material lies inside.
fn is_the_whole_of_its_outer_wire(face: FaceView<'_>, coedge: CoedgeView<'_>) -> bool {
    face.wires().next().is_some_and(|outer| {
        let mut round_it = outer.coedges();
        round_it
            .next()
            .is_some_and(|only| only.key() == coedge.key())
            && round_it.next().is_none()
    })
}

/// Every other circle round the tube of this ring face is walked along its
/// edge the other way from `circle`.
///
/// Every such circle is stored running the way the tube angle grows
/// ([`the_way_it_is_parametrised_about`]). A band of the ring between two of
/// them runs on from one of them and back towards the other, so with its
/// material on the left it walks one the way the tube angle grows and the
/// other against it. Two circles walked the same way bound no band: one says
/// the band runs on round the ring from it and the other says the same.
///
/// # Errors
///
/// [`Invalid::AClosedEdgeIsWalkedAgainstItsSurface`] for a second circle
/// walked the same way, and whatever finding the face across an edge refuses.
fn the_other_circles_round_the_tube_are_walked_the_other_way(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    circle: CoedgeView<'_>,
) -> Result<()> {
    for other in face.wires().flat_map(WireView::coedges) {
        let edge = other.edge();
        if other.key() == circle.key()
            || !matches!(
                edge.stored_run(),
                EdgeRun::RightRoundWhereItsFacesCross | EdgeRun::RightRoundWhereItCrosses { .. }
            )
        {
            continue;
        }
        let Some(across) = the_face_across(solid, face, edge)? else {
            continue;
        };
        let names_it = which_two_surfaces_a_run_is_between(
            edge.stored_run(),
            [&face.stored_surface(), &across.stored_surface()],
        );
        if matches!(
            a_circle_round_the_tube(names_it),
            Some(RoundTheTube::OnTheRing)
        ) && other.orientation_on_its_edge() == circle.orientation_on_its_edge()
        {
            return Err(Invalid::AClosedEdgeIsWalkedAgainstItsSurface.into());
        }
    }
    Ok(())
}

/// The side of one stored plane a ring's face keeps at one of its circles
/// round the axis, as its wire says.
struct SideKept {
    plane: Surface,
    kept: Decision,
}

/// The ring and the plane, when a closed edge of a face stored on `on` is one
/// of the two circles round the axis that a plane square to the ring's axis
/// cuts it in: `on` is a ring and `names_it[1]`, the partner the edge's run
/// names, is that plane.
///
/// **`names_it[0]` need not be the ring.** Where the face across is a pipe, a
/// cone or a ball and the run names the circle's own plane, which is how
/// `rcad-step` reads a circle between two different curved faces and the
/// fuse's `the_plane_of_a_rim_between_two_curved_faces` writes one,
/// [`which_two_surfaces_a_run_is_between`] puts whichever of the two curved
/// faces sorts first beside the plane, and a ring sorts after all three. The
/// edge is still on the ring, since it is an edge of a ring face, and on the
/// plane, since the run says so, so it is still one of those two circles. Read
/// as the pair alone, it was not, and the ring face's side of the plane was
/// read off the other circle's corner: a ring with its rim turned down to a
/// pipe was refused (row Q1173).
///
/// Asked exactly, the way `holds.rs`'s `what_a_ring_face_between_two_planes_holds`
/// reads the same pair, through [`lie_along_one_line`]: every component of the
/// stored normal crossed into the stored axis direction is exactly zero. A
/// plane whose normal is not exactly along the axis is not asked about, and
/// its edges are read off the corners as before.
fn a_circle_round_the_axis(
    on: Surface,
    names_it: [Surface; 2],
) -> Option<(rcad_geom::Torus, Plane)> {
    let (Surface::Torus(ring), Surface::Plane(flat)) = (on, names_it[1]) else {
        return None;
    };
    lie_along_one_line(
        flat.normal().components(),
        ring.axis().direction().components(),
    )
    .then_some((ring, flat))
}

/// What the closed-edge rule is asked with at one closed edge of a face.
struct AskedAt {
    /// The face's own surface and the partner, as the run names them.
    names_it: [Surface; 2],
    /// Each of the two surfaces' gradients, turned the way the solid uses it.
    outward: [Vector; 2],
    /// How the solid uses the partner: as stored when no face of it is on it.
    partner_turn: Orientation,
    /// The edge's corner, where it is stored.
    at: Point,
}

/// What the closed-edge rule is asked with where `face` and `across` meet
/// along `edge`, everything as stored.
fn what_the_rule_is_asked_at(
    face: FaceView<'_>,
    across: FaceView<'_>,
    edge: EdgeView<'_>,
) -> AskedAt {
    let at = edge.stored_ends()[0];
    let outward =
        |used: FaceView<'_>| gradient_of(&used.stored_surface(), at) * used.orientation().sign();
    let names_it = which_two_surfaces_a_run_is_between(
        edge.stored_run(),
        [&face.stored_surface(), &across.stored_surface()],
    );
    let partner = names_it[1];
    let (partner_outward, partner_turn) = if partner == across.stored_surface() {
        (outward(across), across.orientation())
    } else {
        (gradient_of(&partner, at), Orientation::Forward)
    };
    AskedAt {
        names_it,
        outward: [outward(face), partner_outward],
        partner_turn,
        at,
    }
}

/// Which side of `names_it[1]` a face keeps next to a closed edge, read off
/// the way its wire walks the edge.
///
/// This is the closed-edge rule [`a_closed_crossing_is_walked_the_way_it_is_stored`]
/// run backwards, as `holds.rs` reads which side of a plane a ring's face keeps
/// (`docs/decisions.md` §243.3, §243.8). `asked` is what the sweep hands the
/// rule going forwards, and the side comes back turned round when the
/// partner's face is used reversed, because the sweep turns a side round
/// before handing it in.
///
/// Handed back into the rule at the same edge, this side gives back the walk it
/// was read from, so it checks nothing there (§392.2). What it is for is
/// comparing two edges.
///
/// `None` where the rule finds no way round to read, which is the two surfaces
/// touching rather than crossing.
fn the_side_its_wire_keeps(asked: &AskedAt, coedge: CoedgeView<'_>) -> Result<Option<Decision>> {
    let Some(forward_if_it_keeps_the_back) = a_closed_crossing_is_walked_the_way_it_is_stored(
        [&asked.names_it[0], &asked.names_it[1]],
        asked.outward,
        asked.at,
        Decision::Neg,
    )?
    else {
        return Ok(None);
    };
    let walked_forward = coedge.orientation_on_its_edge() == Orientation::Forward;
    let handed = if walked_forward == forward_if_it_keeps_the_back {
        Decision::Neg
    } else {
        Decision::Pos
    };
    Ok(Some(turned_by(handed, asked.partner_turn)))
}

/// The side of `plane` a ring's face keeps at one circle round the axis is the
/// side it keeps at every other circle along the same stored plane.
///
/// # Errors
///
/// [`Invalid::AClosedEdgeIsWalkedAgainstItsSurface`] when an earlier circle
/// along `plane` kept the other side.
fn the_same_side_as_at_its_other_circles(
    sides_kept: &mut Vec<SideKept>,
    plane: Surface,
    kept: Decision,
) -> Result<()> {
    match sides_kept.iter().find(|read| read.plane == plane) {
        Some(read) if read.kept != kept => {
            Err(Invalid::AClosedEdgeIsWalkedAgainstItsSurface.into())
        }
        Some(_) => Ok(()),
        None => {
            sides_kept.push(SideKept { plane, kept });
            Ok(())
        }
    }
}

/// Whether a ring's face has two circles round the axis, along planes square to
/// it, and leaves them going opposite ways round the tube, which is what the
/// sweep then holds it to at those circles in place of a corner.
///
/// **Both bands between two such circles have the same corners**, as §392.2
/// found for circles round the tube. The ring less the two circles is two
/// pieces, and a face whose only edges they are is one of them: the band
/// between the planes, a barrel's wall, or the ring less that band, which a
/// groove turned into the ring's rim leaves. The corner of each circle is on
/// the side of the other circle's plane the band keeps, so read off it, the
/// side kept at a grooved ring's circle came out the band's (row Q1153).
///
/// **What does say it is the other circle.** Each piece runs on round the tube
/// from one of its circles and back towards the other, so it leaves them going
/// opposite ways round the tube ([`which_way_round_the_tube_it_leaves`]).
/// Which way it leaves one is the side of its plane it keeps there, read off the
/// wire, and the side of the pipe of the ring's own radius that [`intersect`]
/// names that circle by. A wire leaving both the same way round bounds neither
/// piece; nor does one with three such circles, since two of them would be
/// left the same way.
///
/// The wire read at one circle is still the only thing saying which piece the
/// face is, and that is right: each piece is a face. Whether the solid round it
/// is sound is the faces across' to say, whose own corners are read as before.
///
/// `false`, leaving every circle to be read as before, when the face has fewer
/// than two such circles, or one of them is where the plane touches the tube,
/// or the pipe names no one circle through its corner.
///
/// # Errors
///
/// [`Invalid::AClosedEdgeIsWalkedAgainstItsSurface`] for two circles left the
/// same way round the tube; whatever [`intersect`] gives for a plane and the
/// ring, and whatever finding the face across an edge refuses.
fn its_circles_round_the_axis_bound_a_band_of_the_tube(
    solid: SolidView<'_>,
    face: FaceView<'_>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    let mut ways_left: Vec<bool> = Vec::new();
    for coedge in face.wires().flat_map(WireView::coedges) {
        let edge = coedge.edge();
        if !matches!(
            edge.stored_run(),
            EdgeRun::RightRoundWhereItsFacesCross | EdgeRun::RightRoundWhereItCrosses { .. }
        ) {
            continue;
        }
        let Some(across) = the_face_across(solid, face, edge)? else {
            continue;
        };
        let asked = what_the_rule_is_asked_at(face, across, edge);
        let Some((ring, flat)) = a_circle_round_the_axis(face.stored_surface(), asked.names_it)
        else {
            continue;
        };
        let Some(up_the_outside) =
            which_way_round_the_tube_it_leaves(&asked, &ring, &flat, coedge, policy, log)?
        else {
            return Ok(false);
        };
        if ways_left.contains(&up_the_outside) {
            return Err(Invalid::AClosedEdgeIsWalkedAgainstItsSurface.into());
        }
        ways_left.push(up_the_outside);
    }
    Ok(ways_left.len() == 2)
}

/// Whether a ring's face leaves a circle round the axis going round the tube
/// the way that runs up its outside and down its inside, "up" being the way
/// the ring's axis is stored pointing.
///
/// On the outer circle of a plane square to the axis, that way round runs up
/// and away from the plane on the side the axis points to; on the inner
/// circle it runs down. So the face goes that way exactly when it keeps the
/// side of the plane the axis points to at an outer circle, or the other side
/// at an inner one. The side is read off the wire ([`the_side_its_wire_keeps`]),
/// and which circle it is off the side of the pipe of the ring's own radius
/// that [`intersect`] names it by (`holds.rs`'s
/// `the_picker_naming_the_circle_through`), the same reading
/// `what_a_ring_face_between_two_planes_holds` tells the two bands apart by.
///
/// `None` when the rule finds no way round to read, which is the plane resting
/// on the tube, or when no one circle certainly keeps the corner.
///
/// # Errors
///
/// [`Error::CannotTellWhichWayAClosedEdgeRuns`] when the rule cannot, and
/// whatever [`intersect`] gives for the plane and the ring.
///
/// `ring` is the face's own surface and `flat` the plane `asked.names_it[1]`,
/// as [`a_circle_round_the_axis`] found them. The side is read against `flat`
/// with the pair the run names, whichever curved surface is first in it, since
/// that pair is what both faces along the edge read its walk by.
///
/// # Panics
///
/// When [`intersect`] names a circle that a plane square to the ring's axis
/// cuts it in by anything but the pipe.
fn which_way_round_the_tube_it_leaves(
    asked: &AskedAt,
    ring: &rcad_geom::Torus,
    flat: &Plane,
    coedge: CoedgeView<'_>,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<bool>> {
    let Some(kept) = the_side_its_wire_keeps(asked, coedge)? else {
        return Ok(None);
    };
    let corner = PredicatePoint::from(asked.at.coordinates());
    let Some(picker) = the_picker_naming_the_circle_through(
        &Surface::Plane(*flat),
        &Surface::Torus(*ring),
        corner,
        policy,
        log,
    )?
    else {
        return Ok(None);
    };
    let outer = matches!(picker, Picker::OutsideAPipe(_));
    assert!(
        outer || matches!(picker, Picker::InsideAPipe(_)),
        "a plane square to a ring's axis names its circles by the pipe, and this one by {picker:?}"
    );
    let keeps_the_side_the_axis_points_to =
        (kept == Decision::Pos) == faces_the_way_its_axis_points(flat, ring);
    Ok(Some(keeps_the_side_the_axis_points_to == outer))
}

/// Whether a plane whose normal lies exactly along a ring's axis points the
/// way the axis does.
///
/// Along one line, the normal is an exact multiple of the axis direction, so a
/// component is zero in both or in neither, and the multiple's sign is whether
/// the signs agree where they are not zero. Nothing is worked out, so nothing
/// rounds.
///
/// # Panics
///
/// When the two are zero along different components, which
/// [`lie_along_one_line`] has shown they are not.
fn faces_the_way_its_axis_points(flat: &Plane, ring: &rcad_geom::Torus) -> bool {
    let (normal, along) = (
        flat.normal().components(),
        ring.axis().direction().components(),
    );
    let axis = (0..3)
        .find(|&axis| along[axis] != 0.0)
        .expect("an axis has a direction");
    assert!(
        normal[axis] != 0.0,
        "a normal along the axis is zero where the axis is not: {normal:?} and {along:?}"
    );
    (normal[axis] > 0.0) == (along[axis] > 0.0)
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
                Decision::Undecidable { enclosure } => {
                    let mut records = ProvenanceLog::new();
                    records.push(Record::new(
                        named,
                        &[point],
                        Level::Exact,
                        enclosure,
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
        Interval::around(coordinate, reach)
            .expect("a positive finite reach makes a box round a finite corner")
    }));
    matches!(
        side_of_surface(surface, within),
        Decision::Zero | Decision::Undecidable { .. }
    )
}

/// Every point where `surface` pinches, each as the box its coordinates round
/// into.
///
/// A ball pinches at its poles, `radius` straight above and below its centre
/// along `z` — the frame is the world's, because a ball has no axis of its own
/// ([`rcad_geom::Sphere::point_at`]), and it is where `Model::add_ball` and the
/// fuse both put them. Adding the radius to the centre rounds, so each pole is
/// the box that sum lands in rather than one `f64`. A cone pinches at its apex,
/// which it stores exactly. A plane, a pipe and a ring pinch nowhere: a ring's
/// tube is always narrower than the ring ([`rcad_geom::Torus::new`] refuses
/// the rest), so it never closes up on its axis.
fn where_it_pinches(surface: &Surface) -> Vec<[Interval; 3]> {
    match surface {
        Surface::Sphere(ball) => {
            let [x, y, z] = ball.centre().coordinates();
            let radius = Interval::exactly(ball.radius());
            vec![
                [
                    Interval::exactly(x),
                    Interval::exactly(y),
                    Interval::exactly(z) + radius,
                ],
                [
                    Interval::exactly(x),
                    Interval::exactly(y),
                    Interval::exactly(z) - radius,
                ],
            ]
        }
        Surface::Cone(cone) => vec![cone.apex().coordinates().map(Interval::exactly)],
        Surface::Plane(_) | Surface::Cylinder(_) | Surface::Torus(_) => Vec::new(),
        other => panic!("a surface nothing here knows the pinches of: {other:?}"),
    }
}

/// Whether a corner widened by `reach` in every direction misses every one of
/// `pinches`.
///
/// Two boxes miss when they miss along one axis, and comparing the ends of two
/// intervals is exact, so this is decided outright and has no undecidable
/// answer to hand back. A reach that is not a finite length proves nothing
/// away, so the corner is kept — the approximation check refuses that length
/// on its own account.
fn provably_away_from_every_pinch(point: Point, reach: f64, pinches: &[[Interval; 3]]) -> bool {
    if !reach.is_finite() {
        return false;
    }
    let corner = point.coordinates().map(|coordinate| {
        Interval::around(coordinate, reach)
            .expect("a finite reach makes a box round a finite corner")
    });
    pinches.iter().all(|pinch| {
        corner
            .iter()
            .zip(pinch)
            .any(|(near, far)| near.hi() < far.lo() || far.hi() < near.lo())
    })
}

/// The pinch point nearest `point`, for the refusal to name.
fn nearest(point: Point, pinches: &[[Interval; 3]]) -> Option<Point> {
    pinches
        .iter()
        .map(|pinch| {
            let [x, y, z] = pinch.map(|along| (along.lo() + along.hi()) / 2.0);
            Point::new(x, y, z)
        })
        .min_by(|one, other| {
            (*one - point)
                .length()
                .total_cmp(&(*other - point).length())
        })
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

/// Which way `wire` winds round `normal`, over the boundary it actually traces.
///
/// [`Decision::Pos`] with it, [`Decision::Neg`] against it, [`Decision::Zero`]
/// when the boundary encloses no area at all.
///
/// # The corner polygon was the wrong thing to wind, and this is the fix
///
/// This used to wind the wire's stored corners and nothing else, and file a
/// face whose corners land on one line as winding against its normal —
/// **however many of them there are**. That was §44.11 item 3's open half, kept
/// open for want of a consumer. The consumer arrived: `rcad-step`'s
/// `a_file_bounded_by_two_half_circles.rs` reads a whole AP203
/// `manifold_solid_brep` for a half-round plate with a half-round rib under it,
/// whose flat top and bottom are each bounded by two half-circles closed by two
/// straights, so their four corners are four points on the `x` axis and the
/// polygon through them sweeps exactly zero. The mapping took every part of
/// that file and `SolidBuilder::build` then refused it for a winding the shape
/// does not have. `a_face_bounded_by_two_half_circles.rs` is the same solid
/// built face by face.
///
/// **What is wound now is the wire's corners plus the points
/// [`rcad_geom::walk`] puts along every curved step**, which is the same
/// boundary the mesher draws — one authority on where an edge is (§27.5). A
/// wire whose every step is straight is its own corner polygon and is wound
/// without walking anything, so a polyhedron pays nothing for this.
///
/// # Why the answer does not depend on how finely it is walked
///
/// It would, if a deflection were simply picked. **A ring's sampled area can
/// come out with the wrong sign, and change to the right one when the same wire
/// is walked one step finer.** The sliver in
/// `a_walk_that_came_back_with_no_points.rs` is bounded by a shallow arc closed
/// underneath by three straights; walked into one inside point its ring sweeps
/// `+3.97e-7`, walked into two it sweeps `−1.86e-7`, and the shape is wound the
/// second way. So a rule that read the sign off whatever points came back would
/// be a guess with a test, and would move when somebody changed how coarse a
/// walk may be.
///
/// **This used to give a different reason and that reason was false.** It named
/// a pair of half-circles — radius `1` walked into one inside point spanning
/// `1.000`, radius `0.99` into two spanning `1.273` — and said a half-pipe's cap
/// was that pair. Both numbers are right and the pair cannot occur.
/// [`rcad_geom::walk`] sets its widest step from `room / reach` with
/// `room = deflection − deviation`, so of two circles drawn in the same place at
/// one deflection the **bigger** is walked into at least as many pieces, never
/// fewer, and the ordering that argument needs never arises.
/// `a_bigger_arc_is_never_walked_into_fewer_pieces_than_a_smaller_one` sweeps
/// radii and deflections through the real walk and says so. Numbers in a
/// paragraph are still a paragraph when they describe something that cannot
/// happen (§16.10).
///
/// So the sign is only taken when the sampled area **outweighs what the
/// sampling could have hidden**. Between one sample point and the next the true
/// boundary strays from the straight piece by at most the deflection asked for,
/// plus however far off the crossing the points themselves may sit
/// ([`PointsAlongACrossing::deviation`](rcad_geom::PointsAlongACrossing::deviation)),
/// and the area of the sliver between a straight piece and a curve that strays
/// by `e` is at most its length times `e`. Summed round the ring that is
/// `perimeter · e`, against which the sampled area is compared before its sign
/// is believed. Fail that and the walk is halved and asked again, starting from
/// the ring's own width — which is [`how_much_space_a_curved_solid_encloses`]'s
/// loop and terminates for the same reason: halving a finite length reaches
/// zero and [`Deflection::of`] refuses a length that is not positive.
///
/// **What is not the same as that loop is where the margin bottoms out.** The
/// deflection halves away but the points' own deviation does not, so the margin
/// never falls below `perimeter · deviation` — and a walked point's deviation is
/// `256 · f64::EPSILON` of the size the crossing was worked out from. So this
/// answers only for a face whose area beats `perimeter · 5.7e-14 · size`, and
/// **any** face thinner than that runs the loop out, not only one that encloses
/// nothing. The volume loop's test has no such floor.
///
/// The walk's own coarseness therefore only decides **which pass answers**,
/// never what the answer is.
///
/// # What it costs the sweep
///
/// Nothing at all for a wire whose every step is straight, which is every wire
/// of a polyhedron and both end faces of the plate below. For a wire with an arc
/// in it, one walk per pass and a polygon sum over what came back.
///
/// Measured on `a_face_bounded_by_two_half_circles.rs`'s plate, whose two caps
/// are the only walked wires in it: the whole of [`Model::validate`] goes from
/// 64.0 µs to 86.4 µs, so about 11 µs a walked wire. Each cap answers on the
/// fifth pass, at a deflection of 0.375 on a plate 6 across, over a ring of
/// eight points.
///
/// **It stays linear in the model.** The passes are a property of one wire's own
/// shape — how thin it is against how far round it goes — and not of how many
/// entities there are, so this is one more per-face cost in a sweep that was
/// already a pass over every face (§4.3).
///
/// **That is the cost of a wire that answers, and the other path is not like
/// it.** A wire too thin to beat the margin is walked again at every halving
/// until the walk itself gives up — the sliver in
/// `a_walk_that_came_back_with_no_points.rs` takes twenty-seven passes, and a
/// wire that keeps refining reaches
/// [`MOST_POINTS_ALONG_ONE_CROSSING`](rcad_geom::MOST_POINTS_ALONG_ONE_CROSSING),
/// a million points, before it does. Each of those passes walks the wire again
/// and sums a polygon over what came back; the polygon used to be summed over a
/// *copy* of the points, which is twenty-five megabytes a pass at the far end,
/// and [`AreaSweptRoundTheNormal`] borrows them now. This is the
/// untrusted-import boundary, so it is a shape a file can ask for.
///
/// Reversing a face reverses this ring and negates every term of the sum
/// exactly, so a face put on back to front is caught exactly as it was before.
///
/// # What is still wound over corners alone
///
/// [`check_flat_face`] does not ask about a boundary with fewer than three
/// corners at all, and that is unchanged. A round cap's boundary is one closed
/// edge and one corner; a face bounded by an arc and its chord has two. Which
/// way those point is asked over the mesh, by
/// [`every_side_of_the_skin_is_used_both_ways`].
///
/// # The sign itself still goes through the cascade
///
/// The quantity decided is the ring's vector area dotted with the normal. That
/// sum cancels — a wire far from the origin adds and subtracts numbers far
/// larger than the area they are measuring — so reading the sign off an `f64`
/// total is a coin toss near the point where the sign turns over. A sweep that
/// got this wrong would pass a solid with its material on the wrong side, or
/// refuse a sound one, so it goes through the cascade like every other
/// geometric decision here (decisions.md §2.1).
///
/// # Errors
///
/// [`Error::Undecidable`] when the cascade cannot settle the sign, and when no
/// walk fine enough to outweigh what it hides can be had — a face that encloses
/// nothing at all is the shape that runs the loop out.
///
/// For the exact `f64` coordinates a model stores the cascade itself only gives
/// up when a coordinate product overflows or falls into the subnormals. This
/// used to add "which is what the dynamic-range check exists to reject", and
/// that check does not reject it: it is a **ratio** test and is blind to
/// absolute magnitude, so a uniformly tiny model passes with a ratio of one.
/// This matters more here than elsewhere, because `Model` is `Deserialize` and
/// `validate` is the untrusted-import boundary — which is exactly where such
/// coordinates arrive. `docs/decisions.md` §16.11.
fn winding_of(
    model: &Model,
    face: FaceView<'_>,
    wire: WireView<'_>,
    normal: Vector,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Decision> {
    let corners: Vec<[f64; 3]> = wire.points().map(|corner| corner.coordinates()).collect();
    if wire
        .coedges()
        .all(|coedge| *coedge.edge().stored_run() == EdgeRun::Straight)
    {
        return winding_round(&corners, normal.components());
    }

    let mut room = how_far_across_the_ring(&corners);
    while let Ok(finer) = Deflection::of(room) {
        // A walk that refuses is the stopping condition and not a failure, for
        // the reason `how_much_space_a_curved_solid_encloses` gives: what is
        // being looked for is a boundary this solid's own numbers can draw, and
        // handing on a refusal about a deflection nobody asked for would
        // replace an answer about the shape with an answer about the walk.
        let Ok(ring) = the_ring_a_wire_traces(model, face, wire, finer, policy, log) else {
            break;
        };
        if let Some(settled) = ring.winding_it_can_answer_for(normal.components(), finer)? {
            return Ok(settled);
        }
        room /= 2.0;
    }
    Err(Error::Undecidable {
        records: a_boundary_that_encloses_too_little_to_wind(&corners),
    })
}

/// The longest side of the box a ring of points sits in.
fn how_far_across_the_ring(ring: &[[f64; 3]]) -> f64 {
    let mut lowest = [f64::MAX; 3];
    let mut highest = [f64::MIN; 3];
    for point in ring {
        for (axis, coordinate) in point.iter().enumerate() {
            lowest[axis] = lowest[axis].min(*coordinate);
            highest[axis] = highest[axis].max(*coordinate);
        }
    }
    (0..3).fold(0.0, |widest: f64, axis| {
        widest.max(highest[axis] - lowest[axis])
    })
}

/// The boundary a wire traces, drawn as points, and how far off the real thing
/// they may be.
struct RingAWireTraces {
    points: Vec<[f64; 3]>,
    /// The worst [`PointsAlongACrossing::deviation`](rcad_geom::PointsAlongACrossing::deviation)
    /// any of the walks reported — one number for the whole ring, the way §24.13
    /// asks for one per run rather than one per point.
    deviation: f64,
}

/// The wire's corners with the points a walk puts along every curved step in
/// between, in the order the wire is being used.
///
/// **The same points the mesher draws**, from the same call, so the sweep and
/// the mesh cannot disagree about where a boundary is (§23.8). The walk hands
/// its points back in the direction the edge is *stored* running, so a step
/// walked the other way has them turned round here.
///
/// # A curved step nobody sampled is refused, whatever the policy says
///
/// [`rcad_geom::walk`] hands a **tolerant** caller an empty list rather than a
/// refusal when the deflection asked for is finer than a point along the
/// crossing can be placed — the caller has already said it will accept a wobble
/// that big, so `room_left_over` gives it the crossing with nothing along it. A
/// certified caller gets [`Error::CannotPlaceAPointThatCloseToACrossing`]
/// instead.
///
/// Taking that silently puts the corners back as the whole ring, which is the
/// bug [`winding_of`] was written to fix. Worse, the margin the sign is weighed
/// against is `perimeter · (deflection + deviation)`, and that is a bound on how
/// far a *sampled* piece strays; the chord of a step nobody sampled strays by
/// the whole of the step's own bend, which nothing here bounds. So the same
/// refusal the certified caller gets is raised here, and the two policies answer
/// this alike. `a_walk_that_came_back_with_no_points.rs` is the shape that
/// reaches it, and was wound backwards before this.
///
/// A run that is not [`EdgeRun::Straight`] but turns out to lie on a straight
/// line comes back empty too, with a deviation of zero — nothing is hidden
/// between its ends and its corners really are the whole of it, which is why
/// what is asked is whether the deflection was the thing that could not be paid
/// for.
///
/// [`Error::CannotPlaceAPointThatCloseToACrossing`]: rcad_geom::Error::CannotPlaceAPointThatCloseToACrossing
///
/// # Errors
///
/// [`Error::CannotPlaceAPointThatCloseToACrossing`] for a curved step the walk
/// came back empty-handed from, whatever else [`rcad_geom::walk`] refuses —
/// wrapped as [`Error::ACrossingAlongThisEdgeCouldNotBeFollowed`], which
/// carries the edge the marcher never saw — and whatever [`NamedRun::new`]
/// makes of a run whose ends do not match it.
fn the_ring_a_wire_traces(
    model: &Model,
    face: FaceView<'_>,
    wire: WireView<'_>,
    deflection: Deflection,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<RingAWireTraces> {
    let mut points: Vec<[f64; 3]> = Vec::new();
    let mut deviation = 0.0f64;
    for coedge in wire.coedges() {
        let edge = coedge.edge();
        points.push(edge.start().point().coordinates());
        if *edge.stored_run() == EdgeRun::Straight {
            continue;
        }
        let (from, to) = if edge.orientation().is_reversed() {
            (edge.end(), edge.start())
        } else {
            (edge.start(), edge.end())
        };
        let run = NamedRun::new(
            edge.stored_run().clone(),
            [from, to].map(|end| PredicatePoint::from(end.point().coordinates())),
        )?;
        let along = the_two_surfaces_along_this_edge(model, face, edge);
        let walked = rcad_geom::walk(&run, [&along[0], &along[1]], deflection, policy, log)
            .map_err(|why| walking_along_this_edge_refused(edge.key(), why))?;
        let strays = walked.deviation().magnitude();
        if walked.points().is_empty() && deflection.length() <= strays {
            return Err(rcad_geom::Error::CannotPlaceAPointThatCloseToACrossing {
                asked: deflection.length(),
                closest: strays,
            }
            .into());
        }
        deviation = deviation.max(strays);
        let mut between: Vec<[f64; 3]> = walked
            .points()
            .iter()
            .map(|point| point.coordinates())
            .collect();
        if edge.orientation().is_reversed() {
            between.reverse();
        }
        points.extend(between);
    }
    Ok(RingAWireTraces { points, deviation })
}

/// The surfaces of the two faces along `edge`, in the model's own frame.
///
/// The sibling of `mesh.rs`'s `the_two_surfaces_either_side`, which is private
/// there and asks the question from a mesh's half-built tables. A seam has this
/// same face on both sides and is listed twice, which is what tells
/// [`rcad_geom::walk`] to read the second surface off the run itself.
///
/// **No refusal for an edge with three faces along it**, where the mesher has
/// one: [`Model::check_edges`](Model::validate) has already refused an edge that
/// is not walked by exactly two coedges by the time this runs, and two coedges
/// name at most two faces.
fn the_two_surfaces_along_this_edge(
    model: &Model,
    face: FaceView<'_>,
    edge: EdgeView<'_>,
) -> [Surface; 2] {
    let mine = face.surface();
    let across = edge
        .neighbouring_faces()
        .find(|&other| other != face.key())
        .map_or(mine, |other| {
            model.face_at(other.slot_in(model.id())).surface()
        });
    [mine, across]
}

impl RingAWireTraces {
    /// Which way this ring winds round `normal`, when its own area outweighs
    /// what walking it at `deflection` could have hidden — and `None` when it
    /// does not.
    ///
    /// The margin is `perimeter · (deflection + deviation)`, doubled because
    /// the sum the predicate decides is *twice* the area. The perimeter is
    /// added up as `|dx| + |dy| + |dz|` per side, which over-estimates the
    /// length of each side by up to `√3` — room enough over the true chord to
    /// swallow the rounding in adding a few thousand of them, and the
    /// over-estimate is on the safe side of the comparison.
    ///
    /// The sampled area is read off the cascade's **interval** level, which is
    /// rigorous by construction, so what is compared is a bound the arithmetic
    /// earned rather than an `f64` total. The sign, once the margin is met, is
    /// still taken from the whole cascade.
    ///
    /// # Errors
    ///
    /// Whatever [`winding_round`] refuses.
    fn winding_it_can_answer_for(
        &self,
        normal: [f64; 3],
        deflection: Deflection,
    ) -> Result<Option<Decision>> {
        let swept = AreaSweptRoundTheNormal {
            corners: &self.points,
            normal,
        };
        let enclosed = swept.on_intervals();
        let hidden = 2.0 * self.how_far_round_it_is() * (deflection.length() + self.deviation);
        let certainly_enclosed = enclosed.lo().abs().min(enclosed.hi().abs());
        if !enclosed.contains_zero() && certainly_enclosed > hidden {
            return winding_round(&self.points, normal).map(Some);
        }
        Ok(None)
    }

    fn how_far_round_it_is(&self) -> f64 {
        let count = self.points.len();
        (0..count)
            .map(|index| {
                let from = self.points[index];
                let to = self.points[(index + 1) % count];
                (0..3)
                    .map(|axis| (to[axis] - from[axis]).abs())
                    .sum::<f64>()
            })
            .sum()
    }
}

/// What to say about a boundary no walk could find enough area in to wind.
///
/// The corners are the part of it a reader can find in the model; the points a
/// walk put between them belong to a deflection that is not one the caller
/// asked for, and §5 records a decision rather than a coordinate.
fn a_boundary_that_encloses_too_little_to_wind(corners: &[[f64; 3]]) -> ProvenanceLog {
    let at: Vec<PredicatePoint> = corners
        .iter()
        .map(|&corner| PredicatePoint::from(corner))
        .collect();
    let mut records = ProvenanceLog::new();
    records.push(Record::new(
        SWEPT_AREA,
        &at,
        Level::Interval,
        // The enclosure of `NOTHING_IS_KNOWN`: no walk got far enough to bound
        // the area, so there is no width to compare against a budget.
        f64::INFINITY,
        Measures::Area,
    ));
    records
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
    let swept = AreaSweptRoundTheNormal { corners, normal };
    let every_number_is_real = are_real_numbers(corners.iter().flatten().copied().chain(normal));
    let (decision, level) = if every_number_is_real {
        decide(&swept)
    } else {
        (NOTHING_IS_KNOWN, Level::Interval)
    };
    match decision {
        Decision::Neg | Decision::Zero | Decision::Pos => Ok(decision),
        Decision::Undecidable { enclosure } => {
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
                enclosure,
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
///
/// **It borrows its corners.** A wire walked to the point cap carries a million
/// of them, and the halving loop builds one of these per pass; owning them
/// copied twenty-five megabytes a pass at the untrusted-import boundary for
/// nothing.
struct AreaSweptRoundTheNormal<'a> {
    corners: &'a [[f64; 3]],
    normal: [f64; 3],
}

/// Which two coordinates each component of a cross product is built from.
///
/// `(u × v)_x = u_y*v_z − u_z*v_y`, and the other two follow it round.
const CROSS_PRODUCT_AXES: [(usize, usize); 3] = [(1, 2), (2, 0), (0, 1)];

impl Predicate for AreaSweptRoundTheNormal<'_> {
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

impl AreaSweptRoundTheNormal<'_> {
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
        Axis, Cone, Cylinder, Deflection, EdgeRun, HalfSpace, Isometry, NamedRun, Orientation,
        Picker, Plane, Point, Scale, Sphere, Surface, Torus, Vector,
    };
    use rcad_predicates::{Approximation, Cause, Point as PredicatePoint, Policy, ProvenanceLog};
    use smallvec::SmallVec;

    use super::{
        Decision, Mesh, Scope, a_circle_round_the_axis,
        a_closed_crossing_is_walked_the_way_it_is_stored, as_coarse_as_a_mesh_gets, gradient_of,
        hand_on_a_refusal_that_found_the_space, how_far_it_strays_from_one_plane,
        how_wide_an_arc_of_one_branch_is, it_lies_flat, largest_coordinate_of,
        the_coarsest_mesh_whose_sides_do_not_cross, winding_of, winding_round,
    };
    use crate::key::{CoedgeSlot, EdgeSlot, FaceSlot, VertexSlot, WireSlot};
    use crate::measure::signed_volume_over;
    use crate::model::{FaceUse, ModelData, ShellData, ShellUse, SolidData, VertexData, WireStep};
    use crate::{DraftStep, Error, FaceView, Invalid, Model, SolidBuilder, SolidView, WireView};

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

    /// The largest coordinate `validate` sizes its room from.
    fn largest_coordinate(model: &Model) -> f64 {
        model.largest_coordinate_in(&Scope::everything(model.data()))
    }

    /// The one invariant the sweep refused this model for.
    fn refusal(model: &Model) -> Invalid {
        match model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
            Err(Error::Invalid(broken)) => broken,
            other => panic!("the sweep should have refused this model, and gave {other:?}"),
        }
    }

    /// A block stored at `x` from 1000 to 1002 and moved back to the origin is
    /// sized where it now sits, 2, because that is where the reader will size
    /// it once the writer has written it there (`docs/decisions.md` §363).
    #[test]
    fn a_solid_moved_toward_the_origin_gets_the_room_of_where_it_now_sits() {
        let mut model = an_empty_model();
        let block = crate::ProfileBuilder::new(vec![
            [1000.0, 0.0],
            [1002.0, 0.0],
            [1002.0, 2.0],
            [1000.0, 2.0],
        ])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square two across");
        let solid = model
            .add_extrusion(&block, 2.0, &Isometry::identity())
            .expect("a block two thick");
        assert_eq!(largest_coordinate(&model), 1002.0);

        let back = Isometry::translation(Vector::new(-1000.0, 0.0, 0.0)).expect("a finite offset");
        model.place_solid(solid, &back).expect("this model's key");
        assert_eq!(largest_coordinate(&model), 2.0);
    }

    /// A cube moved far from the origin keeps the room of where it is stored.
    /// Where it now sits would give it more, and a placement read from a file
    /// could then buy a corner as much room as it liked (§363).
    #[test]
    fn a_solid_moved_away_from_the_origin_gets_no_more_room_than_where_it_is_stored() {
        let mut model = a_cube();
        let at_rest = largest_coordinate(&model);
        let cube = model.solids().next().expect("one solid").key();
        let away = Isometry::translation(Vector::new(1.0e6, 0.0, 0.0)).expect("a finite offset");
        model.place_solid(cube, &away).expect("this model's key");
        assert_eq!(largest_coordinate(&model), at_rest);
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
        assert_eq!(
            refusal_of_the_only_solid(&model),
            Invalid::DanglingKey { entity: "vertex" }
        );
    }

    /// **The walk down from one solid steps over a key that names nothing**,
    /// and the key check refuses it by the name `validate` gives — at every
    /// level, the way
    /// `every_level_of_the_model_is_refused_for_naming_something_that_is_gone`
    /// asks it of the whole model. A walk that followed the key would index
    /// an arena with it and panic.
    #[test]
    fn one_solid_naming_something_that_is_gone_is_refused_rather_than_crashing_the_walk() {
        type TakesOneOut = fn(&mut ModelData);
        let take_one_out: [(&str, TakesOneOut); 6] = [
            ("vertex", |data| {
                let slot = data.vertices.keys().next().expect("a corner");
                data.vertices.remove(slot);
            }),
            ("edge", |data| {
                let slot = data.edges.keys().next().expect("an edge");
                data.edges.remove(slot);
            }),
            ("coedge", |data| {
                let slot = data.coedges.keys().next().expect("a coedge");
                data.coedges.remove(slot);
            }),
            ("wire", |data| {
                let slot = data.wires.keys().next().expect("a wire");
                data.wires.remove(slot);
            }),
            ("face", |data| {
                let slot = data.faces.keys().next().expect("a face");
                data.faces.remove(slot);
            }),
            ("shell", |data| {
                let slot = data.shells.keys().next().expect("a shell");
                data.shells.remove(slot);
            }),
        ];

        for (entity, take_out) in take_one_out {
            let mut model = a_cube();
            take_out(model.data_mut());

            assert_eq!(
                refusal_of_the_only_solid(&model),
                Invalid::DanglingKey { entity },
                "a {entity} taken out"
            );
            assert_eq!(refusal_of_the_only_solid(&model), refusal(&model));
        }
    }

    /// The one invariant [`Model::validate_solid`] refused the model's only
    /// solid for.
    fn refusal_of_the_only_solid(model: &Model) -> Invalid {
        let solid = model.solids().next().expect("one solid").key();
        match model.validate_solid(solid, &Policy::certified(), &mut ProvenanceLog::new()) {
            Err(Error::Invalid(broken)) => broken,
            other => panic!("the solid should have been refused, and gave {other:?}"),
        }
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

    /// A collapsed edge is refused unless its run says it comes back where it
    /// started, and the run kinds that say so are a list — not "anything that is
    /// not straight".
    ///
    /// The sweep used to ask `run == Straight`, which reads a kind it has never
    /// seen as *not straight* and keeps a zero-length edge. It asks
    /// `the_ends_are_the_whole_of_it` now, which is the same list
    /// [`crate::SolidBuilder::add_edge`] refuses on, so neither door keeps what
    /// the other throws out.
    ///
    /// **A fourth kind cannot be forged from in here**, because [`EdgeRun`] can
    /// only grow in `rcad-geom`, so this pins the three that exist and
    /// `tests/a_fourth_kind_of_run.rs` is what says a fourth arrived.
    #[test]
    fn a_collapsed_edge_is_kept_only_when_its_run_names_a_curve() {
        let corner = a_cube_with_one_edge_collapsed_to(EdgeRun::Straight).1;
        let flat_through_the_corner = Surface::Plane(
            Plane::from_point_and_normal(corner, Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal"),
        );

        for run in [
            EdgeRun::RightRoundWhereItsFacesCross,
            EdgeRun::PartOfWhereItCrosses {
                and: flat_through_the_corner,
                picked_by: SmallVec::new(),
            },
        ] {
            let model = a_cube_with_one_edge_collapsed_to(run.clone()).0;
            let broken = model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .err();

            assert!(
                !matches!(broken, Some(Error::Invalid(Invalid::EdgeHasNoLength))),
                "{run:?} names a curve, so an edge back at its one corner has \
                 run right round that curve and has a length: got {broken:?}"
            );
        }

        let model = a_cube_with_one_edge_collapsed_to(EdgeRun::Straight).0;

        assert_eq!(refusal(&model), Invalid::EdgeHasNoLength);
    }

    /// A cube with one edge pointed back at the corner it left, running along
    /// what this says — and where that corner is.
    fn a_cube_with_one_edge_collapsed_to(run: EdgeRun) -> (Model, Point) {
        let mut model = a_cube();
        let edge = model.data().edges.keys().next().expect("a cube has edges");
        let start = model.data().edges[edge].start;
        model.data_mut().edges[edge].end = start;
        model.data_mut().edges[edge].run = run;
        let corner = model.data().vertices[start].point;
        (model, corner)
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

    /// The wider half of the same counter: two different shells, not one
    /// shell listing a face twice.
    ///
    /// `check_faces` counts face uses with `for shell in
    /// self.data().shells.values()`, so the count is global across shells,
    /// not per shell. The test above only exercises one shell listing a face
    /// twice; this is the shape a consumer actually meets — a face arriving
    /// with two placements because two shells both claim it, which
    /// downstream is a wrong picture drawn confidently.
    #[test]
    fn two_shells_that_both_list_one_face_are_caught() {
        let mut model = a_cube();
        let first_shell = model
            .data()
            .shells
            .keys()
            .next()
            .expect("a cube has a shell");
        let shared = model.data().shells[first_shell].faces[0];

        // Identity, because the placement is not what is on trial here and a
        // shift of its own is `Invalid::AFaceUseCarriesAPlacement` now — which
        // would make this test pass under a name it is not about.
        let second_placement = FaceUse {
            face: shared.face,
            loc: Isometry::identity(),
            orient: shared.orient,
        };
        model.data_mut().shells.insert(ShellData {
            faces: vec![second_placement],
        });

        assert_eq!(
            refusal(&model),
            Invalid::UsedMoreThanOnce {
                entity: "face",
                count: 2
            }
        );
    }

    /// **Every way of sharing a cube's six faces, or a round bar's three,
    /// out between two solids is refused by name**: by the sweep, by
    /// `validate_solid` of either half, and by `check_shape_is_possible`,
    /// which is the check `Deserialize` runs.
    ///
    /// Each half is an open skin and every edge along the cut is walked once
    /// from each half. Counted over the model, those edges still have two
    /// coedges running opposite ways, so the count alone let every one of
    /// these through the door (`docs/decisions.md` §342).
    /// `tests/a_skin_that_closes_only_with_another_never_reads_back.rs` shows
    /// the door refusing the same 34 as bytes, where postcard drops the name.
    #[test]
    fn a_skin_shared_out_between_two_solids_is_caught_every_way_it_can_be_cut() {
        let certified = Policy::certified();
        let mut cuts = 0;
        for (whole, faces) in [(a_cube(), 6), (a_round_bar(), 3)] {
            for mask in 1u32..(1 << faces) - 1 {
                if mask & 1 == 0 {
                    continue;
                }
                let first: Vec<usize> = (0..faces).filter(|&at| mask & (1 << at) != 0).collect();
                let model = shared_out_between_two_solids(whole.clone(), &first);

                assert_eq!(
                    refusal(&model),
                    Invalid::AnEdgeIsWalkedOnceFromEachOfTwoShells,
                    "{first:?} of {faces} faces"
                );
                for half in model.solids() {
                    match model.validate_solid(half.key(), &certified, &mut ProvenanceLog::new()) {
                        Err(Error::Invalid(Invalid::AnEdgeIsWalkedOnceFromEachOfTwoShells)) => {}
                        other => panic!("a half of {first:?} of {faces} faces gave {other:?}"),
                    }
                }
                assert!(
                    matches!(
                        model.check_shape_is_possible(),
                        Err(Error::Invalid(
                            Invalid::AnEdgeIsWalkedOnceFromEachOfTwoShells
                        ))
                    ),
                    "the door let {first:?} of {faces} faces in"
                );
                cuts += 1;
            }
        }
        assert_eq!(cuts, 31 + 3);
    }

    /// A model holding one solid of one shell, with the faces at the places
    /// `first` names kept in that shell and the rest moved to a shell of their
    /// own, used by a new solid.
    fn shared_out_between_two_solids(mut model: Model, first: &[usize]) -> Model {
        let shell = model.data().shells.keys().next().expect("one skin");
        let every_face = std::mem::take(&mut model.data_mut().shells[shell].faces);
        let (kept, moved): (Vec<_>, Vec<_>) = every_face
            .into_iter()
            .enumerate()
            .partition(|(at, _)| first.contains(at));
        model.data_mut().shells[shell].faces = kept.into_iter().map(|(_, used)| used).collect();
        let other = model.add_shell(moved.into_iter().map(|(_, used)| used).collect());
        model.add_solid(smallvec::smallvec![ShellUse {
            shell: other,
            loc: Isometry::identity(),
            orient: Orientation::Forward,
        }]);
        model
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
    /// plane is not that plane: the normal is scaled to length one and that
    /// division is not exact, so the point reads a fifth of a coordinate
    /// spacing off a plane it is on. (`docs/decisions.md` §121 and §132 are
    /// what made both of those one rounding rather than two; neither makes
    /// them none.)
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
        let rounding = model.what_writing_a_plane_down_costs(largest_coordinate(&model));

        nudge_the_seam_plane(&mut model, rounding / 2.0);

        a_clean_sweep(&model);
    }

    #[test]
    fn a_corner_further_off_the_plane_of_its_arc_than_that_is_caught() {
        let mut model = an_empty_model();
        model
            .add_ball(2.0, &Isometry::identity())
            .expect("a ball of radius two");
        let rounding = model.what_writing_a_plane_down_costs(largest_coordinate(&model));

        nudge_the_seam_plane(&mut model, rounding * 2.0);

        assert_eq!(refusal(&model), Invalid::VertexOffItsFaceSurface);
    }

    /// A closed rim names its second surface too, and that name is asked about.
    ///
    /// [`EdgeRun::RightRoundWhereItCrosses`] carries the same `and` as
    /// [`EdgeRun::PartOfWhereItCrosses`] does — the second surface the faces
    /// either side of the edge do not name — so the same half of Q16's
    /// invariant has to keep it honest: an edge's ends may not be provably off
    /// the surface the edge's own name says it runs on. `check_edge` asked that
    /// under `if let EdgeRun::PartOfWhereItCrosses`, which the closed kind does
    /// not match, so its surface was stored and never looked at.
    ///
    /// Every rim a fuse writes is sound there — a closed rim's two ends are one
    /// corner, on both surfaces of the crossing by construction — but nothing in
    /// the kernel said so, and a file can name any plane at all. This is one: a
    /// bar's cap circle saying it runs where the wall crosses a plane a hundred
    /// millimetres above the corner it stands on. `docs/decisions.md` §217.5.
    ///
    /// **A hundred and not one ulp.** The check gives a plane
    /// [`Model::what_writing_a_plane_down_costs`] of room and asks
    /// `not_provably_off` first, so a corner merely *near* the wrong plane is
    /// let through by design — the pair of tests above this is that allowance.
    /// Only a corner somebody chose to put elsewhere is refused.
    ///
    /// **A bar's cap circle is not really this kind of run, and cannot be.** The
    /// kind says the faces either side are the one surface and name nothing
    /// between them; a cap circle has a plane one side and a cylinder the other,
    /// which name it perfectly well. The kernel has no in-crate builder for a
    /// solid whose wall is cut in two — that shape comes out of the fuse, and
    /// `tests/a_rim_a_fuse_cut_round_one_cone.rs` sweeps one — so this hand-made
    /// carrier is what `check_edge` can be reached with from here. It gets that
    /// far because `check_edges` is the third check of the sweep and the mesher
    /// runs in the sixth; the twin below says what the mesher makes of the same
    /// model, and why it is asked a narrower question.
    #[test]
    fn a_closed_rim_naming_a_plane_its_corner_is_nowhere_near_is_caught() {
        let mut model = a_round_bar();
        let (rim, corner) = a_cap_circle_and_the_corner_it_stands_on(&model);
        model.data_mut().edges[rim].run = EdgeRun::RightRoundWhereItCrosses {
            and: flat_and_facing_up_through(Point::new(0.0, 0.0, corner.z() + 100.0)),
        };

        assert_eq!(refusal(&model), Invalid::VertexOffItsFaceSurface);
    }

    /// And the half that says the check refuses a wrong plane rather than the
    /// kind.
    ///
    /// The same bar, the same rim, naming the plane its corner really does sit
    /// in, and `check_edges` takes it.
    ///
    /// **It is asked on its own rather than through the whole sweep**, because
    /// the whole sweep refuses this model either way and for something else: the
    /// mesher walks a closed rim as `along[0]` crossed with `and`
    /// (`rcad-geom`'s `walk.rs`), and `along[0]` for the cap face is the cap's
    /// own plane, so it is asked where a plane crosses itself and gets nothing —
    /// `AWireDoesNotFanIntoALoopToMesh { TooFewCorners { count: 1 } }`. That is
    /// the model being a lie about its own shape, not the check being wrong, and
    /// a rim the fuse really wrote meshes and sweeps clean.
    #[test]
    fn a_closed_rim_naming_the_plane_its_corner_really_sits_in_gets_past_that_check() {
        let mut model = a_round_bar();
        let (rim, corner) = a_cap_circle_and_the_corner_it_stands_on(&model);
        model.data_mut().edges[rim].run = EdgeRun::RightRoundWhereItCrosses {
            and: flat_and_facing_up_through(corner),
        };

        model
            .check_edges(&Scope::everything(model.data()))
            .expect("the corner sits in the plane the rim names");
    }

    /// The first edge of this model that comes back to the corner it left, and
    /// where that corner is.
    fn a_cap_circle_and_the_corner_it_stands_on(model: &Model) -> (EdgeSlot, Point) {
        let rim = model
            .data()
            .edges
            .keys()
            .find(|&slot| model.data().edges[slot].start == model.data().edges[slot].end)
            .expect("a round bar's caps are closed circles");
        let corner = model.data().vertices[model.data().edges[rim].start].point;
        (rim, corner)
    }

    fn flat_and_facing_up_through(point: Point) -> Surface {
        Surface::Plane(
            Plane::from_point_and_normal(point, Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal"),
        )
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
        let rounding = model.what_writing_a_point_down_costs(largest_coordinate(&model));

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
        let rounding = model.what_writing_a_point_down_costs(largest_coordinate(&model));

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

    /// The same check, asked about a flat surface that holds its point, the
    /// picker §443 added for where two pipes of one width meet.
    ///
    /// The seam's own half-space, held as its point at the centre and its
    /// facing, picks the same half of the meridian, so the seam is sound.
    /// Through `(5, 0, 0)` facing `+x` the surface leaves the whole ball
    /// behind it, and the seam is caught.
    #[test]
    fn an_edge_outside_a_flat_surface_holding_its_point_that_picked_it_is_caught() {
        let elsewhere = rcad_geom::HalfSpaceThroughAPoint::new(
            Point::new(5.0, 0.0, 0.0),
            Vector::new(1.0, 0.0, 0.0),
        )
        .expect("the x axis has length");

        for far_off in [false, true] {
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
            let at_the_centre = rcad_geom::HalfSpaceThroughAPoint::new(
                Point::origin(),
                picked_by[0]
                    .facing_of_a_flat()
                    .expect("a ball's seam is picked by a flat half-space"),
            )
            .expect("a half-space's normal has length");
            picked_by[0] = rcad_geom::Picker::from(if far_off { elsewhere } else { at_the_centre });

            if far_off {
                assert_eq!(refusal(&model), Invalid::EdgeOutsideWhatPickedIt);
            } else {
                model
                    .validate(&Policy::certified(), &mut ProvenanceLog::new())
                    .expect("the poles lie on the surface through the centre");
            }
        }
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

    /// A cube turned inside out the way a file has to write one: every plane
    /// turned round, and every wire walked the other way so that it still winds
    /// with the normal it is given.
    ///
    /// **Turning the six face *uses* round instead is a different refusal now**
    /// and is `a_flat_face_used_reversed_is_caught` below. Written that way no
    /// single face is wrong and neither is any single wire, so the volume was
    /// the only thing that could notice; written this way it still is, which is
    /// what this test is here to say.
    #[test]
    fn a_solid_turned_inside_out_is_caught() {
        let mut model = a_cube();
        for face in model.data_mut().faces.values_mut() {
            let plane = *face
                .surface
                .as_plane()
                .expect("a cube's faces sit on planes");
            face.surface = Surface::Plane(plane.reversed());
        }
        for coedge in model.data_mut().coedges.values_mut() {
            coedge.orientation = coedge.orientation.reversed();
        }
        for wire in model.data_mut().wires.values_mut() {
            wire.steps.reverse();
        }

        assert!(matches!(
            refusal(&model),
            Invalid::VolumeIsNotPositive { .. }
        ));
    }

    /// One wall of a box turned round, which is the shape ten fuzz crashes came
    /// in as (`two_solids_fused_by_a_stranger`, 2026-09-06).
    ///
    /// **Every other check passes it**, and that is the whole point: the stored
    /// turns are untouched, so
    /// [`Model::check_edge_is_walked_by_two_opposite_coedges`] sees nothing; the
    /// wall's own winding is unchanged, because reversing a face turns its normal
    /// *and* walks its wire the other way; and a `10³` box with one wall turned
    /// round still sums to `333.3` of enclosed space, so the volume has no
    /// complaint either. What used to notice was `Model::check_touched_edge`,
    /// from inside the fuse, in a debug build only.
    ///
    /// The control is the same box left alone, which validates.
    #[test]
    fn a_flat_face_used_reversed_is_caught() {
        let sound = a_cube();
        assert!(
            sound
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .is_ok(),
            "the control has to pass, or the refusal below says nothing"
        );

        for turned in 0..6 {
            let mut model = a_cube();
            let shell = model
                .data()
                .shells
                .keys()
                .next()
                .expect("a cube has one shell");
            let used = &mut model.data_mut().shells[shell].faces[turned];
            used.orient = used.orient.reversed();

            assert_eq!(refusal(&model), Invalid::AFlatFaceIsUsedReversed);
        }
    }

    /// The same bytes, refused one step earlier: at the door `Deserialize` runs,
    /// rather than in the sweep a caller has to ask for.
    ///
    /// `Model::fuse` does not validate its arguments — a sweep per operation is
    /// what §4.3 keeps the suite from paying — so a model read off a wire and
    /// handed straight to the fuse is exactly the shape the fuzzer found, and the
    /// door is the only thing between the two.
    #[test]
    fn a_file_with_a_wall_turned_round_never_becomes_a_model() {
        let mut model = a_cube();
        assert!(
            postcard::from_bytes::<Model>(
                &postcard::to_allocvec(&model).expect("a model writes out")
            )
            .is_ok(),
            "the same box with nothing turned round reads back"
        );

        let shell = model
            .data()
            .shells
            .keys()
            .next()
            .expect("a cube has one shell");
        let used = &mut model.data_mut().shells[shell].faces[0];
        used.orient = used.orient.reversed();

        assert_eq!(
            postcard::from_bytes::<Model>(
                &postcard::to_allocvec(&model).expect("a model writes out")
            )
            .expect_err("a wall turned round is refused"),
            postcard::Error::SerdeDeCustom
        );
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
        let model = a_cube_with_one_closed_edge_spliced_in(EdgeRun::RightRoundWhereItsFacesCross);

        assert_eq!(refusal(&model), Invalid::ClosedEdgeBetweenTwoFlatFaces);
    }

    /// The same shape with a closed run that names a **plane**, which is what a
    /// closed `ELLIPSE` drawn on a flat face writes — §219.6 leaves that a limit
    /// rather than a gap, because no single surface this kernel holds crosses a
    /// plane in a given ellipse without somebody choosing a tilt.
    ///
    /// A plane says no more about where a closed edge goes than the two flat
    /// faces do, so the refusal is the same one. It used to come back as
    /// [`Invalid::AnEdgeBendsButNothingAlongItIsCurved`], the name meant for
    /// open edges, because the match named only the older closed variant and
    /// this one fell into the wildcard — §219.7 item 3.
    #[test]
    fn a_closed_edge_whose_run_names_a_plane_is_refused_under_the_closed_name() {
        let model = a_cube_with_one_closed_edge_spliced_in(EdgeRun::RightRoundWhereItCrosses {
            and: Surface::Plane(the_ground()),
        });

        assert_eq!(refusal(&model), Invalid::ClosedEdgeBetweenTwoFlatFaces);
    }

    fn a_cube_with_one_closed_edge_spliced_in(run: EdgeRun) -> Model {
        let mut model = a_cube();
        let corner = model
            .data()
            .vertices
            .keys()
            .next()
            .expect("a cube has corners");

        let closed_edge = model.add_edge_running(corner, corner, run);
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
        Model::from_parts(model.data().clone())
    }

    /// A frustum cut into two bands by a closed circle, each band on its own
    /// spelling of one cone, is sound walked the way its cone winds: with the
    /// two spellings equal to the bit or one ulp apart, and whichever way the
    /// circle's own plane faces.
    ///
    /// The two bands are one cone, so they cross nowhere near the circle, and
    /// the circle names its plane. That plane is what each band's use of the
    /// circle is read against (§229.4). Read against the other band's cone
    /// instead, the one-ulp spelling was refused: the two gradients at the
    /// corner cross to `(0, 1.07e-14, 0)`, the lower band's far corner is
    /// `-7.1e-15` inside the upper cone, and both of those are rounding.
    #[test]
    fn a_frustum_in_two_bands_walked_the_way_its_cone_winds_is_sound() {
        for upper_radius in [6.0, 6.0_f64.next_up()] {
            for facing in [1.0, -1.0] {
                let (model, built) =
                    a_frustum_in_two_bands(upper_radius, facing, Orientation::Reversed);
                built.unwrap_or_else(|why| {
                    panic!("upper radius {upper_radius}, plane facing {facing} z: {why:?}")
                });
                a_clean_sweep(&model);
            }
        }
    }

    /// The same frustum with the middle circle walked the other way by both
    /// bands is refused by the closed-edge check, by name.
    ///
    /// The two uses still run opposite ways, so no older check has anything to
    /// say. Read against the other band's cone, this check had nothing to say
    /// either: two cones equal to the bit cross to the zero vector, and one ulp
    /// apart the rounding read the lower band's use as right. Only the mesh
    /// refused it then, under another name.
    #[test]
    fn a_frustum_in_two_bands_walked_against_its_cone_is_caught() {
        for upper_radius in [6.0, 6.0_f64.next_up()] {
            for facing in [1.0, -1.0] {
                let (_, built) = a_frustum_in_two_bands(upper_radius, facing, Orientation::Forward);
                assert!(
                    matches!(
                        built,
                        Err(Error::Invalid(
                            Invalid::AClosedEdgeIsWalkedAgainstItsSurface
                        ))
                    ),
                    "upper radius {upper_radius}, plane facing {facing} z: {built:?}"
                );
            }
        }
    }

    /// The frustum of the cone with its apex at the origin and a slope of three
    /// in four, from `z = 4` to `z = 12`, cut into two bands by the circle of
    /// radius six at `z = 8`. Each band is on its own cone, placed by its radius
    /// at that circle the way a file places one: six for the lower band and
    /// `upper_radius` for the upper. The circle names its own plane, facing
    /// `facing` along `z`. The lower band walks the circle `lower_band_walks_it`
    /// and the upper band walks it back.
    ///
    /// The rule stores the circle running the way the cone's longitude `ẑ × ∇C`
    /// goes, which is `+y` at the corner `(6, 0, 8)`. The lower band keeps its
    /// material below the circle, so it walks it `-y`: [`Orientation::Reversed`].
    fn a_frustum_in_two_bands(
        upper_radius: f64,
        facing: f64,
        lower_band_walks_it: Orientation,
    ) -> (Model, crate::Result<crate::SolidKey>) {
        let up = Vector::new(0.0, 0.0, 1.0);
        let placed_at_the_middle_circle = |radius: f64| {
            let apex = Point::new(0.0, 0.0, 8.0 - radius / 0.75);
            Surface::Cone(
                Cone::new(Axis::new(apex, up).expect("z has a length"), 0.75)
                    .expect("a positive slope"),
            )
        };
        let flat = |height: f64, facing: f64| {
            Surface::Plane(
                Plane::from_point_and_normal(Point::new(0.0, 0.0, height), up * facing)
                    .expect("z has a length"),
            )
        };

        let mut builder = SolidBuilder::new();
        let [bottom, middle, top] = [(3.0, 4.0), (6.0, 8.0), (9.0, 12.0)].map(|(radius, z)| {
            builder
                .add_vertex(Point::new(radius, 0.0, z))
                .expect("a corner")
        });
        let mut an_edge = |from, to, run| builder.add_edge(from, to, run).expect("an edge");
        let round_the_bottom = an_edge(bottom, bottom, EdgeRun::RightRoundWhereItsFacesCross);
        let round_the_middle = an_edge(
            middle,
            middle,
            EdgeRun::RightRoundWhereItCrosses {
                and: flat(8.0, facing),
            },
        );
        let round_the_top = an_edge(top, top, EdgeRun::RightRoundWhereItsFacesCross);
        let up_the_lower_band = an_edge(bottom, middle, EdgeRun::Straight);
        let up_the_upper_band = an_edge(middle, top, EdgeRun::Straight);

        let upper_band_walks_it = lower_band_walks_it.reversed();
        let faces = [
            (
                placed_at_the_middle_circle(6.0),
                &[
                    (round_the_bottom, Orientation::Forward),
                    (up_the_lower_band, Orientation::Forward),
                    (round_the_middle, lower_band_walks_it),
                    (up_the_lower_band, Orientation::Reversed),
                ][..],
            ),
            (
                placed_at_the_middle_circle(upper_radius),
                &[
                    (round_the_middle, upper_band_walks_it),
                    (up_the_upper_band, Orientation::Forward),
                    (round_the_top, Orientation::Reversed),
                    (up_the_upper_band, Orientation::Reversed),
                ][..],
            ),
            (
                flat(4.0, -1.0),
                &[(round_the_bottom, Orientation::Reversed)][..],
            ),
            (
                flat(12.0, 1.0),
                &[(round_the_top, Orientation::Forward)][..],
            ),
        ]
        .map(|(surface, walked)| {
            let steps: Vec<DraftStep> = walked
                .iter()
                .map(|&(edge, way)| {
                    DraftStep::Along(builder.add_coedge(edge, way).expect("an edge"))
                })
                .collect();
            let wire = builder.add_wire(&steps).expect("a closed wire");
            (
                builder.add_face(surface, &[wire]).expect("a face"),
                Orientation::Forward,
            )
        });
        builder.add_shell(&faces).expect("the four faces close up");

        let mut model = an_empty_model();
        let built = builder.build(&mut model, &Policy::certified(), &mut ProvenanceLog::new());
        (model, built)
    }

    /// The two faces along a closed crossing must disagree about which way it
    /// runs even where the longitude has nothing to say.
    ///
    /// A ball of radius one at the origin, cut by `x = 0.6`, with the corner put
    /// at `(0.6, 0.8, 0)`: every one of those is exact in `f64` and the point is
    /// on the ball to the last bit. The plane is parallel to the ball's axis and
    /// the corner is on its equator, which is the pair of coincidences that makes
    /// `(∇S × ∇T) · (ẑ × ∇C)` exactly zero — asserted below, so this test cannot
    /// stop exercising the second reference without saying so.
    ///
    /// `tests/a_ball_poking_through_a_cubes_side.rs` is the same coincidence
    /// arriving from a fuzzer and going all the way through `Model::fuse`. This
    /// one is the rule on its own, which is where the bug was.
    #[test]
    fn a_ball_cut_by_a_plane_along_its_axis_is_still_walked_two_ways() {
        let ball = Surface::Sphere(
            Sphere::new(Point::origin(), 1.0).expect("a ball of radius one at the origin"),
        );
        let flat = Surface::Plane(
            Plane::from_point_and_normal(Point::new(0.6, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
                .expect("the plane x = 0.6"),
        );
        let corner = Point::new(0.6, 0.8, 0.0);

        let round = gradient_of(&ball, corner);
        let straight = gradient_of(&flat, corner);
        assert_eq!(
            straight
                .cross(round)
                .dot(Vector::new(0.0, 0.0, 1.0).cross(round)),
            0.0,
            "the longitude is square to the wire here, which is what this test is about"
        );

        let walked = |pair: [&Surface; 2], outward: [Vector; 2]| {
            a_closed_crossing_is_walked_the_way_it_is_stored(pair, outward, corner, Decision::Neg)
                .expect("one of the two is curved")
                .expect("neither a pole nor a touch")
        };

        assert_ne!(
            walked([&flat, &ball], [straight, round]),
            walked([&ball, &flat], [round, straight]),
            "the flat face and the cap have to walk their shared circle opposite ways"
        );
    }

    /// A crossing a third of an `f64` step wide, and its sign is there to be had.
    ///
    /// `(0.1, 0.2, 0.3)` and `(0.3, 0.6, 0.9)` look like one direction written
    /// twice and are not. The doublings are exact — `0.2` is `2 × 0.1` and `0.6`
    /// is `2 × 0.3`, to the bit — so the `z` of the cross product is exactly
    /// zero, but `0.3` is **not** `3 × 0.1` and `0.9` is **not** `9 × 0.1`, so
    /// the other two components are genuinely nonzero. Worked out exactly over
    /// the stored doubles the cross product is
    /// `(2.7755575615628914e-17, -1.3877787807814457e-17, 0)`, which is
    /// `7.39e-17` of the two vectors that made it.
    ///
    /// **So this is a real crossing and not a rounding**, and the answer exists:
    /// the exact dot is `-6.93889390390722838e-18`, and `fbab36ff` read its sign
    /// correctly as `Ok(Some(false))`. Between that sha and §122 this case was
    /// refused, which §111.3 recorded as a *gap* (§16.10) rather than a limit;
    /// §122's exact tier closes it and the answer is `fbab36ff`'s again — reached
    /// this time by arithmetic that proves it rather than by luck.
    ///
    /// What the enclosure runs into, and still does, is that the true difference
    /// in the `y` component is exactly one ulp of the two products it comes from,
    /// while an interval holds each product to half an ulp — so the enclosure of
    /// the difference is one ulp wide and the answer sits on its edge.
    #[test]
    fn a_crossing_barely_one_f64_step_wide_is_settled_by_the_exact_level() {
        let ball =
            Surface::Sphere(Sphere::new(Point::origin(), 1.0).expect("a ball at the origin"));
        let flat = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
                .expect("the plane z = 0"),
        );
        let corner = Point::new(0.1, 0.2, 0.3);
        let nearly_one_direction = [Vector::new(0.1, 0.2, 0.3), Vector::new(0.3, 0.6, 0.9)];
        let walked = nearly_one_direction[0].cross(nearly_one_direction[1]);
        assert_eq!(
            walked.components(),
            [2.7755575615628914e-17, -1.3877787807814457e-17, 0.0],
            "f64 gets the exact cross product here, so the crossing is real"
        );

        assert_eq!(
            a_closed_crossing_is_walked_the_way_it_is_stored(
                [&ball, &flat],
                nearly_one_direction,
                corner,
                Decision::Neg,
            )
            .expect("the exact level has this sign"),
            Some(false)
        );
    }

    /// `kicad/LED_D5.0mm-3.step`'s dome and body, lifted out of the file so the
    /// numbers §109.7 reports can be re-measured without the corpus.
    ///
    /// The file writes `#39 = SPHERICAL_SURFACE('',#40,2.5)` on
    /// `(2.54, -5.051514762044E-015, 9.1)` and
    /// `#73 = CYLINDRICAL_SURFACE('',#74,2.5)` on
    /// `(2.54, -3.030908857227E-015, 9.1)` with a direction of
    /// `(-0, 3.330669073875E-016, -1)`, and `#22` is the corner they share at
    /// `(5.04, -5.051514762044E-015, 9.1)`. A hemisphere sitting on a pipe of the
    /// same radius and an axis three femtometres out: the two are meant to be
    /// tangent and the file's own rounding is the whole of what makes them cross.
    ///
    /// **What this pins is how thin that crossing is**, because that is the fact
    /// the rule has to be honest about and it is a fact about the shape rather
    /// than about any one spelling of the arithmetic. `‖∇S × ∇T‖²` against
    /// `‖∇S‖²‖∇T‖²` is the squared sine of the angle the two surfaces meet at,
    /// and it comes to `6.5e-31` — an angle of `8.1e-16` radians, which is under
    /// four `f64` steps of one.
    ///
    /// **It does not yet pin the rule's answer, and §111 says why.** The rule
    /// still reads `Ok(Some(true))` here, and the arithmetic that gets it there
    /// is exact — every product below is one rounding with nothing to cancel
    /// against, so enclosing the chain over these numbers proves the same sign
    /// rather than refusing. What is not earned is the numbers themselves: the
    /// corner is a decimal rounded to `f64` and is on neither surface, and one
    /// `f64` step in its `y` is `1.8e-15` against a `∇T·ŷ` of `-2.0e-15`. Moving
    /// the corner inside the box it is known to turns the answer over. Closing
    /// that needs the corner to arrive as a box and the gradients to be worked
    /// out over it, which is a change to what this function takes.
    #[test]
    fn a_hemisphere_sitting_on_a_pipe_of_the_same_radius_barely_crosses_it_at_all() {
        let ball = Surface::Sphere(
            Sphere::new(Point::new(2.54, -5.051514762044E-015, 9.1), 2.5).expect("the dome"),
        );
        let pipe = Surface::Cylinder(
            Cylinder::new(
                Axis::new(
                    Point::new(2.54, -3.030908857227E-015, 9.1),
                    Vector::new(-0.0, 3.330669073875E-016, -1.0),
                )
                .expect("the body's axis"),
                2.5,
            )
            .expect("the body"),
        );
        let corner = Point::new(5.04, -5.051514762044E-015, 9.1);

        let round = gradient_of(&ball, corner);
        let straight = gradient_of(&pipe, corner);
        let walked = round.cross(straight);
        assert_eq!(
            walked.components(),
            [0.0, 1.682492399415798e-30, -5.0515147620424996e-15],
            "the wire §109.7 measured, out of the file's own numbers"
        );

        let squared_sine = walked.dot(walked) / (round.dot(round) * straight.dot(straight));
        assert!(
            squared_sine < 1e-30,
            "the dome and the body meet at {} radians, which is more than the file writes",
            squared_sine.sqrt()
        );
        assert!(
            squared_sine.sqrt() < 4.0 * f64::EPSILON,
            "an angle of a few f64 steps is the whole of what makes this hard"
        );
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
        let allowed = model.allowed_wobble(largest_coordinate(&model));
        lift_the_first_corner(&mut model, allowed / 2.0);

        a_clean_sweep(&model);
    }

    #[test]
    fn a_corner_further_off_its_face_plane_than_that_is_caught() {
        let mut model = a_cube();
        let allowed = model.allowed_wobble(largest_coordinate(&model));
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
    ///
    /// The arc is named by the cylinder it runs round. Named by the ground it is
    /// drawn on, nothing would say where it bends, and the sweep refuses that
    /// first as [`Invalid::AnEdgeBendsButNothingAlongItIsCurved`].
    #[test]
    fn a_wire_of_an_arc_and_its_chord_is_long_enough_for_the_sweep() {
        let mut model = an_empty_model();
        let start = model.add_vertex(Point::new(1.0, 0.0, 0.0));
        let end = model.add_vertex(Point::new(-1.0, 0.0, 0.0));
        let round = rcad_geom::Cylinder::new(
            rcad_geom::Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis has length"),
            1.0,
        )
        .expect("a radius of one");
        let arc = model.add_edge_running(
            start,
            end,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Cylinder(round),
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
            winding_of(
                &model,
                view,
                view.outer_wire(),
                the_ground().normal(),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("the cascade settles this"),
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

        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        assert_eq!(
            winding_of(&model, cap, outline, normal, &policy, &mut log)
                .expect("the cascade settles this"),
            Decision::Pos
        );
        assert_eq!(
            winding_of(&model, cap, hole, normal, &policy, &mut log)
                .expect("the cascade settles this"),
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

    /// A plane whose normal is one `f64` ulp off a torus axis direction: the
    /// naive `f64` cross product of the two still rounds to the zero vector,
    /// but the exact cross [`lie_along_one_line`](crate::holds::lie_along_one_line)
    /// reads is not zero, so the plane is not really square to the axis.
    ///
    /// Found by taking a direction off the unit circle,
    /// `(0.3742512179995787, 0.9273273563450136)`, and moving its `x` one
    /// `f64::next_up` to `0.3742512179995788`. Before this row,
    /// `a_circle_round_the_axis` read the naive cross and answered `true` for
    /// this pair, the same wrong answer `holds.rs`'s exact
    /// `what_a_ring_face_between_two_planes_holds` would never give.
    #[test]
    fn a_plane_one_ulp_off_square_to_a_torus_axis_is_not_square_to_it_exactly() {
        let flat = Plane::from_point_and_normal(
            Point::origin(),
            Vector::new(0.3742512179995787, 0.9273273563450136, 0.0),
        )
        .expect("a unit normal");
        let axis = Axis::new(
            Point::origin(),
            Vector::new(0.3742512179995788, 0.9273273563450136, 0.0),
        )
        .expect("a unit direction");
        let ring = Torus::new(axis, 5.0, 1.0).expect("a ring wider than its tube");
        assert_eq!(
            flat.normal().cross(ring.axis().direction()),
            Vector::zero(),
            "the naive f64 cross still rounds to the zero vector at one ulp"
        );
        assert!(
            a_circle_round_the_axis(
                Surface::Torus(ring),
                [Surface::Torus(ring), Surface::Plane(flat)]
            )
            .is_none(),
            "the exact cross is not zero, so this is not a circle round the axis"
        );
    }

    /// A finer mesh of the zero ladder refused because placing the solid took
    /// away the space it encloses is handed on, not read as a reason to stop.
    ///
    /// Stopping returns the zero, and the caller then calls the solid
    /// `VolumeIsNotPositive` (§333.6). No solid is known to send this refusal
    /// from a finer rung, so the rung is asked directly (§379.5, Q783). The
    /// unit ball at `1.08e16`, which lands flat, is refused `4/3` where it
    /// was made and `0` where it lands, and those are the numbers used.
    #[test]
    fn a_finer_mesh_that_lost_its_space_to_the_placement_is_handed_on() {
        let Err(Error::PlacingItTookAwayTheSpaceItsMeshEncloses {
            enclosed_where_made,
            enclosed_where_placed,
        }) = hand_on_a_refusal_that_found_the_space(
            Error::PlacingItTookAwayTheSpaceItsMeshEncloses {
                enclosed_where_made: 4.0 / 3.0,
                enclosed_where_placed: 0.0,
            },
        )
        else {
            panic!("the refusal was not handed on");
        };
        assert_eq!(
            (enclosed_where_made, enclosed_where_placed),
            (4.0 / 3.0, 0.0)
        );

        assert!(
            hand_on_a_refusal_that_found_the_space(Error::DeflectionBelowWhatAMeshCanReach {
                asked: 5.0,
                closest: 1e217,
            })
            .is_ok(),
            "the floor still stops the ladder"
        );
    }

    /// The bound [`how_wide_an_arc_of_one_branch_is`] publishes: every point of
    /// an arc of a parabola or of one branch of a hyperbola is within half that
    /// width of the cone's apex.
    ///
    /// Walked finely on the lid `z = 4`: arcs of a parabola running through its
    /// vertex, evenly and not, to one side of it, and from it; arcs of a
    /// hyperbola's branch on each nappe, through the vertex, from it and to one
    /// side of it. Each is walked both ways round, and every point the walk puts
    /// down is inside the ball, give or take how far the walk says its points may
    /// stray.
    #[test]
    fn every_point_of_an_arc_of_one_branch_is_within_the_reach_of_its_ends() {
        let lid =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 0.0, 1.0))
                .expect("a plane");
        // The lid crosses this one in `x = 1 − y²`.
        let parabola = Cone::new(
            Axis::new(Point::new(1.0, 0.0, 3.5), Vector::new(-1.0, 0.0, 1.0)).expect("an axis"),
            1.0,
        )
        .expect("a cone");
        let on_the_parabola = |y: f64| Point::new(1.0 - y * y, y, 4.0);
        // And this one in `(x − 1.25)² = y² + 0.75²`, whose branch at `x ≤ 0.5`
        // is on one nappe and whose branch at `x ≥ 2` is on the other.
        let hyperbola = Cone::new(
            Axis::new(Point::new(1.25, 0.0, 3.25), Vector::new(-1.0, 0.0, 0.0)).expect("an axis"),
            1.0,
        )
        .expect("a cone");
        let near = |[y, x]: [f64; 2]| Point::new(1.25 - x, y, 4.0);
        let far = |[y, x]: [f64; 2]| Point::new(1.25 + x, y, 4.0);
        let below = |[y, x]: [f64; 2]| [-y, x];
        // `y² + 0.75² = x²` at the vertex, and on the triangles 3-4-5 and 8-15-17.
        let [vertex, three_four_five, eight_fifteen] =
            [[0.0, 0.75], [1.0, 1.25], [1.40625, 1.59375]];
        let arcs = [
            (parabola, [on_the_parabola(-2.0), on_the_parabola(2.0)]),
            (parabola, [on_the_parabola(-1.0), on_the_parabola(2.0)]),
            (parabola, [on_the_parabola(0.5), on_the_parabola(2.0)]),
            (parabola, [on_the_parabola(0.0), on_the_parabola(-1.5)]),
            (
                hyperbola,
                [near(below(three_four_five)), near(eight_fifteen)],
            ),
            (hyperbola, [near(vertex), near(eight_fifteen)]),
            (hyperbola, [near(three_four_five), near(eight_fifteen)]),
            (hyperbola, [far(below(eight_fifteen)), far(three_four_five)]),
            (hyperbola, [far(vertex), far(below(three_four_five))]),
        ];
        let under_everything = HalfSpace::through_a_point_facing(
            Point::new(0.0, 0.0, 100.0),
            Vector::new(0.0, 0.0, -1.0),
        )
        .expect("a half-space");
        let mut outside = Vec::new();
        for (cone, ends) in arcs {
            let reach = how_wide_an_arc_of_one_branch_is(&cone, ends) / 2.0;
            for [from, to] in [ends, [ends[1], ends[0]]] {
                let run = NamedRun::new(
                    EdgeRun::PartOfWhereItCrosses {
                        and: Surface::Cone(cone),
                        picked_by: smallvec::smallvec![Picker::from(under_everything)],
                    },
                    [from, to].map(|end| PredicatePoint::from(end.coordinates())),
                )
                .expect("a run");
                let walked = rcad_geom::walk(
                    &run,
                    [&Surface::Plane(lid), &Surface::Plane(lid)],
                    Deflection::of(1e-6).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("the walk draws the arc");
                assert!(
                    walked.points().len() > 10,
                    "{from:?} to {to:?} walked too coarsely"
                );
                let stray = walked.deviation().magnitude();
                for point in walked.points() {
                    let from_the_apex = (*point - cone.apex()).length();
                    if from_the_apex > reach + stray {
                        outside.push(format!(
                            "{point:?} on {from:?} to {to:?}: {from_the_apex} from the apex, past {reach}"
                        ));
                    }
                }
            }
        }
        assert!(outside.is_empty(), "{outside:#?}");
    }

    /// Row Q1849. A cap of a ball of radius 13 cut off one high by the plane
    /// `x = 0`, its rim of radius 5 written as three arcs between `(0, 3, 4)`,
    /// `(0, 3, −4)` and `(0, −5, 0)`, as a file that splits a circle into arcs
    /// writes it. It holds `38π/3 = 39.79`.
    ///
    /// Each arc spans under half a turn and so is walked in two steps, and the
    /// cap spans under a quarter turn of the ball and so is handed over whole:
    /// its coarsest mesh is the rim's six points and nothing else, flat, and
    /// the lid across the rim is drawn through the same six. Not placed, that
    /// encloses exactly `0`, a finer mesh is asked for, and the cap
    /// validates. Turned `1/40` of a turn about `(3, −1, 2)` the six points
    /// land a rounding off one plane, the flat mesh encloses `−1.4e-15`, and
    /// that was read as the cap's volume and refused
    /// [`Invalid::VolumeIsNotPositive`].
    #[test]
    fn a_cap_whose_rim_is_three_arcs_validates_turned_off_the_axes() {
        let mut model = an_empty_model();
        let cap = a_cap_whose_rim_is_three_arcs(&mut model);
        a_clean_sweep(&model);
        model
            .place_solid(
                cap,
                &Isometry::rotation(
                    &rcad_geom::Axis::new(Point::origin(), Vector::new(3.0, -1.0, 2.0))
                        .expect("a direction with length"),
                    core::f64::consts::TAU / 40.0,
                )
                .expect("a finite turn"),
            )
            .expect("this model made the key");
        a_clean_sweep(&model);
    }

    /// Row Q1849, and the `64` ulps [`it_lies_flat`] allows. The cap of
    /// [`a_cap_whose_rim_is_three_arcs_validates_turned_off_the_axes`], turned
    /// through 39 fortieths of a turn about each of four axes.
    ///
    /// At 63 of the 156 turns its coarsest mesh is the rim's six points, which
    /// stray at most `2.1` ulps of the largest coordinate from one plane; at
    /// the other 93 it has a point off the rim and strays at least `4e14`. The
    /// `64` sits between. Each flat mesh encloses no more than how far it
    /// strays times its area, 34 of them below zero and 29 above, and the cap
    /// validates at every turn.
    ///
    /// **The same cap turned inside out is refused at every turn.** Its flat
    /// mesh encloses a rounding above zero at 28 turns, and that was read as
    /// its volume, so it passed: a solid whose faces all point into it.
    #[test]
    fn a_cap_whose_rim_is_three_arcs_lies_flat_at_every_turn_and_validates() {
        let axes = [
            Vector::new(3.0, -1.0, 2.0),
            Vector::new(1.0, 2.0, 3.0),
            Vector::new(1.0, 1.0, 1.0),
            Vector::new(-2.0, 5.0, 1.0),
        ];
        let mut most_ulps_flat = 0.0f64;
        let mut fewest_ulps_bent = f64::INFINITY;
        let mut rounded_flat = [0; 3];
        let mut rounded_inside_out = [0; 3];
        for direction in axes {
            for fortieths in 1..40 {
                let what = format!("turned {fortieths}/40 about {direction:?}");
                let turn = Isometry::rotation(
                    &Axis::new(Point::origin(), direction).expect("a direction with length"),
                    f64::from(fortieths) * core::f64::consts::TAU / 40.0,
                )
                .expect("a finite turn");

                let (mut model, cap) = a_cap_whose_rim_is_three_arcs_placed(&turn);
                turn_it_inside_out(&mut model);
                let enclosed = signed_volume_over(&its_coarsest_mesh(&model, cap, &what));
                rounded_inside_out[which_side_of_zero(enclosed)] += 1;
                let inside_out = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
                assert!(
                    matches!(
                        inside_out,
                        Err(Error::Invalid(Invalid::VolumeIsNotPositive { .. }))
                    ),
                    "{what}: inside out {inside_out:?}"
                );

                let (model, cap) = a_cap_whose_rim_is_three_arcs_placed(&turn);
                let coarsest = its_coarsest_mesh(&model, cap, &what);
                let strays = how_far_it_strays_from_one_plane(&coarsest);
                let ulps = strays / (f64::EPSILON * largest_coordinate_of(&coarsest));
                if it_lies_flat(&coarsest) {
                    most_ulps_flat = most_ulps_flat.max(ulps);
                    let enclosed = signed_volume_over(&coarsest);
                    rounded_flat[which_side_of_zero(enclosed)] += 1;
                    assert!(
                        enclosed.abs() <= strays * area_of_every_triangle(&coarsest),
                        "{what}: encloses {enclosed:e}, strays {strays:e}"
                    );
                } else {
                    fewest_ulps_bent = fewest_ulps_bent.min(ulps);
                }
                model
                    .validate(&Policy::certified(), &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"));
            }
        }
        assert!(
            most_ulps_flat < 4.0 && fewest_ulps_bent > 1e12,
            "flat meshes stray up to {most_ulps_flat} ulps, bent ones from {fewest_ulps_bent}"
        );
        assert!(
            rounded_flat[0] > 0 && rounded_flat[2] > 0,
            "flat meshes below, at and above zero: {rounded_flat:?}"
        );
        assert!(
            rounded_inside_out[2] > 0,
            "no turn left the cap turned inside out a rounding above zero: {rounded_inside_out:?}"
        );
    }

    fn which_side_of_zero(enclosed: f64) -> usize {
        usize::from(enclosed > 0.0) + usize::from(enclosed >= 0.0)
    }

    fn a_cap_whose_rim_is_three_arcs_placed(turn: &Isometry) -> (Model, crate::SolidKey) {
        let mut model = an_empty_model();
        let cap = a_cap_whose_rim_is_three_arcs(&mut model);
        model
            .place_solid(cap, turn)
            .expect("this model made the key");
        (model, cap)
    }

    fn its_coarsest_mesh(model: &Model, cap: crate::SolidKey, what: &str) -> Mesh {
        the_coarsest_mesh_whose_sides_do_not_cross(
            model.solid(cap).expect("this model made the key"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: {why:?}"))
    }

    /// The cap of [`a_cap_whose_rim_is_three_arcs`] with every face pointing
    /// into it: the lid's plane faces the other way wherever it is named, the
    /// lid and the ball trade rims, and the ball is used reversed. Described
    /// that way to [`SolidBuilder::build`] it is refused before it can be
    /// placed, so it is written in place.
    fn turn_it_inside_out(model: &mut Model) {
        let data = model.data_mut();
        assert_eq!(data.faces.len(), 2, "a cap is a lid and a skin");
        let mut rims = Vec::new();
        for face in data.faces.values_mut() {
            if let Surface::Plane(lid) = face.surface {
                face.surface = Surface::Plane(lid.reversed());
            }
            rims.push(face.wires.clone());
        }
        for (face, rim) in data.faces.values_mut().zip(rims.into_iter().rev()) {
            face.wires = rim;
        }
        for edge in data.edges.values_mut() {
            if let EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(lid),
                ..
            } = &mut edge.run
            {
                *lid = lid.reversed();
            }
        }
        for shell in data.shells.values_mut() {
            for used in &mut shell.faces {
                if data.faces[used.face].surface.as_plane().is_none() {
                    used.orient = used.orient.reversed();
                }
            }
        }
    }

    fn area_of_every_triangle(mesh: &Mesh) -> f64 {
        mesh.triangles()
            .iter()
            .map(|triangle| {
                let [a, b, c] = triangle
                    .corners()
                    .map(|corner| mesh.points()[corner as usize]);
                (b - a).cross(c - a).length() / 2.0
            })
            .sum()
    }

    /// The cap [`a_cap_whose_rim_is_three_arcs_validates_turned_off_the_axes`]
    /// builds: every corner is exact, since `5² + 12² = 13²` and `3² + 4² = 5²`,
    /// and each arc is picked by the planes through the rim's centre and its
    /// two ends.
    fn a_cap_whose_rim_is_three_arcs(model: &mut Model) -> crate::SolidKey {
        let (radius, behind) = (13.0, 12.0);
        let cuts = [(3.0, 4.0), (3.0, -4.0), (-5.0, 0.0)];
        a_cap_with_its_rim_in_arcs(model, radius, behind, &cuts)
            .expect("a cap of a ball is a solid")
    }

    /// A cap of a ball of `radius` centred `behind` the plane `x = 0`, on the
    /// far side of it, its rim cut into arcs at `cuts`, each a `(y, z)` on the
    /// rim, going round from `+z` towards `+y`. Each arc spans under half a
    /// turn.
    fn a_cap_with_its_rim_in_arcs(
        model: &mut Model,
        radius: f64,
        behind: f64,
        cuts: &[(f64, f64)],
    ) -> crate::Result<crate::SolidKey> {
        let lid = Plane::from_point_and_normal(Point::origin(), Vector::new(-1.0, 0.0, 0.0))?;
        let mut builder = SolidBuilder::new();
        let corners: Vec<_> = cuts
            .iter()
            .map(|&(y, z)| builder.add_vertex(Point::new(0.0, y, z)).expect("a point"))
            .collect();
        let mut edges = Vec::new();
        for (at, &(y, z)) in cuts.iter().enumerate() {
            let (ny, nz) = cuts[(at + 1) % cuts.len()];
            let mut picked_by: SmallVec<[Picker; 1]> = SmallVec::new();
            picked_by.push(Picker::from(HalfSpace::through_a_point_facing(
                Point::origin(),
                Vector::new(0.0, z, -y),
            )?));
            if cuts.len() > 2 {
                picked_by.push(Picker::from(HalfSpace::through_a_point_facing(
                    Point::origin(),
                    Vector::new(0.0, -nz, ny),
                )?));
            }
            let run = EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(lid),
                picked_by,
            };
            edges.push(builder.add_edge(corners[at], corners[(at + 1) % cuts.len()], run)?);
        }
        let lid_steps: Vec<_> = edges
            .iter()
            .map(|&edge| {
                DraftStep::Along(builder.add_coedge(edge, Orientation::Forward).expect("e"))
            })
            .collect();
        let skin_steps: Vec<_> = edges
            .iter()
            .rev()
            .map(|&edge| {
                DraftStep::Along(builder.add_coedge(edge, Orientation::Reversed).expect("e"))
            })
            .collect();
        let lid_wire = builder.add_wire(&lid_steps)?;
        let skin_wire = builder.add_wire(&skin_steps)?;
        let skin = builder.add_face(
            Surface::Sphere(Sphere::new(Point::new(-behind, 0.0, 0.0), radius)?),
            &[skin_wire],
        )?;
        let lid_face = builder.add_face(Surface::Plane(lid), &[lid_wire])?;
        builder.add_shell(&[
            (skin, Orientation::Forward),
            (lid_face, Orientation::Forward),
        ])?;
        builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
    }

    /// **Two loops that touch at a corner of the solid touch there at every
    /// room**, so the ladder stops at the first rung that touches where the
    /// coarsest mesh did, and hands that rung's refusal back (Q1866).
    ///
    /// `builder.rs`'s tube has a hole as wide as its wall, and walks each cap's
    /// two circles the same way round from the seam's foot `(2, 0, 0)`, a
    /// corner no chord moves. At height 1 the ladder walked rooms 2, 1 and 0.5,
    /// each touching there, and handed back the last. At height `2⁻³⁵` the two
    /// closest corners are the seam's ends, `2⁻³⁵` apart, and it walked on
    /// towards a floor some thirty-seven halvings down: every rung touched at
    /// `(2, 0, 0)`, the corner the triangulator named there went from number
    /// 6 to number 10507 by room `4.8e-7`, and that rung alone took 36 seconds.
    #[test]
    fn a_tube_whose_hole_is_as_wide_as_its_wall_is_refused_at_the_first_room_it_asks() {
        for height in [1.0, f64::powi(2.0, -35)] {
            let mut model = an_empty_model();
            let tube = a_tube_whose_hole_is_as_wide_as_its_wall(&mut model, height);
            walk_its_hole_the_way_its_rim_goes(&mut model);
            let view = model.solid(tube).expect("this model made the key");
            let handed_back = the_coarsest_mesh_whose_sides_do_not_cross(
                view,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("a hole as wide as its wall touches its rim");
            let half_the_widest_side = Deflection::of(2.0).expect("a positive room");
            let at_the_top = view
                .mesh(
                    half_the_widest_side,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect_err("a hole as wide as its wall touches its rim");
            let (
                Error::CannotMeshAFace { face, why },
                Error::CannotMeshAFace {
                    face: face_at_the_top,
                    why: why_at_the_top,
                },
            ) = (&handed_back, &at_the_top)
            else {
                panic!("height {height}: {handed_back:?} and {at_the_top:?}");
            };
            assert_eq!(
                (face, why),
                (face_at_the_top, why_at_the_top),
                "height {height}"
            );
        }
    }

    /// A tube `height` tall whose hole is as wide as its wall, both pipes of
    /// radius 2, wound as a sound tube is: the hole's wall used reversed, and
    /// each cap's hole walked against its rim. [`SolidBuilder::build`] sweeps
    /// it and refuses it, so it is built with a hole of radius 1 and the hole
    /// is widened in place: its two corners moved out to 2, and its wall's
    /// pipe with them.
    fn a_tube_whose_hole_is_as_wide_as_its_wall(model: &mut Model, height: f64) -> crate::SolidKey {
        let (outer_radius, inner_radius) = (2.0, 1.0);
        let mut builder = SolidBuilder::new();
        let mut corner = |radius, z| {
            builder
                .add_vertex(Point::new(radius, 0.0, z))
                .expect("a finite point")
        };
        let outer_bottom = corner(outer_radius, 0.0);
        let outer_top = corner(outer_radius, height);
        let inner_bottom = corner(inner_radius, 0.0);
        let inner_top = corner(inner_radius, height);

        let mut circle = |corner| {
            builder
                .add_edge(corner, corner, EdgeRun::RightRoundWhereItsFacesCross)
                .expect("one whole circle of the two faces crossing there")
        };
        let outer_bottom_circle = circle(outer_bottom);
        let outer_top_circle = circle(outer_top);
        let inner_bottom_circle = circle(inner_bottom);
        let inner_top_circle = circle(inner_top);
        let outer_seam = builder
            .add_edge(outer_bottom, outer_top, EdgeRun::Straight)
            .expect("a ruling of a pipe is straight");
        let inner_seam = builder
            .add_edge(inner_bottom, inner_top, EdgeRun::Straight)
            .expect("a ruling of a pipe is straight");

        let mut step = |edge, turn| {
            DraftStep::Along(
                builder
                    .add_coedge(edge, turn)
                    .expect("an edge of this build"),
            )
        };
        let round_the_outer_wall = [
            step(outer_bottom_circle, Orientation::Forward),
            step(outer_seam, Orientation::Forward),
            step(outer_top_circle, Orientation::Reversed),
            step(outer_seam, Orientation::Reversed),
        ];
        let round_the_inner_wall = [
            step(inner_bottom_circle, Orientation::Reversed),
            step(inner_seam, Orientation::Forward),
            step(inner_top_circle, Orientation::Forward),
            step(inner_seam, Orientation::Reversed),
        ];
        let round_the_outer_bottom = [step(outer_bottom_circle, Orientation::Reversed)];
        let round_the_inner_bottom = [step(inner_bottom_circle, Orientation::Forward)];
        let round_the_outer_top = [step(outer_top_circle, Orientation::Forward)];
        let round_the_inner_top = [step(inner_top_circle, Orientation::Reversed)];

        let mut face = |surface, wires: &[&[DraftStep]]| {
            let wires: Vec<_> = wires
                .iter()
                .map(|steps| builder.add_wire(steps).expect("a closed wire"))
                .collect();
            builder.add_face(surface, &wires).expect("a face")
        };
        let pipe = |radius| {
            Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a positive radius"))
        };
        let cap = |z, up| {
            Surface::Plane(
                Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, up))
                    .expect("the z axis is a unit normal"),
            )
        };
        let outer_wall = face(pipe(outer_radius), &[&round_the_outer_wall]);
        let inner_wall = face(pipe(inner_radius), &[&round_the_inner_wall]);
        let bottom = face(
            cap(0.0, -1.0),
            &[&round_the_outer_bottom, &round_the_inner_bottom],
        );
        let top = face(
            cap(height, 1.0),
            &[&round_the_outer_top, &round_the_inner_top],
        );
        builder
            .add_shell(&[
                (outer_wall, Orientation::Forward),
                (inner_wall, Orientation::Reversed),
                (bottom, Orientation::Forward),
                (top, Orientation::Forward),
            ])
            .expect("four faces close");
        let tube = builder
            .build(model, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a tube with a hole of radius 1 in a pipe of radius 2");

        let data = model.data_mut();
        let mut moved = 0;
        for corner in data.vertices.values_mut() {
            let [x, y, z] = corner.point.coordinates();
            if x == inner_radius {
                corner.point = Point::new(outer_radius, y, z);
                moved += 1;
            }
        }
        let mut widened = 0;
        for wall in data.faces.values_mut() {
            if matches!(&wall.surface, Surface::Cylinder(hole) if hole.radius() == inner_radius) {
                wall.surface = pipe(outer_radius);
                widened += 1;
            }
        }
        assert_eq!(
            (moved, widened),
            (2, 1),
            "the hole's two corners and its wall"
        );
        tube
    }

    /// Walks the hole of [`a_tube_whose_hole_is_as_wide_as_its_wall`] the way
    /// `builder.rs`'s tube does: each circle of the hole the same way round as
    /// the rim beside it, and the hole's wall used forward like the outer one.
    /// Both loops of a cap then set off from the seam's foot the same way.
    fn walk_its_hole_the_way_its_rim_goes(model: &mut Model) {
        let data = model.data_mut();
        let mut hole_walls = 0;
        let mut circles_round_the_hole = Vec::new();
        for shell in data.shells.values_mut() {
            for used in &mut shell.faces {
                if used.orient == Orientation::Reversed {
                    used.orient = Orientation::Forward;
                    hole_walls += 1;
                    for wire in &data.faces[used.face].wires {
                        for coedge in data.wires[*wire].coedges() {
                            let edge = data.coedges[coedge].edge;
                            if data.edges[edge].start == data.edges[edge].end {
                                circles_round_the_hole.push(edge);
                            }
                        }
                    }
                }
            }
        }
        let mut flipped = 0;
        for coedge in data.coedges.values_mut() {
            if circles_round_the_hole.contains(&coedge.edge) {
                coedge.orientation = coedge.orientation.reversed();
                flipped += 1;
            }
        }
        assert_eq!(
            (hole_walls, circles_round_the_hole.len(), flipped),
            (1, 2, 4),
            "the hole's wall, its two circles, and each circle's wall and cap"
        );
    }

    /// Row Q1890. The lens a ball makes poking `1/16` into a pipe of radius 5
    /// about `z`: the ball's face below and the pipe's above, meeting in a rim
    /// of four corners that do not lie in one plane, the two on `z = 0` lower
    /// by `1/16` than the two on `x = 0`.
    ///
    /// **A mesh that does not lie flat can enclose the wrong sign.** At the
    /// coarsest, each face is cut into triangles across the rim on its own, so
    /// where the two faces join other corners the mesh is a thin solid of
    /// either sign. Built turned `8/40` of a turn about `x`, the sound lens's
    /// coarsest mesh is twelve points and encloses `−0.041`, where the mesh
    /// that answers, at room `0.0297`, encloses `0.188`. It did not lie flat,
    /// so that sum was read as the lens's volume: the lens was refused
    /// [`Invalid::VolumeIsNotPositive`] and the same lens turned inside out
    /// passed. The space that mesh can leave out is `2.2`, so its sum says
    /// nothing about the sign.
    ///
    /// **It is built where it sits, not placed.** A placed curved face is
    /// meshed where it is stored, so placing the lens moves its mesh with it
    /// and never changes which corners the triangles join.
    #[test]
    fn a_lens_where_a_ball_pokes_into_a_pipe_validates_at_every_turn() {
        let mut coarsest_below_zero = 0;
        for fortieths in 0..40 {
            let what = format!("turned {fortieths}/40 about x");
            let turn = Isometry::rotation(
                &Axis::x(),
                f64::from(fortieths) * core::f64::consts::TAU / 40.0,
            )
            .expect("a finite turn");

            let mut model = an_empty_model();
            let lens = a_lens_where_a_ball_pokes_into_a_pipe(&mut model, &turn, false)
                .unwrap_or_else(|why| panic!("{what}: {why:?}"));
            if signed_volume_over(&its_coarsest_mesh(&model, lens, &what)) < 0.0 {
                coarsest_below_zero += 1;
            }
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: {why:?}"));

            let inside_out =
                a_lens_where_a_ball_pokes_into_a_pipe(&mut an_empty_model(), &turn, true);
            assert!(
                matches!(
                    inside_out,
                    Err(Error::Invalid(Invalid::VolumeIsNotPositive { .. }))
                ),
                "{what}: inside out {inside_out:?}"
            );
        }
        assert!(
            coarsest_below_zero > 0,
            "no turn gave the sound lens a coarsest mesh that encloses below zero"
        );
    }

    /// The lens of [`a_lens_where_a_ball_pokes_into_a_pipe_validates_at_every_turn`]
    /// built turned by `turn`, and with both faces pointing into it when
    /// `inside_out`. Its corners are where the rim crosses `x = 0` and `z = 0`,
    /// and each edge between two of them is picked by the half-space facing
    /// their quadrant, which holds both corners strictly inside it at any turn.
    fn a_lens_where_a_ball_pokes_into_a_pipe(
        model: &mut Model,
        turn: &Isometry,
        inside_out: bool,
    ) -> crate::Result<crate::SolidKey> {
        let (pipe_radius, ball_at, rim_low) = (5.0f64, 8.0f64, 4.9375f64);
        let across = (pipe_radius * pipe_radius - rim_low * rim_low).sqrt();
        let along = (2.0 * ball_at * (pipe_radius - rim_low)).sqrt();
        let ball_radius = (across * across + (ball_at - rim_low) * (ball_at - rim_low)).sqrt();
        let pipe = Cylinder::new(Axis::z(), pipe_radius)?.transformed(turn);
        let ball = Sphere::new(Point::new(0.0, ball_at, 0.0), ball_radius)?.transformed(turn);
        let mut builder = SolidBuilder::new();
        let corners = [
            (across, rim_low, 0.0),
            (0.0, pipe_radius, along),
            (-across, rim_low, 0.0),
            (0.0, pipe_radius, -along),
        ]
        .map(|(x, y, z)| {
            builder
                .add_vertex(turn.apply_to_point(Point::new(x, y, z)))
                .expect("a finite corner")
        });
        let quadrants = [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)];
        let mut edges = Vec::new();
        for (at, (x, z)) in quadrants.into_iter().enumerate() {
            let mut picked_by: SmallVec<[Picker; 1]> = SmallVec::new();
            picked_by.push(Picker::from(HalfSpace::through_a_point_facing(
                turn.apply_to_point(Point::origin()),
                turn.apply_to_vector(Vector::new(x, 0.0, z)),
            )?));
            let run = EdgeRun::PartOfWhereItCrosses {
                and: Surface::Cylinder(pipe),
                picked_by,
            };
            edges.push(builder.add_edge(corners[at], corners[(at + 1) % 4], run)?);
        }
        let way = if inside_out {
            Orientation::Reversed
        } else {
            Orientation::Forward
        };
        let mut the_rim_walked = |way: Orientation| -> Vec<_> {
            let mut ordered = edges.clone();
            if way == Orientation::Reversed {
                ordered.reverse();
            }
            ordered
                .into_iter()
                .map(|edge| {
                    DraftStep::Along(
                        builder
                            .add_coedge(edge, way)
                            .expect("an edge this builder made"),
                    )
                })
                .collect()
        };
        let ball_steps = the_rim_walked(way);
        let pipe_steps = the_rim_walked(way.reversed());
        let ball_wire = builder.add_wire(&ball_steps)?;
        let pipe_wire = builder.add_wire(&pipe_steps)?;
        let ball_face = builder.add_face(Surface::Sphere(ball), &[ball_wire])?;
        let pipe_face = builder.add_face(Surface::Cylinder(pipe), &[pipe_wire])?;
        builder.add_shell(&[(ball_face, way), (pipe_face, way)])?;
        builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
    }
}
