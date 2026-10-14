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
//! **Only this shape was searched.** "Bar or frustum cut along the seam" and
//! "plane through a ball" — the other two families a plane's conic sweep
//! shows up in (`along_a_crossing.rs`'s `WhichWayItWinds::About` arm) — are
//! not yet built below working size `1`; that is Q2002's own follow-up.

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

/// Each rule, and for the difference which solid goes first: whether the cone
/// is the argument.
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

// Q2016 WIP: the four `probe_*` tests below are exploratory and not meant to
// be committed as final — they are left here, uncommitted-normally but kept
// for this relay, so the next lane does not have to re-derive them. See the
// handoff for what each one found. None of them is a regression test; they
// must be removed (or turned into one, if a gap is found) before this row is
// reported done.
#[test]
fn probe_family_b_tilt_grid() {
    for tilt_exp in [-20, -16, -12, -10, -8, -6, -4, -3, -2] {
        let tilt = f64::powi(10.0, tilt_exp);
        for size in [0.01, 0.1, 0.3, 0.6, 1.0, 5.0, 25.0] {
            let mut model = Model::new(Scale::millimetres(size).expect("ws"));
            let triangle = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 3.0], [-3.0, 3.0]])
                .build(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("a triangle");
            let tilted = Isometry::rotation(
                &Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("an axis"),
                tilt,
            )
            .expect("a finite turn")
            .compose(&Isometry::translation(Vector::new(0.0, 0.0, LIFT)).expect("a finite shift"));
            let prism = model
                .add_extrusion(&triangle, 32.0, &tilted)
                .expect("a positive height");
            let ball = a_ball_grazing_the_z_axis(&mut model);
            for (rule, cone_first) in EVERY_FUSE {
                let (arguments, tools) = if cone_first {
                    ([ball], [prism])
                } else {
                    ([prism], [ball])
                };
                let mut log = ProvenanceLog::new();
                let fused = model.fuse(
                    &arguments,
                    &tools,
                    rule,
                    &Policy::tolerant(size),
                    &mut log,
                );
                let records: Vec<_> = match &fused {
                    Ok(_) => log.records().to_vec(),
                    Err(Error::Undecidable { records }) => records.records().to_vec(),
                    Err(other) => {
                        println!("tilt=1e{tilt_exp} size={size} rule={rule:?} cone_first={cone_first}: {other:?}");
                        continue;
                    }
                };
                if !records.is_empty() {
                    println!(
                        "tilt=1e{tilt_exp} size={size} rule={rule:?} cone_first={cone_first} fused_ok={} preds={:?}",
                        fused.is_ok(),
                        records
                            .iter()
                            .map(|r| (r.predicate(), r.measures()))
                            .collect::<Vec<_>>()
                    );
                }
            }
        }
    }
}

#[test]
fn probe_family_b_ball_cube() {
    for scale in [1.0, 0.5, 0.3, 0.18, 0.1] {
      for size in [0.1, 0.3, 0.6, 1.0, 2.0, 5.0, 10.0] {
        let mut model = Model::new(Scale::millimetres(size).expect("ws"));
        let ball = model
            .add_ball(
                scale,
                &Isometry::translation(Vector::new(0.0, 0.0, scale)).expect("a finite shift"),
            )
            .expect("a positive radius");
        let cube = a_corner_up_cube_round_the_origin(&mut model, 6.0 * scale);
        for (rule, cone_first) in EVERY_FUSE {
            let (arguments, tools) = if cone_first {
                ([ball], [cube])
            } else {
                ([cube], [ball])
            };
            let mut log = ProvenanceLog::new();
            let fused = model.fuse(
                &arguments,
                &tools,
                rule,
                &Policy::tolerant(size),
                &mut log,
            );
            let records: Vec<_> = match &fused {
                Ok(_) => log.records().to_vec(),
                Err(Error::Undecidable { records }) => records.records().to_vec(),
                Err(other) => {
                    println!("scale={scale} rule={rule:?} cone_first={cone_first}: {other:?}");
                    continue;
                }
            };
            if !records.is_empty() {
                println!(
                    "scale={scale} size={size} rule={rule:?} cone_first={cone_first} fused_ok={} n={} preds={:?}",
                    fused.is_ok(),
                    records.len(),
                    records.iter().map(|r| r.predicate()).collect::<Vec<_>>()
                );
            }
        }
      }
    }
}

#[test]
fn probe_family_a_angles() {
    for seam_name in ["px", "py", "nx", "ny"] {
        let seam = match seam_name {
            "px" => [1.0, 0.0],
            "py" => [0.0, 1.0],
            "nx" => [-1.0, 0.0],
            _ => [0.0, -1.0],
        };
        let mut model = Model::new(Scale::millimetres(2.0).expect("ws"));
        let frustum = a_frustum_seamed_towards(
            &mut model,
            2.0,
            0.01,
            3.0,
            seam,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a frustum");
        let cube = a_corner_up_cube_of_side_six_round_the_origin(&mut model);
        for (rule, cone_first) in EVERY_FUSE {
            let (arguments, tools) = if cone_first {
                ([frustum], [cube])
            } else {
                ([cube], [frustum])
            };
            let mut log = ProvenanceLog::new();
            let fused = model.fuse(
                &arguments,
                &tools,
                rule,
                &Policy::tolerant(2.0),
                &mut log,
            );
            let records: Vec<_> = match &fused {
                Ok(_) => log.records().to_vec(),
                Err(Error::Undecidable { records }) => records.records().to_vec(),
                Err(other) => {
                    println!("seam={seam_name} rule={rule:?} cone_first={cone_first}: {other:?}");
                    continue;
                }
            };
            for record in &records {
                if record.predicate() != ALONG_A_CROSSING {
                    continue;
                }
                for point in record.inputs() {
                    let [x, y, _z] = point.enclosure();
                    let mx = (x.lo() + x.hi()) / 2.0;
                    let my = (y.lo() + y.hi()) / 2.0;
                    let angle = my.atan2(mx).to_degrees();
                    println!(
                        "seam={seam_name} rule={rule:?} cone_first={cone_first} measure={:?} angle={angle:.2} x=[{},{}] y=[{},{}]",
                        record.measures(),
                        x.lo(),
                        x.hi(),
                        y.lo(),
                        y.hi()
                    );
                }
            }
        }
    }
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
