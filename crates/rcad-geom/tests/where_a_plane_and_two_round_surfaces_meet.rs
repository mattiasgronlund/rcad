//! One plane and two curved surfaces, where that is not a quartic.
//!
//! `three_surfaces.rs` works out where three surfaces meet by crossing two
//! planes into a line and cutting that line with the third surface. One plane
//! and two quadrics has no second plane to cross, and in general it is a quartic
//! — two conics in one plane — which nothing in this milestone solves.
//!
//! **These families of it are not a quartic**, and all of them are ordinary
//! mechanical CAD:
//!
//! - Two quadrics that square `p` the same way differ by something *linear*, so
//!   every point they share sits on one flat surface and the triple is two
//!   planes and a quadric after all. Two pipes running the same way are it, and
//!   so are two balls.
//! - Two surfaces turned about one axis — a ball, a pipe, a cone — are cut by a
//!   plane square to that axis in two circles about one centre, which are one
//!   circle or share nothing. A bar through a ball and a cap across both is it.
//! - A plane that misses one of the two settles the triple on its own, and a
//!   plane that touches one at a single point leaves one candidate and one sign
//!   question. A ball and a bar both resting on a block's floor are it.
//! - Two surfaces turned about one axis share circles square to it, and a
//!   plane parallel to the axis cuts each in a chord, touches it or misses it.
//!   A D-flat on a shaft across a chamfer cone is it.
//! - A plane that cuts each of the two in a circle, about two centres, cuts
//!   them in two circles that differ by a line. A floor across a bar and a ball
//!   off its axis is it.
//! - A plane that runs along a pipe cuts it in lines, and each line meets the
//!   other surface where a quadratic is zero. A floor along one of two bars
//!   that cross is it.
//!
//! **Two more are quartics, but ones no algebraic solver is needed for.** A
//! plane cuts a ball, a pipe or a cone in a curve that keeps between two
//! distances from where the axis crosses it:
//!
//! - Where two such bands keep apart the three share nothing. A cap a rounding
//!   off square to a bar and a chamfer cone it keeps apart from is it.
//! - Where the bands overlap and the plane is near enough square to both axes,
//!   one curve is a line bent by little across the other, and they cross at two
//!   points, one on each side of where the other's axis crosses the plane, or
//!   nowhere.
//!   The same cap across a bar and a cone it cuts in one circle on paper is it.
//!   A ball whose centre is on a pipe's or a cone's axis touches it along a
//!   circle on a plane known exactly, and that plane answers it instead: a
//!   slanted cut through where a rod's round end meets it.
//!
//! Every corner below is worked out by hand first and written in the test, and
//! every one of them lands on a whole number — so a box round it has no width
//! and the answer is checked against the arithmetic-free number and not against
//! a second run of the same code.
//!
//! What is still refused is written with the corner it refuses in whole
//! numbers beside it. That is what makes it a **gap** rather than a limit
//! (§16.10): the points are there, and no discriminant this file has reaches
//! them.

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cone, Cylinder, Error, MeetOfThree, Plane, Point, Sphere, Surface, Torus, Vector,
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

/// A cone of `slope` with its apex at `apex`, its axis along `direction`.
fn cone(apex: [f64; 3], direction: [f64; 3], slope: f64) -> Surface {
    Surface::Cone(
        Cone::new(
            Axis::new(
                Point::new(apex[0], apex[1], apex[2]),
                Vector::new(direction[0], direction[1], direction[2]),
            )
            .expect("a direction with length"),
            slope,
        )
        .expect("a slope"),
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
/// the turn and not the geometry. The other thirty-nine come back with a count
/// about the two bars the turned numbers actually name, which are a rounding
/// away from touching and may genuinely miss or genuinely cross.
///
/// It was 65 until §118.11's convention reached `three_surfaces.rs`. A pipe's
/// discriminant used to take off a bare `r²` against terms that were degree two
/// in the direction, so it was certified about a pipe `r/‖w‖` wide rather than
/// the one whose skin was being asked about; fourteen of the ninety went
/// undecided for that reason alone and now decide. Measured on the merged tree,
/// not on the lane that moved it.
///
/// **It was 51 until §322, and every one of the 51 was the exact level's
/// product floor and not the geometry.** The turned discriminant is an
/// expansion whose lowest entries sit far down the range, and squaring it
/// multiplied two of them into a product below `2⁻⁹⁶⁹`, which refused the whole
/// sum. Keeping that product with a box one smallest subnormal wide round what
/// it lost leaves each discriminant a rounding of itself wide or narrower, and
/// all 51 decide. A `Nearly` (`exact.rs`) that lost nothing is the exact
/// expansion entry for entry, so a row that lost nothing could not have moved.
const TANGENT_ROWS_A_TURN_LEAVES_UNDECIDED: usize = 0;

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

/// **Two bars leaning across each other and a floor along one of them**, which
/// was refused until a plane along a pipe became a road of its own.
///
/// A bar of radius five up `z` gives `x² + y² = 25`; a bar of radius five
/// along `x` at height four gives `y² + (z − 4)² = 25`; and the floor `z = 0`
/// runs along the second and cuts it in the two lines `y² = 9`. Each line meets
/// the first bar where `x² = 25 − 9`, so the four corners are `(±4, ±3, 0)`
/// and every number in them is whole.
///
/// The two bars do **not** square `p` the same way, so their difference still
/// has `p·p` in it and no flat surface holds what they share. The floor cuts
/// the bar along `x` in lines and not a curve, and a line meets the bar up `z`
/// where a quadratic is zero, so no quartic is asked. The line `y = −3` comes
/// first and the lower `x` first on each line.
#[test]
fn two_bars_leaning_across_each_other_and_a_floor_meet_at_four_whole_corners() {
    let floor = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
    let upright = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);
    let across = pipe([0.0, 0.0, 4.0], [1.0, 0.0, 0.0], 5.0);

    assert_eq!(
        corners_of([&floor, &upright, &across]),
        vec![
            [-4.0, -3.0, 0.0],
            [4.0, -3.0, 0.0],
            [-4.0, 3.0, 0.0],
            [4.0, 3.0, 0.0]
        ]
    );
}

/// **A ball and a bar off its axis that a plane cuts right through**, about
/// two different centres, which was refused until two circles in one plane
/// became a road of their own.
///
/// A pipe of radius three up `z` and a ball of radius five about `(4, 0, 0)`.
/// The floor `z = 0` cuts the pipe in `x² + y² = 9` and the ball in
/// `(x − 4)² + y² = 25`, and subtracting leaves `8x = 0` — so the corners are
/// `(0, ±3, 0)`, in whole numbers, and `y = −3` comes first. Two circles in
/// one plane differ by a line whatever their centres, so no quartic is asked.
#[test]
fn a_ball_and_a_bar_off_its_axis_that_a_plane_cuts_right_through_meet_at_two_whole_corners() {
    let floor = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
    let upright = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 3.0);
    let round = ball([4.0, 0.0, 0.0], 5.0);

    assert_eq!(
        corners_of([&floor, &upright, &round]),
        vec![[0.0, -3.0, 0.0], [0.0, 3.0, 0.0]]
    );
}

/// **A bar through a ball and a cap square to the bar**, which is the third
/// road: the cap cuts each of the two in a circle about the bar's axis, and two
/// circles about one centre are one circle or share nothing.
///
/// A pipe of radius three up `z` and a ball of radius five about the origin
/// meet where `x² + y² = 9` and `z = ±4`. The caps `z = ±4` cut the ball in
/// `x² + y² = 25 − 16`, which is the pipe's own circle, so the three share it
/// whole — checked at four whole-number points on it. The caps `z = ±2` cut the
/// ball in `x² + y² = 21` against the pipe's `9`, and nothing is on both.
#[test]
fn a_cap_across_a_bar_through_a_ball_shares_their_circle_or_nothing() {
    let upright = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 3.0);
    let round = ball([0.0, 0.0, 0.0], 5.0);

    for height in [4.0, -4.0] {
        let cap = flat([0.0, 0.0, 1.0], [0.0, 0.0, height]);
        for corner in [[3.0, 0.0], [0.0, 3.0], [-3.0, 0.0], [0.0, -3.0]] {
            let point = PredicatePoint::from([corner[0], corner[1], height]);
            for surface in [&cap, &upright, &round] {
                assert_eq!(
                    which_side_of(surface, point),
                    Decision::Zero,
                    "{point:?} is exactly on {surface:?}"
                );
            }
        }
        assert_eq!(
            meet([&cap, &upright, &round]),
            Ok(MeetOfThree::MoreThanAPoint),
            "the cap z = {height}"
        );
    }
    for height in [2.0, -2.0] {
        let cap = flat([0.0, 0.0, 1.0], [0.0, 0.0, height]);
        assert_eq!(
            meet([&cap, &upright, &round]),
            Ok(MeetOfThree::Nowhere),
            "the cap z = {height}"
        );
    }
}

/// A cone is a double cone (`cone.rs` says so out loud), so a plane square to
/// its axis cuts it on either side of the apex, and a bar about the same axis
/// shares a whole circle with it on both.
///
/// A cone of slope one with its apex at the origin is `x² + y² = z²`, and a
/// pipe of radius three about the same axis is `x² + y² = 9`. The planes
/// `z = ±3` cut both in one circle; `z = 2` cuts the cone in a circle of radius
/// two and shares nothing with the pipe.
#[test]
fn a_bar_and_a_cone_about_one_axis_share_a_circle_on_either_nappe_and_nothing_between() {
    let upright = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 3.0);
    let pointed = cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);

    for height in [3.0, -3.0] {
        let cap = flat([0.0, 0.0, 1.0], [0.0, 0.0, height]);
        for corner in [[3.0, 0.0], [0.0, -3.0]] {
            let point = PredicatePoint::from([corner[0], corner[1], height]);
            for surface in [&cap, &upright, &pointed] {
                assert_eq!(
                    which_side_of(surface, point),
                    Decision::Zero,
                    "{point:?} is exactly on {surface:?}"
                );
            }
        }
        assert_eq!(
            meet([&cap, &upright, &pointed]),
            Ok(MeetOfThree::MoreThanAPoint),
            "the plane z = {height}"
        );
    }
    let between = flat([0.0, 0.0, 1.0], [0.0, 0.0, 2.0]);
    assert_eq!(
        meet([&between, &upright, &pointed]),
        Ok(MeetOfThree::Nowhere)
    );
}

/// **A plane through a cone's apex, square to its axis, touches the cone there
/// and nowhere else**, so the three meet at the apex or not at all.
///
/// A ball of radius five resting on the floor at the origin, from above or from
/// below, touches it at the apex: one corner, the stored apex itself, exactly
/// on all three. A pipe of radius three about the same axis does not pass
/// through its own axis, so it shares nothing with them.
#[test]
fn a_plane_through_a_cones_apex_meets_a_ball_resting_there_at_the_apex_and_a_bar_nowhere() {
    let floor = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
    let pointed = cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);

    for centre in [5.0, -5.0] {
        let resting = ball([0.0, 0.0, centre], 5.0);
        assert_eq!(
            corners_of([&floor, &pointed, &resting]),
            vec![[0.0, 0.0, 0.0]],
            "a ball about (0, 0, {centre})"
        );
    }

    let upright = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 3.0);
    assert_eq!(meet([&floor, &pointed, &upright]), Ok(MeetOfThree::Nowhere));
}

/// **Every pair of balls, bars and cones about one axis, cut square at every
/// whole height, against the closed form** — along each of the three axes, the
/// axis off the origin, and each pipe and cone written pointing both ways.
///
/// A plane square to the axis cuts a ball of radius `R` about height `c` in a
/// circle of radius² `R² − (h − c)²`, a pipe in one of `r²`, and a cone of
/// `slope` with its apex at height `a` in one of `slope²(h − a)²`. Two circles
/// about one centre share all of themselves when the two numbers are equal and
/// above zero, the one centre when both are zero, and nothing otherwise. Every
/// number here is a whole number or a half, so the oracle rounds nothing.
///
/// Pairs that bend the same way — two balls, two pipes, two cones of one
/// slope — are the first road's and are swept anyway: the closed form does not
/// care which road answers, and two roads that disagreed would show here.
/// Every ball, pipe and cone the turned sweeps below ask about, built about
/// one coordinate axis and shared with
/// [`every_pair_about_one_axis_cut_square_to_it_meets_where_the_closed_form_says`]
/// so a turned sweep asks the same question of the same 630 pairs.
fn balls_pipes_and_cones_about_one_axis() -> Vec<AboutTheAxis> {
    let mut shapes = Vec::new();
    for centre in [-2.0, 0.0, 3.0] {
        for radius in [1.0, 2.0, 3.0, 5.0] {
            shapes.push(AboutTheAxis::Ball { centre, radius });
        }
    }
    for radius in [1.0, 2.0, 3.0] {
        for up in [true, false] {
            shapes.push(AboutTheAxis::Pipe { radius, up });
        }
    }
    for apex in [-1.0, 0.0, 2.0] {
        for slope in [0.5, 1.0, 2.0] {
            for up in [true, false] {
                shapes.push(AboutTheAxis::Cone { apex, slope, up });
            }
        }
    }
    shapes
}

#[test]
fn every_pair_about_one_axis_cut_square_to_it_meets_where_the_closed_form_says() {
    let shapes = balls_pipes_and_cones_about_one_axis();

    let (mut shared, mut at_one_point, mut nowhere) = (0usize, 0usize, 0usize);
    for along in 0..3 {
        for (first, one) in shapes.iter().enumerate() {
            for other in &shapes[first + 1..] {
                for height in -4..=4 {
                    let height = f64::from(height);
                    let facing = if height < 0.0 { -1.0 } else { 1.0 };
                    let cap = flat(
                        on_the_axis_direction(along, facing),
                        on_the_axis(along, height),
                    );
                    let (here, there) = (one.built(along), other.built(along));
                    let mine = one.squared_radius_at(height);
                    let theirs = other.squared_radius_at(height);
                    let answer = meet([&cap, &here, &there]);
                    let what = format!("{one:?} and {other:?} along axis {along}, cut at {height}");
                    if mine < 0.0 || theirs < 0.0 || mine != theirs {
                        assert_eq!(answer, Ok(MeetOfThree::Nowhere), "{what}");
                        nowhere += 1;
                    } else if mine > 0.0 {
                        assert_eq!(answer, Ok(MeetOfThree::MoreThanAPoint), "{what}");
                        shared += 1;
                    } else {
                        assert_eq!(
                            corners_of([&cap, &here, &there]),
                            vec![on_the_axis(along, height)],
                            "{what}"
                        );
                        at_one_point += 1;
                    }
                }
            }
        }
    }
    println!("{shared} shared a circle, {at_one_point} met at one point, {nowhere} nowhere");
    assert_eq!(
        shared + at_one_point + nowhere,
        3 * 630 * 9,
        "the sweep stopped short"
    );
    for (count, what) in [
        (shared, "shared a circle"),
        (at_one_point, "met at one point"),
        (nowhere, "met nowhere"),
    ] {
        assert!(count >= 30, "only {count} pairs {what}");
    }
}

/// A surface turned about the line `x = 2`, `y = −3` along one coordinate axis,
/// placed by how far along that axis it sits.
#[derive(Clone, Copy, Debug)]
enum AboutTheAxis {
    Ball { centre: f64, radius: f64 },
    Pipe { radius: f64, up: bool },
    Cone { apex: f64, slope: f64, up: bool },
}

impl AboutTheAxis {
    /// The surface, with its axis along coordinate axis `along`.
    fn built(self, along: usize) -> Surface {
        let which_way = |up: bool| on_the_axis_direction(along, if up { 1.0 } else { -1.0 });
        match self {
            AboutTheAxis::Ball { centre, radius } => ball(on_the_axis(along, centre), radius),
            AboutTheAxis::Pipe { radius, up } => {
                pipe(on_the_axis(along, 0.0), which_way(up), radius)
            }
            AboutTheAxis::Cone { apex, slope, up } => {
                cone(on_the_axis(along, apex), which_way(up), slope)
            }
        }
    }

    /// The squared radius of the circle the plane square to the axis at
    /// `height` cuts this in, below zero where it misses.
    fn squared_radius_at(self, height: f64) -> f64 {
        match self {
            AboutTheAxis::Ball { centre, radius } => {
                radius * radius - (height - centre) * (height - centre)
            }
            AboutTheAxis::Pipe { radius, .. } => radius * radius,
            AboutTheAxis::Cone { apex, slope, .. } => {
                slope * slope * (height - apex) * (height - apex)
            }
        }
    }
}

/// The point `how_far` along the line `x = 2`, `y = −3` turned onto coordinate
/// axis `along`.
fn on_the_axis(along: usize, how_far: f64) -> [f64; 3] {
    let mut place = [0.0; 3];
    place[along] = how_far;
    place[(along + 1) % 3] = 2.0;
    place[(along + 2) % 3] = -3.0;
    place
}

/// A direction of length `length` along coordinate axis `along`.
fn on_the_axis_direction(along: usize, length: f64) -> [f64; 3] {
    let mut direction = [0.0; 3];
    direction[along] = length;
    direction
}

/// **A D-flat on a shaft, across the circles a chamfer cone and a ball on its
/// axis cross the shaft in.** The flat runs beside the axis, parallel to it,
/// and cuts each circle in a chord.
///
/// A shaft of radius five up `z`, flattened at `x = 3`, which leaves `y = ±4`
/// on its skin. A cone of slope one with its apex at `z = 8` crosses the
/// shaft where `|z − 8| = 5`, at `z = 3` and `13`; a ball of radius 13 about
/// `(0, 0, 20)` crosses it where `(z − 20)² = 169 − 25`, at `z = 8` and `32`.
/// So the corners are `(3, ±4, h)` at each, whole numbers, lower first and
/// `+y` first at each, since `n × w` is `−y`.
#[test]
fn a_d_flat_on_a_shaft_meets_a_chamfer_cone_and_a_ball_on_its_axis_at_the_ends_of_chords() {
    let shaft = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);
    let d_flat = flat([1.0, 0.0, 0.0], [3.0, 0.0, 0.0]);
    let chamfer = cone([0.0, 0.0, 8.0], [0.0, 0.0, 1.0], 1.0);
    let round = ball([0.0, 0.0, 20.0], 13.0);

    for (other, heights) in [(&chamfer, [3.0, 13.0]), (&round, [8.0, 32.0])] {
        let wanted: Vec<[f64; 3]> = heights
            .iter()
            .flat_map(|&height| [[3.0, 4.0, height], [3.0, -4.0, height]])
            .collect();
        assert_eq!(corners_of([&d_flat, &shaft, other]), wanted, "{other:?}");
        assert_eq!(
            meet([other, &d_flat, &shaft]),
            meet([&shaft, other, &d_flat]),
            "the answer depends on which three they are, not the order"
        );
    }
}

/// **A flat beside the axis that touches one of the circles a cone and a ball
/// share, or misses both, though it cuts the cone and the ball.**
///
/// A cone of slope one with its apex at the origin and a ball of radius five
/// about `(0, 0, 7)` cross where `2t² − 14t + 24 = 0`, at `z = 3` and `4`, in
/// circles of radius three and four. The flat `x = 4` touches the upper circle
/// at `(4, 0, 4)` and misses the lower; the flat `x = 4.5` is inside the ball's
/// reach of five and misses both circles.
#[test]
fn a_flat_beside_a_cone_and_a_ball_on_its_axis_touches_one_of_their_circles_or_misses_both() {
    let pointed = cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
    let round = ball([0.0, 0.0, 7.0], 5.0);

    let touching = flat([1.0, 0.0, 0.0], [4.0, 0.0, 0.0]);
    assert_eq!(
        corners_of([&touching, &pointed, &round]),
        vec![[4.0, 0.0, 4.0]]
    );

    let missing = flat([1.0, 0.0, 0.0], [4.5, 0.0, 0.0]);
    assert_eq!(meet([&missing, &pointed, &round]), Ok(MeetOfThree::Nowhere));
}

/// **An axis that leans to the plane is not this road**, even with a ball
/// centred on it: the plane cuts the pipe in an ellipse, and an ellipse and a
/// circle are the quartic again. The road for curves that cross does not reach
/// it either. The ball crosses the pipe in two circles rather than touching it
/// along one, so no plane known exactly takes the pipe's place, and the floor
/// leans too far off square to the pipe for its ellipse to be a line bent by
/// little across the circle.
///
/// A pipe of radius twelve along `(0, 3, 4)` through the origin and a ball of
/// radius thirteen about `(0, 3, 4)`, cut by the floor `z = 0`. The `x` axis is
/// square to the pipe's axis whatever its stored direction rounded to, so
/// `(±12, 0, 0)` is on the pipe, and `144 + 9 + 16 = 169` puts it on the ball:
/// two corners, in whole numbers, that nothing here reaches.
#[test]
fn a_bar_leaning_to_the_plane_is_still_refused_though_a_ball_sits_on_its_axis() {
    let floor = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
    let leaning = pipe([0.0, 0.0, 0.0], [0.0, 3.0, 4.0], 12.0);
    let round = ball([0.0, 3.0, 4.0], 13.0);

    for corner in [[12.0, 0.0, 0.0], [-12.0, 0.0, 0.0]] {
        for surface in [&floor, &leaning, &round] {
            assert_eq!(
                which_side_of(surface, PredicatePoint::from(corner)),
                Decision::Zero,
                "{corner:?} is exactly on {surface:?}"
            );
        }
    }

    assert_eq!(
        meet([&floor, &leaning, &round]),
        Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
    );
}

/// **Three curved surfaces, two of which bend the same way, are a plane and
/// two curved surfaces** (Q1284). Two quadrics that square `p` the same way
/// differ by a plane: every point both hold is on it, and every point on it
/// and on one of them is on the other. So the plane takes the place of one of
/// the two, and the roads above answer the triple.
///
/// A cut cone less a cone closed by a ball, in `rcad-topo`'s
/// `a_joined_flat_face_whose_rings_enclose_no_area.rs`, asks this of a cone of
/// slope one up `z` with its apex at the origin, a second with its apex at
/// `z = 3`, and the ball of radius `√148` about `(0, 0, 15)`. The cones are
/// `x² + y² = z²` and `x² + y² = (z − 3)²`, which differ by `6z = 9`, the
/// plane `z = 1.5`, and the ball reaches down to `15 − √148`, about `2.83`, and
/// no lower. So the three share nothing.
#[test]
fn two_cones_of_one_slope_and_a_ball_above_the_plane_between_them_share_nothing() {
    let lower = cone([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 1.0);
    let upper = cone([0.0, 0.0, 3.0], [0.0, 0.0, 1.0], 1.0);
    let round = ball([0.0, 0.0, 15.0], 148.0_f64.sqrt());

    for triple in [
        [&lower, &upper, &round],
        [&round, &lower, &upper],
        [&upper, &round, &lower],
    ] {
        assert_eq!(meet(triple), Ok(MeetOfThree::Nowhere), "{triple:?}");
    }
}

/// **Where the plane two of them differ by does cut the third**, by two of
/// the roads. Each corner is checked exactly on all three surfaces.
///
/// - Three cones of slope one up `z`, with their apexes at the origin, at
///   `z = 10` and at `(6, 0, 0)`. The first two differ by `z = 5`, where each
///   is the circle `x² + y² = 25`, and the third is `(x − 6)² + y² = 25`
///   there: all three bend the same way, so the plane and the first and third
///   cones are two planes and a cone, and they meet at `(3, ±4, 5)`.
/// - Two bars of radius five up `z`, six apart, and a ball of radius 13 about
///   the origin. The bars differ by `x = 3`, beside the axis the first bar
///   and the ball share, and those two share the circles of radius five at
///   `z = ±12`. The plane cuts each at `y = ±4`: four corners, `(3, ±4, ±12)`.
#[test]
fn three_curved_surfaces_meet_where_the_plane_two_of_them_differ_by_cuts_the_third() {
    let up = [0.0, 0.0, 1.0];
    let cones = [
        cone([0.0, 0.0, 0.0], up, 1.0),
        cone([0.0, 0.0, 10.0], up, 1.0),
        cone([6.0, 0.0, 0.0], up, 1.0),
    ];
    let bars_and_a_ball = [
        pipe([0.0, 0.0, 0.0], up, 5.0),
        pipe([6.0, 0.0, 0.0], up, 5.0),
        ball([0.0, 0.0, 0.0], 13.0),
    ];

    for (triple, wanted) in [
        (&cones, vec![[3.0, -4.0, 5.0], [3.0, 4.0, 5.0]]),
        (
            &bars_and_a_ball,
            vec![
                [3.0, 4.0, -12.0],
                [3.0, -4.0, -12.0],
                [3.0, 4.0, 12.0],
                [3.0, -4.0, 12.0],
            ],
        ),
    ] {
        let [one, other, third] = triple.each_ref();
        assert_eq!(corners_of([one, other, third]), wanted, "{triple:?}");
        for order in [
            [third, one, other],
            [other, third, one],
            [other, one, third],
        ] {
            assert_eq!(
                meet(order),
                meet([one, other, third]),
                "the answer depends on which three they are, not the order"
            );
        }
    }
}

/// **Two cones of a slope whose square no `f64` holds still differ by a
/// stored plane.** Cones of slope `s` up `z`, with their apexes at heights
/// `h₁` and `h₂`, differ by `2s²(h₁ − h₂)·z = s²(h₁² − h₂²)`, each side
/// several `f64` long where `s²` is, and the plane is `z = (h₁ + h₂)/2`
/// whatever `s` is. A bar of radius 100 about the same axis is wider there
/// than the cones' circle, so each three share nothing.
///
/// Slopes `0.01` to `0.99` by hundredths, against 38 pairs of whole heights
/// with the lower one below 4 and the upper below 12: 3762 triples, all
/// answered. With the two sides rounded to the nearest `f64` before
/// [`Plane::from_coefficients`] divides them, rather than divided exactly
/// first, 641 of them were refused, the stored plane an ulp off the true one
/// (Q1284, the division swapped out by hand and the sweep counted).
#[test]
fn two_cones_of_a_slope_no_f64_squares_still_differ_by_a_stored_plane() {
    let bar = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 100.0);
    for hundredths in 1..100 {
        let slope = f64::from(hundredths) / 100.0;
        for low in 0..4 {
            for high in (low + 1)..12 {
                let lower = cone([0.0, 0.0, f64::from(low)], [0.0, 0.0, 1.0], slope);
                let upper = cone([0.0, 0.0, f64::from(high)], [0.0, 0.0, 1.0], slope);
                assert_eq!(
                    meet([&lower, &upper, &bar]),
                    Ok(MeetOfThree::Nowhere),
                    "slope {slope}, apexes at {low} and {high}"
                );
            }
        }
    }
}

/// **Two quadrics that bend the same way and differ by a number share
/// nothing**, and neither does any third surface with them. Two bars up `z`
/// of radius three and five are `x² + y² = 9` and `x² + y² = 25`, and `0 = 16`
/// holds nowhere.
#[test]
fn two_bars_on_one_axis_with_two_radii_and_a_ball_share_nothing() {
    let thin = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 3.0);
    let thick = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0);
    let round = ball([4.0, 0.0, 0.0], 5.0);
    assert_eq!(meet([&thin, &thick, &round]), Ok(MeetOfThree::Nowhere));
    assert_eq!(meet([&round, &thick, &thin]), Ok(MeetOfThree::Nowhere));
}

/// **One surface written twice is no plane**, so a triple with a repeat in it
/// is refused as it was before any pair was read as a plane, whether or not
/// the third bends the same way.
#[test]
fn a_surface_written_twice_beside_a_third_curved_one_is_still_refused() {
    let round = ball([0.0, 0.0, 0.0], 5.0);
    for third in [
        pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 3.0),
        ball([6.0, 0.0, 0.0], 5.0),
    ] {
        assert_eq!(
            meet([&round, &round, &third]),
            Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat),
            "{third:?}"
        );
        assert_eq!(
            meet([&third, &round, &round]),
            Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat),
            "{third:?}"
        );
    }
}

/// **A plane no stored plane is exactly is still refused, and that is a gap.**
/// Two balls of radius five about the origin and `(3, 3, 0)` differ by
/// `6x + 6y = 18`, the plane `x + y = 3`. A stored plane's normal has length
/// one, `(1/√2, 1/√2, 0)` rounded, and its offset is `3/√2` rounded, and
/// three times that rounded normal is not an `f64`, so no stored plane is
/// `x + y = 3`. A bar of radius three up `z` with them meets both balls at
/// `(3, 0, ±4)` and `(0, 3, ±4)`, whole numbers on all three, and the points
/// are there.
#[test]
fn two_balls_whose_plane_no_stored_plane_is_and_a_bar_are_still_refused() {
    let here = ball([0.0, 0.0, 0.0], 5.0);
    let there = ball([3.0, 3.0, 0.0], 5.0);
    let bar = pipe([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 3.0);

    for corner in [
        [3.0, 0.0, 4.0],
        [3.0, 0.0, -4.0],
        [0.0, 3.0, 4.0],
        [0.0, 3.0, -4.0],
    ] {
        for surface in [&here, &there, &bar] {
            assert_eq!(
                which_side_of(surface, PredicatePoint::from(corner)),
                Decision::Zero,
                "{corner:?} is exactly on {surface:?}"
            );
        }
    }

    assert_eq!(
        meet([&here, &there, &bar]),
        Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
    );
}

/// **Turning the plane and both shapes by one shared [`Isometry`] leaves the
/// closed form's whole balls-pipes-cones sweep refused nowhere**, and every
/// one of the handful of rows it answers differently is a row the closed
/// form itself already calls a boundary — `mine == theirs` on the heights
/// the shapes were built at, before anything was turned, where the expected
/// answer is a shared circle or exactly one point and a rounding can tip the
/// answer either way. Never on a row the closed form calls `Nowhere`.
///
/// The three turns are the ones Q2099's row named: `0.3` about `z`, `1.1`
/// about `(1,2,3)` through `(1,0,-1)`, and `pi/6` about `y`. For each, the
/// cap's normal is also checked bit-for-bit against the axis direction of
/// every pipe or cone it is capping — the same shared turn applied to both,
/// so nothing here rounds the squareness away, which is why **no triple
/// refuses** below: the roads this file answers for balls, pipes and cones
/// about one axis all lean on that squareness.
///
/// [`meet`]'s own doc comment is this file's "nothing was fudged" proof: a
/// certified `Ok` carries an empty provenance log, and that already covers
/// every disagreement below, whichever variant it answers. A disagreement
/// that lands `At` one or more points is **not** also checked
/// [`corners_of`]'s way, on an exact `Decision::Zero` and a box of no
/// width: unlike every hand-worked corner elsewhere in this file, a point an
/// *irrational* turn puts a quadric's root at is not an `f64` either, so its
/// own certified enclosure is a tiny box rather than an exact one — checked
/// here only for staying tiny, the same bound the file's own
/// `a_cap_holding_a_rings_axis_meets_a_ball_or_a_cone_on_it_at_their_circles`
/// gives a hand-worked irrational corner.
#[test]
fn turning_the_whole_sweep_by_one_isometry_disagrees_with_the_closed_form_only_on_boundary_rows() {
    let shapes = balls_pipes_and_cones_about_one_axis();

    for (turn, disagreements_wanted) in [
        (
            rcad_geom::Isometry::rotation(
                &Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0)).expect("a direction"),
                0.3,
            )
            .expect("a finite angle"),
            725,
        ),
        (
            rcad_geom::Isometry::rotation(
                &Axis::new(Point::new(1.0, 0.0, -1.0), Vector::new(1.0, 2.0, 3.0))
                    .expect("a direction"),
                1.1,
            )
            .expect("a finite angle"),
            1119,
        ),
        (
            rcad_geom::Isometry::rotation(
                &Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("a direction"),
                core::f64::consts::PI / 6.0,
            )
            .expect("a finite angle"),
            725,
        ),
    ] {
        let (mut counted, mut disagreed) = (0usize, 0usize);
        for along in 0..3 {
            for (first, one) in shapes.iter().enumerate() {
                for other in &shapes[first + 1..] {
                    for height in -4..=4 {
                        let height = f64::from(height);
                        let facing = if height < 0.0 { -1.0 } else { 1.0 };
                        let cap = flat(
                            on_the_axis_direction(along, facing),
                            on_the_axis(along, height),
                        )
                        .transformed(&turn);
                        let here = one.built(along).transformed(&turn);
                        let there = other.built(along).transformed(&turn);
                        counted += 1;

                        if let Surface::Plane(plane) = &cap {
                            let normal = plane.normal();
                            for shape in [&here, &there] {
                                let direction = if let Surface::Cylinder(this) = shape {
                                    Some(this.axis().direction())
                                } else if let Surface::Cone(this) = shape {
                                    Some(this.axis().direction())
                                } else {
                                    None
                                };
                                if let Some(direction) = direction {
                                    assert_eq!(
                                        (normal.x().abs(), normal.y().abs(), normal.z().abs()),
                                        (
                                            direction.x().abs(),
                                            direction.y().abs(),
                                            direction.z().abs()
                                        ),
                                        "the turned cap is not exactly square to {shape:?}, \
                                         up to which way each points"
                                    );
                                }
                            }
                        }

                        let mine = one.squared_radius_at(height);
                        let theirs = other.squared_radius_at(height);
                        let on_a_boundary = mine >= 0.0 && theirs >= 0.0 && mine == theirs;
                        let expected = if !on_a_boundary {
                            "nowhere"
                        } else if mine > 0.0 {
                            "shared"
                        } else {
                            "one"
                        };

                        let answer = meet([&cap, &here, &there]).unwrap_or_else(|refused| {
                            panic!(
                                "{one:?} and {other:?} along axis {along}, cut at {height}, \
                                 turned: {refused:?}"
                            )
                        });
                        let actual = match &answer {
                            MeetOfThree::Nowhere => "nowhere",
                            MeetOfThree::MoreThanAPoint => "shared",
                            MeetOfThree::At(points) if points.len() == 1 => "one",
                            MeetOfThree::At(_) => "other",
                            other => panic!("an answer this test does not know: {other:?}"),
                        };
                        if actual == expected {
                            continue;
                        }
                        assert!(
                            on_a_boundary,
                            "{one:?} and {other:?} along axis {along}, cut at {height}, turned: \
                             expected {expected} off the closed form but turning answered \
                             {answer:?}, though mine = {mine} and theirs = {theirs} do not even \
                             agree before anything was turned"
                        );
                        disagreed += 1;
                        if let MeetOfThree::At(points) = &answer {
                            for point in points {
                                for side in point.enclosure() {
                                    assert!(
                                        side.width() <= 1e-9,
                                        "{point:?} is a box far too wide to be the single \
                                         irrational corner a boundary row turns onto"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(counted, 17010, "the sweep stopped short");
        assert_eq!(
            disagreed, disagreements_wanted,
            "how many rows this turn moves off the closed form changed"
        );
    }
}

/// **A cap turned by a different composition of turns from its pair rounds
/// its stored axis off square. Where the closed form says the three share
/// nothing it is answered nowhere, and where it says one circle it is answered
/// at two points or nowhere, and never refused.**
///
/// [`Isometry::compose`]'s own doc comment says why this is a real path and
/// not a contrived one: "Any other two turns are multiplied as quaternions,
/// which rounds", and names `rcad-topo`'s `Model::place_solid` — which
/// composes each call's movement onto the one a part already carries — as
/// exactly the caller that does this on every placement. Two half-turns
/// composed are not bit-identical to the one direct turn of the same angle
/// (`zz_q2099_probe_composed_turn` measured a one-ulp difference on a unit
/// vector component before this test replaced it; not re-measured here).
///
/// The pair — a ball, a pipe or a cone about one axis, built and checked
/// against [`AboutTheAxis::squared_radius_at`] the same way the closed-form
/// sweep above is — is turned directly by one [`Isometry::rotation`] about
/// the oblique axis `(1,2,3)` through `(1,0,-1)`, angle `1.1`; the cap that
/// should cut both square to that same axis is turned by two half-turns of
/// it composed instead. The cap's normal is checked, for every pipe or cone
/// in the sweep, against the shape's own turned axis direction — on both a
/// cylinder's and a cone's, since the one-off probe this replaces checked
/// only the cylinder's — and found not bit-exactly equal at least once,
/// which is the rounding-off-square case Q988 asked for and Q2093's
/// same-isometry construction above never makes.
///
/// The roads for circles need that squareness and hand such a triple on. Where
/// the closed form says the three share nothing, either the cap misses a ball,
/// which settles it alone, or the two cuts differ by far more than a rounding,
/// and the cap a rounding off square only widens each cut into a band about a
/// rounding wide. So the bands keep apart, and every such row is answered
/// nowhere: the test asserts it. Where the closed form says one shared
/// circle, the rounding leaves two near-equal curves whose bands overlap. The
/// road for bands that cross answers those at two points where the curves
/// cross, each a box no more than `1e-12` wide that no surface is certified
/// off, and nowhere where one curve keeps inside the other. A ball and a pipe
/// or a cone that touch along a circle, the ball's centre on the axis, have
/// curves that touch rather than cross, and those rows are answered through
/// the plane of that circle, which is known exactly. Ball-ball and pipe-pipe
/// go to an earlier road, since two balls, or two pipes running the same way,
/// differ by something linear. Pinned in
/// [`ROUNDING_OFF_SQUARE_CAP_ANSWERS`] as measured, the same way
/// [`TANGENT_ROWS_A_TURN_LEAVES_UNDECIDED`] pins a count no formula derives —
/// a change here says the rounding moved, not that it is wrong.
#[test]
fn a_cap_turned_by_a_different_composition_of_turns_than_its_pair_is_answered_in_every_family() {
    let shapes = balls_pipes_and_cones_about_one_axis();

    let axis =
        Axis::new(Point::new(1.0, 0.0, -1.0), Vector::new(1.0, 2.0, 3.0)).expect("a direction");
    let angle = 1.1_f64;
    let direct = rcad_geom::Isometry::rotation(&axis, angle).expect("a finite angle");
    let half = rcad_geom::Isometry::rotation(&axis, angle / 2.0).expect("a finite angle");
    let composed = half.compose(&half);

    let along = 0usize;
    let families = [
        "ball-ball",
        "ball-pipe",
        "ball-cone",
        "pipe-pipe",
        "pipe-cone",
        "cone-cone",
    ];
    let mut not_square = 0usize;
    let mut answered = [[0usize; 3]; 6];
    for (first, one) in shapes.iter().enumerate() {
        for other in &shapes[first + 1..] {
            let family = match (one, other) {
                (AboutTheAxis::Ball { .. }, AboutTheAxis::Ball { .. }) => 0,
                (AboutTheAxis::Ball { .. }, AboutTheAxis::Pipe { .. })
                | (AboutTheAxis::Pipe { .. }, AboutTheAxis::Ball { .. }) => 1,
                (AboutTheAxis::Ball { .. }, AboutTheAxis::Cone { .. })
                | (AboutTheAxis::Cone { .. }, AboutTheAxis::Ball { .. }) => 2,
                (AboutTheAxis::Pipe { .. }, AboutTheAxis::Pipe { .. }) => 3,
                (AboutTheAxis::Pipe { .. }, AboutTheAxis::Cone { .. })
                | (AboutTheAxis::Cone { .. }, AboutTheAxis::Pipe { .. }) => 4,
                (AboutTheAxis::Cone { .. }, AboutTheAxis::Cone { .. }) => 5,
            };
            for height in -4..=4 {
                let height = f64::from(height);
                let facing = if height < 0.0 { -1.0 } else { 1.0 };
                let cap = flat(
                    on_the_axis_direction(along, facing),
                    on_the_axis(along, height),
                )
                .transformed(&composed);
                let here = one.built(along).transformed(&direct);
                let there = other.built(along).transformed(&direct);

                let Surface::Plane(plane) = &cap else {
                    panic!("a plane turned is still a plane")
                };
                let normal = plane.normal();
                for shape in [&here, &there] {
                    let direction = if let Surface::Cylinder(this) = shape {
                        Some(this.axis().direction())
                    } else if let Surface::Cone(this) = shape {
                        Some(this.axis().direction())
                    } else {
                        None
                    };
                    if let Some(direction) = direction
                        && (normal.x().abs(), normal.y().abs(), normal.z().abs())
                            != (
                                direction.x().abs(),
                                direction.y().abs(),
                                direction.z().abs(),
                            )
                    {
                        not_square += 1;
                    }
                }

                let mine = one.squared_radius_at(height);
                let theirs = other.squared_radius_at(height);
                let answer = meet([&cap, &here, &there]);
                let what = format!("{one:?} and {other:?} cut at {height}, cap rounded off square");
                if mine < 0.0 || theirs < 0.0 || mine != theirs {
                    assert_eq!(answer, Ok(MeetOfThree::Nowhere), "{what}");
                    continue;
                }
                let answer = answer.unwrap_or_else(|refused| panic!("{what}: {refused:?}"));
                let column = match &answer {
                    MeetOfThree::At(points) => {
                        assert_eq!(points.len(), 2, "{what}: {answer:?}");
                        for point in points {
                            for side in point.enclosure() {
                                assert!(side.width() <= 1e-12, "{what}: {point:?}");
                            }
                            for surface in [&cap, &here, &there] {
                                assert!(
                                    !matches!(
                                        which_side_of(surface, *point),
                                        Decision::Pos | Decision::Neg
                                    ),
                                    "{what}: {point:?} is certified off {surface:?}"
                                );
                            }
                        }
                        0
                    }
                    MeetOfThree::Nowhere => 1,
                    MeetOfThree::MoreThanAPoint => 2,
                    other => panic!("{what}: {other:?}"),
                };
                answered[family][column] += 1;
            }
        }
    }

    assert!(
        not_square > 0,
        "the composed cap came out exactly square to every pair's axis after all"
    );
    for (name, [at_two_points, nowhere, more]) in families.iter().zip(answered) {
        println!("{name}: two points {at_two_points}, nowhere {nowhere}, more than a point {more}");
    }
    assert_eq!(
        answered, ROUNDING_OFF_SQUARE_CAP_ANSWERS,
        "how a cap rounded off square is answered, by family, moved"
    );
}

/// Measured, and a watch rather than a bound (see
/// [`a_cap_turned_by_a_different_composition_of_turns_than_its_pair_is_answered_in_every_family`]):
/// how a cap one rounding off square is answered on the rows the closed form
/// calls one shared circle or one point, as `[two points, nowhere, more than a
/// point]`, by family, in the order ball-ball, ball-pipe, ball-cone,
/// pipe-pipe, pipe-cone, cone-cone. None is refused. Before the road for
/// bands that cross it refused `[0, 22, 40, 0, 104, 64]` of them, and before
/// the road for bands apart `[0, 366, 1098, 0, 972, 972]`.
const ROUNDING_OFF_SQUARE_CAP_ANSWERS: [[usize; 3]; 6] = [
    [3, 9, 0],
    [22, 4, 0],
    [34, 44, 0],
    [0, 0, 27],
    [92, 44, 0],
    [60, 52, 81],
];

/// **A cap leaning a hundredth off square to a bar's axis cuts the bar, a
/// chamfer cone and a ball in curves that keep apart.** The cap through
/// `(0, 0, 5)` with normal `(0, 1, 100)` leans by `t = atan(1/100)` and cuts
/// the bar of radius three up `z` in an ellipse whose every point is between
/// `3` and `3/cos t` from where the axis crosses it, `3.00015` at most. It cuts
/// the cone of slope one from the origin, which is five across there, between
/// `5/(1 + sin t)` and `5/(cos t − sin t)`, `4.95` at least, and the ball of
/// radius two about that point in a circle of radius two. So no point is on all
/// three, and no quartic is needed to say so.
///
/// The bar of radius five against the same cone has bands that overlap, and
/// its ellipse crosses the cone's at `(±5, 0, 5)`, where the cap crosses the
/// circle of radius five the two share. The cap is stored with its normal
/// scaled to length one, which rounds, so the points come back within `1e-13`
/// of those and not on them.
#[test]
fn a_cap_leaning_off_square_to_a_bar_meets_it_and_a_cone_or_a_ball_it_keeps_apart_nowhere() {
    let cap = flat([0.0, 1.0, 100.0], [0.0, 0.0, 5.0]);
    let bar = pipe([0.0; 3], [0.0, 0.0, 1.0], 3.0);
    let chamfer = cone([0.0; 3], [0.0, 0.0, 1.0], 1.0);
    let steeper = cone([0.0; 3], [0.0, 0.0, -1.0], 2.0);
    let small_ball = ball([0.0, 0.0, 5.0], 2.0);
    for (one, other) in [
        (&bar, &chamfer),
        (&bar, &small_ball),
        (&chamfer, &steeper),
        (&chamfer, &small_ball),
    ] {
        for order in [[&cap, one, other], [other, &cap, one], [one, other, &cap]] {
            assert_eq!(meet(order), Ok(MeetOfThree::Nowhere), "{order:?}");
        }
    }

    let wide_bar = pipe([0.0; 3], [0.0, 0.0, 1.0], 5.0);
    for order in [
        [&cap, &wide_bar, &chamfer],
        [&chamfer, &cap, &wide_bar],
        [&wide_bar, &chamfer, &cap],
    ] {
        two_points_near(order, [[-5.0, 0.0, 5.0], [5.0, 0.0, 5.0]], 0);
    }
}

/// The two points `surfaces` meet at, sorted along `sorted_along`, each a box
/// at most `1e-13` wide and within `1e-13` of its corner, and neither certified
/// off any of the three.
///
/// **Within, not on**: the corners are worked out for the plane as written,
/// and the stored plane is its normal scaled to length one, which rounds. So
/// the true corner of what is stored is a rounding or two from the one worked
/// out by hand, and an exact comparison would test the rounding.
fn two_points_near(surfaces: [&Surface; 3], corners: [[f64; 3]; 2], sorted_along: usize) {
    let answer = meet(surfaces).expect("these three meet");
    let MeetOfThree::At(points) = &answer else {
        panic!("{surfaces:?}: {answer:?}");
    };
    assert_eq!(points.len(), 2, "{surfaces:?}: {answer:?}");
    let mut points = points.to_vec();
    points.sort_by(|one, other| {
        one.enclosure()[sorted_along]
            .lo()
            .total_cmp(&other.enclosure()[sorted_along].lo())
    });
    for (point, corner) in points.iter().zip(corners) {
        for (side, expected) in point.enclosure().into_iter().zip(corner) {
            assert!(
                side.width() <= 1e-13
                    && side.lo() - 1e-13 <= expected
                    && expected <= side.hi() + 1e-13,
                "{surfaces:?}: {point:?} where the corner is {corner:?}"
            );
        }
        for surface in surfaces {
            assert!(
                !matches!(
                    which_side_of(surface, *point),
                    Decision::Pos | Decision::Neg
                ),
                "{surfaces:?}: {point:?} is certified off {surface:?}"
            );
        }
    }
}

/// **A slanted cut through the circle where a rod's round end meets the rod.**
/// The rod is the pipe of radius one up `z`, its end the ball of radius one
/// about the origin, and the two touch along the circle `z = 0` rather than
/// cross. The cut `x + z = 0` leans to the rod's axis, so it cuts the rod in an
/// ellipse and the ball in a circle that touch at the two points where the cut
/// crosses that circle, `(0, ±1, 0)`. Two curves that touch are the case the
/// road for curves that cross cannot tell from two crossings close together.
#[test]
fn a_slanted_cut_through_where_a_rods_round_end_meets_it_meets_both_at_two_corners() {
    let cut = flat([1.0, 0.0, 1.0], [0.0; 3]);
    let rod = pipe([0.0; 3], [0.0, 0.0, 1.0], 1.0);
    let end = ball([0.0; 3], 1.0);
    for order in [[&cut, &rod, &end], [&end, &cut, &rod], [&rod, &end, &cut]] {
        two_points_near(order, [[0.0, -1.0, 0.0], [0.0, 1.0, 0.0]], 1);
    }
}

/// **A ball resting in a cone, and a plane leaning across the circle it rests
/// on.** The cone of slope `3/4` up `z` from the origin and the ball of radius
/// three about `(0, 0, 5)` touch along the circle of radius `2.4` at height
/// `3.2`, where `(25/16)z² − 10z + 16`, the difference of the two, is a
/// square. The cut `x + z = 3.2` crosses that circle at `(0, ±2.4, 3.2)`.
#[test]
fn a_plane_across_the_circle_a_ball_rests_on_in_a_cone_meets_both_there() {
    let cut = flat([1.0, 0.0, 1.0], [0.0, 0.0, 3.2]);
    let cone_of_it = cone([0.0; 3], [0.0, 0.0, 1.0], 0.75);
    let resting = ball([0.0, 0.0, 5.0], 3.0);
    for order in [
        [&cut, &cone_of_it, &resting],
        [&resting, &cut, &cone_of_it],
        [&cone_of_it, &resting, &cut],
    ] {
        two_points_near(order, [[0.0, -2.4, 3.2], [0.0, 2.4, 3.2]], 1);
    }
}

/// A ring about `z` through the origin.
fn ring(ring_radius: f64, tube_radius: f64) -> Surface {
    Surface::Torus(Torus::new(Axis::z(), ring_radius, tube_radius).expect("a tube inside its ring"))
}

/// **A bend's end cap across the bar of its ring's radius on its axis**: the
/// bar crosses the tube in the circles on its top and its bottom, and the cap,
/// a plane holding the axis, cuts each circle either side of the axis. Ring
/// four, tube one, and the cap `y = 0`, meet at `(±4, 0, ±1)`; ring six, tube
/// two, and the cap `x = 0`, at `(0, ±6, ±2)`. Lower circles first along `+z`,
/// and on each the `−(n × z)` side first.
///
/// Refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` before row Q1376, which
/// stopped every fuse of such a bend and bar at its caps.
#[test]
fn a_bends_cap_meets_a_bar_of_its_rings_radius_where_the_bars_circles_cross_it() {
    for (cap, bar, bend, corners) in [
        (
            flat([0.0, 1.0, 0.0], [0.0; 3]),
            pipe([0.0, 0.0, -2.0], [0.0, 0.0, 1.0], 4.0),
            ring(4.0, 1.0),
            [
                [-4.0, 0.0, -1.0],
                [4.0, 0.0, -1.0],
                [-4.0, 0.0, 1.0],
                [4.0, 0.0, 1.0],
            ],
        ),
        (
            flat([-1.0, 0.0, 0.0], [0.0; 3]),
            pipe([0.0, 0.0, -4.0], [0.0, 0.0, 1.0], 6.0),
            ring(6.0, 2.0),
            [
                [0.0, -6.0, -2.0],
                [0.0, 6.0, -2.0],
                [0.0, -6.0, 2.0],
                [0.0, 6.0, 2.0],
            ],
        ),
    ] {
        for order in [
            [&cap, &bar, &bend],
            [&bend, &cap, &bar],
            [&bar, &bend, &cap],
        ] {
            assert_eq!(corners_of(order), corners, "{order:?}");
        }
    }
}

/// The same road for a ball and a cone on the ring's axis, whose circles sit
/// at heights no `f64` holds. The ball of radius four about the centre of ring
/// four, tube one, crosses the tube at `(31/8, ±√63/8)`, and the cone of slope
/// four from the centre at `h = ±15/17` and `±1`, `4|h|` out. Each corner is a
/// box a few roundings wide holding the corner worked out by hand.
///
/// A pipe that misses the tube shares nothing with the ring, and a ball off
/// the axis is not this road and is refused as before.
#[test]
fn a_cap_holding_a_rings_axis_meets_a_ball_or_a_cone_on_it_at_their_circles() {
    let cap = flat([0.0, 1.0, 0.0], [0.0; 3]);
    let bend = ring(4.0, 1.0);
    let the_balls = 63f64.sqrt() / 8.0;
    let (fifteen, sixty) = (15.0 / 17.0, 60.0 / 17.0);
    for (other, by_hand) in [
        (
            ball([0.0; 3], 4.0),
            vec![
                [-3.875, -the_balls],
                [3.875, -the_balls],
                [-3.875, the_balls],
                [3.875, the_balls],
            ],
        ),
        (
            cone([0.0; 3], [0.0, 0.0, 1.0], 4.0),
            vec![
                [-4.0, -1.0],
                [4.0, -1.0],
                [-sixty, -fifteen],
                [sixty, -fifteen],
                [-sixty, fifteen],
                [sixty, fifteen],
                [-4.0, 1.0],
                [4.0, 1.0],
            ],
        ),
    ] {
        let answer = meet([&cap, &other, &bend]).expect("these three meet");
        let MeetOfThree::At(points) = &answer else {
            panic!("{other:?}: {answer:?}");
        };
        assert_eq!(points.len(), by_hand.len(), "{other:?}: {answer:?}");
        for (point, [x, z]) in points.iter().zip(by_hand) {
            let sides = point.enclosure();
            for (side, expected) in [(sides[0], x), (sides[1], 0.0), (sides[2], z)] {
                assert!(
                    side.width() <= 1e-14
                        && side.lo() - 1e-15 <= expected
                        && expected <= side.hi() + 1e-15,
                    "{other:?}: {point:?} where the corner is ({x}, 0, {z})"
                );
            }
        }
    }

    assert_eq!(
        meet([&cap, &pipe([0.0; 3], [0.0, 0.0, 1.0], 2.0), &bend]),
        Ok(MeetOfThree::Nowhere)
    );
    assert_eq!(
        meet([&cap, &ball([0.0, 1e-9, 0.0], 4.0), &bend]),
        Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
    );
}
