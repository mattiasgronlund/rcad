//! Closed rims with their one corner where a curved surface has no longitude,
//! on a ball's pole: which way the mesher walks them, and the ones it cannot
//! tell.
//!
//! `BuildingAMesh::walk` turns a closed edge's points round to the way the rim
//! runs, and asks four readings in turn. The first is the longitude of a curved
//! surface either side at the corner (`the_way_a_closed_edge_is_stored_running`).
//! The second is the first curved surface's longitude where the two faces
//! merely touch there (`the_way_a_closed_edge_where_its_faces_touch_runs`).
//! The third is which side of the other surface each face's own corners say it
//! lies on (`the_way_the_faces_along_a_closed_edge_walk_it`, Q665). The last is
//! a flat face whose one wire is the whole rim
//! (`the_way_a_flat_disk_across_a_ball_walks_its_rim`, Q941). The first two
//! read the pair `which_two_surfaces_a_run_is_between` hands over, and for a
//! rim named by a plane that is the plane and only the curved face that sorts
//! first. On a ball's pole that ball's longitude is exactly zero and a plane
//! has none, so neither says anything there. The third reads nothing either
//! when every face along the rim has only the rim's own corner.
//!
//! **A ball and a plane.** The plane `y + 3z = 15` runs through the north pole
//! of a ball of radius 5, and cuts it into the ball less the small cap and the
//! small cap. Each piece is written with the rim named by its two faces, by the
//! lid's plane, and by that plane turned over. The shell lists the skin first
//! or the lid first. The lid answers, because it is a disk (`docs/decisions.md`
//! §220.6). It holds the part of its plane inside the ball, so its use walks the
//! rim along `outward × ∇ball`, and the edge runs the way that use says. Before
//! that the walker's own order was kept. That order turns round with the `and`
//! plane, so of the two ways each spelling can be written exactly one built,
//! and which one depended on the plane (Q606's pin). On the pole the edge's
//! stored direction is not a fact, since the ball's longitude and latitude both
//! vanish there. So the two ways of writing the lid's use are one solid with
//! the edge's direction turned, as §341.5 found for a half ball on a pipe. Both
//! build, and both mesh closed and measure at every deflection, alone and after
//! a union. Off the pole, at `(0, 3, 4)`, the ball's longitude answers, the
//! stored direction is a fact, and one lid coedge per piece builds in every
//! spelling.
//!
//! The lid listed first, with the rim named by the lid's plane turned over, is
//! left out: on the pole that spelling is refused
//! `AWireDoesNotFanIntoALoopToMesh { TooFewCorners }` whichever way round
//! (Q630).
//!
//! **Two balls** (Q629). Balls of radius 5 about the origin and about
//! `(0, 3, 9)` cross on the same plane, in a circle through the first ball's
//! north pole `(0, 0, 5)` and the second ball's south pole `(0, 3, 4)`. Written
//! as two faces either side of that circle, they keep the lens they hold in
//! common when the first ball walks the rim forward, and the union of the two
//! balls when it walks it backward. Cornered on the second ball's pole, the
//! first ball's longitude answers. Named by its two faces and cornered on the
//! first ball's pole, the second ball's longitude answers where the faces
//! touch. One ball written as two faces on itself, the cap and the rest,
//! cornered off its pole, answers by its longitude. All of these are pinned in
//! every spelling, and in each the face walking the rim forward lies beyond the
//! plane.
//!
//! **What no reading answers.** Named by the plane and cornered on the first
//! ball's pole, the rim is read off the first ball and the plane only. Neither
//! face is flat, and neither has a corner off the rim. So the mesher keeps the
//! walker's order, which turns with the plane, and `Model::validate` passes the
//! rim, since no face has a corner to read a side from. One description then
//! means the lens with the plane facing one way and the union of the two balls
//! with it facing the other. The ball written as two faces on itself, split on
//! its pole, measures whole either way, but its two faces trade sides. That is
//! an open defect, and nothing here pins it.
//!
//! **The other corners with no longitude.** A ball whose north pole is a cone's
//! apex, kept in common with the cone, is refused
//! `ACrossingBetweenAConeAndAnotherCurvedSurface` either way round before any
//! reading is asked; `a_cone_cap_whose_rim_runs_through_its_apex.rs` has
//! Q578's other cone-apex corners. A torus never reaches its own axis, since
//! `Torus::new` refuses a tube as wide as the ring or wider
//! (`a_ring_whose_tube_is_as_wide_as_the_ring_cannot_be_made`, in
//! `two_curves_of_one_pair_that_meet.rs`), and a pipe's longitude is never
//! zero.

use core::f64::consts::PI;
use std::collections::HashMap;

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Sphere, Surface,
    Vector,
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

/// Which side of `y + 3z = 15` a face lies on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Side {
    Beyond,
    Short,
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
                    the_volume_of(piece),
                    &format!("{what}, deflection {deflection}"),
                );
            }
            let fused = united_with_a_block_clear_of_it(&mut model, solid);
            meshes_closed_and_measures(
                &model,
                fused,
                0.05,
                the_volume_of(piece),
                &format!("{what}, united"),
            );
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
            meshes_closed_and_measures(&model, solid, 0.05, the_volume_of(piece), &what);
        }
    }
}

/// Cornered on the second ball's south pole `(0, 3, 4)`, the first ball's
/// longitude says which way the rim runs, whichever way the plane naming it
/// faces.
#[test]
fn two_balls_cornered_on_the_second_balls_pole_are_the_lens_one_way_round_and_the_union_the_other()
{
    let south_of_the_second = Point::new(0.0, 3.0, 4.0);
    for and in the_rims_plane_either_way_round(south_of_the_second) {
        the_first_ball_walking_forward_keeps_the_lens_and_backward_the_union(
            south_of_the_second,
            EdgeRun::RightRoundWhereItCrosses {
                and: Surface::Plane(and),
            },
            &format!("named by the plane facing {:?}", and.normal()),
        );
    }
}

/// Named by its two faces, the rim cornered on the first ball's pole is read
/// off the second ball's longitude where the two faces touch, and on the second
/// ball's pole off the first ball's.
#[test]
fn two_balls_whose_rim_is_named_by_its_two_faces_are_the_lens_one_way_round_on_either_pole() {
    for corner in [Point::new(0.0, 0.0, RADIUS), Point::new(0.0, 3.0, 4.0)] {
        the_first_ball_walking_forward_keeps_the_lens_and_backward_the_union(
            corner,
            EdgeRun::RightRoundWhereItsFacesCross,
            &format!("named by its two faces, cornered on {corner:?}"),
        );
    }
}

/// One ball written as two faces on itself, the small cap beyond the plane and
/// the rest, cornered off the pole: the ball's longitude puts the cap on the
/// face walking the rim forward, whichever way the plane naming it faces.
#[test]
fn a_ball_in_two_faces_cornered_off_its_pole_has_the_cap_on_the_face_walking_forward() {
    let off = Point::new(0.0, 3.0, 4.0);
    for and in the_rims_plane_either_way_round(off) {
        builds_both_ways_round_and_measures(
            &format!("a ball in two faces, the plane facing {:?}", and.normal()),
            |_| the_whole_ball(),
            |model, first_way, first_listed_first| {
                a_ball_in_two_faces(
                    model,
                    off,
                    EdgeRun::RightRoundWhereItCrosses {
                        and: Surface::Plane(and),
                    },
                    first_way,
                    first_listed_first,
                )
            },
        );
    }
}

/// A ball whose north pole is a cone's apex, kept in common with the cone's
/// nappe on `+x`: the closed crossing through the apex is refused by name
/// either way round, before any reading is asked which way it runs.
#[test]
fn a_ball_whose_north_pole_is_a_cones_apex_is_refused_by_name() {
    for cone_way in [Orientation::Forward, Orientation::Reversed] {
        let said = a_ball_on_a_cones_apex_in_common(&mut a_model(), cone_way);
        assert!(
            matches!(
                said,
                Err(Error::Geometry(
                    rcad_geom::Error::ACrossingBetweenAConeAndAnotherCurvedSurface
                ))
            ),
            "the cone's coedge {cone_way:?}: {said:?}"
        );
    }
}

/// Written with its first face walking the rim either way round, and listed
/// first or last, the solid builds, validates, meshes closed, measures the
/// closed form for the way the first face walks, and the face walking the rim
/// forward lies beyond the plane.
fn builds_both_ways_round_and_measures(
    what: &str,
    closed_form: impl Fn(Orientation) -> f64,
    build: impl Fn(&mut Model, Orientation, bool) -> Result<SolidKey, Error>,
) {
    for first_listed_first in [false, true] {
        for first_way in [Orientation::Forward, Orientation::Reversed] {
            let what =
                format!("{what}, the first face {first_way:?}, listed first {first_listed_first}");
            let mut model = a_model();
            let solid = build(&mut model, first_way, first_listed_first)
                .unwrap_or_else(|why| panic!("{what}: builds: {why:?}"));
            validates(&model, &what);
            for deflection in [0.5, 0.05] {
                meshes_closed_and_measures(
                    &model,
                    solid,
                    deflection,
                    closed_form(first_way),
                    &format!("{what}, deflection {deflection}"),
                );
            }
            assert_eq!(
                the_side_the_face_walking_the_rim_forward_lies_on(&model, solid, &what),
                Side::Beyond,
                "{what}"
            );
        }
    }
}

fn the_first_ball_walking_forward_keeps_the_lens_and_backward_the_union(
    corner: Point,
    run: EdgeRun,
    what: &str,
) {
    builds_both_ways_round_and_measures(
        &format!("two balls, {what}"),
        |first_way| match first_way {
            Orientation::Forward => the_lens(),
            Orientation::Reversed => the_union_of_the_two_balls(),
        },
        |model, first_way, first_listed_first| {
            two_balls_round_one_rim(model, corner, run.clone(), first_way, first_listed_first)
        },
    );
}

/// The plane `y + 3z = 15` through `corner`, facing away from the origin and
/// towards it.
fn the_rims_plane_either_way_round(corner: Point) -> [Plane; 2] {
    let up = Plane::from_point_and_normal(corner, Vector::new(0.0, 1.0, 3.0)).expect("a plane");
    [up, up.reversed()]
}

/// The ball of radius 5 about the origin written as two faces on itself, the
/// small cap beyond `y + 3z = 15` and the rest, either side of one rim.
fn a_ball_in_two_faces(
    model: &mut Model,
    corner: Point,
    run: EdgeRun,
    first_way: Orientation,
    first_listed_first: bool,
) -> Result<SolidKey, Error> {
    let ball = Surface::Sphere(Sphere::new(Point::origin(), RADIUS).expect("a ball"));
    two_faces_round_one_rim(
        model,
        corner,
        run,
        [ball, ball],
        first_way,
        first_listed_first,
    )
}

/// Two balls of radius 5, the first about the origin and the second about
/// `(0, 3, 9)`, either side of the circle where they cross on `y + 3z = 15`.
/// That circle runs through `(0, 0, 5)`, the first ball's north pole, and
/// `(0, 3, 4)`, the second ball's south pole.
fn two_balls_round_one_rim(
    model: &mut Model,
    corner: Point,
    run: EdgeRun,
    first_way: Orientation,
    first_listed_first: bool,
) -> Result<SolidKey, Error> {
    two_faces_round_one_rim(
        model,
        corner,
        run,
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
    run: EdgeRun,
    [first, second]: [Surface; 2],
    first_way: Orientation,
    first_listed_first: bool,
) -> Result<SolidKey, Error> {
    let mut builder = SolidBuilder::new();
    let corner = builder.add_vertex(corner)?;
    let rim = builder.add_edge(corner, corner, run)?;
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

/// The ball of radius 5 about `(0, 0, −5)`, whose north pole is the apex of
/// the cone of slope `0.75` along `x`, and that cone's nappe on `+x`, kept in
/// common either side of the closed crossing through the apex.
fn a_ball_on_a_cones_apex_in_common(
    model: &mut Model,
    cone_way: Orientation,
) -> Result<SolidKey, Error> {
    let mut builder = SolidBuilder::new();
    let apex = builder.add_vertex(Point::origin())?;
    let rim = builder.add_edge(apex, apex, EdgeRun::RightRoundWhereItsFacesCross)?;
    let on_the_cone = DraftStep::Along(builder.add_coedge(rim, cone_way)?);
    let on_the_ball = DraftStep::Along(builder.add_coedge(rim, cone_way.reversed())?);
    let cone_wire = builder.add_wire(&[on_the_cone])?;
    let ball_wire = builder.add_wire(&[on_the_ball])?;
    let cone = builder.add_face(
        Surface::Cone(Cone::new(Axis::x(), 0.75).expect("a positive slope")),
        &[cone_wire],
    )?;
    let ball = builder.add_face(
        Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, -RADIUS), RADIUS).expect("a ball")),
        &[ball_wire],
    )?;
    builder.add_shell(&[(cone, Orientation::Forward), (ball, Orientation::Forward)])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(125.0).expect("a positive size"))
}

fn validates(model: &Model, what: &str) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: validates: {why:?}"));
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
    validates(model, "the union");
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
/// is under the closed form, since every triangle is a chord of the ball its
/// face lies on, and it is within the error the measure publishes.
fn meshes_closed_and_measures(
    model: &Model,
    solid: SolidKey,
    deflection: f64,
    closed_form: f64,
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

/// The side of `y + 3z = 15` the face walking the rim forward lies on, meshed
/// at 0.05. Every triangle of each face lies on one side, the two faces on
/// different sides, leaving out a triangle whose three corners are on the rim.
/// At 0.5 the cap can be all such triangles.
fn the_side_the_face_walking_the_rim_forward_lies_on(
    model: &Model,
    solid: SolidKey,
    what: &str,
) -> Side {
    const ON_THE_RIM: f64 = 1e-9;
    let view = model.solid(solid).expect("this model made the key");
    let mesh = view
        .mesh(
            Deflection::of(0.05).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the solid meshes: {why:?}"));
    let forward = view
        .faces()
        .find(|face| {
            face.outer_wire()
                .coedges()
                .all(|coedge| coedge.orientation_on_its_edge() == Orientation::Forward)
        })
        .unwrap_or_else(|| panic!("{what}: a face walks the rim forward"))
        .key();
    let mut sides: HashMap<(bool, Side), usize> = HashMap::new();
    for triangle in mesh.triangles() {
        let beyond_by: f64 = triangle
            .corners()
            .map(|corner| {
                let point = mesh.points()[corner as usize];
                point.y() + 3.0 * point.z() - 15.0
            })
            .iter()
            .sum();
        let side = if beyond_by > ON_THE_RIM {
            Side::Beyond
        } else if beyond_by < -ON_THE_RIM {
            Side::Short
        } else {
            continue;
        };
        *sides.entry((triangle.face() == forward, side)).or_default() += 1;
    }
    let the_side_of = |walks_forward: bool| {
        let on: Vec<Side> = [Side::Beyond, Side::Short]
            .into_iter()
            .filter(|side| sides.contains_key(&(walks_forward, *side)))
            .collect();
        let [side] = on.as_slice() else {
            panic!("{what}: the face walking forward {walks_forward} lies on one side: {sides:?}");
        };
        *side
    };
    let forward_side = the_side_of(true);
    assert_ne!(
        forward_side,
        the_side_of(false),
        "{what}: the two faces lie on different sides: {sides:?}"
    );
    forward_side
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

/// Both balls' centres are `15/√10` from the plane, so their lens is two small
/// caps.
fn the_lens() -> f64 {
    2.0 * the_volume_of(Piece::TheCap)
}

fn the_union_of_the_two_balls() -> f64 {
    2.0 * the_whole_ball() - the_lens()
}

fn the_whole_ball() -> f64 {
    4.0 * PI * RADIUS.powi(3) / 3.0
}
