//! A bar moved by whole numbers keeps its seam corner exact.
//!
//! A round bar's cap corner is read out of the model and moved rather than
//! solved for, and the move used to cost it a box every time: half a width of
//! `CORNER_AFTER_MOVING_ROUNDOFF` times its size, whatever the move was. A shift
//! by whole numbers rounds nothing, so the box hid an exact answer. A bar whose
//! seam lies exactly on a block's wall then asked *side of plane* about a box
//! round `(0, 2, 1)` on `x = 0`, and refused, where the answer is `Zero`.
//!
//! This is a gap and not a limit (§16.10): the corner is exactly representable,
//! and only the box hid it.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use rcad_geom::{
    Axis, Cylinder, Isometry, Plane, Point, Scale, Surface, Vector,
    where_a_corner_lands_after_moving,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// The seam corner of a bar of radius one, moved by whole numbers, lands on
/// the whole-number point it was moved to and on no box round it; moved by a
/// tenth, which rounds, it keeps its box.
#[test]
fn a_bar_moved_by_whole_numbers_lands_its_seam_corner_on_an_exact_point() {
    let (corner, [cap, wall_of_the_bar]) = the_seam_corner_of_a_bar_and_its_surfaces();
    let on = [&cap, &wall_of_the_bar];

    let whole = Isometry::translation(Vector::new(-1.0, 2.0, 1.0)).expect("a whole-number shift");
    let (landed, wobble) = where_a_corner_lands_after_moving(corner, on, &whole)
        .expect("a finite corner and a finite shift");
    assert_eq!(
        landed,
        PredicatePoint::from([0.0, 2.0, 1.0]),
        "a whole-number shift of (1, 0, 0) rounds nothing"
    );
    assert_eq!(wobble.magnitude(), 0.0);

    let wall = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0))
        .expect("the x axis is a unit normal");
    assert_eq!(
        wall.side_of_point(landed),
        Decision::Zero,
        "the seam corner is on the wall x = 0"
    );

    let a_tenth = Isometry::translation(Vector::new(0.1, 2.0, 1.0)).expect("a finite shift");
    let (boxed, wobble) = where_a_corner_lands_after_moving(corner, on, &a_tenth)
        .expect("a finite corner and a finite shift");
    assert!(
        boxed.uncertainty() > 0.0 && wobble.magnitude() > 0.0,
        "1 + 0.1 rounds, so the corner is a box and not a point: {boxed:?}"
    );
}

/// A bar standing with its seam on a block's wall, handed over first: every
/// rule used to stop on *side of plane*, asked about the seam corner's box,
/// and now gets past it to the crossing the seam lies along.
///
/// `ABarBesideAWall { side: 4, radius: 1, along: 2, on_its_seam: true }` in
/// `fuses_that_touch.rs`. What it stops on now is the known gap §39.4 files:
/// a wall lying on a bar's skin leaves the crossing with no direction.
#[test]
fn a_bar_with_its_seam_on_a_blocks_wall_is_not_refused_on_which_side_of_the_wall_its_corner_is() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cube = a_cube(&mut model, 4.0);
    let bar = model
        .add_round_bar(
            1.0,
            2.0,
            &Isometry::translation(Vector::new(-1.0, 2.0, 1.0)).expect("a whole-number shift"),
        )
        .expect("a whole-number bar");

    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        let stopped_on = what_stops(&mut model.clone(), bar, cube, rule);
        assert_eq!(
            stopped_on.as_deref(),
            Some("which way a crossing runs"),
            "{rule:?} of the bar and the cube"
        );
    }
}

/// The corner a bar's seam starts from, and the two surfaces round it, as a bar
/// standing where it was built stores them.
fn the_seam_corner_of_a_bar_and_its_surfaces() -> (Point, [Surface; 2]) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(1.0, 2.0, &Isometry::identity())
        .expect("a whole-number bar");
    let corner = Point::new(1.0, 0.0, 0.0);
    assert!(
        model.vertices().any(|vertex| vertex.point() == corner),
        "a bar of radius one stands its seam at (1, 0, 0)"
    );
    let cap = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
            .expect("the z axis is a unit normal"),
    );
    let wall = Surface::Cylinder(Cylinder::new(Axis::z(), 1.0).expect("a radius of one"));
    let held: Vec<Surface> = model
        .solid(bar)
        .expect("the bar was just built")
        .faces()
        .map(|face| face.surface())
        .collect();
    assert!(
        held.contains(&cap) && held.contains(&wall),
        "the bar holds its bottom cap and its wall as these numbers: {held:?}"
    );
    (corner, [cap, wall])
}

fn a_cube(model: &mut Model, side: f64) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    model
        .add_extrusion(&square, side, &Isometry::identity())
        .expect("building a box needs no decision")
}

/// Which predicates a certified fuse gave up on, or `None` if it answered.
fn what_stops(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> Option<String> {
    let why = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .err()?;
    let Error::Undecidable { records } = &why else {
        return Some(format!("{why:?}"));
    };
    let gave_up: BTreeSet<_> = records.records().iter().map(|r| r.predicate()).collect();
    Some(gave_up.into_iter().collect::<Vec<_>>().join(", "))
}
