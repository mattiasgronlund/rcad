//! What a tolerant policy costs a mesh on a hard-leaning bar: **nothing.**
//!
//! This file used to pin a defect. A tolerant caller asking for a mesh of a bar
//! whose cap leans hard was handed `AWireDoesNotFanIntoALoopToMesh` with
//! `TooFewCorners { count: 1 }` — the name for a *malformed solid* — while a
//! certified caller at the same deflection on the same bar was told the truth by
//! `CannotPlaceAPointThatCloseToACrossing`. Twelve cells, twelve of each, and
//! the tolerant caller strictly worse off in every one: same absence of a mesh,
//! worse account of why.
//!
//! The old file said out loud what would close that window:
//!
//! > If `SolidView::room_a_mesh_has_to_work_in` is ever changed to charge a
//! > face's crossings rather than only its corners, the window these twelve
//! > cells sit in closes and this file has to move — which would be the fix for
//! > what it pins, not a regression.
//!
//! That is what happened, and this is the file moved. The mesher now charges a
//! curved face for how far its **edges** run rather than for where its corners
//! sit, so the floor under a leaning bar climbs with the ellipse the cap cuts.
//! Every one of the twelve deflections below is under that floor, and both
//! callers are turned away by [`Error::DeflectionBelowWhatAMeshCanReach`] before
//! the walk is ever reached — **the same refusal, carrying the same two
//! numbers**.
//!
//! # What each test now holds down
//!
//! - [`a_tolerant_caller_is_told_exactly_what_a_certified_one_is_told`] — the
//!   twelve cells, with the two refusals compared field by field.
//! - [`the_floor_under_a_deflection_runs_away_with_the_lean_it_stands_for`] —
//!   the floor is no longer stuck: it tracks the ellipse over a factor of
//!   twenty-five of lean.
//! - [`every_bar_in_the_table_is_built_and_checked_with_no_tolerance_asked_for`]
//!   — unchanged, and still the reason none of this is about a broken solid.
//! - [`nothing_is_waved_through_before_the_floor_turns_a_caller_away`] — the
//!   provenance log of either run is empty, so nothing was approximated on the
//!   way to the refusal. This is where `room left to refine a walk` used to be
//!   recorded for the tolerant caller.
//! - [`the_text_a_tolerant_caller_is_left_holding`] — `docs/consumers.md`'s
//!   consumer keeps an unrecognised refusal's debug text verbatim, and the text
//!   it now files carries both numbers instead of a corner count.
//!
//! # Where the emptied walk went
//!
//! Reaching `rcad-geom`'s `room_left_over` on a closed edge needed the
//! crossing's own rounding to beat the floor `SolidView::mesh` puts under a
//! deflection. On a bar cut by a leaning plane those two now grow together, so
//! this shape no longer reaches it. That is a statement about **this shape at
//! these leans**, measured below, and not a claim that nothing reaches that site
//! — `rcad-geom`'s own tests drive it directly, and a surface whose reach is
//! still charged from its corners could reach it from here.

use rcad_geom::{Deflection, Isometry, Plane, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, SolidKey};

/// The radius of every bar in the table, and the seam is at `x = RADIUS, y = 0`.
const RADIUS: f64 = 1.0;

/// Where the leaning cap crosses the seam, which is the only corner it puts on
/// the wall.
const AT_THE_SEAM: f64 = 0.5;

/// How far each top cap leans, as the ratio of its normal across the bar's axis
/// to its normal along it.
///
/// Four leans spanning a factor of twenty-five, so that no cell below is the
/// only one that works. The ellipse the cap cuts climbs to
/// `AT_THE_SEAM + 2 · lean · RADIUS` up the bar's axis, and the floor climbs
/// with it — from `0.19395282244028492` at the first of these to
/// `4.848820560134335` at the last.
const LEANS: [f64; 4] = [4e9, 1e10, 3e10, 1e11];

/// Deflections a drawing would call fine, and every one of them under the floor
/// of every bar in [`LEANS`].
///
/// The coarsest is a thousandth of a millimetre and the lowest floor in the
/// table is `0.19395282244028492`, so the table clears it by two orders of
/// magnitude with nothing near the edge.
const DEFLECTIONS: [f64; 3] = [1e-4, 3e-4, 1e-3];

/// The number of (lean, deflection) pairs, checked against what the sweep
/// actually visits so a table trimmed by accident cannot pass quietly.
const CELLS: usize = LEANS.len() * DEFLECTIONS.len();

/// The record `room_left_over` pushes when it waves a tolerant caller through.
///
/// Written out rather than imported from `rcad_geom` so that the thing under
/// test does not get to say what the test expects. Nothing here should push it
/// any more, and [`nothing_is_waved_through_before_the_floor_turns_a_caller_away`]
/// is what says so.
const ROOM_LEFT_TO_REFINE_A_WALK: &str = "room left to refine a walk";

/// How far apart the twelve floors are allowed to be once each is divided by
/// the ellipse it stands for.
///
/// The four come out between `2.4244102800671675e-11` and
/// `2.4244102805103595e-11` of their own ellipse, which is one part in five
/// thousand million, so this is loose by a factor of fifty and still nowhere
/// near the factor of twenty-five the old defect let through.
const HOW_FAR_APART_THE_FOUR_RATIOS_MAY_BE: f64 = 1e-8;

/// Every cell: the two callers are told the same thing, and it is the truth.
///
/// The refusal names the deflection that was asked for and the finest one this
/// solid can be walked at. A tolerant policy buys nothing here and costs
/// nothing either — which is the whole of what changed.
#[test]
fn a_tolerant_caller_is_told_exactly_what_a_certified_one_is_told() {
    let mut visited = 0;
    for lean in LEANS {
        let (model, bar) = a_bar_cut_off_almost_along_its_own_axis(
            lean,
            &a_tolerant_caller(),
            &mut ProvenanceLog::new(),
        );
        for asked in DEFLECTIONS {
            let deflection = Deflection::of(asked).expect("a positive length");

            let certified = model.solid(bar).expect("this model made the key").mesh(
                deflection,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            let Err(Error::DeflectionBelowWhatAMeshCanReach {
                asked: certified_asked,
                closest: certified_floor,
            }) = certified
            else {
                panic!(
                    "a bar leaning {lean:e} meshed at {asked:e} by a certified caller: \
                     {certified:?}"
                );
            };

            let tolerant = model.solid(bar).expect("this model made the key").mesh(
                deflection,
                &a_tolerant_caller(),
                &mut ProvenanceLog::new(),
            );
            let Err(Error::DeflectionBelowWhatAMeshCanReach {
                asked: tolerant_asked,
                closest: tolerant_floor,
            }) = tolerant
            else {
                panic!(
                    "a bar leaning {lean:e} meshed at {asked:e} by a tolerant caller: {tolerant:?}"
                );
            };

            assert_eq!(
                (tolerant_asked, tolerant_floor),
                (certified_asked, certified_floor),
                "a bar leaning {lean:e} at {asked:e}: the tolerant caller is told \
                 ({tolerant_asked}, {tolerant_floor}) where the certified one is told \
                 ({certified_asked}, {certified_floor})"
            );
            assert_eq!(
                certified_asked, asked,
                "the refusal quotes back {certified_asked} where {asked:e} was asked for"
            );
            assert!(
                certified_floor > certified_asked,
                "a bar leaning {lean:e}: the floor is {certified_floor} and {certified_asked} was \
                 asked for, so this cell is not one the floor turns away"
            );

            visited += 1;
        }
    }
    assert_eq!(visited, CELLS, "every cell of the table has to be walked");
}

/// The number that used to sit still, measured running.
///
/// The floor was charged from the face's *corners*, and the seam corner of a bar
/// cut this way stays where it always was however far the cap leans — so over
/// these four leans the floor did not move off `6.143234069592533e-13` while the
/// crossing it was supposed to stand for ran from `2.3e-4` to `5.7e-3`. It is
/// charged from the face's *edges* now, and an edge of the leaning cap is the
/// ellipse.
///
/// The oracle is that ellipse: it climbs `AT_THE_SEAM + 2 · lean · RADIUS` up
/// the bar's own axis, so the floor divided by that has to come to the same
/// number at every lean. It does, to nine figures, over a factor of
/// twenty-five.
///
/// The floor is read off the public refusal's `closest` and nothing else. It is
/// not hunted for by bisection here, because a deflection this bar *would*
/// accept is coarser than a whole millimetre and finer than the bar is long: a
/// bisection that ever lands above the floor meshes a solid four thousand
/// million units tall, which is not a thing to do in a test.
#[test]
fn the_floor_under_a_deflection_runs_away_with_the_lean_it_stands_for() {
    let mut floors = Vec::new();
    for lean in LEANS {
        let (model, bar) = a_bar_cut_off_almost_along_its_own_axis(
            lean,
            &a_tolerant_caller(),
            &mut ProvenanceLog::new(),
        );
        let refused = model.solid(bar).expect("this model made the key").mesh(
            Deflection::of(DEFLECTIONS[0]).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let Err(Error::DeflectionBelowWhatAMeshCanReach { closest, .. }) = refused else {
            panic!("a bar leaning {lean:e} was refused with {refused:?}");
        };
        println!("a bar leaning {lean:e} has a floor of {closest:e}");
        floors.push(closest);
    }

    for pair in floors.windows(2) {
        assert!(
            pair[1] > pair[0],
            "leaning the cap harder left the floor at {} and then {}, so the floor is still not \
             following the ellipse it stands for",
            pair[0],
            pair[1]
        );
    }

    let ratios: Vec<f64> = LEANS
        .iter()
        .zip(&floors)
        .map(|(lean, floor)| floor / (AT_THE_SEAM + 2.0 * lean * RADIUS))
        .collect();
    let smallest = ratios.iter().copied().fold(f64::INFINITY, f64::min);
    let largest = ratios.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        largest - smallest < smallest * HOW_FAR_APART_THE_FOUR_RATIOS_MAY_BE,
        "the floor comes to between {smallest:e} and {largest:e} of the ellipse it stands for \
         across these four leans, which is not one number"
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

/// The refusal arrives before anything has been approximated, for either
/// caller.
///
/// This is where the defect lived. The tolerant run used to be waved through
/// `room_left_over` with `room left to refine a walk` in its log and an emptied
/// crossing in its hands; the certified run took the other arm of the same `if`
/// and recorded nothing. Now the floor turns both away first, so **both logs
/// are empty** — an exact count, not an absence of one string.
#[test]
fn nothing_is_waved_through_before_the_floor_turns_a_caller_away() {
    let (model, bar) = a_bar_cut_off_almost_along_its_own_axis(
        LEANS[0],
        &a_tolerant_caller(),
        &mut ProvenanceLog::new(),
    );
    let deflection = Deflection::of(DEFLECTIONS[0]).expect("a positive length");

    for (who, policy) in [
        ("a tolerant caller", a_tolerant_caller()),
        ("a certified caller", Policy::certified()),
    ] {
        let mut log = ProvenanceLog::new();
        let refused = model
            .solid(bar)
            .expect("this model made the key")
            .mesh(deflection, &policy, &mut log);
        assert!(
            matches!(refused, Err(Error::DeflectionBelowWhatAMeshCanReach { .. })),
            "{who} asking below the floor was refused with {refused:?}"
        );
        assert_eq!(
            log.len(),
            0,
            "{who} left {} provenance records on the way to the floor's refusal, the first of \
             them {:?}",
            log.len(),
            log.records().first().map(|record| record.predicate())
        );
        assert!(
            !log.records()
                .iter()
                .any(|record| record.predicate() == ROOM_LEFT_TO_REFINE_A_WALK),
            "{who} was waved through the walk this file used to be about"
        );
    }
}

/// What a consumer that keeps an unrecognised refusal verbatim ends up filing.
///
/// `docs/consumers.md` records one that does exactly that. It used to file a
/// corner count and a face key — neither of the two numbers that say what
/// happened. It now files both of them, under the names the variant gives them.
#[test]
fn the_text_a_tolerant_caller_is_left_holding() {
    let (model, bar) = a_bar_cut_off_almost_along_its_own_axis(
        LEANS[0],
        &a_tolerant_caller(),
        &mut ProvenanceLog::new(),
    );

    let tolerant = model
        .solid(bar)
        .expect("this model made the key")
        .mesh(
            Deflection::of(DEFLECTIONS[0]).expect("a positive length"),
            &a_tolerant_caller(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("this deflection is under the floor of a bar leaning this hard");
    let filed = format!("{tolerant:?}");

    assert!(
        filed.contains("asked") && filed.contains("closest"),
        "the refusal a consumer copies is {filed}, which does not carry the cause"
    );
    assert!(
        !filed.contains("TooFewCorners"),
        "{filed} still blames the shape of a wire for a deflection that was too fine"
    );
}

/// A working size that accepts the wobble the widest leaning ellipse carries.
///
/// `Policy::tolerant(s)` accepts a wobble up to `64 · f64::EPSILON · s`, and the
/// widest cap in [`LEANS`] is cut from numbers a hundred thousand million times
/// the bar's radius, so this has to be a few times the widest lean and it is
/// fifty. It is the policy the bars are *built* under; what the mesh calls do
/// with it is the subject of this file.
fn a_tolerant_caller() -> Policy {
    Policy::tolerant(50.0 * 1e11)
}

/// A bar whose top cap leans so far that its ellipse is worked out from numbers
/// billions of times the bar's own radius.
///
/// The bottom is an ordinary flat cap, so the two closed edges of one solid are
/// charged very differently — and it is the leaning one that sets the floor.
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
