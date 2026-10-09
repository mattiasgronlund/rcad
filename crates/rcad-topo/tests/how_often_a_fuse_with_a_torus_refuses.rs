//! How often a certified fuse with a torus in it gives up, family by family.
//!
//! §17.7's discipline, unchanged: the rates are **never blended**, every family
//! prints its own number with its own denominator, and every refusal is named
//! rather than counted. The headline is expected to be bad, and a bad rate
//! reported is worth more than a good rate got by only drawing the shapes that
//! work.
//!
//! # A torus refuses every family here, and the reason is structural
//!
//! `intersect` names a plane against a torus in exactly two families — one
//! **square to the axis** and one **through the axis** — and refuses every other
//! plane as a quartic. And `fuse/cuts.rs` has no face-level box test, on purpose:
//! its own note argues that skipping a face pair whose boxes miss *changes an
//! answer*, and §17.4 threw the BVH out for the same reason from the other side.
//! So a tool solid cannot hide its awkward planes by keeping them out of the
//! way; every plane it carries is put to `intersect`.
//!
//! Put those two together and the consequence is not a rate but a theorem:
//!
//! > A torus can be fused with a flat-sided solid **only if every plane of that
//! > solid is square to the torus's axis or contains it.** No bounded solid made
//! > of planes has that property, because the planes square to the axis and the
//! > planes through it leave a solid open in the radial direction, and closing it
//! > takes a plane that is neither.
//!
//! `no_box_of_planes_can_close_round_a_torus_axis_without_a_quartic` is that
//! argument made into a measurement rather than left as a paragraph: it counts,
//! for every family below, how many of the tool's planes fall in each of the
//! three rows.
//!
//! # Two refusals answer for four of the five, and both of them are true
//!
//! This paragraph has said three different things across three lanes, and the
//! rate under it has been 0 of 96 for every one of them. What changed each time
//! is *which* refusal fires, and only the last of the three is a reason that
//! survives reading:
//!
//! 1. `WhereALineMeetsATorusIsAQuartic` — refused on surface **type**, before a
//!    root was worked out. Gone: where a straight line meets a torus is answered
//!    now, in nought, one, two, three or four places.
//! 2. `ACrossingThisCannotBeOrderedAlong` — and this one was **false**. Every
//!    crossing a bend has is a circle, and every one of them can be ordered; the
//!    arrangement's list of orderable surface pairs simply had no plane-and-torus
//!    row in it.
//! 3. What is left, which is the theorem at the top of this file arriving as a
//!    measurement:
//!
//! | tool | the refusal | why it is true |
//! |---|---|---|
//! | a slab square to the axis | `ACrossingBetweenAPlaneAndATorusThatIsAQuartic` | the slab's four sides run parallel to the axis and off it, and such a plane cuts Cassini ovals |
//! | a wedge through the axis | the same | two of its planes hold the axis; the one that closes it radially does neither |
//! | a bar through the hole | none: all 96 answered since row Q1334 | the bar is on the bend's axis and one narrower than the hole, so it misses the tube, and `intersect` answers that |
//! | a ball in the bend | `ACrossingBetweenATorusAndAnotherCurvedSurface` | a torus and a ball off its axis cross in a space curve of degree up to eight |
//! | a second bend | the same | two tori, likewise |
//!
//! Both are `intersect`'s, both are raised before the arrangement is reached,
//! and both are **gaps** in `rcad-geom` rather than anything topology can close:
//! the curves are all there and none of them can be named with the runs this
//! kernel has. So the first two rows are the theorem above, and the last two
//! are §36.20's quartic-and-worse, still weighed and still left. A bar on the
//! axis that does reach the tube, the bar of the ring's own radius, answers
//! since row Q1387 by every rule both ways round, and
//! `a_quarter_bend_and_the_bar_of_its_own_radius_fuse_by_every_rule_to_what_pappus_gives`
//! in `a_bend_under_two_planes.rs` checks each answer against Pappus. Neither
//! refusal touches `a_bend_fused_with_a_copy_of_itself_is_the_bend_or_nothing`
//! below — the one test in this file that does reach the arrangement,
//! `select::choose` and `assemble`'s `join_up_what_is_one_face`, under all
//! three rules — because fusing a bend with an exact copy of itself draws no
//! plane-and-torus crossing at all.
//!
//! The rate did not move through any of this — 0 of 96, three times over. That
//! is the finding rather than a disappointment: a refusal walking down the
//! pipeline one stage per lane is what it looks like when several true blockers
//! are stacked, and only the last of them tells you where a bend really stops.
//! Row Q1334 moved the bar through the hole to 96 of 96, by answering a pipe on
//! a ring's axis that misses its tube; the other four rows have not moved.
//!
//! # The control is the whole reason these numbers mean anything
//!
//! A **round bar** of the same size, in the same place, against the same tools:
//! §17.7's precedent is that the control is what tells a timid filter from real
//! undecidability, and the cone lane's most useful result was exactly this —
//! that a corner-up cube refuses a bar nearly as often as a cone. Here it says
//! the opposite for three tools of the five and agrees with the bend for the
//! fourth, and both halves are the finding:
//!
//! | tool | a bend | a round bar in its place |
//! |---|---:|---:|
//! | a slab square to the axis | 0 of 96 answered | **96 of 96** |
//! | a wedge through the axis | 0 of 96 | **96 of 96** |
//! | a bar through the hole | **96 of 96** | **96 of 96** |
//! | a ball in the bend | 0 of 96 | 0 of 96 |
//! | a second bend | 0 of 96 | — |
//!
//! Three of the four controls answer, so the rate above is the torus's and not
//! the tools'. The fourth does not, and that is a result rather than a hole: a
//! ball resting on a bar is a hard pair for this kernel already — all 96
//! refuse `Undecidable`, the last record `a point in a piece that every solid
//! could place` — so that row measures the tool. The fifth takes no control at all, because a bar in place
//! of a second bend still leaves a torus in the pair.

use std::collections::BTreeMap;

use rcad_geom::{Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Model, ProfileBuilder, Rule, SolidKey};

mod rings;

use rings::a_quarter_bend;

/// How many pairs each family draws.
const DRAWS: usize = 16;

/// A deterministic sequence, so that a rate is one number and not a range.
struct Draw(u64);

impl Draw {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        self.0 >> 33
    }

    /// A whole number between `low` and `high`, both ends included.
    fn whole(&mut self, low: i64, high: i64) -> f64 {
        let span = u64::try_from(high - low + 1).expect("a range that runs upwards");
        let picked = i64::try_from(self.next() % span).expect("inside the range") + low;
        picked as f64
    }
}

/// What the curved solid of a pair is: the thing being measured, or the control.
#[derive(Clone, Copy, PartialEq)]
enum Curved {
    /// A quarter bend of pipe, which carries the torus face.
    ABend,
    /// A round bar of the same size in the same place. **The control.**
    ABar,
}

/// The tools, and what each is a measurement of.
#[derive(Clone, Copy)]
enum Tool {
    /// A slab between two planes square to the torus's axis. Its two flat faces
    /// are the family `intersect` names; its four sides run parallel to the
    /// axis and off it, which is the quartic row.
    SlabSquareToTheAxis,
    /// A wedge with two faces through the axis. Two of its planes are the other
    /// family `intersect` names, and the one that closes it radially is not.
    WedgeThroughTheAxis,
    /// A round bar through the middle of the ring, on its axis and one
    /// narrower than the hole: a torus against a pipe that misses its tube.
    BarThroughTheHole,
    /// A ball resting in the bend: a torus against a sphere.
    BallInTheBend,
    /// A second bend on the same axis: a torus against a torus.
    SecondBend,
}

impl Tool {
    fn named(self) -> &'static str {
        match self {
            Tool::SlabSquareToTheAxis => "a slab square to the axis",
            Tool::WedgeThroughTheAxis => "a wedge through the axis",
            Tool::BarThroughTheHole => "a bar through the hole",
            Tool::BallInTheBend => "a ball in the bend",
            Tool::SecondBend => "a second bend",
        }
    }

    /// Whether the control can be drawn against this tool at all: a bar cannot
    /// stand in for a bend against a *second* bend, because then the pair has a
    /// torus in it either way.
    fn takes_a_control(self) -> bool {
        !matches!(self, Tool::SecondBend)
    }
}

/// A convex prism of the given cross-section, extruded `up` from `at`.
fn a_prism(model: &mut Model, outline: Vec<[f64; 2]>, up: f64, at: Vector) -> Option<SolidKey> {
    let profile = ProfileBuilder::new(outline)
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .ok()?;
    model
        .add_extrusion(&profile, up, &Isometry::translation(at).ok()?)
        .ok()
}

fn a_pair(curved: Curved, tool: Tool, draw: &mut Draw) -> Option<(Model, SolidKey, SolidKey)> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ring = draw.whole(4, 8);
    let tube = draw.whole(1, 2);
    let curved = match curved {
        Curved::ABend => a_quarter_bend(&mut model, ring, tube),
        // The same tube, standing where the bend's own `+x` cap does, so the
        // control is the same size in the same place and not a different
        // experiment.
        Curved::ABar => model
            .add_round_bar(
                tube,
                2.0 * tube,
                &Isometry::translation(Vector::new(ring, 0.0, -tube)).ok()?,
            )
            .ok()?,
    };

    let reach = ring + tube + 1.0;
    let other = match tool {
        Tool::SlabSquareToTheAxis => a_prism(
            &mut model,
            vec![
                [-reach, -reach],
                [reach, -reach],
                [reach, reach],
                [-reach, reach],
            ],
            tube,
            Vector::new(0.0, 0.0, draw.whole(-1, 0)),
        )?,
        // Turned forty-five degrees off the bend's own caps, so that the
        // control bar does not stand exactly on one of the wedge's planes —
        // which is a degeneracy about the placement and not about the torus.
        Tool::WedgeThroughTheAxis => a_prism(
            &mut model,
            vec![[0.0, 0.0], [reach, reach], [-reach, reach]],
            2.0 * tube,
            Vector::new(0.0, 0.0, -tube),
        )?,
        Tool::BarThroughTheHole => model
            .add_round_bar(
                ring - tube - 1.0,
                4.0 * tube,
                &Isometry::translation(Vector::new(0.0, 0.0, -2.0 * tube)).ok()?,
            )
            .ok()?,
        Tool::BallInTheBend => model
            .add_ball(
                tube,
                &Isometry::translation(Vector::new(ring, 0.0, 0.0)).ok()?,
            )
            .ok()?,
        Tool::SecondBend => a_quarter_bend(&mut model, ring, tube / 2.0),
    };
    Some((model, curved, other))
}

/// Every rule, both ways round, over one family.
fn how_each_fuse_went(curved: Curved, tool: Tool) -> (usize, usize, BTreeMap<String, usize>) {
    let policy = Policy::certified();
    let mut draw = Draw(0x5EED);
    let mut fuses = 0;
    let mut answered = 0;
    let mut gave_up: BTreeMap<String, usize> = BTreeMap::new();
    for _ in 0..DRAWS {
        let Some((mut model, ring, other)) = a_pair(curved, tool, &mut draw) else {
            continue;
        };
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for (arguments, tools) in [([ring], [other]), ([other], [ring])] {
                fuses += 1;
                let mut log = ProvenanceLog::new();
                match model.fuse(&arguments, &tools, rule, &policy, &mut log) {
                    Ok(_) => {
                        answered += 1;
                        // A fuse hands back a sound model or it refuses. This is
                        // the half that makes the rate mean anything: a family
                        // that answered by writing a broken solid would look
                        // like a good rate.
                        model
                            .validate(&policy, &mut ProvenanceLog::new())
                            .expect("a fuse hands back a sound model or refuses");
                    }
                    Err(Error::Undecidable { records }) => {
                        let named = records
                            .records()
                            .last()
                            .map_or("nothing named", |record| record.predicate());
                        *gave_up.entry(named.to_owned()).or_default() += 1;
                    }
                    Err(why) => *gave_up.entry(format!("{why}")).or_default() += 1,
                }
            }
        }
    }
    (fuses, answered, gave_up)
}

/// The measurement, printed family by family and never blended, each beside its
/// control.
///
/// # What is asserted and what is only reported
///
/// **Every bend family but the bar through the hole is asserted at nothing
/// answered.** That is not a rate to be improved by tuning: each of those four
/// tools carries at least one plane that is neither square to the torus's axis
/// nor through it, or is curved and off the axis, and both of those are shapes
/// this vocabulary has no arm for. A fuse that answered one would be answering
/// something it cannot name, which §4.1 ranks below refusing.
///
/// **The bar through the hole is asserted at everything answered**, each answer
/// validated here and its volume checked in
/// `a_bend_and_a_bar_through_its_hole_on_its_axis_fuse_into_what_pappus_gives`.
/// It stands on the bend's axis one narrower than the hole, and `intersect`
/// answers a pipe on a ring's axis that misses the tube (row Q1334, §378).
///
/// **The controls are reported and one thing about them is asserted**: that some
/// of them answer. That is what makes the rows above mean anything — the same
/// tools, the same rules, the same draws, with a round bar where the bend was,
/// come through, so the refusals are the torus's and not the tools'. It is not
/// asserted of *every* control, and the reason is a result rather than a
/// weakening: a ball resting on a bar is a hard pair for this kernel already,
/// and a control that refuses says the tool is hard rather than that the torus
/// is fine. Both readings are in the printout.
#[test]
fn how_often_a_certified_fuse_with_a_torus_in_it_refuses() {
    let mut every = Vec::new();
    for tool in [
        Tool::SlabSquareToTheAxis,
        Tool::WedgeThroughTheAxis,
        Tool::BarThroughTheHole,
        Tool::BallInTheBend,
        Tool::SecondBend,
    ] {
        for curved in [Curved::ABend, Curved::ABar] {
            if curved == Curved::ABar && !tool.takes_a_control() {
                continue;
            }
            let (fuses, answered, gave_up) = how_each_fuse_went(curved, tool);
            let refused: usize = gave_up.values().sum();
            println!(
                "{} against {}: {fuses} fuses, {answered} answered, {refused} refused \
                 ({:.1}%): {gave_up:?}",
                if curved == Curved::ABend {
                    "a bend"
                } else {
                    "a round bar (the control)"
                },
                tool.named(),
                100.0 * refused as f64 / fuses as f64
            );
            assert!(fuses >= 90, "{}: only {fuses} fuses", tool.named());
            every.push((curved, tool, fuses, answered));
        }
    }

    let mut controls_answered = 0;
    for (curved, tool, fuses, answered) in every {
        if curved == Curved::ABend && matches!(tool, Tool::BarThroughTheHole) {
            assert_eq!(answered, fuses, "{}", tool.named());
        } else if curved == Curved::ABend {
            assert_eq!(
                answered,
                0,
                "{}: every tool here carries a plane a torus cannot be crossed with, or is \
                 curved, and this answered {answered} of {fuses}",
                tool.named()
            );
        } else {
            controls_answered += answered;
        }
    }
    assert!(
        controls_answered > 0,
        "not one control answered anything, so the rows above measure the tools rather than \
         the torus and the whole comparison says nothing"
    );
}

/// The table's bar through the hole, checked answer by answer: every ring and
/// tube the draws take, every rule both ways round, validates and holds what
/// Pappus gives. The bar is one narrower than the hole, so it misses the tube,
/// and `intersect` answers a bend's ring against a pipe on its axis that
/// misses it with nothing (row Q1334, §378). So the union is the two solids
/// apart, the intersection nothing, and each difference its argument.
#[test]
fn a_bend_and_a_bar_through_its_hole_on_its_axis_fuse_into_what_pappus_gives() {
    let policy = Policy::certified();
    let pi = core::f64::consts::PI;
    for ring in [4.0, 5.0, 6.0, 7.0, 8.0] {
        for tube in [1.0, 2.0] {
            let bar_radius = ring - tube - 1.0;
            let (model, bend, bar) = a_bend_and_a_bar_on_its_axis(ring, tube, bar_radius);
            let the_bend = pi * pi * ring * tube * tube / 2.0;
            let the_bar = pi * bar_radius * bar_radius * 4.0 * tube;
            for (rule, arguments, tools, solids, volume) in [
                (Rule::Union, bend, bar, 2, the_bend + the_bar),
                (Rule::Union, bar, bend, 2, the_bend + the_bar),
                (Rule::Intersection, bend, bar, 0, 0.0),
                (Rule::Intersection, bar, bend, 0, 0.0),
                (Rule::Difference, bend, bar, 1, the_bend),
                (Rule::Difference, bar, bend, 1, the_bar),
            ] {
                let named = format!("ring {ring}, tube {tube}, {rule:?}");
                let mut answer = model.clone();
                let fused = answer
                    .fuse(
                        &[arguments],
                        &[tools],
                        rule,
                        &policy,
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|why| panic!("{named}: {why}"));
                answer
                    .validate(&policy, &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{named}: {why}"));
                assert_eq!(fused.solids().len(), solids, "{named}");
                let (mut measured, mut allowed) = (0.0, 0.0);
                for solid in fused.solids() {
                    let held = answer
                        .solid(*solid)
                        .expect("the fuse made the key")
                        .measure(
                            Deflection::of(0.01).expect("a length"),
                            &policy,
                            &mut ProvenanceLog::new(),
                        )
                        .unwrap_or_else(|why| panic!("{named}: {why}"));
                    measured += held.volume();
                    allowed += held.volume_error();
                }
                assert!(
                    (measured - volume).abs() <= allowed,
                    "{named}: measured {measured} where Pappus gives {volume}, allowed {allowed}"
                );
            }
        }
    }
}

/// A quarter bend and a round bar of `bar_radius` on its axis, four tubes tall
/// and standing across the bend's middle, in a model of their own.
fn a_bend_and_a_bar_on_its_axis(
    ring: f64,
    tube: f64,
    bar_radius: f64,
) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bend = a_quarter_bend(&mut model, ring, tube);
    let bar = model
        .add_round_bar(
            bar_radius,
            4.0 * tube,
            &Isometry::translation(Vector::new(0.0, 0.0, -2.0 * tube)).expect("a finite shift"),
        )
        .expect("a bar with a radius and a height");
    (model, bend, bar)
}

/// **The theorem in the module note, made into a measurement.**
///
/// A plane and a torus cross in circles in two families and in a quartic
/// otherwise, so a flat-sided solid is fusible with a torus only if every one of
/// its planes falls in one of the two. This counts, for each of the two
/// flat-sided tools above, how many of its planes do — and the point is that
/// neither reaches all of them, and that no solid of planes can.
///
/// A convex solid of planes is bounded only if its outward normals surround the
/// origin of direction space. The normals square to the axis are `±w`, and the
/// normals of planes through the axis are all square to `w`. Those together
/// surround nothing in the `w` direction *and* nothing across it at once: any
/// direction leaning between the two is uncovered, so the solid is open along
/// it. Bounding it takes a normal that is neither, which is the quartic row.
#[test]
fn no_box_of_planes_can_close_round_a_torus_axis_without_a_quartic() {
    use rcad_geom::Surface;

    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let reach = 8.0;
    let slab = a_prism(
        &mut model,
        vec![
            [-reach, -reach],
            [reach, -reach],
            [reach, reach],
            [-reach, reach],
        ],
        2.0,
        Vector::new(0.0, 0.0, -1.0),
    )
    .expect("a slab");
    let wedge = a_prism(
        &mut model,
        vec![[0.0, 0.0], [reach, reach], [-reach, reach]],
        2.0,
        Vector::new(0.0, 0.0, -1.0),
    )
    .expect("a wedge");

    let axis = Vector::new(0.0, 0.0, 1.0);
    for (named, solid, square_to, through, neither) in
        [("a slab", slab, 2, 0, 4), ("a wedge", wedge, 2, 2, 1)]
    {
        let mut counted = [0usize; 3];
        for face in model.solid(solid).expect("this model made the key").faces() {
            let Surface::Plane(flat) = face.surface() else {
                panic!("{named} is made of planes");
            };
            let normal = flat.normal();
            let row = if normal.cross(axis) == Vector::zero() {
                0
            } else if normal.dot(axis) == 0.0
                && flat.signed_distance_to(Point::origin()).abs() < 1e-15
            {
                1
            } else {
                2
            };
            counted[row] += 1;
        }
        println!(
            "{named}: {} planes square to the axis, {} through it, {} neither",
            counted[0], counted[1], counted[2]
        );
        assert_eq!(counted, [square_to, through, neither], "{named}");
        assert!(
            counted[2] > 0,
            "{named} closes round the axis with no plane in the quartic row, which the note \
             above says cannot happen"
        );
    }
}

/// **The strongest possible case, and it still refuses.**
///
/// The table above only shows that every *tool this corpus drew* fails —
/// eighty planes and curves picked at random, none of them the torus's own.
/// This is the case a corpus can never draw: a bend and a bit-for-bit copy of
/// itself, in the same place. If anything ever fuses with a bend it is this,
/// because every corner the arrangement would need to find is one the two
/// solids already agree on exactly.
///
/// It refuses anyway, under all three rules, and it gets **further** than any
/// drawn tool does. The two bends share every surface, so `intersect` answers
/// `SameSurface` where it would otherwise refuse a torus against a torus, and
/// every plane-and-torus crossing either bend has is one of the two circle
/// families. So the arrangement runs to the end — it finds the corners, puts
/// them in order round the circles they sit on and cuts every face into
/// pieces — and what refuses is choosing the pieces: the point found inside
/// the end cap on `y = 0`, the disk the tube leaves on that plane, cannot be
/// placed against the copy.
///
/// **This used to stop earlier, at `TwoCurvesTouchWithoutCrossing`, and that
/// was the wrong name.** The plane `y = 0` crosses the torus in two circles,
/// one round `(4, 0, 0)` and one round `(−4, 0, 0)`. The cap's corner
/// `(3, 0, 0)` is on the first; the second's picker certainly rules the corner
/// out. But the model lists the torus round that corner, and the corner was
/// put on *both* circles for it — so two ways out of it ran along the one
/// torus, which no turn and no bend can tell apart, and the sort called that
/// two curves touching. Measured with the old rule put back: the refusal
/// was at the cap's corner `(3, 0, 0)`, between the cap edge's own circle and
/// the far one. Since a corner is on a curve of a surface it lists only where
/// that curve's picker allows (§224.2's joined cone needed the same), the
/// arrangement gets past it, and this is what the arithmetic really says.
///
/// **Then it refused finding a point inside the end cap, and that was a gap**
/// (§16.10): the one point tried was the cap's one corner, on the rim, and a
/// disk has no other corner to blend with. `select.rs`'s
/// `the_middles_of_a_round_wire` now offers where the ring crosses the cap's
/// plane — both places, `(4, 0, 0)` and `(−4, 0, 0)`, because which circle a
/// wire goes round is not known there — and the piece holds `(4, 0, 0)`,
/// certified. It does not hold `(−4, 0, 0)`, which is inside the torus but in
/// the other circle's disk; the half-space `intersect` names the cap's circle
/// by is what says so, and without it the piece held that point too.
///
/// **Then it refused placing that point, and the ray was why**: the ray fired
/// from `(4, 0, 0)` at the copy answered `CannotFollowAFacesTrimming` on the
/// copy's torus face. `holds.rs` follows a bend's wall now (§243), and every
/// piece of both bends is placed on the other one, facing the same way.
///
/// **So the difference is answered, and the answer is nothing**, which is what
/// a solid less itself is. The union and the intersection keep the first
/// bend's three faces and drop the copy's — the table's "keep once" — and are
/// the bend again: one solid that validates and holds `2π²`. Until the bend's
/// wall was used forwards, writing those three faces back was refused,
/// `TheFuseWroteABrokenAnswer` with `CoedgesRunTheSameWay` (§243, §361).
///
/// This is still the one place in the file where the arrangement is reached
/// at all, and so the one that says the ordering added for a bend is real code
/// and not a gate nothing walks through.
#[test]
fn a_bend_fused_with_a_copy_of_itself_is_the_bend_or_nothing() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let first = a_quarter_bend(&mut model, 4.0, 1.0);
    let second = a_quarter_bend(&mut model, 4.0, 1.0);

    for rule in [Rule::Union, Rule::Intersection] {
        let mut answer = model.clone();
        let fused = answer
            .fuse(
                &[first],
                &[second],
                rule,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{rule:?} of a bend and a copy of itself: {why}"));
        let [bend] = fused.solids() else {
            panic!("{rule:?} of a bend and a copy of itself: {fused:?}");
        };
        answer
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{rule:?}: the answer does not validate: {why}"));
        let measured = answer
            .solid(*bend)
            .expect("the fuse made the key")
            .measure(
                Deflection::of(0.01).expect("a hundredth of the tube"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{rule:?}: the answer measures: {why}"));
        let pappus = 2.0 * core::f64::consts::PI * core::f64::consts::PI;
        assert!(
            (measured.volume() - pappus).abs() <= measured.volume_error(),
            "{rule:?}: a quarter bend of ring four and tube one holds {pappus}, and the answer \
             measures {} give or take {}",
            measured.volume(),
            measured.volume_error()
        );
    }
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(&[first], &[second], Rule::Difference, &policy, &mut log)
        .expect("a bend less a copy of itself");
    assert!(fused.solids().is_empty(), "{fused:?}");

    let copy = model.solid(second).expect("this model made the key");
    let placed = copy.classify_uncertain_point(
        PredicatePoint::from([4.0, 0.0, 0.0]),
        &policy,
        &mut ProvenanceLog::new(),
    );
    assert_eq!(
        placed,
        Containment::OnBoundary,
        "the middle of the end cap on y = 0 is on the copy's own cap"
    );
}
