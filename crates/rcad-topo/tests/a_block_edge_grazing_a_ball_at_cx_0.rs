//! §244.2's block edge along `x`, grazing a ball centred at the origin: the
//! order of the two places it cuts the ball at is decided now, the sliver
//! between them holds no double, and a tolerant fuse answers.
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
//! # What it gives now
//!
//! With the two places in order, the fuse goes on to place each piece of a
//! face, and the slivers between the two places hold no point with `f64`
//! coordinates. That is §346's limit, the same one the edge along `z` meets at
//! `h = 0` in `a_block_edge_grazing_a_ball.rs`, and every fuse refused on it
//! until §365.3's rule placed those slivers by the sides they share (Q712).
//! Every fuse now gets past `select`. Under `Policy::certified()` each is
//! refused for the range. Under `Policy::tolerant(25.0)` each answers one
//! solid that validates and holds the volume it should. Until Q2238 each was
//! refused `CannotMeshAFace` with `CornersCoincide`: a face of the answer
//! could not be cut into triangles because two of its corners are one. Until
//! Q1837 each was refused earlier, while its answer was written, with
//! `CannotNameWhichPieceOfACrossingThisIs` out of `rcad-geom`.

use rcad_geom::{DYNAMIC_RANGE_CHECK, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Fused, Invalid, Model, ProfileBuilder, Rule};

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The pin: under either policy, every rule with either solid first gets past
/// placing the pieces, and records nothing about the order along a crossing.
/// Under `Policy::certified()` each is refused for the range,
/// `DynamicRangeTooWide` at `2⁵¹ + 2²⁵`, the block's bottom corners at
/// `z = −2 − 2⁻²⁵` over the two places' `2⁻⁵⁰`, and records nothing at all.
/// Under `Policy::tolerant(25.0)` each answers one solid that validates and
/// whose volume, measured at a deflection of `10⁻⁴`, is within its error of
/// [`the_volume_it_should_have`], and records only the `dynamic range check`
/// that the certified six are refused on.
///
/// **Q2238 moved the tolerant six.** Until it, each was refused
/// `CannotMeshAFace` with `CornersCoincide` at position 2, or 3 for the
/// difference with the ball first, with nothing recorded.
///
/// **Q1837 moved them before that.** Until it, each was refused
/// `CannotNameWhichPieceOfACrossingThisIs` by the walk, with one `side of a
/// half-space` record: one arc of a circle had a middle no half-space could
/// tell, and the walk refused though the other arc's middle was told. The walk
/// now keeps the arc that is told and records nothing.
///
/// **Q1614 moved the certified six.** Since Q1614 the fuse asks the range
/// before it nests the answer's shells (`docs/decisions.md` §87.3). Until then
/// the certified six were refused as the tolerant six then were, record and
/// all.
///
/// Here the two places fence off two slivers, not three: the ball's skin past
/// `y = 1` below the top, which is outside the block, and the block's side
/// inside the ball. The block's top inside the ball is the whole disk less a
/// notch, which holds doubles. A print of each placement, taken out, showed
/// both slivers placed by their sides in all twelve fuses, the ball's outside
/// the block and the block's inside the ball, and `select` finishing.
///
/// Along `z` at `h = 0` the six tolerant fuses are refused
/// `ACurvedFaceGoesRightRoundWithNoSeam` instead, with nothing recorded. That
/// refusal has not been traced.
///
/// Before Q449 a certified fuse refused on which of the two places comes
/// first round a circle, and a tolerant one with `CannotTellTheCellsOfAFaceApart`.
/// From Q449 to Q712 every fuse refused `a point inside a piece of a face` on
/// the two places, and with §365.3's rule switched off by hand they still do.
#[test]
fn each_fuse_is_refused_for_the_range_when_certified_and_answers_with_its_volume_when_tolerant() {
    for policy in [Policy::certified(), Policy::tolerant(25.0)] {
        let certified = policy == Policy::certified();
        for rule in EVERY_RULE {
            for ball_first in [true, false] {
                let case = format!("{policy:?} {rule:?}, ball first {ball_first}");
                let mut log = ProvenanceLog::new();
                let (model, answer) =
                    fuse_a_block_edge_grazing_a_ball(rule, ball_first, &policy, &mut log);
                match answer {
                    Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) if certified => {
                        assert_eq!(
                            ratio,
                            (2.0 + f64::powi(2.0, -25)) * f64::powi(2.0, 50),
                            "{case}"
                        );
                        let records = log.records();
                        assert!(records.is_empty(), "{case}: {records:#?}");
                    }
                    Ok(fused) if !certified => {
                        let &[solid] = fused.solids() else {
                            panic!("{case}: {} solids", fused.solids().len());
                        };
                        model
                            .validate(&policy, &mut ProvenanceLog::new())
                            .unwrap_or_else(|why| panic!("{case}: {why:?}"));
                        let measured = model
                            .solid(solid)
                            .expect("the fuse made the key")
                            .measure(
                                Deflection::of(0.0001).expect("a length"),
                                &policy,
                                &mut ProvenanceLog::new(),
                            )
                            .unwrap_or_else(|why| panic!("{case}: {why:?}"));
                        let closed_form = the_volume_it_should_have(rule, ball_first);
                        assert!(
                            (measured.volume() - closed_form).abs() <= measured.volume_error(),
                            "{case}: measured {} ± {}, and the closed form is {closed_form}",
                            measured.volume(),
                            measured.volume_error()
                        );
                        let records = log.records();
                        assert!(
                            records.len() == 1 && records[0].predicate() == DYNAMIC_RANGE_CHECK,
                            "{case}: {records:#?}"
                        );
                    }
                    other => panic!("{case}: {other:?}"),
                }
            }
        }
    }
}

/// The volume of the answer. The ball holds `4/3 πR³`, with `R = 1 + 2⁻⁵¹`,
/// and the block `4 · 3 · 2`. What the two share is the ball's cap below the
/// block's top `z = −2⁻²⁵`, of height `a = R − 2⁻²⁵`, `πa²(3R − a)/3`: the
/// ball's skin past `y = 1` lies above that top but for the sliver, which
/// holds no double.
fn the_volume_it_should_have(rule: Rule, ball_first: bool) -> f64 {
    let radius = 1.0 + f64::powi(2.0, -51);
    let ball = 4.0 / 3.0 * core::f64::consts::PI * radius.powi(3);
    let block = 4.0 * 3.0 * 2.0;
    let height = radius - f64::powi(2.0, -25);
    let both = core::f64::consts::PI * height * height * (3.0 * radius - height) / 3.0;
    match (rule, ball_first) {
        (Rule::Union, _) => ball + block - both,
        (Rule::Intersection, _) => both,
        (Rule::Difference, true) => ball - both,
        (Rule::Difference, false) => block - both,
        (other, _) => panic!("no closed form for {other:?}"),
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
) -> (Model, rcad_topo::Result<Fused>) {
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
    let answer = if ball_first {
        model.fuse(&[ball], &[block], rule, policy, log)
    } else {
        model.fuse(&[block], &[ball], rule, policy, log)
    };
    (model, answer)
}
