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
//! other refusal still stops the loop, except that since Q783
//! `PlacingItTookAwayTheSpaceItsMeshEncloses` — a finer mesh that found the
//! space the zero was hiding — is handed on instead of stopped on
//! (`hand_on_a_refusal_that_found_the_space`).
//!
//! **The fold is real and it is curable; what no solid on this tree does any
//! more is pair it with a zero.** This file is that search rather than a
//! paragraph claiming it (§16.10). The two families that can put every point of
//! a curved solid on one plane are swept here — the cap §41.6 measured, at
//! eight depths square to each of the three axes, and the sliver §206.3
//! measured, at six leans and twelve turns by both rules — and each of the 168
//! solids is meshed at ten rungs of the ladder the rescue walks, from half the
//! ball down. Of those 1 680 meshes **2 fold, every one of them a sliver, and
//! every one of the 2 gives way to a mesh at a finer rung** (38 until Q1027
//! moved a cap side between two rim points out of the triangulator's ear
//! rule). That is what the retry is for, measured: a fold is where the
//! mesher's own cuts happened to fall and not something about the shape, so
//! handing it on as the answer throws away a rung that would have answered.
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
//! family that folds is narrowed, not emptied, and 2 of these 1 680 meshes are
//! what that looks like from outside (38 until Q1027).
//!
//! **The two-closed-curve family is forced to one shape, and that shape now
//! meshes without coming back flat.** A solid encloses nothing when meshed
//! coarsely only if every point that mesh holds sits on one plane, so both of
//! the closed curves bounding such a patch have to lie in that plane too. Of the
//! five surfaces this kernel has, a plane cuts each of the four quadrics in one
//! conic, and only a torus square to its axis is cut in two closed curves —
//! which is what [`Picker::InsideAPipe`](rcad_geom::Picker::InsideAPipe) was
//! added for. So the shape is the crown of a ring and nothing else. Its curved
//! face goes right round the **ring**, and the mesher refused it until it cut a
//! patch round the ring as well as round the tube (§390). Every one of those
//! cuts carries a point of its own over the top of the tube, so no mesh of the
//! crown is flat: `the_crown_of_a_ring_holds_what_pappus_gives_and_no_rung_of_its_ladder_folds`
//! holds it to Pappus and walks its ladder, and its one fold, at the coarsest
//! rung, is cured at the next. With §208.3's arm taken out, that test still
//! passes, because the crown's coarsest mesh already encloses something and
//! never reaches the arm.
//!
//! **The other family the code admits is swept and is quiet.** `cut_in_two` asks
//! `piece.closes_on_one_curve`, which a loop of several arcs fails without any
//! second closed curve — a sliver chopped by further planes. 288 of those are
//! built here by narrowing the block that cuts the ball until its own side faces
//! do the chopping, and all 288 answer: **not one of their 2 880 meshes
//! folds**; the chop puts the rim's own arc off the plane of the cut, so none of
//! them comes back flat either. The bare cut is reached — 93 cuts of the 2 798
//! that sweep takes are `steps = 1` — it just meets nothing.
//!
//! So §208.3's arm still has no test that fails without it, and
//! `the_arm_removal_experiment` is the run that says so rather than a paragraph
//! claiming it.

use core::f64::consts::PI;
use std::collections::BTreeMap;

use rcad_geom::{
    Axis, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale,
    Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftStep, Error, Measurement, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey, SolidView,
};
use smallvec::smallvec;

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
            ("meshed", RUNGS * LEANS.len() * TURNS * 2 - 2),
            ("folded", 2),
            ("folds a later rung cured", 2),
            ("folds no later rung cured", 0),
        ]),
        "every rung of every sliver's ladder"
    );
}

/// The sliver chopped again by two more planes, which is the family `cut_in_two`
/// still charges a bare cut, and it neither folds nor encloses nothing.
///
/// §206.3 closed the sliver by charging a cut across a piece that is still one
/// whole closed curve at least two steps. The code asks
/// `piece.closes_on_one_curve`, which is **weaker than "bounded by more than one
/// closed curve"**: a rim made of several arcs fails it without any second
/// closed curve at all. Chopping the sliver with two more planes square to the
/// cut is the cheapest way to that rim — the block that cuts the ball is simply
/// made narrow, so its own two side faces do the chopping — and those pieces are
/// back on the old, bare cut.
///
/// Swept at four leans, twelve turns and six widths, the widths given as a
/// fraction of the rim's own radius so the chop lands in the same place on every
/// rim. **All 288 solids answer, and not one of their 2 880 meshes folds.** The
/// bare cut is real — instrumenting `cut_in_two` over this sweep showed 93 cuts
/// of 2 798 taken with `steps = 1` — but the chop also puts the rim's own arc
/// off the plane of the cut, so no coarse mesh here comes back flat and none of
/// them reaches the rescue's ladder at all.
///
/// Until Q1837 six refused `Undecidable`, all chopped at half the rim: at lean
/// `0.9` turned 2, 4 and 10 twelfths and at `1.2` turned 4 twelfths, on `side
/// of a half-space` (Q968); at `1.35` and `1.5` turned 4 twelfths, on `a point
/// inside a piece of a face`. Those were the walk's signs about the short arc
/// of a rim cut a hair apart, which it no longer needs where the other arc is
/// told; `the_six_chopped_slivers_the_walk_once_refused_hold_their_volume`
/// holds them to their volume. There were ten, all on that last name, until a
/// piece no point was found in was placed by a side it shares (§365.3, Q712):
/// that answers the four chopped at three tenths of the rim at lean `0.9`,
/// which the test after that holds to their volume. Eight more refused
/// `ResultIsNotManifold`, at leans `1.35` and `1.5` turned `0`, until §340.10:
/// the box round one of the block's faces stopped a rounding short of the nodes
/// on the ball's cut across it, and the cut was dropped as missing the face.
#[test]
fn no_sliver_chopped_by_the_blocks_own_sides_folds_at_any_deflection() {
    const LEANS: [f64; 4] = [0.9, 1.2, 1.35, 1.5];
    const ACROSS: [f64; 6] = [0.3, 0.5, 0.85, 1.2, 1.6, 1.9];
    const TURNS: usize = 12;
    let radius = 1.0;
    let mut tally = BTreeMap::new();
    for lean in LEANS {
        for step in 0..TURNS {
            #[expect(clippy::cast_precision_loss, reason = "twelve turns round a circle")]
            let turn = step as f64 * PI / 6.0;
            for across in ACROSS {
                let what = format!(
                    "the sliver at lean {lean}, turn {turn}, chopped at {across} of its rim"
                );
                let (model, answer) = chopped_sliver(radius, lean, turn, across);
                match answer {
                    Ok(answer) => {
                        *tally.entry("answered").or_insert(0) += 1;
                        count_the_ladder(&model, &answer, radius, &mut tally, &what);
                    }
                    Err(Error::ResultIsNotManifold { .. }) => {
                        *tally.entry("ResultIsNotManifold").or_insert(0) += 1;
                    }
                    Err(Error::Undecidable { .. }) => {
                        *tally.entry("Undecidable").or_insert(0) += 1;
                    }
                    Err(other) => panic!("{what}: refused {other:?}"),
                }
            }
        }
    }
    assert_eq!(
        tally,
        BTreeMap::from([
            ("answered", 288),
            ("meshed", 2880),
            ("folds a later rung cured", 0),
            ("folds no later rung cured", 0),
        ]),
        "every rung of every chopped sliver's ladder"
    );
}

/// The six chopped slivers the walk refused until Q1837, all chopped at half
/// the rim: lean `0.9` turned 2, 4 and 10 twelfths, and leans `1.2`, `1.35` and
/// `1.5` turned 4 twelfths. Each validates and holds the volume a sum over its
/// slices gives.
#[test]
fn the_six_chopped_slivers_the_walk_once_refused_hold_their_volume() {
    let (radius, across) = (1.0, 0.5);
    for (lean, step) in [(0.9, 2), (0.9, 4), (0.9, 10), (1.2, 4), (1.35, 4), (1.5, 4)] {
        let turn = f64::from(step) * PI / 6.0;
        let what =
            format!("the sliver at lean {lean}, turn {turn}, chopped at {across} of its rim");
        let (model, answer) = chopped_sliver(radius, lean, turn, across);
        let answer = answer.unwrap_or_else(|why| panic!("{what}: refused {why:?}"));
        let probe = measure(&model, &answer, 0.001, &what);
        println!(
            "PROBE {what}: solids {} volume {} error {} closed {}",
            answer.len(),
            probe.volume(),
            probe.volume_error(),
            a_chopped_slivers_volume(lean, turn, across * lean.sin())
        );
        measures_within_what_it_declares(
            &model,
            &answer,
            0.001,
            a_chopped_slivers_volume(lean, turn, across * lean.sin()),
            &what,
        );
    }
}

/// The four chopped slivers that only a side they share places: lean `0.9`,
/// chopped at three tenths of the rim, turned 2, 4, 8 and 10 twelfths. Each
/// validates and holds the volume a sum over its slices gives, so the rule
/// that placed them (§365.3) wrote the right answer.
#[test]
fn the_four_chopped_slivers_placed_by_a_side_they_share_hold_their_volume() {
    let (radius, lean, across) = (1.0, 0.9, 0.3);
    for step in [2, 4, 8, 10] {
        let turn = f64::from(step) * PI / 6.0;
        let what =
            format!("the sliver at lean {lean}, turn {turn}, chopped at {across} of its rim");
        let (model, answer) = chopped_sliver(radius, lean, turn, across);
        let answer = answer.unwrap_or_else(|why| panic!("{what}: refused {why:?}"));
        measures_within_what_it_declares(
            &model,
            &answer,
            0.001,
            a_chopped_slivers_volume(lean, turn, across * lean.sin()),
            &what,
        );
    }
}

/// What [`chopped_sliver`] keeps of a unit ball, summed over slices square to
/// the two chopping planes by Simpson's rule.
///
/// The sliver is the ball past a plane `cos(lean)` from its centre, and its
/// rim has radius `sin(lean)` and its centre `sin(lean) · cos(turn)` from the
/// middle of the two planes, which are `half` either side of it. The slice
/// `x` from the rim's centre is a segment of a circle of radius `√(1 − x²)`,
/// cut `cos(lean)` from that circle's centre.
fn a_chopped_slivers_volume(lean: f64, turn: f64, half: f64) -> f64 {
    const SLICES: u32 = 200_000;
    let (rim, apart) = (lean.sin(), lean.cos());
    let off = rim * turn.cos();
    let from = (-half - off).max(-rim);
    let to = (half - off).min(rim);
    let segment = |x: f64| {
        let squared = 1.0 - x * x;
        let circle = squared.sqrt();
        if circle <= apart {
            return 0.0;
        }
        squared * (apart / circle).acos() - apart * (squared - apart * apart).sqrt()
    };
    let step = (to - from) / f64::from(SLICES);
    let inner: f64 = (1..SLICES)
        .map(|i| segment(from + f64::from(i) * step) * if i % 2 == 1 { 4.0 } else { 2.0 })
        .sum();
    (segment(from) + segment(to) + inner) * step / 3.0
}

/// The sliver of §206.3 chopped by the two side faces of its own block, which
/// stand square to the cut and `across` of the rim's radius either side of the
/// pole.
fn chopped_sliver(
    radius: f64,
    lean: f64,
    turn: f64,
    across: f64,
) -> (Model, Result<Vec<SolidKey>, Error>) {
    let leaning = turn_about(Vector::new(turn.sin(), -turn.cos(), 0.0), PI + lean);
    let on_the_pole = Isometry::translation(Vector::new(0.0, 0.0, -radius))
        .expect("a shift")
        .compose(&leaning);
    fused_with_a_block(
        radius,
        across * radius * lean.sin(),
        &on_the_pole,
        Rule::Intersection,
    )
}

/// The crown of a ring is the only shape that could pair a zero with a bare cut
/// across two closed curves, and it builds, holds what Pappus gives, and folds
/// at one rung of its ladder only, which the next rung cures.
///
/// A bare cut is charged to a piece whose loop is not one whole closed curve,
/// and the family `cut_in_two` says nothing about is a curved patch bounded by
/// **two** closed curves. For the solid to enclose nothing when meshed coarsely
/// every corner of it has to sit on one plane, so both of those curves have to
/// lie in that plane — and of the five surfaces this kernel has, a torus square
/// to its axis is the only one a plane meets in two closed curves at all
/// (`Picker::InsideAPipe` says so from the other side; a plane cuts each of the
/// four quadrics in one conic).
///
/// So the shape is forced, and this is it: everything of a torus above a plane
/// square to its axis, cut high enough up the tube that the patch spans under a
/// quarter turn. The face is one wire: the outer circle, a seam across the
/// tube, the inner circle, and the seam back. The walk sees the seam's two ends
/// a whole turn apart round the **ring**, and until the mesher cut a patch
/// round the ring it refused the wire as `AWireWillNotLieFlatOnItsFace`, a gap
/// and not a limit (§280.8). It is cut round the ring now, and each cut that
/// way carries a point of its own: a bare one runs from a point of the outer
/// circle to a point of the inner one, a pair the flat face across both circles
/// may draw too, and the crown folded `TwoPatchesOfTheMeshDrewTheSameSide
/// { drawn: 4, used: 4 }` at every rung with bare cuts (§390).
///
/// **Pappus.** The tube's disc of radius five above the line four from its
/// centre is a segment of area `25·acos(4/5) − 4·3`, with its centroid on the
/// ring's own circle, twenty from the axis, so the crown holds
/// `2π · 20 · (25·acos(4/5) − 12) = 513.66`.
///
/// **The one fold is at the coarsest rung**, room five, the tube's own radius
/// and five times the crown's height, where the curved face and the flat one
/// each join a point of the outer circle to one of the inner. Room two and a
/// half meshes, and so does every rung after it.
#[test]
fn the_crown_of_a_ring_holds_what_pappus_gives_and_no_rung_of_its_ladder_folds() {
    let mut model = Model::new(Scale::millimetres(500.0).expect("a positive size"));
    let crown = a_crown_of_a_ring(&mut model)
        .unwrap_or_else(|why| panic!("the crown of a ring was refused {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the model holding the crown is sound");
    let segment = 25.0 * 0.8_f64.acos() - 4.0 * 3.0;
    let pappus = 2.0 * PI * 20.0 * segment;
    for deflection in [0.5, 0.1, 0.05] {
        measures_within_what_it_declares(
            &model,
            &[crown],
            deflection,
            pappus,
            &format!("the crown of a ring at {deflection}"),
        );
    }
    let mut tally = BTreeMap::new();
    count_the_ladder(&model, &[crown], 5.0, &mut tally, "the crown of a ring");
    assert_eq!(
        tally,
        BTreeMap::from([
            ("meshed", RUNGS - 1),
            ("folded", 1),
            ("folds a later rung cured", 1),
            ("folds no later rung cured", 0),
        ]),
        "every rung of the crown's ladder"
    );
}

/// Taking §208.3's arm out again changes nothing this tree runs, so the arm
/// still has no test that fails without it.
///
/// The arm cannot be shown off by the ordinary "revert the change and watch the
/// test fail", because it is already on the base this file was written against.
/// So it was taken out by hand, run, and put back. Verbatim, in order:
///
/// ```text
/// $ mise exec -- just t -E 'binary(a_mesh_that_folds_at_one_deflection_and_not_the_next)'
///      Summary [  11.383s] 4 tests run: 4 passed, 0 skipped
/// ```
///
/// Then the six lines of `how_much_space_a_curved_solid_encloses` that §208.3
/// names — the `match view.mesh(less, policy, log)` and its three arms — were
/// put back to what stood before it, `let Ok(finer) = view.mesh(less, policy,
/// log) else { break; };`, as an uncommitted edit to
/// `crates/rcad-topo/src/validate.rs`:
///
/// ```text
/// $ mise exec -- just t --no-fail-fast
///      Summary [  63.289s] 2299 tests run: 2298 passed, 1 failed, 6 skipped
///         FAIL a_mesh_that_folds_at_one_deflection_and_not_the_next the_arm_removal_experiment
/// $ git checkout -- crates/rcad-topo/src/validate.rs
/// ```
///
/// That run is the **whole workspace** on the merged tree of §211–§214, and the
/// one test it fails is this one, which reads the arm's text. An earlier
/// transcript here showed `884 passed` twice for `-p rcad-topo`; it was taken
/// before this test existed and stopped being reproducible the moment it did,
/// since taking the arm out is exactly what this test fails on.
///
/// **So this file does not close §208.3's hole, and says so rather than
/// implying it.** Neither family it adds reaches the rescue's ladder at all:
/// the crown of a ring meshes with a point over the top of the tube on every
/// cut round the ring, so its coarsest mesh encloses something, and the chopped
/// sliver takes the bare cut but never comes back flat. The arm is still a
/// guard with nothing behind it. The crown was run against it once it meshed:
/// with the arm put back to `Err(_) => break` as an uncommitted edit, this file
/// and `building_a_solid_out_of_faces.rs` ran `17 tests run: 16 passed, 1
/// failed`, and the one failure was this test (§390).
///
/// What this test can check is that the experiment above is still an experiment
/// about this tree. If the arm moves or goes, the record stops describing what
/// is there, and that is worth a failure rather than a stale paragraph.
#[test]
fn the_arm_removal_experiment() {
    let ladder = include_str!("../src/validate.rs");
    assert!(
        ladder.contains("Err(Error::TwoPatchesOfTheMeshDrewTheSameSide { .. }) => {}"),
        "§208.3's arm is what the experiment above was run against"
    );
}

/// Ring radius `20`, tube radius `5`, cut at `z = 4`.
///
/// The three lengths are a Pythagorean triple on purpose: the cut circles sit at
/// `20 ± 3`, so both corners are exact on the torus. The quartic at `(23, 0, 4)`
/// is `(23² + 4² + 20² − 5²)² − 4·20²·23² = 920² − 846400`, a difference of two
/// products equal number for number.
fn a_crown_of_a_ring(model: &mut Model) -> Result<SolidKey, Error> {
    let skin = Torus::new(Axis::z(), 20.0, 5.0).expect("a tube narrower than its ring");
    let mut builder = SolidBuilder::new();

    let outer_corner = builder
        .add_vertex(Point::new(23.0, 0.0, 4.0))
        .expect("a finite point");
    let inner_corner = builder
        .add_vertex(Point::new(17.0, 0.0, 4.0))
        .expect("a finite point");

    let the_cut =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 4.0), Vector::new(0.0, 0.0, -1.0))
            .expect("the z axis is a unit normal");
    let mut cut_circle = |corner| {
        builder
            .add_edge(corner, corner, EdgeRun::RightRoundWhereItsFacesCross)
            .expect("one whole circle of the two this plane cuts")
    };
    let outer_circle = cut_circle(outer_corner);
    let inner_circle = cut_circle(inner_corner);

    let straight_across = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("the y axis is a unit normal");
    let seam = builder
        .add_edge(
            outer_corner,
            inner_corner,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(straight_across),
                picked_by: smallvec![Picker::from(
                    HalfSpace::new(Vector::new(0.0, 0.0, 1.0), 0.0)
                        .expect("a normal with a length")
                )],
            },
        )
        .expect("the arc of the tube over the top, from the outer cut to the inner one");

    let round_the_crown: Vec<DraftStep> = [
        (outer_circle, Orientation::Forward),
        (seam, Orientation::Forward),
        (inner_circle, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ]
    .into_iter()
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("an edge of this build"),
        )
    })
    .collect();
    let wire = builder
        .add_wire(&round_the_crown)
        .expect("round the outer cut, over the top, back round the inner cut, and over again");
    let crown = builder
        .add_face(Surface::Torus(skin), &[wire])
        .expect("a face on the torus");

    let mut rim = |edge, turn| {
        let coedge = builder
            .add_coedge(edge, turn)
            .expect("an edge of this build");
        builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("one closed edge is a wire")
    };
    let outer_rim = rim(outer_circle, Orientation::Reversed);
    let inner_rim = rim(inner_circle, Orientation::Forward);
    let floor = builder
        .add_face(Surface::Plane(the_cut), &[outer_rim, inner_rim])
        .expect("a flat ring with a hole in it");

    builder
        .add_shell(&[(crown, Orientation::Forward), (floor, Orientation::Forward)])
        .expect("two faces close");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
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
fn fused(radius: f64, placed: &Isometry, rule: Rule) -> (Model, Result<Vec<SolidKey>, Error>) {
    fused_with_a_block(radius, 3.0 * radius, placed, rule)
}

/// The same, with the block `across` wide in its own `x` so its two side faces
/// chop whatever the bottom face cut off.
fn fused_with_a_block(
    radius: f64,
    across: f64,
    placed: &Isometry,
    rule: Rule,
) -> (Model, Result<Vec<SolidKey>, Error>) {
    let mut model = Model::new(Scale::millimetres(25.0 * radius).expect("a positive size"));
    let ball = model
        .add_ball(radius, &Isometry::identity())
        .expect("a ball");
    let wide = 3.0 * radius;
    let square = ProfileBuilder::new(vec![
        [-across, -wide],
        [across, -wide],
        [across, wide],
        [-across, wide],
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
