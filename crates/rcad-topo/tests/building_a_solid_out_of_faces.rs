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
use rcad_predicates::{Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    Containment, DraftEdge, DraftStep, DraftVertex, EdgeKey, EdgeView, Error, Invalid, Measurement,
    Model, ProfileBuilder, Result, Rule, SolidBuilder, SolidKey, VertexKey, VertexView,
    WireStepView,
};

mod rings;

use rings::{a_bend_turned_and_written_out, a_quarter_bend, the_top_half_of_a_ring};

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

/// What a straight-sided or whole-circle edge runs along, read off the model.
///
/// [`EdgeView::is_closed`] is public and is the whole of what these two shapes
/// need: a cube has only straight edges and a round bar adds two cap circles.
fn a_run_the_model_says(edge: EdgeView<'_>) -> EdgeRun {
    if edge
        .is_closed()
        .expect("a cube and a round bar run straight or right round")
    {
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
/// **no solid this kernel holds fuses with a torus-faced solid into a solid
/// today** — not a plane-bounded block, not a second bend on the bend's own
/// axis, not even a bit-for-bit copy of the bend itself
/// (`a_bend_fused_with_a_copy_of_itself_is_nothing_less_it_and_refused_otherwise`
/// in `how_often_a_fuse_with_a_torus_refuses.rs` is the demonstration: a bend
/// less its copy is answered, and is nothing, and the union and the
/// intersection are refused writing the bend back out).
///
/// So the fuse half of the oracle here compares the **refusal**: the same
/// rule against the same third solid must refuse the same way for both.
/// That still reads topology the measurement never touches — the refusal is
/// raised once the arrangement has placed and ordered the corners the rebuild
/// itself produced, walked round them and cut every face into pieces, so a
/// coedge walked backwards or a picker handed the wrong sign would have to be
/// caught before that walk gets as far as the question both descriptions are
/// shown to answer alike.
///
/// **It used to be which side of a plane a point exactly on it is.** The
/// turned bend's end cap at three eighths of a turn is on the plane
/// `x + y = 0`, which crosses the torus in two circles: the cap's own, and a
/// far one through `(5/√2, −5/√2, 0)`. Both are drawn on the first bend's torus
/// face. Grouping the walks into cells asks which side of the cap circle's
/// plane the far circle's one node is on — and that node is on the far circle,
/// so on the same plane, bit for bit the same coefficients. The true answer is
/// nothing at all, and a constructed point's box, `5.7e-13` wide here, can
/// never certify nothing (§35.2).
///
/// **This used to stop earlier, at `TwoCurvesTouchWithoutCrossing`, and that
/// was the wrong name.** The first bend's end cap on `y = 0` has its corner at
/// `(3, 0, 0)`, on the circle the plane cuts round `(4, 0, 0)`. The same plane
/// cuts a second circle round `(−4, 0, 0)`, whose picker certainly rules the
/// corner out; but the model lists the torus round the corner, and the corner
/// was put on both circles for it. Two ways out along one torus are ones no
/// turn and no bend can tell apart, and the sort refused there. Measured with
/// the old rule put back: the refusal was at `(3, 0, 0)` on that cap, between
/// the cap edge's own circle and the far one, for both descriptions. A corner
/// is on a curve of a surface it lists only where that curve's picker allows
/// now (§224.2's joined cone needed the same), and the walk gets past it.
///
/// **That refusal was a gap in §16.10's sense**: the answer exists and is
/// nothing at all, and the drawing already knows it — the node is on a
/// crossing whose partner *is* the plane asked about. `arrange.rs`'s
/// `which_side_of_everything` now reads that off the node's own crossings
/// rather than asking, and moves on to the next node as it does for any point
/// on the plane.
///
/// **Past it, the ray from `(4, 0, 0)` in the end cap on `y = 0` could not
/// follow the turned bend's torus face.** `holds.rs` follows a bend's wall now
/// (§243): that cap is placed outside the turned bend, which starts an eighth
/// of a turn further round, and the cap on `x = 0` inside it.
///
/// **Both descriptions now stop on the first bend's wall**, at the piece of it
/// between `y = 0` and the turned bend's cap: corners `(3, 0, 0)` and where
/// that cap's circle crosses the outer equator, `(5/√2, 5/√2, 0)`. Every point
/// selection tries in a curved piece is built from its corners, so it is a
/// box and not a point, and a ray from a box cannot be put to a torus —
/// `CannotFireARayAtAFacesSurface` on the turned bend's torus face. Both
/// descriptions refuse it word for word alike, which is what this half of the
/// oracle compares.
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
        .expect("this model made the key")
        .measure(deflection, &policy, &mut log)
        .expect("the bend the rings module built measures");
    let other = model
        .solid(rebuilt)
        .expect("this model made the key")
        .measure(deflection, &policy, &mut log)
        .expect("and so does the one described again");
    assert_eq!(one.volume(), other.volume(), "volume");
    assert_eq!(one.surface_area(), other.surface_area(), "area");
    assert_eq!(one.centroid(), other.centroid(), "centroid");

    // A second bend, sharing the ring, tube and axis exactly and so sharing
    // the torus surface itself, turned an eighth of a turn round the ring so
    // it overlaps the first without being a plain duplicate of it.
    let a_turned_bend_of_the_same_ring_and_tube = |model: &mut Model| -> SolidKey {
        let second = a_quarter_bend(model, 4.0, 1.0);
        model
            .place_solid(
                second,
                &Isometry::rotation(&Axis::z(), core::f64::consts::FRAC_PI_4)
                    .expect("a quarter turn is a real angle"),
            )
            .expect("this model made the key");
        second
    };
    let refusal_of_built =
        refused_fusing_a_third_solid(&model, built, &a_turned_bend_of_the_same_ring_and_tube);
    let refusal_of_rebuilt =
        refused_fusing_a_third_solid(&model, rebuilt, &a_turned_bend_of_the_same_ring_and_tube);
    let Error::Undecidable { records } = &refusal_of_built else {
        panic!("expected the piece of the wall not to be placed, got {refusal_of_built:?}");
    };
    let record = records.records().last().expect("a refusal names why");
    assert_eq!(
        record.predicate(),
        "a point in a piece that every solid could place",
        "{refusal_of_built:?}"
    );
    let [near, far] = record.inputs() else {
        panic!("the piece of the wall has two corners: {refusal_of_built:?}");
    };
    assert_eq!(
        near.enclosure().map(|axis| [axis.lo(), axis.hi()]),
        [[3.0, 3.0], [0.0, 0.0], [0.0, 0.0]],
        "the near corner of the first bend's wall: {refusal_of_built:?}"
    );
    let on_the_outer_equator = 5.0 * core::f64::consts::FRAC_1_SQRT_2;
    let where_it_should_be = [on_the_outer_equator, on_the_outer_equator, 0.0];
    for (axis, should) in far.enclosure().iter().zip(where_it_should_be) {
        assert!(
            axis.lo() <= should && should <= axis.hi(),
            "the turned cap's circle crosses the outer equator at (5/√2, 5/√2, 0): {far:?}"
        );
    }
    assert_eq!(
        format!("{refusal_of_built}"),
        format!("{refusal_of_rebuilt}"),
        "the kernel-built bend and the one described again refused the same fuse differently"
    );

    let mut scratch = model.clone();
    let turned = a_turned_bend_of_the_same_ring_and_tube(&mut scratch);
    let turned = scratch.solid(turned).expect("this model made the key");
    let from_the_cap = turned.classify_uncertain_point(
        PredicatePoint::from([4.0, 0.0, 0.0]),
        &policy,
        &mut ProvenanceLog::new(),
    );
    assert_eq!(
        from_the_cap,
        Containment::Outside,
        "the middle of the end cap on y = 0 is an eighth of a turn short of the turned bend"
    );
    let between_the_corners = PredicatePoint::Constructed(core::array::from_fn(|axis| {
        (near.enclosure()[axis] + far.enclosure()[axis]) * Interval::exactly(0.5)
    }));
    let placed =
        turned.classify_uncertain_point(between_the_corners, &policy, &mut ProvenanceLog::new());
    let Containment::CannotFireARayAtAFacesSurface { face } = placed else {
        panic!("a ray from a blend of the wall piece's corners: {placed:?}");
    };
    let surface = turned
        .faces()
        .find(|each| each.key() == face)
        .expect("the ray names a face of the solid it was fired at")
        .surface();
    assert!(
        matches!(surface, Surface::Torus(_)),
        "the face the ray cannot be fired at: {surface:?}"
    );
}

/// A quarter bend written out where it sits after half a turn round its own
/// axis is the bend that was not turned, and so is a half bend written out a
/// quarter turn round, across `π`.
///
/// Both walls run past `π` round the ring, which is where `atan2` cuts the ring
/// angle. The mesher unwrapped the angle round the tube and took the one round
/// the ring as it came, so it read each wall as the rest of the ring with its
/// seam folded, and the builder refused both
/// `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 1, used: 1 }` (§280).
///
/// **Compared within what the two measurements say they can be out by, and
/// not bit for bit.** The two charts differ by a half or a quarter turn, so
/// `point_at` takes different sines and the mesh points land differently.
/// Each is also held to the volume Pappus gives, `π·r²·θ·R`, so that two
/// turned copies wrong the same way could not pass.
#[test]
fn a_bend_turned_past_where_the_ring_angle_is_cut_is_the_bend_that_was_not_turned() {
    let policy = Policy::certified();
    let (ring, tube) = (4.0, 1.0);
    for (quarters, turns) in [(1, 2), (2, 1)] {
        let mut log = ProvenanceLog::new();
        let mut model = Model::new(a_working_size());
        let unturned = a_bend_turned_and_written_out(&mut model, ring, tube, quarters, 0)
            .expect("the bend that was not turned is a solid");
        let turned = match a_bend_turned_and_written_out(&mut model, ring, tube, quarters, turns) {
            Ok(turned) => turned,
            Err(refused) => panic!(
                "a bend {quarters} quarter turns long, turned {turns} quarter turns and \
                 written out, was refused: {refused:?}"
            ),
        };
        model
            .validate(&policy, &mut log)
            .expect("the model holding both is sound");

        let deflection = Deflection::for_scale(model.scale());
        let [one, other] = [unturned, turned].map(|key| {
            model
                .solid(key)
                .expect("this model made the key")
                .measure(deflection, &policy, &mut log)
                .expect("a bend measures")
        });
        let apart = (one.volume() - other.volume()).abs();
        assert!(
            apart <= one.volume_error() + other.volume_error(),
            "{quarters} quarters turned {turns}: {} against {}, {apart} apart where the two \
             measurements allow {} and {}",
            other.volume(),
            one.volume(),
            other.volume_error(),
            one.volume_error()
        );
        let pappus = core::f64::consts::PI
            * tube
            * tube
            * (f64::from(quarters) * core::f64::consts::FRAC_PI_2)
            * ring;
        for (which, measured) in [("unturned", one), ("turned", other)] {
            assert!(
                (measured.volume() - pappus).abs() <= measured.volume_error(),
                "{quarters} quarters, {which}: {} where Pappus gives {pappus}, allowed {}",
                measured.volume(),
                measured.volume_error()
            );
        }
    }
}

/// The top half of a ring described with one wire per equator is refused by
/// name, because each of those wires climbs a whole turn round the ring and a
/// loop in the chart has to close.
///
/// That is a wire that will not lie flat on its face, the name a wire that
/// winds round the tube where it may not has always had (§280.6). Before the
/// ring angle was unwrapped, each equator came back to the corner it left
/// with no turn counted, and the builder refused the half ring
/// `TwoPatchesOfTheMeshDrewTheSameSide { drawn: 3, used: 3 }`.
#[test]
fn the_top_half_of_a_ring_with_no_seam_climbs_a_whole_turn_round_it_and_is_refused() {
    let mut model = Model::new(a_working_size());
    let refused = the_top_half_of_a_ring(&mut model, 4.0, 1.0, false);
    assert!(
        matches!(refused, Err(Error::AWireWillNotLieFlatOnItsFace { .. })),
        "the top half of a ring with no seam came back {refused:?}"
    );
    assert_eq!(model.solid_count(), 0, "a refused build writes nothing");
}

/// The same half ring with a seam across the tube, which is how a file writes a
/// face right round the ring, is refused by the same name: its one wire passes
/// the seam's points a whole turn apart round the ring.
///
/// **This refuses a solid that exists, and on purpose.** With that half of the
/// refusal switched off this very shape built and measured `38.396` where
/// Pappus, `π²·r²·R`, gives `39.478`, inside its stated `8.742`; the crown of
/// a ring in `a_mesh_that_folds_at_one_deflection_and_not_the_next.rs` folded
/// instead. Nothing cuts a patch narrower round the ring, so the mesher is
/// not trusted with a patch a whole turn wide that way (§280.6, and
/// `WalkingAWire::goes_right_round_the_second_way` in `mesh.rs`). A gap, and
/// this test is what moves when it is closed.
#[test]
fn the_top_half_of_a_ring_with_a_seam_across_the_tube_is_refused_by_the_same_name() {
    let mut model = Model::new(a_working_size());
    let refused = the_top_half_of_a_ring(&mut model, 4.0, 1.0, true);
    assert!(
        matches!(refused, Err(Error::AWireWillNotLieFlatOnItsFace { .. })),
        "the top half of a ring with a seam came back {refused:?}"
    );
    assert_eq!(model.solid_count(), 0, "a refused build writes nothing");
}

/// The seam of a bend, named from geometry a file carries rather than from
/// the model — exactly `rings::a_quarter_bend`'s own construction of it,
/// which is what a reader reconstructing a `surface_curve` off a torus does.
fn a_seam_of_a_bend_named_the_way_a_file_names_one(edge: EdgeView<'_>) -> EdgeRun {
    if edge
        .is_closed()
        .expect("a bend's edges are its seam and its two end circles")
    {
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
    assert!(
        !edge
            .is_closed()
            .expect("a ball's one edge is part of a crossing"),
        "a ball's one edge is its seam"
    );
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
///
/// **The name is `AFlatFaceIsUsedReversed` and not `VolumeIsNotPositive`**,
/// which is the more useful half of the two: a cube described inside out has
/// negative volume *and* six planes used reversed, and only the second says
/// which rule was broken. Turning one wall round instead leaves the volume
/// positive and is caught by that name alone — `a_flat_face_used_reversed` in
/// `validate.rs` is where that lives.
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
            Err(Error::Invalid(Invalid::AFlatFaceIsUsedReversed))
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
        .expect("this model made the key")
        .measure(deflection, &policy, &mut log)
        .expect("the solid the kernel built measures");
    let other = model
        .solid(rebuilt)
        .expect("this model made the key")
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
                .solid(piece)?
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
        let view = self.model.solid(self.solid)?;
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
            let face = self.model.face(reference.face())?;
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
                let stored = self.model.edge(coedge.edge().key())?;
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
