//! A piece of a ball whose rim is walked against the way it runs, read back
//! from bytes. `SolidBuilder` refuses it, reading back lets it through, and
//! `Model::validate` refuses it by the builder's name. Queue row Q631;
//! `docs/decisions.md` §44.5, §199.4 and §220.6 are the record.
//!
//! The ball of radius 5 about the origin is cut by `y + 3z = 15` into the ball
//! less the small cap and the small cap, as in
//! `a_closed_edge_neither_direction_rule_answers.rs`. Each piece is the ball's
//! face and a flat lid either side of one closed edge, the rim, with its one
//! corner at `(0, 3, 4)`, a quarter of the way off the north pole. The rim is
//! named three ways: by its two faces, by the lid's plane, and by that plane
//! turned over.
//!
//! **Off the pole the way the rim runs is a fact.** The ball's longitude runs
//! round the rim there, so the edge is stored running one way, and which way
//! each face walks it says which side of the rim that face lies on. For each
//! piece one way of writing the lid builds and the other is refused
//! `FacesDoNotAgreeWhichWayTheyFace`, in every spelling. `Model::validate` finds
//! that in `every_side_of_the_skin_is_used_both_ways`, which nothing but
//! measuring a curved solid's volume reaches.
//!
//! **On the pole there is nothing to read back.** With the corner on
//! `(0, 0, 5)` every spelling builds either way round (§220.6), which
//! `a_rim_cornered_on_the_pole_builds_either_way_round_and_both_ways_measure`
//! pins.
//!
//! **Reading back meshes nothing.** `Deserialize for Model` checks that the keys
//! resolve, that nothing is used twice, that everything has an owner, and then
//! `check_shape_is_possible`, which §44.5 keeps to the checks that need no
//! policy and no arithmetic. So the refused piece is written as the kernel's
//! own bytes for the piece that builds, with its two coedges turned round. That
//! is two bytes, each `0` and `1` swapped, and nothing else.
//!
//! **What a caller who skips validate gets back is wrong, and is not pinned.**
//! `SolidView::mesh` hands back a mesh whose sides along the rim are each used
//! twice the same way round, and `SolidView::measure` a positive volume with no
//! refusal. The ball less the cap, turned, measures as the cap within the error
//! it publishes; the cap, turned, measures as neither piece.
//! [`what_mesh_and_measure_hand_a_caller_who_skips_validate`] prints both at
//! three deflections. That is an open defect, filed by Q631's lane, and the
//! test is to become its pin.

use core::f64::consts::PI;
use std::collections::HashMap;

use rcad_geom::{Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, SolidBuilder, SolidKey, SolidView};

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
fn off_the_pole_the_builder_refuses_the_lid_walking_the_rim_against_the_longitude_by_name() {
    for (piece, spelling) in every_piece_and_spelling() {
        let refused = a_piece_of_the_ball(
            &mut a_model(),
            piece,
            spelling,
            the_way_the_lid_builds(piece).reversed(),
        );
        assert!(
            matches!(
                refused,
                Err(Error::Invalid(Invalid::FacesDoNotAgreeWhichWayTheyFace))
            ),
            "{piece:?}, rim named {spelling:?}: {refused:?}"
        );
    }
}

/// The builder stores each coedge the way it was given, and the bytes it
/// writes differ from the turned ones in the two coedges' orientations alone.
/// Reading back lets the turned piece through, with the lid walking the rim
/// the way the builder refused.
#[test]
fn turning_the_rim_in_the_bytes_turns_two_bytes_and_reading_back_lets_it_through() {
    for (piece, spelling) in every_piece_and_spelling() {
        let what = format!("{piece:?}, rim named {spelling:?}");
        let builds = the_way_the_lid_builds(piece);
        let (built, turned) =
            the_kernels_bytes_and_those_bytes_with_the_rim_turned(piece, spelling);
        assert_eq!(built.len(), turned.len(), "{what}");
        let swapped: Vec<(u8, u8)> = built
            .iter()
            .zip(&turned)
            .filter(|(was, is)| was != is)
            .map(|(was, is)| (*was, *is))
            .collect();
        assert_eq!(swapped.len(), 2, "{what}: {swapped:?}");
        assert!(
            swapped.iter().all(|pair| matches!(pair, (0, 1) | (1, 0))),
            "{what}: {swapped:?}"
        );

        let built: Model = postcard::from_bytes(&built).expect("the built piece reads back");
        assert_eq!(the_way_the_lid_walks_the_rim(&built), builds, "{what}");
        let turned: Model = postcard::from_bytes(&turned)
            .unwrap_or_else(|why| panic!("{what}: reading back meshes nothing: {why:?}"));
        assert_eq!(
            the_way_the_lid_walks_the_rim(&turned),
            builds.reversed(),
            "{what}"
        );
    }
}

#[test]
fn validate_refuses_the_turned_rim_by_the_name_the_builder_refuses_it() {
    for (piece, spelling) in every_piece_and_spelling() {
        let swept = read_back_turned(piece, spelling)
            .validate(&Policy::certified(), &mut ProvenanceLog::new());
        assert!(
            matches!(
                swept,
                Err(Error::Invalid(Invalid::FacesDoNotAgreeWhichWayTheyFace))
            ),
            "{piece:?}, rim named {spelling:?}: {swept:?}"
        );
    }
}

/// Printed rather than pinned, because what comes back is wrong: the rim's
/// sides are each used twice the same way round, and the volume is positive,
/// with no refusal. Run it with `--no-capture` to read it.
///
/// For each mesh: its points and triangles, whether every side has exactly two
/// triangles, how many sides are used twice the same way round, how many of the
/// ball's triangles lie beyond the plane and how many short of it, and which
/// way the lid's triangles face, as the sign of their summed normals along
/// `(0, 1, 3)`. Then the volume and the error `measure` publishes, beside the
/// closed forms of the piece the bytes were built as and of its complement in
/// the ball.
#[test]
fn what_mesh_and_measure_hand_a_caller_who_skips_validate() {
    for (piece, spelling) in every_piece_and_spelling() {
        let model = read_back_turned(piece, spelling);
        let view = model.solids().next().expect("the piece came back");
        for deflection in [0.5, 0.05, 0.005] {
            let what = format!("{piece:?} turned, rim named {spelling:?}, deflection {deflection}");
            let deflection = Deflection::of(deflection).expect("a positive length");
            match view.mesh(deflection, &Policy::certified(), &mut ProvenanceLog::new()) {
                Ok(mesh) => println!("{what}: mesh {}", what_the_mesh_is(view, &mesh)),
                Err(why) => println!("{what}: mesh refused {why:?}"),
            }
            match view.measure(deflection, &Policy::certified(), &mut ProvenanceLog::new()) {
                Ok(measured) => println!(
                    "{what}: volume {} error {}, the piece {} its complement {}",
                    measured.volume(),
                    measured.volume_error(),
                    the_volume_of(piece),
                    the_whole_ball() - the_volume_of(piece)
                ),
                Err(why) => println!("{what}: measure refused {why:?}"),
            }
        }
    }
}

fn what_the_mesh_is(view: SolidView<'_>, mesh: &rcad_topo::Mesh) -> String {
    let lid = view
        .faces()
        .find(|face| face.surface().as_plane().is_some())
        .expect("a lid")
        .key();
    let mut balance: HashMap<(u32, u32), i32> = HashMap::new();
    let mut triangles_on: HashMap<(u32, u32), u32> = HashMap::new();
    let (mut beyond, mut short) = (0, 0);
    let mut lid_facing = 0.0;
    for triangle in mesh.triangles() {
        let [first, second, third] = triangle.corners();
        for (from, to) in [(first, second), (second, third), (third, first)] {
            let side = (from.min(to), from.max(to));
            *balance.entry(side).or_default() += if from < to { 1 } else { -1 };
            *triangles_on.entry(side).or_default() += 1;
        }
        let [a, b, c] = triangle
            .corners()
            .map(|corner| mesh.points()[corner as usize]);
        if triangle.face() == lid {
            lid_facing += (b - a).cross(c - a).dot(Vector::new(0.0, 1.0, 3.0));
            continue;
        }
        let beyond_by: f64 = [a, b, c]
            .iter()
            .map(|point| point.y() + 3.0 * point.z() - 15.0)
            .sum();
        if beyond_by > 0.0 {
            beyond += 1;
        } else {
            short += 1;
        }
    }
    format!(
        "{} points {} triangles, every side has two {}, sides used twice the same way {}, \
         the ball's triangles beyond {beyond} short {short}, the lid faces {}(0, 1, 3)",
        mesh.points().len(),
        mesh.triangles().len(),
        triangles_on.values().all(|count| *count == 2),
        balance.values().filter(|count| **count != 0).count(),
        if lid_facing > 0.0 { "+" } else { "-" }
    )
}

fn every_piece_and_spelling() -> impl Iterator<Item = (Piece, NamedBy)> {
    [Piece::TheBallLessTheCap, Piece::TheCap]
        .into_iter()
        .flat_map(|piece| {
            [
                NamedBy::ItsTwoFaces,
                NamedBy::TheLid,
                NamedBy::TheLidTurnedOver,
            ]
            .map(|spelling| (piece, spelling))
        })
}

/// The way the lid walks the rim in the piece `SolidBuilder` builds, which
/// `off_the_pole_the_longitude_picks_the_way_round_whatever_the_spelling` pins.
fn the_way_the_lid_builds(piece: Piece) -> Orientation {
    match piece {
        Piece::TheBallLessTheCap => Orientation::Forward,
        Piece::TheCap => Orientation::Reversed,
    }
}

fn the_way_the_lid_walks_the_rim(model: &Model) -> Orientation {
    let view = model.solids().next().expect("the piece came back");
    let lid = view
        .faces()
        .find(|face| face.surface().as_plane().is_some())
        .expect("a lid");
    let [way] = lid
        .outer_wire()
        .coedges()
        .map(|coedge| coedge.orientation_on_its_edge())
        .collect::<Vec<_>>()[..]
    else {
        panic!("the lid's wire is the rim alone");
    };
    way
}

fn read_back_turned(piece: Piece, spelling: NamedBy) -> Model {
    let (_, turned) = the_kernels_bytes_and_those_bytes_with_the_rim_turned(piece, spelling);
    postcard::from_bytes(&turned).expect("reading back meshes nothing")
}

/// The bytes postcard writes for the piece as `SolidBuilder` builds it, and
/// the same bytes with each coedge turned round. They are turned through
/// [`mirror`], which is first held to writing the kernel's bytes back exactly.
fn the_kernels_bytes_and_those_bytes_with_the_rim_turned(
    piece: Piece,
    spelling: NamedBy,
) -> (Vec<u8>, Vec<u8>) {
    let mut model = a_model();
    a_piece_of_the_ball(&mut model, piece, spelling, the_way_the_lid_builds(piece))
        .unwrap_or_else(|why| panic!("{piece:?}, rim named {spelling:?}: builds: {why:?}"));
    let built = postcard::to_allocvec(&model).expect("a model writes out");
    let mut mirrored: mirror::Payload = postcard::from_bytes(&built).expect("the mirror reads it");
    assert_eq!(
        postcard::to_allocvec(&mirrored).expect("the mirror writes out"),
        built,
        "the mirror writes the kernel's bytes"
    );
    for coedge in mirrored.data.coedges.values_mut() {
        coedge.orientation = coedge.orientation.reversed();
    }
    let turned = postcard::to_allocvec(&mirrored).expect("the mirror writes out");
    (built, turned)
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(125.0).expect("a positive size"))
}

/// `piece` of the ball of radius 5 about the origin, cut off by `y + 3z = 15`,
/// with the rim's one corner at `(0, 3, 4)`, the lid walking the rim `lid_way`
/// and the skin the other way, and the skin listed first.
fn a_piece_of_the_ball(
    model: &mut Model,
    piece: Piece,
    spelling: NamedBy,
    lid_way: Orientation,
) -> Result<SolidKey, Error> {
    let corner = Point::new(0.0, 3.0, 4.0);
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
    builder.add_shell(&[(skin, Orientation::Forward), (lid, Orientation::Forward)])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The closed form of `piece`. The small cap is `π h² (3r − h) / 3` for its
/// height `h`, the radius less the plane's distance `15/√10` from the centre.
fn the_volume_of(piece: Piece) -> f64 {
    let h = RADIUS - 15.0 / 10.0_f64.sqrt();
    let cap = PI * h * h * (3.0 * RADIUS - h) / 3.0;
    match piece {
        Piece::TheBallLessTheCap => the_whole_ball() - cap,
        Piece::TheCap => cap,
    }
}

fn the_whole_ball() -> f64 {
    4.0 * PI * RADIUS.powi(3) / 3.0
}

/// The model's arenas as `Model` writes them, field for field, copied from
/// `a_half_rings_flat_face_read_back_from_bytes.rs`.
/// [`the_kernels_bytes_and_those_bytes_with_the_rim_turned`] holds it to that
/// on every use.
mod mirror {
    use rcad_geom::{EdgeRun, Isometry, Orientation, Point, Scale, Surface};
    use rcad_predicates::Approximation;
    use serde::{Deserialize, Serialize};
    use slotmap::SlotMap;
    use smallvec::SmallVec;

    slotmap::new_key_type! {
        pub struct VertexKey;
        pub struct EdgeKey;
        pub struct CoedgeKey;
        pub struct WireKey;
        pub struct FaceKey;
        pub struct ShellKey;
        pub struct SolidKey;
    }

    #[derive(Serialize, Deserialize)]
    pub struct Payload {
        pub version: u32,
        pub data: Data,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Data {
        pub scale: Scale,
        pub vertices: SlotMap<VertexKey, Vertex>,
        pub edges: SlotMap<EdgeKey, Edge>,
        pub coedges: SlotMap<CoedgeKey, Coedge>,
        pub wires: SlotMap<WireKey, Wire>,
        pub faces: SlotMap<FaceKey, Face>,
        pub shells: SlotMap<ShellKey, Shell>,
        pub solids: SlotMap<SolidKey, Solid>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Vertex {
        pub point: Point,
        pub approximation: Option<Approximation>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Edge {
        pub start: VertexKey,
        pub end: VertexKey,
        pub run: EdgeRun,
        pub approximation: Option<Approximation>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Coedge {
        pub edge: EdgeKey,
        pub orientation: Orientation,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Wire {
        pub steps: SmallVec<[Step; 4]>,
    }

    #[derive(Clone, Copy, Serialize, Deserialize)]
    pub enum Step {
        Along(CoedgeKey),
        Pole(VertexKey),
    }

    #[derive(Serialize, Deserialize)]
    pub struct Face {
        pub surface: Surface,
        pub wires: SmallVec<[WireKey; 1]>,
        pub approximation: Option<Approximation>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Shell {
        pub faces: Vec<FaceUse>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct FaceUse {
        pub face: FaceKey,
        pub loc: Isometry,
        pub orient: Orientation,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Solid {
        pub shells: SmallVec<[ShellUse; 1]>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ShellUse {
        pub shell: ShellKey,
        pub loc: Isometry,
        pub orient: Orientation,
    }
}
