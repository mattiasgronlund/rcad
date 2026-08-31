//! What a triangulated patch has to be true of, whatever shape went in.
//!
//! The oracle here needs no solid and no kernel: a polygon goes in and
//! triangles come out, and the claims are about the two of them together. That
//! is what `docs/decisions.md` §13 rule 1 asks of a piece that runs beside the
//! rest rather than after it.
//!
//! Four claims, and they are chosen so that between them nothing can be quietly
//! wrong. **Coverage**: the triangles add up to the area the loops enclose, so
//! nothing was dropped and nothing counted twice. **No new corners**: the
//! points that come out are the points that went in. **Every side is there**:
//! each boundary side is an edge of some triangle, so the mesh has not cut
//! across a boundary. **The same answer twice**: the same input gives the same
//! output, bit for bit, because a fuse property that compares two runs is worth
//! nothing if the mesher underneath is free to disagree with itself.

use std::collections::HashSet;

use rcad_geom::{Axis, Cylinder, Deflection, Plane, Point, Scale, Sphere, Surface, Vector};
use rcad_mesh::{CannotTriangulate, Loop, Patch, TrimmedDomain, triangulate};

fn flat() -> Surface {
    Surface::from(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal"),
    )
}

fn a_deflection() -> Deflection {
    Deflection::for_scale(Scale::millimetres(100.0).expect("a working size"))
}

fn domain_of(surface: Surface, outer: Vec<[f64; 2]>, holes: Vec<Vec<[f64; 2]>>) -> TrimmedDomain {
    TrimmedDomain::new(
        surface,
        Loop::through(outer).expect("an outer loop"),
        holes
            .into_iter()
            .map(|hole| Loop::through(hole).expect("a hole"))
            .collect(),
        a_deflection(),
    )
}

/// Twice the area a ring of corners encloses, by the shoelace sum.
///
/// Twice, and as a signed number, because doubling keeps every term of the sum
/// an exact product of two inputs for the whole-number corners these tests use,
/// so the comparison below is an equality and not a near-enough.
fn twice_the_area_of(ring: &[[f64; 2]]) -> f64 {
    let mut total = 0.0;
    for (index, corner) in ring.iter().enumerate() {
        let next = ring[(index + 1) % ring.len()];
        total += corner[0] * next[1] - next[0] * corner[1];
    }
    total
}

fn twice_the_area_of_the_triangles(patch: &Patch) -> f64 {
    patch
        .triangles()
        .iter()
        .map(|&[a, b, c]| {
            let (a, b, c) = (
                patch.corners()[a as usize],
                patch.corners()[b as usize],
                patch.corners()[c as usize],
            );
            ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs()
        })
        .sum()
}

/// Every side of every loop, as an unordered pair of corner indices.
fn every_side(outer: &[[f64; 2]], holes: &[Vec<[f64; 2]>]) -> HashSet<(u32, u32)> {
    let mut sides = HashSet::new();
    let mut first = 0u32;
    for ring in core::iter::once(outer).chain(holes.iter().map(Vec::as_slice)) {
        let count = u32::try_from(ring.len()).expect("a small ring");
        for step in 0..count {
            let (from, to) = (first + step, first + (step + 1) % count);
            sides.insert((from.min(to), from.max(to)));
        }
        first += count;
    }
    sides
}

fn sides_of_the_triangles(patch: &Patch) -> HashSet<(u32, u32)> {
    let mut sides = HashSet::new();
    for &[a, b, c] in patch.triangles() {
        for (from, to) in [(a, b), (b, c), (c, a)] {
            sides.insert((from.min(to), from.max(to)));
        }
    }
    sides
}

/// All four claims, over every shape in the corpus.
fn check_everything(name: &str, surface: Surface, outer: Vec<[f64; 2]>, holes: Vec<Vec<[f64; 2]>>) {
    let domain = domain_of(surface, outer.clone(), holes.clone());
    let patch = triangulate(&domain).unwrap_or_else(|why| panic!("{name} would not cut: {why}"));

    let expected: f64 = twice_the_area_of(&outer).abs()
        - holes
            .iter()
            .map(|h| twice_the_area_of(h).abs())
            .sum::<f64>();
    assert_eq!(
        twice_the_area_of_the_triangles(&patch),
        expected,
        "{name}: the triangles do not add up to the area the loops enclose"
    );

    let corners: usize = outer.len() + holes.iter().map(Vec::len).sum::<usize>();
    assert_eq!(
        patch.corners().len(),
        corners,
        "{name}: the triangulation added corners of its own"
    );

    let missing: Vec<_> = every_side(&outer, &holes)
        .difference(&sides_of_the_triangles(&patch))
        .copied()
        .collect();
    assert!(
        missing.is_empty(),
        "{name}: these boundary sides are not edges of any triangle: {missing:?}"
    );

    let again = triangulate(&domain).expect("and again");
    assert_eq!(patch, again, "{name}: two runs of the same input disagree");
}

#[test]
fn a_square_is_two_triangles_that_add_up_to_the_square() {
    check_everything(
        "a square",
        flat(),
        vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]],
        Vec::new(),
    );
}

/// The shape that catches a triangulation which fills the convex hull and stops.
#[test]
fn a_dented_outline_does_not_get_its_dent_filled_in() {
    check_everything(
        "an ell",
        flat(),
        vec![
            [0.0, 0.0],
            [6.0, 0.0],
            [6.0, 2.0],
            [2.0, 2.0],
            [2.0, 6.0],
            [0.0, 6.0],
        ],
        Vec::new(),
    );
}

#[test]
fn a_square_with_a_square_hole_keeps_the_hole() {
    check_everything(
        "a frame",
        flat(),
        vec![[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]],
        vec![vec![[2.0, 2.0], [2.0, 6.0], [6.0, 6.0], [6.0, 2.0]]],
    );
}

#[test]
fn two_holes_are_both_kept() {
    check_everything(
        "a plate with two holes",
        flat(),
        vec![[0.0, 0.0], [12.0, 0.0], [12.0, 8.0], [0.0, 8.0]],
        vec![
            vec![[2.0, 2.0], [2.0, 4.0], [4.0, 4.0], [4.0, 2.0]],
            vec![[8.0, 4.0], [8.0, 6.0], [10.0, 6.0], [10.0, 4.0]],
        ],
    );
}

/// A hole reaching out to touch the outline, refused.
///
/// **This test found a wrong answer, which is why it is written this way.** The
/// hole's corners at `[8, 3]` and `[8, 5]` sit on the outer loop's right-hand
/// side, so that side is split across them and the stretch between the two is
/// claimed by *both* loops while the triangulation records one boundary edge.
/// The walk counted one crossing where the boundary is two thick, decided the
/// hole was material, and handed back a patch covering it: an area of 128 where
/// the answer is 120.
///
/// It refuses now. A refusal is not the best possible answer here — counting
/// how many boundaries lie on an edge would let it be meshed — but it is an
/// honest one, and nothing in this milestone needs a hole that touches.
#[test]
fn a_hole_reaching_out_to_the_outline_is_refused() {
    let outer = vec![[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]];
    let hole = vec![[8.0, 3.0], [4.0, 4.0], [8.0, 5.0]];
    let refusal = triangulate(&domain_of(flat(), outer, vec![hole]));
    assert!(
        matches!(refusal, Err(CannotTriangulate::LoopsTouch { .. })),
        "got {refusal:?}"
    );
}

/// The rest of the touching family, refused by the same one check.
///
/// Sharing a corner outright, and resting a corner at a single point of
/// someone else's side, are the same problem to the walk. The second of those
/// would in fact be safe — nothing is doubled at a single point — and is
/// refused anyway; the refusal is asserted here so that the strictness is a
/// thing somebody chose rather than a thing that happens.
#[test]
fn the_other_ways_two_loops_can_touch_are_refused_too() {
    let outer = vec![[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]];
    let touching = [
        ("sharing a corner", vec![[4.0, 4.0], [4.0, 8.0], [8.0, 8.0]]),
        (
            "one corner resting on a side, and no more",
            vec![[2.0, 2.0], [8.0, 4.0], [2.0, 6.0]],
        ),
    ];

    for (what, hole) in touching {
        let refusal = triangulate(&domain_of(flat(), outer.clone(), vec![hole]));
        assert!(
            matches!(refusal, Err(CannotTriangulate::LoopsTouch { .. })),
            "{what}: got {refusal:?}"
        );
    }
}

/// The scaling is not decoration: an unscaled pipe of large radius is a strip
/// `2π` wide, and every triangle in it is a sliver.
///
/// What this asserts is that the answer is still *right* — the same four claims
/// hold on a domain whose two coordinates are wildly different in size, which
/// is the case the scaling exists for.
#[test]
fn a_patch_of_a_wide_pipe_comes_out_right() {
    for radius in [1.0, 1000.0, 1e-3] {
        let pipe = Cylinder::new(Axis::z(), radius).expect("a pipe");
        check_everything(
            &format!("a patch of a pipe of radius {radius}"),
            Surface::from(pipe),
            vec![[0.0, 0.0], [1.5, 0.0], [1.5, 2.0], [0.0, 2.0]],
            Vec::new(),
        );
    }
}

#[test]
fn a_patch_of_a_ball_comes_out_right() {
    let ball = Sphere::new(Point::origin(), 5.0).expect("a ball");
    check_everything(
        "a patch of a ball",
        Surface::from(ball),
        vec![[0.0, -1.0], [2.0, -1.0], [2.0, 1.0], [0.0, 1.0]],
        Vec::new(),
    );
}

#[test]
fn a_boundary_that_crosses_itself_is_refused_rather_than_guessed_at() {
    let bowtie = vec![[0.0, 0.0], [4.0, 4.0], [4.0, 0.0], [0.0, 4.0]];
    assert!(matches!(
        triangulate(&domain_of(flat(), bowtie, Vec::new())),
        Err(CannotTriangulate::BoundaryCrossesItself { .. })
    ));
}

#[test]
fn two_corners_in_the_same_place_are_refused() {
    let doubled = vec![[0.0, 0.0], [4.0, 0.0], [4.0, 0.0], [4.0, 4.0]];
    assert!(matches!(
        triangulate(&domain_of(flat(), doubled, Vec::new())),
        Err(CannotTriangulate::CornersCoincide { position: 2 })
    ));
}

/// The range spade will take is narrower than `f64`'s, and it is the **scaled**
/// coordinate that has to fit — so a small angle round an enormous pipe is a
/// large number by the time it gets there.
///
/// A refusal, which is the right way round. The alternative is a wrong answer.
#[test]
fn a_coordinate_too_large_to_triangulate_is_refused() {
    let enormous = Cylinder::new(Axis::z(), 1e300).expect("a pipe of radius 1e300");
    let refusal = triangulate(&domain_of(
        Surface::from(enormous),
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        Vec::new(),
    ));
    assert!(
        matches!(refusal, Err(CannotTriangulate::CoordinateOutOfRange { .. })),
        "got {refusal:?}"
    );
}

/// The same four claims over a corpus nobody hand-picked.
///
/// Every coordinate is a whole number, which is what lets the coverage claim be
/// an **equality** rather than a near-enough: each triangle's area is a cross
/// product of exact integer differences, and a sum of exact integers is exact.
/// A tolerance here would be a number the test was measuring instead of the
/// mesher (§14.1 question 1).
///
/// §13 rule 5 wants the generators across pieces to differ, and this one does:
/// the topology lane draws bars and balls, and this draws plates with holes
/// punched in them and never builds a solid at all.
mod a_plate_with_holes_punched_in_it {
    use super::*;
    use proptest::prelude::*;

    /// A plate `width` by `height` with `holes` cut out of it, each hole two
    /// units square and sitting on the grid so none of them touch.
    fn a_plate(
        width: i32,
        height: i32,
        holes: Vec<(i32, i32)>,
    ) -> (Vec<[f64; 2]>, Vec<Vec<[f64; 2]>>) {
        let outer = vec![
            [0.0, 0.0],
            [f64::from(width), 0.0],
            [f64::from(width), f64::from(height)],
            [0.0, f64::from(height)],
        ];
        let holes = holes
            .into_iter()
            .map(|(across, up)| {
                let (x, y) = (f64::from(across * 3 + 1), f64::from(up * 3 + 1));
                vec![[x, y], [x, y + 1.0], [x + 1.0, y + 1.0], [x + 1.0, y]]
            })
            .collect();
        (outer, holes)
    }

    proptest! {
        #[test]
        fn every_plate_is_cut_into_triangles_that_add_up_to_it(
            across in 1i32..4,
            up in 1i32..4,
            punched in prop::collection::hash_set((0i32..3, 0i32..3), 0..4),
        ) {
            let (width, height) = (across * 3 + 2, up * 3 + 2);
            let punched: Vec<_> = punched
                .into_iter()
                .filter(|&(x, y)| x * 3 + 2 < width && y * 3 + 2 < height)
                .collect();
            let (outer, holes) = a_plate(width, height, punched);
            check_everything("a punched plate", flat(), outer, holes);
        }
    }
}
