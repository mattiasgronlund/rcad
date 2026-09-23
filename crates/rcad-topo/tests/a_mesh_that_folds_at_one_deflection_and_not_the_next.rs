//! What reaches `validate.rs`'s halving loop, and what happens there when a
//! finer mesh folds.
//!
//! `how_much_space_a_curved_solid_encloses` reads a curved solid's sign off a
//! mesh, and a mesh of a patch spanning less than a quarter turn comes back
//! flat — so the solid encloses exactly nothing and the sign is not an answer.
//! §41.6's rescue asks again with half the room, and again, until a mesh finds
//! the bulge the flat patch was hiding. The loop used to break on **any**
//! refusal from a finer mesh, and §206.3 found a shape where that was the wrong
//! call: the sliver's coarse mesh came back flat, its next mesh folded two
//! patches onto each other, and the bulge a third mesh would have found was
//! never asked for. `TwoPatchesOfTheMeshDrewTheSameSide` is now retried; every
//! other refusal still stops the loop.
//!
//! **The fold is real and it is curable; what no solid on this tree does any
//! more is pair it with a zero.** This file is that search rather than a
//! paragraph claiming it (§16.10). The two families that can put every point of
//! a curved solid on one plane are swept here — the cap §41.6 measured, at
//! eight depths square to each of the three axes, and the sliver §206.3
//! measured, at six leans and twelve turns by both rules — and each of the 168
//! solids is meshed at ten rungs of the ladder the rescue walks, from half the
//! ball down. Of those 1 680 meshes **38 fold, every one of them a sliver, and
//! every one of the 38 gives way to a mesh at a finer rung.** That is what the
//! retry is for, measured: a fold is where the mesher's own cuts happened to
//! fall and not something about the shape, so handing it on as the answer
//! throws away a rung that would have answered.
//!
//! **No solid reaches the loop to spend those rungs, though.** §206.3's own
//! shape is fixed at the source — `cut_in_two` charges at least two steps for a
//! cut across a piece that is still one whole closed curve, so the sliver's
//! coarse mesh now carries a point off the plane of its rim and encloses
//! something. Every one of the 168 answers, and none refuses the
//! `VolumeIsNotPositive` a zero handed on comes back as. Running the whole test
//! suite with the loop instrumented to write a line per call that reached it
//! agrees from the other side: four solids reached it, all four from
//! `a_solid_whose_numbers_are_too_big_to_mesh.rs`, and all four stopped on
//! their first rung with `DeflectionBelowWhatAMeshCanReach` and no fold. So the
//! retry costs the gate nothing today.
//!
//! The §16.10 verdict is therefore neither *limit* nor *gap* but **a guard**:
//! the answer exists, the arithmetic reaches it, and no solid asks. It is kept
//! because what closed §206.3 is a cost bound and not a proof — `cut_in_two`'s
//! own doc comment says nothing shows a bare cut across a piece bounded by
//! *more than one* closed curve can never meet a neighbour's diagonal. The
//! family that folds is narrowed, not emptied, and 38 of these 1 680 meshes are
//! what that looks like from outside.

use core::f64::consts::PI;
use std::collections::BTreeMap;

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Measurement, Model, ProfileBuilder, Rule, SolidKey, SolidView};

/// How many rungs of the rescue's ladder each solid is walked down.
///
/// The rescue halves from the width of the mesh's own box, which on a ball of
/// `r` is `r` to start with, so ten rungs reach `r/1024` — a chord of
/// `√(8r · r/1024) = 0.088r` and some seventy steps round the equator. The real
/// loop stops where the mesher refuses; this stops sooner, on purpose, because
/// what is being counted is folds and a fold is a sampling coincidence, which
/// is likeliest where the steps are few.
const RUNGS: usize = 10;

/// The ball of radius four §41.6 measured, cut by a plane at `d` square to each
/// of the three axes.
///
/// A cap of height `h` spans under a quarter turn once `h/r` is under
/// `1 − cos(π/4) = 0.2929`, which on this ball is every depth from `2.83` up —
/// the row where the `x` column stopped answering on §41.6's base. Depths below
/// it are swept too, because a cap cut square to `x` or `y` holds no pole at
/// any depth and so takes the same path through the mesher.
#[test]
fn no_cap_of_a_ball_folds_at_any_deflection_the_rescue_would_ask_for() {
    const DEPTHS: [f64; 8] = [1.0, 2.0, 2.8, 2.83, 3.0, 3.5, 3.9, 3.99];
    let radius = 4.0;
    let mut tally = BTreeMap::new();
    for depth in DEPTHS {
        for axis in ["x", "y", "z"] {
            let what = format!("a cap of a ball of {radius} cut at {depth} square to {axis}");
            let (model, answer) = cap(radius, depth, axis);
            let answer = answer.unwrap_or_else(|why| panic!("{what}: refused {why}"));
            measures_within_what_it_declares(
                &model,
                &answer,
                0.001,
                a_caps_volume(radius, radius - depth),
                &what,
            );
            count_the_ladder(&model, &answer, radius, &mut tally, &what);
        }
    }
    assert_eq!(
        tally,
        BTreeMap::from([
            ("meshed", RUNGS * DEPTHS.len() * 3),
            ("folds a later rung cured", 0),
            ("folds no later rung cured", 0),
        ]),
        "every rung of every cap's ladder"
    );
}

/// The sliver on the far side of a plane through a ball's south pole, over
/// three times the leans and turns `a_ball_cut_through_its_pole.rs` sweeps.
///
/// That file's 63 planes are where §206.3's 72 refusals were counted, and its
/// leans stop at `1.1`. A sliver's rim runs closer to the pole as the lean
/// grows, so the leans are carried right up to a quarter turn here, and the
/// turns are twelve round the circle rather than seven.
#[test]
fn no_sliver_cut_through_a_balls_pole_folds_at_any_deflection() {
    const LEANS: [f64; 6] = [0.2, 0.5, 0.9, 1.2, 1.4, 1.5];
    const TURNS: usize = 12;
    let radius = 1.0;
    let mut tally = BTreeMap::new();
    for lean in LEANS {
        for step in 0..TURNS {
            #[expect(clippy::cast_precision_loss, reason = "twelve turns round a circle")]
            let turn = step as f64 * PI / 6.0;
            for (rule, lean_of_the_block) in
                [(Rule::Intersection, PI + lean), (Rule::Difference, lean)]
            {
                let what = format!("the sliver at lean {lean}, turn {turn}, by {rule:?}");
                let (model, answer) = sliver(radius, lean_of_the_block, turn, rule);
                let answer = answer.unwrap_or_else(|why| panic!("{what}: refused {why}"));
                measures_within_what_it_declares(
                    &model,
                    &answer,
                    0.001 * radius,
                    a_caps_volume(radius, radius - radius * lean.cos()),
                    &what,
                );
                count_the_ladder(&model, &answer, radius, &mut tally, &what);
            }
        }
    }
    assert_eq!(
        tally,
        BTreeMap::from([
            ("meshed", RUNGS * LEANS.len() * TURNS * 2 - 38),
            ("folded", 38),
            ("folds a later rung cured", 38),
            ("folds no later rung cured", 0),
        ]),
        "every rung of every sliver's ladder"
    );
}

/// Mesh the one solid of `answer` at every deflection the rescue would ask it
/// for, and count what came back by name.
///
/// The rescue halves from half the width of the box round the coarse mesh's
/// points; on a ball of `radius` that box is the ball's own and the first rung
/// is `radius`. A refusal other than a fold ends this solid's ladder, the way
/// it ends the rescue's.
fn count_the_ladder(
    model: &Model,
    answer: &[SolidKey],
    radius: f64,
    tally: &mut BTreeMap<&'static str, usize>,
    what: &str,
) {
    let view = one_solid(model, answer, what);
    let mut room = radius;
    let mut folds_waiting_to_be_cured = 0;
    for _ in 0..RUNGS {
        let asked = Deflection::of(room).expect("a positive length");
        let outcome = match view.mesh(asked, &Policy::certified(), &mut ProvenanceLog::new()) {
            Ok(_) => "meshed",
            Err(Error::TwoPatchesOfTheMeshDrewTheSameSide { .. }) => "folded",
            Err(Error::CannotMeshAFace { .. }) => "CannotMeshAFace",
            Err(Error::DeflectionBelowWhatAMeshCanReach { .. }) => {
                "DeflectionBelowWhatAMeshCanReach"
            }
            Err(Error::MoreMeshPointsThanASolidWillHold { .. }) => {
                "MoreMeshPointsThanASolidWillHold"
            }
            Err(other) => panic!("{what}: meshing at {room} refused {other:?}"),
        };
        *tally.entry(outcome).or_insert(0) += 1;
        match outcome {
            "folded" => folds_waiting_to_be_cured += 1,
            "meshed" => {
                *tally.entry("folds a later rung cured").or_insert(0) += folds_waiting_to_be_cured;
                folds_waiting_to_be_cured = 0;
            }
            _ => break,
        }
        room /= 2.0;
    }
    *tally.entry("folds no later rung cured").or_insert(0) += folds_waiting_to_be_cured;
}

/// A ball of `radius` about the origin intersected with a block whose near face
/// is the plane `axis = depth`, keeping the cap beyond it.
fn cap(radius: f64, depth: f64, axis: &str) -> (Model, Result<Vec<SolidKey>, Error>) {
    let quarter_turn_from_z = match axis {
        "x" => turn_about(Vector::new(0.0, 1.0, 0.0), PI / 2.0),
        "y" => turn_about(Vector::new(1.0, 0.0, 0.0), -PI / 2.0),
        "z" => Isometry::identity(),
        other => panic!("no axis called {other}"),
    };
    let along = match axis {
        "x" => Vector::new(depth, 0.0, 0.0),
        "y" => Vector::new(0.0, depth, 0.0),
        _ => Vector::new(0.0, 0.0, depth),
    };
    let standing_on_the_plane = Isometry::translation(along)
        .expect("a shift")
        .compose(&quarter_turn_from_z);
    fused(radius, &standing_on_the_plane, Rule::Intersection)
}

/// A ball of `radius` fused by `rule` with a block standing on its south pole,
/// leaning `lean` about the level axis `turn` round from `x`.
fn sliver(radius: f64, lean: f64, turn: f64, rule: Rule) -> (Model, Result<Vec<SolidKey>, Error>) {
    let leaning = turn_about(Vector::new(turn.sin(), -turn.cos(), 0.0), lean);
    let on_the_pole = Isometry::translation(Vector::new(0.0, 0.0, -radius))
        .expect("a shift")
        .compose(&leaning);
    fused(radius, &on_the_pole, rule)
}

fn turn_about(direction: Vector, angle: f64) -> Isometry {
    Isometry::rotation(
        &Axis::new(Point::origin(), direction).expect("a direction"),
        angle,
    )
    .expect("a turn")
}

/// A ball of `radius` about the origin fused by `rule` with a block three radii
/// wide whose bottom face sits where `placed` puts it.
fn fused(
    radius: f64,
    placed: &Isometry,
    rule: Rule,
) -> (Model, Result<Vec<SolidKey>, Error>) {
    let mut model = Model::new(Scale::millimetres(25.0 * radius).expect("a positive size"));
    let ball = model
        .add_ball(radius, &Isometry::identity())
        .expect("a ball");
    let wide = 3.0 * radius;
    let square = ProfileBuilder::new(vec![
        [-wide, -wide],
        [wide, -wide],
        [wide, wide],
        [-wide, wide],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    let block = model.add_extrusion(&square, wide, placed).expect("a block");
    let answer = model
        .fuse(
            &[ball],
            &[block],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map(|fused| fused.solids().to_vec());
    if answer.is_ok() {
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a fuse writes a sound model or writes nothing");
    }
    (model, answer)
}

/// One solid, within the volume error its own mesh declares of `closed_form`.
///
/// Neither a shallow cap nor a sliver holds the ball's centre, so the `3δ/r`
/// bound `a_ball_cut_through_its_pole.rs` uses says nothing about them; what
/// the measurement declares does.
fn measures_within_what_it_declares(
    model: &Model,
    answer: &[SolidKey],
    deflection: f64,
    closed_form: f64,
    what: &str,
) {
    let measured = measure(model, answer, deflection, what);
    assert!(
        (measured.volume() - closed_form).abs() <= measured.volume_error(),
        "{what}: measured {}, closed form {closed_form}, declared {}",
        measured.volume(),
        measured.volume_error()
    );
}

fn measure(model: &Model, answer: &[SolidKey], deflection: f64, what: &str) -> Measurement {
    one_solid(model, answer, what)
        .measure(
            Deflection::of(deflection).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: the answer measures: {why:?}"))
}

fn one_solid<'a>(model: &'a Model, answer: &[SolidKey], what: &str) -> SolidView<'a> {
    let [solid] = answer else {
        panic!("{what}: one solid, not {}", answer.len());
    };
    model.solid(*solid).expect("this model made the key")
}

/// The cap of height `h` off a ball of `radius`: `π h² (3r − h) / 3`.
fn a_caps_volume(radius: f64, h: f64) -> f64 {
    PI * h * h * (3.0 * radius - h) / 3.0
}
