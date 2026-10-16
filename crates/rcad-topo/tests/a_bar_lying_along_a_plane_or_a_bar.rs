//! A bar lying along a wall, in an inside corner, or along another bar: the
//! three touches along a line `fuses_that_touch.rs` draws, one of each written
//! out by hand, under every rule and both ways round.
//!
//! Along the touching line the two surfaces' gradients are parallel, so `∇S ×
//! ∇Q`, which every other crossing runs along, is exactly nothing. Every fuse of
//! these used to refuse `Undecidable` on *which way a crossing runs* (queue row
//! Q649, §39.4). The direction is the pipe's own axis, and `fuse/arrange.rs`'s
//! `AlongATouch` carries it, turns ways out round it, and gives both sides of the
//! line the one side of the partner the face lies on.
//!
//! **Every rule is answered**, with the answers that exist: the insides are
//! apart, so the union is the two solids as they stood, two lumps whose volumes
//! are the two inputs', the intersection is no solid at all, and each difference
//! is the first solid as it stood.
//!
//! The union used to be refused `ResultIsNotManifold { count: 4 }`. A union
//! keeps both sides of the touching line, of the wall and of the bar's skin
//! alike, so the line came out one edge with four darts along it: two pieces of
//! the wall, and the skin's one piece cut open along the line and walking it
//! both ways (queue row Q760 printed them). `fuse/assemble.rs` now parts such an
//! edge into one per input face, and the wall's is dissolved as any edge between
//! two pieces of one plane is.
//!
//! **A bar whose seam is the touching line keeps its seam's history** (queue row
//! Q801). The seam is left out of the arrangement, so the side the touch cuts
//! along it named no input edge and the seam read as deleted where the answer
//! has an edge on it. `fuse/assemble.rs` names the seam for a straight side
//! along it, gathers every side's input edges onto the edge they share, and
//! hands each half of a parted line its own face's.
//!
//! **A wall that only touches part of the seam keeps both halves in the seam's
//! history** (queue row Q1761, found by Q808). A wall that stops partway up the
//! bar cuts the seam in two at a new corner its own top edge makes, so neither
//! half runs between the seam's own two corners, and until Q1761 neither was
//! named and the seam read as deleted. `the_seam_a_side_runs_along` now names a
//! straight side whose two ends are each a corner of the seam or a place known
//! exactly on it, read where the bar is placed.
//! `a_bar_whose_seam_runs_past_where_a_wall_stops_touching_it` pins it.
//!
//! **A bar on a wall, both turned about `z` by a float angle, answers as a bar
//! clear of the wall** (queue row Q1765). Turned by `π/6` or one radian, the
//! wall's placed plane and the bar's placed pipe miss by `1e−16` or so, which
//! `intersect` proves, so that answer is the sound one. Every fuse was refused
//! `Undecidable` all the same, on which side of the wall the bar's seam corner
//! is: a corner moved by a turn that is not one of a cube's is a box `1e−13`
//! wide. A gap, not a limit: `fuse/arrange.rs` now reads that side off the
//! pipe's axis wherever the plane is proved clear of a pipe the corner is on.
//! `a_bar_on_a_wall_both_turned_by_a_float_angle_answer_every_rule` pins it.
//!
//! **A bar turned onto `x` by a float angle and laid along a block's top or
//! floor answers as a bar clear of that plane** (queue rows Q2071 and Q2102).
//! `Isometry::rotation` about `y` by `π/2` leaves the bar's axis along
//! `(1, 0, 2⁻⁵²)`, so the plane crosses the pipe, far past the bar's end, and
//! Q1765's reading does not answer. `fuse/arrange.rs` now reads the side of a
//! seam corner off the centre of the cap circle it is on, where the plane is
//! proved clear of that circle.
//! `a_bar_turned_onto_x_by_a_float_angle_along_a_blocks_top_answers_every_rule`
//! and its twin on the block's floor pin it.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Deflection, Isometry, Plane, Scale, Surface, SurfaceMeet, Vector, intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{EdgeKey, Model, ProfileBuilder, Rule, SolidKey};

/// How far under the closed form a bar's meshed volume may be.
///
/// The bars here are radius one and two, and a mesh at [`A_DEFLECTION`] round
/// a circle of radius one is 0.33% short of it. A ceiling on a measurement, not
/// a tolerance anything is decided by.
const HOW_FAR_A_MESH_MAY_BE: f64 = 0.005;

/// How fine the meshes the volumes are read off are.
const A_DEFLECTION: f64 = 0.02;

/// A cube of side four and a bar of radius one standing beside its wall `x = 4`,
/// touching it along `(4, 2, z)`. The bar's seam is at `(6, 2, z)`, across the
/// bar from the touch.
#[test]
fn a_bar_standing_against_a_blocks_wall_answers_every_rule() {
    every_rule_of(
        |model| {
            let cube = a_block(model, [4.0, 4.0, 4.0]);
            let bar = a_bar(model, 1.0, 2.0, [5.0, 2.0, 1.0]);
            [(cube, 64.0), (bar, 2.0 * PI)]
        },
        "a bar against a block's wall",
    );
}

/// The same with the bar on the other side of the cube, so its seam `(0, 2, z)`
/// is the touching line itself.
#[test]
fn a_bar_whose_seam_is_the_line_it_touches_a_wall_along_answers_every_rule() {
    every_rule_of(
        |model| {
            let cube = a_block(model, [4.0, 4.0, 4.0]);
            let bar = a_bar(model, 1.0, 2.0, [-1.0, 2.0, 1.0]);
            [(cube, 64.0), (bar, 2.0 * PI)]
        },
        "a bar touching a wall along its seam",
    );
}

/// What became of that bar's seam, under every rule both ways round: the edges
/// of the answer along the touching line, and nothing else.
///
/// The union and the bar less the cube keep the bar whole, seam and all, so the
/// seam has an image there. The intersection and the cube less the bar keep none
/// of the bar's skin, and the cube's wall is one face again, so it has none.
#[test]
fn a_bar_whose_seam_is_the_line_it_touches_a_wall_along_keeps_its_seam_in_history() {
    let mut wrong: Vec<String> = Vec::new();
    for swapped in [false, true] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let cube = a_block(&mut model, [4.0, 4.0, 4.0]);
            let bar = a_bar(&mut model, 1.0, 2.0, [-1.0, 2.0, 1.0]);
            let seam = the_one_straight_edge_of(&model, bar);
            let [first, second] = if swapped { [bar, cube] } else { [cube, bar] };
            let what = format!("{rule:?}, bar first {swapped}");
            let answer = model
                .fuse(
                    &[first],
                    &[second],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what} refused: {why}"));
            let images = answer
                .images_of_edge(seam)
                .expect("this model made the key");
            let along_the_touch: Vec<EdgeKey> = answer
                .solids()
                .iter()
                .flat_map(|&solid| edges_of(&model, solid))
                .filter(|&edge| runs_up_the_line(&model, edge, [0.0, 2.0]))
                .collect();
            let keeps_the_bar = rule == Rule::Union || (rule == Rule::Difference && swapped);
            if along_the_touch.len() != usize::from(keeps_the_bar) || images != along_the_touch {
                wrong.push(format!(
                    "{what}: the answer's edges along the touch are {along_the_touch:?} and the \
                     seam's images {images:?}"
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "the seam's images are the one edge of the answer along it where the answer keeps the \
         bar, and nothing where it does not:\n{}",
        wrong.join("\n")
    );
}

/// A cube `4 × 4 × 2` (`z` in `0..2`) and a bar of radius one, two high, at
/// `(−1, 2, 1)`. The bar's own two seam corners are `(0, 2, 1)` and `(0, 2, 3)`
/// (`a_bar`'s doc comment), so its seam runs from `z = 1` to `z = 3`, but the
/// cube's wall at `x = 0` only stands up to `z = 2`: the touch is the line
/// `(0, 2, z)` for `z` in `[1, 2]` only, and the wall's own top edge crosses
/// the bar's skin at `(0, 2, 2)`, a corner the seam's own two ends do not name.
///
/// Every rule both ways round: never refused, and every fuse keeps the bar
/// whole wherever `a_bar_whose_seam_is_the_line_it_touches_a_wall_along_keeps_its_seam_in_history`
/// does (union, and the bar less the cube). There the answer's seam is cut in
/// two at `(0, 2, 2)` — `(0, 2, 1)`–`(0, 2, 2)` along the touch, and
/// `(0, 2, 2)`–`(0, 2, 3)` above it, where the wall is no longer there to touch.
///
/// **The seam's images are both halves** there, and nothing where the answer
/// keeps none of the bar's skin. Until Q1761 they were empty every time:
/// neither half's two ends are the seam's own two ends, and only a side
/// between those was read as running along the seam.
#[test]
fn a_bar_whose_seam_runs_past_where_a_wall_stops_touching_it() {
    let mut wrong: Vec<String> = Vec::new();
    for swapped in [false, true] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let cube = a_block(&mut model, [4.0, 4.0, 2.0]);
            let bar = a_bar(&mut model, 1.0, 2.0, [-1.0, 2.0, 1.0]);
            let seam = the_one_straight_edge_of(&model, bar);
            let [first, second] = if swapped { [bar, cube] } else { [cube, bar] };
            let what = format!("{rule:?}, bar first {swapped}");
            let answer = model
                .fuse(
                    &[first],
                    &[second],
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what} refused: {why}"));
            let lowest_first = |one: &EdgeKey, other: &EdgeKey| {
                heights_of_the_ends_of(&model, *one)[0]
                    .total_cmp(&heights_of_the_ends_of(&model, *other)[0])
            };
            let mut images = answer
                .images_of_edge(seam)
                .expect("this model made the key")
                .to_vec();
            images.sort_by(lowest_first);
            let mut along_the_line: Vec<EdgeKey> = answer
                .solids()
                .iter()
                .flat_map(|&solid| edges_of(&model, solid))
                .filter(|&edge| runs_up_the_line(&model, edge, [0.0, 2.0]))
                .collect();
            along_the_line.sort_by(lowest_first);
            let heights: Vec<[f64; 2]> = along_the_line
                .iter()
                .map(|&edge| heights_of_the_ends_of(&model, edge))
                .collect();
            let keeps_the_bar = rule == Rule::Union || (rule == Rule::Difference && swapped);
            let wanted: &[[f64; 2]] = if keeps_the_bar {
                &[[1.0, 2.0], [2.0, 3.0]]
            } else {
                &[]
            };
            if heights != wanted || images != along_the_line {
                wrong.push(format!(
                    "{what}: the answer's edges along the line are {along_the_line:?}, from and to \
                     the heights {heights:?} (wanted {wanted:?}), and the seam's images are \
                     {images:?}"
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "the seam's images are its two halves either side of where the wall stops, where the \
         answer keeps the bar, and nothing where it does not:\n{}",
        wrong.join("\n")
    );
}

/// The pairs of the two tests above, the wall four high and two high, with
/// both solids turned about `z` through the origin by `π/6` and by one radian:
/// the cube placed by the turn, and the bar by the turn after its shift to
/// `(−1, 2, 1)`, as a caller turning a sketch's parts into place builds them
/// (queue row Q1765). Every rule both ways round, under `certified`.
///
/// **For the numbers the kernel is handed, the bar is clear of the wall.** The
/// turn is the `f64` nearest it, and composing it with the shift rounds the
/// bar's place, so the plane the model holds for the wall and the pipe it holds
/// for the bar's skin miss by `5.3e−17` at `π/6` and `2.1e−16` at one radian,
/// worked out in exact rationals from the numbers below. `intersect` decides
/// that exactly: they meet in nothing. So the answer is the one a bar clear of
/// a block has, which is a sound one: the union is the two as they stand, the
/// intersection no solid, and each difference the first solid whole. A
/// quarter turn (`Isometry::quarter_turns`) is exact and keeps the touch.
///
/// Every one of the 24 used to be refused `Undecidable`, at the side of the
/// wall's plane of the bar's seam corner. The corner, moved by a turn that is
/// not one of a cube's, is a box `1e−13` wide round where it lands, and the
/// plane runs through the box. The corner is on the pipe, and the plane leaves
/// no room in the pipe, so the side is the side the pipe's axis is on.
///
/// **The seam's history is the one a bar standing clear of the wall has**: the
/// same bar half a unit off the unturned wall, at `(−1.5, 2, 1)`, fused the
/// same way. Nothing lies along the seam, so the fuse joins the skin across it
/// and the seam has no image under any rule; only a wall touching the bar along
/// it keeps it, as in the two tests above. Read edge by edge, the answer then
/// has the same number of straight edges and of round ones as that one.
#[test]
fn a_bar_on_a_wall_both_turned_by_a_float_angle_answer_every_rule() {
    let mut wrong: Vec<String> = Vec::new();
    for angle in [PI / 6.0, 1.0] {
        let turn = Isometry::rotation(&Axis::z(), angle).expect("a finite angle");
        for wall in [4.0, 2.0] {
            for swapped in [false, true] {
                for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                    let what =
                        format!("turned {angle}, wall {wall} high, {rule:?}, bar first {swapped}");
                    let clear = a_bar_against_a_wall(
                        &Isometry::identity(),
                        [-1.5, 2.0, 1.0],
                        wall,
                        swapped,
                        rule,
                    );
                    let turned = a_bar_against_a_wall(&turn, [-1.0, 2.0, 1.0], wall, swapped, rule);
                    match (clear, turned) {
                        (Ok(clear), Ok(turned)) if clear == turned => (),
                        (clear, turned) => wrong.push(format!(
                            "{what}: {turned:?}, and standing clear unturned {clear:?}"
                        )),
                    }
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// What one fuse of
/// [`a_bar_on_a_wall_both_turned_by_a_float_angle_answer_every_rule`] keeps of
/// the bar's seam: how many images the seam has, and how many straight edges
/// and round ones the answer has. The cube is placed by `turn` and the bar by
/// `turn` after its shift to `bar_at`. `Err` says what is wrong with the fuse.
fn a_bar_against_a_wall(
    turn: &Isometry,
    bar_at: [f64; 3],
    wall: f64,
    swapped: bool,
    rule: Rule,
) -> Result<[usize; 3], String> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let cube = model
        .add_extrusion(&rectangle, wall, turn)
        .expect("building a box needs no decision");
    let [x, y, z] = bar_at;
    let shift = Isometry::translation(Vector::new(x, y, z)).expect("a short shift");
    let bar = model
        .add_round_bar(1.0, 2.0, &turn.compose(&shift))
        .expect("a bar");
    if !the_wall_misses_the_pipe(&model, cube, bar) {
        return Err("the wall's plane meets the bar's pipe".to_owned());
    }
    let seam = the_one_straight_edge_of(&model, bar);
    let [(first, first_volume), (second, second_volume)] = if swapped {
        [(bar, 2.0 * PI), (cube, 16.0 * wall)]
    } else {
        [(cube, 16.0 * wall), (bar, 2.0 * PI)]
    };
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(&[first], &[second], rule, &Policy::certified(), &mut log)
        .map_err(|why| format!("refused {why}"))?;
    if !log.records().is_empty() {
        return Err(format!("not every decision is proved: {:?}", log.records()));
    }
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .map_err(|why| format!("the model does not validate: {why}"))?;
    let mut found: Vec<f64> = answer
        .solids()
        .iter()
        .map(|&solid| volume_of(&model, solid))
        .collect();
    found.sort_by(f64::total_cmp);
    let mut expected = match rule {
        Rule::Union => vec![first_volume, second_volume],
        Rule::Intersection => vec![],
        Rule::Difference => vec![first_volume],
        _ => unreachable!("only the three rules are asked"),
    };
    expected.sort_by(f64::total_cmp);
    let near = found.len() == expected.len()
        && found.iter().zip(&expected).all(|(found, expected)| {
            found <= expected && (expected - found) / expected < HOW_FAR_A_MESH_MAY_BE
        });
    if !near {
        return Err(format!(
            "measured {found:?}, and the answer is {expected:?}"
        ));
    }
    let images = answer
        .images_of_edge(seam)
        .expect("this model made the key")
        .len();
    let (round, straight): (Vec<EdgeKey>, Vec<EdgeKey>) = answer
        .solids()
        .iter()
        .flat_map(|&solid| edges_of(&model, solid))
        .partition(|&edge| {
            model
                .edge(edge)
                .expect("this model made the key")
                .is_closed()
                .expect("the edges here are straight or round a circle")
        });
    Ok([images, straight.len(), round.len()])
}

/// Whether `intersect` proves that the cube's upright wall nearest the bar, as
/// the model holds it placed, and the bar's skin meet in nothing.
fn the_wall_misses_the_pipe(model: &Model, cube: SolidKey, bar: SolidKey) -> bool {
    let skin = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .find(|surface| matches!(surface, Surface::Cylinder(_)))
        .expect("a bar has a round skin");
    let Surface::Cylinder(pipe) = skin else {
        unreachable!("found as a pipe")
    };
    let wall = model
        .solid(cube)
        .expect("this model made the key")
        .faces()
        .map(|face| face.plane())
        .filter(|plane| plane.normal().z() == 0.0)
        .min_by(|one, other| {
            let off =
                |plane: &Plane| (plane.signed_distance_to(pipe.axis().origin()).abs() - 1.0).abs();
            off(one).total_cmp(&off(other))
        })
        .expect("a box has faces");
    matches!(
        intersect(
            &Surface::Plane(wall),
            &skin,
            &Policy::certified(),
            &mut ProvenanceLog::new()
        ),
        Ok(SurfaceMeet::Nothing)
    )
}

/// The heights of an upright edge's two ends, lower first.
fn heights_of_the_ends_of(model: &Model, edge: EdgeKey) -> [f64; 2] {
    let edge = model.edge(edge).expect("this model made the key");
    let [one, other] = [edge.start().point().z(), edge.end().point().z()];
    [one.min(other), one.max(other)]
}

/// A bar of radius one in the hole of a tube, radius seven less radius three,
/// touching the inner wall along `(3, 0, z)`, which is the bar's seam. United,
/// both skins keep the line, so the edge along it is parted into one per skin
/// and both stay: the bar's seam is the bar's edge, and not the tube's too.
#[test]
fn a_bar_whose_seam_touches_a_tubes_inner_wall_keeps_its_seam_in_its_own_lump() {
    for swapped in [false, true] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let outer = a_bar(&mut model, 7.0, 4.0, [0.0, 0.0, 0.0]);
        let through = a_bar(&mut model, 3.0, 6.0, [0.0, 0.0, -1.0]);
        let tube = model
            .fuse(
                &[outer],
                &[through],
                Rule::Difference,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a bar less a thinner bar through it is a tube")
            .solids()[0];
        let bar = a_bar(&mut model, 1.0, 4.0, [2.0, 0.0, 0.0]);
        let seam = the_one_straight_edge_of(&model, bar);
        let [first, second] = if swapped { [bar, tube] } else { [tube, bar] };
        let what = format!("bar first {swapped}");
        let answer = model
            .fuse(
                &[first],
                &[second],
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} refused: {why}"));
        let lumps = answer.solids();
        assert_eq!(lumps.len(), 2, "{what}: the tube and the bar, apart");
        let the_bar = if volume_of(&model, lumps[0]) < volume_of(&model, lumps[1]) {
            lumps[0]
        } else {
            lumps[1]
        };
        let images = answer
            .images_of_edge(seam)
            .expect("this model made the key");
        let the_bars_edge_along_the_touch: Vec<EdgeKey> = edges_of(&model, the_bar)
            .into_iter()
            .filter(|&edge| runs_up_the_line(&model, edge, [3.0, 0.0]))
            .collect();
        assert_eq!(
            images, the_bars_edge_along_the_touch,
            "{what}: the seam's one image is the bar's own edge along the touch"
        );
    }
}

/// An L of two arms six long and two thick, four high, with a bar of radius one
/// in its inside corner, touching both walls: `x = 2` along `(2, 3, z)` and
/// `y = 2` along `(3, 2, z)`. The fillet a part's inside corner is.
#[test]
fn a_bar_in_an_inside_corner_answers_every_rule() {
    every_rule_of(
        |model| {
            let corner = an_l(model);
            let bar = a_bar(model, 1.0, 2.0, [3.0, 3.0, 1.0]);
            [(corner, 80.0), (bar, 2.0 * PI)]
        },
        "a bar in an inside corner",
    );
}

/// A bar of radius two and one of radius one three apart along `y`, touching
/// along `(0, 2, z)`. Both seams are off the touch, at `(2, 0, z)` and
/// `(1, 3, z)`.
#[test]
fn two_bars_side_by_side_answer_every_rule() {
    every_rule_of(
        |model| {
            let thick = a_bar(model, 2.0, 4.0, [0.0, 0.0, 0.0]);
            let thin = a_bar(model, 1.0, 2.0, [0.0, 3.0, 1.0]);
            [(thick, 16.0 * PI), (thin, 2.0 * PI)]
        },
        "two bars side by side",
    );
}

/// **A solid resting on itself along a line fuses on** (queue row Q1902). The
/// stepped block and the bar resting on its wall
/// ([`the_bar_resting_on_a_stepped_blocks_wall`]) are one solid whose wall
/// `x = 4` and whose bar's skin touch each other along `(4, 1.75, z)`. Fused
/// with a cube of one far from it, every rule both ways round, the union is the
/// two as they stand, the intersection no solid, and each difference its first
/// solid whole.
///
/// Every one of the six used to be refused `ArgumentIntersectsItself`: the
/// check that no two faces of an input run through each other read the wall
/// and the skin meeting along a line as two faces crossing there. They only
/// touch, and the bar slid a quarter clear of the wall fused on all along.
#[test]
fn a_bar_resting_on_a_stepped_blocks_wall_fuses_on_with_a_far_cube() {
    every_fuse_of_the_bar_resting_on_the_step_with(
        |model| a_cube_of_one_at(model, [20.0, 20.0, 20.0]),
        |rule, it_first| match (rule, it_first) {
            (Rule::Union, _) => vec![1.0, the_bar_resting_on_the_step()],
            (Rule::Intersection, _) => vec![],
            (Rule::Difference, false) => vec![the_bar_resting_on_the_step()],
            (Rule::Difference, true) => vec![1.0],
            _ => unreachable!("only the three rules are asked"),
        },
        "a far cube",
    );
}

/// The same solid against a slab over its top half, `z` from 2 to 5, which
/// cuts the touching line through at `(4, 1.75, 2)` and keeps it below or above
/// that in every answer but the slab less the solid. Below the slab's floor the
/// solid is the step `18 × 2` and the bar's disc out of the step, `π` less the
/// quarter-deep piece of it inside, one high; above it, `18 × 2` and the same
/// disc one high again.
#[test]
fn a_bar_resting_on_a_stepped_blocks_wall_fuses_with_a_slab_across_the_touch() {
    every_fuse_of_the_bar_resting_on_the_step_with(
        |model| {
            let slab = a_block(model, [8.0, 6.0, 3.0]);
            model
                .place_solid(
                    slab,
                    &Isometry::translation(Vector::new(-1.0, -1.0, 2.0)).expect("whole numbers"),
                )
                .expect("this model made the key");
            slab
        },
        |rule, it_first| {
            let half = 36.0 + the_bars_disc_out_of_the_step();
            match (rule, it_first) {
                (Rule::Union, _) => vec![144.0 + half],
                (Rule::Intersection, _) | (Rule::Difference, false) => vec![half],
                (Rule::Difference, true) => vec![144.0 - half],
                _ => unreachable!("only the three rules are asked"),
            }
        },
        "a slab across the top half",
    );
}

/// Every rule, both ways round, of the stepped block united with the bar
/// resting on its wall against the solid `other` builds: never refused, the
/// model validates, and the answer's solids have the volumes `should` gives
/// for the rule and whether `other` came first.
///
/// Volumes are compared both ways within [`HOW_FAR_A_MESH_MAY_BE`]: a mesh is
/// short of a bar but over a solid with a bar taken out of it.
fn every_fuse_of_the_bar_resting_on_the_step_with(
    other: fn(&mut Model) -> SolidKey,
    should: fn(Rule, bool) -> Vec<f64>,
    against: &str,
) {
    let mut wrong: Vec<String> = Vec::new();
    for it_first in [false, true] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let touching = the_bar_resting_on_a_stepped_blocks_wall(&mut model);
            let other = other(&mut model);
            let [first, second] = if it_first {
                [other, touching]
            } else {
                [touching, other]
            };
            let what = format!("{against}, {rule:?}, it first {it_first}");
            let answer = match model.fuse(
                &[first],
                &[second],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            ) {
                Ok(answer) => answer,
                Err(why) => {
                    wrong.push(format!("{what}: refused {why:?}"));
                    continue;
                }
            };
            if let Err(why) = model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
                wrong.push(format!("{what}: the model does not validate: {why:?}"));
                continue;
            }
            let mut found: Vec<f64> = answer
                .solids()
                .iter()
                .map(|&solid| volume_of(&model, solid))
                .collect();
            found.sort_by(f64::total_cmp);
            let mut expected = should(rule, it_first);
            expected.sort_by(f64::total_cmp);
            let near = found.len() == expected.len()
                && found.iter().zip(&expected).all(|(found, expected)| {
                    ((expected - found) / expected).abs() < HOW_FAR_A_MESH_MAY_BE
                });
            if !near {
                wrong.push(format!(
                    "{what}: measured {found:?}, and the answer is {expected:?}"
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The stepped block, the outline `(0, 0)`, `(6, 0)`, `(6, 1)`, `(4, 1)`,
/// `(4, 4)`, `(0, 4)` four high, united with a bar of radius one, two high, at
/// `(5, 1.75, 1)`. The bar rests on the wall `x = 4` along `(4, 1.75, z)` for
/// `z` from 1 to 3 and reaches a quarter into the step below `y = 1`, so the
/// union is one solid touching itself along that line (§39.4, the Q1901 note).
fn the_bar_resting_on_a_stepped_blocks_wall(model: &mut Model) -> SolidKey {
    let outline = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [6.0, 0.0],
        [6.0, 1.0],
        [4.0, 1.0],
        [4.0, 4.0],
        [0.0, 4.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a step");
    let block = model
        .add_extrusion(&outline, 4.0, &Isometry::identity())
        .expect("building a step needs no decision");
    let bar = a_bar(model, 1.0, 2.0, [5.0, 1.75, 1.0]);
    let united = model
        .fuse(
            &[block],
            &[bar],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar resting on a wall and reaching into a step unites with it");
    let [solid] = united.solids() else {
        panic!("the bar reaches into the step, so the union is one solid")
    };
    *solid
}

/// The volume of [`the_bar_resting_on_a_stepped_blocks_wall`]: the step's
/// `18 × 4` and the bar's disc out of the step two high.
fn the_bar_resting_on_the_step() -> f64 {
    72.0 + 2.0 * the_bars_disc_out_of_the_step()
}

/// The bar's disc of radius one at `y = 1.75` less the piece of it below
/// `y = 1`, a quarter deep: `π − (acos 0.75 − 0.75·√(1 − 0.75²))`.
fn the_bars_disc_out_of_the_step() -> f64 {
    PI - (0.75_f64.acos() - 0.75 * (1.0 - 0.75 * 0.75_f64).sqrt())
}

/// A cube of side one with its low corner at `at`.
fn a_cube_of_one_at(model: &mut Model, at: [f64; 3]) -> SolidKey {
    let cube = a_block(model, [1.0, 1.0, 1.0]);
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("whole numbers"),
        )
        .expect("this model made the key");
    cube
}

/// Every rule, both ways round, of a pair whose insides are apart: both solids
/// as they stand for the union, no solid for the intersection, and the first
/// solid for each difference.
fn every_rule_of(build: fn(&mut Model) -> [(SolidKey, f64); 2], pair: &str) {
    for swapped in [false, true] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let [one, other] = build(&mut model);
            let [(first, first_volume), (second, second_volume)] =
                if swapped { [other, one] } else { [one, other] };
            let what = format!("{pair}, {rule:?}, swapped {swapped}");
            let mut log = ProvenanceLog::new();
            let answer = model
                .fuse(&[first], &[second], rule, &Policy::certified(), &mut log)
                .unwrap_or_else(|why| panic!("{what} refused: {why}"));
            assert!(
                log.records().is_empty(),
                "{what}: every decision is proved, got {:?}",
                log.records()
            );
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what} wrote a model that does not validate: {why}"));
            if rule == Rule::Union {
                let mut found: Vec<f64> = answer
                    .solids()
                    .iter()
                    .map(|&solid| volume_of(&model, solid))
                    .collect();
                found.sort_by(f64::total_cmp);
                let mut expected = [first_volume, second_volume];
                expected.sort_by(f64::total_cmp);
                assert_eq!(
                    found.len(),
                    2,
                    "{what}: two solids touching along a line are two lumps, got {found:?}"
                );
                for (found, expected) in found.iter().zip(expected) {
                    assert!(
                        *found <= expected && (expected - found) / expected < HOW_FAR_A_MESH_MAY_BE,
                        "{what}: measured {found}, and the solid is {expected}"
                    );
                }
                continue;
            }
            if rule == Rule::Intersection {
                assert!(
                    answer.solids().is_empty(),
                    "{what}: two solids whose insides are apart share no solid"
                );
                continue;
            }
            assert_eq!(answer.solids().len(), 1, "{what}: the first solid, whole");
            let found = volume_of(&model, answer.solids()[0]);
            assert!(
                found <= first_volume
                    && (first_volume - found) / first_volume < HOW_FAR_A_MESH_MAY_BE,
                "{what}: measured {found}, and the first solid is {first_volume}"
            );
        }
    }
}

fn volume_of(model: &Model, solid: SolidKey) -> f64 {
    model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(A_DEFLECTION).expect("a fiftieth of a millimetre is a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures")
        .volume()
}

/// A bar's seam: of its three edges, the two caps' rims close and the seam is
/// straight.
fn the_one_straight_edge_of(model: &Model, bar: SolidKey) -> EdgeKey {
    let straight: Vec<EdgeKey> = edges_of(model, bar)
        .into_iter()
        .filter(|&edge| {
            !model
                .edge(edge)
                .expect("this model made the key")
                .is_closed()
                .expect("a bar's edges are straight or round a circle")
        })
        .collect();
    assert_eq!(straight.len(), 1, "a bar has one seam, got {straight:?}");
    straight[0]
}

/// Every edge of a solid, once each.
fn edges_of(model: &Model, solid: SolidKey) -> Vec<EdgeKey> {
    let mut edges: Vec<EdgeKey> = Vec::new();
    for face in model.solid(solid).expect("this model made the key").faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge().key();
                if !edges.contains(&edge) {
                    edges.push(edge);
                }
            }
        }
    }
    edges
}

/// Whether an edge runs along the upright line through `(x, y)`: not round a
/// cap's rim, which starts and ends on the line, and with both ends on it. The
/// ends are sums of whole numbers, so they come out exactly.
fn runs_up_the_line(model: &Model, edge: EdgeKey, [x, y]: [f64; 2]) -> bool {
    let edge = model.edge(edge).expect("this model made the key");
    let closes = edge
        .is_closed()
        .expect("the edges here are straight or round a circle");
    !closes
        && [edge.start().point(), edge.end().point()]
            .iter()
            .all(|end| end.x() == x && end.y() == y)
}

/// A box with its low corner at the origin.
fn a_block(model: &mut Model, [along_x, along_y, along_z]: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(&rectangle, along_z, &Isometry::identity())
        .expect("building a box needs no decision")
}

/// An L standing on the floor, four high: two arms two thick and six long,
/// meeting at the origin, with the inside corner at `(2, 2)`. Its area is
/// `36 − 16 = 20`, so its volume is 80.
fn an_l(model: &mut Model) -> SolidKey {
    let outline = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [6.0, 0.0],
        [6.0, 2.0],
        [2.0, 2.0],
        [2.0, 6.0],
        [0.0, 6.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("an L");
    model
        .add_extrusion(&outline, 4.0, &Isometry::identity())
        .expect("building an L needs no decision")
}

/// A bar standing on its foot, its seam a radius along `+x` from its axis.
fn a_bar(model: &mut Model, radius: f64, height: f64, [x, y, z]: [f64; 3]) -> SolidKey {
    model
        .add_round_bar(
            radius,
            height,
            &Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift"),
        )
        .expect("a bar")
}

/// The cylinder twin of Q953's wedge less the frustum (queue row Q1014): a
/// block of side four, and a bar of radius one the same four high, lying
/// inside it against the wall `x = 0`, touching it along `(0, 2, z)`. The
/// wall's own corners sit at `z = 0` and `z = 4`, the same range the ruling
/// runs, so the touch spans the wall's whole height.
///
/// The bar sits wholly inside the block, so the union is the block alone and
/// the bar less the block is nothing. The intersection is the bar alone. The
/// block less the bar is the one case that touches itself: in every cross
/// section the hole's circle is tangent to the wall, so the wall's own
/// thickness there runs to nothing, and the answer stays one piece whose wall
/// folds back on itself along the ruling rather than parting into two.
///
/// Every one of the twelve fuses here — three rules, either argument first,
/// certified or tolerant — answers; none is refused. `mesh.rs`'s fix for a
/// straight edge lying across a flat face of its own solid (§364) is what the
/// block-less-bar answer needs in order to measure at all: the ruling is such
/// an edge, lying across the block's wall.
#[test]
fn a_block_less_a_bar_against_its_wall_touches_itself_along_the_ruling() {
    const BLOCK_SIDE: f64 = 4.0;
    const BAR_RADIUS: f64 = 1.0;
    let block_volume = BLOCK_SIDE * BLOCK_SIDE * BLOCK_SIDE;
    let bar_volume = PI * BAR_RADIUS * BAR_RADIUS * BLOCK_SIDE;

    let mut wrong: Vec<String> = Vec::new();
    for it_first in [false, true] {
        for policy_name in ["certified", "tolerant"] {
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                let mut model =
                    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
                let block = a_block(&mut model, [BLOCK_SIDE, BLOCK_SIDE, BLOCK_SIDE]);
                let bar = a_bar(&mut model, BAR_RADIUS, BLOCK_SIDE, [1.0, 2.0, 0.0]);
                let [first, second] = if it_first { [bar, block] } else { [block, bar] };
                let policy = if policy_name == "certified" {
                    Policy::certified()
                } else {
                    Policy::tolerant(100.0)
                };
                let what = format!("{rule:?}, bar first {it_first}, {policy_name}");
                let answer = match model.fuse(
                    &[first],
                    &[second],
                    rule,
                    &policy,
                    &mut ProvenanceLog::new(),
                ) {
                    Ok(answer) => answer,
                    Err(why) => {
                        wrong.push(format!("{what}: refused {why}"));
                        continue;
                    }
                };
                if let Err(why) = model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
                    wrong.push(format!("{what}: the model does not validate: {why:?}"));
                    continue;
                }
                let solids = answer.solids();
                let wanted_solids = usize::from(!(rule == Rule::Difference && it_first));
                if solids.len() != wanted_solids {
                    wrong.push(format!(
                        "{what}: {} solids, wanted {wanted_solids}: {solids:?}",
                        solids.len()
                    ));
                    continue;
                }
                if wanted_solids == 0 {
                    continue;
                }
                let found = volume_of(&model, solids[0]);
                let near = match rule {
                    Rule::Union => (found - block_volume).abs() < 1e-9,
                    Rule::Intersection => {
                        found <= bar_volume
                            && (bar_volume - found) / bar_volume < HOW_FAR_A_MESH_MAY_BE
                    }
                    Rule::Difference => {
                        let expected = block_volume - bar_volume;
                        found >= expected && (found - expected) / expected < HOW_FAR_A_MESH_MAY_BE
                    }
                    _ => unreachable!("only the three rules are asked"),
                };
                if !near {
                    let expected = match rule {
                        Rule::Union => block_volume,
                        Rule::Intersection => bar_volume,
                        Rule::Difference => block_volume - bar_volume,
                        _ => unreachable!("only the three rules are asked"),
                    };
                    wrong.push(format!(
                        "{what}: measured {found}, and the answer is {expected}"
                    ));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A block of side four, and a bar of radius one and two high standing inside
/// it from `z = 1` to `z = 3`, touching a wall along a ruling (queue row Q1903,
/// from Q1116). Two placings: at `(3, 2, 1)` the bar touches the wall `x = 4`
/// along its own seam, and at `(1, 2, 1)` it touches `x = 0` across from it.
///
/// Unlike [`a_block_less_a_bar_against_its_wall_touches_itself_along_the_ruling`],
/// the bar stops short of the block's floor and roof, so the block less the bar
/// is **one solid of two shells**: the block's six faces, and the bar's three
/// as a hollow, the two touching along a line (`docs/decisions.md` §39.4, the
/// 2026-10-13 note). It was refused `Undecidable` at *one shell inside another*,
/// over the two ends of the touching line: the touch parts the line (Q760),
/// so every corner of the hollow is on the block's wall and none says which
/// side of it the hollow is. The far side of each of the hollow's cap circles
/// does.
///
/// The bar is wholly inside the block, so the union is the block alone, the
/// intersection is the bar alone, and the bar less the block is nothing. Every
/// one of the 24 fuses here — two placings, three rules, either argument first,
/// certified or tolerant — answers.
#[test]
fn a_block_less_a_shorter_bar_touching_its_wall_keeps_the_bar_as_a_hollow() {
    const BLOCK_SIDE: f64 = 4.0;
    const BAR_RADIUS: f64 = 1.0;
    const BAR_HEIGHT: f64 = 2.0;
    let block_volume = BLOCK_SIDE * BLOCK_SIDE * BLOCK_SIDE;
    let bar_volume = PI * BAR_RADIUS * BAR_RADIUS * BAR_HEIGHT;

    let mut wrong: Vec<String> = Vec::new();
    for at in [[3.0, 2.0, 1.0], [1.0, 2.0, 1.0]] {
        for it_first in [false, true] {
            for policy_name in ["certified", "tolerant"] {
                for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                    let mut model =
                        Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
                    let block = a_block(&mut model, [BLOCK_SIDE, BLOCK_SIDE, BLOCK_SIDE]);
                    let bar = a_bar(&mut model, BAR_RADIUS, BAR_HEIGHT, at);
                    let [first, second] = if it_first { [bar, block] } else { [block, bar] };
                    let policy = if policy_name == "certified" {
                        Policy::certified()
                    } else {
                        Policy::tolerant(100.0)
                    };
                    let what =
                        format!("bar at {at:?}, {rule:?}, bar first {it_first}, {policy_name}");
                    let answer = match model.fuse(
                        &[first],
                        &[second],
                        rule,
                        &policy,
                        &mut ProvenanceLog::new(),
                    ) {
                        Ok(answer) => answer,
                        Err(why) => {
                            wrong.push(format!("{what}: refused {why:?}"));
                            continue;
                        }
                    };
                    if let Err(why) =
                        model.validate(&Policy::certified(), &mut ProvenanceLog::new())
                    {
                        wrong.push(format!("{what}: the model does not validate: {why:?}"));
                        continue;
                    }
                    let solids = answer.solids();
                    let wanted_solids = usize::from(!(rule == Rule::Difference && it_first));
                    if solids.len() != wanted_solids {
                        wrong.push(format!(
                            "{what}: {} solids, wanted {wanted_solids}: {solids:?}",
                            solids.len()
                        ));
                        continue;
                    }
                    if wanted_solids == 0 {
                        continue;
                    }
                    let faces_of_each_shell = faces_of_each_shell(&model, solids[0]);
                    let wanted_shells: &[usize] = match rule {
                        Rule::Union => &[6],
                        Rule::Intersection => &[3],
                        Rule::Difference => &[3, 6],
                        _ => unreachable!("only the three rules are asked"),
                    };
                    if faces_of_each_shell != wanted_shells {
                        wrong.push(format!(
                            "{what}: shells of {faces_of_each_shell:?} faces, wanted \
                             {wanted_shells:?}"
                        ));
                    }
                    let found = volume_of(&model, solids[0]);
                    let (expected, near) = match rule {
                        Rule::Union => (block_volume, (found - block_volume).abs() < 1e-9),
                        Rule::Intersection => (
                            bar_volume,
                            found <= bar_volume
                                && (bar_volume - found) / bar_volume < HOW_FAR_A_MESH_MAY_BE,
                        ),
                        Rule::Difference => {
                            let expected = block_volume - bar_volume;
                            (
                                expected,
                                found >= expected
                                    && (found - expected) / expected < HOW_FAR_A_MESH_MAY_BE,
                            )
                        }
                        _ => unreachable!("only the three rules are asked"),
                    };
                    if !near {
                        wrong.push(format!(
                            "{what}: measured {found}, and the answer is {expected}"
                        ));
                    }
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The bar of the test above, radius one and two high from `z = 1` to `z = 3`,
/// touching **two** walls of a block along two rulings (queue rows Q2069 and
/// Q2101). Two placings: in the block `[0, 4]³` at `(1, 1, 1)` it touches the
/// walls `x = 0` and `y = 0`, whose rulings are a quarter turn apart round the
/// bar's axis; in a block 2 by 4 by 4 at `(1, 2, 1)` it fills the block's
/// width and touches `x = 0` and `x = 2`, half a turn apart.
///
/// Q760 parts both rulings, so each of the bar's cap circles is two arcs whose
/// four ends are all on the walls, and no edge of the hollow closes: neither
/// its corners nor the far side of a closed edge say which side of the walls
/// the hollow is. The block less the bar was refused `Undecidable` at *one
/// shell inside another*, over the four corners `(1, 0, 1)`, `(0, 1, 1)`,
/// `(0, 1, 3)`, `(1, 0, 3)` in the first block and `(0, 2, 1)`, `(2, 2, 1)`,
/// `(0, 2, 3)`, `(2, 2, 3)` in the second. A point of each arc off its ends
/// does say.
///
/// The bar is wholly inside the block, so the union is the block alone, the
/// intersection is the bar alone, and the bar less the block is nothing. The
/// block less the bar is one solid of two shells, the block's six faces and
/// the bar's three, measuring `64 − 2π` and `32 − 2π`. Every one of the 24
/// fuses here — two placings, three rules, either argument first, certified or
/// tolerant — answers.
#[test]
fn a_block_less_a_shorter_bar_touching_two_walls_keeps_the_bar_as_a_hollow() {
    const BAR_RADIUS: f64 = 1.0;
    const BAR_HEIGHT: f64 = 2.0;
    let bar_volume = PI * BAR_RADIUS * BAR_RADIUS * BAR_HEIGHT;

    let mut wrong: Vec<String> = Vec::new();
    for (sides, at) in [
        ([4.0, 4.0, 4.0], [1.0, 1.0, 1.0]),
        ([2.0, 4.0, 4.0], [1.0, 2.0, 1.0]),
    ] {
        let block_volume = sides[0] * sides[1] * sides[2];
        for it_first in [false, true] {
            for policy_name in ["certified", "tolerant"] {
                for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                    let mut model =
                        Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
                    let block = a_block(&mut model, sides);
                    let bar = a_bar(&mut model, BAR_RADIUS, BAR_HEIGHT, at);
                    let [first, second] = if it_first { [bar, block] } else { [block, bar] };
                    let policy = if policy_name == "certified" {
                        Policy::certified()
                    } else {
                        Policy::tolerant(100.0)
                    };
                    let what = format!(
                        "block {sides:?}, bar at {at:?}, {rule:?}, bar first {it_first}, \
                         {policy_name}"
                    );
                    let answer = match model.fuse(
                        &[first],
                        &[second],
                        rule,
                        &policy,
                        &mut ProvenanceLog::new(),
                    ) {
                        Ok(answer) => answer,
                        Err(why) => {
                            wrong.push(format!("{what}: refused {why:?}"));
                            continue;
                        }
                    };
                    if let Err(why) =
                        model.validate(&Policy::certified(), &mut ProvenanceLog::new())
                    {
                        wrong.push(format!("{what}: the model does not validate: {why:?}"));
                        continue;
                    }
                    let solids = answer.solids();
                    let wanted_solids = usize::from(!(rule == Rule::Difference && it_first));
                    if solids.len() != wanted_solids {
                        wrong.push(format!(
                            "{what}: {} solids, wanted {wanted_solids}: {solids:?}",
                            solids.len()
                        ));
                        continue;
                    }
                    if wanted_solids == 0 {
                        continue;
                    }
                    let faces_of_each_shell = faces_of_each_shell(&model, solids[0]);
                    let wanted_shells: &[usize] = match rule {
                        Rule::Union => &[6],
                        Rule::Intersection => &[3],
                        Rule::Difference => &[3, 6],
                        _ => unreachable!("only the three rules are asked"),
                    };
                    if faces_of_each_shell != wanted_shells {
                        wrong.push(format!(
                            "{what}: shells of {faces_of_each_shell:?} faces, wanted \
                             {wanted_shells:?}"
                        ));
                    }
                    let found = volume_of(&model, solids[0]);
                    let (expected, near) = match rule {
                        Rule::Union => (block_volume, (found - block_volume).abs() < 1e-9),
                        Rule::Intersection => (
                            bar_volume,
                            found <= bar_volume
                                && (bar_volume - found) / bar_volume < HOW_FAR_A_MESH_MAY_BE,
                        ),
                        Rule::Difference => {
                            let expected = block_volume - bar_volume;
                            (
                                expected,
                                found >= expected
                                    && (found - expected) / expected < HOW_FAR_A_MESH_MAY_BE,
                            )
                        }
                        _ => unreachable!("only the three rules are asked"),
                    };
                    if !near {
                        wrong.push(format!(
                            "{what}: measured {found}, and the answer is {expected}"
                        ));
                    }
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A block of side four less a dome inside it, the top half of a ball of
/// radius one standing on its flat face at `z = 1`, the twin of the test above
/// with a ball where the bar was (queue row Q1903). Two placings: centred at
/// `(2, 1, 1)` the dome's rim touches the wall `y = 0` at one point, and at
/// `(2, 3, 1)` it touches `y = 4`.
///
/// The dome is cut out of the ball by a fuse, and that fuse starts the rim
/// circle at its point towards `−y`, the rim's one corner; the dome's round
/// face has no other. So at `(2, 1, 1)` the only corner the block less the dome
/// can ask is `(2, 0, 1)`, on the block's wall. The point across the rim from
/// it, `(2, 2, 1)`, is inside the block, and it is found through the foot of
/// the ball's centre on the rim's plane, where the bar's is found through the
/// bar's axis. With that ball case switched off by hand, both differences of
/// that placing, certified and tolerant, were refused `Undecidable` at *one
/// shell inside another* over `(2, 0, 1)` alone. At `(2, 3, 1)` the corner is
/// `(2, 2, 1)`, clear of the walls, and answers by itself.
///
/// The block less the dome is one solid of two shells, the block's six faces
/// and the dome's two, measuring `64 − 2π/3` to within what its mesh declares.
/// The union is the block, the intersection is the dome, and the dome less the
/// block is nothing.
#[test]
fn a_block_less_a_dome_whose_rim_touches_its_wall_keeps_the_dome_as_a_hollow() {
    let block_volume = 64.0;
    let dome_volume = 2.0 * PI / 3.0;

    let mut wrong: Vec<String> = Vec::new();
    for at in [[2.0, 1.0, 1.0], [2.0, 3.0, 1.0]] {
        for dome_first in [false, true] {
            for policy_name in ["certified", "tolerant"] {
                for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                    let mut model =
                        Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
                    let block = a_block(&mut model, [4.0, 4.0, 4.0]);
                    let dome = a_dome(&mut model, at);
                    let [first, second] = if dome_first {
                        [dome, block]
                    } else {
                        [block, dome]
                    };
                    let policy = if policy_name == "certified" {
                        Policy::certified()
                    } else {
                        Policy::tolerant(100.0)
                    };
                    let what =
                        format!("dome at {at:?}, {rule:?}, dome first {dome_first}, {policy_name}");
                    let answer = match model.fuse(
                        &[first],
                        &[second],
                        rule,
                        &policy,
                        &mut ProvenanceLog::new(),
                    ) {
                        Ok(answer) => answer,
                        Err(why) => {
                            wrong.push(format!("{what}: refused {why:?}"));
                            continue;
                        }
                    };
                    if let Err(why) =
                        model.validate(&Policy::certified(), &mut ProvenanceLog::new())
                    {
                        wrong.push(format!("{what}: the model does not validate: {why:?}"));
                        continue;
                    }
                    let solids = answer.solids();
                    let wanted_solids = usize::from(!(rule == Rule::Difference && dome_first));
                    if solids.len() != wanted_solids {
                        wrong.push(format!(
                            "{what}: {} solids, wanted {wanted_solids}: {solids:?}",
                            solids.len()
                        ));
                        continue;
                    }
                    if wanted_solids == 0 {
                        continue;
                    }
                    let (wanted_shells, expected): (&[usize], f64) = match rule {
                        Rule::Union => (&[6], block_volume),
                        Rule::Intersection => (&[2], dome_volume),
                        Rule::Difference => (&[2, 6], block_volume - dome_volume),
                        _ => unreachable!("only the three rules are asked"),
                    };
                    let faces_of_each_shell = faces_of_each_shell(&model, solids[0]);
                    if faces_of_each_shell != wanted_shells {
                        wrong.push(format!(
                            "{what}: shells of {faces_of_each_shell:?} faces, wanted \
                             {wanted_shells:?}"
                        ));
                    }
                    let measured = model
                        .solid(solids[0])
                        .expect("this model made the key")
                        .measure(
                            Deflection::of(A_DEFLECTION)
                                .expect("a fiftieth of a millimetre is a length"),
                            &Policy::certified(),
                            &mut ProvenanceLog::new(),
                        )
                        .expect("a solid this size measures");
                    if (measured.volume() - expected).abs() > measured.volume_error() {
                        wrong.push(format!(
                            "{what}: measured {}, and the answer is {expected}, out by more \
                             than the {} its mesh declares",
                            measured.volume(),
                            measured.volume_error()
                        ));
                    }
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A bar of radius one, two long, turned onto `x` and laid along the top of
/// the block `[0, 4]³` on the line `(x, 2, 4)`, `x` from 1 to 3, its seam
/// along that line (queue row Q2071).
///
/// **For the numbers the kernel is handed, the bar is clear of the block.**
/// The turn is [`Isometry::rotation`] about `y` by `π/2`, the `f64` nearest a
/// quarter turn, so the bar's axis is along `(1, 0, 2⁻⁵²)` and rises as it
/// goes. The bar's underside is clear of the block's top by
/// `1 − 1/√(1 + 2⁻¹⁰⁴)` at its near end and by more past it. So every rule,
/// both ways round, under `certified` and `tolerant`, answers as a bar clear
/// of a block does: the union is the two as they stand, the intersection no
/// solid, and each difference the first solid whole, every decision proved.
///
/// Every one used to be refused `Undecidable` at the side of the block's top
/// over the bar's seam corners: a corner moved by a turn that is not one of a
/// cube's is a box `1e−13` wide, and the plane runs through it. Q1765 reads
/// that side off the pipe's axis, but only where the plane misses the whole
/// pipe, and this one crosses it in an ellipse far past the bar's end. The
/// side is now read off the centre of the cap circle the corner is on, where
/// the plane is proved clear of that circle (Q2102).
#[test]
fn a_bar_turned_onto_x_by_a_float_angle_along_a_blocks_top_answers_every_rule() {
    every_fuse_of_a_block_and_a_bar_turned_onto_x_at(
        PI / 2.0,
        [1.0, 2.0, 5.0],
        a_bar_clear_of_the_block,
    );
}

/// The bar of the test above turned the other way, about `y` by `−π/2`, and
/// shifted to `(3, 2, 5)`, so it lies on the same line `(x, 2, 4)` from `x = 3`
/// to 1, its axis along `(−1, 0, 2⁻⁵²)` and its seam along its top (queue row
/// Q2141). It is clear of the block by the same `1 − 1/√(1 + 2⁻¹⁰⁴)` at its
/// near end, and every rule answers as the bar along the top does.
#[test]
fn a_bar_turned_back_onto_x_by_a_float_angle_along_a_blocks_top_answers_every_rule() {
    every_fuse_of_a_block_and_a_bar_turned_onto_x_at(
        -PI / 2.0,
        [3.0, 2.0, 5.0],
        a_bar_clear_of_the_block,
    );
}

/// What every rule gives for the block `[0, 4]³` and a bar of radius one, two
/// long, clear of it: the union is the two as they stand, the intersection no
/// solid, and each difference the first solid whole.
fn a_bar_clear_of_the_block(rule: Rule, bar_first: bool) -> Vec<ShellsAndVolume> {
    match (rule, bar_first) {
        (Rule::Union, _) => vec![(vec![3], 2.0 * PI), (vec![6], 64.0)],
        (Rule::Intersection, _) => vec![],
        (Rule::Difference, false) => vec![(vec![6], 64.0)],
        (Rule::Difference, true) => vec![(vec![3], 2.0 * PI)],
        _ => unreachable!("only the three rules are asked"),
    }
}

/// The bar of the test above sunk into the block to lie on its floor along
/// `(x, 2, 0)`, its axis from `(1, 2, 1)`, and clear of the floor by the same
/// `1 − 1/√(1 + 2⁻¹⁰⁴)` (queue row Q2071). The bar is inside the block, so the
/// union is the block alone, the intersection the bar alone, and the bar less
/// the block nothing. The block less the bar is one solid of two shells, the
/// block's six faces and the bar's three as a hollow, measuring `64 − 2π`.
///
/// The block less the bar was refused `Undecidable` at the side of the floor
/// over the bar's seam corners, as the bar along the top was.
#[test]
fn a_block_less_a_bar_turned_onto_x_by_a_float_angle_along_its_floor_keeps_the_bar_as_a_hollow() {
    every_fuse_of_a_block_and_a_bar_turned_onto_x_at(
        PI / 2.0,
        [1.0, 2.0, 1.0],
        a_bar_inside_the_block,
    );
}

/// The bar turned about `y` by `−π/2` sunk onto the block's floor along
/// `(x, 2, 0)`, its axis from `(3, 2, 1)` (queue row Q2141): the answers of the
/// test above.
#[test]
fn a_block_less_a_bar_turned_back_onto_x_by_a_float_angle_along_its_floor_keeps_the_bar_as_a_hollow()
 {
    every_fuse_of_a_block_and_a_bar_turned_onto_x_at(
        -PI / 2.0,
        [3.0, 2.0, 1.0],
        a_bar_inside_the_block,
    );
}

/// What every rule gives for the block `[0, 4]³` and a bar of radius one, two
/// long, inside it clear of its faces: the union is the block, the
/// intersection the bar, the block less the bar the block with the bar as a
/// hollow, and the bar less the block nothing.
fn a_bar_inside_the_block(rule: Rule, bar_first: bool) -> Vec<ShellsAndVolume> {
    match (rule, bar_first) {
        (Rule::Union, _) => vec![(vec![6], 64.0)],
        (Rule::Intersection, _) => vec![(vec![3], 2.0 * PI)],
        (Rule::Difference, false) => vec![(vec![3, 6], 64.0 - 2.0 * PI)],
        (Rule::Difference, true) => vec![],
        _ => unreachable!("only the three rules are asked"),
    }
}

/// Every rule, both ways round, under `certified` and `tolerant`, of the block
/// `[0, 4]³` and [`a_bar_turned_onto_x`] by `turn` from `at`: never refused,
/// every decision proved, the model validates, and the answer's solids, fewest
/// faces first, have the faces in each shell and the volume `should` gives for
/// the rule and whether the bar came first.
fn every_fuse_of_a_block_and_a_bar_turned_onto_x_at(
    turn: f64,
    at: [f64; 3],
    should: fn(Rule, bool) -> Vec<ShellsAndVolume>,
) {
    let mut wrong: Vec<String> = Vec::new();
    for bar_first in [false, true] {
        for policy_name in ["certified", "tolerant"] {
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                let mut model =
                    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
                let block = a_block(&mut model, [4.0, 4.0, 4.0]);
                let bar = a_bar_turned_onto_x(&mut model, turn, at);
                let [first, second] = if bar_first {
                    [bar, block]
                } else {
                    [block, bar]
                };
                let policy = if policy_name == "certified" {
                    Policy::certified()
                } else {
                    Policy::tolerant(100.0)
                };
                let what = format!("{rule:?}, bar first {bar_first}, {policy_name}");
                let mut log = ProvenanceLog::new();
                let answer = match model.fuse(&[first], &[second], rule, &policy, &mut log) {
                    Ok(answer) => answer,
                    Err(why) => {
                        wrong.push(format!("{what}: refused {why:?}"));
                        continue;
                    }
                };
                if !log.records().is_empty() {
                    wrong.push(format!(
                        "{what}: not every decision is proved: {:?}",
                        log.records()
                    ));
                }
                if let Err(why) = model.validate(&Policy::certified(), &mut ProvenanceLog::new()) {
                    wrong.push(format!("{what}: the model does not validate: {why:?}"));
                    continue;
                }
                let mut found: Vec<ShellsAndVolume> = answer
                    .solids()
                    .iter()
                    .map(|&solid| (faces_of_each_shell(&model, solid), volume_of(&model, solid)))
                    .collect();
                found.sort_by(|one, other| one.0.cmp(&other.0));
                let expected = should(rule, bar_first);
                let near = found.len() == expected.len()
                    && found.iter().zip(&expected).all(
                        |((shells, volume), (wanted_shells, wanted_volume))| {
                            shells == wanted_shells
                                && ((wanted_volume - volume) / wanted_volume).abs()
                                    < HOW_FAR_A_MESH_MAY_BE
                        },
                    );
                if !near {
                    wrong.push(format!(
                        "{what}: the faces of each shell and the volume of each solid are \
                         {found:?}, and the answer is {expected:?}"
                    ));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// One solid of an answer: the faces in each of its shells, and its volume.
type ShellsAndVolume = (Vec<usize>, f64);

/// A bar of radius one, two long, turned onto `x` by [`Isometry::rotation`]
/// about `y` by `turn` and then shifted to `at`. Turned by `π/2`, its axis runs
/// from `at` along `(1, 0, 2⁻⁵²)`, and its seam along its underside, a radius
/// below the axis; turned by `−π/2`, along `(−1, 0, 2⁻⁵²)` with its seam along
/// its top.
fn a_bar_turned_onto_x(model: &mut Model, turn: f64, [x, y, z]: [f64; 3]) -> SolidKey {
    let onto_x = Isometry::rotation(&Axis::y(), turn).expect("a finite angle");
    let shift = Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift");
    model
        .add_round_bar(1.0, 2.0, &shift.compose(&onto_x))
        .expect("a bar")
}

/// The top half of a ball of radius one centred at `centre`, cut off flat
/// at its equator by a box laid over it.
fn a_dome(model: &mut Model, [x, y, z]: [f64; 3]) -> SolidKey {
    let at_the_centre = Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift");
    let ball = model.add_ball(1.0, &at_the_centre).expect("a ball");
    let square = ProfileBuilder::new(vec![[-1.5, -1.5], [1.5, -1.5], [1.5, 1.5], [-1.5, 1.5]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let lid = model
        .add_extrusion(&square, 1.5, &at_the_centre)
        .expect("building a box needs no decision");
    let dome = model
        .fuse(
            &[ball],
            &[lid],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a ball cut at its equator by a box over its top half");
    let [dome] = dome.solids() else {
        panic!("the top half of a ball is one solid: {:?}", dome.solids());
    };
    *dome
}

/// How many faces each shell of a solid has, fewest first.
fn faces_of_each_shell(model: &Model, solid: SolidKey) -> Vec<usize> {
    let mut faces: Vec<usize> = model
        .solid(solid)
        .expect("this model made the key")
        .shells()
        .map(|shell| shell.faces().count())
        .collect();
    faces.sort_unstable();
    faces
}
