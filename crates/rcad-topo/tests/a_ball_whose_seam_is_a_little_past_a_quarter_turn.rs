//! A whole ball cut open on one half-meridian a little past a quarter turn from
//! `x` measures a ball, wherever round the axis its seam is (Q1304).
//!
//! The ball is one sphere face, bounded the way `Model::add_ball` bounds its
//! own: down the seam, over the south pole, back up the seam, over the north
//! pole. Each pole is a whole side of the face's patch, a whole turn of
//! longitude wide, and the walk round the wire writes a corner at each end of
//! it. The far end of the south pole's side and the first corner back up the
//! seam are the same longitude, and the mesher worked them out two ways: the
//! side's far end as the seam's longitude plus a whole turn, rounded once, and
//! the corner back up as that plus the short way round from it to the seam's
//! own reading, rounded three times. With the seam at 99 degrees the two came
//! out one unit in the last place apart, the patch got a pole side one rounding
//! wide, and the triangulator could not put a point beside it: the ball was
//! refused `CannotMeshAFace`, "the mesh still strays 1.67". 13 of 3600 seams a
//! tenth of a degree apart were, all from 99.0 to 114.1 degrees. From 98.4
//! degrees round to 180 the seam's longitude plus a whole turn lands between 8
//! and 16, where one rounding is `1.8e-15`, four or eight times the seam's own.
//!
//! The mesher now writes the side's far end at the longitude it works out for
//! the corner back up the seam, so the two are one number however it rounds.
//!
//! The same ball made by `add_ball` and turned about `z` measured at every one
//! of those angles, because its seam is stored at zero and the turn is kept
//! beside it rather than in its numbers. Written to a STEP file and read back,
//! the turn is baked in, and that is the route by which the night's fuzzing
//! found it (Q1212, Q1213).
//!
//! The oracle is the closed form, `4/3 · π · r³`, and the measure must land
//! within the error it publishes for itself.

use core::f64::consts::PI;

use rcad_geom::{
    Deflection, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Scale, Sphere, Surface,
    Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey};

/// The angle the fuzzer's balls were first seen refused at, at the radius and
/// deflection they were refused at.
#[test]
fn a_ball_whose_seam_is_99_degrees_from_x_measures_a_ball() {
    a_ball_cut_open_at(99.0, 6.0).measures_a_ball_at(0.022);
}

/// Every seam a tenth of a degree apart that Q1213's sweep found refused at a
/// radius of 6 and a deflection of 0.022.
#[test]
fn the_thirteen_seams_a_sweep_found_refused_each_measure_a_ball() {
    for degrees in [
        99.0, 99.2, 102.4, 102.6, 103.9, 105.8, 107.3, 107.5, 110.5, 110.7, 112.2, 113.9, 114.1,
    ] {
        a_ball_cut_open_at(degrees, 6.0).measures_a_ball_at(0.022);
    }
}

/// Q1212's ball at rest at the origin with its seam at 109 degrees, which was
/// refused at every radius and every deflection it was tried at.
///
/// The angle is `to_radians`'s, which is `109 · (π / 180)` and not
/// `109 · π / 180`: the two are a rounding apart, and only the first is the
/// angle Q1212 was refused at.
#[test]
fn a_ball_whose_seam_is_109_degrees_from_x_measures_a_ball_at_radius_1_3_and_8() {
    for radius in [1.0, 3.0, 8.0] {
        for deflection in [0.1, 0.031, 0.0155] {
            a_ball_cut_open_at_an_angle(109f64.to_radians(), radius, 109.0)
                .measures_a_ball_at(deflection);
        }
    }
}

/// A unit ball with its seam at every whole degree round the axis. Before the
/// fix 99 was refused, at this radius and deflection as at the others above.
#[test]
fn a_unit_ball_measures_a_ball_with_its_seam_at_every_whole_degree() {
    for degrees in 0..360 {
        a_ball_cut_open_at(f64::from(degrees), 1.0).measures_a_ball_at(0.1);
    }
}

/// A unit ball with its seam at every tenth of a degree through the second
/// quarter turn, which holds all 13 of the seams above. A unit ball at a
/// deflection of 0.1 refused the same 13 as the radius-6 one at 0.022, in
/// about a thirtieth of the time; the whole turn in tenths takes four times
/// this one and found nothing else.
#[test]
fn a_unit_ball_measures_a_ball_with_its_seam_at_every_tenth_of_a_degree_from_90_to_180() {
    for tenths in 900..=1800 {
        a_ball_cut_open_at(f64::from(tenths) / 10.0, 1.0).measures_a_ball_at(0.1);
    }
}

/// A ball of `radius` about the origin whose seam is the half-meridian
/// `degrees` round from `x`, built face by face.
fn a_ball_cut_open_at(degrees: f64, radius: f64) -> BuiltBall {
    a_ball_cut_open_at_an_angle(degrees * PI / 180.0, radius, degrees)
}

fn a_ball_cut_open_at_an_angle(angle: f64, radius: f64, degrees: f64) -> BuiltBall {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let mut builder = SolidBuilder::new();
    let north = builder
        .add_vertex(Point::new(0.0, 0.0, radius))
        .expect("a finite point");
    let south = builder
        .add_vertex(Point::new(0.0, 0.0, -radius))
        .expect("a finite point");
    let (sin, cos) = angle.sin_cos();
    let meridian = Plane::from_point_and_normal(Point::origin(), Vector::new(-sin, cos, 0.0))
        .expect("a normal with a length");
    let the_seams_side =
        HalfSpace::through_a_point_facing(Point::origin(), Vector::new(cos, sin, 0.0))
            .expect("a normal with a length");
    let seam = builder
        .add_edge(
            north,
            south,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(meridian),
                picked_by: [Picker::InFrontOf(the_seams_side)].into_iter().collect(),
            },
        )
        .expect("the half-meridian runs from pole to pole");
    let down = builder
        .add_coedge(seam, Orientation::Forward)
        .expect("an edge this builder made");
    let up = builder
        .add_coedge(seam, Orientation::Reversed)
        .expect("an edge this builder made");
    let wire = builder
        .add_wire(&[
            DraftStep::Along(down),
            DraftStep::Pole(south),
            DraftStep::Along(up),
            DraftStep::Pole(north),
        ])
        .expect("the seam down, the south pole, the seam up, the north pole");
    let face = builder
        .add_face(
            Surface::Sphere(Sphere::new(Point::origin(), radius).expect("a radius")),
            &[wire],
        )
        .expect("a wire this builder made");
    builder
        .add_shell(&[(face, Orientation::Forward)])
        .expect("a face this builder made");
    let solid = builder
        .build(&mut model, &policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("a ball cut open at {degrees} degrees builds: {why}"));
    BuiltBall {
        model,
        solid,
        degrees,
        radius,
    }
}

struct BuiltBall {
    model: Model,
    solid: SolidKey,
    degrees: f64,
    radius: f64,
}

impl BuiltBall {
    fn measures_a_ball_at(&self, deflection: f64) {
        let Self {
            degrees, radius, ..
        } = *self;
        let measured = self
            .model
            .solid(self.solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(deflection).expect("a working deflection"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| {
                panic!(
                    "a ball of radius {radius} cut open at {degrees} degrees, at a deflection \
                     of {deflection}: {why}"
                )
            });
        let truth = 4.0 / 3.0 * PI * radius.powi(3);
        assert!(
            (measured.volume() - truth).abs() <= measured.volume_error(),
            "a ball of radius {radius} cut open at {degrees} degrees, at a deflection of \
             {deflection}, measured {} where a ball is {truth}, and said it was within {}",
            measured.volume(),
            measured.volume_error()
        );
    }
}
