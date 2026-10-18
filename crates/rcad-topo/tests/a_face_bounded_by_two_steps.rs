//! A face bounded by two steps — an arc and the chord that closes it.
//!
//! `docs/decisions.md` §44.4 stage 2, and §44.11 item 3 for why this is not one
//! rule: the length floor lives in two places, and the winding a flat face is
//! judged by reads a wire's *corners*, which a two-corner boundary cannot tell
//! apart from a hole.
//!
//! **Why these are not tautologies.** Four of them are refusals, and each is a
//! two-step wire that bounds nothing. Two the length rule turns away by name:
//! the same edge walked out and back, and two straight edges between the same
//! two corners. Two more it cannot see and the sweep catches instead: a face
//! whose two edges name the same curve, which needs the faces either side to
//! tell, and a face put on back to front, which is the winding this lane stops
//! asking a two-corner boundary about. The accepting test is the shape that
//! does bound something: a plate cut down to the segment between an arc and its
//! chord, the
//! smallest solid with a two-step boundary, whose numbers are exact for any
//! radius and height an `f64` holds. Its corners sit at `(±r, 0, z)`, so
//! `x² + y² − r²` cancels to nothing and the sweep certifies every one of them
//! with nothing approximated — which is the assertion that says this passed for
//! the right reason rather than because a check gave up.
//!
//! The sweep's own copy of the length rule is asked the same two questions
//! where it lives, in `validate.rs`, because a model arrives off the wire as
//! well as out of a builder and those are two doors.

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftCoedge, DraftEdge, DraftStep, DraftVertex, DraftWire, Error, Invalid, Model, Result,
    SolidBuilder, SolidKey,
};

const RADIUS: f64 = 3.0;
const HEIGHT: f64 = 5.0;

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

/// The sentence §44.10 asks this lane for.
#[test]
fn a_plate_bounded_by_an_arc_and_its_chord() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());

    a_segment_plate(
        &mut model,
        &policy,
        &mut log,
        Spoilt::Nothing,
        Arc::HalfATurn,
    )
    .expect("an arc and its chord bound a face");

    model
        .validate(&policy, &mut log)
        .expect("and the full sweep is happy with it");
    assert!(
        log.records().is_empty(),
        "a segment plate on the z axis needs nothing approximated, got {:?}",
        log.records()
    );
}

/// The two flat faces are the two-step ones; the wall and the chord face have
/// four steps each. Without this the accepting test above would still pass on a
/// plate that quietly grew a third step somewhere.
#[test]
fn the_plates_two_flat_faces_are_bounded_by_two_steps_each() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let plate = a_segment_plate(
        &mut model,
        &policy,
        &mut log,
        Spoilt::Nothing,
        Arc::HalfATurn,
    )
    .expect("an arc and its chord bound a face");

    let mut lengths: Vec<usize> = model
        .solid(plate)
        .expect("this model made the key")
        .shells()
        .flat_map(|shell| shell.faces())
        .map(|face| face.outer_wire().step_count())
        .collect();
    lengths.sort_unstable();

    assert_eq!(lengths, vec![2, 2, 4, 4], "two arc-and-chord faces");
}

/// The half of the winding question this lane could answer, demonstrated
/// rather than argued.
///
/// `check_flat_face` reads which way a wire goes off its corner polygon, and a
/// two-corner boundary sweeps `(p × q) · n + (q × p) · n`, which is exactly
/// zero however the face is turned. So the corners cannot say which way this
/// face points and the sweep skips them — the same exception a round cap's
/// one-corner boundary already gets. What still catches the face put on back to
/// front is the mesh, where it does have a polygon and a neighbour to disagree
/// with.
#[test]
fn a_two_step_face_turned_round_is_still_caught_by_the_mesh() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());

    let refused = a_segment_plate(
        &mut model,
        &policy,
        &mut log,
        Spoilt::TheBottomFaceIsTurnedRound,
        Arc::HalfATurn,
    );

    assert!(
        matches!(
            refused,
            Err(Error::Invalid(Invalid::FacesDoNotAgreeWhichWayTheyFace))
        ),
        "got {refused:?}"
    );
}

/// The two-step wire the length rule deliberately does **not** catch, and what
/// does catch it.
///
/// An edge names a piece of where the faces **either side** of it cross, so two
/// edges carrying equal [`EdgeRun`]s are the same curve only when they have the
/// same two faces — the sound plate's arc and its chord carry equal runs and are
/// a circle and a straight line. A wire is described before the face it trims,
/// so `SolidBuilder::add_wire` cannot tell the two apart and does not try. Give
/// the chord the pipe on its far side as well and it really is the arc again:
/// the flat face is one curve out and back, with no area at all.
///
/// The sweep's mesh is what says so. It walks each copy of the arc into a
/// point at the arc's middle of its own, two table entries at one place, so
/// the bottom face's loop is `A`, `M`, `B`, `M′`. The mesh cuts a loop that
/// names one place twice into two lobes there, and both lobes here are two
/// corners, so the face is drawn as the two triangles that bridge the cut and
/// nothing else, and those cover nothing. Before the mesh cut such loops it
/// refused this one `CornersCoincide`; now the skin it makes does not use
/// every side once each way, and the plate is refused for that, as the face
/// turned round above is.
#[test]
fn a_two_step_face_whose_two_edges_are_the_same_curve_is_caught_by_the_mesh() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());

    let refused = a_segment_plate(
        &mut model,
        &policy,
        &mut log,
        Spoilt::TheChordIsTheArcAgain,
        Arc::HalfATurn,
    );

    assert!(
        matches!(
            refused,
            Err(Error::Invalid(Invalid::FacesDoNotAgreeWhichWayTheyFace))
        ),
        "got {refused:?}"
    );
}

/// The same plate on a quarter arc, which is the shape that found the floor.
///
/// `rcad_geom::walk`'s `how_many_steps` took the larger of two counts and then a
/// floor of **one** step. How far the arc *bends* asks for more steps as the
/// deflection tightens, so a fine mesh did put points on a quarter arc; how far
/// it *turns* asks for one, and at the deflection `Model::validate` meshes with
/// — `as_coarse_as_a_mesh_gets`, which is `f64::MAX` — the bending count was one
/// as well. So the sweep's own mesh put no point between the ends of an arc of a
/// quarter turn or less, the flat face's boundary was two corners, and
/// `rcad-mesh` refused it as a loop rather than triangulating a face that was
/// not there. Refusing what it was handed was right; handing it that was the
/// gap.
///
/// The floor is now `FEWEST_STEPS_THAT_SHOW_A_BEND`, which is two, because one
/// piece shows none of the bend for any arc at any deflection and two shows
/// three quarters of it. **This test is what fails when that floor goes back to
/// one**, at the `TooFewCorners { count: 2 }` it used to fail with.
#[test]
fn a_plate_bounded_by_a_quarter_arc_and_its_chord_meshes_too() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());

    a_segment_plate(
        &mut model,
        &policy,
        &mut log,
        Spoilt::Nothing,
        Arc::AQuarterTurn,
    )
    .expect("a quarter arc and its chord bound a face too");

    model
        .validate(&policy, &mut log)
        .expect("and the full sweep is happy with it");
}

#[test]
fn a_wire_of_two_steps_out_and_back_along_one_edge_is_refused() {
    let mut builder = SolidBuilder::new();
    let start = a_corner(&mut builder, 0.0);
    let end = a_corner(&mut builder, 1.0);
    let edge = builder
        .add_edge(start, end, EdgeRun::Straight)
        .expect("two different corners");
    let there = builder
        .add_coedge(edge, Orientation::Forward)
        .expect("the edge is here");
    let back = builder
        .add_coedge(edge, Orientation::Reversed)
        .expect("the edge is here");

    let refused = builder.add_wire(&[DraftStep::Along(there), DraftStep::Along(back)]);

    assert!(
        matches!(refused, Err(Error::ADraftWireIsTooShort { steps: 2 })),
        "got {refused:?}"
    );
}

/// The case that says the rule is about the *edges* and not about the count.
///
/// Two straight edges between the same two corners are the same segment written
/// twice, so this closes, has two steps and names two different edges — and
/// still bounds nothing.
#[test]
fn a_wire_of_two_straight_steps_between_the_same_two_corners_is_refused() {
    let mut builder = SolidBuilder::new();
    let start = a_corner(&mut builder, 0.0);
    let end = a_corner(&mut builder, 1.0);
    let there = an_edge_walked(
        &mut builder,
        start,
        end,
        EdgeRun::Straight,
        Orientation::Forward,
    );
    let back = an_edge_walked(
        &mut builder,
        start,
        end,
        EdgeRun::Straight,
        Orientation::Reversed,
    );

    let refused = builder.add_wire(&[DraftStep::Along(there), DraftStep::Along(back)]);

    assert!(
        matches!(refused, Err(Error::ADraftWireIsTooShort { steps: 2 })),
        "got {refused:?}"
    );
}

fn a_corner(builder: &mut SolidBuilder, x: f64) -> DraftVertex {
    builder
        .add_vertex(Point::new(x, 0.0, 0.0))
        .expect("a finite corner")
}

fn an_edge_walked(
    builder: &mut SolidBuilder,
    start: DraftVertex,
    end: DraftVertex,
    run: EdgeRun,
    orientation: Orientation,
) -> DraftCoedge {
    let edge = builder
        .add_edge(start, end, run)
        .expect("two different corners");
    builder
        .add_coedge(edge, orientation)
        .expect("the edge is here")
}

/// What is wrong with the plate this time, if anything.
///
/// One thing changes between the sound build and each spoilt one, so a refusal
/// is that thing and nothing else.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Spoilt {
    Nothing,
    /// The bottom face's plane faces into the plate rather than out of it.
    TheBottomFaceIsTurnedRound,
    /// The chord is a second copy of the arc: same two corners, same run, and
    /// the face on its other side is the same pipe, so it names the same curve.
    TheChordIsTheArcAgain,
}

/// A plate whose top and bottom are the piece of disc between an arc of
/// `RADIUS` and its chord, standing `HEIGHT` tall on the ground plane.
///
/// Four corners, six edges, four faces. The two flat faces are the point: each
/// is bounded by an arc and a straight, which is two steps. The wall is a piece
/// of pipe and the chord face is a rectangle, four steps each.
fn a_segment_plate(
    model: &mut Model,
    policy: &Policy,
    log: &mut ProvenanceLog,
    spoilt: Spoilt,
    arc: Arc,
) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();

    let along_x = builder.add_vertex(Point::new(RADIUS, 0.0, 0.0))?;
    let along_y = builder.add_vertex(arc.the_far_end(0.0))?;
    let above_x = builder.add_vertex(Point::new(RADIUS, 0.0, HEIGHT))?;
    let above_y = builder.add_vertex(arc.the_far_end(HEIGHT))?;

    let ground = a_plane_at(0.0, Vector::new(0.0, 0.0, -1.0));
    let lid = a_plane_at(HEIGHT, Vector::new(0.0, 0.0, 1.0));
    let chord_runs = match spoilt {
        Spoilt::TheChordIsTheArcAgain => [arc.running_in(&ground), arc.running_in(&lid)],
        Spoilt::Nothing | Spoilt::TheBottomFaceIsTurnedRound => {
            [EdgeRun::Straight, EdgeRun::Straight]
        }
    };
    let arc_below = builder.add_edge(along_x, along_y, arc.running_in(&ground))?;
    let arc_above = builder.add_edge(above_x, above_y, arc.running_in(&lid))?;
    let [below_run, above_run] = chord_runs;
    let chord_below = builder.add_edge(along_x, along_y, below_run)?;
    let chord_above = builder.add_edge(above_x, above_y, above_run)?;
    let up_at_x = builder.add_edge(along_x, above_x, EdgeRun::Straight)?;
    let up_at_y = builder.add_edge(along_y, above_y, EdgeRun::Straight)?;

    // Seen from below, which is the way the ground plane faces, the boundary
    // runs from the corner on `y` round the arc to the corner on `x` and back
    // along the chord. The lid faces the other way and runs the other way.
    let below = a_wire(
        &mut builder,
        &[
            (arc_below, Orientation::Reversed),
            (chord_below, Orientation::Forward),
        ],
    )?;
    let above = a_wire(
        &mut builder,
        &[
            (arc_above, Orientation::Forward),
            (chord_above, Orientation::Reversed),
        ],
    )?;
    // Round the bottom arc, up the seam at `y`, back round the top arc, down
    // the seam at `x`: anticlockwise about the outward radius.
    let wall = a_wire(
        &mut builder,
        &[
            (arc_below, Orientation::Forward),
            (up_at_y, Orientation::Forward),
            (arc_above, Orientation::Reversed),
            (up_at_x, Orientation::Reversed),
        ],
    )?;
    let flat_side = a_wire(
        &mut builder,
        &[
            (chord_below, Orientation::Reversed),
            (up_at_x, Orientation::Forward),
            (chord_above, Orientation::Forward),
            (up_at_y, Orientation::Reversed),
        ],
    )?;

    let facing = match spoilt {
        Spoilt::TheBottomFaceIsTurnedRound => Vector::new(0.0, 0.0, 1.0),
        Spoilt::Nothing | Spoilt::TheChordIsTheArcAgain => Vector::new(0.0, 0.0, -1.0),
    };
    let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), RADIUS).expect("a positive radius"));
    let along_the_chord = match spoilt {
        Spoilt::TheChordIsTheArcAgain => pipe,
        Spoilt::Nothing | Spoilt::TheBottomFaceIsTurnedRound => {
            Surface::Plane(arc.the_chords_plane())
        }
    };
    let below = builder.add_face(Surface::Plane(a_plane_at(0.0, facing)), &[below])?;
    let above = builder.add_face(Surface::Plane(lid), &[above])?;
    let wall = builder.add_face(pipe, &[wall])?;
    let flat_side = builder.add_face(along_the_chord, &[flat_side])?;

    builder.add_shell(&[
        (below, Orientation::Forward),
        (above, Orientation::Forward),
        (wall, Orientation::Forward),
        (flat_side, Orientation::Forward),
    ])?;
    builder.build(model, policy, log)
}

fn a_plane_at(height: f64, facing: Vector) -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, height), facing)
        .expect("an axis direction has length")
}

/// How far round the plate's arc goes: from `(r, 0)` to `(−r, 0)` the long way
/// over `y`, or only to `(0, r)`.
///
/// **Which one is not a taste, it is a finding.** At the deflection the sweep
/// meshes with — as coarse as a mesh gets — `rcad_geom::walk` steps an arc a
/// quarter turn at a time, so a half-turn arc gets one point on it and fans into
/// a triangle. A quarter-turn arc used to get none and fan into two corners
/// `rcad-mesh` would not take as a loop; the walk now holds every curved run to
/// `FEWEST_STEPS_THAT_SHOW_A_BEND` pieces, so it gets one point too. Both are
/// built here because the two counts come from different halves of the step
/// rule — the half turn from how far it turns, the quarter from the floor — and
/// a change to either shows up in exactly one of them.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Arc {
    HalfATurn,
    AQuarterTurn,
}

impl Arc {
    /// The corner the arc finishes at, at `height`. It starts at `(r, 0)`.
    fn the_far_end(self, height: f64) -> Point {
        match self {
            Arc::HalfATurn => Point::new(-RADIUS, 0.0, height),
            Arc::AQuarterTurn => Point::new(0.0, RADIUS, height),
        }
    }

    /// The piece of the circle where `plane` cuts the wall that runs between
    /// the two corners.
    fn running_in(self, plane: &Plane) -> EdgeRun {
        let mut picked_by = smallvec::smallvec![a_half_space_facing(0.0, 1.0)];
        if self == Arc::AQuarterTurn {
            picked_by.push(a_half_space_facing(1.0, 0.0));
        }
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(*plane),
            picked_by,
        }
    }

    /// The plane the chord runs in, facing back the way the arc came — which is
    /// out of the plate, because the plate is the side of the chord the arc
    /// bulges into.
    ///
    /// Both corners read a signed distance of exactly zero from it. Down `y`
    /// that is plain. Across the diagonal it is because the normal's two
    /// components are the same number and each corner has the radius against
    /// one of them and nothing against the other.
    fn the_chords_plane(self) -> Plane {
        let facing = match self {
            Arc::HalfATurn => Vector::new(0.0, -1.0, 0.0),
            Arc::AQuarterTurn => Vector::new(-1.0, -1.0, 0.0),
        };
        Plane::from_point_and_normal(Point::new(RADIUS, 0.0, 0.0), facing)
            .expect("a direction with length")
    }
}

fn a_half_space_facing(x: f64, y: f64) -> Picker {
    Picker::from(
        HalfSpace::through_a_point_facing(Point::origin(), Vector::new(x, y, 0.0))
            .expect("an axis direction has length"),
    )
}

fn a_wire(builder: &mut SolidBuilder, steps: &[(DraftEdge, Orientation)]) -> Result<DraftWire> {
    let mut walked = Vec::with_capacity(steps.len());
    for &(edge, orientation) in steps {
        walked.push(DraftStep::Along(builder.add_coedge(edge, orientation)?));
    }
    builder.add_wire(&walked)
}
