//! Does cutting a flat face along the *whole* double cone ever give a wrong
//! answer, or only a refusal? `docs/decisions.md` §199.3 point 1 left that open.
//!
//! A cone face as rcad builds it lies on one nappe, but the fuse cuts a plane
//! face wherever the plane meets the cone's surface — including the other
//! nappe, and parts of this nappe the face does not cover
//! (`a_plane_across_a_cone.rs` shows one such cut, far above a frustum). Each
//! family below puts that stray cut somewhere it could matter, runs all three
//! rules (the difference both ways round), and holds every answer against the
//! shape itself:
//!
//! - the model holding the answer passes [`Model::validate`];
//! - it has the number of solids the shapes call for;
//! - its measured volume is within `measure`'s own bound (plus a small slack
//!   for the column sum below) of the volume worked out from the shapes: the
//!   frustum's and the box's are closed forms, and what they share is summed
//!   column by column, since along a vertical line each is one interval;
//! - on a grid of points kept clear of every boundary, the answer holds exactly
//!   the points the rule says it should.
//!
//! **Every family pins how many fuses answer and how each of the others
//! refuses.** A family where everything refuses measures nothing about wrong
//! answers, and that count is what says so.
//!
//! A pointed cone whose apex sits inside a small cube stood on its corner used
//! to answer wrong — both solids back untouched, as if they never met — and was
//! left out of the file for it. Since the cone's box reaches its apex it
//! stopped answering wrong; since `select.rs` counts the bulge between a cone's
//! arc and its chord half of it answers right; and since §208.4 walks a ring of
//! arcs round an apex the other half does too.
//! [`a_corner_up_cube_centred_on_a_pointed_cones_apex_answers_every_rule_right`]
//! pins that. The frustum's cube kept a refusal of its own until §212 let a ring
//! bound a face that also has a hole, and
//! [`a_corner_up_cube_of_side_four_round_a_frustums_apex_answers_right`] and the
//! three beside it pin that it is gone.

use std::collections::BTreeMap;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Invalid, Model, OnAFace, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_pointed_cone};

/// A block steeper than the cone that also crosses the frustum.
///
/// Its low-`x` face is the plane `x = lean · (z − crossing)`, leaning at a
/// quarter or three eighths against the cone's half, so it cuts both nappes:
/// through the upper nappe above, and through the lower nappe, where the
/// frustum has no face, below.
///
/// **Two of the six planes miss the frustum.** The upper branch's lowest point
/// is where `z/2 = lean · (z − crossing)`, and at a lean of three eighths that
/// is `z = 9` for a crossing of `−3` and `z = 18` for `−6`, both above the top
/// cap at `8`: the block misses the frustum at the top rim by `0.125` in `x`
/// for `−3` and by `1.25` for `−6`,
/// and every point of the frustum is on the far side of that plane. So those
/// eight cases are [`APART`]. It was written [`OVERLAPPING`] for all of them
/// and nothing read it while every one refused; §251 is when the first of them
/// answered, with the two solids a union of shapes that never meet leaves.
///
/// **All ninety-six refused until §251**, eighty-four of them for want of a
/// point inside a piece. Forty answer now, each held to the four checks at the
/// top of this file.
///
/// **Forty-four more refused on a wrong piece and not a missing point**
/// (§282): the part of the frustum's face inside the block and the part outside
/// it glued into one by `arrange.rs`'s sign rule, which no point can be in.
/// Since the rule places the bottom rim by which piece goes round the axis
/// with it, twenty-eight of them answer right. Sixteen more found no point in
/// a piece that is one cell; each such piece is now placed by a side it shares
/// with a piece of the other solid's face (`docs/decisions.md` §365.3), and
/// all sixteen answer right. The twelve that could not order along a crossing
/// were the same twelve as before.
///
/// **All ninety-six answer right since Q901**, which orders two meets of three
/// planes that share two by the planes when their boxes cannot say: the twelve
/// at three eighths, turned `0.5`, that could not order along a crossing answer
/// now, each held to the four checks at the top of this file.
///
/// **The family is two tests, one per lean**, so that neither sets the slow
/// tier's wall (`docs/decisions.md` §397.4). Their tallies add up to the counts
/// above: the sixteen are all at a quarter, crossing `−6`.
#[test]
fn a_steeper_plane_at_a_lean_of_a_quarter_across_a_frustum_answers_right_or_refuses() {
    assert_eq!(
        tally(&steeper_planes_across_the_frustum(0.25)),
        Tally {
            right: 48,
            refused: [].into()
        }
    );
}

#[test]
fn a_steeper_plane_at_a_lean_of_three_eighths_across_a_frustum_answers_right_or_refuses() {
    assert_eq!(
        tally(&steeper_planes_across_the_frustum(0.375)),
        Tally {
            right: 48,
            refused: [].into()
        }
    );
}

fn steeper_planes_across_the_frustum(lean: f64) -> Vec<Case> {
    let mut cases = Vec::new();
    for crossing in [-1.0, -3.0, -6.0] {
        let lowest_on_the_upper_branch = -lean * crossing / (THE_FRUSTUM.slope - lean);
        let solids = if lowest_on_the_upper_branch < THE_FRUSTUM.far {
            OVERLAPPING
        } else {
            APART
        };
        for turn in [0.0, 0.5, 2.0, 3.0] {
            cases.push(Case {
                cone: THE_FRUSTUM,
                block: a_leaning_block(lean, crossing, turn),
                solids,
            });
        }
    }
    cases
}

/// The same blocks turned half round, so the frustum's apex is inside the
/// block and so is the stretch of the lower nappe round it.
///
/// **All thirty-two refused until §251**, twenty-four of them for want of a
/// point inside a piece. Twenty-two answered then, and two of the twenty-four
/// gave up one step on, meshing the answer: the intersection at lean three
/// eighths, crossing `−1/4`, turned 3 and 3.5, whose steep face and frustum face
/// share both arcs and at the coarsest mesh both draw the side between the arcs'
/// middles (§285.4). That mesh retries a fold at half the room now, and both
/// answer right.
///
/// **All thirty-two answer right since Q1171.** The eight that were left
/// refused `CannotNameWhichPieceOfACrossingThisIs` were the block whose steep
/// face runs through the apex, lean a quarter and crossing `0`, turned 3 and
/// 3.5. That face cuts the cone in two rulings along `(¼, ±√3/4, 1)`, which no
/// three `f64`s write, so `intersect` could not name them whole; it names the
/// four halves the apex cuts them into now (§294.11), and each of the eight is
/// held to the four checks at the top of this file.
#[test]
fn a_frustum_whose_apex_is_inside_the_block_answers_right_or_refuses() {
    let mut cases = Vec::new();
    for (lean, crossing) in [(0.25, 1.0), (0.375, 0.5), (0.25, 0.0), (0.375, -0.25)] {
        for turn in [3.0, 3.5] {
            cases.push(Case {
                cone: THE_FRUSTUM,
                block: a_leaning_block(lean, crossing, turn),
                solids: OVERLAPPING,
            });
        }
    }
    assert_eq!(
        tally(&cases),
        Tally {
            right: 32,
            refused: [].into()
        }
    );
}

/// A cone that ends in a point, with a steeper face through the apex or an
/// eighth or a half either side of it.
///
/// **All ninety-six refused until §251**, eighty of them for want of a point
/// inside a piece. Fifty-eight answered then; of the eighty, eight still find
/// no point and fourteen gave up one step on, meshing the answer. Two of those
/// fourteen answer since §252 stopped a closed edge's start being a corner,
/// so sixty answered and twelve gave up meshing (§252.9).
///
/// **The eight that found no point were a triangle with the apex for a
/// corner** (§282). A face through the apex cuts the cone in two rulings that
/// meet the cap's chord, and the only candidate aimed across that chord ran
/// through the apex, where the two crossings are one root and rounding put the
/// discriminant below zero. Read as that one root, four of the eight answer and
/// four give up meshing: sixty-four right, sixteen meshing.
///
/// **The sixteen were caps cut into quarters with two cuts from one corner**
/// (§285.2). With the cuts from four different corners, twelve answer right
/// (§354). The other four are the intersection and the block less the cone of
/// the two blocks whose steep face is meant to pass through the apex, at turn
/// one, where the crossing passes a rounding above it (§285.3): they hold every
/// grid point they are asked about, and do not measure at `0.002`.
///
/// **The two of those four at lean three eighths answer right since Q441**,
/// held to the same four checks as every right answer here: the volume
/// measured at `0.002` against the box's closed form and the column sum of
/// what it shares with the cone, and every grid point. Their cap's walk starts
/// at the end of the kept rim where the crossing comes up the cone along a
/// ruling, and the first quarter cut left from that corner and ran down that
/// ruling to the apex, along the rim's own last side; the triangulator refused
/// it `ASideCutsOffOneCornerOfACurvedPatch` at every rung. A cut leaves only
/// from a corner whose meridian meets the rim nowhere else now, and the first
/// leaves from the next corner along the rim, `0.027` round from the ruling at
/// `0.002`. The two at lean one quarter still do not measure at `0.002`, and
/// `the_eighteen_held_to_the_familys_four_checks` says why.
///
/// **The sixteen that could not name a piece answer right since Q1171**, each
/// held to the same four checks. They are the faces through the apex, turned
/// `0` or `3`, eight at each lean. Such a face cuts the cone in two rulings
/// along `(¼, ±√3/4, 1)` at a quarter and `(⅜, ±√7/8, 1)` at three eighths,
/// which no three `f64`s write, so `intersect` could not name them whole; it
/// names the four halves the apex cuts them into now (§294.11). The face runs
/// down past the apex, so all four halves cross it, two of them along the
/// other nappe.
///
/// **The family is two tests, one per lean**, so that neither sets the slow
/// tier's wall (`docs/decisions.md` §397.4). Their tallies add up to the counts
/// above: the two that do not measure are at a quarter.
#[test]
fn a_pointed_cone_with_its_apex_near_a_face_at_a_lean_of_a_quarter_answers_right_or_refuses() {
    assert_eq!(
        tally(&steeper_faces_near_the_pointed_cones_apex(0.25)),
        Tally {
            right: 46,
            refused: [("the answer does not measure: CannotMeshAFace", 2)].into()
        }
    );
}

#[test]
fn a_pointed_cone_with_its_apex_near_a_face_at_a_lean_of_three_eighths_answers_right_or_refuses() {
    assert_eq!(
        tally(&steeper_faces_near_the_pointed_cones_apex(0.375)),
        Tally {
            right: 48,
            refused: [].into()
        }
    );
}

fn steeper_faces_near_the_pointed_cones_apex(lean: f64) -> Vec<Case> {
    let mut cases = Vec::new();
    for crossing in [0.0, 0.125, -0.125, 0.5] {
        for turn in [0.0, 1.0, 3.0] {
            cases.push(Case {
                cone: THE_POINTED_CONE,
                block: a_leaning_block(lean, crossing, turn),
                solids: OVERLAPPING,
            });
        }
    }
    cases
}

/// A block that misses the frustum, below it, whose faces still cross the cone
/// where the frustum has none: its top cuts the upper nappe under the frustum's
/// bottom cap, and its bottom the lower nappe.
///
/// **This is the family where the stray cut is the only cut**, and every fuse
/// that answers hands both solids back as they were.
///
/// **Two blocks of it are not below the frustum at all.** Leaned at `2`, the one
/// reaching up to `1.5` and the one reaching up to `1` each have an edge that
/// rises into the frustum: the column sum of [`shared_volume`] puts `2.917` and
/// `2.847` of the two in common. The first, once it answered, was united with
/// the frustum into one solid. Both were written apart with the rest and
/// nothing read that while they refused, so which pair shares something is
/// worked out from the shapes now, as the steeper family works it out.
///
/// **Twenty refused on a piece glued out of two cells** until §36.3's sign rule
/// stopped doing that (§282.7 named them). Twelve of them answer right now and
/// eight still find no point in a piece that is one cell. Four of the twelve
/// refused `ResultIsNotManifold` further on until §340.10.
///
/// **Two of the eight did not measure until Q423**: the union and the
/// cone-less-block of the block reaching up to `1.5`, turned `0`. They held
/// every grid point they decided, 7327 and 16702, and `measure` refused
/// `DeflectionBelowWhatAMeshCanReach` with a floor of `2069086.58` — before any
/// point was walked, so the mesher never tried the parabola the block's top
/// face cuts along a ruling. The floor charges a cone face for how far up the
/// axis its edges reach, and `mesh.rs` read that parabola's reach off the
/// plane's lean in `f64`, where `0.447213595499958 − 0.5 · 0.8944271909999159`
/// came to `5.55e-17` instead of zero, and a distance divided by it to
/// `5.7e16`. The lean is asked exactly now, and a piece of a curve that runs on
/// for ever is charged its two ends; both measure, `579.01` and `129.01`,
/// against `579.03` and `129.03` from the shapes.
///
/// **Four answer right since §340.10, every rule of one block**: leaned at `2`,
/// from `−3` up to `1`, turned `0`. Its top face is parallel to a ruling of the
/// cone, so it cuts the upper nappe in a parabola, and the ridge where the top
/// face meets the leaning face passes through the frustum at
/// `(1.3416, ±0.9266, 3.2610)`. All four refused `ResultIsNotManifold` because
/// **the top face was not cut there**: `arrange.rs`'s
/// `work_out_how_each_crossing_runs` dropped the parabola as a cut that misses
/// the face, since every node on it was outside `bake.rs`'s box round the face.
/// The two nodes on the ridge are the meets of the top face's plane, the
/// leaning face's plane and the cone, with `x` from `1.341640786499873` to
/// `1.3416407864998736`, and the box was the box round the face's *stored*
/// corners, whose `x` rounded to `1.341640786499874`, a rounding further out.
/// That box is grown by what writing a corner down costs now. All four measure:
/// the union decides 6986 grid points, the intersection 17954, the cone less the
/// block 16361 and the block less the cone 8579. Turned `1`, the same block
/// refused `order along a crossing` instead, until §406.
///
/// **Twelve fewer stop at `order along a crossing` since Q901**, which orders
/// two meets of three planes that share two by the planes when their boxes
/// cannot say. Right answers went from 68 to 72, each held to the four checks
/// at the top of this file, and refusals for want of a point inside a piece
/// from 8 to 16. The sixteen are the blocks from `−3` up to `1` and up to
/// `1.5`, leaned at a quarter, both turns and every rule, and the eight left at
/// the order were the same two blocks leaned at `2` and turned `1`.
///
/// **None of the eight stops at the order since §406.** Turned `1`, the
/// block's top and bottom and their side edges are a rounding off parallel to
/// a ruling, so the top and the bottom cut the cone in ellipses `4.5e16` long
/// and the side edges meet it again `2.26e16` and `4.48e16` out. Two meets by
/// the frustum are put in order round the top's ellipse from its start
/// `2.2e16` out, two far meets on it are left out because nothing drawn
/// crosses there, and the two far meets of the bottom's side edges are boxed
/// to an ulp and ordered round the bottom's ellipse, and how the edge leaves
/// the ellipse there is read off the root. The block up to `1.5` answered right
/// for the intersection and for the block less the cone, held to the four
/// checks and deciding 30172 grid points each, and the other six refused inside
/// the fuse, meshing what it built.
///
/// **All eight answer right since Q1074**, each held to the four checks. The six
/// shared one cause: `rcad_geom::walk` placed the points of an arc of the top's
/// ellipse by the frustum from the ellipse's centre `2.2e16` out, so the arc's
/// one middle point rounded to `(−3, 0, 8)`, `(0, 0, 4)` or `(−2, 0, 8)`, one
/// or two off the cone. Such an arc is walked round the cone now, along its
/// rulings, and the mesher charges its floor by the arc's ends and not by the
/// ellipse's far end (§406.5).
///
/// **The sixteen that found no point inside a piece answer right since
/// Q929**, each held to the four checks. The piece is one of the cone's, and
/// every step of one radius from its corners' middle missed it; the fuse
/// offers the same steps at a half, a quarter, an eighth and a sixteenth of
/// that after every other point, and one of them is inside it
/// (`a_frustums_wall_cut_below_a_blocks_floor.rs`).
#[test]
fn a_block_below_a_frustum_is_cut_where_the_frustum_has_no_face_and_every_answer_is_right() {
    let mut cases = Vec::new();
    for (low, high) in [(-3.0, 1.0), (-6.0, -1.0), (-3.0, 1.5), (-8.0, 1.875)] {
        for lean in [0.0f64, 0.25, 2.0] {
            for turn in [0.0, 1.0] {
                let placed = about_z(turn)
                    .compose(&moved([0.0, 0.0, low]))
                    .compose(&about_y(lean.atan()))
                    .compose(&moved([-5.0, -5.0, 0.0]));
                let mut case = Case {
                    cone: THE_FRUSTUM,
                    block: BoxShape {
                        size: [10.0, 10.0, high - low],
                        placed,
                    },
                    solids: APART,
                };
                if shared_volume(&case) > 0.0 {
                    case.solids = OVERLAPPING;
                }
                cases.push(case);
            }
        }
    }
    assert_eq!(
        tally(&cases),
        Tally {
            right: 96,
            refused: [].into()
        }
    );
}

/// A cube stood on its corner round the frustum's apex, or just under the
/// frustum. Every plane of the cube leans less steeply than the cone, so each
/// cuts one nappe in an ellipse — and the three lower planes cut the lower
/// nappe, where the frustum has no face.
///
/// Half of these refused `AWireWillNotLieFlatOnItsFace` until §212. The
/// intersection's cone face is the frustum's **band**, ringed by arcs round the
/// apex and holed by the frustum's own near cap, and §208.4 walked a ring only
/// when it was its face's one wire. Two wires that each go right round bound the
/// band between them and nothing else, so the walk takes them now and all 48 are
/// right. The name says so rather than keeping `_or_refuses` over a body that
/// does not, exactly as
/// [`a_corner_up_cube_of_side_four_round_a_pointed_cones_apex_answers_right`]
/// did.
///
/// **The family is four tests, one per side and turn**, so that none sets the
/// slow tier's wall (`docs/decisions.md` §397.4). Each holds the three middles,
/// twelve right, and the four add up to the 48.
#[test]
fn a_corner_up_cube_of_side_four_round_a_frustums_apex_answers_right() {
    assert_eq!(
        tally(&corner_up_cubes(
            THE_FRUSTUM,
            &[(4.0, 0.0), (4.0, 1.0), (4.0, 2.5)],
            0.0
        )),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_corner_up_cube_of_side_four_turned_about_the_axis_round_a_frustums_apex_answers_right() {
    assert_eq!(
        tally(&corner_up_cubes(
            THE_FRUSTUM,
            &[(4.0, 0.0), (4.0, 1.0), (4.0, 2.5)],
            0.7
        )),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_corner_up_cube_of_side_six_round_a_frustums_apex_answers_right() {
    assert_eq!(
        tally(&corner_up_cubes(
            THE_FRUSTUM,
            &[(6.0, 0.0), (6.0, 1.0), (6.0, 2.5)],
            0.0
        )),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_corner_up_cube_of_side_six_turned_about_the_axis_round_a_frustums_apex_answers_right() {
    assert_eq!(
        tally(&corner_up_cubes(
            THE_FRUSTUM,
            &[(6.0, 0.0), (6.0, 1.0), (6.0, 2.5)],
            0.7
        )),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

/// The same cubes round a pointed cone, less the side-4 cube centred on the
/// apex, which has a test of its own below.
///
/// All 40 refused until the pointed cone's face could say which side of its cap
/// it lies on (`docs/decisions.md` §200.4's first item): its one corner is on
/// the cap, so no corner said, and every point asked of that face refused. That
/// left 20 right and 20 refused `AWireWillNotLieFlatOnItsFace` — the half whose
/// cone face runs right round the apex, a ring of arcs with no seam that the
/// mesher could not walk. §208.4 taught it to, and all 40 are right now. The
/// name says so rather than keeping `_or_refuses` over a body that does not.
///
/// **The family is four tests, one per side and turn**, as the frustum's is:
/// eight right for each side-4 test and twelve for each side-6, 40 in all.
#[test]
fn a_corner_up_cube_of_side_four_round_a_pointed_cones_apex_answers_right() {
    assert_eq!(
        tally(&corner_up_cubes(
            THE_POINTED_CONE,
            &[(4.0, 1.0), (4.0, 2.5)],
            0.0
        )),
        Tally {
            right: 8,
            refused: [].into()
        }
    );
}

#[test]
fn a_corner_up_cube_of_side_four_turned_about_the_axis_round_a_pointed_cones_apex_answers_right() {
    assert_eq!(
        tally(&corner_up_cubes(
            THE_POINTED_CONE,
            &[(4.0, 1.0), (4.0, 2.5)],
            0.7
        )),
        Tally {
            right: 8,
            refused: [].into()
        }
    );
}

#[test]
fn a_corner_up_cube_of_side_six_round_a_pointed_cones_apex_answers_right() {
    assert_eq!(
        tally(&corner_up_cubes(
            THE_POINTED_CONE,
            &[(6.0, 0.0), (6.0, 1.0), (6.0, 2.5)],
            0.0
        )),
        Tally {
            right: 12,
            refused: [].into()
        }
    );
}

#[test]
fn a_corner_up_cube_of_side_six_turned_about_the_axis_round_a_pointed_cones_apex_answers_right() {
    assert_eq!(
        tally(&corner_up_cubes(
            THE_POINTED_CONE,
            &[(6.0, 0.0), (6.0, 1.0), (6.0, 2.5)],
            0.7
        )),
        Tally {
            right: 12,
            refused: [].into()
        }
    );
}

/// A corner-up cube of each `(side, middle)` round `cone`, all turned `turn`
/// about the axis.
fn corner_up_cubes(cone: ConeShape, cubes: &[(f64, f64)], turn: f64) -> Vec<Case> {
    cubes
        .iter()
        .map(|&(side, middle)| Case {
            cone,
            block: a_corner_up_cube(side, middle, turn),
            solids: OVERLAPPING,
        })
        .collect()
}

/// The side-4 cube stood on its corner and centred on the pointed cone's apex,
/// which `docs/decisions.md` §200.3 found answering wrong: every rule said `Ok`
/// and handed both solids back untouched, because the cone's box stopped at
/// its cap and the fuse looked at no pair of faces (§202.3).
///
/// With the box reaching the apex every fuse refused by name instead, and §204.2
/// called that a **gap**: the two solids share volume, so every rule has an
/// answer to give. The gap is closed now, in two halves. The first was the cone's own
/// big cap, cut by a cube plane into pieces bounded by an arc of the rim and the
/// chord across it — two corners, no polygon to wind, and no point inside the
/// piece at all until `select.rs` counted the bulge between the arc and its
/// chord for a cone as it already did for a ball.
///
/// **The other half was a different gap, in the mesher, and §208.4 closed it.**
/// The intersection's cone face runs right round the apex — the cube holds a
/// neighbourhood of it — so it is a ring of arcs round a cone with no seam, and
/// `mesh.rs` could not walk it (`AWireWillNotLieFlatOnItsFace`, on the wire at
/// `z ≈ 2.559`). It walks one now, so all eight are right and the name no longer
/// says half. §208.4 walked a ring only when it was its face's **one** wire, so a
/// frustum's band — ringed by arcs and holed by its own near cap — stayed
/// refused; §212 walked that too, and
/// [`a_corner_up_cube_of_side_four_round_a_frustums_apex_answers_right`] and the
/// three beside it above are what hold the line now that there is no refusal
/// left in this file to hold.
#[test]
fn a_corner_up_cube_centred_on_a_pointed_cones_apex_answers_every_rule_right() {
    let cases: Vec<Case> = [0.0, 0.7]
        .into_iter()
        .map(|turn| Case {
            cone: THE_POINTED_CONE,
            block: a_corner_up_cube(4.0, 0.0, turn),
            solids: OVERLAPPING,
        })
        .collect();
    assert_eq!(
        tally(&cases),
        Tally {
            right: 8,
            refused: [].into()
        }
    );
}

/// The cone face round the apex answers for its own nappe and turns the other
/// one away, which is what `holds.rs`'s `is_on_the_nappe_the_face_is_on` is doing
/// in front of `what_a_face_cut_by_part_of_a_crossing_holds`.
///
/// The intersection of the pointed cone with the side-6 corner-up cube turned
/// `0.7`: its cone face runs from the apex up to the arcs where the cube's three
/// upper planes cut the cone, and the apex is strictly inside all three of them,
/// which is what `is_a_family_this_follows` asks of a cone before it follows one.
///
/// **Two of these four points are why the guard is there.** `(0.5, 0, −1)` and
/// `(1, 0, −2)` are on the cone's *other* nappe, and they are in front of all
/// three cuts — a cone's surface is both its halves — so without the guard the
/// face calls them **held**, which is a wrong answer and not a refusal. The other
/// two fail the other way: without the cone arm of `is_a_family_this_follows` all
/// four refuse `AFaceTrimmedByACurveThisCannotFollow` instead of answering.
#[test]
fn a_cone_face_round_an_apex_holds_its_own_nappe_and_not_the_other() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let case = Case {
        cone: THE_POINTED_CONE,
        block: a_corner_up_cube(6.0, 0.0, 0.7),
        solids: OVERLAPPING,
    };
    let (cone, block) = build(&mut model, &case);
    let answer = model
        .fuse(
            &[cone],
            &[block],
            Rule::Intersection,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("the cube round the apex intersects the cone");
    let piece = *answer
        .solids()
        .first()
        .expect("an intersection with one piece");
    let view = model.solid(piece).expect("this model made the key");
    let skin = view
        .faces()
        .find(|face| matches!(face.surface(), Surface::Cone(_)))
        .expect("the intersection keeps a piece of the cone")
        .key();
    let asked = |point: Point| {
        view.face_holds_the_point(
            skin,
            PredicatePoint::from(point.coordinates()),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("the cone face says")
    };
    assert_eq!(asked(Point::new(0.5, 0.0, -1.0)), OnAFace::NotHeld);
    assert_eq!(asked(Point::new(1.0, 0.0, -2.0)), OnAFace::NotHeld);
    assert_eq!(asked(Point::new(0.5, 0.0, 1.0)), OnAFace::Held);
    assert_eq!(asked(Point::origin()), OnAFace::OnItsBoundary);
}

/// The shape `holds.rs`'s `is_a_family_this_follows` refuses a kept **outside**
/// for, built through the public API so the refusal has something to point at.
///
/// A slab 2 wide and 12 long, standing from `z = 1` to `z = 6` across the
/// pointed cone's axis. At its top the cone's circle has radius 3 — wider than
/// the slab and shorter than it — so the circle runs in at one long side of that
/// face and out at the other, and what is left of the face **outside** the cone
/// is two pieces with the circle between them. That is the whole reason
/// `is_a_family_this_follows` keeps `cut.kept == Decision::Neg`: a face keeping
/// the cone's inside is one convex piece, and a face keeping its outside is this.
///
/// No rule here answers wrong, which is what the name says and all this file
/// ever asks. What it is for is that the doc comment on
/// `is_a_family_this_follows` names a region in two pieces, and §16.10 wants a
/// shape and not a sentence.
///
/// **Every rule refused until §36.3's sign rule stopped gluing cells**, one
/// step before `holds.rs` was asked: the cone's face comes apart where the
/// slab's walls cut it in two hyperbolas, and the little circle `z = 1` cuts
/// round the tip matched both the piece between the hyperbolas and the two
/// outside them. Placed by which of them goes round the axis, all four answer
/// right.
///
/// **The cone less the slab is two solids**, and this was written as one while
/// every rule refused and nothing read it. The cone is no wider than the slab
/// from `z = 1` to `z = 2`, so the slab takes the whole of it there and the tip
/// below `z = 1` comes away on its own.
#[test]
fn a_slab_the_cones_circle_splits_in_two_is_never_answered_wrong() {
    let case = Case {
        cone: THE_POINTED_CONE,
        block: BoxShape {
            size: [2.0, 12.0, 5.0],
            placed: moved([-1.0, -6.0, 1.0]),
        },
        solids: [1, 1, 2, 1],
    };
    assert_eq!(
        tally(&[case]),
        Tally {
            right: 4,
            refused: BTreeMap::new()
        }
    );
}

/// The closed forms every answer is held against have to say the same as the
/// model about the two inputs, or they check nothing.
#[test]
fn the_closed_forms_agree_with_the_model_about_the_frustum_and_the_blocks() {
    let policy = Policy::certified();
    for block in [
        a_leaning_block(0.375, -1.0, 2.0),
        a_corner_up_cube(6.0, 1.0, 0.7),
    ] {
        let case = Case {
            cone: THE_FRUSTUM,
            block,
            solids: OVERLAPPING,
        };
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let (cone, block) = build(&mut model, &case);
        let mut checked = 0;
        for p in clear_grid_points(&case, 1) {
            for (solid, truth) in [
                (cone, in_cone(case.cone, p)),
                (block, in_block(case.block, p)),
            ] {
                let said = model
                    .solid(solid)
                    .expect("this model made the key")
                    .classify_point(p, &policy, &mut ProvenanceLog::new());
                let want = if truth {
                    Containment::Inside
                } else {
                    Containment::Outside
                };
                assert_eq!(said, want, "at {p:?}");
                checked += 1;
            }
        }
        assert!(checked > 10_000, "only {checked} points were compared");
    }
}

/// A pointed cone whose radius is twice its height, inside a block that holds
/// all of it: every rule has an answer, and every rule gives it.
///
/// **The cone's face was placed by a point on its other nappe** until §350.
/// The face is bounded by its rim alone, so every blend of its one corner is
/// that corner, and of the six steps a radius long from it, four land back on
/// the rim and one above the cap. The sixth goes a radius down the axis, and so
/// does the step towards the side the cap keeps. A radius here is twice the
/// height, so both land at `z = −8`, under the apex, and are put back onto the
/// cone at `(−16, 0, −8)`. That point is below the cap,
/// which was all `select.rs` asked of it, so it placed the face — outside the
/// block, which stops at `z = −1`. The block holds the face, and all four rules
/// refused `ResultIsNotManifold`.
///
/// The fix is in two halves and this fails without either. Without the cone
/// half of `the_piece_holds`, the point under the apex still places the face,
/// since it comes before any other that is inside it. Without the point half
/// way from the rim to the apex, no candidate is left inside the face and all
/// four refuse `a point inside a piece of a face`.
#[test]
fn a_cone_whose_radius_is_twice_its_height_inside_a_block_answers_every_rule_right() {
    let case = Case {
        cone: ConeShape {
            slope: 2.0,
            near: 0.0,
            far: 8.0,
        },
        block: BoxShape {
            size: [40.0, 40.0, 11.0],
            placed: moved([-20.0, -20.0, -1.0]),
        },
        solids: [1, 1, 0, 1],
    };
    assert_eq!(
        tally(&[case]),
        Tally {
            right: 4,
            refused: BTreeMap::new()
        }
    );
}

/// Q467: §350.4's three families round [`THE_STEEP_CONE`], the pointed cone of
/// slope `2` up to `z = 3`, which §350.4 swept once in a throwaway test and
/// took out. §350.4 wrote down neither its turns nor its slabs' heights, so
/// they are chosen here: turns `0`, `1` and `3` for the leaning block, `0`
/// and `0.7` for the corner-up cube, and `0`, `0.5`, `1` and `1.5` for
/// `a_slab`'s 2 by 12 by 14 box at `low` of `−14`, `−8`, `−3`, `0` and `2`.
/// Every solid count comes from [`a_leaning_blocks_solids`],
/// [`a_corner_up_cubes_solids`] or [`a_slabs_solids`], which work it out by
/// hand; none was read off the kernel.
///
/// **Each family is cut into tests of about twenty seconds**, the slow tier's
/// pieces (`docs/decisions.md` §397.4), and each test checks its answers on
/// every second point of the grid ([`tally_on_every_second_point`]). The
/// leaning block is one test per lean and turn, the corner-up cube one per
/// middle and turn, and the slab one for the slabs that start below the apex
/// and one for those that start at it or above. §350.5 gives each family's
/// tally, which its tests add up to, and the tests' walls.
///
/// Leaned `0.25` or `0.375`, the block meets the cone at every crossing, and
/// all ninety-six fuses answer right, sixteen in each test.
#[test]
fn a_block_leaned_a_quarter_unturned_across_a_steep_pointed_cone_answers_every_rule_right() {
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_STEEP_CONE, 0.25, &[0.0])),
        Tally {
            right: 16,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_block_leaned_a_quarter_turned_one_across_a_steep_pointed_cone_answers_every_rule_right() {
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_STEEP_CONE, 0.25, &[1.0])),
        Tally {
            right: 16,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_block_leaned_a_quarter_turned_three_across_a_steep_pointed_cone_answers_every_rule_right() {
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_STEEP_CONE, 0.25, &[3.0])),
        Tally {
            right: 16,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_block_leaned_three_eighths_unturned_across_a_steep_pointed_cone_answers_every_rule_right() {
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_STEEP_CONE, 0.375, &[0.0])),
        Tally {
            right: 16,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_block_leaned_three_eighths_turned_one_across_a_steep_pointed_cone_answers_every_rule_right() {
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_STEEP_CONE, 0.375, &[1.0])),
        Tally {
            right: 16,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_block_leaned_three_eighths_turned_three_across_a_steep_pointed_cone_answers_every_rule_right()
{
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_STEEP_CONE, 0.375, &[3.0])),
        Tally {
            right: 16,
            refused: BTreeMap::new()
        }
    );
}

/// The same block leaned `1` or `4`, against the steep cone's slope of `2`.
///
/// Leaned `1`, the block meets the cone at every crossing. Crossing `0`
/// turned `0`, the union and the cone-less-block refuse `CannotMeshAFace`;
/// every other lean-`1` fuse is right. Leaned `4`, crossing `0` puts the
/// leaning face through the apex and nowhere else on the cone, a touch, and
/// refuses `CannotTellTheCellsOfAFaceApart` on every rule and every turn. The
/// turn rounds that touch either way ([`a_leaning_case`]): turned `3` the face
/// misses the apex ([`APART`]), and turned `0` or `1` the block holds a tip of
/// the cone about `1e−16` tall ([`OVERLAPPING`]). Crossing `−0.5` misses the
/// cone ([`APART`]), the other two crossings meet it ([`OVERLAPPING`]), and
/// every one of those twenty-four answers right.
#[test]
fn a_block_leaned_one_unturned_across_a_steep_pointed_cone_answers_right_or_refuses() {
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_STEEP_CONE, 1.0, &[0.0])),
        Tally {
            right: 14,
            refused: [("the answer does not measure: CannotMeshAFace", 2)].into()
        }
    );
}

#[test]
fn a_block_leaned_one_turned_one_across_a_steep_pointed_cone_answers_every_rule_right() {
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_STEEP_CONE, 1.0, &[1.0])),
        Tally {
            right: 16,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_block_leaned_one_turned_three_across_a_steep_pointed_cone_answers_every_rule_right() {
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_STEEP_CONE, 1.0, &[3.0])),
        Tally {
            right: 16,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_block_leaned_four_unturned_across_a_steep_pointed_cone_answers_right_or_refuses() {
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_STEEP_CONE, 4.0, &[0.0])),
        Tally {
            right: 12,
            refused: [("CannotTellTheCellsOfAFaceApart", 4)].into()
        }
    );
}

#[test]
fn a_block_leaned_four_turned_one_across_a_steep_pointed_cone_answers_right_or_refuses() {
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_STEEP_CONE, 4.0, &[1.0])),
        Tally {
            right: 12,
            refused: [("CannotTellTheCellsOfAFaceApart", 4)].into()
        }
    );
}

#[test]
fn a_block_leaned_four_turned_three_across_a_steep_pointed_cone_answers_right_or_refuses() {
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_STEEP_CONE, 4.0, &[3.0])),
        Tally {
            right: 12,
            refused: [("CannotTellTheCellsOfAFaceApart", 4)].into()
        }
    );
}

/// A cube stood on its corner round the steep cone's apex, of side `2`, `4`
/// and `6` with its middle at `0`, `1`, `2.5` and `3.5`, as §350.4 swept it.
/// Every one shares volume with the cone. Side `6`, middle `0` refuses `order
/// along a crossing` on every rule and both turns; the other eleven answer
/// right, six of them with the block less the cone in two solids, its top cut
/// off by the cap. Each test here holds the three sides at one middle and one
/// turn.
#[test]
fn a_corner_up_cube_centred_on_a_steep_pointed_cones_apex_answers_right_or_refuses() {
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_STEEP_CONE, 0.0, 0.0)),
        Tally {
            right: 8,
            refused: [("order along a crossing", 4)].into()
        }
    );
}

#[test]
fn a_turned_corner_up_cube_centred_on_a_steep_pointed_cones_apex_answers_right_or_refuses() {
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_STEEP_CONE, 0.0, 0.7)),
        Tally {
            right: 8,
            refused: [("order along a crossing", 4)].into()
        }
    );
}

#[test]
fn a_corner_up_cube_centred_one_above_a_steep_pointed_cones_apex_answers_right() {
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_STEEP_CONE, 1.0, 0.0)),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_turned_corner_up_cube_centred_one_above_a_steep_pointed_cones_apex_answers_right() {
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_STEEP_CONE, 1.0, 0.7)),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_corner_up_cube_centred_two_and_a_half_above_a_steep_pointed_cones_apex_answers_right() {
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_STEEP_CONE, 2.5, 0.0)),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_turned_corner_up_cube_centred_two_and_a_half_above_a_steep_pointed_cones_apex_answers_right() {
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_STEEP_CONE, 2.5, 0.7)),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_corner_up_cube_centred_three_and_a_half_above_a_steep_pointed_cones_apex_answers_right() {
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_STEEP_CONE, 3.5, 0.0)),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_turned_corner_up_cube_centred_three_and_a_half_above_a_steep_pointed_cones_apex_answers_right()
{
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_STEEP_CONE, 3.5, 0.7)),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

/// Twenty slabs `14` deep across the steep cone's axis, each `a_slab`'s own
/// 2 by 12 box turned `0`, `0.5`, `1` or `1.5` about it. At `low = −14` the
/// slab's top runs through the apex, a touch ([`APART`]); it answers right
/// only turned `0` and finds no point in a piece of a face every other turn.
/// `low` from `−8` to `0` answers right turned `0`, the cone in two solids
/// either side of the slab, and splits between two more refusals the other
/// three turns; `low = 2` answers right turned `0` and splits the same way
/// turned `0.5` and `1`. This test holds the slabs from `−14`, `−8` and `−3`,
/// and the next those from `0` and `2`.
#[test]
fn a_slab_fourteen_deep_from_below_the_apex_across_a_steep_pointed_cone_answers_right_or_refuses() {
    assert_eq!(
        tally_on_every_second_point(&slabs(THE_STEEP_CONE, &[-14.0, -8.0, -3.0])),
        Tally {
            right: 18,
            refused: [
                ("a point inside a piece of a face", 24),
                ("ACurvedFaceGoesRightRoundWithNoSeam", 4),
                ("side of a half-space", 2),
            ]
            .into()
        }
    );
}

#[test]
fn a_slab_fourteen_deep_from_the_apex_up_across_a_steep_pointed_cone_answers_right_or_refuses() {
    assert_eq!(
        tally_on_every_second_point(&slabs(THE_STEEP_CONE, &[0.0, 2.0])),
        Tally {
            right: 20,
            refused: [
                ("ACurvedFaceGoesRightRoundWithNoSeam", 8),
                ("side of a half-space", 4),
            ]
            .into()
        }
    );
}

/// The same leaning block round a cone of slope `1` instead of `2` — §350.1
/// and §350.3's cone, run for the first time. Leans `0.25` and `0.375` meet
/// the cone at every crossing, and so does a lean of `1` but at two. Leaned
/// `1`, crossing `−0.5` misses the cone ([`APART`]), and crossing `0` touches
/// it along the ruling `x = z`: turned `0` and `3` the face misses it
/// ([`APART`]), and turned `1` the rounding of the turn cuts a sliver from it
/// ([`OVERLAPPING`], [`a_leaning_case`]).
///
/// Leans `0.25` and `0.375` each refuse `CannotMeshAFace` on the intersection
/// and the block-less-cone at crossing `0`, turn `1`. Leaned `1`, the same
/// crossing and turn refuses `AWireWillNotLieFlatOnItsFace` on the union and
/// the cone-less-block, and the sliver the intersection answers does not
/// measure (`TwoPatchesOfTheMeshDrewTheSameSide`). Every lean-`1` crossing
/// but `0`, turned `1`, refuses `order along a crossing` on every rule. Each
/// test here holds one lean at every crossing and turn.
#[test]
fn a_block_leaned_a_quarter_across_a_unit_pointed_cone_answers_right_or_refuses() {
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_UNIT_CONE, 0.25, &[0.0, 1.0, 3.0])),
        Tally {
            right: 46,
            refused: [("the answer does not measure: CannotMeshAFace", 2)].into()
        }
    );
}

#[test]
fn a_block_leaned_three_eighths_across_a_unit_pointed_cone_answers_right_or_refuses() {
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_UNIT_CONE, 0.375, &[0.0, 1.0, 3.0])),
        Tally {
            right: 46,
            refused: [("the answer does not measure: CannotMeshAFace", 2)].into()
        }
    );
}

#[test]
fn a_block_leaned_one_across_a_unit_pointed_cone_answers_right_or_refuses() {
    assert_eq!(
        tally_on_every_second_point(&leaning_blocks(THE_UNIT_CONE, 1.0, &[0.0, 1.0, 3.0])),
        Tally {
            right: 33,
            refused: [
                ("AWireWillNotLieFlatOnItsFace", 2),
                (
                    "the answer does not measure: TwoPatchesOfTheMeshDrewTheSameSide",
                    1
                ),
                ("order along a crossing", 12),
            ]
            .into()
        }
    );
}

/// The same block leaned four times the cone's own slope, apart from the
/// eight fuses at crossing `0`, turns `0` and `1`, which are a known defect
/// ([`a_unit_cone_union_a_lean_four_block_passes_fuse_but_fails_validate`]).
/// Crossing `0` puts the leaning face through the apex and nowhere else on
/// the cone, a touch; turned `3`, which rounds the face `1.6e−15` clear of the
/// apex ([`APART`], [`a_leaning_case`]), it refuses `ResultIsNotManifold` on
/// every rule. Crossing `−0.5` misses the cone by `2` at the apex
/// ([`APART`]), the other two crossings meet it ([`OVERLAPPING`]), and all
/// thirty-six of those answer right.
fn a_unit_pointed_cones_lean_four_cases_away_from_the_defect() -> Vec<Case> {
    let mut cases = vec![a_leaning_case(THE_UNIT_CONE, 4.0, 0.0, 3.0)];
    for crossing in [0.5, -0.5, 1.5] {
        for turn in [0.0, 1.0, 3.0] {
            cases.push(a_leaning_case(THE_UNIT_CONE, 4.0, crossing, turn));
        }
    }
    cases
}

#[test]
fn a_unit_pointed_cone_with_a_lean_of_four_answers_right_or_refuses_away_from_the_known_defect() {
    assert_eq!(
        tally_on_every_second_point(&a_unit_pointed_cones_lean_four_cases_away_from_the_defect()),
        Tally {
            right: 36,
            refused: [("ResultIsNotManifold", 4)].into()
        }
    );
}

/// The corner-up cube round the unit cone, the same sides and middles as
/// round the steep cone ([`corner_up_cubes_of_every_side`]). Every one shares
/// volume with the cone. Side `2`, middle `1` refuses `a point inside a piece
/// of a face` on every rule and both turns; the other eleven answer right.
/// Three counts turn up here that the steep cone has none of: side `4`,
/// middle `3.5` leaves the block less the cone in four, three tips round its
/// lower side corners and its top above the cap; side `6`, middles `0` and
/// `1` leave the cone less the block in three, one past each face through the
/// cube's top corner; and side `6`, middles `2.5` and `3.5` hold the whole
/// cone and leave none. Each test here holds the three sides at one middle and
/// one turn.
#[test]
fn a_corner_up_cube_centred_on_a_unit_pointed_cones_apex_answers_right() {
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_UNIT_CONE, 0.0, 0.0)),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_turned_corner_up_cube_centred_on_a_unit_pointed_cones_apex_answers_right() {
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_UNIT_CONE, 0.0, 0.7)),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_corner_up_cube_centred_one_above_a_unit_pointed_cones_apex_answers_right_or_refuses() {
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_UNIT_CONE, 1.0, 0.0)),
        Tally {
            right: 8,
            refused: [("a point inside a piece of a face", 4)].into()
        }
    );
}

#[test]
fn a_turned_corner_up_cube_centred_one_above_a_unit_pointed_cones_apex_answers_right_or_refuses() {
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_UNIT_CONE, 1.0, 0.7)),
        Tally {
            right: 8,
            refused: [("a point inside a piece of a face", 4)].into()
        }
    );
}

#[test]
fn a_corner_up_cube_centred_two_and_a_half_above_a_unit_pointed_cones_apex_answers_right() {
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_UNIT_CONE, 2.5, 0.0)),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_turned_corner_up_cube_centred_two_and_a_half_above_a_unit_pointed_cones_apex_answers_right() {
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_UNIT_CONE, 2.5, 0.7)),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_corner_up_cube_centred_three_and_a_half_above_a_unit_pointed_cones_apex_answers_right() {
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_UNIT_CONE, 3.5, 0.0)),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

#[test]
fn a_turned_corner_up_cube_centred_three_and_a_half_above_a_unit_pointed_cones_apex_answers_right()
{
    assert_eq!(
        tally_on_every_second_point(&corner_up_cubes_of_every_side(THE_UNIT_CONE, 3.5, 0.7)),
        Tally {
            right: 12,
            refused: BTreeMap::new()
        }
    );
}

/// Twenty slabs across the unit cone's axis, the same sweep as round the
/// steep cone ([`slabs`]). `low = −14` touches the apex ([`APART`]) and
/// answers right every turn; `low = −8` answers right only turned `0` and
/// finds no point in a piece of a face the other three turns; `low` of `−3`,
/// `0` and `2` answer right every turn. This test holds the slabs from `−14`,
/// `−8` and `−3`, and the next those from `0` and `2`.
#[test]
fn a_slab_fourteen_deep_from_below_the_apex_across_a_unit_pointed_cone_answers_right_or_refuses() {
    assert_eq!(
        tally_on_every_second_point(&slabs(THE_UNIT_CONE, &[-14.0, -8.0, -3.0])),
        Tally {
            right: 36,
            refused: [("a point inside a piece of a face", 12)].into()
        }
    );
}

#[test]
fn a_slab_fourteen_deep_from_the_apex_up_across_a_unit_pointed_cone_answers_every_rule_right() {
    assert_eq!(
        tally_on_every_second_point(&slabs(THE_UNIT_CONE, &[0.0, 2.0])),
        Tally {
            right: 32,
            refused: BTreeMap::new()
        }
    );
}

/// [`a_leaning_case`] round `cone` at a lean of `lean`, at crossings `0`,
/// `0.5`, `−0.5` and `1.5`, each turned by every one of `turns`.
fn leaning_blocks(cone: ConeShape, lean: f64, turns: &[f64]) -> Vec<Case> {
    let mut cases = Vec::new();
    for crossing in [0.0, 0.5, -0.5, 1.5] {
        for &turn in turns {
            cases.push(a_leaning_case(cone, lean, crossing, turn));
        }
    }
    cases
}

/// A corner-up cube of side `2`, `4` and `6` round `cone`, each with its
/// middle on the axis at `z = middle` and turned `turn` about it, counted by
/// [`a_corner_up_cubes_solids`].
fn corner_up_cubes_of_every_side(cone: ConeShape, middle: f64, turn: f64) -> Vec<Case> {
    [2.0, 4.0, 6.0]
        .into_iter()
        .map(|side| Case {
            cone,
            block: a_corner_up_cube(side, middle, turn),
            solids: a_corner_up_cubes_solids(cone, side, middle),
        })
        .collect()
}

/// [`a_slab`] across `cone` from each of `lows`, turned `0`, `0.5`, `1` and
/// `1.5`, counted by [`a_slabs_solids`].
fn slabs(cone: ConeShape, lows: &[f64]) -> Vec<Case> {
    let mut cases = Vec::new();
    for &low in lows {
        for turn in [0.0, 0.5, 1.0, 1.5] {
            cases.push(Case {
                cone,
                block: a_slab(low, turn),
                solids: a_slabs_solids(cone, low),
            });
        }
    }
    cases
}

/// The one case of the sweep above that comes back wrong rather than
/// refused: `model.fuse` answers `Ok`, but the model it hands back fails
/// [`Model::validate`]. Eight of the unit lean family's 192 fuses hit this —
/// crossing `0`, turns `0` and `1`, all four rules — and this pins the union,
/// turned `0`, directly against the two halves of `Invalid::DynamicRangeTooWide`
/// rather than through `tally`, which has no way to expect a validation
/// failure from a fuse that answered.
///
/// Before the turn the leaning face touches the unit cone at its apex and
/// nowhere else. The face as the model stores it passes `2.2e−16` beyond the
/// apex (`4.4e−16` turned `1`), so the shapes `fuse` is handed share a tip of
/// the cone about `3e−16` tall ([`a_leaning_case`]), and the union is one
/// solid. That is what `fuse` answers, with a corner where the face meets the
/// cone `3.45e−16` from the apex (`6.9e−16` turned `1`), and `validate`
/// refuses the model for it. So the defect is a fuse that answers with a model
/// its own `validate` refuses, where it should refuse.
///
/// The ratios, measured at `5.364581909073752e16` and
/// `2.6822909545368416e16`, are both `18.5` over that corner's distance from
/// the apex. They match the `5.7e16` of the doc comment on
/// [`a_block_below_a_frustum_is_cut_where_the_frustum_has_no_face_and_every_answer_is_right`]
/// in pattern only, and nothing past this is traced. Queue row Q1562.
#[test]
fn a_unit_cone_union_a_lean_four_block_passes_fuse_but_fails_validate() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let case = a_leaning_case(THE_UNIT_CONE, 4.0, 0.0, 0.0);
    let (cone, block) = build(&mut model, &case);
    model
        .fuse(
            &[cone],
            &[block],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("model.fuse answers Ok here, which is the bug");
    let error = model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect_err("the answer is corrupt: a known, unfixed defect, Q1562");
    let Error::Invalid(Invalid::DynamicRangeTooWide { ratio }) = error else {
        panic!("expected Invalid(DynamicRangeTooWide), got {error:?}");
    };
    assert!(
        ratio > 1.0e15,
        "ratio {ratio} is far smaller than the ~2.7e16-5.4e16 this defect gives"
    );
}

/// [`a_leaning_block`] round `cone`, counted by [`a_leaning_blocks_solids`]
/// except where the turn has rounded a face that only touched the apex to
/// pass beyond it.
///
/// At crossing `0` the leaning face goes through the apex, and at a lean of
/// `4` on either cone, or `1` on the unit cone, it meets the cone nowhere else.
/// Once the block is turned, the face the model stores goes through the apex
/// only to within rounding: it falls short of it or passes beyond it by up to
/// `1.6e−15`, as [`how_far_the_leaning_face_passes_the_apex`] reads it.
///
/// - **Beyond it**, the apex is inside the block, and so is a tip of the cone
///   round it, so the two share volume and the count is [`OVERLAPPING`], as at
///   any crossing that cuts the cone: the cone less the block is the cone cut
///   by one plane, and the block less the cone the block less a tip at its
///   face. Leaned `1` and turned `1`, that tip is a sliver along the ruling the
///   face touched, from the apex to `z ≈ 2.66`, and at most `6e−8` across.
/// - **Short of it**, a lean of `4` still misses the cone, its face being far
///   shallower than the cone's side. A lean of `1` misses only if the stored
///   face also leans away from the ruling it touched, and at turns `0` and `3`,
///   the two that fall short, it does: decided exactly from the stored floats,
///   `n_x² + n_y² − n_z²` of its outward normal is `−1.57e−16` and
///   `−4.38e−16`. So those stay [`APART`].
fn a_leaning_case(cone: ConeShape, lean: f64, crossing: f64, turn: f64) -> Case {
    let block = a_leaning_block(lean, crossing, turn);
    let by_hand = a_leaning_blocks_solids(cone, lean, crossing);
    let solids = if by_hand == APART && how_far_the_leaning_face_passes_the_apex(block) > 0.0 {
        OVERLAPPING
    } else {
        by_hand
    };
    Case {
        cone,
        block,
        solids,
    }
}

/// How far beyond the apex the leaning face of `block` is, as the model stores
/// it once placed: above zero when the apex is inside the block. The leaning
/// face is the block's face nearest the origin, each other one being at least
/// `5` from it.
fn how_far_the_leaning_face_passes_the_apex(block: BoxShape) -> f64 {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let key = a_block(&mut model, block);
    let face = model
        .solid(key)
        .expect("this model made the key")
        .faces()
        .min_by(|one, other| {
            one.plane()
                .offset()
                .abs()
                .total_cmp(&other.plane().offset().abs())
        })
        .expect("a block has faces");
    let plane = face.plane();
    assert_eq!(
        face.outward_normal(),
        plane.normal(),
        "a block's face used reversed"
    );
    plane.offset()
}

/// How many solids each rule leaves for [`a_leaning_block`] round a pointed
/// cone up to `z = 3`, worked out by hand for the shapes before rounding.
/// Turning the block about the axis changes no count, since the cone is
/// round; [`a_leaning_case`] says where the rounding of the turn does.
///
/// In the plane through the axis that the block leans in, the cone reaches
/// furthest into the block's side of the face `x = lean · (z − crossing)`
/// either at the apex, by `lean · crossing`, or at the rim, by
/// `(slope − lean) · far + lean · crossing`. When neither is above zero the two
/// share no volume: below zero they miss, and at zero the face touches the
/// cone at the apex or along a ruling. Either way that is [`APART`].
///
/// Otherwise every rule leaves one solid, [`OVERLAPPING`]:
/// - the union and the intersection, since both shapes are convex;
/// - the block less the cone, since the block holds `(0, 0, −1)`, under the
///   apex. A straight line from there to a point of the block outside the cone
///   and not above the cap stays in the block, which is convex, and outside
///   the cone: in the half-plane through the axis that holds it,
///   `r − slope · z` is above zero at both ends and changes linearly. At lean
///   `4` the block's top is at `z ≈ 2.18`, below the cap, and at a lean up to
///   `1` its section at the cap reaches past the rim, so what is above the cap
///   joins the rest there;
/// - the cone less the block: the block meets each ruling in one stretch, so a
///   point of the cone outside the block runs along its ruling either down to
///   the apex, which is outside the block when the crossing is not above zero,
///   or up to the cap. What the block leaves of the cap is one piece holding
///   its centre, since the centre is on the far side of the face. Past the
///   block's walls square to `y`, at `|y| > 5`, only the steep cone's cap
///   reaches, and only at `|x| < 3.4`, which takes in the far side of the face
///   too.
fn a_leaning_blocks_solids(cone: ConeShape, lean: f64, crossing: f64) -> Solids {
    assert!(
        cone.near == 0.0
            && cone.far == 3.0
            && [1.0, 2.0].contains(&cone.slope)
            && [0.25, 0.375, 1.0, 4.0].contains(&lean)
            && [0.0, 0.5, -0.5, 1.5].contains(&crossing),
        "a cone, lean or crossing this rule was not worked out for"
    );
    let into_the_block_at_the_apex = lean * crossing;
    let into_the_block_at_the_rim = (cone.slope - lean) * cone.far + lean * crossing;
    if into_the_block_at_the_apex.max(into_the_block_at_the_rim) <= 0.0 {
        return APART;
    }
    assert!(
        in_block(
            a_leaning_block(lean, crossing, 0.0),
            Point::new(0.0, 0.0, -1.0)
        ),
        "the block holds no point under the apex"
    );
    OVERLAPPING
}

/// How many solids each rule leaves for [`a_corner_up_cube`] round a pointed
/// cone, worked out by hand. The union and the intersection are one solid
/// each, both shapes being convex, and turning the cube about the axis changes
/// no count, since the cone is round.
///
/// The cube's bottom and top corners are on the axis at
/// `middle ∓ side · √3 / 2`. Its six other corners are `side · √(2/3)` from
/// the axis, three at `middle − side / (2√3)` and three at
/// `middle + side / (2√3)`. At height `z` a face through the bottom corner is
/// `(z − bottom) / √2` from the axis and a face through the top corner
/// `(top − z) / √2`, the nearest point of each being the middle of its edge of
/// the section, by the mirror through the axis.
///
/// **When the cube holds the apex** (`bottom < 0`):
/// - the cone less the cube is one solid for each piece of the cap outside
///   the cube. A point of the cone outside the cube runs up its ruling to the
///   cap without meeting the cube, which is convex and holds the apex, and
///   two pieces of the cap cannot join below it, since that path run up the
///   rulings would join them on the cap. So it is one when the cube's section
///   at the cap is empty or inside the rim. When the section's corners poke
///   past the rim, each of its faces nearer the axis than the rim leaves one
///   piece of the cap beyond it, three faces of each kind, and none leaves
///   nothing: the cube holds the whole cone;
/// - the cube less the cone is one solid below the cap, every point of it
///   joined to the bottom corner by a straight line outside the cone, as for
///   [`a_leaning_blocks_solids`], and one more when the cube's top pokes out
///   of the cap with the section there inside the rim, which cuts it off. That
///   is the count §350.4's lane got wrong.
///
/// **When it does not** (`bottom > 0`):
/// - the cone less the cube is one solid: a point of it moved straight out from
///   the axis stays outside the cube, whose section holds the axis, and the
///   cube comes through the cone's side, if at all, in three holes that touch
///   neither the apex nor the rim;
/// - the cube less the cone is the part above the cap, if any, and three tips
///   round the lower side corners when those poke out of the cone. The faces
///   through the bottom corner come inside the cone at every height when the
///   slope is over `1 / √2`, which keeps the tips apart; the upper side corners
///   inside the cone and the section at the cap inside the rim keep them off
///   the part above the cap.
fn a_corner_up_cubes_solids(cone: ConeShape, side: f64, middle: f64) -> Solids {
    assert_eq!(cone.near, 0.0, "a rule for a pointed cone");
    let rim = cone.slope * cone.far;
    let bottom = middle - side * 3.0f64.sqrt() / 2.0;
    let top = middle + side * 3.0f64.sqrt() / 2.0;
    let side_corners_below = middle - side / (2.0 * 3.0f64.sqrt());
    let side_corners_above = middle + side / (2.0 * 3.0f64.sqrt());
    assert!(
        bottom != 0.0 && bottom < cone.far && top != cone.far,
        "a cube with a corner on the apex or the cap, or wholly above the cap"
    );
    let inside_the_rim = how_far_a_corner_up_cubes_section_reaches(side, cone.far - middle)
        .is_none_or(|reach| {
            assert!(reach != rim, "a corner of the section on the rim");
            reach < rim
        });
    if bottom < 0.0 {
        let cone_less_cube = if inside_the_rim {
            1
        } else {
            let lower_faces_cut_the_rim =
                cone.far < side_corners_above && (cone.far - bottom) / 2.0f64.sqrt() < rim;
            let upper_faces_cut_the_rim =
                cone.far > side_corners_below && (top - cone.far) / 2.0f64.sqrt() < rim;
            3 * (usize::from(lower_faces_cut_the_rim) + usize::from(upper_faces_cut_the_rim))
        };
        [
            1,
            1,
            cone_less_cube,
            1 + usize::from(top > cone.far && inside_the_rim),
        ]
    } else {
        let corner_reach = side * (2.0f64 / 3.0).sqrt();
        let tips = corner_reach > cone.slope * side_corners_below;
        assert!(inside_the_rim, "a cube above the apex out past the rim");
        if tips {
            assert!(
                cone.slope > core::f64::consts::FRAC_1_SQRT_2
                    && corner_reach < cone.slope * side_corners_above,
                "tips that may join, or poke out higher up"
            );
        }
        [1, 1, 1, 3 * usize::from(tips) + usize::from(top > cone.far)]
    }
}

/// How far from the axis the corners of a corner-up cube's section reach,
/// `zeta` above its middle, or `None` past its top or bottom corner. Every
/// corner of the section is the same distance out: on the six middle edges in
/// the hexagon, and on the three edges from the end corner in the triangle.
fn how_far_a_corner_up_cubes_section_reaches(side: f64, zeta: f64) -> Option<f64> {
    let half = side * 3.0f64.sqrt() / 2.0;
    let side_corners = side / (2.0 * 3.0f64.sqrt());
    if zeta.abs() >= half {
        None
    } else if zeta.abs() <= side_corners {
        Some((side * side / 2.0 + 2.0 * zeta * zeta).sqrt())
    } else {
        Some(2.0f64.sqrt() * (half - zeta.abs()))
    }
}

/// How many solids each rule leaves for [`a_slab`] round a pointed cone,
/// worked out by hand.
///
/// A slab whose top is not above the apex shares no volume with the cone: at
/// `low = −14` its top runs through the apex, a touch, [`APART`].
///
/// Otherwise the union and the intersection are one solid each, both shapes
/// being convex, and so is the slab less the cone: the slab runs past the cap,
/// and a point of it outside the cone moved along `y` reaches an end of the
/// slab, which no rim up to `6` comes inside, and the two ends meet above the
/// cap. The cone less the slab is the cone either side of it, two solids
/// unless the tip below the slab joins them, when `low` is above zero. The
/// steep cone's rim of radius `6` touches the slab's ends and joins nothing.
fn a_slabs_solids(cone: ConeShape, low: f64) -> Solids {
    assert_eq!(cone.near, 0.0, "a rule for a pointed cone");
    let top = low + 14.0;
    if top <= 0.0 {
        return APART;
    }
    assert!(
        low < cone.far && top > cone.far && cone.slope * cone.far <= 6.0,
        "a slab that stops short of the cap, or a rim past the slab's ends"
    );
    if low > 0.0 {
        assert!(
            cone.slope * low > 1.0,
            "a tip below the slab that does not reach past both its sides"
        );
        OVERLAPPING
    } else {
        [1, 1, 2, 1]
    }
}

/// How many solids each rule leaves, in the order [`RULES`] runs them.
type Solids = [usize; 4];

/// Two solids that overlap and neither holds the other: one piece each way.
const OVERLAPPING: Solids = [1, 1, 1, 1];

/// Two solids that never meet: both, nothing, and each untouched.
const APART: Solids = [2, 0, 1, 1];

/// Union, intersection, cone less block, block less cone.
const RULES: [(Rule, bool); 4] = [
    (Rule::Union, true),
    (Rule::Intersection, true),
    (Rule::Difference, true),
    (Rule::Difference, false),
];

/// Slope one half about `+z`, from `z = 2` to `z = 8`.
const THE_FRUSTUM: ConeShape = ConeShape {
    slope: 0.5,
    near: 2.0,
    far: 8.0,
};

/// Slope one half about `+z`, from the apex at the origin to `z = 8`.
const THE_POINTED_CONE: ConeShape = ConeShape {
    slope: 0.5,
    near: 0.0,
    far: 8.0,
};

/// Slope two about `+z`, from the apex at the origin to `z = 3` — §350.4's
/// cone, whose sweep there was a throwaway test. §350.2's cone of slope two
/// runs up to `z = 8` and is swallowed by its block.
const THE_STEEP_CONE: ConeShape = ConeShape {
    slope: 2.0,
    near: 0.0,
    far: 3.0,
};

/// Slope one about `+z`, from the apex at the origin to `z = 3` — the cone
/// §350.1 and §350.3 wrote about and never ran.
const THE_UNIT_CONE: ConeShape = ConeShape {
    slope: 1.0,
    near: 0.0,
    far: 3.0,
};

#[derive(Clone, Copy, Debug)]
struct ConeShape {
    slope: f64,
    near: f64,
    far: f64,
}

#[derive(Clone, Copy, Debug)]
struct BoxShape {
    size: [f64; 3],
    placed: Isometry,
}

#[derive(Clone, Copy, Debug)]
struct Case {
    cone: ConeShape,
    block: BoxShape,
    solids: Solids,
}

#[derive(Debug, PartialEq)]
struct Tally {
    right: usize,
    refused: BTreeMap<&'static str, usize>,
}

/// A block 12 by 10 by 15 whose low-`x` face is the plane
/// `x = lean · (z − crossing)`, the block on the `+x` side of it, then turned
/// about the cone's axis.
///
/// Only 10 wide and running from `z ≈ −6` to `z ≈ 9`, so its two walls square
/// to `y` stay clear of the double cone and its top and bottom cut it in
/// ellipses: the one steep face is the face under test.
fn a_leaning_block(lean: f64, crossing: f64, turn: f64) -> BoxShape {
    let placed = about_z(turn)
        .compose(&moved([-lean * crossing, 0.0, 0.0]))
        .compose(&about_y(lean.atan()))
        .compose(&moved([0.0, -5.0, -6.0]));
    BoxShape {
        size: [12.0, 10.0, 15.0],
        placed,
    }
}

/// A cube with its diagonal brought upright and its middle on the axis at
/// `z = middle`, turned about the axis.
fn a_corner_up_cube(side: f64, middle: f64, turn: f64) -> BoxShape {
    let diagonal = Axis::new(Point::origin(), Vector::new(1.0, -1.0, 0.0)).expect("a direction");
    let stood = Isometry::rotation(&diagonal, (1.0f64 / 3.0).sqrt().acos()).expect("a finite turn");
    let placed = about_z(turn)
        .compose(&moved([0.0, 0.0, middle - side * 3.0f64.sqrt() / 2.0]))
        .compose(&stood);
    BoxShape {
        size: [side; 3],
        placed,
    }
}

/// A slab 2 wide, 12 long and 14 deep, standing across the axis from
/// `z = low` to `z = low + 14`, turned about the axis.
fn a_slab(low: f64, turn: f64) -> BoxShape {
    let placed = about_z(turn).compose(&moved([-1.0, -6.0, low]));
    BoxShape {
        size: [2.0, 12.0, 14.0],
        placed,
    }
}

fn about_y(angle: f64) -> Isometry {
    let axis = Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("a direction");
    Isometry::rotation(&axis, angle).expect("a finite turn")
}

fn about_z(angle: f64) -> Isometry {
    Isometry::rotation(&Axis::z(), angle).expect("a finite turn")
}

fn moved([x, y, z]: [f64; 3]) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite move")
}

/// Every way a fuse in this file refuses today, and the two ways an answer
/// that holds every grid point it is asked about can still not be checked:
/// a mesh finer than the mesher reaches, and a face it cannot mesh.
const REFUSALS: [&str; 11] = [
    "a point inside a piece of a face",
    "the answer does not measure: DeflectionBelowWhatAMeshCanReach",
    "the answer does not measure: CannotMeshAFace",
    "Geometry(CannotNameWhichPieceOfACrossingThisIs)",
    "order along a crossing",
    "ACurvedFaceGoesRightRoundWithNoSeam",
    "side of a half-space",
    "CannotTellTheCellsOfAFaceApart",
    "AWireWillNotLieFlatOnItsFace",
    "the answer does not measure: TwoPatchesOfTheMeshDrewTheSameSide",
    "ResultIsNotManifold",
];

/// Runs every rule over every case. Panics on a wrong answer; counts the
/// right ones and the refusals by name, and prints each refusal with the
/// place of its case in `cases`.
fn tally(cases: &[Case]) -> Tally {
    tally_on_a_grid(cases, 1)
}

/// [`tally`], checking the answers on every second point of
/// [`clear_grid_points`]'s grid along each axis: 4225 points, a step of `1`,
/// where the whole grid has 30 625 and a step of `0.5`.
///
/// Q467's families round [`THE_STEEP_CONE`] and [`THE_UNIT_CONE`] take this.
/// On the whole grid the check of one fuse took up to 55 s, and of one
/// union 26 s were `classify_point` on the 29 492 of its 30 222 points that
/// lie outside both shapes. No cut of those families came near the slow tier's
/// pieces of about 20 s (`docs/decisions.md` §350.5).
fn tally_on_every_second_point(cases: &[Case]) -> Tally {
    tally_on_a_grid(cases, 2)
}

fn tally_on_a_grid(cases: &[Case], stride: usize) -> Tally {
    let mut tally = Tally {
        right: 0,
        refused: BTreeMap::new(),
    };
    for (place, case) in cases.iter().enumerate() {
        for (which, (rule, cone_first)) in RULES.into_iter().enumerate() {
            match fuse_and_check(case, rule, cone_first, case.solids[which], stride) {
                None => tally.right += 1,
                Some(name) => {
                    println!("case {place}, {rule:?} (cone first: {cone_first}): {name}");
                    let known = REFUSALS
                        .into_iter()
                        .find(|it| *it == name)
                        .unwrap_or_else(|| panic!("a refusal no family here expects: {name}"));
                    *tally.refused.entry(known).or_default() += 1;
                }
            }
        }
    }
    println!("{tally:?}");
    tally
}

/// `None` when the fuse answered and the answer is right; the refusal's name
/// when it refused.
///
/// # Panics
///
/// When the fuse answers wrongly, which is what this file is looking for.
fn fuse_and_check(
    case: &Case,
    rule: Rule,
    cone_first: bool,
    want_solids: usize,
    stride: usize,
) -> Option<String> {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let (cone, block) = build(&mut model, case);
    let (arguments, tools) = if cone_first {
        ([cone], [block])
    } else {
        ([block], [cone])
    };
    let answer = match model.fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new()) {
        Ok(answer) => answer,
        Err(Error::Undecidable { records }) => {
            let record = records.records().last().expect("a refusal names why");
            return Some(record.predicate().to_owned());
        }
        Err(other) => {
            let text = format!("{other:?}");
            return Some(text.split(' ').next().unwrap_or_default().to_owned());
        }
    };
    let what = format!("{rule:?} (cone first: {cone_first}) of {case:?}");
    let pieces: Vec<SolidKey> = answer.solids().to_vec();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|error| panic!("{what}: the answer does not validate: {error:?}"));
    assert_eq!(pieces.len(), want_solids, "{what}: how many solids");

    let c = case.cone;
    let cone_volume =
        core::f64::consts::PI * c.slope * c.slope * (c.far.powi(3) - c.near.powi(3)) / 3.0;
    let block_volume: f64 = case.block.size.iter().product();
    let shared = shared_volume(case);
    let want = match (rule, cone_first) {
        (Rule::Union, _) => cone_volume + block_volume - shared,
        (Rule::Intersection, _) => shared,
        (Rule::Difference, true) => cone_volume - shared,
        (Rule::Difference, false) => block_volume - shared,
        _ => panic!("a rule this file does not run"),
    };
    // An answer the mesher cannot reach the deflection for is counted by name
    // rather than as right, once the grid below has had its say about it.
    let how_fine = Deflection::of(0.002).expect("a positive length");
    let measured: Result<Vec<_>, _> = pieces
        .iter()
        .map(|&piece| {
            model
                .solid(piece)
                .expect("this model made the key")
                .measure(how_fine, &policy, &mut ProvenanceLog::new())
        })
        .collect();
    if let Ok(measured) = &measured {
        let got: f64 = measured.iter().map(|each| each.volume()).sum();
        let bound: f64 = measured.iter().map(|each| each.volume_error()).sum();
        assert!(
            bound.is_finite(),
            "{what}: a bound of {bound} checks nothing"
        );
        let slack = bound + 0.002 * cone_volume;
        assert!(
            (got - want).abs() <= slack,
            "{what}: volume {got}, where the shapes say {want} (allowed {slack})"
        );
    }

    let mut decided = 0;
    for p in clear_grid_points(case, stride) {
        let a = in_cone(case.cone, p);
        let b = in_block(case.block, p);
        let truth = match (rule, cone_first) {
            (Rule::Union, _) => a || b,
            (Rule::Intersection, _) => a && b,
            (Rule::Difference, true) => a && !b,
            (Rule::Difference, false) => b && !a,
            _ => panic!("a rule this file does not run"),
        };
        let mut inside = false;
        let mut sure = true;
        for &piece in &pieces {
            match model
                .solid(piece)
                .expect("this model made the key")
                .classify_point(p, &policy, &mut ProvenanceLog::new())
            {
                Containment::Inside => inside = true,
                Containment::Outside => {}
                Containment::OnBoundary
                | Containment::Undecidable { .. }
                | Containment::CannotFollowAFacesTrimming { .. }
                | Containment::CannotFireARayAtAFacesSurface { .. }
                | _ => sure = false,
            }
        }
        if sure {
            assert_eq!(inside, truth, "{what}: at {p:?}");
            decided += 1;
        }
    }
    // 9 300, against the 9326 the fewest of the file's 152 answers decides.
    // It was 8 800 against 8879 until `holds.rs` learned to follow a flat face
    // cut by an arc of a cone's rim: the frustum's big cap, cut in two by a
    // block's plane, was the face 16040 of one answer's 20780 undecided points
    // refused on, and §204.3's two candidates — the turning-point box and the
    // ray caster's outline — were neither of them it.
    //
    // **What holds the floor is not the leaning block's hyperbola**, which is
    // what this comment claimed until 2026-09-23 and what §208.6 read off it.
    // The four answers at 9326 are all the side-6 **corner-up cube** centred on
    // the apex and turned `0.7` about it, on the pointed cone and on the
    // frustum; every leaning block sits above 10 500. A corner-up cube's faces
    // lean `acos(1/√3) ≈ 54.7°` off the axis, which for slope one half is inside
    // the closed family, so each of them cuts an **ellipse** — the doc comment
    // on `a_corner_up_cube_of_side_four_round_a_frustums_apex_answers_right` says so — and
    // every step here now follows an ellipse arc as it follows a circle's. The
    // count did not budge for that, so whatever leaves 9326 here is something
    // else again and nobody has named it. §211.
    //
    // **The cone-face arm of `is_a_family_this_follows` did not move it either.**
    // That arm lands with
    // [`a_cone_face_round_an_apex_holds_its_own_nappe_and_not_the_other`] and it
    // works: the same cube's `Intersection` on the pointed cone now decides
    // **30 101 of 30 101**, which is the figure §217.4 forecast for it from
    // 23 094. But the four answers at 9326 are the `Union` and the
    // block-less-cone `Difference` of that cube, and neither moved by a point. So
    // whatever holds this floor is not the cone face either.
    //
    // It was three answers and 128 until §212 let the frustum's band be meshed
    // through a hole: the frustum's own `Difference (cone first: false)` was one
    // of that test's 24 refusals and now decides 9326 too, which is the same
    // shape reaching the same floor and not a new cause.
    //
    // **It is 5 500 since §36.3 stopped gluing cells**, which let two answers
    // through that decide 5529 each and agree with the shapes at every one: the
    // union and the cone-less-block of the family below the frustum's block
    // from `−8` to `1.875` leaned a quarter, whose corner reaches into the
    // frustum. The union of the steep block of that family that reaches up to
    // `1.5` decides 7327, and it has been measured since Q423. The union and
    // the block-less-cone of the steep block that reaches up to `1` decide 6986
    // and 8579 since §340.10, and both measure. Every other answer still decides
    // at least 9326.
    //
    // **An answer that does not measure is held to 7 000.** It was set for
    // that union, whose kept piece of the cone is bounded by an arc of the
    // parabola the block's top face cuts: the ray caster does not follow that
    // arc far enough for more.
    //
    // **On every second point the floors are 1 500 and 4 000**, against the
    // 1553 and 4159 the fewest of Q467's families decide there. The 1553 is
    // the union of the steep cone and the block leaned a quarter, crossing
    // `1.5`, turned `3`.
    let floor = match (stride, measured.is_ok()) {
        (1, true) => 5_500,
        (1, false) => 7_000,
        (2, true) => 1_500,
        (2, false) => 4_000,
        _ => panic!("no floor is set for a stride of {stride}"),
    };
    assert!(
        decided > floor,
        "{what}: only {decided} grid points could be classified"
    );
    match measured {
        Ok(_) => None,
        Err(error) => Some(format!(
            "the answer does not measure: {}",
            format!("{error:?}").split(' ').next().unwrap_or_default()
        )),
    }
}

fn build(model: &mut Model, case: &Case) -> (SolidKey, SolidKey) {
    let cone = if case.cone.near == 0.0 {
        a_pointed_cone(model, case.cone.slope, case.cone.far)
    } else {
        a_frustum(model, case.cone.slope, case.cone.near, case.cone.far)
    };
    (cone, a_block(model, case.block))
}

fn a_block(model: &mut Model, block: BoxShape) -> SolidKey {
    let [x, y, z] = block.size;
    let rectangle = ProfileBuilder::new(vec![[0.0, 0.0], [x, 0.0], [x, y], [0.0, y]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    model
        .add_extrusion(&rectangle, z, &block.placed)
        .expect("a rectangle extrudes")
}

/// Points `stride` halves of a unit apart over `|x|, |y| ≤ 6`, `|z| ≤ 12`,
/// nudged off the lattice so none lands on an axis or a cap, and keeping only
/// those further than `0.02` from every face of either input.
fn clear_grid_points(case: &Case, stride: usize) -> impl Iterator<Item = Point> + '_ {
    (-12..=12).step_by(stride).flat_map(move |i| {
        (-12..=12).step_by(stride).flat_map(move |j| {
            (-24..=24).step_by(stride).filter_map(move |k| {
                let p = Point::new(
                    f64::from(i) * 0.5 + 0.1,
                    f64::from(j) * 0.5 + 0.07,
                    f64::from(k) * 0.5 + 0.03,
                );
                (distance_to_a_boundary(case, p) >= 0.02).then_some(p)
            })
        })
    })
}

fn in_cone(cone: ConeShape, p: Point) -> bool {
    let r = p.x().hypot(p.y());
    p.z() >= cone.near && p.z() <= cone.far && r <= cone.slope * p.z()
}

fn in_block(block: BoxShape, p: Point) -> bool {
    let local = block.placed.inverse().apply_to_point(p).coordinates();
    (0..3).all(|i| local[i] >= 0.0 && local[i] <= block.size[i])
}

/// Not the exact distance, but never more than it: the distance to the
/// nearest of the cone's surface, the two cap planes and the box's six planes.
fn distance_to_a_boundary(case: &Case, p: Point) -> f64 {
    let cone = case.cone;
    let r = p.x().hypot(p.y());
    let to_side = (cone.slope * p.z() - r).abs() / cone.slope.hypot(1.0);
    let to_caps = (p.z() - cone.near).abs().min((p.z() - cone.far).abs());
    let local = case.block.placed.inverse().apply_to_point(p).coordinates();
    let to_block = (0..3)
        .map(|i| local[i].abs().min((local[i] - case.block.size[i]).abs()))
        .fold(f64::INFINITY, f64::min);
    to_side.min(to_caps).min(to_block)
}

/// The volume the cone and the box share, summed over 800 × 800 vertical
/// columns across the cone's widest circle: along each column the cone is one
/// interval in `z`, and the box, being convex, is another.
fn shared_volume(case: &Case) -> f64 {
    let cone = case.cone;
    let reach = cone.slope * cone.far;
    let steps = 800;
    let h = 2.0 * reach / f64::from(steps);
    let inverse = case.block.placed.inverse();
    let up = inverse
        .apply_to_vector(Vector::new(0.0, 0.0, 1.0))
        .components();
    let mut total = 0.0;
    for i in 0..steps {
        for j in 0..steps {
            let x = -reach + (f64::from(i) + 0.5) * h;
            let y = -reach + (f64::from(j) + 0.5) * h;
            let (mut from, mut to) = (cone.near.max(x.hypot(y) / cone.slope), cone.far);
            let at = inverse.apply_to_point(Point::new(x, y, 0.0)).coordinates();
            for k in 0..3 {
                if up[k].abs() < 1e-15 {
                    if at[k] < 0.0 || at[k] > case.block.size[k] {
                        to = from;
                    }
                    continue;
                }
                let one = -at[k] / up[k];
                let other = (case.block.size[k] - at[k]) / up[k];
                from = from.max(one.min(other));
                to = to.min(one.max(other));
            }
            if to > from {
                total += (to - from) * h * h;
            }
        }
    }
    total
}
