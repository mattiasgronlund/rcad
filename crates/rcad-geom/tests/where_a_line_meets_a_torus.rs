//! A straight line meets a ring in nought, one, two, three or four places, and
//! all five are answered.
//!
//! # Why five and not three
//!
//! A line meets a quadric twice, so the only counts a quadric has are nought,
//! one and two, and the one in the middle is a graze. A ring's equation is
//! degree **four** along a line, and a quartic with real coefficients has an
//! even number of real roots counted with multiplicity — so three places and one
//! place are not near misses of four and two. **Each of them needs a double
//! root**, which is the line running tangent to the tube at one of the places
//! and crossing it square at the others. A solver that only chased sign changes
//! would answer four and two and nothing else, and would report a tangent bend
//! of pipe as missing its own neighbour.
//!
//! # What each answer is checked against
//!
//! [`Torus::side_of`] — the ring's own implicit equation, run through the whole
//! cascade — is asked about every place that comes back, and it may never say
//! [`Decision::Pos`] or [`Decision::Neg`]. Where the place landed on numbers the
//! arithmetic can hold, the check is the stronger [`Decision::Zero`]: not "the
//! ring cannot prove this is off me" but "the ring's own quartic is exactly zero
//! here".
//!
//! The five lines below are all built as two planes, because that is how
//! [`meet_of_three_surfaces`] is asked and how a fuse asks it.

use rcad_geom::{Axis, MeetOfThree, Plane, Point, Surface, Torus, Vector, meet_of_three_surfaces};
use rcad_predicates::{Decision, Policy, ProvenanceLog, SideOfSurface};

/// A ring of four and one about `+z`, centred on the origin.
///
/// Its outer equator is at radius five, its inner at three, and it reaches one
/// unit either side of `z = 0`.
fn a_ring() -> Torus {
    Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower than its ring")
}

fn a_plane(normal: [f64; 3], through: [f64; 3]) -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(
            Point::new(through[0], through[1], through[2]),
            Vector::new(normal[0], normal[1], normal[2]),
        )
        .expect("a normal with a length"),
    )
}

/// Every place two planes and the ring meet, ascending, each one checked against
/// the ring's own equation on the way past.
fn where_they_meet(one: Surface, other: Surface) -> Vec<[f64; 3]> {
    let ring = a_ring();
    let answer = meet_of_three_surfaces(
        [&one, &other, &Surface::Torus(ring)],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("two planes and a ring meet somewhere or nowhere, and neither is a refusal");

    if matches!(answer, MeetOfThree::Nowhere) {
        return Vec::new();
    }
    let MeetOfThree::At(places) = &answer else {
        panic!("two planes and a ring do not share a curve: {answer:?}");
    };

    let mut middles: Vec<[f64; 3]> = places
        .iter()
        .map(|place| {
            let side = ring.side_of(*place);
            assert!(
                !matches!(side, Decision::Pos | Decision::Neg),
                "the ring proved one of its own meeting places is off it: {side:?}"
            );
            // A place that had to be built is only known to be inside a box,
            // and a box straddling the skin is what a torus cannot place —
            // §2.5, and that is agreement rather than a failure. A place the
            // arithmetic landed on square gets the stronger check.
            if matches!(side, Decision::Undecidable { .. }) {
                assert!(
                    place.exact_coordinates().is_none(),
                    "a place written out in whole numbers has no excuse for a box"
                );
            } else {
                assert!(matches!(side, Decision::Zero), "{side:?}");
            }
            place.enclosure().map(|side| (side.lo() + side.hi()) / 2.0)
        })
        .collect();
    middles.sort_by(|one, other| {
        one.iter()
            .zip(other)
            .find(|(mine, theirs)| mine != theirs)
            .map_or(core::cmp::Ordering::Equal, |(mine, theirs)| {
                mine.total_cmp(theirs)
            })
    });
    middles
}

/// The `x` axis runs across the hole and through both walls of the tube twice —
/// the four places a ring has that a quadric does not.
#[test]
fn a_line_straight_through_the_hole_meets_the_ring_four_times() {
    assert_eq!(
        where_they_meet(
            a_plane([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
            a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]),
        ),
        vec![
            [-5.0, 0.0, 0.0],
            [-3.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [5.0, 0.0, 0.0],
        ]
    );
}

/// **Three, and the middle one is a touch.** The line `x = 3, z = 0` runs down
/// the equator plane at the radius of the inner equator, so it grazes the hole
/// at `(3, 0, 0)` — a double root — and crosses the outer wall square at
/// `(3, ±4, 0)`. Three places, four roots.
///
/// A capped bend's seam runs along this circle, which is why the touching case
/// is the one that had to work and not a curiosity.
#[test]
fn a_line_grazing_the_inner_equator_and_crossing_the_outer_meets_the_ring_three_times() {
    assert_eq!(
        where_they_meet(
            a_plane([1.0, 0.0, 0.0], [3.0, 0.0, 0.0]),
            a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]),
        ),
        vec![[3.0, -4.0, 0.0], [3.0, 0.0, 0.0], [3.0, 4.0, 0.0]]
    );
}

/// Two, and neither of them lands on a number. The line `y = 4.5, z = 0` cuts
/// the outer wall where `x² = 4.75`, whose root is not an `f64` — so both places
/// come back as boxes, and the ring answers `Undecidable` about both, which is
/// the strongest thing that is true.
#[test]
fn a_line_across_one_side_of_the_ring_meets_it_twice() {
    let found = where_they_meet(
        a_plane([0.0, 1.0, 0.0], [0.0, 4.5, 0.0]),
        a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]),
    );
    assert_eq!(found.len(), 2);
    let reach = 4.75f64.sqrt();
    for (place, expected) in found.iter().zip([-reach, reach]) {
        assert!(
            (place[0] - expected).abs() <= f64::EPSILON * 8.0,
            "{place:?} is not where x² = 4.75"
        );
        assert_eq!((place[1], place[2]), (4.5, 0.0));
    }
}

/// **One, and it is a touch.** The line `x = 5, z = 0` runs tangent to the outer
/// equator and never gets inside the tube, so the quartic has a double root at
/// `(5, 0, 0)` and a pair of complex ones. A solver that only looked for sign
/// changes would answer nowhere.
#[test]
fn a_line_tangent_to_the_outer_equator_meets_the_ring_once() {
    assert_eq!(
        where_they_meet(
            a_plane([1.0, 0.0, 0.0], [5.0, 0.0, 0.0]),
            a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]),
        ),
        vec![[5.0, 0.0, 0.0]]
    );
}

/// Nowhere, and that is an answer and not a refusal. The line `x = 6, z = 0`
/// runs a whole unit outside the widest the ring ever gets.
#[test]
fn a_line_clear_of_the_ring_meets_it_nowhere() {
    assert_eq!(
        where_they_meet(
            a_plane([1.0, 0.0, 0.0], [6.0, 0.0, 0.0]),
            a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]),
        ),
        Vec::<[f64; 3]>::new()
    );
}

/// Every place comes back exactly on the skin, whichever of the three positions
/// the ring is handed over in. The answer depends on which three surfaces they
/// are and not on the order they arrive in (§31.2), and a torus is the one kind
/// whose position in the triple used to decide whether it was answered at all.
#[test]
fn the_ring_is_answered_from_any_position_in_the_triple() {
    let ring = Surface::Torus(a_ring());
    let flat = [
        a_plane([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
        a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]),
    ];
    for triple in [
        [&flat[0], &flat[1], &ring],
        [&flat[0], &ring, &flat[1]],
        [&ring, &flat[0], &flat[1]],
    ] {
        let answer =
            meet_of_three_surfaces(triple, &Policy::certified(), &mut ProvenanceLog::new())
                .expect("the x axis meets this ring");
        let MeetOfThree::At(places) = &answer else {
            panic!("expected four places, got {answer:?}");
        };
        assert_eq!(places.len(), 4, "{triple:?}");
    }
}

/// A ring and a bit-for-bit copy of itself, met by one plane, is the case a
/// corpus can never draw and the one a bend fusing with its own neighbour
/// reaches. Two of the three still have to be flat, so this is refused for that
/// reason and not for the quartic — which is a different refusal and stays.
#[test]
fn two_rings_and_a_plane_still_want_two_flat_surfaces() {
    let ring = Surface::Torus(a_ring());
    let flat = a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
    assert_eq!(
        meet_of_three_surfaces(
            [&flat, &ring, &ring],
            &Policy::certified(),
            &mut ProvenanceLog::new()
        ),
        Err(rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
    );
}

/// The two planes meeting the ring at a place no `f64` names still hand back a
/// box, and the box is narrow: a place on the outer wall of a ring of four and
/// one is known to a handful of roundings and not to a fraction of the shape.
#[test]
fn a_place_that_is_not_a_number_still_comes_back_in_a_narrow_box() {
    let ring = a_ring();
    let answer = meet_of_three_surfaces(
        [
            &a_plane([0.0, 1.0, 0.0], [0.0, 4.5, 0.0]),
            &a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]),
            &Surface::Torus(ring),
        ],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("this line crosses the outer wall twice");
    let MeetOfThree::At(places) = &answer else {
        panic!("expected two places, got {answer:?}");
    };
    for place in places {
        assert!(
            place.uncertainty() < 1e-14,
            "a place on a ring of four and one came back in a box {} wide",
            place.uncertainty()
        );
    }
    assert_eq!(
        places.len(),
        2,
        "and the same two places a quadric's route would have found"
    );
}
