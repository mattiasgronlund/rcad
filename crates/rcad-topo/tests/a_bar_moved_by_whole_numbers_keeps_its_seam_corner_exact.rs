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

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Cylinder, Deflection, Isometry, Plane, Point, Scale, Surface, Vector,
    where_a_corner_lands_after_moving,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// How far under the closed form a bar's meshed volume may be.
///
/// A mesh at [`A_DEFLECTION`] round a circle of radius one is 0.33% short of
/// it. A ceiling on a measurement, not a tolerance anything is decided by.
const HOW_FAR_A_MESH_MAY_BE: f64 = 0.005;

/// How fine the meshes the volumes are read off are.
const A_DEFLECTION: f64 = 0.02;

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
/// rule used to stop on *side of plane*, asked about the seam corner's box, and
/// none is refused on it now.
///
/// `ABarBesideAWall { side: 4, radius: 1, along: 2, on_its_seam: true }` in
/// `fuses_that_touch.rs`. The bar fills `x` from −2 to 0 and the cube `x` from 0
/// to 4, so they meet only along the seam `(0, 2, z)`: the intersection is no
/// solid, and the difference is the bar whole, `π · 1² · 2`. The union's answer
/// is the two solids as they stand, and it is refused
/// `ResultIsNotManifold { count: 4 }` instead, a gap queue row Q371 owns.
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

    let union = model.clone().fuse(
        &[bar],
        &[cube],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(union, Err(Error::ResultIsNotManifold { count: 4 })),
        "the union of the bar and the cube, got {union:?}"
    );

    let mut shared = model.clone();
    assert!(
        answered(&mut shared, bar, cube, Rule::Intersection).is_empty(),
        "a bar and a cube whose insides are apart share no solid"
    );

    let mut cut = model.clone();
    let what_is_left = answered(&mut cut, bar, cube, Rule::Difference);
    assert_eq!(what_is_left.len(), 1, "the cube takes nothing off the bar");
    let the_bar = PI * 1.0 * 1.0 * 2.0;
    let found = volume_of(&cut, what_is_left[0]);
    assert!(
        found <= the_bar && (the_bar - found) / the_bar < HOW_FAR_A_MESH_MAY_BE,
        "the bar less the cube measured {found}, and the bar is {the_bar}"
    );
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

/// The solids a certified fuse answers with, having checked that it proved every
/// decision and wrote a model that validates.
fn answered(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> Vec<SolidKey> {
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(&[one], &[other], rule, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("{rule:?} of the bar and the cube refused: {why}"));
    assert!(
        log.records().is_empty(),
        "{rule:?}: every decision is proved, got {:?}",
        log.records()
    );
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{rule:?} wrote a model that does not validate: {why}"));
    answer.solids().to_vec()
}

fn volume_of(model: &Model, solid: SolidKey) -> f64 {
    model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(A_DEFLECTION).expect("a fiftieth of a millimetre is a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures")
        .volume()
}
