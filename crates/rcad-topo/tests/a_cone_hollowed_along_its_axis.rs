//! A cone less a second cone or a ball on its axis, fused through the public
//! API under every rule, both ways round, against closed-form volumes.
//!
//! `intersect` names a cone against a second cone or a ball on its axis since
//! Q1122: circles square to the axis, or the apex alone for two cones of one
//! apex. Those pairs used to stop every fuse at the first face pair. What a
//! fuse does with the circles depends on where they fall:
//!
//! - **off every face**, the circles cut nothing, and the fuse answers — a
//!   frustum hollowed by a narrower one of the same apex, a frustum cupped by a
//!   cone of the same slope whose apex is above its own, and a pointed cone
//!   dimpled by a ball the cone's side never meets;
//! - **on a face**, the arrangement has to order points round a circle two
//!   cones, or a cone and a ball, cross in, and `fuse/arrange.rs`'s
//!   `a_pair_that_crosses_in_a_conic` does not list either pair, so every rule
//!   is refused `ACrossingThisCannotBeOrderedAlong`. That is the next wall, and
//!   it is outside `rcad-geom`.
//!
//! Every cone here is built by `tests/cones`, face by face through
//! `SolidBuilder` with a power-of-two slope, and every ball by `Model::add_ball`.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_pointed_cone};

/// What a frustum of slope `slope` about `+z`, apex at the origin, holds
/// between `near` and `far` up the axis.
fn a_frustum_holds(slope: f64, near: f64, far: f64) -> f64 {
    PI * slope * slope * (far.powi(3) - near.powi(3)) / 3.0
}

/// Every rule, both ways round, with what each holds: `[union, intersection,
/// first less second, second less first]`, a zero meaning nothing is left.
fn fuses_hold(
    build: impl Fn(&mut Model) -> [SolidKey; 2],
    [union, intersection, first_less_second, second_less_first]: [f64; 4],
) {
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let [first, second] = build(&mut model);
        let [one, other] = if swapped {
            [second, first]
        } else {
            [first, second]
        };
        let what = format!("{rule:?}, swapped {swapped}");
        let answer = model
            .fuse(
                &[one],
                &[other],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} refused: {why}"))
            .solids()
            .to_vec();
        let wanted = match (rule, swapped) {
            (Rule::Union, _) => union,
            (Rule::Intersection, _) => intersection,
            (Rule::Difference, false) => first_less_second,
            (Rule::Difference, true) => second_less_first,
            (other, _) => panic!("not a rule this probes: {other:?}"),
        };
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what} left a model that does not validate: {why}"));
        if wanted == 0.0 {
            assert!(answer.is_empty(), "{what}: nothing is left, not {answer:?}");
            continue;
        }
        let [solid] = answer[..] else {
            panic!("{what}: one solid, not {answer:?}");
        };
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
            (measured.volume() - wanted).abs() <= measured.volume_error(),
            "{what} holds {wanted} and this one says {} give or take {}",
            measured.volume(),
            measured.volume_error()
        );
    }
}

/// Every rule, both ways round, refused under `why`.
fn fuses_refuse(build: impl Fn(&mut Model) -> [SolidKey; 2], why: impl Fn(&Error) -> bool) {
    for (rule, swapped) in every_rule_both_ways() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let [first, second] = build(&mut model);
        let [one, other] = if swapped {
            [second, first]
        } else {
            [first, second]
        };
        let answer = model.fuse(
            &[one],
            &[other],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert!(
            answer.as_ref().is_err_and(&why),
            "{rule:?}, swapped {swapped}: {:?}",
            answer.map(|fused| fused.solids().to_vec())
        );
    }
}

fn every_rule_both_ways() -> impl Iterator<Item = (Rule, bool)> {
    [Rule::Union, Rule::Intersection, Rule::Difference]
        .into_iter()
        .flat_map(|rule| [false, true].map(move |swapped| (rule, swapped)))
}

fn up(model: &mut Model, solid: SolidKey, by: f64) -> SolidKey {
    model
        .place_solid(
            solid,
            &Isometry::translation(Vector::new(0.0, 0.0, by)).expect("a whole-number shift"),
        )
        .expect("this model made the key");
    solid
}

/// A frustum of slope 1 from `z = 2` to `z = 8`, `168π`, less a frustum of
/// slope one half over the same heights, `42π`, with the same apex. The two
/// cones meet at that apex and nowhere else, which is off both, so the narrow
/// one sits inside the wide one with their caps on one pair of planes: a
/// hollow frustum, `126π`, open at both ends.
#[test]
fn a_frustum_hollowed_by_a_narrower_one_of_the_same_apex() {
    let [wide, narrow] = [
        a_frustum_holds(1.0, 2.0, 8.0),
        a_frustum_holds(0.5, 2.0, 8.0),
    ];
    fuses_hold(
        |model| {
            [
                a_frustum(model, 1.0, 2.0, 8.0),
                a_frustum(model, 0.5, 2.0, 8.0),
            ]
        },
        [wide, narrow, wide - narrow, 0.0],
    );
}

/// The same wide frustum, less a cone of the same slope whose apex is two up
/// the axis, cut from `z = 3` to `z = 10`. One slope and two apexes cross in
/// the one circle halfway between them, at `z = 1` with radius 1, below both
/// solids. The second pokes out through the first's top cap: it holds
/// `π(6³ − 1³)/3 = 215π/3` below `z = 8` and `π(8³ − 6³)/3 = 296π/3` above.
#[test]
fn a_frustum_cupped_by_a_cone_of_its_slope_whose_apex_is_above_its_own() {
    let wide = a_frustum_holds(1.0, 2.0, 8.0);
    let [inside, outside] = [215.0 * PI / 3.0, 296.0 * PI / 3.0];
    fuses_hold(
        |model| {
            let cup = a_frustum(model, 1.0, 1.0, 8.0);
            [a_frustum(model, 1.0, 2.0, 8.0), up(model, cup, 2.0)]
        },
        [wide + outside, inside, wide - inside, outside],
    );
}

/// A pointed cone of slope 1 up to `z = 8`, `512π/3`, and a ball of radius 2
/// centred on the middle of its cap. On the axis, eight up, the ball misses the
/// cone's side — `(1 + s²)R² − s²e² = 8 − 64` is below zero — so it only cuts
/// the cap, and the cone is dimpled by half a ball, `16π/3`.
#[test]
fn a_pointed_cone_dimpled_by_a_ball_its_side_never_meets() {
    let cone = 512.0 * PI / 3.0;
    let half_a_ball = 16.0 * PI / 3.0;
    fuses_hold(
        |model| {
            let ball = model
                .add_ball(
                    2.0,
                    &Isometry::translation(Vector::new(0.0, 0.0, 8.0)).expect("a shift"),
                )
                .expect("a ball of radius two");
            [a_pointed_cone(model, 1.0, 8.0), ball]
        },
        [
            cone + half_a_ball,
            half_a_ball,
            cone - half_a_ball,
            half_a_ball,
        ],
    );
}

/// The pointed cone of slope 1 up to `z = 8`, less a pointed cone of slope 2
/// whose apex is two up, cut off at `z = 6`. They cross in two circles, at
/// `z = 4/3` and `z = 4`, both on the first cone's side, and ordering points
/// round either is what the arrangement refuses: two cones are not a pair
/// `a_pair_that_crosses_in_a_conic` lists.
#[test]
fn a_cone_less_a_steeper_one_whose_circles_fall_on_its_side_is_refused_where_they_are_ordered() {
    fuses_refuse(
        |model| {
            let steeper = a_pointed_cone(model, 2.0, 4.0);
            [a_pointed_cone(model, 1.0, 8.0), up(model, steeper, 2.0)]
        },
        |why| matches!(why, Error::ACrossingThisCannotBeOrderedAlong),
    );
}

/// The pointed cone of slope 1 up to `z = 8`, less a ball of radius 3 four up
/// its axis. `(1 + s²)R² − s²e² = 18 − 16` is above zero, so the ball pushes
/// out through the side between `z = (4 − √2)/2` and `z = (4 + √2)/2`, and
/// ordering points round those two circles is refused the same way.
#[test]
fn a_cone_less_a_ball_pushing_out_through_its_side_is_refused_where_the_circles_are_ordered() {
    fuses_refuse(
        |model| {
            let ball = model
                .add_ball(
                    3.0,
                    &Isometry::translation(Vector::new(0.0, 0.0, 4.0)).expect("a shift"),
                )
                .expect("a ball of radius three");
            [a_pointed_cone(model, 1.0, 8.0), ball]
        },
        |why| matches!(why, Error::ACrossingThisCannotBeOrderedAlong),
    );
}
