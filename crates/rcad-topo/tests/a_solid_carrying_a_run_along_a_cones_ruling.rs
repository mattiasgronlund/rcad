//! A solid whose edges are the two rulings a plane through a cone's apex cuts,
//! put through every part of `rcad-topo` that reads a run.
//!
//! `intersect` used to refuse this pair by name, so no model could hold one of
//! these runs and nothing downstream had ever seen one. It answers now — two
//! straight runs told apart by a picker each — and `EdgeRun` is
//! `#[non_exhaustive]`, so **not one line of `rcad-topo` had to change and not
//! one would have failed to compile if it should have.** That is the whole
//! reason this file exists: the compiler cannot say whether the rest of the
//! kernel copes, so a solid carrying such a run is built and asked.
//!
//! # The solid
//!
//! Half a frustum: the `x ≥ 0` half of a cone of slope a half, between `z = 4`
//! and `z = 8`. Four faces, six edges, four corners, and `4 − 6 + 4 = 2`.
//!
//! The flat face is the plane `x = 0`, which runs **down the cone's own axis**,
//! so it passes through the apex and leans more steeply than the cone's side.
//! That is row 7 of `the_seven_shapes_a_plane_cuts_a_cone_in`: the crossing is
//! the two rulings `(0, ±v/2, v)`, and the two edges up the sides of this solid
//! are pieces of them. The apex itself is nowhere near the solid — a frustum
//! stops short of it — which is the point: an *apex run* is a run named by a
//! plane through the apex, and where the solid sits on it is topology's business
//! and not the naming's.
//!
//! Every number here is exact. The slope is a half, so `slope · height` rounds
//! to nothing at either cap, and the two ruling directions `(0, ±1/2, 1)` are
//! stored triples that satisfy the cone's own equation with nothing left over —
//! which is what lets `intersect` name the pair at all.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, HalfSpace, NamedRun, Orientation, Picker, Plane, Point, Scale,
    Surface, SurfaceMeet, Vector, intersect,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, DraftStep, Error, Model, SolidBuilder, SolidKey};

mod cones;

/// The cone the half-frustum is cut out of: slope a half about `+z`, apex at
/// the origin.
fn the_cone() -> Cone {
    Cone::new(Axis::z(), 0.5).expect("a positive slope")
}

/// The flat face's own plane, facing out of the solid.
fn the_cut_plane() -> Plane {
    Plane::from_point_and_normal(Point::origin(), Vector::new(-1.0, 0.0, 0.0))
        .expect("the x axis is a unit normal")
}

/// The two runs `intersect` names for the cut plane against the cone, in the
/// order it hands them back.
fn the_two_rulings() -> Vec<NamedRun> {
    let SurfaceMeet::Curves(runs) = intersect(
        &Surface::Plane(the_cut_plane()),
        &Surface::Cone(the_cone()),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a plane down a cone's axis cuts it in two straight lines") else {
        panic!("two rulings are two curves");
    };
    assert_eq!(runs.len(), 2);
    runs
}

/// Whether every picker on `run` holds `point`, which is how a run says a point
/// is on its own piece of the crossing.
fn holds(run: &NamedRun, point: [f64; 3]) -> bool {
    let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
        panic!("a ruling through the apex is a piece of where the two surfaces cross");
    };
    picked_by.iter().all(|picker| {
        picker.side_of_point(PredicatePoint::from(point)) != rcad_predicates::Decision::Neg
    })
}

/// Half a frustum, described face by face through the public builder.
///
/// The two side edges are the runs `intersect` names, taken from it rather than
/// written out here — a test that spelled its own pickers would be checking
/// itself.
fn a_half_frustum(model: &mut Model) -> SolidKey {
    let skin = the_cone();
    let runs = the_two_rulings();
    // Which run is the `y > 0` line and which the `y < 0` one is the pickers'
    // to say, not the order's.
    let up_the_positive_side = runs
        .iter()
        .find(|run| holds(run, [0.0, 2.0, 4.0]) && !holds(run, [0.0, -2.0, 4.0]))
        .expect("one of the two runs holds the ruling at y > 0 and not the other")
        .run()
        .clone();
    let up_the_negative_side = runs
        .iter()
        .find(|run| holds(run, [0.0, -2.0, 4.0]) && !holds(run, [0.0, 2.0, 4.0]))
        .expect("and the other holds the ruling at y < 0")
        .run()
        .clone();

    let mut builder = SolidBuilder::new();
    let corner = |builder: &mut SolidBuilder, y: f64, z: f64| {
        builder
            .add_vertex(Point::new(0.0, y, z))
            .expect("a finite point")
    };
    let bottom_negative = corner(&mut builder, -2.0, 4.0);
    let bottom_positive = corner(&mut builder, 2.0, 4.0);
    let top_positive = corner(&mut builder, 4.0, 8.0);
    let top_negative = corner(&mut builder, -4.0, 8.0);

    // Each cap plane cuts the whole circle; the half of it this solid has is the
    // half with `x ≥ 0`, and one half-space says so.
    let round_the_front = || EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(the_cut_plane()),
        picked_by: smallvec::smallvec![Picker::from(
            HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
                .expect("the x axis has length")
        )],
    };
    let bottom_arc = builder
        .add_edge(bottom_positive, bottom_negative, round_the_front())
        .expect("half of the bottom cap circle");
    let top_arc = builder
        .add_edge(top_negative, top_positive, round_the_front())
        .expect("half of the top cap circle");
    let up_the_positive = builder
        .add_edge(bottom_positive, top_positive, up_the_positive_side)
        .expect("a piece of the ruling at y > 0");
    let up_the_negative = builder
        .add_edge(bottom_negative, top_negative, up_the_negative_side)
        .expect("a piece of the ruling at y < 0");
    let bottom_across = builder
        .add_edge(bottom_negative, bottom_positive, EdgeRun::Straight)
        .expect("the bottom cap's own straight side");
    let top_across = builder
        .add_edge(top_positive, top_negative, EdgeRun::Straight)
        .expect("the top cap's own straight side");

    let wire_of = |builder: &mut SolidBuilder, steps: &[(_, Orientation)]| {
        let along: Vec<DraftStep> = steps
            .iter()
            .map(|(edge, turn)| {
                DraftStep::Along(
                    builder
                        .add_coedge(*edge, *turn)
                        .expect("an edge of this build"),
                )
            })
            .collect();
        builder.add_wire(&along).expect("a closed wire")
    };

    let bottom_wire = wire_of(
        &mut builder,
        &[
            (bottom_arc, Orientation::Forward),
            (bottom_across, Orientation::Forward),
        ],
    );
    let top_wire = wire_of(
        &mut builder,
        &[
            (top_arc, Orientation::Forward),
            (top_across, Orientation::Forward),
        ],
    );
    let skin_wire = wire_of(
        &mut builder,
        &[
            (bottom_arc, Orientation::Reversed),
            (up_the_positive, Orientation::Forward),
            (top_arc, Orientation::Reversed),
            (up_the_negative, Orientation::Reversed),
        ],
    );
    let cut_wire = wire_of(
        &mut builder,
        &[
            (up_the_negative, Orientation::Forward),
            (top_across, Orientation::Reversed),
            (up_the_positive, Orientation::Reversed),
            (bottom_across, Orientation::Reversed),
        ],
    );

    let cap = |builder: &mut SolidBuilder, at: f64, outward: f64, wire| {
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, at), Vector::new(0.0, 0.0, outward))
                .expect("the z axis is a unit normal");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a half-disc cap")
    };
    let bottom = cap(&mut builder, 4.0, -1.0, bottom_wire);
    let top = cap(&mut builder, 8.0, 1.0, top_wire);
    let side = builder
        .add_face(Surface::Cone(skin), &[skin_wire])
        .expect("half the cone's skin");
    let flat = builder
        .add_face(Surface::Plane(the_cut_plane()), &[cut_wire])
        .expect("the flat face down the axis");

    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (side, Orientation::Forward),
            (flat, Orientation::Forward),
        ])
        .expect("four faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("half a frustum described this way is a solid")
}

/// The full sweep accepts a solid whose side edges are apex runs, and neither
/// of them reads as straight or as closed.
///
/// The second half is the one the compiler could not have caught. A dozen places
/// in `rcad-topo` ask a run `== EdgeRun::Straight` or
/// `== EdgeRun::RightRoundWhereItsFacesCross` and take the `false` branch for
/// everything else; a ruling is a straight *line* and would be a plausible thing
/// to have named `Straight`, which would put a chord where the cone's own side
/// is. It is named as a piece of a crossing instead, and this says so.
#[test]
fn a_solid_whose_sides_are_a_cones_rulings_passes_the_full_sweep() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_half_frustum(&mut model);
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("half a frustum is a sound solid");

    let apex_runs = model
        .solid(solid)
        .faces()
        .flat_map(|face| face.wires())
        .flat_map(|wire| wire.coedges())
        .map(|coedge| coedge.edge())
        .filter(|edge| {
            matches!(
                edge.run(),
                EdgeRun::PartOfWhereItCrosses {
                    and: Surface::Cone(_),
                    ..
                }
            )
        })
        .count();
    assert!(
        apex_runs > 0,
        "the two side edges are runs a plane through the apex named"
    );

    for face in model.solid(solid).faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if !matches!(
                    edge.run(),
                    EdgeRun::PartOfWhereItCrosses {
                        and: Surface::Cone(_),
                        ..
                    }
                ) {
                    continue;
                }
                assert!(
                    !edge
                        .is_closed()
                        .expect("a cone's edges are runs this kernel named"),
                    "a ruling does not come back to where it started"
                );
                assert_ne!(
                    edge.run(),
                    EdgeRun::Straight,
                    "a ruling is a straight line and is still not `EdgeRun::Straight`: the run \
                     names which of the two lines this is, and dropping that names neither"
                );
            }
        }
    }
}

/// The same solid meshed: the mesher walks the two rulings without a panic, and
/// what comes out holds half a frustum's closed-form volume.
#[test]
fn a_solid_carrying_apex_runs_meshes_to_half_a_frustums_volume() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_half_frustum(&mut model);

    let measured = model
        .solid(solid)
        .measure(
            Deflection::of(0.002).expect("a positive length"),
            &policy,
            &mut log,
        )
        .expect("half a frustum measures");
    let (small, big, height) = (2.0f64, 4.0f64, 4.0f64);
    let whole = core::f64::consts::PI * height * (big * big + big * small + small * small) / 3.0;
    assert!(
        (measured.volume() - whole / 2.0).abs() < 0.01,
        "half a frustum holds half its volume, {} against {}",
        measured.volume(),
        whole / 2.0
    );

    let _ = &mut log;
}

/// **Point-in-solid refuses this solid, and the face it names is not one of the
/// two the apex runs bound.** Measured, not read off the code.
///
/// `holds.rs`'s `walk_one_outline` walks a face's outline as a polygon and
/// refuses the moment a coedge's run is anything but [`EdgeRun::Straight`]. What
/// that catches here is the **top cap**, a plain flat face trimmed by half a
/// circle — an arc, which is a [`EdgeRun::PartOfWhereItCrosses`] and has two
/// different ends. The cone face and the flat face carrying the two rulings are
/// never reached.
///
/// The whole frustum, built from the same cone by the same builder, **answers
/// `Inside`**, because its non-straight edges are all *closed* and a wire that
/// is one closed edge takes a different route. So the gap is arcs, not cones and
/// not apex runs: any solid with an arc-trimmed face has always refused here, and
/// a half-frustum is the first one this repo builds.
///
/// The classifier says which face it could not follow and hands that back as an
/// answer of its own rather than a panic, which is the rule this file exists to
/// check.
#[test]
fn an_arc_trimmed_face_refuses_point_in_solid_by_name_and_a_closed_one_does_not() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let half = a_half_frustum(&mut model);
    let whole = cones::a_frustum(&mut model, 0.5, 4.0, 8.0);
    let inside_both = Point::new(1.0, 0.0, 6.0);

    let answer = model
        .solid(half)
        .classify_point(inside_both, &policy, &mut log);
    let Containment::CannotFollowAFacesTrimming { face } = answer else {
        panic!("half a frustum: expected a named refusal, got {answer:?}");
    };
    let trimmed_by_an_arc = model
        .face(face)
        .wires()
        .flat_map(|wire| wire.coedges())
        .any(|coedge| {
            let edge = coedge.edge();
            !edge
                .is_closed()
                .expect("a cone's edges are runs this kernel named")
                && edge.run() != EdgeRun::Straight
        });
    assert!(
        trimmed_by_an_arc,
        "the refusal names the face whose outline it could not walk, and what it could not walk \
         is an arc: {:?}",
        model.face(face).surface()
    );

    assert_eq!(
        model
            .solid(whole)
            .classify_point(inside_both, &policy, &mut log),
        Containment::Inside,
        "the whole frustum, same cone and same builder, is decided — so what stops the half is \
         the arc and not the cone"
    );
}

/// The fuse is put to a solid carrying two apex runs, and whatever it does it
/// does **without a panic**.
///
/// This is the row of the sweep that could not be settled by reading. Two sites
/// in the arrangement — `crossing_for` and `cuts_between` — end in
/// `panic!("two surfaces crossed in a way nothing here knows")` on a
/// [`SurfaceMeet`] they do not recognise, and two more hand back **no pickers at
/// all** for any run that is not `Straight` or closed. None of that is a compile
/// error, so it is asked here instead.
///
/// **Measured: all three rules come back `Undecidable`, and nothing panics.**
/// The pair never reaches the apex arms — these two solids share the cone
/// surface exactly, which the arrangement gives up on first — so what this pins
/// is the rule and not a rate: an answer is a sound model, anything else is
/// named, and a panic is neither.
#[test]
fn a_fuse_with_an_apex_run_in_it_either_answers_soundly_or_refuses_by_name() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let half = a_half_frustum(&mut model);
    // The whole frustum the half was cut out of: its cone face against the
    // half's flat one is exactly the pair whose two apex arms this unit landed,
    // so the arrangement is put to that pair rather than to two coincident
    // copies of one solid.
    let other_half = cones::a_frustum(&mut model, 0.5, 4.0, 8.0);

    for rule in [
        rcad_topo::Rule::Union,
        rcad_topo::Rule::Intersection,
        rcad_topo::Rule::Difference,
    ] {
        let mut scratch = model.clone();
        let keys: Vec<_> = scratch.solids().map(|solid| solid.key()).collect();
        match scratch.fuse(
            &[keys[0]],
            &[keys[1]],
            rule,
            &policy,
            &mut ProvenanceLog::new(),
        ) {
            Ok(_) => scratch
                .validate(&policy, &mut ProvenanceLog::new())
                .expect("a fuse hands back a sound model or refuses"),
            Err(Error::Undecidable { .. }) => println!("{rule:?}: undecidable"),
            Err(refusal) => println!("{rule:?}: {refusal}"),
        }
    }
    let _ = (half, other_half);
}
