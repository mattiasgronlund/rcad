//! A bar across the wall of a tube, united with it: a joined flat face whose
//! outline and hole each run along two pipes.
//!
//! The tube is a bar of radius 7 and height 4 less a bar of radius 1 standing
//! through it. The bar has radius 5 and stands at `(−4, −4)`, so its circle
//! crosses the tube's outer circle at `(0, −7)` and `(−7, 0)` and its inner
//! circle at `(0, −1)` and `(−1, 0)`: it pokes out past the wall on one side
//! and into the hole on the other, and leaves most of the hole open.
//!
//! United, each cap is the tube's ring and the bar's disk joined into one face.
//! Its outline is an arc of the outer pipe and an arc of the bar, and its hole
//! an arc of the inner pipe and an arc of the bar. **Each ring has two corners,
//! so neither encloses any area between its corners**, and neither runs along
//! one surface, so the region it bounds is not simply the inside of a pipe.
//! Before the fix that was a panic in `fuse/assemble.rs`, both ways round: *"2
//! rings of a joined face enclose no area between their corners, and the regions
//! they bound cannot say which of them is the outline"*. Q576 found it; Q359 is
//! the same panic for rings along one pipe, and §191.5 is the family.
//!
//! # The rings `holds_the_other_rings` still leaves unplaced
//!
//! Where it answers `None` for two rings of one joined face, `rings_round`
//! panicked with that same message, and since Q1253 refuses the fuse
//! `CannotPlaceTheRingsOfAJoinedFlatFace`. The rest of this file builds the families it
//! answers `None` for and pins what the fuse does with each. Each one that is
//! refused is refused before `rings_round`, and each test names the stage: that was
//! read with a breakpoint on `select::choose`, on `rings_round` and on
//! `holds_the_other_rings` (Q597), which the two tests above do reach. Two
//! holed bars turned a quarter are answered and reach `holds_the_other_rings`
//! too, but not by the family they were built for (Q616). The same two bars
//! unturned, the probe for that family, were refused before `rings_round`
//! until Q897 and are answered now. The probe for an arc along a cone's
//! parabola, a tube and a cone cut by one leaning plane, is refused before
//! `rings_round` under every rule (Q634), and so is the probe for two arcs
//! along two cones' hyperbolas, a cut cone less a second cone on its axis,
//! because two different cones could not be crossed (Q636), then because the
//! arrangement could not order points round the circle they cross in (Q1122),
//! then because a point could not be tested against the cut cone's face
//! (Q1129), then, the cups made, because the check that the cups do not run
//! through themselves could not tell the cells of their ceiling and then of
//! their skin apart (Q1139, Q1161), and then because the fuse with the flush
//! block found no point inside a band of the second cone's skin with the cups
//! first and could not follow the cut cone's skin with the block first. Every
//! rule is answered now (Q1224) and reaches `rings_round`, but the joined
//! wall's outline winds, so `holds_the_other_rings` is not asked of it.
//! The family of
//! corners that sweep no area and are not on one line is built with corners
//! that cancel exactly, face by face, and the fuse that would join its cap
//! puts a corner on its arc wherever one of its own walls would cross it, so
//! the ring `rings_round` is handed winds (Q641). A closed side that is one of
//! several, its ring coming back through the side's one corner, is reached on
//! a bored block's joined top, where the outline winds and places it; the probe
//! with no ring that winds is refused before `rings_round` (Q802). A hole
//! along the cut cone's hyperbola closed by a ball, which no plane can close
//! without putting corners on the outline, reaches `rings_round` in the union
//! with an outline and a hole along two hyperbolas, and is answered under every
//! rule since `holds_the_other_rings` reads such an arc (Q1233, Q1289). A cap whose sides all
//! run along pipes reaches `rings_round` with corners that cancel exactly, the
//! one ring there and the outline by that (Q1120). A second such ring, as a
//! hole, is the cap's own shape an eighth the size standing through it: the
//! union and the big cap less the small one reach `rings_round` with two rings
//! `holds_the_other_rings` places neither of, and are refused
//! `CannotPlaceTheRingsOfAJoinedFlatFace` (Q1253). With one of those sides straight and leaning,
//! the fuse puts a corner on the arc where the line along it leaves the cap,
//! and the ring winds (Q1254).

use std::f64::consts::PI;

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, EdgeRun, HalfSpace, Isometry, NamedRun, Orientation, Picker,
    Plane, Point, Scale, Surface, SurfaceMeet, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftStep, Error, Model, OnAFace, ProfileBuilder, Rule, SolidBuilder, SolidKey,
};
use smallvec::smallvec;

mod rings;

/// With both caps flush, both are joined, and the union is the tube and the
/// bar less what they share, whichever solid goes first.
#[test]
fn a_bar_across_a_tubes_wall_united_with_it_holds_both() {
    for bar_first in [false, true] {
        let (mut model, tube, bar) = a_tube_and_a_bar_across_its_wall(0.0, 4.0);
        let [one, other] = if bar_first { [bar, tube] } else { [tube, bar] };
        let answer = the_one_solid(&mut model, one, other, Rule::Union);
        let shared = 4.0 * (what_the_bar_shares_with(7.0) - what_the_bar_shares_with(1.0));
        let volume = 4.0 * 48.0 * PI + 4.0 * 25.0 * PI - shared;
        assert_holds(&model, answer, volume, "the union");
    }
}

/// With the bar standing a unit lower, only the top cap is flush and joined.
#[test]
fn a_bar_across_a_tubes_wall_and_below_it_united_with_it_holds_both() {
    let (mut model, tube, bar) = a_tube_and_a_bar_across_its_wall(-1.0, 5.0);
    let answer = the_one_solid(&mut model, tube, bar, Rule::Union);
    let shared = 4.0 * (what_the_bar_shares_with(7.0) - what_the_bar_shares_with(1.0));
    let volume = 4.0 * 48.0 * PI + 5.0 * 25.0 * PI - shared;
    assert_holds(&model, answer, volume, "the union");
}

fn a_tube_and_a_bar_across_its_wall(bottom: f64, height: f64) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let outer = model
        .add_round_bar(7.0, 4.0, &Isometry::identity())
        .expect("a bar of radius seven");
    let through = model
        .add_round_bar(1.0, 6.0, &at(0.0, 0.0, -1.0))
        .expect("a bar of radius one");
    let tube = the_one_solid(&mut model, outer, through, Rule::Difference);
    let bar = model
        .add_round_bar(5.0, height, &at(-4.0, -4.0, bottom))
        .expect("a bar of radius five");
    (model, tube, bar)
}

/// The area the bar's disk shares with a disk of `radius` about the tube's axis:
/// two circular segments either side of the chord between the two crossings.
fn what_the_bar_shares_with(radius: f64) -> f64 {
    let (bar, apart) = (5.0_f64, 32.0_f64.sqrt());
    let segment = |mine: f64, theirs: f64| {
        let half_angle = ((apart * apart + mine * mine - theirs * theirs) / (2.0 * apart * mine))
            .clamp(-1.0, 1.0)
            .acos();
        mine * mine * (half_angle - half_angle.sin() * half_angle.cos())
    };
    segment(radius, bar) + segment(bar, radius)
}

fn the_one_solid(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> SolidKey {
    let answer = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{rule:?} refused: {why}"));
    let [solid] = answer.solids() else {
        panic!("{rule:?} is one solid, not {:?}", answer.solids());
    };
    *solid
}

fn assert_holds(model: &Model, solid: SolidKey, volume: f64, what: &str) {
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} left a model that does not validate: {why}"));
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.001).expect("a fine mesh"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} does not measure: {why}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{what} holds {volume} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

fn at(x: f64, y: f64, z: f64) -> Isometry {
    Isometry::translation(Vector::new(x, y, z)).expect("a whole-number shift")
}

/// **A ring along one torus.** A block whose top and bottom are square to a
/// bend's axis would cut its wall in two circles, but the block's four sides
/// are parallel to the axis and off it, and `intersect` refuses such a plane
/// against a torus as a quartic before a single face is cut — even where, as
/// here for the sides at `±6`, the plane misses the torus entirely. Every rule,
/// both ways round.
///
/// The quarter bend is `rings::a_quarter_bend` with ring 4 and tube 1; the
/// block is the square from `(−6, −6)` to `(6, 6)` stood from `z = 0` to `z = 2`.
/// Refused in `cuts::gather`, so it tests nothing about `rings_round`.
#[test]
fn a_block_square_to_a_bends_axis_is_refused_before_anything_is_cut() {
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for block_first in [false, true] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let bend = rings::a_quarter_bend(&mut model, 4.0, 1.0);
            let square =
                ProfileBuilder::new(vec![[-6.0, -6.0], [6.0, -6.0], [6.0, 6.0], [-6.0, 6.0]])
                    .build(&Policy::certified(), &mut ProvenanceLog::new())
                    .expect("a square");
            let block = model
                .add_extrusion(&square, 2.0, &Isometry::identity())
                .expect("a block");
            let [one, other] = if block_first {
                [block, bend]
            } else {
                [bend, block]
            };
            let refused = model.fuse(
                &[one],
                &[other],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            assert!(
                matches!(
                    refused,
                    Err(Error::Geometry(
                        rcad_geom::Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic
                    ))
                ),
                "{rule:?}, block first {block_first}: {refused:?}"
            );
        }
    }
}

/// **A ring along one torus, and an arc along a torus beside straight
/// sides.** The top halves, `z ≥ 0`, of the two half bends of one ring, ring 4
/// and tube 1: one from `+x` round through `+y` to `−x`, the other from `−x`
/// round through `−y` to `+x`. Their caps meet flush on `y = 0`, and their
/// floors are two half rings of `z = 0`, each a ring of four corners all on
/// the `x` axis — outer arc, straight side, inner arc, straight side — so
/// neither floor encloses any area between its corners.
///
/// Pappus gives the whole ring `2π²·4·1² = 8π²` and each top half of a half
/// bend a quarter of it, `2π²`, so the union is `4π²`, the intersection is
/// nothing, and each difference is the first half, `2π²`. **Every rule, both
/// ways round, gives that, and leaves a model that validates.**
///
/// United, the two floors join into the whole flat ring between the circles of
/// radius 3 and 5, an outline and a hole each along the torus alone, two
/// corners each. `holds_the_other_rings` places them by the disk of the circle
/// each runs along, named by the picker `intersect` names it by: the outer
/// circle's disk holds the inner circle's corners and the inner's does not
/// hold the outer's (`Some(true)` and `Some(false)` both ways round, a
/// temporary print, Q1387).
///
/// Until Q1387 the union, both ways round, was refused `ResultIsNotManifold {
/// count: 4 }` at `darts_along_each_edge` in `fuse/assemble.rs`. It keeps both
/// floors and both walls, so both halves of the circle `z = 0` cuts the ring
/// in, of radius 5 and of radius 3, run between the same two corners,
/// `(±5, 0, 0)` and `(±3, 0, 0)`, and `which_arc_of_a_circle_this_is` told the
/// two arcs of a closed crossing apart for a plane against a ball, a cone or a
/// pipe and not a ring: one edge, carried by `z = 0` and the ring and made from
/// one input edge of each solid, was claimed four times (a temporary print,
/// Q1373). With the arcs told apart it panicked in `rings_round`, because
/// `holds_the_other_rings` answered `None` for a ring along a torus, as Q597
/// had predicted (Q1387).
///
/// Until Q1373 every one was refused `Undecidable` on `a point inside a piece
/// of a face`, its record naming the two corners of the first solid's start
/// cap, as the quarter bends below were and for the same reason: `select`
/// found no point inside that half disk, whose sides are its chord along
/// `z = 0` and half of a circle `y = 0` cuts the ring in.
///
/// Until Q1367 every one was refused `AFaceTrimmedByACurveThisCannotFollow` on
/// the second solid's wall, by `holds.rs`, which read no face on a ring with
/// an arc among its sides; `a_quarter_and_a_half_bends_wall_hold_their_own_points`
/// checks the reading it has now.
///
/// Until Q1267 every one was refused on the second solid's floor, by
/// `holds.rs`. `select` finds `(0, ±3.75, 0)` inside the first solid's floor,
/// a rung towards its outer arc from the middle of the arc's chord, and asks
/// whether the second solid's floor, flush with it, holds that point.
/// `the_way_a_side_runs` found no way into that floor's inner arc, from
/// `(±3, 0, 0)` round the axis (a temporary print, Q1268), because
/// `the_way_into_an_arc` read no arc of a circle a plane cuts a ring in.
///
/// Until Q1268 every one was refused a step earlier, `a point inside a piece
/// of a face` on the first solid's floor, named by its four corners `(±5, 0,
/// 0)` and `(±3, 0, 0)`. Every point offered was a blend of those corners, on
/// the `x` axis: the torus offered no middle to step from on a plane square to
/// its axis, and the floor's outer arc, the only thing that winds round a
/// point of it, was not counted. Until Q617 every one was refused a step
/// earlier still, while that floor was cut into arcs
/// (`which_way_an_edge_runs_round` in `fuse/arrange.rs`),
/// `AFaceTrimmedByACurveThisCannotFollow`. The arc it stopped on was the
/// floor's inner side, half of the circle of radius 3 that `z = 0` cuts the
/// torus in, picked by `y ≥ 0` (a temporary print, Q617). Q597 had put it on
/// the cap's half of a circle `y = 0` cuts. The arrangement read a picker only
/// on a ball's circle or a cone's or a pipe's circle or ellipse. It now also
/// reads a picker whose plane holds both ends on any circle a plane cuts a
/// ring in, and so reads every arc of both solids' floors, caps and walls.
#[test]
fn the_top_halves_of_two_half_bends_fuse_by_every_rule_to_what_pappus_gives() {
    for (rule, volume) in [
        (Rule::Union, 4.0 * PI * PI),
        (Rule::Intersection, 0.0),
        (Rule::Difference, 2.0 * PI * PI),
    ] {
        for turned_first in [false, true] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let [one, other] = [turned_first, !turned_first].map(|turned| {
                the_top_half_of_a_half_bend(&mut model, turned)
                    .unwrap_or_else(|why| panic!("the top half of a half bend: {why:?}"))
            });
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("both halves are sound");
            // Pappus: a half disk of area π/2 whose centroid is 4 from the
            // axis, carried half a turn round it, π·4.
            for half in [one, other] {
                assert_measures(
                    &model,
                    half,
                    PI / 2.0 * PI * 4.0,
                    "the top half of a half bend",
                );
            }
            let what = format!("{rule:?}, turned first {turned_first}");
            assert_fuses_to(&mut model, one, other, rule, volume, &what);
        }
    }
}

/// **Half-meridians the arrangement reads, a floor and a wall `holds.rs`
/// reads, and a cap `select` finds a point in.** The top halves, `z ≥ 0`, of
/// two quarter bends of one ring, ring 4 and tube 1: one from `+x` round to
/// `+y`, the other from `+y` round to `−x`, meeting flush on `x = 0`. Pappus
/// gives each `π/2 · 4 · π/2 = π²`, so the union is the top half of a half
/// bend, `2π²`, the intersection is nothing, and each difference is the first
/// quarter, `π²`. Every rule, both ways round, gives that, and leaves a model
/// that validates. Unlike the half bends' floors above, each floor's four
/// corners are not on one line.
///
/// Until Q1373 every one was refused `Undecidable` on `a point inside a piece
/// of a face`, its record naming the two corners of the first solid's start
/// cap, `(3, 0, 0)` and `(5, 0, 0)`, or `(0, 3, 0)` and `(0, 5, 0)` turned.
/// That half disk's sides are its chord along `z = 0` and half of a circle the
/// cap's plane cuts the ring in, so its two corners wind round nothing, and
/// `select`'s `in_the_bulge_of` counted no arc of a circle a plane through the
/// ring's axis cuts. `(4, 0, 0.5)` was offered, half way from the chord's
/// middle to the arc, and not held (a temporary print, Q1373).
///
/// Until Q1367 every one was refused `AFaceTrimmedByACurveThisCannotFollow` on
/// the second solid's wall, by the arm of
/// `what_a_face_cut_by_part_of_a_crossing_holds` in `holds.rs` that refused
/// every face on a ring with an arc among its sides (a temporary print, Q1267):
/// rays fired from the first solid meet the second wall, at points like
/// `(1.318, 4.403, 0.803)`. `holds.rs` now reads that wall as the part of the
/// ring in front of its floor and both caps, and
/// `a_quarter_and_a_half_bends_wall_hold_their_own_points` checks it.
///
/// Until Q1267 every one was refused a step earlier, on the second solid's
/// floor, because `the_way_into_an_arc` read no arc of a ring's circle: it asked
/// for a centre before its first way, a picker whose plane holds both ends, and
/// two circles have no one centre. Then, the arc read, because no gradient of a
/// ring was read to say which side of the floor's arcs the floor keeps
/// (a temporary print, Q1267). Until Q617 every one was refused earlier still,
/// in the arrangement, on the first solid's start cap: its side over the tube
/// is half of a meridian circle, picked by the start's own side of the axis and
/// `z ≥ 0` (a print in `which_way_an_edge_runs_round`, Q617).
///
/// The arrangement and `holds.rs` now read such an arc by one spelling, and
/// `a_quarter_and_a_half_bends_floor_hold_their_own_points` checks its sign.
#[test]
fn the_top_halves_of_two_quarter_bends_fuse_by_every_rule_to_what_pappus_gives() {
    for (rule, volume) in [
        (Rule::Union, 2.0 * PI * PI),
        (Rule::Intersection, 0.0),
        (Rule::Difference, PI * PI),
    ] {
        for turned_first in [false, true] {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let [one, other] = [turned_first, !turned_first].map(|turned| {
                the_top_half_of_a_quarter_bend(&mut model, turned)
                    .unwrap_or_else(|why| panic!("the top half of a quarter bend: {why:?}"))
            });
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("both quarters are sound");
            for quarter in [one, other] {
                assert_measures(&model, quarter, PI * PI, "the top half of a quarter bend");
            }
            let what = format!("{rule:?}, turned first {turned_first}");
            assert_fuses_to(&mut model, one, other, rule, volume, &what);
        }
    }
}

/// **A quarter bend's floor and a half bend's hold their own points**, which
/// checks the way `the_way_into_an_arc` reads into an arc of a circle a plane
/// cuts a ring in, and how far in the piece between such an arc and its chord
/// reaches. Each floor is a band of `z = 0` between the circles of radius 3
/// and 5, with the two arcs bulging the same way off their chords.
///
/// Each point is in the band or out of it by more than a rounding, and the
/// ones that tell a wrong reading apart:
///
/// - `(2.9, 2.9, 0)` on the quarter floor is past its outer chord `x + y = 5`
///   and inside the circle of radius 5: held only if that arc is read on the
///   far side of its chord from the axis. Read the other way round, it is in
///   no piece and outside the polygon of chords. It is past the inner chord
///   too, and the ring holds it, so reading the ring's own sign as the inside
///   of each circle, as for a ball, puts it in both pieces, which cancel, and
///   answers it not held (tried, Q1267).
/// - `(2, 2, 0)` is between the chords, so inside the polygon, and inside the
///   circle of radius 3, so in the inner arc's piece, which the floor leaves
///   out. The ring's own sign there is outside, in the hole.
/// - `(0, 2, 0)` on the half floor, whose four corners are all on the `x` axis
///   so the polygon of chords has no area: it is inside both circles, in both
///   pieces, which cancel, where `(0, 4, 0)` is in the outer piece alone.
#[test]
fn a_quarter_and_a_half_bends_floor_hold_their_own_points() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let quarter = the_top_half_of_a_quarter_bend(&mut model, false)
        .unwrap_or_else(|why| panic!("the top half of a quarter bend: {why:?}"));
    let half = the_top_half_of_a_half_bend(&mut model, false)
        .unwrap_or_else(|why| panic!("the top half of a half bend: {why:?}"));
    for (solid, asked) in [
        (
            quarter,
            [
                ([2.9, 2.9, 0.0], OnAFace::Held),
                ([2.2, 2.2, 0.0], OnAFace::Held),
                ([4.0, 0.5, 0.0], OnAFace::Held),
                ([2.0, 2.0, 0.0], OnAFace::NotHeld),
                ([3.6, 3.6, 0.0], OnAFace::NotHeld),
                ([-1.0, 4.0, 0.0], OnAFace::NotHeld),
            ],
        ),
        (
            half,
            [
                ([0.0, 4.0, 0.0], OnAFace::Held),
                ([-2.9, 2.9, 0.0], OnAFace::Held),
                ([4.0, 0.5, 0.0], OnAFace::Held),
                ([0.0, 2.0, 0.0], OnAFace::NotHeld),
                ([0.0, 5.5, 0.0], OnAFace::NotHeld),
                ([0.0, -4.0, 0.0], OnAFace::NotHeld),
            ],
        ),
    ] {
        let solid = model.solid(solid).expect("this model made the key");
        let floor = solid
            .faces()
            .find(|face| {
                face.surface()
                    == Surface::Plane(
                        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
                            .expect("the z axis is a unit normal"),
                    )
            })
            .expect("a bend's top half has a floor on z = 0");
        for (at, expected) in asked {
            let held = solid
                .face_holds_the_point(
                    floor.key(),
                    rcad_predicates::Point::from(at),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{at:?} is answered: {why:?}"));
            assert_eq!(held, expected, "{at:?}");
        }
    }
}

/// **A quarter bend's wall and a half bend's hold their own points**, which
/// checks the reading `holds.rs` gives a face on a ring with arcs for sides:
/// the part of the ring in front of its floor and of each cap. Each wall is
/// the top half of the tube, `z > 0`, between its two caps through the axis.
///
/// Each point is on the ring, a degree or more from every cap and off the
/// floor by a fifth of the tube or more, and the ones that tell a wrong
/// reading apart:
///
/// - `(2.828, 2.828, −1)` is under the floor: held only if the floor's side
///   is read the wrong way round.
/// - `(−0.695, 3.939, 1)` and `(3.939, −0.695, 1)` are ten degrees past one
///   cap of the quarter: held only if that cap's side is read wrong.
/// - `(−2.828, 2.828, 1)` and `(2.828, −2.828, 1)` are in front of one cap of
///   the quarter and behind the other: held if the wall were read as the part
///   in front of either cap, as three quarters of a bend would be.
/// - `(1.318, 4.403, 0.803)` is where a ray from the quarter met the turned
///   quarter's wall when every fuse of the two was refused there (Q1267): held
///   by the quarter, and not by the turned one.
#[test]
fn a_quarter_and_a_half_bends_wall_hold_their_own_points() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let quarter = the_top_half_of_a_quarter_bend(&mut model, false)
        .unwrap_or_else(|why| panic!("the top half of a quarter bend: {why:?}"));
    let turned = the_top_half_of_a_quarter_bend(&mut model, true)
        .unwrap_or_else(|why| panic!("the turned top half of a quarter bend: {why:?}"));
    let half = the_top_half_of_a_half_bend(&mut model, false)
        .unwrap_or_else(|why| panic!("the top half of a half bend: {why:?}"));
    for (solid, asked) in [
        (
            quarter,
            vec![
                ([2.828, 2.828, 1.0], OnAFace::Held),
                ([4.792, 0.845, 0.5], OnAFace::Held),
                ([1.318, 4.403, 0.803], OnAFace::Held),
                ([2.828, 2.828, -1.0], OnAFace::NotHeld),
                ([-0.695, 3.939, 1.0], OnAFace::NotHeld),
                ([3.939, -0.695, 1.0], OnAFace::NotHeld),
                ([-2.828, 2.828, 1.0], OnAFace::NotHeld),
                ([2.828, -2.828, 1.0], OnAFace::NotHeld),
            ],
        ),
        (
            turned,
            vec![
                ([1.318, 4.403, 0.803], OnAFace::NotHeld),
                ([-1.318, 4.403, 0.803], OnAFace::Held),
                ([-2.828, 2.828, 1.0], OnAFace::Held),
                ([-2.828, 2.828, -1.0], OnAFace::NotHeld),
            ],
        ),
        (
            half,
            vec![
                ([0.0, 4.0, 1.0], OnAFace::Held),
                ([-4.792, 0.845, 0.5], OnAFace::Held),
                ([4.792, 0.845, 0.5], OnAFace::Held),
                ([0.0, 4.0, -1.0], OnAFace::NotHeld),
                ([0.0, -4.0, 1.0], OnAFace::NotHeld),
                ([4.792, -0.845, 0.5], OnAFace::NotHeld),
            ],
        ),
    ] {
        let solid = model.solid(solid).expect("this model made the key");
        let wall = solid
            .faces()
            .find(|face| matches!(face.surface(), Surface::Torus(_)))
            .expect("a bend's top half has a wall on the ring");
        for (at, expected) in asked {
            let held = solid
                .face_holds_the_point(
                    wall.key(),
                    rcad_predicates::Point::from(at),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{at:?} is answered: {why:?}"));
            assert_eq!(held, expected, "{at:?}");
        }
    }
}

/// That `one` fused with `other` under `rule` answers, leaves a model that
/// validates, and holds `volume`: one solid, or none where `volume` is nothing.
fn assert_fuses_to(
    model: &mut Model,
    one: SolidKey,
    other: SolidKey,
    rule: Rule,
    volume: f64,
    what: &str,
) {
    let answer = model
        .fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} refused: {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what} left a model that does not validate: {why}"));
    if volume == 0.0 {
        assert_eq!(answer.solids(), [], "{what} is nothing");
        return;
    }
    let [solid] = answer.solids() else {
        panic!("{what} is one solid, not {:?}", answer.solids());
    };
    assert_measures(model, *solid, volume, what);
}

fn assert_measures(model: &Model, solid: SolidKey, volume: f64, what: &str) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::for_scale(model.scale()),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what} does not measure: {why}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{what} holds {volume} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The top half, `z ≥ 0`, of a half bend about `+z` with ring 4 and tube 1,
/// from `+x` round through `+y` to `−x`; `turned`, half a turn round, from `−x`
/// through `−y` to `+x`. Four faces: the half torus wall, the half ring of
/// `z = 0` under it, and a half disk of `y = 0` at each end.
///
/// Written out the way `rings.rs` writes a bend: the wall used forwards, as a
/// torus faces out of its tube (`docs/decisions.md` §361), every number turned
/// exactly, and each arc named by the half-spaces and the pipe of the ring's
/// own radius that pick it. No edge here is closed, so there is no circle to
/// turn over: the wall's loop already runs right-handed about the way out of
/// the tube, anticlockwise seen from above.
fn the_top_half_of_a_half_bend(model: &mut Model, turned: bool) -> rcad_topo::Result<SolidKey> {
    let (ring, tube) = (4.0, 1.0);
    let way = if turned { -1.0 } else { 1.0 };
    let at = |x: f64| Point::new(written_out(way * x), 0.0, 0.0);
    let facing =
        |x: f64, y: f64, z: f64| Vector::new(written_out(way * x), written_out(way * y), z);
    let wall =
        Surface::Torus(Torus::new(Axis::z(), ring, tube).expect("a tube narrower than its ring"));
    let floor = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let caps = Plane::from_point_and_normal(Point::origin(), facing(0.0, -1.0, 0.0))
        .expect("the y axis is a unit normal");
    let across = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("the y axis is a unit normal");
    let pipe = Cylinder::new(Axis::z(), ring).expect("the ring's own radius");
    let side =
        |normal: Vector| Picker::from(HalfSpace::new(normal, 0.0).expect("a normal with a length"));

    let mut builder = SolidBuilder::new();
    let outer_start = builder.add_vertex(at(ring + tube))?;
    let inner_start = builder.add_vertex(at(ring - tube))?;
    let outer_end = builder.add_vertex(at(-(ring + tube)))?;
    let inner_end = builder.add_vertex(at(-(ring - tube)))?;

    let along_the_floor = |inside: Picker| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(floor),
        picked_by: smallvec![inside, side(facing(0.0, 1.0, 0.0))],
    };
    let over_the_tube = |towards: f64| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(across),
        picked_by: smallvec![
            side(facing(towards, 0.0, 0.0)),
            side(Vector::new(0.0, 0.0, 1.0))
        ],
    };
    let outer = builder.add_edge(
        outer_start,
        outer_end,
        along_the_floor(Picker::OutsideAPipe(pipe)),
    )?;
    let inner = builder.add_edge(
        inner_start,
        inner_end,
        along_the_floor(Picker::InsideAPipe(pipe)),
    )?;
    let over_the_start = builder.add_edge(outer_start, inner_start, over_the_tube(1.0))?;
    let over_the_end = builder.add_edge(inner_end, outer_end, over_the_tube(-1.0))?;
    let under_the_start = builder.add_edge(inner_start, outer_start, EdgeRun::Straight)?;
    let under_the_end = builder.add_edge(outer_end, inner_end, EdgeRun::Straight)?;

    let floor_wire = a_wire(
        &mut builder,
        &[
            (under_the_start, Orientation::Reversed),
            (inner, Orientation::Forward),
            (under_the_end, Orientation::Reversed),
            (outer, Orientation::Reversed),
        ],
    )?;
    let floor_face = builder.add_face(Surface::Plane(floor), &[floor_wire])?;
    let start_wire = a_wire(
        &mut builder,
        &[
            (under_the_start, Orientation::Forward),
            (over_the_start, Orientation::Forward),
        ],
    )?;
    let start_cap = builder.add_face(Surface::Plane(caps), &[start_wire])?;
    let end_wire = a_wire(
        &mut builder,
        &[
            (under_the_end, Orientation::Forward),
            (over_the_end, Orientation::Forward),
        ],
    )?;
    let end_cap = builder.add_face(Surface::Plane(caps), &[end_wire])?;
    let wall_wire = a_wire(
        &mut builder,
        &[
            (outer, Orientation::Forward),
            (over_the_end, Orientation::Reversed),
            (inner, Orientation::Reversed),
            (over_the_start, Orientation::Reversed),
        ],
    )?;
    let wall_face = builder.add_face(wall, &[wall_wire])?;
    builder.add_shell(&[
        (floor_face, Orientation::Forward),
        (start_cap, Orientation::Forward),
        (end_cap, Orientation::Forward),
        (wall_face, Orientation::Forward),
    ])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The top half, `z ≥ 0`, of a quarter bend about `+z` with ring 4 and tube 1,
/// from `+x` round to `+y`; `turned`, a quarter turn on, from `+y` round to
/// `−x`. Written out as [`the_top_half_of_a_half_bend`] is. Each side of its
/// floor is a quarter of a circle round the axis, of radius `r`, and is picked
/// by the plane through its two ends, `x + y ≥ r` (`y − x ≥ r` turned), which
/// holds both ends exactly and has the whole quarter on or in front of it.
fn the_top_half_of_a_quarter_bend(model: &mut Model, turned: bool) -> rcad_topo::Result<SolidKey> {
    let (ring, tube) = (4.0, 1.0);
    let (start, end) = if turned {
        ([0.0, 1.0], [-1.0, 0.0])
    } else {
        ([1.0, 0.0], [0.0, 1.0])
    };
    let flat = |[x, y]: [f64; 2]| Vector::new(x, y, 0.0);
    let away = |[x, y]: [f64; 2]| [written_out(-x), written_out(-y)];
    let at =
        |[x, y]: [f64; 2], out: f64| Point::new(written_out(x * out), written_out(y * out), 0.0);
    let wall =
        Surface::Torus(Torus::new(Axis::z(), ring, tube).expect("a tube narrower than its ring"));
    let floor = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    // The way round at the start is `end`, and at the end it is `−start`: the
    // start cap faces back along it and the end cap on along it.
    let start_cap = Plane::from_point_and_normal(Point::origin(), flat(away(end)))
        .expect("an axis is a unit normal");
    let end_cap = Plane::from_point_and_normal(Point::origin(), flat(away(start)))
        .expect("an axis is a unit normal");
    let pipe = Cylinder::new(Axis::z(), ring).expect("the ring's own radius");
    let side = |normal: Vector, offset: f64| {
        Picker::from(HalfSpace::new(normal, offset).expect("a normal with a length"))
    };
    let both_ends = Vector::new(start[0] + end[0], start[1] + end[1], 0.0);

    let mut builder = SolidBuilder::new();
    let outer_start = builder.add_vertex(at(start, ring + tube))?;
    let inner_start = builder.add_vertex(at(start, ring - tube))?;
    let outer_end = builder.add_vertex(at(end, ring + tube))?;
    let inner_end = builder.add_vertex(at(end, ring - tube))?;

    let along_the_floor = |inside: Picker, out: f64| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(floor),
        picked_by: smallvec![inside, side(both_ends, out)],
    };
    let over_the_tube = |cap: Plane, towards: [f64; 2]| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(cap),
        picked_by: smallvec![
            side(flat(towards), 0.0),
            side(Vector::new(0.0, 0.0, 1.0), 0.0)
        ],
    };
    let outer = builder.add_edge(
        outer_start,
        outer_end,
        along_the_floor(Picker::OutsideAPipe(pipe), ring + tube),
    )?;
    let inner = builder.add_edge(
        inner_start,
        inner_end,
        along_the_floor(Picker::InsideAPipe(pipe), ring - tube),
    )?;
    let over_the_start =
        builder.add_edge(outer_start, inner_start, over_the_tube(start_cap, start))?;
    let over_the_end = builder.add_edge(inner_end, outer_end, over_the_tube(end_cap, end))?;
    let under_the_start = builder.add_edge(inner_start, outer_start, EdgeRun::Straight)?;
    let under_the_end = builder.add_edge(outer_end, inner_end, EdgeRun::Straight)?;

    let floor_wire = a_wire(
        &mut builder,
        &[
            (under_the_start, Orientation::Reversed),
            (inner, Orientation::Forward),
            (under_the_end, Orientation::Reversed),
            (outer, Orientation::Reversed),
        ],
    )?;
    let floor_face = builder.add_face(Surface::Plane(floor), &[floor_wire])?;
    let start_wire = a_wire(
        &mut builder,
        &[
            (under_the_start, Orientation::Forward),
            (over_the_start, Orientation::Forward),
        ],
    )?;
    let start_face = builder.add_face(Surface::Plane(start_cap), &[start_wire])?;
    let end_wire = a_wire(
        &mut builder,
        &[
            (under_the_end, Orientation::Forward),
            (over_the_end, Orientation::Forward),
        ],
    )?;
    let end_face = builder.add_face(Surface::Plane(end_cap), &[end_wire])?;
    let wall_wire = a_wire(
        &mut builder,
        &[
            (outer, Orientation::Forward),
            (over_the_end, Orientation::Reversed),
            (inner, Orientation::Reversed),
            (over_the_start, Orientation::Reversed),
        ],
    )?;
    let wall_face = builder.add_face(wall, &[wall_wire])?;
    builder.add_shell(&[
        (floor_face, Orientation::Forward),
        (start_face, Orientation::Forward),
        (end_face, Orientation::Forward),
        (wall_face, Orientation::Forward),
    ])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// `−0.0` written as `+0.0`, which is how a person writing a zero out writes it.
fn written_out(number: f64) -> f64 {
    if number == 0.0 { 0.0 } else { number }
}

fn a_wire(
    builder: &mut SolidBuilder,
    steps: &[(DraftEdge, Orientation)],
) -> rcad_topo::Result<rcad_topo::DraftWire> {
    let mut along = Vec::with_capacity(steps.len());
    for &(edge, turn) in steps {
        along.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
    }
    builder.add_wire(&along)
}

mod cones;

/// **No corner of the other rings placed.** Two bars of radius 10 and height
/// 4 standing at `(−6, 0)` and `(6, 0)`, each less a bar of radius 2 through
/// the origin. United, the joined cap's outline would be an arc of each big
/// bar with its two corners where their circles cross, `(0, ±8)`, and its
/// hole the small circle, whose one corner the fuse puts at `(0, −2)`: on the
/// chord of the outline's arcs and inside both big bars, so
/// `holds_the_other_rings` would pass it over and place nothing.
///
/// **Every rule, both ways round, now answers**, and holds what the circles
/// say: the same volumes as the pair turned a quarter, below. Each used to be
/// refused in `select::choose` on `a point inside a piece of a face`, its
/// record naming those three corners on `x = 0` (Q616), until `select` offered
/// rungs nearer the places the line square to each arc's chord crosses its
/// circle (`docs/decisions.md` §365.6, Q897). Over the six fuses,
/// `join_up_what_is_one_face` is reached 18 times and `holds_the_other_rings`
/// 8 times (breakpoints, Q897), and `rings_round` does not panic. Where the
/// fuse put the hole's corner, and what `holds_the_other_rings` answered, was
/// not read.
#[test]
fn two_holed_bars_whose_corners_lie_on_one_line_are_answered_under_every_rule() {
    let lens = 200.0 * 0.6_f64.acos() - 96.0;
    let hole = 4.0 * PI;
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pair = two_bars_holed_through_the_origin(&mut model, [6.0, 0.0]);
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, pair, rule, swapped);
        let area = match rule {
            Rule::Union => 200.0 * PI - lens - hole,
            Rule::Intersection => lens - hole,
            Rule::Difference => 100.0 * PI - lens,
            other => panic!("not a rule this probes: {other:?}"),
        };
        assert_holds(&model, the_only(answer, &what), 4.0 * area, &what);
    }
}

/// The same two holed bars turned a quarter, standing at `(0, ∓6)`. The fuse
/// puts the hole's corner at `(0, 2)` this time, off the chord `y = 0` through
/// the outline's corners `(±8, 0)`, so the hole's corner is placed by how the
/// outline's arcs bulge and nothing is left unplaced. Every rule, both ways
/// round, is answered and holds what the circles say.
///
/// Under the union `holds_the_other_rings` is reached four times, once for
/// each of the two rings of each cap, and answers `Some(false)` for each hole
/// and `Some(true)` for each outline (read off its return with gdb, Q616). So
/// family 5 needs the hole's corner on the chord, and the probe above that
/// puts it there was refused before this until Q897; what it reaches now is
/// read there.
#[test]
fn two_holed_bars_turned_a_quarter_are_answered_with_the_hole_placed_by_the_outlines_bulge() {
    let lens = 200.0 * 0.6_f64.acos() - 96.0;
    let hole = 4.0 * PI;
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pair = two_bars_holed_through_the_origin(&mut model, [0.0, 6.0]);
        let answer = fused_in_order(&mut model, pair, rule, swapped)
            .unwrap_or_else(|why| panic!("{rule:?}, swapped {swapped}: {why:?}"));
        let [solid] = answer[..] else {
            panic!("{rule:?}, swapped {swapped}: {answer:?}");
        };
        let area = match rule {
            Rule::Union => 200.0 * PI - lens - hole,
            Rule::Intersection => lens - hole,
            Rule::Difference => 100.0 * PI - lens,
            other => panic!("not a rule this probes: {other:?}"),
        };
        assert_holds(
            &model,
            solid,
            4.0 * area,
            &format!("{rule:?}, swapped {swapped}"),
        );
    }
}

/// **An arc along a cone's hyperbola.** A pointed cone, apex at the origin,
/// slope 1, base at `z = 20`, and a block from `x = −12` to `24`, `y = ±24`,
/// `z = −1` to `18`. The block's wall `x = −12` cuts the cone in a hyperbola,
/// so the piece of that wall inside the cone is a ring of two corners: an arc
/// of the hyperbola and a chord along the block's top. That is the ring a
/// joined face on `x = −12` would hand `holds_the_other_rings`, which answers
/// `None` for a hyperbola arc. Two nested cups of two cones on that wall, and a
/// block flush with it to join its pieces, were the probe; a ball's pocket
/// in place of the second cone was the other.
///
/// **This pair, the first fuse each needs, is answered under every rule, both
/// ways round**, and each answer holds what the cone and the block say. It
/// used to be refused in `select::choose` on `a point inside a piece of a
/// face`, its record naming the two corners where the block's top meets the
/// cone, `(−12, ±√180, 18)` (Q616). No point the search builds is found inside
/// one piece with those corners, though each such piece holds a double
/// (§365.2, Q635), and that piece is now placed by the side it shares with a
/// piece of the other solid's face instead (§365.3, Q712). Moving the wall to
/// `x = −11` or the top above the cone's base was refused the same way, and
/// has not been run since. The probe of two cones answers its second fuse
/// (the next test, Q1139); the ball's pocket has not been run since, so
/// whether it now reaches `holds_the_other_rings` is not known.
///
/// The part of the cone the block holds is the cone up to `z = 18`, `1944π`,
/// less what lies past the wall, `∫₁₂¹⁸ (z² acos(12/z) − 12√(z² − 144)) dz`,
/// which is `1944 acos(2/3) − 144√180 + 576 ln((18 + √180)/12)`.
#[test]
fn a_cone_cut_by_a_block_wall_off_its_axis_is_answered_under_every_rule() {
    let cone = 8000.0 * PI / 3.0;
    let block = 36.0 * 48.0 * 19.0;
    let shared = what_the_block_holds_of_the_cone();
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pair = a_cone_and_a_block_across_it(&mut model);
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, pair, rule, swapped);
        let volume = match (rule, swapped) {
            (Rule::Union, _) => cone + block - shared,
            (Rule::Intersection, _) => shared,
            (Rule::Difference, false) => cone - shared,
            (Rule::Difference, true) => block - shared,
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_holds(&model, the_only(answer, &what), volume, &what);
    }
}

/// **Two cups along two cones' hyperbolas on one face.** The cone the block
/// above holds, less a second pointed cone of slope 1 on the same axis, apex
/// at `z = 2` and base at `z = 16`. On the wall `x = −12` the second cone's
/// cup reaches down to `(−12, 0, 14)` and is `√52` either side at its base,
/// inside the first's, which reaches down to `(−12, 0, 12)` and is `√112`
/// either side at that height: the two cups nest. United with the block
/// `[−12, −11] × [10, 12] × [17.25, 17.75]`, flush with that wall, the wall's
/// pieces would join into one face whose outline and hole each run along a
/// hyperbola between two corners, which `holds_the_other_rings` answers `None`
/// for.
///
/// **The cut cone and the second cone, the first fuse that needs, are
/// answered under every rule, both ways round**, and each answer holds what
/// the cones and the wall say. The two surfaces cross: a cone here is both
/// nappes, and the first's upper nappe meets the second's lower one in the
/// circle of radius 1 at `z = 1`, which `intersect` names now — it used to
/// refuse every pair of two different cones in `cuts::gather` (a breakpoint on
/// `intersect`, Q636). That circle is on the first cone's face and below the
/// second's apex, so the two faces never meet. The arrangement of the first
/// cone's face refused it next, `ACrossingThisCannotBeOrderedAlong`, because
/// two cones were not a pair `a_pair_that_crosses_in_a_conic` in
/// `fuse/arrange.rs` listed (Q1122), and then `CannotTellTheCellsOfAFaceApart`
/// (Q1129). Then every rule was refused `AFaceTrimmedByACurveThisCannotFollow`
/// from `select::place_the_piece`: every point offered for a piece sent a ray
/// through the cut cone's side face, and `holds.rs` could not say which side of
/// the hyperbola's chord its arc on `x = −12` is on, since
/// `the_way_into_an_arc` reads that only of a curve that closes. Fixed there,
/// the wall `x = −12` refused the same way for its own arc of that hyperbola
/// (Q1139, with a print in `holds.rs`). An arc between two points of a curve
/// that does not close is the bounded piece of it, on the side of its chord
/// away from where the branch runs off, and both faces read it so now.
///
/// The second cone holds `2744π/3`, of which
/// [`what_the_second_cone_holds_past_the_wall`] lies past `x = −12`; the rest
/// is inside the cut cone, which reaches past the second cone's base and round
/// its apex. Both solids are measured first, the cut cone against what the
/// test above says.
#[test]
fn a_cut_cone_less_a_second_cone_on_its_axis_is_answered_under_every_rule() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let [cut, second] = a_cut_cone_and_a_second_cone_on_its_axis(&mut model);
    assert_holds(
        &model,
        cut,
        what_the_block_holds_of_the_cone(),
        "the cut cone",
    );
    assert_holds(&model, second, 2744.0 * PI / 3.0, "the second cone");

    let first_skin = Cone::new(Axis::z(), 1.0).expect("a positive slope");
    let second_skin = first_skin.transformed(&at(0.0, 0.0, 2.0));
    assert_eq!(
        [
            first_skin.point_at(0.0, 1.0),
            second_skin.point_at(0.0, -1.0)
        ],
        [Point::new(1.0, 0.0, 1.0); 2],
        "both cones hold the point one up and one out, the second on its lower nappe"
    );
    let named = rcad_geom::intersect(
        &Surface::Cone(first_skin),
        &Surface::Cone(second_skin),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert_eq!(
        named,
        Ok(SurfaceMeet::Curves(vec![
            NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
                .expect("a whole circle needs nothing picking it")
        ])),
        "one slope, two apexes: the one circle halfway between them"
    );

    let cut = what_the_block_holds_of_the_cone();
    let second = 2744.0 * PI / 3.0;
    let past_the_wall = what_the_second_cone_holds_past_the_wall();
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pair = a_cut_cone_and_a_second_cone_on_its_axis(&mut model);
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, pair, rule, swapped);
        let volume = match (rule, swapped) {
            (Rule::Union, _) => cut + past_the_wall,
            (Rule::Intersection, _) => second - past_the_wall,
            (Rule::Difference, false) => cut - second + past_the_wall,
            (Rule::Difference, true) => past_the_wall,
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_holds(&model, the_only(answer, &what), volume, &what);
    }
}

/// **The cut cone's two faces along the hyperbola hold their own points**: its
/// skin is the cone up to `z = 18` with `x ≥ −12`, on the upper nappe alone,
/// and its wall is the part of `x = −12` inside that, between the hyperbola's
/// upper branch and `z = 18`. Every point asked is on its face's surface
/// exactly. On the wall, `(−12, 0, −20)` is inside the cone too, inside the
/// lower branch, on the side of the chord the arc is on: the part of the
/// plane inside the cone is two regions there, and only the arc's is the
/// wall's (Q1139).
#[test]
fn the_cut_cones_faces_along_the_hyperbola_hold_their_own_points() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let [cone, block] = a_cone_and_a_block_across_it(&mut model);
    let cut = the_one_solid(&mut model, cone, block, Rule::Intersection);
    let solid = model.solid(cut).expect("this model made the key");
    let wall_plane =
        Plane::from_point_and_normal(Point::new(-12.0, 0.0, 0.0), Vector::new(-1.0, 0.0, 0.0))
            .expect("the x axis is a unit normal");
    let skin = solid
        .faces()
        .find(|face| matches!(face.surface(), Surface::Cone(_)))
        .expect("the cut cone has a skin");
    let wall = solid
        .faces()
        .find(|face| face.surface() == Surface::Plane(wall_plane))
        .expect("the cut cone has a wall on x = -12");
    let asked = |face: rcad_topo::FaceKey, at: [f64; 3]| {
        solid
            .face_holds_the_point(
                face,
                rcad_predicates::Point::from(at),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{at:?} is answered: {why:?}"))
    };
    assert_eq!(asked(skin.key(), [3.0, 4.0, 5.0]), OnAFace::Held);
    assert_eq!(asked(skin.key(), [0.0, -10.0, 10.0]), OnAFace::Held);
    assert_eq!(asked(skin.key(), [-13.0, 0.0, 13.0]), OnAFace::NotHeld);
    assert_eq!(asked(skin.key(), [0.0, 20.0, 20.0]), OnAFace::NotHeld);
    assert_eq!(asked(skin.key(), [0.0, 5.0, -5.0]), OnAFace::NotHeld);
    assert_eq!(asked(wall.key(), [-12.0, 0.0, 15.0]), OnAFace::Held);
    assert_eq!(asked(wall.key(), [-12.0, 0.0, 10.0]), OnAFace::NotHeld);
    assert_eq!(asked(wall.key(), [-12.0, 10.0, 15.0]), OnAFace::NotHeld);
    assert_eq!(asked(wall.key(), [-12.0, 0.0, -20.0]), OnAFace::NotHeld);
}

/// **The nested cups against the flush block are answered under every rule,
/// both ways round**, and each answer holds what the cones say: the cut cone
/// less the second cone, and the block `[−12, −11] × [10, 12] × [17.25, 17.75]`
/// flush with its wall `x = −12`.
///
/// **The block is wholly inside the cups and shares one face with them.** Its
/// face on `x = −12` lies on the cups' wall, both facing `−x`, and inside it:
/// the wall at height `z` reaches `√(z² − 144)` either side of `y = 0`, which
/// is `√153.5625`, more than 12, at the block's floor. Every other face of the
/// block is clear of the cups' skin: its corner furthest from the axis,
/// `(−12, 12)`, is `√288` from it, under the cut cone's `17.25` at the floor;
/// its floor is above the cavity's ceiling `z = 16` and its top below the cups'
/// top `z = 18`. So the union holds the cups,
/// `what_the_block_holds_of_the_cone() − 2744π/3 +
/// what_the_second_cone_holds_past_the_wall()`, the intersection the whole
/// block, `1`, the cups less the block one less than the cups, and the block
/// less the cups nothing.
///
/// **The joined wall reaches `rings_round`, and with an outline that winds**,
/// so `holds_the_other_rings` is not asked (a temporary print in `rings_round`,
/// Q1224): the second cone's base plane `z = 16` left two corners on the
/// wall's hyperbola at `(−12, ±√112, 16)`, so its outline has four corners off
/// one line, and the ring along the second cone's hyperbola, two corners on the
/// chord `z = 16`, is taken for a hole. The family this file probes, an outline
/// and a hole that each enclose no area, is not what this pair builds.
///
/// Until Q1224, with the cups first every rule was refused `a point inside a
/// piece of a face` on the band of the second cone's skin between the cups'
/// wall and the block's plane `x = −11`: its four corners are all on the
/// circle of radius 14 at `z = 16`, so every blend of them came back onto that
/// circle, and the band hangs below it. With the block first, every rule was
/// refused `AFaceTrimmedByACurveThisCannotFollow` on the cut cone's skin, whose
/// hyperbola the base plane split, so its middle piece ran below `z = 18` with
/// both ends behind it and `holds.rs` read only an arc of a curve that closes
/// against a plane like that. Until Q1161 every rule was refused
/// `CannotTellTheCellsOfAFaceApart` by the check that the cups do not run
/// through themselves, first on the cups' face on `z = 16` and then on the cut
/// cone's skin.
#[test]
fn the_nested_cups_and_a_block_flush_with_their_wall_are_answered_under_every_rule() {
    let cups = what_the_block_holds_of_the_cone() - 2744.0 * PI / 3.0
        + what_the_second_cone_holds_past_the_wall();
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pair = the_nested_cups_and_the_flush_block(&mut model);
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, pair, rule, swapped);
        let volume = match (rule, swapped) {
            (Rule::Union, _) => cups,
            (Rule::Intersection, _) => 1.0,
            (Rule::Difference, false) => cups - 1.0,
            (Rule::Difference, true) => {
                let nothing = answer.unwrap_or_else(|why| panic!("{what} refused: {why}"));
                assert!(nothing.is_empty(), "{what}: {nothing:?}");
                continue;
            }
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_holds(&model, the_only(answer, &what), volume, &what);
    }
}

/// **The cups' skin holds its own points**: the cut cone's skin, its
/// hyperbola on `x = −12` split at `(−12, ±√112, 16)` where the second cone's
/// base plane crossed it, so one piece of it runs from there down through
/// `(−12, 0, 12)` with both ends behind the top `z = 18`. `holds.rs` refused
/// every point of this skin `AFaceTrimmedByACurveThisCannotFollow` until it
/// read an arc of a hyperbola against a plane both its ends are behind
/// (Q1224); the cut cone's skin, unsplit, was read already
/// ([`the_cut_cones_faces_along_the_hyperbola_hold_their_own_points`]).
#[test]
fn the_cups_skin_holds_its_own_points() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let [cut, second] = a_cut_cone_and_a_second_cone_on_its_axis(&mut model);
    let cups = the_one_solid(&mut model, cut, second, Rule::Difference);
    let solid = model.solid(cups).expect("this model made the key");
    let cone = Surface::Cone(Cone::new(Axis::z(), 1.0).expect("a positive slope"));
    let skin = solid
        .faces()
        .find(|face| face.surface() == cone)
        .expect("the cups have the cut cone's skin");
    let corners_on_the_wall = skin
        .wires()
        .flat_map(|wire| wire.points().collect::<Vec<_>>())
        .filter(|corner| corner.coordinates()[0] == -12.0)
        .count();
    assert_eq!(corners_on_the_wall, 4, "the hyperbola is split in three");
    let asked = |at: [f64; 3]| {
        solid
            .face_holds_the_point(
                skin.key(),
                rcad_predicates::Point::from(at),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{at:?} is answered: {why:?}"))
    };
    assert_eq!(asked([3.0, 4.0, 5.0]), OnAFace::Held);
    assert_eq!(asked([0.0, -10.0, 10.0]), OnAFace::Held);
    assert_eq!(asked([-11.0, 0.0, 11.0]), OnAFace::Held);
    assert_eq!(asked([-13.0, 0.0, 13.0]), OnAFace::NotHeld);
    assert_eq!(asked([0.0, 20.0, 20.0]), OnAFace::NotHeld);
    assert_eq!(asked([0.0, 5.0, -5.0]), OnAFace::NotHeld);
}

/// The cut cone less the second cone, and the block flush with its wall,
/// cups first.
fn the_nested_cups_and_the_flush_block(model: &mut Model) -> [SolidKey; 2] {
    let [cut, second] = a_cut_cone_and_a_second_cone_on_its_axis(model);
    let cups = the_one_solid(model, cut, second, Rule::Difference);
    let block = a_block(
        model,
        &[[-12.0, 10.0], [-11.0, 10.0], [-11.0, 12.0], [-12.0, 12.0]],
        17.25,
        0.5,
    );
    [cups, block]
}

/// The part of the second cone past the wall `x = −12`, where its radius
/// `u = z − 2` is more than 12: `∫₁₂¹⁴ (u² acos(12/u) − 12√(u² − 144)) du`,
/// whose antiderivative is `(u³/3) acos(12/u) − 8u√(u² − 144) + 576 ln(u + √(u² − 144))`,
/// the same as [`what_the_block_holds_of_the_cone`]'s.
fn what_the_second_cone_holds_past_the_wall() -> f64 {
    let rise = 52.0_f64.sqrt();
    2744.0 / 3.0 * (6.0_f64 / 7.0).acos() - 112.0 * rise + 576.0 * ((14.0 + rise) / 12.0).ln()
}

/// The pointed cone of slope 1 with its base at `z = 20`, and the block from
/// `x = −12` to `24`, `y = ±24`, `z = −1` to `18`, cone first.
fn a_cone_and_a_block_across_it(model: &mut Model) -> [SolidKey; 2] {
    let cone = cones::a_pointed_cone(model, 1.0, 20.0);
    let block = a_block(
        model,
        &[[-12.0, -24.0], [24.0, -24.0], [24.0, 24.0], [-12.0, 24.0]],
        -1.0,
        19.0,
    );
    [cone, block]
}

/// The part of [`a_cone_and_a_block_across_it`]'s cone that its block holds,
/// as the test of the pair works it out. A quadrature of the slices agrees to
/// 1e-11 (Q636).
fn what_the_block_holds_of_the_cone() -> f64 {
    let past_the_wall = 1944.0 * (2.0_f64 / 3.0).acos() - 144.0 * 180.0_f64.sqrt()
        + 576.0 * ((18.0 + 180.0_f64.sqrt()) / 12.0).ln();
    1944.0 * PI - past_the_wall
}

/// The cone the block holds, and the second cone, its apex moved up to `z = 2`.
fn a_cut_cone_and_a_second_cone_on_its_axis(model: &mut Model) -> [SolidKey; 2] {
    let [cone, block] = a_cone_and_a_block_across_it(model);
    let cut = the_one_solid(model, cone, block, Rule::Intersection);
    let second = cones::a_pointed_cone(model, 1.0, 14.0);
    model
        .place_solid(second, &at(0.0, 0.0, 2.0))
        .expect("this model made the key");
    [cut, second]
}

/// **An arc along a cone's parabola.** The plane `z = x + 4` leans at the
/// slope of a pointed cone of slope 1, apex at the origin and base at
/// `z = 10`, so it cuts the cone in a parabola. A tube about the same axis,
/// radius 12 less radius 4 from `z = −10` to `20`, and the cone are each cut
/// to the far side of that plane by a wedge built face by face, the only door
/// to a plane leaning along `x` (extrusions run along `z`, and a quarter turn
/// is not exact). United, the two faces on the plane would join into one whose
/// outline is the outer pipe's ellipse, one corner, and whose hole is an arc of
/// the inner pipe's ellipse and an arc of the parabola meeting at `(0, ±4, 4)`,
/// two corners. Neither encloses any area between its corners, and the hole
/// runs along a parabola, which `holds_the_other_rings` answers `None` for.
///
/// **That is not reached: every rule, both ways round, is refused before
/// `rings_round`, on `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`.** Cone first
/// used to stop earlier, on `AFaceTrimmedByACurveThisCannotFollow` naming a
/// face of the tube; sharing the arc picker between `holds.rs` and
/// `arrange.rs` (Q1033) takes it one step further, to the same three-surface
/// meet as tube first. The hole's corners are where the cone, the inner pipe
/// and the leaning plane meet, and `rcad-geom` works out a meet of two curved
/// surfaces and a plane about one axis only when the plane is square to that
/// axis (a gap, `docs/decisions.md` §31.2). Two other shapes were run on the
/// way, and their set-up fuses refused as well (Q634): the bar of radius 12
/// cut to this side less the cut cone, and the cut cone less a bar of radius
/// 1 up its axis, both the same three-surface refusal since Q1033 (Q1092 had
/// the second one on `AFaceTrimmedByACurveThisCannotFollow`, before it).
///
/// Both solids are measured before they are fused. The tube holds the ring's
/// area times its mean height, `128π · 16 = 2048π`. The cone holds
/// `1000π/3 − (2000/3)·atan(1/2) + 416/9`, of which `320π/3 − 512/9` lies inside
/// the inner pipe; the union is then `2048π + 320π/3 − 512/9`, and the
/// intersection the rest of the cone. All four agree with a quadrature of the
/// slices to 1e-12 (Q634).
#[test]
fn a_tube_and_a_cone_cut_by_one_leaning_plane_are_refused_before_their_rings_are_placed() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let [tube, cone] = a_tube_and_a_cone_cut_by_one_leaning_plane(&mut model);
    assert_holds(&model, tube, 2048.0 * PI, "the cut tube");
    let the_cut_cone = 1000.0 * PI / 3.0 - 2000.0 / 3.0 * 0.5_f64.atan() + 416.0 / 9.0;
    assert_holds(&model, cone, the_cut_cone, "the cut cone");

    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pair = a_tube_and_a_cone_cut_by_one_leaning_plane(&mut model);
        let refused = fused_in_order(&mut model, pair, rule, swapped);
        let first = if swapped { "cone" } else { "tube" };
        assert!(
            matches!(
                refused,
                Err(Error::Geometry(
                    rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat
                ))
            ),
            "{rule:?}, {first} first: {refused:?}"
        );
    }
}

/// The tube and the cone, each cut to the far side of the leaning plane, tube
/// first. The bar is cut before it is holed; holed first, it is cut to the
/// same tube (`a_holed_tube_cut_by_the_wedge_is_the_tube_past_the_leaning_plane`).
fn a_tube_and_a_cone_cut_by_one_leaning_plane(model: &mut Model) -> [SolidKey; 2] {
    let outer = model
        .add_round_bar(12.0, 30.0, &at(0.0, 0.0, -10.0))
        .expect("a bar of radius twelve");
    let cone = cones::a_pointed_cone(model, 1.0, 10.0);
    let [bar, cone] = [outer, cone].map(|whole| {
        let wedge = the_wedge_past_the_leaning_plane(model)
            .unwrap_or_else(|why| panic!("the wedge: {why:?}"));
        the_one_solid(model, whole, wedge, Rule::Intersection)
    });
    let through = model
        .add_round_bar(4.0, 34.0, &at(0.0, 0.0, -12.0))
        .expect("a bar of radius four");
    let tube = the_one_solid(model, bar, through, Rule::Difference);
    [tube, cone]
}

/// The prism `z ≥ x + 4`, `x ≥ −14`, `z ≤ 22`, `−14 ≤ y ≤ 14`: a right
/// triangle in the `xz`-plane with corners `(−14, −10)`, `(−14, 22)` and
/// `(18, 22)`, run along `y`. Its face on `z = x + 4` faces `(1, 0, −1)`, and
/// every other face lies clear of the tube and the cone.
fn the_wedge_past_the_leaning_plane(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    use Orientation::{Forward, Reversed};
    let mut builder = SolidBuilder::new();
    let [low, high, far] = [[-14.0, -10.0], [-14.0, 22.0], [18.0, 22.0]];
    let mut both_ends = |[x, z]: [f64; 2]| {
        Ok::<_, rcad_topo::Error>([
            builder.add_vertex(Point::new(x, -14.0, z))?,
            builder.add_vertex(Point::new(x, 14.0, z))?,
        ])
    };
    let [low_front, low_back] = both_ends(low)?;
    let [high_front, high_back] = both_ends(high)?;
    let [far_front, far_back] = both_ends(far)?;
    let mut straight = |start, end| builder.add_edge(start, end, EdgeRun::Straight);
    let low_across = straight(low_front, low_back)?;
    let high_across = straight(high_front, high_back)?;
    let far_across = straight(far_front, far_back)?;
    let front_leaning = straight(low_front, far_front)?;
    let front_top = straight(high_front, far_front)?;
    let front_side = straight(low_front, high_front)?;
    let back_leaning = straight(low_back, far_back)?;
    let back_top = straight(high_back, far_back)?;
    let back_side = straight(low_back, high_back)?;

    let leaning = a_flat_face(
        &mut builder,
        Point::new(0.0, 0.0, 4.0),
        Vector::new(1.0, 0.0, -1.0),
        &[
            (front_leaning, Reversed),
            (low_across, Forward),
            (back_leaning, Forward),
            (far_across, Reversed),
        ],
    )?;
    let side = a_flat_face(
        &mut builder,
        Point::new(-14.0, 0.0, 0.0),
        Vector::new(-1.0, 0.0, 0.0),
        &[
            (front_side, Forward),
            (high_across, Forward),
            (back_side, Reversed),
            (low_across, Reversed),
        ],
    )?;
    let top = a_flat_face(
        &mut builder,
        Point::new(0.0, 0.0, 22.0),
        Vector::new(0.0, 0.0, 1.0),
        &[
            (front_top, Forward),
            (far_across, Forward),
            (back_top, Reversed),
            (high_across, Reversed),
        ],
    )?;
    let front = a_flat_face(
        &mut builder,
        Point::new(0.0, -14.0, 0.0),
        Vector::new(0.0, -1.0, 0.0),
        &[
            (front_leaning, Forward),
            (front_top, Reversed),
            (front_side, Reversed),
        ],
    )?;
    let back = a_flat_face(
        &mut builder,
        Point::new(0.0, 14.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
        &[
            (back_side, Forward),
            (back_top, Forward),
            (back_leaning, Reversed),
        ],
    )?;
    builder.add_shell(&[
        (leaning, Forward),
        (side, Forward),
        (top, Forward),
        (front, Forward),
        (back, Forward),
    ])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

fn a_flat_face(
    builder: &mut SolidBuilder,
    point: Point,
    outward: Vector,
    steps: &[(DraftEdge, Orientation)],
) -> rcad_topo::Result<rcad_topo::DraftFace> {
    let plane = Plane::from_point_and_normal(point, outward).expect("a normal with a length");
    let wire = a_wire(builder, steps)?;
    builder.add_face(Surface::Plane(plane), &[wire])
}

/// **A tube cut by a block whose floor crosses it and whose top clears it is
/// the tube from the block's floor up**, both ways round: the tube of the test
/// above, holed before it is cut, and the block `[−14, 14] × [−14, 14] × [0,
/// 22]`. That is the ring between radius 4 and 12 stood from `z = 0` to `20`,
/// `128π · 20 = 2560π`.
///
/// The block's top, `z = 22`, is two above the tube's, and the two pipes cross
/// its plane in circles of radius 4 and 12 there. Until Q1090 the fuse cut the
/// top along both, and could not find a point inside the ring between them to
/// place it by (Q634, with a print in `select::choose`). Each circle meets
/// nothing else on the top and the pipe face carrying it ends at `z = 20`, so
/// `cuts::gather` leaves it off now. A bar of radius 12 got the circle of
/// radius 12 on the block's top as well and was answered; so was the tube
/// against a block reaching down past it, `z = −30` to `22`.
#[test]
fn a_tube_cut_by_a_block_whose_top_clears_it_is_the_tube_from_the_blocks_floor_up() {
    for swapped in [false, true] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let outer = model
            .add_round_bar(12.0, 30.0, &at(0.0, 0.0, -10.0))
            .expect("a bar of radius twelve");
        let through = model
            .add_round_bar(4.0, 34.0, &at(0.0, 0.0, -12.0))
            .expect("a bar of radius four");
        let tube = the_one_solid(&mut model, outer, through, Rule::Difference);
        let block = a_block(
            &mut model,
            &[[-14.0, -14.0], [14.0, -14.0], [14.0, 14.0], [-14.0, 14.0]],
            0.0,
            22.0,
        );
        let answer = fused_in_order(&mut model, [tube, block], Rule::Intersection, swapped)
            .unwrap_or_else(|why| panic!("swapped {swapped}: {why:?}"));
        let [solid] = answer[..] else {
            panic!("swapped {swapped}: {answer:?}");
        };
        assert_holds(&model, solid, 2560.0 * PI, &format!("swapped {swapped}"));
    }
}

/// **The holed tube cut to the far side of the leaning plane by the wedge**,
/// both ways round, is the tube
/// `a_tube_and_a_cone_cut_by_one_leaning_plane_are_refused_before_their_rings_are_placed`
/// measures, `2048π`. The
/// wedge's top, `z = 22`, clears the tube as the block's above does, and was
/// cut along the same two circles and refused the same way until Q1090.
#[test]
fn a_holed_tube_cut_by_the_wedge_is_the_tube_past_the_leaning_plane() {
    for swapped in [false, true] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let outer = model
            .add_round_bar(12.0, 30.0, &at(0.0, 0.0, -10.0))
            .expect("a bar of radius twelve");
        let through = model
            .add_round_bar(4.0, 34.0, &at(0.0, 0.0, -12.0))
            .expect("a bar of radius four");
        let tube = the_one_solid(&mut model, outer, through, Rule::Difference);
        let wedge = the_wedge_past_the_leaning_plane(&mut model)
            .unwrap_or_else(|why| panic!("the wedge: {why:?}"));
        let answer = fused_in_order(&mut model, [tube, wedge], Rule::Intersection, swapped)
            .unwrap_or_else(|why| panic!("swapped {swapped}: {why:?}"));
        let [solid] = answer[..] else {
            panic!("swapped {swapped}: {answer:?}");
        };
        assert_holds(&model, solid, 2048.0 * PI, &format!("swapped {swapped}"));
    }
}

/// **The same tube and a block narrower than it, whose top clears it**, under
/// every rule, both ways round: the block `[−10, 10] × [−10, 10] × [0, 22]`.
/// The circle of radius 12 on the block's top crosses its four sides, so it
/// splits their edges where the pipe splits the walls', and it stays on
/// although the pipe face never reaches `z = 22`. Left off, the top's edges
/// and the walls' disagree about their corners, and the union is refused with
/// an edge that has one face along it. What they share is the ring between
/// radius 4 and the square's part of the disk of radius 12, from `z = 0` to
/// `20`.
#[test]
fn a_tube_and_a_narrower_block_whose_top_clears_it_are_answered_under_every_rule() {
    let past_the_side = 44.0_f64.sqrt();
    let quarter =
        10.0 * past_the_side + 72.0 * ((10.0_f64 / 12.0).asin() - (past_the_side / 12.0).asin());
    let shared = 20.0 * (4.0 * quarter - 16.0 * PI);
    let (tube_alone, block_alone) = (30.0 * 128.0 * PI, 20.0 * 20.0 * 22.0);
    for (rule, swapped) in every_rule_both_ways() {
        let volume = match (rule, swapped) {
            (Rule::Union, _) => tube_alone + block_alone - shared,
            (Rule::Intersection, _) => shared,
            (Rule::Difference, false) => tube_alone - shared,
            (Rule::Difference, true) => block_alone - shared,
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let outer = model
            .add_round_bar(12.0, 30.0, &at(0.0, 0.0, -10.0))
            .expect("a bar of radius twelve");
        let through = model
            .add_round_bar(4.0, 34.0, &at(0.0, 0.0, -12.0))
            .expect("a bar of radius four");
        let tube = the_one_solid(&mut model, outer, through, Rule::Difference);
        let block = a_block(
            &mut model,
            &[[-10.0, -10.0], [10.0, -10.0], [10.0, 10.0], [-10.0, 10.0]],
            0.0,
            22.0,
        );
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, [tube, block], rule, swapped)
            .unwrap_or_else(|why| panic!("{what}: {why:?}"));
        let [solid] = answer[..] else {
            panic!("{what}: {answer:?}");
        };
        assert_holds(&model, solid, volume, &what);
    }
}

/// **The same tube and a block whose top leans and clears it**, under every
/// rule, both ways round, certified and tolerant: the block `[−14, 14] × [−14,
/// 14]` from `z = 0` up to the plane `4z = x + 96`. Over the tube, `|x| ≤ 12`,
/// that top is at `z = 21` or higher, one or more above the tube's.
///
/// The two pipes cross the top's plane in ellipses that meet nothing else on
/// it, as the circles on the square top of
/// `a_tube_cut_by_a_block_whose_top_clears_it_is_the_tube_from_the_blocks_floor_up`
/// do. Until Q1174 `cuts::gather` could prove a pipe face clear only of a plane
/// square to its axis, so it left them on, and every rule was refused the way
/// Q1090's was, under either policy: no point found inside the ring between
/// the ellipses, `(0, −4, 24)` to `(0, −12, 24)`.
///
/// Nothing of the tube reaches the top, so the answers are the square-topped
/// block's with the block's volume changed: `28² · 24 = 18816`, the area of its
/// floor times its height over the floor's middle. They share `2560π`, the
/// ring stood from `z = 0` to `20`, and the tube is `128π · 30 = 3840π`. The
/// faces are the square-topped block's too: the top is left whole, the floor
/// is cut by both pipes, and each pipe is one face.
#[test]
fn a_tube_cut_by_a_block_whose_leaning_top_clears_it_is_answered_under_every_rule() {
    let (tube_alone, block_alone, shared) = (3840.0 * PI, 18816.0, 2560.0 * PI);
    for (name, policy) in [
        ("certified", Policy::certified()),
        ("tolerant", Policy::tolerant(100.0)),
    ] {
        for (rule, swapped) in every_rule_both_ways() {
            let (volume, flat_and_pipe) = match (rule, swapped) {
                (Rule::Union, _) => (tube_alone + block_alone - shared, [8, 2]),
                (Rule::Intersection, _) => (shared, [2, 2]),
                (Rule::Difference, false) => (tube_alone - shared, [2, 2]),
                (Rule::Difference, true) => (block_alone - shared, [8, 2]),
                (other, _) => panic!("not a rule this probes: {other:?}"),
            };
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let outer = model
                .add_round_bar(12.0, 30.0, &at(0.0, 0.0, -10.0))
                .expect("a bar of radius twelve");
            let through = model
                .add_round_bar(4.0, 34.0, &at(0.0, 0.0, -12.0))
                .expect("a bar of radius four");
            let tube = the_one_solid(&mut model, outer, through, Rule::Difference);
            let block = the_block_under_a_leaning_top(&mut model)
                .unwrap_or_else(|why| panic!("the block: {why:?}"));
            let [one, other] = if swapped {
                [block, tube]
            } else {
                [tube, block]
            };
            let what = format!("{name}, {rule:?}, swapped {swapped}");
            let answer = model
                .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: {why:?}"));
            let [solid] = answer.solids() else {
                panic!("{what}: {:?}", answer.solids());
            };
            assert_holds(&model, *solid, volume, &what);
            let faces: Vec<Surface> = model
                .solid(*solid)
                .expect("this model made the key")
                .faces()
                .map(|face| face.surface())
                .collect();
            let on_pipes = faces
                .iter()
                .filter(|surface| matches!(surface, Surface::Cylinder(_)))
                .count();
            assert_eq!(
                [faces.len() - on_pipes, on_pipes],
                flat_and_pipe,
                "{what}: {faces:?}"
            );
        }
    }
}

/// The block `[−14, 14] × [−14, 14]` from `z = 0` up to the leaning plane
/// `4z = x + 96`, which is at `z = 20.5` over `x = −14` and `27.5` over `x = 14`.
fn the_block_under_a_leaning_top(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    use Orientation::{Forward, Reversed};
    let mut builder = SolidBuilder::new();
    let mut corner = |x: f64, y: f64| {
        let top = 24.0 + x / 4.0;
        Ok::<_, rcad_topo::Error>([
            builder.add_vertex(Point::new(x, y, 0.0))?,
            builder.add_vertex(Point::new(x, y, top))?,
        ])
    };
    let [left_front_low, left_front_high] = corner(-14.0, -14.0)?;
    let [left_back_low, left_back_high] = corner(-14.0, 14.0)?;
    let [right_front_low, right_front_high] = corner(14.0, -14.0)?;
    let [right_back_low, right_back_high] = corner(14.0, 14.0)?;
    let mut straight = |start, end| builder.add_edge(start, end, EdgeRun::Straight);
    let bottom_front = straight(left_front_low, right_front_low)?;
    let bottom_right = straight(right_front_low, right_back_low)?;
    let bottom_back = straight(right_back_low, left_back_low)?;
    let bottom_left = straight(left_back_low, left_front_low)?;
    let top_front = straight(left_front_high, right_front_high)?;
    let top_right = straight(right_front_high, right_back_high)?;
    let top_back = straight(right_back_high, left_back_high)?;
    let top_left = straight(left_back_high, left_front_high)?;
    let up_left_front = straight(left_front_low, left_front_high)?;
    let up_right_front = straight(right_front_low, right_front_high)?;
    let up_right_back = straight(right_back_low, right_back_high)?;
    let up_left_back = straight(left_back_low, left_back_high)?;

    let faces = [
        a_flat_face(
            &mut builder,
            Point::new(0.0, 0.0, 24.0),
            Vector::new(-1.0, 0.0, 4.0),
            &[
                (top_front, Forward),
                (top_right, Forward),
                (top_back, Forward),
                (top_left, Forward),
            ],
        )?,
        a_flat_face(
            &mut builder,
            Point::new(0.0, 0.0, 0.0),
            Vector::new(0.0, 0.0, -1.0),
            &[
                (bottom_left, Reversed),
                (bottom_back, Reversed),
                (bottom_right, Reversed),
                (bottom_front, Reversed),
            ],
        )?,
        a_flat_face(
            &mut builder,
            Point::new(0.0, -14.0, 0.0),
            Vector::new(0.0, -1.0, 0.0),
            &[
                (bottom_front, Forward),
                (up_right_front, Forward),
                (top_front, Reversed),
                (up_left_front, Reversed),
            ],
        )?,
        a_flat_face(
            &mut builder,
            Point::new(0.0, 14.0, 0.0),
            Vector::new(0.0, 1.0, 0.0),
            &[
                (bottom_back, Forward),
                (up_left_back, Forward),
                (top_back, Reversed),
                (up_right_back, Reversed),
            ],
        )?,
        a_flat_face(
            &mut builder,
            Point::new(14.0, 0.0, 0.0),
            Vector::new(1.0, 0.0, 0.0),
            &[
                (bottom_right, Forward),
                (up_right_back, Forward),
                (top_right, Reversed),
                (up_right_front, Reversed),
            ],
        )?,
        a_flat_face(
            &mut builder,
            Point::new(-14.0, 0.0, 0.0),
            Vector::new(-1.0, 0.0, 0.0),
            &[
                (bottom_left, Forward),
                (up_left_front, Forward),
                (top_left, Reversed),
                (up_left_back, Reversed),
            ],
        )?,
    ];
    builder.add_shell(&faces.map(|face| (face, Forward)))?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// **A hollow frustum under the same leaning top, clear of it** (Q1461), under
/// every rule, both ways round, certified and tolerant. The frustum is the cone
/// about `−z` of slope `1/2` with its apex at `(0, 0, 22)`, from `z = −4` up to
/// `20`, wide end down. The bore through it is the cone about `−z` of slope
/// `1/4` with its apex at `(0, 0, 23)`, from `z = −6` up to `21`. The top is
/// `3.75` or more above the frustum's wall, closest over its top rim.
///
/// Each cone crosses the top's plane in an ellipse on its other nappe, above
/// its apex and near `z = 24`, and the two ellipses meet nothing else on the
/// top, as the pipes' do in
/// `a_tube_cut_by_a_block_whose_leaning_top_clears_it_is_answered_under_every_rule`.
/// Until Q1461 both were kept on and every rule was refused under either
/// policy, `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, for two reasons in
/// `cuts::gather`. The band round a cone face's axis was a box in how far up
/// and how far out, and it reaches down to the apex, so it held the apex at
/// the frustum's full width and could not prove the top clear of the wall.
/// And the two cones, on one axis, meet each other in circles at `z = 21` and
/// `22⅓`, so where the three surfaces meet could not be worked out and the
/// cones were taken to meet on the top, though neither circle comes near it.
///
/// A cone of slope `s` about `−z` with its apex at height `a` holds
/// `π·s²·((a − z₀)³ − (a − z₁)³)/3` between heights `z₀ < z₁` below its apex. So
/// the hollow frustum holds `1464π − 409.5π = 2109π/2`, and shares with the
/// block what it holds from `z = 0` up, `2660π/3 − 3035π/12 = 2535π/4`. The
/// block holds `18816`. The faces are the tube's with cones for pipes, the top
/// left whole.
#[test]
fn a_hollow_frustum_under_a_leaning_top_that_clears_it_is_answered_under_every_rule() {
    a_frustum_and_the_block_under_a_leaning_top_answer_every_rule(
        |model| {
            let frustum = a_frustum_about_z(model, 0.5, 22.0, -1.0, [-4.0, 20.0])
                .unwrap_or_else(|why| panic!("the frustum: {why:?}"));
            let bore = a_frustum_about_z(model, 0.25, 23.0, -1.0, [-6.0, 21.0])
                .unwrap_or_else(|why| panic!("the bore: {why:?}"));
            the_one_solid(model, frustum, bore, Rule::Difference)
        },
        [2109.0 * PI / 2.0, 2535.0 * PI / 4.0],
        [[8, 2, 0], [2, 2, 0]],
    );
}

/// **A frustum with no bore under the same top, clear of it**, answered under
/// every rule, both ways round, certified and tolerant, before Q1461 and
/// since: the cone of the hollow frustum above, slope `1/2` about `−z` with its
/// apex at `(0, 0, 22)`, from `z = −4` to `20`; and the same cone with its apex
/// at `(0, 0, 26)`, over the top's plane, from `z = −1` to `20`.
///
/// Each crosses the top in one ellipse on its other nappe. Before Q1461 the
/// band round neither wall proved the top clear of it, and the band round the
/// second still does not: it reaches down to the apex, and the top crosses the
/// axis between the apex and the wall. So the ellipse is kept on, and with no
/// second cone on the axis nothing stops where it meets the top's other cuts
/// being worked out, and the top comes back whole.
///
/// The first holds `1464π` and shares `2660π/3` with the block; the second
/// holds `6489π/4` and shares `4340π/3`.
#[test]
fn a_frustum_under_a_leaning_top_that_clears_it_is_answered_under_every_rule() {
    for (apex, [bottom, top], volumes) in [
        (22.0, [-4.0, 20.0], [1464.0 * PI, 2660.0 * PI / 3.0]),
        (26.0, [-1.0, 20.0], [6489.0 * PI / 4.0, 4340.0 * PI / 3.0]),
    ] {
        a_frustum_and_the_block_under_a_leaning_top_answer_every_rule(
            |model| {
                a_frustum_about_z(model, 0.5, apex, -1.0, [bottom, top])
                    .unwrap_or_else(|why| panic!("the frustum under {apex}: {why:?}"))
            },
            volumes,
            [[7, 1, 0], [2, 1, 0]],
        );
    }
}

/// **The two frustums above, each bored by a pipe, under the same top**
/// (Q1583), under every rule, both ways round, certified and tolerant. Each
/// bore is a round bar of radius `1/2` on the axis, from two below the
/// frustum's wide end to one above its narrow end, which is `1` out on the
/// first frustum and `3` on the second. So the bore runs out through both caps,
/// and its pipe face, which ends at `z = 20`, is clear of the top.
///
/// The top crosses the pipe in an ellipse from `z = 23.875` to `24.125`, over
/// the bore. The pipe meets the first cone in circles at `z = 21` and `23` and
/// the second at `25` and `27`, all `1/2` out, where the top is `12`, `4`, `4`
/// and `12` out from the axis, so the ellipse meets neither cone on the top.
/// But with only the top flat, where the top, the pipe and a cone meet cannot
/// be worked out, and the pipe and the cone do meet, so until Q1583 nothing
/// proved the ellipse clear of the cone and it was kept on.
///
/// The first was answered all the same: its wall is proved clear of the top,
/// so the ellipse where the top crosses its cone is left off (Q1461), and
/// nothing on the top asks where the three meet. The second's wall is not, for
/// the reason the test above gives, so both ellipses were kept, and every rule
/// was refused under either policy, `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`.
/// The ellipse round the pipe is left off now, by the band round the axis that
/// it lies in.
///
/// Each bore takes `π/4` for every unit of the frustum's height: `6π` of the
/// first frustum's `1464π`, and `5π` of what it shares with the block, and
/// `21π/4` of the second's `6489π/4`, and `5π` of what it shares.
#[test]
fn a_frustum_bored_by_a_pipe_under_a_leaning_top_that_clears_it_is_answered_under_every_rule() {
    for (apex, [bottom, top], volumes) in [
        (22.0, [-4.0, 20.0], [1458.0 * PI, 2645.0 * PI / 3.0]),
        (26.0, [-1.0, 20.0], [1617.0 * PI, 4325.0 * PI / 3.0]),
    ] {
        a_frustum_and_the_block_under_a_leaning_top_answer_every_rule(
            |model| {
                let frustum = a_frustum_about_z(model, 0.5, apex, -1.0, [bottom, top])
                    .unwrap_or_else(|why| panic!("the frustum under {apex}: {why:?}"));
                let bore = model
                    .add_round_bar(0.5, top - bottom + 3.0, &at(0.0, 0.0, bottom - 2.0))
                    .unwrap_or_else(|why| panic!("the bore under {apex}: {why:?}"));
                the_one_solid(model, frustum, bore, Rule::Difference)
            },
            volumes,
            [[8, 1, 1], [2, 1, 1]],
        );
    }
}

/// Fuse what `frustum` builds with `the_block_under_a_leaning_top` under every
/// rule, both ways round, certified and tolerant, and check each answer is one
/// solid holding what the frustum's `[alone, shared]` volumes say, with the
/// faces flat, on a cone and on a pipe counted by `[with_the_block, without]`:
/// the first for the union and the block less the frustum, the second for the
/// other two.
fn a_frustum_and_the_block_under_a_leaning_top_answer_every_rule(
    frustum: impl Fn(&mut Model) -> SolidKey,
    [frustum_alone, shared]: [f64; 2],
    [with_the_block, without]: [[usize; 3]; 2],
) {
    let block_alone = 18816.0;
    for (name, policy) in [
        ("certified", Policy::certified()),
        ("tolerant", Policy::tolerant(100.0)),
    ] {
        for (rule, swapped) in every_rule_both_ways() {
            let (volume, flat_cone_and_pipe) = match (rule, swapped) {
                (Rule::Union, _) => (frustum_alone + block_alone - shared, with_the_block),
                (Rule::Intersection, _) => (shared, without),
                (Rule::Difference, false) => (frustum_alone - shared, without),
                (Rule::Difference, true) => (block_alone - shared, with_the_block),
                (other, _) => panic!("not a rule this checks: {other:?}"),
            };
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let built = frustum(&mut model);
            let block = the_block_under_a_leaning_top(&mut model)
                .unwrap_or_else(|why| panic!("the block: {why:?}"));
            let [one, other] = if swapped {
                [block, built]
            } else {
                [built, block]
            };
            let what = format!("{name}, {rule:?}, swapped {swapped}");
            let answer = model
                .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: {why:?}"));
            let [solid] = answer.solids() else {
                panic!("{what}: {:?}", answer.solids());
            };
            assert_holds(&model, *solid, volume, &what);
            let faces: Vec<Surface> = model
                .solid(*solid)
                .expect("this model made the key")
                .faces()
                .map(|face| face.surface())
                .collect();
            let on_cones = faces
                .iter()
                .filter(|surface| matches!(surface, Surface::Cone(_)))
                .count();
            let on_pipes = faces
                .iter()
                .filter(|surface| matches!(surface, Surface::Cylinder(_)))
                .count();
            assert_eq!(
                [faces.len() - on_cones - on_pipes, on_cones, on_pipes],
                flat_cone_and_pipe,
                "{what}: {faces:?}"
            );
        }
    }
}

/// A frustum of the cone about the `z` axis whose apex is at `(0, 0, apex)`
/// and which opens up where `opening` is `1` and down where it is `−1`, with
/// its caps at heights `bottom` and `top`. A circle runs right-handed about
/// the cone's own axis, so about `−z` each one is used the other way round.
fn a_frustum_about_z(
    model: &mut Model,
    slope: f64,
    apex: f64,
    opening: f64,
    [bottom, top]: [f64; 2],
) -> rcad_topo::Result<SolidKey> {
    use Orientation::{Forward, Reversed};
    let skin = Cone::new(
        Axis::new(Point::new(0.0, 0.0, apex), Vector::new(0.0, 0.0, opening))
            .expect("z is a direction"),
        slope,
    )
    .expect("a positive slope");
    let turned = |about_up: Orientation| {
        if opening < 0.0 {
            about_up.reversed()
        } else {
            about_up
        }
    };
    let mut builder = SolidBuilder::new();
    let at = |height: f64| Point::new(slope * (height - apex) * opening, 0.0, height);
    let bottom_corner = builder.add_vertex(at(bottom))?;
    let top_corner = builder.add_vertex(at(top))?;
    let seam = builder.add_edge(bottom_corner, top_corner, EdgeRun::Straight)?;
    let bottom_circle = builder.add_edge(
        bottom_corner,
        bottom_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let top_circle = builder.add_edge(
        top_corner,
        top_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let mut cap = |height: f64, outward: f64, edge, turn| {
        let wire = a_wire(&mut builder, &[(edge, turn)])?;
        builder.add_face(
            Surface::Plane(
                Plane::from_point_and_normal(
                    Point::new(0.0, 0.0, height),
                    Vector::new(0.0, 0.0, outward),
                )
                .expect("the z axis is a unit normal"),
            ),
            &[wire],
        )
    };
    let lower = cap(bottom, -1.0, bottom_circle, turned(Reversed))?;
    let upper = cap(top, 1.0, top_circle, turned(Forward))?;
    let wire = a_wire(
        &mut builder,
        &[
            (bottom_circle, turned(Forward)),
            (seam, Forward),
            (top_circle, turned(Reversed)),
            (seam, Reversed),
        ],
    )?;
    let wall = builder.add_face(Surface::Cone(skin), &[wire])?;
    builder.add_shell(&[(lower, Forward), (upper, Forward), (wall, Forward)])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// **Corners that sweep no area and are not on one line.** A bar of radius 10
/// cut to the region above three sides: `(6, −8)` to `(2, −9)`, on to
/// `(−2, −7)`, on to `(−6, −8)`. The cap that is left runs the long way round
/// the circle from `(−6, −8)` to `(6, −8)` and back along those three sides,
/// so its four corners, as written, go round a bow tie whose two halves cancel
/// exactly, and they are not on one line. Every rule, both ways round, is
/// answered and holds what the circle and the prism say.
///
/// **But the corners do not come back as written.** Three of the four are an
/// ulp or two off (`(6, −8)` as `(5.999999999999999, −8.000000000000002)`),
/// so the bow tie the fuse writes does not cancel: a ring with these corners
/// winds one way or the other and is placed by that, not by
/// `holds_the_other_rings`. Family 1 needs corners that cancel exactly after
/// the fuse has written them. This cap is also one face that nothing joins,
/// so it never reaches `rings_round`; the next test is the probe that would.
#[test]
fn a_bar_cut_to_a_bow_tie_cap_is_answered_with_corners_that_no_longer_cancel() {
    let [a, b, c, d] = BOW_TIE_CORNERS;
    let swept = [[a, b], [b, c], [c, d], [d, a]]
        .iter()
        .map(|[p, q]| p[0] * q[1] - p[1] * q[0])
        .sum::<f64>();
    assert_eq!(swept, 0.0, "the bow tie's two halves cancel");
    let above_the_chord = 100.0 * PI - (100.0 * 0.8_f64.acos() - 48.0);
    let (round, sides) = (400.0 * PI, 6.0 * 2280.0);
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let round_bar = model
            .add_round_bar(10.0, 4.0, &Isometry::identity())
            .expect("a bar of radius ten");
        let prism = a_block(&mut model, &BOW_TIE_SIDES, -1.0, 6.0);
        let answer = fused_in_order(&mut model, [round_bar, prism], rule, swapped)
            .unwrap_or_else(|why| panic!("{rule:?}, swapped {swapped}: {why:?}"));
        let [solid] = answer[..] else {
            panic!("{rule:?}, swapped {swapped}: {answer:?}");
        };
        let shared = 4.0 * above_the_chord;
        let volume = match (rule, swapped) {
            (Rule::Union, _) => round + sides - shared,
            (Rule::Intersection, _) => shared,
            (Rule::Difference, false) => round - shared,
            (Rule::Difference, true) => sides - shared,
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        let what = format!("{rule:?}, swapped {swapped}");
        assert_holds(&model, solid, volume, &what);
        if rule == Rule::Intersection {
            let mut on_top: Vec<[f64; 3]> = Vec::new();
            for face in model.solid(solid).expect("this model made the key").faces() {
                for wire in face.wires() {
                    for coedge in wire.coedges() {
                        let at = coedge.start().point().coordinates();
                        if at[2] == 4.0 && !on_top.contains(&at) {
                            on_top.push(at);
                        }
                    }
                }
            }
            let written = BOW_TIE_CORNERS.map(|[x, y]| {
                *on_top
                    .iter()
                    .find(|at| (at[0] - x).abs() < 1e-9 && (at[1] - y).abs() < 1e-9)
                    .unwrap_or_else(|| panic!("{what}: no corner near ({x}, {y}) in {on_top:?}"))
            });
            let [a, b, c, d] = written;
            let swept = [[a, b], [b, c], [c, d], [d, a]]
                .iter()
                .map(|[p, q]| p[0] * q[1] - p[1] * q[0])
                .sum::<f64>();
            assert_ne!(written, BOW_TIE_CORNERS.map(|[x, y]| [x, y, 4.0]), "{what}");
            assert_ne!(swept, 0.0, "{what}: {written:?}");
        }
    }
}

/// The same bar with a bar of radius 2 through `(0, 4)` taken out of it first,
/// so the bow tie cap has a hole, and a block flush with it would join its
/// pieces into a face with two rings that may enclose no area: the bow tie, if
/// its corners cancelled, and the circle. Taking a ball's pocket out of the bow
/// tie instead is refused, with `AFaceTrimmedByACurveThisCannotFollow`.
///
/// **Cutting the holed bar to the bow tie is answered**, under every rule both
/// ways round. It was refused `CannotTellTheCellsOfAFaceApart` while the bar's
/// floor was cut into cells in `fuse/arrange.rs` (Q616): the hole's circle
/// agrees with every piece of the floor about every closed curve, and the piece
/// above the bow tie has the bar's arc for a side, so its corners — the bow
/// tie, which winds no times round the hole — were not its shape. Counted with
/// its arc, which bulges up past the hole from the chord `y = −8`, it winds
/// once round the hole and every piece below the bow tie none (queue row Q1053).
///
/// The hole takes `16π` out of what the bar and the prism share and nothing out
/// of the prism, so the bar less the prism is what it was without the hole, the
/// prism less the bar holds `16π` more, and the union is the same.
#[test]
fn a_holed_bar_cut_to_a_bow_tie_cap_is_answered_under_every_rule() {
    let above_the_chord = 100.0 * PI - (100.0 * 0.8_f64.acos() - 48.0);
    let (round, sides, hole) = (400.0 * PI, 6.0 * 2280.0, 16.0 * PI);
    let shared = 4.0 * above_the_chord - hole;
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let whole = model
            .add_round_bar(10.0, 4.0, &Isometry::identity())
            .expect("a bar of radius ten");
        let through = model
            .add_round_bar(2.0, 6.0, &at(0.0, 4.0, -1.0))
            .expect("a bar of radius two");
        let holed = the_one_solid(&mut model, whole, through, Rule::Difference);
        let prism = a_block(&mut model, &BOW_TIE_SIDES, -1.0, 6.0);
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, [holed, prism], rule, swapped);
        let volume = match (rule, swapped) {
            (Rule::Union, _) => round - hole + sides - shared,
            (Rule::Intersection, _) => shared,
            (Rule::Difference, false) => round - hole - shared,
            (Rule::Difference, true) => sides - shared,
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_holds(&model, the_only(answer, &what), volume, &what);
    }
}

/// **Corners that cancel exactly, written by hand: the fuse cuts the arc
/// wherever a wall of the cap would cross it, and the ring it hands
/// `rings_round` winds.** A bar of radius 20 and height 4, cut to the piece
/// between its arc from `(20, 0)` to `(0, 20)` and a staircase down through
/// `(0, 18)`, `(8, 18)`, `(8, 14)`, `(12, 14)` and `(12, 0)`, built face by face
/// ([`a_bar_cut_to`]). The area under the staircase is
/// `8·18 + 4·14 = 200`, the same as under the arc's chord `x + y = 20`, so the
/// seven corners sweep no area, and they are not on one line. Every corner is a
/// whole number and every wall square to an axis, so nothing written rounds:
/// read back off the model, the cap's corners are the seven as written, and
/// their sweep is exactly 0. The cap holds what the arc's segment holds,
/// `100π − 200`.
///
/// Against a bar of radius 2 at `(15, 8)`, flush with both of its caps or
/// standing on its top, the cap is cut in two along that bar's circle, and
/// where the two pieces are one face again the fuse joins them, so the cap
/// reaches `rings_round`. **It does not reach it with these corners.** The
/// fuse also cuts the bar's wall along every wall of the staircase that would
/// cross it, `x = 8` at `(8, √336)`, `x = 12` at `(12, 16)`, `y = 14` at
/// `(√204, 14)` and `y = 18` at `(√76, 18)`, though none of those walls reaches
/// the arc. Where pieces of one pipe meet, the corner stays
/// (`what_is_left` in `fuse/assemble.rs`), so the joined cap comes back with
/// eleven corners, four of them on the arc. Their sweep is what the corners
/// on the arc add over its chord, so the ring winds with the plane and is the
/// outline by that, before any ring enclosing nothing is looked at. Over the
/// twelve fuses `rings_round` is reached twelve times, twice in each of the six
/// whose answer keeps the cap, and each time one ring winds with the plane and
/// `holds_the_other_rings` is never asked (breakpoints on `outline =
/// Some(ring)` and on the closure that asks it, Q641, counted again for
/// Q1119).
///
/// **So a ring of one arc and straight sides does not reach `rings_round` with
/// its corners cancelling**, wherever the fuse cuts the arc along every wall
/// the way it does here. If every side ran on the centre's side of every
/// chord between corners on the arc, the corners would go round a polygon
/// that does not cross itself, which sweeps some area. So some side runs
/// between a chord and its arc. The line along it meets the chord's line only
/// once, so it leaves that piece of the disk through the arc, and the fuse
/// puts a corner there: the arc that chord stood for is not one arc of the
/// ring after all. The arc can be longer than half its circle with no corner
/// on it: the bar cut to the step from `(−8, −6)` along `y = −6` to
/// `(−1, −6)`, down to `(−1, −8)` and along `y = −8` to `(6, −8)`, the arc
/// running the long way round, built face by face, answers against a bar of
/// radius 2 standing through it, under every rule both ways round, since
/// Q1094 gave `Surface::Cylinder` a real answer in
/// `what_a_face_cut_by_part_of_a_crossing_holds` (`holds.rs`) in place of the
/// blanket `AFaceTrimmedByACurveThisCannotFollow` it fell into before
/// (`a_step_cap_whose_arc_runs_the_long_way_round_is_answered_by_the_next_fuse`,
/// `docs/decisions.md` §224.1). The same step
/// cut out of a bar by a prism comes back with a corner on the arc at
/// `(−1, √99)`, where `x = −1` crosses it, and one at `(8, −6)`, where
/// `y = −6` does (Q641's probes). Not tried: sides along other pipes, which
/// the line argument does not cover.
///
/// What each fuse answers is pinned: the union, the intersection and both
/// differences, both ways round, with the second bar in both places. The
/// second bar lies wholly inside the staircase's circle and past its walls, so
/// each answer is the staircase, `4(100π − 200)`, the bar, `16π` flush or `8π`
/// on top, their sum or difference, or nothing. All twelve are answered, and
/// each answer that keeps the cap has the eleven corners above. Five of them
/// were refused `ResultIsNotManifold { count: 1 }`: the flush union with the
/// second bar handed over first, the flush intersection both ways round, the
/// flush staircase less the bar, and the union with the bar on top both ways
/// round. They are answered since queue row Q1054 counts a flat face's rim
/// round a closed cut by its curved sides as well as its corners
/// (`say_whether_the_face_holds_each_lump_of_closed_cuts` in
/// `fuse/arrange.rs`).
#[test]
fn a_staircase_cap_whose_corners_cancel_is_cut_on_its_arc_by_its_own_walls() {
    let as_written = STAIRCASE.map(|[x, y]| [x, y, 4.0]);
    let staircase_volume = 4.0 * (100.0 * PI - 200.0);
    {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let staircase = a_bar_cut_to(&mut model, 20.0, &STAIRCASE)
            .unwrap_or_else(|why| panic!("the staircase: {why:?}"));
        assert_holds(&model, staircase, staircase_volume, "the staircase");
        let corners = the_outline_of_the_cap(&model, staircase, as_written[0]);
        assert_eq!(corners, as_written, "read back as written");
        assert_eq!(swept_by(&corners), 0.0, "the staircase's corners cancel");
    }

    let on_the_arc = [
        [8.0, 336.0_f64.sqrt()],
        [12.0, 16.0],
        [204.0_f64.sqrt(), 14.0],
        [76.0_f64.sqrt(), 18.0],
    ];
    for on_top in [false, true] {
        for (rule, swapped) in every_rule_both_ways() {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let staircase = a_bar_cut_to(&mut model, 20.0, &STAIRCASE)
                .unwrap_or_else(|why| panic!("the staircase: {why:?}"));
            let pin = if on_top {
                model.add_round_bar(2.0, 2.0, &at(15.0, 8.0, 4.0))
            } else {
                model.add_round_bar(2.0, 4.0, &at(15.0, 8.0, 0.0))
            }
            .expect("a bar of radius two");
            let what = format!("{rule:?}, swapped {swapped}, on top {on_top}");
            let answer = fused_in_order(&mut model, [staircase, pin], rule, swapped);
            let pin_volume = if on_top { 8.0 * PI } else { 16.0 * PI };
            let volume = match (on_top, rule, swapped) {
                (false, Rule::Union, _) | (true, Rule::Difference, false) => staircase_volume,
                (false, Rule::Difference, false) => staircase_volume - pin_volume,
                (true, Rule::Union, _) => staircase_volume + pin_volume,
                (false, Rule::Intersection, _) | (true, Rule::Difference, true) => {
                    assert_holds(&model, the_only(answer, &what), pin_volume, &what);
                    continue;
                }
                (false, Rule::Difference, true) | (true, Rule::Intersection, _) => {
                    let solids = answer.unwrap_or_else(|why| panic!("{what} refused: {why}"));
                    assert_eq!(solids, Vec::new(), "{what}: no solid is shared");
                    continue;
                }
                (_, other, _) => panic!("not a rule this probes: {other:?}"),
            };
            let with_the_cap = the_only(answer, &what);
            assert_holds(&model, with_the_cap, volume, &what);
            let written = the_outline_of_the_cap(&model, with_the_cap, as_written[0]);
            for corner in as_written {
                assert!(
                    written.contains(&corner),
                    "{what}: {corner:?} is not in {written:?}"
                );
            }
            for [x, y] in on_the_arc {
                assert!(
                    written
                        .iter()
                        .any(|at| (at[0] - x).abs() < 1e-12 && (at[1] - y).abs() < 1e-12),
                    "{what}: no corner at ({x}, {y}) in {written:?}"
                );
            }
            assert_eq!(written.len(), 11, "{what}: {written:?}");
            assert!(swept_by(&written) > 1.0, "{what}: {written:?}");
        }
    }
}

/// The staircase of the test above against a bar of radius 2 at `(15, 3)`
/// standing through it, height 6 from `z = −1`, is answered under every rule
/// both ways round. It was refused `CannotTellTheCellsOfAFaceApart`, for the
/// reason the holed bar cut to Q616's bow tie was
/// (`a_holed_bar_cut_to_a_bow_tie_cap_is_answered_under_every_rule`): the
/// circle's walk on a cap agrees with every piece of it, and the piece it is in
/// has the arc for a side, whose corners sweep no area. Counted with the arc it
/// winds once round the circle (queue row Q1053).
///
/// The bar is `24π` and shares `16π` with the staircase, all of the bar
/// between its caps, so the bar less the staircase is its two ends, `4π` each.
#[test]
fn a_staircase_cap_whose_corners_cancel_is_given_a_hole_under_every_rule() {
    let staircase_volume = 4.0 * (100.0 * PI - 200.0);
    let (pin, shared) = (24.0 * PI, 16.0 * PI);
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let staircase = a_bar_cut_to(&mut model, 20.0, &STAIRCASE)
            .unwrap_or_else(|why| panic!("the staircase: {why:?}"));
        let through = model
            .add_round_bar(2.0, 6.0, &at(15.0, 3.0, -1.0))
            .expect("a bar of radius two");
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, [staircase, through], rule, swapped);
        match (rule, swapped) {
            (Rule::Union, _) => assert_holds(
                &model,
                the_only(answer, &what),
                staircase_volume + pin - shared,
                &what,
            ),
            (Rule::Intersection, _) => {
                assert_holds(&model, the_only(answer, &what), shared, &what);
            }
            (Rule::Difference, false) => assert_holds(
                &model,
                the_only(answer, &what),
                staircase_volume - shared,
                &what,
            ),
            (Rule::Difference, true) => {
                for end in the_two_smaller_first(&model, answer, &what) {
                    assert_holds(&model, end, 4.0 * PI, &what);
                }
            }
            (other, _) => panic!("not a rule this probes: {other:?}"),
        }
    }
}

/// The step of [`LONG_STEP`] holds what the circle above the chord `y = −8`
/// holds, `75π + 50` on its cap, and its four corners, read back as written,
/// sweep no area. Its arc runs three quarters of the way round with no corner
/// on it. `docs/decisions.md` §224.1 is quoted here on an earlier version of
/// this test for the reason taking a bar out of this step was refused: an arc
/// longer than half its circle has its centre in front of any picker that
/// holds the whole arc. That is not what refused it. The refusal named a face
/// of the step's own round wall, and the wall is a `Surface::Cylinder`, which
/// `what_a_face_cut_by_part_of_a_crossing_holds` (holds.rs) answered for no
/// family at all before Q1094 — grouped with `Sphere` and `Torus` under one
/// blanket `AFaceTrimmedByACurveThisCannotFollow`, whatever the cut. Q1094's
/// `in_front_of_a_plane_every_side_is_behind` gives `Cylinder` a real answer:
/// a point in front of a plane that cuts the face, where every other side of
/// the face lies on or behind that plane, is not held. The wall's two straight
/// cuts are such planes, so taking a bar of radius 2 at `(3, 2)` out of it now
/// answers under every rule, both ways round.
///
/// The disk of radius 2 about `(3, 2)` lies entirely inside the step's cap,
/// clear of the arc and of both straight cuts, so the bar meets the step in
/// the full cylinder between the step's caps, `z` in `[0, 4]`: `π·2²·4 = 16π`.
/// The bar itself is `π·2²·6 = 24π`, so the union is the step plus
/// `24π − 16π = 8π`, the intersection `16π`, the step less the bar
/// `step − 16π`, and the bar less the step its two ends outside `[0, 4]`,
/// `z` in `[−1, 0]` and `[4, 5]`, `π·2²·1 = 4π` each.
#[test]
fn a_step_cap_whose_arc_runs_the_long_way_round_is_answered_by_the_next_fuse() {
    let step_volume = 4.0 * (75.0 * PI + 50.0);
    let (pin, shared) = (PI * 2.0 * 2.0 * 6.0, PI * 2.0 * 2.0 * 4.0);
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let step = a_bar_cut_to(&mut model, 10.0, &LONG_STEP)
        .unwrap_or_else(|why| panic!("the step: {why:?}"));
    assert_holds(&model, step, step_volume, "the step");
    let corners = the_outline_of_the_cap(&model, step, [6.0, -8.0, 4.0]);
    assert_eq!(
        corners,
        LONG_STEP.map(|[x, y]| [x, y, 4.0]),
        "read back as written"
    );
    assert_eq!(swept_by(&corners), 0.0, "the step's corners cancel");

    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let step = a_bar_cut_to(&mut model, 10.0, &LONG_STEP)
            .unwrap_or_else(|why| panic!("the step: {why:?}"));
        let through = model
            .add_round_bar(2.0, 6.0, &at(3.0, 2.0, -1.0))
            .expect("a bar of radius two");
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, [step, through], rule, swapped);
        match (rule, swapped) {
            (Rule::Union, _) => assert_holds(
                &model,
                the_only(answer, &what),
                step_volume + pin - shared,
                &what,
            ),
            (Rule::Intersection, _) => {
                assert_holds(&model, the_only(answer, &what), shared, &what);
            }
            (Rule::Difference, false) => {
                assert_holds(&model, the_only(answer, &what), step_volume - shared, &what)
            }
            (Rule::Difference, true) => {
                for end in the_two_smaller_first(&model, answer, &what) {
                    assert_holds(&model, end, 4.0 * PI, &what);
                }
            }
            (other, _) => panic!("not a rule this probes: {other:?}"),
        }
    }
}

/// What the corners sweep round `z`, twice over: the shoelace sum.
fn swept_by(corners: &[[f64; 3]]) -> f64 {
    (0..corners.len())
        .map(|at| {
            let [p, q] = [corners[at], corners[(at + 1) % corners.len()]];
            p[0] * q[1] - p[1] * q[0]
        })
        .sum()
}

/// The corners of the outline of `solid`'s face on `z = 4` that holds the
/// corner `start`, as its ring runs from there.
fn the_outline_of_the_cap(model: &Model, solid: SolidKey, start: [f64; 3]) -> Vec<[f64; 3]> {
    let solid = model.solid(solid).expect("this model made the key");
    let mut outlines = solid.faces().filter_map(|face| {
        let corners: Vec<[f64; 3]> = face
            .outer_wire()
            .points()
            .map(|point| point.coordinates())
            .collect();
        let from = corners.iter().position(|&corner| corner == start)?;
        corners
            .iter()
            .all(|corner| corner[2] == 4.0)
            .then(|| [&corners[from..], &corners[..from]].concat())
    });
    let outline = outlines.next().expect("a cap holds the staircase's corner");
    assert!(outlines.next().is_none(), "one cap holds it");
    outline
}

/// A round bar of `radius` about the `z` axis, height 4, cut to the piece
/// whose cap runs round `corners`: along the bar's circle anticlockwise from
/// the first to the second, picked by the side of the line between them that
/// is on the right going from the first to the second, then straight from
/// corner to corner back to the first. Built face by face, so every corner is
/// where it is written.
fn a_bar_cut_to(
    model: &mut Model,
    radius: f64,
    corners: &[[f64; 2]],
) -> rcad_topo::Result<SolidKey> {
    use Orientation::{Forward, Reversed};
    let mut builder = SolidBuilder::new();
    let mut low = Vec::with_capacity(corners.len());
    let mut high = Vec::with_capacity(corners.len());
    for &[x, y] in corners {
        low.push(builder.add_vertex(Point::new(x, y, 0.0))?);
        high.push(builder.add_vertex(Point::new(x, y, 4.0))?);
    }
    let top = Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 0.0, 1.0))
        .expect("a unit normal");
    let bottom = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("a unit normal");
    let [[from_x, from_y], [to_x, to_y]] = [corners[0], corners[1]];
    let the_arc = Picker::InFrontOf(
        HalfSpace::new(
            Vector::new(to_y - from_y, from_x - to_x, 0.0),
            from_x * to_y - from_y * to_x,
        )
        .expect("a normal with a length"),
    );
    let next = |index: usize| (index + 1) % corners.len();
    let mut up = Vec::with_capacity(corners.len());
    let mut along_the_top = Vec::with_capacity(corners.len());
    let mut along_the_bottom = Vec::with_capacity(corners.len());
    for index in 0..corners.len() {
        up.push(builder.add_edge(low[index], high[index], EdgeRun::Straight)?);
        let [run_high, run_low] = if index == 0 {
            [top, bottom].map(|plane| EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(plane),
                picked_by: smallvec![the_arc],
            })
        } else {
            [EdgeRun::Straight, EdgeRun::Straight]
        };
        along_the_top.push(builder.add_edge(high[index], high[next(index)], run_high)?);
        along_the_bottom.push(builder.add_edge(low[index], low[next(index)], run_low)?);
    }
    let mut faces = Vec::with_capacity(corners.len() + 2);
    let top_steps: Vec<_> = along_the_top.iter().map(|&edge| (edge, Forward)).collect();
    let top_wire = a_wire(&mut builder, &top_steps)?;
    faces.push(builder.add_face(Surface::Plane(top), &[top_wire])?);
    let bottom_steps: Vec<_> = along_the_bottom
        .iter()
        .rev()
        .map(|&edge| (edge, Reversed))
        .collect();
    let bottom_wire = a_wire(&mut builder, &bottom_steps)?;
    faces.push(builder.add_face(Surface::Plane(bottom), &[bottom_wire])?);
    for index in 0..corners.len() {
        let wire = a_wire(
            &mut builder,
            &[
                (along_the_bottom[index], Forward),
                (up[next(index)], Forward),
                (along_the_top[index], Reversed),
                (up[index], Reversed),
            ],
        )?;
        let surface = if index == 0 {
            Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a positive radius"))
        } else {
            let ([x, y], [to_x, to_y]) = (corners[index], corners[next(index)]);
            Surface::Plane(
                Plane::from_point_and_normal(
                    Point::new(x, y, 0.0),
                    Vector::new(to_y - y, x - to_x, 0.0),
                )
                .expect("a side with a length"),
            )
        };
        faces.push(builder.add_face(surface, &[wire])?);
    }
    let shell: Vec<_> = faces.into_iter().map(|face| (face, Forward)).collect();
    builder.add_shell(&shell)?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The corners of the staircase cap, in the order its ring runs: the arc of
/// radius 20 from `(20, 0)` to `(0, 20)`, then the six straight sides.
const STAIRCASE: [[f64; 2]; 7] = [
    [20.0, 0.0],
    [0.0, 20.0],
    [0.0, 18.0],
    [8.0, 18.0],
    [8.0, 14.0],
    [12.0, 14.0],
    [12.0, 0.0],
];

/// The bar of radius 10 cut to the step of the test above it, its arc the long
/// way round from `(6, −8)` to `(−8, −6)`: the polygon through its corners
/// crosses itself where the chord meets the riser, and the two halves cancel.
const LONG_STEP: [[f64; 2]; 4] = [[6.0, -8.0], [-8.0, -6.0], [-1.0, -6.0], [-1.0, -8.0]];

/// **A closed side that is one of several.** A tube, radius 7 less a bar of
/// radius 3 through it, and a bar of radius 1 at `(2, 0)` inside its hole,
/// touching the inner wall along the line through `(3, 0)` — the seam corner
/// of both circles. United, the hole left in the cap would be one ring
/// through `(3, 0)` twice: the inner circle and the small bar's circle, each a
/// closed side of the same ring, which `holds_the_other_rings` answers `None`
/// for. A ring can hold a closed side among others only by coming back to that
/// side's one corner, so this family needs a touch at a corner, and a touch
/// along a line is how two round solids make one.
///
/// **The touch is followed now** (queue row Q649): the line runs up the bars'
/// axis, and the small bar lies inside the hole's pipe on both sides of it. It
/// used to be refused `Undecidable` on *which way a crossing runs* under every
/// rule (Q616), the gap §191.5 measures for bars touching along a line.
///
/// So the intersection is no solid, the tube less the bar is the tube, and the
/// bar less the tube is the bar. **The union is the tube and the bar as they
/// stand**, two lumps. It used to be refused `ResultIsNotManifold { count: 4 }`
/// on the touching line, kept on both sides in both solids, until queue row
/// Q760 parted that edge into one per input face in `fuse/assemble.rs`.
///
/// **The ring above is not reached here.** The two caps on each end plane meet
/// only at the touching line's corner and share no edge, so nothing joins them
/// into one face, and each lump keeps its own cap. A foot standing on a bored
/// block's top joins it, and the next test reaches such a ring that way.
#[test]
fn a_bar_touching_a_tubes_inner_wall_is_answered_under_every_rule() {
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let outer = model
            .add_round_bar(7.0, 4.0, &Isometry::identity())
            .expect("a bar of radius seven");
        let through = model
            .add_round_bar(3.0, 6.0, &at(0.0, 0.0, -1.0))
            .expect("a bar of radius three");
        let tube = the_one_solid(&mut model, outer, through, Rule::Difference);
        let inside = model
            .add_round_bar(1.0, 4.0, &at(2.0, 0.0, 0.0))
            .expect("a bar of radius one");
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, [tube, inside], rule, swapped);
        match (rule, swapped) {
            (Rule::Union, _) => {
                let [bar, tube] = the_two_smaller_first(&model, answer, &what);
                assert_holds(&model, tube, 4.0 * 40.0 * PI, "the tube in the union");
                assert_holds(&model, bar, 4.0 * PI, "the bar in the union");
            }
            (Rule::Intersection, _) => assert_eq!(
                answer.unwrap_or_else(|why| panic!("{what} refused: {why}")),
                Vec::new(),
                "{what}: the bar is in the hole, so the two share no solid"
            ),
            (Rule::Difference, false) => assert_holds(
                &model,
                the_only(answer, &what),
                4.0 * 40.0 * PI,
                "the tube less the bar",
            ),
            (Rule::Difference, true) => assert_holds(
                &model,
                the_only(answer, &what),
                4.0 * PI,
                "the bar less the tube",
            ),
            (other, _) => panic!("not a rule this probes: {other:?}"),
        }
    }
}

/// **A closed side that is one of several, reached.** A block `[−10, 10]²`
/// from `z = 0` to `4`, bored through by a bar of radius 3 about the `z` axis,
/// and two tools: a bar of radius 1 at `(4, 0)` from `z = −1` to `5`, touching
/// the bore's wall from outside along the line through `(3, 0)`, the bore's
/// seam corner, and a foot `[−8, −5] × [4, 7]` from `z = 4` to `6` standing on
/// the block's top. The fuse cuts the top and the floor along every plane of
/// the foot's walls, and the pieces either side of those lines are one face
/// again, so both are joined. None of those planes crosses either circle: with
/// the foot at `[−8, −5] × [−2, 2]` the planes `y = ±2` put two more corners on
/// the bore's circle, and the ring winds.
///
/// **Under the union and the bored block less the tools, both ways round, the
/// joined top has the ring this family needs**: the bore's circle in two arcs,
/// split at `(0, 3)`, and the small bar's circle as one closed side whose one
/// corner is `(3, 0)`, where the ring comes back. With a print of each joined
/// face's rings in `rings_round` (Q802), the union with the block handed over
/// first gives, for the top,
///
/// ```text
/// members 8 ring winds Ok(Zero): [3.0, 0.0, 4.0] -> [0.0, 3.0, 4.0] -> [3.0, 0.0, 4.0] CLOSED
/// ```
///
/// and the same ring on the floor, split at `(0, −3)`. Its corners sweep
/// nothing. The outline is the block's square, with corners where the foot's
/// planes meet the block's walls, and it winds with the plane, so the ring is a
/// hole by that and `holds_the_other_rings` is not asked. The answer keeps the
/// ring as one wire of the top, through `(3, 0, 4)` twice.
///
/// The bored block holds `1600 − 36π`. Of the small bar's `6π`, `4π` lies in
/// the block and `π` past each end; the foot, 18, shares nothing. So the union
/// is `1618 − 34π`, the intersection `4π`, the bored block less the tools
/// `1600 − 40π`, and the tools less the block the bar's two ends, `π` each,
/// and the foot.
#[test]
fn a_bar_touching_a_blocks_bore_at_its_seam_leaves_one_hole_through_that_corner_twice() {
    let seam = [3.0, 0.0, 4.0];
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let (bored, tools) = a_bored_block_a_bar_touching_its_bore_and_a_foot(&mut model);
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_groups_in_order(&mut model, &[bored], &tools, rule, swapped);
        match (rule, swapped) {
            (Rule::Union, _) => {
                let union = the_only(answer, &what);
                assert_holds(&model, union, 1618.0 - 34.0 * PI, &what);
                assert!(
                    a_flat_wire_comes_back_round_a_closed_side(&model, union, seam),
                    "{what}"
                );
            }
            (Rule::Intersection, _) => {
                assert_holds(&model, the_only(answer, &what), 4.0 * PI, &what);
            }
            (Rule::Difference, false) => {
                let left = the_only(answer, &what);
                assert_holds(&model, left, 1600.0 - 40.0 * PI, &what);
                assert!(
                    a_flat_wire_comes_back_round_a_closed_side(&model, left, seam),
                    "{what}"
                );
            }
            (Rule::Difference, true) => {
                let solids = the_smaller_first(&model, answer, &what);
                let [one_end, other_end, foot] = solids[..] else {
                    panic!("{what}: {solids:?}");
                };
                assert_holds(&model, one_end, PI, &what);
                assert_holds(&model, other_end, PI, &what);
                assert_holds(&model, foot, 18.0, &what);
            }
            (other, _) => panic!("not a rule this probes: {other:?}"),
        }
    }
}

/// **Two such rings on one face, and none that winds: refused before the
/// rings are placed.** A bar of radius 7 and height 4 bored by a bar of radius
/// 1 at `(−3, 0)`, and three tools: a bar of radius 3 at `(4, 0)` touching the
/// big bar's wall from inside along the line through `(7, 0)`, the seam corner
/// of both; a bar of radius 1 at `(−1, 0)` touching the bore from outside along
/// the line through `(−2, 0)`, the bore's seam corner, both from `z = −1` to
/// `5`; and a puck of radius 1 at `(0, 4)` from `z = 4` to `6`, standing on the
/// top so the fuse cuts it along the puck's circle and joins it back. The top
/// of the bored bar less the tools would be one joined face with two rings:
/// the big bar's circle and the radius-3 bar's, both closed at `(7, 0)`, and
/// the bore's circle and the radius-1 bar's through `(−2, 0)`. Neither winds
/// round any area and `holds_the_other_rings` answers `None` for both, so
/// `rings_round` would refuse the fuse `CannotPlaceTheRingsOfAJoinedFlatFace`,
/// as it does the cap along four pipes with its own small shape through it.
///
/// **Every rule, both ways round, is refused before that**: `Undecidable`, on
/// `a point inside a piece of a face` in `select.rs`, whose record names the
/// corners of the top's piece outside the puck, `(7, 0, 4)` twice, `(−2, 0, 4)`,
/// `(−3, 1, 4)`, `(−2, 0, 4)` and `(0, 5, 4)` (Q802). The bar unbored, less the
/// radius-3 bar and a puck at `(−3, 0)`, whose one such ring would be the
/// outline, is refused the same way, its record naming the floor's piece with
/// `(7, 0, 0)` twice; the same bar less the radius-3 bar alone answers every
/// rule. Probed with a foot of straight walls, `[−1, 1] × [3, 5]`, instead of
/// the puck, the foot's planes put corners on the circles and the rings wind;
/// the union and the bored bar less the tools were then refused `Undecidable`
/// on `area swept by a wire round a normal`, for a ring on the floor with
/// `(−2, 0, 0)` twice among its four corners (not pinned).
///
/// The answers exist. The bored bar holds `192π`; of the radius-3 bar's `54π`,
/// `36π` is inside it, and of the radius-1 bar's `6π`, `4π`; the puck, `2π`,
/// shares nothing. So the union is `214π`, the intersection two solids of `36π`
/// and `4π`, the bored bar less the tools `152π`, and the tools less the bar
/// the two bars' ends, `9π` and `π` each, and the puck.
#[test]
fn a_bored_bar_touched_inside_its_wall_and_beside_its_bore_is_refused_before_its_rings_are_placed()
{
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let (bored, tools) = a_bored_bar_touched_at_both_seams_and_a_puck(&mut model);
        let what = format!("{rule:?}, swapped {swapped}");
        let refused = fused_groups_in_order(&mut model, &[bored], &tools, rule, swapped);
        let Err(Error::Undecidable { records }) = refused else {
            panic!("{what}: {refused:?}");
        };
        assert!(
            !records.records().is_empty()
                && records
                    .records()
                    .iter()
                    .all(|record| record.predicate() == "a point inside a piece of a face"),
            "{what}: {records:?}"
        );
    }
}

/// The block `[−10, 10]²` from `z = 0` to `4` bored through by a bar of radius
/// 3, and the tools: a bar of radius 1 at `(4, 0)` from `z = −1` to `5`, and the
/// foot `[−8, −5] × [4, 7]` from `z = 4` to `6`.
fn a_bored_block_a_bar_touching_its_bore_and_a_foot(
    model: &mut Model,
) -> (SolidKey, [SolidKey; 2]) {
    let block = a_block(
        model,
        &[[-10.0, -10.0], [10.0, -10.0], [10.0, 10.0], [-10.0, 10.0]],
        0.0,
        4.0,
    );
    let bore = model
        .add_round_bar(3.0, 6.0, &at(0.0, 0.0, -1.0))
        .expect("a bar of radius three");
    let bored = the_one_solid(model, block, bore, Rule::Difference);
    let touching = model
        .add_round_bar(1.0, 6.0, &at(4.0, 0.0, -1.0))
        .expect("a bar of radius one");
    let foot = a_block(
        model,
        &[[-8.0, 4.0], [-5.0, 4.0], [-5.0, 7.0], [-8.0, 7.0]],
        4.0,
        2.0,
    );
    (bored, [touching, foot])
}

/// The bar of radius 7 bored at `(−3, 0)`, and the tools: the bars of radius 3
/// at `(4, 0)` and radius 1 at `(−1, 0)`, from `z = −1` to `5`, and the puck.
fn a_bored_bar_touched_at_both_seams_and_a_puck(model: &mut Model) -> (SolidKey, [SolidKey; 3]) {
    let bar = model
        .add_round_bar(7.0, 4.0, &Isometry::identity())
        .expect("a bar of radius seven");
    let bore = model
        .add_round_bar(1.0, 6.0, &at(-3.0, 0.0, -1.0))
        .expect("a bar of radius one");
    let bored = the_one_solid(model, bar, bore, Rule::Difference);
    let inside_the_wall = model
        .add_round_bar(3.0, 6.0, &at(4.0, 0.0, -1.0))
        .expect("a bar of radius three");
    let beside_the_bore = model
        .add_round_bar(1.0, 6.0, &at(-1.0, 0.0, -1.0))
        .expect("a bar of radius one");
    let puck = model
        .add_round_bar(1.0, 2.0, &at(0.0, 4.0, 4.0))
        .expect("a puck of radius one");
    (bored, [inside_the_wall, beside_the_bore, puck])
}

/// Whether a wire of a flat face of `solid` runs a closed side round from
/// `corner` and leaves `corner` along one other side too: one ring through
/// that corner twice.
fn a_flat_wire_comes_back_round_a_closed_side(
    model: &Model,
    solid: SolidKey,
    corner: [f64; 3],
) -> bool {
    model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .filter(|face| matches!(face.surface(), Surface::Plane(_)))
        .any(|face| {
            face.wires().any(|wire| {
                let sides: Vec<[[f64; 3]; 2]> = wire
                    .coedges()
                    .map(|side| [side.start(), side.end()].map(|end| end.point().coordinates()))
                    .collect();
                sides.contains(&[corner, corner])
                    && sides.iter().filter(|[start, _]| *start == corner).count() == 2
            })
        })
}

fn the_only(answer: rcad_topo::Result<Vec<SolidKey>>, what: &str) -> SolidKey {
    let solids = answer.unwrap_or_else(|why| panic!("{what} refused: {why}"));
    assert_eq!(solids.len(), 1, "{what}: {solids:?}");
    solids[0]
}

/// The two solids of an answer, the one with less inside it first.
fn the_two_smaller_first(
    model: &Model,
    answer: rcad_topo::Result<Vec<SolidKey>>,
    what: &str,
) -> [SolidKey; 2] {
    the_smaller_first(model, answer, what)
        .try_into()
        .unwrap_or_else(|solids| panic!("{what}: two lumps, got {solids:?}"))
}

/// The solids of an answer, the ones with less inside them first.
fn the_smaller_first(
    model: &Model,
    answer: rcad_topo::Result<Vec<SolidKey>>,
    what: &str,
) -> Vec<SolidKey> {
    let mut solids = answer.unwrap_or_else(|why| panic!("{what} refused: {why}"));
    let volume = |solid: &SolidKey| {
        model
            .solid(*solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a fine mesh"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} does not measure: {why}"))
            .volume()
    };
    solids.sort_by(|one, other| volume(one).total_cmp(&volume(other)));
    solids
}

/// The corners of the bow tie cap, in the order its ring runs: two on the
/// circle of radius 10, then two inside it.
const BOW_TIE_CORNERS: [[f64; 2]; 4] = [[-6.0, -8.0], [6.0, -8.0], [2.0, -9.0], [-2.0, -7.0]];

/// A region of area 2280 whose sides through `(2, −9)` and `(−2, −7)` are
/// three of the bow tie's, carried out past the circle of radius 10.
const BOW_TIE_SIDES: [[f64; 2]; 6] = [
    [30.0, -2.0],
    [30.0, 30.0],
    [-30.0, 30.0],
    [-30.0, -14.0],
    [-2.0, -7.0],
    [2.0, -9.0],
];

fn every_rule_both_ways() -> impl Iterator<Item = (Rule, bool)> {
    [Rule::Union, Rule::Intersection, Rule::Difference]
        .into_iter()
        .flat_map(|rule| [false, true].map(move |swapped| (rule, swapped)))
}

/// Two bars of radius 10 and height 4 at `−apart` and `apart`, each less a bar
/// of radius 2 standing through the origin.
fn two_bars_holed_through_the_origin(model: &mut Model, [x, y]: [f64; 2]) -> [SolidKey; 2] {
    [-1.0, 1.0].map(|side| {
        let whole = model
            .add_round_bar(10.0, 4.0, &at(side * x, side * y, 0.0))
            .expect("a bar of radius ten");
        let hole = model
            .add_round_bar(2.0, 6.0, &at(0.0, 0.0, -1.0))
            .expect("a bar of radius two");
        the_one_solid(model, whole, hole, Rule::Difference)
    })
}

fn a_block(model: &mut Model, outline: &[[f64; 2]], bottom: f64, height: f64) -> SolidKey {
    let profile = ProfileBuilder::new(outline.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a simple outline");
    model
        .add_extrusion(&profile, height, &at(0.0, 0.0, bottom))
        .expect("a block")
}

fn fused_in_order(
    model: &mut Model,
    [one, other]: [SolidKey; 2],
    rule: Rule,
    swapped: bool,
) -> rcad_topo::Result<Vec<SolidKey>> {
    fused_groups_in_order(model, &[one], &[other], rule, swapped)
}

fn fused_groups_in_order(
    model: &mut Model,
    one: &[SolidKey],
    other: &[SolidKey],
    rule: Rule,
    swapped: bool,
) -> rcad_topo::Result<Vec<SolidKey>> {
    let [one, other] = if swapped { [other, one] } else { [one, other] };
    model
        .fuse(
            one,
            other,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().to_vec())
}

/// **A hole along a cone's hyperbola, closed by a ball, has its rings placed**
/// (Q1233, Q1289). The nested cups reach `rings_round` with an
/// outline that winds, because the second cone's base plane `z = 16` puts two
/// corners on the cut cone's hyperbola
/// ([`the_nested_cups_and_a_block_flush_with_their_wall_are_answered_under_every_rule`]).
/// A plane cannot close the second cone's cup on `x = −12` without doing the
/// same: the outline there is the hyperbola and its chord on `z = 18`, a line
/// across the cup leaves that outline on both sides, and it meets the chord
/// once at most, so it leaves through the hyperbola at least once.
///
/// So here a ball closes the cup: the second cone, apex at `z = 3`, run on
/// past the top, cut by a ball of radius `√148` about `(0, 0, 15)`. The ball
/// crosses the cut cone's skin only in circles of radius under 12, so on
/// `x = −12` it is the circle of radius 2 about `z = 15`, clear of the outline,
/// and the hole is that circle's arc above the second cone's hyperbola, two
/// corners, with an outline of two corners: the face `holds_the_other_rings`
/// answered `None` for twice. The cone cut by the ball holds the cone up to where
/// the ball crosses it, `w = 6 + √38` above the apex, and the ball's cap above
/// that.
///
/// **Every rule is answered both ways round, and the union reaches
/// `rings_round` with this hole on the cut cone's wall**, the closed cone
/// poking out through the wall and through the cut cone's top `z = 18`, which
/// the ball crosses in a circle of radius `√139`. So the union holds the cut
/// cone, the cap above its top and the sliver past its wall; the intersection
/// the closed cone less those two; the cut cone less the closed cone the
/// difference of the two, with those two put back; and the closed cone less
/// the cut cone the cap and the sliver, two solids apart, since the ball
/// reaches `x = −12` only between `z = 13` and `17`.
///
/// The hole's corners are where the plane `x = −12`, the second cone and the
/// ball meet, which `rcad-geom` answers since Q1271. The meet of the ball and
/// the two cones, three curved surfaces and no plane, was refused next, and is
/// answered since Q1284: the two cones are of one slope about one axis, so
/// they differ by the plane `z = 1.5`, which the ball, `13.5` below its centre
/// against a radius of `√148`, does not reach, and the three share nothing.
/// After that every rule was refused `AFaceTrimmedByACurveThisCannotFollow` on
/// the ball's cap, until `holds.rs` read a point of a ball past the picker
/// (Q1218, Q1289). Then the union panicked in `rings_round`, its wall's outline
/// and hole each two corners with a side along a hyperbola, which
/// `holds_the_other_rings` could not place until it read an arc of a curve
/// that does not close (Q1289).
#[test]
fn a_cut_cone_and_a_cone_closed_by_a_ball_are_answered_under_every_rule() {
    let (crossing, radius) = (6.0 + 38.0_f64.sqrt(), 148.0_f64.sqrt());
    let cap_height = radius + 12.0 - crossing;
    let closed_by_the_ball = PI * crossing.powi(3) / 3.0
        + PI * cap_height * cap_height * (3.0 * radius - cap_height) / 3.0;
    let above_the_top = PI * (radius - 3.0).powi(2) * (2.0 * radius + 3.0) / 3.0;
    let past_the_wall = what_the_closed_cone_holds_past_the_wall();
    let cut = what_the_block_holds_of_the_cone();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let [_, closed] = a_cut_cone_and_a_cone_closed_by_a_ball(&mut model);
    assert_holds(
        &model,
        closed,
        closed_by_the_ball,
        "the cone the ball closes",
    );

    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let pair = a_cut_cone_and_a_cone_closed_by_a_ball(&mut model);
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, pair, rule, swapped);
        let volume = match (rule, swapped) {
            (Rule::Union, _) => cut + above_the_top + past_the_wall,
            (Rule::Intersection, _) => closed_by_the_ball - above_the_top - past_the_wall,
            (Rule::Difference, false) => cut - closed_by_the_ball + above_the_top + past_the_wall,
            (Rule::Difference, true) => {
                let [sliver, cap] = the_two_smaller_first(&model, answer, &what);
                assert_holds(
                    &model,
                    sliver,
                    past_the_wall,
                    &format!("{what}, the sliver"),
                );
                assert_holds(&model, cap, above_the_top, &format!("{what}, the cap"));
                continue;
            }
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        assert_holds(&model, the_only(answer, &what), volume, &what);
    }
}

/// The part of the closed cone past the wall `x = −12`: the second cone's part
/// from `z = 15`, where its radius `u = z − 3` reaches 12, to where the ball
/// closes it, `u = w = 6 + √38`, and then the ball's part above that.
///
/// The cone's part is [`what_the_second_cone_holds_past_the_wall`]'s integral
/// from 12 to `w`. The ball's part is the piece of a ball of radius `R = √148`
/// past two planes square to each other, `|x| > a = 12` and `z − 15 > b = w −
/// 12`: slicing it across `x`, the slice is the circle of radius `√(R² − x²)`
/// cut by a chord `b` from its centre, which is
/// `(R² − x²) acos(b/√(R² − x²)) − b√(k² − x²)` for `k² = R² − b² = w²`.
/// Integrated by parts, the first term leaves `(b/3)∫(x² − 2R² + 2R⁴/(R² −
/// x²))/√(k² − x²) dx`, all three standard, so from `a` to `k` it is `P(k) −
/// P(a)` for
///
/// ```text
/// P(x) = (R²x − x³/3) acos(b/√(R² − x²)) − (b(3R² − b²)/3) asin(x/k)
///        − (2b/3) x√(k² − x²) + (2R³/3) atan(bx/(R√(k² − x²)))
/// ```
///
/// with `P(k) = πR³/3 − πb(3R² − b²)/6`. At `b = 0` it is half a cap of height
/// `R − a`, as it should be, and a midpoint sum of the slices across `z` in
/// 200 000 steps agrees with the whole to 2e-11 (Q1289).
fn what_the_closed_cone_holds_past_the_wall() -> f64 {
    let past_the_wall_of_a_cone = |u: f64| {
        let rise = (u * u - 144.0).sqrt();
        u.powi(3) / 3.0 * (12.0 / u).acos() - 8.0 * u * rise + 576.0 * (u + rise).ln()
    };
    let (squared, wide) = (148.0_f64, 6.0 + 38.0_f64.sqrt());
    let (radius, up) = (squared.sqrt(), wide - 12.0);
    let across = |x: f64| {
        let left = (wide * wide - x * x).sqrt();
        (squared * x - x.powi(3) / 3.0) * (up / (squared - x * x).sqrt()).acos()
            - up * (3.0 * squared - up * up) / 3.0 * (x / wide).asin()
            - 2.0 * up / 3.0 * x * left
            + 2.0 * radius.powi(3) / 3.0 * (up * x / (radius * left)).atan()
    };
    let at_the_far_end = PI * radius.powi(3) / 3.0 - PI * up * (3.0 * squared - up * up) / 6.0;
    past_the_wall_of_a_cone(wide) - past_the_wall_of_a_cone(12.0) + at_the_far_end - across(12.0)
}

/// The cut cone, and the second cone with its apex at `z = 3` cut by the ball
/// of radius `√148` about `(0, 0, 15)`.
fn a_cut_cone_and_a_cone_closed_by_a_ball(model: &mut Model) -> [SolidKey; 2] {
    let [cone, block] = a_cone_and_a_block_across_it(model);
    let cut = the_one_solid(model, cone, block, Rule::Intersection);
    let second = cones::a_pointed_cone(model, 1.0, 26.0);
    model
        .place_solid(second, &at(0.0, 0.0, 3.0))
        .expect("this model made the key");
    let ball = model
        .add_ball(148.0_f64.sqrt(), &at(0.0, 0.0, 15.0))
        .expect("a ball of radius √148");
    let closed = the_one_solid(model, second, ball, Rule::Intersection);
    [cut, closed]
}

/// **Corners that cancel exactly and sides along pipes** (Q1120). A bar of
/// radius 10 cut to the cap from `(10, 0)` anticlockwise round its circle to
/// `(−8, 6)`, then clockwise round three pipes: radius 5 about `(−5, 2)` to
/// `(−2, 6)`, radius 6 about `(−2, 0)` to `(4, 0)`, and radius 5 about
/// `(7, −4)` back to `(10, 0)`, the cap outside each, built face by face
/// ([`a_bar_cut_to_pipes`]). The four corners sweep `60 − 36 − 24 + 0 = 0`, and
/// they are not on one line. No side is straight: the line along a straight
/// side next to `(−2, 6)`, which lies between the chord `y = (10 − x)/3` and the
/// arc, would leave through the arc (the staircase above), and each of these
/// pipes crosses the bar's circle nowhere on its arc. The cap holds the arc's
/// segment less the three pipes' segments, each read off its angle.
///
/// **Read back, the corners are the four as written and sweep exactly 0.**
/// Against a bar of radius 2 at `(3, 7)` standing on the top, every rule both
/// ways round answers what the volumes say, and the cap less that bar reaches
/// `rings_round` with this ring and no other: it winds round nothing, so it is
/// the outline by the rule for one such ring, and `holds_the_other_rings` is not
/// asked (a temporary print in `rings_round`, Q1120). The fuse puts no corner on
/// the arc here.
///
/// **Flush with the cap, the same bar is answered under every rule, both ways
/// round** (row Q1252). It stands inside the cap from `z = 0` to `4`: its
/// circle reaches `√58 + 2 < 10` from the bar's axis, and stays outside each
/// pipe, `√137`, `√74` and `√89` from their axes against `7`, `8` and `7`. So
/// the union is the cap, the intersection is the bar, `16π`, the cap less the
/// bar is the cap with a hole through it, and the bar less the cap is nothing.
///
/// Before, every one of those was refused `AFaceTrimmedByACurveThisCannotFollow`
/// on the cap's wall on the bar's circle, from `select.rs`: a ray from a point
/// inside the bar meets that wall, and `face_holds_the_point` refused every
/// point of it. That wall is cut by its floor, its roof and the two pipes its
/// straight sides run along, and the part of the bar's circle outside both
/// pipes is two runs, the wall's through `(0, 10)` and another through
/// `(0, −10)`. `holds.rs` now tells the two apart, and `(0, −10, 2)`, in front
/// of every cut the wall has, is not on it.
#[test]
fn a_cap_along_four_pipes_whose_corners_cancel_is_answered_with_a_bar_on_top_or_flush() {
    let cap_volume = what_the_cap_along_four_pipes_holds();
    let bar_wall = Surface::Cylinder(Cylinder::new(Axis::z(), 10.0).expect("a positive radius"));
    let as_written = PIPE_CAP.map(|[x, y]| [x, y, 4.0]);
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cap = a_bar_cut_to_pipes(&mut model, 10.0, &PIPE_CAP, &PIPE_CAP_SIDES, AS_WRITTEN)
        .unwrap_or_else(|why| panic!("the cap: {why:?}"));
    assert_holds(&model, cap, cap_volume, "the cap");
    let corners = the_outline_of_the_cap(&model, cap, as_written[0]);
    assert_eq!(corners, as_written, "read back as written");
    assert_eq!(swept_by(&corners), 0.0, "the cap's corners cancel");
    let solid = model.solid(cap).expect("this model made the key");
    let wall = solid
        .faces()
        .find(|face| face.surface() == bar_wall)
        .expect("the cap has a wall on the bar's circle");
    for (point, held) in [
        ([0.0, 10.0, 2.0], OnAFace::Held),
        ([0.0, -10.0, 2.0], OnAFace::NotHeld),
        ([10.0, 0.0, 2.0], OnAFace::OnItsBoundary),
        ([0.0, 10.0, 4.0], OnAFace::OnItsBoundary),
        ([0.0, 10.0, 5.0], OnAFace::NotHeld),
    ] {
        let asked = solid.face_holds_the_point(
            wall.key(),
            rcad_predicates::Point::from(point),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert_eq!(
            asked.unwrap_or_else(|why| panic!("the wall at {point:?} refused: {why}")),
            held,
            "the wall at {point:?}"
        );
    }

    let on_top = 8.0 * PI;
    let inside = 16.0 * PI;
    for flush in [true, false] {
        for (rule, swapped) in every_rule_both_ways() {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let cap = a_bar_cut_to_pipes(&mut model, 10.0, &PIPE_CAP, &PIPE_CAP_SIDES, AS_WRITTEN)
                .unwrap_or_else(|why| panic!("the cap: {why:?}"));
            let bar = if flush {
                model.add_round_bar(2.0, 4.0, &at(3.0, 7.0, 0.0))
            } else {
                model.add_round_bar(2.0, 2.0, &at(3.0, 7.0, 4.0))
            }
            .expect("a bar of radius two");
            let what = format!("{rule:?}, swapped {swapped}, flush {flush}");
            let answer = fused_in_order(&mut model, [cap, bar], rule, swapped);
            if flush {
                match (rule, swapped) {
                    (Rule::Union, _) => {
                        assert_holds(&model, the_only(answer, &what), cap_volume, &what);
                    }
                    (Rule::Intersection, _) => {
                        assert_holds(&model, the_only(answer, &what), inside, &what);
                    }
                    (Rule::Difference, false) => {
                        let left = the_only(answer, &what);
                        assert_holds(&model, left, cap_volume - inside, &what);
                        let corners = the_outline_of_the_cap(&model, left, as_written[0]);
                        assert_eq!(corners, as_written, "{what}");
                    }
                    (Rule::Difference, true) => assert_eq!(
                        answer.unwrap_or_else(|why| panic!("{what} refused: {why}")),
                        Vec::new(),
                        "{what}: the bar is inside the cap"
                    ),
                    (other, _) => panic!("not a rule this probes: {other:?}"),
                }
                continue;
            }
            match (rule, swapped) {
                (Rule::Union, _) => {
                    assert_holds(&model, the_only(answer, &what), cap_volume + on_top, &what)
                }
                (Rule::Intersection, _) => assert_eq!(
                    answer.unwrap_or_else(|why| panic!("{what} refused: {why}")),
                    Vec::new(),
                    "{what}: the two share no solid"
                ),
                (Rule::Difference, false) => {
                    let left = the_only(answer, &what);
                    assert_holds(&model, left, cap_volume, &what);
                    let corners = the_outline_of_the_cap(&model, left, as_written[0]);
                    assert_eq!(corners, as_written, "{what}");
                }
                (Rule::Difference, true) => {
                    assert_holds(&model, the_only(answer, &what), on_top, &what);
                }
                (other, _) => panic!("not a rule this probes: {other:?}"),
            }
        }
    }
}

/// The cap along four pipes from `z = 0` to `4`: the arc's segment less the
/// three pipes' segments, each read off its angle, four high.
fn what_the_cap_along_four_pipes_holds() -> f64 {
    let segment = |radius: f64, angle: f64| radius * radius / 2.0 * (angle - angle.sin());
    let small_angle = (7.0_f64 / 25.0).acos();
    4.0 * (segment(10.0, PI - 0.75_f64.atan())
        - 2.0 * segment(5.0, small_angle)
        - segment(6.0, PI / 2.0))
}

/// **The cap along four pipes with its own shape, an eighth the size, standing
/// through it** (Q1253). The small cap's corners are `(−2, 8)` plus an eighth
/// of the big one's, and so are its pipes, and it stands from `z = −1` to `5`.
/// Every number is a sum of eighths, so its corners are where they are written
/// and also sweep exactly 0. It is clear of every side of the big cap: its
/// circle reaches `√68 + 5/4 < 10` from the big cap's axis, and stays `√45 −
/// 5/4 > 5` and `8 − 5/4 > 6` from the axes of the two pipes nearest it.
///
/// So the intersection is the small cap from `z = 0` to `4`, a 64th of the big
/// one, and the small cap less the big one is its two ends, `z = −1` to `0` and
/// `4` to `5`, each a 256th of it. **The union and the big cap less the small
/// one are refused `CannotPlaceTheRingsOfAJoinedFlatFace`**, both ways round:
/// each joins the big cap's top out of pieces into a face with two rings, the
/// big cap's outline and the small one's, and both wind round nothing. Neither
/// runs along one surface and neither has its corners on one line, so
/// `holds_the_other_rings` places neither, and nothing says which is the
/// outline. That was a panic in `rings_round` until Q1253. The answers exist:
/// the union holds the big cap and a 128th more, the difference the big cap
/// less a 64th.
#[test]
fn a_cap_along_four_pipes_with_its_own_shape_an_eighth_the_size_through_it_is_refused_where_its_top_is_joined()
 {
    let cap_volume = what_the_cap_along_four_pipes_holds();
    let small = Placed {
        by: 0.125,
        centre: [-2.0, 8.0],
        bottom: -1.0,
        top: 5.0,
    };
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let cap = a_bar_cut_to_pipes(&mut model, 10.0, &PIPE_CAP, &PIPE_CAP_SIDES, AS_WRITTEN)
            .unwrap_or_else(|why| panic!("the cap: {why:?}"));
        let through = a_bar_cut_to_pipes(&mut model, 10.0, &PIPE_CAP, &PIPE_CAP_SIDES, small)
            .unwrap_or_else(|why| panic!("the small cap: {why:?}"));
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = fused_in_order(&mut model, [cap, through], rule, swapped);
        match (rule, swapped) {
            (Rule::Union, _) | (Rule::Difference, false) => assert!(
                matches!(
                    answer,
                    Err(Error::CannotPlaceTheRingsOfAJoinedFlatFace { .. })
                ),
                "{what}: {answer:?}"
            ),
            (Rule::Intersection, _) => {
                assert_holds(&model, the_only(answer, &what), cap_volume / 64.0, &what);
            }
            (Rule::Difference, true) => {
                for end in the_two_smaller_first(&model, answer, &what) {
                    assert_holds(&model, end, cap_volume / 256.0, &what);
                }
            }
            (other, _) => panic!("not a rule this probes: {other:?}"),
        }
    }
}

/// **One arc, two pipes and a leaning line** (Q1254). The cap along four pipes
/// above with its middle side, round the pipe of radius 6 about `(−2, 0)`, made
/// straight: from `(−2, 6)` to `(4, 0)` along `x + y = 4`, on the plane whose
/// normal is `(−1, −1, 0)/√2`. That is the side to straighten because it is the
/// only one of the three whose corners share neither an `x` nor a `y`: the
/// other two, straight, would lie on `y = 6` and `y = 0`, square to an axis.
/// The corners are the four as before, so they still sweep exactly 0.
///
/// The cap holds the arc's segment less the two pipes' segments, each read off
/// its angle; a straight side cuts no segment. The arc turns `π − atan(3/4)`,
/// whose sine is `3/5`, and each pipe `acos(7/25) = 2·atan(3/4)`, whose sine is
/// `24/25`. So the cap's face is
///
/// ```text
/// 50(π − atan(3/4) − 3/5) − 2·(25/2)(2·atan(3/4) − 24/25) = 50π − 100·atan(3/4) − 6
/// ```
///
/// and the cap, 4 high, `200π − 400·atan(3/4) − 24`, about 346.92: the cap
/// along four pipes with the radius-6 pipe's segment, `4·18(π/2 − 1)`, given
/// back.
///
/// **The line along the straight side leaves the cap through its arc**, past
/// `(−2, 6)`, at `(2 − √46, 2 + √46)`: `(−3, 7)` is on it, inside the circle
/// and outside the pipe about `(−5, 2)`. That is the sentence the cap along four
/// pipes gives for having no straight side next to `(−2, 6)`. The fuse puts a
/// corner on the arc there, as on the staircase's, so every answer that keeps
/// the cap reads its top back as the four corners as written with that one
/// after `(10, 0)`, and those five sweep more than nothing.
///
/// A bar of radius 2 at `(3, 7)`, as for the four pipes, stays clear of every
/// side: its circle reaches `√58 + 2 < 10` from the bar's axis, is `√89` and
/// `√137` from the two pipes' axes against `7`, and `3√2 > 2` from the line, on
/// the cap's side, so it is clear of the fuse's cut along the line as well. On
/// top, from `z = 4` to `6`, it is `8π`: the union is the cap and the bar, the
/// cap less the bar is the cap, the bar less the cap is the bar, and the two
/// share no solid. Flush, from `z = 0` to `4`, it is `16π` inside the cap: the
/// union is the cap, the intersection the bar, the cap less the bar the cap
/// less `16π`, and the bar less the cap nothing. Every rule, both ways round,
/// answers that under `Policy::certified()`.
///
/// **Each of those twelve fuses reaches `into_the_arc_away_from_the_centre`**
/// (`holds.rs`, Q1093), and is refused `AFaceTrimmedByACurveThisCannotFollow`
/// on the cap's top with that function made to answer `None` (run for Q1254).
/// Q605's lane, on a base before Q1093, saw such a cap refused that way and did
/// not pin it. What reaches it is the two pipe sides, not the arc of the bar: a
/// print there names the pipe about `(−5, 2)` with its picker `y ≥ 6` and the
/// pipe about `(7, −4)` with `y ≥ 0`, each with the end it shares with the
/// straight side, `(−2, 6)` or `(4, 0)`, solved into a box under `10⁻¹⁴` wide
/// that the picker's plane runs through, on the top and on the floor. The cap
/// along four pipes, whose corners are where two pipes meet, still answers with
/// that function made to answer `None`.
#[test]
fn a_cap_along_three_pipes_and_a_leaning_line_is_answered_with_a_bar_on_top_or_flush() {
    let cap_volume = 200.0 * PI - 400.0 * 0.75_f64.atan() - 24.0;
    let as_written = PIPE_CAP.map(|[x, y]| [x, y, 4.0]);
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cap = a_bar_cut_to_pipes(&mut model, 10.0, &PIPE_CAP, &LEANING_CAP_SIDES, AS_WRITTEN)
        .unwrap_or_else(|why| panic!("the cap: {why:?}"));
    assert_holds(&model, cap, cap_volume, "the cap");
    let corners = the_outline_of_the_cap(&model, cap, as_written[0]);
    assert_eq!(corners, as_written, "read back as written");
    assert_eq!(swept_by(&corners), 0.0, "the cap's corners cancel");

    let [leaves_x, leaves_y] = [2.0 - 46.0_f64.sqrt(), 2.0 + 46.0_f64.sqrt()];
    for flush in [true, false] {
        for (rule, swapped) in every_rule_both_ways() {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let cap =
                a_bar_cut_to_pipes(&mut model, 10.0, &PIPE_CAP, &LEANING_CAP_SIDES, AS_WRITTEN)
                    .unwrap_or_else(|why| panic!("the cap: {why:?}"));
            let bar = if flush {
                model.add_round_bar(2.0, 4.0, &at(3.0, 7.0, 0.0))
            } else {
                model.add_round_bar(2.0, 2.0, &at(3.0, 7.0, 4.0))
            }
            .expect("a bar of radius two");
            let what = format!("{rule:?}, swapped {swapped}, flush {flush}");
            let answer = fused_in_order(&mut model, [cap, bar], rule, swapped);
            let bar_volume = if flush { 16.0 * PI } else { 8.0 * PI };
            let volume = match (flush, rule, swapped) {
                (true, Rule::Union, _) | (false, Rule::Difference, false) => cap_volume,
                (true, Rule::Difference, false) => cap_volume - bar_volume,
                (false, Rule::Union, _) => cap_volume + bar_volume,
                (true, Rule::Intersection, _) | (false, Rule::Difference, true) => {
                    assert_holds(&model, the_only(answer, &what), bar_volume, &what);
                    continue;
                }
                (true, Rule::Difference, true) | (false, Rule::Intersection, _) => {
                    let solids = answer.unwrap_or_else(|why| panic!("{what} refused: {why}"));
                    assert_eq!(solids, Vec::new(), "{what}: nothing is left");
                    continue;
                }
                (_, other, _) => panic!("not a rule this probes: {other:?}"),
            };
            let with_the_cap = the_only(answer, &what);
            assert_holds(&model, with_the_cap, volume, &what);
            let written = the_outline_of_the_cap(&model, with_the_cap, as_written[0]);
            let [first, [x, y, z], second, third, fourth] = written[..] else {
                panic!("{what}: five corners, not {written:?}");
            };
            assert_eq!([first, second, third, fourth], as_written, "{what}");
            assert!(
                (x - leaves_x).abs() < 1e-12 && (y - leaves_y).abs() < 1e-12 && z == 4.0,
                "{what}: no corner where the line leaves the arc in {written:?}"
            );
            assert!(swept_by(&written) > 1.0, "{what}: {written:?}");
        }
    }
}

/// The corners of the cap along four pipes, in the order its ring runs.
const PIPE_CAP: [[f64; 2]; 4] = [[10.0, 0.0], [-8.0, 6.0], [-2.0, 6.0], [4.0, 0.0]];

/// How each side of the cap along four pipes runs after the arc.
const PIPE_CAP_SIDES: [Side; 3] = [
    Side::RoundAPipe([-5.0, 2.0], 5.0),
    Side::RoundAPipe([-2.0, 0.0], 6.0),
    Side::RoundAPipe([7.0, -4.0], 5.0),
];

/// How each side of the cap along three pipes and a leaning line runs after
/// the arc: [`PIPE_CAP_SIDES`] with the middle one straight.
const LEANING_CAP_SIDES: [Side; 3] = [
    Side::RoundAPipe([-5.0, 2.0], 5.0),
    Side::Straight,
    Side::RoundAPipe([7.0, -4.0], 5.0),
];

/// How a side of a cap after its arc runs from its corner to the next.
#[derive(Clone, Copy)]
enum Side {
    /// Clockwise round the pipe of this centre and radius, the shorter way,
    /// with the cap outside the pipe.
    RoundAPipe([f64; 2], f64),
    /// Straight, on the plane through both corners.
    Straight,
}

/// Where a cap along pipes stands: shrunk `by` about the origin, moved to
/// `centre`, and stood from `z = bottom` up to `top`.
#[derive(Clone, Copy)]
struct Placed {
    by: f64,
    centre: [f64; 2],
    bottom: f64,
    top: f64,
}

/// A cap along pipes where its corners and pipes are written, from `z = 0` to `4`.
const AS_WRITTEN: Placed = Placed {
    by: 1.0,
    centre: [0.0, 0.0],
    bottom: 0.0,
    top: 4.0,
};

#[test]
fn probe_q1253() {
    let small = Placed {
        by: 0.125,
        centre: [-2.0, 8.0],
        bottom: -1.0,
        top: 5.0,
    };
    for (rule, swapped) in every_rule_both_ways() {
        let outcome = std::panic::catch_unwind(|| {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let cap = a_bar_cut_to_pipes(&mut model, 10.0, &PIPE_CAP, &PIPE_CAP_SIDES, AS_WRITTEN)
                .unwrap_or_else(|why| panic!("the cap: {why:?}"));
            let through = a_bar_cut_to_pipes(&mut model, 10.0, &PIPE_CAP, &PIPE_CAP_SIDES, small)
                .unwrap_or_else(|why| panic!("the small cap: {why:?}"));
            let answer = fused_in_order(&mut model, [cap, through], rule, swapped);
            let volumes: Vec<f64> = answer
                .iter()
                .flatten()
                .map(|&solid| {
                    model
                        .solid(solid)
                        .expect("key")
                        .measure(
                            Deflection::of(0.001).expect("a fine mesh"),
                            &Policy::certified(),
                            &mut ProvenanceLog::new(),
                        )
                        .map(|m| m.volume())
                        .unwrap_or(f64::NAN)
                })
                .collect();
            let valid = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
            format!("{answer:?} volumes {volumes:?} valid {valid:?}")
        });
        eprintln!("PROBE first {rule:?} swapped {swapped}: {outcome:?}");
    }
    for (rule, swapped) in every_rule_both_ways() {
        let outcome = std::panic::catch_unwind(|| {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let cap = a_bar_cut_to_pipes(&mut model, 10.0, &PIPE_CAP, &PIPE_CAP_SIDES, AS_WRITTEN)
                .unwrap_or_else(|why| panic!("the cap: {why:?}"));
            let through = a_bar_cut_to_pipes(&mut model, 10.0, &PIPE_CAP, &PIPE_CAP_SIDES, small)
                .unwrap_or_else(|why| panic!("the small cap: {why:?}"));
            let holed = the_one_solid(&mut model, cap, through, Rule::Difference);
            let bar = model
                .add_round_bar(2.0, 2.0, &at(3.0, 7.0, 4.0))
                .expect("a bar of radius two");
            let answer = fused_in_order(&mut model, [holed, bar], rule, swapped);
            format!("{answer:?}")
        });
        eprintln!("PROBE second {rule:?} swapped {swapped}: {outcome:?}");
    }
}

/// [`a_bar_cut_to`], with side `i + 1` running as `sides[i]` says, `placed`
/// where that says.
fn a_bar_cut_to_pipes(
    model: &mut Model,
    radius: f64,
    corners: &[[f64; 2]],
    sides: &[Side],
    placed: Placed,
) -> rcad_topo::Result<SolidKey> {
    use Orientation::{Forward, Reversed};
    let Placed {
        by,
        centre: [centre_x, centre_y],
        bottom: floor,
        top: roof,
    } = placed;
    let moved = |[x, y]: [f64; 2]| [centre_x + by * x, centre_y + by * y];
    let corners: Vec<[f64; 2]> = corners.iter().map(|&corner| moved(corner)).collect();
    let sides: Vec<Side> = sides
        .iter()
        .map(|&side| match side {
            Side::RoundAPipe(at, pipe) => Side::RoundAPipe(moved(at), by * pipe),
            Side::Straight => Side::Straight,
        })
        .collect();
    let mut builder = SolidBuilder::new();
    let mut low = Vec::with_capacity(corners.len());
    let mut high = Vec::with_capacity(corners.len());
    for &[x, y] in &corners {
        low.push(builder.add_vertex(Point::new(x, y, floor))?);
        high.push(builder.add_vertex(Point::new(x, y, roof))?);
    }
    let top = Plane::from_point_and_normal(Point::new(0.0, 0.0, roof), Vector::new(0.0, 0.0, 1.0))
        .expect("a unit normal");
    let bottom =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, floor), Vector::new(0.0, 0.0, -1.0))
            .expect("a unit normal");
    let right_of = |[from_x, from_y]: [f64; 2], [to_x, to_y]: [f64; 2]| {
        Picker::InFrontOf(
            HalfSpace::new(
                Vector::new(to_y - from_y, from_x - to_x, 0.0),
                from_x * to_y - from_y * to_x,
            )
            .expect("a normal with a length"),
        )
    };
    let next = |index: usize| (index + 1) % corners.len();
    let after_the_arc = |index: usize| index.checked_sub(1).map(|side| sides[side]);
    let mut up = Vec::with_capacity(corners.len());
    let mut along_the_top = Vec::with_capacity(corners.len());
    let mut along_the_bottom = Vec::with_capacity(corners.len());
    for index in 0..corners.len() {
        up.push(builder.add_edge(low[index], high[index], EdgeRun::Straight)?);
        let picker = match after_the_arc(index) {
            None => Some(right_of(corners[0], corners[1])),
            // A shorter arc clockwise round its pipe bulges to the left of its
            // chord, which is the right going the other way.
            Some(Side::RoundAPipe(..)) => Some(right_of(corners[next(index)], corners[index])),
            Some(Side::Straight) => None,
        };
        let [run_high, run_low] = [top, bottom].map(|plane| match picker {
            Some(picker) => EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(plane),
                picked_by: smallvec![picker],
            },
            None => EdgeRun::Straight,
        });
        along_the_top.push(builder.add_edge(high[index], high[next(index)], run_high)?);
        along_the_bottom.push(builder.add_edge(low[index], low[next(index)], run_low)?);
    }
    let mut faces = Vec::with_capacity(corners.len() + 2);
    let top_steps: Vec<_> = along_the_top.iter().map(|&edge| (edge, Forward)).collect();
    let top_wire = a_wire(&mut builder, &top_steps)?;
    faces.push((builder.add_face(Surface::Plane(top), &[top_wire])?, Forward));
    let bottom_steps: Vec<_> = along_the_bottom
        .iter()
        .rev()
        .map(|&edge| (edge, Reversed))
        .collect();
    let bottom_wire = a_wire(&mut builder, &bottom_steps)?;
    faces.push((
        builder.add_face(Surface::Plane(bottom), &[bottom_wire])?,
        Forward,
    ));
    for index in 0..corners.len() {
        let wire = a_wire(
            &mut builder,
            &[
                (along_the_bottom[index], Forward),
                (up[next(index)], Forward),
                (along_the_top[index], Reversed),
                (up[index], Reversed),
            ],
        )?;
        let (surface, used) = match after_the_arc(index) {
            None => {
                let axis = Axis::new(
                    Point::new(centre_x, centre_y, 0.0),
                    Vector::new(0.0, 0.0, 1.0),
                )
                .expect("a finite axis");
                (
                    Surface::Cylinder(Cylinder::new(axis, by * radius).expect("a positive radius")),
                    Forward,
                )
            }
            Some(Side::RoundAPipe([x, y], pipe)) => {
                let axis = Axis::new(Point::new(x, y, 0.0), Vector::new(0.0, 0.0, 1.0))
                    .expect("a finite axis");
                (
                    Surface::Cylinder(Cylinder::new(axis, pipe).expect("a positive radius")),
                    Reversed,
                )
            }
            Some(Side::Straight) => {
                let ([x, y], [to_x, to_y]) = (corners[index], corners[next(index)]);
                let outward = Vector::new(to_y - y, x - to_x, 0.0);
                (
                    Surface::Plane(
                        Plane::from_point_and_normal(Point::new(x, y, 0.0), outward)
                            .expect("a side with a length"),
                    ),
                    Forward,
                )
            }
        };
        faces.push((builder.add_face(surface, &[wire])?, used));
    }
    builder.add_shell(&faces)?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

#[test]
fn probe_q633_sweep() {
    for k in 1..=300 {
        let delta = f64::from(k) * 1e-18;
        for swapped in [false, true] {
            let what = format!("delta {delta:e}, swapped {swapped}");
            eprintln!("Q633 RUN {what}");
            let answer = std::panic::catch_unwind(|| {
                let mut model =
                    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
                let pair = a_bored_plate_and_a_holed_bar_inside_the_bore(&mut model, delta);
                fused_in_order(&mut model, pair, Rule::Union, swapped)
                    .map(|solids| solids.len())
                    .map_err(|why| format!("{why:?}").chars().take(80).collect::<String>())
            });
            match answer {
                Ok(Ok(solids)) => eprintln!("Q633 ANSWER {what}: {solids}"),
                Ok(Err(why)) => eprintln!("Q633 REFUSED {what}: {why}"),
                Err(_) => eprintln!("Q633 PANIC {what}"),
            }
        }
    }
}

#[test]
fn probe_q633_round() {
    for delta in [1e-1, 1e-8, 1e-12, 1e-14, 1e-15, 1e-16, 3e-17] {
        for (rule, swapped) in every_rule_both_ways() {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let pair = a_bored_plate_and_a_holed_bar_inside_the_bore(&mut model, delta);
            let what = format!("delta {delta:e}, {rule:?}, swapped {swapped}");
            eprintln!("Q633 RUN {what}");
            match fused_in_order(&mut model, pair, rule, swapped) {
                Ok(solids) => {
                    let volumes: Vec<String> = solids
                        .iter()
                        .map(|solid| {
                            model
                                .solid(*solid)
                                .expect("this model made the key")
                                .measure(
                                    Deflection::of(0.001).expect("a fine mesh"),
                                    &Policy::certified(),
                                    &mut ProvenanceLog::new(),
                                )
                                .map_or_else(
                                    |why| format!("does not measure: {why}"),
                                    |measured| {
                                        format!(
                                            "{} +- {}",
                                            measured.volume(),
                                            measured.volume_error()
                                        )
                                    },
                                )
                        })
                        .collect();
                    let valid = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
                    eprintln!(
                        "Q633 ANSWER {what}: {volumes:?} expected {} valid {valid:?}",
                        what_the_bored_plate_and_the_holed_bar_answer(rule, swapped, delta)
                    );
                }
                Err(why) => eprintln!("Q633 REFUSED {what}: {why:?}"),
            }
        }
    }
}

fn what_the_bored_plate_and_the_holed_bar_answer(rule: Rule, swapped: bool, delta: f64) -> f64 {
    let sliver = what_the_bore_holds_past_the_bar(delta);
    match (rule, swapped) {
        (Rule::Union, _) => 576.0 * PI - PI / 4.0 - 4.0 * sliver,
        (Rule::Intersection, _) => 219.0 * PI + 4.0 * sliver,
        (Rule::Difference, false) => 320.0 * PI - 4.0 * sliver,
        (Rule::Difference, true) => 36.0 * PI + 0.75 * PI - 4.0 * sliver,
        (other, _) => panic!("not a rule this probes: {other:?}"),
    }
}

fn what_the_bore_holds_past_the_bar(delta: f64) -> f64 {
    let apart = 5.0 + delta;
    let segment = |mine: f64, theirs: f64| {
        let half_angle = ((apart * apart + mine * mine - theirs * theirs) / (2.0 * apart * mine))
            .clamp(-1.0, 1.0)
            .acos();
        mine * mine * (half_angle - half_angle.sin() * half_angle.cos())
    };
    9.0 * PI - (segment(3.0, 8.0) + segment(8.0, 3.0))
}

fn a_bored_plate_and_a_holed_bar_inside_the_bore(model: &mut Model, delta: f64) -> [SolidKey; 2] {
    let plate = model
        .add_round_bar(12.0, 4.0, &at(-3.0, -4.0, -4.0))
        .expect("a bar of radius twelve");
    let bore = model
        .add_round_bar(3.0, 6.0, &at(0.6 * delta, 0.8 * delta, -5.0))
        .expect("a bar of radius three");
    let bored = the_one_solid(model, plate, bore, Rule::Difference);
    let wide_hole = model
        .add_round_bar(0.5, 6.0, &at(-6.0, -4.0, -5.0))
        .expect("a bar of radius a half");
    let holed_plate = the_one_solid(model, bored, wide_hole, Rule::Difference);
    let bar = model
        .add_round_bar(8.0, 4.0, &at(-3.0, -4.0, -4.0))
        .expect("a bar of radius eight");
    let narrow_hole = model
        .add_round_bar(0.25, 6.0, &at(-6.0, -4.0, -5.0))
        .expect("a bar of radius a quarter");
    let holed = the_one_solid(model, bar, narrow_hole, Rule::Difference);
    [holed_plate, holed]
}

#[test]
fn probe_q633() {
    for delta in [1e-1, 1e-8, 1e-10, 1e-12, 1e-13, 1e-14, 1e-15] {
        for (rule, swapped) in every_rule_both_ways() {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
            let pair = a_bored_plate_and_a_holed_block_whose_wall_leans(&mut model, delta);
            let what = format!("delta {delta:e}, {rule:?}, swapped {swapped}");
            eprintln!("Q633 RUN {what}");
            match fused_in_order(&mut model, pair, rule, swapped) {
                Ok(solids) => {
                    let volumes: Vec<String> = solids
                        .iter()
                        .map(|solid| {
                            model
                                .solid(*solid)
                                .expect("this model made the key")
                                .measure(
                                    Deflection::of(0.001).expect("a fine mesh"),
                                    &Policy::certified(),
                                    &mut ProvenanceLog::new(),
                                )
                                .map_or_else(
                                    |why| format!("does not measure: {why}"),
                                    |measured| {
                                        format!(
                                            "{} +- {}",
                                            measured.volume(),
                                            measured.volume_error()
                                        )
                                    },
                                )
                        })
                        .collect();
                    let valid = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
                    eprintln!(
                        "Q633 ANSWER {what}: {volumes:?} expected {} valid {valid:?}",
                        what_the_bored_plate_and_the_holed_block_answer(rule, swapped, delta)
                    );
                }
                Err(why) => eprintln!("Q633 REFUSED {what}: {why:?}"),
            }
        }
    }
}

fn what_the_bored_plate_and_the_holed_block_answer(rule: Rule, swapped: bool, delta: f64) -> f64 {
    let past_the_wall = what_the_bore_holds_past_the_leaning_wall(delta);
    match (rule, swapped) {
        (Rule::Union, _) => 399.0 * PI - past_the_wall,
        (Rule::Intersection, _) => 300.0 - 36.0 * PI + past_the_wall,
        (Rule::Difference, false) => 400.0 * PI - 300.0 - past_the_wall,
        (Rule::Difference, true) => 35.0 * PI - past_the_wall,
        (other, _) => panic!("not a rule this probes: {other:?}"),
    }
}

fn what_the_bore_holds_past_the_leaning_wall(delta: f64) -> f64 {
    let antiderivative = |h: f64| {
        let half_chord = (9.0 - h * h).max(0.0).sqrt();
        9.0 * (h * (h / 3.0).clamp(-1.0, 1.0).acos() - half_chord) + half_chord.powi(3) / 3.0
    };
    4.0 * (antiderivative(3.0 - delta) - antiderivative(2.0 - delta))
}

fn a_bored_plate_and_a_holed_block_whose_wall_leans(
    model: &mut Model,
    delta: f64,
) -> [SolidKey; 2] {
    let plate = model
        .add_round_bar(10.0, 4.0, &at(0.0, 0.0, -4.0))
        .expect("a bar of radius ten");
    let bore = model
        .add_round_bar(3.0, 6.0, &at(delta, 0.0, -5.0))
        .expect("a bar of radius three");
    let bored = the_one_solid(model, plate, bore, Rule::Difference);
    let block = a_block_whose_wall_leans(model).unwrap_or_else(|why| panic!("the block: {why:?}"));
    let hole = model
        .add_round_bar(0.5, 6.0, &at(0.0, 0.0, -5.0))
        .expect("a bar of radius a half");
    let holed = the_one_solid(model, block, hole, Rule::Difference);
    [bored, holed]
}

/// The block `−5 ≤ x`, `−5 ≤ y ≤ 5`, `−4 ≤ z ≤ 0`, up to the wall `4x = z + 12`,
/// which is at `x = 3` along the top and `x = 2` along the floor.
fn a_block_whose_wall_leans(model: &mut Model) -> rcad_topo::Result<SolidKey> {
    use Orientation::{Forward, Reversed};
    let mut builder = SolidBuilder::new();
    let mut both_ends = |[x, z]: [f64; 2]| {
        Ok::<_, rcad_topo::Error>([
            builder.add_vertex(Point::new(x, -5.0, z))?,
            builder.add_vertex(Point::new(x, 5.0, z))?,
        ])
    };
    let [rear_floor_front, rear_floor_back] = both_ends([-5.0, -4.0])?;
    let [rear_top_front, rear_top_back] = both_ends([-5.0, 0.0])?;
    let [wall_top_front, wall_top_back] = both_ends([3.0, 0.0])?;
    let [wall_floor_front, wall_floor_back] = both_ends([2.0, -4.0])?;
    let mut straight = |start, end| builder.add_edge(start, end, EdgeRun::Straight);
    let rear_floor_across = straight(rear_floor_front, rear_floor_back)?;
    let rear_top_across = straight(rear_top_front, rear_top_back)?;
    let wall_top_across = straight(wall_top_front, wall_top_back)?;
    let wall_floor_across = straight(wall_floor_front, wall_floor_back)?;
    let front_rear = straight(rear_floor_front, rear_top_front)?;
    let front_top = straight(rear_top_front, wall_top_front)?;
    let front_wall = straight(wall_floor_front, wall_top_front)?;
    let front_floor = straight(rear_floor_front, wall_floor_front)?;
    let back_rear = straight(rear_floor_back, rear_top_back)?;
    let back_top = straight(rear_top_back, wall_top_back)?;
    let back_wall = straight(wall_floor_back, wall_top_back)?;
    let back_floor = straight(rear_floor_back, wall_floor_back)?;

    let top = a_flat_face(
        &mut builder,
        Point::new(0.0, 0.0, 0.0),
        Vector::new(0.0, 0.0, 1.0),
        &[
            (front_top, Forward),
            (wall_top_across, Forward),
            (back_top, Reversed),
            (rear_top_across, Reversed),
        ],
    )?;
    let floor = a_flat_face(
        &mut builder,
        Point::new(0.0, 0.0, -4.0),
        Vector::new(0.0, 0.0, -1.0),
        &[
            (rear_floor_across, Forward),
            (back_floor, Forward),
            (wall_floor_across, Reversed),
            (front_floor, Reversed),
        ],
    )?;
    let rear = a_flat_face(
        &mut builder,
        Point::new(-5.0, 0.0, 0.0),
        Vector::new(-1.0, 0.0, 0.0),
        &[
            (front_rear, Forward),
            (rear_top_across, Forward),
            (back_rear, Reversed),
            (rear_floor_across, Reversed),
        ],
    )?;
    let wall = a_flat_face(
        &mut builder,
        Point::new(3.0, 0.0, 0.0),
        Vector::new(4.0, 0.0, -1.0),
        &[
            (wall_floor_across, Forward),
            (back_wall, Forward),
            (wall_top_across, Reversed),
            (front_wall, Reversed),
        ],
    )?;
    let front = a_flat_face(
        &mut builder,
        Point::new(0.0, -5.0, 0.0),
        Vector::new(0.0, -1.0, 0.0),
        &[
            (front_floor, Forward),
            (front_wall, Forward),
            (front_top, Reversed),
            (front_rear, Reversed),
        ],
    )?;
    let back = a_flat_face(
        &mut builder,
        Point::new(0.0, 5.0, 0.0),
        Vector::new(0.0, 1.0, 0.0),
        &[
            (back_rear, Forward),
            (back_top, Forward),
            (back_wall, Reversed),
            (back_floor, Reversed),
        ],
    )?;
    builder.add_shell(&[
        (top, Forward),
        (floor, Forward),
        (rear, Forward),
        (wall, Forward),
        (front, Forward),
        (back, Forward),
    ])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}
