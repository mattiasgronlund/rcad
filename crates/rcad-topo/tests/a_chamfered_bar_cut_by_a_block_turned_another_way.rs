//! A bar with a chamfer, cut by a block placed by another composition of turns.
//!
//! The bar is built face by face: a cone from `z = 0`, radius one, to `z = 1`,
//! radius two, then a pipe of radius two up to `z = 5`. It is placed once by
//! the turn of `1.1` about the line through `(1, 0, −1)` along `(1, 2, 3)`. The
//! block runs from `z = −2` up to `z = ½` and is placed twice by half that
//! turn. [`Model::place_solid`] composes each placement onto the last, and
//! [`Isometry::compose`] multiplies two turns as quaternions, which rounds, so
//! the block's top comes out a rounding off square to the bar's axis.
//!
//! The fuse asks where the block's top meets the pipe and the cone, which
//! share the circle at `z = 1`. On paper the top cuts the cone in a circle of
//! radius one and a half and the pipe in one of radius two, so the three share
//! nothing. Off square by a rounding, each cut is an ellipse, and before row
//! Q2127 no road in `three_surfaces.rs` took a plane leaning to a pipe's or a
//! cone's axis, so the difference was refused
//! `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`. Each ellipse keeps within a
//! rounding of its circle, so the two keep apart and the three share nothing.
//!
//! The block is sixty across. Six to eight across, its sides run a rounding
//! off parallel to the bar's axis, three to four from it, and before row
//! Q2156 the triple of a side, the pipe and the cone was refused, with the
//! block placed by the bar's own turn as well as by two halves of it: the band
//! road gives a plane that steep no outer edge. The road for a plane beside a
//! pipe answers it now, and the second test pins six across. Ten across and
//! more, the fuse answered already.
//!
//! A block whose side runs one and a half from the bar's axis cuts a D-flat
//! along the whole bar. Placed by the bar's turn, the side is a rounding off
//! parallel to the axis and cuts both circles the pipe and the cone share,
//! and before row Q2177 no road took a plane that cuts them and leans to the
//! axis they share. The third test pins the four points that triple meets at
//! and every rule.
//! The side's normal and the axis have an `f64` dot of exactly nothing, and
//! before row Q2233 ordering points round the ellipse they cross in read that
//! dot and called them straight lines.
//!
//! A cone in a solid of its own beside the bar does not reach that question:
//! the fuse then asks where the pipe and the cone cross first.
//!
//! A block whose top is `z = 1`, neither solid placed, holds the circle the
//! pipe and the cone share in its top. The pipe and the cone each name the
//! side along that circle by the block's top since row Q2179, the name the
//! top's own face gives it, so the circle is one edge; the fourth test pins
//! the difference, the union and the intersection.
//!
//! The same pair with the bar placed by the turn and the block by the same
//! turn comes out with the top square to the bar's axis to the bit, `2e−15`
//! above the circle; since row Q2181 the fuse reads which side of the top, the
//! pipe or the cone a point is on off their stored numbers, and the fifth test
//! pins every rule. The intersection keeps a band of the pipe that `2e−15`
//! high, and since row Q2257 it is refused under `certified` as a spread past
//! `10¹⁶`, and answered under `tolerant` with a warning. With the block placed
//! by two halves of the turn the top is an ulp off square and crosses the
//! circle, which runs from `4.7e−16` above it to `1.6e−16` under it. The sixth
//! test pins that every rule is refused, and the seventh that the answer would
//! need a spread past `10¹⁶`: a limit.
//!
//! The answers are counted by the faces they have and not measured. The bar
//! alone, placed by the turn, meshes at `0.01` and at `4.06`. Before row Q2183
//! it was refused `DeflectionBelowWhatAMeshCanReach` with `closest` about
//! `242259`: the circle the pipe and the cone share was bounded on the pipe
//! only by the pipe crossed with itself, read in `f64` as running about `10¹⁶`
//! up the axis, and since then the plane its run names bounds it too.

use rcad_geom::{
    Axis, Cone, Cylinder, DynamicRange, EdgeRun, Isometry, MeetOfThree, Orientation, Plane, Point,
    Scale, Surface, Vector, meet_of_three_surfaces,
};
use rcad_predicates::expansion::{expansion_sum, two_diff, two_product};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftStep, DraftWire, Error, Invalid, Model, ProfileBuilder, Rule, SolidBuilder,
    SolidKey,
};

#[test]
fn a_chamfered_bar_less_a_block_placed_twice_keeps_the_bar_above_the_blocks_top() {
    let axis =
        Axis::new(Point::new(1.0, 0.0, -1.0), Vector::new(1.0, 2.0, 3.0)).expect("a direction");
    let turn = Isometry::rotation(&axis, 1.1).expect("a finite angle");
    let half = Isometry::rotation(&axis, 0.55).expect("a finite angle");

    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let bar = a_chamfered_bar(&mut model);
    model
        .place_solid(bar, &turn)
        .expect("this model made the key");
    let block = a_block_below_half(&mut model, 60.0);
    model
        .place_solid(block, &half)
        .expect("this model made the key");
    model
        .place_solid(block, &half)
        .expect("this model made the key");

    assert!(
        the_blocks_top_is_off_square_to_the_bars_axis(&model, bar, block),
        "the block placed twice came out exactly square to the bar after all"
    );

    assert_eq!(
        what_the_bar_less_the_block_is_made_of(&mut model, bar, block),
        (1, 1, 2),
        "the chamfer, the pipe, the bar's top and the block's top"
    );
}

/// The block six across, whose sides run three from the bar's axis and a
/// rounding off parallel to it, placed by the bar's own turn and by two halves
/// of it. Each side cuts the pipe no nearer than about `10¹⁶` along it, where
/// the cone is that wide, so the side, the pipe and the cone share nothing; before row
/// Q2156 that was refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`.
#[test]
fn a_chamfered_bar_less_a_block_six_across_keeps_the_bar_above_the_blocks_top() {
    let axis =
        Axis::new(Point::new(1.0, 0.0, -1.0), Vector::new(1.0, 2.0, 3.0)).expect("a direction");
    let turn = Isometry::rotation(&axis, 1.1).expect("a finite angle");
    let half = Isometry::rotation(&axis, 0.55).expect("a finite angle");

    for turns in [vec![turn], vec![half, half]] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = a_chamfered_bar(&mut model);
        model
            .place_solid(bar, &turn)
            .expect("this model made the key");
        let block = a_block_below_half(&mut model, 6.0);
        for placed in &turns {
            model
                .place_solid(block, placed)
                .expect("this model made the key");
        }
        assert_eq!(
            what_the_bar_less_the_block_is_made_of(&mut model, bar, block),
            (1, 1, 2),
            "the chamfer, the pipe, the bar's top and the block's top"
        );
    }
}

/// The block from `x = 1.5` out past the bar, taller and wider than it, placed
/// by the bar's own turn. Its side `x = 1.5` runs a rounding off parallel to
/// the bar's axis and cuts both circles of radius two the pipe and the cone
/// share, the one at `z = 1` and the one at `z = −3` past the apex, at
/// `y = ±√1.75`. Before row Q2177 the triple of that side, the pipe and the
/// cone was refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`; it is the four
/// points, each a rounding off where the turn puts them.
///
/// The side's normal and the pipe's axis have an `f64` dot of exactly nothing,
/// and they cross in an ellipse about `10¹⁶` long. Before row Q2189 the walk
/// read that dot, took the side and the pipe for a plane along a pipe, which
/// crosses it in straight lines, and the fuse was refused
/// `ACrossingThisCannotBeOrderedAlong`. Before row Q2233
/// `which_comes_first_round_a_crossing` read it too, said the two do not cross
/// in a conic, and the difference was refused
/// `TheseTwoSurfacesDoNotCrossInAConic`. Both ask the dot exactly now.
///
/// The difference is a D-flat along the whole bar: the chamfer, the pipe, the
/// bar's bottom, its top and the flat. The union is the chamfer and the pipe
/// with the bar's bottom and top and the block's six sides. The intersection
/// is the sliver past the side: a piece of the chamfer and of the pipe, closed
/// by the flat and by the bar's top, its bottom missing the block.
///
/// Before row Q2240 the intersection was refused
/// `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }`. The coarsest
/// mesh of the sliver folds, and the next one finer was refused
/// `DeflectionBelowWhatAMeshCanReach` with `closest` about `242259`: the
/// sliver's arc of the circle the pipe and the cone share is a piece of a
/// crossing whose second surface is the pipe itself, and the pipe crossed
/// with itself was read as running about `10¹⁶` up the axis.
#[test]
fn a_block_beside_a_chamfered_bars_axis_meets_both_its_circles_and_cuts_a_flat_along_it() {
    let turn = the_turn();
    let (model, bar, block) = a_bar_and_a_block_beside_its_axis(&turn);

    let [side, pipe, cone] = the_side_the_pipe_and_the_cone(&model, bar, block, &turn);
    let answer = meet_of_three_surfaces(
        [&side, &pipe, &cone],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("the side, the pipe and the cone are answered");
    let MeetOfThree::At(points) = &answer else {
        panic!("expected four points, got {answer:?}");
    };
    assert_eq!(points.len(), 4, "{answer:?}");
    let out = 1.75_f64.sqrt();
    for unplaced in [
        [1.5, out, -3.0],
        [1.5, -out, -3.0],
        [1.5, out, 1.0],
        [1.5, -out, 1.0],
    ] {
        let placed = turn.apply_to_point(Point::new(unplaced[0], unplaced[1], unplaced[2]));
        let wanted = [placed.x(), placed.y(), placed.z()];
        assert!(
            points.iter().any(|point| {
                let boxed = point.enclosure();
                (0..3).all(|axis| {
                    let middle = (boxed[axis].lo() + boxed[axis].hi()) / 2.0;
                    (middle - wanted[axis]).abs() < 1e-13 && boxed[axis].width() < 1e-13
                })
            }),
            "no narrow box round {wanted:?}, {unplaced:?} turned, in {answer:?}"
        );
    }

    let Surface::Plane(flat) = side else {
        panic!("the block's side is a plane: {side:?}");
    };
    let Surface::Cylinder(round) = pipe else {
        panic!("the bar's pipe is a pipe: {pipe:?}");
    };
    assert_eq!(
        flat.normal().dot(round.axis().direction()),
        0.0,
        "the side and the pipe's axis have an `f64` dot of exactly nothing, which is \
         what this pins"
    );

    for (rule, wanted) in [
        (Rule::Difference, (1, 1, 3)),
        (Rule::Union, (1, 1, 8)),
        (Rule::Intersection, (1, 1, 2)),
    ] {
        let (mut model, bar, block) = a_bar_and_a_block_beside_its_axis(&turn);
        assert_eq!(
            what_the_fuse_is_made_of(&mut model, bar, block, rule),
            wanted,
            "the cones, pipes and planes {rule:?} has faces on"
        );
    }
}

/// The bar and the block from `x = 1.5`, each placed by `turn`.
fn a_bar_and_a_block_beside_its_axis(turn: &Isometry) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let bar = a_chamfered_bar(&mut model);
    model
        .place_solid(bar, turn)
        .expect("this model made the key");
    let block = a_block_from_x_one_and_a_half(&mut model);
    model
        .place_solid(block, turn)
        .expect("this model made the key");
    (model, bar, block)
}

/// The block sixty across from `z = −2` up to `z = 1`, neither solid placed, so
/// the block's top holds the circle where the bar's cone meets its pipe. Less
/// the block, the bar is the pipe from `z = 1` up, closed by a disc there; with
/// it, the block with the pipe standing on it; and the two share the cone
/// closed by two discs. Before row Q2179 the difference and the union were
/// refused `ResultIsNotManifold { count: 1 }`: the pipe kept the circle as the
/// bar's own edge, and the block's top named it a new crossing of the top and
/// the pipe, so one circle was two edges with one face along each.
#[test]
fn a_block_whose_top_holds_the_circle_where_the_cone_meets_the_pipe_cuts_the_bar_there() {
    for (rule, wanted) in [
        (Rule::Difference, (0, 1, 2)),
        (Rule::Union, (0, 1, 7)),
        (Rule::Intersection, (1, 0, 2)),
    ] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = a_chamfered_bar(&mut model);
        let block = a_block_from_below(&mut model, 60.0, 1.0);
        assert_eq!(
            what_the_fuse_is_made_of(&mut model, bar, block, rule),
            wanted,
            "the cones, pipes and planes {rule:?} has faces on"
        );
    }
}

/// The same bar and block, both placed by the turn of `1.1`. The block's
/// placement composes its own offset in and rounds, so its top comes out
/// square to the bar's axis to the bit and about `2e−15` up it from the
/// circle the pipe and the cone share. Before row Q2181 every rule was refused
/// `Undecidable` on the side of the top at the bar's seam corner on that
/// circle, whose box is about `1e−13` wide, and then on the side of the cone at
/// a point where the top cuts the pipe. Each side is now read off the stored
/// numbers of a plane square to the one axis of a pipe and a cone.
///
/// **The intersection keeps a band of the pipe about `2e−15` high**, its two
/// rings a corner of one `2.2e−15` from the other, beside block corners as far
/// as `42.9` from the origin: a spread of `1.9e16`, past §3.5's `10¹⁶`. Under
/// `certified` it is refused `DynamicRangeTooWide`, on how close a corner of
/// one loop of a face comes to a side of another (`the_narrowest_a_face_is` in
/// `validate.rs`, queue rows Q2237 and Q2257). Under `tolerant` that spread is
/// a warning: the answer is the cone, closed below by the bar's bottom and
/// above by the block's top with the band of pipe between, it carries one
/// record, of the range check, and it validates under `tolerant` alone (§68).
#[test]
fn a_block_whose_top_holds_the_circle_turned_with_the_bar_cuts_the_bar_just_above_it() {
    let both_turned = || {
        let turn = the_turn();
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = a_chamfered_bar(&mut model);
        model
            .place_solid(bar, &turn)
            .expect("this model made the key");
        let block = a_block_from_below(&mut model, 60.0, 1.0);
        model
            .place_solid(block, &turn)
            .expect("this model made the key");
        (model, bar, block)
    };
    for (rule, wanted) in [(Rule::Difference, (0, 1, 2)), (Rule::Union, (0, 1, 7))] {
        let (mut model, bar, block) = both_turned();
        assert_eq!(
            what_the_fuse_is_made_of(&mut model, bar, block, rule),
            wanted,
            "the cones, pipes and planes {rule:?} has faces on"
        );
    }

    let (mut model, bar, block) = both_turned();
    let certified = model.fuse(
        &[bar],
        &[block],
        Rule::Intersection,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert_refused_for_the_band(&certified);

    let (mut model, bar, block) = both_turned();
    let tolerant = Policy::tolerant(100.0);
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(&[bar], &[block], Rule::Intersection, &tolerant, &mut log)
        .unwrap_or_else(|why| panic!("the tolerant intersection was refused: {why:?}"));
    let [band] = fused.solids()[..] else {
        panic!("one solid is left: {:?}", fused.solids())
    };
    assert_eq!(what_the_solid_is_made_of(&model, band), (1, 1, 2));
    assert!(
        log.records().len() == 1 && log.records()[0].predicate() == "dynamic range check",
        "{:#?}",
        log.records()
    );
    model
        .validate(&tolerant, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the tolerant intersection does not validate: {why}"));
    assert_refused_for_the_band(&model.validate(&Policy::certified(), &mut ProvenanceLog::new()));
}

/// Checks that `refused` is `DynamicRangeTooWide`, past the `10¹¹` §3.5
/// supports, for the band of pipe the intersection keeps.
fn assert_refused_for_the_band<T: std::fmt::Debug>(refused: &rcad_topo::Result<T>) {
    assert!(
        matches!(
            refused,
            Err(Error::Invalid(Invalid::DynamicRangeTooWide { ratio }))
                if *ratio > DynamicRange::LARGEST_SUPPORTED_RATIO
        ),
        "{refused:?}"
    );
}

/// The same block placed by two halves of the turn instead, each rounding:
/// its top comes out an ulp off square to the bar's axis and crosses the
/// circle the pipe and the cone share, which the next test measures. Every
/// rule is refused `Undecidable` on the side of the top at the bar's seam
/// corner on that circle.
#[test]
fn a_block_whose_top_holds_the_circle_turned_in_two_halves_is_refused() {
    let half = the_half_turn();
    for rule in [Rule::Difference, Rule::Union, Rule::Intersection] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = a_chamfered_bar(&mut model);
        model
            .place_solid(bar, &the_turn())
            .expect("this model made the key");
        let block = a_block_from_below(&mut model, 60.0, 1.0);
        for _ in 0..2 {
            model
                .place_solid(block, &half)
                .expect("this model made the key");
        }
        let refused = model.fuse(
            &[bar],
            &[block],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert!(
            matches!(refused, Err(Error::Undecidable { .. })),
            "{rule:?}: {refused:?}"
        );
    }
}

/// Why the refusal above is a limit and not a gap (§16.10). Every answer would
/// keep a crescent of the bar less than `1e−15` high, while the block's sides
/// run thirty from the bar's axis: a spread past `10¹⁶`. §3.5's ceiling is
/// `10¹¹`, and a feature written straight into the coordinates is gone by
/// about `10¹⁶` (§294.23 found the same of its own seam).
///
/// Let `n` and `d` be the top's normal and offset, `a` the cone's apex and `w`
/// its axis, and `e = n − w`, which is exact. The circle is two along the axis
/// from the apex and two from it, so a point on it is `a + 2ŵ + 2û` for a unit
/// `û` square to `w`. Its height above the top, `n·p − d`, runs over
/// `centre ± tilt`, with `centre = n·a − d + 2·n·ŵ` and
/// `tilt = 2·|n × ŵ| = 2·|e × w|/|w|`. Put `n·w = 1 + α` and `w·w = 1 + β`.
/// Then `n·ŵ` is `1 + α − β/2` up to about `α·β + β²`, below `10⁻³⁰` here, so
/// `centre` is `n·a − d + 2·n·w − w·w + 1`, a sum of products of stored numbers
/// added up exactly and rounded once.
///
/// `centre` comes out `1.553181e−16` and `tilt` `3.153804e−16`, as an
/// 80-digit decimal sum of the same stored numbers gave too, so the circle
/// runs from `4.7e−16` above the top to `1.6e−16` under it, and the top, the
/// pipe and the cone meet at two points.
#[test]
fn turned_in_two_halves_the_blocks_top_cuts_the_circle_too_close_to_hold() {
    let half = the_half_turn();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let bar = a_chamfered_bar(&mut model);
    model
        .place_solid(bar, &the_turn())
        .expect("this model made the key");
    let block = a_block_from_below(&mut model, 60.0, 1.0);
    for _ in 0..2 {
        model
            .place_solid(block, &half)
            .expect("this model made the key");
    }

    let up = the_turn().apply_to_vector(Vector::new(0.0, 0.0, 1.0));
    let [top, pipe, cone] = the_face_facing_the_pipe_and_the_cone(&model, bar, block, up);
    let (Surface::Plane(flat), Surface::Cylinder(round), Surface::Cone(chamfer)) =
        (top, pipe, cone)
    else {
        panic!("a plane, a pipe and a cone: {top:?}, {pipe:?}, {cone:?}");
    };
    assert_eq!((round.radius(), chamfer.slope()), (2.0, 1.0));

    let n = flat.normal().components();
    let d = flat.offset();
    let a = chamfer.apex().coordinates();
    let w = chamfer.axis().direction().components();
    let e = [0, 1, 2].map(|axis| {
        let (difference, lost) = two_diff(n[axis], w[axis]);
        assert_eq!(lost, 0.0, "n − w rounds in {axis}");
        difference
    });

    let alpha = exactly(&[(n[0], w[0]), (n[1], w[1]), (n[2], w[2])], &[-1.0]);
    let beta = exactly(&[(w[0], w[0]), (w[1], w[1]), (w[2], w[2])], &[-1.0]);
    assert!(
        alpha.abs() < 1e-15 && beta.abs() < 1e-15,
        "{alpha:e} {beta:e}"
    );
    let centre = exactly(
        &[
            (n[0], a[0]),
            (n[1], a[1]),
            (n[2], a[2]),
            (2.0 * n[0], w[0]),
            (2.0 * n[1], w[1]),
            (2.0 * n[2], w[2]),
            (-w[0], w[0]),
            (-w[1], w[1]),
            (-w[2], w[2]),
        ],
        &[-d, 1.0],
    );
    let across = [
        e[1] * w[2] - e[2] * w[1],
        e[2] * w[0] - e[0] * w[2],
        e[0] * w[1] - e[1] * w[0],
    ];
    let length = |v: [f64; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let tilt = 2.0 * length(across) / length(w);
    assert!((centre - 1.553181e-16).abs() < 1e-22, "{centre:e}");
    assert!((tilt - 3.153804e-16).abs() < 1e-22, "{tilt:e}");

    let (highest, lowest) = (centre + tilt, centre - tilt);
    assert!(
        highest > 0.0 && lowest < 0.0,
        "the top crosses the circle: {highest:e} {lowest:e}"
    );
    let crossing = meet_of_three_surfaces(
        [&top, &pipe, &cone],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(&crossing, Ok(MeetOfThree::At(points)) if points.len() == 2),
        "{crossing:?}"
    );

    let crescent = highest.max(-lowest);
    assert!(crescent < 1e-15, "{crescent:e}");
    let spread = 30.0 / crescent;
    assert!(spread > 1e11, "past §3.5's ceiling of 1e11: {spread:e}");
    assert!(
        spread > 1e16,
        "past the 1e16 a feature is already gone by: {spread:e}"
    );
}

/// `Σ left·right + Σ plain`, added up without rounding and rounded at the end.
fn exactly(products: &[(f64, f64)], plain: &[f64]) -> f64 {
    let parts = products
        .iter()
        .flat_map(|&(left, right)| {
            let (product, lost) = two_product(left, right);
            [lost, product]
        })
        .chain(plain.iter().copied());
    let mut sum: Vec<f64> = Vec::new();
    for part in parts {
        let mut out = vec![0.0; sum.len() + 1];
        let used = expansion_sum(&sum, &[part], &mut out);
        out.truncate(used);
        sum = out;
    }
    sum.iter().sum()
}

/// The turn of `1.1` about the line through `(1, 0, −1)` along `(1, 2, 3)`.
fn the_turn() -> Isometry {
    Isometry::rotation(&the_turns_axis(), 1.1).expect("a finite angle")
}

/// Half [`the_turn`].
fn the_half_turn() -> Isometry {
    Isometry::rotation(&the_turns_axis(), 0.55).expect("a finite angle")
}

fn the_turns_axis() -> Axis {
    Axis::new(Point::new(1.0, 0.0, -1.0), Vector::new(1.0, 2.0, 3.0)).expect("a direction")
}

/// The block's side that was `x = 1.5` before the turn, and the bar's pipe and
/// cone, each where the turn put it.
fn the_side_the_pipe_and_the_cone(
    model: &Model,
    bar: SolidKey,
    block: SolidKey,
    turn: &Isometry,
) -> [Surface; 3] {
    the_face_facing_the_pipe_and_the_cone(
        model,
        bar,
        block,
        turn.apply_to_vector(Vector::new(-1.0, 0.0, 0.0)),
    )
}

/// The block's face whose normal is about `facing`, and the bar's pipe and
/// cone, each where its placement put it.
fn the_face_facing_the_pipe_and_the_cone(
    model: &Model,
    bar: SolidKey,
    block: SolidKey,
    facing: Vector,
) -> [Surface; 3] {
    let facing = facing.components();
    let face = model
        .solid(block)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .find(|surface| {
            let Surface::Plane(plane) = surface else {
                return false;
            };
            let normal = plane.normal().components();
            (0..3)
                .map(|which| normal[which] * facing[which])
                .sum::<f64>()
                > 0.999
        })
        .expect("the block has a face that way");
    let curved = |wanted: fn(&Surface) -> bool| {
        model
            .solid(bar)
            .expect("this model made the key")
            .faces()
            .map(|face| face.surface())
            .find(wanted)
            .expect("the bar has a pipe and a cone")
    };
    let pipe = curved(|surface| matches!(surface, Surface::Cylinder(_)));
    let cone = curved(|surface| matches!(surface, Surface::Cone(_)));
    [face, pipe, cone]
}

/// The bar less the block, which is one solid: how many cones, pipes and
/// planes it has faces on.
fn what_the_bar_less_the_block_is_made_of(
    model: &mut Model,
    bar: SolidKey,
    block: SolidKey,
) -> (usize, usize, usize) {
    what_the_fuse_is_made_of(model, bar, block, Rule::Difference)
}

/// The bar and the block fused under `rule`, which is one solid: how many
/// cones, pipes and planes it has faces on.
fn what_the_fuse_is_made_of(
    model: &mut Model,
    bar: SolidKey,
    block: SolidKey,
    rule: Rule,
) -> (usize, usize, usize) {
    let solids = model
        .fuse(
            &[bar],
            &[block],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the {rule:?} was refused: {why:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the {rule:?} does not validate: {why}"));
    assert_eq!(solids.len(), 1, "one solid is left");
    what_the_solid_is_made_of(model, solids[0])
}

/// How many cones, pipes and planes `solid` has faces on.
fn what_the_solid_is_made_of(model: &Model, solid: SolidKey) -> (usize, usize, usize) {
    let (mut cones, mut pipes, mut planes) = (0, 0, 0);
    for face in model.solid(solid).expect("this model made the key").faces() {
        let surface = face.surface();
        if let Surface::Cone(_) = surface {
            cones += 1;
        } else if let Surface::Cylinder(_) = surface {
            pipes += 1;
        } else if let Surface::Plane(_) = surface {
            planes += 1;
        } else {
            panic!("a face the bar and the block do not have: {surface:?}");
        }
    }
    (cones, pipes, planes)
}

/// The block's top face, `z = ½` before it was placed, against the bar's pipe:
/// whether the normal and the axis differ in some component, up to which way
/// each points.
fn the_blocks_top_is_off_square_to_the_bars_axis(
    model: &Model,
    bar: SolidKey,
    block: SolidKey,
) -> bool {
    let axis = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .find_map(|face| {
            let Surface::Cylinder(pipe) = face.surface() else {
                return None;
            };
            Some(pipe.axis().direction().components())
        })
        .expect("the bar has a pipe");
    let tops: Vec<[f64; 3]> = model
        .solid(block)
        .expect("this model made the key")
        .faces()
        .filter_map(|face| {
            let Surface::Plane(plane) = face.surface() else {
                return None;
            };
            Some(plane.normal().components())
        })
        .filter(|normal| {
            let along: f64 = (0..3).map(|which| normal[which] * axis[which]).sum();
            along.abs() > 0.5
        })
        .collect();
    assert_eq!(tops.len(), 2, "the block has a top and a bottom: {tops:?}");
    tops.iter()
        .any(|normal| (0..3).any(|which| normal[which].abs() != axis[which].abs()))
}

/// The block `across` wide about the `z` axis and from `z = −2` up to
/// `z = ½`, not yet placed.
fn a_block_below_half(model: &mut Model, across: f64) -> SolidKey {
    a_block_from_below(model, across, 0.5)
}

/// The block `across` wide about the `z` axis and from `z = −2` up to
/// `z = top`, not yet placed.
fn a_block_from_below(model: &mut Model, across: f64, top: f64) -> SolidKey {
    let square = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [across, 0.0],
        [across, across],
        [0.0, across],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square is a profile");
    let corner =
        Isometry::translation(Vector::new(-across / 2.0, -across / 2.0, -2.0)).expect("a shift");
    model
        .add_extrusion(&square, top + 2.0, &corner)
        .expect("the block has a height")
}

/// The block from `x = 1.5` to `x = 11.5`, `y = −5` to `y = 5` and `z = −2`
/// to `z = 6`, not yet placed.
fn a_block_from_x_one_and_a_half(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square is a profile");
    let corner = Isometry::translation(Vector::new(1.5, -5.0, -2.0)).expect("a shift");
    model
        .add_extrusion(&square, 8.0, &corner)
        .expect("the block has a height")
}

/// The bar, not yet placed: the cone of slope one with its apex at `z = −1`
/// from `z = 0` to `z = 1`, and the pipe of radius two from `z = 1` to `z = 5`,
/// each with its seam on the ruling through `y = 0`, `x > 0`.
fn a_chamfered_bar(model: &mut Model) -> SolidKey {
    let chamfer = Cone::new(
        Axis::new(Point::new(0.0, 0.0, -1.0), Vector::new(0.0, 0.0, 1.0)).expect("a direction"),
        1.0,
    )
    .expect("a positive slope");
    let wall = Cylinder::new(Axis::z(), 2.0).expect("a positive radius");
    let mut builder = SolidBuilder::new();

    let corners = [(1.0, 0.0), (2.0, 1.0), (2.0, 5.0)].map(|(x, z)| {
        builder
            .add_vertex(Point::new(x, 0.0, z))
            .expect("a finite point")
    });
    let seams = [0, 1].map(|which| {
        builder
            .add_edge(corners[which], corners[which + 1], EdgeRun::Straight)
            .expect("a ruling is straight")
    });
    let bottom = builder
        .add_edge(
            corners[0],
            corners[0],
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("the bottom cuts the cone in one closed curve");
    let middle = builder
        .add_edge(
            corners[1],
            corners[1],
            EdgeRun::RightRoundWhereItCrosses {
                and: Surface::Plane(level(1.0, 1.0)),
            },
        )
        .expect("the pipe and the cone cross in the circle at z = 1");
    let top = builder
        .add_edge(
            corners[2],
            corners[2],
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("the top cuts the pipe in one closed curve");

    let underneath = wire(&mut builder, &[(bottom, Orientation::Reversed)]);
    let round_the_chamfer = wire(
        &mut builder,
        &[
            (bottom, Orientation::Forward),
            (seams[0], Orientation::Forward),
            (middle, Orientation::Reversed),
            (seams[0], Orientation::Reversed),
        ],
    );
    let round_the_wall = wire(
        &mut builder,
        &[
            (middle, Orientation::Forward),
            (seams[1], Orientation::Forward),
            (top, Orientation::Reversed),
            (seams[1], Orientation::Reversed),
        ],
    );
    let on_top = wire(&mut builder, &[(top, Orientation::Forward)]);

    let faces = [
        (Surface::Plane(level(0.0, -1.0)), underneath),
        (Surface::Cone(chamfer), round_the_chamfer),
        (Surface::Cylinder(wall), round_the_wall),
        (Surface::Plane(level(5.0, 1.0)), on_top),
    ]
    .map(|(surface, walk)| {
        (
            builder
                .add_face(surface, &[walk])
                .expect("a face trimmed by its walk"),
            Orientation::Forward,
        )
    });
    builder.add_shell(&faces).expect("four faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a chamfered bar described this way is a solid")
}

/// The wire that walks `steps`, each edge the way it names.
fn wire(builder: &mut SolidBuilder, steps: &[(DraftEdge, Orientation)]) -> DraftWire {
    let steps: Vec<DraftStep> = steps
        .iter()
        .map(|&(edge, turn)| {
            DraftStep::Along(
                builder
                    .add_coedge(edge, turn)
                    .expect("an edge of this build"),
            )
        })
        .collect();
    builder.add_wire(&steps).expect("a closed walk is a wire")
}

/// The plane `z = at`, facing `+z` where `outward` is one and `−z` where it is
/// minus one.
fn level(at: f64, outward: f64) -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, at), Vector::new(0.0, 0.0, outward))
        .expect("the z axis is a unit normal")
}
