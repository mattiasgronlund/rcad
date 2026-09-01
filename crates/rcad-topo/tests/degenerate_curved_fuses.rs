//! Roadmap §7.4's degeneracy catalogue over curved solids: the four entries
//! `degenerate_fuses.rs` could not reach with planes, and the other entries of
//! the same list asked again with a ball or a bar in them.
//!
//! §31.7 criterion 14. The planar file next door is the shape this follows, and
//! two things about it are deliberately not followed.
//!
//! # A row that refuses is a result; a row that answers wrongly is not
//!
//! The planar catalogue has two outcomes, answered and refused, and says a
//! refusal is as good an outcome as an answer provided it is right on its own
//! terms. Over curved solids there is a third, and it is the reason this file
//! exists rather than four more rows next door: **some of these pairs come back
//! with `Ok` and the wrong answer.** A ball that nothing cuts is dropped, with
//! no refusal and an empty log, and every entry about a ball that does not cross
//! its partner inherits that. Those rows are pinned here by tests whose names
//! say they are wrong, so that the day the kernel stops doing it something goes
//! red. **Nothing in this file blesses them.** §37 classifies each one.
//!
//! A fourth outcome is a **panic**, on input a caller can build through the
//! public API, which §16.11 says may not happen. Three of them are pinned the
//! only way a panic can be, with `#[should_panic]` and the message.
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

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Fused, Invalid, Model, ProfileBuilder, Rule, SolidKey};

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
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(
            &pair.arguments,
            &pair.tools,
            rule,
            &Policy::certified(),
            &mut log,
        )
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

/// How much space the answer takes up, all of its solids together.
fn volume_of(model: &Model, solids: &[SolidKey]) -> f64 {
    solids
        .iter()
        .map(|&solid| {
            model
                .solid(solid)
                .measure(
                    Deflection::of(A_DEFLECTION).expect("a fiftieth of a millimetre is a length"),
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

/// **A defect, pinned rather than blessed.** Two balls that touch at one point
/// unite into nothing at all.
///
/// The right answer is two solids that happen to touch — the planar catalogue's
/// `two_cubes_that_only_touch_at_a_corner` is the same question with corners and
/// gives exactly that. What comes back instead is `Ok` with an empty answer, no
/// refusal, and an empty log: a confident wrong answer, which §4.1 ranks below
/// both crashing and refusing.
///
/// **This entry never reaches tangency.** Neither ball's face is cut by the
/// other, and [`a_ball_that_nothing_cuts_is_dropped_by_the_fuse`] is the whole
/// of it — the same nothing comes back when the two balls are a mile apart. So
/// roadmap §7.4's tangent-sphere entry is not measuring tangency today, and
/// saying it passes would be the vacuity §14.7 exists to stop.
#[test]
fn two_balls_that_touch_at_one_point_come_back_as_nothing_which_is_wrong() {
    let mut model = a_model();
    let pair = two_balls_touching_at_one_point(&mut model);
    let each = volume_of(&model, &pair.arguments);
    measures(each, 4.0 / 3.0 * PI * 8.0, Side::Under);

    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        let answer = fused(&mut model, &pair, rule);
        assert!(
            answer.solids().is_empty(),
            "pinning what happens today: {rule:?} of two balls that touch gives nothing"
        );
    }
}

/// The root of it, in one fuse and one solid. A ball fused with no tool at all
/// comes back empty.
///
/// A ball is one face, its boundary is a seam with a pole at each end, and
/// nothing cuts it — so the arrangement of that face against the empty set of
/// cuts yields no piece the face's own boundary winds once round, and the solid
/// is gone. **A bar in the same position comes back whole**, which is the
/// control: a bar has three faces and its wall is bounded by a seam *and* two
/// cap circles.
#[test]
fn a_ball_that_nothing_cuts_is_dropped_by_the_fuse() {
    let mut model = a_model();
    let ball = model
        .add_ball(2.0, &Isometry::identity())
        .expect("a ball of radius two");
    measures(
        volume_of(&model, &[ball]),
        4.0 / 3.0 * PI * 8.0,
        Side::Under,
    );

    let gone = fused(
        &mut model,
        &Pair {
            arguments: vec![ball],
            tools: Vec::new(),
        },
        Rule::Union,
    );
    assert!(
        gone.solids().is_empty(),
        "pinning what happens today: a ball fused on its own is dropped"
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

/// The control that says the family is not empty everywhere: two balls that
/// really do cross each other are **refused by name** rather than dropped.
///
/// Which name depends on how far in they are, and §36.28 records only the first
/// of the two. Both are refusals and neither is a wrong answer, so this is the
/// row that shows the arithmetic can tell a crossing pair from a touching one —
/// what it cannot do is keep the touching one.
#[test]
fn two_balls_that_really_cross_are_refused_by_name() {
    for (apart, expected) in [
        (2.0, "goes right round with no seam"),
        (3.0, "encloses nothing"),
    ] {
        let mut model = a_model();
        let one = model
            .add_ball(2.0, &Isometry::identity())
            .expect("a ball of radius two");
        let other = model
            .add_ball(
                2.0,
                &Isometry::translation(Vector::new(apart, 0.0, 0.0)).expect("a shift"),
            )
            .expect("a ball of radius two");
        let pair = Pair::of(one, other);

        match refusal(&mut model, &pair, Rule::Union, &Policy::certified()) {
            Error::ACurvedFaceGoesRightRoundWithNoSeam { .. } => {
                assert_eq!(expected, "goes right round with no seam");
            }
            Error::Invalid(Invalid::VolumeIsNotPositive { volume }) => {
                assert_eq!(expected, "encloses nothing");
                assert_eq!(volume, 0.0);
            }
            other => panic!("two balls {apart} apart refused with {other:?}"),
        }
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

/// The tangency is refused by name, and the name is the tangency: two curves on
/// the bar's wall that touch and do not cross.
///
/// **The control is one coordinate moved by a tenth.** With the wall at
/// `y = -1.9` it cuts the bar in two curves that really do cross, and the
/// refusal changes to a different one by name — `Undecidable` over a point
/// inside a piece of a face, which is §36.28's chord-and-arc gap and has nothing
/// to do with tangency. Two named outcomes either side of one number is what
/// says this row is about what it claims.
///
/// A tolerant policy makes no difference, which is the other half: this is a
/// statement about the shape and not about a budget.
#[test]
fn a_wall_that_touches_a_bar_without_crossing_it_is_refused_by_name() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            let mut model = a_model();
            let pair = a_block_whose_wall_touches_a_bar(&mut model);
            match refusal(&mut model, &pair, rule, &policy) {
                Error::TwoCurvesTouchWithoutCrossing { .. } => (),
                other => panic!("a wall touching a bar's skin refused with {other:?}"),
            }
        }
    }

    let mut model = a_model();
    let bar = a_bar(&mut model);
    let crossing = block(&mut model, [2.0, 3.0, 2.0], Vector::new(1.0, -1.9, 3.0));
    let why = refusal(
        &mut model,
        &Pair::of(bar, crossing),
        Rule::Union,
        &Policy::certified(),
    );
    assert_eq!(
        what_it_gave_up_on(&why),
        "a point inside a piece of a face",
        "moving the wall off the tangent changes which refusal this is"
    );
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

/// The same ball cut by `x = 2` instead, where the crossing circle meets the
/// seam **twice**, is refused by name.
///
/// The circle `x = 2` on that ball meets the plane `y = 0` at `(2, 0, ±√12)`,
/// and both of those have `x ≥ 0`, so both are on the seam. One crossing works
/// and two do not, and the two cuts differ in nothing else: same ball, same
/// knife, same lattice, one axis.
#[test]
fn a_ball_cut_by_a_plane_that_crosses_its_seam_twice_is_refused_by_name() {
    let mut model = a_model();
    let ball = model
        .add_ball(4.0, &Isometry::identity())
        .expect("a ball of radius four");
    let knife = block(&mut model, [12.0, 12.0, 12.0], Vector::new(2.0, -6.0, -6.0));
    let pair = Pair::of(ball, knife);

    match refusal(&mut model, &pair, Rule::Union, &Policy::certified()) {
        Error::ACurvedFaceGoesRightRoundWithNoSeam { .. } => (),
        other => panic!("a cut meeting the seam twice refused with {other:?}"),
    }
}

/// **A defect, pinned rather than blessed.** The same ball cut by the plane its
/// seam lies in panics.
///
/// `x = 0` is the meridian plane, so the crossing does not cross the seam at a
/// point — it runs right along it. The fuse gets far enough to build an edge
/// with both its coedges the same way round and then trips the invariant that
/// says they may not be. **A panic on a solid a caller can build through the
/// public API**, which §16.11 says may not happen: this is a refusal that has
/// not been written yet.
#[test]
#[should_panic(expected = "the two coedges of an edge must run opposite ways")]
fn a_ball_cut_by_the_very_plane_its_seam_lies_in_panics() {
    let mut model = a_model();
    let ball = model
        .add_ball(4.0, &Isometry::identity())
        .expect("a ball of radius four");
    let knife = block(&mut model, [12.0, 12.0, 12.0], Vector::new(0.0, -6.0, -6.0));
    let _ = model.fuse(
        &[ball],
        &[knife],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
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
        .faces()
        .flat_map(|face| face.wires())
        .flat_map(|wire| wire.coedges())
        .filter(|coedge| coedge.edge().is_closed())
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

/// Two coincident curved faces are refused, and what it gives up on is placing a
/// point inside a piece of one of them.
///
/// The point it names is `(2, 0, 0)`, the bar's own seam corner, and the
/// enclosure is infinite rather than merely wide — nothing was nearly decided.
/// §36.16 lists this refusal and §36.28 says arcs make it reachable from a plain
/// shape; two copies of one bar is plainer still, and it is the same refusal
/// under a tolerant policy, so it is not a budget question.
///
/// **A gap and not a limit** (§16.10): the answer exists and is the bar.
#[test]
fn a_bar_and_a_copy_of_it_in_the_same_place_are_refused_for_want_of_a_point_inside_a_piece() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            let mut model = a_model();
            let pair = a_bar_and_a_copy_of_it(&mut model);
            let why = refusal(&mut model, &pair, rule, &policy);
            assert_eq!(what_it_gave_up_on(&why), "a point inside a piece of a face");
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

/// **A defect, pinned rather than blessed.** A corner exactly on a bar's wall
/// panics inside the arrangement.
///
/// Two arcs of the crossing leave that corner the same way, because the corner
/// is on the wall and on the block's two walls at once, and the walk that orders
/// what leaves a node asserts that cannot happen. It can, from public input.
#[test]
#[should_panic(expected = "two arcs leave one node the same way")]
fn a_block_corner_exactly_on_a_bars_wall_panics() {
    let mut model = a_model();
    let pair = a_block_corner_on_a_bars_wall(&mut model, 4.0);
    let _ = model.fuse(
        &pair.arguments,
        &pair.tools,
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
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

/// **A defect, pinned rather than blessed.** The same corner on a *ball* panics
/// in the same place, and its controls do not.
///
/// `(3, 4, 0)` is on a ball of radius five for the same reason. Either side of
/// it the fuse refuses with `Undecidable` naming order along a crossing rather
/// than answering, so the ball case is a panic between two refusals where the
/// bar case is a panic between two answers — the panic is the thing they have in
/// common and the exact incidence is the only thing that reaches it.
#[test]
#[should_panic(expected = "two arcs leave one node the same way")]
fn a_block_corner_exactly_on_a_balls_skin_panics() {
    let mut model = a_model();
    let ball = model
        .add_ball(5.0, &Isometry::identity())
        .expect("a ball of radius five");
    let standing = block(&mut model, [3.0, 4.0, 4.0], Vector::zero());
    let _ = model.fuse(
        &[ball],
        &[standing],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
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

/// Two tools meeting on a plane that cuts a round wall are refused, naming the
/// two rulings where the shared plane meets the bar.
///
/// The refusal's own record names `(0, -2, 0)` and `(0, 2, 0)`, which are
/// exactly the two points where the shared plane `x = 0` touches the cylinder of
/// radius two — so the entry demonstrably reaches its subject rather than
/// stopping somewhere else on the way.
///
/// **A gap** (§16.10): the answer exists, and it is the one the single-block
/// control below gives.
#[test]
fn two_tools_meeting_on_a_plane_that_cuts_a_round_wall_are_refused_by_name() {
    let mut model = a_model();
    let pair = a_bar_and_two_blocks_on_the_same_planes(&mut model);
    let why = refusal(&mut model, &pair, Rule::Difference, &Policy::certified());
    assert_eq!(what_it_gave_up_on(&why), "a point inside a piece of a face");

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
    assert_eq!(model.solid(disc.solids()[0]).faces().count(), 3);
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
    assert_eq!(model.solid(united.solids()[0]).faces().count(), 6);
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

/// The third rule leaves one solid with a bubble in it, which is §38.2's fix of
/// the panic §37.4 pinned here.
///
/// The planar catalogue's `a_cube_wholly_inside_another_under_all_three_rules`
/// is the same answer with boxes: an outer shell and a void inside it, one
/// solid. The volume is the block less the bar, and it measures **over** the
/// closed form because the bubble's wall is meshed inside the true cylinder, so
/// the bubble the mesh describes is smaller than the one that is there.
///
/// **The bubble is where a winding error would not cancel.** Its wall is a
/// closed edge walked by two faces, and a closed edge's two ends are one corner,
/// so nothing about the corner says which way either face walks it. Get it
/// backwards and the void's shell comes out wound the same way as the outer one
/// — its volume adds instead of subtracting, and the number below is `512 + 4π`
/// rather than `512 − 4π`. A mesh that merely closes says nothing about that;
/// the closed form does.
#[test]
fn hollowing_a_block_out_with_a_bar_inside_it_leaves_a_bubble() {
    let mut model = a_model();
    let pair = a_bar_wholly_inside_a_block(&mut model);
    let hollowed = fused(&mut model, &pair, Rule::Difference);

    assert_eq!(hollowed.solids().len(), 1, "one solid with a void in it");
    let solid = model.solid(hollowed.solids()[0]);
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
/// **This is the shape where the fix does not cancel**, and the bubble above is
/// the other: the hole has one cap inside the material and one outside, so a
/// wall walked the wrong way round takes the hole's volume off the block twice
/// instead of once and lands at `512 − 6π`. A hole through the block would have
/// the same wall wound the same way at both ends and could hide it.
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
    let hollowed = model.solid(drilled.solids()[0]);
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

/// **A defect, pinned rather than blessed.** A ball wholly inside a block is
/// dropped, so two of the three rules give a confident wrong answer.
///
/// The union is right by luck — the block swallows the ball either way. The
/// intersection should be the ball and comes back empty; the difference should
/// be a block with a bubble in it and comes back as the whole block. Both are
/// [`a_ball_that_nothing_cuts_is_dropped_by_the_fuse`] again, and the bar in the
/// same position gets two of the three right, which is what says it is the
/// ball's own shape and not containment.
#[test]
fn a_ball_wholly_inside_a_block_is_dropped_which_is_wrong() {
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
    let shared = fused(&mut model, &pair, Rule::Intersection);
    assert!(
        shared.solids().is_empty(),
        "pinning what happens today: what a block and a ball inside it share is nothing"
    );

    let mut model = a_model();
    let pair = build(&mut model);
    let hollowed = fused(&mut model, &pair, Rule::Difference);
    assert_eq!(
        volume_of(&model, hollowed.solids()),
        512.0,
        "pinning what happens today: hollowing a block with a ball takes nothing away"
    );
}

// 10. Touching but not overlapping — §7.4's eighth entry, asked of a curved
// surface.

/// **A defect, pinned rather than blessed.** A ball resting on a block's top,
/// touching it at one point, unites into the block alone.
///
/// The right answer is two solids that touch, which is what the planar
/// catalogue's `two_cubes_that_only_touch_at_a_corner` gives. The ball's centre
/// is at `z = 10` and the block's top is `z = 8`, both whole numbers, so the
/// touching point `(4, 4, 8)` is exact. It is dropped for the same reason again.
#[test]
fn a_ball_resting_on_a_block_at_one_point_loses_the_ball_which_is_wrong() {
    let mut model = a_model();
    let solid = block(&mut model, [8.0, 8.0, 8.0], Vector::zero());
    let ball = model
        .add_ball(
            2.0,
            &Isometry::translation(Vector::new(4.0, 4.0, 10.0)).expect("a shift"),
        )
        .expect("a ball of radius two");

    let united = fused(&mut model, &Pair::of(solid, ball), Rule::Union);
    assert_eq!(united.solids().len(), 1);
    assert_eq!(
        volume_of(&model, united.solids()),
        512.0,
        "pinning what happens today: the ball is not in the union at all"
    );
}

/// A bar and a block touching along a cap edge and nowhere else are refused by
/// name, and the name is the touching.
#[test]
fn a_block_sitting_on_a_bars_cap_is_refused_by_name() {
    let mut model = a_model();
    let bar = a_bar(&mut model);
    let sitting = block(&mut model, [2.0, 2.0, 2.0], Vector::new(2.0, 0.0, 8.0));
    match refusal(
        &mut model,
        &Pair::of(bar, sitting),
        Rule::Union,
        &Policy::certified(),
    ) {
        Error::TwoCurvesTouchWithoutCrossing { .. } => (),
        other => panic!("a block on a bar's cap refused with {other:?}"),
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
            model.place_solid(solid, &turn);
        }
        let why = refusal(&mut model, &pair, Rule::Difference, &policy);
        assert_eq!(what_it_gave_up_on(&why), "order along a crossing");
    }
}

/// A bar cut by one plane that misses its axis refuses for want of a point
/// inside a piece of a face, which is §36.28's chord-and-arc gap.
#[test]
fn a_bar_cut_by_one_plane_off_its_axis_is_refused_for_want_of_a_point_inside_a_piece() {
    let mut model = a_model();
    let bar = a_bar(&mut model);
    let knife = block(&mut model, [8.0, 8.0, 16.0], Vector::new(1.0, -4.0, -4.0));
    let why = refusal(
        &mut model,
        &Pair::of(bar, knife),
        Rule::Difference,
        &Policy::certified(),
    );
    assert_eq!(what_it_gave_up_on(&why), "a point inside a piece of a face");
}

/// A ball with a pipe through it refuses by name, and it is §36.20's quartic
/// rather than anything about arcs or seams.
///
/// A tolerant policy changes nothing, which is the half §36.20 did not measure:
/// ordering points along a crossing of two quadrics is not a budget question.
#[test]
fn a_ball_with_a_pipe_through_it_is_refused_by_name() {
    for policy in [Policy::certified(), Policy::tolerant(100.0)] {
        let mut model = a_model();
        let ball = model
            .add_ball(4.0, &Isometry::identity())
            .expect("a ball of radius four");
        let pipe = model
            .add_round_bar(
                1.0,
                12.0,
                &Isometry::translation(Vector::new(0.0, 0.0, -6.0)).expect("a shift"),
            )
            .expect("a pipe long enough to go right through");
        match refusal(&mut model, &Pair::of(ball, pipe), Rule::Difference, &policy) {
            Error::ACrossingThisCannotBeOrderedAlong => (),
            other => panic!("a ball with a pipe through it refused with {other:?}"),
        }
    }
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
            Case::Panics { rule, test } => format!("{rule:?} | **panics**, by `{test}`"),
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
    /// The fuse panics, so the row cannot be run alongside the others and names
    /// the `#[should_panic]` test that pins it instead.
    Panics { rule: Rule, test: &'static str },
    /// Discharged by a test that is not a plain fuse of a plain pair.
    Elsewhere { rule: Rule, test: &'static str },
}

/// What a certified fuse of a catalogue pair came back with.
///
/// [`Outcome::Nothing`] is the variant the planar catalogue has no need of and
/// this one cannot do without: an answer, proved, and empty, where the geometry
/// plainly has something in it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Outcome {
    /// An answer, with every decision in it proved.
    Answered,
    /// An answer, proved, with no solids in it — and wrong.
    Nothing,
    /// A refusal naming a decision nothing could settle.
    GaveUp(&'static str),
    /// A refusal naming a shape this milestone does not build.
    Refused(&'static str),
}

impl Outcome {
    fn said(self) -> String {
        match self {
            Outcome::Answered => "answered".to_owned(),
            Outcome::Nothing => "**answered with nothing, which is wrong**".to_owned(),
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
                outcome: Outcome::Nothing,
            },
        },
        Entry {
            degeneracy: "tangent spheres, the control",
            pair: "the same two balls, three apart, really crossing",
            case: Case::Elsewhere {
                rule: Rule::Union,
                test: "two_balls_that_really_cross_are_refused_by_name",
            },
        },
        Entry {
            degeneracy: "tangent cylinder-plane",
            pair: "a bar of radius two and a wall on `y = -2`",
            case: Case::Fused {
                build: a_block_whose_wall_touches_a_bar,
                rule: Rule::Difference,
                outcome: Outcome::Refused("TwoCurvesTouchWithoutCrossing"),
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
                test: "a_ball_cut_by_a_plane_that_crosses_its_seam_twice_is_refused_by_name",
            },
        },
        Entry {
            degeneracy: "seam-crossing intersections",
            pair: "a ball cut by the very plane its seam lies in",
            case: Case::Panics {
                rule: Rule::Union,
                test: "a_ball_cut_by_the_very_plane_its_seam_lies_in_panics",
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
        Entry {
            degeneracy: "coincident curved faces",
            pair: "one bar built twice, in the same place",
            case: Case::Fused {
                build: a_bar_and_a_copy_of_it,
                rule: Rule::Union,
                outcome: Outcome::GaveUp("a point inside a piece of a face"),
            },
        },
        Entry {
            degeneracy: "vertex on a curved face",
            pair: "a 3x4 block corner on a bar of radius five",
            case: Case::Panics {
                rule: Rule::Union,
                test: "a_block_corner_exactly_on_a_bars_wall_panics",
            },
        },
        Entry {
            degeneracy: "vertex on a curved face",
            pair: "the same corner on a ball of radius five",
            case: Case::Panics {
                rule: Rule::Union,
                test: "a_block_corner_exactly_on_a_balls_skin_panics",
            },
        },
        Entry {
            degeneracy: "zero-length edges",
            pair: "two tools meeting on a plane that cuts a round wall",
            case: Case::Fused {
                build: a_bar_and_two_blocks_on_the_same_planes,
                rule: Rule::Difference,
                outcome: Outcome::GaveUp("a point inside a piece of a face"),
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
                test: "a_ball_wholly_inside_a_block_is_dropped_which_is_wrong",
            },
        },
        Entry {
            degeneracy: "touching, not overlapping",
            pair: "a ball resting on a block's top at one point",
            case: Case::Elsewhere {
                rule: Rule::Union,
                test: "a_ball_resting_on_a_block_at_one_point_loses_the_ball_which_is_wrong",
            },
        },
        Entry {
            degeneracy: "touching, not overlapping",
            pair: "a block sitting on a bar's cap",
            case: Case::Elsewhere {
                rule: Rule::Union,
                test: "a_block_sitting_on_a_bars_cap_is_refused_by_name",
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
            if fused.solids().is_empty() {
                Outcome::Nothing
            } else {
                Outcome::Answered
            }
        }
        Err(Error::Undecidable { records }) => Outcome::GaveUp(
            match records
                .records()
                .last()
                .expect("a refusal names a decision")
                .predicate()
            {
                "a point inside a piece of a face" => "a point inside a piece of a face",
                "order along a crossing" => "order along a crossing",
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
        Err(why) => panic!("no row of this table expects {why:?}"),
    }
}
