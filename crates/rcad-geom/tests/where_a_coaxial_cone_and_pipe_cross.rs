//! A cone and a pipe on one axis cross in two circles, and every other cone
//! against a pipe is still refused.
//!
//! The two circles sit `ρ/slope` along the axis from the apex either way, one in
//! each nappe, and there is **no shape question**: a cone is double-napped and a
//! pipe's radius is above zero, so `slope·|h| = ρ` has exactly two roots
//! whatever the two surfaces are. So this file has no table of shapes to walk.
//! What it has instead is the boundary: the pairs that share an axis are named,
//! the pairs that lean or stand off it are refused, and nothing else in the
//! cone's row moved.
//!
//! # Why the pairs are written on coordinate axes
//!
//! Sharing an axis is decided **exactly**, so a cone whose apex was written as
//! `origin + t·ŵ` on a direction that is not a coordinate one lands an ulp or
//! two off the stored line and is refused — correctly, because those two
//! surfaces really do not share an axis. That is a limit and not a gap
//! (`docs/decisions.md` §16.10, §50.11), and
//! `an_apex_written_along_a_leaning_axis_lands_off_it_and_is_refused` is the row
//! that pins it rather than leaving it as a sentence.

use rcad_geom::{
    Axis, Cone, Cylinder, EdgeRun, Error, HalfSpace, Picker, Point, Sphere, Surface, SurfaceMeet,
    Vector, intersect,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface};

/// The 45-degree cone about `+z` with its apex at the origin.
fn a_cone_about_z(slope: f64) -> Cone {
    Cone::new(Axis::z(), slope).expect("a slope above zero")
}

/// A pipe about `+z` through the origin.
fn a_pipe_about_z(radius: f64) -> Cylinder {
    Cylinder::new(Axis::z(), radius).expect("a radius above zero")
}

fn crossed(cone: &Cone, pipe: &Cylinder) -> Result<SurfaceMeet, Error> {
    intersect(
        &Surface::Cone(*cone),
        &Surface::Cylinder(*pipe),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

fn crossed_the_other_way(cone: &Cone, pipe: &Cylinder) -> Result<SurfaceMeet, Error> {
    intersect(
        &Surface::Cylinder(*pipe),
        &Surface::Cone(*cone),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

/// The half-space each run is picked out by, in the order they came back.
fn the_pickers(answer: &SurfaceMeet) -> Vec<HalfSpace> {
    let SurfaceMeet::Curves(runs) = answer else {
        panic!("a coaxial cone and pipe cross in two circles: {answer:?}");
    };
    runs.iter()
        .map(|run| match run.run() {
            EdgeRun::PartOfWhereItCrosses { picked_by, .. } => match picked_by.as_slice() {
                [Picker::InFrontOf(half_space)] => *half_space,
                other => panic!("each circle is picked by one flat half-space: {other:?}"),
            },
            other => panic!("each circle is a piece of the crossing: {other:?}"),
        })
        .collect()
}

/// The headline: a frustum and a bar on one axis, which is the pair
/// `AFrustumAgainstABar` fuses.
#[test]
fn a_cone_and_a_pipe_on_one_axis_cross_in_two_circles() {
    let cone = a_cone_about_z(0.5);
    let pipe = a_pipe_about_z(2.0);

    for answer in [crossed(&cone, &pipe), crossed_the_other_way(&cone, &pipe)] {
        let named = answer.expect("a cone and a pipe on one axis cross in two circles");
        let SurfaceMeet::Curves(runs) = &named else {
            panic!("two circles, and not {named:?}");
        };
        assert_eq!(runs.len(), 2, "one circle in each nappe");
        for run in runs {
            assert!(
                run.is_the_whole_curve(),
                "a circle has no ends until topology puts them on it: {run:?}"
            );
        }
    }
}

/// The two circles are `ρ/slope` up and down the axis, and the pickers hold one
/// each.
///
/// Written for a pipe of radius two and a cone of slope one half, so the
/// circles sit at `z = ±4` and each has radius two. The check is on the
/// **pickers** rather than on any point this file worked out: what `intersect`
/// hands back is a pair of half-spaces, and what they have to do is hold one
/// circle each.
#[test]
fn each_picker_holds_the_circle_in_its_own_nappe_and_not_the_other() {
    let cone = a_cone_about_z(0.5);
    let pipe = a_pipe_about_z(2.0);
    let named = crossed(&cone, &pipe).expect("a coaxial pair is named");
    let pickers = the_pickers(&named);
    assert_eq!(pickers.len(), 2);

    let on_the_upper_circle = PredicatePoint::from([2.0, 0.0, 4.0]);
    let on_the_lower_circle = PredicatePoint::from([0.0, -2.0, -4.0]);
    for (picker, held) in pickers
        .iter()
        .zip([on_the_upper_circle, on_the_lower_circle])
    {
        assert_eq!(
            picker.side_of_point(held),
            Decision::Pos,
            "{picker:?} holds its own circle"
        );
    }
    assert_eq!(
        pickers[0].side_of_point(on_the_lower_circle),
        Decision::Neg,
        "and none of the other one"
    );
    assert_eq!(pickers[1].side_of_point(on_the_upper_circle), Decision::Neg);
}

/// Both circles are on both surfaces, which is what says the standoff above is
/// the right one.
#[test]
fn the_two_circles_lie_on_the_cone_and_on_the_pipe() {
    let cone = a_cone_about_z(0.5);
    let pipe = a_pipe_about_z(2.0);

    for corner in [
        PredicatePoint::from([2.0, 0.0, 4.0]),
        PredicatePoint::from([0.0, 2.0, 4.0]),
        PredicatePoint::from([-2.0, 0.0, -4.0]),
        PredicatePoint::from([0.0, -2.0, -4.0]),
    ] {
        assert_eq!(
            cone.side_of(corner),
            Decision::Zero,
            "{corner:?} is on the cone"
        );
        assert_eq!(
            pipe.side_of(corner),
            Decision::Zero,
            "{corner:?} is on the pipe"
        );
    }
}

/// A pipe wide enough to swallow the apex is still exactly two circles, and so
/// is one thin enough to sit inside every ruling.
///
/// **The row that says there is no shape question.** For a ball and a pipe the
/// same sweep gives nothing, one circle and two; here the answer never moves.
#[test]
fn every_radius_and_every_slope_on_one_axis_gives_the_same_two_circles() {
    for slope in [1e-3, 0.25, 1.0, 7.0, 1e3] {
        for radius in [1e-3, 0.5, 2.0, 400.0, 1e3] {
            let cone = a_cone_about_z(slope);
            let pipe = a_pipe_about_z(radius);
            let named = crossed(&cone, &pipe).expect("a coaxial pair is always named");
            let SurfaceMeet::Curves(runs) = &named else {
                panic!("slope {slope} against radius {radius} came back {named:?}");
            };
            assert_eq!(runs.len(), 2, "slope {slope} against radius {radius}");
        }
    }
}

/// The pipe's axis written the other way up is the same line, so the pair is
/// still named.
#[test]
fn a_pipe_whose_axis_is_written_backwards_shares_the_axis_just_the_same() {
    let cone = a_cone_about_z(1.0);
    let pipe = Cylinder::new(Axis::z().reversed(), 3.0).expect("a radius above zero");

    let named = crossed(&cone, &pipe).expect("one line written two ways is one line");
    let SurfaceMeet::Curves(runs) = &named else {
        panic!("two circles, and not {named:?}");
    };
    assert_eq!(runs.len(), 2);
}

/// An apex off the pipe's axis is refused, and it is the second of the two sign
/// questions that refuses it.
#[test]
fn a_cone_beside_a_pipe_rather_than_down_it_is_refused() {
    let cone = Cone::new(
        Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0)).expect("a direction"),
        1.0,
    )
    .expect("a slope above zero");
    let pipe = a_pipe_about_z(2.0);

    for answer in [crossed(&cone, &pipe), crossed_the_other_way(&cone, &pipe)] {
        assert_eq!(
            answer,
            Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
            "the axes are parallel but they are not the same line"
        );
    }
}

/// A cone leaning against a pipe is refused, which is the row's standing gap.
#[test]
fn a_cone_leaning_against_a_pipe_is_still_refused() {
    let cone = a_cone_about_z(1.0);
    let pipe = Cylinder::new(
        Axis::new(Point::origin(), Vector::new(1.0, 0.0, 0.0)).expect("a direction"),
        2.0,
    )
    .expect("a radius above zero");

    for answer in [crossed(&cone, &pipe), crossed_the_other_way(&cone, &pipe)] {
        assert_eq!(
            answer,
            Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface)
        );
    }
}

/// A pipe laid along one of a cone's rulings holds a **straight line**, and it
/// is refused.
///
/// **The reason the coaxial pair is the whole of what was named.** `rcad-topo`'s
/// `assemble.rs` answers *this crossing holds no straight line* for every cone
/// against a pipe, and that answer is right only for as long as `intersect`
/// refuses the leaning ones. Naming any of them turns it into a wrong answer —
/// the same trap §50.2 item 4 spotted for two cones sharing an apex, in a second
/// family.
///
/// The pair is written on whole numbers so that the ruling really is on the
/// pipe and not nearly on it. For the cone of slope one about `+z` and the ruling
/// `(t, 0, t)`, against the pipe of radius two about `(1, 0, 1)` through
/// `(0, 2, 0)`: `(p − o) × d` is `(−2, 0, 2)` whatever `t` is, so
/// `‖(p − o) × d‖² = 8 = 4·‖d‖²` and every point of the ruling sits exactly two
/// from the axis.
#[test]
fn a_pipe_laid_along_one_of_a_cones_rulings_is_refused_rather_than_named() {
    let cone = a_cone_about_z(1.0);
    let pipe = Cylinder::new(
        Axis::new(Point::new(0.0, 2.0, 0.0), Vector::new(1.0, 0.0, 1.0)).expect("a direction"),
        2.0,
    )
    .expect("a radius above zero");

    for answer in [crossed(&cone, &pipe), crossed_the_other_way(&cone, &pipe)] {
        assert_eq!(
            answer,
            Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
            "this crossing holds a straight line and nothing downstream is ready for it"
        );
    }
}

/// Nothing else in the cone's row moved: a ball and a second cone are refused
/// whether or not they share the axis.
///
/// **The second cone is the one that matters.** `rcad-topo`'s `assemble.rs`
/// answers *this crossing holds no straight line* for the whole cone-and-cone
/// row on the strength of this refusal, and two cones that share an apex meet
/// along rulings, which are lines (`docs/decisions.md` §50.2 item 4).
#[test]
fn a_cone_against_a_ball_or_another_cone_is_refused_axis_or_no_axis() {
    let cone = a_cone_about_z(1.0);
    let on_the_axis = [
        Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, 4.0), 2.0).expect("a radius")),
        Surface::Cone(
            Cone::new(
                Axis::new(Point::new(0.0, 0.0, 9.0), Vector::new(0.0, 0.0, 1.0))
                    .expect("a direction"),
                2.0,
            )
            .expect("a slope"),
        ),
    ];
    let off_it = [
        Surface::Sphere(Sphere::new(Point::new(3.0, 1.0, 4.0), 2.0).expect("a radius")),
        Surface::Cone(
            Cone::new(
                Axis::new(Point::new(0.0, 0.0, 9.0), Vector::new(1.0, 0.0, 1.0))
                    .expect("a direction"),
                2.0,
            )
            .expect("a slope"),
        ),
    ];

    for partner in on_the_axis.into_iter().chain(off_it) {
        for pair in [
            [Surface::Cone(cone), partner],
            [partner, Surface::Cone(cone)],
        ] {
            assert_eq!(
                intersect(
                    &pair[0],
                    &pair[1],
                    &Policy::certified(),
                    &mut ProvenanceLog::new()
                ),
                Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
                "{pair:?}"
            );
        }
    }
}

/// An apex built as `origin + t·ŵ` on a direction that is not a coordinate one
/// usually lands off the stored line, and every one that does is refused.
///
/// **A limit and not a gap** (§16.10). The multiply-and-add rounds, the apex it
/// produces is genuinely not on the pipe's axis, and the crossing of the two
/// surfaces *as stored* genuinely is a quartic. There is no answer being refused
/// here, so there is nothing to fix — which is why this is a test and not a
/// paragraph. §50.11 measured 20 000 of 20 000 apexes drawn this way refused,
/// worst miss `2.6e-14` on coordinates of order 100.
///
/// **Five of these seven, not seven of seven, and the split is the point.**
/// `t = 1` and `t = 10` happen to scale this particular direction onto three
/// `f64`s that are still exactly proportional to it, and those two are **named**
/// — so the refusal tracks where the apex really landed rather than refusing the
/// whole family. A blanket refusal would pass this row just as well and would be
/// a worse kernel.
#[test]
fn an_apex_written_along_a_leaning_axis_lands_off_it_and_is_refused() {
    let direction = Vector::new(1.0, 2.0, 3.0)
        .normalized()
        .expect("a direction with a length");
    let axis = Axis::new(Point::origin(), direction).expect("a direction");
    let pipe = Cylinder::new(axis, 2.0).expect("a radius above zero");
    let named_anyway = [1.0, 10.0];

    for along in [1.0, 3.0, 7.0, 10.0, 100.0, 1000.0, 12345.0] {
        let apex = Point::origin() + direction * along;
        let cone = Cone::new(Axis::new(apex, direction).expect("a direction"), 0.5)
            .expect("a slope above zero");
        let answer = crossed(&cone, &pipe);
        if named_anyway.contains(&along) {
            let named = answer.expect("this one landed on the line");
            let SurfaceMeet::Curves(runs) = &named else {
                panic!("two circles, and not {named:?}");
            };
            assert_eq!(runs.len(), 2, "at {along} along the axis");
        } else {
            assert_eq!(
                answer,
                Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
                "at {along} along the axis the apex is off the stored line"
            );
        }
    }

    // The apex written as the axis origin itself, which is on the line whatever
    // the direction was.
    let cone = Cone::new(
        Axis::new(Point::origin(), direction).expect("a direction"),
        0.5,
    )
    .expect("a slope above zero");
    let named = crossed(&cone, &pipe).expect("an apex that is on the line is named");
    let SurfaceMeet::Curves(runs) = &named else {
        panic!("two circles, and not {named:?}");
    };
    assert_eq!(runs.len(), 2);
}

/// An apex so far from the origin that the rounding of `w·a` swallows a
/// circle's whole standoff is refused rather than picked wrongly.
///
/// **A gap and not a limit**: the two circles are there and they are tellable
/// apart, and what runs out is how finely an `f64` offset can be written beside
/// a large coordinate. The standoff is `r/slope`, which is one here; the apex is
/// out at `1e150` on a leaning direction, so `w·a` is about `1.6e150` and one
/// ulp of it is about `2e134`.
///
/// **It takes a leaning direction to reach at all.** On a coordinate axis
/// `w·a` is a single product by one and never rounds, so no apex on `z`, however
/// far out, refuses here — `1e300` was tried and is named.
#[test]
fn an_apex_too_far_out_for_its_own_standoff_is_refused_rather_than_guessed() {
    let direction = Vector::new(1.0, 1.0, 1.0)
        .normalized()
        .expect("a direction with a length");
    let far_out = Point::new(1e150, 1e150, 1e150);
    let axis = Axis::new(far_out, direction).expect("a direction");
    let cone = Cone::new(axis, 1.0).expect("a slope above zero");
    let pipe = Cylinder::new(axis, 1.0).expect("a radius above zero");

    assert_eq!(
        crossed(&cone, &pipe),
        Err(Error::CannotNameWhichPieceOfACrossingThisIs),
        "one rounding of the apex is bigger than the whole crossing"
    );

    let on_z =
        Axis::new(Point::new(0.0, 0.0, 1e300), Vector::new(0.0, 0.0, 1.0)).expect("a direction");
    let straight_cone = Cone::new(on_z, 1.0).expect("a slope above zero");
    let straight_pipe = Cylinder::new(on_z, 1.0).expect("a radius above zero");
    assert!(
        crossed(&straight_cone, &straight_pipe).is_ok(),
        "a dot product with a coordinate direction never rounds"
    );
}
