//! A ball and a bar on one axis, one of them cut in half by a wall through
//! that axis, and a thin bar through a fat one, the thin one cut in half by a
//! wall through both axes, fused through the public API under every rule both
//! ways round, against closed-form volumes.
//!
//! A ball of radius 5 and a bar of radius 4 on one axis cross in two circles
//! square to it, three either side of the ball's centre. The wall puts two
//! corners on each circle, and the whole partner's skin holds the whole circle,
//! so a rule that keeps the partner's skin either side of the arc behind the
//! wall keeps that arc as well as the one along the half, between the same two
//! corners and along the same two surfaces. Named by their corners alone the two
//! arcs were one edge with four faces along it, and the unions both ways round
//! and the whole less the half were refused `ResultIsNotManifold { count: 4 }`.
//! `fuse/assemble.rs`'s `which_arc_of_a_circle_this_is` names them apart since
//! Q1329, as it does for a ball or a pipe on a cone's axis (Q1269,
//! `tests/a_cone_hollowed_along_its_axis.rs`), and every rule then answers.
//! Where the bar is the half and the ball's poles are on the wall, the ball's
//! skin is a band between a ring over each pole, which a ball less two blocks
//! leaves too, and the mesh the fuse checks its answer with cuts it since
//! Q1590.
//!
//! The rest of what a ball and a pipe cross in does not reach that function
//! with two corners on one curve. A wall or a cap through a loop off the axis
//! has its corners placed where the three surfaces meet since Q1591, and then
//! the one loop a ball reaching into a bar's side leaves is refused before its
//! arcs are named, as it is with no plane through it (Q1593). The circle a ball
//! touches its bar round is refused before its arcs are named.
//!
//! The two loops a thin bar through a fat one leaves, cut by a wall through
//! both axes, do reach it. Named by their corners alone, a loop's two arcs were
//! one edge with six faces along it or four, and every rule was refused as not
//! a manifold. That function names two pipes' arcs apart since Q1643, and every
//! rule was then refused one step on, where `rcad-geom`'s `walk` could not draw
//! an arc of such a loop between two corners. It draws one since Q1659, and
//! every rule answers, the part the two share held to a sum worked by hand and
//! the rest to that part as measured. The cross-drilled shaft of
//! `tests/a_cone_hollowed_along_its_axis.rs` (Q1483), whose fat bar's seam
//! crosses a loop twice, sent no side of two pipes to that function between
//! two corners when run with a print in it before Q1643.
//!
//! A ball keeps its poles on the `z` line through its centre however it is
//! placed, so the bars along `x` below keep them off the wall.

use std::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// What the half of the ball and the bar below share: the ball of radius 5
/// less the band of it outside radius 4, `500π/3 − 36π = 392π/3`, halved.
const WHAT_A_HALF_SHARES: f64 = 196.0 * PI / 3.0;

/// The ball of radius 5 at `(0, 0, 5)` cut to `x ≥ 0`, `250π/3`, against the
/// bar of radius 4 from `z = −1` to `z = 11`, `192π`, seamed up `−x`. The
/// ball's poles, `(0, 0, 0)` and `(0, 0, 10)`, are on the wall.
///
/// The union keeps the bar's skin at `x < 0` between the circles `z = 2` and
/// `z = 8`, and the ball's skin either side of the arc of each circle at
/// `x > 0`: four faces along two arcs between `(0, ±4, z)`. So does the bar
/// less the half. Each answers since Q1329: the union `250π/3 + 192π − 196π/3
/// = 210π`, the bar less the half `192π − 196π/3 = 380π/3`, and the half less
/// the bar `250π/3 − 196π/3 = 18π`.
#[test]
fn a_half_ball_against_a_bar_on_its_axis_answers_every_rule() {
    let gives = what_each_rule_gives(|model| {
        let ball = ball(model, [0.0, 0.0, 5.0]);
        [
            cut(model, ball, [0.0, -20.0, -20.0], [20.0, 20.0, 20.0]),
            bar_up_z(model),
        ]
    });
    let wanted = [
        Gives::Holds(210.0 * PI),
        Gives::Holds(210.0 * PI),
        Gives::Holds(WHAT_A_HALF_SHARES),
        Gives::Holds(WHAT_A_HALF_SHARES),
        Gives::Holds(18.0 * PI),
        Gives::Holds(380.0 * PI / 3.0),
    ];
    assert!(
        gives
            .iter()
            .zip(&wanted)
            .all(|(answer, wanted)| wanted.is(answer)),
        "{gives:?}"
    );
}

/// The bar of radius 4 from `x = −6` to `x = 6`, `192π`, and the ball of
/// radius 5 at the origin, `500π/3`, with the wall `z = 0` or `y = 0` through
/// both centres: the bar cut to `z ≥ 0`, to `y ≥ 0`, or the ball cut to
/// `z ≥ 0`. The ball's poles are off the wall, and its two arcs on each
/// circle `x = ±3` were one edge as above.
///
/// Half the bar, `96π`, against the ball gives a union of `96π + 500π/3 −
/// 196π/3 = 592π/3`, the half less the ball two ends of `46π/3`, and the ball
/// less the half `304π/3`. Half the ball against the bar gives what the half
/// ball of the test above does.
#[test]
fn a_half_bar_or_a_half_ball_with_the_balls_poles_off_the_wall_answers_every_rule() {
    let pairs: [fn(&mut Model) -> [SolidKey; 2]; 3] = [
        |model| {
            let bar = bar_along_x(model);
            [
                cut(model, bar, [-20.0, -20.0, 0.0], [20.0, 20.0, 20.0]),
                ball(model, [0.0, 0.0, 0.0]),
            ]
        },
        |model| {
            let bar = bar_along_x(model);
            [
                cut(model, bar, [-20.0, 0.0, -20.0], [20.0, 20.0, 20.0]),
                ball(model, [0.0, 0.0, 0.0]),
            ]
        },
        |model| {
            let ball = ball(model, [0.0, 0.0, 0.0]);
            [
                cut(model, ball, [-20.0, -20.0, 0.0], [20.0, 20.0, 20.0]),
                bar_along_x(model),
            ]
        },
    ];
    use Gives::{Holds, HoldsTwo};
    let half_a_bar = [
        Holds(592.0 * PI / 3.0),
        Holds(592.0 * PI / 3.0),
        Holds(WHAT_A_HALF_SHARES),
        Holds(WHAT_A_HALF_SHARES),
        HoldsTwo(46.0 * PI / 3.0, 46.0 * PI / 3.0),
        Holds(304.0 * PI / 3.0),
    ];
    let half_a_ball = [
        Holds(210.0 * PI),
        Holds(210.0 * PI),
        Holds(WHAT_A_HALF_SHARES),
        Holds(WHAT_A_HALF_SHARES),
        Holds(18.0 * PI),
        Holds(380.0 * PI / 3.0),
    ];
    for (pair, wanted) in pairs.into_iter().zip([half_a_bar, half_a_bar, half_a_ball]) {
        let gives = what_each_rule_gives(pair);
        assert!(
            gives
                .iter()
                .zip(&wanted)
                .all(|(answer, wanted)| wanted.is(answer)),
            "{gives:?}"
        );
    }
}

/// The bar of the first test cut to `x ≥ 0`, `96π`, against the whole ball of
/// radius 5 at `(0, 0, 5)`, whose poles are on the wall.
///
/// The unions and the ball less the half keep the ball less the two pieces of
/// it inside the half, one face with two rings: the arc of `z = 8` at `x > 0`
/// and the arc of the wall over the north pole, and the same round the south
/// pole. The face holds neither pole: it is the band between the two rings.
/// Before Q1329 the two arcs of each circle were one edge and these were
/// refused `ResultIsNotManifold { count: 4 }`; named apart, the mesh the fuse
/// checks its answer with refused `LoopsTouch` at `(0, 4, 8)` until Q1590,
/// because the band's first cut ran from `(0, 4, 2)` straight up the wall to
/// the north pole, through the corner `(0, 4, 8)` it should have ended at.
///
/// **Every rule answers**, on the same sums as the half bar along `x` against
/// the ball at the origin: the union `96π + 500π/3 − 196π/3 = 592π/3`, the
/// intersection half of the `392π/3` the ball and the bar share, the half less
/// the ball two ends of `(96π − 196π/3)/2 = 46π/3`, and the ball less the half
/// `500π/3 − 196π/3 = 304π/3`.
#[test]
fn a_half_bar_on_a_balls_axis_through_its_poles_answers_every_rule() {
    let gives = what_each_rule_gives(|model| {
        let bar = bar_up_z(model);
        [
            cut(model, bar, [0.0, -20.0, -20.0], [20.0, 20.0, 20.0]),
            ball(model, [0.0, 0.0, 5.0]),
        ]
    });
    let wanted = [
        Gives::Holds(592.0 * PI / 3.0),
        Gives::Holds(592.0 * PI / 3.0),
        Gives::Holds(WHAT_A_HALF_SHARES),
        Gives::Holds(WHAT_A_HALF_SHARES),
        Gives::HoldsTwo(46.0 * PI / 3.0, 46.0 * PI / 3.0),
        Gives::Holds(304.0 * PI / 3.0),
    ];
    assert!(
        gives
            .iter()
            .zip(&wanted)
            .all(|(answer, wanted)| wanted.is(answer)),
        "{gives:?}"
    );
}

/// The ball of radius 5 at `(0, 0, 5)`, `500π/3`, less the block `x ≥ 0,
/// z ≥ 8`, and less that block and `x ≥ 0, z ≤ 2` in one fuse.
///
/// Each block takes half of a cap of height 2, and a cap of height `h` on a
/// ball of radius `R` is `πh²(3R − h)/3`, `52π/3` here. The ball less the top
/// block, one ring over the north pole round a face that holds the south one,
/// is `500π/3 − 26π/3 = 474π/3`. The ball less both is `500π/3 − 52π/3 =
/// 448π/3`, and its skin is the band of the half bar's test above with no two
/// arcs of one crossing in it: refused `LoopsTouch` at `(0, 4, 8)` the same
/// way until Q1590.
#[test]
fn a_ball_less_half_its_cap_at_each_pole_answers() {
    let top = ([0.0, -20.0, 8.0], [20.0, 20.0, 20.0]);
    let bottom = ([0.0, -20.0, -20.0], [20.0, 20.0, 2.0]);
    for (blocks, wanted) in [
        (&[top][..], Gives::Holds(474.0 * PI / 3.0)),
        (&[top, bottom][..], Gives::Holds(448.0 * PI / 3.0)),
    ] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let ball = ball(&mut model, [0.0, 0.0, 5.0]);
        let blocks: Vec<SolidKey> = blocks
            .iter()
            .map(|&(low, high)| block(&mut model, low, high))
            .collect();
        let gives = what_one_fuse_gives(&mut model, &[ball], &blocks, Rule::Difference);
        assert!(wanted.is(&gives), "{gives:?}");
    }
}

/// The bar of the first test, seamed up `+x`, holds the ball of radius 4 at
/// `(0, 0, 5)`, which touches its skin round the circle `z = 5`. Along that
/// circle the two gradients line up and nothing tells two arcs of it apart, so
/// `which_arc_of_a_circle_this_is` names a ball's arcs against a pipe on the
/// word that no fuse puts two corners on it.
///
/// **The whole bar answers every rule**: its seam puts one corner on the
/// circle, the bar holds the ball, so the union is the bar, `192π`, the
/// intersection the ball, `256π/3`, the bar less the ball `320π/3`, and the
/// ball less the bar nothing. **A wall that puts two is refused under every
/// rule both ways before an edge is named**, ordering the points on the circle
/// along a touch: the bar's half `x ≥ 0`, the bar seamed up `−x` cut to
/// `x ≥ 2`, off the axis, and the ball's half `x ≥ 0` against the whole bar.
#[test]
fn a_ball_a_bar_holds_round_one_circle_is_refused_where_a_wall_crosses_it() {
    let whole = what_each_rule_gives(|model| [bar(model, 0), snug(model)]);
    let wanted = [
        Gives::Holds(192.0 * PI),
        Gives::Holds(192.0 * PI),
        Gives::Holds(256.0 * PI / 3.0),
        Gives::Holds(256.0 * PI / 3.0),
        Gives::Holds(320.0 * PI / 3.0),
        Gives::Nothing,
    ];
    assert!(
        whole
            .iter()
            .zip(&wanted)
            .all(|(answer, wanted)| wanted.is(answer)),
        "{whole:?}"
    );
    let walls: [fn(&mut Model) -> [SolidKey; 2]; 3] = [
        |model| {
            let bar = bar(model, 0);
            [
                cut(model, bar, [0.0, -20.0, -20.0], [20.0, 20.0, 20.0]),
                snug(model),
            ]
        },
        |model| {
            let bar = bar(model, 2);
            [
                cut(model, bar, [2.0, -20.0, -20.0], [20.0, 20.0, 20.0]),
                snug(model),
            ]
        },
        |model| {
            let ball = snug(model);
            [
                cut(model, ball, [0.0, -20.0, -20.0], [20.0, 20.0, 20.0]),
                bar(model, 0),
            ]
        },
    ];
    for wall in walls {
        let gives = what_each_rule_gives(wall);
        assert!(
            gives
                .iter()
                .all(|answer| matches!(answer, Err(Error::ACrossingThisCannotBeOrderedAlong))),
            "{gives:?}"
        );
    }
}

/// The ball of radius 3 at `(0, 3, 5)` reaches into the side of a bar of
/// radius 2 up `z`, and the two cross in one loop. Three planes put two corners
/// on it: the bar's cap `z = 5` where the bar runs from `z = −1` to `z = 5`,
/// at `(±4√2/3, 2/3, 5)`; and the wall `x = 0` through the bar's axis and the
/// ball's centre, at `(0, 2, 5 ± 2√2)`, with the bar to `z = 11` cut to `x ≥ 0`
/// against the whole ball, or the ball cut to `x ≥ 0` against the whole bar.
///
/// Until Q1591 each was refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`,
/// before the fuse had a corner. `rcad-geom`'s `three_surfaces.rs` places those
/// corners now, and every rule both ways round is refused one step further on,
/// `ACrossingThisCannotBeOrderedAlong`, the refusal the same ball and bar get
/// with no plane through the loop (Q1593). Of the places that refusal is
/// raised, the one a loop of a ball and a pipe reaches is `fuse/arrange.rs`'s
/// `how_a_crossing_runs`, which orders the two loops a ball crosses a pipe it
/// holds in and not the one a ball reaching into a pipe's side leaves. So no
/// arc of this loop reaches `which_arc_of_a_circle_this_is` yet.
#[test]
fn a_plane_through_the_loop_a_ball_off_a_bars_axis_crosses_it_in_is_refused_ordering_the_loop() {
    let pairs: [fn(&mut Model) -> [SolidKey; 2]; 3] = [
        |model| {
            let rod = model
                .add_round_bar(2.0, 6.0, &at(0.0, 0.0, -1.0))
                .expect("a bar of radius two");
            [
                rod,
                model.add_ball(3.0, &at(0.0, 3.0, 5.0)).expect("a ball"),
            ]
        },
        |model| {
            let rod = model
                .add_round_bar(2.0, 12.0, &at(0.0, 0.0, -1.0))
                .expect("a bar of radius two");
            [
                cut(model, rod, [0.0, -20.0, -20.0], [20.0, 20.0, 20.0]),
                model.add_ball(3.0, &at(0.0, 3.0, 5.0)).expect("a ball"),
            ]
        },
        |model| {
            let rod = model
                .add_round_bar(2.0, 12.0, &at(0.0, 0.0, -1.0))
                .expect("a bar of radius two");
            let ball = model.add_ball(3.0, &at(0.0, 3.0, 5.0)).expect("a ball");
            [
                rod,
                cut(model, ball, [0.0, -20.0, -20.0], [20.0, 20.0, 20.0]),
            ]
        },
    ];
    for pair in pairs {
        let gives = what_each_rule_gives(pair);
        assert!(
            gives
                .iter()
                .all(|answer| matches!(answer, Err(Error::ACrossingThisCannotBeOrderedAlong))),
            "{gives:?}"
        );
    }
}

/// A bar of radius 1 along `x` at `z = 5`, from `x = −10` to `x = 10` and cut
/// to `y ≥ 0`, `10π`, through a bar of radius 4 up `z` from `z = 0` to
/// `z = 10`, `160π`: the fat bar swallows the thin one's whole section, so the
/// two cross in two loops, and the wall `y = 0` through both axes puts two
/// corners on each, at `(±4, 0, 4)` and `(±4, 0, 6)`. Between each two corners
/// run three curves: the loop's arc over the wall, its arc under the wall,
/// which the fat bar's whole skin holds, and the line the wall cuts that skin
/// along.
///
/// Until Q1591 every rule was refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`
/// before the fuse had a corner. With the corners placed, nothing named two
/// pipes' arcs apart, so a loop's two arcs were one edge named by its corners:
/// six faces along it for the unions and the fat bar less the half, four for
/// the intersections and the half less the fat bar, and every rule both ways
/// round was refused `ResultIsNotManifold` in `fuse/assemble.rs`'s
/// `darts_along_each_edge`. `which_arc_of_a_circle_this_is` names the two arcs
/// apart since Q1643. Every rule was then refused
/// `CannotNameWhichPieceOfACrossingThisIs` drawing an arc, because
/// `rcad-geom`'s `walk` picked the loop an arc is on by the wall's `y ≥ 0`
/// alone, which holds the place it starts from on both loops, and handed back
/// the whole loop for a run whose two ends differ. It picks the loop by the
/// run's end since Q1659 and keeps the stretch between the two ends.
///
/// **Every rule answers.** What the two share,
/// `I = 4∫₀¹ √((16 − y²)(1 − y²)) dy`, has no closed form, so the intersection
/// is held to [`what_a_half_thin_bar_shares_with_a_fat_one`] and every other
/// answer to the intersection as measured: the union `10π + 160π − I`, one
/// solid of nine faces; the fat bar less the half `160π − I`, one solid of
/// five with a channel through it; and the half less the fat bar two ends of
/// `(10π − I)/2`, four faces each.
#[test]
fn a_wall_through_the_axes_of_a_thin_bar_through_a_fat_one_answers_every_rule() {
    let gives = what_each_rule_gives(|model| {
        let thin = model
            .add_round_bar(
                1.0,
                20.0,
                &at(-10.0, 0.0, 5.0)
                    .compose(&Isometry::quarter_turns(&Axis::y(), 1).expect("y is an axis")),
            )
            .expect("a bar of radius one");
        let fat = model
            .add_round_bar(4.0, 10.0, &Isometry::identity())
            .expect("a bar of radius four");
        [
            fat,
            cut(model, thin, [-20.0, 0.0, -20.0], [20.0, 20.0, 20.0]),
        ]
    });
    let solids = |answer: &Result<Vec<Measured>, Error>| -> Vec<Measured> {
        answer
            .as_ref()
            .unwrap_or_else(|why| panic!("every rule answers: {why:?} in {gives:?}"))
            .clone()
    };
    let [shared] = solids(&gives[2])[..] else {
        panic!("one solid shared: {gives:?}");
    };
    assert!(
        (shared.volume - what_a_half_thin_bar_shares_with_a_fat_one()).abs() <= shared.error,
        "{gives:?}"
    );
    let (half, fat) = (10.0 * PI, 160.0 * PI);
    let one_end = (half - shared.volume) / 2.0;
    let wanted: [&[(f64, usize)]; 6] = [
        &[(half + fat - shared.volume, 9)],
        &[(half + fat - shared.volume, 9)],
        &[(shared.volume, 4)],
        &[(shared.volume, 4)],
        &[(fat - shared.volume, 5)],
        &[(one_end, 4), (one_end, 4)],
    ];
    for (answer, wanted) in gives.iter().zip(wanted) {
        let solids = solids(answer);
        assert_eq!(
            solids.iter().map(|solid| solid.faces).collect::<Vec<_>>(),
            wanted.iter().map(|&(_, faces)| faces).collect::<Vec<_>>(),
            "{gives:?}"
        );
        for (solid, &(volume, _)) in solids.iter().zip(wanted) {
            assert!(
                (solid.volume - volume).abs() <= solid.error + shared.error,
                "{gives:?}"
            );
        }
    }
}

/// `4∫₀¹ √((16 − y²)(1 − y²)) dy`, the part of the half bar of radius 1 above
/// inside the bar of radius 4, worked by the midpoint rule in `t` with
/// `y = sin t`. The integrand `4cos²t·√(16 − sin²t)` is smooth and its odd
/// derivatives vanish at both ends of `[0, π/2]`, so the rule settles fast:
/// 2000 steps give `12.46741344968615`, where the rule in `y` itself, whose
/// integrand has a square root's corner at `y = 1`, is still `1.5e-8` off it
/// at 200 000 steps.
fn what_a_half_thin_bar_shares_with_a_fat_one() -> f64 {
    let steps = 2000_u32;
    let step = PI / 2.0 / f64::from(steps);
    (0..steps)
        .map(|at| {
            let t = (f64::from(at) + 0.5) * step;
            t.cos().powi(2) * (16.0 - t.sin().powi(2)).sqrt()
        })
        .sum::<f64>()
        * step
        * 4.0
}

/// What one fuse is pinned to give.
#[derive(Clone, Copy)]
enum Gives {
    /// One solid of this volume.
    Holds(f64),
    /// Two solids of these volumes, the smaller first.
    HoldsTwo(f64, f64),
    /// No solid at all.
    Nothing,
}

impl Gives {
    fn is(&self, answer: &Result<Vec<Measured>, Error>) -> bool {
        let near = |solid: Measured, wanted: f64| (solid.volume - wanted).abs() <= solid.error;
        match (self, answer) {
            (Gives::Holds(wanted), Ok(volumes)) => {
                matches!(volumes[..], [one] if near(one, *wanted))
            }
            (Gives::HoldsTwo(smaller, larger), Ok(volumes)) => {
                matches!(volumes[..], [one, other] if near(one, *smaller) && near(other, *larger))
            }
            (Gives::Nothing, Ok(volumes)) => volumes.is_empty(),
            _ => false,
        }
    }
}

/// One solid of an answer.
#[derive(Clone, Copy, Debug)]
struct Measured {
    volume: f64,
    /// How far off [`Measured::volume`] may be.
    error: f64,
    faces: usize,
}

/// What every rule, both ways round, comes back with, in the order
/// [`every_rule_both_ways`] gives them: `Ok` with each solid, the smallest
/// first, or the refusal.
fn what_each_rule_gives(
    build: impl Fn(&mut Model) -> [SolidKey; 2],
) -> Vec<Result<Vec<Measured>, Error>> {
    every_rule_both_ways()
        .map(|(rule, swapped)| {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let [first, second] = build(&mut model);
            let [one, other] = if swapped {
                [second, first]
            } else {
                [first, second]
            };
            what_one_fuse_gives(&mut model, &[one], &[other], rule)
        })
        .collect()
}

/// What `one` fused with `other` under `rule` comes back with: `Ok` with each
/// solid, the smallest first, once the model validates, or the refusal.
fn what_one_fuse_gives(
    model: &mut Model,
    one: &[SolidKey],
    other: &[SolidKey],
    rule: Rule,
) -> Result<Vec<Measured>, Error> {
    let answer = model.fuse(
        one,
        other,
        rule,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )?;
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("an answer validates");
    let mut solids: Vec<Measured> = answer
        .solids()
        .iter()
        .map(|&solid| {
            let solid = model.solid(solid).expect("this model made the key");
            let measured = solid
                .measure(
                    Deflection::of(0.01).expect("a fine mesh"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("an answer measures");
            Measured {
                volume: measured.volume(),
                error: measured.volume_error(),
                faces: solid.faces().count(),
            }
        })
        .collect();
    solids.sort_by(|one, other| one.volume.total_cmp(&other.volume));
    Ok(solids)
}

fn every_rule_both_ways() -> impl Iterator<Item = (Rule, bool)> {
    [Rule::Union, Rule::Intersection, Rule::Difference]
        .into_iter()
        .flat_map(|rule| [false, true].map(move |swapped| (rule, swapped)))
}

/// `solid` cut down to what lies inside the block from `low` to `high`.
fn cut(model: &mut Model, solid: SolidKey, low: [f64; 3], high: [f64; 3]) -> SolidKey {
    let block = block(model, low, high);
    let answer = model
        .fuse(
            &[solid],
            &[block],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("cutting: {why:?}"))
        .solids()
        .to_vec();
    let [solid] = answer[..] else {
        panic!("one solid, got {answer:?}");
    };
    solid
}

fn block(model: &mut Model, low: [f64; 3], high: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [low[0], low[1]],
        [high[0], low[1]],
        [high[0], high[1]],
        [low[0], high[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(&rectangle, high[2] - low[2], &at(0.0, 0.0, low[2]))
        .expect("a block")
}

/// The bar of radius 4 from `z = −1` to `z = 11`, seamed up `−x`, which a cut
/// to `x ≥ 0` takes away.
fn bar_up_z(model: &mut Model) -> SolidKey {
    bar(model, 2)
}

/// The bar of radius 4 from `z = −1` to `z = 11`, its seam turned this many
/// quarter turns about `z` from `+x`.
fn bar(model: &mut Model, seam: i32) -> SolidKey {
    model
        .add_round_bar(
            4.0,
            12.0,
            &at(0.0, 0.0, -1.0)
                .compose(&Isometry::quarter_turns(&Axis::z(), seam).expect("z is an axis")),
        )
        .expect("a bar of radius four")
}

/// The bar of radius 4 from `x = −6` to `x = 6`.
fn bar_along_x(model: &mut Model) -> SolidKey {
    model
        .add_round_bar(
            4.0,
            12.0,
            &at(-6.0, 0.0, 0.0)
                .compose(&Isometry::quarter_turns(&Axis::y(), 1).expect("y is an axis")),
        )
        .expect("a bar of radius four")
}

/// The ball of radius 5 about `centre`.
fn ball(model: &mut Model, centre: [f64; 3]) -> SolidKey {
    model
        .add_ball(5.0, &at(centre[0], centre[1], centre[2]))
        .expect("a ball of radius five")
}

/// The ball of radius 4 at `(0, 0, 5)`, which [`bar`] holds.
fn snug(model: &mut Model) -> SolidKey {
    model
        .add_ball(4.0, &at(0.0, 0.0, 5.0))
        .expect("a ball of radius four")
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
}
