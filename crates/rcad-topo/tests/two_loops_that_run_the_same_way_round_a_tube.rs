//! A wall between two circles round a tube that walks both circles the same
//! way round, on a torus and on a pipe, with each flat cap facing the way its
//! own circle's walk says.
//!
//! A band runs on round the tube from one of its circles and back towards the
//! other, so with its material on its left it walks them opposite ways. Two
//! circles walked the same way bound no band, and no answer but a refusal is
//! right. `docs/decisions.md` §343.3 left what each step makes of one unrun.
//!
//! **The builder never writes one.** [`SolidBuilder::build`] runs
//! `Model::validate` over a scratch model first, and refuses
//! `AClosedEdgeIsWalkedAgainstItsSurface`. So `validate` and the mesh are asked
//! of the same face read back from a file: `Model`'s `Deserialize` does not
//! call `validate`. The file is a sound band's bytes with one circle turned
//! round, both of its coedges and the plane of its cap, which is what the
//! builder would have written had it not refused.
//!
//! **`validate` refuses every one, and `SolidView::mesh` and `measure` answer
//! every one.** The mesh closes everywhere but along the turned circle, whose
//! sides the wall and the cap both run the same way round. Nothing in the
//! mesher reads the second circle's direction, and `mesh` does not ask whether
//! its skin closes. These last pins are the defect, not the behaviour wanted.

use std::collections::{BTreeSet, HashMap};

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Orientation, Plane, Point, Scale, Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftFace, DraftStep, Error, Invalid, Mesh, Model, Result, SolidBuilder, SolidKey,
};

use Orientation::{Forward, Reversed};

const RING: f64 = 4.0;
const TUBE: f64 = 1.0;
const RADIUS: f64 = 1.0;
const HEIGHT: f64 = 2.0;

/// The second circle round the tube a quarter and half a turn round the ring,
/// `(cos θ, sin θ)` written out exactly.
const SECOND_CIRCLES: [[f64; 2]; 2] = [[0.0, 1.0], [-1.0, 0.0]];

/// Three deflections, the coarsest the working size's own.
const DEFLECTIONS: [f64; 3] = [0.1, 0.03, 0.01];

#[test]
fn a_torus_wall_walking_both_its_circles_one_way_is_refused_by_the_builder() {
    for second in SECOND_CIRCLES {
        for walk in [Forward, Reversed] {
            let mut model = a_working_model();
            let built = a_band(&mut model, second, [walk, walk]);
            assert!(
                matches!(
                    built,
                    Err(Error::Invalid(
                        Invalid::AClosedEdgeIsWalkedAgainstItsSurface
                    ))
                ),
                "the band to {second:?} walking both circles {walk:?} came back {built:?}"
            );
            assert_eq!(model.solid_count(), 0, "a refused build writes nothing");
        }
    }
}

#[test]
fn a_torus_wall_walking_both_its_circles_one_way_read_back_from_a_file_is_refused_by_validate() {
    for case in the_torus_walls() {
        refused_by_validate(&case);
    }
}

#[test]
fn a_torus_wall_walking_both_its_circles_one_way_read_back_from_a_file_meshes_with_its_skin_open() {
    for case in the_torus_walls() {
        meshed_and_measured_with_its_skin_open(&case);
    }
}

#[test]
fn a_pipe_wall_walking_both_its_circles_one_way_is_refused_by_the_builder() {
    for walk in [Forward, Reversed] {
        let mut model = a_working_model();
        let built = a_piece_of_pipe(&mut model, [walk, walk]);
        assert!(
            matches!(
                built,
                Err(Error::Invalid(
                    Invalid::AClosedEdgeIsWalkedAgainstItsSurface
                ))
            ),
            "the pipe walking both circles {walk:?} came back {built:?}"
        );
        assert_eq!(model.solid_count(), 0, "a refused build writes nothing");
    }
}

#[test]
fn a_pipe_wall_walking_both_its_circles_one_way_read_back_from_a_file_is_refused_by_validate() {
    for case in the_pipe_walls() {
        refused_by_validate(&case);
    }
}

#[test]
fn a_pipe_wall_walking_both_its_circles_one_way_read_back_from_a_file_meshes_with_its_skin_open() {
    for case in the_pipe_walls() {
        meshed_and_measured_with_its_skin_open(&case);
    }
}

/// One wall walking both its circles `walk`, written into a file.
struct OneWay {
    what: String,
    bytes: Vec<u8>,
    walk: Orientation,
    /// Each circle's one corner, in the order the circles were added.
    corners: [Point; 2],
    /// The plane of the cap on each circle, facing the way the walk says.
    caps: [Plane; 2],
    /// Which circle was turned round from the sound solid's bytes.
    turned: usize,
}

/// Every band of [`a_band`] walking both circles one way, each read off the
/// sound band that walks its first circle the same way.
fn the_torus_walls() -> Vec<OneWay> {
    let mut walls = Vec::new();
    for second in SECOND_CIRCLES {
        for walk in [Forward, Reversed] {
            let mut sound = a_working_model();
            a_band(&mut sound, second, [walk, walk.reversed()])
                .expect("a band walking its circles opposite ways is a solid");
            let [cos, sin] = second;
            let inner = RING - TUBE;
            walls.push(OneWay {
                what: format!("the band to {second:?} walking both circles {walk:?}"),
                bytes: with_a_circle_turned_round(&sound, 2, a_ring_cap(second, walk.reversed())),
                walk,
                corners: [
                    Point::new(inner, 0.0, 0.0),
                    Point::new(cos * inner, sin * inner, 0.0),
                ],
                caps: [a_ring_cap([1.0, 0.0], walk), a_ring_cap(second, walk)],
                turned: 1,
            });
        }
    }
    walls
}

/// [`a_piece_of_pipe`] walking both circles each way, read off the sound pipe:
/// its top circle turned round for walking both forwards, its bottom one for
/// walking both reversed.
fn the_pipe_walls() -> Vec<OneWay> {
    let mut sound = a_working_model();
    a_piece_of_pipe(&mut sound, [Forward, Reversed]).expect("a sound piece of pipe");
    [(Forward, 1), (Reversed, 0)]
        .into_iter()
        .map(|(walk, turned)| {
            let heights = [0.0, HEIGHT];
            OneWay {
                what: format!("the pipe walking both circles {walk:?}"),
                bytes: with_a_circle_turned_round(
                    &sound,
                    turned as u8 + 1,
                    a_pipe_cap(heights[turned], walk.reversed()),
                ),
                walk,
                corners: heights.map(|height| Point::new(RADIUS, 0.0, height)),
                caps: heights.map(|height| a_pipe_cap(height, walk)),
                turned,
            }
        })
        .collect()
}

/// The file reads back, holds what the builder was asked for, and
/// `Model::validate` refuses it as the builder does.
fn refused_by_validate(case: &OneWay) {
    let model = read_back(case);
    let validated = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
    assert!(
        matches!(
            validated,
            Err(Error::Invalid(
                Invalid::AClosedEdgeIsWalkedAgainstItsSurface
            ))
        ),
        "{}: validate gave {validated:?}",
        case.what
    );
}

/// **The defect, pinned as it stands**: the mesh and the measure answer at
/// every deflection, and the only sides of the mesh not used once each way
/// round are those along the turned circle, which its wall and its cap both
/// run the same way. `points_along_edge` holds a circle's points short of its
/// corner, so `n` of them close into `n + 1` sides.
fn meshed_and_measured_with_its_skin_open(case: &OneWay) {
    let policy = Policy::certified();
    let model = read_back(case);
    let solid = model.solids().next().expect("one solid read back");
    let turned = model
        .edges()
        .find(|edge| edge.start().point() == case.corners[case.turned])
        .expect("the turned circle")
        .key();
    for length in DEFLECTIONS {
        let deflection = Deflection::of(length).expect("a length");
        let mesh = solid
            .mesh(deflection, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{} at {length}: the mesh refused {why:?}", case.what));
        let open = sides_used_twice_the_same_way(&mesh);
        let along: BTreeSet<u32> = mesh.points_along_edge(turned).iter().copied().collect();
        assert!(
            open.len() == along.len() + 1
                && open
                    .iter()
                    .all(|(from, to)| along.contains(from) || along.contains(to)),
            "{} at {length}: the sides {open:?} are open, and {along:?} are along the \
             turned circle short of its corner",
            case.what
        );
        solid
            .measure(deflection, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{} at {length}: the measure refused {why:?}", case.what));
    }
}

/// The model in `case`'s file, checked to walk each circle as `case` says.
fn read_back(case: &OneWay) -> Model {
    let model: Model = postcard::from_bytes(&case.bytes)
        .unwrap_or_else(|why| panic!("{}: the file is let in: {why:?}", case.what));
    for face in model.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let circle = case
                    .corners
                    .iter()
                    .position(|&corner| corner == coedge.edge().start().point())
                    .expect("every edge is one of the two circles");
                let walked = coedge.orientation_on_its_edge();
                match face.surface().as_plane() {
                    Some(plane) => {
                        assert_eq!(
                            plane.coefficients(),
                            case.caps[circle].coefficients(),
                            "{}: the cap on circle {circle}",
                            case.what
                        );
                        assert_eq!(walked, case.walk.reversed(), "{}: a cap", case.what);
                    }
                    None => assert_eq!(walked, case.walk, "{}: the wall", case.what),
                }
            }
        }
    }
    model
}

fn sides_used_twice_the_same_way(mesh: &Mesh) -> BTreeSet<(u32, u32)> {
    let mut balance: HashMap<(u32, u32), i32> = HashMap::new();
    for triangle in mesh.triangles() {
        let [first, second, third] = triangle.corners();
        for (from, to) in [(first, second), (second, third), (third, first)] {
            *balance.entry((from.min(to), from.max(to))).or_default() +=
                if from < to { 1 } else { -1 };
        }
    }
    balance
        .into_iter()
        .filter(|&(_, count)| count != 0)
        .map(|(side, _)| side)
        .collect()
}

fn a_working_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// The band of the ring `RING`, `TUBE` about `+z` between the circle round
/// `(RING, 0, 0)` and the one round `RING·(cos θ, sin θ, 0)`, its wall walking
/// the two circles `walks`, with a flat cap on each circle's plane facing the
/// way its circle is walked ([`a_ring_cap`]).
///
/// `[Reversed, Forward]` is `two_loops_side_by_side_on_a_torus.rs`'s band
/// anticlockwise from `+x`, `[Forward, Reversed]` the rest of the ring.
fn a_band(model: &mut Model, [cos, sin]: [f64; 2], walks: [Orientation; 2]) -> Result<SolidKey> {
    let inner = RING - TUBE;
    let mut builder = SolidBuilder::new();
    let first_corner = builder.add_vertex(Point::new(inner, 0.0, 0.0))?;
    let second_corner = builder.add_vertex(Point::new(cos * inner, sin * inner, 0.0))?;
    let first_circle = builder.add_edge(
        first_corner,
        first_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let second_circle = builder.add_edge(
        second_corner,
        second_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let first_cap = a_cap(
        &mut builder,
        a_ring_cap([1.0, 0.0], walks[0]),
        first_circle,
        walks[0],
    )?;
    let second_cap = a_cap(
        &mut builder,
        a_ring_cap([cos, sin], walks[1]),
        second_circle,
        walks[1],
    )?;
    let ring = Torus::new(Axis::z(), RING, TUBE).expect("a tube narrower than its ring");
    let wall = a_wall(
        &mut builder,
        Surface::Torus(ring),
        [first_circle, second_circle],
        walks,
    )?;
    builder.add_shell(&[(first_cap, Forward), (second_cap, Forward), (wall, Forward)])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// A pipe of `RADIUS` about `+z` from `z = 0` to `HEIGHT` with no seam, its
/// wall walking the bottom and top circles `walks`, with a flat cap on each
/// facing the way its circle is walked ([`a_pipe_cap`]).
/// `[Forward, Reversed]` is the sound one.
fn a_piece_of_pipe(model: &mut Model, walks: [Orientation; 2]) -> Result<SolidKey> {
    let mut builder = SolidBuilder::new();
    let bottom_corner = builder.add_vertex(Point::new(RADIUS, 0.0, 0.0))?;
    let top_corner = builder.add_vertex(Point::new(RADIUS, 0.0, HEIGHT))?;
    let bottom_circle = builder.add_edge(
        bottom_corner,
        bottom_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let top_circle = builder.add_edge(
        top_corner,
        top_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let bottom = a_cap(
        &mut builder,
        a_pipe_cap(0.0, walks[0]),
        bottom_circle,
        walks[0],
    )?;
    let top = a_cap(
        &mut builder,
        a_pipe_cap(HEIGHT, walks[1]),
        top_circle,
        walks[1],
    )?;
    let pipe = Cylinder::new(Axis::z(), RADIUS).expect("a radius");
    let wall = a_wall(
        &mut builder,
        Surface::Cylinder(pipe),
        [bottom_circle, top_circle],
        walks,
    )?;
    builder.add_shell(&[(bottom, Forward), (top, Forward), (wall, Forward)])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The plane through the axis holding the circle round the tube at
/// `RING·(cos, sin, 0)`, facing on round the ring anticlockwise when the wall
/// walks that circle forwards and back when it walks it reversed: away from
/// the side the walk puts the wall on.
fn a_ring_cap([cos, sin]: [f64; 2], walk: Orientation) -> Plane {
    let on_round = match walk {
        Forward => 1.0,
        Reversed => -1.0,
    };
    Plane::from_point_and_normal(
        Point::origin(),
        Vector::new(-sin * on_round, cos * on_round, 0.0),
    )
    .expect("a coordinate direction is a unit normal")
}

/// The plane `z = height`, facing down when the wall walks the circle on it
/// forwards, which puts the wall above it, and up when reversed.
fn a_pipe_cap(height: f64, walk: Orientation) -> Plane {
    let up = match walk {
        Forward => -1.0,
        Reversed => 1.0,
    };
    Plane::from_point_and_normal(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, up))
        .expect("a coordinate direction is a unit normal")
}

/// A flat face on `plane` bounded by `circle`, walked against the wall's
/// `walk` of it.
fn a_cap(
    builder: &mut SolidBuilder,
    plane: Plane,
    circle: DraftEdge,
    walk: Orientation,
) -> Result<DraftFace> {
    let round_it = builder.add_coedge(circle, walk.reversed())?;
    let wire = builder.add_wire(&[DraftStep::Along(round_it)])?;
    builder.add_face(Surface::Plane(plane), &[wire])
}

/// A face on `surface` with one wire per circle and no seam.
fn a_wall(
    builder: &mut SolidBuilder,
    surface: Surface,
    circles: [DraftEdge; 2],
    walks: [Orientation; 2],
) -> Result<DraftFace> {
    let mut wires = Vec::with_capacity(2);
    for (circle, walk) in circles.into_iter().zip(walks) {
        let round_it = builder.add_coedge(circle, walk)?;
        wires.push(builder.add_wire(&[DraftStep::Along(round_it)])?);
    }
    builder.add_face(surface, &wires)
}

/// The bytes of one coedge slot: "something here", its edge's two-byte key,
/// its orientation, and the slot's version.
const ONE_COEDGE: usize = 5;

/// The coedge arena of a solid of two circles: three slots' worth of count
/// and empty slot zero, then the caps' two coedges and the wall's two.
const FOUR_COEDGES: usize = 3 + 4 * ONE_COEDGE;

/// `model`'s bytes with the circle added `circle`-th, counting from one,
/// walked the other way by both faces along it, and its cap, on `cap`, turned
/// to face the other way.
///
/// Each arena is written as its number of slots, then each slot: an empty slot
/// zero, then a `1` for "something here", the entity and the slot's version,
/// `1` for everything written once (as `a_skin_that_closes_only_with_another_
/// never_reads_back.rs` reads it). A plane is its four coefficients. Each
/// place is found by its layout and asserted to be the only one;
/// [`read_back`] checks the answer through the views.
fn with_a_circle_turned_round(model: &Model, circle: u8, cap: Plane) -> Vec<u8> {
    let mut bytes = postcard::to_allocvec(model).expect("a model writes out");
    let coedges = the_one_place(&bytes, "the coedge arena", holds_four_coedges);
    let mut turned = 0;
    for slot in bytes[coedges + 3..coedges + FOUR_COEDGES].chunks_mut(ONE_COEDGE) {
        if slot[1] == circle {
            slot[3] = 1 - slot[3];
            turned += 1;
        }
    }
    assert_eq!(turned, 2, "a circle is walked by its wall and its cap");

    let written = written_as(cap);
    let after = written.len() + 2;
    let at = 2 + the_one_place(&bytes, "the cap", |here| {
        here.len() >= after + 5
            && here[..2] == [1, 0]
            && here[2..after] == written[..]
            && here[after..after + 5] == [1, circle, 1, 0, 1]
    });
    bytes[at..at + written.len()].copy_from_slice(&written_as(cap.reversed()));
    bytes
}

fn holds_four_coedges(bytes: &[u8]) -> bool {
    bytes.len() >= FOUR_COEDGES
        && bytes[..3] == [5, 0, 0]
        && bytes[3..FOUR_COEDGES].chunks(ONE_COEDGE).all(|slot| {
            slot[0] == 1
                && (1..=2).contains(&slot[1])
                && slot[2] == 1
                && slot[3] <= 1
                && slot[4] == 1
        })
}

fn written_as(plane: Plane) -> Vec<u8> {
    plane
        .coefficients()
        .iter()
        .flat_map(|number| number.to_le_bytes())
        .collect()
}

fn the_one_place(bytes: &[u8], what: &str, holds: impl Fn(&[u8]) -> bool) -> usize {
    let places: Vec<usize> = (0..bytes.len()).filter(|&at| holds(&bytes[at..])).collect();
    let [at] = places[..] else {
        panic!("{what} is written at {places:?}");
    };
    at
}
