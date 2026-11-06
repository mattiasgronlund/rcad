//! A plate with a round tab at its corner whose cap leans, turned about `z`
//! (Q2856, from Q2854 and Q2716; Q2797 and Q2798).
//!
//! At `Scale::millimetres(100.0)` the plate is the square `[0, 10]²` extruded
//! 2 high, united under certified with `add_bar_cut_off_by(3, [the plane
//! z = 0 facing down, the plane through local (3, 0, 1) with normal lean])`
//! at `(10, 10, 0)`, the two placed with the same turn before they are
//! united. The block it is then united with is `[50, 56]² × [0, 3]`, placed
//! with the same turn and nowhere near it, so the answer is the two solids as
//! they were.
//!
//! The cap cuts the bar's wall in an ellipse, and that ellipse is walked from
//! an end of its short axis, along the bar's axis crossed with the cap's
//! normal. Each of the plate's walls stands on a line through the bar's axis,
//! so one of them crosses the ellipse right where it is walked from: the wall
//! `y = 10` at `(7, 10, 1)` for the lean `(0, 0.1, 1)`, and the wall `x = 10`
//! at `(10, 13, 1.3)` for `(0.1, 0, 1)`. Turned, that corner is a rounding off
//! the walked start, and its box misses it.

use rcad_geom::{Axis, Deflection, Isometry, Plane, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// The lean `(0, 0.1, 1)` turned `29 · 0.05`: the ellipse is walked from
/// `(−9.083610524804325, 8.154018630936783, 0.9999999999999999)`, and the
/// wall corner's box is `[1, 1.0000000000000007]` in `z`, one ulp short of it.
/// No node's box held the start, so none was counted first round the
/// ellipse, and the sweep from the start to that corner against the seam
/// corner across, `7.4e-14`, refused the union on the order along a crossing.
#[test]
fn the_wall_corner_one_ulp_off_the_rims_start_is_counted_first_round_it() {
    unites_with_the_far_block_as_the_two_it_was([0.0, 0.1, 1.0], f64::from(29) * 0.05)
        .unwrap_or_else(|why| panic!("{why}"));
}

/// The lean `(0.1, 0, 1)` turned `0.05`: the first fuse leaves a corner where
/// the wall `x = 10` crosses the cap's rim, exact at
/// `(9.337773403430845, 13.483545077841342, 1.2999999999999998)`, two ulps
/// off the walked start `(9.337773403430843, 13.483545077841345, 1.3)`. The
/// sweep from the start put that corner third of the four round the rim, two
/// edges of the cap were drawn along one stretch of it, and the union was
/// refused `AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve`.
#[test]
fn the_rim_corner_two_ulps_off_the_rims_start_is_counted_first_round_it() {
    unites_with_the_far_block_as_the_two_it_was([0.1, 0.0, 1.0], 0.05)
        .unwrap_or_else(|why| panic!("{why}"));
}

/// Every turn `step · 0.05` about `z` at which the union with the far block
/// was refused (Q2797): `AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve` for the lean
/// `(0.1, 0, 1)`, and the order along a crossing for the other two, but for
/// the lean `(0, 0.1, 1)` turned `50 · 0.05`, which was refused on the side
/// of the cylinder at the box `[−16, 16] × [0, 32] × [1, 4]`.
#[test]
fn every_turn_the_union_with_the_far_block_was_refused_at_answers() {
    let refused: [([f64; 3], &[u8]); 3] = [
        ([0.1, 0.0, 1.0], &[1, 14, 27, 32, 35, 40, 52]),
        (
            [0.0, 0.1, 1.0],
            &[29, 30, 34, 35, 40, 44, 49, 50, 56, 57, 58],
        ),
        ([-0.1, 0.0, 1.0], &[37, 42, 50, 55, 57, 59]),
    ];
    let wrong: Vec<String> = refused
        .into_iter()
        .flat_map(|(lean, steps)| {
            steps.iter().filter_map(move |&step| {
                unites_with_the_far_block_as_the_two_it_was(lean, f64::from(step) * 0.05).err()
            })
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Every turn `step · 0.05` about `z` at which the plate's own union, the
/// square with the tab, was refused on the order along a crossing (Q2798).
#[test]
fn every_turn_the_plate_was_refused_at_answers_as_the_plate() {
    let refused: [([f64; 3], &[u8]); 2] = [
        ([0.0, 0.1, 1.0], &[11, 12, 13, 43, 45]),
        ([-0.1, 0.0, 1.0], &[19, 20, 52]),
    ];
    let wrong: Vec<String> = refused
        .into_iter()
        .flat_map(|(lean, steps)| {
            steps.iter().filter_map(move |&step| {
                the_plate_as_it_should_be(lean, f64::from(step) * 0.05).err()
            })
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The lean `(−0.1, 0, 1)`, unturned, and the block `[6, 12]² × [1, 3]`, whose
/// floor `z = 1` touches the cap's rim at its highest point: the bar's seam
/// corner `(13, 10, 1)`, where the cap, the pipe and the wall plane `y = 10`
/// through the bar's axis meet. The corner was stored at `(13, 10,
/// 1.0000000000000002)`, the floor's touch of the rim split into two places a
/// rounding either side of it, and every rule was refused on the order along a
/// crossing of the cap's face (Q3228).
#[test]
fn a_floor_touching_the_rim_at_the_seam_corner_fuses_under_every_rule() {
    fuses_with_the_block_on_the_seam_corner([-0.1, 0.0, 1.0]).unwrap_or_else(|why| panic!("{why}"));
}

/// The lean `(0.3, −0.2, 1)`, unturned, and the same block, whose floor
/// crosses the rim at the seam corner. On the floor the line `y = 10` meets
/// the pipe and the cap there a rounding apart, nothing put a line's meet
/// with a pipe in order against its meet with a plane, and every rule was
/// refused on the order along a crossing of the floor (Q3228).
#[test]
fn a_floor_crossing_the_rim_at_the_seam_corner_fuses_under_every_rule() {
    fuses_with_the_block_on_the_seam_corner([0.3, -0.2, 1.0]).unwrap_or_else(|why| panic!("{why}"));
}

/// How far a solid's volume may move because its turned corners are each a
/// rounding off where the turn puts them. The block's mesh is its own flat
/// faces, so its volume error is about `1.3e-14`, and turned `0.05` it
/// measured `108.0000000000001`.
const TURNED_A_ROUNDING_OFF: f64 = 1e-12;

/// The plate of `lean` and the far block, turned by `angle` about `z` and
/// united under certified; the answer must validate as two solids that
/// measure the plate's closed form and the block's `108`, each within its
/// mesh's own volume error and [`TURNED_A_ROUNDING_OFF`].
fn unites_with_the_far_block_as_the_two_it_was(lean: [f64; 3], angle: f64) -> Result<(), String> {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let turn = Isometry::rotation(&Axis::z(), angle).expect("a finite angle");
    let plate = a_plate_with_a_leaning_tab(&mut model, &turn, lean)
        .map_err(|why| format!("lean {lean:?} turned {angle}: {why}"))?;
    let block = a_block(&mut model, [50.0, 50.0, 0.0], [56.0, 56.0, 3.0]);
    model
        .place_solid(block, &turn)
        .expect("this model made the key");
    let fused = model
        .fuse(
            &[plate],
            &[block],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .map_err(|why| format!("lean {lean:?} turned {angle}: refused {why:?}"))?;
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("lean {lean:?} turned {angle}: does not validate: {why}"))?;
    let mut volumes = fused
        .solids()
        .iter()
        .map(|solid| {
            measured(&model, *solid).map_err(|why| format!("lean {lean:?} turned {angle}: {why}"))
        })
        .collect::<Result<Vec<_>, String>>()?;
    volumes.sort_by(|one, other| one.0.total_cmp(&other.0));
    let closed_forms = [108.0, the_plates_volume(lean)];
    let [(block, block_error), (plate, plate_error)] = volumes[..] else {
        return Err(format!(
            "lean {lean:?} turned {angle}: {} solids, not two",
            volumes.len()
        ));
    };
    if (block - closed_forms[0]).abs() > block_error + TURNED_A_ROUNDING_OFF
        || (plate - closed_forms[1]).abs() > plate_error + TURNED_A_ROUNDING_OFF
    {
        return Err(format!(
            "lean {lean:?} turned {angle}: measured {block} ± {block_error} and {plate} ± \
             {plate_error}, not within {closed_forms:?}"
        ));
    }
    Ok(())
}

/// The plate of `lean` turned by `angle` about `z`; it must validate and
/// measure its closed form within its mesh's own volume error and
/// [`TURNED_A_ROUNDING_OFF`].
fn the_plate_as_it_should_be(lean: [f64; 3], angle: f64) -> Result<(), String> {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let turn = Isometry::rotation(&Axis::z(), angle).expect("a finite angle");
    let plate = a_plate_with_a_leaning_tab(&mut model, &turn, lean)
        .map_err(|why| format!("lean {lean:?} turned {angle}: {why}"))?;
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("lean {lean:?} turned {angle}: does not validate: {why}"))?;
    let (volume, error) =
        measured(&model, plate).map_err(|why| format!("lean {lean:?} turned {angle}: {why}"))?;
    let closed_form = the_plates_volume(lean);
    if (volume - closed_form).abs() > error + TURNED_A_ROUNDING_OFF {
        return Err(format!(
            "lean {lean:?} turned {angle}: measured {volume} ± {error}, not {closed_form}"
        ));
    }
    Ok(())
}

/// The plate of `lean` and the block `[6, 12]² × [1, 3]`, unturned, fused under
/// certified by the union either way round, the intersection, and the
/// difference either way round. Each answer must validate, and the volumes
/// must add up within the meshes' own volume errors: the union and the
/// intersection to the plate and the block, and each difference and the
/// intersection to the one it was taken from.
fn fuses_with_the_block_on_the_seam_corner(lean: [f64; 3]) -> Result<(), String> {
    let measure = |rule: Rule, plate_first: bool| -> Result<(f64, f64), String> {
        let policy = Policy::certified();
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let plate = a_plate_with_a_leaning_tab(&mut model, &Isometry::identity(), lean)?;
        let block = a_block(&mut model, [6.0, 6.0, 1.0], [12.0, 12.0, 3.0]);
        let (one, other) = if plate_first {
            (plate, block)
        } else {
            (block, plate)
        };
        let fused = model
            .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
            .map_err(|why| format!("refused {why:?}"))?;
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .map_err(|why| format!("does not validate: {why}"))?;
        fused
            .solids()
            .iter()
            .try_fold((0.0, 0.0), |(volume, error), solid| {
                let (more, more_error) = measured(&model, *solid)?;
                Ok((volume + more, error + more_error))
            })
    };
    let alone = |which: usize| -> Result<(f64, f64), String> {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let solids = [
            a_plate_with_a_leaning_tab(&mut model, &Isometry::identity(), lean)?,
            a_block(&mut model, [6.0, 6.0, 1.0], [12.0, 12.0, 3.0]),
        ];
        measured(&model, solids[which])
    };
    let mut wrong = Vec::new();
    let mut answers = Vec::new();
    for (name, rule, plate_first) in [
        ("the union", Rule::Union, true),
        ("the union, the block first", Rule::Union, false),
        ("the intersection", Rule::Intersection, true),
        ("the plate less the block", Rule::Difference, true),
        ("the block less the plate", Rule::Difference, false),
    ] {
        match measure(rule, plate_first) {
            Ok(answer) => answers.push(answer),
            Err(why) => wrong.push(format!("lean {lean:?}, {name}: {why}")),
        }
    }
    if !wrong.is_empty() {
        return Err(wrong.join("\n"));
    }
    let (plate, block) = (alone(0)?, alone(1)?);
    let [
        union,
        union_block_first,
        intersection,
        plate_less,
        block_less,
    ] = answers[..]
    else {
        unreachable!("five rules, five answers");
    };
    let both = [plate, block];
    for (what, answer, whole) in [
        ("the union", union, &both[..]),
        ("the union, the block first,", union_block_first, &both[..]),
        ("the plate less the block", plate_less, &both[..1]),
        ("the block less the plate", block_less, &both[1..]),
    ] {
        let parts = answer.0 + intersection.0;
        let wholes: f64 = whole.iter().map(|(volume, _)| volume).sum();
        let room = answer.1 + intersection.1 + whole.iter().map(|(_, error)| error).sum::<f64>();
        if (parts - wholes).abs() > room {
            wrong.push(format!(
                "lean {lean:?}: {what} and the intersection measure {parts}, not {wholes} within \
                 {room}"
            ));
        }
    }
    if wrong.is_empty() {
        Ok(())
    } else {
        Err(wrong.join("\n"))
    }
}

/// The plate's volume: the square's `200`, and the tab's, less the quarter of
/// the tab inside the square. For a lean `(a, b, 1)` the cap is `z = 1 + 3a −
/// au − bv`, with `u` and `v` the steps from the bar's axis, so the tab is
/// `9π(1 + 3a)` and the quarter where `u` and `v` are both below nought is
/// `9π(1 + 3a)/4 + 9a + 9b`. Only for a lean whose cap stays inside `0 < z <
/// 2` over the tab.
fn the_plates_volume([along_x, along_y, up]: [f64; 3]) -> f64 {
    let (a, b) = (along_x / up, along_y / up);
    200.0 - 9.0 * (a + b) + 27.0 * std::f64::consts::PI / 4.0 * (1.0 + 3.0 * a)
}

/// The square united with the tab whose cap leans by `lean`, both placed by
/// `turn`, or why the union was refused.
fn a_plate_with_a_leaning_tab(
    model: &mut Model,
    turn: &Isometry,
    lean: [f64; 3],
) -> Result<SolidKey, String> {
    let square = a_block(model, [0.0, 0.0, 0.0], [10.0, 10.0, 2.0]);
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("a unit normal");
    let leaning = Plane::from_point_and_normal(
        Point::new(3.0, 0.0, 1.0),
        Vector::new(lean[0], lean[1], lean[2]),
    )
    .expect("a normal with a length");
    let tab = model
        .add_bar_cut_off_by(
            3.0,
            [flat, leaning],
            &Isometry::translation(Vector::new(10.0, 10.0, 0.0)).expect("a finite shift"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map_err(|why| format!("the tab: {why:?}"))?;
    model
        .place_solid(square, turn)
        .expect("this model made the key");
    model
        .place_solid(tab, turn)
        .expect("this model made the key");
    let fused = model
        .fuse(
            &[square],
            &[tab],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map_err(|why| format!("the plate: {why:?}"))?;
    let [plate] = fused.solids() else {
        return Err(format!("the plate: {} solids", fused.solids().len()));
    };
    Ok(*plate)
}

/// The volume of `solid` and how far its mesh may be off it.
fn measured(model: &Model, solid: SolidKey) -> Result<(f64, f64), String> {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map_err(|why| format!("does not measure: {why:?}"))?;
    Ok((measured.volume(), measured.volume_error()))
}

fn a_block(model: &mut Model, [x0, y0, z0]: [f64; 3], [x1, y1, z1]: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    model
        .add_extrusion(
            &rectangle,
            z1 - z0,
            &Isometry::translation(Vector::new(0.0, 0.0, z0)).expect("a finite shift"),
        )
        .expect("a rectangle extrudes")
}
