//! The four floors under the four leaning bars, as numbers and not as a ratio.
//!
//! `what_a_tolerant_policy_costs_a_mesh.rs` sweeps a bar whose cap leans `4e9`,
//! `1e10`, `3e10` and `1e11` and checks that each floor is the same **share** of
//! the ellipse it stands for. That is the right check for the defect that file
//! pins, and it asserts no value at all: divide every floor by two and the ratio
//! test still passes. `docs/decisions.md` §15.6 wants the numbers themselves
//! written down somewhere, and this is where.
//!
//! The floor is read off the public refusal's `closest` and off nothing else, on
//! the same four bars built the same way. Nothing here meshes: every deflection
//! asked for is far under every floor in the table, so each call comes back
//! immediately with [`Error::DeflectionBelowWhatAMeshCanReach`].
//!
//! # The four are four, and that is asserted too
//!
//! `docs/consumers.md`'s table of what the floor move cost a consumer says of
//! these four bars: *"one answer for all four"*, `4.848820560097969e-11`. That is
//! not what they give. They give four different answers spanning a factor of
//! twenty-five, because the floor now follows the ellipse the cap cuts and the
//! four ellipses are four different sizes — which is the very change that row is
//! reporting. The `4.848820560097969e-11` in it is the **unit ball's** floor; the
//! last of these four is `4.848820560134335`, the same leading digits eleven
//! decades up, which is how the two came to be confused. So this file asserts
//! that the four are distinct as well as what each one is.

use rcad_geom::{Deflection, Isometry, Plane, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, SolidKey};

/// The radius of every bar here; the seam sits at `x = RADIUS, y = 0`.
const RADIUS: f64 = 1.0;

/// Where the leaning cap crosses the seam.
const AT_THE_SEAM: f64 = 0.5;

/// The four leans, exactly those `what_a_tolerant_policy_costs_a_mesh.rs`
/// sweeps.
const LEANS: [f64; 4] = [4e9, 1e10, 3e10, 1e11];

/// What the mesher says the closest a mesh of each of those bars can be cut is.
const THE_FLOOR_UNDER_EACH_OF_THEM: [f64; 4] = [
    0.19395282244028492,
    0.484882056046163,
    1.4546461680657568,
    4.848820560134335,
];

/// The last of the four floors divided by the first.
///
/// Twenty-five is what the two ellipses are apart, and the answer is a hair
/// under it. Some of that hair is that both ellipses climb from [`AT_THE_SEAM`]
/// rather than from nothing, so `(0.5 + 2e11) / (0.5 + 8e9)` is itself short of
/// twenty-five — by rather less than the floors are, which says the rest of the
/// hair is elsewhere. This file does not claim to have taken the difference
/// apart; it pins what the division comes to.
const THE_WIDEST_CAPS_FLOOR_OVER_THE_NARROWEST_ONES: f64 = 24.9999999955;

/// Under the lowest floor in the table by six decades, so every row refuses and
/// the refusal carries the floor.
const FINER_THAN_ANY_OF_THEM_CAN_BE_CUT: f64 = 1e-7;

/// Each of the four floors, to the last digit.
#[test]
fn a_bar_whose_cap_leans_is_given_a_floor_that_climbs_with_the_ellipse_it_cuts() {
    let measured: Vec<f64> = LEANS.iter().map(|lean| the_floor_under(*lean)).collect();
    assert_eq!(measured, THE_FLOOR_UNDER_EACH_OF_THEM);
}

/// Four bars, four ellipses, four floors — and not one answer repeated.
///
/// A floor charged from the face's *corners* gave the same number at every lean,
/// because the seam corner does not move when the cap leans. This is what says
/// that is over, without leaning on a ratio to say it.
#[test]
fn the_four_leaning_bars_are_not_given_one_floor_between_them() {
    let measured: Vec<f64> = LEANS.iter().map(|lean| the_floor_under(*lean)).collect();
    for (leans, floors) in LEANS.windows(2).zip(measured.windows(2)) {
        assert!(
            floors[1] > floors[0] * 2.0,
            "leaning the cap from {:e} to {:e} took the floor from {} to {}, which is not \
             the ellipse it stands for climbing",
            leans[0],
            leans[1],
            floors[0],
            floors[1]
        );
    }
    assert_eq!(
        measured[3] / measured[0],
        THE_WIDEST_CAPS_FLOOR_OVER_THE_NARROWEST_ONES,
        "the widest lean's ellipse is twenty-five times the narrowest one's, so its \
         floor has to be twenty-five times as high"
    );
}

/// The floor this bar puts under a deflection, out of the refusal a certified
/// caller gets for asking below it.
fn the_floor_under(lean: f64) -> f64 {
    let (model, bar) = a_bar_cut_off_leaning(lean);
    let refused = model.solid(bar).mesh(
        Deflection::of(FINER_THAN_ANY_OF_THEM_CAN_BE_CUT).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    let Err(Error::DeflectionBelowWhatAMeshCanReach { asked, closest }) = refused else {
        panic!("a bar leaning {lean:e} was answered with {refused:?}");
    };
    assert_eq!(asked, FINER_THAN_ANY_OF_THEM_CAN_BE_CUT);
    closest
}

/// A bar whose top cap leans `lean` across the axis for every 1 along it, built
/// exactly as `what_a_tolerant_policy_costs_a_mesh.rs` builds it.
fn a_bar_cut_off_leaning(lean: f64) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let leaning = Plane::from_point_and_normal(
        Point::new(RADIUS, 0.0, AT_THE_SEAM),
        Vector::new(lean, 0.0, 1.0),
    )
    .expect("a direction with length");
    let bar = model
        .add_bar_cut_off_by(
            RADIUS,
            [flat, leaning],
            &Isometry::identity(),
            &a_caller_that_accepts_the_wobble_the_widest_cap_carries(),
            &mut ProvenanceLog::new(),
        )
        .expect("a leaning plane cuts an ellipse right round a pipe");
    (model, bar)
}

/// A working size that accepts the wobble the widest cap in [`LEANS`] carries
/// when the bar is **built**. Nothing here meshes under it.
fn a_caller_that_accepts_the_wobble_the_widest_cap_carries() -> Policy {
    Policy::tolerant(50.0 * 1e11)
}
