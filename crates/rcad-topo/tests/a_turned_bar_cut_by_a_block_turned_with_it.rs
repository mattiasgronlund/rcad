//! A round bar of radius 3 and height 5, notched by a block of side 2, with
//! bar and block turned together by `Model::place_solid`. Q765's probe found
//! every such fuse refused under `Policy::certified()` at *where a plane and a
//! pipe cross*, turned half a turn about `y` or one radian about `(1, 1, 1)`,
//! and kept nothing but the doc of `a_turned_bar_comes_back_the_same` in
//! `rcad-step`. Q1732 rebuilt the fuses and pins what they answer here
//! (`docs/decisions.md` §246.6).
//!
//! **The shapes.** The bar is `add_round_bar(3, 5)`, with its seam on the
//! ruling `(3, 0, z)`. A notch is a block `2 × 2 × 2` from `z = 1` to `z = 3`:
//! the *seam notch* `[2, 4] × [−1, 1]` across the seam, or the *wall notch*
//! `[−1, 1] × [2, 4]` a quarter turn round the wall from it. Four sides of
//! either run along the bar's axis.
//!
//! **Unturned, and turned an exact half turn** about `y` by
//! `Isometry::quarter_turns`, every fuse answers what it holds by hand under
//! both policies and logs nothing.
//!
//! **Half a turn about `y` by `Isometry::rotation(y, π)`: refused, and a side
//! a rounding off parallel to the axis is not why.** Every side runs *exactly*
//! along the turned bar: how it leans against the pipe comes out zero. But
//! `sin π` rounds to `1.2e-16`, so the turn is not one of a cube's, and the
//! fuse reads the bar's seam corner through `where_a_corner_lands_after_moving`
//! as a box `± CORNER_AFTER_MOVING_ROUNDOFF · 3`. The bottom cap circle starts
//! at that corner. Every certified fuse is refused *order along a crossing*
//! between that box and where a side's line crosses the bottom circle — Q885's
//! case, met there on a shape only a probe builds, and here on one a designer
//! draws. Every tolerant fuse is refused `CannotTellTheCellsOfAFaceApart`. The
//! seam notch tilted off the axis, so that no side runs along it, is refused at
//! the same box when certified, and as two curves touching when tolerant.
//!
//! **One radian about `(1, 1, 1)`: answered, though a side lies a rounding off
//! parallel to the axis.** The turn rounds each side's normal and the bar's
//! axis apart, so every side leans against the pipe by a rounding. Its plane's
//! crossing with the pipe is then named a closed one, an ellipse whose sweep
//! starts more than `1e15` from the bar. Every fuse, under either policy, was
//! refused *order along a crossing*: first between the two points where one
//! side's plane crosses the wall at the two caps, `5` apart, with an enclosure
//! past `100`, and then, once those were ordered, between a cap's and where
//! the line two sides of the notch cross in leaves the pipe, `1e16` off.
//! `fuse/arrange.rs` orders both now (`along_the_stretch_between_the_two` and
//! `Arrangement::as_the_pipes_axis_says`). Every fuse was then refused
//! `CannotNameWhichPieceOfACrossingThisIs` on a wall edge along a side's
//! ellipse (Q1885), until Q1882 walked such an arc up the pipe. Every fuse now
//! answers what it holds by hand under both policies and logs nothing, and so
//! does the seam notch tilted off the axis, with the volume it has unturned.
//!
//! **The half turn by rotation is a gap, not a limit** (§16.10). A turn moves
//! no material, so each answer exists and is the unturned one. The exact half
//! turn reaches it for the same shape, and so does the one-radian turn, whose
//! rounding is in the turn itself.
//!
//! *Where a plane and a pipe cross*, the predicate Q765 named, refuses none of
//! these fuses.

use core::f64::consts::PI;

use rcad_geom::{
    Axis, CORNER_AFTER_MOVING_ROUNDOFF, Cylinder, Deflection, Isometry, Plane, Point, Scale,
    Surface, SurfaceMeet, Unit, Vector, how_a_plane_leans_against_a_pipe, intersect,
    where_a_closed_crossing_starts, where_a_corner_lands_after_moving,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog, Record};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

const RADIUS: f64 = 3.0;
const HEIGHT: f64 = 5.0;
const SIDE: f64 = 2.0;
const RULES: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];
const ALONG_A_CROSSING: &str = "order along a crossing";
/// How near a point moved back out of a refusal's box must come to where it is
/// named to be. The boxes are about `1e-15` wide.
const CLOSE: f64 = 1e-12;

/// A notch's corner nearest the origin, before any turn: the seam notch is
/// `[2, 4] × [−1, 1] × [1, 3]`.
const THE_SEAM_NOTCH: [f64; 3] = [2.0, -1.0, 1.0];
/// `[−1, 1] × [2, 4] × [1, 3]`.
const THE_WALL_NOTCH: [f64; 3] = [-1.0, 2.0, 1.0];
const BOTH_NOTCHES: [[f64; 3]; 2] = [THE_SEAM_NOTCH, THE_WALL_NOTCH];

/// The controls: the bar unturned, and turned half a turn about `y` by two
/// quarter turns, which round nothing.
#[test]
fn a_notched_bar_unturned_or_turned_an_exact_half_turn_answers_every_fuse_by_hand() {
    answers_every_fuse_by_hand("unturned", &Isometry::identity());
    answers_every_fuse_by_hand("an exact half turn", &half_a_turn_exactly());
}

/// Every side of either notch, turned by `rotation(y, π)` with the bar, runs
/// exactly along the bar's axis: the turn rounds a side's normal and the axis
/// the same way, and `n · w` comes out zero.
#[test]
fn turned_half_a_turn_by_rotation_every_notch_side_runs_exactly_along_the_bar() {
    let turn = half_a_turn_by_rotation();
    for notch in BOTH_NOTCHES {
        let (model, bar, notch_key) = a_notched_bar(&turn, notch, &Isometry::identity());
        let pipe = the_pipe(&model, bar);
        for side in the_sides(&model, notch_key, &turn) {
            assert_eq!(
                how_a_plane_leans_against_a_pipe(&side, &pipe),
                Decision::Zero,
                "notch at {notch:?}, side {side:?}"
            );
        }
    }
}

/// What tells the two half turns apart is the bar's seam corner on its bottom
/// cap. Two quarter turns hand it back as the exact point it lands on;
/// `rotation(y, π)` hands it back as a box `± CORNER_AFTER_MOVING_ROUNDOFF · 3`
/// round where it lands.
#[test]
fn the_seam_corner_is_a_box_after_rotation_by_pi_and_a_point_after_two_quarter_turns() {
    let exactly = half_a_turn_exactly();
    let (corner, wobble) = the_seam_corner_as_the_fuse_reads_it(&exactly);
    assert_eq!(wobble, 0.0);
    assert_eq!(corner, PredicatePoint::from([-RADIUS, 0.0, 0.0]));

    let rounded = half_a_turn_by_rotation();
    let (corner, wobble) = the_seam_corner_as_the_fuse_reads_it(&rounded);
    assert_eq!(wobble, CORNER_AFTER_MOVING_ROUNDOFF * RADIUS);
    let lands = rounded
        .apply_to_point(Point::new(RADIUS, 0.0, 0.0))
        .coordinates();
    for (side, at) in corner.enclosure().into_iter().zip(lands) {
        assert!(side.lo() < at && at < side.hi(), "{at} outside {side:?}");
        assert!(side.width() >= wobble, "{side:?} is narrower than {wobble}");
    }
}

/// Q885's case on a designer's shape: the bottom cap circle starts inside the
/// seam corner's box, and a side's line crosses the circle a little way round.
/// Every fuse of either notch, under either policy, answers what it holds by
/// hand and logs nothing, as the exact half turn does.
#[test]
fn turned_half_a_turn_by_rotation_every_fuse_answers_what_it_holds_by_hand() {
    answers_every_fuse_by_hand("half a turn by rotation", &half_a_turn_by_rotation());
}

/// The seam notch tilted off the bar's axis, so that no side of it runs along
/// the bar, is refused at the same box under the same turn: what refuses here
/// is the seam corner, and not a side along the axis.
#[test]
fn turned_half_a_turn_by_rotation_a_notch_tilted_off_the_axis_is_refused_at_the_same_box() {
    let turn = half_a_turn_by_rotation();
    for rule in RULES {
        let what = format!("tilted, {rule:?}");
        let (_, fused, _) = notched_and_fused(&turn, THE_SEAM_NOTCH, &a_tilt(), rule, false);
        refused_between_the_seam_corner_and_the_bottom_circle(fused, &turn, &what);

        let (_, fused, _) = notched_and_fused(&turn, THE_SEAM_NOTCH, &a_tilt(), rule, true);
        let Err(Error::TwoCurvesTouchWithoutCrossing { .. }) = fused else {
            panic!("{what}, tolerant: {fused:?}");
        };
    }
}

/// Turned one radian about `(1, 1, 1)`, every side of either notch leans
/// against the bar's axis by a rounding. Its plane's crossing with the pipe is
/// named a closed one, and the sweep round it starts more than `1e15` from the
/// bar.
#[test]
fn turned_one_radian_about_a_diagonal_every_notch_side_leans_by_a_rounding_and_crosses_far_off() {
    let turn = one_radian_about_a_diagonal();
    for notch in BOTH_NOTCHES {
        let (model, bar, notch_key) = a_notched_bar(&turn, notch, &Isometry::identity());
        let pipe = the_pipe(&model, bar);
        let wall = Surface::Cylinder(pipe);
        for side in the_sides(&model, notch_key, &turn) {
            let what = format!("notch at {notch:?}, side {side:?}");
            let leans = how_a_plane_leans_against_a_pipe(&side, &pipe);
            assert!(
                matches!(leans, Decision::Pos | Decision::Neg),
                "{what}: {leans:?}"
            );
            let flat = Surface::Plane(side);
            let SurfaceMeet::Curves(runs) = intersect(
                &flat,
                &wall,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a plane and a pipe meet in something named") else {
                panic!("{what}: no curves");
            };
            let [run] = runs.as_slice() else {
                panic!("{what}: {runs:?}");
            };
            let [x, y, z] =
                where_a_closed_crossing_starts(run, [&flat, &wall], &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"))
                    .coordinates();
            let how_far = (x * x + y * y + z * z).sqrt();
            assert!(how_far > 1e15, "{what}: starts {how_far} from the bar");
        }
    }
}

/// Every fuse of either notch, under either policy, answers what it holds by
/// hand and logs nothing. It puts the nodes round each side's long ellipse in
/// order: where a side meets the caps and the notch's ends, near the bar, and
/// where the line two sides of the notch cross in leaves the pipe, `1e16` off.
/// It then walks each wall edge on such an ellipse up the pipe (Q1882), where
/// it was refused `CannotNameWhichPieceOfACrossingThisIs` (Q1885).
#[test]
fn turned_one_radian_about_a_diagonal_every_fuse_answers_what_it_holds_by_hand() {
    answers_every_fuse_by_hand("one radian about (1, 1, 1)", &one_radian_about_a_diagonal());
}

/// The control for the one-radian turn: the seam notch tilted so that no side
/// runs along the bar answers every fuse under both policies, with the volume
/// the same fuse has unturned.
#[test]
fn turned_one_radian_about_a_diagonal_a_notch_tilted_off_the_axis_answers_every_fuse() {
    let turn = one_radian_about_a_diagonal();
    for tolerant in [false, true] {
        for rule in RULES {
            let what = format!("tolerant {tolerant}, {rule:?}");
            let (model, fused, log) = notched_and_fused(
                &Isometry::identity(),
                THE_SEAM_NOTCH,
                &a_tilt(),
                rule,
                tolerant,
            );
            assert!(log.is_empty(), "{what}, unturned: logged {log:?}");
            let (unturned, unturned_error) =
                the_volume_of_the_one_solid(&model, fused, &format!("{what}, unturned"));

            let (model, fused, log) =
                notched_and_fused(&turn, THE_SEAM_NOTCH, &a_tilt(), rule, tolerant);
            assert!(log.is_empty(), "{what}, turned: logged {log:?}");
            let (turned, turned_error) =
                the_volume_of_the_one_solid(&model, fused, &format!("{what}, turned"));
            assert!(
                (turned - unturned).abs() <= turned_error + unturned_error,
                "{what}: turned {turned} ± {turned_error}, unturned {unturned} ± {unturned_error}"
            );
        }
    }
}

/// Fuses either notch with the bar, both placed by `turn`, by every rule under
/// both policies, and checks that each comes back one sound solid holding what
/// [`what_it_holds_by_hand`] says, with nothing logged.
fn answers_every_fuse_by_hand(which: &str, turn: &Isometry) {
    for notch in BOTH_NOTCHES {
        for tolerant in [false, true] {
            for rule in RULES {
                let what = format!("{which}, notch at {notch:?}, tolerant {tolerant}, {rule:?}");
                let (model, fused, log) =
                    notched_and_fused(turn, notch, &Isometry::identity(), rule, tolerant);
                let (volume, error) = the_volume_of_the_one_solid(&model, fused, &what);
                let by_hand = what_it_holds_by_hand(rule);
                assert!(
                    (volume - by_hand).abs() <= error,
                    "{what}: measured {volume} ± {error}, by hand {by_hand}"
                );
                assert!(log.is_empty(), "{what}: logged {log:?}");
                model
                    .validate(&Policy::certified(), &mut ProvenanceLog::new())
                    .unwrap_or_else(|why| panic!("{what}: {why:?}"));
            }
        }
    }
}

fn half_a_turn_exactly() -> Isometry {
    Isometry::quarter_turns(&Axis::y(), 2).expect("two quarter turns about y")
}

fn half_a_turn_by_rotation() -> Isometry {
    Isometry::rotation(&Axis::y(), PI).expect("half a turn about y")
}

fn one_radian_about_a_diagonal() -> Isometry {
    let diagonal =
        Axis::new(Point::origin(), Vector::new(1.0, 1.0, 1.0)).expect("a finite direction");
    Isometry::rotation(&diagonal, 1.0).expect("one radian about (1, 1, 1)")
}

/// `0.3` radians about an axis through `(3, 0, 2)`, on the seam inside the
/// notch, that runs along no side of it.
fn a_tilt() -> Isometry {
    let through_the_notch =
        Axis::new(Point::new(3.0, 0.0, 2.0), Vector::new(1.0, 2.0, 0.5)).expect("a finite axis");
    Isometry::rotation(&through_the_notch, 0.3).expect("a tilt")
}

/// Builds the bar and a notch with its corner at `notch`, and fuses them by
/// `rule`, bar first.
fn notched_and_fused(
    turn: &Isometry,
    notch: [f64; 3],
    tilt: &Isometry,
    rule: Rule,
    tolerant: bool,
) -> (Model, Result<Vec<SolidKey>, Error>, ProvenanceLog) {
    let (mut model, bar, notch_key) = a_notched_bar(turn, notch, tilt);
    let policy = if tolerant {
        Policy::tolerant(model.scale().nominal_size())
    } else {
        Policy::certified()
    };
    let mut log = ProvenanceLog::new();
    let fused = model
        .fuse(&[bar], &[notch_key], rule, &policy, &mut log)
        .map(|fused| fused.solids().to_vec());
    (model, fused, log)
}

/// The bar, placed by `turn`, and the notch, placed by `turn` after `tilt`.
fn a_notched_bar(
    turn: &Isometry,
    [x, y, z]: [f64; 3],
    tilt: &Isometry,
) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::new(Unit::Millimetre, 5.0).expect("5 mm is a working size"));
    let bar = model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar");
    model.place_solid(bar, turn).expect("a turn moves a bar");
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [SIDE, 0.0], [SIDE, SIDE], [0.0, SIDE]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square is a profile");
    let notch = model
        .add_extrusion(
            &square,
            SIDE,
            &Isometry::translation(Vector::new(x, y, z)).expect("a finite shift"),
        )
        .expect("a block");
    model
        .place_solid(notch, &turn.compose(tilt))
        .expect("a turn moves a block");
    (model, bar, notch)
}

fn the_volume_of_the_one_solid(
    model: &Model,
    fused: Result<Vec<SolidKey>, Error>,
    what: &str,
) -> (f64, f64) {
    let fused = fused.unwrap_or_else(|why| panic!("{what}: {why:?}"));
    let [solid] = fused.as_slice() else {
        panic!("{what}: {} solids", fused.len());
    };
    let measured = model
        .solid(*solid)
        .expect("the fuse made the key")
        .measure(
            Deflection::for_scale(model.scale()),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid the fuse made can be measured");
    (measured.volume(), measured.volume_error())
}

/// The bar is `π · 3² · 5 = 45π`. Of it, either notch holds its height `2`
/// times the part of `[2, 4] × [−1, 1]` inside the circle of radius 3, which is
/// `∫ (√(9 − y²) − 2) dy` over `y` from `−1` to `1`, or `√8 + 9 asin(1/3) − 4`.
/// The wall notch is the seam notch a quarter turn round the axis.
fn what_it_holds_by_hand(rule: Rule) -> f64 {
    let the_bar = PI * RADIUS * RADIUS * HEIGHT;
    let the_notch = SIDE * SIDE * SIDE;
    let held = SIDE * (8.0_f64.sqrt() + RADIUS * RADIUS * (1.0_f64 / 3.0).asin() - 4.0);
    match rule {
        Rule::Union => the_bar + the_notch - held,
        Rule::Intersection => held,
        Rule::Difference => the_bar - held,
        other => panic!("no volume worked out by hand for {other:?}"),
    }
}

fn the_pipe(model: &Model, bar: SolidKey) -> Cylinder {
    let mut pipes = model
        .solid(bar)
        .expect("this model made the bar")
        .faces()
        .map(|face| face.surface())
        .filter(|surface| matches!(surface, Surface::Cylinder(_)));
    let (Some(Surface::Cylinder(pipe)), None) = (pipes.next(), pipes.next()) else {
        panic!("a round bar has one wall");
    };
    pipe
}

/// The planes of the notch's four sides: the faces whose normals, moved back
/// by the turn, lie across the bar's axis.
fn the_sides(model: &Model, notch: SolidKey, turn: &Isometry) -> Vec<Plane> {
    let back = turn.inverse();
    let sides: Vec<Plane> = model
        .solid(notch)
        .expect("this model made the notch")
        .faces()
        .map(|face| {
            let Surface::Plane(flat) = face.surface() else {
                panic!("a block is flat");
            };
            flat
        })
        .filter(|flat| back.apply_to_vector(flat.normal()).components()[2].abs() < 0.5)
        .collect();
    assert_eq!(sides.len(), 4, "a block has four sides");
    sides
}

/// What `where_a_corner_lands_after_moving` makes of the bar's seam corner on
/// its bottom cap, `(3, 0, 0)`, with the cap and the wall round it as the bar
/// stores them, and how far it says the corner may be from the box's middle.
fn the_seam_corner_as_the_fuse_reads_it(turn: &Isometry) -> (PredicatePoint, f64) {
    let bottom = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
            .expect("a unit normal"),
    );
    let wall = Surface::Cylinder(Cylinder::new(Axis::z(), RADIUS).expect("a positive radius"));
    let (corner, wobble) =
        where_a_corner_lands_after_moving(Point::new(RADIUS, 0.0, 0.0), [&bottom, &wall], turn)
            .expect("a finite corner and a finite move");
    (corner, wobble.magnitude())
}

/// Checks that `fused` was refused once, on ordering the seam corner's box
/// against another point, and that the other point lies on the bottom cap
/// circle. Hands back that point, moved back by the turn.
fn refused_between_the_seam_corner_and_the_bottom_circle(
    fused: Result<Vec<SolidKey>, Error>,
    turn: &Isometry,
    what: &str,
) -> [f64; 3] {
    let (seam_corner, _) = the_seam_corner_as_the_fuse_reads_it(turn);
    let record = the_one_refusal(fused, what);
    assert_eq!(record.predicate(), ALONG_A_CROSSING, "{what}");
    let [one, other] = record.inputs() else {
        panic!("{what}: {record:?}");
    };
    let on_the_circle = match (*one == seam_corner, *other == seam_corner) {
        (true, false) => other,
        (false, true) => one,
        neither_or_both => panic!("{what}: {neither_or_both:?}, {seam_corner:?} in {record:?}"),
    };
    let [x, y, z] = where_it_was_before_the_turn(*on_the_circle, turn);
    assert!(
        on_the_wall([x, y, z]) && z.abs() < CLOSE,
        "{what}: ({x}, {y}, {z}) is off the bottom circle"
    );
    [x, y, z]
}

fn the_one_refusal(fused: Result<Vec<SolidKey>, Error>, what: &str) -> Record {
    let Err(Error::Undecidable { records }) = fused else {
        panic!("{what}: {fused:?}");
    };
    let [record] = records.records() else {
        panic!("{what}: {records:?}");
    };
    record.clone()
}

/// The middle of a point's box, moved back into the bar's own frame.
fn where_it_was_before_the_turn(point: PredicatePoint, turn: &Isometry) -> [f64; 3] {
    let [x, y, z] = point.enclosure().map(|side| (side.lo() + side.hi()) / 2.0);
    turn.inverse()
        .apply_to_point(Point::new(x, y, z))
        .coordinates()
}

fn on_the_wall([x, y, _]: [f64; 3]) -> bool {
    ((x * x + y * y).sqrt() - RADIUS).abs() < CLOSE
}
