//! Rigid movement: a rotation followed by a translation.

use nalgebra as na;
use serde::{Deserialize, Serialize};

use crate::exact::{binary_exponent, multiplied_by_a_power_of_two};
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
/// places to within rounding, not bit for bit — **except a turn of a cube**,
/// which reads back as the very numbers it was written as (see
/// [`Isometry::quarter_turns`]).
///
/// The scale the four coefficients are written at makes no difference to the
/// rotation they name, and none to the one that comes back: they are brought
/// onto a power of two of their own first, so the squares the scaling back needs
/// can neither overflow nor underflow
/// (`brought_into_the_range_its_squares_fit_in`).
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
    /// `18 · f64::EPSILON · L`. The 0.6% left over was meant to cover the
    /// second-order terms — a rounding is a relative slip of the *computed*
    /// value, not of the true one — and the fact that a stored rotation has
    /// length one only to within rounding, without saying how far.
    ///
    /// Gappa proves the 18 (`docs/decisions.md` §464.19): each coordinate lands
    /// within `18 · f64::EPSILON · L` while the turn's `q·q`, worked out
    /// exactly, is within `6 · f64::EPSILON` of one, the nine products are
    /// normal numbers, no factor is zero and no sum cancels to zero. The count
    /// above charges each stage at its own worst case, and those cannot all
    /// happen at once: `|w| = 1` forces `v = 0`, and so `k = 0`. Charged
    /// together, the proof reaches `24.80·u·L`, which leaves `11.2·u·L` of room
    /// where the count left `0.22·u·L`. A turn of a cube is one addition per
    /// coordinate, within `2u·L`. The zero factors, the sums that cancel and
    /// the products below the normals have not been run.
    ///
    /// **A turn further than `6 · f64::EPSILON` from length one is outside the
    /// proof, and can go past the 18**: one four times as long as one put 40
    /// of 3072 coordinates past it, by up to 2.46 times
    /// (`a_turn_four_times_the_length_of_one_moves_points_past_the_published_roundoff`).
    /// So no route to a movement keeps such a turn. [`Isometry::compose`]
    /// divides a product that has drifted, a movement read from stored parts
    /// is divided, and so is one read from a nalgebra isometry whose turn is
    /// not near length one, since nalgebra's `Unit::new_unchecked` stores a
    /// quaternion of any length.
    ///
    /// Every step of the count is a relative argument, so it holds only while
    /// nothing overflows and nothing falls into the subnormals.
    ///
    /// # What this bounds, and what it does not
    ///
    /// The arithmetic, given the numbers this movement holds. It says nothing
    /// about how well those numbers match the movement a caller asked for:
    /// `cos(π/2)` in `f64` is `6.1e-17` rather than zero, so even a quarter turn
    /// built by [`Isometry::rotation`] arrives a rounding away from the turn that
    /// was named. [`Isometry::quarter_turns`] is how to name one exactly, and
    /// moves a point by one addition per coordinate. A caller deciding
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
    /// **`FRAC_PI_2` is not a quarter turn**: it is the `f64` nearest `π/2`,
    /// and the turn it names sends `z` to `(1, 0, 2⁻⁵²)` about `y` rather than
    /// to `x`. A shape that rests on the turn being exact — a bar turned onto
    /// `x` through a ball's centre — wants [`Isometry::quarter_turns`].
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

    /// `quarters` quarter turns about `axis`, right-handed about its direction,
    /// said exactly. Any number of quarters, negative ones turning the other
    /// way; four is no turn at all.
    ///
    /// ```
    /// use rcad_geom::{Axis, Isometry, Vector};
    ///
    /// let onto_x = Isometry::quarter_turns(&Axis::y(), 1)?;
    /// assert_eq!(onto_x.apply_to_vector(Vector::new(0.0, 0.0, 1.0)), Vector::new(1.0, 0.0, 0.0));
    /// # Ok::<(), rcad_geom::Error>(())
    /// ```
    ///
    /// **Exactly, where [`Isometry::rotation`] by `FRAC_PI_2` is not.** A turn
    /// about `x`, `y` or `z` by a whole number of quarters takes each
    /// coordinate onto another, so turning a point moves its coordinates about
    /// and negates some, and nothing rounds. Such a turn stays exact through
    /// [`Isometry::compose`] with another of its kind, through
    /// [`Isometry::inverse`], and through being written and read back.
    ///
    /// **Points on the axis stay where they are to within the one subtraction
    /// per coordinate the shift costs.** A turn about an axis through `o` is
    /// the turn about a parallel one through the origin and then the shift
    /// `o − turned(o)`, which rounds by at most `f64::EPSILON` times the
    /// largest coordinate of `o`. A point on the axis is turned exactly and
    /// lands on itself plus that slip, rounded, so within twice the slip of
    /// where it was.
    /// `a_point_on_an_axis_through_anywhere_stays_within_two_epsilon_of_itself`
    /// is the demonstration. Where the subtractions are exact — an axis through
    /// a point with small whole coordinates, say — the points on it stay where
    /// they are bit for bit.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if the axis is not made of finite numbers, and
    /// [`Error::QuarterTurnsNeedAnAxisAlongXYOrZ`] if its direction does not
    /// run exactly along `x`, `y` or `z`.
    pub fn quarter_turns(axis: &Axis, quarters: i32) -> Result<Self> {
        if !axis.origin().is_finite() || !axis.direction().is_finite() {
            return Err(Error::NotFinite { what: "axis" });
        }
        let direction = axis.direction().components();
        let mut along = direction
            .iter()
            .enumerate()
            .filter(|(_, component)| **component != 0.0);
        let (Some((which, component)), None) = (along.next(), along.next()) else {
            return Err(Error::QuarterTurnsNeedAnAxisAlongXYOrZ {
                direction: axis.direction(),
            });
        };
        let way = if *component > 0.0 { 1 } else { -1 };
        let mut signs = [0; 4];
        match quarters.rem_euclid(4) {
            0 => signs[0] = 1,
            1 => [signs[0], signs[1 + which]] = [1, way],
            2 => signs[1 + which] = way,
            3 => [signs[0], signs[1 + which]] = [1, -way],
            more => unreachable!("rem_euclid(4) gave {more}"),
        }
        let turn = TurnOfACube(signs);
        let through = na::Vector3::from(axis.origin().coordinates());
        Ok(Self(na::Isometry3::from_parts(
            na::Translation3::from(turn.turned(-through) + through),
            turn.as_stored(),
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
    ///
    /// Two turns of a cube compose to the turn of a cube they make, worked out
    /// on the signs of their coefficients and stored the one way that turn is
    /// always stored. Multiplying the stored coefficients would round, and a
    /// product that came out with its coefficients no longer all of one size
    /// would no longer be turned exactly.
    ///
    /// Any other two turns are multiplied as quaternions, which rounds, and a
    /// product whose squared length has drifted more than `4ε` from one is
    /// divided by its length (`held_near_length_one`). Without that
    /// a part turned again and again — `rcad-topo`'s `Model::place_solid`
    /// composes each call's movement onto the one the part already has — was
    /// turned by a quaternion a little longer or shorter each time, which
    /// stretches what it turns.
    #[must_use]
    pub fn compose(&self, inner: &Isometry) -> Isometry {
        let turn = if let (Some(outer), Some(first)) = (
            TurnOfACube::named_by(&self.0.rotation),
            TurnOfACube::named_by(&inner.0.rotation),
        ) {
            outer.after(first).as_stored()
        } else {
            held_near_length_one(self.0.rotation * inner.0.rotation)
        };
        let shift =
            self.0.translation.vector + turned_by(&self.0.rotation, inner.0.translation.vector);
        turn_then_shift(turn, shift)
    }

    /// The movement that undoes this one.
    #[must_use]
    pub fn inverse(&self) -> Isometry {
        let undoing = self.0.rotation.inverse();
        turn_then_shift(undoing, turned_by(&undoing, -self.0.translation.vector))
    }

    /// Where `point` ends up.
    ///
    /// Within [`Isometry::POINT_ROUNDOFF`] times the largest number involved of
    /// where the true movement would put it.
    ///
    /// **A turn of a cube does better than that, and something rests on it.**
    /// It moves each coordinate onto another and negates some, which rounds
    /// nothing, so the whole movement comes to one addition per coordinate. No
    /// turn at all is one of those, so a shift a caller has arranged to be
    /// exact — where Sterbenz's condition holds, say — really does come out
    /// exact. `rcad-topo`'s mesher rests the whole of the frame it works a
    /// far-off face out in on that (§39.7 item 2), and
    /// `a_movement_with_no_turn_moves_a_point_by_one_addition_and_nothing_else`
    /// is the demonstration rather than this sentence.
    #[must_use]
    pub fn apply_to_point(&self, point: Point) -> Point {
        let at: na::Point3<f64> = point.into();
        Point::from(na::Point3::from(
            turned_by(&self.0.rotation, at.coords) + self.0.translation.vector,
        ))
    }

    /// Which way `vector` ends up pointing.
    ///
    /// A vector is the difference between two points, so the shift cancels out
    /// and only the turn acts on it. Applying the translation here is the
    /// classic way to move a normal to the wrong place.
    ///
    /// A turn of a cube hands the vector back with its components moved about
    /// and some negated, bit for bit, and a movement with no turn hands it
    /// straight back — same demonstration as [`Isometry::apply_to_point`]. It
    /// is what keeps a shifted pipe's axis direction and a shifted plane's
    /// normal exactly what they were, and a bar turned a quarter turn onto `x`
    /// pointing exactly along it.
    #[must_use]
    pub fn apply_to_vector(&self, vector: Vector) -> Vector {
        turned_by(&self.0.rotation, vector.into()).into()
    }

    /// The shift part, applied after the turn.
    #[must_use]
    pub fn translation_part(&self) -> Vector {
        self.0.translation.vector.into()
    }

    /// The turn part, as `[w, i, j, k]`. Of length one to within rounding.
    ///
    /// **`glam`'s `DQuat::from_xyzw` wants those four in exactly the opposite
    /// order.** Handing it this array as it stands compiles, runs, and builds
    /// another turn of length one — a *different* turn, with nothing in the
    /// types, the arithmetic or the drawing to say so — so the solid comes out
    /// at the wrong angle and looks entirely plausible there. It wants
    /// `[i, j, k, w]`.
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

/// A nalgebra isometry, with its turn held near length one as
/// [`Isometry::compose`] holds its own.
///
/// nalgebra's `Unit::new_unchecked` stores a quaternion of any length, and a
/// turn longer or shorter than one stretches what it turns and is outside what
/// [`Isometry::POINT_ROUNDOFF`] is proved for. So a turn that has not drifted
/// past `TURN_LENGTH_DRIFT` keeps its bits, and any other is read as stored
/// parts are: brought into range and divided by its length, so no length is
/// too long or too short to read. Either way a turn of a cube is stored the one
/// way every route stores it. So the movements this file builds come back
/// from nalgebra bit for bit as they went
/// (`a_movement_handed_to_nalgebra_and_back_keeps_its_bits`), and a turn of any
/// other length comes back within `6 · f64::EPSILON` of length one
/// (`a_turn_from_nalgebra_off_length_one_is_held_within_six_epsilon_of_it`).
///
/// # Errors
///
/// [`Error::NotFinite`] if the shift or the turn holds a number that is not
/// finite, and [`Error::ZeroLength`] if the turn is four zeros.
impl TryFrom<na::Isometry3<f64>> for Isometry {
    type Error = Error;

    fn try_from(movement: na::Isometry3<f64>) -> Result<Self> {
        let shift = movement.translation.vector;
        let turn = movement.rotation.into_inner();
        if sits_near_length_one(&turn) && Vector::from(shift).is_finite() {
            return Ok(turn_then_shift(movement.rotation, shift));
        }
        Self::try_from((
            Vector::from(shift).components(),
            [turn.w, turn.i, turn.j, turn.k],
        ))
    }
}

impl TryFrom<([f64; 3], [f64; 4])> for Isometry {
    type Error = Error;

    fn try_from((offset, stored): ([f64; 3], [f64; 4])) -> Result<Self> {
        let offset = Vector::from(offset);
        if !offset.is_finite() {
            return Err(Error::NotFinite {
                what: "translation",
            });
        }
        if !stored.iter().all(|coefficient| coefficient.is_finite()) {
            return Err(Error::NotFinite { what: "rotation" });
        }
        if stored.iter().all(|coefficient| *coefficient == 0.0) {
            return Err(Error::ZeroLength { what: "rotation" });
        }
        let [w, i, j, k] = brought_into_the_range_its_squares_fit_in(stored);
        Ok(turn_then_shift(
            na::UnitQuaternion::from_quaternion(na::Quaternion::new(w, i, j, k)),
            offset.into(),
        ))
    }
}

/// `coefficients`, every one multiplied by the same power of two, chosen so the
/// largest of them lands in `[1, 2)`.
///
/// A quaternion names a rotation by the direction of its four coefficients, so
/// multiplying all four by one nonzero factor names the very same rotation, and
/// a power of two costs no bits while the results stay normal.
///
/// **Where all four squares were already normal numbers this hands back the
/// identical unit quaternion, bit for bit.** Every square scales by exactly
/// `2^2e`, so the sum does, so its square root scales by exactly `2^e`, and the
/// division cancels the factor again. *All four*, and not merely their sum: one
/// square can fall into the subnormals — or underflow to zero outright — while
/// the sum is an ordinary number, and a norm built from a square that has lost
/// bits is not the norm. There this differs, by a single ulp, and it is the
/// unscaled answer that was wrong. **The first draft of this sentence said "the
/// squares were already in range" and the demonstration is what narrowed it**;
/// `scaling_first_moves_no_answer_whose_four_squares_were_normal_to_begin_with`
/// counts all three outcomes and asserts a floor under each.
///
/// **Where the sum itself was out of range, this is what reaches the answer at
/// all.** The sum of
/// the four squares spans twice the range the coefficients do. Written above
/// about `1.3e154` it overflows, `norm()` is infinity, dividing by it gives four
/// zeros, and four zeros normalize to the *identity* — a rotation the caller's
/// bytes did not name, which `docs/decisions.md` §4.1 ranks below both crashing
/// and refusing. Written below about `1.5e-162` the same sum underflows to zero
/// and the rotation is refused for having no length. Brought into `[1, 2)` the
/// largest square is between 1 and 4 and neither end is anywhere near.
///
/// **What the scaling can lose, and why it is nothing.** A coefficient more than
/// 1022 binades under the largest is pushed into the subnormals and rounds
/// there, by at most half of `2^-1074`. It is that far under the norm as well,
/// so the number it stands for in the unit quaternion is itself under
/// `2^-1022` — dividing by a norm that is now between 1 and 2 keeps the slip
/// where it was, and half of the smallest number an `f64` holds is the closest
/// any answer gets. So no quaternion is refused for being unscalable: the
/// refusals left above are four zeros and a coefficient that is not a number,
/// and both are bytes that name no rotation for any arithmetic to find.
///
/// `a_rotation_written_at_any_scale_is_the_same_rotation` is the demonstration,
/// over every one of the 2098 scales an `f64` exponent field holds.
fn brought_into_the_range_its_squares_fit_in(coefficients: [f64; 4]) -> [f64; 4] {
    let largest = coefficients
        .iter()
        .copied()
        .map(f64::abs)
        .fold(0.0, f64::max);
    assert!(
        largest > 0.0,
        "a quaternion of four zeros is refused before it gets here"
    );
    let exponent = -binary_exponent(largest);
    coefficients.map(|coefficient| multiplied_by_a_power_of_two(coefficient, exponent))
}

/// How far from one a composed turn's squared length `q·q` may sit, as `f64`
/// works it out, before [`Isometry::compose`] divides the turn by its length —
/// and a turn read from a nalgebra isometry, before it is divided too.
///
/// **Not lower, so that a turn which has not drifted keeps its bits.** A turn
/// straight from [`Isometry::rotation`] sits a few `ε` off by that sum — at
/// most `2ε` over the 100 000 draws of
/// `a_turn_straight_from_rotation_is_never_divided_though_dividing_would_move_one_in_five`,
/// and `3ε` over the 100 000 of a probe (`docs/decisions.md` §355.7) — and
/// dividing would move the last bit of about one in five of them.
/// At `4ε` no product of two, three or four such turns is divided either, in
/// the 200 chains
/// `a_turn_composed_of_any_number_of_turns_stays_within_six_epsilon_of_length_one`
/// draws, so a part placed a few times holds the very numbers it held before
/// [`Isometry::compose`] divided anything.
///
/// **Not higher, so that a turned normal stays well inside
/// [`Plane::NORMAL_LENGTH_ROUNDOFF`](crate::Plane::NORMAL_LENGTH_ROUNDOFF).**
/// A kept turn is within `6ε` of length one and a divided one within about
/// `4ε` ([`held_near_length_one`] says why), and the worst normal
/// `every_normal_a_turn_leaves_sits_well_inside_the_bound` turns through 16
/// or 256 composed turns sits about `22ε` off length one, under half of the
/// `64ε` that bound allows.
const TURN_LENGTH_DRIFT: f64 = 4.0 * f64::EPSILON;

/// `turn`, with its own bits while `q·q`, as `f64` works it out, is within
/// [`TURN_LENGTH_DRIFT`] of one, and divided by its length past that.
///
/// **Why it divides at all.** Multiplying two quaternions rounds, and nothing
/// else brings the product back to length one, so a turn composed of many
/// turns walked further off with each one, and a plane's normal turned by it
/// walked past [`Plane::NORMAL_LENGTH_ROUNDOFF`](crate::Plane::NORMAL_LENGTH_ROUNDOFF).
/// Such a normal is scaled when it is read back, so a plane turned a degree at
/// a time came back from its file an ulp moved, the first after 63 steps
/// (`a_plane_turned_a_step_at_a_time_reads_back_with_the_normal_it_was_written_with`,
/// `docs/decisions.md` §355.7).
///
/// **Why it does not always divide.** Dividing rounds too. It would move the
/// last bit of turns that sat as near length one as a division leaves them —
/// one in five turns straight from [`Isometry::rotation`], and with them every
/// placement built as a shift after a turn — and so move answers far from here
/// for nothing.
///
/// **What it holds a turn to: `|q·q − 1| ≤ 6ε`, worked out exactly.** A kept
/// turn has its `f64` sum of four squares within `4ε` of one, and that sum is
/// within `γ₄·q·q`, about `2ε`, of the exact one. A divided turn carries the
/// sum's rounding, about `2ε`, then the square root's half ulp and the
/// division's half ulp, each counted twice in a square: about `4ε` in all.
/// `a_turn_composed_of_any_number_of_turns_stays_within_six_epsilon_of_length_one`
/// works `q·q` out without rounding over 200 chains of 256 turns.
fn held_near_length_one(turn: na::UnitQuaternion<f64>) -> na::UnitQuaternion<f64> {
    if sits_near_length_one(turn.quaternion()) {
        return turn;
    }
    na::UnitQuaternion::new_normalize(turn.into_inner())
}

/// Whether `turn`'s `q·q`, as `f64` works it out, is within
/// [`TURN_LENGTH_DRIFT`] of one. A turn holding a number that is not finite is
/// not.
fn sits_near_length_one(turn: &na::Quaternion<f64>) -> bool {
    (turn.norm_squared() - 1.0).abs() <= TURN_LENGTH_DRIFT
}

/// The movement that turns by `turn` and then shifts by `shift`, with a turn of
/// a cube stored the one way every route to it stores it.
///
/// So the same turn of a cube compares equal however it was reached — built,
/// composed, undone, or read back from a file that wrote its coefficients at
/// some other size — and is written back out as the numbers it was read as.
/// Nothing else is touched: a turn that is not one of the 24 keeps the bits it
/// came with.
fn turn_then_shift(turn: na::UnitQuaternion<f64>, shift: na::Vector3<f64>) -> Isometry {
    let turn = TurnOfACube::named_by(&turn).map_or(turn, TurnOfACube::as_stored);
    Isometry(na::Isometry3::from_parts(
        na::Translation3::from(shift),
        turn,
    ))
}

/// `vector` turned by `turn`: as the coordinate swap it is when `turn` is a
/// turn of a cube, and through Rodrigues' form otherwise.
fn turned_by(turn: &na::UnitQuaternion<f64>, vector: na::Vector3<f64>) -> na::Vector3<f64> {
    match TurnOfACube::named_by(turn) {
        Some(swap) => swap.turned(vector),
        None => turn.transform_vector(&vector),
    }
}

/// One of the 24 turns that take a cube onto itself, named by the signs of the
/// four coefficients `[w, i, j, k]` of a quaternion that names it.
///
/// A quaternion names a turn by the direction of its four coefficients, so
/// coefficients that are one, two or four non-zero numbers, all of one size,
/// name one of these turns exactly whatever that size is and however it was
/// rounded: the size divides out. The 48 sign patterns of that shape are the 24
/// turns, each named twice, by `q` and by `−q`.
///
/// **Why the turn is not left to Rodrigues' form.** Each of these sends every
/// axis onto an axis, so it turns a vector by moving its components about and
/// negating some, which rounds nothing. Rodrigues' form rounds, and for a
/// quarter turn about `y` it cannot help it: `z` turns to `1 − fl(j·2j)` along
/// `z`, where `j` is the stored `j` coefficient, and no `f64` `j` has
/// `fl(j·2j) = 1`. So a bar turned onto `x` came out leaning `2⁻⁵²` off it, and
/// missed the centre of a ball it was meant to pass through (Q1245).
/// `rodrigues_form_cannot_turn_z_exactly_onto_x_whatever_the_stored_numbers` is
/// the demonstration of the "no `j`".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TurnOfACube([i8; 4]);

impl TurnOfACube {
    /// The turn of a cube `turn`'s stored coefficients name, if they name one.
    fn named_by(turn: &na::UnitQuaternion<f64>) -> Option<Self> {
        let stored = turn.quaternion();
        let coefficients = [stored.w, stored.i, stored.j, stored.k];
        let size = coefficients
            .iter()
            .copied()
            .map(f64::abs)
            .fold(0.0, f64::max);
        let non_zero = coefficients.iter().filter(|c| **c != 0.0).count();
        let all_one_size = coefficients.iter().all(|c| *c == 0.0 || c.abs() == size);
        (matches!(non_zero, 1 | 2 | 4) && all_one_size).then(|| {
            Self(coefficients.map(|c| {
                if c > 0.0 {
                    1
                } else if c < 0.0 {
                    -1
                } else {
                    0
                }
            }))
        })
    }

    /// `self` after `first`: the product of the two sign quaternions, which is
    /// a whole-number quaternion whose non-zero coefficients are again one,
    /// two or four of one size.
    fn after(self, first: Self) -> Self {
        let [a, b, c, d] = self.0.map(i32::from);
        let [e, f, g, h] = first.0.map(i32::from);
        let product = [
            a * e - b * f - c * g - d * h,
            a * f + b * e + c * h - d * g,
            a * g - b * h + c * e + d * f,
            a * h + b * g - c * f + d * e,
        ];
        let size = product.iter().map(|c| c.abs()).max().unwrap_or(0);
        let non_zero = product.iter().filter(|c| **c != 0).count();
        assert!(
            matches!(non_zero, 1 | 2 | 4) && product.iter().all(|c| *c == 0 || c.abs() == size),
            "two turns of a cube made {product:?}, which is not a turn of a cube"
        );
        Self(product.map(|c| match c.signum() {
            1 => 1,
            -1 => -1,
            _ => 0,
        }))
    }

    /// For each component of a turned vector, which component of the vector
    /// it is and whether it is negated.
    ///
    /// The turn's matrix is the usual one of a quaternion, written on its sign
    /// pattern and divided by the pattern's own squared length, so every entry
    /// is a whole number and every row has exactly one that is not zero.
    fn rows(self) -> [(usize, bool); 3] {
        let [a, b, c, d] = self.0.map(i32::from);
        let length_squared = a * a + b * b + c * c + d * d;
        let matrix = [
            [
                a * a + b * b - c * c - d * d,
                2 * (b * c - a * d),
                2 * (b * d + a * c),
            ],
            [
                2 * (b * c + a * d),
                a * a - b * b + c * c - d * d,
                2 * (c * d - a * b),
            ],
            [
                2 * (b * d - a * c),
                2 * (c * d + a * b),
                a * a - b * b - c * c + d * d,
            ],
        ];
        matrix.map(|row| {
            let mut entries = row.iter().enumerate().filter(|(_, entry)| **entry != 0);
            let (Some((from, entry)), None) = (entries.next(), entries.next()) else {
                panic!("the turn of a cube {self:?} has the row {row:?}, which is not one axis");
            };
            assert_eq!(
                entry.abs(),
                length_squared,
                "the turn of a cube {self:?} stretches along a row"
            );
            (from, *entry < 0)
        })
    }

    /// `vector` turned, with nothing rounded.
    fn turned(self, vector: na::Vector3<f64>) -> na::Vector3<f64> {
        let [x, y, z] = self.rows().map(
            |(from, negated)| {
                if negated { -vector[from] } else { vector[from] }
            },
        );
        na::Vector3::new(x, y, z)
    }

    /// The coefficients this turn is stored as: its signs times `1`, `1/√2` or
    /// `1/2` for one, two or four of them, the `f64` nearest the size that
    /// makes the four a unit quaternion.
    fn as_stored(self) -> na::UnitQuaternion<f64> {
        let size = match self.0.iter().filter(|sign| **sign != 0).count() {
            1 => 1.0,
            2 => core::f64::consts::FRAC_1_SQRT_2,
            4 => 0.5,
            other => unreachable!("a turn of a cube has {other} non-zero coefficients"),
        };
        let [w, i, j, k] = self.0.map(|sign| f64::from(sign) * size);
        na::UnitQuaternion::new_unchecked(na::Quaternion::new(w, i, j, k))
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
    use crate::exact::Exact;

    fn quarter_turn_about_z() -> Isometry {
        Isometry::rotation(&Axis::z(), FRAC_PI_2).unwrap()
    }

    /// §16.10 for [`Isometry::apply_to_point`]'s "one addition per coordinate
    /// and nothing else".
    ///
    /// **It was a claim about a dependency** until no turn at all went through
    /// [`TurnOfACube`] like the other 23, and a claim that rests on which road
    /// the identity takes is one an edit elsewhere can break. `rcad-topo`'s
    /// mesher shifts a face stored far from the origin into a frame where its
    /// numbers are small, and the whole reason that is allowed is that the
    /// shift is *exact*; a turn of the identity that cost a rounding would make
    /// it inexact and the mesh floor would be understating what a point costs.
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

    /// §15.6 for [`held_near_length_one`]: the turn [`Isometry::compose`] hands
    /// back has `|q·q − 1|` at most `6ε` however many turns went into it, and a
    /// product that had not drifted past [`TURN_LENGTH_DRIFT`] keeps the bits
    /// the quaternion product gave it.
    ///
    /// `6ε` is the drift plus what the `f64` sum of four squares can be off by,
    /// about `2ε`, for a product that is kept; one that is divided by its
    /// length comes out within about `4ε`. `|q·q − 1|` is worked out exactly
    /// here, so the check does not share the rounding it checks.
    ///
    /// The floor on how many were divided is §14.7's: without it a change that
    /// never divided would leave the second half tested on nothing.
    #[test]
    fn a_turn_composed_of_any_number_of_turns_stays_within_six_epsilon_of_length_one() {
        let mut numbers = Numbers::seeded();
        let mut worst = 0.0_f64;
        let mut divided_at_depth = [0_u32; 4];
        let chains = 200;
        for _ in 0..chains {
            let mut placed = numbers.a_movement();
            for depth in 2..=256 {
                let turn = numbers.a_movement();
                let product = turn.0.rotation * placed.0.rotation;
                placed = turn.compose(&placed);

                let drifted = (product.norm_squared() - 1.0).abs() > TURN_LENGTH_DRIFT;
                let product = product.quaternion();
                let kept =
                    placed.rotation_quaternion() == [product.w, product.i, product.j, product.k];
                assert!(
                    drifted || kept,
                    "composing changed the bits of {product:?}, which had not drifted"
                );
                if drifted {
                    divided_at_depth[(depth - 2).min(3)] += 1;
                }
                let off_one = off_length_one_in_epsilons(placed.rotation_quaternion()).abs();
                assert!(
                    off_one <= 6.0,
                    "a turn composed of {depth} turns sits {off_one}ε off length one"
                );
                worst = worst.max(off_one);
            }
        }
        let [second, third, fourth, later] = divided_at_depth;
        println!(
            "of {chains} chains of 256 turns, divided by its length at the second turn {second}, \
             the third {third}, the fourth {fourth}, later {later}; worst |q·q − 1| {worst:.2}ε"
        );
        assert!(
            later > 1_000,
            "only {later} composed turns were divided by their length, so that half is barely tested"
        );
    }

    /// The floor under [`TURN_LENGTH_DRIFT`]: a turn straight from
    /// [`Isometry::rotation`], with a shift composed after it the way a
    /// placement is built, keeps the turn's bits, where dividing every product
    /// by its length would move about one in five.
    ///
    /// The floor on how many dividing would move is §14.7's: without it, the
    /// keeping would be tested on turns that dividing leaves alone anyway.
    #[test]
    fn a_turn_straight_from_rotation_is_never_divided_though_dividing_would_move_one_in_five() {
        let shift = Isometry::translation(Vector::new(1.0, 2.0, 3.0)).expect("a finite shift");
        let mut numbers = Numbers::seeded();
        let draws = 100_000;
        let mut dividing_would_move = 0;
        let mut worst = 0.0_f64;
        for _ in 0..draws {
            let direction = Vector::new(
                numbers.between_minus_one_and_one(),
                numbers.between_minus_one_and_one(),
                numbers.between_minus_one_and_one(),
            );
            let axis = Axis::new(numbers.a_point(), direction)
                .expect("three numbers from a fixed seed are not all exactly zero");
            let turn = Isometry::rotation(
                &axis,
                core::f64::consts::TAU * numbers.between_minus_one_and_one(),
            )
            .expect("a finite angle about a finite axis");

            let placed = shift.compose(&turn);
            assert_eq!(
                placed.rotation_quaternion().map(f64::to_bits),
                turn.rotation_quaternion().map(f64::to_bits),
                "composing a shift after {turn:?} changed the turn's bits"
            );

            let stored = turn.0.rotation.into_inner();
            worst = worst.max((stored.norm_squared() - 1.0).abs() / f64::EPSILON);
            let divided = na::UnitQuaternion::new_normalize(stored).into_inner();
            if divided.coords.map(f64::to_bits) != stored.coords.map(f64::to_bits) {
                dividing_would_move += 1;
            }
        }
        println!(
            "of {draws} turns straight from rotation, dividing would move {dividing_would_move}; \
             worst |fl(q·q) − 1| {worst:.2}ε"
        );
        assert!(
            dividing_would_move > draws / 10,
            "dividing would move only {dividing_would_move} of {draws}, so keeping their bits is \
             barely tested"
        );
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

    /// §464.19's proof of [`Isometry::POINT_ROUNDOFF`] needs the turn's `q·q`
    /// within `6ε` of one, and nalgebra's `Unit::new_unchecked` stores a
    /// quaternion of any length. So a movement read from nalgebra has its turn
    /// held to length one, as [`Isometry::compose`] holds its own.
    ///
    /// Here the turns of 64 movements, made longer or shorter by a part in a
    /// billion and by factors up to `2^600`, come back within `6ε` of length
    /// one, worked out exactly, and move every point within the bound of where
    /// their own numbers put it.
    #[test]
    fn a_turn_from_nalgebra_off_length_one_is_held_within_six_epsilon_of_it() {
        let mut numbers = Numbers::seeded();
        for scale in [
            1.0 + 2f64.powi(-30),
            1.0 - 2f64.powi(-30),
            2.0,
            4.0,
            1e3,
            2f64.powi(-600),
            2f64.powi(600),
        ] {
            for _ in 0..64 {
                let unit = numbers.a_movement();
                let read =
                    read_from_nalgebra(unit.0.translation, unit.0.rotation.into_inner() * scale)
                        .expect("a finite turn that is not four zeros names a rotation");
                let off_one = off_length_one_in_epsilons(read.rotation_quaternion()).abs();
                assert!(
                    off_one <= 6.0,
                    "the turn of {unit:?} made {scale} times as long was read as one \
                     {off_one}ε off length one"
                );

                for _ in 0..16 {
                    let point = numbers.a_point();
                    let largest = point
                        .largest_coordinate_magnitude()
                        .max(largest_of(read.translation_part().components()));
                    let computed = read.apply_to_point(point).coordinates();
                    let exact = moved_without_rounding(&read, point);
                    for axis in 0..3 {
                        let slipped = magnitude_of(&sum(&exact[axis], &entry(-computed[axis])));
                        assert!(
                            slipped <= Isometry::POINT_ROUNDOFF * largest,
                            "{read:?}, read from a turn {scale} times as long as {unit:?}'s, \
                             moved {point:?} {slipped} off on axis {axis}, past {} times the \
                             largest coordinate {largest}",
                            Isometry::POINT_ROUNDOFF
                        );
                    }
                }
            }
        }
    }

    /// §16.10 for [`Isometry::POINT_ROUNDOFF`]'s "a turn further than `6ε`
    /// from length one can go past it", and the reason every route to a movement
    /// holds its turn near length one: a turn four times as long as one, so
    /// `q·q = 16`, moves points past the bound of where its own numbers put
    /// them.
    ///
    /// The movement is built around every route, straight from the stored
    /// nalgebra isometry, since no route keeps such a turn.
    #[test]
    fn a_turn_four_times_the_length_of_one_moves_points_past_the_published_roundoff() {
        let mut numbers = Numbers::seeded();
        let (mut past, mut cases, mut worst) = (0, 0, 0.0f64);
        for _ in 0..64 {
            let unit = numbers.a_movement();
            let stretched = Isometry(na::Isometry3::from_parts(
                unit.0.translation,
                na::Unit::new_unchecked(unit.0.rotation.into_inner() * 4.0),
            ));
            for _ in 0..16 {
                let point = numbers.a_point();
                let largest = point
                    .largest_coordinate_magnitude()
                    .max(largest_of(stretched.translation_part().components()));
                let computed = stretched.apply_to_point(point).coordinates();
                let exact = moved_without_rounding(&stretched, point);
                for axis in 0..3 {
                    let slipped = magnitude_of(&sum(&exact[axis], &entry(-computed[axis])));
                    let used = slipped / (Isometry::POINT_ROUNDOFF * largest);
                    worst = worst.max(used);
                    past += usize::from(used > 1.0);
                    cases += 1;
                }
            }
        }
        println!(
            "a turn four times as long as one put {past} of {cases} coordinates past the bound, \
             the worst at {worst:.2} times it"
        );
        assert!(
            past > 0,
            "no coordinate went past the bound, so it is not shown that a turn this far from \
             length one needs holding to it"
        );
    }

    /// The other half of holding a turn read from nalgebra as
    /// [`Isometry::compose`] holds its own: a turn that had not drifted past
    /// [`TURN_LENGTH_DRIFT`] keeps its bits. So every movement this file
    /// builds — turned, composed 64 deep, turned by quarters, undone, and read
    /// back from its stored parts, which divides its turn by its length —
    /// comes back from nalgebra as the very movement it was.
    #[test]
    fn a_movement_handed_to_nalgebra_and_back_keeps_its_bits() {
        let mut numbers = Numbers::seeded();
        let mut movements = Vec::new();
        for _ in 0..64 {
            let mut placed = numbers.a_movement();
            movements.push(placed);
            for _ in 0..64 {
                placed = numbers.a_movement().compose(&placed);
                movements.push(placed);
            }
        }
        for axis in [Axis::x(), Axis::y(), Axis::z()] {
            for quarters in 0..4 {
                movements.push(
                    Isometry::quarter_turns(&axis, quarters).expect("an axis along x, y or z"),
                );
            }
        }
        let undone: Vec<_> = movements.iter().map(Isometry::inverse).collect();
        let reread: Vec<_> = movements
            .iter()
            .map(|movement| {
                Isometry::try_from(<([f64; 3], [f64; 4])>::from(*movement))
                    .expect("a movement's own stored parts name a movement")
            })
            .collect();
        movements.extend(undone);
        movements.extend(reread);

        for movement in movements {
            let back = read_from_nalgebra(movement.0.translation, movement.0.rotation.into_inner())
                .expect("a movement this file built names a movement");
            assert_eq!(
                back.rotation_quaternion().map(f64::to_bits),
                movement.rotation_quaternion().map(f64::to_bits),
                "{movement:?} came back from nalgebra as {back:?}"
            );
            assert_eq!(
                back.translation_part().components().map(f64::to_bits),
                movement.translation_part().components().map(f64::to_bits),
                "{movement:?} came back from nalgebra as {back:?}"
            );
        }
    }

    /// A nalgebra isometry can hold numbers that name no movement, and those
    /// are refused as the same numbers read from stored parts are.
    #[test]
    fn a_movement_from_nalgebra_whose_numbers_name_no_movement_is_refused() {
        let shift = na::Translation3::new(1.0, 2.0, 3.0);
        let no_turn = na::Quaternion::new(1.0, 0.0, 0.0, 0.0);
        let refusal = |error: Error| Err(error.to_string());

        assert_eq!(
            read_from_nalgebra(shift, na::Quaternion::new(0.0, 0.0, 0.0, 0.0)),
            refusal(Error::ZeroLength { what: "rotation" })
        );
        assert_eq!(
            read_from_nalgebra(shift, na::Quaternion::new(1.0, f64::NAN, 0.0, 0.0)),
            refusal(Error::NotFinite { what: "rotation" })
        );
        assert_eq!(
            read_from_nalgebra(shift, na::Quaternion::new(f64::INFINITY, 0.0, 0.0, 0.0)),
            refusal(Error::NotFinite { what: "rotation" })
        );
        assert_eq!(
            read_from_nalgebra(na::Translation3::new(f64::INFINITY, 0.0, 0.0), no_turn),
            refusal(Error::NotFinite {
                what: "translation"
            })
        );
        assert_eq!(
            read_from_nalgebra(na::Translation3::new(0.0, f64::NAN, 0.0), no_turn * 2.0),
            refusal(Error::NotFinite {
                what: "translation"
            })
        );
    }

    /// The movement nalgebra's `shift` and `turn`, the turn stored as it comes
    /// by `Unit::new_unchecked`, are read as, with a refusal as its message.
    fn read_from_nalgebra(
        shift: na::Translation3<f64>,
        turn: na::Quaternion<f64>,
    ) -> core::result::Result<Isometry, String> {
        Isometry::try_from(na::Isometry3::from_parts(
            shift,
            na::Unit::new_unchecked(turn),
        ))
        .map_err(|refusal| refusal.to_string())
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

    /// `q·q − 1` for the turn `[w, i, j, k]`, worked out without rounding and
    /// then given in units of `f64::EPSILON`, so a check of it does not share
    /// the rounding it checks.
    fn off_length_one_in_epsilons(turn: [f64; 4]) -> f64 {
        let squared_length = turn.iter().fold(Exact::of(0.0), |sum, coefficient| {
            sum.plus(
                &Exact::product_of(*coefficient, *coefficient)
                    .expect("a coefficient of a turn squares exactly"),
            )
        });
        squared_length.minus(&Exact::of(1.0)).nearest_f64() / f64::EPSILON
    }

    /// §16.10 for [`brought_into_the_range_its_squares_fit_in`]'s claim that
    /// where the squares were already in range it hands back the identical unit
    /// quaternion, bit for bit.
    ///
    /// **The precondition is narrower than it first reads, and the first way of
    /// writing this test found that out.** "The squares were in range" is not
    /// "the sum of them was a normal number": a *single* square can fall into
    /// the subnormals while the sum is fine, and a square down there has lost
    /// bits, so the norm built from it is not the norm. There the rescaled
    /// answer differs — by one ulp, on quaternions whose coefficients are more
    /// than about 150 binades apart — and it is the unscaled one that was wrong.
    ///
    /// So the run counts three outcomes and asserts a floor under each. Without
    /// the floors a change that made every draw overflow would leave the
    /// bit-for-bit half tested on nothing, which is §14.7's rule.
    #[test]
    fn scaling_first_moves_no_answer_whose_four_squares_were_normal_to_begin_with() {
        let mut seed = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let (mut bit_for_bit, mut one_square_subnormal, mut out_of_range) = (0u32, 0u32, 0u32);
        for _ in 0..40_000 {
            // A middle to write the rotation at and a spread to write its four
            // coefficients over. Both are drawn, because the bit-for-bit half
            // needs quaternions the old arithmetic could handle, the boundary
            // half needs one square in the subnormals, and the third needs the
            // sum out of range altogether — and no single width reaches all
            // three.
            let middle = (next() % 800) as i32 - 400;
            let spread = [20, 200, 1200][(next() % 3) as usize];
            let coefficients = [0, 1, 2, 3].map(|_| {
                let mantissa = f64::from_bits((next() & ((1 << 52) - 1)) | (1023 << 52));
                let exponent = middle + (next() % spread) as i32 - spread as i32 / 2;
                let half = exponent / 2;
                let sign = if next() & 1 == 0 { 1.0 } else { -1.0 };
                sign * mantissa * 2f64.powi(half) * 2f64.powi(exponent - half)
            });
            if !coefficients.iter().all(|c| c.is_finite()) {
                continue;
            }
            let stored = na::Quaternion::new(
                coefficients[0],
                coefficients[1],
                coefficients[2],
                coefficients[3],
            );
            if stored.norm() == 0.0 {
                continue;
            }
            let without_the_rescale = na::UnitQuaternion::from_quaternion(stored);
            let without_the_rescale = {
                let turn = without_the_rescale.quaternion();
                [turn.w, turn.i, turn.j, turn.k]
            };
            let with_it = Isometry::try_from(([0.0; 3], coefficients))
                .expect("a finite quaternion that is not four zeros names a rotation")
                .rotation_quaternion();

            let squares = coefficients.map(|c| c * c);
            let every_square_is_normal =
                (0..4).all(|which| coefficients[which] == 0.0 || squares[which].is_normal());
            if !squares.iter().sum::<f64>().is_normal() {
                out_of_range += 1;
            } else if every_square_is_normal {
                assert_eq!(
                    without_the_rescale, with_it,
                    "the rescale moved {coefficients:?}, whose four squares are all normal"
                );
                bit_for_bit += 1;
            } else {
                one_square_subnormal += 1;
                for which in 0..4 {
                    let apart = i64::from(without_the_rescale[which].to_bits() as i32)
                        - i64::from(with_it[which].to_bits() as i32);
                    assert!(
                        without_the_rescale[which]
                            .to_bits()
                            .abs_diff(with_it[which].to_bits())
                            <= 1,
                        "{coefficients:?} component {which} moved {apart} bits"
                    );
                }
            }
        }
        println!(
            "bit for bit {bit_for_bit}, one square subnormal {one_square_subnormal}, \
             out of range {out_of_range}"
        );
        assert!(
            bit_for_bit > 10_000,
            "only {bit_for_bit} went down the path this is about"
        );
        assert!(
            one_square_subnormal > 500,
            "only {one_square_subnormal} reached the boundary, so it is not being pinned"
        );
        assert!(
            out_of_range > 1_000,
            "only {out_of_range} had a sum out of range, so the third outcome is barely drawn"
        );
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
