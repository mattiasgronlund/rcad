//! Flat faces cut by a bar's pipe, and a bar's wall cut by flat faces: which
//! of them say what points they hold, and which are refused by name.
//!
//! A cap keeping the **inside** of the pipe is answered: the inside of a pipe is
//! every point within its radius of the axis, which is convex, and a convex
//! piece is what `holds.rs`'s `is_a_family_this_follows` needs. A wall cut by
//! two planes along the axis is answered when the two meet inside the pipe — a
//! quarter of a bar — and refused when they do not, which is a bar with two
//! flats. A flat face keeping the **outside** of the pipe is refused. Each
//! refusal comes with the shape that shows why: two points the face's own cuts
//! cannot tell apart, one on the face and one on another face.

use core::f64::consts::PI;

use rcad_geom::{Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, FaceKey, Model, OnAFace, ProfileBuilder, Rule, SolidKey};

/// A bar of radius 5 from `z = 0` to `z = 4`, less the part outside `x ≥ 0` and
/// `y ≥ 0`: its wall is cut by two planes along the axis that meet on the
/// axis, and is the quarter of the pipe between them and the caps.
#[test]
fn a_quarter_of_a_bar_holds_the_points_of_its_wall() {
    for bar_first in [true, false] {
        let (model, solid) = a_quarter_of_a_bar(bar_first);
        let wall = the_only_face(&model, solid, |surface| {
            matches!(surface, Surface::Cylinder(_))
        });
        let mut held = 0;
        let mut not_held = 0;
        for degrees in [-15.0, 15.0, 45.0, 75.0, 105.0, 180.0, 255.0] {
            for z in [-0.5, 0.5, 2.0, 3.5, 4.5] {
                let on_the_wall = 0.0 < degrees && degrees < 90.0 && 0.0 < z && z < 4.0;
                let expected = if on_the_wall {
                    held += 1;
                    OnAFace::Held
                } else {
                    not_held += 1;
                    OnAFace::NotHeld
                };
                assert_eq!(
                    holds(&model, solid, wall, on_the_pipe(5.0, degrees, z)),
                    Ok(expected),
                    "the quarter's wall, bar first {bar_first}, at {degrees}° and z = {z}"
                );
            }
        }
        assert_eq!((held, not_held), (9, 26));
    }
}

/// The whole quarter, asked point by point: the ray caster crosses the wall,
/// so a wall that refuses makes the solid refuse.
#[test]
fn a_quarter_of_a_bar_says_which_points_are_inside_it() {
    for bar_first in [true, false] {
        let (model, solid) = a_quarter_of_a_bar(bar_first);
        let view = model.solid(solid).expect("this model made the key");
        for (point, expected) in [
            ([1.0, 1.0, 2.0], Containment::Inside),
            ([3.0, 3.0, 1.0], Containment::Inside),
            ([4.5, 1.0, 3.0], Containment::Inside),
            ([-1.0, 1.0, 2.0], Containment::Outside),
            ([1.0, -1.0, 2.0], Containment::Outside),
            ([4.0, 4.0, 2.0], Containment::Outside),
            ([1.0, 1.0, 4.5], Containment::Outside),
        ] {
            let mut log = ProvenanceLog::new();
            let [x, y, z] = point;
            assert_eq!(
                view.classify_point(Point::new(x, y, z), &Policy::certified(), &mut log),
                expected,
                "the quarter, bar first {bar_first}, at {point:?}"
            );
            assert!(log.is_empty(), "a certified answer logs nothing: {log:?}");
        }
    }
}

/// The quarter's roof is a flat face cut by the pipe, keeping its inside, and
/// by the two planes: a quarter of a disc.
#[test]
fn a_cap_trimmed_by_a_pipe_holds_its_points() {
    for bar_first in [true, false] {
        let (model, solid) = a_quarter_of_a_bar(bar_first);
        let roof = the_only_face(&model, solid, |surface| {
            surface
                .as_plane()
                .is_some_and(|flat| flat.normal().components() == [0.0, 0.0, 1.0])
        });
        for (x, y) in [(1.0, 1.0), (3.0, 1.0), (1.0, 3.0), (2.5, 2.5), (4.5, 0.5)] {
            assert_eq!(
                holds(&model, solid, roof, [x, y, 4.0]),
                Ok(OnAFace::Held),
                "the roof, bar first {bar_first}, at ({x}, {y})"
            );
        }
        for (x, y) in [(-1.0, 1.0), (1.0, -1.0), (4.0, 4.0), (-2.0, -2.0)] {
            assert_eq!(
                holds(&model, solid, roof, [x, y, 4.0]),
                Ok(OnAFace::NotHeld),
                "the roof, bar first {bar_first}, at ({x}, {y})"
            );
        }
    }
}

/// A bar of radius 5 with two flats, `|x| ≤ 3`. Each wall is one run of the
/// pipe's rulings, round `+y` or round `−y`, cut by both flats and both caps —
/// and the other wall is in front of all four cuts too. "The part of the pipe
/// in front of every cut" is both walls, so it cannot be how either is read,
/// and both are refused by name rather than answered wrong. Each wall's answer
/// exists, so under `docs/decisions.md` §16.10 this is a gap: one of the
/// argument and not of the face.
#[test]
fn a_bar_with_two_flats_has_each_wall_refused_by_name() {
    let mut model = a_model();
    let bar = a_bar(&mut model, 5.0, [0.0, 0.0, 0.0]);
    let slab = a_block(&mut model, [-3.0, -10.0, -1.0], [6.0, 20.0, 6.0]);
    let solid = the_one_solid(&mut model, bar, slab, Rule::Intersection);
    let walls = the_faces(&model, solid, |surface| {
        matches!(surface, Surface::Cylinder(_))
    });
    assert_eq!(walls.len(), 2, "one wall each side of the flats");
    for wall in walls {
        for degrees in [90.0, 270.0] {
            assert_eq!(
                holds(&model, solid, wall, on_the_pipe(5.0, degrees, 2.0)),
                Err(wall)
            );
        }
        assert_eq!(
            sides_of_every_neighbour(&model, solid, wall, [0.0, 5.0, 2.0]),
            sides_of_every_neighbour(&model, solid, wall, [0.0, -5.0, 2.0]),
            "the two walls are on the same side of every cut of one of them"
        );
    }
}

/// A block `2 × 12 × 4` with a bar of radius 3 bored down into its middle as
/// far as `z = 2`. The bar's circle runs into the top at one long side and out
/// at the other, so the top is two faces, each keeping the outside of the pipe
/// — and the other piece is in front of every one of this piece's cuts.
/// Refused by name rather than answered wrong; the answer exists, so this is a
/// gap of the argument as the two flats are.
#[test]
fn a_block_with_a_bar_bored_into_it_has_its_top_refused_by_name() {
    let mut model = a_model();
    let block = a_block(&mut model, [-1.0, -6.0, 0.0], [2.0, 12.0, 4.0]);
    let bar = a_bar(&mut model, 3.0, [0.0, 0.0, 2.0]);
    let solid = the_one_solid(&mut model, block, bar, Rule::Difference);
    let tops = the_faces(&model, solid, |surface| {
        surface.as_plane().is_some_and(|flat| {
            flat.normal().components() == [0.0, 0.0, 1.0] && flat.offset() == 4.0
        })
    });
    assert_eq!(tops.len(), 2, "the top either side of the bore");
    for top in tops {
        for y in [4.5, -4.5] {
            assert_eq!(holds(&model, solid, top, [0.0, y, 4.0]), Err(top));
        }
        assert_eq!(
            sides_of_every_neighbour(&model, solid, top, [0.0, 4.5, 4.0]),
            sides_of_every_neighbour(&model, solid, top, [0.0, -4.5, 4.0]),
            "the two pieces of the top are on the same side of every cut of one of them"
        );
    }
}

/// A block `10 × 10 × 4` with a bar of radius 3 bored down through the middle
/// of one edge. Its floor keeps the outside of the pipe and is **one piece**,
/// the square less half a disc, so `(5, 5, 0)` is held and `(1, 5, 0)` is not.
/// It is refused by name all the same: what `is_a_family_this_follows` asks
/// cannot tell it from the top of the block above, where the same cuts leave
/// two pieces. A gap, pinned so that answering it fails here and gets moved to
/// a test of its answers.
#[test]
fn a_block_with_a_bar_bored_through_one_edge_has_its_floor_refused_though_it_is_one_piece() {
    let mut model = a_model();
    let block = a_block(&mut model, [0.0, 0.0, 0.0], [10.0, 10.0, 4.0]);
    let bar = a_bar(&mut model, 3.0, [0.0, 5.0, -1.0]);
    let solid = the_one_solid(&mut model, block, bar, Rule::Difference);
    let floor = the_only_face(&model, solid, |surface| {
        surface
            .as_plane()
            .is_some_and(|flat| flat.normal().components() == [0.0, 0.0, -1.0])
    });
    for point in [[5.0, 5.0, 0.0], [1.0, 5.0, 0.0]] {
        assert_eq!(holds(&model, solid, floor, point), Err(floor));
    }
}

fn a_quarter_of_a_bar(bar_first: bool) -> (Model, SolidKey) {
    let mut model = a_model();
    let bar = a_bar(&mut model, 5.0, [0.0, 0.0, 0.0]);
    let block = a_block(&mut model, [0.0, 0.0, -1.0], [10.0, 10.0, 6.0]);
    let [one, other] = if bar_first {
        [bar, block]
    } else {
        [block, bar]
    };
    let solid = the_one_solid(&mut model, one, other, Rule::Intersection);
    (model, solid)
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// A bar of `radius`, 4 tall from `foot` for a radius of 5 and 6 tall for the
/// thinner bores, standing up `z`.
fn a_bar(model: &mut Model, radius: f64, foot: [f64; 3]) -> SolidKey {
    let height = if radius == 5.0 { 4.0 } else { 6.0 };
    model
        .add_round_bar(radius, height, &at(foot))
        .expect("a bar of positive size")
}

fn a_block(model: &mut Model, low: [f64; 3], size: [f64; 3]) -> SolidKey {
    let [x, y, z] = size;
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [x, 0.0], [x, y], [0.0, y]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    model
        .add_extrusion(&rectangle, z, &at(low))
        .expect("a positive height")
}

fn at(shift: [f64; 3]) -> Isometry {
    let [x, y, z] = shift;
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}

fn the_one_solid(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(&[one], &[other], rule, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("{rule:?} is refused: {why:?}"));
    assert!(log.is_empty(), "a certified fuse logs nothing: {log:?}");
    let [solid] = answer.solids()[..] else {
        panic!("{rule:?} is one solid, not {:?}", answer.solids());
    };
    solid
}

fn the_faces(model: &Model, solid: SolidKey, wanted: impl Fn(&Surface) -> bool) -> Vec<FaceKey> {
    model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .filter(|face| wanted(&face.surface()))
        .map(|face| face.key())
        .collect()
}

fn the_only_face(model: &Model, solid: SolidKey, wanted: impl Fn(&Surface) -> bool) -> FaceKey {
    let faces = the_faces(model, solid, wanted);
    let [face] = faces[..] else {
        panic!("one face is wanted, and {} are", faces.len());
    };
    face
}

fn on_the_pipe(radius: f64, degrees: f64, z: f64) -> [f64; 3] {
    let turn = degrees * PI / 180.0;
    [radius * turn.cos(), radius * turn.sin(), z]
}

/// What `face` says of `point`, or the face a refusal by name names.
fn holds(
    model: &Model,
    solid: SolidKey,
    face: FaceKey,
    point: [f64; 3],
) -> Result<OnAFace, FaceKey> {
    let mut log = ProvenanceLog::new();
    let answer = model
        .solid(solid)
        .expect("this model made the key")
        .face_holds_the_point(
            face,
            rcad_predicates::Point::from(point),
            &Policy::certified(),
            &mut log,
        );
    assert!(log.is_empty(), "a certified answer logs nothing: {log:?}");
    match answer {
        Ok(found) => Ok(found),
        Err(Error::AFaceTrimmedByACurveThisCannotFollow { face: named }) => Err(named),
        Err(other) => panic!("{point:?} on {face:?} is refused some other way: {other:?}"),
    }
}

/// Which side of each face next to `face` the point is on, read off the
/// surfaces as plain numbers: the test's own reading, for points a long way
/// from every one of them.
fn sides_of_every_neighbour(
    model: &Model,
    solid: SolidKey,
    face: FaceKey,
    point: [f64; 3],
) -> Vec<bool> {
    let view = model.solid(solid).expect("this model made the key");
    let this = view
        .faces()
        .find(|used| used.key() == face)
        .expect("a face of this solid");
    let mut neighbours: Vec<FaceKey> = this
        .wires()
        .flat_map(|wire| wire.coedges().collect::<Vec<_>>())
        .flat_map(|coedge| coedge.edge().neighbouring_faces().collect::<Vec<_>>())
        .filter(|other| *other != face)
        .collect();
    neighbours.sort_by_key(|key| format!("{key:?}"));
    neighbours.dedup();
    neighbours
        .into_iter()
        .map(|key| {
            let neighbour = view
                .faces()
                .find(|used| used.key() == key)
                .expect("a neighbour in this solid");
            let value = signed_value(&neighbour.surface(), point);
            assert!(value.abs() > 0.1, "{point:?} is clear of every neighbour");
            value > 0.0
        })
        .collect()
}

fn signed_value(surface: &Surface, point: [f64; 3]) -> f64 {
    match surface {
        Surface::Plane(flat) => {
            let normal = flat.normal().components();
            (0..3).map(|axis| normal[axis] * point[axis]).sum::<f64>() - flat.offset()
        }
        Surface::Cylinder(pipe) => {
            let origin = pipe.axis().origin().coordinates();
            let [x, y, _] = core::array::from_fn::<f64, 3, _>(|axis| point[axis] - origin[axis]);
            assert_eq!(
                pipe.axis().direction().components(),
                [0.0, 0.0, 1.0],
                "the bores here stand up z"
            );
            x * x + y * y - pipe.radius() * pipe.radius()
        }
        Surface::Sphere(_) | Surface::Cone(_) | Surface::Torus(_) | _ => {
            panic!("only planes and the bore are next to the faces asked about here")
        }
    }
}
