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

use rcad_geom::{Axis, Deflection, Isometry, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog, side_of_a_plane_where_three_planes_meet};
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
    fuses_with_the_block_on_the_seam_corner([-0.1, 0.0, 1.0], &Isometry::identity())
        .unwrap_or_else(|why| panic!("{why}"));
}

/// The lean `(0.3, −0.2, 1)`, unturned, and the same block, whose floor
/// crosses the rim at the seam corner. On the floor the line `y = 10` meets
/// the pipe and the cap there a rounding apart, nothing put a line's meet
/// with a pipe in order against its meet with a plane, and every rule was
/// refused on the order along a crossing of the floor (Q3228).
#[test]
fn a_floor_crossing_the_rim_at_the_seam_corner_fuses_under_every_rule() {
    fuses_with_the_block_on_the_seam_corner([0.3, -0.2, 1.0], &Isometry::identity())
        .unwrap_or_else(|why| panic!("{why}"));
}

/// Every turn `step · 0.05` about `z` at which a rule was refused for the two
/// leans whose cap's rim the floor `z = 1` crosses at the seam corner (Q2799):
/// `CannotTellTheCellsOfAFaceApart`, `CannotMeshAFace`, or
/// `Invalid(DynamicRangeTooWide)`. Turned, the seam corner is stored a rounding
/// off where the turn puts it, and the cap's face took it as the place where
/// the floor crosses the rim only where the two boxes share a point. Every one
/// of these answers once that demand is dropped (Q3228).
#[test]
fn every_turn_a_floor_crossing_the_rim_at_the_seam_corner_was_refused_at_fuses() {
    let refused: [([f64; 3], &[u8]); 2] = [
        (
            [0.1, 0.07, 1.0],
            &[9, 18, 21, 30, 34, 39, 40, 41, 45, 49, 51, 56],
        ),
        (
            [0.3, -0.2, 1.0],
            &[
                9, 10, 16, 18, 19, 21, 28, 30, 34, 35, 38, 39, 43, 44, 49, 50, 51, 53, 54, 59,
            ],
        ),
    ];
    let wrong: Vec<String> = refused
        .into_iter()
        .flat_map(|(lean, steps)| {
            steps.iter().filter_map(move |&step| {
                let angle = f64::from(step) * 0.05;
                let turn = Isometry::rotation(&Axis::z(), angle).expect("a finite angle");
                fuses_with_the_block_on_the_seam_corner(lean, &turn)
                    .err()
                    .map(|why| format!("turned {angle}: {why}"))
            })
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The lean whose cap's plane holds the line where the wall `y = 10` meets the
/// floor `z = 1` of the block `[6, 12]² × [1, 3]`, so the cap's straight edge
/// from `(7, 10, 1)` to `(10, 10, 1)` lies in that floor (Q3229).
const THE_CAP_THROUGH_THE_FLOORS_LINE: [f64; 3] = [0.0, 0.1, 1.0];

#[test]
fn probe3229_signs() {
    let shift = 2.0_f64.powi(-43);
    let mut lines = Vec::new();
    for (name, turn) in the_turns_the_sweep_tries() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let plate = a_plate_with_a_leaning_tab(&mut model, &turn, THE_CAP_THROUGH_THE_FLOORS_LINE)
            .expect("the plate");
        let block = a_block(&mut model, [6.0, 6.0, 1.0], [12.0, 12.0, 3.0]);
        model
            .place_solid(block, &turn)
            .expect("this model made the key");
        let [along_x, along_y, up] = THE_CAP_THROUGH_THE_FLOORS_LINE;
        let lean = Vector::new(along_x, along_y, up)
            .normalized()
            .expect("a lean with a length");
        let wall = a_stored_plane(&model, plate, &turn, Vector::new(0.0, 1.0, 0.0), 10.0);
        let floor = a_stored_plane(&model, block, &turn, Vector::new(0.0, 0.0, -1.0), -1.0);
        let cap = a_stored_plane(&model, plate, &turn, lean, 2.0 / 1.01_f64.sqrt());
        let ends = [
            a_stored_plane(&model, block, &turn, Vector::new(-1.0, 0.0, 0.0), -6.0),
            a_stored_plane(&model, block, &turn, Vector::new(1.0, 0.0, 0.0), 12.0),
        ];
        let side = |plane: [f64; 4], end: [f64; 4]| {
            side_of_a_plane_where_three_planes_meet([wall, floor, end], plane)
                .expect("the wall, the floor and a side of the block meet in a point")
        };
        let [a, b, c, d] = cap;
        let sides = ends.map(|end| {
            (
                side(cap, end),
                side([a, b, c, d - shift], end),
                side([a, b, c, d + shift], end),
            )
        });
        lines.push(format!(
            "PROBE3229 {name}: {sides:?} cap {cap:?} wall {wall:?} floor {floor:?}"
        ));
    }
    panic!("{}", lines.join("\n"));
}

#[test]
fn probe3229_tolerant() {
    let mut lines = Vec::new();
    let mut refused = 0;
    for (name, turn) in the_turns_the_sweep_tries() {
        for (policy_name, policy) in [
            ("tolerant", Policy::tolerant(100.0)),
            ("certified", Policy::certified()),
        ] {
            for (rule_name, rule, plate_first) in [
                ("union", Rule::Union, true),
                ("union block first", Rule::Union, false),
                ("intersection", Rule::Intersection, true),
                ("plate less block", Rule::Difference, true),
                ("block less plate", Rule::Difference, false),
            ] {
                let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
                let plate =
                    a_plate_with_a_leaning_tab(&mut model, &turn, THE_CAP_THROUGH_THE_FLOORS_LINE)
                        .expect("the plate");
                let block = a_block(&mut model, [6.0, 6.0, 1.0], [12.0, 12.0, 3.0]);
                model.place_solid(block, &turn).expect("key");
                let (one, other) = if plate_first {
                    (plate, block)
                } else {
                    (block, plate)
                };
                let mut log = ProvenanceLog::new();
                let answer = model.fuse(&[one], &[other], rule, &policy, &mut log);
                let named: Vec<String> = log
                    .records()
                    .iter()
                    .map(|record| format!("{record:?}").chars().take(300).collect())
                    .collect();
                let outcome = match answer {
                    Ok(_) => {
                        match model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
                            Ok(()) => "answers, validates".to_string(),
                            Err(why) => format!("answers, validate {why}"),
                        }
                    }
                    Err(why) => format!("refused {why:?}").chars().take(300).collect(),
                };
                if policy_name == "certified" && outcome != "answers, validates" {
                    refused += 1;
                }
                lines.push(format!(
                    "PROBE3229 {policy_name} {name} {rule_name}: {outcome} log {named:?}"
                ));
            }
        }
    }
    lines.push(format!("PROBE3229 certified {refused} of 305"));
    std::fs::write(
        "/tmp/claude-1000/-home-mattias-git-mattiasgronlund-rcad-worktrees-clean-up-dead-sessions/d396fe66-1e1a-45c1-87e7-028262ea1610/scratchpad/tolerant.txt",
        lines.join("\n"),
    )
    .expect("write");
}

#[test]
fn probe3229_short_edges() {
    let mut lines = Vec::new();
    let turn = Isometry::rotation(&Axis::z(), 2.1).expect("a finite angle");
    let back = turn.inverse();
    for (rule_name, rule, plate_first) in [
        ("union", Rule::Union, true),
        ("intersection", Rule::Intersection, true),
        ("plate less block", Rule::Difference, true),
        ("block less plate", Rule::Difference, false),
    ] {
        let policy = Policy::tolerant(100.0);
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let plate = a_plate_with_a_leaning_tab(&mut model, &turn, THE_CAP_THROUGH_THE_FLOORS_LINE)
            .expect("the plate");
        let block = a_block(&mut model, [6.0, 6.0, 1.0], [12.0, 12.0, 3.0]);
        model.place_solid(block, &turn).expect("key");
        let (one, other) = if plate_first {
            (plate, block)
        } else {
            (block, plate)
        };
        let fused = model
            .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
            .expect("tolerant answers at 2.1");
        for solid in fused.solids() {
            for face in model.solid(*solid).expect("key").faces() {
                for wire in face.wires() {
                    for coedge in wire.coedges() {
                        let edge = coedge.edge();
                        let length = edge.distance_between_its_ends();
                        if length < 1e-9 && !edge.is_closed().unwrap_or(true) {
                            lines.push(format!(
                                "PROBE3229 {rule_name}: face {:?} edge {:?} {length:e} from {:?} to {:?} run {:?}",
                                face.surface(),
                                edge.key(),
                                back.apply_to_point(edge.start().point()),
                                back.apply_to_point(edge.end().point()),
                                edge.run()
                            ));
                        }
                    }
                }
            }
        }
    }
    panic!("{}", lines.join("\n"));
}

/// The 61 turns the sweep of Q2799 and Q3229 placed the plate and the block
/// by: `step · 0.05` about `z` for the 60 steps from `0`, and `0.4` about `x`.
fn the_turns_the_sweep_tries() -> Vec<(String, Isometry)> {
    (0..=59u8)
        .map(|step| (Axis::z(), f64::from(step) * 0.05))
        .chain([(Axis::x(), 0.4)])
        .map(|(axis, angle)| {
            let about = if axis.direction().components()[2] == 1.0 {
                "z"
            } else {
                "x"
            };
            (
                format!("turned {angle} about {about}"),
                Isometry::rotation(&axis, angle).expect("a finite angle"),
            )
        })
        .collect()
}

/// The stored plane, as `[a, b, c, d]`, of the faces of `solid` whose plane
/// is the one through the normal `normal` at `offset` before `turn` placed
/// it. Every such face must share that one plane.
fn a_stored_plane(
    model: &Model,
    solid: SolidKey,
    turn: &Isometry,
    normal: Vector,
    offset: f64,
) -> [f64; 4] {
    let normal = turn.apply_to_vector(normal).components();
    let planes: Vec<[f64; 4]> = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .filter_map(|face| {
            let Surface::Plane(plane) = face.surface() else {
                return None;
            };
            let along: f64 = (0..3)
                .map(|axis| plane.normal().components()[axis] * normal[axis])
                .sum();
            (along > 1.0 - 1e-12 && (plane.offset() - offset).abs() < 1e-12)
                .then(|| plane.coefficients())
        })
        .collect();
    let [first, ..] = planes[..] else {
        panic!("no face of {solid:?} lies on the plane {normal:?} at {offset}");
    };
    assert!(
        planes.iter().all(|plane| *plane == first),
        "the faces on {normal:?} at {offset} lie on {planes:?}"
    );
    first
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

/// The plate of `lean` and the block `[6, 12]² × [1, 3]`, both placed by
/// `turn`, fused under certified by the union either way round, the
/// intersection, and the difference either way round. Each answer must
/// validate, and the volumes must add up within the meshes' own volume errors:
/// the union and the intersection to the plate and the block, and each
/// difference and the intersection to the one it was taken from.
fn fuses_with_the_block_on_the_seam_corner(lean: [f64; 3], turn: &Isometry) -> Result<(), String> {
    let the_block = |model: &mut Model| {
        let block = a_block(model, [6.0, 6.0, 1.0], [12.0, 12.0, 3.0]);
        model
            .place_solid(block, turn)
            .expect("this model made the key");
        block
    };
    let measure = |rule: Rule, plate_first: bool| -> Result<(f64, f64), String> {
        let policy = Policy::certified();
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let plate = a_plate_with_a_leaning_tab(&mut model, turn, lean)?;
        let block = the_block(&mut model);
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
            a_plate_with_a_leaning_tab(&mut model, turn, lean)?,
            the_block(&mut model),
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
