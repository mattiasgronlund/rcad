//! A corner read out of a model, and where it lands once the model is placed.
//!
//! # §17.1 does not generalize, and this is the half that does not
//!
//! §17.1 re-derives every corner of a solid as the meet of the planes around it,
//! and that works because a corner of a *planar* solid has three planes round it
//! and Cramer's rule places it. A corner of a **round bar** has two surfaces
//! round it — the cap's plane and the wall's cylinder — and the whole cap circle
//! satisfies both. There is nothing to solve, so such a corner is read out of the
//! model rather than re-derived (`docs/decisions.md` §31.3).
//!
//! Read, it is exact and better than a Cramer solve: `check_curved_face` has
//! already certified it [`Decision::Zero`](rcad_predicates::Decision::Zero) on
//! its surface, which is a stronger promise than a box.
//!
//! **Moved, it is not.** `bake` composes placement, and both the point and the
//! surface round it are rounded on the way — separately, and in different ways.
//! So the moved corner is *near* the moved surface and not on it, and handing it
//! to [`SideOfSurface`](rcad_predicates::SideOfSurface) as an exact triple buys
//! a confident sign about a rounding. That is §15.5's mistake arriving by the
//! road §27.7 already found once, and
//! [`where_a_corner_lands_after_moving`] is what stops it: the corner becomes
//! the box it is really known to be inside, and a sign question about it comes
//! back undecided, which is the truth.

use rcad_predicates::{Approximation, Cause, Interval, Point as PredicatePoint};

use crate::isometry::Isometry;
use crate::point::Point;
use crate::{Error, Result, Surface};

/// How far a point that was exactly on a surface can end up from the **stored**
/// moved surface once both have been moved, as a multiple of the largest number
/// involved.
///
/// The size to multiply by is the largest magnitude among the corner's
/// coordinates before and after the move, the movement's own shift, and the
/// numbers each surface places and sizes itself with before and after — a
/// plane's offset, a ball's centre and radius, a pipe's axis origin and radius.
/// Absolute in that size and never relative to the answer, for the reason
/// [`Plane::SIGNED_DISTANCE_ROUNDOFF`](crate::Plane::SIGNED_DISTANCE_ROUNDOFF)
/// opens with.
///
/// # Where the 128 comes from
///
/// §31.10 item 6 asks whether this is one [`Isometry::POINT_ROUNDOFF`] or two,
/// "because `Surface::transformed` rounds the surface as well as the point".
/// **It is neither: it is five, and the constant published here is seven.**
/// Two slips add, and the second depends on which surface the corner sits on.
///
/// **The point.** [`Isometry::apply_to_point`] puts `p` within
/// `POINT_ROUNDOFF · L` of the true `M(p)`. That is one.
///
/// **A ball.** `Sphere::transformed` moves the centre and carries the radius
/// over exactly, so the stored moved ball is a ball of exactly the right size
/// whose centre is `POINT_ROUNDOFF · L` out. Every point of the true moved ball
/// is that far from it and no further. **One more.**
///
/// **A plane.** `Plane::transformed` turns the normal and carries the offset
/// over, adding `n' · t`. Writing `ν = Rn` and `Δ = δ + ν·t` for the true moved
/// plane, and `u = f64::EPSILON / 2`, a point `x` on it satisfies
/// `n'·x − δ' = (n' − ν)·x + (Δ − δ')`. The turned normal is a unit vector, so
/// its slip is `POINT_ROUNDOFF` with `L = 1`, and the first term is at most
/// `√3 · POINT_ROUNDOFF · L`. The second carries the same slip through the shift
/// — another `√3 · POINT_ROUNDOFF · L` — plus the dot product's own `4u·√3·L`
/// and the sum's `u·2L`. Divided by `|n'|`, which is one to within
/// `POINT_ROUNDOFF`, that comes to under `2√3 · POINT_ROUNDOFF · L + 8u·L`,
/// which is under **four** `POINT_ROUNDOFF · L`.
///
/// **A pipe.** `Axis::transformed` moves the origin and turns the direction and
/// rescales neither, and the radius is carried over exactly. The origin slips
/// `POINT_ROUNDOFF · L`; the direction slips `POINT_ROUNDOFF` as a unit vector,
/// which moves the axis under a point at most `2L` away by
/// `2 · POINT_ROUNDOFF · L`; and the certified radius `r/‖w‖` moves by
/// `r · POINT_ROUNDOFF / ‖w‖²`, under `POINT_ROUNDOFF · L`. **Four again.**
///
/// So the worst is four plus the point's one, which is `90 · f64::EPSILON · L`.
/// The constant is `128`, about seven, and the room left over is the same room
/// [`Isometry::POINT_ROUNDOFF`] leaves for second-order terms — a rounding is a
/// relative slip of the *computed* value rather than of the true one — taken
/// twice, because two of those chains compose here.
///
/// `a_corner_read_out_of_a_model_and_moved_is_a_box_around_where_it_lands.rs`
/// measures the worst a search can find and prints the figure (§15.6).
pub const CORNER_AFTER_MOVING_ROUNDOFF: f64 = 128.0 * f64::EPSILON;

/// Where a corner with two surfaces round it lands once `movement` is applied,
/// as the box it is really known to be inside.
///
/// `corner` must be a point the model already holds as being on both of `on`,
/// which is what `check_curved_face` certifies. What comes back is
/// [`Point::Constructed`](rcad_predicates::Point::Constructed) with half-width
/// [`CORNER_AFTER_MOVING_ROUNDOFF`] times the largest number involved, tagged
/// [`Cause::ConstructedPoint`] — **representation wobble, which refuses
/// nothing** (§17.2), rather than an approximation anyone chose.
///
/// # Errors
///
/// [`Error::NotFinite`] when the corner, the movement or a surface holds a
/// number that is not finite, or when the moved corner overflows. A box round a
/// place that is not a place says nothing, and a caller handed one would take it
/// for a place.
pub fn where_a_corner_lands_after_moving(
    corner: Point,
    on: [&Surface; 2],
    movement: &Isometry,
) -> Result<(PredicatePoint, Approximation)> {
    if !(corner.is_finite() && movement.is_finite()) {
        return Err(Error::NotFinite {
            what: "corner of a curved face",
        });
    }
    let moved = movement.apply_to_point(corner);
    let placed = on.map(|surface| surface.transformed(movement));
    let size = the_largest_number_involved(corner, moved, on, &placed, movement);
    let reach = CORNER_AFTER_MOVING_ROUNDOFF * size;
    if !(moved.is_finite() && reach.is_finite()) {
        return Err(Error::NotFinite {
            what: "corner of a curved face after it is moved",
        });
    }
    Ok((
        PredicatePoint::Constructed(moved.coordinates().map(|coordinate| {
            Interval::new(coordinate - reach, coordinate + reach)
                .expect("a reach is a positive length, so the low end is below the high one")
        })),
        Approximation::new(reach, Cause::ConstructedPoint),
    ))
}

/// The largest magnitude anything in this move was worked out from.
fn the_largest_number_involved(
    corner: Point,
    moved: Point,
    before: [&Surface; 2],
    after: &[Surface; 2],
    movement: &Isometry,
) -> f64 {
    let widest = [corner, moved, Point::origin() + movement.translation_part()]
        .iter()
        .map(|place| place.largest_coordinate_magnitude())
        .chain(before.iter().map(|surface| how_big_a_surface_is(surface)))
        .chain(after.iter().map(how_big_a_surface_is))
        .fold(0.0, f64::max);
    // A surface at the origin with no size still has its move worked out by
    // arithmetic that rounds against the smallest normal number, and a size of
    // zero would publish a bound of zero.
    widest.max(f64::MIN_POSITIVE)
}

/// The largest number a surface places or sizes itself with.
fn how_big_a_surface_is(surface: &Surface) -> f64 {
    match surface {
        Surface::Plane(flat) => flat.offset().abs(),
        Surface::Sphere(ball) => ball
            .centre()
            .largest_coordinate_magnitude()
            .max(ball.radius()),
        Surface::Cylinder(pipe) => pipe
            .axis()
            .origin()
            .largest_coordinate_magnitude()
            .max(pipe.radius()),
    }
}
