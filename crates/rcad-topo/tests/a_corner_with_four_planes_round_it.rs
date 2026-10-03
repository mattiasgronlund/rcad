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
//! planes `bake` solved the corner among however far it reaches. "With §395"
//! drops the test that came before both: whether the meet's box touches the
//! corner's at all.
//!
//! | Fold | Meet's box sticks out | Exact meet sticks out | §253 alone | §253 alone, room ×0.5 | §253 alone, room ×1000 | With §356 | With §395 |
//! |---|---|---|---|---|---|---|---|
//! | 1 | 0.12 | 0.04 | 108 | 108 | 108 | 108 | 108 |
//! | 1/4 | 0.24 | 0.12 | 103 | 103 | 103 | 103 | 108 |
//! | 1/16 | 1.17 | 0.46 | 99 | 31 | 103 | 103 | 108 |
//! | 1/64 | 4.7 | 1.6 | 0 | 0 | 107 | 107 | 108 |
//! | 1/256 | 21 | 6.9 | 0 | 0 | 105 | 107 | 107 |
//! | 1/4096 | 290 | 108 | 0 | 0 | 97 | 104 | 104 |
//!
//! The first six columns are §356.1's, on `9612b25f`. "With §356" was 105 and
//! 97 at the two smallest folds when that rule landed. It is re-measured on
//! `25860052`, where `bake` no longer refuses the base corner the two folding
//! faces share, and "with §395" is the same table on this file's commit
//! (§395.2).
//!
//! **This file used to say the refusals left with §356 had every apex meet
//! folded. They did not** (§344.2, §344.4). A side face has a meet at the apex
//! for each pair of the other three side planes, and `bake` solved the apex
//! from one triple of the four. The other meets' boxes can lie inside the
//! apex's room and still miss its own box, by up to four tenths of a room, and
//! the touch test turned them away before the room was looked at. Left beside
//! the apex, two of them could not be ordered along a crossing, or the cells
//! they cut off round it wound outside 0 and 1. The 5 left — 1 at a 256th and
//! 4 at a 4096th — are apex meets of three of the apex's own planes whose boxes
//! miss the apex's own box by 1.3 to 60 rooms, so its grown box too. §356's
//! argument would fold those too, and does answer all 648 when it is followed
//! (§395.4).
//!
//! **So a four-plane corner does fill the room, and the room is not what it
//! should be measured in.** Every plane round the apex passes through the
//! apex's box grown by the room, which `bake` checks; carried through a triple
//! of nearly-flat planes, that room grows by about one over the fold. The
//! sixty-fourth was a gap, not a limit, and is closed.
//!
//! Not reached here: a corner with five or more planes round it, a side face
//! whose own angle at the apex is a sliver, and an apex far enough from the
//! origin that the room grows with it.

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
/// meet of three planes `bake` solved the apex among, which folds whatever the
/// room, so this no longer holds the room from below; the hand-written test
/// beside `a_corner_is_this_place` does (§301.5).
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

/// The pyramid bent by a quarter, by a sixteenth and by a sixty-fourth, at
/// heights `1/4`, `1` and `4`, is united at every turn and spelling: 108
/// unions for each bend.
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
    for fold in [1.0 / 4.0, 1.0 / 16.0, 1.0 / 64.0] {
        for height in [1.0 / 4.0, 1.0, 4.0] {
            every_turn_and_spelling_is_united(fold, height);
        }
    }
}

/// Five pyramids bent by a 256th or a 4096th are still refused. Their apex
/// meets of three of the apex's own planes have boxes that miss the apex's
/// own box by 1.3 to 60 rooms, so its box grown by the room too, and are left
/// nodes beside it (§395.4).
///
/// **The answer exists, so this is a gap.** §356's argument, that three planes
/// meet once, folds these meets as well, and folding them answers every one
/// of the module table's 648 unions with the two solids, their volumes and an
/// empty log. When one of these is united, check that it is united like that,
/// and move it to the tests above.
#[test]
fn a_pyramid_whose_apex_meets_miss_even_its_room_is_still_refused() {
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
        (1.0 / 4096.0, 4.0, Through::TheApex, 0.3, [3.0, -1.0, 2.0]),
    ] {
        let (mut model, pyramid, cube) =
            a_pyramid_and_a_cube_clear_of_it(fold, height, through, turn, axis);
        let fused = model.fuse(
            &[pyramid],
            &[cube],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert!(
            fused.is_err(),
            "a pyramid folding {fold}, {height} high, with its planes through {through:?}, \
             turned {turn} about {axis:?}, was united: if into the two solids with the right \
             volume and an empty log, apex meets outside the room are folded now, and it \
             belongs with the pyramids that unite"
        );
    }
}

/// A pyramid bent by `fold`, `height` high, is united with a cube clear of it
/// at every turn and spelling, under a certified policy, into the two of them
/// with their volumes to a billionth and an empty log.
fn every_turn_and_spelling_is_united(fold: f64, height: f64) {
    for through in SPELLINGS {
        for turn in TURNS {
            for axis in AXES {
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
                            "a pyramid folding {fold} with its planes through {through:?}, turned \
                             {turn} about {axis:?}, was refused: {why}"
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
        }
    }
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
