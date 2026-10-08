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
//!
//! A quarter of a pointed cone is built the same way, with a ruling from each
//! of two planes through the apex, to ask what two such cuts on one cone face
//! cost (`docs/decisions.md` §389). The two meet along the axis, inside the
//! cone, so its skin is one run of rulings and answers, on either nappe. The
//! other three quarters, built from the same edges, have a skin that is the part
//! in front of the cap and of either plane, not of both, and that answers too
//! (row Q1009).

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, HalfSpace, NamedRun, Orientation, Picker, Plane, Point, Scale,
    Surface, SurfaceMeet, Vector, intersect,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, DraftStep, Error, Model, OnAFace, SolidBuilder, SolidKey};

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

/// The one of the two runs `intersect` names for `plane` against the cone that
/// holds `here` and not `not_here`.
///
/// Which run is which line is the pickers' to say, not the order's, and the run
/// is taken from `intersect` rather than written out here — a test that spelled
/// its own pickers would be checking itself.
fn the_ruling_on(plane: Plane, here: [f64; 3], not_here: [f64; 3]) -> EdgeRun {
    let SurfaceMeet::Curves(runs) = intersect(
        &Surface::Plane(plane),
        &Surface::Cone(the_cone()),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a plane down a cone's axis cuts it in two straight lines") else {
        panic!("two rulings are two curves");
    };
    assert_eq!(runs.len(), 2);
    runs.iter()
        .find(|run| holds(run, here) && !holds(run, not_here))
        .expect("one of the two runs holds the one point and not the other")
        .run()
        .clone()
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
/// The two side edges are the runs `intersect` names ([`the_ruling_on`]).
fn a_half_frustum(model: &mut Model) -> SolidKey {
    let skin = the_cone();
    let up_the_positive_side = the_ruling_on(the_cut_plane(), [0.0, 2.0, 4.0], [0.0, -2.0, 4.0]);
    let up_the_negative_side = the_ruling_on(the_cut_plane(), [0.0, -2.0, 4.0], [0.0, 2.0, 4.0]);

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
        .expect("this model made the key")
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

    for face in model.solid(solid).expect("this model made the key").faces() {
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
        .expect("this model made the key")
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

/// **Point-in-solid answers this solid, and every answer is the one the shape's
/// own sizes give** — worked out in plain arithmetic by
/// [`is_in_the_half_frustum`] before the kernel is asked, because a refusal that
/// turns into an answer is only a fix if the answer is right.
///
/// `(1, 0, 6)` against each of the four faces:
///
/// - the bottom cap is `z = 4`, facing down, and the point is `6 − 4 = 2` above
///   it;
/// - the top cap is `z = 8`, facing up, and the point is `8 − 6 = 2` below it;
/// - the flat face is `x = 0`, facing `−x`, and the point is at `x = 1`;
/// - the cone at height `6` has radius `0.5 · 6 = 3`, and the point is `1` from
///   the axis, so `x² + y² − (z/2)² = 1 − 9 = −8`, on the nappe `z > 0`.
///
/// Each is strictly the solid's side. The half frustum is convex — the inside of
/// one nappe is, and so is each half-space it is cut down by — so it is exactly
/// the part of space on the solid's side of all four, and a point strictly on
/// that side of every one is inside and on no face. The other points each break
/// one of the four, and come back outside.
///
/// This used to refuse, `CannotFollowAFacesTrimming` naming the cone face:
/// `holds.rs` read the two side edges, pieces of the rulings `x = 0` cuts the
/// cone in, as arcs it could not follow, and so it followed neither the skin
/// nor the flat face. A piece of a ruling is read as the straight segment it is
/// since `docs/decisions.md` §389, so both answer. The caps, each trimmed by
/// half a rim circle, had answered since §208.1. The whole frustum, whose
/// curved edges are closed, answers as it always did. An arc no family here
/// follows is still refused by name, but this solid no longer has one.
#[test]
fn point_in_solid_answers_a_half_frustum_as_its_own_sizes_say() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let half = a_half_frustum(&mut model);
    let whole = cones::a_frustum(&mut model, 0.5, 4.0, 8.0);

    for point in [
        [1.0, 0.0, 6.0],
        [2.0, -1.5, 7.5],
        [-1.0, 0.0, 6.0],
        [1.0, 0.0, 3.0],
        [1.0, 0.0, 9.0],
        [3.5, 0.0, 6.0],
        [1.0, 0.0, -6.0],
    ] {
        let want = if is_in_the_half_frustum(point) {
            Containment::Inside
        } else {
            Containment::Outside
        };
        assert_eq!(
            model
                .solid(half)
                .expect("this model made the key")
                .classify_point(Point::new(point[0], point[1], point[2]), &policy, &mut log),
            want,
            "half a frustum, asked about {point:?}"
        );
    }
    assert!(is_in_the_half_frustum([1.0, 0.0, 6.0]));

    assert_eq!(
        model
            .solid(whole)
            .expect("this model made the key")
            .classify_point(Point::new(1.0, 0.0, 6.0), &policy, &mut log),
        Containment::Inside,
        "the whole frustum, same cone and same builder"
    );
}

/// Whether `point` is strictly inside the half frustum, from its sizes alone:
/// above the bottom cap at `z = 4`, below the top cap at `z = 8`, on the `x > 0`
/// side of the flat face, and nearer the axis than the cone's radius `z/2` there.
///
/// # Panics
///
/// If `point` is on one of the four surfaces, where "strictly" says nothing.
fn is_in_the_half_frustum(point: [f64; 3]) -> bool {
    let [x, y, z] = point;
    let sides = [z - 4.0, 8.0 - z, x, (z / 2.0).powi(2) - (x * x + y * y)];
    assert!(
        sides.iter().all(|side| *side != 0.0),
        "{point:?} is on a face of the half frustum"
    );
    sides.iter().all(|side| *side > 0.0)
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

/// **A quarter of a pointed cone holds its points, and point-in-solid answers
/// it, on either nappe.**
///
/// The quarter is the `x ≥ 0`, `y ≥ 0` part of the cone of slope a half, eight
/// tall ([`a_pointed_cone_cut_along_its_axis`]). Its skin is cut by `x = 0` and
/// `y = 0`, **both through the apex**, and by the cap the apex is in front of.
/// Two planes through the apex can leave the rulings in two runs that meet only
/// at the apex (`two_planes_through_the_apex_leave_a_nappe_in_two_sheets` in
/// `holds.rs`). These two do not: they meet along the axis, inside the cone, so
/// the wedge they keep is one run of rulings, `0 < φ < 90°`, and `holds.rs`
/// follows it. `(1, 1, 6)` is inside, at `x = 1`, `y = 1`, below the cap at `8`,
/// and `√2` from the axis where the radius is `3`.
///
/// The same quarter turned a half turn about `x`, `(x, y, z) → (x, −y, −z)`, is
/// a quarter of the other nappe, and every point asked is turned with it. The
/// turn is a proper rotation and exact in `f64`, so the wires keep their
/// orientation and every stored number is the unturned one up to sign.
///
/// On the base `b1136e72` and until `docs/decisions.md` §389's row Q820 the
/// skin was refused by name, `AFaceTrimmedByACurveThisCannotFollow`, at every
/// point, and `(1, 1, 6)` came back `CannotFollowAFacesTrimming` naming it.
#[test]
fn a_quarter_of_a_pointed_cone_holds_its_points() {
    let policy = Policy::certified();
    let unturned: fn([f64; 3]) -> [f64; 3] = |point| point;
    let turned: fn([f64; 3]) -> [f64; 3] = |[x, y, z]| [x, -y, -z];
    for turn in [unturned, turned] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let quarter = a_pointed_cone_cut_along_its_axis(&mut model, Kept::TheQuarter, turn);
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .expect("a quarter of a pointed cone is a sound solid");
        let view = model.solid(quarter).expect("this model made the key");
        let face_on = |which: &dyn Fn(&Surface) -> bool| {
            let mut found = view.faces().filter(|face| which(&face.surface()));
            let face = found.next().expect("a face on that surface");
            assert!(found.next().is_none(), "one face on that surface");
            face.key()
        };
        let skin = face_on(&|surface| matches!(surface, Surface::Cone(_)));
        let flat_facing = |normal: [f64; 3]| {
            face_on(&|surface: &Surface| {
                surface
                    .as_plane()
                    .is_some_and(|plane| plane.normal().components() == turn(normal))
            })
        };
        let on_x_is_zero = flat_facing([-1.0, 0.0, 0.0]);
        let on_y_is_zero = flat_facing([0.0, -1.0, 0.0]);
        let asked = |face, point: [f64; 3]| {
            view.face_holds_the_point(
                face,
                PredicatePoint::from(turn(point)),
                &policy,
                &mut ProvenanceLog::new(),
            )
        };

        for (face, point, want) in [
            (skin, [1.8, 2.4, 6.0], OnAFace::Held),
            (skin, [-1.8, -2.4, 6.0], OnAFace::NotHeld),
            (skin, [1.8, 2.4, -6.0], OnAFace::NotHeld),
            (on_x_is_zero, [0.0, 1.0, 6.0], OnAFace::Held),
            (on_x_is_zero, [0.0, -1.0, 6.0], OnAFace::NotHeld),
            (on_x_is_zero, [0.0, 3.5, 6.0], OnAFace::NotHeld),
            (on_x_is_zero, [0.0, 1.0, -6.0], OnAFace::NotHeld),
            (on_x_is_zero, [0.0, 0.0, 4.0], OnAFace::OnItsBoundary),
            (on_y_is_zero, [1.0, 0.0, 6.0], OnAFace::Held),
            (on_y_is_zero, [-1.0, 0.0, 6.0], OnAFace::NotHeld),
            (on_y_is_zero, [3.0, 0.0, 6.0], OnAFace::OnItsBoundary),
        ] {
            assert_eq!(
                asked(face, point).unwrap_or_else(|error| panic!("{point:?}: {error:?}")),
                want,
                "{point:?}, turned to {:?}",
                turn(point)
            );
        }

        let classified = |point: [f64; 3]| {
            let [x, y, z] = turn(point);
            view.classify_point(Point::new(x, y, z), &policy, &mut ProvenanceLog::new())
        };
        assert_eq!(
            classified([1.0, 1.0, 6.0]),
            Containment::Inside,
            "turned to {:?}",
            turn([1.0, 1.0, 6.0])
        );
        for outside in [[3.0, 3.0, 6.0], [1.0, 1.0, 9.0], [1.0, 1.0, -6.0]] {
            assert_eq!(
                classified(outside),
                Containment::Outside,
                "{outside:?}, turned to {:?}",
                turn(outside)
            );
        }
    }
}

/// **The other three quarters of the same pointed cone hold their points, and
/// point-in-solid answers them as their own sizes say, on either nappe.** No
/// point of the missing quarter is ever held.
///
/// The solid is the part with `x ≤ 0` or `y ≤ 0`, built from the same edges as
/// the quarter with the rim the long way round
/// ([`a_pointed_cone_cut_along_its_axis`]); it meshes to three quarters of the
/// cone's `π · 4² · 8 / 3`, which is `32π`. Its skin is the part of the nappe
/// in front of the cap and in front of **either** `x = 0` or `y = 0`, not of
/// both: the part in front of both is the opposite quarter, and the rim runs
/// across both planes. `holds.rs` answers it as that
/// (`the_cap_and_the_two_cuts_a_cone_face_is_in_front_of_either_of`, which
/// says why that part is the skin). So a point on the cone is held where `x < 0`
/// or `y < 0` below the cap, turned away where `x > 0` and `y > 0`, and on the
/// skin's boundary along the ruling up `x = 0` at `y > 0`, the one up `y = 0` at
/// `x > 0`, the rim, and the apex. The ruling up `x = 0` at `y < 0` is inside
/// the skin, which runs on across it.
///
/// Point-in-solid is checked against [`is_in_the_three_quarters`] at the middle
/// of every one of `16³` cells of the box round the cone, half a unit on a
/// side, none of which is on a face; and the cells it calls inside add up to
/// `32π` to within the grid's own coarseness, 813 of them, `101.6` against
/// `100.5`.
///
/// The cap winds its own sides since row Q1002, with the rim read as its chord
/// from `(4, 0)` to `(0, 4)`: so it holds three quarters of the disc and turns
/// away the quarter `x, y > 0`, the chord across it included, and what lies
/// outside the disc.
///
/// Until row Q1009 the skin was refused by name,
/// `AFaceTrimmedByACurveThisCannotFollow`, at every point on its own nappe,
/// because the rim lies across `y = 0`; and `(−1, −1, 6)`, `(−1, 1, 6)` and
/// `(1, −1, 6)`, each inside, came back `CannotFollowAFacesTrimming` naming it.
#[test]
fn three_quarters_of_a_pointed_cone_hold_their_points() {
    let policy = Policy::certified();
    let unturned: fn([f64; 3]) -> [f64; 3] = |point| point;
    let turned: fn([f64; 3]) -> [f64; 3] = |[x, y, z]| [x, -y, -z];
    for turn in [unturned, turned] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let rest = a_pointed_cone_cut_along_its_axis(&mut model, Kept::TheRest, turn);
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .expect("three quarters of a pointed cone is a sound solid");
        let view = model.solid(rest).expect("this model made the key");

        let measured = view
            .measure(
                Deflection::of(0.002).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("three quarters of a pointed cone measures");
        assert!(
            (measured.volume() - THREE_QUARTERS_OF_THE_CONE).abs() < 0.05,
            "three quarters of the cone's volume, {} against {THREE_QUARTERS_OF_THE_CONE}",
            measured.volume()
        );

        let face_on = |which: &dyn Fn(&Surface) -> bool| {
            let mut found = view.faces().filter(|face| which(&face.surface()));
            let face = found.next().expect("a face on that surface");
            assert!(found.next().is_none(), "one face on that surface");
            face.key()
        };
        let skin = face_on(&|surface| matches!(surface, Surface::Cone(_)));
        let flat_facing = |normal: [f64; 3]| {
            face_on(&|surface: &Surface| {
                surface
                    .as_plane()
                    .is_some_and(|plane| plane.normal().components() == turn(normal))
            })
        };
        let cap = flat_facing([0.0, 0.0, 1.0]);
        let on_x_is_zero = flat_facing([1.0, 0.0, 0.0]);
        let on_y_is_zero = flat_facing([0.0, 1.0, 0.0]);
        let asked = |face, point: [f64; 3]| {
            view.face_holds_the_point(
                face,
                PredicatePoint::from(turn(point)),
                &policy,
                &mut ProvenanceLog::new(),
            )
        };

        for (face, point, want) in [
            (skin, [-1.8, -2.4, 6.0], OnAFace::Held),
            (skin, [-1.8, 2.4, 6.0], OnAFace::Held),
            (skin, [1.8, -2.4, 6.0], OnAFace::Held),
            (skin, [1.8, 2.4, 6.0], OnAFace::NotHeld),
            (skin, [2.4, 1.8, 6.0], OnAFace::NotHeld),
            (skin, [0.6, 0.8, 2.0], OnAFace::NotHeld),
            (skin, [0.0, 3.0, 6.0], OnAFace::OnItsBoundary),
            (skin, [3.0, 0.0, 6.0], OnAFace::OnItsBoundary),
            (skin, [0.0, -3.0, 6.0], OnAFace::Held),
            (skin, [-3.0, 0.0, 6.0], OnAFace::Held),
            (skin, [0.0, -4.0, 8.0], OnAFace::OnItsBoundary),
            (skin, [-2.4, 3.2, 8.0], OnAFace::OnItsBoundary),
            (skin, [2.4, 3.2, 8.0], OnAFace::NotHeld),
            (skin, [-5.0, 0.0, 10.0], OnAFace::NotHeld),
            (skin, [0.0, 0.0, 0.0], OnAFace::OnItsBoundary),
            (skin, [1.8, 2.4, -6.0], OnAFace::NotHeld),
            (skin, [-1.8, -2.4, -6.0], OnAFace::NotHeld),
            (cap, [-1.0, -1.0, 8.0], OnAFace::Held),
            (cap, [-1.0, 1.0, 8.0], OnAFace::Held),
            (cap, [1.0, -3.0, 8.0], OnAFace::Held),
            (cap, [1.0, 1.0, 8.0], OnAFace::NotHeld),
            (cap, [2.0, 2.0, 8.0], OnAFace::NotHeld),
            (cap, [1.0, 3.0, 8.0], OnAFace::NotHeld),
            (cap, [3.0, 3.0, 8.0], OnAFace::NotHeld),
            (cap, [-3.0, -3.0, 8.0], OnAFace::NotHeld),
            (cap, [0.0, -4.0, 8.0], OnAFace::OnItsBoundary),
            (cap, [0.0, 2.0, 8.0], OnAFace::OnItsBoundary),
            (cap, [0.0, 4.0, 8.0], OnAFace::OnItsBoundary),
            (on_x_is_zero, [0.0, 1.0, 6.0], OnAFace::Held),
            (on_x_is_zero, [0.0, -1.0, 6.0], OnAFace::NotHeld),
            (on_y_is_zero, [1.0, 0.0, 6.0], OnAFace::Held),
            (on_y_is_zero, [-1.0, 0.0, 6.0], OnAFace::NotHeld),
        ] {
            assert_eq!(
                asked(face, point).unwrap_or_else(|error| panic!("{point:?}: {error:?}")),
                want,
                "{point:?}, turned to {:?}",
                turn(point)
            );
        }

        let classified = |point: [f64; 3]| {
            let [x, y, z] = turn(point);
            view.classify_point(Point::new(x, y, z), &policy, &mut ProvenanceLog::new())
        };
        for (point, want) in [
            ([-1.0, -1.0, 6.0], Containment::Inside),
            ([-1.0, 1.0, 6.0], Containment::Inside),
            ([1.0, -1.0, 6.0], Containment::Inside),
            ([1.0, 1.0, 6.0], Containment::Outside),
            ([2.0, 2.0, 6.0], Containment::Outside),
            ([1.0, 1.0, 9.0], Containment::Outside),
            ([-1.0, -1.0, -6.0], Containment::Outside),
        ] {
            assert_eq!(
                classified(point),
                want,
                "{point:?}, turned to {:?}",
                turn(point)
            );
        }

        let middles: Vec<f64> = (0..16).map(|step| f64::from(step) * 0.5 + 0.25).collect();
        let mut inside = 0u32;
        for &x in &middles {
            for &y in &middles {
                for &z in &middles {
                    let point = [x - 4.0, y - 4.0, z];
                    let want = if is_in_the_three_quarters(point) {
                        inside += 1;
                        Containment::Inside
                    } else {
                        Containment::Outside
                    };
                    assert_eq!(
                        classified(point),
                        want,
                        "{point:?}, turned to {:?}",
                        turn(point)
                    );
                }
            }
        }
        let held = f64::from(inside) * 0.5f64.powi(3);
        assert!(
            (held - THREE_QUARTERS_OF_THE_CONE).abs() < 0.02 * THREE_QUARTERS_OF_THE_CONE,
            "{inside} cells inside hold {held}, against {THREE_QUARTERS_OF_THE_CONE}"
        );
    }
}

/// Three quarters of `π · 4² · 8 / 3`, the volume of the cone of slope a half
/// from its apex to its cap at `z = 8`: `32π`.
const THREE_QUARTERS_OF_THE_CONE: f64 = 0.75 * core::f64::consts::PI * 4.0 * 4.0 * 8.0 / 3.0;

/// Whether `point` is strictly inside the three quarters of the pointed cone,
/// from their sizes alone: above the apex, below the cap at `z = 8`, nearer
/// the axis than the cone's radius `z/2` there, and on the `x < 0` side of
/// `x = 0` or the `y < 0` side of `y = 0`.
///
/// # Panics
///
/// If `point` is on one of the four surfaces, where "strictly" says nothing.
fn is_in_the_three_quarters(point: [f64; 3]) -> bool {
    let [x, y, z] = point;
    let sides = [z, 8.0 - z, (z / 2.0).powi(2) - (x * x + y * y), -x.min(y)];
    assert!(
        sides.iter().all(|side| *side != 0.0),
        "{point:?} is on a face of the three quarters"
    );
    sides.iter().all(|side| *side > 0.0)
}

/// Which part of the pointed cone [`a_pointed_cone_cut_along_its_axis`] builds.
#[derive(Clone, Copy)]
enum Kept {
    /// The part with `x ≥ 0` and `y ≥ 0`.
    TheQuarter,
    /// The other three quarters: the part with `x ≤ 0` or `y ≤ 0`.
    TheRest,
}

/// The cone of slope a half about `+z`, from its apex up to a cap at `z = 8`,
/// cut by `x = 0` and `y = 0` down to the part `kept` says, and turned by
/// `turn`, which must be a proper rotation. Four faces — the cap, a triangle on
/// each of `x = 0` and `y = 0`, and the skin — six edges and four corners, the
/// apex one of them.
///
/// The skin's two sides up from the apex are rulings, one named by each plane
/// ([`the_ruling_on`]); the two triangles meet along the axis. The other three
/// quarters are bounded by the same lines: the rim runs the long way round, and
/// every face faces the other way, so every wire runs the other way too.
fn a_pointed_cone_cut_along_its_axis(
    model: &mut Model,
    kept: Kept,
    turn: fn([f64; 3]) -> [f64; 3],
) -> SolidKey {
    let (outward, way) = match kept {
        Kept::TheQuarter => (1.0, Orientation::Forward),
        Kept::TheRest => (-1.0, Orientation::Reversed),
    };
    let point = |at: [f64; 3]| {
        let [x, y, z] = turn(at);
        Point::new(x, y, z)
    };
    let vector = |along: [f64; 3]| {
        let [x, y, z] = turn(along);
        Vector::new(x, y, z)
    };
    let x_is_zero = Plane::from_point_and_normal(Point::origin(), vector([-outward, 0.0, 0.0]))
        .expect("the x axis is a unit normal");
    let y_is_zero = Plane::from_point_and_normal(Point::origin(), vector([0.0, -outward, 0.0]))
        .expect("the y axis is a unit normal");
    let cap_plane = Plane::from_point_and_normal(point([0.0, 0.0, 8.0]), vector([0.0, 0.0, 1.0]))
        .expect("the z axis is a unit normal");

    let mut builder = SolidBuilder::new();
    let apex = builder.add_vertex(Point::origin()).expect("a finite point");
    let on_x = builder
        .add_vertex(point([4.0, 0.0, 8.0]))
        .expect("a finite point");
    let on_y = builder
        .add_vertex(point([0.0, 4.0, 8.0]))
        .expect("a finite point");
    let on_the_axis = builder
        .add_vertex(point([0.0, 0.0, 8.0]))
        .expect("a finite point");

    let up_y_is_zero = builder
        .add_edge(
            apex,
            on_x,
            the_ruling_on(y_is_zero, turn([2.0, 0.0, 4.0]), turn([-2.0, 0.0, 4.0])),
        )
        .expect("a piece of the ruling over +x");
    let up_x_is_zero = builder
        .add_edge(
            apex,
            on_y,
            the_ruling_on(x_is_zero, turn([0.0, 2.0, 4.0]), turn([0.0, -2.0, 4.0])),
        )
        .expect("a piece of the ruling over +y");
    let up_the_axis = builder
        .add_edge(apex, on_the_axis, EdgeRun::Straight)
        .expect("the axis");
    let round_the_rim = builder
        .add_edge(
            on_x,
            on_y,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(cap_plane),
                picked_by: smallvec::smallvec![Picker::from(
                    HalfSpace::through_a_point_facing(
                        point([4.0, 0.0, 8.0]),
                        vector([outward, outward, 0.0])
                    )
                    .expect("a direction with a length")
                )],
            },
        )
        .expect("the rim between the two rulings");
    let cap_along_x = builder
        .add_edge(on_the_axis, on_x, EdgeRun::Straight)
        .expect("the cap's side along y = 0");
    let cap_along_y = builder
        .add_edge(on_y, on_the_axis, EdgeRun::Straight)
        .expect("the cap's side along x = 0");

    let wire_of = |builder: &mut SolidBuilder, steps: &[(_, Orientation)]| {
        let mut along: Vec<DraftStep> = steps
            .iter()
            .map(|(edge, step)| {
                DraftStep::Along(
                    builder
                        .add_coedge(*edge, step.composed_with(way))
                        .expect("an edge of this build"),
                )
            })
            .collect();
        if way.is_reversed() {
            along.reverse();
        }
        builder.add_wire(&along).expect("a closed wire")
    };
    let cap_wire = wire_of(
        &mut builder,
        &[
            (cap_along_x, Orientation::Forward),
            (round_the_rim, Orientation::Forward),
            (cap_along_y, Orientation::Forward),
        ],
    );
    let on_x_is_zero_wire = wire_of(
        &mut builder,
        &[
            (up_the_axis, Orientation::Forward),
            (cap_along_y, Orientation::Reversed),
            (up_x_is_zero, Orientation::Reversed),
        ],
    );
    let on_y_is_zero_wire = wire_of(
        &mut builder,
        &[
            (up_y_is_zero, Orientation::Forward),
            (cap_along_x, Orientation::Reversed),
            (up_the_axis, Orientation::Reversed),
        ],
    );
    let skin_wire = wire_of(
        &mut builder,
        &[
            (up_x_is_zero, Orientation::Forward),
            (round_the_rim, Orientation::Reversed),
            (up_y_is_zero, Orientation::Reversed),
        ],
    );
    let cap = builder
        .add_face(Surface::Plane(cap_plane), &[cap_wire])
        .expect("the cap");
    let on_x_is_zero = builder
        .add_face(Surface::Plane(x_is_zero), &[on_x_is_zero_wire])
        .expect("a triangle on x = 0");
    let on_y_is_zero = builder
        .add_face(Surface::Plane(y_is_zero), &[on_y_is_zero_wire])
        .expect("a triangle on y = 0");
    let skin = builder
        .add_face(Surface::Cone(the_cone()), &[skin_wire])
        .expect("the skin");
    builder
        .add_shell(&[
            (cap, Orientation::Forward),
            (on_x_is_zero, Orientation::Forward),
            (on_y_is_zero, Orientation::Forward),
            (skin, Orientation::Forward),
        ])
        .expect("four faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a pointed cone cut along its axis, described this way, is a solid")
}
