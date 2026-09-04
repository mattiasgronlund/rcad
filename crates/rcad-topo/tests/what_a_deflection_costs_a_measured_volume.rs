//! How far a curved solid's measured volume sits from the one it really has.
//!
//! `Mesh::deviation`'s doc publishes a bound on that distance, and §15.6 says a
//! published bound needs a test that could fail. This is it. The true answers
//! here are known in closed form — `4πr³/3` and `4πr²` for a ball, `πr²h` and
//! `2πr(r + h)` for a bar — so nothing is checked against the kernel's own idea
//! of what it did.
//!
//! The bound is worked out **here**, from the geometry the shape was built with,
//! and not read back off the `Measurement`. Checking a kernel's answer against
//! the same kernel's `volume_error` would pass however wrong both were.
//!
//! `docs/decisions.md` §47.4 item 1 is why the first thing every check does is
//! ask whether the bound is a number. A bound of `f64::INFINITY` cannot be
//! violated, and a suite that asserts against one has measured nothing.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Measurement, Model, Rule, SolidKey};

const PI: f64 = core::f64::consts::PI;

/// The deflections every shape below is measured at.
///
/// Six of them over three decades, because two points fix a line through
/// anything and say nothing about whether the thing is a line.
const HOW_FINE: [f64; 6] = [1.0, 0.3, 0.1, 0.03, 0.01, 0.003];

/// The bound `Mesh::deviation` publishes: `skin · d · (R / (R − d))²`.
///
/// `skin` is the area of the triangles, `d` how far any of them may sit from the
/// surface, and `R` the smallest radius any curved face of the solid bends at.
/// Infinite when the mesh may be a whole radius out, which is the case that has
/// to be checked for before anything is asserted against it.
fn how_far_a_measured_volume_can_be_out(skin: f64, deflection: f64, smallest_radius: f64) -> f64 {
    let inner = smallest_radius - deflection;
    if inner <= 0.0 {
        return f64::INFINITY;
    }
    let widening = smallest_radius / inner;
    skin * deflection * widening * widening
}

/// A ball, a tall bar and a short wide one, at four sizes each.
///
/// Every one carries the volume it really has and the radius its curved faces
/// bend at, both worked out from what it was built with.
fn shapes_whose_volume_is_known() -> (Model, Vec<(String, SolidKey, f64, f64)>) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let mut shapes = Vec::new();
    for radius in [0.25, 1.0, 3.0, 10.0] {
        let ball = model
            .add_ball(radius, &Isometry::identity())
            .expect("a ball");
        shapes.push((
            format!("ball of radius {radius}"),
            ball,
            4.0 / 3.0 * PI * radius * radius * radius,
            radius,
        ));
        for height in [0.5, 5.0] {
            let bar = model
                .add_round_bar(radius, height, &Isometry::identity())
                .expect("a bar");
            shapes.push((
                format!("bar {radius} across and {height} tall"),
                bar,
                PI * radius * radius * height,
                radius,
            ));
        }
    }
    (model, shapes)
}

fn measured(model: &Model, solid: SolidKey, how_fine: f64) -> Measurement {
    model
        .solid(solid)
        .measure(
            Deflection::of(how_fine).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("measuring at a deflection of {how_fine}: {why}"))
}

/// The bound holds over every shape whose volume this suite knows in closed
/// form, and it holds with room to spare that is stated rather than hoped for.
///
/// Two numbers come out of this and both are asserted. The bound is never
/// broken, which is what makes it a bound; and somewhere in the family the real
/// error gets within a stated share of it, which is what stops it being a bound
/// nothing can violate.
#[test]
fn a_measured_volume_sits_inside_the_published_bound_over_balls_and_bars() {
    let (model, shapes) = shapes_whose_volume_is_known();

    let mut worst_share = 0.0f64;
    let mut worst_at = String::new();
    let mut too_coarse_to_bound = 0;
    let mut bounded = 0;
    for (what, solid, true_volume, radius) in shapes {
        for how_fine in HOW_FINE {
            let measured = measured(&model, solid, how_fine);
            let bound =
                how_far_a_measured_volume_can_be_out(measured.surface_area(), how_fine, radius);
            if !bound.is_finite() {
                assert!(
                    how_fine >= radius,
                    "the bound is only allowed to give up when the mesh may be a whole \
                     radius out, and a {what} at a deflection of {how_fine} is not that"
                );
                too_coarse_to_bound += 1;
                continue;
            }
            bounded += 1;

            let out_by = (true_volume - measured.volume()).abs();
            assert!(
                out_by <= bound,
                "a {what} measured at a deflection of {how_fine} came to {} against the \
                 {true_volume} it really holds, which is {out_by} out of a bound of {bound}",
                measured.volume()
            );
            let share = out_by / bound;
            if share > worst_share {
                worst_share = share;
                worst_at = format!("a {what} at a deflection of {how_fine}");
            }
        }
    }

    assert!(
        too_coarse_to_bound > 0 && bounded > 60,
        "the family has to reach both sides of the bound's own limit, and it bounded \
         {bounded} and gave up on {too_coarse_to_bound}"
    );
    // A fifth of the bound is what a ball of radius three at a deflection of
    // 0.003 really loses, and it is the number `Mesh::deviation`'s doc
    // publishes as the bound's slack. Falling under this means the bound has
    // got looser than the doc says it is, which is the failure this catches —
    // the same failure as breaking the bound, from the other side.
    assert!(
        worst_share > 0.18,
        "the bound is meant to be within about five times what really goes missing, and \
         the closest anything came was {worst_share} at {worst_at}"
    );
    println!("worst share of the bound used: {worst_share:.4} at {worst_at}");
}

/// What goes missing falls **in step with** the deflection, not with its square.
///
/// Fitted, not eyeballed off two points: a least-squares line through
/// `log(missing)` against `log(deflection)` over the five finest deflections of
/// every shape in the family. A slope of one is the claim; the test is what says
/// the data supports it.
#[test]
fn the_volume_a_mesh_misses_falls_in_step_with_the_deflection() {
    let (model, shapes) = shapes_whose_volume_is_known();

    let mut steepest = f64::MIN;
    let mut shallowest = f64::MAX;
    for (what, solid, true_volume, _radius) in shapes {
        let mut points = Vec::new();
        for how_fine in [0.1, 0.03, 0.01, 0.003, 0.001] {
            let out_by = (true_volume - measured(&model, solid, how_fine).volume()).abs();
            assert!(
                out_by > 0.0,
                "a {what} at a deflection of {how_fine} claims to have been measured exactly"
            );
            points.push((how_fine.ln(), out_by.ln()));
        }
        let slope = slope_of_the_best_line_through(&points);
        assert!(
            (0.9..=1.1).contains(&slope),
            "a {what} loses volume as the deflection to the power {slope}, and the doc \
             says that power is one"
        );
        steepest = steepest.max(slope);
        shallowest = shallowest.min(slope);
    }
    println!("slopes ran from {shallowest:.3} to {steepest:.3}");
}

fn slope_of_the_best_line_through(points: &[(f64, f64)]) -> f64 {
    let count = points.len() as f64;
    let mean_across = points.iter().map(|(across, _)| across).sum::<f64>() / count;
    let mean_up = points.iter().map(|(_, up)| up).sum::<f64>() / count;
    let mut leaning = 0.0;
    let mut spread = 0.0;
    for (across, up) in points {
        leaning += (across - mean_across) * (up - mean_up);
        spread += (across - mean_across) * (across - mean_across);
    }
    leaning / spread
}

/// The bound is on the **distance**, and this is the shape that says it has to
/// be: a block with a round hole measures **bigger** than it is.
///
/// A round bar's wall bends away from its material, so its chords lie inside the
/// bar and the sum over them is short. A hole's wall bends the other way: the
/// chords cut into the material the block is made of, so the mesh solid is the
/// block minus a prism inscribed in the hole, which is more block than there
/// really is. Same shell, same thickness, opposite sign.
#[test]
fn a_block_with_a_round_hole_measures_bigger_than_it_is() {
    const RADIUS: f64 = 1.0;
    let true_volume = 512.0 - PI * RADIUS * RADIUS * 8.0;

    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let block = model
        .add_cube(8.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a block eight across");
    let bar = model
        .add_round_bar(
            RADIUS,
            12.0,
            &Isometry::translation(Vector::new(4.0, 4.0, -2.0)).expect("a shift"),
        )
        .expect("a bar through the middle of it");
    let holed = model
        .fuse(
            &[block],
            &[bar],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a block with a hole through it");
    let holed = holed.solids()[0];

    for how_fine in [0.1, 0.03, 0.01, 0.003] {
        let measured = measured(&model, holed, how_fine);
        assert!(
            measured.volume() > true_volume,
            "the mesh of a hole is a prism inside it, so at a deflection of {how_fine} the \
             block has to come out bigger than the {true_volume} it is, and it came out {}",
            measured.volume()
        );
        let bound = how_far_a_measured_volume_can_be_out(measured.surface_area(), how_fine, RADIUS);
        assert!(
            bound.is_finite(),
            "nothing is worth asserting against an infinite bound (§47.4)"
        );
        assert!(
            measured.volume() - true_volume <= bound,
            "the block is {} over the {true_volume} it holds, against a bound of {bound}",
            measured.volume() - true_volume
        );
    }
}

/// A mesh never strays further than the deflection it was asked for, which is
/// what lets the published bound be worked out before anything is meshed.
///
/// Only under a certified policy. A tolerant one is allowed to mesh **coarser**
/// than asked when the arithmetic cannot reach the request, and then the
/// deflection is not an upper bound on anything.
#[test]
fn a_certified_mesh_never_strays_further_than_the_deflection_it_was_asked_for() {
    let (model, shapes) = shapes_whose_volume_is_known();
    for (what, solid, _volume, _radius) in shapes {
        for how_fine in HOW_FINE {
            let deviation = model
                .solid(solid)
                .mesh(
                    Deflection::of(how_fine).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a mesh")
                .deviation()
                .magnitude();
            assert!(
                deviation <= how_fine,
                "a {what} asked for {how_fine} charged itself {deviation}"
            );
        }
    }
}

/// The bound gives up exactly where it says it does, and this is what "gives up"
/// looks like: not a large number, an infinite one.
///
/// The case that matters is a cone that ends in a point, whose radius of
/// curvature goes to zero at the apex. There is no public builder for one here,
/// so what is checked is the arithmetic of the bound itself at the radius such a
/// face reaches. §47.4 item 1 is a volume checked against exactly this number.
#[test]
fn the_bound_is_infinite_where_a_surface_bends_tighter_than_the_mesh_strays() {
    assert!(how_far_a_measured_volume_can_be_out(100.0, 0.1, 0.0).is_infinite());
    assert!(how_far_a_measured_volume_can_be_out(100.0, 0.1, 0.1).is_infinite());
    assert!(how_far_a_measured_volume_can_be_out(100.0, 0.1, 0.05).is_infinite());
    assert!(how_far_a_measured_volume_can_be_out(100.0, 0.1, 0.2).is_finite());
}
