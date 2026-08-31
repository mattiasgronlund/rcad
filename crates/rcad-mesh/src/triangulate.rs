//! Cutting a trimmed patch into triangles.

use std::collections::VecDeque;

use spade::{ConstrainedDelaunayTriangulation, InsertionError, Point2, Triangulation};

use rcad_predicates::{Decision, orient2d};

use crate::domain::TrimmedDomain;

/// A patch cut into triangles.
///
/// The triangles name their corners by **index**, and that is the whole point
/// of handing them back this way. A caller meshing a solid works out where a
/// shared edge's points are once and gives both faces along it the same 3D
/// points; if this returned coordinates instead, each face would map its own
/// `(u, v)` back through its own surface and the two sides would land a
/// rounding apart. An index cannot drift.
///
/// Corners `0..n` are the domain's own, in the order it listed them — the outer
/// loop first, then each hole. Nothing else is in here yet; when refinement
/// arrives, the points it adds go on the end, so an index into the input stays
/// an index into the input.
#[derive(Clone, Debug, PartialEq)]
pub struct Patch {
    corners: Vec<[f64; 2]>,
    triangles: Vec<[u32; 3]>,
}

impl Patch {
    /// The corners, in the order the domain listed them.
    #[must_use]
    pub fn corners(&self) -> &[[f64; 2]] {
        &self.corners
    }

    /// The triangles, each naming three corners by index.
    #[must_use]
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }
}

/// Cut `domain` into triangles.
///
/// # How the domain is scaled, and why
///
/// The triangulation runs on `(u, v)` multiplied by
/// [`Surface::furthest_a_unit_step_goes`], not on `(u, v)` itself. For a
/// cylinder that scaling is an **exact isometry** — a cylinder unrolls flat and
/// the factor is one multiplication — so the triangles Delaunay picks in the
/// scaled domain are the ones it would pick on the surface. Without it a pipe
/// of radius 1000 would be triangulated as though it were a strip `2π` wide,
/// and every triangle would be a sliver.
///
/// A ball does not unroll, so no scaling makes its domain faithful and the
/// crowding near its poles is real. It costs triangles rather than correctness.
///
/// [`Surface::furthest_a_unit_step_goes`]: rcad_geom::Surface::furthest_a_unit_step_goes
///
/// # What is kept
///
/// A Delaunay triangulation fills the whole convex hull of the corners, which
/// is more than the domain: it covers the holes and the dents. Which triangles
/// to throw away is decided by **walking the dual graph and counting**, not by
/// testing points. Start outside, and every time you step across a boundary
/// edge you change sides. Triangles an odd number of boundaries in are inside.
///
/// That is worth doing this way rather than the usual one. Testing whether a
/// triangle's middle is inside the outline is a geometric question about a
/// point nobody constructed exactly, and near a thin sliver it is a question
/// that can fail to have an answer. Counting boundary crossings is arithmetic
/// -free, so there is nothing here to be undecidable about.
///
/// # Errors
///
/// [`CannotTriangulate::BoundaryCrossesItself`] when two boundary edges cross,
/// [`CannotTriangulate::CornersCoincide`] when two corners are the same point,
/// and [`CannotTriangulate::CoordinateOutOfRange`] for a scaled coordinate the
/// triangulation will not take — see that variant, the range is narrower than
/// `f64`'s.
pub fn triangulate(domain: &TrimmedDomain) -> Result<Patch, CannotTriangulate> {
    let corners = every_corner(domain);
    refuse_loops_that_touch(&corners, domain)?;
    let (across, along) = domain.surface().furthest_a_unit_step_goes();

    let mut mesh = ConstrainedDelaunayTriangulation::<Point2<f64>>::new();
    let mut handles = Vec::with_capacity(corners.len());
    for (position, corner) in corners.iter().enumerate() {
        let handle = mesh
            .insert(Point2::new(corner[0] * across, corner[1] * along))
            .map_err(|why| CannotTriangulate::CoordinateOutOfRange { position, why })?;
        if handles.contains(&handle) {
            return Err(CannotTriangulate::CornersCoincide { position });
        }
        handles.push(handle);
    }

    for (from, to) in every_side(domain) {
        // Asking first, because `add_constraint` panics on a crossing and this
        // crate takes its loops from whoever calls it.
        if !mesh.can_add_constraint(handles[from], handles[to]) {
            return Err(CannotTriangulate::BoundaryCrossesItself { at: from });
        }
        mesh.add_constraint(handles[from], handles[to]);
    }

    let inside = walk_in_from_the_outside(&mesh);
    let mut triangles = Vec::new();
    for face in mesh.inner_faces() {
        if !inside[face.index()] {
            continue;
        }
        let [a, b, c] = face.vertices().map(|vertex| {
            u32::try_from(
                handles
                    .iter()
                    .position(|handle| *handle == vertex.fix())
                    .expect("every vertex of the mesh is a corner we put in"),
            )
            .expect("a domain with more than four billion corners is not a domain")
        });
        triangles.push([a, b, c]);
    }

    Ok(Patch { corners, triangles })
}

/// Refuse a domain where a corner of one loop sits on a side of another.
///
/// **This is a precondition and not a nicety, and it was found by a test rather
/// than by thinking.** Where a hole reaches out and touches the outline, the
/// outer side gets split across the hole's corners, and the stretch between
/// them is then claimed by *both* loops while the triangulation records it as
/// one boundary edge. The walk in [`walk_in_from_the_outside`] counts one
/// crossing where the boundary is really two thick, decides the hole is
/// material, and hands back a patch covering it. That is a wrong answer, which
/// is worse than either a refusal or a crash (`docs/decisions.md` §4.1).
///
/// **Stricter than it strictly has to be**, and the limit is worth stating
/// exactly. A corner touching another loop at a *single point* is harmless —
/// nothing is doubled there and the count stays right — and this refuses it
/// anyway, because telling the two apart means measuring how far the sharing
/// goes, and the fix for the case that matters is to count multiplicity rather
/// than to classify the touch. Nothing in this milestone needs a hole that
/// touches, so the honest move is to refuse the family and say so. The kernel's
/// own profile builder already refuses the same shape.
///
/// Exact throughout: [`orient2d`] settles whether a corner is on a side, and a
/// coordinate comparison settles whether it is between the ends. No tolerance
/// and nothing to tune.
///
/// # Errors
///
/// [`CannotTriangulate::LoopsTouch`], and
/// [`CannotTriangulate::CannotTellIfLoopsTouch`] if the cascade cannot settle
/// it — which for the exact `f64` corners a caller stores means a coordinate
/// product overflowed or fell subnormal.
fn refuse_loops_that_touch(
    corners: &[[f64; 2]],
    domain: &TrimmedDomain,
) -> Result<(), CannotTriangulate> {
    let sides = every_side(domain);
    let loop_of = loop_of_each_corner(domain);

    for &(from, to) in &sides {
        for (corner, at) in corners.iter().enumerate() {
            if loop_of[corner] == loop_of[from] {
                continue;
            }
            match orient2d(corners[from], corners[to], *at) {
                Decision::Pos | Decision::Neg => {}
                Decision::Zero => {
                    if lies_between(*at, corners[from], corners[to]) {
                        return Err(CannotTriangulate::LoopsTouch { corner, side: from });
                    }
                }
                Decision::Undecidable { enclosure } => {
                    return Err(CannotTriangulate::CannotTellIfLoopsTouch { corner, enclosure });
                }
                other => panic!("orient2d came back with a decision nothing here knows: {other:?}"),
            }
        }
    }
    Ok(())
}

/// Which loop each corner belongs to, by position in [`every_corner`]'s list.
fn loop_of_each_corner(domain: &TrimmedDomain) -> Vec<usize> {
    let mut which = Vec::new();
    for (index, one_loop) in core::iter::once(domain.outer())
        .chain(domain.holes())
        .enumerate()
    {
        which.extend(core::iter::repeat_n(index, one_loop.corners().len()));
    }
    which
}

/// Whether `point`, already known to be on the line through `from` and `to`,
/// lies within the segment between them.
///
/// Comparisons only, so exact.
fn lies_between(point: [f64; 2], from: [f64; 2], to: [f64; 2]) -> bool {
    (0..2).all(|axis| {
        point[axis] >= from[axis].min(to[axis]) && point[axis] <= from[axis].max(to[axis])
    })
}

/// Every corner of the domain, outer loop first and then each hole.
fn every_corner(domain: &TrimmedDomain) -> Vec<[f64; 2]> {
    let mut corners = domain.outer().corners().to_vec();
    for hole in domain.holes() {
        corners.extend_from_slice(hole.corners());
    }
    corners
}

/// Every side of every loop, as a pair of positions in [`every_corner`]'s list.
fn every_side(domain: &TrimmedDomain) -> Vec<(usize, usize)> {
    let mut sides = Vec::new();
    let mut first = 0;
    for one_loop in core::iter::once(domain.outer()).chain(domain.holes()) {
        let count = one_loop.corners().len();
        for step in 0..count {
            sides.push((first + step, first + (step + 1) % count));
        }
        first += count;
    }
    sides
}

/// Which faces are inside the domain, by how many boundaries they sit behind.
///
/// A breadth-first walk of the dual graph starting at the outer face, which is
/// outside by definition. Stepping across a boundary edge changes sides;
/// stepping across any other edge does not. Purely combinatorial: the geometry
/// has already had its say in where the constraints went.
fn walk_in_from_the_outside(mesh: &ConstrainedDelaunayTriangulation<Point2<f64>>) -> Vec<bool> {
    let mut inside = vec![false; mesh.num_all_faces()];
    let mut seen = vec![false; mesh.num_all_faces()];
    let mut queue = VecDeque::new();

    // The convex hull is the border with the outside, so the faces just within
    // it are one step in. Whether that step crossed a boundary is whether the
    // hull edge there is one.
    for hull_edge in mesh.convex_hull() {
        let Some(face) = hull_edge.rev().face().as_inner() else {
            continue;
        };
        let index = face.index();
        if seen[index] {
            continue;
        }
        seen[index] = true;
        inside[index] = hull_edge.is_constraint_edge();
        queue.push_back(face.fix());
    }

    while let Some(fixed) = queue.pop_front() {
        let face = mesh.face(fixed);
        let index = face.index();
        for edge in face.adjacent_edges() {
            let Some(neighbour) = edge.rev().face().as_inner() else {
                continue;
            };
            let next = neighbour.index();
            if seen[next] {
                continue;
            }
            seen[next] = true;
            inside[next] = inside[index] != edge.is_constraint_edge();
            queue.push_back(neighbour.fix());
        }
    }
    inside
}

/// Why a domain could not be cut into triangles.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, thiserror::Error)]
pub enum CannotTriangulate {
    /// Two sides of the boundary cross each other.
    ///
    /// A refusal about the input, not a failure here: a patch whose boundary
    /// crosses itself has no inside to fill.
    #[error("the boundary crosses itself, at the side leaving corner {at}")]
    BoundaryCrossesItself {
        /// Which corner the offending side leaves from.
        at: usize,
    },

    /// Two corners are the same point.
    #[error("the corner at {position} is in the same place as an earlier one")]
    CornersCoincide {
        /// Which corner repeats an earlier one.
        position: usize,
    },

    /// A corner of one loop sits on a side of another.
    ///
    /// See [`refuse_loops_that_touch`] for why this is refused, and for the
    /// part of the family that is refused although it would in fact be fine.
    #[error("the corner at {corner} sits on the side leaving corner {side}, in another loop")]
    LoopsTouch {
        /// The corner sitting on someone else's side.
        corner: usize,
        /// Which corner that side leaves from.
        side: usize,
    },

    /// Nothing could prove whether two loops touch.
    ///
    /// For the exact `f64` corners a caller stores this needs a coordinate
    /// product to overflow or fall subnormal. `Undecidable` at the level of a
    /// whole operation is a failure, not a result (`docs/decisions.md` §4.2).
    #[error("nothing can prove whether the corner at {corner} is on another loop's side")]
    CannotTellIfLoopsTouch {
        /// Which corner could not be settled.
        corner: usize,
        /// How wide the enclosure that straddled zero was.
        enclosure: f64,
    },

    /// A scaled coordinate is outside what the triangulation will take.
    ///
    /// The range is `2^-142` to `2^201`, which is narrower than `f64`'s at both
    /// ends and narrower than this kernel's own limits. Note that it is the
    /// **scaled** coordinate that has to fit: an angle round a pipe of radius
    /// `1e60` is a small number that becomes a large one.
    ///
    /// A refusal and never a wrong answer, which is the right way round.
    #[error("the corner at {position} is outside the range a triangulation takes: {why:?}")]
    CoordinateOutOfRange {
        /// Which corner.
        position: usize,
        /// What the triangulation said about it.
        why: InsertionError,
    },
}
