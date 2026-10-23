//! A point a classification places for certain leaves nothing in the log,
//! even where a ray fired from it was thrown away first (Q2407,
//! `docs/decisions.md` §24.13).
//!
//! `SolidView::classify_uncertain_point` fires up to three rays, and throws a
//! ray away when it grazes something: an edge met dead on, or a sign it cannot
//! tell. A sign it cannot tell is a record. Where a later ray, or an exact
//! point beside the point, places it, nothing the answer rests on was in doubt,
//! and its doc says a fully certified classification leaves the log untouched.
//! A fuse drains each placement's log into its own and refuses, under a
//! certified policy, any log that holds a record, so a record a thrown-away
//! ray left behind refused fuses whose every piece was placed.
//!
//! The shape is Q2267's: two quarter bends of ring 4 and tube 1, one placed
//! round by `−0.3565` and a copy placed a quarter turn further back, so the
//! copy's end cap touches the first bend's start cap. The points are a few
//! roundings either side of the copy's end cap's inner corner, at the ring's
//! inner equator, which is on the first bend's start cap's rim: there a ray's
//! `side of torus` cannot be told. Only the first bend is built, since only it
//! is asked.

use core::f64::consts::FRAC_PI_2;

use rcad_geom::{Axis, Isometry, Point, Scale};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, Model, SolidKey};

mod rings;

use rings::a_quarter_bend;

const START: f64 = -0.3565;

/// The first ray met the first bend's start cap on its rim and could not tell
/// `side of torus`, and so did the second; the third placed the point outside
/// the bend.
#[test]
fn a_point_the_third_ray_places_outside_a_bend_leaves_no_record() {
    let (model, first) = the_first_bend();
    let (placed, log) = what_it_says(&model, first, on_the_copys_end_cap(-4, 0.0, 0.0));
    assert_eq!(placed, Containment::Outside);
    assert!(log.is_empty(), "{:?}", log.records());
}

/// The first ray could not tell `side of torus`, and could not be fired at the
/// bend's ring either, so an exact point beside this one was fired from. The
/// first target that point was offered grazed something too; a later one
/// placed it outside.
#[test]
fn a_point_a_point_beside_it_places_outside_a_bend_leaves_no_record() {
    let (model, first) = the_first_bend();
    let (placed, log) = what_it_says(&model, first, on_the_copys_end_cap(-7, 0.0, 0.0));
    assert_eq!(placed, Containment::Outside);
    assert!(log.is_empty(), "{:?}", log.records());
}

/// Where no ray places the point, what each ray could not tell is why, and it
/// stays in the log: two rays could not tell `side of torus`, and none could be
/// fired at the bend's ring.
#[test]
fn a_point_no_ray_places_keeps_what_each_ray_could_not_tell() {
    let (model, first) = the_first_bend();
    let (placed, log) = what_it_says(&model, first, on_the_copys_end_cap(-8, 0.0, 0.0));
    assert!(
        matches!(placed, Containment::CannotFireARayAtAFacesSurface { .. }),
        "{placed:?}"
    );
    let asked: Vec<&str> = log
        .records()
        .iter()
        .map(|record| record.predicate())
        .collect();
    assert_eq!(asked, ["side of torus", "side of torus"]);
}

/// Every point from eight roundings in from the copy's end cap's inner corner
/// to sixty-four out, and a hair off the cap and off the ring's middle plane,
/// that is placed inside, outside or on the first bend leaves the log empty.
#[test]
fn every_point_by_the_copys_inner_corner_that_is_placed_leaves_no_record() {
    let (model, first) = the_first_bend();
    let mut placed_with_a_record = Vec::new();
    for roundings in -8..=64 {
        for off_the_cap in [0.0, 1e-16, -1e-16, 3e-16, -3e-16] {
            for off_the_middle in [0.0, 1e-16, -1e-16, 1e-15, -1e-15] {
                let at = on_the_copys_end_cap(roundings, off_the_cap, off_the_middle);
                let (placed, log) = what_it_says(&model, first, at);
                let for_certain = matches!(
                    placed,
                    Containment::Inside | Containment::Outside | Containment::OnBoundary
                );
                if for_certain && !log.is_empty() {
                    placed_with_a_record.push((
                        roundings,
                        off_the_cap,
                        off_the_middle,
                        placed,
                        log.len(),
                    ));
                }
            }
        }
    }
    assert!(placed_with_a_record.is_empty(), "{placed_with_a_record:?}");
}

fn the_first_bend() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let first = a_quarter_bend(&mut model, 4.0, 1.0);
    model
        .place_solid(first, &turned_round_by(START))
        .expect("this model made the key");
    (model, first)
}

/// A point on the copy's end cap `roundings` roundings of `3` out from its
/// inner corner along the ring's middle plane, moved `off_the_cap` square to
/// the cap and `off_the_middle` square to that plane, placed where the copy
/// is.
fn on_the_copys_end_cap(roundings: i32, off_the_cap: f64, off_the_middle: f64) -> PredicatePoint {
    let a_rounding_of_three = 2.0 * f64::EPSILON;
    let on_the_copy = Point::new(
        off_the_cap,
        3.0 + f64::from(roundings) * a_rounding_of_three,
        off_the_middle,
    );
    let at = turned_round_by(START - FRAC_PI_2).apply_to_point(on_the_copy);
    PredicatePoint::from(at.coordinates())
}

fn what_it_says(model: &Model, bend: SolidKey, at: PredicatePoint) -> (Containment, ProvenanceLog) {
    let mut log = ProvenanceLog::new();
    let placed = model
        .solid(bend)
        .expect("this model made the key")
        .classify_uncertain_point(at, &Policy::certified(), &mut log);
    (placed, log)
}

fn turned_round_by(turn: f64) -> Isometry {
    Isometry::rotation(&Axis::z(), turn).expect("a real angle")
}
