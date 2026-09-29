//! A closed rim between a ball and a plane, with its one corner on the ball's
//! north pole: the closed edge neither of the mesher's direction rules answers.
//!
//! `BuildingAMesh::walk` turns a closed edge's points round to the way the
//! longitude of a curved surface either side runs at the corner. On a ball's
//! pole that longitude is exactly zero, and a plane has none, so neither rule
//! says anything and the walk's own order is kept (`docs/decisions.md` §199.4,
//! §202.1). Which way that order runs is fixed by the run and its two surfaces
//! alone; the `and` plane turned over turns it round.
//!
//! **It is not a wrong mesh, and this file is the demonstration.**
//! `SolidBuilder::build` runs `Model::validate`, whose sweep meshes the solid
//! with the same walk and counts each side of the skin both ways round. A solid
//! whose coedges run against the walk has its lid's triangles run round the rim
//! the same way as its skin's, and is refused `FacesDoNotAgreeWhichWayTheyFace`.
//! So of the two ways round each spelling can be written, exactly one builds,
//! and that one meshes closed and measures at every deflection, alone and after
//! a union.
//!
//! Two solids on one circle, the plane `y + 3z = 15` through the north pole of
//! a ball of radius 5: the ball less the small cap, and the small cap. Each is
//! written with the rim named by its two faces, by the lid's plane, and by that
//! plane turned over, and with the shell listing the skin first and the lid
//! first. The lid's coedge that builds is pinned per spelling, because it is
//! **not** the one the same solid builds with when the corner is off the pole:
//! there `the_way_a_closed_edge_is_stored_running` answers, by the ball's
//! longitude, and the small cap then builds with its lid's coedge reversed in
//! every spelling. Q581 is what settles which of the two defines a pole corner's
//! stored direction for `Model::validate`; whichever it takes, the walk here
//! has to follow, and this pin says so when it moves.
//!
//! The lid listed first, with the rim named by the lid's plane turned over, is
//! left out: that spelling hands the walk two planes and is refused
//! `AWireDoesNotFanIntoALoopToMesh` whichever way round, off the pole as on it.

use core::f64::consts::PI;
use std::collections::HashMap;

use rcad_geom::{
    Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Sphere, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

const RADIUS: f64 = 5.0;

/// Which of the two pieces the plane cuts the ball into is kept.
#[derive(Clone, Copy, Debug)]
enum Piece {
    TheBallLessTheCap,
    TheCap,
}

/// How the rim's run names the second surface of its crossing.
#[derive(Clone, Copy, Debug)]
enum Spelling {
    ByItsTwoFaces,
    ByTheLid,
    ByTheLidTurnedOver,
}

#[test]
fn a_rim_cornered_on_the_pole_builds_one_way_round_and_that_way_measures() {
    let spellings = [
        (Spelling::ByItsTwoFaces, false),
        (Spelling::ByItsTwoFaces, true),
        (Spelling::ByTheLid, false),
        (Spelling::ByTheLid, true),
        (Spelling::ByTheLidTurnedOver, false),
    ];
    let north = Point::new(0.0, 0.0, RADIUS);
    let pieces_and_spellings = [Piece::TheBallLessTheCap, Piece::TheCap]
        .into_iter()
        .flat_map(|piece| spellings.map(|(spelling, lid_first)| (piece, spelling, lid_first)));
    for (piece, spelling, lid_first) in pieces_and_spellings {
        let builds = the_lid_coedge_that_builds_on_the_pole(spelling);
        let what = format!("{piece:?}, rim named {spelling:?}, lid listed first {lid_first}");
        let mut model = a_model();
        let solid = a_piece_of_the_ball(&mut model, piece, spelling, north, builds, lid_first)
            .unwrap_or_else(|why| panic!("{what}: lid {builds:?} builds: {why:?}"));
        let refused = a_piece_of_the_ball(
            &mut a_model(),
            piece,
            spelling,
            north,
            builds.reversed(),
            lid_first,
        );
        assert!(
            matches!(
                refused,
                Err(Error::Invalid(Invalid::FacesDoNotAgreeWhichWayTheyFace))
            ),
            "{what}: the lid {:?} is refused because its faces disagree: {refused:?}",
            builds.reversed()
        );
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

/// Which way round the lid walks the rim in the one solid that builds, with the
/// corner on the pole: the walk's own order, which the lid's plane turned over
/// turns round, whichever piece is kept.
fn the_lid_coedge_that_builds_on_the_pole(spelling: Spelling) -> Orientation {
    match spelling {
        Spelling::ByItsTwoFaces | Spelling::ByTheLid => Orientation::Forward,
        Spelling::ByTheLidTurnedOver => Orientation::Reversed,
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
            Spelling::ByItsTwoFaces,
            Spelling::ByTheLid,
            Spelling::ByTheLidTurnedOver,
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

fn a_model() -> Model {
    Model::new(Scale::millimetres(125.0).expect("a positive size"))
}

/// `piece` of the ball of radius 5 about the origin, cut off by `y + 3z = 15`,
/// with the rim's one corner at `corner` and the lid walking the rim `lid_way`.
fn a_piece_of_the_ball(
    model: &mut Model,
    piece: Piece,
    spelling: Spelling,
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
        Spelling::ByItsTwoFaces => EdgeRun::RightRoundWhereItsFacesCross,
        Spelling::ByTheLid => EdgeRun::RightRoundWhereItCrosses {
            and: Surface::Plane(lid),
        },
        Spelling::ByTheLidTurnedOver => EdgeRun::RightRoundWhereItCrosses {
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
