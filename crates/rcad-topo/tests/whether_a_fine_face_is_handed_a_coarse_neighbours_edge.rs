//! Two curved faces of very different curvature share one edge. Who walks it?
//!
//! [`SolidView::mesh`](rcad_topo::SolidView::mesh) works an edge's points out
//! **once**, in the first pass, and skips any edge already walked. So an edge
//! between two faces is polylined by whichever of the two comes first in the
//! solid's stored order, and both faces then read the same points. That keeps
//! the mesh watertight by construction (`docs/decisions.md` §25.7) and raises
//! two separate questions, which this file keeps apart:
//!
//! 1. **Can the true worst deviation beat the number `Mesh::deviation` hands
//!    out?** That number is published as *"how far any point of any triangle
//!    can be from the surface it stands in for"*, and §15.6 says a published
//!    bound needs a test that could fail rather than a paragraph. Nothing in
//!    the tree measured it: `what_a_deflection_costs_a_measured_volume.rs`
//!    checks the mesh's own number against the deflection asked for, which is
//!    the mesher marking its own homework. This file measures the distance from
//!    the triangles to the surfaces and compares.
//! 2. **Can the mesh come back coarser than the deflection asked for?** A
//!    boundary the caller cannot see is refined by a neighbour, and a consumer
//!    that draws edges from these points inherits whatever that neighbour
//!    settled for. Honest reporting would not make that less of a surprise.
//!
//! # The shape
//!
//! The lens between two overlapping balls of radius 10 and 0.5. It has exactly
//! two faces, both spheres, twenty times apart in how tightly they bend, and
//! they share one edge — the circle where the two balls cross. A ball against
//! a bar is sharper still, fifty-fold, and it is a solid to mesh now: down the
//! ball's own axis ([`a_bar_down_a_balls_axis_is_answered_under_every_rule`])
//! and turned onto `x` by an exact quarter turn
//! ([`a_bar_turned_exactly_onto_x_is_answered_under_every_rule`]) or by
//! `FRAC_PI_2`, which leans it off the ball's centre
//! ([`a_bar_turned_by_frac_pi_2_leans_off_the_balls_centre_and_is_answered_under_every_rule`]).
//! It is not the shape the two questions are asked of because its wall is a
//! pipe, and the closed-form oracle below is written for a sphere. Two
//! *coaxial* bars of different radii, the other shape to reach for, meet along
//! a flat annulus and not along an edge at all, so the annulus is a plane face
//! between them and neither curved face ever sees the other's boundary.
//!
//! Fusing the small ball as the argument rather than the tool reverses the
//! stored order of the two faces, which is the knob that decides who walks the
//! shared circle. [`which_ball_walks_the_shared_circle_changes_nothing`] turns
//! it and compares the two meshes point for point.
//!
//! # Why turning it changes nothing, which is the answer to the second question
//!
//! The budget an edge is walked to is
//! `self.deflection.length() - how_far_moving_this_face_rounds(face)`, and the
//! subtracted term is what *placing* that face rounds — not how tightly it
//! bends. So the two faces along an edge hand the walk the same budget and
//! there is no coarse face's budget for a fine face to inherit. The bit-for-bit
//! comparison below is what says so; this paragraph only says where to look.
//!
//! # The oracle is closed form, and that is the point
//!
//! Both faces are spheres, so how far a flat triangle gets from the surface it
//! stands in for is exact arithmetic and not a sampled guess: over a closed
//! triangle the distance to the centre is largest at a corner and smallest at
//! the one point of the triangle nearest the centre, and both are worked out
//! rather than searched for. No tolerance appears anywhere below.
//!
//! [`measuring_a_coarse_mesh_against_a_fine_ones_number_finds_it_out`] is what
//! says the measurement has teeth: the same machinery, pointed at a mesh cut
//! ten times too coarse for the number it is checked against, sees it.

use core::f64::consts::{FRAC_PI_2, PI};

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Sphere, Surface, Vector};
use rcad_predicates::{Cause, Policy, ProvenanceLog};
use rcad_topo::{Mesh, Model, Rule, SolidKey};

/// The deflections every shape here is cut at.
const HOW_FINE: [f64; 3] = [0.1, 0.01, 0.001];

/// The three lenses: the two radii and how far apart the centres are.
///
/// Each pair properly overlaps — the centres are further apart than the
/// difference of the radii and closer than the sum — so the answer is a lens
/// with one face on each ball and one edge between them.
const LENSES: [[f64; 3]; 3] = [[10.0, 0.5, 10.3], [100.0, 0.5, 100.3], [10.0, 0.1, 10.06]];

/// The lens between two overlapping balls, with the big one meshed first or the
/// small one.
fn a_lens_between_two_balls(
    [big, small, apart]: [f64; 3],
    big_ball_first: bool,
) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let ball = model
        .add_ball(big, &Isometry::identity())
        .expect("a ball at the origin");
    let pea = model
        .add_ball(
            small,
            &Isometry::translation(Vector::new(0.0, apart, 0.0)).expect("a shift"),
        )
        .expect("a smaller ball overlapping it");
    let (arguments, tools) = if big_ball_first {
        ([ball], [pea])
    } else {
        ([pea], [ball])
    };
    let fused = model
        .fuse(
            &arguments,
            &tools,
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("two overlapping balls have a lens between them");
    let solids = fused.solids().to_vec();
    assert_eq!(solids.len(), 1, "a lens is one solid");
    let lens = solids[0];
    assert_eq!(
        model
            .solid(lens)
            .expect("this model made the key")
            .faces()
            .count(),
        2,
        "a lens has one face on each ball"
    );
    (model, lens)
}

/// Q1. The number `Mesh::deviation` publishes covers every triangle there is.
///
/// Nine meshes: three lenses at three deflections. For each, the worst any
/// point of any triangle really strays from the surface that triangle stands in
/// for, against the one number the mesh hands out.
///
/// **It also asserts the worst is not nothing**, which is the other side §15.6
/// asks for. A bound is untested from below by a mesh that happens to be exact.
#[test]
fn the_published_deviation_covers_what_the_triangles_really_do() {
    let mut closest_to_the_bound = 0.0_f64;
    let mut meshes = 0;
    for lens in LENSES {
        for how_fine in HOW_FINE {
            let (model, solid) = a_lens_between_two_balls(lens, true);
            let mesh = meshed(&model, solid, how_fine);
            let published = mesh.deviation().magnitude();
            let really = the_worst_any_triangle_really_strays(&model, solid, &mesh);
            assert!(
                really <= published,
                "a lens of {lens:?} cut at {how_fine} publishes {published:e} and a triangle of \
                 it really strays {really:e}"
            );
            assert!(
                really > 0.0,
                "a lens of {lens:?} cut at {how_fine} strays nothing at all, so the bound is \
                 untested from below"
            );
            closest_to_the_bound = closest_to_the_bound.max(really / published);
            meshes += 1;
        }
    }
    assert_eq!(meshes, LENSES.len() * HOW_FINE.len(), "every lens was cut");
    println!(
        "the closest any of the {meshes} meshes came to its own published number was \
         {:.1}% of it",
        closest_to_the_bound * 100.0
    );
    assert!(
        closest_to_the_bound > 0.1,
        "the worst measured was under a tenth of the bound everywhere ({closest_to_the_bound:e}), \
         so nothing here would notice the bound halving"
    );
}

/// The measurement has teeth, shown by pointing it at a mesh it must convict.
///
/// A lens cut at the coarsest deflection, checked against the number the same
/// lens publishes when cut at the finest. If
/// [`the_worst_any_triangle_really_strays`] could not see a coarse mesh it
/// would not have seen a broken bound either, and the test above would be a
/// green light for nothing. It covers
/// [`no_triangle_of_either_face_strays_further_than_the_deflection_asked_for`]
/// too: the same coarse mesh strays far past the finest deflection here, so a
/// mesh cut coarser than it was asked for is a thing this file can see.
#[test]
fn measuring_a_coarse_mesh_against_a_fine_ones_number_finds_it_out() {
    let lens = LENSES[0];
    let (model, solid) = a_lens_between_two_balls(lens, true);
    let coarse = meshed(&model, solid, HOW_FINE[0]);
    let fine = meshed(&model, solid, HOW_FINE[2]);
    let really = the_worst_any_triangle_really_strays(&model, solid, &coarse);
    let published_by_the_fine_one = fine.deviation().magnitude();
    assert!(
        really > published_by_the_fine_one,
        "a lens cut at {} strays {really:e}, which the number a cut at {} publishes \
         ({published_by_the_fine_one:e}) would have to cover",
        HOW_FINE[0],
        HOW_FINE[2]
    );
    println!(
        "the coarse mesh strays {really:e}, which is {:.0} times the fine mesh's \
         {published_by_the_fine_one:e}",
        really / published_by_the_fine_one
    );
}

/// Q2, first half. Which of the two faces walks the shared circle changes
/// nothing at all.
///
/// The two faces are meshed in the solid's stored order and the shared edge
/// goes to whichever comes first, so if the walk depended on the face that
/// reached it the two meshes would differ. They do not — the same points to the
/// last bit, the same triangles between them, and the same published number.
///
/// **The reversal is asserted and not assumed.** A test that turned a knob that
/// was not connected would pass for the wrong reason, so the two runs are
/// checked to have their faces stored the other way round before the meshes are
/// compared.
#[test]
fn which_ball_walks_the_shared_circle_changes_nothing() {
    for lens in LENSES {
        let (big_first, one) = a_lens_between_two_balls(lens, true);
        let (small_first, other) = a_lens_between_two_balls(lens, false);
        let radii = |model: &Model, solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .faces()
                .map(|face| match face.surface() {
                    Surface::Sphere(ball) => ball.radius(),
                    other @ (Surface::Plane(_)
                    | Surface::Cylinder(_)
                    | Surface::Cone(_)
                    | Surface::Torus(_))
                    | other => panic!("a lens has no {other:?} face"),
                })
                .collect::<Vec<_>>()
        };
        let one_way = radii(&big_first, one);
        let the_other_way = radii(&small_first, other);
        assert_eq!(
            one_way,
            the_other_way.iter().rev().copied().collect::<Vec<_>>(),
            "turning the fuse round did not turn the stored face order round, so there is no \
             knob here to turn"
        );

        for how_fine in HOW_FINE {
            let walked_by_the_big_one = meshed(&big_first, one, how_fine);
            let walked_by_the_small_one = meshed(&small_first, other, how_fine);
            let (points, other_points) = (
                every_point_in_order(&walked_by_the_big_one),
                every_point_in_order(&walked_by_the_small_one),
            );
            assert!(
                points == other_points,
                "a lens of {lens:?} cut at {how_fine} has {} points when the big ball walks the \
                 circle and {} when the small one does, and {} of them are not the same place",
                points.len(),
                other_points.len(),
                how_many_differ(&points, &other_points)
            );
            let (triangles, other_triangles) = (
                every_triangle_in_order(&walked_by_the_big_one),
                every_triangle_in_order(&walked_by_the_small_one),
            );
            assert!(
                triangles == other_triangles,
                "a lens of {lens:?} cut at {how_fine} has {} triangles when the big ball walks \
                 the circle and {} when the small one does, and {} of them are not the same \
                 triangle",
                triangles.len(),
                other_triangles.len(),
                how_many_differ(&triangles, &other_triangles)
            );
            assert_eq!(
                walked_by_the_big_one.deviation().magnitude().to_bits(),
                walked_by_the_small_one.deviation().magnitude().to_bits(),
                "a lens of {lens:?} cut at {how_fine} publishes a different number depending on \
                 which face walked the circle between them"
            );
        }
    }
}

/// Q2, second half. Neither face is left coarser than the deflection asked for.
///
/// The face that did **not** walk the shared circle reads points it had no say
/// in. This asks what that costs it: the worst any triangle of either face
/// really strays, against the deflection the caller passed — not against the
/// mesh's own number, which is what the other question is about.
#[test]
fn no_triangle_of_either_face_strays_further_than_the_deflection_asked_for() {
    let mut closest_to_the_deflection = 0.0_f64;
    for lens in LENSES {
        for how_fine in HOW_FINE {
            let (model, solid) = a_lens_between_two_balls(lens, true);
            let mesh = meshed(&model, solid, how_fine);
            for (radius, really) in the_worst_each_face_really_strays(&model, solid, &mesh) {
                assert!(
                    really <= how_fine,
                    "the face of a lens of {lens:?} on radius {radius} strays {really:e} where \
                     {how_fine} was asked for"
                );
                closest_to_the_deflection = closest_to_the_deflection.max(really / how_fine);
            }
        }
    }
    println!(
        "the closest any face came to the deflection it was cut at was {:.1}% of it",
        closest_to_the_deflection * 100.0
    );
    assert!(
        closest_to_the_deflection > 0.1,
        "no face came within a tenth of its deflection ({closest_to_the_deflection:e}), so this \
         would not notice a mesh cut twice as coarse as it asked for"
    );
}

/// The shape this file would rather have had, down a ball's own axis, and it is
/// answered under every rule both ways round, under either policy.
///
/// A bar down a ball's own axis crosses its skin in two circles, which is a
/// pipe of radius a fifth against a ball of radius ten sharing an edge — a
/// fifty-fold difference in curvature where the lens manages twenty.
///
/// It was refused by the result's own mesh, which found two corners of a face
/// in one place (Q1241). The fuse writes each circle as a closed run with its
/// corner on it and nothing picking it, and `rcad_geom::walk` marched every
/// such run from the seed on the circle ahead of the centre, so the circle
/// behind was walked round the one ahead. Each is drawn in closed form now and
/// told apart by its corner. Before that it was refused
/// `ACrossingThisCannotBeOrderedAlong`, while the arrangement could not tell two
/// circles from a quartic (Q1140).
///
/// With `R = 10`, `r = 0.2` and the bar 40 long, so it reaches past the ball
/// both ends: the ball less the bar is `(4π/3)(R² − r²)^(3/2)`, the two share
/// `4000π/3` less that, the union is `4000π/3 + 40π·0.04` less what they share,
/// and the bar less the ball is `40π·0.04` less what they share, in two pieces.
/// Every answer is measured short of its closed form, since a mesh inscribed in
/// a round wall is inside it, and within half a per cent: the two no wider than
/// the bar at a five-hundredth, where a fiftieth leaves them 2.3% short, and
/// the rest at a fiftieth.
#[test]
fn a_bar_down_a_balls_axis_is_answered_under_every_rule() {
    answered_under_every_rule(
        &Isometry::translation(Vector::new(0.0, 0.0, -20.0)).expect("a shift"),
    );
}

/// The same bar **turned to lie along `x`** by an exact quarter turn about
/// `y`, against the same closed forms, and answered the same way.
///
/// It was refused `ACrossingThisCannotBeOrderedAlong` under every rule while
/// the only quarter turn to hand was `Isometry::rotation` by `FRAC_PI_2`
/// (Q1245, Q1320), which leaves the bar leaning off the ball's centre; the
/// next test says how, and that the leaning bar answers too since Q1554.
/// `Isometry::quarter_turns` turns the bar's axis onto `(1, 0, 0)` bit for
/// bit, through the ball's centre.
#[test]
fn a_bar_turned_exactly_onto_x_is_answered_under_every_rule() {
    let along_x = Isometry::translation(Vector::new(-20.0, 0.0, 0.0))
        .expect("a shift")
        .compose(&Isometry::quarter_turns(&Axis::y(), 1).expect("y is a coordinate axis"));
    let (origin, direction) = the_bar_axis(&along_x);
    assert_eq!(
        (origin, direction),
        ([-20.0, 0.0, 0.0], [1.0, 0.0, 0.0]),
        "the bar does not lie along x"
    );
    answered_under_every_rule(&along_x);
}

/// The bar turned by `Isometry::rotation(&Axis::y(), FRAC_PI_2)`, which misses
/// the ball's centre by `20·2⁻⁵²`, is answered under every rule both ways
/// round, under either policy, against the closed forms of the bar through it.
///
/// `FRAC_PI_2` is not `π/2`, and Rodrigues' form sends `z` to
/// `(1, 0, 1 − fl(j·2j))` with `j` the turn's stored `j` coefficient, which no
/// `f64` makes exactly `1`. So the pipe's axis is stored leaning `2⁻⁵²` off
/// `x`, and misses the ball's centre by `20·2⁻⁵²` (Q1245). A pipe whose axis
/// misses a ball's centre crosses it in two loops that are not circles, and
/// until Q1554 nothing ordered points along them, so every fuse was refused
/// `ACrossingThisCannotBeOrderedAlong`. Q1554 orders them round the pipe's
/// axis (`the_way_a_loop_round_a_pipe_through_a_ball_winds` in
/// `along_a_crossing.rs`). A centre that far off the axis moves no volume the
/// closed forms could tell apart, so they are the oracle here too.
#[test]
fn a_bar_turned_by_frac_pi_2_leans_off_the_balls_centre_and_is_answered_under_every_rule() {
    let along_x = Isometry::translation(Vector::new(-20.0, 0.0, 0.0))
        .expect("a shift")
        .compose(&Isometry::rotation(&Axis::y(), FRAC_PI_2).expect("a finite angle"));
    let (origin, direction) = the_bar_axis(&along_x);
    assert_eq!(
        (origin, direction),
        ([-20.0, 0.0, 0.0], [1.0, 0.0, 2f64.powi(-52)]),
        "the turned bar's axis is not the one Q1245 traced"
    );
    answered_under_every_rule(&along_x);
}

/// Where the pipe of a bar of radius 0.2 and length 40 placed by `placement`
/// is stored, as its axis's origin and direction.
fn the_bar_axis(placement: &Isometry) -> ([f64; 3], [f64; 3]) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let bar = model
        .add_round_bar(0.2, 40.0, placement)
        .expect("a bar of a working size");
    let mut pipes = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .filter_map(|face| {
            let Surface::Cylinder(pipe) = face.surface() else {
                return None;
            };
            Some(pipe.axis())
        });
    let (Some(axis), None) = (pipes.next(), pipes.next()) else {
        panic!("a round bar has one pipe");
    };
    (axis.origin().coordinates(), axis.direction().components())
}

/// A ball of radius 10 at the origin, and a bar of radius 0.2 and length 40
/// placed by `placement` so that it runs through the ball's centre, or misses
/// it by a few roundings, fused under every rule both ways round under either
/// policy, and each answer measured short of its closed form by under half a
/// per cent.
fn answered_under_every_rule(placement: &Isometry) {
    let (ball_volume, bar_volume) = (4000.0 * PI / 3.0, 40.0 * PI * 0.04);
    let left = 4.0 * PI / 3.0 * (100.0_f64 - 0.04).powf(1.5);
    let shared = ball_volume - left;
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for (rule, ball_first, pieces, closed_form, how_fine) in [
            (
                Rule::Union,
                true,
                1,
                ball_volume + bar_volume - shared,
                0.02,
            ),
            (
                Rule::Union,
                false,
                1,
                ball_volume + bar_volume - shared,
                0.02,
            ),
            (Rule::Intersection, true, 1, shared, 0.002),
            (Rule::Intersection, false, 1, shared, 0.002),
            (Rule::Difference, true, 1, left, 0.02),
            (Rule::Difference, false, 2, bar_volume - shared, 0.002),
        ] {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
            let ball = model
                .add_ball(10.0, &Isometry::identity())
                .expect("a ball at the origin");
            let bar = model
                .add_round_bar(0.2, 40.0, placement)
                .expect("a bar right through it");
            let (argument, tool) = if ball_first { (ball, bar) } else { (bar, ball) };
            let what = format!("{policy:?}, {rule:?}, the ball first {ball_first}");
            let mut log = ProvenanceLog::new();
            let answer = model
                .fuse(&[argument], &[tool], rule, &policy, &mut log)
                .unwrap_or_else(|why| panic!("{what}: refused with {why:?}"));
            assert!(log.is_empty(), "{what}: left {:?}", log.records());
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("a fuse writes a sound model or writes nothing");
            assert_eq!(answer.solids().len(), pieces, "{what}");
            let volume: f64 = answer
                .solids()
                .iter()
                .map(|&solid| {
                    model
                        .solid(solid)
                        .expect("this model made the key")
                        .measure(
                            Deflection::of(how_fine).expect("a positive length"),
                            &Policy::certified(),
                            &mut ProvenanceLog::new(),
                        )
                        .expect("a solid this size measures")
                        .volume()
                })
                .sum();
            let short = (closed_form - volume) / closed_form;
            println!("{what}: {short:.6} short of the closed form at {how_fine}");
            assert!(
                short > 0.0 && short < 0.005,
                "{what}: measured {volume} at {how_fine} against {closed_form}"
            );
        }
    }
}

/// One mesh, cut under a certified policy with nothing left unsettled.
fn meshed(model: &Model, solid: SolidKey, how_fine: f64) -> Mesh {
    let mut log = ProvenanceLog::new();
    let mesh = model
        .solid(solid)
        .expect("this model made the key")
        .mesh(
            Deflection::of(how_fine).expect("a positive length"),
            &Policy::certified(),
            &mut log,
        )
        .unwrap_or_else(|why| panic!("cutting at {how_fine}: {why}"));
    assert!(
        log.is_empty(),
        "nothing about this mesh had to go unsettled, got {:?}",
        log.records()
    );
    assert_eq!(
        mesh.deviation().cause(),
        Cause::FlatStandIn,
        "the deviation is a flat piece standing in for a curved one"
    );
    mesh
}

/// How far the worst point of the worst triangle really is from the surface
/// that triangle stands in for.
fn the_worst_any_triangle_really_strays(model: &Model, solid: SolidKey, mesh: &Mesh) -> f64 {
    the_worst_each_face_really_strays(model, solid, mesh)
        .into_iter()
        .map(|(_, strays)| strays)
        .fold(0.0, f64::max)
}

/// The same, one entry per face, each carrying the radius that face bends at.
fn the_worst_each_face_really_strays(
    model: &Model,
    solid: SolidKey,
    mesh: &Mesh,
) -> Vec<(f64, f64)> {
    let view = model.solid(solid).expect("this model made the key");
    let mut worst = Vec::new();
    for face in view.faces() {
        let Surface::Sphere(ball) = face.surface() else {
            panic!("every face of a lens is a sphere");
        };
        let mut strays = 0.0_f64;
        let mut triangles = 0;
        for triangle in mesh.triangles() {
            if triangle.face() != face.key() {
                continue;
            }
            let corners = triangle.corners().map(|at| mesh.points()[at as usize]);
            strays = strays.max(how_far_a_flat_triangle_gets_from_a_ball(&ball, corners));
            triangles += 1;
        }
        assert!(triangles > 0, "a face of the lens has no triangles at all");
        worst.push((ball.radius(), strays));
    }
    assert_eq!(worst.len(), 2, "a lens has two faces");
    worst
}

/// How far the furthest point of a flat triangle is from a ball's skin, exactly.
///
/// The distance from the centre is a convex function on the plane, so over a
/// closed triangle it is largest at a corner; and it is smallest at the one
/// point of the triangle nearest the centre, which
/// [`the_point_of_a_triangle_nearest`] works out. The answer is whichever of
/// the two misses the radius by more.
fn how_far_a_flat_triangle_gets_from_a_ball(ball: &Sphere, corners: [Point; 3]) -> f64 {
    let furthest = corners
        .iter()
        .map(|corner| (*corner - ball.centre()).length())
        .fold(0.0, f64::max);
    let nearest =
        (the_point_of_a_triangle_nearest(corners, ball.centre()) - ball.centre()).length();
    (furthest - ball.radius())
        .max(ball.radius() - nearest)
        .max(0.0)
}

/// The point of a closed triangle nearest `to`.
///
/// The usual case analysis: the foot of the perpendicular when it lands inside,
/// and otherwise the nearest point of the nearest side.
fn the_point_of_a_triangle_nearest(corners: [Point; 3], to: Point) -> Point {
    let [a, b, c] = corners;
    let (along, across) = (b - a, c - a);
    let normal = along.cross(across);
    let squared = normal.length_squared();
    if squared > 0.0 {
        let onto = to - normal * (normal.dot(to - a) / squared);
        let inside = [(a, b), (b, c), (c, a)]
            .into_iter()
            .all(|(from, next)| (next - from).cross(onto - from).dot(normal) >= 0.0);
        if inside {
            return onto;
        }
    }
    [(a, b), (b, c), (c, a)]
        .into_iter()
        .map(|side| the_point_of_a_side_nearest(side, to))
        .fold(a, |best, here| {
            if (here - to).length_squared() < (best - to).length_squared() {
                here
            } else {
                best
            }
        })
}

/// The point of one side of a triangle nearest `to`.
fn the_point_of_a_side_nearest((from, next): (Point, Point), to: Point) -> Point {
    let along = next - from;
    let squared = along.length_squared();
    if squared == 0.0 {
        return from;
    }
    from + along * (along.dot(to - from) / squared).clamp(0.0, 1.0)
}

/// How many entries of two sorted lists are not in both, so that a difference
/// is reported as a count rather than as two tables of numbers.
fn how_many_differ<T: Ord>(one: &[T], other: &[T]) -> usize {
    one.iter().filter(|each| !other.contains(each)).count()
        + other.iter().filter(|each| !one.contains(each)).count()
}

/// Every point of the table, as bits, in an order that does not depend on which
/// face was walked first.
fn every_point_in_order(mesh: &Mesh) -> Vec<[u64; 3]> {
    let mut points: Vec<[u64; 3]> = mesh
        .points()
        .iter()
        .map(|point| point.coordinates().map(f64::to_bits))
        .collect();
    points.sort_unstable();
    points
}

/// Every triangle, named by the bits of the three points it stands on rather
/// than by their entries in the table, and in the same order-free way.
fn every_triangle_in_order(mesh: &Mesh) -> Vec<[[u64; 3]; 3]> {
    let mut triangles: Vec<[[u64; 3]; 3]> = mesh
        .triangles()
        .iter()
        .map(|triangle| {
            let mut corners = triangle
                .corners()
                .map(|at| mesh.points()[at as usize].coordinates().map(f64::to_bits));
            corners.sort_unstable();
            corners
        })
        .collect();
    triangles.sort_unstable();
    triangles
}
