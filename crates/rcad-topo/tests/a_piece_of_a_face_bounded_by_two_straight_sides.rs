//! Two walls a hair apart, and what now stands between them and the two-sided
//! piece §67 refuses.
//!
//! §67 found `arrange.rs` asserting where it should refuse: a walk that comes
//! back with a cycle of two straight sides, which enclose no area. The refusal
//! it put there is unchanged and still right. **What changed under it is who can
//! still get to it**, and this file is that question written down.
//!
//! §67's own test drove the walk with the `two_solids_fused_by_a_stranger`
//! corpus entry `crash-7d309176caedc7140756f167d7db5e7740eff1c3`. Those bytes
//! reached the fuse because `Model::allowed_wobble` read the size the *model
//! declared about itself* — `1.8e21` mm, two hundred light-years — and handed
//! back `1.8e12` mm of room. §70 closed that: the allowance is now the smaller
//! of what the model declares and what its own corners show. So the bytes no
//! longer get past the sweep, and the first test here is that they do not.
//!
//! # What the walk needs, and why nothing here can hand it over any more
//!
//! A cycle of two straight sides needs two crossings on one face that are two
//! *different* lines — `Arrangement::crossing_like` folds two partners that cut
//! a face in the same line into one crossing, by an exact determinant — sharing
//! *both* their nodes. Two different lines meet at one point, so the second node
//! exists only because the arrangement could not tell two points apart. That
//! wants two planes a hair from parallel and nothing else.
//!
//! **The hair is still legal, and the second test builds it**: a block whose
//! `y` wall is written with `1e-100` in the `x` slot of its normal is a sound
//! solid — its own corners sit `1e-99` off it, which is nothing beside a
//! coordinate of ten — and it walks through the tightened sweep. What it does
//! *not* do any more is reach the walk. Every arrangement built out of it is
//! refused by name first, and the test pins which name.
//!
//! **That is a census and not a proof.** Nine shapes have now been tried across
//! §67 and this section, and nine were caught by six different correct refusals;
//! that is evidence that the walk is hard to reach and not a demonstration that
//! it cannot be. §16.10 is the rule and it cuts this way too — so §67's refusal
//! stays, and this file says plainly that what guards it today is a refusal
//! upstream rather than a test that drives it.

use std::collections::HashMap;

use rcad_geom::{Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftStep, DraftVertex, Error, Invalid, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey,
};

/// The bytes that found the two-sided piece do not get past the sweep any more.
///
/// This is the guard on §70's cap as much as it is §67's own history: it fails
/// the moment `Model::allowed_wobble` goes back to reading a size the model
/// declares about itself, which is the one way to make §67's original test green
/// again and the one way that would put the hole back.
#[test]
fn the_bytes_that_found_the_two_sided_piece_no_longer_get_past_the_sweep() {
    let model = the_model_the_fuzzer_found();

    let refused = model.validate(&Policy::certified(), &mut ProvenanceLog::new());

    assert!(
        matches!(
            refused,
            Err(Error::Invalid(Invalid::VertexOffItsFacePlane { .. }))
        ),
        "the sweep names a corner off its own face's plane: {refused:?}"
    );
}

/// Three walls are off their own planes, not the two §67 counted, and the one
/// the sweep names is the nearest of them rather than the worst.
///
/// `check_flat_face` returns on the first corner it finds outside the
/// allowance, walking faces in the order the model stores them, so the number in
/// the refusal is a witness and never a maximum. §67.4 read it as the latter and
/// §67.6 corrects that.
#[test]
fn the_sweep_names_a_witness_and_not_the_worst_wall() {
    let model = the_model_the_fuzzer_found();

    let mut how_far_each_wall_is_out: Vec<f64> = model
        .faces()
        .filter_map(|face| {
            let Surface::Plane(plane) = face.surface() else {
                return None;
            };
            let worst = face
                .wires()
                .flat_map(|wire| wire.points())
                .map(|point| plane.signed_distance_to(point).abs())
                .fold(0.0_f64, f64::max);
            (worst > 1e-9).then_some(worst)
        })
        .collect();
    how_far_each_wall_is_out.sort_by(f64::total_cmp);

    assert_eq!(
        how_far_each_wall_is_out,
        vec![0.359_384_536_734_069_1, 10.0, 20.0],
        "three walls are out, not two"
    );

    let Err(Error::Invalid(Invalid::VertexOffItsFacePlane { distance })) =
        model.validate(&Policy::certified(), &mut ProvenanceLog::new())
    else {
        panic!("the sweep names a corner off its own face's plane");
    };
    assert_eq!(
        distance, 9.536_734_069_115_482e-6,
        "the witness is the first corner outside the allowance, on none of those three walls"
    );
    assert!(
        how_far_each_wall_is_out.iter().all(|&out| out > distance),
        "every wall that is out is further out than the number the refusal carries"
    );
}

/// A wall written with a hair of lean is a sound solid, and the arrangement it
/// makes is refused by name before the walk can build a two-sided piece.
///
/// The two knives put two planes a hair from parallel through one face of
/// `standing`: one written plain, one leaned by `1e-100`, so the two lines they
/// cut it in meet at `x = 0` and are `1e-99` apart at `x = 10` — the shape the
/// two-sided walk needs. It never gets there.
///
/// **The tolerant fuse stopped on the two lines touching at one corner**
/// (`TwoCurvesTouchWithoutCrossing`) until Q901 landed Q676's order of two
/// meets of three planes by their planes. It then stopped a step later,
/// refused `Undecidable` at *a point inside a piece of a face*: the record's
/// corners were `(5, 5, 0)` and `(10, 5, 0)` and two boxes a rounding above
/// them in `y`, the sliver between the two walls. That sliver is about
/// `1e-99` wide at a `y` of `5`, so no `f64` point lies inside it — but
/// §365.9's `by_the_piece_across_a_side` places it anyway, by taking the
/// placement of the square across `y = 5` on the same face: the leaning
/// knife's plane holds `(0, 5, 10)` and not `(5, 5, 10)`, and the sliver lies
/// on the same side of it as the square. So the refusal was a gap, not a
/// limit, and Q1357 closed it. The tolerant fuse now stops a piece later, at
/// *a point in a piece that every solid could place*, on a wall piece with
/// corners `(-0, 5, 2)`, `(-5, [5-ulp, 5], 2)`, `(-5, .., 5)`, `(-0, 5, 5)`: a
/// point was found inside and no ray placed it. §417's inside rule — which
/// places a flat piece inside a convex flat solid by its face planes rather
/// than by a ray — now decides that stop: both the tolerant fuse and the
/// certified one answer one solid, each measuring `2450.0`
/// (`3000 − 125 − 200 − 300 + 75`). It is still not the two-sided walk.
#[test]
fn two_walls_a_hair_apart_are_caught_before_the_walk_gets_a_two_sided_piece() {
    for policy in [Policy::tolerant(1e6), Policy::certified()] {
        let (mut model, standing, plain_knife, leaning_knife) = a_block_between_two_knives();

        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a wall with a hair of lean is a sound solid, corners and all");

        let outcome = model.fuse(
            &[standing, plain_knife],
            &[leaning_knife],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        );
        assert!(
            !matches!(outcome, Err(Error::Invalid(Invalid::WireTooShort { .. }))),
            "if this ever reaches the two-sided walk, §67's refusal has a driver \
             again and this test should become one: {outcome:?}"
        );
        let fused = outcome.unwrap_or_else(|why| {
            panic!("{policy:?}: the sliver between the walls is placed by §417's inside rule now: {why:?}")
        });
        assert_eq!(
            fused.solids().len(),
            1,
            "{policy:?}: the two walls and the sliver between them make one solid: {fused:?}"
        );
        let solid = fused.solids()[0];
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{policy:?}: the fused wall does not validate: {why}"));
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{policy:?}: the fused wall does not measure: {why}"));
        assert!(
            (measured.volume() - 2450.0).abs() <= measured.volume_error(),
            "{policy:?}: volume {} against 2450.0 (3000 − 125 − 200 − 300 + 75), error {}",
            measured.volume(),
            measured.volume_error()
        );
    }
}

/// A block, a knife whose far wall sits at `y = 5`, and a second knife whose far
/// wall sits there too but is written with a hair of lean.
fn a_block_between_two_knives() -> (Model, SolidKey, SolidKey, SolidKey) {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));

    let standing = a_plain_block(&mut model, Vector::new(0.0, 0.0, 0.0));
    let plain_knife = a_plain_block(&mut model, Vector::new(-5.0, -5.0, -5.0));
    let leaning_knife = a_block_whose_far_wall_leans(
        &mut model,
        Vector::new(-5.0, -5.0, 2.0),
        A_LEAN_NO_COORDINATE_OF_TEN_CAN_SHOW,
        &policy,
        &mut log,
    );

    (model, standing, plain_knife, leaning_knife)
}

/// How far off the `y` axis the leaning wall's normal points.
///
/// Ten units along the wall that is `1e-99`, which is nothing beside a
/// coordinate of ten — so the wall's own corners are on it as far as any
/// tolerance in the kernel is concerned, and the sweep passes it. It is still a
/// different plane from the one beside it, and `relation_of_two_lines` says so
/// with an exact determinant, which is what keeps the two crossings apart.
const A_LEAN_NO_COORDINATE_OF_TEN_CAN_SHOW: f64 = -1e-100;

fn a_plain_block(model: &mut Model, at: Vector) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    model
        .add_extrusion(
            &square,
            10.0,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("building a block needs no decision")
}

/// A ten-block with its low corner at `at`, whose high-`y` wall carries `lean`
/// in the `x` slot of its normal.
///
/// [`Plane::from_coefficients`] takes `a * x + b * y + c * z + d = 0`, so the
/// fourth number is *minus* the offset. §67's own reconstructions had that sign
/// the wrong way round and were refused for being the wrong shape rather than
/// for anything about this bug; §67.6 records it.
fn a_block_whose_far_wall_leans(
    model: &mut Model,
    at: Vector,
    lean: f64,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> SolidKey {
    let side = 10.0;
    let corner_of = |corner: usize| {
        Point::new(
            at.x() + side * f64::from(u8::try_from(corner & 1).expect("a bit")),
            at.y() + side * f64::from(u8::try_from((corner >> 1) & 1).expect("a bit")),
            at.z() + side * f64::from(u8::try_from((corner >> 2) & 1).expect("a bit")),
        )
    };
    let low = corner_of(0);
    let far_y = low.y() + side;
    // Wire, then the four numbers the wall's plane is written down as. Each wire
    // walks anticlockwise seen from outside, so the wall's own normal is the
    // outward one.
    let walls: [([usize; 4], [f64; 4]); 6] = [
        ([0, 2, 3, 1], [0.0, 0.0, -1.0, low.z()]),
        ([4, 5, 7, 6], [0.0, 0.0, 1.0, -(low.z() + side)]),
        ([0, 1, 5, 4], [0.0, -1.0, 0.0, low.y()]),
        ([1, 3, 7, 5], [1.0, 0.0, 0.0, -(low.x() + side)]),
        ([3, 2, 6, 7], [lean, 1.0, 0.0, -(far_y + lean * low.x())]),
        ([2, 0, 4, 6], [-1.0, 0.0, 0.0, low.x()]),
    ];

    let mut builder = SolidBuilder::new();
    let corners: Vec<DraftVertex> = (0..8)
        .map(|corner| {
            builder
                .add_vertex(corner_of(corner))
                .expect("a corner of a block")
        })
        .collect();

    let mut sides = HashMap::new();
    let mut faces = Vec::new();
    for (around, coefficients) in &walls {
        let mut steps = Vec::new();
        for step in 0..4 {
            let (from, to) = (around[step], around[(step + 1) % 4]);
            let (first, second) = (from.min(to), from.max(to));
            let side = match sides.get(&(first, second)) {
                Some(&side) => side,
                None => {
                    let side = builder
                        .add_edge(corners[first], corners[second], EdgeRun::Straight)
                        .expect("a straight side of a block");
                    sides.insert((first, second), side);
                    side
                }
            };
            let orientation = if from == first {
                Orientation::Forward
            } else {
                Orientation::Reversed
            };
            steps.push(DraftStep::Along(
                builder
                    .add_coedge(side, orientation)
                    .expect("a side walked one way"),
            ));
        }
        let wire = builder.add_wire(&steps).expect("a wall of a block closes");
        let plane = Plane::from_coefficients(*coefficients).expect("a wall's plane");
        faces.push((
            builder
                .add_face(Surface::Plane(plane), &[wire])
                .expect("a wall of a block"),
            Orientation::Forward,
        ));
    }
    builder.add_shell(&faces).expect("six walls close a block");
    builder
        .build(model, policy, log)
        .expect("a wall with a hair of lean is still a block")
}

/// The model the bytes decode to.
fn the_model_the_fuzzer_found() -> Model {
    let bytes = what_the_fuzzer_wrote();
    postcard::from_bytes(&bytes).expect("the corpus entry decodes")
}

fn what_the_fuzzer_wrote() -> Vec<u8> {
    let digits = THE_CORPUS_ENTRYS_SECOND_HALF.as_bytes();
    assert!(digits.len().is_multiple_of(2), "hex comes in pairs");
    digits
        .chunks(2)
        .map(|pair| {
            let text = core::str::from_utf8(pair).expect("hex is ascii");
            u8::from_str_radix(text, 16).expect("hex is hex")
        })
        .collect()
}

/// `crash-7d309176caedc7140756f167d7db5e7740eff1c3`, bytes `2382..4763`.
///
/// The target splits its input into a header byte and two halves and decodes a
/// `Model` from each; the first half of this one is not a model at all, and
/// this is the half that is.
const THE_CORPUS_ENTRYS_SECOND_HALF: &str = concat!(
    "030100000000000059441100000100000000000000000000000000000000000000000000000000010100000000000024",
    "400000000000000000000000000000000000010100000000000024400000000000002440000000000000000000010100",
    "000000000000000000000000002440000000000000000000010100000000000000000000000000003200000000000000",
    "2440000101000000000000244000000000000000000000000000b8244000010100000000000024400000000000002440",
    "000000000000244000010100000000000000000000000000002440000000040000244000010100000000000000000000",
    "000000000000000000000000000000010100000000000024400000000000000000000000000000000000010100000000",
    "000024400000000000002440000000000000000000010100000000000000000000000000002440000000000000000000",
    "010100000000000000000000000000000000000000000000244000010100000000000024400000000000000000000000",
    "000000244000010100000000000024400000000000002440000000000000244000010100000000000000000000000000",
    "002440000000000000244000011900000101010201000001010201030100000101030104010000010104010101000001",
    "010501060100000101060107010000010107010801000001010801050100000101010105010000010102010601000001",
    "010301070100000101040108010000010109010a01000001010a010b01000001010b010c01000001010c010901000001",
    "010d010e01000001010e010f01000001010f0110010000010110010d010000010109010d01000001010a010e01000001",
    "010b010f01000001010c0110010000013100000104010101010301010101020101010101010101010501000101060100",
    "01010701000101080100010101010001010a010001010501010101090101010102010001010b0100010106010101010a",
    "0101010103010001010c0100010107010101010b010101010401000101090100010108010101010c0101010110010101",
    "010f010101010e010101010d0101010111010001011201000101130100010114010001010d0100010116010001011101",
    "01010115010101010e010001011701000101120101010116010101010f01000101180100010113010101011701010101",
    "100100010115010001011401010101180101010d00000104000101000201000301000401010104000501000601000701",
    "000801010104000901000a01000b01000c01010104000d01000e01000f01001001010104001101001201001301001401",
    "010104001501001601001701001801010104001901001a01001b01001c01010104001d01001e01001f01002001010104",
    "002101002201002301002401010104002501002601002701002801010104002901002a01002b01002c01010104002d01",
    "002e01002f01003001010d000001000000f100000000000000000000040000000000000000f0bf000000000008008001",
    "01010001010000000000000000000000000000000000000000000100f03f00000000000024c001020100010100000000",
    "0000000000000000000000f0bf0000000000000000000000000000008001030100010100000000000000f03f00000000",
    "00000080000000000000000000000000000024c0010401000101000000000000000000000000000000f03f0000000000",
    "00000000000000000024c001050100010100000000000000f0bf00002500000000800000000000000000000000080000",
    "00800106010001010000000000000000000000000000210000000000000000f0bf000000000000008001070100010100",
    "00000000000000000000004000000000000000000000f03f00000000000024c001080100010100000000000000000000",
    "0000000000f0bf0000000000000000000000000000008001090100010100000000000000f03f00000000000000800000",
    "0000000000000000000000002440010a01000101000000000000000000000000000000f03f0000000000000000000000",
    "00000024c0010b0100010100000000000000f0bf00000000000000fffffffe00000000000000000000000080010c0100",
    "0103000001060101000000000000000000000000000000000000000000000000000000000000f03f0000000000000000",
    "000000000000000000000000000000000002010000000000000000000000000000000000000000000000000000000000",
    "00f03f000000000000000000000000000000000000000000000000000301000000000000000000000000000000000000",
    "000000000000000000000000f03f00000000000000000000000000000000000000000000000000040100000000000000",
    "0000000000000000000000000000000000000000000000f03f0000000000000000000000000000000000000000000000",
    "00000501000000000000000000000000000000000000000000000000000000000000f03f000000000000000000000000",
    "000000000000000000000000000601000000000000000000000000000000000000000000000000000000000000f03f00",
    "000000000000000000000000000000000000000000000000010106070100000000000000000000000000000000000000",
    "0000000000000000000000f03f0000000000000000000000000000000000000000000000000008010000000000000000",
    "00000000000000000000000000000000000000000000f03f000000000000000000000000000000000000000000000000",
    "000901000000000000000000000000000000000000000000000000000000000000f03f00000000000000000000000000",
    "0000000000000000000000000a01000000000000000000000000000000000000000000000000000000000000f03f0000",
    "00000000000000000000000000000000000000000000000b010000000000000000000000000000000000000000000000",
    "00000000000000f03f000000000000000000000000000000000000000000000000000c01000000000000000000000000",
    "000000000000000000000000000000000000f03f00000000000000000000000000000000000000000000000000010300",
    "0001010101000000000000000000000000000000000000000000000000000000000000f03f00000000000000fffb0000",
    "0000000000000000000000000000010101020100000000000014400000000000bf144000000000000014000000400000",
    "00f03f0000000000000000000000000000000000000000240000000001",
);
