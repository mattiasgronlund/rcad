//! A walk that comes back with nothing is not a boundary a sign may be read off.
//!
//! [`winding_of`](rcad_topo::Model::validate) winds the ring a wire traces — its
//! corners **plus** the points `rcad_geom::walk` puts along each curved step —
//! because the corners alone got `a_face_bounded_by_two_half_circles.rs`
//! backwards. There is one way back to the corners alone, and this file drives
//! it:
//!
//! 1. `walk` asks `room_left_over` how much of the deflection is left once the
//!    points' own deviation is paid for.
//! 2. When there is none left, a **certified** caller is refused
//!    (`CannotPlaceAPointThatCloseToACrossing`) and a **tolerant** one that
//!    accepts a wobble that big gets `Ok` with an empty point list.
//! 3. The ring is then the wire's corners and nothing else.
//! 4. The margin the sign has to beat is then `2·perimeter·deviation` and
//!    nothing else, because the deflection halving away is the only other term
//!    in it — so the sign is certified off the corners, which is the whole of
//!    the bug the rewrite was for.
//!
//! Nothing about that is a corner case of the policy: `Policy::tolerant(s)`
//! accepts the wobble whenever the crossing's own size is under `s/4`, and the
//! STEP door is driven with a tolerant policy.
//!
//! # The shape
//!
//! A sliver: a shallow arc of a very large circle, closed underneath by three
//! straights that stay just inside it. Its own area is real but small, and the
//! polygon through its four corners has **more** area than it and the opposite
//! sign — the corners cut the shelf's rectangle, the arc bulges further. So the
//! two answers are not the same answer, and which one is given can be seen.
//!
//! Every number in it is derived from the two the file picks, `RADIUS` and how
//! many deviations of sagitta to draw: they put the deflection at which the walk
//! empties in between the passes where the ring is still too thin to answer for.

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, HalfSpace, NamedRun, Orientation, Picker, Plane, Point,
    Scale, Surface, Vector,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{DraftEdge, DraftStep, DraftWire, Model, Result, SolidBuilder, SolidKey};

/// The circle the arc is cut from. Large enough that the points along it are
/// worked out from numbers a million times what the sliver itself spans, which
/// is what makes their deviation big enough to matter.
const RADIUS: f64 = 1e6;

/// How far the arc stands off its chord, counted in the deviation a point along
/// it carries.
///
/// It has a window either side of it and the tests above are what says so: too
/// small and the corner polygon stops beating the margin, too large and the
/// sliver's own area answers before the walk runs out of room. Thirty sits in
/// the middle of it, about twice clear of each end.
const SAGITTAS_OF_DEVIATION: f64 = 30.0;

const THICKNESS: f64 = 1.0;

/// A working size four times the crossing's own, which is the whole of what
/// [`Policy::wobble_is_acceptable`](rcad_predicates::Policy) needs to accept a
/// walk that cannot be refined.
fn a_tolerant_caller() -> Policy {
    Policy::tolerant(4.0 * RADIUS)
}

/// The sentence this lane earns its fix with.
///
/// The sliver is wound one way by the boundary it traces and the other way by
/// the four corners the model stores — `the_walk_comes_back_with_no_points_and_
/// the_corners_wind_the_other_way` below measures both. A certified caller is
/// told the arithmetic cannot see it. A tolerant one used to be told something
/// about the *shape* instead: that its outer wire winds against its normal,
/// which is the refusal `a_face_bounded_by_two_half_circles.rs` exists to have
/// stopped.
#[test]
fn a_sliver_no_walk_could_see_is_not_wound_off_its_corners() {
    let policy = a_tolerant_caller();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());

    let refused = a_sliver(&mut model, Vector::new(0.0, 0.0, -1.0), &policy, &mut log);

    assert!(
        matches!(refused, Err(rcad_topo::Error::Undecidable { .. })),
        "a walk that came back with nothing decides nothing, and this said {refused:?}"
    );
}

/// And a certified caller gets the same answer, which is the point.
///
/// Whether a policy accepts a wobble says what a *predicate* may settle. It does
/// not say that a step nobody sampled may be wound as if it were straight, so
/// the two policies have to agree here.
#[test]
fn a_certified_caller_is_told_the_same_thing_about_the_same_sliver() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());

    let refused = a_sliver(&mut model, Vector::new(0.0, 0.0, -1.0), &policy, &mut log);

    assert!(
        matches!(refused, Err(rcad_topo::Error::Undecidable { .. })),
        "got {refused:?}"
    );
}

/// The evidence, measured rather than argued.
///
/// This walks the sliver's arc exactly as `the_ring_a_wire_traces` does, at the
/// deflections `winding_of` halves its way through, and reports three things:
///
/// - **the sliver really is wound the other way** — a ring of ten thousand
///   points read straight off the circle sweeps a negative area, and so does
///   every ring the walk hands back while it is still handing back points;
/// - **the walk empties** — at the first deflection under the points' own
///   deviation it comes back `Ok` with no points at all;
/// - **and the corners alone then beat the margin**, with the other sign.
///
/// It asks `rcad_geom::walk` and not `Model::validate`, so it says the same
/// thing before and after the fix: this is what the shape is, and the tests
/// above are what the sweep makes of it.
#[test]
fn the_walk_comes_back_with_no_points_and_the_corners_wind_the_other_way() {
    let policy = a_tolerant_caller();
    let mut log = ProvenanceLog::new();
    let corners = the_slivers_corners();

    assert!(
        area_swept_by(&a_ring_read_straight_off_the_circle()) < 0.0,
        "the sliver itself winds clockwise"
    );
    assert!(
        area_swept_by(&corners) > 0.0,
        "and the polygon through its corners winds the other way"
    );

    let mut room = how_far_across(&corners);
    let mut last_ring_the_walk_could_draw = None;
    let emptied = loop {
        let deflection = Deflection::of(room).expect("halving a positive length stays positive");
        let walked = rcad_geom::walk(
            &the_arc_as_a_run(),
            [
                &Surface::Plane(a_plane_at(0.0, Vector::new(0.0, 0.0, -1.0))),
                &the_pipe_the_arc_runs_on(),
            ],
            deflection,
            &policy,
            &mut log,
        )
        .expect("a tolerant caller is never refused this walk");

        let ring = the_ring_this_walk_traces(&walked);
        let hidden =
            2.0 * how_far_round(&ring) * (deflection.length() + walked.deviation().magnitude());
        assert!(
            area_swept_by(&ring).abs() <= hidden || walked.points().is_empty(),
            "no walk that put points down was ever allowed to answer, and one at {} was",
            deflection.length()
        );
        if walked.points().is_empty() {
            break (deflection, ring, hidden);
        }
        assert!(
            area_swept_by(&ring) < 0.0 || walked.points().len() < 2,
            "every ring drawn from two points or more winds clockwise"
        );
        last_ring_the_walk_could_draw = Some(area_swept_by(&ring));
        room /= 2.0;
    };

    let (deflection, ring, hidden) = emptied;
    assert_eq!(ring, corners, "the ring collapsed to the corners");
    assert!(
        last_ring_the_walk_could_draw.expect("the walk drew rings before it emptied") < 0.0,
        "the last ring the walk could draw winds clockwise"
    );
    assert!(
        area_swept_by(&ring) > hidden,
        "and the corners beat the margin the other way: {} against {hidden} at a deflection of {}",
        area_swept_by(&ring),
        deflection.length()
    );
}

/// The pair `winding_of`'s rustdoc used to be justified by, and cannot happen.
///
/// It said a half-circle of radius `1` walked into one inside point encloses
/// less than one of radius `0.99` walked into two, and that a half-pipe's cap is
/// that pair. The arithmetic is right and the pair is not reachable:
/// [`rcad_geom::walk`] sets its widest step from `room / reach` with
/// `room = deflection − deviation`, so a bigger circle drawn in the same place
/// gets a smaller widest step and at least as many pieces.
///
/// Swept over ascending radii at each of a spread of deflections, through the
/// real walk and not a model of it.
#[test]
fn a_bigger_arc_is_never_walked_into_fewer_pieces_than_a_smaller_one() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut asked = 0;

    for coarseness in 0..24 {
        let deflection = Deflection::of(2.0_f64.powi(-coarseness)).expect("a positive length");
        let mut fewest_so_far = 0;
        for step in 0..64 {
            let radius = 0.25 * f64::from(step + 1);
            let Ok(walked) = a_half_circle_of_radius(radius, deflection, &policy, &mut log) else {
                continue;
            };
            assert!(
                walked >= fewest_so_far,
                "a half-circle of radius {radius} was walked into {walked} pieces at a deflection \
                 of {}, where a smaller one got {fewest_so_far}",
                deflection.length()
            );
            fewest_so_far = walked;
            asked += 1;
        }
    }
    assert!(asked > 1000, "only {asked} pairs were reachable");
}

/// How many points a walk puts inside the half of a circle of `radius` that lies
/// over the `x` axis.
fn a_half_circle_of_radius(
    radius: f64,
    deflection: Deflection,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<usize> {
    let flat = a_plane_at(0.0, Vector::new(0.0, 0.0, -1.0));
    let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a positive radius"));
    let run = NamedRun::new(
        the_arc_on(&flat),
        [[radius, 0.0, 0.0], [-radius, 0.0, 0.0]].map(PredicatePoint::from),
    )?;
    Ok(rcad_geom::walk(
        &run,
        [&Surface::Plane(flat), &pipe],
        deflection,
        policy,
        log,
    )?
    .points()
    .len())
}

fn a_working_size() -> Scale {
    Scale::millimetres(4.0).expect("4 mm is a working size")
}

/// How far the arc stands off its own chord.
fn sagitta() -> f64 {
    SAGITTAS_OF_DEVIATION * rcad_geom::POINT_ALONG_A_CROSSING_ROUNDOFF * RADIUS
}

/// Half the chord the arc is drawn on, which is what `sagitta = half²/(2·RADIUS)`
/// makes it.
fn half_chord() -> f64 {
    (2.0 * RADIUS * sagitta()).sqrt()
}

/// How high the shelf under the arc stands: just inside it, at a third of the
/// way out, where the arc is still `8/9` of its sagitta up.
fn shelf() -> f64 {
    0.95 * sagitta() * (1.0 - 1.0 / 9.0)
}

/// The four corners in the order the bottom cap's wire walks them.
fn the_slivers_corners() -> Vec<[f64; 3]> {
    let (half, shelf) = (half_chord(), shelf());
    vec![
        [-half, 0.0, 0.0],
        [half, 0.0, 0.0],
        [half / 3.0, shelf, 0.0],
        [-half / 3.0, shelf, 0.0],
    ]
}

/// The same wire drawn from the circle itself, finely enough that what it
/// sweeps is the sliver's own area and not an artefact of the drawing.
fn a_ring_read_straight_off_the_circle() -> Vec<[f64; 3]> {
    const HOW_MANY: usize = 10_000;
    let corners = the_slivers_corners();
    let centre = sagitta() - RADIUS;
    let opening = (half_chord() / RADIUS).asin();
    let mut ring = vec![corners[0]];
    for step in 1..HOW_MANY {
        let angle = -opening + 2.0 * opening * (step as f64) / (HOW_MANY as f64);
        ring.push([RADIUS * angle.sin(), centre + RADIUS * angle.cos(), 0.0]);
    }
    ring.extend_from_slice(&corners[1..]);
    ring
}

/// The corners with whatever the walk put along the arc between the first two,
/// which is what `the_ring_a_wire_traces` builds.
fn the_ring_this_walk_traces(walked: &rcad_geom::PointsAlongACrossing) -> Vec<[f64; 3]> {
    let corners = the_slivers_corners();
    let mut ring = vec![corners[0]];
    ring.extend(walked.points().iter().map(Point::coordinates));
    ring.extend_from_slice(&corners[1..]);
    ring
}

/// Twice the area this ring sweeps about `z`, halved: positive counter-clockwise.
fn area_swept_by(ring: &[[f64; 3]]) -> f64 {
    let count = ring.len();
    (0..count)
        .map(|index| {
            let (from, to) = (ring[index], ring[(index + 1) % count]);
            from[0] * to[1] - to[0] * from[1]
        })
        .sum::<f64>()
        / 2.0
}

/// The same `|dx| + |dy| + |dz|` perimeter the margin is built on.
fn how_far_round(ring: &[[f64; 3]]) -> f64 {
    let count = ring.len();
    (0..count)
        .map(|index| {
            let (from, to) = (ring[index], ring[(index + 1) % count]);
            (0..3)
                .map(|axis| (to[axis] - from[axis]).abs())
                .sum::<f64>()
        })
        .sum()
}

/// The longest side of the box the ring sits in, which is where the halving
/// starts.
fn how_far_across(ring: &[[f64; 3]]) -> f64 {
    (0..3)
        .map(|axis| {
            let reach = ring.iter().map(|point| point[axis]);
            reach.clone().fold(f64::MIN, f64::max) - reach.fold(f64::MAX, f64::min)
        })
        .fold(0.0, f64::max)
}

fn the_arc_as_a_run() -> NamedRun {
    let corners = the_slivers_corners();
    NamedRun::new(
        the_arc_on(&a_plane_at(0.0, Vector::new(0.0, 0.0, -1.0))),
        [corners[0], corners[1]].map(PredicatePoint::from),
    )
    .expect("the arc's two ends are on it")
}

/// The sliver, face by face.
///
/// Eight corners, twelve edges, six faces: the two caps, the wall the arc
/// sweeps, and one for each of the three straights under it.
fn a_sliver(
    model: &mut Model,
    ground_faces: Vector,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();

    let ground = a_plane_at(0.0, ground_faces);
    let lid = a_plane_at(THICKNESS, ground_faces * -1.0);

    let profile = the_slivers_corners();
    let mut at = Vec::with_capacity(8);
    for height in [0.0, THICKNESS] {
        for corner in [profile[0], profile[3], profile[2], profile[1]] {
            at.push(builder.add_vertex(Point::new(corner[0], corner[1], height))?);
        }
    }

    let edges = [
        (0, 1, EdgeRun::Straight),
        (1, 2, EdgeRun::Straight),
        (2, 3, EdgeRun::Straight),
        (3, 0, the_arc_on(&ground)),
        (4, 5, EdgeRun::Straight),
        (5, 6, EdgeRun::Straight),
        (6, 7, EdgeRun::Straight),
        (7, 4, the_arc_on(&lid)),
        (0, 4, EdgeRun::Straight),
        (1, 5, EdgeRun::Straight),
        (2, 6, EdgeRun::Straight),
        (3, 7, EdgeRun::Straight),
    ];
    let mut edge = Vec::with_capacity(edges.len());
    for (from, to, run) in edges {
        edge.push(builder.add_edge(at[from], at[to], run)?);
    }

    let walked = |builder: &mut SolidBuilder, steps: &[(usize, Orientation)]| {
        let steps: Vec<(DraftEdge, Orientation)> = steps
            .iter()
            .map(|&(which, orientation)| (edge[which], orientation))
            .collect();
        a_wire(builder, &steps)
    };

    let below = walked(
        &mut builder,
        &[
            (3, Orientation::Reversed),
            (2, Orientation::Reversed),
            (1, Orientation::Reversed),
            (0, Orientation::Reversed),
        ],
    )?;
    let above = walked(
        &mut builder,
        &[
            (4, Orientation::Forward),
            (5, Orientation::Forward),
            (6, Orientation::Forward),
            (7, Orientation::Forward),
        ],
    )?;
    let round_the_arc = walked(
        &mut builder,
        &[
            (3, Orientation::Forward),
            (8, Orientation::Forward),
            (7, Orientation::Reversed),
            (11, Orientation::Reversed),
        ],
    )?;

    let mut faces = vec![
        builder.add_face(Surface::Plane(ground), &[below])?,
        builder.add_face(Surface::Plane(lid), &[above])?,
        builder.add_face(the_pipe_the_arc_runs_on(), &[round_the_arc])?,
    ];
    for step in 0..3 {
        let wire = walked(
            &mut builder,
            &[
                (step, Orientation::Forward),
                (step + 9, Orientation::Forward),
                (step + 4, Orientation::Reversed),
                (step + 8, Orientation::Reversed),
            ],
        )?;
        faces.push(builder.add_face(Surface::Plane(a_wall_under(step)), &[wire])?);
    }

    let skin: Vec<_> = faces
        .into_iter()
        .map(|face| (face, Orientation::Forward))
        .collect();
    builder.add_shell(&skin)?;
    builder.build(model, policy, log)
}

fn a_plane_at(height: f64, facing: Vector) -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, height), facing)
        .expect("an axis direction has length")
}

/// The wall under the `step`th straight of the shelf, facing out of the sliver.
///
/// The shelf is walked with the material on its left, so out of it is the
/// quarter turn to the right.
fn a_wall_under(step: usize) -> Plane {
    let profile = the_slivers_corners();
    let walked = [profile[0], profile[3], profile[2], profile[1]];
    let (from, to) = (walked[step], walked[step + 1]);
    Plane::from_point_and_normal(
        Point::new(from[0], from[1], 0.0),
        Vector::new(to[1] - from[1], from[0] - to[0], 0.0),
    )
    .expect("a wall of the shelf has length")
}

/// The half of the circle `plane` cuts that lies over the `x` axis.
fn the_arc_on(plane: &Plane) -> EdgeRun {
    EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(*plane),
        picked_by: smallvec::smallvec![Picker::from(
            HalfSpace::through_a_point_facing(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("an axis direction has length")
        )],
    }
}

/// The cylinder the arc is cut out of: the sliver lies inside it.
fn the_pipe_the_arc_runs_on() -> Surface {
    Surface::Cylinder(
        Cylinder::new(
            Axis::new(
                Point::new(0.0, sagitta() - RADIUS, 0.0),
                Vector::new(0.0, 0.0, 1.0),
            )
            .expect("an axis direction has length"),
            RADIUS,
        )
        .expect("a positive radius"),
    )
}

fn a_wire(builder: &mut SolidBuilder, steps: &[(DraftEdge, Orientation)]) -> Result<DraftWire> {
    let mut walked: Vec<DraftStep> = Vec::with_capacity(steps.len());
    for &(edge, orientation) in steps {
        walked.push(DraftStep::Along(builder.add_coedge(edge, orientation)?));
    }
    builder.add_wire(&walked)
}
