//! A ball pressed a little way into the side of a round bar, and the piece
//! the two share (row Q2044); the search for any solid of fewer than two
//! corners whose coarsest mesh folds (row Q2498); what a thin bar resting
//! inside a thick one shares, a solid of one corner whose coarsest mesh
//! crosses and is handed back at once (row Q2561); and a ball less a bar whose
//! two curves come closer than its two corners (rows Q2641 and Q1232).
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
//! A fold is asked again at half the room, rung by rung, down to a floor. Until
//! Q2641 that floor was read off the distance between the two closest corners
//! of the solid (`how_close_two_corners_come`). The lens has one corner, so
//! there is no distance, that function says `None`, and the arm that read it
//! handed the coarsest fold straight back without asking for one finer mesh.
//! Since Q2641 the floor is read off the shorter of that distance and the
//! shortest side two patches drew on any rung, so a fold with no corner
//! distance is asked down to the floor its own side gives, and only a crossing
//! is still handed back at once. Since Q2178 this lens does not fold at its
//! coarsest mesh, so no rung is asked for it at all.
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
//! # Not the case row Q1232 searched for
//!
//! Q1232 looked for a fold that outlasts every rung down to `closest²/(6ρ)`,
//! where two corners stand far apart and two curves both faces share pass close
//! between them. Here no rung was asked at all, because there is no second
//! corner. Q1232's case is the last one below, two curves closer than any two
//! corners.
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
//! # The arm for a solid of fewer than two corners (rows Q2498 and Q2561)
//!
//! Until Q2641, the arm `let Some(closest) = how_close_two_corners_come(view)
//! else { return Err(finest_refusal) }` in
//! `the_coarsest_mesh_whose_sides_do_not_cross` handed a fold or a crossing
//! back without asking a finer mesh when the corners gave no distance, a guard
//! with no shape known behind it. Q2498 looked for a solid of fewer than two
//! corner places whose coarsest mesh folds, and found none. Q2641 took the arm
//! out for a fold: a fold on such a solid now runs the ladder down to the floor
//! its own folded side gives, and only a crossing there is still handed back at
//! once. Q2561 found a solid of one corner place that reaches that crossing
//! arm, and again none that folds. Three tests here keep what the two rows
//! found, and a fold among them would now be asked at finer rungs rather than
//! handed back:
//!
//! - [`no_solid_of_fewer_than_two_corners_these_intersections_leave_folds_at_its_coarsest_mesh`]
//!   intersects four families, either solid first, and meshes at the coarsest
//!   deflection each solid of the answer with fewer than two corner places:
//!   108 from a ball in the bar's side, 32 from a bar across the bar, 24 from a
//!   cap beside a ball's pole and 108 from two balls. None folds.
//! - [`a_whole_ring_in_one_face_with_one_corner_meshes_at_the_coarsest_deflection`]
//!   builds a whole ring as one face, which meets itself along both its edges
//!   at its one corner, in five sizes and four orders. None folds.
//! - [`what_a_thin_bar_resting_inside_a_thick_one_shares_is_handed_back_at_once_round_one_corner`]
//!   builds by hand what a thin bar resting inside a thick one shares, round
//!   its one corner where the two pipes touch. It does not fold: its coarsest
//!   mesh crosses, and that is handed back at once.
//!
//! Tried on Q2498's branch with probes that were not kept:
//!
//! - a wider sweep of intersections: balls of 8 radii at 9 reaches and 10
//!   turns in the bar's side gave 520 answers of fewer than two corner places;
//!   bars across the bar gave 89, and lenses of two balls 144. None folds.
//! - a slab of a block across a ball of 3: 52 of 132 were refused
//!   `CurvedFaceWithAHoleGoesTooFarRound`, each one whose answer is a band of
//!   the ball's skin holding neither pole, so no mesh of it was asked; the 80
//!   that answered held two corner places or more. Since Q2556 such bands
//!   answer. Run again on Q2641's branch, 432 slabs along `x`, `y` and `z`,
//!   their near face at twelve places from `−2.9` to `2.5` and `0.3` or `1`
//!   thick, fused with the ball by each rule, either solid first, all answer
//!   and validate. Of the 144 intersections, 132 hold two corner places; the
//!   12 of one are the caps the slabs from `2` and `2.5` cut off the ball's
//!   end along each axis, and each meshes at the coarsest deflection.
//! - a block whose face leaves a cap of a ball of 3 that misses its pole by a
//!   little: 212 answers of one corner place, none folding.
//! - a ring of 5 and 3 cut into two faces by its outer equator, the circle
//!   round its tube in `x = 0` and a Villarceau circle, all three through the
//!   one corner `(0, 8, 0)`: `SolidBuilder::build` refuses it all 16 ways
//!   round, so no mesh of it is ever asked.
//!
//! # Why no such fold may exist, and the one case run (row Q2561)
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
//! build. That much is a reading, and not a run.
//!
//! Q2561 built the case of two surfaces, and it does not fold. A bar of 1
//! along `x` at `y = 1` rests on the inside of the bar of 2 up `z`, the two
//! pipes touching at `(0, 2, 0)`, and what they share is bounded by two loops
//! that meet there, one each side of `x = 0`. The fuse refuses that pair where
//! the loops meet (row Q2637), so it is built by hand, of two faces: the thin
//! pipe's band between the loops, and the thick pipe's two lobes, one inside
//! each loop, as one face walked round both, since a loop `intersect` names
//! closes no wire of its own. Both faces hold both closed edges through the
//! one corner. At the coarsest mesh the face of both lobes is refused
//! `AMeshCoveredTheWrongSideOfALoop`, a crossing and not a fold, and with one
//! corner place `the_coarsest_mesh_whose_sides_do_not_cross` hands that back at
//! once. So the arm Q2641 left for a crossing is reached through the public
//! API and cannot become an assertion.
//!
//! Handing it back loses no answer for this solid. Measured on `1df3fb5f` with
//! a probe in `the_coarsest_mesh_whose_sides_do_not_cross`, not kept, at every
//! room from `2`, half the box's width, down to `2⁻¹²`: the solid meshes at
//! none. The face of both lobes is refused `AMeshCoveredTheWrongSideOfALoop`
//! at `2` and at every room from `2⁻⁵` down to `2⁻¹¹`, and
//! `CurvedFaceWithAHoleGoesTooFarRound` at `2⁻⁴`. The band is refused
//! `ACurvedFaceCannotBeCutSmallEnoughToMesh` at `1`, `0.5` and `0.125`, a
//! boundary that crosses itself at `0.25`, and
//! `CurvedFaceWithAHoleGoesTooFarRound` at `2⁻¹²`. Given a floor for the
//! crossing, the ladder would stop at room `1`, on the band's refusal, which is
//! about something else and is handed back as it is. Each band way round gives
//! the same rooms. What stops this solid is the mesher at every rung, and not
//! the arm.
//!
//! # Two curves closer than any two corners (rows Q2641 and Q1232)
//!
//! A ball of 3 less a bar of 2.9 bored `0.08` off its centre is one solid of
//! two faces, the ball's band and the bore's wall, between two loops. Its two
//! corners stand `2.047` apart, where the loops are furthest apart, and both
//! faces draw the folded side where the loops come closest, about `0.70` long.
//! With the floor read off the corners, `0.2328`, the ladder stopped at room
//! `0.375`, and the fuse was refused
//! `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }`; the fold clears
//! at `0.046875`. Q1232's ring of a ball of `221/64` less a bar of `220/64` on
//! its axis, built with its two corners apart round the axis, was refused the
//! same way, where the same ring with both corners on one meridian builds. A
//! gap and not a limit: run past the corners' floor, the same ladder meshes
//! them all.
//!
//! Since Q2641 the floor is read off the shorter of the two closest corners and
//! the shortest side two patches drew on any rung, and both answer:
//! [`a_ball_with_a_hole_bored_a_little_off_its_centre_answers`] and
//! [`a_ring_of_a_ball_less_a_bar_on_its_axis_builds_wherever_its_two_corners_stand`].
//!
//! The fuses run certified, in a model of 100 mm.

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Sphere,
    Surface, SurfaceMeet, Torus, Vector,
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
/// asks `the_coarsest_mesh_whose_sides_do_not_cross` for a finer mesh where the
/// corners give no distance. Until Q2641 a fold there was handed back without
/// one; now a fold runs the ladder on its own folded side, and only a crossing
/// is handed back. The count of such solids, family by family, is the proof
/// that the sweep reached solids of fewer than two corners at all:
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

/// A ball of `radius` less a round bar of `bar` up `z` whose axis is `off` from
/// the ball's centre along `x`, the ball turned each quarter round `z`, is one
/// solid of two faces, the ball's band and the bore's wall, and measures the
/// ball less [`what_a_bar_bores_out_of_a_ball`]. Until Q2641 every bore here
/// but the last, a bar of 2 half off, was refused
/// `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }`.
#[test]
fn a_ball_with_a_hole_bored_a_little_off_its_centre_answers() {
    for (radius, bar, off) in [
        (3.0, 2.9, -0.08),
        (3.0, 2.9, 0.08),
        (3.0, 2.9, -0.05),
        (3.0, 2.8, 0.15),
        (3.0, 2.8, -0.15),
        (3.0, 2.5, -0.3),
        (10.0, 8.0, -1.5),
        (3.0, 2.0, -0.5),
    ] {
        let left = 4.0 * PI / 3.0 * f64::powi(radius, 3)
            - what_a_bar_bores_out_of_a_ball(radius, bar, off, 1000);
        for turn in 0..4 {
            let what = format!("a ball of {radius} less a bar of {bar} {off} off, turned {turn}");
            let mut model = a_model();
            let ball = model
                .add_ball(
                    radius,
                    &Isometry::quarter_turns(&Axis::z(), turn).expect("a turn about z"),
                )
                .expect("a ball");
            let bar = model
                .add_round_bar(bar, 4.0 * radius, &moved_to([off, 0.0, -2.0 * radius]))
                .expect("a bar");
            let said = model
                .fuse(
                    &[ball],
                    &[bar],
                    Rule::Difference,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .map(|answer| answer.solids().to_vec());
            answers(&mut model, said, left, &[2], &what);
        }
    }
}

/// [`what_a_bar_bores_out_of_a_ball`] with the bar on the ball's axis is the
/// ball less the napkin ring it leaves, `4π/3 · (r³ − (r² − a²)^{3/2})`, and
/// off the axis it agrees with the rule at four times the steps, each to within
/// a millionth of what the bar takes out, for every bore this file makes. The
/// mesh at `0.01` the fuses are measured at may miss by a tenth or more, far
/// more than that.
#[test]
fn what_a_bar_bores_out_of_a_ball_is_within_a_millionth_of_its_closed_form_and_a_finer_rule() {
    for (radius, bar) in [(3.0, 2.9), (10.0, 8.0)] {
        let rule = what_a_bar_bores_out_of_a_ball(radius, bar, 0.0, 1000);
        let closed = 4.0 * PI / 3.0 * (radius.powi(3) - (radius * radius - bar * bar).powf(1.5));
        assert!(
            (rule - closed).abs() < 1e-6 * closed,
            "a bar of {bar} on the axis of a ball of {radius}: {rule} by the rule, {closed} closed"
        );
    }
    for (radius, bar, off) in [
        (3.0, 2.9, -0.08),
        (3.0, 2.8, 0.15),
        (3.0, 2.5, -0.3),
        (10.0, 8.0, -1.5),
        (3.0, 2.0, -0.5),
    ] {
        let (coarse, fine) = (
            what_a_bar_bores_out_of_a_ball(radius, bar, off, 1000),
            what_a_bar_bores_out_of_a_ball(radius, bar, off, 4000),
        );
        assert!(
            (coarse - fine).abs() < 1e-6 * fine,
            "a bar of {bar} {off} off in a ball of {radius}: {coarse} at 1000 steps, {fine} at 4000"
        );
    }
}

/// Q1232's ring: a ball of `221/64` less a bar of `220/64` on its axis, built
/// as two faces, the ball's band and the bore's wall, between the two circles
/// at `z = ±21/64` where they meet, each circle one closed edge with its one
/// corner at a quarter turn round `z`. It builds, validates, and measures the
/// napkin ring `4π/3 · (21/64)³`, wherever the two corners stand.
///
/// With both corners on one meridian it always did. With them apart round the
/// axis the two closest corners are `0.86` or more apart while the two circles
/// are `42/64` apart everywhere, and until Q2641 the floor a fold was asked
/// down to was read off the corners: the ball's band and the bore's wall drew
/// the same side between the circles at every rung down to it, and the build
/// was refused `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 4, used: 4 }`.
#[test]
fn a_ring_of_a_ball_less_a_bar_on_its_axis_builds_wherever_its_two_corners_stand() {
    for (top, bottom) in [(0, 0), (0, 2), (0, 1), (1, 3)] {
        let what = format!("the ring with its top corner at {top} and its bottom one at {bottom}");
        let mut model = a_model();
        let solid = a_ring_of_a_ball_less_a_bar(&mut model, top, bottom).map(|solid| vec![solid]);
        answers(
            &mut model,
            solid,
            4.0 * PI / 3.0 * (21.0_f64 / 64.0).powi(3),
            &[2],
            &what,
        );
    }
}

/// What a bar of radius 1 along `x` at `y = 1` shares with a bar of radius 2 up
/// `z`, built round its one corner `(0, 2, 0)`, where the thin pipe rests on the
/// inside of the thick one and its two loops meet (row Q2561). The fuse refuses
/// that pair where the loops meet
/// (`a_thin_bar_resting_on_the_inside_of_a_thick_one_is_refused_where_its_two_loops_meet`,
/// row Q2637), so it is built by hand.
///
/// Each loop is one closed edge from the corner back to it, as `intersect`
/// names it, and such an edge closes no wire of its own: a lobe of the thick
/// pipe inside one loop is refused `ADraftWireIsTooShort`. So the thick pipe's
/// two lobes are one face, walked round both loops, and the thin pipe's band
/// between the loops is the other. Both faces hold both closed edges through
/// the one corner, which is what a fold there needs.
///
/// It does not fold. Whichever way the band walks its loops, the build is
/// refused `AMeshCoveredTheWrongSideOfALoop`, for the face of both lobes at the
/// coarsest mesh. `the_coarsest_mesh_whose_sides_do_not_cross` asks that again
/// the way it asks a crossing, and with one corner place there is no distance
/// to read a crossing's floor off, so it hands the refusal back at once. That
/// arm is reached through the public API, so it cannot become an assertion.
#[test]
fn what_a_thin_bar_resting_inside_a_thick_one_shares_is_handed_back_at_once_round_one_corner() {
    let thick = Surface::Cylinder(Cylinder::new(Axis::z(), 2.0).expect("a pipe"));
    let along_x =
        Axis::new(Point::new(0.0, 1.0, 0.0), Vector::new(1.0, 0.0, 0.0)).expect("an axis");
    let thin = Surface::Cylinder(Cylinder::new(along_x, 1.0).expect("a pipe"));
    let Ok(SurfaceMeet::Curves(loops)) = rcad_geom::intersect(
        &thick,
        &thin,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) else {
        panic!("the thin pipe crosses the thick one in curves")
    };
    let [one, other] = &loops[..] else {
        panic!("two loops, not {}", loops.len())
    };
    let (f, r) = (Orientation::Forward, Orientation::Reversed);

    let mut builder = SolidBuilder::new();
    let corner = builder
        .add_vertex(Point::new(0.0, 2.0, 0.0))
        .expect("a corner");
    let edge = builder
        .add_edge(corner, corner, one.run().clone())
        .expect("a loop");
    let coedge = builder.add_coedge(edge, f).expect("a coedge");
    assert!(
        matches!(
            builder.add_wire(&[DraftStep::Along(coedge)]),
            Err(rcad_topo::Error::ADraftWireIsTooShort { steps: 1 })
        ),
        "a lobe inside one loop is a wire of its own"
    );

    for (band_way, lobes_way) in [(f, r), (r, f)] {
        let mut builder = SolidBuilder::new();
        let corner = builder
            .add_vertex(Point::new(0.0, 2.0, 0.0))
            .expect("a corner");
        let edges = [one, other].map(|named| {
            builder
                .add_edge(corner, corner, named.run().clone())
                .expect("a loop")
        });
        let mut both_loops = |way: Orientation| {
            let steps = edges
                .map(|edge| DraftStep::Along(builder.add_coedge(edge, way).expect("a coedge")));
            builder.add_wire(&steps).expect("a wire round both loops")
        };
        let (band, lobes) = (both_loops(band_way), both_loops(lobes_way));
        let band = builder.add_face(thin, &[band]).expect("the band");
        let lobes = builder.add_face(thick, &[lobes]).expect("the lobes");
        builder
            .add_shell(&[(band, f), (lobes, f)])
            .expect("a shell");
        let said = builder.build(
            &mut a_model(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert!(
            matches!(
                said,
                Err(rcad_topo::Error::AMeshCoveredTheWrongSideOfALoop { .. })
            ),
            "the band walking its loops {band_way:?}: {said:?}"
        );
    }
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

/// How much of a ball of `radius` at the origin a round bar of `bar` up `z`
/// takes out, its axis `off` from the centre along `x`, by a midpoint rule of
/// `steps` steps out from the bar's axis and `steps` round it.
///
/// Over the bar's disc the ball is `2√(radius² − ρ²)` tall, `ρ` how far the
/// point is from the ball's axis: `ρ² = off² + s² + 2·off·s·cos θ` at `s` out
/// from the bar's axis and `θ` round it. Every bore here keeps `|off| + bar`
/// under `radius`, so the height is above zero over the whole disc.
fn what_a_bar_bores_out_of_a_ball(radius: f64, bar: f64, off: f64, steps: u32) -> f64 {
    assert!(
        off.abs() + bar < radius,
        "the bar runs wholly inside the ball"
    );
    let (out, round) = (bar / f64::from(steps), 2.0 * PI / f64::from(steps));
    let mut sum = 0.0;
    for i in 0..steps {
        let s = out * (f64::from(i) + 0.5);
        for j in 0..steps {
            let theta = round * (f64::from(j) + 0.5);
            let squared = off * off + s * s + 2.0 * off * s * theta.cos();
            sum += 2.0 * (radius * radius - squared).sqrt() * s;
        }
    }
    sum * out * round
}

/// Q1232's ring, built: a ball of `221/64` less a bar of `220/64` on its axis
/// meet in two circles at `z = ±21/64`, each a closed edge whose one corner
/// stands `top` or `bottom` quarter turns round `z` from `+x`. The ball's band
/// runs the top circle backwards and the bottom one forwards, and the bore's
/// wall each the other way, the one walk of the four that builds with both
/// corners on one meridian.
fn a_ring_of_a_ball_less_a_bar(
    model: &mut Model,
    top: i32,
    bottom: i32,
) -> rcad_topo::Result<SolidKey> {
    let (ball, bar, high) = (221.0 / 64.0, 220.0 / 64.0, 21.0 / 64.0);
    let skin = Surface::Sphere(Sphere::new(Point::origin(), ball).expect("a ball"));
    let wall = Surface::Cylinder(Cylinder::new(Axis::z(), bar).expect("a pipe"));
    let mut builder = SolidBuilder::new();
    let [top_circle, bottom_circle] = [(top, high), (bottom, -high)].map(|(quarters, z)| {
        let (x, y) = match quarters {
            0 => (bar, 0.0),
            1 => (0.0, bar),
            2 => (-bar, 0.0),
            3 => (0.0, -bar),
            _ => panic!("a corner stands 0 to 3 quarter turns round, not {quarters}"),
        };
        let corner = builder.add_vertex(Point::new(x, y, z)).expect("a corner");
        let level =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, 1.0))
                .expect("a plane");
        builder
            .add_edge(
                corner,
                corner,
                EdgeRun::RightRoundWhereItCrosses {
                    and: Surface::Plane(level),
                },
            )
            .expect("a circle")
    });
    let (f, r) = (Orientation::Forward, Orientation::Reversed);
    let mut band = |surface: Surface, turns: [Orientation; 2]| {
        let wires: Vec<_> = [top_circle, bottom_circle]
            .into_iter()
            .zip(turns)
            .map(|(circle, turn)| {
                let coedge = builder.add_coedge(circle, turn).expect("a coedge");
                builder
                    .add_wire(&[DraftStep::Along(coedge)])
                    .expect("a wire")
            })
            .collect();
        builder.add_face(surface, &wires).expect("a band")
    };
    let outside = band(skin, [r, f]);
    let inside = band(wall, [f, r]);
    builder
        .add_shell(&[(outside, f), (inside, r)])
        .expect("a shell");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}
