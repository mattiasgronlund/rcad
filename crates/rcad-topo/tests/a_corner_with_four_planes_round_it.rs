//! A pyramid's apex, where four planes meet, and a face's own meet folded into
//! it (`docs/decisions.md` §301.4, §301.7).
//!
//! `bake` solves a corner with four planes round it from the best-conditioned
//! of its four triples. `place_the_meets` asks each side face where its *own*
//! triple meets: the face and its two neighbours. At a pyramid's apex the four
//! triples are the four faces' own, so one face asks about the triple `bake`
//! used and three ask about another. The fold takes a face's meet as the
//! corner only when every point of the meet's box lies inside the corner's box
//! grown by the room writing the corner down cost (§253).
//!
//! **Bend the pyramid's base at one corner, and the two side faces there fold
//! by only a little.** Both triples holding those two faces are then three
//! planes nearly through one line, so the solve gives a wide box, while `bake`
//! picks one of the other two triples and its box stays narrow. So those two
//! faces' meets stick out past the apex's box, by roughly the inverse of the
//! fold. Measured with §301.6's probe, a scratch edit to `a_corner_is_this_place`,
//! over 108 certified unions per fold, each pyramid with a cube clear of it:
//! heights `1/4`, `1` and `4`, each side plane written through the apex or
//! through a base corner, and the 18 turns below.
//!
//! | Fold | Furthest an apex meet sticks out, in rooms | Answered | Room ×0.5 | Room ×1000 |
//! |---|---|---|---|---|
//! | 1 | 0.12 | 108 | 108 | 108 |
//! | 1/4 | 0.24 | 103 | 103 | 103 |
//! | 1/16 | 1.17 | 99 | 31 | 103 |
//! | 1/64 | 4.7 | 0 | 0 | 107 |
//! | 1/256 | 21 | 0 | 0 | 105 |
//! | 1/4096 | 290 | 0 | 0 | 97 |
//!
//! The apex's own box stayed under 0.18 rooms wide at every fold. The few
//! refused at ×1000, and at a fold of `1/4`, had every apex meet folded, so
//! they are refused somewhere else, and they were not traced.
//!
//! **So a four-plane corner does fill the room**, which none of §301's shapes
//! did. The first test is the pin between the thin wedges' hundredth and the
//! turned-pair rate test at zero: a room of half fails it. The second is where
//! the room runs out, and is a gap rather than a limit.
//!
//! Not reached here: a corner with five or more planes round it, a side face
//! whose own angle at the apex is a sliver, and an apex far enough from the
//! origin that the room grows with it.

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftEdge, DraftStep, Error, Model, Rule, SolidBuilder, SolidKey};

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
/// folded. With the room ×0.5 in a scratch edit to `a_corner_is_this_place`, 31
/// of the 36 are refused.
#[test]
fn a_pyramid_whose_two_faces_fold_by_a_sixteenth_is_united_though_its_apex_meets_stick_out_past_half_the_room()
 {
    let (fold, height) = (1.0 / 16.0, 1.0);
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

/// The same pyramid bent by a sixty-fourth is refused at every one of the 36,
/// because its apex meets stick out 1.6 to 3.7 rooms past the apex's box and
/// the fold does not take them. The meet stays a node beside the apex,
/// overlapping it, and the certified policy cannot order the two along the
/// face's edge.
///
/// **This is a gap, not a limit** (§16.10). The answer exists — two solids that
/// do not touch — and with the room ×1000 in a scratch edit all 36 give it.
/// What stops them is that the corner's box and the meet's box come from two
/// different triples of the corner's own planes. This pins today's refusal, so
/// the change that closes the gap has to turn this test round.
#[test]
fn a_pyramid_whose_two_faces_fold_by_a_sixty_fourth_is_refused_because_its_apex_meets_stick_out_past_the_room()
 {
    let (fold, height) = (1.0 / 64.0, 1.0);
    for through in SPELLINGS {
        for turn in TURNS {
            for axis in AXES {
                let (mut model, pyramid, cube) =
                    a_pyramid_and_a_cube_clear_of_it(fold, height, through, turn, axis);
                let refused = model.fuse(
                    &[pyramid],
                    &[cube],
                    Rule::Union,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                let records = match refused {
                    Err(Error::Undecidable { records }) => records,
                    Ok(fused) => panic!(
                        "a pyramid folding {fold} with its planes through {through:?}, turned \
                         {turn} about {axis:?}, was united into {} solids: the gap is closed, so \
                         turn this test round",
                        fused.solids().len()
                    ),
                    Err(why) => panic!(
                        "a pyramid folding {fold} with its planes through {through:?}, turned \
                         {turn} about {axis:?}, was refused for another reason: {why:?}"
                    ),
                };
                assert!(
                    records
                        .records()
                        .iter()
                        .any(|record| record.predicate() == "order along a crossing"),
                    "a meet left beside its corner is refused as two points along an edge that \
                     cannot be ordered, and this refusal named {records:?}"
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
