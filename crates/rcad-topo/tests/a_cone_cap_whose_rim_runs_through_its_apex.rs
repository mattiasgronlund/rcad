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
//! print there stayed silent through every fuse and build here, and through
//! the whole workspace and the slow tier. With its two comparisons made
//! `<=` and `>=` instead, every answer here came out the same.
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
//! reaches its own axis. A plane through the apex cuts the cone in two rulings,
//! or touches it along one. The only other edge left is a straight ruling
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
//! - **A plane along one ruling.** A `Plane` stores a unit normal, and one with
//!   two components that are not zero is never exact in binary, since
//!   `a² + b² = 4ᵏ` has no answer in whole numbers but with `a` or `b` zero. So
//!   the block resting on the cone along a ruling cuts it a rounding deep
//!   instead, and every fuse is refused `Undecidable` before anything is
//!   meshed.
//! - **A closed edge cornered at the apex.** It has to run along a closed
//!   crossing through the apex, and every one is refused by name, through a
//!   fuse and through `SolidBuilder` alike: a bar whose wall runs through the
//!   apex, and a ball through the apex whose crossing with the cone has a cusp
//!   there.

use core::f64::consts::PI;
use std::collections::HashMap;

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Sphere, Surface,
    Vector,
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
            meshes_closed(&model, solid, &what);
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
            let prism = a_prism_standing_on_the_axis(&mut model, outline, top);
            let solids = fuse(&mut model, cone, prism, rule, cone_first)
                .unwrap_or_else(|why| panic!("{what}: refused {why:?}"));
            let [solid] = solids[..] else {
                panic!("{what}: {} solids", solids.len())
            };
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: validates: {why:?}"));
            meshes_closed(&model, solid, &what);
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
/// would touch the cone along the ruling `x = 0, y = z/2`, and every fuse is
/// refused `Undecidable`, at "a point inside a piece of a face", before
/// anything is meshed.
///
/// The floor's normal is `(0, −2, 1)/√5`, which rounds, so the plane as
/// stored cuts the cone in two rulings a rounding apart rather than touching
/// it along one. The points the fuse could not place are `1.7e-7` either side
/// of `x = 0` near the far end of that ruling.
#[test]
fn a_block_resting_on_the_cone_along_a_ruling_is_refused_before_any_mesh() {
    for (rule, cone_first) in six_fuses() {
        let mut model = a_model();
        let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);
        let rectangle =
            ProfileBuilder::new(vec![[-6.0, -12.0], [6.0, -12.0], [6.0, 4.0], [-6.0, 4.0]])
                .build(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("a rectangle is a profile");
        let turn = Isometry::rotation(&Axis::x(), f64::atan2(-2.0, -1.0)).expect("a finite turn");
        let block = model
            .add_extrusion(&rectangle, 6.0, &turn)
            .expect("a rectangle extrudes");
        let said = fuse(&mut model, cone, block, rule, cone_first);
        assert!(
            matches!(said, Err(Error::Undecidable { .. })),
            "{rule:?}, cone first {cone_first}: {said:?}"
        );
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

/// The prism on `outline` from `z = −2` up to `top`, with a corner of the
/// outline at the origin, so that its edge there runs up the cone's axis.
fn a_prism_standing_on_the_axis(model: &mut Model, outline: &[[f64; 2]], top: f64) -> SolidKey {
    let outline = ProfileBuilder::new(outline.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("an outline is a profile");
    let placed = Isometry::translation(Vector::new(0.0, 0.0, -2.0)).expect("a finite move");
    model
        .add_extrusion(&outline, top + 2.0, &placed)
        .expect("an outline extrudes")
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

/// The solid meshes into a closed surface with the shape of a ball: every
/// side of a triangle is used once each way, and points less sides plus
/// triangles is two.
fn meshes_closed(model: &Model, solid: SolidKey, what: &str) {
    let mesh = model
        .solid(solid)
        .expect("this model made the key")
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
        2,
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
