//! A bend's end cap holds its own disk and not the other one (§250).
//!
//! A plane through a torus's axis cuts it in **two** circles, and the inside of
//! the tube on that plane is both their disks. A bend's cap is one of them. So
//! the torus's sign alone cannot say what the cap holds: it is inside at the
//! centre of the other disk too, and zero all round the other circle.

use rcad_geom::{Point, Scale};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, FaceKey, Model, OnAFace, SolidKey};

mod rings;

use rings::a_quarter_bend;

/// A quarter bend of ring four and tube one, and its cap on `y = 0`, whose
/// circle goes round `(4, 0, 0)` through the corner `(3, 0, 0)`.
///
/// The same plane cuts the ring in a second circle, round `(−4, 0, 0)`, and
/// nothing of the bend is there: it runs from `+x` to `+y`.
fn a_bend_and_its_cap_on_y_equals_zero() -> (Model, SolidKey, FaceKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let solid = a_quarter_bend(&mut model, 4.0, 1.0);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound bend");
    let cap = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .find(|face| {
            face.surface().as_plane().is_some()
                && face
                    .wires()
                    .flat_map(|wire| wire.points())
                    .any(|corner| corner == Point::new(3.0, 0.0, 0.0))
        })
        .expect("the bend's cap through (3, 0, 0)")
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
        (at(4.0, 0.5), OnAFace::Held, "half a tube above its own centre"),
        (at(3.0, 0.0), OnAFace::OnItsBoundary, "its own corner"),
        (at(5.0, 0.0), OnAFace::OnItsBoundary, "its own circle, across from the corner"),
        (at(4.0, 1.0), OnAFace::OnItsBoundary, "the top of its own circle"),
        (at(4.0, 2.0), OnAFace::NotHeld, "a tube above its own circle"),
        (at(0.0, 0.0), OnAFace::NotHeld, "the ring's centre, four from each circle's centre"),
        (at(-4.0, 0.0), OnAFace::NotHeld, "the other circle's centre"),
        (at(-4.0, 0.5), OnAFace::NotHeld, "half a tube above the other centre"),
        (at(-3.0, 0.0), OnAFace::NotHeld, "the other circle"),
        (at(-5.0, 0.0), OnAFace::NotHeld, "the other circle"),
        (at(-4.0, 1.0), OnAFace::NotHeld, "the top of the other circle"),
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
