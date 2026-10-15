//! Two half bars stacked end to end on one pipe, each a bar less a block over
//! `x < 0`, and what their fuses should answer.
//!
//! # The half bars fuse directly
//!
//! Every fuse of the two half bars used to refuse, all six of them, with one
//! spelling of the wall or two: `AFaceTrimmedByACurveThisCannotFollow`, first
//! in `fuse/arrange.rs` and then in `SolidView::face_holds_the_point`
//! (`docs/decisions.md` §325). A half bar's wall and its two half-disc caps have
//! a half circle for a side, and reading which half of the circle that is needs
//! the circle's centre, which `where_a_closed_crossing_is_centred` in
//! `holds.rs` gave for a ball and a cone and not for a pipe. Behind that,
//! `holds.rs` read no straight side on a curved face and followed no face on a
//! pipe, nor a pipe cutting a flat face.
//!
//! Those refusals were **gaps** and not limits (§16.10): the answers exist, and
//! the kernel reached them when the block is cut off *after* the whole bars are
//! fused, which is the same set by plain set algebra.
//! [`cutting_the_block_off_after_fusing_the_whole_bars_answers_what_the_half_bars_should`]
//! is that route, and
//! [`two_half_bars_fuse_directly_to_what_cutting_the_block_off_afterwards_gives`]
//! now holds the direct fuses to the same answers.
//!
//! # The same route with the wall spelled twice
//!
//! §325.4's second refusal is answered: the whole bars' union left a wall
//! spelled twice as two faces, and cutting the block off it refused
//! `AFaceMeetsItsNeighbourOnItsOwnSurface` on the circle between them. The
//! joiner now joins pieces of one pipe however it is written, so the union's
//! wall is one face and every later fuse of it answers.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface,
    Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, DraftStep, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

const RADIUS: f64 = 2.0;

/// Each bar is two tall, the lower one on `z = 0` and the upper one on it.
const HEIGHT: f64 = 2.0;

/// `(A − B) ∪ (U − B)`, `(A − B) ∩ (U − B)` and `(A − B) − (U − B)` are
/// `(A ∪ U) − B`, `(A ∩ U) − B` and `(A − U) − B`, so fusing the whole bars
/// first and cutting the block off afterwards is the answer the fuse of the
/// two half bars should give.
///
/// - The union is a half bar four tall: `π·2²·4 / 2 = 8π`.
/// - The difference, either way round, is one half bar two tall: `4π`.
/// - The intersection is nothing. The bars only touch, on the disc at `z = 2`.
#[test]
fn cutting_the_block_off_after_fusing_the_whole_bars_answers_what_the_half_bars_should() {
    let policy = Policy::certified();
    for (rule, by_hand) in [
        (Rule::Union, Some(8.0 * PI)),
        (Rule::Intersection, None),
        (Rule::Difference, Some(4.0 * PI)),
    ] {
        for lower_first in [true, false] {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
            let lower = a_bar_on_the_one_pipe(&mut model, 0.0);
            let upper = a_bar_on_the_one_pipe(&mut model, HEIGHT);
            let [argument, tool] = if lower_first {
                [lower, upper]
            } else {
                [upper, lower]
            };
            let whole = model
                .fuse(
                    &[argument],
                    &[tool],
                    rule,
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|refused| panic!("{rule:?} of the whole bars: {refused:?}"));

            let mut halves = Vec::new();
            for solid in whole.solids().to_vec() {
                let block = a_block_over_negative_x(&mut model);
                let cut = model
                    .fuse(
                        &[solid],
                        &[block],
                        Rule::Difference,
                        &policy,
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|refused| panic!("{rule:?}, then less the block: {refused:?}"));
                halves.extend_from_slice(cut.solids());
            }
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|broken| panic!("{rule:?} came back broken: {broken:?}"));

            let Some(by_hand) = by_hand else {
                assert!(halves.is_empty(), "{rule:?}: {halves:?}");
                continue;
            };
            let [half] = halves.as_slice() else {
                panic!("{rule:?}, lower first {lower_first}: {halves:?}");
            };
            let measured = model
                .solid(*half)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.01).expect("a positive length"),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .expect("a half bar measures");
            assert!(
                (measured.volume() - by_hand).abs() <= measured.volume_error(),
                "{rule:?}, lower first {lower_first}: {} ± {} against {by_hand}",
                measured.volume(),
                measured.volume_error()
            );
        }
    }
}

/// **The two half bars fuse directly, by every rule and both ways round, to what
/// cutting the block off the whole bars' fuse gives**: a union of one half bar
/// four tall, `8π`; a difference of one half bar two tall, `4π`; and an
/// intersection of nothing, since the two only touch on the half disc at
/// `z = 2`.
///
/// Each answer that is a solid is a half bar, so it has four faces: two half
/// discs, a half wall and the flat face along `x = 0`.
///
/// Asked with the wall spelled once, both bars naming the pipe's axis at the
/// origin, and spelled twice, the upper bar naming it at `z = 2`. All twelve
/// refused `AFaceTrimmedByACurveThisCannotFollow` until a pipe's crossing with a
/// plane had a centre and `holds.rs` followed a half bar's faces (§325.5,
/// row 1).
#[test]
fn two_half_bars_fuse_directly_to_what_cutting_the_block_off_afterwards_gives() {
    let policy = Policy::certified();
    for upper_axis_at in [0.0, HEIGHT] {
        for (rule, by_hand) in [
            (Rule::Union, Some(8.0 * PI)),
            (Rule::Intersection, None),
            (Rule::Difference, Some(4.0 * PI)),
        ] {
            for lower_first in [true, false] {
                let asked = format!(
                    "{rule:?} with the upper axis at z = {upper_axis_at}, lower first {lower_first}"
                );
                let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
                let lower = a_half_bar_whose_wall_names_its_axis_at(&mut model, 0.0, 0.0);
                let upper =
                    a_half_bar_whose_wall_names_its_axis_at(&mut model, HEIGHT, upper_axis_at);
                let [argument, tool] = if lower_first {
                    [lower, upper]
                } else {
                    [upper, lower]
                };
                let fused = model
                    .fuse(
                        &[argument],
                        &[tool],
                        rule,
                        &policy,
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|refused| panic!("{asked}: {refused:?}"));
                model
                    .validate(&policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|broken| panic!("{asked} came back broken: {broken:?}"));

                let Some(by_hand) = by_hand else {
                    assert!(fused.solids().is_empty(), "{asked}: {:?}", fused.solids());
                    continue;
                };
                let [half] = fused.solids() else {
                    panic!("{asked}: {:?}", fused.solids());
                };
                let half = model.solid(*half).expect("this model made the key");
                assert_eq!(half.faces().count(), 4, "{asked}: the faces of a half bar");
                let measured = half
                    .measure(
                        Deflection::of(0.01).expect("a positive length"),
                        &policy,
                        &mut ProvenanceLog::new(),
                    )
                    .expect("a half bar measures");
                assert!(
                    (measured.volume() - by_hand).abs() <= measured.volume_error(),
                    "{asked}: {} ± {} against {by_hand}",
                    measured.volume(),
                    measured.volume_error()
                );
            }
        }
    }
}

/// The whole bars' union with the upper wall naming its axis at `z = 2`, less
/// the block, is the half bar four tall, `8π`, either way round.
///
/// It used to refuse `AFaceMeetsItsNeighbourOnItsOwnSurface` (§325.4): the
/// union kept its wall as two faces, one per spelling, and the cut asked
/// `intersect` about the circle between them and heard `SameSurface`.
#[test]
fn a_wall_spelled_twice_is_cut_to_the_same_half_bar_as_a_wall_spelled_once() {
    let policy = Policy::certified();
    for lower_first in [true, false] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let lower = a_bar_whose_wall_names_its_axis_at(&mut model, 0.0, 0.0);
        let upper = a_bar_whose_wall_names_its_axis_at(&mut model, HEIGHT, HEIGHT);
        let [argument, tool] = if lower_first {
            [lower, upper]
        } else {
            [upper, lower]
        };
        let whole = model
            .fuse(
                &[argument],
                &[tool],
                Rule::Union,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|refused| {
                panic!("the whole bars, lower first {lower_first}: {refused:?}")
            });
        let block = a_block_over_negative_x(&mut model);
        let cut = model
            .fuse(
                whole.solids(),
                &[block],
                Rule::Difference,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|refused| {
                panic!("less the block, lower first {lower_first}: {refused:?}")
            });
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|broken| panic!("lower first {lower_first}: {broken:?}"));
        let [half] = cut.solids() else {
            panic!("lower first {lower_first}: {:?}", cut.solids());
        };
        let measured = model
            .solid(*half)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("a half bar measures");
        assert!(
            (measured.volume() - 8.0 * PI).abs() <= measured.volume_error(),
            "lower first {lower_first}: {} ± {} against 8π",
            measured.volume(),
            measured.volume_error()
        );
    }
}

/// The upper bar's wall spelled the other way — axis reversed and the origin
/// slid off `z = 2` to `z = 123` — still joins the lower bar's wall as one
/// pipe: the union is the half bar four tall, `8π`, either way round.
///
/// `a_ring_goes_right_round` finds a ring's side partner with `one_surface`
/// (§354), which folds a pipe's axis reversal into one name, and the origin
/// slide too since this axis runs along a coordinate, where the fold is exact
/// (§51.9). Found by the stored fields instead, with no fold, the two
/// spellings would not match and the fuse would refuse — as the hand-built
/// case did before §354
/// (`a_side_naming_its_pipe_by_the_other_spelling_is_still_counted_round`).
#[test]
fn a_wall_spelled_the_other_way_round_still_fuses_to_one_half_bar() {
    let policy = Policy::certified();
    for lower_first in [true, false] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let lower = a_half_bar_whose_wall_names_its_axis_at(&mut model, 0.0, 0.0);
        let upper =
            a_half_bar_whose_wall_names_its_axis_the_other_way_round(&mut model, HEIGHT, 123.0);
        let [argument, tool] = if lower_first {
            [lower, upper]
        } else {
            [upper, lower]
        };
        let fused = model
            .fuse(
                &[argument],
                &[tool],
                Rule::Union,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|refused| panic!("lower first {lower_first}: {refused:?}"));
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|broken| {
                panic!("lower first {lower_first} came back broken: {broken:?}")
            });
        let [half] = fused.solids() else {
            panic!("lower first {lower_first}: {:?}", fused.solids());
        };
        let half = model.solid(*half).expect("this model made the key");
        assert_eq!(
            half.faces().count(),
            4,
            "lower first {lower_first}: the faces of a half bar"
        );
        let measured = half
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("a half bar measures");
        assert!(
            (measured.volume() - 8.0 * PI).abs() <= measured.volume_error(),
            "lower first {lower_first}: {} ± {} against 8π",
            measured.volume(),
            measured.volume_error()
        );
    }
}

/// Two bars from `Model::add_round_bar`, the second placed on the first,
/// united and then united with a cube far away that touches neither: two
/// solids, `16π + 8` between them, either way round.
///
/// Each builder call writes its wall's axis where its own placement puts it,
/// so the two walls are one pipe spelled twice. The union used to keep them
/// as two faces, and this second fuse refused on the lower one
/// (`AFaceMeetsItsNeighbourOnItsOwnSurface`) though the cube meets nothing.
#[test]
fn two_round_bars_stacked_and_united_fuse_with_a_cube_that_touches_nothing() {
    let policy = Policy::certified();
    let placed_at = |z: f64| Isometry::translation(Vector::new(0.0, 0.0, z)).expect("a shift");
    for lower_first in [true, false] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let lower = model
            .add_round_bar(RADIUS, HEIGHT, &placed_at(0.0))
            .expect("a bar");
        let upper = model
            .add_round_bar(RADIUS, HEIGHT, &placed_at(HEIGHT))
            .expect("a bar");
        let [argument, tool] = if lower_first {
            [lower, upper]
        } else {
            [upper, lower]
        };
        let bar = model
            .fuse(
                &[argument],
                &[tool],
                Rule::Union,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|refused| {
                panic!("the two bars, lower first {lower_first}: {refused:?}")
            });
        let cube = a_cube_far_from_the_bars(&mut model);
        let both = model
            .fuse(
                bar.solids(),
                &[cube],
                Rule::Union,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|refused| {
                panic!("with the far cube, lower first {lower_first}: {refused:?}")
            });
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|broken| panic!("lower first {lower_first}: {broken:?}"));
        assert_eq!(both.solids().len(), 2, "lower first {lower_first}");
        let (mut volume, mut bound) = (0.0, 0.0);
        for &solid in both.solids() {
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.01).expect("a positive length"),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .expect("a bar and a cube measure");
            volume += measured.volume();
            bound += measured.volume_error();
        }
        let by_hand = 16.0 * PI + 8.0;
        assert!(
            (volume - by_hand).abs() <= bound,
            "lower first {lower_first}: {volume} ± {bound} against 16π + 8"
        );
    }
}

/// A cube two on a side at `(20, 20, 20)`, nowhere near either bar.
fn a_cube_far_from_the_bars(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![[20.0, 20.0], [22.0, 20.0], [22.0, 22.0], [20.0, 22.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    model
        .add_extrusion(
            &square,
            2.0,
            &Isometry::translation(Vector::new(0.0, 0.0, 20.0)).expect("a finite shift"),
        )
        .expect("a positive height")
}

/// **One half bar places `(1, 0.5, 0.5)` inside**, which is what the whole bar
/// and the block cut off it say between them: the bar holds the point and the
/// block does not.
///
/// The half bar used to answer `CannotFollowAFacesTrimming` on its wall, whose
/// sides are two half circles and two straight lines along `x = 0` (§325.5).
#[test]
fn a_half_bar_places_a_point_inside_as_the_bar_and_the_block_say() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let half = a_half_bar_whose_wall_names_its_axis_at(&mut model, 0.0, 0.0);
    let bar = a_bar_on_the_one_pipe(&mut model, 0.0);
    let block = a_block_over_negative_x(&mut model);
    let point = Point::new(1.0, 0.5, 0.5);

    let [in_the_half, in_the_bar, in_the_block] = [half, bar, block].map(|solid| {
        model
            .solid(solid)
            .expect("this model made the key")
            .classify_point(point, &policy, &mut ProvenanceLog::new())
    });
    assert_eq!(in_the_bar, Containment::Inside);
    assert_eq!(in_the_block, Containment::Outside);
    assert_eq!(in_the_half, Containment::Inside);
}

/// A block over `x < 0` reaching well past both bars, above, below and to the
/// sides, so the only face of it the bars meet is `x = 0`.
fn a_block_over_negative_x(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![
        [-10.0, -10.0],
        [0.0, -10.0],
        [0.0, 10.0],
        [-10.0, 10.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    model
        .add_extrusion(
            &square,
            10.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -3.0)).expect("a finite shift"),
        )
        .expect("a positive height")
}

/// A round bar from `bottom` up [`HEIGHT`] less the block over `x < 0`, whose
/// wall names its axis at `z = axis_origin`.
fn a_half_bar_whose_wall_names_its_axis_at(
    model: &mut Model,
    bottom: f64,
    axis_origin: f64,
) -> SolidKey {
    let bar = a_bar_whose_wall_names_its_axis_at(model, bottom, axis_origin);
    let block = a_block_over_negative_x(model);
    let cut = model
        .fuse(
            &[bar],
            &[block],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar less the block");
    let [half] = cut.solids() else {
        panic!("a bar less the block is one solid: {:?}", cut.solids());
    };
    *half
}

/// A round bar from `bottom` up [`HEIGHT`] less the block over `x < 0`, whose
/// wall names the same pipe the other way round from
/// [`a_half_bar_whose_wall_names_its_axis_at`]: axis direction reversed,
/// origin at `z = axis_origin`.
fn a_half_bar_whose_wall_names_its_axis_the_other_way_round(
    model: &mut Model,
    bottom: f64,
    axis_origin: f64,
) -> SolidKey {
    let bar = a_bar_whose_wall_names_its_axis_the_other_way_round(model, bottom, axis_origin);
    let block = a_block_over_negative_x(model);
    let cut = model
        .fuse(
            &[bar],
            &[block],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar less the block");
    let [half] = cut.solids() else {
        panic!("a bar less the block is one solid: {:?}", cut.solids());
    };
    *half
}

/// A round bar from `bottom` up [`HEIGHT`], whose wall names its axis at the
/// origin whichever bar it is.
///
/// Built face by face rather than with `Model::add_round_bar`, which places the
/// whole solid and so moves the wall's axis origin with it: the fuse would then
/// read the two walls as two spellings of one pipe (§231, §247).
fn a_bar_on_the_one_pipe(model: &mut Model, bottom: f64) -> SolidKey {
    a_bar_whose_wall_names_its_axis_at(model, bottom, 0.0)
}

/// A round bar from `bottom` up [`HEIGHT`], whose wall names its axis at
/// `z = axis_origin`.
fn a_bar_whose_wall_names_its_axis_at(
    model: &mut Model,
    bottom: f64,
    axis_origin: f64,
) -> SolidKey {
    let top = bottom + HEIGHT;
    let mut builder = SolidBuilder::new();
    let low = builder
        .add_vertex(Point::new(RADIUS, 0.0, bottom))
        .expect("a finite corner");
    let high = builder
        .add_vertex(Point::new(RADIUS, 0.0, top))
        .expect("a finite corner");
    let seam = builder
        .add_edge(low, high, EdgeRun::Straight)
        .expect("the seam up the wall");
    let low_circle = builder
        .add_edge(low, low, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the bottom cap circle");
    let high_circle = builder
        .add_edge(high, high, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the top cap circle");

    let cap = |builder: &mut SolidBuilder, z: f64, outward: f64, edge, turn| {
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, outward))
                .expect("the z axis is a unit normal");
        let coedge = builder.add_coedge(edge, turn).expect("one use of the rim");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("a cap is bounded by one whole circle");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a flat cap")
    };
    let bottom_cap = cap(
        &mut builder,
        bottom,
        -1.0,
        low_circle,
        Orientation::Reversed,
    );
    let top_cap = cap(&mut builder, top, 1.0, high_circle, Orientation::Forward);

    let round_the_wall = [
        (low_circle, Orientation::Forward),
        (seam, Orientation::Forward),
        (high_circle, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ]
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("a step round the wall"),
        )
    });
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("the wall's one wire");
    let pipe = Cylinder::new(
        Axis::new(
            Point::new(0.0, 0.0, axis_origin),
            Vector::new(0.0, 0.0, 1.0),
        )
        .expect("a direction with a length"),
        RADIUS,
    )
    .expect("a positive radius");
    let wall = builder
        .add_face(Surface::Cylinder(pipe), &[wire])
        .expect("the curved wall");

    builder
        .add_shell(&[
            (bottom_cap, Orientation::Forward),
            (top_cap, Orientation::Forward),
            (wall, Orientation::Forward),
        ])
        .expect("a closed skin");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar described face by face")
}

/// A round bar from `bottom` up [`HEIGHT`], whose wall names the same pipe the
/// other way round from [`a_bar_whose_wall_names_its_axis_at`]: axis direction
/// reversed, origin at `z = axis_origin`.
///
/// A closed edge on a curved face is stored running along `ŵ × ∇C` for the
/// wall's **stored** direction `ŵ`, so a wall written with the axis pointing
/// down walks both its cap circles the other way round from one written
/// pointing up — same bar, same skin, nothing but the spelling changed
/// (`Model::add_round_bar`'s `round_a_circle`).
fn a_bar_whose_wall_names_its_axis_the_other_way_round(
    model: &mut Model,
    bottom: f64,
    axis_origin: f64,
) -> SolidKey {
    let top = bottom + HEIGHT;
    let mut builder = SolidBuilder::new();
    let low = builder
        .add_vertex(Point::new(RADIUS, 0.0, bottom))
        .expect("a finite corner");
    let high = builder
        .add_vertex(Point::new(RADIUS, 0.0, top))
        .expect("a finite corner");
    let seam = builder
        .add_edge(low, high, EdgeRun::Straight)
        .expect("the seam up the wall");
    let low_circle = builder
        .add_edge(low, low, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the bottom cap circle");
    let high_circle = builder
        .add_edge(high, high, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the top cap circle");

    let cap = |builder: &mut SolidBuilder, z: f64, outward: f64, edge, turn| {
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, outward))
                .expect("the z axis is a unit normal");
        let coedge = builder.add_coedge(edge, turn).expect("one use of the rim");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("a cap is bounded by one whole circle");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a flat cap")
    };
    let bottom_cap = cap(&mut builder, bottom, -1.0, low_circle, Orientation::Forward);
    let top_cap = cap(&mut builder, top, 1.0, high_circle, Orientation::Reversed);

    let round_the_wall = [
        (low_circle, Orientation::Reversed),
        (seam, Orientation::Forward),
        (high_circle, Orientation::Forward),
        (seam, Orientation::Reversed),
    ]
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("a step round the wall"),
        )
    });
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("the wall's one wire");
    let pipe = Cylinder::new(
        Axis::new(
            Point::new(0.0, 0.0, axis_origin),
            Vector::new(0.0, 0.0, -1.0),
        )
        .expect("a direction with a length"),
        RADIUS,
    )
    .expect("a positive radius");
    let wall = builder
        .add_face(Surface::Cylinder(pipe), &[wire])
        .expect("the curved wall");

    builder
        .add_shell(&[
            (bottom_cap, Orientation::Forward),
            (top_cap, Orientation::Forward),
            (wall, Orientation::Forward),
        ])
        .expect("a closed skin");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar described face by face")
}
