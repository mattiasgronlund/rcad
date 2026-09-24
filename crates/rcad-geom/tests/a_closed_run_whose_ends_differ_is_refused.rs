//! A run that says it closes has to close, whichever way it says it.
//!
//! Two variants of [`EdgeRun`] go right round a closed curve:
//! `RightRoundWhereItsFacesCross`, and `RightRoundWhereItCrosses` which §216
//! added for a rim between two bands of the one cone. Two places tested only
//! the first of them **by name**, so a run written the second way walked
//! straight through both (`docs/decisions.md` §219.7):
//!
//! - [`NamedRun::new`] built one with two different ends and said `Ok`,
//!   although its own doc claims the check for a closed curve generally.
//! - `walk` handed back a **parabola** for one — an open, unbounded curve —
//!   where the caller had already asked `is_closed()` and been told yes.
//!
//! Both doors are public, and `EdgeRun` is `Deserialize` besides, so a file
//! states such a run too.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Error, NamedRun, Plane, Point, Surface, Vector, walk,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};

/// The 45-degree cone about `+z` with its apex at the origin, as
/// `the_seven_shapes_a_plane_cuts_a_cone_in.rs` uses it: slope one, so
/// `slope²` is exactly one and the lean below is a difference of two stored
/// squares with nothing else in it.
fn a_right_angled_cone() -> Cone {
    Cone::new(Axis::z(), 1.0).expect("a slope of one is 45 degrees")
}

/// `y + z = 4`, parallel to the cone's own side. Its two live normal
/// components are equal, so whatever they round to the lean is exactly zero
/// and the crossing is exactly a parabola rather than a hair either side of
/// one.
fn a_plane_along_a_ruling() -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 1.0, 1.0))
        .expect("a normal with a length")
}

fn a_place(at: [f64; 3]) -> PredicatePoint {
    PredicatePoint::from(at)
}

/// Site 1: the constructor's own invariant.
#[test]
fn a_closed_run_named_the_second_way_still_cannot_have_two_different_ends() {
    let and = Surface::Plane(a_plane_along_a_ruling());
    let apart = [a_place([0.0, 2.0, 2.0]), a_place([1.0, 1.875, 2.125])];

    for closed in [
        EdgeRun::RightRoundWhereItsFacesCross,
        EdgeRun::RightRoundWhereItCrosses { and },
    ] {
        assert!(
            matches!(
                NamedRun::new(closed.clone(), apart),
                Err(Error::AClosedRunDoesNotComeBackToWhereItStarted { .. })
            ),
            "a closed run with two different ends names a curve that does not exist: {closed:?}"
        );
        NamedRun::new(closed, [apart[0]; 2]).expect("one point at both ends is a closed run");
    }
}

/// And the two that do not close still take whatever ends they are given —
/// the widened check must not have swallowed an open run.
#[test]
fn an_open_run_still_takes_two_different_ends() {
    let apart = [a_place([0.0, 2.0, 2.0]), a_place([1.0, 1.875, 2.125])];

    NamedRun::new(EdgeRun::Straight, apart).expect("a straight run's ends are two places");
    NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(a_plane_along_a_ruling()),
            picked_by: smallvec::smallvec![],
        },
        apart,
    )
    .expect("a piece of a crossing runs between two places");
}

/// Site 2: `where_a_plane_cuts_a_cone`, reached through `walk`.
///
/// The §216 shape: the faces either side are both the one cone, and the run's
/// own `and` is the plane that cut it. Here that plane is parallel to a ruling,
/// so the curve is a parabola — and a parabola is not closed, so a run that
/// says it closes is a run about a curve that is not there.
///
/// A rim a fuse writes has a lean above zero by construction and never reaches
/// this, which is why the run below is built by hand rather than fused.
#[test]
fn a_closed_run_named_the_second_way_is_refused_on_a_parabola_and_not_walked() {
    let cone = a_right_angled_cone();
    let flat = a_plane_along_a_ruling();
    let on_the_bend = [a_place([0.0, 2.0, 2.0]); 2];

    let both_bands_of_the_one_cone = Surface::Cone(cone);
    let run = NamedRun::new(
        EdgeRun::RightRoundWhereItCrosses {
            and: Surface::Plane(flat),
        },
        on_the_bend,
    )
    .expect("one point at both ends is a closed run");

    let drawn = walk(
        &run,
        [&both_bands_of_the_one_cone, &both_bands_of_the_one_cone],
        Deflection::of(0.008).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    assert!(
        matches!(drawn, Err(Error::TheCrossingOfTheseTwoSurfacesIsNotClosed)),
        "a parabola is no closed curve for a closed run to go right round, got {drawn:?}"
    );
}

/// The same refusal the first-way run has always had at that site, so the two
/// now agree. The faces either side name the pair themselves here, which is
/// what that variant means.
#[test]
fn a_closed_run_named_the_first_way_is_refused_on_the_same_parabola() {
    let cone = Surface::Cone(a_right_angled_cone());
    let flat = Surface::Plane(a_plane_along_a_ruling());
    let run = NamedRun::new(
        EdgeRun::RightRoundWhereItsFacesCross,
        [a_place([0.0, 2.0, 2.0]); 2],
    )
    .expect("one point at both ends is a closed run");

    let drawn = walk(
        &run,
        [&flat, &cone],
        Deflection::of(0.008).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    assert!(
        matches!(drawn, Err(Error::TheCrossingOfTheseTwoSurfacesIsNotClosed)),
        "a parabola is no closed curve for a closed run to go right round, got {drawn:?}"
    );
}
