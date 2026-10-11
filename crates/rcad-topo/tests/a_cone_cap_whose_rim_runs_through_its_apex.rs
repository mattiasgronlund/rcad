//! A cone face whose rim has a corner at the cone's apex, and what
//! `which_pole_this_cap_reaches` in `mesh.rs` makes of it (Q578,
//! `docs/decisions.md` §199.4).
//!
//! `which_pole_this_cap_reaches` answers `None` for a rim with a corner at the
//! pinch it would reach, and its caller refuses the face as
//! `ACurvedFaceGoesRightRoundWithNoSeam`. Q578 looked for a face that gets
//! there and found none. Each way this kernel has to put a corner of a cone's
//! rim on its apex is below, with what it gives. None of them hands
//! `which_pole_this_cap_reaches` a rim with a corner at the apex. A scratch
//! print there, on every call that answered `None` or saw a corner on a
//! pinch, saw no cone through every fuse and build here, nor through the whole
//! workspace and the slow tier: only balls' rims with a corner on a pole, and
//! none of them answered `None`. With its two comparisons made
//! `<=` and `>=` instead, the 44 builds and fuses of the probe the first four
//! tests here came from all came out the same.
//!
//! **Why none gets there.** A corner on a pinch opens a pole side across the
//! material there (`WalkingAWire::turn_at_a_corner_on_a_pole`), so the loop
//! the walk closes keeps the apex off the material. Either the material is the
//! bounded side, and the loop climbs nothing and is not a cap at all, or the
//! loop goes right round with the apex on its far side. On a cone that far
//! side is the endless rest of the nappe, the other `None`, and it is right to
//! refuse it.
//!
//! **And these are the only ways.** A corner sits on the apex only where an
//! edge runs through it. A cone has no crossing through its apex but a plane's:
//! `rcad_geom::intersect` refuses a cone against a pipe, a ball or a cone off
//! its axis, and a ball on its axis whose skin runs through the apex, all as
//! `ACrossingBetweenAConeAndAnotherCurvedSurface`. Anything else on its axis
//! cuts it in circles round the axis, which miss the apex, and a torus never
//! reaches its own axis. Two cones on one axis with one apex and two slopes
//! meet at the apex and nowhere else, which puts no edge there; that pair was
//! not built. A plane through the apex cuts the cone in two rulings, or
//! touches it along one. The only other edge left is a straight ruling
//! described through `SolidBuilder`, and a ruling walked up and back down is a
//! seam. So the shapes are:
//!
//! - **A seam up one ruling**, through `SolidBuilder`. It builds whether or not
//!   the wire stands still at the apex and whichever step comes first, meshes
//!   closed and measures.
//! - **Two planes through the apex**, a block's edge up the cone's axis: a
//!   quarter cut out, a quarter cut out up to `z = 4` only, and an eighth.
//!   Every fuse answers and measures. The half of `a_block_face_through_a_cones_apex.rs`
//!   is the same family.
//! - **A plane along one ruling.** Laid exactly on its plane, a block's wall
//!   touches the cone along the ruling, and every fuse answers the two as they
//!   stand: the skin is never cut along the ruling. Turned into place instead,
//!   the block's floor lands a rounding off the touching plane, and every fuse
//!   is refused once its answer is meshed: three by name while meshing, three
//!   `Undecidable` after.
//! - **A closed edge cornered at the apex.** It has to run along a closed
//!   crossing through the apex, and every one is refused by name, through a
//!   fuse and through `SolidBuilder` alike: a bar whose wall runs through the
//!   apex, and a ball through the apex whose crossing with the cone has a cusp
//!   there.

use core::f64::consts::PI;
use std::collections::HashMap;

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Sphere, Surface,
    SurfaceMeet, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

mod cones;

use cones::a_pointed_cone;

const SLOPE: f64 = 0.5;
const HEIGHT: f64 = 8.0;

/// The pointed cone of slope one half and height eight, `π · 4² · 8 / 3`.
const CONE: f64 = 128.0 * PI / 3.0;

/// The cone whose skin is one wire round its cap circle, down the ruling to
/// `(4, 0, 8)` and back up it, builds, meshes closed and measures, in every
/// spelling: with the wire standing still at the apex or not, and with the
/// seam's two steps first or last.
///
/// The walk comes down the ruling, crosses the apex a whole turn back, and
/// goes up again, so it climbs nothing and the skin is no cap. The volume is
/// under the closed form, since every chord of a convex solid lies inside it.
#[test]
fn a_seam_up_one_ruling_from_the_apex_meshes_closed_and_measures() {
    for stands_at_the_apex in [false, true] {
        for seam_first in [false, true] {
            let what =
                format!("standing at the apex {stands_at_the_apex}, seam first {seam_first}");
            let mut model = a_model();
            let solid =
                a_pointed_cone_seamed_up_a_ruling(&mut model, stands_at_the_apex, seam_first)
                    .unwrap_or_else(|why| panic!("{what}: builds: {why:?}"));
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: validates: {why:?}"));
            meshes_closed(&model, solid, 1, &what);
            let (found, error) = measured(&model, solid, &what);
            assert!(
                (0.0..=error).contains(&(CONE - found)),
                "{what}: measured {found}, closed form {CONE}, published error {error}"
            );
        }
    }
}

/// A block whose edge runs up the cone's axis cuts it with its faces `x = 0`
/// and `y = 0`, each through the apex along two rulings: every fuse answers
/// one solid that validates, meshes closed and measures. So does a wedge
/// between `y = 0` and `y = x`.
///
/// Each outline has a corner at the origin, so that the block's edge there
/// runs up the cone's axis.
///
/// The block is `[0, 6] × [0, 6] × [−2, top]`. Up to `z = 10` it holds a
/// quarter of the cone. Up to `z = 4` it holds a quarter of the tip below
/// `z = 4`, of radius two, which is `π · 2² · 4 / 12`, and the cone less it
/// is one skin round both the cap circle and the notch. The wedge is the
/// triangle `(0, 0)`, `(6, 0)`, `(6, 6)` from `z = −2` to `10` and holds an
/// eighth of the cone.
#[test]
fn planes_through_the_apex_cut_a_piece_out_of_the_cone_and_every_fuse_answers() {
    let square = [[0.0, 0.0], [6.0, 0.0], [6.0, 6.0], [0.0, 6.0]];
    let triangle = [[0.0, 0.0], [6.0, 0.0], [6.0, 6.0]];
    for (what, outline, top, other, shared) in [
        ("a quarter", &square[..], 10.0, 432.0, CONE / 4.0),
        (
            "a quarter up to z = 4",
            &square[..],
            4.0,
            216.0,
            4.0 * PI / 3.0,
        ),
        ("an eighth", &triangle[..], 10.0, 216.0, CONE / 8.0),
    ] {
        for (rule, cone_first) in six_fuses() {
            let volume = match (rule, cone_first) {
                (Rule::Union, _) => CONE + other - shared,
                (Rule::Intersection, _) => shared,
                (Rule::Difference, true) => CONE - shared,
                (Rule::Difference, false) => other - shared,
                (other, _) => panic!("{other:?} is not one of the six"),
            };
            let what = format!("{what} cut out, {rule:?}, cone first {cone_first}");
            let mut model = a_model();
            let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);
            let prism = a_prism(&mut model, outline, -2.0, top);
            let solids = fuse(&mut model, cone, prism, rule, cone_first)
                .unwrap_or_else(|why| panic!("{what}: refused {why:?}"));
            let [solid] = solids[..] else {
                panic!("{what}: {} solids", solids.len())
            };
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: validates: {why:?}"));
            meshes_closed(&model, solid, 1, &what);
            let (found, error) = measured(&model, solid, &what);
            assert!(
                (found - volume).abs() <= error,
                "{what}: measured {found}, closed form {volume}, published error {error}"
            );
        }
    }
}

/// The block `[−6, 6] × [−12, 4] × [0, 6]` turned about `x` so that its floor
/// lies on `2y − z = 0` and the block on the side away from the axis: it
/// would touch the cone along the ruling `x = 0, y = z/2`. Every fuse gets
/// past `select`, and every one is refused once the answer's shells are
/// nested, which meshes them to measure their volumes. The union both ways
/// round and the cone less the block refuse
/// `CannotNameWhichPieceOfACrossingThisIs` while that mesh is made. The other
/// three are refused `Undecidable` on one "side of a half-space" left
/// undecided at `(0, 4, 8)`, where the ruling meets the cap's rim.
///
/// The turn's sine and cosine round apart, so the floor as turned is not the
/// touching plane: its normal's parts are `−0.8944271909999157` and
/// `0.44721359549995765`, where the nearest numbers to `−2/√5` and `1/√5` are
/// `−0.8944271909999159` and `0.4472135954999579`. It cuts the cone in two
/// rulings a rounding apart rather than touching it along one. Until Q1537
/// every fuse was refused `Undecidable` before anything was meshed, at "a
/// point inside a piece of a face", on a lens of the block's end face whose
/// corners are `1.7e-7` either side of `x = 0` at `y = 5.37, z = 10.73`, where
/// that ruling carried on past the cap meets the end face. The cap is at
/// `z = 8`, so the lens is outside the cone, and the rule that places a piece
/// by the piece across a side clear of every face places it there now
/// (§35.6). Laid exactly on its plane instead, the same touch answers every
/// fuse, as the next test shows.
#[test]
fn a_block_resting_on_the_cone_along_a_ruling_is_refused_once_its_answer_is_meshed() {
    let touching = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, -2.0, 1.0))
        .expect("a normal that is not zero");
    let skin = Surface::Cone(Cone::new(Axis::z(), SLOPE).expect("a positive slope"));
    let mut model = a_model();
    let block = a_block_turned_onto_the_ruling(&mut model);
    for face in model.solid(block).expect("this model made the key").faces() {
        let surface = face.surface();
        assert_ne!(
            surface,
            Surface::Plane(touching),
            "a face lies on the touching plane"
        );
        let meet = rcad_geom::intersect(
            &surface,
            &skin,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert!(
            !touches_along_one_ruling(&meet),
            "{surface:?} touches the cone along a ruling"
        );
    }

    for (rule, cone_first) in six_fuses() {
        let mut model = a_model();
        let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);
        let block = a_block_turned_onto_the_ruling(&mut model);
        let said = fuse(&mut model, cone, block, rule, cone_first);
        let refused_while_meshing = matches!(
            (rule, cone_first),
            (Rule::Union, _) | (Rule::Difference, true)
        );
        match said {
            Err(Error::Geometry(rcad_geom::Error::CannotNameWhichPieceOfACrossingThisIs))
                if refused_while_meshing => {}
            Err(Error::Undecidable { records }) if !refused_while_meshing => {
                let [left_undecided] = records.records() else {
                    panic!("{rule:?}, cone first {cone_first}: {records:#?}")
                };
                assert_eq!(
                    left_undecided.predicate(),
                    "side of a half-space",
                    "{rule:?}, cone first {cone_first}: {left_undecided:#?}"
                );
            }
            other => panic!("{rule:?}, cone first {cone_first}: {other:?}"),
        }
    }
}

/// A block whose wall lies exactly on the plane `x − 2y = 0`, against the
/// cone of slope one half lying along `x`: the wall touches the cone along
/// the ruling `y = x/2, z = 0`, and every fuse answers the two as they stand.
/// The skin is never cut along the ruling, so no rim of it has a corner at the
/// apex.
///
/// A plane touches the cone where the ratio of its normal's parts is right,
/// whatever its length. The wall from `(−2, −1)` to `(10, 5)` stores its
/// normal `(1, −2, 0)/√5` part by part, each the nearest number to its true
/// value, and doubling a number is exact, so the stored parts keep the ratio
/// two exactly. `rcad_geom::intersect` answers the one straight run of a
/// plane touching a cone along its whole length.
///
/// The blocks run from `z = −6` to `6`. On the far side of the wall is the
/// triangle `(−2, −1)`, `(10, 5)`, `(−2, 5)`, of volume 432. Round the cone is
/// `(−2, −6)`, `(10, −6)`, `(10, 5)`, `(−2, −1)`, of volume 1152, and the block
/// less the cone is one solid of two shells: the block, and the cone's hollow
/// inside it, touching its wall along the ruling.
#[test]
fn a_block_whose_wall_touches_the_cone_exactly_along_a_ruling_leaves_both_as_they_stand() {
    let wall = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, -2.0, 0.0))
        .expect("a normal that is not zero");
    let the_cone = (CONE, 2, 1);
    let far_block = (432.0, 5, 1);
    let round_block = (1152.0, 6, 1);
    let far_side = [[-2.0, -1.0], [10.0, 5.0], [-2.0, 5.0]];
    let round_the_cone = [[-2.0, -6.0], [10.0, -6.0], [10.0, 5.0], [-2.0, -1.0]];
    for (where_it_is, outline, its_wall, [union, common, cone_less, block_less]) in [
        (
            "on the far side",
            &far_side[..],
            wall,
            [
                vec![the_cone, far_block],
                vec![],
                vec![the_cone],
                vec![far_block],
            ],
        ),
        (
            "round the cone",
            &round_the_cone[..],
            wall.reversed(),
            [
                vec![round_block],
                vec![the_cone],
                vec![],
                vec![(1152.0 - CONE, 8, 2)],
            ],
        ),
    ] {
        let mut model = a_model();
        let block = a_prism(&mut model, outline, -6.0, 6.0);
        let surfaces: Vec<Surface> = model
            .solid(block)
            .expect("this model made the key")
            .faces()
            .map(|face| face.surface())
            .collect();
        assert!(
            surfaces.contains(&Surface::Plane(its_wall)),
            "the block {where_it_is} has no face on the wall: {surfaces:?}"
        );
        let meet = rcad_geom::intersect(
            &Surface::Plane(its_wall),
            &Surface::Cone(a_cone_along_x()),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert!(
            touches_along_one_ruling(&meet),
            "the wall of the block {where_it_is} and the cone: {meet:?}"
        );

        for (rule, cone_first) in six_fuses() {
            let expected = match (rule, cone_first) {
                (Rule::Union, _) => &union,
                (Rule::Intersection, _) => &common,
                (Rule::Difference, true) => &cone_less,
                (Rule::Difference, false) => &block_less,
                (other, _) => panic!("{other:?} is not one of the six"),
            };
            let what = format!("the block {where_it_is}, {rule:?}, cone first {cone_first}");
            let mut model = a_model();
            let cone = a_pointed_cone_lying_along_x(&mut model)
                .unwrap_or_else(|why| panic!("{what}: the cone builds: {why:?}"));
            let block = a_prism(&mut model, outline, -6.0, 6.0);
            let solids = fuse(&mut model, cone, block, rule, cone_first)
                .unwrap_or_else(|why| panic!("{what}: refused {why:?}"));
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: validates: {why:?}"));
            let mut pieces: Vec<(f64, f64, SolidKey)> = solids
                .iter()
                .map(|&solid| {
                    let (found, error) = measured(&model, solid, &what);
                    (found, error, solid)
                })
                .collect();
            pieces.sort_by(|one, other| one.0.total_cmp(&other.0));
            assert_eq!(
                pieces.len(),
                expected.len(),
                "{what}: measured {pieces:?}, expected {expected:?}"
            );
            for (&(found, error, solid), &(volume, faces, shells)) in pieces.iter().zip(expected) {
                assert!(
                    (found - volume).abs() <= error,
                    "{what}: measured {found}, closed form {volume}, published error {error}"
                );
                assert_eq!(
                    model
                        .solid(solid)
                        .expect("this model made the key")
                        .faces()
                        .count(),
                    faces,
                    "{what}: faces of the piece of volume {volume}"
                );
                meshes_closed(&model, solid, shells, &what);
            }
        }
    }
}

/// A bar whose wall runs through the apex, and a ball through the apex, are
/// refused `ACrossingBetweenAConeAndAnotherCurvedSurface` under every fuse,
/// and so is the cone and the ball kept in common described through
/// `SolidBuilder` with its one closed edge cornered at the apex.
///
/// The bar is `x² + (y − 1)² ≤ 1` from `z = −2` to `10`, against the cone of
/// slope one half. The ball has radius 5 about `(0, −4, 3)`, against a cone of
/// slope `0.75` and height 12. It runs through the apex, and its plane there,
/// `4y − 3z = 0`, touches the cone along the ruling `x = 0, y = 0.75 z`, so
/// their crossing comes into the apex from both sides along that one ruling.
/// That is the one shape where a pole side could be read a hair wide rather
/// than a whole turn, and it never reaches the mesher.
#[test]
fn a_bar_or_a_ball_whose_wall_runs_through_the_apex_is_refused_by_name() {
    let refused_by_name = |said: &rcad_topo::Result<Vec<SolidKey>>| {
        matches!(
            said,
            Err(Error::Geometry(
                rcad_geom::Error::ACrossingBetweenAConeAndAnotherCurvedSurface
            ))
        )
    };
    for (rule, cone_first) in six_fuses() {
        let mut model = a_model();
        let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);
        let bar = model
            .add_round_bar(
                1.0,
                12.0,
                &Isometry::translation(Vector::new(0.0, 1.0, -2.0)).expect("a finite move"),
            )
            .expect("a bar");
        let said = fuse(&mut model, cone, bar, rule, cone_first);
        assert!(
            refused_by_name(&said),
            "the bar, {rule:?}, cone first {cone_first}: {said:?}"
        );

        let mut model = a_model();
        let cone = a_pointed_cone(&mut model, 0.75, 12.0);
        let ball = model
            .add_ball(
                5.0,
                &Isometry::translation(Vector::new(0.0, -4.0, 3.0)).expect("a finite move"),
            )
            .expect("a ball");
        let said = fuse(&mut model, cone, ball, rule, cone_first);
        assert!(
            refused_by_name(&said),
            "the ball, {rule:?}, cone first {cone_first}: {said:?}"
        );
    }
    for cone_way in [Orientation::Forward, Orientation::Reversed] {
        let said =
            the_cone_and_the_ball_in_common(&mut a_model(), cone_way).map(|solid| vec![solid]);
        assert!(
            refused_by_name(&said),
            "the two in common, the cone's coedge {cone_way:?}: {said:?}"
        );
    }
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// The cone of slope one half and height eight whose skin's one wire goes
/// round the cap circle, down the ruling to `(4, 0, 8)`, and back up it.
fn a_pointed_cone_seamed_up_a_ruling(
    model: &mut Model,
    stands_at_the_apex: bool,
    seam_first: bool,
) -> rcad_topo::Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let rim = builder.add_vertex(Point::new(SLOPE * HEIGHT, 0.0, HEIGHT))?;
    let apex = builder.add_vertex(Point::origin())?;
    let circle = builder.add_edge(rim, rim, EdgeRun::RightRoundWhereItsFacesCross)?;
    let ruling = builder.add_edge(apex, rim, EdgeRun::Straight)?;

    let lid =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, HEIGHT), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let lid_step = DraftStep::Along(builder.add_coedge(circle, Orientation::Forward)?);
    let lid_wire = builder.add_wire(&[lid_step])?;
    let lid = builder.add_face(Surface::Plane(lid), &[lid_wire])?;

    let round = DraftStep::Along(builder.add_coedge(circle, Orientation::Reversed)?);
    let down = DraftStep::Along(builder.add_coedge(ruling, Orientation::Reversed)?);
    let up = DraftStep::Along(builder.add_coedge(ruling, Orientation::Forward)?);
    let seam: &[DraftStep] = if stands_at_the_apex {
        &[down, DraftStep::Pole(apex), up]
    } else {
        &[down, up]
    };
    let steps: Vec<DraftStep> = if seam_first {
        seam.iter().copied().chain([round]).collect()
    } else {
        [round].into_iter().chain(seam.iter().copied()).collect()
    };
    let skin_wire = builder.add_wire(&steps)?;
    let skin = builder.add_face(
        Surface::Cone(Cone::new(Axis::z(), SLOPE).expect("a positive slope")),
        &[skin_wire],
    )?;
    builder.add_shell(&[(lid, Orientation::Forward), (skin, Orientation::Forward)])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The pointed cone of slope one half and length eight lying along `x`, its
/// apex at the origin and its lid on `x = 8`, `π · 4² · 8 / 3` like [`CONE`].
fn a_pointed_cone_lying_along_x(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let rim = builder.add_vertex(Point::new(HEIGHT, -SLOPE * HEIGHT, 0.0))?;
    let circle = builder.add_edge(rim, rim, EdgeRun::RightRoundWhereItsFacesCross)?;
    let lid =
        Plane::from_point_and_normal(Point::new(HEIGHT, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
            .expect("the x axis is a unit normal");
    let lid_step = DraftStep::Along(builder.add_coedge(circle, Orientation::Forward)?);
    let lid_wire = builder.add_wire(&[lid_step])?;
    let lid = builder.add_face(Surface::Plane(lid), &[lid_wire])?;
    let skin_step = DraftStep::Along(builder.add_coedge(circle, Orientation::Reversed)?);
    let skin_wire = builder.add_wire(&[skin_step])?;
    let skin = builder.add_face(Surface::Cone(a_cone_along_x()), &[skin_wire])?;
    builder.add_shell(&[(lid, Orientation::Forward), (skin, Orientation::Forward)])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

fn a_cone_along_x() -> Cone {
    Cone::new(Axis::x(), SLOPE).expect("a positive slope")
}

/// The prism on `outline` from `z = bottom` up to `top`.
fn a_prism(model: &mut Model, outline: &[[f64; 2]], bottom: f64, top: f64) -> SolidKey {
    let outline = ProfileBuilder::new(outline.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("an outline is a profile");
    let placed = Isometry::translation(Vector::new(0.0, 0.0, bottom)).expect("a finite move");
    model
        .add_extrusion(&outline, top - bottom, &placed)
        .expect("an outline extrudes")
}

/// The block `[−6, 6] × [−12, 4] × [0, 6]` turned about `x` until its floor
/// lies on `2y − z = 0`, as near as the turn's rounding lets it.
fn a_block_turned_onto_the_ruling(model: &mut Model) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![[-6.0, -12.0], [6.0, -12.0], [6.0, 4.0], [-6.0, 4.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    let turn = Isometry::rotation(&Axis::x(), f64::atan2(-2.0, -1.0)).expect("a finite turn");
    model
        .add_extrusion(&rectangle, 6.0, &turn)
        .expect("a rectangle extrudes")
}

/// The crossing is one straight run: a plane touching a cone along the whole
/// of one ruling.
fn touches_along_one_ruling(meet: &rcad_geom::Result<SurfaceMeet>) -> bool {
    let Ok(SurfaceMeet::Curves(runs)) = meet else {
        return false;
    };
    matches!(&runs[..], [run] if run.run() == &EdgeRun::Straight)
}

/// The cone of slope `0.75` and the ball of radius 5 about `(0, −4, 3)`, kept
/// in common: the cone's skin inside the ball and the ball's skin inside the
/// cone, either side of their one closed crossing, cornered at the apex.
fn the_cone_and_the_ball_in_common(
    model: &mut Model,
    cone_way: Orientation,
) -> rcad_topo::Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let apex = builder.add_vertex(Point::origin())?;
    let rim = builder.add_edge(apex, apex, EdgeRun::RightRoundWhereItsFacesCross)?;
    let on_the_cone = DraftStep::Along(builder.add_coedge(rim, cone_way)?);
    let on_the_ball = DraftStep::Along(builder.add_coedge(rim, cone_way.reversed())?);
    let cone_wire = builder.add_wire(&[on_the_cone])?;
    let ball_wire = builder.add_wire(&[on_the_ball])?;
    let cone = builder.add_face(
        Surface::Cone(Cone::new(Axis::z(), 0.75).expect("a positive slope")),
        &[cone_wire],
    )?;
    let ball = builder.add_face(
        Surface::Sphere(Sphere::new(Point::new(0.0, -4.0, 3.0), 5.0).expect("a ball")),
        &[ball_wire],
    )?;
    builder.add_shell(&[(cone, Orientation::Forward), (ball, Orientation::Forward)])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

fn six_fuses() -> [(Rule, bool); 6] {
    [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Intersection, true),
        (Rule::Intersection, false),
        (Rule::Difference, true),
        (Rule::Difference, false),
    ]
}

fn fuse(
    model: &mut Model,
    cone: SolidKey,
    other: SolidKey,
    rule: Rule,
    cone_first: bool,
) -> rcad_topo::Result<Vec<SolidKey>> {
    let (arguments, tools) = if cone_first {
        ([cone], [other])
    } else {
        ([other], [cone])
    };
    model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().to_vec())
}

/// The solid has `shells` shells and meshes into that many closed surfaces,
/// each with the shape of a ball: every side of a triangle is used once each
/// way, and points less sides plus triangles is two for each shell.
fn meshes_closed(model: &Model, solid: SolidKey, shells: usize, what: &str) {
    let view = model.solid(solid).expect("this model made the key");
    assert_eq!(view.shells().count(), shells, "{what}: shells");
    let mesh = view
        .mesh(
            Deflection::of(0.01).expect("a positive length"),
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
        2 * shells as i64,
        "{what}: {points} points, {} sides, {} triangles",
        times_used.len(),
        mesh.triangles().len()
    );
}

/// The solid's volume and the error its measure publishes.
fn measured(model: &Model, solid: SolidKey, what: &str) -> (f64, f64) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the solid measures: {why:?}"));
    (measured.volume(), measured.volume_error())
}
