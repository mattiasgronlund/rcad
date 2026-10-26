//! A cone and a pipe on one axis cross in two circles, and off one axis in the
//! loops of a quartic, or are refused by name.
//!
//! The two circles sit `r/slope` along the axis from the apex either way, one in
//! each nappe, and there is **no shape question**: a cone is double-napped and a
//! pipe's radius is above zero, so `slope·|h| = r` has exactly two roots
//! whatever the two surfaces are. So this file has no table of shapes to walk.
//! What it has instead is the boundary: the pairs that share an axis are named
//! as two circles, a pipe beside the axis as a loop round each nappe, and a pipe
//! along one of the cone's own lines is refused. A ball is named on the axis and
//! off it, and a second cone only on it
//! (`a_cone_crossed_by_a_cone_or_ball_on_its_axis.rs` is theirs).
//!
//! # Why the pairs are written on coordinate axes
//!
//! Sharing an axis is decided **exactly**, so a cone whose apex was written as
//! `origin + t·ŵ` on a direction that is not a coordinate one lands an ulp or
//! two off the stored line, correctly, because those two surfaces really do not
//! share an axis as stored. Since row Q2788 such a pair is named as the pair off
//! one axis it is, two loops a hair off the two circles, where it was refused
//! as a gap (`docs/decisions.md` §16.10, §50.11), and
//! `an_apex_written_along_a_leaning_axis_lands_on_it_or_off_it_and_crosses_in_two_runs_either_way`
//! pins both answers.

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
        .map(|run| {
            let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
                panic!("each circle is a piece of the crossing: {run:?}");
            };
            let [Picker::InFrontOf(half_space)] = picked_by.as_slice() else {
                panic!("each circle is picked by one flat half-space: {picked_by:?}");
            };
            *half_space
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

/// The two circles are `r/slope` up and down the axis, and the pickers hold one
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

/// An apex off the pipe's axis, on a parallel line, crosses it in two loops,
/// one round each nappe, and not in two circles: the flat through the apex
/// square to the axis holds them apart, and picks each.
#[test]
fn a_cone_beside_a_pipe_rather_than_down_it_crosses_it_in_a_loop_round_each_nappe() {
    let cone = Cone::new(
        Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0)).expect("a direction"),
        1.0,
    )
    .expect("a slope above zero");
    let pipe = a_pipe_about_z(2.0);

    for answer in [crossed(&cone, &pipe), crossed_the_other_way(&cone, &pipe)] {
        assert_eq!(
            the_flats_that_pick(answer),
            [
                Some(Vector::new(0.0, 0.0, 1.0)),
                Some(Vector::new(0.0, 0.0, -1.0))
            ],
            "the axes are parallel but they are not the same line"
        );
    }
}

/// Which way the one flat through the apex that picks each run faces, for a
/// pair off one axis that crosses in two loops.
fn the_flats_that_pick(answer: Result<SurfaceMeet, Error>) -> Vec<Option<Vector>> {
    let Ok(SurfaceMeet::Curves(runs)) = &answer else {
        panic!("two loops, and not {answer:?}");
    };
    runs.iter()
        .map(|run| {
            let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
                panic!("each loop is a piece of the crossing, not {run:?}");
            };
            let [picker @ Picker::InFrontOfAFlatThrough(_)] = picked_by.as_slice() else {
                panic!("one flat picks each loop, not {picked_by:?}");
            };
            picker.facing_of_a_flat()
        })
        .collect()
}

/// A pipe along `x` through a cone's apex, square across its axis, is refused,
/// though it crosses the cone in two loops, one above the apex and one below.
///
/// A **gap** (§43.6). The pipe is shallower than the cone, so it is read along
/// its own straight lines, and named only where every one of them is proved to
/// meet the cone twice. Here they do not all: the line `(t, 2, 0)` misses the
/// cone, since `t² + 4 = 0` has no root, while `(t, 0, 2)` meets it at
/// `t = ±2`. So each loop runs across the flat surface through the apex square
/// to the pipe, `x = 0`, which picks a shallower pipe's loops, and what holds
/// these two apart is `z = 0`, which nothing asks for this pipe.
#[test]
fn a_pipe_across_a_cones_axis_through_its_apex_is_refused() {
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
/// **The one pipe off the axis that has to stay refused.** `rcad-topo`'s
/// `assemble.rs` answers *this crossing holds no straight line* for every cone
/// against a pipe, and that answer is right only for as long as `intersect`
/// refuses this one. Naming it turns that into a wrong answer — the same trap
/// §50.2 item 4 spotted for two cones sharing an apex, in a second family. The
/// only straight lines on a pipe run along its axis, so a pipe holding a ruling
/// points along it, and every other pipe's lines meet the cone at most twice.
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

/// A ball is named on a cone's axis and off it, as the pipe is, and a second
/// cone only where it shares the axis.
///
/// On the axis they cross in circles square to it, or not at all: the ball of
/// radius two four up sits inside the upper nappe, `4/√2` from its side, and
/// misses it; the cone of slope two with its apex nine up meets the one of
/// slope one where `|z| = 2|z − 9|`, at `z = 6` and `z = 18`. **No row of that
/// holds a straight line**, which is what `rcad-topo`'s `assemble.rs` answers
/// the whole cone-and-cone row with. Off the axis, the ball centred at
/// `(3, 1, 4)` reaches across the side of the upper nappe and crosses it in one
/// loop right round. Two cones meet along rulings only where they share an
/// apex and their axes lean apart (`docs/decisions.md` §50.2 item 4), and the
/// second cone off the axis here is that shape's refusal.
///
/// **It still is, since `how_two_cones_off_one_axis_cross` was taught to name
/// a crossing in two loops, or one loop and two branches (`intersect.rs`).**
/// Of its four ways, the first two want `L` above zero and the other two want
/// `U`, which only exists where `|c|` clears `s·S`. Taking the base cone as
/// the one crossed, `L` comes to `-4`; taking `cone_off_it` as the one
/// crossed, `L` comes to `-8`; and `|c|` never clears `s·S` either way round
/// — it sits under it the first way and exactly on it the second. That is the
/// `d·w` taking both signs gap `how_two_cones_off_one_axis_cross`'s own doc
/// names among what none of the four ways proves, not a shape nobody has
/// tried.
#[test]
fn a_ball_is_named_on_a_cones_axis_and_off_it_and_a_second_cone_only_on_it() {
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
    let [ball_off_it, cone_off_it] = [
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

    let pairs = |partner: Surface| {
        [
            [Surface::Cone(cone), partner],
            [partner, Surface::Cone(cone)],
        ]
    };
    let [ball, second_cone] = on_the_axis;
    for pair in pairs(ball) {
        assert_eq!(
            intersect(
                &pair[0],
                &pair[1],
                &Policy::certified(),
                &mut ProvenanceLog::new()
            ),
            Ok(SurfaceMeet::Nothing),
            "{pair:?}"
        );
    }
    for pair in pairs(second_cone) {
        let named = intersect(
            &pair[0],
            &pair[1],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("two cones on one axis are named");
        let SurfaceMeet::Curves(runs) = &named else {
            panic!("two circles, and not {named:?}");
        };
        assert_eq!(runs.len(), 2, "{pair:?}");
    }
    for pair in pairs(ball_off_it) {
        let named = intersect(
            &pair[0],
            &pair[1],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let Ok(SurfaceMeet::Curves(runs)) = &named else {
            panic!("one loop, and not {named:?}");
        };
        let runs: Vec<&EdgeRun> = runs.iter().map(|run| run.run()).collect();
        assert_eq!(runs, [&EdgeRun::RightRoundWhereItsFacesCross], "{pair:?}");
    }
    for pair in pairs(cone_off_it) {
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

/// An apex built as `origin + t·ŵ` on a direction that is not a coordinate one
/// usually lands off the stored line, and every one that does is named as the
/// pair off one axis it is.
///
/// The multiply-and-add rounds, the apex it produces is genuinely not on the
/// pipe's axis, and the coaxiality question is answered correctly there. The
/// pair then crosses in a loop round each nappe, a hair off the two circles,
/// and the flat through the apex square to the cone's axis holds them apart and
/// picks each. Until row Q2788 that quartic was refused, a gap reached by a
/// placement rather than by an axis someone wrote leaning on purpose: §50.11
/// measured 20 000 of 20 000 apexes drawn this way refused, worst miss
/// `2.6e-14` on coordinates of order 100, and `intersect.rs`'s
/// `a_frustum_in_a_bore_turned_off_the_lattice_stops_sharing_its_axis_and_crosses_in_two_loops`
/// is the ordinary form of it.
///
/// **Five of these seven, not seven of seven, land off the line.** `t = 1` and
/// `t = 10` happen to scale this particular direction onto three `f64`s that
/// are still exactly proportional to it, and those two are named as two circles
/// each picked by a half-space across the axis, so the answer tracks where the
/// apex really landed.
#[test]
fn an_apex_written_along_a_leaning_axis_lands_on_it_or_off_it_and_crosses_in_two_runs_either_way() {
    let direction = Vector::new(1.0, 2.0, 3.0)
        .normalized()
        .expect("a direction with a length");
    let axis = Axis::new(Point::origin(), direction).expect("a direction");
    let pipe = Cylinder::new(axis, 2.0).expect("a radius above zero");
    let on_the_line = [1.0, 10.0];

    for along in [1.0, 3.0, 7.0, 10.0, 100.0, 1000.0, 12345.0] {
        let apex = Point::origin() + direction * along;
        let cone = Cone::new(Axis::new(apex, direction).expect("a direction"), 0.5)
            .expect("a slope above zero");
        let answer = crossed(&cone, &pipe);
        if on_the_line.contains(&along) {
            let named = answer.expect("this one landed on the line");
            let SurfaceMeet::Curves(runs) = &named else {
                panic!("two circles, and not {named:?}");
            };
            assert_eq!(runs.len(), 2, "at {along} along the axis");
            for run in runs {
                let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
                    panic!("at {along}: each circle is a piece, not {run:?}");
                };
                assert!(
                    matches!(picked_by.as_slice(), [Picker::InFrontOf(_)]),
                    "at {along}: a half-space across the axis picks each circle, not {picked_by:?}"
                );
            }
        } else {
            assert_eq!(
                the_flats_that_pick(answer),
                [Some(direction), Some(direction * -1.0)],
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
