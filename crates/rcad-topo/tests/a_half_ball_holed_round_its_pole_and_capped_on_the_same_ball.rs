//! The half ball `z ≥ 0` of radius 5, its skin holed along the plane
//! `−y + 7z = 25` and the hole capped on the same ball, built face by face.
//!
//! The plane cuts the ball in a circle whose highest point is `(0, 3, 4)` and
//! whose lowest is `(0, −4, 3)`. The north pole is on the far side of it from
//! the centre, so the cap holds the pole and the skin is a band between the
//! equator and the hole. The solid has three faces: the flat floor, the cap,
//! and the skin. The rim between the skin and the cap has the ball on both
//! sides, so it is written `RightRoundWhereItCrosses { and: the plane }`.
//!
//! **Why it was refused.** The mesher cuts a cap up to the pole it holds, and
//! `which_pole_this_cap_reaches` in `mesh.rs` once read that pole off the
//! surface across the cap's rim. Across this rim is the same ball, so there was
//! nothing to read, and Q374 found every right spelling refused
//! `ACurvedFaceGoesRightRoundWithNoSeam { cap }`. Q508 (`dfa75f1e`) reads the
//! pole off the walk round the rim instead, and the shape answers with no
//! change of its own (Q895). With the old reading put back in that function,
//! `SolidBuilder::build` refuses the first right spelling that way again.
//!
//! **Which spellings are right.** Both corners are off the pole, where the
//! ball's longitude says which way the rim is stored. The cap walks the rim
//! `Forward` and the skin walks it `Reversed`, whichever way the plane faces and
//! whichever of the skin's two wires is listed first. Of the other twelve
//! spellings at each corner, the eight that walk the rim the same way on both
//! faces are refused `ADraftEdgeIsWalkedTheSameWayTwice` and the four with the
//! skin's walk and the cap's swapped are refused
//! `AClosedEdgeIsWalkedAgainstItsSurface`.
//!
//! The oracle is half the ball's volume, `250π/3 = 261.799388`: the hole and
//! the cap filling it take nothing away.

use core::f64::consts::PI;
use std::collections::HashMap;

use rcad_geom::{Deflection, EdgeRun, Orientation, Plane, Point, Scale, Sphere, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, Result, SolidBuilder, SolidKey};

const RADIUS: f64 = 5.0;

/// The rim's corner at the circle's highest point and at its lowest, each with
/// the plane facing either way and the skin's hole listed after its equator
/// and before it.
#[test]
fn every_right_spelling_meshes_closed_and_measures_half_the_ball() {
    let closed_form = 2.0 * PI * RADIUS.powi(3) / 3.0;
    assert_eq!(format!("{closed_form:.6}"), "261.799388");
    for (corner, and, hole_first) in every_corner_plane_and_wire_order() {
        let what = format!(
            "corner {corner:?}, and {:?}, hole first {hole_first}",
            and.normal()
        );
        let mut model = Model::new(Scale::millimetres(125.0).expect("a positive size"));
        let solid = a_holed_half_ball(
            &mut model,
            corner,
            and,
            [Orientation::Reversed, Orientation::Forward],
            hole_first,
        )
        .unwrap_or_else(|why| panic!("{what}: a holed and capped half ball is a solid: {why:?}"));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what}: a holed and capped half ball is sound: {why:?}"));
        for deflection in [1.0, 0.1, 0.01, 0.005] {
            meshes_closed_and_measures(
                &model,
                solid,
                deflection,
                closed_form,
                &format!("{what}, deflection {deflection}"),
            );
        }
    }
}

/// The skin walking the rim the way the cap does, or each walking it the way
/// the other should, is refused by name in every placing of the corner, the
/// plane and the wires.
#[test]
fn every_wrong_spelling_is_refused_by_name() {
    let mut refused = 0;
    for (corner, and, hole_first) in every_corner_plane_and_wire_order() {
        for walks in [
            [Orientation::Forward, Orientation::Forward],
            [Orientation::Reversed, Orientation::Reversed],
            [Orientation::Forward, Orientation::Reversed],
        ] {
            let mut model = Model::new(Scale::millimetres(125.0).expect("a positive size"));
            let got = a_holed_half_ball(&mut model, corner, and, walks, hole_first);
            let named = if walks[0] == walks[1] {
                matches!(
                    got,
                    Err(Error::ADraftEdgeIsWalkedTheSameWayTwice { edge: 1 })
                )
            } else {
                matches!(
                    got,
                    Err(Error::Invalid(
                        Invalid::AClosedEdgeIsWalkedAgainstItsSurface
                    ))
                )
            };
            let what = format!(
                "corner {corner:?}, and {:?}, hole first {hole_first}, skin and cap walk the rim \
                 {walks:?}: {got:?}",
                and.normal()
            );
            assert!(named, "{what}");
            println!("{what}");
            refused += 1;
        }
    }
    assert_eq!(refused, 24);
}

/// The rim's corner at `(0, 3, 4)` and at `(0, −4, 3)`, the plane `−y + 7z = 25`
/// facing either way, and the skin's hole listed after its equator or before it.
fn every_corner_plane_and_wire_order() -> Vec<(Point, Plane, bool)> {
    let plane =
        Plane::from_point_and_normal(Point::new(0.0, 3.0, 4.0), Vector::new(0.0, -1.0, 7.0))
            .expect("a plane");
    let mut all = Vec::new();
    for corner in [Point::new(0.0, 3.0, 4.0), Point::new(0.0, -4.0, 3.0)] {
        for and in [plane, plane.reversed()] {
            for hole_first in [false, true] {
                all.push((corner, and, hole_first));
            }
        }
    }
    all
}

/// The half ball `z ≥ 0` with its skin holed along `and` and the hole capped
/// on the same ball, the rim's one corner at `corner`, walked as `walks` says
/// by the skin and then by the cap. Built and validated by
/// `SolidBuilder::build`.
fn a_holed_half_ball(
    model: &mut Model,
    corner: Point,
    and: Plane,
    walks: [Orientation; 2],
    hole_first: bool,
) -> Result<SolidKey> {
    let [hole_on_the_skin, rim_of_the_cap] = walks;
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
    let corner = builder.add_vertex(corner).expect("a point");
    let rim = builder
        .add_edge(
            corner,
            corner,
            EdgeRun::RightRoundWhereItCrosses {
                and: Surface::Plane(and),
            },
        )
        .expect("a closed edge");
    let mut walking = |edge, orientation| {
        let coedge = builder
            .add_coedge(edge, orientation)
            .expect("an edge of this builder");
        builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("a closed wire")
    };
    let equator_on_the_skin = walking(equator, Orientation::Forward);
    let equator_on_the_floor = walking(equator, Orientation::Reversed);
    let hole = walking(rim, hole_on_the_skin);
    let rim_of_the_cap = walking(rim, rim_of_the_cap);
    let ball = Surface::Sphere(Sphere::new(Point::origin(), RADIUS).expect("a ball"));
    let skin_wires = if hole_first {
        [hole, equator_on_the_skin]
    } else {
        [equator_on_the_skin, hole]
    };
    let skin = builder
        .add_face(ball, &skin_wires)
        .expect("a face with two wires");
    let cap = builder
        .add_face(ball, &[rim_of_the_cap])
        .expect("a face with one wire");
    let floor = builder
        .add_face(
            Surface::Plane(
                Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
                    .expect("a plane"),
            ),
            &[equator_on_the_floor],
        )
        .expect("a face with one wire");
    builder.add_shell(&[
        (skin, Orientation::Forward),
        (cap, Orientation::Forward),
        (floor, Orientation::Forward),
    ])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The solid meshes into a closed surface with the shape of a ball, its volume
/// is under `closed_form` by no more than `3δ/r` of it, and `closed_form` is
/// within the error the measure publishes.
///
/// Closed means every side of every triangle is used once each way round; the
/// shape of a ball means points less sides plus triangles is two. The `3δ/r`
/// bound is the one `a_half_ball_holed_through_its_pole_and_capped_on_the_same_ball.rs`
/// takes from `a_cap_whose_rim_runs_through_the_other_pole.rs`: the missing
/// shell is at most the curved area times `δ`, and a piece holding the ball's
/// centre has volume at least that area times `r/3`.
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
    println!(
        "{what}: measured {found:.6} against {closed_form:.6}, published error {:.6}",
        measured.volume_error()
    );
    let under = (closed_form - found) / closed_form;
    assert!(
        (0.0..3.0 * deflection / RADIUS).contains(&under),
        "{what}: measured {found}, closed form {closed_form}, {:.4}% under",
        100.0 * under
    );
    assert!(
        (closed_form - found).abs() <= measured.volume_error(),
        "{what}: measured {found}, closed form {closed_form}, published error {}",
        measured.volume_error()
    );
}
