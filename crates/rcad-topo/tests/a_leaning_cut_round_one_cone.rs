//! A frustum split into two bands by a **leaning** plane, described through the
//! public builder.
//!
//! §215.1 built §214.4's fix (a): one cone twice, with a closed run, is answered
//! as the circle through the run's own end square to the cone's axis. It also
//! wrote down what that arm guesses — a plane leaning across the axis cuts a
//! cone in an **ellipse**, which is closed too, so such a run is handed the same
//! arm and named a circle — and it measured only that no *fuse* hands the arm
//! such a run today. §16.10 says a claim that nothing reaches something is about
//! the **public API**, so this file asks the builder.
//!
//! The answer is that the builder reaches it, in four faces, and that the wrong
//! answer is not a rounding: the rim the mesher builds stands **1.45 mm** off
//! the plane that was meant to cut it, on a solid whose working size is 100 mm.
//!
//! # The shape
//!
//! A cone about `+z`, apex at the origin, a half out per unit up. A frustum of
//! it between `z = 4` and `z = 8`. The plane `x + 4z = 27` leans across the
//! axis by about fourteen degrees, which is well inside the cone's own
//! twenty-seven, so it cuts the cone in an ellipse rather than in a parabola or
//! a hyperbola. That ellipse runs from `(3, 0, 6)` round to
//! `(-3.857…, 0, 7.714…)` — a rim that climbs 1.7 up the axis as it goes round.
//!
//! Split there, the frustum's wall is two faces, both on the **one** cone, and
//! the rim between them is one closed edge with that cone either side. That is
//! exactly the pair and exactly the run `what_a_run_between_these_two_lies_on`'s
//! same-surface arm answers.
//!
//! # What the model can say, and what it cannot
//!
//! It cannot say which plane cut it. The cutting plane is **not a surface of
//! this solid** — no face of the split frustum lies on it — so there is nowhere
//! in the model for it to be. What the rim carries is a run kind and one corner,
//! and one point on a cone does not name an ellipse. The model below is
//! therefore *ambiguous*: it is the same description a frustum split by the
//! level plane `z = 6` would have, and the kernel answers it as that one.
//!
//! So this is not a bug in the arm's arithmetic. It is a **vocabulary** gap, and
//! §214.4's (d) — writing the cutting plane into the run — is what would close
//! it. This file is the demonstration that asks for it.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey};

/// Apex at the origin, a half out per unit up. A power of two, so every
/// corner's `radius = slope · height` is an exact product and the sweep
/// certifies it (`tests/cones/mod.rs` says why).
const SLOPE: f64 = 0.5;
const BOTTOM: f64 = 4.0;
const TOP: f64 = 8.0;

/// `x + 4z = 27`, through `(3, 0, 6)`, which is on the cone.
const LEAN: f64 = 1.0;
const UP: f64 = 4.0;
const OFFSET: f64 = 27.0;

/// Where the cutting plane crosses the seam ruling at `y = 0, x > 0`.
const RIM_HEIGHT: f64 = 6.0;

/// Said the only way it could be said before §216, the rim is a circle and the
/// cut is 1.45 mm away from where it was asked for.
#[test]
fn a_rim_that_names_no_cutting_plane_is_answered_as_a_circle() {
    let (level, worst) = what_the_mesh_put_on_the_rim(EdgeRun::RightRoundWhereItsFacesCross);
    println!(
        "a rim with no plane in it: {level} mesh points level with its corner, the furthest of \
         them {worst} from the plane that was meant to cut the cone"
    );

    assert!(
        level > 8,
        "the rim is one closed curve on the cone, so the mesh carries a ring of points along it; \
         only {level} came back level with its corner"
    );
    assert!(
        worst > 1.4,
        "a rim that names no plane stands {worst} off the one that was meant to cut it. A number \
         this small means one point on a cone started naming an ellipse, which it cannot — so \
         re-read what this file claims (docs/decisions.md §216)"
    );
}

/// The same rim, saying which plane cut it, is the ellipse.
///
/// **This is the one that fails without §216's variant.** Said as
/// [`EdgeRun::RightRoundWhereItCrosses`], the rim's pair is the cone and the
/// cutting plane rather than the cone twice, so `where_a_plane_cuts_a_cone`
/// answers it — and that function has built all four shapes since §214.
#[test]
fn a_rim_that_names_its_cutting_plane_is_answered_as_the_ellipse() {
    let (level, worst) = what_the_mesh_put_on_the_rim(EdgeRun::RightRoundWhereItCrosses {
        and: Surface::Plane(the_cutting_plane()),
    });
    println!(
        "a rim that names its plane: {level} mesh points level with its corner, the furthest of \
         them {worst} from the plane that was meant to cut the cone"
    );

    assert_eq!(
        level, 1,
        "the ellipse crosses the level of its own corner once and once only, so exactly one mesh \
         point belongs there; {level} of them says a circle was built again"
    );
    assert!(
        worst < 1e-12,
        "the corner of the rim is on the cutting plane by construction, and came back {worst} off it"
    );
}

/// How many mesh points sit exactly level with the rim's own corner, and how far
/// the furthest of them stands from the plane that was meant to make the cut.
///
/// A ring of mesh points level with `(3, 0, 6)` is the **circle**. The ellipse
/// meets that level in one point and one only, so the count is what tells the
/// two answers apart, and the distance from `x + 4z = 27` is how far apart they
/// are.
fn what_the_mesh_put_on_the_rim(rim: EdgeRun) -> (usize, f64) {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_frustum_split_by_a_leaning_plane(&mut model, rim, &policy, &mut log)
        .expect("a frustum split in two along a rim on its own cone is a solid");
    model
        .validate(&policy, &mut log)
        .expect("the split frustum is sound");

    let mesh = model
        .solid(solid)
        .expect("this model made the key")
        .mesh(
            Deflection::of(0.01).expect("a positive length"),
            &policy,
            &mut log,
        )
        .expect("the split frustum meshes");

    let across = (LEAN * LEAN + UP * UP).sqrt();
    let mut level = 0;
    let mut worst: f64 = 0.0;
    for point in mesh.points() {
        if point.z() != RIM_HEIGHT {
            continue;
        }
        level += 1;
        worst = worst.max((LEAN * point.x() + UP * point.z() - OFFSET).abs() / across);
    }
    (level, worst)
}

/// `x + 4z = 27`, leaning about fourteen degrees across the axis of a cone whose
/// own half-angle is twenty-seven, so what it cuts is an ellipse.
fn the_cutting_plane() -> Plane {
    Plane::from_point_and_normal(
        Point::new(SLOPE * RIM_HEIGHT, 0.0, RIM_HEIGHT),
        Vector::new(LEAN, 0.0, UP),
    )
    .expect("a normal with a length is a plane")
}

/// The frustum, its wall cut in two along a closed rim through `(3, 0, 6)`.
///
/// Four faces: the two caps, and two bands **both on the one cone**. The rim
/// between the bands is one closed edge, and the cutting plane appears nowhere
/// — which is the whole point.
fn a_frustum_split_by_a_leaning_plane(
    model: &mut Model,
    rim_run: EdgeRun,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> rcad_topo::Result<SolidKey> {
    let skin = Cone::new(Axis::z(), SLOPE).expect("a positive slope");
    let mut builder = SolidBuilder::new();

    let bottom_corner = builder.add_vertex(Point::new(SLOPE * BOTTOM, 0.0, BOTTOM))?;
    let rim_corner = builder.add_vertex(Point::new(SLOPE * RIM_HEIGHT, 0.0, RIM_HEIGHT))?;
    let top_corner = builder.add_vertex(Point::new(SLOPE * TOP, 0.0, TOP))?;

    let lower_seam = builder.add_edge(bottom_corner, rim_corner, EdgeRun::Straight)?;
    let upper_seam = builder.add_edge(rim_corner, top_corner, EdgeRun::Straight)?;
    let bottom_circle = builder.add_edge(
        bottom_corner,
        bottom_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let top_circle = builder.add_edge(top_corner, top_corner, EdgeRun::RightRoundWhereItsFacesCross)?;
    let rim = builder.add_edge(rim_corner, rim_corner, rim_run)?;

    let bottom = a_cap(&mut builder, BOTTOM, -1.0, bottom_circle, Orientation::Reversed)?;
    let top = a_cap(&mut builder, TOP, 1.0, top_circle, Orientation::Forward)?;
    let lower_band = a_band(&mut builder, skin, bottom_circle, lower_seam, rim)?;
    let upper_band = a_band(&mut builder, skin, rim, upper_seam, top_circle)?;

    builder.add_shell(&[
        (bottom, Orientation::Forward),
        (top, Orientation::Forward),
        (lower_band, Orientation::Forward),
        (upper_band, Orientation::Forward),
    ])?;
    builder.build(model, policy, log)
}

fn a_cap(
    builder: &mut SolidBuilder,
    at: f64,
    outward: f64,
    edge: rcad_topo::DraftEdge,
    turn: Orientation,
) -> rcad_topo::Result<rcad_topo::DraftFace> {
    let plane =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, at), Vector::new(0.0, 0.0, outward))
            .expect("the z axis is a unit normal");
    let coedge = builder.add_coedge(edge, turn)?;
    let wire = builder.add_wire(&[DraftStep::Along(coedge)])?;
    builder.add_face(Surface::Plane(plane), &[wire])
}

/// One band of the wall: round its lower rim, up the seam, back round its upper
/// rim, down the seam. The frustum's own wall wire, with either end allowed to
/// be a cap circle or the split.
fn a_band(
    builder: &mut SolidBuilder,
    skin: Cone,
    lower: rcad_topo::DraftEdge,
    seam: rcad_topo::DraftEdge,
    upper: rcad_topo::DraftEdge,
) -> rcad_topo::Result<rcad_topo::DraftFace> {
    let mut steps = Vec::new();
    for (edge, turn) in [
        (lower, Orientation::Forward),
        (seam, Orientation::Forward),
        (upper, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ] {
        steps.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
    }
    let wire = builder.add_wire(&steps)?;
    builder.add_face(Surface::Cone(skin), &[wire])
}
