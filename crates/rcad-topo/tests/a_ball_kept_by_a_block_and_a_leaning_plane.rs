//! A ball's skin cut by three planes that no point inside the ball is behind
//! all of, left by two fuses a designer makes (row Q2038, `docs/decisions.md`
//! §35.6 and §450.7).
//!
//! **The bite.** A ball of radius 3 at the origin kept to the block
//! `[1, 9] × [2, 8] × [−5, 10]`, then to a slab whose top face is the plane
//! `x + y + z = 5.1`. The block leaves a lens of the ball's skin, `x ≥ 1`,
//! `y ≥ 2`, with its top corner at `(1, 2, 2)`, where `x + y + z = 5`, as high
//! as it gets along the side on `x = 1`. So the slab's plane bites only the
//! side on `y = 2`, and the skin is one piece with four sides. The three
//! planes meet at `(1, 2, 2.1)`, outside the ball, and the centre is in front
//! of the slab's plane, so `holds.rs`'s `leaves_a_ball_in_one_piece` found no
//! point to push the sphere out from, and every point whose ray met the skin
//! was refused `CannotFollowAFacesTrimming`.
//!
//! **The touch.** A ball of radius 5 kept to `x ≥ 3` and `y ≤ 4` by a block,
//! then to `x ≥ z` by a turned slab: `y = 4` only touches what the other two
//! keep, at `(3, 4, 0)`, and the three planes meet at `(3, 4, 3)`, outside.
//! Both are the shapes `holds.rs`'s
//! `a_circle_touching_or_biting_what_two_planes_keep_is_refused` pinned as
//! refused (row Q1350).

use core::f64::consts::FRAC_PI_4;

use rcad_geom::{Axis, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Model, ProfileBuilder, Rule, SolidKey};

/// Each point is placed as the hand places it: in the ball, `x > 1`, `y > 2`
/// and `x + y + z < 5.1` is inside. `(1.5, 2.05, 1.58)` is in the ball and in
/// the block, but past the slab's plane, in the bite.
#[test]
fn a_ball_kept_to_a_blocks_corner_and_under_a_leaning_plane_places_each_point() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ball = model
        .add_ball(3.0, &Isometry::identity())
        .expect("a ball of radius 3");
    let corner = block(
        &mut model,
        [1.0, 2.0, -5.0],
        [9.0, 8.0, 10.0],
        &Isometry::identity(),
    );
    let lens = kept(&mut model, ball, corner);
    let leaning = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(-1.0, 1.0, 0.0)).expect("a direction"),
        (1.0 / 3f64.sqrt()).acos(),
    )
    .expect("a finite turn");
    assert_turns_z_onto(&leaning, [1.0 / 3f64.sqrt(); 3]);
    let top = 5.1 / 3f64.sqrt();
    let slab = block(
        &mut model,
        [-20.0, -20.0, top - 20.0],
        [20.0, 20.0, top],
        &leaning,
    );
    let solid = kept(&mut model, lens, slab);
    for (at, want) in [
        ([1.5, 2.3, 1.0], Containment::Inside),
        ([1.2, 2.2, 0.0], Containment::Inside),
        ([2.0, 2.05, -0.5], Containment::Inside),
        ([0.0, 0.0, 0.0], Containment::Outside),
        ([1.5, 2.05, 1.58], Containment::Outside),
        ([0.5, 2.5, 0.0], Containment::Outside),
    ] {
        assert_eq!(placed(&model, solid, at), want, "{at:?}");
    }
}

/// Each point is placed as the hand places it: in the ball, `x > 3`, `x > z`
/// and `y < 4` is inside. `(3.2, 0, 3.4)` is in the ball and the block but
/// past `x = z`.
#[test]
fn a_ball_kept_to_x_at_least_3_and_z_beside_a_plane_touching_it_places_each_point() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ball = model
        .add_ball(5.0, &Isometry::identity())
        .expect("a ball of radius 5");
    let wall = block(
        &mut model,
        [3.0, -20.0, -20.0],
        [20.0, 4.0, 20.0],
        &Isometry::identity(),
    );
    let capped = kept(&mut model, ball, wall);
    let leaning = Isometry::rotation(&Axis::y(), -FRAC_PI_4).expect("a finite turn");
    assert_turns_z_onto(&leaning, [-(0.5f64.sqrt()), 0.0, 0.5f64.sqrt()]);
    let slab = block(
        &mut model,
        [-30.0, -30.0, -30.0],
        [30.0, 30.0, 0.0],
        &leaning,
    );
    let solid = kept(&mut model, capped, slab);
    for (at, want) in [
        ([4.0, 0.0, 0.0], Containment::Inside),
        ([3.5, 3.5, 0.0], Containment::Inside),
        ([3.5, 2.0, 2.0], Containment::Inside),
        ([3.2, 0.0, 3.4], Containment::Outside),
        ([0.0, 0.0, 0.0], Containment::Outside),
    ] {
        assert_eq!(placed(&model, solid, at), want, "{at:?}");
    }
}

/// That `turn` sends `z` to within a few ulps of `want`, so the slab's top
/// face is the plane the doc comments name.
fn assert_turns_z_onto(turn: &Isometry, want: [f64; 3]) {
    let got = turn
        .apply_to_vector(Vector::new(0.0, 0.0, 1.0))
        .components();
    assert!(
        (0..3).all(|axis| (got[axis] - want[axis]).abs() < 1e-15),
        "z turned is {got:?}, not {want:?}"
    );
}

/// Where `at` is against `solid`, under the certified policy.
fn placed(model: &Model, solid: SolidKey, at: [f64; 3]) -> Containment {
    model
        .solid(solid)
        .expect("this model made the key")
        .classify_point(
            Point::from(at),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
}

/// The one solid both `one` and `other` keep, in a model that validates.
fn kept(model: &mut Model, one: SolidKey, other: SolidKey) -> SolidKey {
    let answer = model
        .fuse(
            &[one],
            &[other],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the intersection refused: {why:?}"))
        .solids()
        .to_vec();
    let [solid] = answer[..] else {
        panic!("the intersection is one solid, not {answer:?}");
    };
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| {
            panic!("the intersection left a model that does not validate: {why}")
        });
    solid
}

/// A block from `low` to `high`, a rectangle extruded up `z`, then turned by
/// `turn` about the origin.
fn block(model: &mut Model, low: [f64; 3], high: [f64; 3], turn: &Isometry) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [low[0], low[1]],
        [high[0], low[1]],
        [high[0], high[1]],
        [low[0], high[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    let placed = turn
        .compose(&Isometry::translation(Vector::new(0.0, 0.0, low[2])).expect("a finite shift"));
    model
        .add_extrusion(&rectangle, high[2] - low[2], &placed)
        .expect("a block")
}
