//! The rim of a cone's cap cut at two points a quarter of a micrometre apart,
//! and which of the two arcs between them a half-space picks (Q1837, from
//! Q1836's trace).
//!
//! The shape is the pointed cone of slope one half and height eight in
//! `rcad-topo`'s `a_cone_cap_whose_rim_runs_through_its_apex.rs`, with the
//! block turned onto the ruling `x = 0, y = z/2`. The block's floor cuts the
//! rim `z = 8` at `x = ∓1.2604847710616158e-7, y = 3.9999999999999982`, and
//! every fuse walks one of the two arcs between those ends with the floor as
//! its only half-space: the union and the cone less the block the long arc,
//! the other three the sliver. The numbers below are the ones those fuses
//! hand `walk`, printed from inside it.
//!
//! Each arc is asked at its middle, inside a box `256 ε · 8`, about
//! `4.5e-13`, wide. The sliver's middle is `(0, 4, 8)`, and the floor is
//! `1.5e-15` from it, under one ulp of the circle's size, so no honest box
//! round a point placed by trigonometry decides it. The long arc's middle,
//! `(0, −4, 8)`, is eight away and decided outright.
//!
//! **One arc decided is enough.** The half-spaces on a run are there to pick
//! one of its two arcs, so when one arc is surely held and the other cannot be
//! told, the held one is the run; and when one is surely ruled out, the other
//! is. Until Q1837 the walk read the sliver's undecided sign as holding it:
//! with the long arc held as well that picked both and refused, and with the
//! long arc ruled out it picked the sliver but left an undecided record behind,
//! which a certified fuse refuses as `Undecidable`.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Error, HalfSpace, NamedRun, Picker, Plane, Point,
    SIDE_OF_A_HALF_SPACE, Surface, Vector, walk,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog, Record};

/// Where the turned block's floor cuts the rim, either side of `x = 0`.
const ENDS: [[f64; 3]; 2] = [
    [-1.260_484_771_061_615_8e-7, 3.999_999_999_999_998_2, 8.0],
    [1.260_484_771_061_615_8e-7, 3.999_999_999_999_998_2, 8.0],
];

/// The floor's half-space as the union and the cone less the block hand it
/// over: it holds the long arc's middle and is `1.2e-35` short of the sliver's.
fn the_floor_facing_the_long_arc() -> HalfSpace {
    HalfSpace::new(
        Vector::new(0.0, -8.010_743_024_149_836e-21, 0.0),
        -3.204_297_209_659_933e-20,
    )
    .expect("a normal with a length")
}

/// The union's half-space picks the long arc: its middle is held, and the
/// sliver's middle is too close to the floor to tell. The walk goes round the
/// long arc and leaves nothing undecided.
#[test]
fn the_long_arc_is_picked_when_it_is_held_and_the_sliver_cannot_be_told() {
    let (walked, records) = walked_with(&[the_floor_facing_the_long_arc()]);
    let points = walked.unwrap_or_else(|why| panic!("refused {why:?}, records {records:#?}"));
    assert!(
        points.iter().any(|point| point.y() < -3.9),
        "the long arc passes (0, -4, 8), and these points do not: {points:?}"
    );
    assert!(records.is_empty(), "left undecided: {records:#?}");
}

/// The intersection's half-space, the union's turned round, picks the sliver:
/// the long arc's middle is ruled out. The walk stays on the sliver and leaves
/// nothing undecided.
#[test]
fn the_sliver_is_picked_with_nothing_left_undecided_when_the_long_arc_is_ruled_out() {
    let (walked, records) = walked_with(&[the_floor_facing_the_long_arc().reversed()]);
    let points = walked.unwrap_or_else(|why| panic!("refused {why:?}, records {records:#?}"));
    assert!(
        points
            .iter()
            .all(|point| point.x().abs() < 1.3e-7 && point.y() > 3.99),
        "the sliver is {:?} to {:?}, and these points are not on it: {points:?}",
        ENDS[0],
        ENDS[1]
    );
    assert!(records.is_empty(), "left undecided: {records:#?}");
}

/// Two half-spaces, each too close to tell at one middle and holding the other,
/// leave neither arc decided: the walk is refused, and says which half-spaces
/// could not tell.
#[test]
fn neither_arc_decided_is_still_refused() {
    let through_the_long_arcs_middle =
        HalfSpace::new(Vector::new(0.0, 1.0, 0.0), -4.0).expect("a normal with a length");
    let (walked, records) = walked_with(&[
        the_floor_facing_the_long_arc(),
        through_the_long_arcs_middle,
    ]);
    assert!(
        matches!(walked, Err(Error::CannotNameWhichPieceOfACrossingThisIs)),
        "{walked:?}"
    );
    assert_eq!(records.len(), 2, "{records:#?}");
    assert!(
        records
            .iter()
            .all(|record| record.predicate() == SIDE_OF_A_HALF_SPACE),
        "{records:#?}"
    );
}

/// Walk the rim `z = 8` of the cone of slope one half between [`ENDS`], picked
/// by `half_spaces`, under a certified policy.
fn walked_with(half_spaces: &[HalfSpace]) -> (Result<Vec<Point>, Error>, Vec<Record>) {
    let cap = Plane::from_point_and_normal(Point::new(0.0, 0.0, 8.0), Vector::new(0.0, 0.0, 1.0))
        .expect("the z axis is a unit normal");
    let skin = Surface::Cone(Cone::new(Axis::z(), 0.5).expect("a positive slope"));
    let run = NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: skin.clone(),
            picked_by: half_spaces.iter().copied().map(Picker::from).collect(),
        },
        ENDS.map(PredicatePoint::from),
    )
    .expect("a piece of a crossing with two ends");
    let mut log = ProvenanceLog::new();
    let walked = walk(
        &run,
        [&Surface::Plane(cap), &skin],
        Deflection::of(0.01).expect("a positive length"),
        &Policy::certified(),
        &mut log,
    )
    .map(|walked| walked.points().to_vec());
    (walked, log.records().to_vec())
}
