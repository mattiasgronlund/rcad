//! The floor under a mesh follows the lean of the cut that makes the face big.
//!
//! A bar of radius one cut off by a plane leaning `lean` in x for every 1 in z
//! has a wall face whose ellipse climbs `2 · lean · radius`. Both of that wall's
//! **corners** sit on the seam and neither of them moves with the lean, so a
//! floor worked out from corners is the same number at every lean — `2` — while
//! the face it stands for grows without bound. `mesh.rs`'s
//! `the_finest_deflection_the_walk_will_take` said so in prose and this is the
//! test that says it in numbers.
//!
//! # The oracle is closed form, and it is the ellipse and not the kernel
//!
//! The top of the wall is where the leaning plane meets the far side of the
//! pipe, at `z = at_the_seam + 2 · lean · radius`, worked out here from what the
//! bar was built with. The floor is a fixed multiple of that — which multiple is
//! `mesh.rs`'s business and not this file's — so **the floor divided by the
//! reach has to be the same number at every lean**, and that is what is checked.
//! A floor read off the corners gives a ratio that falls like `1 / lean`, which
//! is the shape of failure this file is here to catch.
//!
//! Nothing here reads the floor's own formula back. The only number taken off
//! the kernel is `closest` out of the refusal it hands the caller, which is
//! public.
//!
//! # Why the refusal, and not a mesh
//!
//! A certified caller is refused rather than given a coarser mesh (§23.5), and
//! the refusal carries the floor — so asking for a deflection far below any
//! floor is how the floor is read without a private function. Every lean below
//! must refuse; a lean that meshed would mean the deflection was not below the
//! floor and the row measured nothing, which is why the count is asserted.
//!
//! A **flat**-capped bar of the same radius is the control. It has no lean to
//! follow, so its floor must sit where a bar's floor has always sat, and the
//! leaning ones must leave it far behind. On the corner charge all five were the
//! same number.

use rcad_geom::{Deflection, Isometry, Plane, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, SolidKey};

const RADIUS: f64 = 1.0;

/// Where the leaning cap crosses the seam, which is the only corner it puts on
/// the wall and the whole of what the corner charge could see.
const AT_THE_SEAM: f64 = 0.5;

/// Far below any floor any of these bars has, so every one of them refuses and
/// the refusal carries the floor.
const FINER_THAN_ANY_OF_THEM_CAN_BE_CUT: f64 = 1e-300;

/// How far the leans below span: four decades, because a ratio that holds over
/// one decade holds over a rounding.
const LEANS: [f64; 4] = [1e6, 1e7, 1e8, 1e9];

/// The floor rises in step with the lean of the cut, and not with the corners,
/// which do not move at all.
#[test]
fn the_floor_under_a_leaning_bar_follows_its_ellipse_and_not_its_corners() {
    let flat = the_floor_under(&a_bar_cut_off_flat());
    let mut ratios = Vec::new();
    for lean in LEANS {
        let floor = the_floor_under(&a_bar_cut_off_leaning(lean));
        let reaches = AT_THE_SEAM + 2.0 * lean * RADIUS;
        println!(
            "leaning {lean:e}: the ellipse reaches {reaches:e}, the floor is {floor:e}, \
             {:e} of it, and a flat bar's floor is {flat:e}",
            floor / reaches
        );
        assert!(
            floor > flat * lean / 10.0,
            "a bar leaning {lean:e} was given a floor of {floor}, which is not far enough \
             above the {flat} a bar with no lean at all gets"
        );
        ratios.push(floor / reaches);
    }

    assert_eq!(ratios.len(), LEANS.len(), "every lean has to have refused");
    let (low, high) = ratios
        .iter()
        .fold((f64::MAX, f64::MIN), |(low, high), &at| {
            (low.min(at), high.max(at))
        });
    assert!(
        high / low < 1.01,
        "the floor is a fixed share of how far the ellipse reaches, and over the four \
         leans that share ran from {low:e} to {high:e}"
    );
    assert!(
        low > 0.0,
        "a share of zero would make the check above pass on any floor at all"
    );
}

/// What the caller is told, which is the number and not a guess about the shape.
///
/// `docs/consumers.md` records `AWireDoesNotFanIntoALoopToMesh` reaching a
/// consumer's bug reports verbatim, where it reads as *your solid is malformed*.
/// A bar cut off at an angle is a sound bar however far it leans — `validate`
/// says so below — so the refusal has to be about the deflection, and it is:
/// `DeflectionBelowWhatAMeshCanReach`, carrying what was asked for and what the
/// arithmetic reaches.
#[test]
fn a_bar_leaning_too_far_to_mesh_finely_is_refused_by_the_deflection_and_not_by_its_shape() {
    let (model, bar) = a_bar_cut_off_leaning(1e9);
    model
        .validate(&a_tolerant_caller(1e9), &mut ProvenanceLog::new())
        .expect("a bar cut off at an angle is a sound bar however far it leans");

    let refusal = model
        .solid(bar)
        .expect("this model made the key")
        .mesh(
            Deflection::of(FINER_THAN_ANY_OF_THEM_CAN_BE_CUT).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("no mesh of this bar is cut that finely");
    let Error::DeflectionBelowWhatAMeshCanReach { asked, closest } = refusal else {
        panic!("a bar that cannot be meshed this finely was refused with {refusal:?}");
    };
    assert_eq!(asked, FINER_THAN_ANY_OF_THEM_CAN_BE_CUT);
    assert!(
        closest > asked,
        "the refusal says the closest a mesh comes is {closest}, which is not further \
         off than the {asked} it was asked for"
    );
}

/// The floor this solid puts under a deflection, read off the refusal a
/// certified caller gets for asking below it.
fn the_floor_under((model, bar): &(Model, SolidKey)) -> f64 {
    let refusal = model
        .solid(*bar)
        .expect("this model made the key")
        .mesh(
            Deflection::of(FINER_THAN_ANY_OF_THEM_CAN_BE_CUT).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("a deflection this fine is under every one of these bars' floors");
    let Error::DeflectionBelowWhatAMeshCanReach { closest, .. } = refusal else {
        panic!("asking for a mesh finer than the arithmetic reaches gave {refusal:?}");
    };
    closest
}

/// A bar whose top cap leans `lean` across the axis for every 1 along it.
fn a_bar_cut_off_leaning(lean: f64) -> (Model, SolidKey) {
    let leaning = Plane::from_point_and_normal(
        Point::new(RADIUS, 0.0, AT_THE_SEAM),
        Vector::new(lean, 0.0, 1.0),
    )
    .expect("a direction with length");
    a_bar_cut_off_by(leaning, &a_tolerant_caller(lean))
}

/// The control: the same bar with both caps square to its axis.
fn a_bar_cut_off_flat() -> (Model, SolidKey) {
    let square_on = Plane::from_point_and_normal(
        Point::new(0.0, 0.0, AT_THE_SEAM),
        Vector::new(0.0, 0.0, 1.0),
    )
    .expect("the z axis is a unit normal");
    a_bar_cut_off_by(square_on, &Policy::certified())
}

fn a_bar_cut_off_by(top: Plane, policy: &Policy) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let bar = model
        .add_bar_cut_off_by(
            RADIUS,
            [flat, top],
            &Isometry::identity(),
            policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a plane that is not along the axis cuts one closed curve out of a pipe");
    (model, bar)
}

/// A caller that accepts the wobble a cap leaning this far carries when the bar
/// is **built**. Nothing here meshes under it.
fn a_tolerant_caller(lean: f64) -> Policy {
    Policy::tolerant(50.0 * lean)
}
