//! §19 for the evaluator: where a crossing sits must not change how finely it
//! is walked.
//!
//! `walk` carries two numbers about a conic and they are not the same one. The
//! **size everything was worked out from** is the biggest magnitude in the
//! chain — a plane's offset, a ball's centre, an axis origin — and it is the
//! right scale for a rounding, which is what
//! `POINT_ALONG_A_CROSSING_ROUNDOFF` is a multiple of. The **reach** is how far
//! the conic goes from its own centre, and that is what says how sharply it
//! bends.
//!
//! Using the first for both is the mistake this file is here to catch, and it
//! is invisible until something reads the point count. The cap circle at the
//! top of a bar ten tall bends exactly as the one at the bottom does; charged
//! against the ten it came back walked into more pieces than the bottom one,
//! and the middle of the meshed bar was no longer half way up it.

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, NamedRun, Plane, Point, Sphere, Surface, Vector, walk,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};

/// The circle where the plane `z = height` cuts a pipe of radius three about
/// the z axis, walked from the corner at `(3, 0, height)`.
fn a_cap_circle_of_a_bar(height: f64) -> Vec<Point> {
    let wall = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a pipe"));
    let cap = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal"),
    );
    let corner = PredicatePoint::from([3.0, 0.0, height]);
    let run = NamedRun::new(EdgeRun::RightRoundWhereItsFacesCross, [corner, corner])
        .expect("the same corner twice does come back to where it started");

    walk(
        &run,
        [&wall, &cap],
        Deflection::of(0.05).expect("a length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a cap circle is a circle")
    .points()
    .to_vec()
}

/// Two cap circles of one bar are the same circle at two heights, so they are
/// walked into the same number of pieces.
///
/// A bar ten tall is where the two numbers are furthest apart: the bottom cap's
/// whole chain lives inside a radius of three, and the top cap's reaches ten up
/// the axis while bending exactly as much.
#[test]
fn two_cap_circles_of_one_bar_are_walked_into_the_same_number_of_pieces() {
    let at_the_bottom = a_cap_circle_of_a_bar(0.0);
    assert!(
        at_the_bottom.len() > 8,
        "a circle three across at a deflection of a twentieth takes more than \
         eight pieces, and took {}",
        at_the_bottom.len()
    );
    for height in [1.0, 10.0, 1000.0] {
        let up_there = a_cap_circle_of_a_bar(height);
        assert_eq!(
            up_there.len(),
            at_the_bottom.len(),
            "the cap circle {height} up a bar was walked into {} pieces and the \
             one at the bottom into {}",
            up_there.len(),
            at_the_bottom.len()
        );
    }
}

/// The same claim for a ball, where the reach is the circle's own radius and
/// the size is wherever the ball has been put.
#[test]
fn a_circle_cut_out_of_a_ball_is_walked_the_same_wherever_the_ball_sits() {
    let cut = |along: f64| {
        let ball = Surface::Sphere(
            Sphere::new(Point::new(along, 0.0, 0.0), 5.0).expect("a ball of radius five"),
        );
        let flat = Surface::Plane(
            Plane::from_point_and_normal(Point::new(along, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal"),
        );
        let corner = PredicatePoint::from([along + 5.0, 0.0, 0.0]);
        let run = NamedRun::new(EdgeRun::RightRoundWhereItsFacesCross, [corner, corner])
            .expect("the same corner twice does come back to where it started");
        walk(
            &run,
            [&ball, &flat],
            Deflection::of(0.1).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a plane through a ball's centre cuts a great circle")
        .points()
        .len()
    };

    let here = cut(0.0);
    assert!(
        here > 8,
        "a great circle of radius five takes {here} pieces"
    );
    for along in [1.0, 100.0, 1e6] {
        assert_eq!(
            cut(along),
            here,
            "the same ball {along} along the x axis was walked into {} pieces \
             and the one at the origin into {here}",
            cut(along)
        );
    }
}
