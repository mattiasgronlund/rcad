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
//!
//! An **eighth** of the ball is built the same way at the end of this file, and
//! it is here for its flat faces rather than for its measure: each is a pie
//! slice, a sector of a quarter turn, where a wedge's flat faces are half-discs.
//! A half-disc's arc has both ends on every wall; a pie slice's arc has one end
//! on each, which is the case §169 taught the route to read.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Sphere,
    Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, DraftStep, Model, SolidBuilder, SolidKey};

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

/// Inside and outside of a wedge of a ball are not in doubt anywhere away from
/// its skin, so every point asked must come back one or the other, and the
/// right one (`docs/decisions.md` §167). Until §167 every one of them came back
/// `CannotFollowAFacesTrimming`: the sphere face and both half-discs are bounded
/// by *half* of where their surfaces cross, and nothing could say which half.
///
/// The oracle is where the point is: inside the unit ball, and swept from the
/// wedge's first half-plane by less than the wedge's own turn. Every point is
/// held well clear of the skin, so reading that off `atan2` decides nothing near
/// a boundary.
#[test]
fn a_wedge_of_a_ball_says_which_points_are_inside_it_wherever_it_sits() {
    wedges_say_which_points_are_inside_them(&Isometry::identity());
}

/// Moved or turned, a wedge is the same wedge, so it must give the same answers
/// about the same points moved with it (`docs/decisions.md` §168). Until §168 a
/// turn or a shift made every one of them `CannotFollowAFacesTrimming`: the
/// movement rounds the arc's two ends a hair off the plane of the half-space
/// that picks it, and §167's route asked for them to be on it exactly.
///
/// The shift is by thirds and tenths, which no power of two holds, so the
/// wall's offset and the corners round too; the turn is about an axis that
/// misses the ball's centre and lies along none of the coordinate axes.
#[test]
fn a_moved_or_turned_wedge_of_a_ball_says_which_points_are_inside_it() {
    let moved = Isometry::translation(Vector::new(0.1, -1.0 / 3.0, 2.7)).expect("a finite shift");
    let turned = Isometry::rotation(
        &Axis::new(Point::new(0.3, -0.2, 0.5), Vector::new(1.0, 2.0, 3.0))
            .expect("an axis with a length"),
        0.7,
    )
    .expect("a finite angle");
    for movement in [moved, turned, moved.compose(&turned)] {
        wedges_say_which_points_are_inside_them(&movement);
    }
}

/// Every wedge [`a_wedge_of_a_ball`] is asked about here, placed by `movement`,
/// answers ten points moved with it, and gets each one right with nothing
/// written to the log.
fn wedges_say_which_points_are_inside_them(movement: &Isometry) {
    let asked = [
        [0.35, 0.2, 0.0],
        [0.5, 0.2, 0.3],
        [-0.2, -0.35, 0.0],
        [0.35, -0.15, 0.1],
        [0.1, 0.6, -0.5],
        [-0.4, 0.3, 0.2],
        [0.2, -0.5, -0.4],
        [-0.6, 0.1, -0.3],
        [0.9, 0.9, 0.5],
        [-0.1, -0.2, 1.3],
    ];
    let policy = Policy::certified();
    let mut inside = 0;
    let mut outside = 0;
    for (between, fraction) in [
        ([[1.0, 0.0], [0.0, 1.0]], 0.25),
        ([[1.0, -1.0], [1.0, 1.0]], 0.25),
        ([[-1.0, 1.0], [-1.0, -1.0]], 0.25),
        ([[0.0, -1.0], [-1.0, 0.0]], 0.75),
    ] {
        let mut model = Model::new(rcad_geom::Scale::millimetres(100.0).expect("a working size"));
        let wedge = a_wedge_of_a_ball(&mut model, between, 0, &policy);
        model
            .place_solid(wedge, movement)
            .expect("this model made the key");
        let solid = model.solid(wedge).expect("this model made the key");
        for [x, y, z] in asked {
            let expected = where_it_is(between[0], fraction, [x, y, z]);
            let mut log = ProvenanceLog::new();
            let answer = solid.classify_point(
                movement.apply_to_point(Point::new(x, y, z)),
                &policy,
                &mut log,
            );
            assert_eq!(
                answer,
                expected,
                "the wedge between {between:?}, placed by {movement:?}, asked about {:?}",
                [x, y, z]
            );
            assert!(
                log.is_empty(),
                "a certified answer about an exact point records nothing"
            );
            match expected {
                Containment::Inside => inside += 1,
                Containment::Outside => outside += 1,
                Containment::OnBoundary
                | Containment::Undecidable { .. }
                | Containment::CannotFollowAFacesTrimming { .. }
                | Containment::CannotFireARayAtAFacesSurface { .. }
                | _ => unreachable!("the oracle only says inside or outside, not {expected:?}"),
            }
        }
    }
    eprintln!("{inside} points inside a wedge and {outside} outside, every one answered");
    assert!(
        inside >= 10 && outside >= 10,
        "the points should land on both sides often: {inside} inside, {outside} outside"
    );
}

/// A pie slice is a sector of a disc under a half turn, and until §169 an
/// eighth of a ball answered `CannotFollowAFacesTrimming` about every point.
/// Each of its three flat faces is one, and so is every arc of its skin: the
/// arc has one end on each of the two walls that meet at the slice's point, and
/// §167's step 2 could only read an arc with *both* ends on the cut it was being
/// asked about.
///
/// The oracle is the octant: inside the unit ball with all three coordinates
/// positive. Every point is held well clear of all four faces, so nothing here
/// turns on a sign near a boundary.
#[test]
fn a_pie_slice_of_a_ball_says_which_points_are_inside_it_wherever_it_sits() {
    let moved = Isometry::translation(Vector::new(0.1, -1.0 / 3.0, 2.7)).expect("a finite shift");
    let turned = Isometry::rotation(
        &Axis::new(Point::new(0.3, -0.2, 0.5), Vector::new(1.0, 2.0, 3.0))
            .expect("an axis with a length"),
        0.7,
    )
    .expect("a finite angle");
    let asked = [
        [0.3, 0.3, 0.3],
        [0.5, 0.2, 0.1],
        [0.15, 0.6, 0.25],
        [0.2, 0.2, 0.7],
        [-0.3, 0.3, 0.3],
        [0.3, -0.3, 0.3],
        [0.3, 0.3, -0.3],
        [0.7, 0.7, 0.7],
        [0.2, 0.2, 1.3],
        [-0.5, -0.5, -0.5],
    ];
    let policy = Policy::certified();
    let mut inside = 0;
    let mut outside = 0;
    for movement in [Isometry::identity(), moved, turned, moved.compose(&turned)] {
        let mut model = Model::new(rcad_geom::Scale::millimetres(100.0).expect("a working size"));
        let eighth = an_eighth_of_a_ball(&mut model, &policy);
        model
            .place_solid(eighth, &movement)
            .expect("this model made the key");
        let solid = model.solid(eighth).expect("this model made the key");
        for [x, y, z] in asked {
            let expected = where_it_is_in_an_eighth([x, y, z]);
            let mut log = ProvenanceLog::new();
            let answer = solid.classify_point(
                movement.apply_to_point(Point::new(x, y, z)),
                &policy,
                &mut log,
            );
            assert_eq!(
                answer,
                expected,
                "an eighth of a ball, placed by {movement:?}, asked about {:?}",
                [x, y, z]
            );
            assert!(
                log.is_empty(),
                "a certified answer about an exact point records nothing"
            );
            match expected {
                Containment::Inside => inside += 1,
                Containment::Outside => outside += 1,
                Containment::OnBoundary
                | Containment::Undecidable { .. }
                | Containment::CannotFollowAFacesTrimming { .. }
                | Containment::CannotFireARayAtAFacesSurface { .. }
                | _ => unreachable!("the oracle only says inside or outside, not {expected:?}"),
            }
        }
    }
    eprintln!(
        "{inside} points inside an eighth of a ball and {outside} outside, every one answered"
    );
    assert!(
        inside >= 4 && outside >= 4,
        "the points should land on both sides often: {inside} inside, {outside} outside"
    );
}

/// Inside or outside the eighth of the unit ball where every coordinate is
/// positive, for a point well clear of all four of its faces.
fn where_it_is_in_an_eighth([x, y, z]: [f64; 3]) -> Containment {
    let clear = 0.05;
    let reach = (x * x + y * y + z * z).sqrt();
    assert!(
        (reach - 1.0).abs() > clear && [x, y, z].iter().all(|along| along.abs() > clear),
        "{:?} is too near a face of the eighth",
        [x, y, z]
    );
    if reach < 1.0 && x > 0.0 && y > 0.0 && z > 0.0 {
        Containment::Inside
    } else {
        Containment::Outside
    }
}

/// Inside or outside a wedge of the unit ball swept anticlockwise from `first`
/// by `fraction` of a turn, for a point well clear of its skin.
fn where_it_is(first: [f64; 2], fraction: f64, [x, y, z]: [f64; 3]) -> Containment {
    let clear = 0.05;
    let reach = (x * x + y * y + z * z).sqrt();
    assert!(
        (reach - 1.0).abs() > clear,
        "{:?} is too near the sphere",
        [x, y, z]
    );
    if reach > 1.0 {
        return Containment::Outside;
    }
    let turn = 2.0 * PI;
    let swept = (y.atan2(x) - first[1].atan2(first[0])).rem_euclid(turn);
    let sweep = fraction * turn;
    assert!(
        swept.min(turn - swept) > clear && (swept - sweep).abs() > clear,
        "{:?} is too near a half-disc",
        [x, y, z]
    );
    if swept < sweep {
        Containment::Inside
    } else {
        Containment::Outside
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

/// The eighth of the unit ball where every coordinate is positive.
///
/// Four faces: a triangle of the sphere, and a quarter-disc on each of the three
/// coordinate planes. Each quarter-disc is a **pie slice** — two radii out from
/// the centre and an arc between their far ends — where a wedge's flat faces are
/// half-discs, whose one straight side runs right through the centre.
///
/// Each wall's normal points away from the material, so the wire on it walks the
/// slice with the material on its left: out along one radius, back along the
/// other, and round the arc.
fn an_eighth_of_a_ball(model: &mut Model, policy: &Policy) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let corner = |builder: &mut SolidBuilder, at: [f64; 3]| {
        builder
            .add_vertex(Point::new(at[0], at[1], at[2]))
            .expect("a corner is a point")
    };
    let centre = corner(&mut builder, [0.0, 0.0, 0.0]);
    let along_x = corner(&mut builder, [1.0, 0.0, 0.0]);
    let along_y = corner(&mut builder, [0.0, 1.0, 0.0]);
    let along_z = corner(&mut builder, [0.0, 0.0, 1.0]);

    let wall = |facing: [f64; 3]| {
        Plane::from_point_and_normal(
            Point::origin(),
            Vector::new(facing[0], facing[1], facing[2]),
        )
        .expect("a coordinate direction has length")
    };
    let [off_x, off_y, off_z] = [
        wall([-1.0, 0.0, 0.0]),
        wall([0.0, -1.0, 0.0]),
        wall([0.0, 0.0, -1.0]),
    ];

    let radius = |builder: &mut SolidBuilder, to| {
        builder
            .add_edge(centre, to, EdgeRun::Straight)
            .expect("a radius out of the centre")
    };
    let out_along_x = radius(&mut builder, along_x);
    let out_along_y = radius(&mut builder, along_y);
    let out_along_z = radius(&mut builder, along_z);

    // A quarter of a circle is the part of it in front of the plane through its
    // two ends, and those ends are two of `(1, 0, 0)`, `(0, 1, 0)` and
    // `(0, 0, 1)`: a normal of ones and zeroes and an offset of one read exactly
    // `Zero` at both, which is what the route asks of a picker.
    let quarter = |builder: &mut SolidBuilder, from, to, on: Plane, picking: [f64; 3]| {
        let run = EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(on),
            picked_by: smallvec::smallvec![Picker::from(
                HalfSpace::new(Vector::new(picking[0], picking[1], picking[2]), 1.0)
                    .expect("a direction with a length")
            )],
        };
        builder
            .add_edge(from, to, run)
            .expect("a quarter of a great circle")
    };
    let x_to_y = quarter(&mut builder, along_x, along_y, off_z, [1.0, 1.0, 0.0]);
    let y_to_z = quarter(&mut builder, along_y, along_z, off_x, [0.0, 1.0, 1.0]);
    let z_to_x = quarter(&mut builder, along_z, along_x, off_y, [1.0, 0.0, 1.0]);

    let along = |builder: &mut SolidBuilder, edge, orientation| {
        DraftStep::Along(
            builder
                .add_coedge(edge, orientation)
                .expect("an edge of this builder"),
        )
    };
    let skin_steps = [
        along(&mut builder, x_to_y, Orientation::Forward),
        along(&mut builder, y_to_z, Orientation::Forward),
        along(&mut builder, z_to_x, Orientation::Forward),
    ];
    let off_z_steps = [
        along(&mut builder, out_along_x, Orientation::Reversed),
        along(&mut builder, out_along_y, Orientation::Forward),
        along(&mut builder, x_to_y, Orientation::Reversed),
    ];
    let off_y_steps = [
        along(&mut builder, out_along_x, Orientation::Forward),
        along(&mut builder, z_to_x, Orientation::Reversed),
        along(&mut builder, out_along_z, Orientation::Reversed),
    ];
    let off_x_steps = [
        along(&mut builder, out_along_y, Orientation::Reversed),
        along(&mut builder, out_along_z, Orientation::Forward),
        along(&mut builder, y_to_z, Orientation::Reversed),
    ];
    let mut faces = Vec::with_capacity(4);
    for (surface, steps) in [
        (
            Surface::Sphere(Sphere::new(Point::origin(), 1.0).expect("a unit ball")),
            skin_steps,
        ),
        (Surface::Plane(off_z), off_z_steps),
        (Surface::Plane(off_y), off_y_steps),
        (Surface::Plane(off_x), off_x_steps),
    ] {
        let wire = builder.add_wire(&steps).expect("a closed ring of steps");
        faces.push((
            builder
                .add_face(surface, &[wire])
                .expect("a face with one wire"),
            Orientation::Forward,
        ));
    }
    builder.add_shell(&faces).expect("four faces close up");
    builder
        .build(model, policy, &mut ProvenanceLog::new())
        .expect("an eighth of a ball is a solid")
}
