//! Two places a block edge cuts a ball, `2⁻⁵¹` either side of one point, whose
//! boxes touch.
//!
//! `place_the_meets` used to throw a new meet away whenever a node on both its
//! crossings had a box touching its own. Three surfaces that meet twice meet at
//! two *different* points, and the other meet of the same call is such a node —
//! so where the two boxes touched, two points became one and no policy was asked
//! (`docs/decisions.md` §244.3). §17.4 says overlapping boxes are the policy's
//! to merge or refuse, and `merge_points_that_are_one_point` is where it is
//! asked. The fix keeps the second meet, and that is where the pair goes now.
//!
//! # The shape
//!
//! §244.2's numbers, turned so the edge runs along `z`: `(2m)² + (2m²)² + 1 =
//! (2m² + 1)²` with `m = 2²⁵`, scaled by `2⁻⁵¹`, makes `1 + (2⁻²⁵)² + 2⁻¹⁰²`
//! exactly `(1 + 2⁻⁵¹)²`. A ball of radius `1 + 2⁻⁵¹` centred on the `z` axis
//! at height `h` and a block whose edge is the line `x = −1, y = 2⁻²⁵` therefore
//! meet where that edge meets the ball, at `z = h ± 2⁻⁵¹`, `8.9e-16` apart.
//!
//! The edge runs along `z` rather than §244.2's `x` for the ball's sake. A ball
//! here is one face whose seam is the half of `y = 0` with `x ≥ 0`, and an edge
//! along `x` grazes it at `x = 0`, on the plane that seam is picked out by,
//! where `Arrangement::turns_anticlockwise` refuses on the kept meet whether or
//! not the other one is folded into it (§248.3, §317.1). An edge along `z` at
//! `x = −1` grazes it at the equator, a whole unit from the seam, and is kept
//! because order along a crossing refuses first there, before that turn is ever
//! asked (§317.5).
//!
//! At `h = 0` both places are exact, `z = ±2⁻⁵¹`, and their boxes do not
//! touch. At `h = 16` the last place of `16` is `2⁻⁴⁸` wide, so neither place is
//! an `f64`: their boxes are `[15.999999999999998, 16]` and
//! `[16, 16.000000000000004]`, and they share the point `16`.

use rcad_geom::{Isometry, Point, Scale, Vector};
use rcad_predicates::{Decision, Level, Point as PredicatePoint, Policy, ProvenanceLog, Record};
use rcad_topo::{Containment, Error, Fused, Model, ProfileBuilder, Rule};

/// The name `order_along` records a refusal under.
const ALONG_A_CROSSING: &str = "order along a crossing";

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The regression: under a certified policy the two touching meets are what
/// the fuse refuses on, by name, and nothing else is.
///
/// Without the fix the second meet was dropped, and the one that was kept went
/// on to a later refusal with no second point in it.
#[test]
fn two_meets_whose_boxes_touch_are_what_a_certified_fuse_refuses_on() {
    for rule in EVERY_RULE {
        let records = what_it_recorded(16.0, rule, &Policy::certified());
        assert_eq!(
            records.len(),
            1,
            "{rule:?}: a certified refusal carries the one decision it could not take, \
             and this carried {records:#?}"
        );
        let refused = &records[0];
        assert_eq!(
            refused.predicate(),
            ALONG_A_CROSSING,
            "{rule:?}: {refused:#?}"
        );
        assert_eq!(refused.level(), Level::Interval, "{rule:?}");
        assert!(
            is_the_two_meets(refused),
            "{rule:?}: the pair it refused on is not the two meets: {refused:#?}"
        );
    }
}

/// And a tolerant policy is asked the same question about the same pair, and
/// folds it.
///
/// The fuse still refuses in the end, and for a reason this file is not about:
/// the plane `x = −1` cuts the ball in a circle of radius `2⁻²⁵`, and no point
/// can be placed inside that piece. It refuses the same way at `h = 0`, where
/// there is no pair to fold — see the test below.
#[test]
fn a_tolerant_policy_asked_about_the_same_two_meets_folds_them() {
    for rule in EVERY_RULE {
        let records = what_it_recorded(16.0, rule, &Policy::tolerant(25.0));
        let folded = records
            .iter()
            .filter(|record| record.predicate() == ALONG_A_CROSSING && is_the_two_meets(record))
            .collect::<Vec<_>>();
        assert!(
            !folded.is_empty(),
            "{rule:?}: the policy was never asked about the two meets: {records:#?}"
        );
        for record in folded {
            assert_eq!(
                record.outcome(),
                Decision::Zero,
                "{rule:?}: a wobble of 5.3e-15 at a 25 mm working size is inside the budget, \
                 so the two are one point: {record:#?}"
            );
        }
    }
}

/// The control: where the two meets are exact and apart, nothing asks whether
/// they are one, under either policy.
///
/// What the fuse refuses on at `h = 0` is the tiny piece, the same as the
/// tolerant fuse at `h = 16` — which is what says that refusal is not the fold.
/// That piece holds no point with `f64` coordinates, which the next test
/// shows, so `select` cannot place it (`docs/decisions.md` §346).
#[test]
fn where_the_two_meets_are_exact_nothing_asks_whether_they_are_one() {
    for policy in [Policy::certified(), Policy::tolerant(25.0)] {
        for rule in EVERY_RULE {
            let records = what_it_recorded(0.0, rule, &policy);
            assert!(
                records
                    .iter()
                    .all(|record| record.predicate() != ALONG_A_CROSSING),
                "{rule:?}: {records:#?}"
            );
            assert!(
                records
                    .iter()
                    .any(|record| record.predicate() == "a point inside a piece of a face"),
                "{rule:?}: {records:#?}"
            );
        }
    }
}

/// No point with `f64` coordinates is inside any of the three pieces the two
/// places fence off at `h = 0`, so no point `select` builds is (§346).
///
/// The three are the ball's skin inside the block, the block's face `x = −1`
/// inside the ball, and its face `y = 2⁻²⁵` inside the ball. A point inside
/// any of them has `x ≤ −1` and `y ≥ 2⁻²⁵`, with one of the two strict, and is
/// on the ball or in it. The doubles nearest past those two planes are
/// `−1 − 2⁻⁵²` and `2⁻²⁵ + 2⁻⁷⁷`, and with the other two coordinates as near
/// the ball's centre as they may be, each is already outside the ball. Every
/// other such point is further from the centre still. The same holds for a
/// box: one proved past a plane has every point past the nearest double.
#[test]
fn no_point_with_f64_coordinates_is_inside_the_piece_between_the_two_places() {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let ball = model
        .add_ball(1.0 + f64::powi(2.0, -51), &Isometry::identity())
        .expect("a positive radius");
    let ball = model.solid(ball).expect("this model made the key");
    let edge = f64::powi(2.0, -25);
    let place = f64::powi(2.0, -51);
    let where_it_is = |at: [f64; 3]| {
        ball.classify_point(
            Point::new(at[0], at[1], at[2]),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
    };
    for corner in [[-1.0, edge, -place], [-1.0, edge, place]] {
        assert_eq!(
            where_it_is(corner),
            Containment::OnBoundary,
            "{corner:?} is one of the two places"
        );
    }
    assert_eq!(f64::next_down(-1.0), -1.0 - f64::EPSILON);
    assert_eq!(f64::next_up(edge), edge + f64::powi(2.0, -77));
    for nearest in [
        [f64::next_down(-1.0), edge, 0.0],
        [-1.0, f64::next_up(edge), 0.0],
    ] {
        assert_eq!(
            where_it_is(nearest),
            Containment::Outside,
            "{nearest:?} is the nearest a double gets to the pieces"
        );
    }
}

/// Every record the fuse wrote, whether it answered or refused.
///
/// # Panics
///
/// On any error but a refusal: this shape is a ball and a block, and anything
/// else coming back is not what this file is about.
fn what_it_recorded(height: f64, rule: Rule, policy: &Policy) -> Vec<Record> {
    let mut log = ProvenanceLog::new();
    match fuse_a_block_edge_grazing_a_ball(height, rule, policy, &mut log) {
        Ok(_) => log.records().to_vec(),
        Err(Error::Undecidable { records }) => records.records().to_vec(),
        Err(other) => panic!("{rule:?} at height {height}: {other:?}"),
    }
}

/// Whether a record's two inputs are the two places the edge cuts the ball at
/// `h = 16`, in either order.
fn is_the_two_meets(record: &Record) -> bool {
    let heights: Vec<[f64; 2]> = record
        .inputs()
        .iter()
        .map(|point| match point {
            PredicatePoint::Constructed(sides) => {
                assert_eq!(
                    [sides[0].lo(), sides[0].hi(), sides[1].lo(), sides[1].hi()],
                    [-1.0, -1.0, f64::powi(2.0, -25), f64::powi(2.0, -25)],
                    "both meets are on the edge x = -1, y = 2^-25, exactly"
                );
                [sides[2].lo(), sides[2].hi()]
            }
            PredicatePoint::Exact(at) => [at[2], at[2]],
            other => panic!("a kind of point this file does not know: {other:?}"),
        })
        .collect();
    let below = [15.999_999_999_999_998, 16.0];
    let above = [16.0, 16.000_000_000_000_004];
    heights == [below, above] || heights == [above, below]
}

fn fuse_a_block_edge_grazing_a_ball(
    height: f64,
    rule: Rule,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> rcad_topo::Result<Fused> {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let ball = model
        .add_ball(
            1.0 + f64::powi(2.0, -51),
            &Isometry::translation(Vector::new(0.0, 0.0, height)).expect("a finite shift"),
        )
        .expect("a positive radius");
    let edge = f64::powi(2.0, -25);
    let square = ProfileBuilder::new(vec![
        [-4.0, edge],
        [-1.0, edge],
        [-1.0, edge + 3.0],
        [-4.0, edge + 3.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    let block = model
        .add_extrusion(
            &square,
            4.0,
            &Isometry::translation(Vector::new(0.0, 0.0, height - 2.0)).expect("a finite shift"),
        )
        .expect("a positive height");
    model.fuse(&[ball], &[block], rule, policy, log)
}
