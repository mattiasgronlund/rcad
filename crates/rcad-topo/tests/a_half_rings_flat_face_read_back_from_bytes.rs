//! A half-ring's flat face, read back from bytes, holds the ring-shaped piece
//! of its plane that it is. `docs/decisions.md` §250.7 and §250.9 are the
//! record.
//!
//! The half-ring is the ring of radius four and tube one about `z`, cut by
//! `z = 0`, keeping `z ≥ 0`. Its flat face is `z = 0` facing `−z`, bounded by
//! the two circles `z = 0` cuts the ring in, of radius five and three, each one
//! closed edge with its corner on the `+x` axis. So the face is the annulus
//! `3 ≤ r ≤ 5` at `z = 0`, with `r` the distance from the `z` axis, and every
//! answer below is read off `r` against those two radii:
//!
//! | point | `r` | the flat face | the half-ring |
//! |---|---|---|---|
//! | `(4, 0, 0)` | 4 | held | on its skin |
//! | `(3.5, 0, 0)` | 3.5 | held | |
//! | `(0, 0, 0)` | 0 | not held, in the hole | outside |
//! | `(6, 0, 0)` | 6 | not held, beyond the rim | |
//! | `(5, 0, 0)` | 5 | on its boundary, the outer circle | |
//! | `(3, 0, 0)` | 3 | on its boundary, the inner circle | |
//!
//! `SolidBuilder` refuses the half-ring (§250.7), and `Deserialize` does not
//! call `Model::validate`, so the model is written straight into bytes through
//! [`mirror`], a copy of the model's arenas, and read back. Each closed edge's
//! direction on the flat face and the wall's orientation in the shell are each
//! tried both ways, eight models in all; none of the six answers depends on
//! them, because a round wire is counted by its place in the face's list and
//! not by its direction.
//!
//! Only one of the eight is a solid, the half-ring. `Model::validate` passes
//! it and refuses the other seven
//! ([`validate_refuses_every_wall_walking_its_two_circles_the_same_way_round`]).
//! It used to pass one more, which walks the flat face's hole the same way
//! round as its outline and uses the wall `Forward`, and the answer pins were
//! asked of that one as though it were a solid. They leave it out now
//! ([`is_the_hole_walked_the_way_its_outline_runs`]).
//!
//! On `c564dec5` every one of the six was refused
//! `AFaceTrimmedByACurveThisCannotFollow`, and both points given to
//! `classify_point` were refused `CannotFollowAFacesTrimming` naming the flat
//! face. `intersect` names the two circles a plane square to a ring's axis cuts
//! by the two sides of the pipe of the ring's own radius, and the lookup the
//! flat face asked took only a flat half-space.
//!
//! On `c11bf21f` the wall refused every point asked of it,
//! `AFaceMissesPartOfWhereItsSurfacesCross`, so `classify_point` at
//! `(4, 0, 0)` was refused naming the wall: a ray from it meets the wall, and
//! the rule for a ring face between planes followed only planes through the
//! ring's axis. A plane square to the axis is followed now, and the wall is
//! the half of the ring its wire keeps, wherever its two circles agree on
//! which half that is (§243.8).

use rcad_geom::{Axis, Isometry, Orientation, Plane, Point, Scale, Surface, Torus, Vector};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, FORMAT_VERSION, FaceKey, Invalid, Model, OnAFace, SolidKey};

mod rings;

const RING: f64 = 4.0;
const TUBE: f64 = 1.0;

/// A wall that walks its two circles the same way round keeps one side of
/// `z = 0` at one circle and the other side at the other, so it is neither
/// half of the ring, and `Model::validate` refuses it by reading that side off
/// the wire at each circle. Every corner of the flat face is on the ring and
/// every corner of the wall on the plane, so no corner could say it, and the
/// mesh draws the hole-walked-wrong choice as the half-ring's own triangles.
///
/// The half-ring validates. The three other choices are refused as they were
/// before the wire was read: walking the outer circle `Reversed`,
/// `FacesDoNotAgreeWhichWayTheyFace`, and the half-ring's walk with the wall
/// used `Reversed`, `VolumeIsNotPositive`.
#[test]
fn validate_refuses_every_wall_walking_its_two_circles_the_same_way_round() {
    for choice in every_choice() {
        let half_ring = read_back(choice);
        let swept = half_ring
            .model
            .validate(&Policy::certified(), &mut ProvenanceLog::new());
        if the_side_of_z_0_the_wall_keeps(choice).is_none() {
            assert!(
                matches!(
                    swept,
                    Err(Error::Invalid(
                        Invalid::AClosedEdgeIsWalkedAgainstItsSurface
                    ))
                ),
                "{choice:?}: {swept:?}"
            );
        } else if is_the_half_ring(choice) {
            assert!(swept.is_ok(), "{choice:?}: {swept:?}");
        } else {
            assert!(swept.is_err(), "{choice:?}");
        }
    }
}

/// Answers about the flat face, asked of every choice but
/// [`is_the_hole_walked_the_way_its_outline_runs`].
#[test]
fn the_flat_face_holds_the_annulus_between_its_two_circles() {
    let expected = [
        ([4.0, 0.0, 0.0], OnAFace::Held),
        ([3.5, 0.0, 0.0], OnAFace::Held),
        ([0.0, 0.0, 0.0], OnAFace::NotHeld),
        ([6.0, 0.0, 0.0], OnAFace::NotHeld),
        ([5.0, 0.0, 0.0], OnAFace::OnItsBoundary),
        ([3.0, 0.0, 0.0], OnAFace::OnItsBoundary),
    ];
    for choice in
        every_choice().filter(|&choice| !is_the_hole_walked_the_way_its_outline_runs(choice))
    {
        let half_ring = read_back(choice);
        let view = half_ring.model.solid(half_ring.solid).expect("its own key");
        for (point, answer) in expected {
            let held = view.face_holds_the_point(
                half_ring.flat,
                PredicatePoint::from(point),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            assert!(
                matches!(held, Ok(got) if got == answer),
                "{choice:?} at {point:?}: {held:?}"
            );
        }
    }
}

/// The middle of the hole is outside the half-ring, and a point on the flat
/// face is on its skin. The ray from that point meets the wall, so the wall is
/// asked too, and where its wire keeps one side of `z = 0` at one circle and
/// the other side at the other it refuses (§243.8). The flat face holds the
/// point all the same, so it is on the skin on every choice (Q1722).
///
/// Asked of every choice but [`is_the_hole_walked_the_way_its_outline_runs`].
#[test]
fn classify_point_finds_the_hole_outside_and_the_flat_face_on_the_skin() {
    for choice in
        every_choice().filter(|&choice| !is_the_hole_walked_the_way_its_outline_runs(choice))
    {
        let half_ring = read_back(choice);
        let view = half_ring.model.solid(half_ring.solid).expect("its own key");
        let policy = Policy::certified();

        let in_the_hole = view.classify_point(
            Point::new(0.0, 0.0, 0.0),
            &policy,
            &mut ProvenanceLog::new(),
        );
        assert_eq!(in_the_hole, Containment::Outside, "{choice:?}");

        let on_the_face = view.classify_point(
            Point::new(4.0, 0.0, 0.0),
            &policy,
            &mut ProvenanceLog::new(),
        );
        assert_eq!(on_the_face, Containment::OnBoundary, "{choice:?}");
    }
}

/// The wall is the half of the ring on the side of `z = 0` its wire keeps,
/// and holds nothing of the other half; its two circles are its boundary.
/// Where its wire keeps different sides at its two circles, no half of the
/// ring is that, and every point is refused (§243.8).
///
/// Every point asked is exactly on the ring: at the top or the bottom of the
/// tube, or on one of the two circles `z = 0` cuts it in, away from the
/// corners as well as at them.
#[test]
fn the_wall_holds_the_half_of_the_ring_its_wire_keeps() {
    for choice in every_choice() {
        let half_ring = read_back(choice);
        let view = half_ring.model.solid(half_ring.solid).expect("its own key");
        let asked = |point: [f64; 3]| {
            view.face_holds_the_point(
                half_ring.wall,
                PredicatePoint::from(point),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
        };
        let Some(up) = the_side_of_z_0_the_wall_keeps(choice) else {
            for point in [[4.0, 0.0, 1.0], [4.0, 0.0, -1.0], [5.0, 0.0, 0.0]] {
                let held = asked(point);
                assert!(
                    matches!(
                        held,
                        Err(Error::AFaceMissesPartOfWhereItsSurfacesCross { face })
                            if face == half_ring.wall
                    ),
                    "{choice:?} at {point:?}: {held:?}"
                );
            }
            continue;
        };
        let expected = [
            ([4.0, 0.0, up], OnAFace::Held),
            ([0.0, 4.0, up], OnAFace::Held),
            ([-4.0, 0.0, up], OnAFace::Held),
            ([4.0, 0.0, -up], OnAFace::NotHeld),
            ([0.0, -4.0, -up], OnAFace::NotHeld),
            ([5.0, 0.0, 0.0], OnAFace::OnItsBoundary),
            ([3.0, 0.0, 0.0], OnAFace::OnItsBoundary),
            ([0.0, -5.0, 0.0], OnAFace::OnItsBoundary),
            ([-3.0, 0.0, 0.0], OnAFace::OnItsBoundary),
        ];
        for (point, answer) in expected {
            let held = asked(point);
            assert!(
                matches!(held, Ok(got) if got == answer),
                "{choice:?} at {point:?}: {held:?}"
            );
        }
    }
}

/// Which side of `z = 0` the wall's wire keeps, as the sign of `z` on it, or
/// `None` when the wire keeps one side at one circle and the other side at the
/// other.
///
/// Both circles are stored running the way the ring's own coordinate round its
/// axis runs, so both the same way round. The flat face is a ring-shaped face
/// only when it walks the two opposite ways round, outline one way and hole
/// the other, and the wall walks each the other way from the flat face. The
/// half-ring walks its outer circle `Forward` on the flat face — `Model::validate`
/// refuses every choice that walks it `Reversed` — and uses its wall `Forward`, and then
/// the wall keeps `z ≥ 0`. Turning the walk over, or the use round, turns
/// that over.
fn the_side_of_z_0_the_wall_keeps(choice: Choice) -> Option<f64> {
    if choice.outer_on_the_flat_face == choice.inner_on_the_flat_face {
        return None;
    }
    let walked_as_the_half_ring = choice.outer_on_the_flat_face == Orientation::Forward;
    let used_as_the_half_ring = choice.wall_in_the_shell == Orientation::Forward;
    Some(if walked_as_the_half_ring == used_as_the_half_ring {
        1.0
    } else {
        -1.0
    })
}

/// The one choice that is a solid: the outer circle walked `Forward` on the
/// flat face, the hole `Reversed`, and the wall used `Forward`.
fn is_the_half_ring(choice: Choice) -> bool {
    choice.outer_on_the_flat_face == Orientation::Forward
        && choice.inner_on_the_flat_face == Orientation::Reversed
        && choice.wall_in_the_shell == Orientation::Forward
}

/// The half-ring with its flat face's hole walked `Forward`, the same way round
/// as its outline. It is no solid, and it is the one choice besides the
/// half-ring that `Model::validate` passed before it read the wall's side off
/// its wire, so its answers were pinned as a solid's. The answer pins leave it
/// out.
fn is_the_hole_walked_the_way_its_outline_runs(choice: Choice) -> bool {
    choice.outer_on_the_flat_face == Orientation::Forward
        && choice.inner_on_the_flat_face == Orientation::Forward
        && choice.wall_in_the_shell == Orientation::Forward
}

/// The mirror decodes a model the kernel built and writes it out again to the
/// same bytes, and the half-ring the mirror writes reads back and writes out to
/// the same bytes too. So the arenas below are the model's layout, and the
/// model each test asks is the one written here.
#[test]
fn the_mirror_writes_the_bytes_the_kernel_writes() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    model
        .add_round_bar(3.0, 10.0, &Isometry::identity())
        .expect("a round bar");
    rings::a_quarter_bend(&mut model, RING, TUBE);
    model
        .add_cube(5.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube");
    let bytes = postcard::to_allocvec(&model).expect("a model writes out");
    let mirrored: mirror::Payload = postcard::from_bytes(&bytes).expect("the mirror reads it");
    assert_eq!(
        postcard::to_allocvec(&mirrored).expect("the mirror writes out"),
        bytes
    );

    for choice in every_choice() {
        let bytes = half_ring_bytes(choice);
        let model: Model = postcard::from_bytes(&bytes).expect("the half-ring reads back");
        assert_eq!(
            postcard::to_allocvec(&model).expect("a model writes out"),
            bytes,
            "{choice:?}"
        );
    }
}

/// Which way each closed edge runs on the flat face, and which way the wall is
/// used in the shell. The wall walks each edge the other way from the flat face.
#[derive(Clone, Copy, Debug)]
struct Choice {
    outer_on_the_flat_face: Orientation,
    inner_on_the_flat_face: Orientation,
    wall_in_the_shell: Orientation,
}

fn every_choice() -> impl Iterator<Item = Choice> {
    let either = |forward: bool| {
        if forward {
            Orientation::Forward
        } else {
            Orientation::Reversed
        }
    };
    (0u32..8).map(move |bits| Choice {
        outer_on_the_flat_face: either(bits & 1 == 0),
        inner_on_the_flat_face: either(bits & 2 == 0),
        wall_in_the_shell: either(bits & 4 == 0),
    })
}

struct HalfRing {
    model: Model,
    solid: SolidKey,
    flat: FaceKey,
    wall: FaceKey,
}

fn read_back(choice: Choice) -> HalfRing {
    let model: Model =
        postcard::from_bytes(&half_ring_bytes(choice)).expect("the half-ring reads back");
    let view = model.solids().next().expect("one solid");
    let solid = view.key();
    let face_on = |flat: bool| {
        view.faces()
            .find(|face| face.surface().as_plane().is_some() == flat)
            .expect("a flat face and a wall")
            .key()
    };
    let (flat, wall) = (face_on(true), face_on(false));
    HalfRing {
        model,
        solid,
        flat,
        wall,
    }
}

/// The half-ring as `rings::the_top_half_of_a_ring(model, 4, 1, false)`
/// describes it, written straight into the arenas.
fn half_ring_bytes(choice: Choice) -> Vec<u8> {
    use mirror::{
        Coedge, Data, Edge, Face, FaceUse, Payload, Shell, ShellUse, Solid, Step, Vertex, Wire,
    };
    use slotmap::SlotMap;

    let mut data = Data {
        scale: Scale::millimetres(100.0).expect("a working size"),
        vertices: SlotMap::with_key(),
        edges: SlotMap::with_key(),
        coedges: SlotMap::with_key(),
        wires: SlotMap::with_key(),
        faces: SlotMap::with_key(),
        shells: SlotMap::with_key(),
        solids: SlotMap::with_key(),
    };
    let circle_through = |data: &mut Data, x: f64| {
        let corner = data.vertices.insert(Vertex {
            point: Point::new(x, 0.0, 0.0),
            approximation: None,
        });
        data.edges.insert(Edge {
            start: corner,
            end: corner,
            run: rcad_geom::EdgeRun::RightRoundWhereItsFacesCross,
            approximation: None,
        })
    };
    let outer = circle_through(&mut data, RING + TUBE);
    let inner = circle_through(&mut data, RING - TUBE);
    let one_coedge_wire = |data: &mut Data, edge, orientation| {
        let coedge = data.coedges.insert(Coedge { edge, orientation });
        data.wires.insert(Wire {
            steps: smallvec::smallvec![Step::Along(coedge)],
        })
    };
    let flat_wires = smallvec::smallvec![
        one_coedge_wire(&mut data, outer, choice.outer_on_the_flat_face),
        one_coedge_wire(&mut data, inner, choice.inner_on_the_flat_face),
    ];
    let underneath = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let flat = data.faces.insert(Face {
        surface: Surface::Plane(underneath),
        wires: flat_wires,
        approximation: None,
    });
    let wall_wires = smallvec::smallvec![
        one_coedge_wire(&mut data, outer, choice.outer_on_the_flat_face.reversed()),
        one_coedge_wire(&mut data, inner, choice.inner_on_the_flat_face.reversed()),
    ];
    let wall = data.faces.insert(Face {
        surface: Surface::Torus(
            Torus::new(Axis::z(), RING, TUBE).expect("a tube narrower than its ring"),
        ),
        wires: wall_wires,
        approximation: None,
    });
    let shell = data.shells.insert(Shell {
        faces: vec![
            FaceUse {
                face: flat,
                loc: Isometry::identity(),
                orient: Orientation::Forward,
            },
            FaceUse {
                face: wall,
                loc: Isometry::identity(),
                orient: choice.wall_in_the_shell,
            },
        ],
    });
    data.solids.insert(Solid {
        shells: smallvec::smallvec![ShellUse {
            shell,
            loc: Isometry::identity(),
            orient: Orientation::Forward,
        }],
    });
    postcard::to_allocvec(&Payload {
        version: FORMAT_VERSION,
        data,
    })
    .expect("the mirror writes out")
}

/// The model's arenas as `Model` writes them, field for field.
/// [`the_mirror_writes_the_bytes_the_kernel_writes`] is what holds it to that.
mod mirror {
    use rcad_geom::{EdgeRun, Isometry, Orientation, Point, Scale, Surface};
    use rcad_predicates::Approximation;
    use serde::{Deserialize, Serialize};
    use slotmap::SlotMap;
    use smallvec::SmallVec;

    slotmap::new_key_type! {
        pub struct VertexKey;
        pub struct EdgeKey;
        pub struct CoedgeKey;
        pub struct WireKey;
        pub struct FaceKey;
        pub struct ShellKey;
        pub struct SolidKey;
    }

    #[derive(Serialize, Deserialize)]
    pub struct Payload {
        pub version: u32,
        pub data: Data,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Data {
        pub scale: Scale,
        pub vertices: SlotMap<VertexKey, Vertex>,
        pub edges: SlotMap<EdgeKey, Edge>,
        pub coedges: SlotMap<CoedgeKey, Coedge>,
        pub wires: SlotMap<WireKey, Wire>,
        pub faces: SlotMap<FaceKey, Face>,
        pub shells: SlotMap<ShellKey, Shell>,
        pub solids: SlotMap<SolidKey, Solid>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Vertex {
        pub point: Point,
        pub approximation: Option<Approximation>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Edge {
        pub start: VertexKey,
        pub end: VertexKey,
        pub run: EdgeRun,
        pub approximation: Option<Approximation>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Coedge {
        pub edge: EdgeKey,
        pub orientation: Orientation,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Wire {
        pub steps: SmallVec<[Step; 4]>,
    }

    #[derive(Clone, Copy, Serialize, Deserialize)]
    pub enum Step {
        Along(CoedgeKey),
        Pole(VertexKey),
    }

    #[derive(Serialize, Deserialize)]
    pub struct Face {
        pub surface: Surface,
        pub wires: SmallVec<[WireKey; 1]>,
        pub approximation: Option<Approximation>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Shell {
        pub faces: Vec<FaceUse>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct FaceUse {
        pub face: FaceKey,
        pub loc: Isometry,
        pub orient: Orientation,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Solid {
        pub shells: SmallVec<[ShellUse; 1]>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ShellUse {
        pub shell: ShellKey,
        pub loc: Isometry,
        pub orient: Orientation,
    }
}
