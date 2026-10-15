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
//! `a_circle_touching_twice_what_two_planes_keep_is_refused_and_once_or_biting_followed`
//! pinned as refused until this row (row Q1350).
//!
//! **The flat ground off.** A ball of radius 3 kept to `x, y ≥ 1.5` by a
//! block, then to `x + y ≤ 4.2` by a turned slab. That plane's circle is
//! centred at `(2.1, 2.1, 0)` with radius `√0.18`, so it comes no nearer
//! either of the block's planes than `x = 1.8` and lies wholly inside the
//! lens: the ball's skin is the lens with a round hole, and the hole's circle
//! is a wire of its own with no corner on any other side. Every point whose
//! ray met that skin was refused, since a whole circle has no two ends to read
//! the way it runs between.
//!
//! **The band across the lens** (row Q2052). A ball of radius 3 kept to
//! `x ≥ 0.5` and `y ≥ 1.5` by a block, then under a slab turned 1.3 about
//! `(−1, 1, 0)` with its top at 2.4. The slab's circle crosses the side on
//! `x = 0.5` at `z ≈ 0.18` and `z ≈ 1.87`, and the side on `y = 1.5` at
//! `z ≈ −0.98` and `z ≈ 2.35`, so it takes a band right across the lens and
//! leaves the skin in two pieces, a three-sided tip at the top and the rest
//! below. Each is a face, and the part of the sphere in front of all three
//! planes is both of them, so no reading of the face's cuts tells it from the
//! other piece and every point whose ray met the skin was refused. The second
//! shape is the same with the block's corner at `(1.5, 1.5, −1)` and the
//! slab's top at 2.8.

use core::f64::consts::{FRAC_PI_2, FRAC_PI_4};

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

/// Each point is placed as the hand places it: in the ball, `x > 1.5`,
/// `y > 1.5` and `x + y < 4.2` is inside. `(2.12, 2.12, 0)` is in the ball and
/// in the block, but past `x + y = 4.2`, in the part ground off.
#[test]
fn a_ball_kept_to_a_blocks_corner_with_a_flat_ground_off_inside_it_places_each_point() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ball = model
        .add_ball(3.0, &Isometry::identity())
        .expect("a ball of radius 3");
    let corner = block(
        &mut model,
        [1.5, 1.5, -5.0],
        [9.0, 8.0, 10.0],
        &Isometry::identity(),
    );
    let lens = kept(&mut model, ball, corner);
    let upright = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(-1.0, 1.0, 0.0)).expect("a direction"),
        FRAC_PI_2,
    )
    .expect("a finite turn");
    assert_turns_z_onto(&upright, [0.5f64.sqrt(), 0.5f64.sqrt(), 0.0]);
    let top = 2.1 * 2f64.sqrt();
    let slab = block(
        &mut model,
        [-20.0, -20.0, top - 20.0],
        [20.0, 20.0, top],
        &upright,
    );
    let solid = kept(&mut model, lens, slab);
    for (at, want) in [
        ([2.0, 2.0, 0.0], Containment::Inside),
        ([1.8, 1.8, 1.0], Containment::Inside),
        ([1.6, 1.6, 1.9], Containment::Inside),
        ([2.0, 1.6, -1.5], Containment::Inside),
        ([2.12, 2.12, 0.0], Containment::Outside),
        ([0.0, 0.0, 0.0], Containment::Outside),
        ([1.0, 2.0, 0.0], Containment::Outside),
        ([1.6, 1.6, 2.0], Containment::Outside),
    ] {
        assert_eq!(placed(&model, solid, at), want, "{at:?}");
    }
}

/// Each point is placed as the hand places it: in the ball, `x > 0.5`,
/// `y > 1.5` and below the slab's top is inside. `(0.7, 1.7, 2.2)` is under
/// the tip, `(1.6, 1.8, 0.5)` is in the ball and the block but in the band the
/// slab takes away, and `(0.7, 1.7, 2.6)` is over the tip, outside the ball.
#[test]
fn a_ball_kept_to_a_blocks_corner_with_a_band_taken_across_it_places_each_point() {
    let (model, solid) = a_ball_kept_to_a_blocks_corner_under_a_slab([0.5, 1.5, -5.0], 2.4);
    for (at, want) in [
        ([0.95, 1.9, -0.9], Containment::Inside),
        ([0.95, 1.9, 0.9], Containment::Inside),
        ([1.9, 1.9, -0.9], Containment::Inside),
        ([1.6, 1.6, 0.5], Containment::Inside),
        ([2.0, 1.6, -1.5], Containment::Inside),
        ([0.7, 1.7, 2.2], Containment::Inside),
        ([0.0, 0.0, 0.0], Containment::Outside),
        ([-2.0, 0.0, 0.0], Containment::Outside),
        ([1.0, 1.0, -1.0], Containment::Outside),
        ([1.6, 1.8, 0.5], Containment::Outside),
        ([1.9, 1.9, 1.0], Containment::Outside),
        ([0.7, 1.7, 2.6], Containment::Outside),
    ] {
        assert_eq!(placed(&model, solid, at), want, "{at:?}");
    }
}

/// Each point is placed as the hand places it: in the ball, `x > 1.5`,
/// `y > 1.5`, `z > −1` and below the slab's top is inside. `(1.6, 1.6, 1.9)`
/// is under the tip, and `(1.9, 1.9, 1.0)` is in the ball and the block but in
/// the band the slab takes away.
#[test]
fn a_ball_kept_to_a_blocks_floored_corner_with_a_band_taken_across_it_places_each_point() {
    let (model, solid) = a_ball_kept_to_a_blocks_corner_under_a_slab([1.5, 1.5, -1.0], 2.8);
    for (at, want) in [
        ([1.9, 1.9, -0.9], Containment::Inside),
        ([1.6, 1.6, 0.5], Containment::Inside),
        ([1.6, 1.8, 0.5], Containment::Inside),
        ([1.6, 1.6, 1.9], Containment::Inside),
        ([0.0, 0.0, 0.0], Containment::Outside),
        ([0.95, 1.9, -0.9], Containment::Outside),
        ([1.6, 1.6, -1.5], Containment::Outside),
        ([2.0, 1.6, -1.5], Containment::Outside),
        ([1.9, 1.9, 1.0], Containment::Outside),
    ] {
        assert_eq!(placed(&model, solid, at), want, "{at:?}");
    }
}

/// A ball of radius 3 at the origin kept to the block from `corner` to
/// `(9, 8, 10)`, then under a slab turned 1.3 about `(−1, 1, 0)` with its top
/// at `top`, the plane `(s, s, cos 1.3) · p = top` with `s = sin 1.3 / √2`.
fn a_ball_kept_to_a_blocks_corner_under_a_slab(corner: [f64; 3], top: f64) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ball = model
        .add_ball(3.0, &Isometry::identity())
        .expect("a ball of radius 3");
    let block_corner = block(&mut model, corner, [9.0, 8.0, 10.0], &Isometry::identity());
    let lens = kept(&mut model, ball, block_corner);
    let leaning = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(-1.0, 1.0, 0.0)).expect("a direction"),
        1.3,
    )
    .expect("a finite turn");
    let across = 1.3f64.sin() / 2f64.sqrt();
    assert_turns_z_onto(&leaning, [across, across, 1.3f64.cos()]);
    let slab = block(
        &mut model,
        [-20.0, -20.0, top - 20.0],
        [20.0, 20.0, top],
        &leaning,
    );
    let solid = kept(&mut model, lens, slab);
    (model, solid)
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
