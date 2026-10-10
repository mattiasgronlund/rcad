//! The top half of a ring standing on a round plate, fused every way (row
//! Q1514).
//!
//! **The half ring** is `rings::the_top_half_of_a_ring(model, 4, 1, seam)`: the
//! tube of radius one round the ring of radius four about `z`, cut by `z = 0`
//! and keeping `z ≥ 0`. By Pappus it holds half of `2π² · 4 · 1`, so `4π²`.
//!
//! **The plate** is a round bar of radius six about `z`, from `z = −2` up to
//! its top. With its top at `z = 0` the half ring stands on it, its flat face
//! lying on the plate's top, and the plate holds `72π`. With its top at
//! `z = 1/2` it holds `90π`, and its top cuts the half ring in two whole
//! circles round the axis, of radius `4 ± √3/2`.
//!
//! What the half ring and the taller plate share is the tube's cross-section
//! between `z = 0` and `z = 1/2`, `√3/4 + π/6`, swept round the ring's circle
//! of length `8π`: `4π²/3 + 2π√3`.
//!
//! **The bar** (row Q1580) is a round bar of radius `23/8` about `z`, from
//! `z = −1` up to its top, standing in the half ring's hole and touching
//! nothing of it, fused with the half ring as one group against the low plate,
//! or against the half ring alone (row Q1515). Each unit of its height holds
//! `529π/64`, and one unit of it is inside the low plate. With its top at
//! `z = 2` it stands above the tube, and with its top at `z = 1` level with
//! the top of the tube, where the tube's skin is one circle four out (row
//! Q1588).
//!
//! **The whole ring** is the same tube right round, written as its two halves
//! either side of `z = 0`, and it holds `8π²`. Against a bar it touches
//! nowhere, nothing cuts it, and a fuse has to hand it back whole (row Q1588).
//! Turned about six axes a fortieth of a turn at a time, it is handed back at
//! all but seven of the 234 turns.

use core::f64::consts::{PI, TAU};

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, Rule, SolidBuilder, SolidKey};

mod rings;

/// How fine a mesh every volume here is read off.
const DEFLECTION: f64 = 0.01;

/// Half of a ring of four round a tube of one.
const THE_HALF_RING: f64 = 4.0 * PI * PI;

/// The whole of that ring.
const THE_WHOLE_RING: f64 = 8.0 * PI * PI;

/// The plate with its top at `z = 0`.
const THE_LOW_PLATE: f64 = 72.0 * PI;

/// The plate with its top at `z = 1/2`.
const THE_HIGH_PLATE: f64 = 90.0 * PI;

/// One unit of the bar's height.
const A_UNIT_OF_THE_BAR: f64 = 529.0 * PI / 64.0;

#[test]
fn standing_on_the_plate_with_a_seam_every_fuse_answers() {
    standing_on_the_plate_every_fuse_answers(true);
}

#[test]
fn standing_on_the_plate_with_no_seam_every_fuse_answers() {
    standing_on_the_plate_every_fuse_answers(false);
}

/// The half ring's flat face lies on the plate's top, so the two touch and
/// share nothing: the union is one solid of both, the intersection nothing,
/// and each difference its first argument.
fn standing_on_the_plate_every_fuse_answers(a_seam: bool) {
    every_fuse_answers(
        a_seam,
        0.0,
        None,
        [
            vec![THE_LOW_PLATE + THE_HALF_RING],
            Vec::new(),
            vec![THE_HALF_RING],
            vec![THE_LOW_PLATE],
        ],
    );
}

#[test]
fn cut_by_the_plate_with_a_seam_every_fuse_answers() {
    cut_by_the_plate_every_fuse_answers(true);
}

#[test]
fn cut_by_the_plate_with_no_seam_every_fuse_answers() {
    cut_by_the_plate_every_fuse_answers(false);
}

/// The plate's top at `z = 1/2` cuts the half ring in two circles, so every
/// answer is one solid: the union, the band of the tube below `z = 1/2`, the
/// cap of the tube above it, and the plate with that band taken out.
fn cut_by_the_plate_every_fuse_answers(a_seam: bool) {
    let shared = 4.0 * PI * PI / 3.0 + 2.0 * PI * 3f64.sqrt();
    every_fuse_answers(
        a_seam,
        0.5,
        None,
        [
            vec![THE_HIGH_PLATE + THE_HALF_RING - shared],
            vec![shared],
            vec![THE_HALF_RING - shared],
            vec![THE_HIGH_PLATE - shared],
        ],
    );
}

#[test]
fn with_a_bar_in_its_hole_standing_on_the_plate_with_a_seam_every_fuse_answers() {
    with_a_bar_in_its_hole_standing_on_the_plate_every_fuse_answers(true);
}

#[test]
fn with_a_bar_in_its_hole_standing_on_the_plate_with_no_seam_every_fuse_answers() {
    with_a_bar_in_its_hole_standing_on_the_plate_every_fuse_answers(false);
}

/// The half ring and the bar up to `z = 2`, as one group, on the low plate:
/// the half ring touches the plate and the bar goes one unit into it, so the
/// union is one solid, the intersection the bar's unit in the plate, the group
/// less the plate the half ring and the bar's two units above it, and the
/// plate less the group the plate with the bar's hole in it.
///
/// The plate's top between the bar's circle and the half ring's inner circle
/// is a band whose piece comes with the bar's circle as its first wire. Read
/// as the band's outer wire, it wound every point of the band zero times, and
/// fuses with the half ring first and with the plate first were refused 'a
/// point inside a piece of a face' at `(0, 23/8, 0)` and `(3, 0, 0)` (row
/// Q1580).
fn with_a_bar_in_its_hole_standing_on_the_plate_every_fuse_answers(a_seam: bool) {
    every_fuse_answers(
        a_seam,
        0.0,
        Some(2.0),
        [
            vec![THE_LOW_PLATE + THE_HALF_RING + 2.0 * A_UNIT_OF_THE_BAR],
            vec![A_UNIT_OF_THE_BAR],
            vec![THE_HALF_RING, 2.0 * A_UNIT_OF_THE_BAR],
            vec![THE_LOW_PLATE - A_UNIT_OF_THE_BAR],
        ],
    );
}

#[test]
fn with_a_bar_level_with_the_top_of_the_tube_standing_on_the_plate_with_a_seam_every_fuse_answers()
{
    with_a_bar_level_with_the_top_of_the_tube_standing_on_the_plate_every_fuse_answers(true);
}

#[test]
fn with_a_bar_level_with_the_top_of_the_tube_standing_on_the_plate_with_no_seam_every_fuse_answers()
{
    with_a_bar_level_with_the_top_of_the_tube_standing_on_the_plate_every_fuse_answers(false);
}

/// The bar's top at `z = 1`, level with the top of the tube, touches the half
/// ring's skin nowhere: there the skin is the one circle of radius four, and
/// the bar's top reaches `23/8` out. The answers are those of
/// [`with_a_bar_in_its_hole_standing_on_the_plate_every_fuse_answers`] with one
/// unit of the bar less above the plate.
///
/// Every one of them was refused 'a point inside a piece of a face' at
/// `(4, 0, 1)` (row Q1588). The bar's top plane cut the half ring's face along
/// that circle, because nothing proved a torus clear of the band round the
/// axis the bar's top lies in, and the piece beyond the circle, on the side of
/// the plane where the skin has no point, had no point inside to place.
fn with_a_bar_level_with_the_top_of_the_tube_standing_on_the_plate_every_fuse_answers(
    a_seam: bool,
) {
    every_fuse_answers(
        a_seam,
        0.0,
        Some(1.0),
        [
            vec![THE_LOW_PLATE + THE_HALF_RING + A_UNIT_OF_THE_BAR],
            vec![A_UNIT_OF_THE_BAR],
            vec![THE_HALF_RING, A_UNIT_OF_THE_BAR],
            vec![THE_LOW_PLATE - A_UNIT_OF_THE_BAR],
        ],
    );
}

#[test]
fn with_a_bar_level_with_the_top_of_the_tube_and_no_plate_with_a_seam_every_fuse_answers() {
    with_a_bar_level_with_the_top_of_the_tube_and_no_plate_every_fuse_answers(true);
}

#[test]
fn with_a_bar_level_with_the_top_of_the_tube_and_no_plate_with_no_seam_every_fuse_answers() {
    with_a_bar_level_with_the_top_of_the_tube_and_no_plate_every_fuse_answers(false);
}

/// The half ring against that bar alone (row Q1515): the two touch nowhere, so
/// the union is the two solids apart, the intersection nothing, and each
/// difference its first argument. Every one was refused at `(4, 0, 1)` for
/// the same reason as on the plate (row Q1588).
fn with_a_bar_level_with_the_top_of_the_tube_and_no_plate_every_fuse_answers(a_seam: bool) {
    every_fuse_of_the_two_answers(
        &format!("seam {a_seam}, the bar alone"),
        |model| {
            (
                vec![the_half_ring(model, a_seam)],
                vec![a_bar_in_the_hole(model, 1.0)],
            )
        },
        [THE_HALF_RING, 2.0 * A_UNIT_OF_THE_BAR],
        [
            vec![THE_HALF_RING, 2.0 * A_UNIT_OF_THE_BAR],
            Vec::new(),
            vec![THE_HALF_RING],
            vec![2.0 * A_UNIT_OF_THE_BAR],
        ],
    );
}

/// The whole ring and the bar from `z = −1` to `1` standing twenty out from
/// its axis, where no box of the one reaches the other. Nothing cuts the ring,
/// so `bake` joins its two halves into one face with no boundary at all, and
/// every fuse was refused `ACurvedFaceGoesRightRoundWithNoSeam`: a ring has no
/// point to open a seam up to, as a ball does (row Q1588).
#[test]
fn a_whole_ring_and_a_bar_far_from_it_fuse_to_the_two_apart() {
    a_whole_ring_and_a_bar_clear_of_it_fuse_to_the_two_apart("far from it", 20.0);
}

/// The whole ring and the bar in its hole from `z = −1` to `1`, level with
/// the top and the bottom of the tube. The bar's caps reach `23/8` out and the
/// tube's skin is one circle four out at `z = ±1`, so the two touch nowhere;
/// once that is proved, nothing cuts the ring, as when the bar is far from it.
#[test]
fn a_whole_ring_and_a_bar_in_its_hole_level_with_its_tube_fuse_to_the_two_apart() {
    a_whole_ring_and_a_bar_clear_of_it_fuse_to_the_two_apart("in its hole", 0.0);
}

/// The whole ring turned a quarter of a turn about `(1, 2, 3)`, and the bar
/// twenty out. The plane a turned ring is cut along has to keep the bits of
/// the ring's own axis. Scaled again, it sits an ulp off square to that axis,
/// its crossing with the ring is a quartic, and the fuse was refused
/// `ACrossingBetweenAPlaneAndATorusThatIsAQuartic` (row Q1588).
#[test]
fn a_whole_ring_turned_a_quarter_about_a_slant_and_a_bar_far_from_it_fuse_to_the_two_apart() {
    let slant = Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("a direction");
    a_turned_whole_ring_and_a_bar_clear_of_it_fuse_to_the_two_apart(
        "far from it, turned a quarter about (1, 2, 3)",
        20.0,
        &Isometry::rotation(&slant, TAU / 4.0).expect("a finite turn"),
    );
}

/// The whole ring turned about six axes a fortieth of a turn at a time, and
/// the bar twenty out, fused under every rule both ways round: 1404 fuses.
/// Each one that answers gives the two apart, nothing, or its first argument,
/// and validates. The rest, 21, are refused `VolumeIsNotPositive`, a ring of
/// volume `8π²` read as enclosing about `−4·10⁻¹⁶`. They are the three fuses
/// whose answer holds the ring, at seven of the 234 turns, and that is a
/// defect pinned at its count: the answer exists (row Q1588).
#[test]
fn a_whole_ring_turned_any_way_and_a_bar_far_from_it_answer_or_are_refused_as_enclosing_nothing() {
    let mut refused = Vec::new();
    for direction in [
        Vector::new(1.0, 2.0, 3.0),
        Vector::new(1.0, 0.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
        Vector::new(1.0, 1.0, 0.0),
        Vector::new(-3.0, 1.0, 7.0),
        Vector::new(0.3, -0.7, 0.1),
    ] {
        let axis = Axis::new(Point::origin(), direction).expect("a direction");
        for fortieths in 1..40 {
            let turn = Isometry::rotation(&axis, f64::from(fortieths) * TAU / 40.0)
                .expect("a finite turn");
            for (rule, ring_first, solids) in [
                (Rule::Union, true, 2),
                (Rule::Union, false, 2),
                (Rule::Intersection, true, 0),
                (Rule::Intersection, false, 0),
                (Rule::Difference, true, 1),
                (Rule::Difference, false, 1),
            ] {
                let what = format!(
                    "{rule:?}, ring first {ring_first}, {fortieths}/40 about {direction:?}"
                );
                let mut model = Model::new(Scale::millimetres(50.0).expect("a positive size"));
                let ring = a_whole_ring(&mut model);
                model
                    .place_solid(ring, &turn)
                    .expect("this model made the key");
                let bar = a_bar_standing(&mut model, 20.0, 1.0);
                let (arguments, tools) = if ring_first {
                    ([ring], [bar])
                } else {
                    ([bar], [ring])
                };
                match model.fuse(
                    &arguments,
                    &tools,
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                ) {
                    Ok(answer) => {
                        assert_eq!(answer.solids().len(), solids, "{what}");
                        model
                            .validate(&Policy::certified(), &mut ProvenanceLog::new())
                            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
                    }
                    Err(Error::Invalid(Invalid::VolumeIsNotPositive { volume }))
                        if volume.abs() < 1e-12 =>
                    {
                        refused.push(what);
                    }
                    Err(why) => panic!("{what}: refused {why:?}"),
                }
            }
        }
    }
    assert_eq!(refused.len(), 21, "{refused:#?}");
}

fn a_whole_ring_and_a_bar_clear_of_it_fuse_to_the_two_apart(shape: &str, out: f64) {
    a_turned_whole_ring_and_a_bar_clear_of_it_fuse_to_the_two_apart(
        shape,
        out,
        &Isometry::identity(),
    );
}

fn a_turned_whole_ring_and_a_bar_clear_of_it_fuse_to_the_two_apart(
    shape: &str,
    out: f64,
    turn: &Isometry,
) {
    every_fuse_of_the_two_answers(
        &format!("the whole ring and a bar {shape}"),
        |model| {
            let ring = a_whole_ring(model);
            model
                .place_solid(ring, turn)
                .expect("this model made the key");
            (vec![ring], vec![a_bar_standing(model, out, 1.0)])
        },
        [THE_WHOLE_RING, 2.0 * A_UNIT_OF_THE_BAR],
        [
            vec![THE_WHOLE_RING, 2.0 * A_UNIT_OF_THE_BAR],
            Vec::new(),
            vec![THE_WHOLE_RING],
            vec![2.0 * A_UNIT_OF_THE_BAR],
        ],
    );
}

/// Every rule, with the half ring as the argument and as the tool, on a plate
/// whose top is at `top`, and with the bar beside the half ring in one group
/// where `bar` gives its top. `truths` are the union, the intersection, the
/// group less the plate and the plate less the group.
fn every_fuse_answers(a_seam: bool, top: f64, bar: Option<f64>, truths: [Vec<f64>; 4]) {
    let the_bar = bar.map_or(0.0, |bar_top| (bar_top + 1.0) * A_UNIT_OF_THE_BAR);
    let the_plate = (top + 2.0) * 36.0 * PI;
    every_fuse_of_the_two_answers(
        &format!("seam {a_seam}, top {top}, bar {bar:?}"),
        |model| {
            let mut group = vec![the_half_ring(model, a_seam)];
            group.extend(bar.map(|bar_top| a_bar_in_the_hole(model, bar_top)));
            (group, vec![a_plate(model, top)])
        },
        [THE_HALF_RING + the_bar, the_plate],
        truths,
    );
}

/// Every rule, with the group `build` makes first as the argument and as the
/// tool, against the other group it makes. `sizes` are what the two groups
/// hold, and `truths` the union, the intersection, the first less the other
/// and the other less the first, which are checked to add up first: the union
/// is both less what they share, and each difference its first argument less
/// what they share.
fn every_fuse_of_the_two_answers(
    shape: &str,
    build: impl Fn(&mut Model) -> (Vec<SolidKey>, Vec<SolidKey>),
    sizes: [f64; 2],
    truths: [Vec<f64>; 4],
) {
    let [union, intersection, first_less_other, other_less_first] = truths;
    let [first, other] = sizes;
    let shared: f64 = intersection.iter().sum();
    for (truth, should_be) in [
        (&union, first + other - shared),
        (&first_less_other, first - shared),
        (&other_less_first, other - shared),
    ] {
        let sum: f64 = truth.iter().sum();
        assert!(
            (sum - should_be).abs() <= 1e-9 * should_be,
            "{shape}: {truth:?} adds up to {sum}, not {should_be}"
        );
    }
    for (rule, ring_first, truths) in [
        (Rule::Union, true, &union),
        (Rule::Union, false, &union),
        (Rule::Intersection, true, &intersection),
        (Rule::Intersection, false, &intersection),
        (Rule::Difference, true, &first_less_other),
        (Rule::Difference, false, &other_less_first),
    ] {
        let what = format!("{rule:?}, {shape}, half ring first {ring_first}");
        let mut model = Model::new(Scale::millimetres(50.0).expect("a positive size"));
        let (group, other) = build(&mut model);
        let (arguments, tools) = if ring_first {
            (group, other)
        } else {
            (other, group)
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

fn the_half_ring(model: &mut Model, a_seam: bool) -> SolidKey {
    rings::the_top_half_of_a_ring(model, 4.0, 1.0, a_seam)
        .expect("the top half of a ring described this way is a solid")
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

/// A round bar of radius `23/8` about `z`, from `z = −1` up to `top`.
fn a_bar_in_the_hole(model: &mut Model, top: f64) -> SolidKey {
    a_bar_standing(model, 0.0, top)
}

/// A round bar of radius `23/8` about the line parallel to `z` through
/// `(out, 0)`, from `z = −1` up to `top`.
fn a_bar_standing(model: &mut Model, out: f64, top: f64) -> SolidKey {
    model
        .add_round_bar(
            23.0 / 8.0,
            top + 1.0,
            &Isometry::translation(Vector::new(out, 0.0, -1.0)).expect("a finite shift"),
        )
        .expect("a bar of radius 23/8")
}

/// A round bar of radius six about `z`, from `z = −2` up to `top`.
fn a_plate(model: &mut Model, top: f64) -> SolidKey {
    model
        .add_round_bar(
            6.0,
            top + 2.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -2.0)).expect("a finite shift"),
        )
        .expect("a bar of radius 6")
}

/// One solid for each of `truths`, each measuring as its own one inside the
/// error its measurement gives.
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
