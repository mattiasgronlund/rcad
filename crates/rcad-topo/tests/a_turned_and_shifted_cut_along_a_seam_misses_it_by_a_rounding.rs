//! Q913, from Q717 and §294.9 (Q912): whether the turned-then-shifted round
//! bar cut along its seam, refused `Undecidable` "side of plane" at the
//! bottom seam corner's box (§294.9), is a gap or a limit.
//!
//! `add_round_bar`'s bar seams at `(3, 0, 0)`: the plane through its axis
//! (`z`, through the origin) and that corner is `y = 0`.
//! `THE_HALF_PAST_THE_SEAM`'s own face at `y = 0`, before the extrusion's own
//! offset is composed in, is the same plane. §294.9 measured that moving
//! both the same way keeps them one plane, turned; composing a shift in too
//! loses it by a rounding. This pins both halves straight from the surfaces,
//! through the public `Plane`, `Cylinder` and `Isometry` API and
//! `rcad_predicates::expansion`, without building a model or running a fuse
//! — no bake, no arrange, no `place_the_corners`.

use rcad_geom::{Axis, Isometry, Plane, Point, Vector};
use rcad_predicates::expansion::{expansion_sum, two_product};

fn the_turn() -> Isometry {
    Isometry::rotation(&Axis::x(), 0.4).unwrap()
}

fn the_shift() -> Isometry {
    Isometry::translation(Vector::new(7.0, -5.0, 2.5)).unwrap()
}

/// The plane through the bar's axis and its seam corner `(3, 0, 0)`: the
/// direction from the axis to the corner, `x`, crossed with the axis, `z`.
fn the_bars_seam_plane() -> Plane {
    Plane::from_point_and_normal(
        Point::origin(),
        Vector::new(1.0, 0.0, 0.0).cross(Vector::new(0.0, 0.0, 1.0)),
    )
    .unwrap()
}

/// `THE_HALF_PAST_THE_SEAM`'s own face at `y = 0`, in the block's own frame
/// before `Model::add_extrusion`'s own offset is composed in.
fn the_blocks_own_face() -> Plane {
    Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, -1.0, 0.0)).unwrap()
}

/// `Model::add_extrusion`'s own placement of the block's footprint, composed
/// inside whatever places the block as a whole (`a_block` in Q717's probe,
/// `target/q717/q717_probe.rs`).
fn the_blocks_own_offset() -> Isometry {
    Isometry::translation(Vector::new(-5.0, 0.0, -1.0)).unwrap()
}

/// The exact value of `n · p − d`, as the sum of an expansion rounded once —
/// Shewchuk's recipe, the one Q717's probe used.
fn exactly(n: [f64; 3], p: [f64; 3], d: f64) -> f64 {
    let mut acc: Vec<f64> = vec![-d];
    for axis in 0..3 {
        let (hi, lo) = two_product(n[axis], p[axis]);
        let mut out = vec![0.0; acc.len() + 2];
        let len = expansion_sum(&acc, &[lo, hi], &mut out);
        out.truncate(len);
        acc = out;
    }
    acc.iter().sum()
}

#[test]
fn turned_the_seam_plane_is_the_blocks_face_bit_for_bit() {
    let seam = the_bars_seam_plane().transformed(&the_turn());
    let face = the_blocks_own_face().transformed(&the_turn().compose(&the_blocks_own_offset()));
    assert_eq!(seam.normal(), face.normal());
    assert_eq!(seam.offset(), face.offset());
    assert_eq!(seam.offset(), 0.0);
}

#[test]
fn turned_and_shifted_the_seam_plane_misses_the_blocks_face_by_one_ulp() {
    let placement = the_shift().compose(&the_turn());
    let seam = the_bars_seam_plane().transformed(&placement);
    let face = the_blocks_own_face().transformed(&placement.compose(&the_blocks_own_offset()));

    assert_eq!(seam.normal(), face.normal());
    assert_ne!(seam.offset(), face.offset());
    assert_eq!(
        seam.offset().to_bits() - face.offset().to_bits(),
        1,
        "the block's face sits one ulp under the seam plane's own offset"
    );
    assert_eq!(seam.offset(), 3.6317591142427994);
    assert_eq!(face.offset(), 3.631759114242799);
}

/// §3.5's ceiling on `max coordinate magnitude / smallest feature size` is
/// `1e11`, and a feature written straight into the coordinates is gone by a
/// spread of `1e16`. The miss here is a spread of about `3e16` against the
/// scale the placed bar reaches: past both, so there is no answer an `f64`
/// scene at this scale could hold, not an answer the arithmetic failed to
/// reach.
#[test]
fn turned_and_shifted_the_miss_is_a_limit_the_scale_already_rules_out() {
    let placement = the_shift().compose(&the_turn());
    let face = the_blocks_own_face().transformed(&placement.compose(&the_blocks_own_offset()));
    let corner = placement.apply_to_point(Point::new(3.0, 0.0, 0.0));

    let miss = exactly(
        face.normal().components(),
        corner.coordinates(),
        face.offset(),
    );
    assert_eq!(miss, 3.885780586188048e-16);

    let top_corner = placement.apply_to_point(Point::new(3.0, 0.0, 10.0));
    let scale = top_corner.coordinates()[2];
    assert_eq!(scale, 11.710609940028851);

    let spread = scale / miss;
    assert!(
        spread > 1e11,
        "past §3.5's own validity ceiling, which is 1e11: {spread:e}"
    );
    assert!(
        spread > 1e15,
        "in the range §3.5 says a feature is already gone, around 1e16: {spread:e}"
    );
}
