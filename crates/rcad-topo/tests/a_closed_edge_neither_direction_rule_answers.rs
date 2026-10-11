//! A closed rim between a ball and a plane, with its one corner on the ball's
//! north pole: the closed edge none of the mesher's three direction rules
//! answers.
//!
//! `BuildingAMesh::walk` turns a closed edge's points round to the way the
//! longitude of a curved surface either side runs at the corner
//! (`the_way_a_closed_edge_is_stored_running`), or, where that reads nothing,
//! the way the first curved surface's longitude runs where the two faces
//! merely touch there (`the_way_a_closed_edge_where_its_faces_touch_runs`). On
//! a ball's pole that longitude is exactly zero, and a plane has none, so
//! neither says anything. Since Q665, a third rule asks instead which side of
//! the other surface each face's own corners say it lies on
//! (`the_way_the_faces_along_a_closed_edge_walk_it`) — and here it reads
//! nothing too, because every face along this rim has only the rim's own
//! corner, and that corner is never off either surface for either face to
//! read a side from.
//!
//! **The lid answers instead, because it is a disk** (Q941,
//! `docs/decisions.md` §220.6). A flat face whose one wire is the whole rim
//! holds the part of its plane inside the ball, so its use walks the rim along
//! `outward × ∇ball`, and the edge runs the way that use says — §341.5's rule
//! with the side known from the wire rather than read off a corner. Before
//! that the walk's own order was kept, which the `and` plane turned over turns
//! round, and of the two ways each spelling can be written exactly one built:
//! the other had its lid's triangles run round the rim the same way as its
//! skin's and was refused `FacesDoNotAgreeWhichWayTheyFace`, and which one
//! depended on the plane (Q606's pin).
//!
//! On the pole the edge's stored direction is not a fact — the ball's
//! longitude and latitude both vanish there — so the two ways of writing the
//! lid's use are one solid with the edge's direction turned, as §341.5 found
//! for a half ball on a pipe. Both build now, and both mesh closed and measure
//! at every deflection, alone and after a union.
//!
//! Two solids on one circle, the plane `y + 3z = 15` through the north pole of
//! a ball of radius 5: the ball less the small cap, and the small cap. Each is
//! written with the rim named by its two faces, by the lid's plane, and by that
//! plane turned over, and with the shell listing the skin first and the lid
//! first. Off the pole, at `(0, 3, 4)`, `the_way_a_closed_edge_is_stored_running`
//! answers by the ball's longitude, the stored direction is a fact, and one
//! lid coedge per piece builds in every spelling.
//!
//! The lid listed first, with the rim named by the lid's plane turned over, is
//! left out: on the pole that spelling is refused
//! `AWireDoesNotFanIntoALoopToMesh { TooFewCorners }` whichever way round.

use core::f64::consts::PI;
use std::collections::HashMap;

use rcad_geom::{
    Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Sphere, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

const RADIUS: f64 = 5.0;

/// Which of the two pieces the plane cuts the ball into is kept.
#[derive(Clone, Copy, Debug)]
enum Piece {
    TheBallLessTheCap,
    TheCap,
}

/// What the rim's run names as the second surface of its crossing.
#[derive(Clone, Copy, Debug)]
enum NamedBy {
    ItsTwoFaces,
    TheLid,
    TheLidTurnedOver,
}

#[test]
fn a_rim_cornered_on_the_pole_builds_either_way_round_and_both_ways_measure() {
    let spellings = [
        (NamedBy::ItsTwoFaces, false),
        (NamedBy::ItsTwoFaces, true),
        (NamedBy::TheLid, false),
        (NamedBy::TheLid, true),
        (NamedBy::TheLidTurnedOver, false),
    ];
    let north = Point::new(0.0, 0.0, RADIUS);
    let pieces_and_spellings = [Piece::TheBallLessTheCap, Piece::TheCap]
        .into_iter()
        .flat_map(|piece| spellings.map(|(spelling, lid_first)| (piece, spelling, lid_first)));
    for (piece, spelling, lid_first) in pieces_and_spellings {
        for lid_way in [Orientation::Forward, Orientation::Reversed] {
            let what = format!(
                "{piece:?}, rim named {spelling:?}, lid listed first {lid_first}, lid {lid_way:?}"
            );
            let mut model = a_model();
            let solid = a_piece_of_the_ball(&mut model, piece, spelling, north, lid_way, lid_first)
                .unwrap_or_else(|why| panic!("{what}: builds: {why:?}"));
            for deflection in [0.5, 0.05, 0.005] {
                meshes_closed_and_measures(
                    &model,
                    solid,
                    deflection,
                    piece,
                    &format!("{what}, deflection {deflection}"),
                );
            }
            let fused = united_with_a_block_clear_of_it(&mut model, solid);
            meshes_closed_and_measures(&model, fused, 0.05, piece, &format!("{what}, united"));
        }
    }
}

/// The same two pieces with the corner a quarter of the way off the pole, at
/// `(0, 3, 4)`, where the ball's longitude answers: every spelling builds with
/// one lid coedge per piece, and for the cap it is the other one.
#[test]
fn off_the_pole_the_longitude_picks_the_way_round_whatever_the_spelling() {
    let off = Point::new(0.0, 3.0, 4.0);
    for (piece, builds) in [
        (Piece::TheBallLessTheCap, Orientation::Forward),
        (Piece::TheCap, Orientation::Reversed),
    ] {
        for spelling in [
            NamedBy::ItsTwoFaces,
            NamedBy::TheLid,
            NamedBy::TheLidTurnedOver,
        ] {
            let what = format!("{piece:?} cornered off the pole, rim named {spelling:?}");
            let mut model = a_model();
            let solid = a_piece_of_the_ball(&mut model, piece, spelling, off, builds, false)
                .unwrap_or_else(|why| panic!("{what}: lid {builds:?} builds: {why:?}"));
            assert!(
                a_piece_of_the_ball(
                    &mut a_model(),
                    piece,
                    spelling,
                    off,
                    builds.reversed(),
                    false
                )
                .is_err(),
                "{what}: the lid {:?} is refused",
                builds.reversed()
            );
            meshes_closed_and_measures(&model, solid, 0.05, piece, &what);
        }
    }
}

#[test]
#[ignore = "Q629 relay: a probe that prints, to be turned into pins"]
fn probe_q629() {
    let north = Point::new(0.0, 0.0, RADIUS);
    let south_of_the_other = Point::new(0.0, 3.0, 4.0);
    let plane = Plane::from_point_and_normal(north, Vector::new(0.0, 1.0, 3.0)).expect("a plane");
    for and in [plane, plane.reversed()] {
        for cap_first in [false, true] {
            for cap_way in [Orientation::Forward, Orientation::Reversed] {
                let what = format!(
                    "BALL and {:?} cap first {cap_first} cap {cap_way:?}",
                    and.normal()
                );
                eprintln!("PROBE-CASE {what}");
                let mut model = a_model();
                match a_ball_in_two_faces(&mut model, and, cap_way, cap_first) {
                    Err(why) => eprintln!("PROBE-RESULT {what}: refused {why:?}"),
                    Ok(solid) => report(&model, solid, PI * 4.0 * RADIUS.powi(3) / 3.0, &what),
                }
                for corner in [north, south_of_the_other] {
                    let what = format!(
                        "LENS corner {corner:?} and {:?} a first {cap_first} a {cap_way:?}",
                        and.normal()
                    );
                    eprintln!("PROBE-CASE {what}");
                    let mut model = a_model();
                    match a_lens(&mut model, corner, and, cap_way, cap_first) {
                        Err(why) => eprintln!("PROBE-RESULT {what}: refused {why:?}"),
                        Ok(solid) => {
                            report(&model, solid, 2.0 * the_volume_of(Piece::TheCap), &what);
                        }
                    }
                }
            }
        }
    }
}

fn report(model: &Model, solid: SolidKey, closed_form: f64, what: &str) {
    let validated = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
    eprintln!("PROBE-RESULT {what}: builds, validate {validated:?}");
    let view = model.solid(solid).expect("key");
    for deflection in [0.5, 0.05] {
        let mesh = view.mesh(
            Deflection::of(deflection).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        match mesh {
            Err(why) => eprintln!("PROBE-RESULT {what} {deflection}: mesh refused {why:?}"),
            Ok(mesh) => {
                let mut beyond_by_face: HashMap<rcad_topo::FaceKey, (usize, usize)> =
                    HashMap::new();
                for triangle in mesh.triangles() {
                    let beyond = triangle
                        .corners()
                        .map(|corner| mesh.points()[corner as usize])
                        .iter()
                        .map(|point| point.y() + 3.0 * point.z())
                        .sum::<f64>()
                        > 45.0;
                    let entry = beyond_by_face.entry(triangle.face()).or_default();
                    if beyond {
                        entry.0 += 1;
                    } else {
                        entry.1 += 1;
                    }
                }
                let measured = view.measure(
                    Deflection::of(deflection).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                let faces: Vec<_> = view
                    .faces()
                    .map(|face| {
                        (
                            face.key(),
                            face.surface(),
                            beyond_by_face.get(&face.key()).copied(),
                        )
                    })
                    .collect();
                eprintln!(
                    "PROBE-RESULT {what} {deflection}: faces (beyond, short) {faces:?}, measured {:?} closed form {closed_form}",
                    measured.map(|m| (m.volume(), m.volume_error()))
                );
            }
        }
    }
}

/// The ball of radius 5 about the origin written as two faces on itself: the
/// small cap beyond `y + 3z = 15`, and the rest. The rim between them is named
/// by `and` and cornered on the north pole.
fn a_ball_in_two_faces(
    model: &mut Model,
    and: Plane,
    cap_way: Orientation,
    cap_first: bool,
) -> Result<SolidKey, Error> {
    let ball = Surface::Sphere(Sphere::new(Point::origin(), RADIUS).expect("a ball"));
    two_faces_round_one_rim(
        model,
        Point::new(0.0, 0.0, RADIUS),
        and,
        [ball, ball],
        cap_way,
        cap_first,
    )
}

/// The lens two balls of radius 5 hold in common, one about the origin and one
/// about `(0, 3, 9)`, which cross on `y + 3z = 15`.
fn a_lens(
    model: &mut Model,
    corner: Point,
    and: Plane,
    first_way: Orientation,
    first_listed_first: bool,
) -> Result<SolidKey, Error> {
    two_faces_round_one_rim(
        model,
        corner,
        and,
        [
            Surface::Sphere(Sphere::new(Point::origin(), RADIUS).expect("a ball")),
            Surface::Sphere(Sphere::new(Point::new(0.0, 3.0, 9.0), RADIUS).expect("a ball")),
        ],
        first_way,
        first_listed_first,
    )
}

fn two_faces_round_one_rim(
    model: &mut Model,
    corner: Point,
    and: Plane,
    [first, second]: [Surface; 2],
    first_way: Orientation,
    first_listed_first: bool,
) -> Result<SolidKey, Error> {
    let mut builder = SolidBuilder::new();
    let corner = builder.add_vertex(corner)?;
    let rim = builder.add_edge(
        corner,
        corner,
        EdgeRun::RightRoundWhereItCrosses {
            and: Surface::Plane(and),
        },
    )?;
    let first_step = DraftStep::Along(builder.add_coedge(rim, first_way)?);
    let second_step = DraftStep::Along(builder.add_coedge(rim, first_way.reversed())?);
    let first_wire = builder.add_wire(&[first_step])?;
    let second_wire = builder.add_wire(&[second_step])?;
    let first = (
        builder.add_face(first, &[first_wire])?,
        Orientation::Forward,
    );
    let second = (
        builder.add_face(second, &[second_wire])?,
        Orientation::Forward,
    );
    builder.add_shell(&if first_listed_first {
        [first, second]
    } else {
        [second, first]
    })?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(125.0).expect("a positive size"))
}

/// `piece` of the ball of radius 5 about the origin, cut off by `y + 3z = 15`,
/// with the rim's one corner at `corner` and the lid walking the rim `lid_way`.
fn a_piece_of_the_ball(
    model: &mut Model,
    piece: Piece,
    spelling: NamedBy,
    corner: Point,
    lid_way: Orientation,
    lid_first: bool,
) -> Result<SolidKey, Error> {
    let out_of_the_ball_less_the_cap = Vector::new(0.0, 1.0, 3.0);
    let lid = Plane::from_point_and_normal(
        corner,
        match piece {
            Piece::TheBallLessTheCap => out_of_the_ball_less_the_cap,
            Piece::TheCap => -out_of_the_ball_less_the_cap,
        },
    )
    .expect("a plane");
    let run = match spelling {
        NamedBy::ItsTwoFaces => EdgeRun::RightRoundWhereItsFacesCross,
        NamedBy::TheLid => EdgeRun::RightRoundWhereItCrosses {
            and: Surface::Plane(lid),
        },
        NamedBy::TheLidTurnedOver => EdgeRun::RightRoundWhereItCrosses {
            and: Surface::Plane(lid.reversed()),
        },
    };

    let mut builder = SolidBuilder::new();
    let corner = builder.add_vertex(corner)?;
    let rim = builder.add_edge(corner, corner, run)?;
    let skin_step = DraftStep::Along(builder.add_coedge(rim, lid_way.reversed())?);
    let lid_step = DraftStep::Along(builder.add_coedge(rim, lid_way)?);
    let skin_wire = builder.add_wire(&[skin_step])?;
    let lid_wire = builder.add_wire(&[lid_step])?;
    let skin = builder.add_face(
        Surface::Sphere(Sphere::new(Point::origin(), RADIUS).expect("a ball")),
        &[skin_wire],
    )?;
    let lid = builder.add_face(Surface::Plane(lid), &[lid_wire])?;
    let (skin, lid) = ((skin, Orientation::Forward), (lid, Orientation::Forward));
    builder.add_shell(&if lid_first { [lid, skin] } else { [skin, lid] })?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The union of `solid` with a block of side 2 well clear of it, and the piece
/// of the answer that is not the block.
fn united_with_a_block_clear_of_it(model: &mut Model, solid: SolidKey) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let block = model
        .add_extrusion(
            &square,
            2.0,
            &Isometry::translation(Vector::new(20.0, 20.0, 20.0)).expect("a placement"),
        )
        .expect("a block");
    let fused = model
        .fuse(
            &[solid],
            &[block],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a union with a block clear of it answers");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the union is a sound model");
    let pieces: Vec<SolidKey> = fused
        .solids()
        .iter()
        .copied()
        .filter(|piece| {
            model
                .solid(*piece)
                .expect("the fuse made the key")
                .faces()
                .any(|face| face.surface().as_plane().is_none())
        })
        .collect();
    let [piece] = pieces.as_slice() else {
        panic!("one piece of the union is curved, not {}", pieces.len());
    };
    *piece
}

/// The solid meshes into a closed surface with the shape of a ball, its volume
/// is under the closed form, since every chord of a convex solid lies inside
/// it, and it is within the error the measure publishes.
fn meshes_closed_and_measures(
    model: &Model,
    solid: SolidKey,
    deflection: f64,
    piece: Piece,
    what: &str,
) {
    let view = model.solid(solid).expect("this model made the key");
    let mesh = view
        .mesh(
            Deflection::of(deflection).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the solid meshes: {why:?}"));
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

    let closed_form = the_volume_of(piece);
    let measured = view
        .measure(
            Deflection::of(deflection).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the solid measures: {why:?}"));
    let found = measured.volume();
    assert!(
        (0.0..=measured.volume_error()).contains(&(closed_form - found)),
        "{what}: measured {found}, closed form {closed_form}, published error {}",
        measured.volume_error()
    );
}

/// The closed form of `piece`. The small cap is `π h² (3r − h) / 3` for its
/// height `h`, the radius less the plane's distance `15/√10` from the centre.
fn the_volume_of(piece: Piece) -> f64 {
    let h = RADIUS - 15.0 / 10.0_f64.sqrt();
    let cap = PI * h * h * (3.0 * RADIUS - h) / 3.0;
    match piece {
        Piece::TheBallLessTheCap => 4.0 * PI * RADIUS.powi(3) / 3.0 - cap,
        Piece::TheCap => cap,
    }
}
