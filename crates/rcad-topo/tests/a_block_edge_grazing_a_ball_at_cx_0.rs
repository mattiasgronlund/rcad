//! §244.2's block edge along `x`, grazing a ball centred at the origin: the
//! order of the two places it cuts the ball at is decided now, and what the
//! fuse refuses on instead is a sliver no double lies in.
//!
//! # The shape
//!
//! A ball of radius `1 + 2⁻⁵¹` centred at the origin, and a block whose top
//! is `z = −2⁻²⁵` and whose side is `y = 1`. That edge cuts the ball at
//! `x = ±2⁻⁵¹`, and at `x = 0` both places are exact: `(±2⁻⁵¹, 1, −2⁻²⁵)`.
//! The numbers make `1 + (2⁻²⁵)² + 2⁻¹⁰²` exactly `(1 + 2⁻⁵¹)²`.
//!
//! # What used to refuse
//!
//! Both the top's circle and the side's start at `(0, 1, −2⁻²⁵)`, on the edge,
//! halfway between the two places and `2⁻¹⁰³` inside the ball. The start and
//! the two places are in one line, so they sweep exactly nothing, and neither
//! place's box holds the start. `the_one_the_start_is` in `fuse/arrange.rs`
//! answered that `Undecidable`, and every rule refused on `order along a
//! crossing` under any policy. Its doc comment said a start off the conic
//! would get there, and §244.2 called the refusal the right answer.
//!
//! It was not: the start lies between the two places along the circle, and
//! the one ahead of it along the way the circle runs comes first (Q449). The
//! unit test `a_start_inside_the_ball_between_two_places_puts_the_one_ahead_of_it_first`
//! checks which one against the sweep to a third point.
//!
//! # What refuses now
//!
//! With the two places in order, the fuse goes on to place each piece of a
//! face, and the slivers between the two places hold no point with `f64`
//! coordinates. That is §346's limit, the same one the edge along `z` meets at
//! `h = 0` in `a_block_edge_grazing_a_ball.rs`, and every fuse refused on it
//! until §365.3's rule placed those slivers by the sides they share (Q712).
//! Every fuse now gets past `select` and is refused while its answer is
//! written, with `CannotNameWhichPieceOfACrossingThisIs` out of `rcad-geom`.

use rcad_geom::{Error as GeometryError, Isometry, Point, Scale, Vector};
use rcad_predicates::{Level, Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Fused, Invalid, Model, ProfileBuilder, Rule};

/// The name `order_along` records a refusal under.
const ALONG_A_CROSSING: &str = "order along a crossing";

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The pin: under either policy, every rule with either solid first gets past
/// placing the pieces, and nothing is recorded about the order along a
/// crossing. Under `Policy::tolerant(25.0)` each is refused
/// `CannotNameWhichPieceOfACrossingThisIs`, with one `side of a half-space`
/// record. Under `Policy::certified()` each is refused for the range,
/// `DynamicRangeTooWide` at `2⁵¹`, the two places' `2⁻⁵⁰` beside the block's
/// corners at 2, with no record at all.
///
/// **Q1614 moved the certified six.** The tolerant refusal and its record both
/// come out of nesting the answer's shells, and since Q1614 the fuse asks the
/// range before that (`docs/decisions.md` §87.3). Until then the certified six
/// were refused as the tolerant six are, record and all.
///
/// Here the two places fence off two slivers, not three: the ball's skin past
/// `y = 1` below the top, which is outside the block, and the block's side
/// inside the ball. The block's top inside the ball is the whole disk less a
/// notch, which holds doubles. A print of each placement, taken out, showed
/// both slivers placed by their sides in all twelve fuses, the ball's outside
/// the block and the block's inside the ball, and `select` finishing.
///
/// The tolerant intersection and block less the ball are refused the same way
/// here, where along `z` at `h = 0` they are refused
/// `ACurvedFaceGoesRightRoundWithNoSeam` instead. Neither refusal has been
/// traced.
///
/// Before Q449 a certified fuse refused on which of the two places comes
/// first round a circle, and a tolerant one with `CannotTellTheCellsOfAFaceApart`.
/// From Q449 to Q712 every fuse refused `a point inside a piece of a face` on
/// the two places, and with §365.3's rule switched off by hand they still do.
#[test]
fn the_order_of_the_two_places_is_decided_and_the_slivers_between_them_are_placed_by_their_sides() {
    for policy in [Policy::certified(), Policy::tolerant(25.0)] {
        let certified = policy == Policy::certified();
        for rule in EVERY_RULE {
            for ball_first in [true, false] {
                let case = format!("{policy:?} {rule:?}, ball first {ball_first}");
                let mut log = ProvenanceLog::new();
                match fuse_a_block_edge_grazing_a_ball(rule, ball_first, &policy, &mut log) {
                    Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) if certified => {
                        assert_eq!(ratio, f64::powi(2.0, 51), "{case}");
                    }
                    Err(Error::Geometry(GeometryError::CannotNameWhichPieceOfACrossingThisIs))
                        if !certified => {}
                    Ok(fused) => panic!(
                        "{case}: the fuse answered with {} solids; check each against its \
                         volume and pin it",
                        fused.solids().len()
                    ),
                    Err(other) => panic!("{case}: {other:?}"),
                }
                let records = log.records();
                assert!(
                    records
                        .iter()
                        .all(|record| record.predicate() != ALONG_A_CROSSING),
                    "{case}: {records:#?}"
                );
                let expected: &[(&str, Level)] = if certified {
                    &[]
                } else {
                    &[("side of a half-space", Level::Interval)]
                };
                assert_eq!(
                    records
                        .iter()
                        .map(|record| (record.predicate(), record.level()))
                        .collect::<Vec<_>>(),
                    expected,
                    "{case}: {records:#?}"
                );
            }
        }
    }
}

/// No point with `f64` coordinates is inside the sliver the two places fence
/// off, so no point the fuse builds is (§346).
///
/// A point of the sliver, on the ball, on the block's top or on its side, has
/// `y ≥ 1` and `z ≤ −2⁻²⁵`, with one of the two strict, and is on the ball or
/// in it. The doubles nearest past those two planes are `1 + 2⁻⁵²` and
/// `−2⁻²⁵ − 2⁻⁷⁷`, and with `x` at the ball's centre and the other coordinate
/// on its plane, each is already outside the ball. Every other such point is
/// further from the centre still.
#[test]
fn no_point_with_f64_coordinates_is_inside_the_sliver_between_the_two_places() {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let ball = model
        .add_ball(1.0 + f64::powi(2.0, -51), &Isometry::identity())
        .expect("a positive radius");
    let ball = model.solid(ball).expect("this model made the key");
    let top = -f64::powi(2.0, -25);
    let place = f64::powi(2.0, -51);
    let where_it_is = |at: [f64; 3]| {
        ball.classify_point(
            Point::new(at[0], at[1], at[2]),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
    };
    for corner in [[-place, 1.0, top], [place, 1.0, top]] {
        assert_eq!(
            where_it_is(corner),
            Containment::OnBoundary,
            "{corner:?} is one of the two places"
        );
    }
    assert_eq!(
        where_it_is([0.0, 1.0, top]),
        Containment::Inside,
        "the walked start is inside the ball"
    );
    assert_eq!(f64::next_up(1.0), 1.0 + f64::EPSILON);
    assert_eq!(f64::next_down(top), top - f64::powi(2.0, -77));
    for nearest in [
        [0.0, f64::next_up(1.0), top],
        [0.0, 1.0, f64::next_down(top)],
    ] {
        assert_eq!(
            where_it_is(nearest),
            Containment::Outside,
            "{nearest:?} is the nearest a double gets to the sliver"
        );
    }
}

fn fuse_a_block_edge_grazing_a_ball(
    rule: Rule,
    ball_first: bool,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> rcad_topo::Result<Fused> {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let ball = model
        .add_ball(1.0 + f64::powi(2.0, -51), &Isometry::identity())
        .expect("a positive radius");
    let square = ProfileBuilder::new(vec![[-2.0, -2.0], [2.0, -2.0], [2.0, 1.0], [-2.0, 1.0]])
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
