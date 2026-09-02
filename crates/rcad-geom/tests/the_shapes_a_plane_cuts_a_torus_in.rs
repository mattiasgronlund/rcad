//! Every shape a plane and a torus can cross in, named or refused by name.
//!
//! A plane cuts a quadric in a conic and there are seven of those. It cuts a
//! **torus** in a quartic, and a quartic is a bigger family: one oval, two
//! ovals, a figure of eight, a pair of circles round the axis, a pair of
//! meridian circles, a pair of Villarceau circles on either of the two bitangent
//! planes, one circle touched all the way round, or nothing.
//!
//! **Four of those are named here and every other is refused by name**, and the
//! point of this file is that no row may come back as another: two ovals named
//! as one closed curve is a shell that does not close, which §4.1 ranks below
//! both crashing and refusing.
//!
//! # The two signs that settle which family this is
//!
//! `‖n × w‖²` says whether the plane is square to the axis, and `n·w` with
//! `n·c − d` say whether it holds the axis. Both are polynomials in the numbers
//! as stored, so level 3 settles them and no row is ever reached by a rounding.
//!
//! Reaching each row exactly takes the same care the cone's table needed. A
//! [`Plane`] scales its normal to length one and that division rounds, so a
//! normal is only exactly square to the axis when it is written along a
//! coordinate direction — which is what every row below does.

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, EdgeRun, Error, NamedRun, Picker, Plane, Point, Sphere,
    Surface, SurfaceMeet, Torus, Vector, intersect, walk, where_a_closed_crossing_starts,
};
use rcad_predicates::{Decision, Policy, ProvenanceLog, SideOfSurface};

/// A ring of four and one about `+z`, centred on the origin.
///
/// Its outer equator is at radius five, its inner at three, and it reaches one
/// unit either side of `z = 0`.
fn a_ring() -> Torus {
    Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring")
}

fn a_plane(through: Point, facing: Vector) -> Plane {
    Plane::from_point_and_normal(through, facing).expect("a normal with a length")
}

fn crossed(flat: &Plane, ring: &Torus) -> Result<SurfaceMeet, Error> {
    intersect(
        &Surface::Plane(*flat),
        &Surface::Torus(*ring),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

/// The same pair the other way round, which has to give the same answer.
fn crossed_the_other_way(flat: &Plane, ring: &Torus) -> Result<SurfaceMeet, Error> {
    intersect(
        &Surface::Torus(*ring),
        &Surface::Plane(*flat),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

/// How far a point sits off a torus, in the metric the torus's own stored
/// numbers name.
fn how_far_off(ring: &Torus, at: Point) -> f64 {
    let along = ring.axis().direction();
    let out = at - ring.centre();
    let across = out.cross(along).length() / along.length() - ring.ring_radius();
    let axial = out.dot(along) / along.length();
    (across.hypot(axial) - ring.tube_radius()).abs()
}

/// The one picker on a run that is one piece of a crossing.
fn the_picker_on(run: &NamedRun) -> Picker {
    let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
        panic!("this run is not one piece of a crossing");
    };
    assert_eq!(picked_by.len(), 1, "one picker and no more");
    picked_by[0]
}

/// Every point `walk` draws along one run, with the ends left off.
fn walked(run: &NamedRun, flat: &Plane, ring: &Torus) -> Vec<Point> {
    let along = [&Surface::Plane(*flat), &Surface::Torus(*ring)];
    walk(
        run,
        along,
        Deflection::of(0.01).expect("a positive deflection"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a circle is walked")
    .points()
    .to_vec()
}

// ---------------------------------------------------------------------------
// Square to the axis: nothing, one circle, or two.
// ---------------------------------------------------------------------------

/// A plane square to the axis and further off than the tube reaches shares no
/// point with the torus at all.
#[test]
fn a_plane_clear_of_the_tube_crosses_a_ring_nowhere() {
    let ring = a_ring();
    for height in [1.5, -1.5, 40.0] {
        let flat = a_plane(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, 1.0));
        assert_eq!(
            crossed(&flat, &ring),
            Ok(SurfaceMeet::Nothing),
            "at {height}"
        );
        assert_eq!(
            crossed_the_other_way(&flat, &ring),
            Ok(SurfaceMeet::Nothing)
        );
    }
}

/// A plane resting on the top of the tube touches it along **one whole
/// circle**, of the ring's own radius.
///
/// Tangency along a closed curve, which [`EdgeRun::RightRoundWhereItsFacesCross`]
/// says exactly — a plane resting along a pipe is named `Straight` for the same
/// reason. It is the only row here that is one curve rather than two.
#[test]
fn a_plane_resting_on_the_tube_touches_a_ring_along_one_whole_circle() {
    let ring = a_ring();
    for height in [1.0, -1.0] {
        let flat = a_plane(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, 1.0));
        let SurfaceMeet::Curves(runs) = crossed(&flat, &ring).expect("a named crossing") else {
            panic!("a plane resting on a ring touches it along a circle");
        };
        assert_eq!(runs.len(), 1, "one circle and not two, at {height}");
        assert_eq!(runs[0].run(), &EdgeRun::RightRoundWhereItsFacesCross);

        for at in walked(&runs[0], &flat, &ring) {
            assert!(how_far_off(&ring, at) < 1e-13, "{at:?} is off the ring");
            assert!(
                ((at - ring.centre())
                    .cross(Vector::new(0.0, 0.0, 1.0))
                    .length()
                    - 4.0)
                    .abs()
                    < 1e-13,
                "the circle a plane rests on has the ring's own radius"
            );
        }
    }
}

/// A plane square to the axis and inside the tube cuts **two** circles, one
/// inside the other, and the pipe of the ring's own radius is what tells them
/// apart.
///
/// The radii are `R ± √(r² − t²)`, and both rows below are chosen so that root
/// is an exact `f64`: at `t = 0` it is the tube itself and at `t = 0.6` it is
/// `0.8`.
#[test]
fn a_plane_through_a_ring_cuts_two_circles_told_apart_by_a_pipe() {
    let ring = a_ring();
    for (height, outer, inner) in [(0.0, 5.0, 3.0), (0.6, 4.8, 3.2), (-0.6, 4.8, 3.2)] {
        let flat = a_plane(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, 1.0));
        let SurfaceMeet::Curves(runs) = crossed(&flat, &ring).expect("a named crossing") else {
            panic!("a plane through a ring cuts two circles");
        };
        assert_eq!(runs.len(), 2, "two circles at {height}");

        for (run, radius) in runs.iter().zip([outer, inner]) {
            let EdgeRun::PartOfWhereItCrosses { and, picked_by } = run.run() else {
                panic!("one of two circles is named as a piece of the crossing");
            };
            assert_eq!(and, &Surface::Torus(ring));
            assert_eq!(picked_by.len(), 1);

            for at in walked(run, &flat, &ring) {
                assert!(how_far_off(&ring, at) < 1e-13, "{at:?} is off the ring");
                assert!(
                    (at.coordinates()[0].hypot(at.coordinates()[1]) - radius).abs() < 1e-12,
                    "{at:?} is not on the circle of radius {radius}"
                );
                // **Each run's own picker holds its own circle and the other
                // run's rules it out**, which is the whole of what makes two
                // concentric circles nameable at all.
                let here = rcad_predicates::Point::from(at.coordinates());
                assert_eq!(picked_by[0].side_of_point(here), Decision::Pos);
                let other = if radius > 4.0 { &runs[1] } else { &runs[0] };
                let EdgeRun::PartOfWhereItCrosses {
                    picked_by: theirs, ..
                } = other.run()
                else {
                    panic!("both circles are pieces of the crossing");
                };
                assert_eq!(
                    theirs[0].side_of_point(here),
                    Decision::Neg,
                    "the other run's picker would take this point too"
                );
            }
        }
        // The two pickers are one pipe read two ways round: the ring's own
        // radius about the torus's own axis, which lies strictly between the
        // two circles because the root above is strictly positive.
        let pipe = Cylinder::new(Axis::z(), 4.0).expect("a radius");
        assert_eq!(
            runs.iter().map(the_picker_on).collect::<Vec<_>>(),
            vec![Picker::OutsideAPipe(pipe), Picker::InsideAPipe(pipe)]
        );
    }
}

// ---------------------------------------------------------------------------
// Through the axis: the two meridian circles.
// ---------------------------------------------------------------------------

/// A plane through the axis cuts the two circles of the tube's own radius,
/// centred where the plane meets the centre circle, and a flat half-space tells
/// them apart.
#[test]
fn a_plane_through_the_axis_cuts_the_two_meridian_circles() {
    let ring = a_ring();
    for facing in [Vector::new(0.0, 1.0, 0.0), Vector::new(1.0, 0.0, 0.0)] {
        let flat = a_plane(Point::origin(), facing);
        let SurfaceMeet::Curves(runs) = crossed(&flat, &ring).expect("a named crossing") else {
            panic!("a plane through the axis cuts two circles");
        };
        assert_eq!(runs.len(), 2);
        assert_eq!(crossed_the_other_way(&flat, &ring), crossed(&flat, &ring));

        for run in &runs {
            let EdgeRun::PartOfWhereItCrosses { and, picked_by } = run.run() else {
                panic!("one of two circles is named as a piece of the crossing");
            };
            assert_eq!(and, &Surface::Torus(ring));
            assert!(matches!(picked_by[0], Picker::InFrontOf(_)));

            let points = walked(run, &flat, &ring);
            assert!(!points.is_empty(), "a circle is walked into pieces");
            for at in points {
                assert!(how_far_off(&ring, at) < 1e-13, "{at:?} is off the ring");
                // Every point of a meridian circle is exactly the tube's radius
                // from the centre circle, and the centre circle here meets the
                // plane at (±4, 0, 0) or (0, ±4, 0).
                let from_the_axis =
                    at.coordinates()[0].hypot(at.coordinates()[1]) - ring.ring_radius();
                assert!(
                    (from_the_axis.hypot(at.coordinates()[2]) - 1.0).abs() < 1e-13,
                    "{at:?} is not one unit from the centre circle"
                );
            }
        }
    }
}

/// The one place a vertex can go on each of the two meridian circles is a point
/// on both surfaces, and the two are not the same place.
#[test]
fn each_meridian_circle_starts_somewhere_on_both_surfaces() {
    let ring = a_ring();
    let flat = a_plane(Point::origin(), Vector::new(0.0, 1.0, 0.0));
    let SurfaceMeet::Curves(runs) = crossed(&flat, &ring).expect("a named crossing") else {
        panic!("two circles");
    };

    let starts: Vec<Point> = runs
        .iter()
        .map(|run| {
            where_a_closed_crossing_starts(
                run,
                [&Surface::Plane(flat), &Surface::Torus(ring)],
                &mut ProvenanceLog::new(),
            )
            .expect("a closed crossing has a place to start")
        })
        .collect();
    for at in &starts {
        assert!(how_far_off(&ring, *at) < 1e-13, "{at:?} is off the ring");
        assert!(flat.signed_distance_to(*at).abs() < 1e-13, "off the plane");
    }
    assert_ne!(starts[0], starts[1], "the two circles start in two places");
}

// ---------------------------------------------------------------------------
// Everything else is a quartic, and is refused by name.
// ---------------------------------------------------------------------------

/// A plane parallel to the axis but off it cuts a Cassini oval, or two of them,
/// or a figure of eight — a genuine quartic, and no row of the vocabulary here
/// says any of those.
#[test]
fn a_plane_beside_the_axis_cuts_a_ring_in_a_quartic() {
    let ring = a_ring();
    // Well outside the ring, right through the middle of the tube, exactly on
    // the centre circle, inside the hole, and past the hole: five quite
    // different quartics and one refusal for all of them.
    for offset in [4.0, 4.5, 3.0, 2.0, 0.5] {
        let flat = a_plane(Point::new(0.0, offset, 0.0), Vector::new(0.0, 1.0, 0.0));
        assert_eq!(
            crossed(&flat, &ring),
            Err(Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic),
            "at {offset}"
        );
        assert_eq!(
            crossed_the_other_way(&flat, &ring),
            Err(Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic)
        );
    }
}

/// A leaning plane cuts a quartic too, and the two **bitangent** planes — whose
/// crossing is the pair of Villarceau circles — are leaning planes, so they are
/// refused with everything else.
///
/// A ring of four and one is bitangent to the planes through its centre that
/// stand `r = 1` off the axis direction, which is a lean of `asin(1/4)`. The
/// row is included as the shape it is rather than reached exactly, because what
/// it demonstrates is that the refusal does not depend on which quartic it is.
#[test]
fn a_leaning_plane_cuts_a_ring_in_a_quartic_including_the_villarceau_pair() {
    let ring = a_ring();
    let leans = f64::from(4.0f32).recip().asin();
    for facing in [
        Vector::new(0.0, 1.0, 1.0),
        Vector::new(1.0, 2.0, 3.0),
        Vector::new(0.0, leans.cos(), leans.sin()),
    ] {
        let flat = a_plane(Point::origin(), facing);
        assert_eq!(
            crossed(&flat, &ring),
            Err(Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic),
            "facing {facing:?}"
        );
    }
}

/// Two tori holding the same numbers are the same surface, and a torus against
/// anything else curved runs to degree eight and is refused by name.
#[test]
fn a_ring_against_anything_curved_is_refused_and_against_itself_is_one_surface() {
    let ring = a_ring();
    let settle = |one: &Surface, other: &Surface| {
        intersect(one, other, &Policy::certified(), &mut ProvenanceLog::new())
    };
    assert_eq!(
        settle(&Surface::Torus(ring), &Surface::Torus(ring)),
        Ok(SurfaceMeet::SameSurface)
    );

    let others = [
        Surface::Sphere(Sphere::new(Point::origin(), 4.0).expect("a radius")),
        Surface::Cylinder(Cylinder::new(Axis::z(), 4.0).expect("a radius")),
        Surface::Cone(Cone::new(Axis::z(), 1.0).expect("a slope")),
        Surface::Torus(Torus::new(Axis::z(), 4.0, 2.0).expect("a thicker tube")),
        Surface::Torus(Torus::new(Axis::x(), 4.0, 1.0).expect("a ring the other way")),
    ];
    for other in others {
        assert_eq!(
            settle(&Surface::Torus(ring), &other),
            Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface),
            "against {other:?}"
        );
        assert_eq!(
            settle(&other, &Surface::Torus(ring)),
            Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface)
        );
    }
}

/// A straight line meets a torus four times and every other surface here twice,
/// so where three surfaces meet is refused by name rather than answered with a
/// quadratic's two roots.
#[test]
fn where_two_planes_and_a_ring_meet_is_refused_because_a_line_meets_it_four_times() {
    let ring = a_ring();
    let one = a_plane(Point::origin(), Vector::new(0.0, 1.0, 0.0));
    let other = a_plane(Point::origin(), Vector::new(0.0, 0.0, 1.0));

    // The `x` axis really does meet this ring four times, at ±3 and ±5, which
    // is the fact the refusal is about.
    for x in [-5.0, -3.0, 3.0, 5.0] {
        assert_eq!(
            ring.side_of(rcad_predicates::Point::from([x, 0.0, 0.0])),
            Decision::Zero,
            "at x = {x}"
        );
    }

    for triple in [
        [
            &Surface::Plane(one),
            &Surface::Plane(other),
            &Surface::Torus(ring),
        ],
        [
            &Surface::Torus(ring),
            &Surface::Plane(one),
            &Surface::Plane(other),
        ],
    ] {
        assert_eq!(
            rcad_geom::meet_of_three_surfaces(
                triple,
                &Policy::certified(),
                &mut ProvenanceLog::new()
            ),
            Err(Error::WhereALineMeetsATorusIsAQuartic)
        );
    }
}
