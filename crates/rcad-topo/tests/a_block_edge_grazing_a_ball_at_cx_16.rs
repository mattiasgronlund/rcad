//! §244.3's block edge along `x`, grazing a ball centred at `x = 16`: what a
//! certified fuse refuses on, and the sign of the turn it used to refuse on.
//!
//! # The shape
//!
//! A ball of radius `1 + 2⁻⁵¹` centred at `(16, 0, 0)`, and a block whose top
//! is `z = −2⁻²⁵` and whose side is `y = 1`. That edge meets the ball at
//! `x = 16 ± 2⁻⁵¹`, which are not `f64`s: the two places come back as the boxes
//! `[15.999999999999998, 16]` and `[16, 16.000000000000004]`.
//!
//! # What refuses
//!
//! The walked start of both the top's circle and the side's is `(16, 1, −2⁻²⁵)`,
//! on the edge between the two places and off both circles by a rounding. All
//! three points lie on one line along `x`, so going round either circle from
//! that start the two places sweep exactly zero, and their boxes touch at `16`.
//! A certified fuse refuses on which of the two comes first, which is the
//! refusal §244.2 found where the two places are exact at `x = 0`.
//!
//! Until §244.5 that pair was folded into one node without a policy being
//! asked, and the fuse went on to refuse on the turn below.
//!
//! # The turn it used to refuse on, and why that is a gap
//!
//! Splitting the ball's face, `Arrangement::turns_anticlockwise` in
//! `fuse/arrange.rs` asks which way two crossings turn at a place they meet:
//! the sign of the determinant of the ball's gradient there and the two
//! planes' normals. The gradient is the place minus the ball's centre, taken
//! from the box, so its `x` is `[−2⁻⁴⁹, 0]` and the sign cannot be told. The
//! seam of the ball is not involved; `docs/decisions.md` §248.3 guessed that it
//! was.
//!
//! The sign exists. The determinant is minus the gradient's `x`, which is
//! `∓2⁻⁵¹`, never zero. And the same three surfaces solved about the ball's
//! own centre meet at two `f64` triples, where the same determinant is decided.
//! So this is a gap and not a limit (§16.10). `docs/decisions.md` §317 has the
//! measurement and the question a fix should ask. A certified fuse of this
//! shape refuses on the order of the two places before it asks the turn, so
//! only the measurement below shows the gap here.

use rcad_geom::{
    Isometry, MeetOfThree, Plane, Point, Scale, Sphere, Surface, Vector, meet_of_three_surfaces,
};
use rcad_predicates::{
    Decision, Interval, Level, Point as PredicatePoint, Policy, ProvenanceLog, Record,
    orient3d_of_points,
};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule};

/// The name `order_along` records a refusal under.
const ALONG_A_CROSSING: &str = "order along a crossing";

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The pin: under a certified policy every rule refuses on which of the two
/// places comes first round a circle, and on nothing else.
///
/// Before §244.5 the two places were folded into one node here, because the
/// sweep over them is exactly zero and their boxes touch, and every rule went
/// on to refuse on the turn at the place below `x = 16`.
#[test]
fn a_certified_fuse_refuses_on_the_order_of_the_two_places() {
    let places = where_the_edge_meets_a_ball_centred_at(16.0);
    for rule in EVERY_RULE {
        let mut log = ProvenanceLog::new();
        let refused = match fuse_a_block_edge_grazing_a_ball(rule, &mut log) {
            Err(Error::Undecidable { records }) => records.records().to_vec(),
            other => panic!("{rule:?}: {other:?}"),
        };
        assert_eq!(refused.len(), 1, "{rule:?}: {refused:#?}");
        assert!(
            is_the_order_of_the_two_places(&refused[0], &places),
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

/// Whether a record is `order_along`'s about the two places, in either order,
/// as wide as the last place of `16`, `2⁻⁴⁸`, which is how wide either box is.
fn is_the_order_of_the_two_places(record: &Record, places: &[PredicatePoint]) -> bool {
    let inputs = record.inputs();
    record.predicate() == ALONG_A_CROSSING
        && record.level() == Level::Interval
        && (inputs == places || inputs == [places[1], places[0]])
        && record.enclosure() == f64::powi(2.0, -48)
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
