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
//! - **one circle on a face**, and the fuse answers since Q1129, which listed
//!   both pairs in `fuse/arrange.rs`'s `a_pair_that_crosses_in_a_conic` and
//!   taught the arrangement to read which side of each circle a point of a
//!   cone's or a ball's face is on — a frustum less a steeper cone, with one
//!   node on the circle or two, and a frustum less a ball cut flat below its
//!   lower circle;
//! - **both circles of one pair drawn on a face**, and every rule that keeps a
//!   piece either side of both circles is refused after the faces are cut:
//!   `fuse/assemble.rs` names a closed crossing by its two surfaces alone, so
//!   the two circles look like one curve there. A pointed cone less a steeper
//!   cone two up, and a pointed cone or a frustum less a whole ball pushing out
//!   through its side, are the shapes.
//!
//! Every cone here is built by `tests/cones`, face by face through
//! `SolidBuilder` with a power-of-two slope, and every ball by `Model::add_ball`.

use std::f64::consts::PI;

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::{a_frustum, a_frustum_seamed_towards, a_pointed_cone};

/// What a frustum of slope `slope` about `+z`, apex at the origin, holds
/// between `near` and `far` up the axis.
fn a_frustum_holds(slope: f64, near: f64, far: f64) -> f64 {
    PI * slope * slope * (far.powi(3) - near.powi(3)) / 3.0
}

/// Every rule, both ways round, with the volume of each solid it leaves, the
/// smallest first: `[union, intersection, first less second, second less
/// first]`, an empty list meaning nothing is left.
fn fuses_hold(
    build: impl Fn(&mut Model) -> [SolidKey; 2],
    [union, intersection, first_less_second, second_less_first]: [&[f64]; 4],
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
        assert_eq!(
            answer.len(),
            wanted.len(),
            "{what}: {} solids, not {answer:?}",
            wanted.len()
        );
        let mut measured: Vec<(f64, f64)> = answer
            .iter()
            .map(|&solid| {
                let measured = model
                    .solid(solid)
                    .expect("this model made the key")
                    .measure(
                        Deflection::of(0.001).expect("a fine mesh"),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    )
                    .unwrap_or_else(|why| panic!("{what} does not measure: {why}"));
                (measured.volume(), measured.volume_error())
            })
            .collect();
        measured.sort_by(|one, other| one.0.total_cmp(&other.0));
        for (&wanted, (volume, error)) in wanted.iter().zip(measured) {
            assert!(
                (volume - wanted).abs() <= error,
                "{what} holds {wanted} and this one says {volume} give or take {error}"
            );
        }
    }
}

/// What every rule, both ways round, comes back with, in the order
/// [`every_rule_both_ways`] gives them: `Ok` with the volume of each solid and
/// how far off it may be, the smallest first, or the refusal.
fn what_each_rule_gives(
    build: impl Fn(&mut Model) -> [SolidKey; 2],
) -> Vec<Result<Vec<(f64, f64)>, Error>> {
    every_rule_both_ways()
        .map(|(rule, swapped)| {
            let mut model =
                Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
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
            )?;
            let mut volumes: Vec<(f64, f64)> = answer
                .solids()
                .iter()
                .map(|&solid| {
                    let measured = model
                        .solid(solid)
                        .expect("this model made the key")
                        .measure(
                            Deflection::of(0.001).expect("a fine mesh"),
                            &Policy::certified(),
                            &mut ProvenanceLog::new(),
                        )
                        .expect("an answer measures");
                    (measured.volume(), measured.volume_error())
                })
                .collect();
            volumes.sort_by(|one, other| one.0.total_cmp(&other.0));
            Ok(volumes)
        })
        .collect()
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

fn a_ball_four_up(model: &mut Model, radius: f64) -> SolidKey {
    model
        .add_ball(
            radius,
            &Isometry::translation(Vector::new(0.0, 0.0, 4.0)).expect("a shift"),
        )
        .expect("a ball")
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
        [&[wide], &[narrow], &[wide - narrow], &[]],
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
        [&[wide + outside], &[inside], &[wide - inside], &[outside]],
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
            &[cone + half_a_ball],
            &[half_a_ball],
            &[cone - half_a_ball],
            &[half_a_ball],
        ],
    );
}

/// The wide frustum from `z = 2` to `z = 8`, `168π`, less a pointed cone of
/// slope 2 whose apex is two up, cut off at `z = 6`, `256π/3`. The two cones
/// cross where `z = 2|z − 2|`: at `z = 4`, on both sides, and at `z = 4/3`,
/// below the frustum and on the steep cone's lower nappe, so on neither face.
/// The steep cone is the narrower below `z = 4` and the wider above, so the
/// two share `π∫₂⁴ 4(z − 2)² dz + π∫₄⁶ z² dz = 32π/3 + 152π/3`. The frustum
/// less the steep cone is two solids, the ring below `z = 4`, `8π`, and the
/// top above `z = 6`, `296π/3`. Both used to be refused
/// `ACrossingThisCannotBeOrderedAlong`, because two cones were not a pair
/// `a_pair_that_crosses_in_a_conic` listed (Q1129).
#[test]
fn a_frustum_less_a_steeper_cone_whose_one_circle_falls_on_both_sides() {
    let wide = a_frustum_holds(1.0, 2.0, 8.0);
    let steep = 256.0 * PI / 3.0;
    let shared = 184.0 * PI / 3.0;
    fuses_hold(
        |model| {
            let steeper = a_pointed_cone(model, 2.0, 4.0);
            [a_frustum(model, 1.0, 2.0, 8.0), up(model, steeper, 2.0)]
        },
        [
            &[wide + steep - shared],
            &[shared],
            &[8.0 * PI, 296.0 * PI / 3.0],
            &[steep - shared],
        ],
    );
}

/// The same pair with the steep cone cut to a frustum from `z = 2.5`, and
/// each frustum's seam a quarter turn from the other's, up `+x` and up `+y`.
/// Each seam crosses the circle at `z = 4`, so the circle has two nodes on
/// each side where the one above had none: one is where the circle starts,
/// and the other is put after it without asking the order round the circle.
/// The steep frustum holds `π·4(4³ − ½³)/3 = 511π/6`, and shares
/// `π∫₂.₅⁴ 4(z − 2)² dz + 152π/3 = 63π/6 + 304π/6`.
#[test]
fn a_frustum_less_a_steeper_frustum_seamed_a_quarter_turn_round_from_it() {
    let wide = a_frustum_holds(1.0, 2.0, 8.0);
    let steep = 511.0 * PI / 6.0;
    let shared = 367.0 * PI / 6.0;
    fuses_hold(
        |model| {
            let seamed = |model: &mut Model, slope, near, seam| {
                a_frustum_seamed_towards(
                    model,
                    slope,
                    near,
                    if slope == 1.0 { 8.0 } else { 4.0 },
                    seam,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a frustum seamed up a whole direction")
            };
            let steeper = seamed(model, 2.0, 0.5, [0.0, 1.0]);
            [seamed(model, 1.0, 2.0, [1.0, 0.0]), up(model, steeper, 2.0)]
        },
        [
            &[wide + steep - shared],
            &[shared],
            &[49.0 * PI / 6.0, 296.0 * PI / 3.0],
            &[steep - shared],
        ],
    );
}

/// The wide frustum less a ball of radius 3 four up its axis, cut flat at
/// `z = 1.75`. The whole ball would cross the cone where
/// `2z² − 8z + 7 = 0`, at `z = 2 ± √2/2`; cut flat, the ball's face holds only
/// the upper circle, which is on the frustum's side too, and the lower one is
/// on neither face. The ball cut flat holds `36π` less a cap `0.75` deep,
/// `π·0.75²(9 − 0.75)/3`. Below the upper circle the cone is the narrower, and
/// above it the ball, so the two share `π(z₂³ − 2³)/3` and the ball's slices
/// from `z₂ = 2 + √2/2` to its top, `π[9u − u³/3]` from `u = z₂ − 4` to `3`.
#[test]
fn a_frustum_less_a_ball_cut_flat_below_its_lower_circle() {
    let wide = a_frustum_holds(1.0, 2.0, 8.0);
    let cut_ball = 36.0 * PI - PI * 0.75 * 0.75 * (9.0 - 0.75) / 3.0;
    let upper = 2.0 + 2.0_f64.sqrt() / 2.0;
    let slices = |u: f64| 9.0 * u - u.powi(3) / 3.0;
    let shared = PI * (upper.powi(3) - 8.0) / 3.0 + PI * (slices(3.0) - slices(upper - 4.0));
    fuses_hold(
        |model| {
            let ball = a_ball_four_up(model, 3.0);
            let profile = ProfileBuilder::new(vec![
                [-10.0, -10.0],
                [10.0, -10.0],
                [10.0, 10.0],
                [-10.0, 10.0],
            ])
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a square");
            let floor = model
                .add_extrusion(
                    &profile,
                    10.0,
                    &Isometry::translation(Vector::new(0.0, 0.0, 1.75)).expect("a shift"),
                )
                .expect("a block over the ball's upper part");
            let cut = model
                .fuse(
                    &[ball],
                    &[floor],
                    Rule::Intersection,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a ball cut flat")
                .solids()[0];
            [a_frustum(model, 1.0, 2.0, 8.0), cut]
        },
        [
            &[wide + cut_ball - shared],
            &[shared],
            &[wide - shared],
            &[cut_ball - shared],
        ],
    );
}

/// The pointed cone of slope 1 up to `z = 8`, `512π/3`, less the steep
/// pointed cone of the frustum above. Now both circles are on the first
/// cone's face, `z = 4/3` as well as `z = 4`, and a rule that keeps the
/// pieces either side of both is refused `ResultIsNotManifold { count: 4 }`:
/// the union and the first less the second. The other two keep no piece
/// either side of `z = 4/3` and answer, as the frustum above does: the shared
/// `184π/3`, and the steep cone less the first, `24π`.
///
/// **What refuses is the name the two circles share.** `fuse/assemble.rs`
/// names a closed crossing by its two surfaces alone, and only a plane
/// through a ring's axis is told apart further, so two closed edges between
/// one pair of cones look like one edge with four faces along it. Not traced
/// to the edge: the frustum above, whose face is off the lower circle, is
/// the same pair less that circle, and it answers every rule (Q1129).
#[test]
fn a_pointed_cone_less_a_steeper_one_whose_circles_both_fall_on_its_side() {
    let shared = 184.0 * PI / 3.0;
    let gives = what_each_rule_gives(|model| {
        let steeper = a_pointed_cone(model, 2.0, 4.0);
        [a_pointed_cone(model, 1.0, 8.0), up(model, steeper, 2.0)]
    });
    let not_a_manifold = |answer: &Result<Vec<(f64, f64)>, Error>| {
        matches!(answer, Err(Error::ResultIsNotManifold { count: 4 }))
    };
    let holds = |answer: &Result<Vec<(f64, f64)>, Error>, wanted: f64| {
        matches!(answer, Ok(volumes)
            if matches!(volumes[..], [(volume, error)] if (volume - wanted).abs() <= error))
    };
    let [
        union,
        swapped_union,
        intersection,
        swapped_intersection,
        cone_less,
        less_cone,
    ] = &gives[..]
    else {
        panic!("six fuses, not {gives:?}");
    };
    assert!(
        not_a_manifold(union) && not_a_manifold(swapped_union) && not_a_manifold(cone_less),
        "{gives:?}"
    );
    assert!(
        holds(intersection, shared)
            && holds(swapped_intersection, shared)
            && holds(less_cone, 256.0 * PI / 3.0 - shared),
        "{gives:?}"
    );
}

/// The pointed cone of slope 1 up to `z = 8`, less a whole ball of radius 3
/// four up its axis. `(1 + s²)R² − s²e² = 18 − 16` is above zero, so the ball
/// pushes out through the side between `z = 2 − √2/2` and `z = 2 + √2/2`, and
/// both circles are on both faces. Every rule is refused after the faces are
/// cut: the mesh of a face finds two of its corners are one point
/// (`CannotMeshAFace` with `CornersCoincide`), except the cone less the ball,
/// which cannot say whether one shell is inside another. With the frustum in
/// place of the pointed cone, so that only the ball's face holds the lower
/// circle, every rule is refused `ResultIsNotManifold { count: 3 }` instead.
/// The frustum less the same ball cut flat below its lower circle answers
/// every rule (the test above), so this is the shared name again (Q1129).
#[test]
fn a_cone_less_a_whole_ball_pushing_out_through_its_side_is_refused_after_its_faces_are_cut() {
    let pointed = what_each_rule_gives(|model| {
        let ball = a_ball_four_up(model, 3.0);
        [a_pointed_cone(model, 1.0, 8.0), ball]
    });
    for (index, answer) in pointed.iter().enumerate() {
        let refused_right = match index {
            4 => matches!(answer, Err(Error::Undecidable { .. })),
            _ => matches!(answer, Err(Error::CannotMeshAFace { .. })),
        };
        assert!(refused_right, "fuse {index} of {pointed:?}");
    }

    let frustum = what_each_rule_gives(|model| {
        let ball = a_ball_four_up(model, 3.0);
        [a_frustum(model, 1.0, 2.0, 8.0), ball]
    });
    assert!(
        frustum
            .iter()
            .all(|answer| matches!(answer, Err(Error::ResultIsNotManifold { count: 3 }))),
        "{frustum:?}"
    );
}
