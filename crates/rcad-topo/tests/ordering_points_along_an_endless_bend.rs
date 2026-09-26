//! What a block takes out of a frustum, now that a curve with no ends and a
//! bend in it can be put in order.
//!
//! §49.6 published a rule on a spike's word: *the frustum family answers iff the
//! cone, carried up to the block's roof, is still narrower than the distance to
//! the nearest side wall*, over a grid of eighty-four. §15.6 says a published
//! bound needs a test, and this is that test — run against the ordering rather
//! than against the refusal, so what it pins is the rule's **death**. The
//! boundary that rule describes is a block wall parallel to the cone's axis,
//! which cuts a hyperbola, and a hyperbola is exactly what `arrange.rs`'s
//! `HowItRuns::AlongABend` now orders.
//!
//! # What the grid is
//!
//! The same eighty-four configurations `fuses_that_must_agree.rs` built for the
//! rule: three slopes, every block from six to twelve across, and the frustum's
//! axis at every whole place that leaves the wide cap a unit clear of both
//! walls. Written out here rather than shared, because the two files disagree on
//! purpose — that one asserts the rule and this one asserts what replaced it,
//! and a shared helper would have to hold both.
//!
//! # The ordering is not the last thing in the way, and this file says which is
//!
//! Ordering along the bend takes the grid from thirty-eight answers to
//! fifty-three, and naming the two arcs of a cone's rim circle apart takes it to
//! fifty-seven; **not one configuration refuses on the ordering any more**. The
//! twenty-seven that still refused all gave up in `fuse/select.rs`, which §49.6
//! lists among what stands under its own name — twenty-five of them for want of
//! any point inside the piece, and two where a point was found and no solid
//! could place it. §217 split those two names apart; §224.4 answered the two,
//! and §251 the twenty-five, so all eighty-four answer now, and
//! [`every_answer_on_the_grid_holds_what_the_shapes_hold_by_hand`] holds each
//! of them against the frustum's closed form.

// `pairs` describes its frustum through `cones::a_frustum`.
mod cones;
mod pairs;

use std::collections::BTreeMap;

use rcad_geom::Deflection;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

use pairs::FrustumAndBlock;

/// Every configuration on §49.6's grid, in the order that section counted them.
fn the_grid() -> Vec<FrustumAndBlock> {
    let mut grid = Vec::new();
    for up_per_unit_wider in [2u32, 4, 8] {
        for side in 6u32..=12 {
            for at in 3u32..=(side - 3) {
                grid.push(FrustumAndBlock {
                    side,
                    narrow: 1,
                    wide: 2,
                    up_per_unit_wider,
                    at: [at, at],
                });
            }
        }
    }
    grid
}

/// How one fuse of the grid went, in the only three ways it can go.
#[derive(PartialEq, Eq)]
enum Went {
    Answered,
    NothingCouldOrderTheCurve,
    GaveUpOn(String),
}

/// Union the block with the frustum standing up through its floor.
///
/// **The answers are checked and not just counted.** A fuse that answered by
/// writing a broken solid would read as a better rate than a fuse that refused,
/// so every answer goes through [`Model::validate`].
fn how_the_fuse_went(pair: FrustumAndBlock) -> Went {
    let policy = Policy::certified();
    let (mut model, keys) = pair.build();
    match model.fuse(
        &keys[..1],
        &keys[1..],
        Rule::Union,
        &policy,
        &mut ProvenanceLog::new(),
    ) {
        Ok(_) => {
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .expect("a fuse hands back a sound model or refuses");
            Went::Answered
        }
        Err(Error::ACrossingThisCannotBeOrderedAlong) => Went::NothingCouldOrderTheCurve,
        Err(Error::Undecidable { records }) => Went::GaveUpOn(
            records
                .records()
                .last()
                .map_or("nothing named", |record| record.predicate())
                .to_owned(),
        ),
        Err(why) => Went::GaveUpOn(format!("{why}")),
    }
}

/// **Nothing on the grid is refused for want of an order along its curve any
/// more**, and that is the whole of what the third `HowItRuns` variant buys.
///
/// Forty-four of these eighty-four used to come back
/// [`Error::ACrossingThisCannotBeOrderedAlong`], because a block standing
/// upright beside a cone has four walls parallel to that cone's axis and every
/// one of them cuts it in a hyperbola. Run this against a tree without the bend
/// and it fails on the first configuration it reaches with a wall through the
/// cone — `side: 6, at: [3, 3]`, at a slope of one half.
#[test]
fn not_one_block_on_the_grid_is_refused_for_want_of_an_order_along_its_curve() {
    let mut refused = Vec::new();
    for pair in the_grid() {
        if how_the_fuse_went(pair) == Went::NothingCouldOrderTheCurve {
            refused.push(pair);
        }
    }
    assert!(
        refused.is_empty(),
        "these still have a curve nothing can order along: {refused:#?}"
    );
}

/// How many of the eighty-four answer, and what the rest give up on instead.
///
/// **Fifty-seven of eighty-four**, up from the thirty-eight §49.6 measured, and
/// the twenty-seven that are left all give up in one place that is not the
/// ordering. §15.10 asks what the cases that moved have in common: all nineteen
/// are configurations the old rule turned down, every one of them has a block
/// wall cutting the cone in a hyperbola, and every one of them is a wall the
/// face can still be sorted out around afterwards.
///
/// **Not eighty-four of eighty-four**, which §50.5 named as this lane's bar. The
/// bar is not met and the reason is still not the ordering. All twenty-seven are
/// `fuse/select.rs`, which §49.6 lists as its own open row and which no work on
/// `arrange.rs` reaches — and since §217 they are seen to be **two** walls and
/// not one: twenty-five where no candidate was inside the piece at all, and
/// **two** where one was found and no solid could place it. They wore a single
/// name until that split, which is the whole of what §216.4 found.
///
/// **It was 53 answered with 27 of that wall and 4 of *an edge would have 4
/// faces along it*.** §206's bulge count carried those four past `select.rs` and
/// into `assemble`'s edge naming, where `which_arc_of_a_circle_this_is` read a
/// ball-and-plane circle only and so called two arcs of a cone's rim circle one
/// edge. Naming the two arcs apart answers all four, which is why the second
/// wall is gone rather than smaller. §206.4.
///
/// **It was 59, and the second wall was gone.** The two that found a point and
/// could not place it were the flush pair, whose top cap lies in the block's
/// roof: the cap piece's point lay on the frustum's cap, and that cap, a disk
/// with one corner, was wound by its corners and held nothing. §224.4 asks a
/// flat face with a curved side through the solid's own point test instead.
///
/// **It is 84, and the first wall is gone too** (§251). The twenty-five were a
/// block wall's piece inside the cone carried on above the frustum's top cap:
/// bounded by one arc of the hyperbola the wall cuts and the chord the roof cuts
/// across it, so two corners and no polygon. `select.rs` now counts a
/// hyperbola arc's bulge on the arc's own half of the cone and aims a candidate
/// square off the chord at the cone, so that piece gets a point, is outside the
/// frustum, and stays in the union.
/// [`every_answer_on_the_grid_holds_what_the_shapes_hold_by_hand`] is what says
/// the twenty-five are right and not just answered.
#[test]
fn all_eighty_four_answer() {
    let mut answered = 0;
    let mut elsewhere: Vec<String> = Vec::new();
    for pair in the_grid() {
        match how_the_fuse_went(pair) {
            Went::Answered => answered += 1,
            Went::NothingCouldOrderTheCurve => {
                panic!("{pair:?} has a curve nothing can order along")
            }
            Went::GaveUpOn(named) => elsewhere.push(named),
        }
    }
    println!(
        "a frustum up through a block, on §49.6's grid of {}: {answered} answered, {} gave up",
        the_grid().len(),
        elsewhere.len()
    );
    let mut walls: BTreeMap<&str, usize> = BTreeMap::new();
    for named in &elsewhere {
        *walls.entry(named.as_str()).or_default() += 1;
    }
    println!("and the walls they gave up at are {walls:?}");
    assert_eq!(
        walls,
        BTreeMap::new(),
        "these are the walls the grid's refusals were measured against; re-derive them when they \
         move rather than widening the test (§19.1)"
    );
    assert_eq!(
        answered, 84,
        "the grid answered 38 of 84 before the bend could be ordered, 53 after, 57 once a cone's \
         rim circle had its two arcs named apart, 59 once a flat cap was asked what it holds, and \
         84 once a hyperbola arc's bulge was counted on its own half of the cone"
    );
}

/// Every union and every intersection on the grid holds what the frustum and
/// the block hold, worked out by hand.
///
/// **Only the frustum's closed form is needed**, because on this grid the
/// frustum's own face never reaches a wall: its widest cap has radius two and
/// its axis is at least three from every wall. So a wall cuts only the cone
/// carried on past the face, and what the two solids share is the frustum
/// between the floor and whichever of its top cap and the roof comes first.
/// With the frustum dropped as [`FrustumAndBlock::build`] drops it, its radius
/// at height `z` is `ρ(z) = 1 + (z + 1)/u` from `z = −1` to `z = u − 1`, for
/// `u` up per unit wider, and the volume between two heights is
/// `π u (ρ(b)³ − ρ(a)³) / 3`. The union is the block, the unit below the floor,
/// and what pokes out of the roof — only the steepest cone in the smallest
/// block, whose top cap is at seven against a roof at six.
///
/// The tolerance is what `measure` declares and nothing chosen here.
#[test]
fn every_answer_on_the_grid_holds_what_the_shapes_hold_by_hand() {
    let policy = Policy::certified();
    let how_fine = Deflection::of(0.01).expect("a length");
    for pair in the_grid() {
        let up = f64::from(pair.up_per_unit_wider);
        let side = f64::from(pair.side);
        let radius_at = |z: f64| 1.0 + (z + 1.0) / up;
        let frustum_between = |from: f64, to: f64| {
            core::f64::consts::PI * up * (radius_at(to).powi(3) - radius_at(from).powi(3)) / 3.0
        };
        let top = up - 1.0;
        let shared = frustum_between(0.0, top.min(side));
        let out_of_the_roof = if top > side {
            frustum_between(side, top)
        } else {
            0.0
        };
        let united = side.powi(3) + frustum_between(-1.0, 0.0) + out_of_the_roof;
        for (rule, held) in [(Rule::Union, united), (Rule::Intersection, shared)] {
            let (mut model, keys) = pair.build();
            let answer = model
                .fuse(
                    &keys[..1],
                    &keys[1..],
                    rule,
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{pair:?} {rule:?} answers: {why:?}"));
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{pair:?} {rule:?} is sound: {why:?}"));
            assert_eq!(answer.solids().len(), 1, "{pair:?} {rule:?}: one solid");
            let (volume, worst) = how_much_it_holds(&model, answer.solids(), how_fine);
            assert!(
                (volume - held).abs() <= worst,
                "{pair:?} {rule:?}: holds {held} by hand and measured {volume}, further off than \
                 the {worst} the mesh declares"
            );
        }
    }
}

/// §49.6's rule, asked of the same grid: it is now false, and here is by how
/// much.
///
/// A rule that no longer holds is worth a measurement rather than a deleted
/// line. The rule allows forty of the eighty-four and turns down forty-four.
/// **All forty-four answer now** — fifteen until a cone's rim circle had its
/// two arcs named apart, nineteen after that, and the other twenty-five since
/// §251 counted a hyperbola arc's bulge. Of the forty it allows, all forty
/// answer. The last two were the pair whose top cap lands exactly in the
/// block's roof, which §49.6 counted separately as a flush cap, and they answer
/// since §224.4 asks the frustum's one-cornered cap what it holds rather than
/// winding its corners.
#[test]
fn the_rule_that_said_which_frustums_a_block_could_cut_is_wrong_about_all_it_turned_down() {
    let mut the_rule_said_yes_and_it_answered = 0;
    let mut the_rule_said_no_and_it_answered = 0;
    for pair in the_grid() {
        if how_the_fuse_went(pair) != Went::Answered {
            continue;
        }
        if the_cone_stays_clear_of_the_walls(&pair) {
            the_rule_said_yes_and_it_answered += 1;
        } else {
            the_rule_said_no_and_it_answered += 1;
        }
    }
    assert_eq!(
        the_rule_said_yes_and_it_answered, 40,
        "the rule allowed 40, and the flush pair among them answers since §224.4"
    );
    assert_eq!(
        the_rule_said_no_and_it_answered, 44,
        "the rule turned down 44 and this many of them answer now"
    );
}

/// Whether the cone, carried up to the block's roof, stays inside the block's
/// side walls — §49.6's rule, written the way that section wrote it.
///
/// The frustum's apex sits one below the block's floor plus its own lower cap's
/// height, because [`FrustumAndBlock::build`] drops it by that much.
fn the_cone_stays_clear_of_the_walls(pair: &FrustumAndBlock) -> bool {
    let apex = -1.0 - pair.near();
    let wide_at_the_roof = (f64::from(pair.side) - apex) * pair.slope();
    let nearest_wall = pair
        .at
        .iter()
        .flat_map(|&along| [along, pair.side - along])
        .min()
        .expect("a block has walls");
    wide_at_the_roof < f64::from(nearest_wall)
}

/// A count says a fuse did not give up and nothing about whether it was right.
/// This says what one of the answers **holds**, against arithmetic done by hand.
///
/// The block is six across with the frustum's axis in the middle of its floor,
/// the cone goes out one unit for every two up, and the frustum runs from a cap
/// of radius one a unit below the floor to a cap of radius two a unit above it.
/// So the apex sits three below the floor, the cone is a unit and a half across
/// where the floor cuts it, and everything above the floor is inside the block —
/// the wide cap is two across and the nearest wall three away. The union is the
/// block plus the one unit sticking out below it, a cone frustum of radii one
/// and one and a half and height one: `216 + π(1 + 1.5 + 2.25)/3`.
///
/// **The tolerance is the mesher's own and is not a number chosen here.**
/// `Measurement::volume_error` is what the mesher says sits between its
/// triangles and the surface they stand in for, so the claim is *the answer is
/// right to within what the measurement declares* and there is nothing to tune.
/// Asserted at two deflections a hundred apart, because a bound that holds at
/// one is a bound that might have been loose.
#[test]
fn a_frustum_up_through_a_block_unions_to_the_block_plus_the_bit_below_it() {
    let policy = Policy::certified();
    let pair = FrustumAndBlock {
        side: 6,
        narrow: 1,
        wide: 2,
        up_per_unit_wider: 2,
        at: [3, 3],
    };
    let (mut model, keys) = pair.build();
    let answer = model
        .fuse(
            &keys[..1],
            &keys[1..],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("the block's walls cut this cone in hyperbolas, which are ordered now");
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("a fuse hands back a sound model");

    let below_the_floor = core::f64::consts::PI * (1.0 + 1.5 + 2.25) / 3.0;
    let held = 6.0 * 6.0 * 6.0 + below_the_floor;
    for how_fine in [0.1, 0.001] {
        let (volume, worst) = how_much_it_holds(
            &model,
            answer.solids(),
            Deflection::of(how_fine).expect("a length"),
        );
        assert!(
            (volume - held).abs() <= worst,
            "a block of side six with a unit of frustum below it holds {held}, and at a \
             deflection of {how_fine} the answer measured {volume}, further off than the \
             {worst} the mesh declares"
        );
    }
}

/// What an answer holds, and how much of that the mesh will not vouch for.
fn how_much_it_holds(model: &Model, solids: &[SolidKey], how_fine: Deflection) -> (f64, f64) {
    solids
        .iter()
        .map(|&solid| {
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(how_fine, &Policy::certified(), &mut ProvenanceLog::new())
                .expect("a solid this size measures");
            (measured.volume(), measured.volume_error())
        })
        .fold((0.0, 0.0), |(volume, error), (more, worse)| {
            (volume + more, error + worse)
        })
}
