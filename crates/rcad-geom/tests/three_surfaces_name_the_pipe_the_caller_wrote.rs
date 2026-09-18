//! A three-surface meet must be about the same pipe `Cylinder::side_of` is
//! about, down to the last bit.
//!
//! `docs/decisions.md` §118.10 made the sign predicate homogeneous: it takes the
//! sign of `|(p − o) × w|² − r²‖w‖²`, so a pipe is the points exactly `r` from
//! its axis whatever length the stored direction rounded to. §27.1's rule is
//! that an intersector's polynomials are written in the same stored numbers the
//! sign predicate reads, with no normalization anywhere — otherwise the two
//! halves of a meet are certified about two surfaces a rounding apart.
//!
//! `three_surfaces.rs` writes a pipe out twice, and both sites squared the
//! radius against no direction at all: `how_a_pipe_is_written` for the quadric
//! whose difference with another quadric is a plane, and
//! `how_a_pipe_cuts_the_line` for the quadratic a line cuts out of a pipe. Each
//! was therefore about the pipe of radius `r/‖w‖`. These tests are the two
//! routes in, one each.
//!
//! # Where the teeth are
//!
//! Every pipe below runs along `(1, 1, 0)`, which does **not** store a unit
//! vector: `‖w‖²` comes back a rounding off one, and `the_axis_direction_these_
//! tests_lean_on_does_not_store_a_unit_vector` pins that before anything else is
//! asked. The gap between the two pipes is then about `1e-16·r`, which decides
//! nothing anywhere except where a surface is exactly touched.
//!
//! So both questions below are asked at a **tangency**, where the answer changes
//! from "a whole line" to "nothing at all" the moment the line stops lying on
//! the pipe. Nothing here is an inequality with room in it.
//!
//! **Two spellings of one direction are never compared against each other.**
//! `Axis::new` scales by the exact power of two that puts the widest component
//! in `[1, 2)` before it divides by the length, so `(1, 1, 0)` and `(3, 3, 0)`
//! store the same bits and a test that compared them would pass while both were
//! wrong. Every assertion is against the
//! geometry: a line that really does lie on the pipe's wall, and a point that is
//! really on its skin.
//!
//! The controls are the same two questions asked about `(0, 1, 0)`, which stores
//! exactly. They passed before the fix and after it, so a red above is about the
//! direction's length and not about the shape of the question.

use rcad_geom::{
    Axis, Cylinder, MeetOfThree, Plane, Point, Surface, Vector, meet_of_three_surfaces,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface};

/// The radius of the pipe every test builds first, and the height of the wall
/// line each of them asks about. A quarter is exact, so nothing below rounds
/// except the axis direction.
const RADIUS: f64 = 0.25;

/// The radius of the second pipe, and `RADIUS + WIDER` is how far apart the two
/// axes sit when they touch. Both exact.
const WIDER: f64 = 0.5;

/// The precondition the whole file leans on: `(1, 1, 0)` does not store a
/// direction of length one, so `r²` and `r²‖w‖²` are two different numbers.
///
/// Asserted rather than assumed, because if the normalize ever landed on a
/// direction squaring exactly to one, every test below would go green while
/// saying nothing at all.
#[test]
fn the_axis_direction_these_tests_lean_on_does_not_store_a_unit_vector() {
    let stored = leaning_axis(0.0).direction().components();
    assert_eq!(stored[2], 0.0, "the direction has to lie flat in xy");
    let squared_length = stored[0] * stored[0] + stored[1] * stored[1] + stored[2] * stored[2];
    assert_ne!(
        squared_length, 1.0,
        "{stored:?} squares to {squared_length}, which is what gives these tests their teeth",
    );

    assert_eq!(
        upright_axis(0.0).direction().components(),
        [0.0, 1.0, 0.0],
        "and the control's direction stores exactly",
    );
}

/// Two planes crossing in a line down a pipe's own wall share that whole line,
/// and the pipe's stored direction cannot be unit for it.
///
/// This is `how_a_pipe_cuts_the_line`'s route: two planes hand over a line, and
/// the pipe turns it into `A·s² + 2B·s + C`. The line `x = y, z = r` runs along
/// the pipe's axis direction, so `A` and `B` are exactly zero and the answer is
/// the sign of `C = ‖V × w‖² − m²r²(w·w)` alone. `V × w` comes to `m·r·‖w‖`, so
/// `C` is exactly zero and the line is on the wall.
///
/// Squaring the radius against `m²` alone instead leaves `m²r²(‖w‖² − 1)`, which
/// is about `1e-17` here and not zero, and the answer becomes
/// [`MeetOfThree::Nowhere`] — the kernel denying that a line lying on a pipe
/// touches it.
#[test]
fn a_line_down_the_wall_of_a_pipe_about_a_leaning_axis_is_more_than_a_point() {
    let pipe = pipe_about(leaning_axis(0.0), RADIUS);
    assert_eq!(
        pipe_of(&pipe).side_of(a_point_on_the_wall_line()),
        Decision::Zero,
        "the sign predicate puts this point on the skin, so the meet has to agree",
    );

    assert_eq!(
        meet([&flat_at_the_wall_height(), &the_plane_x_equals_y(), &pipe]),
        Ok(MeetOfThree::MoreThanAPoint),
    );
}

/// The control: the same question about a direction that stores exactly.
#[test]
fn a_line_down_the_wall_of_a_pipe_about_an_upright_axis_is_more_than_a_point() {
    let pipe = pipe_about(upright_axis(0.0), RADIUS);
    assert_eq!(
        meet([
            &flat_at_the_wall_height(),
            &the_plane_x_equals_zero(),
            &pipe,
        ]),
        Ok(MeetOfThree::MoreThanAPoint),
    );
}

/// Two pipes running the same way and touching along one line, with a plane
/// through that line, share the whole of it.
///
/// This is `how_a_pipe_is_written`'s route, and the only one that reads a pipe's
/// constant term: two quadrics that bend the same way have a flat difference,
/// and the meet is that flat surface crossed with the stored plane and then with
/// one of the pipes. Axes `r₁ + r₂` apart touch along `z = r₁`, and the
/// difference of the two constants — `(w·w)(h² − r₂² + r₁²)` against a leaning
/// term of `2(w·w)h` — puts the flat surface exactly there.
///
/// Take the `w·w` off the radii and the two constants differ by
/// `(w·w)h² − r₂² + r₁²` instead, which moves the flat surface off `z = r₁` by
/// about `1e-17`. The line it then hands on misses both pipes, and the answer
/// becomes [`MeetOfThree::Nowhere`].
#[test]
fn two_pipes_about_leaning_axes_touching_along_a_line_share_the_whole_of_it() {
    let thin = pipe_about(leaning_axis(0.0), RADIUS);
    let fat = pipe_about(leaning_axis(RADIUS + WIDER), WIDER);
    for both in [&thin, &fat] {
        assert_eq!(
            pipe_of(both).side_of(a_point_on_the_wall_line()),
            Decision::Zero,
            "the touch line is on both pipes' skins as the sign predicate reads them",
        );
    }

    assert_eq!(
        meet([&the_plane_x_equals_y(), &thin, &fat]),
        Ok(MeetOfThree::MoreThanAPoint),
    );
}

/// The control: the same question about directions that store exactly.
#[test]
fn two_pipes_about_upright_axes_touching_along_a_line_share_the_whole_of_it() {
    let thin = pipe_about(upright_axis(0.0), RADIUS);
    let fat = pipe_about(upright_axis(RADIUS + WIDER), WIDER);
    assert_eq!(
        meet([&the_plane_x_equals_zero(), &thin, &fat]),
        Ok(MeetOfThree::MoreThanAPoint),
    );
}

/// An axis along `(1, 1, 0)` through `(0, 0, height)`.
fn leaning_axis(height: f64) -> Axis {
    Axis::new(Point::new(0.0, 0.0, height), Vector::new(1.0, 1.0, 0.0))
        .expect("a direction with a length")
}

/// An axis along `(0, 1, 0)` through `(0, 0, height)`, whose direction stores
/// exactly.
fn upright_axis(height: f64) -> Axis {
    Axis::new(Point::new(0.0, 0.0, height), Vector::new(0.0, 1.0, 0.0))
        .expect("a direction with a length")
}

fn pipe_about(axis: Axis, radius: f64) -> Surface {
    Surface::Cylinder(Cylinder::new(axis, radius).expect("a positive radius"))
}

fn pipe_of(surface: &Surface) -> Cylinder {
    let Surface::Cylinder(pipe) = surface else {
        panic!("this file builds pipes and nothing else");
    };
    *pipe
}

/// `z = RADIUS`, the plane the wall line lies in. Both axes above lie flat in
/// the `xy` plane, so this height is exactly one radius from the first of them.
fn flat_at_the_wall_height() -> Surface {
    plane([0.0, 0.0, 1.0], [0.0, 0.0, RADIUS])
}

/// `x = y`, which holds the wall line of a pipe about the leaning axis.
fn the_plane_x_equals_y() -> Surface {
    plane([1.0, -1.0, 0.0], [0.0, 0.0, 0.0])
}

/// `x = 0`, which holds the wall line of a pipe about the upright axis.
fn the_plane_x_equals_zero() -> Surface {
    plane([1.0, 0.0, 0.0], [0.0, 0.0, 0.0])
}

/// A point on the wall line, and exactly [`RADIUS`] from the leaning axis: the
/// axis lies flat in the `xy` plane and this sits on the `z` axis above its
/// origin, so the distance is the height and no subtraction rounds.
fn a_point_on_the_wall_line() -> PredicatePoint {
    PredicatePoint::from([0.0, 0.0, RADIUS])
}

fn plane(normal: [f64; 3], through: [f64; 3]) -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(
            Point::new(through[0], through[1], through[2]),
            Vector::new(normal[0], normal[1], normal[2]),
        )
        .expect("a normal with length"),
    )
}

fn meet(surfaces: [&Surface; 3]) -> Result<MeetOfThree, rcad_geom::Error> {
    meet_of_three_surfaces(surfaces, &Policy::certified(), &mut ProvenanceLog::new())
}
