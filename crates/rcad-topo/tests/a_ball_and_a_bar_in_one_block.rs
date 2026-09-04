//! A block with a ball half sunk in its floor and a bar driven through it.
//!
//! §48's curved trio. The block's floor is one plane crossed by two round
//! surfaces — the ball's skin and the bar's wall — so the arrangement asks where
//! plane, sphere and pipe meet, and
//! [`rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat`] is what comes
//! back: one plane short of what §31.2 row 8 can solve.
//!
//! What this file pins is that the question was never worth asking. The ball
//! stands clear of the bar, the sphere and the pipe have **no point in common at
//! all**, and three surfaces two of which share nothing share nothing. So there
//! is no corner to place and the fuse has nothing to give up on.
//!
//! # The skip is a proof and not a distance
//!
//! Every entry below moves the ball and the bar towards each other and asks
//! again. They fuse while `intersect` answers `SurfaceMeet::Nothing`, down to a
//! gap of a millionth of a unit, and the moment they *touch* — one point in
//! common, no more — the refusal comes back by name. Nothing here is compared
//! against a size.
//!
//! # What is still refused, and it is no longer the triple
//!
//! The three fuse when they are handed over **together**. Fused a pair at a
//! time, the ball goes in first and leaves the block's floor with a round hole,
//! and the bar's circle is then a closed cut in a face one of whose rim wires is
//! a single closed edge. That stops on `CannotTellTheCellsOfAFaceApart`, which
//! §39.4 already lists as a gap and marks undemonstrated;
//! [`the_same_three_one_pair_at_a_time_stop_somewhere_else`] is the
//! demonstration and says what the fan cannot count.
//!
//! # And what a point inside the block still cannot be asked
//!
//! `SolidView::classify_point` answers about every point outside the block —
//! inside the bar's stubs and outside everything else — and comes back
//! `Undecidable` about a point in the middle of it, because every ray out of
//! there has to be followed across the floor, and the floor is a plane with two
//! round holes in it. That is the classifier's own gap and not this fuse's; the
//! answer's volume and its face count are what pin the shape here.

use core::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, Model, Rule, SolidKey};

/// How fine the meshes every volume here is read off are.
const A_DEFLECTION: f64 = 0.02;

/// How far from the closed form a volume measured at [`A_DEFLECTION`] may be.
///
/// Half a per cent, the same ceiling `degenerate_curved_fuses.rs` uses and for
/// the same reason. Nothing in the kernel reads it.
const HOW_FAR_A_MESH_MAY_BE: f64 = 0.005;

/// The block is seven across.
const SIDE: f64 = 7.0;

/// Both round solids have radius one.
const RADIUS: f64 = 1.0;

/// A block with a ball half sunk in its floor and a bar right through it.
///
/// The ball's centre sits on the floor plane at `x = ball_at`, the bar stands up
/// `z` at `x = 5.0`, and both are halfway across in `y`. The bar is long enough
/// to stick two units out of either end.
fn a_block_a_ball_and_a_bar(ball_at: f64) -> (Model, [SolidKey; 3]) {
    let mut model = Model::new(Scale::millimetres(10.0).expect("a working size"));
    let halfway = SIDE / 2.0;
    let block = model
        .add_cube(SIDE, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a whole-number cube");
    let ball = model
        .add_ball(
            RADIUS,
            &Isometry::translation(Vector::new(ball_at, halfway, 0.0)).expect("a shift"),
        )
        .expect("a whole-number ball");
    let bar = model
        .add_round_bar(
            RADIUS,
            SIDE + 4.0,
            &Isometry::translation(Vector::new(5.0, halfway, -2.0)).expect("a shift"),
        )
        .expect("a bar long enough to go right through");
    (model, [block, ball, bar])
}

/// Unite all three in one fuse: the block against the ball and the bar together.
///
/// The block's floor is then crossed by two round surfaces that are both
/// **cuts**, and its own rim is the plain square it started as.
fn united_all_at_once(ball_at: f64) -> (Model, Result<Vec<SolidKey>, Error>) {
    let (mut model, [block, ball, bar]) = a_block_a_ball_and_a_bar(ball_at);
    let answer = model
        .fuse(
            &[block],
            &[ball, bar],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|fused| fused.solids().to_vec());
    (model, answer)
}

/// Unite the three in the order they were built, one fuse at a time.
fn united_one_at_a_time(ball_at: f64) -> (Model, Result<Vec<SolidKey>, Error>) {
    let (mut model, [block, ball, bar]) = a_block_a_ball_and_a_bar(ball_at);
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let first = match model.fuse(&[block], &[ball], Rule::Union, &policy, &mut log) {
        Ok(fused) => fused.solids().to_vec(),
        Err(why) => return (model, Err(why)),
    };
    let second = model
        .fuse(&first, &[bar], Rule::Union, &policy, &mut log)
        .map(|fused| fused.solids().to_vec());
    (model, second)
}

/// The trio fuses, and the answer is the right solid.
///
/// The ball stands three units from the bar's axis, radii one and one, so the
/// sphere and the pipe are two units apart at their nearest and no point lies on
/// both. The volume is the block, plus the half of the ball hanging below the
/// floor, plus the two two-unit stubs of bar sticking out of either end.
#[test]
fn a_ball_and_a_bar_that_miss_each_other_fuse_into_one_block() {
    let (model, answer) = united_all_at_once(2.0);
    let solids = answer.expect("a ball and a bar that miss each other unite with a block");
    assert_eq!(solids.len(), 1, "one block with two lumps on it");

    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the answer is a valid solid");

    let solid = model.solid(solids[0]);
    assert_eq!(
        solid.faces().count(),
        11,
        "six of the block, the ball's lower half, and the bar's wall and cap \
         at each end"
    );

    let closed_form =
        SIDE * SIDE * SIDE + 2.0 / 3.0 * PI * RADIUS * RADIUS * RADIUS + 4.0 * PI * RADIUS * RADIUS;
    let measured = volume_of(&model, &solids);
    assert!(
        (0.0..HOW_FAR_A_MESH_MAY_BE).contains(&((closed_form - measured) / closed_form)),
        "the answer is a block with half a ball and two bar stubs on it, and a \
         mesh inside a round skin under-measures it: {measured} against {closed_form}"
    );
    // Leaving the ball's own half out would come to 355.57, which is 0.59% below
    // the closed form and five times the width of the band above. The measured
    // 357.59 is 0.019% below it. A count of faces and a volume within half a per
    // cent are two different claims and this file makes both (§47.2).
    assert!(
        measured > SIDE * SIDE * SIDE + 4.0 * PI * RADIUS * RADIUS + 2.0,
        "the ball's own half is in the answer and not merely a hole in the floor: \
         {measured}"
    );

    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    assert_eq!(
        solid.classify_point(Point::new(5.0, 3.25, -1.5), &policy, &mut log),
        Containment::Inside,
        "a unit and a half under the floor, inside the bar stub"
    );
    assert_eq!(
        solid.classify_point(Point::new(5.0, 3.25, SIDE + 1.5), &policy, &mut log),
        Containment::Inside,
        "the stub out of the top of the block"
    );
    for (x, y) in [(3.25, 1.75), (2.0, 3.25), (3.5, 3.25), (6.3, 5.7)] {
        for z in [-1.5, SIDE + 1.5] {
            assert_eq!(
                solid.classify_point(Point::new(x, y, z), &policy, &mut log),
                Containment::Outside,
                "nothing but the bar reaches past the block at ({x}, {y}, {z})"
            );
        }
    }
}

/// The gap the skip needs is not a size.
///
/// The ball's centre is a millionth of a unit further from the bar's axis than
/// the two radii add up to. The sphere and the pipe still share nothing, the
/// proof still goes through, and the trio still fuses.
#[test]
fn a_ball_a_millionth_of_a_unit_clear_of_the_bar_still_fuses() {
    let (_, answer) = united_all_at_once(5.0 - 2.0 - 1.0e-6);
    answer.expect("a millionth of a unit of clear air is clear air");
}

/// And the moment they touch, the refusal comes back.
///
/// The ball's centre is exactly two units from the bar's axis, so the sphere
/// rests against the pipe at one point. One point in common is not none, the
/// proof does not go through, and the fuse asks where plane, sphere and pipe
/// meet — which is the question it cannot answer.
#[test]
fn a_ball_resting_against_the_bar_is_refused_by_name() {
    let (_, answer) = united_all_at_once(3.0);
    assert!(
        matches!(
            answer,
            Err(Error::Geometry(
                rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat
            ))
        ),
        "a ball touching the bar is a triple that really does meet: {answer:?}"
    );
}

/// The same where they overlap outright.
#[test]
fn a_ball_driven_into_the_bar_is_refused_by_name() {
    let (_, answer) = united_all_at_once(3.5);
    assert!(
        matches!(
            answer,
            Err(Error::Geometry(
                rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat
            ))
        ),
        "a ball crossing the bar is a triple that really does meet: {answer:?}"
    );
}

/// What stops the same three solids fused **one pair at a time**, now that the
/// triple no longer does.
///
/// This is the refusal that was standing behind the one this file closed, and it
/// is a different one: `CannotTellTheCellsOfAFaceApart`, §39.4's row already
/// classified a **gap** and marked undemonstrated. Here is the demonstration.
///
/// Fusing the ball in first leaves the block's floor with a round *hole* in it,
/// and then the bar's circle is a closed cut in a face one of whose rim wires is
/// a single closed edge. `arrange.rs`'s `how_often_the_rim_winds_round` fans a
/// wire's corners into a polygon; a wire with one corner fans into the segment
/// from that corner to itself, which no turn can be read off, and the whole
/// count comes back `None`. So the bar's lump gets no winding, nothing tells the
/// inside of the floor from the outside of it for that lump, and two walks that
/// wind different numbers of times end up in one cell.
///
/// **Fixing that is a different lane.** A rim with a curved wire needs a winding
/// count that is not a fan over corners, and skipping such a wire would answer
/// wrongly for a cut inside the hole rather than refusing.
#[test]
fn the_same_three_one_pair_at_a_time_stop_somewhere_else() {
    let (_, answer) = united_one_at_a_time(2.0);
    assert!(
        matches!(answer, Err(Error::CannotTellTheCellsOfAFaceApart { .. })),
        "the triple is no longer what stops this, and what does is §39.4's own row: {answer:?}"
    );
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
                .expect("a solid this file built can be measured")
                .volume()
        })
        .sum()
}
