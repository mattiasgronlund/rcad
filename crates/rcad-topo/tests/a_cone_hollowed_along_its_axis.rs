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
//! - **both circles of one pair drawn on a face**, and every rule that kept a
//!   piece either side of both circles was refused after the faces were cut,
//!   while `fuse/assemble.rs` named the corner a closed crossing starts at by
//!   its two surfaces alone and the two circles got one. Named by what picks
//!   each circle out as well (Q1148), a pointed cone less a steeper cone two up
//!   answers every rule, and so does a pointed cone less a whole ball pushing
//!   out through its side, since the check's mesh is asked fine enough to stop
//!   the ball's band and the cone's folding onto each other (Q1198). So does a
//!   frustum less that ball, since a side along one of the two circles also asks
//!   what picks its circle out (Q1197);
//! - **a wall through a circle**, where the wall, the cone and the partner meet
//!   with two of them curved, and every rule was refused there until Q1259
//!   placed those points. The circle then holds two nodes off its start, so
//!   the order round it is asked, and three intersections and one difference
//!   answer; every other rule is refused after the meet.
//!
//! Without a wall no fuse here asks the order round one of these circles. A
//! seam puts one node on a face's circle beside its start, and each face has
//! one seam, which is why two frustums seamed off where their circle starts
//! answer as well; `rcad-geom`'s `along_a_crossing.rs` orders points round
//! them since Q1141, and a wall through the circle is what reaches it.
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
            model
                .validate(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("an answer validates");
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

/// The pair above with the wide frustum seamed up `−x` and the steep one up
/// `+y`, so **neither seam is where the circle at `z = 4` starts**, which is
/// `(4, 0, 4)`: `walk` starts a circle round a cone's axis along the cone's
/// own first basis direction, `+x` here. Every rule answers the volumes above.
///
/// What orders the nodes is that each face's circle holds its own seam's node
/// and the start and nothing else, so a node is only ever put after the start
/// by name and the order round the circle is never asked. That was measured
/// with `which_comes_first_round_a_crossing` refusing every pair with a cone in
/// it, as it did before Q1141: this pair, the seams `(+y, −x)` and `(−y, +y)`,
/// and a frustum and a bar of radius 4 with the bar's seam at the start each
/// answered every rule both ways all the same.
#[test]
fn a_frustum_less_a_steeper_frustum_both_seamed_off_where_their_circle_starts() {
    let wide = a_frustum_holds(1.0, 2.0, 8.0);
    let steep = 511.0 * PI / 6.0;
    let shared = 367.0 * PI / 6.0;
    fuses_hold(
        |model| {
            let steeper = seamed(model, 2.0, 0.5, 4.0, [0.0, 1.0]);
            [
                seamed(model, 1.0, 2.0, 8.0, [-1.0, 0.0]),
                up(model, steeper, 2.0),
            ]
        },
        [
            &[wide + steep - shared],
            &[shared],
            &[49.0 * PI / 6.0, 296.0 * PI / 3.0],
            &[steep - shared],
        ],
    );
}

/// The wide frustum seamed up `−x`, `168π`, and a bar of radius 4 from `z = 0`
/// to `z = 10`, `160π`, whose seam is up `+x`. The cone crosses the pipe at
/// `z = ±4`, and only `z = 4` is on both sides. Below it the frustum is the
/// narrower and above it the bar, so the two share `π(4³ − 2³)/3 + 16π·4 =
/// 248π/3`. The frustum less the bar is the ring between them from `z = 4` up,
/// `256π/3`, and the bar less the frustum is two solids: its top above `z = 8`,
/// `32π`, and its bottom below `z = 4` less the frustum's, `32π + 40π/3`.
#[test]
fn a_frustum_and_a_bar_of_radius_four_cross_in_one_circle_on_both_sides() {
    let wide = a_frustum_holds(1.0, 2.0, 8.0);
    let bar = 160.0 * PI;
    let shared = 248.0 * PI / 3.0;
    fuses_hold(
        |model| {
            let bar = model
                .add_round_bar(4.0, 10.0, &Isometry::identity())
                .expect("a bar of radius four");
            [seamed(model, 1.0, 2.0, 8.0, [-1.0, 0.0]), bar]
        },
        [
            &[wide + bar - shared],
            &[shared],
            &[wide - shared],
            &[32.0 * PI, 136.0 * PI / 3.0],
        ],
    );
}

/// **A wall through the circle is placed, and most rules then stop after it.**
/// The wide frustum seamed up `−x` and cut down to `x ≥ 0` by a block, against
/// the steep frustum, the bar of radius 4 and a whole ball of radius 3 four up.
/// The wall `x = 0` crosses each pair's circle on the half frustum's side, at
/// `(0, ±4, 4)` for the first two and `(0, ±z₂, z₂)`, `z₂ = 2 + √2/2`, for the
/// ball, so the circle there has two nodes off its start, `(4, 0, 4)` and
/// `(z₂, 0, z₂)`.
///
/// Each is where the wall, the cone and the partner meet, two of them curved,
/// and every rule both ways was refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`
/// there. `rcad-geom`'s `three_surfaces.rs` places them since Q1259: a plane
/// holding the axis two surfaces are turned about cuts each circle they share
/// in two points. The order round the circle from its start is then asked, as
/// `along_a_crossing.rs` answers it since Q1141.
///
/// Each partner turns about the axis, so it holds the same either side of the
/// wall and the half frustum shares half of what the whole one does. With `W`
/// the whole frustum's volume, `P` the partner's and `S` what the two share,
/// the answers are `W/2 + P − S/2`, `S/2`, `(W − S)/2` and `P − S/2`: for the
/// steep frustum `1663π/12`, `367π/12`, `49π/12` with `148π/3`, and `655π/12`;
/// for the bar `608π/3`, `124π/3`, `128π/3` and `356π/3`; for the ball, with
/// `S = (100 − √2)π/3`, `(620 + √2)π/6`, `(100 − √2)π/6`, `(404 + √2)π/6` and
/// `(116 + √2)π/6`.
///
/// **What answers is the intersection with the steep frustum, both ways,
/// `367π/12`, with the bar, both ways, `124π/3`, and the half less the bar,
/// `128π/3`.** Every other rule is refused after the meet, and not traced: the
/// unions and the rest of the differences `ResultIsNotManifold` with one edge or
/// four, and the ball's intersections and the half less the ball
/// `CannotMeshAFace` with `CornersCoincide`.
#[test]
fn a_wall_through_the_circle_a_cone_crosses_a_frustum_a_bar_or_a_ball_in_is_placed_and_most_rules_stop_after()
 {
    let half = |model: &mut Model| {
        let wide = seamed(model, 1.0, 2.0, 8.0, [-1.0, 0.0]);
        the_x_positive_half_of(model, wide)
    };
    let partners: [fn(&mut Model) -> SolidKey; 3] = [
        |model| {
            let steeper = a_frustum(model, 2.0, 0.5, 4.0);
            up(model, steeper, 2.0)
        },
        |model| {
            model
                .add_round_bar(4.0, 10.0, &Isometry::identity())
                .expect("a bar of radius four")
        },
        |model| a_ball_four_up(model, 3.0),
    ];
    use Gives::{CornersCoincide, Holds, NotManifold};
    let wanted = [
        [
            NotManifold(1),
            NotManifold(4),
            Holds(367.0 * PI / 12.0),
            Holds(367.0 * PI / 12.0),
            NotManifold(1),
            NotManifold(4),
        ],
        [
            NotManifold(4),
            NotManifold(4),
            Holds(124.0 * PI / 3.0),
            Holds(124.0 * PI / 3.0),
            Holds(128.0 * PI / 3.0),
            NotManifold(4),
        ],
        [
            NotManifold(4),
            NotManifold(4),
            CornersCoincide,
            CornersCoincide,
            CornersCoincide,
            NotManifold(4),
        ],
    ];
    for (partner, wanted) in partners.into_iter().zip(wanted) {
        let gives = what_each_rule_gives(|model| [half(model), partner(model)]);
        assert!(
            gives
                .iter()
                .zip(&wanted)
                .all(|(answer, wanted)| wanted.is(answer)),
            "{gives:?}"
        );
    }
}

/// What one fuse is pinned to give: one solid of a closed-form volume, or a
/// refusal by name.
enum Gives {
    Holds(f64),
    NotManifold(usize),
    CornersCoincide,
}

impl Gives {
    fn is(&self, answer: &Result<Vec<(f64, f64)>, Error>) -> bool {
        match (self, answer) {
            (Gives::Holds(wanted), Ok(volumes)) => {
                matches!(volumes[..], [(volume, error)] if (volume - wanted).abs() <= error)
            }
            (Gives::NotManifold(wanted), Err(Error::ResultIsNotManifold { count })) => {
                count == wanted
            }
            (
                Gives::CornersCoincide,
                Err(Error::CannotMeshAFace {
                    why: rcad_mesh::CannotTriangulate::CornersCoincide { .. },
                    ..
                }),
            ) => true,
            _ => false,
        }
    }
}

fn seamed(model: &mut Model, slope: f64, near: f64, far: f64, seam: [f64; 2]) -> SolidKey {
    a_frustum_seamed_towards(
        model,
        slope,
        near,
        far,
        seam,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a frustum seamed up a whole direction")
}

/// What of `solid` lies in `x ≥ 0`, cut by a block from `z = 1` to `z = 9`.
fn the_x_positive_half_of(model: &mut Model, solid: SolidKey) -> SolidKey {
    let profile = ProfileBuilder::new(vec![[0.0, -10.0], [10.0, -10.0], [10.0, 10.0], [0.0, 10.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let block = model
        .add_extrusion(
            &profile,
            8.0,
            &Isometry::translation(Vector::new(0.0, 0.0, 1.0)).expect("a shift"),
        )
        .expect("a block");
    model
        .fuse(
            &[solid],
            &[block],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("half a frustum")
        .solids()[0]
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
/// cone's face, `z = 4/3` as well as `z = 4`. They share the `184π/3` the
/// frustum above shares with it, since below `z = 2` the steep cone holds
/// nothing. The first less the second is two solids: the cone below `z = 4`
/// less the steep cone's tip, `64π/3 − 32π/3`, and the top above `z = 6`,
/// `296π/3`. The steep cone less the first is `24π`.
///
/// **The union and the first less the second were refused
/// `ResultIsNotManifold { count: 4 }`** while the corner each circle starts
/// at was named by the two cones alone: both circles started at one corner,
/// so they were one closed edge with four faces along it. The corner is now
/// named by what picks its circle out of the two as well (Q1148).
#[test]
fn a_pointed_cone_less_a_steeper_one_whose_circles_both_fall_on_its_side() {
    let cone = 512.0 * PI / 3.0;
    let steep = 256.0 * PI / 3.0;
    let shared = 184.0 * PI / 3.0;
    fuses_hold(
        |model| {
            let steeper = a_pointed_cone(model, 2.0, 4.0);
            [a_pointed_cone(model, 1.0, 8.0), up(model, steeper, 2.0)]
        },
        [
            &[cone + steep - shared],
            &[shared],
            &[32.0 * PI / 3.0, 296.0 * PI / 3.0],
            &[steep - shared],
        ],
    );
}

/// A cone of slope 1 less a whole ball of radius 3 four up its axis, pointed
/// up to `z = 8` and then a frustum from `z = 2`. `(1 + s²)R² − s²e² = 18 − 16`
/// is above zero, so the ball pushes out through the side between
/// `z₁ = 2 − √2/2` and `z₂ = 2 + √2/2`, where `2z² − 8z + 7` is below zero and
/// the cone is the narrower. Both circles are on both faces of the pointed
/// cone; the frustum's face holds the upper one alone.
///
/// The ball's slices hold `π[9u − u³/3]` with `u` measured from its centre,
/// so the pointed cone shares the ball below `z₁`, its own slices from `z₁`
/// to `z₂`, and the ball above. The pointed cone less the ball is its tip
/// below `z₁` less the ball's bottom, and the rest above. The frustum less the
/// ball is one solid, cut through from its bottom to `z₂`, and the ball less
/// the frustum is one too, its bottom joined to the ring round the frustum's
/// side.
///
/// **The ball less the pointed cone is the ring round the cone's side.** At
/// height `z` the ball's slice reaches `9 − (z − 4)² = 8z − 7 − z²` squared out
/// from the axis and the cone's `z²`, and the first is the wider exactly where
/// `2z² − 8z + 7 = 2(z − z₁)(z − z₂)` is below zero. So the cone leaves the
/// ball nothing below `z₁`, where its bottom is inside the cone, and nothing
/// above `z₂`, and between them the slice is the annulus
/// `π(8z − 7 − 2z²) = −2π(z − z₁)(z − z₂)`, which adds up to
/// `π(z₂ − z₁)³/3`. With `z₂ − z₁ = √2` that is `2√2π/3 = 2.9619`.
///
/// **Every fuse of the pointed cone was refused after the faces were cut,
/// and every fuse of the frustum `ResultIsNotManifold { count: 3 }`**, while
/// the corner each circle starts at was named by the two surfaces alone and
/// both circles got one (Q1148).
///
/// **Then the ball less the pointed cone was refused by the check's mesh**,
/// `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }` (Q1198). The
/// ring is two faces between the same two circles, the ball's band outside and
/// the cone's band inside. Neither had a point inside it at deflections 7, 3.5
/// and 1.75 (8, 11 and 14 points in all), so each drew sides straight from one
/// circle to the other, and both drew the same ones. `validate.rs`'s
/// `the_coarsest_mesh_whose_sides_do_not_cross` stopped there, at half the
/// distance between the circles' starts, which are the two closest corners, 2
/// apart. A side from one circle to the other is at least 2 long, and the ball
/// charges a side `L²/(3r)`, so below `4/9` its band takes a point inside and
/// the two stop drawing the same sides: at 0.875 they still folded, and at
/// 0.4375 the mesh was taken.
///
/// **The frustum shares `(100 − √2)π/3` with the ball.** It starts at `z = 2`,
/// between `z₁` and `z₂`, where the ball's slice is the wider, so up to `z₂`
/// the two share the frustum's own slices, `π(z₂³ − 8)/3`, and above it the
/// ball's, `π∫(8z − 7 − z²)dz` from `z₂` to 7. By hand, with
/// `z₂² = 9/2 + 2√2` and `z₂³ = 11 + 25√2/4`, the first is `π(1 + 25√2/12)`;
/// `4z² − 7z − z³/3` is `98/3` at 7 and `1/3 + 29√2/12` at `z₂`, so the second
/// is `π(97/3 − 29√2/12)`, and together they are `π(100/3 − √2/3)`. The ball
/// less the frustum checks it from the other side: its bottom below `z = 2`,
/// `π∫₁²(8z − 7 − z²)dz = 8π/3`, and the ring round the frustum's side from 2
/// to `z₂`, `π∫(8z − 7 − 2z²)dz = √2π/3`, are `36π` less it, `(8 + √2)π/3`.
/// The union is `(512 + √2)π/3` and the frustum less the ball `(404 + √2)π/3`.
///
/// **Every fuse of the frustum was refused `ResultIsNotManifold { count: 1 }`
/// by selection** (Q1138). The ball's face is cut in three, the cap below
/// `z₁`, the band and the cap above `z₂`, and the circle at `z₁` is one edge
/// with one corner. But `select.rs`'s `the_piece_holds` asked a curved piece
/// only which side of each bounding circle's partner a point is on, and both
/// caps are inside the cone. So the lower cap was placed by a point of the
/// upper one, the step up from its corner, `(2.93, 0, 4.66)`, and found inside
/// the frustum, which it is wholly below. Every rule then kept the band or the
/// lower cap and not both, and the circle had one face along it. A side along
/// one of two circles now also asks what picks its circle out, unless its piece
/// has a side along the other circle too (Q1197).
#[test]
fn a_cone_less_a_whole_ball_pushing_out_through_its_side() {
    let [low, high] = [-1.0, 1.0].map(|side| 2.0 + side * 2.0_f64.sqrt() / 2.0);
    let slices = |z: f64| PI * (9.0 * (z - 4.0) - (z - 4.0).powi(3) / 3.0);
    let ball = 36.0 * PI;
    let pointed = 512.0 * PI / 3.0;
    let frustum = a_frustum_holds(1.0, 2.0, 8.0);
    let above = slices(7.0) - slices(high);
    let pointed_shares = slices(low) - slices(1.0) + a_frustum_holds(1.0, low, high) + above;
    let tip = a_frustum_holds(1.0, 0.0, low) - (slices(low) - slices(1.0));
    let ring = PI * (high - low).powi(3) / 3.0;

    let gives = what_each_rule_gives(|model| {
        let ball = a_ball_four_up(model, 3.0);
        [a_pointed_cone(model, 1.0, 8.0), ball]
    });
    let holds = |answer: &Result<Vec<(f64, f64)>, Error>, wanted: &[f64]| {
        matches!(answer, Ok(volumes)
            if volumes.len() == wanted.len()
                && volumes
                    .iter()
                    .zip(wanted)
                    .all(|(&(volume, error), &wanted)| (volume - wanted).abs() <= error))
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
    let union_holds = pointed + ball - pointed_shares;
    println!("the ball less the pointed cone, {ring} by its closed form: {less_cone:?}");
    assert!(
        holds(union, &[union_holds])
            && holds(swapped_union, &[union_holds])
            && holds(intersection, &[pointed_shares])
            && holds(swapped_intersection, &[pointed_shares])
            && holds(cone_less, &[tip, pointed - pointed_shares - tip])
            && holds(less_cone, &[ring]),
        "{gives:?}"
    );

    let frustum_shares = (100.0 - 2.0_f64.sqrt()) * PI / 3.0;
    fuses_hold(
        |model| {
            let ball = a_ball_four_up(model, 3.0);
            [a_frustum(model, 1.0, 2.0, 8.0), ball]
        },
        [
            &[frustum + ball - frustum_shares],
            &[frustum_shares],
            &[frustum - frustum_shares],
            &[ball - frustum_shares],
        ],
    );
}
