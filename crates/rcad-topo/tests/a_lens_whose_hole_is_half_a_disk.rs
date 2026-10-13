//! A flat face whose outline is two arcs meeting at two corners, with a hole
//! that is half a disk whose straight side lies on the line through those two
//! corners. Every corner of both rings is on one line, so neither ring winds
//! round any area, and `rings_round` in `fuse/assemble.rs` has to ask
//! `holds_the_other_rings` which one is the outline (§191.5). Q576 built the
//! same question with rings of arcs only; this is the family with a straight
//! side, which it argued about and did not build (Q598).
//!
//! Every shape here is whole numbers. The outline comes from bars of radius 5
//! standing on `z = 0`, four tall, at `(−3, 0)` and `(3, 0)`, whose circles
//! cross at `(0, ±4)`. The hole comes from a half bar of radius 1: a round bar
//! less a block over `x > 0`, so it bulges towards `−x` and its flat side is on
//! `x = 0`, the line through `(0, ±4)`. It stands from `z = 1` to `z = 6`, so
//! it rises out of the cap at `z = 4`.
//!
//! **What answers.** A bar at `(−3, 0)` fused with the half bar at the origin
//! answers by every rule, both ways round. The fuse runs the line `x = 0` right
//! across the bar's cap, so the cap is cut at `(0, ±4)` and joined back up: its
//! outline is two arcs of the one pipe meeting there, and its hole is the half
//! disk from `(0, −1)` to `(0, 1)`. That is this family with both arcs on one
//! surface.
//!
//! Run under gdb on base `04a469ad`, with a breakpoint on the first line of
//! `holds_the_other_rings`, the three fuses that keep the cap each call it
//! twice: the ring along the one pipe answers that it holds the other, and the
//! half disk, through the count of turns round a corner, that it holds nothing.
//!
//! With the two arcs on two pipes — a real lens, where the bars at `(−3, 0)`
//! and `(3, 0)` meet — the plane `x = 0` used to cross each pipe along the
//! vertical line through a corner of the lens, where the two walls meet, and
//! every route was refused in the arrangement before any face was joined:
//! `TwoCurvesTouchWithoutCrossing` on a wall bounded by those lines, §38.2's
//! two curves along one ruling. Since queue row Q930's
//! `fold_the_straight_cuts_along_one_line` folds such a pair into one
//! crossing drawn before it, all four routes to the lens now answer, validate
//! and hold their closed forms under every rule and both ways round.
//!
//! **What still refuses.** A half bar whose corner is the lens's own corner —
//! `(0, −4)`, where its flat side runs from that corner to `(0, −2)` — puts
//! four surfaces round the one vertical line through it. Built from one bar
//! and the half bar directly, every fuse answers. The fold answers the union,
//! the intersection (both ways round), and the half bar less the bar. The bar
//! less the half bar was refused `ResultIsNotManifold { count: 4 }` until
//! queue row Q1901, and answers `98.5π` since: the pocket it cuts meets the
//! bar's rim at the lens's own corner, so the answer is one solid touching
//! itself along the vertical line through `(0, −4)` from `z = 1` to the cap.
//! This file called that refusal a limit. By §16.10 it was a gap, since the
//! answer exists: a solid touching itself along a line is one
//! (`docs/decisions.md` §39.4).
//!
//! Built by uniting or overlapping the two bars first, the fuse meets the half
//! bar across the one line where the lens's two walls and the half bar's pipe
//! all run, three curved surfaces, and
//! `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` refused every one of those
//! twelve fuses until Q1284 read three pipes up `z` as the plane two of them
//! differ by and two pipes. Now every one answers but **the union of the bars
//! united and the half bar, which panics, both ways round**, in `rings_round`.
//! The bars less the half bar is `200π − 4L − 1.5π`, for the lens's area `L`,
//! again one solid touching itself along that line. There the line is an edge
//! of both inputs: the bars' two walls meet along it, and so do the half bar's
//! flat side and its pipe. So four input faces lie round it, and it was
//! refused `ResultIsNotManifold { count: 4 }` until queue row Q1913, which
//! parts such a line where its four faces are one shell without it
//! (`docs/decisions.md` §39.4).
//!
//! A half bar of radius 4 at the origin, whose flat side is the whole chord,
//! puts both of its corners on the lens's own. Against one bar alone every
//! fuse answers, the bar less the half bar since Q1901: `76π`, one solid
//! touching itself along the vertical lines through `(0, ±4)`. Against the two
//! bars united or where they overlap it was refused the same way as the half
//! bar above, and since Q1284 answers, the bars united less the half bar,
//! `176π − 4L`, since Q1913, with the same four input faces round each of
//! those lines. The two D's side by side without the half bar answer too.
//! Q625 found the half bar against one bar, and the two D's, refused
//! `TwoCurvesTouchWithoutCrossing` before Q930 and did not pin them.
//!
//! **Which ring is the outline, run rather than read.** On base `90c2e6a9`,
//! with a print in `rings_round` and `holds_the_other_rings` in
//! `fuse/assemble.rs` that was not kept, ten fuses of the lens's four routes
//! keep a cap with the half disk in it: every union both ways round, and the
//! bars united or the lens less the half bar. Each asks
//! `holds_the_other_rings` of both rings. The half disk answers that it holds
//! nothing: the outline's corner `(0, −4)` or `(0, 4)` is on the line through
//! its chord but past its ends, and its arc winds round it no times. The
//! outline, two arcs on two pipes, answers `None`: both of the half disk's
//! corners lie on its chord inside both pipes, and such a corner is passed
//! over. So `rings_round` takes the outline as the one ring nothing could
//! place. On one pipe, in the first union of the third route, the outline
//! answers that it holds the other, as under gdb above. A second ring that
//! nothing could place is, since Q1253, refused
//! `Error::CannotPlaceTheRingsOfAJoinedFlatFace` rather than reaching a panic
//! in `rings_round`; only rings that each answer they hold none of the
//! others still reach a panic there. The second is what
//! the union of the bars united and the half bar at the lens corner reaches
//! (`a_half_bar_whose_corner_is_the_lens_corner_mostly_answers`), on base
//! `1c002fe1` with Q1284's change, and it was not traced.
//!
//! `the_rings_of_the_cap` reads the outline first, as the face stores it, so
//! the pins hold `rings_round` to that choice. Nothing else here would notice
//! a swap: with `rings_round` made to take the other ring, every fuse still
//! validated and held its volume, and only those pins failed.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// A way to build the two solids a fuse is asked about, what it is called, and
/// the volume and solid count every fuse of them is expected to give, for
/// `[union, intersection, first less second, second less first]`.
type AnswerRoute = (&'static str, fn(&mut Model) -> [SolidKey; 2], [Expect; 4]);

/// The rings of a cap kept with the half disk of radius 1 in it, as
/// [`the_rings_of_the_cap`] reads them: the outline of two arcs from
/// `(0, −4)` to `(0, 4)`, then the hole from `(0, −1)` to `(0, 1)`.
const THE_OUTLINE_THEN_THE_HALF_DISK: &[&[[f64; 2]]] =
    &[&[[0.0, -4.0], [0.0, 4.0]], &[[0.0, -1.0], [0.0, 1.0]]];

/// A bar of radius 5 at `(−3, 0)` and the half bar rising out of its cap, by
/// every rule and both ways round.
///
/// - The union is the bar and the half bar's two units above the cap:
///   `100π + π = 101π`.
/// - The intersection is the half bar from `z = 1` to the cap: `1.5π`.
/// - The bar less the half bar has a pocket three deep: `98.5π`.
/// - The half bar less the bar is its two units above the cap: `π`.
///
/// Where the answer keeps the bar's cap, the cap is the face this file is
/// about: an outline with corners at `(0, ±4)` and a hole with corners at
/// `(0, ±1)`, four corners on one line.
#[test]
fn a_half_bar_rising_out_of_a_bars_cap_is_fused_by_every_rule_either_way_round() {
    for (rule, bar_first, volume, keeps_the_cap) in [
        (Rule::Union, true, 101.0 * PI, true),
        (Rule::Union, false, 101.0 * PI, true),
        (Rule::Intersection, true, 1.5 * PI, false),
        (Rule::Intersection, false, 1.5 * PI, false),
        (Rule::Difference, true, 98.5 * PI, true),
        (Rule::Difference, false, PI, false),
    ] {
        let asked = format!("{rule:?}, the bar first {bar_first}");
        let mut model = a_model();
        let bar = a_bar_at(&mut model, -3.0);
        let half = a_half_bar_bulging_left(&mut model, 1.0, 0.0);
        let [one, other] = if bar_first { [bar, half] } else { [half, bar] };
        let answer =
            fuse(&mut model, one, other, rule).unwrap_or_else(|why| panic!("{asked}: {why}"));
        let [solid] = answer[..] else {
            panic!("{asked} is one solid, not {answer:?}");
        };
        assert_holds(&model, solid, volume, &asked);
        if keeps_the_cap {
            assert_eq!(
                the_rings_of_the_cap(&model, solid),
                THE_OUTLINE_THEN_THE_HALF_DISK,
                "{asked}: the corners of the cap's outline, then of its hole"
            );
        }
    }
}

/// A lens of two pipes with the half bar standing on its chord, built four
/// ways, answers every rule and both ways round, validates, and holds its
/// closed form.
///
/// `L` is the lens — the area where the two circles of radius 5 overlap, their
/// centres `6` apart: `L = 50·acos(0.6) − 24`. `U = 4·(50π − L)` is the volume
/// of the two bars united (each bar `100π`, less the lens counted twice).
/// `D = 4·(25π − (25·acos(0.6) − 12))` is the volume of one bar cut by
/// `x = 0`, keeping the larger piece that holds its own axis.
///
/// - The bars at `(−3, 0)` and `(3, 0)` united first, then fused with the half
///   bar: union `U + π` (the half bar's part above the cap sticks out past the
///   union), intersection `1.5π` (the half bar sits entirely inside the
///   union's footprint up to the cap), the union less the half bar `U − 1.5π`,
///   the half bar less the union `π`.
/// - Where the two bars overlap (the lens itself, `4L`), fused with the half
///   bar: union `4L + π`, intersection `1.5π` (the half bar sits entirely
///   inside the lens's footprint too), the lens less the half bar `4L − 1.5π`,
///   the half bar less the lens `π`.
/// - The bar at `(−3, 0)` united with the half bar first (`101π`, pinned
///   below), then fused with the bar at `(3, 0)`: union `U + π` again, since
///   union does not care which pair meets first; intersection `4L`, since the
///   half bar's own share is already inside the lens; the first union less the
///   second bar leaves **two** solids, the bar less the lens (`100π − 4L`) and
///   the half bar's part above the cap (`π`), disjoint since that part sits
///   outside the first bar entirely — total `101π − 4L`; the second bar less
///   the first union is `100π − 4L`, one solid.
/// - The bars cut down to the D on each side of `x = 0`, the left one with
///   the half bar bitten out of its flat side (`D − 1.5π`, since the whole
///   half bar up to the cap sits inside that D): union `2D − 1.5π` (one
///   solid — `2D = U`, the two D's reconstruct the bars' union exactly), no
///   intersection (the two D's meet only along `x = 0`, which has no volume),
///   the bitten D less the other D is itself, `D − 1.5π`, and the other way
///   round is `D`.
///
/// Every union keeps a cap with the half disk in it, and so does the union
/// of the bars, or the lens, less the half bar: the outline is two arcs on two
/// pipes from `(0, −4)` to `(0, 4)` and the hole is the half disk from
/// `(0, −1)` to `(0, 1)`. The outline comes first as the face stores it, which
/// is the ring `rings_round` took for it.
#[test]
fn a_lens_of_two_pipes_with_a_half_bar_on_its_chord_answers_by_every_rule() {
    let lens = the_lens();
    let bars_united = 4.0 * (50.0 * PI - lens);
    let d = the_volume_of_a_d();
    let routes: [AnswerRoute; 4] = [
        (
            "the two bars united, then the half bar",
            |model| {
                let bars = two_bars_united(model);
                [bars, a_half_bar_bulging_left(model, 1.0, 0.0)]
            },
            [
                Expect::one(bars_united + PI).with_the_cap(THE_OUTLINE_THEN_THE_HALF_DISK),
                Expect::one(1.5 * PI),
                Expect::one(bars_united - 1.5 * PI).with_the_cap(THE_OUTLINE_THEN_THE_HALF_DISK),
                Expect::one(PI),
            ],
        ),
        (
            "where the two bars overlap, then the half bar",
            |model| {
                let lens = where_two_bars_overlap(model);
                [lens, a_half_bar_bulging_left(model, 1.0, 0.0)]
            },
            [
                Expect::one(4.0 * lens + PI).with_the_cap(THE_OUTLINE_THEN_THE_HALF_DISK),
                Expect::one(1.5 * PI),
                Expect::one(4.0 * lens - 1.5 * PI).with_the_cap(THE_OUTLINE_THEN_THE_HALF_DISK),
                Expect::one(PI),
            ],
        ),
        (
            "one bar united with the half bar, then the other bar",
            |model| {
                let bar = a_bar_at(model, -3.0);
                let half = a_half_bar_bulging_left(model, 1.0, 0.0);
                let [both] =
                    fuse(model, bar, half, Rule::Union).expect("one bar and the half bar")[..]
                else {
                    panic!("one bar and the half bar are one solid");
                };
                [both, a_bar_at(model, 3.0)]
            },
            [
                Expect::one(bars_united + PI).with_the_cap(THE_OUTLINE_THEN_THE_HALF_DISK),
                Expect::one(4.0 * lens),
                Expect {
                    volume: 101.0 * PI - 4.0 * lens,
                    solids: 2,
                    cap: None,
                },
                Expect::one(100.0 * PI - 4.0 * lens),
            ],
        ),
        (
            "the D on each side, the left one bitten",
            |model| {
                let left = a_d_of_the_bar_at(model, -3.0);
                let bite = a_half_bar_bulging_left(model, 1.0, 0.0);
                let [bitten] = fuse(model, left, bite, Rule::Difference).expect("a bitten D")[..]
                else {
                    panic!("a bitten D is one solid");
                };
                [bitten, a_d_of_the_bar_at(model, 3.0)]
            },
            [
                Expect::one(2.0 * d - 1.5 * PI).with_the_cap(THE_OUTLINE_THEN_THE_HALF_DISK),
                Expect::nothing(),
                Expect::one(d - 1.5 * PI),
                Expect::one(d),
            ],
        ),
    ];
    for (route, build, expect) in routes {
        assert_every_fuse_answers(route, build, expect);
    }
}

/// The two D's side by side without the half bar, which Q625 found refused
/// `TwoCurvesTouchWithoutCrossing` before Q930 and did not pin, answer every
/// rule both ways round.
///
/// The union is `2D`, one solid whose cap is the two D's caps joined across
/// `x = 0` into one ring of two arcs from `(0, −4)` to `(0, 4)`, the outline
/// by being the only ring. The two meet only on `x = 0`, so there is no
/// intersection and each less the other is itself, `D`.
#[test]
fn the_two_ds_side_by_side_answer_by_every_rule() {
    let d = the_volume_of_a_d();
    assert_every_fuse_answers(
        "the D on each side",
        |model| {
            let left = a_d_of_the_bar_at(model, -3.0);
            [left, a_d_of_the_bar_at(model, 3.0)]
        },
        [
            Expect::one(2.0 * d).with_the_cap(&[&[[0.0, -4.0], [0.0, 4.0]]]),
            Expect::nothing(),
            Expect::one(d),
            Expect::one(d),
        ],
    );
}

/// A half bar of radius 4 at the origin, whose flat side is the whole chord
/// from `(0, −4)` to `(0, 4)`. Q625 found it refused
/// `TwoCurvesTouchWithoutCrossing` against one bar before Q930 and did not
/// pin it.
///
/// The bar at `(−3, 0)` holds the half disk and its circle runs through both
/// of its corners. The union is the bar and the half bar's two units above
/// the cap, `100π + 16π`; the intersection both ways round is the half bar's
/// three units up to the cap, `24π`; the half bar less the bar is `16π`. The
/// bar less the half bar is `100π − 24π = 76π`. What is left of the bar's
/// section holds `(0.1, 3.9)` on one side of `x = 0` and `(−0.1, 4)` on the
/// other, and near `(0, 4)` it cannot cross that line: just below the corner
/// the line is the half disk's flat side, and just above it is outside the
/// bar. So the section is two pieces meeting at a point there, and at
/// `(0, −4)` the same, and the solid touches itself along the vertical lines
/// through `(0, ±4)` from `z = 1` to the cap. It was refused
/// `ResultIsNotManifold { count: 4 }` until queue row Q1901, and this doc
/// called that a limit, since the answer is not a manifold. By §16.10 it was
/// a gap, since the answer exists: a solid touching itself along a line is
/// one (`docs/decisions.md` §39.4).
///
/// The half disk holds the part of the bar at `(3, 0)` on its side of
/// `x = 0`, half the lens, so the two share `1.5L` from `z = 1` to the cap:
/// union `140π − 1.5L`, intersection `1.5L`, the bar less the half bar
/// `100π − 1.5L`, and the half bar less the bar `40π − 1.5L`.
///
/// Against the two bars united or where they overlap, both its corners are
/// the lens's own, and the lens's two walls and the half bar's pipe all run
/// down the vertical lines through `(0, ±4)`. Three pipes up `z` bend the same
/// way, so since Q1284 their meet is the plane the lens's two walls differ by,
/// `x = 0`, and two pipes, and they share those whole lines. Every fuse then
/// answers. The half bar's half disk is inside the bar at `(−3, 0)`, so
/// against the two bars united, `U = 200π − 4L`, they share `24π` as against
/// that one bar: union `216π − 4L`, intersection `24π`, and the half bar less
/// the bars `16π`. The bars less the half bar is `176π − 4L`, one solid
/// touching itself at `(0, ±4)` as against the one bar. Each of those lines is
/// an edge of both inputs, where the bars' two walls meet and where the half
/// bar's flat side meets its pipe, so four input faces lie round it; it was
/// refused `ResultIsNotManifold { count: 4 }` until queue row Q1913
/// (`docs/decisions.md` §39.4). Where the two bars overlap, the lens `4L`,
/// they share `1.5L` as
/// against the bar at `(3, 0)`: union `40π + 2.5L`, intersection `1.5L`, the
/// lens less the half bar `2.5L`, and the half bar less the lens
/// `40π − 1.5L`.
#[test]
fn a_half_bar_whose_flat_side_is_the_whole_chord_answers_against_either_bar_alone() {
    assert_each_fuse(
        "the bar at (−3, 0) and the half bar of radius 4",
        |model| {
            let bar = a_bar_at(model, -3.0);
            [bar, a_half_bar_bulging_left(model, 4.0, 0.0)]
        },
        &[
            (Rule::Union, true, 116.0 * PI),
            (Rule::Union, false, 116.0 * PI),
            (Rule::Intersection, true, 24.0 * PI),
            (Rule::Intersection, false, 24.0 * PI),
            (Rule::Difference, true, 76.0 * PI),
            (Rule::Difference, false, 16.0 * PI),
        ],
    );
    let shared = 1.5 * the_lens();
    assert_every_fuse_answers(
        "the bar at (3, 0) and the half bar of radius 4",
        |model| {
            let bar = a_bar_at(model, 3.0);
            [bar, a_half_bar_bulging_left(model, 4.0, 0.0)]
        },
        [
            Expect::one(140.0 * PI - shared),
            Expect::one(shared),
            Expect::one(100.0 * PI - shared),
            Expect::one(40.0 * PI - shared),
        ],
    );
    let the_bars = 200.0 * PI - 4.0 * the_lens();
    assert_each_fuse(
        "the two bars united, then the half bar of radius 4",
        |model| {
            let bars = two_bars_united(model);
            [bars, a_half_bar_bulging_left(model, 4.0, 0.0)]
        },
        &[
            (Rule::Union, true, the_bars + 16.0 * PI),
            (Rule::Union, false, the_bars + 16.0 * PI),
            (Rule::Intersection, true, 24.0 * PI),
            (Rule::Intersection, false, 24.0 * PI),
            (Rule::Difference, true, the_bars - 24.0 * PI),
            (Rule::Difference, false, 16.0 * PI),
        ],
    );
    assert_every_fuse_answers(
        "where the two bars overlap, then the half bar of radius 4",
        |model| {
            let lens = where_two_bars_overlap(model);
            [lens, a_half_bar_bulging_left(model, 4.0, 0.0)]
        },
        [
            Expect::one(40.0 * PI + 2.5 * the_lens()),
            Expect::one(shared),
            Expect::one(2.5 * the_lens()),
            Expect::one(40.0 * PI - shared),
        ],
    );
}

/// The closed-form volume a fuse is expected to answer, how many solids it
/// comes back as, and, where it is checked, the rings of its cap as
/// [`the_rings_of_the_cap`] reads them.
#[derive(Clone, Copy)]
struct Expect {
    volume: f64,
    solids: usize,
    cap: Option<&'static [&'static [[f64; 2]]]>,
}

impl Expect {
    fn one(volume: f64) -> Self {
        Expect {
            volume,
            solids: 1,
            cap: None,
        }
    }

    fn nothing() -> Self {
        Expect {
            volume: 0.0,
            solids: 0,
            cap: None,
        }
    }

    fn with_the_cap(self, rings: &'static [&'static [[f64; 2]]]) -> Self {
        Expect {
            cap: Some(rings),
            ..self
        }
    }
}

/// Runs `build`'s pair through every rule and both ways round, and checks
/// each answers, validates, and holds the volume `expect` gives for
/// `[union, intersection, first less second, second less first]`.
fn assert_every_fuse_answers(
    route: &str,
    build: fn(&mut Model) -> [SolidKey; 2],
    expect: [Expect; 4],
) {
    let [union, intersection, first_minus_second, second_minus_first] = expect;
    for (rule, first_first, expect) in [
        (Rule::Union, true, union),
        (Rule::Union, false, union),
        (Rule::Intersection, true, intersection),
        (Rule::Intersection, false, intersection),
        (Rule::Difference, true, first_minus_second),
        (Rule::Difference, false, second_minus_first),
    ] {
        let asked = format!("{route}, {rule:?}, the first first {first_first}");
        let mut model = a_model();
        let [first, second] = build(&mut model);
        let [one, other] = if first_first {
            [first, second]
        } else {
            [second, first]
        };
        let answer =
            fuse(&mut model, one, other, rule).unwrap_or_else(|why| panic!("{asked}: {why}"));
        assert_eq!(
            answer.len(),
            expect.solids,
            "{asked} came back as {answer:?}"
        );
        assert_total_holds(&model, &answer, expect.volume, &asked);
        let Some(rings) = expect.cap else {
            continue;
        };
        let [solid] = answer[..] else {
            panic!("{asked}: a cap is read off one solid, not {answer:?}");
        };
        assert_eq!(
            the_rings_of_the_cap(&model, solid),
            rings,
            "{asked}: the corners of the cap's outline, then of its holes"
        );
    }
}

/// The half bar moved to `(0, −3)`, so its flat side runs from the lens's own
/// corner `(0, −4)` to `(0, −2)`: four surfaces round the one vertical line
/// through `(0, −4)`.
///
/// Built straight from one bar and the half bar, every fuse answers: the union
/// (`101π`, the bar plus the half bar's part above the cap, since the half
/// bar's footprint is entirely inside the bar's and meets its rim only at the
/// corner), the intersection both ways round (`1.5π`, the half bar's part up
/// to the cap), the half bar less the bar (`π`, its part above the cap,
/// outside the bar entirely), and the bar less the half bar (`98.5π`). The
/// pocket that last one cuts meets the bar's rim at the corner, so it is one
/// solid touching itself along the vertical line through `(0, −4)` from
/// `z = 1` to the cap. It was refused `ResultIsNotManifold { count: 4 }` until
/// queue row Q1901, a gap by §16.10 rather than a limit, since the answer
/// exists: a solid touching itself along a line is one (`docs/decisions.md`
/// §39.4).
///
/// Built by uniting or overlapping the two bars first, the fuse meets the half
/// bar along the one line through `(0, −4)` where the lens's two walls and the
/// half bar's pipe all run. Three pipes up `z` bend the same way, so since
/// Q1284 their meet is the plane the lens's two walls differ by, `x = 0`, and
/// two pipes, and they share that whole line.
///
/// Where the two bars overlap, every fuse answers. The half bar's half disk
/// crosses the bar at `(3, 0)`'s circle at `(0, −4)` and `(−1, −3)`, and the
/// part of it inside that circle, `A = π/4 − 3 + 25·asin(√2/10)`, is the half
/// disk less what lies between its arc and the bar's across the chord from
/// `(0, −4)` to `(−1, −3)`: a segment of the half disk's circle, `π/4 − 1/2`,
/// less one of the bar's, `25/2·(θ − 0.28)` for `θ = 2·asin(√2/10)`. So the
/// lens `4L` and the half bar `2.5π` share `3A`.
///
/// United first, the two bars answer the intersection both ways round,
/// `1.5π`, and the half bar less the bars, `π`, as against the one bar. The
/// bars less the half bar is `200π − 4L − 1.5π`, one solid touching itself
/// along the same line. That line is an edge of both inputs, where the bars'
/// two walls meet and where the half bar's flat side meets its pipe, so four
/// input faces lie round it; it was refused `ResultIsNotManifold { count: 4 }`
/// until queue row Q1913 (`docs/decisions.md` §39.4). **The union panics, both
/// ways round**, in `rings_round` in
/// `fuse/assemble.rs`: every ring of a joined face answers that it holds
/// none of the others, which that function says no face can bring. It would
/// be the bars and the half bar's part above the cap, `201π − 4L`.
#[test]
fn a_half_bar_whose_corner_is_the_lens_corner_mostly_answers() {
    assert_each_fuse(
        "one bar and the half bar at the lens corner",
        |model| {
            let bar = a_bar_at(model, -3.0);
            [bar, a_half_bar_bulging_left(model, 1.0, -3.0)]
        },
        &[
            (Rule::Union, true, 101.0 * PI),
            (Rule::Union, false, 101.0 * PI),
            (Rule::Intersection, true, 1.5 * PI),
            (Rule::Intersection, false, 1.5 * PI),
            (Rule::Difference, true, 98.5 * PI),
            (Rule::Difference, false, PI),
        ],
    );

    let shared = 3.0 * (PI / 4.0 - 3.0 + 25.0 * (2.0_f64.sqrt() / 10.0).asin());
    assert_every_fuse_answers(
        "where the two bars overlap, then the half bar",
        |model| {
            let lens = where_two_bars_overlap(model);
            [lens, a_half_bar_bulging_left(model, 1.0, -3.0)]
        },
        [
            Expect::one(4.0 * the_lens() + 2.5 * PI - shared),
            Expect::one(shared),
            Expect::one(4.0 * the_lens() - shared),
            Expect::one(2.5 * PI - shared),
        ],
    );

    let the_bars_and_the_half_bar = |model: &mut Model| {
        let bars = two_bars_united(model);
        [bars, a_half_bar_bulging_left(model, 1.0, -3.0)]
    };
    assert_each_fuse(
        "the two bars united, then the half bar",
        the_bars_and_the_half_bar,
        &[
            (Rule::Intersection, true, 1.5 * PI),
            (Rule::Intersection, false, 1.5 * PI),
            (
                Rule::Difference,
                true,
                200.0 * PI - 4.0 * the_lens() - 1.5 * PI,
            ),
            (Rule::Difference, false, PI),
        ],
    );
    for first_first in [true, false] {
        let union = std::panic::catch_unwind(|| {
            let mut model = a_model();
            let [first, second] = the_bars_and_the_half_bar(&mut model);
            let [one, other] = if first_first {
                [first, second]
            } else {
                [second, first]
            };
            fuse(&mut model, one, other, Rule::Union)
        });
        let Err(why) = union else {
            panic!("the union, the bars first {first_first}, gives {union:?}; pin it");
        };
        assert_eq!(
            why.downcast_ref::<&str>(),
            Some(
                &"every face joined here brought an outline of its own and joining only takes \
                  edges away, so the joined face has one too, and none of its rings winds with \
                  the plane or bounds a region holding the others"
            ),
            "the bars first {first_first}"
        );
    }
}

/// The pair `build` makes, fused by each rule and way round in `answers`:
/// one solid holding the volume it gives.
fn assert_each_fuse(
    route: &str,
    build: impl Fn(&mut Model) -> [SolidKey; 2],
    answers: &[(Rule, bool, f64)],
) {
    for &(rule, first_first, volume) in answers {
        let asked = format!("{route}, {rule:?}, the first first {first_first}");
        let mut model = a_model();
        let [first, second] = build(&mut model);
        let [one, other] = if first_first {
            [first, second]
        } else {
            [second, first]
        };
        let answer =
            fuse(&mut model, one, other, rule).unwrap_or_else(|why| panic!("{asked}: {why:?}"));
        assert_eq!(answer.len(), 1, "{asked} came back as {answer:?}");
        assert_total_holds(&model, &answer, volume, &asked);
    }
}

/// The first union of the third route above: the bar at `(−3, 0)` with the
/// half bar rising out of its cap, on its own, before the second fuse that
/// refuses. `101π`, by the same count as
/// `a_half_bar_rising_out_of_a_bars_cap_is_fused_by_every_rule_either_way_round`'s
/// union. Q503 joins a wall cut into a grid into one face, but this wall is
/// cut by the half bar's flat side as well as its cap, and Q503's own lane
/// counted six faces here in a probe it did not commit, two of them on the
/// bar's pipe rather than one — still not the single wall of the route's own
/// first bullet. Not traced further; re-files what is left of Q626.
#[test]
fn the_first_union_of_the_third_route_answers_with_two_wall_pieces() {
    let mut model = a_model();
    let bar = a_bar_at(&mut model, -3.0);
    let half = a_half_bar_bulging_left(&mut model, 1.0, 0.0);
    let solid = the_one_solid(&mut model, bar, half, Rule::Union);
    assert_holds(
        &model,
        solid,
        101.0 * PI,
        "the first union of the third route",
    );
    let faces: Vec<_> = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .collect();
    assert_eq!(faces.len(), 6, "the first union's face count");
    let on_the_pipe = faces
        .iter()
        .filter(|face| matches!(face.surface(), rcad_geom::Surface::Cylinder(_)))
        .count();
    assert_eq!(on_the_pipe, 2, "faces on the bar's pipe");
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// A bar of radius 5 at `(x, 0)`, from `z = 0` to `z = 4`.
fn a_bar_at(model: &mut Model, x: f64) -> SolidKey {
    model
        .add_round_bar(5.0, 4.0, &at(x, 0.0, 0.0))
        .expect("a bar of radius five")
}

/// A half bar of `radius` about `(0, y)`, from `z = 1` to `z = 6`, over
/// `x ≤ 0`.
fn a_half_bar_bulging_left(model: &mut Model, radius: f64, y: f64) -> SolidKey {
    let bar = model
        .add_round_bar(radius, 5.0, &at(0.0, y, 1.0))
        .expect("a bar");
    let block = a_block_over(model, [0.0, -20.0], [20.0, 20.0]);
    the_one_solid(model, bar, block, Rule::Difference)
}

/// The bars at `(−3, 0)` and `(3, 0)` united.
fn two_bars_united(model: &mut Model) -> SolidKey {
    let left = a_bar_at(model, -3.0);
    let right = a_bar_at(model, 3.0);
    the_one_solid(model, left, right, Rule::Union)
}

/// Where the bars at `(−3, 0)` and `(3, 0)` overlap.
fn where_two_bars_overlap(model: &mut Model) -> SolidKey {
    let left = a_bar_at(model, -3.0);
    let right = a_bar_at(model, 3.0);
    the_one_solid(model, left, right, Rule::Intersection)
}

/// The part of the bar at `(x, 0)` on its own side of `x = 0`.
fn a_d_of_the_bar_at(model: &mut Model, x: f64) -> SolidKey {
    let bar = a_bar_at(model, x);
    let block = if x < 0.0 {
        a_block_over(model, [0.0, -20.0], [20.0, 20.0])
    } else {
        a_block_over(model, [-20.0, -20.0], [0.0, 20.0])
    };
    the_one_solid(model, bar, block, Rule::Difference)
}

/// `L`, the area where the circles of radius 5 about `(−3, 0)` and `(3, 0)`
/// overlap.
fn the_lens() -> f64 {
    50.0 * f64::acos(0.6) - 24.0
}

/// `D`, the volume of [`a_d_of_the_bar_at`]: the bar less the piece the
/// chord `x = 0` cuts off, which is half the lens.
fn the_volume_of_a_d() -> f64 {
    4.0 * (25.0 * PI - the_lens() / 2.0)
}

/// A block over the rectangle from `low` to `high`, from `z = −5` to
/// `z = 15`, well past everything else here.
fn a_block_over(model: &mut Model, low: [f64; 2], high: [f64; 2]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [low[0], low[1]],
        [high[0], low[1]],
        [high[0], high[1]],
        [low[0], high[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(&rectangle, 20.0, &at(0.0, 0.0, -5.0))
        .expect("a positive height")
}

fn the_one_solid(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let answer = fuse(model, one, other, rule).unwrap_or_else(|why| panic!("{rule:?}: {why}"));
    let [solid] = answer[..] else {
        panic!("{rule:?} is one solid, not {answer:?}");
    };
    solid
}

fn fuse(
    model: &mut Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
) -> Result<Vec<SolidKey>, Error> {
    model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().to_vec())
}

fn assert_holds(model: &Model, solid: SolidKey, volume: f64, what: &str) {
    assert_total_holds(model, &[solid], volume, what);
}

/// Checks that `model` validates and that the solids in `answer`, taken
/// together, hold `volume` give or take the sum of their meshes'
/// `volume_error`. An empty `answer` holds `0.0` exactly.
fn assert_total_holds(model: &Model, answer: &[SolidKey], volume: f64, what: &str) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} left a model that does not validate: {why}"));
    let mut found = 0.0;
    let mut error = 0.0;
    for &solid in answer {
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.001).expect("a fine mesh"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} does not measure: {why}"));
        found += measured.volume();
        error += measured.volume_error();
    }
    assert!(
        (found - volume).abs() <= error,
        "{what} holds {volume} and this one says {found} give or take {error}"
    );
}

/// The corners of each ring of the face at `z = 4` facing up, each ring's
/// corners in order of `y`: the outline first, as the face stores it, then
/// the holes in order of their lowest corner.
fn the_rings_of_the_cap(model: &Model, solid: SolidKey) -> Vec<Vec<[f64; 2]>> {
    let solid = model.solid(solid).expect("this model made the key");
    let [cap] = solid
        .faces()
        .filter(|face| {
            face.surface()
                .as_plane()
                .is_some_and(|plane| plane.normal().components() == [0.0, 0.0, 1.0])
                && face
                    .wires()
                    .flat_map(|wire| wire.points())
                    .all(|point| point.coordinates()[2] == 4.0)
        })
        .collect::<Vec<_>>()[..]
    else {
        panic!("one face at z = 4 faces up");
    };
    let mut rings: Vec<Vec<[f64; 2]>> = cap
        .wires()
        .map(|wire| {
            let mut corners: Vec<[f64; 2]> = wire
                .points()
                .map(|point| {
                    let [x, y, _] = point.coordinates();
                    [x, y]
                })
                .collect();
            corners.sort_by(|one, other| one[1].total_cmp(&other[1]));
            corners
        })
        .collect();
    rings[1..].sort_by(|one, other| one[0][1].total_cmp(&other[0][1]));
    rings
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift")
}
