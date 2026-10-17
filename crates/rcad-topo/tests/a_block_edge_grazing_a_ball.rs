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
//! The edge runs along `z` rather than §244.2's `x` so that the pair is what a
//! certified fuse refuses on. §248.3 chose it to keep the edge off the ball's
//! seam; the edge along `x` refused on the turn at the kept meet instead,
//! `Arrangement::turns_anticlockwise` over a box ending at the place (§317.1),
//! and since Q914 it refuses on neither the turn nor the pair
//! (`a_block_edge_grazing_a_ball_at_cx_16.rs`). An edge along `z` at `x = −1`
//! grazes the ball at the equator, and a certified fuse refuses on the order of
//! the two meets round the circles the block's two faces cut the ball in, with
//! either solid first, before any turn there is asked. Q914 decides the order
//! of two such meets along the edge itself, and not round a circle (§317.7).
//!
//! At `h = 0` both places are exact, `z = ±2⁻⁵¹`, and their boxes do not
//! touch. At `h = 16` the last place of `16` is `2⁻⁴⁸` wide, so neither place is
//! an `f64`: their boxes are `[15.999999999999998, 16]` and
//! `[16, 16.000000000000004]`, and they share the point `16`.

use rcad_geom::{Isometry, Point, Scale, Vector};
use rcad_predicates::{Decision, Level, Point as PredicatePoint, Policy, ProvenanceLog, Record};
use rcad_topo::{Containment, Error, Fused, Invalid, Model, ProfileBuilder, Rule};

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
        for ball_first in [true, false] {
            let records = what_it_recorded(16.0, rule, &Policy::certified(), ball_first);
            assert_eq!(
                records.len(),
                1,
                "{rule:?}, ball first {ball_first}: a certified refusal carries the one \
                 decision it could not take, and this carried {records:#?}"
            );
            let refused = &records[0];
            assert_eq!(
                refused.predicate(),
                ALONG_A_CROSSING,
                "{rule:?}, ball first {ball_first}: {refused:#?}"
            );
            assert_eq!(
                refused.level(),
                Level::Interval,
                "{rule:?}, ball first {ball_first}"
            );
            assert!(
                is_the_two_meets(refused),
                "{rule:?}, ball first {ball_first}: the pair it refused on is not the two \
                 meets: {refused:#?}"
            );
        }
    }
}

/// And a tolerant policy is asked the same question about the same pair, and
/// folds it.
///
/// The fuse still refuses in the end, and for a reason this file is not about:
/// the plane `x = −1` cuts the ball in a circle of radius `2⁻²⁵`, and a point
/// found inside the piece the fold leaves at the one point could not be placed
/// against the other solid (`a point in a piece that every solid could
/// place`). At `h = 0`, where there is no pair to fold, the pieces between the
/// two places hold no point at all and are placed by their sides — see the
/// tests below.
#[test]
fn a_tolerant_policy_asked_about_the_same_two_meets_folds_them() {
    for rule in EVERY_RULE {
        for ball_first in [true, false] {
            let records = what_it_recorded(16.0, rule, &Policy::tolerant(25.0), ball_first);
            let folded = records
                .iter()
                .filter(|record| record.predicate() == ALONG_A_CROSSING && is_the_two_meets(record))
                .collect::<Vec<_>>();
            assert!(
                !folded.is_empty(),
                "{rule:?}, ball first {ball_first}: the policy was never asked about the two \
                 meets: {records:#?}"
            );
            for record in folded {
                assert_eq!(
                    record.outcome(),
                    Decision::Zero,
                    "{rule:?}, ball first {ball_first}: a wobble of 5.3e-15 at a 25 mm \
                     working size is inside the budget, so the two are one point: {record:#?}"
                );
            }
        }
    }
}

/// The control: where the two meets are exact and apart, nothing asks whether
/// they are one, under either policy, in either order.
#[test]
fn where_the_two_meets_are_exact_nothing_asks_whether_they_are_one() {
    for policy in [Policy::certified(), Policy::tolerant(25.0)] {
        for rule in EVERY_RULE {
            for ball_first in [true, false] {
                let (_, records) = fused_and_recorded(rule, &policy, ball_first);
                assert!(
                    records
                        .iter()
                        .all(|record| record.predicate() != ALONG_A_CROSSING),
                    "{rule:?}, ball first {ball_first}: {records:#?}"
                );
            }
        }
    }
}

/// At `h = 0` every fuse gets past `select`, and none of the six answers under
/// either policy.
///
/// The two places fence off three slivers that hold no point with `f64`
/// coordinates (the next test), so no point search places them. Each is placed
/// by the side it shares with a piece of the other solid's face, the ball's
/// inside the block and the block's two inside the ball (`docs/decisions.md`
/// §365.3; `src/fuse/select.rs` pins which pieces each rule then keeps). Until
/// that rule, all six refused `a point inside a piece of a face` on the two
/// places, under either policy.
///
/// What each answer would be is a solid with a sliver `2⁻⁵⁰` long and `2⁻⁷⁸`
/// wide in it or of it. **Under `Policy::certified()` each is refused for the
/// range**, `DynamicRangeTooWide` at `2⁵²`: that sliver's `2⁻⁵⁰` beside the
/// block's corners at 4. Under `Policy::tolerant(25.0)`, which only warns on
/// the range, each is refused `ACurvedFaceGoesRightRoundWithNoSeam` out of
/// meshing the ball's face — the refusal §346.2's small ball on a cube's
/// corner is given too. The intersection is the three slivers alone and the
/// block less the ball has the ball's sliver as its dent, so the ball's face
/// there is the sliver; the union and the ball less the block keep the ball
/// less the sliver, and the mesher finds no cut from a rim that small up to
/// both poles.
///
/// **Q1614 moved the certified six.** The fuse meshes the ball's face while
/// it nests the answer's shells, and since Q1614 it asks the range before
/// that (`docs/decisions.md` §87.3). Until then the certified six were refused
/// as the tolerant six are, with the two records the tolerant six then had.
///
/// **The last two were refused earlier, while the answer was written**, by
/// `CannotNameWhichPieceOfACrossingThisIs` out of `rcad-geom`, until the
/// joiner joined a ball's pieces (§229.2). The ball's skin outside the block
/// was then three faces, cut apart along the long arcs of the two circles the
/// block's faces cut. Joined, those arcs are no edges, the face is bounded by
/// the sliver's two short arcs alone, and the refusal comes from the mesh.
/// Neither ever answered.
///
/// **No fuse records anything.** Until Q1837 each tolerant one came back with
/// two records, `side of a half-space`, written while the shells were nested.
/// Q1837 changed nothing but the walk, which since then picks the arc of a
/// circle whose middle a half-space surely holds without asking the sign of
/// the other arc's middle, and records a sign only when neither arc is told.
/// So the two were signs about the middle of an arc nobody could tell, asked
/// while the other arc was told. The refusal did not move.
#[test]
fn the_slivers_no_point_is_inside_are_placed_by_their_sides_and_each_answer_is_refused_later() {
    for rule in EVERY_RULE {
        for ball_first in [true, false] {
            let case = (rule, ball_first);
            let (fused, records) = fused_and_recorded(rule, &Policy::certified(), ball_first);
            assert!(records.is_empty(), "certified, {case:?}: {records:#?}");
            match fused {
                Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => {
                    assert_eq!(ratio, f64::powi(2.0, 52), "certified, {case:?}");
                }
                Ok(fused) => panic!(
                    "certified, {case:?}: the fuse answered with {} solids; check each \
                     against its volume and pin it",
                    fused.solids().len()
                ),
                Err(other) => panic!("certified, {case:?}: {other:?}"),
            }

            let (fused, records) = fused_and_recorded(rule, &Policy::tolerant(25.0), ball_first);
            assert!(records.is_empty(), "tolerant, {case:?}: {records:#?}");
            match fused {
                Err(Error::ACurvedFaceGoesRightRoundWithNoSeam { .. }) => {}
                Ok(fused) => panic!(
                    "tolerant, {case:?}: the fuse answered with {} solids; check each \
                     against its volume and pin it",
                    fused.solids().len()
                ),
                Err(other) => panic!("tolerant, {case:?}: {other:?}"),
            }
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
fn what_it_recorded(height: f64, rule: Rule, policy: &Policy, ball_first: bool) -> Vec<Record> {
    let mut log = ProvenanceLog::new();
    match fuse_a_block_edge_grazing_a_ball(height, rule, policy, ball_first, &mut log) {
        Ok(_) => log.records().to_vec(),
        Err(Error::Undecidable { records }) => records.records().to_vec(),
        Err(other) => panic!("{rule:?} at height {height}, ball first {ball_first}: {other:?}"),
    }
}

/// What the fuse at `h = 0` gave, and every record it wrote, whatever it gave.
fn fused_and_recorded(
    rule: Rule,
    policy: &Policy,
    ball_first: bool,
) -> (rcad_topo::Result<Fused>, Vec<Record>) {
    let mut log = ProvenanceLog::new();
    let fused = fuse_a_block_edge_grazing_a_ball(0.0, rule, policy, ball_first, &mut log);
    let records = match &fused {
        Err(Error::Undecidable { records }) => records.records().to_vec(),
        Ok(_) | Err(_) => log.records().to_vec(),
    };
    (fused, records)
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
    ball_first: bool,
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
    if ball_first {
        model.fuse(&[ball], &[block], rule, policy, log)
    } else {
        model.fuse(&[block], &[ball], rule, policy, log)
    }
}
