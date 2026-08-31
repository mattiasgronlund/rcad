use rcad_geom::{Isometry, Orientation, Plane, Point, Surface, Vector};
use rcad_predicates::{
    Approximation, Decision, Policy, ProvenanceLog, Record, orient2d_tolerantly,
};
use smallvec::SmallVec;

use crate::key::{CoedgeSlot, EdgeSlot, FaceSlot, VertexSlot, WireSlot};
use crate::model::{FaceUse, ShellUse};
use crate::{EdgeKey, Error, FaceKey, Model, Result, SolidKey, VertexKey};

/// A flat outline with holes in it, checked and ready to extrude.
///
/// Sitting in the `z = 0` plane in the model's own units. Extruding it upwards
/// is how M0 makes solids: a cube, a plate with holes in it, or whatever the
/// property tests dream up (decisions.md §9.1).
///
/// Every profile that exists has already been checked: at least three points
/// per loop, no repeated points, nothing crossing anything, every hole wholly
/// inside the outline and outside every other hole. The loops are also turned
/// the right way round — outline anticlockwise, holes clockwise — so the
/// extrusion code does not have to think about it.
#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    outline: Vec<[f64; 2]>,
    holes: Vec<Vec<[f64; 2]>>,
    approximation: Option<Approximation>,
}

impl Profile {
    /// The outer boundary, running anticlockwise.
    #[must_use]
    pub fn outline(&self) -> &[[f64; 2]] {
        &self.outline
    }

    /// The holes, each running clockwise.
    #[must_use]
    pub fn holes(&self) -> &[Vec<[f64; 2]>] {
        &self.holes
    }

    /// The slop a tolerant policy had to accept to call these loops a profile,
    /// or `None` when every turn was proved.
    ///
    /// The profile is only sound *because* that slop was allowed, so
    /// [`Model::add_extrusion`] hangs it on everything it builds from these
    /// loops rather than leaving the caller to remember (decisions.md §1.2).
    #[must_use]
    pub fn approximation(&self) -> Option<Approximation> {
        self.approximation
    }
}

/// Collects the loops of a [`Profile`] and checks them all at once.
///
/// The checks have to see every loop together — a hole is only a hole if it is
/// inside the outline and outside the other holes — so there is nothing useful
/// to check until [`ProfileBuilder::build`] is called.
#[derive(Clone, Debug, Default)]
pub struct ProfileBuilder {
    outline: Vec<[f64; 2]>,
    holes: Vec<Vec<[f64; 2]>>,
}

impl ProfileBuilder {
    /// Start from an outer boundary. Either way round is fine; it is turned
    /// anticlockwise on the way out.
    ///
    /// The last point must not repeat the first: the loop closes on its own.
    #[must_use]
    pub fn new(outline: impl Into<Vec<[f64; 2]>>) -> Self {
        Self {
            outline: outline.into(),
            holes: Vec::new(),
        }
    }

    /// Add a hole. Either way round is fine; it is turned clockwise on the way
    /// out.
    #[must_use]
    pub fn with_hole(mut self, hole: impl Into<Vec<[f64; 2]>>) -> Self {
        self.holes.push(hole.into());
        self
    }

    /// Check everything and hand back a profile that can be extruded.
    ///
    /// The crossing check compares every edge against every other one, so this
    /// costs the square of the total number of points. That is fine for the
    /// sizes M0 builds and is worth knowing before someone feeds it a mesh.
    ///
    /// Every turn `policy` could not certify lands in `log`, whether the build
    /// went on to succeed or not. A build that proved everything leaves `log`
    /// untouched, and that empty log is itself the signal that nothing was
    /// approximated (decisions.md §5).
    ///
    /// # Errors
    ///
    /// [`Error::TooFewPoints`], [`Error::PointNotFinite`],
    /// [`Error::RepeatedPoint`], [`Error::ProfileCrossesItself`],
    /// [`Error::HoleNotInsideOutline`] or [`Error::HoleInsideAnotherHole`]
    /// when the loops are not a sound profile,
    /// and [`Error::Undecidable`] when a turn could not be settled under
    /// `policy`. That last one carries the records itself and leaves `log`
    /// alone, so no decision is reported in two places at once.
    pub fn build(self, policy: &Policy, log: &mut ProvenanceLog) -> Result<Profile> {
        let mut mine = ProvenanceLog::new();
        let built = self.checked_under(policy, &mut mine);
        mine.drain_into(log);
        built
    }

    fn checked_under(self, policy: &Policy, log: &mut ProvenanceLog) -> Result<Profile> {
        let Self {
            mut outline,
            mut holes,
        } = self;

        check_one_loop(&outline)?;
        for hole in &holes {
            check_one_loop(hole)?;
        }
        check_nothing_crosses(&outline, &holes, policy, log)?;

        if !ring_runs_anticlockwise(&outline, policy, log)? {
            outline.reverse();
        }
        for hole in &mut holes {
            if ring_runs_anticlockwise(hole, policy, log)? {
                hole.reverse();
            }
        }

        check_holes_sit_inside(&outline, &holes, policy, log)?;

        Ok(Profile {
            outline,
            holes,
            approximation: worst_approximation(log),
        })
    }
}

/// The largest slop any of these decisions left behind.
///
/// The largest, because the profile is only sound if *every* turn in it was
/// settled, so the worst one is what the whole profile is worth.
fn worst_approximation(log: &ProvenanceLog) -> Option<Approximation> {
    log.records()
        .iter()
        .filter_map(Record::approximation)
        .max_by(|one, other| one.magnitude().total_cmp(&other.magnitude()))
}

fn check_one_loop(ring: &[[f64; 2]]) -> Result<()> {
    if ring.len() < 3 {
        return Err(Error::TooFewPoints { count: ring.len() });
    }
    for (position, point) in ring.iter().enumerate() {
        if !point.iter().all(|value| value.is_finite()) {
            return Err(Error::PointNotFinite { position });
        }
        if *point == ring[(position + 1) % ring.len()] {
            return Err(Error::RepeatedPoint {
                position: (position + 1) % ring.len(),
            });
        }
    }
    Ok(())
}

fn check_nothing_crosses(
    outline: &[[f64; 2]],
    holes: &[Vec<[f64; 2]>],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<()> {
    let mut edges: Vec<ProfileEdge> = Vec::new();
    for (ring, points) in core::iter::once(outline)
        .chain(holes.iter().map(Vec::as_slice))
        .enumerate()
    {
        for (index, &start) in points.iter().enumerate() {
            edges.push(ProfileEdge {
                ring,
                index,
                count: points.len(),
                ends: [start, points[(index + 1) % points.len()]],
            });
        }
    }

    for first in 0..edges.len() {
        for second in (first + 1)..edges.len() {
            let (one, other) = (&edges[first], &edges[second]);
            match one.runs_into(other) {
                Some(corner) => check_the_corner_does_not_double_back(corner, policy, log)?,
                None => {
                    if segments_meet(one.ends, other.ends, policy, log)? {
                        return Err(Error::ProfileCrossesItself);
                    }
                }
            }
        }
    }
    Ok(())
}

/// One edge of one loop of a profile, and where it sits in that loop.
///
/// Which loop and which position matter. Two edges that meet end to end always
/// touch and that is fine; two that share a point without being neighbours are
/// a loop pinched against itself, which is not. Deciding that by comparing
/// coordinates would confuse the two.
struct ProfileEdge {
    ring: usize,
    index: usize,
    count: usize,
    ends: [[f64; 2]; 2],
}

impl ProfileEdge {
    /// The three points `a`, `b`, `c` when these two edges are neighbours in
    /// the same loop, running `a` to `b` to `c`. `None` when they are not
    /// neighbours.
    fn runs_into(&self, other: &ProfileEdge) -> Option<[[f64; 2]; 3]> {
        if self.ring != other.ring {
            return None;
        }
        if (self.index + 1) % self.count == other.index {
            return Some([self.ends[0], self.ends[1], other.ends[1]]);
        }
        if (other.index + 1) % other.count == self.index {
            return Some([other.ends[0], other.ends[1], self.ends[1]]);
        }
        None
    }
}

/// Two edges that meet end to end may only touch at that corner.
///
/// They share it by construction, so a shared corner is never a crossing however
/// sharp it is. The only way `a` to `b` to `c` can go wrong is by folding flat:
/// all three on one line, with the two edges then covering a stretch of that
/// line twice over.
///
/// The turn has to be decided before the between test runs. [`point_is_between`]
/// is a bounding-box test and only means anything once the three points are
/// known to share a line; on a corner that genuinely turns it reports any
/// overlap of the two boxes as a fold, which every triangle has.
///
/// # Errors
///
/// [`Error::ProfileCrossesItself`] if the corner folds flat. [`Error::Undecidable`]
/// if the turn cannot be decided.
fn check_the_corner_does_not_double_back(
    [a, b, c]: [[f64; 2]; 3],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<()> {
    match turn_of(a, b, c, policy, log)? {
        Turn::Left | Turn::Right => Ok(()),
        Turn::Straight if point_is_between(c, a, b) || point_is_between(a, b, c) => {
            Err(Error::ProfileCrossesItself)
        }
        Turn::Straight => Ok(()),
    }
}

fn check_holes_sit_inside(
    outline: &[[f64; 2]],
    holes: &[Vec<[f64; 2]>],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<()> {
    for (index, hole) in holes.iter().enumerate() {
        match where_point_sits(hole[0], outline, policy, log)? {
            Where::Inside => {}
            Where::OnEdge | Where::Outside => {
                return Err(Error::HoleNotInsideOutline { hole: index });
            }
        }
        for (other_index, other) in holes.iter().enumerate() {
            if other_index == index {
                continue;
            }
            match where_point_sits(hole[0], other, policy, log)? {
                Where::Outside => {}
                Where::Inside | Where::OnEdge => {
                    return Err(Error::HoleInsideAnotherHole { hole: index });
                }
            }
        }
    }
    Ok(())
}

/// Whether the ring runs anticlockwise.
///
/// Read off the corner nobody can argue about: the lowest point, and the
/// leftmost of those. A simple ring always turns the same way as the ring does
/// at that corner.
///
/// # Errors
///
/// [`Error::Undecidable`] if the turn cannot be decided, **including when it
/// comes back straight**.
///
/// Straight used to be an `unreachable!` here, on the argument that
/// [`check_nothing_crosses`] has already run and rules it out: both neighbours
/// of the lowest, leftmost corner sit at or above it, so three points on one
/// line would leave both neighbours on the *same* ray out of the corner, which
/// is the fold [`check_the_corner_does_not_double_back`] refuses.
///
/// That argument is about three points on one line, and [`Turn::Straight`] is
/// not that. It is whatever `orient2d_tolerantly` called
/// [`Decision::Zero`](rcad_predicates::Decision::Zero), and under
/// [`Strictness::TolerantFallback`](rcad_predicates::Strictness) that includes
/// an enclosure the policy waved through without settling. A clean right-angled
/// corner reads straight once its coordinates are small enough — an ordinary
/// square of side `1e-170` did it — while
/// [`check_the_corner_does_not_double_back`] passes, because that one asks an
/// exact question the tolerant policy never touches.
///
/// So the two checks did not stop agreeing. They were answering different
/// questions, and only one of them was the question the argument was about.
/// A corner nobody could settle is exactly [`Error::Undecidable`], and this is
/// caller input rather than a broken invariant (`docs/decisions.md` §4.1).
fn ring_runs_anticlockwise(
    ring: &[[f64; 2]],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    let count = ring.len();
    let mut lowest = 0;
    for (index, point) in ring.iter().enumerate() {
        let best = ring[lowest];
        if point[1] < best[1] || (point[1] == best[1] && point[0] < best[0]) {
            lowest = index;
        }
    }
    let before = ring[(lowest + count - 1) % count];
    let after = ring[(lowest + 1) % count];
    match turn_of(before, ring[lowest], after, policy, log)? {
        Turn::Left => Ok(true),
        Turn::Right => Ok(false),
        Turn::Straight => Err(Error::Undecidable {
            records: core::mem::take(log),
        }),
    }
}

/// Whether the two closed segments share any point at all.
///
/// # Errors
///
/// [`Error::Undecidable`] if any of the four turns cannot be decided.
fn segments_meet(
    first: [[f64; 2]; 2],
    second: [[f64; 2]; 2],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    let [p, q] = first;
    let [r, s] = second;

    let p_side = turn_of(r, s, p, policy, log)?;
    let q_side = turn_of(r, s, q, policy, log)?;
    let r_side = turn_of(p, q, r, policy, log)?;
    let s_side = turn_of(p, q, s, policy, log)?;

    let straddles_second = p_side != q_side && p_side != Turn::Straight && q_side != Turn::Straight;
    let straddles_first = r_side != s_side && r_side != Turn::Straight && s_side != Turn::Straight;
    if straddles_second && straddles_first {
        return Ok(true);
    }

    Ok((p_side == Turn::Straight && point_is_between(p, r, s))
        || (q_side == Turn::Straight && point_is_between(q, r, s))
        || (r_side == Turn::Straight && point_is_between(r, p, q))
        || (s_side == Turn::Straight && point_is_between(s, p, q)))
}

/// Where `point` sits relative to the closed ring `ring`.
///
/// Counts how many times the ring winds round the point, which is right for a
/// ring that runs either way and does not care whether the ring is convex.
///
/// # Errors
///
/// [`Error::Undecidable`] if any turn cannot be decided.
fn where_point_sits(
    point: [f64; 2],
    ring: &[[f64; 2]],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Where> {
    let mut winding = 0i32;
    for (index, &start) in ring.iter().enumerate() {
        let end = ring[(index + 1) % ring.len()];
        let side = turn_of(start, end, point, policy, log)?;
        if side == Turn::Straight && point_is_between(point, start, end) {
            return Ok(Where::OnEdge);
        }
        if start[1] <= point[1] {
            if end[1] > point[1] && side == Turn::Left {
                winding += 1;
            }
        } else if end[1] <= point[1] && side == Turn::Right {
            winding -= 1;
        }
    }
    Ok(if winding == 0 {
        Where::Outside
    } else {
        Where::Inside
    })
}

/// Whether `point` is inside a ring, on one of its edges, or outside.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Where {
    Inside,
    OnEdge,
    Outside,
}

/// Which way `a`, `b`, `c` turn, decided by the exact cascade.
///
/// # Errors
///
/// [`Error::Undecidable`] when the cascade cannot settle the sign. For plain
/// `f64` inputs that only happens when a product overflows or underflows.
///
/// This used to add "which is what the dynamic-range check exists to catch".
/// It does not: `DynamicRange::is_within_supported_range` compares a **ratio**
/// against `LARGEST_SUPPORTED_RATIO` and never looks at absolute magnitude, so
/// a model that is uniformly tiny or uniformly enormous passes it with a ratio
/// of exactly one — and uniformly tiny is precisely where the products fall
/// into the subnormals. `docs/decisions.md` §16.11 carries the open question.
fn turn_of(
    a: [f64; 2],
    b: [f64; 2],
    c: [f64; 2],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Turn> {
    match orient2d_tolerantly(a, b, c, policy, log) {
        Decision::Pos => Ok(Turn::Left),
        Decision::Zero => Ok(Turn::Straight),
        Decision::Neg => Ok(Turn::Right),
        Decision::Undecidable { .. } => Err(Error::Undecidable {
            records: core::mem::take(log),
        }),
        // `Decision` grows a variant at every milestone. Whatever the new one
        // means, guessing is worse than stopping.
        other => panic!("orient2d came back with a decision nothing here knows: {other:?}"),
    }
}

/// Which way three points in a row turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Turn {
    /// Anticlockwise.
    Left,
    /// All three on one straight line.
    Straight,
    /// Clockwise.
    Right,
}

/// Whether `point`, already known to be on the line through `start` and `end`,
/// is also between them.
fn point_is_between(point: [f64; 2], start: [f64; 2], end: [f64; 2]) -> bool {
    (0..2).all(|axis| {
        point[axis] >= start[axis].min(end[axis]) && point[axis] <= start[axis].max(end[axis])
    })
}

impl Model {
    /// Build a cube with one corner at the origin and the opposite one at
    /// `(side, side, side)`.
    ///
    /// # Errors
    ///
    /// [`Error::HeightNotPositive`] if `side` is not a positive finite number,
    /// and whatever [`ProfileBuilder::build`] makes of the square.
    pub fn add_cube(
        &mut self,
        side: f64,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<SolidKey> {
        if !(side.is_finite() && side > 0.0) {
            return Err(Error::HeightNotPositive { height: side });
        }
        let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
            .build(policy, log)?;
        self.add_extrusion(&square, side, &Isometry::identity())
    }

    /// Extrude a profile straight up by `height` and place the result with
    /// `placement`.
    ///
    /// The solid comes out with its faces pointing outwards, every edge used by
    /// exactly two coedges running opposite ways, and the placement recorded on
    /// the shell rather than baked into the coordinates — so the profile's own
    /// numbers survive untouched and the location composes on the way out
    /// (decisions.md §3.2).
    ///
    /// Nothing here makes a decision — the profile arrived already checked — so
    /// there is no provenance to hand back. What the profile *does* bring with
    /// it is the slop a tolerant policy had to accept to call it a profile at
    /// all, and that lands on every entity this builds
    /// (see [`Profile::approximation`]).
    ///
    /// # Errors
    ///
    /// [`Error::HeightNotPositive`] if `height` is not a positive finite
    /// number.
    ///
    /// # Panics
    ///
    /// If the solid it just built breaks one of the invariants checked on
    /// touched entities. That is a bug here, not bad input.
    pub fn add_extrusion(
        &mut self,
        profile: &Profile,
        height: f64,
        placement: &Isometry,
    ) -> Result<SolidKey> {
        if !(height.is_finite() && height > 0.0) {
            return Err(Error::HeightNotPositive { height });
        }
        let approximation_before = self.worst_approximation();

        let rings: Vec<&[[f64; 2]]> = core::iter::once(profile.outline())
            .chain(profile.holes().iter().map(Vec::as_slice))
            .collect();
        let ladders: Vec<Ladder> = rings
            .iter()
            .map(|ring| Ladder::built_in(self, ring, height))
            .collect();

        let mut faces = Vec::new();
        faces.push(self.build_flat_face(&ladders, height, Side::Bottom));
        faces.push(self.build_flat_face(&ladders, height, Side::Top));
        for (ring, ladder) in rings.iter().zip(&ladders) {
            for index in 0..ring.len() {
                faces.push(self.build_side_face(ring, ladder, index));
            }
        }

        let shell = self.add_shell(
            faces
                .iter()
                .map(|&face| FaceUse {
                    face,
                    loc: Isometry::identity(),
                    orient: Orientation::Forward,
                })
                .collect(),
        );
        let solid = self.add_solid(SmallVec::from_buf([ShellUse {
            shell,
            loc: *placement,
            orient: Orientation::Forward,
        }]));

        self.check_touched_solid(solid);
        match profile.approximation() {
            None => self.assert_approximation_did_not_grow(approximation_before),
            Some(tag) => {
                self.tag_everything_the_profile_made(&ladders, &faces, tag);
                // An extrusion of an approximated profile is allowed to leave
                // the model more approximate than it found it — by exactly the
                // slop the profile recorded, and not a bit more.
                self.assert_approximation_did_not_grow(approximation_before.max(tag.magnitude()));
            }
        }
        Ok(self.key_for_solid(solid))
    }

    /// The bottom or top cap, one wire per ring.
    fn build_flat_face(&mut self, ladders: &[Ladder], height: f64, side: Side) -> FaceSlot {
        let plane = Plane::from_point_and_normal(
            Point::new(0.0, 0.0, side.z(height)),
            Vector::new(0.0, 0.0, side.outward_z()),
        )
        .expect("the z axis is a unit normal");

        let wires: SmallVec<[WireSlot; 1]> = ladders
            .iter()
            .map(|ladder| {
                let coedges = match side {
                    Side::Top => ladder
                        .top_edges
                        .iter()
                        .map(|&edge| self.add_coedge(edge, Orientation::Forward))
                        .collect(),
                    Side::Bottom => ladder
                        .bottom_edges
                        .iter()
                        .rev()
                        .map(|&edge| self.add_coedge(edge, Orientation::Reversed))
                        .collect(),
                };
                self.add_wire(coedges)
            })
            .collect();

        self.add_face(Surface::Plane(plane), wires)
    }

    /// The wall standing on ring edge `index`, running up and back down.
    fn build_side_face(&mut self, ring: &[[f64; 2]], ladder: &Ladder, index: usize) -> FaceSlot {
        let count = ring.len();
        let next = (index + 1) % count;
        let along = [
            ring[next][0] - ring[index][0],
            ring[next][1] - ring[index][1],
        ];
        let plane = Plane::from_point_and_normal(
            Point::new(ring[index][0], ring[index][1], 0.0),
            Vector::new(along[1], -along[0], 0.0),
        )
        .expect("a profile never repeats a point, so this normal has length");

        let coedges: SmallVec<[CoedgeSlot; 4]> = SmallVec::from_buf([
            self.add_coedge(ladder.bottom_edges[index], Orientation::Forward),
            self.add_coedge(ladder.vertical_edges[next], Orientation::Forward),
            self.add_coedge(ladder.top_edges[index], Orientation::Reversed),
            self.add_coedge(ladder.vertical_edges[index], Orientation::Reversed),
        ]);
        let wire = self.add_wire(coedges);
        self.add_face(Surface::Plane(plane), SmallVec::from_buf([wire]))
    }

    /// Hang the profile's slop on everything the extrusion just built.
    ///
    /// The same tag everywhere, because the profile was accepted as a whole:
    /// the turn that had to be approximated is what made all of these entities
    /// legal, and there is no honest way to say the slop belongs to one corner
    /// rather than its neighbour.
    ///
    /// Vertices, then edges, then faces, so `tol(vertex) >= tol(edge) >=
    /// tol(face)` holds after every single step and not only at the end
    /// (roadmap §5.4).
    ///
    /// # Panics
    ///
    /// If any of those tags is refused. Everything here was made moments ago
    /// and carries no slop yet, so a refusal is a bug in this function.
    fn tag_everything_the_profile_made(
        &mut self,
        ladders: &[Ladder],
        faces: &[FaceSlot],
        tag: Approximation,
    ) {
        let model = self.id();
        let vertices: Vec<VertexSlot> = ladders.iter().flat_map(Ladder::vertices).collect();
        for vertex in vertices {
            self.set_vertex_approximation(VertexKey::new(vertex, model), tag)
                .expect("a fresh vertex carries no slop for this tag to undercut");
        }
        let edges: Vec<EdgeSlot> = ladders.iter().flat_map(Ladder::edges).collect();
        for edge in edges {
            self.set_edge_approximation(EdgeKey::new(edge, model), tag)
                .expect("the vertices were tagged with the same amount just now");
        }
        for &face in faces {
            self.set_face_approximation(FaceKey::new(face, model), tag)
                .expect("the edges were tagged with the same amount just now");
        }
    }
}

/// Which cap of the extrusion.
#[derive(Clone, Copy, Debug)]
enum Side {
    Bottom,
    Top,
}

impl Side {
    fn z(self, height: f64) -> f64 {
        match self {
            Side::Bottom => 0.0,
            Side::Top => height,
        }
    }

    fn outward_z(self) -> f64 {
        match self {
            Side::Bottom => -1.0,
            Side::Top => 1.0,
        }
    }
}

/// The vertices and edges one ring of the profile turns into: a bottom rung, a
/// top rung, and the uprights joining them.
struct Ladder {
    bottom_vertices: Vec<VertexSlot>,
    top_vertices: Vec<VertexSlot>,
    bottom_edges: Vec<EdgeSlot>,
    top_edges: Vec<EdgeSlot>,
    vertical_edges: Vec<EdgeSlot>,
}

impl Ladder {
    fn built_in(model: &mut Model, ring: &[[f64; 2]], height: f64) -> Self {
        let bottom_vertices: Vec<VertexSlot> = ring
            .iter()
            .map(|point| model.add_vertex(Point::new(point[0], point[1], 0.0)))
            .collect();
        let top_vertices: Vec<VertexSlot> = ring
            .iter()
            .map(|point| model.add_vertex(Point::new(point[0], point[1], height)))
            .collect();

        let count = ring.len();
        let bottom_edges = (0..count)
            .map(|index| {
                model.add_edge(bottom_vertices[index], bottom_vertices[(index + 1) % count])
            })
            .collect();
        let top_edges = (0..count)
            .map(|index| model.add_edge(top_vertices[index], top_vertices[(index + 1) % count]))
            .collect();
        let vertical_edges = (0..count)
            .map(|index| model.add_edge(bottom_vertices[index], top_vertices[index]))
            .collect();

        Self {
            bottom_vertices,
            top_vertices,
            bottom_edges,
            top_edges,
            vertical_edges,
        }
    }

    fn vertices(&self) -> impl Iterator<Item = VertexSlot> + '_ {
        self.bottom_vertices
            .iter()
            .chain(&self.top_vertices)
            .copied()
    }

    fn edges(&self) -> impl Iterator<Item = EdgeSlot> + '_ {
        self.bottom_edges
            .iter()
            .chain(&self.top_edges)
            .chain(&self.vertical_edges)
            .copied()
    }
}
