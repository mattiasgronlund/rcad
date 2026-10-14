//! A pin standing beside a ring it never touches, in one group with the ring
//! and a bar in the ring's hole, fused with a round plate every way (row
//! Q1975, found by Q1137).
//!
//! **The ring** is the tube of radius one round the ring of radius four about
//! `z`, right round, written as its two halves either side of `z = 0`. By
//! Pappus it holds `2π² · 4 · 1 = 8π²`.
//!
//! **The plate** is a round bar of radius `11/2` about `z`, from `z = −2` to
//! `1/2`, so it holds `605π/8`. Its top cuts the ring in two circles, and
//! below its top the ring holds the tube's cross-section below `z = 1/2`,
//! `2π/3 + √3/4`, swept round the ring's circle of length `8π`:
//! `16π²/3 + 2π√3`.
//!
//! **The bar** has radius 2 about `z`, from `z = 1/4` to `3/4`, in the ring's
//! hole and clear of it. It holds `2π`, and half of it is in the plate.
//!
//! **The pin** has radius `0.15` about the line along `z` through `(5.2, 0)`,
//! from `z = 1/4` to `3/4`, so it holds `0.01125π`, half of it in the plate.
//! It comes no nearer the axis than `5.05`, and the tube reaches out to `5`.
//!
//! Every fuse of the plate with the group was refused
//! `ACrossingBetweenATorusAndAnotherCurvedSurface`. The ring, the bar and the
//! pin all reach the plate, so all four solids are cut against each other,
//! every face against every face (`gather` in `src/fuse/cuts.rs`), and
//! `intersect` refused a torus against any pipe that is not on its axis
//! without asking whether the two meet at all. A pipe along the ring's axis
//! but off it meets the tube nowhere when every distance from the axis it
//! reaches is past the tube's, short of it, or round the whole of it, and
//! that is now answered. A plane holding the ring's axis, the ring and that
//! pipe were refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` for the same
//! reason, and now meet nowhere.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Error, Isometry, MeetOfThree, Orientation, Plane, Point,
    Scale, Surface, SurfaceMeet, Torus, Vector, intersect, meet_of_three_surfaces,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, Rule, SolidBuilder, SolidKey};

/// How fine a mesh every volume here is read off.
const DEFLECTION: f64 = 0.01;

const THE_RING: f64 = 8.0 * PI * PI;

const THE_PLATE: f64 = 605.0 * PI / 8.0;

const THE_BAR: f64 = 2.0 * PI;

const THE_PIN: f64 = PI * 0.15 * 0.15 * 0.5;

/// The plate with the ring, the bar and the pin all standing in it: one solid
/// each way round, the plate less the group or the group with the plate.
/// The plate shares with the group the ring below its top and the bottom half
/// of the bar and of the pin; the group less the plate is the ring's top and
/// the top half of the bar and of the pin, apart.
#[test]
fn a_plate_and_a_ring_with_a_bar_in_its_hole_and_a_pin_beside_it_fuse_every_way() {
    let the_ring_in_the_plate = 16.0 * PI * PI / 3.0 + 2.0 * PI * 3f64.sqrt();
    let in_the_plate = [the_ring_in_the_plate, THE_BAR / 2.0, THE_PIN / 2.0];
    let shared: f64 = in_the_plate.iter().sum();
    let the_group = THE_RING + THE_BAR + THE_PIN;
    let union = vec![THE_PLATE + the_group - shared];
    let intersection = in_the_plate.to_vec();
    let the_group_less_the_plate = vec![
        THE_RING - the_ring_in_the_plate,
        THE_BAR / 2.0,
        THE_PIN / 2.0,
    ];
    let the_plate_less_the_group = vec![THE_PLATE - shared];
    for (rule, plate_first, truths) in [
        (Rule::Union, true, &union),
        (Rule::Union, false, &union),
        (Rule::Intersection, true, &intersection),
        (Rule::Intersection, false, &intersection),
        (Rule::Difference, true, &the_plate_less_the_group),
        (Rule::Difference, false, &the_group_less_the_plate),
    ] {
        let what = format!("{rule:?}, plate first {plate_first}");
        let mut model = Model::new(Scale::millimetres(100.0).expect("a positive size"));
        let plate = vec![a_bar_about_z(&mut model, 5.5, -2.0, 0.5)];
        let group = vec![
            a_whole_ring(&mut model),
            a_bar_about_z(&mut model, 2.0, 0.25, 0.75),
            model
                .add_round_bar(
                    0.15,
                    0.5,
                    &Isometry::translation(Vector::new(5.2, 0.0, 0.25)).expect("a finite shift"),
                )
                .expect("a bar of radius 0.15"),
        ];
        let (arguments, tools) = if plate_first {
            (plate, group)
        } else {
            (group, plate)
        };
        let answer = model
            .fuse(
                &arguments,
                &tools,
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: refused {why:?}"))
            .solids()
            .to_vec();
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        the_solids_measure(&model, &answer, truths, &what);
    }
}

/// The pin's own pipe against the ring's torus, either way round, and the
/// same two turned and moved off the axes: a pipe along the ring's axis whose
/// nearest reach to the axis is past the tube's outermost.
#[test]
fn a_pipe_along_a_rings_axis_beside_its_tube_meets_it_nowhere() {
    the_two_meet_nowhere(
        &a_ring([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        &a_pipe([5.2, 0.0, 0.25], [0.0, 0.0, 1.0], 0.15),
    );
    // `(5.2, −2.6, 0)` is square to `(1, 2, 3)` and `2.6·√5`, about `5.81`,
    // long, so the pipe of radius `1/2` comes no nearer the axis than `5.31`.
    the_two_meet_nowhere(
        &a_ring([1.0, -2.0, 7.0], [1.0, 2.0, 3.0]),
        &a_pipe([6.2, -4.6, 7.0], [1.0, 2.0, 3.0], 0.5),
    );
}

/// A pin in the ring's hole off its axis: its farthest reach from the axis,
/// `1.5 + 0.5`, is short of the tube's innermost, `3`.
#[test]
fn a_pipe_along_a_rings_axis_in_its_hole_off_the_axis_meets_it_nowhere() {
    the_two_meet_nowhere(
        &a_ring([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        &a_pipe([1.5, 0.0, 0.0], [0.0, 0.0, 1.0], 0.5),
    );
}

/// A pipe round the whole ring off its axis: its nearest reach to the ring's
/// axis, `7 − 1`, is past the tube's outermost, `5`.
#[test]
fn a_pipe_along_a_rings_axis_round_the_whole_ring_meets_it_nowhere() {
    the_two_meet_nowhere(
        &a_ring([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        &a_pipe([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 7.0),
    );
}

/// A pipe along the ring's axis that runs through the tube, or round the
/// axis through the tube, or that rests on the tube at one point from
/// outside, from the hole or from round the whole ring, is not answered as
/// meeting it nowhere: each shares a point with it. Each is refused, as every
/// pipe off the ring's axis was before.
#[test]
fn a_pipe_along_a_rings_axis_that_reaches_its_tube_is_not_said_to_miss_it() {
    let ring = a_ring([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    for (what, pipe) in [
        (
            "through the tube",
            a_pipe([4.5, 0.0, 0.0], [0.0, 0.0, 1.0], 0.15),
        ),
        (
            "round the axis through the tube",
            a_pipe([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 5.0),
        ),
        (
            "resting on the outer rim",
            a_pipe([5.5, 0.0, 0.0], [0.0, 0.0, 1.0], 0.5),
        ),
        (
            "resting on the inner rim",
            a_pipe([2.5, 0.0, 0.0], [0.0, 0.0, 1.0], 0.5),
        ),
        (
            "resting round the outer rim",
            a_pipe([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 6.0),
        ),
    ] {
        for (one, other) in [(&ring, &pipe), (&pipe, &ring)] {
            let met = intersect(one, other, &Policy::certified(), &mut ProvenanceLog::new());
            assert!(
                !matches!(met, Ok(SurfaceMeet::Nothing)),
                "a pipe {what} shares a point with the ring: {met:?}"
            );
            assert_eq!(
                met,
                Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface),
                "a pipe {what}"
            );
        }
    }
}

/// A plane holding the ring's axis, the ring and the pin's pipe beside it share
/// no point, since the ring and the pipe share none, in every order.
#[test]
fn a_plane_through_a_rings_axis_the_ring_and_a_pipe_beside_it_meet_nowhere() {
    let ring = a_ring([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    let pipe = a_pipe([5.2, 0.0, 0.25], [0.0, 0.0, 1.0], 0.15);
    let flat = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
            .expect("the y axis is a unit normal"),
    );
    for surfaces in [
        [&flat, &ring, &pipe],
        [&ring, &pipe, &flat],
        [&pipe, &flat, &ring],
    ] {
        assert_eq!(
            meet_of_three_surfaces(surfaces, &Policy::certified(), &mut ProvenanceLog::new()),
            Ok(MeetOfThree::Nowhere),
            "{surfaces:?}"
        );
    }
}

fn the_two_meet_nowhere(ring: &Surface, pipe: &Surface) {
    for (one, other) in [(ring, pipe), (pipe, ring)] {
        assert_eq!(
            intersect(one, other, &Policy::certified(), &mut ProvenanceLog::new()),
            Ok(SurfaceMeet::Nothing),
            "{one:?} against {other:?}"
        );
    }
}

/// The ring of four round a tube of one about the line through `centre`
/// along `along`.
fn a_ring(centre: [f64; 3], along: [f64; 3]) -> Surface {
    Surface::Torus(
        Torus::new(
            Axis::new(Point::from(centre), Vector::from(along)).expect("a direction"),
            4.0,
            1.0,
        )
        .expect("a ring wider than its tube"),
    )
}

fn a_pipe(through: [f64; 3], along: [f64; 3], radius: f64) -> Surface {
    Surface::Cylinder(
        Cylinder::new(
            Axis::new(Point::from(through), Vector::from(along)).expect("a direction"),
            radius,
        )
        .expect("a radius"),
    )
}

/// A round bar of radius `radius` about `z`, from `z = bottom` up to `top`.
fn a_bar_about_z(model: &mut Model, radius: f64, bottom: f64, top: f64) -> SolidKey {
    model
        .add_round_bar(
            radius,
            top - bottom,
            &Isometry::translation(Vector::new(0.0, 0.0, bottom)).expect("a finite shift"),
        )
        .expect("a round bar")
}

/// The ring of four round a tube of one about `z`, right round, as two faces:
/// the half below `z = 0` and the half above, meeting along the outer and the
/// inner equator.
fn a_whole_ring(model: &mut Model) -> SolidKey {
    let middle = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let skin = Surface::Torus(Torus::new(Axis::z(), 4.0, 1.0).expect("a tube narrower"));
    let mut builder = SolidBuilder::new();
    let [outer, inner] = [-5.0, -3.0].map(|x| {
        let corner = builder
            .add_vertex(Point::new(x, 0.0, 0.0))
            .expect("a corner on the ring and on the plane");
        builder
            .add_edge(
                corner,
                corner,
                EdgeRun::RightRoundWhereItCrosses {
                    and: Surface::Plane(middle),
                },
            )
            .expect("an equator is a whole closed curve, so one corner twice is an edge")
    });
    let mut half = |turns: [Orientation; 2]| {
        let wires = [outer, inner].into_iter().zip(turns).map(|(edge, turn)| {
            let coedge = builder
                .add_coedge(edge, turn)
                .expect("an edge of this build");
            builder
                .add_wire(&[DraftStep::Along(coedge)])
                .expect("one closed edge is a wire")
        });
        let wires: Vec<_> = wires.collect();
        builder
            .add_face(skin, &wires)
            .expect("a band between two circles")
    };
    let one = half([Orientation::Reversed, Orientation::Forward]);
    let other = half([Orientation::Forward, Orientation::Reversed]);
    builder
        .add_shell(&[(one, Orientation::Forward), (other, Orientation::Forward)])
        .expect("two halves close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a whole ring in two faces is a solid")
}

/// One solid for each of `truths`, each measuring as its own one inside the
/// error its measurement off a mesh as fine as [`DEFLECTION`] gives.
fn the_solids_measure(model: &Model, answer: &[SolidKey], truths: &[f64], what: &str) {
    assert_eq!(
        answer.len(),
        truths.len(),
        "{what}: {answer:?} against {truths:?}"
    );
    let mut left = truths.to_vec();
    for &solid in answer {
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(DEFLECTION).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: the solid measures: {why:?}"));
        let at = left
            .iter()
            .position(|&truth| (measured.volume() - truth).abs() <= measured.volume_error())
            .unwrap_or_else(|| {
                panic!(
                    "{what}: {} measured, {} off, against {left:?}",
                    measured.volume(),
                    measured.volume_error()
                )
            });
        left.remove(at);
    }
}
