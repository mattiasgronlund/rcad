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
//! through, and putting points in order along that loop is refused by name.
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
//! the bar's stubs and beside them, whether the ball stands clear of the bar
//! or rests against it. Where it rests against it, the floor's two round holes
//! touch, and the floor's one inner wire is both circles closing at the touch
//! point. Each circle is read against the round surface across it, so a ray
//! that crosses the floor is answered there too. A second bar notching the
//! floor's edge gives the floor an arc among its sides, which sends it down
//! another path, and that path reads the two touching circles the same way.

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

/// Where they overlap outright, the fuse refuses to put in order the points on
/// the loop the ball and the bar cross in, by name.
///
/// The ball's centre is one and a half units from the bar's axis, so the two
/// cross in a loop round the bar's wall near `x = 4`, and the floor runs
/// through the ball's centre and so through that loop, putting two corners on
/// it. Ordering points along a loop a ball and a pipe cross in is open row
/// Q1593's refusal, and Q1593 takes it. Q1593's own shape has the bar's seam
/// through the loop; this bar's seam is at `x = 6`, on the far side, and it is
/// the floor that runs through this one.
#[test]
fn a_ball_driven_into_the_bar_is_refused_ordering_the_loop_they_cross_in() {
    let (_, answer) = united_all_at_once(3.5);
    assert!(
        matches!(answer, Err(Error::ACrossingThisCannotBeOrderedAlong)),
        "a ball crossing the bar in a loop the floor runs through: {answer:?}"
    );
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
/// The volume is held to the closed form within the measure's own
/// `volume_error` as well, which at [`A_DEFLECTION`] is about `0.61`: the
/// smallest piece the answer could lose, the half ball's `2π/3 ≈ 2.09`, is
/// more than three times that.
fn it_measures_as_the_block_with_half_a_ball_and_two_stubs(model: &Model, solids: &[SolidKey]) {
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

    let closed_form =
        SIDE * SIDE * SIDE + 2.0 / 3.0 * PI * RADIUS * RADIUS * RADIUS + 4.0 * PI * RADIUS * RADIUS;
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
