//! A round bar of radius 3 leaning across a round bar of radius 2 whose seam
//! it presses into, fused either way round (rows Q2571 and Q2590).
//!
//! # The shape
//!
//! The upright bar is radius 2, up `z` from −1 to 11, turned a quarter round
//! `z` so that its seam runs up `(0, 2, z)`. The leaning bar is radius 3 and
//! 20 long, its axis through `(0, 4, 5)` along `(cos 0.3, 0, −sin 0.3)`. Their
//! axes pass 4 apart, inside the 5 their radii add to, so the leaning bar
//! presses 1 into the upright one and the two pipes cross in one closed loop.
//! The upright bar's seam crosses that loop twice, at `(0, 2, 5 ± √5/cos 0.3)`,
//! and at both corners the loop runs exactly along the leaning pipe's ruling.
//!
//! # Why one order was refused
//!
//! With the upright bar first, the intersection was refused
//! `TheFuseWroteABrokenAnswer { broken: CoedgesRunTheSameWay }` and the union
//! and the difference `AWireWillNotLieFlatOnItsFace`; the leaning bar first
//! answered all three. `Edges::of` in `fuse/assemble.rs` read which way a
//! closed side is walked at its corner as the corner stood when the first face
//! asked for it, and `Corners::of` moved that corner afterwards, to the lower
//! rounding the other piece's meet gave it. The upright pipe's side was read at
//! `x = 4.695e-16` and its edge written at `x = 5.869e-17`. The way is the
//! sign of a dot product that is zero at the true corner, since the loop runs
//! along the ruling there, and its sign differed between the two roundings
//! (`+4.4e-16` and `−2.7e-15`), so the two faces walked the one edge the same
//! way. The corners are now all settled before any closed side is read.
//!
//! # A way read off rounding gives the right solid (Q2590)
//!
//! Settling the corners first makes both faces read the one sign, but at that
//! corner the sign is still rounding: the dot of the loop's direction with the
//! leaning pipe's longitude is zero in exact arithmetic and comes out about
//! `±1e-15`. The guess was that some such shape stores the edge the wrong way
//! on both faces, and so gives a solid that validates and is wrong.
//!
//! **It cannot while every reader takes the one sign.** The sign says only
//! which way the edge is *stored* running. The fuse's
//! `which_way_a_closed_edge_is_walked` then says whether each face walks it
//! that way, as the same sign read against which side of its partner the
//! face's material is on. So a face walks the edge along `∇S × ∇T` or against
//! it by its material alone, and the sign cancels out of where the face goes.
//! What the sign settles is the stored way, a convention, and
//! `Model::validate` reads it back with the same certified function at the
//! same corner.
//!
//! The pins below are the same two bars at other leans, with the pair turned
//! about the upright bar's axis and the leaning bar rolled about its own. At
//! each, the loop runs along the leaning pipe's ruling where it crosses the
//! upright seam. At each but the turn of four radians the fuse reads the way
//! against the leaning pipe, and the rounding comes out positive at some and
//! negative at others ([`the_shapes_here_read_the_way_off_rounding_and_get_both_signs`]).
//! Every answer measures what the formula says either way round. The lane
//! that ran this swept 89 shapes, at leans from 0.1 to 0.6, eight turns, two
//! rolls and three pairs of radii, with all three rules either way round: 534
//! fuses, none of them wrong, none failing `Model::validate`, and 83 refused.
//!
//! # A dot that rounds to nothing (Q2706)
//!
//! Leaning 0.6 and turned 2.9, and leaning 0.4 and turned 2.0, the dot rounds
//! to exactly `0.0` in `f64` and the certified rule says it is negative. The
//! mesher used to read the same rule in plain `f64`
//! (`the_way_a_closed_edge_is_stored_running` in `mesh.rs`), fell to the
//! latitude on that `0.0`, and stored the edge the other way from the fuse: the
//! union and both differences were refused `AWireWillNotLieFlatOnItsFace`. It
//! now reads the way with the fuse's own certified function.
//!
//! # What is still refused
//!
//! - Turned 4.0 or 5.5, and at some turns leaning 0.6, the union and one
//!   difference are refused `CurvedFaceWithAHoleGoesTooFarRound`. At 4.0 the
//!   fuse reads the way against the upright pipe rather than the leaning one
//!   ([`the_same_pair_turned_four_radians_is_refused_its_union`]). Leaning 0.6
//!   and turned 2.9 the union and the leaning bar less the upright one are
//!   refused the same way
//!   ([`the_union_where_the_dot_rounds_to_nothing_is_refused`]).
//!
//! The fuses run certified, in a model of 100 mm.

use std::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Decision, Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

/// How far Q2571's leaning bar's axis leans down from `x`, in radians.
const LEAN: f64 = 0.3;

/// The two bars share the same piece whichever is handed in first, and it
/// measures what they share.
#[test]
fn the_piece_two_bars_share_is_the_same_either_way_round() {
    let bars = TwoBars::as_q2571_built();
    let shared = what_the_two_bars_share(LEAN);
    let upright_first = answer(&bars, Rule::Intersection, true, shared, 2);
    let leaning_first = answer(&bars, Rule::Intersection, false, shared, 2);
    the_same_volume(upright_first, leaning_first, "the intersection");
}

/// The two bars unite to the same solid whichever is handed in first: each
/// pipe with a hole, and the four caps.
#[test]
fn two_bars_unite_to_the_same_solid_either_way_round() {
    let bars = TwoBars::as_q2571_built();
    let whole = UPRIGHT + LEANING - what_the_two_bars_share(LEAN);
    let upright_first = answer(&bars, Rule::Union, true, whole, 6);
    let leaning_first = answer(&bars, Rule::Union, false, whole, 6);
    the_same_volume(upright_first, leaning_first, "the union");
}

/// Each bar less the other answers, and measures its own volume less what
/// the two share: its pipe with a hole, its two caps, and the dent.
#[test]
fn each_bar_less_the_other_answers() {
    let bars = TwoBars::as_q2571_built();
    let shared = what_the_two_bars_share(LEAN);
    answer(&bars, Rule::Difference, true, UPRIGHT - shared, 4);
    answer(&bars, Rule::Difference, false, LEANING - shared, 4);
}

/// Leaning 0.6 and turned to 1.0, the corner's rounding is negative: the
/// edge is stored the other way from the one the latitude would name.
#[test]
fn leaning_further_and_turned_the_two_bars_answer_all_three_either_way_round() {
    all_three_answer_either_way_round(&TwoBars::turned(0.6, 1.0, 0.0));
}

/// Leaning 0.1 and turned to 2.0, the corner's rounding is negative.
#[test]
fn leaning_less_and_turned_the_two_bars_answer_all_three_either_way_round() {
    all_three_answer_either_way_round(&TwoBars::turned(0.1, 2.0, 0.0));
}

/// Leaning as Q2571's and turned to 1.0, the corner's rounding is positive.
#[test]
fn turned_the_other_way_the_two_bars_answer_all_three_either_way_round() {
    all_three_answer_either_way_round(&TwoBars::turned(LEAN, 1.0, 0.0));
}

/// Leaning 0.6, turned to 2.0 and the leaning bar rolled 1.3 about its own
/// axis, so that its seam is elsewhere, the corner's rounding is positive.
#[test]
fn with_the_leaning_bar_rolled_the_two_bars_answer_all_three_either_way_round() {
    all_three_answer_either_way_round(&TwoBars::turned(0.6, 2.0, 1.3));
}

/// Leaning 0.6 and turned to 2.9, the dot rounds to exactly `0.0` in `f64`,
/// and the piece the two share still answers right either way round.
#[test]
fn the_piece_two_bars_share_where_the_dot_rounds_to_nothing_is_the_same_either_way_round() {
    let bars = TwoBars::turned(0.6, 2.9, 0.0);
    let shared = what_the_two_bars_share(bars.lean);
    let upright_first = answer(&bars, Rule::Intersection, true, shared, 2);
    let leaning_first = answer(&bars, Rule::Intersection, false, shared, 2);
    the_same_volume(upright_first, leaning_first, "the intersection");
}

/// The same shape's upright bar less the leaning one answers, and its union
/// either way round and the leaning bar less the upright one are refused
/// `CurvedFaceWithAHoleGoesTooFarRound`, as at the turn of four radians.
///
/// All four were refused `AWireWillNotLieFlatOnItsFace` while the mesher read
/// the way the closed edge is stored running in plain `f64`: it got `0.0`
/// against the longitude where the fuse's certified reading is negative, fell
/// to the latitude, and stored the edge the other way from the fuse (Q2706).
#[test]
fn the_union_where_the_dot_rounds_to_nothing_is_refused() {
    let bars = TwoBars::turned(0.6, 2.9, 0.0);
    let shared = what_the_two_bars_share(bars.lean);
    answer(&bars, Rule::Difference, true, UPRIGHT - shared, 4);
    for (rule, upright_first) in [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Difference, false),
    ] {
        let why = refused(&bars, rule, upright_first);
        assert!(
            matches!(why, Error::CurvedFaceWithAHoleGoesTooFarRound { .. }),
            "{rule:?}, upright bar first: {upright_first}: {why:?}"
        );
    }
}

/// Leaning 0.4 and turned to 2.0, all three answer either way round.
///
/// Here too the dot against the longitude rounds to exactly `0.0` in `f64` at
/// the closed edge's corner, `(−0.832, 1.819, 7.428)`, where the fuse's
/// certified reading is negative. The union and both differences were refused
/// `AWireWillNotLieFlatOnItsFace` while the mesher read that dot in plain
/// `f64` and fell to the latitude (Q2706).
#[test]
fn leaning_0_4_and_turned_2_the_two_bars_answer_all_three_either_way_round() {
    all_three_answer_either_way_round(&TwoBars::turned(0.4, 2.0, 0.0));
}

/// Q2571's two bars turned 4.0 about the upright bar's axis, rather than a
/// quarter turn, answer the piece they share and the leaning bar less the
/// upright one, and are refused the union either way round and the upright
/// bar less the leaning one.
#[test]
fn the_same_pair_turned_four_radians_is_refused_its_union() {
    let bars = TwoBars::turned(LEAN, 4.0, 0.0);
    let shared = what_the_two_bars_share(bars.lean);
    let upright_first = answer(&bars, Rule::Intersection, true, shared, 2);
    let leaning_first = answer(&bars, Rule::Intersection, false, shared, 2);
    the_same_volume(upright_first, leaning_first, "the intersection");
    answer(&bars, Rule::Difference, false, LEANING - shared, 4);
    for (rule, upright_first) in [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Difference, true),
    ] {
        let why = refused(&bars, rule, upright_first);
        assert!(
            matches!(why, Error::CurvedFaceWithAHoleGoesTooFarRound { .. }),
            "{rule:?}, upright bar first: {upright_first}: {why:?}"
        );
    }
}

/// At each turned shape pinned above but the one turned four radians, the way
/// the closed edge is stored running is read off a dot product that is zero
/// but for its rounding, and that rounding is positive at some shapes and
/// negative at others. So the pins have a right answer with the edge stored
/// either way, and not only the one the sign happened to give at Q2571's shape.
///
/// It is read as the fuse reads it: off the answer's own two pipes at the
/// edge's corner, the loop's direction `∇S × ∇T` against the leaning pipe's
/// longitude `axis × ∇T`, with each gradient as `Model::validate` spells it.
/// The pipe the fuse reads against is whichever sorts first by its bits
/// (`the_curved_one_of` in `validate.rs`), not always the leaning one. A print
/// in the fuse, since removed, showed it reading the leaning pipe at these five
/// shapes, at the corner read here and to the sign read here, and the upright
/// pipe at the turn of four radians.
#[test]
fn the_shapes_here_read_the_way_off_rounding_and_get_both_signs() {
    let read: Vec<Decision> = [
        TwoBars::turned(0.6, 1.0, 0.0),
        TwoBars::turned(0.1, 2.0, 0.0),
        TwoBars::turned(LEAN, 1.0, 0.0),
        TwoBars::turned(0.6, 2.0, 1.3),
        TwoBars::turned(0.6, 2.9, 0.0),
    ]
    .iter()
    .map(the_way_read_at_the_corner)
    .collect();
    assert!(
        read.iter().any(|way| matches!(way, Decision::Pos))
            && read.iter().any(|way| matches!(way, Decision::Neg)),
        "the rounding came out one way at every shape: {read:?}"
    );
}

/// The upright bar's volume, `π · 2² · 12`.
const UPRIGHT: f64 = 48.0 * PI;

/// The leaning bar's volume, `π · 3² · 20`.
const LEANING: f64 = 180.0 * PI;

/// The two bars, placed.
struct TwoBars {
    said: String,
    lean: f64,
    upright: Isometry,
    leaning: Isometry,
}

impl TwoBars {
    /// Q2571's two bars, placed as that row placed them.
    fn as_q2571_built() -> Self {
        Self {
            said: "Q2571's".to_owned(),
            lean: LEAN,
            upright: seamed_up_y(),
            leaning: leaning_across(),
        }
    }

    /// The upright bar with its seam turned to `turned` about `z`, and the
    /// leaning bar `lean` down from level, its axis 4 out from the upright
    /// one's along that seam, square to it and 5 up, rolled `roll` about its
    /// own axis. At `turned = π/2` and `roll = 0` this is Q2571's pair, but
    /// for the rounding in its turns.
    fn turned(lean: f64, turned: f64, roll: f64) -> Self {
        let z = Axis::z();
        let the_y_axis = Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("an axis");
        Self {
            said: format!("leaning {lean}, turned {turned}, rolled {roll}"),
            lean,
            upright: moved_to([0.0, 0.0, -1.0])
                .compose(&Isometry::rotation(&z, turned).expect("a turn")),
            leaning: moved_to([4.0 * turned.cos(), 4.0 * turned.sin(), 5.0])
                .compose(&Isometry::rotation(&z, turned - PI / 2.0).expect("a turn"))
                .compose(&Isometry::rotation(&the_y_axis, PI / 2.0 + lean).expect("a turn"))
                .compose(&moved_to([0.0, 0.0, -10.0]))
                .compose(&Isometry::rotation(&z, roll).expect("a turn")),
        }
    }
}

/// All three rules answer either way round, and measure what the formula
/// says: the piece the two share, the union and each bar less the other.
fn all_three_answer_either_way_round(bars: &TwoBars) {
    let shared = what_the_two_bars_share(bars.lean);
    let upright_first = answer(bars, Rule::Intersection, true, shared, 2);
    let leaning_first = answer(bars, Rule::Intersection, false, shared, 2);
    the_same_volume(upright_first, leaning_first, "the intersection");
    let whole = UPRIGHT + LEANING - shared;
    let ends = faces_cut_off_the_leaning_ends(bars.lean);
    let upright_first = answer(bars, Rule::Union, true, whole, 6 + ends);
    let leaning_first = answer(bars, Rule::Union, false, whole, 6 + ends);
    the_same_volume(upright_first, leaning_first, "the union");
    answer(bars, Rule::Difference, true, UPRIGHT - shared, 4);
    answer(bars, Rule::Difference, false, LEANING - shared, 4 + ends);
}

/// How many faces the fuse adds to an answer holding the leaning bar's ends,
/// by cutting them along the planes of the upright bar's two caps, `z = 11`
/// and `z = −1`: two when the leaning bar reaches past those planes, and none
/// when it does not. Each end comes off its pipe as a face of its own, though
/// the cap whose plane cut it is a disc of radius 2 about nine away. The
/// answer measures right and validates all the same.
///
/// The leaning bar's highest point is its upper cap's centre, `10 sin lean`
/// above the `z = 5` its axis crosses at, plus that cap's radius of 3 tilted
/// by the lean. So its ends pass the planes when `10 sin lean + 3 cos lean >
/// 6`, which is from a lean of about 0.32.
fn faces_cut_off_the_leaning_ends(lean: f64) -> usize {
    if 10.0 * lean.sin() + 3.0 * lean.cos() > 6.0 {
        2
    } else {
        0
    }
}

/// What a measured volume is, and how far its mesh may miss it.
#[derive(Clone, Copy, Debug)]
struct Measured {
    volume: f64,
    error: f64,
}

/// The fuse `rule` of the two bars, the upright one first or not: assert that
/// it answers one solid of `faces` faces, that the model validates, and that
/// the solid measures `volume` to within its mesh's own error. Hands back what
/// it measured.
fn answer(bars: &TwoBars, rule: Rule, upright_first: bool, volume: f64, faces: usize) -> Measured {
    let what = format!(
        "{} bars, {rule:?}, upright bar first: {upright_first}",
        bars.said
    );
    let policy = Policy::certified();
    let (model, fused) = the_fuse(bars, rule, upright_first, &policy);
    let solids = fused.unwrap_or_else(|why| panic!("{what}: refused: {why:?}"));
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: the answer does not validate: {why:?}"));
    let [solid] = solids[..] else {
        panic!("{what}: one solid, got {solids:?}");
    };
    let measure = model
        .solid(solid)
        .expect("a solid the fuse made")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the answer does not measure: {why:?}"));
    let measured = Measured {
        volume: measure.volume(),
        error: measure.volume_error(),
    };
    assert_eq!(
        model
            .solid(solid)
            .expect("a solid the fuse made")
            .faces()
            .count(),
        faces,
        "{what}: faces"
    );
    assert!(
        (measured.volume - volume).abs() <= measured.error,
        "{what}: measured {measured:?} against {volume}"
    );
    measured
}

/// What the fuse `rule` of the two bars is refused with, the upright one
/// first or not.
fn refused(bars: &TwoBars, rule: Rule, upright_first: bool) -> Error {
    let (_, fused) = the_fuse(bars, rule, upright_first, &Policy::certified());
    let Err(why) = fused else {
        panic!(
            "{} bars, {rule:?}, upright bar first: {upright_first}: answered {fused:?}",
            bars.said
        );
    };
    why
}

/// The model holding the two bars, and what fusing them by `rule` gave.
fn the_fuse(
    bars: &TwoBars,
    rule: Rule,
    upright_first: bool,
    policy: &Policy,
) -> (Model, rcad_topo::Result<Vec<SolidKey>>) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let upright = model
        .add_round_bar(2.0, 12.0, &bars.upright)
        .expect("a bar");
    let leaning = model
        .add_round_bar(3.0, 20.0, &bars.leaning)
        .expect("a bar");
    let (arguments, tools) = if upright_first {
        ([upright], [leaning])
    } else {
        ([leaning], [upright])
    };
    let fused = model
        .fuse(&arguments, &tools, rule, policy, &mut ProvenanceLog::new())
        .map(|fused| fused.solids().to_vec());
    (model, fused)
}

/// Assert that the two orders' answers measure the same, to within the two
/// meshes' errors added.
fn the_same_volume(upright_first: Measured, leaning_first: Measured, what: &str) {
    assert!(
        (upright_first.volume - leaning_first.volume).abs()
            <= upright_first.error + leaning_first.error,
        "{what}: upright bar first {upright_first:?}, leaning bar first {leaning_first:?}"
    );
}

/// Which way the one closed edge of the piece the two bars share is stored
/// running, read against the leaning pipe as the fuse reads it at the shapes
/// this is handed, after asserting that the dot product it is read off is zero
/// but for rounding.
///
/// That dot is `(∇U × ∇L) · (axis × ∇L)` for `U` the upright pipe and `L` the
/// leaning one, both gradients of length 2 or 3 and the axis of length one,
/// so the latitude it falls back on comes to about 40; a dot under `1e-12`
/// is the rounding in a sum of terms that size.
fn the_way_read_at_the_corner(bars: &TwoBars) -> Decision {
    let policy = Policy::certified();
    let (model, fused) = the_fuse(bars, Rule::Intersection, true, &policy);
    let solids = fused.unwrap_or_else(|why| panic!("{} bars: refused: {why:?}", bars.said));
    let edge = model
        .solid(solids[0])
        .expect("a solid the fuse made")
        .faces()
        .flat_map(|face| face.wires())
        .flat_map(|wire| wire.coedges())
        .map(|coedge| coedge.edge())
        .find(|edge| edge.is_closed().expect("an edge of the model"))
        .unwrap_or_else(|| {
            panic!(
                "{} bars: the piece they share has no closed edge",
                bars.said
            )
        });
    let corner = edge.start().point();
    let pipes: Vec<(Vector, Vector)> = edge
        .neighbouring_faces()
        .map(|key| {
            let surface = model.face(key).expect("a face").surface();
            let Surface::Cylinder(pipe) = surface else {
                panic!(
                    "{} bars: a face of the piece they share is {surface:?}",
                    bars.said
                );
            };
            let (origin, along) = (pipe.axis().origin(), pipe.axis().direction());
            let out = corner - origin;
            (out * along.dot(along) - along * along.dot(out), along)
        })
        .collect();
    let [first, second] = pipes[..] else {
        panic!(
            "{} bars: one edge between two faces, got {pipes:?}",
            bars.said
        );
    };
    let is_upright = |(_, along): (Vector, Vector)| along.components()[2].abs() > 0.999;
    let ((upright, _), (leaning, leaning_axis)) = if is_upright(first) {
        (first, second)
    } else {
        (second, first)
    };
    let along_the_ruling = upright.cross(leaning).dot(leaning_axis.cross(leaning));
    assert!(
        along_the_ruling.abs() < 1e-12,
        "{} bars: the loop does not run along the leaning pipe's ruling: {along_the_ruling:e}",
        bars.said
    );
    rcad_geom::which_way_a_closed_crossing_turns([upright, leaning], leaning, leaning_axis)
}

/// The upright bar, up `z` from −1 to 11, turned a quarter round `z` so that
/// its seam runs up `(0, 2, z)`.
fn seamed_up_y() -> Isometry {
    moved_to([0.0, 0.0, -1.0])
        .compose(&Isometry::quarter_turns(&Axis::z(), 1).expect("a turn about z"))
}

/// The leaning bar, its axis through `(0, 4, 5)` along `(cos 0.3, 0, −sin
/// 0.3)`, reaching 10 each way from there.
fn leaning_across() -> Isometry {
    let the_y_axis = Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("an axis");
    moved_to([0.0, 4.0, 5.0])
        .compose(&Isometry::rotation(&the_y_axis, PI / 2.0 + LEAN).expect("a turn"))
        .compose(&moved_to([0.0, 0.0, -10.0]))
}

fn moved_to(at: [f64; 3]) -> Isometry {
    Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a finite move")
}

/// What the two bars share, at a lean of `lean`, by a midpoint rule of 1000
/// steps.
///
/// Turned and rolled as [`TwoBars::turned`] places them, the pair is Q2571's
/// at its own lean, and a turn changes no volume, so it is enough to measure
/// that one. A line up `z` through `(x, y)` inside the upright bar is inside
/// the leaning one where `(y − 4)² + (x·sin lean + (z − 5)·cos lean)² ≤ 9`, a
/// length of `2√(9 − (y − 4)²)/cos lean`, for `y` from 1 to 2, well inside
/// both bars' ends. Across `x` the upright bar is `2√(4 − y²)` wide, so the
/// volume is `(4/cos lean) ∫₁² √((2 − y)(y − 1)) √((2 + y)(7 − y)) dy`. Put
/// `y = 1.5 − cos(t)/2`: the first root is `sin(t)/2` and `dy` is
/// `sin(t)/2 dt`, which leaves a smooth integrand over `t` from 0 to π.
fn what_the_two_bars_share(lean: f64) -> f64 {
    let steps = 1000;
    let step = PI / f64::from(steps);
    let sum: f64 = (0..steps)
        .map(|at| {
            let t = step * (f64::from(at) + 0.5);
            let y = 1.5 - t.cos() / 2.0;
            t.sin().powi(2) / 4.0 * ((2.0 + y) * (7.0 - y)).sqrt()
        })
        .sum();
    4.0 / lean.cos() * sum * step
}
