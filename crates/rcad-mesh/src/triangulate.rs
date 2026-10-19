//! Cutting a trimmed patch into triangles.

use std::collections::{HashMap, VecDeque};

use spade::handles::{FixedDirectedEdgeHandle, FixedFaceHandle, FixedVertexHandle, InnerTag};
use spade::{ConstrainedDelaunayTriangulation, InsertionError, Point2, Triangulation};

use rcad_geom::{Cone, Deflection, Point, Surface, Torus};
use rcad_predicates::{Approximation, Cause, Decision, orient2d};

use crate::domain::{Loop, TrimmedDomain};

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
/// loop first, then each hole. Anything after those is a point the refinement
/// put **strictly inside** the patch, so an index into the input stays an index
/// into the input and no loop ever grows a corner it did not arrive with
/// (`docs/decisions.md` §25.1).
#[derive(Clone, Debug, PartialEq)]
pub struct Patch {
    corners: Vec<[f64; 2]>,
    triangles: Vec<[u32; 3]>,
    deviation: Approximation,
}

impl Patch {
    /// The corners: the domain's own first, then whatever the refinement added.
    #[must_use]
    pub fn corners(&self) -> &[[f64; 2]] {
        &self.corners
    }

    /// The triangles, each naming three corners by index.
    #[must_use]
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }

    /// How far any point of any triangle can be from the surface.
    ///
    /// A **length**, in the model's own unit, and one number for the whole
    /// patch rather than one per triangle (`docs/decisions.md` §25.3). Always
    /// at or under the [`Deflection`] the domain asked for, because
    /// [`triangulate`] refuses rather than hand back a patch that missed it —
    /// usually well under, which is why the number is worth carrying at all.
    ///
    /// Exactly zero for a plane. A flat triangle standing in for a flat surface
    /// stands in for nothing.
    ///
    /// [`Deflection`]: rcad_geom::Deflection
    #[must_use]
    pub fn deviation(&self) -> Approximation {
        self.deviation
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
/// # What is refined, and what is not
///
/// The loops that arrive are **final**. Points go strictly inside the patch and
/// never onto a boundary, because this crate is shown one face at a time and
/// cannot see that a boundary is shared; two faces refining the same edge in
/// their own `(u, v)` would land on different 3D points and not by a rounding
/// (`docs/decisions.md` §25.1).
///
/// So the boundary sets a floor. A side of a loop is a chord of the surface,
/// and if that one chord already strays further than the [`Deflection`] asks
/// for, no number of interior points will help and this refuses. Refining the
/// boundary is the caller's job, and it is the caller's job precisely because
/// it is the only one that can see both sides of it.
///
/// Inside that floor the loop is: measure every triangle with
/// [`how_far_a_flat_piece_strays`], and cut any that strays too far in half
/// across its longest side. **It terminates**, and the reason is a packing
/// argument rather than a hope: no point is placed on top of another one, and
/// only so many points that far apart fit in a patch of finite area.
/// [`refine_the_inside`] carries the argument in full.
///
/// [`Deflection`]: rcad_geom::Deflection
///
/// # A flat patch the caller asks to be cut by the order of its corners
///
/// Is cut that way when it can be ([`TrimmedDomain::cut_by_corner_order`],
/// [`cut_by_corner_order`]), and the Delaunay test does not choose its
/// triangles. Every refusal below is asked first all the same, so asking never
/// turns a refusal into a patch.
///
/// # Errors
///
/// About the loops: [`CannotTriangulate::BoundaryCrossesItself`] when two
/// boundary edges cross, [`CannotTriangulate::CornersCoincide`] when one loop
/// names the same point twice — two *different* loops naming it is a pinch and
/// is allowed, see [`refuse_loops_that_touch`] —
/// [`CannotTriangulate::LoopsTouch`] when a corner sits on a side of its own
/// loop or another between that side's two ends, and
/// [`CannotTriangulate::CoordinateOutOfRange`] for a scaled coordinate the
/// triangulation will not take — see that variant, the range is narrower than
/// `f64`'s, and one too small is taken as zero where that changes nothing.
///
/// About the deflection:
/// [`CannotTriangulate::BoundarySideStraysFurtherThanTheDeflection`],
/// [`CannotTriangulate::NoMeshCanGetThatCloseToTheSurface`],
/// [`CannotTriangulate::MoreTrianglesThanAPatchWillHold`] and
/// [`CannotTriangulate::RanOutOfRoomBeforeMeetingTheDeflection`]. Every one of
/// them is about the *request* — the loops are the caller's and so is the
/// number it asked them to be good to.
pub fn triangulate(domain: &TrimmedDomain) -> Result<Patch, CannotTriangulate> {
    let mut corners = every_corner(domain);
    let loop_of = loop_of_each_corner(domain);
    refuse_loops_that_touch(&corners, &loop_of, domain)?;
    let surface = domain.surface();
    let (across, along) = how_this_domain_is_scaled(surface, &corners);

    let mut mesh = ConstrainedDelaunayTriangulation::<Point2<f64>>::new();
    let mut corner_of: Vec<u32> = Vec::new();
    let mut handles = Vec::with_capacity(corners.len());
    let scaled = |corner: [f64; 2]| Point2::new(corner[0] * across, corner[1] * along);
    let flushed = the_first_corner_the_flush_moves(&corners, scaled);
    for (position, corner) in corners.iter().enumerate() {
        let handle = mesh
            .insert(spade::mitigate_underflow(scaled(*corner)))
            .map_err(|why| CannotTriangulate::CoordinateOutOfRange { position, why })?;
        // Two loops are allowed to meet at a point, so a repeat is only a
        // refusal when it is a loop repeating itself. A repeat across loops is
        // one vertex of the mesh wearing two of the domain's corner numbers,
        // and the first of the two is the one the triangles get told about.
        match handles.iter().position(|earlier| *earlier == handle) {
            Some(earlier) if scaled(corners[earlier]) != scaled(*corner) => {
                return Err(the_refusal_the_flush_replaced(flushed.expect(
                    "two corners in different places share a vertex only when the flush moved one",
                )));
            }
            Some(earlier) if loop_of[earlier] == loop_of[position] => {
                return Err(CannotTriangulate::CornersCoincide { position });
            }
            Some(_) => {}
            None => remember_which_corner(&mut corner_of, handle.index(), position),
        }
        handles.push(handle);
    }

    for (from, to) in every_side(domain) {
        // Asking first, because `add_constraint` panics on a crossing and this
        // crate takes its loops from whoever calls it. A crossing among flushed
        // corners may be one the flush made, so it says nothing true about the
        // loops that were asked for.
        if !mesh.can_add_constraint(handles[from], handles[to]) {
            return Err(match flushed {
                Some(position) => the_refusal_the_flush_replaced(position),
                None => CannotTriangulate::BoundaryCrossesItself { at: from },
            });
        }
        mesh.add_constraint(handles[from], handles[to]);
    }
    if flushed.is_none() {
        refuse_a_side_with_a_corner_on_it(&mesh, &handles, domain)?;
    }

    let deflection = domain.deflection().length();
    let slop = how_far_a_point_lands_off_its_surface(surface, &corners);
    let whole_patch = the_range_of_each_coordinate(&corners);
    let step = a_step_that_stays_within(surface, whole_patch, deflection, slop);
    if step <= 0.0 {
        return Err(CannotTriangulate::NoMeshCanGetThatCloseToTheSurface {
            deflection,
            closest: how_far_a_flat_piece_strays(surface, whole_patch, &[], slop),
        });
    }
    // Size first and the loops after, because how many triangles the deflection
    // asks for is a question about the deflection alone, and whether the loops
    // are fine enough is a question about both.
    refuse_a_patch_too_big_to_hold(domain, (across, along), step)?;
    for (from, to) in every_side(domain) {
        let side = [corners[from], corners[to]];
        let across = the_range_of_each_coordinate(&side);
        let strays = how_far_a_flat_piece_strays(surface, across, &side, slop);
        if strays > deflection {
            return Err(
                CannotTriangulate::BoundarySideStraysFurtherThanTheDeflection {
                    side: from,
                    strays,
                },
            );
        }
    }

    let corners_inside_one_edge = if matches!(surface, Surface::Plane(_)) {
        Vec::new()
    } else {
        corners_inside_one_edge(domain, &handles)
    };
    let deviation = refine_the_inside(
        &mut mesh,
        &mut corners,
        &mut corner_of,
        &HowFineToGo {
            surface,
            across,
            along,
            deflection,
            slop,
            step,
            corners_inside_one_edge: &corners_inside_one_edge,
        },
    )?;

    match flushed {
        Some(position) => {
            let unflushed = |vertex: FixedVertexHandle| {
                let at = scaled(corners[corner_of[vertex.index()] as usize]);
                [at.x, at.y]
            };
            if !the_flush_moved_nothing_the_patch_rests_on(&mesh, domain, &handles, unflushed) {
                return Err(the_refusal_the_flush_replaced(position));
            }
        }
        // **Only the refinement can have split a side here**, since
        // `refuse_a_side_with_a_corner_on_it` saw every side whole before it
        // ran. It promises not to: it puts a point only at the middle of a side
        // off the loops, which is strictly inside the patch, but worked out in
        // `f64` that middle is a rounding off. A point rounded onto a side
        // would split it here while the face across draws it whole, a crack
        // (`docs/decisions.md` §25.1), and a crash beats a crack.
        //
        // **A mesh with no triangle is let through with a side in pieces.** All
        // its corners are on one line, spade holds them as a line of edges, and
        // the side running past the middle corner is two of those. No triangle
        // comes back, so none draws a piece of that side, and `rcad-topo` draws
        // such a face as a fan of its own sides. Every side Q2221 found not one
        // edge after the refinement was of this kind, and already in pieces
        // before it: 292 sides, each of a flat patch of three corners with no
        // triangle, among 4499449 unflushed patches past the refinement in
        // every test of `rcad-mesh`, `rcad-topo` and `rcad`.
        None => {
            let in_pieces = a_side_the_mesh_holds_in_pieces(&mesh, domain, &handles);
            assert!(
                in_pieces.is_none() || mesh.num_inner_faces() == 0,
                "the refinement put a point on a side: the first side in pieces, by the \
                 corner it leaves, is {in_pieces:?}"
            );
        }
    }

    if let Some(triangles) = cut_by_corner_order(domain, &corners) {
        return Ok(Patch {
            corners,
            triangles,
            deviation: Approximation::new(deviation, Cause::FlatStandIn),
        });
    }

    let inside = walk_in_from_the_outside(&mesh);
    let mut triangles = Vec::new();
    for face in mesh.inner_faces() {
        if !inside[face.index()] {
            continue;
        }
        triangles.push(face.vertices().map(|vertex| corner_of[vertex.index()]));
    }

    Ok(Patch {
        corners,
        triangles,
        deviation: Approximation::new(deviation, Cause::FlatStandIn),
    })
}

/// Cut the band between two loops that each go once right round the first
/// coordinate, the outline one way and the hole the other, into one strip of
/// triangles, each standing on a side of one loop and reaching across to the
/// other.
///
/// **For a band the chart cannot draw**: one whose two loops it reads at one
/// value of the second coordinate, or within a rounding of one, so that
/// [`triangulate`] sees one loop on top of the other. A torus wall between a
/// bend's cap rim and a meridian circle a rounding round from it is one: both
/// circles go right round the tube at the same angle round the ring, read a
/// whole turn apart, and cut across they named one place twice and were refused
/// [`CannotTriangulate::CornersCoincide`] (Q1734). Saying that a band is such a
/// band is the caller's, which sees the loops as two curves where this sees
/// corners. Nothing here reads the second coordinate except to measure how far a
/// triangle strays.
///
/// **Which loop steps next is read off how far round each has got**, both
/// counted from the outline's corner 0 the way the outline goes: the step
/// whose far end comes first goes next, the outline's on a tie. Ordered by
/// their far ends, the steps of the two loops interleave so that a triangle
/// standing on one side of one loop reaches a corner of the other no further
/// round than that side's far end and no further back than its near end, or
/// lies within one side of the other loop. So no triangle is wider round the
/// first coordinate than the widest side either loop has, and each side was
/// walked by the caller to stay within the deflection.
///
/// The corners are the outline's and then the hole's, numbered as [`Patch`]
/// numbers them, and nothing is put inside: a band a rounding wide has no inside
/// to put a point in. The triangles use each side of either loop the way its
/// loop runs, and each rung across once each way ([`a_strip_round_a_ring`]).
/// They are not turned any way in the chart, since a band a rounding wide has
/// no turn there to go by: a caller drawing them on a solid takes them as they
/// are, the way its wires run.
///
/// # Errors
///
/// [`CannotTriangulate::BoundaryCrossesItself`] when either loop does not go
/// once round, a step at a time, the outline either way and the hole the
/// other: `at` is the corner the first side that does not go that way leaves
/// from, which is folded back over the sides before it, or the loop's corner 0
/// when every side goes the right way and the loop goes round more than once.
///
/// [`CannotTriangulate::BoundarySideStraysFurtherThanTheDeflection`] when a side
/// of either loop strays further than `deflection`, or the triangle standing on
/// it does.
pub fn zip_up_a_band(
    surface: &Surface,
    outline: &Loop,
    hole: &Loop,
    deflection: Deflection,
) -> Result<Patch, CannotTriangulate> {
    let (outline, hole) = (outline.corners(), hole.corners());
    let (out, back) = (outline.len(), hole.len());
    let way = outline
        .iter()
        .enumerate()
        .map(|(at, corner)| within_half_a_turn(outline[(at + 1) % out][0] - corner[0]))
        .sum::<f64>()
        .signum();
    let round_the_outline = how_far_round_each_corner_is(outline, way)
        .map_err(|at| CannotTriangulate::BoundaryCrossesItself { at })?;

    let across = the_corner_of_the_hole_nearest_round(outline[0], hole);
    let hole_backwards: Vec<[f64; 2]> = (0..back)
        .map(|step| hole[(across + back - step) % back])
        .collect();
    let from_the_outlines_first = way * within_half_a_turn(hole[across][0] - outline[0][0]);
    let round_the_hole: Vec<f64> = how_far_round_each_corner_is(&hole_backwards, way)
        .map_err(|step| CannotTriangulate::BoundaryCrossesItself {
            at: out + (across + back - step % back) % back,
        })?
        .into_iter()
        .map(|round| from_the_outlines_first + round)
        .collect();

    let triangles = a_strip_round_a_ring(out, back, across, |one, other| {
        round_the_outline[one + 1] <= round_the_hole[other + 1]
    });

    let corners: Vec<[f64; 2]> = outline.iter().chain(hole).copied().collect();
    let slop = how_far_a_point_lands_off_its_surface(surface, &corners);
    let strays = |piece: &[[f64; 2]]| {
        how_far_a_flat_piece_strays(surface, the_range_of_each_coordinate(piece), piece, slop)
    };
    let mut worst = 0.0_f64;
    for triangle in &triangles {
        let [first, second, third] = triangle.map(|corner| corners[corner as usize]);
        let near_the_first = |corner: [f64; 2]| {
            [
                first[0] + within_half_a_turn(corner[0] - first[0]),
                corner[1],
            ]
        };
        let piece = [first, near_the_first(second), near_the_first(third)];
        let (side, its_ends) = if (triangle[1] as usize) < out {
            (triangle[0], [piece[0], piece[1]])
        } else {
            (triangle[1], [piece[1], piece[2]])
        };
        for strays in [strays(&its_ends), strays(&piece)] {
            if strays > deflection.length() {
                return Err(
                    CannotTriangulate::BoundarySideStraysFurtherThanTheDeflection {
                        side: side as usize,
                        strays,
                    },
                );
            }
            worst = worst.max(strays);
        }
    }

    Ok(Patch {
        corners,
        triangles,
        deviation: Approximation::new(worst, Cause::FlatStandIn),
    })
}

/// The triangles of a patch cut by the order of its corners, when the caller
/// asked for that ([`TrimmedDomain::cut_by_corner_order`]) and the patch is one
/// it can be done to: flat, with an outline that turns the same way at every
/// corner and goes round once, and either no hole
/// ([`every_other_corner_cut_off`]) or one hole that turns the other way at
/// every corner and goes round once ([`in_step_round_a_ring`]). `None` leaves
/// the patch to the Delaunay triangulation.
///
/// `corners` is [`every_corner`]'s list, the outline's first and then the
/// hole's.
///
/// Turned anticlockwise when the outline runs clockwise, since that is how
/// every triangle [`triangulate`] hands back is turned.
fn cut_by_corner_order(domain: &TrimmedDomain, corners: &[[f64; 2]]) -> Option<Vec<[u32; 3]>> {
    if !domain.is_cut_by_corner_order() || !matches!(domain.surface(), Surface::Plane(_)) {
        return None;
    }
    let outline = domain.outer().corners();
    let anticlockwise = turns_one_way_once_round(outline)?;
    let mut triangles = match domain.holes() {
        [] => every_other_corner_cut_off(outline.len()),
        [hole] => {
            let hole = hole.corners();
            if turns_one_way_once_round(hole)? == anticlockwise {
                return None;
            }
            let across = the_corner_of_the_hole_across_from(outline[0], hole, anticlockwise)?;
            let triangles = in_step_round_a_ring(outline.len(), hole.len(), across);
            let every_one_turns_with_the_outline = triangles.iter().all(|triangle| {
                let [a, b, c] = triangle.map(|corner| corners[corner as usize]);
                which_way_it_turns(a, b, c) == Some(anticlockwise)
            });
            if !every_one_turns_with_the_outline {
                return None;
            }
            triangles
        }
        _ => return None,
    };
    if !anticlockwise {
        for triangle in &mut triangles {
            triangle.swap(1, 2);
        }
    }
    Some(triangles)
}

/// A convex outline of `count` corners cut into triangles that turn the way it
/// does.
///
/// **Every other corner is cut off, round after round.** The first round joins
/// each corner to the one after next, the next round does the same among the
/// corners left, and so on, so the triangles stay about as wide as they are
/// long where a fan from one corner would draw slivers. Done to every flat disc
/// with nobody asking, this made 68 of 1440 meshes of a ball cut through its
/// pole fold, and the fan 96, against the Delaunay cut's 38 then: a lid's
/// corners are joined by the ball's own triangles too (`docs/decisions.md`
/// §379.5). The Delaunay cut's are 2 since Q1027 ([`corners_inside_one_edge`]).
/// That is what the caller's asking is for.
///
/// **Why it needs an outline that turns one way, once round:** then the patch
/// is convex, and any three of its corners in order are a triangle inside it
/// that turns the way the outline does, so every round leaves a smaller
/// outline of the same kind. Both are settled exactly: the turns by
/// [`orient2d`], and going round once by the second coordinate climbing and
/// then falling once, since an outline that turns one way and goes round twice
/// climbs and falls twice. A turn of zero, or one that cannot be settled, is
/// not one way, and leaves the patch to the Delaunay triangulation.
fn every_other_corner_cut_off(count: usize) -> Vec<[u32; 3]> {
    let mut left: Vec<u32> = (0..count).map(a_corner_number).collect();
    let mut triangles = Vec::with_capacity(count - 2);
    while left.len() >= 3 {
        let mut kept = Vec::with_capacity(left.len() / 2 + 1);
        let mut at = 0;
        while at + 2 < left.len() {
            triangles.push([left[at], left[at + 1], left[at + 2]]);
            kept.push(left[at]);
            at += 2;
        }
        if at + 2 == left.len() {
            triangles.push([left[at], left[at + 1], left[0]]);
            kept.push(left[at]);
        } else if at + 1 == left.len() {
            kept.push(left[at]);
        }
        left = kept;
    }
    triangles
}

/// The band of triangles between an outline of `outline` corners and a hole
/// of `hole` corners inside it, numbered as [`every_corner`] numbers them, cut
/// by stepping round both loops together from the outline's corner 0 and the
/// hole's corner `across`: each triangle stands on one side of one loop and
/// reaches across to the corner the other loop has got to. They turn the way
/// the outline does when the hole runs round the other way.
///
/// **Which loop steps next is read from the corner numbers alone.** The
/// outline is walked the way it runs and the hole the other way, so that both
/// go round the same way; each loop's steps are taken as equal shares of a
/// turn, and the step whose middle comes first round goes next, the outline's
/// on a tie. That is a comparison of whole numbers, so exact. On two circles
/// cut into even steps, which is how a pipe's two rims come to its flat end,
/// each side of the outline reaches about the hole's corner across from its
/// middle, and each side of the hole the outline's. No coordinate chooses a
/// step, so a ring moved or turned is cut into the same triangles corner for
/// corner once it starts from the same corner, where the Delaunay test is tied
/// wherever a side of the outline and a side of the hole face each other
/// square: those four corners are on one circle (`docs/decisions.md` §379.5).
/// The start, `across`, is chosen from coordinates, and
/// [`the_corner_of_the_hole_across_from`] says where a rounding can move it.
///
/// **Why they are a triangulation of the ring when every one of them turns the
/// way the outline does**, which [`cut_by_corner_order`] settles with
/// [`orient2d`] before it takes them. [`a_strip_round_a_ring`] says why the
/// triangles' sides taken together are the outline once round its way and the
/// hole once round the other. That pair winds once round every point of the
/// ring and not at all round a point of the hole or outside, and a triangle
/// that turns the outline's way winds once round each point inside it and not
/// at all round any other. So every point of the ring is under exactly one
/// triangle, and no point outside the ring under any. No corner lies along
/// another triangle's side either: the triangles at a corner run round it in
/// order from one of its loop's sides to the other, and so already cover the
/// ring beside it once.
///
/// Any other ring turns some triangle the wrong way or none, and is left to the
/// Delaunay triangulation: a hole far off to one side, say, or two loops that
/// touch, where the rung between the two corners in one place has no length.
fn in_step_round_a_ring(outline: usize, hole: usize, across: usize) -> Vec<[u32; 3]> {
    a_strip_round_a_ring(outline, hole, across, |one, other| {
        comes_no_later_round(one, outline, other, hole)
    })
}

/// The triangles between an outline of `outline` corners and a hole of `hole`
/// corners, numbered as [`every_corner`] numbers them, stepping round both
/// loops together from the outline's corner 0 and the hole's corner `across`:
/// the outline the way it runs and the hole the other way, so that both go
/// round the same way. Each triangle stands on one side of one loop and
/// reaches across to the corner the other loop has got to.
/// `the_outline_steps_next(one, other)`, asked with the outline's `one` steps
/// and the hole's `other` taken and some of each left, says which loop steps.
///
/// **Whatever it says, each side of either loop is a side of exactly one
/// triangle, the way its loop runs, and each rung across is a side of two,
/// once each way.** A triangle's sides are one side of a loop and two rungs:
/// the rung it starts from, and the one it hands to the next triangle, and the
/// last hands back the first. So the triangles' sides taken together run every
/// rung once each way, and what is left is the outline once round its way and
/// the hole once round the other.
///
/// **No side joins two corners of one loop**, only a corner of the outline to
/// a corner of the hole, so a face across either rim draws none of them unless
/// it reaches the other rim too (§187.4).
fn a_strip_round_a_ring(
    outline: usize,
    hole: usize,
    across: usize,
    the_outline_steps_next: impl Fn(usize, usize) -> bool,
) -> Vec<[u32; 3]> {
    let on_the_outline = |step: usize| a_corner_number(step % outline);
    let on_the_hole = |step: usize| a_corner_number(outline + (across + hole - step % hole) % hole);
    let (mut round_the_outline, mut round_the_hole) = (0, 0);
    let mut triangles = Vec::with_capacity(outline + hole);
    while round_the_outline < outline || round_the_hole < hole {
        let the_outline_steps = round_the_hole == hole
            || (round_the_outline < outline
                && the_outline_steps_next(round_the_outline, round_the_hole));
        if the_outline_steps {
            triangles.push([
                on_the_outline(round_the_outline),
                on_the_outline(round_the_outline + 1),
                on_the_hole(round_the_hole),
            ]);
            round_the_outline += 1;
        } else {
            triangles.push([
                on_the_outline(round_the_outline),
                on_the_hole(round_the_hole + 1),
                on_the_hole(round_the_hole),
            ]);
            round_the_hole += 1;
        }
    }
    triangles
}

/// Whether the middle of step `one` of `ones` equal steps round comes no later
/// than the middle of step `other` of `others`: `(one + ½) / ones ≤ (other +
/// ½) / others`, multiplied out into whole numbers wide enough not to
/// overflow.
fn comes_no_later_round(one: usize, ones: usize, other: usize, others: usize) -> bool {
    let wide = |count: usize| u128::from(a_corner_number(count));
    (2 * wide(one) + 1) * wide(others) <= (2 * wide(other) + 1) * wide(ones)
}

/// How far round the first coordinate each of `corners` is from the first,
/// stepping the short way round from each corner to the next and on back to
/// the first, counted the way `way` says: `corners.len() + 1` numbers, from
/// `0` to about a whole turn.
///
/// `Err` with the corner a step leaves from when that step does not go `way`,
/// and with corner 0 when every step does and they go round more than once.
fn how_far_round_each_corner_is(corners: &[[f64; 2]], way: f64) -> Result<Vec<f64>, usize> {
    let count = corners.len();
    let mut so_far = Vec::with_capacity(count + 1);
    so_far.push(0.0);
    for into in 1..=count {
        let step = way * within_half_a_turn(corners[into % count][0] - corners[into - 1][0]);
        if step <= 0.0 {
            return Err(into - 1);
        }
        so_far.push(so_far[into - 1] + step);
    }
    if (so_far[count] / core::f64::consts::TAU).round() != 1.0 {
        return Err(0);
    }
    Ok(so_far)
}

/// The corner of `hole` nearest `first` round the first coordinate.
fn the_corner_of_the_hole_nearest_round(first: [f64; 2], hole: &[[f64; 2]]) -> usize {
    (0..hole.len())
        .min_by(|&one, &other| {
            let how_far = |at: usize| within_half_a_turn(hole[at][0] - first[0]).abs();
            how_far(one).total_cmp(&how_far(other))
        })
        .expect("a loop has corners")
}

/// `turn` moved by whole turns to within half a turn of nothing.
fn within_half_a_turn(turn: f64) -> f64 {
    let whole = core::f64::consts::TAU;
    turn - whole * (turn / whole).round()
}

/// The corner of `hole` that [`in_step_round_a_ring`] starts from beside the
/// outline's corner `first`, which turns the way `anticlockwise` says: the
/// middle of the run of the hole's sides that `first` sees from outside, or
/// the first corner of the middle side when the run is odd. `None` when
/// `first` sees no side from outside, or [`orient2d`] cannot settle which side
/// of one it is on.
///
/// **A side whose line `first` lies on is not seen**, since it is not seen
/// from outside. It used to give `None` and send the ring to the Delaunay
/// triangulation, so an octagon round an octagon with its corner exactly on
/// that line was cut by spade, and the same ring placed, a rounding off the
/// line, by corner order: 14 of its 16 triangles apart under each of the
/// twelve placements of `docs/decisions.md` §333.2 (Q1274). Any start is safe
/// to try, because [`cut_by_corner_order`] takes the cut only when every
/// triangle turns the outline's way.
///
/// **Read from signs alone, so a ring moved by a rounding starts from the same
/// corner unless `first` lies within a rounding of the line through a side at
/// one end of the run.** Even then it holds at the end where that side joining
/// or leaving the run leaves the middle where it was. At the other end the
/// start moves one corner, and the ring is cut into other triangles, a whole
/// cut of the ring each time: 15 of 36 placed copies of that octagon ring,
/// cornered on the other end's line or a rounding to either side of it
/// (`a_ring_whose_corner_is_a_rounding_off_a_hole_sides_line_starts_where_the_corner_sees`
/// in `cutting_a_patch_into_triangles.rs`).
///
/// **No start read from the corners handed in holds for every ring.** Turned
/// a whole turn against the outline, the hole's start has to walk once round
/// it, or some triangle turns over — a start fixed at the hole's corner 0 does
/// on a pipe's ring (§379.5, Q1109) — so the start changes somewhere as the
/// hole turns, and a ring there and the same ring placed are a rounding apart.
/// Of rules that read only which sides `first` sees, none holds at both ends
/// of the run either: one that kept its start whichever end's side joined or
/// left would keep it as the run walks round, and so never move. What would
/// hold is a start read where the solid is stored, which only the caller has.
fn the_corner_of_the_hole_across_from(
    first: [f64; 2],
    hole: &[[f64; 2]],
    anticlockwise: bool,
) -> Option<usize> {
    let count = hole.len();
    let mut seen = Vec::with_capacity(count);
    for at in 0..count {
        let sees_it_from_outside = match orient2d(hole[at], hole[(at + 1) % count], first) {
            Decision::Pos => anticlockwise,
            Decision::Neg => !anticlockwise,
            Decision::Zero => false,
            Decision::Undecidable { .. } => return None,
            other => panic!("orient2d came back with a decision nothing here knows: {other:?}"),
        };
        seen.push(sees_it_from_outside);
    }
    let start = (0..count).find(|&at| seen[at] && !seen[(at + count - 1) % count])?;
    let run = seen.iter().filter(|&&sees| sees).count();
    Some((start + run / 2) % count)
}

/// The number a triangle names the corner at `position` by.
fn a_corner_number(position: usize) -> u32 {
    u32::try_from(position).expect("a loop of fewer than four billion corners")
}

/// Whether an outline turns anticlockwise at every corner (`Some(true)`) or
/// clockwise at every corner (`Some(false)`) and goes round once; `None` for
/// any other outline.
fn turns_one_way_once_round(corners: &[[f64; 2]]) -> Option<bool> {
    let count = corners.len();
    let mut way = None;
    for at in 0..count {
        let anticlockwise = which_way_it_turns(
            corners[at],
            corners[(at + 1) % count],
            corners[(at + 2) % count],
        )?;
        if *way.get_or_insert(anticlockwise) != anticlockwise {
            return None;
        }
    }
    let climbs: Vec<bool> = (0..count)
        .map(|at| (corners[at][1], corners[(at + 1) % count][1]))
        .filter(|(from, to)| to != from)
        .map(|(from, to)| to > from)
        .collect();
    let turns_from_climbing_to_falling = (0..climbs.len())
        .filter(|&at| climbs[at] != climbs[(at + 1) % climbs.len()])
        .count();
    if turns_from_climbing_to_falling != 2 {
        return None;
    }
    way
}

/// Whether `a`, `b`, `c` turn anticlockwise (`Some(true)`) or clockwise
/// (`Some(false)`), by [`orient2d`]; `None` for no turn, or one it cannot
/// settle.
///
/// # Panics
///
/// On a [`Decision`] this file has not been taught.
fn which_way_it_turns(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Option<bool> {
    match orient2d(a, b, c) {
        Decision::Pos => Some(true),
        Decision::Neg => Some(false),
        Decision::Zero | Decision::Undecidable { .. } => None,
        other => panic!("orient2d came back with a decision nothing here knows: {other:?}"),
    }
}

/// The first corner with a scaled coordinate that is not zero and is under
/// spade's floor of `2⁻¹⁴²`, which [`spade::mitigate_underflow`] sets to zero.
///
/// `None` for almost every patch, and then spade is handed exactly the numbers
/// it was handed before the flush existed, so nothing about the answer can have
/// changed. Compared bit for bit, so a coordinate that is not a number is not
/// counted as moved: spade refuses that one on its own.
fn the_first_corner_the_flush_moves(
    corners: &[[f64; 2]],
    scaled: impl Fn([f64; 2]) -> Point2<f64>,
) -> Option<usize> {
    corners.iter().position(|corner| {
        let before = scaled(*corner);
        let after = spade::mitigate_underflow(before);
        before.x.to_bits() != after.x.to_bits() || before.y.to_bits() != after.y.to_bits()
    })
}

/// What [`triangulate`] said about a patch with a coordinate under `2⁻¹⁴²`
/// before it flushed any: spade refused the first such corner as too small.
///
/// So a flushed patch that cannot be shown to come out the same gets the
/// refusal it always got, and the flush only ever turns a refusal into a mesh
/// (`docs/decisions.md` §246.5).
fn the_refusal_the_flush_replaced(position: usize) -> CannotTriangulate {
    CannotTriangulate::CoordinateOutOfRange {
        position,
        why: InsertionError::TooSmall,
    }
}

/// Whether the triangles spade drew with some corners flushed to zero are
/// still a triangulation of the patch **as it was asked for**, with every
/// corner where the caller put it.
///
/// A flushed corner moves by less than `2⁻¹⁴²` in the scaled domain, and a
/// move that small can still change the answer. Each of these is a test in
/// this file that fails without its check:
///
/// - **Every side of every loop is one side of a triangle.** A corner flushed
///   onto the line of a side splits it, and a side split here is drawn whole by
///   the face on the other side of it, which is a crack between the two
///   (`a_corner_flushed_onto_another_loops_side_is_refused`).
/// - **Every triangle still turns anticlockwise**, read with the corners where
///   they really are. A corner flushed across the line of a thin triangle turns
///   it over (`a_triangle_the_flush_turns_over_is_refused`), and so does a thin
///   hole turned inside out, whose own triangle is not kept but is still what
///   stops the kept ones overlapping
///   (`a_hole_the_flush_turns_inside_out_is_refused`).
/// - **The hull's outline does not cross or touch itself**, read the same way.
///   It need not stay convex: a corner flushed onto a straight side of the
///   hull dents it by less than `2⁻¹⁴²` read where it really is, and the
///   triangles still tile the patch
///   (`a_corner_flushed_onto_a_straight_side_of_the_hull_is_meshed`).
///
/// Why those three are enough: with every triangle anticlockwise, how many of
/// them cover a point is the number of times the hull's outline winds round
/// it, and an outline that does not cross itself winds round a point once or
/// not at all. So the triangles tile the inside of that outline without
/// overlapping, as they did before the flush. The loops are sides of that
/// tiling, so which triangles are inside them is the same count of crossings
/// it always was. **The third check is there because the argument needs it.**
/// No test here is built to fail it and pass the other two; that it cannot
/// happen is not claimed.
///
/// Two corners in different places flushed onto one vertex are refused before
/// this runs, where [`triangulate`] inserts them.
///
/// Only what the flush moved is looked at: a triangle, or a side of the hull,
/// none of whose corners moved, is exactly what spade drew, and spade's own
/// predicates are exact. A turn that cannot be settled counts as a failure,
/// since then nothing shows the patch unchanged.
///
/// # Panics
///
/// On a [`Decision`] this file has not been taught.
fn the_flush_moved_nothing_the_patch_rests_on(
    mesh: &ConstrainedDelaunayTriangulation<Point2<f64>>,
    domain: &TrimmedDomain,
    handles: &[FixedVertexHandle],
    unflushed: impl Fn(FixedVertexHandle) -> [f64; 2],
) -> bool {
    if a_side_the_mesh_holds_in_pieces(mesh, domain, handles).is_some() {
        return false;
    }

    let moved = |vertex: FixedVertexHandle| {
        let at = mesh.vertex(vertex).position();
        [at.x, at.y] != unflushed(vertex)
    };

    for face in mesh.inner_faces() {
        let corners = face.vertices().map(|vertex| vertex.fix());
        if !corners.iter().any(|&vertex| moved(vertex)) {
            continue;
        }
        let [a, b, c] = corners.map(&unflushed);
        match orient2d(a, b, c) {
            Decision::Pos => {}
            Decision::Neg | Decision::Zero | Decision::Undecidable { .. } => return false,
            other => panic!("orient2d came back with a decision nothing here knows: {other:?}"),
        }
    }

    let hull: Vec<([[f64; 2]; 2], bool)> = mesh
        .convex_hull()
        .map(|outward| {
            let ends = [outward.from().fix(), outward.to().fix()];
            (ends.map(&unflushed), ends.iter().any(|&end| moved(end)))
        })
        .collect();
    hull.iter().enumerate().all(|(which, &(side, side_moved))| {
        !side_moved
            || hull.iter().enumerate().all(|(other, &(beside, _))| {
                other == which || !two_sides_of_an_outline_meet(side, beside)
            })
    })
}

/// The corner the first side of the domain leaves from that is not one edge of
/// `mesh`, or `None` when every side is.
///
/// A side with a point of the mesh on it is two edges or more, since spade
/// splits a side at any point it is given there.
fn a_side_the_mesh_holds_in_pieces(
    mesh: &ConstrainedDelaunayTriangulation<Point2<f64>>,
    domain: &TrimmedDomain,
    handles: &[FixedVertexHandle],
) -> Option<usize> {
    every_side(domain).into_iter().find_map(|(from, to)| {
        mesh.get_edge_from_neighbors(handles[from], handles[to])
            .is_none()
            .then_some(from)
    })
}

/// Whether two sides of the hull's outline share a point they should not.
///
/// Two sides that follow one another share their one corner, and that is all
/// they may share: they meet wrongly only when the second turns straight back
/// along the first. Any other two may share nothing at all. Exact, from
/// [`orient2d`] and comparisons; a turn it cannot settle counts as meeting,
/// since then nothing shows they do not.
///
/// # Panics
///
/// On a [`Decision`] this file has not been taught.
fn two_sides_of_an_outline_meet(one: [[f64; 2]; 2], other: [[f64; 2]; 2]) -> bool {
    let turn = |a, b, c| match orient2d(a, b, c) {
        Decision::Pos => Some(1),
        Decision::Neg => Some(-1),
        Decision::Zero => Some(0),
        Decision::Undecidable { .. } => None,
        other => panic!("orient2d came back with a decision nothing here knows: {other:?}"),
    };
    if one[1] == other[0] || one[0] == other[1] {
        let (before, shared, after) = if one[1] == other[0] {
            (one[0], one[1], other[1])
        } else {
            (other[0], other[1], one[1])
        };
        return match turn(before, shared, after) {
            Some(0) => !lies_between(shared, before, after),
            Some(_) => false,
            None => true,
        };
    }
    let (Some(one_from), Some(one_to), Some(other_from), Some(other_to)) = (
        turn(other[0], other[1], one[0]),
        turn(other[0], other[1], one[1]),
        turn(one[0], one[1], other[0]),
        turn(one[0], one[1], other[1]),
    ) else {
        return true;
    };
    if one_from * one_to < 0 && other_from * other_to < 0 {
        return true;
    }
    (one_from == 0 && lies_between(one[0], other[0], other[1]))
        || (one_to == 0 && lies_between(one[1], other[0], other[1]))
        || (other_from == 0 && lies_between(other[0], one[0], one[1]))
        || (other_to == 0 && lies_between(other[1], one[0], one[1]))
}

/// Refuse a domain where one loop lands on the *inside* of another loop's side.
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
/// **Two loops meeting at a shared corner are let through, and nothing else
/// is.** That is narrower than it first looks, and the line is drawn where it
/// is for a reason that is not about this crate's own arithmetic:
///
/// - A corner sitting *between* a side's two ends is refused even though the
///   patch would come out right. The triangulation splits that side at it, so a
///   boundary side the caller handed over whole comes back as two edges — and
///   refining a boundary is the caller's job precisely because this crate is
///   shown one face at a time and cannot see the face on the other side of it
///   (`docs/decisions.md` §25.1). A silent split here is a crack there.
/// - A corner sitting *at* a side's end splits nothing. Both loops already name
///   that point, every constrained edge still belongs to exactly one loop, and
///   the crossing count stays right, so the patch comes back with its hole in
///   it. [`triangulate`] lets the repeated point through for the same reason.
/// - Two sides of different loops with the *same two ends* are refused by
///   [`refuse_loops_that_share_a_whole_side`], which is the one way a stretch
///   can be doubled without any corner landing inside anything.
/// - A corner of a loop on a side of **its own** loop is refused once the
///   corners are in the triangulation, by [`refuse_a_side_with_a_corner_on_it`].
///
/// The shared corner used to be refused with the rest, on the stated grounds
/// that nothing needed it. Something does: a round pin drawn tangent to a flat
/// of the body it stands in meets that flat at a vertex both loops name, and
/// that is how a whole family of the STEP corpus is drawn.
///
/// Exact throughout: [`orient2d`] settles whether a corner is on a side, and
/// coordinate comparisons settle whether it is between the ends and whether it
/// is one of them. No tolerance and nothing to tune.
///
/// # Errors
///
/// [`CannotTriangulate::LoopsTouch`], and
/// [`CannotTriangulate::CannotTellIfLoopsTouch`] if the cascade cannot settle
/// it — which for the exact `f64` corners a caller stores means a coordinate
/// product overflowed or fell subnormal.
fn refuse_loops_that_touch(
    corners: &[[f64; 2]],
    loop_of: &[usize],
    domain: &TrimmedDomain,
) -> Result<(), CannotTriangulate> {
    let sides = every_side(domain);

    for &(from, to) in &sides {
        for (corner, at) in corners.iter().enumerate() {
            if loop_of[corner] == loop_of[from] || *at == corners[from] || *at == corners[to] {
                continue;
            }
            match orient2d(corners[from], corners[to], *at) {
                Decision::Pos | Decision::Neg => {}
                Decision::Zero => {
                    if lies_between(*at, corners[from], corners[to]) {
                        return Err(CannotTriangulate::LoopsTouch {
                            corner,
                            side: from,
                            at: domain.surface().point_at(at[0], at[1]),
                        });
                    }
                }
                Decision::Undecidable { enclosure } => {
                    return Err(CannotTriangulate::CannotTellIfLoopsTouch {
                        corner,
                        enclosure,
                        at: domain.surface().point_at(at[0], at[1]),
                    });
                }
                other => panic!("orient2d came back with a decision nothing here knows: {other:?}"),
            }
        }
    }
    refuse_loops_that_share_a_whole_side(corners, loop_of, &sides, domain)
}

/// Refuse two loops that name the same side, end for end.
///
/// The one doubled stretch [`refuse_loops_that_touch`] cannot see, and the
/// reason it cannot is that neither loop puts a corner anywhere but at the
/// other's corners: an overlap longer than a point has an end, that end is a
/// corner of one of the two sides, and a corner inside the other side is
/// already refused — so what is left over is two sides with the same two ends.
///
/// Comparisons of the corners' own coordinates, so exact.
///
/// # Errors
///
/// [`CannotTriangulate::LoopsTouch`].
fn refuse_loops_that_share_a_whole_side(
    corners: &[[f64; 2]],
    loop_of: &[usize],
    sides: &[(usize, usize)],
    domain: &TrimmedDomain,
) -> Result<(), CannotTriangulate> {
    for (which, &(from, to)) in sides.iter().enumerate() {
        for &(other_from, other_to) in &sides[which + 1..] {
            if loop_of[from] == loop_of[other_from] {
                continue;
            }
            let ends = [corners[from], corners[to]];
            let other_ends = [corners[other_from], corners[other_to]];
            if ends != other_ends && ends != [other_ends[1], other_ends[0]] {
                continue;
            }
            let at = corners[other_from];
            return Err(CannotTriangulate::LoopsTouch {
                corner: other_from,
                side: from,
                at: domain.surface().point_at(at[0], at[1]),
            });
        }
    }
    Ok(())
}

/// Refuse a domain where a side did not go into the triangulation whole.
///
/// The triangulation splits a side at any corner lying on it between its two
/// ends, and the face across that side draws it whole: a crack, for the reason
/// [`refuse_loops_that_touch`] gives. That check asked about a corner of
/// another loop; this asks about every corner, **its own loop's too**. A loop
/// that runs down a line and back up the same line rests its own corners on
/// its own side, and no triangle can draw those sides, since its three corners
/// would be on one line. Q1307 saw one come back as a fan that covered the
/// loop's area and left the side it ran back up in no triangle at all.
///
/// **A domain with every corner on one line is let through**, and comes back
/// with no triangles. It encloses nothing, so no triangles is all of it, and no
/// triangle draws a piece of a side, though the side running past a corner on
/// it stays two edges of the mesh to the end. `rcad-topo`'s
/// `a_block_edge_grazing_a_ball.rs` cuts slivers `2⁻⁷⁸` wide, a flat face of
/// one comes here as three corners on one line, and the solid is refused
/// later, on its curved face, as it was before this check.
///
/// Asked of the triangulation and not of the corners as they were handed
/// over, because the triangulation takes them scaled and the triangles follow
/// its answer. Exact: spade's predicates are, and so is [`orient2d`] on the
/// same numbers.
///
/// A patch with a corner [`spade::mitigate_underflow`] moved is left to
/// [`the_flush_moved_nothing_the_patch_rests_on`], which asks every side the
/// same and refuses the patch the way it was refused before the flush.
///
/// # Errors
///
/// [`CannotTriangulate::LoopsTouch`], and
/// [`CannotTriangulate::CannotTellIfLoopsTouch`] when no corner can be shown to
/// be on the side and [`orient2d`] could not settle one of them.
///
/// # Panics
///
/// When spade split a side and no corner is on it, which is spade breaking its
/// own rule; and on a [`Decision`] this file has not been taught.
fn refuse_a_side_with_a_corner_on_it(
    mesh: &ConstrainedDelaunayTriangulation<Point2<f64>>,
    handles: &[FixedVertexHandle],
    domain: &TrimmedDomain,
) -> Result<(), CannotTriangulate> {
    if mesh.num_inner_faces() == 0 {
        return Ok(());
    }
    let scaled = |position: usize| {
        let at = mesh.vertex(handles[position]).position();
        [at.x, at.y]
    };
    for (from, to) in every_side(domain) {
        if mesh
            .get_edge_from_neighbors(handles[from], handles[to])
            .is_some()
        {
            continue;
        }
        let place_of = |corner: usize| {
            let at = every_corner(domain)[corner];
            domain.surface().point_at(at[0], at[1])
        };
        let mut unsettled = None;
        for corner in 0..handles.len() {
            if handles[corner] == handles[from] || handles[corner] == handles[to] {
                continue;
            }
            match orient2d(scaled(from), scaled(to), scaled(corner)) {
                Decision::Pos | Decision::Neg => {}
                Decision::Zero => {
                    if lies_between(scaled(corner), scaled(from), scaled(to)) {
                        return Err(CannotTriangulate::LoopsTouch {
                            corner,
                            side: from,
                            at: place_of(corner),
                        });
                    }
                }
                Decision::Undecidable { enclosure } => {
                    unsettled.get_or_insert((corner, enclosure));
                }
                other => panic!("orient2d came back with a decision nothing here knows: {other:?}"),
            }
        }
        let (corner, enclosure) = unsettled.unwrap_or_else(|| {
            panic!("the triangulation split the side leaving corner {from} where no corner is")
        });
        return Err(CannotTriangulate::CannotTellIfLoopsTouch {
            corner,
            enclosure,
            at: place_of(corner),
        });
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

/// Note that the mesh vertex numbered `vertex` is the corner at `position`.
///
/// The mesh numbers its vertices as it takes them and never lets one go, so
/// this list only ever grows and an entry never moves.
fn remember_which_corner(corner_of: &mut Vec<u32>, vertex: usize, position: usize) {
    let position = u32::try_from(position)
        .expect("a domain with more than four billion corners is not a domain");
    if corner_of.len() <= vertex {
        corner_of.resize(vertex + 1, u32::MAX);
    }
    corner_of[vertex] = position;
}

/// How far a point from [`Surface::point_at`] can land off the surface it names.
///
/// Not the same thing as [`how_far_a_flat_piece_strays`] and not covered by it.
/// A chord bound is about where the *flat* piece is; this is about the corners
/// of it being a little bit off the surface before the piece is even drawn. The
/// evaluator's sine and cosine each land within an ulp, and the point is then
/// added to the surface's own origin, which rounds against the size of *that*
/// rather than against the radius: a ball of radius one a kilometre out has its
/// points known to a rounding of the kilometre.
///
/// A plane gets zero, and the zero is exact rather than rounded down. Nothing
/// is evaluated for a plane — a flat piece of a flat surface never strays, so
/// no interior point is ever added to one and the only points in the mesh are
/// the caller's own corners.
///
/// # Panics
///
/// On a surface this crate has not been taught. [`Surface`] grows each
/// milestone and a new arm arriving here unhandled is a broken invariant of the
/// workspace rather than bad input from a caller.
fn how_far_a_point_lands_off_its_surface(surface: &Surface, corners: &[[f64; 2]]) -> f64 {
    match surface {
        Surface::Plane(_) => 0.0,
        // Every point of a ball is within its radius of its centre.
        Surface::Sphere(ball) => {
            HOW_FAR_THE_EVALUATOR_ROUNDS
                * (ball.centre().largest_coordinate_magnitude() + ball.radius())
        }
        // A pipe is endless, so how big its points are depends on how far along
        // it the patch reaches — and the patch stays inside the span of its own
        // corners, because every point put inside is between two of them.
        Surface::Cylinder(pipe) => {
            let reach = corners
                .iter()
                .map(|corner| corner[1].abs())
                .fold(0.0, f64::max);
            HOW_FAR_THE_EVALUATOR_ROUNDS
                * (pipe.axis().origin().largest_coordinate_magnitude() + pipe.radius() + reach)
        }
        // A cone is endless the same way, and how far out its points are is
        // about how far along it the patch reaches, times the slope.
        Surface::Cone(cone) => {
            let [lowest, highest] = the_range_of_each_coordinate(corners)[1];
            let reach = lowest.abs().max(highest.abs());
            HOW_FAR_THE_EVALUATOR_ROUNDS
                * (cone.apex().largest_coordinate_magnitude() + cone.radius_at(reach) + reach)
        }
        // A torus is bounded, so this needs no corners at all: every point of it
        // is within `ring + tube` of its centre, whichever way a patch on it
        // wraps.
        Surface::Torus(ring) => {
            HOW_FAR_THE_EVALUATOR_ROUNDS
                * (ring.centre().largest_coordinate_magnitude()
                    + ring.ring_radius()
                    + ring.tube_radius())
        }
        other => panic!("this crate cannot say how far a point of {other:?} lands off it"),
    }
}

/// How far along its own axis a set of corners in a surface's own two
/// coordinates reaches, lowest first — **signed**, and the sign is load-bearing.
///
/// Meaningless for a plane, a ball or a pipe, and read only by the cone arms
/// below, which is why it is a pair of plain numbers rather than a method on
/// [`Surface`]: what a second coordinate *means* is the surface's own business,
/// and only a cone's decides how big the surface is where the patch sits.
///
/// The sign is what says whether the patch is on **one nappe**, which every
/// bound below assumes: the derivation on [`how_far_a_flat_piece_strays`] puts
/// `t·p_∥ = Σλᵢrᵢ`, and that holds only while every corner's `p_∥` has the same
/// sign. A patch spanning both nappes is pinched at the apex in its own interior
/// and is not a surface patch at all; nothing in the kernel builds one, and
/// `SolidBuilder` is public so a caller can describe one — which is why it is
/// caught here rather than assumed away.
fn the_range_of_each_coordinate(corners: &[[f64; 2]]) -> [[f64; 2]; 2] {
    core::array::from_fn(|which| {
        let lowest = corners
            .iter()
            .map(|corner| corner[which])
            .fold(f64::INFINITY, f64::min);
        let highest = corners
            .iter()
            .map(|corner| corner[which])
            .fold(f64::NEG_INFINITY, f64::max);
        [lowest, highest]
    })
}

/// How far from the axis a torus reaches over a patch — the widest circle round
/// the ring that any point of it sits on.
///
/// `R + r·cos v` at the largest `cos v` the patch's **first** coordinate covers,
/// which is one where the patch straddles the outer equator and an end of the
/// range where it does not. Never below `R − r`, so it is always a size.
///
/// **The chord round the ring is measured here and divided by here**, which is
/// the same discipline `how_wide_a_cone_is_over` follows and for the same
/// reason: a chord read on a wider circle than the one it is charged against
/// leaves a boundary side that the walker refined by the sagitta of the circle
/// it really lies on failing a bound written for a circle it does not.
fn how_wide_a_torus_is_over(ring: &Torus, [lowest, highest]: [f64; 2]) -> f64 {
    let whole_turn = core::f64::consts::TAU;
    let straddles_the_outer_equator =
        highest - lowest >= whole_turn || (lowest / whole_turn).ceil() * whole_turn <= highest;
    let widest = if straddles_the_outer_equator {
        1.0
    } else {
        lowest.cos().max(highest.cos())
    };
    ring.ring_radius() + ring.tube_radius() * widest
}

/// How far out a cone reaches over a patch, and [`f64::INFINITY`] when the patch
/// spans both nappes — see [`how_far_up_its_axis_this_reaches`].
fn how_wide_a_cone_is_over(cone: &Cone, [lowest, highest]: [f64; 2]) -> f64 {
    if lowest < 0.0 && highest > 0.0 {
        return f64::INFINITY;
    }
    cone.radius_at(lowest.abs().max(highest.abs()))
}

/// How the domain is stretched before the triangulation runs.
///
/// [`Surface::furthest_a_unit_step_goes`] for every surface that has a ceiling
/// over the whole of itself. A cone does not — a radian round it carries
/// `slope · v`, which grows for ever — so the patch's own furthest reach stands
/// in, which is exactly the ceiling over *this* patch and is what a scaling is
/// for.
///
/// # Panics
///
/// On a surface this crate has not been taught — see
/// [`how_far_a_point_lands_off_its_surface`].
fn how_this_domain_is_scaled(surface: &Surface, corners: &[[f64; 2]]) -> (f64, f64) {
    match surface {
        Surface::Cone(cone) => (
            how_wide_a_cone_is_over(cone, the_range_of_each_coordinate(corners)[1]),
            cone.slope().hypot(1.0),
        ),
        Surface::Plane(_) | Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Torus(_) => {
            surface.furthest_a_unit_step_goes()
        }
        other => panic!("this crate cannot say how {other:?} scales its domain"),
    }
}

/// How many roundings deep a point from the evaluator sits, as a multiple of
/// `f64::EPSILON` times how far out the surface reaches.
///
/// Two for the sine and cosine, two more for the multiplications that turn them
/// into a coordinate, and the rest for the addition onto the surface's origin
/// and for a distance built out of three such coordinates. A published bound,
/// so `a_point_from_the_evaluator_lands_this_close_to_its_surface` is a test
/// that could fail (`docs/decisions.md` §15.6).
const HOW_FAR_THE_EVALUATOR_ROUNDS: f64 = 8.0 * f64::EPSILON;

/// The straight line between two corners, across the way the surface curves.
///
/// For a ball that is the whole 3D chord, because a ball curves both ways. For
/// a pipe it is the chord of the **cross-section** alone: a pipe is flat along
/// its axis, so a triangle stretched up one strays by nothing, and measuring
/// its long side would refine it for no reason. For a plane it is zero, and the
/// zero says there is nothing here to bound.
///
/// **A cone is not here**, and that is deliberate: its bound is charged against
/// each corner's own radius, which is exactly what collapsing a pair of corners
/// into one length throws away. [`how_far_a_flat_piece_of_a_cone_strays`] says
/// why, and it is the only arm of [`how_far_a_flat_piece_strays`] that never
/// asks for a chord.
///
/// # Panics
///
/// On a surface this crate has not been taught — see
/// [`how_far_a_point_lands_off_its_surface`] — and on a cone, which nothing
/// routes here.
fn chords_across_the_curve(
    surface: &Surface,
    across: [[f64; 2]; 2],
    from: [f64; 2],
    to: [f64; 2],
) -> [f64; 2] {
    match surface {
        Surface::Plane(_) => [0.0, 0.0],
        Surface::Sphere(ball) => [
            ball.point_at(from[0], from[1])
                .distance_to(ball.point_at(to[0], to[1])),
            0.0,
        ],
        // Twice the radius times the sine of half the turn, which is the chord
        // for any turn and not only for a short one.
        Surface::Cylinder(pipe) => [
            2.0 * pipe.radius() * ((from[0] - to[0]) / 2.0).sin().abs(),
            0.0,
        ],
        // A cone is charged against each corner's own radius, which is exactly
        // what one length between two corners has thrown away, so nothing routes
        // one here and a cone arriving is a broken invariant of this file.
        Surface::Cone(_) => {
            panic!(
                "a cone is charged by how_far_a_flat_piece_of_a_cone_strays and never by a chord"
            )
        }
        // **A torus curves two ways by two different amounts, so it has two
        // chords and this is the longer.** Round the ring the chord is measured
        // on the outer equator, which is the widest circle the surface has —
        // `how_far_a_flat_piece_strays` divides by that same `R + r`, and a
        // chord read on a smaller circle than the one it is charged against is a
        // bound on nothing. Round the tube it is the tube's own chord.
        Surface::Torus(ring) => {
            let widest = how_wide_a_torus_is_over(ring, across[0]);
            [
                2.0 * ring.tube_radius() * ((from[0] - to[0]) / 2.0).sin().abs(),
                2.0 * widest * ((from[1] - to[1]) / 2.0).sin().abs(),
            ]
        }
        other => panic!("this crate cannot say which way {other:?} curves"),
    }
}

/// How far a flat piece with its corners on the surface can stray from it.
///
/// Put the corners `vᵢ` on a ball of radius `r` about `c` and take a point
/// `p = Σλᵢvᵢ` of the flat piece, the `λᵢ` non-negative and adding to one.
/// Since `|vᵢ − vⱼ|² = 2r² − 2(vᵢ−c)·(vⱼ−c)`, and `Σᵢⱼ λᵢλⱼ` is one,
///
/// ```text
/// r² − |p − c|²  =  ½ Σᵢⱼ λᵢλⱼ |vᵢ − vⱼ|²
/// ```
///
/// With `L` the longest of those chords, and `Σᵢ≠ⱼ λᵢλⱼ ≤ 2/3` for three
/// corners, the right-hand side is at most `L²/3`. So
///
/// ```text
/// r − |p − c|  =  (r² − |p − c|²) / (r + |p − c|)  ≤  L² / (3r).
/// ```
///
/// **Tight rather than generous.** Three corners spread evenly round a great
/// circle have `L = r√3` and a middle exactly `r` off the ball, which is
/// exactly what this gives.
///
/// A pipe is the same sum with the axis direction dropped out of every
/// distance, which is why `L` there is a chord of the cross-section and a
/// triangle stretched along the pipe is bounded by nothing at all.
///
/// A cone is not here at all: it is charged corner by corner rather than by a
/// chord, and [`how_far_a_flat_piece_of_a_cone_strays`] carries that derivation.
///
/// # A torus gets `C²·(1/(3r) + 1/(2S))`, in two halves that are the two ways it
/// bends
///
/// Neither of the two derivations above transfers whole: a torus is not a sphere
/// and its two radii of curvature are different numbers, `r` round the tube and
/// `R + r·cos v` over `cos v` round the ring. What does transfer is both of
/// them, one to each half of the surface's own two coordinates.
///
/// Write a point of the torus as `(s, a)` — how far it is from the axis and how
/// far along it — so that the surface is the set with `(s − R)² + a² = r²`, and
/// how far a point `p` sits off it is `‖(s_p − R, a_p)‖ − r`.
///
/// Take a flat piece `p = Σλᵢvᵢ` with its corners `vᵢ` on the torus, and let
/// `Yᵢ = (sᵢ, aᵢ)`, which sit on the circle of radius `r` about `(R, 0)`.
///
/// - **`a` is linear**, so `a_p = Σλᵢaᵢ` exactly.
/// - **`s` is not**, and how far it falls short is the cone's own step, one
///   dimension down: with `S = Σλᵢsᵢ`, `S − s_p ≤ Σλᵢsᵢ(1 − cos(uᵢ − u_k))` for
///   any fixed `k`, which taking `k` at the widest corner and measuring the
///   chords on that corner's circle comes to `C_u²/(2·S_max)`.
/// - So `P = (s_p, a_p)` sits within `C_u²/(2·S_max)` of `P' = (S, a_p)`, and
///   `P'` is a convex combination of the `Yᵢ`, which the **sphere's** sum
///   bounds: `r − ‖P' − (R,0)‖ ≤ C_v²/(3r)`, with `C_v` the chord in the
///   `(s, a)` plane — which is exactly the chord round the tube.
///
/// Adding the two:
///
/// ```text
/// how far the piece strays  ≤  C_v²/(3r)  +  C_u²/(2·S_max)
/// ```
///
/// **Two chords and not one, and that is why this takes a pair.** Collapsing
/// them into `C²/ρ` for one `ρ` is sound for a triangle only if `C` covers both
/// — and the two terms are largest on two different sides as often as on one, so
/// a single number would have to double itself to be safe. It also makes a
/// *boundary* side unaffordable: a side running purely round the ring is charged
/// `C²/(2S)` here and `C²/ρ` collapsed, five times more on a ring of four and
/// one, while the walker that drew it sizes its steps by the sagitta of the
/// circle it really lies on and has nothing to pay that with.
///
/// **`S` is the patch's own widest reach and not the surface's.** A patch high
/// up the tube sits on a circle of radius `R + r·cos v`, which is `R − r` at the
/// inner equator; measuring the chord on `R + r` and dividing by `R + r` is a
/// valid bound but leaves the same boundary side over budget, for the same
/// reason. [`how_wide_a_torus_is_over`] is the number, and it follows
/// [`how_wide_a_cone_is_over`]'s discipline exactly.
///
/// **Nothing about it can degenerate.** Both denominators are positive for every
/// torus and every patch — `r > 0` and `S ≥ R − r > 0` — so a torus reaches
/// neither of the two shapes a cone does: no patch of one is bounded by nothing
/// and none has a radius of zero.
///
/// `how_far_a_flat_piece_of_a_torus_really_strays_stays_under_the_published_bound`
/// is the search §15.6 asks for: 450 triangles over six shapes of ring, five
/// spans in each of the two coordinates, and three places round the tube
/// including the **inner** equator, where a torus bends the other way and where
/// a bound derived on the outer one is likeliest to be wrong. The worst uses
/// **59.2%** of the bound, because the two halves of this one are added rather
/// than one covering both.
///
/// A boundary side is a piece with two corners rather than three, and the same
/// sum gives it `L²/(4r)`. It is measured with `L²/(3r)` anyway, because what
/// gets published is the bound on the *triangle* that side ends up in, and a
/// side refused by this is one that would have failed there.
///
/// The corners are not exactly on the surface. One off by `slop` moves every
/// point of the piece by at most `slop` and lengthens every chord by at most
/// twice it, which is what the two `slop` terms below are doing.
///
/// # Panics
///
/// On a surface this crate has not been taught — see
/// [`how_far_a_point_lands_off_its_surface`].
fn how_far_a_flat_piece_strays(
    surface: &Surface,
    across: [[f64; 2]; 2],
    corners: &[[f64; 2]],
    slop: f64,
) -> f64 {
    if let Surface::Cone(cone) = surface {
        return how_far_a_flat_piece_of_a_cone_strays(cone, across, corners, slop);
    }
    let longest_chords = the_longest_chords(surface, across, corners);
    // **A torus is charged on both of its chords and every other surface on
    // one**, which is what the two-term derivation above needs and what
    // collapsing it into a single `chord²/ρ` cannot give: the two terms are
    // largest on two different sides of a triangle as often as on one, and a
    // single number would have to double itself to cover that. It is also what
    // keeps a *boundary* side affordable — a side running purely round the ring
    // is charged `C²/(2S)` here where the collapsed form charged it `C²/ρ`,
    // five times more on a ring of four and one, and the walker that drew it
    // sizes its steps by a sagitta and has no way to pay that.
    if let Surface::Torus(ring) = surface {
        let widest = how_wide_a_torus_is_over(ring, across[0]);
        let [over, around] = longest_chords.map(|chord| chord + 2.0 * slop);
        return over * over / (3.0 * ring.tube_radius()) + around * around / (2.0 * widest) + slop;
    }
    let radius = match surface {
        Surface::Plane(_) => return 0.0,
        Surface::Sphere(ball) => ball.radius(),
        Surface::Cylinder(pipe) => pipe.radius(),
        // Both answered above, each on its own terms.
        other @ (Surface::Torus(_) | Surface::Cone(_)) | other => {
            panic!("this crate cannot say how far a flat piece of {other:?} strays")
        }
    };
    let chord = longest_chords[0] + 2.0 * slop;
    chord * chord / (3.0 * radius) + slop
}

/// How far a flat piece with its corners on a cone can stray from it —
/// `maxᵢ rᵢ(1 − cos(uᵢ − u_k)) / √(1 + t²)`, one term per **corner**.
///
/// A cone's radius of curvature is not one number: it shrinks to nothing at the
/// apex, so there is no `r` to put in the sphere's sum. What is true instead is
/// this. Write the cone about its apex, `x = v·(ŵ + t·ê(u))`, so a point of it
/// at `v` sits `r = t·v` off the axis and the distance from any point `p` inside
/// the nappe to the cone is `(t·p_∥ − ‖p_⊥‖)/√(1 + t²)`.
///
/// A flat piece `p = Σλᵢvᵢ` with its corners on the cone has
/// `t·p_∥ = Σλᵢrᵢ` and `p_⊥ = Σλᵢ rᵢê(uᵢ)`, so
///
/// ```text
/// (t·p_∥ − ‖p_⊥‖)  ≤  Σλᵢ rᵢ(1 − cos(uᵢ − u_k))  ≤  maxᵢ rᵢ(1 − cos(uᵢ − u_k))
/// ```
///
/// for any fixed `k` — the first step because `‖Σ‖ ≥ Σ·ê(u_k)`, the second
/// because a convex combination is at most its largest term. `k` here is the
/// corner furthest up the cone, which is the choice that makes the widest turns
/// cheapest.
///
/// # Why it stops there, and does not go on to a chord
///
/// It used to. Taking `R_c = maxᵢ rᵢ`, measuring every chord on the circle of
/// that radius gives `C² = 2R_c²(1 − cos Δ)`, so `rᵢ(1 − cos Δ) ≤ C²/(2R_c)` and
/// the whole piece comes down to one number, `C²/(2·R_c·√(1 + t²))`.
///
/// **That last step throws away the one thing a cone's bound turns on.** It
/// replaces every corner's own radius by the largest, and a corner *at the apex*
/// has no radius at all. The shape where that bites is not exotic: cutting a
/// pointed cone's face up to its apex leaves a patch with the apex at two
/// longitudes a quarter turn apart (`cut_a_cap_up_to_its_pole` in `rcad-topo`),
/// so every triangulation of it has one triangle with **two corners on the
/// apex**. In 3D that triangle is the segment from the apex to its third corner
/// — a *ruling*, which lies exactly on the cone and strays by nothing. The
/// collapsed bound charged it `t·v/√(1 + t²)`, a fifth of the way down the cone,
/// and the refinement then chased a deflection it could never meet by fanning
/// ever-thinner triangles at the apex until the packing floor in
/// [`refine_the_inside`] declined the next point. That is where §51.7's
/// `RanOutOfRoomBeforeMeetingTheDeflection` came from, and why a coarser
/// deflection could refuse where a finer one worked: the answer existed and the
/// bound could not see it (`docs/decisions.md` §16.10).
///
/// So the corners stay corners. Nothing else about the derivation moves, and the
/// bound is never larger than the collapsed one — `rᵢ ≤ R_c` for every `i`.
///
/// **A bound that is never larger is one the refinement stops against sooner, so
/// meshes of cones got coarser, and they did.** That is not a side effect to
/// apologise for; it is the same statement as the fix. Swept at 1e-4 of each
/// solid's own size by `rcad-topo`'s
/// `whether_a_coarser_deflection_always_meshes.rs`, the five cone solids in that
/// family went `53742 → 32812` (frustum of slope 0.5 from 1 to 3),
/// `56880 → 14550` (slope 0.25 from 2 to 8), `31666 → 29144` (slope 2 from 1 to
/// 2), `71648 → 724` (pointed cone of slope 0.5 up to 4) and `77874 → 1026`
/// (pointed cone of slope 1 up to 2). Every one of the other fifteen solids in
/// that family came back with the identical triangle count at every rung, cones
/// being the only surface this touched.
///
/// The claim worth making about those coarser meshes is **not** that nothing got
/// coarser. It is that the smaller number is still an *upper* bound on how far
/// the piece really strays, so a mesh the refinement stopped against it still
/// meets the deflection it was asked for. That is what the §15.6 search below
/// measures, and what `a_cone_patch_that_reaches_its_apex.rs` measures again
/// against the cone's own closed-form distance on the meshes themselves.
///
/// **A patch spanning both nappes is bounded by nothing**, because
/// `t·p_∥ = Σλᵢrᵢ` is false there. [`how_wide_a_cone_is_over`] is where the
/// infinity comes from and says why it is the right answer.
///
/// A piece whose every corner is on the apex is one 3D point, and every term
/// above is zero, so what comes back is `slop` — which is the truth, since the
/// apex is on the cone.
///
/// The corners are not exactly on the surface: one off by `slop` moves every
/// point of the piece by at most `slop` and its radius by at most `slop`, which
/// is what the two `slop` terms are doing.
///
/// `how_far_a_flat_piece_of_a_cone_really_strays_stays_under_the_published_bound`
/// is the search §15.6 asks for.
fn how_far_a_flat_piece_of_a_cone_strays(
    cone: &Cone,
    across: [[f64; 2]; 2],
    corners: &[[f64; 2]],
    slop: f64,
) -> f64 {
    if !how_wide_a_cone_is_over(cone, across[1]).is_finite() {
        return f64::INFINITY;
    }
    let Some(furthest_up) = corners
        .iter()
        .max_by(|one, other| one[1].abs().total_cmp(&other[1].abs()))
    else {
        return slop;
    };
    let mut strays = 0.0_f64;
    for corner in corners {
        let radius = cone.radius_at(corner[1].abs()) + slop;
        let half_a_turn_away = (corner[0] - furthest_up[0]) / 2.0;
        strays = strays.max(2.0 * radius * half_a_turn_away.sin().powi(2));
    }
    strays / cone.slope().hypot(1.0) + slop
}

/// The longest chord of each of the two ways a surface bends, over every pair of
/// corners a flat piece has.
///
/// Zero when there is no pair, which is what [`triangulate`] asks for when it
/// wants how close a mesh of no size at all can get.
fn the_longest_chords(surface: &Surface, across: [[f64; 2]; 2], corners: &[[f64; 2]]) -> [f64; 2] {
    let mut longest = [0.0_f64; 2];
    for (which, corner) in corners.iter().enumerate() {
        for other in &corners[(which + 1)..] {
            let chords = chords_across_the_curve(surface, across, *corner, *other);
            longest = [longest[0].max(chords[0]), longest[1].max(chords[1])];
        }
    }
    longest
}

/// The one radius a torus's chord bound divides by, `6rS / (2S + 3r)` for
/// `S = ring + tube`.
///
/// See [`how_far_a_flat_piece_strays`] for where it comes from. It is written as
/// one number with a share of one, rather than as two terms, so that a torus
/// reads through the same `chord² / (share · radius)` every other surface does —
/// and so that [`a_step_that_stays_within`] can turn it round without a second
/// derivation.
///
/// **Always finite and always above zero**, which a cone's is not: it lies
/// between `2r` and `3r`, because `6rS/(2S + 3r)` rises from `2r` at `S = r`
/// towards `3r` as `S` grows. So a torus never reaches the two shapes a cone
/// does — a patch that is bounded by nothing, and a patch whose radius is
/// nothing at all — and neither guard below can fire for one.
fn how_far_a_torus_bends_over_a_patch(ring: &Torus, across: [[f64; 2]; 2]) -> f64 {
    let widest = how_wide_a_torus_is_over(ring, across[0]);
    let tube = ring.tube_radius();
    6.0 * tube * widest / (2.0 * widest + 3.0 * tube)
}

/// The longest chord a triangle may have and still stay within `deflection`.
///
/// [`how_far_a_flat_piece_strays`] turned round. Zero or less when even a chord
/// of no length would miss, which is a real answer and not a failure of the
/// arithmetic: it says the deflection asks for a closer fit than points from
/// the evaluator can be placed to, and the caller is told so.
///
/// # Panics
///
/// On a surface this crate has not been taught — see
/// [`how_far_a_point_lands_off_its_surface`].
fn a_step_that_stays_within(
    surface: &Surface,
    across: [[f64; 2]; 2],
    deflection: f64,
    slop: f64,
) -> f64 {
    let (radius, share) = match surface {
        Surface::Plane(_) => return f64::INFINITY,
        Surface::Sphere(ball) => (ball.radius(), 3.0),
        Surface::Cylinder(pipe) => (pipe.radius(), 3.0),
        Surface::Torus(ring) => (how_far_a_torus_bends_over_a_patch(ring, across), 1.0),
        Surface::Cone(cone) => (
            how_wide_a_cone_is_over(cone, across[1]) * cone.slope().hypot(1.0),
            2.0,
        ),
        other => panic!("this crate cannot size a step across {other:?}"),
    };
    // No step at all across a patch spanning both nappes, for the reason
    // `how_far_a_flat_piece_strays` gives one function up. The caller reads a
    // step of zero as `NoMeshCanGetThatCloseToTheSurface`, which is the truth.
    if !radius.is_finite() {
        return 0.0;
    }
    (share * radius * (deflection - slop)).max(0.0).sqrt() - 2.0 * slop
}

/// Refuse a deflection needing more points than one patch will hold.
///
/// Counted before any of them are placed, from the area the outer loop encloses
/// in the scaled domain and the step the deflection asks for: a mesh of that
/// step carries about one point per step squared. An **estimate** and not the
/// answer, and it does not have to be the answer — what it is here to stop is a
/// request that would run for a week, and those miss by orders of magnitude
/// rather than by a factor.
///
/// # Errors
///
/// [`CannotTriangulate::MoreTrianglesThanAPatchWillHold`].
fn refuse_a_patch_too_big_to_hold(
    domain: &TrimmedDomain,
    scaling: (f64, f64),
    step: f64,
) -> Result<(), CannotTriangulate> {
    let ring = domain.outer().corners();
    let mut twice_the_area = 0.0;
    for (index, corner) in ring.iter().enumerate() {
        let next = ring[(index + 1) % ring.len()];
        twice_the_area += (corner[0] * next[1] - next[0] * corner[1]) * scaling.0 * scaling.1;
    }
    let would_need = twice_the_area.abs() / (2.0 * step * step);
    if would_need > MOST_POINTS_A_PATCH_WILL_HOLD {
        return Err(CannotTriangulate::MoreTrianglesThanAPatchWillHold { would_need });
    }
    Ok(())
}

/// How many points one patch will hold before this gives up on it.
///
/// A million, which is around two million triangles and some tens of megabytes
/// for one face of one solid. Not a knob to tune: it is the line between a mesh
/// that is merely fine and a request that will not finish, and `docs/decisions.md` §25.3 puts a
/// refusal on that line rather than a wait.
const MOST_POINTS_A_PATCH_WILL_HOLD: f64 = 1_048_576.0;

/// What the refinement is aiming at, which does not change while it runs.
struct HowFineToGo<'a> {
    surface: &'a Surface,
    across: f64,
    along: f64,
    deflection: f64,
    slop: f64,
    step: f64,
    /// Each corner of the domain that lies inside one edge, with a corner
    /// before and a corner after it along its own loop, once for each pair
    /// [`corners_inside_one_edge`] gives. Empty on a plane: see
    /// [`corners_cut_off_by_one_side`].
    corners_inside_one_edge: &'a [(usize, [FixedVertexHandle; 2])],
}

/// Put points strictly inside the patch until no triangle strays too far, and
/// say how far the worst of them ends up straying.
///
/// Each round measures every triangle that is inside the domain, and cuts the
/// ones that miss in half across their **longest interior side in the scaled
/// domain**. Two different measures, on purpose: whether a triangle misses is
/// a question about chords, because that is what straying is; which side to cut
/// is a question about which side makes the triangle big, and the scaled domain
/// is where that is asked, so the pieces come out shaped like pieces. Choosing
/// the longest chord instead makes a mesh a third bigger, because it keeps
/// cutting across the same direction and leaves slivers behind it. Where a
/// placed point crowds the middle of that side, the cut goes across it to the
/// next triangle's longest side instead ([`where_to_cut`]).
///
/// A loop is never cut: only an interior side is ever chosen, and
/// [`triangulate`] asserts afterwards that every side of every loop is still
/// one edge. A triangle made of nothing but boundary sides therefore has
/// nothing to offer, and it almost never needs to — its longest chord is a
/// boundary chord, and those were all measured against the deflection before
/// any of this and passed.
///
/// # Why it stops
///
/// A point is placed only if it is at least an eighth of a `step` from every
/// point already placed. Points that far apart **pack**: discs a sixteenth of a
/// step across do not overlap, they all sit inside a patch of finite area, and
/// so there are at most `4·area·256/(π·step²)` of them. [`triangulate`] refuses
/// a patch whose area over its step squared is already past what it will hold,
/// so this is a count it agreed to within a constant.
///
/// The eighth is not a tolerance and nothing is measured against it. It is
/// there so the count above is finite, and it is set well under the spacing the
/// mesh is aiming for so that it does not decide the answer. **It did decide
/// one**: the middle of a sliver's long side sits next to the sliver's own third
/// corner, and declining it left the sliver straying too far with nothing else
/// to place (Q2192). A crowded middle is therefore not given up but handed
/// across its side, and every point that offers still keeps the eighth. A round
/// that places nothing is the end of the road, and the answer there is a
/// refusal rather than a mesh that quietly missed.
///
/// # Errors
///
/// [`CannotTriangulate::RanOutOfRoomBeforeMeetingTheDeflection`].
fn refine_the_inside(
    mesh: &mut ConstrainedDelaunayTriangulation<Point2<f64>>,
    corners: &mut Vec<[f64; 2]>,
    corner_of: &mut Vec<u32>,
    aim: &HowFineToGo<'_>,
) -> Result<f64, CannotTriangulate> {
    let mut placed = PointsAlreadyPlaced::new(aim.step / 8.0);
    loop {
        let inside = walk_in_from_the_outside(mesh);
        let mut furthest = 0.0_f64;
        let mut anything_missed = false;
        let mut wanted = Vec::new();
        for face in mesh.inner_faces() {
            if !inside[face.index()] {
                continue;
            }
            // **The reach comes first and over the whole triangle**, because it
            // is the radius every one of the three chords below is charged
            // against — see [`chords_across_the_curve`].
            let at = face
                .vertices()
                .map(|vertex| corners[corner_of[vertex.index()] as usize]);
            let across = the_range_of_each_coordinate(&at);
            let strays = how_far_a_flat_piece_strays(aim.surface, across, &at, aim.slop);
            furthest = furthest.max(strays);
            if strays <= aim.deflection {
                continue;
            }
            anything_missed = true;

            // A triangle made of nothing but boundary sides has a boundary
            // chord for its longest, and every boundary side came in under the
            // deflection, so it is almost never over budget in the first place.
            // Almost: on a torus or a cone a triangle is charged over its three
            // corners at once, which is not the same number as any one side's.
            // There is nothing to cut there, and the refusal below is the
            // answer.
            let Some(corner) = where_to_cut(mesh, face.fix(), corners, corner_of, &placed, aim)
            else {
                continue;
            };
            wanted.push(corner);
        }

        // **A side cutting off a corner is cut whether or not a point is close
        // by**, because a point close by does not remove the side and this one
        // does. That leaves the packing argument below intact: the side's
        // midpoint is the same place every round, so each corner can add at
        // most one point this way, and a second attempt finds the first one
        // there and places nothing.
        let corners_cut_off = corners_cut_off_by_one_side(mesh, &inside, aim);
        let breaking = corners_cut_off.iter().map(|(_, either_side)| {
            let [from, to] = either_side.map(|handle| corners[corner_of[handle.index()] as usize]);
            ([(from[0] + to[0]) / 2.0, (from[1] + to[1]) / 2.0], true)
        });
        let wanted: Vec<([f64; 2], bool)> = wanted
            .into_iter()
            .map(|corner| (corner, false))
            .chain(breaking)
            .collect();

        if !anything_missed && corners_cut_off.is_empty() {
            return Ok(furthest);
        }

        let mut anything_placed = false;
        for (corner, breaks_a_side) in wanted {
            let at = Point2::new(corner[0] * aim.across, corner[1] * aim.along);
            if !breaks_a_side && !placed.has_room_for(at) {
                continue;
            }
            let before = mesh.num_vertices();
            // A midpoint lies between two coordinates the mesh already took, so
            // it is never too big for one; it can be too *small*, when the two
            // very nearly cancel. Declining it leaves the triangle over budget
            // and the refusal below says so, which beats a panic.
            let Ok(handle) = mesh.insert(at) else {
                continue;
            };
            if mesh.num_vertices() == before {
                continue;
            }
            remember_which_corner(corner_of, handle.index(), corners.len());
            corners.push(corner);
            placed.remember(at);
            anything_placed = true;
        }

        if !anything_placed {
            if let (false, Some(&(corner, _))) = (anything_missed, corners_cut_off.first()) {
                return Err(CannotTriangulate::ASideCutsOffOneCornerOfACurvedPatch { corner });
            }
            return Err(CannotTriangulate::RanOutOfRoomBeforeMeetingTheDeflection {
                strays: furthest,
                deflection: aim.deflection,
            });
        }
    }
}

/// Where to put the point that cuts `face`, a triangle that strays too far:
/// the middle of its longest side off the loops, or, when a point already
/// placed crowds that middle, the middle of the longest side of the triangle
/// across it, and on along the path of longest sides for as long as each side
/// is longer than the one before. `None` when `face` has no side off the loops,
/// or the path ends before a middle with room.
///
/// **The middle that was seen crowded is the middle of a sliver's long side**,
/// crowded by the sliver's own third corner, which sits nearly on that side.
/// The middle of a triangle's longest side is at least `L/√12` from the
/// opposite corner while the angle there is under 120°, so it takes a sliver
/// to put the two close. On a cone, where a triangle running up a ruling
/// strays by nothing however long it is, the refinement leaves such slivers:
/// three points round one height under a triangle that reaches up a ruling to
/// the rim. Q2192's band of a unit cone cut by a face leaned `4` and turned `1`
/// had three, their third corners `0.0135` to `0.0161` from their long sides'
/// middles in the scaled domain where a new point keeps `0.0163`, and the band
/// was refused for want of the three points that would have cut those sides.
///
/// **The triangle across a sliver's long side is the one to cut.** Its far
/// corner is outside the sliver's circumcircle, which is what the triangulation
/// being Delaunay means, and where the sliver's angle is over 120° that corner
/// sees the shared side at under 60°, so the shared side is not that
/// triangle's longest and the path goes on. A point put down along it lands
/// inside the sliver's circumcircle when it lands close enough to the shared
/// side, and the triangulation then flips the shared side away; the next round
/// asks again of whatever is left.
///
/// Each step goes to a strictly longer side, so the path ends. Every point it
/// offers still has to keep the eighth of a step from every placed point, so
/// the packing argument on [`refine_the_inside`] holds as it stands.
fn where_to_cut(
    mesh: &ConstrainedDelaunayTriangulation<Point2<f64>>,
    face: FixedFaceHandle<InnerTag>,
    corners: &[[f64; 2]],
    corner_of: &[u32],
    placed: &PointsAlreadyPlaced,
    aim: &HowFineToGo<'_>,
) -> Option<[f64; 2]> {
    let (mut reach, mut side) = the_longest_side_off_the_loops(mesh, face)?;
    loop {
        let edge = mesh.directed_edge(side);
        let [from, to] =
            [edge.from(), edge.to()].map(|end| corners[corner_of[end.index()] as usize]);
        let middle = [(from[0] + to[0]) / 2.0, (from[1] + to[1]) / 2.0];
        if placed.has_room_for(Point2::new(middle[0] * aim.across, middle[1] * aim.along)) {
            return Some(middle);
        }
        let beyond = edge
            .rev()
            .face()
            .as_inner()
            .expect("a side off the loops of a triangle inside the patch has one across it");
        let (further, next) = the_longest_side_off_the_loops(mesh, beyond.fix())?;
        if further <= reach {
            return None;
        }
        (reach, side) = (further, next);
    }
}

/// The longest side of `face` that is not on a loop, by the square of its
/// length in the scaled domain, or `None` when every side is on one.
fn the_longest_side_off_the_loops(
    mesh: &ConstrainedDelaunayTriangulation<Point2<f64>>,
    face: FixedFaceHandle<InnerTag>,
) -> Option<(f64, FixedDirectedEdgeHandle)> {
    let mut longest: Option<(f64, FixedDirectedEdgeHandle)> = None;
    for edge in mesh.face(face).adjacent_edges() {
        if edge.is_constraint_edge() {
            continue;
        }
        let (here, there) = (edge.from().position(), edge.to().position());
        let (dx, dy) = (there.x - here.x, there.y - here.y);
        let reach = dx.mul_add(dx, dy * dy);
        if longest.is_none_or(|(so_far, _)| reach > so_far) {
            longest = Some((reach, edge.fix()));
        }
    }
    longest
}

/// The corners of a curved patch that one side of the triangulation cuts
/// straight across, from the corner before to the corner after along the loop,
/// or from a corner that is the same point in space as either
/// ([`corners_inside_one_edge`]).
///
/// **Such a side can be a side of the next face too, and then the mesh is
/// wrong.** Where two faces meet at a sharp edge — a cap leaning steeply across
/// a bar meets the wall at eleven degrees near the top of its ellipse — the
/// three corners around the edge's sharpest bend are a natural ear for both
/// faces. Each cuts it off with the same side, both keep the same triangle facing
/// opposite ways, and the side between the corner before and the corner after is
/// then used four times: a flat pocket with nothing inside it, pinched off the
/// rest of the mesh (`docs/decisions.md` §187.4). This crate is shown one face at
/// a time and cannot see the other one, so it cannot agree a side with it; what
/// it can do is never draw one that the other face could also draw.
///
/// **Only a curved patch is held to this**, and one is enough. The three
/// corners of such a triangle all lie on the boundary and so on the edge's own
/// curve; on a plane that triangle is exactly the surface, and on a curved face
/// it is the edge's curve standing in for a strip of surface. Keeping the flat
/// face's triangle and cutting the curved face's side in two gives the curved
/// face a point off the plane of that curve, so the two no longer coincide. Two
/// flat faces never meet along a curve, since two planes meet in a line.
///
/// **Only a corner inside one edge is held to it**, when the caller has said
/// which those are ([`TrimmedDomain::with_corners_between_two_edges`]). Where
/// the boundary passes from one edge to another there is in general no one face
/// across both, and every corner of a rectangle is such a place.
///
/// A plane's list is empty, so this finds nothing there.
fn corners_cut_off_by_one_side(
    mesh: &ConstrainedDelaunayTriangulation<Point2<f64>>,
    inside: &[bool],
    aim: &HowFineToGo<'_>,
) -> Vec<(usize, [FixedVertexHandle; 2])> {
    let mut cut_off = Vec::new();
    for &(position, [before, after]) in aim.corners_inside_one_edge {
        let Some(side) = mesh.get_edge_from_neighbors(before, after) else {
            continue;
        };
        if side.is_constraint_edge() {
            continue;
        }
        if side
            .face()
            .as_inner()
            .is_some_and(|face| inside[face.index()])
        {
            cut_off.push((position, [before, after]));
        }
    }
    cut_off
}

/// Each corner that lies inside one edge, with the triangulation's handles for
/// a corner before it and a corner after it along its own loop: the one either
/// side, and every corner that is the same point in space as one of those.
///
/// **Where a surface pinches, one point is several corners in a row**, and the
/// side that cuts a corner off is a side between two points, whichever of the
/// corners standing at each it is drawn from. A ball's pole is a whole side of
/// the patch, from the longitude the boundary arrives at to the one it leaves
/// by (`docs/decisions.md` §25.6). Asked only of the corner beside the cut-off
/// one, the side from the far end of that pole side was free to be drawn
/// instead. Q889's shape is where it showed: a unit ball less a cube whose
/// face plane tilts 30 degrees and misses `N` by half a rounding leaves a cap
/// with `N` on its rim, and a half of it runs from where a cut reaches `N`,
/// along the pole side to where the rim arrives at `N`, and on down the rim.
/// The side from the rim's `N` to the rim point two on was cut here, the
/// triangulation drew the side from the cut's `N` to that point, the disc
/// across, fanned from `N`, drew it too, and measured at `0.3` the answer was
/// refused `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }` (Q1027,
/// `a_turned_block_whose_face_holds_a_balls_pole.rs` in `rcad-topo`).
///
/// A corner is taken to be one point with the next only when both stand at
/// the very second coordinate the surface pinches at
/// ([`Surface::where_it_pinches`]), which is where a caller writes a pole
/// side. One read as a pole but written a rounding off it is not seen as one,
/// and is asked about as any corner is.
fn corners_inside_one_edge(
    domain: &TrimmedDomain,
    handles: &[FixedVertexHandle],
) -> Vec<(usize, [FixedVertexHandle; 2])> {
    let corners = every_corner(domain);
    let mut before = vec![0; handles.len()];
    let mut after = vec![0; handles.len()];
    for (from, to) in every_side(domain) {
        after[from] = to;
        before[to] = from;
    }
    let pinch = domain.surface().where_it_pinches();
    let one_point = |at: usize, next: usize| {
        pinch.is_some_and(|pinch| corners[at][1].abs() == pinch)
            && corners[next][1] == corners[at][1]
    };
    let the_corners_at_one_point_from = |first: usize, onwards: &[usize], cut_off: usize| {
        let mut run = vec![first];
        let mut at = first;
        while onwards[at] != cut_off && !run.contains(&onwards[at]) && one_point(at, onwards[at]) {
            at = onwards[at];
            run.push(at);
        }
        run
    };
    let mut inside = Vec::new();
    for position in (0..handles.len()).filter(|&position| !domain.is_between_two_edges(position)) {
        let afters = the_corners_at_one_point_from(after[position], &after, position);
        for one_before in the_corners_at_one_point_from(before[position], &before, position) {
            for &one_after in &afters {
                inside.push((position, [handles[one_before], handles[one_after]]));
            }
        }
    }
    inside
}

/// The points the refinement has put down, bucketed by how far apart they have
/// to be, so that asking whether a new one crowds an old one is not a look at
/// all of them.
struct PointsAlreadyPlaced {
    apart: f64,
    buckets: HashMap<(i64, i64), Vec<Point2<f64>>>,
}

impl PointsAlreadyPlaced {
    fn new(apart: f64) -> Self {
        Self {
            apart,
            buckets: HashMap::new(),
        }
    }

    /// Whether `at` is far enough from every point already placed.
    ///
    /// A bucket is exactly that far across, so anything too close to `at` is in
    /// one of the nine buckets around it, and no bucket holds more than a
    /// handful because its own points are that far apart too.
    fn has_room_for(&self, at: Point2<f64>) -> bool {
        let (column, row) = self.bucket(at);
        for across in -1..=1_i64 {
            for up in -1..=1_i64 {
                let Some(neighbours) = self
                    .buckets
                    .get(&(column.saturating_add(across), row.saturating_add(up)))
                else {
                    continue;
                };
                for other in neighbours {
                    let (dx, dy) = (other.x - at.x, other.y - at.y);
                    if dx * dx + dy * dy < self.apart * self.apart {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn remember(&mut self, at: Point2<f64>) {
        self.buckets.entry(self.bucket(at)).or_default().push(at);
    }

    fn bucket(&self, at: Point2<f64>) -> (i64, i64) {
        (
            (at.x / self.apart).floor() as i64,
            (at.y / self.apart).floor() as i64,
        )
    }
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

    /// One loop names the same point twice.
    ///
    /// Two different loops naming the same point is a pinch, not a broken loop,
    /// and it is allowed — see [`refuse_loops_that_touch`].
    #[error("the corner at {position} is in the same place as an earlier one of its own loop")]
    CornersCoincide {
        /// Which corner repeats an earlier one.
        position: usize,
    },

    /// A corner of a loop lands on a side of another loop, or of its own,
    /// without being one of that side's own ends.
    ///
    /// See [`refuse_loops_that_touch`] for why this is refused while two loops
    /// meeting at a corner they both name is not, and
    /// [`refuse_a_side_with_a_corner_on_it`] for a loop touching itself.
    ///
    /// **`at` is the field a caller further up can do anything with**, and it
    /// is here because the two indices are not. They count corners of the
    /// domain, which for the kernel's mesher is a list it assembled itself —
    /// wires walked in an order of its own choosing, with a point put on every
    /// bend — so "corner 7" is an address into something the caller of
    /// `SolidView::mesh` never built. A place in space is one it can find.
    /// `docs/decisions.md` §123.
    #[error("the corner at {corner} sits on the side leaving corner {side}")]
    LoopsTouch {
        /// The corner sitting on someone else's side.
        corner: usize,
        /// Which corner that side leaves from.
        side: usize,
        /// Where the two loops touch, on the surface this domain named.
        ///
        /// A caller that moved the domain into a frame of its own is expected
        /// to move this back before publishing it — which is what the kernel's
        /// mesher does (§113.6, §116, §123).
        at: Point,
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
        /// Where that corner is, on the surface this domain named. The same
        /// field as [`CannotTriangulate::LoopsTouch`]'s, for the same reason.
        at: Point,
    },

    /// A scaled coordinate is outside what the triangulation will take.
    ///
    /// The range is `2^-142` to `2^201`, which is narrower than `f64`'s at both
    /// ends and narrower than this kernel's own limits. Note that it is the
    /// **scaled** coordinate that has to fit: an angle round a pipe of radius
    /// `1e60` is a small number that becomes a large one.
    ///
    /// **Under `2^-142` is refused only when zero would change the answer.**
    /// [`triangulate`] hands spade such a coordinate as zero, then checks with
    /// the corners where they really are that the triangles still tile the
    /// patch and draw every side whole. When they do not, this is the refusal,
    /// with `why` [`InsertionError::TooSmall`] and `position` the first corner
    /// the flush moved — the one spade refused before the flush existed
    /// (`docs/decisions.md` §246.5, §321).
    ///
    /// A refusal and never a wrong answer, which is the right way round.
    #[error("the corner at {position} is outside the range a triangulation takes: {why:?}")]
    CoordinateOutOfRange {
        /// Which corner.
        position: usize,
        /// What the triangulation said about it.
        why: InsertionError,
    },

    /// One side of one loop already strays further than the deflection allows.
    ///
    /// A side is a chord of the surface, and a chord's distance from the
    /// surface is fixed the moment the two corners are chosen. This crate may
    /// not put a point on a loop — the two faces along a shared edge would each
    /// refine it in their own `(u, v)` and land on different 3D points
    /// (`docs/decisions.md` §25.1) — so no number of interior points helps and
    /// there is nothing here to try.
    ///
    /// **About the request and never about the surface.** The fix is at the
    /// caller, which walks the boundary and is the only one that can see both
    /// sides of it: hand over more corners, or ask for a coarser mesh.
    ///
    /// From [`zip_up_a_band`] it is that side or the triangle standing on it,
    /// which reaches across the band to a corner of the other loop and is no
    /// wider round the band than the widest side either loop has. A refused
    /// triangle is fixed the same way, by more corners on either loop.
    #[error(
        "the side leaving corner {side} strays {strays} from the surface, \
         which is further than the deflection allows"
    )]
    BoundarySideStraysFurtherThanTheDeflection {
        /// Which corner that side leaves from.
        side: usize,
        /// How far that one chord strays, in the model's own unit.
        strays: f64,
    },

    /// The deflection asks for a closer fit than a point can be placed to.
    ///
    /// [`Surface::point_at`] rounds, and a mesh made of its points is no nearer
    /// the surface than they are. A deflection under that is not a request for
    /// more triangles; it is a request for arithmetic `f64` does not have, and
    /// refining towards it would run forever without ever arriving.
    ///
    /// The floor is not a property of the surface alone: a ball of radius one a
    /// kilometre from the origin has its points known to a rounding of the
    /// kilometre, so the same ball at the origin can be meshed far finer. That
    /// is the *representable grid* speaking and not this crate — the gaps
    /// between `f64`s near a coordinate of a thousand are what they are, and no
    /// ordering, no rescaling and no compensated sum puts a point between two
    /// of them. **A scaling does reach it, and only the way §18 says**: shrink
    /// the whole model and the floor shrinks with it, in exact step, so nothing
    /// relative is being refused.
    ///
    /// **What the floor is measured against is a bound and not the grid**, and
    /// the difference shows on a pipe. [`how_far_a_point_lands_off_its_surface`]
    /// charges the rounding against how far the whole point reaches, and a pipe
    /// is invariant along its own axis: a pipe of radius one whose axis origin
    /// sits at `z = 1e9` has its skin known to a rounding of *one*, and this
    /// refuses it a deflection of `1e-9` on the strength of a rounding of the
    /// billion. That is a §16.10 gap in the bound rather than in the answer,
    /// and it is left standing because closing it needs the axis *direction* —
    /// an error along `z` moves a point off a `z` pipe by nothing and off a
    /// slanted one by all of it — and because `rcad-topo` never builds one:
    /// `add_round_bar` puts the axis at the origin and carries the placement on
    /// the solid.
    ///
    /// [`Surface::point_at`]: rcad_geom::Surface::point_at
    #[error(
        "no mesh of this surface built from these points gets closer than {closest}, \
         and the deflection is {deflection}"
    )]
    NoMeshCanGetThatCloseToTheSurface {
        /// What was asked for, in the model's own unit.
        deflection: f64,
        /// The closest a mesh of this surface built from these points can be,
        /// in the same unit.
        closest: f64,
    },

    /// The deflection needs more points than one patch will hold.
    ///
    /// Counted before any of them are placed, so this is a refusal rather than
    /// a wait. An estimate: a mesh of the step this deflection asks for carries
    /// about one point per step squared, and a request that trips this misses
    /// by orders of magnitude rather than by a factor.
    #[error(
        "this deflection needs about {would_need} points, which is more than a patch will hold"
    )]
    MoreTrianglesThanAPatchWillHold {
        /// About how many points the deflection asks for.
        would_need: f64,
    },

    /// The refinement had nowhere left to put a point and had still not
    /// arrived.
    ///
    /// No best-effort answer: a patch that quietly missed its deflection would
    /// need every caller to compare two numbers and none would
    /// (`docs/decisions.md` §25.3). This says what the mesh got to, so the
    /// caller can ask for that instead.
    ///
    /// Reachable when the point that would cut a triangle in half cannot be put
    /// where it is wanted: the two coordinates it is between very nearly
    /// cancel and the triangulation will not take what is left, or the place it
    /// would go is already occupied.
    ///
    /// **Only the first of those two is known to reach this.** With the two
    /// ways [`refine_the_inside`] can decline a point tallied separately and the
    /// whole test suite run, this refusal came back exactly once in 1694 tests,
    /// with no point declined for crowding and two declined by the
    /// triangulation — `a_refinement_point_the_triangulation_will_not_take_is_refused_and_never_a_panic`.
    /// The packing floor used to be reached by a pointed cone chasing a
    /// deflection the collapsed bound invented for it, and
    /// [`how_far_a_flat_piece_of_a_cone_strays`] no longer invents it. It was
    /// reached once more, after that count, by a sliver on a cone whose long
    /// side's middle sat next to its own third corner (Q2192), and a crowded
    /// middle is now handed across its side rather than given up
    /// ([`where_to_cut`]). The floor stays because it is what makes
    /// [`refine_the_inside`] terminate at all, not because something is
    /// expected to land on it.
    #[error(
        "the mesh still strays {strays}, the deflection is {deflection}, and there is nowhere left to put a point"
    )]
    RanOutOfRoomBeforeMeetingTheDeflection {
        /// How far the mesh strays as it stands, in the model's own unit.
        strays: f64,
        /// What was asked for, in the same unit.
        deflection: f64,
    },

    /// On a curved surface, a side from the corner before `corner` to the corner
    /// after it would not go away, and the face on the other side of the
    /// boundary may draw the same side.
    ///
    /// The refinement cuts such a side in half at its middle, and this is what
    /// is left when that point cannot be placed: the triangulation would not
    /// take it, or it went in on an earlier round and the side survived it.
    /// Neither has been seen in this workspace. Returning the patch instead
    /// could hand back a mesh with one side used four times
    /// (`docs/decisions.md` §187.4), which is worse than no mesh.
    #[error(
        "on a curved surface, a side cuts straight across corner {corner} of the boundary \
         and there is nowhere to put the point that would break it"
    )]
    ASideCutsOffOneCornerOfACurvedPatch {
        /// Which corner is cut off.
        corner: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcad_geom::{Axis, Cylinder, Point, Sphere, Torus, Vector};

    /// A star of seven points drawn by joining every second one turns the same
    /// way at every corner and goes round twice, so it is not the convex outline
    /// [`cut_by_corner_order`] needs. Its sides cross, so [`triangulate`]
    /// refuses it before the rule is asked; this asks the rule's own check.
    #[test]
    fn an_outline_that_turns_one_way_is_cut_by_corner_order_only_once_round() {
        let seven = |step: usize| {
            (0..7)
                .map(|at| {
                    let angle = core::f64::consts::TAU * (at * step % 7) as f64 / 7.0;
                    [angle.cos(), angle.sin()]
                })
                .collect::<Vec<_>>()
        };
        let mut clockwise = seven(1);
        clockwise.reverse();
        let dented = vec![[0.0, 0.0], [4.0, 0.0], [2.0, 1.0], [4.0, 4.0], [0.0, 4.0]];
        let straight = vec![[0.0, 0.0], [2.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];

        assert_eq!(turns_one_way_once_round(&seven(1)), Some(true));
        assert_eq!(turns_one_way_once_round(&clockwise), Some(false));
        assert_eq!(
            orient2d(seven(2)[0], seven(2)[1], seven(2)[2]),
            Decision::Pos,
            "the star turns anticlockwise at its corners"
        );
        assert_eq!(turns_one_way_once_round(&seven(2)), None);
        assert_eq!(turns_one_way_once_round(&dented), None);
        assert_eq!(turns_one_way_once_round(&straight), None);
    }

    /// A point on a side splits it, and the check [`triangulate`] asserts once
    /// the refinement is done sees the split. No refinement has been seen to put
    /// a point on a side (Q2221), so the point is put there by hand here, the
    /// way a midpoint rounded onto the side would be.
    #[test]
    fn a_point_put_on_a_side_leaves_that_side_in_pieces() {
        let flat =
            rcad_geom::Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal");
        let domain = TrimmedDomain::new(
            Surface::from(flat),
            crate::domain::Loop::through(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]])
                .expect("a square"),
            Vec::new(),
            rcad_geom::Deflection::of(1.0).expect("a positive length"),
        );
        let mut mesh = ConstrainedDelaunayTriangulation::<Point2<f64>>::new();
        let handles: Vec<FixedVertexHandle> = every_corner(&domain)
            .into_iter()
            .map(|[u, v]| {
                mesh.insert(Point2::new(u, v))
                    .expect("a small whole number")
            })
            .collect();
        for (from, to) in every_side(&domain) {
            mesh.add_constraint(handles[from], handles[to]);
        }
        assert_eq!(
            a_side_the_mesh_holds_in_pieces(&mesh, &domain, &handles),
            None
        );

        mesh.insert(Point2::new(4.0, 2.0))
            .expect("the middle of the side from corner 1");
        assert_eq!(
            a_side_the_mesh_holds_in_pieces(&mesh, &domain, &handles),
            Some(1)
        );
    }

    /// Half a cap of a unit ball with the north pole `N` on its rim, as
    /// `rcad-topo` hands it over for Q889's shape at tilt direction 30 and
    /// `−0.5` roundings: from where a cut reaches `N`, along the pole side to
    /// where the rim arrives at `N`, down the rim through four points, and back
    /// up the cut through one. The rim is one closed edge that a flat disc
    /// holds too, so no side may cut a rim point off between its neighbours,
    /// and the two corners at `N` are one neighbour. Before Q1027 the side from
    /// the rim's `N` to the second rim point was cut, and the one from the
    /// cut's `N` to it was drawn.
    #[test]
    fn no_side_cuts_off_the_rim_point_beside_a_pole_from_either_end_of_the_pole_side() {
        let pole = core::f64::consts::FRAC_PI_2;
        let corners = vec![
            [0.517_088_499_497_801_1, pole],
            [-0.858_556_149_995_533_4, pole],
            [-0.858_556_149_995_533_4, 1.355_101_762_754_067_1],
            [-0.449_901_067_827_468_6, 0.942_862_505_055_460_4],
            [0.008_044_512_237_852_575, 0.639_814_837_818_900_4],
            [0.515_025_951_126_744_1, 0.523_630_599_236_529_3],
            [0.516_057_225_312_272_6, 1.047_213_463_015_713],
        ];
        // `N` twice, the rim's points 1 to 4, and the point along the cut.
        let point_of: [u32; 7] = [0, 0, 1, 2, 3, 4, 8];
        let ball = Sphere::new(Point::origin(), 1.0).expect("a unit ball");
        let domain = TrimmedDomain::new(
            Surface::from(ball),
            crate::domain::Loop::through(corners).expect("seven corners"),
            Vec::new(),
            rcad_geom::Deflection::of(0.3).expect("a positive length"),
        )
        .with_corners_between_two_edges(vec![true, true, false, false, false, true, true]);
        let patch = triangulate(&domain).expect("the half cap is cut");

        let point = |corner: u32| point_of.get(corner as usize).copied();
        let cutting_off_a_rim_point = [[0, 2], [1, 3], [2, 4]].map(|ends| ends.map(Some));
        for triangle in patch.triangles() {
            for (from, to) in [(0, 1), (1, 2), (2, 0)] {
                let side = [point(triangle[from]), point(triangle[to])];
                assert!(
                    !cutting_off_a_rim_point
                        .iter()
                        .any(|&[one, other]| side == [one, other] || side == [other, one]),
                    "triangle {triangle:?} cuts a rim point off with the side between points \
                     {side:?}"
                );
            }
        }
    }

    /// [`how_far_a_flat_piece_strays`]'s torus arm is a published bound, so here
    /// is the search that could find something over it (`docs/decisions.md`
    /// §15.6).
    ///
    /// The claim is `C²/ρ` with `ρ = 6rS/(2S + 3r)` and `S = R + r`, and what is
    /// measured against it is how far a point of the flat triangle really sits
    /// off the torus — which is `‖(s − R, a)‖ − r` **exactly**, because a torus
    /// is one of the few surfaces whose distance function is a closed form. No
    /// sampling of the surface and no root-finding.
    ///
    /// The search runs over six shapes of ring, from one barely fatter than its
    /// hole to one whose tube is nine tenths of its ring, and over triangles
    /// spanning up to most of a half turn in **both** coordinates at once —
    /// including ones sitting on the inner equator, which is where a torus bends
    /// hardest the other way and where a bound derived on the outer equator
    /// would be most likely to be wrong.
    #[test]
    fn how_far_a_flat_piece_of_a_torus_really_strays_stays_under_the_published_bound() {
        let mut worst_share = 0.0_f64;
        let mut checked = 0usize;

        for (big, small) in [
            (4.0, 1.0),
            (10.0, 0.5),
            (1.0, 0.9),
            (2.0, 1.0),
            (100.0, 3.0),
            (0.5, 0.25),
        ] {
            let ring = Torus::new(Axis::z(), big, small).expect("a tube inside its ring");
            let surface = Surface::from(ring);
            for round in [0.02, 0.2, 0.8, 1.5, 2.6] {
                for over in [0.02, 0.2, 0.8, 1.5, 2.6] {
                    // Sitting on the outer equator, on the inner one, and half
                    // way up the side.
                    for start in [0.0, core::f64::consts::PI, core::f64::consts::FRAC_PI_2] {
                        let at = [[0.0, start], [round, start], [round / 2.0, start + over]];
                        let corners = at.map(|[u, v]| ring.point_at(u, v));

                        let across = the_range_of_each_coordinate(&at);
                        let bound = how_far_a_flat_piece_strays(&surface, across, &at, 0.0);
                        assert!(
                            bound.is_finite() && bound > 0.0,
                            "a torus is always bounded"
                        );

                        let mut furthest = 0.0_f64;
                        const STEPS: usize = 20;
                        for first in 0..=STEPS {
                            for second in 0..=(STEPS - first) {
                                let third = STEPS - first - second;
                                let weights =
                                    [first, second, third].map(|share| share as f64 / STEPS as f64);
                                let inside = Point::new(
                                    (0..3).map(|c| weights[c] * corners[c].x()).sum(),
                                    (0..3).map(|c| weights[c] * corners[c].y()).sum(),
                                    (0..3).map(|c| weights[c] * corners[c].z()).sum(),
                                );
                                let from_the_axis = inside.x().hypot(inside.y()) - big;
                                furthest =
                                    furthest.max((from_the_axis.hypot(inside.z()) - small).abs());
                            }
                        }
                        checked += 1;
                        worst_share = worst_share.max(furthest / bound);
                        assert!(
                            furthest <= bound,
                            "a triangle on a ring of {big} and {small} spanning {round} by \
                             {over} from {start} strays {furthest}, and the bound says {bound}"
                        );
                    }
                }
            }
        }

        println!("the worst of {checked} triangles used {worst_share} of its published bound");
        assert!(checked > 400, "the search covered {checked} triangles");
        assert!(
            worst_share > 0.05,
            "a bound nothing comes near proves nothing, and this one was only \
             {worst_share} used"
        );
    }

    /// How the three corners of one search triangle sit up the cone: the `v` of
    /// the corner at `u = 0`, of the one at `u = span`, and of the one at
    /// `u = span/2`.
    ///
    /// **Three distinct heights in most of them, and that is the point.** The
    /// bound charges every corner its *own* radius, so a family whose corners
    /// all sit on one circle cannot tell it apart from the collapsed
    /// `C²/(2·R_c)` this arm replaced — on `[far, far, near]` the two formulas
    /// are algebraically the same number, `2·t·far·sin²(span/2)`, and a search
    /// made only of those would have measured a bound the code does not
    /// compute.
    ///
    /// The six `[far, far, near]` rows are still here, because that is the
    /// shape a first row of triangles off a ring really has and one of them
    /// reaches the apex. Every one of them scores **exactly 1** against the
    /// collapsed form. The other six separate the two: the three frustum bands
    /// `rcad-topo`'s own sweep meshes — from 1 to 3, from 2 to 8 and from 1 to
    /// 2 — cut the way the refinement cuts them, so the third corner lands half
    /// way up and the piece is charged `near/far` of what it was, a third, a
    /// quarter and a half; the first of those three mirrored, so that the
    /// widest corner is at the far end of the span rather than the near one and
    /// nothing rests on which end `k` lands at; one with a corner **on** the apex,
    /// which goes to an eighth; and one with **two** corners on the apex, which
    /// in 3D is a ruling lying exactly on the cone and is charged nothing.
    const HOW_THE_THREE_CORNERS_SIT: [[f64; 3]; 12] = [
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 0.001],
        [1.1, 1.1, 0.1],
        [2.0, 2.0, 1.0],
        [5.0, 5.0, 4.0],
        [8.0, 8.0, 1.0],
        [3.0, 1.0, 2.0],
        [8.0, 2.0, 5.0],
        [2.0, 1.0, 1.5],
        [1.0, 3.0, 2.0],
        [1.0, 0.0, 0.5],
        [0.0, 0.0, 1.0],
    ];

    /// [`how_far_a_flat_piece_of_a_cone_strays`] is a published bound, so here
    /// is the search that could find something over it (`docs/decisions.md`
    /// §15.6).
    ///
    /// The claim is `maxᵢ rᵢ(1 − cos(uᵢ − u_k))/√(1 + t²)`, **one term per
    /// corner**, with `rᵢ = t·vᵢ` that corner's own radius and `k` the corner
    /// furthest up the cone. What is measured against it is how far a point of
    /// the flat triangle really sits off the cone, which for a point inside the
    /// nappe is `(t·p_∥ − ‖p_⊥‖)/√(1 + t²)` exactly — no sampling of the surface
    /// and no root-finding, because a cone is one of the few surfaces whose
    /// distance function is a closed form.
    ///
    /// The search is over eight slopes from a very shallow cone to a very steep
    /// one, over triangles spanning from a hundredth of a turn to most of a half
    /// turn, and over the twelve ways [`HOW_THE_THREE_CORNERS_SIT`] puts the
    /// three corners up it. Each triangle is sampled on a barycentric grid, and
    /// each is given the `slop` [`triangulate`] would give it rather than none,
    /// so what is measured is the number the mesher really compares against.
    ///
    /// **It measures the corner-by-corner bound and not the collapsed one it
    /// replaced**, and it says so with a number rather than by assertion:
    /// [`what_the_collapsed_bound_would_have_charged`] is computed beside it on
    /// every triangle, and the tightest triangle of the family sits at **an
    /// eighth** of what the collapsed form charged. The frustum rows sit at a
    /// half, a third and a quarter, which is where the tightening that cut the
    /// slope-0.25 frustum's mesh from 56880 triangles to 14550 at 1e-4 of its
    /// size lives. On the 48 rulings the collapsed form charges the whole span
    /// at the far corner's radius, where the truth — and this bound — is nothing
    /// at all.
    ///
    /// Over 576 triangles the worst uses **65.9%** of the bound, which is the
    /// ordinary gap between a worst case and a drawn one. The lower assertion at
    /// the end is the other half of §15.6: a bound nothing ever comes near is a
    /// bound that proves nothing.
    #[test]
    fn how_far_a_flat_piece_of_a_cone_really_strays_stays_under_the_published_bound() {
        let mut worst_share = 0.0_f64;
        let mut tightest_against_the_collapsed_form = 1.0_f64;
        let mut rulings = 0usize;
        let mut checked = 0usize;

        for slope in [0.05, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 20.0] {
            let cone = Cone::new(Axis::z(), slope).expect("a positive slope");
            let surface = Surface::from(cone);
            for span in [0.01, 0.1, 0.5, 1.0, 1.6, 2.6] {
                for [at_the_start, at_the_end, half_way_round] in HOW_THE_THREE_CORNERS_SIT {
                    let at = [
                        [0.0, at_the_start],
                        [span, at_the_end],
                        [span / 2.0, half_way_round],
                    ];
                    let corners = at.map(|[round, along]| cone.point_at(round, along));

                    let across = the_range_of_each_coordinate(&at);
                    let slop = how_far_a_point_lands_off_its_surface(&surface, &at);
                    let bound = how_far_a_flat_piece_strays(&surface, across, &at, slop);
                    let collapsed = what_the_collapsed_bound_would_have_charged(&cone, &at, slop);
                    assert!(
                        bound <= collapsed,
                        "the corner-by-corner bound is never larger than the collapsed one, \
                         and on a cone of slope {slope} spanning {span} over {at:?} it says \
                         {bound} against {collapsed}"
                    );
                    // A piece the bound charges nothing for once the corners are
                    // taken to be exactly on the cone is one that lies on the
                    // cone: a ruling. Its share of the collapsed form is
                    // whatever the rounding allowance happens to be over
                    // whatever the span happens to cost, so it is counted rather
                    // than averaged in.
                    if how_far_a_flat_piece_strays(&surface, across, &at, 0.0) == 0.0 {
                        rulings += 1;
                    } else {
                        tightest_against_the_collapsed_form =
                            tightest_against_the_collapsed_form.min(bound / collapsed);
                    }

                    let mut furthest = 0.0_f64;
                    const STEPS: usize = 24;
                    for first in 0..=STEPS {
                        for second in 0..=(STEPS - first) {
                            let third = STEPS - first - second;
                            let weights =
                                [first, second, third].map(|share| share as f64 / STEPS as f64);
                            let inside = Point::new(
                                (0..3).map(|c| weights[c] * corners[c].x()).sum(),
                                (0..3).map(|c| weights[c] * corners[c].y()).sum(),
                                (0..3).map(|c| weights[c] * corners[c].z()).sum(),
                            );
                            let up = inside.z();
                            let out = inside.x().hypot(inside.y());
                            furthest = furthest.max((slope * up - out) / slope.hypot(1.0));
                        }
                    }
                    checked += 1;
                    worst_share = worst_share.max(furthest / bound);
                    assert!(
                        furthest <= bound,
                        "a triangle on a cone of slope {slope} spanning {span} over {at:?} \
                         strays {furthest}, and the bound says {bound}"
                    );
                }
            }
        }

        println!(
            "the worst of {checked} triangles used {worst_share} of its published bound; \
             {rulings} of them are rulings the collapsed form charged the whole span for, \
             and of the rest the tightest sits at {tightest_against_the_collapsed_form} \
             of what the collapsed form charged"
        );
        assert!(checked > 500, "the search covered {checked} triangles");
        // And it is the bound this file computes that was searched, not the
        // collapsed one it replaced. Those two agree on every triangle whose
        // corners share a circle, so a family made of only those would pass
        // this whole test against either — which is exactly what the search
        // here used to be. A ratio of one means the family stopped separating
        // them; a ratio of one *everywhere* means the code went back.
        assert!(
            rulings > 0,
            "not one triangle of the search lies on a ruling, which is the shape \
             the collapsed bound could not see"
        );
        assert!(
            tightest_against_the_collapsed_form < 0.2,
            "the two formulas are supposed to be far apart on this family and the \
             tightest triangle only got to {tightest_against_the_collapsed_form} \
             of the collapsed form"
        );

        // And a patch that spans both nappes is bounded by nothing at all,
        // because the derivation's `t·p_∥ = Σλᵢrᵢ` is false there. Nothing in
        // the kernel builds one; `SolidBuilder` is public, so a caller can
        // describe one, and what it gets is a refusal rather than a number.
        let cone = Cone::new(Axis::z(), 1.0).expect("a positive slope");
        let both_nappes = [[0.0, -1.0], [1.0, 1.0], [2.0, 2.0]];
        assert_eq!(
            how_far_a_flat_piece_strays(
                &Surface::from(cone),
                the_range_of_each_coordinate(&both_nappes),
                &both_nappes,
                0.0
            ),
            f64::INFINITY
        );
        assert!(
            worst_share > 0.05,
            "a bound nothing comes near is a bound that proves nothing: the worst was \
             {worst_share} of it"
        );
    }

    /// What the collapsed bound this arm replaced would have charged the same
    /// piece: `C²/(2·R_c·√(1 + t²))`, with `R_c` the radius of the widest corner
    /// and `C` the longest chord measured on **that** corner's circle.
    ///
    /// Here so the search above can show it is measuring the bound the code
    /// really computes. Written as `2·R_c·sin²(Δ/2)`, which is the same number
    /// with the squaring and the dividing cancelled out and no chance of a
    /// rounding pulling it under the corner-by-corner form for a reason that is
    /// not the derivation.
    fn what_the_collapsed_bound_would_have_charged(
        cone: &Cone,
        at: &[[f64; 2]; 3],
        slop: f64,
    ) -> f64 {
        let widest = at
            .iter()
            .map(|corner| cone.radius_at(corner[1].abs()))
            .fold(0.0_f64, f64::max)
            + slop;
        let mut widest_turn = 0.0_f64;
        for (which, corner) in at.iter().enumerate() {
            for other in &at[(which + 1)..] {
                widest_turn = widest_turn.max((corner[0] - other[0]).abs());
            }
        }
        2.0 * widest * (widest_turn / 2.0).sin().powi(2) / cone.slope().hypot(1.0) + slop
    }

    /// [`HOW_FAR_THE_EVALUATOR_ROUNDS`] is a published bound, so here is the
    /// test that could fail (`docs/decisions.md` §15.6).
    ///
    /// Every other number in this file rests on it. A mesh built out of these
    /// points is never nearer its surface than they are, which is why a
    /// deflection under this floor is refused rather than chased, and why the
    /// deviation a patch reports carries it as a term.
    ///
    /// Measured the way a bound like this has to be: against how far out the
    /// surface *reaches* and not against its radius, because a ball of radius
    /// one a million units from the origin has coordinates of a million and
    /// rounds like a million.
    ///
    /// A ball, a pipe and a cone are one rotation each; a ring composes two,
    /// so it was the one arm of [`how_far_a_point_lands_off_its_surface`] this
    /// search had never actually reached — cone and torus landed after the
    /// evaluator's own bound was last measured. It comes in under the same
    /// constant with room to spare (worst seen: an eighth of it), so the extra
    /// rotation costs nothing this constant does not already cover.
    #[test]
    fn a_point_from_the_evaluator_lands_this_close_to_its_surface() {
        let mut worst = 0.0_f64;

        for radius in [1e-6, 1.0, 7.5, 1e6] {
            for out in [0.0, 1.0, 1e6] {
                let ball = Sphere::new(Point::new(out, out / 2.0, -out), radius).expect("a ball");
                let reach = ball.centre().largest_coordinate_magnitude() + radius;
                for round in -32..=32 {
                    for up in -16..=16 {
                        let point = ball.point_at(f64::from(round) * 0.1, f64::from(up) * 0.1);
                        let off = (point.distance_to(ball.centre()) - radius).abs();
                        worst = worst.max(off / reach);
                    }
                }

                let origin = Point::new(-out, out, out / 4.0);
                let axis = Axis::new(origin, Vector::new(0.0, 0.0, 1.0)).expect("an axis");
                let pipe = Cylinder::new(axis, radius).expect("a pipe");
                let cone = Cone::new(axis, radius).expect("a cone");
                for round in -32..=32 {
                    for along in [-3.0, 0.5, 3.0] {
                        let point = cone.point_at(f64::from(round) * 0.1, along);
                        let across = ((point.x() - origin.x()).powi(2)
                            + (point.y() - origin.y()).powi(2))
                        .sqrt();
                        let here = cone.radius_at(along);
                        let reach = origin.largest_coordinate_magnitude() + here + along.abs();
                        worst = worst.max((across - here).abs() / reach);
                    }
                }

                for round in -32..=32 {
                    for along in [-3.0, 0.0, 3.0] {
                        let point = pipe.point_at(f64::from(round) * 0.1, along);
                        let across = ((point.x() - origin.x()).powi(2)
                            + (point.y() - origin.y()).powi(2))
                        .sqrt();
                        let reach = origin.largest_coordinate_magnitude() + radius + along.abs();
                        worst = worst.max((across - radius).abs() / reach);
                    }
                }

                let ring = Torus::new(axis, radius * 3.0, radius).expect("a tube inside its ring");
                let reach =
                    origin.largest_coordinate_magnitude() + ring.ring_radius() + ring.tube_radius();
                for tube in -16..=16 {
                    for round in -16..=16 {
                        let point = ring.point_at(f64::from(tube) * 0.2, f64::from(round) * 0.2);
                        let radial = ((point.x() - origin.x()).powi(2)
                            + (point.y() - origin.y()).powi(2))
                        .sqrt();
                        let height = point.z() - origin.z();
                        let from_the_ring_circle = (radial - ring.ring_radius()).hypot(height);
                        let off = (from_the_ring_circle - ring.tube_radius()).abs();
                        worst = worst.max(off / reach);
                    }
                }
            }
        }

        assert!(
            worst <= HOW_FAR_THE_EVALUATOR_ROUNDS,
            "a point landed {worst} of how far its surface reaches off it, \
             and the bound is {HOW_FAR_THE_EVALUATOR_ROUNDS}"
        );
        // And it is a bound rather than a shrug: a constant ten times too big
        // would still pass the line above, and a floor ten times too high
        // refuses deflections a mesh could have met.
        assert!(
            worst * 10.0 >= HOW_FAR_THE_EVALUATOR_ROUNDS,
            "the worst point landed {worst} off and the bound allows \
             {HOW_FAR_THE_EVALUATOR_ROUNDS}, which is not a bound on anything"
        );
    }

    /// The band of torus wall that two quarter bends of ring 4 and tube 1 leave
    /// half a turn round from each other (Q1734): two meridian circles at one
    /// angle round the ring, each four corners a quarter of the tube apart, the
    /// outline walked up the tube angle and the hole down it, starting half a
    /// turn apart and read a whole turn apart. A quarter of a tube of 1 strays
    /// under 1, and well over a tenth.
    #[test]
    fn a_band_between_two_circles_at_one_angle_round_the_ring_is_zipped_up() {
        use core::f64::consts::{FRAC_PI_2, PI};
        let ring = Surface::from(Torus::new(Axis::z(), 4.0, 1.0).expect("a tube inside its ring"));
        let at_one_angle = |round_the_tube: [f64; 4]| {
            Loop::through(round_the_tube.map(|at| [at, FRAC_PI_2]).to_vec()).expect("four corners")
        };
        let outline = at_one_angle([PI, 1.5 * PI, 2.0 * PI, 2.5 * PI]);
        let hole = at_one_angle([0.0, -FRAC_PI_2, -PI, -1.5 * PI]);
        let deflection = |length| Deflection::of(length).expect("a positive length");

        let patch = zip_up_a_band(&ring, &outline, &hole, deflection(1.0)).expect("a band");
        assert_eq!(patch.triangles().len(), 8);
        each_rim_side_once_its_rims_way_and_each_rung_once_each_way(&patch, 4, "the band");

        let the_outlines_way = at_one_angle([0.0, FRAC_PI_2, PI, 1.5 * PI]);
        assert_eq!(
            zip_up_a_band(&ring, &outline, &the_outlines_way, deflection(1.0)),
            Err(CannotTriangulate::BoundaryCrossesItself { at: 6 })
        );
        let refused = zip_up_a_band(&ring, &outline, &hole, deflection(0.1));
        assert!(
            matches!(
                refused,
                Err(CannotTriangulate::BoundarySideStraysFurtherThanTheDeflection { side: 0, strays })
                    if strays > 0.1 && strays < 1.0
            ),
            "{refused:?}"
        );
    }

    /// [`zip_up_a_band`] says no triangle it draws reaches further round the
    /// tube than the widest side either rim has, and that is what makes each
    /// side's own walk enough to keep the triangles near the surface. So here
    /// is the search that could find one that does (`docs/decisions.md`
    /// §15.6): rims of three to seven corners each, the hole's first corner set
    /// off from the outline's by nothing, a little, most of a side, half a turn
    /// and back, at angles round the ring a rounding apart.
    #[test]
    fn no_zipped_triangle_reaches_further_round_than_the_widest_side_of_a_rim() {
        use core::f64::consts::{PI, TAU};
        let ring = Surface::from(Torus::new(Axis::z(), 4.0, 1.0).expect("a tube inside its ring"));
        let coarse = Deflection::of(10.0).expect("a positive length");
        for out in 3..=7 {
            for back in 3..=7 {
                for set_off in [0.0, 0.1, 1.0, PI, -2.5] {
                    let what = format!("{out} corners out, {back} back, set off {set_off}");
                    let round_the_tube = |count: usize, from: f64, way: f64| -> Vec<[f64; 2]> {
                        (0..count)
                            .map(|at| [from + way * TAU * at as f64 / count as f64, 0.3])
                            .collect()
                    };
                    let outline = round_the_tube(out, 0.4, 1.0);
                    let mut hole = round_the_tube(back, 0.4 + set_off, -1.0);
                    for corner in &mut hole {
                        corner[1] = 0.3_f64.next_up();
                    }
                    let patch = zip_up_a_band(
                        &ring,
                        &Loop::through(outline.clone()).expect("an outline"),
                        &Loop::through(hole.clone()).expect("a hole"),
                        coarse,
                    )
                    .unwrap_or_else(|why| panic!("{what}: {why}"));
                    each_rim_side_once_its_rims_way_and_each_rung_once_each_way(&patch, out, &what);

                    let widest = TAU / out.min(back) as f64;
                    let round: Vec<f64> = outline
                        .iter()
                        .chain(&hole)
                        .map(|corner| corner[0])
                        .collect();
                    for triangle in patch.triangles() {
                        let first = round[triangle[0] as usize];
                        let reach = triangle
                            .map(|corner| within_half_a_turn(round[corner as usize] - first));
                        let width = reach.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                            - reach.iter().copied().fold(f64::INFINITY, f64::min);
                        // Every corner is under 10 round the tube and was
                        // rounded a few times on the way to `width`.
                        assert!(
                            width <= widest + 8.0 * 10.0 * f64::EPSILON,
                            "{what}: triangle {triangle:?} reaches {width} round the tube, \
                             and the widest side is {widest}"
                        );
                    }
                }
            }
        }
    }

    /// That `patch`, from [`zip_up_a_band`] with an outline of `out` corners,
    /// uses each side of either rim once the way its rim runs, and each other
    /// side once each way, from a corner of one rim to a corner of the other.
    fn each_rim_side_once_its_rims_way_and_each_rung_once_each_way(
        patch: &Patch,
        out: usize,
        what: &str,
    ) {
        let (out, all) = (out as u32, patch.corners().len() as u32);
        let mut used: HashMap<(u32, u32), u32> = HashMap::new();
        for &[one, two, three] in patch.triangles() {
            for side in [(one, two), (two, three), (three, one)] {
                *used.entry(side).or_default() += 1;
            }
        }
        let the_next_round_its_rim = |corner: u32| match corner + 1 {
            next if next == out => 0,
            next if next == all => out,
            next => next,
        };
        for corner in 0..all {
            let side = (corner, the_next_round_its_rim(corner));
            assert_eq!(used.get(&side), Some(&1), "{what}: the side {side:?}");
        }
        for (&(from, to), &times) in &used {
            if to == the_next_round_its_rim(from) {
                continue;
            }
            assert!(
                (from < out) != (to < out),
                "{what}: the side from {from} to {to} joins two corners of one rim"
            );
            assert_eq!(
                (times, used.get(&(to, from))),
                (1, Some(&1)),
                "{what}: the rung from {from} to {to}"
            );
        }
    }
}
