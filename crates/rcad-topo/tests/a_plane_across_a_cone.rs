//! A plane steeper than a cone's side cuts it in two branches, and a fuse sees
//! both — which is why `select.rs`'s `in_the_bulge_of` counts a side cut by a
//! cone in a curve that does not close only when the point is on that side's
//! own nappe (§251; §198.2 flagged the older sentence, that it did not count
//! such a side at all, as prose with nothing behind it, and this file is what
//! is behind it).
//!
//! A cone *face* as rcad builds it is on one nappe (`tests/cones/mod.rs`), but
//! the surface it carries is the whole double cone (`rcad_geom::Cone`), and two
//! things read the surface rather than the face:
//!
//! 1. **The fuse cuts a plane face along the whole crossing**, not only where
//!    the cone's face is. [`a_block_leaning_through_a_frustum_is_cut_along_the_cone_off_the_frustums_own_face`].
//! 2. **"Inside the cone" is the cone's sign**, and on a plane steeper than the
//!    cone that sign is inside in two separate regions, one per nappe.
//!    [`on_a_block_wall_inside_the_cone_is_two_regions_either_side_of_the_apex`].
//!
//! Put together at the one point the bulge candidates are built round — where
//! the cone's axis crosses the plane — and that point can be inside the cone,
//! in the *other* nappe's region, and outside the solid:
//! [`where_the_axis_crosses_a_leaning_face_is_inside_the_cone_and_outside_the_frustum`].
//! A ball's inside on a plane is one disk, and the bulge argument leans on that.

use rcad_geom::{Axis, Cone, Isometry, Plane, Point, Scale, Vector};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog, SideOfSurface};
use rcad_topo::{Containment, Error, Model, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::a_frustum;

/// The frustum every test here uses: slope one half about `+z`, from `z = 2`
/// to `z = 8`. So its face is on the upper nappe and nowhere below `z = 2`.
fn the_frustum(model: &mut Model) -> SolidKey {
    a_frustum(model, 0.5, 2.0, 8.0)
}

/// A block whose low-`x` face is the plane `x = 3/8 · (z + 1)`, with the block
/// on the `+x` side of it.
///
/// That plane leans at `3/8` against the cone's `1/2`, so it is steeper than
/// the cone's side and cuts both nappes: the upper branch has its lowest point
/// at `z = 3` (where `z/2 = 3/8 · (z + 1)`), inside the frustum's span, and the
/// lower one is below the apex. It crosses the cone's axis at `z = −1`.
fn a_leaning_block(model: &mut Model) -> SolidKey {
    let rectangle =
        ProfileBuilder::new(vec![[0.0, -10.0], [20.0, -10.0], [20.0, 10.0], [0.0, 10.0]])
            .build(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a rectangle is a profile");
    let block = model
        .add_extrusion(
            &rectangle,
            40.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -20.0)).expect("a finite move"),
        )
        .expect("a rectangle extrudes");
    let about_y = Axis::new(Point::origin(), Vector::new(0.0, 1.0, 0.0)).expect("a direction");
    let lean = Isometry::rotation(&about_y, 0.375f64.atan()).expect("a finite turn");
    let placed = Isometry::translation(Vector::new(0.375, 0.0, 0.0))
        .expect("a finite move")
        .compose(&lean);
    model
        .place_solid(block, &placed)
        .expect("this model made the key");
    block
}

/// The block's `y = −10` wall is cut along the cone at `z ≈ −20`, on the nappe
/// below the apex where the frustum has no face at all, and that is the piece
/// the fuse cannot find a point in.
///
/// Nothing of the frustum is down there. The cut is the cone's surface carried
/// on past its face, which is `fuse/cuts.rs`'s own rule — a surface reaches past
/// the face carrying it — applied to a cone. So a flat piece's side "along the
/// cone" is along the double cone, and the branch it follows need not be the
/// face's.
///
/// # This used to be a piece of the block's top face, up at `z ≈ 18.7`
///
/// That piece is answered now, and the two planes below say why. The top face
/// leans three eighths the *other* way, so its normal is within the cone's own
/// half angle of the axis and its crossing is an **ellipse**: one closed curve
/// on one nappe, whose inside on that plane is one convex region. Since §211
/// that is what `select.rs`'s `in_the_bulge_of` counts and what `holds.rs` can
/// read an arc's own side of a chord from, so the piece gets a point inside it
/// and the fuse walks on.
///
/// # And it used to be the block's `y = −10` wall, at `z ≈ −20`
///
/// That piece is answered too, since §251. The wall is parallel to the axis, so
/// it cuts a **hyperbola**, and "inside the cone" on it is two regions with the
/// band round the apex between them —
/// [`on_a_block_wall_inside_the_cone_is_two_regions_either_side_of_the_apex`] is
/// that same wall. That was read as the limit, and it is not one: the two
/// regions are one per nappe, told apart by the sign of `(x − apex)·w`, and on
/// one nappe the inside is a convex cone that leaves one convex region on any
/// plane. `select.rs` counts a hyperbola arc's bulge when the point is on the
/// arc's own nappe, and aims a candidate square off the chord at the cone, so
/// the wall's piece gets a point: outside the frustum, kept in the union.
///
/// **What is refused now is a piece of the frustum's own face**, one step on,
/// and it is **two pieces glued into one** (§282). The block's low-`x` face
/// cuts the frustum's face in an arc of the upper branch, from `(3.375, −2.147,
/// 8)` on the top rim down to `(1.5, 0, 3)` and back up to `(3.375, 2.147, 8)`.
/// That leaves two pieces: the patch inside the block, between the arc and the
/// top rim through `(4, 0, 8)`, and the rest, between the bottom rim and the
/// arc with the far side of the top rim. The piece refused has **three wires**
/// — the bottom rim; the rim through `(4, 0, 8)` and the arc; the far rim and
/// the arc again, run the other way — so it holds the arc with the block's
/// material on one side of it and the outside on the other, and no point is on
/// both sides of one plane. Every candidate was asked, and each failed exactly
/// one of the arc's two sides.
///
/// **The glue is `arrange.rs`'s `group_the_cycles_into_cells`.** §36.3 joins
/// two walks in different lumps when they agree about every closed curve. The
/// bottom rim's upper walk agrees about both rims with the inside patch *and*
/// with the rest, since what tells those two apart is the arc, which runs on
/// past the rim and is never drawn in full; so it is joined to both, and they to
/// each other through it. Selection is right to refuse: a point held for the
/// glued piece would keep or drop both halves as one, and in the union one half
/// is dropped and the other kept.
#[test]
fn a_block_leaning_through_a_frustum_is_cut_along_the_cone_off_the_frustums_own_face() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = the_frustum(&mut model);
    let block = a_leaning_block(&mut model);

    let refused = model.fuse(
        &[block],
        &[cone],
        Rule::Union,
        &policy,
        &mut ProvenanceLog::new(),
    );
    let Err(Error::Undecidable { records }) = refused else {
        panic!("the union was expected to refuse, and gave {refused:?}");
    };
    let record = records.records().last().expect("a refusal names why");
    assert_eq!(record.predicate(), "a point inside a piece of a face");
    let corners = record.inputs();
    assert_eq!(
        corners.len(),
        6,
        "the piece's corners, wire by wire: {corners:?}"
    );
    let where_they_are: Vec<[i64; 3]> = corners
        .iter()
        .map(|corner| {
            let [x, y, z] = corner.enclosure();
            assert!(
                (2.0..=8.0).contains(&z.lo()) && (2.0..=8.0).contains(&z.hi()),
                "on the frustum's own face, between its caps: {corner:?}"
            );
            // On the cone, `x² + y² = (z/2)²`, to a rounding.
            let across = (x.lo() * x.lo() + y.lo() * y.lo()).sqrt();
            assert!((across - 0.5 * z.lo()).abs() < 1e-9, "{corner:?}");
            [x.lo(), y.lo(), z.lo()].map(|along| (along * 1000.0).round() as i64)
        })
        .collect();
    assert_eq!(
        where_they_are,
        [
            [1000, 0, 2000],
            [3375, 2147, 8000],
            [4000, 0, 8000],
            [3375, -2147, 8000],
            [3375, -2147, 8000],
            [3375, 2147, 8000],
        ],
        "wire by wire: the bottom rim's seam; the patch inside the block, round the \
         top rim's seam and back down the arc; and the rest of the top rim, back \
         along the same arc the other way"
    );

    let skin = Cone::new(Axis::z(), 0.5).expect("a positive slope");
    let wall = Plane::from_coefficients([0.0, 1.0, 0.0, 10.0]).expect("a plane");
    assert_eq!(
        skin.is_cut_in_a_closed_curve_by(&wall),
        Some(false),
        "a wall parallel to the axis cuts a hyperbola, which never closes"
    );
    let top = Plane::from_coefficients([0.375, 0.0, 1.0, -20.0]).expect("a plane");
    assert_eq!(
        skin.is_cut_in_a_closed_curve_by(&top),
        Some(true),
        "the block's top face leans the other way and cuts an ellipse, which closes"
    );
}

/// On the plane `y = −10`, which is a wall of the leaning block, the cone's
/// inside is two regions: above the apex and below it, with the band round the
/// apex outside.
///
/// The plane is parallel to the axis, so it cuts both nappes in the two
/// branches of `x² + 100 = z²/4`. The sign is what `select.rs` reads as "the
/// side of the partner the material is on", and it is the same `Neg` in both.
#[test]
fn on_a_block_wall_inside_the_cone_is_two_regions_either_side_of_the_apex() {
    let skin = Cone::new(Axis::z(), 0.5).expect("a positive slope");
    let side_at = |z: f64| skin.side_of(PredicatePoint::Exact([0.0, -10.0, z]));
    assert_eq!(side_at(22.0), Decision::Neg, "inside the upper nappe");
    assert_eq!(side_at(0.0), Decision::Pos, "outside, between the two");
    assert_eq!(side_at(-22.0), Decision::Neg, "inside the lower nappe too");
}

/// Where the cone's axis crosses the leaning face, `(0, 0, −1)`, the cone says
/// inside and the frustum says outside.
///
/// That is the point `select.rs`'s `the_middle_of_a_round_wire` hands
/// `half_way_into_either_arc` for a side cut by a cone, and its argument that
/// the point is inside the curve holds for an ellipse only. On this plane it is
/// in the lower nappe's region, below the apex, while a side along the upper
/// branch bounds the upper region only. So the check `in_the_bulge_of` relies
/// on — the partner's sign says the point is in the one region the side bounds
/// — passes for a point that region does not contain.
#[test]
fn where_the_axis_crosses_a_leaning_face_is_inside_the_cone_and_outside_the_frustum() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = the_frustum(&mut model);
    let skin = Cone::new(Axis::z(), 0.5).expect("a positive slope");
    let crossing = Point::new(0.0, 0.0, -1.0);
    assert_eq!(
        0.375 * (crossing.z() + 1.0),
        crossing.x(),
        "on the leaning plane"
    );

    assert_eq!(
        skin.side_of(PredicatePoint::Exact(crossing.coordinates())),
        Decision::Neg,
        "the cone's sign calls it inside"
    );
    let placed = model
        .solid(cone)
        .expect("this model made the key")
        .classify_point(crossing, &policy, &mut ProvenanceLog::new());
    assert!(
        matches!(placed, Containment::Outside),
        "and the frustum does not hold it: {placed:?}"
    );
}
