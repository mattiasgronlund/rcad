//! A corner with fewer than three surfaces round it is handed to the fuse as the
//! model stored it, and **the fuse does not ask those surfaces about it**.
//! `docs/decisions.md` §286 says why, and this file is its demonstration.
//!
//! A corner with three or more surfaces round it is solved again from three of
//! them, and every surface is then asked about the solved point, grown by
//! `what_writing_the_corner_down_cost` (`bake.rs`, `derive_a_corner`). A corner
//! with one or two surfaces round it has nothing to solve: two surfaces meet
//! along a curve, and the whole curve satisfies both. So `place_a_corner` reads
//! the stored corner out of the model and hands it on. §253.6 asked whether it
//! should grow that corner and ask too.
//!
//! **Asking would refuse a block `Model::validate` accepts.** The stored corner
//! is the very point `validate` already asked these same surfaces about. The
//! only thing a second ask can add is a different allowance, and the one
//! `derive_a_corner` grows by is the corner's own rounding: `16 × 4ε × size`.
//! `validate` lets a corner of a flat face sit `1e-9 × size` off its plane on
//! top of that, which is a modelling tolerance and not a rounding. So a block
//! whose front edge has one extra corner a nanometre off the edge's line passes
//! `validate`, and asking would prove that corner off its face and refuse the
//! block. §75.6's rule is that a solid `validate` accepts is one the fuse has to
//! be able to bake. Today the fuse cuts it and gives the right answer, which
//! is what the test below pins.

use rcad_geom::{Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftStep, DraftVertex, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey,
};

/// How far the extra corner on the block's top front edge sits off the front
/// face's plane: a nanometre, in a block two millimetres across.
const OFF_THE_FRONT_PLANE: f64 = 1e-9;

/// The block is `[0, 2]³`, so its largest coordinate is two.
const SIDE: f64 = 2.0;

/// `MEASURING_SLACK` in `crates/rcad-topo/src/validate.rs`. It is
/// `pub(crate)`, so it is written out here; the margin below is four orders of
/// magnitude, and no change to it that small would move this test.
const MEASURING_SLACK: f64 = 16.0;

/// The block with one corner a nanometre off the line its two faces cross in:
/// accepted by `validate`, proved off its face by the ask `derive_a_corner`
/// makes, and cut by the fuse to the volume a plain block gives.
///
/// **The first assertion is what stops this being a tautology.** It checks that
/// the corner really is proved off its face once grown by the room
/// `derive_a_corner` would grow it by, so the fuse answering `Ok` below is the
/// fuse not asking, and not the question having become agreeable. With the ask
/// added to `place_a_corner`, the fuse refuses this block with
/// `Invalid::VertexOffItsFacePlane` (§286.3 has the patch and the run).
#[test]
fn a_block_with_a_corner_off_its_edge_by_less_than_validate_allows_is_cut_and_not_refused() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let scale = Scale::millimetres(100.0).expect("100 mm is a working size");
    let mut model = Model::new(scale);

    let bent_corner = Point::new(1.0, OFF_THE_FRONT_PLANE, SIDE);
    let front = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, -1.0, 0.0))
        .expect("an axis is a direction");
    let room = MEASURING_SLACK * Plane::SIGNED_DISTANCE_ROUNDOFF * SIDE;
    let grown = PredicatePoint::Constructed(bent_corner.coordinates().map(|coordinate| {
        Interval::new(coordinate - room, coordinate + room).expect("a finite box")
    }));
    assert!(
        matches!(front.side_of_point(grown), Decision::Pos | Decision::Neg),
        "the corner, grown by the {room:e} that writing it down cost, is proved off the \
         front face's plane"
    );

    let block = a_block_with_its_top_front_edge_bent_through(bent_corner)
        .build(&mut model, &policy, &mut log)
        .expect("validate lets a flat face's corner sit 1e-9 of the size off its plane");

    let corner = the_cube_over_the_far_corner(&mut model, &policy, &mut log);
    let fused = model
        .fuse(&[block], &[corner], Rule::Difference, &policy, &mut log)
        .expect("a corner validate accepts is one the fuse has to be able to bake");

    assert_eq!(
        fused.solids().len(),
        1,
        "cutting a corner off leaves one solid"
    );
    let volume = model
        .solid(fused.solids()[0])
        .expect("the fuse made this key")
        .measure(Deflection::for_scale(scale), &policy, &mut log)
        .expect("a flat solid measures")
        .volume();
    let cut_off = 0.5 * 0.5 * 0.5;
    assert!(
        (volume - (SIDE * SIDE * SIDE - cut_off)).abs() < 10.0 * OFF_THE_FRONT_PLANE,
        "the block less the corner cube, to within what the bent edge moves: {volume}"
    );
    model
        .validate(&policy, &mut log)
        .expect("and the answer is as sound as the block was");
}

/// `[0, 2]³`, with its top front edge running through `bent_corner` on its
/// way from `(0, 0, 2)` to `(2, 0, 2)`, so that one corner has only the top
/// face and the front face round it.
fn a_block_with_its_top_front_edge_bent_through(bent_corner: Point) -> SolidBuilder {
    let mut builder = SolidBuilder::new();
    let at = |builder: &mut SolidBuilder, [x, y, z]: [f64; 3]| {
        builder
            .add_vertex(Point::new(x, y, z))
            .expect("a finite corner")
    };
    let c = [
        at(&mut builder, [0.0, 0.0, 0.0]),
        at(&mut builder, [SIDE, 0.0, 0.0]),
        at(&mut builder, [SIDE, SIDE, 0.0]),
        at(&mut builder, [0.0, SIDE, 0.0]),
        at(&mut builder, [0.0, 0.0, SIDE]),
        at(&mut builder, [SIDE, 0.0, SIDE]),
        at(&mut builder, [SIDE, SIDE, SIDE]),
        at(&mut builder, [0.0, SIDE, SIDE]),
        builder.add_vertex(bent_corner).expect("a finite corner"),
    ];
    let bent = c[8];

    let near = Point::origin();
    let far = Point::new(SIDE, SIDE, SIDE);
    let faces = [
        (vec![c[0], c[3], c[2], c[1]], near, [0.0, 0.0, -1.0]),
        (vec![c[4], bent, c[5], c[6], c[7]], far, [0.0, 0.0, 1.0]),
        (vec![c[0], c[1], c[5], bent, c[4]], near, [0.0, -1.0, 0.0]),
        (vec![c[3], c[7], c[6], c[2]], far, [0.0, 1.0, 0.0]),
        (vec![c[0], c[4], c[7], c[3]], near, [-1.0, 0.0, 0.0]),
        (vec![c[1], c[2], c[6], c[5]], far, [1.0, 0.0, 0.0]),
    ];

    let mut edges: Vec<(DraftVertex, DraftVertex, DraftEdge)> = Vec::new();
    let mut shell = Vec::new();
    for (ring, through, [nx, ny, nz]) in faces {
        let mut steps = Vec::new();
        for (position, &from) in ring.iter().enumerate() {
            let to = ring[(position + 1) % ring.len()];
            let (edge, orientation) = match edges
                .iter()
                .find(|(start, end, _)| *start == to && *end == from)
            {
                Some(&(_, _, edge)) => (edge, Orientation::Reversed),
                None => {
                    let edge = builder
                        .add_edge(from, to, EdgeRun::Straight)
                        .expect("two different corners");
                    edges.push((from, to, edge));
                    (edge, Orientation::Forward)
                }
            };
            steps.push(DraftStep::Along(
                builder
                    .add_coedge(edge, orientation)
                    .expect("an edge of this builder"),
            ));
        }
        let wire = builder.add_wire(&steps).expect("the ring closes");
        let plane = Plane::from_point_and_normal(through, Vector::new(nx, ny, nz))
            .expect("an axis is a direction");
        let face = builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("one wire");
        shell.push((face, Orientation::Forward));
    }
    builder.add_shell(&shell).expect("six faces");
    builder
}

/// The unit cube `[1.5, 2.5]³`, which takes one corner off the block and comes
/// nowhere near the bent edge.
fn the_cube_over_the_far_corner(
    model: &mut Model,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
        .build(policy, log)
        .expect("a unit square");
    model
        .add_extrusion(
            &square,
            1.0,
            &Isometry::translation(Vector::new(1.5, 1.5, 1.5)).expect("a plain translation"),
        )
        .expect("a unit cube")
}
