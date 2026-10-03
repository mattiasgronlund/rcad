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
//! On `c564dec5` every one of the six was refused
//! `AFaceTrimmedByACurveThisCannotFollow`, and both points given to
//! `classify_point` were refused `CannotFollowAFacesTrimming` naming the flat
//! face. `intersect` names the two circles a plane square to a ring's axis cuts
//! by the two sides of the pipe of the ring's own radius, and the lookup the
//! flat face asked took only a flat half-space.

use rcad_geom::{Axis, Isometry, Orientation, Plane, Point, Scale, Surface, Torus, Vector};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, FORMAT_VERSION, FaceKey, Model, OnAFace, SolidKey};

mod rings;

const RING: f64 = 4.0;
const TUBE: f64 = 1.0;

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
    for choice in every_choice() {
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

/// The middle of the hole is outside the half-ring. A point on the flat face
/// is on its skin, and that one is still refused — but by the wall, which
/// refuses every point asked of it, and no longer by the flat face. The wall is
/// a ring face trimmed by one plane square to the ring's axis along both of its
/// circles.
#[test]
fn classify_point_finds_the_hole_outside_and_is_stopped_by_the_wall_on_the_face() {
    for choice in every_choice() {
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
        assert_eq!(
            on_the_face,
            Containment::CannotFollowAFacesTrimming {
                face: half_ring.wall
            },
            "{choice:?}"
        );
    }
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
