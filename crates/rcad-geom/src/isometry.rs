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
/// cannot hand us a movement that also stretches the part.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "([f64; 3], [f64; 4])", into = "([f64; 3], [f64; 4])")]
pub struct Isometry(na::Isometry3<f64>);

impl Isometry {
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
    #[must_use]
    pub fn apply_to_point(&self, point: Point) -> Point {
        self.0.transform_point(&point.into()).into()
    }

    /// Which way `vector` ends up pointing.
    ///
    /// A vector is the difference between two points, so the shift cancels out
    /// and only the turn acts on it. Applying the translation here is the
    /// classic way to move a normal to the wrong place.
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

    use super::*;

    fn quarter_turn_about_z() -> Isometry {
        Isometry::rotation(&Axis::z(), FRAC_PI_2).unwrap()
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
}
