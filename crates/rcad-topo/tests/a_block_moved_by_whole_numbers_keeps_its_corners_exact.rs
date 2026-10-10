//! A block moved by whole numbers keeps its corners exact, so it fuses as it
//! does unmoved.
//!
//! A corner the fuse reads out of the model, rather than solving for, is moved
//! into place by `rcad_geom::where_a_corner_lands_after_moving`. That used to
//! hand back a box of `CORNER_AFTER_MOVING_ROUNDOFF` times the size of the move
//! whatever the move was; since Q509 a move that rounds nothing hands back the
//! exact point.
//!
//! **Where three of a block's planes meet, the box never changed an answer.**
//! Such a corner is solved from its planes, and the corner read out of the
//! model only picks which of the places they meet at is this one. Three planes
//! meet at one place at most, and a wider box picks whatever a narrower box
//! inside it picks, so the box could only ever accept more. The sweep below
//! runs the cube through every partner, shift and rule with and without the
//! exact road and gets the same 216 answers.
//!
//! **Where only two of its planes meet, the box was the corner.** A straight
//! edge with a corner halfway along it, which a block described face by face
//! can have, has two planes round that corner and nothing to solve, so the
//! fuse reads the corner, moves it, and asks every plane of the partner about
//! it. With the box, a partner wall through the corner's `x` answered
//! *undecided*, and the fuse refused with `Error::Undecidable` where the
//! unmoved block answers. With the exact corner it answers as unmoved.
//!
//! This is a gap and not a limit (§16.10): every corner here is a whole number,
//! and only the box hid it.
//!
//! **The corner halfway is a point on the edge, not a different solid.** United
//! with a box that shares the front bottom edge and nothing else, the cube is
//! refused `ResultIsNotManifold`: the two touch along a line that no one solid
//! holds, and §80.1 and §80.9 keep that refusal. The block with the corner
//! halfway was answered as two solids instead, because its edge stopped at the
//! corner and the other box's edge did not, so no edge had four faces round
//! it. Now the fuse reads past a corner that every face round it walks
//! straight past, and both blocks are refused (Q1568).

#![forbid(unsafe_code)]

use core::mem::{Discriminant, discriminant};

use rcad_geom::{Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftEdge, DraftStep, DraftVertex, Error, Model, Rule, SolidBuilder, SolidKey};

/// The block with a corner halfway along its front bottom edge, moved by
/// `(-7, 11, 5)`, against a partner whose wall `x = 1` runs through that
/// corner: every rule both ways round answers the volumes the unmoved pair
/// has.
///
/// The block is `[0, 2]³` and the partner `[1, 3] × [-1, 1] × [-1, 1]` before
/// the move, so they share `[1, 2] × [0, 1] × [0, 1]`, of volume 1. With the
/// box every one of these six was refused `Error::Undecidable`.
#[test]
fn a_block_with_a_corner_halfway_along_an_edge_moved_by_whole_numbers_fuses_as_unmoved() {
    let shift = [-7.0, 11.0, 5.0];
    let rows = [
        (Rule::Union, true, 15.0),
        (Rule::Union, false, 15.0),
        (Rule::Intersection, true, 1.0),
        (Rule::Intersection, false, 1.0),
        (Rule::Difference, true, 7.0),
        (Rule::Difference, false, 7.0),
    ];
    for (rule, block_first, volume) in rows {
        let (mut model, block, partner) = a_block_and_a_partner(
            Block::CornerHalfwayAlongAnEdge,
            WALL_THROUGH_THE_CORNER_HALFWAY,
            shift,
        );
        let (one, other) = if block_first {
            (block, partner)
        } else {
            (partner, block)
        };
        let mut log = ProvenanceLog::new();
        let fused = model
            .fuse(&[one], &[other], rule, &Policy::certified(), &mut log)
            .unwrap_or_else(|why| panic!("{rule:?}, block first {block_first}: {why:?}"));
        assert!(
            log.records().is_empty(),
            "{rule:?}: every decision is proved, got {:?}",
            log.records()
        );
        let [solid] = fused.solids() else {
            panic!("{rule:?}, block first {block_first}: one solid, got {fused:?}");
        };
        let found = model
            .solid(*solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.1).expect("a tenth is a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a box measures")
            .volume();
        assert!(
            (found - volume).abs() <= 1e-12 * volume,
            "{rule:?}, block first {block_first}: measured {found}, the closed form is {volume}"
        );
    }
}

/// Every fuse of a block moved by whole numbers answers what the same fuse of
/// the unmoved block answers: the same number of solids, or the same refusal.
///
/// Two blocks, six partners, six shifts, three rules and both ways round, 432
/// fuses. The cube has three planes at every corner and the box never touched
/// it; the block with a corner halfway along an edge had 192 of its 216 fuses
/// refused `Error::Undecidable` with the box: all 36 at each of the four
/// larger shifts, and at the two smallest the 24 against the four partners
/// with a wall at `x = 1`, through that corner. Each is checked against the
/// unmoved fuse rather than a count, so a shift that changed any answer at all
/// would show.
///
/// The block with a corner halfway is refused only where the cube is: united
/// with the box that shares its front bottom edge and nothing else, 12 fuses
/// over six shifts and both ways round, refused `ResultIsNotManifold` as §80.1
/// and §80.9 keep it (Q1568).
///
/// The shifts stop at `2^36`, because the model refuses a coordinate more than
/// `1e11` times its shortest edge (`Error::Invalid` of `DynamicRangeTooWide`),
/// and the block's shortest edge is 1.
#[test]
fn every_fuse_of_a_block_moved_by_whole_numbers_answers_as_the_unmoved_block_does() {
    let sharing_one_edge = PARTNERS[3];
    let shifts = [
        [1.0, 0.0, 0.0],
        [-7.0, 11.0, 5.0],
        [1024.0, -3.0, 1_048_576.0],
        [2f64.powi(36), 0.0, 0.0],
        [2f64.powi(36) - 1.0, 5.0, -2f64.powi(36)],
        [-3.0, 2f64.powi(30), -2f64.powi(20)],
    ];
    let mut compared = 0;
    let mut differ = Vec::new();
    let mut refused_sharing_one_edge = 0;
    let mut refused_otherwise = Vec::new();
    for block in [Block::Cube, Block::CornerHalfwayAlongAnEdge] {
        for partner in PARTNERS {
            for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
                for block_first in [true, false] {
                    let unmoved = fused(block, partner, [0.0; 3], rule, block_first);
                    for shift in shifts {
                        let moved = fused(block, partner, shift, rule, block_first);
                        compared += 1;
                        let case = format!(
                            "{block:?} with the partner {}, moved by {shift:?}, {rule:?}, block \
                             first {block_first}",
                            partner.name
                        );
                        if moved.0 != unmoved.0 {
                            differ.push(format!("{case}: {} where unmoved {}", moved.1, unmoved.1));
                        }
                        if block == Block::CornerHalfwayAlongAnEdge && moved.0.is_err() {
                            if rule == Rule::Union && partner.name == sharing_one_edge.name {
                                refused_sharing_one_edge += 1;
                            } else {
                                refused_otherwise.push(format!("{case}: {}", moved.1));
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(compared, 432);
    assert!(
        differ.is_empty(),
        "{} of {compared} fuses answer otherwise once moved: {differ:#?}",
        differ.len()
    );
    assert!(
        refused_otherwise.is_empty(),
        "the block with a corner halfway along an edge is refused where the cube is not: \
         {refused_otherwise:#?}"
    );
    assert_eq!(
        refused_sharing_one_edge, 12,
        "the union with {} is refused at every shift, both ways round",
        sharing_one_edge.name
    );
}

/// A box that shares the block's front bottom edge and nothing else fuses the
/// same with the cube and with the block that has a corner halfway along that
/// edge, under every rule and both ways round: the corner is a point on the
/// edge and not a different solid.
///
/// The union is refused `ResultIsNotManifold`. Two solids that touch along an
/// edge of both share a line that no single solid holds, and §80.1 and §80.9
/// keep that refusal. The block with the corner halfway was answered as two
/// solids, because its edge stopped at the corner and the partner's did not,
/// so no edge had four faces round it (Q1568).
#[test]
fn a_box_sharing_one_edge_fuses_the_same_whether_the_edge_has_a_corner_halfway_or_not() {
    let sharing_one_edge = PARTNERS[3];
    assert_eq!(
        sharing_one_edge.name,
        "sharing the front bottom edge and nothing else"
    );
    let not_a_solid = discriminant(&Error::ResultIsNotManifold { count: 4 });
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for block_first in [true, false] {
            let cube = fused(Block::Cube, sharing_one_edge, [0.0; 3], rule, block_first);
            let halfway = fused(
                Block::CornerHalfwayAlongAnEdge,
                sharing_one_edge,
                [0.0; 3],
                rule,
                block_first,
            );
            assert_eq!(
                halfway.0, cube.0,
                "{rule:?}, block first {block_first}: the block with a corner halfway {}, \
                 the cube {}",
                halfway.1, cube.1
            );
            if rule == Rule::Union {
                assert_eq!(
                    halfway.0,
                    Err(not_a_solid),
                    "block first {block_first}: {}",
                    halfway.1
                );
            }
        }
    }
}

/// The block that is moved: a cube as a designer builds one, or the same cube
/// described face by face with a corner halfway along its front bottom edge,
/// where only the front face and the bottom face meet.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Block {
    Cube,
    CornerHalfwayAlongAnEdge,
}

/// A partner as its low and high corners, in the frame of the block `[0, 2]³`,
/// moved with it.
#[derive(Clone, Copy)]
struct Partner {
    name: &'static str,
    low: [f64; 3],
    high: [f64; 3],
}

/// Its wall `x = 1` runs through the corner halfway along the edge.
const WALL_THROUGH_THE_CORNER_HALFWAY: Partner = Partner {
    name: "wall through the corner halfway",
    low: [1.0, -1.0, -1.0],
    high: [3.0, 1.0, 1.0],
};

const PARTNERS: [Partner; 6] = [
    WALL_THROUGH_THE_CORNER_HALFWAY,
    Partner {
        name: "flush on the front face",
        low: [-1.0, -2.0, 0.0],
        high: [3.0, 0.0, 2.0],
    },
    Partner {
        name: "over the far top corner",
        low: [1.0, 1.0, 1.0],
        high: [3.0, 3.0, 3.0],
    },
    Partner {
        name: "sharing the front bottom edge and nothing else",
        low: [0.0, -2.0, -2.0],
        high: [2.0, 0.0, 0.0],
    },
    Partner {
        name: "the right half of the block",
        low: [1.0, 0.0, 0.0],
        high: [2.0, 2.0, 2.0],
    },
    Partner {
        name: "through the front face, up from the corner halfway",
        low: [1.0, -1.0, 0.0],
        high: [3.0, 1.0, 2.0],
    },
];

/// What one fuse answered: how many solids, or which refusal, and how to say
/// it.
type Answer = (Result<usize, Discriminant<Error>>, String);

fn fused(block: Block, partner: Partner, shift: [f64; 3], rule: Rule, block_first: bool) -> Answer {
    let (mut model, block, partner) = a_block_and_a_partner(block, partner, shift);
    let (one, other) = if block_first {
        (block, partner)
    } else {
        (partner, block)
    };
    let mut log = ProvenanceLog::new();
    match model.fuse(&[one], &[other], rule, &Policy::certified(), &mut log) {
        Ok(fused) => {
            assert!(
                log.records().is_empty(),
                "an answered fuse of whole numbers proved every decision, got {:?}",
                log.records()
            );
            let solids = fused.solids().len();
            (Ok(solids), format!("{solids} solids"))
        }
        Err(why) => (Err(discriminant(&why)), format!("refused {why:?}")),
    }
}

/// The block, built where it was described and then moved by `shift`, and the
/// partner, described where the move puts it and not moved at all, so that
/// only the block's corners are read out of the model and moved.
fn a_block_and_a_partner(
    block: Block,
    partner: Partner,
    shift: [f64; 3],
) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let block = match block {
        Block::Cube => model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube of side 2"),
        Block::CornerHalfwayAlongAnEdge => a_box_face_by_face(
            &mut model,
            [0.0; 3],
            [2.0; 3],
            Halfway::OnTheFrontBottomEdge,
        ),
    };
    if shift != [0.0; 3] {
        let movement = Isometry::translation(Vector::new(shift[0], shift[1], shift[2]))
            .expect("a finite shift");
        model
            .place_solid(block, &movement)
            .expect("this model's key");
    }
    let moved = |at: [f64; 3]| [0, 1, 2].map(|axis| at[axis] + shift[axis]);
    let partner = a_box_face_by_face(
        &mut model,
        moved(partner.low),
        moved(partner.high),
        Halfway::Nowhere,
    );
    (model, block, partner)
}

#[derive(Clone, Copy, PartialEq)]
enum Halfway {
    Nowhere,
    OnTheFrontBottomEdge,
}

/// The box from `low` to `high`, described face by face where it stands, every
/// face's outline walked anticlockwise round its outward normal.
fn a_box_face_by_face(
    model: &mut Model,
    low: [f64; 3],
    high: [f64; 3],
    halfway: Halfway,
) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let [x0, y0, z0] = low;
    let [x1, y1, z1] = high;
    let mut corners: Vec<DraftVertex> = Vec::new();
    let mut corner = |builder: &mut SolidBuilder, x: f64, y: f64, z: f64| {
        corners.push(builder.add_vertex(Point::new(x, y, z)).expect("a corner"));
        corners.len() - 1
    };
    let v0 = corner(&mut builder, x0, y0, z0);
    let v1 = corner(&mut builder, x1, y0, z0);
    let v2 = corner(&mut builder, x1, y1, z0);
    let v3 = corner(&mut builder, x0, y1, z0);
    let v4 = corner(&mut builder, x0, y0, z1);
    let v5 = corner(&mut builder, x1, y0, z1);
    let v6 = corner(&mut builder, x1, y1, z1);
    let v7 = corner(&mut builder, x0, y1, z1);
    let (front, bottom) = match halfway {
        Halfway::Nowhere => (vec![v0, v1, v5, v4], vec![v0, v3, v2, v1]),
        Halfway::OnTheFrontBottomEdge => {
            let middle = corner(&mut builder, (x0 + x1) / 2.0, y0, z0);
            (vec![v0, middle, v1, v5, v4], vec![v0, v3, v2, v1, middle])
        }
    };
    let faces = [
        (bottom, [0.0, 0.0, -1.0], low),
        (vec![v4, v5, v6, v7], [0.0, 0.0, 1.0], high),
        (front, [0.0, -1.0, 0.0], low),
        (vec![v3, v7, v6, v2], [0.0, 1.0, 0.0], high),
        (vec![v0, v4, v7, v3], [-1.0, 0.0, 0.0], low),
        (vec![v1, v2, v6, v5], [1.0, 0.0, 0.0], high),
    ];
    let mut edges: Vec<(usize, usize, DraftEdge)> = Vec::new();
    let mut shell = Vec::new();
    for (round, [nx, ny, nz], [px, py, pz]) in faces {
        let mut steps = Vec::new();
        for (position, &from) in round.iter().enumerate() {
            let to = round[(position + 1) % round.len()];
            let walked_back = edges
                .iter()
                .find(|&&(start, end, _)| (start, end) == (to, from))
                .map(|&(_, _, edge)| edge);
            let (edge, way) = match walked_back {
                Some(edge) => (edge, Orientation::Reversed),
                None => {
                    let edge = builder
                        .add_edge(corners[from], corners[to], EdgeRun::Straight)
                        .expect("an edge");
                    edges.push((from, to, edge));
                    (edge, Orientation::Forward)
                }
            };
            steps.push(DraftStep::Along(
                builder.add_coedge(edge, way).expect("a coedge"),
            ));
        }
        let wire = builder.add_wire(&steps).expect("a wire");
        let plane = Plane::from_point_and_normal(Point::new(px, py, pz), Vector::new(nx, ny, nz))
            .expect("an axis is a unit normal");
        let face = builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a face");
        shell.push((face, Orientation::Forward));
    }
    builder.add_shell(&shell).expect("a shell");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a box of whole numbers validates")
}
