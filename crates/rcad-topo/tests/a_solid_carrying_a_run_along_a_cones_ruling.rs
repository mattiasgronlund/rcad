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
//! cost (`docs/decisions.md` §389).

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

/// **A quarter of a pointed cone has its skin refused by name, and that is a
/// gap: the skin is one piece.** Measured, and the same refusal on the base
/// before `docs/decisions.md` §389.
///
/// The quarter is the `x ≥ 0`, `y ≥ 0` part of the cone of slope a half, eight
/// tall ([`a_quarter_of_a_pointed_cone`]). Its skin is cut by `x = 0` and
/// `y = 0`, **both through the apex**, and by the cap the apex is in front of.
/// `holds.rs` follows a cone face beside at most one cut through its apex,
/// because two can leave the rulings in two runs that meet only at the apex
/// (`two_planes_through_the_apex_leave_a_nappe_in_two_sheets` there). These two
/// do not: they meet along the axis, inside the cone, so each half-plane round
/// the axis meets the nappe in one ruling and the wedge they keep is one run of
/// rulings, `0 < φ < 90°`. The answer exists — `(1, 1, 6)` is inside, at `x = 1`,
/// `y = 1`, below the cap at `8`, and `√2` from the axis where the radius is `3`
/// — and the classifier names the skin instead.
///
/// On the base `b1136e72`, where a piece of a ruling was not read as straight,
/// the skin was refused by the same name, and so were both flat triangles;
/// `(1, 1, 6)` came back naming the skin, as here. So this change brought in no
/// new refusal for the quarter, and it took two away: the triangles answer now,
/// and are pinned here with them. Telling the quarter's two cuts from the two
/// sheets — where the two planes meet, against the cone — is the fix, and it is
/// filed rather than done (`docs/decisions.md` §389).
///
/// What is outside the cone or above the cap is still answered: the ray from it
/// meets no face that refuses.
#[test]
fn a_quarter_of_a_pointed_cone_has_its_skin_refused_by_name_though_it_is_one_piece() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let quarter = a_quarter_of_a_pointed_cone(&mut model);
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
                .is_some_and(|plane| plane.normal().components() == normal)
        })
    };
    let on_x_is_zero = flat_facing([-1.0, 0.0, 0.0]);
    let on_y_is_zero = flat_facing([0.0, -1.0, 0.0]);
    let asked = |face, point: [f64; 3]| {
        view.face_holds_the_point(
            face,
            PredicatePoint::from(point),
            &policy,
            &mut ProvenanceLog::new(),
        )
    };

    for point in [[1.8, 2.4, 6.0], [-1.8, -2.4, 6.0], [1.8, 2.4, -6.0]] {
        assert!(
            matches!(
                asked(skin, point),
                Err(Error::AFaceTrimmedByACurveThisCannotFollow { face }) if face == skin
            ),
            "the skin, asked about {point:?}"
        );
    }
    for (face, point, want) in [
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
            "a flat triangle, asked about {point:?}"
        );
    }

    let classified = |point: [f64; 3]| {
        view.classify_point(
            Point::new(point[0], point[1], point[2]),
            &policy,
            &mut ProvenanceLog::new(),
        )
    };
    assert_eq!(
        classified([1.0, 1.0, 6.0]),
        Containment::CannotFollowAFacesTrimming { face: skin }
    );
    for outside in [[3.0, 3.0, 6.0], [1.0, 1.0, 9.0], [1.0, 1.0, -6.0]] {
        assert_eq!(classified(outside), Containment::Outside, "{outside:?}");
    }
}

/// A quarter of a pointed cone: the cone of slope a half about `+z`, from its
/// apex up to a cap at `z = 8`, cut down to `x ≥ 0` and `y ≥ 0`. Four faces —
/// the quarter disc of the cap, a triangle on each of `x = 0` and `y = 0`, and a
/// quarter of the skin — six edges and four corners, the apex one of them.
///
/// The skin's two sides up from the apex are rulings, one named by each plane
/// ([`the_ruling_on`]); the two triangles meet along the axis.
fn a_quarter_of_a_pointed_cone(model: &mut Model) -> SolidKey {
    let x_is_zero = the_cut_plane();
    let y_is_zero = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, -1.0, 0.0))
        .expect("the y axis is a unit normal");
    let cap_plane =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 8.0), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");

    let mut builder = SolidBuilder::new();
    let apex = builder.add_vertex(Point::origin()).expect("a finite point");
    let on_x = builder
        .add_vertex(Point::new(4.0, 0.0, 8.0))
        .expect("a finite point");
    let on_y = builder
        .add_vertex(Point::new(0.0, 4.0, 8.0))
        .expect("a finite point");
    let on_the_axis = builder
        .add_vertex(Point::new(0.0, 0.0, 8.0))
        .expect("a finite point");

    let up_y_is_zero = builder
        .add_edge(
            apex,
            on_x,
            the_ruling_on(y_is_zero, [2.0, 0.0, 4.0], [-2.0, 0.0, 4.0]),
        )
        .expect("a piece of the ruling over +x");
    let up_x_is_zero = builder
        .add_edge(
            apex,
            on_y,
            the_ruling_on(x_is_zero, [0.0, 2.0, 4.0], [0.0, -2.0, 4.0]),
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
                        Point::new(4.0, 0.0, 8.0),
                        Vector::new(1.0, 1.0, 0.0)
                    )
                    .expect("a direction with a length")
                )],
            },
        )
        .expect("a quarter of the rim");
    let cap_along_x = builder
        .add_edge(on_the_axis, on_x, EdgeRun::Straight)
        .expect("the cap's side along y = 0");
    let cap_along_y = builder
        .add_edge(on_y, on_the_axis, EdgeRun::Straight)
        .expect("the cap's side along x = 0");

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
        .expect("a quarter disc");
    let on_x_is_zero = builder
        .add_face(Surface::Plane(x_is_zero), &[on_x_is_zero_wire])
        .expect("a triangle on x = 0");
    let on_y_is_zero = builder
        .add_face(Surface::Plane(y_is_zero), &[on_y_is_zero_wire])
        .expect("a triangle on y = 0");
    let skin = builder
        .add_face(Surface::Cone(the_cone()), &[skin_wire])
        .expect("a quarter of the skin");
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
        .expect("a quarter of a pointed cone described this way is a solid")
}
