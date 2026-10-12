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
//! two, refuses the shape by name before any arrangement, or cannot make the
//! surface at all. The one surface with a point and no normal is a cone, at
//! its apex, and that is the node §347.1 built. Two pipes of one radius whose
//! axes meet are tangent at two points, and their crossing reaches them
//! without being pushed there: that is the Steinmetz cross, which gets its
//! nodes, and every fuse of it answers.
//!
//! The fuses run three rules both ways round, certified, in a model of 100 mm.

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, Isometry, Plane, Point, Sphere, Surface, SurfaceMeet, Torus,
    Vector, WhereFollowingACrossingGaveUp, intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

mod cones;
mod rings;

use std::f64::consts::PI;

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
/// pipe touches the thick one from inside, and `intersect` refuses that
/// (`a_thin_bar_resting_on_the_inside_of_a_thick_one_is_refused_before_any_arrangement`).
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
/// touches the ball from inside at `(2, 0, 0)`, and the two loops it would
/// cross the ball in meet there, Viviani's figure of eight. `intersect`
/// refuses it, so no arrangement is ever asked to put the node.
#[test]
fn a_bar_touching_a_ball_from_inside_is_refused_before_any_arrangement() {
    every_fuse_is_refused_where_two_loops_meet(a_bar_touching_a_ball_from_inside);
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
/// touches it from inside along `(x, 2, 0)`: the two loops of
/// `a_thin_bar_through_a_thick_one_crosses_it_in_two_loops_and_every_fuse_answers`
/// pushed together until they meet. `intersect` refuses it.
#[test]
fn a_thin_bar_resting_on_the_inside_of_a_thick_one_is_refused_before_any_arrangement() {
    every_fuse_is_refused_where_two_loops_meet(a_thin_bar_resting_on_the_inside_of_a_thick_one);
}

/// A ball of radius 2 with a bar from `z = −3` to 3 through it off its axis,
/// so that the pipe crosses the ball in two loops apart, one above `z = 0` and
/// one below. They come together only where the bar touches the ball from
/// inside
/// (`a_bar_touching_a_ball_from_inside_is_refused_before_any_arrangement`).
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
/// is the start, `rcad-geom` is asked which way the loop runs and refuses by
/// name (`a_ball_driven_into_the_bar_is_refused_ordering_the_loop_they_cross_in`
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
/// `z = −1`. That circle lies on no face of the half ring, yet every fuse is
/// refused ordering points along it.
#[test]
fn a_half_bar_whose_floor_the_whole_ring_would_rest_on_is_refused_ordering_that_circle() {
    for (rule, first_first) in six_fuses() {
        let mut model = a_model();
        let said = fuse(
            &mut model,
            |model| a_half_ring_and_a_half_bar_with_its_floor_at(model, -1.0),
            rule,
            first_first,
        );
        assert!(
            matches!(
                said,
                Err(rcad_topo::Error::Geometry(
                    rcad_geom::Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
                        at: WhereFollowingACrossingGaveUp::OrderingTheCircleAPlaneRestsOnARingAlong,
                        ..
                    }
                ))
            ),
            "{rule:?} (ring first: {first_first}): {said:?}"
        );
    }
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

fn every_fuse_answers(build: Pair, volumes: [f64; 3], faces: [&[usize]; 4]) {
    for one in six_fuses() {
        the_fuse_answers(build, one, volumes, faces);
    }
}

/// The fuse `rule` of `build`'s two solids, the first first or not, answers
/// with the volume and faces `every_fuse_answers` reads for it.
fn the_fuse_answers(
    build: Pair,
    (rule, first_first): (Rule, bool),
    [first, second, shared]: [f64; 3],
    faces: [&[usize]; 4],
) {
    let (volume, faces) = match (rule, first_first) {
        (Rule::Union, _) => (first + second - shared, faces[0]),
        (Rule::Intersection, _) => (shared, faces[1]),
        (Rule::Difference, true) => (first - shared, faces[2]),
        (Rule::Difference, false) => (second - shared, faces[3]),
        (other, _) => panic!("{other:?} is not one of the six"),
    };
    let mut model = a_model();
    let said = fuse(&mut model, build, rule, first_first);
    let what = format!(
        "{rule:?} (first first: {first_first}), solids of {faces:?} faces and volume {volume}"
    );
    answers(&mut model, said, volume, faces, &what);
}

fn every_fuse_is_refused_where_two_loops_meet(build: Pair) {
    for (rule, first_first) in six_fuses() {
        let mut model = a_model();
        let said = fuse(&mut model, build, rule, first_first);
        assert!(
            matches!(
                said,
                Err(rcad_topo::Error::Geometry(
                    rcad_geom::Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo
                ))
            ),
            "{rule:?} (first first: {first_first}): {said:?}"
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
    build: Pair,
    rule: Rule,
    first_first: bool,
) -> rcad_topo::Result<Vec<SolidKey>> {
    let [first, second] = build(model);
    let (arguments, tools) = if first_first {
        ([first], [second])
    } else {
        ([second], [first])
    };
    model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().to_vec())
}

/// Assert that `said` is one solid for each of `faces`, each with that many
/// faces, that the model validates, and that together they measure `volume`
/// to within the mesh's own error. `what` is the right answer, which is what
/// every message carries.
fn answers(
    model: &mut Model,
    said: rcad_topo::Result<Vec<SolidKey>>,
    volume: f64,
    faces: &[usize],
    what: &str,
) {
    let policy = Policy::certified();
    let solids = said.unwrap_or_else(|why| panic!("{what}: refused: {why:?}"));
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: the answer does not validate: {why:?}"));
    let mut counted = Vec::with_capacity(solids.len());
    let (mut measured, mut error) = (0.0, 0.0);
    for solid in solids {
        let view = model.solid(solid).expect("a solid the fuse made");
        counted.push(view.faces().count());
        let measure = view
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("the answer measures");
        measured += measure.volume();
        error += measure.volume_error();
    }
    assert_eq!(counted, faces, "{what}: faces of each solid");
    assert!(
        (measured - volume).abs() <= error,
        "{what}: measured {measured} and the mesh may miss by {error}"
    );
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

fn a_half_ring_cut_by_a_round_plate(model: &mut Model) -> [SolidKey; 2] {
    let ring = rings::the_top_half_of_a_ring(model, 4.0, 1.0, false).expect("a half ring");
    [ring, a_bar(model, 6.0, [0.0, 0.0, -2.0], 2.5)]
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
