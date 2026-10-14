//! A block with a ball half sunk in its floor and a bar driven through it.
//!
//! §48's curved trio. The block's floor is one plane crossed by two round
//! surfaces — the ball's skin and the bar's wall — so the arrangement can ask
//! where plane, sphere and pipe meet. The floor is square to the bar, so it
//! cuts each of the two in a circle, and two circles in one plane differ by a
//! line: `three_surfaces.rs` answers that from the line and the ball.
//!
//! What this file pins first is that where the ball stands clear of the bar the
//! question is never asked. The sphere and the pipe have **no point in common
//! at all**, and three surfaces two of which share nothing share nothing. So
//! there is no corner to place.
//!
//! # The skip is a proof and not a distance
//!
//! Every entry below moves the ball and the bar towards each other and asks
//! again. They fuse while `intersect` answers `SurfaceMeet::Nothing`, down to a
//! gap of a millionth of a unit. Where they *touch* — one point in common, on
//! the floor — the skip does not go through, the floor's two circles touch at
//! that point, the corner is placed there, and they fuse into the same block.
//! Where the ball is driven into the bar they cross in a loop the floor runs
//! through, the floor's two corners are put in order along that loop, and they
//! fuse into one block with the ball and the bar run together under its floor.
//! Nothing here is compared against a size.
//!
//! # One pair at a time, the same block
//!
//! The three fuse when they are handed over **together**, and they fuse into
//! the same block a pair at a time. The ball goes in first and leaves the
//! block's floor with a round hole and half a ball hanging under it. That half
//! ball's one corner is on the floor, so no corner says which half of the ball
//! it is, and uniting the bar with it used to build the upper half and catch
//! it as `Invalid(FacesDoNotAgreeWhichWayTheyFace)`. Which way its wire walks
//! the circle says, and [`the_same_three_one_pair_at_a_time_make_the_same_block`]
//! is the answer that came of reading it (`docs/decisions.md` §340.5).
//!
//! # And where a point is asked
//!
//! `SolidView::classify_point` answers a point in the middle of the block, in
//! the bar's stubs and beside them, whether the ball stands clear of the bar,
//! rests against it or is driven into it. Where it rests against it, the
//! floor's two round holes touch, and the floor's one inner wire is both
//! circles closing at the touch point. Each circle is read against the round
//! surface across it, so a ray that crosses the floor is answered there too. A
//! second bar notching the floor's edge gives the floor an arc among its sides,
//! which sends it down another path, and that path reads the two touching
//! circles the same way. Where the ball is driven into the bar, its lower half
//! and the bar's lower wall are each cut by a piece of the one loop the sphere
//! and the pipe cross in, and each is read as the part of its surface in front
//! of all its cuts: the floor, the bar's bottom cap and the other round surface
//! (row Q1863).

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Model, OnAFace, Rule, SolidKey};

/// How fine the meshes every volume here is read off are.
const A_DEFLECTION: f64 = 0.02;

/// How far from the closed form a volume measured at [`A_DEFLECTION`] may be.
///
/// Half a per cent, the same ceiling `degenerate_curved_fuses.rs` uses and for
/// the same reason. Nothing in the kernel reads it.
const HOW_FAR_A_MESH_MAY_BE: f64 = 0.005;

/// The block is seven across.
const SIDE: f64 = 7.0;

/// Both round solids have radius one.
const RADIUS: f64 = 1.0;

/// A block with a ball half sunk in its floor and a bar right through it.
///
/// The ball's centre sits on the floor plane at `x = ball_at`, the bar stands up
/// `z` at `x = 5.0`, and both are halfway across in `y`. The bar is long enough
/// to stick two units out of either end.
fn a_block_a_ball_and_a_bar(ball_at: f64) -> (Model, [SolidKey; 3]) {
    let mut model = Model::new(Scale::millimetres(10.0).expect("a working size"));
    let halfway = SIDE / 2.0;
    let block = model
        .add_cube(SIDE, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a whole-number cube");
    let ball = model
        .add_ball(
            RADIUS,
            &Isometry::translation(Vector::new(ball_at, halfway, 0.0)).expect("a shift"),
        )
        .expect("a whole-number ball");
    let bar = model
        .add_round_bar(
            RADIUS,
            SIDE + 4.0,
            &Isometry::translation(Vector::new(5.0, halfway, -2.0)).expect("a shift"),
        )
        .expect("a bar long enough to go right through");
    (model, [block, ball, bar])
}

/// Unite all three in one fuse: the block against the ball and the bar together.
///
/// The block's floor is then crossed by two round surfaces that are both
/// **cuts**, and its own rim is the plain square it started as.
fn united_all_at_once(ball_at: f64) -> (Model, Result<Vec<SolidKey>, Error>) {
    let (mut model, [block, ball, bar]) = a_block_a_ball_and_a_bar(ball_at);
    let answer = model
        .fuse(
            &[block],
            &[ball, bar],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|fused| fused.solids().to_vec());
    (model, answer)
}

/// Unite the three in the order they were built, one fuse at a time.
fn united_one_at_a_time(ball_at: f64) -> (Model, Result<Vec<SolidKey>, Error>) {
    let (mut model, [block, ball, bar]) = a_block_a_ball_and_a_bar(ball_at);
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let first = match model.fuse(&[block], &[ball], Rule::Union, &policy, &mut log) {
        Ok(fused) => fused.solids().to_vec(),
        Err(why) => return (model, Err(why)),
    };
    let second = model
        .fuse(&first, &[bar], Rule::Union, &policy, &mut log)
        .map(|fused| fused.solids().to_vec());
    (model, second)
}

/// Fuse the block against the ball and the bar together under any `rule`, as
/// [`united_all_at_once`] does for [`Rule::Union`], with the block as the
/// arguments when `block_first` and as the tool otherwise.
fn fused_all_at_once(
    ball_at: f64,
    rule: Rule,
    block_first: bool,
) -> (Model, Result<Vec<SolidKey>, Error>) {
    let (mut model, [block, ball, bar]) = a_block_a_ball_and_a_bar(ball_at);
    let (arguments, tools) = if block_first {
        (vec![block], vec![ball, bar])
    } else {
        (vec![ball, bar], vec![block])
    };
    let answer = model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|fused| fused.solids().to_vec());
    (model, answer)
}

/// The trio fuses, and the answer is the right solid.
///
/// The ball stands three units from the bar's axis, radii one and one, so the
/// sphere and the pipe are two units apart at their nearest and no point lies on
/// both.
#[test]
fn a_ball_and_a_bar_that_miss_each_other_fuse_into_one_block() {
    let (model, answer) = united_all_at_once(2.0);
    let solids = answer.expect("a ball and a bar that miss each other unite with a block");
    it_is_the_block_with_half_a_ball_and_two_stubs(&model, &solids);
}

/// The gap the skip needs is not a size.
///
/// The ball's centre is a millionth of a unit further from the bar's axis than
/// the two radii add up to. The sphere and the pipe still share nothing, the
/// proof still goes through, and the trio still fuses.
#[test]
fn a_ball_a_millionth_of_a_unit_clear_of_the_bar_still_fuses() {
    let (_, answer) = united_all_at_once(5.0 - 2.0 - 1.0e-6);
    answer.expect("a millionth of a unit of clear air is clear air");
}

/// And where they touch they still fuse, into the same block.
///
/// The ball's centre is exactly two units from the bar's axis, so the sphere
/// rests against the pipe at the one point `(4, 3.5, 0)`, on the floor. One
/// point in common is not none, so the proof does not go through and the fuse
/// asks where the floor, the sphere and the pipe meet. The floor cuts the ball
/// in its equator and the bar in a circle of the same radius two units away,
/// two circles that touch, and `three_surfaces.rs` places the corner where
/// they do. A touch at one point adds no volume, so the closed form is the one
/// a ball clear of the bar has.
///
/// The floor's one inner wire is both circles, each a whole closed edge, and
/// both close at the touch point. `SolidView::face_holds_the_point` reads each
/// circle against the round surface across it: the floor holds a point of it
/// far from both circles, not one inside either circle, and has the touch
/// point on its edge. So a point under the floor is answered as when the ball
/// stands clear, and one in the half ball is inside.
#[test]
fn a_ball_resting_against_the_bar_makes_the_same_block_with_both_circles_in_its_floor() {
    let (model, answer) = united_all_at_once(3.0);
    let solids = answer.expect("a ball resting against the bar unites with the block");
    it_is_the_block_with_half_a_ball_and_two_stubs(&model, &solids);
    let solid = model.solid(solids[0]).expect("this model made the key");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    assert_eq!(
        solid.classify_point(Point::new(3.0, 3.5, -0.5), &policy, &mut log),
        Containment::Inside,
        "half a unit under the floor, inside the half ball"
    );
    let floor = solid
        .faces()
        .find(|face| {
            matches!(face.surface(), Surface::Plane(flat)
                if flat.coefficients() == [0.0, 0.0, -1.0, 0.0])
        })
        .expect("the block's floor is still one face")
        .key();
    for (at, held, what) in [
        ([1.0, 1.0, 0.0], OnAFace::Held, "far from both circles"),
        (
            [3.0, 3.5, 0.0],
            OnAFace::NotHeld,
            "inside the ball's circle",
        ),
        ([5.0, 3.5, 0.0], OnAFace::NotHeld, "inside the bar's circle"),
        (
            [4.0, 3.5, 0.0],
            OnAFace::OnItsBoundary,
            "where the two circles touch",
        ),
    ] {
        assert_eq!(
            solid
                .face_holds_the_point(floor, PredicatePoint::Exact(at), &policy, &mut log)
                .expect("the floor follows two whole circles that touch"),
            held,
            "the floor at {at:?}, {what}"
        );
    }
}

/// A floor with a round notch in its edge follows the two touching circles too.
///
/// A second bar of radius 1 stands up `z` on the block's side `x = 0`, so the
/// floor loses a half disk at its edge and has an arc among its sides. A flat
/// face with an arc among its sides is read by its chords and arcs, a path of
/// its own, and that path has to count the wire of the ball's circle and the
/// first bar's circle, touching at `(4, 3.5, 0)`, as a round wire as well, or
/// it refuses the floor everywhere.
#[test]
fn a_floor_notched_by_a_second_bar_still_follows_the_two_touching_circles() {
    let (mut model, [block, ball, bar]) = a_block_a_ball_and_a_bar(3.0);
    let second_bar = model
        .add_round_bar(
            RADIUS,
            SIDE + 4.0,
            &Isometry::translation(Vector::new(0.0, SIDE / 2.0, -2.0)).expect("a shift"),
        )
        .expect("a bar long enough to go right through");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let solids = model
        .fuse(
            &[block],
            &[ball, bar, second_bar],
            Rule::Union,
            &policy,
            &mut log,
        )
        .expect("a block notched by a second bar unites with the ball and the bar")
        .solids()
        .to_vec();
    assert_eq!(solids.len(), 1, "one block with three lumps on it");
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("the answer is a valid solid");
    let solid = model.solid(solids[0]).expect("this model made the key");
    let floor = solid
        .faces()
        .find(|face| {
            matches!(face.surface(), Surface::Plane(flat)
                if flat.coefficients() == [0.0, 0.0, -1.0, 0.0])
        })
        .expect("the block's floor is still one face")
        .key();
    for (at, held, what) in [
        ([1.0, 1.0, 0.0], OnAFace::Held, "far from every circle"),
        (
            [1.5, 3.5, 0.0],
            OnAFace::Held,
            "between the notch and the ball's circle",
        ),
        ([0.5, 3.5, 0.0], OnAFace::NotHeld, "in the notch"),
        (
            [3.0, 3.5, 0.0],
            OnAFace::NotHeld,
            "inside the ball's circle",
        ),
        ([5.0, 3.5, 0.0], OnAFace::NotHeld, "inside the bar's circle"),
        (
            [4.0, 3.5, 0.0],
            OnAFace::OnItsBoundary,
            "where the two circles touch",
        ),
    ] {
        assert_eq!(
            solid
                .face_holds_the_point(floor, PredicatePoint::Exact(at), &policy, &mut log)
                .expect("the floor follows its notch and two whole circles that touch"),
            held,
            "the floor at {at:?}, {what}"
        );
    }
    for (at, where_it_is, what) in [
        (
            Point::new(3.0, 3.5, -0.5),
            Containment::Inside,
            "half a unit under the floor, inside the half ball",
        ),
        (
            Point::new(5.0, 3.25, -1.5),
            Containment::Inside,
            "inside the bar stub under the floor",
        ),
        (
            Point::new(0.25, 3.5, -1.5),
            Containment::Inside,
            "inside the second bar's stub under the floor",
        ),
        (
            Point::new(1.5, 3.5, -0.5),
            Containment::Outside,
            "under the floor, between the second bar and the half ball",
        ),
        (
            Point::new(1.5, 1.5, -1.5),
            Containment::Outside,
            "under the floor, clear of everything",
        ),
    ] {
        assert_eq!(
            solid.classify_point(at, &policy, &mut log),
            where_it_is,
            "{what}"
        );
    }
}

/// Where they overlap outright they fuse too, into one block with the ball's
/// lower half and the bar's lower stub run together under its floor.
///
/// The ball's centre is one and a half units from the bar's axis, so the two
/// cross in one loop round the bar's wall near `x = 4`, and the floor runs
/// through the ball's centre and so through that loop, putting two corners on
/// it. Since Q1661 that loop is drawn, and neither of the floor's two corners
/// is where the walk starts on it, so which comes first round the loop is
/// asked of `rcad-geom`. Since Q1767 `which_comes_first_round_a_crossing`
/// answers by how far round the line through the ball's centre square to the
/// bar's axis each corner is. The bar's own seam is at `x = 6`, on the far
/// side; a bar whose seam runs through the loop is in
/// `two_curves_of_one_pair_that_meet.rs`.
///
/// The faces are the eleven a ball clear of the bar makes: the loop cuts one
/// arc out of the ball's lower half and one out of the bar's lower wall and
/// leaves each one face. The volume is that answer's less the lower half of
/// what the ball and the bar share, [`what_the_ball_and_the_bar_share`]. That
/// half is `0.245`, less than the measure's own error, so the floor is asked
/// too: its hole is both circles run together. Points round the block are asked
/// in [`every_point_round_a_ball_driven_into_the_bar_is_answered`], and the
/// ball's lower half and the bar's lower wall, which the loop trims, in
/// [`the_loop_leaves_the_ball_and_the_bar_wall_what_is_outside_the_other`].
#[test]
fn a_ball_driven_into_the_bar_fuses_into_one_block() {
    let (model, answer) = united_all_at_once(3.5);
    let solids = answer
        .expect("a ball crossing the bar in a loop the floor runs through unites with the block");
    let closed_form =
        SIDE * SIDE * SIDE + 2.0 / 3.0 * PI * RADIUS * RADIUS * RADIUS + 4.0 * PI * RADIUS * RADIUS
            - what_the_ball_and_the_bar_share(1.5) / 2.0;
    let measured = it_is_one_solid_of_eleven_faces_measuring(&model, &solids, closed_form);
    assert!(
        measured > SIDE * SIDE * SIDE + 4.0 * PI * RADIUS * RADIUS + 1.5,
        "the ball's own half, less what the bar already holds of it, is in the answer: \
         {measured}"
    );
    let solid = model.solid(solids[0]).expect("this model made the key");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let floor = solid
        .faces()
        .find(|face| {
            matches!(face.surface(), Surface::Plane(flat)
                if flat.coefficients() == [0.0, 0.0, -1.0, 0.0])
        })
        .expect("the block's floor is still one face")
        .key();
    for (at, held, what) in [
        ([1.0, 1.0, 0.0], OnAFace::Held, "far from both circles"),
        (
            [3.0, 3.5, 0.0],
            OnAFace::NotHeld,
            "in the ball's circle alone",
        ),
        ([4.25, 3.5, 0.0], OnAFace::NotHeld, "in both circles"),
        (
            [5.5, 3.5, 0.0],
            OnAFace::NotHeld,
            "in the bar's circle alone",
        ),
        (
            [3.5, 4.4, 0.0],
            OnAFace::NotHeld,
            "in the ball's circle, off the line between the two",
        ),
        (
            [4.0, 4.4, 0.0],
            OnAFace::Held,
            "outside both, in the notch where the circles cross",
        ),
    ] {
        assert_eq!(
            solid
                .face_holds_the_point(floor, PredicatePoint::Exact(at), &policy, &mut log)
                .expect("the floor follows two arcs of circles that cross"),
            held,
            "the floor at {at:?}, {what}"
        );
    }
}

/// Pins the other two rules on the same shape as the test above, both ways
/// round (queue row Q2022).
///
/// By the same inclusion-exclusion as that test's closed form, `block ∪ (ball
/// ∪ bar) = block + (ball ∪ bar) − block ∩ (ball ∪ bar)`: the intersection is
/// the ball's upper hemisphere and the bar, both already inside the block,
/// less half of what the ball and the bar share — the ball sits astride the
/// floor, so only the half of their shared lens with `z ≥ 0` is in the block
/// too. The block less the two is the block less that. The two less the
/// block is **two** solids: the bar's top stub alone, and the bottom stub run
/// together with the ball's lower hemisphere, less the other half of their
/// shared lens.
#[test]
fn a_ball_driven_into_the_bar_answers_intersection_and_difference_both_ways_round() {
    let half_ball = 2.0 / 3.0 * PI * RADIUS * RADIUS * RADIUS;
    let bar_in_block = SIDE * PI * RADIUS * RADIUS;
    let stubs = 4.0 * PI * RADIUS * RADIUS;
    let half_share = what_the_ball_and_the_bar_share(1.5) / 2.0;
    let intersection = half_ball + bar_in_block - half_share;
    let block_less_the_two = SIDE * SIDE * SIDE - intersection;
    let the_two_less_the_block = half_ball + stubs - half_share;

    for (rule, block_first, closed_form, how_many) in [
        (Rule::Intersection, true, intersection, 1),
        (Rule::Intersection, false, intersection, 1),
        (Rule::Difference, true, block_less_the_two, 1),
        (Rule::Difference, false, the_two_less_the_block, 2),
    ] {
        let (model, answer) = fused_all_at_once(3.5, rule, block_first);
        let solids =
            answer.unwrap_or_else(|why| panic!("{rule:?} (block first: {block_first}): {why:?}"));
        assert_eq!(
            solids.len(),
            how_many,
            "{rule:?} (block first: {block_first})"
        );
        it_measures_within_its_own_error(&model, &solids, closed_form);
    }
}

/// The model's answer validates, and its solids' volumes sum to within
/// their own measures' error, at [`A_DEFLECTION`], of `closed_form`.
fn it_measures_within_its_own_error(model: &Model, solids: &[SolidKey], closed_form: f64) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the answer is a valid solid");
    let (measured, error) = solids.iter().fold((0.0, 0.0), |(volume, error), &key| {
        let measurement = model
            .solid(key)
            .expect("this model made the key")
            .measure(
                Deflection::of(A_DEFLECTION).expect("a fiftieth of a millimetre is a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a solid this file built can be measured");
        (
            volume + measurement.volume(),
            error + measurement.volume_error(),
        )
    });
    assert!(
        (measured - closed_form).abs() <= error,
        "{measured} is further from {closed_form} than the measures' own error, {error}"
    );
}

/// Where the ball is driven into the bar, a point is answered wherever it is.
///
/// The ball's lower half and the bar's lower wall are each trimmed by the one
/// loop the sphere and the pipe cross in, and `classify_point` used to refuse
/// every point whose ray crossed either, `CannotFollowAFacesTrimming`: the
/// middle of the block, both stubs, the half ball and everything under the
/// floor (row Q1863, `docs/decisions.md` §35.6).
#[test]
fn every_point_round_a_ball_driven_into_the_bar_is_answered() {
    let (model, answer) = united_all_at_once(3.5);
    let solids = answer
        .expect("a ball crossing the bar in a loop the floor runs through unites with the block");
    let solid = model.solid(solids[0]).expect("this model made the key");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    for (at, where_it_is, what) in [
        (
            Point::new(3.5, 3.5, 3.5),
            Containment::Inside,
            "the middle of the block",
        ),
        (
            Point::new(5.0, 3.25, SIDE + 1.5),
            Containment::Inside,
            "the stub out of the top of the block",
        ),
        (
            Point::new(5.0, 3.25, -1.5),
            Containment::Inside,
            "a unit and a half under the floor, inside the bar stub",
        ),
        (
            Point::new(3.0, 3.5, -0.5),
            Containment::Inside,
            "half a unit under the floor, inside the half ball",
        ),
        (
            Point::new(4.25, 3.5, -0.5),
            Containment::Inside,
            "half a unit under the floor, inside both the half ball and the bar stub",
        ),
        (
            Point::new(3.5, 3.5, -1.25),
            Containment::Outside,
            "a quarter of a unit under the half ball's lowest point",
        ),
        (
            Point::new(1.5, 1.5, -1.5),
            Containment::Outside,
            "under the floor, clear of everything",
        ),
    ] {
        assert_eq!(
            solid.classify_point(at, &policy, &mut log),
            where_it_is,
            "{what}"
        );
    }
    for (x, y) in BESIDE_THE_STUBS {
        for z in [-1.5, SIDE + 1.5] {
            assert_eq!(
                solid.classify_point(Point::new(x, y, z), &policy, &mut log),
                Containment::Outside,
                "nothing but the bar reaches past the block at ({x}, {y}, {z})"
            );
        }
    }
}

/// The two faces the loop trims hold what it leaves them: the ball's lower half
/// holds what is under the floor and outside the bar, and the bar's lower wall
/// what is between its bottom cap and the floor and outside the ball.
///
/// Each point is on the face's surface, or a rounding off it, and is asked of
/// that face alone, so a side of any one cut read the wrong way round shows
/// here even where no ray `classify_point` fires meets the face near it. Both
/// faces were refused, `AFaceTrimmedByACurveThisCannotFollow`, before row
/// Q1863.
#[test]
fn the_loop_leaves_the_ball_and_the_bar_wall_what_is_outside_the_other() {
    let (model, answer) = united_all_at_once(3.5);
    let solids = answer
        .expect("a ball crossing the bar in a loop the floor runs through unites with the block");
    let solid = model.solid(solids[0]).expect("this model made the key");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let ball = solid
        .faces()
        .find(|face| matches!(face.surface(), Surface::Sphere(_)))
        .expect("the ball's lower half is one face")
        .key();
    let lower_wall = solid
        .faces()
        .find(|face| {
            matches!(face.surface(), Surface::Cylinder(_))
                && face
                    .wires()
                    .flat_map(|wire| wire.points())
                    .any(|corner| corner.coordinates()[2] < -1.0)
        })
        .expect("the bar's wall under the floor is one face")
        .key();
    for (face, at, held, what) in [
        (
            ball,
            [3.5, 3.5, -1.0],
            OnAFace::Held,
            "the ball's lowest point",
        ),
        (
            ball,
            [3.5, 4.1, -0.8],
            OnAFace::Held,
            "the ball under the floor, clear of the bar",
        ),
        (
            ball,
            [4.1, 3.5, -0.8],
            OnAFace::NotHeld,
            "the ball inside the bar",
        ),
        (
            ball,
            [3.5, 3.5, 1.0],
            OnAFace::NotHeld,
            "the ball's highest point, above the floor",
        ),
        (
            ball,
            [3.5, 2.5, 0.0],
            OnAFace::OnItsBoundary,
            "where the floor cuts the ball, clear of the bar",
        ),
        (
            ball,
            [4.5, 3.5, 0.0],
            OnAFace::NotHeld,
            "where the floor cuts the ball, inside the bar",
        ),
        (
            lower_wall,
            [6.0, 3.5, -1.0],
            OnAFace::Held,
            "the side of the wall away from the ball",
        ),
        (
            lower_wall,
            [4.0, 3.5, -1.5],
            OnAFace::Held,
            "the side of the wall facing the ball, under it",
        ),
        (
            lower_wall,
            [4.0, 3.5, -0.5],
            OnAFace::NotHeld,
            "the side of the wall facing the ball, inside it",
        ),
        (
            lower_wall,
            [6.0, 3.5, -3.0],
            OnAFace::NotHeld,
            "below the bar's bottom cap",
        ),
        (
            lower_wall,
            [6.0, 3.5, 1.0],
            OnAFace::NotHeld,
            "above the floor",
        ),
        (
            lower_wall,
            [6.0, 3.5, -2.0],
            OnAFace::OnItsBoundary,
            "on the bar's bottom cap",
        ),
        (
            lower_wall,
            [5.0, 4.5, 0.0],
            OnAFace::OnItsBoundary,
            "on the floor, clear of the ball",
        ),
    ] {
        assert_eq!(
            solid
                .face_holds_the_point(face, PredicatePoint::Exact(at), &policy, &mut log)
                .expect("a face the loop trims is followed"),
            held,
            "{what}, at {at:?}"
        );
    }
}

/// The volume a ball of [`RADIUS`] and a bar of the same radius share, with the
/// ball's centre `off` from the bar's axis.
///
/// Worked by Simpson's rule over slices square to the bar, each the lens two
/// circles share: the ball's slice, of radius `√(1 − z²)`, and the bar's, `off`
/// apart. At `off = 1.5` that is `0.49071`, to five figures from a hundred
/// slices to a hundred thousand.
fn what_the_ball_and_the_bar_share(off: f64) -> f64 {
    let slices = 2_000_u32;
    let width = 2.0 * RADIUS / f64::from(slices);
    let total: f64 = (0..=slices)
        .map(|slice| {
            let z = -RADIUS + width * f64::from(slice);
            let weight = match slice {
                0 => 1.0,
                last if last == slices => 1.0,
                odd if odd % 2 == 1 => 4.0,
                _ => 2.0,
            };
            weight * what_two_circles_share((RADIUS * RADIUS - z * z).max(0.0).sqrt(), RADIUS, off)
        })
        .sum();
    total * width / 3.0
}

/// The area two circles of radius `one` and `other`, with centres `apart`,
/// share.
fn what_two_circles_share(one: f64, other: f64, apart: f64) -> f64 {
    if apart >= one + other {
        return 0.0;
    }
    if apart <= (one - other).abs() {
        return PI * one.min(other) * one.min(other);
    }
    let reach = |near: f64, far: f64| {
        near * near * ((apart * apart + near * near - far * far) / (2.0 * apart * near)).acos()
    };
    let kite = ((-apart + one + other)
        * (apart + one - other)
        * (apart - one + other)
        * (apart + one + other))
        .sqrt();
    reach(one, other) + reach(other, one) - kite / 2.0
}

/// The same three solids fused **one pair at a time** make the same block.
///
/// Fusing the ball in first leaves the block's floor with a round hole and the
/// lower half of the ball hanging under it. Uniting the bar with that, the half
/// ball is the face nothing cuts. Its one wire is the circle where it meets the
/// floor and that circle's one corner is on the floor, so no corner says which
/// side of the floor it hangs on — both halves of the ball have that wire.
/// `Arrangement::material_is_left_of_forward` took the circle's forward
/// direction there, which is the upper half; the floor and the half ball then
/// walked their shared circle the same way, and `how_much_space_it_encloses`
/// in `assemble.rs` refused the answer
/// `Invalid(FacesDoNotAgreeWhichWayTheyFace)`.
///
/// Which way the half ball's wire walks its circle is in the model, read with
/// the closed-edge rule `Model::validate` checks every closed edge against, and
/// `bake.rs` hands it to the arrangement now (`docs/decisions.md` §340.5).
#[test]
fn the_same_three_one_pair_at_a_time_make_the_same_block() {
    let (model, answer) = united_one_at_a_time(2.0);
    let solids = answer.expect("the ball, then the bar, unite with the block");
    it_is_the_block_with_half_a_ball_and_two_stubs(&model, &solids);
}

/// Where across the floor and how far past the block's floor and top a point
/// stands beside the bar's stubs and touches nothing.
const BESIDE_THE_STUBS: [(f64, f64); 4] = [(3.25, 1.75), (2.0, 3.25), (3.5, 3.25), (6.3, 5.7)];

/// The answer is one valid solid, the block with the lower half of the ball and
/// two stubs of the bar on it: it measures as that, a point in the middle of
/// the block or in either stub is inside it, and a point beside the stubs is
/// outside.
fn it_is_the_block_with_half_a_ball_and_two_stubs(model: &Model, solids: &[SolidKey]) {
    it_measures_as_the_block_with_half_a_ball_and_two_stubs(model, solids);

    let solid = model.solid(solids[0]).expect("this model made the key");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    assert_eq!(
        solid.classify_point(Point::new(3.5, 3.5, 3.5), &policy, &mut log),
        Containment::Inside,
        "the middle of the block, clear of the ball and of the bar"
    );
    assert_eq!(
        solid.classify_point(Point::new(5.0, 3.25, -1.5), &policy, &mut log),
        Containment::Inside,
        "a unit and a half under the floor, inside the bar stub"
    );
    assert_eq!(
        solid.classify_point(Point::new(5.0, 3.25, SIDE + 1.5), &policy, &mut log),
        Containment::Inside,
        "the stub out of the top of the block"
    );
    for (x, y) in BESIDE_THE_STUBS {
        for z in [-1.5, SIDE + 1.5] {
            assert_eq!(
                solid.classify_point(Point::new(x, y, z), &policy, &mut log),
                Containment::Outside,
                "nothing but the bar reaches past the block at ({x}, {y}, {z})"
            );
        }
    }
}

/// The answer is one valid solid of eleven faces, and its volume is the block
/// with the lower half of the ball and two stubs of the bar on it.
///
/// The closed form, for a block of side 7 and two round solids of radius 1:
/// the block is `7³ = 343`, the half ball below the floor is `½ · 4/3 · π · 1³
/// = 2π/3`, and each stub is a bar of radius 1 two units long, `π · 1² · 2 =
/// 2π`, so two of them are `4π`. That is `343 + 2π/3 + 4π = 357.661`.
///
/// The smallest piece the answer could lose, the half ball's `2π/3 ≈ 2.09`, is
/// more than three times the measure's own error.
fn it_measures_as_the_block_with_half_a_ball_and_two_stubs(model: &Model, solids: &[SolidKey]) {
    let closed_form =
        SIDE * SIDE * SIDE + 2.0 / 3.0 * PI * RADIUS * RADIUS * RADIUS + 4.0 * PI * RADIUS * RADIUS;
    let measured = it_is_one_solid_of_eleven_faces_measuring(model, solids, closed_form);
    // Leaving the ball's own half out would come to 355.57, which is 0.59% below
    // the closed form and five times the width of the band above. The measured
    // 357.59 is 0.019% below it. A count of faces and a volume within half a per
    // cent are two different claims and this file makes both (§47.2).
    assert!(
        measured > SIDE * SIDE * SIDE + 4.0 * PI * RADIUS * RADIUS + 2.0,
        "the ball's own half is in the answer and not merely a hole in the floor: \
         {measured}"
    );
}

/// The answer is one valid solid of eleven faces, six of the block, the ball's
/// lower half, and the bar's wall and cap at each end, and its volume is
/// `closed_form`: within the measure's own `volume_error`, which at
/// [`A_DEFLECTION`] is about `0.61`, and under it by less than
/// [`HOW_FAR_A_MESH_MAY_BE`]. Hands back the volume it measured.
fn it_is_one_solid_of_eleven_faces_measuring(
    model: &Model,
    solids: &[SolidKey],
    closed_form: f64,
) -> f64 {
    assert_eq!(solids.len(), 1, "one block with two lumps on it");

    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the answer is a valid solid");

    let solid = model.solid(solids[0]).expect("this model made the key");
    assert_eq!(
        solid.faces().count(),
        11,
        "six of the block, the ball's lower half, and the bar's wall and cap \
         at each end"
    );

    let measurement = solid
        .measure(
            Deflection::of(A_DEFLECTION).expect("a fiftieth of a millimetre is a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this file built can be measured");
    let measured = measurement.volume();
    assert!(
        (measured - closed_form).abs() <= measurement.volume_error(),
        "{measured} is further from {closed_form} than the measure's own error, \
         {}",
        measurement.volume_error()
    );
    assert!(
        (0.0..HOW_FAR_A_MESH_MAY_BE).contains(&((closed_form - measured) / closed_form)),
        "the answer is a block with half a ball and two bar stubs on it, and a \
         mesh inside a round skin under-measures it: {measured} against {closed_form}"
    );
    measured
}
