//! Every shape a plane and a cone can cross in, named and drawn.
//!
//! A plane cuts a cone in a circle, an ellipse, a parabola, a hyperbola, one
//! point, one line or two — seven answers, and `intersect` has to give a
//! different one for each. **It now names all seven**, and this file is the
//! table: **not one of them may come back as another**, because a hyperbola
//! named as an ellipse is a closed curve where there is none and that is the
//! wrong answer §4.1 ranks below both crashing and refusing.
//!
//! The two that used to be refused by name — the parabola and the hyperbola —
//! are the two that run on for ever and bend. What they need beyond the rest is
//! a picker each, because a parabola lies in one nappe of the cone and a
//! hyperbola puts one branch in each, and a stretch to be drawn over, because
//! neither has an end. Both are checked below.
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
    Axis, Cone, Deflection, EdgeRun, Error, Isometry, NamedRun,
    POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF, Plane, Point, PointsAlongACrossing, Sphere, Surface,
    SurfaceMeet, Vector, intersect, walk, where_a_closed_crossing_starts,
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

/// Row 3: a plane parallel to one ruling and clear of the apex cuts one endless
/// bend, named as the piece of the crossing the nappe it lies in holds.
///
/// **`E` is exactly zero here and that is the whole point of the normal chosen.**
/// `(0, 1, 1)` stores as `(0, s, s)`, so `E = s² − 1·(0 + s²)` sums to zero on
/// the exact level rather than to a number a hair either side of it — which is
/// what would turn this row into row 2 or row 4 and make the test prove nothing.
///
/// The plane `y + z = 4` against `x² + y² = z²` is `x² = 16 − 8y`, which lies
/// wholly in `z ≥ 2`: the upper nappe, and that is the one the picker must name.
#[test]
fn a_plane_parallel_to_a_cones_side_cuts_it_in_one_endless_bend() {
    let cone = a_right_angled_cone();
    let along_a_ruling = a_plane(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 1.0, 1.0));

    for answer in [
        crossed(&along_a_ruling, &cone),
        crossed_the_other_way(&along_a_ruling, &cone),
    ] {
        let SurfaceMeet::Curves(runs) = answer.expect("a parabola is named") else {
            panic!("a plane along a cone's side cuts one endless bend");
        };
        assert_eq!(runs.len(), 1);
        assert!(
            runs[0].is_the_whole_curve(),
            "a parabola has no end to be cut at"
        );
        let EdgeRun::PartOfWhereItCrosses { and, picked_by } = runs[0].run() else {
            panic!("a parabola is the piece of the crossing that lies in one nappe");
        };
        assert_eq!(and, &Surface::Cone(cone));
        assert_eq!(picked_by.len(), 1, "one nappe is one half-space");

        // Every point of `x² = 16 − 8y`, out to a long way, is held; the mirror
        // of it in the other nappe is not.
        for reach in [0.0, 1.0, -3.0, 1e7] {
            let here = rcad_predicates::Point::from([
                reach,
                2.0 - reach * reach / 8.0,
                2.0 + reach * reach / 8.0,
            ]);
            assert_eq!(cone.side_of(here), Decision::Zero, "{here:?}");
            assert_ne!(
                picked_by[0].side_of_point(here),
                Decision::Neg,
                "the picker holds the whole of the parabola: {here:?}"
            );
        }
        assert_eq!(
            picked_by[0].side_of_point(rcad_predicates::Point::from([0.0, 0.0, -4.0])),
            Decision::Neg,
            "and rules out the nappe the parabola is not in"
        );
    }
}

/// Row 4: a plane steeper than the cone's own side and clear of the apex cuts a
/// hyperbola — **two** endless bends, one in each nappe, told apart by the flat
/// surface through the apex square to the axis.
#[test]
fn a_plane_steeper_than_a_cones_side_cuts_it_in_two_endless_bends_one_per_nappe() {
    let cone = a_right_angled_cone();
    // Down the axis: the normal is `(1, 0, 0)`, so `c` is exactly zero and
    // `E = −slope²` without a rounding anywhere.
    let down_the_axis = a_plane(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0));

    // The two branches really are there, one in each nappe: the plane `x = 1`
    // meets the cone at `(1, y, ±√(1 + y²))` and at nothing with `z = 0`.
    let on_a_branch = |up: f64, sideways: f64| {
        rcad_predicates::Point::from([1.0, sideways, up * (1.0 + sideways * sideways).sqrt()])
    };
    for up in [1.0, -1.0] {
        assert_eq!(cone.side_of(on_a_branch(up, 0.0)), Decision::Zero);
    }

    for answer in [
        crossed(&down_the_axis, &cone),
        crossed_the_other_way(&down_the_axis, &cone),
    ] {
        let SurfaceMeet::Curves(runs) = answer.expect("a hyperbola is named") else {
            panic!("a plane steeper than a cone's side cuts two endless bends");
        };
        assert_eq!(runs.len(), 2);

        let mut nappes_held = Vec::new();
        for run in &runs {
            assert!(run.is_the_whole_curve(), "a branch has no end to be cut at");
            let EdgeRun::PartOfWhereItCrosses { and, picked_by } = run.run() else {
                panic!("one of two branches is a piece of where the two surfaces cross");
            };
            assert_eq!(and, &Surface::Cone(cone));
            assert_eq!(picked_by.len(), 1, "one nappe is one half-space");

            let held: Vec<f64> = [1.0, -1.0]
                .into_iter()
                .filter(|up| {
                    [0.0, 2.0, -5.0, 1e7].into_iter().all(|sideways| {
                        picked_by[0].side_of_point(on_a_branch(*up, sideways)) != Decision::Neg
                    })
                })
                .collect();
            assert_eq!(
                held.len(),
                1,
                "one run holds one whole branch, got {held:?}"
            );
            nappes_held.push(held[0]);
        }
        assert!(
            nappes_held[0] != nappes_held[1],
            "the two runs are the two different branches, got {nappes_held:?}"
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

/// Rows 3 and 4 through the two public walking doors, where a caller has stated
/// the run as **closed** and it is not.
///
/// The refusal this row used to assert was the shape's own name, and the shape
/// is now drawn — so what is left to refuse is the claim the run makes rather
/// than the pair. A parabola and a hyperbola branch run on for ever, so a run
/// that says it goes right round names a curve that is not there, and
/// [`where_a_closed_crossing_starts`] has no one place to put a vertex on
/// either. Both doors say so under the same name every other endless crossing
/// gets.
///
/// [`NamedRun`] and [`EdgeRun`] are public, [`EdgeRun`] is `Deserialize`, and
/// both walk entry points take whatever pair of surfaces the caller hands them,
/// so a file read back in reaches this. Nothing here forges an internal state:
/// every value goes in through a published constructor.
#[test]
fn a_closed_run_a_caller_names_on_a_crossing_that_bends_endlessly_is_refused_by_name() {
    let cone = Surface::Cone(a_right_angled_cone());
    for (through, facing) in [
        (Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 1.0, 1.0)),
        (Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0)),
    ] {
        let flat = Surface::Plane(a_plane(through, facing));
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
            Some(Error::TheCrossingOfTheseTwoSurfacesIsNotClosed),
            "a run that claims to close on a crossing that does not: {flat:?}"
        );
        assert_eq!(
            where_a_closed_crossing_starts(&stated, [&flat, &cone], &mut ProvenanceLog::new())
                .err(),
            Some(Error::TheCrossingOfTheseTwoSurfacesIsNotClosed),
            "the other public way in reaches the same place: {flat:?}"
        );

        // And the runs `intersect` really mints get the same answer from the
        // door that has to place a vertex, because neither shape closes.
        let SurfaceMeet::Curves(runs) = intersect(
            &flat,
            &cone,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("both bending shapes are named") else {
            panic!("a plane clear of the apex cuts the cone in curves: {flat:?}");
        };
        for run in runs {
            assert_eq!(
                where_a_closed_crossing_starts(&run, [&flat, &cone], &mut ProvenanceLog::new())
                    .err(),
                Some(Error::TheCrossingOfTheseTwoSurfacesIsNotClosed),
                "there is no closed curve here to put a vertex on: {run:?}"
            );
        }
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

/// How far off both surfaces the worst point of a walk sits, as a share of the
/// bound the walk itself published for that run.
///
/// The distance to the plane is exact for a stored unit normal; the distance to
/// the cone is `|√(x² + y²) − |z||` scaled by `1/√(1 + t²)`, which is the
/// perpendicular distance to the nearer nappe for a point beside it.
fn the_worst_share_of_its_own_bound(
    walked: &PointsAlongACrossing,
    flat: &Plane,
    cone: &Cone,
) -> f64 {
    let room = walked.deviation().magnitude();
    let along = cone.axis().direction();
    let slope = cone.slope();
    walked
        .points()
        .iter()
        .map(|point| {
            let off_the_plane = (flat.normal().dot(*point - Point::origin()) - flat.offset()).abs();
            let out = *point - cone.apex();
            let up = out.dot(along) / along.length();
            let across = (out.length_squared() - up * up).max(0.0).sqrt();
            let off_the_cone = (across - slope * up.abs()).abs() / (1.0 + slope * slope).sqrt();
            off_the_plane.max(off_the_cone) / room
        })
        .fold(0.0, f64::max)
}

fn walked(flat: &Surface, cone: &Surface, run: &NamedRun, deflection: f64) -> PointsAlongACrossing {
    walk(
        run,
        [flat, cone],
        Deflection::of(deflection).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("an endless bend is walked without complaint")
}

/// The parabola a plane parallel to a cone's side cuts it in, walked, and
/// measured against the closed form.
///
/// The plane `y + z = 4` against `x² + y² = z²` is `x² = 16 − 8y`, so the curve
/// is `(x, 2 − x²/8, 2 + x²/8)` with its vertex at `(0, 2, 2)`. `‖bend‖` is
/// `√2/8`, so the stretch walked is `|s| ≤ 8/√2 = 4√2` and `x` runs over
/// `±4√2` — which this checks, because the stretch is the choice the walker
/// makes and a test that only looked at the shape would not see it move.
#[test]
fn the_parabola_a_plane_along_a_cones_side_cuts_is_walked_into_points_on_both() {
    let cone = a_right_angled_cone();
    let flat = a_plane(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 1.0, 1.0));
    let SurfaceMeet::Curves(runs) = crossed(&flat, &cone).expect("a parabola is named") else {
        panic!("a plane along a cone's side cuts one endless bend");
    };
    let drawn = walked(
        &Surface::Plane(flat),
        &Surface::Cone(cone),
        &runs[0],
        8.0 / 1000.0,
    );

    assert!(
        drawn.points().len() >= 8,
        "a bend eight across at a thousandth of that takes more than eight pieces, got {}",
        drawn.points().len()
    );
    let room = drawn.deviation().magnitude();
    for point in drawn.points() {
        assert!(
            (point.y() + point.z() - 4.0).abs() / 2.0f64.sqrt() <= room,
            "a point of the crossing lies in the plane `y + z = 4`: {point:?}"
        );
        assert!(
            (point.y() - (2.0 - point.x() * point.x() / 8.0)).abs() <= room * 8.0,
            "and on `x² = 16 − 8y`, which is the closed form: {point:?}"
        );
    }

    // The two ends of the stretch are kept, because nothing else knows where the
    // walk was cut off, and they are where `1/‖bend‖` puts them.
    let ends = [drawn.points()[0], drawn.points()[drawn.points().len() - 1]];
    let reach = ends.map(|end| end.x().abs());
    for far in reach {
        assert!(
            (far - 4.0 * 2.0f64.sqrt()).abs() < 0.01,
            "the stretch runs one bend-length either way from the vertex, got {far}"
        );
    }
    assert!(ends[0].x() * ends[1].x() < 0.0, "one end either side");
}

/// The two branches a plane steeper than a cone's side cuts it in, walked, one
/// per nappe.
///
/// The plane `x = 1` against `x² + y² = z²` is `z² − y² = 1`, so the two
/// branches are `(1, sinh σ, ±cosh σ)` with their vertices at `(1, 0, ±1)`. The
/// stretch walked is `cosh σ ≤ 2`, so each branch runs from `z = ±1` out to
/// `z = ±2` and `y` over `±√3`.
#[test]
fn the_two_branches_a_steeper_plane_cuts_are_walked_one_per_nappe() {
    let cone = a_right_angled_cone();
    let flat = a_plane(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0));
    let SurfaceMeet::Curves(runs) = crossed(&flat, &cone).expect("a hyperbola is named") else {
        panic!("a plane steeper than a cone's side cuts two endless bends");
    };
    assert_eq!(runs.len(), 2);

    let mut nappes_drawn = Vec::new();
    for run in &runs {
        let drawn = walked(
            &Surface::Plane(flat),
            &Surface::Cone(cone),
            run,
            1.0 / 1000.0,
        );
        assert!(
            drawn.points().len() >= 8,
            "a branch two tall at a thousandth of that takes more than eight pieces, got {}",
            drawn.points().len()
        );
        let room = drawn.deviation().magnitude();
        let mut nappe = 0.0f64;
        for point in drawn.points() {
            assert!(
                (point.x() - 1.0).abs() <= room,
                "a point of the branch lies in the plane `x = 1`: {point:?}"
            );
            assert!(
                (point.z() * point.z() - point.y() * point.y() - 1.0).abs() <= room * 8.0,
                "and on `z² − y² = 1`, which is the closed form: {point:?}"
            );
            assert!(
                point.z().abs() >= 1.0 - room && point.z().abs() <= 2.0 + room,
                "the stretch runs one semi-axis past the vertex: {point:?}"
            );
            if nappe == 0.0 {
                nappe = point.z().signum();
            }
            assert_eq!(
                point.z().signum(),
                nappe,
                "one branch stays in one nappe: {point:?}"
            );
        }
        nappes_drawn.push(nappe);
    }
    assert!(
        nappes_drawn[0] != nappes_drawn[1],
        "the two runs are the two different branches, got {nappes_drawn:?}"
    );
}

/// Every point of an endless bend is on both surfaces inside
/// [`POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF`], over a search wide enough to catch
/// the expression drifting out from under its own derivation.
///
/// # What this can fail on, and what it cannot
///
/// §15.6 asks a published bound for **a test that could fail, not a paragraph**,
/// and says in the same breath that such a test cannot show a bound is *tight*,
/// because roundings cancel. Both halves show up here and the numbers are worth
/// writing down rather than hiding.
///
/// - **It fails if the bound is wrong.** Every walked point is measured against
///   the deviation the walk itself published for that run, so a change to any
///   step of the chain that costs more than the derivation allows is caught.
///   The scaling assertion below is the sharper half: it fails if the error
///   stops being proportional to `L`.
/// - **It cannot fail on the constant being twice too big.** The worst this
///   search finds is printed, and it is around a twentieth of the bound. That is
///   the ordinary gap between a rounding *count* and a rounding *sum*: the
///   derivation charges 98 roundings their full half-ulp each and the same 98
///   errors, being signed and near enough independent, add to about `√98/√3`
///   of that. **Halving the constant does not fail this test**, and neither
///   would quartering it — what makes `64 · ε` a bound is the derivation, and
///   what this test does is stop the derivation being fiction.
///
/// The search sweeps what the chain is actually sensitive to: the **slope**,
/// because `C`'s bracket `1 − (1 + t²)(n̂·ŵ)²` loses its leading digits as `t`
/// nears one; how far off the plane the **apex** stands, because every term is a
/// multiple of it; and **both bending rows**, the parabola reached with the
/// power-of-two slopes whose `E` is exactly zero.
#[test]
fn every_point_along_an_endless_bend_is_on_both_surfaces_inside_the_published_bound() {
    let mut worst = 0.0f64;
    let mut worst_at = String::new();
    let mut walks = 0;
    let mut points = 0;

    let mut measure = |flat: Plane, cone: Cone, told: String, worst: &mut f64| {
        let Ok(SurfaceMeet::Curves(runs)) = crossed(&flat, &cone) else {
            return;
        };
        for run in &runs {
            let Ok(drawn) = walk(
                run,
                [&Surface::Plane(flat), &Surface::Cone(cone)],
                Deflection::of(1.0 / 64.0).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            ) else {
                continue;
            };
            walks += 1;
            points += drawn.points().len();
            let share = the_worst_share_of_its_own_bound(&drawn, &flat, &cone);
            if share > *worst {
                *worst = share;
                worst_at = told.clone();
            }
        }
    };

    // The hyperbola row: every plane steeper than the cone's own side.
    for slope_step in 1..40 {
        let slope = f64::from(slope_step) / 8.0;
        let cone = Cone::new(Axis::z(), slope).expect("a positive slope");
        for height_step in 1..24 {
            let height = f64::from(height_step) * 0.37;
            for lean_step in 0..6 {
                let lean = f64::from(lean_step) / 8.0;
                let flat = a_plane(
                    Point::new(height, 0.0, height / 3.0),
                    Vector::new(1.0, 0.0, lean),
                );
                measure(
                    flat,
                    cone,
                    format!("hyperbola, slope {slope}, height {height}, lean {lean}"),
                    &mut worst,
                );
            }
        }
    }

    // The parabola row. `E = 0` on a **stored** normal wants `c = t·b` after the
    // normalizing division, and a slope that is a power of two is what makes
    // that division give it: scaling by a power of two rounds nothing, so
    // `(0, 1, t)` stores as `(0, b, t·b)` to the bit and `E = t²b² − t²b²` sums
    // to zero exactly rather than to a hair either side of it.
    for slope in [0.25, 0.5, 1.0, 2.0, 4.0] {
        let cone = Cone::new(Axis::z(), slope).expect("a positive slope");
        for height_step in 1..40 {
            let height = f64::from(height_step) * 0.37;
            for out in [0.0, 1.0, 1024.0] {
                let flat = a_plane(Point::new(out, out, height), Vector::new(0.0, 1.0, slope));
                measure(
                    flat,
                    cone,
                    format!("parabola, slope {slope}, height {height}, out {out}"),
                    &mut worst,
                );
            }
        }
    }

    println!(
        "{walks} walks, {points} points; the worst sits at {worst:.4} of {} · ε · L, on {worst_at}",
        POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF / f64::EPSILON,
    );
    assert!(walks > 1000, "the search walked almost nothing: {walks}");
    assert!(
        points > 10_000,
        "the search measured almost nothing: {points}"
    );
    assert!(worst <= 1.0, "the bound is a bound, and this found {worst}");
    // Not vacuous: the arithmetic really is being made to round. A search that
    // found nothing at all would mean the measurement, not the walk, was wrong.
    assert!(
        worst > 0.001,
        "the search measured no rounding at all: {worst}"
    );
}

/// The same bend a thousand times bigger strays a thousand times as far, which
/// is what makes [`POINT_ALONG_AN_ENDLESS_BEND_ROUNDOFF`] a bound *per unit of
/// size* rather than an absolute number that happens to hold at one scale.
///
/// **This is the half of the bound a wide search cannot check**, and it is the
/// half that catches a term dropped out of `how_big_this_is`: a chain whose
/// error stopped being proportional to `L` would pass the search at one scale
/// and fail here.
#[test]
fn what_an_endless_bend_strays_grows_with_the_size_it_was_worked_out_from() {
    let mut shares = Vec::new();
    for scale in [1.0, 32.0, 1024.0, 32768.0] {
        let cone = Cone::new(
            Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0)).expect("a direction"),
            1.0,
        )
        .expect("a positive slope");
        let flat = a_plane(
            Point::new(0.0, 0.0, 4.0 * scale),
            Vector::new(0.0, 1.0, 1.0),
        );
        let SurfaceMeet::Curves(runs) = crossed(&flat, &cone).expect("a parabola is named") else {
            panic!("a plane along a cone's side cuts one endless bend");
        };
        let drawn = walked(
            &Surface::Plane(flat),
            &Surface::Cone(cone),
            &runs[0],
            scale / 128.0,
        );
        shares.push(the_worst_share_of_its_own_bound(&drawn, &flat, &cone));
    }

    println!("the share of the bound used at four scales: {shares:?}");
    for share in &shares {
        assert!(
            *share <= 1.0,
            "the bound holds at every scale, and this found {share}"
        );
    }
    let widest = shares.iter().copied().fold(0.0, f64::max);
    let narrowest = shares.iter().copied().fold(f64::MAX, f64::min);
    assert!(
        widest <= narrowest * 8.0 + 0.001,
        "a bend written 32768 times larger uses the same share of its own bound, got {shares:?}"
    );
}

/// Moving a cone and its plane together moves the walked parabola and changes
/// nothing else about it.
///
/// **This is the demonstration the range argument rests on** (§19). The stretch
/// walked is chosen from the two surfaces, and the 20-minute spike's
/// `√(size/‖bend‖)` would have failed here: `size` reads the *coordinates* of
/// the vertex and the apex, so the same parabola written a thousand units out
/// would get a longer stretch and a different count of points.
#[test]
fn moving_a_cone_and_its_plane_together_moves_the_walked_parabola_and_nothing_else() {
    let cone = a_right_angled_cone();
    let flat = a_plane(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 1.0, 1.0));
    let SurfaceMeet::Curves(runs) = crossed(&flat, &cone).expect("a parabola is named") else {
        panic!("a plane along a cone's side cuts one endless bend");
    };
    let here = walked(
        &Surface::Plane(flat),
        &Surface::Cone(cone),
        &runs[0],
        8.0 / 1000.0,
    );

    // A shift of whole numbers, so the two surfaces move with no rounding at
    // all and any difference left is the walker's own choice of stretch.
    let shift = Vector::new(1024.0, -2048.0, 512.0);
    let movement = Isometry::translation(shift).expect("a finite shift");
    let moved_cone = cone.transformed(&movement);
    let moved_flat = flat.transformed(&movement);
    let SurfaceMeet::Curves(moved_runs) =
        crossed(&moved_flat, &moved_cone).expect("a parabola is named after the move")
    else {
        panic!("the moved pair still cuts one endless bend");
    };
    let there = walked(
        &Surface::Plane(moved_flat),
        &Surface::Cone(moved_cone),
        &moved_runs[0],
        8.0 / 1000.0,
    );

    assert_eq!(
        here.points().len(),
        there.points().len(),
        "the same pair a thousand units out is walked into the same number of points"
    );
    for (before, after) in here.points().iter().zip(there.points()) {
        let strayed = (*after - (*before + shift)).length();
        assert!(
            strayed <= there.deviation().magnitude() + here.deviation().magnitude(),
            "point {before:?} moved to {after:?}, which is {strayed} off where the shift put it"
        );
    }
}
