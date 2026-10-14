//! The very long ellipse a lid cuts a cone in, cut again by a wall at two
//! points a quarter of a micrometre apart, and which of the two arcs between
//! them a half-space picks (Q1846, from Q1837).
//!
//! The cone has slope one half about `+z` and its apex at the origin. The lid
//! is `z = 4 + (2 − 2⁻²⁴)·y`, `2⁻²⁴` less steep than the ruling up `+y`, so it
//! cuts the cone in an ellipse from its near end at `(0, −1, 2)` to its far end
//! at `(0, 2²⁶, 2²⁷)`. That reaches more than
//! `PointsAlongACrossing::WALKED_ROUND_THE_CONE_PAST` times further up the axis
//! than the near end, so the walk places the arc that misses the far end round
//! the cone, ruling by ruling, and asks the run's half-spaces which of the two
//! arcs they pick in `AnArcRoundACone::which_arc`, not in
//! `Conic::the_stretch_this_run_covers` where Q1837 made the same choice.
//!
//! The wall stands square to the lid and cuts the ellipse on the rulings
//! `1.25e-7` either side of the near end. Each arc is asked at its middle: the
//! sliver's round the cone, inside a box reaching `256 ε` times the size of the
//! apex and the ends, `1.3e-13`, either way of it; the long arc's round the
//! ellipse, at its far end, inside a box reaching `256 ε` times the ellipse's
//! size, `4.3e-6`, either way. The sliver's middle is `8.7e-15` from the wall,
//! so no box round it tells which side it is on. The far end is `1.5e8` from
//! the wall and told outright.
//!
//! **One arc told is enough**, as in `the_two_arcs_of_a_cap_rim_cut_a_hair_apart.rs`.
//! Until Q1846 the walk round the cone read the sliver's untold sign as holding
//! it: with the long arc held as well that picked both and refused, and with
//! the long arc ruled out it picked the sliver but left an undecided record
//! behind, which a certified fuse refuses as `Undecidable`.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Error, HalfSpace, NamedRun, Picker, Plane, Point,
    PointsAlongACrossing, SIDE_OF_A_HALF_SPACE, Surface, Vector, walk,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog, Record};

/// The cone of slope one half about `+z`, apex at the origin.
fn the_cone() -> Cone {
    Cone::new(Axis::z(), 0.5).expect("a slope above zero")
}

/// `z = 4 + (2 − 2⁻²⁴)·y`, `2⁻²⁴` short of the ruling up `+y`.
fn the_lid() -> Plane {
    let lean = 2.0 - 2.0f64.powi(-24);
    Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, -lean, 1.0))
        .expect("a normal with a length")
}

/// Where the ruling at `round` radians round the cone meets the lid, worked
/// out here and not by the walk.
fn on_the_ruling_at(round: f64) -> Point {
    let lid = the_lid();
    let out = Vector::new(round.cos() * 0.5, round.sin() * 0.5, 1.0);
    let along = -lid.signed_distance_to(Point::origin()) / lid.normal().dot(out);
    Point::origin() + out * along
}

/// The angle round the cone of the near end, `(0, −1, 2)`.
const THE_NEAR_END: f64 = -core::f64::consts::FRAC_PI_2;

/// Where the wall cuts the ellipse, on the rulings either side of the near end.
fn the_ends() -> [Point; 2] {
    [
        on_the_ruling_at(THE_NEAR_END - 1.25e-7),
        on_the_ruling_at(THE_NEAR_END + 1.25e-7),
    ]
}

/// The ellipse's far end, `(0, y, 2y)` on the lid as it is stored, with `y`
/// from `n_y·y + n_z·2y = offset`. The bottom is a difference of two numbers
/// within a factor of two of each other, so it is exact.
fn the_far_end() -> Point {
    let lid = the_lid();
    let [_, across, up] = lid.normal().components();
    let out = lid.offset() / (across + 2.0 * up);
    Point::new(0.0, out, 2.0 * out)
}

/// The wall through both ends, square to the lid and to the chord between
/// them, facing up the lid towards the far end.
fn the_wall_facing_the_long_arc() -> HalfSpace {
    let [_, across, up] = the_lid().normal().components();
    let normal = Vector::new(0.0, up, -across);
    HalfSpace::new(normal, normal.dot(the_ends()[0] - Point::origin()))
        .expect("a normal with a length")
}

/// The shape is the one the module comment says: the sliver's middle is closer
/// to the wall than its box is wide, and the far end is not.
#[test]
fn the_wall_is_closer_to_the_slivers_middle_than_its_box_is_wide() {
    let wall = the_wall_facing_the_long_arc();
    let off_the_wall = |point: Point| wall.normal().dot(point - Point::origin()) - wall.offset();
    let middle = off_the_wall(on_the_ruling_at(THE_NEAR_END));
    assert!(
        middle < 0.0 && middle > -1e-14,
        "the sliver's middle is {middle:e} off the wall"
    );
    let far = off_the_wall(the_far_end());
    assert!(far > 1e8, "the far end is {far:e} off the wall");
}

/// The wall facing the long arc picks it: its middle, the far end, is held,
/// and the sliver's middle is too close to the wall to tell. The walk goes
/// round the long arc and leaves nothing undecided.
#[test]
fn the_long_arc_is_picked_when_it_is_held_and_the_sliver_cannot_be_told() {
    let (walked, records) = walked_with(&[the_wall_facing_the_long_arc()]);
    let along = walked.unwrap_or_else(|why| panic!("refused {why:?}, records {records:#?}"));
    assert!(records.is_empty(), "left undecided: {records:#?}");
    let deviation = along.deviation().magnitude();
    let far_end = the_far_end();
    let furthest = along
        .points()
        .iter()
        .map(|point| point.y())
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        furthest <= far_end.y() + deviation,
        "nothing on the ellipse is past {far_end:?}, and these points reach {furthest:e}"
    );
    let longest_step = along
        .points()
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).length())
        .fold(0.0, f64::max);
    let nearest_the_far_end = along
        .points()
        .iter()
        .map(|point| (*point - far_end).length())
        .fold(f64::INFINITY, f64::min);
    assert!(
        nearest_the_far_end <= longest_step,
        "the long arc passes {far_end:?}, and the nearest of these points is \
         {nearest_the_far_end:e} from it, more than the longest step of {longest_step:e}"
    );
    for point in along.points() {
        let off_the_lid = the_lid().signed_distance_to(*point).abs();
        let off_the_cone = how_far_off_the_cone(*point);
        assert!(
            off_the_lid <= deviation && off_the_cone <= deviation,
            "{point:?} is {off_the_lid:e} off the lid and {off_the_cone:e} off the cone, \
             against a deviation of {deviation:e}"
        );
    }
}

/// The wall turned round picks the sliver: the long arc's middle is ruled
/// out. The walk stays on the sliver, its one middle point is where the ruling
/// at the near end meets the lid, and nothing is left undecided.
#[test]
fn the_sliver_is_picked_with_nothing_left_undecided_when_the_long_arc_is_ruled_out() {
    let (walked, records) = walked_with(&[the_wall_facing_the_long_arc().reversed()]);
    let along = walked.unwrap_or_else(|why| panic!("refused {why:?}, records {records:#?}"));
    assert!(records.is_empty(), "left undecided: {records:#?}");
    let deviation = along.deviation().magnitude();
    let [middle] = along.points() else {
        panic!("one point between ends 2.5e-7 apart: {:?}", along.points());
    };
    let off = (*middle - on_the_ruling_at(THE_NEAR_END)).length();
    assert!(
        off <= deviation,
        "{middle:?} is {off:e} from the near end, against a deviation of {deviation:e}"
    );
}

/// Two half-spaces, each too close to tell at one middle and holding the other,
/// leave neither arc told: the walk is refused, and says which half-spaces
/// could not tell.
#[test]
fn neither_arc_told_is_still_refused() {
    let through_the_far_end = HalfSpace::new(Vector::new(0.0, -1.0, 0.0), -the_far_end().y())
        .expect("a normal with a length");
    let (walked, records) = walked_with(&[the_wall_facing_the_long_arc(), through_the_far_end]);
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

/// Walk the ellipse between [`the_ends`], picked by `half_spaces`, under a
/// certified policy, with a deflection of one: the long arc is `2²⁸` long.
fn walked_with(half_spaces: &[HalfSpace]) -> (Result<PointsAlongACrossing, Error>, Vec<Record>) {
    let run = NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(the_lid()),
            picked_by: half_spaces.iter().copied().map(Picker::from).collect(),
        },
        the_ends().map(|end| PredicatePoint::from(end.coordinates())),
    )
    .expect("a piece of a crossing with two ends");
    let mut log = ProvenanceLog::new();
    let walked = walk(
        &run,
        [&Surface::Cone(the_cone()), &Surface::Plane(the_lid())],
        Deflection::of(1.0).expect("a positive length"),
        &Policy::certified(),
        &mut log,
    );
    (walked, log.records().to_vec())
}

/// How far `point` sits off the cone, as a plain distance, read off a cross
/// product so that a miss does not cancel away.
fn how_far_off_the_cone(point: Point) -> f64 {
    let cone = the_cone();
    let along = cone.axis().direction();
    let out = point - cone.apex();
    (out.cross(along).length() - cone.slope() * out.dot(along).abs()).abs()
        / (along.length() * cone.slope().hypot(1.0))
}
