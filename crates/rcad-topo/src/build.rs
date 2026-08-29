use rcad_geom::{Isometry, Orientation, Plane, Point, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use smallvec::SmallVec;

use crate::exact2d::{
    Where, point_is_between, ring_runs_anticlockwise, segments_meet, where_point_sits,
};
use crate::key::{CoedgeSlot, EdgeSlot, FaceSlot, VertexSlot, WireSlot};
use crate::model::{FaceUse, ShellUse};
use crate::{Error, Model, Result, SolidKey};

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
    /// # Errors
    ///
    /// [`Error::TooFewPoints`], [`Error::PointNotFinite`],
    /// [`Error::RepeatedPoint`], [`Error::ProfileCrossesItself`],
    /// [`Error::ProfileEnclosesNoArea`], [`Error::HoleNotInsideOutline`] or
    /// [`Error::HoleInsideAnotherHole`] when the loops are not a sound profile,
    /// and [`Error::Undecidable`] when a turn could not be settled under
    /// `policy`.
    pub fn build(self, policy: &Policy) -> Result<Profile> {
        let mut log = ProvenanceLog::new();
        let Self {
            mut outline,
            mut holes,
        } = self;

        check_one_loop(&outline)?;
        for hole in &holes {
            check_one_loop(hole)?;
        }
        check_nothing_crosses(&outline, &holes, policy, &mut log)?;

        if !ring_runs_anticlockwise(&outline, policy, &mut log)? {
            outline.reverse();
        }
        for hole in &mut holes {
            if ring_runs_anticlockwise(hole, policy, &mut log)? {
                hole.reverse();
            }
        }

        check_holes_sit_inside(&outline, &holes, policy, &mut log)?;

        Ok(Profile { outline, holes })
    }
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
                Some(corner) => check_the_corner_does_not_double_back(corner)?,
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
/// What matters is whether one of them lies back along the other, which is a
/// loop folding onto itself.
fn check_the_corner_does_not_double_back([a, b, c]: [[f64; 2]; 3]) -> Result<()> {
    if point_is_between(c, a, b) || point_is_between(a, b, c) {
        return Err(Error::ProfileCrossesItself);
    }
    Ok(())
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

impl Model {
    /// Build a cube with one corner at the origin and the opposite one at
    /// `(side, side, side)`.
    ///
    /// # Errors
    ///
    /// [`Error::HeightNotPositive`] if `side` is not a positive finite number.
    pub fn add_cube(&mut self, side: f64) -> Result<SolidKey> {
        if !(side.is_finite() && side > 0.0) {
            return Err(Error::HeightNotPositive { height: side });
        }
        let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
            .build(&Policy::certified())?;
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
    bottom_edges: Vec<EdgeSlot>,
    top_edges: Vec<EdgeSlot>,
    vertical_edges: Vec<EdgeSlot>,
}

impl Ladder {
    fn built_in(model: &mut Model, ring: &[[f64; 2]], height: f64) -> Self {
        let bottom: Vec<VertexSlot> = ring
            .iter()
            .map(|point| model.add_vertex(Point::new(point[0], point[1], 0.0)))
            .collect();
        let top: Vec<VertexSlot> = ring
            .iter()
            .map(|point| model.add_vertex(Point::new(point[0], point[1], height)))
            .collect();

        let count = ring.len();
        let bottom_edges = (0..count)
            .map(|index| model.add_edge(bottom[index], bottom[(index + 1) % count]))
            .collect();
        let top_edges = (0..count)
            .map(|index| model.add_edge(top[index], top[(index + 1) % count]))
            .collect();
        let vertical_edges = (0..count)
            .map(|index| model.add_edge(bottom[index], top[index]))
            .collect();

        Self {
            bottom_edges,
            top_edges,
            vertical_edges,
        }
    }
}
