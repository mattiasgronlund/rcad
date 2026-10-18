use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Sphere,
    Surface, SurfaceMeet, Vector,
};
use rcad_predicates::{
    Approximation, Decision, Policy, ProvenanceLog, Record, orient2d_tolerantly,
};
use smallvec::SmallVec;

use crate::key::{CoedgeSlot, EdgeSlot, FaceSlot, VertexSlot, WireSlot};
use crate::model::{FaceUse, ShellUse, WireStep};
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
/// an enclosure the policy waved through without settling, while
/// [`check_the_corner_does_not_double_back`] passes, because that one asks an
/// exact question the tolerant policy never touches.
///
/// So the two checks did not stop agreeing. They were answering different
/// questions, and only one of them was the question the argument was about.
/// A corner nobody could settle is exactly [`Error::Undecidable`], and this is
/// caller input rather than a broken invariant (`docs/decisions.md` §4.1).
///
/// The input that showed this — an ordinary square of side `1e-170` — no
/// longer reads straight, because `orient2d` now normalises uniformly small
/// coordinates and settles that corner outright. Nothing here has an example
/// left, and that is a reason to keep the arm rather than to drop it:
/// unreachable was the claim that turned out to be wrong the first time.
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

    /// A round bar of `radius` standing `height` tall on the origin, placed
    /// with `placement`.
    ///
    /// The first solid in this kernel with a curved face, and it is built the
    /// way it is because of what it has to force into existence rather than
    /// because a bar is useful. It has:
    ///
    /// - a **curved face**, the wall, which no planar check can look at;
    /// - a **seam**, the ruling up the wall where the surface wraps round, used
    ///   *twice by the same face* — roadmap §5.3 calls this the source of most
    ///   subtle bugs, and nothing here could reach it before;
    /// - two **closed edges**, the cap circles, which start and end at one
    ///   vertex and so are the whole of the curve they run along.
    ///
    /// Two vertices and three edges is the whole of it, which is the point: a
    /// bar this small has every structure and no arithmetic to hide behind.
    ///
    /// The seam sits at `x = radius, y = 0`, so both vertices are exact for any
    /// radius `f64` can hold, and both are certified onto the cylinder by
    /// `x² + y² − r²` cancelling to exactly zero. `Model::validate` asks that of
    /// every corner of every curved face, and this is the builder that has to
    /// pass it.
    ///
    /// # Errors
    ///
    /// [`Error::HeightNotPositive`] if `height` is not a positive finite
    /// number, and whatever [`Cylinder::new`] makes of the radius.
    ///
    /// # Panics
    ///
    /// If the solid it just built breaks one of the invariants checked on
    /// touched entities. That is a bug here, not bad input.
    pub fn add_round_bar(
        &mut self,
        radius: f64,
        height: f64,
        placement: &Isometry,
    ) -> Result<SolidKey> {
        if !(height.is_finite() && height > 0.0) {
            return Err(Error::HeightNotPositive { height });
        }
        let wall = Cylinder::new(Axis::z(), radius)?;
        let approximation_before = self.worst_approximation();

        let bottom_corner = self.add_vertex(Point::new(radius, 0.0, 0.0));
        let top_corner = self.add_vertex(Point::new(radius, 0.0, height));
        let seam = self.add_edge(bottom_corner, top_corner);
        let bottom_circle = self.add_closed_edge(bottom_corner);
        let top_circle = self.add_closed_edge(top_corner);

        let cap = |model: &mut Self, z: f64, outward: f64, edge, turn| {
            let plane = Plane::from_point_and_normal(
                Point::new(0.0, 0.0, z),
                Vector::new(0.0, 0.0, outward),
            )
            .expect("the z axis is a unit normal");
            let coedge = model.add_coedge(edge, turn);
            let wire = model.add_wire(smallvec::smallvec![coedge]);
            model.add_face(Surface::Plane(plane), SmallVec::from_buf([wire]))
        };
        let bottom = cap(self, 0.0, -1.0, bottom_circle, Orientation::Reversed);
        let top = cap(self, height, 1.0, top_circle, Orientation::Forward);

        // Round the bottom circle, up the seam, back round the top circle, and
        // down the seam again. The seam's two coedges are both in this one
        // wire, which is what makes it a seam rather than an ordinary edge.
        let round_the_wall: SmallVec<[CoedgeSlot; 4]> = SmallVec::from_buf([
            self.add_coedge(bottom_circle, Orientation::Forward),
            self.add_coedge(seam, Orientation::Forward),
            self.add_coedge(top_circle, Orientation::Reversed),
            self.add_coedge(seam, Orientation::Reversed),
        ]);
        let wire = self.add_wire(round_the_wall);
        let side = self.add_face(Surface::Cylinder(wall), SmallVec::from_buf([wire]));

        let shell = self.add_shell(
            [bottom, top, side]
                .into_iter()
                .map(|face| FaceUse {
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
        self.assert_approximation_did_not_grow(approximation_before);
        Ok(self.key_for_solid(solid))
    }

    /// A ball of `radius` about the origin, placed with `placement`.
    ///
    /// **One face, one seam, two poles**, and the alternatives were rejected
    /// for what they would fail to exercise. Two hemispheres joined at the
    /// equator never puts a seam on one face; a six-patch cube-sphere has
    /// neither a seam nor a pole. A ball whose whole purpose is to be the first
    /// shape with both should not be built in the one arrangement that has
    /// neither.
    ///
    /// So the boundary of the single face is a rectangle in `(u, v)` whose four
    /// sides are: down the seam, the south pole, back up the seam, the north
    /// pole. Two of those four steps go nowhere at all, which is what a
    /// [`WireStep::Pole`] is.
    ///
    /// The seam is the edge that forced [`EdgeRun::PartOfWhereItCrosses`]. Both
    /// its ends are the poles and both its sides are the same sphere, so
    /// neither the faces nor the endpoints say which half of the meridian
    /// circle it is — see that variant for why.
    ///
    /// Both poles sit at `(0, 0, ±radius)`, so `x² + y² + z² − r²` is
    /// `r² − r²` and cancels to exactly zero: the corners are *certified* onto
    /// the sphere for any radius `f64` can hold, which is what `Model::validate`
    /// asks of every corner of every curved face.
    ///
    /// # Errors
    ///
    /// Whatever [`Sphere::new`] makes of the radius.
    ///
    /// # Panics
    ///
    /// If the solid it just built breaks one of the invariants checked on
    /// touched entities. That is a bug here, not bad input.
    pub fn add_ball(&mut self, radius: f64, placement: &Isometry) -> Result<SolidKey> {
        let skin = Sphere::new(Point::origin(), radius)?;
        let approximation_before = self.worst_approximation();

        let north = self.add_vertex(Point::new(0.0, 0.0, radius));
        let south = self.add_vertex(Point::new(0.0, 0.0, -radius));

        // The meridian lies in the plane `y = 0`, which crosses the sphere in
        // one whole circle; the seam is the half of that circle with `x >= 0`.
        let meridian = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
            .expect("the y axis is a unit normal");
        let facing_x =
            HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
                .expect("the x axis has length");
        let seam = self.add_edge_running(
            north,
            south,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(meridian),
                picked_by: smallvec::smallvec![Picker::from(facing_x)],
            },
        );

        let steps: SmallVec<[WireStep; 4]> = SmallVec::from_buf([
            WireStep::Along(self.add_coedge(seam, Orientation::Forward)),
            WireStep::Pole(south),
            WireStep::Along(self.add_coedge(seam, Orientation::Reversed)),
            WireStep::Pole(north),
        ]);
        let wire = self.add_wire_of_steps(steps);
        let face = self.add_face(Surface::Sphere(skin), SmallVec::from_buf([wire]));

        let shell = self.add_shell(vec![FaceUse {
            face,
            loc: Isometry::identity(),
            orient: Orientation::Forward,
        }]);
        let solid = self.add_solid(SmallVec::from_buf([ShellUse {
            shell,
            loc: *placement,
            orient: Orientation::Forward,
        }]));

        self.check_touched_solid(solid);
        self.assert_approximation_did_not_grow(approximation_before);
        Ok(self.key_for_solid(solid))
    }

    /// A round bar of `radius` about the z axis cut off by two planes, with its
    /// cap edges **worked out rather than declared**.
    ///
    /// [`Model::add_round_bar`] says its caps are circles and nothing checks it.
    /// This asks [`intersect`](rcad_geom::intersect) where the wall and each cap
    /// plane cross and puts the answer on the edge. Two planes square to the
    /// axis give that same bar, computed instead of asserted; one of them
    /// leaning gives a shape `add_round_bar` cannot make at all — a bar cut off
    /// at an angle, whose cap edge is an ellipse. `docs/decisions.md` §24.2 names
    /// exactly that pair as M3's own consumer, and §24.18's criterion 8 is what
    /// the mesh of it has to pass.
    ///
    /// `caps[0]` is the bottom and `caps[1]` the top, each facing **out** of the
    /// bar: the bottom's normal points down the axis and the top's up it.
    ///
    /// The seam is the ruling at `x = radius, y = 0`, so both corners are exact
    /// on the wall for any radius `f64` can hold — `x² + y² − r²` cancels to
    /// zero — which is what `Model::validate` asks of every corner of a curved
    /// face. How far up the seam each of them sits comes from the cap plane it
    /// is on, and that division rounds, so a corner is on its cap to a rounding
    /// rather than exactly. A flat face is checked against a tolerance and a
    /// curved one against a predicate, which is the right way round: the wall is
    /// where an error would put a corner off the shape.
    ///
    /// # Errors
    ///
    /// Whatever [`Cylinder::new`] makes of the radius, and whatever
    /// [`intersect`](rcad_geom::intersect) makes of the pair.
    /// [`Error::ACapDoesNotCutRightRoundABar`] when a cap plane runs down the
    /// bar rather than across it, misses it, or only touches it,
    /// [`Error::ACapOfABarFacesIntoIt`] when a cap's normal points along the
    /// axis towards the other cap, and [`Error::TheCapsOfABarMeetInsideIt`] when
    /// the two caps leave no body between them.
    ///
    /// # Panics
    ///
    /// If the solid it just built breaks one of the invariants checked on
    /// touched entities. That is a bug here, not bad input.
    pub fn add_bar_cut_off_by(
        &mut self,
        radius: f64,
        caps: [Plane; 2],
        placement: &Isometry,
        policy: &Policy,
        log: &mut ProvenanceLog,
    ) -> Result<SolidKey> {
        let wall = Cylinder::new(Axis::z(), radius)?;
        let mut runs = Vec::with_capacity(2);
        for (which, cap) in caps.iter().enumerate() {
            runs.push(right_round_where_a_cap_cuts_a_bar(
                &wall, cap, which, policy, log,
            )?);
        }
        for (which, cap) in caps.iter().enumerate() {
            let facing = cap.normal().components()[2];
            if (which == 0) != (facing < 0.0) {
                return Err(Error::ACapOfABarFacesIntoIt { which });
            }
        }
        if how_much_body_the_caps_leave(radius, &caps) <= 0.0 {
            return Err(Error::TheCapsOfABarMeetInsideIt);
        }
        let approximation_before = self.worst_approximation();

        let heights = [0, 1].map(|which| how_far_up_the_seam_a_cap_cuts(radius, &caps[which]));
        let corners = heights.map(|height| self.add_vertex(Point::new(radius, 0.0, height)));
        let seam = self.add_edge(corners[0], corners[1]);
        let circles = [0, 1].map(|which| {
            self.add_edge_running(corners[which], corners[which], runs[which].clone())
        });

        let cap = |model: &mut Self, plane, edge, turn| {
            let coedge = model.add_coedge(edge, turn);
            let wire = model.add_wire(smallvec::smallvec![coedge]);
            model.add_face(Surface::Plane(plane), SmallVec::from_buf([wire]))
        };
        let bottom = cap(self, caps[0], circles[0], Orientation::Reversed);
        let top = cap(self, caps[1], circles[1], Orientation::Forward);

        let round_the_wall: SmallVec<[CoedgeSlot; 4]> = SmallVec::from_buf([
            self.add_coedge(circles[0], Orientation::Forward),
            self.add_coedge(seam, Orientation::Forward),
            self.add_coedge(circles[1], Orientation::Reversed),
            self.add_coedge(seam, Orientation::Reversed),
        ]);
        let wire = self.add_wire(round_the_wall);
        let side = self.add_face(Surface::Cylinder(wall), SmallVec::from_buf([wire]));

        let shell = self.add_shell(
            [bottom, top, side]
                .into_iter()
                .map(|face| FaceUse {
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
        self.assert_approximation_did_not_grow(approximation_before);
        Ok(self.key_for_solid(solid))
    }

    /// The bottom or top cap of an extrusion, one wire per ring.
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
    ///
    /// **Its plane goes through whichever of the side's two corners is nearer
    /// the origin**, not through the one the ring lists first. A tilted wall's
    /// normal is rounded, since no `f64` unit vector but an axis's is exact,
    /// and the offset is that normal times the corner it goes through, so the
    /// further that corner is from the origin, the more of the normal's
    /// rounding the offset can carry. A corner at the origin gives offset `0`
    /// exactly. The wedge `y` from `0.25·x` to `0.75·x` lists `(5, 3.75)`
    /// before the origin on its tilted side back, and through `(5, 3.75)` that
    /// side's plane missed the origin by `2.78e-16`, which was enough for its
    /// edge to miss an apex the planes of a pyramid meet at, and for every fuse
    /// of the two to be refused (Q2331).
    ///
    /// Of two corners as near, the rule still names one, so a wall a prism
    /// and the hole it fills both stand on is one plane, turned round, from
    /// either side.
    fn build_side_face(&mut self, ring: &[[f64; 2]], ladder: &Ladder, index: usize) -> FaceSlot {
        let count = ring.len();
        let next = (index + 1) % count;
        let along = [
            ring[next][0] - ring[index][0],
            ring[next][1] - ring[index][1],
        ];
        let [x, y] = nearer_the_origin(ring[index], ring[next]);
        let plane = Plane::from_point_and_normal(
            Point::new(x, y, 0.0),
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

/// Whichever of `one` and `other` is nearer the origin, and of two as near,
/// the one with the smaller `x`, then the smaller `y`, so that the answer is
/// the same whichever is named first.
fn nearer_the_origin(one: [f64; 2], other: [f64; 2]) -> [f64; 2] {
    let how_far = |[x, y]: [f64; 2]| x * x + y * y;
    let order = how_far(one)
        .total_cmp(&how_far(other))
        .then(one[0].total_cmp(&other[0]))
        .then(one[1].total_cmp(&other[1]));
    if order.is_le() { one } else { other }
}

/// What a cap plane cuts out of a bar's wall, which has to be one closed curve.
///
/// **Computed and not declared**, which is the whole point of the builder above.
/// [`Model::add_round_bar`] writes [`EdgeRun::RightRoundWhereItsFacesCross`] on
/// its cap edges because a person knew a plane square to a pipe cuts a circle;
/// this asks. A plane running down the inside of the pipe cuts two straight
/// lines and one along its skin cuts one, and neither is a cap — so the answer
/// is read rather than assumed.
///
/// # Errors
///
/// [`Error::ACapDoesNotCutRightRoundABar`] for every other answer, and whatever
/// [`intersect`](rcad_geom::intersect) itself refuses.
fn right_round_where_a_cap_cuts_a_bar(
    wall: &Cylinder,
    cap: &Plane,
    which: usize,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<EdgeRun> {
    let meet = rcad_geom::intersect(
        &Surface::Cylinder(*wall),
        &Surface::Plane(*cap),
        policy,
        log,
    )?;
    let SurfaceMeet::Curves(runs) = &meet else {
        return Err(Error::ACapDoesNotCutRightRoundABar { which });
    };
    match runs.as_slice() {
        [only] if only.run() == &EdgeRun::RightRoundWhereItsFacesCross => Ok(only.run().clone()),
        _ => Err(Error::ACapDoesNotCutRightRoundABar { which }),
    }
}

/// How far up the seam at `x = radius, y = 0` a cap plane cuts it.
///
/// # Panics
///
/// If the plane is parallel to the axis. The caller has already had
/// [`intersect`](rcad_geom::intersect) answer "one closed curve" about this
/// pair, and a plane parallel to the axis cannot: it cuts two lines, or one, or
/// nothing.
fn how_far_up_the_seam_a_cap_cuts(radius: f64, cap: &Plane) -> f64 {
    let [across, _, up] = cap.normal().components();
    assert!(
        up != 0.0,
        "a plane along a pipe's axis does not cut one closed curve out of it"
    );
    (cap.offset() - across * radius) / up
}

/// How thick the bar between two caps is at its thinnest, which is negative
/// where they cross inside it.
///
/// Each cap meets the wall at a height that goes round as
/// `(d − r(nₓ cos θ + n_y sin θ)) / n_z`, so the gap between them is a constant
/// less a single sinusoid, and its smallest value is that constant less the
/// sinusoid's own size. **Ordinary `f64` and no predicate**, because this is a
/// question about the caller's request rather than about the shape that comes
/// out: a bar too thin to be one is bad input and a bar that is merely thin is a
/// bar (§4.1).
fn how_much_body_the_caps_leave(radius: f64, caps: &[Plane; 2]) -> f64 {
    let leaning = |cap: &Plane| {
        let [across, and_across, up] = cap.normal().components();
        [cap.offset() / up, across / up, and_across / up]
    };
    let [bottom, top] = [leaning(&caps[0]), leaning(&caps[1])];
    (top[0] - bottom[0]) - radius * (top[1] - bottom[1]).hypot(top[2] - bottom[2])
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
