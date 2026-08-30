//! The solid the parallel benchmark asks about, the points it asks about, and
//! the two ways of asking.
//!
//! Both bench targets share this, so the run that is timed and the run that is
//! checked bit for bit are the same work.

use std::f64::consts::TAU;

use rayon::prelude::*;
use rcad::{
    Containment, Isometry, Model, Point, Policy, ProfileBuilder, ProvenanceLog, Scale, SolidKey,
    SolidView,
};

/// Build a cog-shaped plate with round holes in it, and hand back the model
/// that owns it together with a key naming it.
///
/// Not a cube. A cube has twelve triangles, and classifying a point against
/// twelve triangles is over before the loop that drives it has warmed up —
/// which would make the benchmark a measurement of `rayon`'s scheduler rather
/// than of the kernel (decisions.md §9.2). This plate has 146 faces and 572
/// triangles, so a single query is a few thousand `orient3d` calls.
///
/// The teeth make the outline non-convex and the holes give it inner loops,
/// which is the same reason decisions.md §9.1 generates extruded polygons with
/// holes rather than convex hulls.
///
/// # Panics
///
/// If the profile it just described is not a sound one. That would be a bug in
/// the numbers below, not bad input from anywhere.
pub fn plate_with_holes() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(NOMINAL_SIZE).expect("100 mm is a fine size"));

    let mut outline = Vec::with_capacity(TEETH * 2);
    for corner in 0..TEETH * 2 {
        let angle = TAU * corner as f64 / (TEETH * 2) as f64;
        let radius = if corner % 2 == 0 {
            TOOTH_TIP_RADIUS
        } else {
            TOOTH_ROOT_RADIUS
        };
        outline.push([radius * angle.cos(), radius * angle.sin()]);
    }

    let mut builder = ProfileBuilder::new(outline);
    for hole in 0..HOLES {
        builder = builder.with_hole(hole_ring(hole));
    }
    let profile = builder
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the plate outline and its holes are a sound profile");

    let plate = model
        .add_extrusion(&profile, HEIGHT, &Isometry::identity())
        .expect("the plate has a positive height");
    (model, plate)
}

/// Roughly how big the plate is, which is what the model's default tolerances
/// are derived from (decisions.md §3.5).
const NOMINAL_SIZE: f64 = 100.0;

const TEETH: usize = 24;
const TOOTH_TIP_RADIUS: f64 = 50.0;
const TOOTH_ROOT_RADIUS: f64 = 40.0;
const HEIGHT: f64 = 20.0;

const HOLES: usize = 6;
const HOLE_CORNERS: usize = 16;
const HOLE_RADIUS: f64 = 6.0;
const HOLE_CIRCLE_RADIUS: f64 = 25.0;

/// One hole, as a sixteen-sided ring sitting on a circle of its own.
///
/// A hole reaches out to 31 mm and the tooth roots come in to 40 mm, so every
/// hole is clear of the outline; neighbouring hole centres are 25 mm apart and
/// the holes are 6 mm across, so they are clear of each other. `Profile::build`
/// checks both, and this is why it passes.
fn hole_ring(hole: usize) -> Vec<[f64; 2]> {
    let centre_angle = TAU * hole as f64 / HOLES as f64;
    let centre = [
        HOLE_CIRCLE_RADIUS * centre_angle.cos(),
        HOLE_CIRCLE_RADIUS * centre_angle.sin(),
    ];
    (0..HOLE_CORNERS)
        .map(|corner| {
            let angle = TAU * corner as f64 / HOLE_CORNERS as f64;
            [
                centre[0] + HOLE_RADIUS * angle.cos(),
                centre[1] + HOLE_RADIUS * angle.sin(),
            ]
        })
        .collect()
}

/// An evenly spaced grid of `points_per_side` cubed sample points spanning the
/// solid's bounding box, both faces of the box included.
///
/// Spanning the box exactly rather than overshooting it is deliberate. A point
/// outside the box is rejected by one comparison per axis, so a grid that
/// reached past the plate would be mostly free queries pretending to be work,
/// and the threads would spend their time on wildly uneven loads. Inside the
/// box the grid still gets all four answers: material, the air inside the holes
/// and between the teeth, and the caps and tooth tips the box touches.
///
/// # Panics
///
/// If asked for fewer than two points per side, which cannot span anything, or
/// if the solid has no faces to bound.
pub fn grid_over(solid: SolidView<'_>, points_per_side: usize) -> Vec<Point> {
    assert!(
        points_per_side >= 2,
        "a grid spanning a box needs at least both ends of it, got {points_per_side}"
    );
    let (low, high) = solid
        .bounding_box()
        .expect("the plate has faces to be bounded by");
    let low = low.coordinates();
    let high = high.coordinates();

    let last = (points_per_side - 1) as f64;
    let along =
        |axis: usize, index: usize| low[axis] + (high[axis] - low[axis]) * index as f64 / last;

    let mut points = Vec::with_capacity(points_per_side.pow(3));
    for x in 0..points_per_side {
        for y in 0..points_per_side {
            for z in 0..points_per_side {
                points.push(Point::new(along(0, x), along(1, y), along(2, z)));
            }
        }
    }
    points
}

/// Classify every point on one thread, in order.
pub fn classify_one_at_a_time(solid: SolidView<'_>, points: &[Point]) -> Vec<Answer> {
    let policy = Policy::certified();
    points
        .iter()
        .map(|&point| ask_about(solid, point, &policy))
        .collect()
}

/// Classify every point across whatever `rayon` pool is installed.
///
/// `solid` is a `Copy` view holding a single `&Model`, and every worker reads
/// through that same borrow. Nothing is cloned, nothing is built per thread and
/// nothing is locked — which is the entire claim roadmap §10 makes for
/// `&Model: Sync`, and the reason decisions.md §7.3 keeps `rayon` out of the
/// library. If the kernel needed any cooperation from inside to be shared this
/// way, this function could not be written.
pub fn classify_across_threads(solid: SolidView<'_>, points: &[Point]) -> Vec<Answer> {
    let policy = Policy::certified();
    points
        .par_iter()
        .map(|&point| ask_about(solid, point, &policy))
        .collect()
}

/// What one query answered, and what it had to write down on the way.
///
/// The provenance is carried alongside the answer because decisions.md §9.2
/// wants the bit-identity check to catch order-dependence "in the cascade or in
/// provenance recording", and comparing the answers alone would only cover the
/// first of those.
#[derive(Clone, Copy, Debug)]
pub struct Answer {
    containment: Containment,
    records: usize,
    worst_enclosure: f64,
}

fn ask_about(solid: SolidView<'_>, point: Point, policy: &Policy) -> Answer {
    let mut log = ProvenanceLog::new();
    let containment = solid.classify_point(point, policy, &mut log);
    Answer {
        containment,
        records: log.len(),
        worst_enclosure: log.worst_enclosure(),
    }
}

/// The first point the two runs disagree about, or `None` when every answer
/// matches bit for bit.
///
/// Bits, not `==`. `==` calls two `NaN` enclosures different when they are the
/// same number, and calls `-0.0` and `0.0` the same when they are not — either
/// way a real difference gets through, which would make this check worthless.
///
/// # Panics
///
/// If the two runs answered different numbers of questions.
pub fn first_difference(serial: &[Answer], parallel: &[Answer]) -> Option<usize> {
    assert_eq!(
        serial.len(),
        parallel.len(),
        "the two runs were asked about different numbers of points"
    );
    (0..serial.len()).find(|&index| bits_of(serial[index]) != bits_of(parallel[index]))
}

fn bits_of(answer: Answer) -> (u8, u64, usize, u64) {
    let (tag, enclosure) = match answer.containment {
        Containment::Inside => (0, 0),
        Containment::OnBoundary => (1, 0),
        Containment::Outside => (2, 0),
        Containment::Undecidable { enclosure } => (3, enclosure.to_bits()),
        // `Containment` grows a variant at every milestone. Comparing a variant
        // this does not know about would silently pass whatever it carries.
        other => panic!("Containment grew an answer this benchmark cannot compare: {other:?}"),
    };
    (
        tag,
        enclosure,
        answer.records,
        answer.worst_enclosure.to_bits(),
    )
}

/// How many points came out inside, on the boundary, outside, and undecided,
/// in that order.
///
/// A grid where everything came out the same way would benchmark one branch and
/// prove nothing, so both bench targets check this rather than assume it.
pub fn tally(answers: &[Answer]) -> [usize; 4] {
    let mut counts = [0usize; 4];
    for answer in answers {
        let (tag, ..) = bits_of(*answer);
        counts[tag as usize] += 1;
    }
    counts
}
