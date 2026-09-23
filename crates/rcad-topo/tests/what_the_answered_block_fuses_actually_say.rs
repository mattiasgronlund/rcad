//! What the answers in `a frustum against a block` actually are.
//!
//! `how_often_a_fuse_with_a_cone_refuses.rs` pins that family's answered count and
//! re-derives that floor every run. A count is not a check: a fuse returning `Ok`
//! says a model came back, and says nothing at all about what is in it. §4.1 ranks
//! a wrong answer below a refusal, so the count on its own was pinning the
//! cheapest half of the claim.
//!
//! # Every answer is a draw where nothing touches
//!
//! The family draws 16 rows and asks six fuses of each — union, intersection and
//! difference, both ways round. **All 24 answers come from rows where the block
//! misses the frustum altogether.**
//!
//! How far apart they are takes one comparison. The block is put down with its
//! low corner at `(1, 1, near + 1)` and only ever runs further out in `x` and `y`,
//! so every point of it is at least `√2` from the `z` axis. The frustum is about
//! that axis with its apex at the origin and is never further out than
//! `slope · far`. So `slope · far < √2` makes them disjoint, and no `z` reasoning
//! is needed at all.
//!
//! Over this grid — slope one of `1/8`, `1/4`, `1/2`, and `far` a whole number
//! from 6 to 12 — that is `slope = 1/8` with `far ≤ 11`, and five of the sixteen
//! rows land there. Four answer all six of their fuses; the fifth answers none.
//!
//! **Three of those four used to answer only half their fuses**, and what filled
//! them in was §215's same-surface arm in `walk`: the union both ways round and
//! `frustum \ block` were the 9 that stopped on one cone twice, and they are the
//! difference between 15 answers here and 24.
//!
//! **An earlier version of this file said `slope · far < 1`, and called it
//! exact.** It is sound but not exact: it compares the block's nearest `x` against
//! the frustum's widest radius and never looks at `y`, so the three rows at
//! `slope · far = 1` are disjoint for a reason it cannot see. When they began to
//! answer, the test below read them as a block reaching into a frustum and said so
//! in as many words. They are nothing of the kind, and the count that would have
//! been re-pinned would have recorded the opposite of what happened.
//!
//! So the honest sentence about that family is **not** "a block can now take a
//! piece out of a frustum". It is:
//!
//! > No block in this family has ever taken a piece out of a frustum. The fuses
//! > that answer are ways of asking about a block and a frustum that do not meet,
//! > and every one of them hands back an input untouched.
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
    /// The block's low corner is at `(1, 1, near + 1)` and it only runs further
    /// out in both `x` and `y`, so every point of it is at least `√2` from the
    /// `z` axis. The frustum is about that axis and is never further out than
    /// `slope · far`. So one comparison settles it, and no `z` reasoning is
    /// needed at all.
    ///
    /// Comparing against `1` instead — the block's nearest `x` — is sound and is
    /// what this used to do, but it ignores `y` and so misses the three rows at
    /// `slope · far = 1`, which are disjoint too.
    fn the_block_misses_the_frustum(self) -> bool {
        self.slope * self.far < core::f64::consts::SQRT_2
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
/// The load-bearing half of the `15 answered of 96` pin. What that number counts
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
fn every_block_and_frustum_fuse_that_answers_is_a_way_of_missing() {
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
        5,
        "this family is meant to draw five rows whose blocks clear the frustum, and it drew \
         {clear:?} — re-derive which fuses answer before trusting anything below"
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
                    println!(
                        "a missing pair answered: {row:?} {rule:?} way {ways_round} round, \
                         {} pieces",
                        pieces.len()
                    );
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
        answered_missing, 24,
        "the answers this family is pinned at are all fuses of its clear rows — all six from each \
         of the four rows at slope · far = 0.75 and slope · far = 1, with the fifth clear row at \
         slope · far = 1.25 answering none — and {answered_missing} of them came through. The \
         rows that answered are printed above; re-derive from those rather than from this \
         sentence"
    );
    assert_eq!(
        answered_meeting, 0,
        "a block that reaches into a frustum answered {answered_meeting} times. That may be the \
         arrangement learning to order points along a hyperbola, which is good news — but it is \
         not covered by anything here, so give it a check of its own before letting this pass. \
         Check first that the pair really does meet: the last time this fired, all of them were \
         disjoint and `the_block_misses_the_frustum` was the thing that was wrong"
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
                    .expect("this model made the key")
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
            for face in model.solid(piece).expect("this model made the key").faces() {
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
                        inside |= model
                            .solid(piece)
                            .expect("this model made the key")
                            .classify_point(at, &policy, &mut ProvenanceLog::new())
                            == Containment::Inside;
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
