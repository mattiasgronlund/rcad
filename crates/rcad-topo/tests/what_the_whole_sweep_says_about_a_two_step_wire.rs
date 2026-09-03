//! What the **model's** fate is, for the two shapes `builder.rs` argued about
//! by running one function.
//!
//! Two doc comments in `builder.rs` justified how
//! `two_steps_that_bound_a_face` reads a run it has never seen. Each rested on
//! a claim about what the rest of the kernel does, and each claim had been
//! measured on one function in isolation:
//!
//! - *"the length rule is the only thing that asks"* — measured by calling
//!   `check_edges` and reading its `Ok(())`.
//! - *"every other check still runs over it"* — measured on a plate with a
//!   **cylinder** wall, which is the one case where the sweep's mesh is not
//!   gated off.
//!
//! Both are answered here by handing a whole model to `Model::validate` and
//! writing down what comes back. The two shapes are the two the arguments were
//! about, built out of runs that exist today: a run naming a **plane** is not
//! [`EdgeRun::Straight`], so it takes both the generous reading a fourth kind
//! of run would get at `two_steps_that_bound_a_face` and the generous reading
//! `the_ends_are_the_whole_of_it` refuses a fourth kind. That is what makes
//! these measurements of the real code rather than of a clone.

use rcad_geom::{EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftFace, DraftStep, DraftVertex, DraftWire, Error, Model, Result, SolidBuilder,
    SolidKey,
};

/// A zero-length edge the length rule lets through does **not** go unlooked at:
/// the sweep refuses the model, twice over.
///
/// `builder.rs` said the length rule was the only thing that asks, so a run
/// read generously there was "a zero-length edge nobody looks at again". The
/// model below is that edge — a tetrahedron with one corner written down twice
/// and an edge of no length between the two copies, running along a plane both
/// copies sit on, which is a run `the_ends_are_the_whole_of_it` keeps. Every
/// other thing about the solid is sound: the four faces are the tetrahedron's,
/// the volume is the tetrahedron's, every edge is walked once each way.
///
/// `check_dynamic_range` is what catches it, and it catches it on the **chord**
/// rather than on the run: an edge that is not [`EdgeRun::RightRoundWhereItsFacesCross`]
/// is measured end to end, and an end-to-end distance of nothing is not a
/// feature size. So the length rule is one of two things that ask, not the
/// only one.
#[test]
fn a_zero_length_edge_the_length_rule_keeps_is_still_refused_by_the_whole_sweep() {
    let mut model = Model::new(a_working_size());

    let refused = a_tetrahedron_with_one_corner_written_down_twice(&mut model);

    let why = refused.expect_err("a corner written down twice is no solid");

    assert!(
        matches!(
            why,
            Error::Geometry(rcad_geom::Error::FeatureSizeNotPositive { size: 0.0 })
        ),
        "an edge of no length went through the whole sweep unlooked at: got {why:?}"
    );
}

/// A two-step face that bounds nothing comes back **sound** when every face of
/// the solid is flat.
///
/// This is the hole `builder.rs` claimed there was none of. Two edges between
/// the same two corners, one [`EdgeRun::Straight`] and one naming a plane both
/// corners lie on, are the same segment written twice — so the face between
/// them has no area at all. Three checks were named as standing behind the
/// two-step wire, and on this model none of them fires:
///
/// - the winding a flat face is judged by skips a wire of under three steps
///   (`validate.rs`'s `take(3).count() < 3` — a two-corner polygon sweeps
///   exactly nothing whichever way it is walked);
/// - the sweep's **mesh** is what caught the same mistake on a plate with a
///   cylinder wall, and the mesh is gated off for a solid that is wholly flat;
/// - `check_edges` asks only that an edge's corners lie on the surface its run
///   names, which two corners on a plane through both of them do.
///
/// So this test asserts the gap and not a refusal. The day something does
/// stand behind a two-step wire on a flat solid, this goes red, and that is the
/// notice to go and rewrite the paragraph in `builder.rs` that points here.
#[test]
fn a_flat_two_step_face_that_bounds_nothing_is_not_caught_by_anything() {
    let mut model = Model::new(a_working_size());

    let built = a_tetrahedron_with_one_edge_written_down_twice(&mut model);

    let solid = built.expect("nothing in the sweep asks a flat two-step wire anything");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("and the full sweep says the same");
    assert_eq!(
        model
            .solid(solid)
            .shells()
            .flat_map(|shell| shell.faces())
            .filter(|face| face.outer_wire().step_count() == 2)
            .count(),
        1,
        "the face this is about is the two-step one"
    );
}

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

/// A tetrahedron on the three axes whose corner at the origin is two vertices,
/// with an edge of no length between them.
///
/// The two faces that meet along the tetrahedron's `AB` side each grow a step,
/// so the extra edge is walked once each way like every other.
fn a_tetrahedron_with_one_corner_written_down_twice(model: &mut Model) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let [a, b, c, d] = corners_of_a_tetrahedron(&mut builder)?;
    let also_a = builder.add_vertex(Point::origin())?;

    let no_length = builder.add_edge(a, also_a, along_the_ground())?;
    let to_b = builder.add_edge(also_a, b, EdgeRun::Straight)?;
    let [b_to_c, c_to_a, a_to_d, b_to_d, c_to_d] =
        the_other_five_sides(&mut builder, [a, b, c, d])?;

    let ground = a_wire(
        &mut builder,
        &[
            (c_to_a, Orientation::Reversed),
            (b_to_c, Orientation::Reversed),
            (to_b, Orientation::Reversed),
            (no_length, Orientation::Reversed),
        ],
    )?;
    let down_y = a_wire(
        &mut builder,
        &[
            (no_length, Orientation::Forward),
            (to_b, Orientation::Forward),
            (b_to_d, Orientation::Forward),
            (a_to_d, Orientation::Reversed),
        ],
    )?;
    let (down_x, slope) =
        the_other_two_faces(&mut builder, [a_to_d, c_to_d, c_to_a, b_to_c, b_to_d])?;

    close_the_tetrahedron(&mut builder, [ground, down_y, down_x, slope], &[])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The same tetrahedron with its `AB` side written down twice — once straight
/// and once as a piece of where the ground meets a plane through both corners —
/// and a face of no area between the two copies.
fn a_tetrahedron_with_one_edge_written_down_twice(model: &mut Model) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let [a, b, c, d] = corners_of_a_tetrahedron(&mut builder)?;

    let straight = builder.add_edge(a, b, EdgeRun::Straight)?;
    let named = builder.add_edge(a, b, along_the_ground())?;
    let [b_to_c, c_to_a, a_to_d, b_to_d, c_to_d] =
        the_other_five_sides(&mut builder, [a, b, c, d])?;

    let ground = a_wire(
        &mut builder,
        &[
            (c_to_a, Orientation::Reversed),
            (b_to_c, Orientation::Reversed),
            (straight, Orientation::Reversed),
        ],
    )?;
    let down_y = a_wire(
        &mut builder,
        &[
            (named, Orientation::Forward),
            (b_to_d, Orientation::Forward),
            (a_to_d, Orientation::Reversed),
        ],
    )?;
    let bounds_nothing = a_wire(
        &mut builder,
        &[
            (straight, Orientation::Forward),
            (named, Orientation::Reversed),
        ],
    )?;
    let (down_x, slope) =
        the_other_two_faces(&mut builder, [a_to_d, c_to_d, c_to_a, b_to_c, b_to_d])?;

    // The sliver stands on the plane through `AB` that leans between the two
    // faces either side of it, so its corners are on it exactly.
    let leaning = builder.add_face(
        Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, -1.0, -1.0))
                .expect("a direction with length"),
        ),
        &[bounds_nothing],
    )?;
    close_the_tetrahedron(
        &mut builder,
        [ground, down_y, down_x, slope],
        &[(leaning, Orientation::Forward)],
    )?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// `A` at the origin and `B`, `C`, `D` two along each axis.
fn corners_of_a_tetrahedron(builder: &mut SolidBuilder) -> Result<[DraftVertex; 4]> {
    Ok([
        builder.add_vertex(Point::origin())?,
        builder.add_vertex(Point::new(2.0, 0.0, 0.0))?,
        builder.add_vertex(Point::new(0.0, 2.0, 0.0))?,
        builder.add_vertex(Point::new(0.0, 0.0, 2.0))?,
    ])
}

/// A run naming the ground plane rather than the two ends.
///
/// Every corner used with this sits at height nothing, so `check_edges` reads
/// each of them exactly on the plane the run names.
fn along_the_ground() -> EdgeRun {
    EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
                .expect("the z axis is a unit normal"),
        ),
        picked_by: smallvec::SmallVec::new(),
    }
}

fn the_other_five_sides(
    builder: &mut SolidBuilder,
    [a, b, c, d]: [DraftVertex; 4],
) -> Result<[DraftEdge; 5]> {
    Ok([
        builder.add_edge(b, c, EdgeRun::Straight)?,
        builder.add_edge(c, a, EdgeRun::Straight)?,
        builder.add_edge(a, d, EdgeRun::Straight)?,
        builder.add_edge(b, d, EdgeRun::Straight)?,
        builder.add_edge(c, d, EdgeRun::Straight)?,
    ])
}

/// The two faces neither shape touches: the one down `x` and the sloping one.
fn the_other_two_faces(
    builder: &mut SolidBuilder,
    [a_to_d, c_to_d, c_to_a, b_to_c, b_to_d]: [DraftEdge; 5],
) -> Result<(DraftWire, DraftWire)> {
    let down_x = a_wire(
        builder,
        &[
            (a_to_d, Orientation::Forward),
            (c_to_d, Orientation::Reversed),
            (c_to_a, Orientation::Forward),
        ],
    )?;
    let slope = a_wire(
        builder,
        &[
            (b_to_c, Orientation::Forward),
            (c_to_d, Orientation::Forward),
            (b_to_d, Orientation::Reversed),
        ],
    )?;
    Ok((down_x, slope))
}

fn close_the_tetrahedron(
    builder: &mut SolidBuilder,
    wires: [DraftWire; 4],
    also: &[(DraftFace, Orientation)],
) -> Result<()> {
    let [ground, down_y, down_x, slope] = wires;
    let faces = [
        (ground, Vector::new(0.0, 0.0, -1.0), Point::origin()),
        (down_y, Vector::new(0.0, -1.0, 0.0), Point::origin()),
        (down_x, Vector::new(-1.0, 0.0, 0.0), Point::origin()),
        (slope, Vector::new(1.0, 1.0, 1.0), Point::new(2.0, 0.0, 0.0)),
    ];
    let mut standing = Vec::with_capacity(faces.len());
    for (wire, facing, through) in faces {
        let plane = Plane::from_point_and_normal(through, facing).expect("a direction with length");
        standing.push((
            builder.add_face(Surface::Plane(plane), &[wire])?,
            Orientation::Forward,
        ));
    }
    standing.extend_from_slice(also);
    builder.add_shell(&standing)?;
    Ok(())
}

fn a_wire(builder: &mut SolidBuilder, steps: &[(DraftEdge, Orientation)]) -> Result<DraftWire> {
    let mut walked = Vec::with_capacity(steps.len());
    for &(edge, orientation) in steps {
        walked.push(DraftStep::Along(builder.add_coedge(edge, orientation)?));
    }
    builder.add_wire(&walked)
}
