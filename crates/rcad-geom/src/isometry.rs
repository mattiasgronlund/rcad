//! Rigid movement: a rotation followed by a translation.

use nalgebra as na;
use serde::{Deserialize, Serialize};

use crate::point::{Axis, Point, Vector};
use crate::{Error, Result};

/// A rigid movement — turn, then shift. Never a scale, never a mirror, so it
/// leaves every distance and every angle alone.
///
/// Topology stores one of these next to every reference to a shared shape, and
/// composes it on the way out. That composition is real from day one:
/// `docs/decisions.md` §3.2 says an accessor written without it is one somebody
/// has to find and fix later.
///
/// Serializes as a `([x, y, z], [w, i, j, k])` pair of translation and rotation.
/// The stored rotation is scaled back to length one when it is read, so a file
/// cannot hand us a movement that also stretches the part. That scaling can move
/// the last bit or two of a rotation that was already very nearly unit length,
/// so a movement that has been written and read back puts points in the same
/// places to within rounding, not bit for bit.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "([f64; 3], [f64; 4])", into = "([f64; 3], [f64; 4])")]
pub struct Isometry(na::Isometry3<f64>);

impl Isometry {
    /// How far a moved point can land from where this movement's own
    /// arithmetic, done without rounding, would put it — per unit of `L`, the
    /// largest coordinate magnitude of the point being moved and of the
    /// movement's shift.
    ///
    /// # Where the 18 comes from
    ///
    /// [`Isometry::apply_to_point`] is not a matrix multiply. A rotation is
    /// stored as a quaternion `(w, v)` and Rodrigues' form of it is what runs:
    ///
    /// ```text
    /// c = v × p        t = 2c        k = v × t        moved = ((t·w + k) + p) + shift
    /// ```
    ///
    /// Counting halves of an ulp per operation does not give the answer,
    /// because a rounding costs half an ulp of the *running value*, which is
    /// bigger than any one term. Charge each term for every rounding it passes
    /// through instead. Write `u = f64::EPSILON / 2`, and use twice over the
    /// fact that two different components of a unit vector satisfy
    /// `|v_a| + |v_c| ≤ √2`:
    ///
    /// - `c` is two products and a subtraction per component, over terms
    ///   summing to at most `√2·L`. So `|c_i| ≤ √2·L`, and each term meets two
    ///   roundings: `2u·√2·L`.
    /// - `t = 2c` is exact. `|t_i| ≤ 2√2·L` and the error doubles to `4√2·u·L`.
    /// - `k` is two products and a subtraction again. It carries in
    ///   `(|v_a| + |v_c|)·4√2·u·L ≤ 8u·L`, and adds two roundings of its own on
    ///   terms summing to at most `4L`, another `8u·L`. So `|k_i| ≤ 4L` and its
    ///   error is `16u·L`.
    /// - The last line is a product and two additions. It carries in
    ///   `|w|·4√2·u·L + 16u·L`. The product rounds on `|t_i·w| ≤ 2√2·L`; the
    ///   first addition on `|t_i·w + k_i| ≤ (2√2 + 4)·L`; the second on the
    ///   turned point itself, and a turn does not change a length, so that is
    ///   at most `√3·L`. Adding the shift rounds on `(√3 + 1)·L`.
    ///
    /// Added up, `(8√2 + 21 + 2√3)·u·L ≈ 35.78·u·L`, which is under
    /// `18 · f64::EPSILON · L`. The 0.6% left over covers the second-order
    /// terms — a rounding is a relative slip of the *computed* value, not of
    /// the true one — and the fact that a stored rotation has length one only
    /// to within rounding.
    ///
    /// Every step of that is a relative argument, so it holds only while
    /// nothing overflows and nothing falls into the subnormals.
    ///
    /// # What this bounds, and what it does not
    ///
    /// The arithmetic, given the numbers this movement holds. It says nothing
    /// about how well those numbers match the movement a caller asked for:
    /// `cos(π/2)` in `f64` is `6.1e-17` rather than zero, so even a quarter turn
    /// arrives a rounding away from the turn that was named. A caller deciding
    /// how far apart two differently routed movements of the same point may
    /// land wants this times a slack factor of its own, and that factor belongs
    /// at the caller with its own reason (`docs/decisions.md` §15.6).
    pub const POINT_ROUNDOFF: f64 = 18.0 * f64::EPSILON;

    /// The movement that changes nothing.
    #[must_use]
    pub fn identity() -> Self {
        Self(na::Isometry3::identity())
    }

    /// A shift by `offset`, with no turn.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `offset` has a component that is not a finite
    /// number.
    pub fn translation(offset: Vector) -> Result<Self> {
        if !offset.is_finite() {
            return Err(Error::NotFinite {
                what: "translation",
            });
        }
        Ok(Self(na::Isometry3::translation(
            offset.x(),
            offset.y(),
            offset.z(),
        )))
    }

    /// A shift by `offset` after each of its components has been rounded to the
    /// nearest multiple of two raised to `grid_exponent`.
    ///
    /// A translation on a power-of-two grid has zeros in the low mantissa bits,
    /// so composing it and undoing it does not drift, and intersections done in
    /// the moved frame stay as well-conditioned as they were in the original
    /// one.
    ///
    /// **Nothing in this kernel snaps for you.** Some placements are genuinely
    /// arbitrary, and quietly moving the user's part somewhere they did not ask
    /// for is worse than a slightly worse-conditioned intersection
    /// (`docs/decisions.md` §3.2). Call this when you know the placement is free
    /// to move; otherwise use [`Isometry::translation`].
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `offset` has a component that is not a finite
    /// number, or if the grid spacing is outside the `f64` range.
    pub fn snapped_translation(offset: Vector, grid_exponent: i32) -> Result<Self> {
        Self::translation(offset.snapped_to_power_of_two_grid(grid_exponent)?)
    }

    /// A turn of `angle_in_radians` about `axis`, right-handed about the axis
    /// direction. Points on the axis stay where they are.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if the angle or the axis is not made of finite
    /// numbers.
    pub fn rotation(axis: &Axis, angle_in_radians: f64) -> Result<Self> {
        if !angle_in_radians.is_finite() {
            return Err(Error::NotFinite {
                what: "rotation angle",
            });
        }
        if !axis.origin().is_finite() || !axis.direction().is_finite() {
            return Err(Error::NotFinite { what: "axis" });
        }
        let direction = na::Unit::new_normalize(na::Vector3::from(axis.direction()));
        let turn = na::UnitQuaternion::from_axis_angle(&direction, angle_in_radians);
        Ok(Self(na::Isometry3::rotation_wrt_point(
            turn,
            axis.origin().into(),
        )))
    }

    /// The single movement that does `inner` first and then `self`.
    ///
    /// ```
    /// use rcad_geom::{Isometry, Point, Vector};
    ///
    /// let shift = Isometry::translation(Vector::new(10.0, 0.0, 0.0))?;
    /// let lift = Isometry::translation(Vector::new(0.0, 0.0, 1.0))?;
    ///
    /// let both = lift.compose(&shift);
    /// assert_eq!(both.apply_to_point(Point::origin()), Point::new(10.0, 0.0, 1.0));
    /// # Ok::<(), rcad_geom::Error>(())
    /// ```
    ///
    /// This is the order topology needs: a parent's placement composed with a
    /// child's places the child in the parent's frame first, then the world.
    #[must_use]
    pub fn compose(&self, inner: &Isometry) -> Isometry {
        Self(self.0 * inner.0)
    }

    /// The movement that undoes this one.
    #[must_use]
    pub fn inverse(&self) -> Isometry {
        Self(self.0.inverse())
    }

    /// Where `point` ends up.
    ///
    /// Within [`Isometry::POINT_ROUNDOFF`] times the largest number involved of
    /// where the true movement would put it.
    ///
    /// **A movement with no turn does better than that, and something rests on
    /// it.** It comes to one addition per coordinate and nothing else, so a
    /// shift a caller has arranged to be exact — where Sterbenz's condition
    /// holds, say — really does come out exact. `rcad-topo`'s mesher rests the
    /// whole of the frame it works a far-off face out in on that (§39.7 item 2),
    /// and it is a fact about what this delegates to rather than one this crate
    /// arranges, so
    /// `a_movement_with_no_turn_moves_a_point_by_one_addition_and_nothing_else`
    /// is the demonstration rather than this sentence.
    #[must_use]
    pub fn apply_to_point(&self, point: Point) -> Point {
        self.0.transform_point(&point.into()).into()
    }

    /// Which way `vector` ends up pointing.
    ///
    /// A vector is the difference between two points, so the shift cancels out
    /// and only the turn acts on it. Applying the translation here is the
    /// classic way to move a normal to the wrong place.
    ///
    /// A movement with no turn hands the vector straight back, bit for bit —
    /// same demonstration as [`Isometry::apply_to_point`], and it is what keeps
    /// a shifted pipe's axis direction and a shifted plane's normal exactly what
    /// they were.
    #[must_use]
    pub fn apply_to_vector(&self, vector: Vector) -> Vector {
        self.0.transform_vector(&vector.into()).into()
    }

    /// The shift part, applied after the turn.
    #[must_use]
    pub fn translation_part(&self) -> Vector {
        self.0.translation.vector.into()
    }

    /// The turn part, as `[w, i, j, k]`. Always has length one.
    #[must_use]
    pub fn rotation_quaternion(&self) -> [f64; 4] {
        let turn = self.0.rotation.quaternion();
        [turn.w, turn.i, turn.j, turn.k]
    }

    /// Whether every number in the movement is finite.
    #[must_use]
    pub fn is_finite(&self) -> bool {
        self.translation_part().is_finite()
            && self.rotation_quaternion().iter().all(|c| c.is_finite())
    }
}

impl Default for Isometry {
    fn default() -> Self {
        Self::identity()
    }
}

impl TryFrom<([f64; 3], [f64; 4])> for Isometry {
    type Error = Error;

    fn try_from((offset, [w, i, j, k]): ([f64; 3], [f64; 4])) -> Result<Self> {
        let offset = Vector::from(offset);
        if !offset.is_finite() {
            return Err(Error::NotFinite {
                what: "translation",
            });
        }
        let stored = na::Quaternion::new(w, i, j, k);
        if !stored.coords.iter().all(|c| c.is_finite()) {
            return Err(Error::NotFinite { what: "rotation" });
        }
        if stored.norm() == 0.0 {
            return Err(Error::ZeroLength { what: "rotation" });
        }
        Ok(Self(na::Isometry3::from_parts(
            na::Translation3::new(offset.x(), offset.y(), offset.z()),
            na::UnitQuaternion::from_quaternion(stored),
        )))
    }
}

impl From<Isometry> for ([f64; 3], [f64; 4]) {
    fn from(movement: Isometry) -> Self {
        (
            movement.translation_part().components(),
            movement.rotation_quaternion(),
        )
    }
}

impl From<na::Isometry3<f64>> for Isometry {
    fn from(movement: na::Isometry3<f64>) -> Self {
        Self(movement)
    }
}

impl From<Isometry> for na::Isometry3<f64> {
    fn from(movement: Isometry) -> Self {
        movement.0
    }
}

#[cfg(test)]
mod tests {
    use core::f64::consts::FRAC_PI_2;

    use rcad_predicates::expansion::{expansion_sum, product_is_exact, scale_expansion};

    use super::*;

    fn quarter_turn_about_z() -> Isometry {
        Isometry::rotation(&Axis::z(), FRAC_PI_2).unwrap()
    }

    /// §16.10 for [`Isometry::apply_to_point`]'s "one addition per coordinate
    /// and nothing else".
    ///
    /// **This is a claim about a dependency and not about arithmetic written
    /// here**, which is exactly the kind that stops being true without anybody
    /// touching this file. `rcad-topo`'s mesher shifts a face stored far from
    /// the origin into a frame where its numbers are small, and the whole reason
    /// that is allowed is that the shift is *exact*; a turn of the identity that
    /// cost a rounding would make it inexact and the mesh floor would be
    /// understating what a point costs.
    ///
    /// So: over two hundred thousand draws sixty octaves wide, a shift moves a
    /// point by exactly what adding its three components does, bit for bit, and
    /// hands a vector straight back.
    #[test]
    fn a_movement_with_no_turn_moves_a_point_by_one_addition_and_nothing_else() {
        let mut seed = 0x1234_5678_9abc_def0_u64;
        let mut next_bits = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let a_number = |bits: u64| {
            let mantissa = (bits >> 11) as f64 / (1u64 << 53) as f64;
            let octave = ((bits >> 3) % 60) as i32 - 30;
            let sign = if bits & 1 == 0 { 1.0 } else { -1.0 };
            sign * (1.0 + mantissa) * 2.0f64.powi(octave)
        };

        for _ in 0..200_000 {
            let by = Vector::new(
                a_number(next_bits()),
                a_number(next_bits()),
                a_number(next_bits()),
            );
            let shift = Isometry::translation(by).expect("a finite shift");

            let place = Point::new(
                a_number(next_bits()),
                a_number(next_bits()),
                a_number(next_bits()),
            );
            let moved = shift.apply_to_point(place).coordinates();
            for (axis, (was, along)) in place
                .coordinates()
                .into_iter()
                .zip(by.components())
                .enumerate()
            {
                assert_eq!(
                    moved[axis].to_bits(),
                    (was + along).to_bits(),
                    "shifting {place:?} by {by:?} did something other than add \
                     along coordinate {axis}"
                );
            }

            let way = Vector::new(
                a_number(next_bits()),
                a_number(next_bits()),
                a_number(next_bits()),
            );
            let turned = shift.apply_to_vector(way);
            for (moved, was) in turned.components().into_iter().zip(way.components()) {
                assert_eq!(
                    moved.to_bits(),
                    was.to_bits(),
                    "a shift with no turn changed the direction {way:?}"
                );
            }
        }
    }

    #[test]
    fn a_quarter_turn_about_z_takes_the_x_axis_onto_the_y_axis() {
        let moved = quarter_turn_about_z().apply_to_point(Point::new(1.0, 0.0, 0.0));
        assert!(moved.distance_to(Point::new(0.0, 1.0, 0.0)) < 1e-15);
    }

    #[test]
    fn a_rotation_about_an_offset_axis_leaves_the_axis_alone() {
        let axis = Axis::new(Point::new(5.0, 5.0, 0.0), Vector::new(0.0, 0.0, 1.0)).unwrap();
        let turn = Isometry::rotation(&axis, 0.7).unwrap();
        let on_the_axis = Point::new(5.0, 5.0, 3.0);
        assert!(turn.apply_to_point(on_the_axis).distance_to(on_the_axis) < 1e-15);
    }

    #[test]
    fn a_vector_is_turned_but_not_shifted() {
        let shift = Isometry::translation(Vector::new(100.0, 200.0, 300.0)).unwrap();
        let along_x = Vector::new(1.0, 0.0, 0.0);
        assert_eq!(shift.apply_to_vector(along_x), along_x);
    }

    #[test]
    fn composing_does_the_inner_movement_first() {
        let shift = Isometry::translation(Vector::new(1.0, 0.0, 0.0)).unwrap();
        let turn = quarter_turn_about_z();

        let shift_then_turn = turn.compose(&shift);
        assert!(
            shift_then_turn
                .apply_to_point(Point::origin())
                .distance_to(Point::new(0.0, 1.0, 0.0))
                < 1e-15
        );

        let turn_then_shift = shift.compose(&turn);
        assert!(
            turn_then_shift
                .apply_to_point(Point::origin())
                .distance_to(Point::new(1.0, 0.0, 0.0))
                < 1e-15
        );
    }

    #[test]
    fn snapping_a_translation_moves_it_onto_the_grid() {
        let snapped = Isometry::snapped_translation(Vector::new(0.3, 1.1, -0.7), -2).unwrap();
        assert_eq!(snapped.translation_part(), Vector::new(0.25, 1.0, -0.75));
    }

    #[test]
    fn a_plain_translation_is_left_exactly_where_the_caller_put_it() {
        let awkward = Vector::new(0.1, 0.2, 0.3);
        assert_eq!(
            Isometry::translation(awkward).unwrap().translation_part(),
            awkward
        );
    }

    #[test]
    fn a_translation_that_is_not_a_number_is_refused() {
        assert_eq!(
            Isometry::translation(Vector::new(f64::INFINITY, 0.0, 0.0)),
            Err(Error::NotFinite {
                what: "translation"
            })
        );
    }

    #[test]
    fn a_rotation_by_an_angle_that_is_not_a_number_is_refused() {
        assert_eq!(
            Isometry::rotation(&Axis::z(), f64::NAN),
            Err(Error::NotFinite {
                what: "rotation angle"
            })
        );
    }

    #[test]
    fn a_stored_movement_with_no_rotation_at_all_is_refused() {
        assert_eq!(
            Isometry::try_from(([0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0])),
            Err(Error::ZeroLength { what: "rotation" })
        );
    }

    #[test]
    fn a_stored_rotation_is_scaled_back_to_length_one_when_it_is_read() {
        let stretched = Isometry::try_from(([0.0, 0.0, 0.0], [2.0, 0.0, 0.0, 0.0])).unwrap();
        assert_eq!(stretched, Isometry::identity());
    }

    #[test]
    fn a_movement_survives_a_round_trip_through_its_stored_parts() {
        let movement = quarter_turn_about_z()
            .compose(&Isometry::translation(Vector::new(1.0, 2.0, 3.0)).unwrap());
        let stored: ([f64; 3], [f64; 4]) = movement.into();
        assert_eq!(Isometry::try_from(stored).unwrap(), movement);
    }

    /// The test [`Isometry::POINT_ROUNDOFF`]'s derivation would otherwise be
    /// without.
    ///
    /// `docs/decisions.md` §15.6 was amended because a bound that had a
    /// derivation written next to it was wrong anyway: nothing executed the
    /// derivation. So this runs the same expression a second time in exact
    /// expansion arithmetic, where no step rounds, and subtracts.
    ///
    /// **It cannot show the bound is tight.** Roundings cancel, so it passes at
    /// a smaller figure too: the worst of the 3072 coordinates checked here
    /// uses an eighth of the bound, and a sweep a thousand times longer only
    /// reaches a quarter of it. That is why the worst case is printed rather
    /// than asserted against. The derivation is what makes 18 a bound; this is
    /// what stops [`Isometry::apply_to_point`] drifting out from under it.
    ///
    /// The points run from a thousandth up to a million, because the bound is
    /// stated per unit of the largest coordinate and a bound only ever asked
    /// about coordinates near one is not being asked.
    #[test]
    fn a_moved_point_lands_inside_the_published_roundoff_of_the_exact_answer() {
        let mut worst = 0.0f64;
        let mut cases = 0;

        let mut numbers = Numbers::seeded();
        for _ in 0..64 {
            let movement = numbers.a_movement();
            for _ in 0..16 {
                let point = numbers.a_point();
                let largest = point
                    .largest_coordinate_magnitude()
                    .max(largest_of(movement.translation_part().components()));

                let computed = movement.apply_to_point(point).coordinates();
                let exact = moved_without_rounding(&movement, point);
                for axis in 0..3 {
                    let slipped = magnitude_of(&sum(&exact[axis], &entry(-computed[axis])));
                    assert!(
                        slipped <= Isometry::POINT_ROUNDOFF * largest,
                        "moving {point:?} slipped {slipped} on axis {axis}, \
                         which is past {} times the largest coordinate {largest}",
                        Isometry::POINT_ROUNDOFF
                    );
                    worst = worst.max(slipped / (Isometry::POINT_ROUNDOFF * largest));
                    cases += 1;
                }
            }
        }

        println!(
            "worst of {cases} coordinates used {:.1}% of the bound",
            worst * 100.0
        );
    }

    /// [`Isometry::apply_to_point`]'s expression with nothing rounded away, one
    /// expansion per coordinate.
    ///
    /// Term for term the same as `nalgebra`'s `UnitQuaternion * Point3`
    /// followed by `Translation * Point3`, which is what the derivation
    /// describes and what the bound is a bound on.
    fn moved_without_rounding(movement: &Isometry, point: Point) -> [Vec<f64>; 3] {
        let [w, i, j, k] = movement.rotation_quaternion();
        let turn = [i, j, k];
        let shift = movement.translation_part().components();
        let at = point.coordinates().map(entry);

        let across = cross(turn, &at);
        let twice_across = across.map(|component| scaled(&component, 2.0));
        let twisted = cross(turn, &twice_across);

        [0, 1, 2].map(|axis| {
            let leaning = scaled(&twice_across[axis], w);
            let turned = sum(&sum(&leaning, &twisted[axis]), &at[axis]);
            sum(&turned, &entry(shift[axis]))
        })
    }

    /// `left × right`, with `left` three plain numbers and `right` three
    /// expansions.
    fn cross(left: [f64; 3], right: &[Vec<f64>; 3]) -> [Vec<f64>; 3] {
        [[1, 2], [2, 0], [0, 1]].map(|[first, second]| {
            let forwards = scaled(&right[second], left[first]);
            let backwards = scaled(&right[first], left[second]);
            sum(&forwards, &negated(&backwards))
        })
    }

    /// One `f64` as an expansion. An exact zero carries no information and is
    /// dropped, which is what the rest of the expansion code does too.
    fn entry(value: f64) -> Vec<f64> {
        if value == 0.0 { vec![] } else { vec![value] }
    }

    /// The exact product of an expansion and a plain number.
    ///
    /// The exactness has a precondition — a product whose roundoff term falls
    /// into the subnormals cannot hold its own bits — and `product_is_exact` is
    /// where the expansion code states it. Asserted rather than assumed, so an
    /// oracle that has quietly stopped being exact fails loudly.
    fn scaled(expansion: &[f64], factor: f64) -> Vec<f64> {
        assert!(
            expansion.iter().all(|&e| product_is_exact(e, factor)),
            "the oracle only stays exact while its products do"
        );
        let mut out = vec![0.0; 2 * expansion.len()];
        let used = scale_expansion(expansion, factor, &mut out);
        out.truncate(used);
        out
    }

    fn sum(left: &[f64], right: &[f64]) -> Vec<f64> {
        let mut out = vec![0.0; left.len() + right.len()];
        let used = expansion_sum(left, right, &mut out);
        out.truncate(used);
        out
    }

    fn negated(expansion: &[f64]) -> Vec<f64> {
        expansion.iter().map(|entry| -entry).collect()
    }

    /// At least as much as the expansion is worth.
    ///
    /// The entries can be of either sign, so adding their magnitudes can only
    /// over-estimate — the safe direction both for a bound check and for the
    /// worst case it prints.
    fn magnitude_of(expansion: &[f64]) -> f64 {
        expansion.iter().map(|entry| entry.abs()).sum()
    }

    fn largest_of(components: [f64; 3]) -> f64 {
        components.into_iter().map(f64::abs).fold(0.0, f64::max)
    }

    /// The same movements and points every run, so the worst case this prints
    /// is a number that can be compared with the last one.
    struct Numbers(u64);

    impl Numbers {
        fn seeded() -> Self {
            Self(0x2545_f491_4f6c_dd1d)
        }

        /// A turn about an arbitrary axis, through an arbitrary point, followed
        /// by an arbitrary shift.
        fn a_movement(&mut self) -> Isometry {
            let direction = Vector::new(
                self.between_minus_one_and_one(),
                self.between_minus_one_and_one(),
                self.between_minus_one_and_one(),
            );
            let axis = Axis::new(self.a_point(), direction)
                .expect("three numbers from a fixed seed are not all exactly zero");
            let turn = Isometry::rotation(
                &axis,
                core::f64::consts::PI * self.between_minus_one_and_one(),
            )
            .expect("a finite angle about a finite axis");
            let shift = Isometry::translation(Vector::new(
                self.a_size() * self.between_minus_one_and_one(),
                self.a_size() * self.between_minus_one_and_one(),
                self.a_size() * self.between_minus_one_and_one(),
            ))
            .expect("a finite shift");
            shift.compose(&turn)
        }

        fn a_point(&mut self) -> Point {
            let size = self.a_size();
            Point::new(
                size * self.between_minus_one_and_one(),
                size * self.between_minus_one_and_one(),
                size * self.between_minus_one_and_one(),
            )
        }

        fn a_size(&mut self) -> f64 {
            [1e-3, 1.0, 1e3, 1e6][(self.next() >> 60) as usize % 4]
        }

        fn between_minus_one_and_one(&mut self) -> f64 {
            (self.next() >> 11) as f64 / (1u64 << 52) as f64 * 2.0 - 1.0
        }

        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            self.0
        }
    }
}
