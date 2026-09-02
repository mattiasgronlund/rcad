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
//! # One refusal answers for all five, and it is not the one this lane built
//!
//! Every bend family below comes back
//! `WhereALineMeetsATorusIsAQuartic` and never
//! `ACrossingBetweenAPlaneAndATorusThatIsAQuartic`. That is the arrangement's
//! order and not an accident: a fuse asks **where three surfaces meet** before
//! it asks what two of them cross in, and a line meets a torus four times
//! whatever the pair is. So the crossing refusals this lane named are correct
//! and are reachable from `intersect`, and *no fuse reaches them* — which means
//! naming the plane-and-torus quartic buys a fuse nothing until the
//! three-surface quartic is solved too, and that is worth knowing before
//! somebody spends a lane on the first alone.
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
//! | a slab square to the axis | 0 of 96 answered | **24 of 96** |
//! | a wedge through the axis | 0 of 96 | **96 of 96** |
//! | a bar through the hole | 0 of 96 | **96 of 96** |
//! | a ball in the bend | 0 of 96 | 0 of 96 |
//! | a second bend | 0 of 96 | — |
//!
//! Three of the four controls answer, so the rate above is the torus's and not
//! the tools'. The fourth does not, and that is a result rather than a hole: a
//! ball resting on a bar is a hard pair for this kernel already — `where three
//! surfaces meet is only worked out when two of them are flat` — so that row
//! measures the tool. The fifth takes no control at all, because a bar in place
//! of a second bend still leaves a torus in the pair.

use std::collections::BTreeMap;

use rcad_geom::{Error as GeomError, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

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
    /// A round bar through the middle of the ring: a torus against a pipe,
    /// which runs to degree eight.
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
/// **Every bend family is asserted at nothing answered.** That is not a rate to
/// be improved by tuning: each of the five tools carries at least one plane that
/// is neither square to the torus's axis nor through it, or is itself curved,
/// and both of those are shapes this vocabulary has no arm for. A fuse that
/// answered one would be answering something it cannot name, which §4.1 ranks
/// below refusing.
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
        if curved == Curved::ABend {
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
        for face in model.solid(solid).faces() {
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
/// It refuses anyway, under all three rules, and always with the same
/// complaint: `fuse/arrange.rs` asks where three surfaces meet **before** it
/// asks what two of them cross in, and a straight line meets a torus in a
/// quartic whatever the pair behind it is — the same reason
/// `WhereALineMeetsATorusIsAQuartic` answers for every family in
/// `how_often_a_certified_fuse_with_a_torus_in_it_refuses`. So this is not
/// four more measurements of the same thing; it is the demonstration that
/// there is no tool, drawn or undrawn, this kernel fuses a bend with today.
#[test]
fn a_bend_refuses_to_fuse_with_a_copy_of_itself_under_every_rule() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let first = a_quarter_bend(&mut model, 4.0, 1.0);
    let second = a_quarter_bend(&mut model, 4.0, 1.0);

    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        let mut log = ProvenanceLog::new();
        let refused = model.fuse(&[first], &[second], rule, &policy, &mut log);
        assert!(
            matches!(
                refused,
                Err(Error::Geometry(GeomError::WhereALineMeetsATorusIsAQuartic))
            ),
            "{rule:?} of a bend and a bit-for-bit copy of itself: expected the three-surface \
             quartic to refuse it, got {refused:?}"
        );
    }
}
