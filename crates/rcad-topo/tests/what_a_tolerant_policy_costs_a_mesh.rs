//! What a tolerant policy costs a mesh, where the walk runs out of room.
//!
//! A tolerant caller is the escape hatch: it has said how far out of place it
//! will let a point be, and in exchange it gets an answer where a certified
//! caller gets a refusal. Everywhere `rcad-geom`'s `room_left_over` runs out of
//! room, that trade does not happen. **Neither caller gets a mesh, and the
//! tolerant one is told something untrue about why.**
//!
//! - **Certified**: `CannotPlaceAPointThatCloseToACrossing { asked, closest }`.
//!   It carries both numbers, and the two of them are the whole diagnosis: the
//!   walk had `asked` of the deflection left and the crossing rounds by
//!   `closest`, which is more.
//! - **Tolerant**: `AWireDoesNotFanIntoALoopToMesh` with
//!   `TooFewCorners { count: 1 }`. That is the name for a *malformed solid*, and
//!   the solid is not malformed — see
//!   [`every_bar_in_the_table_is_built_and_checked_with_no_tolerance_asked_for`]
//!   below, which builds each of these bars under `Policy::certified()`, gets an
//!   empty provenance log for it, and passes `Model::validate` certified too.
//!
//! Twelve cells, twelve of each. The tolerant caller is strictly worse off in
//! every one: same absence of a mesh, worse account of why.
//!
//! # The mechanism
//!
//! `room_left_over` (`rcad-geom/src/walk.rs`) is one `if` with two arms. A
//! certified caller takes the `Err` arm. A tolerant one is waved through with
//! `Ok(None)`, and the walk hands back a crossing with **no points between its
//! ends**. The leaning cap's only edge is a closed one, so its one corner then
//! sits at both ends of a polyline of length one, and the mesher is offered a
//! wire that fans into a single corner. Nothing between the wave-through and the
//! refusal knows the two are the same event —
//! [`the_walk_that_gave_up_is_the_one_the_log_names`] is what ties them.
//!
//! `docs/consumers.md` records a downstream consumer that keeps an unrecognised
//! refusal's debug text verbatim in its bug reports. What that consumer files
//! for this is [`the_text_a_tolerant_caller_is_left_holding`]: a corner count
//! and a face key, and not one of the two numbers that say what happened.
//!
//! # Why the bar has to lean so hard
//!
//! Reaching `room_left_over` on a closed edge needs the crossing's own rounding
//! to beat the floor `SolidView::mesh` puts under a deflection, and for an
//! ordinary bar those two are in step. They come apart when a cap plane leans:
//! the crossing charges its rounding against the ellipse the plane cuts, whose
//! semi-axis down the slope grows without bound as the plane lies down, while
//! the mesher's floor charges the face's *corners*, and the seam corner of a bar
//! cut this way stays where it always was.
//!
//! [`the_floor_under_a_deflection_stays_put_while_the_crossing_it_stands_for_runs_away`]
//! measures both halves of that: over the four leans here the floor does not
//! move off `6.14e-13` while the crossing's rounding runs from `2.3e-4` to
//! `5.7e-3`.
//!
//! **So this test depends on where that floor is charged.** If
//! `SolidView::room_a_mesh_has_to_work_in` is ever changed to charge a face's
//! crossings rather than only its corners, the window these twelve cells sit in
//! closes and this file has to move — which would be the fix for what it pins,
//! not a regression. `what_the_mesher_does_with_an_empty_walk_on_a_closed_edge.rs`
//! reaches the same site from the other side, on one cell.

use rcad_geom::{Deflection, Isometry, Plane, Point, Scale, Vector};
use rcad_mesh::NotALoop;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, FaceKey, Model, SolidKey};

/// The radius of every bar in the table, and the seam is at `x = RADIUS, y = 0`.
const RADIUS: f64 = 1.0;

/// Where the leaning cap crosses the seam, which is the only corner it puts on
/// the wall.
const AT_THE_SEAM: f64 = 0.5;

/// How far each top cap leans, as the ratio of its normal across the bar's axis
/// to its normal along it.
///
/// The crossing's rounding comes out at `lean · 5.7e-14` for these — measured,
/// from the `closest` the certified refusal carries — and what it has to beat is
/// three sixteenths of the deflection, which is the share of the budget a
/// boundary side is refined to. Four leans spanning a factor of twenty-five, so
/// that no cell below is the only one that works.
const LEANS: [f64; 4] = [4e9, 1e10, 3e10, 1e11];

/// Deflections coarse enough that the flat bottom cap and the wall are walked
/// normally, and still far under every crossing rounding in [`LEANS`].
///
/// The floor `SolidView::room_a_mesh_has_to_work_in` puts under a bar of
/// [`RADIUS`] at the origin measures `6.14e-13` — see
/// [`the_floor_under_a_deflection_stays_put_while_the_crossing_it_stands_for_runs_away`],
/// which hunts for it rather than taking anyone's word — so the coarsest of
/// these is nine orders of magnitude clear of it and none is a deflection the
/// mesher would refuse outright.
const DEFLECTIONS: [f64; 3] = [1e-4, 3e-4, 1e-3];

/// How many times the hunt for the floor halves the range it has left.
///
/// A hundred geometric halvings take `1e-300 .. 1e-4` down to nothing, and the
/// two tests that share this file run in well under a second together.
const HALVINGS_TO_FIND_THE_FLOOR: usize = 100;

/// The number of (lean, deflection) pairs, checked against what the sweep
/// actually visits so a table trimmed by accident cannot pass quietly.
const CELLS: usize = LEANS.len() * DEFLECTIONS.len();

/// The record `room_left_over` pushes when it waves a tolerant caller through.
///
/// Written out rather than imported from `rcad_geom` so that the thing under
/// test does not get to say what the test expects.
const ROOM_LEFT_TO_REFINE_A_WALK: &str = "room left to refine a walk";

/// Every cell: a certified caller is told what happened, a tolerant one is told
/// its solid is malformed.
#[test]
fn a_tolerant_caller_is_refused_as_if_its_solid_were_broken_and_a_certified_one_is_not() {
    let mut visited = 0;
    for lean in LEANS {
        let (model, bar) = a_bar_cut_off_almost_along_its_own_axis(
            lean,
            &a_tolerant_caller(),
            &mut ProvenanceLog::new(),
        );
        for asked in DEFLECTIONS {
            let deflection = Deflection::of(asked).expect("a positive length");

            let certified =
                model
                    .solid(bar)
                    .mesh(deflection, &Policy::certified(), &mut ProvenanceLog::new());
            let Err(Error::Geometry(rcad_geom::Error::CannotPlaceAPointThatCloseToACrossing {
                asked: left_to_walk_it_in,
                closest: rounds_by,
            })) = certified
            else {
                panic!(
                    "a bar leaning {lean:e} meshed at {asked:e} by a certified caller: \
                     {certified:?}"
                );
            };
            assert!(
                rounds_by > left_to_walk_it_in,
                "the crossing rounds by {rounds_by} and {left_to_walk_it_in} was left to walk it \
                 in, so this cell is not one where the walk ran out of room"
            );

            let tolerant =
                model
                    .solid(bar)
                    .mesh(deflection, &a_tolerant_caller(), &mut ProvenanceLog::new());
            let Err(Error::AWireDoesNotFanIntoALoopToMesh { face, why }) = tolerant else {
                panic!(
                    "a bar leaning {lean:e} meshed at {asked:e} by a tolerant caller: {tolerant:?}"
                );
            };
            assert_eq!(
                why,
                NotALoop::TooFewCorners { count: 1 },
                "a bar leaning {lean:e} at {asked:e}: the emptied circle's one corner is the \
                 whole of its cap's wire"
            );
            assert_eq!(
                face,
                the_cap_that_leans(&model, bar),
                "a bar leaning {lean:e} at {asked:e}: the face refused is the cap the emptied \
                 ellipse bounds"
            );

            visited += 1;
        }
    }
    assert_eq!(visited, CELLS, "every cell of the table has to be walked");
}

/// The two numbers that are supposed to keep step, measured coming apart.
///
/// The mesher's floor is what a caller is told it may ask for; the crossing's
/// rounding is what the walk can actually deliver. Lean the cap harder and the
/// second grows in proportion while **the first does not move at all**. Every
/// cell in the table lives in the gap that opens between them, which is why no
/// deflection a caller is allowed to ask for can be walked.
///
/// Both numbers are read off the public API — the floor by hunting for where
/// `DeflectionBelowWhatAMeshCanReach` stops being the answer, the rounding out
/// of the `closest` the certified refusal carries. Neither is worked out from
/// the arithmetic that produces it.
#[test]
fn the_floor_under_a_deflection_stays_put_while_the_crossing_it_stands_for_runs_away() {
    let mut floors = Vec::new();
    let mut roundings = Vec::new();
    for lean in LEANS {
        let (model, bar) = a_bar_cut_off_almost_along_its_own_axis(
            lean,
            &a_tolerant_caller(),
            &mut ProvenanceLog::new(),
        );

        let mesh_it = |asked: f64| {
            model.solid(bar).mesh(
                Deflection::of(asked).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
        };
        let (mut under, mut over): (f64, f64) = (1e-300, DEFLECTIONS[0]);
        for _ in 0..HALVINGS_TO_FIND_THE_FLOOR {
            let middle = (under * over).sqrt();
            if matches!(
                mesh_it(middle),
                Err(Error::DeflectionBelowWhatAMeshCanReach { .. })
            ) {
                under = middle;
            } else {
                over = middle;
            }
        }
        floors.push(over);

        let refused = mesh_it(DEFLECTIONS[0]);
        let Err(Error::Geometry(rcad_geom::Error::CannotPlaceAPointThatCloseToACrossing {
            closest,
            ..
        })) = refused
        else {
            panic!("a bar leaning {lean:e} was refused with {refused:?}");
        };
        roundings.push(closest);
    }

    for (lean, floor) in LEANS.iter().zip(&floors) {
        assert_eq!(
            *floor, floors[0],
            "the floor under a bar leaning {lean:e} is {floor:e}, and under one leaning {:e} it \
             is {:e} — if these ever differ the gap this file lives in has been closed",
            LEANS[0], floors[0]
        );
    }
    for pair in roundings.windows(2) {
        assert!(
            pair[1] > pair[0],
            "leaning the cap harder took the crossing's rounding from {} to {}, so it is not the \
             lean that opens the gap",
            pair[0],
            pair[1]
        );
    }
    assert!(
        roundings[0] > floors[0] * 1e8,
        "the crossing already rounds by {} against a floor of {}, so the gap is not a near thing",
        roundings[0],
        floors[0]
    );
}

/// The solid is not the thing that is wrong with any of this.
///
/// Each bar in the table is built by `Model::add_bar_cut_off_by` under
/// `Policy::certified()`, which writes **no provenance record at all** — nothing
/// about it was approximated — and `Model::validate` passes it certified as
/// well. The tolerance is asked for at mesh time and nowhere else.
#[test]
fn every_bar_in_the_table_is_built_and_checked_with_no_tolerance_asked_for() {
    for lean in LEANS {
        let mut log = ProvenanceLog::new();
        let (model, _) =
            a_bar_cut_off_almost_along_its_own_axis(lean, &Policy::certified(), &mut log);
        assert!(
            log.is_empty(),
            "building a bar leaning {lean:e} left {} provenance records",
            log.len()
        );
        model
            .validate(&Policy::certified(), &mut log)
            .unwrap_or_else(|why| panic!("a bar leaning {lean:e} is not sound: {why:?}"));
    }
}

/// The refusal and the wave-through are the same event, and the log says so.
///
/// The tolerant run records `room left to refine a walk`; the certified run at
/// the same deflection on the same bar records nothing, because it took the
/// other arm of the same `if`.
#[test]
fn the_walk_that_gave_up_is_the_one_the_log_names() {
    let (model, bar) = a_bar_cut_off_almost_along_its_own_axis(
        LEANS[0],
        &a_tolerant_caller(),
        &mut ProvenanceLog::new(),
    );
    let deflection = Deflection::of(DEFLECTIONS[0]).expect("a positive length");

    let mut waved_through = ProvenanceLog::new();
    model
        .solid(bar)
        .mesh(deflection, &a_tolerant_caller(), &mut waved_through)
        .expect_err("the leaning cap's ellipse cannot be walked this finely");
    assert!(
        waved_through
            .records()
            .iter()
            .any(|record| record.predicate() == ROOM_LEFT_TO_REFINE_A_WALK),
        "a tolerant caller's walk was emptied somewhere else than the site this file is about"
    );

    let mut refused = ProvenanceLog::new();
    model
        .solid(bar)
        .mesh(deflection, &Policy::certified(), &mut refused)
        .expect_err("a certified caller is never handed a walk it could not place");
    assert!(
        !refused
            .records()
            .iter()
            .any(|record| record.predicate() == ROOM_LEFT_TO_REFINE_A_WALK),
        "a certified caller was waved through the site it is supposed to be refused at"
    );
}

/// What a consumer that keeps an unrecognised refusal verbatim ends up filing.
///
/// `docs/consumers.md` records one that does exactly that. The certified
/// refusal's two field names are `asked` and `closest`, and the text a tolerant
/// caller is left holding has neither in it — so a report built from it cannot
/// say how fine a mesh was asked for or how close a point could be placed, which
/// is the whole of the cause.
#[test]
fn the_text_a_tolerant_caller_is_left_holding() {
    let (model, bar) = a_bar_cut_off_almost_along_its_own_axis(
        LEANS[0],
        &a_tolerant_caller(),
        &mut ProvenanceLog::new(),
    );

    let tolerant = model
        .solid(bar)
        .mesh(
            Deflection::of(DEFLECTIONS[0]).expect("a positive length"),
            &a_tolerant_caller(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("the leaning cap's ellipse cannot be walked this finely");
    let filed = format!("{tolerant:?}");

    assert!(
        filed.contains("TooFewCorners"),
        "the refusal a consumer copies is {filed}"
    );
    assert!(
        !filed.contains("asked") && !filed.contains("closest"),
        "{filed} carries a number the certified refusal carries, so a report built from it is \
         not missing the cause after all"
    );
}

/// A working size that accepts the wobble the widest leaning ellipse carries.
///
/// `Policy::tolerant(s)` accepts a wobble up to `64 · f64::EPSILON · s`, and the
/// widest crossing in [`LEANS`] rounds by about `5.7e-3` — measured, from the
/// `closest` the certified refusal carries — so this has to be a few times the
/// widest lean and it is fifty.
fn a_tolerant_caller() -> Policy {
    Policy::tolerant(50.0 * 1e11)
}

/// A bar whose top cap leans so far that its ellipse is worked out from numbers
/// billions of times the bar's own radius.
///
/// The bottom is an ordinary flat cap and stays walkable, which is what makes
/// the two closed edges of one solid answer differently.
fn a_bar_cut_off_almost_along_its_own_axis(
    lean: f64,
    building: &Policy,
    log: &mut ProvenanceLog,
) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let leaning = Plane::from_point_and_normal(
        Point::new(RADIUS, 0.0, AT_THE_SEAM),
        Vector::new(lean, 0.0, 1.0),
    )
    .expect("a direction with length");

    let bar = model
        .add_bar_cut_off_by(
            RADIUS,
            [flat, leaning],
            &Isometry::identity(),
            building,
            log,
        )
        .expect("a leaning plane cuts an ellipse right round a pipe");
    (model, bar)
}

/// The cap face whose plane leans, which is the one every refusal here names.
///
/// It is the flat face whose own corner sits higher up the seam, because the
/// other one is the bar's bottom.
fn the_cap_that_leans(model: &Model, bar: SolidKey) -> FaceKey {
    let how_high = |face: &rcad_topo::FaceView<'_>| {
        face.wires()
            .flat_map(|wire| wire.points())
            .map(|corner| corner.z())
            .fold(f64::MIN, f64::max)
    };
    model
        .solid(bar)
        .faces()
        .filter(|face| face.surface().as_plane().is_some())
        .max_by(|one, other| how_high(one).total_cmp(&how_high(other)))
        .expect("a bar has two flat caps")
        .key()
}
