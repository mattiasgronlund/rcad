//! Every way a description of a solid can be wrong, refused by name through the
//! public API.
//!
//! `docs/decisions.md` §42.12 criterion 2 — *no sequence of builder calls
//! panics; every misuse is a named `Err`* — and §42.5's rule that once the
//! builder is public, every arena invariant is caller input.
//!
//! **The division under test.** A refusal comes from one of two places and the
//! tests below say which. What one call can decide from its own arguments is
//! decided in that call, so the error names the entity the caller was working
//! on — which is the whole reason an importer wants a builder and not an arena.
//! What takes the whole solid to see comes back from `Model::validate`, under
//! the name the sweep already had, because a second copy of those checks is a
//! second authority on what a sound solid is (§1.5).
//!
//! **Not vacuous.** Every test here starts from a description that builds — the
//! same unit cube, `a_cube_that_builds` proves it — and breaks exactly one
//! thing. A refusal that came from somewhere else would name something else,
//! and every assertion here names the variant it expects.

use rcad_geom::{EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftFace, DraftStep, DraftVertex, Error, Invalid, Model, SolidBuilder, SolidKey};

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn built(builder: SolidBuilder, model: &mut Model) -> rcad_topo::Result<SolidKey> {
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

#[test]
fn a_cube_that_builds() {
    let mut model = a_model();
    let (mut builder, faces) = ACube::default().described();
    builder.add_shell(&faces).expect("six faces make one shell");

    built(builder, &mut model).expect("the unit cube every test here starts from");

    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("and it is sound");
}

// What one call decides for itself.

#[test]
fn a_corner_that_is_not_a_number_is_refused_by_name() {
    let mut builder = SolidBuilder::new();
    let refused = builder.add_vertex(Point::new(0.0, f64::NAN, 0.0));

    assert!(
        matches!(refused, Err(Error::ADraftPointIsNotFinite { vertex: 0 })),
        "got {refused:?}"
    );
}

#[test]
fn a_straight_edge_from_a_corner_back_to_itself_is_refused_by_name() {
    let mut builder = SolidBuilder::new();
    let corner = builder
        .add_vertex(Point::origin())
        .expect("the origin is finite");

    let refused = builder.add_edge(corner, corner, EdgeRun::Straight);

    assert!(
        matches!(refused, Err(Error::ADraftEdgeHasNoLength { edge: 0 })),
        "got {refused:?}"
    );
}

#[test]
fn a_straight_edge_between_two_corners_at_the_same_point_is_refused_by_name() {
    let mut builder = SolidBuilder::new();
    let one = builder
        .add_vertex(Point::new(1.0, 2.0, 3.0))
        .expect("finite");
    let other = builder
        .add_vertex(Point::new(1.0, 2.0, 3.0))
        .expect("finite");

    let refused = builder.add_edge(one, other, EdgeRun::Straight);

    assert!(
        matches!(refused, Err(Error::ADraftEdgeHasNoLength { edge: 0 })),
        "got {refused:?}"
    );
}

/// A handle is an index, so the same index in a second builder names a
/// different entity. Silently resolving to it is the corruption keys already
/// guard against, and here it is an `Err` rather than a panic because
/// everything a builder is handed is caller input.
#[test]
fn a_handle_from_another_builder_is_refused_rather_than_naming_the_wrong_corner() {
    let mut one = SolidBuilder::new();
    let stranger = one.add_vertex(Point::new(9.0, 9.0, 9.0)).expect("finite");
    let mut other = SolidBuilder::new();
    let mine = other.add_vertex(Point::origin()).expect("finite");

    let refused = other.add_edge(mine, stranger, EdgeRun::Straight);

    assert!(
        matches!(
            refused,
            Err(Error::ADraftHandleCameFromAnotherBuilder { entity: "vertex" })
        ),
        "got {refused:?}"
    );
}

#[test]
fn a_wire_of_two_steps_is_refused_by_name() {
    let mut builder = SolidBuilder::new();
    let corners = [
        builder.add_vertex(Point::origin()).expect("finite"),
        builder
            .add_vertex(Point::new(1.0, 0.0, 0.0))
            .expect("finite"),
    ];
    let edge = builder
        .add_edge(corners[0], corners[1], EdgeRun::Straight)
        .expect("two different corners");
    let there = builder
        .add_coedge(edge, Orientation::Forward)
        .expect("the edge is here");
    let back = builder
        .add_coedge(edge, Orientation::Reversed)
        .expect("the edge is here");

    let refused = builder.add_wire(&[DraftStep::Along(there), DraftStep::Along(back)]);

    assert!(
        matches!(refused, Err(Error::ADraftWireIsTooShort { steps: 2 })),
        "got {refused:?}"
    );
}

#[test]
fn a_wire_whose_steps_do_not_join_up_is_refused_by_name() {
    let mut builder = SolidBuilder::new();
    let corners: Vec<DraftVertex> = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ]
    .into_iter()
    .map(|[x, y, z]| {
        builder
            .add_vertex(Point::new(x, y, z))
            .expect("a corner of a square")
    })
    .collect();
    // Three sides of the square walked in an order that breaks at the second.
    let sides = [(0, 1), (2, 3), (3, 0)].map(|(from, to)| {
        let edge = builder
            .add_edge(corners[from], corners[to], EdgeRun::Straight)
            .expect("a side of a square");
        DraftStep::Along(
            builder
                .add_coedge(edge, Orientation::Forward)
                .expect("the edge is here"),
        )
    });

    let refused = builder.add_wire(&sides);

    assert!(
        matches!(refused, Err(Error::ADraftWireDoesNotClose { position: 1 })),
        "got {refused:?}"
    );
}

/// The same mistake a file makes when two `edge_loop`s share an
/// `oriented_edge`.
#[test]
fn a_coedge_walked_by_two_wires_is_refused_by_name() {
    let mut builder = SolidBuilder::new();
    let steps = a_triangle(&mut builder);
    builder.add_wire(&steps).expect("a triangle closes");

    let refused = builder.add_wire(&steps);

    assert!(
        matches!(
            refused,
            Err(Error::ADraftEntityIsUsedTwice {
                entity: "coedge",
                index: 0
            })
        ),
        "got {refused:?}"
    );
}

#[test]
fn a_wire_trimming_two_faces_is_refused_by_name() {
    let mut builder = SolidBuilder::new();
    let steps = a_triangle(&mut builder);
    let wire = builder.add_wire(&steps).expect("a triangle closes");
    let flat = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal"),
    );
    builder
        .add_face(flat, &[wire])
        .expect("a triangle trims a face");

    let refused = builder.add_face(flat, &[wire]);

    assert!(
        matches!(
            refused,
            Err(Error::ADraftEntityIsUsedTwice {
                entity: "wire",
                index: 0
            })
        ),
        "got {refused:?}"
    );
}

/// Three corners in the `z = 0` plane and the three coedges round them.
fn a_triangle(builder: &mut SolidBuilder) -> Vec<DraftStep> {
    let corners: Vec<DraftVertex> = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
        .into_iter()
        .map(|[x, y]| {
            builder
                .add_vertex(Point::new(x, y, 0.0))
                .expect("a corner of a triangle")
        })
        .collect();
    [(0, 1), (1, 2), (2, 0)]
        .into_iter()
        .map(|(from, to)| {
            let edge = builder
                .add_edge(corners[from], corners[to], EdgeRun::Straight)
                .expect("a side of a triangle");
            DraftStep::Along(
                builder
                    .add_coedge(edge, Orientation::Forward)
                    .expect("the edge is here"),
            )
        })
        .collect()
}

#[test]
fn a_face_with_no_wires_is_refused_by_name() {
    let mut builder = SolidBuilder::new();
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
        .expect("the z axis is a unit normal");

    let refused = builder.add_face(Surface::Plane(flat), &[]);

    assert!(
        matches!(refused, Err(Error::ADraftFaceHasNoWires)),
        "got {refused:?}"
    );
}

#[test]
fn a_shell_with_no_faces_is_refused_by_name() {
    let mut builder = SolidBuilder::new();

    let refused = builder.add_shell(&[]);

    assert!(
        matches!(refused, Err(Error::ADraftShellHasNoFaces)),
        "got {refused:?}"
    );
}

#[test]
fn the_same_face_twice_in_one_shell_is_refused_by_name() {
    let (mut builder, faces) = ACube::default().described();
    let twice: Vec<(DraftFace, Orientation)> = faces
        .iter()
        .copied()
        .chain(core::iter::once(faces[0]))
        .collect();

    let refused = builder.add_shell(&twice);

    assert!(
        matches!(
            refused,
            Err(Error::ADraftEntityIsUsedTwice {
                entity: "face",
                index: 0
            })
        ),
        "got {refused:?}"
    );
}

/// A void inside a solid, and a solid in two separate lumps, are both a second
/// shell, and `SolidBuilder::add_shell` refuses either by name here rather
/// than write and hope for it. A fuse builds one freely:
/// `a_cube_with_a_smaller_one_taken_out_of_the_middle_is_one_solid_with_a_void`
/// in `fusing_solids.rs` cuts a cube out of a block's middle and measures the
/// result, two shells and all. So the refusal is this builder's own — the
/// face-by-face path a caller walks by hand — not a claim that a void goes
/// unbuilt anywhere in this kernel.
///
/// **`Model::validate` does not refuse it**, which is why the refusal has to
/// live here. `the_sweep_does_not_refuse_a_solid_with_two_shells` in
/// `builder.rs` is the demonstration.
#[test]
fn a_second_shell_is_refused_because_a_solid_built_here_has_one() {
    let (mut builder, faces) = ACube::default().described();
    builder.add_shell(&faces).expect("six faces make one shell");

    let refused = builder.add_shell(&faces);

    assert!(
        matches!(refused, Err(Error::ASolidBuiltFaceByFaceHasOneShell)),
        "got {refused:?}"
    );
}

#[test]
fn a_description_with_no_shell_at_all_is_refused_by_name() {
    let mut model = a_model();
    let (builder, _) = ACube::default().described();

    let refused = built(builder, &mut model);

    assert!(
        matches!(refused, Err(Error::ADraftSolidHasNoShell)),
        "got {refused:?}"
    );
    assert_eq!(model.solid_count(), 0);
}

// What takes the whole solid to see.

#[test]
fn a_corner_no_edge_and_no_pole_names_is_refused_by_name() {
    let mut model = a_model();
    let (mut builder, faces) = ACube {
        spare_corner: true,
        ..ACube::default()
    }
    .described();
    builder.add_shell(&faces).expect("six faces make one shell");

    let refused = built(builder, &mut model);

    assert!(
        matches!(
            refused,
            Err(Error::ADraftEntityIsUsedByNothing {
                entity: "vertex",
                index: 8
            })
        ),
        "got {refused:?}"
    );
    assert_eq!(model.solid_count(), 0, "a refused build writes nothing");
}

/// A face described and then left out of the shell.
///
/// The same loop names an unused coedge, wire or edge; a file that describes an
/// `advanced_face` and never puts it in its `closed_shell` is the common way to
/// reach it.
#[test]
fn a_face_the_shell_does_not_hold_is_refused_by_name() {
    let mut model = a_model();
    let (mut builder, faces) = ACube::default().described();
    builder
        .add_shell(&faces[..5])
        .expect("five of the six faces are still a shell");

    let refused = built(builder, &mut model);

    assert!(
        matches!(
            refused,
            Err(Error::ADraftEntityIsUsedByNothing {
                entity: "face",
                index: 5
            })
        ),
        "got {refused:?}"
    );
    assert_eq!(model.solid_count(), 0, "a refused build writes nothing");
}

/// A shell with a face missing is an open skin, and what says so is the four
/// edges left with one coedge each.
///
/// Naming the edge rather than the shell is the point: a file that leaves an
/// `advanced_face` out of its `closed_shell` is a file with a hole in it, and
/// the hole is where the importer has to look.
#[test]
fn a_shell_that_does_not_close_is_refused_by_the_edge_it_is_open_along() {
    let mut model = a_model();
    let (mut builder, faces) = ACube {
        leave_out: Some(1),
        ..ACube::default()
    }
    .described();
    builder.add_shell(&faces).expect("five faces still a shell");

    let refused = built(builder, &mut model);

    assert!(
        matches!(
            refused,
            Err(Error::ADraftEdgeIsNotWalkedByTwoCoedges { count: 1, .. })
        ),
        "got {refused:?}"
    );
    assert_eq!(model.solid_count(), 0, "a refused build writes nothing");
}

/// One loop of the file given the wrong way round.
///
/// The cube's top face walked backwards still closes — that is what makes this
/// worth a test rather than a wire-closure failure — and leaves the four top
/// edges walked reversed by two coedges each, so the two faces along each of
/// them do not agree which side the material is on.
#[test]
fn a_loop_given_the_wrong_way_round_leaves_an_edge_walked_the_same_way_twice() {
    let mut model = a_model();
    let (mut builder, faces) = ACube {
        walk_backwards: Some(1),
        ..ACube::default()
    }
    .described();
    builder.add_shell(&faces).expect("six faces make one shell");

    let refused = built(builder, &mut model);

    assert!(
        matches!(
            refused,
            Err(Error::ADraftEdgeIsWalkedTheSameWayTwice { .. })
        ),
        "got {refused:?}"
    );
    assert_eq!(model.solid_count(), 0, "a refused build writes nothing");
}

/// A wire that is not on the surface its face claims.
///
/// The builder does not look at this and should not: it is the one question
/// that needs the geometry of a whole face at once, and `Model::validate`
/// already asks it of every face of every model. So the refusal comes back
/// under the sweep's own name and this lane wrote no second copy of it.
#[test]
fn a_wire_on_the_wrong_surface_is_refused_by_the_sweep() {
    let mut model = a_model();
    let sloping = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 1.0))
        .expect("a normal with length");
    let (mut builder, faces) = ACube {
        wrong_surface: Some((1, Surface::Plane(sloping))),
        ..ACube::default()
    }
    .described();
    builder.add_shell(&faces).expect("six faces make one shell");

    let refused = built(builder, &mut model);

    assert!(
        matches!(
            refused,
            Err(Error::Invalid(Invalid::VertexOffItsFacePlane { .. }))
        ),
        "got {refused:?}"
    );
    assert_eq!(model.solid_count(), 0, "a refused build writes nothing");
}

/// A refused build leaves the model exactly as it found it, whichever half of
/// the checking refused.
///
/// Counted rather than asserted one at a time: a build that wrote its entities
/// and then failed would leave a model that no later `validate` could explain.
#[test]
fn nothing_a_refused_build_described_reaches_the_model() {
    let mut model = a_model();
    model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");
    let before = (
        model.vertex_count(),
        model.edge_count(),
        model.coedge_count(),
        model.wire_count(),
        model.face_count(),
        model.shell_count(),
        model.solid_count(),
    );

    let mut refusals = 0;
    for broken in [
        ACube {
            spare_corner: true,
            ..ACube::default()
        },
        ACube {
            leave_out: Some(1),
            ..ACube::default()
        },
        ACube {
            walk_backwards: Some(1),
            ..ACube::default()
        },
        ACube {
            wrong_surface: Some((
                1,
                Surface::Plane(
                    Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 1.0))
                        .expect("a normal with length"),
                ),
            )),
            ..ACube::default()
        },
    ] {
        let (mut builder, faces) = broken.described();
        builder.add_shell(&faces).expect("a shell of some faces");
        if built(builder, &mut model).is_err() {
            refusals += 1;
        }
    }

    assert_eq!(refusals, 4, "every one of these was supposed to be refused");
    assert_eq!(
        before,
        (
            model.vertex_count(),
            model.edge_count(),
            model.coedge_count(),
            model.wire_count(),
            model.face_count(),
            model.shell_count(),
            model.solid_count(),
        ),
        "a refused build wrote something into the model"
    );
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("and the cube that was already there is still sound");
}

/// The unit cube as a description, with a knob for the one thing each test
/// breaks.
#[derive(Default)]
struct ACube {
    /// Describe every face but this one, which leaves the skin open.
    leave_out: Option<usize>,
    /// Put this surface on that face instead of its own plane.
    wrong_surface: Option<(usize, Surface)>,
    /// Walk this face's loop the other way round. It still closes.
    walk_backwards: Option<usize>,
    /// Describe a ninth corner no edge and no pole names.
    spare_corner: bool,
}

impl ACube {
    fn described(self) -> (SolidBuilder, Vec<(DraftFace, Orientation)>) {
        let mut builder = SolidBuilder::new();
        let corners: Vec<DraftVertex> = CORNERS
            .iter()
            .map(|&[x, y, z]| {
                builder
                    .add_vertex(Point::new(x, y, z))
                    .expect("a corner of the unit cube")
            })
            .collect();
        let edges: Vec<_> = SIDES
            .iter()
            .map(|&[from, to]| {
                builder
                    .add_edge(corners[from], corners[to], EdgeRun::Straight)
                    .expect("a side of the unit cube")
            })
            .collect();

        let mut faces = Vec::new();
        for (which, (through, normal, walk)) in FACES.into_iter().enumerate() {
            if self.leave_out == Some(which) {
                continue;
            }
            let mut walk = walk.to_vec();
            if self.walk_backwards == Some(which) {
                walk.reverse();
                for step in &mut walk {
                    step.1 = !step.1;
                }
            }
            let steps: Vec<DraftStep> = walk
                .into_iter()
                .map(|(side, forwards)| {
                    let orientation = if forwards {
                        Orientation::Forward
                    } else {
                        Orientation::Reversed
                    };
                    DraftStep::Along(
                        builder
                            .add_coedge(edges[side], orientation)
                            .expect("the side is here"),
                    )
                })
                .collect();
            let wire = builder.add_wire(&steps).expect("a face of a cube closes");
            let surface = match self.wrong_surface {
                Some((face, surface)) if face == which => surface,
                _ => Surface::Plane(
                    Plane::from_point_and_normal(
                        Point::new(through[0], through[1], through[2]),
                        Vector::new(normal[0], normal[1], normal[2]),
                    )
                    .expect("an axis is a unit normal"),
                ),
            };
            faces.push((
                builder
                    .add_face(surface, &[wire])
                    .expect("a face has its wire"),
                Orientation::Forward,
            ));
        }

        if self.spare_corner {
            builder
                .add_vertex(Point::new(5.0, 5.0, 5.0))
                .expect("a corner nothing names is still a finite point");
        }
        (builder, faces)
    }
}

/// The eight corners of the unit cube, `(x, y, z)` bit by bit.
const CORNERS: [[f64; 3]; 8] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [1.0, 1.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
    [1.0, 0.0, 1.0],
    [1.0, 1.0, 1.0],
    [0.0, 1.0, 1.0],
];

/// The twelve sides: the bottom ring, the top ring, then the four uprights.
const SIDES: [[usize; 2]; 12] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0],
    [4, 5],
    [5, 6],
    [6, 7],
    [7, 4],
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
];

/// The six faces: a point on the plane, its outward normal, and the four sides
/// of its loop with `true` for walking a side the way it is stored.
///
/// Every loop runs anticlockwise seen from outside, which is anticlockwise about
/// the outward normal — what `Model::validate` asks of an outer wire.
type AFace = ([f64; 3], [f64; 3], [(usize, bool); 4]);
const FACES: [AFace; 6] = [
    (
        [0.0, 0.0, 0.0],
        [0.0, 0.0, -1.0],
        [(3, false), (2, false), (1, false), (0, false)],
    ),
    (
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [(4, true), (5, true), (6, true), (7, true)],
    ),
    (
        [0.0, 0.0, 0.0],
        [0.0, -1.0, 0.0],
        [(0, true), (9, true), (4, false), (8, false)],
    ),
    (
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [(1, true), (10, true), (5, false), (9, false)],
    ),
    (
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [(2, true), (11, true), (6, false), (10, false)],
    ),
    (
        [0.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [(3, true), (8, true), (7, false), (11, false)],
    ),
];
