//! A solid taken apart through the public API and described again through
//! [`SolidBuilder`], measuring the same and fusing the same.
//!
//! `docs/decisions.md` §42.12 criterion 1. Until this lane there was no public
//! way to build a solid out of faces at all — §42.5's `E0624` — so an importer
//! could not turn `advanced_face` and `closed_shell` into a `Model`.
//!
//! **Why this is not a tautology.** The rebuild does not copy an arena: it
//! walks the solid the way a reader walks a file — shell, faces, wires, steps,
//! coedges, edges, corners — through views only, and hands the pieces back one
//! call at a time. Nothing in `SolidBuilder` sees the original. If the builder
//! wrote a coedge the wrong way round, dropped a pole, reversed a hole or mixed
//! up which wire is the outer one, the sweep, the measurement or the fuse would
//! say so.
//!
//! **The oracles, strongest first** (testing skill's table). Volume, area and
//! centroid are compared **bit for bit**, not within a tolerance: the same
//! corners summed in the same order give the same `f64`, and anything else is a
//! real difference. The fuse comparison is the strong one — two solids that
//! measure the same can still fuse differently, because a fuse reads topology
//! the sums never touch.
//!
//! **The ball's seam is supplied rather than read**, and that is deliberate.
//! `EdgeView::run` is `pub(crate)`: a run carries a `Surface` and `Picker`s in
//! the frame the face is stored in, and handing one out un-located is the thing
//! CLAUDE.md forbids. So a caller rebuilding a ball does what a reader does —
//! it names the meridian from the geometry it holds, not from the model.

use std::collections::HashMap;

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point,
    Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftStep, DraftVertex, EdgeKey, EdgeView, Error, Invalid, Measurement, Model,
    ProfileBuilder, Result, Rule, SolidBuilder, SolidKey, VertexKey, VertexView, WireStepView,
};

mod rings;

use rings::a_quarter_bend;

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

/// What a straight-sided or whole-circle edge runs along, read off the model.
///
/// [`EdgeView::is_closed`] is public and is the whole of what these two shapes
/// need: a cube has only straight edges and a round bar adds two cap circles.
fn a_run_the_model_says(edge: EdgeView<'_>) -> EdgeRun {
    if edge.is_closed() {
        EdgeRun::RightRoundWhereItsFacesCross
    } else {
        EdgeRun::Straight
    }
}

#[test]
fn a_cube_built_face_by_face_is_the_cube_add_cube_builds() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let built = model
        .add_cube(2.0, &policy, &mut log)
        .expect("a cube of side two");

    let rebuilt = Walk::of(&model, built)
        .describe_again(&a_run_the_model_says)
        .expect("a cube can be described again")
        .build(&mut model, &policy, &mut log)
        .expect("and the sweep is happy with the description");

    assert!(log.is_empty(), "nothing here had to be approximated");
    model
        .validate(&policy, &mut log)
        .expect("the model holding both is sound");
    assert_eq!(model.solid_count(), 2);
    // The cutter takes the corner octant off, on a whole-number lattice.
    the_same_numbers(&model, built, rebuilt, &|model| {
        a_block(model, [2.0, 2.0, 2.0], Vector::new(1.0, 1.0, 1.0))
    });
}

#[test]
fn a_round_bar_built_face_by_face_is_the_round_bar_add_round_bar_builds() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let built = model
        .add_round_bar(2.0, 8.0, &Isometry::identity())
        .expect("a bar of radius two");

    let rebuilt = Walk::of(&model, built)
        .describe_again(&a_run_the_model_says)
        .expect("a bar can be described again")
        .build(&mut model, &policy, &mut log)
        .expect("and the sweep is happy with the description");

    model
        .validate(&policy, &mut log)
        .expect("the model holding both is sound");
    // `a_notch_across_the_seam` in `degenerate_curved_fuses.rs` is this cut:
    // the notch runs across the bar's seam at `(2, 0, z)`, so the comparison
    // reaches the one piece of topology a rebuild is most likely to get wrong.
    the_same_numbers(&model, built, rebuilt, &|model| {
        a_block(model, [2.0, 2.0, 2.0], Vector::new(1.0, -1.0, 3.0))
    });
}

/// The one that needs a pole, a seam walked twice by one face, and a run the
/// model will not hand back.
#[test]
fn a_ball_built_face_by_face_is_the_ball_add_ball_builds() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let built = model
        .add_ball(4.0, &Isometry::identity())
        .expect("a ball of radius four");

    let rebuilt = Walk::of(&model, built)
        .describe_again(&a_seam_named_the_way_a_file_names_one)
        .expect("a ball can be described again")
        .build(&mut model, &policy, &mut log)
        .expect("and the sweep is happy with the description");

    model
        .validate(&policy, &mut log)
        .expect("the model holding both is sound");
    // `a_ball_cut_across_its_seam`: the plane `z = 2` meets the seam once.
    the_same_numbers(&model, built, rebuilt, &|model| {
        a_block(model, [12.0, 12.0, 12.0], Vector::new(-6.0, -6.0, 2.0))
    });
}

/// A quarter bend built face by face is the bend `rings::a_quarter_bend`
/// builds.
///
/// The measurement half of the oracle is exactly the cube's, the bar's and
/// the ball's: volume, area and centroid, read through views only, compared
/// bit for bit. The fuse half cannot copy `the_same_numbers`'s shape, because
/// **no solid this kernel holds fuses successfully with a torus-faced solid
/// today** — not a plane-bounded block, not a second bend on the bend's own
/// axis, not even a bit-for-bit copy of the bend itself
/// (`a_bend_refuses_to_fuse_with_a_copy_of_itself_under_every_rule` in
/// `how_often_a_fuse_with_a_torus_refuses.rs` is the demonstration: two bends
/// sharing a torus get further than anything else does — the corners are found
/// and put in order round the circles they sit on — and stop at two coincident
/// curves touching on one face without crossing).
///
/// So the fuse half of the oracle here compares the **refusal**: the same
/// rule against the same third solid must refuse the same way for both.
/// That still reads topology the measurement never touches — the refusal is
/// raised while the arrangement is walking the corners the rebuild itself
/// produced, so a coedge walked backwards or a picker handed the wrong sign
/// would have to be caught before that walk gets as far as the question both
/// descriptions are shown to answer alike.
#[test]
fn a_bend_built_face_by_face_is_the_bend_a_quarter_bend_builds() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let built = a_quarter_bend(&mut model, 4.0, 1.0);

    let rebuilt = Walk::of(&model, built)
        .describe_again(&a_seam_of_a_bend_named_the_way_a_file_names_one)
        .expect("a bend can be described again")
        .build(&mut model, &policy, &mut log)
        .expect("and the sweep is happy with the description");

    model
        .validate(&policy, &mut log)
        .expect("the model holding both is sound");

    let deflection = Deflection::for_scale(model.scale());
    let one = model
        .solid(built)
        .measure(deflection, &policy, &mut log)
        .expect("the bend the rings module built measures");
    let other = model
        .solid(rebuilt)
        .measure(deflection, &policy, &mut log)
        .expect("and so does the one described again");
    assert_eq!(one.volume(), other.volume(), "volume");
    assert_eq!(one.surface_area(), other.surface_area(), "area");
    assert_eq!(one.centroid(), other.centroid(), "centroid");

    // A second bend, sharing the ring, tube and axis exactly and so sharing
    // the torus surface itself, turned a quarter of the way round the ring so
    // it overlaps the first without being a plain duplicate of it.
    let a_turned_bend_of_the_same_ring_and_tube = |model: &mut Model| -> SolidKey {
        let second = a_quarter_bend(model, 4.0, 1.0);
        model.place_solid(
            second,
            &Isometry::rotation(&Axis::z(), core::f64::consts::FRAC_PI_4)
                .expect("a quarter turn is a real angle"),
        );
        second
    };
    let refusal_of_built =
        refused_fusing_a_third_solid(&model, built, &a_turned_bend_of_the_same_ring_and_tube);
    let refusal_of_rebuilt =
        refused_fusing_a_third_solid(&model, rebuilt, &a_turned_bend_of_the_same_ring_and_tube);
    assert!(
        matches!(
            refusal_of_built,
            Error::TwoCurvesTouchWithoutCrossing { .. }
        ),
        "expected the arrangement to reach the two coincident curves and refuse there, got \
         {refusal_of_built:?}"
    );
    assert_eq!(
        format!("{refusal_of_built}"),
        format!("{refusal_of_rebuilt}"),
        "the kernel-built bend and the one described again refused the same fuse differently"
    );
}

/// The seam of a bend, named from geometry a file carries rather than from
/// the model — exactly `rings::a_quarter_bend`'s own construction of it,
/// which is what a reader reconstructing a `surface_curve` off a torus does.
fn a_seam_of_a_bend_named_the_way_a_file_names_one(edge: EdgeView<'_>) -> EdgeRun {
    if edge.is_closed() {
        return EdgeRun::RightRoundWhereItsFacesCross;
    }
    let flat_through_the_middle =
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(flat_through_the_middle),
        picked_by: smallvec::smallvec![
            Picker::InsideAPipe(Cylinder::new(Axis::z(), 4.0).expect("the ring's own radius")),
            Picker::from(
                HalfSpace::new(Vector::new(1.0, 1.0, 0.0), 0.0).expect("a normal with a length")
            ),
        ],
    }
}

/// One solid of the model, fused with a third one under [`Rule::Difference`],
/// on a clone so the two runs this is called from cannot see each other's
/// answer — the refusal `cut_by_a_third_solid` expects not to happen here.
fn refused_fusing_a_third_solid(
    model: &Model,
    which: SolidKey,
    cutter: &dyn Fn(&mut Model) -> SolidKey,
) -> Error {
    let policy = Policy::certified();
    let position = model
        .solids()
        .position(|solid| solid.key() == which)
        .expect("the solid is in the model it came from");
    let mut scratch = model.clone();
    let keep = scratch
        .solids()
        .nth(position)
        .expect("a clone holds the same solids in the same order")
        .key();
    let knife = cutter(&mut scratch);
    scratch
        .fuse(
            &[keep],
            &[knife],
            Rule::Difference,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect_err("a bend does not fuse successfully with anything this lane found")
}

/// A box of the given lengths, with its low corner at `at`.
fn a_block(model: &mut Model, [along_x, along_y, along_z]: [f64; 3], at: Vector) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [along_x, 0.0],
        [along_x, along_y],
        [0.0, along_y],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle");
    model
        .add_extrusion(
            &rectangle,
            along_z,
            &Isometry::translation(at).expect("a finite placement"),
        )
        .expect("building a box needs no decision")
}

/// The seam of a ball, named from geometry a file carries rather than from the
/// model.
///
/// A meridian is where the sphere crosses the plane `y = 0`, which is one whole
/// circle; the seam is the half of it with `x >= 0`. Both facts come from the
/// shape, which is what an importer reads out of a `surface_curve`.
fn a_seam_named_the_way_a_file_names_one(edge: EdgeView<'_>) -> EdgeRun {
    assert!(!edge.is_closed(), "a ball's one edge is its seam");
    let meridian = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("the y axis is a unit normal");
    let facing_x = HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
        .expect("the x axis has length");
    EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(meridian),
        picked_by: smallvec::smallvec![Picker::from(facing_x)],
    }
}

/// What N occurrences of one shape cost, when each is described again.
///
/// §42.7 settles that M5 imports N occurrences as N solids and publishes what
/// that costs. A builder holds no sharing at all: every occurrence is described
/// from scratch and written from scratch, so the arena grows by exactly one
/// copy per occurrence and nothing records that the copies came from one shape.
/// This is that number, asserted rather than described.
#[test]
fn five_copies_of_one_shape_cost_five_times_the_entities() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let first = model
        .add_cube(2.0, &policy, &mut log)
        .expect("a cube of side two");
    let (vertices, edges, faces) = (model.vertex_count(), model.edge_count(), model.face_count());

    for copy in 1..5 {
        let away = Isometry::translation(Vector::new(10.0 * f64::from(copy), 0.0, 0.0))
            .expect("a finite shift");
        Walk::of(&model, first)
            .moved_by(away)
            .describe_again(&a_run_the_model_says)
            .expect("a cube can be described again somewhere else")
            .build(&mut model, &policy, &mut log)
            .expect("a shifted cube is a cube");
    }

    assert_eq!(model.solid_count(), 5);
    assert_eq!(model.vertex_count(), 5 * vertices);
    assert_eq!(model.edge_count(), 5 * edges);
    assert_eq!(model.face_count(), 5 * faces);
    model
        .validate(&policy, &mut log)
        .expect("five cubes side by side are a sound model");
}

/// `Model::validate` still sees nothing that takes two solids to see, and the
/// pass that does see it is a call of its own.
///
/// §42.6 demonstrated the gap on solids the kernel built itself. Here it is
/// through the new door: two descriptions of the very same cube, both accepted,
/// both written, and the sweep calls the pair sound. **A refusal you cannot
/// detect is not a refusal**, so it was recorded as a test rather than as a
/// sentence — and §42.12 criterion 8 has now closed it.
///
/// What changed is the second half of the test. The sweep's `Ok` is still
/// asserted, because criterion 8 landed as a pass of its own and not as a check
/// inside the sweep, and the same two cubes are now refused by
/// `Model::check_no_two_solids_share_space`. Both halves are here together so
/// that moving the check into the sweep breaks this test rather than passing it
/// quietly.
#[test]
fn the_sweep_passes_two_cubes_at_one_place_and_the_pass_of_its_own_refuses_them() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let built = model
        .add_cube(2.0, &policy, &mut log)
        .expect("a cube of side two");
    Walk::of(&model, built)
        .describe_again(&a_run_the_model_says)
        .expect("a cube can be described again")
        .build(&mut model, &policy, &mut log)
        .expect("and the same cube in the same place is accepted");

    assert_eq!(model.solid_count(), 2);
    model
        .validate(&policy, &mut log)
        .expect("nothing in the sweep compares one solid against another");

    let refused = model.check_no_two_solids_share_space(&policy, &mut log);
    assert!(
        matches!(refused, Err(Error::TwoSolidsShareSpace { .. })),
        "expected the pass of its own to refuse the pair, got {refused:?}"
    );
}

/// A face turned round in the shell is a solid whose skin points inwards, and
/// the sweep says so rather than the builder.
///
/// This is the division the builder is built on: what one call can decide is
/// decided there, and what takes the whole solid comes back from
/// [`Model::validate`] under the name the sweep already had.
#[test]
fn a_solid_described_with_every_face_turned_round_is_refused_by_the_sweep() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let built = model
        .add_cube(2.0, &policy, &mut log)
        .expect("a cube of side two");

    let refused = Walk::of(&model, built)
        .with_every_face_turned_round()
        .describe_again(&a_run_the_model_says)
        .expect("a cube described inside out is still a description")
        .build(&mut model, &policy, &mut log);

    assert!(
        matches!(
            refused,
            Err(Error::Invalid(Invalid::VolumeIsNotPositive { .. }))
        ),
        "expected the sweep to say the faces point inwards, got {refused:?}"
    );
    assert_eq!(model.solid_count(), 1, "a refused build writes nothing");
}

/// Both solids measure the same, and both fuse the same against a third.
///
/// The measurement is compared bit for bit. The fuse is the check measurement
/// cannot make: it reads which coedge runs which way and which wire is the
/// outer one, and two solids that agree on volume can disagree there.
///
/// **Both fuses have to answer**, and the cut has to take something away. Two
/// refusals that agree would compare nothing at all, which is the vacuity the
/// testing skill's first rule is about.
fn the_same_numbers(
    model: &Model,
    built: SolidKey,
    rebuilt: SolidKey,
    cutter: &dyn Fn(&mut Model) -> SolidKey,
) {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let deflection = Deflection::for_scale(model.scale());

    let one = model
        .solid(built)
        .measure(deflection, &policy, &mut log)
        .expect("the solid the kernel built measures");
    let other = model
        .solid(rebuilt)
        .measure(deflection, &policy, &mut log)
        .expect("and so does the one described again");
    assert_eq!(one.volume(), other.volume(), "volume");
    assert_eq!(one.surface_area(), other.surface_area(), "area");
    assert_eq!(one.centroid(), other.centroid(), "centroid");

    let whole = one.volume();
    let cut_one = cut_by_a_third_solid(model, built, cutter, deflection)
        .expect("the solid the kernel built is cut by the third one");
    let cut_other = cut_by_a_third_solid(model, rebuilt, cutter, deflection)
        .expect("and so is the one described again");

    assert_eq!(
        cut_one.len(),
        cut_other.len(),
        "the two fuses came back with different numbers of pieces"
    );
    assert!(!cut_one.is_empty(), "the cut left nothing to compare");
    let left = cut_one.iter().map(|piece| piece.volume()).sum::<f64>();
    assert!(
        left < whole,
        "the cut took nothing away, so this compared two solids nothing happened to: \
         {left} of {whole}"
    );
    for (one, other) in cut_one.iter().zip(&cut_other) {
        assert_eq!(one.volume(), other.volume(), "a fused piece's volume");
        assert_eq!(
            one.surface_area(),
            other.surface_area(),
            "a fused piece's area"
        );
        assert_eq!(one.centroid(), other.centroid(), "a fused piece's centroid");
    }
}

/// One solid of the model, cut by a third one it overlaps, measured piece by
/// piece.
///
/// Done on a clone so the two runs cannot see each other's answer.
fn cut_by_a_third_solid(
    model: &Model,
    which: SolidKey,
    cutter: &dyn Fn(&mut Model) -> SolidKey,
    deflection: Deflection,
) -> Result<Vec<Measurement>> {
    let policy = Policy::certified();
    let position = model
        .solids()
        .position(|solid| solid.key() == which)
        .expect("the solid is in the model it came from");
    let mut scratch = model.clone();
    let keep = scratch
        .solids()
        .nth(position)
        .expect("a clone holds the same solids in the same order")
        .key();
    let knife = cutter(&mut scratch);

    let fused = scratch.fuse(
        &[keep],
        &[knife],
        Rule::Difference,
        &policy,
        &mut ProvenanceLog::new(),
    )?;
    let pieces: Vec<SolidKey> = fused.solids().to_vec();
    pieces
        .into_iter()
        .map(|piece| {
            scratch
                .solid(piece)
                .measure(deflection, &policy, &mut ProvenanceLog::new())
        })
        .collect()
}

/// A walk over one solid that describes what it finds to a [`SolidBuilder`].
///
/// Everything is read through views, so nothing here reaches past the public
/// API. What it may vary is what a reader varies: where the shape sits, and
/// which way round its faces are used.
struct Walk<'m> {
    model: &'m Model,
    solid: SolidKey,
    moved_by: Isometry,
    turn: Orientation,
}

impl<'m> Walk<'m> {
    fn of(model: &'m Model, solid: SolidKey) -> Self {
        Self {
            model,
            solid,
            moved_by: Isometry::identity(),
            turn: Orientation::Forward,
        }
    }

    /// Describe the shape somewhere else, with the movement baked into the
    /// numbers rather than carried as a placement (§42.8).
    fn moved_by(mut self, movement: Isometry) -> Self {
        self.moved_by = movement;
        self
    }

    fn with_every_face_turned_round(mut self) -> Self {
        self.turn = Orientation::Reversed;
        self
    }

    fn describe_again(self, run_of: &dyn Fn(EdgeView<'_>) -> EdgeRun) -> Result<SolidBuilder> {
        let view = self.model.solid(self.solid);
        let mut shells = view.shells();
        let shell = shells.next().expect("a solid has a shell");
        assert!(shells.next().is_none(), "these solids have one shell each");
        assert_eq!(
            shell.location(),
            Isometry::identity(),
            "this walk reads stored geometry, so the shell has to sit at the identity"
        );

        let mut builder = SolidBuilder::new();
        let mut corners: HashMap<VertexKey, DraftVertex> = HashMap::new();
        let mut edges: HashMap<EdgeKey, DraftEdge> = HashMap::new();
        let mut faces = Vec::new();

        for reference in shell.face_refs() {
            assert_eq!(
                reference.location(),
                Isometry::identity(),
                "this walk reads stored geometry, so each face sits at the identity"
            );
            let face = self.model.face(reference.face());
            let mut wires = Vec::new();
            for wire in face.wires() {
                let mut steps = Vec::new();
                for step in wire.steps() {
                    steps.push(self.describe_a_step(
                        step,
                        run_of,
                        &mut builder,
                        &mut corners,
                        &mut edges,
                    )?);
                }
                wires.push(builder.add_wire(&steps)?);
            }
            let orient = if self.turn.is_reversed() {
                reference.orientation().reversed()
            } else {
                reference.orientation()
            };
            faces.push((
                builder.add_face(face.surface().transformed(&self.moved_by), &wires)?,
                orient,
            ));
        }
        builder.add_shell(&faces)?;
        Ok(builder)
    }

    fn describe_a_step(
        &self,
        step: WireStepView<'m>,
        run_of: &dyn Fn(EdgeView<'_>) -> EdgeRun,
        builder: &mut SolidBuilder,
        corners: &mut HashMap<VertexKey, DraftVertex>,
        edges: &mut HashMap<EdgeKey, DraftEdge>,
    ) -> Result<DraftStep> {
        match step {
            WireStepView::Along(coedge) => {
                // `CoedgeView::edge` turns the edge the way the coedge uses it,
                // so its ends are the coedge's ends and not the edge's own.
                // Reading the stored pair back means asking the model for the
                // edge on its own account.
                let stored = self.model.edge(coedge.edge().key());
                let described = match edges.get(&stored.key()) {
                    Some(&described) => described,
                    None => {
                        let start = self.same_corner(builder, corners, stored.start())?;
                        let end = self.same_corner(builder, corners, stored.end())?;
                        let described = builder.add_edge(start, end, run_of(stored))?;
                        edges.insert(stored.key(), described);
                        described
                    }
                };
                Ok(DraftStep::Along(
                    builder.add_coedge(described, coedge.orientation_on_its_edge())?,
                ))
            }
            WireStepView::Pole(vertex) => {
                Ok(DraftStep::Pole(self.same_corner(builder, corners, vertex)?))
            }
            // `WireStepView` grows a variant at every milestone, and a step
            // described as whichever of the two it happens to resemble is a
            // solid that is not the one being copied.
            other => panic!("a wire step nothing here knows: {other:?}"),
        }
    }

    /// The handle for a corner, describing it the first time it is met.
    fn same_corner(
        &self,
        builder: &mut SolidBuilder,
        corners: &mut HashMap<VertexKey, DraftVertex>,
        vertex: VertexView<'_>,
    ) -> Result<DraftVertex> {
        if let Some(&described) = corners.get(&vertex.key()) {
            return Ok(described);
        }
        let described = builder.add_vertex(self.moved_by.apply_to_point(vertex.point()))?;
        corners.insert(vertex.key(), described);
        Ok(described)
    }
}
