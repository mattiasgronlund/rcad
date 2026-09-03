//! `EdgeView::run` hands back the run where the solid actually is, not where it
//! is stored.
//!
//! A run is un-located geometry: an [`EdgeRun::PartOfWhereItCrosses`] carries a
//! second [`Surface`] and a list of [`Picker`]s, all written in the frame the
//! face is stored in. A caller reading them through a view that has a placement
//! composed onto it — an importer describing an occurrence, a writer emitting a
//! `SURFACE_CURVE` — would be handed a plane and a half-space in the wrong
//! place, and every point it tested against them would come back on the wrong
//! side.
//!
//! # Why none of this is a tautology
//!
//! The two location-free variants are the trap. [`EdgeRun::Straight`] and
//! [`EdgeRun::RightRoundWhereItsFacesCross`] name the crossing of the two faces
//! either side, so moving the faces moves the crossing and the run comes back
//! byte for byte the same. A cube's twelve edges are all straight, so a suite
//! that only looked at a cube would pass with an accessor that ignored the
//! placement entirely. That is the control row here, and it is the reason every
//! other row is about a **ball**: a ball's seam is the one shape this kernel
//! builds whose run carries geometry of its own.
//!
//! # The oracle is exact, and picked so it can be
//!
//! A movement with no turn takes a half-space `(n, d)` to `(n, d + n·t)` with
//! the normal untouched, so a ball shifted by whole numbers has a seam whose
//! plane and picker are whole numbers too, and the comparison is `assert_eq!`
//! rather than a tolerance.
//!
//! A turn cannot promise that — `fl(Rn)` is not `Rn` — so the turned row does
//! not compare four numbers. It uses the one thing a turn about the z axis
//! *is* exact about: the third column of the rotation is `(0, 0, 1)` number for
//! number, so a ball's poles at `(0, 0, ±r)` do not move and the turned
//! meridian's normal keeps an exactly zero z component. The seam plane
//! therefore still holds both poles with an exact zero, whatever the turn did
//! to the other two components.

use rcad_geom::{Axis, EdgeRun, HalfSpace, Isometry, Picker, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Decision, Point as PredicatePoint};
use rcad_topo::{Model, SolidKey};

const RADIUS: f64 = 2.0;

fn a_ball_placed(at: &Isometry) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ball = model.add_ball(RADIUS, at).expect("a ball of radius two");
    (model, ball)
}

/// The runs of every edge of `solid`, read through the placement the solid is
/// used at.
fn runs_of(model: &Model, solid: SolidKey) -> Vec<EdgeRun> {
    model
        .solid(solid)
        .faces()
        .flat_map(|face| {
            face.wires()
                .flat_map(|wire| wire.coedges().map(|coedge| coedge.edge().run()))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The one run of `solid` that carries geometry of its own.
fn the_seam_of(model: &Model, solid: SolidKey) -> EdgeRun {
    let mut picked: Vec<EdgeRun> = runs_of(model, solid)
        .into_iter()
        .filter(|run| matches!(run, EdgeRun::PartOfWhereItCrosses { .. }))
        .collect();
    picked.dedup();
    assert_eq!(
        picked.len(),
        1,
        "a ball has exactly one seam, and it is the whole point of this suite \
         that it is a picked run: {picked:?}"
    );
    picked.pop().expect("just counted one")
}

#[test]
fn a_shifted_balls_seam_names_the_meridian_where_the_ball_actually_is() {
    let shift = Vector::new(3.0, 5.0, 7.0);
    let (model, ball) = a_ball_placed(&Isometry::translation(shift).expect("a finite shift"));

    let meridian =
        Plane::from_point_and_normal(Point::new(0.0, 5.0, 0.0), Vector::new(0.0, 1.0, 0.0))
            .expect("the y axis is a unit normal");
    let facing_x =
        HalfSpace::through_a_point_facing(Point::new(3.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
            .expect("the x axis has length");

    assert_eq!(
        the_seam_of(&model, ball),
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(meridian),
            picked_by: [Picker::InFrontOf(facing_x)].into_iter().collect(),
        },
        "the seam of a ball five millimetres up the y axis lies in the plane \
         y = 5, and is picked out by the half-space x >= 3"
    );
}

/// The headline: what the accessor hands back for a placed solid is not what is
/// stored, and the stored answer would have been wrong.
#[test]
fn a_ball_away_from_the_origin_has_a_different_seam_from_one_at_the_origin() {
    let shift = Vector::new(3.0, 5.0, 7.0);
    let (moved, there) = a_ball_placed(&Isometry::translation(shift).expect("a finite shift"));
    let (still, here) = a_ball_placed(&Isometry::identity());

    let there = the_seam_of(&moved, there);
    let here = the_seam_of(&still, here);
    assert_ne!(
        there, here,
        "the two balls are the same description at two placements, so a run \
         read without its placement would have come back identical"
    );

    let EdgeRun::PartOfWhereItCrosses { and, .. } = &here else {
        unreachable!("the_seam_of only hands back picked runs")
    };
    assert_eq!(
        and,
        &Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("the y axis is a unit normal")
        ),
        "the unmoved ball still reads the meridian it was described with"
    );
}

#[test]
fn a_turned_balls_seam_plane_still_holds_both_poles_exactly() {
    let turn = Isometry::rotation(&Axis::z(), 0.7).expect("a finite angle about the z axis");
    let (model, ball) = a_ball_placed(&turn);

    let EdgeRun::PartOfWhereItCrosses { and, .. } = the_seam_of(&model, ball) else {
        unreachable!("the_seam_of only hands back picked runs")
    };
    let Surface::Plane(meridian) = and else {
        panic!("a ball's seam runs along a plane, and came back {and:?}")
    };

    assert_ne!(
        meridian.normal(),
        Vector::new(0.0, 1.0, 0.0),
        "a turn of 0.7 radians moves the meridian, or this row is asking nothing"
    );
    for up in [RADIUS, -RADIUS] {
        assert_eq!(
            meridian.side_of_point(PredicatePoint::from([0.0, 0.0, up])),
            Decision::Zero,
            "a turn about the z axis leaves the poles where they were, and the \
             turned meridian still runs between them"
        );
    }
}

/// The control. Every edge of a cube is straight, and a straight run names the
/// crossing of the two faces either side rather than any geometry of its own —
/// so moving the cube must leave all twelve runs alone.
#[test]
fn a_cube_reads_the_same_runs_wherever_it_is_put() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cube = model
        .add_cube(
            2.0,
            &rcad_predicates::Policy::certified(),
            &mut rcad_predicates::ProvenanceLog::new(),
        )
        .expect("a cube of side two");

    let before = runs_of(&model, cube);
    assert_eq!(before.len(), 24, "twelve edges, each walked by two coedges");
    assert!(
        before.iter().all(|run| *run == EdgeRun::Straight),
        "a cube is bounded by straight edges"
    );

    model.place_solid(
        cube,
        &Isometry::rotation(&Axis::z(), 0.7)
            .expect("a finite angle")
            .compose(&Isometry::translation(Vector::new(3.0, 5.0, 7.0)).expect("a finite shift")),
    );

    assert_eq!(runs_of(&model, cube), before);
}
