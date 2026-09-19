//! A quarter of a unit ball, built face by face, measures a quarter of a ball
//! (`docs/decisions.md` §166).
//!
//! The quarter is three faces: a lune of the sphere between two half-meridians,
//! and a half-disc on each of the two cutting planes. Its sphere face is bounded
//! the way the kernel's own ball is — down one meridian, over the south pole,
//! up the other meridian, over the north pole — except that the two meridians
//! are different ones. It passed `Model::validate` and measured `5.209`, where
//! the truth is `π/3`: the mesher crossed each pole as if the face went right
//! round the ball, so the lune came out a whole turn wider than it is.
//!
//! The oracle is a closed form. A wedge spanning a fraction `f` of a turn has
//! volume `f · 4π/3` and area `f · 4π` of sphere plus two half-discs of `π/2`,
//! and the mesh must land within its own published error of both. The three
//! quarters, built the same way, measured right before the fix and are the
//! control that the builder itself is not what is wrong.

use core::f64::consts::PI;

use rcad_geom::{
    Deflection, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Sphere, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey};

#[test]
fn a_quarter_of_a_ball_measures_a_quarter_of_a_ball_wherever_it_sits() {
    for (between, fraction) in [
        ([[1.0, 0.0], [0.0, 1.0]], 0.25),
        ([[1.0, -1.0], [1.0, 1.0]], 0.25),
        ([[-1.0, 1.0], [-1.0, -1.0]], 0.25),
        ([[0.0, -1.0], [-1.0, 0.0]], 0.75),
    ] {
        a_wedge_measures_what_it_is(between, 0, fraction);
    }
}

/// Which step of the sphere face's wire comes first says nothing about the
/// face, so it must say nothing about the measure. Opened going **up** a
/// meridian, the walk meets the north pole first, and the pole side there runs
/// against the longitude: a sign that alternated from the first pole crossed
/// would send it the long way round.
#[test]
fn a_quarter_of_a_ball_measures_the_same_whichever_step_its_skin_starts_at() {
    for starting_at in 0..4 {
        a_wedge_measures_what_it_is([[1.0, 0.0], [0.0, 1.0]], starting_at, 0.25);
    }
}

fn a_wedge_measures_what_it_is(between: [[f64; 2]; 2], starting_at: usize, fraction: f64) {
    let policy = Policy::certified();
    let mut model = Model::new(rcad_geom::Scale::millimetres(100.0).expect("a working size"));
    let wedge = a_wedge_of_a_ball(&mut model, between, starting_at, &policy);
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("a wedge of a ball is a sound solid");

    let measured = model
        .solid(wedge)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a hundredth of a millimetre is a length"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a wedge of a ball measures");
    let volume = fraction * 4.0 * PI / 3.0;
    let area = fraction * 4.0 * PI + PI;
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "the wedge between {between:?}, skin starting at step {starting_at}, should hold \
         {volume}, measured {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
    assert!(
        (measured.surface_area() - area).abs() <= measured.area_error(),
        "the wedge between {between:?}, skin starting at step {starting_at}, should have \
         {area} of skin, measured {} give or take {}",
        measured.surface_area(),
        measured.area_error()
    );
}

/// The part of a unit ball between two half-planes that hang off the `z` axis,
/// swept anticlockwise seen from above from `between[0]` to `between[1]`, each
/// named by the way it points away from the axis.
///
/// The sphere face's wire is written starting from its step `starting_at`: `0`
/// goes down the first meridian, `1` opens on the south pole, `2` goes up the
/// second meridian and `3` opens on the north pole.
///
/// A copy of the builder in `fuses_that_must_agree.rs`, widened to take the
/// diagonals and a starting step: a test file cannot reach another's functions.
fn a_wedge_of_a_ball(
    model: &mut Model,
    between: [[f64; 2]; 2],
    starting_at: usize,
    policy: &Policy,
) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let at = |builder: &mut SolidBuilder, z: f64| {
        builder
            .add_vertex(Point::new(0.0, 0.0, z))
            .expect("a pole is a point")
    };
    let north = at(&mut builder, 1.0);
    let south = at(&mut builder, -1.0);

    // Out of the wedge across the first half-plane is clockwise of it, and out
    // across the second is anticlockwise of it.
    let [first, second] = between;
    let outward = [[first[1], -first[0]], [-second[1], second[0]]];
    let wall = |facing: [f64; 2]| {
        Plane::from_point_and_normal(Point::origin(), Vector::new(facing[0], facing[1], 0.0))
            .expect("a direction off the axis has length")
    };
    let mut meridians = Vec::with_capacity(2);
    for (way, facing) in between.into_iter().zip(outward) {
        let run = EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(wall(facing)),
            picked_by: smallvec::smallvec![Picker::from(
                HalfSpace::through_a_point_facing(
                    Point::origin(),
                    Vector::new(way[0], way[1], 0.0)
                )
                .expect("a direction off the axis has length")
            )],
        };
        meridians.push(
            builder
                .add_edge(north, south, run)
                .expect("a half-meridian between the poles"),
        );
    }
    let axis = builder
        .add_edge(north, south, EdgeRun::Straight)
        .expect("the axis between the poles");

    let along = |builder: &mut SolidBuilder, edge, orientation| {
        DraftStep::Along(
            builder
                .add_coedge(edge, orientation)
                .expect("an edge of this builder"),
        )
    };
    let mut skin_steps = [
        along(&mut builder, meridians[0], Orientation::Forward),
        DraftStep::Pole(south),
        along(&mut builder, meridians[1], Orientation::Reversed),
        DraftStep::Pole(north),
    ];
    skin_steps.rotate_left(starting_at);
    let first_wall_steps = [
        along(&mut builder, meridians[0], Orientation::Reversed),
        along(&mut builder, axis, Orientation::Forward),
    ];
    let second_wall_steps = [
        along(&mut builder, meridians[1], Orientation::Forward),
        along(&mut builder, axis, Orientation::Reversed),
    ];
    let mut faces = Vec::with_capacity(3);
    for (surface, steps) in [
        (
            Surface::Sphere(Sphere::new(Point::origin(), 1.0).expect("a unit ball")),
            &skin_steps[..],
        ),
        (Surface::Plane(wall(outward[0])), &first_wall_steps[..]),
        (Surface::Plane(wall(outward[1])), &second_wall_steps[..]),
    ] {
        let wire = builder.add_wire(steps).expect("a closed ring of steps");
        faces.push((
            builder
                .add_face(surface, &[wire])
                .expect("a face with one wire"),
            Orientation::Forward,
        ));
    }
    builder.add_shell(&faces).expect("three faces close up");
    builder
        .build(model, policy, &mut ProvenanceLog::new())
        .expect("a wedge of a ball is a solid")
}
