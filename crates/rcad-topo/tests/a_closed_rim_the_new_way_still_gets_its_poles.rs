//! A ball split down its polar axis, with the rim between the two halves
//! written the way §216 added.
//!
//! The plane `x = 0` holds the ball's polar axis, so it cuts the ball in a
//! great circle through **both** poles and leaves two lunes. Both of them are
//! on the one sphere, so the pair of faces either side of the rim between them
//! says nothing at all about where it runs, and
//! [`EdgeRun::RightRoundWhereItCrosses`] is that rim naming the plane that cut
//! it instead. §219.7 names this shape as the one a fuse reaches.
//!
//! # What the rim being written that way cost
//!
//! `mesh.rs` guarded pole insertion on
//! [`EdgeRun::RightRoundWhereItsFacesCross`] **by name**, so a rim written the
//! new way took the `else` and was walked with no pole guaranteed among its
//! points. `put_in_the_poles_a_closed_crossing_passes` says what that costs:
//! the walk "jumps half a turn of longitude with nothing to say which way".
//!
//! Measured on the base `64b83b31`, at the seven deflections below and with the
//! whole ball as the oracle: **five of the seven were refused**
//! `ACurvedFaceGoesRightRoundWithNoSeam`, and the two that answered — `1.0` and
//! `0.05` — answered because the walk happened to land on the pole by itself,
//! which is §193.3's luck and not a rule. The two that answered at all measured
//! `467.11215906329` and `521.1197805153512` against `523.5987755982989`. With
//! the poles put in, all seven answer and every one of them has a mesh point
//! **exactly** on each pole.
//!
//! The second surface handed to the pole insertion is the run's own and not the
//! two faces: the faces say `[ball, ball]`, which reads as a ball crossed with
//! itself and is walked straight past. It is the pair `rcad_geom::walk` already
//! read off the same run to draw the circle with.

use std::collections::HashMap;

use rcad_geom::{Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey};

const RADIUS: f64 = 5.0;

/// Coarse enough that the walk steps past a pole in one go, and fine enough
/// that it takes thousands of steps to go round.
const DEFLECTIONS: [f64; 7] = [1.0, 0.5, 0.2, 0.1, 0.05, 0.01, 0.005];

/// Every one of them meshes, and the mesh closes up with the shape of a ball
/// and measures the ball's closed form.
#[test]
fn a_ball_split_down_its_polar_axis_meshes_closed_and_measures_a_ball() {
    for way in [Orientation::Forward, Orientation::Reversed] {
        for room in DEFLECTIONS {
            let what = format!("the rim walked {way:?}, at a deflection of {room}");
            let (model, solid) = a_ball_split_down_its_polar_axis(way);
            meshes_closed_and_measures(&model, solid, room, &what);
        }
    }
}

/// Each pole is a point of the mesh exactly, because the rim runs through both
/// of them.
///
/// This is the assertion that fails on the base: a walk with no pole among its
/// points is what the refusals above come out of, and on the two deflections
/// the base answered at, this stood only by luck.
#[test]
fn both_poles_are_points_of_the_mesh() {
    for way in [Orientation::Forward, Orientation::Reversed] {
        for room in DEFLECTIONS {
            let what = format!("the rim walked {way:?}, at a deflection of {room}");
            let (model, solid) = a_ball_split_down_its_polar_axis(way);
            let points = mesh_of(&model, solid, room, &what).points().to_vec();
            for up in [1.0, -1.0] {
                let pole = Point::new(0.0, 0.0, up * RADIUS);
                let nearest = points
                    .iter()
                    .map(|at| (*at - pole).length())
                    .fold(f64::INFINITY, f64::min);
                assert_eq!(
                    nearest, 0.0,
                    "{what}: the nearest mesh point to the pole {pole:?} is {nearest} away"
                );
            }
        }
    }
}

/// The two lunes of the ball of `RADIUS` about the origin, with the great
/// circle between them written as a rim that names the plane `x = 0`.
///
/// Its one corner is `(0, 5, 0)`, which is a quarter turn from either pole, so
/// the walk reaches both poles part way along rather than starting at one.
fn a_ball_split_down_its_polar_axis(way: Orientation) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(25.0 * RADIUS).expect("a positive size"));
    let skin = Surface::Sphere(Sphere::new(Point::origin(), RADIUS).expect("a ball"));
    let cut = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0))
        .expect("the x axis is a unit normal");

    let mut builder = SolidBuilder::new();
    let corner = builder
        .add_vertex(Point::new(0.0, RADIUS, 0.0))
        .expect("a point on the ball and on the plane");
    let rim = builder
        .add_edge(
            corner,
            corner,
            EdgeRun::RightRoundWhereItCrosses {
                and: Surface::Plane(cut),
            },
        )
        .expect("a rim is a whole closed curve, so one corner twice is an edge");
    let east_step = DraftStep::Along(
        builder
            .add_coedge(rim, way)
            .expect("an edge of this builder"),
    );
    let west_step = DraftStep::Along(
        builder
            .add_coedge(rim, way.reversed())
            .expect("an edge of this builder"),
    );
    let east_wire = builder.add_wire(&[east_step]).expect("a closed wire");
    let west_wire = builder.add_wire(&[west_step]).expect("a closed wire");
    let east = builder
        .add_face(skin, &[east_wire])
        .expect("a face with one wire");
    let west = builder
        .add_face(skin, &[west_wire])
        .expect("a face with one wire");
    builder
        .add_shell(&[(east, Orientation::Forward), (west, Orientation::Forward)])
        .expect("two lunes close up");
    let solid = builder
        .build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a ball split in two is a solid");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a ball split in two is a sound model");
    (model, solid)
}

/// The mesh closes up with the shape of a ball, and the solid measures the
/// ball's volume under it by no more than `3δ/r` of it.
///
/// Closed means every side of every triangle is used once each way round; the
/// shape of a ball means points less sides plus triangles is two. The bound is
/// `a_half_ball_skin_with_a_round_hole.rs`'s: the shell a mesh leaves out is at
/// most the curved area times `δ`, and a whole ball's volume is that area times
/// `r/3`.
fn meshes_closed_and_measures(model: &Model, solid: SolidKey, room: f64, what: &str) {
    let view = model.solid(solid).expect("this model made the key");
    let mesh = mesh_of(model, solid, room, what);
    let mut times_used: HashMap<(u32, u32), i32> = HashMap::new();
    for triangle in mesh.triangles() {
        let [first, second, third] = triangle.corners();
        for (from, to) in [(first, second), (second, third), (third, first)] {
            *times_used.entry((from.min(to), from.max(to))).or_default() +=
                if from < to { 1 } else { -1 };
        }
    }
    for (side, balance) in &times_used {
        assert_eq!(
            *balance, 0,
            "{what}: the side {side:?} is used {balance} more times one way than the other"
        );
    }
    let points = mesh.points().len() as i64;
    assert_eq!(
        points - times_used.len() as i64 + mesh.triangles().len() as i64,
        2,
        "{what}: {points} points, {} sides, {} triangles",
        times_used.len(),
        mesh.triangles().len()
    );

    let closed_form = 4.0 * core::f64::consts::PI * RADIUS * RADIUS * RADIUS / 3.0;
    let found = view
        .measure(
            Deflection::of(room).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the split ball measures: {why:?}"))
        .volume();
    let under = (closed_form - found) / closed_form;
    assert!(
        (0.0..3.0 * room / RADIUS).contains(&under),
        "{what}: measured {found}, closed form {closed_form}, {:.4}% under",
        100.0 * under
    );
}

fn mesh_of(model: &Model, solid: SolidKey, room: f64, what: &str) -> rcad_topo::Mesh {
    model
        .solid(solid)
        .expect("this model made the key")
        .mesh(
            Deflection::of(room).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the split ball meshes: {why:?}"))
}
