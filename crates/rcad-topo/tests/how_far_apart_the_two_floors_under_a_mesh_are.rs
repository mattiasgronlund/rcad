//! Two floors sit under a mesh, and this is how far apart they are.
//!
//! `mesh.rs` publishes both of them for a round bar of radius one and two tall
//! at the origin, and `error.rs` publishes both of them for a unit ball at the
//! origin, along with the seventy-nine that separates them. Four numbers and a
//! ratio, and §15.6 says a published bound needs a test. This is that test.
//!
//! - **Where a point can be *placed*** is what a **caller's own** deflection is
//!   let through against. Ask for less and the mesher refuses with
//!   [`Error::DeflectionBelowWhatAMeshCanReach`]; ask for that or more and the
//!   number goes through to the walk, which then answers for it itself.
//! - **How fine a mesh can be *cut*** is what the mesher measures **its own**
//!   number against, and it is what the refusal carries as `closest`.
//!
//! # How each is read, and neither is read off its own formula
//!
//! The cutting floor is `closest` out of the refusal, which is public. The
//! placing floor is not carried anywhere, so it is **hunted for**: it is the
//! deflection at which `DeflectionBelowWhatAMeshCanReach` stops being the
//! answer, and a bisection on the answer's *name* finds it exactly. Below it the
//! guard refuses; at it and above, the guard waves the number through and
//! `rcad_geom::walk` refuses instead — a different refusal from a different
//! place, which is what makes the boundary readable at all. **Which** of the
//! walk's refusals it is depends on the shape, and
//! `a_bar_leaves_the_walk_room_to_place_a_point_where_a_ball_leaves_it_none`
//! is where that is pinned and where what it says is spelled out.
//!
//! Both shapes sit at the origin, and that is what makes this cheap: the
//! refusals either side of the boundary are immediate. **A solid far from the
//! origin cannot be hunted this way.** Out there the first deflection above the
//! placing floor is one the walk *accepts*, so the bisection meshes for real and
//! does not come back; a probe on a solid four thousand million units tall was
//! still going after two minutes and had to be killed.
//!
//! # The ratio is a fact about the added term, not about either shape
//!
//! A bar and a ball share no face, no surface and no floor, and they agree on
//! the ratio between their two floors to the last bit of an `f64`. That is what
//! says the seventy-nine belongs to the term
//! `the_finest_deflection_the_walk_will_take` adds on top of
//! `how_far_a_mesh_point_of_this_face_rounds`, and not to a bar or to a ball. So
//! the ratio is asserted **equal between the two**, and not merely near 78.93
//! twice.

use rcad_geom::{Deflection, Isometry, Scale};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Mesh, Model, SolidKey};

/// How close a point of a bar of radius one and two tall at the origin can be
/// placed, published by `mesh.rs`'s `the_finest_deflection_the_walk_will_take`.
const A_BAR_PLACES_A_POINT_TO: f64 = 9.2148511043888e-13;

/// How fine a mesh of that bar can be cut, published beside it.
const A_BAR_IS_CUT_TO: f64 = 7.273230840146953e-11;

/// How close a point of a unit ball at the origin can be placed.
///
/// Halved when a ball stopped being charged from its poles and started being
/// charged from its centre: its poles are a whole radius out, so `corner +
/// radius` was `2` for a face that reaches `1`. See
/// `where_a_ball_and_a_ring_are_measured_from.rs`. **`error.rs` still names the
/// old pair** on [`Error::DeflectionBelowWhatAMeshCanReach`] and has to move
/// with these two.
const A_BALL_PLACES_A_POINT_TO: f64 = 3.0716170347962663e-13;

/// How fine a mesh of that ball can be cut, halved for the same reason.
const A_BALL_IS_CUT_TO: f64 = 2.4244102800489844e-11;

/// The number both of them come to when the second is divided by the first.
///
/// `error.rs` says "78.93 times apart" and `mesh.rs` says "seventy-nine times
/// more"; this is what the division actually gives, on both shapes, bit for bit.
const HOW_FAR_APART_THE_TWO_FLOORS_ARE: f64 = 78.92944506377209;

/// Finer than either floor by three hundred decades, so the refusal that carries
/// the cutting floor is the answer whatever else moves.
const FINER_THAN_ANYTHING_CAN_BE_CUT: f64 = 1e-300;

/// Both of a bar's floors, and both of them to the last digit published.
#[test]
fn a_round_bar_at_the_origin_places_a_point_a_thousand_times_closer_than_it_can_be_cut() {
    let (model, bar) = a_round_bar();
    assert_eq!(how_fine_a_mesh_of_it_is_cut(&model, bar), A_BAR_IS_CUT_TO);
    assert_eq!(
        how_close_it_places_a_point(&model, bar),
        A_BAR_PLACES_A_POINT_TO
    );
}

/// Both of a ball's floors, which are the pair `error.rs` publishes on the
/// refusal itself.
#[test]
fn a_unit_ball_at_the_origin_places_a_point_a_thousand_times_closer_than_it_can_be_cut() {
    let (model, ball) = a_unit_ball();
    assert_eq!(how_fine_a_mesh_of_it_is_cut(&model, ball), A_BALL_IS_CUT_TO);
    assert_eq!(
        how_close_it_places_a_point(&model, ball),
        A_BALL_PLACES_A_POINT_TO
    );
}

/// The seventy-nine, on two solids that share nothing.
///
/// Neither shape gets to be the one that decides it. If the ratio were a fact
/// about a cap circle, or about a seam, these two would not agree — and they
/// agree exactly, which is the whole of the claim.
#[test]
fn a_bar_and_a_ball_put_the_same_distance_between_their_two_floors() {
    let (model, bar) = a_round_bar();
    let bar_apart =
        how_fine_a_mesh_of_it_is_cut(&model, bar) / how_close_it_places_a_point(&model, bar);

    let (model, ball) = a_unit_ball();
    let ball_apart =
        how_fine_a_mesh_of_it_is_cut(&model, ball) / how_close_it_places_a_point(&model, ball);

    assert_eq!(bar_apart, HOW_FAR_APART_THE_TWO_FLOORS_ARE);
    assert_eq!(ball_apart, HOW_FAR_APART_THE_TWO_FLOORS_ARE);
    assert_eq!(
        bar_apart, ball_apart,
        "a bar puts {bar_apart} between its two floors and a ball puts {ball_apart}, \
         so the distance between them is a fact about one of the two shapes"
    );
}

/// Which of the walk's own refusals the first deflection past the guard gets,
/// and why the two shapes give different ones.
///
/// A bar's face reaches `2` where the cap circle it walks reaches `1`, so when
/// the mesher's floor runs out the walk still has a factor of two of room to
/// place a point in, and what it runs out of instead is its cap on how many
/// points one crossing may hold. **A ball's face reaches `1` and the meridian it
/// walks reaches `1`**: the mesher's floor and the walk's are the same floor,
/// and the walk hits its own on the first deflection the guard lets past.
///
/// That agreement is worth an assertion of its own, because it is what says a
/// ball is charged for exactly what it covers and not for twice it. Charged from
/// its poles it carried a bar's factor of two — a pole is a whole radius out —
/// and answered `TooManyPointsAlongOneCrossing` like one.
#[test]
fn a_bar_leaves_the_walk_room_to_place_a_point_where_a_ball_leaves_it_none() {
    let (model, bar) = a_round_bar();
    let past_a_bars_guard = how_close_it_places_a_point(&model, bar);
    let what_a_bar_says = what_meshing_it_at(&model, bar, past_a_bars_guard);
    assert!(
        matches!(
            what_a_bar_says,
            Err(Error::Geometry(
                rcad_geom::Error::TooManyPointsAlongOneCrossing { .. }
            ))
        ),
        "a bar's face reaches twice what its cap circle does, so the walk has room left to \
         place a point at {past_a_bars_guard:e} and has to refuse for the other reason — it \
         said {what_a_bar_says:?}"
    );

    let (model, ball) = a_unit_ball();
    let past_a_balls_guard = how_close_it_places_a_point(&model, ball);
    let what_a_ball_says = what_meshing_it_at(&model, ball, past_a_balls_guard);
    assert!(
        matches!(
            what_a_ball_says,
            Err(Error::Geometry(
                rcad_geom::Error::CannotPlaceAPointThatCloseToACrossing { .. }
            ))
        ),
        "a ball's face reaches exactly what its meridian does, so the mesher's floor and the \
         walk's are one floor and the walk has nowhere to put a point at \
         {past_a_balls_guard:e} — it said {what_a_ball_says:?}"
    );
}

/// The cutting floor, taken off the public refusal and off nothing else.
fn how_fine_a_mesh_of_it_is_cut(model: &Model, solid: SolidKey) -> f64 {
    let refused = model.solid(solid).mesh(
        Deflection::of(FINER_THAN_ANYTHING_CAN_BE_CUT).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    let Err(Error::DeflectionBelowWhatAMeshCanReach { asked, closest }) = refused else {
        panic!("a deflection under every floor this solid has was answered with {refused:?}");
    };
    assert_eq!(asked, FINER_THAN_ANYTHING_CAN_BE_CUT);
    closest
}

/// The placing floor, hunted for as the deflection where the guard stops being
/// the thing that refuses.
///
/// The bracket is a million either side of where both floors sit, so neither end
/// is near the answer, and both ends are checked to be on the side they are
/// meant to be on before a single bit is halved.
fn how_close_it_places_a_point(model: &Model, solid: SolidKey) -> f64 {
    let (mut under, mut over) = (1e-18, 1e-6);
    assert!(
        the_guard_is_what_refuses(model, solid, under),
        "the bottom of the bracket, {under:e}, is not under the placing floor"
    );
    assert!(
        !the_guard_is_what_refuses(model, solid, over),
        "the top of the bracket, {over:e}, is not over the placing floor"
    );
    while under.to_bits() + 1 < over.to_bits() {
        let between = f64::from_bits((under.to_bits() + over.to_bits()) / 2);
        if the_guard_is_what_refuses(model, solid, between) {
            under = between;
        } else {
            over = between;
        }
    }
    let refused_by_the_walk = what_meshing_it_at(model, solid, over);
    assert!(
        matches!(refused_by_the_walk, Err(Error::Geometry(_))),
        "the first deflection the guard lets through, {over:e}, has to be refused by \
         the walk instead, or the boundary this hunt found is not the placing floor — \
         and it was answered with {refused_by_the_walk:?}"
    );
    over
}

fn the_guard_is_what_refuses(model: &Model, solid: SolidKey, asked: f64) -> bool {
    matches!(
        what_meshing_it_at(model, solid, asked),
        Err(Error::DeflectionBelowWhatAMeshCanReach { .. })
    )
}

fn what_meshing_it_at(model: &Model, solid: SolidKey, asked: f64) -> Result<Mesh, Error> {
    model.solid(solid).mesh(
        Deflection::of(asked).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
}

fn a_round_bar() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(1.0, 2.0, &Isometry::identity())
        .expect("a round bar of radius one and two tall");
    (model, bar)
}

fn a_unit_ball() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ball = model
        .add_ball(1.0, &Isometry::identity())
        .expect("a ball of radius one");
    (model, ball)
}
