//! A caller who names nothing but `rcad::` can hold the answer to "where do
//! these two surfaces cross".
//!
//! `docs/decisions.md` §24.15 exports the intersector, [`SurfaceMeet`] and
//! [`NamedRun`]: the entry point has no topology in its signature once §24.9
//! splits the type, so exporting it costs nothing, and quadric intersection
//! with certified results and an explicit refusal has users who are not kernel
//! users. `FaceMeet` and its `along` bookkeeping stay inside `rcad-topo`,
//! because those name input edges.
//!
//! The intersector itself lands in M3's first stage. The vocabulary lands at the
//! gate, because both stages and the mesher write against it (§13 rule 4), and
//! this file is what says a caller outside the workspace can hold it.
//!
//! Nothing in this file imports anything but `rcad`. That is the test.

use rcad::{
    Cylinder, EdgeRun, NamedRun, Plane, Point, PredicatePoint, Sphere, Surface, SurfaceMeet, Vector,
};

/// A ball's seam, said in the vocabulary a crossing comes back in.
///
/// It is the hardest of the three shapes the kernel builds: both faces along it
/// are the same sphere, both its ends are poles, and only the meridian plane and
/// a half-space say which half of that circle it is.
#[test]
fn the_hardest_edge_the_kernel_builds_can_be_said_in_this_vocabulary() {
    let meridian = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("the y axis is a unit normal");
    let facing_x = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0))
        .expect("the x axis is a unit normal");
    let poles = [
        PredicatePoint::from([0.0, 0.0, 3.0]),
        PredicatePoint::from([0.0, 0.0, -3.0]),
    ];

    let seam = NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(meridian),
            picked_by: [Surface::Plane(facing_x)].into_iter().collect(),
        },
        poles,
    );

    assert_eq!(seam.ends(), poles);
    let EdgeRun::PartOfWhereItCrosses { and, .. } = seam.run() else {
        panic!("a seam is part of where its face crosses a meridian");
    };
    assert_eq!(and, &Surface::Plane(meridian));
}

/// Touching at a point and being the same surface are answers rather than
/// errors, so a caller out here has to be able to match on them.
#[test]
fn every_way_two_surfaces_can_meet_is_something_a_caller_can_match_on() {
    let pipe = Surface::Cylinder(
        Cylinder::new(
            rcad::Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0)).expect("a direction"),
            1.0,
        )
        .expect("a pipe of radius one"),
    );
    let ball = Surface::Sphere(Sphere::new(Point::origin(), 1.0).expect("a ball of radius one"));

    let met = [
        SurfaceMeet::Nothing,
        SurfaceMeet::TouchAtOnePoint {
            at: PredicatePoint::from([1.0, 0.0, 0.0]),
        },
        SurfaceMeet::Curves(vec![NamedRun::new(
            EdgeRun::PartOfWhereItCrosses {
                and: pipe,
                picked_by: [ball].into_iter().collect(),
            },
            [PredicatePoint::from([1.0, 0.0, 0.0]); 2],
        )]),
        SurfaceMeet::SameSurface,
    ];

    let named: Vec<&'static str> = met
        .iter()
        .map(|meet| match meet {
            SurfaceMeet::Nothing => "nothing",
            SurfaceMeet::TouchAtOnePoint { .. } => "one point",
            SurfaceMeet::Curves(runs) => {
                assert_eq!(runs.len(), 1);
                "curves"
            }
            SurfaceMeet::SameSurface => "the same surface",
            // `SurfaceMeet` grows as the pair table does, and guessing what a
            // new arm means is worse than stopping.
            other => panic!("unhandled crossing: {other:?}"),
        })
        .collect();

    assert_eq!(
        named,
        ["nothing", "one point", "curves", "the same surface"]
    );
}
