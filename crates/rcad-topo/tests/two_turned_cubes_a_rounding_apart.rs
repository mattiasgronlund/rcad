//! Two cubes of side two that a turn leaves a rounding apart (queue rows Q397,
//! Q676 and Q991).
//!
//! The second cube is moved by `(2, 0.5, 0.25)`, so upright the two touch face
//! to face, and both are turned about the axis through `(1, 0, 0)` along
//! `(1, 2, 3)`. The first cube's face `x = 2` and the second's face `x = 0`
//! then face exactly opposite ways, and their offsets no longer quite cancel:
//! at 1.1 and 2.5 they leave a gap of `2^-51` and `2^-52`, and at 0.3 the two
//! faces overlap by `2^-52`.
//!
//! **What it took to answer.** Every fuse used to refuse at the first question:
//! which side of the second cube's face `x = 0` a corner of the first cube lies
//! on, asked of the corner's box, which the face runs straight through.
//! `Arrangement::passes_through` asks the three planes the corner is the meet
//! of instead, and they decide it (Q397). The fuse then refused one step on, in
//! `Arrangement::order_along`, ordering that corner and the point a rounding
//! past it where the other face crosses the corner's edge. Both are meets of
//! three planes that share two, and the planes now put them in order too
//! (Q676, `which_meet_comes_first_along_a_line`).
//!
//! Every fuse then stopped in selection. The first cube's face `x = 2` comes
//! out of the arrangement whole, and so does the second's face `x = 0`; each
//! has to be placed against the other cube. The point picked inside it lies on
//! its own plane and so a rounding off the other cube's face, and the ray cast
//! from that point's box into the other cube cannot say which side of that
//! face it starts on. Selection now places such a face by a plane instead
//! (`by_a_plane` in `fuse/select.rs`, Q991): the other cube has every corner on
//! or behind its own touching face, and the piece has every corner in front of
//! it, each corner asked as the meet of three planes, exactly.
//! [`every_corner_of_each_cube_is_in_front_of_the_others_touching_face_only_when_apart`]
//! asks the same question by hand, and
//! [`every_fuse_of_two_cubes_a_rounding_apart_gives_back_the_cubes_as_they_stand`]
//! pins the five answers.
//!
//! **0.3 is a limit.** The faces overlap by `2^-52`, so the union and the
//! intersection keep a sliver whose edges are about `2.2e-16` long at a
//! coordinate of about `2.18`, a spread of `1e16` against the `1e11` a model
//! supports. It is pinned by name here and is to be refused whatever the
//! later stages learn.

use rcad_geom::{Axis, Deflection, Isometry, Plane, Point, Scale, Vector};
use rcad_predicates::{Decision, Policy, ProvenanceLog, side_of_a_plane_where_three_planes_meet};
use rcad_topo::{Error, Invalid, Measurement, Model, Rule, SolidKey};

/// The turns at which the two touching faces leave a gap, and what their
/// offsets add up to there: `-2^-51` and `-2^-52`.
const A_ROUNDING_APART: [(f64, f64); 2] = [(1.1, -2.0 * f64::EPSILON), (2.5, -f64::EPSILON)];

/// The turn at which the two faces overlap instead, by `2^-52`.
const A_ROUNDING_INTO_EACH_OTHER: f64 = 0.3;

#[test]
fn the_touching_faces_lie_a_rounding_apart_at_1_1_and_2_5_and_overlap_at_0_3() {
    for (angle, offsets_add_to) in A_ROUNDING_APART {
        assert_eq!(
            what_the_touching_offsets_add_to(angle),
            offsets_add_to,
            "{angle}"
        );
    }
    assert_eq!(
        what_the_touching_offsets_add_to(A_ROUNDING_INTO_EACH_OTHER),
        f64::EPSILON
    );
}

/// **At 1.1 and 2.5 the cubes share no point, and every fuse says so.** The
/// union is the two cubes, each less the other is itself, and the
/// intersection is nothing. Each solid of an answer is held against the input
/// it is: one face on each of that cube's six planes, bit for bit, each corner
/// within the slop the fuse tagged it with of a corner of that cube, and its
/// volume within what the two measurements say they may be out by and what
/// those corners' moves can add.
///
/// **The corners move, so the volume is given room for it.** The fuse writes
/// each corner down again from the planes it is the meet of, a rounding or so
/// from where the input had it, and a measurement's own error is only the
/// rounding of its sums: at 2.5 the second cube came back `5e-15` smaller,
/// where the two errors add to `7e-16`. Moving the corners of a closed skin
/// of triangles by at most `d` changes what it encloses by at most its area
/// times `d`, near enough, since the volume changes with a corner by a third
/// of the area of the triangles round it, turned along their normals.
///
/// Before Q991 every one of these was refused at `a point in a piece that
/// every solid could place`, since Q397 and Q676 got the fuse through the
/// arrangement.
#[test]
fn every_fuse_of_two_cubes_a_rounding_apart_gives_back_the_cubes_as_they_stand() {
    for (angle, _) in A_ROUNDING_APART {
        let (mut model, a, b) = two_turned_cubes(angle);
        let as_it_stands = |solid: SolidKey| {
            (
                the_planes_of(&model, solid),
                the_corners_of(&model, solid),
                measured(&model, solid),
            )
        };
        let (cube_a, cube_b) = (as_it_stands(a), as_it_stands(b));
        for (what, left, right, rule, mut want) in [
            ("a union b", a, b, Rule::Union, vec![&cube_a, &cube_b]),
            ("b union a", b, a, Rule::Union, vec![&cube_a, &cube_b]),
            ("a and b", a, b, Rule::Intersection, vec![]),
            ("a less b", a, b, Rule::Difference, vec![&cube_a]),
            ("b less a", b, a, Rule::Difference, vec![&cube_b]),
        ] {
            let answer = model
                .fuse(
                    &[left],
                    &[right],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{angle} {what} refused: {why:?}"))
                .solids()
                .to_vec();
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{angle} {what} does not validate: {why}"));
            for solid in answer {
                let planes = the_planes_of(&model, solid);
                let at = want
                    .iter()
                    .position(|(theirs, _, _)| *theirs == planes)
                    .unwrap_or_else(|| {
                        panic!("{angle} {what}: a solid on {planes:?} is no cube still wanted")
                    });
                let (_, corners, cube) = want.swap_remove(at);
                let mut moved = 0.0_f64;
                for (corner, slop) in the_corners_of(&model, solid) {
                    let nearest = corners
                        .iter()
                        .map(|(theirs, _)| corner.distance_to(*theirs))
                        .fold(f64::INFINITY, f64::min);
                    assert!(
                        nearest <= slop,
                        "{angle} {what}: a corner moved {nearest:e}, tagged {slop:e}"
                    );
                    moved = moved.max(nearest);
                }
                let found = measured(&model, solid);
                assert!(
                    (found.volume() - cube.volume()).abs()
                        <= found.volume_error()
                            + cube.volume_error()
                            + found.surface_area() * moved,
                    "{angle} {what}: volume {} against {}, corners moved {moved:e}",
                    found.volume(),
                    cube.volume()
                );
            }
            assert!(want.is_empty(), "{angle} {what}: a cube is missing");
        }
    }
}

/// **At 1.1 and 2.5 every answer exists.** A cube is where all six of its
/// planes have it behind them, so it lies in the hull of its eight corners,
/// each the meet of three of its planes. Asked of those planes exactly
/// ([`side_of_a_plane_where_three_planes_meet`](rcad_predicates::side_of_a_plane_where_three_planes_meet)),
/// every corner of each cube is in front of the other cube's touching face, so
/// the two share no point: the union is the two as they stand, each less the
/// other is itself, and the intersection is nothing. At 0.3 the four corners
/// on each touching face are behind the other's, which is the sliver.
///
/// A point picked inside the touching face lies on its plane and so only
/// `2^-51` from the other face, and the ray from it cannot say which side it
/// starts on (the module doc). This is the question the fuse now puts to the
/// planes instead.
#[test]
fn every_corner_of_each_cube_is_in_front_of_the_others_touching_face_only_when_apart() {
    let in_front = [Decision::Pos; 8];
    for (angle, _) in A_ROUNDING_APART {
        assert_eq!(
            sides_of_the_corners_against_the_other_touching_face(angle),
            [in_front, in_front],
            "{angle}"
        );
    }
    for corners in sides_of_the_corners_against_the_other_touching_face(A_ROUNDING_INTO_EACH_OTHER)
    {
        let behind = corners
            .iter()
            .filter(|side| **side == Decision::Neg)
            .count();
        assert_eq!(behind, 4, "{corners:?}");
    }
}

/// A limit, not a gap: every answer needs an edge `2.2e-16` long at `2.18`,
/// and every fuse is now refused for just that, `DynamicRangeTooWide` at a
/// spread of `3.9e16` to `7.5e16`.
///
/// It used to stop sooner, in selection, at the search for a point inside a
/// piece. Q991 and Q1288 placed the strips `2^-52` wide on the first cube's
/// bottom, and on its top from `y = 0` to `0.5`, outside the second cube by
/// one of its planes. The last stop was the rest of the top strip, from
/// `y = 0.5` to `2`, inside the second cube. Since Q1353 `by_a_plane` places a
/// piece inside a solid when every plane of the solid has every corner of the
/// piece on or behind it and one strictly behind (`docs/decisions.md` §417),
/// which that strip has, so every fuse gets as far as writing the sliver and
/// is refused for the edge it needs.
#[test]
fn two_cubes_overlapping_by_a_rounding_are_refused_for_the_sliver_between_them() {
    let (mut model, a, b) = two_turned_cubes(A_ROUNDING_INTO_EACH_OTHER);
    for (what, left, right, rule) in [
        ("a union b", a, b, Rule::Union),
        ("b union a", b, a, Rule::Union),
        ("a and b", a, b, Rule::Intersection),
        ("a less b", a, b, Rule::Difference),
        ("b less a", b, a, Rule::Difference),
    ] {
        let outcome = model.fuse(
            &[left],
            &[right],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert!(
            matches!(
                outcome,
                Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) if ratio > 1e16
            ),
            "{what} is refused for an edge a rounding long, and came back {outcome:?}"
        );
    }
}

fn two_turned_cubes(angle: f64) -> (Model, SolidKey, SolidKey) {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let a = model
        .add_cube(2.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let b = model
        .add_cube(2.0, &policy, &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let turn = Isometry::rotation(
        &Axis::new(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 2.0, 3.0))
            .expect("a direction with length"),
        angle,
    )
    .expect("a rotation");
    model
        .place_solid(a, &turn)
        .expect("this model made the key");
    model
        .place_solid(
            b,
            &turn.compose(&Isometry::translation(Vector::new(2.0, 0.5, 0.25)).expect("a shift")),
        )
        .expect("this model made the key");
    (model, a, b)
}

/// The plane of each face of `solid`, as coefficients, in one fixed order.
fn the_planes_of(model: &Model, solid: SolidKey) -> Vec<[f64; 4]> {
    let mut planes: Vec<[f64; 4]> = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .map(|face| face.plane().coefficients())
        .collect();
    planes.sort_by(|one, other| {
        one.iter()
            .zip(other)
            .map(|(one, other)| one.total_cmp(other))
            .find(|order| order.is_ne())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    planes
}

/// Where each corner of `solid` is, with the slop a decision put into it, or
/// `0.0` where none did, once for every face it is a corner of.
fn the_corners_of(model: &Model, solid: SolidKey) -> Vec<(Point, f64)> {
    let mut corners = Vec::new();
    for face in model.solid(solid).expect("this model made the key").faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let corner = coedge.start();
                let slop = corner
                    .approximation()
                    .map_or(0.0, |approximation| approximation.magnitude());
                corners.push((corner.point(), slop));
            }
        }
    }
    corners
}

/// The volume and skin of one solid, each with how far it may be out.
fn measured(model: &Model, solid: SolidKey) -> Measurement {
    model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("a solid does not measure: {why}"))
}

/// The offsets of the one pair of faces, one from each cube, that face
/// exactly opposite ways and nearly touch, added up.
fn what_the_touching_offsets_add_to(angle: f64) -> f64 {
    let (model, a, b) = two_turned_cubes(angle);
    let mut sums = Vec::new();
    for mine in model.solid(a).expect("this model made the key").faces() {
        for theirs in model.solid(b).expect("this model made the key").faces() {
            let (mine, theirs) = (mine.plane(), theirs.plane());
            if mine.normal().components() == (-theirs.normal()).components()
                && (mine.offset() + theirs.offset()).abs() < 1e-12
            {
                sums.push(mine.offset() + theirs.offset());
            }
        }
    }
    let [only] = sums[..] else {
        panic!("{angle}: one pair of faces nearly touches, and {sums:?} do");
    };
    only
}

/// For each cube, which side of the other cube's touching face each of its
/// eight corners lies on, worked out from the planes the corner is the meet
/// of. [`Decision::Pos`] is in front of that face, outside the other cube.
///
/// A corner is any three of the cube's six planes that meet in a point: one
/// of each pair of opposite faces. Three that take both of one pair are
/// refused as meeting nowhere, which leaves eight.
fn sides_of_the_corners_against_the_other_touching_face(angle: f64) -> [[Decision; 8]; 2] {
    let (model, a, b) = two_turned_cubes(angle);
    let planes_of = |solid: SolidKey| -> Vec<Plane> {
        model
            .solid(solid)
            .expect("this model made the key")
            .faces()
            .map(|face| face.plane())
            .collect()
    };
    let (mine, theirs) = (planes_of(a), planes_of(b));
    let touching = |one: &[Plane], other: &[Plane]| -> Plane {
        let mut found = other.iter().filter(|plane| {
            one.iter().any(|mine| {
                mine.normal().components() == (-plane.normal()).components()
                    && (mine.offset() + plane.offset()).abs() < 1e-12
            })
        });
        let (Some(only), None) = (found.next(), found.next()) else {
            panic!("{angle}: one face of the other cube touches");
        };
        *only
    };
    [
        (&mine, touching(&mine, &theirs)),
        (&theirs, touching(&theirs, &mine)),
    ]
    .map(|(planes, face)| {
        let mut sides = Vec::new();
        for first in 0..planes.len() {
            for second in first + 1..planes.len() {
                for third in second + 1..planes.len() {
                    let three = [first, second, third].map(|at| planes[at].coefficients());
                    if let Ok(side) =
                        side_of_a_plane_where_three_planes_meet(three, face.coefficients())
                    {
                        sides.push(side);
                    }
                }
            }
        }
        sides
            .try_into()
            .unwrap_or_else(|sides| panic!("{angle}: a cube has eight corners, not {sides:?}"))
    })
}
