//! Where two curves of one pair of surfaces meet, and whether a fuse gets there.
//!
//! `intersect` names two curves for some pairs of surfaces: two lines where a
//! plane or a second pipe runs along a pipe, two circles round one axis, the
//! two branches of a hyperbola, two loops. `place_the_meets` in
//! `fuse/arrange.rs` puts a node where two of those curves meet for two pairs:
//! a plane's two rulings through a cone's apex (§347.1), and the four half
//! ellipses of two pipes of one radius whose axes meet (§347.9). §347.5 listed
//! the other pairs and said no shape had been built that brought two of their
//! curves together. This file builds each of them, as near to a meet as the
//! kernel lets it go, and pins what happens. §347.9 has the table.
//!
//! Two curves of one pair can only share a point where the two surfaces are
//! tangent, or where one of them has no normal. Away from such points the
//! crossing is a smooth curve, one piece through each point. So each family
//! here reaches its meet only at its tangent limit, and the tests below show
//! what the kernel does there: `intersect` names one curve where there were
//! two, or cannot make the surface at all. A pipe resting on the inside of a
//! thick pipe or of a ball is tangent at one point, and there `intersect` names
//! its two loops and the arrangement puts a node where they meet, so every
//! fuse of it answers. The one surface with a point and no normal is a cone,
//! at its apex, and that is the node §347.1 built. Two pipes of one radius
//! whose axes meet are tangent at two points, and their crossing reaches them
//! without being pushed there: that is the Steinmetz cross, which gets its
//! nodes, and every fuse of it answers.
//!
//! The fuses run three rules both ways round, certified unless a test names a
//! tolerant policy, in a model of 100 mm.

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, DynamicRange, Isometry, Plane, Point, Sphere, Surface,
    SurfaceMeet, Torus, Vector, intersect,
};
use rcad_predicates::expansion::{expansion_sign, expansion_sum, scale_expansion};
use rcad_predicates::{Decision, Policy, ProvenanceLog, Record};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

mod cones;
mod rings;

use std::f64::consts::{PI, SQRT_2};

type Pair = fn(&mut Model) -> [SolidKey; 2];

/// The bar of radius 2 up `z` from 0 to 4, and the block `[1, 5] × [−5, 5] ×
/// [1, 3]`. The block's face `x = 1` cuts the pipe in two lines `√3` either
/// side of `y = 0`. They come together only for a face `x = 2`, resting on the
/// pipe (`a_plane_resting_on_a_pipe_touches_it_along_one_line`).
///
/// What they share is the slice of the disc past `x = 1`, `4π/3 − √3`, two
/// high.
#[test]
fn a_block_along_a_bar_cuts_its_pipe_in_two_lines_and_every_fuse_answers() {
    let shared = 2.0 * (4.0 * PI / 3.0 - 3f64.sqrt());
    every_fuse_answers(
        a_block_along_a_bar,
        [16.0 * PI, 80.0, shared],
        [&[14], &[4], &[10], &[8]],
    );
}

/// Two bars of radius 2, one up `z` from 0 to 4 and one at `(2, 0)` from 1 to
/// 3. The pipes cross in two lines at `x = 1`, `y = ±√3`, which come together
/// only where the axes are `4` or `0` apart: one line, or one pipe
/// (`two_pipes_resting_side_by_side_touch_along_one_line`).
///
/// What they share is a lens of two discs of radius 2 whose centres are 2
/// apart, `8π/3 − 2√3`, two high.
#[test]
fn two_bars_side_by_side_cross_in_two_lines_and_every_fuse_answers() {
    let shared = 2.0 * (8.0 * PI / 3.0 - 2.0 * 3f64.sqrt());
    every_fuse_answers(
        two_bars_side_by_side,
        [16.0 * PI, 8.0 * PI, shared],
        [&[10], &[4], &[10], &[4]],
    );
}

/// A ball of radius 2 at the origin and a bar of radius 1 down its axis, from
/// `z = −3` to 3. The two cross in circles at `z = ±√3`, which come together
/// only for a bar as wide as the ball
/// (`a_ball_on_a_pipes_axis_as_wide_as_the_pipe_touches_it_round_one_circle`).
///
/// The bar holds `(4π/3)(2³ − (2² − 1)^(3/2))` of the ball. The bar less the
/// ball is its two ends, three faces each.
#[test]
fn a_ball_on_a_bars_axis_crosses_it_in_two_circles_and_every_fuse_answers() {
    let shared = 32.0 * PI / 3.0 - 4.0 * 3f64.sqrt() * PI;
    every_fuse_answers(
        a_ball_on_a_bars_axis,
        [32.0 * PI / 3.0, 6.0 * PI, shared],
        [&[5], &[3], &[2], &[3, 3]],
    );
}

/// A pointed cone of slope 1 up to `z = 4` and a bar of radius 1 down its axis
/// from `z = −2` to 6. The pipe crosses the cone in circles at `z = ±1`, and
/// the two come together only for a pipe of no width or a flat cone, neither of
/// which can be made (`a_pipe_of_no_width_and_a_flat_cone_cannot_be_made`).
///
/// The bar holds the cone's tip below `z = 1`, `π/3`, and three of the bar
/// above it. The bar less the cone is the bar below the apex and above the
/// base.
#[test]
fn a_cone_and_a_bar_on_one_axis_cross_in_two_circles_and_every_fuse_answers() {
    every_fuse_answers(
        a_cone_and_a_bar_on_one_axis,
        [64.0 * PI / 3.0, 8.0 * PI, 10.0 * PI / 3.0],
        [&[6], &[3], &[3], &[3, 3]],
    );
}

/// The pointed cone of slope 1 up to `z = 4` and the block `[1, 6] × [−6, 6] ×
/// [−2, 6]`. The block's face `x = 1` cuts the cone in a hyperbola, a branch
/// on each nappe. The two nappes share the apex and nothing else, so the
/// branches come together only for a face through the apex, which is §347.1's
/// node (`a_plane_cuts_a_cones_two_nappes_apart_unless_it_runs_through_the_apex`).
#[test]
fn a_block_beside_a_cones_apex_cuts_both_nappes_and_every_fuse_answers() {
    every_fuse_answers(
        a_block_beside_a_cones_apex,
        [64.0 * PI / 3.0, 480.0, cone_beyond(1.0, 4.0)],
        [&[8], &[3], &[3], &[8]],
    );
}

/// A bar of radius 2 up `z` from −2 to 2, and a bar of radius 1 along `x` from
/// −4 to 4 at `y = 1/2`, `z = 0`. The thin pipe crosses the thick one in two
/// loops, one either side of `x = 0`. They come together only where the thin
/// pipe touches the thick one from inside
/// (`a_thin_bar_resting_on_the_inside_of_a_thick_one_crosses_it_in_two_loops_that_meet_and_every_fuse_answers`).
///
/// The thin bar less the thick one is its two ends.
#[test]
fn a_thin_bar_through_a_thick_one_crosses_it_in_two_loops_and_every_fuse_answers() {
    every_fuse_answers(
        a_thin_bar_through_a_thick_one,
        [16.0 * PI, 8.0 * PI, bar_through_bar(2.0, 1.0, 0.5)],
        [&[7], &[3], &[4], &[3, 3]],
    );
}

/// The top half of a ring of 4 round a tube of 1 about `z`, and the `x ≥ 0`
/// half of a bar of radius 10 about `z` from −2 to 5, made by a first fuse.
/// The half bar's face `x = 0` runs through the ring's axis and cuts the tube
/// in two circles, at `y = ±4`, both drawn on that one face. They are `2(R −
/// r)` apart and would meet only for a tube as wide as the ring
/// (`a_ring_whose_tube_is_as_wide_as_the_ring_cannot_be_made`). The half bar's
/// pipe and caps miss the ring, since a plane parallel to its axis and not
/// through it is a quartic `intersect` refuses.
///
/// The half bar is `350π` and holds half the half ring, `2π²`. What they share
/// is the half ring's `x ≥ 0` half: the tube, the flat face and the two half
/// discs on `x = 0`, one round each circle. The half bar less the ring keeps
/// its four faces, `x = 0` with two holes, and the tube and flat face of the
/// hollow.
#[test]
fn a_half_bar_on_a_rings_axis_cuts_the_tube_in_two_circles_and_every_fuse_answers() {
    every_fuse_answers(
        a_half_ring_and_a_half_bar_on_its_axis,
        [4.0 * PI * PI, 350.0 * PI, 2.0 * PI * PI],
        [&[6], &[4], &[4], &[6]],
    );
}

/// A ball of radius 2 at the origin and a bar of radius 1 at `x = 1`: the bar
/// touches the ball from inside at `(2, 0, 0)`, and the two loops it crosses
/// the ball in meet there, Viviani's figure of eight. Each loop runs through
/// one of the ball's poles, `(0, 0, ±2)`.
///
/// Every fuse was refused by `intersect` until Q2649, which names the two
/// loops either side of the plane through the ball's centre square to the
/// bar, and puts a node where they meet and one half way round each. What they
/// share is Viviani's window, `16π/3 − 64/9`, and every answer has a corner at
/// `(2, 0, 0)`.
#[test]
fn a_bar_touching_a_ball_from_inside_crosses_it_in_two_loops_that_meet_and_every_fuse_answers() {
    every_fuse_answers_with_a_corner_at(
        a_bar_touching_a_ball_from_inside,
        [32.0 * PI / 3.0, 6.0 * PI, 16.0 * PI / 3.0 - 64.0 / 9.0],
        [&[5], &[3], &[2], &[3, 3]],
        [2.0, 0.0, 0.0],
    );
}

/// Two bars of radius 1, one up `z` and one along `x`, each from −2 to 2, whose
/// axes meet at the origin. Their pipes cross in two ellipses, on the planes
/// `x = z` and `x = −z`, which meet at `(0, ±1, 0)`: the Steinmetz cross.
/// `place_the_meets` puts a node at each of the two points, with the four
/// halves of the ellipses running into it, and one half way along each half,
/// at `(±1, 0, ±1)`.
///
/// Each bar is `4π`. What they share is the Steinmetz solid, `16r³/3`, which
/// stays within `|x|, |z| ≤ 1`, inside both bars. The union keeps each pipe
/// in two pieces, either side of the other bar, and the four caps, 8 faces;
/// what they share has 4; each bar less the other is two pieces of 3 faces,
/// which touch at the two points.
///
/// Every fuse was refused telling the cells of a pipe apart until Q1660. Which
/// cell a walk round a pipe is in is read off how often each walk crosses a
/// half-plane through the axis, and every half-plane tried ran along an axis
/// of the model: on each pipe the two meeting points and the four middles sat
/// on all of them. The half-plane is now chosen half way across the widest
/// gap between the nodes. Past it every fuse was refused
/// `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` in `assemble`, where the
/// two pipes' gradients are parallel at `(0, ±1, 0)` and the way along an edge
/// out of there was read as nought.
#[test]
fn two_bars_of_one_radius_whose_axes_meet_cross_in_four_half_ellipses_and_every_fuse_answers() {
    every_fuse_answers(
        two_bars_of_one_radius_whose_axes_meet,
        [4.0 * PI, 4.0 * PI, 16.0 / 3.0],
        [&[8], &[4], &[3, 3], &[3, 3]],
    );
}

/// A bar of radius 2 up `z`, and a bar of radius 1 along `x` at `y = 1`, which
/// touches it from inside at the one point `(0, 2, 0)`: the two loops of
/// `a_thin_bar_through_a_thick_one_crosses_it_in_two_loops_and_every_fuse_answers`
/// pushed together until they meet there, crossing like an X.
///
/// `intersect` names the two loops since Q2628. Until Q2649 the arrangement
/// refused them, having no node for where they meet: let through, the two
/// loops ran through `(0, 2, 0)` with no vertex anywhere near it, and the
/// answers validated all the same. So every answer is held to a corner there.
#[test]
fn a_thin_bar_resting_on_the_inside_of_a_thick_one_crosses_it_in_two_loops_that_meet_and_every_fuse_answers()
 {
    every_fuse_answers_with_a_corner_at(
        a_thin_bar_resting_on_the_inside_of_a_thick_one,
        [16.0 * PI, 8.0 * PI, bar_through_bar(2.0, 1.0, 1.0)],
        [&[7], &[3], &[4], &[3, 3]],
        [0.0, 2.0, 0.0],
    );
}

/// [`a_thin_bar_resting_on_the_inside_of_a_thick_one_crosses_it_in_two_loops_that_meet_and_every_fuse_answers`]
/// turned by quarter turns: the thin bar along `y` at `x = 1` and at `x = −1`,
/// and along `x` at `y = −1`, so its two loops meet at `(2, 0, 0)`,
/// `(−2, 0, 0)` and `(0, −2, 0)`. The thick bar's wall between its caps is a
/// ring with two windows in it that touch there, and the mesh cuts a ring
/// across at four quarter turns. Here one of the four runs exactly through
/// where the windows touch; with the thin bar at `y = 1` that cut misses the
/// point by an ulp.
///
/// Until Q2649 a cut that met two windows was refused, and so was every Union
/// and the Difference thick bar first, `CurvedFaceWithAHoleGoesTooFarRound`.
/// Then a cut through where they touch went round the first window it met, and
/// along `y` at `x = 1` that was the window no other cut met: no patch took
/// either side of it, and the Union thin bar first went on being refused.
#[test]
fn a_thin_bar_resting_inside_a_thick_one_where_a_cut_across_its_wall_runs_answers_every_fuse() {
    let placements: [(Pair, [f64; 3]); 3] = [
        (
            |model| {
                [
                    a_bar(model, 2.0, [0.0, 0.0, -2.0], 4.0),
                    a_bar_along_y(model, 1.0, [1.0, -4.0, 0.0], 8.0),
                ]
            },
            [2.0, 0.0, 0.0],
        ),
        (
            |model| {
                [
                    a_bar(model, 2.0, [0.0, 0.0, -2.0], 4.0),
                    a_bar_along_y(model, 1.0, [-1.0, -4.0, 0.0], 8.0),
                ]
            },
            [-2.0, 0.0, 0.0],
        ),
        (
            |model| {
                [
                    a_bar(model, 2.0, [0.0, 0.0, -2.0], 4.0),
                    a_bar_along_x(model, 1.0, [-4.0, -1.0, 0.0], 8.0),
                ]
            },
            [0.0, -2.0, 0.0],
        ),
    ];
    for (build, corner) in placements {
        every_fuse_answers_with_a_corner_at(
            build,
            [16.0 * PI, 8.0 * PI, bar_through_bar(2.0, 1.0, 1.0)],
            [&[7], &[3], &[4], &[3, 3]],
            corner,
        );
    }
}

/// The thin bar of
/// `a_thin_bar_resting_on_the_inside_of_a_thick_one_crosses_it_in_two_loops_that_meet_and_every_fuse_answers`
/// thinner, resting on the inside of the bar of radius 2 at `(0, 2, 0)`: of
/// `3/4` along `x` at `y = 5/4`, and of `1/2` along `x` at `y = 3/2`, each 8
/// long.
///
/// Every fuse was refused `TheseTwoSurfacesDoNotCrossInAConic` until Q2709.
/// Two points on one loop that came out of the arrangement as one were put in
/// order round it, and a thin pipe touching a fat one from inside was refused
/// there, though each of its two loops runs once round the thin pipe's axis as
/// the loops of a thin pipe the fat one swallows do. The bar of radius 1 asked
/// no such order.
#[test]
fn a_thinner_bar_resting_inside_a_thick_one_answers_every_fuse() {
    let placements: [(Pair, f64, f64); 2] = [
        (
            |model| {
                [
                    a_bar(model, 2.0, [0.0, 0.0, -2.0], 4.0),
                    a_bar_along_x(model, 0.75, [-4.0, 1.25, 0.0], 8.0),
                ]
            },
            0.75,
            1.25,
        ),
        (
            |model| {
                [
                    a_bar(model, 2.0, [0.0, 0.0, -2.0], 4.0),
                    a_bar_along_x(model, 0.5, [-4.0, 1.5, 0.0], 8.0),
                ]
            },
            0.5,
            1.5,
        ),
    ];
    for (build, radius, off) in placements {
        every_fuse_answers_with_a_corner_at(
            build,
            [
                16.0 * PI,
                8.0 * PI * radius * radius,
                bar_through_bar(2.0, radius, off),
            ],
            [&[7], &[3], &[4], &[3, 3]],
            [0.0, 2.0, 0.0],
        );
    }
}

/// [`a_thinner_bar_resting_inside_a_thick_one_answers_every_fuse`] turned by
/// quarter turns, so the loops meet where a cut across the thick bar's wall
/// runs: of `3/4` along `y` at `x = 5/4`, meeting at `(2, 0, 0)`, and of `1/2`
/// along `y` at `x = −3/2` and along `x` at `y = −3/2`, meeting at
/// `(−2, 0, 0)` and `(0, −2, 0)`.
///
/// Every fuse was refused `TheseTwoSurfacesDoNotCrossInAConic` until Q2709.
/// Then the union and the thick bar less the thin one were refused
/// `CurvedFaceWithAHoleGoesTooFarRound` meshing the thick bar's wall, whose
/// two windows are each narrower than a quarter turn here, so no cut but the
/// one through where they touch meets either: that cut went round one window
/// there and touched it at that point alone, and no patch took either side of
/// it (Q2711).
#[test]
fn a_thinner_bar_resting_inside_a_thick_one_where_a_cut_across_its_wall_runs_answers_every_fuse() {
    let placements: [(Pair, f64, f64, [f64; 3]); 3] = [
        (
            |model| {
                [
                    a_bar(model, 2.0, [0.0, 0.0, -2.0], 4.0),
                    a_bar_along_y(model, 0.75, [1.25, -4.0, 0.0], 8.0),
                ]
            },
            0.75,
            1.25,
            [2.0, 0.0, 0.0],
        ),
        (
            |model| {
                [
                    a_bar(model, 2.0, [0.0, 0.0, -2.0], 4.0),
                    a_bar_along_y(model, 0.5, [-1.5, -4.0, 0.0], 8.0),
                ]
            },
            0.5,
            1.5,
            [-2.0, 0.0, 0.0],
        ),
        (
            |model| {
                [
                    a_bar(model, 2.0, [0.0, 0.0, -2.0], 4.0),
                    a_bar_along_x(model, 0.5, [-4.0, -1.5, 0.0], 8.0),
                ]
            },
            0.5,
            1.5,
            [0.0, -2.0, 0.0],
        ),
    ];
    for (build, radius, off, corner) in placements {
        every_fuse_answers_with_a_corner_at(
            build,
            [
                16.0 * PI,
                8.0 * PI * radius * radius,
                bar_through_bar(2.0, radius, off),
            ],
            [&[7], &[3], &[4], &[3, 3]],
            corner,
        );
    }
}

/// The bar of radius 1 from `(1, 0, −3)` up `z`, 6 long, through the bar of
/// radius 2 up `z` from `−2` to 2. The thin pipe rests on the inside of the
/// thick one along the line `x = 2`, `y = 0`, where both seams run, and
/// `intersect` names that one line. Turned the least bit about `x`, it crosses
/// in two curves that meet
/// ([`a_thin_bar_turned_a_hair_inside_a_thick_one_is_refused_at_the_first_face_a_fuse_cuts_that_refuses`]).
///
/// What they share is the thin bar between the thick bar's caps, `4π`.
#[test]
fn a_thin_bar_up_the_inside_of_a_thick_one_rests_on_it_along_its_seam_and_every_fuse_answers() {
    let mut model = a_model();
    let [thick, thin] =
        a_thin_bar_up_the_inside_of_a_thick_one(&mut model).map(|solid| the_wall(&model, solid));
    assert_eq!(curves(&thick, &thin), Ok(1));
    assert_eq!(curves(&thin, &thick), Ok(1));
    every_fuse_answers(
        a_thin_bar_up_the_inside_of_a_thick_one,
        [16.0 * PI, 6.0 * PI, 4.0 * PI],
        [&[7], &[3], &[4], &[3, 3]],
    );
}

/// [`a_thin_bar_up_the_inside_of_a_thick_one_rests_on_it_along_its_seam_and_every_fuse_answers`]
/// with the thin bar turned `lean` about `x`: `4e−16`, `1e−15`, `1e−14` and
/// `1e−8`. Its axis then runs `(x, −lean·z)`, so the thin pipe touches the
/// thick one at `(2, 0, 0)` alone and pokes out of it a little above and
/// below. `intersect` names two curves both ways round, and they cross at
/// that point, at angles `−lean·z (1 ± 1/√2)` round the thick axis. Two
/// placements: the thin bar at `x = 1`, whose seam runs along the thick wall,
/// and at `x = −1`, where it touches the thick wall opposite its seam. Every
/// fuse is refused, at every lean, and which refusal depends on which bar
/// comes first. Both refusals are a limit up to `1e−14` and a gap at `1e−8`
/// (§16.10). The places below are worked out to first order in the lean.
///
/// **The order.** A fuse cuts every face of its first solid before any face of
/// its second, and stops at the first face that refuses
/// (`Scratch::split_both_groups` in `fuse/mod.rs`). With the thick bar first
/// the first face cut is its floor, and that is refused
/// `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`. With the thin bar first at
/// `x = 1` it is the thin floor, refused `side of cylinder` at its seam corner
/// `(2, 3·lean, −3)`. At `x = −1` the thin floor has no corner on the thick
/// wall, the thin floor and roof are cut, and the thin wall is refused
/// `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`. Before any face is cut,
/// `tell_each_edge_what_its_neighbour_leaves_out` in `fuse/cuts.rs` asks the
/// same side of both thin caps and drops the refusal, as its doc says it does.
///
/// **`WhereThreeSurfacesMeetNeedsTwoOfThemFlat`** is asked where a thick cap
/// meets both walls. It meets them in two points `4√2·lean` apart, beside
/// coordinates of 3: a spread past
/// [`DynamicRange::LARGEST_SUPPORTED_RATIO`] up to `1e−14`, a limit, and of
/// `5.3e7` at `1e−8`, where the two points are there to find, a gap. That gap
/// is Q2202's family, a plane and two round surfaces that Q2196's mirror road
/// does not take.
///
/// **`side of cylinder`** is asked of the thin floor's seam corner, which is a
/// sample of its rim (§17.1). The rim is `(1 + cos t, 3·lean + sin t, −3)`,
/// which lies outside the thick wall where `−t² + 6·lean·t + 9·lean² > 0`,
/// between the two crossings `t = 3·lean (1 ± √2)`, at `y = 3·lean (2 ± √2)`.
/// The corner is `t = 0`, outside by `(3·lean)²/4`, too little for its box to
/// show. Up to `1e−14` the corner's box holds both crossings, so rim points in
/// it lie on both sides, and the crossings are too close for the supported
/// spread: a limit. At `1e−8` the box lies between the crossings, so every rim
/// point in it is outside and the answer is there: a gap.
/// `a_corner_of_two_surfaces_is_where_the_partner_crosses_their_curve` in
/// `fuse/arrange.rs` would read the corner on its rim, and does not, since
/// the meet of the floor and both walls is refused
/// `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` too.
#[test]
fn a_thin_bar_turned_a_hair_inside_a_thick_one_is_refused_at_the_first_face_a_fuse_cuts_that_refuses()
 {
    for (x, seam_on_the_thick_wall) in [(1.0, true), (-1.0, false)] {
        for (lean, answer_is_there) in
            [(4e-16, false), (1e-15, false), (1e-14, false), (1e-8, true)]
        {
            let build =
                move |model: &mut Model| a_thin_bar_turned_inside_a_thick_one(model, x, lean);
            let mut model = a_model();
            let [thick, thin] = build(&mut model).map(|solid| the_wall(&model, solid));
            assert_eq!(curves(&thick, &thin), Ok(2), "x {x}, lean {lean}");
            assert_eq!(curves(&thin, &thick), Ok(2), "x {x}, lean {lean}");
            let caps_meet_the_walls =
                DynamicRange::measure(3.0, 4.0 * SQRT_2 * lean).expect("two points apart");
            assert_eq!(
                caps_meet_the_walls.is_within_supported_range(),
                answer_is_there,
                "x {x}, lean {lean}: a cap meets both walls at a spread of {}",
                caps_meet_the_walls.ratio()
            );
            let [low, high] = [2.0 - SQRT_2, 2.0 + SQRT_2].map(|k| 3.0 * lean * k);
            let crossings = DynamicRange::measure(3.0, high - low).expect("two points apart");
            assert_eq!(
                crossings.is_within_supported_range(),
                answer_is_there,
                "x {x}, lean {lean}: the rim crosses the thick wall at a spread of {}",
                crossings.ratio()
            );
            for (rule, thick_first) in six_fuses() {
                let what = format!("x {x}, lean {lean}: {rule:?} (thick first: {thick_first})");
                let said = fuse(&mut a_model(), build, rule, thick_first);
                if seam_on_the_thick_wall && !thick_first {
                    let refused = refused_on_one_question_at(
                        &said,
                        "side of cylinder",
                        [2.0, 3.0 * lean, -3.0],
                        &what,
                    );
                    let [corner] = refused.inputs() else {
                        panic!("{what}: one corner is asked about: {refused:#?}");
                    };
                    let y = corner.enclosure()[1];
                    assert_eq!(
                        y.lo() < low && high < y.hi(),
                        !answer_is_there,
                        "{what}: the box {y:?} holds both crossings {low} and {high}"
                    );
                    assert_eq!(
                        low < y.lo() && y.hi() < high,
                        answer_is_there,
                        "{what}: the box {y:?} lies between the crossings {low} and {high}"
                    );
                } else {
                    assert!(
                        matches!(
                            said,
                            Err(rcad_topo::Error::Geometry(
                                rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat
                            ))
                        ),
                        "{what}: {said:?}"
                    );
                }
            }
        }
    }
}

/// A ball of radius 2 with a bar from `z = −3` to 3 through it off its axis,
/// so that the pipe crosses the ball in two loops apart, one above `z = 0` and
/// one below. They come together only where the bar touches the ball from
/// inside
/// (`a_bar_touching_a_ball_from_inside_crosses_it_in_two_loops_that_meet_and_every_fuse_answers`).
/// Four placements: the bar of radius 1 at `x = 1/2` and at `y = 1/2`, the bar
/// of radius 1/2 at `y = 1` clear of the ball's poles, and the ball turned a
/// quarter round `z` with the bar at `x = 1/2`. Every fuse was refused ordering
/// points along a loop until Q1554 ordered them round the bar's axis.
///
/// What they share is the ball inside the bar, [`ball_in_bar`]. With the bar
/// first every fuse answers too
/// (`a_ball_with_a_bar_through_it_off_its_axis_crosses_it_in_two_loops_and_every_fuse_with_the_bar_first_answers`).
#[test]
fn a_ball_with_a_bar_through_it_off_its_axis_crosses_it_in_two_loops_and_every_fuse_with_the_ball_first_answers()
 {
    for (build, radius, off) in balls_with_a_bar_through_them_off_their_axis() {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            the_fuse_answers(
                build,
                (rule, true),
                [
                    32.0 * PI / 3.0,
                    6.0 * PI * radius * radius,
                    ball_in_bar(radius, off),
                ],
                [&[5], &[3], &[2], &[3, 3]],
            );
        }
    }
}

/// The shapes of
/// `a_ball_with_a_bar_through_it_off_its_axis_crosses_it_in_two_loops_and_every_fuse_with_the_ball_first_answers`
/// taken the other way round, held to the same volumes. Each loop is an edge
/// with one corner, at its lowest or highest point, and with the bar first the
/// mesh walks the loop on the bar's wall in a frame that puts the corner at
/// `(−1/2, 0, 0)`. Every fuse was refused `CannotMeshAFace { why:
/// CornersCoincide }` there until Q1598: the marcher's first trial settled back
/// one rounding off the corner and was kept as a step of `5.9e-17`, and the
/// steps after it doubled from there, so two corners of the wall's chart
/// coincided. A trial that comes back short to within its two ends' own bounds
/// is now shortened and tried again rather than kept.
#[test]
fn a_ball_with_a_bar_through_it_off_its_axis_crosses_it_in_two_loops_and_every_fuse_with_the_bar_first_answers()
 {
    for (build, radius, off) in balls_with_a_bar_through_them_off_their_axis() {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            the_fuse_answers(
                build,
                (rule, false),
                [
                    32.0 * PI / 3.0,
                    6.0 * PI * radius * radius,
                    ball_in_bar(radius, off),
                ],
                [&[5], &[3], &[2], &[3, 3]],
            );
        }
    }
}

/// A bar of radius 2 up `z` from −1 to 11, turned a quarter round `z` so that
/// its seam runs up `(0, 2, z)`, and a ball of radius 3 at `(0, 3, 5)`. The
/// ball reaches into the bar's side and crosses its pipe in one loop, the whole
/// of their crossing, and the seam runs through the loop's top and bottom,
/// `(0, 2, 5 ± 2√2)`. One loop has no second curve of the pair to meet.
///
/// Every fuse was refused ordering points along the loop until Q1661: a loop
/// that runs back and forth round the pipe's axis is ordered by no
/// determinant, and `fuse/arrange.rs` let only the two loops of a ball that
/// swallows the pipe through. The walk starts on the loop's top, where the
/// seam crosses it, so the only other place on the loop that anything is cut
/// is the bottom, and the start comes first by its name. Where neither place
/// is the start, `rcad-geom` is asked which way the loop runs. Since Q1767 it
/// answers, by how far round the line through the ball's centre square to the
/// pipe's axis each place is (`a_ball_driven_into_the_bar_fuses_into_one_block`
/// in `a_ball_and_a_bar_in_one_block.rs`).
///
/// What they share is the ball inside the bar, [`ball_in_the_side_of_a_bar`].
/// The bar less the ball keeps its pipe, its two caps and the dent; the ball
/// less the bar is its skin and the pipe's patch.
#[test]
fn a_ball_in_a_bars_side_across_its_seam_crosses_it_in_one_loop_and_every_fuse_answers() {
    every_fuse_answers(
        a_bar_seamed_up_y_and_a_ball_in_its_side,
        [48.0 * PI, 36.0 * PI, ball_in_the_side_of_a_bar(1000)],
        [&[4], &[2], &[4], &[2]],
    );
}

/// [`ball_in_the_side_of_a_bar`] at the 1000 steps the fuse is held to is
/// within `1e-6` of the rule at 100 000, which is `6e-7` above it. The mesh
/// of `0.01` may miss the volumes it is held against by `0.4` or more.
#[test]
fn the_ball_in_the_side_of_a_bar_is_within_a_millionth_of_a_rule_a_hundred_times_finer() {
    let (coarse, fine) = (
        ball_in_the_side_of_a_bar(1000),
        ball_in_the_side_of_a_bar(100_000),
    );
    assert!(
        (coarse - fine).abs() < 1e-6,
        "{coarse} at 1000 steps, {fine} at 100 000"
    );
}

/// [`ball_in_bar`] on the ball's axis is the closed form
/// `(4π/3)(2³ − (2² − 1)^(3/2))` of
/// `a_ball_on_a_bars_axis_crosses_it_in_two_circles_and_every_fuse_answers`,
/// to the `1e-6` its doc comment gives it.
#[test]
fn the_ball_in_a_bar_on_its_axis_is_the_closed_form() {
    let closed = 4.0 * PI / 3.0 * (8.0 - 3f64.powf(1.5));
    let ruled = ball_in_bar(1.0, 0.0);
    assert!(
        (ruled - closed).abs() < 1e-6,
        "{ruled} by the rule, {closed} closed"
    );
}

/// The top half of a ring of 4 round a tube of 1 about `z`, and a bar of
/// radius 6 about `z` from −2 to 1/2. The bar's top cuts the ring in two
/// circles round the axis, of radius `4 ± √3/2`; they come together only for a
/// plane on top of the tube
/// (`a_plane_resting_on_top_of_a_ring_touches_it_round_one_circle`). Every
/// fuse was refused telling the cells of the ring's face apart until Q1514
/// put the bands in order round the tube.
///
/// What they share is the tube's cross-section below `z = 1/2`, `√3/4 + π/6`,
/// swept round the ring's circle of length `8π`: `4π²/3 + 2π√3`.
#[test]
fn a_half_ring_cut_by_a_round_plate_crosses_it_in_two_circles_and_every_fuse_answers() {
    let shared = 4.0 * PI * PI / 3.0 + 2.0 * PI * 3f64.sqrt();
    every_fuse_answers(
        a_half_ring_cut_by_a_round_plate,
        [4.0 * PI * PI, 90.0 * PI, shared],
        [&[5], &[4], &[2], &[7]],
    );
}

/// The half bar of
/// `a_half_bar_on_a_rings_axis_cuts_the_tube_in_two_circles_and_every_fuse_answers`
/// with its floor raised to `z = −1`. The floor is still clear of the half
/// ring, which lies above `z = 0`, but it is the plane the whole ring would
/// rest on, and it touches the ring's torus round the circle of radius 4 at
/// `z = −1`. That circle lies on no face of the half ring. Every fuse was
/// refused ordering points along it, where the plane and the torus are
/// tangent and no way along follows from them, until Q2648 left it off.
///
/// The half bar is `300π`; the rest is as with the floor at `z = −2`.
#[test]
fn a_half_bar_whose_floor_the_whole_ring_would_rest_on_touches_it_round_a_circle_no_face_reaches_and_every_fuse_answers()
 {
    every_fuse_answers(
        |model| a_half_ring_and_a_half_bar_with_its_floor_at(model, -1.0),
        [4.0 * PI * PI, 300.0 * PI, 2.0 * PI * PI],
        [&[6], &[4], &[4], &[6]],
    );
}

/// The top half of a ring of 4 round a tube of 1 about `z`, as a bead on a
/// round plate of radius 10 from `z = −1` to 0: the ring's flat face lies on
/// the plate's top, and the plate's bottom is the plane the whole ring would
/// rest on, touching its torus round the circle of radius 4 at `z = −1`, on
/// no face of the half ring. Every fuse was refused `Undecidable` on a point
/// inside a piece of the plate's bottom at `(4, 0, −1)` until Q2648 left that
/// circle off.
///
/// They share the flat face and no volume.
#[test]
fn a_half_ring_bead_on_a_plate_as_thick_as_its_tube_shares_its_flat_face_and_every_fuse_answers() {
    every_fuse_answers(
        |model| a_half_ring_bead_on_a_plate_up_to(model, 0.0),
        [4.0 * PI * PI, 100.0 * PI, 0.0],
        [&[5], &[], &[2], &[3]],
    );
}

/// The bead of
/// `a_half_ring_bead_on_a_plate_as_thick_as_its_tube_shares_its_flat_face_and_every_fuse_answers`
/// sunk in a plate whose top is at `z = 1/2`. The top cuts the ring in two
/// circles, as in
/// `a_half_ring_cut_by_a_round_plate_crosses_it_in_two_circles_and_every_fuse_answers`,
/// and the bottom touches the torus round the circle at `z = −1`. Every fuse
/// was refused telling the cells of the bottom apart until Q2648 left that
/// circle off.
///
/// What they share is the tube's cross-section below `z = 1/2` swept round the
/// ring, `4π²/3 + 2π√3`.
#[test]
fn a_half_ring_bead_sunk_in_a_plate_whose_bottom_the_ring_would_rest_on_crosses_its_top_in_two_circles_and_every_fuse_answers()
 {
    let shared = 4.0 * PI * PI / 3.0 + 2.0 * PI * 3f64.sqrt();
    every_fuse_answers(
        |model| a_half_ring_bead_on_a_plate_up_to(model, 0.5),
        [4.0 * PI * PI, 150.0 * PI, shared],
        [&[5], &[4], &[2], &[7]],
    );
}

/// A plane through a ring's axis cuts the tube in two circles, `2(R − r)`
/// apart. They would meet only for a tube as wide as the ring, and
/// `Torus::new` refuses that; a ring read from a file is checked the same way
/// (`#[serde(try_from)]`).
#[test]
fn a_ring_whose_tube_is_as_wide_as_the_ring_cannot_be_made() {
    assert!(matches!(
        Torus::new(Axis::z(), 1.0, 1.0),
        Err(rcad_geom::Error::ATorusThatRunsThroughItself { .. })
    ));
    let narrowest = Torus::new(Axis::z(), 1.0, 1.0f64.next_down()).expect("a tube narrower");
    let through_the_axis = a_plane([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    assert_eq!(curves(&through_the_axis, &Surface::Torus(narrowest)), Ok(2));
}

/// A plane along a pipe's axis cuts it in two lines `2√(r² − d²)` apart, `d`
/// from the axis. At `d = r` `intersect` names one line, where the plane
/// rests on the pipe.
#[test]
fn a_plane_resting_on_a_pipe_touches_it_along_one_line() {
    let pipe = a_pipe([0.0, 0.0], 2.0);
    assert_eq!(
        curves(&a_plane([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]), &pipe),
        Ok(2)
    );
    assert_eq!(
        curves(&a_plane([2.0, 0.0, 0.0], [1.0, 0.0, 0.0]), &pipe),
        Ok(1)
    );
}

/// Two pipes along `z`, of radius 2 and 1, cross in two lines unless their
/// axes are `3` or `1` apart, where the thin one rests on the fat one outside
/// or inside it, and `intersect` names one line.
#[test]
fn two_pipes_resting_side_by_side_touch_along_one_line() {
    let fat = a_pipe([0.0, 0.0], 2.0);
    assert_eq!(curves(&fat, &a_pipe([2.0, 0.0], 1.0)), Ok(2));
    assert_eq!(curves(&fat, &a_pipe([3.0, 0.0], 1.0)), Ok(1));
    assert_eq!(curves(&fat, &a_pipe([1.0, 0.0], 1.0)), Ok(1));
}

/// A ball on a pipe's axis crosses it in two circles `√(R² − r²)` either side
/// of its centre, and a ball as wide as the pipe touches it round one.
#[test]
fn a_ball_on_a_pipes_axis_as_wide_as_the_pipe_touches_it_round_one_circle() {
    let pipe = a_pipe([0.0, 0.0], 1.0);
    assert_eq!(curves(&a_ball([0.0, 0.0, 5.0], 2.0), &pipe), Ok(2));
    assert_eq!(curves(&a_ball([0.0, 0.0, 5.0], 1.0), &pipe), Ok(1));
}

/// A plane square to a ring's axis at height `h` cuts it in two circles of
/// radius `R ± √(r² − h²)`, and the plane on top of the tube, `h = r`, touches
/// it round one.
#[test]
fn a_plane_resting_on_top_of_a_ring_touches_it_round_one_circle() {
    let ring = Surface::Torus(Torus::new(Axis::z(), 4.0, 1.0).expect("a ring"));
    assert_eq!(
        curves(&a_plane([0.0, 0.0, 0.5], [0.0, 0.0, 1.0]), &ring),
        Ok(2)
    );
    assert_eq!(
        curves(&a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]), &ring),
        Ok(1)
    );
}

/// A plane along a cone's axis `d` from it cuts the two nappes in the two
/// branches of a hyperbola. The nappes share only the apex, so the branches
/// meet only where the plane runs through it, and there they are the two
/// rulings §347.1 puts a node between.
#[test]
fn a_plane_cuts_a_cones_two_nappes_apart_unless_it_runs_through_the_apex() {
    let cone = Surface::Cone(Cone::new(Axis::z(), 1.0).expect("a cone"));
    assert_eq!(
        curves(&a_plane([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]), &cone),
        Ok(2)
    );
    assert_eq!(
        curves(&a_plane([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]), &cone),
        Ok(2)
    );
}

/// A pipe on a cone's axis crosses it in two circles at `±r/s` from the apex,
/// which would meet only for `r = 0` or for `s` without end, a cone flattened
/// into a plane. `Cylinder::new` refuses the first and `Cone::new` the second.
#[test]
fn a_pipe_of_no_width_and_a_flat_cone_cannot_be_made() {
    assert!(matches!(
        Cylinder::new(Axis::z(), 0.0),
        Err(rcad_geom::Error::RadiusNotPositive { .. })
    ));
    assert!(matches!(
        Cone::new(Axis::z(), f64::INFINITY),
        Err(rcad_geom::Error::SlopeNotPositive { .. })
    ));
    let cone = Surface::Cone(Cone::new(Axis::z(), 1.0).expect("a cone"));
    assert_eq!(curves(&cone, &a_pipe([0.0, 0.0], f64::MIN_POSITIVE)), Ok(2));
}

/// The coaxial pairs §347.5's table leaves out, a ring with a pipe, a ball or a
/// cone on its axis, a cone and a ball, and two cones, give circles round one
/// axis at heights apart. The heights come together only at a tangency, where
/// `intersect` names one circle, or at a cone's apex: two cones of slopes `s₁ ≠
/// s₂` with their apexes `d` apart cross at `s₂·d/(s₂ ± s₁)` along the axis
/// from the first apex, one height only for `d = 0`, where they touch at the
/// apex and nowhere else.
#[test]
fn coaxial_circles_come_together_only_where_their_surfaces_touch() {
    let ring = Surface::Torus(Torus::new(Axis::z(), 4.0, 1.0).expect("a ring"));
    assert_eq!(
        curves(&ring, &a_pipe([0.0, 0.0], 4.5)),
        Ok(2),
        "a pipe through the tube"
    );
    assert_eq!(
        curves(&ring, &a_pipe([0.0, 0.0], 5.0)),
        Ok(1),
        "a pipe round the tube"
    );
    assert_eq!(
        curves(&ring, &a_pipe([0.0, 0.0], 3.0)),
        Ok(1),
        "a pipe in the hole"
    );
    assert_eq!(
        curves(&ring, &a_ball([0.0, 0.0, 0.0], 4.5)),
        Ok(2),
        "a ball through the tube"
    );
    assert_eq!(
        curves(&ring, &a_ball([0.0, 0.0, 0.0], 5.0)),
        Ok(1),
        "a ball round the tube"
    );
    assert_eq!(
        curves(&ring, &a_ball([0.0, 0.0, 0.0], 3.0)),
        Ok(1),
        "a ball in the hole"
    );
    assert_eq!(
        curves(&ring, &a_cone(-6.0, 0.75)),
        Ok(2),
        "a cone through the tube"
    );
    assert_eq!(
        curves(&ring, &a_cone(-7.0, 0.75)),
        Ok(1),
        "a cone resting on the tube"
    );
    let cone = Surface::Cone(Cone::new(Axis::z(), 0.75).expect("a cone"));
    assert_eq!(
        curves(&cone, &a_ball([0.0, 0.0, 5.0], 4.0)),
        Ok(2),
        "a ball through the cone"
    );
    assert_eq!(
        curves(&cone, &a_ball([0.0, 0.0, 5.0], 3.0)),
        Ok(1),
        "a ball resting in the cone"
    );
    assert_eq!(
        curves(&cone, &a_cone(1.0, 2.0)),
        Ok(2),
        "two cones with their apexes apart"
    );
    assert_eq!(
        curves(&cone, &a_cone(0.0, 2.0)),
        Ok(0),
        "two cones with one apex"
    );
}

/// Two pointed cones of slope ½ up to 8 on one apex at the origin, the second
/// turned 0.3 about `x`. Their sides cross in the two rulings of the plane
/// half way between the axes, and these meet at the apex: §347.1's node,
/// between two cones rather than a plane and a cone. Each ruling runs up to
/// `z = 8` and so reaches both rims, where both caps meet the two sides, at
/// once or very nearly.
///
/// **Very nearly, and so under the certified policy every fuse is refused, and
/// that is a limit.** The turned cap's normal and the turned cone's axis are
/// one stored vector `w`, a rounding short of length one: `|w|² − 1` is
/// `−9.1e−17`, decided exactly. A plane holds the points with `w · p = 8`
/// whatever `|w|`, and a cone is exact about the line its axis names, so the
/// turned rim lies on the sphere about the apex of radius `√80 / |w|`, and the
/// upright rim on the one of radius `√80`. The two rims never meet. The
/// upright rim runs into the turned cone through its cap, where `w · p = 8`,
/// and out through its side, where `w · p = 8|w|`, `1.3e−15` further round.
/// Every fuse keeps both points as corners: the union and the upright cone
/// less the turned one keep the rim outside the turned cone, and the
/// intersection and the turned cone less the upright one keep the stretch
/// inside. Two corners that close, beside a corner 8 from the origin, span
/// more than [`DynamicRange::LARGEST_SUPPORTED_RATIO`], so there is no
/// certified answer to find. What refuses is the order along a rim, which the
/// boxes round the two corners cannot tell: where a cap and both cones meet,
/// the box is `1.3e−14` across, ten times the gap between the two.
///
/// Turned 0.25 or 0.4 rather than 0.3, the two are refused the same way where
/// their rulings reach the rims. The tolerant policy is
/// [`two_cones_on_one_apex_leaning_apart_answer_every_fuse_under_a_tolerant_policy`]'s.
#[test]
fn two_cones_on_one_apex_leaning_apart_meet_both_rims_in_corners_too_close_for_a_certified_answer()
{
    let mut model = a_model();
    let [upright, turned] = two_cones_on_one_apex_leaning_apart(&mut model);
    let [(upright_cap, upright_side), (turned_cap, turned_side)] =
        [upright, turned].map(|solid| the_cap_and_the_side(&model, solid));
    for (cap, side) in [(upright_cap, upright_side), (turned_cap, turned_side)] {
        assert_eq!(cap.offset(), 8.0, "{cap:?}");
        assert_eq!(cap.normal(), side.axis().direction(), "{cap:?} {side:?}");
        assert_eq!(side.axis().origin(), Point::origin(), "{side:?}");
        assert_eq!(side.slope(), 0.5, "{side:?}");
    }
    assert_eq!(upright_cap.normal(), Vector::new(0.0, 0.0, 1.0));
    let w = turned_cap.normal();
    assert_eq!(w.x(), 0.0, "{w:?}");
    let (sign, short) = squared_length_less_one(w);
    assert_eq!(sign, Decision::Neg, "|w|² − 1 is {short}");
    let sin_where_the_rim_meets_the_turned_cap = 2.0 * (1.0 - w.z()) / w.y();
    let apart = 4.0 * short.abs()
        / (w.y().abs() * (1.0 - sin_where_the_rim_meets_the_turned_cap.powi(2)).sqrt());
    let range = DynamicRange::measure(8.0, apart).expect("two corners apart");
    assert!(
        !range.is_within_supported_range(),
        "corners {apart} apart beside one 8 from the origin span {}",
        range.ratio()
    );
    for (rule, first_first) in six_fuses() {
        let mut model = a_model();
        let said = fuse(
            &mut model,
            two_cones_on_one_apex_leaning_apart,
            rule,
            first_first,
        );
        refused_on_one_question_at(
            &said,
            "order along a crossing",
            [3.812888843803106, -1.2090817444663618, 8.0],
            &format!("{rule:?} (first first: {first_first})"),
        );
    }
}

/// The cones of the test above with the turned one cut short at 6. Its
/// rulings now end on the short cone's rim alone, so no point there lies on
/// all four faces.
///
/// The tall cone's side comes back in three faces, not one. The arrangement
/// cuts it along the turned cone's two rulings, run on past the short rim at
/// `z = 6` up to the tall rim at `z = 8`, and along the whole ellipse where the
/// short cap's plane crosses it. The three pieces are not joined up again, so
/// the union and the tall cone less the short one have six faces each.
#[test]
fn a_short_cone_on_the_apex_of_a_tall_one_leaning_apart_crosses_it_in_two_rulings_and_every_fuse_answers()
 {
    every_fuse_adds_up(
        a_short_cone_on_the_apex_of_a_tall_one_leaning_apart,
        [128.0 * PI / 3.0, 54.0 * PI / 3.0],
        [&[6], &[3], &[6], &[3]],
    );
}

/// The union of the test above, fused again with the block `[−10, 10]² ×
/// [7, 10]` over its top, answers every rule both ways round. The short cone
/// stays below `z = 7`, so what the two share is the tall cone's top from 7 to
/// 8, a frustum of `169π/12`, and the block less the union is the rest of the
/// block. The tall side's pieces are still not joined up, so the union has 12
/// faces and the union less the block 7.
///
/// Every fuse was refused on the order along a crossing at the corner the
/// union keeps on the tall rim, `(3.8129, −1.2091, 8)`, where the turned
/// cone's ruling is cut on up to `z = 8`, until Q2741 narrowed the box round
/// the meet of the tall cap and both cones from `1.9e−11` across to a few
/// roundings.
#[test]
fn the_union_of_a_short_cone_on_the_apex_of_a_tall_one_answers_every_fuse_when_a_block_cuts_it_again()
 {
    let frustum = 169.0 * PI / 12.0;
    let block = 1200.0;
    let mut model = a_model();
    let [union, _] = a_block_over_the_union_of_a_short_cone_on_the_apex_of_a_tall_one(&mut model);
    let (_, cones, cones_may_miss) =
        measures(&mut model, Ok(vec![union]), &[6], "the union of the cones");
    let faces: [&[usize]; 4] = [&[12], &[5], &[7], &[10]];
    for (rule, union_first) in six_fuses() {
        let which = which_answer(rule, union_first);
        let (volume, volume_may_miss) = [
            (cones + block - frustum, cones_may_miss),
            (frustum, 0.0),
            (cones - frustum, cones_may_miss),
            (block - frustum, 0.0),
        ][which];
        let mut model = a_model();
        let said = fuse(
            &mut model,
            a_block_over_the_union_of_a_short_cone_on_the_apex_of_a_tall_one,
            rule,
            union_first,
        );
        let what = format!(
            "{rule:?} (the union first: {union_first}), solids of {:?} faces and volume {volume}",
            faces[which]
        );
        let (_, measured, may_miss) = measures(&mut model, said, faces[which], &what);
        assert!(
            (measured - volume).abs() <= may_miss + volume_may_miss,
            "{what}: measured {measured} and the meshes may miss by {}",
            may_miss + volume_may_miss
        );
    }
}

/// The two cones of
/// [`two_cones_on_one_apex_leaning_apart_meet_both_rims_in_corners_too_close_for_a_certified_answer`],
/// under a tolerant policy for the model's own working size of 100, and for
/// one ten times coarser: every fuse answers, and the answers add up. Such a
/// policy may read the four meets of a cap with the other three surfaces as
/// one corner once it knows them to lie within 64 roundings of the working
/// size of one another, `1.4e−12` at 100.
///
/// At 100 every fuse was refused on the order along a crossing at
/// `(3.8129, −1.2091, 8)` until Q2741: the road for bands that cross put a
/// box `1.9e−11` across round the meet of the upright cap and both cones,
/// since it stopped after eight rounds while its boxes still shrank.
#[test]
fn two_cones_on_one_apex_leaning_apart_answer_every_fuse_under_a_tolerant_policy() {
    for working_size in [100.0, 1000.0] {
        every_fuse_adds_up_under(
            two_cones_on_one_apex_leaning_apart,
            [128.0 * PI / 3.0, 128.0 * PI / 3.0],
            [&[8], &[4], &[3, 3], &[3, 3]],
            &Policy::tolerant(working_size),
        );
    }
}

fn every_fuse_answers(build: Pair, volumes: [f64; 3], faces: [&[usize]; 4]) {
    for one in six_fuses() {
        the_fuse_answers(build, one, volumes, faces);
    }
}

/// Every fuse of `build`'s two solids, of volumes `first` and `second`,
/// answers with the faces `faces` holds for it in `every_fuse_answers`'s
/// order, and the answers add up: the union and the intersection to `first +
/// second`, the first less the second and the intersection to `first`, and the
/// second less the first and the intersection to `second`, each within what
/// the two meshes may miss by together.
fn every_fuse_adds_up(build: Pair, volumes: [f64; 2], faces: [&[usize]; 4]) {
    every_fuse_adds_up_under(build, volumes, faces, &Policy::certified());
}

/// [`every_fuse_adds_up`], with every fuse, check and measure under `policy`.
fn every_fuse_adds_up_under(
    build: Pair,
    [first, second]: [f64; 2],
    faces: [&[usize]; 4],
    policy: &Policy,
) {
    let mut measured: [Vec<(String, f64, f64)>; 4] = Default::default();
    for (rule, first_first) in six_fuses() {
        let which = which_answer(rule, first_first);
        let mut model = a_model();
        let said = fuse_under(&mut model, build, rule, first_first, policy);
        let what = format!(
            "{rule:?} (first first: {first_first}), solids of {:?} faces",
            faces[which]
        );
        let (_, volume, error) = measures_under(&mut model, said, faces[which], &what, policy);
        measured[which].push((what, volume, error));
    }
    let [unions, shared, first_less, second_less] = &measured;
    for (with, whole) in [
        (unions, first + second),
        (first_less, first),
        (second_less, second),
    ] {
        for (what, volume, error) in with {
            for (what_shared, shared, shared_error) in shared {
                assert!(
                    (volume + shared - whole).abs() <= error + shared_error,
                    "{what} measured {volume} and {what_shared} measured {shared}, \
                     which should add up to {whole} within {}",
                    error + shared_error
                );
            }
        }
    }
}

/// The fuse `rule` of `build`'s two solids, the first first or not, answers
/// with the volume and faces `every_fuse_answers` reads for it. The model and
/// the answer's solids are handed back.
fn the_fuse_answers(
    build: Pair,
    (rule, first_first): (Rule, bool),
    [first, second, shared]: [f64; 3],
    faces: [&[usize]; 4],
) -> (Model, Vec<SolidKey>) {
    let which = which_answer(rule, first_first);
    let volume = [
        first + second - shared,
        shared,
        first - shared,
        second - shared,
    ][which];
    let faces = faces[which];
    let mut model = a_model();
    let said = fuse(&mut model, build, rule, first_first);
    let what = format!(
        "{rule:?} (first first: {first_first}), solids of {faces:?} faces and volume {volume}"
    );
    let solids = answers(&mut model, said, volume, faces, &what);
    (model, solids)
}

/// Where the answer of the fuse `rule`, the first first or not, stands in
/// `every_fuse_answers`'s four: the union, the intersection, the first less the
/// second, and the second less the first.
fn which_answer(rule: Rule, first_first: bool) -> usize {
    match (rule, first_first) {
        (Rule::Union, _) => 0,
        (Rule::Intersection, _) => 1,
        (Rule::Difference, true) => 2,
        (Rule::Difference, false) => 3,
        (other, _) => panic!("{other:?} is not one of the six"),
    }
}

/// [`every_fuse_answers`], with every solid of every answer holding a corner
/// within `1e-9` of `corner`: where two loops meet, which an answer whose
/// loops run through it with no vertex there still validates.
fn every_fuse_answers_with_a_corner_at(
    build: Pair,
    volumes: [f64; 3],
    faces: [&[usize]; 4],
    corner: [f64; 3],
) {
    for one in six_fuses() {
        the_fuse_answers_with_a_corner_at(build, one, volumes, faces, corner);
    }
}

/// [`the_fuse_answers`], with every solid of the answer holding a corner
/// within `1e-9` of `corner`, as [`every_fuse_answers_with_a_corner_at`] says.
fn the_fuse_answers_with_a_corner_at(
    build: Pair,
    (rule, first_first): (Rule, bool),
    volumes: [f64; 3],
    faces: [&[usize]; 4],
    corner: [f64; 3],
) {
    let (model, solids) = the_fuse_answers(build, (rule, first_first), volumes, faces);
    for solid in solids {
        let view = model.solid(solid).expect("a solid the fuse made");
        let nearest = view
            .faces()
            .flat_map(|face| face.wires().flat_map(|wire| wire.points()))
            .map(|point| {
                let at = point.coordinates();
                (0..3)
                    .map(|i| (at[i] - corner[i]).powi(2))
                    .sum::<f64>()
                    .sqrt()
            })
            .fold(f64::INFINITY, f64::min);
        assert!(
            nearest <= 1e-9,
            "{rule:?} (first first: {first_first}): the corner nearest {corner:?} is {nearest} from it"
        );
    }
}

fn six_fuses() -> [(Rule, bool); 6] {
    [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Intersection, true),
        (Rule::Intersection, false),
        (Rule::Difference, true),
        (Rule::Difference, false),
    ]
}

fn a_model() -> Model {
    Model::new(rcad_geom::Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn fuse(
    model: &mut Model,
    build: impl Fn(&mut Model) -> [SolidKey; 2],
    rule: Rule,
    first_first: bool,
) -> rcad_topo::Result<Vec<SolidKey>> {
    fuse_under(model, build, rule, first_first, &Policy::certified())
}

fn fuse_under(
    model: &mut Model,
    build: impl Fn(&mut Model) -> [SolidKey; 2],
    rule: Rule,
    first_first: bool,
    policy: &Policy,
) -> rcad_topo::Result<Vec<SolidKey>> {
    let [first, second] = build(model);
    let (arguments, tools) = if first_first {
        ([first], [second])
    } else {
        ([second], [first])
    };
    model
        .fuse(&arguments, &tools, rule, policy, &mut ProvenanceLog::new())
        .map(|answer| answer.solids().to_vec())
}

/// Assert what [`measures`] does, and that together the solids measure
/// `volume` to within the mesh's own error, and hand the solids back. `what` is
/// the right answer, which is what every message carries.
fn answers(
    model: &mut Model,
    said: rcad_topo::Result<Vec<SolidKey>>,
    volume: f64,
    faces: &[usize],
    what: &str,
) -> Vec<SolidKey> {
    let (solids, measured, error) = measures(model, said, faces, what);
    assert!(
        (measured - volume).abs() <= error,
        "{what}: measured {measured} and the mesh may miss by {error}"
    );
    solids
}

/// Assert that `said` is one solid for each of `faces`, each with that many
/// faces, and that the model validates, and hand the solids back with the
/// volume they measure together and how far the mesh may miss it by. `what`
/// names the answer for every message.
fn measures(
    model: &mut Model,
    said: rcad_topo::Result<Vec<SolidKey>>,
    faces: &[usize],
    what: &str,
) -> (Vec<SolidKey>, f64, f64) {
    measures_under(model, said, faces, what, &Policy::certified())
}

/// [`measures`], with the check and the measure under `policy`.
fn measures_under(
    model: &mut Model,
    said: rcad_topo::Result<Vec<SolidKey>>,
    faces: &[usize],
    what: &str,
    policy: &Policy,
) -> (Vec<SolidKey>, f64, f64) {
    let solids = said.unwrap_or_else(|why| panic!("{what}: refused: {why:?}"));
    model
        .validate(policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: the answer does not validate: {why:?}"));
    let mut counted = Vec::with_capacity(solids.len());
    let (mut measured, mut error) = (0.0, 0.0);
    for &solid in &solids {
        let view = model.solid(solid).expect("a solid the fuse made");
        counted.push(view.faces().count());
        let measure = view
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: the answer does not measure: {why:?}"));
        measured += measure.volume();
        error += measure.volume_error();
    }
    assert_eq!(counted, faces, "{what}: faces of each solid");
    (solids, measured, error)
}

/// Assert that `said` is refused `Undecidable` on one question, the predicate
/// named `question`, asked of points that all lie within `1e-9` of `place`,
/// and hand back its record. `what` names the fuse for every message.
fn refused_on_one_question_at<'a>(
    said: &'a rcad_topo::Result<Vec<SolidKey>>,
    question: &str,
    place: [f64; 3],
    what: &str,
) -> &'a Record {
    let Err(rcad_topo::Error::Undecidable { records }) = said else {
        panic!("{what}: should be refused Undecidable, and said {said:?}");
    };
    let [refused] = records.records() else {
        panic!("{what}: should be refused on one question: {records:#?}");
    };
    assert_eq!(refused.predicate(), question, "{what}: {refused:#?}");
    for input in refused.inputs() {
        let sides = input.enclosure();
        let far = (0..3)
            .map(|i| {
                (sides[i].lo() - place[i])
                    .abs()
                    .max((sides[i].hi() - place[i]).abs())
            })
            .fold(0.0, f64::max);
        assert!(
            far <= 1e-9,
            "{what}: an input reaches {far} from {place:?}: {refused:#?}"
        );
    }
    refused
}

/// The plane and the cone of a pointed cone's two faces, as the model places
/// them.
fn the_cap_and_the_side(model: &Model, solid: SolidKey) -> (Plane, Cone) {
    let surfaces: Vec<Surface> = model
        .solid(solid)
        .expect("a solid of this model")
        .faces()
        .map(|face| face.surface())
        .collect();
    let ([Surface::Plane(cap), Surface::Cone(side)] | [Surface::Cone(side), Surface::Plane(cap)]) =
        surfaces[..]
    else {
        panic!("a pointed cone has a plane and a cone, and not {surfaces:?}");
    };
    (cap, side)
}

/// The pipe of a bar's wall, as the model places it.
fn the_wall(model: &Model, solid: SolidKey) -> Surface {
    model
        .solid(solid)
        .expect("a solid of this model")
        .faces()
        .map(|face| face.surface())
        .find(|surface| matches!(surface, Surface::Cylinder(_)))
        .expect("a bar's wall is a pipe")
}

/// The sign of `|w|² − 1`, decided exactly, and its value to a rounding.
fn squared_length_less_one(w: Vector) -> (Decision, f64) {
    let mut sum = vec![-1.0];
    for x in [w.x(), w.y(), w.z()] {
        let mut square = [0.0; 2];
        let squared = scale_expansion(&[x], x, &mut square);
        let mut wider = vec![0.0; sum.len() + squared];
        let len = expansion_sum(&sum, &square[..squared], &mut wider);
        wider.truncate(len);
        sum = wider;
    }
    (expansion_sign(&sum), sum.iter().sum())
}

/// How many curves `intersect` names where the two cross, `0` for a touch at
/// one point or nothing at all.
fn curves(one: &Surface, other: &Surface) -> Result<usize, String> {
    match intersect(one, other, &Policy::certified(), &mut ProvenanceLog::new()) {
        Ok(SurfaceMeet::Curves(runs)) => Ok(runs.len()),
        Ok(SurfaceMeet::Nothing | SurfaceMeet::TouchAtOnePoint { .. }) => Ok(0),
        said => Err(format!("{said:?}")),
    }
}

fn a_plane(through: [f64; 3], normal: [f64; 3]) -> Surface {
    Surface::Plane(
        Plane::from_point_and_normal(
            Point::new(through[0], through[1], through[2]),
            Vector::new(normal[0], normal[1], normal[2]),
        )
        .expect("a plane"),
    )
}

fn a_pipe(at: [f64; 2], radius: f64) -> Surface {
    let axis =
        Axis::new(Point::new(at[0], at[1], 0.0), Vector::new(0.0, 0.0, 1.0)).expect("an axis up z");
    Surface::Cylinder(Cylinder::new(axis, radius).expect("a pipe"))
}

fn a_ball(centre: [f64; 3], radius: f64) -> Surface {
    Surface::Sphere(
        Sphere::new(Point::new(centre[0], centre[1], centre[2]), radius).expect("a ball"),
    )
}

/// The cone up `z` with its apex at height `apex`, widening by `slope`.
fn a_cone(apex: f64, slope: f64) -> Surface {
    let axis =
        Axis::new(Point::new(0.0, 0.0, apex), Vector::new(0.0, 0.0, 1.0)).expect("an axis up z");
    Surface::Cone(Cone::new(axis, slope).expect("a cone"))
}

fn a_block_along_a_bar(model: &mut Model) -> [SolidKey; 2] {
    [
        a_bar(model, 2.0, [0.0, 0.0, 0.0], 4.0),
        a_block(model, [1.0, -5.0, 1.0], [5.0, 5.0, 3.0]),
    ]
}

fn two_bars_side_by_side(model: &mut Model) -> [SolidKey; 2] {
    [
        a_bar(model, 2.0, [0.0, 0.0, 0.0], 4.0),
        a_bar(model, 2.0, [2.0, 0.0, 1.0], 2.0),
    ]
}

fn a_ball_on_a_bars_axis(model: &mut Model) -> [SolidKey; 2] {
    a_ball_and_a_bar(model, &Isometry::identity(), 1.0, [0.0, 0.0])
}

fn a_cone_and_a_bar_on_one_axis(model: &mut Model) -> [SolidKey; 2] {
    let cone = cones::a_pointed_cone(model, 1.0, 4.0);
    [cone, a_bar(model, 1.0, [0.0, 0.0, -2.0], 8.0)]
}

fn a_block_beside_a_cones_apex(model: &mut Model) -> [SolidKey; 2] {
    let cone = cones::a_pointed_cone(model, 1.0, 4.0);
    [cone, a_block(model, [1.0, -6.0, -2.0], [6.0, 6.0, 6.0])]
}

fn a_thin_bar_through_a_thick_one(model: &mut Model) -> [SolidKey; 2] {
    [
        a_bar(model, 2.0, [0.0, 0.0, -2.0], 4.0),
        a_bar_along_x(model, 1.0, [-4.0, 0.5, 0.0], 8.0),
    ]
}

fn a_half_ring_and_a_half_bar_on_its_axis(model: &mut Model) -> [SolidKey; 2] {
    a_half_ring_and_a_half_bar_with_its_floor_at(model, -2.0)
}

/// The top half of a ring of 4 round a tube of 1 about `z`, and the `x ≥ 0`
/// half of a bar of radius 10 about `z` from `floor` up to 5.
fn a_half_ring_and_a_half_bar_with_its_floor_at(model: &mut Model, floor: f64) -> [SolidKey; 2] {
    let ring = rings::the_top_half_of_a_ring(model, 4.0, 1.0, false).expect("a half ring");
    let bar = a_bar(model, 10.0, [0.0, 0.0, floor], 5.0 - floor);
    let block = a_block(model, [0.0, -20.0, floor], [20.0, 20.0, 6.0]);
    let halved = model
        .fuse(
            &[bar],
            &[block],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a block cuts a bar in half");
    let [half] = halved.solids()[..] else {
        panic!("half a bar is one solid")
    };
    [ring, half]
}

fn a_bar_touching_a_ball_from_inside(model: &mut Model) -> [SolidKey; 2] {
    a_ball_and_a_bar(model, &Isometry::identity(), 1.0, [1.0, 0.0])
}

fn two_bars_of_one_radius_whose_axes_meet(model: &mut Model) -> [SolidKey; 2] {
    [
        a_bar(model, 1.0, [0.0, 0.0, -2.0], 4.0),
        a_bar_along_x(model, 1.0, [-2.0, 0.0, 0.0], 4.0),
    ]
}

fn a_thin_bar_resting_on_the_inside_of_a_thick_one(model: &mut Model) -> [SolidKey; 2] {
    [
        a_bar(model, 2.0, [0.0, 0.0, -2.0], 4.0),
        a_bar_along_x(model, 1.0, [-4.0, 1.0, 0.0], 8.0),
    ]
}

fn a_thin_bar_up_the_inside_of_a_thick_one(model: &mut Model) -> [SolidKey; 2] {
    [
        a_bar(model, 2.0, [0.0, 0.0, -2.0], 4.0),
        a_bar(model, 1.0, [1.0, 0.0, -3.0], 6.0),
    ]
}

/// The bar of radius 2 up `z` from `−2` to 2, and the bar of radius 1 up `z`
/// from `(x, 0, −3)`, 6 long, turned `lean` about `x`.
fn a_thin_bar_turned_inside_a_thick_one(model: &mut Model, x: f64, lean: f64) -> [SolidKey; 2] {
    let thick = a_bar(model, 2.0, [0.0, 0.0, -2.0], 4.0);
    let thin = a_bar(model, 1.0, [x, 0.0, -3.0], 6.0);
    model
        .place_solid(
            thin,
            &Isometry::rotation(&Axis::x(), lean).expect("a finite turn"),
        )
        .expect("this model made the key");
    [thick, thin]
}

fn a_half_ring_cut_by_a_round_plate(model: &mut Model) -> [SolidKey; 2] {
    let ring = rings::the_top_half_of_a_ring(model, 4.0, 1.0, false).expect("a half ring");
    [ring, a_bar(model, 6.0, [0.0, 0.0, -2.0], 2.5)]
}

/// The top half of a ring of 4 round a tube of 1 about `z`, and a round plate
/// of radius 10 about `z` from `z = −1`, where the whole ring would rest, up
/// to `top`.
fn a_half_ring_bead_on_a_plate_up_to(model: &mut Model, top: f64) -> [SolidKey; 2] {
    let ring = rings::the_top_half_of_a_ring(model, 4.0, 1.0, false).expect("a half ring");
    [ring, a_bar(model, 10.0, [0.0, 0.0, -1.0], top + 1.0)]
}

/// The four balls with a bar through them off their axis, each with the bar's
/// radius and how far its axis is off the ball's centre.
fn balls_with_a_bar_through_them_off_their_axis() -> [(Pair, f64, f64); 4] {
    [
        (
            |model| a_ball_and_a_bar(model, &Isometry::identity(), 1.0, [0.5, 0.0]),
            1.0,
            0.5,
        ),
        (
            |model| a_ball_and_a_bar(model, &Isometry::identity(), 1.0, [0.0, 0.5]),
            1.0,
            0.5,
        ),
        (
            |model| a_ball_and_a_bar(model, &Isometry::identity(), 0.5, [0.0, 1.0]),
            0.5,
            1.0,
        ),
        (
            |model| {
                let turned = Isometry::quarter_turns(&Axis::z(), 1).expect("a turn about z");
                a_ball_and_a_bar(model, &turned, 1.0, [0.5, 0.0])
            },
            1.0,
            0.5,
        ),
    ]
}

/// A ball of radius 2, placed by `ball`, and a bar of `radius` up `z` from −3
/// to 3 at `at`.
fn a_ball_and_a_bar(
    model: &mut Model,
    ball: &Isometry,
    radius: f64,
    at: [f64; 2],
) -> [SolidKey; 2] {
    let ball = model.add_ball(2.0, ball).expect("a ball");
    [ball, a_bar(model, radius, [at[0], at[1], -3.0], 6.0)]
}

fn a_bar_seamed_up_y_and_a_ball_in_its_side(model: &mut Model) -> [SolidKey; 2] {
    let seam_up_y = Isometry::translation(Vector::new(0.0, 0.0, -1.0))
        .expect("a finite move")
        .compose(&Isometry::quarter_turns(&Axis::z(), 1).expect("a turn about z"));
    let bar = model.add_round_bar(2.0, 12.0, &seam_up_y).expect("a bar");
    let ball = model
        .add_ball(
            3.0,
            &Isometry::translation(Vector::new(0.0, 3.0, 5.0)).expect("a finite move"),
        )
        .expect("a ball");
    [bar, ball]
}

fn two_cones_on_one_apex_leaning_apart(model: &mut Model) -> [SolidKey; 2] {
    two_cones_on_one_apex_turned_apart(model, 8.0)
}

fn a_short_cone_on_the_apex_of_a_tall_one_leaning_apart(model: &mut Model) -> [SolidKey; 2] {
    two_cones_on_one_apex_turned_apart(model, 6.0)
}

/// The union of [`a_short_cone_on_the_apex_of_a_tall_one_leaning_apart`]'s two
/// cones, and the block `[−10, 10]² × [7, 10]` over its top.
fn a_block_over_the_union_of_a_short_cone_on_the_apex_of_a_tall_one(
    model: &mut Model,
) -> [SolidKey; 2] {
    let said = fuse(
        model,
        a_short_cone_on_the_apex_of_a_tall_one_leaning_apart,
        Rule::Union,
        true,
    )
    .expect("the union answers");
    let [union] = said[..] else {
        panic!("the union of two cones that overlap is one solid: {said:?}");
    };
    [
        union,
        a_block(model, [-10.0, -10.0, 7.0], [10.0, 10.0, 10.0]),
    ]
}

/// A pointed cone of slope ½ up `z` to 8, and one up to `far` turned 0.3 about
/// `x` through the apex the two share at the origin.
fn two_cones_on_one_apex_turned_apart(model: &mut Model, far: f64) -> [SolidKey; 2] {
    let upright = cones::a_pointed_cone(model, 0.5, 8.0);
    let turned = cones::a_pointed_cone(model, 0.5, far);
    model
        .place_solid(
            turned,
            &Isometry::rotation(&Axis::x(), 0.3).expect("a finite turn"),
        )
        .expect("this model made the key");
    [upright, turned]
}

fn a_block(model: &mut Model, from: [f64; 3], to: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [to[0] - from[0], 0.0],
        [to[0] - from[0], to[1] - from[1]],
        [0.0, to[1] - from[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle is a profile");
    let placed =
        Isometry::translation(Vector::new(from[0], from[1], from[2])).expect("a finite move");
    model
        .add_extrusion(&rectangle, to[2] - from[2], &placed)
        .expect("a rectangle extrudes")
}

fn a_bar(model: &mut Model, radius: f64, at: [f64; 3], height: f64) -> SolidKey {
    let placed = Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a finite move");
    model.add_round_bar(radius, height, &placed).expect("a bar")
}

/// A bar of `radius` and `length` along `+x` from `at`, turned onto `x` by an
/// exact quarter turn about `y`.
fn a_bar_along_x(model: &mut Model, radius: f64, at: [f64; 3], length: f64) -> SolidKey {
    let turned = Isometry::quarter_turns(&Axis::y(), 1).expect("a turn about y");
    let placed = Isometry::translation(Vector::new(at[0], at[1], at[2]))
        .expect("a finite move")
        .compose(&turned);
    model.add_round_bar(radius, length, &placed).expect("a bar")
}

/// A bar of `radius` and `length` along `+y` from `at`, turned onto `y` by
/// three exact quarter turns about `x`.
fn a_bar_along_y(model: &mut Model, radius: f64, at: [f64; 3], length: f64) -> SolidKey {
    let turned = Isometry::quarter_turns(&Axis::x(), 3).expect("a turn about x");
    let placed = Isometry::translation(Vector::new(at[0], at[1], at[2]))
        .expect("a finite move")
        .compose(&turned);
    model.add_round_bar(radius, length, &placed).expect("a bar")
}

/// Volume of the pointed cone `ρ ≤ z`, `0 ≤ z ≤ far`, past the plane `x = d`:
/// `∫ z² acos(d/z) − d√(z² − d²) dz` from `d` to `far`.
fn cone_beyond(d: f64, far: f64) -> f64 {
    let root = (far * far - d * d).sqrt();
    let log = ((far + root) / d).ln();
    far.powi(3) / 3.0 * (d / far).acos()
        - d / 6.0 * (far * root + d * d * log)
        - d / 2.0 * (far * root - d * d * log)
}

/// Volume the bar of `big` along `z` shares with the bar of `small` along `x`
/// at `y = off`, by a 20 000-step midpoint rule round the thin bar's section.
fn bar_through_bar(big: f64, small: f64, off: f64) -> f64 {
    let steps = 20_000;
    let mut sum = 0.0;
    for step in 0..steps {
        let theta = -PI / 2.0 + PI * (f64::from(step) + 0.5) / f64::from(steps);
        let y = off + small * theta.sin();
        let across = 2.0 * (big * big - y * y).sqrt() * 2.0 * small * theta.cos();
        sum += across * small * theta.cos() * PI / f64::from(steps);
    }
    sum
}

/// Volume the ball of radius 2 at the origin shares with a bar up `z` of
/// `radius` whose axis is `off` from the ball's, long enough to go right
/// through it: `∫∫ 2√(4 − ρ² − off² − 2·off·ρ·cos θ) ρ dρ dθ` over the bar's
/// section, `ρ` out from the bar's axis.
///
/// A midpoint rule of 1000 steps out from the axis by 64 round it. The
/// integrand is smooth and periodic round the axis, so 64 steps there already
/// agree with 16 to `1e-12`; out from the axis the error falls as the square
/// of the step, and on the axis 1000 steps leave it `4.4e-7` above the closed
/// form ([`the_ball_in_a_bar_on_its_axis_is_the_closed_form`]). Off the axis,
/// read as `4/3` of how far 1000 steps are from 2000, it is `4.9e-7` over for
/// a bar of 1 at `1/2` and `3.6e-8` for a bar of `1/2` at 1: far below the
/// `0.12` or more that a mesh of `0.01` may miss these volumes by. The bar
/// must lie inside the ball's equator, `radius + off < 2`.
fn ball_in_bar(radius: f64, off: f64) -> f64 {
    let (out, round) = (1000, 64);
    let (step_out, step_round) = (radius / f64::from(out), 2.0 * PI / f64::from(round));
    let mut sum = 0.0;
    for i in 0..out {
        let rho = step_out * (f64::from(i) + 0.5);
        for j in 0..round {
            let theta = step_round * (f64::from(j) + 0.5);
            let squared = 4.0 - rho * rho - off * off - 2.0 * off * rho * theta.cos();
            sum += 2.0 * squared.sqrt() * rho;
        }
    }
    sum * step_out * step_round
}

/// Volume the ball of radius 3 at `(0, 3)` shares with the bar of radius 2
/// whose axis runs through the origin, both up `z` and the bar long enough to
/// hold the ball's whole height there, by a midpoint rule of `steps` across
/// `x`.
///
/// Seen down `z`, what they share stands on the lens where the bar's circle
/// and the ball's meet, at `x = ±4√2/3`, `y = 2/3`. At each `x` the lens runs
/// up `y` from the ball's edge `3 − c`, `c = √(9 − x²)`, to the bar's
/// `√(4 − x²)`, and the ball is `2√(c² − (y − 3)²)` high there, so with
/// `t = y − 3` the strip is `t√(c² − t²) + c² asin(t/c)` from `−c` to the
/// bar's edge: closed form. Only the rule across `x` is approximate, and
/// [`the_ball_in_the_side_of_a_bar_is_within_a_millionth_of_a_rule_a_hundred_times_finer`]
/// says how far.
fn ball_in_the_side_of_a_bar(steps: u32) -> f64 {
    let edge = 4.0 * 2f64.sqrt() / 3.0;
    let step = 2.0 * edge / f64::from(steps);
    let mut sum = 0.0;
    for i in 0..steps {
        let x = -edge + step * (f64::from(i) + 0.5);
        let squared = 9.0 - x * x;
        let c = squared.sqrt();
        let top = (4.0 - x * x).sqrt() - 3.0;
        sum += top * (squared - top * top).sqrt() + squared * (top / c).asin() + squared * PI / 2.0;
    }
    sum * step
}
