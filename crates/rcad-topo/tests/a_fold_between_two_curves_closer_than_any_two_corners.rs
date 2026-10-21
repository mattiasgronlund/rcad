//! A ball pressed a little way into the side of a round bar, and the piece
//! the two share (row Q2044); and the search for any solid of fewer than two
//! corners whose coarsest mesh folds (row Q2498).
//!
//! # The shape
//!
//! The bar of `a_bar_seamed_up_y_and_a_ball_in_its_side` in
//! `two_curves_of_one_pair_that_meet.rs`: radius 2, up `z` from −1 to 11,
//! turned a quarter round `z` so that its seam runs up `(0, 2, z)`. A ball of
//! radius `r` centred at `(0, 2 + r − reach, 5)` reaches `reach` into its
//! side. What they share is a thin lens of two faces, a patch of the pipe and
//! a cap of the ball, bounded by one closed edge with **one corner**, where the
//! seam crosses the loop's top. The ball can sit on any side of the bar: put on
//! the other three, off the seam, the lens still has one corner.
//!
//! # Why it was refused
//!
//! `Model::fuse` asks how much space each curved piece of its answer encloses
//! before it writes it (`how_much_space_it_encloses` in `fuse/assemble.rs`,
//! which is `how_much_space_a_curved_solid_encloses` in `validate.rs`). The
//! first thing that does is `the_coarsest_mesh_whose_sides_do_not_cross`.
//! Until Q2178, at the coarsest mesh both faces of the lens drew the same side
//! across the loop, and the fuse was refused
//! `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }`. Since Q2178 a
//! curved patch draws no side between two corners of one edge with a corner of
//! the edge between them, the coarsest mesh does not fold, and all six fuses
//! answer.
//!
//! A fold is meant to be asked again at half the room, rung by rung, down to a
//! floor read off the distance between the two closest corners of the solid
//! (`how_close_two_corners_come`). The lens has one corner, so there is no
//! distance, that function says `None`, and the arm that reads it hands the
//! coarsest fold straight back without asking for one finer mesh. That arm is
//! unchanged; this lens no longer reaches it.
//!
//! # For the lens, a gap and not a limit
//!
//! A finer mesh of the same lens does not fold. Measured on `d6433677` with a
//! probe in `the_coarsest_mesh_whose_sides_do_not_cross`, not kept: for the
//! ball of 3 at reach `0.25`, rooms `4`, `2`, `1` and `0.5` fold and every room
//! from `0.25` down to `0.002` meshes; at reach `0.5` the rooms from `0.5` down
//! mesh; for the ball of 6 at reach `0.25`, the rooms from `0.25` down. With
//! the ladder run for a solid with fewer than two corners too, its floor the
//! box's half width over `2¹³`, all six refused fuses answered, validated, and
//! measured within the mesh's error of [`ball_in_the_side_of_the_bar`]. So the
//! answer is there, and only the missing distance kept the fuse from it.
//!
//! # Not the case row Q1232 searches for
//!
//! Q1232 looks for a fold that outlasts every rung down to `closest²/(6ρ)`,
//! where two corners stand far apart and two curves both faces share pass close
//! between them. Here no rung is asked at all, because there is no second
//! corner. A floor read off the gap between the curves, which is what Q1232
//! would put in place of the corners, still needs this arm to start the
//! ladder when the corners give no distance.
//!
//! # What answers
//!
//! The union and both differences of the same shallow press answer, and so
//! does the intersection at reach `1`, whose coarsest mesh does not fold.
//! Thin pieces of the same kind that designers make answer too: a lens of two
//! balls a quarter or a half thick, and a ball's cap as high past a block's
//! face, each two faces round one loop, and neither folds at its coarsest
//! mesh. So no shape designers make on purpose was known to be refused this
//! way, and the refusal was class C.
//!
//! # No shape is known to reach the arm (row Q2498)
//!
//! The arm `let Some(closest) = how_close_two_corners_come(view) else { return
//! Err(finest_refusal) }` in `the_coarsest_mesh_whose_sides_do_not_cross` is
//! still a guard with no shape known behind it. Q2498 looked for a solid of
//! fewer than two corner places whose coarsest mesh folds, and found none. Two
//! tests here keep what it found:
//!
//! - [`no_solid_of_fewer_than_two_corners_these_intersections_leave_folds_at_its_coarsest_mesh`]
//!   intersects four families, either solid first, and meshes at the coarsest
//!   deflection each solid of the answer with fewer than two corner places:
//!   108 from a ball in the bar's side, 32 from a bar across the bar, 24 from a
//!   cap beside a ball's pole and 108 from two balls. None folds.
//! - [`a_whole_ring_in_one_face_with_one_corner_meshes_at_the_coarsest_deflection`]
//!   builds a whole ring as one face, which meets itself along both its edges
//!   at its one corner, in five sizes and four orders. None folds.
//!
//! Tried on Q2498's branch with probes that were not kept:
//!
//! - a wider sweep of intersections: balls of 8 radii at 9 reaches and 10
//!   turns in the bar's side gave 520 answers of fewer than two corner places;
//!   bars across the bar gave 89, and lenses of two balls 144. None folds.
//! - a slab of a block across a ball of 3: 52 of 132 are refused
//!   `CurvedFaceWithAHoleGoesTooFarRound`, each one whose answer is a band of
//!   the ball's skin holding neither pole, so no mesh of it is asked; the 80
//!   that answer hold two corner places or more.
//! - a block whose face leaves a cap of a ball of 3 that misses its pole by a
//!   little: 212 answers of one corner place, none folding.
//! - a ring of 5 and 3 cut into two faces by its outer equator, the circle
//!   round its tube in `x = 0` and a Villarceau circle, all three through the
//!   one corner `(0, 8, 0)`: `SolidBuilder::build` refuses it all 16 ways
//!   round, so no mesh of it is ever asked.
//!
//! # Why the arm may hold nothing, a reading nobody has run
//!
//! A fold's side has both ends on edges the two folding faces share. Since
//! Q2178 a curved patch draws no side between two points along one edge with
//! a point inside that edge between them (`corners_cut_off_by_one_side` in
//! `rcad-mesh`'s `triangulate.rs`), so with one corner place the two ends lie
//! on two different closed edges through that place, and both faces hold both
//! edges. On two different surfaces that needs the surfaces to touch at that
//! place: where two smooth surfaces cross, they meet in one smooth curve there,
//! and two closed edges through the place would run along each other. On one
//! surface it needs a ring, not a ball: two loops through one point of a ball
//! cut it into three pieces, and only the one between them holds both. A ring
//! needs three loops through one point, its outer equator, the circle round
//! its tube and a Villarceau circle, which is the shape above that will not
//! build.
//!
//! The fuses run certified, in a model of 100 mm.

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey};

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::f64::consts::PI;

/// The three shallow presses that were refused until Q2178: a ball of 3 at
/// reach `0.25` and `0.5`, and a ball of 6 at reach `0.25`.
const SHALLOW: [(f64, f64); 3] = [(3.0, 0.25), (3.0, 0.5), (6.0, 0.25)];

/// The intersection of the bar and a ball of 3 pressed `0.25` or `0.5` into
/// it, or of 6 pressed `0.25`, either solid first, is two faces and
/// [`ball_in_the_side_of_the_bar`]. Until Q2178 each was refused
/// `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }` for the fold at
/// its coarsest mesh.
#[test]
fn what_a_ball_pressed_a_quarter_or_a_half_into_a_bar_shares_with_it_answers() {
    for (radius, reach) in SHALLOW {
        for bar_first in [true, false] {
            let mut model = a_model();
            let said = fuse(&mut model, radius, reach, Rule::Intersection, bar_first);
            answers(
                &mut model,
                said,
                ball_in_the_side_of_the_bar(radius, reach, 1000),
                &[2],
                &format!("ball of {radius} at reach {reach}, bar first: {bar_first}"),
            );
        }
    }
}

/// Pressed `1` in, the ball of 3 and the ball of 6 share with the bar a lens
/// whose coarsest mesh does not fold, and the intersection answers with it.
#[test]
fn what_a_ball_pressed_one_into_a_bar_shares_with_it_answers() {
    for radius in [3.0, 6.0] {
        for bar_first in [true, false] {
            let mut model = a_model();
            let said = fuse(&mut model, radius, 1.0, Rule::Intersection, bar_first);
            answers(
                &mut model,
                said,
                ball_in_the_side_of_the_bar(radius, 1.0, 1000),
                &[2],
                &format!("ball of {radius} at reach 1, bar first: {bar_first}"),
            );
        }
    }
}

/// The union and both differences of each shallow press answer. The bar less
/// the ball keeps its pipe, its two caps and the dent; the ball less the bar
/// is its skin and the pipe's patch.
#[test]
fn the_other_four_fuses_of_a_shallow_press_answer() {
    for (radius, reach) in SHALLOW {
        let bar = 48.0 * PI;
        let ball = 4.0 * PI / 3.0 * radius.powi(3);
        let shared = ball_in_the_side_of_the_bar(radius, reach, 1000);
        for (rule, bar_first, volume, faces) in [
            (Rule::Union, true, bar + ball - shared, 4),
            (Rule::Union, false, bar + ball - shared, 4),
            (Rule::Difference, true, bar - shared, 4),
            (Rule::Difference, false, ball - shared, 2),
        ] {
            let mut model = a_model();
            let said = fuse(&mut model, radius, reach, rule, bar_first);
            answers(
                &mut model,
                said,
                volume,
                &[faces],
                &format!(
                    "{rule:?} of the ball of {radius} at reach {reach}, bar first: {bar_first}"
                ),
            );
        }
    }
}

/// Two balls of 3 whose centres are `6 − thick` apart, along each axis, share
/// a lens `thick` thick, `π(4r + d)(2r − d)²/12` for `d` the distance between
/// the centres. It answers, either ball first.
#[test]
fn a_lens_of_two_balls_a_quarter_or_a_half_thick_answers() {
    for thick in [0.25, 0.5] {
        let apart = 6.0 - thick;
        let lens = PI * (12.0 + apart) * thick * thick / 12.0;
        for axis in 0..3 {
            for first_first in [true, false] {
                let mut model = a_model();
                let mut at = [0.0; 3];
                at[axis] = apart;
                let one = model.add_ball(3.0, &Isometry::identity()).expect("a ball");
                let other = model.add_ball(3.0, &moved_to(at)).expect("a ball");
                let said = intersect(&mut model, [one, other], first_first);
                answers(
                    &mut model,
                    said,
                    lens,
                    &[2],
                    &format!(
                        "two balls {thick} thick along axis {axis}, first first: {first_first}"
                    ),
                );
            }
        }
    }
}

/// A ball of 3 at the origin and a block whose face is `high` inside the ball,
/// across each axis but `−z`, share the ball's cap, `πh²(3r − h)/3`. It
/// answers, either solid first.
#[test]
fn a_balls_cap_a_quarter_or_a_half_high_past_a_blocks_face_answers() {
    for high in [0.25, 0.5] {
        let cap = PI * high * high * (9.0 - high) / 3.0;
        for (axis, beyond) in [(0, true), (0, false), (1, true), (1, false), (2, true)] {
            for ball_first in [true, false] {
                let mut model = a_model();
                let (mut from, mut to) = ([-5.0; 3], [5.0; 3]);
                if beyond {
                    from[axis] = 3.0 - high;
                } else {
                    to[axis] = high - 3.0;
                }
                let ball = model.add_ball(3.0, &Isometry::identity()).expect("a ball");
                let block = a_block(&mut model, from, to);
                let said = intersect(&mut model, [ball, block], ball_first);
                answers(
                    &mut model,
                    said,
                    cap,
                    &[2],
                    &format!(
                        "cap {high} high past the face across axis {axis}, on the far side: \
                         {beyond}, ball first: {ball_first}"
                    ),
                );
            }
        }
    }
}

/// [`ball_in_the_side_of_the_bar`] at the 1000 steps the fuses are held to is
/// within a millionth of the rule at 100 000, for every press this file makes.
#[test]
fn the_ball_in_the_side_of_the_bar_is_within_a_millionth_of_a_rule_a_hundred_times_finer() {
    for (radius, reach) in [(3.0, 0.25), (3.0, 0.5), (3.0, 1.0), (6.0, 0.25), (6.0, 1.0)] {
        let (coarse, fine) = (
            ball_in_the_side_of_the_bar(radius, reach, 1000),
            ball_in_the_side_of_the_bar(radius, reach, 100_000),
        );
        assert!(
            (coarse - fine).abs() < 1e-6,
            "ball of {radius} at reach {reach}: {coarse} at 1000 steps, {fine} at 100 000"
        );
    }
}

/// A whole ring of `ring` and `tube` about `z` built as one face: its outer
/// equator and the circle round its tube at `y = 0` are its two edges, both
/// closed at the one corner `(ring + tube, 0, 0)`, and its wire runs each of
/// them both ways, in each of the four orders. It is a solid with one corner
/// whose one face meets itself along both edges, and it meshes at the coarsest
/// deflection.
#[test]
fn a_whole_ring_in_one_face_with_one_corner_meshes_at_the_coarsest_deflection() {
    for (ring, tube) in [(4.0, 1.0), (20.0, 5.0), (3.0, 2.0), (10.0, 1.0), (2.0, 1.0)] {
        for order in 0..4 {
            let what = format!("the ring of {ring} and {tube} in order {order}");
            let skin = Torus::new(Axis::z(), ring, tube).expect("a ring");
            let mut builder = SolidBuilder::new();
            let corner = builder
                .add_vertex(Point::new(ring + tube, 0.0, 0.0))
                .expect("a corner");
            let level = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
                .expect("a plane");
            let across = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("a plane");
            let equator = builder
                .add_edge(
                    corner,
                    corner,
                    EdgeRun::RightRoundWhereItCrosses {
                        and: Surface::Plane(level),
                    },
                )
                .expect("the outer equator");
            let round_the_tube = builder
                .add_edge(
                    corner,
                    corner,
                    EdgeRun::RightRoundWhereItCrosses {
                        and: Surface::Plane(across),
                    },
                )
                .expect("the circle round the tube");
            let (f, r) = (Orientation::Forward, Orientation::Reversed);
            let walks = match order {
                0 => [
                    (round_the_tube, f),
                    (equator, f),
                    (round_the_tube, r),
                    (equator, r),
                ],
                1 => [
                    (round_the_tube, f),
                    (equator, r),
                    (round_the_tube, r),
                    (equator, f),
                ],
                2 => [
                    (equator, f),
                    (round_the_tube, f),
                    (equator, r),
                    (round_the_tube, r),
                ],
                _ => [
                    (equator, f),
                    (round_the_tube, r),
                    (equator, r),
                    (round_the_tube, f),
                ],
            };
            let steps: Vec<DraftStep> = walks
                .into_iter()
                .map(|(edge, turn)| {
                    DraftStep::Along(builder.add_coedge(edge, turn).expect("a coedge"))
                })
                .collect();
            let wire = builder.add_wire(&steps).expect("a wire");
            let face = builder
                .add_face(Surface::Torus(skin), &[wire])
                .expect("a face");
            builder
                .add_shell(&[(face, Orientation::Forward)])
                .expect("a shell");
            let mut model = a_model();
            let solid = builder
                .build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())
                .unwrap_or_else(|why| panic!("{what}: not built: {why:?}"));
            assert_eq!(corner_places(&model, solid), 1, "{what}: corners");
            coarsest_mesh(&model, solid, &what);
        }
    }
}

/// Every intersection of four families answers, and each solid of fewer than
/// two corners among the answers meshes at the coarsest deflection, so none
/// reaches the arm of `the_coarsest_mesh_whose_sides_do_not_cross` that hands
/// a fold or a crossing back without asking a finer mesh when the corners give
/// no distance. The count of such solids, family by family, is the proof that
/// the sweep reached that arm's shapes at all:
///
/// - a ball of `1.5`, `3` or `6` pressed `0.05`, `0.5` or `1` into the side of
///   the bar, at six turns round it;
/// - a bar of `0.5`, `1` or `2` across that bar, its axis `2 + radius − reach`
///   off the bar's, square to it and leaning `0.7`;
/// - two balls, of `3` and of `1`, `3` or `9`, sharing a lens `0.05`, `0.5` or
///   `1.5` thick along six directions;
/// - a ball of `3` and a block whose face leans `0.6` or `1.5` from `z` and
///   leaves a cap that misses the north pole by `0.05` or `0.001` of a radian,
///   at three turns round `z`.
#[test]
fn no_solid_of_fewer_than_two_corners_these_intersections_leave_folds_at_its_coarsest_mesh() {
    let mut counted = BTreeMap::new();
    let seam_up_y = moved_to([0.0, 0.0, -1.0])
        .compose(&Isometry::quarter_turns(&Axis::z(), 1).expect("a turn about z"));
    for radius in [1.5, 3.0, 6.0] {
        for reach in [0.05, 0.5, 1.0] {
            for around in [0.0, 0.9, 2.2, PI, 4.0, 5.3] {
                count_what_has_fewer_than_two_corners(
                    &mut counted,
                    "a ball in the bar's side",
                    &format!("a ball of {radius} pressed {reach} into the bar, {around} round it"),
                    |model| {
                        let bar = model.add_round_bar(2.0, 12.0, &seam_up_y).expect("a bar");
                        let out = 2.0 + radius - reach;
                        let ball = model
                            .add_ball(
                                radius,
                                &moved_to([-out * f64::sin(around), out * f64::cos(around), 5.0]),
                            )
                            .expect("a ball");
                        [bar, ball]
                    },
                );
            }
        }
    }
    let the_y_axis = Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("an axis");
    for radius in [0.5, 1.0, 2.0] {
        for reach in [0.1, 0.6, 1.5] {
            if reach >= 2.0 * radius {
                continue;
            }
            for lean in [0.0, 0.7] {
                count_what_has_fewer_than_two_corners(
                    &mut counted,
                    "a bar across the bar",
                    &format!("a bar of {radius} across the bar, {reach} into it, leaning {lean}"),
                    |model| {
                        let bar = model.add_round_bar(2.0, 12.0, &seam_up_y).expect("a bar");
                        let across = moved_to([0.0, 2.0 + radius - reach, 5.0])
                            .compose(
                                &Isometry::rotation(&the_y_axis, PI / 2.0 + lean).expect("a turn"),
                            )
                            .compose(&moved_to([0.0, 0.0, -10.0]));
                        let other = model.add_round_bar(radius, 20.0, &across).expect("a bar");
                        [bar, other]
                    },
                );
            }
        }
    }
    for other in [1.0, 3.0, 9.0] {
        for thick in [0.05, 0.5, 1.5] {
            for along in [
                [1.0, 0.0, 0.0],
                [0.6, 0.8, 0.0],
                [0.48, 0.6, 0.64],
                [-0.36, 0.48, 0.8],
                [0.0, -0.6, 0.8],
                [-0.8, -0.36, 0.48],
            ] {
                count_what_has_fewer_than_two_corners(
                    &mut counted,
                    "two balls",
                    &format!("balls of 3 and {other} sharing a lens {thick} thick along {along:?}"),
                    |model| {
                        let apart = 3.0 + other - thick;
                        let one = model.add_ball(3.0, &Isometry::identity()).expect("a ball");
                        let two = model
                            .add_ball(other, &moved_to(along.map(|x| x * apart)))
                            .expect("a ball");
                        [one, two]
                    },
                );
            }
        }
    }
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    for lean in [0.6, 1.5] {
        for short in [0.05, 0.001] {
            for around in [0.0, 2.5, 4.0] {
                count_what_has_fewer_than_two_corners(
                    &mut counted,
                    "a cap beside a ball's pole",
                    &format!("a cap leaning {lean}, {short} short of the pole, {around} round z"),
                    |model| {
                        let placed = Isometry::rotation(&Axis::z(), around)
                            .expect("a turn")
                            .compose(&Isometry::rotation(&the_y_axis, lean).expect("a turn"))
                            .compose(&moved_to([-5.0, -5.0, 3.0 * f64::cos(lean - short)]));
                        let ball = model.add_ball(3.0, &Isometry::identity()).expect("a ball");
                        let block = model.add_extrusion(&square, 5.0, &placed).expect("a block");
                        [ball, block]
                    },
                );
            }
        }
    }
    assert_eq!(
        counted,
        BTreeMap::from([
            ("a ball in the bar's side", 108),
            ("a bar across the bar", 32),
            ("a cap beside a ball's pole", 24),
            ("two balls", 108),
        ]),
        "solids of fewer than two corners"
    );
}

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// The fuse `rule` of the bar and a ball of `radius` pressed `reach` into it,
/// the bar first or not.
fn fuse(
    model: &mut Model,
    radius: f64,
    reach: f64,
    rule: Rule,
    bar_first: bool,
) -> rcad_topo::Result<Vec<SolidKey>> {
    let seam_up_y = moved_to([0.0, 0.0, -1.0])
        .compose(&Isometry::quarter_turns(&Axis::z(), 1).expect("a turn about z"));
    let bar = model.add_round_bar(2.0, 12.0, &seam_up_y).expect("a bar");
    let ball = model
        .add_ball(radius, &moved_to([0.0, 2.0 + radius - reach, 5.0]))
        .expect("a ball");
    let (arguments, tools) = if bar_first {
        ([bar], [ball])
    } else {
        ([ball], [bar])
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

fn intersect(
    model: &mut Model,
    [first, second]: [SolidKey; 2],
    first_first: bool,
) -> rcad_topo::Result<Vec<SolidKey>> {
    let (arguments, tools) = if first_first {
        ([first], [second])
    } else {
        ([second], [first])
    };
    model
        .fuse(
            &arguments,
            &tools,
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|answer| answer.solids().to_vec())
}

fn moved_to(at: [f64; 3]) -> Isometry {
    Isometry::translation(Vector::new(at[0], at[1], at[2])).expect("a finite move")
}

fn a_block(model: &mut Model, from: [f64; 3], to: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [to[0] - from[0], 0.0],
        [to[0] - from[0], to[1] - from[1]],
        [0.0, to[1] - from[1]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle is a profile");
    model
        .add_extrusion(&rectangle, to[2] - from[2], &moved_to(from))
        .expect("a rectangle extrudes")
}

/// Assert that `said` is one solid for each of `faces`, each with that many
/// faces, that the model validates, and that together they measure `volume`
/// to within the mesh's own error. `what` names the fuse in every message.
fn answers(
    model: &mut Model,
    said: rcad_topo::Result<Vec<SolidKey>>,
    volume: f64,
    faces: &[usize],
    what: &str,
) {
    let policy = Policy::certified();
    let solids = said.unwrap_or_else(|why| panic!("{what}: refused: {why:?}"));
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: the answer does not validate: {why:?}"));
    let mut counted = Vec::with_capacity(solids.len());
    let (mut measured, mut error) = (0.0, 0.0);
    for solid in solids {
        let view = model.solid(solid).expect("a solid the fuse made");
        counted.push(view.faces().count());
        let measure = view
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("the answer measures");
        measured += measure.volume();
        error += measure.volume_error();
    }
    assert_eq!(counted, faces, "{what}: faces of each solid");
    assert!(
        (measured - volume).abs() <= error,
        "{what}: measured {measured} against {volume}, and the mesh may miss by {error}"
    );
}

/// What a ball of `radius` centred at `(0, 2 + radius − reach, 5)` shares with
/// the bar of radius 2 up `z`, by a midpoint rule of `steps` steps across `x`.
///
/// At each `x` the ball's section is a disc of radius `a = √(radius² − x²)`
/// about `y = c`, `c = 2 + radius − reach`, and the bar keeps the part of it
/// at `y ≤ √(4 − x²)`: the segment `a²(π/2 + asin(s/a)) + s√(a² − s²)`, for
/// `s = √(4 − x²) − c` how far that line is past the disc's centre. The rule
/// runs between the two `x` where the segment closes, `√(4 − x²) + a = c`,
/// at which `a = (c + (radius² − 4)/c)/2`.
fn ball_in_the_side_of_the_bar(radius: f64, reach: f64, steps: u32) -> f64 {
    let centre = 2.0 + radius - reach;
    let at_the_edge = (centre + (radius * radius - 4.0) / centre) / 2.0;
    let edge = (radius * radius - at_the_edge * at_the_edge).sqrt();
    let step = 2.0 * edge / f64::from(steps);
    let mut sum = 0.0;
    for i in 0..steps {
        let x = -edge + step * (f64::from(i) + 0.5);
        let squared = radius * radius - x * x;
        let past = (4.0 - x * x).sqrt() - centre;
        sum += squared * (PI / 2.0 + (past / squared.sqrt()).asin())
            + past * (squared - past * past).sqrt();
    }
    sum * step
}

/// Intersect the two solids `make` builds, either one first, and count under
/// `family` each solid of the answer with fewer than two corners, after meshing
/// it at the coarsest deflection. Panics, naming `what`, when the fuse or the
/// mesh is refused.
fn count_what_has_fewer_than_two_corners(
    counted: &mut BTreeMap<&'static str, usize>,
    family: &'static str,
    what: &str,
    make: impl Fn(&mut Model) -> [SolidKey; 2],
) {
    for first_first in [true, false] {
        let mut model = a_model();
        let pair = make(&mut model);
        let solids = intersect(&mut model, pair, first_first)
            .unwrap_or_else(|why| panic!("{what}, first first: {first_first}: refused {why:?}"));
        for solid in solids {
            if corner_places(&model, solid) < 2 {
                coarsest_mesh(&model, solid, what);
                *counted.entry(family).or_insert(0) += 1;
            }
        }
    }
}

/// How many places the corners of `solid` stand at, two corners in one place
/// counted once, as `how_close_two_corners_come` in `validate.rs` counts them.
fn corner_places(model: &Model, solid: SolidKey) -> usize {
    let mut corners: Vec<[f64; 3]> = model
        .solid(solid)
        .expect("a solid of the model")
        .faces()
        .flat_map(|face| {
            face.wires()
                .flat_map(|wire| wire.points())
                .map(|point| point.coordinates())
                .collect::<Vec<_>>()
        })
        .collect();
    corners.sort_by(|one, other| {
        (0..3)
            .map(|axis| one[axis].total_cmp(&other[axis]))
            .find(|order| order.is_ne())
            .unwrap_or(Ordering::Equal)
    });
    corners.dedup();
    corners.len()
}

/// Mesh `solid` at the coarsest deflection there is, the one
/// `the_coarsest_mesh_whose_sides_do_not_cross` asks first, and panic, naming
/// `what`, when that is refused.
fn coarsest_mesh(model: &Model, solid: SolidKey, what: &str) {
    model
        .solid(solid)
        .expect("a solid of the model")
        .mesh(
            Deflection::of(f64::MAX).expect("the largest f64 is a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the coarsest mesh is refused: {why:?}"));
}
