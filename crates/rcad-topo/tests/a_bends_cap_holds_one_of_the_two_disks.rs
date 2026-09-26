//! A bend's end cap holds its own disk and not the other one (§250).
//!
//! A plane through a torus's axis cuts it in **two** circles, and the inside of
//! the tube on that plane is both their disks. A bend's cap is one of them. So
//! the torus's sign alone cannot say what the cap holds: it is inside at the
//! centre of the other disk too, and zero all round the other circle.
//!
//! What tells the two disks apart is a flat half-space whose plane holds the
//! ring's axis and stands square to the cap: for the cap on `y = 0` that plane
//! is `x = 0`. It cuts the tube too, in the disks of two more circles, round
//! `(0, ±4, 0)`. The last three tests are about points there.

use rcad_geom::{Axis, Isometry, Point, SIDE_OF_A_HALF_SPACE, Scale};
use rcad_predicates::{Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, FaceKey, Model, OnAFace, SolidKey};

mod rings;

use rings::a_quarter_bend;

/// A quarter bend of ring four and tube one, and its cap on `y = 0`, whose
/// circle goes round `(4, 0, 0)` through the corner `(3, 0, 0)`.
///
/// The same plane cuts the ring in a second circle, round `(−4, 0, 0)`, and
/// nothing of the bend is there: it runs from `+x` to `+y`.
fn a_bend_and_its_cap_on_y_equals_zero() -> (Model, SolidKey, FaceKey) {
    a_bend_and_its_cap_through(4.0, 1.0)
}

/// A quarter bend of ring `ring` and tube `tube`, and its cap on `y = 0`,
/// found by its one corner `(ring − tube, 0, 0)`.
fn a_bend_and_its_cap_through(ring: f64, tube: f64) -> (Model, SolidKey, FaceKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let solid = a_quarter_bend(&mut model, ring, tube);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound bend");
    let corner = Point::new(ring - tube, 0.0, 0.0);
    let cap = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .find(|face| {
            face.surface().as_plane().is_some()
                && face
                    .wires()
                    .flat_map(|wire| wire.points())
                    .any(|found| found == corner)
        })
        .expect("the bend's cap through its inner corner")
        .key();
    (model, solid, cap)
}

fn at(x: f64, z: f64) -> PredicatePoint {
    PredicatePoint::from([x, 0.0, z])
}

/// Every point is on `y = 0`, and each answer is read off the distance from
/// the point to the two circle centres `(±4, 0, 0)` against the tube's radius
/// of one: under one is inside that disk, exactly one is on that circle.
#[test]
fn a_bends_cap_holds_its_own_disk_and_not_the_other() {
    let (model, solid, cap) = a_bend_and_its_cap_on_y_equals_zero();
    let bend = model.solid(solid).expect("this model made the key");
    let holds = |point| {
        bend.face_holds_the_point(cap, point, &Policy::certified(), &mut ProvenanceLog::new())
    };
    let mut wrong = Vec::new();
    for (point, expected, why) in [
        (at(4.0, 0.0), OnAFace::Held, "its own circle's centre"),
        (
            at(4.0, 0.5),
            OnAFace::Held,
            "half a tube above its own centre",
        ),
        (at(3.0, 0.0), OnAFace::OnItsBoundary, "its own corner"),
        (
            at(5.0, 0.0),
            OnAFace::OnItsBoundary,
            "its own circle, across from the corner",
        ),
        (
            at(4.0, 1.0),
            OnAFace::OnItsBoundary,
            "the top of its own circle",
        ),
        (
            at(4.0, 2.0),
            OnAFace::NotHeld,
            "a tube above its own circle",
        ),
        (
            at(0.0, 0.0),
            OnAFace::NotHeld,
            "the ring's centre, four from each circle's centre",
        ),
        (at(-4.0, 0.0), OnAFace::NotHeld, "the other circle's centre"),
        (
            at(-4.0, 0.5),
            OnAFace::NotHeld,
            "half a tube above the other centre",
        ),
        (at(-3.0, 0.0), OnAFace::NotHeld, "the other circle"),
        (at(-5.0, 0.0), OnAFace::NotHeld, "the other circle"),
        (
            at(-4.0, 1.0),
            OnAFace::NotHeld,
            "the top of the other circle",
        ),
    ] {
        let answer = holds(point);
        if !matches!(answer, Ok(found) if found == expected) {
            wrong.push(format!("{point:?}, {why}: {answer:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// The bend runs a quarter of the way round from `+x` to `+y`, so the other
/// disk on `y = 0`, half a turn away, is nowhere near it. A ray from any point
/// there starts on the cap's plane, and the cap is asked whether it holds the
/// point: holding it put the point on the bend's skin.
#[test]
fn a_point_in_the_other_disk_is_outside_the_bend() {
    let (model, solid, _) = a_bend_and_its_cap_on_y_equals_zero();
    let bend = model.solid(solid).expect("this model made the key");
    let wrong: Vec<_> = [
        Point::new(-4.0, 0.0, 0.0),
        Point::new(-4.0, 0.0, 0.5),
        Point::new(-3.0, 0.0, 0.0),
        Point::new(-5.0, 0.0, 0.0),
    ]
    .into_iter()
    .map(|point| {
        (
            point,
            bend.classify_point(point, &Policy::certified(), &mut ProvenanceLog::new()),
        )
    })
    .filter(|(_, found)| *found != Containment::Outside)
    .collect();
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// A point the tube holds on `x = 0`, the plane between the cap's disk and the
/// other one, is counted in the cap's disk: the half-space naming the cap's
/// circle keeps what is on its plane, as a picker keeps what is on it. The
/// fuse asks the same half-space through the same function, so this is its
/// rule too, though no piece it tries a point in can reach it (Q117).
///
/// **None of these points is on the cap's plane**, and no caller inside the
/// kernel hands the cap one: ray casting, the fuse and the overlap search all
/// ask about points on it, where `x = 0` meets only the ring's axis, three
/// from the tube. `face_holds_the_point` does not ask whether its point is on
/// the face, so this is the one way to reach that rule, and without this test
/// nothing notices it change (§250.6).
#[test]
fn a_point_in_the_tube_on_the_plane_between_the_two_disks_is_kept() {
    let (model, solid, cap) = a_bend_and_its_cap_on_y_equals_zero();
    let bend = model.solid(solid).expect("this model made the key");
    let mut wrong = Vec::new();
    for (point, expected, why) in [
        ([0.0, 4.0, 0.0], OnAFace::Held, "the other cap's centre"),
        (
            [0.0, 4.0, 0.5],
            OnAFace::Held,
            "half a tube above the other cap's centre",
        ),
        (
            [0.0, 3.0, 0.0],
            OnAFace::OnItsBoundary,
            "the other cap's corner, on the tube",
        ),
        (
            [0.0, 5.0, 0.0],
            OnAFace::OnItsBoundary,
            "the other cap's circle, across from its corner",
        ),
    ] {
        let answer = bend.face_holds_the_point(
            cap,
            PredicatePoint::from(point),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        if !matches!(answer, Ok(found) if found == expected) {
            wrong.push(format!("{point:?}, {why}: {answer:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// A small box round the other cap's centre is all inside the tube, so the
/// torus lets it through, and it reaches across `x = 0`, so the half-space
/// cannot say which disk it is in. That is a refusal, and the half-space is
/// what it names.
#[test]
fn a_box_across_the_plane_between_the_two_disks_is_refused() {
    let (model, solid, cap) = a_bend_and_its_cap_on_y_equals_zero();
    let bend = model.solid(solid).expect("this model made the key");
    let across = PredicatePoint::Constructed([
        Interval::new(-0.001, 0.001).expect("a box a thousandth either side of x = 0"),
        Interval::exactly(4.0),
        Interval::exactly(0.0),
    ]);
    let answer =
        bend.face_holds_the_point(cap, across, &Policy::certified(), &mut ProvenanceLog::new());
    assert!(is_the_half_space_giving_up(&answer), "{answer:?}");
}

/// The same refusal for a box that holds a point of the cap itself, so it is
/// not only a question asked off the cap's plane.
///
/// A fat bend, ring four and tube three, turned an eighth of a turn about its
/// axis, has its first cap on `x = y` and the plane between the two disks on
/// `x = −y`. The box `[−2.1, 2.1] × {2} × {0}` holds `(2, 2, 0)`, which is on
/// the cap, and reaches across `x = −y`, and the torus certifies all of it
/// inside the tube.
///
/// A bend square to the axes has no such box: a box holding a point of
/// `y = 0` and reaching across `x = 0` holds a point of the ring's axis, and
/// the axis is outside the tube.
#[test]
fn a_box_holding_a_point_of_a_turned_fat_bends_cap_is_refused_across_the_plane_between_the_disks() {
    let (mut model, solid, cap) = a_bend_and_its_cap_through(4.0, 3.0);
    let an_eighth_of_a_turn =
        Isometry::rotation(&Axis::z(), std::f64::consts::FRAC_PI_4).expect("a finite turn");
    model
        .place_solid(solid, &an_eighth_of_a_turn)
        .expect("this model made the key");
    let bend = model.solid(solid).expect("this model made the key");
    let on_the_cap = PredicatePoint::from([2.0, 2.0, 0.0]);
    let answer = bend.face_holds_the_point(
        cap,
        on_the_cap,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(answer, Ok(OnAFace::Held)),
        "(2, 2, 0) is a unit and a bit from the centre of the cap's circle, of radius three: \
         {answer:?}",
    );
    let across = PredicatePoint::Constructed([
        Interval::new(-2.1, 2.1).expect("a box from past x = -y to past x = y"),
        Interval::exactly(2.0),
        Interval::exactly(0.0),
    ]);
    let answer =
        bend.face_holds_the_point(cap, across, &Policy::certified(), &mut ProvenanceLog::new());
    assert!(is_the_half_space_giving_up(&answer), "{answer:?}");
}

fn is_the_half_space_giving_up(answer: &Result<OnAFace, Error>) -> bool {
    match answer {
        Err(Error::Undecidable { records }) => {
            !records.is_empty()
                && records
                    .records()
                    .iter()
                    .all(|record| record.predicate() == SIDE_OF_A_HALF_SPACE)
        }
        Ok(_) | Err(_) => false,
    }
}
