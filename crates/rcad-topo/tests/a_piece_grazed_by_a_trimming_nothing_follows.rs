//! Probe sweep for Q1760, not committed.

use rcad_geom::{
    Axis, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Sphere, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, Rule, SolidBuilder, SolidKey};

const RADIUS: f64 = 5.0;

#[derive(Clone, Copy, Debug)]
enum Other {
    Cube(f64, [f64; 3]),
    Ball(f64, [f64; 3]),
    HalfBallAlone,
    CopyOfTheHoled,
    HoledTurned(f64),
    HoledFloorFirst,
}

#[test]
fn probe_q1760_sweep() {
    let others = [
        Other::Cube(12.0, [-6.0, -6.0, 3.0]),
        Other::Cube(1.0, [-0.5, 1.0, 4.0]),
        Other::Cube(2.0, [-1.0, 0.0, 4.0]),
        Other::Cube(2.0, [-1.0, -1.0, 5.0]),
        Other::Cube(4.0, [-2.0, -2.0, -4.0]),
        Other::Cube(2.0, [-1.0, -1.0, -1.0]),
        Other::Cube(3.0, [2.0, -1.0, -1.0]),
        Other::Cube(1.0, [-0.5, 2.5, 3.5]),
        Other::Cube(6.0, [-3.0, -3.0, 0.0]),
        Other::Cube(2.0, [-1.0, 2.0, 3.0]),
        Other::Cube(10.0, [-5.0, -5.0, 4.0]),
        Other::Cube(10.0, [-5.0, 1.0, -5.0]),
        Other::Ball(5.0, [0.0, 0.0, 0.0]),
        Other::Ball(2.0, [0.0, 1.5, 4.5]),
        Other::Ball(1.0, [0.0, 1.2649110640673518, 3.7947331922020555]),
        Other::Ball(5.0, [0.0, 0.0, 10.0]),
        Other::Ball(5.0, [0.0, 0.0, 8.0]),
        Other::Ball(3.0, [0.0, 0.0, 0.0]),
        Other::Ball(2.0, [0.0, 3.0, 4.0]),
        Other::Ball(1.0, [0.0, 0.0, 5.0]),
        Other::HalfBallAlone,
        Other::CopyOfTheHoled,
        Other::HoledTurned(0.5),
        Other::HoledTurned(core::f64::consts::PI),
        Other::HoledFloorFirst,
    ];
    let before = std::env::var_os("Q1760_BEFORE").is_some();
    for other in others {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for holed_first in [true, false] {
                let mut model = Model::new(Scale::millimetres(125.0).expect("a positive size"));
                let holed = a_half_ball(&mut model, true, false);
                let other_key = match other {
                    Other::Cube(side, corner) => {
                        let cube = model
                            .add_cube(side, &Policy::certified(), &mut ProvenanceLog::new())
                            .expect("a cube");
                        model
                            .place_solid(
                                cube,
                                &Isometry::translation(Vector::new(
                                    corner[0], corner[1], corner[2],
                                ))
                                .expect("a shift"),
                            )
                            .expect("key");
                        cube
                    }
                    Other::Ball(radius, at) => model
                        .add_ball(
                            radius,
                            &Isometry::translation(Vector::new(at[0], at[1], at[2]))
                                .expect("a shift"),
                        )
                        .expect("a ball"),
                    Other::HalfBallAlone => a_half_ball(&mut model, false, false),
                    Other::CopyOfTheHoled => a_half_ball(&mut model, true, false),
                    Other::HoledFloorFirst => a_half_ball(&mut model, true, true),
                    Other::HoledTurned(angle) => {
                        let turned = a_half_ball(&mut model, true, false);
                        model
                            .place_solid(
                                turned,
                                &Isometry::rotation(
                                    &Axis::new(Point::origin(), Vector::new(0.0, 0.0, 1.0))
                                        .expect("an axis"),
                                    angle,
                                )
                                .expect("a turn"),
                            )
                            .expect("key");
                        turned
                    }
                };
                let (arguments, tools) = if holed_first {
                    ([holed], [other_key])
                } else {
                    ([other_key], [holed])
                };
                let fused = model.fuse(
                    &arguments,
                    &tools,
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                let said = match fused {
                    Ok(answer) => format!("Ok({} solids)", answer.solids().len()),
                    Err(why) => format!("Err({why:?})"),
                };
                println!(
                    "Q1760 before={before} {other:?} {rule:?} holed_first={holed_first}: {said}"
                );
            }
        }
    }
}

fn a_half_ball(model: &mut Model, holed: bool, floor_first: bool) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let on_the_equator = builder
        .add_vertex(Point::new(RADIUS, 0.0, 0.0))
        .expect("a point");
    let equator = builder
        .add_edge(
            on_the_equator,
            on_the_equator,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a closed edge");
    let walking = |builder: &mut SolidBuilder, edge, orientation| {
        let coedge = builder
            .add_coedge(edge, orientation)
            .expect("an edge of this builder");
        builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("a closed wire")
    };
    let equator_on_the_skin = walking(&mut builder, equator, Orientation::Forward);
    let equator_on_the_floor = walking(&mut builder, equator, Orientation::Reversed);
    let ball = Surface::Sphere(Sphere::new(Point::origin(), RADIUS).expect("a ball"));
    let floor = builder
        .add_face(
            Surface::Plane(
                Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
                    .expect("a plane"),
            ),
            &[equator_on_the_floor],
        )
        .expect("a face with one wire");
    let mut faces = if holed {
        let corner = builder
            .add_vertex(Point::new(0.0, 0.0, RADIUS))
            .expect("a point");
        let rim = builder
            .add_edge(
                corner,
                corner,
                EdgeRun::RightRoundWhereItCrosses {
                    and: Surface::Plane(
                        Plane::from_point_and_normal(
                            Point::new(0.0, 3.0, 4.0),
                            Vector::new(0.0, 1.0, 3.0),
                        )
                        .expect("a plane"),
                    ),
                },
            )
            .expect("a closed edge");
        let hole = walking(&mut builder, rim, Orientation::Reversed);
        let rim_of_the_cap = walking(&mut builder, rim, Orientation::Forward);
        let skin = builder
            .add_face(ball, &[equator_on_the_skin, hole])
            .expect("a face with two wires");
        let cap = builder
            .add_face(ball, &[rim_of_the_cap])
            .expect("a face with one wire");
        vec![(skin, Orientation::Forward), (cap, Orientation::Forward)]
    } else {
        let skin = builder
            .add_face(ball, &[equator_on_the_skin])
            .expect("a face with one wire");
        vec![(skin, Orientation::Forward)]
    };
    if floor_first {
        faces.insert(0, (floor, Orientation::Forward));
    } else {
        faces.push((floor, Orientation::Forward));
    }
    builder.add_shell(&faces).expect("the faces close up");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a half ball is a solid")
}
