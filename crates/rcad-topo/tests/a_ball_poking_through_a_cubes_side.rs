//! A ball poking a whisker through the side of a cube, united with it.
//!
//! A two-hour `cargo fuzz` soak of `two_solids_fused_by_a_stranger` found this
//! one. The model is a cube of side two with a ball of radius one sitting at
//! one of its faces, pushed 2.899e-12 too far so the ball breaks through. The
//! union keeps the whisker of ball that pokes out, and the circle where the
//! ball crosses the flat face becomes a **closed** edge — one edge with one
//! corner, walked by the flat face on one side and the round cap on the other.
//!
//! Those two have to walk it opposite ways round. On `2ca6916` they walk it the
//! same way, and the solid the fuse hands back is broken.
//!
//! # Why the bytes and not the builder
//!
//! The cube's `x = 2` face has the normal `[1, 7.418412301374843e-68, 0]`. That
//! whisker of `y` is far below anything that moves a point — the face is `x = 2`
//! to every digit — but it is what decides *where on the crossing circle* the
//! one corner is put, and the corner's place on the circle is the whole of what
//! this bug turns on (see the failure below). `SolidBuilder` builds a cube out
//! of exact axis normals, so it cannot make that face; the fixture beside this
//! file is the model, postcard-encoded, exactly as the fuzzer handed it over.
//!
//! # What goes wrong
//!
//! `a_closed_crossing_is_walked_the_way_it_is_stored` reads the sign of
//! `(∇S × ∇T) · (ẑ × ∇C)` — the wire's direction against the curved surface's
//! longitude. Swapping the two faces swaps `S` and `T`, which negates the cross
//! product, so the two faces get opposite answers **as long as that dot is not
//! zero**. Write the dot out with `n` the plane's normal and `r` the radius
//! vector at the corner: it is `(n·ẑ)R² − (n·r)(r·ẑ)`. Here the plane is
//! parallel to the ball's winding axis, so `n·ẑ = 0`, and the corner lands on
//! the ball's equator, so `r·ẑ = 0`. The dot is exactly `0.0`, `> 0.0` is
//! `false` for both orderings, and both faces walk the circle the same way.

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, Rule};

/// The model the fuzzer built, postcard-encoded.
const A_BALL_POKING_THROUGH_A_CUBES_SIDE: &[u8] =
    include_bytes!("a_ball_poking_through_a_cubes_side.postcard");

/// A fuse that says it worked hands back a model that is still sound.
///
/// The assert is the property and not the panic: on `2ca6916` this comes apart
/// as a panic from `Model::check_touched_edge` in a debug build, and in a build
/// with debug assertions off the same fuse gets as far as writing the broken
/// shell into a scratch model. That difference between the two builds is gone —
/// §66.3 put those checks in every build — but the reason for asking the
/// property rather than the panic has not changed: what is asked here is the
/// only thing a caller can ask, since `fuse` said `Ok`, so `validate` has to
/// agree.
#[test]
fn uniting_them_does_not_walk_the_crossing_circle_twice_the_same_way() {
    let mut model = postcard::from_bytes::<Model>(A_BALL_POKING_THROUGH_A_CUBES_SIDE)
        .expect("the fixture is a model");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("and a sound one — this is an input the fuse is entitled to be given");

    let solids: Vec<_> = model.solids().map(|solid| solid.key()).collect();
    assert_eq!(solids.len(), 2, "a cube and a ball");

    let policy = Policy::tolerant(model.scale().nominal_size());
    let fused = model.fuse(
        &[],
        &solids,
        Rule::Union,
        &policy,
        &mut ProvenanceLog::new(),
    );

    if fused.is_ok() {
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a fuse that says it worked leaves a sound model behind");
    }
}
