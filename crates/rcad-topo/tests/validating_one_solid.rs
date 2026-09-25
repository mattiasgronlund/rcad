//! Asking whether one solid is sound, rather than the whole model.
//!
//! [`Model::validate_solid`] runs every check [`Model::validate`] runs, over one
//! solid and what it is built from. What it promises is that the answer is the
//! one `validate` gives for a model holding that solid alone — and so that a
//! broken solid elsewhere in the model does not change it.
//!
//! **Neither answer says what the other will be**, and two tests here build the
//! model that shows each direction:
//!
//! - a small cube beside a block far out, with one corner of the cube moved off
//!   its plane by more than the cube's own rounding room and less than the room
//!   the far block buys the whole model — `validate` passes, `validate_solid`
//!   refuses the cube;
//! - a fuse with a `2^-30` step near the origin beside a block a thousand out —
//!   each is narrow enough to compute in, and together they are not, so
//!   `validate` refuses and `validate_solid` passes both.
//!
//! The model a file reads back into is the only public way to break a solid
//! that the builders refuse to make, and `Deserialize` itself refuses dangling
//! keys, sharing, orphans and a skin that cannot close. So a broken solid here
//! is a sound one with a corner moved in its bytes: geometry, which only the
//! sweep asks about.

mod cones;
mod rings;

use rcad_geom::{DynamicRange, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Invalid, Model, ProfileBuilder, Rule, SolidKey};

/// A way of putting one solid into a model.
type Builds = fn(&mut Model) -> SolidKey;

/// Every kind of solid the kernel builds, each through the door a caller would
/// use: the constructors, [`rcad_topo::SolidBuilder`] for a cone and a torus,
/// and a fuse.
const SHAPES: [(&str, Builds); 7] = [
    ("a cube", a_cube),
    ("a ball", a_ball),
    ("a round bar", a_round_bar),
    ("a frustum", a_frustum),
    ("a pointed cone", a_pointed_cone),
    ("a quarter of a torus", a_quarter_bend),
    ("the union of two overlapping blocks", the_union_of_two_overlapping_blocks),
];

#[test]
fn a_solid_gets_the_answer_a_model_holding_it_alone_gets() {
    for (name, builds) in SHAPES {
        let mut model = a_model();
        let solid = builds(&mut model);

        assert_eq!(
            said(&model.validate_solid(solid, &Policy::certified(), &mut ProvenanceLog::new())),
            what_validate_says_of_it_alone(&model, solid),
            "{name}"
        );
    }
}

/// What [`Model::validate`] says of a model holding `solid` and nothing else.
///
/// A fuse leaves what it fused in the model beside its answer, and nothing
/// public takes a solid out of a model, so the answer is described again, face
/// by face, into a model of its own.
fn what_validate_says_of_it_alone(model: &Model, solid: SolidKey) -> String {
    if model.solid_count() == 1 {
        return said(&model.validate(&Policy::certified(), &mut ProvenanceLog::new()));
    }
    let mut alone = a_model();
    cones::describe_again(model, solid)
        .expect("a flat solid with one shell describes again")
        .build(&mut alone, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("and builds into a model of its own");
    assert_eq!(alone.solid_count(), 1);
    said(&alone.validate(&Policy::certified(), &mut ProvenanceLog::new()))
}

/// Every shape built alone, then every shape built beside every other one —
/// each pair in one model, the second a long way along `x` — and the answer
/// for a solid is the one it got alone.
///
/// Every pair's whole model is sound too, so this is also the other half of
/// what the brief asked: over these shapes, a model `validate` passes is one
/// whose every solid `validate_solid` passes. That is a fact about these
/// shapes and not a law — `a_corner_the_far_solid_makes_room_for_is_refused_on_its_own`
/// is a model where it fails.
#[test]
fn a_solid_gets_the_same_answer_beside_any_other_as_it_gets_alone() {
    let alone: Vec<String> = SHAPES
        .iter()
        .map(|(_, builds)| {
            let mut model = a_model();
            let solid = builds(&mut model);
            said(&model.validate_solid(solid, &Policy::certified(), &mut ProvenanceLog::new()))
        })
        .collect();

    for (first, (first_name, builds_first)) in SHAPES.iter().enumerate() {
        for (second, (second_name, builds_second)) in SHAPES.iter().enumerate() {
            let mut model = a_model();
            let one = builds_first(&mut model);
            let other = builds_second(&mut model);
            model
                .place_solid(
                    other,
                    &Isometry::translation(Vector::new(A_LONG_WAY_OUT, 0.0, 0.0))
                        .expect("a plain translation"),
                )
                .expect("this model made the key");

            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| {
                    panic!("{first_name} beside {second_name} is a sound model: {why:?}")
                });
            for (solid, position, name) in [(one, first, first_name), (other, second, second_name)] {
                assert_eq!(
                    said(&model.validate_solid(
                        solid,
                        &Policy::certified(),
                        &mut ProvenanceLog::new()
                    )),
                    alone[position],
                    "{name}, beside {}",
                    if solid == one { second_name } else { first_name }
                );
            }
        }
    }
}

/// **The room effect.** The rounding room a corner gets off its face's plane
/// grows with the largest coordinate in what is being checked — the smaller of
/// the model's declared size and its largest corner, times
/// `Scale::RELATIVE_LINEAR_TOLERANCE`, plus what writing a plane down costs.
///
/// Here the declared size is `100` and the cube's largest corner is
/// [`A_HEIGHT_NOTHING_ELSE_WRITES`], so the cube alone gets about `2.75e-9` of
/// room. The block at [`A_LONG_WAY_OUT`] brings the model's largest corner past
/// `100`, so the whole model gets about `1e-7`. A corner moved by
/// [`BETWEEN_THE_TWO_ROOMS`] is inside the second and outside the first.
#[test]
fn a_corner_the_far_solid_makes_room_for_is_refused_on_its_own() {
    let room_the_cube_gets_alone = Scale::RELATIVE_LINEAR_TOLERANCE * A_HEIGHT_NOTHING_ELSE_WRITES;
    let room_the_model_gets = Scale::RELATIVE_LINEAR_TOLERANCE * A_DECLARED_SIZE;
    assert!(
        room_the_cube_gets_alone * 2.0 < BETWEEN_THE_TWO_ROOMS
            && BETWEEN_THE_TWO_ROOMS * 2.0 < room_the_model_gets,
        "the move sits well clear of both rooms"
    );

    let mut model = a_model();
    a_block_at(&mut model, A_LONG_WAY_OUT, 2.0);
    a_block_at(&mut model, 0.0, A_HEIGHT_NOTHING_ELSE_WRITES);
    let moved = A_HEIGHT_NOTHING_ELSE_WRITES + BETWEEN_THE_TWO_ROOMS;
    let model = read_back(&bytes_with_the_first_number_replaced(
        &model,
        A_HEIGHT_NOTHING_ELSE_WRITES,
        moved,
    ));
    let [far, cube] = the_two_solids_of(&model);
    assert!(
        model
            .vertices()
            .any(|corner| corner.point().coordinates()[2] == moved),
        "the corner that moved is one of the cube's top corners"
    );

    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the far block makes room for the moved corner in the whole model");
    model
        .validate_solid(far, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the far block is sound");
    let refused = model
        .validate_solid(cube, &Policy::certified(), &mut ProvenanceLog::new())
        .expect_err("the cube alone has no room for it");
    assert!(
        matches!(
            refused,
            Error::Invalid(Invalid::VertexOffItsFacePlane { distance })
                if distance > room_the_cube_gets_alone
        ),
        "{refused:?}"
    );

    let mut alone = a_model();
    a_block_at(&mut alone, 0.0, A_HEIGHT_NOTHING_ELSE_WRITES);
    let alone = read_back(&bytes_with_the_first_number_replaced(
        &alone,
        A_HEIGHT_NOTHING_ELSE_WRITES,
        moved,
    ));
    assert_eq!(
        said(&alone.validate(&Policy::certified(), &mut ProvenanceLog::new())),
        said(&Err(refused)),
        "and it is what the whole sweep says of the cube in a model of its own"
    );
}

/// **The range effect**, and the consumer's complaint in the other direction.
///
/// `a_fuse_is_not_blamed_for_a_solid_it_never_touched.rs`'s numbers: the fuse
/// leaves edges `2^-30` long near the origin, and the block sits at `1024`.
/// Each on its own is inside [`DynamicRange::LARGEST_SUPPORTED_RATIO`]; the
/// largest coordinate of one against the shortest edge of the other is
/// `1026 · 2^30`, about `1.1e12`, which is not.
#[test]
fn two_solids_each_fine_on_its_own_can_be_too_far_apart_together() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::millimetres(4.0).expect("four millimetres is a size"));
    let far = a_two_millimetre_block(&mut model, Vector::new(1024.0, 0.0, 0.0));
    let here = a_two_millimetre_block(&mut model, Vector::new(0.0, 0.0, 0.0));
    let corner = 2.0 - A_TINY_STEP;
    let overlapping = a_two_millimetre_block(&mut model, Vector::new(corner, corner, corner));
    let fused = model
        .fuse(&[here], &[overlapping], Rule::Union, &policy, &mut log)
        .expect("two blocks overlapping in one corner unite");
    let [answer] = fused.solids() else {
        panic!("a union of two touching blocks is one solid");
    };
    assert_eq!(
        model.solid_count(),
        4,
        "the far block, the two blocks fused, which a fuse leaves where they \
         were, and their union"
    );

    let refused = model
        .validate(&policy, &mut log)
        .expect_err("the far block and the union together are too wide to compute in");
    assert!(
        matches!(
            refused,
            Error::Invalid(Invalid::DynamicRangeTooWide { ratio })
                if ratio > DynamicRange::LARGEST_SUPPORTED_RATIO
        ),
        "{refused:?}"
    );
    model
        .validate_solid(far, &policy, &mut log)
        .expect("the far block is narrow enough on its own");
    model
        .validate_solid(*answer, &policy, &mut log)
        .expect("and so is the union, step and all");
    for solid in model.solids() {
        model
            .validate_solid(solid.key(), &policy, &mut log)
            .expect("every solid in the model is");
    }
}

#[test]
fn a_broken_solid_does_not_change_the_answer_for_the_one_beside_it() {
    let mut model = a_model();
    a_block_at(&mut model, A_LONG_WAY_OUT, 2.0);
    a_block_at(&mut model, 0.0, A_HEIGHT_NOTHING_ELSE_WRITES);
    let moved = A_HEIGHT_NOTHING_ELSE_WRITES + FAR_OFF_ANY_ROOM;
    let model = read_back(&bytes_with_the_first_number_replaced(
        &model,
        A_HEIGHT_NOTHING_ELSE_WRITES,
        moved,
    ));
    let [sound, broken] = the_two_solids_of(&model);

    let refused = model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect_err("one of its solids has a corner well off its plane");
    assert!(
        matches!(refused, Error::Invalid(Invalid::VertexOffItsFacePlane { .. })),
        "{refused:?}"
    );
    model
        .validate_solid(sound, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the block beside it is sound");
    assert_eq!(
        said(&model.validate_solid(broken, &Policy::certified(), &mut ProvenanceLog::new())),
        said(&Err(refused)),
        "the broken solid is refused for what the whole model is refused for"
    );
}

#[test]
fn a_solid_from_another_model_is_refused_by_name() {
    let mut elsewhere = a_model();
    let theirs = a_cube(&mut elsewhere);
    let mut model = a_model();
    a_cube(&mut model);

    let refused = model
        .validate_solid(theirs, &Policy::certified(), &mut ProvenanceLog::new())
        .expect_err("a key from another model names nothing here");
    assert!(
        matches!(refused, Error::KeyFromAnotherModel { entity: "solid" }),
        "{refused:?}"
    );
}

/// The declared size of every model here but the range test's.
const A_DECLARED_SIZE: f64 = 100.0;

/// Far enough out that the model's largest corner is past [`A_DECLARED_SIZE`],
/// near enough that nothing is anywhere near too wide to compute in.
const A_LONG_WAY_OUT: f64 = 1000.0;

/// The cube's height, and a number no other solid here writes, so the first
/// place it appears in a model's bytes is one of the cube's top corners.
const A_HEIGHT_NOTHING_ELSE_WRITES: f64 = 2.75;

/// How far one corner is moved for the room test: roughly four times the room
/// the cube gets alone and a tenth of what the whole model gets.
const BETWEEN_THE_TWO_ROOMS: f64 = 1e-8;

/// How far one corner is moved to break a solid under any room at all.
const FAR_OFF_ANY_ROOM: f64 = 0.5;

/// How far the two blocks the range test fuses overlap, and the shortest edge
/// their union carries.
const A_TINY_STEP: f64 = 1.0 / 1_073_741_824.0;

fn a_model() -> Model {
    Model::new(Scale::millimetres(A_DECLARED_SIZE).expect("100 mm is a working size"))
}

/// What the answer said, for comparing two answers — [`Error`] does not
/// compare, and its `Debug` carries every field.
fn said(answer: &rcad_topo::Result<()>) -> String {
    format!("{answer:?}")
}

fn a_cube(model: &mut Model) -> SolidKey {
    model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two")
}

fn a_ball(model: &mut Model) -> SolidKey {
    model
        .add_ball(2.0, &Isometry::identity())
        .expect("a ball of radius two")
}

fn a_round_bar(model: &mut Model) -> SolidKey {
    model
        .add_round_bar(1.0, 4.0, &Isometry::identity())
        .expect("a bar of radius one")
}

fn a_frustum(model: &mut Model) -> SolidKey {
    cones::a_frustum(model, 0.5, 2.0, 8.0)
}

fn a_pointed_cone(model: &mut Model) -> SolidKey {
    cones::a_pointed_cone(model, 0.5, 8.0)
}

fn a_quarter_bend(model: &mut Model) -> SolidKey {
    rings::a_quarter_bend(model, 4.0, 1.0)
}

fn the_union_of_two_overlapping_blocks(model: &mut Model) -> SolidKey {
    let one = a_two_millimetre_block(model, Vector::new(0.0, 0.0, 0.0));
    let other = a_two_millimetre_block(model, Vector::new(1.0, 1.0, 1.0));
    let fused = model
        .fuse(
            &[one],
            &[other],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("two blocks overlapping by half unite");
    let [answer] = fused.solids() else {
        panic!("the union of two overlapping blocks is one solid");
    };
    *answer
}

fn a_two_millimetre_block(model: &mut Model, at: Vector) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a two-millimetre square");
    model
        .add_extrusion(
            &square,
            2.0,
            &Isometry::translation(at).expect("a plain translation"),
        )
        .expect("a two-millimetre block")
}

/// A block two wide in `x` and `y` and `height` tall, whose corners are
/// **stored** `x` along — written into the profile rather than a placement, so
/// that the room the sweep sizes from its stored corners sees where it is.
fn a_block_at(model: &mut Model, x: f64, height: f64) -> SolidKey {
    let square = ProfileBuilder::new(vec![[x, 0.0], [x + 2.0, 0.0], [x + 2.0, 2.0], [x, 2.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a two-millimetre square");
    model
        .add_extrusion(&square, height, &Isometry::identity())
        .expect("a block")
}

fn read_back(bytes: &[u8]) -> Model {
    postcard::from_bytes(bytes).expect("a corner off its plane still reads back")
}

/// The model's two solids, in the order they were built — slot order survives
/// the round trip, and the keys from before it do not.
fn the_two_solids_of(model: &Model) -> [SolidKey; 2] {
    let solids: Vec<SolidKey> = model.solids().map(|solid| solid.key()).collect();
    solids
        .try_into()
        .unwrap_or_else(|solids: Vec<SolidKey>| panic!("two solids, not {}", solids.len()))
}

/// The model's bytes with the first `f64` reading `marker` replaced by
/// `instead` — `writing_and_reading.rs`'s helper of the same name.
fn bytes_with_the_first_number_replaced(model: &Model, marker: f64, instead: f64) -> Vec<u8> {
    let mut bytes = postcard::to_allocvec(model).expect("a model writes out");
    let wanted = marker.to_le_bytes();
    let at = bytes
        .windows(wanted.len())
        .position(|window| window == wanted)
        .unwrap_or_else(|| panic!("{marker} is written somewhere in the model"));
    bytes[at..at + wanted.len()].copy_from_slice(&instead.to_le_bytes());
    bytes
}
