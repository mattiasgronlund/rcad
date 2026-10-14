//! Two bars of radius 1/4 whose axes meet at a slant (Q1646).
//!
//! The first bar runs along `x` from `through − (2, 0, 0)`, 4 long. The second
//! is turned onto `x` and then by `degrees` about `z`, and is either 32 long and
//! centred on `through = (11, 0, 0)`, so it runs right through the first, or 2
//! long and starts at `through = (11, −3.25, 0.5)`, so its near cap passes
//! through where the axes meet. Each test runs all six fuses — three rules,
//! each bar first in turn. Every one answers: through either point at 50°,
//! 60°, 70°, 100° and 120°, through `(11, −3.25, 0.5)` at 85° and 95° too, and
//! through `(11, 0, 0)` at 80° and at the angles from 105° to 135° tested here.
//!
//! **What they should come to**, with `r = 1/4` and `a` the angle: the first
//! bar is `A = π/4`. What two pipes of one width share where their axes cross at
//! `a` is `16r³/(3 sin a)`, Steinmetz's `16r³/3` at a right angle, which is
//! `1/(12 sin a)` here. Through `(11, 0, 0)` the second bar is `B = 2π` and
//! runs well past that solid both ways, so the shared part is `I = 1/(12 sin
//! a)`. Through `(11, −3.25, 0.5)` the second bar is `B = π/8` and starts at
//! the meeting point. Turning every point half a turn about that point keeps
//! both axes where they are, so it keeps the shared solid too, and it swaps the
//! two sides of the cap: the cap cuts the solid in two equal halves, and `I =
//! 1/(24 sin a)`. In both, the union is `A + B − I`, the first less the second
//! `A − I` and the second less the first `B − I`.
//!
//! Two more pairs are built another way, as Q1896 and Q1897 build them: two
//! bars 2 long, each `π/8`, with the axes meeting in the middle of both, so each
//! runs past the shared solid both ways and `I = 1/(12 sin a)` again.

use std::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

/// The second bar runs right through the first at 50°, 60°, 70°, 100° or 120°,
/// and every fuse answers what the module doc says, with `I = 1/(12 sin a)`: the
/// union in 8 faces, the shared part in 4, and each bar less the other as two
/// solids of 3 faces, which touch at the two points the pipes are tangent at.
///
/// Before Q1646 these were refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`,
/// where each bar's far cap cuts its own pipe in a circle and the arrangement
/// could not prove that circle misses the other pipe; and `a_half_of_an_ellipse`
/// did not know a half picked by flat surfaces that do not face along the
/// pipes' stored directions. Both are fixed in `fuse/arrange.rs`.
///
/// Then until Q1660 every fuse was refused telling the cells of a pipe apart,
/// the way the square Steinmetz cross was
/// (`two_bars_of_one_radius_whose_axes_meet_cross_in_four_half_ellipses_and_every_fuse_answers`).
/// The cap circle at `x = 9` is to be matched with one of the four walks the
/// half ellipses cut the pipe into, and which one is read off how often each
/// walk goes round the axis, counted where it crosses a half-plane through the
/// axis. Of the six half-planes tried, two ran along the pipe's own axis, and
/// each of the other four ran through a node: the two points the halves meet
/// at sit on `±z`, and the middles of the four halves on `±y`. The half-plane
/// is now chosen half way across the widest gap between the nodes. Past it
/// every fuse was refused `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed` in
/// `fuse/assemble.rs`, where the way along an edge out of a point the two
/// pipes are tangent at was read as nought.
///
/// The long pipe is also cut where it crosses the first bar's cap planes,
/// `x = 9` and `x = 13`, far off either cap. Those cuts are joined back, since
/// each runs right round the pipe as one closed curve
/// (`joined_only_across_whole_rings` in `fuse/assemble.rs`), so the faces
/// counted below are those of the bars and nothing more. The joining keeps the
/// middle of each half ellipse as a corner, since a half folded into one edge
/// from touch point to touch point cannot be named.
///
/// **Until Q1965 every fuse at 120° was refused** (Q1793): the unions
/// `CannotNameWhichPieceOfACrossingThisIs`, the rest `CannotMeshAFace { why:
/// CornersCoincide }`, `AWireWillNotLieFlatOnItsFace` or
/// `ACurvedFaceGoesRightRoundWithNoSeam`. There the second bar's stored
/// direction is `(−0.5, 0.866…, −2.2e−16)`, a hair off the flat `z = 0`, so the
/// two points the pipes touch at sit at `y = ±6.4e−17`, and `∇S × ∇T` there is
/// `(1.6e−17, 0, 0)` (measured in Q1660), not nought. `fuse/assemble.rs` asked
/// whether the pipes are tangent at a corner by that cross being exactly nought,
/// so it read a rounding at each touch point, named the arcs from it by that
/// rounding, and folded each half ellipse into one edge from touch point to
/// touch point. A corner the arrangement names as one of the two places the
/// pipes touch now carries that it is (`docs/decisions.md` §448.3).
#[test]
fn a_long_bar_through_another_at_a_slant_answers_every_fuse() {
    let wrong = what_goes_wrong_with_a_long_bar_through_another(&[50.0, 60.0, 70.0, 100.0, 120.0]);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The second bar starts where the axes meet, at 50°, 60°, 70°, 100° or 120°,
/// and every fuse answers what the module doc says, with `I = 1/(24 sin a)`: the
/// union in 9 faces, the shared part in 4, the first bar less the second in 10
/// and the second less the first in 3.
///
/// The first bar's wall is cut in six by the two ellipses the pipes cross in
/// and by the near cap's plane, which runs right round it through the two
/// points the pipes touch at. The piece inside the second bar goes and the
/// other five stay faces of their own, so the union is those five, the first
/// bar's two caps, the second bar's wall and its far cap; the first bar less
/// the second is the same five and two caps, the near cap and the two pieces
/// of the second pipe inside the first; the shared part is the near cap, one
/// piece of the first pipe and two of the second.
///
/// **Until Q1908 every fuse was refused `Undecidable` `orient3d` on the near
/// cap**, asking which way the cap turns at a point where the pipes touch,
/// from its crossing with the first pipe to its own circle round the second.
/// The two pipes are tangent there, so their gradients are parallel, and the
/// determinant of the cap's normal and the two is nought, which their boxes
/// cannot prove. The node is named as one of the two places the pipes touch,
/// so the turn there is now read as nought and the two curves are ordered by
/// how they bend (`turns_anticlockwise` in `fuse/arrange.rs`): the circle at
/// `1/r`, and the crossing with the first pipe, an ellipse with half axes `r`
/// and `r/|cos a|`, at `cos²a/r`.
///
/// **Until Q1898 every fuse was refused `order along a crossing` on the
/// near cap**, ordering the two points `c ± r·ẑ` where the pipes touch round
/// the cap's closed crossing with the first pipe. That crossing starts at
/// `(11, −3.25, 0.25)`, inside the `3.1e−13` box of the cap's corner there,
/// and the box reaches as far behind the start as ahead of it along the way
/// the crossing runs, so whether the corner comes first or last is not read
/// off it (`round_a_closed_crossing_and_what_it_measures`). Those two are the
/// only nodes on that crossing, and their boxes are clear of each other: two
/// places on a loop cut it into the same two arcs whichever is called first,
/// so `nodes_along` in `fuse/arrange.rs` no longer asks.
///
/// Each bar's faces are split in the order the bars are handed in, and the
/// first bar's are all split before the near cap is reached, its wall in six.
/// **Until Q1907, at 120° with the first bar first, the first bar's wall was
/// refused** on the order round the same crossing, where the node at the start
/// is a box `4.4e-16` wide and proved not behind it
/// (`a_wall_told_a_cap_holds_where_the_axes_meet_is_split_where_the_cap_meets_the_halves`
/// in `fuse/arrange.rs`). At 120° the order was reached since Q1645 named the
/// crossing there; before it, `intersect` refused all six
/// `CannotNameWhichPieceOfACrossingThisIs`.
///
/// That corner is the bottom of `add_round_bar`'s seam, at `(r, 0, 0)`
/// before the move, and the turn onto `x` puts it at `c − r·ẑ`. **Until
/// Q1815 the near cap and wall were refused `side of cylinder`** in
/// `place_the_corners`, asking through `passes_through` which side of the
/// first pipe the corner's box is on (Q1663); the box straddles that pipe.
/// The corner is now taken as the touch point, and the cap's meet with the
/// two pipes placed as the walls place it.
///
/// **Until Q1820 every fuse with the first bar first was refused
/// `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`** on the first bar's wall:
/// `place_the_meets` asked `meet_of_three_surfaces` where that pipe, the
/// second bar's near cap and the second pipe meet, which it cannot solve for
/// one plane and two pipes (Q1663). The cap's cut across the first pipe and
/// one half ellipse of the two pipes do share points, so the pair is not
/// waved through as two cuts that share nothing.
///
/// The second bar's near cap is square to its axis through the meeting point
/// `c`, so it holds both points `c ± r·ẑ` where the two pipes touch, `(11,
/// −3.25, 0.25)` and `(11, −3.25, 0.75)`: there the cap and both pipes meet,
/// two of the three tangent to each other. At 50° to 100° the second pipe's
/// stored direction lies in `z = 0`, so both points are exact in `f64`; at 120°
/// it is a hair off `z = 0`, as the test above says. **That holds before the
/// cap is turned, and not in the numbers the arrangement is handed**: the turn
/// rounds the cap, and the cap as stored misses both points by a rounding
/// (`the_near_cap_as_the_model_hands_it_out_misses_both_points_the_pipes_touch_at`).
///
/// **The answer rests on the cap holding `c`.** The stored cap proves itself
/// off both points, so nothing exact asked of the stored surfaces names them
/// as the meet; the cap holding `c` is the second bar's own incidence, cap
/// through the start of its axis, kept by its move as a seam corner's is on
/// the seam plane (§294.9). Since Q1814 `fuse/bake.rs` reads it on the
/// numbers before the move and carries it, moved, on the second bar's near
/// cap and wall (`caps_through_the_start_of_their_axis`), and since Q1820
/// gathering the cuts in `fuse/cuts.rs` tells it to every face those two cut,
/// the first bar's wall among them. That wall names the cap's meet with the
/// two pipes as the node where the four halves meet. Since Q1815 the corner
/// at `c − r·ẑ` is that node too: it is on the cap's circle by construction,
/// and that circle meets the first pipe only at `c ± r·ẑ` because the cap
/// holds `c`.
///
/// **Until Q1965 every fuse at 120° got past the near cap and was refused**
/// (Q1929): the union and both differences `AWireWillNotLieFlatOnItsFace`, the
/// shared part `ACurvedFaceGoesRightRoundWithNoSeam`. `fuse/assemble.rs` asked
/// whether the pipes are tangent at the two touch points by an exact nought of
/// `∇S × ∇T` at their rounded points, as
/// `a_long_bar_through_another_at_a_slant_answers_every_fuse` says, and now
/// reads it off the name the arrangement gives them.
#[test]
fn a_bar_starting_where_it_crosses_another_at_a_slant_answers_every_fuse() {
    let wrong =
        what_goes_wrong_with_a_bar_starting_where_the_axes_meet(&[50.0, 60.0, 70.0, 100.0, 120.0]);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// As above at 85° and 95°, where the near cap's plane crosses the first
/// bar's wall in an ellipse that runs past both of the first bar's caps: it
/// reaches `r·tan a`, `2.86`, either way along `x` from the meeting point, and
/// the caps are `2` off at `x = 9` and `x = 13`. So the loop has six nodes on
/// the wall, where it crosses the two cap circles four times and the two
/// points the pipes touch at, and they have to be put in order round it.
///
/// **Until Q1968 every fuse was refused `order along a crossing`** on the first
/// bar's wall, between the node at `c − r·ẑ` and the node where the loop
/// crosses the circle at `x = 9`. The loop is walked from `c − r·ẑ`, `(11,
/// −3.25, 0.25)` exactly, and the node there was worked out as `c −
/// r·n̂/‖n̂‖`, for `n̂ = (0, 0, sin a)` square to both axes, so it was a box
/// `2e-16` tall in `z` and of no width in `x` and `y`. Every point of that box
/// reads exactly nought along the way the loop runs at its start, so the node
/// was not counted first round the loop, and nothing else could say whether
/// it comes first or last. Where `n̂` lies along a coordinate axis it is now
/// taken as that axis, and the node is the start itself, by name
/// (`where_two_pipes_touch` in `fuse/arrange.rs`).
#[test]
fn a_bar_starting_where_it_crosses_another_at_85_or_95_degrees_answers_every_fuse() {
    let wrong = what_goes_wrong_with_a_bar_starting_where_the_axes_meet(&[85.0, 95.0]);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// One line for each of the six fuses of the bar starting where the axes meet
/// at each of `angles` that does not answer as
/// `a_bar_starting_where_it_crosses_another_at_a_slant_answers_every_fuse`
/// says.
fn what_goes_wrong_with_a_bar_starting_where_the_axes_meet(angles: &[f64]) -> Vec<String> {
    let (first, second) = (PI / 4.0, PI / 8.0);
    let mut wrong = Vec::new();
    for &degrees in angles {
        let shared = 1.0 / (24.0 * degrees.to_radians().sin());
        for (rule, first_first) in six_fuses() {
            let (volume, faces): (f64, &[usize]) = match (rule, first_first) {
                (Rule::Union, _) => (first + second - shared, &[9]),
                (Rule::Intersection, _) => (shared, &[4]),
                (Rule::Difference, true) => (first - shared, &[10]),
                (Rule::Difference, false) => (second - shared, &[3]),
                (_, _) => unreachable!("six_fuses names three rules and no other"),
            };
            let mut model =
                Model::new(rcad_geom::Scale::millimetres(100.0).expect("100 mm is a working size"));
            let said = fuse_in(&mut model, degrees, [11.0, -3.25, 0.5], rule, first_first)
                .map_err(|why| format!("refused: {why:?}"))
                .and_then(|solids| how_far_off(&model, &solids, volume, faces));
            if let Err(why) = said {
                wrong.push(format!(
                    "{degrees}° {rule:?} (first first: {first_first}), solids of {faces:?} faces and volume {volume}: {why}"
                ));
            }
        }
    }
    wrong
}

/// Two bars 2 long, each cut off 1 either side of where the axes meet, one
/// along `(3, 4, 0)` and the other along `(4, 0, 3)`, so `cos a = 12/25`,
/// answer every fuse at each of the nine places Q1896 has the axes meet, with
/// `I = 1/(12 sin a)` as the module doc says: the union in 8 faces, the shared
/// part in 4, and each bar less the other as two solids of 3 faces. The pipe of
/// the one taken away cuts the other bar in two, and the two pieces touch at
/// the two points the pipes are tangent at, each thinning to nothing there.
///
/// **Until Q1966 the mesh drew one sliver twice there**, and the fuse was
/// refused `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }` in
/// `nest_the_shells`, which meshes each shell to measure it. At the touch point
/// a piece of each pipe meets the other along the same two arcs, and each cut
/// the corner off by the same side between the two mesh points next to it, so
/// the same triangle was drawn twice, facing both ways (`docs/decisions.md`
/// §446.8). The bar along `(3, 4, 0)` less the other, the same pinch, meshed.
///
/// **Until Q1965 that held only where the axes meet at `(0, 0, 0)`.** The fuse
/// asked whether the pipes are tangent at a touch point by an exact nought at
/// its rounded point, and elsewhere it could read a rounding there, say no, and
/// pick an arc from the touch point the wrong way. At `(0, 1, 0)` a face then
/// came to the mesh with an arc from a touch point walked three quarters of the
/// way round its ellipse, and was refused `ASideCutsOffOneCornerOfACurvedPatch`
/// (Q1953's trace).
#[test]
fn two_bars_on_3_4_0_and_4_0_3_answer_every_fuse_wherever_the_axes_meet() {
    let wrong = what_goes_wrong_with_two_bars_crossing(
        [[3.0, 4.0, 0.0], [4.0, 0.0, 3.0]],
        &[
            [0.0, 0.0, 0.0],
            [0.5, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.25, 0.5, 0.75],
            [11.0, -3.25, 0.5],
        ],
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Two bars 2 long as above, one along `(3, 4, 0)` and the other along `(4, −3,
/// 0)`, square to each other in `z = 0`, answer every fuse at each of the four
/// places Q1897 has the axes meet, with `I = 1/12`.
#[test]
fn two_bars_on_3_4_0_and_4_minus_3_0_answer_every_fuse_wherever_the_axes_meet() {
    let wrong = what_goes_wrong_with_two_bars_crossing(
        [[3.0, 4.0, 0.0], [4.0, -3.0, 0.0]],
        &[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [11.0, 0.0, 0.0],
            [11.0, -3.25, 0.5],
        ],
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// One line for each of the six fuses of the two bars [`two_bars_crossing_at`]
/// builds along `ways` about each of `middles` that does not answer as
/// `two_bars_on_3_4_0_and_4_0_3_answer_every_fuse_wherever_the_axes_meet` says.
fn what_goes_wrong_with_two_bars_crossing(
    ways: [[f64; 3]; 2],
    middles: &[[f64; 3]],
) -> Vec<String> {
    let [one, other] = ways.map(|way| rcad_geom::Vector::new(way[0], way[1], way[2]));
    let sin = one.cross(other).length() / (one.length() * other.length());
    let (bar, shared) = (PI / 8.0, 1.0 / (12.0 * sin));
    let mut wrong = Vec::new();
    for &middle in middles {
        for (rule, first_first) in six_fuses() {
            let (volume, faces): (f64, &[usize]) = match rule {
                Rule::Union => (2.0 * bar - shared, &[8]),
                Rule::Intersection => (shared, &[4]),
                Rule::Difference => (bar - shared, &[3, 3]),
                _ => unreachable!("six_fuses names three rules and no other"),
            };
            let what = format!(
                "{middle:?} {rule:?} (first first: {first_first}), solids of {faces:?} faces and volume {volume}"
            );
            let mut model =
                Model::new(rcad_geom::Scale::millimetres(100.0).expect("100 mm is a working size"));
            let [first, second] = two_bars_crossing_at(&mut model, ways, middle);
            let (arguments, tools) = if first_first {
                ([first], [second])
            } else {
                ([second], [first])
            };
            let said = model
                .fuse(
                    &arguments,
                    &tools,
                    rule,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .map(|answer| answer.solids().to_vec())
                .map_err(|why| format!("refused: {why:?}"))
                .and_then(|solids| how_far_off(&model, &solids, volume, faces));
            if let Err(why) = said {
                wrong.push(format!("{what}: {why}"));
            }
        }
    }
    wrong
}

/// The second bar's near cap, as the model hands it out, is proved off both
/// points where the pipes touch, `(11, −3.25, 0.25)` and `(11, −3.25, 0.75)`,
/// at 50°, 60°, 70° and 100°, where both are exact in `f64`. Before the move
/// the cap is `z = 0` and holds the start of the bar's axis exactly; the turn
/// rounds the cap's normal and offset, and the rounded cap misses `c`. So the
/// meet of the cap with the two pipes cannot be shown to be those two points
/// from the stored cap (Q1663).
#[test]
fn the_near_cap_as_the_model_hands_it_out_misses_both_points_the_pipes_touch_at() {
    for degrees in [50.0, 60.0, 70.0, 100.0] {
        let mut model =
            Model::new(rcad_geom::Scale::millimetres(100.0).expect("100 mm is a working size"));
        let [_, second] = two_bars(&mut model, degrees, [11.0, -3.25, 0.5]);
        let near_cap = the_near_cap(&model, second);
        for z in [0.25, 0.75] {
            let side = near_cap.side_of_point(PredicatePoint::from([11.0, -3.25, z]));
            assert!(
                matches!(side, Decision::Pos | Decision::Neg),
                "{degrees}°: the near cap {near_cap:?} at (11, -3.25, {z}): {side:?}"
            );
        }
    }
}

/// The cap of the bar `second` that starts at `(11, −3.25, 0.5)`, as the
/// model hands it out.
fn the_near_cap(model: &Model, second: SolidKey) -> rcad_geom::Plane {
    model
        .solid(second)
        .expect("the second bar")
        .faces()
        .filter_map(|face| face.surface().as_plane().copied())
        .find(|cap| {
            cap.signed_distance_to(rcad_geom::Point::new(11.0, -3.25, 0.5))
                .abs()
                < 1e-12
        })
        .expect("a bar starting at the meeting point has a cap within a rounding of it")
}

/// The second bar runs right through the first at 122°, 125°, 128°, 129° or
/// 135°, where the axes meet at `(11, 0, 0)`, which `f64` holds, and every fuse
/// answers what `a_long_bar_through_another_at_a_slant_answers_every_fuse`
/// says. The two tests after this one do the same at 80°, 112° and 116°, and at
/// 105° to 119°.
///
/// **Until Q1967 all 84 fuses of the three tests were refused
/// `CannotNameWhichPieceOfACrossingThisIs` by `intersect`**, splitting the
/// faces, before any piece was chosen. Each of the four halves of the ellipses
/// two pipes of one width cross in is picked by two flat surfaces through where
/// the axes meet, and that point was worked out in `f64` and then asked exactly
/// of both axes. It failed three ways, one for each test, each fixed in
/// `four_half_ellipses_where_two_pipes_of_one_width_meet` in
/// `crates/rcad-geom/src/intersect.rs`. Here the `f64` formula landed beside
/// `(11, 0, 0)`, and the point is now worked out exactly
/// (`where_two_axes_meet_exactly`).
///
/// **Then until Q1965 every one got past the split and none answered**
/// (Q1991): the unions, both ways round, were refused
/// `CannotNameWhichPieceOfACrossingThisIs` after the pieces were chosen, as at
/// 120°, and the rest `CannotMeshAFace`, `AWireWillNotLieFlatOnItsFace`,
/// `ACurvedFaceGoesRightRoundWithNoSeam` or
/// `Invalid(FacesDoNotAgreeWhichWayTheyFace)`. What made them answer is what
/// made 120° answer: a corner the arrangement names as one of the two places
/// the pipes touch carries that it is, and the fuse no longer reads a rounding
/// there (`docs/decisions.md` §448.3).
#[test]
fn a_long_bar_through_another_at_122_to_135_degrees_answers_every_fuse() {
    let wrong =
        what_goes_wrong_with_a_long_bar_through_another(&[122.0, 125.0, 128.0, 129.0, 135.0]);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// As above at 80°, 112° and 116°, where the second bar's stored direction
/// lies exactly in `z = 0` and `f64` holds no point where the axes meet. Until
/// Q1967 each flat surface that picks a half was asked to hold that point; it
/// now holds the whole of the other axis (`a_surface_through_the_other_axis`).
#[test]
fn a_long_bar_through_another_at_80_112_or_116_degrees_answers_every_fuse() {
    let wrong = what_goes_wrong_with_a_long_bar_through_another(&[80.0, 112.0, 116.0]);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// As above at 105°, 110°, 111°, 113°, 114° and 119°, where the second bar's
/// stored direction is a hair off `z = 0` and `f64` holds no point where the
/// axes meet either. Until Q1967 each flat surface that picks a half was asked
/// to hold that point; it now passes through a point `f64` holds that is level
/// with the meeting point along the surface's facing
/// (`a_surface_through_a_point_beside_the_middle`).
#[test]
fn a_long_bar_through_another_a_hair_off_flat_at_105_to_119_degrees_answers_every_fuse() {
    let wrong = what_goes_wrong_with_a_long_bar_through_another(&[
        105.0, 110.0, 111.0, 113.0, 114.0, 119.0,
    ]);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// One line for each of the six fuses of the long bar through the first at
/// each of `angles` that does not answer as
/// `a_long_bar_through_another_at_a_slant_answers_every_fuse` says.
fn what_goes_wrong_with_a_long_bar_through_another(angles: &[f64]) -> Vec<String> {
    let (first, second) = (PI / 4.0, 2.0 * PI);
    let mut wrong = Vec::new();
    for &degrees in angles {
        let shared = 1.0 / (12.0 * degrees.to_radians().sin());
        for (rule, first_first) in six_fuses() {
            let (volume, faces): (f64, &[usize]) = match (rule, first_first) {
                (Rule::Union, _) => (first + second - shared, &[8]),
                (Rule::Intersection, _) => (shared, &[4]),
                (Rule::Difference, true) => (first - shared, &[3, 3]),
                (Rule::Difference, false) => (second - shared, &[3, 3]),
                (_, _) => unreachable!("six_fuses names three rules and no other"),
            };
            let mut model =
                Model::new(rcad_geom::Scale::millimetres(100.0).expect("100 mm is a working size"));
            let said = fuse_in(&mut model, degrees, [11.0, 0.0, 0.0], rule, first_first)
                .map_err(|why| format!("refused: {why:?}"))
                .and_then(|solids| how_far_off(&model, &solids, volume, faces));
            if let Err(why) = said {
                wrong.push(format!(
                    "{degrees}° {rule:?} (first first: {first_first}), solids of {faces:?} faces and volume {volume}: {why}"
                ));
            }
        }
    }
    wrong
}

fn six_fuses() -> [(Rule, bool); 6] {
    [
        (Rule::Union, true),
        (Rule::Union, false),
        (Rule::Intersection, true),
        (Rule::Intersection, false),
        (Rule::Difference, true),
        (Rule::Difference, false),
    ]
}

fn fuse_in(
    model: &mut Model,
    degrees: f64,
    through: [f64; 3],
    rule: Rule,
    first_first: bool,
) -> Result<Vec<SolidKey>, Error> {
    let [first, second] = two_bars(model, degrees, through);
    let (arguments, tools) = if first_first {
        ([first], [second])
    } else {
        ([second], [first])
    };
    model
        .fuse(
            &arguments,
            &tools,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().to_vec())
}

/// Whether `solids` are one solid for each of `faces`, each with that many
/// faces, the model validates, and together they measure `volume` to within
/// the mesh's own error, with what is wrong when they do not.
fn how_far_off(
    model: &Model,
    solids: &[SolidKey],
    volume: f64,
    faces: &[usize],
) -> Result<(), String> {
    let policy = Policy::certified();
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("the answer does not validate: {why:?}"))?;
    let mut counted = Vec::with_capacity(solids.len());
    let (mut measured, mut error) = (0.0, 0.0);
    for &solid in solids {
        let view = model.solid(solid).expect("a solid the fuse made");
        counted.push(view.faces().count());
        let measure = view
            .measure(
                Deflection::of(0.001).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .map_err(|why| format!("the answer does not measure: {why:?}"))?;
        measured += measure.volume();
        error += measure.volume_error();
    }
    if counted != faces {
        return Err(format!("solids of {counted:?} faces"));
    }
    if (measured - volume).abs() > error {
        return Err(format!(
            "measured {measured} and the mesh may miss by {error}"
        ));
    }
    Ok(())
}

fn two_bars(model: &mut Model, degrees: f64, through: [f64; 3]) -> [SolidKey; 2] {
    let onto_x = Isometry::quarter_turns(&Axis::y(), 1).expect("a turn about y");
    let first_at = Isometry::translation(rcad_geom::Vector::new(
        through[0] - 2.0,
        through[1],
        through[2],
    ))
    .expect("a finite move")
    .compose(&onto_x);
    let first = model.add_round_bar(0.25, 4.0, &first_at).expect("a bar");
    let turned = Isometry::rotation(&Axis::z(), f64::to_radians(degrees))
        .expect("a turn about z")
        .compose(&onto_x);
    let along = turned.apply_to_vector(rcad_geom::Vector::new(0.0, 0.0, 1.0));
    let (start, length) = if through[1] == 0.0 {
        (
            [
                through[0] - 16.0 * along.x(),
                through[1] - 16.0 * along.y(),
                through[2] - 16.0 * along.z(),
            ],
            32.0,
        )
    } else {
        (through, 2.0)
    };
    let second_at = Isometry::translation(rcad_geom::Vector::new(start[0], start[1], start[2]))
        .expect("a finite move")
        .compose(&turned);
    let second = model
        .add_round_bar(0.25, length, &second_at)
        .expect("a bar");
    [first, second]
}

/// A bar along each of `ways`, each of radius 1/4 and cut off 1 either side of
/// `middle`, where their axes meet: each is the bar along `z` cut off at
/// `z = ±1`, turned onto its way about `ẑ × way` and moved to `middle`.
fn two_bars_crossing_at(model: &mut Model, ways: [[f64; 3]; 2], middle: [f64; 3]) -> [SolidKey; 2] {
    let caps = [-1.0, 1.0].map(|z| {
        rcad_geom::Plane::from_point_and_normal(
            rcad_geom::Point::new(0.0, 0.0, z),
            rcad_geom::Vector::new(0.0, 0.0, z),
        )
        .expect("a cap")
    });
    let up = rcad_geom::Vector::new(0.0, 0.0, 1.0);
    ways.map(|way| {
        let way = rcad_geom::Vector::new(way[0], way[1], way[2])
            .normalized()
            .expect("a way");
        let turn = Isometry::rotation(
            &Axis::new(rcad_geom::Point::origin(), up.cross(way)).expect("an axis"),
            up.dot(way).acos(),
        )
        .expect("a turn");
        let at = Isometry::translation(rcad_geom::Vector::new(middle[0], middle[1], middle[2]))
            .expect("a finite move")
            .compose(&turn);
        model
            .add_bar_cut_off_by(
                0.25,
                caps,
                &at,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a bar")
    })
}
