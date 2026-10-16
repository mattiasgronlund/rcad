//! Two nodes merged round a closed crossing by a tolerant policy that read
//! what it was handed as a volume: first two places a ball is cut at, whose
//! boxes' width is a length, then two corners round a cone's ellipse, whose
//! sweep from the start is an area.
//!
//! # The ball's shape
//!
//! `a_tilted_wall_grazing_a_ball_sums_to_zero.rs`'s, lifted by `1008` so the
//! ball's centre is at `z = 1024`. A prism on the triangle `(0, 0)`, `(4, 3)`,
//! `(−3, 3)`, from `z = 1008` to `1040`, whose walls `3x = 4y` and `x = −y`
//! both hold the `z` axis, and a ball of radius `1 + 2⁻⁵¹` at
//! `(1, 2⁻²⁵, 1024)`. The `z` axis cuts the ball at `z = 1024 ± 2⁻⁵¹`, two
//! places `2⁻⁵⁰` apart, which come back as boxes on the axis whose `z` sides
//! overlap and are `1.5e-12` wide: an `f64` step at `1024` is `2.3e-13`.
//!
//! Round either wall's circle on the ball's face the two places and the start,
//! `(0, 0, 1024)`, are on one line, so the sweep is exactly zero and the order
//! goes to the arm of `round_a_closed_crossing` that answers as wide as the
//! wider box. Q699 does not order them: both boxes hold the start inside them.
//!
//! # What this pins
//!
//! A tolerant policy at a 25 mm working size has a budget of `64 ε · 25 =
//! 3.6e-13` for a length. That arm's refusals were filed as a volume, read as
//! the width over `25²`, so the two places were merged on the ball's face,
//! four times, the policy accepting a box four times wider than its budget.
//! Every fuse then refused all the same, on the order of the two places along
//! the edge the two walls share, a length. Filed as a length, the ball's face
//! refuses first, and the log holds nothing the budget did not cover.
//!
//! **No answer changed.** 168 fuses were searched for one that the merge let
//! through: this shape lifted by `0`, `240`, `1008`, `4080`, `65520` and
//! `1048560` under working sizes of `1`, `25`, `100` and `1000`, and
//! `two_meets_in_one_box_on_a_closed_crossing.rs`'s block and ball at `1e10`
//! under `25`, `100`, `1000` and `10000`, three rules each, either solid
//! first. 21 of them merged two places a length budget refuses, and each of
//! the 21 refused on the edge next. The two places are meets of two planes and
//! a ball, so they are on the planes' straight crossing too, where their order
//! was always filed as a length.
//!
//! # The cone's shape
//!
//! `a_plane_cut_along_a_cones_other_nappe.rs`'s steep cone, slope two about
//! `z` from its apex at the origin up to a cap at `z = 3`, and a cube of side
//! six stood on its corner with its middle on the apex. Three of the cube's
//! faces cut the cone in ellipses, and two corners on one of them are ordered
//! round it by the sweep `((one − from) × (other − from)) · n`, with `n` the
//! face's normal, of length one. So the sweep is an area. Its enclosure is
//! about `1e-13`, and a certified fuse refuses on it.
//!
//! # What this pins
//!
//! A tolerant policy at a 2 mm working size has a budget of `64 ε · 2 =
//! 2.8e-14` for a length. The sweep was filed as a volume, read as its
//! enclosure over `2²`, which that budget covers for every enclosure up to
//! the widest, `1.06e-13`, so the corners were merged and every fuse
//! answered. Q1273's probe held those answers to the checks of the nappe
//! file, with the model at 100 mm and the policy at 2 to 2.6 mm, and each was
//! right. Read over `2`, the enclosure the order refuses on is past the
//! budget, and every fuse refuses. An area budget covers the widest from a
//! working size of `2.73` up.
//!
//! # Below a working size of one
//!
//! Below `1`, the area reading is the more lenient one, by `1/s`: at a shape
//! scale of `0.18` and a working size of `0.6`, the cube on the cone's apex
//! merges two corners the volume reading would have refused — a real gap,
//! searched across 13 shape scales (`0.1` to `0.5`), 17 working sizes (`0.1`
//! to `0.9`) and the four rule/order combinations above (884 fuses, no
//! panics). **No wrong answer came of it.**
//! `a_tolerant_fuse_of_a_cube_on_a_cone_below_working_size_one_merges_the_same_shape_as_at_full_scale`
//! pins one such merge and compares its solid's face, wire and shell counts
//! against the same pair fused at full scale and a 10 mm working size — past
//! `2.73`, so trusted — and they agree: fusing is a similarity transform, so
//! scaling the shape and the working size together cannot change which of two
//! points comes first round the ellipse.
//!
//! The same gap shows up for "frustum cut near its seam": swap the cone's
//! sharp apex for `a_frustum_seamed_towards`'s flat near cap, cut at a
//! three-hundredth of the far end (`far / 300`), and the same 13-scale x
//! 17-size x 4-rule grid merges two corners the volume reading would have
//! refused at, among others, a shape scale of `0.26` and a working size of
//! `0.75`. The seam's own direction never matters to where the two faces of
//! the cube cross the cone (`probe_family_a_angles`, run during this search,
//! found the same crossing azimuth whichever of the four whole directions the
//! seam was cut towards), so this result stands for "bar cut along its seam"
//! too — a bar is `a_frustum_seamed_towards` with its near and far close
//! together; a true cylinder-faced bar, which needs a slope of zero and so
//! cannot be built this way, was not tried separately.
//! `a_tolerant_fuse_of_a_cube_on_a_frustum_cut_near_its_seam_below_working_size_one_merges_the_same_shape_as_at_full_scale`
//! pins the `0.26`/`0.75` merge the same way. **No wrong answer came of it
//! either.**
//!
//! # The far rim the cube's edges run through
//!
//! A near cap cut this close (`far / 300`) sometimes refuses earlier, with
//! `CannotTellTheCellsOfAFaceApart`, before the crossing is even reached.
//! Q2028 traced it, and it is not the near cap. In a grid of 21 shape scales
//! (`0.1` to `0.5`), 17 working sizes (`0.1` to `0.9`) and the four fuses,
//! all 304 of these refusals came from `spread_the_winding` in
//! `fuse/arrange.rs`, on the frustum's far cap.
//!
//! Each of the six edges round the cube's middle, where an upper face meets a
//! lower one, carried on past its upper corner, runs exactly through the far
//! rim: at a scale of one it reaches `z = 3` at `6` from the axis, and a cone
//! of slope two is `6` from the axis there. So four surfaces meet at one
//! point, the cap, the cone and two faces of the cube, and on the cap that
//! point is three meets: the rim with each face's line, and the two lines
//! with each other. Stored in `f64` they come back as three boxes a few
//! `1e-16` apart, and `merge_points_that_are_one_point` decides them a pair at
//! a time. At a shape scale of `0.1` and a working size of `0.25` it joins the
//! rim's meet with one line to the meet of the two lines, and proves the
//! rim's meet with the other line apart from both. That other line then meets
//! the rim twice, a hair apart, and `spread_the_winding` cannot tell the
//! cap's cells apart. The answer exists, so this is a gap (§16.10).
//!
//! **No answer was wrong.** The 48 fuses of that grid that answered pass
//! `Model::validate`, have the solid, face, wire and shell counts of the same
//! fuse at a scale of one and a working size of `10`, and its volume to
//! within `1.9e-5`. Lifting the far cap by a thousandth of its height takes it
//! off the edges' lines, and all 1428 fuses answer. Turning the seam to `+x`
//! still leaves 347 refusals, so the seam plays no part. The two power-of-two
//! images of a scale of `0.1` and a size of `0.25`, which round the same way,
//! refuse the same, so small numbers play no part either.
//! `a_tolerant_fuse_of_a_cube_whose_edges_run_through_a_frustums_far_rim_cannot_tell_the_caps_cells_apart`
//! and `the_same_fuses_answer_once_the_far_cap_is_lifted_off_the_cubes_edges`
//! pin both.
//!
//! A designer draws the same kind of meeting on purpose: a round bar notched
//! by a block whose inside corner stands on the bar's wall, at `(0.6 r, 0.8
//! r)`. At a radius of `12.8` or `25.6` and working sizes of `10`, `25` and
//! `100`, every fuse is refused from the same place in `spread_the_winding`,
//! on the bar's wall, with two meets whose boxes overlap kept apart.
//! `a_tolerant_fuse_of_a_round_bar_and_a_block_cornered_on_its_wall_cannot_tell_the_walls_cells_apart`
//! pins the `12.8`/`25` case. At other radii the same bar is refused
//! `ResultIsNotManifold` or `TheFuseWroteABrokenAnswer`, or runs for minutes;
//! those are rows of their own.
//!
//! # Plane through a ball
//!
//! The same gap shows up a third time for "plane through a ball", the one
//! remaining family a plane's conic sweep runs through
//! (`along_a_crossing.rs`'s `WhichWayItWinds::About` arm, reached for a plane
//! and a sphere at `along_a_crossing.rs:401-402` unconditionally — a sphere
//! has no apex or seam to force the walk's start near, so the construction
//! that reaches it mirrors the cone's apex case directly: a cube stood on its
//! corner with a ball centred at the cube's own centre, its radius the
//! cube's half-diagonal, so the cube's one near vertex sits exactly on the
//! ball's surface. Several of the cube's faces then cut the ball in circles
//! close to that one vertex, the same way three of the cube's faces cut the
//! cone's apex in ellipses.
//!
//! A `5`-scale x `7`-size x `4`-rule grid (narrower than the cone's own,
//! since a ball has no seam direction to vary) finds the gap at, among
//! others, a shape scale of `0.18` and a working size of `0.6` — the cone's
//! own pin values, shared by chance of the two narrower grids overlapping,
//! not because the shapes are the same.
//! `a_tolerant_fuse_of_a_cube_on_a_ball_through_its_corner_below_working_size_one_merges_the_same_shape_as_at_full_scale`
//! pins that merge the same way the other two families' tests do. **No wrong
//! answer came of this one either.**
//!
//! A first attempt at this family — tilting `a_ball_grazing_the_z_axis`'s
//! wall, and later a three-solid construction meant to separate the walk's
//! start from the forced collinearity a shared edge gives a block's own
//! corner — never reached the crossing-order predicate at all, refusing
//! earlier on an unrelated question at every tried tolerance. Reading
//! `cascade.rs`'s `decide` (the function every predicate in this file's
//! search goes through) explains why: *"the one case that comes back
//! `Decision::Undecidable` is an interval that straddles zero when level 3
//! cannot run. For exact `f64` inputs that never happens, short of an
//! intermediate product overflowing or falling into the subnormals."* Making
//! a line through a ball's circle *exactly* collinear or *exactly* tangent
//! does not reach `Undecidable` by itself — stored, exact coordinates always
//! get a certified sign at the exact level, however close together. What
//! reaches it is the ordinary thing every other family's gap search relies
//! on: a point that already is not exact, because the walk's three points in
//! a real fuse are new vertices, built by arithmetic on a genuine crossing,
//! not hand-picked exact coordinates — the same way the cone's and the
//! frustum's gaps were found, by a grid search over ordinary shapes rather
//! than by engineering an exact degeneracy.

mod cones;

use cones::{a_frustum_seamed_towards, a_pointed_cone};
use rcad_geom::{
    Axis, Isometry, MeetOfThree, Point, Scale, Surface, Vector, meet_of_three_surfaces,
};
use rcad_predicates::{Level, Measures, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Error, Fused, Model, ProfileBuilder, Rule, SolidKey};

/// The name `order_along` records a refusal under.
const ALONG_A_CROSSING: &str = "order along a crossing";

/// How far the shape is lifted along `z`.
const LIFT: f64 = 1008.0;

/// The working size of the model, and of the policy.
const WORKING_SIZE: f64 = 25.0;

const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

/// The window the regression sits in: the wider box is wider than the
/// policy's budget read as a length, and narrower than it read as a volume.
#[test]
fn the_two_places_boxes_are_past_a_length_budget_and_inside_a_volume_one() {
    let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"));
    let prism = a_prism_with_two_walls_through_the_z_axis(&mut model);
    let ball = a_ball_grazing_the_z_axis(&mut model);
    let places = where_the_edge_cuts_the_ball(
        the_walls_through_the_z_axis(&model, prism),
        the_only_surface_of(&model, ball),
    );
    for place in places {
        let [x, y, z] = place.enclosure();
        assert_eq!([x.lo(), x.hi(), y.lo(), y.hi()], [0.0; 4], "{place:?}");
        assert!(z.lo() < 1024.0 && 1024.0 < z.hi(), "{place:?}");
    }
    let widest = places[0].uncertainty().max(places[1].uncertainty());
    let policy = Policy::tolerant(WORKING_SIZE);
    assert!(
        !policy.wobble_is_acceptable(Measures::Length.wobble_of(widest, WORKING_SIZE)),
        "{widest} against {}",
        policy.max_wobble()
    );
    assert!(
        policy.wobble_is_acceptable(Measures::Volume.wobble_of(widest, WORKING_SIZE)),
        "{widest} against {}",
        policy.max_wobble()
    );
}

/// The regression: with the ball first, every tolerant fuse refuses on the
/// order of the two places round a wall's circle, filed as a length, as wide
/// as the wider box, and with nothing else in its log.
///
/// Filed as a volume, the log held four merges of the two places that the
/// policy accepted, and then the refusal along the walls' edge.
#[test]
fn a_tolerant_fuse_refuses_to_merge_the_two_places_round_a_walls_circle() {
    for rule in EVERY_RULE {
        let mut model = Model::new(Scale::millimetres(WORKING_SIZE).expect("a working size"));
        let prism = a_prism_with_two_walls_through_the_z_axis(&mut model);
        let ball = a_ball_grazing_the_z_axis(&mut model);
        let places = where_the_edge_cuts_the_ball(
            the_walls_through_the_z_axis(&model, prism),
            the_only_surface_of(&model, ball),
        );
        let widest = places[0].uncertainty().max(places[1].uncertainty());
        let fused = model.fuse(
            &[ball],
            &[prism],
            rule,
            &Policy::tolerant(WORKING_SIZE),
            &mut ProvenanceLog::new(),
        );
        let Err(Error::Undecidable { records }) = fused else {
            panic!("{rule:?}: {:?}", fused.map(|_: Fused| "an answer"));
        };
        let records = records.records().to_vec();
        let [refused] = &records[..] else {
            panic!("{rule:?}: {records:#?}");
        };
        let inputs = refused.inputs();
        assert_eq!(refused.predicate(), ALONG_A_CROSSING, "{rule:?}");
        assert_eq!(refused.level(), Level::Interval, "{rule:?}");
        assert!(
            inputs == places || inputs == [places[1], places[0]],
            "{rule:?}: {refused:#?}"
        );
        assert_eq!(refused.measures(), Measures::Length, "{rule:?}");
        assert_eq!(refused.enclosure(), widest, "{rule:?}");
    }
}

fn where_the_edge_cuts_the_ball(walls: [Surface; 2], skin: Surface) -> [PredicatePoint; 2] {
    let mut log = ProvenanceLog::new();
    let meet = meet_of_three_surfaces(
        [&walls[0], &walls[1], &skin],
        &Policy::certified(),
        &mut log,
    )
    .expect("a certified answer");
    assert!(log.is_empty(), "{log:?}");
    let MeetOfThree::At(places) = meet else {
        panic!("the z axis cuts the ball: {meet:?}");
    };
    let [one, other] = places[..] else {
        panic!("a line that nearly touches a ball cuts it twice: {places:?}");
    };
    assert_ne!(one, other, "two boxes, not one");
    [one, other]
}

fn a_prism_with_two_walls_through_the_z_axis(model: &mut Model) -> SolidKey {
    let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 3.0], [-3.0, 3.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a triangle");
    model
        .add_extrusion(
            &triangle,
            32.0,
            &Isometry::translation(Vector::new(0.0, 0.0, LIFT)).expect("a finite shift"),
        )
        .expect("a positive height")
}

fn a_ball_grazing_the_z_axis(model: &mut Model) -> SolidKey {
    model
        .add_ball(
            1.0 + f64::powi(2.0, -51),
            &Isometry::translation(Vector::new(1.0, f64::powi(2.0, -25), LIFT + 16.0))
                .expect("a finite shift"),
        )
        .expect("a positive radius")
}

/// The two walls whose planes hold the `z` axis, as the fuse is handed them.
fn the_walls_through_the_z_axis(model: &Model, prism: SolidKey) -> [Surface; 2] {
    let walls: Vec<Surface> = model
        .solid(prism)
        .expect("the prism")
        .faces()
        .map(|face| face.surface())
        .filter(|surface| {
            matches!(surface, Surface::Plane(flat)
                if flat.normal().components()[2] == 0.0 && flat.offset() == 0.0)
        })
        .collect();
    let [one, other] = walls[..] else {
        panic!("a triangle with a corner at the origin has two walls there: {walls:?}");
    };
    [one, other]
}

fn the_only_surface_of(model: &Model, solid: SolidKey) -> Surface {
    let mut faces = model.solid(solid).expect("the ball").faces();
    let surface = faces.next().expect("a face").surface();
    assert!(faces.next().is_none(), "a ball is one face");
    surface
}

/// The working size of the model the cube and the cone are fused in, and of
/// the policy.
const THE_CUBES_WORKING_SIZE: f64 = 2.0;

/// Each rule, and for the difference which solid goes first: whether the
/// curved one, the cone, the frustum or the bar, is the argument.
const EVERY_FUSE: [(Rule, bool); 4] = [
    (Rule::Union, true),
    (Rule::Intersection, true),
    (Rule::Difference, true),
    (Rule::Difference, false),
];

/// The regression: every tolerant fuse of the cube and the cone refuses on the
/// order of two corners round an ellipse, filed as an area, with a sweep past
/// the policy's budget read as an area and inside it read as a volume.
///
/// Filed as a volume, the policy merged them, and every fuse answered.
#[test]
fn a_tolerant_fuse_of_a_cube_on_a_cones_apex_refuses_a_sweep_past_an_area_budget() {
    let policy = Policy::tolerant(THE_CUBES_WORKING_SIZE);
    for (rule, cone_first) in EVERY_FUSE {
        let mut model =
            Model::new(Scale::millimetres(THE_CUBES_WORKING_SIZE).expect("a working size"));
        let cone = a_pointed_cone(&mut model, 2.0, 3.0);
        let cube = a_corner_up_cube_of_side_six_round_the_origin(&mut model);
        let (arguments, tools) = if cone_first {
            ([cone], [cube])
        } else {
            ([cube], [cone])
        };
        let mut log = ProvenanceLog::new();
        let fused = model.fuse(&arguments, &tools, rule, &policy, &mut log);
        let Err(Error::Undecidable { records }) = fused else {
            let merged: Vec<_> = log
                .records()
                .iter()
                .filter(|record| record.predicate() == ALONG_A_CROSSING)
                .map(|record| (record.measures(), record.enclosure()))
                .collect();
            panic!(
                "{rule:?}, cone first {cone_first}: {:?}, after merging {merged:?}",
                fused.map(|_: Fused| "an answer")
            );
        };
        let refused = records.records().last().expect("a refusal names why");
        let enclosure = refused.enclosure();
        assert_eq!(
            refused.predicate(),
            ALONG_A_CROSSING,
            "{rule:?} {cone_first}"
        );
        assert_eq!(refused.measures(), Measures::Area, "{rule:?} {cone_first}");
        assert!(
            !policy
                .wobble_is_acceptable(Measures::Area.wobble_of(enclosure, THE_CUBES_WORKING_SIZE)),
            "{rule:?} {cone_first}: {enclosure}"
        );
        assert!(
            policy.wobble_is_acceptable(
                Measures::Volume.wobble_of(enclosure, THE_CUBES_WORKING_SIZE)
            ),
            "{rule:?} {cone_first}: {enclosure}"
        );
    }
}

/// The gap found below a working size of `1` for "plane through a ball": a
/// cube stood on its corner, with that corner's own vertex sitting exactly on
/// a ball centred where the cube is — the ball's radius is the cube's own
/// half-diagonal, `3 * scale * sqrt(3)` for a side of `6 * scale` — the same
/// construction as the cone's apex case above, with the cone's single apex
/// point swapped for the one cube vertex a sphere has no apex to match.
/// Several of the cube's faces cut the ball in circles, each ordered by the
/// sweep `((one − from) × (other − from)) · n`, with `n` the face's normal:
/// `WhichWayItWinds::About` is reached here exactly as it is for a plane and
/// a pipe or a cone, `along_a_crossing.rs:401-402` computing the same normal
/// unconditionally for a plane and a sphere. Searching the cone's own
/// 5-scale x 7-size x 4-rule grid (the frustum's two scale/size lists narrowed
/// to keep the search small, since a ball has no seam direction to vary)
/// finds the same area-acceptable, volume-not-acceptable gap, at a shape
/// scale of `0.18` and a working size of `0.6` among others — the cone's own
/// pin values, which the half-diagonal construction shares by chance of the
/// grids overlapping, not by the shapes being the same.
///
/// **The merged answer is not wrong**, by the same similarity-transform
/// argument as the cone's apex case: scaled up by `1 / 0.18` and run at a
/// 10 mm working size, comfortably past the `2.73` mm where an area budget
/// covers the widest sweep, the merge gives a solid with the same face, wire
/// and shell counts.
#[test]
fn a_tolerant_fuse_of_a_cube_on_a_ball_through_its_corner_below_working_size_one_merges_the_same_shape_as_at_full_scale()
 {
    let gap_scale = 0.18;
    let gap_size = 0.6;
    let gap_policy = Policy::tolerant(gap_size);
    let mut gap_model = Model::new(Scale::millimetres(gap_size).expect("a working size"));
    let gap_ball = gap_model
        .add_ball(
            3.0 * gap_scale * 3.0f64.sqrt(),
            &Isometry::translation(Vector::new(0.0, 0.0, 0.0)).expect("a finite shift"),
        )
        .expect("a positive radius");
    let gap_cube = a_corner_up_cube_round_the_origin(&mut gap_model, 6.0 * gap_scale);
    let mut gap_log = ProvenanceLog::new();
    let gap_fused = gap_model
        .fuse(
            &[gap_ball],
            &[gap_cube],
            Rule::Union,
            &gap_policy,
            &mut gap_log,
        )
        .expect("the gap case answers Ok");

    gap_log
        .records()
        .iter()
        .find(|record| {
            record.predicate() == ALONG_A_CROSSING
                && record.measures() == Measures::Area
                && gap_policy
                    .wobble_is_acceptable(Measures::Area.wobble_of(record.enclosure(), gap_size))
                && !gap_policy
                    .wobble_is_acceptable(Measures::Volume.wobble_of(record.enclosure(), gap_size))
        })
        .expect("the area reading merges a sweep the volume reading would have refused");

    let gap_solid = gap_model
        .solid(gap_fused.solids()[0])
        .expect("the fuse's one solid");
    let gap_shape = (
        gap_solid.faces().count(),
        gap_solid.faces().flat_map(|face| face.wires()).count(),
        gap_solid.shells().count(),
    );

    let reference_size = 10.0;
    let mut reference_model =
        Model::new(Scale::millimetres(reference_size).expect("a working size"));
    let reference_ball = reference_model
        .add_ball(
            3.0 * 3.0f64.sqrt(),
            &Isometry::translation(Vector::new(0.0, 0.0, 0.0)).expect("a finite shift"),
        )
        .expect("a positive radius");
    let reference_cube = a_corner_up_cube_round_the_origin(&mut reference_model, 6.0);
    let reference_fused = reference_model
        .fuse(
            &[reference_ball],
            &[reference_cube],
            Rule::Union,
            &Policy::tolerant(reference_size),
            &mut ProvenanceLog::new(),
        )
        .expect("a working size of 10, well past 2.73, merges comfortably under the area budget");
    let reference_solid = reference_model
        .solid(reference_fused.solids()[0])
        .expect("the fuse's one solid");
    let reference_shape = (
        reference_solid.faces().count(),
        reference_solid
            .faces()
            .flat_map(|face| face.wires())
            .count(),
        reference_solid.shells().count(),
    );

    assert_eq!(gap_fused.solids().len(), 1, "one solid, the union of both");
    assert_eq!(
        reference_fused.solids().len(),
        1,
        "one solid, the union of both"
    );
    assert_eq!(
        gap_shape, reference_shape,
        "the gap case's topology should match the full-scale reference's"
    );
}

/// The gap found below a working size of `1` for "frustum cut near its
/// seam": the cone's apex case's shape, with `a_frustum_seamed_towards`'s
/// flat near cap (cut at `far / 300`) in place of the cone's sharp point,
/// shrunk to a shape scale of `0.26` and fused at a `0.75` mm working size,
/// merges the two corners round the ellipse under the area reading where the
/// volume reading would have refused.
///
/// **The merged answer is not wrong**, by the same similarity-transform
/// argument as the cone's apex case: scaled up by `1 / 0.26` and run at a
/// 10 mm working size, comfortably past the `2.73` mm where an area budget
/// covers the widest sweep, the merge gives a solid with the same face, wire
/// and shell counts.
#[test]
fn a_tolerant_fuse_of_a_cube_on_a_frustum_cut_near_its_seam_below_working_size_one_merges_the_same_shape_as_at_full_scale()
 {
    let gap_scale = 0.26;
    let gap_size = 0.75;
    let gap_policy = Policy::tolerant(gap_size);
    let mut gap_model = Model::new(Scale::millimetres(gap_size).expect("a working size"));
    let gap_far = 3.0 * gap_scale;
    let gap_frustum = a_frustum_seamed_towards(
        &mut gap_model,
        2.0,
        gap_far / 300.0,
        gap_far,
        [-1.0, 0.0],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a frustum");
    let gap_cube = a_corner_up_cube_round_the_origin(&mut gap_model, 6.0 * gap_scale);
    let mut gap_log = ProvenanceLog::new();
    let gap_fused = gap_model
        .fuse(
            &[gap_frustum],
            &[gap_cube],
            Rule::Union,
            &gap_policy,
            &mut gap_log,
        )
        .expect("the gap case answers Ok");

    gap_log
        .records()
        .iter()
        .find(|record| {
            record.predicate() == ALONG_A_CROSSING
                && record.measures() == Measures::Area
                && gap_policy
                    .wobble_is_acceptable(Measures::Area.wobble_of(record.enclosure(), gap_size))
                && !gap_policy
                    .wobble_is_acceptable(Measures::Volume.wobble_of(record.enclosure(), gap_size))
        })
        .expect("the area reading merges a sweep the volume reading would have refused");

    let gap_solid = gap_model
        .solid(gap_fused.solids()[0])
        .expect("the fuse's one solid");
    let gap_shape = (
        gap_solid.faces().count(),
        gap_solid.faces().flat_map(|face| face.wires()).count(),
        gap_solid.shells().count(),
    );

    let reference_size = 10.0;
    let mut reference_model =
        Model::new(Scale::millimetres(reference_size).expect("a working size"));
    let reference_far = 3.0;
    let reference_frustum = a_frustum_seamed_towards(
        &mut reference_model,
        2.0,
        reference_far / 300.0,
        reference_far,
        [-1.0, 0.0],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a frustum");
    let reference_cube = a_corner_up_cube_round_the_origin(&mut reference_model, 6.0);
    let reference_fused = reference_model
        .fuse(
            &[reference_frustum],
            &[reference_cube],
            Rule::Union,
            &Policy::tolerant(reference_size),
            &mut ProvenanceLog::new(),
        )
        .expect("a working size of 10, well past 2.73, merges comfortably under the area budget");
    let reference_solid = reference_model
        .solid(reference_fused.solids()[0])
        .expect("the fuse's one solid");
    let reference_shape = (
        reference_solid.faces().count(),
        reference_solid
            .faces()
            .flat_map(|face| face.wires())
            .count(),
        reference_solid.shells().count(),
    );

    assert_eq!(gap_fused.solids().len(), 1, "one solid, the union of both");
    assert_eq!(
        reference_fused.solids().len(),
        1,
        "one solid, the union of both"
    );
    assert_eq!(
        gap_shape, reference_shape,
        "the gap case's topology should match the full-scale reference's"
    );
}

/// Q2028's trace. Each of the six edges round a corner-up cube's middle,
/// carried on past its upper corner, runs exactly through the far rim of a
/// slope-two frustum whose far cap is at half the cube's side. So four
/// surfaces meet at one point there, and every tolerant fuse refuses on the
/// far cap, `z = 3 s`: at a shape scale of `0.1` and a working size of `0.25`,
/// and at its two power-of-two images, which round the same way.
#[test]
fn a_tolerant_fuse_of_a_cube_whose_edges_run_through_a_frustums_far_rim_cannot_tell_the_caps_cells_apart()
 {
    for (scale, size) in [(0.1, 0.25), (0.8, 2.0), (6.4, 16.0)] {
        for (rule, frustum_first) in EVERY_FUSE {
            let (model, fused) = a_frustum_cut_near_its_seam_fused_with_a_corner_up_cube(
                scale,
                size,
                1.0,
                rule,
                frustum_first,
            );
            let Err(Error::CannotTellTheCellsOfAFaceApart { face }) = fused else {
                panic!(
                    "{scale} {size} {rule:?} {frustum_first}: {:?}",
                    fused.map(|_: Fused| "an answer")
                );
            };
            let surface = model.face(face).expect("the refused face").surface();
            let Surface::Plane(cap) = surface else {
                panic!("{scale} {size} {rule:?} {frustum_first}: {surface:?}");
            };
            assert_eq!(
                (cap.normal().components(), cap.offset()),
                ([0.0, 0.0, 1.0], 3.0 * scale),
                "{scale} {size} {rule:?} {frustum_first}"
            );
        }
    }
}

/// The far cap lifted by a thousandth of its height is off the cube's edges'
/// lines, so no four surfaces meet at one point there, and every fuse of the
/// shape above at a scale of `0.1` and a working size of `0.25` answers with
/// a model `Model::validate` passes.
#[test]
fn the_same_fuses_answer_once_the_far_cap_is_lifted_off_the_cubes_edges() {
    let size = 0.25;
    for (rule, frustum_first) in EVERY_FUSE {
        let (model, fused) = a_frustum_cut_near_its_seam_fused_with_a_corner_up_cube(
            0.1,
            size,
            1.001,
            rule,
            frustum_first,
        );
        let fused = fused.unwrap_or_else(|error| panic!("{rule:?} {frustum_first}: {error:?}"));
        assert!(!fused.solids().is_empty(), "{rule:?} {frustum_first}");
        model
            .validate(&Policy::tolerant(size), &mut ProvenanceLog::new())
            .unwrap_or_else(|error| panic!("{rule:?} {frustum_first}: {error:?}"));
    }
}

/// A designer draws the same kind of meeting on purpose: a round bar notched
/// by a block whose inside corner edge stands on the bar's wall. At a radius
/// of `12.8` and a working size of `25`, every tolerant fuse refuses on the
/// bar's wall.
#[test]
fn a_tolerant_fuse_of_a_round_bar_and_a_block_cornered_on_its_wall_cannot_tell_the_walls_cells_apart()
 {
    for (rule, bar_first) in EVERY_FUSE {
        let (model, fused) =
            a_round_bar_fused_with_a_block_cornered_on_its_wall(12.8, 25.0, rule, bar_first);
        let Err(Error::CannotTellTheCellsOfAFaceApart { face }) = fused else {
            panic!(
                "{rule:?} {bar_first}: {:?}",
                fused.map(|_: Fused| "an answer")
            );
        };
        let surface = model.face(face).expect("the refused face").surface();
        assert!(
            matches!(surface, Surface::Cylinder(_)),
            "{rule:?} {bar_first}: {surface:?}"
        );
    }
}

#[test]
fn probe_q2082_bar() {
    let (_, fused) =
        a_round_bar_fused_with_a_block_cornered_on_its_wall(12.8, 25.0, Rule::Union, true);
    eprintln!("PROBE result {:?}", fused.map(|_| "answer"));
}

#[test]
fn probe_q2082_frustum() {
    let (_, fused) =
        a_frustum_cut_near_its_seam_fused_with_a_corner_up_cube(0.1, 0.25, 1.0, Rule::Union, true);
    eprintln!("PROBE result {:?}", fused.map(|_| "answer"));
}

/// The cube of side `6 scale` stood on its corner with its middle at the
/// origin, and a frustum of slope two about `z` with its near cap at
/// `far / 300` and its far cap at `far = 3 scale far_factor`, its seam cut
/// towards `-x`, fused by `rule` under a tolerant policy at `size`, the
/// frustum first when `frustum_first`. At a `far_factor` of `1` the far cap's
/// rim holds the point every edge round the cube's middle, carried on past
/// its upper corner, runs through.
fn a_frustum_cut_near_its_seam_fused_with_a_corner_up_cube(
    scale: f64,
    size: f64,
    far_factor: f64,
    rule: Rule,
    frustum_first: bool,
) -> (Model, rcad_topo::Result<Fused>) {
    let mut model = Model::new(Scale::millimetres(size).expect("a working size"));
    let far = 3.0 * scale * far_factor;
    let frustum = a_frustum_seamed_towards(
        &mut model,
        2.0,
        far / 300.0,
        far,
        [-1.0, 0.0],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a frustum");
    let cube = a_corner_up_cube_round_the_origin(&mut model, 6.0 * scale);
    let (arguments, tools) = if frustum_first {
        ([frustum], [cube])
    } else {
        ([cube], [frustum])
    };
    let fused = model.fuse(
        &arguments,
        &tools,
        rule,
        &Policy::tolerant(size),
        &mut ProvenanceLog::new(),
    );
    (model, fused)
}

/// A round bar of radius `radius` and height `2 radius` up from the origin,
/// and a block over `x` from `0.6 radius` to `2 radius`, `y` from
/// `-2 radius` to `0.8 radius` and `z` from `-radius` to `3 radius`, fused by
/// `rule` under a tolerant policy at `size`, the bar first when `bar_first`.
/// The block's inside corner edge, at `(0.6 radius, 0.8 radius)`, stands on
/// the bar's wall, as a designer cutting a notch to the wall draws it, so at
/// each of the bar's caps four surfaces meet at one point: the cap, the wall
/// and the two faces of the block on that edge.
fn a_round_bar_fused_with_a_block_cornered_on_its_wall(
    radius: f64,
    size: f64,
    rule: Rule,
    bar_first: bool,
) -> (Model, rcad_topo::Result<Fused>) {
    let mut model = Model::new(Scale::millimetres(size).expect("a working size"));
    let bar = model
        .add_round_bar(
            radius,
            2.0 * radius,
            &Isometry::translation(Vector::new(0.0, 0.0, 0.0)).expect("a shift"),
        )
        .expect("a bar");
    let notch = ProfileBuilder::new(vec![
        [0.6 * radius, -2.0 * radius],
        [2.0 * radius, -2.0 * radius],
        [2.0 * radius, 0.8 * radius],
        [0.6 * radius, 0.8 * radius],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    let block = model
        .add_extrusion(
            &notch,
            4.0 * radius,
            &Isometry::translation(Vector::new(0.0, 0.0, -radius)).expect("a shift"),
        )
        .expect("a block");
    let (arguments, tools) = if bar_first {
        ([bar], [block])
    } else {
        ([block], [bar])
    };
    let fused = model.fuse(
        &arguments,
        &tools,
        rule,
        &Policy::tolerant(size),
        &mut ProvenanceLog::new(),
    );
    (model, fused)
}

/// A cube of side six with its diagonal brought upright and its middle at the
/// origin, built as `a_plane_cut_along_a_cones_other_nappe.rs` builds it.
fn a_corner_up_cube_of_side_six_round_the_origin(model: &mut Model) -> SolidKey {
    a_corner_up_cube_round_the_origin(model, 6.0)
}

/// [`a_corner_up_cube_of_side_six_round_the_origin`], scaled to `side`.
fn a_corner_up_cube_round_the_origin(model: &mut Model, side: f64) -> SolidKey {
    let diagonal = Axis::new(Point::origin(), Vector::new(1.0, -1.0, 0.0)).expect("a direction");
    let stood = Isometry::rotation(&diagonal, (1.0f64 / 3.0).sqrt().acos()).expect("a finite turn");
    let placed = Isometry::translation(Vector::new(0.0, 0.0, -side * 3.0f64.sqrt() / 2.0))
        .expect("a finite move")
        .compose(&stood);
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    model
        .add_extrusion(&square, side, &placed)
        .expect("a square extrudes")
}

/// The gap Q1273 left open below a working size of `1`: the cube-on-a-cone
/// shape, shrunk to a shape scale of `0.18` and fused at a `0.6` mm working
/// size, merges the two corners round the ellipse under the area reading
/// where the volume reading would have refused — the area budget covers more
/// than the volume one below a working size of `1`, by `1/s`.
///
/// **The merged answer is not wrong.** A tolerant fuse of the same pair,
/// scaled up by `1/0.18` and run at a 10 mm working size — comfortably past
/// the `2.73` mm where §318.5 says an area budget covers the widest sweep, so
/// its merge is trusted — gives a solid with the same face, wire and shell
/// counts. Fusing is a similarity transform: scaling the shape and the
/// working size together cannot change which of two points comes first round
/// the ellipse, so the two runs answer the same combinatorial question and
/// agree.
#[test]
fn a_tolerant_fuse_of_a_cube_on_a_cone_below_working_size_one_merges_the_same_shape_as_at_full_scale()
 {
    let gap_scale = 0.18;
    let gap_size = 0.6;
    let gap_policy = Policy::tolerant(gap_size);
    let mut gap_model = Model::new(Scale::millimetres(gap_size).expect("a working size"));
    let gap_cone = a_pointed_cone(&mut gap_model, 2.0, 3.0 * gap_scale);
    let gap_cube = a_corner_up_cube_round_the_origin(&mut gap_model, 6.0 * gap_scale);
    let mut gap_log = ProvenanceLog::new();
    let gap_fused = gap_model
        .fuse(
            &[gap_cone],
            &[gap_cube],
            Rule::Union,
            &gap_policy,
            &mut gap_log,
        )
        .expect("the gap case answers Ok");

    gap_log
        .records()
        .iter()
        .find(|record| {
            record.predicate() == ALONG_A_CROSSING
                && record.measures() == Measures::Area
                && gap_policy
                    .wobble_is_acceptable(Measures::Area.wobble_of(record.enclosure(), gap_size))
                && !gap_policy
                    .wobble_is_acceptable(Measures::Volume.wobble_of(record.enclosure(), gap_size))
        })
        .expect("the area reading merges a sweep the volume reading would have refused");

    let gap_solid = gap_model
        .solid(gap_fused.solids()[0])
        .expect("the fuse's one solid");
    let gap_shape = (
        gap_solid.faces().count(),
        gap_solid.faces().flat_map(|face| face.wires()).count(),
        gap_solid.shells().count(),
    );

    let reference_size = 10.0;
    let mut reference_model =
        Model::new(Scale::millimetres(reference_size).expect("a working size"));
    let reference_cone = a_pointed_cone(&mut reference_model, 2.0, 3.0);
    let reference_cube = a_corner_up_cube_round_the_origin(&mut reference_model, 6.0);
    let reference_fused = reference_model
        .fuse(
            &[reference_cone],
            &[reference_cube],
            Rule::Union,
            &Policy::tolerant(reference_size),
            &mut ProvenanceLog::new(),
        )
        .expect("a working size of 10, well past 2.73, merges comfortably under the area budget");
    let reference_solid = reference_model
        .solid(reference_fused.solids()[0])
        .expect("the fuse's one solid");
    let reference_shape = (
        reference_solid.faces().count(),
        reference_solid
            .faces()
            .flat_map(|face| face.wires())
            .count(),
        reference_solid.shells().count(),
    );

    assert_eq!(gap_fused.solids().len(), 1, "one solid, the union of both");
    assert_eq!(
        reference_fused.solids().len(),
        1,
        "one solid, the union of both"
    );
    assert_eq!(
        gap_shape, reference_shape,
        "the gap case's topology should match the full-scale reference's"
    );
}
