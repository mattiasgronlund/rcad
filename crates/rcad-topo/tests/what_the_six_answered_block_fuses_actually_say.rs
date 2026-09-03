//! What the six answers in `a frustum against a block` actually are.
//!
//! `how_often_a_fuse_with_a_cone_refuses.rs` pins that family at **6 answered of
//! 96** and re-derives that floor every run. A count is not a check: six fuses
//! returning `Ok` says six models came back, and says nothing at all about what
//! is in them. §4.1 ranks a wrong answer below a refusal, so the count on its own
//! was pinning the cheapest half of the claim.
//!
//! # The six are one draw, and it is the draw where nothing touches
//!
//! The family draws 16 rows and asks six fuses of each — union, intersection and
//! difference, both ways round. **All six answers come from one row**, and that
//! row is the one where the block misses the frustum altogether.
//!
//! The block is always put down with its low corner at `x = 1`, and the frustum
//! is always about the `z` axis and never wider than `slope · far`. So the two
//! are disjoint exactly when `slope · far < 1`, which over this grid — slope one
//! of `1/8`, `1/4`, `1/2`, and `far` a whole number from 6 to 12 — happens for
//! `slope · far ∈ {0.75, 0.875}` and nothing else. One row of the sixteen lands
//! there: `slope = 1/8`, `near = 4`, `far = 6`.
//!
//! So the honest sentence about that family is **not** "a block can now take a
//! piece out of a frustum". It is:
//!
//! > No block in this family has ever taken a piece out of a frustum. The six
//! > fuses that answer are the six ways of asking about a block and a frustum
//! > that do not meet, and every one of them hands back an input untouched.
//!
//! That is still worth having — a fuse that cannot even tell two disjoint solids
//! apart would be worse — and it is worth having *pinned*, because "answered" and
//! "answered correctly" came apart here and the count could not see it.
//!
//! # Nothing else in the tree checks these
//!
//! `fuses_that_must_agree.rs` carries the metamorphic properties, and **it has no
//! cone in it**: its curved family, `CurvedPair`, is a round bar against a block
//! and a bar against a copy of itself. Grep it for `Cone` and the only hit is the
//! surface-kind bookkeeping in `WhatItSitsOn`, which describes an answer's
//! surfaces rather than generating one. So none of commutativity, associativity,
//! idempotence or absorption has ever run over a cone, and these six were
//! uncovered until this file.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::a_frustum;

/// How many rows the family draws, and the seed it draws them with.
///
/// Both copied from `how_often_a_fuse_with_a_cone_refuses.rs` on purpose: this
/// file is about *that* family's six answers, so it has to draw that family's
/// rows and not a family of its own.
const DRAWS: usize = 16;

struct Draw(u64);

impl Draw {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        self.0 >> 33
    }

    fn whole(&mut self, low: i64, high: i64) -> f64 {
        let span = u64::try_from(high - low + 1).expect("a range that runs upwards");
        let picked = i64::try_from(self.next() % span).expect("inside the range") + low;
        picked as f64
    }
}

/// One row of the family: the frustum's shape, and the block's, in one place.
#[derive(Clone, Copy, Debug)]
struct Row {
    slope: f64,
    near: f64,
    far: f64,
}

impl Row {
    /// The block this row puts beside the frustum: a square-sided box as wide as
    /// the frustum's wide end, with its low corner at `(1, 1, near + 1)`.
    fn block(self) -> ([f64; 3], Vector) {
        let side = self.slope * self.far;
        (
            [side, side, self.far],
            Vector::new(1.0, 1.0, self.near + 1.0),
        )
    }

    /// Whether the block clears the frustum without touching it.
    ///
    /// The frustum is about the `z` axis and is never further out than
    /// `slope · far`; the block starts at `x = 1` and only goes further out. So
    /// one comparison settles it, and no `z` reasoning is needed at all.
    fn the_block_misses_the_frustum(self) -> bool {
        self.slope * self.far < 1.0
    }
}

/// The sixteen rows the family draws, in the order it draws them.
fn the_rows_this_family_draws() -> Vec<Row> {
    let mut draw = Draw(0x5EED);
    (0..DRAWS)
        .map(|_| {
            let slope = 2.0_f64.powi(i32::try_from(draw.next() % 3).expect("a small number") - 3);
            let far = draw.whole(6, 12);
            let near = draw.whole(1, 4);
            Row { slope, near, far }
        })
        .collect()
}

fn a_block(model: &mut Model, [along_x, along_y, along_z]: [f64; 3], at: Vector) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle with four corners");
    model
        .add_extrusion(
            &rectangle,
            along_z,
            &Isometry::translation(at).expect("a finite shift"),
        )
        .expect("a box")
}

/// A frustum and its block, each in a fresh model, so no fuse sees another's
/// leavings.
fn a_pair(row: Row) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_frustum(&mut model, row.slope, row.near, row.far);
    let (lengths, at) = row.block();
    let block = a_block(&mut model, lengths, at);
    (model, cone, block)
}

/// **Every fuse of this family that answers is one where the block never reaches
/// the frustum.**
///
/// The load-bearing half of the `6 answered of 96` pin. What that number counts
/// is fuses that returned `Ok`; what this asserts is that none of them cut
/// anything, which is a claim about the answers and not about the count.
///
/// # This does not go red when the kernel gets better
///
/// The assertion is one-sided on purpose. A row where the two solids *do* meet
/// starting to answer is the arrangement learning to order points along a
/// hyperbola, which is the whole of what this family is waiting on — so such a
/// row is **checked** here rather than forbidden: it has to hand back the same
/// answer whichever way round it is asked, and its union and intersection have to
/// account for the same material as its two inputs. What may not happen is a
/// meeting pair answering *without* being checked.
#[test]
fn the_six_block_and_frustum_fuses_that_answer_are_six_ways_of_missing() {
    let policy = Policy::certified();
    let rows = the_rows_this_family_draws();
    let clear: Vec<Row> = rows
        .iter()
        .copied()
        .filter(|row| row.the_block_misses_the_frustum())
        .collect();
    println!(
        "of {} rows, {} miss entirely: {clear:?}",
        rows.len(),
        clear.len()
    );
    assert_eq!(
        clear.len(),
        1,
        "this family is meant to draw exactly one row whose block clears the frustum, and it \
         drew {clear:?} — re-derive what the six answers are before trusting anything below"
    );

    let mut answered_missing = 0;
    let mut answered_meeting = 0;
    let mut refused = 0;
    for row in rows {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for ways_round in 0..2 {
                let (model, cone, block) = a_pair(row);
                let (arguments, tools) = if ways_round == 0 {
                    ([cone], [block])
                } else {
                    ([block], [cone])
                };
                let mut copy = model.clone();
                let Ok(answer) =
                    copy.fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())
                else {
                    refused += 1;
                    continue;
                };
                let pieces: Vec<SolidKey> = answer.solids().to_vec();
                copy.validate(&policy, &mut ProvenanceLog::new())
                    .expect("a fuse hands back a sound model or refuses");
                if row.the_block_misses_the_frustum() {
                    answered_missing += 1;
                    let want = match rule {
                        Rule::Union => 2,
                        Rule::Intersection => 0,
                        Rule::Difference => 1,
                        _ => panic!("a fourth rule landed and nothing here knows its answer"),
                    };
                    assert_eq!(
                        pieces.len(),
                        want,
                        "{rule:?} of a frustum and a block that never touch, asked way {ways_round} \
                         round, came back in {} pieces rather than {want}",
                        pieces.len()
                    );
                } else {
                    answered_meeting += 1;
                    println!(
                        "a meeting pair answered: {row:?} {rule:?} way {ways_round} round, \
                         {} pieces",
                        pieces.len()
                    );
                }
            }
        }
    }

    println!(
        "{answered_missing} answered where the block misses, {answered_meeting} where it meets, \
         {refused} refused"
    );
    assert_eq!(
        answered_missing, 6,
        "the six answers this family is pinned at are the six fuses of its one clear row, and \
         {answered_missing} of them came through"
    );
    assert_eq!(
        answered_meeting, 0,
        "a block that reaches into a frustum answered {answered_meeting} times. That is the \
         arrangement learning to order points along a hyperbola, which is good news — but it is \
         not covered by anything here, so give it a check of its own before letting this pass"
    );
}

/// The six answers, sized against closed forms and against the inputs they must
/// hand straight back.
///
/// Nothing here is a number recorded from a run. The frustum's volume is
/// `πh(R² + Rr + r²)/3` and its skin `π(R + r)·slant` plus two discs; the block's
/// are a box's. Every comparison is against `measure`'s own published bound, and
/// the bounds are required to be finite first — `assert!(x <= f64::INFINITY)`
/// cannot fail, and a check that cannot fail is not one.
///
/// **Where no bound is needed, none is used.** A fuse that takes nothing away
/// has to hand back the *same solid*, so two things are compared exactly against
/// the untouched input: its corners, bit for bit, and which of a grid of 9 741
/// points it holds. A measurement is a mesh away from the thing being compared
/// and would only blur either.
///
/// # A corner is compared as a set and not as a list, and here is why
///
/// The fuse **drops the seam** of a curved wall it never touched. A frustum goes
/// in as one cone face bounded by a single wire of four steps — round the bottom
/// circle, up the seam, back round the top, down the seam — and comes out as one
/// cone face bounded by two wires of one step each, with the seam gone. Same
/// surface, same two corners, same volume, and `Model::validate` is happy with
/// both.
///
/// That is not this unit's doing and it is not a cone's. A **round bar** put
/// through the same disjoint union loses its seam in exactly the same way, and a
/// bar has been in this kernel since M2 — `probe`-ing the two side by side is how
/// that was settled rather than assumed. So it is the fuse's normal form for a
/// curved wall, `build.rs`'s deliberately seamed bar notwithstanding, and the
/// list of corners differs by multiplicity while the *set* does not.
///
/// Comparing the set is therefore not a relaxation to make a failure go away:
/// every corner still has to be the same point to the bit, and the point grid
/// catches any difference in what the solid actually holds.
#[test]
fn a_block_that_misses_a_frustum_hands_both_of_them_back_untouched() {
    let policy = Policy::certified();
    let row = the_rows_this_family_draws()
        .into_iter()
        .find(|row| row.the_block_misses_the_frustum())
        .expect("the one row whose block clears the frustum");

    let (small, big) = (row.slope * row.near, row.slope * row.far);
    let height = row.far - row.near;
    let frustum_volume =
        core::f64::consts::PI * height * (big * big + big * small + small * small) / 3.0;
    let slant = (height * height + (big - small) * (big - small)).sqrt();
    let frustum_area = core::f64::consts::PI * ((big + small) * slant + big * big + small * small);
    let ([side, _, tall], _) = row.block();
    let block_volume = side * side * tall;
    let block_area = 2.0 * side * side + 4.0 * side * tall;

    let how_fine = Deflection::of(0.002).expect("a positive length");
    let sizes = |model: &Model, pieces: &[SolidKey]| -> Vec<(f64, f64, f64, f64)> {
        let mut measured: Vec<(f64, f64, f64, f64)> = pieces
            .iter()
            .map(|&piece| {
                let it = model
                    .solid(piece)
                    .measure(how_fine, &policy, &mut ProvenanceLog::new())
                    .expect("every piece of these answers measures");
                assert!(
                    it.volume_error().is_finite() && it.area_error().is_finite(),
                    "a bound of {} on the volume and {} on the area is not a bound to check \
                     anything against",
                    it.volume_error(),
                    it.area_error()
                );
                (
                    it.volume(),
                    it.volume_error(),
                    it.surface_area(),
                    it.area_error(),
                )
            })
            .collect();
        measured.sort_by(|one, other| one.0.total_cmp(&other.0));
        measured
    };
    let is_about = |what: &str,
                    (volume, volume_bound, area, area_bound): (f64, f64, f64, f64),
                    want_volume: f64,
                    want_area: f64| {
        assert!(
            (volume - want_volume).abs() <= volume_bound,
            "{what}: volume {volume} misses {want_volume} by more than the bound {volume_bound}"
        );
        assert!(
            (area - want_area).abs() <= area_bound,
            "{what}: area {area} misses {want_area} by more than the bound {area_bound}"
        );
    };

    let corners_of = |model: &Model, pieces: &[SolidKey]| {
        let mut corners: Vec<[u64; 3]> = Vec::new();
        for &piece in pieces {
            for face in model.solid(piece).faces() {
                for wire in face.wires() {
                    for at in wire.points() {
                        corners.push(at.coordinates().map(f64::to_bits));
                    }
                }
            }
        }
        corners.sort_unstable();
        corners.dedup();
        corners
    };

    let held_at = |model: &Model, pieces: &[SolidKey]| {
        let mut held = Vec::new();
        for x in -8..=16 {
            for y in -8..=16 {
                for z in 28..=92 {
                    let at = rcad_geom::Point::new(
                        f64::from(x) * 0.125,
                        f64::from(y) * 0.125,
                        f64::from(z) * 0.125,
                    );
                    let mut inside = false;
                    for &piece in pieces {
                        inside |= model.solid(piece).classify_point(
                            at,
                            &policy,
                            &mut ProvenanceLog::new(),
                        ) == Containment::Inside;
                    }
                    held.push(inside);
                }
            }
        }
        held
    };

    let (model, cone, block) = a_pair(row);
    let frustum_corners = corners_of(&model, &[cone]);
    let block_corners = corners_of(&model, &[block]);
    let both_corners = corners_of(&model, &[cone, block]);
    let both_inputs = sizes(&model, &[cone, block]);
    let frustum_holds = held_at(&model, &[cone]);
    let block_holds = held_at(&model, &[block]);
    let both_hold = held_at(&model, &[cone, block]);
    let counted = |held: &[bool]| held.iter().filter(|it| **it).count();
    println!(
        "of {} grid points, the frustum holds {}, the block {}",
        frustum_holds.len(),
        counted(&frustum_holds),
        counted(&block_holds)
    );
    assert!(
        counted(&frustum_holds) > 1000 && counted(&block_holds) > 1000,
        "the grid has to land inside both solids or it compares nothing"
    );

    let mut union_volume = [0.0f64; 2];
    let mut intersection_volume = [0.0f64; 2];
    let mut written = Vec::new();
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for ways_round in 0..2 {
            let (mut model, cone, block) = a_pair(row);
            let (arguments, tools) = if ways_round == 0 {
                ([cone], [block])
            } else {
                ([block], [cone])
            };
            let answer = model
                .fuse(&arguments, &tools, rule, &policy, &mut ProvenanceLog::new())
                .expect("a block that misses a frustum is a fuse this kernel answers");
            let pieces: Vec<SolidKey> = answer.solids().to_vec();
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .expect("and the model holding the answer is sound");
            let measured = sizes(&model, &pieces);
            let corners = corners_of(&model, &pieces);
            let holds = held_at(&model, &pieces);
            println!("{rule:?} way {ways_round} round: {measured:?}");

            match rule {
                Rule::Union => {
                    assert_eq!(
                        measured.len(),
                        2,
                        "two solids that never touch unite as two"
                    );
                    is_about(
                        "the frustum in the union",
                        measured[0],
                        frustum_volume,
                        frustum_area,
                    );
                    is_about(
                        "the block in the union",
                        measured[1],
                        block_volume,
                        block_area,
                    );
                    union_volume[ways_round] = measured.iter().map(|it| it.0).sum();
                    assert_eq!(
                        corners, both_corners,
                        "uniting two solids that never touch has to hand back the corners of both"
                    );
                    assert!(
                        holds == both_hold,
                        "the union holds different points from the two solids it was made of"
                    );
                }
                Rule::Intersection => {
                    assert!(
                        measured.is_empty(),
                        "two solids that never touch share nothing, and this found {measured:?}"
                    );
                    intersection_volume[ways_round] = 0.0;
                }
                Rule::Difference => {
                    assert_eq!(measured.len(), 1, "taking away nothing leaves one solid");
                    let (want, name, want_corners, want_holds) = if ways_round == 0 {
                        (
                            (frustum_volume, frustum_area),
                            "the frustum",
                            &frustum_corners,
                            &frustum_holds,
                        )
                    } else {
                        (
                            (block_volume, block_area),
                            "the block",
                            &block_corners,
                            &block_holds,
                        )
                    };
                    is_about(name, measured[0], want.0, want.1);
                    assert_eq!(
                        &corners, want_corners,
                        "taking away a solid that is nowhere near has to hand {name} back corner \
                         for corner, and it came back different"
                    );
                    assert!(
                        &holds == want_holds,
                        "taking away a solid that is nowhere near left something that holds \
                         different points from {name}"
                    );
                }
                _ => panic!("a fourth rule landed and nothing here knows its answer"),
            }
            written.push((rule, ways_round, corners));
        }
    }

    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        let both: Vec<&Vec<[u64; 3]>> = written
            .iter()
            .filter(|(which, _, _)| *which == rule)
            .map(|(_, _, corners)| corners)
            .collect();
        if matches!(rule, Rule::Difference) {
            // `A \ B` and `B \ A` are different questions; nothing to compare.
            continue;
        }
        assert_eq!(
            both[0], both[1],
            "{rule:?} of a frustum and a block wrote different corners depending on which was \
             named first"
        );
    }

    // Inclusion and exclusion, which needs no closed form: whatever the mesh
    // makes of a curved solid, the union and the intersection together have to
    // account for exactly the two inputs.
    let inputs: f64 = both_inputs.iter().map(|it| it.0).sum();
    let bound: f64 = both_inputs.iter().map(|it| it.1).sum::<f64>() * 2.0;
    assert!(bound.is_finite(), "a bound of {bound} checks nothing");
    for ways_round in 0..2 {
        let together = union_volume[ways_round] + intersection_volume[ways_round];
        println!("union + intersection {together} against the two inputs {inputs}");
        assert!(
            (together - inputs).abs() <= bound,
            "the union and the intersection together hold {together} where the two inputs hold \
             {inputs}, which is further apart than the bound {bound}"
        );
    }
}
