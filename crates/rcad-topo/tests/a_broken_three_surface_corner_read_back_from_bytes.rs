//! A box whose top corner, where three planes meet, and a pyramid whose apex,
//! where four planes meet, are moved in their file, read back, and fused.
//! `docs/decisions.md` §294.16 and §294.17 are the record and this file is
//! their measurement.
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
//! but one: cut by a block whose face runs exactly through the apex, it
//! cannot say which side of that face the apex is on, which is a defect of its
//! own and pinned here as it stands.
//!
//! The corner is moved in the bytes because nothing in the public API writes a
//! corner off its surfaces: every builder validates what it built.

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, SIDE_OF_PLANE, Scale, Surface,
    Vector, WHERE_THREE_SURFACES_MEET,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftStep, Error, Fused, Invalid, Model, ProfileBuilder, Rule, SolidBuilder,
    SolidKey,
};

/// What a fuse's record names when it cannot say whether a point is inside a
/// piece of a face. `rcad-topo` keeps the name private, in `fuse/select.rs`.
const A_POINT_INSIDE_A_PIECE_OF_A_FACE: &str = "a point inside a piece of a face";

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
/// the apex, pinned on its own below.
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

/// **The sound pyramid cut through its apex refuses**, which is a defect and
/// not the moved apex's refusal (§294.17). `THE_HALF_PAST_THE_SEAM`'s face
/// `y = 0` runs exactly through the apex, and under `certified` the fuse asks
/// which side of a plane a box round the apex is on, a few ulps across, and
/// cannot say, at every placement. Under `tolerant(100)` it answers half the
/// pyramid with no placement, and refuses with each of the other three on four
/// such questions and one about a point inside a piece of a face.
#[test]
fn a_sound_pyramid_cut_through_its_apex_cannot_tell_which_side_of_the_cut_its_apex_is_on() {
    let [identity, shift, turn, turn_then_shift] = every_placement();
    for rule in [Rule::Difference, Rule::Intersection] {
        for placement in every_placement() {
            assert_eq!(
                the_refusal_cutting_through_the_apex(placement, rule, &Policy::certified()),
                [SIDE_OF_PLANE],
                "certified, {placement:?}, {rule:?}"
            );
        }
        for placement in [shift, turn, turn_then_shift] {
            assert_eq!(
                the_refusal_cutting_through_the_apex(placement, rule, &Policy::tolerant(100.0)),
                [
                    SIDE_OF_PLANE,
                    SIDE_OF_PLANE,
                    SIDE_OF_PLANE,
                    SIDE_OF_PLANE,
                    A_POINT_INSIDE_A_PIECE_OF_A_FACE
                ],
                "tolerant, {placement:?}, {rule:?}"
            );
        }

        let case = format!("tolerant, no placement, {rule:?}");
        let policy = Policy::tolerant(100.0);
        let (mut model, solid) = the_pyramid_read_back(identity, WHERE_THE_PYRAMID_PUTS_ITS_APEX);
        let block = a_block(&mut model, THE_HALF_PAST_THE_SEAM, &identity);
        let answer = model
            .fuse(&[solid], &[block], rule, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|error| panic!("{case} refused: {error:?}"));
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|error| panic!("{case}: the answer does not validate: {error:?}"));
        let half = (2.0 * SIDE) * (2.0 * SIDE) * HEIGHT / 3.0 / 2.0;
        assert_eq!(
            what_the_pyramid_and_a_block_hold(THE_HALF_PAST_THE_SEAM, rule),
            (1, half)
        );
        assert_eq!(answer.solids().len(), 1, "{case}: how many solids");
        let measured = the_volume_of(&model, &answer, &policy, &case);
        assert!(
            (measured - half).abs() <= 1e-12 * half,
            "{case}: the answer holds {measured}, half the pyramid {half}"
        );
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

#[test]
fn probe_q1813_half_bar() {
    let mut moves = EVERY_MOVE_OF_THE_HALF_BAR.to_vec();
    moves.push(THE_OTHER_END_OF_THE_CUT);
    moves.push(WHERE_THE_HALF_BAR_PUTS_IT);
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for (index, placement) in every_placement().into_iter().enumerate() {
            for &moved_to in &moves {
                let (model, solid) = the_half_bar_read_back(placement, moved_to);
                let faces = model.solid(solid).expect("key").faces().count();
                let valid = model.validate(&policy, &mut ProvenanceLog::new());
                eprintln!(
                    "VALIDATE {:?} p{index} {moved_to:?} faces {faces}: {valid:?}",
                    policy.max_wobble()
                );
                for (which, (partner, rule)) in EVERY_PARTNER.into_iter().enumerate() {
                    let (mut model, solid) = the_half_bar_read_back(placement, moved_to);
                    let block = a_block(&mut model, partner, &placement);
                    let outcome =
                        model.fuse(&[solid], &[block], rule, &policy, &mut ProvenanceLog::new());
                    let said = match &outcome {
                        Ok(answer) => {
                            let volume: f64 = answer
                                .solids()
                                .iter()
                                .map(|&piece| {
                                    model
                                        .solid(piece)
                                        .expect("piece")
                                        .measure(
                                            Deflection::of(0.001).expect("fine"),
                                            &policy,
                                            &mut ProvenanceLog::new(),
                                        )
                                        .map(|m| m.volume())
                                        .unwrap_or(f64::NAN)
                                })
                                .sum();
                            format!("Ok {} solids, volume {volume}", answer.solids().len())
                        }
                        Err(Error::Undecidable { records }) => format!(
                            "Undecidable {:?}",
                            records
                                .records()
                                .iter()
                                .map(|record| record.predicate())
                                .collect::<Vec<_>>()
                        ),
                        Err(other) => format!("Err {other:?}"),
                    };
                    eprintln!(
                        "FUSE {:?} p{index} {moved_to:?} partner{which} {rule:?}: {said}",
                        policy.max_wobble()
                    );
                }
            }
        }
    }
}

/// The sound solid from `the_sound_solid` at each placement, fused with each
/// of `partners` under both policies, answers, validates, and holds what
/// `what_the_two_hold` says, to `1e-12` relative.
fn answers_with_what_the_two_hold(
    the_sound_solid: impl Fn(Isometry) -> (Model, SolidKey),
    partners: &[(Block, Rule)],
    what_the_two_hold: fn(Block, Rule) -> (usize, f64),
) {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for placement in every_placement() {
            for &(partner, rule) in partners {
                let case = format!("{policy:?}, {placement:?}, {partner:?} {rule:?}");
                let (mut model, solid) = the_sound_solid(placement);
                let block = a_block(&mut model, partner, &placement);
                let answer = model
                    .fuse(&[solid], &[block], rule, &policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|error| panic!("{case} refused: {error:?}"));
                model
                    .validate(&policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|error| {
                        panic!("{case}: the answer does not validate: {error:?}")
                    });

                let (solids, volume) = what_the_two_hold(partner, rule);
                assert_eq!(answer.solids().len(), solids, "{case}: how many solids");
                let measured = the_volume_of(&model, &answer, &policy, &case);
                assert!(
                    (measured - volume).abs() <= 1e-12 * volume,
                    "{case}: the answer holds {measured}, the two {volume}"
                );
            }
        }
    }
}

/// The sound pyramid at `placement` fused with `THE_HALF_PAST_THE_SEAM` under
/// `rule`, which has to refuse and leave the model as it was. Returns the
/// predicates the refusal's records name, each "side of plane" one checked to
/// have been asked of one box, and that box round the placed apex.
fn the_refusal_cutting_through_the_apex(
    placement: Isometry,
    rule: Rule,
    policy: &Policy,
) -> Vec<&'static str> {
    let case = format!("{policy:?}, {placement:?}, {rule:?}");
    let [x, y, z] = WHERE_THE_PYRAMID_PUTS_ITS_APEX;
    let apex = placement.apply_to_point(Point::new(x, y, z)).coordinates();
    let (mut model, solid) = the_pyramid_read_back(placement, WHERE_THE_PYRAMID_PUTS_ITS_APEX);
    let block = a_block(&mut model, THE_HALF_PAST_THE_SEAM, &placement);
    let before = postcard::to_allocvec(&model).expect("a model writes out");
    let Err(Error::Undecidable { records }) =
        model.fuse(&[solid], &[block], rule, policy, &mut ProvenanceLog::new())
    else {
        panic!("{case}: the cut through the apex did not refuse Undecidable");
    };
    assert_eq!(
        postcard::to_allocvec(&model).expect("a model writes out"),
        before,
        "{case}: a refused fuse leaves the model as it was"
    );
    for record in records
        .records()
        .iter()
        .filter(|record| record.predicate() == SIDE_OF_PLANE)
    {
        let [asked] = record.inputs() else {
            panic!("{case}: side of plane asked of {:?}", record.inputs());
        };
        let holds_the_apex = asked
            .enclosure()
            .iter()
            .zip(apex)
            .all(|(side, coordinate)| side.lo() <= coordinate && coordinate <= side.hi());
        assert!(
            holds_the_apex,
            "{case}: side of plane asked of {asked:?}, not round the apex {apex:?}"
        );
    }
    records
        .records()
        .iter()
        .map(|record| record.predicate())
        .collect()
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

fn the_volume_of(model: &Model, answer: &Fused, policy: &Policy, case: &str) -> f64 {
    answer
        .solids()
        .iter()
        .map(|&piece| {
            model
                .solid(piece)
                .expect("a solid the fuse just wrote")
                .measure(
                    Deflection::for_scale(model.scale()),
                    policy,
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|error| panic!("{case}: measuring refused: {error:?}"))
                .volume()
        })
        .sum()
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

/// How many solids and how much volume the sound box and `partner` give under
/// `rule`, from where the two overlap along each axis.
fn what_the_two_boxes_hold(partner: Block, rule: Rule) -> (usize, f64) {
    let the_box = (2.0 * SIDE) * (2.0 * SIDE) * HEIGHT;
    let the_block =
        (partner.high[0] - partner.low[0]) * (partner.high[1] - partner.low[1]) * partner.tall;
    let shared = overlap([-SIDE, SIDE], [partner.low[0], partner.high[0]])
        * overlap([-SIDE, SIDE], [partner.low[1], partner.high[1]])
        * overlap([0.0, HEIGHT], [partner.floor, partner.floor + partner.tall]);
    match rule {
        Rule::Union if shared == 0.0 => (2, the_box + the_block),
        Rule::Union => (1, the_box + the_block - shared),
        Rule::Intersection => {
            assert!(shared > 0.0, "no partner here leaves nothing in common");
            (1, shared)
        }
        Rule::Difference => (1, the_box - shared),
        _ => panic!("no partner here is fused under {rule:?}"),
    }
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
    let the_block =
        (partner.high[0] - partner.low[0]) * (partner.high[1] - partner.low[1]) * partner.tall;
    let shared = what_the_pyramid_shares_with(partner);
    match rule {
        Rule::Union if shared == 0.0 => (2, the_pyramid + the_block),
        Rule::Union => (1, the_pyramid + the_block - shared),
        Rule::Intersection if shared == 0.0 => (0, 0.0),
        Rule::Intersection => (1, shared),
        Rule::Difference => {
            assert!(
                shared < the_pyramid,
                "no partner here holds the whole pyramid"
            );
            (1, the_pyramid - shared)
        }
        _ => panic!("no partner here is fused under {rule:?}"),
    }
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

fn overlap(one: [f64; 2], other: [f64; 2]) -> f64 {
    (one[1].min(other[1]) - one[0].max(other[0])).max(0.0)
}
