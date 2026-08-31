//! The one way M0 can genuinely fail to decide (decisions.md §2.5).
//!
//! M0's geometry is planar, so there are no quadrics to make a predicate
//! struggle. Instead: intersect three planes into a point, then ask which side
//! of a plane that point is on. The intersection needs a division, so the point
//! is known only as a box. Level 1's error bound assumes exact inputs, level 2
//! straddles zero, and level 3 cannot help, because exact arithmetic on a
//! coordinate we only know approximately is an exact answer to the wrong
//! question.

use rcad_predicates::{
    Approximation, Cause, Decision, Level, Measures, Point, Policy, ProvenanceLog,
    meet_of_three_planes, orient3d_reporting_level, orient3d_tolerantly,
};

/// `x + y + z = 1`, `x = y` and `y = z`, which meet at `(1/3, 1/3, 1/3)`.
///
/// A third is not an `f64`, so the box around it has real width. The
/// determinant is 3, which is not a power of two, so nothing here divides out
/// exactly.
fn the_meeting_point() -> Point {
    meet_of_three_planes(
        [1.0, 1.0, 1.0, -1.0],
        [1.0, -1.0, 0.0, 0.0],
        [0.0, 1.0, -1.0, 0.0],
    )
    .expect("these three planes meet at one point")
}

/// Three corners of the plane `x + y + z = 1`, all exact. The meeting point
/// lies exactly on it, so the true answer is zero — and nothing available can
/// prove that.
fn the_plane_through_the_meeting_point() -> [Point; 3] {
    [
        Point::from([1.0, 0.0, 0.0]),
        Point::from([0.0, 1.0, 0.0]),
        Point::from([0.0, 0.0, 1.0]),
    ]
}

#[test]
fn three_planes_meeting_on_the_plane_we_ask_about_cannot_be_decided() {
    let [a, b, c] = the_plane_through_the_meeting_point();
    let meet = the_meeting_point();

    let (decision, level) = orient3d_reporting_level(a, b, c, meet);

    match decision {
        Decision::Neg | Decision::Pos => panic!("nothing here can prove a side"),
        Decision::Zero => panic!("nothing here can prove coplanarity either"),
        Decision::Undecidable { enclosure } => {
            assert!(
                enclosure > 0.0,
                "an undecided answer straddles a real width"
            );
            assert!(
                enclosure < 1e-14,
                "the box is one rounding wide, so the enclosure should be too, \
                 got {enclosure}"
            );
        }
        other => panic!("unhandled decision {other:?}"),
    }
    assert_eq!(
        level,
        Level::Interval,
        "exact arithmetic must decline to run on a constructed point"
    );
}

/// The same question with the fourth point read straight out of the model
/// instead of built by an intersection. Nothing else changes, and the answer
/// comes back proved.
#[test]
fn the_same_question_about_an_exact_point_on_that_plane_is_decided() {
    let [a, b, c] = the_plane_through_the_meeting_point();
    let (decision, _) = orient3d_reporting_level(a, b, c, Point::from([0.5, 0.25, 0.25]));

    assert_eq!(decision, Decision::Zero);
    assert!(decision.is_certified());
}

#[test]
fn a_constructed_point_well_off_the_plane_is_still_decided_by_the_interval() {
    let far_side = Point::from([0.0, 0.0, 2.0]);
    let [a, b, _] = the_plane_through_the_meeting_point();
    let meet = the_meeting_point();

    let (decision, level) = orient3d_reporting_level(a, b, far_side, meet);

    match decision {
        Decision::Neg | Decision::Pos => {}
        Decision::Zero => panic!("the meeting point is not on this plane"),
        Decision::Undecidable { enclosure } => panic!(
            "the box is nowhere near this plane, so the interval should have \
             settled it, but it came back {enclosure} wide"
        ),
        other => panic!("unhandled decision {other:?}"),
    }
    assert_eq!(level, Level::Interval);
}

#[test]
fn a_certified_policy_reports_the_undecided_answer_and_records_why() {
    let [a, b, c] = the_plane_through_the_meeting_point();
    let mut log = ProvenanceLog::new();

    let decision =
        orient3d_tolerantly(a, b, c, the_meeting_point(), &Policy::certified(), &mut log);

    match decision {
        Decision::Neg | Decision::Zero | Decision::Pos => {
            panic!("a certified policy must not invent an answer")
        }
        Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
        other => panic!("unhandled decision {other:?}"),
    }

    assert_eq!(log.len(), 1);
    let record = &log.records()[0];
    assert_eq!(record.predicate(), "orient3d");
    assert_eq!(record.inputs().len(), 4);
    assert_eq!(record.inputs()[3], the_meeting_point());
    assert_eq!(record.level(), Level::Interval);
    assert!(record.enclosure() > 0.0);
    assert_eq!(
        record.approximation(),
        None,
        "a certified policy resolves nothing, so it approximates nothing"
    );
}

/// Deliberately not at a working size of one. `orient3d` measures a volume, so
/// at a working size of one its enclosure and the wobble that enclosure stands
/// for are the same number — which is where a units bug of this shape hides
/// (decisions.md §12.4).
#[test]
fn a_tolerant_policy_reads_the_undecided_answer_as_coplanar_and_tags_it() {
    let [a, b, c] = the_plane_through_the_meeting_point();
    let working_size = 1000.0;
    let policy = Policy::tolerant(working_size);
    let mut log = ProvenanceLog::new();

    let decision = orient3d_tolerantly(a, b, c, the_meeting_point(), &policy, &mut log);

    assert_eq!(decision, Decision::Zero);
    assert_eq!(log.len(), 1);
    let record = &log.records()[0];
    let tag = record
        .approximation()
        .expect("a resolved record carries the tag to hang on the entity");
    assert_eq!(tag.cause(), Cause::UndecidableResolved);

    // The tag is a distance; the record keeps the volume the cascade saw.
    assert_eq!(record.measures(), Measures::Volume);
    // Divided once per dimension rather than by the square, because the square
    // is not a finite number at every working size an f64 can hold.
    assert_eq!(
        tag.magnitude(),
        record.enclosure() / working_size / working_size
    );
    assert_eq!(log.worst_wobble(working_size), tag.magnitude());
    assert!(policy.wobble_is_acceptable(tag.magnitude()));
}

#[test]
fn a_tolerant_policy_for_a_much_smaller_model_refuses_the_same_slop() {
    let [a, b, c] = the_plane_through_the_meeting_point();
    // A model a thousand times smaller will put up with a wobble a thousand
    // times smaller, and this enclosure stands for far more than that.
    let working_size = 1e-3;
    let policy = Policy::tolerant(working_size);
    let budget = policy.max_wobble();
    let mut log = ProvenanceLog::new();

    let decision = orient3d_tolerantly(a, b, c, the_meeting_point(), &policy, &mut log);

    match decision {
        Decision::Neg | Decision::Zero | Decision::Pos => {
            panic!("this enclosure moves a point far further than {budget}")
        }
        Decision::Undecidable { enclosure } => {
            assert!(Measures::Volume.wobble_of(enclosure, working_size) > budget);
        }
        other => panic!("unhandled decision {other:?}"),
    }
    assert_eq!(log.records()[0].approximation(), None);
}

/// An approximation tag is what ends up on an entity, so it has to be usable
/// on its own once the log is gone.
#[test]
fn the_tag_from_a_resolved_decision_stands_on_its_own() {
    let [a, b, c] = the_plane_through_the_meeting_point();
    let mut log = ProvenanceLog::new();
    orient3d_tolerantly(
        a,
        b,
        c,
        the_meeting_point(),
        &Policy::tolerant(1.0),
        &mut log,
    );

    let tag: Approximation = log.records()[0].approximation().unwrap();
    drop(log);

    assert!(tag.magnitude() > 0.0);
    assert_eq!(tag.cause(), Cause::UndecidableResolved);
}
