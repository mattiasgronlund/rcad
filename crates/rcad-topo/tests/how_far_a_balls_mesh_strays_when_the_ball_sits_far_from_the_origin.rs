//! What `Mesh::deviation` promises, checked against the ball it describes.
//!
//! `docs/decisions.md` §25.3 says the number a mesh carries is *"the furthest
//! any point of any triangle can be from the surface"*. §15.6 says a published
//! bound needs a test.
//!
//! `how_far_a_balls_mesh_points_really_sit_from_the_ball.rs` holds it **where a
//! part sits**: four radii from 0.25 to 10, at the origin and ten thousand out,
//! sampled 21 points to a triangle. This file holds the far end of the same
//! claim — radii down to `1e-4`, placements out to `1e13`, and an exact worst
//! rather than a sampled one — because the two ends fail differently, and that
//! file's framing sentence, *"every mesh point of a ball is on the sphere by
//! construction"*, is true at `1e4` and false by `1e11`. The points are off the
//! sphere by 0.708% of the radius there and by 95% at `1e13`, which no chord sag
//! explains: a ball smaller than the grid it sits on has no representable point
//! on its own surface.
//!
//! Neither file's claim was held before them. Every older assertion on
//! `deviation` checks the *ceiling above it* — `deviation <= the deflection
//! asked for`, which is `meshing_a_curved_solid.rs:125` and
//! `a_certified_mesh_never_strays_further_than_the_deflection_it_was_asked_for`
//! — and never the claim itself. §23.4's oracle, the one every mesh test runs,
//! is combinatorial and has no distance anywhere in it by design.
//!
//! # Where the question came from
//!
//! A consumer meshed a ball of radius `1e-3` placed `1.7e12` away, under
//! `Policy::certified()`, and shaded a visibly lumpy ball with an **empty
//! provenance log** and no refusal. An empty log looks like "nothing was
//! approximated", so the report read as a defect.
//!
//! It is not one, and that is worth a test rather than a paragraph. §25.3: the
//! mesher *"makes no decision"* — it refines until its own bound is under the
//! deflection it was handed — so there is nothing for it to log, and the number
//! describing the lumpiness rides on the mesh instead. That ball is smaller
//! than the spacing of the `f64` grid it sits on, and
//! [`a_ball_meshed_far_from_the_origin_leaves_the_log_empty_and_says_so_on_the_mesh_instead`]
//! is the standing statement that **an empty log is not a claim that the points
//! landed on the surface**.
//!
//! # The oracle
//!
//! Closed form, not sampled. `Model::add_ball` stores its sphere about
//! `Point::origin()`, so the located centre is the placement applied to the
//! origin, and the surface is every point exactly `radius` from it. Over one
//! flat triangle, `|p - centre|` is convex, so its **largest** value is at a
//! corner; its **smallest** is at the point of the triangle nearest the centre,
//! which [`closest_point_on_a_triangle`] gives exactly. The furthest that
//! triangle strays is then the worse of the two ends, with no step size in it
//! and nothing to tune.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Mesh, Model};

/// Sizes small enough that a far placement outruns them, and one that never is.
const RADII: [f64; 4] = [1e-4, 1e-3, 1.0, 100.0];

/// How far out the ball is put, along the diagonal, in the model's own unit.
///
/// `1e13` is past where a `1e-3` ball can be placed at all — the grid spacing
/// there is twice its radius — and the point of carrying it is that the mesh
/// still describes what it did rather than quietly getting it wrong.
const DISTANCES: [f64; 7] = [0.0, 1e6, 1e9, 1e10, 1e11, 1e12, 1e13];

/// Deflections as multiples of the radius, from far coarser than the ball down
/// to a tenth of it. The coarse end is the half that matters: asking for less
/// accuracy than the grid can give is the one request nothing refuses.
const COARSENESSES: [f64; 6] = [50.0, 10.0, 4.0, 1.0, 0.5, 0.1];

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// A shift out along the diagonal, and the same shift with a turn on either
/// side of it, because a turn is what stops the centre being three copies of
/// one exactly representable number.
fn placements(distance: f64) -> [(&'static str, Isometry); 3] {
    let shift = Isometry::translation(Vector::new(distance, distance, distance)).expect("a shift");
    [
        ("a shift", shift),
        (
            "a turn then a shift",
            Isometry::rotation(&Axis::x(), 0.7)
                .expect("a turn")
                .compose(&shift),
        ),
        (
            "a shift then a turn",
            shift.compose(&Isometry::rotation(&Axis::z(), 1.3).expect("a turn")),
        ),
    ]
}

/// §25.3's number, held against the ball it describes.
///
/// The floor at the end is what stops this passing by refusing everything, and
/// the ceiling next to it is the other half of §15.6: a bound no answer ever
/// comes close to is untested from below, and would stay green if the mesher
/// started reporting a metre.
#[test]
fn every_triangle_of_a_far_off_ball_stays_inside_the_deviation_the_mesh_reports() {
    let mut meshed = 0;
    let mut closest_any_answer_came_to_the_bound = 0.0_f64;
    for radius in RADII {
        for distance in DISTANCES {
            for (how, placement) in placements(distance) {
                let centre = placement.apply_to_point(Point::origin());
                for coarseness in COARSENESSES {
                    let Some(mesh) = meshed_ball(radius, &placement, radius * coarseness) else {
                        continue;
                    };
                    let claimed = mesh.deviation().magnitude();
                    let strayed = how_far_the_mesh_strays_from_the_ball(&mesh, centre, radius);
                    assert!(
                        strayed <= claimed,
                        "a ball of {radius} put {distance} away by {how} and meshed at \
                         {}: a triangle strays {strayed} from it, which is further than \
                         the {claimed} the mesh reports",
                        radius * coarseness
                    );
                    meshed += 1;
                    closest_any_answer_came_to_the_bound =
                        closest_any_answer_came_to_the_bound.max(strayed / claimed);
                }
            }
        }
    }

    println!(
        "{meshed} balls meshed; the worst came to \
         {closest_any_answer_came_to_the_bound:.3} of the deviation it reported"
    );
    assert!(
        meshed > 300,
        "only {meshed} of the sweep meshed at all, which is not a sweep"
    );
    assert!(
        closest_any_answer_came_to_the_bound > 0.5,
        "no answer came within half of the number it published (worst was \
         {closest_any_answer_came_to_the_bound}), so this bound is not being tested from below"
    );
}

/// The teeth of the test above: the number carries a placement term.
///
/// Without one, a ball's deviation would depend only on how finely it was cut,
/// and the far cells of the sweep would fail: a `1e-3` ball at `1e13` strays
/// `9.53e-4` under a shift and the whole `1e-3` under a shift and a turn —
/// a mesh point landing on the centre — against the `6.667e-4` the same ball
/// reports at the origin. So this is what would go red first if that term were
/// ever dropped, and it names the reason instead of leaving a distance
/// assertion somewhere else to fail with a number in it.
///
/// The whole column is cut into the same number of triangles, which is checked
/// rather than assumed: a coarser mesh further out would explain a bigger
/// number all on its own and the comparison would say nothing.
#[test]
fn the_further_out_a_ball_sits_the_bigger_the_deviation_its_mesh_reports() {
    const RADIUS: f64 = 1e-3;
    let coarsest = RADIUS * COARSENESSES[0];

    let mut reported = Vec::new();
    let mut corners = None;
    for distance in DISTANCES {
        let placement =
            Isometry::translation(Vector::new(distance, distance, distance)).expect("a shift");
        let mesh = meshed_ball(RADIUS, &placement, coarsest).unwrap_or_else(|| {
            panic!("a ball {distance} away meshes at a deflection of {coarsest}")
        });
        let points = mesh.points().len();
        let first = *corners.get_or_insert(points);
        assert_eq!(
            first, points,
            "the ball {distance} away was cut into {points} points where the ball at the origin \
             was cut into {first}, so their deviations are not comparable"
        );
        reported.push(mesh.deviation().magnitude());
    }

    for pair in reported.windows(2) {
        assert!(
            pair[1] >= pair[0],
            "moving the ball further out cut its reported deviation from {} to {}",
            pair[0],
            pair[1]
        );
    }
    let at_the_origin = reported[0];
    let furthest_out = *reported.last().expect("a row for every distance");
    assert!(
        furthest_out >= at_the_origin * 10.0,
        "the same ball reports {furthest_out} at {} away and {at_the_origin} at the origin, \
         which is not the placement showing up in the number",
        DISTANCES[DISTANCES.len() - 1]
    );
    println!(
        "the same {RADIUS} ball reports {at_the_origin} at the origin and {furthest_out} at \
         {} away",
        DISTANCES[DISTANCES.len() - 1]
    );
}

/// An empty log is not a claim that the points landed on the surface.
///
/// This is the sentence the consumer report turned on, so it is pinned rather
/// than explained. §25.3: the mesher makes no decision, so under
/// `Policy::certified()` it writes nothing whatever it was handed — and the
/// balls here are deformed by a large fraction of their own radius. Anyone
/// reading the log to find out how good a mesh is has to read
/// `Mesh::deviation` instead.
#[test]
fn a_ball_meshed_far_from_the_origin_leaves_the_log_empty_and_says_so_on_the_mesh_instead() {
    const RADIUS: f64 = 1e-3;
    let mut deformed = 0;
    for distance in [1e11, 1e12, 1e13] {
        for (_, placement) in placements(distance) {
            let centre = placement.apply_to_point(Point::origin());
            for coarseness in COARSENESSES {
                let deflection = Deflection::of(RADIUS * coarseness).expect("a length");
                let mut model = a_model();
                let ball = model.add_ball(RADIUS, &placement).expect("a ball");
                let mut log = ProvenanceLog::new();
                let Ok(mesh) = model.solid(ball).expect("this model made the key").mesh(
                    deflection,
                    &Policy::certified(),
                    &mut log,
                ) else {
                    continue;
                };
                assert!(
                    log.is_empty(),
                    "the mesher settled every sign it was asked about, so it has nothing to \
                     record, got {:?}",
                    log.records()
                );
                if how_far_the_mesh_strays_from_the_ball(&mesh, centre, RADIUS) > RADIUS * 0.1 {
                    deformed += 1;
                }
            }
        }
    }
    assert!(
        deformed > 5,
        "only {deformed} of these balls came back deformed by a tenth of their radius, so an \
         empty log here says nothing"
    );
    println!("{deformed} balls deformed by over a tenth of their radius, every log empty");
}

fn meshed_ball(radius: f64, placement: &Isometry, deflection: f64) -> Option<Mesh> {
    let mut model = a_model();
    let ball = model.add_ball(radius, placement).expect("a ball");
    model
        .solid(ball)
        .expect("this model made the key")
        .mesh(
            Deflection::of(deflection).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .ok()
}

/// The furthest any point of any triangle gets from the ball's surface.
///
/// `|p - centre|` is convex over a flat triangle, so it is largest at a corner
/// and smallest at the triangle's nearest point to the centre. Both ends count:
/// a corner pushed off the sphere outwards and a flat triangle sagging inwards
/// are the same failure seen from either side.
fn how_far_the_mesh_strays_from_the_ball(mesh: &Mesh, centre: Point, radius: f64) -> f64 {
    let mut worst = 0.0_f64;
    for triangle in mesh.triangles() {
        let [a, b, c] = triangle
            .corners()
            .map(|corner| mesh.points()[corner as usize]);
        let furthest = [a, b, c]
            .iter()
            .map(|corner| (*corner - centre).length())
            .fold(0.0_f64, f64::max);
        let nearest = (closest_point_on_a_triangle(centre, a, b, c) - centre).length();
        worst = worst.max(furthest - radius).max(radius - nearest);
    }
    worst
}

/// The point of triangle `abc` nearest to `p`.
///
/// The seven cases are the triangle's Voronoi regions — three corners, three
/// sides, and the inside — taken in an order that lets each one be ruled out
/// with dot products already computed. Ericson, *Real-Time Collision
/// Detection*, §5.1.5.
fn closest_point_on_a_triangle(p: Point, a: Point, b: Point, c: Point) -> Point {
    let ab = b - a;
    let ac = c - a;

    let ap = p - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let along_ab = d1 * d4 - d3 * d2;
    if along_ab <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return a + ab * (d1 / (d1 - d3));
    }

    let along_ac = d5 * d2 - d1 * d6;
    if along_ac <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return a + ac * (d2 / (d2 - d6));
    }

    let along_bc = d3 * d6 - d5 * d4;
    if along_bc <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }

    let total = along_ab + along_ac + along_bc;
    a + ab * (along_ac / total) + ac * (along_ab / total)
}
