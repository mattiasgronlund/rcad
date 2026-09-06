//! The two ways a placed cone can be asked where its apex is, and whether they
//! agree.
//!
//! `docs/decisions.md` §56 turned on a cone face whose apex appeared to arrive
//! by two routes. Reading a placed face back gives the apex twice over:
//! `Cone::point_at(round, 0)` on the surface the face is meshed on, and
//! `Cone::transformed(location).apex()` on the way back out of the frame it was
//! meshed in. The argument that closed §56 is that those are **one expression
//! evaluated twice**, so no placement can separate them — and §16.10 says a
//! claim that something cannot happen needs a demonstration and not a paragraph.
//! This file is the demonstration, and it is here because the claim was argued
//! and not guarded.
//!
//! Two steps carry it, one per test below:
//!
//! 1. `point_at(round, 0)` is the stored apex plus two vectors that are zero,
//!    whatever angle is asked for.
//! 2. `transformed(m).apex()` is `m` applied to the stored apex.
//!
//! # The oracle is equality, with no tolerance anywhere
//!
//! Both claims are exact or they are worthless: the whole point is that a caller
//! cannot tell the two spellings apart, and "apart by less than a tolerance" is
//! being able to tell them apart. So every assertion here is `==` between
//! points.
//!
//! **`==` is not the whole claim, because `-0.0 == 0.0`.** §32.5's orderings —
//! `name_of_surface_bits`, `Surface::how_it_sorts_against` — use
//! `f64::total_cmp`, which sorts `-0.0` before `0.0`, so two spellings that agree
//! as values and differ as bits are two surfaces and not one. Both tests
//! therefore count bit differences as well and assert there are none. The count
//! is not decoration: `Cone::transformed` folds its apex through `Axis::holding`
//! and the raw `apply_to_point` spelling does not, so the two go through
//! different code and only agree because the fold moves nothing. "A shift by
//! negative zero" in [`movements`] is there to attack exactly that, and does not
//! break it.

use rcad_geom::{Axis, Cone, Isometry, Point, Vector};

/// Apexes that stress the addition rather than confirm it.
///
/// The origin is the case §56 was reported at. The rest are negative, mixed and
/// far enough out that a coordinate has no bits left for a small radius, which
/// is where an addition of zero would show up if it were not exact.
const APEXES: [[f64; 3]; 7] = [
    [0.0, 0.0, 0.0],
    [1.0, -2.0, 3.5],
    [-1e-7, 4e-3, -0.0],
    [1e6, -1e6, 1e6],
    [1.7e12, 1.7e12, 1.7e12],
    [-1e13, 3.0, 0.0],
    [1e-300, -1e-300, 1e-300],
];

/// Axis directions, including ones with a zero component and ones that are not
/// unit until [`Axis::new`] scales them.
const DIRECTIONS: [[f64; 3]; 6] = [
    [0.0, 0.0, 1.0],
    [-1.0, 0.0, 0.0],
    [1.0, 1.0, 1.0],
    [0.3, -0.7, 0.2],
    [-2.0, 0.0, 5.0],
    [1e-8, 1.0, -1e-8],
];

const SLOPES: [f64; 5] = [1e-6, 0.25, 1.0, 4.0, 1e6];

/// Angles round the cone, including the ones whose sine or cosine is not exact
/// and the ones that are negative or past a full turn.
const ROUNDS: [f64; 8] = [
    0.0,
    -0.0,
    0.7,
    core::f64::consts::FRAC_PI_2,
    core::f64::consts::PI,
    -1.3,
    7.5,
    -1e5,
];

fn cones() -> Vec<Cone> {
    let mut cones = Vec::new();
    for apex in APEXES {
        for direction in DIRECTIONS {
            let axis = Axis::new(
                Point::new(apex[0], apex[1], apex[2]),
                Vector::new(direction[0], direction[1], direction[2]),
            )
            .expect("a finite apex and a direction with a length is an axis");
            for slope in SLOPES {
                cones.push(Cone::new(axis, slope).expect("a positive finite slope is a cone"));
            }
        }
    }
    cones
}

/// Whether two points are the same number, and not merely the same value.
///
/// `-0.0 == 0.0` is true, so `==` cannot see the one difference §32.5 cares
/// about. Each caller tallies the failures and asserts on the total, so a
/// message names how many there were rather than only the first.
fn the_same_bits(a: Point, b: Point) -> bool {
    a.coordinates()
        .iter()
        .zip(b.coordinates().iter())
        .all(|(x, y)| x.to_bits() == y.to_bits())
}

/// Step one: no distance along the cone is the apex, whatever angle is asked.
///
/// `point_at` adds `(u·cos + v·sin)·(slope·0)` and `direction·0` to the stored
/// apex. Both are zero vectors, so this is the claim that adding them changes
/// nothing — for a coordinate of `1.7e12` as much as for the origin.
#[test]
fn asking_a_cone_for_a_point_no_distance_along_it_gives_the_apex_back() {
    let mut checked = 0;
    let mut away_from_the_origin = 0;
    let mut differed_in_bits = 0;
    let mut a_distance_along_moved_it = 0;

    for cone in cones() {
        let apex = cone.apex();
        for round in ROUNDS {
            if cone.point_at(round, 1.0) != apex {
                a_distance_along_moved_it += 1;
            }
            let asked = cone.point_at(round, 0.0);
            assert_eq!(
                asked,
                apex,
                "a cone of slope {} about {:?} put its apex at {asked:?} when asked at angle \
                 {round} and at {apex:?} when asked directly",
                cone.slope(),
                cone.axis().direction().components(),
            );
            if !the_same_bits(asked, apex) {
                differed_in_bits += 1;
            }
            checked += 1;
        }
        if apex != Point::origin() {
            away_from_the_origin += ROUNDS.len();
        }
    }

    println!(
        "{checked} cones asked; {away_from_the_origin} of them away from the origin; \
         {a_distance_along_moved_it} moved when asked a unit along"
    );
    assert!(
        checked > 1000,
        "only {checked} cases, which is not a sweep of the placements this has to hold for"
    );
    assert_eq!(
        a_distance_along_moved_it, checked,
        "{a_distance_along_moved_it} of {checked} cones moved off the apex when asked a unit \
         along it, so the rest would pass this whatever point_at did with its arguments"
    );
    assert!(
        away_from_the_origin * 2 > checked,
        "only {away_from_the_origin} of {checked} cases had an apex away from the origin, so this \
         is mostly the claim that the origin plus zero is the origin"
    );
    assert_eq!(
        differed_in_bits, 0,
        "{differed_in_bits} of {checked} cases agreed as values and differed as bits, which \
         §32.5 says would make one surface into two"
    );
}

/// Step two: moving the cone moves the apex to where the movement puts it.
///
/// `Cone::transformed` hands the axis to `Axis::transformed`, which applies the
/// movement to the origin and hands the result to `Axis::holding`. The other
/// spelling stops at `apply_to_point`, so `holding` is the whole difference
/// between the two paths and this is the claim that it does not move the point
/// it folds.
///
/// The bit count watches the only way it could: a moved apex carrying a `-0.0`,
/// which `holding` would turn into `0.0` and the raw spelling would keep.
/// Nothing in the sweep produces one — `-0.0 + 0.0` is `0.0` and a movement
/// reaches a coordinate through an addition — including the shift by negative
/// zero that is in [`movements`] to try.
#[test]
fn moving_a_cone_and_moving_its_apex_land_on_the_same_point() {
    let mut checked = 0;
    let mut the_movement_moved_the_apex = 0;
    let mut differed_in_bits = 0;

    for cone in cones() {
        for (how, movement) in movements() {
            let moved_the_cone = cone.transformed(&movement).apex();
            let moved_the_apex = movement.apply_to_point(cone.apex());
            assert_eq!(
                moved_the_cone,
                moved_the_apex,
                "moving a cone about {:?} by {how} put its apex at {moved_the_cone:?}, and moving \
                 the apex itself put it at {moved_the_apex:?}",
                cone.axis().origin().coordinates(),
            );
            if !the_same_bits(moved_the_cone, moved_the_apex) {
                differed_in_bits += 1;
            }
            if moved_the_apex != cone.apex() {
                the_movement_moved_the_apex += 1;
            }
            checked += 1;
        }
    }

    println!(
        "{checked} placements; {the_movement_moved_the_apex} of them moved the apex, \
         {differed_in_bits} agreed as values and differed as bits"
    );
    assert!(
        checked > 1000,
        "only {checked} cases, which is not a sweep of the placements this has to hold for"
    );
    assert!(
        the_movement_moved_the_apex * 2 > checked,
        "only {the_movement_moved_the_apex} of {checked} movements moved the apex at all, so this \
         is mostly the claim that standing still is standing still"
    );
    assert_eq!(
        differed_in_bits, 0,
        "{differed_in_bits} of {checked} placements agreed as values and differed as bits. The \
         two spellings are then two surfaces to §32.5's orderings, which is the thing §56's \
         argument says cannot happen"
    );
}

/// Placements, from standing still to the one the §56 report came in at.
///
/// A turn is what stops the moved apex being three copies of one exactly
/// representable number, and a turn about an axis away from the origin is what
/// stops the translation part being zero.
fn movements() -> Vec<(&'static str, Isometry)> {
    let shift = Isometry::translation(Vector::new(1.7e12, -3.0, 0.25)).expect("a shift");
    let turn = Isometry::rotation(&Axis::z(), 1.3).expect("a turn");
    let turn_off_the_origin = Isometry::rotation(
        &Axis::new(Point::new(2.0, -1.0, 0.5), Vector::new(1.0, 1.0, 0.0)).expect("an axis"),
        0.7,
    )
    .expect("a turn");
    vec![
        ("standing still", Isometry::identity()),
        ("a shift", shift),
        ("a turn", turn),
        ("a turn then a shift", shift.compose(&turn)),
        ("a shift then a turn", turn.compose(&shift)),
        ("a turn about a line off the origin", turn_off_the_origin),
        (
            "a small shift",
            Isometry::translation(Vector::new(1e-9, 0.0, -1e-9)).expect("a shift"),
        ),
        (
            "a shift by negative zero",
            Isometry::translation(Vector::new(-0.0, -0.0, -0.0)).expect("a shift"),
        ),
    ]
}
