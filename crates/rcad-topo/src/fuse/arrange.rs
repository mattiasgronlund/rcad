//! One face against every line that cuts it, all at once.
//!
//! Never one line and then the next: the pieces would depend on the order the
//! lines arrived in, which depends on the order of the arguments, and §14.8's
//! commutativity and associativity oracles would fail by construction (§17.4).
//! The cut planes are sorted into a canonical order on the way in, so what comes
//! out is a function of the face and the *set* of planes cutting it.
//!
//! # Everything here is a pair of planes
//!
//! A face `F` sits on a plane `p`. Every line that matters to it is `p ∩ Q`:
//! `Q` is the plane of the face across a boundary edge, or the plane of a face
//! cutting it. So:
//!
//! - every corner of the arrangement is [`Plane::meet_of_three`], and no
//!   two-plane meet is needed anywhere (§17.5);
//! - whether two lines cross, run parallel or are the same line is the rank of a
//!   three-by-four matrix of stored coefficients, which is four exact
//!   determinants ([`relation_of_two_lines`]);
//! - the anticlockwise order of two lines round a corner is one more exact
//!   determinant ([`Arrangement::turns_anticlockwise`]);
//! - the order of points along a line is an interval comparison on the
//!   direction's dominant axis, and two points whose intervals overlap are
//!   undecidable, which is exactly the coincidence case that gets merged
//!   ([`Arrangement::order_along`]).
//!
//! # How the pieces are found
//!
//! The arcs of the arrangement are drawn on the plane; each one is walked from
//! both sides, and the walks close into cycles with the region they bound on the
//! left. How many times the face's boundary winds round each cycle's region is
//! then a count that propagates across the arcs: crossing a boundary arc from
//! its right to its left adds one, crossing a cut arc adds nothing. The cells
//! wound once are the pieces.
//!
//! A hole of the face that no cut line touches is a lump of the drawing that
//! nothing joins to the rest, so which piece it belongs to is the one question
//! the walk cannot answer, and [`Arrangement::piece_around`] answers it by
//! following one of the hole's own lines until it runs into another lump.

use core::cmp::Ordering;

use rcad_geom::{Orientation, Plane, PlaneAgreement, SIDE_OF_PLANE};
use rcad_predicates::{
    Decision, Interval, Measures, ORIENT2D, ORIENT3D, Point as PredicatePoint, Policy,
    ProvenanceLog, orient2d, orient3d,
};

use smallvec::SmallVec;

use crate::{FaceKey, Invalid, Result, VertexKey};

use super::bake::BakedFace;
use super::{
    FaceMeet, MeetPoint, Sameness, WHERE_THREE_PLANES_MEET, as_exact_as_it_is, gave_up,
    must_decide, the_other_way,
};

/// Cut one face into the pieces its cut lines leave behind.
///
/// A face with nothing cutting it comes back as one piece: itself.
///
/// # Errors
///
/// [`Error::Undecidable`](crate::Error::Undecidable) when a decision the split depends on could not be
/// settled under `policy`, and [`Error::Invalid`](crate::Error::Invalid) when the face turns out to be
/// broken — two of its corners in the same place, say.
///
/// # Panics
///
/// If the arrangement comes out contradicting itself: a cell that is both inside
/// and outside the face, a piece with fewer than three sides, or a hole that
/// does not sit in any piece. Those are broken internal invariants, and a
/// corrupt solid is worse than a crash (§4.1).
pub(crate) fn split_a_face(
    face: &BakedFace,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Vec<Piece>> {
    let arrangement = Arrangement::of(face, &canonical_cut_planes(face), policy, log)?;
    arrangement.pieces(log)
}

/// The stretches of the line `face.plane ∩ partner` that run through the inside
/// of `face`.
///
/// Used to tell a solid that crosses itself from one that merely has two faces
/// whose planes cross somewhere out in space (§17.4).
///
/// # Errors
///
/// The same as [`split_a_face`].
pub(crate) fn stretches_of_a_line_inside(
    face: &BakedFace,
    partner: &Plane,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Stretches> {
    let arrangement = Arrangement::of(face, core::slice::from_ref(partner), policy, log)?;
    Ok(arrangement.stretches_inside())
}

/// One piece a face was cut into.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Piece {
    /// The input faces this came out of. Every piece has at least one ancestor,
    /// and nothing here is allowed to invent geometry that has none; more than
    /// one when coplanar faces of the input were joined into a single face on
    /// the way in. They are all faces of one solid.
    pub(crate) ancestors: Vec<FaceKey>,
    /// The plane, inherited from the ancestor bit for bit (§17.1).
    pub(crate) plane: Plane,
    /// Which way round the ancestor's solid used it.
    pub(crate) orientation: Orientation,
    /// The boundary: the outer wire first, then one wire per hole. The outer one
    /// runs anticlockwise round [`Piece::plane`]'s normal and the holes run
    /// clockwise, which is the winding [`Model::validate`] asks for.
    ///
    /// [`Model::validate`]: crate::Model::validate
    pub(crate) wires: Vec<Vec<FaceMeet>>,
}

/// Where a line runs through the inside of a face, as ranges of one coordinate.
#[derive(Clone, Debug)]
pub(crate) struct Stretches {
    /// The coordinate axis the ranges are measured on: the one the line climbs
    /// fastest along.
    axis: usize,
    /// One range per stretch, each a pair of enclosures of its two ends.
    spans: Vec<[Interval; 2]>,
}

impl Stretches {
    /// Whether the two sets of stretches *provably* share more than a point.
    ///
    /// Proof only. Two stretches whose enclosures merely touch have not been
    /// shown to overlap, and reading that as an overlap would refuse solids that
    /// are perfectly sound — a pair of faces meeting at one corner, for
    /// instance.
    ///
    /// # Panics
    ///
    /// If the two were measured on different axes, which would make the numbers
    /// incomparable. Both come from the same pair of planes, so both pick the
    /// same axis, and an empty set of stretches has no axis to disagree about.
    pub(crate) fn provably_overlap(&self, other: &Stretches) -> bool {
        if self.spans.is_empty() || other.spans.is_empty() {
            return false;
        }
        assert!(
            self.axis == other.axis,
            "two stretches of the same line must be measured on the same axis"
        );
        self.spans.iter().any(|mine| {
            other.spans.iter().any(|theirs| {
                let (mine, theirs) = (certain_part(*mine), certain_part(*theirs));
                mine.0 < theirs.1 && theirs.0 < mine.1
            })
        })
    }
}

/// The stretch of the axis a span certainly covers, whatever its ends do inside
/// their boxes.
fn certain_part([from, to]: [Interval; 2]) -> (f64, f64) {
    let (low, high) = if from.hi() <= to.lo() {
        (from.hi(), to.lo())
    } else {
        (to.hi(), from.lo())
    };
    (low, high)
}

/// The cut planes of a face, in an order that depends on the set and nothing
/// else.
///
/// Sorted on the four coefficients so that the same set of planes always
/// arrives the same way round, and the arrangement is therefore a function of
/// the face and the set (§17.4). Planes that are equal as four numbers are
/// dropped; two *different* planes that happen to cut `p` in the same line are
/// found later by [`relation_of_two_lines`], which is exact.
fn canonical_cut_planes(face: &BakedFace) -> Vec<Plane> {
    let mut planes: Vec<Plane> = face.cuts.clone();
    planes.sort_by(|one, other| {
        let (one, other) = (one.coefficients(), other.coefficients());
        (0..4)
            .map(|index| one[index].total_cmp(&other[index]))
            .find(|order| order.is_ne())
            .unwrap_or(Ordering::Equal)
    });
    planes.dedup();
    planes
}

/// A face, the lines drawn on it, and everything that follows from them.
struct Arrangement<'f> {
    face: &'f BakedFace,
    plane: Plane,
    lines: Vec<Line>,
    nodes: Vec<Node>,
    arcs: Vec<Arc>,
    /// Which line each cut plane handed in turned into, in the order they were
    /// handed in. `None` when the plane is parallel to the face and cuts
    /// nothing.
    cut_lines: Vec<Option<usize>>,
    /// Which line each boundary edge lies on, by wire and position.
    line_of_edge: Vec<Vec<usize>>,
    /// Which node each corner is, by wire and position.
    node_of_corner: Vec<Vec<usize>>,
    /// The half-edges leaving each node, sorted anticlockwise.
    leaving: Vec<Vec<usize>>,
    /// Which cycle each half-edge is walked by.
    cycle_of: Vec<usize>,
    /// The cycles, each a chain of half-edges with its region on the left.
    cycles: Vec<Vec<usize>>,
    /// Which lump of the drawing each cycle belongs to.
    lump_of: Vec<usize>,
    /// How many times the face's boundary winds round the region each cycle
    /// bounds.
    winding: Vec<Option<i32>>,
}

/// One line on the face: the meet of the face's plane with one other.
struct Line {
    partner: Plane,
    /// The axis the line climbs fastest along, which is the one to order points
    /// on. The choice is made in `f64` because choosing an axis decides nothing;
    /// that the chosen component is really not zero is proved exactly.
    axis: usize,
    /// Whether `n_p × n_partner` points up that axis rather than down it.
    forward_climbs_the_axis: bool,
    /// Whether a face of the other group put this line here.
    cuts: bool,
}

/// A point of the arrangement.
struct Node {
    point: PredicatePoint,
    /// The input corner this is, when it is one.
    vertex: Option<VertexKey>,
    /// The three planes that placed it, the face's own first.
    planes: [Plane; 3],
    /// The lines running through it, in increasing order.
    lines: Vec<usize>,
}

/// A piece of one line between two neighbouring nodes on it.
struct Arc {
    line: usize,
    /// The two ends, the first one earlier along the line's forward direction.
    ends: [usize; 2],
    /// Which wire of the face walks along here, and which way.
    boundary: Option<Boundary>,
}

/// How the face's own boundary uses an arc.
#[derive(Clone, Copy, PartialEq)]
struct Boundary {
    /// Which wire: zero is the outer one.
    wire: usize,
    /// Whether the wire walks this arc in the line's forward direction.
    runs_forward: bool,
    /// How far round the wire the side this stretch is part of sits. The input
    /// edges are named there rather than copied here, because joining coplanar
    /// input faces can leave a side that is several of them.
    position: usize,
}

/// A way to leave a node: along a line, one way or the other.
#[derive(Clone, Copy, PartialEq)]
struct Direction {
    line: usize,
    /// `true` for `n_p × n_partner`, `false` for the other way.
    forward: bool,
}

impl<'f> Arrangement<'f> {
    fn of(
        face: &'f BakedFace,
        cuts: &[Plane],
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Self> {
        let mut arrangement = Self {
            face,
            plane: face.plane,
            lines: Vec::new(),
            nodes: Vec::new(),
            arcs: Vec::new(),
            cut_lines: Vec::new(),
            line_of_edge: Vec::new(),
            node_of_corner: Vec::new(),
            leaving: Vec::new(),
            cycle_of: Vec::new(),
            cycles: Vec::new(),
            lump_of: Vec::new(),
            winding: Vec::new(),
        };
        arrangement.draw_the_lines(cuts, policy, log)?;
        arrangement.place_the_corners(policy, log)?;
        arrangement.place_the_crossings(policy, log)?;
        arrangement.merge_points_that_are_one_point(policy, log)?;
        arrangement.cut_the_lines_into_arcs();
        arrangement.walk_the_cycles(policy, log)?;
        arrangement.count_the_windings();
        Ok(arrangement)
    }

    /// The pieces: the cells the face's boundary winds once round.
    ///
    /// A cell of the lump the outer wire is in is a piece. A lump nothing joins
    /// to that one is a hole of the face that no cut line touched, and it turns
    /// into a hole of whichever piece surrounds it.
    fn pieces(&self, log: &mut ProvenanceLog) -> Result<Vec<Piece>> {
        let outer_lump = self.lump_of[self.cycle_of[self.a_half_edge_of_the_outer_wire()]];

        let mut wires_of: Vec<(usize, Vec<usize>)> = (0..self.cycles.len())
            .filter(|&cycle| self.winding[cycle] == Some(1) && self.lump_of[cycle] == outer_lump)
            .map(|cycle| (cycle, Vec::new()))
            .collect();

        let mut lumps = self.lump_of.clone();
        lumps.sort_unstable();
        lumps.dedup();
        for lump in lumps {
            if lump == outer_lump {
                continue;
            }
            let hole = self.the_cycle_of_the_lump_that_is_wound_once(lump);
            let piece = self.piece_around(lump, outer_lump, log)?;
            let holder = wires_of
                .iter_mut()
                .find(|(cycle, _)| *cycle == piece)
                .expect("a hole of a face is surrounded by a piece of that face");
            holder.1.push(hole);
        }

        Ok(wires_of
            .into_iter()
            .map(|(outer, holes)| self.piece_from(outer, &holes))
            .collect())
    }

    /// One line per plane that matters to this face, with the ones that are
    /// really the same line folded together.
    fn draw_the_lines(
        &mut self,
        cuts: &[Plane],
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        for wire in &self.face.wires {
            let mut lines = Vec::new();
            for edge in wire {
                let line = self
                    .line_for(edge.across, false, policy, log)?
                    .expect("a face and the face across one of its edges are never parallel");
                lines.push(line);
            }
            self.line_of_edge.push(lines);
        }
        for &cut in cuts {
            let line = self.line_for(cut, true, policy, log)?;
            self.cut_lines.push(line);
        }
        Ok(())
    }

    /// The line `plane ∩ partner`, adding it if it is not there yet.
    ///
    /// `None` when `partner` is parallel to the face's plane, so there is no
    /// line at all.
    fn line_for(
        &mut self,
        partner: Plane,
        cuts: bool,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Option<usize>> {
        if planes_are_parallel(&self.plane, &partner, policy, log)? {
            return Ok(None);
        }
        for index in 0..self.lines.len() {
            if let LineRelation::Same = relation_of_two_lines(
                &self.plane,
                &self.lines[index].partner,
                &partner,
                policy,
                log,
            )? {
                self.lines[index].cuts |= cuts;
                return Ok(Some(index));
            }
        }
        let (axis, forward_climbs_the_axis) = dominant_axis(&self.plane, &partner, log)?;
        self.lines.push(Line {
            partner,
            axis,
            forward_climbs_the_axis,
            cuts,
        });
        Ok(Some(self.lines.len() - 1))
    }

    /// One node per corner of the face, re-derived rather than read (§17.1).
    ///
    /// Which lines run through a corner is mostly a question about the model:
    /// the two edges either side of it, and, where four or more faces meet
    /// there, every other plane the corner already knows about. Only a cut line
    /// has to be *asked*, and asking is [`Plane::side_of_point`].
    fn place_the_corners(&mut self, policy: &Policy, log: &mut ProvenanceLog) -> Result<()> {
        for (wire, edges) in self.face.wires.iter().enumerate() {
            let mut corners = Vec::new();
            for (position, edge) in edges.iter().enumerate() {
                let before = (position + edges.len() - 1) % edges.len();
                let mut lines = vec![
                    self.line_of_edge[wire][before],
                    self.line_of_edge[wire][position],
                ];
                for index in 0..self.lines.len() {
                    if lines.contains(&index) {
                        continue;
                    }
                    let known_from_the_model = edge
                        .start
                        .around
                        .iter()
                        .any(|around| *around == self.lines[index].partner);
                    if known_from_the_model
                        || self.passes_through(index, edge.start.point, policy, log)?
                    {
                        lines.push(index);
                    }
                }
                lines.sort_unstable();
                lines.dedup();
                corners.push(self.nodes.len());
                self.nodes.push(Node {
                    point: edge.start.point,
                    vertex: Some(edge.start.vertex),
                    planes: edge.start.planes,
                    lines,
                });
            }
            self.node_of_corner.push(corners);
        }
        Ok(())
    }

    fn passes_through(
        &self,
        line: usize,
        point: PredicatePoint,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<bool> {
        let decision = self.lines[line].partner.side_of_point(point);
        let settled = must_decide(
            decision,
            SIDE_OF_PLANE,
            &[point],
            Measures::Length,
            policy,
            log,
        )?;
        Ok(settled == Decision::Zero)
    }

    /// One node per pair of lines that cross, unless a corner already is that
    /// crossing.
    fn place_the_crossings(&mut self, policy: &Policy, log: &mut ProvenanceLog) -> Result<()> {
        for first in 0..self.lines.len() {
            for second in (first + 1)..self.lines.len() {
                if self
                    .nodes
                    .iter()
                    .any(|node| node.lines.contains(&first) && node.lines.contains(&second))
                {
                    continue;
                }
                let relation = relation_of_two_lines(
                    &self.plane,
                    &self.lines[first].partner,
                    &self.lines[second].partner,
                    policy,
                    log,
                )?;
                let LineRelation::CrossSomewhere = relation else {
                    continue;
                };
                let planes = in_a_fixed_order([
                    self.plane,
                    self.lines[first].partner,
                    self.lines[second].partner,
                ]);
                let Ok(point) = Plane::meet_of_three(&planes[0], &planes[1], &planes[2]) else {
                    return Err(gave_up(
                        WHERE_THREE_PLANES_MEET,
                        &planes.map(|plane| PredicatePoint::from(plane.normal().components())),
                        Measures::Volume,
                        log,
                    ));
                };
                self.nodes.push(Node {
                    point: as_exact_as_it_is(point),
                    vertex: None,
                    planes,
                    lines: vec![first, second],
                });
            }
        }
        Ok(())
    }

    /// Fold together the nodes that are one point.
    ///
    /// Two nodes are compared only where it matters, which is along a line they
    /// share — and every coincidence shows up there, because two nodes that are
    /// the same point are both crossings of lines through it, and every pair of
    /// those lines has a node of its own. Overlapping enclosures are exactly the
    /// coincidence case (§17.4): the policy decides whether they merge or the
    /// operation gives up.
    fn merge_points_that_are_one_point(
        &mut self,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<()> {
        loop {
            let mut same = Sameness::of(self.nodes.len());
            for line in 0..self.lines.len() {
                let order = self.nodes_along(line);
                for pair in order.windows(2) {
                    if self.order_along(line, pair[0], pair[1], policy, log)? == Decision::Zero {
                        same.join(pair[0], pair[1]);
                    }
                }
            }
            if !same.joined_anything() {
                return Ok(());
            }
            self.fold_the_nodes(&mut same)?;
        }
    }

    fn fold_the_nodes(&mut self, same: &mut Sameness) -> Result<()> {
        let mut folded: Vec<Option<usize>> = vec![None; self.nodes.len()];
        let mut nodes: Vec<Node> = Vec::new();
        for index in 0..self.nodes.len() {
            let leader = same.leader(index);
            match folded[leader] {
                None => {
                    folded[leader] = Some(nodes.len());
                    folded[index] = Some(nodes.len());
                    nodes.push(Node {
                        point: self.nodes[index].point,
                        vertex: self.nodes[index].vertex,
                        planes: self.nodes[index].planes,
                        lines: self.nodes[index].lines.clone(),
                    });
                }
                Some(into) => {
                    folded[index] = Some(into);
                    let mine = &self.nodes[index];
                    // A corner the input already had wins, whatever its box is:
                    // it is the point the faces around it agree on, and the
                    // neighbouring faces will emit the same one.
                    let takes_over = match (mine.vertex, nodes[into].vertex) {
                        (Some(mine), Some(theirs)) if mine != theirs => {
                            return Err(Invalid::EdgeHasNoLength.into());
                        }
                        (Some(_), Some(_)) => false,
                        (Some(_), None) => true,
                        (None, Some(_)) => false,
                        (None, None) => mine.point.uncertainty() < nodes[into].point.uncertainty(),
                    };
                    if takes_over {
                        nodes[into].vertex = mine.vertex;
                        nodes[into].point = mine.point;
                        nodes[into].planes = mine.planes;
                    }
                    let lines = mine.lines.clone();
                    nodes[into].lines.extend(lines);
                    nodes[into].lines.sort_unstable();
                    nodes[into].lines.dedup();
                }
            }
        }
        for corners in &mut self.node_of_corner {
            for corner in corners.iter_mut() {
                *corner = folded[*corner].expect("every node lands somewhere");
            }
        }
        self.nodes = nodes;
        Ok(())
    }

    /// The nodes on one line, ordered along its forward direction.
    ///
    /// Sorted on the low end of each enclosure, which settles nothing on its
    /// own; every neighbouring pair is then *proved* to be in that order or
    /// merged, and proved order between neighbours is order all the way along.
    fn nodes_along(&self, line: usize) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.nodes.len())
            .filter(|&node| self.nodes[node].lines.contains(&line))
            .collect();
        let axis = self.lines[line].axis;
        order.sort_by(|&one, &other| {
            let low = |node: usize| self.nodes[node].point.enclosure()[axis].lo();
            low(one).total_cmp(&low(other))
        });
        if !self.lines[line].forward_climbs_the_axis {
            order.reverse();
        }
        order
    }

    /// Which of two nodes on a line comes first along it: [`Decision::Pos`] when
    /// `first` does, [`Decision::Zero`] when they are the same point.
    fn order_along(
        &self,
        line: usize,
        first: usize,
        second: usize,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Decision> {
        let axis = self.lines[line].axis;
        let at = |node: usize| self.nodes[node].point.enclosure()[axis];
        let mut apart = at(second) - at(first);
        if !self.lines[line].forward_climbs_the_axis {
            apart = -apart;
        }
        must_decide(
            apart.sign(),
            ALONG_A_LINE,
            &[self.nodes[first].point, self.nodes[second].point],
            Measures::Length,
            policy,
            log,
        )
    }

    /// Cut every line into the arcs between neighbouring nodes on it, and throw
    /// away the ones that carry nothing.
    fn cut_the_lines_into_arcs(&mut self) {
        for line in 0..self.lines.len() {
            let order = self.nodes_along(line);
            let mut boundary: Vec<Option<Boundary>> = vec![None; order.len().saturating_sub(1)];
            let mut present = vec![self.lines[line].cuts; order.len().saturating_sub(1)];

            for (wire, edges) in self.face.wires.iter().enumerate() {
                for position in 0..edges.len() {
                    if self.line_of_edge[wire][position] != line {
                        continue;
                    }
                    let from = at_in(&order, self.node_of_corner[wire][position]);
                    let to = at_in(
                        &order,
                        self.node_of_corner[wire][(position + 1) % edges.len()],
                    );
                    let runs_forward = from < to;
                    for step in from.min(to)..from.max(to) {
                        assert!(
                            boundary[step].is_none(),
                            "two edges of one face run along the same stretch of one line"
                        );
                        boundary[step] = Some(Boundary {
                            wire,
                            runs_forward,
                            position,
                        });
                        present[step] = true;
                    }
                }
            }

            for step in 0..present.len() {
                if present[step] {
                    self.arcs.push(Arc {
                        line,
                        ends: [order[step], order[step + 1]],
                        boundary: boundary[step],
                    });
                }
            }
        }
        self.drop_the_lumps_that_are_only_cuts();
        self.join_arcs_that_meet_at_nothing();
    }

    /// Throw away lumps of the drawing made of cut arcs alone.
    ///
    /// A cut line that runs through the face crosses its boundary to get there,
    /// and a crossing is a node the boundary shares, so a lump with no boundary
    /// arc in it never touches the inside of the face. It bounds nothing that
    /// gets kept and it winds nothing round anything, so it can go — and once it
    /// is gone every remaining lump has a boundary arc to count windings from.
    fn drop_the_lumps_that_are_only_cuts(&mut self) {
        let mut same = Sameness::of(self.nodes.len());
        for arc in &self.arcs {
            same.join(arc.ends[0], arc.ends[1]);
        }
        let keep: Vec<usize> = self
            .arcs
            .iter()
            .filter(|arc| arc.boundary.is_some())
            .map(|arc| same.leader(arc.ends[0]))
            .collect();
        self.arcs
            .retain(|arc| keep.contains(&same.leader(arc.ends[0])));
    }

    /// Join two arcs that meet at a node nothing else touches.
    ///
    /// A cut line crosses the *line* of a far edge at a point that is not on the
    /// edge itself, and that crossing is a node with nothing else through it. It
    /// would come out as a corner of a piece where the piece has no corner, so
    /// the two arcs either side of it become one. A node the input already had
    /// is never dropped: the faces around it still have it, and dropping it here
    /// would leave an edge with a corner on one side and none on the other.
    fn join_arcs_that_meet_at_nothing(&mut self) {
        loop {
            let Some((node, first, second)) = self.a_node_that_is_no_corner() else {
                return;
            };
            let other_end = |arc: &Arc| {
                if arc.ends[0] == node {
                    arc.ends[1]
                } else {
                    arc.ends[0]
                }
            };
            let (before, after) = (other_end(&self.arcs[first]), other_end(&self.arcs[second]));
            let ends = if self.arcs[first].ends[1] == node {
                [before, after]
            } else {
                [after, before]
            };
            self.arcs[first].ends = ends;
            self.arcs.remove(second);
        }
    }

    fn a_node_that_is_no_corner(&self) -> Option<(usize, usize, usize)> {
        for node in 0..self.nodes.len() {
            if self.nodes[node].vertex.is_some() {
                continue;
            }
            let touching: Vec<usize> = (0..self.arcs.len())
                .filter(|&arc| self.arcs[arc].ends.contains(&node))
                .collect();
            if touching.len() != 2 {
                continue;
            }
            let (first, second) = (&self.arcs[touching[0]], &self.arcs[touching[1]]);
            if first.line == second.line && first.boundary == second.boundary {
                return Some((node, touching[0], touching[1]));
            }
        }
        None
    }

    /// Walk every arc from both sides, keeping the region on the left, and see
    /// what the walks close into.
    fn walk_the_cycles(&mut self, policy: &Policy, log: &mut ProvenanceLog) -> Result<()> {
        self.leaving = vec![Vec::new(); self.nodes.len()];
        for half_edge in 0..2 * self.arcs.len() {
            let node = self.from(half_edge);
            self.leaving[node].push(half_edge);
        }
        for node in 0..self.nodes.len() {
            let round_here = core::mem::take(&mut self.leaving[node]);
            self.leaving[node] = self.sorted_anticlockwise(&round_here, policy, log)?;
        }

        self.cycle_of = vec![usize::MAX; 2 * self.arcs.len()];
        for start in 0..2 * self.arcs.len() {
            if self.cycle_of[start] != usize::MAX {
                continue;
            }
            let mut chain = Vec::new();
            let mut half_edge = start;
            loop {
                self.cycle_of[half_edge] = self.cycles.len();
                chain.push(half_edge);
                half_edge = self.next_around_the_cell(half_edge);
                if half_edge == start {
                    break;
                }
                assert!(
                    chain.len() <= 2 * self.arcs.len(),
                    "a walk round a cell must come back to where it started"
                );
            }
            self.cycles.push(chain);
        }

        let mut same = Sameness::of(self.nodes.len());
        for arc in &self.arcs {
            same.join(arc.ends[0], arc.ends[1]);
        }
        self.lump_of = self
            .cycles
            .iter()
            .map(|chain| same.leader(self.from(chain[0])))
            .collect();
        Ok(())
    }

    /// The next half-edge round the cell this one has on its left.
    ///
    /// At the node this half-edge arrives at, the cell carries on along the
    /// half-edge one step *clockwise* from the way back down this arc. That is
    /// what keeps the region on the left all the way round, and it is what makes
    /// a bounded cell come out anticlockwise.
    fn next_around_the_cell(&self, half_edge: usize) -> usize {
        let node = self.to(half_edge);
        let back = half_edge ^ 1;
        let leaving = &self.leaving[node];
        let position = leaving
            .iter()
            .position(|&other| other == back)
            .expect("the way back down an arc leaves the node it arrives at");
        leaving[(position + leaving.len() - 1) % leaving.len()]
    }

    /// The half-edges leaving a node, in anticlockwise order.
    ///
    /// A sign from [`Arrangement::turns_anticlockwise`] compares two directions
    /// only when they are less than half a turn apart, so it is not an order on
    /// its own. Splitting the directions into the two halves of the turn that
    /// starts at the first line through the node makes it one: inside a half
    /// nothing is more than half a turn from anything else, so the sign is a
    /// genuine total order there, and the two halves are then laid end to end.
    /// Both steps are exact determinants of stored normals — no angle is ever
    /// worked out.
    fn sorted_anticlockwise(
        &self,
        half_edges: &[usize],
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Vec<usize>> {
        if half_edges.len() < 2 {
            return Ok(half_edges.to_vec());
        }
        let reference = Direction {
            line: half_edges
                .iter()
                .map(|&half_edge| self.direction(half_edge).line)
                .min()
                .expect("there is at least one way out of this node"),
            forward: true,
        };

        let mut sorted: Vec<usize> = Vec::with_capacity(half_edges.len());
        for &half_edge in half_edges {
            let mine = self.half_of_the_turn(reference, self.direction(half_edge), policy, log)?;
            let mut place = sorted.len();
            for (index, &placed) in sorted.iter().enumerate() {
                let theirs =
                    self.half_of_the_turn(reference, self.direction(placed), policy, log)?;
                let before = match mine.cmp(&theirs) {
                    Ordering::Less => true,
                    Ordering::Greater => false,
                    Ordering::Equal => {
                        let turn = self.turns_anticlockwise(
                            self.direction(half_edge),
                            self.direction(placed),
                            policy,
                            log,
                        )?;
                        assert!(
                            turn != Decision::Zero,
                            "two arcs leave one node the same way, so they lie on top of each other"
                        );
                        turn == Decision::Pos
                    }
                };
                if before {
                    place = index;
                    break;
                }
            }
            sorted.insert(place, half_edge);
        }
        Ok(sorted)
    }

    /// Which half of the turn starting at `reference` a direction is in: `0` for
    /// the half that starts at `reference` itself, `1` for the other.
    fn half_of_the_turn(
        &self,
        reference: Direction,
        direction: Direction,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<u8> {
        match self.turns_anticlockwise(reference, direction, policy, log)? {
            Decision::Pos => Ok(0),
            Decision::Neg => Ok(1),
            // Two different lines through one node are never parallel, so a
            // turn of nothing at all means this is the reference's own line,
            // pointing one way or the other.
            Decision::Zero => {
                assert!(
                    direction.line == reference.line,
                    "two different lines through a node cannot be parallel"
                );
                Ok(u8::from(direction.forward != reference.forward))
            }
            other => panic!("a turn came back with a decision nothing here knows: {other:?}"),
        }
    }

    /// Whether `second` is anticlockwise of `first` about the face's normal, by
    /// less than half a turn.
    ///
    /// An arc on the line `p ∩ Q` runs along `±(n_p × n_Q)`, so for two of them
    /// `(n_p × n_Q1) × (n_p × n_Q2) = det(n_p, n_Q1, n_Q2) · n_p`, using
    /// `(a×b)×(a×c) = a·((a×b)·c)` with `a = n_p`. Dotting that with `n_p` gives
    /// `det(n_p, n_Q1, n_Q2) · |n_p|²`, and `|n_p|²` is positive, so the sign of
    /// the turn is the sign of that determinant — with a change of sign for each
    /// direction that runs the other way down its line.
    ///
    /// `orient3d(a, b, c, d)` is the determinant of the rows `a-d`, `b-d`,
    /// `c-d`, so putting the origin last makes it exactly
    /// `det(n_p, n_Q1, n_Q2)` on the stored normals. No new predicate, no
    /// arithmetic of our own, and exact. The test at the bottom of this file
    /// pins the convention down against a determinant worked out by hand.
    fn turns_anticlockwise(
        &self,
        first: Direction,
        second: Direction,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<Decision> {
        let normal = self.plane.normal().components();
        let one = self.lines[first.line].partner.normal().components();
        let other = self.lines[second.line].partner.normal().components();
        let decision = must_decide(
            orient3d(normal, one, other, [0.0; 3]),
            ORIENT3D,
            &[
                PredicatePoint::from(normal),
                PredicatePoint::from(one),
                PredicatePoint::from(other),
            ],
            Measures::Volume,
            policy,
            log,
        )?;
        Ok(if first.forward == second.forward {
            decision
        } else {
            the_other_way(decision)
        })
    }

    /// How many times the face's boundary winds round the region each cycle
    /// bounds.
    ///
    /// One inside the face and nothing outside it. Crossing a boundary arc from
    /// the right of the way its wire runs to the left adds one, because the face
    /// is on the left of its own boundary; crossing a cut arc adds nothing.
    /// Every lump of the drawing has a boundary arc in it, and the material side
    /// of any boundary arc is inside the face, so every lump has somewhere to
    /// start counting from.
    ///
    /// # Panics
    ///
    /// If two ways round to the same cell disagree, or if a cell comes out wound
    /// more than once or less than none. A face that overlaps itself could do
    /// that, and it is not a face.
    fn count_the_windings(&mut self) {
        self.winding = vec![None; self.cycles.len()];
        for arc in 0..self.arcs.len() {
            let Some(boundary) = self.arcs[arc].boundary else {
                continue;
            };
            let material = self.cycle_of[2 * arc + usize::from(!boundary.runs_forward)];
            if self.winding[material].is_some() {
                continue;
            }
            self.winding[material] = Some(1);
            self.spread_the_winding(material);
        }
    }

    fn spread_the_winding(&mut self, from: usize) {
        let mut waiting = vec![from];
        while let Some(cycle) = waiting.pop() {
            let known = self.winding[cycle].expect("only settled cycles wait");
            for step in 0..self.cycles[cycle].len() {
                let half_edge = self.cycles[cycle][step];
                let arc = half_edge / 2;
                let across = self.cycle_of[half_edge ^ 1];
                let climb = match self.arcs[arc].boundary {
                    None => 0,
                    Some(boundary) => {
                        let material = 2 * arc + usize::from(!boundary.runs_forward);
                        if material == half_edge { -1 } else { 1 }
                    }
                };
                let theirs = known + climb;
                assert!(
                    (0..=1).contains(&theirs),
                    "the boundary of a face winds round a point once or not at all, not {theirs} \
                     times"
                );
                match self.winding[across] {
                    None => {
                        self.winding[across] = Some(theirs);
                        waiting.push(across);
                    }
                    Some(already) => assert!(
                        already == theirs,
                        "two ways round to one cell disagree about whether it is inside the face"
                    ),
                }
            }
        }
    }

    /// Which piece a lump of the drawing that nothing joins to the rest sits in.
    ///
    /// Follow one of the lump's own lines away from one of its nodes. Nothing
    /// changes which cell of the rest of the drawing we are in until we run into
    /// an arc of it, so the first arc we meet is a boundary of the cell we
    /// started in, and which side of that arc we came from says which cell that
    /// is.
    ///
    /// Every node of the lump is tried, not just the first. The whole lump is
    /// inside one cell of the rest of the drawing, so every node of it gives the
    /// same answer, and a corner where every ray runs into the end of an arc is
    /// no reason to give up while another corner has a ray that does not.
    ///
    /// Only the arcs of the lump the outer wire is in are looked at. Another
    /// hole that no cut line touches is inside the same cell we are, so running
    /// into its ring says nothing about which cell that is — and taking it as
    /// the answer would hand back a cycle that is no piece at all.
    ///
    /// # Errors
    ///
    /// [`Error::Undecidable`](crate::Error::Undecidable) when no line through the lump gives an answer.
    ///
    /// A ray that cannot be followed — one that runs along an arc, or that
    /// reaches an arc's end, or whose distances cannot be told apart — is
    /// abandoned and another is tried, without a word in the log. Abandoning a
    /// route is not a decision anybody failed to prove, and a record of a way we
    /// did not go would be noise in a log whose emptiness is supposed to mean
    /// something (§5).
    ///
    /// # Panics
    ///
    /// If the cell the ray lands in is not a piece. A hole of a face is inside
    /// that face, so the cell round it is always one.
    fn piece_around(
        &self,
        lump: usize,
        outer_lump: usize,
        log: &mut ProvenanceLog,
    ) -> Result<usize> {
        let mut corners: Vec<usize> = (0..self.arcs.len())
            .filter(|&arc| self.lump_of[self.cycle_of[2 * arc]] == lump)
            .flat_map(|arc| self.arcs[arc].ends)
            .collect();
        corners.sort_unstable();
        corners.dedup();
        assert!(!corners.is_empty(), "a lump has an arc in it");

        for &from in &corners {
            for line in self.nodes[from].lines.clone() {
                for forward in [true, false] {
                    if let Some(cycle) =
                        self.first_arc_met(from, Direction { line, forward }, outer_lump)
                    {
                        return Ok(cycle);
                    }
                }
            }
        }
        Err(gave_up(
            AROUND_A_HOLE,
            &[self.nodes[corners[0]].point],
            Measures::Length,
            log,
        ))
    }

    /// The cell of `outer_lump`'s drawing that a ray from `from` along
    /// `direction` starts in, or `None` when this ray cannot say.
    fn first_arc_met(&self, from: usize, direction: Direction, outer_lump: usize) -> Option<usize> {
        let axis = self.lines[direction.line].axis;
        let climbs = self.lines[direction.line].forward_climbs_the_axis == direction.forward;
        let onwards = |from_here: Interval| if climbs { from_here } else { -from_here };

        let mut nearest: Option<(Interval, usize)> = None;
        for arc in 0..self.arcs.len() {
            // Asked of every arc, the lump's own included: a ray that runs
            // along an arc cannot say which side of it we are on, and one of
            // the lump's own arcs is as much in the way as anybody else's. An
            // arc that only touches the ray at the corner it starts from, and
            // otherwise lies behind, is not in the way at all — and it is the
            // ordinary case, because the corner a ray starts from is a corner of
            // the ring the ray is trying to place.
            if self.arcs[arc].line == direction.line {
                let not_behind_us = |end: usize| {
                    end != from
                        && onwards(
                            self.nodes[end].point.enclosure()[axis]
                                - self.nodes[from].point.enclosure()[axis],
                        )
                        .sign()
                            != Decision::Neg
                };
                if self.arcs[arc].ends.iter().copied().any(not_behind_us) {
                    return None;
                }
                continue;
            }
            if self.lump_of[self.cycle_of[2 * arc]] != outer_lump {
                continue;
            }
            let at = match self.crossing_of(direction.line, arc) {
                Crossing::Misses => continue,
                // Touching an arc's end says nothing about which side of it we
                // are on — but only where the touch is ahead of us. An arc the
                // ray's line grazes behind our back is out of the story.
                Crossing::AtAnEnd(at) => {
                    match onwards(at - self.nodes[from].point.enclosure()[axis]).sign() {
                        Decision::Neg => continue,
                        _ => return None,
                    }
                }
                Crossing::At(at) => at,
            };
            match onwards(at - self.nodes[from].point.enclosure()[axis]).sign() {
                Decision::Pos => (),
                Decision::Neg => continue,
                _ => return None,
            }
            nearest = match nearest {
                None => Some((at, arc)),
                Some((best, best_arc)) => match onwards(at - best).sign() {
                    Decision::Neg => Some((at, arc)),
                    Decision::Pos => Some((best, best_arc)),
                    _ => return None,
                },
            };
        }
        let (_, arc) = nearest?;
        // An arc's forward half-edge has the negative side of its partner plane
        // on its left: on the plane `z = 0` the line `x = 0` runs along
        // `z × x = +y`, and walking up `y` with `z` overhead leaves `x < 0` on
        // the left.
        let half_edge = match self.lines[self.arcs[arc].line]
            .partner
            .side_of_point(self.nodes[from].point)
        {
            Decision::Neg => 2 * arc,
            Decision::Pos => 2 * arc + 1,
            _ => return None,
        };
        Some(self.cycle_of[half_edge])
    }

    /// Where a line crosses an arc, as a coordinate on the line's own axis.
    fn crossing_of(&self, line: usize, arc: usize) -> Crossing {
        let other = self.arcs[arc].line;
        let normals = |of: usize| self.lines[of].partner.normal().components();
        if orient3d(
            self.plane.normal().components(),
            normals(line),
            normals(other),
            [0.0; 3],
        ) == Decision::Zero
        {
            return Crossing::Misses;
        }
        let Ok(at) = Plane::meet_of_three(
            &self.plane,
            &self.lines[line].partner,
            &self.lines[other].partner,
        ) else {
            return Crossing::Misses;
        };
        let at = as_exact_as_it_is(at);
        let along = self.lines[other].axis;
        let past =
            |end: usize| (at.enclosure()[along] - self.nodes[end].point.enclosure()[along]).sign();
        match (past(self.arcs[arc].ends[0]), past(self.arcs[arc].ends[1])) {
            (Decision::Pos, Decision::Neg) | (Decision::Neg, Decision::Pos) => {
                Crossing::At(at.enclosure()[self.lines[line].axis])
            }
            (Decision::Pos, Decision::Pos) | (Decision::Neg, Decision::Neg) => Crossing::Misses,
            _ => Crossing::AtAnEnd(at.enclosure()[self.lines[line].axis]),
        }
    }

    /// Where the one cut line this arrangement was built with runs through the
    /// inside of the face.
    ///
    /// A cut line that turned out to be a line the face's own boundary already
    /// runs along has no cut arcs at all, and the answer is that it goes nowhere
    /// near the inside — which is right: the boundary is not the inside.
    fn stretches_inside(&self) -> Stretches {
        let Some(line) = self.cut_lines[0] else {
            return Stretches {
                axis: 0,
                spans: Vec::new(),
            };
        };
        let axis = self.lines[line].axis;
        let spans = (0..self.arcs.len())
            .filter(|&arc| {
                self.arcs[arc].line == line
                    && self.arcs[arc].boundary.is_none()
                    && self.winding[self.cycle_of[2 * arc]] == Some(1)
            })
            .map(|arc| {
                self.arcs[arc]
                    .ends
                    .map(|end| self.nodes[end].point.enclosure()[axis])
            })
            .collect();
        Stretches { axis, spans }
    }

    fn the_cycle_of_the_lump_that_is_wound_once(&self, lump: usize) -> usize {
        let mut wound = (0..self.cycles.len())
            .filter(|&cycle| self.lump_of[cycle] == lump && self.winding[cycle] == Some(1));
        let only = wound.next().expect("a hole has a material side");
        assert!(
            wound.next().is_none(),
            "a hole of a face that no cut line touches has one material side"
        );
        only
    }

    fn a_half_edge_of_the_outer_wire(&self) -> usize {
        (0..2 * self.arcs.len())
            .find(|&half_edge| {
                self.arcs[half_edge / 2]
                    .boundary
                    .is_some_and(|boundary| boundary.wire == 0)
            })
            .expect("a face has an outer wire")
    }

    fn piece_from(&self, outer: usize, holes: &[usize]) -> Piece {
        let wires = core::iter::once(outer)
            .chain(holes.iter().copied())
            .map(|cycle| self.wire_from(cycle))
            .collect();
        Piece {
            ancestors: self.face.keys.clone(),
            plane: self.plane,
            orientation: self.face.orientation,
            wires,
        }
    }

    /// # Panics
    ///
    /// If the wire has fewer than three sides. Two straight lines enclose no
    /// area, so a piece like that is a zero-area face, which §17.4 prevents
    /// rather than filters: reaching here means the splitter is wrong.
    fn wire_from(&self, cycle: usize) -> Vec<FaceMeet> {
        assert!(
            self.cycles[cycle].len() >= 3,
            "a piece of a face has at least three sides, this one has {}",
            self.cycles[cycle].len()
        );
        self.cycles[cycle]
            .iter()
            .map(|&half_edge| FaceMeet {
                carried_by: [
                    self.plane,
                    self.lines[self.arcs[half_edge / 2].line].partner,
                ],
                ends: [
                    self.meet_point(self.from(half_edge)),
                    self.meet_point(self.to(half_edge)),
                ],
                along: self.arcs[half_edge / 2]
                    .boundary
                    .map_or_else(SmallVec::new, |boundary| {
                        self.face.wires[boundary.wire][boundary.position]
                            .keys
                            .clone()
                    }),
                approximation: None,
            })
            .collect()
    }

    fn meet_point(&self, node: usize) -> MeetPoint {
        match self.nodes[node].vertex {
            Some(vertex) => MeetPoint::Existing {
                vertex,
                point: self.nodes[node].point,
            },
            None => MeetPoint::OfThreePlanes {
                planes: self.nodes[node].planes,
                point: self.nodes[node].point,
            },
        }
    }

    fn direction(&self, half_edge: usize) -> Direction {
        Direction {
            line: self.arcs[half_edge / 2].line,
            forward: half_edge.is_multiple_of(2),
        }
    }

    fn from(&self, half_edge: usize) -> usize {
        self.arcs[half_edge / 2].ends[half_edge % 2]
    }

    fn to(&self, half_edge: usize) -> usize {
        self.arcs[half_edge / 2].ends[1 - half_edge % 2]
    }
}

/// Three planes sorted on their coefficients, so that whichever face asks for
/// their meeting point asks for it the same way round.
///
/// Cramer's rule rounds differently when its rows are permuted, so a corner
/// solved as `meet(p, q, r)` by one face and as `meet(q, p, r)` by the face next
/// to it comes out a bit apart and the two faces stop agreeing about where their
/// shared corner is. Which face gets there first depends on which side of the
/// fuse its solid was on, so this is §14.8's commutativity oracle again. The
/// order is the one [`Corners::of`](super::assemble) already names the corner
/// by, so the name and the numbers are settled the same way.
fn in_a_fixed_order(mut planes: [Plane; 3]) -> [Plane; 3] {
    planes.sort_unstable_by_key(|plane| plane.coefficients().map(f64::to_bits));
    planes
}

/// The name the ordering of two points along a line goes by in provenance
/// records.
const ALONG_A_LINE: &str = "order along a line";

/// The name the search for the piece a hole sits in goes by in provenance
/// records.
const AROUND_A_HOLE: &str = "which piece of a face a hole is in";

/// Where a line meets an arc.
enum Crossing {
    /// Nowhere along it.
    Misses,
    /// At this coordinate on the line's own axis.
    At(Interval),
    /// At one of the arc's ends, or too close to one to say. Carries where, on
    /// the line's own axis.
    AtAnEnd(Interval),
}

fn at_in(order: &[usize], node: usize) -> usize {
    order
        .iter()
        .position(|&other| other == node)
        .expect("a corner is one of the nodes on the line its edge runs along")
}

/// Which axis the line `p ∩ q` climbs fastest along, and whether
/// `n_p × n_q` climbs it rather than descends it.
///
/// The line's direction is `n_p × n_q`, whose components are two-by-two
/// determinants of stored coefficients, so [`orient2d`] measured from the origin
/// gives each of their signs exactly. Picking the largest of them is done in
/// `f64` and decides nothing: any axis whose component is *proved* non-zero
/// orders the line's points correctly, and the largest is only the one that
/// separates them furthest.
///
/// # Errors
///
/// [`Error::Undecidable`](crate::Error::Undecidable) when no component can be shown to be non-zero, which
/// for two planes that are not parallel takes coefficients too extreme to
/// multiply.
fn dominant_axis(plane: &Plane, partner: &Plane, log: &mut ProvenanceLog) -> Result<(usize, bool)> {
    let (mine, theirs) = (plane.normal().components(), partner.normal().components());
    let mut best: Option<(usize, bool, f64)> = None;
    for axis in 0..3 {
        let (one, other) = ((axis + 1) % 3, (axis + 2) % 3);
        let size = (mine[one] * theirs[other] - mine[other] * theirs[one]).abs();
        let sign = orient2d(
            [0.0, 0.0],
            [mine[one], mine[other]],
            [theirs[one], theirs[other]],
        );
        let climbs = match sign {
            Decision::Pos => true,
            Decision::Neg => false,
            _ => continue,
        };
        if best.is_none_or(|(_, _, so_far)| so_far < size) {
            best = Some((axis, climbs, size));
        }
    }
    match best {
        Some((axis, climbs, _)) => Ok((axis, climbs)),
        None => Err(gave_up(
            ORIENT2D,
            &[PredicatePoint::from(mine), PredicatePoint::from(theirs)],
            Measures::Area,
            log,
        )),
    }
}

/// How the lines `p ∩ first` and `p ∩ second` lie.
enum LineRelation {
    /// They cross at one point.
    CrossSomewhere,
    /// They run alongside each other and never meet.
    Parallel,
    /// They are the same line.
    Same,
}

/// Which of the three it is, decided exactly.
///
/// The three planes share a line exactly when the three-by-four matrix of their
/// coefficients has rank two, and that is when all four of its three-by-three
/// minors vanish. Each minor is the determinant of three `f64` triples, which is
/// [`orient3d`] measured from the origin — so this is four exact determinants of
/// stored numbers, with no constructed point and nothing to be undecided about
/// short of an overflow.
///
/// The first minor is the one built from the normals. Non-zero, and the three
/// planes meet at a point, so the two lines cross. Zero, and the lines are
/// parallel or the same, and the other three minors say which.
fn relation_of_two_lines(
    plane: &Plane,
    first: &Plane,
    second: &Plane,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<LineRelation> {
    let rows = [
        plane.coefficients(),
        first.coefficients(),
        second.coefficients(),
    ];
    let minor = |columns: [usize; 3], log: &mut ProvenanceLog| -> Result<Decision> {
        let pick = |row: [f64; 4]| [row[columns[0]], row[columns[1]], row[columns[2]]];
        let taken = rows.map(pick);
        must_decide(
            orient3d(taken[0], taken[1], taken[2], [0.0; 3]),
            ORIENT3D,
            &taken.map(PredicatePoint::from),
            Measures::Volume,
            policy,
            log,
        )
    };

    if minor([0, 1, 2], log)? != Decision::Zero {
        return Ok(LineRelation::CrossSomewhere);
    }
    for columns in [[0, 1, 3], [0, 2, 3], [1, 2, 3]] {
        if minor(columns, log)? != Decision::Zero {
            return Ok(LineRelation::Parallel);
        }
    }
    Ok(LineRelation::Same)
}

/// Whether two planes never meet, or are the same plane.
///
/// The normals are parallel exactly when all three two-by-two minors of them
/// vanish, which [`orient2d`] measured from the origin decides exactly.
pub(crate) fn planes_are_parallel(
    plane: &Plane,
    other: &Plane,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    let (mine, theirs) = (plane.normal().components(), other.normal().components());
    for axis in 0..3 {
        let (one, other) = ((axis + 1) % 3, (axis + 2) % 3);
        let decision = must_decide(
            orient2d(
                [0.0, 0.0],
                [mine[one], mine[other]],
                [theirs[one], theirs[other]],
            ),
            ORIENT2D,
            &[PredicatePoint::from(mine), PredicatePoint::from(theirs)],
            Measures::Area,
            policy,
            log,
        )?;
        if decision != Decision::Zero {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Whether two planes are the same plane, refusing when nothing can tell.
///
/// [`PlaneAgreement::CannotTell`] takes coefficients whose products fall out of
/// the range `f64` multiplication can be undone in — §3.5's dynamic-range check
/// is what keeps models like that out. Nothing can bound how wrong the answer
/// would be, so the record says so rather than naming a width it does not have.
pub(crate) fn agreement_of_two_planes(
    plane: &Plane,
    other: &Plane,
    log: &mut ProvenanceLog,
) -> Result<PlaneAgreement> {
    match plane.agreement_with(other) {
        settled @ (PlaneAgreement::SameWay
        | PlaneAgreement::OppositeWay
        | PlaneAgreement::Different) => Ok(settled),
        PlaneAgreement::CannotTell => Err(gave_up(
            ORIENT2D,
            &[
                PredicatePoint::from(plane.normal().components()),
                PredicatePoint::from(other.normal().components()),
            ],
            Measures::Area,
            log,
        )),
        other => panic!("plane agreement came back with an answer nothing here knows: {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use rcad_geom::{Isometry, Plane, Scale, Vector};
    use rcad_predicates::{Decision, Policy, ProvenanceLog, orient3d};

    use super::{Piece, split_a_face, the_other_way};
    use crate::fuse::bake::{BakedFace, bake};
    use crate::fuse::{FaceMeet, MeetPoint};
    use crate::{Model, ProfileBuilder, SolidKey};

    /// The identity the anticlockwise order of two lines round a corner rests
    /// on, checked against a determinant worked out by hand.
    ///
    /// `orient3d(a, b, c, d)` is the determinant of the rows `a-d`, `b-d`,
    /// `c-d`, so with the origin last it is the determinant of the three rows
    /// themselves. Small integers keep every product exact, so the plain `f64`
    /// determinant is a fair oracle. The brief this was built from had the
    /// origin *first*, which is the same determinant with its sign flipped —
    /// that is what a test is for.
    #[test]
    fn orient3d_with_the_origin_last_is_the_determinant_of_the_three_rows() {
        for first in -2..=2 {
            for second in -2..=2 {
                for third in -2..=2 {
                    let rows = [
                        [1.0, f64::from(first), 0.5],
                        [f64::from(second), 2.0, -1.0],
                        [0.25, f64::from(third), 3.0],
                    ];
                    let by_hand = rows[0][0] * (rows[1][1] * rows[2][2] - rows[1][2] * rows[2][1])
                        - rows[0][1] * (rows[1][0] * rows[2][2] - rows[1][2] * rows[2][0])
                        + rows[0][2] * (rows[1][0] * rows[2][1] - rows[1][1] * rows[2][0]);
                    let expected = if by_hand > 0.0 {
                        Decision::Pos
                    } else if by_hand < 0.0 {
                        Decision::Neg
                    } else {
                        Decision::Zero
                    };
                    assert_eq!(orient3d(rows[0], rows[1], rows[2], [0.0; 3]), expected);
                    assert_eq!(
                        orient3d([0.0; 3], rows[0], rows[1], rows[2]),
                        the_other_way(expected),
                        "with the origin first the sign is the other way round"
                    );
                }
            }
        }
    }

    fn a_cube_of_side_two() -> (Model, SolidKey) {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let cube = model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube of side two");
        (model, cube)
    }

    /// The top of the cube: the square from `(0, 0, 2)` to `(2, 2, 2)`, facing
    /// up.
    fn the_top_of_a_cube(model: &Model, cube: SolidKey) -> BakedFace {
        let baked = bake(model, &[cube], &mut ProvenanceLog::new()).expect("a cube bakes");
        baked
            .into_iter()
            .next()
            .expect("one solid was baked")
            .faces
            .into_iter()
            .find(|face| face.plane.normal() == Vector::new(0.0, 0.0, 1.0))
            .expect("a cube has a face pointing up")
    }

    /// The upward-facing top of a profile extruded by one.
    fn the_top_of_an_extruded(outline: &[[f64; 2]]) -> BakedFace {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let profile = ProfileBuilder::new(outline.to_vec())
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a profile");
        let solid = model
            .add_extrusion(&profile, 1.0, &Isometry::identity())
            .expect("an extrusion");
        bake(&model, &[solid], &mut ProvenanceLog::new())
            .expect("it bakes")
            .swap_remove(0)
            .faces
            .into_iter()
            .find(|face| face.plane.normal() == Vector::new(0.0, 0.0, 1.0))
            .expect("an extrusion has a top")
    }

    fn cut_by(face: &BakedFace, planes: &[[f64; 4]]) -> BakedFace {
        let mut cut = face.clone();
        cut.cuts = planes
            .iter()
            .map(|&coefficients| Plane::from_coefficients(coefficients).expect("a plane"))
            .collect();
        cut
    }

    fn split(face: &BakedFace) -> Vec<Piece> {
        let mut log = ProvenanceLog::new();
        let pieces = split_a_face(face, &Policy::certified(), &mut log)
            .expect("a lattice-aligned face splits with nothing left undecided");
        assert!(
            log.is_empty(),
            "every decision here is exactly representable, so none needed a policy"
        );
        pieces
    }

    /// Twice the area a wire sweeps round `normal`, summed as `Σ (p × q) · n`.
    ///
    /// Exact, and that is the whole point of measuring it this way here: every
    /// corner of a lattice-aligned face is a small `f64` with a handful of
    /// significant bits, so each product and each sum is exact in `f64` and the
    /// areas of the pieces add up to the parent's area with no tolerance in
    /// sight. The signs come out right for holes as well, because a hole is
    /// wound the other way and its area is subtracted.
    fn twice_the_area(wire: &[FaceMeet], normal: [f64; 3]) -> f64 {
        wire.iter()
            .map(|side| {
                let at = |end: &MeetPoint| {
                    end.point()
                        .exact_coordinates()
                        .expect("a lattice-aligned corner is an f64")
                };
                let (p, q) = (at(&side.ends[0]), at(&side.ends[1]));
                let cross = [
                    p[1] * q[2] - p[2] * q[1],
                    p[2] * q[0] - p[0] * q[2],
                    p[0] * q[1] - p[1] * q[0],
                ];
                cross[0] * normal[0] + cross[1] * normal[1] + cross[2] * normal[2]
            })
            .sum()
    }

    fn twice_the_area_of(piece: &Piece) -> f64 {
        piece
            .wires
            .iter()
            .map(|wire| twice_the_area(wire, piece.plane.normal().components()))
            .sum()
    }

    fn every_wire_closes(piece: &Piece) {
        for wire in &piece.wires {
            for step in 0..wire.len() {
                assert_eq!(
                    wire[step].ends[1].point(),
                    wire[(step + 1) % wire.len()].ends[0].point(),
                    "a side of a piece has to start where the one before it ended"
                );
            }
        }
    }

    #[test]
    fn a_face_with_nothing_cutting_it_comes_back_as_itself() {
        let (model, cube) = a_cube_of_side_two();
        let top = the_top_of_a_cube(&model, cube);

        let pieces = split(&top);

        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].ancestors, top.keys);
        assert_eq!(pieces[0].plane, top.plane);
        assert_eq!(pieces[0].wires.len(), 1);
        assert_eq!(pieces[0].wires[0].len(), 4);
        assert_eq!(twice_the_area_of(&pieces[0]), 8.0, "a square of side two");
        every_wire_closes(&pieces[0]);
        for side in &pieces[0].wires[0] {
            assert!(
                matches!(side.ends[0], MeetPoint::Existing { .. }),
                "an uncut face keeps the corners the input had"
            );
            assert_eq!(side.approximation, None, "planes meet in straight lines");
        }
    }

    #[test]
    fn a_face_cut_by_one_line_through_its_middle_comes_back_as_two_pieces() {
        let (model, cube) = a_cube_of_side_two();
        let top = cut_by(&the_top_of_a_cube(&model, cube), &[[1.0, 0.0, 0.0, -1.0]]);

        let pieces = split(&top);

        assert_eq!(pieces.len(), 2);
        for piece in &pieces {
            assert_eq!(piece.wires.len(), 1);
            assert_eq!(piece.wires[0].len(), 4);
            assert_eq!(twice_the_area_of(piece), 4.0, "half the square each");
            every_wire_closes(piece);
        }
    }

    #[test]
    fn a_face_cut_by_two_crossing_lines_comes_back_as_four_pieces() {
        let (model, cube) = a_cube_of_side_two();
        let top = cut_by(
            &the_top_of_a_cube(&model, cube),
            &[[1.0, 0.0, 0.0, -1.0], [0.0, 1.0, 0.0, -1.0]],
        );

        let pieces = split(&top);

        assert_eq!(pieces.len(), 4);
        for piece in &pieces {
            assert_eq!(piece.wires[0].len(), 4);
            assert_eq!(
                twice_the_area_of(piece),
                2.0,
                "a quarter of the square each"
            );
            every_wire_closes(piece);
        }
    }

    /// The splitter's oracle, and it needs no tolerance: the pieces of a face
    /// are a partition of it, so their areas add up to its area exactly.
    #[test]
    fn the_pieces_of_a_face_add_up_to_the_area_it_started_with() {
        let (model, cube) = a_cube_of_side_two();
        let uncut = the_top_of_a_cube(&model, cube);
        let whole = twice_the_area_of(&split(&uncut)[0]);

        for cuts in [
            vec![[1.0, 0.0, 0.0, -1.0]],
            vec![[1.0, 0.0, 0.0, -0.5], [0.0, 1.0, 0.0, -1.5]],
            vec![
                [1.0, 0.0, 0.0, -0.5],
                [0.0, 1.0, 0.0, -1.5],
                [1.0, 1.0, 0.0, -2.0],
            ],
        ] {
            let pieces = split(&cut_by(&uncut, &cuts));
            let summed: f64 = pieces.iter().map(twice_the_area_of).sum();
            assert_eq!(summed, whole, "{} pieces from {cuts:?}", pieces.len());
        }
    }

    /// The other half of the oracle. Every side of every piece is walked once,
    /// and a side that two pieces share is walked once each way — which is what
    /// makes the pieces fit together with nothing over and nothing missing.
    #[test]
    fn every_side_of_a_piece_is_walked_once_and_a_shared_one_once_each_way() {
        let (model, cube) = a_cube_of_side_two();
        let top = cut_by(
            &the_top_of_a_cube(&model, cube),
            &[[1.0, 0.0, 0.0, -1.0], [0.0, 1.0, 0.0, -1.0]],
        );
        let rim: Vec<Plane> = top.wires.iter().flatten().map(|edge| edge.across).collect();

        let mut walked: Vec<([f64; 3], [f64; 3], Plane)> = Vec::new();
        for piece in &split(&top) {
            every_wire_closes(piece);
            for wire in &piece.wires {
                for side in wire {
                    let at = |end: &MeetPoint| {
                        end.point()
                            .exact_coordinates()
                            .expect("a lattice-aligned corner is an f64")
                    };
                    walked.push((at(&side.ends[0]), at(&side.ends[1]), side.carried_by[1]));
                }
            }
        }

        for (position, side) in walked.iter().enumerate() {
            assert!(
                !walked[position + 1..].contains(side),
                "one side of one piece is walked twice the same way"
            );
            let back = (side.1, side.0, side.2);
            assert!(
                walked.contains(&back) || rim.contains(&side.2),
                "a side nobody walks back along has to be on the rim of the face"
            );
        }
    }

    #[test]
    fn every_piece_names_the_one_face_it_came_from() {
        let (model, cube) = a_cube_of_side_two();
        let top = cut_by(
            &the_top_of_a_cube(&model, cube),
            &[[1.0, 0.0, 0.0, -1.0], [0.0, 1.0, 0.0, -1.0]],
        );

        for piece in &split(&top) {
            assert_eq!(piece.ancestors, top.keys);
            assert_eq!(piece.orientation, top.orientation);
        }
    }

    /// The property §14.8's commutativity oracle needs one level down: the
    /// pieces depend on the *set* of cut lines and not on the order they were
    /// found in, so `fuse(A, B)` and `fuse(B, A)` cut the same face the same
    /// way.
    #[test]
    fn presenting_the_same_cuts_the_other_way_round_gives_the_very_same_pieces() {
        let (model, cube) = a_cube_of_side_two();
        let uncut = the_top_of_a_cube(&model, cube);
        let cuts = [
            [1.0, 0.0, 0.0, -0.5],
            [0.0, 1.0, 0.0, -1.5],
            [1.0, 1.0, 0.0, -2.0],
        ];
        let mut backwards = cuts;
        backwards.reverse();

        assert_eq!(
            split(&cut_by(&uncut, &cuts)),
            split(&cut_by(&uncut, &backwards))
        );
    }

    /// A face with a hole in it, cut by a line that misses the hole. The hole is
    /// a lump of the drawing nothing joins to the rest, so which of the two
    /// pieces it belongs to is the one thing the walk cannot work out on its
    /// own.
    #[test]
    fn a_hole_the_cut_misses_ends_up_in_the_piece_that_surrounds_it() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let plate = ProfileBuilder::new(vec![[0.0, 0.0], [8.0, 0.0], [8.0, 4.0], [0.0, 4.0]])
            .with_hole(vec![[5.0, 1.0], [7.0, 1.0], [7.0, 3.0], [5.0, 3.0]])
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a plate with a square hole");
        let solid = model
            .add_extrusion(&plate, 1.0, &Isometry::identity())
            .expect("an extrusion");
        let top = bake(&model, &[solid], &mut ProvenanceLog::new())
            .expect("the plate bakes")
            .swap_remove(0)
            .faces
            .into_iter()
            .find(|face| face.plane.normal() == Vector::new(0.0, 0.0, 1.0))
            .expect("the plate has a top");
        let cut = cut_by(&top, &[[1.0, 0.0, 0.0, -2.0]]);

        let pieces = split(&cut);

        assert_eq!(pieces.len(), 2);
        let holed = pieces
            .iter()
            .find(|piece| piece.wires.len() == 2)
            .expect("the piece the hole is in keeps the hole");
        assert_eq!(holed.wires[1].len(), 4, "the hole is still a square");
        assert_eq!(
            twice_the_area_of(holed),
            2.0 * (6.0 * 4.0 - 2.0 * 2.0),
            "the piece with the hole is the hole's area short"
        );
        let plain = pieces
            .iter()
            .find(|piece| piece.wires.len() == 1)
            .expect("the other piece has no hole");
        assert_eq!(twice_the_area_of(plain), 2.0 * 2.0 * 4.0);
        for piece in &pieces {
            every_wire_closes(piece);
        }
    }

    /// A cut that runs corner to corner. Both of its ends land exactly on a
    /// corner the face already had, so the corner has to absorb the cut line
    /// rather than a new point being made a hair away from it — which is the
    /// vertex-on-face case of roadmap §7.4's catalogue, built to land on rather
    /// than approached.
    #[test]
    fn a_cut_from_one_corner_of_a_face_to_another_gives_two_pieces_and_no_new_corners() {
        let (model, cube) = a_cube_of_side_two();
        let top = cut_by(&the_top_of_a_cube(&model, cube), &[[1.0, 1.0, 0.0, -2.0]]);

        let pieces = split(&top);

        assert_eq!(pieces.len(), 2);
        for piece in &pieces {
            assert_eq!(piece.wires[0].len(), 3, "each piece is a triangle");
            assert_eq!(twice_the_area_of(piece), 4.0);
            every_wire_closes(piece);
            for side in &piece.wires[0] {
                assert!(
                    matches!(side.ends[0], MeetPoint::Existing { .. }),
                    "a cut along a diagonal makes no corner the face did not have"
                );
            }
        }
    }

    /// A face that is not convex, cut by a line that also crosses the *line* of
    /// one of its own far edges at a point in the middle of the face.
    ///
    /// That crossing is a node with nothing else through it, and leaving it in
    /// would put a corner on a piece where the piece is perfectly straight.
    #[test]
    fn a_face_that_bends_back_on_itself_is_cut_into_two_with_no_corners_invented() {
        let top = the_top_of_an_extruded(&[
            [0.0, 0.0],
            [4.0, 0.0],
            [4.0, 2.0],
            [2.0, 2.0],
            [2.0, 4.0],
            [0.0, 4.0],
        ]);

        let across = split(&cut_by(&top, &[[0.0, 1.0, 0.0, -1.0]]));
        assert_eq!(across.len(), 2);
        let mut areas: Vec<f64> = across.iter().map(twice_the_area_of).collect();
        areas.sort_by(f64::total_cmp);
        assert_eq!(areas, [2.0 * 4.0, 2.0 * 8.0]);
        for piece in &across {
            every_wire_closes(piece);
            assert_eq!(piece.wires.len(), 1);
        }
        assert_eq!(
            across
                .iter()
                .find(|piece| twice_the_area_of(piece) == 8.0)
                .expect("the strip below the cut")
                .wires[0]
                .len(),
            4,
            "the strip below the cut is a rectangle, and the crossing with the far edge's line              does not put a fifth corner on it"
        );
    }

    /// A face shaped like a U, cut by a line straight across both arms. The two
    /// pieces above the cut are on the same side of it and are still two pieces,
    /// which is the case that tells a real arrangement from clipping a polygon
    /// against a half-space.
    #[test]
    fn a_cut_across_both_arms_of_a_u_leaves_three_pieces_and_not_two() {
        let top = the_top_of_an_extruded(&[
            [0.0, 0.0],
            [6.0, 0.0],
            [6.0, 4.0],
            [4.0, 4.0],
            [4.0, 2.0],
            [2.0, 2.0],
            [2.0, 4.0],
            [0.0, 4.0],
        ]);

        let pieces = split(&cut_by(&top, &[[0.0, 1.0, 0.0, -3.0]]));

        assert_eq!(pieces.len(), 3);
        let mut areas: Vec<f64> = pieces.iter().map(twice_the_area_of).collect();
        areas.sort_by(f64::total_cmp);
        assert_eq!(areas, [2.0 * 2.0, 2.0 * 2.0, 2.0 * 16.0]);
        for piece in &pieces {
            every_wire_closes(piece);
        }
    }

    /// A cut that lands exactly on an edge of the face adds nothing: the line is
    /// one the boundary already runs along, and the face comes back whole.
    #[test]
    fn a_cut_along_an_edge_the_face_already_has_leaves_the_face_alone() {
        let (model, cube) = a_cube_of_side_two();
        let top = cut_by(&the_top_of_a_cube(&model, cube), &[[1.0, 0.0, 0.0, 0.0]]);

        let pieces = split(&top);

        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].wires[0].len(), 4);
        assert_eq!(twice_the_area_of(&pieces[0]), 8.0);
    }
}
