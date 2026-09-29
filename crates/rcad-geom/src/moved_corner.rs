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

use rcad_predicates::expansion::two_sum;
use rcad_predicates::{Approximation, Cause, Decision, Interval, Point as PredicatePoint};

use crate::exact::Exact;
use crate::isometry::Isometry;
use crate::point::{Axis, Point, Vector};
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
/// `2 · POINT_ROUNDOFF · L`; and the certified radius does not move at all.
/// **Three, and it used to be four.** The fourth was the radius term: the
/// certified radius was `r/‖w‖`, which a turned direction moved by
/// `r · POINT_ROUNDOFF / ‖w‖²`. §118.11 put `‖w‖²` on both sides of the pipe's
/// comparison, so the certified radius is now the stored `r` and
/// `Axis::transformed` carries it over untouched.
///
/// So the worst is the **plane's** four plus the point's one, which is
/// `90 · f64::EPSILON · L`, the same total as before — a pipe used to tie with a
/// plane and now comes in under it.
/// The constant is `128`, about seven, and the room left over is the same room
/// [`Isometry::POINT_ROUNDOFF`] leaves for second-order terms — a rounding is a
/// relative slip of the *computed* value rather than of the true one — taken
/// twice, because two of those chains compose here.
///
/// `a_corner_read_out_of_a_model_and_moved_is_a_box_around_where_it_lands.rs`
/// measures the worst a search can find and prints the figure (§15.6).
pub const CORNER_AFTER_MOVING_ROUNDOFF: f64 = 128.0 * f64::EPSILON;

/// How far a **shift** can put a surface or a picker from where it truly goes,
/// as a multiple of the largest number involved.
///
/// The size to multiply by is the largest magnitude among the shift's own
/// components and the numbers the surface places and sizes itself with, before
/// and after — the same list [`CORNER_AFTER_MOVING_ROUNDOFF`] reads.
///
/// # Where the eight comes from
///
/// [`CORNER_AFTER_MOVING_ROUNDOFF`]'s reckoning with every term that comes from
/// *turning* struck out, because a shift does not turn. Writing
/// `u = f64::EPSILON / 2`:
///
/// - **A ball or a pipe.** The centre or the axis origin gains the shift one
///   coordinate at a time, each an addition on terms under `2L`, so under `u·2L`
///   apiece and under `2√3·u·L` as a vector. The direction and the radius are
///   carried over untouched, so nothing else moves. **Under `3.5u·L`.**
/// - **A plane or a half-space.** The normal is carried over untouched and the
///   offset gains `n·t`: three products on terms under `L`, two sums on partial
///   sums under `√3·L`, and the last addition on terms under `2L`. That is under
///   `(3 + 2√3 + 2)·u·L`, divided by `|n|` — which a plane makes one and a
///   half-space leaves to its caller. **Under `9u·L` for a unit normal.**
///
/// The worst is the plane's, `4.5 · f64::EPSILON · L`. The constant is eight,
/// and the room left over is the same room
/// [`Isometry::POINT_ROUNDOFF`](crate::Isometry::POINT_ROUNDOFF) leaves for
/// second-order terms — a rounding is a relative slip of the *computed* value
/// rather than of the true one.
///
/// **Nothing here holds for a movement that turns.** Both rows rest on the
/// normal, the direction and the radius being carried over untouched, which is
/// what a shift does and a turn does not.
/// [`HalfSpace::transformed`](crate::HalfSpace::transformed) is the general
/// movement, and it mints no third constant: its normal and offset are the flat
/// plane [`CORNER_AFTER_MOVING_ROUNDOFF`] already reckons with, under four
/// [`Isometry::POINT_ROUNDOFF`](crate::Isometry::POINT_ROUNDOFF) for a unit
/// normal. What a turn takes away that no bound describes is the exact
/// squareness of a picking normal (§32.3), and
/// [`HalfSpace::transformed`](crate::HalfSpace::transformed) is where that is
/// set out and demonstrated.
///
/// `a_surface_moved_by_a_shift_stays_inside_its_published_bound` measures the
/// worst a search can find and prints the figure (§15.6). `rcad-topo`'s mesher
/// is what spends it.
pub const SURFACE_AFTER_SHIFTING_ROUNDOFF: f64 = 8.0 * f64::EPSILON;

/// Where a corner lands once `movement` is applied, as the box it is really
/// known to be inside — which is a point, when the move rounded nothing.
///
/// `on` is every surface round the corner, as the model stores them before the
/// move. What comes back is
/// [`Point::Constructed`](rcad_predicates::Point::Constructed) with half-width
/// [`CORNER_AFTER_MOVING_ROUNDOFF`] times the largest number involved, tagged
/// [`Cause::ConstructedPoint`] — **representation wobble, which refuses
/// nothing** (§17.2), rather than an approximation anyone chose.
///
/// # A shift that rounds nothing gives the moved corner exactly
///
/// The box is there because the moved corner and the moved surfaces are each
/// rounded on the way, separately, so what the corner was to its surfaces
/// before the move it is only nearly afterwards. When nothing was rounded that
/// is not so, and a box would turn an exact answer into a refusal: a bar moved
/// by whole numbers whose seam lies on a block's wall asked *side of plane*
/// about a box round `(0, 2, 1)` on `x = 0`, where the answer is `Zero`.
///
/// So the corner comes back as the exact point, with a wobble of zero, when
/// **the moved corner is the stored corner plus the movement's shift, and every
/// moved surface is its stored surface carried along by that shift**, each sum
/// checked exactly rather than against a tolerance. That is sufficient and not
/// merely plausible. The side a surface puts a point on is the sign of one
/// expression in their numbers — `n·p − d` for a plane, `|p − c|² − r²` for a
/// ball, and the same with the axis origin or the apex for the rest — and
/// adding one vector to the point and to the surface's place, with `d` gaining
/// `n·t`, leaves every one of those expressions exactly the number it was. So
/// every question a surface round the corner answers about the moved corner is
/// answered as it was before the move, whatever that answer was; the move added
/// nothing, and there is nothing for a box to cover. It does not rest on the
/// stored corner being exactly on its surfaces, which the model does not
/// promise (§75.6), and it does not rest on how [`Surface::transformed`] does
/// its arithmetic, only on what it handed back.
///
/// A movement that turns fails the first sum for every corner off its axis, so
/// in practice this is a shift and nothing else. A shift whose sums round, or
/// whose products do, keeps its box: that is what the box is for.
///
/// # Errors
///
/// [`Error::NotFinite`] when the corner, the movement or a surface holds a
/// number that is not finite, or when the moved corner overflows. A box round a
/// place that is not a place says nothing, and a caller handed one would take it
/// for a place.
pub fn where_a_corner_lands_after_moving<'a>(
    corner: Point,
    on: impl IntoIterator<Item = &'a Surface>,
    movement: &Isometry,
) -> Result<(PredicatePoint, Approximation)> {
    if !(corner.is_finite() && movement.is_finite()) {
        return Err(Error::NotFinite {
            what: "corner of a curved face",
        });
    }
    let on: Vec<&Surface> = on.into_iter().collect();
    let moved = movement.apply_to_point(corner);
    if a_shift_that_rounded_nothing(corner, moved, &on, movement) {
        return Ok((
            PredicatePoint::from(moved.coordinates()),
            Approximation::new(0.0, Cause::ConstructedPoint),
        ));
    }
    let placed: Vec<Surface> = on
        .iter()
        .map(|surface| surface.transformed(movement))
        .collect();
    let size = the_largest_number_involved(corner, moved, &on, &placed, movement);
    let reach = CORNER_AFTER_MOVING_ROUNDOFF * size;
    if !(moved.is_finite() && reach.is_finite()) {
        return Err(Error::NotFinite {
            what: "corner of a curved face after it is moved",
        });
    }
    Ok((
        PredicatePoint::Constructed(moved.coordinates().map(|coordinate| {
            Interval::around(coordinate, reach)
                .expect("a reach is a positive length, so the low end is below the high one")
        })),
        Approximation::new(reach, Cause::ConstructedPoint),
    ))
}

/// Whether `movement` took the corner and every surface round it to exactly
/// themselves shifted by its shift, with nothing rounded on the way.
fn a_shift_that_rounded_nothing(
    corner: Point,
    moved: Point,
    on: &[&Surface],
    movement: &Isometry,
) -> bool {
    let by = movement.translation_part();
    is_exactly_shifted(corner, moved, by)
        && on
            .iter()
            .all(|surface| is_carried_exactly_by_its_shift(surface, movement, by))
}

/// Whether `after` is `before` plus `by`, each coordinate summed without a
/// rounding.
fn is_exactly_shifted(before: Point, after: Point, by: Vector) -> bool {
    let [before, after, by] = [before.coordinates(), after.coordinates(), by.components()];
    (0..3).all(|axis| {
        let (sum, lost) = two_sum(before[axis], by[axis]);
        lost == 0.0 && sum == after[axis]
    })
}

/// Whether `surface`, moved, is the same surface with its place moved by
/// exactly `by` and nothing else touched.
///
/// Every number a surface stores is compared, the ones a shift must leave alone
/// as well as the ones it moves, so that the answer is about what
/// [`Surface::transformed`] handed back and not about how it got there.
fn is_carried_exactly_by_its_shift(surface: &Surface, movement: &Isometry, by: Vector) -> bool {
    match surface {
        Surface::Plane(flat) => {
            let moved = flat.transformed(movement);
            moved.normal() == flat.normal()
                && is_the_offset_it_should_have(moved.offset(), flat.offset(), flat.normal(), by)
        }
        Surface::Sphere(ball) => {
            let moved = ball.transformed(movement);
            moved.radius() == ball.radius() && is_exactly_shifted(ball.centre(), moved.centre(), by)
        }
        Surface::Cylinder(pipe) => {
            let moved = pipe.transformed(movement);
            moved.radius() == pipe.radius() && is_the_axis_shifted(pipe.axis(), moved.axis(), by)
        }
        Surface::Cone(cone) => {
            let moved = cone.transformed(movement);
            moved.slope() == cone.slope() && is_the_axis_shifted(cone.axis(), moved.axis(), by)
        }
        Surface::Torus(ring) => {
            let moved = ring.transformed(movement);
            moved.ring_radius() == ring.ring_radius()
                && moved.tube_radius() == ring.tube_radius()
                && is_the_axis_shifted(ring.axis(), moved.axis(), by)
        }
    }
}

/// Whether a plane shifted by `by` holds exactly `offset + normal · by`.
///
/// Summed through [`Exact`], so a dot product that rounds anywhere on the way
/// is caught whichever order it was worked out in.
fn is_the_offset_it_should_have(moved: f64, offset: f64, normal: Vector, by: Vector) -> bool {
    // A product whose low half would fall into the subnormals has no exact
    // two-term form, so nothing can be proved about it.
    let Some(carried) = Exact::dot_of(normal.components(), by.components()) else {
        return false;
    };
    match Exact::of(offset)
        .plus(&carried)
        .minus(&Exact::of(moved))
        .sign()
    {
        Decision::Zero => true,
        Decision::Pos | Decision::Neg => false,
        // An overflow upstream: nothing was proved.
        Decision::Undecidable { .. } => false,
        other => unreachable!("unhandled decision from exact arithmetic: {other:?}"),
    }
}

/// Whether an axis moved to `after` is `before` with its origin moved by
/// exactly `by` and its direction untouched.
fn is_the_axis_shifted(before: Axis, after: Axis, by: Vector) -> bool {
    after.direction() == before.direction()
        && is_exactly_shifted(before.origin(), after.origin(), by)
}

/// The largest magnitude anything in this move was worked out from.
fn the_largest_number_involved(
    corner: Point,
    moved: Point,
    before: &[&Surface],
    after: &[Surface],
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
        // A cone has no radius of its own — how far out it reaches is about how
        // far along it you look, which is a question about a face and not about
        // the surface. What it places itself with is its apex, and the slope is
        // a ratio of order one, so the apex is the whole of the size here.
        Surface::Cone(cone) => cone.apex().largest_coordinate_magnitude().max(cone.slope()),
        // A torus is bounded and so has a size of its own, unlike a cone: it
        // reaches `ring + tube` from its centre and never further.
        Surface::Torus(ring) => ring
            .centre()
            .largest_coordinate_magnitude()
            .max(ring.ring_radius() + ring.tube_radius()),
    }
}

#[cfg(test)]
mod tests {
    use crate::{Cone, Cylinder, HalfSpace, Plane, Sphere, Torus};

    use super::*;

    /// §15.6 for [`SURFACE_AFTER_SHIFTING_ROUNDOFF`]: a search for the worst a
    /// shift can do to a surface or a picker, against the number published for
    /// it.
    ///
    /// **The truth is summed exactly and not in `f64`**, so what this compares
    /// against is where the shifted surface really goes rather than a second
    /// rounded guess at it. A plane and a half-space are the interesting rows —
    /// their offset moves by a dot product — and a ball and a pipe are here to
    /// show that the coordinate-at-a-time rows are the small ones.
    ///
    /// The bound is also checked from the other side: a search that found
    /// nothing at all would pass whatever the constant said.
    #[test]
    fn a_surface_moved_by_a_shift_stays_inside_its_published_bound() {
        let mut worst: [f64; 4] = [0.0; 4];
        let mut seed = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            // Numbers spread over sixty octaves, positive and negative, so the
            // dot product has both cancellation and none to work with.
            // A whole fifty-three bits of mantissa: a number with only
            // twenty-four significant bits rounds far less than one that fills
            // the word, and a search over those would understate every slip.
            let mantissa = (seed >> 11) as f64 / (1u64 << 53) as f64;
            let octave = ((seed >> 3) % 60) as i32 - 30;
            let sign = if seed & 1 == 0 { 1.0 } else { -1.0 };
            sign * (1.0 + mantissa) * 2.0f64.powi(octave)
        };

        for _ in 0..20_000 {
            let by = Vector::new(next(), next(), next());
            let shift = Isometry::translation(by).expect("a finite shift");
            let normal = Vector::new(next(), next(), next());
            let through = Point::new(next(), next(), next());

            let flat = Plane::from_point_and_normal(through, normal).expect("a plane");
            worst[0] = worst[0].max(how_far_a_flat_surface_slipped(
                flat.normal(),
                flat.offset(),
                flat.transformed(&shift).offset(),
                by,
            ));

            let picker = HalfSpace::new(normal, normal.dot(through - Point::origin()))
                .expect("a half-space");
            worst[1] = worst[1].max(how_far_a_flat_surface_slipped(
                picker.normal(),
                picker.offset(),
                picker.shifted(by).offset(),
                by,
            ));

            let ball = Sphere::new(through, next().abs()).expect("a ball");
            worst[2] = worst[2].max(how_far_a_place_slipped(
                ball.centre(),
                ball.transformed(&shift).centre(),
                by,
            ));

            let pipe = Cylinder::new(
                Axis::new(through, Vector::new(next(), next(), next())).expect("an axis"),
                next().abs(),
            )
            .expect("a pipe");
            worst[3] = worst[3].max(how_far_a_place_slipped(
                pipe.axis().origin(),
                pipe.transformed(&shift).axis().origin(),
                by,
            ));
        }

        println!(
            "worst shift slip, as a multiple of f64::EPSILON times the size: \
             plane {}, half-space {}, ball {}, pipe {}",
            worst[0] / f64::EPSILON,
            worst[1] / f64::EPSILON,
            worst[2] / f64::EPSILON,
            worst[3] / f64::EPSILON
        );
        for (what, slip) in ["plane", "half-space", "ball", "pipe"].iter().zip(worst) {
            assert!(
                slip <= SURFACE_AFTER_SHIFTING_ROUNDOFF,
                "a shifted {what} slipped {slip}, over the published \
                 {SURFACE_AFTER_SHIFTING_ROUNDOFF}"
            );
        }
        assert!(
            worst[0] > 0.0 && worst[1] > 0.0 && worst[2] > 0.0 && worst[3] > 0.0,
            "a search that found no slip at all would pass whatever the \
             constant said, and found {worst:?}"
        );
    }

    /// A whole-number shift hands back the exact corner round every kind of
    /// surface there is, each set square to the axes.
    ///
    /// Square to the axes because a slanted plane's offset gains a dot product,
    /// and whole numbers times a normal that is not whole can round.
    #[test]
    fn a_whole_number_shift_moves_a_corner_on_any_square_surface_exactly() {
        let whole = Isometry::translation(Vector::new(-7.0, 11.0, 5.0)).expect("a finite shift");
        let flat =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, 3.0), Vector::new(0.0, 0.0, 1.0))
                .expect("a plane");
        let rows = [
            (Point::new(1.0, 2.0, 3.0), Surface::Plane(flat)),
            (
                Point::new(0.0, 0.0, 2.0),
                Surface::Sphere(Sphere::new(Point::origin(), 2.0).expect("a ball")),
            ),
            (
                Point::new(1.0, 0.0, 3.0),
                Surface::Cylinder(Cylinder::new(Axis::z(), 1.0).expect("a pipe")),
            ),
            (
                Point::new(1.0, 0.0, 1.0),
                Surface::Cone(Cone::new(Axis::z(), 1.0).expect("a cone")),
            ),
            (
                Point::new(4.0, 0.0, 0.0),
                Surface::Torus(Torus::new(Axis::z(), 3.0, 1.0).expect("a ring")),
            ),
        ];
        for (corner, surface) in rows {
            let (landed, wobble) = where_a_corner_lands_after_moving(corner, [&surface], &whole)
                .expect("finite numbers");
            let [x, y, z] = corner.coordinates();
            assert_eq!(
                landed,
                PredicatePoint::from([x - 7.0, y + 11.0, z + 5.0]),
                "{corner:?} on {surface:?}"
            );
            assert_eq!(wobble.magnitude(), 0.0);
        }
    }

    /// A shift that moves the corner without a rounding but rounds a surface
    /// round it keeps the box, because the corner is then only nearly what it
    /// was to that surface.
    ///
    /// One row for each way a surface can round while its corner does not: a
    /// pipe written through a far point of its axis, whose origin's sum rounds
    /// where the corner's does not, and a slanted plane, whose offset gains a dot
    /// product that rounds where every coordinate's sum is exact. Each is beside
    /// the same surface under a shift that rounds nothing, so what keeps the box
    /// is the rounding and not the surface.
    #[test]
    fn a_corner_moved_exactly_on_a_surface_that_was_not_keeps_its_box() {
        let far_on_its_axis = Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::new(0.0, 0.0, 3.0), Vector::new(0.0, 0.0, 1.0)).expect("an axis"),
                1.0,
            )
            .expect("a pipe"),
        );
        let slanted = Surface::Plane(
            Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 1.0, 0.0))
                .expect("a plane"),
        );
        let shift = |x: f64, y: f64, z: f64| {
            Isometry::translation(Vector::new(x, y, z)).expect("a finite shift")
        };
        let rows = [
            (
                Point::new(1.0, 0.0, 0.0),
                far_on_its_axis,
                shift(0.0, 0.0, 2f64.powi(-53)),
                shift(0.0, 0.0, 1.0),
            ),
            (
                Point::origin(),
                slanted,
                shift(1.0, 2f64.powi(-30), 0.0),
                shift(1.0, 1.0, 0.0),
            ),
        ];
        for (corner, surface, rounds, whole) in rows {
            assert!(
                is_exactly_shifted(
                    corner,
                    rounds.apply_to_point(corner),
                    rounds.translation_part()
                ),
                "the corner itself moves exactly, so only the surface can round"
            );
            let (boxed, _) = where_a_corner_lands_after_moving(corner, [&surface], &rounds)
                .expect("finite numbers");
            assert!(
                boxed.uncertainty() > 0.0,
                "{surface:?} moved by {rounds:?} rounds, so the corner is a box: {boxed:?}"
            );
            let (exact, _) = where_a_corner_lands_after_moving(corner, [&surface], &whole)
                .expect("finite numbers");
            assert_eq!(exact.uncertainty(), 0.0, "{surface:?} moved by {whole:?}");
        }
    }

    /// How far a shifted flat surface is from where it truly goes, as a multiple
    /// of the largest number involved.
    ///
    /// The true offset is `offset + n · by` summed through [`Exact`], and the
    /// answer is divided by the normal's length because that is what turns a
    /// slip in the offset into a distance.
    fn how_far_a_flat_surface_slipped(
        normal: Vector,
        offset: f64,
        shifted: f64,
        by: Vector,
    ) -> f64 {
        let Some(carried) = Exact::dot_of(normal.components(), by.components()) else {
            return 0.0;
        };
        // Both the slip and the size are divided by the normal's length,
        // because an offset is a distance times that length and only the
        // distance is comparable with a shift.
        let long = normal.length();
        let slipped = Exact::of(offset)
            .plus(&carried)
            .minus(&Exact::of(shifted))
            .nearest_f64()
            .abs()
            / long;
        let size = (the_largest_number_in([offset, shifted]) / long).max(
            by.components()
                .into_iter()
                .map(f64::abs)
                .fold(0.0, f64::max),
        );
        if slipped.is_finite() && size > 0.0 {
            slipped / size
        } else {
            0.0
        }
    }

    /// How far a shifted place is from where it truly goes, as a multiple of the
    /// largest number involved.
    fn how_far_a_place_slipped(place: Point, shifted: Point, by: Vector) -> f64 {
        let mut worst = 0.0f64;
        for axis in 0..3 {
            let truth =
                Exact::of(place.coordinates()[axis]).plus(&Exact::of(by.components()[axis]));
            let slipped = truth
                .minus(&Exact::of(shifted.coordinates()[axis]))
                .nearest_f64()
                .abs();
            let size = place
                .largest_coordinate_magnitude()
                .max(shifted.largest_coordinate_magnitude())
                .max(
                    by.components()
                        .into_iter()
                        .map(f64::abs)
                        .fold(0.0, f64::max),
                );
            if slipped.is_finite() && size > 0.0 {
                worst = worst.max(slipped / size);
            }
        }
        worst
    }

    /// The largest magnitude in a handful of numbers, never zero.
    fn the_largest_number_in(numbers: [f64; 2]) -> f64 {
        numbers
            .into_iter()
            .map(f64::abs)
            .fold(f64::MIN_POSITIVE, f64::max)
    }
}
