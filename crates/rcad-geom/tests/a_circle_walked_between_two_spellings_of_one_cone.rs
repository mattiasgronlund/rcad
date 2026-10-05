//! A circle walked between a cone about `+z` and the same cone about `−z`.
//!
//! A cone here is both nappes, so the cone about `+z` and the cone about `−z`
//! with one apex and one slope are one point set, and `intersect` has answered
//! the pair `SameSurface` since §357. `walk`'s arm for one cone twice still
//! asked whether the two held the same numbers, so a closed run between the two
//! spellings — the rim between two bands of one frustum wall, one band naming
//! the cone each way — was refused `ACrossingBetweenAConeAndAnotherCurvedSurface`
//! there, in both orders.
//!
//! # What the spelling changes about the circle, and what it does not
//!
//! The circle through the run's corner square to the axis has the **same centre
//! and the same radius, bit for bit**, whichever spelling builds it: turning the
//! axis round negates both the step up the axis and the height read along it,
//! and their product is the same number. It also starts at the same point,
//! because both spellings take the same first direction across. What turns round
//! is **which way it goes**: each spelling walks its circle right-handed about
//! its own axis, so the two walk one circle in opposite directions.
//!
//! So a pair of two spellings has to pick one, and pick it from the numbers and
//! not from which face was listed first, or one closed edge is walked two ways
//! by the two faces either side of it. `walk` takes the spelling that
//! `Surface::how_it_sorts_against` puts first, which is the one `rcad-topo`'s
//! `the_curved_one_of` reads the edge's direction off.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Error, NamedRun, PointsAlongACrossing, Result, Surface, walk,
    where_a_closed_crossing_starts,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};

/// A half out per unit up, apex at the origin: a power of two, so the corner
/// below sits on the cone exactly.
const SLOPE: f64 = 0.5;

/// How far up the axis the circle runs round, and so how far out it is.
const HEIGHT: f64 = 4.0;
const RADIUS: f64 = SLOPE * HEIGHT;

#[test]
fn a_circle_between_a_cone_and_its_reversed_spelling_is_walked_in_both_orders() {
    let [up, down] = [about(Axis::z()), about(Axis::z().reversed())];
    for (along, which) in [([&up, &down], "up first"), ([&down, &up], "down first")] {
        let walked = walked(along).unwrap_or_else(|why| {
            panic!(
                "{which}: the two spellings are one cone, so a closed run between them is the \
                 circle through its corner, and it was refused {why:?}"
            )
        });
        let room = walked.deviation().magnitude();
        assert!(
            walked.points().len() > 8,
            "{which}: a circle of radius {RADIUS} walked at deflection 0.01 has more than eight \
             points between its ends, and came back with {}",
            walked.points().len()
        );
        for point in walked.points() {
            let out = point.x().hypot(point.y());
            assert!(
                (point.z() - HEIGHT).abs() <= room && (out - RADIUS).abs() <= room,
                "{which}: {point:?} is off the circle of radius {RADIUS} at height {HEIGHT} by \
                 more than the {room} the walk publishes"
            );
        }
    }
}

/// Both orders walk the circle exactly as the spelling that sorts first walks it
/// against itself, and start it in the same place.
#[test]
fn both_orders_walk_the_circle_the_way_the_spelling_that_sorts_first_winds() {
    let [up, down] = [about(Axis::z()), about(Axis::z().reversed())];
    let first = if up.how_it_sorts_against(&down).is_le() {
        up
    } else {
        down
    };
    let alone = walked([&first, &first]).expect("one cone twice is walked");
    let starts_alone = starts([&first, &first]).expect("one cone twice starts somewhere");

    for (along, which) in [([&up, &down], "up first"), ([&down, &up], "down first")] {
        let walked = walked(along)
            .unwrap_or_else(|why| panic!("{which}: one cone written two ways, refused {why:?}"));
        assert_eq!(
            walked.points(),
            alone.points(),
            "{which}: the pair walks its circle the way the spelling that sorts first does, \
             point for point, whichever face is listed first"
        );
        assert_eq!(walked.deviation(), alone.deviation(), "{which}");
        assert_eq!(
            starts(along).unwrap_or_else(|why| panic!("{which}: refused {why:?}")),
            starts_alone,
            "{which}: the one vertex a closed crossing takes goes where it goes for one spelling"
        );
    }
}

/// Each spelling alone walks the one circle, with the same bound, from the same
/// corner — and the other way round.
///
/// This is the whole of what the spelling changes, and the reason the pair has
/// to choose one of the two: the point a third of the way round one walk is the
/// point a third of the way round the other from the far end.
#[test]
fn the_two_spellings_alone_walk_one_circle_the_opposite_way_round() {
    let [up, down] = [about(Axis::z()), about(Axis::z().reversed())];
    let [up_walked, down_walked] = [&up, &down].map(|cone| {
        walked([cone, cone]).expect("one cone twice is walked, whichever way it is written")
    });

    assert_eq!(up_walked.deviation(), down_walked.deviation());
    assert_eq!(
        starts([&up, &up]).expect("a circle starts somewhere"),
        starts([&down, &down]).expect("a circle starts somewhere"),
        "both spellings take the same first direction across, so the circle starts at one place"
    );
    assert!(
        up_walked.points()[0].y() > 0.0 && down_walked.points()[0].y() < 0.0,
        "about +z the circle leaves (2, 0, 4) towards +y, and about -z towards -y; it went to \
         {:?} and {:?}",
        up_walked.points()[0],
        down_walked.points()[0]
    );

    let room = up_walked.deviation().magnitude();
    assert_eq!(up_walked.points().len(), down_walked.points().len());
    for (one, other) in up_walked
        .points()
        .iter()
        .zip(down_walked.points().iter().rev())
    {
        assert!(
            (*one - *other).length() <= 2.0 * room,
            "{one:?} on the walk about +z and {other:?} on the one about -z, taken from the far \
             end, are the same place on one circle and stand further apart than both bounds"
        );
    }
}

/// The test is exact: a cone one ulp steeper, about `−z`, is another cone, and
/// the pair is refused by name as it always was.
#[test]
fn a_reversed_spelling_one_ulp_steeper_is_another_cone_and_is_still_refused() {
    let up = about(Axis::z());
    let steeper =
        Surface::Cone(Cone::new(Axis::z().reversed(), SLOPE.next_up()).expect("a positive slope"));
    for along in [[&up, &steeper], [&steeper, &up]] {
        let walked = walked(along);
        assert!(
            matches!(
                walked,
                Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface)
            ),
            "two cones a hair apart cross in a curve nothing here can draw, got {walked:?}"
        );
    }
}

fn about(axis: Axis) -> Surface {
    Surface::Cone(Cone::new(axis, SLOPE).expect("a positive slope"))
}

/// A closed run through `(2, 0, 4)` that names no second surface: the rim
/// between two bands of one frustum wall, cut level.
fn the_rim() -> NamedRun {
    NamedRun::new(
        EdgeRun::RightRoundWhereItsFacesCross,
        [PredicatePoint::from([RADIUS, 0.0, HEIGHT]); 2],
    )
    .expect("one point at both ends is a closed run")
}

fn walked(along: [&Surface; 2]) -> Result<PointsAlongACrossing> {
    walk(
        &the_rim(),
        along,
        Deflection::of(0.01).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

fn starts(along: [&Surface; 2]) -> Result<rcad_geom::Point> {
    where_a_closed_crossing_starts(&the_rim(), along, &mut ProvenanceLog::new())
}
