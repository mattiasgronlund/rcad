//! Which two surfaces a run lies between has one answer, and it does not depend
//! on which face a caller lists first.
//!
//! `docs/decisions.md` §23.8 is the rule — one question, one spelling — and
//! §221.5 is where it was found broken three ways: a private copy in `walk.rs`,
//! a narrower one in `rcad-topo`'s `Model::validate`, and a third in a mesher
//! test. The private one answered a closed run `[along[0], and]` whatever was
//! either side, so an `and` repeating the first face came back as that face
//! twice, which names no crossing at all, and the same edge with its faces the
//! other way round came back as the honest pair.

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, NamedRun, Picker, Plane, Point, Sphere, Surface, Vector,
    where_a_closed_crossing_starts, which_two_surfaces_a_run_is_between,
};
use rcad_predicates::ProvenanceLog;
use smallvec::smallvec;

/// A ball of radius 2 at the origin.
fn a_ball() -> Surface {
    Surface::Sphere(Sphere::new(Point::origin(), 2.0).expect("a positive radius"))
}

/// A pipe of radius 1 up the `z` axis through the origin, `x` off it by `shift`.
fn a_pipe(shift: f64) -> Surface {
    Surface::Cylinder(
        Cylinder::new(
            Axis::new(Point::new(shift, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0)).expect("z"),
            1.0,
        )
        .expect("a positive radius"),
    )
}

/// The plane `z = height`.
fn level_at(height: f64) -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, 1.0))
            .expect("z"),
    )
}

fn right_round(and: Surface) -> EdgeRun {
    EdgeRun::RightRoundWhereItCrosses { and }
}

fn a_piece_of(and: Surface) -> EdgeRun {
    EdgeRun::PartOfWhereItCrosses {
        and,
        picked_by: smallvec![Picker::from(
            HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
                .expect("x")
        )],
    }
}

fn the_same_up_to_order(one: [Surface; 2], other: [Surface; 2]) -> bool {
    one == other || one == [other[1], other[0]]
}

/// **The defect, as one case.** A ball and a pipe either side of a closed run
/// whose `and` is the ball, bit for bit, listed both ways round.
///
/// Before §221.5's fix `[ball, pipe]` answered `[ball, ball]` and `[pipe, ball]`
/// answered `[pipe, ball]`. An `and` that repeats a face is the writer saying it
/// adds nothing, so the pair either side stands, both ways round.
#[test]
fn a_run_that_names_one_of_two_curved_faces_again_names_both_either_way_round() {
    let ball = a_ball();
    let pipe = a_pipe(0.0);
    for and in [ball, pipe] {
        for run in [right_round(and), a_piece_of(and)] {
            let one_way = which_two_surfaces_a_run_is_between(&run, [&ball, &pipe]);
            let other_way = which_two_surfaces_a_run_is_between(&run, [&pipe, &ball]);
            assert!(
                the_same_up_to_order(one_way, [ball, pipe]),
                "{run:?} between a ball and a pipe named {one_way:?}"
            );
            assert!(
                the_same_up_to_order(other_way, [ball, pipe]),
                "{run:?} between a pipe and a ball named {other_way:?}"
            );
        }
    }
}

/// The same defect through the kernel's own door: where a closed crossing
/// starts. A plane cutting a ball is a circle, and the run names the ball again
/// as its `and`.
///
/// Listed `[ball, plane]` the old answer was the ball crossed with itself, which
/// is refused as one surface naming no crossing; listed `[plane, ball]` it was the
/// circle. Both have to be the circle, and the same point on it.
#[test]
fn where_a_closed_crossing_starts_does_not_care_which_face_is_listed_first() {
    let ball = a_ball();
    let flat = level_at(1.0);
    let run = NamedRun::the_whole_of(right_round(ball))
        .expect("a closed run carrying its own second surface needs nothing picking it");

    let one_way = where_a_closed_crossing_starts(&run, [&ball, &flat], &mut ProvenanceLog::new());
    let other_way = where_a_closed_crossing_starts(&run, [&flat, &ball], &mut ProvenanceLog::new());
    assert!(
        one_way.is_ok(),
        "a plane through a ball crosses it in a circle: {one_way:?}"
    );
    assert_eq!(one_way, other_way);
}

/// **The input `rcad-step` writes for a circle between one curved surface
/// written twice a hair apart** (§219.6, §221.5): the circle's own plane in
/// `and`. That plane is a third surface, so the answer is the first face and
/// the plane — exactly, not up to order, because the importer's answer is
/// pinned to that.
#[test]
fn a_closed_run_between_one_pipe_written_twice_names_the_pipe_and_its_plane() {
    let pipe = a_pipe(0.0);
    let written_again = a_pipe(1e-15);
    let rims_plane = level_at(3.0);
    assert_ne!(
        pipe, written_again,
        "the two spellings differ in their bits"
    );

    assert_eq!(
        which_two_surfaces_a_run_is_between(&right_round(rims_plane), [&pipe, &written_again]),
        [pipe, rims_plane]
    );
}

/// A circle a file imprinted on a flat face: the plane both sides, a hair apart,
/// and the circle's own pipe in `and` (§176.2, §219.6). `Model::validate` reads
/// a closed edge's winding off this pair, so it has to carry the pipe.
#[test]
fn a_closed_run_between_two_flat_faces_names_a_face_and_its_curved_second_surface() {
    let flat = level_at(0.0);
    let written_again = level_at(1e-15);
    let pipe = a_pipe(0.0);
    for run in [right_round(pipe), a_piece_of(pipe)] {
        assert_eq!(
            which_two_surfaces_a_run_is_between(&run, [&flat, &written_again]),
            [flat, pipe],
            "{run:?}"
        );
    }
}

/// Asking twice changes nothing. The mesher hands the walk a pair it has
/// already read off the run, and the walk reads it again, so an answer that
/// moved on a second asking would walk one curve and wind by another.
#[test]
fn handing_the_answer_back_in_gives_the_same_answer() {
    let surfaces = [
        a_ball(),
        a_pipe(0.0),
        a_pipe(1e-15),
        level_at(0.0),
        level_at(1e-15),
        level_at(1.0),
    ];
    let mut asked = 0;
    for and in surfaces {
        let runs = [
            EdgeRun::Straight,
            EdgeRun::RightRoundWhereItsFacesCross,
            right_round(and),
            a_piece_of(and),
        ];
        for run in &runs {
            for one in &surfaces {
                for other in &surfaces {
                    let once = which_two_surfaces_a_run_is_between(run, [one, other]);
                    let twice = which_two_surfaces_a_run_is_between(run, [&once[0], &once[1]]);
                    assert_eq!(once, twice, "{run:?} between {one:?} and {other:?}");
                    asked += 1;
                }
            }
        }
    }
    assert_eq!(asked, 6 * 4 * 6 * 6);
}
