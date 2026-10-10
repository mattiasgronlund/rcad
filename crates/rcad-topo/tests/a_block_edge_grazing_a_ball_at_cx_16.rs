//! §244.3's block edge along `x`, grazing a ball centred at `x = 16`: the three
//! questions a fuse of the two used to refuse on, each now decided, and what it
//! refuses on after them.
//!
//! # The shape
//!
//! A ball of radius `1 + 2⁻⁵¹` centred at `(16, 0, 0)`, and a block whose top
//! is `z = −2⁻²⁵` and whose side is `y = 1`. That edge meets the ball at
//! `x = 16 ± 2⁻⁵¹`, which are not `f64`s: the two places come back as the boxes
//! `[15.999999999999998, 16]` and `[16, 16.000000000000004]`.
//!
//! # The order of the two places round each circle is decided
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
//! # The turn at each place is decided
//!
//! Splitting the ball's face, `Arrangement::turns_anticlockwise` asks which way
//! two crossings turn at a place they meet: the sign of the determinant of the
//! ball's gradient there and the two planes' normals. Over the place's box the
//! gradient's `x` is `[−2⁻⁴⁹, 0]` and the sign cannot be told. It is `∓2⁻⁵¹`,
//! never zero (the second test below), and since Q914 it is read off which of
//! the edge's two meets with the ball the place is, by two exact signs of the
//! ball along the edge (`fuse/arrange.rs`,
//! `the_turn_on_a_balls_face_where_a_block_edge_grazes_it_is_read_off_the_root`).
//! Until then every certified fuse with the ball first refused on it, the gap
//! `docs/decisions.md` §317 measured.
//!
//! # The order of the two places along the edge is decided
//!
//! On the block's top and side the two places lie on the edge, in boxes that
//! share only `16` along it, so read as boxes they are apart by a range from
//! nothing up. Until Q914 every certified fuse with the block first refused on
//! that. The edge's point at `16` is `2⁻¹⁰³` inside the ball, so neither place
//! is at `16`, and the one whose box ends there comes first along the edge's
//! `x` (`Arrangement::as_the_quadric_says`).
//!
//! # What it refuses on next
//!
//! Every fuse, with either solid first and under either policy, then refuses on
//! a point inside a piece of a face between the two places, with those two
//! places as the record's inputs. No fuse answers, so none is held to a volume.
//! Why §365.3's rule, which places the slivers between the same two places on
//! the edge along `z` at `h = 0` in `a_block_edge_grazing_a_ball.rs`, does not
//! place these is not traced (§317.7).

use rcad_geom::{
    Isometry, MeetOfThree, Plane, Point, Scale, Sphere, Surface, Vector, meet_of_three_surfaces,
};
use rcad_predicates::{
    Decision, Interval, Level, Point as PredicatePoint, Policy, ProvenanceLog, orient3d_of_points,
};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule};

/// The name a point search records a refusal under when nothing places a
/// piece of a face.
const A_POINT_INSIDE_A_PIECE: &str = "a point inside a piece of a face";

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The pin: every fuse gets past the turn and both orders, and refuses on a
/// point inside a piece between the two places, and on nothing else.
///
/// Until Q914 a certified fuse refused on the turn at the place below `16`
/// with the ball first, and on the order of the two places along the edge
/// with the block first; under `Policy::tolerant(25.0)` every one was
/// `ResultIsNotManifold { count: 3 }`.
#[test]
fn every_fuse_refuses_on_a_point_inside_a_piece_between_the_two_places() {
    let places = where_the_edge_meets_a_ball_centred_at(16.0);
    for policy in [Policy::certified(), Policy::tolerant(25.0)] {
        for ball_first in [true, false] {
            for rule in EVERY_RULE {
                let case = (policy.strictness(), ball_first, rule);
                let mut log = ProvenanceLog::new();
                let refused =
                    match fuse_a_block_edge_grazing_a_ball(rule, &policy, ball_first, &mut log) {
                        Err(Error::Undecidable { records }) => records.records().to_vec(),
                        other => panic!("{case:?}: {other:?}"),
                    };
                let [refused] = &refused[..] else {
                    panic!("{case:?}: one record, not {refused:#?}");
                };
                assert_eq!(
                    refused.predicate(),
                    A_POINT_INSIDE_A_PIECE,
                    "{case:?}: {refused:#?}"
                );
                assert_eq!(refused.level(), Level::Interval, "{case:?}");
                let named = refused.inputs();
                assert!(
                    named == [places[0], places[1]] || named == [places[1], places[0]],
                    "{case:?}: the piece is not named by the two places: {refused:#?}"
                );
            }
        }
    }
}

#[test]
fn probe_what_each_fuse_gives() {
    for policy in [Policy::certified(), Policy::tolerant(25.0)] {
        for ball_first in [true, false] {
            for rule in EVERY_RULE {
                let case = (policy.strictness(), ball_first, rule);
                let mut log = ProvenanceLog::new();
                match fuse_a_block_edge_grazing_a_ball(rule, &policy, ball_first, &mut log) {
                    Ok(fused) => println!("PROBE {case:?}: Ok {} solids", fused.solids().len()),
                    Err(Error::Undecidable { records }) => println!(
                        "PROBE {case:?}: Undecidable {:?}",
                        records
                            .records()
                            .iter()
                            .map(|record| (record.predicate(), record.inputs().len()))
                            .collect::<Vec<_>>()
                    ),
                    Err(other) => println!("PROBE {case:?}: {other:?}"),
                }
            }
        }
    }
}

/// The measurement that made the turn a gap: the sign it needs exists, and
/// the same three surfaces solved about the ball's own centre reach it.
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
    policy: &Policy,
    ball_first: bool,
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
    if ball_first {
        model.fuse(&[ball], &[block], rule, policy, log)
    } else {
        model.fuse(&[block], &[ball], rule, policy, log)
    }
}

/// The block's top, `z = −2⁻²⁵`, and its side, `y = 1`, facing the way the
/// fuse's record of the turn had them.
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
/// record of it had them.
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
