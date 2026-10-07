//! `(−B ± √Δ)/A` is ascending in that order only while `A` is positive. A
//! cone's `A` is `‖D × w‖² − slope²(D·w)²`, which turns negative for a line
//! that points further into the cone than its own rulings do — the axis
//! itself, for instance, is steeper than every ruling a cone of finite slope
//! has. [`WhereALineMeetsIt::At`] promises its steps "ascending" regardless,
//! and the first test below checks that promise against a line parallel to a
//! slope-one cone's axis, where `A = -1`.
//!
//! The second test runs the same line through [`meet_of_three_surfaces`]
//! instead, built from two crossing planes — see its own doc for what that
//! found about the module's `n1 × n2` claim.

use rcad_geom::{
    Axis, Cone, MeetOfThree, MeetsAt, Plane, Point, Surface, Vector, WhereALineMeetsIt,
    meet_of_three_surfaces, where_a_line_meets_a_surface,
};
use rcad_predicates::{Interval, Policy, ProvenanceLog};

fn exactly(triple: [f64; 3]) -> [Interval; 3] {
    triple.map(Interval::exactly)
}

fn a_slope_one_cone() -> Surface {
    Surface::Cone(Cone::new(Axis::z(), 1.0).expect("a slope"))
}

/// Every step the line meets the surface at, in the order it came back, each
/// one exactly — a step that is a box rather than a number is a failure here,
/// because this cone's roots are whole numbers the arithmetic reaches with no
/// rounding.
fn steps(answer: &WhereALineMeetsIt) -> Vec<f64> {
    let WhereALineMeetsIt::At(found) = answer else {
        panic!("expected places along the line, got {answer:?}");
    };
    found
        .iter()
        .map(|step| {
            let MeetsAt::Here(step) = step else {
                panic!("expected a settled place, got {step:?}");
            };
            assert_eq!(
                step.lo(),
                step.hi(),
                "a slope-one cone's roots here are whole numbers, not a box"
            );
            step.lo()
        })
        .collect()
}

/// A line parallel to a slope-one cone's own axis is steeper than every one
/// of its rulings — a ruling climbs one unit of height per unit out, and the
/// axis climbs without ever going out — which is exactly where `A` in
/// `(−B ± √Δ)/A` turns negative (Q1259's road). This line, `x = 1, y = 0`,
/// meets the cone where its radius is one: `z = −1` and `z = 1`.
///
/// Without the fix this came back as `[1.0, -1.0]` — descending, because
/// dividing `(−B − √Δ)` and `(−B + √Δ)` by a negative `A` turns their order
/// around. [`WhereALineMeetsIt::At`]'s doc promises ascending, so this must
/// be `[-1.0, 1.0]`.
#[test]
fn a_line_parallel_to_a_slope_one_cones_axis_meets_it_ascending() {
    let found = steps(
        &where_a_line_meets_a_surface(
            exactly([1.0, 0.0, 0.0]),
            exactly([0.0, 0.0, 1.0]),
            &a_slope_one_cone(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the line crosses the cone twice"),
    );
    assert_eq!(found, vec![-1.0, 1.0], "the steps must come back ascending");
}

/// Two planes crossing in the same line as the test above — `x = 1` and
/// `y = 0` meet along `x = 1, y = 0` — against the same cone, read through
/// [`meet_of_three_surfaces`] instead of [`where_a_line_meets_a_surface`].
///
/// **What this found, and why it is not "ascending along `n1 × n2`".** The
/// module doc's `n1 × n2` is the *un-normalised* cross product of the two
/// planes' stored normals in [`Surface::how_it_sorts_against`]'s fixed
/// order, but [`the_line_two_planes_cross_in`]'s own line divides every
/// coordinate by one signed component of that same cross product (its
/// largest in magnitude), so the line's *physical* direction of ascending
/// `step` is `n1 × n2` turned so that component is positive — not
/// necessarily `n1 × n2` itself. For this pair that component is negative
/// (`-1`), so the physical direction is `-(n1 × n2)` and the points below
/// come back ascending in `z`, the opposite of raw `n1 × n2 = (0, 0, -1)`.
///
/// That divisor sign is independent of the `A < 0` bug this file is about —
/// it is `the_line_two_planes_cross_in`'s own doubling of the parametrising
/// direction and divisor, used (deliberately, for `fuse(A, B) == fuse(B,
/// A)`) by every two-plane meet, curved or not. Both corners are still
/// checked against the closed form directly, which is the claim this test
/// can actually stand behind.
#[test]
fn two_planes_and_a_slope_one_cone_meet_where_the_closed_form_says() {
    let one = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("a normal with a length");
    let other = Plane::from_point_and_normal(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
        .expect("a normal with a length");
    let cone = a_slope_one_cone();

    let answer = meet_of_three_surfaces(
        [&Surface::Plane(one), &Surface::Plane(other), &cone],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("two planes crossing the cone's axis one unit out meet it twice");
    let MeetOfThree::At(points) = &answer else {
        panic!("expected two points, got {answer:?}");
    };
    assert_eq!(points.len(), 2, "the line meets the cone twice");

    let placed: Vec<[f64; 3]> = points
        .iter()
        .map(|point| {
            let enclosure = point.enclosure();
            enclosure.map(|axis| {
                assert_eq!(
                    axis.lo(),
                    axis.hi(),
                    "this corner is a whole number, not a box"
                );
                axis.lo()
            })
        })
        .collect();
    let mut heights: Vec<f64> = placed.iter().map(|place| place[2]).collect();
    heights.sort_by(f64::total_cmp);
    assert_eq!(
        heights,
        vec![-1.0, 1.0],
        "both corners must be where the cone's radius is one, got {placed:?}"
    );
}
