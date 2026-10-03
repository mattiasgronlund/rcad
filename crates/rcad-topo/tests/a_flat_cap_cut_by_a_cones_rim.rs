//! A cone's cap, cut down to a piece bounded by an arc of the cone's own rim,
//! asked which points it holds (`docs/decisions.md` §206.2).
//!
//! `holds.rs`'s `is_a_family_this_follows` took a flat face whose curved sides
//! are arcs of a **ball** it lies inside, and refused `Surface::Cone` by name.
//! So every cap the fuse cut in two — and that is what a block does to a
//! frustum's big cap — came back `CannotFollowAFacesTrimming`, which is what
//! held the floor in `a_plane_cut_along_a_cones_other_nappe.rs` at 8 879 of its
//! grid points however much else was fixed.
//!
//! The family the fix adds is **a plane standing square to the cone's axis**,
//! which is what every cap in this kernel is. Such a plane meets the cone in a
//! circle round the axis, on one nappe, and what the cone's inside leaves on the
//! plane is the round disc that circle bounds — convex, so the part of the plane
//! in front of every cut is still one piece and the caller's argument goes
//! through unchanged. A **leaned** plane is refused and stays refused: far
//! enough over it cuts both nappes in a hyperbola, whose inside is two regions
//! with nothing between them.
//!
//! # The solid
//!
//! Half a frustum — the `x ≥ 0` half of a cone of slope a half between `z = 4`
//! and `z = 8` — described face by face through the public builder, the same
//! shape `a_solid_carrying_a_run_along_a_cones_ruling.rs` builds and for the
//! same reason: every number in it is exact, so nothing here is asking the
//! arithmetic for a favour. Each cap is then a half-disc whose curved side is
//! half of the rim circle and whose straight side is the chord across it.
//!
//! **The flat face down the axis and the skin are not answered by this
//! family.** Both are bounded by pieces of the two rulings a plane through the
//! apex cuts, and both were refused until `docs/decisions.md` §334 read a piece
//! of a ruling as the straight segment it is:
//! [`the_face_down_the_axis_holds_its_four_corners_and_not_the_other_nappe`] and
//! [`the_skin_of_a_half_frustum_holds_its_own_half_between_the_caps`]. Neither
//! admits a leaned plane's hyperbola, which no arm here reads as straight.

use rcad_geom::{
    Axis, Cone, EdgeRun, HalfSpace, Isometry, NamedRun, Orientation, Picker, Plane, Point, Scale,
    Surface, SurfaceMeet, Vector, intersect,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{DraftStep, FaceKey, Model, OnAFace, SolidBuilder, SolidKey};

mod cones;

/// Slope a half, so `slope · height` rounds to nothing at either cap.
const SLOPE: f64 = 0.5;

/// The two heights the half-frustum's caps sit at.
const LOW: f64 = 4.0;
const HIGH: f64 = 8.0;

/// Every point of a cap, away from its edges, is held or not held — never
/// refused — and which it is agrees with the half-disc the cap is.
#[test]
fn each_cap_of_a_half_frustum_says_which_points_it_holds() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_half_frustum(&mut model);
    let mut asked = 0;
    for (at, outward) in [(LOW, -1.0), (HIGH, 1.0)] {
        let cap = the_cap_at(&model, solid, at, outward);
        let view = model.solid(solid).expect("this model made the key");
        for point in clear_grid_points(at) {
            let said = view
                .face_holds_the_point(
                    cap,
                    PredicatePoint::from(point.coordinates()),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|error| panic!("the cap at {at} refused {point:?}: {error:?}"));
            let want = if on_the_half_disc(at, point) {
                OnAFace::Held
            } else {
                OnAFace::NotHeld
            };
            assert_eq!(said, want, "the cap at {at}, asked about {point:?}");
            asked += 1;
        }
    }
    assert!(asked > 700, "only {asked} points were asked");
}

/// The three kinds of boundary a half-disc cap has — the arc, the chord, and
/// the two corners where they meet — read as boundary and not as one side or
/// the other.
#[test]
fn a_cap_of_a_half_frustum_knows_its_own_arc_and_chord() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_half_frustum(&mut model);
    let cap = the_cap_at(&model, solid, HIGH, 1.0);
    let view = model.solid(solid).expect("this model made the key");
    let asked = |point: Point| {
        view.face_holds_the_point(
            cap,
            PredicatePoint::from(point.coordinates()),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("the big cap says")
    };

    let reach = SLOPE * HIGH;
    assert_eq!(asked(Point::new(reach, 0.0, HIGH)), OnAFace::OnItsBoundary);
    assert_eq!(asked(Point::new(0.0, 2.0, HIGH)), OnAFace::OnItsBoundary);
    assert_eq!(asked(Point::new(0.0, reach, HIGH)), OnAFace::OnItsBoundary);
    assert_eq!(asked(Point::new(0.0, -reach, HIGH)), OnAFace::OnItsBoundary);
    assert_eq!(asked(Point::new(2.0, 1.0, HIGH)), OnAFace::Held);
    assert_eq!(asked(Point::new(reach + 1.0, 0.0, HIGH)), OnAFace::NotHeld);
    assert_eq!(asked(Point::new(-1.0, 0.0, HIGH)), OnAFace::NotHeld);
}

/// Moved and turned, a cap is the same cap and answers the same points moved
/// with it.
///
/// This is §168's rule paying off for a cone: the square-to-the-axis test, the
/// circle's centre and which side of a cut each side of the face lies on are all
/// asked of the face **where it is stored**, where a cap's plane normal and its
/// cone's axis direction are the same triple bit for bit. A movement rounds both
/// and would put them a hair apart, and then every point asked of a placed cap
/// would refuse.
#[test]
fn a_placed_cap_answers_the_same_points_moved_with_it() {
    let shifted = Isometry::translation(Vector::new(0.1, -1.0 / 3.0, 2.7)).expect("a finite shift");
    let turned = Isometry::rotation(
        &Axis::new(Point::new(0.3, -0.2, 0.5), Vector::new(1.0, 2.0, 3.0))
            .expect("an axis with a length"),
        0.7,
    )
    .expect("a finite angle");
    let policy = Policy::certified();
    for movement in [shifted, turned, shifted.compose(&turned)] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let solid = a_half_frustum(&mut model);
        let cap = the_cap_at(&model, solid, HIGH, 1.0);
        model
            .place_solid(solid, &movement)
            .expect("this model made the key");
        let view = model.solid(solid).expect("this model made the key");
        for point in clear_grid_points(HIGH) {
            let moved = movement.apply_to_point(point);
            let said = view
                .face_holds_the_point(
                    cap,
                    PredicatePoint::from(moved.coordinates()),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|error| {
                    panic!("the placed cap refused {moved:?}: {error:?}, placed by {movement:?}")
                });
            let want = if on_the_half_disc(HIGH, point) {
                OnAFace::Held
            } else {
                OnAFace::NotHeld
            };
            assert_eq!(said, want, "placed by {movement:?}, asked about {point:?}");
        }
    }
}

/// The flat face down the axis holds the four-cornered piece of `x = 0` it is,
/// and nothing on the cone's other nappe.
///
/// Its two side edges are pieces of the two rulings a plane **through the
/// apex** cuts the cone in. This was refused as a limit: such a plane leans
/// against the cone, and the cone's inside on it is two pieces, a wedge on each
/// nappe meeting at the apex. That is true of the cone's inside, and the face is
/// not read off the cone's inside any more. A piece of a ruling is the straight
/// segment between its two corners, so the face is the polygon of its corners
/// and is walked as any flat face with straight sides is (`docs/decisions.md`
/// §334). `(0, 1, −6)` is inside the cone, on the other nappe's wedge, and
/// outside the polygon; a reading that took the cone's inside for the face
/// would hold it.
///
/// What admits a side as straight is the plane's own sign at the apex, exactly
/// zero, and its two ends on one line with the apex. A plane leaned off the
/// apex cuts a curve, and none of its sides is read as straight.
#[test]
fn the_face_down_the_axis_holds_its_four_corners_and_not_the_other_nappe() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_half_frustum(&mut model);
    let flat = the_face_on(&model, solid, |surface| {
        surface
            .as_plane()
            .is_some_and(|plane| plane.normal().components() == [-1.0, 0.0, 0.0])
    });
    for (point, want) in [
        ([0.0, 1.0, 6.0], OnAFace::Held),
        ([0.0, -2.5, 7.5], OnAFace::Held),
        ([0.0, 3.0, 6.0], OnAFace::OnItsBoundary),
        ([0.0, 0.0, LOW], OnAFace::OnItsBoundary),
        ([0.0, 3.5, 6.0], OnAFace::NotHeld),
        ([0.0, 0.0, 3.0], OnAFace::NotHeld),
        ([0.0, 0.0, 9.0], OnAFace::NotHeld),
        ([0.0, 1.0, -6.0], OnAFace::NotHeld),
    ] {
        assert_eq!(
            asked(&model, solid, flat, point),
            want,
            "asked about {point:?}"
        );
    }
}

/// The half frustum's skin holds the half of the cone between its caps on the
/// `x ≥ 0` side, and nothing else.
///
/// Its sides along `x = 0` are the same two rulings, read as straight, and it
/// is cut by three planes: the small cap the apex is behind, the big cap the
/// apex is in front of, and `x = 0` through the apex. `holds.rs` took one cut
/// beside the one the apex is behind and refused two; the big cap is side by
/// side with the small one, and a cut side by side with that one keeps every
/// ruling or none, so it costs nothing (`docs/decisions.md` §334). This skin
/// was the first face a ray asked when the half frustum was fused with a block
/// across its top, and every fuse stopped there.
#[test]
fn the_skin_of_a_half_frustum_holds_its_own_half_between_the_caps() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_half_frustum(&mut model);
    let skin = the_face_on(&model, solid, |surface| matches!(surface, Surface::Cone(_)));
    for (point, want) in [
        ([3.0, 0.0, 6.0], OnAFace::Held),
        ([2.5, 0.0, 5.0], OnAFace::Held),
        ([0.0, 3.0, 6.0], OnAFace::OnItsBoundary),
        ([0.0, -3.0, 6.0], OnAFace::OnItsBoundary),
        ([4.0, 0.0, HIGH], OnAFace::OnItsBoundary),
        ([-3.0, 0.0, 6.0], OnAFace::NotHeld),
        ([1.5, 0.0, 3.0], OnAFace::NotHeld),
        ([4.5, 0.0, 9.0], OnAFace::NotHeld),
        ([3.0, 0.0, -6.0], OnAFace::NotHeld),
    ] {
        assert_eq!(
            asked(&model, solid, skin, point),
            want,
            "asked about {point:?}"
        );
    }
}

/// The cap of the whole frustum the half is cut out of, which is trimmed by one
/// closed circle and not by an arc, answers as it always did.
///
/// A whole cap goes down a different route entirely — a wire that is one closed
/// edge is counted rather than walked — so this is the control that says the two
/// tests above are about the *arc* and not about the cone.
#[test]
fn a_whole_cap_answers_as_it_always_did() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = cones::a_frustum(&mut model, SLOPE, LOW, HIGH);
    let cap = the_cap_at(&model, solid, HIGH, 1.0);
    let view = model.solid(solid).expect("this model made the key");
    let asked = |point: Point| {
        view.face_holds_the_point(
            cap,
            PredicatePoint::from(point.coordinates()),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a whole cap says")
    };
    assert_eq!(asked(Point::new(-2.0, 1.0, HIGH)), OnAFace::Held);
    assert_eq!(asked(Point::new(2.0, 1.0, HIGH)), OnAFace::Held);
    assert_eq!(asked(Point::new(5.0, 0.0, HIGH)), OnAFace::NotHeld);
}

/// The face of `solid` on the plane `z = at` facing `outward`.
fn the_cap_at(model: &Model, solid: SolidKey, at: f64, outward: f64) -> FaceKey {
    model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .find(|face| {
            face.surface().as_plane().is_some_and(|plane| {
                plane.normal().components() == [0.0, 0.0, outward]
                    && plane.side_of_point(PredicatePoint::from([0.0, 0.0, at])) == Decision::Zero
            })
        })
        .expect("a cap facing that way")
        .key()
}

/// The one face of `solid` whose surface `which` picks.
fn the_face_on(model: &Model, solid: SolidKey, which: impl Fn(&Surface) -> bool) -> FaceKey {
    let mut found = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .filter(|face| which(&face.surface()));
    let face = found.next().expect("a face on that surface");
    assert!(found.next().is_none(), "one face on that surface");
    face.key()
}

/// What `face` of `solid` says about `point`, which must not be a refusal.
fn asked(model: &Model, solid: SolidKey, face: FaceKey, point: [f64; 3]) -> OnAFace {
    model
        .solid(solid)
        .expect("this model made the key")
        .face_holds_the_point(
            face,
            PredicatePoint::from(point),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|error| panic!("asked about {point:?}: {error:?}"))
}

/// Whether the cap at height `at` holds `point`, which is what the half-disc
/// itself says: inside the rim circle and on the `x > 0` side of the cut.
fn on_the_half_disc(at: f64, point: Point) -> bool {
    let reach = SLOPE * at;
    point.x() > 0.0 && point.x().hypot(point.y()) < reach
}

/// Points a fifth of a unit apart across the cap at height `at`, reaching a
/// unit past its rim, and never nearer than `0.05` to the rim circle or to the
/// chord down `x = 0`.
fn clear_grid_points(at: f64) -> impl Iterator<Item = Point> {
    let reach = SLOPE * at;
    let steps = ((reach + 1.0) / 0.2).ceil() as i32;
    (-steps..=steps).flat_map(move |i| {
        (-steps..=steps).filter_map(move |j| {
            let (x, y) = (f64::from(i) * 0.2, f64::from(j) * 0.2);
            let clear = x.abs() >= 0.05 && (x.hypot(y) - reach).abs() >= 0.05;
            clear.then(|| Point::new(x, y, at))
        })
    })
}

/// The cone the half-frustum is cut out of: slope a half about `+z`, apex at
/// the origin.
fn the_cone() -> Cone {
    Cone::new(Axis::z(), SLOPE).expect("a positive slope")
}

/// The flat face's own plane, facing out of the solid.
fn the_cut_plane() -> Plane {
    Plane::from_point_and_normal(Point::origin(), Vector::new(-1.0, 0.0, 0.0))
        .expect("the x axis is a unit normal")
}

/// Half a frustum, described face by face through the public builder, with the
/// two side edges taken from [`intersect`] rather than spelled out here — a
/// test that wrote its own pickers would be checking itself.
fn a_half_frustum(model: &mut Model) -> SolidKey {
    let runs = the_two_rulings();
    let up_the_positive_side = the_ruling_through(&runs, 2.0);
    let up_the_negative_side = the_ruling_through(&runs, -2.0);

    let mut builder = SolidBuilder::new();
    let corner = |builder: &mut SolidBuilder, y: f64, z: f64| {
        builder
            .add_vertex(Point::new(0.0, y, z))
            .expect("a finite point")
    };
    let bottom_negative = corner(&mut builder, -SLOPE * LOW, LOW);
    let bottom_positive = corner(&mut builder, SLOPE * LOW, LOW);
    let top_positive = corner(&mut builder, SLOPE * HIGH, HIGH);
    let top_negative = corner(&mut builder, -SLOPE * HIGH, HIGH);

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
    let bottom = cap(&mut builder, LOW, -1.0, bottom_wire);
    let top = cap(&mut builder, HIGH, 1.0, top_wire);
    let side = builder
        .add_face(Surface::Cone(the_cone()), &[skin_wire])
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

/// The two runs [`intersect`] names for the cut plane against the cone.
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

/// Whichever of the two rulings holds `(0, y, 4)` and not its mirror, which is
/// the pickers' to say and not the order's.
fn the_ruling_through(runs: &[NamedRun], y: f64) -> EdgeRun {
    let holds = |run: &NamedRun, at: [f64; 3]| {
        let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
            panic!("a ruling through the apex is a piece of where the two surfaces cross");
        };
        picked_by
            .iter()
            .all(|picker| picker.side_of_point(PredicatePoint::from(at)) != Decision::Neg)
    };
    runs.iter()
        .find(|run| holds(run, [0.0, y, LOW]) && !holds(run, [0.0, -y, LOW]))
        .expect("one of the two runs holds that ruling and not the other")
        .run()
        .clone()
}
