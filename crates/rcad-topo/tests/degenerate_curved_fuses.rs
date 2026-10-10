//! Roadmap §7.4's degeneracy catalogue over curved solids: the four entries
//! `degenerate_fuses.rs` could not reach with planes, and the other entries of
//! the same list asked again with a ball or a bar in them.
//!
//! §31.7 criterion 14. The planar file next door is the shape this follows, and
//! two things about it are deliberately not followed.
//!
//! # Two outcomes, which is what §38 left this file with
//!
//! The planar catalogue has two, answered and refused, and says a refusal is as
//! good an outcome as an answer provided it is right on its own terms. When §37
//! wrote this file it had four: those two, a **wrong answer** — a ball nothing
//! cuts came back `Ok` and empty, and every entry about a ball that does not
//! cross its partner inherited it — and a **panic**, on input a caller builds
//! through the public API, which §16.11 says may not happen.
//!
//! §38 closed all four defects, and this file is back to the planar file's two.
//! Every row that was a wrong answer is now the answer, and every row that was a
//! panic is a refusal by name or an answer. **The tests that pinned the wrong
//! behaviour were re-derived and not deleted** — §19's way round is that a test
//! asserts what the code does so that a fix fails it loudly, and each of the
//! seven that went red is now the same shape with the right assertion and a doc
//! saying which defect it was.
//!
//! What is left where a row refuses is classified in §38.6, `limit` or `gap`,
//! with the demonstration §16.10 asks for.
//!
//! # How each entry is known to reach what it is about
//!
//! §14.7's vacuity rule, and lane 5c's finding that a family can read 0.00%
//! because it never reaches the hard case. Every entry below is built so that
//! the degeneracy is *exact* — shared coordinates, or a Pythagorean triple, or a
//! plane through a stored radius — and every one of them is paired with a
//! control a tenth of a millimetre away that is **not** degenerate. Where the
//! degenerate case and its control come back the same way, the entry is not
//! reaching its subject and the row says so.
//!
//! # What a volume is compared against
//!
//! A curved solid measures through a mesh, so its volume is an approximation and
//! not a whole number (§25.4 is about the flat case, where it is). Every volume
//! here is judged against the closed form, at [`A_DEFLECTION`], within
//! [`HOW_FAR_A_MESH_MAY_BE`] — and **which side** it falls is asserted wherever
//! the geometry says: a mesh inscribed in a bar under-measures the bar, and a
//! hole cut by that same inscribed wall leaves too much material behind.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface,
    SurfaceMeet, Vector, intersect, where_a_closed_crossing_starts,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    Containment, DraftStep, EdgeKey, Error, Fused, Model, ProfileBuilder, Rule, SolidBuilder,
    SolidKey, WireView,
};

mod cones;

/// How fine the meshes every volume here is read off are.
const A_DEFLECTION: f64 = 0.02;

/// How far from the closed form a volume measured at [`A_DEFLECTION`] may be.
///
/// Half a per cent. The worst any answered entry here comes to is 0.48%, on the
/// bar of radius one wholly inside a block, and the reason a smaller radius is
/// further out is that a deflection is an absolute length: the same 0.02 buys
/// fewer sides round a smaller circle. It is a ceiling on a measurement rather
/// than a tolerance a decision is taken against — nothing in the kernel reads
/// it.
const HOW_FAR_A_MESH_MAY_BE: f64 = 0.005;

/// A bar of radius two and height eight, which most of these entries cut.
const A_BAR: f64 = PI * 4.0 * 8.0;

/// A ball of radius four.
const A_BIG_BALL: f64 = 4.0 / 3.0 * PI * 64.0;

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// A box of the given lengths, with its low corner at `at`.
fn block(model: &mut Model, [along_x, along_y, along_z]: [f64; 3], at: Vector) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            along_z,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("building a box needs no decision")
}

/// The bar of radius two and height eight, about the z axis, with its seam at
/// `(2, 0, z)`.
fn a_bar(model: &mut Model) -> SolidKey {
    model
        .add_round_bar(2.0, 8.0, &Isometry::identity())
        .expect("a bar of radius two")
}

/// The two sides of a fuse, as the catalogue builds them.
struct Pair {
    arguments: Vec<SolidKey>,
    tools: Vec<SolidKey>,
}

impl Pair {
    fn of(argument: SolidKey, tool: SolidKey) -> Self {
        Pair {
            arguments: vec![argument],
            tools: vec![tool],
        }
    }
}

/// Fuse under a certified policy and insist nothing had to be approximated.
///
/// Every pair here is written on a small lattice or from a Pythagorean triple,
/// so the incidences are exact and the boxes the arrangement solves for collapse
/// onto one. A record here would be a decision nobody proved.
fn fused(model: &mut Model, pair: &Pair, rule: Rule) -> Fused {
    fused_under(model, pair, rule, &Policy::certified())
}

/// The same under `policy`, which still records nothing: a tolerant policy
/// only loosens what a proof could not settle, and here every sign is proved.
fn fused_under(model: &mut Model, pair: &Pair, rule: Rule, policy: &Policy) -> Fused {
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(&pair.arguments, &pair.tools, rule, policy, &mut log)
        .unwrap_or_else(|why| panic!("this {rule:?} is decided by proof alone: {why}"));
    assert!(
        log.records().is_empty(),
        "a certified fuse of an exactly written pair records nothing at all, got {:?}",
        log.records()
    );
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a fuse writes a sound model or writes nothing");
    answer
}

/// The refusal a fuse came back with, or a panic naming what it did instead.
fn refusal(model: &mut Model, pair: &Pair, rule: Rule, policy: &Policy) -> Error {
    match model.fuse(
        &pair.arguments,
        &pair.tools,
        rule,
        policy,
        &mut ProvenanceLog::new(),
    ) {
        Err(why) => why,
        Ok(fused) => panic!("expected a refusal, got {} solids", fused.solids().len()),
    }
}

/// Which predicate a refusal named last, or a panic if it was not a refusal.
fn what_it_gave_up_on(why: &Error) -> &str {
    match why {
        Error::Undecidable { records } => records
            .records()
            .last()
            .expect("a refusal names at least one decision")
            .predicate(),
        other => panic!("expected `Undecidable`, got {other:?}"),
    }
}

/// How much space the answer takes up, all of its solids together, at
/// [`A_DEFLECTION`].
fn volume_of(model: &Model, solids: &[SolidKey]) -> f64 {
    volume_of_at(model, solids, A_DEFLECTION)
}

/// The same at a deflection of this row's own choosing.
///
/// **Only for a row whose answer is thin**, and the reason is the one
/// [`HOW_FAR_A_MESH_MAY_BE`] already gives: a deflection is an absolute length,
/// so the same 0.02 buys a smaller share of a small shape. The lens between two
/// balls of radius two is 1.6 units at its widest and 10.47 in volume, and
/// [`A_DEFLECTION`] leaves it 0.725% short where a fifth of it leaves 0.062%.
fn volume_of_at(model: &Model, solids: &[SolidKey], deflection: f64) -> f64 {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(deflection).expect("a fiftieth of a millimetre is a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a solid this size measures")
                .volume()
        })
        .sum()
}

/// Insist a measured volume is within [`HOW_FAR_A_MESH_MAY_BE`] of the closed
/// form, on the side the geometry says it should be.
fn measures(found: f64, closed_form: f64, side: Side) {
    let apart = (found - closed_form).abs() / closed_form;
    assert!(
        apart < HOW_FAR_A_MESH_MAY_BE,
        "measured {found}, closed form {closed_form}, {:.3}% apart",
        100.0 * apart
    );
    match side {
        Side::Under => assert!(found < closed_form, "{found} is not under {closed_form}"),
        Side::Over => assert!(found > closed_form, "{found} is not over {closed_form}"),
        Side::EitherWay => {}
    }
}

/// Which side of the closed form a meshed volume has to land.
#[derive(Clone, Copy, Debug)]
enum Side {
    /// A mesh inscribed in a round wall is inside the solid, so it measures
    /// less than the solid does.
    Under,
    /// The same inscribed wall bounding a *hole* leaves the hole too small, so
    /// what is left round it measures more.
    Over,
    /// A round outer wall pulls the measure under and a round hole pulls it
    /// over, so which way it lands is the mesh's doing and not the shape's.
    EitherWay,
}

// 1. Tangent spheres — §31.7 criterion 14's first entry.

/// Two balls of radius two whose centres are four apart, so they touch at
/// exactly one point and share no space at all.
///
/// The coincidence is exact: `4 - 2 = 2` and `2 + 2 = 4` in whole numbers, so
/// the touching point `(2, 0, 0)` is the same `f64` read from either ball.
fn two_balls_touching_at_one_point(model: &mut Model) -> Pair {
    let one = model
        .add_ball(2.0, &Isometry::identity())
        .expect("a ball of radius two");
    let other = model
        .add_ball(
            2.0,
            &Isometry::translation(Vector::new(4.0, 0.0, 0.0)).expect("a whole-number shift"),
        )
        .expect("a ball of radius two");
    Pair::of(one, other)
}

/// Two balls that touch at one point unite into two solids that touch, which is
/// §38.1's fix of the wrong answer §37.4 pinned here.
///
/// The planar catalogue's `two_cubes_that_only_touch_at_a_corner` is the same
/// question with corners and gives exactly the same shape of answer. What comes
/// back now is both balls under `Union`, nothing at all under `Intersection` —
/// one point has no volume, and an empty intersection is a right answer and not
/// a dropped solid — and the first ball alone under `Difference`.
///
/// **This entry still does not reach tangency**, and saying otherwise would be
/// the vacuity §14.7 exists to stop: neither ball's face is cut by the other, so
/// the same three answers come back when the two balls are a mile apart. What it
/// does now reach is the thing that was wrong — a ball nothing cuts —
/// and [`a_ball_that_nothing_cuts_comes_back_whole`] is the root of it.
#[test]
fn two_balls_that_touch_at_one_point_unite_into_two_solids_that_touch() {
    let each_one = 4.0 / 3.0 * PI * 8.0;
    let mut model = a_model();
    let pair = two_balls_touching_at_one_point(&mut model);
    measures(volume_of(&model, &pair.arguments), each_one, Side::Under);

    for (rule, how_many, closed_form) in [
        (Rule::Union, 2, 2.0 * each_one),
        (Rule::Difference, 1, each_one),
    ] {
        let mut model = a_model();
        let pair = two_balls_touching_at_one_point(&mut model);
        let answer = fused(&mut model, &pair, rule);
        assert_eq!(answer.solids().len(), how_many, "{rule:?}");
        measures(volume_of(&model, answer.solids()), closed_form, Side::Under);
    }

    let mut model = a_model();
    let pair = two_balls_touching_at_one_point(&mut model);
    let shared = fused(&mut model, &pair, Rule::Intersection);
    assert!(
        shared.solids().is_empty(),
        "what two balls touching at one point share has no volume"
    );
}

/// The root of it, in one fuse and one solid: a ball fused with no tool at all
/// comes back whole, seam, poles and volume.
///
/// A ball is one face whose one wire is a seam, and a seam is left out at the
/// door — so the arrangement of that face against no cuts has no curve drawn on
/// it at all, no cell, and no piece. It used to come back as `Ok` and nothing.
/// §38.1 makes the empty arrangement mean what it says: the whole surface, cut
/// open along a seam of the answer's own.
///
/// **A bar in the same position is the control** and always came back whole: a
/// bar has three faces and its wall is bounded by a seam *and* two cap circles,
/// so its arrangement has curves on it.
///
/// The four steps are what say the seam is a seam. A ball's face is one wire of
/// two coedges — the seam walked each way — with a pole standing between them,
/// which is what [`Model::add_ball`] builds and what the mesher reads.
#[test]
fn a_ball_that_nothing_cuts_comes_back_whole() {
    let mut model = a_model();
    let ball = model
        .add_ball(2.0, &Isometry::identity())
        .expect("a ball of radius two");
    measures(
        volume_of(&model, &[ball]),
        4.0 / 3.0 * PI * 8.0,
        Side::Under,
    );

    let kept = fused(
        &mut model,
        &Pair {
            arguments: vec![ball],
            tools: Vec::new(),
        },
        Rule::Union,
    );
    assert_eq!(kept.solids().len(), 1, "a ball fused on its own comes back");
    measures(
        volume_of(&model, kept.solids()),
        4.0 / 3.0 * PI * 8.0,
        Side::Under,
    );

    let skin = model
        .solid(kept.solids()[0])
        .expect("this model made the key");
    assert_eq!(skin.faces().count(), 1, "a ball is one face");
    let wires: Vec<_> = skin
        .faces()
        .flat_map(|face| face.wires().collect::<Vec<_>>())
        .collect();
    assert_eq!(wires.len(), 1);
    assert_eq!(wires[0].step_count(), 4, "a seam twice and a pole between");
    assert_eq!(
        wires[0].coedges().count(),
        2,
        "two of the four are the seam"
    );

    let bar = a_bar(&mut model);
    let kept = fused(
        &mut model,
        &Pair {
            arguments: vec![bar],
            tools: Vec::new(),
        },
        Rule::Union,
    );
    assert_eq!(kept.solids().len(), 1, "a bar fused on its own comes back");
    measures(volume_of(&model, kept.solids()), A_BAR, Side::Under);
}

/// A ball turned about an awkward axis and a ball a long way from the origin
/// both come back with a ball's volume.
///
/// A ball has no frame of its own, so the seam the answer carries is the one
/// `Sphere::point_at(0, ·)` runs along in the world's axes, whatever the input
/// was turned by. **Carrying the input's own seam through instead is the trap**,
/// and it is invisible on a ball standing where it was built, where the two are
/// the same meridian. This is the row where they are not.
///
/// **What is *not* load-bearing, measured rather than argued.** Naming the other
/// half of the meridian changes nothing anywhere: both halves run pole to pole,
/// so both cut the surface open along a meridian and the patch is the same
/// rectangle shifted. Flipping it leaves all 368 tests of this crate green.
/// Putting the poles on the equator instead — still on the ball and still on the
/// meridian plane, so every check about *where* they are passes — is caught by
/// the plainest ball there is: the triangulator refuses a patch whose corner
/// stands on an earlier one, and every ball row in this file goes red at once. So
/// that one needs no shape of its own either.
#[test]
fn a_turned_ball_and_a_far_one_come_back_with_the_volume_of_a_ball() {
    let closed_form = 4.0 / 3.0 * PI * 8.0;
    let axis = Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("an axis");
    for placement in [
        Isometry::rotation(&axis, 0.4).expect("a share of a turn"),
        Isometry::translation(Vector::new(37.0, -19.0, 23.0)).expect("a shift"),
    ] {
        let mut model = a_model();
        let ball = model
            .add_ball(2.0, &placement)
            .expect("a ball of radius two");
        let kept = fused(
            &mut model,
            &Pair {
                arguments: vec![ball],
                tools: Vec::new(),
            },
            Rule::Union,
        );
        assert_eq!(kept.solids().len(), 1);
        measures(volume_of(&model, kept.solids()), closed_form, Side::Under);
    }
}

/// Two balls of radius two, `apart` between their centres.
fn two_balls_that_cross(model: &mut Model, apart: f64) -> Pair {
    let one = model
        .add_ball(2.0, &Isometry::identity())
        .expect("a ball of radius two");
    let other = model
        .add_ball(
            2.0,
            &Isometry::translation(Vector::new(apart, 0.0, 0.0)).expect("a shift"),
        )
        .expect("a ball of radius two");
    Pair::of(one, other)
}

/// The control that says the family is not empty everywhere: two balls that
/// really do cross each other are **kept apart from two that only touch**.
///
/// The lens between them is answered against its closed form, which is the row
/// that shows the arithmetic can tell a crossing pair from a touching one — what
/// it cannot do is keep the touching one.
///
/// `π(4r + d)(2r − d)²/12` is the lens of two balls of one radius `d` apart, so
/// `10π/3` at two apart and `11π/12` at three. Measured at a fifth of
/// [`A_DEFLECTION`] because a lens is thin — see [`volume_of_at`].
///
/// **Three apart used to refuse and now answers**, and it is the second half of
/// what §41.6 found. The lens there spans under a quarter turn of either ball,
/// so the coarsest mesh — the one the fuse reads a shell's *sign* off, not the
/// one this file asks for — put no point inside it and it came back flat,
/// enclosing exactly nothing, and the fuse refused `VolumeIsNotPositive`. The
/// sign is read off a finer mesh now when a coarse one encloses nothing, and
/// this is the smallest shape in the suite that reaches it.
#[test]
fn what_two_balls_that_really_cross_share_is_the_lens_between_them() {
    for (apart, lens) in [(2.0, 10.0 * PI / 3.0), (3.0, 11.0 * PI / 12.0)] {
        let mut model = a_model();
        let pair = two_balls_that_cross(&mut model, apart);
        let shared = fused(&mut model, &pair, Rule::Intersection);
        assert_eq!(
            shared.solids().len(),
            1,
            "two balls {apart} apart share one lens"
        );
        measures(
            volume_of_at(&model, shared.solids(), A_DEFLECTION / 5.0),
            lens,
            Side::Under,
        );
    }
}

/// Two crossing balls unite into one solid, which is both balls less the lens
/// they share.
///
/// Each ball keeps the part of its skin **outside** the other, and that piece
/// holds both of its poles: it is the whole ball less a disc. That was refused
/// `ACurvedFaceGoesRightRoundWithNoSeam` until the mesher learned to open such a
/// face along a seam half a turn from its rim (§202.1). Before the mesher could
/// tell the two pieces apart at all, this came back `Ok` measuring 10.40, the
/// lens, where the union is 56.55.
#[test]
fn what_is_round_two_crossing_balls_is_their_union() {
    let one_ball = 4.0 / 3.0 * PI * 8.0;
    for (apart, lens) in [(2.0, 10.0 * PI / 3.0), (3.0, 11.0 * PI / 12.0)] {
        let mut model = a_model();
        let pair = two_balls_that_cross(&mut model, apart);
        let round = fused(&mut model, &pair, Rule::Union);
        assert_eq!(
            round.solids().len(),
            1,
            "two balls {apart} apart unite into one solid"
        );
        measures(
            volume_of_at(&model, round.solids(), A_DEFLECTION / 5.0),
            2.0 * one_ball - lens,
            Side::Under,
        );
    }
}

// 2. Tangent cylinder–plane — criterion 14's second entry.

/// A bar of radius two and a block whose near wall is the plane `y = -2`, which
/// touches the bar's skin along the ruling `(0, -2, z)` and crosses it nowhere.
///
/// Exact: the wall's offset and the bar's radius are the same whole number, so
/// `x² + y² - r²` on that ruling cancels to zero rather than nearly to zero.
fn a_block_whose_wall_touches_a_bar(model: &mut Model) -> Pair {
    let bar = a_bar(model);
    let touching = block(model, [2.0, 3.0, 2.0], Vector::new(1.0, -2.0, 3.0));
    Pair::of(bar, touching)
}

/// The tangency is answered under every rule: the wall lies **on** the bar's
/// skin along the ruling `(0, -2, z)`, and that crossing runs up the bar's axis.
///
/// **The two curves the old name spoke of are ordered, and they were on the
/// caps and not on the wall.** On the plane `z = 0`, and again on `z = 8`, the
/// cap's own boundary circle and the line the wall `y = -2` cuts in the cap
/// plane touch at `(0, -2, 0)` and `(0, -2, 8)`; the three ways out of that node
/// come back in the order their curvatures put them.
///
/// **What stopped it until queue row Q649 was a stage further in.** The wall
/// `y = -2` is tangent to the bar's *surface* along the whole ruling, where the
/// plane's normal `(0, -1, 0)` and the pipe's gradient `(0, -2, 0)` are
/// parallel. `∇S × ∇Q`, which every other crossing runs along, is exactly the
/// zero vector there, and every rule was refused `Undecidable` on *which way a
/// crossing runs*. It was a gap: the line runs along the pipe's stored axis,
/// which `arrange.rs`'s `AlongATouch` now carries, along with the one side of
/// the wall both sides of the line lie on.
///
/// The block also crosses the bar: its wall `x = 1` cuts it. So the shared
/// piece is the slab `3 ≤ z ≤ 5` of the bar with `x ≥ 1` and `y ≤ 1`, and the
/// wall at `y = -2` cuts nothing off it. Its area is `∫(√(4 − y²) − 1) dy` from
/// `−√3` to `1`, which is exactly `π − 1`, as in
/// [`a_bar_with_a_tab_on_it_is_answered_rather_than_left_with_a_hole_in_its_skin`].
///
/// **The control is one coordinate moved by a tenth, and it is answered too.**
/// With the wall at `y = -1.9` it cuts the bar in two curves that really do
/// cross. It used to refuse by a different name — `Undecidable` over a point
/// inside a piece of a face, §36.28's chord-and-arc gap — until counting a
/// pipe's bulge in `select.rs` closed that. Its shared piece is the same
/// integral up to `1.1`.
///
/// A tolerant policy answers too: the touch is decided by the shape and no
/// budget is spent on it.
#[test]
fn a_wall_lying_on_a_bars_skin_runs_up_the_bars_axis_and_is_answered() {
    let shared = 2.0 * (PI - 1.0);
    for (rule, closed_form) in [
        (Rule::Union, A_BAR + 12.0 - shared),
        (Rule::Intersection, shared),
        (Rule::Difference, A_BAR - shared),
    ] {
        let mut model = a_model();
        let pair = a_block_whose_wall_touches_a_bar(&mut model);
        let answer = fused(&mut model, &pair, rule);
        assert_eq!(answer.solids().len(), 1, "{rule:?}");
        measures(volume_of(&model, answer.solids()), closed_form, Side::Under);

        let mut model = a_model();
        let pair = a_block_whose_wall_touches_a_bar(&mut model);
        let tolerant = model
            .fuse(
                &pair.arguments,
                &pair.tools,
                rule,
                &Policy::tolerant(100.0),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("a tolerant {rule:?} refused: {why}"));
        assert_eq!(tolerant.solids().len(), 1, "a tolerant {rule:?}");
    }

    let mut model = a_model();
    let bar = a_bar(&mut model);
    let crossing = block(&mut model, [2.0, 3.0, 2.0], Vector::new(1.0, -1.9, 3.0));
    let united = fused(&mut model, &Pair::of(bar, crossing), Rule::Union);
    assert_eq!(
        united.solids().len(),
        1,
        "moving the wall off the tangent gets the same kind of answer"
    );
    let shared = 2.0
        * (0.55 * 2.79_f64.sqrt() + 2.0 * 0.55_f64.asin() + 2.0 * PI / 3.0
            - 1.1
            - 3.0_f64.sqrt() / 2.0);
    measures(
        volume_of(&model, united.solids()),
        A_BAR + 12.0 - shared,
        Side::Under,
    );
}

// 2b. Tangent cone–plane — the same entry, asked of a cone instead of a bar.

/// How far out per unit up the cone of this entry leans.
///
/// An eighth. Two things want a power of two here. `cones/mod.rs` wants one so
/// that `slope · height` is exact and the frustum's own corners sit on it. This
/// entry wants one for a second reason: a plane written `(1, 0, −slope)` keeps
/// `n_z = −slope · n_x` **bit for bit** through the normalizing that
/// [`Plane::from_point_and_normal`] does, because scaling by a power of two
/// commutes with rounding. That relation is the whole of the tangency —
/// [`the_plane_of_the_wedges_slanted_face_really_does_rest_on_the_cone`]
/// measures it rather than assuming it.
const A_CONES_SLOPE: f64 = 0.125;

/// Where the frustum starts and stops. The wedge lives at `8 ≤ z ≤ 24`, well
/// inside both, so no corner of it lands on a cap.
const A_CONES_NEAR_END: f64 = 4.0;
const A_CONES_FAR_END: f64 = 28.0;

/// The cone the wedge rests on, and the shallower one that is the control.
fn a_cone_of_slope(slope: f64) -> Cone {
    Cone::new(Axis::z(), slope).expect("a positive slope")
}

/// The plane `x = slope · z`, through the apex and along the ruling
/// `(t/8, 0, t)`, facing out of the wedge below.
fn a_plane_resting_on_a_cones_side() -> Plane {
    Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, -A_CONES_SLOPE))
        .expect("a normal with length")
}

/// A wedge of four corners and four flat faces, one of them lying on
/// [`a_plane_resting_on_a_cones_side`].
///
/// **Why not the block the bar entry uses.** A block standing beside a cone has
/// four faces parallel to the cone's axis, and a plane parallel to the axis cuts
/// a cone in a hyperbola — a run nothing here can order along. The fuse would
/// give up on one of those long before it reached the tangency, and the entry
/// would be measuring something else. So every face of this wedge but the
/// tangent one is made to cut the cone in a **closed** curve.
///
/// Which is a sign the kernel settles exactly, on
/// `E = (n·w)² − slope²‖n × w‖²`. With the axis up `z` that is
/// `n_z² − slope²(n_x² + n_y²)`, and on the four normals below, written as whole
/// numbers, it comes to
///
/// ```text
/// (8, 0, −1)      1 − 64/64   = 0      the tangency: one ruling, touched
/// (0, 0, −1)      1 − 0       = 1      a circle
/// (−2, 6, 1)      1 − 40/64   = 24/64  an ellipse
/// (−2, −3, 1)     1 − 13/64   = 51/64  an ellipse
/// ```
///
/// Every one of those is a whole number over 64 of a sum of whole numbers, so
/// the four signs are what they read.
///
/// The wedge really does overlap the cone: the axis point `(0, 0, 9)` is inside
/// all four of its half-spaces and inside the cone, whose radius there is
/// `9/8`.
///
/// **Its back corner is at `(−5, 1, 8)` and not `(−5, 0, 8)`**, and
/// [`A_BACK_WHOSE_ELLIPSE_STARTS_ON_THE_RULING`] says what the other one meets.
fn a_wedge_resting_on_a_cones_side(model: &mut Model) -> SolidKey {
    a_wedge_resting_on_a_cones_side_with_its_back_at(model, THE_WEDGES_BACK)
}

/// The wedge's back corner and the normals of the two faces through it, as
/// [`a_wedge_resting_on_a_cones_side`] writes them.
const THE_WEDGES_BACK: Back = Back {
    at: [-5.0, 1.0, 8.0],
    one_side: [-2.0, 6.0, 1.0],
    other_side: [-2.0, -3.0, 1.0],
};

/// The back corner this entry was first written with, `(−5, 0, 8)`, where the
/// face `(−8, −16, 5)` through it cuts the cone in an ellipse that **starts**
/// on the ruling the slanted face rests on.
///
/// Every closed crossing starts at one place the two surfaces alone pick
/// (`where_a_closed_crossing_starts`), and this ellipse's is `(2.5, 0, 20)`,
/// which is also where the wedge's edge on the slanted face touches the
/// ellipse. That node is a meet of two planes and a cone at a double root, a
/// box and not a point, so sweeping round the ellipse from its start cannot
/// tell the node from the start, and the order round it is refused.
/// [`a_wedge_whose_ellipse_starts_where_its_edge_touches_it_is_refused_on_the_order_round_it`]
/// pins it.
const A_BACK_WHOSE_ELLIPSE_STARTS_ON_THE_RULING: Back = Back {
    at: [-5.0, 0.0, 8.0],
    one_side: [-8.0, 16.0, 3.0],
    other_side: [-8.0, -16.0, 5.0],
};

/// Where a wedge resting on the cone's side has its back corner, and the normals
/// of the two faces that meet there, pointing out of the wedge.
#[derive(Clone, Copy)]
struct Back {
    at: [f64; 3],
    one_side: [f64; 3],
    other_side: [f64; 3],
}

/// [`a_wedge_resting_on_a_cones_side`] with its back corner somewhere else on
/// the floor `z = 8`. The slanted face and the floor are the same planes.
fn a_wedge_resting_on_a_cones_side_with_its_back_at(model: &mut Model, back_at: Back) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let corner = |builder: &mut SolidBuilder, [x, y, z]: [f64; 3]| {
        builder
            .add_vertex(Point::new(x, y, z))
            .expect("a finite point")
    };
    let left = corner(&mut builder, [1.0, -3.0, 8.0]);
    let right = corner(&mut builder, [1.0, 3.0, 8.0]);
    let back = corner(&mut builder, back_at.at);
    let up = corner(&mut builder, [3.0, 1.0, 24.0]);

    let mut edge = |from, to| {
        builder
            .add_edge(from, to, EdgeRun::Straight)
            .expect("two corners of this build")
    };
    let left_to_right = edge(left, right);
    let left_to_back = edge(left, back);
    let right_to_back = edge(right, back);
    let left_to_up = edge(left, up);
    let right_to_up = edge(right, up);
    let back_to_up = edge(back, up);

    let mut face = |normal: Vector, through: Point, round: [(_, Orientation); 3]| {
        let steps: Vec<DraftStep> = round
            .into_iter()
            .map(|(edge, way)| {
                DraftStep::Along(
                    builder
                        .add_coedge(edge, way)
                        .expect("an edge of this build"),
                )
            })
            .collect();
        let wire = builder.add_wire(&steps).expect("three edges close");
        let plane = Plane::from_point_and_normal(through, normal).expect("a normal with length");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a triangle on its own plane")
    };
    let resting = face(
        Vector::new(1.0, 0.0, -A_CONES_SLOPE),
        Point::origin(),
        [
            (left_to_right, Orientation::Forward),
            (right_to_up, Orientation::Forward),
            (left_to_up, Orientation::Reversed),
        ],
    );
    let bottom = face(
        Vector::new(0.0, 0.0, -1.0),
        Point::new(0.0, 0.0, 8.0),
        [
            (left_to_back, Orientation::Forward),
            (right_to_back, Orientation::Reversed),
            (left_to_right, Orientation::Reversed),
        ],
    );
    let [ax, ay, az] = back_at.at;
    let [ox, oy, oz] = back_at.one_side;
    let [tx, ty, tz] = back_at.other_side;
    let one_side = face(
        Vector::new(ox, oy, oz),
        Point::new(1.0, 3.0, 8.0),
        [
            (right_to_back, Orientation::Forward),
            (back_to_up, Orientation::Forward),
            (right_to_up, Orientation::Reversed),
        ],
    );
    let other_side = face(
        Vector::new(tx, ty, tz),
        Point::new(ax, ay, az),
        [
            (left_to_back, Orientation::Reversed),
            (left_to_up, Orientation::Forward),
            (back_to_up, Orientation::Reversed),
        ],
    );

    builder
        .add_shell(&[
            (resting, Orientation::Forward),
            (bottom, Orientation::Forward),
            (one_side, Orientation::Forward),
            (other_side, Orientation::Forward),
        ])
        .expect("four faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a wedge described this way is a solid")
}

/// The frustum of [`A_CONES_SLOPE`] and the wedge whose slanted face rests along
/// its side.
fn a_wedge_lying_on_a_cones_side(model: &mut Model) -> Pair {
    let cone = cones::a_frustum(model, A_CONES_SLOPE, A_CONES_NEAR_END, A_CONES_FAR_END);
    let wedge = a_wedge_resting_on_a_cones_side(model);
    Pair::of(cone, wedge)
}

/// **Before anything is fused: the plane really is tangent, and here are the
/// numbers.**
///
/// Three of them, each stronger than the last.
///
/// 1. The stored normal, after normalizing, still satisfies
///    `n_z = −slope · n_x` exactly — the assertion is `==` on `f64`, not a
///    tolerance.
/// 2. `E = n_z² − slope²(n_x² + n_y²)`, which is the kernel's own way of asking
///    how a plane leans against a cone, is exactly `0.0`. Zero is the tangency;
///    above it the plane cuts across the cone, below it the plane is steeper and
///    reaches both nappes.
/// 3. `intersect` — asked the question in exact arithmetic, not in `f64` —
///    hands back **one straight run**, which is the row of its own table
///    reading "apex on the plane, `E = 0`: one ruling, touched along its whole
///    length".
///
/// And the control cone says the same thing from the other side: halve the
/// slope and the very same plane no longer rests on the cone at all — it now
/// leans across it, and touches only at the apex.
#[test]
fn the_plane_of_the_wedges_slanted_face_really_does_rest_on_the_cone() {
    let flat = a_plane_resting_on_a_cones_side();
    let [across_x, across_y, up_z] = flat.normal().components();
    assert_eq!(across_y, 0.0, "the plane holds the y direction");
    assert_eq!(
        up_z,
        -A_CONES_SLOPE * across_x,
        "normalizing kept the slope relation bit for bit"
    );
    let leaning =
        up_z * up_z - A_CONES_SLOPE * A_CONES_SLOPE * (across_x * across_x + across_y * across_y);
    assert_eq!(
        leaning, 0.0,
        "the plane leans at exactly the cone's own half-angle"
    );
    // The ruling itself: `(1, 0, 8)` is on the plane — `1 = 8/8` — and on the
    // cone — `1² + 0² = (8/8)²`. Its distance to the plane is not small, it is
    // zero.
    assert_eq!(
        flat.signed_distance_to(Point::new(1.0, 0.0, 8.0)),
        0.0,
        "a point of the ruling is on the plane exactly"
    );

    let touching = intersect(
        &Surface::Plane(flat),
        &Surface::Cone(a_cone_of_slope(A_CONES_SLOPE)),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("all seven shapes of a plane against a cone are named");
    let SurfaceMeet::Curves(runs) = &touching else {
        panic!("a plane resting on a cone's side touches along a curve, got {touching:?}")
    };
    assert_eq!(runs.len(), 1, "a tangent plane touches along one ruling");
    assert!(
        matches!(runs[0].run(), EdgeRun::Straight),
        "and a ruling is a straight line, not a bending run: {:?}",
        runs[0].run()
    );

    let leaning_across = intersect(
        &Surface::Plane(flat),
        &Surface::Cone(a_cone_of_slope(A_CONES_SLOPE / 2.0)),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("all seven shapes of a plane against a cone are named");
    assert!(
        matches!(leaning_across, SurfaceMeet::TouchAtOnePoint { .. }),
        "halving the slope takes the plane off the side: {leaning_across:?}"
    );
}

/// The wedge is the solid it is meant to be, and it really does cut the cone.
///
/// §14.7's vacuity rule. A tangency nothing reaches is a row about nothing, so
/// the two things the entry leans on are measured: the wedge holds the 96 units
/// its four corners say it should, and the point `(0, 0, 9)` — on the cone's
/// axis, where the cone's radius is `9/8` — is **inside both**.
#[test]
fn the_wedge_and_the_cone_it_rests_on_really_do_share_space() {
    let policy = Policy::certified();
    let mut model = a_model();
    let pair = a_wedge_lying_on_a_cones_side(&mut model);
    let [cone, wedge] = [pair.arguments[0], pair.tools[0]];

    let held = volume_of(&model, &[wedge]);
    assert!(
        (held - 96.0).abs() < 1e-9,
        "the wedge holds 96, measured {held}"
    );
    for (solid, named) in [(cone, "the frustum"), (wedge, "the wedge")] {
        assert_eq!(
            model
                .solid(solid)
                .expect("this model made the key")
                .classify_point(
                    Point::new(0.0, 0.0, 9.0),
                    &policy,
                    &mut ProvenanceLog::new()
                ),
            Containment::Inside,
            "(0, 0, 9) is inside {named}"
        );
    }
}

/// **A wall lying on a cone's side runs along the ruling it rests on, and every
/// fuse of the wedge and the frustum is answered** and held to the volumes
/// worked out level by level without the kernel.
///
/// This is [`a_wall_lying_on_a_bars_skin_runs_up_the_bars_axis_and_is_answered`]
/// asked of a cone. Along a ruling a plane rests on, the plane's normal and the
/// cone's gradient are parallel at every point of it, so `∇S × ∇Q` is exactly
/// the zero vector, and every fuse here used to refuse *which way a crossing
/// runs* on it. That was a gap (§16.10): the direction is the ruling's own,
/// `(1, 0, 8)` up to length, and `arrange.rs` now reads it off the plane's
/// normal and the cone's axis once the touch is proved (`AlongATouch`, queue row
/// Q473). Each of the wedge's other faces meets the slanted one in an edge that
/// touches that face's conic of the cone where it crosses the ruling, and the
/// turn there is read as the touch it is proved to be.
///
/// **The union is answered, either way round and under either policy**, one
/// solid that validates and measures `403.3953` against the hand-worked
/// `403.7413`, under it as an inscribed mesh is. It keeps the wedge's floor less
/// the circle the frustum cuts in it at `z = 8`, and that circle touches the
/// floor's edge at `(1, 0, 8)`, so the floor's wire passes that corner twice.
/// The mesher refused it `CornersCoincide` until it learned to cut such a wire
/// there into an outline and a hole touching it (§364).
///
/// **The wedge less the frustum is answered**, built either way round and
/// under either policy: one solid that validates and measures `45.6815`
/// against the hand-worked `96 − shared = 45.5997`, over it, since the cone's
/// skin bounds a hole in this answer and an inscribed mesh leaves the hole too
/// small. It is two slivers either side of the ruling, each between the
/// slanted face and the cone and thin to nothing at the ruling, joined round
/// the back of the cone below `z ≈ 14.9`, so one solid that touches itself
/// along the ruling. **What the fuse writes is a sound model**, printed and
/// walked under queue row Q953: six faces, and every edge used by exactly two
/// coedges. The slanted face is kept whole, with `(1, 0, 8)` and
/// `(2.5, 0, 20)` on its outline and no edge between them, and the cone is cut
/// open along the ruling, which is one edge used twice by the cone alone — the
/// way `fuse/assemble.rs` parts a line two faces rest against each other along,
/// and what [`uses_of_the_ruling`] checks.
///
/// **It used to be refused by the mesher**, `TwoPatchesOfTheMeshDrewTheSameSide
/// { drawn: 2, used: 4 }`, and before §364 `CornersCoincide`, like the union.
/// The two ends of the ruling sit in the middle of two straight sides of the
/// slanted face, so both of its triangulations without a flat triangle draw
/// the side between them, and the cone's seam was that same side, walked with
/// no point between its ends. The mesher now puts a point in the middle of a
/// straight edge lying across a flat face of its own solid. Meshed finer, the
/// answer comes down to the hand-worked volume, `45.6200` at a deflection of
/// `0.005` and `45.6051` at `0.00125`.
///
/// **The control halves the cone's slope and nothing else.** The same wedge
/// against a cone of slope a sixteenth: the slanted face no longer rests on the
/// side, it leans across it and meets the cone at the apex alone, and the union
/// is answered.
#[test]
fn a_wedge_lying_on_a_cones_side_shares_and_leaves_what_is_worked_out_by_hand() {
    let frustum =
        PI * A_CONES_SLOPE * A_CONES_SLOPE * (A_CONES_FAR_END.powi(3) - A_CONES_NEAR_END.powi(3))
            / 3.0;
    let shared = what_the_wedge_shares_with_a_cone_of_slope(A_CONES_SLOPE, THE_WEDGES_BACK);
    for turned in [false, true] {
        let mut model = a_model();
        let pair = either_way_round(a_wedge_lying_on_a_cones_side(&mut model), turned);
        let both = fused(&mut model, &pair, Rule::Intersection);
        assert_eq!(both.solids().len(), 1);
        measures(volume_of(&model, both.solids()), shared, Side::Under);
    }
    let mut model = a_model();
    let pair = a_wedge_lying_on_a_cones_side(&mut model);
    let left = fused(&mut model, &pair, Rule::Difference);
    assert_eq!(
        left.solids().len(),
        2,
        "the wedge's floor holds the frustum's whole disc at z = 8, so the frustum below it is cut off"
    );
    measures(
        volume_of(&model, left.solids()),
        frustum - shared,
        Side::Under,
    );

    for turned in [false, true] {
        let mut model = a_model();
        let pair = either_way_round(a_wedge_lying_on_a_cones_side(&mut model), turned);
        let united = fused(&mut model, &pair, Rule::Union);
        assert_eq!(united.solids().len(), 1, "turned {turned}");
        measures(
            volume_of(&model, united.solids()),
            frustum + 96.0 - shared,
            Side::Under,
        );

        let mut model = a_model();
        let pair = either_way_round(a_wedge_lying_on_a_cones_side(&mut model), turned);
        let united = model
            .fuse(
                &pair.arguments,
                &pair.tools,
                Rule::Union,
                &Policy::tolerant(100.0),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("a tolerant union, turned {turned}, refused: {why}"));
        assert_eq!(
            united.solids().len(),
            1,
            "a tolerant union, turned {turned}"
        );
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a fuse writes a sound model or writes nothing");
        measures(
            volume_of(&model, united.solids()),
            frustum + 96.0 - shared,
            Side::Under,
        );
    }

    for wedge_first in [false, true] {
        let mut model = a_model();
        let pair = the_wedge_less_the_frustum(&mut model, wedge_first);
        let left = fused(&mut model, &pair, Rule::Difference);
        assert_eq!(left.solids().len(), 1, "wedge first {wedge_first}");
        measures(volume_of(&model, left.solids()), 96.0 - shared, Side::Over);
        let uses = uses_of_the_ruling(&model, left.solids()[0]);
        assert!(
            matches!(
                uses.as_slice(),
                [(one, Surface::Cone(_)), (other, Surface::Cone(_))] if one == other
            ),
            "wedge first {wedge_first}: the ruling is one edge, the cone's seam, and no \
             edge of the slanted face: {uses:?}"
        );

        let mut model = a_model();
        let pair = the_wedge_less_the_frustum(&mut model, wedge_first);
        let left = model
            .fuse(
                &pair.arguments,
                &pair.tools,
                Rule::Difference,
                &Policy::tolerant(100.0),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| {
                panic!("a tolerant difference, wedge first {wedge_first}, refused: {why}")
            });
        assert_eq!(left.solids().len(), 1, "a tolerant difference");
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a fuse writes a sound model or writes nothing");
        measures(volume_of(&model, left.solids()), 96.0 - shared, Side::Over);
    }

    let mut model = a_model();
    let cone = cones::a_frustum(
        &mut model,
        A_CONES_SLOPE / 2.0,
        A_CONES_NEAR_END,
        A_CONES_FAR_END,
    );
    let wedge = a_wedge_resting_on_a_cones_side(&mut model);
    let united = fused(&mut model, &Pair::of(cone, wedge), Rule::Union);
    assert_eq!(united.solids().len(), 1);
    let halved = A_CONES_SLOPE / 2.0;
    let frustum = PI * halved * halved * (A_CONES_FAR_END.powi(3) - A_CONES_NEAR_END.powi(3)) / 3.0;
    measures(
        volume_of(&model, united.solids()),
        frustum + 96.0 - what_the_wedge_shares_with_a_cone_of_slope(halved, THE_WEDGES_BACK),
        Side::Under,
    );
}

/// The wedge and the frustum it lies on, with the wedge built into the model
/// first or second, as the wedge less the frustum.
fn the_wedge_less_the_frustum(model: &mut Model, wedge_first: bool) -> Pair {
    if wedge_first {
        let wedge = a_wedge_resting_on_a_cones_side(model);
        let cone = cones::a_frustum(model, A_CONES_SLOPE, A_CONES_NEAR_END, A_CONES_FAR_END);
        return Pair::of(wedge, cone);
    }
    either_way_round(a_wedge_lying_on_a_cones_side(model), true)
}

/// Every use of an edge of `solid` that runs between the two ends of the ruling
/// the wedge rests on, `(1, 0, 8)` and `(2.5, 0, 20)`: the edge, and the surface
/// of the face that uses it.
fn uses_of_the_ruling(model: &Model, solid: SolidKey) -> Vec<(EdgeKey, Surface)> {
    let at = |point: Point, [x, y, z]: [f64; 3]| point.distance_to(Point::new(x, y, z)) < 1e-9;
    let mut uses = Vec::new();
    for face in model.solid(solid).expect("this model made the key").faces() {
        for coedge in face.wires().flat_map(WireView::coedges) {
            let edge = coedge.edge();
            let ends = [edge.start().point(), edge.end().point()];
            if ends.iter().any(|&end| at(end, [1.0, 0.0, 8.0]))
                && ends.iter().any(|&end| at(end, [2.5, 0.0, 20.0]))
            {
                uses.push((edge.key(), face.surface()));
            }
        }
    }
    uses
}

/// The pair as built, or with its two solids swapped.
fn either_way_round(pair: Pair, turned: bool) -> Pair {
    if turned {
        Pair::of(pair.tools[0], pair.arguments[0])
    } else {
        pair
    }
}

/// **With its back corner at `(−5, 0, 8)` the same wedge is refused on the
/// order round one ellipse**, and what stops it is where that ellipse starts,
/// not the touch.
///
/// [`A_BACK_WHOSE_ELLIPSE_STARTS_ON_THE_RULING`] has the geometry. Every
/// certified fuse, either way round, gives up on *order along a crossing* with
/// the node at `(2.5, 0, 20)` in its record, and the ellipse's start is inside
/// that node's box: sweeping round from a point in the box cannot say where in
/// the box the node is. A tolerant policy settles that sweep its own way and
/// then cannot tell the cells of a face apart.
///
/// **A gap and not a limit** (§16.10). The order round the ellipse exists, and
/// the node is a box only because a meet of two planes and a cone at a double
/// root is not one point in `f64`; the wedge one unit along `y` at its back,
/// [`a_wedge_lying_on_a_cones_side_shares_and_leaves_what_is_worked_out_by_hand`],
/// has its ellipses start elsewhere and answers.
#[test]
fn a_wedge_whose_ellipse_starts_where_its_edge_touches_it_is_refused_on_the_order_round_it() {
    let flat = Surface::Plane(
        Plane::from_point_and_normal(Point::new(-5.0, 0.0, 8.0), Vector::new(-8.0, -16.0, 5.0))
            .expect("a normal with length"),
    );
    let cone = Surface::Cone(a_cone_of_slope(A_CONES_SLOPE));
    let SurfaceMeet::Curves(runs) = intersect(
        &flat,
        &cone,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a plane across a cone is named") else {
        panic!("a plane steeper than the cone's side cuts it in a curve");
    };
    let start = where_a_closed_crossing_starts(&runs[0], [&flat, &cone], &mut ProvenanceLog::new())
        .expect("an ellipse has a start")
        .coordinates();
    let holds = |node: [[f64; 2]; 3], point: [f64; 3]| {
        (0..3).all(|axis| node[axis][0] <= point[axis] && point[axis] <= node[axis][1])
    };

    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for turned in [false, true] {
                let mut model = a_model();
                let cone =
                    cones::a_frustum(&mut model, A_CONES_SLOPE, A_CONES_NEAR_END, A_CONES_FAR_END);
                let wedge = a_wedge_resting_on_a_cones_side_with_its_back_at(
                    &mut model,
                    A_BACK_WHOSE_ELLIPSE_STARTS_ON_THE_RULING,
                );
                let pair = either_way_round(Pair::of(cone, wedge), turned);
                let why = refusal(&mut model, &pair, rule, &policy);
                if policy.max_wobble() > 0.0 {
                    assert!(
                        matches!(why, Error::CannotTellTheCellsOfAFaceApart { .. }),
                        "{rule:?}, turned {turned}: {why:?}"
                    );
                    continue;
                }
                assert_eq!(what_it_gave_up_on(&why), "order along a crossing");
                let nodes = the_points_it_could_not_order(&why);
                assert!(
                    nodes
                        .iter()
                        .any(|&node| holds(node, start) && holds(node, [2.5, 0.0, 20.0])),
                    "{rule:?}, turned {turned}: one node is the touch and holds the start, {nodes:?}"
                );
            }
        }
    }
}

/// How much space the wedge shares with the cone of `slope` about `+z`, worked
/// out without the kernel: the area of each level's disc of radius `slope · z`
/// that lies inside the wedge's triangle there, added up from the wedge's floor
/// at `z = 8` to its tip at `z = 24`.
///
/// The disc is a polygon of [`SIDES`] corners on its rim, which is short of the
/// disc by a share of `(2π/SIDES)²/6`, under a millionth; the levels are added
/// by Simpson's rule. Neither comes near what a mesh at [`A_DEFLECTION`] is
/// allowed.
fn what_the_wedge_shares_with_a_cone_of_slope(slope: f64, back: Back) -> f64 {
    const SIDES: u32 = 4096;
    const LEVELS: u32 = 4000;
    // Each slanted face as `n` and a point on it, with `n` pointing out of the
    // wedge, as `a_wedge_resting_on_a_cones_side_with_its_back_at` writes them.
    let faces = [
        ([1.0, 0.0, -A_CONES_SLOPE], [0.0, 0.0, 0.0]),
        (back.one_side, [1.0, 3.0, 8.0]),
        (back.other_side, back.at),
    ];
    let area_at = |z: f64| {
        let radius = slope * z;
        let mut ring: Vec<[f64; 2]> = (0..SIDES)
            .map(|corner| {
                let turned = 2.0 * PI * f64::from(corner) / f64::from(SIDES);
                [radius * turned.cos(), radius * turned.sin()]
            })
            .collect();
        for ([nx, ny, nz], [qx, qy, qz]) in faces {
            let most = nx * qx + ny * qy + nz * qz - nz * z;
            ring = inside_of_a_line(&ring, |[x, y]| nx * x + ny * y - most);
        }
        (0..ring.len())
            .map(|corner| {
                let [ax, ay] = ring[corner];
                let [bx, by] = ring[(corner + 1) % ring.len()];
                ax * by - bx * ay
            })
            .sum::<f64>()
            / 2.0
    };
    let (floor, tip) = (8.0, 24.0);
    let step = (tip - floor) / f64::from(LEVELS);
    (0..=LEVELS)
        .map(|level| {
            let weight = match level {
                0 => 1.0,
                _ if level == LEVELS => 1.0,
                _ if level % 2 == 1 => 4.0,
                _ => 2.0,
            };
            weight * area_at(floor + step * f64::from(level))
        })
        .sum::<f64>()
        * step
        / 3.0
}

/// The part of a polygon where `past` is not above zero: one pass of clipping
/// against a line.
fn inside_of_a_line(ring: &[[f64; 2]], past: impl Fn([f64; 2]) -> f64) -> Vec<[f64; 2]> {
    let mut kept = Vec::with_capacity(ring.len() + 1);
    for corner in 0..ring.len() {
        let (here, next) = (ring[corner], ring[(corner + 1) % ring.len()]);
        let (at_here, at_next) = (past(here), past(next));
        if at_here <= 0.0 {
            kept.push(here);
        }
        if (at_here <= 0.0) != (at_next <= 0.0) {
            let share = at_here / (at_here - at_next);
            kept.push([
                here[0] + share * (next[0] - here[0]),
                here[1] + share * (next[1] - here[1]),
            ]);
        }
    }
    kept
}

/// The two nodes an order along a crossing could not tell apart, read straight
/// out of the refusal's own record, each as its box.
fn the_points_it_could_not_order(why: &Error) -> Vec<[[f64; 2]; 3]> {
    let Error::Undecidable { records } = why else {
        panic!("expected `Undecidable`, got {why:?}");
    };
    let last = records
        .records()
        .last()
        .expect("a refusal names a decision");
    last.inputs()
        .iter()
        .map(|node| {
            let PredicatePoint::Constructed(box_) = node else {
                panic!("a node this refusal names is worked out, got {node:?}");
            };
            box_.map(|along| [along.lo(), along.hi()])
        })
        .collect()
}

// 3. Seam-crossing intersections — criterion 14's third entry.

/// A bar notched by a two-cube whose notch runs **across the bar's seam**: the
/// seam is at `(2, 0, z)` and the block covers `1 ≤ x ≤ 3`, `-1 ≤ y ≤ 1`.
fn a_notch_across_the_seam(model: &mut Model) -> Pair {
    let bar = a_bar(model);
    let notch = block(model, [2.0, 2.0, 2.0], Vector::new(1.0, -1.0, 3.0));
    Pair::of(bar, notch)
}

/// The very same notch on the far side of the bar, where it meets no seam.
///
/// Mirrored in `x`, and the bar is symmetric under `x ↦ -x` while its seam is
/// not. So the two answers are the same solid reflected and their volumes are
/// equal exactly, while their meshes are not the same triangles.
fn a_notch_away_from_the_seam(model: &mut Model) -> Pair {
    let bar = a_bar(model);
    let notch = block(model, [2.0, 2.0, 2.0], Vector::new(-3.0, -1.0, 3.0));
    Pair::of(bar, notch)
}

/// Crossing the seam costs the answer nothing at all.
///
/// The claim needs no bound and no closed form: the two pairs are exact mirror
/// images in the plane `x = 0`, the bar is its own mirror image and the notch is
/// the other one's, so the two answers are congruent solids and their volumes
/// are one number. The seam is the only thing the reflection does not carry over
/// — it stays at `x = +2` — so any difference between the two is the seam and
/// nothing else.
///
/// Measured, the two agree to seven parts in `10^17`, which is the reflection
/// reordering a sum of triangles and not the seam doing anything.
#[test]
fn crossing_a_bars_seam_costs_the_answer_nothing() {
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        let mut across = a_model();
        let pair = a_notch_across_the_seam(&mut across);
        let cut = fused(&mut across, &pair, rule);
        let on_the_seam = volume_of(&across, cut.solids());

        let mut away = a_model();
        let pair = a_notch_away_from_the_seam(&mut away);
        let cut = fused(&mut away, &pair, rule);
        let off_the_seam = volume_of(&away, cut.solids());

        let apart = (on_the_seam - off_the_seam).abs() / off_the_seam;
        assert!(
            apart < 1e-12,
            "{rule:?}: {on_the_seam} across the seam against {off_the_seam} away from it"
        );
    }

    let mut model = a_model();
    let pair = a_notch_across_the_seam(&mut model);
    let notched = fused(&mut model, &pair, Rule::Difference);
    assert_eq!(notched.solids().len(), 1, "a notched bar is one solid");
    assert!(
        volume_of(&model, notched.solids()) < A_BAR,
        "a notch takes material away"
    );
}

/// A ball of radius four cut by the plane `z = 2`, which meets its seam once.
///
/// A ball's seam is the half of the meridian circle `y = 0` with `x ≥ 0`,
/// running pole to pole. The crossing circle `z = 2` meets the plane `y = 0` at
/// `(±√12, 0, 2)` and only the positive one is on the seam — so this cut crosses
/// the seam exactly **once**, which is the case roadmap §7.4 is naming.
fn a_ball_cut_across_its_seam(model: &mut Model) -> Pair {
    let ball = model
        .add_ball(4.0, &Isometry::identity())
        .expect("a ball of radius four");
    let knife = block(model, [12.0, 12.0, 12.0], Vector::new(-6.0, -6.0, 2.0));
    Pair::of(ball, knife)
}

/// A cut that crosses a ball's seam once is answered, and the answer measures
/// what the closed form says.
///
/// The cap above `z = 2` on a ball of radius four is `πh²(3r − h)/3` with
/// `h = 2`, which is `40π/3`. The rest of the ball is the ball less that.
#[test]
fn a_ball_cut_by_a_plane_that_crosses_its_seam_once_is_answered() {
    let cap = PI * 4.0 * 10.0 / 3.0;

    let mut model = a_model();
    let pair = a_ball_cut_across_its_seam(&mut model);
    let above = fused(&mut model, &pair, Rule::Intersection);
    assert_eq!(above.solids().len(), 1);
    measures(volume_of(&model, above.solids()), cap, Side::Under);

    let mut model = a_model();
    let pair = a_ball_cut_across_its_seam(&mut model);
    let below = fused(&mut model, &pair, Rule::Difference);
    assert_eq!(below.solids().len(), 1);
    measures(
        volume_of(&model, below.solids()),
        A_BIG_BALL - cap,
        Side::Under,
    );
}

/// The same ball cut by `x = 2` instead is answered under every rule.
///
/// **How many times the cut meets the seam is not what decides it, and this row
/// used to say it was.** The circle `x = 2` meets the seam at `(2, 0, ±√12)`,
/// twice against `z = 2`'s once, and this refused where `z = 2` answered — but
/// the seam is not the difference. `z = 2` cuts the ball into two pieces each
/// holding one pole; `x = 2` cuts it into a cap holding **no** pole and a
/// remainder holding **both**. The cap is answered here, at `40π/3`, exactly as
/// `z = 2`'s is. The both-pole piece was refused because nothing wrote a cut
/// closed face with a seam and two poles of its own (§38.3). The mesher now
/// opens it along a seam half a turn from its rim (§202.1). The seam-crossing
/// count is a coincidence of these two knives.
#[test]
fn a_ball_cut_off_its_polar_axis_is_answered_under_every_rule() {
    let cap = PI * 4.0 * 10.0 / 3.0;

    let mut model = a_model();
    let ball = model
        .add_ball(4.0, &Isometry::identity())
        .expect("a ball of radius four");
    let knife = block(&mut model, [12.0, 12.0, 12.0], Vector::new(2.0, -6.0, -6.0));
    let pair = Pair::of(ball, knife);
    let above = fused(&mut model, &pair, Rule::Intersection);
    assert_eq!(above.solids().len(), 1, "the cap beyond x = 2 is one solid");
    measures(volume_of(&model, above.solids()), cap, Side::Under);

    let the_knife = 12.0 * 12.0 * 12.0;
    for (rule, closed_form) in [
        (Rule::Union, A_BIG_BALL - cap + the_knife),
        (Rule::Difference, A_BIG_BALL - cap),
    ] {
        let mut model = a_model();
        let ball = model
            .add_ball(4.0, &Isometry::identity())
            .expect("a ball of radius four");
        let knife = block(&mut model, [12.0, 12.0, 12.0], Vector::new(2.0, -6.0, -6.0));
        let pair = Pair::of(ball, knife);
        let answer = fused(&mut model, &pair, rule);
        assert_eq!(
            answer.solids().len(),
            1,
            "{rule:?} of a ball and a knife off its axis is one solid"
        );
        measures(volume_of(&model, answer.solids()), closed_form, Side::Under);
    }
}

/// A ball cut by a plane down its polar axis is answered under all three rules,
/// by either of the two planes that hold the axis.
///
/// §37.4 called `x = 0` "the plane the seam lies in". It is not — the seam lies
/// in `y = 0` — and what the two planes have in common is the thing that
/// matters: **both contain the ball's polar axis**, so the great circle each of
/// them cuts runs through both poles, and the one corner the arrangement puts on
/// that circle is a pole. A closed edge starting at a pole has no direction, so
/// this was a panic (§37.4), then the refusal
/// `AClosedEdgeOfTheAnswerWouldStartAtAPole` (§38.3).
///
/// §194.4 cuts such a circle a second time straight across from its start,
/// which here is the other pole, so each half of the circle is a meridian from
/// pole to pole and each half of the ball's skin is a lune half a turn wide,
/// both of whose corners are poles. The mesher's walk round that wire set off
/// from one pole and never arrived back at it, so it ended half a turn from
/// where it began and was refused `AWireWillNotLieFlatOnItsFace` (§194.5).
/// Every volume below is the closed form's, read through an inscribed mesh.
#[test]
fn a_ball_cut_by_a_plane_down_its_polar_axis_is_answered() {
    let half_a_ball = A_BIG_BALL / 2.0;
    let the_knife = 12.0 * 12.0 * 12.0;
    for at in [Vector::new(0.0, -6.0, -6.0), Vector::new(-6.0, 0.0, -6.0)] {
        for (rule, closed_form) in [
            (Rule::Union, the_knife + half_a_ball),
            (Rule::Intersection, half_a_ball),
            (Rule::Difference, half_a_ball),
        ] {
            let mut model = a_model();
            let ball = model
                .add_ball(4.0, &Isometry::identity())
                .expect("a ball of radius four");
            let knife = block(&mut model, [12.0, 12.0, 12.0], at);
            let answer = fused(&mut model, &Pair::of(ball, knife), rule);
            assert_eq!(
                answer.solids().len(),
                1,
                "the {rule:?} of a ball and a knife down its axis at {at:?} is one solid"
            );
            measures(volume_of(&model, answer.solids()), closed_form, Side::Under);
        }
    }
}

/// The controls for the entry above, the same ball cut through the same centre
/// by planes that do **not** hold its polar axis.
///
/// `z = 0` is answered: the great circle is the equator, which runs through no
/// pole, so each half is a cap with a pole inside it and the mesher cuts it up
/// to that pole. A plane through the centre leaning by 0.4 of a radian is
/// answered for the same reason. These went through the fuse by a different
/// road from the polar-axis cut when that one was refused, and they still do:
/// no circle here starts at a pole, so none is cut in two.
#[test]
fn a_ball_cut_through_its_middle_by_a_plane_off_its_polar_axis_is_answered() {
    let half_a_ball = 2.0 / 3.0 * PI * 64.0;

    let mut model = a_model();
    let ball = model
        .add_ball(4.0, &Isometry::identity())
        .expect("a ball of radius four");
    let knife = block(&mut model, [12.0, 12.0, 12.0], Vector::new(-6.0, -6.0, 0.0));
    let above = fused(&mut model, &Pair::of(ball, knife), Rule::Intersection);
    assert_eq!(above.solids().len(), 1);
    measures(volume_of(&model, above.solids()), half_a_ball, Side::Under);

    let mut model = a_model();
    let ball = model
        .add_ball(4.0, &Isometry::identity())
        .expect("a ball of radius four");
    let knife = block(&mut model, [12.0, 12.0, 12.0], Vector::new(-6.0, -6.0, 0.0));
    let axis = Axis::new(Point::origin(), Vector::new(1.0, 0.0, 0.0)).expect("an axis");
    model
        .place_solid(knife, &Isometry::rotation(&axis, 0.4).expect("a lean"))
        .expect("this model made the key");
    let leaning = fused(&mut model, &Pair::of(ball, knife), Rule::Intersection);
    assert_eq!(leaning.solids().len(), 1);
    measures(
        volume_of(&model, leaning.solids()),
        half_a_ball,
        Side::Under,
    );
}

// 4. Closed periodic surfaces — criterion 14's fourth entry.

/// A block of side eight with a bar of radius two driven right through it.
///
/// What makes this the entry rather than any other curved pair is what is left
/// of the bar's wall: a tube, closed right round the cylinder, whose two edges
/// are **whole circles** and not arcs of anything. A cylinder is periodic in one
/// coordinate and this is the answer that keeps a whole period of it.
fn a_block_with_a_round_hole(model: &mut Model) -> Pair {
    let solid = block(model, [8.0, 8.0, 8.0], Vector::zero());
    let bar = model
        .add_round_bar(
            2.0,
            12.0,
            &Isometry::translation(Vector::new(4.0, 4.0, -2.0)).expect("a whole-number shift"),
        )
        .expect("a bar long enough to go right through");
    Pair::of(solid, bar)
}

/// The hole's two edges go right round the closed surface they lie on, and the
/// answer measures the block less the bar.
///
/// It measures **over** the closed form on purpose: the wall of the hole is
/// meshed inside the true cylinder, so the hole the mesh describes is smaller
/// than the hole that is there and what is left round it is bigger. That is the
/// one place in this file where the sign of the error is the other way up, and
/// asserting it is what stops the band being read as slop.
#[test]
fn a_block_with_a_round_hole_through_it_is_bounded_by_whole_circles() {
    let mut model = a_model();
    let pair = a_block_with_a_round_hole(&mut model);

    let drilled = fused(&mut model, &pair, Rule::Difference);
    assert_eq!(drilled.solids().len(), 1);
    measures(
        volume_of(&model, drilled.solids()),
        512.0 - A_BAR,
        Side::Over,
    );

    let closed = model
        .solid(drilled.solids()[0])
        .expect("this model made the key")
        .faces()
        .flat_map(|face| face.wires())
        .flat_map(|wire| wire.coedges())
        .filter(|coedge| {
            coedge
                .edge()
                .is_closed()
                .expect("a drilled block's edges are runs this kernel named")
        })
        .count();
    assert!(
        closed > 0,
        "the wall of the hole is bounded by circles that close on themselves"
    );
}

// 5. Coincident curved faces — §7.4's first entry, asked of a curved surface.

/// One bar, built twice, in the very same place. Every coordinate is shared,
/// so the two walls are the same cylinder bit for bit.
fn a_bar_and_a_copy_of_it(model: &mut Model) -> Pair {
    let one = a_bar(model);
    let other = a_bar(model);
    Pair::of(one, other)
}

/// Two coincident bars answer: the union and the common part are the bar, and
/// the difference is nothing.
///
/// **This test was called `..._are_refused_though_a_point_inside_was_found`
/// until §224.4, and `..._for_want_of_a_point_inside_a_piece` until §217.**
/// §37's and §39's tables cite it under those names and under the refusals
/// *a point inside a piece of a face* and *a point in a piece that every solid
/// could place*; they record what was measured when they were written, and this
/// is the forwarding address. The refusal was about the flat caps, not the
/// wall: a cap is a disk with one corner, and `select.rs` asked whether the
/// copy's cap held a point by winding that one corner, which winds round
/// nothing. A flat face with a curved side is now asked through the solid's own
/// point test, and the gap §16.10 named is closed.
#[test]
fn a_bar_and_a_copy_of_it_in_the_same_place_answer_with_the_bar() {
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        let mut model = a_model();
        let pair = a_bar_and_a_copy_of_it(&mut model);
        let answer = fused(&mut model, &pair, rule);
        if rule == Rule::Difference {
            assert!(answer.solids().is_empty(), "a bar less itself is nothing");
        } else {
            assert_eq!(answer.solids().len(), 1, "{rule:?} of a bar and its copy");
            measures(volume_of(&model, answer.solids()), A_BAR, Side::Under);
        }
    }
}

// 6. Vertex on a curved face — §7.4's third entry, asked of a curved surface.

/// A block of `3 × 4` standing in a bar of radius five, so that its corner at
/// `(3, 4, z)` is exactly on the bar's wall.
///
/// `3² + 4² = 5²` in whole numbers, so `x² + y² − r²` at that corner cancels to
/// exactly zero. This is the only way to put a corner *provably* on a round wall
/// in `f64`, and it is why the entry is built from a Pythagorean triple rather
/// than by nudging.
fn a_block_corner_on_a_bars_wall(model: &mut Model, depth: f64) -> Pair {
    let bar = model
        .add_round_bar(5.0, 8.0, &Isometry::identity())
        .expect("a bar of radius five");
    let standing = block(model, [3.0, depth, 2.0], Vector::new(0.0, 0.0, 2.0));
    Pair::of(bar, standing)
}

/// A corner exactly on a bar's wall is answered: the union is the bar and the
/// intersection the block.
///
/// Two arcs leave that corner with no turn between them, and they are not merely
/// tangent — they are the **same line**. The plane `x = 3` cuts the bar of radius
/// five in the two rulings `(3, ±4, z)` and the plane `y = 4` cuts it in
/// `(±3, 4, z)`, so `(3, 4, z)` is on both, and the block touches the bar's wall
/// along that one ruling without cutting it anywhere.
///
/// The arrangement holds one crossing per curve, and here it had two for one,
/// so there was no anticlockwise order round the corner: §38.2 turned the
/// panic §37.4 pinned here into [`Error::TwoCurvesTouchWithoutCrossing`], and
/// every policy refused with it. Since queue row Q930 `arrange.rs` folds a
/// straight cut into a straight crossing drawn before it when the two leave a
/// node along one line, which is §38.2's family of fixes for straight lines.
///
/// A `3 × 4` rectangle measured from the axis is inside a circle of radius
/// five, so the block is wholly inside the bar, touching its wall along the
/// ruling: the union is the bar, the intersection is the block,
/// `3 · 4 · 2 = 24`, and the bar less the block is the bar with the block's
/// room left empty inside it, one solid of `200π − 24` whose hollow's edge
/// `(3, 4, z)` lies on the bar's wall. A tolerant policy gives the same: the
/// fold is proved and spends no budget.
#[test]
fn a_block_corner_exactly_on_a_bars_wall_is_answered() {
    let bar = PI * 25.0 * 8.0;
    for (rule, closed_form) in [
        (Rule::Union, bar),
        (Rule::Intersection, 24.0),
        (Rule::Difference, bar - 24.0),
    ] {
        let mut model = a_model();
        let pair = a_block_corner_on_a_bars_wall(&mut model, 4.0);
        let answer = fused(&mut model, &pair, rule);
        assert_eq!(answer.solids().len(), 1, "{rule:?}");
        let found = volume_of(&model, answer.solids());
        if rule == Rule::Intersection {
            assert!(
                (found - closed_form).abs() < 1e-12,
                "a block is flat and meshes exactly, got {found}"
            );
        } else {
            measures(found, closed_form, Side::Under);
        }

        let mut model = a_model();
        let pair = a_block_corner_on_a_bars_wall(&mut model, 4.0);
        let tolerant = model
            .fuse(
                &pair.arguments,
                &pair.tools,
                rule,
                &Policy::tolerant(100.0),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("a tolerant {rule:?} refused: {why}"));
        assert_eq!(tolerant.solids().len(), 1, "a tolerant {rule:?}");
    }
}

/// The controls, a tenth of a millimetre either side, and they are what make the
/// panic above a statement about the incidence rather than about the shape.
///
/// At `3.9` the corner is inside the bar and at `4.1` it is outside, and the
/// union is answered both times with a volume the closed form agrees with: the
/// bar of radius five and height eight, plus whatever of the block sticks out of
/// it, which is nothing, because a `3 × 4` rectangle from the axis is inside a
/// circle of radius five at both depths.
#[test]
fn a_block_corner_a_tenth_off_a_bars_wall_is_answered_either_side() {
    for depth in [3.9, 4.1] {
        let mut model = a_model();
        let pair = a_block_corner_on_a_bars_wall(&mut model, depth);
        let united = fused(&mut model, &pair, Rule::Union);
        assert_eq!(united.solids().len(), 1);
        measures(
            volume_of(&model, united.solids()),
            PI * 25.0 * 8.0,
            Side::Under,
        );
    }
}

/// The same corner on a *ball*, and it is answered.
///
/// `(3, 4, 0)` is on a ball of radius five, and the block's two walls `x = 3`
/// and `y = 4` cut the skin in circles that touch there — both tangent along
/// `±z`, the `x = 3` circle of radius four curving toward `(3, 0, 0)` and the
/// `y = 4` circle of radius three toward `(0, 4, 0)`. Worked out by hand about
/// the outward normal `(3, 4, 0)/5`, the six ways out of that node run `y = 4`
/// up, `x = 3` up, `z = 0` one way, `x = 3` down, `y = 4` down, `z = 0` the
/// other, and the arrangement hands back exactly that order.
///
/// **The mesher used to refuse the answer, and that refusal was a defect.** The
/// block's walls `x = 0` and `y = 0` meet the ball at `(0, 0, -5)`, which is a
/// pole of the sphere: a corner with no longitude of its own, and a whole *side*
/// of the patch in the sphere's two coordinates. Nothing in a wire says so
/// unless a whole closed surface was cut open, so the walk read `atan2(0, 0)`
/// there; one of the two faces round that pole came back 5.64 radians from the
/// 1.571 it set off at and was refused `AWireWillNotLieFlatOnItsFace`, and the
/// other was not refused and closed straight across the chart instead of along
/// the pole, which is worse.
///
/// # What the volume is checked against
///
/// A ball of radius five and a `3 × 4 × 4` block from the origin, so the union
/// is `500π/3 + 48 − V`, where `V` is how much of the block is inside the ball.
/// The block's base rectangle is exactly inscribed in the ball's equator, `3`
/// and `4` and `5` being what they are, and that is what makes `V` come out in
/// whole multiples of `π`.
///
/// Over the quarter disc of radius three the ball is at least four high, so that
/// much of `V` is `4 · 9π/4 = 9π`. Over the whole rectangle the height
/// `√(25 − r²)` integrates in polar coordinates to `125π/6 − (A(3) + A(4))/3`,
/// and taking off the quarter disc's own `61π/6` leaves the rest. Each
/// `A(c) = ∫₀^{arccos(c/5)} (25cos²θ − c²)^{3/2} sec³θ dθ` turns into
/// `k⁴c ∫₀^{π/2} cos⁴u / (c² + k² sin²u) du` with `k² = 25 − c²`, under
/// `t = tan θ` and then `t = (k/c) sin u`; `∫₀^{π/2} du / (p + q sin²u)` is
/// `π / (2√(p(p + q)))`, and `p + q` is `25` both times, so both roots are whole
/// numbers. They come to `A(3) = 13π` and `A(4) = 7π/2`, which leaves
/// `V = 85π/6` and a union of `305π/2 + 48`.
///
/// A grid sum of `min(4, √(25 − x² − y²))` over the base rectangle agrees with
/// `85π/6` to eight figures.
#[test]
fn a_block_corner_exactly_on_a_balls_skin_is_answered() {
    let mut model = a_model();
    let ball = model
        .add_ball(5.0, &Isometry::identity())
        .expect("a ball of radius five");
    let standing = block(&mut model, [3.0, 4.0, 4.0], Vector::zero());
    let united = fused(&mut model, &Pair::of(ball, standing), Rule::Union);
    assert_eq!(united.solids().len(), 1);
    measures(
        volume_of(&model, united.solids()),
        305.0 / 2.0 * PI + 48.0,
        Side::Under,
    );
}

/// The ball's controls, refusing by name a tenth either side.
#[test]
fn a_block_corner_a_tenth_off_a_balls_skin_is_refused_by_name_either_side() {
    for depth in [3.9, 4.1] {
        let mut model = a_model();
        let ball = model
            .add_ball(5.0, &Isometry::identity())
            .expect("a ball of radius five");
        let standing = block(&mut model, [3.0, depth, 4.0], Vector::zero());
        let why = refusal(
            &mut model,
            &Pair::of(ball, standing),
            Rule::Union,
            &Policy::certified(),
        );
        assert_eq!(what_it_gave_up_on(&why), "order along a crossing");
    }
}

// 7. Zero-length edges — §7.4's fifth entry, asked of a curved surface.

/// Two blocks cutting one bar, sharing the plane `x = 0` between them and both
/// carrying the planes `z = 3` and `z = 5`.
///
/// A splitter that cut the bar's wall by each plane in turn would put two
/// corners at the same point on the two rulings `(0, ±2, z)` and join them with
/// an edge of no length. The planar catalogue's entry asserts the merge happened
/// instead; this one records that the curved case does not get that far.
fn a_bar_and_two_blocks_on_the_same_planes(model: &mut Model) -> Pair {
    let bar = a_bar(model);
    let left = block(model, [4.0, 8.0, 2.0], Vector::new(-4.0, -4.0, 3.0));
    let right = block(model, [4.0, 8.0, 2.0], Vector::new(0.0, -4.0, 3.0));
    Pair {
        arguments: vec![bar],
        tools: vec![left, right],
    }
}

/// Two tools meeting on a plane that cuts a round wall are answered, and they
/// give exactly what one tool doing the same cut gives.
///
/// **This was the gap §16.10 asked to be classified, and it is closed.** The
/// refusal was `Undecidable` over *a point inside a piece of a face*, and its
/// own record named `(0, -2, 0)` and `(0, 2, 0)` — the two points where the
/// shared plane `x = 0` touches the cylinder of radius two, so the entry was
/// reaching its subject and not stopping short of it. What could not be placed
/// was a half of a cap disc: two corners on the rim, an arc between them, and no
/// polygon for the winding count to find a point in. `select.rs`'s
/// `in_the_bulge_of` now counts a pipe's arc the way it counts a ball's, and the
/// two-tool answer is the single-tool answer, which is what §16.10 said the
/// answer would be.
#[test]
fn two_tools_meeting_on_a_plane_that_cuts_a_round_wall_are_answered() {
    let mut model = a_model();
    let pair = a_bar_and_two_blocks_on_the_same_planes(&mut model);
    let cut = fused(&mut model, &pair, Rule::Difference);
    assert_eq!(
        cut.solids().len(),
        2,
        "two tools doing one cut leave the bar in two pieces"
    );
    measures(volume_of(&model, cut.solids()), PI * 4.0 * 6.0, Side::Under);

    let mut model = a_model();
    let bar = a_bar(&mut model);
    let one = block(&mut model, [8.0, 8.0, 2.0], Vector::new(-4.0, -4.0, 3.0));
    let cut = fused(&mut model, &Pair::of(bar, one), Rule::Difference);
    assert_eq!(
        cut.solids().len(),
        2,
        "one tool doing the same cut leaves the bar in two pieces"
    );
    measures(volume_of(&model, cut.solids()), PI * 4.0 * 6.0, Side::Under);
}

// 8. Sliver faces — §7.4's sixth entry, asked of a curved surface.

/// How thick the slab a sliver cut leaves is. A power of two, so the thin
/// coordinate is exactly representable and this is a sliver rather than a
/// rounding — the planar catalogue's own constant and its own reason.
const A_SLIVER: f64 = 1.0 / (1 << 20) as f64;

/// A bar cut by a slab a millionth of a millimetre thick, taken out of the
/// middle of it so that neither of the bar's own caps is involved.
fn a_bar_and_a_knife_a_sliver_thick(model: &mut Model) -> Pair {
    let bar = a_bar(model);
    let knife = block(model, [8.0, 8.0, A_SLIVER], Vector::new(-4.0, -4.0, 4.0));
    Pair::of(bar, knife)
}

/// The sliver is **kept**, and it is a disc of positive volume rather than
/// nothing.
///
/// §12.3 is what this is here to stop happening again, one surface up: a
/// certifiably non-degenerate sliver refused twice in M0. What the curved case
/// adds is that the sliver's own wall is a piece of the cylinder a millionth
/// tall, which is the narrowest curved face anything in this tree builds.
#[test]
fn a_cut_a_millionth_thick_across_a_bar_keeps_the_disc_it_leaves() {
    let mut model = a_model();
    let pair = a_bar_and_a_knife_a_sliver_thick(&mut model);

    let disc = fused(&mut model, &pair, Rule::Intersection);
    assert_eq!(disc.solids().len(), 1);
    assert_eq!(
        model
            .solid(disc.solids()[0])
            .expect("this model made the key")
            .faces()
            .count(),
        3
    );
    measures(
        volume_of(&model, disc.solids()),
        PI * 4.0 * A_SLIVER,
        Side::Under,
    );

    let mut model = a_model();
    let pair = a_bar_and_a_knife_a_sliver_thick(&mut model);
    let split = fused(&mut model, &pair, Rule::Difference);
    assert_eq!(split.solids().len(), 2, "the sliver cuts the bar in two");
}

// 9. Fully contained solids — §7.4's seventh entry, asked of a curved surface.

/// A bar of radius one, four tall, in the middle of a block of side eight.
fn a_bar_wholly_inside_a_block(model: &mut Model) -> Pair {
    let solid = block(model, [8.0, 8.0, 8.0], Vector::zero());
    let bar = model
        .add_round_bar(
            1.0,
            4.0,
            &Isometry::translation(Vector::new(4.0, 4.0, 2.0)).expect("a whole-number shift"),
        )
        .expect("a small bar");
    Pair::of(solid, bar)
}

/// Two of the three rules are right: the union swallows the bar and the
/// intersection is the bar.
#[test]
fn a_bar_wholly_inside_a_block_unites_and_intersects() {
    let mut model = a_model();
    let pair = a_bar_wholly_inside_a_block(&mut model);
    let united = fused(&mut model, &pair, Rule::Union);
    assert_eq!(united.solids().len(), 1);
    assert_eq!(
        model
            .solid(united.solids()[0])
            .expect("this model made the key")
            .faces()
            .count(),
        6
    );
    assert_eq!(
        volume_of(&model, united.solids()),
        512.0,
        "the block, whole, and a polyhedron measures exactly"
    );

    let mut model = a_model();
    let pair = a_bar_wholly_inside_a_block(&mut model);
    let shared = fused(&mut model, &pair, Rule::Intersection);
    assert_eq!(shared.solids().len(), 1);
    measures(volume_of(&model, shared.solids()), PI * 4.0, Side::Under);
}

/// The third rule leaves one solid with a bubble in it, which is §38.4's fix of
/// the panic §37.4 pinned here.
///
/// The planar catalogue's `a_cube_wholly_inside_another_under_all_three_rules`
/// is the same answer with boxes: an outer shell and a void inside it, one
/// solid. The volume is the block less the bar, and it measures **over** the
/// closed form because the bubble's wall is meshed inside the true cylinder, so
/// the bubble the mesh describes is smaller than the one that is there.
///
/// **Both ways of getting a closed edge's direction wrong are caught before a
/// volume is read**, which is worth saying because the volume is the obvious
/// thing to lean on. Its wall is a closed edge walked by two faces and a closed
/// edge's two ends are one corner, so nothing about the corner says which way
/// either face walks it; get it backwards and the arena's own rule that two
/// coedges run opposite ways trips. Measured: putting the corner test back
/// leaves every other test in the crate green and panics here and on the blind
/// hole below. What the closed form adds is that this is the **right** solid and
/// not merely a valid one.
#[test]
fn hollowing_a_block_out_with_a_bar_inside_it_leaves_a_bubble() {
    let mut model = a_model();
    let pair = a_bar_wholly_inside_a_block(&mut model);
    let hollowed = fused(&mut model, &pair, Rule::Difference);

    assert_eq!(hollowed.solids().len(), 1, "one solid with a void in it");
    let solid = model
        .solid(hollowed.solids()[0])
        .expect("this model made the key");
    assert_eq!(
        solid.shells().count(),
        2,
        "the block's own skin, and the bubble the bar left"
    );
    assert_eq!(
        solid.faces().count(),
        9,
        "the block's six, the bubble's wall, and its two caps"
    );
    measures(
        volume_of(&model, hollowed.solids()),
        512.0 - PI * 4.0,
        Side::Over,
    );
}

/// §36.15's blind hole, committed and not only arranged — the second shape
/// §37.4's third defect reached.
///
/// A bar of radius one standing on the block's top with `2 ≤ z ≤ 5` of it inside
/// the block, so its bottom cap is buried and its top cap is in the air. The
/// difference is the block with a hole three deep in its top, and the hole's
/// floor is that buried cap.
///
/// The hole has one cap inside the material and one outside, which is what makes
/// it the shape §36.15 named and the one a user reaches first. It is not a shape
/// that tells a winding error from a right one — the arena catches that before a
/// volume is read, and the bubble above records the measurement — but it is the
/// shape that says the answer is a hole three deep and not a hole all the way
/// through, which is `512 − 3π` against `512 − 8π`.
#[test]
fn a_blind_hole_is_committed_and_not_only_arranged() {
    let mut model = a_model();
    let solid = model
        .add_cube(8.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a block eight across");
    let bar = model
        .add_round_bar(
            1.0,
            5.0,
            &Isometry::translation(Vector::new(4.0, 4.0, 5.0)).expect("a shift"),
        )
        .expect("a bar standing on the block's top");
    let drilled = fused(&mut model, &Pair::of(solid, bar), Rule::Difference);

    assert_eq!(drilled.solids().len(), 1);
    let hollowed = model
        .solid(drilled.solids()[0])
        .expect("this model made the key");
    assert_eq!(hollowed.shells().count(), 1, "a blind hole is one shell");
    assert_eq!(
        hollowed.faces().count(),
        8,
        "the block's six, the hole's wall, and the cap it stops on"
    );
    measures(
        volume_of(&model, drilled.solids()),
        512.0 - PI * 3.0,
        Side::Over,
    );
}

/// A ball wholly inside a block answers all three rules, which is §38.1's fix of
/// the two wrong answers §37.4 pinned here.
///
/// The union was right by luck — the block swallows the ball either way. The
/// intersection is the ball and used to come back empty; the difference is a
/// block with a bubble in it and used to come back as the whole block. Both were
/// [`a_ball_that_nothing_cuts_comes_back_whole`] again.
///
/// **This is the row where §38.1's fix does not cancel**, and a ball fused on
/// its own is the row where it does. Fusing a ball alone never asks selection to
/// place a point inside the whole-surface piece — with no other solid there is
/// nothing to place it against — and never turns the piece round. Both happen
/// here, and the difference is where they show: a void shell has to measure
/// negative for the block to lose its volume rather than gain it. Measured, with
/// the whole-surface piece left unturned, this row is the **only** one in the
/// crate that goes red, and it goes red on two solids where there should be
/// one.
#[test]
fn a_ball_wholly_inside_a_block_answers_all_three_rules() {
    let ball_of_two = 4.0 / 3.0 * PI * 8.0;
    let build = |model: &mut Model| {
        let solid = block(model, [8.0, 8.0, 8.0], Vector::zero());
        let ball = model
            .add_ball(
                2.0,
                &Isometry::translation(Vector::new(4.0, 4.0, 4.0)).expect("a shift"),
            )
            .expect("a ball of radius two");
        Pair::of(solid, ball)
    };

    let mut model = a_model();
    let pair = build(&mut model);
    let united = fused(&mut model, &pair, Rule::Union);
    assert_eq!(
        volume_of(&model, united.solids()),
        512.0,
        "the block, whole, and a polyhedron measures exactly"
    );

    let mut model = a_model();
    let pair = build(&mut model);
    let shared = fused(&mut model, &pair, Rule::Intersection);
    assert_eq!(shared.solids().len(), 1);
    measures(volume_of(&model, shared.solids()), ball_of_two, Side::Under);

    let mut model = a_model();
    let pair = build(&mut model);
    let hollowed = fused(&mut model, &pair, Rule::Difference);
    assert_eq!(hollowed.solids().len(), 1);
    assert_eq!(
        model
            .solid(hollowed.solids()[0])
            .expect("this model made the key")
            .shells()
            .count(),
        2,
        "the block's own skin and the bubble inside it"
    );
    // The bubble's wall is meshed inside the true ball, so the bubble the mesh
    // describes is smaller than the one that is there and what is left round it
    // measures more.
    measures(
        volume_of(&model, hollowed.solids()),
        512.0 - ball_of_two,
        Side::Over,
    );
}

// 10. Touching but not overlapping — §7.4's eighth entry, asked of a curved
// surface.

/// A ball resting on a block's top, touching it at one point, unites into two
/// solids that touch — §38.1's fix of the third wrong answer §37.4 pinned here.
///
/// The planar catalogue's `two_cubes_that_only_touch_at_a_corner` gives the same
/// shape of answer. The ball's centre is at `z = 10` and the block's top is
/// `z = 8`, both whole numbers, so the touching point `(4, 4, 8)` is exact — and
/// it is also the ball's own south pole, so the answer's seam ends exactly on
/// the block's top face. The ball used to be dropped here.
#[test]
fn a_ball_resting_on_a_block_at_one_point_keeps_both() {
    let mut model = a_model();
    let solid = block(&mut model, [8.0, 8.0, 8.0], Vector::zero());
    let ball = model
        .add_ball(
            2.0,
            &Isometry::translation(Vector::new(4.0, 4.0, 10.0)).expect("a shift"),
        )
        .expect("a ball of radius two");

    let united = fused(&mut model, &Pair::of(solid, ball), Rule::Union);
    assert_eq!(united.solids().len(), 2, "two solids that touch at a point");
    measures(
        volume_of(&model, united.solids()),
        512.0 + 4.0 / 3.0 * PI * 8.0,
        Side::Under,
    );
}

/// A ball touching a wall at the one point every other candidate is ruled out
/// at, united with it, keeps both.
///
/// **This is the row that says a whole closed surface is trimmed by nothing.**
/// A piece is kept by classifying one point provably inside it, and a whole
/// sphere has exactly one corner pair — its two poles — so the candidates are
/// three blends of those, all of them the centre, and six points spread a radius
/// from the middle along each axis. All nine are put back onto the ball, and
/// eight of the nine land on the seam's own meridian plane or on the wrong side
/// of it: the blends and the two spread along `x` land on the prime meridian, the
/// two along `z` land on the poles, and one of the two along `y` is on the far
/// side. The ninth is `centre + (0, r, 0)`. Put a wall exactly there and it comes
/// back `OnBoundary`, which is a candidate that says nothing and is abandoned —
/// so reading the seam as a trimming leaves selection with no candidate at all
/// and the fuse refuses a pair it can plainly answer.
///
/// Measured: with the seam read as a trimming this refuses `Undecidable` on
/// *a point inside a piece of a face*, and every other row in this file stays
/// green. The ball is at `(4, 4, 4)` with radius two and the wall is `y = 6`,
/// all whole numbers, so the touch is exact.
#[test]
fn a_ball_touching_a_wall_at_the_one_point_that_can_place_it_keeps_both() {
    let mut model = a_model();
    let ball = model
        .add_ball(
            2.0,
            &Isometry::translation(Vector::new(4.0, 4.0, 4.0)).expect("a shift"),
        )
        .expect("a ball of radius two");
    let wall = block(&mut model, [8.0, 4.0, 8.0], Vector::new(0.0, 6.0, 0.0));

    let united = fused(&mut model, &Pair::of(ball, wall), Rule::Union);
    assert_eq!(united.solids().len(), 2, "two solids touching at one point");
    measures(
        volume_of(&model, united.solids()),
        4.0 / 3.0 * PI * 8.0 + 256.0,
        Side::Under,
    );
}

/// A block of side two standing on the bar's cap with its low corner at
/// `(x, y, 8)`, and which of the two is the argument.
fn a_block_on_a_bars_cap(model: &mut Model, [x, y]: [f64; 2], block_first: bool) -> Pair {
    let bar = a_bar(model);
    let sitting = block(model, [2.0, 2.0, 2.0], Vector::new(x, y, 8.0));
    if block_first {
        Pair::of(sitting, bar)
    } else {
        Pair::of(bar, sitting)
    }
}

/// A block sitting on a bar's cap with its corner on the rim meets the bar at
/// that one point, under every rule and both ways round.
///
/// The block covers `2 ≤ x ≤ 4` and `0 ≤ y ≤ 2` and stands on `z = 8`, so it
/// meets the bar at `(2, 0, 8)` and nowhere else. The union is the two solids
/// as they stand, `32π + 8`; they share no space, so the intersection is
/// empty; and each less the other is itself, the bar or the block.
///
/// **What refused it until queue row Q930 was one line drawn as two
/// crossings.** A cut is by a plane and not by the face carrying it (`cuts.rs`
/// says why), so the plane `x = 2` rests on the bar's wall along the ruling
/// `(2, 0, z)`, and the plane `y = 0` cuts the wall along that same ruling,
/// the one of its two picked by `x ≥ 0`. `crossing_like` folds two partners
/// only where they are one surface, or are planes cutting a flat face in one
/// line, and these are two planes cutting a pipe; so at `(2, 0, 0)` two ways
/// out left up the ruling, one along each, bending alike, and no turn parted
/// them: `TwoCurvesTouchWithoutCrossing`, under every rule. The block first,
/// it refused the same way on the block's wall `y = 0` at `(2, 0, 8)`, where
/// the wall's edge with `x = 2` and the bar's line `x = 2` are the one line.
/// A print where the sort refused named those two faces; this test's earlier
/// doc guessed the wall `x = 2`, and nothing refused there. `arrange.rs` now
/// folds a straight cut into a straight crossing drawn before it when the two
/// leave a node along one line (§39.4).
///
/// **The controls.** Turned a quarter, a half and three quarters round the
/// bar's axis, and mirrored across `y = 0`, the block puts its corner on the
/// rim away from the bar's seam and on the seam's other side: the same shape,
/// held to the same answers. A quarter turn from the seam is what needs the
/// fold after the nodes that are one point are merged, since there the ruling
/// ends on the cap circle at two meets, one per crossing, and no corner of the
/// face holds both. Slid a tenth clear along `x`, the block touches nothing;
/// slid a tenth along `−y`, its corner leaves the rim while the plane `x = 2`
/// still rests on the bar, so the touch stays and the second line goes. Both
/// give the same three answers too.
#[test]
fn a_block_sitting_on_a_bars_cap_meets_it_at_one_point_under_every_rule() {
    for (placed, at) in [
        ("on the rim at the bar's seam", [2.0, 0.0]),
        ("turned a quarter round", [-2.0, 2.0]),
        ("turned half round", [-4.0, -2.0]),
        ("turned three quarters round", [0.0, -4.0]),
        ("mirrored across y = 0", [2.0, -2.0]),
        ("slid a tenth clear", [2.1, 0.0]),
        ("slid a tenth along the touch", [2.0, -0.1]),
    ] {
        for block_first in [false, true] {
            let asked = |rule: Rule| format!("{placed}, the block first {block_first}, {rule:?}");

            let mut model = a_model();
            let pair = a_block_on_a_bars_cap(&mut model, at, block_first);
            let united = fused(&mut model, &pair, Rule::Union);
            assert_eq!(united.solids().len(), 2, "{}", asked(Rule::Union));
            measures(volume_of(&model, united.solids()), A_BAR + 8.0, Side::Under);

            let mut model = a_model();
            let pair = a_block_on_a_bars_cap(&mut model, at, block_first);
            let shared = fused(&mut model, &pair, Rule::Intersection);
            assert!(shared.solids().is_empty(), "{}", asked(Rule::Intersection));

            let mut model = a_model();
            let pair = a_block_on_a_bars_cap(&mut model, at, block_first);
            let left = fused(&mut model, &pair, Rule::Difference);
            assert_eq!(left.solids().len(), 1, "{}", asked(Rule::Difference));
            let found = volume_of(&model, left.solids());
            if block_first {
                assert!(
                    (found - 8.0).abs() < 1e-12,
                    "{}: a block is flat and meshes exactly, got {found}",
                    asked(Rule::Difference)
                );
            } else {
                measures(found, A_BAR, Side::Under);
            }
        }
    }
}

// §37.5's one unclassified refusal, classified and closed.

/// A bar with a rectangular tab on it: `ResultIsNotManifold { count: 1 }`
/// classified, and it was **the chord standing in for an arc** (§38.5).
///
/// §37.5 reached an edge used *once* rather than four times — a hole in the
/// skin, not §17.4's four faces along an edge — and left it unclassified because
/// nothing had worked out whether the answer exists. It does, and this is it: a
/// bar of radius two with a block on `1 ≤ x ≤ 3`, `−1 ≤ y ≤ 7`, `3 ≤ z ≤ 5`, so
/// the block's two inner walls cut the bar and the rest of it sticks out.
///
/// **A gap** (§16.10). The block's top at `z = 5` comes apart into the bit inside
/// the bar and the bit outside it, and the outside bit has one *arc* among its
/// sides. A piece is kept by classifying a point provably inside it, and the
/// winding count that proves it reads the corners as a polygon — so the middle of
/// the **chord** across that arc counted as inside the piece, when it is on the
/// far side of the arc and inside the bar. The piece was classified `Inside`,
/// dropped, and the arc left with one face along it. The manifold check caught
/// the hole, so what came out was a refusal and not a corrupt solid; what it was
/// refusing was a wrong classification.
///
/// The three closed forms are exact and worth writing out, because the shared
/// piece is the one number here that is not obvious: the disc of radius two with
/// `x ≥ 1` has area `4π/3 − √3`, the corner of it with `y ≤ −1` has area
/// `1 − √3 + π/3`, and the difference is exactly `π − 1`.
#[test]
fn a_bar_with_a_tab_on_it_is_answered_rather_than_left_with_a_hole_in_its_skin() {
    let shared = 2.0 * (PI - 1.0);
    for (rule, closed_form) in [
        (Rule::Union, A_BAR + 32.0 - shared),
        (Rule::Intersection, shared),
        (Rule::Difference, A_BAR - shared),
    ] {
        let mut model = a_model();
        let bar = a_bar(&mut model);
        let tab = block(&mut model, [2.0, 8.0, 2.0], Vector::new(1.0, -1.0, 3.0));
        let answer = fused(&mut model, &Pair::of(bar, tab), rule);
        assert_eq!(answer.solids().len(), 1, "{rule:?}");
        measures(volume_of(&model, answer.solids()), closed_form, Side::Under);
    }
}

// The gaps §36 already named, placed here so the catalogue is where they are
// looked up rather than four sections apart.

/// A notched bar turned about an awkward axis refuses inside the arrangement,
/// before any edge is named, and refuses the same way under a tolerant policy.
///
/// §36.28 records this. What the catalogue adds is that it is the same shape as
/// [`a_notch_across_the_seam`], which is decided and records nothing where it
/// stands — so the turn is the whole of the difference.
#[test]
fn a_turned_notched_bar_is_refused_for_want_of_an_order_along_a_crossing() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        let mut model = a_model();
        let pair = a_notch_across_the_seam(&mut model);
        let axis = Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("an axis");
        let turn = Isometry::rotation(&axis, 0.4).expect("a share of a turn");
        for &solid in pair.arguments.iter().chain(&pair.tools) {
            model
                .place_solid(solid, &turn)
                .expect("this model made the key");
        }
        let why = refusal(&mut model, &pair, Rule::Difference, &policy);
        assert_eq!(what_it_gave_up_on(&why), "order along a crossing");
    }
}

/// A bar cut by one plane that misses its axis is answered, and it used to be
/// §36.28's chord-and-arc gap.
///
/// **This is what counting a pipe's bulge bought.** The piece the fuse could not
/// place is the part of a cap disc left of `x = 1`: two corners on the rim, an
/// arc between them, and no polygon at all for the winding count to find a point
/// in. `select.rs`'s `in_the_bulge_of` counted a ball's arc and a cone's and not
/// a pipe's, so every candidate wound 0 and the fuse gave up on *a point inside
/// a piece of a face*. Counting the pipe's arc gives the same piece the same
/// treatment a half disc of a ball already got.
///
/// What is left is the slice of the bar with `x ≤ 1`, whole height. The disc of
/// radius two with `x ≥ 1` has area `4π/3 − √3` (the tab entry above works it
/// out), so what stays is `8π/3 + √3` per unit of height.
#[test]
fn a_bar_cut_by_one_plane_off_its_axis_is_answered() {
    let mut model = a_model();
    let bar = a_bar(&mut model);
    let knife = block(&mut model, [8.0, 8.0, 16.0], Vector::new(1.0, -4.0, -4.0));
    let cut = fused(&mut model, &Pair::of(bar, knife), Rule::Difference);
    assert_eq!(cut.solids().len(), 1, "one plane leaves one piece of a bar");
    measures(
        volume_of(&model, cut.solids()),
        8.0 * (8.0 * PI / 3.0 + 3.0_f64.sqrt()),
        Side::Under,
    );
}

/// A ball of radius four with a bar of radius one through its centre is
/// answered under every rule both ways, and a tolerant policy answers the same.
///
/// It was refused after its faces were cut: the result's mesh found two corners
/// of a face in one place, and the bar less the ball had faces that did not
/// agree which way they face (Q1241). The fuse writes each of the two circles
/// as a closed run with its corner on it and nothing picking it, and
/// `rcad_geom::walk` marched every such run from the seed on the circle ahead
/// of the centre, so the circle behind was walked round the one ahead. Each is
/// drawn in closed form now and told apart by its corner. Before that it was
/// refused `ACrossingThisCannotBeOrderedAlong` for every rule, as §36.20's
/// quartic, while the arrangement could not tell the two circles from the
/// quartic loops of a ball off the axis, which `intersect` names with the same
/// runs (Q1140).
///
/// [`what_a_ball_with_a_bar_through_its_centre_leaves`] has the closed forms.
#[test]
fn a_ball_with_a_bar_through_its_centre_is_answered_under_every_rule() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for (rule, ball_first, pieces, closed_form, how_fine) in
            what_a_ball_with_a_bar_through_its_centre_leaves()
        {
            let mut model = a_model();
            let ball = model
                .add_ball(4.0, &Isometry::identity())
                .expect("a ball of radius four");
            let bar = model
                .add_round_bar(
                    1.0,
                    12.0,
                    &Isometry::translation(Vector::new(0.0, 0.0, -6.0)).expect("a shift"),
                )
                .expect("a bar long enough to go right through");
            let pair = if ball_first {
                Pair::of(ball, bar)
            } else {
                Pair::of(bar, ball)
            };
            let answer = fused_under(&mut model, &pair, rule, &policy);
            assert_eq!(answer.solids().len(), pieces, "{policy:?}, {rule:?}");
            measures(
                volume_of_at(&model, answer.solids(), how_fine),
                closed_form,
                Side::Under,
            );
        }
    }
}

/// A ball one rounding off a bar's axis crosses it in two quartic loops and
/// not in two circles, and every fuse is refused meshing a face, under every
/// rule both ways. The bar's axis runs up `z` through `(1, 0, 0)` and the
/// ball's centre is the next `f64` past it along `x`, `2⁻⁵²` out; centred on
/// `(1, 0, 0)` itself, the same pair is answered, as the pair above is.
///
/// Until Q1554 it was refused before any order was asked,
/// `ACrossingThisCannotBeOrderedAlong`. The two loops are ordered round the
/// bar's axis now, and the mesh refuses instead: two corners of a face's chart
/// coincide, or a side cuts off one corner of a curved patch
/// ([`refused_meshing_a_ball_one_rounding_off_a_bars_axis`]). Not
/// traced. A ball of 2 with a bar of 1 half a unit off its axis is refused
/// `CornersCoincide` too, with the bar first only, in the marcher's first step
/// (`two_curves_of_one_pair_that_meet.rs`).
#[test]
fn a_ball_one_rounding_off_a_bars_axis_is_refused_meshing_every_fuse() {
    let one_past = f64::from_bits(1.0_f64.to_bits() + 1);
    for (centre, on_the_axis) in [(1.0, true), (one_past, false)] {
        for (rule, ball_first, pieces, closed_form, how_fine) in
            what_a_ball_with_a_bar_through_its_centre_leaves()
        {
            let mut model = a_model();
            let ball = model
                .add_ball(
                    4.0,
                    &Isometry::translation(Vector::new(centre, 0.0, 0.0)).expect("a shift"),
                )
                .expect("a ball of radius four");
            let bar = model
                .add_round_bar(
                    1.0,
                    12.0,
                    &Isometry::translation(Vector::new(1.0, 0.0, -6.0)).expect("a shift"),
                )
                .expect("a bar long enough to go right through");
            let pair = if ball_first {
                Pair::of(ball, bar)
            } else {
                Pair::of(bar, ball)
            };
            if on_the_axis {
                let answer = fused(&mut model, &pair, rule);
                assert_eq!(answer.solids().len(), pieces, "{rule:?}");
                measures(
                    volume_of_at(&model, answer.solids(), how_fine),
                    closed_form,
                    Side::Under,
                );
            } else {
                let why = refusal(&mut model, &pair, rule, &Policy::certified());
                assert!(
                    refused_meshing_a_ball_one_rounding_off_a_bars_axis(&why, rule, ball_first),
                    "{rule:?}, the ball first {ball_first}: refused with {why:?}"
                );
            }
        }
    }
}

/// Each rule, with `true` for the ball named first, and what a ball of radius
/// four with a bar of radius one and length twelve through its centre leaves
/// under it: how many solids, how much space they hold, and the deflection to
/// measure that at.
///
/// The bar reaches past the ball both ends, so the ball less the bar is
/// `(4π/3)(R² − r²)^(3/2) = (4π/3)·15^(3/2)`, about 243.35; the two share the
/// ball less that, about 24.74; the union is the ball and the bar's `12π` less
/// what they share; and the bar less the ball is `12π` less what they share, in
/// two pieces. The two that are no wider than the bar are measured at a
/// quarter of [`A_DEFLECTION`], which leaves what they share 0.52% short.
fn what_a_ball_with_a_bar_through_its_centre_leaves() -> [(Rule, bool, usize, f64, f64); 6] {
    let left = 4.0 * PI / 3.0 * 15.0_f64.powf(1.5);
    let shared = A_BIG_BALL - left;
    let the_bar = 12.0 * PI;
    let union = A_BIG_BALL + the_bar - shared;
    let thin = A_DEFLECTION / 4.0;
    [
        (Rule::Union, true, 1, union, A_DEFLECTION),
        (Rule::Union, false, 1, union, A_DEFLECTION),
        (Rule::Intersection, true, 1, shared, thin),
        (Rule::Intersection, false, 1, shared, thin),
        (Rule::Difference, true, 1, left, A_DEFLECTION),
        (Rule::Difference, false, 2, the_bar - shared, thin),
    ]
}

/// Whether `why` is the mesh refusal each fuse of
/// [`a_ball_one_rounding_off_a_bars_axis_is_refused_meshing_every_fuse`]'s ball
/// off the axis gives: two corners of a face's chart in one place with the
/// ball first, and with the bar first for the bar less the ball; a side
/// cutting off one corner of a curved patch for the union and the intersection
/// with the bar first. Only the kind is read, not the corner it names.
fn refused_meshing_a_ball_one_rounding_off_a_bars_axis(
    why: &Error,
    rule: Rule,
    ball_first: bool,
) -> bool {
    match (rule, ball_first) {
        (Rule::Union | Rule::Intersection, false) => matches!(
            why,
            Error::CannotMeshAFace {
                why: rcad_mesh::CannotTriangulate::ASideCutsOffOneCornerOfACurvedPatch { .. },
                ..
            }
        ),
        _ => matches!(
            why,
            Error::CannotMeshAFace {
                why: rcad_mesh::CannotTriangulate::CornersCoincide { .. },
                ..
            }
        ),
    }
}

/// A ball of radius four in a bar of radius four down its axis touches the
/// bar's wall all the way round its middle, and every rule is answered both
/// ways round and under either policy, with no edge where the two touch. A bar
/// a unit wider, which the ball touches nowhere, is the control, and it answers
/// the same shapes.
///
/// It was expected to be refused (Q1244). Since Q1140 the arrangement lets a
/// ball centred on a pipe's axis through as a pair that crosses in circles
/// (`how_far_a_ball_sits_off_a_pipes_axis` in `fuse/arrange.rs`), and the
/// ordering refuses a touch,
/// `CannotFollowACrossingWhereItsSurfacesAreTangent` (`rcad-geom`'s
/// `a_ball_inscribed_in_a_pipe_has_no_direction_to_be_ordered_along`). None of
/// the twelve fuses of the touching bar meets that refusal, and where the
/// arrangement stops asking about the circle was not traced.
///
/// **Each answer is the two inputs' faces whole**: the union is the bar, what
/// they share is the ball, the ball less the bar is nothing, and the bar less
/// the ball is one solid, the bar's skin with the ball's skin inside it as a
/// void. At equal radii that void touches the wall round the circle, so the
/// inside of the answer is two pieces, above and below it, held as one solid —
/// the inputs as they stand, which is how `fuses_that_touch.rs` finds a ball
/// inside a bar touching it at one point answered, and §80.9 is where the form
/// of an answer that touches itself was left open. Every volume is the closed
/// form's.
///
/// [`what_a_ball_in_a_bar_leaves`] has the closed forms.
#[test]
fn a_ball_in_a_bar_of_its_own_radius_is_answered_under_every_rule() {
    for bar_radius in [4.0, 5.0] {
        for policy in [Policy::certified(), Policy::tolerant(100.0)] {
            for (rule, ball_first, left) in what_a_ball_in_a_bar_leaves(bar_radius) {
                let said = format!(
                    "a bar of radius {bar_radius}, {policy:?}, {rule:?}, the ball first {ball_first}"
                );
                let mut model = a_model();
                let pair = a_ball_in_a_bar(&mut model, bar_radius, ball_first);
                let answer = fused_under(&mut model, &pair, rule, &policy);
                let Some(left) = left else {
                    assert!(answer.solids().is_empty(), "{said}: {answer:?}");
                    continue;
                };
                assert_eq!(answer.solids().len(), 1, "{said}");
                let mut faces_of_each_shell: Vec<usize> = model
                    .solid(answer.solids()[0])
                    .expect("this model made the key")
                    .shells()
                    .map(|shell| shell.face_count())
                    .collect();
                faces_of_each_shell.sort_unstable();
                assert_eq!(faces_of_each_shell, left.faces_of_each_shell, "{said}");
                measures(
                    volume_of(&model, answer.solids()),
                    left.closed_form,
                    left.side,
                );
            }
        }
    }
}

/// A ball of radius four about the origin, and a bar of `bar_radius` twelve
/// long down the `z` axis through it, reaching two past the ball either end.
fn a_ball_in_a_bar(model: &mut Model, bar_radius: f64, ball_first: bool) -> Pair {
    let ball = model
        .add_ball(4.0, &Isometry::identity())
        .expect("a ball of radius four");
    let bar = model
        .add_round_bar(
            bar_radius,
            12.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -6.0)).expect("a shift"),
        )
        .expect("a bar longer than the ball");
    if ball_first {
        Pair::of(ball, bar)
    } else {
        Pair::of(bar, ball)
    }
}

/// Each rule, with `true` for the ball named first, and the one solid
/// [`a_ball_in_a_bar`] leaves under it, or `None` for no solid at all.
///
/// The ball is wholly inside the bar, so the union is the bar,
/// `12π·bar_radius²`, what they share is the ball, `256π/3`, the ball less the
/// bar is nothing, and the bar less the ball is the one less the other. That
/// last has a round wall outside and a round hole inside, so its mesh lands on
/// whichever side the two errors leave it: at equal radii they all but cancel,
/// 335.1056 against 335.1032.
fn what_a_ball_in_a_bar_leaves(bar_radius: f64) -> [(Rule, bool, Option<Left>); 6] {
    let the_bar = 12.0 * PI * bar_radius * bar_radius;
    let the_bar_alone = || Left {
        faces_of_each_shell: vec![3],
        closed_form: the_bar,
        side: Side::Under,
    };
    let the_ball_alone = || Left {
        faces_of_each_shell: vec![1],
        closed_form: A_BIG_BALL,
        side: Side::Under,
    };
    [
        (Rule::Union, true, Some(the_bar_alone())),
        (Rule::Union, false, Some(the_bar_alone())),
        (Rule::Intersection, true, Some(the_ball_alone())),
        (Rule::Intersection, false, Some(the_ball_alone())),
        (Rule::Difference, true, None),
        (
            Rule::Difference,
            false,
            Some(Left {
                faces_of_each_shell: vec![1, 3],
                closed_form: the_bar - A_BIG_BALL,
                side: Side::EitherWay,
            }),
        ),
    ]
}

/// The one solid a fuse leaves: how many faces each of its shells has, fewest
/// first, how much space it holds, and which side of that its mesh lands.
struct Left {
    faces_of_each_shell: Vec<usize>,
    closed_form: f64,
    side: Side,
}

// §31.7 criterion 14's table, and the rest of the catalogue beside it.

/// Criterion 14's four entries, side by side.
///
/// Four hand-written entries are not a sample, so this is a table and not a
/// rate — the planar catalogue's own reason. Each row is built by the very
/// function the assertions above are about, so the table cannot describe a pair
/// different from the one the tests are about.
#[test]
fn the_four_curved_entries_as_a_table() {
    print_the_table(&criterion_fourteens_four());
}

/// The rest of roadmap §7.4, asked again with a curved face in it.
#[test]
fn the_rest_of_the_catalogue_over_curved_solids_as_a_table() {
    print_the_table(&the_rest());
}

fn print_the_table(entries: &[Entry]) {
    println!("| §7.4 entry | the pair | rule | outcome |");
    println!("|---|---|---|---|");
    for entry in entries {
        let said = match &entry.case {
            Case::Fused {
                build,
                rule,
                outcome,
            } => {
                assert_eq!(
                    &what_happened(*build, *rule),
                    outcome,
                    "{}, {}: §37's table says otherwise",
                    entry.degeneracy,
                    entry.pair
                );
                format!("{rule:?} | {}", outcome.said())
            }
            Case::Elsewhere { rule, test } => format!("{rule:?} | by `{test}`"),
        };
        println!("| {} | {} | {said} |", entry.degeneracy, entry.pair);
    }
}

/// One row of the catalogue table.
struct Entry {
    /// Which of roadmap §7.4's degeneracies this is a case of.
    degeneracy: &'static str,
    /// The pair, in a few words.
    pair: &'static str,
    case: Case,
}

/// How a row is discharged.
enum Case {
    /// Build the pair here and ask `rule` of it.
    Fused {
        build: fn(&mut Model) -> Pair,
        rule: Rule,
        outcome: Outcome,
    },
    /// Discharged by a test that is not a plain fuse of a plain pair.
    Elsewhere { rule: Rule, test: &'static str },
}

/// What a certified fuse of a catalogue pair came back with.
///
/// §37 needed a fourth, [`Outcome::Nothing`] — an answer, proved, and empty,
/// where the geometry plainly had something in it. §38 left no row that says it,
/// so it is gone: an outcome nothing reaches is a row nobody can read.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Outcome {
    /// An answer, with every decision in it proved.
    Answered,
    /// A refusal naming a decision nothing could settle.
    GaveUp(&'static str),
    /// A refusal naming a shape this milestone does not build.
    Refused(&'static str),
}

impl Outcome {
    fn said(self) -> String {
        match self {
            Outcome::Answered => "answered".to_owned(),
            Outcome::GaveUp(predicate) => format!("refused, `Undecidable` on \"{predicate}\""),
            Outcome::Refused(named) => format!("refused, `{named}`"),
        }
    }
}

fn criterion_fourteens_four() -> Vec<Entry> {
    vec![
        Entry {
            degeneracy: "tangent spheres",
            pair: "two balls of radius two, centres four apart",
            case: Case::Fused {
                build: two_balls_touching_at_one_point,
                rule: Rule::Union,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "tangent spheres, the control",
            pair: "the same two balls, three apart, really crossing",
            case: Case::Elsewhere {
                rule: Rule::Union,
                test: "what_is_round_two_crossing_balls_is_their_union",
            },
        },
        Entry {
            degeneracy: "tangent cylinder-plane",
            pair: "a bar of radius two and a wall on `y = -2`",
            case: Case::Fused {
                build: a_block_whose_wall_touches_a_bar,
                rule: Rule::Difference,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "tangent cylinder-plane, over a cone",
            pair: "a frustum of slope an eighth and a wedge resting on its side",
            case: Case::Fused {
                build: a_wedge_lying_on_a_cones_side,
                rule: Rule::Difference,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "seam-crossing intersections",
            pair: "a bar notched across its seam",
            case: Case::Fused {
                build: a_notch_across_the_seam,
                rule: Rule::Difference,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "seam-crossing intersections",
            pair: "a ball cut by a plane meeting its seam once",
            case: Case::Fused {
                build: a_ball_cut_across_its_seam,
                rule: Rule::Difference,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "seam-crossing intersections",
            pair: "a ball cut by a plane meeting its seam twice",
            case: Case::Elsewhere {
                rule: Rule::Union,
                test: "a_ball_cut_off_its_polar_axis_is_answered_under_every_rule",
            },
        },
        Entry {
            degeneracy: "seam-crossing intersections",
            pair: "a ball cut by a plane down its polar axis",
            case: Case::Elsewhere {
                rule: Rule::Union,
                test: "a_ball_cut_by_a_plane_down_its_polar_axis_is_answered",
            },
        },
        Entry {
            degeneracy: "closed periodic surfaces",
            pair: "a block with a round hole right through it",
            case: Case::Fused {
                build: a_block_with_a_round_hole,
                rule: Rule::Difference,
                outcome: Outcome::Answered,
            },
        },
    ]
}

fn the_rest() -> Vec<Entry> {
    vec![
        // Refused on "a point in a piece that every solid could place" until
        // §224.4: the caps' one corner wound round nothing.
        Entry {
            degeneracy: "coincident curved faces",
            pair: "one bar built twice, in the same place",
            case: Case::Fused {
                build: a_bar_and_a_copy_of_it,
                rule: Rule::Union,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "vertex on a curved face",
            pair: "a 3x4 block corner on a bar of radius five",
            case: Case::Fused {
                build: |model| a_block_corner_on_a_bars_wall(model, 4.0),
                rule: Rule::Union,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "vertex on a curved face",
            pair: "the same corner on a ball of radius five",
            case: Case::Elsewhere {
                rule: Rule::Union,
                test: "a_block_corner_exactly_on_a_balls_skin_is_refused_by_name",
            },
        },
        Entry {
            degeneracy: "zero-length edges",
            pair: "two tools meeting on a plane that cuts a round wall",
            case: Case::Fused {
                build: a_bar_and_two_blocks_on_the_same_planes,
                rule: Rule::Difference,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "sliver faces",
            pair: "a bar cut by a slab a millionth thick",
            case: Case::Fused {
                build: a_bar_and_a_knife_a_sliver_thick,
                rule: Rule::Intersection,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "fully contained solids",
            pair: "a bar in the middle of a block, united",
            case: Case::Fused {
                build: a_bar_wholly_inside_a_block,
                rule: Rule::Union,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "fully contained solids",
            pair: "the same, hollowed out",
            case: Case::Fused {
                build: a_bar_wholly_inside_a_block,
                rule: Rule::Difference,
                outcome: Outcome::Answered,
            },
        },
        Entry {
            degeneracy: "fully contained solids",
            pair: "a ball in the middle of a block",
            case: Case::Elsewhere {
                rule: Rule::Intersection,
                test: "a_ball_wholly_inside_a_block_answers_all_three_rules",
            },
        },
        Entry {
            degeneracy: "touching, not overlapping",
            pair: "a ball resting on a block's top at one point",
            case: Case::Elsewhere {
                rule: Rule::Union,
                test: "a_ball_resting_on_a_block_at_one_point_keeps_both",
            },
        },
        Entry {
            degeneracy: "touching, not overlapping",
            pair: "a block sitting on a bar's cap",
            case: Case::Elsewhere {
                rule: Rule::Union,
                test: "a_block_sitting_on_a_bars_cap_meets_it_at_one_point_under_every_rule",
            },
        },
        Entry {
            degeneracy: "self-intersecting inputs",
            pair: "nothing in the public API builds a curved one",
            case: Case::Elsewhere {
                rule: Rule::Union,
                test: "a_solid_that_runs_through_itself_cannot_be_fused",
            },
        },
    ]
}

/// Build a catalogue pair, fuse it under a certified policy, and say what came
/// back.
fn what_happened(build: fn(&mut Model) -> Pair, rule: Rule) -> Outcome {
    let mut model = a_model();
    let pair = build(&mut model);
    let mut log = ProvenanceLog::new();

    match model.fuse(
        &pair.arguments,
        &pair.tools,
        rule,
        &Policy::certified(),
        &mut log,
    ) {
        Ok(fused) => {
            assert!(
                log.records().is_empty(),
                "a certified fuse of an exactly written pair records nothing at all"
            );
            assert!(
                !fused.solids().is_empty(),
                "an answer with no solids in it is the wrong answer §38.1 closed"
            );
            Outcome::Answered
        }
        Err(Error::Undecidable { records }) => Outcome::GaveUp(
            match records
                .records()
                .last()
                .expect("a refusal names a decision")
                .predicate()
            {
                "a point inside a piece of a face" => "a point inside a piece of a face",
                "a point in a piece that every solid could place" => {
                    "a point in a piece that every solid could place"
                }
                "order along a crossing" => "order along a crossing",
                "which way a crossing runs" => "which way a crossing runs",
                other => panic!("a refusal named {other:?}, which no row expects"),
            },
        ),
        Err(Error::TwoCurvesTouchWithoutCrossing { .. }) => {
            Outcome::Refused("TwoCurvesTouchWithoutCrossing")
        }
        Err(Error::ACurvedFaceGoesRightRoundWithNoSeam { .. }) => {
            Outcome::Refused("ACurvedFaceGoesRightRoundWithNoSeam")
        }
        Err(Error::ACrossingThisCannotBeOrderedAlong) => {
            Outcome::Refused("ACrossingThisCannotBeOrderedAlong")
        }
        Err(Error::AClosedEdgeOfTheAnswerWouldStartAtAPole { .. }) => {
            Outcome::Refused("AClosedEdgeOfTheAnswerWouldStartAtAPole")
        }
        Err(why) => panic!("no row of this table expects {why:?}"),
    }
}

/// The same ball, the same slab, three axes, **one answer**.
///
/// §39.7 item 1 wanted this: a ball cut anywhere but through a pole used to be
/// refused, and it refused two different ways. Along `y` no pole was found to
/// fan the cap up to and it came back `ACurvedFaceGoesRightRoundWithNoSeam`;
/// along `x` the *wrong* pole was found and the triangulator was handed a
/// boundary that crosses itself. One cause under both: a wire made of one closed
/// edge was **assumed** to run right round the surface, and on a ball a closed
/// edge is any circle at all — `atan2(1, ±√15)` and `atan2(±√15, 1)` are 2.636
/// radians of longitude apart either way and neither circle encircles a pole.
///
/// **The three rows must agree, and that is §19 and not tidiness**: same ball,
/// same slab, same one unit from the centre, so the same spherical cap
/// `πh²(3r − h)/3` with `r = 4` and `h = 3`, which is `27π`. A rigid motion of
/// both operands is not allowed to change the answer, and the axis a cut is
/// square to is nothing but a rigid motion of the pair.
#[test]
fn the_same_ball_cut_the_same_way_along_three_axes_gives_the_same_cap() {
    for (axis, shift) in the_three_axes(1.0) {
        let mut model = a_model();
        let cut = a_ball_cut_one_unit_from_its_centre(&mut model, &Isometry::identity(), shift);
        match cut {
            Ok(fused) => {
                assert_eq!(
                    fused.solids().len(),
                    1,
                    "a ball cut once square to {axis} is one solid"
                );
                measures(volume_of(&model, fused.solids()), A_CAP_ONE_IN, Side::Under);
            }
            Err(why) => panic!("the cut square to {axis} says {why:?}"),
        }
    }
}

/// The volume of the cap this file cuts off a ball of radius four one unit from
/// its centre: `πh²(3r − h)/3` with `h = 3`, which is `27π`.
const A_CAP_ONE_IN: f64 = PI * 27.0;

/// The one cut, square to each of the three axes in turn, as the shift that
/// puts the twenty-cube's near face `depth` from the ball's centre.
///
/// A rigid motion of both operands and nothing else: the ball is the same ball
/// whichever axis is named, and the slab is the same slab turned a quarter turn
/// about the origin.
fn the_three_axes(depth: f64) -> [(&'static str, Vector); 3] {
    [
        ("z", Vector::new(-10.0, -10.0, depth)),
        ("y", Vector::new(-10.0, depth, -10.0)),
        ("x", Vector::new(depth, -10.0, -10.0)),
    ]
}

/// The cap a plane `depth` from the centre leaves on a ball of radius four:
/// `πh²(3r − h)/3` with `h = 4 − depth`.
fn a_cap_cut_at(depth: f64) -> f64 {
    let height = 4.0 - depth;
    PI * height * height * (12.0 - height) / 3.0
}

/// A ball of radius four turned by `placement`, cut by a twenty-cube shifted so
/// that one of its faces sits one unit from the ball's centre.
fn a_ball_cut_one_unit_from_its_centre(
    model: &mut Model,
    placement: &Isometry,
    shift: Vector,
) -> Result<Fused, Error> {
    let ball = model
        .add_ball(4.0, placement)
        .expect("a ball of radius four");
    let block = model
        .add_cube(20.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a block round it");
    let put_it = placement.compose(&Isometry::translation(shift).expect("a shift"));
    model.place_solid(block, &put_it)?;
    model.fuse(
        &[ball],
        &[block],
        Rule::Intersection,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

/// Turning the ball and the slab **together** does not change the cap they
/// leave, at any angle.
///
/// This is §19 itself, and it is the half of §39.7 item 1 that made it first in
/// the list. §39.3 measured this very sweep about `x` on the M4 tree and found
/// it answered from 1.00 to 1.30 radians and refused from 1.40 to 1.50, either
/// side of `acos(1/4) = 1.3181` — the angle at which the cap stops holding a
/// pole. A rigid motion of both operands is not allowed to decide anything, and
/// a boundary at `acos(1/4)` is a fact about the ball's stored parametrisation
/// and not about the pair.
///
/// Twenty-one angles rather than the four §39.3 quotes, so that the flip cannot
/// hide between two of them: `acos(1/4)` is 1.3181 and the steps are 0.1 apart.
#[test]
fn turning_a_ball_and_the_slab_that_cuts_it_together_does_not_change_the_cap() {
    let axis = Axis::new(Point::origin(), Vector::new(1.0, 0.0, 0.0)).expect("the x axis");
    for tenth in 0..=20 {
        let angle = f64::from(tenth) / 10.0;
        let mut model = a_model();
        let turn = Isometry::rotation(&axis, angle).expect("a share of a turn");
        let cut =
            a_ball_cut_one_unit_from_its_centre(&mut model, &turn, Vector::new(-10.0, 1.0, -10.0));
        match cut {
            Ok(fused) => {
                assert_eq!(fused.solids().len(), 1, "turned {angle} it is one solid");
                measures(volume_of(&model, fused.solids()), A_CAP_ONE_IN, Side::Under);
            }
            Err(why) => panic!("turned {angle} radians about x the pair says {why:?}"),
        }
    }
}

/// The same ball and the same slab, three axes and **three depths**, and the
/// nine answers are three volumes.
///
/// [`the_same_ball_cut_the_same_way_along_three_axes_gives_the_same_cap`] pins
/// one depth, and §41.6 is what a gate found under it: square to `z` the cut
/// answers down to a cap a hundredth of a unit high, and square to `x` or `y` it
/// stops answering at `h/r = 1 − cos(π/4) = 0.2929`, which on a ball of radius
/// four is a cap of **15.52** and 5.8% of the ball. Same two solids, same
/// relative position, a quarter turn apart, one answering and one refusing.
/// That is §19, and §19 is the property this kernel sells.
///
/// **The three columns are compared with each other and not only against the
/// closed form**, because agreeing with each other *is* the claim. A depth where
/// all three answered the same wrong volume would sail through a test that only
/// asked each of them to answer, and a cap is a shape whose closed form nobody
/// has to trust this file for.
///
/// Read at a tenth of [`A_DEFLECTION`], for the reason [`volume_of_at`] already
/// gives: a deflection is an absolute length and the deepest cut here leaves a
/// cap one unit high, where 0.02 is 0.6% short of the closed form on its own.
#[test]
fn the_same_ball_cut_at_three_depths_along_three_axes_gives_the_same_cap_each_time() {
    for depth in [1.0, 2.0, 3.0] {
        let closed_form = a_cap_cut_at(depth);
        let mut volumes = Vec::new();
        for (axis, shift) in the_three_axes(depth) {
            let mut model = a_model();
            match a_ball_cut_one_unit_from_its_centre(&mut model, &Isometry::identity(), shift) {
                Ok(fused) => {
                    assert_eq!(
                        fused.solids().len(),
                        1,
                        "a ball cut {depth} from its centre square to {axis} is one solid"
                    );
                    let found = volume_of_at(&model, fused.solids(), A_DEFLECTION / 10.0);
                    measures(found, closed_form, Side::Under);
                    volumes.push((axis, found));
                }
                Err(why) => panic!("cut {depth} from the centre square to {axis} says {why:?}"),
            }
        }
        let (widest, narrowest) = (
            volumes.iter().map(|(_, v)| *v).fold(f64::MIN, f64::max),
            volumes.iter().map(|(_, v)| *v).fold(f64::MAX, f64::min),
        );
        let apart = (widest - narrowest) / closed_form;
        assert!(
            apart < HOW_FAR_THREE_AXES_MAY_BE_APART,
            "cut {depth} from the centre the three axes give {volumes:?}, {:.4}% apart",
            100.0 * apart
        );
    }
}

/// How far the three axes' volumes may be from each other.
///
/// A twentieth of [`HOW_FAR_A_MESH_MAY_BE`], because the three are the same
/// shape meshed three ways and not three shapes: what separates them is the
/// order a sum of triangles is added up in and which way the tessellation
/// happened to fall, not anything about the cap. Measured, the worst of the nine
/// is 0.0033% and this is 0.025%.
const HOW_FAR_THREE_AXES_MAY_BE_APART: f64 = HOW_FAR_A_MESH_MAY_BE / 20.0;
