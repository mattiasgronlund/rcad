//! One plane and two curved surfaces, where that is not a quartic.
//!
//! `three_surfaces.rs` works out where three surfaces meet by crossing two
//! planes into a line and cutting that line with the third surface. One plane
//! and two quadrics has no second plane to cross, and in general it is a quartic
//! — two conics in one plane — which nothing in this milestone solves.
//!
//! **Two families of it are not a quartic**, and both are ordinary mechanical
//! CAD:
//!
//! - Two quadrics that square `p` the same way differ by something *linear*, so
//!   every point they share sits on one flat surface and the triple is two
//!   planes and a quadric after all. Two pipes running the same way are it, and
//!   so are two balls.
//! - A plane that misses one of the two settles the triple on its own, and a
//!   plane that touches one at a single point leaves one candidate and one sign
//!   question. A ball and a bar both resting on a block's floor are it.
//!
//! Every corner below is worked out by hand first and written in the test, and
//! every one of them lands on a whole number — so a box round it has no width
//! and the answer is checked against the arithmetic-free number and not against
//! a second run of the same code.
//!
//! What is still refused is at the bottom, with the corner it refuses written
//! out in whole numbers beside it. That is what makes it a **gap** rather than a
//! limit (§16.10): the points are there, and no discriminant this file has
//! reaches them.

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cylinder, Error, MeetOfThree, Plane, Point, Sphere, Surface, Vector,
    meet_of_three_surfaces,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface};

/// A plane through `through` with normal `normal`.
fn flat(normal: [f64; 3], through: [f64; 3]) -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(
            Point::new(through[0], through[1], through[2]),
            Vector::new(normal[0], normal[1], normal[2]),
        )
        .expect("a normal with length"),
    )
}

/// A pipe of `radius` running along `direction` through `through`.
fn pipe(through: [f64; 3], direction: [f64; 3], radius: f64) -> Surface {
    Surface::Cylinder(
        Cylinder::new(
            Axis::new(
                Point::new(through[0], through[1], through[2]),
                Vector::new(direction[0], direction[1], direction[2]),
            )
            .expect("a direction with length"),
            radius,
        )
        .expect("a radius"),
    )
}

/// A ball of `radius` about `centre`.
fn ball(centre: [f64; 3], radius: f64) -> Surface {
    Surface::Sphere(
        Sphere::new(Point::new(centre[0], centre[1], centre[2]), radius).expect("a radius"),
    )
}

/// The answer, and nothing written into the log — a fully certified operation
/// logs nothing, and that empty log is what says nothing was fudged (§5).
fn meet(surfaces: [&Surface; 3]) -> Result<MeetOfThree, Error> {
    let mut log = ProvenanceLog::new();
    let answer = meet_of_three_surfaces(surfaces, &Policy::certified(), &mut log);
    if answer.is_ok() {
        assert!(
            log.is_empty(),
            "a certified answer wrote {log:?} into the provenance log"
        );
    }
    answer
}

/// The corners, each checked to be a box of no width, and each checked to sit
/// exactly on all three surfaces it came from.
///
/// **Both halves matter.** A box of no width says the arithmetic rounded
/// nothing, so comparing it against a whole number is comparing the whole
/// answer. `Decision::Zero` on all three says the number it landed on is the
/// corner and not some other exact number.
fn corners_of(surfaces: [&Surface; 3]) -> Vec<[f64; 3]> {
    let answer = meet(surfaces).expect("these three meet");
    let MeetOfThree::At(points) = &answer else {
        panic!("expected corners, got {answer:?}");
    };
    points
        .iter()
        .map(|point| {
            let sides = point.enclosure();
            for surface in surfaces {
                assert_eq!(
                    which_side_of(surface, *point),
                    Decision::Zero,
                    "{point:?} is not certified on {surface:?}"
                );
            }
            core::array::from_fn(|axis| {
                assert_eq!(
                    sides[axis].lo(),
                    sides[axis].hi(),
                    "{point:?} is a box with width, so nothing here is exact"
                );
                sides[axis].lo()
            })
        })
        .collect()
}

/// Which side of a surface a point is on.
fn which_side_of(surface: &Surface, point: PredicatePoint) -> Decision {
    match surface {
        Surface::Plane(this) => this.side_of_point(point),
        Surface::Sphere(this) => this.side_of(point),
        Surface::Cylinder(this) => this.side_of(point),
        Surface::Cone(this) => this.side_of(point),
        Surface::Torus(this) => this.side_of(point),
        other => panic!("a surface this test cannot ask about: {other:?}"),
    }
}

/// **The corner that stood between the kernel and unioning two overlapping
/// parallel pipes.**
///
/// Two bars of radius five whose axes run up `z` six apart, and the cap plane of
/// one of them. A cap meets both pipes where `x² + y² = 25` and
/// `(x − 6)² + y² = 25`, and subtracting one from the other leaves `12x = 36` —
/// so `x = 3`, `y² = 16`, and the two corners are `(3, ±4, 0)`. A 3-4-5, and
/// every number in it whole.
///
/// `rcad-topo/tests/fusing_two_bars_side_by_side.rs` is the fuse this unblocks
/// and names the same two points.
#[test]
fn a_cap_plane_meets_two_bars_running_the_same_way_at_two_corners() {
    let cap = flat([0.0, 0.0, -1.0], [0.0, 0.0, 0.0]);
    let here = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);
    let there = pipe([6.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);

    let mut corners = corners_of([&cap, &here, &there]);
    corners.sort_by(|one, other| one[1].total_cmp(&other[1]));
    assert_eq!(corners, vec![[3.0, -4.0, 0.0], [3.0, 4.0, 0.0]]);
}

/// The same, with the two bars a different size from each other, so nothing
/// leans on the two radii being equal.
///
/// Radius thirteen at the origin and radius five twelve along `x`:
/// `x² + y² = 169` against `(x − 12)² + y² = 25` leaves `24x = 288`, so `x = 12`
/// and `y² = 25`. The corners are `(12, ±5, 0)` — a 5-12-13 — and the second
/// bar's wall passes through its own axis's `x`, which is what makes the two
/// radii pulling in different directions visible in the answer.
#[test]
fn two_bars_of_different_sizes_running_the_same_way_meet_a_cap_where_they_should() {
    let cap = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
    let here = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 13.0);
    let there = pipe([12.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);

    let mut corners = corners_of([&cap, &here, &there]);
    corners.sort_by(|one, other| one[1].total_cmp(&other[1]));
    assert_eq!(corners, vec![[12.0, -5.0, 0.0], [12.0, 5.0, 0.0]]);
}

/// Two pipes running the same way but written with opposite directions are still
/// two pipes running the same way.
///
/// `‖x × w‖²` does not change when `w` flips, so the two square `p` the same way
/// and the road holds. Nothing in a solid guarantees two parallel faces store
/// their axes pointing the same way, so this is the case a modeller reaches
/// without meaning to.
#[test]
fn two_bars_whose_axes_are_written_pointing_opposite_ways_still_meet_a_cap() {
    let cap = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
    let here = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);
    let upside_down = pipe([6.0, 0.0, 0.0], [0.0, 0.0, -1.0], 5.0);

    let mut corners = corners_of([&cap, &here, &upside_down]);
    corners.sort_by(|one, other| one[1].total_cmp(&other[1]));
    assert_eq!(corners, vec![[3.0, -4.0, 0.0], [3.0, 4.0, 0.0]]);
}

/// Two balls and a plane, which is the same road: two balls square `p` the same
/// way whatever their centres and sizes.
///
/// Radius five about the origin and radius five eight along `x` share the circle
/// `x = 4`, and the plane `y = 0` cuts that circle at `z² = 25 − 16`. The corners
/// are `(4, 0, ±3)`.
#[test]
fn a_plane_meets_two_balls_where_their_own_shared_circle_crosses_it() {
    let wall = flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]);
    let here = ball([0.0, 0.0, 0.0], 5.0);
    let there = ball([8.0, 0.0, 0.0], 5.0);

    let mut corners = corners_of([&wall, &here, &there]);
    corners.sort_by(|one, other| one[2].total_cmp(&other[2]));
    assert_eq!(corners, vec![[4.0, 0.0, -3.0], [4.0, 0.0, 3.0]]);
}

/// **A `Nowhere` that is earned rather than guessed**, and one that is not
/// obvious from either pair on its own.
///
/// Two bars of radius five up `z`, eight apart, share exactly the two lines at
/// `x = 4`. The plane `x = 5` cuts *both* bars in real curves — a single line
/// down the first, since it is tangent, and two lines down the second — so
/// neither pair is empty and neither is a touch. What settles it is that `x = 5`
/// runs parallel to `x = 4` and never reaches it.
#[test]
fn a_plane_running_parallel_to_where_two_bars_meet_shares_nothing_with_them() {
    let here = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);
    let there = pipe([8.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);

    for reach in [5.0, 6.0, -5.0] {
        let wall = flat([1.0, 0.0, 0.0], [reach, 0.0, 0.0]);
        assert_eq!(
            meet([&wall, &here, &there]),
            Ok(MeetOfThree::Nowhere),
            "the plane x = {reach} does not reach x = 4"
        );
    }
}

/// The plane that holds the whole of where the two bars meet has no corner on
/// it, and saying `Nowhere` there would be as wrong as guessing one.
///
/// `x = 4` is exactly where the two bars of radius five eight apart share their
/// wall, so the three surfaces share two whole lines — `y = ±3`, `z` free.
#[test]
fn a_plane_holding_the_whole_of_where_two_bars_meet_is_more_than_a_point() {
    let here = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);
    let there = pipe([8.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);
    let wall = flat([1.0, 0.0, 0.0], [4.0, 0.0, 0.0]);

    assert_eq!(
        meet([&wall, &here, &there]),
        Ok(MeetOfThree::MoreThanAPoint)
    );
}

/// Two balls about one centre share nothing at all, and the plane never comes
/// into it.
///
/// Their difference has no `p` left in it either — it is `9 − 25`, the equation
/// `0 = −16`, which nothing satisfies.
#[test]
fn two_balls_about_one_centre_and_a_plane_share_nothing() {
    let wall = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
    let small = ball([0.0, 0.0, 0.0], 3.0);
    let large = ball([0.0, 0.0, 0.0], 5.0);

    assert_eq!(meet([&wall, &small, &large]), Ok(MeetOfThree::Nowhere));
}

/// Two balls written with the same numbers share their whole selves, so a plane
/// across them shares a whole circle and there is no corner to name.
#[test]
fn one_ball_written_twice_and_a_plane_share_a_whole_circle() {
    let wall = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
    let one = ball([0.0, 0.0, 0.0], 5.0);

    assert_eq!(meet([&wall, &one, &one]), Ok(MeetOfThree::MoreThanAPoint));
}

/// **A ball and a bar both resting on a block's floor**, which is the other
/// road: the floor touches the ball at one point and the whole question is
/// whether the bar passes through it.
///
/// The ball has radius five about `(0, 0, 5)`, so it touches `z = 0` at the
/// origin exactly. The bar has radius three about an axis running along `x` at
/// height three, so it touches the same floor along a line — and how far along
/// `y` that line sits is the only thing that changes between the three rows.
///
/// **The answer does not depend on how far apart they are**, and that is the
/// point of measuring one, nine and twenty-seven: the refusal this replaces did
/// not depend on it either.
#[test]
fn a_ball_and_a_bar_resting_on_one_floor_are_answered_at_every_distance() {
    let floor = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
    let resting = ball([0.0, 0.0, 5.0], 5.0);

    for apart in [1.0, 9.0, 27.0] {
        let bar = pipe([0.0, apart, 3.0], [1.0, 0.0, 0.0], 3.0);
        assert_eq!(
            meet([&floor, &resting, &bar]),
            Ok(MeetOfThree::Nowhere),
            "a ball touching the floor at the origin and a bar touching it at y = {apart}"
        );
    }
}

/// The same three, moved until they really do share a corner.
///
/// The bar's axis at `(0, 0, 3)` puts its wall through the origin, and the ball
/// already touches the floor there. So all three pass through `(0, 0, 0)` and
/// the answer is that one point — checked against the whole numbers and against
/// `Decision::Zero` on all three, not against a second run of the same code.
#[test]
fn a_ball_and_a_bar_touching_one_floor_at_one_place_meet_there() {
    let floor = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
    let resting = ball([0.0, 0.0, 5.0], 5.0);
    let bar = pipe([0.0, 0.0, 3.0], [1.0, 0.0, 0.0], 3.0);

    assert_eq!(corners_of([&floor, &resting, &bar]), vec![[0.0, 0.0, 0.0]]);
}

/// A plane that misses one of the two settles it on its own, and neither the
/// second surface nor a quartic comes into it.
#[test]
fn a_plane_that_misses_one_of_the_two_leaves_them_sharing_nothing() {
    let far_off = flat([0.0, 0.0, 1.0], [0.0, 0.0, 40.0]);
    let resting = ball([0.0, 0.0, 5.0], 5.0);
    let bar = pipe([0.0, 0.0, 3.0], [1.0, 0.0, 0.0], 3.0);

    assert_eq!(meet([&far_off, &resting, &bar]), Ok(MeetOfThree::Nowhere));
}

/// **How many corners there are, against the closed form, over six hundred
/// pairs of bars — and the same six hundred turned.**
///
/// Two circles of radius `r₁` and `r₂` whose centres are `apart` cross twice
/// when `|r₁ − r₂| < apart < r₁ + r₂`, touch once at either end of that, and
/// miss otherwise. Nothing about that needs the arithmetic, so it is the oracle
/// and the code is what is measured against it.
///
/// **The turned half is the first time transform invariance has run on a curved
/// face at all** (§48.2 item 1). A rotation does not move a circle relative to
/// another circle, so the count may not change — and once turned, no corner
/// lands on a number `f64` holds, so the second half of the oracle is the one
/// the property suite uses: a corner may come back in a box, and it may never be
/// certified *off* any of the three surfaces it came from.
///
/// **The tangent rows are the only ones the turn is allowed to move, and that
/// is a rule rather than an exception.** Two bars that touch have a
/// discriminant of exactly zero, and a rotation is not exact — so what the
/// turned numbers name is two bars a rounding away from touching, which may
/// touch, may miss, or may cross twice, and the arithmetic is entitled to any of
/// those or to no answer at all. `every_turned_quartic_crossing_that_refuses_
/// was_exactly_tangent_before_the_turn` says the same thing for a pair of
/// surfaces, one dimension up.
///
/// A turned pair that was **not** tangent before the turn has a whole unit of
/// room either side, so it must answer and must answer what the closed form
/// says. That is the half of this that would catch a wrong corner, and it covers
/// **1184 of the 1274 rows** — every square one and every turned one that was
/// not tangent. What the ninety tangent-and-turned rows do is measured and
/// pinned rather than asserted, so a change of shape there shows up without
/// being called wrong.
#[test]
fn how_many_corners_two_bars_running_the_same_way_have_is_what_the_closed_form_says() {
    let turn = rcad_geom::Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("a direction"),
        0.4,
    )
    .expect("a finite angle");

    let mut counted = 0usize;
    let mut tangent_and_turned = 0usize;
    let mut refused_tangent = 0usize;
    for here in 1..=7 {
        for there in 1..=7 {
            for apart in 1..=13 {
                let (here, there, apart) = (f64::from(here), f64::from(there), f64::from(apart));
                let expected = if apart > (here - there).abs() && apart < here + there {
                    2
                } else if apart == here + there || apart == (here - there).abs() {
                    1
                } else {
                    0
                };

                for turned in [false, true] {
                    let place = |surface: Surface| {
                        if turned {
                            surface.transformed(&turn)
                        } else {
                            surface
                        }
                    };
                    let surfaces = [
                        place(flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0])),
                        place(pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], here)),
                        place(pipe([apart, 0.0, 0.0], [0.0, 0.0, 1.0], there)),
                    ];
                    let asked = [&surfaces[0], &surfaces[1], &surfaces[2]];
                    let a_rounding_could_move_it = turned && expected == 1;
                    counted += 1;
                    if a_rounding_could_move_it {
                        tangent_and_turned += 1;
                    }
                    let answer = match meet(asked) {
                        Ok(answer) => answer,
                        Err(refused) => {
                            assert!(
                                a_rounding_could_move_it,
                                "r = {here} and {there}, {apart} apart, turned {turned}: \
                                 {refused:?}"
                            );
                            refused_tangent += 1;
                            continue;
                        }
                    };
                    let found = match &answer {
                        MeetOfThree::Nowhere => 0,
                        MeetOfThree::At(points) => points.len(),
                        // Two bars that are two bars share no whole curve, and a
                        // plane square to both cannot hold one either.
                        MeetOfThree::MoreThanAPoint => {
                            panic!("r = {here} and {there}, {apart} apart share a whole curve")
                        }
                        other => panic!("r = {here} and {there}, {apart} apart: {other:?}"),
                    };
                    if a_rounding_could_move_it {
                        assert!(
                            found <= 2,
                            "r = {here} and {there}, {apart} apart, turned: {answer:?}"
                        );
                    } else {
                        assert_eq!(
                            found, expected,
                            "r = {here} and {there}, {apart} apart, turned {turned}: {answer:?}"
                        );
                    }
                    if let MeetOfThree::At(points) = &answer {
                        for point in points {
                            for surface in asked {
                                let side = which_side_of(surface, *point);
                                assert!(
                                    !matches!(side, Decision::Pos | Decision::Neg),
                                    "{point:?} was certified {side:?} off {surface:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(counted, 1274, "the sweep stopped short");
    assert_eq!(tangent_and_turned, 90, "the tangent rows moved");
    // Measured, and a watch rather than a bound: how many of the tangent rows
    // the turn leaves undecided depends on where the rotation puts each pair's
    // discriminant, and nothing derives it. What it is there to catch is the
    // figure moving a long way, which would say the discriminant stopped being
    // read out of the exact expansion.
    assert_eq!(
        refused_tangent, TANGENT_ROWS_A_TURN_LEAVES_UNDECIDED,
        "how many turned tangent pairs go undecided moved"
    );
}

/// How many of the ninety tangent rows above the turn leaves undecided.
///
/// Square to the axes **not one of the ninety refuses**, so the whole of this is
/// the turn and not the geometry. The other twenty-five come back with a count
/// about the two bars the turned numbers actually name, which are a rounding
/// away from touching and may genuinely miss or genuinely cross.
const TANGENT_ROWS_A_TURN_LEAVES_UNDECIDED: usize = 65;

/// Which three surfaces they are decides the answer, and which face asked first
/// does not (§31.3).
#[test]
fn the_same_triple_in_every_order_gives_the_same_answer() {
    let cap = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
    let here = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);
    let there = pipe([6.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);

    let first = meet([&cap, &here, &there]);
    for order in [
        [&cap, &there, &here],
        [&here, &cap, &there],
        [&here, &there, &cap],
        [&there, &cap, &here],
        [&there, &here, &cap],
    ] {
        assert_eq!(meet(order), first, "{order:?} answered differently");
    }
}

/// **What is still refused, with the corners it refuses written out in whole
/// numbers.**
///
/// Two bars leaning across each other and a plane. A bar of radius five up `z`
/// gives `x² + y² = 25`; a bar of radius five along `x` at height four gives
/// `y² + (z − 4)² = 25`; and the floor `z = 0` turns the second into `y² = 9`.
/// So the four corners are `(±4, ±3, 0)` and every number in them is whole.
///
/// The two bars do **not** square `p` the same way — one is
/// `I − ẑẑᵀ` and the other `I − x̂x̂ᵀ` — so their difference still has `p·p` in
/// it and there is no flat surface to fall back on. The floor cuts each of them
/// in a real curve, so neither is missed and neither is touched. That leaves the
/// quartic, and nothing here solves one.
///
/// A **gap** and not a limit (§16.10): the answer exists, it is exact, and what
/// is missing is a solver.
#[test]
fn two_bars_leaning_across_each_other_and_a_plane_are_still_refused() {
    let floor = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
    let upright = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);
    let across = pipe([0.0, 0.0, 4.0], [1.0, 0.0, 0.0], 5.0);

    for corner in [
        [4.0, 3.0, 0.0],
        [4.0, -3.0, 0.0],
        [-4.0, 3.0, 0.0],
        [-4.0, -3.0, 0.0],
    ] {
        let point = PredicatePoint::from(corner);
        for surface in [&floor, &upright, &across] {
            assert_eq!(
                which_side_of(surface, point),
                Decision::Zero,
                "{corner:?} is exactly on {surface:?}"
            );
        }
    }

    assert_eq!(
        meet([&floor, &upright, &across]),
        Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
    );
}

/// A ball and a bar that the plane cuts in curves rather than grazes is the
/// other half of what is left, and it is refused for the same reason.
///
/// A pipe of radius three up `z` and a ball of radius five about the origin meet
/// where `x² + y² = 9` and `z = ±4`, so `(3, 0, 4)` is on both in whole numbers.
/// The plane `z = 4` holds it too. A ball squares `p` as `I` and a pipe as
/// `I − ẑẑᵀ`, which is not the same way, and the plane cuts each in a circle —
/// so both roads are shut and the quartic is what is left.
#[test]
fn a_ball_and_a_bar_a_plane_cuts_right_through_are_still_refused() {
    let cap = flat([0.0, 0.0, 1.0], [0.0, 0.0, 4.0]);
    let upright = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 3.0);
    let round = ball([0.0, 0.0, 0.0], 5.0);

    let corner = PredicatePoint::from([3.0, 0.0, 4.0]);
    for surface in [&cap, &upright, &round] {
        assert_eq!(
            which_side_of(surface, corner),
            Decision::Zero,
            "(3, 0, 4) is exactly on {surface:?}"
        );
    }

    assert_eq!(
        meet([&cap, &upright, &round]),
        Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
    );
}

/// Three curved surfaces and no plane at all is refused before either road is
/// tried, because both of them start from a plane.
#[test]
fn three_curved_surfaces_are_refused_before_either_road_is_tried() {
    let here = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);
    let there = pipe([6.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);
    let round = ball([0.0, 0.0, 0.0], 5.0);

    assert_eq!(
        meet([&here, &there, &round]),
        Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
    );
}
