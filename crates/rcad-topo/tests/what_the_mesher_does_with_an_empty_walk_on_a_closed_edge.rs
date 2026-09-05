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
//! The wall on the other side of that same circle is refused too, for a
//! different reason and by a different name — the wall's wire is not one corner,
//! so nothing here would see it. `mesh.rs`'s
//! `the_wall_beside_a_closed_edge_that_walked_to_nothing_is_refused_as_well` is
//! that half, put to `cut_up` directly.
//!
//! **Two barriers and not one, measured.** With `Loop::through`'s three-corner
//! rule loosened to one, the cap sails through and the mesh runs on to the wall,
//! which refuses with `AWireWillNotLieFlatOnItsFace` — so the test below fails
//! with that in its hands rather than passing. Neither face meshes an emptied
//! closed edge, and the two say so for reasons that have nothing to do with each
//! other.
//!
//! # Not the winding question
//!
//! `a_walk_that_came_back_with_no_points.rs` drives the same site —
//! `rcad-geom/src/walk.rs`'s `room_left_over` — into `Model::validate`'s
//! winding, on an **open** edge. §51.7 says out loud that this is a different
//! question, and it is: nothing here reads a sign off a ring.
//!
//! # How the walk is emptied, and why it takes a strange bar
//!
//! `room_left_over` hands a **tolerant** caller an empty point list when the
//! deflection it was given is already inside how far a point of that crossing
//! can be placed. A **certified** caller is refused there instead, and
//! `a_certified_caller_is_told_the_arithmetic_could_not_place_the_points` below
//! asks for exactly that at exactly the same deflection — one `if` with two
//! arms, so nothing else can be making the list empty.
//!
//! Reaching it on a *closed* edge needs the crossing's own rounding to be larger
//! than the floor `SolidView::mesh` puts under a deflection, and those two are
//! deliberately in step for an ordinary bar. They come apart when a cap plane
//! **leans hard**. `where_a_plane_cuts_a_pipe` charges its rounding against the
//! ellipse it cuts, whose semi-axis down the slope is `radius · ‖n‖ / |n·axis|`
//! and grows without bound as the plane lies down; the mesher's floor charges
//! [the face's *corners*], and the seam corner of a bar cut this way stays where
//! it always was. So the two closed edges of one bar answer differently: the
//! flat bottom circle is walked into points and the leaning top's ellipse comes
//! back with none.
//!
//! [the face's *corners*]: rcad_topo::Mesh::deviation
//!
//! That is a real shape and not a rigged one — a bar cut off at an angle is what
//! `add_bar_cut_off_by` is for (§24.2) — but the tilt is far past anything a
//! drawing holds. It is what the arithmetic needs to reach the case, which is
//! why it is built rather than argued about.

use rcad_geom::{Deflection, Isometry, Plane, Point, Scale, Vector};
use rcad_mesh::NotALoop;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, FaceKey, Model, SolidKey};

const RADIUS: f64 = 1.0;

/// How far the top cap leans, as the ratio of its normal across the pipe's axis
/// to its normal along it.
///
/// Two thousand million. What it has to beat is the mesher's floor, and both
/// that and the crossing's rounding are multiples of `f64::EPSILON` — so the
/// lean buys about `LEAN / (corners + radius)` of room between "walked into
/// points" and "walked into none", and this leaves the deflection below sitting
/// comfortably on the far side of it with the bottom circle still walking into a
/// couple of hundred points.
const LEAN: f64 = 2e9;

/// Where the leaning cap crosses the seam, which is the only corner it puts on
/// the wall.
const AT_THE_SEAM: f64 = 0.5;

/// Fine enough that the leaning cap's ellipse comes back with no points, and
/// coarse enough that everything else about the same bar is walked normally.
const A_MESH_THAT_STARVES_THE_LEANING_CAP: f64 = 5e-4;

/// A working size that accepts the wobble the leaning cap's ellipse carries.
///
/// `Policy::tolerant(s)` accepts a wobble up to `64 · f64::EPSILON · s` and the
/// crossing carries `256 · f64::EPSILON` of the numbers it was worked out from,
/// so this has to be a few times [`LEAN`] and it is fifty.
fn a_tolerant_caller() -> Policy {
    Policy::tolerant(50.0 * LEAN)
}

/// The answer §51.7 asked for.
///
/// The mesh is **refused**, the refusal names the cap face the emptied circle
/// bounds, and what it says about it is that its wire fanned into one corner.
#[test]
fn an_empty_walk_on_a_closed_edge_is_refused_and_no_mesh_is_handed_back() {
    let (model, bar) = a_bar_cut_off_almost_along_its_own_axis();

    let starved = model
        .solid(bar)
        .mesh(
            Deflection::of(A_MESH_THAT_STARVES_THE_LEANING_CAP).expect("a positive length"),
            &a_tolerant_caller(),
            &mut ProvenanceLog::new(),
        )
        .expect_err("the leaning cap's ellipse cannot be walked this finely");

    let Error::AWireDoesNotFanIntoALoopToMesh { face, why } = starved else {
        panic!("a closed edge that walked to nothing was refused with {starved:?}");
    };
    assert_eq!(
        why,
        NotALoop::TooFewCorners { count: 1 },
        "a closed edge's one corner is the whole of its cap's wire"
    );
    assert_eq!(
        face,
        the_cap_that_leans(&model, bar),
        "the face refused is the cap the emptied circle bounds"
    );
}

/// The deflection is not what is wrong, and this is what says so.
///
/// An ordinary bar of the same radius meshes at the very number the leaning one
/// is refused at, so the refusal above is about one edge that could not be
/// walked and not about a mesh nobody could have cut.
#[test]
fn an_ordinary_bar_meshes_at_the_deflection_the_leaning_one_is_refused_at() {
    let mut model = Model::new(a_working_size());
    let bar = model
        .add_round_bar(RADIUS, 2.0, &Isometry::identity())
        .expect("a bar");

    let mesh = model
        .solid(bar)
        .mesh(
            Deflection::of(A_MESH_THAT_STARVES_THE_LEANING_CAP).expect("a positive length"),
            &a_tolerant_caller(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar whose caps do not lean is walked into points at this deflection");

    let along_a_cap_circle = model
        .solid(bar)
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
    let (model, bar) = a_bar_cut_off_almost_along_its_own_axis();

    let refused = model
        .solid(bar)
        .mesh(
            Deflection::of(A_MESH_THAT_STARVES_THE_LEANING_CAP).expect("a positive length"),
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

/// A bar whose top cap leans so far that its ellipse is worked out from numbers
/// two thousand million times the bar's own radius.
///
/// The bottom is an ordinary flat cap and stays walkable, which is what makes
/// the two closed edges of one solid answer differently.
fn a_bar_cut_off_almost_along_its_own_axis() -> (Model, SolidKey) {
    let mut model = Model::new(a_working_size());
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let leaning = Plane::from_point_and_normal(
        Point::new(RADIUS, 0.0, AT_THE_SEAM),
        Vector::new(LEAN, 0.0, 1.0),
    )
    .expect("a direction with length");

    let bar = model
        .add_bar_cut_off_by(
            RADIUS,
            [flat, leaning],
            &Isometry::identity(),
            &a_tolerant_caller(),
            &mut ProvenanceLog::new(),
        )
        .expect("a leaning plane cuts an ellipse right round a pipe");
    model
        .validate(&a_tolerant_caller(), &mut ProvenanceLog::new())
        .expect("a bar cut off at an angle is a sound bar however far it leans");
    (model, bar)
}

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

/// The cap face whose plane leans, which is the one this file is about.
///
/// It is the flat face whose own corner sits higher up the seam, because the
/// other one is the bar's bottom.
fn the_cap_that_leans(model: &Model, bar: SolidKey) -> FaceKey {
    let how_high = |face: &rcad_topo::FaceView<'_>| {
        face.wires()
            .flat_map(|wire| wire.points())
            .map(|corner| corner.z())
            .fold(f64::MIN, f64::max)
    };
    model
        .solid(bar)
        .faces()
        .filter(|face| face.surface().as_plane().is_some())
        .max_by(|one, other| how_high(one).total_cmp(&how_high(other)))
        .expect("a bar has two flat caps")
        .key()
}
