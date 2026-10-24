//! A box whose top corner, where three planes meet, a pyramid whose apex,
//! where four planes meet, and a half bar whose corner, where a cap, a round
//! wall and a cut meet, are moved in their file, read back, and fused.
//! `docs/decisions.md` §294.16 to §294.21 are the record
//! and this file is their measurement.
//!
//! §294 moved a round bar's top corner, which has only two surfaces round it,
//! and found no wrong `Ok` and no panic
//! (`a_broken_two_surface_corner_read_back_from_bytes.rs`). §294.7 left a corner
//! with three surfaces round it untried. Here it is the top corner of a box, `x`
//! and `y` from -3 to 3 and `z` from 0 to 10, where the planes `x = 3`, `y = 3`
//! and `z = 10` meet. The file moves it a millimetre four ways, at four
//! placements, and fuses each moved box against §294.2's partners and rules,
//! under `certified` and under `tolerant(100)`.
//!
//! **Every one of those fuses refuses with `Error::Undecidable`, one record
//! named "where three surfaces meet", and leaves the model as it was.** None is
//! a wrong `Ok` and none panics. Unlike the bar's, the box's moved corner is
//! refused even where the partner cuts it away, and even where the partner is
//! nowhere near. Before a fuse looks at the partner, it works out every corner
//! of every input from the surfaces round it. A corner with three planes round
//! it is where the three planes cross, and the corner the file stores only
//! says which crossing it is. A stored corner further from the crossing than
//! `validate` lets a corner sit off its plane says none, nothing else places
//! the corner, and the fuse gives up.
//!
//! That refusal names the arithmetic and not the file: `validate` is what says
//! the file is broken, `VertexOffItsFacePlane { distance: 1.0 }`. §294.16 says
//! why the fuse does not say it too.
//!
//! The pyramid stands on the box's footprint with its apex at `(0, 0, 10)`,
//! where its four side planes meet, and §294.17 moves the apex a millimetre
//! four ways. **Every fuse of a moved apex refuses as the box's corner does.**
//! Each three of the four planes cross at the apex alone, so no three of them
//! place an apex a millimetre away. The sound pyramid answers every partner
//! but one, a block whose face runs exactly through the apex. Cut by that, it
//! answers where it stands. Placed, its four side planes cut the cut's face
//! into specks a rounding across by the apex, and the half pyramid built round
//! them keeps an edge a rounding long, which `certified` refuses and
//! `tolerant(100)` mostly takes: a defect of its own, pinned here as it stands
//! (§294.21). Cut where it was built and placed after, it answers half of
//! itself at every placement (§448.7).
//!
//! A wedge whose edge runs up through the apex answers too, where the pyramid
//! stands and cut there and placed after. **Placed first, it is refused
//! `CannotTellTheCellsOfAFaceApart` by every rule under both policies** at 50
//! of the 112 wedges and placements Q1937's lane swept. At 8 of the 50 the
//! order along a side face's edge goes round in a loop by the apex, which is a
//! rounding off that edge: the apex is put in order against each of the two
//! places the wedge's faces cross the edge by that face alone, and the two
//! places against each other by the boxes or the planes round them. Ending
//! the loop in the 5 where `as_the_planes_say` makes it changed none of the
//! 672 fuses swept, and the other 42 refuse with no loop, so the loop is not
//! what refuses. The apex is. Placed, each three of the four side planes meet
//! in an apex of their own, a rounding from the others, and the fuse puts the
//! one corner it makes of them on every side plane's line across a face. At
//! each of the 23 fuses of 39 tried here that are refused so, a wedge plane
//! runs between two of those apexes, and then no drawing of the face with
//! straight lines keeps the order along each of them. Drawing it takes one
//! reading of the corner that every face round it shares, a row of its own.
//! Turned a tenth of a radian about `(0.3, −1, 2)`, the wedge
//! round the `x` axis keeps an edge a rounding long by the apex, as the cut
//! through it does: `certified` refuses every rule for it, and
//! `tolerant(100)` answers the difference and the union and is refused the
//! intersection for a triangle a rounding across. The union panicked there
//! until the fuse left out a stretch of a wedge face's outline that the other
//! wedge face walks back. Turned 0.4 about `x`, the wedge over the `+y` edge
//! cuts the `−x` side face into a triangle a rounding across by the apex that
//! no point could be placed in, and every rule was refused `Undecidable` under
//! both policies until the fuse read the triangle's corner at the apex where
//! the lines of its two sides cross. Now `certified` refuses it at the range
//! check, and so it does at the two shifts after a turn where the same wedge
//! was refused `Undecidable` too; `tolerant(100)` answers at the turn about
//! `x`.
//!
//! The half bar is a round bar of radius 3 and height 10 less the block that
//! takes its `y > 0` half. Its corner at `(−3, 0, 10)` has the cap's plane,
//! the wall's cylinder and the cut's plane `y = 0` round it, and those three
//! meet at `(3, 0, 10)` as well, where the half bar keeps a corner of its own.
//! §294.18 moves the corner a millimetre four ways, and **every fuse refuses
//! as the box's corner does**, though one of its surfaces is curved. It also
//! moves the corner onto `(3, 0, 10)`, the one move that could pick the other
//! place and come back a wrong `Ok`. The fuse does put the corner there, on
//! top of the half bar's own, and **every fuse refuses**, most of them as
//! `validate` does, with `EdgeHasNoLength`. The sound half bar answers every
//! partner, the block that cut it too, fused again flush along its flat face,
//! at every placement (§294.24).
//!
//! The corner is moved in the bytes because nothing in the public API writes a
//! corner off its surfaces: every builder validates what it built.

use std::f64::consts::PI;

use rcad_geom::{
    Axis, DYNAMIC_RANGE_CHECK, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale,
    Surface, Vector, WHERE_THREE_SURFACES_MEET,
};
use rcad_predicates::{Decision, Policy, ProvenanceLog, side_of_a_plane_where_three_planes_meet};
use rcad_topo::{
    DraftEdge, DraftStep, Error, Fused, Invalid, Model, ProfileBuilder, Rule, SolidBuilder,
    SolidKey,
};

/// What a fuse's record names when it cannot say which of two points comes
/// first along a crossing. `rcad-topo` keeps the name private, in
/// `fuse/arrange.rs`.
const ALONG_A_CROSSING: &str = "order along a crossing";

/// The box's footprint runs from `-SIDE` to `SIDE` in `x` and in `y`.
const SIDE: f64 = 3.0;
const HEIGHT: f64 = 10.0;

/// Where the box puts the corner the file moves: the top corner on the `+x`,
/// `+y` side, where the planes `x = 3`, `y = 3` and `z = 10` meet.
const WHERE_THE_BOX_PUTS_IT: [f64; 3] = [SIDE, SIDE, HEIGHT];

/// A millimetre out from the `x = 3` wall, still on the other wall and the top.
const OFF_A_WALL: [f64; 3] = [SIDE + 1.0, SIDE, HEIGHT];

/// A millimetre above the top, still on both walls.
const OFF_THE_TOP: [f64; 3] = [SIDE, SIDE, HEIGHT + 1.0];

/// A millimetre into the box from each of its three planes.
const INTO_THE_BOX: [f64; 3] = [SIDE - 1.0, SIDE - 1.0, HEIGHT - 1.0];

/// A millimetre out from each of its three planes.
const OFF_ALL_THREE: [f64; 3] = [SIDE + 1.0, SIDE + 1.0, HEIGHT + 1.0];

const EVERY_MOVE: [[f64; 3]; 4] = [OFF_A_WALL, OFF_THE_TOP, INTO_THE_BOX, OFF_ALL_THREE];

/// Where the pyramid on the box's footprint puts its apex, where its four side
/// planes meet.
const WHERE_THE_PYRAMID_PUTS_ITS_APEX: [f64; 3] = [0.0, 0.0, HEIGHT];

/// A millimetre above the apex, off all four planes.
const ABOVE_THE_APEX: [f64; 3] = [0.0, 0.0, HEIGHT + 1.0];

/// A millimetre down into the pyramid, off all four planes.
const DOWN_INTO_THE_PYRAMID: [f64; 3] = [0.0, 0.0, HEIGHT - 1.0];

/// A millimetre down the edge to the base corner `(3, 3, 0)`, still on the two
/// planes either side of it to a rounding of `0.3`, off the other two.
const DOWN_AN_EDGE: [f64; 3] = [0.3, 0.3, HEIGHT - 1.0];

/// A millimetre down the `+x` face, on its plane to a rounding of `0.3`, off
/// the other three.
const DOWN_A_FACE: [f64; 3] = [0.3, 0.0, HEIGHT - 1.0];

const EVERY_MOVE_OF_THE_APEX: [[f64; 3]; 4] = [
    ABOVE_THE_APEX,
    DOWN_INTO_THE_PYRAMID,
    DOWN_AN_EDGE,
    DOWN_A_FACE,
];

/// The round bar's, which `add_round_bar` stands on the `z` axis with its seam
/// at `(RADIUS, 0)`.
const RADIUS: f64 = 3.0;

/// Where the round bar less [`THE_HALF_PAST_THE_SEAM`] keeps the corner the
/// file moves: the far end of the cut across its top, where the cap's plane
/// `z = 10`, the wall's cylinder and the cut's plane `y = 0` meet.
const WHERE_THE_HALF_BAR_PUTS_IT: [f64; 3] = [-RADIUS, 0.0, HEIGHT];

/// The other place those three surfaces meet: the near end of the cut across
/// the top, where the half bar keeps a corner of its own.
const THE_OTHER_END_OF_THE_CUT: [f64; 3] = [RADIUS, 0.0, HEIGHT];

/// A millimetre out from the wall, still on the cap and the cut.
const OFF_THE_ROUND_WALL: [f64; 3] = [-RADIUS - 1.0, 0.0, HEIGHT];

/// A millimetre above the cap, still on the wall and the cut.
const OFF_THE_CAP: [f64; 3] = [-RADIUS, 0.0, HEIGHT + 1.0];

/// A millimetre into the half bar from the cap and the cut, and in from the
/// wall.
const INTO_THE_HALF_BAR: [f64; 3] = [-RADIUS + 1.0, -1.0, HEIGHT - 1.0];

/// A millimetre out from the cap and the cut, and out from the wall.
const OFF_THE_CAP_THE_WALL_AND_THE_CUT: [f64; 3] = [-RADIUS - 1.0, 1.0, HEIGHT + 1.0];

const EVERY_MOVE_OF_THE_HALF_BAR: [[f64; 3]; 4] = [
    OFF_THE_ROUND_WALL,
    OFF_THE_CAP,
    INTO_THE_HALF_BAR,
    OFF_THE_CAP_THE_WALL_AND_THE_CUT,
];

/// A block, as the two corners of its footprint, the height of its floor and
/// how tall it stands. The blocks are §294.2's and keep its names, which speak
/// of the bar's seam: the box has no seam, and the bar's seam corner and the
/// box's moved corner are both on the `+x` side.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Block {
    low: [f64; 2],
    high: [f64; 2],
    floor: f64,
    tall: f64,
}

/// Nowhere near the box.
const FAR_AWAY: Block = Block {
    low: [20.0, 20.0],
    high: [22.0, 22.0],
    floor: 20.0,
    tall: 2.0,
};

/// Over the bottom two millimetres of the box.
const OVER_THE_BOTTOM: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 5.0],
    floor: -1.0,
    tall: 3.0,
};

/// Over the top two millimetres of the box, and the moved corner with them.
const OVER_THE_TOP: Block = Block {
    low: [-5.0, -5.0],
    high: [5.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// Over the top two millimetres of the `x > 2` side, which holds the corner.
const OVER_THE_RIM_BY_THE_SEAM: Block = Block {
    low: [2.0, -5.0],
    high: [5.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// Over the top two millimetres of the `x < -2` side, away from the corner.
const OVER_THE_RIM_AWAY_FROM_THE_SEAM: Block = Block {
    low: [-5.0, -5.0],
    high: [-2.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// A notch in the top of the `y = 3` wall, halfway along it.
const A_NOTCH_A_QUARTER_TURN_ROUND: Block = Block {
    low: [-1.0, 2.0],
    high: [1.0, 5.0],
    floor: 8.0,
    tall: 5.0,
};

/// The whole `y > 0` half of the box, the corner with it.
const THE_HALF_PAST_THE_SEAM: Block = Block {
    low: [-5.0, 0.0],
    high: [5.0, 5.0],
    floor: -1.0,
    tall: 12.0,
};

/// §294.2's ten partners and rules, and `OVER_THE_TOP` kept as well, which
/// cuts the corner away.
const EVERY_PARTNER: [(Block, Rule); 11] = [
    (FAR_AWAY, Rule::Union),
    (OVER_THE_BOTTOM, Rule::Difference),
    (OVER_THE_TOP, Rule::Intersection),
    (OVER_THE_TOP, Rule::Difference),
    (OVER_THE_RIM_BY_THE_SEAM, Rule::Difference),
    (OVER_THE_RIM_BY_THE_SEAM, Rule::Union),
    (OVER_THE_RIM_BY_THE_SEAM, Rule::Intersection),
    (OVER_THE_RIM_AWAY_FROM_THE_SEAM, Rule::Difference),
    (A_NOTCH_A_QUARTER_TURN_ROUND, Rule::Difference),
    (THE_HALF_PAST_THE_SEAM, Rule::Difference),
    (THE_HALF_PAST_THE_SEAM, Rule::Intersection),
];

/// A wedge from `z = −1` to `z = 11` whose edge runs up the `z` axis, through
/// the pyramid's apex: the slice `low·x ≤ y ≤ high·x` out to `x = 5`.
#[derive(Clone, Copy, Debug)]
struct Wedge {
    low: f64,
    high: f64,
}

/// Either side of the `x` axis, so both its faces cut the `+x` side face.
const A_WEDGE_ROUND_THE_X_AXIS: Wedge = Wedge {
    low: -0.5,
    high: 0.5,
};

/// From the `+x` side face over the edge to the `+y` one.
const A_WEDGE_OVER_THE_PLUS_Y_EDGE: Wedge = Wedge {
    low: 0.5,
    high: 2.0,
};

/// From the `+x` side face over the edge to the `−y` one.
const A_WEDGE_OVER_THE_MINUS_Y_EDGE: Wedge = Wedge {
    low: -2.0,
    high: -0.5,
};

const EVERY_WEDGE: [Wedge; 3] = [
    A_WEDGE_ROUND_THE_X_AXIS,
    A_WEDGE_OVER_THE_PLUS_Y_EDGE,
    A_WEDGE_OVER_THE_MINUS_Y_EDGE,
];

/// Both faces on the `+y` half of the `+x` side face, so the side of its
/// profile that runs back to the origin, from `(5, 3.75)`, is the only one
/// whose plane is tilted and does not start at the origin.
const A_WEDGE_ON_ONE_SIDE_OF_THE_X_AXIS: Wedge = Wedge {
    low: 0.25,
    high: 0.75,
};

/// What a fuse of the half bar refused with, as much of it as the pins below
/// tell apart.
#[derive(Debug, PartialEq)]
enum Refused {
    Invalid(Invalid),
    /// The predicates the refusal's records name, in order.
    Undecidable(Vec<&'static str>),
    TwoCurvesTouchWithoutCrossing,
}

/// Each move takes the corner a millimetre off at least one of its planes, and
/// `validate` says so under both policies at every placement. The same bytes
/// with the corner left alone validate.
#[test]
fn a_box_whose_top_corner_is_moved_a_millimetre_in_its_file_reads_back_and_does_not_validate() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for placement in every_placement() {
            for moved_to in EVERY_MOVE {
                let (model, _) = read_back(placement, moved_to);
                let case = format!("{policy:?}, {placement:?}, the corner at {moved_to:?}");
                match model.validate(&policy, &mut ProvenanceLog::new()) {
                    Err(Error::Invalid(found)) => assert_eq!(
                        found,
                        Invalid::VertexOffItsFacePlane { distance: 1.0 },
                        "{case}"
                    ),
                    other => panic!("{case} validates as {other:?}"),
                }
            }
            let (sound, _) = read_back(placement, WHERE_THE_BOX_PUTS_IT);
            sound
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|error| {
                    panic!("{policy:?}, {placement:?}: the sound box does not validate: {error:?}")
                });
        }
    }
}

/// The control for the refusals below: the same bytes with the corner left
/// alone answer every partner, and the answer holds what box arithmetic says
/// the two boxes hold. So what refuses the moved box is the moved corner, not
/// the partner, the rule or the placement.
#[test]
fn the_sound_box_answers_every_partner_with_what_the_two_boxes_hold() {
    answers_with_what_the_two_hold(
        |placement| read_back(placement, WHERE_THE_BOX_PUTS_IT),
        &EVERY_PARTNER,
        what_the_two_boxes_hold,
    );
}

/// **Every partner, every rule, every placement and both policies give the
/// same refusal**: `Undecidable`, with one record, "where three surfaces meet",
/// and the model byte for byte as it was. That includes the partner far away
/// and the partners that cut the corner away, which the bar answered (§294.2):
/// the box's corners are all worked out before the partner is looked at, and
/// the moved one is too far from where its three planes cross to say that it
/// is that crossing.
#[test]
fn every_fuse_of_a_box_whose_top_corner_is_moved_refuses_where_three_surfaces_meet() {
    every_fuse_refuses_where_three_surfaces_meet(read_back, EVERY_MOVE);
}

/// Each move takes the apex a millimetre off at least two of its four planes,
/// and `validate` says so under both policies at every placement. The
/// distance it names is to one plane the apex is off, not the farthest: down
/// the face the apex is `3/√109` off the `±y` planes and `6/√109` off the `−x`
/// plane, and `validate` names `3/√109`. The same bytes with the apex left
/// alone validate.
#[test]
fn a_pyramid_whose_apex_is_moved_a_millimetre_in_its_file_reads_back_and_does_not_validate() {
    // A side plane rises `HEIGHT` over `SIDE`, so a millimetre up or down, or
    // a millimetre out from the middle, is this far off it.
    let a_millimetre_off_a_side = SIDE / SIDE.hypot(HEIGHT);
    let off_by = [
        (ABOVE_THE_APEX, a_millimetre_off_a_side),
        (DOWN_INTO_THE_PYRAMID, a_millimetre_off_a_side),
        (DOWN_AN_EDGE, 2.0 * a_millimetre_off_a_side),
        (DOWN_A_FACE, a_millimetre_off_a_side),
    ];
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for placement in every_placement() {
            for (moved_to, distance) in off_by {
                let (model, _) = the_pyramid_read_back(placement, moved_to);
                let case = format!("{policy:?}, {placement:?}, the apex at {moved_to:?}");
                match model.validate(&policy, &mut ProvenanceLog::new()) {
                    Err(Error::Invalid(Invalid::VertexOffItsFacePlane { distance: found })) => {
                        assert!(
                            (found - distance).abs() <= 1e-12,
                            "{case}: {found} off a plane, not {distance}"
                        );
                    }
                    other => panic!("{case} validates as {other:?}"),
                }
            }
            let (sound, _) = the_pyramid_read_back(placement, WHERE_THE_PYRAMID_PUTS_ITS_APEX);
            sound
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|error| {
                    panic!(
                        "{policy:?}, {placement:?}: the sound pyramid does not validate: {error:?}"
                    )
                });
        }
    }
}

/// The control for the pyramid's refusals, as for the box's: the same bytes
/// with the apex left alone answer every partner but one, and the answer holds
/// what the pyramid and the block hold. The one left out is the cut through
/// the apex, which answers cleanly only where the pyramid stands, below.
#[test]
fn the_sound_pyramid_answers_every_partner_but_a_cut_through_its_apex_with_what_the_two_hold() {
    let not_through_the_apex: Vec<(Block, Rule)> = EVERY_PARTNER
        .into_iter()
        .filter(|&(partner, _)| partner != THE_HALF_PAST_THE_SEAM)
        .collect();
    assert_eq!(not_through_the_apex.len(), 9);
    answers_with_what_the_two_hold(
        |placement| the_pyramid_read_back(placement, WHERE_THE_PYRAMID_PUTS_ITS_APEX),
        &not_through_the_apex,
        what_the_pyramid_and_a_block_hold,
    );
}

/// **The sound pyramid cut through its apex where it stands answers half the
/// pyramid, under both policies** (§294.19). `THE_HALF_PAST_THE_SEAM`'s face
/// `y = 0` runs exactly through the apex, and each three of the four side
/// planes meet exactly on it. The box bake solved round the apex reaches
/// `4.2e−16` either side of the face, so the fuse asks the three planes bake
/// solved the apex from, and they put it on the face. Asking the box alone,
/// `certified` refused "side of plane".
#[test]
fn a_sound_pyramid_cut_through_its_apex_where_it_stands_answers_half_of_it() {
    let [identity, ..] = every_placement();
    let half = (2.0 * SIDE) * (2.0 * SIDE) * HEIGHT / 3.0 / 2.0;
    for rule in [Rule::Difference, Rule::Intersection] {
        assert_eq!(
            what_the_pyramid_and_a_block_hold(THE_HALF_PAST_THE_SEAM, rule),
            (1, half)
        );
        for policy in [Policy::certified(), Policy::tolerant(100.0)] {
            fuses_into_what_the_two_hold(
                the_pyramid_read_back(identity, WHERE_THE_PYRAMID_PUTS_ITS_APEX),
                identity,
                (THE_HALF_PAST_THE_SEAM, rule),
                &policy,
                what_the_pyramid_and_a_block_hold,
            );
        }
    }
}

/// **Placed, the sound pyramid cut through its apex keeps a speck of the cut
/// in what it builds**, which is a defect and not the moved apex's refusal
/// (§294.19 to §294.21). Shifted, turned, or turned and then shifted, the four
/// side planes as written meet three by three at points a rounding apart, and
/// the three bake solved the apex from put it a rounding off the cut. The four
/// side planes cross the cut's own face in four lines that pass a rounding
/// from one another by the apex, and cut it into specks a rounding across.
/// Each speck is now placed, from the planes, as the three bake solved each
/// corner of the pyramid from (Q1936), and what is built is what those numbers
/// hold: half of a pyramid whose apex is a rounding off the cut. The half
/// without the apex is 5 faces and 18 sides, its cut face keeping an edge a
/// rounding long under the apex. The half with it is 6 faces and 22 sides,
/// the sixth a triangle a rounding across. Where the pyramid stands, each half
/// is 5 faces and 16 sides.
///
/// - Under `certified` every fuse refuses the short edge as too short beside
///   the coordinates, `DynamicRangeTooWide`, and leaves the model as it was.
/// - Under `tolerant(100)` four fuses answer half the pyramid, 60, in one
///   solid that validates, and the log's one record is the range check that
///   took the short edge. In the other two the half holding the apex is
///   built, and its speck face has three corners within `5e−15` of one
///   another, which wind against its normal: on one line, shifted for the
///   intersection, and the wrong way round, turned and shifted for the
///   difference. The fuse refuses that and writes nothing.
///
/// Before Q1936, all twelve refused `Undecidable`, "a point inside a piece of a
/// face", on one of the specks.
#[test]
fn a_sound_pyramid_placed_and_cut_through_its_apex_keeps_a_speck_of_the_cut() {
    let [_, shift, turn, turn_then_shift] = every_placement();
    let [certified, tolerant] = [Policy::certified(), Policy::tolerant(100.0)];
    let without_the_apex = Some(FacesAndSides {
        faces: 5,
        sides: 18,
    });
    let with_the_apex = Some(FacesAndSides {
        faces: 6,
        sides: 22,
    });
    let wound_against_its_normal = None;
    let what_tolerant_builds = [
        (Rule::Difference, shift, without_the_apex),
        (Rule::Difference, turn, with_the_apex),
        (Rule::Difference, turn_then_shift, wound_against_its_normal),
        (Rule::Intersection, shift, wound_against_its_normal),
        (Rule::Intersection, turn, without_the_apex),
        (Rule::Intersection, turn_then_shift, without_the_apex),
    ];
    for (rule, placement, built) in what_tolerant_builds {
        let case = format!("{placement:?}, {rule:?}");
        match the_cut_through_the_apex(placement, rule, &certified) {
            Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => {
                assert!(ratio > 1e15, "certified, {case}: a range of {ratio}");
            }
            other => panic!("certified, {case} came back {other:?}"),
        }
        match (the_cut_through_the_apex(placement, rule, &tolerant), built) {
            (Ok((named, shape)), Some(expected)) => {
                assert_eq!(named, [DYNAMIC_RANGE_CHECK], "tolerant, {case}");
                assert_eq!(shape, expected, "tolerant, {case}");
            }
            (Err(Error::TheFuseWroteABrokenAnswer { broken }), None) => {
                assert_eq!(
                    broken,
                    Invalid::OuterWireWindsAgainstTheNormal,
                    "tolerant, {case}"
                );
            }
            (other, _) => panic!("tolerant, {case} came back {other:?}"),
        }
    }
}

/// **Cut where they were built and placed after, the sound pyramid and the
/// half bar answer at every placement.** Each is fused with
/// `THE_HALF_PAST_THE_SEAM` before the movement the two share, and the answer
/// is moved after, so no rounding of the movement reaches the fuse. Under
/// `certified` every answer validates, holds what the two hold, and leaves
/// nothing in the log, and each half pyramid is 5 faces and 16 sides, as where
/// it stands. Placed first, the pyramid refuses (above) and the half bar
/// refuses when turned (below). The kernel keeps one placement per shell and
/// is not told which movement two solids share, so fusing first is the
/// caller's to choose (§448.7).
#[test]
fn a_cut_made_where_the_solids_were_built_and_then_placed_answers_at_every_placement() {
    let certified = Policy::certified();
    for placement in every_placement() {
        for rule in [Rule::Difference, Rule::Intersection] {
            let case = format!("the pyramid, {placement:?}, {rule:?}");
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let pyramid = a_pyramid(&mut model);
            let block = a_block(&mut model, THE_HALF_PAST_THE_SEAM, &Isometry::identity());
            let answer = fused_then_placed(&mut model, (pyramid, block), rule, placement, &case);
            the_answer_holds_what_the_two_hold(
                &model,
                &answer,
                (THE_HALF_PAST_THE_SEAM, rule),
                &certified,
                what_the_pyramid_and_a_block_hold,
                &case,
            );
            assert_eq!(
                the_faces_and_sides_of(&model, &answer),
                FacesAndSides {
                    faces: 5,
                    sides: 16
                },
                "{case}"
            );

            let case = format!("the half bar, {placement:?}, {rule:?}");
            let (mut model, half_bar) =
                the_half_bar_read_back(Isometry::identity(), WHERE_THE_HALF_BAR_PUTS_IT);
            let block = a_block(&mut model, THE_HALF_PAST_THE_SEAM, &Isometry::identity());
            let answer = fused_then_placed(&mut model, (half_bar, block), rule, placement, &case);
            the_answer_holds_what_the_two_hold(
                &model,
                &answer,
                (THE_HALF_PAST_THE_SEAM, rule),
                &certified,
                what_the_half_bar_and_a_block_hold,
                &case,
            );
        }
    }
}

/// **Where the pyramid stands, a wedge whose edge runs up through its apex
/// cuts it cleanly, and so does one cut there and placed after.** Every wedge
/// holds an eighth of the pyramid, 15, and every rule answers what the two
/// hold, under both policies where it stands and under `certified` with
/// nothing in the log cut first and placed after, at the eight placements
/// where the fuse refuses below. These are the controls for that refusal:
/// what refuses there is the placement, not the wedge or the rule.
#[test]
fn a_sound_pyramid_cut_by_a_wedge_through_its_apex_answers_where_it_stands_and_placed_after() {
    let every_rule = [Rule::Difference, Rule::Intersection, Rule::Union];
    for wedge in EVERY_WEDGE {
        for rule in every_rule {
            for policy in [Policy::certified(), Policy::tolerant(100.0)] {
                let case = format!("{policy:?}, {wedge:?} {rule:?}, where it stands");
                if let Err(refused) =
                    the_wedge_through_the_apex(wedge, Isometry::identity(), rule, &policy)
                {
                    panic!("{case} refused: {refused:?}");
                }
            }
        }
    }
    for (wedge, placement) in where_the_orders_loop_by_the_apex() {
        for rule in every_rule {
            let case = format!("{placement:?}, {wedge:?} {rule:?}, cut first");
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let pyramid = a_pyramid(&mut model);
            let cut = a_wedge(&mut model, wedge, &Isometry::identity());
            let answer = fused_then_placed(&mut model, (pyramid, cut), rule, placement, &case);
            the_answer_holds(
                &model,
                &answer,
                what_the_pyramid_and_a_wedge_hold(wedge, rule),
                &Policy::certified(),
                &case,
            );
        }
    }
}

/// **A wedge whose profile runs back to the origin along a tilted side cuts
/// the sound pyramid where it stands under `certified`, with nothing in the
/// log.** The side from `(5, 3.75)` back to `(0, 0)` stands on the plane `y =
/// 0.75·x`, and an extrusion writes a side's plane through whichever of the
/// side's two corners is nearer the origin. Written through `(5, 3.75)`, with
/// a normal no `f64` holds exactly, its offset came out `-2.78e-16` and not
/// `0`, the wedge's edge missed the pyramid's apex by that much, and every
/// rule was refused `DynamicRangeTooWide` (Q2320, Q2331). Through the origin
/// the offset is `0`, the six planes meet at the apex exactly, and the
/// answers are the pyramid's 120 less the 7.5 the two share, that 7.5, and
/// the pyramid's 120 and the wedge's 75 less it.
#[test]
fn a_sound_pyramid_cut_by_a_wedge_with_a_tilted_side_back_to_its_edge_answers_where_it_stands() {
    let wedge = A_WEDGE_ON_ONE_SIDE_OF_THE_X_AXIS;
    for (rule, volume) in [
        (Rule::Difference, 112.5),
        (Rule::Intersection, 7.5),
        (Rule::Union, 187.5),
    ] {
        let case = format!("{wedge:?} {rule:?}, where it stands");
        assert_eq!(
            what_the_pyramid_and_a_wedge_hold(wedge, rule),
            (1, volume),
            "{case}"
        );
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pyramid = a_pyramid(&mut model);
        let cut = a_wedge(&mut model, wedge, &Isometry::identity());
        let mut log = ProvenanceLog::new();
        let answer = model
            .fuse(&[pyramid], &[cut], rule, &Policy::certified(), &mut log)
            .unwrap_or_else(|refused| panic!("{case} refused: {refused:?}"));
        assert!(
            log.records().is_empty(),
            "{case}: a certified answer logs nothing"
        );
        the_answer_holds(&model, &answer, (1, volume), &Policy::certified(), &case);
    }
}

/// **A tilted wall that a prism and the hole it fills both stand on is one
/// plane, turned round, from either side.** The prism runs its outline
/// anticlockwise and the block its hole clockwise, so the two list the wall
/// from opposite ends. Its two corners, `(11, 7)` and `(1, 13)`, are as far as
/// each other from the origin, so it is the rule for two corners as near that
/// picks the one both planes go through, whichever is listed first. Through
/// `(11, 7)` the offset is `11.6619037896906` and through `(1, 13)` it is
/// `11.661903789690601`, an ulp apart, which is what each side's own first
/// corner gave it.
#[test]
fn a_tilted_wall_is_one_plane_whether_a_prism_or_the_hole_it_fills_stands_on_it() {
    let triangle = vec![[11.0, 7.0], [1.0, 13.0], [2.0, 2.0]];
    let square = vec![[-20.0, -20.0], [20.0, -20.0], [20.0, 20.0], [-20.0, 20.0]];
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let prism = ProfileBuilder::new(triangle.clone())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a triangle");
    let prism = model
        .add_extrusion(&prism, HEIGHT, &Isometry::identity())
        .expect("a prism");
    let block = ProfileBuilder::new(square)
        .with_hole(triangle)
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square with a triangle out of it");
    let block = model
        .add_extrusion(&block, HEIGHT, &Isometry::identity())
        .expect("a block with a hole");
    let the_wall_facing = |solid: SolidKey, sign: f64| {
        let walls: Vec<[f64; 4]> = model
            .solid(solid)
            .expect("this model made the key")
            .faces()
            .filter_map(|face| face.surface().as_plane().map(Plane::coefficients))
            .filter(|&[x, y, _, _]| sign * x > 0.0 && sign * y > 0.0)
            .collect();
        let &[wall] = walls.as_slice() else {
            panic!("one wall faces {sign} in x and y, got {walls:?}");
        };
        wall
    };
    let from_the_prism = the_wall_facing(prism, 1.0);
    let from_the_hole = the_wall_facing(block, -1.0);
    assert_eq!(
        from_the_hole,
        from_the_prism.map(|coefficient| -coefficient)
    );
}

/// **Placed, the sound pyramid cut by a wedge whose edge runs up through its
/// apex cannot tell the cells of a face apart**, by every rule and under both
/// policies, and each refusal leaves the model as it was. Q1937 asked whether
/// `as_the_planes_say` in `fuse/arrange.rs` orders two cuts against a corner a
/// rounding off a crossing's line the way it orders the cuts against each
/// other. These are the eight placements, of the 112 the lane swept, where it
/// does not. The crossing is a side face's edge, the corner off it is the
/// apex, and the two cuts are where the wedge's two faces cross that edge. The
/// apex is put in order against each cut by that cut's own wedge face, the
/// boxes or the planes order the two cuts against each other, and the three
/// answers go round in a loop.
///
/// The loop does not make the refusal. A change to `as_the_planes_say` that
/// let the apex order the two cuts it falls between, which ends the loop in
/// the five of the eight where the planes ordered the cuts, changed none of the
/// 672 fuses the lane swept, these 48 among them. The other three are ordered
/// by the boxes, which `as_the_planes_say` is never asked. And 42 more
/// placements with no loop refuse the same way. What does make it is the
/// apex itself, a rounding off one of the lines it is put on, with a wedge
/// plane running between it and where that line meets another (the test
/// below). No change to `fuse/` comes with this pin.
#[test]
fn a_sound_pyramid_placed_and_cut_by_a_wedge_through_its_apex_cannot_tell_its_cells_apart() {
    for (wedge, placement) in where_the_orders_loop_by_the_apex() {
        for rule in [Rule::Difference, Rule::Intersection, Rule::Union] {
            for policy in [Policy::certified(), Policy::tolerant(100.0)] {
                let case = format!("{policy:?}, {placement:?}, {wedge:?} {rule:?}");
                match the_wedge_through_the_apex(wedge, placement, rule, &policy) {
                    Err(Error::CannotTellTheCellsOfAFaceApart { .. }) => {}
                    other => panic!("{case} came back {other:?}"),
                }
            }
        }
    }
}

/// **Placed, the pyramid has four apexes a rounding apart, and it cannot tell
/// a face's cells apart only where a wedge plane runs between two of them.**
/// Each three of the four side planes meet in an apex of their own. The fuse
/// makes one corner of them, worked out from three of the planes, and on each
/// side face puts that corner on every line where another side plane crosses
/// the face. So on each face the corner is on two of those lines exactly and
/// off the third by a rounding.
///
/// On the `−y` face at the plain shift, with the wedge over the `+y` edge, a
/// print put in `order_along` in `fuse/arrange.rs` and taken out found the
/// corner worked out from the `−y`, `+x` and `+y` planes, on the `+x` and `+y`
/// lines and off the `−x` one, the face's other edge. The wedge's plane `y =
/// 2x` runs between that corner and the apex where the `−x` and `+x` lines
/// meet. Call the lines through the corner `a` and `b`, the one off it `c`,
/// and the wedge's line `r`, which crosses `a` between the corner and `c`.
/// Where those four cross one another is put in order along each line as the
/// real points lie, but for where the corner falls on `c`. The orders along
/// `a`, `b` and `r` put `r` on one side of the corner. A straight `c` through
/// the corner then crosses `r` on one side of where `a` does, while the order
/// along `r` has the real `c` cross it on the other. No drawing of the face
/// with straight lines and the corner on all three keeps every order, and the
/// fuse refuses as above. The lane that found this let the planes overrule
/// the boxes on that face, and then turned round two cuts along `r` as well:
/// each still refused, the first on two windings that disagree.
///
/// Of the 39 fuses here, `Difference` and `certified` with each wedge at each
/// of 13 placements, 23 are refused so, and each of those has a wedge plane
/// between two apexes. 4 more have one and are refused at the range check
/// instead; why the orders' quarrel does not show there is not traced. 3 of
/// those 4 were refused `Undecidable` until `by_a_plane` in `fuse/select.rs`
/// read a corner of the input with four planes round it where the lines of
/// the piece's two sides there cross (Q2334): the wedge over the `+y` edge at
/// `every_placement`'s turn about `x` shifted, and twice over at the shift
/// after the turn 0.1 about `(1, 2, 3)`. 12 have none, and none of those is
/// refused so.
///
/// So the refusal is the gap `limit_or_gap` names for it: the cells are there,
/// and it is the fuse's reading of the corner that contradicts itself. What a
/// face drawn whole would come to under `certified`, the range check §448.7
/// settled for the cut through the apex or something else, nobody has run.
/// Drawing it takes one reading of the corner that every face round it
/// shares, which is a row of its own.
#[test]
fn a_placed_pyramid_cannot_tell_its_cells_apart_only_where_a_wedge_plane_runs_between_two_apexes() {
    let placements = every_placement()
        .into_iter()
        .chain(where_the_orders_loop_by_the_apex().map(|(_, placement)| placement))
        .chain([a_tenth_of_a_radian_about_a_tilted_axis()]);
    let mut refused_with_a_plane_between = 0;
    let mut a_plane_between_and_refused_at_the_range_check = 0;
    let mut no_plane_between = 0;
    for placement in placements {
        for wedge in EVERY_WEDGE {
            let between = a_wedge_plane_runs_between_two_apexes(wedge, placement);
            let fused = the_wedge_through_the_apex(
                wedge,
                placement,
                Rule::Difference,
                &Policy::certified(),
            );
            let refused = matches!(fused, Err(Error::CannotTellTheCellsOfAFaceApart { .. }));
            match (between, refused) {
                (true, true) => refused_with_a_plane_between += 1,
                (true, false) => {
                    assert!(
                        matches!(
                            fused,
                            Err(Error::Invalid(Invalid::DynamicRangeTooWide { .. }))
                        ),
                        "{placement:?}, {wedge:?}: a wedge plane between two apexes came back \
                         {fused:?}"
                    );
                    a_plane_between_and_refused_at_the_range_check += 1;
                }
                (false, false) => no_plane_between += 1,
                (false, true) => panic!(
                    "{placement:?}, {wedge:?}: refused with no wedge plane between two apexes"
                ),
            }
        }
    }
    assert_eq!(
        (
            refused_with_a_plane_between,
            a_plane_between_and_refused_at_the_range_check,
            no_plane_between
        ),
        (23, 4, 12)
    );
}

/// **Shifted to `(7, −5, 2.5)`, the sound pyramid cut by the wedge over its
/// `+y` edge built there is refused at the range check under `certified`**,
/// the limit §448.7 settled for the cut through the apex, and leaves the
/// model as it was. It was refused `CannotTellTheCellsOfAFaceApart`: on the
/// `−y` face the fuse put the corner it solved from three side planes on
/// every side plane's line, a rounding off one of them, with the wedge plane
/// `y = 2x` running between that corner and where the other two lines meet.
#[test]
fn a_shifted_pyramid_cut_by_a_wedge_over_its_plus_y_edge_is_refused_at_the_range_check() {
    let shift = every_placement()[1];
    let case = format!("{shift:?}, {A_WEDGE_OVER_THE_PLUS_Y_EDGE:?}");
    match the_wedge_through_the_apex(
        A_WEDGE_OVER_THE_PLUS_Y_EDGE,
        shift,
        Rule::Difference,
        &Policy::certified(),
    ) {
        Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => {
            assert!(ratio > 1e15, "{case}: a range of {ratio}");
        }
        other => panic!("{case} came back {other:?}"),
    }
}

/// **Turned a tenth of a radian about `(0.3, −1, 2)`, the sound pyramid and
/// the wedge round the `x` axis built there are refused for an edge too short
/// by every rule under `certified`**, and leave the model as it was. Turned,
/// the wedge's two planes and the pyramid's four side planes miss one point by
/// a rounding, so the answer of those numbers keeps an edge a rounding long by
/// the apex: the limit §294.21 and §448.7 settled for the cut through the
/// apex. The difference and the intersection already refused so. The union
/// panicked in `edges_after_joining`, "a piece of a face has a straight side
/// with no length" (queue rows Q2317 and Q2332). One wedge face's outline went
/// out along the wedge's edge through two corners a rounding off the apex and
/// came back before it went on, and the other wedge face walked those three
/// sides back inside its own outline, so the three folded into one straight
/// edge from a corner back to itself. The fuse leaves out such a stretch of an
/// outline, as it leaves out a whole ring another plane walks back.
#[test]
fn a_sound_pyramid_turned_and_cut_by_a_wedge_round_the_x_axis_is_refused_for_an_edge_too_short() {
    for rule in [Rule::Difference, Rule::Intersection, Rule::Union] {
        let case = format!("{A_WEDGE_ROUND_THE_X_AXIS:?} {rule:?}");
        match the_wedge_through_the_apex(
            A_WEDGE_ROUND_THE_X_AXIS,
            a_tenth_of_a_radian_about_a_tilted_axis(),
            rule,
            &Policy::certified(),
        ) {
            Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => {
                assert!(ratio > 1e15, "{case}: a range of {ratio}");
            }
            other => panic!("{case} came back {other:?}"),
        }
    }
}

/// **Under `tolerant(100)` the same turned pyramid and wedge answer the
/// difference and the union**, the pyramid's 120 less the 15 the two share,
/// and the pyramid's 120 and the wedge's 150 less it, each with the range
/// check `tolerant` takes in the log. The union panicked here too (queue row
/// Q2332). **The intersection is refused `TheFuseWroteABrokenAnswer`** with
/// its outline winding against its normal, and leaves the model as it was. A
/// print put in `validate::check_flat_face` and taken out found the face: a
/// triangle on a side of the pyramid whose three corners are within `4e−15`
/// of one another by the apex, which winds `Neg`. That is the triangle a
/// rounding across that §294.21 found on the placed cut through the apex,
/// which no winding of its stored corners can carry: the same limit, one step
/// on from the range check (§448.7).
#[test]
fn a_sound_pyramid_turned_and_cut_by_a_wedge_round_the_x_axis_answers_tolerant_but_for_the_intersection()
 {
    for (rule, answers) in [
        (Rule::Difference, true),
        (Rule::Intersection, false),
        (Rule::Union, true),
    ] {
        let case = format!("{A_WEDGE_ROUND_THE_X_AXIS:?} {rule:?}");
        match (
            the_wedge_through_the_apex(
                A_WEDGE_ROUND_THE_X_AXIS,
                a_tenth_of_a_radian_about_a_tilted_axis(),
                rule,
                &Policy::tolerant(100.0),
            ),
            answers,
        ) {
            (Ok(named), true) => assert_eq!(named, [DYNAMIC_RANGE_CHECK], "{case}"),
            (Err(Error::TheFuseWroteABrokenAnswer { broken }), false) => {
                assert_eq!(broken, Invalid::OuterWireWindsAgainstTheNormal, "{case}");
            }
            (other, _) => panic!("{case} came back {other:?}"),
        }
    }
}

/// **Turned 0.4 about `x`, and that turn shifted, the sound pyramid and the
/// wedge over the `+y` edge built there reach the range check: `certified`
/// refuses every rule `DynamicRangeTooWide`, and `tolerant(100)` answers every
/// rule** with that check in the log, each answer what the two hold. Every one
/// was refused `Undecidable`, "a point inside a piece of a face" (queue rows
/// Q2321 and Q2334). Turned, the wedge's face `y = 2x` cuts the pyramid's `−x`
/// face into a triangle a rounding across by the apex. The fuse solved the
/// apex from the `−y`, `+x` and `+y` planes, so on the `−x` face it is a
/// rounding off the face's plane, no point was found inside the triangle, and
/// `by_a_plane` in `fuse/select.rs` read a corner of the input from the planes
/// round it only where they are exactly three. It now reads the triangle's
/// corner there as where the lines of its two sides cross, the `−x` face's own
/// top corner, and finds all three corners in front of the wedge's other face
/// `y = 0.5x`, so outside the wedge. The range the answer then spans is the
/// limit §294.21 and §448.7 settled for the cut through the apex.
#[test]
fn a_sound_pyramid_turned_about_x_and_cut_by_a_wedge_over_the_plus_y_edge_reaches_the_range_check()
{
    let wedge = A_WEDGE_OVER_THE_PLUS_Y_EDGE;
    let [_, _, turned, turned_and_shifted] = every_placement();
    for placement in [turned, turned_and_shifted] {
        for rule in [Rule::Difference, Rule::Intersection, Rule::Union] {
            let case = format!("{placement:?}, {wedge:?} {rule:?}");
            match the_wedge_through_the_apex(wedge, placement, rule, &Policy::certified()) {
                Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => {
                    assert!(ratio > 1e15, "{case}: a range of {ratio}");
                }
                other => panic!("{case} came back {other:?} under certified"),
            }
            match the_wedge_through_the_apex(wedge, placement, rule, &Policy::tolerant(100.0)) {
                Ok(named) => assert_eq!(named, [DYNAMIC_RANGE_CHECK], "{case}"),
                other => panic!("{case} came back {other:?} under tolerant(100)"),
            }
        }
    }
}

/// **Shifted after the turn 0.1 about `(1, 2, 3)`, or after the turn 0.7
/// about `(0.3, −1, 2)`, the sound pyramid and the wedge over the `+y` edge
/// built there are refused at the range check by every rule under
/// `certified`.** Queue row Q2576 found the difference refused `Undecidable`
/// at each. With `by_a_plane` reading a corner of the input with four planes
/// round it where the lines of the piece's two sides there cross (Q2334), each
/// rule comes to the limit §448.7 settled for the cut through the apex, as at
/// the turn about `x` in the test above.
#[test]
fn a_sound_pyramid_shifted_after_a_turn_and_cut_by_a_wedge_over_the_plus_y_edge_is_refused_at_the_range_check()
 {
    let wedge = A_WEDGE_OVER_THE_PLUS_Y_EDGE;
    let shift = Isometry::translation(Vector::new(7.0, -5.0, 2.5)).expect("a plain translation");
    let turn = |[x, y, z]: [f64; 3], angle: f64| {
        let axis = Axis::new(Point::new(0.0, 0.0, 0.0), Vector::new(x, y, z)).expect("an axis");
        Isometry::rotation(&axis, angle).expect("a turn")
    };
    for placement in [
        shift.compose(&turn([1.0, 2.0, 3.0], 0.1)),
        shift.compose(&turn([0.3, -1.0, 2.0], 0.7)),
    ] {
        for rule in [Rule::Difference, Rule::Intersection, Rule::Union] {
            let case = format!("{placement:?}, {wedge:?} {rule:?}");
            match the_wedge_through_the_apex(wedge, placement, rule, &Policy::certified()) {
                Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio })) => {
                    assert!(ratio > 1e15, "{case}: a range of {ratio}");
                }
                other => panic!("{case} came back {other:?}"),
            }
        }
    }
}

/// **Every partner, every rule, every placement and both policies give the
/// box's refusal** for every move of the apex, the cut through the apex among
/// them. Each three of the four side planes cross at the apex alone, so no
/// three of them place an apex a millimetre away, as no three placed the box's
/// moved corner.
#[test]
fn every_fuse_of_a_pyramid_whose_apex_is_moved_refuses_where_three_surfaces_meet() {
    every_fuse_refuses_where_three_surfaces_meet(the_pyramid_read_back, EVERY_MOVE_OF_THE_APEX);
}

/// Each of the four moves takes the corner a millimetre off at least one of
/// the three surfaces round it, and `validate` says so under both policies at
/// every placement. It names a curved surface even for the move off the cap's
/// plane: the fuse that made the half bar wrote each edge at the corner as
/// where two surfaces cross, and `validate` asks the ends of such an edge
/// about the second surface before it asks a face about its own.
///
/// Moved onto the other end of the cut, the corner is still on all three
/// surfaces, and what `validate` finds is the cut's straight edge across the
/// top, whose two ends are now one point. The same bytes with the corner left
/// alone validate.
#[test]
fn a_half_bar_whose_corner_is_moved_in_its_file_reads_back_and_does_not_validate() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for placement in every_placement() {
            let each_move = EVERY_MOVE_OF_THE_HALF_BAR
                .map(|moved_to| (moved_to, Invalid::VertexOffItsFaceSurface))
                .into_iter()
                .chain([(THE_OTHER_END_OF_THE_CUT, Invalid::EdgeHasNoLength)]);
            for (moved_to, invalid) in each_move {
                let (model, _) = the_half_bar_read_back(placement, moved_to);
                let case = format!("{policy:?}, {placement:?}, the corner at {moved_to:?}");
                match model.validate(&policy, &mut ProvenanceLog::new()) {
                    Err(Error::Invalid(found)) => assert_eq!(found, invalid, "{case}"),
                    other => panic!("{case} validates as {other:?}"),
                }
            }
            let (sound, _) = the_half_bar_read_back(placement, WHERE_THE_HALF_BAR_PUTS_IT);
            sound
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|error| {
                    panic!(
                        "{policy:?}, {placement:?}: the sound half bar does not validate: {error:?}"
                    )
                });
        }
    }
}

/// The control for the half bar's refusals, as for the box's and the
/// pyramid's: the same bytes with the corner left alone answer every partner
/// but one, and the answer holds what the half bar and the block hold, to the
/// bound the measure publishes for a curved solid. The one left out is the cut
/// along the half bar's flat face, which answers too, on its own below.
#[test]
fn the_sound_half_bar_answers_every_partner_but_a_cut_along_its_flat_face_with_what_the_two_hold() {
    let not_along_the_flat_face: Vec<(Block, Rule)> = EVERY_PARTNER
        .into_iter()
        .filter(|&(partner, _)| partner != THE_HALF_PAST_THE_SEAM)
        .collect();
    assert_eq!(not_along_the_flat_face.len(), 9);
    answers_with_what_the_two_hold(
        |placement| the_half_bar_read_back(placement, WHERE_THE_HALF_BAR_PUTS_IT),
        &not_along_the_flat_face,
        what_the_half_bar_and_a_block_hold,
    );
}

/// **The sound half bar cut again by the block that made it answers at every
/// placement, under both policies.** `THE_HALF_PAST_THE_SEAM`'s face `y = 0`
/// lies on the half bar's flat face, and the answers are the whole half bar for
/// the difference and nothing for the intersection. Turned then shifted, the
/// block's face lands one ulp past the flat face, on the side away from the
/// bar, and every fuse was refused, `certified` on the side of a plane and
/// `tolerant(100)` on two curves on a face that touch without crossing (Q2043,
/// §294.24).
#[test]
fn a_sound_half_bar_cut_flush_along_its_flat_face_answers_in_every_placement() {
    let (solids, volume) =
        what_the_half_bar_and_a_block_hold(THE_HALF_PAST_THE_SEAM, Rule::Difference);
    assert_eq!(solids, 1);
    assert!((volume - 45.0 * PI).abs() <= 1e-12 * volume, "{volume}");
    assert_eq!(
        what_the_half_bar_and_a_block_hold(THE_HALF_PAST_THE_SEAM, Rule::Intersection),
        (0, 0.0)
    );
    answers_with_what_the_two_hold(
        |placement| the_half_bar_read_back(placement, WHERE_THE_HALF_BAR_PUTS_IT),
        &[
            (THE_HALF_PAST_THE_SEAM, Rule::Difference),
            (THE_HALF_PAST_THE_SEAM, Rule::Intersection),
        ],
        what_the_half_bar_and_a_block_hold,
    );
}

/// **Every millimetre move of the half bar's corner gets the box's refusal**,
/// at every partner, rule, placement and policy, though one of the three
/// surfaces round the corner is curved and they meet at two places. The
/// stored corner is further from both than `validate` lets a corner sit off
/// its surfaces, so neither is picked and nothing else places the corner.
#[test]
fn every_fuse_of_a_half_bar_whose_corner_is_moved_a_millimetre_refuses_where_three_surfaces_meet() {
    every_fuse_refuses_where_three_surfaces_meet(
        the_half_bar_read_back,
        EVERY_MOVE_OF_THE_HALF_BAR,
    );
}

/// **The corner moved onto the other place its three surfaces meet is refused
/// by every fuse**, and none comes back a wrong `Ok`, which is what the move
/// risked: the stored corner now picks the other end of the cut, where the half
/// bar keeps a corner of its own, so the fuse finds two corners of one solid
/// at one place. Under `tolerant(100)` at every placement, and under
/// `certified` with no placement and shifted, it says so as `validate` does,
/// `EdgeHasNoLength`. Under `certified` turned it cannot order two points
/// along a crossing, and turned and shifted the same. Each refusal leaves the
/// model as it was.
#[test]
fn a_half_bar_whose_corner_is_moved_onto_the_other_end_of_its_cut_answers_no_fuse() {
    let [identity, shift, turn, turn_then_shift] = every_placement();
    let [certified, tolerant] = [Policy::certified(), Policy::tolerant(100.0)];
    for (partner, rule) in EVERY_PARTNER {
        let refused = [
            (
                identity,
                &tolerant,
                Refused::Invalid(Invalid::EdgeHasNoLength),
            ),
            (shift, &tolerant, Refused::Invalid(Invalid::EdgeHasNoLength)),
            (turn, &tolerant, Refused::Invalid(Invalid::EdgeHasNoLength)),
            (
                turn_then_shift,
                &tolerant,
                Refused::Invalid(Invalid::EdgeHasNoLength),
            ),
            (
                identity,
                &certified,
                Refused::Invalid(Invalid::EdgeHasNoLength),
            ),
            (
                shift,
                &certified,
                Refused::Invalid(Invalid::EdgeHasNoLength),
            ),
            (
                turn,
                &certified,
                Refused::Undecidable(vec![ALONG_A_CROSSING]),
            ),
            (
                turn_then_shift,
                &certified,
                Refused::Undecidable(vec![ALONG_A_CROSSING]),
            ),
        ];
        for (placement, policy, refusal) in refused {
            assert_eq!(
                the_half_bar_refuses(THE_OTHER_END_OF_THE_CUT, placement, (partner, rule), policy),
                refusal,
                "{policy:?}, {placement:?}, {partner:?} {rule:?}"
            );
        }
    }
}

/// The sound solid from `the_sound_solid` at each placement, fused with each
/// of `partners` under both policies, answers what `what_the_two_hold` says.
fn answers_with_what_the_two_hold(
    the_sound_solid: impl Fn(Isometry) -> (Model, SolidKey),
    partners: &[(Block, Rule)],
    what_the_two_hold: fn(Block, Rule) -> (usize, f64),
) {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for placement in every_placement() {
            for &partner in partners {
                fuses_into_what_the_two_hold(
                    the_sound_solid(placement),
                    placement,
                    partner,
                    &policy,
                    what_the_two_hold,
                );
            }
        }
    }
}

/// `sound`, a model and the solid in it, fused with the block `partner` names
/// at `placement` under its rule, answers, validates, and holds what
/// `what_the_two_hold` says: as many solids, and a volume within `1e-12`
/// relative and the bound the measure publishes.
fn fuses_into_what_the_two_hold(
    sound: (Model, SolidKey),
    placement: Isometry,
    (partner, rule): (Block, Rule),
    policy: &Policy,
    what_the_two_hold: fn(Block, Rule) -> (usize, f64),
) {
    let case = format!("{policy:?}, {placement:?}, {partner:?} {rule:?}");
    let (mut model, solid) = sound;
    let block = a_block(&mut model, partner, &placement);
    let answer = model
        .fuse(&[solid], &[block], rule, policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|error| panic!("{case} refused: {error:?}"));
    the_answer_holds_what_the_two_hold(
        &model,
        &answer,
        (partner, rule),
        policy,
        what_the_two_hold,
        &case,
    );
}

/// `answer`, in `model`, validates and holds what `what_the_two_hold` says the
/// solid and the block `partner` names hold under its rule: as many solids,
/// and a volume within `1e-12` relative and the bound the measure publishes.
fn the_answer_holds_what_the_two_hold(
    model: &Model,
    answer: &Fused,
    (partner, rule): (Block, Rule),
    policy: &Policy,
    what_the_two_hold: fn(Block, Rule) -> (usize, f64),
    case: &str,
) {
    the_answer_holds(
        model,
        answer,
        what_the_two_hold(partner, rule),
        policy,
        case,
    );
}

/// `answer`, in `model`, validates and holds `solids` solids and `volume`, to
/// `1e-12` relative and the bound the measure publishes.
fn the_answer_holds(
    model: &Model,
    answer: &Fused,
    (solids, volume): (usize, f64),
    policy: &Policy,
    case: &str,
) {
    model
        .validate(policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|error| panic!("{case}: the answer does not validate: {error:?}"));

    assert_eq!(answer.solids().len(), solids, "{case}: how many solids");
    let (measured, bound) = the_volume_of(model, answer, policy, case);
    assert!(
        (measured - volume).abs() <= 1e-12 * volume + bound,
        "{case}: the answer holds {measured} to {bound}, the two {volume}"
    );
}

/// The sound pyramid at `placement` fused with `THE_HALF_PAST_THE_SEAM` under
/// `rule`. An answer has to validate and hold what the pyramid and the block
/// hold, and comes back as the predicates the fuse's log names and how many
/// faces and sides it has. A refusal has to leave the model byte for byte as it
/// was, and comes back as it is.
fn the_cut_through_the_apex(
    placement: Isometry,
    rule: Rule,
    policy: &Policy,
) -> Result<(Vec<&'static str>, FacesAndSides), Error> {
    let case = format!("{policy:?}, {placement:?}, {rule:?}");
    let (mut model, solid) = the_pyramid_read_back(placement, WHERE_THE_PYRAMID_PUTS_ITS_APEX);
    let block = a_block(&mut model, THE_HALF_PAST_THE_SEAM, &placement);
    let before = postcard::to_allocvec(&model).expect("a model writes out");
    let mut log = ProvenanceLog::new();
    match model.fuse(&[solid], &[block], rule, policy, &mut log) {
        Ok(answer) => {
            the_answer_holds_what_the_two_hold(
                &model,
                &answer,
                (THE_HALF_PAST_THE_SEAM, rule),
                policy,
                what_the_pyramid_and_a_block_hold,
                &case,
            );
            let named = log
                .records()
                .iter()
                .map(|record| record.predicate())
                .collect();
            Ok((named, the_faces_and_sides_of(&model, &answer)))
        }
        Err(refused) => {
            assert_eq!(
                postcard::to_allocvec(&model).expect("a model writes out"),
                before,
                "{case}: a refused fuse leaves the model as it was"
            );
            Err(refused)
        }
    }
}

/// The sound pyramid built where it stands and moved by `placement`, fused
/// under `rule` with `wedge` built at `placement`. An answer has to validate
/// and hold what the pyramid and the wedge hold, and comes back as the
/// predicates the fuse's log names. A refusal has to leave the model byte for
/// byte as it was, and comes back as it is.
fn the_wedge_through_the_apex(
    wedge: Wedge,
    placement: Isometry,
    rule: Rule,
    policy: &Policy,
) -> Result<Vec<&'static str>, Error> {
    let case = format!("{policy:?}, {placement:?}, {wedge:?} {rule:?}");
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let pyramid = a_pyramid(&mut model);
    model
        .place_solid(pyramid, &placement)
        .expect("this model made the key");
    let cut = a_wedge(&mut model, wedge, &placement);
    let before = postcard::to_allocvec(&model).expect("a model writes out");
    let mut log = ProvenanceLog::new();
    match model.fuse(&[pyramid], &[cut], rule, policy, &mut log) {
        Ok(answer) => {
            the_answer_holds(
                &model,
                &answer,
                what_the_pyramid_and_a_wedge_hold(wedge, rule),
                policy,
                &case,
            );
            Ok(log
                .records()
                .iter()
                .map(|record| record.predicate())
                .collect())
        }
        Err(refused) => {
            assert_eq!(
                postcard::to_allocvec(&model).expect("a model writes out"),
                before,
                "{case}: a refused fuse leaves the model as it was"
            );
            Err(refused)
        }
    }
}

/// The eight placements where Q1937 found the order along one crossing go
/// round in a loop by the apex, with the wedge each was found with. Each is a
/// turn about an axis through the origin, and four are shifted after it by
/// `every_placement`'s shift.
fn where_the_orders_loop_by_the_apex() -> [(Wedge, Isometry); 8] {
    let shift = Isometry::translation(Vector::new(7.0, -5.0, 2.5)).expect("a plain translation");
    let turn = |[x, y, z]: [f64; 3], angle: f64| {
        let axis = Axis::new(Point::new(0.0, 0.0, 0.0), Vector::new(x, y, z)).expect("an axis");
        Isometry::rotation(&axis, angle).expect("a turn")
    };
    [
        (
            A_WEDGE_ROUND_THE_X_AXIS,
            shift.compose(&turn([1.0, 2.0, 3.0], 0.1)),
        ),
        (A_WEDGE_ROUND_THE_X_AXIS, turn([1.0, 2.0, 3.0], 0.7)),
        (A_WEDGE_ROUND_THE_X_AXIS, turn([-2.0, 1.0, 0.5], 2.9)),
        (
            A_WEDGE_OVER_THE_MINUS_Y_EDGE,
            shift.compose(&turn([1.0, 2.0, 3.0], 0.1)),
        ),
        (
            A_WEDGE_OVER_THE_MINUS_Y_EDGE,
            shift.compose(&turn([0.3, -1.0, 2.0], 0.7)),
        ),
        (A_WEDGE_OVER_THE_PLUS_Y_EDGE, turn([1.0, 2.0, 3.0], 0.7)),
        (A_WEDGE_OVER_THE_PLUS_Y_EDGE, turn([-2.0, 1.0, 0.5], 0.7)),
        (
            A_WEDGE_OVER_THE_PLUS_Y_EDGE,
            shift.compose(&turn([-2.0, 1.0, 0.5], 1.3)),
        ),
    ]
}

/// The turn under which the wedge round the `x` axis keeps three corners a
/// rounding apart by the apex along its edge (queue rows Q2317 and Q2332).
fn a_tenth_of_a_radian_about_a_tilted_axis() -> Isometry {
    let axis = Axis::new(Point::new(0.0, 0.0, 0.0), Vector::new(0.3, -1.0, 2.0)).expect("an axis");
    Isometry::rotation(&axis, 0.1).expect("a turn")
}

/// Whether one of the two planes of `wedge` through the apex, built at
/// `placement`, has two of the sound pyramid's four apexes, moved by
/// `placement`, on opposite sides of it, each side worked out exactly. The
/// four apexes are where each three of the four side planes meet. Where the
/// pyramid stands they are one point and both planes run through it.
fn a_wedge_plane_runs_between_two_apexes(wedge: Wedge, placement: Isometry) -> bool {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let pyramid = a_pyramid(&mut model);
    model
        .place_solid(pyramid, &placement)
        .expect("this model made the key");
    let cut = a_wedge(&mut model, wedge, &placement);
    let [x, y, z] = placement
        .apply_to_point(Point::new(0.0, 0.0, HEIGHT))
        .coordinates();
    // The side planes and the wedge's two faces miss the moved apex by a
    // rounding; the base, the wedge's caps and its far face by millimetres.
    let through_the_apex = |solid: SolidKey| -> Vec<[f64; 4]> {
        model
            .solid(solid)
            .expect("this model made the key")
            .faces()
            .filter_map(|face| face.surface().as_plane().map(Plane::coefficients))
            .filter(|[a, b, c, d]| (a * x + b * y + c * z + d).abs() < 1e-9)
            .collect()
    };
    let &[first, second, third, fourth] = through_the_apex(pyramid).as_slice() else {
        panic!("{placement:?}: four side planes run through the apex");
    };
    let apexes = [
        [second, third, fourth],
        [first, third, fourth],
        [first, second, fourth],
        [first, second, third],
    ];
    let wedge_planes = through_the_apex(cut);
    assert_eq!(
        wedge_planes.len(),
        2,
        "{placement:?}, {wedge:?}: two wedge planes run through the apex"
    );
    wedge_planes.into_iter().any(|plane| {
        let sides = apexes.map(|three| {
            side_of_a_plane_where_three_planes_meet(three, plane)
                .expect("three side planes meet in a point")
        });
        assert!(
            !sides
                .iter()
                .any(|side| matches!(side, Decision::Undecidable { .. })),
            "{placement:?}, {wedge:?}: exact arithmetic decides every side, got {sides:?}"
        );
        sides.contains(&Decision::Pos) && sides.contains(&Decision::Neg)
    })
}

/// `solid` and `partner` fused under `rule` where they were built, which has
/// to answer under `certified` with nothing in the log, and every solid of the
/// answer then moved by `placement`.
fn fused_then_placed(
    model: &mut Model,
    (solid, partner): (SolidKey, SolidKey),
    rule: Rule,
    placement: Isometry,
    case: &str,
) -> Fused {
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(&[solid], &[partner], rule, &Policy::certified(), &mut log)
        .unwrap_or_else(|error| panic!("{case}: refused where they were built: {error:?}"));
    assert!(
        log.records().is_empty(),
        "{case}: a certified answer logs nothing"
    );
    for &piece in answer.solids() {
        model
            .place_solid(piece, &placement)
            .expect("this model made the key");
    }
    answer
}

/// How many faces an answer has, and how many sides their outlines and holes
/// have between them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FacesAndSides {
    faces: usize,
    sides: usize,
}

fn the_faces_and_sides_of(model: &Model, answer: &Fused) -> FacesAndSides {
    let faces: Vec<_> = answer
        .solids()
        .iter()
        .flat_map(|&solid| model.solid(solid).expect("this model made the key").faces())
        .collect();
    let sides = faces
        .iter()
        .flat_map(|face| face.wires())
        .map(|wire| wire.coedges().count())
        .sum();
    FacesAndSides {
        faces: faces.len(),
        sides,
    }
}

/// The half bar with its corner moved to `corner` in its file, at `placement`,
/// fused with the block `partner` names under its rule, which has to refuse and
/// leave the model byte for byte as it was. What it refused with.
fn the_half_bar_refuses(
    corner: [f64; 3],
    placement: Isometry,
    (partner, rule): (Block, Rule),
    policy: &Policy,
) -> Refused {
    let case = format!("{policy:?}, {placement:?}, the corner at {corner:?}, {partner:?} {rule:?}");
    let (mut model, solid) = the_half_bar_read_back(placement, corner);
    let block = a_block(&mut model, partner, &placement);
    let before = postcard::to_allocvec(&model).expect("a model writes out");
    let refused = match model.fuse(&[solid], &[block], rule, policy, &mut ProvenanceLog::new()) {
        Err(Error::Invalid(invalid)) => Refused::Invalid(invalid),
        Err(Error::Undecidable { records }) => Refused::Undecidable(
            records
                .records()
                .iter()
                .map(|record| record.predicate())
                .collect(),
        ),
        Err(Error::TwoCurvesTouchWithoutCrossing { .. }) => Refused::TwoCurvesTouchWithoutCrossing,
        other => panic!("{case} came back {other:?}"),
    };
    assert_eq!(
        postcard::to_allocvec(&model).expect("a model writes out"),
        before,
        "{case}: a refused fuse leaves the model as it was"
    );
    refused
}

/// For every moved solid `moved` reads back at each placement, under both
/// policies, every fuse with every partner refuses `Undecidable` with one
/// record, "where three surfaces meet", and leaves the model byte for byte as
/// it was.
fn every_fuse_refuses_where_three_surfaces_meet(
    moved: fn(Isometry, [f64; 3]) -> (Model, SolidKey),
    moves: [[f64; 3]; 4],
) {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for placement in every_placement() {
            for moved_to in moves {
                for (partner, rule) in EVERY_PARTNER {
                    let case = format!(
                        "{policy:?}, {placement:?}, the corner at {moved_to:?}, {partner:?} {rule:?}"
                    );
                    let (mut model, solid) = moved(placement, moved_to);
                    let block = a_block(&mut model, partner, &placement);
                    let before = postcard::to_allocvec(&model).expect("a model writes out");
                    match model.fuse(&[solid], &[block], rule, &policy, &mut ProvenanceLog::new()) {
                        Err(Error::Undecidable { records }) => {
                            let named: Vec<&str> = records
                                .records()
                                .iter()
                                .map(|record| record.predicate())
                                .collect();
                            assert_eq!(named, [WHERE_THREE_SURFACES_MEET], "{case}");
                        }
                        other => panic!("{case} came back {other:?}"),
                    }
                    assert_eq!(
                        postcard::to_allocvec(&model).expect("a model writes out"),
                        before,
                        "{case}: a refused fuse leaves the model as it was"
                    );
                }
            }
        }
    }
}

/// How much the answer's solids hold, measured with their curved faces cut
/// into triangles no more than a thousandth of a millimetre off them, and how
/// far that can be from what they really hold. A solid with only flat faces
/// is measured exactly whatever the deflection, to a rounding.
fn the_volume_of(model: &Model, answer: &Fused, policy: &Policy, case: &str) -> (f64, f64) {
    let deflection = Deflection::of(0.001).expect("a thousandth of a millimetre");
    answer
        .solids()
        .iter()
        .map(|&piece| {
            let measured = model
                .solid(piece)
                .expect("a solid the fuse just wrote")
                .measure(deflection, policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|error| panic!("{case}: measuring refused: {error:?}"));
            (measured.volume(), measured.volume_error())
        })
        .fold((0.0, 0.0), |(volume, bound), (more, more_bound)| {
            (volume + more, bound + more_bound)
        })
}

fn every_placement() -> [Isometry; 4] {
    let shift = Isometry::translation(Vector::new(7.0, -5.0, 2.5)).expect("a plain translation");
    let turn = Isometry::rotation(&Axis::x(), 0.4).expect("a turn about x");
    [Isometry::identity(), shift, turn, shift.compose(&turn)]
}

/// The box at `placement`, written out with postcard, its corner overwritten in
/// the bytes with `moved_to`, and read back.
///
/// The placement goes on the box's shell, so the file stores the corner at
/// [`WHERE_THE_BOX_PUTS_IT`] at every placement, and `moved_to` is in that
/// stored frame too.
fn read_back(placement: Isometry, moved_to: [f64; 3]) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let footprint = ProfileBuilder::new(vec![
        [-SIDE, -SIDE],
        [SIDE, -SIDE],
        [SIDE, SIDE],
        [-SIDE, SIDE],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    let the_box = model
        .add_extrusion(&footprint, HEIGHT, &placement)
        .expect("a box");
    moved_in_its_bytes(&model, the_box, WHERE_THE_BOX_PUTS_IT, moved_to)
}

/// The pyramid at `placement`, written out with postcard, its apex overwritten
/// in the bytes with `moved_to`, and read back.
///
/// The placement goes on the pyramid's shell, as on the box's, so the file
/// stores the apex at [`WHERE_THE_PYRAMID_PUTS_ITS_APEX`] at every placement.
fn the_pyramid_read_back(placement: Isometry, moved_to: [f64; 3]) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let pyramid = a_pyramid(&mut model);
    model
        .place_solid(pyramid, &placement)
        .expect("this model made the key");
    moved_in_its_bytes(&model, pyramid, WHERE_THE_PYRAMID_PUTS_ITS_APEX, moved_to)
}

/// The round bar less [`THE_HALF_PAST_THE_SEAM`], at `placement`, written out
/// with postcard, its corner at [`WHERE_THE_HALF_BAR_PUTS_IT`] overwritten in
/// the bytes with `moved_to`, and read back.
///
/// The bar and the block are cut where they are built and the half bar placed
/// after, as the pyramid is, so the file stores the corner at the same place
/// at every placement. The bar and the block stay in the model beside it.
fn the_half_bar_read_back(placement: Isometry, moved_to: [f64; 3]) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar of radius three and height ten");
    let cut = a_block(&mut model, THE_HALF_PAST_THE_SEAM, &Isometry::identity());
    let answer = model
        .fuse(
            &[bar],
            &[cut],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the bar less the half past its seam");
    let [half_bar] = answer.solids()[..] else {
        panic!("the bar less a half is {} solids", answer.solids().len());
    };
    model
        .place_solid(half_bar, &placement)
        .expect("this model made the key");
    moved_in_its_bytes(&model, half_bar, WHERE_THE_HALF_BAR_PUTS_IT, moved_to)
}

/// `model` written out with postcard, the one place its bytes hold the corner
/// `was` overwritten with `moved_to`, and read back, with the solid that comes
/// back where `solid` was among the model's solids.
fn moved_in_its_bytes(
    model: &Model,
    solid: SolidKey,
    was: [f64; 3],
    moved_to: [f64; 3],
) -> (Model, SolidKey) {
    let position = model
        .solids()
        .position(|view| view.key() == solid)
        .expect("the model holds the solid");
    let mut bytes = postcard::to_allocvec(model).expect("a model writes out");

    let stored = the_bytes_of(was);
    let places: Vec<usize> = bytes
        .windows(stored.len())
        .enumerate()
        .filter(|(_, window)| *window == stored.as_slice())
        .map(|(at, _)| at)
        .collect();
    let [at] = places.as_slice() else {
        panic!("the corner is written once, and was found at {places:?}");
    };
    bytes[*at..*at + stored.len()].copy_from_slice(&the_bytes_of(moved_to));

    let read_back: Model =
        postcard::from_bytes(&bytes).expect("reading back does not ask where a corner is");
    let solid = read_back
        .solids()
        .nth(position)
        .expect("the solid came back")
        .key();
    (read_back, solid)
}

fn the_bytes_of(point: [f64; 3]) -> Vec<u8> {
    point
        .iter()
        .flat_map(|coordinate| coordinate.to_le_bytes())
        .collect()
}

fn a_block(model: &mut Model, block: Block, placement: &Isometry) -> SolidKey {
    let [wide, deep] = [block.high[0] - block.low[0], block.high[1] - block.low[1]];
    let footprint = ProfileBuilder::new(vec![[0.0, 0.0], [wide, 0.0], [wide, deep], [0.0, deep]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let own = Isometry::translation(Vector::new(block.low[0], block.low[1], block.floor))
        .expect("a plain translation");
    model
        .add_extrusion(&footprint, block.tall, &placement.compose(&own))
        .expect("a block")
}

fn a_wedge(model: &mut Model, wedge: Wedge, placement: &Isometry) -> SolidKey {
    let footprint = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [5.0, 5.0 * wedge.low],
        [5.0, 5.0 * wedge.high],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a triangle");
    let own = Isometry::translation(Vector::new(0.0, 0.0, -1.0)).expect("a plain translation");
    model
        .add_extrusion(&footprint, 12.0, &placement.compose(&own))
        .expect("a wedge")
}

/// How many solids and how much volume the sound box and `partner` give under
/// `rule`, from where the two overlap along each axis.
fn what_the_two_boxes_hold(partner: Block, rule: Rule) -> (usize, f64) {
    let the_box = (2.0 * SIDE) * (2.0 * SIDE) * HEIGHT;
    let shared = overlap([-SIDE, SIDE], [partner.low[0], partner.high[0]])
        * overlap([-SIDE, SIDE], [partner.low[1], partner.high[1]])
        * overlap([0.0, HEIGHT], [partner.floor, partner.floor + partner.tall]);
    what_a_solid_and_a_block_hold(the_box, shared, partner, rule)
}

/// A pyramid on the box's footprint with its apex at
/// [`WHERE_THE_PYRAMID_PUTS_ITS_APEX`], over the middle, so four planes meet
/// there, built face by face with each side face's plane written through the
/// apex.
fn a_pyramid(model: &mut Model) -> SolidKey {
    let points = [
        WHERE_THE_PYRAMID_PUTS_ITS_APEX,
        [-SIDE, -SIDE, 0.0],
        [SIDE, -SIDE, 0.0],
        [SIDE, SIDE, 0.0],
        [-SIDE, SIDE, 0.0],
    ]
    .map(|[x, y, z]| Point::new(x, y, z));
    // Counter-clockwise seen from outside, the apex first on every side face.
    let faces: [&[usize]; 5] = [
        &[0, 1, 2],
        &[0, 2, 3],
        &[0, 3, 4],
        &[0, 4, 1],
        &[1, 4, 3, 2],
    ];

    let mut builder = SolidBuilder::new();
    let vertices = points.map(|point| builder.add_vertex(point).expect("a finite corner"));
    let mut edges: Vec<([usize; 2], DraftEdge)> = Vec::new();
    let drafted = faces.map(|face| {
        let steps: Vec<DraftStep> = (0..face.len())
            .map(|position| {
                let (from, to) = (face[position], face[(position + 1) % face.len()]);
                let (edge, orientation) = match edges.iter().find(|(ends, _)| *ends == [to, from]) {
                    Some(&(_, edge)) => (edge, Orientation::Reversed),
                    None => {
                        let edge = builder
                            .add_edge(vertices[from], vertices[to], EdgeRun::Straight)
                            .expect("a side between two corners");
                        edges.push(([from, to], edge));
                        (edge, Orientation::Forward)
                    }
                };
                DraftStep::Along(
                    builder
                        .add_coedge(edge, orientation)
                        .expect("one use of a side"),
                )
            })
            .collect();
        let wire = builder.add_wire(&steps).expect("a closed boundary");
        let plane = Plane::through_three_points(points[face[0]], points[face[1]], points[face[2]])
            .expect("three corners of a face not in a line");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a face of a pyramid")
    });
    builder
        .add_shell(&drafted.map(|face| (face, Orientation::Forward)))
        .expect("five faces close a pyramid");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a pyramid is a sound solid")
}

/// How many solids and how much volume the sound pyramid and `partner` give
/// under `rule`.
fn what_the_pyramid_and_a_block_hold(partner: Block, rule: Rule) -> (usize, f64) {
    let the_pyramid = (2.0 * SIDE) * (2.0 * SIDE) * HEIGHT / 3.0;
    what_a_solid_and_a_block_hold(
        the_pyramid,
        what_the_pyramid_shares_with(partner),
        partner,
        rule,
    )
}

/// The volume the pyramid and `block` hold in common.
///
/// At height `z` the pyramid is a square `SIDE·(1 − z/HEIGHT)` either way of
/// its middle, so what it shares with the block there is a product of two
/// overlaps, each straight in `z` between the heights where the square's edge
/// passes one of the block's sides. Between those heights the product is a
/// quadratic, and Simpson's rule is exact for it.
fn what_the_pyramid_shares_with(block: Block) -> f64 {
    let [bottom, top] = [block.floor.max(0.0), (block.floor + block.tall).min(HEIGHT)];
    if bottom >= top {
        return 0.0;
    }
    let mut heights = vec![bottom, top];
    for side in [block.low[0], block.high[0], block.low[1], block.high[1]] {
        let height = HEIGHT * (1.0 - side.abs() / SIDE);
        if bottom < height && height < top {
            heights.push(height);
        }
    }
    heights.sort_by(f64::total_cmp);
    let across = |z: f64| {
        let half = SIDE * (1.0 - z / HEIGHT);
        overlap([-half, half], [block.low[0], block.high[0]])
            * overlap([-half, half], [block.low[1], block.high[1]])
    };
    heights
        .windows(2)
        .map(|pair| {
            let [from, to] = [pair[0], pair[1]];
            (to - from) / 6.0 * (across(from) + 4.0 * across(f64::midpoint(from, to)) + across(to))
        })
        .sum()
}

/// How many solids and how much volume the sound pyramid and `wedge` give
/// under `rule`. They always overlap, so every rule gives one solid.
///
/// At height `z` the pyramid is the square `|x|, |y| ≤ s` with `s = SIDE·(1 −
/// z/HEIGHT)`, and the wedge takes the part of it between two lines through
/// its middle, `s²` times what it takes of the square `|x|, |y| ≤ 1`. Of that
/// square the line `y = m·x`, `x ≥ 0`, leaves `m/2` between it and `y = 0`
/// for `|m| ≤ 1`, and `±(1 − 1/(2|m|))` past a corner. The pyramid's `s²`
/// over its height is `SIDE²·HEIGHT/3`.
fn what_the_pyramid_and_a_wedge_hold(wedge: Wedge, rule: Rule) -> (usize, f64) {
    let up_to = |m: f64| {
        if m.abs() <= 1.0 {
            m / 2.0
        } else {
            m.signum() * (1.0 - 1.0 / (2.0 * m.abs()))
        }
    };
    let shared = (up_to(wedge.high) - up_to(wedge.low)) * SIDE * SIDE * HEIGHT / 3.0;
    let the_pyramid = (2.0 * SIDE) * (2.0 * SIDE) * HEIGHT / 3.0;
    let the_wedge = 5.0 * 5.0 * (wedge.high - wedge.low) / 2.0 * 12.0;
    if rule == Rule::Difference {
        (1, the_pyramid - shared)
    } else if rule == Rule::Intersection {
        (1, shared)
    } else {
        assert_eq!(rule, Rule::Union, "no wedge here is fused under {rule:?}");
        (1, the_pyramid + the_wedge - shared)
    }
}

/// How many solids and how much volume the sound half bar and `partner` give
/// under `rule`.
fn what_the_half_bar_and_a_block_hold(partner: Block, rule: Rule) -> (usize, f64) {
    let the_half_bar = PI * RADIUS * RADIUS * HEIGHT / 2.0;
    what_a_solid_and_a_block_hold(
        the_half_bar,
        what_the_half_bar_shares_with(partner),
        partner,
        rule,
    )
}

/// The volume the half bar and `block` hold in common.
///
/// The half bar stands on the half of a disc of radius `RADIUS` where `y ≤ 0`.
/// Every block here either keeps to `y ≥ 0` or reaches right across that half
/// disc, from `y = −RADIUS` to `y = 0`, so what the two footprints share is
/// the half disc between the block's two `x` sides. The half disc's area left
/// of `x` is `(x·√(R² − x²) + R²·asin(x/R))/2` plus a constant.
fn what_the_half_bar_shares_with(block: Block) -> f64 {
    if block.low[1] >= 0.0 {
        return 0.0;
    }
    assert!(
        block.low[1] <= -RADIUS && block.high[1] >= 0.0,
        "{block:?} reaches only part of the way across the half disc"
    );
    let left_of = |x: f64| {
        let x = x.clamp(-RADIUS, RADIUS);
        (x * (RADIUS * RADIUS - x * x).sqrt() + RADIUS * RADIUS * (x / RADIUS).asin()) / 2.0
    };
    (left_of(block.high[0]) - left_of(block.low[0]))
        * overlap([0.0, HEIGHT], [block.floor, block.floor + block.tall])
}

/// How many solids and how much volume a solid holding `the_solid` and
/// `partner` give under `rule`, when the two hold `shared` in common. A union
/// of two that share nothing is two solids: no partner here fused under
/// `Union` touches the solid.
fn what_a_solid_and_a_block_hold(
    the_solid: f64,
    shared: f64,
    partner: Block,
    rule: Rule,
) -> (usize, f64) {
    let the_block =
        (partner.high[0] - partner.low[0]) * (partner.high[1] - partner.low[1]) * partner.tall;
    match rule {
        Rule::Union if shared == 0.0 => (2, the_solid + the_block),
        Rule::Union => (1, the_solid + the_block - shared),
        Rule::Intersection if shared == 0.0 => (0, 0.0),
        Rule::Intersection => (1, shared),
        Rule::Difference => {
            assert!(shared < the_solid, "no partner here holds the whole solid");
            (1, the_solid - shared)
        }
        _ => panic!("no partner here is fused under {rule:?}"),
    }
}

fn overlap(one: [f64; 2], other: [f64; 2]) -> f64 {
    (one[1].min(other[1]) - one[0].max(other[0])).max(0.0)
}
