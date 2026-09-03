use core::sync::atomic::{AtomicU64, Ordering};

use rcad_geom::{EdgeRun, Isometry, Orientation, Point, Surface};
use rcad_predicates::{Policy, ProvenanceLog};
use smallvec::SmallVec;

use crate::key::{CoedgeSlot, EdgeSlot, FaceSlot, SolidSlot, VertexSlot, WireSlot};
use crate::model::{FaceUse, ShellUse, WireStep};
use crate::{Error, Model, Result, SolidKey};

/// A solid described the way a file describes one, checked as a whole, and only
/// then written into a [`Model`].
///
/// Points, then edges between them, then uses of those edges running one way or
/// the other, then wires, then faces, then the one shell that closes it up.
/// That is the order an AP203 `manifold_solid_brep` is laid out in, and this is
/// the public route §42.5 says the kernel did not have: nothing outside
/// `rcad-topo` could reach an arena, so an importer had no way to turn
/// `advanced_face` and `closed_shell` into a solid.
///
/// # Nothing half-built ever reaches a model
///
/// A description is checked twice before a single entity is written. Every call
/// that can decide something from its own arguments decides it there — a wire
/// that does not close, a coedge already walked by another wire — so the
/// refusal names the entity the caller was working on. Everything that takes
/// the whole solid to see is checked at [`SolidBuilder::build`], which writes
/// the description into a model of its own, runs [`Model::validate`] over that,
/// and only writes into the caller's model once the sweep is happy.
///
/// So the sweep is the one authority on what a sound solid is, and a build that
/// fails leaves the caller's model exactly as it found it. That costs one
/// throwaway copy of the solid per build, which is linear in the solid.
///
/// # World coordinates, and no placement
///
/// Everything written here is stored at the identity and the numbers are the
/// ones handed over, which is §42.8's decision to bake. A caller placing one
/// shape at several occurrences moves the points itself and describes the shape
/// once per occurrence: N occurrences are N solids and cost N times the
/// entities.
///
/// **There is no placement argument, and that is a gap rather than a limit**
/// (§16.10). It used to be an arithmetic gap: an
/// [`EdgeRun::PartOfWhereItCrosses`] carries [`Picker`](rcad_geom::Picker)s, and
/// `HalfSpace` and `Picker` had `shifted` and nothing else, so a `build` taking
/// an [`Isometry`] could not move an arc's own name with it. That half is
/// built — [`HalfSpace::transformed`](rcad_geom::HalfSpace::transformed),
/// [`Picker::transformed`](rcad_geom::Picker::transformed) and
/// [`EdgeRun::transformed`](rcad_geom::EdgeRun::transformed) all take
/// `(n, d)` to `(Rn, d + Rn·t)`, exact wherever `R` is — so what is left is
/// entirely in this crate: moving each point and each run on the way in, and
/// saying which of the movement's roundings the caller is buying. What calls it
/// due is a reader that wants to place an occurrence without redescribing its
/// points.
///
/// # Errors
///
/// Everything a builder is handed is caller input, so every misuse is a named
/// [`Error`] and none of them panics. That is CLAUDE.md's rule applied to an
/// API before it exists rather than to its clauses afterwards (§42.5).
#[derive(Clone, Debug)]
pub struct SolidBuilder {
    id: BuilderId,
    points: Vec<Point>,
    vertex_is_used: Vec<bool>,
    edges: Vec<DraftEdgeData>,
    coedges: Vec<DraftCoedgeData>,
    coedge_is_used: Vec<bool>,
    wires: Vec<Vec<Step>>,
    wire_is_used: Vec<bool>,
    faces: Vec<DraftFaceData>,
    face_is_used: Vec<bool>,
    shell: Option<Vec<(usize, Orientation)>>,
}

impl SolidBuilder {
    /// An empty description.
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: BuilderId::next(),
            points: Vec::new(),
            vertex_is_used: Vec::new(),
            edges: Vec::new(),
            coedges: Vec::new(),
            coedge_is_used: Vec::new(),
            wires: Vec::new(),
            wire_is_used: Vec::new(),
            faces: Vec::new(),
            face_is_used: Vec::new(),
            shell: None,
        }
    }

    /// Describe a corner, in the model's own units and already in world
    /// coordinates.
    ///
    /// # Errors
    ///
    /// [`Error::ADraftPointIsNotFinite`] if a coordinate is infinite or is not
    /// a number.
    pub fn add_vertex(&mut self, point: Point) -> Result<DraftVertex> {
        if !point.is_finite() {
            return Err(Error::ADraftPointIsNotFinite {
                vertex: self.points.len(),
            });
        }
        self.points.push(point);
        self.vertex_is_used.push(false);
        Ok(DraftVertex::new(self.points.len() - 1, self.id))
    }

    /// Describe an edge between two corners, and what it runs along between
    /// them.
    ///
    /// [`EdgeRun::Straight`] is a straight line, and the two ends are the whole
    /// of it. [`EdgeRun::RightRoundWhereItsFacesCross`] is a whole closed curve
    /// and starts and ends at the one corner it has.
    /// [`EdgeRun::PartOfWhereItCrosses`] is an arc, and carries the second
    /// surface and the half-spaces that say which arc.
    ///
    /// # Errors
    ///
    /// [`Error::ADraftHandleCameFromAnotherBuilder`] if either end was made
    /// elsewhere, and [`Error::ADraftEdgeHasNoLength`] if the two ends are in
    /// one place and the run does not say the edge comes back there — see
    /// [`the_ends_are_the_whole_of_it`].
    pub fn add_edge(
        &mut self,
        start: DraftVertex,
        end: DraftVertex,
        run: EdgeRun,
    ) -> Result<DraftEdge> {
        let start = start.index_in(self.id)?;
        let end = end.index_in(self.id)?;
        let edge = self.edges.len();
        let both_ends_in_one_place = start == end || self.points[start] == self.points[end];
        if both_ends_in_one_place && the_ends_are_the_whole_of_it(&run) {
            return Err(Error::ADraftEdgeHasNoLength { edge });
        }
        self.vertex_is_used[start] = true;
        self.vertex_is_used[end] = true;
        self.edges.push(DraftEdgeData { start, end, run });
        Ok(DraftEdge::new(edge, self.id))
    }

    /// Describe one use of an edge by one wire, running one way round it.
    ///
    /// # Errors
    ///
    /// [`Error::ADraftHandleCameFromAnotherBuilder`] if the edge was made
    /// elsewhere.
    pub fn add_coedge(&mut self, edge: DraftEdge, orientation: Orientation) -> Result<DraftCoedge> {
        let edge = edge.index_in(self.id)?;
        self.coedges.push(DraftCoedgeData { edge, orientation });
        self.coedge_is_used.push(false);
        Ok(DraftCoedge::new(self.coedges.len() - 1, self.id))
    }

    /// Describe a closed chain of steps round the boundary of a face.
    ///
    /// # Errors
    ///
    /// [`Error::ADraftHandleCameFromAnotherBuilder`] if a step names something
    /// made elsewhere, [`Error::ADraftEntityIsUsedTwice`] if a coedge is
    /// already walked by another wire or by this one twice,
    /// [`Error::ADraftWireIsTooShort`] if there are too few steps to close
    /// round anything, and [`Error::ADraftWireDoesNotClose`] if the steps do
    /// not join up end to end.
    pub fn add_wire(&mut self, steps: &[DraftStep]) -> Result<DraftWire> {
        let mut resolved = Vec::with_capacity(steps.len());
        for step in steps {
            resolved.push(match *step {
                DraftStep::Along(coedge) => Step::Along(coedge.index_in(self.id)?),
                DraftStep::Pole(vertex) => Step::Pole(vertex.index_in(self.id)?),
            });
        }
        for (position, step) in resolved.iter().enumerate() {
            let Step::Along(coedge) = *step else {
                continue;
            };
            let walked_here_already = resolved[..position]
                .iter()
                .any(|earlier| matches!(*earlier, Step::Along(other) if other == coedge));
            if self.coedge_is_used[coedge] || walked_here_already {
                return Err(Error::ADraftEntityIsUsedTwice {
                    entity: "coedge",
                    index: coedge,
                });
            }
        }
        self.check_the_steps_close(&resolved)?;

        for step in &resolved {
            match *step {
                Step::Along(coedge) => self.coedge_is_used[coedge] = true,
                Step::Pole(vertex) => self.vertex_is_used[vertex] = true,
            }
        }
        self.wires.push(resolved);
        self.wire_is_used.push(false);
        Ok(DraftWire::new(self.wires.len() - 1, self.id))
    }

    /// Describe a patch of surface, trimmed by its wires.
    ///
    /// The first wire is the outer one and the rest are holes.
    ///
    /// # Errors
    ///
    /// [`Error::ADraftHandleCameFromAnotherBuilder`] if a wire was made
    /// elsewhere, [`Error::ADraftFaceHasNoWires`] if there are none, and
    /// [`Error::ADraftEntityIsUsedTwice`] if a wire already trims another face.
    pub fn add_face(&mut self, surface: Surface, wires: &[DraftWire]) -> Result<DraftFace> {
        if wires.is_empty() {
            return Err(Error::ADraftFaceHasNoWires);
        }
        let mut resolved: Vec<usize> = Vec::with_capacity(wires.len());
        for &wire in wires {
            let wire = wire.index_in(self.id)?;
            if self.wire_is_used[wire] || resolved.contains(&wire) {
                return Err(Error::ADraftEntityIsUsedTwice {
                    entity: "wire",
                    index: wire,
                });
            }
            resolved.push(wire);
        }

        for &wire in &resolved {
            self.wire_is_used[wire] = true;
        }
        self.faces.push(DraftFaceData {
            surface,
            wires: resolved,
        });
        self.face_is_used.push(false);
        Ok(DraftFace::new(self.faces.len() - 1, self.id))
    }

    /// Describe the closed skin of faces that bounds the solid, each face with
    /// the way round this shell uses it.
    ///
    /// [`Orientation::Forward`] means the face's own surface points out of the
    /// material, which is a STEP `advanced_face` with `same_sense` true.
    ///
    /// # Errors
    ///
    /// [`Error::ASolidBuiltFaceByFaceHasOneShell`] if a shell was already
    /// described, [`Error::ADraftShellHasNoFaces`] if the list is empty,
    /// [`Error::ADraftHandleCameFromAnotherBuilder`] if a face was made
    /// elsewhere, and [`Error::ADraftEntityIsUsedTwice`] if a face appears more
    /// than once.
    pub fn add_shell(&mut self, faces: &[(DraftFace, Orientation)]) -> Result<()> {
        if self.shell.is_some() {
            return Err(Error::ASolidBuiltFaceByFaceHasOneShell);
        }
        if faces.is_empty() {
            return Err(Error::ADraftShellHasNoFaces);
        }
        let mut resolved: Vec<(usize, Orientation)> = Vec::with_capacity(faces.len());
        for &(face, orientation) in faces {
            let face = face.index_in(self.id)?;
            if self.face_is_used[face] || resolved.iter().any(|&(other, _)| other == face) {
                return Err(Error::ADraftEntityIsUsedTwice {
                    entity: "face",
                    index: face,
                });
            }
            resolved.push((face, orientation));
        }

        for &(face, _) in &resolved {
            self.face_is_used[face] = true;
        }
        self.shell = Some(resolved);
        Ok(())
    }

    /// Check the whole description and write it into `model` as one solid.
    ///
    /// The description is written into a throwaway model first and
    /// [`Model::validate`] is run over that, so the answer to "is this a sound
    /// solid" comes from the sweep and from nowhere else. A refusal leaves
    /// `model` untouched, down to the last slot.
    ///
    /// Records from the sweep land in `log` whether the build went on to
    /// succeed or not. A description that needed nothing approximated leaves it
    /// untouched, and that empty log is itself the signal (decisions.md §5).
    ///
    /// # Errors
    ///
    /// [`Error::ADraftSolidHasNoShell`] if nothing was described to build,
    /// [`Error::ADraftEntityIsUsedByNothing`] for anything the shell does not
    /// reach, [`Error::ADraftEdgeIsNotWalkedByTwoCoedges`] and
    /// [`Error::ADraftEdgeIsWalkedTheSameWayTwice`] where the skin does not
    /// close along an edge, and then whatever [`Model::validate`] makes of the
    /// solid — which is where the geometry is judged: a corner off its own
    /// surface, a wire winding the wrong way, a shell whose faces point inwards.
    ///
    /// # Panics
    ///
    /// If the solid it just wrote breaks one of the invariants checked on
    /// touched entities. The sweep passed on the same description moments
    /// earlier, so that is a bug here and not bad input.
    pub fn build(
        self,
        model: &mut Model,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<SolidKey> {
        let Some(shell) = self.shell.as_deref() else {
            return Err(Error::ADraftSolidHasNoShell);
        };
        self.check_nothing_is_left_over()?;
        self.check_every_edge_is_walked_once_each_way()?;

        let mut scratch = Model::new(model.scale());
        self.write_into(&mut scratch, shell);
        let mut mine = ProvenanceLog::new();
        let swept = scratch.validate(policy, &mut mine);
        mine.drain_into(log);
        swept?;

        let solid = self.write_into(model, shell);
        model.check_touched_solid(solid);
        Ok(model.key_for_solid(solid))
    }

    /// Every step starting where the one before it left off, and the last one
    /// coming back to where the first set off.
    ///
    /// A pole starts and ends at its own corner, because it goes nowhere: the
    /// surface has pinched to a point and the boundary is standing on it. So
    /// closure is one rule rather than two.
    fn check_the_steps_close(&self, steps: &[Step]) -> Result<()> {
        if steps.len() < 3
            && !self.closes_on_one_edge(steps)
            && !self.two_steps_that_bound_a_face(steps)
        {
            return Err(Error::ADraftWireIsTooShort { steps: steps.len() });
        }
        let mut previous_end = None;
        for (position, step) in steps.iter().enumerate() {
            let (start, end) = self.step_ends(*step);
            if previous_end.is_some_and(|previous| previous != start) {
                return Err(Error::ADraftWireDoesNotClose { position });
            }
            previous_end = Some(end);
        }
        if previous_end != Some(self.step_ends(steps[0]).0) {
            return Err(Error::ADraftWireDoesNotClose {
                position: steps.len(),
            });
        }
        Ok(())
    }

    /// Whether these steps are one coedge on an edge that already comes back to
    /// where it started — a round cap's boundary, which is one whole circle.
    fn closes_on_one_edge(&self, steps: &[Step]) -> bool {
        let [Step::Along(only)] = steps[..] else {
            return false;
        };
        self.edges[self.coedges[only].edge].run == EdgeRun::RightRoundWhereItsFacesCross
    }

    /// Whether these steps are two that go somewhere and come back a different
    /// way, which is a face — an arc and the chord that closes it, or two arcs
    /// bulging opposite ways.
    ///
    /// The rule is about the edges rather than about the count. **One edge
    /// walked out and back** retraces the same curve, whatever that curve is.
    /// **Two straight edges between the same two corners** are the same segment
    /// written twice, since a straight edge is the whole of what its two ends
    /// say (see [`EdgeRun::Straight`]). Both sweep exactly no area.
    ///
    /// **Those are the two this can see, and not every two-step wire that
    /// bounds nothing.** Two *curved* edges can name the same curve as well,
    /// and equal [`EdgeRun`]s do not say so: a run names a piece of where the
    /// faces **either side** of the edge cross, so the same run between
    /// different pairs of faces is a different curve — the segment plate's arc
    /// and its chord carry equal runs and are a circle and a straight line. A
    /// wire is described before the face it trims, so the pair is not known
    /// here at all. Two edges that really do name the same curve are refused by
    /// the sweep's mesh instead, as `CornersCoincide`; that is not new to two
    /// steps, and a three-step wire holding such a pair is refused there too.
    ///
    /// **A run this crate does not know reads as one that bends, and that is the
    /// right way round here** — the opposite of what
    /// [`the_ends_are_the_whole_of_it`] does with the same unknown, and the
    /// difference is what stands behind each.
    ///
    /// There, the length rule is the *only* thing that asks, so a run read
    /// generously is a zero-length edge nobody looks at again: measured on a
    /// clone of this tree carrying a fourth [`EdgeRun`], `check_edges` handed
    /// back `Ok(())` for a cube with one edge pointed at the corner it left.
    ///
    /// Here nothing is being decided about the shape. All this says is whether a
    /// two-step wire may skip the floor of three steps, and every other check
    /// still runs over it — a wire that really does bound nothing is refused by
    /// the mesh, by name, and
    /// `tests/a_face_bounded_by_two_steps.rs`'s
    /// `a_two_step_face_whose_two_edges_are_the_same_curve_is_caught_by_the_mesh`
    /// is the test that says so rather than this paragraph. Reading the unknown
    /// strictly would buy nothing and cost a real refusal: a new run kind names
    /// a curve, which is exactly the thing two steps *can* bound a face with, and
    /// the refusal it would get is [`Error::ADraftWireIsTooShort`] — a complaint
    /// about how many steps there are, when the number of steps was never the
    /// problem.
    ///
    /// `docs/decisions.md` §44.11 item 3. This and
    /// [`Model::wire_of_two_steps_bounds_a_face`](crate::Model) are the same
    /// rule at the kernel's two doors, for the reason
    /// [`Model::check_wire_closes`](crate::Model) gives: one audience wants a
    /// refusal and the other a crash, and that is no reason for two answers.
    /// Both read an unknown run the same way, so there is nothing to bring back
    /// into step.
    fn two_steps_that_bound_a_face(&self, steps: &[Step]) -> bool {
        let [Step::Along(one), Step::Along(other)] = steps[..] else {
            return false;
        };
        let (one, other) = (self.coedges[one].edge, self.coedges[other].edge);
        one != other
            && (self.edges[one].run != EdgeRun::Straight
                || self.edges[other].run != EdgeRun::Straight)
    }

    /// Where one step round a wire starts and where it leaves off.
    fn step_ends(&self, step: Step) -> (usize, usize) {
        match step {
            Step::Along(coedge) => {
                let coedge = &self.coedges[coedge];
                let edge = &self.edges[coedge.edge];
                if coedge.orientation.is_reversed() {
                    (edge.end, edge.start)
                } else {
                    (edge.start, edge.end)
                }
            }
            Step::Pole(vertex) => (vertex, vertex),
        }
    }

    /// Nothing described that the shell does not reach.
    ///
    /// The sweep refuses these too, by [`Invalid::Orphan`](crate::Invalid), and
    /// what this adds is *which* one: a file names its entities and an importer
    /// that can only say "something is unused" cannot point at the line.
    fn check_nothing_is_left_over(&self) -> Result<()> {
        let mut edge_is_used = vec![false; self.edges.len()];
        for coedge in &self.coedges {
            edge_is_used[coedge.edge] = true;
        }
        let flagged: [(&'static str, &[bool]); 5] = [
            ("vertex", &self.vertex_is_used),
            ("edge", &edge_is_used),
            ("coedge", &self.coedge_is_used),
            ("wire", &self.wire_is_used),
            ("face", &self.face_is_used),
        ];
        for (entity, flags) in flagged {
            if let Some(index) = flags.iter().position(|used| !used) {
                return Err(Error::ADraftEntityIsUsedByNothing { entity, index });
            }
        }
        Ok(())
    }

    /// Every edge walked by exactly two coedges running opposite ways, which is
    /// what says the skin closes along it.
    fn check_every_edge_is_walked_once_each_way(&self) -> Result<()> {
        let mut walked: Vec<SmallVec<[Orientation; 2]>> = vec![SmallVec::new(); self.edges.len()];
        for coedge in &self.coedges {
            walked[coedge.edge].push(coedge.orientation);
        }
        for (edge, senses) in walked.iter().enumerate() {
            match senses.as_slice() {
                [one, other] if one != other => {}
                [_, _] => return Err(Error::ADraftEdgeIsWalkedTheSameWayTwice { edge }),
                _ => {
                    return Err(Error::ADraftEdgeIsNotWalkedByTwoCoedges {
                        edge,
                        count: senses.len(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Write the description into an arena, in the order it was described.
    ///
    /// Called twice per build — once into the model the sweep judges and once
    /// into the caller's — and that is the point. The alternative was to write
    /// once and then move the entities across, which is a second piece of code
    /// that has to agree with this one about what a solid's arena looks like.
    /// This walks the same description in the same order into a fresh arena
    /// twice, so the only way the two can differ is if this function is not a
    /// function of its arguments.
    fn write_into(&self, model: &mut Model, shell: &[(usize, Orientation)]) -> SolidSlot {
        let vertices: Vec<VertexSlot> = self
            .points
            .iter()
            .map(|&point| model.add_vertex(point))
            .collect();
        let edges: Vec<EdgeSlot> = self
            .edges
            .iter()
            .map(|edge| {
                model.add_edge_running(vertices[edge.start], vertices[edge.end], edge.run.clone())
            })
            .collect();
        let coedges: Vec<CoedgeSlot> = self
            .coedges
            .iter()
            .map(|coedge| model.add_coedge(edges[coedge.edge], coedge.orientation))
            .collect();
        let wires: Vec<WireSlot> = self
            .wires
            .iter()
            .map(|steps| {
                let steps: SmallVec<[WireStep; 4]> = steps
                    .iter()
                    .map(|step| match *step {
                        Step::Along(coedge) => WireStep::Along(coedges[coedge]),
                        Step::Pole(vertex) => WireStep::Pole(vertices[vertex]),
                    })
                    .collect();
                model.add_wire_of_steps(steps)
            })
            .collect();
        let faces: Vec<FaceSlot> = self
            .faces
            .iter()
            .map(|face| {
                let trimmed: SmallVec<[WireSlot; 1]> =
                    face.wires.iter().map(|&wire| wires[wire]).collect();
                model.add_face(face.surface, trimmed)
            })
            .collect();

        let skin = model.add_shell(
            shell
                .iter()
                .map(|&(face, orient)| FaceUse {
                    face: faces[face],
                    loc: Isometry::identity(),
                    orient,
                })
                .collect(),
        );
        model.add_solid(SmallVec::from_buf([ShellUse {
            shell: skin,
            loc: Isometry::identity(),
            orient: Orientation::Forward,
        }]))
    }
}

/// Whether an edge's two ends are the whole of what says where it runs.
///
/// Only [`EdgeRun::Straight`] says that. A run that names a curve puts the edge
/// where its ends do not, so two ends in one place is a closed loop and not an
/// edge with no length.
///
/// **A kind of run this crate does not know is not taken for one of those.**
/// [`EdgeRun`] is `#[non_exhaustive]` and lives in `rcad-geom`, so a fourth kind
/// arrives here with no compile error at all — a clone of this tree carrying one
/// checks clean — and reading it as "not straight" would walk an edge with its
/// two ends in one place through a public door. Refusing it names the edge and is
/// one word here to undo; letting it in is a zero-length edge inside a model
/// somebody will go on to mesh.
///
/// **This is the one list, and both of the kernel's doors read it.**
/// [`SolidBuilder::add_edge`] asks it of a description, and `check_edges` in
/// [`Model::validate`](crate::Model::validate) asks it of an edge already in an
/// arena, so [`Model::add_edge_running`](crate::Model::add_edge_running) cannot
/// keep what [`SolidBuilder::add_edge`] refuses. It was two lists until this
/// unit, and they had already drifted apart on exactly the case above: the sweep
/// asked `run == Straight`, so a run it did not know read as *not straight* and
/// the whole check was skipped. Two copies of a rule that must agree is the drift
/// waiting to happen, so there is one copy (`docs/decisions.md` §44.11, one rule
/// at the kernel's two doors).
///
/// Written as a match with all three kinds named rather than as a `matches!`,
/// because a wildcard that covers nothing today is what `just check-wildcards`
/// starts counting the day it covers something: measured on that same clone,
/// `cargo clippy -p rcad-topo --all-targets -- -W clippy::wildcard_enum_match_arm`
/// says nothing about this line now and names it the moment there is a fourth
/// kind. A `matches!` is invisible to the lint either way. One list is also one
/// warning to answer instead of two, with no way to answer one and forget the
/// other.
///
/// `tests/a_fourth_kind_of_run.rs` is what says a fourth kind arrived. What each
/// door then does about it cannot be tested from inside this tree at all, and was
/// measured on the clone instead: with a fourth kind forged into `rcad-geom`, a
/// cube with one edge pointed back at the corner it left gets `Ok(())` out of
/// `check_edges` when the sweep asks `run == Straight`, and
/// `Invalid::EdgeHasNoLength` when it asks this. `add_edge` says
/// `ADraftEdgeHasNoLength` either way.
///
/// `validate::tests::a_collapsed_edge_is_kept_only_when_its_run_names_a_curve`
/// pins the three kinds that do exist, in both directions.
pub(crate) fn the_ends_are_the_whole_of_it(run: &EdgeRun) -> bool {
    match run {
        EdgeRun::Straight => true,
        EdgeRun::RightRoundWhereItsFacesCross | EdgeRun::PartOfWhereItCrosses { .. } => false,
        _ => true,
    }
}

impl Default for SolidBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// One step round a wire being described.
///
/// A wire is almost always a chain of coedges. What a ball adds is the pole:
/// the surface pinches to a point, the boundary has to go somewhere between
/// arriving there and leaving, and where it goes is nowhere at all.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DraftStep {
    /// Along one use of an edge.
    Along(DraftCoedge),
    /// Standing still at a pole, where the surface pinches to a point.
    Pole(DraftVertex),
}

/// An edge of a description: its two ends and what it runs along between them.
#[derive(Clone, Debug)]
struct DraftEdgeData {
    start: usize,
    end: usize,
    run: EdgeRun,
}

/// One use of an edge by one wire, running one way round.
#[derive(Clone, Copy, Debug)]
struct DraftCoedgeData {
    edge: usize,
    orientation: Orientation,
}

/// A patch of surface and the wires that trim it, the first one the outer.
#[derive(Clone, Debug)]
struct DraftFaceData {
    surface: Surface,
    wires: Vec<usize>,
}

/// One step round a wire, once its handles have been resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Along(usize),
    Pole(usize),
}

/// Which builder a handle belongs to.
///
/// A handle is an index, so the same index in a second builder resolves
/// silently to a different entity. Carried in every build profile and not only
/// in debug, because the answer to a foreign handle is an [`Error`] a caller
/// can match on rather than a panic, and an error that appears only in one
/// profile is worse than none (CLAUDE.md's rule on bad caller input).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct BuilderId(u64);

impl BuilderId {
    fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

macro_rules! define_handle {
    ($handle:ident, $what:literal) => {
        #[doc = concat!("A handle on one ", $what, " described in a [`SolidBuilder`].")]
        ///
        /// Every kind of entity has its own handle type, so handing a face
        /// where an edge belongs is a compile error rather than a silent lookup
        /// of the wrong thing. A handle also remembers which builder made it,
        /// and using it on another is refused by name.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $handle {
            index: usize,
            builder: BuilderId,
        }

        impl $handle {
            fn new(index: usize, builder: BuilderId) -> Self {
                Self { index, builder }
            }

            fn index_in(self, builder: BuilderId) -> Result<usize> {
                if self.builder != builder {
                    return Err(Error::ADraftHandleCameFromAnotherBuilder { entity: $what });
                }
                Ok(self.index)
            }
        }
    };
}

define_handle!(DraftVertex, "vertex");
define_handle!(DraftEdge, "edge");
define_handle!(DraftCoedge, "coedge");
define_handle!(DraftWire, "wire");
define_handle!(DraftFace, "face");

#[cfg(test)]
mod tests {
    use rcad_geom::Scale;
    use rcad_predicates::{Policy, ProvenanceLog};

    use crate::Model;

    /// The one refusal in this module the sweep will not make for us, shown
    /// rather than claimed.
    ///
    /// [`SolidBuilder::add_shell`] refuses a second shell by name, and the
    /// reason it has to is here: `check_shells_and_solids` counts how many
    /// **solids** use each shell and refuses anything but one, and says nothing
    /// at all about how many shells a solid has. So a solid carrying two is a
    /// model this crate never builds, never meshes and never measures, and the
    /// sweep calls it sound.
    ///
    /// `Model` derives `Deserialize`, so those bytes are reachable. That makes
    /// this a **gap** and not a limit (`docs/decisions.md` §16.10): the data
    /// model holds `shells` as a list, a void inside a solid and a solid in two
    /// lumps are both real shapes, and what is missing is the code that means
    /// anything by them. §42.12 has no criterion for it, and this test is the
    /// record that it is open.
    #[test]
    fn the_sweep_does_not_refuse_a_solid_with_two_shells() {
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        model
            .add_cube(2.0, &policy, &mut log)
            .expect("a cube of side two");
        model
            .add_cube(3.0, &policy, &mut log)
            .expect("and a cube of side three, on top of it");

        let solids: Vec<_> = model.data().solids.keys().collect();
        let [first, second] = solids[..] else {
            panic!("two cubes are two solids");
        };
        let moved = model.data().solids[second].shells[0];
        model.data_mut().solids[first].shells.push(moved);
        model.data_mut().solids.remove(second);

        assert_eq!(model.solid_count(), 1);
        assert_eq!(model.shell_count(), 2);
        model
            .validate(&policy, &mut log)
            .expect("nothing in the sweep counts how many shells a solid has");
    }
}
