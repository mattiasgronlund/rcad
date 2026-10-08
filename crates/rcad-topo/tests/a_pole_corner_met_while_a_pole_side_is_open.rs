//! A whole ball whose seam has a corner a hair from a pole measures a ball, or
//! is refused by name (Q1309).
//!
//! The ball is one sphere face bounded the way `Model::add_ball` bounds its
//! own — down the seam, over the south pole, back up, over the north pole —
//! but with the seam cut into edges, one of them from a pole to a corner
//! `3e-14` away along the seam. The mesher calls that corner a pole too
//! (`the_pole_a_point_sits_within_a_rounding_of`), and the edge to it has no
//! points along it, so the walk meets it right after crossing the pole and
//! before any corner with a longitude of its own.
//!
//! **There the walk opened the pole side again**, as a side between two
//! meridians, and a seam's side is a whole turn. A side between two meridians
//! runs as far as the two readings of the seam are apart: nothing, which it
//! counts as a whole turn, when the walk reaches the pole on its first pass
//! down the seam; a hair, when it reaches it on its second, after the
//! longitude has been carried a whole turn and rounded. With a third corner on
//! the equator the walk sets off there, and set off the way that reaches the
//! hair's pole second, the ball was refused as a wire that will not lie flat
//! at 22 of 720 whole degrees and poles, measured at a tenth of the radius.
//!
//! **The radius is `0.001` because nothing bigger is inside the range check.**
//! One rounding of a pole is `256 · ε` of the largest coordinate and of one, so
//! for a ball of radius one or more the edge to the corner is at most `5.7e-14`
//! of the radius and the solid spans `1.76e13` of its own smallest feature,
//! where `docs/decisions.md` §3.5 refuses past `1e11`. Under radius `5.7e-3`
//! the floor of one in that rounding lets a corner that far off stay inside it.
//!
//! The oracle is the closed form, `4/3 · π · r³` and `4 · π · r²`, and each
//! measure must land within the error it publishes for itself.

use core::f64::consts::PI;

use rcad_geom::{
    Deflection, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Scale, Sphere, Surface,
    Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, SolidBuilder, SolidKey};

const RADIUS: f64 = 0.001;
const A_HAIR: f64 = 3e-14;

/// The first angle the walk crossed the north pole a hair rather than a whole
/// turn at, setting off from the equator corner down the seam.
#[test]
fn a_ball_whose_seam_has_a_corner_a_hair_from_its_north_pole_at_104_degrees_measures_a_ball() {
    a_ball(104.0, Pole::North, Equator::Cornered, SetOff::Down)
        .unwrap_or_else(|why| panic!("the ball builds: {why}"))
        .measures_a_ball_at(0.01);
}

/// Every whole degree, the hair at either pole, and the walk setting off from
/// the equator corner either way along the seam. The ones refused before the
/// fix all reached the hair's pole on the walk's second pass along the seam:
/// set off down it with the hair at the north pole, or up it with the hair at
/// the south.
///
/// Measured at a tenth of the radius, which takes a quarter of the time a
/// hundredth does over 1440 balls; a pole side a hair wide in place of a whole
/// turn is a ball refused or measured as nothing, which any deflection sees.
#[test]
fn a_ball_whose_seam_has_a_corner_on_its_equator_and_one_a_hair_from_a_pole_measures_a_ball() {
    for degrees in 0..360 {
        for pole in [Pole::North, Pole::South] {
            for set_off in [SetOff::Down, SetOff::Up] {
                a_ball(f64::from(degrees), pole, Equator::Cornered, set_off)
                    .unwrap_or_else(|why| {
                        panic!("the ball at {degrees} degrees, {pole:?}, {set_off:?} builds: {why}")
                    })
                    .measures_a_ball_at(0.1);
            }
        }
    }
}

/// With no corner on the equator every corner of the wire is a pole, so the
/// walk sets off along the first edge the wire lists. Listed from the far
/// pole, the walk meets the hair corner right after the pole step on its
/// first pass, where the two readings of the seam are one number, and that
/// measured a ball before the fix as well as after.
#[test]
fn a_ball_whose_seam_runs_from_the_far_pole_to_a_hair_from_the_near_one_measures_a_ball() {
    for degrees in 0..360 {
        for pole in [Pole::North, Pole::South] {
            a_ball(
                f64::from(degrees),
                pole,
                Equator::Bare,
                SetOff::FromTheFarPole,
            )
            .unwrap_or_else(|why| panic!("the ball at {degrees} degrees, {pole:?} builds: {why}"))
            .measures_a_ball_at(0.1);
        }
    }
}

/// Listed from the hair corner, the walk sets off where there is no longitude
/// and crosses the pole before it has one, which the walk refuses rather than
/// guesses. A ball, and a refusal of one; pinned so that the day it measures,
/// this says so.
#[test]
fn a_ball_whose_wire_is_listed_from_the_hair_corner_is_refused_as_a_wire_that_will_not_lie_flat() {
    for pole in [Pole::North, Pole::South] {
        let refusal = a_ball(99.0, pole, Equator::Bare, SetOff::FromTheHair).err();
        assert!(
            matches!(refusal, Some(Error::AWireWillNotLieFlatOnItsFace { .. })),
            "{pole:?}: {refusal:?}"
        );
    }
}

/// Listed from the pole the hair is at, the walk closes on that pole along a
/// side from the hair corner, two points a hair apart in space and a whole
/// turn apart in the patch. Pinned for the same reason.
#[test]
fn a_ball_whose_wire_is_listed_from_the_pole_by_the_hair_is_refused_as_facing_two_ways() {
    for pole in [Pole::North, Pole::South] {
        let refusal = a_ball(99.0, pole, Equator::Bare, SetOff::FromTheNearPole).err();
        assert!(
            matches!(
                refusal,
                Some(Error::Invalid(Invalid::FacesDoNotAgreeWhichWayTheyFace))
            ),
            "{pole:?}: {refusal:?}"
        );
    }
}

/// At radius six the same corner, `1e-13` off the pole, is inside one
/// rounding of it and outside the range check, which refuses before anything
/// is walked: the solid spans `6e13` of its own smallest feature.
#[test]
fn a_hair_corner_on_a_ball_of_radius_six_is_refused_as_too_wide_a_range() {
    let refusal = a_ball_of(
        6.0,
        1e-13,
        99.0,
        Pole::South,
        Equator::Bare,
        SetOff::FromTheFarPole,
    )
    .err();
    assert!(
        matches!(
            refusal,
            Some(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) if ratio > 1e11
        ),
        "{refusal:?}"
    );
}

#[derive(Clone, Copy, Debug)]
enum Pole {
    North,
    South,
}

/// Whether the seam has a corner on the equator, which the walk sets off from
/// when there is one.
#[derive(Clone, Copy)]
enum Equator {
    Cornered,
    Bare,
}

/// Which edge the wire is listed from. The walk sets off from the first
/// corner off a pole and, where every corner is a pole, from the first edge.
#[derive(Clone, Copy, Debug)]
enum SetOff {
    /// From the equator corner, down the seam.
    Down,
    /// From the equator corner, up the seam.
    Up,
    /// From the pole across the ball from the hair, along the seam.
    FromTheFarPole,
    /// From the hair corner, along the edge to its pole.
    FromTheHair,
    /// From the pole beside the hair, along the edge to it.
    FromTheNearPole,
}

fn a_ball(degrees: f64, pole: Pole, equator: Equator, set_off: SetOff) -> Result<BuiltBall, Error> {
    a_ball_of(RADIUS, A_HAIR, degrees, pole, equator, set_off)
}

/// A ball of `radius` about the origin whose seam is the half-meridian
/// `degrees` round from `x`, with a corner `hair` from `pole` along it, built
/// face by face.
fn a_ball_of(
    radius: f64,
    hair: f64,
    degrees: f64,
    pole: Pole,
    equator: Equator,
    set_off: SetOff,
) -> Result<BuiltBall, Error> {
    let angle = degrees * PI / 180.0;
    let (sin, cos) = angle.sin_cos();
    let up = match pole {
        Pole::North => 1.0,
        Pole::South => -1.0,
    };
    let near_the_pole = Point::new(hair * cos, hair * sin, up * radius);
    let mut along = vec![Point::new(0.0, 0.0, radius)];
    if let Pole::North = pole {
        along.push(near_the_pole);
    }
    if let Equator::Cornered = equator {
        along.push(Point::new(radius * cos, radius * sin, 0.0));
    }
    if let Pole::South = pole {
        along.push(near_the_pole);
    }
    along.push(Point::new(0.0, 0.0, -radius));

    let mut builder = SolidBuilder::new();
    let corners: Vec<_> = along
        .iter()
        .map(|&at| builder.add_vertex(at).expect("a finite point"))
        .collect();
    let meridian = Plane::from_point_and_normal(Point::origin(), Vector::new(-sin, cos, 0.0))
        .expect("a normal with a length");
    // The seam's half of the meridian circle holds both poles on its rim, so
    // it cannot tell the edge between the pole and the hair from the rest of
    // the circle; the cap past half way to that pole can.
    let the_seams_side =
        HalfSpace::through_a_point_facing(Point::origin(), Vector::new(cos, sin, 0.0))
            .expect("a normal with a length");
    let the_poles_cap = HalfSpace::through_a_point_facing(
        Point::new(0.0, 0.0, up * radius / 2.0),
        Vector::new(0.0, 0.0, up),
    )
    .expect("a normal with a length");
    let mut downs = Vec::new();
    let mut ups = Vec::new();
    for pair in along.windows(2) {
        let picker = if pair[0].z() == pair[1].z() {
            the_poles_cap
        } else {
            the_seams_side
        };
        let at = downs.len();
        let edge = builder.add_edge(
            corners[at],
            corners[at + 1],
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(meridian),
                picked_by: [Picker::InFrontOf(picker)].into_iter().collect(),
            },
        )?;
        downs.push(builder.add_coedge(edge, Orientation::Forward)?);
        ups.push(builder.add_coedge(edge, Orientation::Reversed)?);
    }
    let south = *corners.last().expect("the south pole");
    let mut steps: Vec<DraftStep> = downs.iter().map(|&down| DraftStep::Along(down)).collect();
    steps.push(DraftStep::Pole(south));
    steps.extend(ups.iter().rev().map(|&up| DraftStep::Along(up)));
    steps.push(DraftStep::Pole(corners[0]));
    // The steps run down the edges, over the south pole, back up the edges in
    // turn, over the north pole: the second edge down is at 1, and the last
    // and the one before it up are at `edges + 1` and `edges + 2`.
    let edges = downs.len();
    let first = match (set_off, pole) {
        (SetOff::Down | SetOff::FromTheHair, Pole::South) | (SetOff::Down, Pole::North) => 1,
        (SetOff::FromTheFarPole, Pole::South) | (SetOff::FromTheNearPole, Pole::North) => 0,
        (SetOff::Up | SetOff::FromTheFarPole, Pole::North)
        | (SetOff::FromTheNearPole, Pole::South) => edges + 1,
        (SetOff::Up, Pole::South) | (SetOff::FromTheHair, Pole::North) => edges + 2,
    };
    steps.rotate_left(first);
    let wire = builder.add_wire(&steps)?;
    let face = builder.add_face(
        Surface::Sphere(Sphere::new(Point::origin(), radius).expect("a radius")),
        &[wire],
    )?;
    builder.add_shell(&[(face, Orientation::Forward)])?;
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = builder.build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())?;
    Ok(BuiltBall {
        model,
        solid,
        radius,
        what: format!("{degrees} degrees, {pole:?}, {set_off:?}"),
    })
}

struct BuiltBall {
    model: Model,
    solid: SolidKey,
    radius: f64,
    what: String,
}

impl BuiltBall {
    /// Measured at a deflection of `share` of the radius.
    fn measures_a_ball_at(&self, share: f64) {
        let Self { radius, what, .. } = self;
        let measured = self
            .model
            .solid(self.solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(share * radius).expect("a working deflection"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("the ball at {what}: {why}"));
        let volume = 4.0 / 3.0 * PI * radius.powi(3);
        let area = 4.0 * PI * radius.powi(2);
        assert!(
            (measured.volume() - volume).abs() <= measured.volume_error(),
            "the ball at {what} measured {} where a ball is {volume}, and said it was within {}",
            measured.volume(),
            measured.volume_error()
        );
        assert!(
            (measured.surface_area() - area).abs() <= measured.area_error(),
            "the ball at {what} measured a skin of {} where a ball's is {area}, and said it was \
             within {}",
            measured.surface_area(),
            measured.area_error()
        );
    }
}
