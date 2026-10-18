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
//!
//! **The same bar turned the other way, by `−π/2`, is not clear of the block:
//! it dips into it by `2⁻¹⁰⁵`** (queue row Q2141). Its cap at the low end is
//! stored `2⁻⁵²` past the end of its axis, since the offset the cap's plane
//! needs falls halfway between two `f64`s and rounds outward, and there the
//! pipe's underside is already below the block's top. So the cap circle crosses
//! the top, and the two share a sliver.
//! `the_cap_circle_of_a_bar_turned_back_onto_x_crosses_the_blocks_top_and_floor`
//! proves it. That sliver is the answer, and any answer has an edge between
//! the two points the cap circle crosses the top at, `2⁻⁵¹` apart. So every
//! certified fuse is refused `DynamicRangeTooWide`, §3.5's limit: the spread is
//! `6 · 2⁵¹`, where the kernel works to `1e11`. It was refused `Undecidable`
//! at the order of those two points along a crossing, and then at which way
//! three surfaces turn at them, until queue row Q2160 read both off which of
//! the two meets of one line a point is.
//!
//! **Under `tolerant`, every fuse answers but the three whose answer is the
//! sliver alone, which are refused `CannotTellWhichWayASolidsFacesPoint`**
//! (queue rows Q2161, Q2195 and Q2238). Every decision the arrangement takes
//! is proved. No double lies inside the sliver: it spans `x` from `3 + 2⁻⁵³`
//! to `3 + 2⁻⁵²`, and the doubles there are `2⁻⁵¹` apart. So every point drawn
//! on it rounds onto `x = 3`. The cap circle's two crossings are stored at
//! `y = 2 − 2⁻⁵¹` and `y = 2`, and the one point `mesh.rs` puts inside each of
//! the sliver's two arcs at `y = 2 − 2⁻⁵²`, between them. The hole in the
//! block's face is then three points on one line, and the sliver's piece of
//! the pipe names that one point for both its arcs. Until Q2238 the mesh that
//! checks each answer before it is written refused nine fuses there,
//! `CannotMeshAFace`, `LoopsTouch` for the hole and `CornersCoincide` for the
//! pipe; it now draws the hole as a fan and cuts the pipe's loop in two at the
//! point it names twice. The sliver alone then meshes to points all on one
//! line, so its mesh encloses exactly nothing, and `validate.rs`, which reads
//! which way a curved solid's faces point off the sign of what its mesh
//! encloses, refuses it; the answer is there, so that is a gap, left to a row
//! of its own. Each other answer validates under `tolerant` and is refused
//! `DynamicRangeTooWide` under `certified`, which is what a tolerant fuse
//! promises, and right: valid is a property of a model and a policy
//! (`Model::fuse`'s doc, §68).
//! `a_bar_dipping_through_a_blocks_top_leaves_an_edge_too_short_for_its_coordinates`
//! and its twin on the floor pin all of it.
//!
//! **The same bar turned twice, so that its seam is underneath, shares a
//! sliver with the block along nearly its whole length** (queue row Q2187).
//! Turned about `x` by `π` and then about `y` by `π/2`, its axis runs along
//! `(−1, −1.2e−16, −2⁻⁵²)`, so its far end drops `4.4e−16` and its far cap's
//! circle crosses the top. Every fuse was refused at the side of the top over
//! the far seam corner, the lowest point of that circle, in a box the top runs
//! through. The top reaches the circle, so Q2102's reading does not answer;
//! `fuse/arrange.rs` now reads the side off which arc of the circle the box is
//! on, between the two points `meet_of_three_surfaces` proves the circle
//! crosses the top at, and the near seam corner's off the near cap's circle,
//! which is clear of the top.
//!
//! **Every answer of that bar holds a feature too small for its coordinates,
//! so every certified fuse is refused `DynamicRangeTooWide`** (queue row
//! Q2237). An answer holding the sliver holds the piece of the far cap that
//! closes it, a segment of the cap's circle `4.4e−16` thick, and one holding
//! the bar less the sliver holds its pipe, where the arc the top cuts turns
//! about `5e−16` from the near seam corner, the corner of the pipe's other
//! loop. Both are §3.5's limit, at coordinates of 4 to 6. Under `tolerant`
//! every fuse answers with one record of that spread, and the model validates
//! under `tolerant` alone. The union with the block first was refused meshing
//! the pipe until queue row Q2238.
//! `a_bar_turned_twice_with_its_seam_underneath_dips_through_a_blocks_top` and
//! its twin on the floor pin them.
//!
//! **A bar standing on the block's top flush with its edge answers every
//! rule** (queue row Q2257). Its foot touches the top's outline at one point
//! that is no corner of it. Q2237's measure reads a corner of one loop on a
//! side of another as a neck of no width, since it floors the distance at one
//! rounding, but the fuse never writes such a face: it cuts the top's edge at
//! the touch and writes the foot into the outline, one loop through the touch
//! twice. The same top written by hand with `SolidBuilder` as an outline and a
//! hole is refused `DynamicRangeTooWide`, the wrong name for a touch.
//! `a_bar_standing_on_a_blocks_top_flush_with_its_edge_answers_every_rule` and
//! `a_top_whose_hole_touches_its_outline_at_a_point_is_refused_as_too_thin`
//! pin the two.
//!
//! **A thousand float turn angles about `z`, swept over the same cube and bar
//! turned together, land in the same three bins Q1765 found and the same
//! single limit `Q2237` and `Q2257` already pin** (queue row Q2024). Of 2000
//! `(angle, wall)` pairs, angle strictly between `0` and a quarter turn and
//! wall `4.0` or `2.0`, the turned wall's plane misses the bar's pipe on 930,
//! touches it on none, and crosses it on 1070; every fuse answers right on
//! every one, and 3745 of the crossing pairs' rule-and-order combinations
//! refuse `DynamicRangeTooWide`, never `Undecidable`. None of those refusals
//! go through `Arrangement::passes_through` or `which_side_of_everything`'s
//! ring-fallback chain (`fuse/arrange.rs`) at all: they are the same
//! coordinate-spread-past-`2^53` sliver the bar turned twice already pins, at
//! the same coordinates of 4 to 6, so no new row is filed for them (`docs/decisions.md`
//! §3.5, §16.10). `a_thousand_turns_of_a_cube_and_a_bar` pins it, with the
//! fuller argument in its own doc comment.

use core::f64::consts::PI;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use rcad_geom::{
    Axis, Cylinder, Deflection, DynamicRange, EdgeRun, Isometry, MeetOfThree, Orientation, Plane,
    Point, Scale, Surface, SurfaceMeet, Vector, intersect, meet_of_three_surfaces,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftStep, EdgeKey, Error, Fused, Invalid, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey,
};

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

/// A thousand float turn angles about `z`, strictly between `0` and a quarter
/// turn, over the same pair as
/// [`a_bar_on_a_wall_both_turned_by_a_float_angle_answer_every_rule`]: the cube
/// `4 × 4 × 4` or `4 × 4 × 2` and the bar of radius one, two high, at
/// `(−1, 2, 1)`, the cube placed by `Isometry::rotation` and the bar by that
/// turn composed with its shift (queue row Q2024).
///
/// **The two solids turn together, so only rounding moves them apart.** `turn`
/// places both, so the configuration *between* them never changes — the bar
/// sits exactly where it always does relative to the wall — and the only thing
/// a generic angle changes is how `Isometry::rotation`'s own rounding lands.
/// `intersect` sorts every angle into one of three bins: the wall's plane
/// misses the bar's pipe, as at `π/6` and one radian
/// (queue row Q1765, a miss by `5.3e−17` or `2.1e−16`), crosses it in two
/// straight lines a rounding apart, or runs along it touching in one.
///
/// **Every refusal found is the same too-thin-to-compute limit
/// [`a_bar_turned_twice_with_its_seam_underneath_dips_through_a_blocks_top`]
/// already pins** (queue rows Q2237 and Q2257): a coordinate spread past
/// `2^53`, the point past which a feature written straight into the
/// coordinates is gone (`docs/decisions.md` §3.5). None of the 3745 refusals
/// below are `Undecidable`; every one is `DynamicRangeTooWide`, the model's
/// own check of how wide its coordinates spread over its smallest feature,
/// which a fuse asks behind a guard separate from the sweep that
/// `Undecidable` would come from. A wrong answer, caught by
/// inclusion-exclusion rather than by name, would be a different kind of
/// fault and always fails the test; a refusal of this one kind is pinned by
/// count instead, since §16.10 says a limit like this is deferred, not fixed.
#[test]
fn a_thousand_turns_of_a_cube_and_a_bar() {
    const HOW_MANY: usize = 1000;
    let mut miss = 0usize;
    let mut touch = 0usize;
    let mut cross = 0usize;
    let mut refusals: Vec<String> = Vec::new();
    let mut wrong: Vec<String> = Vec::new();
    for step in 0..HOW_MANY {
        let angle = (step as f64 + 0.5) / HOW_MANY as f64 * (PI / 2.0);
        let turn = Isometry::rotation(&Axis::z(), angle).expect("a finite angle");
        for wall in [4.0, 2.0] {
            match how_a_turned_wall_meets_the_pipe(&turn, wall) {
                SurfaceMeet::Nothing => {
                    miss += 1;
                    continue;
                }
                SurfaceMeet::Curves(runs) if runs.len() == 1 => touch += 1,
                SurfaceMeet::Curves(runs) if runs.len() == 2 => cross += 1,
                other => panic!(
                    "a plane that holds a pipe's own axis direction only ever misses it, \
                     touches it in one line, or crosses it in two: angle {angle}, wall \
                     {wall}, got {other:?}"
                ),
            }
            let answers = how_every_fuse_of_a_turned_bar_against_a_wall_answers(&turn, wall, angle);
            refusals.extend(answers.refusals);
            wrong.extend(answers.wrong);
        }
    }
    println!(
        "miss {miss} touch {touch} cross {cross} of {} angles",
        HOW_MANY * 2
    );
    assert_eq!(
        miss, 930,
        "how many of the 2000 angle-and-wall pairs the wall's plane misses the pipe on"
    );
    assert_eq!(
        cross, 1070,
        "how many of the 2000 pairs the wall's plane crosses the pipe on (touch is measure zero and was not seen once in 1000 angles)"
    );
    assert_eq!(
        miss + touch + cross,
        HOW_MANY * 2,
        "every angle lands in exactly one bin"
    );
    println!("{} refusals, {} wrong", refusals.len(), wrong.len());
    assert!(
        wrong.is_empty(),
        "a wrong answer is always a regression:\n{}",
        wrong.join("\n")
    );
    let not_the_known_limit: Vec<&String> = refusals
        .iter()
        .filter(|refusal| {
            !refusal.contains("refused the model is broken: coordinates span a range of")
        })
        .collect();
    assert!(
        not_the_known_limit.is_empty(),
        "every refusal here should be DynamicRangeTooWide, and these are not:\n{not_the_known_limit:#?}"
    );
    assert_eq!(
        refusals.len(),
        3745,
        "every refusal here is the known DynamicRangeTooWide sliver limit (see the doc \
         comment above); a count that moves is either that limit's shape changing or a new \
         kind of refusal, and either needs a look:\n{}",
        refusals.join("\n")
    );
}

/// Where `intersect` places the wall of
/// [`a_thousand_turns_of_a_cube_and_a_bar`]'s cube, turned by `turn` and
/// `wall` high, against the bar's own skin.
fn how_a_turned_wall_meets_the_pipe(turn: &Isometry, wall: f64) -> SurfaceMeet {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let cube = model
        .add_extrusion(&rectangle, wall, turn)
        .expect("building a box needs no decision");
    let shift = Isometry::translation(Vector::new(-1.0, 2.0, 1.0)).expect("a short shift");
    let bar = model
        .add_round_bar(1.0, 2.0, &turn.compose(&shift))
        .expect("a bar");
    the_wall_and_the_pipe_meet(&model, cube, bar)
}

/// What [`how_every_fuse_of_a_turned_bar_against_a_wall_answers`] found: a
/// refusal, named directly, and a wrong answer, caught by inclusion-exclusion
/// instead — kept apart because only the second is ever a regression that
/// should fail the test on its own (`docs/decisions.md` §16.10).
struct AnswerFaults {
    refusals: Vec<String>,
    wrong: Vec<String>,
}

/// Every rule both ways round, under `certified`, for the cube and the bar of
/// [`a_thousand_turns_of_a_cube_and_a_bar`] at one angle and one wall height,
/// whatever `intersect` says of the wall and the pipe. A refusal is named
/// directly; a wrong answer is caught by inclusion-exclusion — the union, and
/// each difference, measured against the intersection's own volume, however
/// that volume was reached — without needing a closed form for the overlap
/// itself.
fn how_every_fuse_of_a_turned_bar_against_a_wall_answers(
    turn: &Isometry,
    wall: f64,
    angle: f64,
) -> AnswerFaults {
    let bar_at = [-1.0, 2.0, 1.0];
    let union_unswapped =
        how_a_turned_bar_against_a_wall_fuses(turn, bar_at, wall, false, Rule::Union);
    let union_swapped =
        how_a_turned_bar_against_a_wall_fuses(turn, bar_at, wall, true, Rule::Union);
    let meet_unswapped =
        how_a_turned_bar_against_a_wall_fuses(turn, bar_at, wall, false, Rule::Intersection);
    let meet_swapped =
        how_a_turned_bar_against_a_wall_fuses(turn, bar_at, wall, true, Rule::Intersection);
    let cube_less_bar =
        how_a_turned_bar_against_a_wall_fuses(turn, bar_at, wall, false, Rule::Difference);
    let bar_less_cube =
        how_a_turned_bar_against_a_wall_fuses(turn, bar_at, wall, true, Rule::Difference);

    let mut refusals = Vec::new();
    for (what, got) in [
        ("Union, cube first", &union_unswapped),
        ("Union, bar first", &union_swapped),
        ("Intersection, cube first", &meet_unswapped),
        ("Intersection, bar first", &meet_swapped),
        ("cube less the bar", &cube_less_bar),
        ("bar less the cube", &bar_less_cube),
    ] {
        if let Err(why) = got {
            refusals.push(format!("angle {angle}, wall {wall}, {what}: {why}"));
        }
    }

    let mut wrong = Vec::new();
    let cube_volume = 16.0 * wall;
    let bar_volume = 2.0 * PI;
    let the_overlap: Vec<(&str, f64)> = [
        (
            "Intersection, cube first",
            meet_unswapped.as_ref().ok().copied(),
        ),
        (
            "Intersection, bar first",
            meet_swapped.as_ref().ok().copied(),
        ),
        (
            "the cube's volume less what cube less the bar left",
            cube_less_bar.as_ref().ok().map(|found| cube_volume - found),
        ),
        (
            "the bar's volume less what bar less the cube left",
            bar_less_cube.as_ref().ok().map(|found| bar_volume - found),
        ),
    ]
    .into_iter()
    .filter_map(|(what, found)| found.map(|found| (what, found)))
    .collect();
    if let [(_, first), rest @ ..] = the_overlap.as_slice() {
        for (what, found) in rest {
            if (found - first).abs() > HOW_FAR_A_MESH_MAY_BE * first.abs().max(found.abs()).max(1.0)
            {
                wrong.push(format!(
                    "angle {angle}, wall {wall}: {what} puts the overlap at {found}, and \
                     Intersection, cube first puts it at {first}"
                ));
            }
        }
        if let Ok(found) = &union_unswapped {
            let expected = cube_volume + bar_volume - first;
            if (found - expected).abs() > HOW_FAR_A_MESH_MAY_BE * expected {
                wrong.push(format!(
                    "angle {angle}, wall {wall}: Union, cube first measured {found}, and the \
                     overlap says {expected}"
                ));
            }
        }
    }
    AnswerFaults { refusals, wrong }
}

/// The total volume [`Model::fuse`] answers for the cube and the bar of
/// [`a_thousand_turns_of_a_cube_and_a_bar`], whatever `intersect` says of the
/// wall's plane and the bar's pipe: clear of it, touching it in one line, or
/// crossing it in two. `Err` says what is wrong with the fuse; a model that
/// does not validate is one of them.
fn how_a_turned_bar_against_a_wall_fuses(
    turn: &Isometry,
    bar_at: [f64; 3],
    wall: f64,
    swapped: bool,
    rule: Rule,
) -> Result<f64, String> {
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
    let [first, second] = if swapped { [bar, cube] } else { [cube, bar] };
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
    Ok(answer
        .solids()
        .iter()
        .map(|&solid| volume_of(&model, solid))
        .sum())
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
    matches!(
        the_wall_and_the_pipe_meet(model, cube, bar),
        SurfaceMeet::Nothing
    )
}

/// Where `intersect` places the cube's upright wall nearest the bar, as the
/// model holds it placed, against the bar's own skin.
fn the_wall_and_the_pipe_meet(model: &Model, cube: SolidKey, bar: SolidKey) -> SurfaceMeet {
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
    intersect(
        &Surface::Plane(wall),
        &skin,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a plane against a pipe is always named")
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
    every_fuse_of_the_block_and_a_bar(
        |model| a_bar_turned_onto_x(model, PI / 2.0, [1.0, 2.0, 5.0]),
        a_bar_clear_of_the_block,
    );
}

/// Where the block's top or floor, the cap at the low end of a bar turned onto
/// `x`, and the bar's pipe meet, read off the surfaces the model holds placed
/// (queue row Q2141).
///
/// **Turned the other way, by `−π/2`, the bar dips into the block.** From
/// `(3, 2, 5)` its axis runs along `(−1, 0, 2⁻⁵²)`, and its cap there is the
/// plane `(1, 0, −2⁻⁵²) · p = 3 − 4·2⁻⁵²`. The plane square to the axis through
/// its start has `3 − 5·2⁻⁵²` there, which falls halfway between two `f64`s
/// and rounds to the even one, so the cap stands `2⁻⁵²` past the axis's end,
/// where the pipe's underside is `2⁻¹⁰⁵` below the block's top. The three meet
/// in two points round `(3, 2, 4)`, proved under `certified`: the cap circle
/// crosses the top, and the bar and the block share a sliver. Sunk to
/// `(3, 2, 1)`, the cap's `3 − 2⁻⁵²` rounds to `3` and the same happens round
/// `(3, 2, 0)`.
///
/// The bars turned by `π/2` that Q2102 lays from `(1, 2, 5)` and `(1, 2, 1)`
/// have caps at `−1 − 5·2⁻⁵²` and `−1 − 2⁻⁵²`, which `f64` holds, and meet the
/// top and the floor nowhere.
#[test]
fn the_cap_circle_of_a_bar_turned_back_onto_x_crosses_the_blocks_top_and_floor() {
    let mut wrong: Vec<String> = Vec::new();
    for (turn, at, height, cap_offset, crosses) in [
        (
            -PI / 2.0,
            [3.0, 2.0, 5.0],
            4.0,
            3.0 - 4.0 * f64::EPSILON,
            true,
        ),
        (-PI / 2.0, [3.0, 2.0, 1.0], 0.0, 3.0, true),
        (
            PI / 2.0,
            [1.0, 2.0, 5.0],
            4.0,
            -1.0 - 5.0 * f64::EPSILON,
            false,
        ),
        (PI / 2.0, [1.0, 2.0, 1.0], 0.0, -1.0 - f64::EPSILON, false),
    ] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let block = a_block(&mut model, [4.0, 4.0, 4.0]);
        let bar = a_bar_turned_onto_x(&mut model, turn, at);
        let (plane, cap, pipe) = the_plane_the_low_cap_and_the_pipe(&model, block, bar, height);
        let mut log = ProvenanceLog::new();
        let meet = meet_of_three_surfaces(
            [
                &Surface::Plane(plane),
                &Surface::Plane(cap),
                &Surface::Cylinder(pipe),
            ],
            &Policy::certified(),
            &mut log,
        );
        let as_it_should = if crosses {
            matches!(&meet, Ok(MeetOfThree::At(points)) if points.len() == 2
            && points.iter().all(|point| {
                point
                    .enclosure()
                    .iter()
                    .zip([at[0], 2.0, height])
                    .all(|(side, near)| near - 1e-14 < side.lo() && side.hi() < near + 1e-14)
            }))
        } else {
            matches!(meet, Ok(MeetOfThree::Nowhere))
        };
        if !as_it_should || !log.records().is_empty() || cap.offset() != cap_offset {
            wrong.push(format!(
                "turned {turn} from {at:?}: {meet:?}, records {:?}, cap {cap:?}, pipe {pipe:?}",
                log.records()
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The block's face at `height`, the cap of the bar whose plane runs nearest
/// the start of the bar's axis, and the bar's pipe, as the model holds them
/// placed.
fn the_plane_the_low_cap_and_the_pipe(
    model: &Model,
    block: SolidKey,
    bar: SolidKey,
    height: f64,
) -> (Plane, Plane, Cylinder) {
    let plane = model
        .solid(block)
        .expect("this model made the key")
        .faces()
        .map(|face| face.plane())
        .find(|plane| {
            plane.normal().z() != 0.0
                && plane.signed_distance_to(Point::new(2.0, 2.0, height)) == 0.0
        })
        .expect("the block has a face at that height");
    let surfaces: Vec<Surface> = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .collect();
    let Some(Surface::Cylinder(pipe)) = surfaces
        .iter()
        .find(|surface| matches!(surface, Surface::Cylinder(_)))
    else {
        panic!("a bar has a round skin: {surfaces:?}");
    };
    let cap = surfaces
        .iter()
        .filter_map(|surface| {
            if let Surface::Plane(cap) = surface {
                Some(cap)
            } else {
                None
            }
        })
        .min_by(|one, other| {
            let off = |cap: &Plane| cap.signed_distance_to(pipe.axis().origin()).abs();
            off(one).total_cmp(&off(other))
        })
        .expect("a bar has two caps");
    (plane, *cap, *pipe)
}

/// The bar of the test above turned the other way, about `y` by `−π/2`, and
/// shifted to `(3, 2, 5)`, so it lies on the same line `(x, 2, 4)` from `x = 3`
/// to 1, its axis along `(−1, 0, 2⁻⁵²)` and its seam along its top (queue row
/// Q2141). Its low cap is stored `2⁻⁵²` past the axis's end, so the bar dips
/// `2⁻¹⁰⁵` into the block, and the cap's circle crosses the top in two points
/// `2⁻⁵¹` apart.
///
/// **Every certified fuse, both ways round, is refused `DynamicRangeTooWide`,
/// and that is §3.5's limit, not a gap.** Any answer has an edge between those
/// two points, `2⁻⁵¹` long against the bar's top at height 6, a spread of
/// `6 · 2⁵¹` where the kernel works to `1e11`. Until queue row Q2160 they were
/// refused `Undecidable` first at the order of the two points along a
/// crossing, whose boxes overlap, and then at which way the top, the cap and
/// the pipe turn at each of them. `fuse/arrange.rs` now reads both off which
/// of the two meets a point is.
///
/// Every tolerant fuse but the intersection answers and validates under
/// `tolerant` alone (queue rows Q2161, Q2195 and Q2238), with the faces
/// [`what_a_bar_dipping_through_the_top_gives`]. The intersection, the sliver
/// alone, is refused both ways round
/// ([`what_tolerant_gives_for_a_bar_turned_once_dipping_through_the_top`]).
#[test]
fn a_bar_dipping_through_a_blocks_top_leaves_an_edge_too_short_for_its_coordinates() {
    every_fuse_of_the_block_and_a_bar_dipping_through_it(
        |model| a_bar_turned_onto_x(model, -PI / 2.0, [3.0, 2.0, 5.0]),
        what_tolerant_gives_for_a_bar_turned_once_dipping_through_the_top,
    );
}

/// What a tolerant fuse of the block and the bar turned once, dipping through
/// its top, gives: the faces [`what_a_bar_dipping_through_the_top_gives`], but
/// for the intersection, the sliver alone, which holds no double in `x`.
fn what_tolerant_gives_for_a_bar_turned_once_dipping_through_the_top(
    rule: Rule,
    bar_first: bool,
) -> WhatTolerantGives {
    if matches!(rule, Rule::Intersection) {
        WhatTolerantGives::TheSliverAloneIsRefused
    } else {
        WhatTolerantGives::Faces(what_a_bar_dipping_through_the_top_gives(rule, bar_first))
    }
}

/// What one tolerant fuse of the block and a bar dipping through it gives.
enum WhatTolerantGives {
    /// An answer, with these faces in each shell of each solid.
    Faces(Vec<Vec<usize>>),
    /// The sliver alone, whose every point rounds onto one line, refused
    /// `CannotTellWhichWayASolidsFacesPoint` with its mesh enclosing exactly
    /// nothing: `validate.rs` reads which way a curved solid's faces point off
    /// the sign of what its mesh encloses. The answer is there, so this is a
    /// gap, left by queue row Q2238 to a row of its own.
    TheSliverAloneIsRefused,
}

/// The faces in each shell of each solid a tolerant fuse of the block and a bar
/// dipping through its top in a sliver gives. The union is one shell of nine
/// faces, the block's six, the top holding the hole the sliver leaves, and the
/// bar's three; the intersection the sliver's three; the block less the bar the
/// block's six and the sliver's pieces of the pipe and the cap, a groove; and
/// the bar less the block its three and the piece of the top that cuts the
/// sliver off.
fn what_a_bar_dipping_through_the_top_gives(rule: Rule, bar_first: bool) -> Vec<Vec<usize>> {
    match (rule, bar_first) {
        (Rule::Union, _) => vec![vec![9]],
        (Rule::Intersection, _) => vec![vec![3]],
        (Rule::Difference, false) => vec![vec![8]],
        (Rule::Difference, true) => vec![vec![4]],
        _ => unreachable!("only the three rules are asked"),
    }
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
    every_fuse_of_the_block_and_a_bar(
        |model| a_bar_turned_onto_x(model, PI / 2.0, [1.0, 2.0, 1.0]),
        a_bar_inside_the_block,
    );
}

/// The bar turned about `y` by `−π/2` sunk onto the block's floor along
/// `(x, 2, 0)`, its axis from `(3, 2, 1)` (queue row Q2141), which dips through
/// the floor as the bar along the top dips through the top. Every certified
/// fuse is refused `DynamicRangeTooWide` as there, the edge `2⁻⁵¹` long against
/// the block's corners at 4, a spread of `2⁵³`. Every tolerant one but the bar
/// less the block answers and validates under `tolerant` alone (queue rows
/// Q2161, Q2195 and Q2238), with the faces
/// [`what_a_bar_dipping_through_the_floor_gives`]. The bar less the block, the
/// sliver alone, is refused as the intersection on the top is
/// ([`what_tolerant_gives_for_a_bar_turned_once_dipping_through_the_floor`]).
#[test]
fn a_bar_dipping_through_a_blocks_floor_leaves_an_edge_too_short_for_its_coordinates() {
    every_fuse_of_the_block_and_a_bar_dipping_through_it(
        |model| a_bar_turned_onto_x(model, -PI / 2.0, [3.0, 2.0, 1.0]),
        what_tolerant_gives_for_a_bar_turned_once_dipping_through_the_floor,
    );
}

/// What a tolerant fuse of the block and the bar turned once, dipping through
/// its floor, gives: the faces [`what_a_bar_dipping_through_the_floor_gives`],
/// but for the bar less the block, the sliver alone.
fn what_tolerant_gives_for_a_bar_turned_once_dipping_through_the_floor(
    rule: Rule,
    bar_first: bool,
) -> WhatTolerantGives {
    if matches!(rule, Rule::Difference) && bar_first {
        WhatTolerantGives::TheSliverAloneIsRefused
    } else {
        WhatTolerantGives::Faces(what_a_bar_dipping_through_the_floor_gives(rule, bar_first))
    }
}

/// The faces in each shell of each solid a tolerant fuse of the block and a bar
/// dipping through its floor in a sliver gives. The union is one shell of eight
/// faces, the block's six and the sliver's pieces of the pipe and the cap; the
/// intersection the bar's three and the piece of the floor that cuts the
/// sliver off; the block less the bar a hollow open through the floor in nine,
/// the floor holding the hole the sliver leaves; and the bar less the block
/// the sliver's three.
fn what_a_bar_dipping_through_the_floor_gives(rule: Rule, bar_first: bool) -> Vec<Vec<usize>> {
    match (rule, bar_first) {
        (Rule::Union, _) => vec![vec![8]],
        (Rule::Intersection, _) => vec![vec![4]],
        (Rule::Difference, false) => vec![vec![9]],
        (Rule::Difference, true) => vec![vec![3]],
        _ => unreachable!("only the three rules are asked"),
    }
}

/// A bar laid along the block `[0, 4]³`'s top from `(3, 2, 5)` to `x = 1`, as
/// the bar dipping through it is, but turned by [`Isometry::rotation`] about
/// `y` by `π/2` and then about `x` by `π`, which leaves its seam along its
/// underside (queue row Q2187). Its axis runs along `(−1, −1.2e−16, −2⁻⁵²)`,
/// so its far end drops `4.4e−16` and its far cap's circle crosses the top at
/// `y = 2 ± 2.98e−8`. The bar and the block share a sliver from there back to
/// the bar's near end, where its near cap's circle is clear of the top by
/// about `1e−31`.
///
/// **Every certified fuse is refused `DynamicRangeTooWide`, and that is §3.5's
/// limit, not a gap** (queue row Q2237). The intersection, the sliver, and the
/// block less the bar, a groove the sliver leaves, each hold the piece of the
/// far cap the sliver ends in: a segment of the cap's circle between the two
/// points it crosses the top at, `5.96e−8` apart, and so `4.4e−16` thick. The
/// union and the bar less the block each hold the bar's pipe less the sliver,
/// a face of two loops: the near cap's circle, whose one corner is the near
/// seam corner `(3, 2, 4)`, and the arc the top cuts the pipe in with the far
/// cap's long arc. That arc turns at its tip about `5e−16` from the near seam
/// corner, and the corner is stored on the top itself. Either way the spread
/// is past `1e15` at coordinates of 4 to 6. Until Q2237 the range read only
/// the edges' chords, `5.96e−8` at the shortest, and every fuse answered.
///
/// Under `tolerant` every fuse answers with one record of that spread, every
/// other decision proved, and the model validates under `tolerant` alone: the
/// union with the bar first is one shell of the block's six faces and the
/// bar's three, the intersection the sliver's three, the block less the bar
/// eight faces, and the bar less the block four. Every fuse used to be
/// refused: under `certified`, `Undecidable` at the side of the top over the
/// far seam corner, a box `2.8e−13` wide the top runs through, and under
/// `tolerant`, `CannotTellTheCellsOfAFaceApart`. `fuse/arrange.rs` now reads
/// that side off which arc of the far cap's circle the box is on, and the side
/// of the top at the near seam corner off the near cap's circle, which is
/// clear of it.
///
/// The tolerant union with the block first was refused `CannotMeshAFace`, a
/// loop naming its fourth corner twice, though the union the other way round
/// answered, until queue row Q2238. `mesh.rs` puts one point inside the arc
/// where the bar's pipe meets the top, at the arc's tip, and with the block
/// first that point is stored at exactly the near seam corner's numbers
/// `(3, 2, 4)`; with the bar first it is stored one rounding step off in `y`.
/// The pipe's loop is now cut in two at the place it names twice. The answers
/// are not measured: the sliver is too thin to mesh to the deflection
/// `volume_of` asks for.
#[test]
fn a_bar_turned_twice_with_its_seam_underneath_dips_through_a_blocks_top() {
    every_fuse_of_the_block_and_a_bar_dipping_through_it(
        |model| a_bar_with_its_seam_underneath(model, [3.0, 2.0, 5.0]),
        |rule, bar_first| {
            WhatTolerantGives::Faces(what_a_bar_dipping_through_the_top_gives(rule, bar_first))
        },
    );
}

/// The bar of the test above sunk to `(3, 2, 1)`, along the block's floor,
/// where its far end drops through the floor in the same sliver (queue row
/// Q2187). Every certified fuse is refused `DynamicRangeTooWide` as there
/// (queue row Q2237): the union and the bar less the block hold the sliver's
/// far cap, and the intersection and the block less the bar the bar's pipe
/// less the sliver, whose arc along the floor turns about `5e−16` from the
/// near seam corner `(3, 2, 0)`. Under `tolerant` each answers with one record
/// of that spread and validates under `tolerant` alone: the union is one shell
/// of the block's six faces and the sliver's pipe and far cap, the
/// intersection the bar less the sliver in four faces, the block less the bar
/// a hollow open through the floor in nine, and the bar less the block the
/// sliver's three. Every one used to be refused as the bar along the top was,
/// round `(1, 2, 0)`.
#[test]
fn a_bar_turned_twice_with_its_seam_underneath_dips_through_a_blocks_floor() {
    every_fuse_of_the_block_and_a_bar_dipping_through_it(
        |model| a_bar_with_its_seam_underneath(model, [3.0, 2.0, 1.0]),
        |rule, bar_first| {
            WhatTolerantGives::Faces(what_a_bar_dipping_through_the_floor_gives(rule, bar_first))
        },
    );
}

/// A bar of radius one, two high, standing on the block `[0, 4]³`'s top flush
/// with its edge `x = 4`, from `(3, 2, 4)`, and the same bar flush with the
/// edge `x = 0`, from `(1, 2, 4)` (queue row Q2257). Either way the bar's foot
/// touches the top's outline at one point that is no corner of the outline:
/// `(4, 2, 4)`, the bar's seam corner, or `(0, 2, 4)`, across the foot from
/// it.
///
/// **Every rule answers, every decision proved, and every face of the union
/// is one loop.** The union is one solid of eight faces, `64 + 2π`, the
/// intersection no solid, and each difference the first solid whole. The fuse
/// cuts the top's edge at the touch and writes the foot's circle into the
/// top's outline there, one loop through the touch twice, with the circle's
/// corner moved to the touch. So no face of the answer holds two loops, one's
/// corner on the other's side, which `the_narrowest_a_face_is` reads as a neck
/// of no width and refuses `DynamicRangeTooWide`.
/// [`a_top_whose_hole_touches_its_outline_at_a_point_is_refused_as_too_thin`]
/// pins the one way such a face is made.
#[test]
fn a_bar_standing_on_a_blocks_top_flush_with_its_edge_answers_every_rule() {
    for at in [[3.0, 2.0, 4.0], [1.0, 2.0, 4.0]] {
        let the_bar = |model: &mut Model| a_bar(model, 1.0, 2.0, at);
        every_fuse_of_the_block_and_a_bar(the_bar, a_bar_standing_on_the_block);
        for bar_first in [false, true] {
            let (model, union) = fuse_the_block_and_a_bar(
                the_bar,
                Rule::Union,
                bar_first,
                true,
                &mut ProvenanceLog::new(),
            );
            let union = union.expect("the union of a bar standing on a block answers");
            let loops_of_each_face: Vec<usize> = union
                .solids()
                .iter()
                .flat_map(|&solid| {
                    model
                        .solid(solid)
                        .expect("this model made the key")
                        .shells()
                        .flat_map(|shell| shell.faces())
                        .map(|face| face.wire_count())
                })
                .collect();
            assert!(
                loops_of_each_face.iter().all(|&loops| loops == 1),
                "the bar from {at:?}, bar first {bar_first}: the union's faces have \
                 {loops_of_each_face:?} loops, and the fuse writes each as one"
            );
        }
    }
}

/// What every rule gives for the block `[0, 4]³` and a bar of radius one, two
/// high, standing on its top: the union is one solid, the block's six faces
/// and the bar's pipe and top, the intersection no solid, and each difference
/// the first solid whole.
fn a_bar_standing_on_the_block(rule: Rule, bar_first: bool) -> Vec<ShellsAndVolume> {
    match (rule, bar_first) {
        (Rule::Union, _) => vec![(vec![8], 64.0 + 2.0 * PI)],
        (Rule::Intersection, _) => vec![],
        (Rule::Difference, false) => vec![(vec![6], 64.0)],
        (Rule::Difference, true) => vec![(vec![3], 2.0 * PI)],
        _ => unreachable!("only the three rules are asked"),
    }
}

/// The block `[0, 4]³` with a prism two high standing on its top, written
/// face by face with [`SolidBuilder`] (queue row Q2257). The prism's foot is
/// the triangle `(4, 2)`, `(3, 3)`, `(3, 1)`, whose corner `(4, 2, 4)` sits on
/// the middle of the top's edge `x = 4`, so the top round the foot touches its
/// own outline at one point, as under the bar of
/// [`a_bar_standing_on_a_blocks_top_flush_with_its_edge_answers_every_rule`].
///
/// **Written as the fuse writes it, the top is one loop through the touch
/// twice, and the solid builds. Written as an outline and a hole, it is
/// refused `DynamicRangeTooWide`, with a spread of `6 / (4ε)`.** The hole's
/// corner is no distance from the outline's side, and
/// `how_close_a_corner_comes_to_a_side_of_another_loop` floors that at one
/// rounding of the 4 the two are stored at. Under `tolerant` the solid builds
/// with that one record, every other check passing it, and is then refused the
/// same under `certified`.
///
/// That is the wrong name for it. No feature of this solid is near a rounding:
/// the two loops touch at a point, as two loops sharing a corner do, which is
/// a solid touching itself and an answer (§39.4). Whether such a face is a
/// touch, or a refusal that says what it is, is the row Q2257 leaves. No fuse
/// writes such a face, and [`ProfileBuilder`] refuses such a hole
/// `ProfileCrossesItself`, so only a solid written by hand meets it.
#[test]
fn a_top_whose_hole_touches_its_outline_at_a_point_is_refused_as_too_thin() {
    let refused_for_the_touch = |refusal: Option<&Error>| {
        matches!(
            refusal,
            Some(Error::Invalid(Invalid::DynamicRangeTooWide { ratio }))
                if *ratio == 6.0 / (4.0 * f64::EPSILON)
        )
    };

    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let mut log = ProvenanceLog::new();
    let one_loop = a_block_with_a_prism_on_its_top(
        &mut model,
        TheTop::OneLoopThroughTheTouchTwice,
        &Policy::certified(),
        &mut log,
    );
    assert!(
        one_loop.is_ok() && log.records().is_empty(),
        "the top as one loop: {one_loop:?}, records {:?}",
        log.records()
    );

    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let certainly = a_block_with_a_prism_on_its_top(
        &mut model,
        TheTop::AnOutlineAndAHole,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        refused_for_the_touch(certainly.as_ref().err()),
        "the top as an outline and a hole, under certified: {certainly:?}"
    );

    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let mut log = ProvenanceLog::new();
    let tolerantly = a_block_with_a_prism_on_its_top(
        &mut model,
        TheTop::AnOutlineAndAHole,
        &Policy::tolerant(100.0),
        &mut log,
    );
    let records: Vec<&str> = log
        .records()
        .iter()
        .map(|record| record.predicate())
        .collect();
    assert!(
        tolerantly.is_ok() && records == ["dynamic range check"],
        "the top as an outline and a hole, under tolerant: {tolerantly:?}, records {records:?}"
    );
    let then = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
    assert!(
        refused_for_the_touch(then.as_ref().err()),
        "the tolerant answer validated under certified: {then:?}"
    );

    let as_a_profile = ProfileBuilder::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]])
        .with_hole(vec![[4.0, 2.0], [3.0, 1.0], [3.0, 3.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new());
    assert!(
        matches!(as_a_profile, Err(Error::ProfileCrossesItself)),
        "a square with a hole whose corner is on its side: {as_a_profile:?}"
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
/// `[0, 4]³` and the bar `bar` puts in the model after it: never refused,
/// every decision proved, the model validates, and the answer's solids, fewest
/// faces first, have the faces in each shell and the volume `should` gives for
/// the rule and whether the bar came first.
fn every_fuse_of_the_block_and_a_bar(
    bar: impl Fn(&mut Model) -> SolidKey,
    should: fn(Rule, bool) -> Vec<ShellsAndVolume>,
) {
    let mut wrong: Vec<String> = Vec::new();
    for bar_first in [false, true] {
        for certified in [true, false] {
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                let what = format!("{rule:?}, bar first {bar_first}, certified {certified}");
                let mut log = ProvenanceLog::new();
                let (model, fused) =
                    fuse_the_block_and_a_bar(&bar, rule, bar_first, certified, &mut log);
                let answer = match fused {
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

/// Every rule, both ways round, of the block `[0, 4]³` and the bar `bar` puts
/// in the model after it, which dips through one of the block's faces in a
/// sliver too thin for its coordinates: refused `DynamicRangeTooWide` past the
/// spread the kernel works to under `certified`, and under `tolerant` what
/// `tolerant` says for the rule and whether the bar came first. A tolerant
/// answer must have those faces in each shell, carry one record, of the range
/// check, validate under `tolerant`, and be refused `DynamicRangeTooWide`
/// under `certified`: valid is a property of a model and a policy, and
/// `Model::fuse` promises the first and not the second (§68).
///
/// An answer is not measured: its sliver is too thin to mesh to the
/// deflection `volume_of` asks for.
fn every_fuse_of_the_block_and_a_bar_dipping_through_it(
    bar: impl Fn(&mut Model) -> SolidKey,
    tolerant: fn(Rule, bool) -> WhatTolerantGives,
) {
    let mut wrong: Vec<String> = Vec::new();
    for bar_first in [false, true] {
        for certified in [true, false] {
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                let what = format!("{rule:?}, bar first {bar_first}, certified {certified}");
                let mut log = ProvenanceLog::new();
                let (model, fused) =
                    fuse_the_block_and_a_bar(&bar, rule, bar_first, certified, &mut log);
                if certified {
                    if !matches!(
                        &fused,
                        Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio }))
                            if *ratio > DynamicRange::LARGEST_SUPPORTED_RATIO
                    ) {
                        let got = match &fused {
                            Ok(_) => "answered".to_owned(),
                            Err(why) => format!("refused {why:?}"),
                        };
                        wrong.push(format!(
                            "{what}: {got}, and it should be refused DynamicRangeTooWide"
                        ));
                    }
                    continue;
                }
                let expected = match tolerant(rule, bar_first) {
                    WhatTolerantGives::Faces(expected) => expected,
                    WhatTolerantGives::TheSliverAloneIsRefused => {
                        if !matches!(
                            &fused,
                            Err(Error::CannotTellWhichWayASolidsFacesPoint { enclosed, .. })
                                if *enclosed == 0.0
                        ) {
                            let got = match &fused {
                                Ok(_) => "answered".to_owned(),
                                Err(why) => format!("refused {why:?}"),
                            };
                            wrong.push(format!(
                                "{what}: {got}, and it should be refused \
                                 CannotTellWhichWayASolidsFacesPoint, enclosing exactly 0"
                            ));
                        }
                        continue;
                    }
                };
                let answer = match &fused {
                    Ok(answer) => answer,
                    Err(why) => {
                        wrong.push(format!("{what}: refused {why:?}"));
                        continue;
                    }
                };
                let mut found: Vec<Vec<usize>> = answer
                    .solids()
                    .iter()
                    .map(|&solid| faces_of_each_shell(&model, solid))
                    .collect();
                found.sort();
                if found != expected {
                    wrong.push(format!("{what}: {found:?}, and it should be {expected:?}"));
                }
                let only_the_range = log.records().len() == 1
                    && log.records()[0].predicate() == "dynamic range check";
                if !only_the_range {
                    wrong.push(format!(
                        "{what}: the records are not the range check's one: {:?}",
                        log.records()
                    ));
                }
                let tolerantly =
                    model.validate(&Policy::tolerant(100.0), &mut ProvenanceLog::new());
                let certainly = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
                let valid_only_tolerantly = tolerantly.is_ok()
                    && matches!(
                        &certainly,
                        Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio }))
                            if *ratio > DynamicRange::LARGEST_SUPPORTED_RATIO
                    );
                if !valid_only_tolerantly {
                    wrong.push(format!(
                        "{what}: the model validates {tolerantly:?} under tolerant and \
                         {certainly:?} under certified"
                    ));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The block `[0, 4]³` and the bar `bar` puts in the model after it, fused by
/// `rule` with the bar first or second, under `certified` or under `tolerant`,
/// and the model the answer is in.
fn fuse_the_block_and_a_bar(
    bar: impl FnOnce(&mut Model) -> SolidKey,
    rule: Rule,
    bar_first: bool,
    certified: bool,
    log: &mut ProvenanceLog,
) -> (Model, rcad_topo::Result<Fused>) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let block = a_block(&mut model, [4.0, 4.0, 4.0]);
    let bar = bar(&mut model);
    let [first, second] = if bar_first {
        [bar, block]
    } else {
        [block, bar]
    };
    let policy = if certified {
        Policy::certified()
    } else {
        Policy::tolerant(100.0)
    };
    let fused = model.fuse(&[first], &[second], rule, &policy, log);
    (model, fused)
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

/// A bar of radius one, two long, turned by [`Isometry::rotation`] about `x`
/// by `π`, then about `y` by `π/2`, and then shifted to `at`. Its axis runs
/// from `at` along `(−1, −1.2e−16, −2⁻⁵²)`, and its seam along its underside,
/// a radius below the axis.
fn a_bar_with_its_seam_underneath(model: &mut Model, [x, y, z]: [f64; 3]) -> SolidKey {
    let flip = Isometry::rotation(&Axis::x(), PI).expect("a finite angle");
    let onto_x = Isometry::rotation(&Axis::y(), PI / 2.0).expect("a finite angle");
    let shift = Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift");
    model
        .add_round_bar(1.0, 2.0, &shift.compose(&onto_x).compose(&flip))
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

/// How the block's top round the foot of the prism standing on it is written.
#[derive(Debug, Clone, Copy)]
enum TheTop {
    /// As the fuse writes it: one loop, round the outline to the touch, round
    /// the foot, and back through the touch to the rest of the outline.
    OneLoopThroughTheTouchTwice,
    /// The outline, and the foot as a hole whose corner is on the outline's
    /// side.
    AnOutlineAndAHole,
}

/// The block `[0, 4]³` and the prism of
/// [`a_top_whose_hole_touches_its_outline_at_a_point_is_refused_as_too_thin`]
/// standing on its top, from `z = 4` to `z = 6`, one solid of ten flat faces
/// built under `policy`, with the top written as `top` says.
fn a_block_with_a_prism_on_its_top(
    model: &mut Model,
    top: TheTop,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> rcad_topo::Result<SolidKey> {
    let [p, q, r] = [[4.0, 2.0, 4.0], [3.0, 3.0, 4.0], [3.0, 1.0, 4.0]];
    let [high_p, high_q, high_r] = [p, q, r].map(|[x, y, _]| [x, y, 6.0]);
    let mut faces: Vec<AFlatFace> = vec![
        (
            [0.0, 0.0, -1.0],
            vec![vec![
                [0.0, 0.0, 0.0],
                [0.0, 4.0, 0.0],
                [4.0, 4.0, 0.0],
                [4.0, 0.0, 0.0],
            ]],
        ),
        (
            [-1.0, 0.0, 0.0],
            vec![vec![
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 4.0],
                [0.0, 4.0, 4.0],
                [0.0, 4.0, 0.0],
            ]],
        ),
        (
            [0.0, -1.0, 0.0],
            vec![vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [4.0, 0.0, 4.0],
                [0.0, 0.0, 4.0],
            ]],
        ),
        (
            [0.0, 1.0, 0.0],
            vec![vec![
                [0.0, 4.0, 0.0],
                [0.0, 4.0, 4.0],
                [4.0, 4.0, 4.0],
                [4.0, 4.0, 0.0],
            ]],
        ),
        ([0.0, 0.0, 1.0], vec![vec![high_p, high_q, high_r]]),
        ([1.0, 1.0, 0.0], vec![vec![p, q, high_q, high_p]]),
        ([-1.0, 0.0, 0.0], vec![vec![q, r, high_r, high_q]]),
        ([1.0, -1.0, 0.0], vec![vec![r, p, high_p, high_r]]),
    ];
    match top {
        TheTop::OneLoopThroughTheTouchTwice => {
            faces.push((
                [1.0, 0.0, 0.0],
                vec![vec![
                    [4.0, 0.0, 0.0],
                    [4.0, 4.0, 0.0],
                    [4.0, 4.0, 4.0],
                    p,
                    [4.0, 0.0, 4.0],
                ]],
            ));
            faces.push((
                [0.0, 0.0, 1.0],
                vec![vec![
                    [0.0, 0.0, 4.0],
                    [4.0, 0.0, 4.0],
                    p,
                    r,
                    q,
                    p,
                    [4.0, 4.0, 4.0],
                    [0.0, 4.0, 4.0],
                ]],
            ));
        }
        TheTop::AnOutlineAndAHole => {
            faces.push((
                [1.0, 0.0, 0.0],
                vec![vec![
                    [4.0, 0.0, 0.0],
                    [4.0, 4.0, 0.0],
                    [4.0, 4.0, 4.0],
                    [4.0, 0.0, 4.0],
                ]],
            ));
            faces.push((
                [0.0, 0.0, 1.0],
                vec![
                    vec![
                        [0.0, 0.0, 4.0],
                        [4.0, 0.0, 4.0],
                        [4.0, 4.0, 4.0],
                        [0.0, 4.0, 4.0],
                    ],
                    vec![p, r, q],
                ],
            ));
        }
    }
    a_solid_of_flat_faces(model, &faces, policy, log)
}

/// A flat face: its outward normal, and its loops, the outline first, each
/// as its corners in order, turning anticlockwise seen from outside and the
/// holes clockwise.
type AFlatFace = ([f64; 3], Vec<Vec<[f64; 3]>>);

/// One solid of one shell of `faces`, written with [`SolidBuilder`] and built
/// under `policy`. Each point is one corner however many loops name it, and
/// each pair of corners one straight edge, walked forward by the loop that
/// names it first and backward by the other.
fn a_solid_of_flat_faces(
    model: &mut Model,
    faces: &[AFlatFace],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> rcad_topo::Result<SolidKey> {
    let bits = |at: [f64; 3]| at.map(f64::to_bits);
    let mut builder = SolidBuilder::new();
    let mut corners = BTreeMap::new();
    let mut edges = BTreeMap::new();
    let mut shell = Vec::new();
    for (normal, loops) in faces {
        let mut wires = Vec::new();
        for round in loops {
            for &at in round {
                if let Entry::Vacant(slot) = corners.entry(bits(at)) {
                    slot.insert(builder.add_vertex(Point::new(at[0], at[1], at[2]))?);
                }
            }
            let mut steps = Vec::new();
            for (position, &from) in round.iter().enumerate() {
                let to = round[(position + 1) % round.len()];
                let (edge, way) = if let Some(&edge) = edges.get(&(bits(to), bits(from))) {
                    (edge, Orientation::Reversed)
                } else {
                    let edge = builder.add_edge(
                        corners[&bits(from)],
                        corners[&bits(to)],
                        EdgeRun::Straight,
                    )?;
                    edges.insert((bits(from), bits(to)), edge);
                    (edge, Orientation::Forward)
                };
                steps.push(DraftStep::Along(builder.add_coedge(edge, way)?));
            }
            wires.push(builder.add_wire(&steps)?);
        }
        let [x, y, z] = loops[0][0];
        let plane = Plane::from_point_and_normal(
            Point::new(x, y, z),
            Vector::new(normal[0], normal[1], normal[2]),
        )
        .expect("each face's normal is a direction");
        shell.push((
            builder.add_face(Surface::Plane(plane), &wires)?,
            Orientation::Forward,
        ));
    }
    builder.add_shell(&shell)?;
    builder.build(model, policy, log)
}
