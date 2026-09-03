//! Every shape a plane and a cone can cross in, named or refused by name.
//!
//! A plane cuts a cone in a circle, an ellipse, a parabola, a hyperbola, one
//! point, one line or two — seven answers, and `intersect` has to give a
//! different one for each. Five of them it names and two it refuses, and this
//! file is the table: **not one of the seven may come back as another**, because
//! a hyperbola named as an ellipse is a closed curve where there is none and
//! that is the wrong answer §4.1 ranks below both crashing and refusing.
//!
//! # The two signs that settle all seven, and how each row is reached exactly
//!
//! Where the apex stands, `n·a − d`, and how the plane leans:
//! `E = (n·w)² − slope²‖n × w‖²`. For a cone about `+z` with apex at the origin
//! and a stored unit normal `(a, b, c)`, `E` is `c² − slope²(a² + b²)`.
//!
//! **Every row below is reached with `E` and `n·a − d` exactly what the row
//! wants, and reaching `E = 0` exactly takes care.** A [`Plane`] scales its
//! normal to length one and that division rounds, so a normal written at 45° to
//! the axis does *not* come back with `c² = a² + b²` — the parabola row would
//! quietly become a hyperbola or an ellipse. What does work is a normal whose
//! two live components are **equal**: `(0, 1, 1)` normalizes to `(0, s, s)` for
//! one `s`, whatever `s` rounds to, so against a cone of slope one `E` is
//! `s² − 1·s²`, which the exact level sums to zero and not to nearly zero.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Error, NamedRun, Plane, Point, Sphere, Surface, SurfaceMeet,
    Vector, intersect, walk, where_a_closed_crossing_starts,
};
use rcad_predicates::{Decision, Policy, ProvenanceLog, SideOfSurface};

/// The 45-degree cone about `+z` with its apex at the origin.
///
/// Slope one, so `slope²` is exactly one and every `E` below is a difference of
/// two stored squares with nothing else in it.
fn a_right_angled_cone() -> Cone {
    Cone::new(Axis::z(), 1.0).expect("a slope of one is 45 degrees")
}

fn a_plane(through: Point, facing: Vector) -> Plane {
    Plane::from_point_and_normal(through, facing).expect("a normal with a length")
}

fn crossed(flat: &Plane, cone: &Cone) -> Result<SurfaceMeet, Error> {
    intersect(
        &Surface::Plane(*flat),
        &Surface::Cone(*cone),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

/// The same pair the other way round, which has to give the same answer.
fn crossed_the_other_way(flat: &Plane, cone: &Cone) -> Result<SurfaceMeet, Error> {
    intersect(
        &Surface::Cone(*cone),
        &Surface::Plane(*flat),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

/// Row 1: a plane square to the axis and clear of the apex cuts one circle.
#[test]
fn a_plane_square_to_the_axis_cuts_a_cone_in_one_closed_curve() {
    let cone = a_right_angled_cone();
    let cap = a_plane(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 0.0, 1.0));

    for answer in [crossed(&cap, &cone), crossed_the_other_way(&cap, &cone)] {
        let SurfaceMeet::Curves(runs) = answer.expect("a cap plane cuts a cone in a circle") else {
            panic!("a plane square to the axis cuts one closed curve");
        };
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].run(), &EdgeRun::RightRoundWhereItsFacesCross);
        assert!(runs[0].is_the_whole_curve());
    }
}

/// Row 2: a plane leaning less steeply than the cone's own side cuts an
/// ellipse, and it is named the same way the circle is — neither is drawn here,
/// so neither needs telling apart (§24.5).
#[test]
fn a_plane_leaning_across_a_cone_cuts_one_closed_curve_too() {
    let cone = a_right_angled_cone();
    // `(0, −1, 2)` normalizes to a stored `(0, −b, c)` with `c` twice `b` up to
    // the one rounding, so `E = c² − b²` is plainly positive.
    let leaning = a_plane(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, -1.0, 2.0));

    let SurfaceMeet::Curves(runs) = crossed(&leaning, &cone).expect("an ellipse is named") else {
        panic!("a plane leaning less than the cone's side cuts one closed curve");
    };
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].run(), &EdgeRun::RightRoundWhereItsFacesCross);
}

/// Row 3: a plane parallel to one ruling and clear of the apex cuts a parabola,
/// which this vocabulary has no arm for.
///
/// **`E` is exactly zero here and that is the whole point of the normal chosen.**
/// `(0, 1, 1)` stores as `(0, s, s)`, so `E = s² − 1·(0 + s²)` sums to zero on
/// the exact level rather than to a number a hair either side of it — which is
/// what would turn this row into row 2 or row 4 and make the test prove nothing.
#[test]
fn a_plane_parallel_to_a_ruling_cuts_a_parabola_and_is_refused_by_name() {
    let cone = a_right_angled_cone();
    let along_a_ruling = a_plane(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 1.0, 1.0));

    for answer in [
        crossed(&along_a_ruling, &cone),
        crossed_the_other_way(&along_a_ruling, &cone),
    ] {
        assert_eq!(answer, Err(Error::ACrossingThatIsAParabola));
    }
}

/// Row 4: a plane steeper than the cone's own side and clear of the apex cuts a
/// hyperbola — two endless curves, one in each nappe.
#[test]
fn a_plane_steeper_than_a_cones_side_cuts_a_hyperbola_and_is_refused_by_name() {
    let cone = a_right_angled_cone();
    // Down the axis: the normal is `(1, 0, 0)`, so `c` is exactly zero and
    // `E = −slope²` without a rounding anywhere.
    let down_the_axis = a_plane(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0));

    assert_eq!(
        crossed(&down_the_axis, &cone),
        Err(Error::ACrossingThatIsAHyperbola)
    );

    // And the two branches really are there, one in each nappe: the plane
    // `x = 1` meets the cone at `(1, 0, ±1)` and at nothing with `z = 0`.
    for height in [1.0, -1.0] {
        assert_eq!(
            cone.side_of(rcad_predicates::Point::from([1.0, 0.0, height])),
            Decision::Zero,
            "the hyperbola passes through (1, 0, {height})"
        );
    }
}

/// Row 5: a plane through the apex and leaning less than the cone's side meets
/// it at the apex and nowhere else, and the point is **exact**.
#[test]
fn a_plane_through_the_apex_and_across_the_cone_touches_it_at_one_point() {
    let cone = Cone::new(
        Axis::new(Point::new(2.0, -3.0, 5.0), Vector::new(0.0, 0.0, 1.0))
            .expect("a direction with length"),
        1.0,
    )
    .expect("a positive slope");
    let through_the_apex = a_plane(Point::new(2.0, -3.0, 5.0), Vector::new(0.0, 0.0, 1.0));

    for answer in [
        crossed(&through_the_apex, &cone),
        crossed_the_other_way(&through_the_apex, &cone),
    ] {
        let SurfaceMeet::TouchAtOnePoint { at } = answer.expect("a plane through the apex") else {
            panic!("a plane across the cone through its apex meets it there and nowhere else");
        };
        assert_eq!(
            at.exact_coordinates(),
            Some([2.0, -3.0, 5.0]),
            "the touching point is the apex, which is a stored triple and not a box"
        );
    }
}

/// Row 6: a plane through the apex and parallel to one ruling touches the cone
/// along the whole of that ruling, which is one endless straight run.
///
/// The same naming a plane resting along a pipe's own side gets
/// (`intersect.rs`'s `where_a_plane_and_a_pipe_cross`, the `Decision::Zero` arm):
/// one [`EdgeRun::Straight`] with no ends, because the crossing is the whole line
/// and topology is what puts vertices on it.
#[test]
fn a_plane_touching_a_cone_along_a_ruling_is_one_straight_run() {
    let cone = a_right_angled_cone();
    let tangent = a_plane(Point::origin(), Vector::new(0.0, 1.0, 1.0));

    for answer in [
        crossed(&tangent, &cone),
        crossed_the_other_way(&tangent, &cone),
    ] {
        let SurfaceMeet::Curves(runs) = answer.expect("a ruling is one straight line") else {
            panic!("a plane along a ruling touches the cone along the whole of it");
        };
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].run(), &EdgeRun::Straight);
        assert!(runs[0].is_the_whole_curve());
    }

    // The ruling really is on both: `(0, −v, v)` is on the cone for every `v`,
    // and the plane through the origin facing `(0, 1, 1)` holds all of it.
    for along in [1.0, 3.0, -2.0] {
        assert_eq!(
            cone.side_of(rcad_predicates::Point::from([0.0, -along, along])),
            Decision::Zero
        );
        assert_eq!(
            tangent.side_of_point(rcad_predicates::Point::from([0.0, -along, along])),
            Decision::Zero
        );
    }
}

/// Row 7: a plane through the apex and steeper than the cone's side cuts two
/// rulings that meet at the apex — **two** straight runs, told apart by a pair
/// of opposite half-spaces each.
///
/// The refusal this replaces said no half-space picks one of the two lines,
/// *"because both pass through the apex and a half-space that holds one holds
/// half of the other"*. Both halves of that are true and the conclusion is not:
/// a piece is picked by **every** half-space on it, so the two opposite ones
/// about one flat surface pick what lies *in* that surface. The flat surface
/// through the apex holding `(0, −v, v)` meets `(0, v, v)` at the apex alone.
///
/// The two lines are `(0, ±v, v)`, both exactly on the cone and exactly in the
/// plane `x = 0`, and this walks a long way out along each: a line is endless,
/// and a picker that leans into its direction by one rounding stops holding it
/// somewhere far along.
#[test]
fn a_plane_through_the_apex_and_down_it_cuts_two_straight_lines_told_apart_by_pickers() {
    let cone = a_right_angled_cone();
    let down_the_axis = a_plane(Point::origin(), Vector::new(1.0, 0.0, 0.0));

    let far_along_each_line = |sideways: f64| {
        [1.0, 3.0, -2.0, 1e15].map(|v| rcad_predicates::Point::from([0.0, sideways * v, v]))
    };
    for sideways in [1.0, -1.0] {
        for point in far_along_each_line(sideways) {
            assert_eq!(cone.side_of(point), Decision::Zero, "{point:?}");
            assert_eq!(
                down_the_axis.side_of_point(point),
                Decision::Zero,
                "{point:?}"
            );
        }
    }

    for answer in [
        crossed(&down_the_axis, &cone),
        crossed_the_other_way(&down_the_axis, &cone),
    ] {
        let SurfaceMeet::Curves(runs) = answer.expect("two rulings are two straight lines") else {
            panic!("a plane down a cone's axis cuts two lines through the apex");
        };
        assert_eq!(runs.len(), 2);

        let picked: Vec<_> = runs
            .iter()
            .map(|run| {
                assert!(run.is_the_whole_curve(), "a ruling has no end to be cut at");
                let EdgeRun::PartOfWhereItCrosses { and, picked_by } = run.run() else {
                    panic!("one of two lines is a piece of where the two surfaces cross");
                };
                assert_eq!(and, &Surface::Cone(cone));
                assert_eq!(
                    picked_by.len(),
                    2,
                    "one flat surface, read from both sides, is what holds a line that runs \
                     through the apex"
                );
                picked_by.clone()
            })
            .collect();

        // Each run holds the whole of one line and, of the other, only the apex.
        // That is what makes them two pieces and not one said twice.
        let holds = |pickers: &[rcad_geom::Picker], point| {
            pickers
                .iter()
                .all(|picker| picker.side_of_point(point) != Decision::Neg)
        };
        let mut lines_held = Vec::new();
        for pickers in &picked {
            let held: Vec<f64> = [1.0, -1.0]
                .into_iter()
                .filter(|sideways| {
                    far_along_each_line(*sideways)
                        .into_iter()
                        .all(|point| holds(pickers, point))
                })
                .collect();
            assert_eq!(held.len(), 1, "one run holds one whole line, got {held:?}");
            lines_held.push(held[0]);
            assert!(
                holds(pickers, rcad_predicates::Point::from([0.0, 0.0, 0.0])),
                "both lines run through the apex, so every picker holds it"
            );
        }
        assert_eq!(
            lines_held.len(),
            2,
            "the two runs are the two different lines, not one line twice"
        );
        assert!(lines_held[0] != lines_held[1], "got {lines_held:?}");
    }
}

/// Rows 3 and 4 again, reached this time through [`walk`] and
/// [`where_a_closed_crossing_starts`] — which used to panic on both.
///
/// `intersect` refuses these two shapes before a run exists, and
/// `where_a_plane_cuts_a_cone` asserted that that was enough. **It is not.**
/// [`NamedRun`] and [`EdgeRun`] are public, [`EdgeRun`] is `Deserialize`, and
/// both walk entry points are public and take whatever pair of surfaces the
/// caller hands them — so a caller, or a file read back in, states a closed run
/// between a cone and a plane that cuts it in a parabola and the assert fires.
/// Nothing here forges an internal state: every value goes in through a
/// published constructor.
///
/// The two are told apart the same way `intersect` tells them apart, so the
/// namer and the walker never disagree about what a pair is called.
#[test]
fn a_run_a_caller_names_between_a_plane_and_a_cone_that_bends_endlessly_is_refused_by_name() {
    let cone = Surface::Cone(a_right_angled_cone());
    for (through, facing, refusal) in [
        (
            Point::new(0.0, 0.0, 4.0),
            Vector::new(0.0, 1.0, 1.0),
            Error::ACrossingThatIsAParabola,
        ),
        (
            Point::new(1.0, 0.0, 0.0),
            Vector::new(1.0, 0.0, 0.0),
            Error::ACrossingThatIsAHyperbola,
        ),
    ] {
        let flat = Surface::Plane(a_plane(through, facing));
        assert_eq!(
            intersect(
                &flat,
                &cone,
                &Policy::certified(),
                &mut ProvenanceLog::new()
            ),
            Err(refusal.clone()),
            "the claim the assert rested on: the namer refuses this pair"
        );

        let stated = NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed curve is never a piece with nothing picking it");
        assert_eq!(
            walk(
                &stated,
                [&flat, &cone],
                Deflection::of(1.0).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .err(),
            Some(refusal.clone()),
            "and here is a run between them anyway, walked: {flat:?}"
        );
        assert_eq!(
            where_a_closed_crossing_starts(&stated, [&flat, &cone], &mut ProvenanceLog::new())
                .err(),
            Some(refusal),
            "the other public way in reaches the same place: {flat:?}"
        );
    }
}

/// Rows 6 and 7 through the same two public doors, where the answer is now
/// *straight* rather than a refusal.
///
/// A run the two arms mint is walked into no points at all, which is what a
/// straight run is: its two ends are the whole of it, and the ends are
/// topology's to put on. And a run a caller states as closed on the same pair
/// gets the answer every other pair that crosses in straight lines gets — the
/// empty walk, and `TheCrossingOfTheseTwoSurfacesIsNotClosed` from the door that
/// has to place a vertex.
#[test]
fn a_run_along_a_cones_ruling_is_walked_as_a_straight_line_from_both_public_doors() {
    let cone = Surface::Cone(a_right_angled_cone());
    let apex = Point::origin();
    for facing in [Vector::new(0.0, 1.0, 1.0), Vector::new(1.0, 0.0, 0.0)] {
        let flat = Surface::Plane(a_plane(apex, facing));
        let SurfaceMeet::Curves(runs) = intersect(
            &flat,
            &cone,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a plane through the apex cuts straight lines") else {
            panic!("a plane through the apex cuts the cone in one line or two: {flat:?}");
        };

        for stated in
            runs.into_iter().chain([
                NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
                    .expect("a closed curve is never a piece with nothing picking it"),
            ])
        {
            assert_eq!(
                walk(
                    &stated,
                    [&flat, &cone],
                    Deflection::of(1.0).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a straight run is walked without complaint")
                .points(),
                &[],
                "a straight run needs nothing between its ends: {stated:?}"
            );
            assert_eq!(
                where_a_closed_crossing_starts(&stated, [&flat, &cone], &mut ProvenanceLog::new())
                    .err(),
                Some(Error::TheCrossingOfTheseTwoSurfacesIsNotClosed),
                "there is no closed curve here to put a vertex on: {stated:?}"
            );
        }
    }
}

/// A cone against anything else curved is refused by name, in both orders.
#[test]
fn a_cone_against_a_ball_a_pipe_or_another_cone_is_refused_by_name() {
    let cone = Surface::Cone(a_right_angled_cone());
    let ball = Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, 4.0), 2.0).expect("a radius"));
    let pipe =
        Surface::Cylinder(rcad_geom::Cylinder::new(Axis::z(), 2.0).expect("a pipe of radius two"));
    let other_cone = Surface::Cone(
        Cone::new(
            Axis::new(Point::new(0.0, 0.0, 10.0), Vector::new(0.0, 0.0, -1.0))
                .expect("a direction with length"),
            1.0,
        )
        .expect("a positive slope"),
    );

    for partner in [ball, pipe, other_cone] {
        for pair in [[&cone, &partner], [&partner, &cone]] {
            assert_eq!(
                intersect(
                    pair[0],
                    pair[1],
                    &Policy::certified(),
                    &mut ProvenanceLog::new()
                ),
                Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
                "{pair:?}"
            );
        }
    }
}

/// Two cones holding the same numbers are one surface, which is the answer a
/// fuse needs when two solids press coincident cone faces together.
///
/// **The same cone written with its axis reversed is not caught**, and that is
/// reported rather than fixed: reversing the axis direction leaves the double
/// cone exactly where it was, so two spellings name one surface. A
/// [`Cylinder`](rcad_geom::Cylinder) has had the same two spellings since M2 and
/// the kernel has always lived with it — a face's surface is compared on the
/// numbers it stores (§32.5), and neither type has a canonical form. Refusing
/// the reversed pair is safe; calling it `SameSurface` would need a canonical
/// spelling that `transformed` could not round away.
#[test]
fn two_cones_holding_the_same_numbers_are_one_surface() {
    let cone = Surface::Cone(a_right_angled_cone());
    assert_eq!(
        intersect(
            &cone,
            &cone,
            &Policy::certified(),
            &mut ProvenanceLog::new()
        ),
        Ok(SurfaceMeet::SameSurface)
    );

    let reversed = Surface::Cone(
        Cone::new(Axis::z().reversed(), 1.0).expect("the same double cone written backwards"),
    );
    assert_eq!(
        intersect(
            &cone,
            &reversed,
            &Policy::certified(),
            &mut ProvenanceLog::new()
        ),
        Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
        "the same surface in two spellings is refused rather than answered wrongly"
    );
}

/// The circle a cap plane cuts out of a cone, walked, and measured against the
/// closed form.
///
/// `radius = slope · height`, and the centre is where the axis meets the plane.
/// Every point of the walk is on both surfaces to within the published bound,
/// which is what says the ellipse this file's `Conic` builds is the right one
/// and not merely a plausible one.
#[test]
fn the_circle_a_cap_cuts_out_of_a_cone_is_walked_into_points_on_both() {
    let cone = Surface::Cone(Cone::new(Axis::z(), 0.5).expect("a positive slope"));
    let cap = Surface::Plane(a_plane(
        Point::new(0.0, 0.0, 8.0),
        Vector::new(0.0, 0.0, 1.0),
    ));

    let walked = walk(
        &NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed curve is never a piece with nothing picking it"),
        [&cone, &cap],
        Deflection::of(8.0 / 1000.0).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a cone cut square across is walked without complaint");

    assert!(
        walked.points().len() >= 8,
        "a whole circle at a thousandth of its size takes more than eight pieces, got {}",
        walked.points().len()
    );
    let room = walked.deviation().magnitude();
    for point in walked.points() {
        assert!(
            (point.z() - 8.0).abs() <= room,
            "a point of the circle sits in the cap's own plane: {point:?}"
        );
        let across = (point.x() * point.x() + point.y() * point.y()).sqrt();
        assert!(
            (across - 4.0).abs() <= room,
            "the circle's radius is slope times height, which is 4 here, got {across}"
        );
    }
}

/// The ellipse a leaning plane cuts out of a cone, measured against the closed
/// form both ways across.
///
/// The cone has slope one and the plane leans at `atan(1/2)` to the axis, so the
/// two vertices sit where `z = ±y` meets `z = 4 + y/2`: at `y = 8, z = 8` and at
/// `y = −8/3, z = 8/3`. The major semi-axis is half the distance between them
/// and the centre is their midpoint, both of which this checks — and the minor
/// one comes out of the same picture as `√(h²t²/E)` with `E = 3/5`.
#[test]
fn the_ellipse_a_leaning_plane_cuts_out_of_a_cone_matches_the_closed_form() {
    let cone = Surface::Cone(Cone::new(Axis::z(), 1.0).expect("a positive slope"));
    // `2z − y = 8`, so the normal is `(0, −1, 2)` and the plane passes through
    // `(0, 0, 4)`.
    let leaning = Surface::Plane(a_plane(
        Point::new(0.0, 0.0, 4.0),
        Vector::new(0.0, -1.0, 2.0),
    ));

    let walked = walk(
        &NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
            .expect("a closed curve is never a piece with nothing picking it"),
        [&cone, &leaning],
        Deflection::of(8.0 / 4000.0).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a cone cut across at an angle is walked without complaint");

    let room = walked.deviation().magnitude().max(1e-6);
    let here = Point::new(0.0, 8.0, 8.0);
    let there = Point::new(0.0, -8.0 / 3.0, 8.0 / 3.0);
    let middle = Point::new(
        0.0,
        (here.y() + there.y()) / 2.0,
        (here.z() + there.z()) / 2.0,
    );
    let major = here.distance_to(there) / 2.0;
    // `b = |h|t/√E`. The plane is `2z − y = 8`, so its stored normal is
    // `(0, −1, 2)/√5` and its offset is `8/√5`; the apex is at the origin, so
    // `|h| = 8/√5`. And `E = (n·w)² − t²‖n × w‖² = (2² − 1²)/5 = 3/5`, which
    // makes `b = 8/√3`. The same numbers give `a = |h|t/E = 8√5/3`, which is
    // half the distance between the two vertices above — two routes to the same
    // semi-axis, and the test uses the vertices for one and the formula for the
    // other so that neither is checked against itself.
    let minor = (8.0 / 5.0f64.sqrt()) / (3.0f64 / 5.0).sqrt();

    let from_the_middle = walked
        .points()
        .iter()
        .map(|point| (*point - middle).length());
    let narrowest = from_the_middle.clone().fold(f64::MAX, f64::min);
    let widest = from_the_middle.fold(f64::MIN, f64::max);
    assert!(
        (widest - major).abs() < 0.01 + room,
        "the long way across is {major}, got {widest}"
    );
    assert!(
        (narrowest - minor).abs() < 0.01 + room,
        "the short way across is {minor}, got {narrowest}"
    );

    // And every point of it is on both surfaces, which is what says the ellipse
    // is the right one rather than merely one of the right size: an ellipse of
    // the right semi-axes about the wrong centre, or turned the wrong way in the
    // plane, would pass the two checks above and fail this.
    for point in walked.points() {
        assert!(
            (2.0 * point.z() - point.y() - 8.0).abs() / 5.0f64.sqrt() <= room,
            "a point of the crossing lies in the plane `2z − y = 8`: {point:?}"
        );
        let across = (point.x() * point.x() + point.y() * point.y()).sqrt();
        assert!(
            (across - point.z().abs()).abs() <= room * 2.0f64.sqrt(),
            "a point of the crossing lies on the cone `√(x² + y²) = |z|`: {point:?}"
        );
    }
}
