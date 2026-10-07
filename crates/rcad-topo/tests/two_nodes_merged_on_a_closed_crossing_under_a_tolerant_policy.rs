//! Two places a ball is cut at, merged round a closed crossing by a tolerant
//! policy that read the width of their boxes as a volume.
//!
//! # The shape
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

use rcad_geom::{Isometry, MeetOfThree, Scale, Surface, Vector, meet_of_three_surfaces};
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
