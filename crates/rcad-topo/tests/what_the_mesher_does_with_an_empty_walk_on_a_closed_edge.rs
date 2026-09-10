//! A closed edge whose walk came back with no points at all.
//!
//! `docs/decisions.md` §51.7 carries this as **unknown and not reproduced**:
//!
//! > *what the mesher does with an empty walk list on a closed edge — unknown,
//! > and a different question from the winding one that file is about. A closed
//! > edge's one corner sits at both ends of its polyline, so an empty list makes
//! > a wire of a single point.*
//!
//! It is reproduced here, and the answer is that **the mesher refuses the whole
//! mesh by name**: `AWireDoesNotFanIntoALoopToMesh` carrying
//! `NotALoop::TooFewCorners { count: 1 }` and the face it happened on. A closed
//! edge is the only edge of its cap's wire and its one corner is that wire's
//! only corner, so the loop the triangulator is offered has one corner and a
//! loop needs three. Nothing degenerate is triangulated and no
//! [`Mesh`](rcad_topo::Mesh) is handed back, so `points_along_edge` never
//! publishes the empty list to anybody.
//!
//! # Not the winding question
//!
//! `a_walk_that_came_back_with_no_points.rs` drives the same site —
//! `rcad-geom/src/walk.rs`'s `room_left_over` — into `Model::validate`'s
//! winding, on an **open** edge. §51.7 says out loud that this is a different
//! question, and it is: nothing here reads a sign off a ring.
//!
//! # How the walk is emptied, and why it takes a needle
//!
//! `room_left_over` hands a **tolerant** caller an empty point list when the
//! deflection it was given is already inside how far a point of that crossing
//! can be placed. A **certified** caller is refused there instead, and
//! `a_certified_caller_is_told_the_arithmetic_could_not_place_the_points` below
//! asks for exactly that at exactly the same deflection — one `if` with two
//! arms, so nothing else can be making the list empty.
//!
//! Reaching it on a *closed* edge needs the crossing's own rounding to be larger
//! than the floor `SolidView::mesh` puts under a deflection. The crossing rounds
//! against [the size it was worked out from] and the floor charges
//! [how far the points of the face reach], so the site is open exactly where **a
//! face's charged reach is no larger than one of its own crossings**.
//!
//! [the size it was worked out from]: rcad_geom::walk
//! [how far the points of the face reach]: rcad_topo::Mesh::deviation
//!
//! That is not something a leaning cut can buy. A plane cutting a pipe reaches
//! further and further as it lies down, but so does the wall it is charged
//! against, and every term of the ellipse's size is under a term of the wall's
//! reach — so on that family the two never come apart, however hard the plane
//! leans.
//!
//! **The mesher's own fallback is where they do.**
//! `how_far_the_points_of_reach` bounds a pipe or a cone along its axis from the
//! pair of surfaces either side of each of the face's edges, and where no pair
//! bounds one it falls back to the corners plus how far out the surface is
//! *there*. One unbounded edge puts a whole face on that fallback, and a cone
//! crossed with a pipe **on its own axis** is such an edge: nothing about a
//! direction square to the shared axis says how far along the axis the crossing
//! sits, so `room` there comes out negative and no bound is offered. Both curved
//! faces meeting at that circle are then charged from their corners alone.
//!
//! # The shape, and every number in it
//!
//! A pipe of radius one flaring into a **needle cone** on the same axis: the
//! cone's apex is `RADIUS / SLOPE` — sixteen million — below the circle where
//! the two meet, which is at the origin. Flat caps close it a unit below and a
//! unit above. Four faces, and the top one's whole wire is the circle where the
//! lid cuts the cone.
//!
//! Both circles worked out from that far-away apex round by
//! `9.5367431640625e-7` — the junction from the standoff `RADIUS / SLOPE`, the
//! lid's circle from the apex itself — and the deflection below leaves only
//! `1.8749999850119882e-7` of itself for the walk, which is the pair of numbers
//! the certified test below is handed. Meanwhile the coaxial junction puts
//! both curved faces on the fallback, so what they are charged is their corners,
//! a unit out, plus a unit of radius: `2`. So the whole solid's floor is
//! `4.848820849110196e-11` — a hair over what a plain bar of radius one and one
//! tall gets, which is `4.848820560097969e-11` — against circles that round by
//! twenty thousand times that. The slope is ours to choose and the gap has no
//! bound.
//!
//! **Two barriers again, and the shell names the lid first.** The cone face
//! beside the same junction circle cannot be meshed either: its wire is a
//! rectangle whose top and bottom sides are each drawn through one point, and
//! put first in the shell instead it comes back `CannotMeshAFace` with
//! `NotALoop::CornersCoincide { position: 1 }` — measured, by swapping the two
//! entries in [`SolidBuilder::add_shell`] below and reading the panic. So
//! neither face meshes an emptied closed edge, the two say so for reasons that
//! have nothing to do with each other, and which name arrives is which face
//! `SolidView::mesh` asks first — which is the order the shell holds them in.
//!
//! The needle is far past anything a drawing holds — it flares by
//! `5.960464477539063e-8` over its whole unit of height, on a radius of one. It
//! is what the arithmetic needs to
//! reach the case, which is why it is built rather than argued about, and there
//! is a ceiling on it: halving the slope twice more still builds, and the
//! **third** halving is refused by `SolidBuilder::build` itself with
//! `AClosedEdgeIsWalkedAgainstItsSurface`. That is the winding walk emptying,
//! which is §51.7's *other* question rather than this one, so the slope here sits
//! two doublings clear of it.

use rcad_geom::{
    Axis, Cone, Cylinder, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface,
    Vector,
};
use rcad_mesh::NotALoop;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, FaceKey, Model, SolidBuilder, SolidKey};

/// The pipe's radius, and so the radius of the circle it cuts the cone in.
const RADIUS: f64 = 1.0;

/// How far out the needle spreads for every unit along its axis.
///
/// A power of two, twice over. `Model::validate` asks that every corner of a
/// curved face be **certified** on its surface, and a cone's corner satisfies
/// `radius = slope · height` only if that product rounds to nothing —
/// `crates/rcad-topo/tests/cones/mod.rs` is where that is written down. And it
/// is the whole of the gap this file needs: the circles below are worked out
/// from `RADIUS / SLOPE` while the faces are charged for `2`.
const SLOPE: f64 = 1.0 / 16777216.0;

/// Where the needle's apex sits: `RADIUS / SLOPE` below the circle it shares
/// with the pipe, which is at the origin.
const APEX: f64 = -RADIUS / SLOPE;

/// Where the pipe is capped off below.
const FLOOR: f64 = -1.0;

/// Where the needle is capped off above.
const LID: f64 = 1.0;

/// The needle's radius where the lid cuts it, worked out the way the surface
/// works it out so that the corner sits on the cone exactly.
const LID_RADIUS: f64 = SLOPE * (LID - APEX);

/// Fine enough that the two circles the far-away apex was worked into come back
/// with no points, and coarse enough that everything else about the same solid
/// is walked normally.
const A_MESH_THAT_STARVES_THE_NEEDLE: f64 = 1e-6;

/// A working size that accepts the wobble the needle's circles carry.
///
/// `Policy::tolerant(s)` accepts a wobble up to `64 · f64::EPSILON · s` and the
/// crossings carry `256 · f64::EPSILON` of the numbers they were worked out
/// from, so this has to be a few times `RADIUS / SLOPE` and it is fifty.
fn a_tolerant_caller() -> Policy {
    Policy::tolerant(50.0 / SLOPE)
}

/// The answer §51.7 asked for.
///
/// The mesh is **refused**, the refusal names the lid the emptied circle bounds,
/// and what it says about it is that its wire fanned into one corner.
#[test]
fn an_empty_walk_on_a_closed_edge_is_refused_and_no_mesh_is_handed_back() {
    let (model, needle) = a_pipe_flaring_into_a_needle_cone();

    let starved = model
        .solid(needle)
        .expect("this model made the key")
        .mesh(
            Deflection::of(A_MESH_THAT_STARVES_THE_NEEDLE).expect("a positive length"),
            &a_tolerant_caller(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("the needle's circles cannot be walked this finely");

    let Error::AWireDoesNotFanIntoALoopToMesh { face, why } = starved else {
        panic!("a closed edge that walked to nothing was refused with {starved:?}");
    };
    assert_eq!(
        why,
        NotALoop::TooFewCorners { count: 1 },
        "a closed edge's one corner is the whole of its lid's wire"
    );
    assert_eq!(
        face,
        the_lid(&model, needle),
        "the face refused is the lid the emptied circle bounds"
    );
}

/// The deflection is not what is wrong, and this is what says so.
///
/// An ordinary bar of the same radius meshes at the very number the needle is
/// refused at, so the refusal above is about two edges that could not be walked
/// and not about a mesh nobody could have cut.
#[test]
fn an_ordinary_bar_meshes_at_the_deflection_the_needle_is_refused_at() {
    let mut model = Model::new(a_working_size());
    let bar = model
        .add_round_bar(RADIUS, 2.0, &Isometry::identity())
        .expect("a bar");

    let mesh = model
        .solid(bar)
        .expect("this model made the key")
        .mesh(
            Deflection::of(A_MESH_THAT_STARVES_THE_NEEDLE).expect("a positive length"),
            &a_tolerant_caller(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar with no needle in it is walked into points at this deflection");

    let along_a_cap_circle = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .flat_map(|face| face.wires())
        .flat_map(|wire| wire.coedges())
        .map(|coedge| coedge.edge())
        .filter(|edge| {
            edge.is_closed()
                .expect("a bar's cap circle knows it is closed")
        })
        .map(|edge| mesh.points_along_edge(edge.key()).len())
        .max()
        .expect("a bar has two closed edges");
    assert!(
        along_a_cap_circle > 2,
        "a closed edge of an ordinary bar is drawn through {along_a_cap_circle} points at this \
         deflection, so an empty one there would not have stood out"
    );
}

/// The same walk, asked for by a caller who will not accept a wobble.
///
/// `room_left_over` is one `if` with two arms: a tolerant caller gets `Ok` and
/// no points, a certified one gets `CannotPlaceAPointThatCloseToACrossing`. This
/// asks at the very deflection the first test is refused at, so the empty list
/// up there is that site and cannot be anything else.
#[test]
fn a_certified_caller_is_told_the_arithmetic_could_not_place_the_points() {
    let (model, needle) = a_pipe_flaring_into_a_needle_cone();

    let refused = model
        .solid(needle)
        .expect("this model made the key")
        .mesh(
            Deflection::of(A_MESH_THAT_STARVES_THE_NEEDLE).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("a certified caller is never handed a walk it could not place");

    let Error::Geometry(rcad_geom::Error::CannotPlaceAPointThatCloseToACrossing { asked, closest }) =
        refused
    else {
        panic!("a certified caller was refused with {refused:?}");
    };
    assert!(
        closest > asked,
        "the crossing rounds by {closest} and only {asked} was left to walk it in"
    );
}

/// A pipe of radius one, flaring at the origin into a cone whose apex is sixteen
/// million units below it, closed off a unit either way.
///
/// Four faces, and they are described one at a time through [`SolidBuilder`],
/// which is the door an importer comes through. The lid goes into the shell
/// first because it is the face this file is about.
fn a_pipe_flaring_into_a_needle_cone() -> (Model, SolidKey) {
    let mut model = Model::new(a_working_size());
    let up = Vector::new(0.0, 0.0, 1.0);
    let needle = Cone::new(
        Axis::new(Point::new(0.0, 0.0, APEX), up).expect("the z axis is a direction with length"),
        SLOPE,
    )
    .expect("a positive slope");
    let pipe = Cylinder::new(
        Axis::new(Point::origin(), up).expect("the z axis is a direction with length"),
        RADIUS,
    )
    .expect("a positive radius");
    let floor = Plane::from_point_and_normal(Point::new(0.0, 0.0, FLOOR), -up)
        .expect("the z axis is a unit normal");
    let lid = Plane::from_point_and_normal(Point::new(0.0, 0.0, LID), up)
        .expect("the z axis is a unit normal");

    let mut builder = SolidBuilder::new();
    let low = builder
        .add_vertex(Point::new(RADIUS, 0.0, FLOOR))
        .expect("a finite point");
    let middle = builder
        .add_vertex(Point::new(RADIUS, 0.0, 0.0))
        .expect("a finite point");
    let high = builder
        .add_vertex(Point::new(LID_RADIUS, 0.0, LID))
        .expect("a finite point");
    let low_circle = builder
        .add_edge(low, low, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the floor cuts the pipe in one closed curve");
    let junction = builder
        .add_edge(middle, middle, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a cone and a pipe on one axis cross in a circle");
    let high_circle = builder
        .add_edge(high, high, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the lid cuts the cone in one closed curve");
    let pipe_seam = builder
        .add_edge(low, middle, EdgeRun::Straight)
        .expect("a ruling of a pipe is straight");
    let cone_seam = builder
        .add_edge(middle, high, EdgeRun::Straight)
        .expect("a ruling of a cone is straight");

    let mut step = |edge, turn| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("an edge of this build"),
        )
    };
    let round_the_lid = [step(high_circle, Orientation::Forward)];
    let round_the_cone = [
        step(junction, Orientation::Forward),
        step(cone_seam, Orientation::Forward),
        step(high_circle, Orientation::Reversed),
        step(cone_seam, Orientation::Reversed),
    ];
    let round_the_pipe = [
        step(low_circle, Orientation::Forward),
        step(pipe_seam, Orientation::Forward),
        step(junction, Orientation::Reversed),
        step(pipe_seam, Orientation::Reversed),
    ];
    let round_the_floor = [step(low_circle, Orientation::Reversed)];

    let mut face = |surface, steps: &[DraftStep]| {
        let wire = builder.add_wire(steps).expect("a closed wire");
        builder.add_face(surface, &[wire]).expect("a face")
    };
    let lid_face = face(Surface::Plane(lid), &round_the_lid);
    let cone_face = face(Surface::Cone(needle), &round_the_cone);
    let pipe_face = face(Surface::Cylinder(pipe), &round_the_pipe);
    let floor_face = face(Surface::Plane(floor), &round_the_floor);

    builder
        .add_shell(&[
            (lid_face, Orientation::Forward),
            (cone_face, Orientation::Forward),
            (pipe_face, Orientation::Forward),
            (floor_face, Orientation::Forward),
        ])
        .expect("four faces close");
    let solid = builder
        .build(&mut model, &a_tolerant_caller(), &mut ProvenanceLog::new())
        .expect("a pipe flaring into a cone described this way is a solid");
    model
        .validate(&a_tolerant_caller(), &mut ProvenanceLog::new())
        .expect("a pipe flaring into a cone on its own axis is a sound solid");
    (model, solid)
}

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

/// The lid, which is the flat face whose whole wire is the emptied circle.
///
/// It is the flat face sitting higher up the axis, because the other one is the
/// floor.
fn the_lid(model: &Model, needle: SolidKey) -> FaceKey {
    let how_high = |face: &rcad_topo::FaceView<'_>| {
        face.wires()
            .flat_map(|wire| wire.points())
            .map(|corner| corner.z())
            .fold(f64::MIN, f64::max)
    };
    model
        .solid(needle)
        .expect("this model made the key")
        .faces()
        .filter(|face| face.surface().as_plane().is_some())
        .max_by(|one, other| how_high(one).total_cmp(&how_high(other)))
        .expect("this solid has two flat caps")
        .key()
}
