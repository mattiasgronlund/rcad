//! §244.3's block edge along `x`, grazing a ball centred at `x = 16`: what a
//! certified fuse refuses on, and why the sign it refuses on is a gap.
//!
//! # The shape
//!
//! A ball of radius `1 + 2⁻⁵¹` centred at `(16, 0, 0)`, and a block whose top
//! is `z = −2⁻²⁵` and whose side is `y = 1`. That edge meets the ball at
//! `x = 16 ± 2⁻⁵¹`, which are not `f64`s: the two places come back as the boxes
//! `[15.999999999999998, 16]` and `[16, 16.000000000000004]`.
//!
//! # The order of the two places is decided
//!
//! The walked start of both the top's circle and the side's is `(16, 1, −2⁻²⁵)`,
//! on the edge between the two places and `2⁻¹⁰³` inside the ball. All three
//! points lie on one line along `x`, so going round either circle from that
//! start the two places sweep exactly zero, and their boxes touch at `16`.
//! From §244.5 until Q699 a certified fuse refused on which of the two comes
//! first. It is decided now: the one point of either box at `x = 16` is the
//! start, which is not on the ball, so one place is proved past the start and
//! the other short of it, and the one past it along the way the circle runs
//! comes first (`fuse/arrange.rs`,
//! `two_places_whose_boxes_end_at_the_start_put_the_one_past_it_first`).
//!
//! # What refuses next, and why that is a gap
//!
//! Splitting the ball's face, `Arrangement::turns_anticlockwise` in
//! `fuse/arrange.rs` asks which way two crossings turn at a place they meet:
//! the sign of the determinant of the ball's gradient there and the two
//! planes' normals. The gradient is the place minus the ball's centre, taken
//! from the box, so its `x` is `[−2⁻⁴⁹, 0]` and the sign cannot be told. That
//! is what every certified fuse refuses on with the ball first, the record
//! §317.1 found before §244.5 folded the two places into one node and Q663
//! stopped the fold. The seam of the ball is not involved; `docs/decisions.md`
//! §248.3 guessed that it was.
//!
//! The sign exists. The determinant is minus the gradient's `x`, which is
//! `∓2⁻⁵¹`, never zero. And the same three surfaces solved about the ball's
//! own centre meet at two `f64` triples, where the same determinant is decided.
//! So this is a gap and not a limit (§16.10). `docs/decisions.md` §317 has the
//! measurement and the question a fix should ask.

use rcad_geom::{
    Isometry, MeetOfThree, Plane, Point, Scale, Sphere, Surface, Vector, meet_of_three_surfaces,
};
use rcad_predicates::{
    Decision, Interval, Level, Point as PredicatePoint, Policy, ProvenanceLog, Record,
    orient3d_of_points,
};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule};

/// The name `turns_anticlockwise`'s determinant records a refusal under.
const THE_TURN: &str = "orient3d";

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The pin: under a certified policy every rule refuses on the turn at the
/// place below `x = 16`, and on nothing else.
///
/// From §244.5 until Q699 every rule refused one step earlier, on which of the
/// two places comes first round a circle.
#[test]
fn a_certified_fuse_refuses_on_the_turn_at_the_place_below_16() {
    let places = where_the_edge_meets_a_ball_centred_at(16.0);
    let below: Vec<PredicatePoint> = places
        .iter()
        .copied()
        .filter(|place| place.enclosure()[0].hi() == 16.0)
        .collect();
    let [below] = below[..] else {
        panic!("one place has its box end at 16 from below: {places:?}");
    };
    let gradient = gradient_of_the_ball(below, 16.0);
    for rule in EVERY_RULE {
        let mut log = ProvenanceLog::new();
        let refused = match fuse_a_block_edge_grazing_a_ball(rule, &mut log) {
            Err(Error::Undecidable { records }) => records.records().to_vec(),
            other => panic!("{rule:?}: {other:?}"),
        };
        assert_eq!(refused.len(), 1, "{rule:?}: {refused:#?}");
        assert!(
            is_the_turn_at(&refused[0], gradient),
            "{rule:?}: {:#?}",
            refused[0]
        );
    }
}

/// The measurement that makes it a gap: the sign the turn needs exists, and the
/// same three surfaces solved about the ball's own centre reach it.
///
/// Moving the ball to the origin moves nothing else. Both planes' normals have
/// no `x` part, so a shift along `x` leaves their four numbers as they were,
/// and the two problems are one problem `16` apart. The turn reads only the
/// place *minus the centre*, so it is the same number in both.
#[test]
fn the_sign_that_turn_needs_is_there_when_the_ball_is_solved_about_its_own_centre() {
    let (lid, side) = the_block_edge();
    for flat in [lid, side] {
        assert_eq!(
            flat.normal().components()[0],
            0.0,
            "a shift along x must leave the plane as it was"
        );
    }

    let placed = where_the_edge_meets_a_ball_centred_at(16.0);
    let centred = where_the_edge_meets_a_ball_centred_at(0.0);

    for place in &placed {
        let turn = the_turn_at(gradient_of_the_ball(*place, 16.0));
        assert!(
            matches!(turn, Decision::Undecidable { .. }),
            "about x = 16 the box of the place touches the centre: {place:?} gave {turn:?}"
        );
    }

    let quarter = f64::powi(2.0, -51);
    let top = -f64::powi(2.0, -25);
    let mut signs = Vec::new();
    for place in &centred {
        let [x, y, z] = collapsed(*place);
        assert!(x == -quarter || x == quarter, "{place:?}");
        assert_eq!([y, z], [1.0, top]);
        let turn = the_turn_at(gradient_of_the_ball(*place, 0.0));
        let expected = if x < 0.0 {
            Decision::Pos
        } else {
            Decision::Neg
        };
        assert_eq!(turn, expected, "the determinant is minus x, at x = {x:e}");
        signs.push(turn);

        let holding_it = placed
            .iter()
            .filter(|world| {
                let [box_of_x, ..] = gradient_of_the_ball(**world, 16.0).enclosure();
                box_of_x.lo() <= x && x <= box_of_x.hi()
            })
            .count();
        assert_eq!(
            holding_it, 1,
            "each place about the centre is inside exactly one place about x = 16"
        );
    }
    signs.sort_by_key(|sign| *sign == Decision::Pos);
    assert_eq!(signs, [Decision::Neg, Decision::Pos]);
}

fn fuse_a_block_edge_grazing_a_ball(
    rule: Rule,
    log: &mut ProvenanceLog,
) -> rcad_topo::Result<Fused> {
    let centre_x = 16.0;
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let ball = model
        .add_ball(
            1.0 + f64::powi(2.0, -51),
            &Isometry::translation(Vector::new(centre_x, 0.0, 0.0)).expect("a finite shift"),
        )
        .expect("a positive radius");
    let square = ProfileBuilder::new(vec![
        [centre_x - 2.0, -2.0],
        [centre_x + 2.0, -2.0],
        [centre_x + 2.0, 1.0],
        [centre_x - 2.0, 1.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    let top = -f64::powi(2.0, -25);
    let block = model
        .add_extrusion(
            &square,
            2.0,
            &Isometry::translation(Vector::new(0.0, 0.0, top - 2.0)).expect("a finite shift"),
        )
        .expect("a positive height");
    model.fuse(&[ball], &[block], rule, &Policy::certified(), log)
}

/// Whether a record is the turn at the place whose ball gradient is `gradient`,
/// with the block's top and side as its other two rows, as wide as that
/// gradient's `x`, `2⁻⁴⁹`.
fn is_the_turn_at(record: &Record, gradient: PredicatePoint) -> bool {
    record.predicate() == THE_TURN
        && record.level() == Level::Interval
        && record.inputs()
            == [
                gradient,
                PredicatePoint::from([0.0, 0.0, 1.0]),
                PredicatePoint::from([0.0, 1.0, 0.0]),
            ]
        && record.enclosure() == f64::powi(2.0, -49)
}

/// The block's top, `z = −2⁻²⁵`, and its side, `y = 1`, facing the way the
/// fuse's record has them.
fn the_block_edge() -> (Plane, Plane) {
    let top = -f64::powi(2.0, -25);
    let lid = Plane::from_point_and_normal(Point::new(0.0, 0.0, top), Vector::new(0.0, 0.0, 1.0))
        .expect("a unit normal");
    let side = Plane::from_point_and_normal(Point::new(0.0, 1.0, 0.0), Vector::new(0.0, 1.0, 0.0))
        .expect("a unit normal");
    (lid, side)
}

fn where_the_edge_meets_a_ball_centred_at(centre_x: f64) -> Vec<PredicatePoint> {
    let (lid, side) = the_block_edge();
    let ball = Sphere::new(Point::new(centre_x, 0.0, 0.0), 1.0 + f64::powi(2.0, -51))
        .expect("a positive radius");
    let meet = meet_of_three_surfaces(
        [
            &Surface::Sphere(ball),
            &Surface::Plane(lid),
            &Surface::Plane(side),
        ],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a ball and two planes");
    let MeetOfThree::At(places) = meet else {
        panic!("the edge grazes the ball at two places: {meet:?}");
    };
    assert_eq!(places.len(), 2, "{places:?}");
    places.to_vec()
}

/// What `fuse/arrange.rs`'s `gradient_at` makes of a ball: the place minus the
/// centre, taken from the place's box.
fn gradient_of_the_ball(place: PredicatePoint, centre_x: f64) -> PredicatePoint {
    let [x, y, z] = place.enclosure();
    PredicatePoint::Constructed([x - Interval::exactly(centre_x), y, z])
}

/// `turns_anticlockwise`'s determinant, with the rows in the order the fuse's
/// record has them.
fn the_turn_at(gradient: PredicatePoint) -> Decision {
    orient3d_of_points(
        gradient,
        PredicatePoint::from([0.0, 0.0, 1.0]),
        PredicatePoint::from([0.0, 1.0, 0.0]),
        PredicatePoint::from([0.0; 3]),
    )
}

/// The one `f64` triple a box holds.
///
/// # Panics
///
/// If any side of the box is wider than a point.
fn collapsed(place: PredicatePoint) -> [f64; 3] {
    place.enclosure().map(|side| {
        assert_eq!(side.lo(), side.hi(), "{place:?} is not one point");
        side.lo()
    })
}
