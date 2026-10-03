//! What a triangulated patch has to be true of, whatever shape went in.
//!
//! The oracle here needs no solid and no kernel: a polygon goes in and
//! triangles come out, and the claims are about the two of them together. That
//! is what `docs/decisions.md` §13 rule 1 asks of a piece that runs beside the
//! rest rather than after it.
//!
//! Five claims, and they are chosen so that between them nothing can be quietly
//! wrong.
//!
//! **Coverage**: the triangles add up to the area the loops enclose, so nothing
//! was dropped and nothing counted twice. Exactly, while every corner is a
//! whole number; once the refinement puts a midpoint in, that midpoint is a
//! rounded one and the sum over the pieces carries a rounding per triangle, so
//! the claim gets an error term and nothing else.
//!
//! **The loops come back as they went in**: the first corners out are the
//! corners in, in order and unmoved, and every corner after them is *strictly
//! inside* the patch — off every side of every loop. This used to read "no new
//! corners", which is no longer true of the inside and is still exactly true of
//! the boundary. It is the sharper claim of the two, because a mesher that
//! refined a shared edge would break watertightness at the layer above
//! (`docs/decisions.md` §25.1) and this is what would catch it.
//!
//! **Every side is there**: each boundary side is an edge of some triangle, so
//! the mesh has not cut across a boundary.
//!
//! **It is as close as it was asked to be**: the patch says how far it strays,
//! that number is at or under the deflection, and it blames the flat stand-in.
//! What makes it a claim rather than a restatement is that it is checked
//! against the **real surface** further down, and not against the bound.
//!
//! **The same answer twice**: the same input gives the same output, bit for
//! bit, because a fuse property that compares two runs is worth nothing if the
//! mesher underneath is free to disagree with itself.

use std::collections::HashSet;
use std::f64::consts::{FRAC_PI_2, TAU};

use rcad_geom::{Axis, Cylinder, Deflection, Plane, Point, Scale, Sphere, Surface, Vector};
use rcad_mesh::{CannotTriangulate, Loop, Patch, TrimmedDomain, triangulate};
use rcad_predicates::{Cause, Decision, orient2d};

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
    domain_at(a_deflection(), surface, outer, holes)
}

fn domain_at(
    deflection: Deflection,
    surface: Surface,
    outer: Vec<[f64; 2]>,
    holes: Vec<Vec<[f64; 2]>>,
) -> TrimmedDomain {
    TrimmedDomain::new(
        surface,
        Loop::through(outer).expect("an outer loop"),
        holes
            .into_iter()
            .map(|hole| Loop::through(hole).expect("a hole"))
            .collect(),
        deflection,
    )
}

/// A rectangle in the surface's own two coordinates, each side already walked
/// into `steps` corners.
///
/// The caller refines a boundary and this crate never does (`docs/decisions.md`
/// §25.1), so a test that wants a curved patch worth refining has to hand one
/// over the way a caller would: a loop already fine enough that no single side
/// of it strays further than the deflection on its own.
fn a_walked_rectangle(u: [f64; 2], v: [f64; 2], steps: usize) -> Vec<[f64; 2]> {
    let mut ring = Vec::new();
    let mut walk = |from: [f64; 2], to: [f64; 2]| {
        for step in 0..steps {
            let how_far = step as f64 / steps as f64;
            ring.push([
                from[0] + (to[0] - from[0]) * how_far,
                from[1] + (to[1] - from[1]) * how_far,
            ]);
        }
    };
    walk([u[0], v[0]], [u[1], v[0]]);
    walk([u[1], v[0]], [u[1], v[1]]);
    walk([u[1], v[1]], [u[0], v[1]]);
    walk([u[0], v[1]], [u[0], v[0]]);
    ring
}

/// Twice the area a ring of corners encloses, by the shoelace sum.
///
/// Twice, and as a signed number, because doubling keeps every term of the sum
/// an exact product of two inputs for the whole-number corners many of these
/// tests use, so the comparison below is an equality and not a near-enough.
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
///
/// Indices are put through [`the_first_corner_in_each_place`] first, because
/// two loops are allowed to meet at a point and a point the mesh holds once
/// cannot be two indices to a triangle. That is the identity map for every
/// shape here but the pinched ones.
fn every_side(outer: &[[f64; 2]], holes: &[Vec<[f64; 2]>]) -> HashSet<(u32, u32)> {
    let corners: Vec<[f64; 2]> = outer
        .iter()
        .chain(holes.iter().flatten())
        .copied()
        .collect();
    let stands_for = the_first_corner_in_each_place(&corners);
    let mut sides = HashSet::new();
    let mut first = 0u32;
    for ring in core::iter::once(outer).chain(holes.iter().map(Vec::as_slice)) {
        let count = u32::try_from(ring.len()).expect("a small ring");
        for step in 0..count {
            let from = stands_for[(first + step) as usize];
            let to = stands_for[(first + (step + 1) % count) as usize];
            sides.insert((from.min(to), from.max(to)));
        }
        first += count;
    }
    sides
}

/// For each corner, the first corner standing in the same place — itself,
/// unless an earlier loop already named that place.
fn the_first_corner_in_each_place(corners: &[[f64; 2]]) -> Vec<u32> {
    corners
        .iter()
        .enumerate()
        .map(|(index, corner)| {
            let first = corners
                .iter()
                .position(|earlier| earlier == corner)
                .unwrap_or(index);
            u32::try_from(first).expect("a small domain")
        })
        .collect()
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

/// Whether a corner sits inside the outer loop, out of every hole, and on no
/// side of either.
fn is_strictly_inside(outer: &[[f64; 2]], holes: &[Vec<[f64; 2]>], corner: [f64; 2]) -> bool {
    if sits_on(outer, corner) || !is_wrapped_by(outer, corner) {
        return false;
    }
    holes
        .iter()
        .all(|hole| !sits_on(hole, corner) && !is_wrapped_by(hole, corner))
}

fn sits_on(ring: &[[f64; 2]], corner: [f64; 2]) -> bool {
    ring.iter().enumerate().any(|(index, from)| {
        let to = ring[(index + 1) % ring.len()];
        let off_the_line =
            (to[0] - from[0]) * (corner[1] - from[1]) - (to[1] - from[1]) * (corner[0] - from[0]);
        off_the_line == 0.0
            && (0..2).all(|axis| {
                corner[axis] >= from[axis].min(to[axis]) && corner[axis] <= from[axis].max(to[axis])
            })
    })
}

/// A ray going out in `+u`, counting the sides it crosses.
fn is_wrapped_by(ring: &[[f64; 2]], corner: [f64; 2]) -> bool {
    let mut crossings = 0;
    for (index, from) in ring.iter().enumerate() {
        let to = ring[(index + 1) % ring.len()];
        if (from[1] > corner[1]) == (to[1] > corner[1]) {
            continue;
        }
        let at = from[0] + (corner[1] - from[1]) / (to[1] - from[1]) * (to[0] - from[0]);
        if at > corner[0] {
            crossings += 1;
        }
    }
    crossings % 2 == 1
}

/// All five claims, over every shape in the corpus.
fn check_everything(name: &str, domain: &TrimmedDomain) {
    let outer = domain.outer().corners().to_vec();
    let holes: Vec<Vec<[f64; 2]>> = domain
        .holes()
        .iter()
        .map(|hole| hole.corners().to_vec())
        .collect();
    let went_in: Vec<[f64; 2]> = outer
        .iter()
        .chain(holes.iter().flatten())
        .copied()
        .collect();
    let patch = triangulate(domain).unwrap_or_else(|why| panic!("{name} would not cut: {why}"));

    let expected: f64 = twice_the_area_of(&outer).abs()
        - holes
            .iter()
            .map(|hole| twice_the_area_of(hole).abs())
            .sum::<f64>();
    // Exact while every corner is a whole number: each triangle's doubled area
    // is then a difference of exact integer products and a sum of exact
    // integers is exact. Otherwise every one of the triangles contributes a
    // term of at most the whole area, the running sum rounds once per term, and
    // a midpoint the refinement put in is itself a rounded point off the edge
    // it split — a few epsilons of the area per triangle, and no more.
    let all_whole = patch
        .corners()
        .iter()
        .all(|corner| corner.iter().all(|value| value.fract() == 0.0));
    let slack = if all_whole {
        0.0
    } else {
        8.0 * expected * f64::EPSILON * patch.triangles().len() as f64
    };
    let covered = twice_the_area_of_the_triangles(&patch);
    assert!(
        (covered - expected).abs() <= slack,
        "{name}: the triangles cover {covered} where the loops enclose {expected}"
    );

    assert_eq!(
        &patch.corners()[..went_in.len()],
        &went_in[..],
        "{name}: the loops did not come back the way they went in"
    );
    for extra in &patch.corners()[went_in.len()..] {
        assert!(
            is_strictly_inside(&outer, &holes, *extra),
            "{name}: the refinement put {extra:?} somewhere other than strictly inside"
        );
    }

    let missing: Vec<_> = every_side(&outer, &holes)
        .difference(&sides_of_the_triangles(&patch))
        .copied()
        .collect();
    assert!(
        missing.is_empty(),
        "{name}: these boundary sides are not edges of any triangle: {missing:?}"
    );

    assert_eq!(
        patch.deviation().cause(),
        Cause::FlatStandIn,
        "{name}: a mesh's slop is a flat piece standing in for a curved one"
    );
    assert!(
        patch.deviation().magnitude() <= domain.deflection().length(),
        "{name}: strays {} where {} was asked for",
        patch.deviation().magnitude(),
        domain.deflection().length()
    );

    let again = triangulate(domain).expect("and again");
    assert_eq!(
        &patch, &again,
        "{name}: two runs of the same input disagree"
    );
}

#[test]
fn a_square_is_two_triangles_that_add_up_to_the_square() {
    check_everything(
        "a square",
        &domain_of(
            flat(),
            vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]],
            Vec::new(),
        ),
    );
}

/// The shape that catches a triangulation which fills the convex hull and stops.
#[test]
fn a_dented_outline_does_not_get_its_dent_filled_in() {
    check_everything(
        "an ell",
        &domain_of(
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
        ),
    );
}

#[test]
fn a_square_with_a_square_hole_keeps_the_hole() {
    check_everything(
        "a frame",
        &domain_of(
            flat(),
            vec![[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]],
            vec![vec![[2.0, 2.0], [2.0, 6.0], [6.0, 6.0], [6.0, 2.0]]],
        ),
    );
}

#[test]
fn two_holes_are_both_kept() {
    check_everything(
        "a plate with two holes",
        &domain_of(
            flat(),
            vec![[0.0, 0.0], [12.0, 0.0], [12.0, 8.0], [0.0, 8.0]],
            vec![
                vec![[2.0, 2.0], [2.0, 4.0], [4.0, 4.0], [4.0, 2.0]],
                vec![[8.0, 4.0], [8.0, 6.0], [10.0, 6.0], [10.0, 4.0]],
            ],
        ),
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

/// The refusal says **where** the two loops touch, and not only which corner
/// numbers touched.
///
/// A corner number is an address into the list of corners the caller handed
/// over, which is fine for the caller that built it and useless to anyone
/// further up: the kernel's mesher assembles that list itself out of walked
/// wires, so its own caller never saw it. A point on the surface is a thing
/// anybody holding the domain can find. `docs/decisions.md` §123.
///
/// The two corners on the outline here are `[8, 3]` and `[8, 5]`, and which of
/// them the sweep meets first is the order the sides come out in and nothing a
/// caller should depend on — so both are allowed, and each has to be exact. A
/// plane's own two coordinates are not `x` and `y`, so the two places are asked
/// of the surface rather than written out: what is claimed is that the point is
/// the touching **corner** and not some other corner.
#[test]
fn a_hole_reaching_out_to_the_outline_says_where_it_touches() {
    let outer = vec![[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]];
    let hole = vec![[8.0, 3.0], [4.0, 4.0], [8.0, 5.0]];
    let refusal = triangulate(&domain_of(flat(), outer, vec![hole]));
    let Err(CannotTriangulate::LoopsTouch { at, .. }) = refusal else {
        panic!("got {refusal:?}")
    };
    let on_the_outline = [flat().point_at(8.0, 3.0), flat().point_at(8.0, 5.0)];
    assert!(
        on_the_outline.contains(&at),
        "the touch is at a corner of the hole, not at {at:?}"
    );
}

/// The rest of the landing-inside family, refused by the same one check.
///
/// A corner resting at a single point in the middle of somebody else's side
/// doubles nothing, and the patch it gives is right — `[2, 2], [8, 4], [2, 6]`
/// came back covering 104 of the outline's 128, which is the answer. It is
/// refused anyway, and not out of timidity: the triangulation splits the outer
/// side at that corner, so a side the caller handed over whole comes back as
/// two edges. This crate sees one face at a time and the face on the other side
/// of that side gets no such split, so what looks like a harmless extra vertex
/// is a crack between two faces (`docs/decisions.md` §25.1).
///
/// The second shape is the one that gets past every corner test and still has
/// to go: both its ends are corners the outline already names, and the stretch
/// between them is the whole of the outline's own side.
#[test]
fn the_other_ways_a_loop_can_land_on_another_loops_side_are_refused_too() {
    let outer = vec![[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]];
    let touching = [
        (
            "one corner resting in the middle of a side, and no more",
            vec![[2.0, 2.0], [8.0, 4.0], [2.0, 6.0]],
        ),
        (
            "a hole naming one of the outline's own sides, end for end",
            vec![[8.0, 8.0], [0.0, 8.0], [4.0, 4.0]],
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

/// A hole meeting the outline at a corner they both name is meshed, hole and
/// all.
///
/// **This is the shape the `LED_*_Horizontal_*` files of the STEP corpus stop
/// at.** The round pin is drawn tangent to a flat of the body it stands in, the
/// file writes a vertex at the tangency point, and both the flat's outline and
/// the pin's circle run through it — so the two loops hold one point in common
/// and no stretch at all. Reduced to whole numbers here, because the claim is
/// about the shape and the corpus file is the corroboration.
///
/// It used to refuse, and the refusal was deliberate: `refuse_loops_that_touch`
/// said in as many words that a single-point touch is harmless and was turning
/// it away because nothing needed it. What makes letting it through safe is
/// that both loops already name the point, so nothing is split and no
/// constrained edge ends up belonging to two loops — the crossing count in
/// `walk_in_from_the_outside` stays right.
///
/// That is a claim about the answer and not about the refusal, so it is checked
/// as one: `check_everything`'s coverage claim is **exact** here, every corner
/// being a whole number, and a patch that filled the hole in would report 128
/// where the answer is 112.
#[test]
fn a_hole_meeting_the_outline_at_a_corner_they_both_name_is_meshed() {
    check_everything(
        "a hole sharing one corner with the outline and nothing else",
        &domain_of(
            flat(),
            vec![[0.0, 0.0], [8.0, 0.0], [8.0, 8.0], [0.0, 8.0]],
            vec![vec![[8.0, 8.0], [4.0, 4.0], [6.0, 2.0]]],
        ),
    );
}

/// A plane is flat, a triangle is flat, and standing one in for the other loses
/// nothing at all.
///
/// So a plane is never refined however fine the deflection, and the number it
/// reports is **exactly** zero rather than something small. A plane that came
/// back with a corner it was not given, or with any deviation at all, would be
/// a bug in the arm that is supposed to be the easy one.
#[test]
fn a_plane_is_never_refined_and_never_strays() {
    let outer = vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
    for length in [1e6, 1.0, 1e-9, f64::MIN_POSITIVE] {
        let deflection = Deflection::of(length).expect("a positive length");
        let patch = triangulate(&domain_at(deflection, flat(), outer.clone(), Vec::new()))
            .expect("a square of a plane");
        assert_eq!(
            patch.corners(),
            &outer[..],
            "asked to be within {length} of a plane, and it moved"
        );
        assert_eq!(patch.deviation().magnitude(), 0.0);
        assert_eq!(patch.deviation().cause(), Cause::FlatStandIn);
    }
}

/// A disc of 40 corners round a circle, turned by `turn` and moved by `(3, −5)`.
fn a_disc_turned_and_moved(turn: f64) -> Vec<[f64; 2]> {
    (0..40)
        .map(|step| {
            let angle = TAU * f64::from(step) / 40.0 + turn;
            [3.0 + 2.0 * angle.cos(), -5.0 + 2.0 * angle.sin()]
        })
        .collect()
}

fn every_triangle_turns_anticlockwise(name: &str, patch: &Patch) {
    for &triangle in patch.triangles() {
        let [a, b, c] = triangle.map(|corner| patch.corners()[corner as usize]);
        assert_eq!(
            orient2d(a, b, c),
            Decision::Pos,
            "{name}: the triangle {triangle:?} does not turn anticlockwise"
        );
    }
}

/// The points round a circle are a near-tie for the Delaunay test in any four,
/// so the same disc turned and moved is cut into other triangles. Asked to be
/// cut by the order of its corners, it is cut the same every time
/// (`docs/decisions.md` §379.5).
#[test]
fn a_disc_cut_by_corner_order_is_cut_the_same_however_it_is_turned_and_moved() {
    let turns = (0..12).map(|step| 0.3 * f64::from(step) + 0.01);
    let by_delaunay: Vec<Vec<[u32; 3]>> = turns
        .clone()
        .map(|turn| {
            let domain = domain_of(flat(), a_disc_turned_and_moved(turn), Vec::new());
            let mut triangles = triangulate(&domain).expect("a disc").triangles().to_vec();
            for triangle in &mut triangles {
                triangle.sort_unstable();
            }
            triangles.sort_unstable();
            triangles
        })
        .collect();
    let cut_differently = by_delaunay
        .iter()
        .filter(|triangles| **triangles != by_delaunay[0])
        .count();
    println!(
        "the Delaunay test cut {cut_differently} of 11 turned discs differently from the first"
    );
    assert!(
        cut_differently > 0,
        "the Delaunay test cut every turned disc the same, so this tells corner order from it \
         by nothing"
    );

    let mut by_corner_order = Vec::new();
    for turn in turns {
        let name = format!("a disc turned by {turn}, cut by corner order");
        let domain =
            domain_of(flat(), a_disc_turned_and_moved(turn), Vec::new()).cut_by_corner_order();
        check_everything(&name, &domain);
        let patch = triangulate(&domain).expect("checked just above");
        every_triangle_turns_anticlockwise(&name, &patch);
        by_corner_order.push(patch.triangles().to_vec());
    }
    assert!(
        by_corner_order
            .iter()
            .all(|triangles| *triangles == by_corner_order[0]),
        "cut by corner order, the turned discs still came out in different triangles"
    );
}

/// An outline running clockwise is cut into triangles that turn anticlockwise,
/// like every other patch's.
#[test]
fn a_clockwise_disc_cut_by_corner_order_turns_every_triangle_anticlockwise() {
    let mut disc = a_disc_turned_and_moved(0.0);
    disc.reverse();
    let domain = domain_of(flat(), disc, Vec::new()).cut_by_corner_order();
    check_everything("a clockwise disc", &domain);
    every_triangle_turns_anticlockwise(
        "a clockwise disc",
        &triangulate(&domain).expect("checked just above"),
    );
}

/// Corner order cuts only a flat patch with no hole and an outline that turns
/// one way at every corner. Every other patch asked for it is cut as though it
/// had not been asked.
#[test]
fn a_patch_corner_order_cannot_cut_is_cut_as_though_it_had_not_been_asked() {
    let disc = a_disc_turned_and_moved(0.0);
    let hole: Vec<[f64; 2]> = a_disc_turned_and_moved(0.0)
        .into_iter()
        .rev()
        .map(|[u, v]| [3.0 + (u - 3.0) / 4.0, -5.0 + (v + 5.0) / 4.0])
        .collect();
    let five_petals: Vec<[f64; 2]> = (0..60)
        .map(|step| {
            let angle = TAU * f64::from(step) / 60.0;
            let reach = 2.0 + 0.6 * (5.0 * angle).cos();
            [reach * angle.cos(), reach * angle.sin()]
        })
        .collect();
    let with_a_straight_corner = vec![[0.0, 0.0], [2.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
    let on_a_pipe = Surface::from(Cylinder::new(Axis::z(), 2.0).expect("a pipe"));
    for (name, surface, outer, holes) in [
        ("a disc with a hole", flat(), disc.clone(), vec![hole]),
        ("five petals", flat(), five_petals, Vec::new()),
        (
            "a corner that does not turn",
            flat(),
            with_a_straight_corner,
            Vec::new(),
        ),
        (
            "a patch of a pipe",
            on_a_pipe,
            a_walked_rectangle([0.0, 1.0], [0.0, 1.0], 8),
            Vec::new(),
        ),
    ] {
        let asked = domain_of(surface, outer.clone(), holes.clone()).cut_by_corner_order();
        check_everything(name, &asked);
        assert_eq!(
            triangulate(&asked),
            triangulate(&domain_of(surface, outer, holes)),
            "{name}: asked to be cut by corner order, it was cut otherwise than unasked"
        );
    }
}

/// The scaling is not decoration: an unscaled pipe of large radius is a strip
/// `2π` wide, and every triangle in it is a sliver.
///
/// What this asserts is that the answer is still *right* — the five claims hold
/// on a domain whose two coordinates are wildly different in size, which is the
/// case the scaling exists for. The steps differ per radius because the loop is
/// the caller's to refine and a caller would refine it by the same rule: enough
/// corners that no one side is already too far off the pipe.
#[test]
fn a_patch_of_a_wide_pipe_comes_out_right() {
    for (radius, steps) in [(1.0, 4), (1000.0, 90), (1e-3, 1)] {
        let pipe = Cylinder::new(Axis::z(), radius).expect("a pipe");
        check_everything(
            &format!("a patch of a pipe of radius {radius}"),
            &domain_of(
                Surface::from(pipe),
                a_walked_rectangle([0.0, 1.5], [0.0, 2.0], steps),
                Vec::new(),
            ),
        );
    }
}

#[test]
fn a_patch_of_a_ball_comes_out_right() {
    let ball = Sphere::new(Point::origin(), 5.0).expect("a ball");
    check_everything(
        "a patch of a ball",
        &domain_of(
            Surface::from(ball),
            a_walked_rectangle([0.0, 2.0], [-1.0, 1.0], 10),
            Vec::new(),
        ),
    );
}

/// A ball's one face, in the shape `docs/decisions.md` §25.6 settled: a
/// rectangle in `(u, v)` whose top and bottom sides are each one point in 3D.
#[test]
fn a_patch_of_a_ball_that_reaches_both_poles_comes_out_right() {
    let ball = Sphere::new(Point::new(1.0, 2.0, 3.0), 2.0).expect("a ball");
    check_everything(
        "a whole ball",
        &domain_at(
            Deflection::of(0.1).expect("a tenth"),
            Surface::from(ball),
            a_walked_rectangle([0.0, TAU], [-FRAC_PI_2, FRAC_PI_2], 12),
            Vec::new(),
        ),
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

/// Spade's floor: it takes no coordinate under this that is not zero.
fn spade_floor() -> f64 {
    2f64.powi(-142)
}

/// What `triangulate` answers when it cannot show that flushing the corner at
/// `position` to zero left the patch as it was: the refusal it gave before it
/// flushed anything.
fn the_refusal_before_the_flush(position: usize) -> Result<Patch, CannotTriangulate> {
    Err(CannotTriangulate::CoordinateOutOfRange {
        position,
        why: spade::InsertionError::TooSmall,
    })
}

/// The bottom cap of a round bar of radius two moved by `2⁻¹⁰⁰⁷` along `v`, as
/// `docs/decisions.md` §246 found it: read back from a file, the move is in the
/// corners, and spade refused the corner at `v = 2⁻¹⁰⁰⁷` as too small.
///
/// Flushed to zero, that corner is exactly the unmoved disc's, and so is every
/// other corner, because `2 sin θ + 2⁻¹⁰⁰⁷` rounds to `2 sin θ` wherever
/// `sin θ` is not zero. So the triangles are the unmoved disc's too, index for
/// index, and the patch still hands back the corners it was given.
#[test]
fn a_disc_moved_by_two_to_the_minus_1007_is_cut_as_the_disc_that_was_not_moved() {
    let moved_by = f64::from_bits(0x0100_0000_0000_0000);
    assert_eq!(moved_by, 2f64.powi(-1007));
    let a_disc = |moved_by: f64| -> Vec<[f64; 2]> {
        (0..24)
            .map(|step| {
                let angle = TAU * f64::from(step) / 24.0;
                [2.0 * angle.cos(), 2.0 * angle.sin() + moved_by]
            })
            .collect()
    };
    let moved = a_disc(moved_by);
    assert!(
        moved
            .iter()
            .any(|corner| corner[1] != 0.0 && corner[1].abs() < spade_floor()),
        "no corner of the moved disc is under spade's floor, so this tests nothing"
    );

    let still = triangulate(&domain_of(flat(), a_disc(0.0), Vec::new()))
        .expect("the disc that was not moved");
    check_everything(
        "a disc moved by 2^-1007",
        &domain_of(flat(), moved.clone(), Vec::new()),
    );
    let cut = triangulate(&domain_of(flat(), moved, Vec::new())).expect("checked just above");
    assert_eq!(cut.triangles(), still.triangles());
}

/// A thin triangle whose third corner the flush carries across the line of
/// the other two. Kept as spade drew it, it would come back turned over.
///
/// The line from `left` to `right` crosses `u = 0` at `v = 2⁻¹⁶¹`. The third
/// corner sits above it at `2⁻¹⁵⁰`, and flushed to zero it sits below it, so
/// the three corners turn one way as asked for and the other way as spade
/// sees them. The corners are at the same place on both sides and the
/// triangle spade draws has the same three, the other way round.
#[test]
fn a_triangle_the_flush_turns_over_is_refused() {
    let left = [-1.0, 2f64.powi(-140) + 2f64.powi(-160)];
    let right = [1.0, -2f64.powi(-140)];
    let middle = [0.0, 2f64.powi(-150)];
    assert!(middle[1] < spade_floor());
    assert_eq!(orient2d(left, right, middle), Decision::Pos);
    assert_eq!(
        orient2d(left, right, [0.0, 0.0]),
        Decision::Neg,
        "flushed, the third corner is on the other side of the line"
    );

    assert_eq!(
        triangulate(&domain_of(flat(), vec![left, right, middle], Vec::new())),
        the_refusal_before_the_flush(2)
    );
}

/// The same thin triangle as a hole in a square. Every triangle that is kept
/// still turns the right way; the one inside the hole is turned over, and
/// what that means for the kept ones is that the hole's own sliver is covered
/// twice, by the triangle above its long side and by the ones below its two
/// short ones.
#[test]
fn a_hole_the_flush_turns_inside_out_is_refused() {
    let hole = vec![
        [-1.0, 2f64.powi(-140) + 2f64.powi(-160)],
        [1.0, -2f64.powi(-140)],
        [0.0, 2f64.powi(-150)],
    ];
    assert_eq!(
        triangulate(&domain_of(
            flat(),
            vec![[-4.0, -4.0], [4.0, -4.0], [4.0, 4.0], [-4.0, 4.0]],
            vec![hole],
        )),
        the_refusal_before_the_flush(6)
    );
}

/// A corner of one hole `2⁻¹⁵⁰` above a side of another. Flushed, it lands on
/// that side, and spade splits the side there: the patch would draw it as two
/// sides meeting at a corner of the other hole, while the face on the other
/// side of that edge draws it whole.
///
/// Every triangle still turns the right way and the corner is nowhere near the
/// hull, so the one thing that catches it is that the side is not there.
#[test]
fn a_corner_flushed_onto_another_loops_side_is_refused() {
    let below = vec![[-1.0, 0.0], [0.0, -1.0], [1.0, 0.0]];
    let above = vec![[0.0, 2f64.powi(-150)], [0.2, 1.0], [-0.2, 1.0]];
    assert_eq!(
        triangulate(&domain_of(
            flat(),
            vec![[-4.0, -4.0], [4.0, -4.0], [4.0, 4.0], [-4.0, 4.0]],
            vec![below, above],
        )),
        the_refusal_before_the_flush(7)
    );
}

/// Two holes whose tips are `2⁻¹⁵⁰` apart. Flushed, both tips are one place,
/// and spade gives them one vertex. Two loops meeting at a corner they both
/// name is a pinch this crate meshes, but these two do not meet: the patch
/// would tie both holes to the first tip's point, and the second hole's two
/// sides into that tip would be drawn to a point that is not on them.
#[test]
fn two_corners_the_flush_puts_in_one_place_are_refused() {
    let below = vec![[0.5, 0.0], [0.3, -1.0], [0.7, -1.0]];
    let above = vec![[0.5, 2f64.powi(-150)], [0.7, 1.0], [0.3, 1.0]];
    assert_eq!(
        triangulate(&domain_of(
            flat(),
            vec![[-4.0, -4.0], [4.0, -4.0], [4.0, 4.0], [-4.0, 4.0]],
            vec![below, above],
        )),
        the_refusal_before_the_flush(7)
    );
}

/// A corner `2⁻¹⁵⁰` above a straight side of the outline, which the flush puts
/// on it. Flushed, the corner is on the hull; read where it really is, it dents
/// the hull inwards by `2⁻¹⁵⁰`. The triangles spade drew are still a tiling of
/// the patch that was asked for, so it is meshed.
///
/// The first version of the check asked for the hull to stay convex and
/// refused this, and with it 95 of the writer fuzzer's 2026-09-26 files that
/// the check as it stands meshes (`docs/decisions.md` §321.3).
#[test]
fn a_corner_flushed_onto_a_straight_side_of_the_hull_is_meshed() {
    check_everything(
        "a corner a hair inside a straight side",
        &domain_of(
            flat(),
            vec![
                [-1.0, 0.0],
                [0.0, 2f64.powi(-150)],
                [1.0, 0.0],
                [1.0, 1.0],
                [-1.0, 1.0],
            ],
            Vec::new(),
        ),
    );
}

/// The same, with the corner the smallest `f64` above the side, and a few
/// heights between that and the flush's floor. The three numbers along the
/// side are the ones a bar turned `5.6e-309` radians about `x` put there, in
/// one of the writer fuzzer's 2026-09-26 files.
///
/// Whether the corner is inside the side from `along[1]` to `along[2]` is the
/// sign of a determinant about `0.2 · 2⁻¹⁰⁷⁴`, which is under the smallest
/// `f64`. `orient2d` decides it by lifting every coordinate by one exact power
/// of two, so the check asks `orient2d` and nothing else. It used to lift the
/// coordinates itself, when `orient2d` took its power of two only from the
/// largest difference and refused this; `docs/decisions.md` §324.8 measured
/// that the lift decided nothing more once `orient2d` lifted the largest
/// coordinate too, and it went.
#[test]
fn a_corner_a_subnormal_inside_a_straight_side_of_the_hull_is_meshed() {
    let smallest = f64::from_bits(1);
    let along = [8.012115492376656, 15.391695551144624, 15.602540695680851];
    assert_eq!(
        orient2d([along[1], 0.0], [along[2], 0.0], [along[0], smallest]),
        Decision::Pos,
        "the flush check stands on orient2d deciding this turn"
    );
    for height in [smallest, 2f64.powi(-1000), 2f64.powi(-600), 2f64.powi(-143)] {
        check_everything(
            &format!("a corner {height:e} inside a straight side"),
            &domain_of(
                flat(),
                vec![
                    [0.0, 0.0],
                    [along[0], height],
                    [along[1], 0.0],
                    [along[2], 0.0],
                    [along[2], 4.0],
                    [0.0, 4.0],
                ],
                Vec::new(),
            ),
        );
    }
}

/// How far the mesh really is from the surface, and not how far it says it is.
///
/// The corners are put on the surface and the triangle between them is *flat*,
/// which is the whole point: a sample is a weighted average of three 3D
/// corners, never the surface point at the average of three `(u, v)`.
fn how_far_the_mesh_really_strays(
    patch: &Patch,
    surface: &Surface,
    how_far_off: impl Fn(Point) -> f64,
) -> f64 {
    let places: Vec<Point> = patch
        .corners()
        .iter()
        .map(|corner| surface.point_at(corner[0], corner[1]))
        .collect();
    let steps = 6;
    let mut worst = 0.0_f64;
    for &[a, b, c] in patch.triangles() {
        let (a, b, c) = (
            places[a as usize].coordinates(),
            places[b as usize].coordinates(),
            places[c as usize].coordinates(),
        );
        for first in 0..=steps {
            for second in 0..=steps - first {
                let third = steps - first - second;
                let weigh = |axis: usize| {
                    (a[axis] * f64::from(first)
                        + b[axis] * f64::from(second)
                        + c[axis] * f64::from(third))
                        / f64::from(steps)
                };
                worst = worst.max(how_far_off(Point::new(weigh(0), weigh(1), weigh(2))));
            }
        }
    }
    worst
}

fn how_far_off_the_ball(ball: Sphere) -> impl Fn(Point) -> f64 {
    move |point| (point.distance_to(ball.centre()) - ball.radius()).abs()
}

fn how_far_off_a_pipe_on_z(radius: f64) -> impl Fn(Point) -> f64 {
    move |point| ((point.x() * point.x() + point.y() * point.y()).sqrt() - radius).abs()
}

/// The claim that matters, measured against the pipe rather than against the
/// bound: no point of any triangle is further off the pipe than the patch says,
/// and nothing is further off than was asked for.
#[test]
fn a_mesh_of_a_pipe_stays_within_its_deflection_of_the_pipe() {
    for (radius, steps) in [(1.0, 8), (40.0, 60)] {
        let pipe = Cylinder::new(Axis::z(), radius).expect("a pipe");
        let surface = Surface::from(pipe);
        let deflection = Deflection::of(0.05).expect("a twentieth");
        let domain = domain_at(
            deflection,
            surface,
            a_walked_rectangle([0.0, 2.5], [-1.0, 3.0], steps),
            Vec::new(),
        );
        let patch = triangulate(&domain).expect("a patch of a pipe");
        let really =
            how_far_the_mesh_really_strays(&patch, &surface, how_far_off_a_pipe_on_z(radius));
        assert!(
            really <= patch.deviation().magnitude(),
            "a pipe of radius {radius} really strays {really} and the patch published {}",
            patch.deviation().magnitude()
        );
        assert!(
            really <= deflection.length(),
            "a pipe of radius {radius} really strays {really} and {} was asked for",
            deflection.length()
        );
    }
}

/// The same for a ball, including the patch that reaches both poles.
#[test]
fn a_mesh_of_a_ball_stays_within_its_deflection_of_the_ball() {
    let deflection = Deflection::of(0.05).expect("a twentieth");
    let cases = [
        (
            "a patch off the poles",
            Sphere::new(Point::new(1.0, 2.0, 3.0), 3.0).expect("a ball"),
            a_walked_rectangle([0.0, 2.0], [-1.0, 1.0], 16),
        ),
        (
            "a patch reaching both poles",
            Sphere::new(Point::origin(), 1.5).expect("a ball"),
            a_walked_rectangle([0.0, TAU], [-FRAC_PI_2, FRAC_PI_2], 16),
        ),
    ];

    for (what, ball, outer) in cases {
        let surface = Surface::from(ball);
        let domain = domain_at(deflection, surface, outer, Vec::new());
        let patch =
            triangulate(&domain).unwrap_or_else(|why| panic!("{what} would not cut: {why}"));
        let really = how_far_the_mesh_really_strays(&patch, &surface, how_far_off_the_ball(ball));
        assert!(
            really <= patch.deviation().magnitude(),
            "{what} really strays {really} and the patch published {}",
            patch.deviation().magnitude()
        );
        assert!(
            really <= deflection.length(),
            "{what} really strays {really} and {} was asked for",
            deflection.length()
        );
    }
}

/// Asking for a closer fit never gets a worse one.
///
/// Not implied by either of the two claims above: a patch that met every
/// deflection it was given could still wander about underneath them, and a
/// caller that halves the number and gets a rougher mesh has been told
/// something untrue by the knob.
#[test]
fn halving_the_deflection_never_makes_the_mesh_stray_further() {
    let ball = Sphere::new(Point::origin(), 3.0).expect("a ball");
    let outer = a_walked_rectangle([0.0, 2.0], [-1.0, 1.0], 20);
    let mut coarser = f64::INFINITY;
    let mut triangles = 0;
    for halving in 0..4 {
        let length = 0.2 / f64::from(1 << halving);
        let deflection = Deflection::of(length).expect("a positive length");
        let domain = domain_at(deflection, Surface::from(ball), outer.clone(), Vec::new());
        let patch = triangulate(&domain).expect("a patch of a ball");
        assert!(
            patch.deviation().magnitude() <= coarser,
            "asking for {length} strayed {} where the deflection before it strayed {coarser}",
            patch.deviation().magnitude()
        );
        assert!(patch.deviation().magnitude() <= length);
        assert!(
            patch.triangles().len() >= triangles,
            "asking for {length} used fewer triangles than the deflection before it"
        );
        coarser = patch.deviation().magnitude();
        triangles = patch.triangles().len();
    }
}

/// A boundary side is a chord, and a chord's distance from the surface is fixed
/// the moment its two corners are chosen.
///
/// This crate may not put a point on a loop, so there is nothing here to try
/// and the honest answer is a refusal naming the side. Refining the boundary
/// belongs to the caller, which is the only one that can see both sides of it
/// (`docs/decisions.md` §25.1).
#[test]
fn a_boundary_side_further_off_the_surface_than_the_deflection_is_refused() {
    let pipe = Cylinder::new(Axis::z(), 1.0).expect("a pipe");
    // A quarter turn of a unit pipe is a chord standing 0.29 off it, and a
    // thousand more triangles inside the patch will not move that chord.
    let refusal = triangulate(&domain_at(
        Deflection::of(0.01).expect("a hundredth"),
        Surface::from(pipe),
        vec![[0.0, 0.0], [FRAC_PI_2, 0.0], [FRAC_PI_2, 1.0], [0.0, 1.0]],
        Vec::new(),
    ));
    assert!(
        matches!(
            refusal,
            Err(CannotTriangulate::BoundarySideStraysFurtherThanTheDeflection { .. })
        ),
        "got {refusal:?}"
    );

    // The same patch walked into sixteen steps, which is what a caller hands
    // over, cuts without complaint.
    let fine_enough = domain_at(
        Deflection::of(0.01).expect("a hundredth"),
        Surface::from(pipe),
        a_walked_rectangle([0.0, FRAC_PI_2], [0.0, 1.0], 16),
        Vec::new(),
    );
    check_everything("a quarter of a pipe, walked", &fine_enough);
}

/// A coarse mesh is a request and not a failure, and a fine one is a request
/// too. Both have to finish, and the coarse one has to stay small.
#[test]
fn a_coarse_deflection_stays_small_and_a_fine_one_still_finishes() {
    let pipe = Cylinder::new(Axis::z(), 1.0).expect("a pipe");

    let coarse = domain_at(
        Deflection::of(0.3).expect("three tenths"),
        Surface::from(pipe),
        a_walked_rectangle([0.0, 1.5], [0.0, 2.0], 2),
        Vec::new(),
    );
    let patch = triangulate(&coarse).expect("a coarse patch");
    assert!(
        patch.triangles().len() <= 24,
        "a coarse mesh came out as {} triangles",
        patch.triangles().len()
    );
    assert!(patch.deviation().magnitude() <= 0.3);

    let fine = domain_at(
        Deflection::of(1e-4).expect("a ten-thousandth"),
        Surface::from(pipe),
        a_walked_rectangle([0.0, 1.5], [0.0, 2.0], 100),
        Vec::new(),
    );
    let patch = triangulate(&fine).expect("a fine patch");
    assert!(
        patch.triangles().len() > 2_000,
        "a fine mesh came out as {} triangles, which is not fine",
        patch.triangles().len()
    );
    assert!(patch.deviation().magnitude() <= 1e-4);
}

/// A deflection under what the evaluator itself rounds to is refused, and not
/// chased.
///
/// The points a mesh is made of come out of `Surface::point_at`, which rounds.
/// Asking to be nearer the surface than they are is not a request for more
/// triangles — no number of them arrives — so it is answered at once rather
/// than after a very long wait.
///
/// The floor moves with where the surface is, not only with how big it is: the
/// same ball a kilometre out has its points known to a rounding of the
/// kilometre, and a deflection the ball at the origin takes is refused there.
#[test]
fn a_deflection_finer_than_a_point_can_be_placed_is_refused() {
    let outer = a_walked_rectangle([0.0, 0.01], [-0.005, 0.005], 32);
    let at_the_origin = Sphere::new(Point::origin(), 1.0).expect("a ball");
    let far_away = Sphere::new(Point::new(1e9, 0.0, 0.0), 1.0).expect("the same ball, moved");
    let deflection = Deflection::of(1e-7).expect("a ten-millionth");

    let close_enough = triangulate(&domain_at(
        deflection,
        Surface::from(at_the_origin),
        outer.clone(),
        Vec::new(),
    ));
    assert!(close_enough.is_ok(), "got {close_enough:?}");

    let too_close = triangulate(&domain_at(
        deflection,
        Surface::from(far_away),
        outer,
        Vec::new(),
    ));
    assert!(
        matches!(
            too_close,
            Err(CannotTriangulate::NoMeshCanGetThatCloseToTheSurface { .. })
        ),
        "got {too_close:?}"
    );
}

/// A deflection that would need a mesh nobody can hold is refused before a
/// single point of it is placed.
#[test]
fn a_deflection_needing_more_points_than_a_patch_holds_is_refused() {
    let ball = Sphere::new(Point::origin(), 1.0).expect("a ball");
    let refusal = triangulate(&domain_at(
        Deflection::of(1e-11).expect("a hundred-billionth"),
        Surface::from(ball),
        a_walked_rectangle([0.0, 1.0], [-0.5, 0.5], 8),
        Vec::new(),
    ));
    assert!(
        matches!(
            refusal,
            Err(CannotTriangulate::MoreTrianglesThanAPatchWillHold { .. })
        ),
        "got {refusal:?}"
    );
}

/// The refinement's own points have to fit in the triangulation's range too,
/// and a midpoint of two coordinates that very nearly cancel does not.
///
/// A midpoint lies between two coordinates already in the mesh, so it is never
/// too *big*; it can be too small. What must not happen is a panic. What
/// happens instead is that the point is declined, the triangle it would have
/// split stays over budget, and the patch is refused with the distance it got
/// to.
///
/// The numbers are chosen rather than found: a ball of radius `1e-27` puts its
/// scaled corners around `5e-28`, comfortably inside spade's `2^-142 .. 2^201`,
/// while the diagonal of the rectangle is the longest chord and its midpoint
/// lands at `5.5e-44`, under `2^-142`, because the two `u` are a single `f64`
/// step from cancelling.
#[test]
fn a_refinement_point_the_triangulation_will_not_take_is_refused_and_never_a_panic() {
    let tiny = Sphere::new(Point::origin(), 1e-27).expect("a very small ball");
    let left = -0.5;
    let right = (0.5_f64).next_up();
    let refusal = triangulate(&domain_at(
        Deflection::of(4e-28).expect("a positive length"),
        Surface::from(tiny),
        vec![[left, -0.5], [right, -0.5], [right, 0.5], [left, 0.5]],
        Vec::new(),
    ));
    assert!(
        matches!(
            refusal,
            Err(CannotTriangulate::RanOutOfRoomBeforeMeetingTheDeflection { .. })
        ),
        "got {refusal:?}"
    );
}

/// The same five claims over a corpus nobody hand-picked.
///
/// Every coordinate is a whole number, which is what lets the coverage claim be
/// an **equality** rather than a near-enough: each triangle's area is a cross
/// product of exact integer differences, and a sum of exact integers is exact.
/// A tolerance here would be a number the test was measuring instead of the
/// mesher (§14.1 question 1). The plane is what keeps it exact — a plane is
/// never refined, so no rounded midpoint ever joins the sum.
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
            check_everything("a punched plate", &domain_of(flat(), outer, holes));
        }
    }
}
