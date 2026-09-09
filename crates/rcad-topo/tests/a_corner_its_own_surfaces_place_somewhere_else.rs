//! A solid whose stored corner is not where its own three planes cross.
//!
//! `docs/decisions.md` §75.6 left one thing owed: the comparison between a
//! baked corner and the corner the model stored ran only where a triple gave
//! **more than one** place, so a corner placed by a single triple was taken
//! without anyone asking whether the model's own corner was anywhere near it.
//! That is §70.3's ill-conditioned corner, and this file is what reaches it.
//!
//! # Why this solid is not a broken one
//!
//! `Model::validate` lets a corner sit off the plane of its face by
//! `Model::allowed_wobble`, which for a 100 mm model is a tenth of a nanometre
//! before the plane's own roundoff is added. That is a **modelling tolerance**
//! and not a rounding: it is there so a reader can bring in a file whose
//! numbers were written by somebody else's kernel. So a box with one corner
//! nudged `1e-10` off its top face is a solid this kernel accepts, builds,
//! sweeps and — before the change this file is the test for — fuses without a
//! word, silently replacing the stored corner with the place the three planes
//! cross.
//!
//! Nudged along `z` only, so the corner stays *exactly* on the two side planes
//! it also belongs to. One plane disagrees with it and two do not, which is the
//! smallest disagreement that can exist here.
//!
//! # The two boxes here have no width at all
//!
//! Every plane of this box is an axis plane, so the meet of three of them
//! divides by one and comes back as an `f64` triple with a box of no width. The
//! solid carries no placement, so the stored corner is not moved and its box has
//! no width either. Two points, and nothing but the room the selector adds for
//! rounding stands between them — which makes this the file that says how wide
//! that room is, from both sides.
//!
//! The control below is the first of the other three. The same box with the
//! nudge taken out fuses, so the refusal is about the corner and not about a
//! solid described face by face. Then a corner half that room off its top is
//! baked, and a corner twice that room off it is refused.

use rcad_geom::{
    EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector, WHERE_THREE_SURFACES_MEET,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, Rule, SolidBuilder, SolidKey};

/// How far the one corner sits off the top of the box, in millimetres.
///
/// Under `Model::allowed_wobble` for this model — `Scale::millimetres(100.0)`
/// gives `1e-9` against corners of size one — and far above anything `f64`
/// rounds by at that size, which is about `2e-16`. So it is a corner somebody
/// *placed* there, not a corner that rounded there, and the sweep takes it.
const NUDGE: f64 = 1e-10;

#[test]
fn a_corner_a_tenth_of_a_nanometre_off_its_own_top_is_refused() {
    let mut model = Model::new(a_working_size());
    let nudged = a_box_of_side_two(&mut model, NUDGE);
    let overlapping = an_overlapping_cube(&mut model);

    let refused = model.fuse(
        &[nudged],
        &[overlapping],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    let why = match refused {
        Err(why) => why,
        Ok(fused) => panic!(
            "a corner its own three planes place somewhere else is nothing this can bake, and the \
             fuse answered with {} solids",
            fused.solids().len()
        ),
    };
    let Error::Undecidable { records } = &why else {
        panic!("nothing placed the corner, so this is a refusal and not {why:?}");
    };
    assert_eq!(
        records
            .records()
            .last()
            .expect("a refusal names at least one decision")
            .predicate(),
        WHERE_THREE_SURFACES_MEET,
        "no triple placed the corner, which is what nothing having placed it is called"
    );
}

/// How much room the selector adds to the stored corner before comparing it
/// with the place its surfaces cross, at corners of size one.
///
/// `MEASURING_SLACK` (16) times `Plane::SIGNED_DISTANCE_ROUNDOFF` (`4 · ε`),
/// both spelled out here rather than imported: `MEASURING_SLACK` is
/// `pub(crate)` and this test cannot see it, and even where a constant is
/// public a bound a doc comment publishes wants a test that goes red when
/// somebody moves it (§15.6).
const ROOM_FOR_ROUNDING: f64 = 16.0 * 4.0 * f64::EPSILON;

/// A corner half that room off its top is a corner that could have rounded
/// there, so the meet places it and the fuse carries on.
///
/// This is the half of the change that `NUDGE` above cannot show: without the
/// room, two zero-width boxes overlap only when they are equal, and this corner
/// is not equal by 32 ulps.
#[test]
fn a_corner_half_the_rounding_room_off_its_own_top_is_baked() {
    let mut model = Model::new(a_working_size());
    let barely_off = a_box_of_side_two(&mut model, ROOM_FOR_ROUNDING / 2.0);
    let overlapping = an_overlapping_cube(&mut model);

    let fused = model
        .fuse(
            &[barely_off],
            &[overlapping],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a corner no f64 could have written any nearer is a corner the meet places");

    assert_eq!(fused.solids().len(), 1);
}

/// Twice that room is over the line, and the line is what this pins: the room
/// is a rounding and not a licence to move a corner.
#[test]
fn a_corner_twice_the_rounding_room_off_its_own_top_is_refused() {
    let mut model = Model::new(a_working_size());
    let too_far = a_box_of_side_two(&mut model, ROOM_FOR_ROUNDING * 2.0);
    let overlapping = an_overlapping_cube(&mut model);

    let refused = model.fuse(
        &[too_far],
        &[overlapping],
        Rule::Union,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );

    assert!(
        matches!(refused, Err(Error::Undecidable { .. })),
        "twice the room the numbers earn is a corner nothing placed, not {refused:?}"
    );
}

/// The same box with the nudge taken out, which is the control: everything
/// about the description is the same and only the corner has moved back onto
/// the plane its face sits on.
#[test]
fn the_same_box_with_the_corner_back_where_its_planes_cross_fuses() {
    let mut model = Model::new(a_working_size());
    let square = a_box_of_side_two(&mut model, 0.0);
    let overlapping = an_overlapping_cube(&mut model);
    let mut log = ProvenanceLog::new();

    let fused = model
        .fuse(
            &[square],
            &[overlapping],
            Rule::Union,
            &Policy::certified(),
            &mut log,
        )
        .expect("two lattice-aligned boxes are united by proof alone");

    assert_eq!(fused.solids().len(), 1);
    assert!(log.is_empty(), "nothing here had to be approximated");
}

fn a_working_size() -> Scale {
    Scale::millimetres(100.0).expect("100 mm is a working size")
}

/// A cube of side two the fuse has something to do with: `Model::add_cube` puts
/// it between the origin and `(2, 2, 2)`, so it holds the nudged corner at its
/// own centre and the corner is not merely present but engaged.
fn an_overlapping_cube(model: &mut Model) -> SolidKey {
    model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two")
}

/// A box of side two round the origin, described face by face, with the corner
/// at `(1, 1, 1)` lifted `off_the_top` above the plane its top face sits on.
///
/// Every face is given the axis plane it belongs on, so lifting the corner
/// moves the corner and not the planes. That is the whole of the disagreement:
/// §17.1 says a face *is* its plane and a stored corner is a cached sample of
/// where the planes cross, and here the sample is wrong by more than roundoff.
fn a_box_of_side_two(model: &mut Model, off_the_top: f64) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let corners = [
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0 + off_the_top],
        [-1.0, 1.0, 1.0],
    ]
    .map(|[x, y, z]| {
        builder
            .add_vertex(Point::new(x, y, z))
            .expect("a corner of a box")
    });

    let straight = |builder: &mut SolidBuilder, from: usize, to: usize| {
        builder
            .add_edge(corners[from], corners[to], EdgeRun::Straight)
            .expect("a side of a box")
    };
    let edges = [
        straight(&mut builder, 0, 1),
        straight(&mut builder, 1, 2),
        straight(&mut builder, 2, 3),
        straight(&mut builder, 3, 0),
        straight(&mut builder, 4, 5),
        straight(&mut builder, 5, 6),
        straight(&mut builder, 6, 7),
        straight(&mut builder, 7, 4),
        straight(&mut builder, 0, 4),
        straight(&mut builder, 1, 5),
        straight(&mut builder, 2, 6),
        straight(&mut builder, 3, 7),
    ];

    let faces = [
        (
            [0.0, 0.0, -1.0],
            [(3, false), (2, false), (1, false), (0, false)],
        ),
        (
            [0.0, 0.0, 1.0],
            [(4, true), (5, true), (6, true), (7, true)],
        ),
        (
            [0.0, -1.0, 0.0],
            [(0, true), (9, true), (4, false), (8, false)],
        ),
        (
            [0.0, 1.0, 0.0],
            [(11, true), (6, false), (10, false), (2, true)],
        ),
        (
            [-1.0, 0.0, 0.0],
            [(8, true), (7, false), (11, false), (3, true)],
        ),
        (
            [1.0, 0.0, 0.0],
            [(1, true), (10, true), (5, false), (9, false)],
        ),
    ]
    .map(|(normal, walk)| {
        let steps = walk.map(|(edge, forwards)| {
            let orientation = if forwards {
                Orientation::Forward
            } else {
                Orientation::Reversed
            };
            DraftStep::Along(
                builder
                    .add_coedge(edges[edge], orientation)
                    .expect("one use of a side"),
            )
        });
        let wire = builder.add_wire(&steps).expect("a square boundary");
        let plane = Plane::from_point_and_normal(
            Point::new(normal[0], normal[1], normal[2]),
            Vector::new(normal[0], normal[1], normal[2]),
        )
        .expect("an axis direction is a unit normal");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a side of a box")
    });

    builder
        .add_shell(&faces.map(|face| (face, Orientation::Forward)))
        .expect("six faces close a box");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a corner inside the wobble the sweep allows is a corner the sweep allows")
}
