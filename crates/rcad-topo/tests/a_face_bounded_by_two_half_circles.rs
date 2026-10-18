//! A flat face whose whole area is in how its two arcs bulge.
//!
//! The half of §44.11 item 3's winding question `a_face_bounded_by_two_steps.rs`
//! left open. A wire of two steps has two corners, and two corners sweep exactly
//! zero however the face is turned, so the sweep stopped asking about them. This
//! is the shape behind that: **four corners, all on one line**. There are enough
//! of them to be a polygon, the sweep does ask, and the polygon still sweeps
//! nothing — the area is all in the two half-circles.
//!
//! The plate is the upper half of a disc of `OUTER` radius with the lower half
//! of a disc of `INNER` radius hung under it, standing `HEIGHT` tall. Both arcs
//! bulge away from the `x` axis they are cut by, so both walls are cylinders
//! with their material inside them and no face is turned round. It encloses
//! `π(OUTER² + INNER²)·HEIGHT/2`, and every corner sits at `(±R, 0, z)` where
//! `x² + y² − r²` cancels to nothing.
//!
//! The last test writes the same two half-circles as the rim of a round bar and
//! builds the bar on a turned axis, where the ends no longer land exactly on the
//! flat surfaces of the half-spaces that pick each arc out.

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale,
    Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftFace, DraftStep, DraftWire, Model, Result, SolidBuilder, SolidKey,
};

const OUTER: f64 = 3.0;
const INNER: f64 = 1.0;
const HEIGHT: f64 = 5.0;

const BAR_RADIUS: f64 = 1.0;

/// The sentence this lane earns its fix with.
///
/// Every one of this plate's four flat corners sits on the `x` axis, so the
/// polygon through them sweeps exactly zero and the winding this face is judged
/// by has to come from somewhere else. Before the fix this was
/// [`Invalid::OuterWireWindsAgainstTheNormal`](rcad_topo::Invalid), raised by
/// `SolidBuilder::build` and again by `Model::validate`. Now the winding is read
/// off the boundary the wire actually traces — its corners with the points
/// `rcad_geom::walk` puts along each arc — so the area is where the shape has
/// it.
#[test]
fn a_plate_bounded_by_two_half_circles_is_a_sound_solid() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());

    a_plate_bounded_by_two_half_circles(&mut model, &policy, &mut log)
        .expect("two half-circles and two straights bound a face");

    model
        .validate(&policy, &mut log)
        .expect("and the full sweep is happy with it");
}

/// That it passed for the right reason: the plate really does enclose what its
/// two half-circles say, so the winding the sweep now reads is the winding the
/// shape has.
#[test]
fn the_plate_encloses_what_its_two_half_circles_say() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let plate = a_plate_bounded_by_two_half_circles(&mut model, &policy, &mut log)
        .expect("two half-circles and two straights bound a face");

    let measured = model
        .solid(plate)
        .expect("this model made the key")
        .measure(
            rcad_geom::Deflection::of(0.001).expect("a fine mesh"),
            &policy,
            &mut log,
        )
        .expect("a plate measures");

    let space = core::f64::consts::PI * (OUTER * OUTER + INNER * INNER) / 2.0 * HEIGHT;
    assert!(
        (measured.volume() - space).abs() <= measured.volume_error(),
        "the plate encloses {space} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// The four flat corners are on one line, which is what makes this face worth a
/// test rather than a rerun of the quarter-disc's.
#[test]
fn the_plates_four_flat_corners_are_all_on_the_x_axis() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let plate = a_plate_bounded_by_two_half_circles(&mut model, &policy, &mut log)
        .expect("two half-circles and two straights bound a face");

    let flat: Vec<Vec<[f64; 3]>> = model
        .solid(plate)
        .expect("this model made the key")
        .shells()
        .flat_map(|shell| shell.faces())
        .filter(|face| matches!(face.surface(), Surface::Plane(flat) if flat.normal().components()[2] != 0.0))
        .map(|face| {
            face.outer_wire()
                .points()
                .map(|corner| corner.coordinates())
                .collect()
        })
        .collect();

    assert_eq!(flat.len(), 2, "a top and a bottom");
    for corners in flat {
        assert_eq!(corners.len(), 4, "four corners: {corners:?}");
        assert!(
            corners.iter().all(|corner| corner[1] == 0.0),
            "every corner on the x axis: {corners:?}"
        );
    }
}

/// The same plate with its bottom cap facing into its own material.
///
/// **The half of this fix that must not have been given away.** Winding a ring
/// of sampled points instead of a ring of corners would be no check at all if it
/// had merely stopped refusing things; reversing a face negates every term of
/// the sum exactly, so the one refusal this rule exists to make has to survive.
/// Only the bottom plane's normal is turned here — every corner, edge and wire
/// is the one the sound plate has — so what changes is which side of that face
/// its material is on, and nothing else.
#[test]
fn a_plate_whose_bottom_faces_into_its_own_material_is_still_refused() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());

    let refused = a_plate(
        &mut model,
        OUTER,
        INNER,
        HEIGHT,
        Vector::new(0.0, 0.0, 1.0),
        &policy,
        &mut log,
    );

    assert!(
        matches!(
            refused,
            Err(rcad_topo::Error::Invalid(
                rcad_topo::Invalid::OuterWireWindsAgainstTheNormal
            ))
        ),
        "got {refused:?}"
    );
}

/// The rule follows the shape and not a length somebody picked.
///
/// The same plate a thousand times smaller and a thousand times bigger. A walk
/// coarse enough to answer for one of these is far too coarse for the next, so a
/// rule that read its sign off a fixed deflection would answer these three
/// differently. The refinement is what makes them one answer.
#[test]
fn the_same_plate_a_thousand_times_smaller_and_bigger_is_the_same_answer() {
    for scale in [1e-3, 1.0, 1e3] {
        let policy = Policy::certified();
        let mut log = ProvenanceLog::new();
        let mut model = Model::new(a_working_size());

        a_plate(
            &mut model,
            OUTER * scale,
            INNER * scale,
            HEIGHT * scale,
            Vector::new(0.0, 0.0, -1.0),
            &policy,
            &mut log,
        )
        .unwrap_or_else(|why| panic!("a plate {scale} times the size is still a plate: {why:?}"));

        model
            .validate(&policy, &mut log)
            .unwrap_or_else(|why| panic!("and still sweeps clean at {scale}: {why:?}"));
    }
}

/// A bar whose rim is these two half-circles, built straight onto a turned axis.
///
/// The bar is `BAR_RADIUS` round and stands `z` 0 to 4 in two bands, one on
/// each spelling of its pipe, and the circle between the bands at `z = 2` is two
/// half-circles written as this file writes the plate's: each is the part of
/// where its band's pipe crosses the plane `z = 2` that lies in front of a
/// half-space through the origin facing `+y` or `-y`. So each half-space's flat
/// surface passes exactly through its arc's two ends.
///
/// Built turned, every point, surface and run is moved before the builder sees
/// it, and a turn rounds an end off its half-space's flat surface by an ulp or
/// so, to either side. Every turn here not about `x` or `y` was refused
/// `EdgeOutsideWhatPickedIt` — 200 of the 250, in all four ways of laying the
/// two spellings up and down, 800 builds of 1000.
#[test]
fn a_bar_whose_rim_is_two_half_circles_builds_turned_every_way() {
    let up = Cylinder::new(Axis::z(), BAR_RADIUS).expect("a radius");
    let down = Cylinder::new(Axis::z().reversed(), BAR_RADIUS).expect("a radius");
    let policy = Policy::certified();
    let mut wrong = Vec::new();
    let mut built = 0;
    for (turn_name, turn) in the_turns_off_z() {
        for (lower, upper) in [(up, up), (up, down), (down, up), (down, down)] {
            let what = format!(
                "bands on {} and {}, turned {turn_name}",
                which_way(lower),
                which_way(upper)
            );
            let mut model = Model::new(a_working_size());
            let mut log = ProvenanceLog::new();
            match a_bar_whose_rim_is_two_half_circles(
                &mut model,
                [lower, upper],
                &turn,
                &policy,
                &mut log,
            ) {
                Ok(_) => built += 1,
                Err(why) => wrong.push(format!("{what}: refused {why:?}")),
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{built} built and {} refused:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

fn a_plate_bounded_by_two_half_circles(
    model: &mut Model,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SolidKey> {
    a_plate(
        model,
        OUTER,
        INNER,
        HEIGHT,
        Vector::new(0.0, 0.0, -1.0),
        policy,
        log,
    )
}

/// The plate, face by face.
///
/// Eight corners, twelve edges, six faces: the flat top and bottom, a wall for
/// each half-circle, and the rectangle each straight end sweeps out.
///
/// `ground_faces` is which way the bottom cap points, and a sound plate has it
/// pointing down and away from its own material.
fn a_plate(
    model: &mut Model,
    outer: f64,
    inner: f64,
    height: f64,
    ground_faces: Vector,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();

    let ground = a_plane_at(0.0, ground_faces);
    let lid = a_plane_at(height, Vector::new(0.0, 0.0, 1.0));

    let corners = [
        (outer, 0.0),
        (inner, 0.0),
        (-inner, 0.0),
        (-outer, 0.0),
        (outer, height),
        (inner, height),
        (-inner, height),
        (-outer, height),
    ];
    let mut at = Vec::with_capacity(corners.len());
    for (x, z) in corners {
        at.push(builder.add_vertex(Point::new(x, 0.0, z))?);
    }

    let edges = [
        (0, 3, an_arc_over(&ground)),
        (3, 2, EdgeRun::Straight),
        (2, 1, an_arc_under(&ground)),
        (1, 0, EdgeRun::Straight),
        (4, 7, an_arc_over(&lid)),
        (7, 6, EdgeRun::Straight),
        (6, 5, an_arc_under(&lid)),
        (5, 4, EdgeRun::Straight),
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
    let over = walked(
        &mut builder,
        &[
            (0, Orientation::Forward),
            (11, Orientation::Forward),
            (4, Orientation::Reversed),
            (8, Orientation::Reversed),
        ],
    )?;
    let under = walked(
        &mut builder,
        &[
            (2, Orientation::Forward),
            (9, Orientation::Forward),
            (6, Orientation::Reversed),
            (10, Orientation::Reversed),
        ],
    )?;
    let near_end = walked(
        &mut builder,
        &[
            (3, Orientation::Forward),
            (8, Orientation::Forward),
            (7, Orientation::Reversed),
            (9, Orientation::Reversed),
        ],
    )?;
    let far_end = walked(
        &mut builder,
        &[
            (1, Orientation::Forward),
            (10, Orientation::Forward),
            (5, Orientation::Reversed),
            (11, Orientation::Reversed),
        ],
    )?;

    let cut = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, -1.0, 0.0))
            .expect("an axis direction has length"),
    );
    let faces = [
        builder.add_face(Surface::Plane(ground), &[below])?,
        builder.add_face(Surface::Plane(lid), &[above])?,
        builder.add_face(a_pipe_of_radius(outer), &[over])?,
        builder.add_face(a_pipe_of_radius(inner), &[under])?,
        builder.add_face(cut, &[near_end])?,
        builder.add_face(cut, &[far_end])?,
    ];

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

/// The half of the circle `plane` cuts that lies over the `x` axis.
fn an_arc_over(plane: &Plane) -> EdgeRun {
    an_arc_on(plane, 1.0)
}

/// And the half under it.
fn an_arc_under(plane: &Plane) -> EdgeRun {
    an_arc_on(plane, -1.0)
}

fn an_arc_on(plane: &Plane, side: f64) -> EdgeRun {
    EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(*plane),
        picked_by: smallvec::smallvec![Picker::from(
            HalfSpace::through_a_point_facing(Point::origin(), Vector::new(0.0, side, 0.0))
                .expect("an axis direction has length")
        )],
    }
}

fn a_pipe_of_radius(radius: f64) -> Surface {
    Surface::Cylinder(Cylinder::new(Axis::z(), radius).expect("a positive radius"))
}

fn a_wire(builder: &mut SolidBuilder, steps: &[(DraftEdge, Orientation)]) -> Result<DraftWire> {
    let mut walked: Vec<DraftStep> = Vec::with_capacity(steps.len());
    for &(edge, orientation) in steps {
        walked.push(DraftStep::Along(builder.add_coedge(edge, orientation)?));
    }
    builder.add_wire(&walked)
}

/// Each of ten axes through the origin, turned by `0.25·k` for `k` = 1 to 25.
fn the_turns_off_z() -> Vec<(String, Isometry)> {
    let axes = [
        (1.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        (1.0, 1.0, 0.0),
        (1.0, 0.0, 1.0),
        (0.0, 1.0, 1.0),
        (1.0, 2.0, 3.0),
        (-1.0, 2.0, 0.5),
        (3.0, -1.0, 2.0),
        (1.0, 1.0, 1.0),
        (2.0, 0.0, -1.0),
    ];
    let mut turns = Vec::with_capacity(axes.len() * 25);
    for (x, y, z) in axes {
        let axis = Axis::new(Point::origin(), Vector::new(x, y, z)).expect("a direction");
        for k in 1..=25 {
            let angle = 0.25 * f64::from(k);
            turns.push((
                format!("{angle} about {:?}", (x, y, z)),
                Isometry::rotation(&axis, angle).expect("a finite turn"),
            ));
        }
    }
    turns
}

fn which_way(wall: Cylinder) -> &'static str {
    if wall.axis().direction().z() > 0.0 {
        "the pipe pointing up"
    } else {
        "the pipe pointing down"
    }
}

/// The bar of the test above: `BAR_RADIUS` round from `z = 0` to `z = 4`, the
/// band below `z = 2` on `walls[0]` and the band above it on `walls[1]`, with
/// every point, surface and run moved by `built_on` before the builder sees it.
///
/// The rim is two arcs between the corners at `(±BAR_RADIUS, 0, 2)`: the one
/// through `-y` runs from `+x` to `-x` and the one through `+y` back again, so
/// the lower band walks the rim against its pipe's longitude and the upper band
/// with it, whichever way round the pipe is written.
///
/// # Errors
///
/// Whatever [`SolidBuilder::build`] refuses the bar for.
fn a_bar_whose_rim_is_two_half_circles(
    model: &mut Model,
    walls: [Cylinder; 2],
    built_on: &Isometry,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SolidKey> {
    let [lower, upper] = walls;
    let mut draft = SolidBuilder::new();
    let corner = |draft: &mut SolidBuilder, x: f64, z: f64| {
        draft.add_vertex(built_on.apply_to_point(Point::new(x, 0.0, z)))
    };
    let at_0 = corner(&mut draft, BAR_RADIUS, 0.0)?;
    let east = corner(&mut draft, BAR_RADIUS, 2.0)?;
    let west = corner(&mut draft, -BAR_RADIUS, 2.0)?;
    let at_4 = corner(&mut draft, BAR_RADIUS, 4.0)?;

    let rim_plane = a_plane_at(2.0, Vector::new(0.0, 0.0, 1.0));
    let seam_below = draft.add_edge(at_0, east, EdgeRun::Straight)?;
    let seam_above = draft.add_edge(east, at_4, EdgeRun::Straight)?;
    let bottom_circle = draft.add_edge(at_0, at_0, EdgeRun::RightRoundWhereItsFacesCross)?;
    let through_minus_y =
        draft.add_edge(east, west, an_arc_under(&rim_plane).transformed(built_on))?;
    let through_plus_y =
        draft.add_edge(west, east, an_arc_over(&rim_plane).transformed(built_on))?;
    let top_circle = draft.add_edge(at_4, at_4, EdgeRun::RightRoundWhereItsFacesCross)?;
    let bottom = a_cap(&mut draft, 0.0, -1.0, bottom_circle, lower, built_on)?;
    let top = a_cap(&mut draft, 4.0, 1.0, top_circle, upper, built_on)?;

    let below = a_band(
        &mut draft,
        lower.transformed(built_on),
        &[
            (
                bottom_circle,
                the_way_it_is_walked(lower, Orientation::Forward),
            ),
            (seam_below, Orientation::Forward),
            (through_minus_y, Orientation::Forward),
            (through_plus_y, Orientation::Forward),
            (seam_below, Orientation::Reversed),
        ],
    )?;
    let above = a_band(
        &mut draft,
        upper.transformed(built_on),
        &[
            (through_plus_y, Orientation::Reversed),
            (through_minus_y, Orientation::Reversed),
            (seam_above, Orientation::Forward),
            (
                top_circle,
                the_way_it_is_walked(upper, Orientation::Reversed),
            ),
            (seam_above, Orientation::Reversed),
        ],
    )?;
    draft.add_shell(&[bottom, below, above, top].map(|face| (face, Orientation::Forward)))?;
    draft.build(model, policy, log)
}

fn a_band(
    draft: &mut SolidBuilder,
    wall: Cylinder,
    round: &[(DraftEdge, Orientation)],
) -> Result<DraftFace> {
    let wire = a_wire(draft, round)?;
    draft.add_face(Surface::Cylinder(wall), &[wire])
}

/// A flat cap at height `z` facing `outward` along `z`, moved by `built_on`,
/// bounded by `circle` walked against the wall's use of it.
fn a_cap(
    draft: &mut SolidBuilder,
    z: f64,
    outward: f64,
    circle: DraftEdge,
    wall: Cylinder,
    built_on: &Isometry,
) -> Result<DraftFace> {
    let plane = a_plane_at(z, Vector::new(0.0, 0.0, outward)).transformed(built_on);
    let used = if outward < 0.0 {
        Orientation::Reversed
    } else {
        Orientation::Forward
    };
    let wire = a_wire(draft, &[(circle, the_way_it_is_walked(wall, used))])?;
    draft.add_face(Surface::Plane(plane), &[wire])
}

/// A cap circle between `wall` and a plane is stored running along `ŵ × ∇C`
/// for the wall's stored direction `ŵ`, so a wall written pointing down walks
/// both its cap circles the other way round: `turn` is how a wall pointing up
/// would use the circle.
fn the_way_it_is_walked(wall: Cylinder, turn: Orientation) -> Orientation {
    if wall.axis().direction().z() > 0.0 {
        turn
    } else {
        turn.reversed()
    }
}
