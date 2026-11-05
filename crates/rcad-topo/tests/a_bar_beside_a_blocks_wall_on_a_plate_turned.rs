//! A plate with a block united onto it, and a round bar standing through the
//! plate against the block's wall, all turned about `z`.
//!
//! At `Scale::millimetres(100.0)` the plate is `x` and `y` 0 to 4, `z` 0 to 1;
//! the block on it is `x` 0.5 to 1.5, `y` 1.5 to 3.5, `z` 1 to 2, united with
//! it first, so the foot of the block's wall `x = 1.5` is a side of the
//! plate's top. The bar, of radius `0.5`, stands at `(2, 2.5)` from `z = −1`
//! to `2.5`, so its pipe touches that wall along the line `(1.5, 2.5)`.
//!
//! The bar's pipe cuts the plate's top in a circle that crosses nothing else
//! drawn there, a lump of the drawing of its own, and its one node is where
//! the circle starts: the circle's point along the first direction of the
//! top's plane, `+y`, wherever the turn puts the bar. Turned by `−π/2`, `+y`
//! is the way to the block's wall, so the node is the point where the pipe
//! touches the wall's foot, a rounding from that side of the top. The count
//! of how often the top's rim winds round the node could not be read there,
//! and the top was refused `CannotTellTheCellsOfAFaceApart` under every rule
//! both ways round (Q2353).
//!
//! Turned, the wall's plane is stored a rounding off the pipe, where unturned
//! it touches it. Turned by `−π/2` it misses the pipe: the bar and the block
//! are a rounding apart. So the bar met with the plate and the bar less the
//! plate keep nothing of the gap and answer, while the union and the plate
//! less the bar hold it, a rounding wide, and are refused
//! `DynamicRangeTooWide`. Turned by `π/2`, and at about half of sixty turns
//! of `0.05`, it cuts into the pipe instead, in two lines about `2e−8` apart,
//! and it is the other three fuses that are refused: the bar met with the
//! plate and the bar less the plate keep the sliver of the bar inside the
//! block, about `1e−16` thick, while the union and the plate less the bar
//! keep nothing thinner than the `2e−8` and answer.
//!
//! **Each of those refusals is a limit and not a gap** (Q2504,
//! `docs/decisions.md` §16.10, §437.6). The sliver is in the answer the
//! stored numbers give, as a bar dipping `4.4e−16` through a block's top is
//! (Q2237): at every turn measured, the wall's plane and the pipe are under
//! `1e−14` apart, or overlap by under that, and the plate's corners reach `4`
//! from the axis of the turn. Any answer that keeps the sliver spans over
//! `4e14` times its own thinnest part, past §3.5's `1e11` a thousand times
//! over. Scaling the shape scales both, and no placement puts the whole of a
//! plate 4 wide within 2 of the origin, so no placement or size of the same
//! shape brings it inside either. The answers that would fit leave the sliver
//! out or close the gap, which answers a shape that was not asked.
//!
//! **What still disagrees with that** is the turns where the plane misses the
//! pipe but the circle's node is away from the gap: there the union and the
//! plate less the bar answer, though their answer keeps the same gap.
//! `the_narrowest_a_face_is` in `validate.rs` reads how close a corner of one
//! loop comes to a side of another, and the circle's one corner is at the gap
//! only turned by `−π/2`.

use core::f64::consts::{FRAC_PI_2, PI};

use rcad_geom::{
    Axis, Deflection, Isometry, Point, Scale, Surface, SurfaceMeet, Vector, intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Fused, Invalid, Model, ProfileBuilder, Rule, SolidKey};

const BAR_IN_THE_PLATE: f64 = 0.25 * PI;
const BAR_BELOW_THE_PLATE: f64 = 0.25 * PI;
const BAR_ABOVE_THE_PLATE: f64 = 0.375 * PI;

const A_QUARTER_TURN_BACK: f64 = -FRAC_PI_2;

#[test]
fn a_bar_against_a_blocks_wall_turned_a_quarter_turn_back_is_met_and_cut_by_the_plate() {
    for plate_first in [true, false] {
        let (model, fused) = fused(A_QUARTER_TURN_BACK, Rule::Intersection, plate_first);
        holds(
            &model,
            &fused.unwrap_or_else(|refused| panic!("met, plate first {plate_first}: {refused:?}")),
            &[BAR_IN_THE_PLATE],
        );
    }
    let (model, fused) = fused(A_QUARTER_TURN_BACK, Rule::Difference, false);
    holds(
        &model,
        &fused.unwrap_or_else(|refused| panic!("the bar less the plate: {refused:?}")),
        &[BAR_BELOW_THE_PLATE, BAR_ABOVE_THE_PLATE],
    );
}

#[test]
fn a_bar_a_rounding_from_a_blocks_wall_is_refused_where_the_answer_keeps_the_gap() {
    let (model, keys) = turned_by(A_QUARTER_TURN_BACK);
    let plate = model
        .solid(keys.plate)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface());
    let pipe = the_pipe(&model, &keys);
    let flat_faces_it_meets = plate
        .filter(|surface| matches!(surface, Surface::Plane(_)))
        .filter(|flat| {
            !matches!(
                intersect(flat, &pipe, &Policy::certified(), &mut ProvenanceLog::new()),
                Ok(SurfaceMeet::Nothing)
            )
        })
        .count();
    assert_eq!(
        flat_faces_it_meets, 3,
        "the pipe meets the plate's floor and top and the block's top, and the wall's plane nowhere"
    );

    for (rule, plate_first) in [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Difference, true),
    ] {
        let (_, fused) = fused(A_QUARTER_TURN_BACK, rule, plate_first);
        assert!(
            is_refused_for_its_range(&fused),
            "{rule:?}, plate first {plate_first}: {:?}",
            fused.map(|fused| fused.solids().len())
        );
    }
}

/// The module doc's limit, at each of sixty turns of `0.05`: the stored wall
/// is under `1e−14` off the pipe, every fuse whose answer keeps the sliver of
/// the bar inside the block is refused for its range, and every fuse whose
/// answer keeps no sliver answers.
///
/// Where the wall misses the pipe, the union and the plate less the bar keep
/// the gap and may answer or be refused for their range: the module doc's
/// last paragraph says why both happen.
#[test]
fn turned_sixty_ways_only_a_fuse_whose_answer_keeps_the_sliver_is_refused() {
    const THE_BAR_AND_ITS_SLIVER: [(Rule, bool); 3] = [
        (Rule::Intersection, true),
        (Rule::Intersection, false),
        (Rule::Difference, false),
    ];
    const THE_PLATE_AND_THE_GAP: [(Rule, bool); 3] = [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Difference, true),
    ];
    const EVERY_FUSE: [(Rule, bool); 6] = [
        THE_BAR_AND_ITS_SLIVER[0],
        THE_BAR_AND_ITS_SLIVER[1],
        THE_BAR_AND_ITS_SLIVER[2],
        THE_PLATE_AND_THE_GAP[0],
        THE_PLATE_AND_THE_GAP[1],
        THE_PLATE_AND_THE_GAP[2],
    ];
    let mut turns_where_the_wall_cuts_into_the_pipe = 0;
    for step in 0..60 {
        let angle = f64::from(step) * 0.05;
        let (model, keys) = turned_by(angle);
        let pipe = the_pipe(&model, &keys);
        let wall = the_blocks_wall(&model, &keys, angle);
        let off = how_far_the_plane_misses_the_pipe(&wall, &pipe);
        assert!(
            off.abs() < 1e-14,
            "turned {angle}: the wall is {off:e} off the pipe"
        );

        let meet = intersect(
            &Surface::Plane(wall),
            &pipe,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let (refused, answer_or_refused, answers): (&[_], &[_], &[_]) = match meet {
            Ok(SurfaceMeet::Curves(lines)) if lines.len() == 2 => {
                turns_where_the_wall_cuts_into_the_pipe += 1;
                (&THE_BAR_AND_ITS_SLIVER, &[], &THE_PLATE_AND_THE_GAP)
            }
            Ok(SurfaceMeet::Nothing) => (&[], &THE_PLATE_AND_THE_GAP, &THE_BAR_AND_ITS_SLIVER),
            Ok(SurfaceMeet::Curves(lines)) if lines.len() == 1 => (&[], &[], &EVERY_FUSE),
            other => panic!("turned {angle}: the wall meets the pipe as {other:?}"),
        };
        for &(rule, plate_first) in refused {
            let (_, fused) = fused(angle, rule, plate_first);
            assert!(
                is_refused_for_its_range(&fused),
                "turned {angle}, {rule:?}, plate first {plate_first}: {:?}",
                fused.map(|fused| fused.solids().len())
            );
        }
        for &(rule, plate_first) in answer_or_refused {
            let (_, fused) = fused(angle, rule, plate_first);
            assert!(
                fused.is_ok() || is_refused_for_its_range(&fused),
                "turned {angle}, {rule:?}, plate first {plate_first}: {:?}",
                fused.map(|fused| fused.solids().len())
            );
        }
        for &(rule, plate_first) in answers {
            let (_, fused) = fused(angle, rule, plate_first);
            assert!(
                fused.is_ok(),
                "turned {angle}, {rule:?}, plate first {plate_first}: {:?}",
                fused.map(|fused| fused.solids().len())
            );
        }
    }
    assert!(
        turns_where_the_wall_cuts_into_the_pipe >= 20,
        "the wall cut into the pipe at {turns_where_the_wall_cuts_into_the_pipe} of sixty turns"
    );
}

/// The plate and the bar of [`turned_by`] `angle`, fused under `rule`.
fn fused(angle: f64, rule: Rule, plate_first: bool) -> (Model, rcad_topo::Result<Fused>) {
    let (mut model, keys) = turned_by(angle);
    let (first, second) = if plate_first {
        (keys.plate, keys.bar)
    } else {
        (keys.bar, keys.plate)
    };
    let fused = model.fuse(
        &[first],
        &[second],
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    if fused.is_ok() {
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| {
                panic!("turned {angle}, {rule:?}, plate first {plate_first}: {why}")
            });
    }
    (model, fused)
}

fn is_refused_for_its_range(fused: &rcad_topo::Result<Fused>) -> bool {
    matches!(
        fused,
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { .. }))
    )
}

/// Checks the answer is one solid for each of `volumes`, measuring them in
/// order of size.
fn holds(model: &Model, fused: &Fused, volumes: &[f64]) {
    let mut measured: Vec<(f64, f64)> = fused
        .solids()
        .iter()
        .map(|&solid| {
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.001).expect("a fine mesh"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|refused| panic!("{refused:?}"));
            (measured.volume(), measured.volume_error())
        })
        .collect();
    measured.sort_by(|one, other| one.0.total_cmp(&other.0));
    assert_eq!(
        measured.len(),
        volumes.len(),
        "{measured:?} against {volumes:?}"
    );
    for (&(volume, error), &meant) in measured.iter().zip(volumes) {
        assert!(
            (volume - meant).abs() <= error,
            "{volume} ± {error} against {meant}"
        );
    }
}

/// How far `wall` is from the pipe, less where it overlaps it, in `f64`:
/// good to a few roundings of the coordinates, which is all the module doc's
/// limit asks of it. Both stand along `z`, since the turn is about `z`.
fn how_far_the_plane_misses_the_pipe(wall: &rcad_geom::Plane, pipe: &Surface) -> f64 {
    let Surface::Cylinder(pipe) = pipe else {
        panic!("a bar's pipe is a cylinder");
    };
    let axis = pipe.axis().origin() - Point::new(0.0, 0.0, 0.0);
    (wall.normal().dot(axis) - wall.offset()).abs() - pipe.radius()
}

fn the_pipe(model: &Model, keys: &TheTwo) -> Surface {
    model
        .solid(keys.bar)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .find(|surface| matches!(surface, Surface::Cylinder(_)))
        .expect("a bar has a pipe")
}

/// The plane of the block's wall `x = 1.5`, turned by `angle`: the one face
/// of the plate with its block whose normal is the turned `x` and whose plane
/// is `1.5` from the axis of the turn.
fn the_blocks_wall(model: &Model, keys: &TheTwo, angle: f64) -> rcad_geom::Plane {
    let turned_x = Isometry::rotation(&Axis::z(), angle)
        .expect("a finite angle")
        .apply_to_vector(Vector::new(1.0, 0.0, 0.0));
    let walls: Vec<rcad_geom::Plane> = model
        .solid(keys.plate)
        .expect("this model made the key")
        .faces()
        .filter_map(|face| match face.surface() {
            Surface::Plane(plane)
                if (plane.normal().dot(turned_x).abs() - 1.0).abs() < 1e-9
                    && (plane.offset().abs() - 1.5).abs() < 1e-9 =>
            {
                Some(plane)
            }
            _ => None,
        })
        .collect();
    let [wall] = walls[..] else {
        panic!("turned {angle}: {} walls at 1.5", walls.len());
    };
    wall
}

struct TheTwo {
    plate: SolidKey,
    bar: SolidKey,
}

/// The plate with its block, and the bar, both turned by `angle` about `z`.
fn turned_by(angle: f64) -> (Model, TheTwo) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let plate = a_plate_with_a_block_on_it(&mut model);
    let bar = model
        .add_round_bar(
            0.5,
            3.5,
            &Isometry::translation(Vector::new(2.0, 2.5, -1.0)).expect("a finite placement"),
        )
        .expect("a bar");
    let turn = Isometry::rotation(&Axis::z(), angle).expect("a finite angle");
    model
        .place_solid(plate, &turn)
        .expect("this model made the key");
    model
        .place_solid(bar, &turn)
        .expect("this model made the key");
    (model, TheTwo { plate, bar })
}

fn a_plate_with_a_block_on_it(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let plate = model
        .add_extrusion(&square, 1.0, &Isometry::identity())
        .expect("building a plate needs no decision");
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 2.0], [0.0, 2.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let block = model
        .add_extrusion(
            &rectangle,
            1.0,
            &Isometry::translation(Vector::new(0.5, 1.5, 1.0)).expect("a finite placement"),
        )
        .expect("building a box needs no decision");
    let united = model
        .fuse(
            &[plate],
            &[block],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a block standing on a plate unites with it");
    let [one] = united.solids() else {
        panic!(
            "a block on a plate unites into one solid, got {}",
            united.solids().len()
        );
    };
    *one
}
