//! A corner with fewer than three surfaces round it is handed to the fuse as the
//! model stored it, and **the fuse does not ask those surfaces about it**.
//! `docs/decisions.md` §286 says why, and this file is its demonstration.
//!
//! A corner with three or more surfaces round it is solved again from three of
//! them, and every surface is then asked about the solved point (`bake.rs`,
//! `derive_a_corner`). A corner with one or two surfaces round it has nothing
//! to solve: two surfaces meet along a curve, and the whole curve satisfies
//! both. So `place_a_corner` reads the stored corner out of the model and hands
//! it on. §253.6 asked whether it should grow that corner and ask too.
//!
//! **Asking with the rounding alone would refuse a block `Model::validate`
//! accepts.** The stored corner is the very point `validate` already asked
//! these same surfaces about. The only thing a second ask can add is a
//! different allowance, and the rounding, `16 × 4ε × size`, is narrower than
//! what `validate` grants: it lets a corner of a flat face sit `1e-9 × size`
//! off its plane on top of that, a modelling tolerance and not a rounding. So
//! a block whose front edge has one extra corner a nanometre off the edge's
//! line passes `validate`, and asking with the rounding would prove that
//! corner off its face and refuse the block. §75.6's rule is that a solid
//! `validate` accepts is one the fuse has to be able to bake. The fuse cuts it
//! and gives the right answer, which is what the first test below pins.
//!
//! **Give the same nanometre to a corner with three planes round it and the
//! fuse cuts that block too.** Such a corner is solved from its three planes,
//! and the stored corner is only used to pick which crossing it is. That
//! comparison grows the stored corner by the room `Model::validate_solid`
//! grants a corner of the block, so the crossing a nanometre away is picked.
//! Until §368 it grew the corner by the rounding alone, and the fuse refused
//! the block `Undecidable` (§136.8). The second test pins the block cut, and
//! the third that the room is measured at the block's size and not the
//! corner's.

use rcad_geom::{Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftStep, DraftVertex, Fused, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey,
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

/// Where [`the_corners_of_the_block`] keeps the corner at the origin.
const BOTTOM_FRONT_LEFT: usize = 0;

/// Where [`the_corners_of_the_block`] keeps the corner at `(0, 0, 2)`.
const TOP_FRONT_LEFT: usize = 4;

/// The block with one corner a nanometre off the line its two faces cross in:
/// accepted by `validate`, proved off its face once grown by the rounding
/// alone, and cut by the fuse to the volume a plain block gives.
///
/// **The first assertion is what stops this being a tautology.** It checks that
/// the corner really is proved off its face once grown by the rounding, so the
/// fuse answering `Ok` below is the fuse not asking with that room, and not the
/// question having become agreeable. With that ask added to `place_a_corner`,
/// the fuse refuses this block with `Invalid::VertexOffItsFacePlane` (§286.3
/// has the patch and the run).
#[test]
fn a_block_with_a_corner_off_its_edge_by_less_than_validate_allows_is_cut_and_not_refused() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let scale = Scale::millimetres(100.0).expect("100 mm is a working size");
    let mut model = Model::new(scale);

    let bent_corner = Point::new(1.0, OFF_THE_FRONT_PLANE, SIDE);
    let front = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, -1.0, 0.0))
        .expect("an axis is a direction");
    let room = rounding_at(SIDE);
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

    let corner = a_unit_cube_from(Vector::new(1.5, 1.5, 1.5), &mut model, &policy, &mut log);
    let fused = model
        .fuse(&[block], &[corner], Rule::Difference, &policy, &mut log)
        .expect("a corner validate accepts is one the fuse has to be able to bake");

    is_the_block_less_one_corner_cube(&model, &fused, scale, &policy, &mut log);
}

/// The block with its top front left corner moved a nanometre off the front
/// face's plane, and still on the top and left planes: accepted by `validate`,
/// and **cut by the fuse** (§136.8, §368).
///
/// The corner has three planes round it, so the fuse solves it from them. They
/// cross at `(0, 0, 2)`, a nanometre from the stored corner, and the fuse picks
/// that crossing because the two lie within the room `Model::validate_solid`
/// grants a corner of the block off a plane. Until §368 that room was the
/// rounding alone, `2.84e-14`, no crossing was picked, and the fuse refused
/// the block `Undecidable`: a solid `validate` accepts that the fuse could not
/// bake, at ordinary size.
#[test]
fn a_block_with_a_corner_off_one_of_its_three_planes_by_less_than_validate_allows_is_cut() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let scale = Scale::millimetres(100.0).expect("100 mm is a working size");

    let validate_allows = what_validate_allows_off_a_plane(scale, SIDE);
    eprintln!(
        "the corner is {OFF_THE_FRONT_PLANE:e} off its front plane; validate allows \
         {validate_allows:e}; the rounding alone is {:e}",
        rounding_at(SIDE)
    );
    assert!(
        rounding_at(SIDE) < OFF_THE_FRONT_PLANE && OFF_THE_FRONT_PLANE < validate_allows,
        "the miss is past the rounding alone and inside what validate allows"
    );

    let off_the_front_plane = Point::new(0.0, OFF_THE_FRONT_PLANE, SIDE);
    let (mut model, block, cube) = a_block_with_one_corner_moved_and_a_cube(
        TOP_FRONT_LEFT,
        off_the_front_plane,
        Vector::new(-0.5, -0.5, -0.5),
        scale,
        &policy,
        &mut log,
    );
    model
        .validate(&policy, &mut log)
        .expect("validate accepts the model the fuse is handed");
    let fused = model
        .fuse(&[block], &[cube], Rule::Difference, &policy, &mut log)
        .expect("a corner validate accepts is one the fuse has to be able to bake");

    is_the_block_less_one_corner_cube(&model, &fused, scale, &policy, &mut log);
}

/// A corner at the origin a nanometre and a half off one of its three planes
/// is cut too, and this is the test that says **whose size the room is
/// measured at**.
///
/// `Model::validate_solid` measures it at the block's largest coordinate, two.
/// The corner's own size floored at one, which is what the rounding half of
/// the fuse's room is measured at, grants the modelling tolerance of a solid
/// of size one, and the miss is past that. So a fuse that sized the whole room
/// off the corner refuses this block, and one that asks `validate_solid` cuts
/// it. The cube comes off the far corner, nowhere near this one, so nothing
/// but baking the corner at the origin is asked of it.
#[test]
fn a_corner_at_the_origin_gets_the_room_its_solid_gets_and_not_the_room_its_own_size_would() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let scale = Scale::millimetres(100.0).expect("100 mm is a working size");

    let off = 1.5e-9;
    let at_the_corners_own_size = what_validate_allows_off_a_plane(scale, 1.0);
    let at_the_blocks_size = what_validate_allows_off_a_plane(scale, SIDE);
    assert!(
        at_the_corners_own_size < off && off < at_the_blocks_size,
        "the miss {off:e} is past the {at_the_corners_own_size:e} a solid of size one gets and \
         inside the {at_the_blocks_size:e} this block gets"
    );

    let (mut model, block, cube) = a_block_with_one_corner_moved_and_a_cube(
        BOTTOM_FRONT_LEFT,
        Point::new(0.0, off, 0.0),
        Vector::new(1.5, 1.5, 1.5),
        scale,
        &policy,
        &mut log,
    );
    let fused = model
        .fuse(&[block], &[cube], Rule::Difference, &policy, &mut log)
        .expect("the room is the one validate_solid grants the block");

    is_the_block_less_one_corner_cube(&model, &fused, scale, &policy, &mut log);
}

/// How far `Model::validate_solid` lets a corner of a solid of size `size`
/// sit off a plane: `Model::allowed_wobble`, spelled out because it is
/// `pub(crate)`.
fn what_validate_allows_off_a_plane(scale: Scale, size: f64) -> f64 {
    scale
        .default_linear_tolerance()
        .min(Scale::RELATIVE_LINEAR_TOLERANCE * size)
        + rounding_at(size)
}

/// What writing a plane down costs at `size`, with the sweep's slack.
fn rounding_at(size: f64) -> f64 {
    MEASURING_SLACK * Plane::SIGNED_DISTANCE_ROUNDOFF * size
}

/// The fuse gave one solid, `[0, 2]³` less a unit cube's eighth off one
/// corner, to within what the moved corner shifts, and it validates.
fn is_the_block_less_one_corner_cube(
    model: &Model,
    fused: &Fused,
    scale: Scale,
    policy: &Policy,
    log: &mut ProvenanceLog,
) {
    assert_eq!(
        fused.solids().len(),
        1,
        "cutting a corner off leaves one solid"
    );
    let volume = model
        .solid(fused.solids()[0])
        .expect("the fuse made this key")
        .measure(Deflection::for_scale(scale), policy, log)
        .expect("a flat solid measures")
        .volume();
    let cut_off = 0.5 * 0.5 * 0.5;
    assert!(
        (volume - (SIDE * SIDE * SIDE - cut_off)).abs() < 10.0 * OFF_THE_FRONT_PLANE,
        "the block less the corner cube, to within what the moved corner shifts: {volume}"
    );
    model
        .validate(policy, log)
        .expect("and the answer is as sound as the block was");
}

/// A fresh model holding `[0, 2]³` with its corner number `which` at `corner`,
/// built face by face so the corner may sit off its planes, and the unit cube
/// with its lowest corner at `cube_from`.
fn a_block_with_one_corner_moved_and_a_cube(
    which: usize,
    corner: Point,
    cube_from: Vector,
    scale: Scale,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(scale);
    let mut corners = the_corners_of_the_block();
    corners[which] = corner;
    let block = a_block_on_the_planes_of_the_cube(
        &corners,
        [
            &[0, 3, 2, 1],
            &[4, 5, 6, 7],
            &[0, 1, 5, 4],
            &[3, 7, 6, 2],
            &[0, 4, 7, 3],
            &[1, 2, 6, 5],
        ],
    )
    .build(&mut model, policy, log)
    .expect("validate lets a flat face's corner sit 1e-9 of the size off its plane");
    let cube = a_unit_cube_from(cube_from, &mut model, policy, log);
    (model, block, cube)
}

/// `[0, 2]³`, with its top front edge running through `bent_corner` on its
/// way from `(0, 0, 2)` to `(2, 0, 2)`, so that one corner has only the top
/// face and the front face round it.
fn a_block_with_its_top_front_edge_bent_through(bent_corner: Point) -> SolidBuilder {
    let mut corners = the_corners_of_the_block();
    corners.push(bent_corner);
    a_block_on_the_planes_of_the_cube(
        &corners,
        [
            &[0, 3, 2, 1],
            &[4, 8, 5, 6, 7],
            &[0, 1, 5, 8, 4],
            &[3, 7, 6, 2],
            &[0, 4, 7, 3],
            &[1, 2, 6, 5],
        ],
    )
}

/// The eight corners of `[0, 2]³`: the bottom four anticlockwise from the
/// origin seen from above, then the top four the same way.
fn the_corners_of_the_block() -> Vec<Point> {
    [
        [0.0, 0.0, 0.0],
        [SIDE, 0.0, 0.0],
        [SIDE, SIDE, 0.0],
        [0.0, SIDE, 0.0],
        [0.0, 0.0, SIDE],
        [SIDE, 0.0, SIDE],
        [SIDE, SIDE, SIDE],
        [0.0, SIDE, SIDE],
    ]
    .map(|[x, y, z]| Point::new(x, y, z))
    .to_vec()
}

/// A solid whose six faces lie on the planes of `[0, 2]³` — bottom, top,
/// front, back, left, right, in that order — each bounded by the ring of
/// `corners` given for it. The corners are wherever they were put, which is
/// what lets one of them sit off its planes.
fn a_block_on_the_planes_of_the_cube(corners: &[Point], rings: [&[usize]; 6]) -> SolidBuilder {
    let mut builder = SolidBuilder::new();
    let c: Vec<DraftVertex> = corners
        .iter()
        .map(|&corner| builder.add_vertex(corner).expect("a finite corner"))
        .collect();

    let near = Point::origin();
    let far = Point::new(SIDE, SIDE, SIDE);
    let planes = [
        (near, [0.0, 0.0, -1.0]),
        (far, [0.0, 0.0, 1.0]),
        (near, [0.0, -1.0, 0.0]),
        (far, [0.0, 1.0, 0.0]),
        (near, [-1.0, 0.0, 0.0]),
        (far, [1.0, 0.0, 0.0]),
    ];

    let mut edges: Vec<(DraftVertex, DraftVertex, DraftEdge)> = Vec::new();
    let mut shell = Vec::new();
    for (ring, (through, [nx, ny, nz])) in rings.into_iter().zip(planes) {
        let mut steps = Vec::new();
        for (position, &corner) in ring.iter().enumerate() {
            let from = c[corner];
            let to = c[ring[(position + 1) % ring.len()]];
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

/// The unit cube with its lowest corner at `lowest`. From `(1.5, 1.5, 1.5)` it
/// takes the block's far corner off and comes nowhere near the corners the
/// tests move; from `(-0.5, -0.5, -0.5)` it takes the corner at the origin off
/// and leaves the model's largest coordinate at the block's two.
fn a_unit_cube_from(
    lowest: Vector,
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
            &Isometry::translation(lowest).expect("a plain translation"),
        )
        .expect("a unit cube")
}
