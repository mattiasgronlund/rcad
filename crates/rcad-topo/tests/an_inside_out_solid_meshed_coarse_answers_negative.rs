//! A curved solid turned inside out, meshed so coarse that the space its mesh
//! can hide is bigger than the mesh, is measured with a negative volume rather
//! than refused — and that is the right answer, not a refusal gone missing.
//!
//! [`SolidView::measure`] refuses a curved solid with
//! [`Invalid::VolumeIsNotPositive`] only when the sum over its mesh is at most
//! minus [`Measurement::volume_error`], because only then has the arithmetic
//! shown the solid encloses no space. A ball meshed to a deflection of `2` is
//! an octahedron, six points on the sphere: it holds `4/3` of the ball's
//! `4π/3` and owes an error of `20.0`, so its sum says nothing about which
//! way round the ball is. Turned inside out it sums `-4/3` and is answered.
//!
//! **Refusing every negative sum would refuse a sound solid.** A ball with a
//! cube of side `1.15` taken out of its middle holds `2.67` and passes
//! [`Model::validate`], but the same octahedron round the same cube sums
//! `4/3 - 1.15³ = -0.19`. So at that coarseness a sum below zero is what a
//! sound solid gives too, and refusing on its sign alone is a claim about the
//! solid the arithmetic has not earned. A refusal somewhere between zero and
//! minus the error would have no derivation behind it either: the bound leaves
//! the whole range open, and the hollow ball shows that range is really
//! reached, not merely allowed.
//!
//! What the inside-out ball gets back is still true of it. Its own signed
//! volume, `-4π/3`, lies within the published error of the sum, and the
//! answer is negative, as [`Measurement::volume`] allows. What refuses it is
//! [`Model::validate`], which meshes at its own deflection; and `measure`
//! itself refuses it at a deflection of `0.1`, where the error is smaller than
//! the sum.
//!
//! [`SolidView::measure`]: rcad_topo::SolidView::measure
//! [`Measurement::volume_error`]: rcad_topo::Measurement::volume_error
//! [`Measurement::volume`]: rcad_topo::Measurement::volume

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Orientation, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Measurement, Model, ProfileBuilder, Rule, SolidKey, SolidView};

/// Coarse enough that a ball of radius one is meshed as an octahedron, the
/// coarsest mesh the kernel makes of one.
const COARSE: f64 = 2.0;

/// Fine enough that the same ball's error is smaller than what it holds.
const FINE: f64 = 0.1;

const CUBE: f64 = 1.15;

#[test]
fn an_inside_out_ball_meshed_coarse_is_measured_negative_within_its_error() {
    let model = a_ball_turned_inside_out();
    let ball = model.solids().next().expect("the ball came back");

    let coarse = measured(ball, COARSE).expect("an error bigger than the sum is answered");

    assert_eq!(coarse.volume(), -4.0 / 3.0, "an octahedron turned round");
    assert!(
        coarse.volume() > -coarse.volume_error(),
        "answered only while the sum is within its error of zero: {coarse:?}"
    );
    let its_own = -4.0 * PI / 3.0;
    assert!(
        (coarse.volume() - its_own).abs() <= coarse.volume_error(),
        "the ball turned round encloses {its_own}, and {coarse:?} does not reach it"
    );
    assert_eq!(
        coarse.centroid_wobble(),
        f64::INFINITY,
        "nothing to divide the moment by"
    );

    let refused = measured(ball, FINE).expect_err("finer, the sum is past its error");
    assert!(
        matches!(refused, Error::Invalid(Invalid::VolumeIsNotPositive { volume }) if volume < 0.0),
        "at {FINE} measure gave {refused:?}"
    );
    let swept = model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect_err("validate refuses the same bytes");
    assert!(
        matches!(swept, Error::Invalid(Invalid::VolumeIsNotPositive { .. })),
        "validate gave {swept:?}"
    );
}

/// The sound solid that sums below zero at the same coarseness, so the sign of
/// the sum cannot be what refuses the inside-out ball.
#[test]
fn a_sound_ball_hollowed_by_a_cube_sums_below_zero_when_meshed_as_coarse() {
    let (model, hollowed) = a_ball_hollowed_by_a_cube();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a ball with a cube taken out of its middle is sound");
    let solid = model.solid(hollowed).expect("this model made the key");
    let true_volume = 4.0 * PI / 3.0 - CUBE * CUBE * CUBE;
    assert!(true_volume > 2.6, "the hollow ball holds {true_volume}");

    let coarse = measured(solid, COARSE).expect("a sound solid is measured");
    assert!(
        coarse.volume() < 0.0,
        "the octahedron holds less than the cube takes out, and the sum came out {coarse:?}"
    );
    assert!(
        (coarse.volume() - true_volume).abs() <= coarse.volume_error(),
        "the hollow ball holds {true_volume}, and {coarse:?} does not reach it"
    );

    let inside_out = a_ball_turned_inside_out();
    let inside_out = measured(
        inside_out.solids().next().expect("the ball came back"),
        COARSE,
    )
    .expect("answered, as the test above says");
    assert_eq!(
        coarse.volume_error(),
        inside_out.volume_error(),
        "the cube's faces owe nothing, so the two errors are the ball's alone"
    );

    let fine = measured(solid, FINE).expect("a sound solid is measured");
    assert!(
        fine.volume() > 0.0 && (fine.volume() - true_volume).abs() <= fine.volume_error(),
        "finer, the sum comes out above zero and near {true_volume}: {fine:?}"
    );
}

/// The case the error is infinite in: a ball with a small ball taken out of
/// it, meshed so coarse that the mesh may be further from the small ball than
/// its radius, so nothing bounds the space hidden under it. Turned inside out,
/// any sum at all is within that error of zero, and it is answered.
#[test]
fn an_inside_out_ball_whose_error_has_no_bound_is_measured_negative() {
    let model = a_ball_hollowed_by_a_ball_turned_inside_out();
    let solid = model
        .solids()
        .find(|solid| solid.shells().count() == 2)
        .expect("the hollow ball came back");

    let coarse = measured(solid, COARSE).expect("an infinite error is answered");

    assert_eq!(coarse.volume_error(), f64::INFINITY);
    assert!(
        coarse.volume() < 0.0,
        "two octahedra, both turned round: {coarse:?}"
    );
    assert_eq!(coarse.centroid_wobble(), f64::INFINITY);
}

fn measured(solid: SolidView<'_>, deflection: f64) -> Result<Measurement, Error> {
    solid.measure(
        Deflection::of(deflection).expect("a length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

/// A ball of radius one whose solid uses its one shell the other way round,
/// read back from bytes, the only door that lets one in.
///
/// The solid arena is the last thing a model writes, and a ball's solid holds
/// one shell use: its shell's key, its placement, its orientation, then the
/// slot's version. So the orientation is the second byte from the end, `0` for
/// forward; the read-back shell is asserted reversed, so a layout that moved
/// fails here.
fn a_ball_turned_inside_out() -> Model {
    let mut model = a_model();
    model
        .add_ball(1.0, &Isometry::identity())
        .expect("a ball of radius one");
    let mut bytes = postcard::to_allocvec(&model).expect("a model writes out");
    let at = bytes.len() - 2;
    assert_eq!(bytes[at..], [0, 1], "a forward shell use, then version one");
    bytes[at] = 1;

    let read_back: Model = postcard::from_bytes(&bytes).expect("and reads back");
    let ball = read_back.solids().next().expect("the ball came back");
    assert!(
        ball.shells()
            .all(|shell| shell.orientation() == Orientation::Reversed),
        "the ball's shell is used reversed"
    );
    read_back
}

/// A ball of radius one with a cube of side [`CUBE`] taken out of its middle.
/// The cube's corners are `0.996` out, inside the ball.
fn a_ball_hollowed_by_a_cube() -> (Model, SolidKey) {
    let policy = Policy::certified();
    let mut model = a_model();
    let ball = model
        .add_ball(1.0, &Isometry::identity())
        .expect("a ball of radius one");
    let half = CUBE / 2.0;
    let square = ProfileBuilder::new(vec![
        [-half, -half],
        [half, -half],
        [half, half],
        [-half, half],
    ])
    .build(&policy, &mut ProvenanceLog::new())
    .expect("a square");
    let cube = model
        .add_extrusion(
            &square,
            CUBE,
            &Isometry::translation(Vector::new(0.0, 0.0, -half)).expect("a shift"),
        )
        .expect("a cube round the middle of the ball");
    let fused = model
        .fuse(
            &[ball],
            &[cube],
            Rule::Difference,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a ball with a cube taken out of it");
    let [hollowed] = fused.solids()[..] else {
        panic!("one solid, got {:?}", fused.solids());
    };
    (model, hollowed)
}

/// A ball of radius one with a ball of radius a tenth taken out of its middle,
/// read back with both of its shells used the other way round.
///
/// The fused solid is the last one written, and holds two shell uses of
/// [`ONE_SHELL_USE`] bytes each, so their orientations are the second byte
/// from the end and that many before it.
fn a_ball_hollowed_by_a_ball_turned_inside_out() -> Model {
    /// A two-byte key, seven numbers of placement, and the orientation.
    const ONE_SHELL_USE: usize = 2 + 7 * 8 + 1;

    let policy = Policy::certified();
    let mut model = a_model();
    let ball = model
        .add_ball(1.0, &Isometry::identity())
        .expect("a ball of radius one");
    let hollow = model
        .add_ball(0.1, &Isometry::identity())
        .expect("a ball of radius a tenth");
    model
        .fuse(
            &[ball],
            &[hollow],
            Rule::Difference,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a ball with a hollow in it");

    let mut bytes = postcard::to_allocvec(&model).expect("a model writes out");
    for at in [bytes.len() - 2, bytes.len() - 2 - ONE_SHELL_USE] {
        assert_eq!(bytes[at], 0, "a forward shell use at {at}");
        bytes[at] = 1;
    }
    let read_back: Model = postcard::from_bytes(&bytes).expect("and reads back");
    let hollowed = read_back
        .solids()
        .find(|solid| solid.shells().count() == 2)
        .expect("the hollow ball came back");
    assert!(
        hollowed
            .shells()
            .all(|shell| shell.orientation() == Orientation::Reversed),
        "both of the hollow ball's shells are used reversed"
    );
    read_back
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}
