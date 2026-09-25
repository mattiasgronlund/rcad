//! Arbitrary bytes into `rcad_step::write`, which must never panic, and every
//! sound model it writes must read back as the same model.
//!
//! Everything the target *checks* is in `crates/rcad-step/tests/fuzzing/writer.rs`,
//! and this target **reaches across to that file** rather than copying it — so
//! the property `cargo fuzz` explores and the property the ordinary test suite
//! replays are the same lines of code. `docs/decisions.md` §29.9 records which
//! of the two runs in CI and which does not, and §95 records this target
//! joining the arrangement.
//!
//! # Every mutation is still a model
//!
//! The input is `postcard` bytes of a [`Model`], the route a stranger's saved
//! file takes, and libFuzzer's own byte mutations almost never land on bytes
//! that still decode: 19 of the 1183 seeds checked in on 2026-09-25 did, and 16
//! of those were empty. So [`still_a_model`] replaces libFuzzer's mutator. It
//! decodes the input and makes one change a model can take — a byte changed
//! where the result still decodes, one more solid, one solid moved or turned,
//! or a fresh model — and encodes the result. In two 60-second runs on
//! 2026-09-25, 11 145 of 11 179 and 12 272 of 12 309 mutated inputs decoded
//! (`docs/decisions.md` §241).
//!
//! The shapes are the ones the writer has to spell differently: a cube, a round
//! bar, a ball, a frustum, a pointed cone and a quarter bend of pipe, which is
//! the torus a mechanical file holds. No builder in the kernel makes the last
//! three, so they are described face by face here, each with its corners exact
//! on its curved surface — a power-of-two slope on a cone, the inner equator on
//! a torus — because `Model::validate` asks that every corner of a curved face
//! be certified on it.

#![no_main]

use libfuzzer_sys::{fuzz_mutator, fuzz_target, fuzzer_mutate};

// `#[path]` and not `include!`, because the shared file opens with a module doc
// comment and an inner attribute, and neither of those can be pasted into the
// middle of another file.
#[path = "../../crates/rcad-step/tests/fuzzing/writer.rs"]
mod writer;

use rcad_fuzz_timing::timed;
use rcad_geom::{
    Axis, Cone, Cylinder, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale,
    Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey};
use writer::one_model_decoded_and_written;

fuzz_target!(|bytes: &[u8]| {
    timed(env!("CARGO_BIN_NAME"), bytes, || {
        let _ = one_model_decoded_and_written(bytes);
    });
});

fuzz_mutator!(|data: &mut [u8], size: usize, max_size: usize, seed: u32| {
    still_a_model(data, size, max_size, seed)
});

/// Change `data[..size]` into the `postcard` bytes of a model, and return how
/// many bytes that took.
///
/// An input that does not decode becomes a fresh model, so the 1164 seeds that
/// never reached the writer each start a model rather than staying bytes.
///
/// The one input this leaves as it came is one where even a fresh one-solid
/// model does not fit in `max_size`. With no `-max_len`, libFuzzer's limit is
/// 4096 bytes — its own floor, since the corpus's longest seed is 4064 — and
/// the longest one-solid model drawn here, the quarter bend, is 681.
fn still_a_model(data: &mut [u8], size: usize, max_size: usize, seed: u32) -> usize {
    let mut draw = Draw::new(seed);
    let mut model = match postcard::from_bytes::<Model>(&data[..size]) {
        Ok(model) => model,
        Err(_) => a_fresh_model(&mut draw),
    };
    match draw.below(8) {
        0..=2 => {
            if let Some(nudged) = one_byte_changed_and_still_a_model(data, size, max_size) {
                return nudged;
            }
            one_more_solid(&mut model, &mut draw);
        }
        3 | 4 => one_more_solid(&mut model, &mut draw),
        5 | 6 => one_solid_moved(&mut model, &mut draw),
        _ => model = a_fresh_model(&mut draw),
    }
    written_over(data, max_size, &model)
        .or_else(|| written_over(data, max_size, &a_fresh_model(&mut draw)))
        .unwrap_or(size)
}

/// Encode `model` over the front of `data`, if it fits in `max_size`.
fn written_over(data: &mut [u8], max_size: usize, model: &Model) -> Option<usize> {
    let bytes = postcard::to_allocvec(model).expect("a model always encodes");
    if bytes.len() > max_size {
        return None;
    }
    data[..bytes.len()].copy_from_slice(&bytes);
    Some(bytes.len())
}

/// libFuzzer's own mutation, kept only if the bytes still decode.
///
/// This is the half that reaches the writer with a model `Model::validate`
/// would refuse — a corner nudged off its face, a plane turned a little — which
/// no builder makes and which the writer owes an `Err` for rather than a crash.
fn one_byte_changed_and_still_a_model(
    data: &mut [u8],
    size: usize,
    max_size: usize,
) -> Option<usize> {
    let original = data[..size].to_vec();
    for _ in 0..4 {
        let nudged = fuzzer_mutate(data, size, max_size);
        if postcard::from_bytes::<Model>(&data[..nudged]).is_ok() {
            return Some(nudged);
        }
        data[..size].copy_from_slice(&original);
    }
    None
}

/// A model of the size every shape here is drawn at, holding one of them.
fn a_fresh_model(draw: &mut Draw) -> Model {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let solid = a_drawn_solid(&mut model, draw)
        .expect("every shape drawn here is sound in a model of this size");
    maybe_moved(&mut model, solid, draw);
    model
}

/// Add one drawn shape to `model`, or start again if the model will not take
/// it.
///
/// A model that came through the door can carry any scale at all, and a shape
/// a few units across is refused outright in a model whose working size is
/// `1e-300`. That refusal is the builder's answer to a caller, not a defect,
/// so the mutation moves on rather than keeping a model it could not change.
fn one_more_solid(model: &mut Model, draw: &mut Draw) {
    match a_drawn_solid(model, draw) {
        Ok(solid) => maybe_moved(model, solid, draw),
        Err(_) => *model = a_fresh_model(draw),
    }
}

/// Move or turn one of `model`'s solids.
///
/// Only a sound model is moved. `Model::place_solid` checks the solid it moved
/// and panics if the solid is broken, which is right for a solid the kernel
/// built and wrong to ask of one a changed byte broke on the way in.
fn one_solid_moved(model: &mut Model, draw: &mut Draw) {
    let sound = model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .is_ok();
    let count = model.solid_count() as u64;
    if !sound || count == 0 {
        one_more_solid(model, draw);
        return;
    }
    let which = usize::try_from(draw.below(count)).expect("below a usize count");
    let solid = model
        .solids()
        .nth(which)
        .expect("fewer than the model's own count")
        .key();
    model
        .place_solid(solid, &a_movement(draw))
        .expect("the key came from this model");
}

/// Leave the new solid where it was built a third of the time, move it a
/// third, and move and turn it the last third.
fn maybe_moved(model: &mut Model, solid: SolidKey, draw: &mut Draw) {
    if draw.below(3) == 0 {
        return;
    }
    model
        .place_solid(solid, &a_movement(draw))
        .expect("the key came from this model");
}

/// A move by whole units, and half the time a turn about one of the three axes
/// or a slanted one — by a multiple of an eighth of a turn, which is not exact
/// in `f64`, or by any angle at all.
fn a_movement(draw: &mut Draw) -> Isometry {
    let offset = Vector::new(
        draw.whole(-16, 16),
        draw.whole(-16, 16),
        draw.whole(-16, 16),
    );
    let moved = Isometry::translation(offset).expect("a finite offset");
    if draw.below(2) == 0 {
        return moved;
    }
    let axis = match draw.below(4) {
        0 => Axis::x(),
        1 => Axis::y(),
        2 => Axis::z(),
        _ => Axis::new(
            Point::origin(),
            Vector::new(draw.whole(1, 4), draw.whole(-4, 4), draw.whole(-4, 4)),
        )
        .expect("a direction whose first component is at least one"),
    };
    let angle = if draw.below(2) == 0 {
        draw.whole(1, 7) * core::f64::consts::FRAC_PI_4
    } else {
        draw.fraction() * core::f64::consts::TAU
    };
    let turned = Isometry::rotation(&axis, angle).expect("a finite angle about a unit axis");
    moved.compose(&turned)
}

/// One of the six shapes, with whole-number sizes and a power-of-two slope.
fn a_drawn_solid(model: &mut Model, draw: &mut Draw) -> rcad_topo::Result<SolidKey> {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    match draw.below(6) {
        0 => model.add_cube(draw.whole(1, 8), &policy, &mut log),
        1 => model.add_round_bar(draw.whole(1, 8), draw.whole(1, 16), &Isometry::identity()),
        2 => model.add_ball(draw.whole(1, 8), &Isometry::identity()),
        3 => {
            let near = draw.whole(1, 8);
            let far = near + draw.whole(1, 8);
            a_frustum(model, a_slope(draw), near, far)
        }
        4 => a_pointed_cone(model, a_slope(draw), draw.whole(1, 8)),
        _ => {
            let tube = draw.whole(1, 4);
            a_quarter_bend(model, tube + draw.whole(1, 8), tube)
        }
    }
}

/// A power of two, so a cone's corner at `(slope · height, 0, height)` is on
/// the cone exactly.
fn a_slope(draw: &mut Draw) -> f64 {
    [0.25, 0.5, 1.0, 2.0][usize::try_from(draw.below(4)).expect("below four")]
}

/// A frustum about `+z` with its apex at the origin and its caps at `near` and
/// `far` up the axis: two cap circles, a seam up the ruling over `+x`, three
/// faces — a round bar's structure on a cone.
fn a_frustum(model: &mut Model, slope: f64, near: f64, far: f64) -> rcad_topo::Result<SolidKey> {
    let skin = Cone::new(Axis::z(), slope).expect("a positive slope");
    let mut builder = SolidBuilder::new();
    let bottom_corner = builder
        .add_vertex(Point::new(slope * near, 0.0, near))
        .expect("a finite point");
    let top_corner = builder
        .add_vertex(Point::new(slope * far, 0.0, far))
        .expect("a finite point");
    let seam = builder
        .add_edge(bottom_corner, top_corner, EdgeRun::Straight)
        .expect("a ruling of a cone is straight");
    let bottom_circle = builder
        .add_edge(
            bottom_corner,
            bottom_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a cap plane cuts the cone in one closed curve");
    let top_circle = builder
        .add_edge(top_corner, top_corner, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("a cap plane cuts the cone in one closed curve");

    let bottom = a_cap(
        &mut builder,
        Point::new(0.0, 0.0, near),
        Vector::new(0.0, 0.0, -1.0),
        bottom_circle,
        Orientation::Reversed,
    );
    let top = a_cap(
        &mut builder,
        Point::new(0.0, 0.0, far),
        Vector::new(0.0, 0.0, 1.0),
        top_circle,
        Orientation::Forward,
    );
    let side = a_wall(&mut builder, Surface::Cone(skin), bottom_circle, seam, top_circle);
    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("three faces close");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// A cone that ends in a point: a base cap at `far` up the axis and one face on
/// the cone reaching down to the apex. Two faces, one edge, one vertex, and the
/// apex is not a vertex at all — it is where the cone's own chart runs out.
fn a_pointed_cone(model: &mut Model, slope: f64, far: f64) -> rcad_topo::Result<SolidKey> {
    let skin = Cone::new(Axis::z(), slope).expect("a positive slope");
    let mut builder = SolidBuilder::new();
    let rim = builder
        .add_vertex(Point::new(slope * far, 0.0, far))
        .expect("a finite point");
    let circle = builder
        .add_edge(rim, rim, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the cap plane cuts the cone in one closed curve");
    let cap = a_cap(
        &mut builder,
        Point::new(0.0, 0.0, far),
        Vector::new(0.0, 0.0, 1.0),
        circle,
        Orientation::Forward,
    );
    let side_coedge = builder
        .add_coedge(circle, Orientation::Reversed)
        .expect("an edge of this build");
    let side_wire = builder
        .add_wire(&[DraftStep::Along(side_coedge)])
        .expect("one closed edge is a wire");
    let side = builder
        .add_face(Surface::Cone(skin), &[side_wire])
        .expect("a face on the cone, bounded by its base circle");
    builder
        .add_shell(&[(cap, Orientation::Forward), (side, Orientation::Forward)])
        .expect("two faces close");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// A quarter of a torus about `+z`, from the cap in the plane `y = 0` to the
/// cap in the plane `x = 0`, with its seam along the inner equator.
///
/// The corners sit on the inner equator, `(ring − tube, 0, 0)` and
/// `(0, ring − tube, 0)`, where the torus's quartic is a difference of two
/// products that are equal number for number, so it cancels exactly for any
/// sizes. The seam names which of the two circles the plane `z = 0` cuts by
/// saying it is inside the pipe of the ring's own radius, and which quarter of
/// that circle by a half-space.
fn a_quarter_bend(model: &mut Model, ring: f64, tube: f64) -> rcad_topo::Result<SolidKey> {
    let skin = Torus::new(Axis::z(), ring, tube).expect("a tube narrower than its ring");
    let mut builder = SolidBuilder::new();
    let inner = ring - tube;
    let start_corner = builder
        .add_vertex(Point::new(inner, 0.0, 0.0))
        .expect("a finite point");
    let end_corner = builder
        .add_vertex(Point::new(0.0, inner, 0.0))
        .expect("a finite point");
    let flat_through_the_middle =
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let seam = builder
        .add_edge(
            start_corner,
            end_corner,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(flat_through_the_middle),
                picked_by: [
                    Picker::InsideAPipe(
                        Cylinder::new(Axis::z(), ring).expect("the ring's own radius"),
                    ),
                    Picker::from(
                        HalfSpace::new(Vector::new(1.0, 1.0, 0.0), 0.0)
                            .expect("a normal with a length"),
                    ),
                ]
                .into_iter()
                .collect(),
            },
        )
        .expect("a quarter of the inner equator");
    let start_circle = builder
        .add_edge(start_corner, start_corner, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("one whole circle of the two this plane cuts");
    let end_circle = builder
        .add_edge(end_corner, end_corner, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("one whole circle of the two this plane cuts");

    let start_cap = a_cap(
        &mut builder,
        Point::origin(),
        Vector::new(0.0, -1.0, 0.0),
        start_circle,
        Orientation::Reversed,
    );
    let end_cap = a_cap(
        &mut builder,
        Point::origin(),
        Vector::new(-1.0, 0.0, 0.0),
        end_circle,
        Orientation::Forward,
    );
    let side = a_wall(&mut builder, Surface::Torus(skin), start_circle, seam, end_circle);
    builder
        .add_shell(&[
            (start_cap, Orientation::Forward),
            (end_cap, Orientation::Forward),
            (side, Orientation::Reversed),
        ])
        .expect("three faces close");
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// A flat face through `at`, facing `outward`, trimmed by one closed edge.
fn a_cap(
    builder: &mut SolidBuilder,
    at: Point,
    outward: Vector,
    circle: rcad_topo::DraftEdge,
    turn: Orientation,
) -> rcad_topo::DraftFace {
    let plane = Plane::from_point_and_normal(at, outward).expect("a coordinate direction");
    let coedge = builder
        .add_coedge(circle, turn)
        .expect("an edge of this build");
    let wire = builder
        .add_wire(&[DraftStep::Along(coedge)])
        .expect("one closed edge is a wire");
    builder
        .add_face(Surface::Plane(plane), &[wire])
        .expect("a cap is a plane trimmed by its circle")
}

/// The curved face between two closed edges: round the first, along the seam,
/// back round the second, and back down the seam — the seam's two coedges in
/// one wire, which is what makes it a seam.
fn a_wall(
    builder: &mut SolidBuilder,
    surface: Surface,
    first: rcad_topo::DraftEdge,
    seam: rcad_topo::DraftEdge,
    second: rcad_topo::DraftEdge,
) -> rcad_topo::DraftFace {
    let round_the_wall: Vec<DraftStep> = [
        (first, Orientation::Forward),
        (seam, Orientation::Forward),
        (second, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ]
    .into_iter()
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("an edge of this build"),
        )
    })
    .collect();
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("round one end, along the seam, round the other, and back");
    builder
        .add_face(surface, &[wire])
        .expect("a face on the curved surface")
}

/// Numbers drawn from the seed libFuzzer hands the mutator, so a mutation is
/// the same every time its seed is — except libFuzzer's own byte mutation,
/// which draws from libFuzzer's generator and not from this seed.
struct Draw(u64);

impl Draw {
    fn new(seed: u32) -> Self {
        Self(u64::from(seed))
    }

    /// SplitMix64: every seed, even zero, gives a well-mixed stream.
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, count: u64) -> u64 {
        self.next() % count
    }

    /// A whole number from `low` to `high`, both included.
    fn whole(&mut self, low: i32, high: i32) -> f64 {
        let span = u64::try_from(high - low + 1).expect("low is not above high");
        f64::from(low) + self.below(span) as f64
    }

    /// A number from zero up to but not including one.
    fn fraction(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1_u64 << 53) as f64
    }
}
