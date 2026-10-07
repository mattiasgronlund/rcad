//! A cube turned so its corner points into another cube, with that corner
//! placed exactly on the other cube's face.
//!
//! The block is `add_cube(10.0)`, `[0, 10]³`. The turned cube is a second
//! `add_cube(10.0)` turned 1.5 about the axis through the origin along
//! `(1, 1, 0)`, which leans all three of its edges at that corner towards
//! `+x`, and moved so the corner is at `(0, 3, 3)`, on the block's `x = 0`
//! face. Its three planes there do not meet on the face: worked out in
//! rationals they meet at `x = −1.64e−16`, a rounding outside it, so the
//! three edges cross `x = 0` a rounding from the corner and from each other,
//! and the arrangement cuts that tiny triangle out of the block's face.
//!
//! **Six of the twelve fuses here panicked** in `rings_round`
//! (`fuse/assemble.rs`) under both policies: the union both ways round and
//! the block less the turned cube (Q1287). The block's `x = 0` face, joined
//! back up round the triangle, has the triangle as a hole, and the hole's
//! stored corners, a rounding apart, wind the same way as the outline. Two
//! rings claiming to be the outline was an assert; it is now
//! `TheFuseWroteABrokenAnswer { broken: HoleWindsWithTheOuterWire }`, which is
//! what `Model::validate` would say of that face if it were written. The
//! other six were refused by name already and still are. Every test below
//! pins one fuse to its refusal.
//!
//! **Under `Policy::certified()` the refusal is a limit, but not for the
//! reason the windings suggest.** Rounding each coordinate of the tip and of
//! the three crossings down or up, 256 ways (one coordinate is exact), 78 make
//! the hole and the three triangles of the tip all wind the right way, so the
//! windings alone are a gap. But in every one of the 78 the shortest edge is
//! at most `4.7e−16`, beside a corner of the block at 10: a range of at least
//! `2.1e16` against the `1e11` a certified fuse fails on
//! (`DynamicRange::LARGEST_SUPPORTED_RATIO`). `Assembly::check_dynamic_range`
//! says why that refusal is a limit — a length over a length is the same at
//! every scale — and it is what the intersection here is refused for. The
//! same count on Q1208's shape below: 160 of 512, every one with a range of
//! at least `1.4e17`.
//!
//! **Under `Policy::tolerant(100.0)` it is a gap.** The answer meant is the
//! block with its `x = 0` face whole, with the turned cube touching it from
//! outside; it exists, and Q1288 owns finding it. When it does, these pins
//! move to volumes.
//!
//! With the corner a thousandth inside the face, every fuse answers, and the
//! union and the intersection hold both cubes between them:
//! [`with_the_corner_a_thousandth_inside_the_union_and_intersection_hold_both_cubes`].

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, Rule, SolidBuilder, SolidKey};

/// The corner of the turned cube, on the block's `x = 0` face.
const ON_THE_FACE: [f64; 3] = [0.0, 3.0, 3.0];

#[test]
fn the_block_united_with_the_turned_cube_is_refused_certified() {
    assert_refused(
        Rule::Union,
        Order::BlockFirst,
        &certified(),
        Refused::AHoleWindingWithItsOutline,
    );
}

#[test]
fn the_block_united_with_the_turned_cube_is_refused_tolerant() {
    assert_refused(
        Rule::Union,
        Order::BlockFirst,
        &tolerant(),
        Refused::AHoleWindingWithItsOutline,
    );
}

#[test]
fn the_turned_cube_united_with_the_block_is_refused_certified() {
    assert_refused(
        Rule::Union,
        Order::TurnedFirst,
        &certified(),
        Refused::AHoleWindingWithItsOutline,
    );
}

#[test]
fn the_turned_cube_united_with_the_block_is_refused_tolerant() {
    assert_refused(
        Rule::Union,
        Order::TurnedFirst,
        &tolerant(),
        Refused::AHoleWindingWithItsOutline,
    );
}

#[test]
fn the_block_less_the_turned_cube_is_refused_certified() {
    assert_refused(
        Rule::Difference,
        Order::BlockFirst,
        &certified(),
        Refused::AHoleWindingWithItsOutline,
    );
}

#[test]
fn the_block_less_the_turned_cube_is_refused_tolerant() {
    assert_refused(
        Rule::Difference,
        Order::BlockFirst,
        &tolerant(),
        Refused::AHoleWindingWithItsOutline,
    );
}

#[test]
fn the_turned_cube_less_the_block_is_refused_certified() {
    assert_refused(
        Rule::Difference,
        Order::TurnedFirst,
        &certified(),
        Refused::NoSpaceInside,
    );
}

#[test]
fn the_turned_cube_less_the_block_is_refused_tolerant() {
    assert_refused(
        Rule::Difference,
        Order::TurnedFirst,
        &tolerant(),
        Refused::NoSpaceInside,
    );
}

#[test]
fn the_block_and_the_turned_cube_intersected_are_refused_certified() {
    assert_refused(
        Rule::Intersection,
        Order::BlockFirst,
        &certified(),
        Refused::TooWideARange,
    );
}

#[test]
fn the_block_and_the_turned_cube_intersected_are_refused_tolerant() {
    assert_refused(
        Rule::Intersection,
        Order::BlockFirst,
        &tolerant(),
        Refused::AnOutlineWindingAgainstItsFace,
    );
}

#[test]
fn the_turned_cube_and_the_block_intersected_are_refused_certified() {
    assert_refused(
        Rule::Intersection,
        Order::TurnedFirst,
        &certified(),
        Refused::TooWideARange,
    );
}

#[test]
fn the_turned_cube_and_the_block_intersected_are_refused_tolerant() {
    assert_refused(
        Rule::Intersection,
        Order::TurnedFirst,
        &tolerant(),
        Refused::AnOutlineWindingAgainstItsFace,
    );
}

/// The same pair with the corner a thousandth inside the block's face, where
/// nothing is a rounding from anything else.
///
/// The faces of the union and of the intersection, taken together, are the
/// faces of the two cubes cut up, so the two volumes add up to 2000. They add
/// up to it but for the corners the fuse made, which are rounded: moving a
/// corner by `d` moves a volume by at most `√3·d` times the faces round it,
/// the faces of both answers come to the cubes' 1200, and every corner here is
/// within 17 of the origin, so allowing each corner four roundings at 16 is
/// `1200 · √3 · 4 · 16ε` — about `3e−11`, and a missing or doubled piece of
/// either answer is many orders more.
#[test]
fn with_the_corner_a_thousandth_inside_the_union_and_intersection_hold_both_cubes() {
    let inside = [1e-3, ON_THE_FACE[1], ON_THE_FACE[2]];
    let allowed = 1200.0 * 3.0_f64.sqrt() * 4.0 * 16.0 * f64::EPSILON;
    for policy in [certified(), tolerant()] {
        for order in [Order::BlockFirst, Order::TurnedFirst] {
            let asked = format!("{order:?}, {policy:?}");
            let union = volume_of_the_answer(inside, Rule::Union, order, &policy, &asked);
            let intersection =
                volume_of_the_answer(inside, Rule::Intersection, order, &policy, &asked);
            assert!(
                (union + intersection - 2000.0).abs() <= allowed,
                "{asked}: the union {union} and the intersection {intersection} hold \
                 {} between them, not 2000",
                union + intersection
            );
        }
    }
}

/// Q1208's crash, the turned cube of `crash-45d4e616…` of the night of
/// 2026-10-06, read off the sample and described through `SolidBuilder`: its
/// eight corners as stored, with the one at `(2.44e−149, 5, 7.63e−5)` put at
/// `x = 0`, and its six planes through `Plane::from_coefficients`, which
/// moves four of them by an ulp. Its three planes at that corner meet at
/// `x = −6.8e−17`. Every fuse is refused just as the turned cube above is.
///
/// The same cube made by turning `add_cube(10.0)` with the turn its corners
/// say, about `(0.0381, 0.0379, −1.098)` by 0.582, is not this shape: its
/// planes differ in the last bits, and none of its fuses panicked.
#[test]
fn the_cube_of_q1208s_crash_is_refused_by_name() {
    let table = [
        (
            Rule::Union,
            Order::BlockFirst,
            true,
            Refused::AHoleWindingWithItsOutline,
        ),
        (
            Rule::Union,
            Order::BlockFirst,
            false,
            Refused::AHoleWindingWithItsOutline,
        ),
        (
            Rule::Union,
            Order::TurnedFirst,
            true,
            Refused::AHoleWindingWithItsOutline,
        ),
        (
            Rule::Union,
            Order::TurnedFirst,
            false,
            Refused::AHoleWindingWithItsOutline,
        ),
        (
            Rule::Difference,
            Order::BlockFirst,
            true,
            Refused::AHoleWindingWithItsOutline,
        ),
        (
            Rule::Difference,
            Order::BlockFirst,
            false,
            Refused::AHoleWindingWithItsOutline,
        ),
        (
            Rule::Difference,
            Order::TurnedFirst,
            true,
            Refused::NoSpaceInside,
        ),
        (
            Rule::Difference,
            Order::TurnedFirst,
            false,
            Refused::NoSpaceInside,
        ),
        (
            Rule::Intersection,
            Order::BlockFirst,
            true,
            Refused::TooWideARange,
        ),
        (
            Rule::Intersection,
            Order::BlockFirst,
            false,
            Refused::AnOutlineWindingAgainstItsFace,
        ),
        (
            Rule::Intersection,
            Order::TurnedFirst,
            true,
            Refused::TooWideARange,
        ),
        (
            Rule::Intersection,
            Order::TurnedFirst,
            false,
            Refused::AnOutlineWindingAgainstItsFace,
        ),
    ];
    for (rule, order, is_certified, refused) in table {
        let policy = if is_certified {
            certified()
        } else {
            tolerant()
        };
        let mut model = a_model();
        let block = a_block(&mut model);
        let turned = the_cube_of_q1208s_crash(&mut model);
        let asked = format!("Q1208's cube, {rule:?}, {order:?}, {policy:?}");
        refused.assert_is(
            fuse(&mut model, [block, turned], rule, order, &policy),
            &asked,
        );
    }
}

/// The 30 of Q1208's 400 random turns whose union panicked.
///
/// Q1208's sweep drew a turn about a random axis by a random angle, kept it
/// when all three edges at the corner lean to `+x`, and put the corner at
/// `(0, y, z)` with `y` and `z` in 1 to 9. Of 400, each policy refused 370 by
/// name and answered or panicked on the rest; the 30 here panicked under both.
/// The whole sweep takes eight seconds, so only these are kept. Each is now
/// refused by name under both policies, and with the corner a thousandth
/// inside, each answers one solid that validates, so the turned cube is sound
/// and the refusal is about where its corner is.
#[test]
fn thirty_random_turns_that_panicked_are_refused_by_name() {
    const PANICKED: [usize; 30] = [
        1, 29, 34, 57, 69, 75, 76, 109, 127, 164, 169, 171, 179, 183, 189, 193, 216, 218, 222, 261,
        269, 270, 275, 286, 290, 366, 370, 378, 388, 400,
    ];
    let turns = q1208s_random_turns(400);
    for index in PANICKED {
        let (turn, at) = turns[index - 1];
        for policy in [certified(), tolerant()] {
            let asked = format!("turn {index}, {policy:?}");
            let mut model = a_model();
            let pair = a_block_and_a_cube_turned(&mut model, &turn, at);
            Refused::AHoleWindingWithItsOutline.assert_is(
                fuse(&mut model, pair, Rule::Union, Order::BlockFirst, &policy),
                &asked,
            );
        }
        let mut model = a_model();
        let pair = a_block_and_a_cube_turned(&mut model, &turn, [1e-3, at[1], at[2]]);
        let answer = fuse(
            &mut model,
            pair,
            Rule::Union,
            Order::BlockFirst,
            &tolerant(),
        )
        .unwrap_or_else(|why| panic!("turn {index}, the corner inside: {why}"));
        assert_eq!(answer.len(), 1, "turn {index}, the corner inside");
        model
            .validate(&tolerant(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("turn {index}, the corner inside: {why}"));
    }
}

/// What a fuse here is refused for.
#[derive(Clone, Copy, Debug)]
enum Refused {
    /// The block's `x = 0` face joined round the tiny triangle has the
    /// triangle as a hole winding with its outline, refused in `rings_round`
    /// before anything is written.
    AHoleWindingWithItsOutline,
    /// The intersection's cap on `x = 0` is the tiny triangle, and its stored
    /// corners wind against the face, refused by the sweep of the answer.
    AnOutlineWindingAgainstItsFace,
    /// The answer's shortest edge is a rounding long beside corners at 10.
    TooWideARange,
    /// The turned cube less the block is the tip outside the face, which
    /// encloses less than nothing once its corners are rounded.
    NoSpaceInside,
}

impl Refused {
    fn assert_is(self, answer: Result<Vec<SolidKey>, Error>, asked: &str) {
        let why = match answer {
            Ok(solids) => panic!("{asked} answers {solids:?}, not {self:?}"),
            Err(why) => why,
        };
        let matched = match self {
            Self::AHoleWindingWithItsOutline => matches!(
                why,
                Error::TheFuseWroteABrokenAnswer {
                    broken: Invalid::HoleWindsWithTheOuterWire
                }
            ),
            Self::AnOutlineWindingAgainstItsFace => matches!(
                why,
                Error::TheFuseWroteABrokenAnswer {
                    broken: Invalid::OuterWireWindsAgainstTheNormal
                }
            ),
            Self::TooWideARange => {
                matches!(why, Error::Invalid(Invalid::DynamicRangeTooWide { .. }))
            }
            Self::NoSpaceInside => matches!(
                why,
                Error::Invalid(Invalid::VolumeIsNotPositive { volume }) if volume <= 0.0
            ),
        };
        assert!(matched, "{asked} is refused {why:?}, not {self:?}");
    }
}

#[derive(Clone, Copy, Debug)]
enum Order {
    BlockFirst,
    TurnedFirst,
}

fn assert_refused(rule: Rule, order: Order, policy: &Policy, refused: Refused) {
    let mut model = a_model();
    let pair = the_block_and_the_turned_cube(&mut model, ON_THE_FACE);
    let asked = format!("{rule:?}, {order:?}, {policy:?}");
    refused.assert_is(fuse(&mut model, pair, rule, order, policy), &asked);
}

fn volume_of_the_answer(
    at: [f64; 3],
    rule: Rule,
    order: Order,
    policy: &Policy,
    asked: &str,
) -> f64 {
    let mut model = a_model();
    let pair = the_block_and_the_turned_cube(&mut model, at);
    let answer = fuse(&mut model, pair, rule, order, policy)
        .unwrap_or_else(|why| panic!("{asked}, {rule:?}: {why}"));
    model
        .validate(policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{asked}, {rule:?}: {why}"));
    answer
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.1).expect("a deflection"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{asked}, {rule:?} does not measure: {why}"))
                .volume()
        })
        .sum()
}

fn fuse(
    model: &mut Model,
    [block, turned]: [SolidKey; 2],
    rule: Rule,
    order: Order,
    policy: &Policy,
) -> Result<Vec<SolidKey>, Error> {
    let (one, other) = match order {
        Order::BlockFirst => (block, turned),
        Order::TurnedFirst => (turned, block),
    };
    model
        .fuse(&[one], &[other], rule, policy, &mut ProvenanceLog::new())
        .map(|answer| answer.solids().to_vec())
}

fn certified() -> Policy {
    Policy::certified()
}

fn tolerant() -> Policy {
    Policy::tolerant(100.0)
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("a scale"))
}

fn a_block(model: &mut Model) -> SolidKey {
    model
        .add_cube(10.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube")
}

/// The block, and the cube turned 1.5 about `(1, 1, 0)` with its corner at `at`.
fn the_block_and_the_turned_cube(model: &mut Model, at: [f64; 3]) -> [SolidKey; 2] {
    let axis = Axis::new(Point::origin(), Vector::new(1.0, 1.0, 0.0)).expect("an axis");
    let turn = Isometry::rotation(&axis, 1.5).expect("a turn");
    a_block_and_a_cube_turned(model, &turn, at)
}

fn a_block_and_a_cube_turned(model: &mut Model, turn: &Isometry, at: [f64; 3]) -> [SolidKey; 2] {
    let block = a_block(model);
    let turned = a_block(model);
    let shift = Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a shift");
    model
        .place_solid(turned, &shift.compose(turn))
        .expect("a placement");
    [block, turned]
}

/// Q1208's first `count` turns, each with where it puts the corner: drawn
/// from a seeded generator, keeping each turn that leans all three edges at
/// the corner towards `+x` by more than 0.05.
fn q1208s_random_turns(count: usize) -> Vec<(Isometry, [f64; 3])> {
    let mut seed: u64 = 1208;
    let mut next = || {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        #[expect(clippy::cast_precision_loss, reason = "53 bits fit an f64 exactly")]
        let drawn = (seed >> 11) as f64 / (1_u64 << 53) as f64;
        drawn
    };
    let mut turns = Vec::new();
    while turns.len() < count {
        let axis = Vector::new(next() - 0.5, next() - 0.5, next() - 0.5);
        let angle = next() * 3.0;
        let turn = Isometry::rotation(&Axis::new(Point::origin(), axis).expect("an axis"), angle)
            .expect("a turn");
        let leans_to_plus_x = [
            Vector::new(1.0, 0.0, 0.0),
            Vector::new(0.0, 1.0, 0.0),
            Vector::new(0.0, 0.0, 1.0),
        ]
        .into_iter()
        .all(|edge| turn.apply_to_vector(edge).components()[0] > 0.05);
        if leans_to_plus_x {
            let at = [0.0, 1.0 + 8.0 * next(), 1.0 + 8.0 * next()];
            turns.push((turn, at));
        }
    }
    turns
}

/// The corners of the turned cube in Q1208's sample, as stored.
const CORNERS_OF_Q1208S_CUBE: [[f64; 3]; 8] = [
    [0.0, 5.0, 7.629_394_531_25e-5],
    [
        0.132_229_525_990_410_93,
        4.752_663_665_461_631,
        9.996_142_524_721_972,
    ],
    [
        5.622_430_786_809_978_5,
        13.109_678_230_140_135,
        10.130_297_962_355_264,
    ],
    [
        5.490_201_260_819_568,
        13.357_014_564_678_504,
        0.134_231_731_578_605_46,
    ],
    [
        8.357_045_259_429_41,
        -0.486_267_611_351_861_36,
        -0.246_220_739_085_831_75,
    ],
    [
        13.847_246_520_248_977,
        7.870_746_953_326_642,
        -0.112_065_301_452_538_79,
    ],
    [
        13.979_476_046_239_39,
        7.623_410_618_788_274,
        9.884_000_929_324_118,
    ],
    [
        8.489_274_785_419_823,
        -0.733_603_945_890_230_6,
        9.749_845_491_690_827,
    ],
];

/// Its faces: the outward normal and offset the sample stores, and the
/// corners round it, walked the way the sample walks them.
const FACES_OF_Q1208S_CUBE: [([f64; 3], f64, [usize; 4]); 6] = [
    (
        [
            -0.835_704_525_942_941,
            0.548_626_761_135_186_2,
            0.024_629_703_303_114_425,
        ],
        2.743_135_684_773_167_6,
        [0, 1, 2, 3],
    ),
    (
        [
            0.835_704_525_942_941,
            -0.548_626_761_135_186_2,
            -0.024_629_703_303_114_425,
        ],
        7.256_864_315_226_832,
        [4, 5, 6, 7],
    ),
    (
        [
            -0.013_222_952_599_041_16,
            0.024_733_633_453_837_056,
            -0.999_606_623_077_665_8,
        ],
        0.123_591_903_336_150_18,
        [0, 3, 5, 4],
    ),
    (
        [
            0.549_020_126_081_956_7,
            0.835_701_456_467_850_4,
            0.013_415_543_763_329_263,
        ],
        14.178_508_305_864_014,
        [3, 2, 6, 5],
    ),
    (
        [
            0.013_222_952_599_041_16,
            -0.024_733_633_453_837_056,
            0.999_606_623_077_665_8,
        ],
        9.876_408_096_663_85,
        [2, 1, 7, 6],
    ),
    (
        [
            -0.549_020_126_081_956_7,
            -0.835_701_456_467_850_4,
            -0.013_415_543_763_329_263,
        ],
        -4.178_508_305_864_014,
        [1, 0, 4, 7],
    ),
];

fn the_cube_of_q1208s_crash(model: &mut Model) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let corners: Vec<_> = CORNERS_OF_Q1208S_CUBE
        .iter()
        .map(|&[x, y, z]| {
            builder
                .add_vertex(Point::new(x, y, z))
                .expect("a finite corner")
        })
        .collect();
    let sides = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (3, 5),
        (2, 6),
        (1, 7),
    ];
    let edges: Vec<_> = sides
        .iter()
        .map(|&(from, to)| {
            builder
                .add_edge(corners[from], corners[to], EdgeRun::Straight)
                .expect("an edge with length")
        })
        .collect();
    let mut faces = Vec::new();
    for (normal, offset, ring) in FACES_OF_Q1208S_CUBE {
        let steps: Vec<DraftStep> = (0..4)
            .map(|step| {
                let (from, to) = (ring[step], ring[(step + 1) % 4]);
                let (edge, way) = match sides.iter().position(|&side| side == (from, to)) {
                    Some(edge) => (edge, Orientation::Forward),
                    None => (
                        sides
                            .iter()
                            .position(|&side| side == (to, from))
                            .expect("every side of a face is an edge"),
                        Orientation::Reversed,
                    ),
                };
                DraftStep::Along(
                    builder
                        .add_coedge(edges[edge], way)
                        .expect("an edge of this builder"),
                )
            })
            .collect();
        let wire = builder.add_wire(&steps).expect("a ring of four");
        let plane =
            Plane::from_coefficients([normal[0], normal[1], normal[2], -offset]).expect("a plane");
        let face = builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a face");
        faces.push((face, Orientation::Forward));
    }
    builder.add_shell(&faces).expect("one shell");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("Q1208's cube is a sound solid")
}
