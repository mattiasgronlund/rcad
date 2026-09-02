//! Cutting a trimmed patch into triangles.

use std::collections::{HashMap, VecDeque};

use spade::{ConstrainedDelaunayTriangulation, InsertionError, Point2, Triangulation};

use rcad_geom::{Cone, Surface};
use rcad_predicates::{Approximation, Cause, Decision, orient2d};

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
/// # Errors
///
/// About the loops: [`CannotTriangulate::BoundaryCrossesItself`] when two
/// boundary edges cross, [`CannotTriangulate::CornersCoincide`] when two
/// corners are the same point, and
/// [`CannotTriangulate::CoordinateOutOfRange`] for a scaled coordinate the
/// triangulation will not take — see that variant, the range is narrower than
/// `f64`'s.
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
    refuse_loops_that_touch(&corners, domain)?;
    let surface = domain.surface();
    let (across, along) = how_this_domain_is_scaled(surface, &corners);

    let mut mesh = ConstrainedDelaunayTriangulation::<Point2<f64>>::new();
    let mut corner_of: Vec<u32> = Vec::new();
    let mut handles = Vec::with_capacity(corners.len());
    for (position, corner) in corners.iter().enumerate() {
        let handle = mesh
            .insert(Point2::new(corner[0] * across, corner[1] * along))
            .map_err(|why| CannotTriangulate::CoordinateOutOfRange { position, why })?;
        if handles.contains(&handle) {
            return Err(CannotTriangulate::CornersCoincide { position });
        }
        remember_which_corner(&mut corner_of, handle.index(), position);
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

    let deflection = domain.deflection().length();
    let slop = how_far_a_point_lands_off_its_surface(surface, &corners);
    let whole_patch = how_far_up_its_axis_this_reaches(&corners);
    let step = a_step_that_stays_within(surface, whole_patch, deflection, slop);
    if step <= 0.0 {
        return Err(CannotTriangulate::NoMeshCanGetThatCloseToTheSurface {
            deflection,
            closest: how_far_a_flat_piece_strays(surface, whole_patch, 0.0, slop),
        });
    }
    // Size first and the loops after, because how many triangles the deflection
    // asks for is a question about the deflection alone, and whether the loops
    // are fine enough is a question about both.
    refuse_a_patch_too_big_to_hold(domain, (across, along), step)?;
    for (from, to) in every_side(domain) {
        let across = how_far_up_its_axis_this_reaches(&[corners[from], corners[to]]);
        let chord = chord_across_the_curve(surface, across, corners[from], corners[to]);
        let strays = how_far_a_flat_piece_strays(surface, across, chord, slop);
        if strays > deflection {
            return Err(
                CannotTriangulate::BoundarySideStraysFurtherThanTheDeflection {
                    side: from,
                    strays,
                },
            );
        }
    }

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
        },
    )?;

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
            let [lowest, highest] = how_far_up_its_axis_this_reaches(corners);
            let reach = lowest.abs().max(highest.abs());
            HOW_FAR_THE_EVALUATOR_ROUNDS
                * (cone.apex().largest_coordinate_magnitude() + cone.radius_at(reach) + reach)
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
fn how_far_up_its_axis_this_reaches(corners: &[[f64; 2]]) -> [f64; 2] {
    let lowest = corners
        .iter()
        .map(|corner| corner[1])
        .fold(f64::INFINITY, f64::min);
    let highest = corners
        .iter()
        .map(|corner| corner[1])
        .fold(f64::NEG_INFINITY, f64::max);
    [lowest, highest]
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
fn how_this_domain_is_scaled(surface: &Surface, corners: &[[f64; 2]]) -> (f64, f64) {
    match surface {
        Surface::Cone(cone) => (
            how_wide_a_cone_is_over(cone, how_far_up_its_axis_this_reaches(corners)),
            cone.slope().hypot(1.0),
        ),
        Surface::Plane(_) | Surface::Sphere(_) | Surface::Cylinder(_) | _ => {
            surface.furthest_a_unit_step_goes()
        }
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
/// # Panics
///
/// On a surface this crate has not been taught — see
/// [`how_far_a_point_lands_off_its_surface`].
fn chord_across_the_curve(
    surface: &Surface,
    across: [f64; 2],
    from: [f64; 2],
    to: [f64; 2],
) -> f64 {
    match surface {
        Surface::Plane(_) => 0.0,
        Surface::Sphere(ball) => ball
            .point_at(from[0], from[1])
            .distance_to(ball.point_at(to[0], to[1])),
        // Twice the radius times the sine of half the turn, which is the chord
        // for any turn and not only for a short one.
        Surface::Cylinder(pipe) => 2.0 * pipe.radius() * ((from[0] - to[0]) / 2.0).sin().abs(),
        // A cone is flat along its rulings too, so this is the pipe's chord
        // measured on the widest circle the piece touches. **Measured there and
        // not at either end**, because that is the radius
        // [`how_far_a_flat_piece_strays`] divides by, and a chord read at a
        // smaller radius than the one it is charged against is a bound on
        // nothing.
        Surface::Cone(cone) => {
            2.0 * how_wide_a_cone_is_over(cone, across) * ((from[0] - to[0]) / 2.0).sin().abs()
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
/// # A cone gets `L²/(2R)` and its own derivation, because a sphere's does not
/// transfer
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
/// because a convex combination is at most its largest term. Take `k` to be the
/// corner with the **largest** radius, `R_c = maxᵢ rᵢ`. Measuring every chord on
/// the circle of that radius — which is what
/// [`chord_across_the_curve`] does for a cone — gives
/// `C² = 2R_c²(1 − cos Δ)`, so `rᵢ(1 − cos Δ) = rᵢC²/(2R_c²) ≤ C²/(2R_c)`, and
///
/// ```text
/// how far the piece strays  ≤  C² / (2 · R_c · √(1 + t²))  =  C² / (2R)
/// ```
///
/// with `R = R_c√(1 + t²)` the cone's radius of normal curvature at the widest
/// corner. That is the `share = 2` arm below, and unlike the sphere's it stays
/// finite **at the apex**: a triangle with a corner on the apex is charged
/// against its two other corners' radius, which is exactly where its chords were
/// measured.
///
/// It is loose by about a factor of four against a drawn case, the same way the
/// sphere's `L²/(3r)` is loose against a two-corner side, and
/// `how_far_a_flat_piece_of_a_cone_really_strays_stays_under_the_published_bound`
/// is the search §15.6 asks for.
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
    across: [f64; 2],
    longest_chord: f64,
    slop: f64,
) -> f64 {
    let (radius, share) = match surface {
        Surface::Plane(_) => return 0.0,
        Surface::Sphere(ball) => (ball.radius(), 3.0),
        Surface::Cylinder(pipe) => (pipe.radius(), 3.0),
        Surface::Cone(cone) => (
            how_wide_a_cone_is_over(cone, across) * cone.slope().hypot(1.0),
            2.0,
        ),
        other => panic!("this crate cannot say how far a flat piece of {other:?} strays"),
    };
    // **A patch spanning both nappes is bounded by nothing**, and infinity in
    // the denominator would say the opposite — `chord²/(share·∞)` is no stray at
    // all. `how_wide_a_cone_is_over` is where the infinity comes from and says
    // why it is the right answer *there*.
    if !radius.is_finite() {
        return f64::INFINITY;
    }
    // **A piece whose every corner is on the apex, which a cone has and no other
    // surface here does.** Cutting a cone's cap up to its apex leaves a side
    // running from the apex to the apex — one 3D point at two longitudes — and
    // the derivation above divides by the radius the chord was measured at,
    // which there is nothing at all. The answer is not the infinity that falls
    // out of the arithmetic: every corner of such a piece is within `slop` of
    // the apex, so every point of it is, and the apex is on the cone.
    if radius == 0.0 {
        return slop;
    }
    let chord = longest_chord + 2.0 * slop;
    chord * chord / (share * radius) + slop
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
    across: [f64; 2],
    deflection: f64,
    slop: f64,
) -> f64 {
    let (radius, share) = match surface {
        Surface::Plane(_) => return f64::INFINITY,
        Surface::Sphere(ball) => (ball.radius(), 3.0),
        Surface::Cylinder(pipe) => (pipe.radius(), 3.0),
        Surface::Cone(cone) => (
            how_wide_a_cone_is_over(cone, across) * cone.slope().hypot(1.0),
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
/// cutting across the same direction and leaves slivers behind it.
///
/// A loop is never cut: only an interior side is ever chosen. A triangle made
/// of nothing but boundary sides therefore has nothing to offer, and it almost
/// never needs to — its longest chord is a boundary chord, and those were all
/// measured against the deflection before any of this and passed.
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
/// mesh is aiming for so that it does not decide the answer: it declines points
/// that are no longer dividing anything. A round that places nothing is the end
/// of the road, and the answer there is a refusal rather than a mesh that
/// quietly missed.
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
            // against — see [`chord_across_the_curve`].
            let across = how_far_up_its_axis_this_reaches(
                &face
                    .vertices()
                    .map(|vertex| corners[corner_of[vertex.index()] as usize]),
            );
            let mut longest_chord = f64::NEG_INFINITY;
            let mut to_split: Option<(f64, [f64; 2], [f64; 2])> = None;
            for edge in face.adjacent_edges() {
                let from = corners[corner_of[edge.from().index()] as usize];
                let to = corners[corner_of[edge.to().index()] as usize];
                longest_chord =
                    longest_chord.max(chord_across_the_curve(aim.surface, across, from, to));
                if edge.is_constraint_edge() {
                    continue;
                }
                let (here, there) = (edge.from().position(), edge.to().position());
                let (dx, dy) = (there.x - here.x, there.y - here.y);
                let reach = dx.mul_add(dx, dy * dy);
                if to_split.is_none_or(|(longest, _, _)| reach > longest) {
                    to_split = Some((reach, from, to));
                }
            }

            let strays = how_far_a_flat_piece_strays(aim.surface, across, longest_chord, aim.slop);
            furthest = furthest.max(strays);
            if strays <= aim.deflection {
                continue;
            }
            anything_missed = true;

            // A triangle made of nothing but boundary sides has a boundary
            // chord for its longest, and every boundary side came in under the
            // deflection, so it is almost never over budget in the first place.
            // Almost: a loop may rest one of its own corners on one of its own
            // sides, and then the triangulation holds a constraint edge that is
            // a *piece* of a side — and a piece of a side more than half a turn
            // round a pipe has a longer chord than the whole of it. There is
            // nothing to cut there, and the refusal below is the answer.
            let Some((_, from, to)) = to_split else {
                continue;
            };
            wanted.push([(from[0] + to[0]) / 2.0, (from[1] + to[1]) / 2.0]);
        }

        if !anything_missed {
            return Ok(furthest);
        }

        let mut anything_placed = false;
        for corner in wanted {
            let at = Point2::new(corner[0] * aim.across, corner[1] * aim.along);
            if !placed.has_room_for(at) {
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
            return Err(CannotTriangulate::RanOutOfRoomBeforeMeetingTheDeflection {
                strays: furthest,
                deflection: aim.deflection,
            });
        }
    }
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
    #[error(
        "the mesh still strays {strays}, the deflection is {deflection}, and there is nowhere left to put a point"
    )]
    RanOutOfRoomBeforeMeetingTheDeflection {
        /// How far the mesh strays as it stands, in the model's own unit.
        strays: f64,
        /// What was asked for, in the same unit.
        deflection: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcad_geom::{Axis, Cylinder, Point, Sphere, Vector};

    /// [`how_far_a_flat_piece_strays`]'s cone arm is a published bound, so here
    /// is the search that could find something over it (`docs/decisions.md`
    /// §15.6).
    ///
    /// The claim is `C²/(2·R_c·√(1 + t²))` with `C` the longest chord measured
    /// on the circle of the piece's **widest** corner and `R_c = t·v_max` that
    /// corner's own radius. What is measured against it is how far a point of
    /// the flat triangle really sits off the cone, which for a point inside the
    /// nappe is `(t·p_∥ − ‖p_⊥‖)/√(1 + t²)` exactly — no sampling of the surface
    /// and no root-finding, because a cone is one of the few surfaces whose
    /// distance function is a closed form.
    ///
    /// The search is over eight slopes from a very shallow cone to a very steep
    /// one, over triangles spanning from a hundredth of a turn to most of a
    /// half turn, and over triangles that **reach the apex** — the case the
    /// sphere's `L²/(3r)` has no answer for at all, and the reason this arm
    /// exists. Each triangle is sampled on a barycentric grid.
    ///
    /// Over 240 triangles the worst uses **39.4%** of the bound, which is the
    /// ordinary gap between a worst case and a drawn one and about the same
    /// slack the sphere's own two-corner case carries. The lower assertion at
    /// the end is the other half of §15.6: a bound nothing ever comes near is a
    /// bound that proves nothing.
    #[test]
    fn how_far_a_flat_piece_of_a_cone_really_strays_stays_under_the_published_bound() {
        let mut worst_share = 0.0_f64;
        let mut checked = 0usize;

        for slope in [0.05, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 20.0] {
            let cone = Cone::new(Axis::z(), slope).expect("a positive slope");
            let surface = Surface::from(cone);
            for span in [0.01, 0.1, 0.5, 1.0, 1.6, 2.6] {
                for near in [0.0, 0.001, 0.1, 1.0, 4.0] {
                    let far = near + 1.0;
                    // A triangle with two corners on the far ring and one on the
                    // near one, which is what a mesh of a cone is made of, and
                    // which is degenerate at the apex when `near` is zero.
                    let at = [[0.0, far], [span, far], [span / 2.0, near]];
                    let corners = at.map(|[round, along]| cone.point_at(round, along));

                    let across = how_far_up_its_axis_this_reaches(&at);
                    let mut longest = 0.0_f64;
                    for one in 0..3 {
                        for other in (one + 1)..3 {
                            longest = longest
                                .max(chord_across_the_curve(&surface, across, at[one], at[other]));
                        }
                    }
                    let bound = how_far_a_flat_piece_strays(&surface, across, longest, 0.0);
                    if bound == 0.0 {
                        continue;
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
                        "a triangle on a cone of slope {slope} spanning {span} from {near} \
                         strays {furthest}, and the bound says {bound}"
                    );
                }
            }
        }

        println!("the worst of {checked} triangles used {worst_share} of its published bound");
        assert!(checked > 200, "the search covered {checked} triangles");

        // And a patch that spans both nappes is bounded by nothing at all,
        // because the derivation's `t·p_∥ = Σλᵢrᵢ` is false there. Nothing in
        // the kernel builds one; `SolidBuilder` is public, so a caller can
        // describe one, and what it gets is a refusal rather than a number.
        let cone = Cone::new(Axis::z(), 1.0).expect("a positive slope");
        let both_nappes = [[0.0, -1.0], [1.0, 1.0], [2.0, 2.0]];
        assert_eq!(
            how_far_a_flat_piece_strays(
                &Surface::from(cone),
                how_far_up_its_axis_this_reaches(&both_nappes),
                1.0,
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
}
