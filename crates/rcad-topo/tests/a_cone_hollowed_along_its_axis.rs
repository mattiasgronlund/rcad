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
//!   the order round it is asked, and four intersections and two differences
//!   answer since Q1269 named the circle's two arcs apart, two more
//!   intersections and a difference with the ball since Q1270 put its pole
//!   among the points walked along the wall's arc, and the steep frustum's
//!   unions and the half less it since Q1337 put the steep cone's apex on the
//!   half's floor as well as its wall, the bar's unions and the bar less the
//!   half since Q1338 named the cone in the run of the arc between two pieces
//!   of the bar's wall, and the ball's unions and the ball less the half since
//!   Q1385 let a ring through one of the ball's poles go right round the
//!   other. Every rule answers.
//!
//! Without a wall no fuse here asks the order round one of these circles. A
//! seam puts one node on a face's circle beside its start, and each face has
//! one seam, which is why two frustums seamed off where their circle starts
//! answer as well; `rcad-geom`'s `along_a_crossing.rs` orders points round
//! them since Q1141, and a wall through the circle is what reaches it.
//!
//! A wall through a circle also leaves each arc of it between two corners of
//! three surfaces, and such a side does not say which of a pair's two circles
//! it runs along, so it asks nothing of what picks its circle out (Q1248). The
//! ball cut in half by a wall through both its circles answers every rule
//! against the frustum and the pointed cone all the same. A frustum from
//! `z = 1.5`, which reaches below the plane that picks the upper circle out,
//! answers every rule since Q1883, which charged the check's mesh of a cone's
//! face: before, the ball less it was refused `VolumeIsNotPositive` on a
//! coarse mesh taken on its sign, after selection placed every piece right.
//! The pointed cone cut in half against the whole ball is refused not knowing
//! which curve a face runs along. A thin bar through a fat one has its two
//! loops ordered and its cells told apart since Q1483, and walked through the
//! fat bar's axis since Q1493, and answers every rule since `rcad-geom`'s
//! `walk.rs` started each loop's walk on that loop (Q1262) and kept its corner
//! off the points it walked (Q1508).
//!
//! Every cone here is built by `tests/cones`, face by face through
//! `SolidBuilder` with a power-of-two slope, and every ball by `Model::add_ball`.

use std::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Scale, Vector};
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

/// **A wall through the circle is placed, and every rule then answers against
/// the steep frustum.** The wide frustum seamed up `−x` and cut down to `x ≥ 0`
/// by a block, `84π`, against the steep frustum of the tests above, from
/// `z = 2.5` to `z = 6` about the apex `(0, 0, 2)`, `511π/6`. The wall `x = 0`
/// crosses their circle at `z = 4` on the half frustum's side, at `(0, ±4, 4)`,
/// so the circle there has two nodes off its start, `(4, 0, 4)`.
///
/// Each is where the wall, the cone and the partner meet, two of them curved,
/// and every rule both ways was refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`
/// there. `rcad-geom`'s `three_surfaces.rs` places them since Q1259: a plane
/// holding the axis two surfaces are turned about cuts each circle they share
/// in two points. The order round the circle from its start is then asked, as
/// `along_a_crossing.rs` answers it since Q1141.
///
/// The steep frustum turns about the axis, so it holds the same either side of
/// the wall and the half shares half of the `367π/6` the whole frustum does.
/// The union is `84π + 511π/6 − 367π/12 = 1663π/12` and the intersection
/// `367π/12`. The half less the steep frustum is two solids, as the whole one
/// less it is: the half of the ring below `z = 4`,
/// `½(π∫₂⁴ z² dz − π∫₂.₅⁴ 4(z − 2)² dz) = ½(56π/3 − 63π/6) = 49π/12`, and the
/// half of the top above `z = 6`, `½ · π(8³ − 6³)/3 = 148π/3`. The steep
/// frustum less the half is `511π/6 − 367π/12 = 655π/12`.
///
/// **The unions and the half less the steep frustum were refused
/// `ResultIsNotManifold { count: 1 }`** (Q1337). The steep cone's apex is on
/// the edge where the wall meets the half's floor `z = 2`. The wall is cut
/// along the two rulings `x = 0` cuts the steep cone in, drawn in full down to
/// the apex, so it walked that edge as two sides either side of the apex. The
/// floor touches the steep cone at the apex and nowhere else, which drew
/// nothing on it, so it walked the edge as one side. The floor is now handed
/// the cone as a surface that touches it, and `fuse/arrange.rs`'s
/// `place_the_touches` puts the apex on the one curve of the floor through it,
/// the edge along the wall.
#[test]
fn a_half_frustum_against_a_steeper_frustum_whose_apex_is_on_its_floor_answers_every_rule() {
    let gives = what_each_rule_gives(|model| [a_half_frustum(model), a_steep_frustum(model)]);
    let wanted = [
        Gives::Holds(1663.0 * PI / 12.0),
        Gives::Holds(1663.0 * PI / 12.0),
        Gives::Holds(367.0 * PI / 12.0),
        Gives::Holds(367.0 * PI / 12.0),
        Gives::HoldsTwo(49.0 * PI / 12.0, 148.0 * PI / 3.0),
        Gives::Holds(655.0 * PI / 12.0),
    ];
    assert!(
        gives
            .iter()
            .zip(&wanted)
            .all(|(answer, wanted)| wanted.is(answer)),
        "{gives:?}"
    );
}

/// **A wall through the circle is placed, and every rule answers.** The half
/// frustum of the test above against the bar of radius 4 and a whole ball of
/// radius 3 four up, and against the ball again with the pair turned over. The wall `x = 0` crosses each pair's circle
/// on the half frustum's side, at `(0, ±4, 4)` for the bar and `(0, ±z₂, z₂)`,
/// `z₂ = 2 + √2/2`, for the ball, so the circle there has two nodes off its
/// start, `(4, 0, 4)` and `(z₂, 0, z₂)`. They are placed as for the steep
/// frustum above.
///
/// Each partner turns about the axis, so it holds the same either side of the
/// wall and the half frustum shares half of what the whole one does. With `W`
/// the whole frustum's volume, `P` the partner's and `S` what the two share,
/// the answers are `W/2 + P − S/2`, `S/2`, `(W − S)/2` and `P − S/2`: for the
/// bar `608π/3`, `124π/3`, `128π/3` and `356π/3`; for the ball, with
/// `S = (100 − √2)π/3`, `(620 + √2)π/6`, `(100 − √2)π/6`, `(404 + √2)π/6` and
/// `(116 + √2)π/6`.
///
/// The partner's skin holds the whole circle, and the arc of it at `x < 0`,
/// between the same two corners as the arc along the cone, was one edge with
/// it: every union and the partner less the half were refused
/// `ResultIsNotManifold { count: 4 }`, with the partner's skin three times
/// along it and the cone once, and so were the steep frustum's above.
/// `fuse/assemble.rs`'s `which_arc_of_a_circle_this_is` names the two arcs
/// apart since Q1269, and each of those rules then stopped one step on:
///
/// - against the bar, the unions and the bar less the half were refused
///   `ACurvedFaceGoesRightRoundWithNoSeam`, by the mesh the fuse checks its
///   answer with (Q1338). They keep the bar's wall below the circle and the
///   piece of it at `x < 0` above, two pieces with the arc at `x < 0` between
///   them, and the joiner leaves them apart. That arc's run named the pipe
///   twice: a pipe either side, and the pipe again in `and`, since a pipe sorts
///   ahead of a cone. The walk put no point between the arc's ends, the mesher
///   read the half turn between them the wrong way round, and the piece above
///   went right round the pipe with one wire. An arc between two pieces of one
///   surface now names the other surface in `and`, here the cone
///   (`fuse/assemble.rs`'s `the_surface_that_cut_each_edge_its_faces_do_not_name`),
///   and they answer `608π/3`, `608π/3` and `356π/3`;
/// - against the ball, the unions and the ball less the half were refused
///   `AWireWillNotLieFlatOnItsFace`, by the mesh the fuse checks its answer
///   with (Q1385). They keep the ball less the piece of it inside the half,
///   one face since the joiner joined a ball's pieces (§229.2), and its one
///   ring of two arcs runs over the ball's north pole along the wall `x = 0`.
///   The face holds the south pole, so the walk round that ring comes back a
///   whole turn on, and a wire of more than one edge was let do that only if
///   it never stood at a pole. A wire through one pole goes right round the
///   other whenever its face holds that one, and `mesh.rs`'s
///   `WalkingAWire::may_come_back_a_whole_turn_on` no longer asks, so they
///   answer `(620 + √2)π/6`, `(620 + √2)π/6` and `(116 + √2)π/6`. Turned over,
///   the ring runs over the ball's south pole and goes round the north one
///   the other way, which is why the turned pair is here too. Left in pieces
///   they were refused one step sooner,
///   `TheseTwoSurfacesAreOneSurfaceAndNameNoCrossing`, which `walk` gives for
///   two balls of one centre and one radius, the ball on either side of an
///   edge between two of its own pieces.
///
/// The ball's intersections and the half less the ball were refused
/// `CannotMeshAFace` with `CornersCoincide` by the mesh the fuse checks its
/// answer with (Q1270). The wall bounds the ball's skin inside the half with
/// one arc over the ball's north pole, and the walk along that open arc
/// stepped past the pole from `+y` to `−y`, half a turn of longitude the wrong
/// way. The skin was read as a cap right round the pole, cut up to it along the
/// meridian the arc runs down, and the patch between the two went through the
/// pole twice. `mesh.rs`'s `put_in_the_poles_a_crossing_passes` puts the pole
/// in along an open crossing too, where it is between the edge's ends.
#[test]
fn a_half_frustum_whose_wall_crosses_the_circle_it_meets_a_bar_or_a_ball_in_answers_every_rule() {
    let pairs: [fn(&mut Model) -> [SolidKey; 2]; 3] = [
        |model| {
            let bar = model
                .add_round_bar(4.0, 10.0, &Isometry::identity())
                .expect("a bar of radius four");
            [a_half_frustum(model), bar]
        },
        |model| [a_half_frustum(model), a_ball_four_up(model, 3.0)],
        |model| {
            let pair = [a_half_frustum(model), a_ball_four_up(model, 3.0)];
            pair.map(|solid| turned_over(model, solid))
        },
    ];
    use Gives::Holds;
    let root = 2.0_f64.sqrt();
    let against_the_ball = [
        Holds((620.0 + root) * PI / 6.0),
        Holds((620.0 + root) * PI / 6.0),
        Holds((100.0 - root) * PI / 6.0),
        Holds((100.0 - root) * PI / 6.0),
        Holds((404.0 + root) * PI / 6.0),
        Holds((116.0 + root) * PI / 6.0),
    ];
    let wanted = [
        [
            Holds(608.0 * PI / 3.0),
            Holds(608.0 * PI / 3.0),
            Holds(124.0 * PI / 3.0),
            Holds(124.0 * PI / 3.0),
            Holds(128.0 * PI / 3.0),
            Holds(356.0 * PI / 3.0),
        ],
        against_the_ball,
        against_the_ball,
    ];
    for (pair, wanted) in pairs.into_iter().zip(wanted) {
        let gives = what_each_rule_gives(pair);
        assert!(
            gives
                .iter()
                .zip(&wanted)
                .all(|(answer, wanted)| wanted.is(answer)),
            "{gives:?}"
        );
    }
}

/// A frustum of slope 3/4 from `z = 1` to `z = 9`, seamed up `−x`, and the ball
/// of radius 3 five up its axis, which it holds touching round one circle:
/// `(1 + s²)R² − s²e² = 25·9/16 − 9·25/16` is zero, so the circle is at
/// `h = e/(1 + s²) = 3.2`, `2.4` across. Along it `∇S × ∇T` is nothing, so
/// nothing tells its two arcs between two corners apart, and
/// `fuse/assemble.rs`'s `which_arc_of_a_circle_this_is` names a cone's arcs on
/// the word that no fuse gets that far.
///
/// **Every rule both ways is refused before an edge is named.** The whole
/// frustum, whose seam puts one corner on the circle beside its start, stops
/// at selection, `Undecidable` placing a point at the start; its half `x ≥ 0`,
/// whose wall puts two, stops at the ordering, which refuses to run along a
/// touch.
#[test]
fn a_ball_a_frustum_touches_round_one_circle_is_refused_before_its_arcs_are_named() {
    let ball = |model: &mut Model| {
        model
            .add_ball(
                3.0,
                &Isometry::translation(Vector::new(0.0, 0.0, 5.0)).expect("a shift"),
            )
            .expect("a ball")
    };
    let whole = what_each_rule_gives(|model| {
        let frustum = seamed(model, 0.75, 1.0, 9.0, [-1.0, 0.0]);
        [frustum, ball(model)]
    });
    assert!(
        whole
            .iter()
            .all(|answer| matches!(answer, Err(Error::Undecidable { .. }))),
        "{whole:?}"
    );
    let half = what_each_rule_gives(|model| {
        let frustum = seamed(model, 0.75, 1.0, 9.0, [-1.0, 0.0]);
        [the_x_positive_half_of(model, frustum), ball(model)]
    });
    assert!(
        half.iter().all(|answer| matches!(
            answer,
            Err(Error::Geometry(
                rcad_geom::Error::CannotFollowACrossingWhereItsSurfacesAreTangent { .. }
            ))
        )),
        "{half:?}"
    );
}

/// What one fuse is pinned to give: one solid of a closed-form volume, or two
/// solids of two, the smaller first.
#[derive(Clone, Copy)]
enum Gives {
    Holds(f64),
    HoldsTwo(f64, f64),
}

impl Gives {
    fn is(&self, answer: &Result<Vec<(f64, f64)>, Error>) -> bool {
        let near = |(volume, error): (f64, f64), wanted: f64| (volume - wanted).abs() <= error;
        match (self, answer) {
            (Gives::Holds(wanted), Ok(volumes)) => {
                matches!(volumes[..], [one] if near(one, *wanted))
            }
            (Gives::HoldsTwo(smaller, larger), Ok(volumes)) => {
                matches!(volumes[..], [one, other] if near(one, *smaller) && near(other, *larger))
            }
            _ => false,
        }
    }
}

/// `solid` turned half a turn about `x`, which takes `z` to `−z` and keeps the
/// wall `x = 0` where it is.
fn turned_over(model: &mut Model, solid: SolidKey) -> SolidKey {
    model
        .place_solid(
            solid,
            &Isometry::quarter_turns(&Axis::x(), 2).expect("x is an axis"),
        )
        .expect("this model made the key");
    solid
}

/// The wide frustum seamed up `−x`, cut down to `x ≥ 0`.
fn a_half_frustum(model: &mut Model) -> SolidKey {
    let wide = seamed(model, 1.0, 2.0, 8.0, [-1.0, 0.0]);
    the_x_positive_half_of(model, wide)
}

/// The frustum of slope 2 from `z = 2.5` to `z = 6` about the apex `(0, 0, 2)`.
fn a_steep_frustum(model: &mut Model) -> SolidKey {
    let steeper = a_frustum(model, 2.0, 0.5, 4.0);
    up(model, steeper, 2.0)
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

/// **A wall through both circles leaves each of the ball's caps a side between
/// two corners, and every rule still answers** (Q1248). The ball of the test
/// above cut down to `x ≥ 0` by a block from `z = 0` to `z = 10`, `18π`,
/// against the frustum from `z = 2` and against the pointed cone, each of
/// which turns about the axis.
///
/// The wall `x = 0` crosses each circle the ball's face holds at two corners
/// where the wall, the ball and the cone meet, so every arc of a circle on the
/// half ball runs between two corners of three surfaces, and
/// `fuse/select.rs`'s `what_else_a_curved_side_asks` asks such a side nothing
/// about what picks its circle out. Against the pointed cone that leaves the
/// cap below `z₁` and the cap above `z₂`, both inside the cone, told apart by
/// nothing the partner says, and the guess was that one of them would be
/// placed by a point of the other. Neither is: every rule answers.
///
/// With `C` the cone's volume and `S` what it shares with the whole ball, the
/// half ball shares `S/2`, so the answers are `C + 18π − S/2`, `S/2`,
/// `C − S/2` and `18π − S/2`. For the pointed cone the last is half the ring
/// round its side, `√2π/3`.
#[test]
fn a_half_ball_whose_wall_crosses_both_circles_it_meets_a_cone_in_answers_every_rule() {
    let [low, high] = [-1.0, 1.0].map(|side| 2.0 + side * 2.0_f64.sqrt() / 2.0);
    let slices = |z: f64| PI * (9.0 * (z - 4.0) - (z - 4.0).powi(3) / 3.0);
    let half_a_ball = 18.0 * PI;
    let cones: [fn(&mut Model) -> SolidKey; 2] = [
        |model| a_frustum(model, 1.0, 2.0, 8.0),
        |model| a_pointed_cone(model, 1.0, 8.0),
    ];
    let what_each_cone_holds_and_shares = [
        (
            a_frustum_holds(1.0, 2.0, 8.0),
            (100.0 - 2.0_f64.sqrt()) * PI / 3.0,
        ),
        (
            512.0 * PI / 3.0,
            slices(low) - slices(1.0) + a_frustum_holds(1.0, low, high) + slices(7.0)
                - slices(high),
        ),
    ];
    for (cone, (holds, shares)) in cones.into_iter().zip(what_each_cone_holds_and_shares) {
        let gives = what_each_rule_gives(|model| {
            let ball = a_ball_four_up(model, 3.0);
            let half = the_x_positive_half_between(model, ball, 0.0, 10.0);
            [cone(model), half]
        });
        use Gives::Holds;
        let wanted = [
            Holds(holds + half_a_ball - shares / 2.0),
            Holds(holds + half_a_ball - shares / 2.0),
            Holds(shares / 2.0),
            Holds(shares / 2.0),
            Holds(holds - shares / 2.0),
            Holds(half_a_ball - shares / 2.0),
        ];
        println!("{gives:?}");
        assert!(
            gives
                .iter()
                .zip(&wanted)
                .all(|(answer, wanted)| wanted.is(answer)),
            "{gives:?}"
        );
    }
}

/// **The pointed cone cut down to `x ≥ 0`, against the whole ball, is refused
/// under every rule both ways**, `TwoCurvesTouchWithoutCrossing` on one face,
/// the half cone's wall (Q1248). The block runs from `z = −1` to `z = 10`, so
/// it takes the whole of the cone's `x ≥ 0` half, `256π/3`, and its wall
/// crosses both circles on the cone's side as it does on the half ball's above.
///
/// Until Q2461 that face was refused
/// `CannotTellWhichCurveOfACrossingAFaceRunsAlong`. One of its edges runs along
/// `x = 0` from the apex `(0, 0, 0)` to `(0, 8, 8)`, and the plane `x = 0` cuts
/// two rulings `y = ±z` from the cone, both of which hold the apex, so the edge's
/// start point could not say which it runs along. `fuse/arrange.rs`'s
/// `crossing_for` now asks the end corner too, and it picks `y = z`. What then
/// touches without crossing is not traced.
///
/// The answers it is to reach, with `S` what the whole pointed cone shares with
/// the ball: `256π/3 + 36π − S/2`, `S/2`, `256π/3 − S/2` and `36π − S/2`.
#[test]
fn a_half_pointed_cone_against_a_ball_pushing_out_through_its_side_is_refused_under_every_rule() {
    let gives = what_each_rule_gives(|model| {
        let cone = a_pointed_cone(model, 1.0, 8.0);
        let half = the_x_positive_half_between(model, cone, -1.0, 10.0);
        [half, a_ball_four_up(model, 3.0)]
    });
    println!("{gives:?}");
    assert!(
        gives
            .iter()
            .all(|answer| matches!(answer, Err(Error::TwoCurvesTouchWithoutCrossing { .. }))),
        "{gives:?}"
    );
}

/// **A frustum from `z = 1.5` reaches below the plane that picks the upper
/// circle out, and its band is still placed** (Q1248). The ball of radius 3
/// four up crosses the cone at `z₁ = 2 − √2/2` and `z₂ = 2 + √2/2`, and
/// `intersect` picks each circle out by a half-space whose plane is `z = 2`,
/// between them. The frustum's face holds only the upper circle, so its band
/// inside the ball has a side along that circle alone and asks the picker, and
/// the picker rules out the band's points below `z = 2`. Its points above are
/// still candidates, and every rule placed it right.
///
/// The frustum holds `π(8³ − 1.5³)/3`, and below `z₂` it is the narrower, so
/// the two share its slices from `1.5` to `z₂` and the ball's above, `S`. The
/// union, the intersection and the frustum less the ball answer `F + 36π − S`,
/// `S` and `F − S`.
///
/// **The ball less the frustum, `36π − S = 5.016`, answers `5.0112`, give or
/// take `0.0547`** (Q1883). It is a cup whose wall thins to nothing at `z₂`
/// and is `√2.75 − 1.5 = 0.158` across at its floor. Selection placed every
/// one of the seven pieces right, as prints in `choose` and `place_the_piece`
/// showed (commit `10bfada5`): the ball's cap above `z₂` dropped, its band and
/// its cap below kept, and the frustum's floor and its band below `z₂` kept
/// turned round. Until Q1883 the cup was refused
/// `VolumeIsNotPositive { volume: -2.64 }` all the same (Q1481).
/// `fuse/assemble.rs`'s `nest_the_shells` weighs each shell by `validate.rs`'s
/// `how_much_space_a_curved_solid_encloses`, whose coarsest mesh of the cup,
/// nine points `1.91` off, encloses `−2.64`. A solid with a face of a cone was
/// charged nothing there, so that sum was taken on its sign, and
/// `nest_the_shells` refused a shell of negative volume no other shell holds.
/// Charged as `SolidView::measure` charges it, the same mesh can leave out
/// `309`, so finer meshes are asked for, and one `0.082` off answers `4.76`
/// against a charge of `4.60`.
#[test]
fn a_frustum_from_below_the_plane_between_the_two_circles_answers_every_rule() {
    let high = 2.0 + 2.0_f64.sqrt() / 2.0;
    let slices = |z: f64| PI * (9.0 * (z - 4.0) - (z - 4.0).powi(3) / 3.0);
    let ball = 36.0 * PI;
    let frustum = a_frustum_holds(1.0, 1.5, 8.0);
    let shares = a_frustum_holds(1.0, 1.5, high) + slices(7.0) - slices(high);
    let gives = what_each_rule_gives(|model| {
        let ball = a_ball_four_up(model, 3.0);
        [a_frustum(model, 1.0, 1.5, 8.0), ball]
    });
    use Gives::Holds;
    let answered = [
        Holds(frustum + ball - shares),
        Holds(frustum + ball - shares),
        Holds(shares),
        Holds(shares),
        Holds(frustum - shares),
        Holds(ball - shares),
    ];
    println!("{gives:?}");
    assert!(
        gives.len() == 6
            && gives
                .iter()
                .zip(&answered)
                .all(|(answer, wanted)| wanted.is(answer)),
        "{gives:?}"
    );
}

/// **A thin bar through a fat one answers every rule both ways**, the shape of
/// a cross-drilled shaft, through the fat bar's axis and half off it, with the
/// fat bar's seam up `+x` or up `+y`. A bar of radius 4 from `z = 0` to
/// `z = 10`, `160π`, and a bar of radius 1 along `x` from `x = −10` to `x = 10`
/// at `z = 5`, `20π`. The two pipes cross in two closed loops, one each side of
/// `x = 0`, and `intersect` picks each loop out by a half-space whose plane
/// runs through the fat bar's axis square to the thin one, `x = 0`, ten from
/// either end of the thin bar, so each of the thin bar's caps is on one side of
/// it. With `S` the volume the two share,
/// [`what_a_thin_bar_through_a_fat_one_shares`], the union is `180π − S`, the
/// intersection `S`, the fat bar less the thin `160π − S`, and the thin less
/// the fat its two ends, `(20π − S)/2` each.
///
/// Every rule was refused `ACrossingThisCannotBeOrderedAlong` before selection
/// until Q1483 put points along each loop in order, round the thin bar's axis,
/// and `fuse/arrange.rs`'s `whether_past_the_plane_between_two_loops` told the
/// fat bar's wall apart into its cells. Then the mesh the fuse checks its
/// answer with refused every rule. Through the axis, until Q1493, the walk did
/// not get round at all: `march.rs`'s `one_step` only halved a trial that
/// settled back onto the seed `(4, 0, 6)`, and fifty halvings of `4.1e153`
/// never came near the loop. Half off it, until Q1494, `mesh.rs` cut no ring
/// with islands in it across, and the fat bar's wall is such a ring.
///
/// Both halves were then refused for two faults in `rcad-geom`'s `walk.rs`.
/// Each loop is an edge that goes right round where its faces cross, with a
/// corner, and nothing picks the seed it starts from: the loop at `−x` was
/// walked from the seed on the loop at `+x` (Q1262), and a start an ulp off its
/// corner, or on it, was kept among the points walked, which the triangulator
/// refused as two corners that coincide (Q1508). Through the axis that gave
/// `AWireWillNotLieFlatOnItsFace`, `CannotMeshAFace` with two corners
/// coinciding, and `ACurvedFaceGoesRightRoundWithNoSeam`; half off it
/// `CurvedFaceWithAHoleGoesTooFarRound` and `CannotMeshAFace` with its
/// boundary crossing itself or two corners coinciding.
#[test]
fn a_thin_bar_through_a_fat_one_answers_every_rule_through_the_axis_and_half_off_it() {
    for seam in [0, 1] {
        for off in [0.0, 0.5] {
            let shares = what_a_thin_bar_through_a_fat_one_shares(off, 32);
            let an_end_of_the_thin_bar = (20.0 * PI - shares) / 2.0;
            println!("{off} off the axis, seam {seam}, sharing {shares}");
            fuses_hold(
                |model| a_thin_bar_through_a_fat_one(model, off, seam),
                [
                    &[180.0 * PI - shares],
                    &[shares],
                    &[160.0 * PI - shares],
                    &[an_end_of_the_thin_bar, an_end_of_the_thin_bar],
                ],
            );
        }
    }
}

/// The trapezium rule [`what_a_thin_bar_through_a_fat_one_shares`] works the
/// shared volume out by has closed in on it by 32 points, and what it gives is
/// less than all of the thin bar.
#[test]
fn what_a_thin_bar_through_a_fat_one_shares_closes_in_by_thirty_two_points() {
    for off in [0.0, 0.5] {
        let coarse = what_a_thin_bar_through_a_fat_one_shares(off, 32);
        let fine = what_a_thin_bar_through_a_fat_one_shares(off, 256);
        println!("{off}: {coarse} by 32 points, {fine} by 256");
        assert!(
            (coarse - fine).abs() <= 4e-15 * fine,
            "{off}: {coarse} and {fine}"
        );
        assert!(
            fine < 20.0 * PI,
            "{off}: the two share {fine} of the thin bar's 20π"
        );
    }
}

/// A bar of radius 4 about `z` from `z = 0` to `z = 10`, its seam `seam`
/// quarter turns round from `+x`, and [`a_thin_bar_along_x`] through it `off`
/// from its axis.
fn a_thin_bar_through_a_fat_one(model: &mut Model, off: f64, seam: i32) -> [SolidKey; 2] {
    let fat = model
        .add_round_bar(
            4.0,
            10.0,
            &Isometry::quarter_turns(&Axis::z(), seam).expect("z is an axis"),
        )
        .expect("a fat bar");
    [fat, a_thin_bar_along_x(model, off)]
}

/// What a bar of radius 4 about `z` shares with a bar of radius 1 along `x`
/// that it swallows, `off` along `y` from its axis: at `y` the fat bar is
/// `2√(16 − y²)` across along `x` and the thin one `2√(1 − (y − off)²)` along
/// `z`, so the two share `∫ 4 √(16 − (s + off)²) √(1 − s²) ds` over
/// `−1 ≤ s ≤ 1`. Through the axis that is `8 ∫₀¹ √((1 − y²)(16 − y²)) dy`.
///
/// Put `s = sin t` and the integral is `2 ∫ cos² t √(16 − (sin t + off)²) dt`
/// over a whole turn, since the half turn from `π/2` to `3π/2` runs over the
/// same `s` again. That integrand is smooth and periodic, so the trapezium
/// rule over a whole turn of `points` points closes in on it geometrically: 32
/// points and 256 agree to `4e-15` of it for both `off` here
/// (`24.9348268993723` and `24.73281320654972`), far inside the `0.001` mesh's
/// own error on the volume.
fn what_a_thin_bar_through_a_fat_one_shares(off: f64, points: u32) -> f64 {
    let step = 2.0 * PI / f64::from(points);
    let sum: f64 = (0..points)
        .map(|at| {
            let t = step * f64::from(at);
            t.cos().powi(2) * (16.0 - (t.sin() + off).powi(2)).sqrt()
        })
        .sum();
    2.0 * step * sum
}

/// A bar of radius 1 along `x` from `x = −10` to `x = 10`, at `z = 5` and
/// `off` along `y`.
fn a_thin_bar_along_x(model: &mut Model, off: f64) -> SolidKey {
    let placement = Isometry::translation(Vector::new(-10.0, off, 5.0))
        .expect("a shift")
        .compose(&Isometry::quarter_turns(&Axis::y(), 1).expect("y is an axis"));
    model
        .add_round_bar(1.0, 20.0, &placement)
        .expect("a thin bar")
}

/// What of `solid` lies in `x ≥ 0`, cut by a block from `z = low` to
/// `z = high`. [`the_x_positive_half_of`]'s block starts at `z = 1`, where the
/// ball four up would only touch it.
fn the_x_positive_half_between(
    model: &mut Model,
    solid: SolidKey,
    low: f64,
    high: f64,
) -> SolidKey {
    let profile = ProfileBuilder::new(vec![[0.0, -10.0], [10.0, -10.0], [10.0, 10.0], [0.0, 10.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle");
    let block = model
        .add_extrusion(
            &profile,
            high - low,
            &Isometry::translation(Vector::new(0.0, 0.0, low)).expect("a shift"),
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
        .expect("half a solid")
        .solids()[0]
}
