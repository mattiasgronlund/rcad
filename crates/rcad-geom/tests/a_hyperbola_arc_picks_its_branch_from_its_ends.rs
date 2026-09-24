//! A run along one branch of a hyperbola is walked on the branch its own two
//! ends are on, whatever half-space picks it.
//!
//! The plane `x = 1` against the double cone `x² + y² = z²` is `z² − y² = 1`:
//! two branches, `(1, sinh σ, ±cosh σ)`, one in each nappe. A half-space
//! through the chord between two ends on the upper branch holds the arc below
//! that chord — and the whole of the lower branch with it, and not the upper
//! branch's vertex. So a walker that asked the picker at each branch's vertex
//! chose the lower branch and walked an arc across the apex onto the other
//! nappe. The fuse used to write exactly such a picker (e2cd175d fixed that
//! writer), and `walk` is public and takes any run, so the walker has to get it
//! right on its own.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Error, HalfSpace, NamedRun, Picker, Plane, Point, Surface,
    Vector, walk,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use smallvec::smallvec;

fn the_plane_x_equals_one() -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
            .expect("a normal with a length"),
    )
}

fn a_right_angled_double_cone() -> Surface {
    Surface::Cone(Cone::new(Axis::z(), 1.0).expect("a slope of one is 45 degrees"))
}

/// The point `σ` along the branch whose nappe is `nappe` (`+1` up, `−1` down).
fn on_the_branch(nappe: f64, along: f64) -> Point {
    Point::new(1.0, along.sinh(), nappe * along.cosh())
}

/// The half-space through the chord from `start` to `end`, facing the side the
/// arc between them bulges to — which, for the upper branch, is down and away
/// from the apex's plane.
fn in_front_of_the_chord(start: Point, end: Point) -> Picker {
    let chord = end - start;
    let normal = Vector::new(0.0, chord.z(), -chord.y());
    let offset = normal.dot(start - Point::new(0.0, 0.0, 0.0));
    Picker::from(HalfSpace::new(normal, offset).expect("a chord with a length"))
}

fn a_run_between(start: Point, end: Point, picker: Picker) -> NamedRun {
    NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: a_right_angled_double_cone(),
            picked_by: smallvec![picker],
        },
        [
            PredicatePoint::from(start.coordinates()),
            PredicatePoint::from(end.coordinates()),
        ],
    )
    .expect("a run that does not close")
}

fn walked(run: &NamedRun) -> Result<rcad_geom::PointsAlongACrossing, Error> {
    walk(
        run,
        [&the_plane_x_equals_one(), &a_right_angled_double_cone()],
        Deflection::of(1.0 / 1000.0).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

#[test]
fn an_arc_picked_by_its_own_chord_is_walked_on_the_branch_its_ends_are_on() {
    for nappe in [1.0, -1.0] {
        let start = on_the_branch(nappe, 1.0);
        let end = on_the_branch(nappe, 2.0);
        let picker = if nappe > 0.0 {
            in_front_of_the_chord(start, end)
        } else {
            in_front_of_the_chord(end, start)
        };
        let drawn = walked(&a_run_between(start, end, picker)).expect("an arc with two ends");

        assert!(
            drawn.points().len() >= 8,
            "an arc two tall at a thousandth of that takes more than eight pieces, got {}",
            drawn.points().len()
        );
        let room = drawn.deviation().magnitude();
        for point in drawn.points() {
            assert!(
                point.z() * nappe > 0.0,
                "a point of the arc is on the same side of the apex as its ends \
                 (nappe {nappe}): {point:?}"
            );
            assert!(
                (point.x() - 1.0).abs() <= room,
                "a point of the arc lies in the plane `x = 1`: {point:?}"
            );
            assert!(
                (point.z() * point.z() - point.y() * point.y() - 1.0).abs() <= room * 8.0,
                "and on the cone, which there is `z² − y² = 1`: {point:?}"
            );
            assert!(
                point.y() >= start.y() - room && point.y() <= end.y() + room,
                "and between its two ends: {point:?}"
            );
        }
    }
}

/// Both ends of one run are on one branch, so a run with an end on each is not
/// one run at all, and there is no arc to walk.
#[test]
fn a_run_with_an_end_on_each_branch_is_refused() {
    let start = on_the_branch(1.0, 1.0);
    let end = on_the_branch(-1.0, 2.0);
    let answer = walked(&a_run_between(
        start,
        end,
        in_front_of_the_chord(start, end),
    ));
    assert!(
        matches!(answer, Err(Error::CannotNameWhichPieceOfACrossingThisIs)),
        "two ends in two nappes name no one branch, got {answer:?}"
    );
}
