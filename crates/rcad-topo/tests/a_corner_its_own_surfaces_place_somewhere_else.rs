//! A solid whose stored corner is not where its own three planes cross.
//!
//! `docs/decisions.md` §75.6 left one thing owed: the comparison between a
//! baked corner and the corner the model stored ran only where a triple gave
//! **more than one** place, so a corner placed by a single triple was taken
//! without anyone asking whether the model's own corner was anywhere near it.
//! That is §70.3's ill-conditioned corner, and this file is what reaches it.
//!
//! # Why this solid is not a broken one
//!
//! `Model::validate` lets a corner sit off the plane of its face by
//! `Model::allowed_wobble`, which for a 100 mm model is `1e-9` against corners
//! of size one before the plane's own roundoff is added. That is a **modelling
//! tolerance** and not a rounding: it is there so a reader can bring in a file
//! whose numbers were written by somebody else's kernel. So a box with one
//! corner nudged `1e-10` off its top face is a solid this kernel accepts,
//! builds and sweeps.
//!
//! **And fuses, moving the corner onto where its three planes cross** (§368).
//! §17.1 says a face *is* its plane and a stored corner is a cached sample of
//! where the planes cross. A sample the sweep accepts is one the fuse solves
//! again from the planes, and the corner moves by no more than the sweep let it
//! sit off them. §75.6's rule is that a solid the sweep accepts is one the fuse
//! has to be able to bake. Until §368 the fuse compared the stored corner with
//! the crossing within a rounding alone and refused this box `Undecidable`.
//!
//! Nudged along `z` only, so the corner stays *exactly* on the two side planes
//! it also belongs to. One plane disagrees with it and two do not, which is the
//! smallest disagreement that can exist here.
//!
//! # The two boxes here have no width at all
//!
//! Every plane of this box is an axis plane, so the meet of three of them
//! divides by one and comes back as an `f64` triple with a box of no width. The
//! solid carries no placement, so the stored corner is not moved and its box has
//! no width either. Two points, and nothing but the room the fuse grows the
//! stored corner by stands between them — which makes this the file that says
//! how wide that room is.
//!
//! The control below is the first of the tests. The same box with the nudge
//! taken out fuses with nothing approximated, so an answer or a refusal is
//! about the corner and not about a solid described face by face. A corner half
//! a rounding off its top, twice a rounding off it and a tenth of a nanometre
//! off it are all baked: the room is the sweep's modelling tolerance plus a
//! rounding. **The far side of the room cannot be reached through the fuse**:
//! a corner past it is past what the sweep allows, and `SolidBuilder::build`
//! sweeps the solid and refuses it first. The last test shows that refusal.

use rcad_geom::{Deflection, EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, Rule, SolidBuilder, SolidKey};

/// How far the one corner sits off the top of the box, in millimetres.
///
/// Under `Model::allowed_wobble` for this model — `Scale::millimetres(100.0)`
/// gives `1e-9` against corners of size one — and far above anything `f64`
/// rounds by at that size, which is about `2e-16`. So it is a corner somebody
/// *placed* there, not a corner that rounded there, and the sweep takes it.
const NUDGE: f64 = 1e-10;

/// A corner a tenth of a nanometre off its top is baked, and **the answer's
/// corner is where the box's three planes cross**, `(1, 1, 1)` bit for bit.
///
/// The intersection keeps that corner where a union would swallow it inside the
/// cube: the answer is `[0, 1]³`, and its corner at `(1, 1, 1)` is the box's.
/// So this sees where the fuse put it. Until §368 the fuse picked no crossing
/// for the corner and refused the box `Undecidable` at "where three surfaces
/// meet".
///
/// The first assertion is what stops this being a tautology: the nudge is past
/// the rounding the fuse used to grant and inside what the sweep allows.
#[test]
fn a_corner_a_tenth_of_a_nanometre_off_its_own_top_is_moved_onto_where_its_planes_cross() {
    assert!(
        ROOM_FOR_ROUNDING < NUDGE && NUDGE < what_the_sweep_allows_off_a_plane(1.0),
        "the nudge {NUDGE:e} is past the rounding {ROOM_FOR_ROUNDING:e} and inside the {:e} the \
         sweep allows",
        what_the_sweep_allows_off_a_plane(1.0)
    );
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(a_working_size());
    let nudged = a_box_of_side_two(&mut model, NUDGE);
    let overlapping = an_overlapping_cube(&mut model);

    let fused = model
        .fuse(
            &[nudged],
            &[overlapping],
            Rule::Intersection,
            &policy,
            &mut log,
        )
        .expect("a corner the sweep accepts is one the fuse has to be able to bake");

    let [shared] = fused.solids() else {
        panic!(
            "the box and the cube share one unit cube, not {} solids",
            fused.solids().len()
        )
    };
    let volume = model
        .solid(*shared)
        .expect("the fuse made this key")
        .measure(Deflection::for_scale(a_working_size()), &policy, &mut log)
        .expect("a flat solid measures")
        .volume();
    assert!(
        (volume - 1.0).abs() < 10.0 * NUDGE,
        "the box and the cube share [0, 1]³, and this is {volume}"
    );
    let where_the_planes_cross = Point::new(1.0, 1.0, 1.0);
    assert!(
        model
            .vertices()
            .any(|vertex| vertex.point() == where_the_planes_cross),
        "the answer's corner is where the box's three planes cross, bit for bit"
    );
    model
        .validate(&policy, &mut log)
        .expect("and the answer is as sound as the box was");
}

/// The rounding half of the room the fuse grows the stored corner by before
/// comparing it with the place its surfaces cross, at corners of size one:
/// `what_writing_the_corner_down_cost` in `bake.rs`. The other half is what
/// the sweep allows the corner's solid (§368).
///
/// `MEASURING_SLACK` (16) times `Plane::SIGNED_DISTANCE_ROUNDOFF` (`4 · ε`),
/// both spelled out here rather than imported: `MEASURING_SLACK` is
/// `pub(crate)` and this test cannot see it, and even where a constant is
/// public a bound a doc comment publishes wants a test that goes red when
/// somebody moves it (§15.6).
const ROOM_FOR_ROUNDING: f64 = 16.0 * 4.0 * f64::EPSILON;

/// A corner half the rounding off its top is a corner that could have rounded
/// there, so the meet places it and the fuse carries on.
///
/// Without any room, two zero-width boxes overlap only when they are equal,
/// and this corner is not equal by 32 ulps.
#[test]
fn a_corner_half_the_rounding_room_off_its_own_top_is_baked() {
    let mut model = Model::new(a_working_size());
    let barely_off = a_box_of_side_two(&mut model, ROOM_FOR_ROUNDING / 2.0);
    let overlapping = an_overlapping_cube(&mut model);

    let fused = model
        .fuse(
            &[barely_off],
            &[overlapping],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a corner no f64 could have written any nearer is a corner the meet places");

    assert_eq!(fused.solids().len(), 1);
}

/// Twice the rounding off its top is inside the room too, because the room is
/// the sweep's modelling tolerance on top of the rounding (§368).
///
/// Until §368 the room was the rounding alone, this corner was over the line,
/// and the fuse refused it `Undecidable`.
#[test]
fn a_corner_twice_the_rounding_room_off_its_own_top_is_baked() {
    let mut model = Model::new(a_working_size());
    let twice_off = a_box_of_side_two(&mut model, ROOM_FOR_ROUNDING * 2.0);
    let overlapping = an_overlapping_cube(&mut model);

    let fused = model
        .fuse(
            &[twice_off],
            &[overlapping],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("twice the rounding is well inside what the sweep allows");

    assert_eq!(fused.solids().len(), 1);
}

/// A corner just past the fuse's room is refused **by the builder, before the
/// fuse is handed it** — which is why nothing in this file pins the far side of
/// the room through the fuse (§368.3).
///
/// The fuse's room off a plane is what the sweep allows plus a rounding, so a
/// corner past it is past what the sweep allows too, and `SolidBuilder::build`
/// sweeps the solid it builds. This corner is a rounding past the fuse's room.
#[test]
fn a_corner_just_past_the_fuses_room_is_refused_before_the_fuse_sees_it() {
    let past_the_room = what_the_sweep_allows_off_a_plane(1.0) + 2.0 * ROOM_FOR_ROUNDING;
    let mut model = Model::new(a_working_size());

    let refused = a_box_of_side_two_face_by_face(past_the_room).build(
        &mut model,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    let Err(Error::Invalid(Invalid::VertexOffItsFacePlane { distance })) = refused else {
        panic!("a corner past what the sweep allows is off its plane, not {refused:?}")
    };
    assert!(
        distance > what_the_sweep_allows_off_a_plane(1.0),
        "the sweep measured the corner {distance:e} off its top"
    );
}

/// The same box with the nudge taken out, which is the control: everything
/// about the description is the same and only the corner has moved back onto
/// the plane its face sits on.
#[test]
fn the_same_box_with_the_corner_back_where_its_planes_cross_fuses() {
    let mut model = Model::new(a_working_size());
    let square = a_box_of_side_two(&mut model, 0.0);
    let overlapping = an_overlapping_cube(&mut model);
    let mut log = ProvenanceLog::new();

    let fused = model
        .fuse(
            &[square],
            &[overlapping],
            Rule::Union,
            &Policy::certified(),
            &mut log,
        )
        .expect("two lattice-aligned boxes are united by proof alone");

    assert_eq!(fused.solids().len(), 1);
    assert!(log.is_empty(), "nothing here had to be approximated");
}

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

/// How far off its plane `Model::allowed_wobble` lets a corner of a solid of
/// `size` sit in this model: a modelling tolerance and a rounding.
/// `allowed_wobble` is private, so it is written out here.
fn what_the_sweep_allows_off_a_plane(size: f64) -> f64 {
    a_working_size()
        .default_linear_tolerance()
        .min(Scale::RELATIVE_LINEAR_TOLERANCE * size)
        + ROOM_FOR_ROUNDING * size
}

/// A cube of side two the fuse has something to do with: `Model::add_cube` puts
/// it between the origin and `(2, 2, 2)`, so it holds the nudged corner at its
/// own centre and the corner is not merely present but engaged.
fn an_overlapping_cube(model: &mut Model) -> SolidKey {
    model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two")
}

/// [`a_box_of_side_two_face_by_face`], built into `model`.
fn a_box_of_side_two(model: &mut Model, off_the_top: f64) -> SolidKey {
    a_box_of_side_two_face_by_face(off_the_top)
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a corner inside the wobble the sweep allows is a corner the sweep allows")
}

/// A box of side two round the origin, described face by face, with the corner
/// at `(1, 1, 1)` lifted `off_the_top` above the plane its top face sits on.
///
/// Every face is given the axis plane it belongs on, so lifting the corner
/// moves the corner and not the planes. That is the whole of the disagreement:
/// §17.1 says a face *is* its plane and a stored corner is a cached sample of
/// where the planes cross, and here the sample is wrong by more than roundoff.
fn a_box_of_side_two_face_by_face(off_the_top: f64) -> SolidBuilder {
    let mut builder = SolidBuilder::new();
    let corners = [
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0 + off_the_top],
        [-1.0, 1.0, 1.0],
    ]
    .map(|[x, y, z]| {
        builder
            .add_vertex(Point::new(x, y, z))
            .expect("a corner of a box")
    });

    let straight = |builder: &mut SolidBuilder, from: usize, to: usize| {
        builder
            .add_edge(corners[from], corners[to], EdgeRun::Straight)
            .expect("a side of a box")
    };
    let edges = [
        straight(&mut builder, 0, 1),
        straight(&mut builder, 1, 2),
        straight(&mut builder, 2, 3),
        straight(&mut builder, 3, 0),
        straight(&mut builder, 4, 5),
        straight(&mut builder, 5, 6),
        straight(&mut builder, 6, 7),
        straight(&mut builder, 7, 4),
        straight(&mut builder, 0, 4),
        straight(&mut builder, 1, 5),
        straight(&mut builder, 2, 6),
        straight(&mut builder, 3, 7),
    ];

    let faces = [
        (
            [0.0, 0.0, -1.0],
            [(3, false), (2, false), (1, false), (0, false)],
        ),
        (
            [0.0, 0.0, 1.0],
            [(4, true), (5, true), (6, true), (7, true)],
        ),
        (
            [0.0, -1.0, 0.0],
            [(0, true), (9, true), (4, false), (8, false)],
        ),
        (
            [0.0, 1.0, 0.0],
            [(11, true), (6, false), (10, false), (2, true)],
        ),
        (
            [-1.0, 0.0, 0.0],
            [(8, true), (7, false), (11, false), (3, true)],
        ),
        (
            [1.0, 0.0, 0.0],
            [(1, true), (10, true), (5, false), (9, false)],
        ),
    ]
    .map(|(normal, walk)| {
        let steps = walk.map(|(edge, forwards)| {
            let orientation = if forwards {
                Orientation::Forward
            } else {
                Orientation::Reversed
            };
            DraftStep::Along(
                builder
                    .add_coedge(edges[edge], orientation)
                    .expect("one use of a side"),
            )
        });
        let wire = builder.add_wire(&steps).expect("a square boundary");
        let plane = Plane::from_point_and_normal(
            Point::new(normal[0], normal[1], normal[2]),
            Vector::new(normal[0], normal[1], normal[2]),
        )
        .expect("an axis direction is a unit normal");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a side of a box")
    });

    builder
        .add_shell(&faces.map(|face| (face, Orientation::Forward)))
        .expect("six faces close a box");
    builder
}
