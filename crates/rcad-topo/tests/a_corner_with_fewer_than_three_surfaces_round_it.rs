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
//! is what the first test below pins.
//!
//! **Give the same nanometre to a corner with three planes round it and the
//! fuse does refuse.** Such a corner is solved from its three planes, and the
//! stored corner is only used to pick which crossing it is. That comparison
//! grows the stored corner by the rounding and nothing more, so a crossing a
//! nanometre away is not picked, nothing places the corner, and the fuse gives
//! up `Undecidable`. That is §136.4's gap — `validate` accepts a solid the fuse
//! cannot bake — and the second test pins it as it is today (§136.8).

use rcad_geom::{
    Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
    WHERE_THREE_SURFACES_MEET,
};
use rcad_predicates::{Decision, Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftStep, DraftVertex, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey,
};

/// How far the extra corner on the block's top front edge sits off the front
/// face's plane: a nanometre, in a block two millimetres across.
const OFF_THE_FRONT_PLANE: f64 = 1e-9;

/// The block is `[0, 2]³`, so its largest coordinate is two.
const SIDE: f64 = 2.0;

/// The far end of `the_cube_over_the_far_corner`, and so the largest
/// coordinate in the model once that cube is in it.
const THE_CUBE_REACHES: f64 = 2.5;

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

/// The block with its top front left corner moved a nanometre off the front
/// face's plane, and still on the top and left planes: accepted by `validate`,
/// and **refused by the fuse as `Undecidable`**, because nothing places that
/// corner.
///
/// The corner has three planes round it, so the fuse solves it from them. They
/// cross at `(0, 0, 2)`, a nanometre from the stored corner, and the fuse picks
/// that crossing only if the two lie within `what_writing_the_corner_down_cost`
/// of each other, which is the rounding and not the modelling tolerance
/// `validate` also grants. So no crossing is picked, and `derive_a_corner` gives
/// up with a record naming [`WHERE_THREE_SURFACES_MEET`].
///
/// **This is §136.4's gap, pinned as it is, and the test changes the day the
/// gap closes**: the fuse then has to cut this block the way it cuts the
/// control, and the `Err` expected below becomes that `Ok`.
///
/// The fix §136.4 names — sizing the room by the model's largest coordinate
/// instead of the corner's — does not reach this case, and the third
/// assertion is why: even at the largest coordinate the model has when it is
/// fused, the rounding is under a ten-thousandth of the miss.
///
/// **The control is what stops the refusal being blamed on something else.**
/// The same block with that corner off its plane by exactly the room the fuse
/// grants, fused with the same cube, is cut. So the block and the cube are
/// not what refuses, and neither is the corner being off its plane at all:
/// the fuse takes a miss up to its room, and a nanometre is past it.
#[test]
fn a_block_with_a_corner_off_one_of_its_three_planes_by_less_than_validate_allows_is_refused() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let scale = Scale::millimetres(100.0).expect("100 mm is a working size");

    let rounding_at = |size: f64| MEASURING_SLACK * Plane::SIGNED_DISTANCE_ROUNDOFF * size;
    let validate_allows = scale
        .default_linear_tolerance()
        .min(Scale::RELATIVE_LINEAR_TOLERANCE * SIDE)
        + rounding_at(SIDE);
    let the_fuse_allows = rounding_at(SIDE);
    let at_the_models_size = rounding_at(THE_CUBE_REACHES);
    eprintln!(
        "the corner is {OFF_THE_FRONT_PLANE:e} off its front plane; validate allows \
         {validate_allows:e}; the fuse grows the stored corner by {the_fuse_allows:e}, and \
         would by {at_the_models_size:e} at the model's largest coordinate"
    );
    assert!(
        OFF_THE_FRONT_PLANE < validate_allows,
        "the miss is inside what validate allows"
    );
    assert!(
        OFF_THE_FRONT_PLANE > the_fuse_allows,
        "the miss is outside the room the fuse grows the stored corner by"
    );
    assert!(
        OFF_THE_FRONT_PLANE > 1e4 * at_the_models_size,
        "and outside it by more than four orders of magnitude at the model's own size"
    );

    let off_by_the_fuses_room = Point::new(0.0, the_fuse_allows, SIDE);
    let (mut model, block, cube) =
        a_block_with_one_corner_at_and_the_cube(off_by_the_fuses_room, scale, &policy, &mut log);
    model
        .fuse(&[block], &[cube], Rule::Difference, &policy, &mut log)
        .expect("the control: a corner off its plane by the fuse's own room is placed");

    let off_the_front_plane = Point::new(0.0, OFF_THE_FRONT_PLANE, SIDE);
    let (mut model, block, cube) =
        a_block_with_one_corner_at_and_the_cube(off_the_front_plane, scale, &policy, &mut log);
    model
        .validate(&policy, &mut log)
        .expect("validate accepts the model the fuse is handed");
    let refused = model.fuse(&[block], &[cube], Rule::Difference, &policy, &mut log);
    let Err(Error::Undecidable { records }) = refused else {
        panic!(
            "§136.4's gap: nothing places a corner a nanometre off one of its three planes, \
             so the fuse refuses the block validate accepted, and it was {refused:?}"
        );
    };
    assert_eq!(
        records.records().last().map(|record| record.predicate()),
        Some(WHERE_THREE_SURFACES_MEET),
        "the refusal is the corner nothing placed, and not a surface proving it off its face"
    );
}

/// A fresh model holding `[0, 2]³` with its top front left corner at
/// `corner`, built face by face so the corner may sit off its planes, and the
/// cube over the block's far corner.
fn a_block_with_one_corner_at_and_the_cube(
    corner: Point,
    scale: Scale,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(scale);
    let mut corners = the_corners_of_the_block();
    corners[4] = corner;
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
    let cube = the_cube_over_the_far_corner(&mut model, policy, log);
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
