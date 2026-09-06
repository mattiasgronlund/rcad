//! `rscene` counts how often a tolerant policy waved a walk through by
//! matching [`rcad::Record::predicate`] against
//! [`rcad::ROOM_LEFT_TO_REFINE_A_WALK`] — not against a copy of the string
//! `rcad-geom` happens to use today. `rscene` depends on the `rcad` facade
//! only, so if this crate ever stopped re-exporting the name, a rename inside
//! `rcad-geom` would break nothing here and `rscene`'s counter would silently
//! start matching zero records instead of failing to build.
//!
//! This file plays `rscene`'s part: it forces the one situation that pushes a
//! record under that name — a tolerant caller asking for a walk finer than any
//! point on the crossing can deliver — and then finds that record the same
//! way a counting consumer would, by the exported name and nothing else.
//! Delete the re-export and this file stops compiling; that is the point of
//! it.

use rcad::{
    Axis, Cylinder, Deflection, EdgeRun, HalfSpace, NamedRun, Picker, Plane, Point, Policy,
    PredicatePoint, ProvenanceLog, Record, Surface, Vector, walk,
};

/// Large enough that the points `walk` would place along it sit further off
/// the true curve than any deflection this file asks for, which is what
/// forces the wave-through this file is about.
const RADIUS: f64 = 1e6;

/// A tolerant caller whose working size clears the crossing's own deviation
/// with room to spare, so the wave-through is accepted rather than refused.
fn a_tolerant_caller() -> Policy {
    Policy::tolerant(16.0 * RADIUS)
}

/// A half-circle where a plane through the origin cuts a pipe lying along the
/// `z` axis — the same shape `walk` is asked to refine.
fn the_half_circle(
    deflection: Deflection,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<rcad::PointsAlongACrossing, rcad::GeometryError> {
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("an axis direction has length");
    let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), RADIUS).expect("a positive radius"));
    let run = NamedRun::new(
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(flat),
            picked_by: [Picker::from(
                HalfSpace::through_a_point_facing(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                    .expect("an axis direction has length"),
            )]
            .into_iter()
            .collect(),
        },
        [[RADIUS, 0.0, 0.0], [-RADIUS, 0.0, 0.0]].map(PredicatePoint::from),
    )
    .expect("the two ends of a half circle do not have to meet");
    walk(
        &run,
        [&Surface::Plane(flat), &pipe],
        deflection,
        policy,
        log,
    )
}

/// The scenario a real STEP import hits: a tolerant caller asks for a
/// deflection nothing on this crossing can meet, gets waved through with the
/// two ends and nothing between them, and the wave-through is filed under
/// [`rcad::ROOM_LEFT_TO_REFINE_A_WALK`].
///
/// A caller counting these — as `rscene` does — reads the log by that name.
/// Comparing against a copied string literal here would defeat the whole
/// point, so this test never writes the string out; it only ever names
/// [`rcad::ROOM_LEFT_TO_REFINE_A_WALK`].
#[test]
fn a_facade_caller_can_find_a_wave_through_by_the_facades_own_name() {
    let mut log = ProvenanceLog::new();

    // First, a coarse deflection that a walk this size can actually meet —
    // this measures how far off the crossing a placed point sits, without
    // assuming a specific multiple of `POINT_ALONG_A_CROSSING_ROUNDOFF`.
    let coarse = the_half_circle(
        Deflection::of(RADIUS / 8.0).expect("a positive length"),
        &a_tolerant_caller(),
        &mut log,
    )
    .expect("a deflection an eighth of the radius leaves room to spare");
    let deviation = coarse.deviation().magnitude();
    assert!(
        deviation > 0.0 && deviation.is_finite(),
        "a point on a circle of radius {RADIUS} sits some way off it, and this said {deviation}"
    );

    // Now ask for half of what the crossing can deliver: nothing can meet it,
    // so a tolerant caller is waved through instead of refused.
    let mut wave_through_log = ProvenanceLog::new();
    let waved = the_half_circle(
        Deflection::of(deviation / 2.0).expect("half a positive length is positive"),
        &a_tolerant_caller(),
        &mut wave_through_log,
    )
    .expect("a tolerant caller is waved through rather than refused");
    assert!(
        waved.points().is_empty(),
        "the wave-through this file is about comes back with nothing between the two ends"
    );

    let matches_by_the_facades_name: Vec<&Record> = wave_through_log
        .records()
        .iter()
        .filter(|record| record.predicate() == rcad::ROOM_LEFT_TO_REFINE_A_WALK)
        .collect();

    assert_eq!(
        matches_by_the_facades_name.len(),
        1,
        "one wave-through is one record filed under rcad::ROOM_LEFT_TO_REFINE_A_WALK, and the \
         log holds {wave_through_log:?}"
    );

    // A record under any other name in the same log is not what a counting
    // consumer is after, so the match above has to be exact rather than "the
    // log is non-empty".
    assert_eq!(
        wave_through_log.records().len(),
        1,
        "nothing else was pushed for this walk, so a consumer matching by name alone gets the \
         right count: {wave_through_log:?}"
    );
}
