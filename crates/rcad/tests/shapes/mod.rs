//! The shapes the property suite is built on, and the flat-polygon arithmetic
//! that acts as its oracle.
//!
//! # Why these shapes and not random convex hulls
//!
//! A convex hull of random points is always valid and always convex, so it
//! never exercises a wire that turns back on itself or an inner loop — which is
//! exactly where the wire-orientation invariants bite. Convex hulls are
//! tempting because generating them is easy, and that ease is precisely why
//! they test nothing interesting (`docs/decisions.md` §9.1).
//!
//! What is generated instead is a **notched star**: corners at evenly spaced
//! angles about the origin, alternating between a long arm and a short notch,
//! with holes punched inside. Every notch corner is reflex, so the outline is
//! genuinely not convex, and there is always at least one inner loop.
//!
//! # Why one family of shapes is not enough
//!
//! The star's corners sit at strictly increasing angles about the origin, so
//! every edge stays inside its own narrow wedge and no two edges ever have
//! bounding boxes lying on top of each other. Axis-aligned rectangles have the
//! same property for a different reason. A self-crossing check that confused
//! "these two boxes overlap" with "these two edges cross" would refuse a plain
//! triangle and still pass every property here — which is exactly what happened.
//!
//! [`a_simple_polygon`] is the answer: a second and third family, chosen for
//! the shapes the star cannot make. Triangles, corners that lean instead of
//! turning square, adjacent edges that come within a whisker of parallel, and
//! edges whose boxes lie almost entirely on top of one another without the
//! edges ever touching.
//!
//! # What shrinks, and toward what
//!
//! This matters more than generation does (roadmap §11.3). A naive
//! `Vec<[f64; 2]>` strategy shrinks toward a polygon that is degenerate for an
//! uninteresting reason — three points in a line, a zero-length edge — and a
//! counterexample like that teaches nothing about the property that failed.
//!
//! So the strategy shrinks **parameters**, never coordinates. It generates
//! three things:
//!
//! - a `Vec<u32>` of arm extensions, one per *arm*. Its length is half the
//!   corner count, so proptest dropping elements shrinks the polygon toward the
//!   fewest arms; each element shrinks toward `0`, the shortest arm.
//! - how many holes there are, shrinking toward one.
//! - how big the holes are, shrinking toward the smallest.
//! - whether the holes are triangles, shrinking toward `false`, the square.
//!
//! The shrink target is therefore a regular four-armed star — eight corners,
//! every arm at [`SHORTEST_ARM`] and every notch at [`NOTCH_RADIUS`] — with one
//! smallest square hole in it. Small, drawable, and still carrying the reflex
//! corners and the inner loop that are the two things this shape exists to test.
//!
//! [`a_simple_polygon`] shrinks the same way: whole-number width, height, lean,
//! tooth count and shear, each toward its own floor. The triangle bottoms out at
//! the unit right triangle `[[0, 0], [1, 0], [0, 1]]` and the comb, with one
//! tooth and no lean, at the upright rectangle `[[0, 0], [1, 0], [1, 2], [0, 2]]`.
//!
//! Deliberately breaking three properties confirms it. "Always fail" shrinks to
//! exactly that four-armed star with one hole of half-size `0.25`. "Fail above
//! ten corners" shrinks to exactly twelve corners with everything else at its
//! floor — the boundary in the parameter, not a mangled polygon. "Fail when a
//! hole is wide" shrinks to the smallest hole that is still too wide. Each
//! reported counterexample is a sound simple polygon with a hole in it, which
//! is the whole point.
//!
//! # Why every parameter maps to a sound profile
//!
//! It is not checked at run time by filtering, which would throw away shrink
//! candidates and blunt the mechanism. It holds by construction:
//!
//! - Corners sit at strictly increasing angles about the origin with positive
//!   radii, so each edge stays inside its own angular wedge and no two
//!   non-neighbouring edges can meet. The outline is simple whatever the radii.
//! - The nearest the outline comes to the origin is [`NOTCH_RADIUS`], so the
//!   disc of that radius is wholly inside it. Every hole fits inside that disc,
//!   which puts every hole inside the outline without a test.
//! - The holes sit on a ring far enough apart that the largest of them still
//!   cannot touch its neighbour. A triangular hole is cut out of the same square
//!   its square counterpart fills, so both arguments above cover it unchanged.
//!
//! The two families in [`a_simple_polygon`] hold by construction too, and each
//! carries its own argument at the strategy that builds it.
//!
//! [`extruded`] therefore `expect`s rather than filters: a profile this builder
//! refuses is a bug here, not an input to be skipped.
//!
//! # Why the centroid is always inside the material
//!
//! One of the properties asserts that, and for a shape that is not convex it is
//! not a theorem — it is something this generator has to arrange.
//!
//! The corner count is always even, so an arm and a notch alternate all the way
//! round with no seam. The worst the centroid can be pulled off the origin is
//! then one arm at its longest and the rest at their shortest, which works out
//! at about `1.4` from the origin for the fewest corners and less for more of
//! them; the worst over several thousand generated shapes is `1.43`. Every hole
//! is at least `HOLE_RING_RADIUS - largest half size * sqrt(2)`, about `2.5`,
//! from the origin. So the centroid is inside the outline and outside every
//! hole, with room to spare.
//!
//! An earlier version allowed an odd corner count, which puts two long arms
//! side by side and pulls the centroid straight into the hole at that angle.
//! That is why the count is even.

// Shared between test binaries, and each one uses a different subset of it.
#![allow(dead_code)]

use proptest::prelude::*;
use rcad::{
    Axis, Isometry, Model, Point, Policy, Profile, ProfileBuilder, ProvenanceLog, Scale, SolidKey,
    Vector,
};

/// How close the outline comes to the origin, and so how big the disc is that
/// the holes have to fit inside.
pub const NOTCH_RADIUS: f64 = 4.5;

/// How far out the long arms start.
pub const SHORTEST_ARM: f64 = 8.0;

/// How much longer an arm can get than [`SHORTEST_ARM`].
pub const LONGEST_ARM_EXTENSION: f64 = 6.0;

/// How many steps an arm extension is generated in.
///
/// Whole numbers, so that shrinking walks down a grid and reaches `0` — the
/// shortest arm — rather than halving a float forever.
const ARM_STEPS: u32 = 60;

/// How far the middle of each hole sits from the origin.
pub const HOLE_RING_RADIUS: f64 = 3.2;

/// The smallest and largest half-width of a hole, in hundredths.
///
/// The upper end is what keeps a hole clear of both the outline and the
/// furthest the centroid can wander; see the module docs.
const HOLE_HALF_SIZES_IN_HUNDREDTHS: core::ops::RangeInclusive<u32> = 25..=50;

/// The most holes a generated profile has.
pub const MOST_HOLES: usize = 3;

/// The fewest corners a generated outline has.
///
/// Eight, not three. With fewer, the neighbours of a notch come round far
/// enough that the notch stops being reflex and the outline turns convex —
/// which is the one thing this generator exists not to produce.
pub const FEWEST_CORNERS: usize = 8;

/// The most corners a generated outline has.
pub const MOST_CORNERS: usize = 16;

/// The most teeth a generated comb has.
pub const MOST_TEETH: usize = 5;

/// How thick the base a comb's teeth stand on is.
const COMB_BASE_HEIGHT: f64 = 1.0;

/// Roughly how big a generated shape is, for [`Scale`] and for turning a
/// relative tolerance into an absolute one.
pub const WORKING_SIZE: f64 = 30.0;

/// A profile that is not convex, with between one and [`MOST_HOLES`] holes.
///
/// See the module docs for what shrinks and toward what.
pub fn a_notched_polygon_with_holes() -> impl Strategy<Value = Profile> {
    (
        proptest::collection::vec(0..=ARM_STEPS, FEWEST_CORNERS / 2..=MOST_CORNERS / 2),
        1usize..=MOST_HOLES,
        HOLE_HALF_SIZES_IN_HUNDREDTHS,
        any::<bool>(),
    )
        .prop_map(
            |(arm_extensions, hole_count, hole_size_in_hundredths, holes_are_triangles)| {
                notched_polygon_with_holes(
                    &arm_extensions,
                    hole_count,
                    f64::from(hole_size_in_hundredths) / 100.0,
                    holes_are_triangles,
                )
            },
        )
}

/// An outline that is simple whatever its parameters come out as, from a family
/// the notched star cannot reach.
///
/// See the module docs for why one family of shapes was not enough.
pub fn a_simple_polygon() -> impl Strategy<Value = Vec<[f64; 2]>> {
    prop_oneof![a_triangle(), a_sheared_comb()]
}

/// Corners at `(0, 0)`, `(width, 0)` and `(lean, height)`.
///
/// Three corners that are not all in one line always make a simple polygon, and
/// `height` is never zero, so every parameter here is sound. `lean` reaches well
/// past both ends of the base, so the apex overhangs and two of the three edges
/// come to lie almost entirely within each other's bounding box — the case a
/// star about the origin never produces. It shrinks to the unit right triangle.
fn a_triangle() -> impl Strategy<Value = Vec<[f64; 2]>> {
    (1i32..=10, 1i32..=10, -20i32..=20).prop_map(|(width, height, lean)| {
        vec![
            [0.0, 0.0],
            [f64::from(width), 0.0],
            [f64::from(lean), f64::from(height)],
        ]
    })
}

/// A comb of square teeth standing on a base, leaned over sideways.
///
/// Square-on it is plainly simple: each tooth stands on its own stretch of the
/// base with a gap either side. Leaning it over is a straight-line map of the
/// plane, which cannot make two edges cross that did not cross before, and with
/// whole numbers this small the arithmetic is exact in `f64` — so the leaned
/// comb is simple too, without anything having to check.
///
/// The lean is what this family is for. A tooth travels `shear * height`
/// sideways while staying one wide, so its edges end up sharing almost all of
/// their bounding box with the teeth either side of it without ever touching
/// them, and the top of a tooth runs within a whisker of parallel to the side it
/// meets. It shrinks to one upright tooth on the base, and no lean at all.
fn a_sheared_comb() -> impl Strategy<Value = Vec<[f64; 2]>> {
    (1usize..=MOST_TEETH, 1i32..=8, 0i32..=20).prop_map(|(teeth, tooth_height, shear)| {
        sheared_comb(teeth, f64::from(tooth_height), f64::from(shear))
    })
}

/// An arbitrary rigid movement: a turn about an arbitrary axis, then a shift.
///
/// The axis direction is a whole number vector so that generation cannot
/// produce a direction of no length, and it is normalized on the way into
/// [`Axis`] anyway.
pub fn an_arbitrary_movement() -> impl Strategy<Value = Isometry> {
    (
        -8i32..=8,
        -8i32..=8,
        -8i32..=8,
        0.0f64..core::f64::consts::TAU,
        -50.0f64..=50.0,
        -50.0f64..=50.0,
        -50.0f64..=50.0,
    )
        .prop_filter_map(
            "the axis direction has to point somewhere",
            |(i, j, k, angle, x, y, z)| {
                let direction = Vector::new(f64::from(i), f64::from(j), f64::from(k));
                let axis = Axis::new(Point::origin(), direction).ok()?;
                let turn = Isometry::rotation(&axis, angle).ok()?;
                let shift = Isometry::translation(Vector::new(x, y, z)).ok()?;
                Some(shift.compose(&turn))
            },
        )
}

/// A model holding just this profile, extruded straight up by `height`.
///
/// # Panics
///
/// If the extrusion is refused. The generator only produces profiles that
/// extrude, so that would be a bug in this module.
pub fn extruded(profile: &Profile, height: f64) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"));
    let solid = model
        .add_extrusion(profile, height, &Isometry::identity())
        .expect("every generated profile extrudes");
    (model, solid)
}

/// How much flat area the profile covers, holes taken out, by the shoelace
/// formula.
///
/// This is the independent half of the volume oracle: the kernel gets the same
/// number by summing signed tetrahedra over triangles in three dimensions, and
/// this gets it by summing cross products round two-dimensional rings. Two
/// different routes to one number.
///
/// [`Profile`] hands back its outline anticlockwise and its holes clockwise, so
/// summing the signed areas subtracts the holes without any special casing —
/// and a sum that came out negative would mean that canonicalization had
/// failed.
pub fn area_of(profile: &Profile) -> f64 {
    let mut total = signed_area_of(profile.outline());
    for hole in profile.holes() {
        total += signed_area_of(hole);
    }
    total
}

/// The total length of every loop of the profile, outline and holes alike.
pub fn perimeter_of(profile: &Profile) -> f64 {
    let mut total = length_round(profile.outline());
    for hole in profile.holes() {
        total += length_round(hole);
    }
    total
}

/// Whether flat point `[x, y]` is in the material, when it is far enough from
/// every edge for the answer not to be a close call.
///
/// `None` means the point is within `margin` of some edge. There is no honest
/// answer to give there, and inventing one would be testing this file's
/// arithmetic rather than the kernel's.
pub fn where_the_point_sits(profile: &Profile, point: [f64; 2], margin: f64) -> Option<Material> {
    if too_close_to_an_edge(profile.outline(), point, margin) {
        return None;
    }
    for hole in profile.holes() {
        if too_close_to_an_edge(hole, point, margin) {
            return None;
        }
    }

    if !ring_contains(profile.outline(), point) {
        return Some(Material::Empty);
    }
    for hole in profile.holes() {
        if ring_contains(hole, point) {
            return Some(Material::Empty);
        }
    }
    Some(Material::Solid)
}

/// Whether a flat point is in the material or not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Material {
    /// Inside the outline and outside every hole.
    Solid,
    /// Outside the outline, or inside one of the holes.
    Empty,
}

fn sheared_comb(teeth: usize, tooth_height: f64, shear: f64) -> Vec<[f64; 2]> {
    let top = COMB_BASE_HEIGHT + tooth_height;
    let span = (2 * teeth - 1) as f64;

    let mut corners = vec![[0.0, 0.0], [span, 0.0]];
    for tooth in (0..teeth).rev() {
        let left = 2.0 * tooth as f64;
        corners.push([left + 1.0, top]);
        corners.push([left, top]);
        if tooth > 0 {
            corners.push([left, COMB_BASE_HEIGHT]);
            corners.push([left - 1.0, COMB_BASE_HEIGHT]);
        }
    }

    for corner in &mut corners {
        corner[0] += shear * corner[1];
    }
    corners
}

fn notched_polygon_with_holes(
    arm_extensions: &[u32],
    hole_count: usize,
    hole_half_size: f64,
    holes_are_triangles: bool,
) -> Profile {
    let corners = arm_extensions.len() * 2;
    let outline: Vec<[f64; 2]> = (0..corners)
        .map(|index| {
            let angle = core::f64::consts::TAU * index as f64 / corners as f64;
            let radius = if index % 2 == 0 {
                let extension = arm_extensions[index / 2];
                SHORTEST_ARM + f64::from(extension) * (LONGEST_ARM_EXTENSION / f64::from(ARM_STEPS))
            } else {
                NOTCH_RADIUS
            };
            [radius * angle.cos(), radius * angle.sin()]
        })
        .collect();

    let mut builder = ProfileBuilder::new(outline);
    for index in 0..hole_count {
        let angle = core::f64::consts::TAU * index as f64 / hole_count as f64;
        let (x, y) = (
            HOLE_RING_RADIUS * angle.cos(),
            HOLE_RING_RADIUS * angle.sin(),
        );
        // Half of the same square, cut corner to corner. It stays inside every
        // bound the square had, so the module docs' argument that a hole sits
        // inside the outline and clear of its neighbours covers it without a
        // word changing — and a right triangle is the shape a self-crossing
        // check gets wrong first.
        builder = builder.with_hole(if holes_are_triangles {
            vec![
                [x - hole_half_size, y - hole_half_size],
                [x + hole_half_size, y - hole_half_size],
                [x - hole_half_size, y + hole_half_size],
            ]
        } else {
            vec![
                [x - hole_half_size, y - hole_half_size],
                [x + hole_half_size, y - hole_half_size],
                [x + hole_half_size, y + hole_half_size],
                [x - hole_half_size, y + hole_half_size],
            ]
        });
    }
    builder
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the generator only builds sound profiles")
}

fn signed_area_of(ring: &[[f64; 2]]) -> f64 {
    let mut twice_the_area = 0.0;
    for (index, start) in ring.iter().enumerate() {
        let end = ring[(index + 1) % ring.len()];
        twice_the_area += start[0] * end[1] - end[0] * start[1];
    }
    twice_the_area / 2.0
}

fn length_round(ring: &[[f64; 2]]) -> f64 {
    let mut total = 0.0;
    for (index, start) in ring.iter().enumerate() {
        let end = ring[(index + 1) % ring.len()];
        total += (end[0] - start[0]).hypot(end[1] - start[1]);
    }
    total
}

fn too_close_to_an_edge(ring: &[[f64; 2]], point: [f64; 2], margin: f64) -> bool {
    (0..ring.len()).any(|index| {
        let start = ring[index];
        let end = ring[(index + 1) % ring.len()];
        distance_to_segment(point, start, end) <= margin
    })
}

fn distance_to_segment(point: [f64; 2], start: [f64; 2], end: [f64; 2]) -> f64 {
    let along = [end[0] - start[0], end[1] - start[1]];
    let to_point = [point[0] - start[0], point[1] - start[1]];
    let length_squared = along[0] * along[0] + along[1] * along[1];
    let how_far =
        ((to_point[0] * along[0] + to_point[1] * along[1]) / length_squared).clamp(0.0, 1.0);
    let nearest = [start[0] + how_far * along[0], start[1] + how_far * along[1]];
    (point[0] - nearest[0]).hypot(point[1] - nearest[1])
}

/// Crossing count of a ray going in the `+x` direction, with the half-open rule
/// on `y` so that a corner is counted once rather than twice or not at all.
fn ring_contains(ring: &[[f64; 2]], point: [f64; 2]) -> bool {
    let mut inside = false;
    for (index, start) in ring.iter().enumerate() {
        let end = ring[(index + 1) % ring.len()];
        if (start[1] > point[1]) != (end[1] > point[1]) {
            let crossing_x =
                start[0] + (point[1] - start[1]) / (end[1] - start[1]) * (end[0] - start[0]);
            if crossing_x > point[0] {
                inside = !inside;
            }
        }
    }
    inside
}
