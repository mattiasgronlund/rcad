//! A pyramid's apex, where four planes meet, and a face's own meet folded into
//! it (`docs/decisions.md` §301.4, §301.7).
//!
//! `bake` solves a corner with four planes round it from the best-conditioned
//! of its four triples. `place_the_meets` asks each side face where its *own*
//! triple meets: the face and its two neighbours. At a pyramid's apex the four
//! triples are the four faces' own, so one face asks about the triple `bake`
//! used and three ask about another.
//!
//! **Bend the pyramid's base at one corner, and the two side faces there fold
//! by only a little.** Both triples holding those two faces are then three
//! planes nearly through one line, so the solve gives a wide box, while `bake`
//! picks one of the other two triples and its box stays narrow. So those two
//! faces' meets stick out past the apex's box, by roughly the inverse of the
//! fold — and not only their boxes: the point the three stored planes meet at,
//! worked out exactly, sticks out too, because a plane rounded by an ulp moves
//! the line two nearly-flat faces cross in by about an ulp over the fold.
//!
//! Measured over 108 certified unions per fold, each pyramid with a cube clear
//! of it: heights `1/4`, `1` and `4`, each side plane written through the apex
//! or through a base corner, and the 18 turns below. The two "sticks out"
//! columns are in rooms past the corner's own box, over every meet whose box
//! touched it, since until §395 the fold asked nothing about any other meet.
//! "§253 alone" takes a meet only when its whole box lies inside the corner's
//! box grown by the room. "With §356" adds the rule that takes a meet of three
//! planes `bake` solved the corner among however far it reaches, once its box
//! touches the corner's grown box. "With §395"
//! drops the test that came before both: whether the meet's box touches the
//! corner's at all. "With §395.8" takes a meet of three planes `bake` solved
//! the corner among while its narrowed box lies within the room `bake` asked
//! those planes with, and no further.
//!
//! | Fold | Meet's box sticks out | Exact meet sticks out | §253 alone | §253 alone, room ×0.5 | §253 alone, room ×1000 | With §356 | With §395 | With §395.8 |
//! |---|---|---|---|---|---|---|---|---|
//! | 1 | 0.12 | 0.04 | 108 | 108 | 108 | 108 | 108 | 108 |
//! | 1/4 | 0.24 | 0.12 | 103 | 103 | 103 | 103 | 108 | 108 |
//! | 1/16 | 1.17 | 0.46 | 99 | 31 | 103 | 103 | 108 | 108 |
//! | 1/64 | 4.7 | 1.6 | 0 | 0 | 107 | 107 | 108 | 108 |
//! | 1/256 | 21 | 6.9 | 0 | 0 | 105 | 107 | 107 | 108 |
//! | 1/4096 | 290 | 108 | 0 | 0 | 97 | 104 | 104 | 108 |
//!
//! The first six columns are §356.1's, on `9612b25f`. "With §356" was 105 and
//! 97 at the two smallest folds when that rule landed. It is re-measured on
//! `25860052`, where `bake` no longer refuses the base corner the two folding
//! faces share, and "with §395" is the same table on this file's commit
//! (§395.2). What stopped that refusal is the room `bake`'s picker grants a
//! corner, `validate`'s and not the rounding alone (§368), and the nine it
//! answered are pinned by name below (§301.9). Since Q866 orders a meet of
//! three planes by its narrowed box where the boxes cannot say, the 4096th
//! answers 105. Two of its 4 refusals were on the order: one answers now, and
//! the other refuses further on, as the other 2 and the 256th's 1 do, with two
//! regions of a face that cannot be told apart.
//!
//! **This file used to say the refusals left with §356 had every apex meet
//! folded. They did not** (§344.2, §344.4). A side face has a meet at the apex
//! for each pair of the other three side planes, and `bake` solved the apex
//! from one triple of the four. The other meets' boxes can lie inside the
//! apex's room and still miss its own box, by up to four tenths of a room, and
//! the touch test turned them away before the room was looked at. Left beside
//! the apex, two of them could not be ordered along a crossing, or the cells
//! they cut off round it wound outside 0 and 1. The 4 left after §395 — 1 at a
//! 256th and 3 at a 4096th — were left by apex meets of three of the apex's
//! own planes that miss its grown box. Over the 648 unions the fold now takes
//! 27 such calls it did not take before, sticking out 3.7 to 133 rooms past
//! the apex's own box. §356's argument folds those too, but followed with no bound it also
//! folds meets of three planes that share a line, 1.6 and 326 units off their
//! corners in a KiCad relay (§395.7). The fold now grants a meet of three of
//! the apex's own planes the room `bake` asked them with, at least `1e-9`
//! here, and all 648 unite (§395.8).
//!
//! **So a four-plane corner does fill the room, and the room is not what it
//! should be measured in.** Every plane round the apex passes through the
//! apex's box grown by the room, which `bake` checks; carried through a triple
//! of nearly-flat planes, that room grows by about one over the fold. The
//! sixty-fourth was a gap, not a limit, and is closed.
//!
//! **`bake`'s room runs out at a fold of about `2⁻²⁴`.** Folded by `2⁻²⁰`,
//! all 108 unite, and the furthest apex meet reaches a quarter of that room
//! past the apex's box. Folded by `2⁻²⁴`, 34 of 108 refuse, with apex meets
//! left from just past the room (§395.8).
//!
//! **Three more corners, reached below (Q275).** A corner with five or six
//! planes round it folds no meet at all: a regular pentagon or hexagon base
//! gives `bake` more well-conditioned triples to choose among, not fewer, so
//! every one of 72 certified unions measures `stick_out_rooms=0`
//! (`a_pyramid_with_five_or_six_planes_round_its_apex_is_united`,
//! `RCAD_Q275_PROBE`). A side face with a sliver of an angle at its apex —
//! two base corners a thousandth, a hundred-thousandth or a millionth apart —
//! folds nothing either, over 108 unions at three gaps: a sliver narrows one
//! face's triangle, not any pair of planes round the apex, so it does not
//! make `bake`'s own triple worse-conditioned
//! (`a_pyramid_with_a_sliver_face_at_its_apex_is_united`). A gap of a
//! hundred-millionth fails to build at all, at every turn and spelling tried
//! but one — `Plane::through_three_points` fits the sliver face's plane
//! through its first two corners, and the third then lands a few billionths
//! off it, past `VertexOffItsFacePlane`'s tolerance — pinned by name at one
//! such turn
//! (`a_pyramid_with_too_fine_a_sliver_face_refuses_to_build`). This is a
//! limit of fitting a plane through near-coincident points and not of the
//! fold room, so it is left there rather than chased further. An apex a
//! million units from the origin
//! folds nothing either and unites at every turn and spelling
//! (`a_pyramid_whose_apex_is_far_from_the_origin_is_united`); a trillion
//! units out refuses to build at all, past `DynamicRange::LARGEST_SUPPORTED_RATIO`
//! (`1e11`, §3.5), a limit of `Model::new`'s own declared scale envelope and
//! not of the fold
//! (`a_pyramid_whose_apex_is_far_enough_to_leave_the_scale_envelope_refuses_to_build`).
//!
//! Measured with `RCAD_Q275_PROBE=1 mise exec -- cargo nextest run -p
//! rcad-topo -E 'binary(merged) and test(/^a_corner_with_four_planes_round_it::/)'
//! --no-capture`,
//! a probe temporarily added to `Arrangement::the_corner_that_is_this_place`
//! (`crates/rcad-topo/src/fuse/arrange.rs`) and taken back out before this
//! file's own commit — none of these three corners stuck a meet out past its
//! own box at all, so no second table is needed beside §310's.

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftEdge, DraftStep, Model, Rule, SolidBuilder, SolidKey};

const TURNS: [f64; 6] = [0.1, 0.3, 0.7, 1.1, 1.9, 2.5];

const AXES: [[f64; 3]; 3] = [[1.0, 2.0, 3.0], [3.0, -1.0, 2.0], [0.3, 1.0, -0.7]];

/// Where a side face's plane is written down: through the apex, or through the
/// first base corner of that face. The four planes then miss one common point
/// by different roundings.
const SPELLINGS: [Through; 2] = [Through::TheApex, Through::ABaseCorner];

/// A pyramid of height one whose base is bent by a sixteenth at one corner is
/// united with a cube clear of it, turned 18 ways and with each spelling of
/// its planes, under a certified policy. Every one of the 36 answers with the
/// two solids, their volumes to a billionth, and an empty log.
///
/// **Its apex meets stick out up to 0.875 of the room past the apex's box, and
/// in 31 of the 36 more than half of it.** A meet the fold leaves standing
/// overlaps the apex, and a certified policy refuses to say whether two
/// overlapping points are one, so an answer here means every apex meet was
/// folded. Before §356, a room ×0.5 in a scratch edit to
/// `a_corner_is_this_place` refused 31 of the 36. Now every apex meet is a
/// meet of three planes `bake` solved the apex among, which folds within the
/// room `bake` asked those planes with, far wider than the rounding, so this
/// no longer holds the rounding from below; the hand-written test beside
/// `a_corner_is_this_place` does (§301.5, §395.8).
#[test]
fn a_pyramid_whose_two_faces_fold_by_a_sixteenth_is_united_though_its_apex_meets_stick_out_past_half_the_room()
 {
    every_turn_and_spelling_is_united(1.0 / 16.0, 1.0);
}

/// The same pyramid bent by a sixty-fourth, and by a two hundred and
/// fifty-sixth, is united at every one of the 36 turns and spellings of each.
///
/// At a sixty-fourth the folding faces' meets' boxes stick out up to 4.7 rooms
/// past the apex's box, and the point the three stored planes meet at, worked
/// out exactly, up to 1.6 rooms (the module table). Before §356 the fold did
/// not take them, the meet stayed a node beside the apex, and the certified
/// policy could not order the two along the face's edge: all 36 were refused
/// on "order along a crossing". **That was a gap, not a limit** (§16.10,
/// §310.3): every one of the four planes passes through the apex's box grown
/// by the room, which is what `bake` checks, and only the nearly-flat triple
/// carries that room out further in space.
#[test]
fn a_pyramid_whose_two_faces_fold_by_a_sixty_fourth_is_united_though_its_apex_meets_stick_out_past_the_room()
 {
    for fold in [1.0 / 64.0, 1.0 / 256.0] {
        every_turn_and_spelling_is_united(fold, 1.0);
    }
}

/// The pyramid bent by one, by a quarter, by a sixteenth and by a
/// sixty-fourth, at heights `1/4`, `1` and `4`, is united at every turn and
/// spelling: 108 unions for each bend. With the test below, that is the whole
/// of the module table, 648 unions.
///
/// **Before §395, 5 of the quarter's 108 were refused, 5 of the sixteenth's
/// and 1 of the sixty-fourth's**, every one with its side planes written
/// through a base corner. A side face has a meet at the apex for each pair of
/// the other three side planes. Some of those meets' boxes lie inside the
/// apex's room but miss its own box, and the fold asked whether the two boxes
/// touch before it looked at the room. So they stayed nodes beside the apex,
/// and the certified policy could not order them along a crossing or tell the
/// cells round them apart (§344.2). The fold now reads the room alone.
#[test]
fn a_pyramid_whose_apex_meets_lie_inside_its_room_but_miss_its_box_is_united_at_every_height() {
    for fold in [1.0, 1.0 / 4.0, 1.0 / 16.0, 1.0 / 64.0] {
        for height in [1.0 / 4.0, 1.0, 4.0] {
            every_turn_and_spelling_is_united(fold, height);
        }
    }
}

/// The pyramid bent by a 256th and by a 4096th, at heights `1/4`, `1` and `4`,
/// is united at every turn and spelling: 108 unions for each bend.
///
/// **Some of their apex meets miss the apex's room by up to 133 times it.**
/// They are meets of three of the apex's own planes, two of them nearly one
/// plane, so a rounding in either moves the meet about one over the fold
/// further. `bake` asked each of those planes about the apex grown by its own
/// room, `validate`'s room off a plane plus the rounding, which is at least
/// `1e-9` for these pyramids, and no meet folded here reaches past the apex's
/// box by more than `1.7e-3` of that. So the fold takes such a meet as far as
/// that room and no further (§395.8). Before it took them only as far as the
/// rounding, and 4 of these 216 were refused, each with its planes written
/// through a base corner (§395.4).
#[test]
fn a_pyramid_whose_apex_meets_miss_the_rounding_but_lie_within_bakes_room_is_united() {
    for fold in [1.0 / 256.0, 1.0 / 4096.0] {
        for height in [1.0 / 4.0, 1.0, 4.0] {
            every_turn_and_spelling_is_united(fold, height);
        }
    }
}

/// The four pyramids §395.4 left refused, each 1/4 high with its planes
/// written through a base corner, are united, and each comes back with the
/// corners it was built with.
///
/// They refused `CannotTellTheCellsOfAFaceApart`: their apex meets were left
/// nodes beside the apex, and the cells round it could not be told apart. The
/// corners check says the union keeps no corner beside the apex either
/// (§395.8).
#[test]
fn the_four_pyramids_whose_apex_meets_missed_the_rounding_come_back_with_their_corners() {
    for (fold, height, through, turn, axis) in [
        (
            1.0 / 256.0,
            0.25,
            Through::ABaseCorner,
            1.9,
            [3.0, -1.0, 2.0],
        ),
        (
            1.0 / 4096.0,
            0.25,
            Through::ABaseCorner,
            0.3,
            [3.0, -1.0, 2.0],
        ),
        (
            1.0 / 4096.0,
            0.25,
            Through::ABaseCorner,
            1.9,
            [0.3, 1.0, -0.7],
        ),
        (
            1.0 / 4096.0,
            0.25,
            Through::ABaseCorner,
            2.5,
            [1.0, 2.0, 3.0],
        ),
    ] {
        is_united(fold, height, through, turn, axis);
        comes_back_with_the_corners_it_was_built_with(fold, height, through, turn, axis);
    }
}

/// Nine pyramids bent by a 256th or a 4096th, whose base corner `(0, -1 -
/// fold, 0)` has one place where its three planes meet and that place lies
/// 2.8 to 47 rooms past the stored corner, are united.
///
/// That corner is where the two folding side faces meet the base. Each
/// folding plane misses it by its own rounding, and two planes `fold` apart
/// carry that miss about `1/fold` times further. `bake`'s picker refused all
/// nine at 'where three surfaces meet' while it granted the corner its own
/// rounding alone (§344.3). It grants the room `Model::validate` grants the
/// pyramid's corners as well (§368), and the furthest place is a four
/// thousandth of that (§301.9). The tenth pyramid §344.3 named, folding a
/// 4096th, 4 high, through its apex and turned 0.3 about `(3, -1, 2)`, is
/// placed too. It was refused later, at its apex, until Q866, and is united
/// now ([`a_pyramid_whose_apex_meet_stays_a_node_beside_the_apex_is_united`]).
#[test]
fn a_pyramid_whose_base_corner_lies_past_its_own_rounding_from_where_its_planes_meet_is_united() {
    for (fold, height, through, turn, axis) in [
        (1.0 / 256.0, 4.0, Through::TheApex, 0.3, [0.3, 1.0, -0.7]),
        (1.0 / 256.0, 4.0, Through::TheApex, 0.7, [3.0, -1.0, 2.0]),
        (1.0 / 4096.0, 0.25, Through::TheApex, 0.1, [1.0, 2.0, 3.0]),
        (
            1.0 / 4096.0,
            0.25,
            Through::ABaseCorner,
            1.1,
            [0.3, 1.0, -0.7],
        ),
        (1.0 / 4096.0, 1.0, Through::TheApex, 0.1, [1.0, 2.0, 3.0]),
        (1.0 / 4096.0, 4.0, Through::TheApex, 0.1, [0.3, 1.0, -0.7]),
        (1.0 / 4096.0, 4.0, Through::TheApex, 1.9, [1.0, 2.0, 3.0]),
        (1.0 / 4096.0, 4.0, Through::TheApex, 2.5, [1.0, 2.0, 3.0]),
        (
            1.0 / 4096.0,
            4.0,
            Through::ABaseCorner,
            0.3,
            [3.0, -1.0, 2.0],
        ),
    ] {
        is_united(fold, height, through, turn, axis);
    }
}

/// A pyramid bent by a 4096th, 4 high, its side planes written through the
/// apex and turned 0.3 about `(3, −1, 2)`, is united, and comes back with the
/// corners it was built with.
///
/// It was one of the five §395.4 left refused, on "order along a crossing". A
/// meet of three of its apex's own planes misses the apex's room, so it is
/// not folded and stays a node beside the apex. Along a side face's edge its
/// box overlaps the apex's, and the planes cannot order the two either, since
/// the apex has four planes round it and `bake` does not say which three it
/// was solved from. The meet's box narrowed by one checked step lies `7e-13`
/// past the apex along that edge, which puts the two in order and the meet
/// off the face (Q866).
#[test]
fn a_pyramid_whose_apex_meet_stays_a_node_beside_the_apex_is_united() {
    let (fold, height, through, turn, axis) =
        (1.0 / 4096.0, 4.0, Through::TheApex, 0.3, [3.0, -1.0, 2.0]);
    is_united(fold, height, through, turn, axis);
    comes_back_with_the_corners_it_was_built_with(fold, height, through, turn, axis);
}

/// One pyramid united with a cube clear of it has as many corners as the two
/// were built with, and each lies within `1e-9` of one of them.
fn comes_back_with_the_corners_it_was_built_with(
    fold: f64,
    height: f64,
    through: Through,
    turn: f64,
    axis: [f64; 3],
) {
    let (mut model, pyramid, cube) =
        a_pyramid_and_a_cube_clear_of_it(fold, height, through, turn, axis);
    let built = corners_of(&model, &[pyramid, cube]);
    let fused = model
        .fuse(
            &[pyramid],
            &[cube],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the union answers");
    let united = corners_of(&model, fused.solids());
    assert_eq!(
        united.len(),
        built.len(),
        "the union has as many corners as the pyramid and the cube were built with"
    );
    for corner in united {
        let off = built
            .iter()
            .map(|was| corner.distance_to(*was))
            .fold(f64::INFINITY, f64::min);
        assert!(
            off <= 1e-9,
            "the union's corner {corner:?} is {off} from every corner it was built with"
        );
    }
}

/// Every corner of `solids`, once each.
fn corners_of(model: &Model, solids: &[SolidKey]) -> Vec<Point> {
    let mut corners: Vec<Point> = solids
        .iter()
        .flat_map(|&solid| model.solid(solid).expect("this model made the key").faces())
        .flat_map(|face| face.outer_wire().points().collect::<Vec<_>>())
        .collect();
    corners.sort_by_key(|corner| corner.coordinates().map(f64::to_bits));
    corners.dedup();
    corners
}

/// A pyramid bent by `fold`, `height` high, is united with a cube clear of it
/// at every turn and spelling, under a certified policy, into the two of them
/// with their volumes to a billionth and an empty log.
fn every_turn_and_spelling_is_united(fold: f64, height: f64) {
    for through in SPELLINGS {
        for turn in TURNS {
            for axis in AXES {
                is_united(fold, height, through, turn, axis);
            }
        }
    }
}

/// One pyramid united with a cube clear of it, as
/// [`every_turn_and_spelling_is_united`] asks of each.
fn is_united(fold: f64, height: f64, through: Through, turn: f64, axis: [f64; 3]) {
    let (mut model, pyramid, cube) =
        a_pyramid_and_a_cube_clear_of_it(fold, height, through, turn, axis);
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(
            &[pyramid],
            &[cube],
            Rule::Union,
            &Policy::certified(),
            &mut log,
        )
        .unwrap_or_else(|why| {
            panic!(
                "a pyramid folding {fold}, {height} high, with its planes through {through:?}, \
                 turned {turn} about {axis:?}, was refused: {why}"
            )
        });
    assert_eq!(
        fused.solids().len(),
        2,
        "two solids that do not touch unite as the two of them"
    );
    assert!(log.is_empty(), "a certified union recorded {log:?}");

    let pyramid_volume = (2.0 + fold) * height / 3.0;
    let united = volume_of(&model, fused.solids());
    assert!(
        (united - (pyramid_volume + 1.0)).abs() <= 1e-9,
        "a pyramid of {pyramid_volume} and a unit cube united to {united}"
    );
}

#[derive(Clone, Copy, Debug)]
enum Through {
    TheApex,
    ABaseCorner,
}

fn a_pyramid_and_a_cube_clear_of_it(
    fold: f64,
    height: f64,
    through: Through,
    turn: f64,
    [x, y, z]: [f64; 3],
) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let turned = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(x, y, z)).expect("a direction with length"),
        turn,
    )
    .expect("a finite turn");
    let pyramid = a_pyramid(&mut model, fold, height, through, &turned);
    let cube = model
        .add_cube(1.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a unit cube");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(100.0, 100.0, 100.0)).expect("a finite shift"),
        )
        .expect("this model made the key");
    (model, pyramid, cube)
}

/// A pyramid with its apex `height` above the middle of a four-cornered base,
/// described face by face in turned coordinates.
///
/// The base is `(-1, -1)`, `(0, -1 - fold)`, `(1, -1)`, `(0, 1)`: a triangle
/// with one long side pushed out by `fold`, so the two side faces over that
/// long side meet at a fold of about `fold` and the apex has four planes round
/// it. Its volume is `(2 + fold) × height / 3`.
fn a_pyramid(
    model: &mut Model,
    fold: f64,
    height: f64,
    through: Through,
    turned: &Isometry,
) -> SolidKey {
    let points = [
        [0.0, 0.0, height],
        [-1.0, -1.0, 0.0],
        [0.0, -1.0 - fold, 0.0],
        [1.0, -1.0, 0.0],
        [0.0, 1.0, 0.0],
    ]
    .map(|[x, y, z]| turned.apply_to_point(Point::new(x, y, z)));
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
        let [first, second, third] = match (through, face.len()) {
            (Through::ABaseCorner, 3) => [face[1], face[2], face[0]],
            _ => [face[0], face[1], face[2]],
        };
        let plane = Plane::through_three_points(points[first], points[second], points[third])
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
        .expect("a turned pyramid is a sound solid")
}

fn volume_of(model: &Model, solids: &[SolidKey]) -> f64 {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.1).expect("a tenth of a millimetre is a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a flat solid measures")
                .volume()
        })
        .sum()
}

/// A pyramid with a five-sided and a six-sided base — five and six planes
/// round its apex, where Q240 and §310 built only four — is united with a
/// cube clear of it, at every turn and spelling: 36 unions for each.
///
/// A fifth and sixth plane give `bake` more triples to pick a well-conditioned
/// one from, not fewer, so no meet here stuck out past the apex's box by more
/// than the four-plane pyramid's own worst fold did (`RCAD_Q275_PROBE`, the
/// second table above). All 72 answer, validate and match their closed-form
/// volume.
#[test]
fn a_pyramid_with_five_or_six_planes_round_its_apex_is_united() {
    for sides in [5, 6] {
        let base = a_regular_polygon(sides, 1.0);
        for through in SPELLINGS {
            for turn in TURNS {
                for axis in AXES {
                    is_united_with_base(&base, 1.0, [0.0, 0.0, 0.0], through, turn, axis);
                }
            }
        }
    }
}

/// A pyramid whose base brings two neighbouring corners to within `gap` of
/// each other, so the face between them has a sliver of an angle at the
/// apex, is united with a cube clear of it, at every turn and spelling, for
/// `gap` a thousandth, a millionth and a billionth: 108 unions.
///
/// The sliver face's own triple is three planes meeting close to a line (the
/// apex and the two nearly-coincident base corners), the same shape §356's
/// argument folds into the apex's box, and all 108 answer, validate and match
/// their closed-form volume. The furthest meet measured sticks out
/// `RCAD_Q275_PROBE`'s number below, well short of a refusal.
#[test]
fn a_pyramid_with_a_sliver_face_at_its_apex_is_united() {
    for gap in [1e-3, 1e-5, 1e-6] {
        let base = [
            [-1.0, -1.0],
            [1.0, -gap / 2.0],
            [1.0, gap / 2.0],
            [-1.0, 1.0],
        ];
        for through in SPELLINGS {
            for turn in TURNS {
                for axis in AXES {
                    is_united_with_base(&base, 1.0, [0.0, 0.0, 0.0], through, turn, axis);
                }
            }
        }
    }
}

/// The same sliver base, with the two close corners a hundred-millionth
/// apart, refuses to build at all.
///
/// `Plane::through_three_points` fits the sliver face's plane through its
/// first two corners (through the apex, with [`Through::TheApex`]) and then
/// checks the third against it; at this gap the fit cannot keep that third
/// corner within tolerance, so the face it builds is off its own plane. This
/// is a limit of fitting a plane through two points this close together, not
/// of the fold room §356 measures, so it is left as a limit rather than
/// chased further (§16.10).
#[test]
fn a_pyramid_with_too_fine_a_sliver_face_refuses_to_build() {
    let gap = 1e-8;
    let base = [
        [-1.0, -1.0],
        [1.0, -gap / 2.0],
        [1.0, gap / 2.0],
        [-1.0, 1.0],
    ];
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let turned = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("a direction with length"),
        0.3,
    )
    .expect("a finite turn");
    let err = a_pyramid_with_base(
        &mut model,
        &base,
        1.0,
        [0.0, 0.0, 0.0],
        Through::TheApex,
        &turned,
    )
    .expect_err("a sliver gap of a hundred-millionth cannot fit a plane through it");
    assert!(
        matches!(
            err,
            rcad_topo::Error::Invalid(rcad_topo::Invalid::VertexOffItsFacePlane { .. })
        ),
        "expected VertexOffItsFacePlane, got {err}"
    );
}

/// The pyramid folded by a sixteenth (as at the top of this file), shifted so
/// its apex sits a million units from the origin, is united with a cube
/// clear of it, at every turn and spelling: 36 unions.
///
/// [`what_writing_the_corner_down_cost`](rcad_topo) grows with a corner's own
/// coordinates, so the room `bake` grants the apex grows along with it; moving
/// the apex out does not shrink how far a meet can stick out and still fold.
#[test]
fn a_pyramid_whose_apex_is_far_from_the_origin_is_united() {
    let base = [
        [-1.0, -1.0],
        [0.0, -1.0 - 1.0 / 16.0],
        [1.0, -1.0],
        [0.0, 1.0],
    ];
    for through in SPELLINGS {
        for turn in TURNS {
            for axis in AXES {
                is_united_with_base(&base, 1.0, [1e6, 1e6, 1e6], through, turn, axis);
            }
        }
    }
}

/// The same pyramid, shifted so its apex sits a trillion units from the
/// origin, refuses to build at all.
///
/// A trillion is past [`DynamicRange::LARGEST_SUPPORTED_RATIO`](rcad_geom),
/// `1e11`, the ratio of a model's own size to its smallest feature that `f64`
/// can still carry (§3.5). This pyramid's own size is about the apex's
/// distance from the origin and its smallest feature is the millimetre scale
/// the model was declared with, so the ratio crosses `1e11` once the apex
/// passes about a hundred billion units out — a limit this row's family of
/// fixes (a wider room for the fold) cannot move, since it is `Model::new`'s
/// own declared envelope and not the fold's (§16.10).
#[test]
fn a_pyramid_whose_apex_is_far_enough_to_leave_the_scale_envelope_refuses_to_build() {
    let base = [
        [-1.0, -1.0],
        [0.0, -1.0 - 1.0 / 16.0],
        [1.0, -1.0],
        [0.0, 1.0],
    ];
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let turned = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("a direction with length"),
        0.1,
    )
    .expect("a finite turn");
    let err = a_pyramid_with_base(
        &mut model,
        &base,
        1.0,
        [1e12, 1e12, 1e12],
        Through::TheApex,
        &turned,
    )
    .expect_err("a pyramid a trillion units from the origin is past the scale envelope");
    assert!(
        matches!(
            err,
            rcad_topo::Error::Invalid(rcad_topo::Invalid::DynamicRangeTooWide { .. })
        ),
        "expected DynamicRangeTooWide, got {err}"
    );
}

/// `sides` points evenly spaced on a circle of `radius`, wound
/// counter-clockwise: a regular polygon base for
/// [`a_pyramid_with_five_or_six_planes_round_its_apex_is_united`].
fn a_regular_polygon(sides: usize, radius: f64) -> Vec<[f64; 2]> {
    (0..sides)
        .map(|corner| {
            let angle = 2.0 * std::f64::consts::PI * corner as f64 / sides as f64;
            [radius * angle.cos(), radius * angle.sin()]
        })
        .collect()
}

/// A pyramid on an arbitrary polygon `base`, shifted by `shift` and united
/// with a cube clear of it, as [`is_united`] asks of the fixed four-cornered
/// base above.
fn is_united_with_base(
    base: &[[f64; 2]],
    height: f64,
    shift: [f64; 3],
    through: Through,
    turn: f64,
    axis: [f64; 3],
) {
    let (mut model, pyramid, cube) =
        a_pyramid_with_base_and_a_cube_clear_of_it(base, height, shift, through, turn, axis);
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(
            &[pyramid],
            &[cube],
            Rule::Union,
            &Policy::certified(),
            &mut log,
        )
        .unwrap_or_else(|why| {
            panic!(
                "a pyramid on base {base:?}, {height} high, shifted by {shift:?}, with its \
                 planes through {through:?}, turned {turn} about {axis:?}, was refused: {why}"
            )
        });
    assert_eq!(
        fused.solids().len(),
        2,
        "two solids that do not touch unite as the two of them"
    );
    assert!(log.is_empty(), "a certified union recorded {log:?}");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a certified union leaves a valid model");

    let pyramid_volume = a_polygon_area(base) * height / 3.0;
    let united = volume_of(&model, fused.solids());
    assert!(
        (united - (pyramid_volume + 1.0)).abs() <= 1e-9,
        "a pyramid of {pyramid_volume} and a unit cube united to {united}"
    );
}

/// A pyramid on an arbitrary polygon `base`, its apex `height` above the
/// origin and the whole shape shifted by `shift` after it is turned, with a
/// cube clear of it, as [`a_pyramid_and_a_cube_clear_of_it`] builds the fixed
/// four-cornered one.
fn a_pyramid_with_base_and_a_cube_clear_of_it(
    base: &[[f64; 2]],
    height: f64,
    shift: [f64; 3],
    through: Through,
    turn: f64,
    [x, y, z]: [f64; 3],
) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let turned = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(x, y, z)).expect("a direction with length"),
        turn,
    )
    .expect("a finite turn");
    let pyramid = a_pyramid_with_base(&mut model, base, height, shift, through, &turned)
        .expect("a turned pyramid is a sound solid");
    let cube = model
        .add_cube(1.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a unit cube");
    model
        .place_solid(
            cube,
            &Isometry::translation(Vector::new(
                shift[0] + 100.0,
                shift[1] + 100.0,
                shift[2] + 100.0,
            ))
            .expect("a finite shift"),
        )
        .expect("this model made the key");
    (model, pyramid, cube)
}

/// A pyramid with an arbitrary polygon `base`, apex `height` above the
/// origin, described face by face in turned coordinates and shifted by
/// `shift` once turned. The base's points are wound counter-clockwise seen
/// from above, in the plane `z = 0`; `base.len()` side planes meet at the
/// apex.
///
/// `Err` when the shape itself is sound but the model it was written into
/// refuses it — [`a_pyramid_whose_apex_is_far_enough_to_leave_the_scale_envelope_refuses_to_build`]
/// reads this to pin a refusal by name rather than letting it panic.
fn a_pyramid_with_base(
    model: &mut Model,
    base: &[[f64; 2]],
    height: f64,
    shift: [f64; 3],
    through: Through,
    turned: &Isometry,
) -> rcad_topo::Result<SolidKey> {
    let sides = base.len();
    let mut raw = vec![[0.0, 0.0, height]];
    raw.extend(base.iter().map(|&[x, y]| [x, y, 0.0]));
    let points: Vec<Point> = raw
        .into_iter()
        .map(|[x, y, z]| {
            let turned_point = turned.apply_to_point(Point::new(x, y, z));
            let [tx, ty, tz] = turned_point.coordinates();
            Point::new(tx + shift[0], ty + shift[1], tz + shift[2])
        })
        .collect();
    // Counter-clockwise seen from outside, the apex first on every side face,
    // the base face last and wound the other way round.
    let mut faces: Vec<Vec<usize>> = (0..sides)
        .map(|side| vec![0, side + 1, (side + 1) % sides + 1])
        .collect();
    faces.push((1..=sides).rev().collect());

    let mut builder = SolidBuilder::new();
    let vertices: Vec<_> = points
        .iter()
        .map(|&point| builder.add_vertex(point).expect("a finite corner"))
        .collect();
    let mut edges: Vec<([usize; 2], DraftEdge)> = Vec::new();
    let drafted: Vec<_> = faces
        .iter()
        .map(|face| {
            let steps: Vec<DraftStep> = (0..face.len())
                .map(|position| {
                    let (from, to) = (face[position], face[(position + 1) % face.len()]);
                    let (edge, orientation) =
                        match edges.iter().find(|(ends, _)| *ends == [to, from]) {
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
            let [first, second, third] = match (through, face.len()) {
                (Through::ABaseCorner, 3) => [face[1], face[2], face[0]],
                _ => [face[0], face[1], face[2]],
            };
            let plane = Plane::through_three_points(points[first], points[second], points[third])
                .expect("three corners of a face not in a line");
            builder
                .add_face(Surface::Plane(plane), &[wire])
                .expect("a face of a pyramid")
        })
        .collect();
    let shell: Vec<_> = drafted
        .iter()
        .map(|&face| (face, Orientation::Forward))
        .collect();
    builder.add_shell(&shell).expect("a pyramid's faces close");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The area of an arbitrary simple polygon `base`, by the shoelace formula,
/// for [`is_united_with_base`]'s closed-form volume check.
fn a_polygon_area(base: &[[f64; 2]]) -> f64 {
    let sides = base.len();
    0.5 * (0..sides)
        .map(|corner| {
            let [x1, y1] = base[corner];
            let [x2, y2] = base[(corner + 1) % sides];
            x1 * y2 - x2 * y1
        })
        .sum::<f64>()
        .abs()
}
