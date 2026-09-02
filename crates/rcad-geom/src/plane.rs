//! Flat surfaces.

use rcad_predicates::expansion::{expansion_sign, expansion_sum, product_is_exact, two_product};
use rcad_predicates::{
    Decision, Filtered, Interval, NOTHING_IS_KNOWN, Point as PredicatePoint, Predicate,
    are_real_numbers, decide, is_a_real_point, meet_of_three_planes, orient2d, product_is_normal,
};
use serde::{Deserialize, Serialize};

use crate::isometry::Isometry;
use crate::point::{Point, Vector};
use crate::{Error, Result};

/// The name the plane's sign predicate goes by in provenance records.
pub const SIDE_OF_PLANE: &str = "side of plane";

/// A flat surface: every point `p` on it satisfies `normal · p = offset`.
///
/// Every constructor scales the normal to length one, so
/// [`Plane::signed_distance_to`] is a real distance and not a scaled one.
/// [`Plane::transformed`] turns a normal without scaling it again — a read of a
/// plane has to give back the plane that was stored — so a moved plane's normal
/// has length one to within the rounding of the turn.
///
/// This is the only surface that takes part in topology in M0
/// (`docs/decisions.md` §3.1): a real cube, with volume, area and centroid
/// summed face by face from the divergence theorem.
///
/// Serializes as its four [`Plane::coefficients`], and is re-checked and scaled
/// back to a unit normal on the way in. Scaling a normal that is already very
/// nearly unit length can move the last bit or two, so a plane that has been
/// written and read back is the same plane to within rounding, not bit for bit.
///
/// # Signs here are arithmetic, not decisions
///
/// [`Plane::signed_distance_to`] is three multiplications and two additions.
/// Near the plane its sign is not trustworthy, and nothing in this crate is
/// allowed to branch on it. The certified "which side is this point on" answer
/// is a predicate returning a `Decision`, and it arrives with the predicate
/// cascade — [`Plane::coefficients`] is the shape it will consume, and it is
/// also what a three-plane intersection wants.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "[f64; 4]", into = "[f64; 4]")]
pub struct Plane {
    normal: Vector,
    offset: f64,
}

impl Plane {
    /// What rounding can cost [`Plane::signed_distance_to`], per unit of the
    /// largest coordinate it is handed.
    ///
    /// The expression is `normal · (point − origin) − offset`. The origin is
    /// exactly zero, so subtracting it rounds nothing; what is left is three
    /// products, two additions and a subtraction.
    ///
    /// Counting halves of an ulp does not give the answer, because a rounding
    /// costs half an ulp of the *running sum*, which is bigger than any one
    /// term. Track how many roundings each term passes through instead. With
    /// `u = f64::EPSILON / 2` and terms `tᵢ = normalᵢ · pointᵢ`: `t₁` and `t₂`
    /// are rounded on the way in and then survive two additions and the
    /// subtraction, so `4u` each; `t₃` joins later and takes `3u`; the offset
    /// only meets the subtraction, so `u`. That gives
    /// `4u·(|t₁| + |t₂| + |t₃|) + u·|offset|`.
    ///
    /// A unit normal has `|n₁| + |n₂| + |n₃| ≤ √3`, so the sum of the terms is
    /// at most `√3` times the largest coordinate. Against `L`, the largest of
    /// the coordinates and the offset, the whole thing comes to
    /// `u·(4√3 + 1)·L ≈ 3.97·u·2·L`, which is under `4 · f64::EPSILON · L`.
    ///
    /// This bounds the *reading* and nothing else. It says nothing about how
    /// well the stored plane matches the one a caller meant — a normal is
    /// scaled to unit length on the way in, and that rounds too. A caller
    /// deciding how far off a plane a point may sit wants this times a slack
    /// factor of its own, and that factor belongs at the caller with its own
    /// reason (`docs/decisions.md` §15.6).
    pub const SIGNED_DISTANCE_ROUNDOFF: f64 = 4.0 * f64::EPSILON;

    /// The plane through `point` at right angles to `normal`.
    ///
    /// Only the direction of `normal` is read. Two vectors along the same line
    /// give the same coefficients bit for bit however long each one is — see
    /// [`Plane::reduced_by_largest_component`].
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `point` is not made of finite numbers or if the
    /// plane would sit so far from the origin that its offset overflows, and
    /// [`Error::ZeroLength`] if `normal` has no length to point along.
    pub fn from_point_and_normal(point: Point, normal: Vector) -> Result<Self> {
        if !point.is_finite() {
            return Err(Error::NotFinite {
                what: "point on a plane",
            });
        }
        let (reduced, _) = Self::reduced_by_largest_component(normal, "plane normal")?;
        let normal = reduced.normalized_as("plane normal")?;
        let offset = normal.dot(point - Point::origin());
        if !offset.is_finite() {
            return Err(Error::NotFinite {
                what: "distance from the origin to a plane",
            });
        }
        Ok(Self { normal, offset })
    }

    /// The plane through three points, with the normal following the right-hand
    /// rule around `first`, `second`, `third`.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if any point is not made of finite numbers, and
    /// [`Error::PointsAreInALine`] if the three points give no plane at all.
    ///
    /// That last check only catches the cross product coming out exactly zero.
    /// Three points that are *nearly* in a line still give a plane here, and a
    /// badly conditioned one. Deciding "nearly" needs the exact orientation
    /// predicate, which is a `Decision` and arrives with the predicate cascade;
    /// picking a threshold here instead would be a second, quieter answer to the
    /// same question.
    pub fn through_three_points(first: Point, second: Point, third: Point) -> Result<Self> {
        for point in [first, second, third] {
            if !point.is_finite() {
                return Err(Error::NotFinite {
                    what: "point on a plane",
                });
            }
        }
        let normal = (second - first).cross(third - first);
        if normal == Vector::zero() {
            return Err(Error::PointsAreInALine);
        }
        Self::from_point_and_normal(first, normal)
    }

    /// The plane `a * x + b * y + c * z + d = 0`, from `[a, b, c, d]`.
    ///
    /// The coefficients are scaled so the normal has length one, which changes
    /// the numbers but not the plane. All four are divided by the largest of
    /// `|a|`, `|b|`, `|c|` first, so any two proportional coefficient sets give
    /// the same plane bit for bit — see
    /// [`Plane::reduced_by_largest_component`].
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if a coefficient is not a finite number, and
    /// [`Error::ZeroLength`] if `a`, `b` and `c` are all zero, which describes
    /// no plane.
    pub fn from_coefficients([a, b, c, d]: [f64; 4]) -> Result<Self> {
        if !d.is_finite() {
            return Err(Error::NotFinite {
                what: "plane coefficient",
            });
        }
        let (reduced, scale) =
            Self::reduced_by_largest_component(Vector::new(a, b, c), "plane normal")?;
        let length = reduced.length();
        let normal = reduced.normalized_as("plane normal")?;
        let offset = -(d / scale) / length;
        if !offset.is_finite() {
            return Err(Error::NotFinite {
                what: "distance from the origin to a plane",
            });
        }
        Ok(Self { normal, offset })
    }

    /// `direction` divided by its largest-magnitude component, and the positive
    /// number it was divided by.
    ///
    /// Every plane in the kernel comes through here, and the reason is that a
    /// face *is* its plane (`docs/decisions.md` §17.1): two walls that describe
    /// the same plane have to carry the same four numbers, or the fuse keeps
    /// whichever belonged to whichever argument came first. Normalizing alone
    /// does not give that. `(1, 1, 0)` and `(3, 3, 0)` are the same direction
    /// and their lengths round differently, so they used to store coefficients
    /// an ulp apart (§22.6).
    ///
    /// Dividing by the largest component first removes it. Division is
    /// correctly rounded, so `a/|m|` and `sa/|sm|` are the same real number and
    /// therefore the same `f64`, whatever `s` is. What comes out depends on the
    /// ratios between the components and on nothing else.
    ///
    /// It also widens what can be answered at all, in both directions, because
    /// every component that comes out is in `[-1, 1]` and the largest is
    /// exactly `±1`. The length that follows cannot overflow on a vector near
    /// [`f64::MAX`], and cannot underflow to zero on a subnormal one. Both were
    /// refused before, and both are §16.10 *gaps* rather than limits — the
    /// answer existed and this arithmetic reaches it.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if any component is infinite or not a number, and
    /// [`Error::ZeroLength`] if every component is exactly zero.
    fn reduced_by_largest_component(
        direction: Vector,
        what: &'static str,
    ) -> Result<(Vector, f64)> {
        if !direction.is_finite() {
            return Err(Error::NotFinite { what });
        }
        let [x, y, z] = direction.components();
        let scale = x.abs().max(y.abs()).max(z.abs());
        if scale == 0.0 {
            return Err(Error::ZeroLength { what });
        }
        Ok((Vector::new(x / scale, y / scale, z / scale), scale))
    }

    /// Which way the plane faces. Scaled to length one when the plane was built.
    #[must_use]
    pub fn normal(&self) -> Vector {
        self.normal
    }

    /// How far the plane sits from the origin, measured along the normal.
    ///
    /// Positive when the origin is on the back of the plane.
    #[must_use]
    pub fn offset(&self) -> f64 {
        self.offset
    }

    /// `[a, b, c, d]` for `a * x + b * y + c * z + d = 0`.
    ///
    /// `[a, b, c]` is the unit normal. This is the form a three-plane
    /// intersection and the side-of-plane predicate take.
    #[must_use]
    pub fn coefficients(&self) -> [f64; 4] {
        let [a, b, c] = self.normal.components();
        [a, b, c, -self.offset]
    }

    /// The point on the plane nearest the world origin.
    ///
    /// Also the corner that [`Plane::plane_coordinates_of`] measures from.
    #[must_use]
    pub fn origin(&self) -> Point {
        Point::origin() + self.normal * self.offset
    }

    /// How far `point` sits in front of the plane. Negative behind it.
    ///
    /// The size of this is a real distance. Its *sign*, close to zero, is not
    /// something to trust — see the note on [`Plane`].
    #[must_use]
    pub fn signed_distance_to(&self, point: Point) -> f64 {
        self.normal.dot(point - Point::origin()) - self.offset
    }

    /// The point on the plane nearest to `point`.
    #[must_use]
    pub fn project(&self, point: Point) -> Point {
        point - self.normal * self.signed_distance_to(point)
    }

    /// Two directions in the plane, at right angles to each other and to the
    /// normal, with `u` crossed into `v` giving the normal back.
    ///
    /// The choice is arbitrary but fixed: the same plane always gives the same
    /// pair, so a face parameterised today matches one parameterised tomorrow.
    ///
    /// # Panics
    ///
    /// If the stored normal has no length. That never happens: both public
    /// constructors, [`Plane::from_point_and_normal`] and
    /// [`Plane::from_coefficients`], check it, and `Deserialize` goes through
    /// the second one — `#[serde(try_from = "[f64; 4]")]` makes it the only
    /// route in on the wire. `a_plane_stored_with_no_normal_is_refused_on_the_way_in`
    /// (`crates/rcad-geom/tests/serialization.rs`) shows the bytes refused
    /// before a `Plane` exists to ask.
    #[must_use]
    pub fn basis(&self) -> (Vector, Vector) {
        let [x, y, z] = self.normal.components().map(f64::abs);
        let least_aligned_axis = if x <= y && x <= z {
            Vector::new(1.0, 0.0, 0.0)
        } else if y <= z {
            Vector::new(0.0, 1.0, 0.0)
        } else {
            Vector::new(0.0, 0.0, 1.0)
        };
        let u = self
            .normal
            .cross(least_aligned_axis)
            .normalized_as("direction in a plane")
            .expect("a unit normal is never parallel to its least aligned coordinate axis");
        (u, self.normal.cross(u))
    }

    /// Where `point` lands in the plane's own two coordinates.
    ///
    /// Points off the plane are projected onto it first, so this throws away
    /// however far off the plane the point was.
    #[must_use]
    pub fn plane_coordinates_of(&self, point: Point) -> (f64, f64) {
        let (u, v) = self.basis();
        let across = point - self.origin();
        (across.dot(u), across.dot(v))
    }

    /// The point at the given two coordinates in the plane.
    ///
    /// Undoes [`Plane::plane_coordinates_of`] for points that were on the plane.
    #[must_use]
    pub fn point_at(&self, along_u: f64, along_v: f64) -> Point {
        let (u, v) = self.basis();
        self.origin() + u * along_u + v * along_v
    }

    /// The same flat surface facing the other way.
    ///
    /// Same set of points, opposite normal, so every signed distance flips sign.
    #[must_use]
    pub fn reversed(&self) -> Self {
        Self {
            normal: -self.normal,
            offset: -self.offset,
        }
    }

    /// The same plane after `movement` is applied.
    ///
    /// **Moving a plane by [`Isometry::identity`] gives back the same four
    /// numbers, and this is the only line in the crate that has to arrange it.**
    /// `FaceView::plane` in `rcad-topo` is this function applied to a face's
    /// composed location whatever that location is, so anything here that moves a
    /// bit is charged on every read of every face, and a chain of fuses pays it
    /// once per fuse. `docs/decisions.md` §17.1 rejected vertex-authoritative
    /// geometry to avoid exactly that leak; §17.9 is what it cost when it arrived
    /// through the accessor instead.
    ///
    /// Two things follow, and neither is an optimisation:
    ///
    /// - **The offset is carried over, not re-derived.** A turn leaves
    ///   `normal · point` alone, so all the shift can do is move the plane along
    ///   its own moved normal. Rebuilding it as `moved_normal · movement(origin())`
    ///   multiplies the offset by `nx² + ny² + nz²`, which is only *nearly* one.
    /// - **The moved normal is not scaled back to unit length.** It arrived unit
    ///   from a constructor and a turn does not change a length; scaling it again
    ///   is the second scaling of an already-scaled vector, which the note on
    ///   [`Plane`] says moves the last bit or two. A stored normal whose computed
    ///   length is `0.9999999999999999` — an ordinary one — grew on every read.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        let normal = movement.apply_to_vector(self.normal);
        Self {
            normal,
            offset: self.offset + normal.dot(movement.translation_part()),
        }
    }

    /// Which side of the plane `point` is on: [`Decision::Pos`] in front of the
    /// normal, [`Decision::Neg`] behind it, [`Decision::Zero`] exactly on it.
    ///
    /// Same sign convention as [`Plane::signed_distance_to`], decided through
    /// the predicate cascade instead of guessed from an `f64`.
    ///
    /// # What this certifies, and what it does not
    ///
    /// This is the exact sign of `normal · point - offset` **for the four
    /// numbers this plane holds**. For a point with exact `f64` coordinates it
    /// always decides; for a constructed point whose box straddles the plane it
    /// returns [`Decision::Undecidable`], which is the honest answer.
    ///
    /// It is *not* a certified answer about the plane the caller meant. Every
    /// constructor here divides by the normal's length, and that division
    /// rounds, so the stored plane sits a rounding away from the one that was
    /// asked for. A vertex used to build a face is genuinely on the face's true
    /// plane and genuinely, provably, a rounding off the stored one — so this
    /// predicate will answer `Pos` or `Neg` with total confidence where the
    /// truthful answer is `Zero`. That is an exact answer to the wrong
    /// question, which is the failure this kernel exists to avoid
    /// (`docs/decisions.md` §2.5).
    ///
    /// **So: a caller asking "is this point on the plane this face was built
    /// from" must not ask here.** It must call
    /// [`orient3d_of_points`](rcad_predicates::orient3d_of_points) on the three
    /// points that defined the face, which never divides and is certified about
    /// the geometry the caller actually named. `Point` converts into the
    /// predicate crate's point type, so that call is a one-liner.
    ///
    /// What this *is* good for is consistency: it agrees with
    /// [`Plane::signed_distance_to`], [`Plane::project`] and every other
    /// accessor on this type, because they all use the same stored numbers. Use
    /// it to classify against a plane, not to rediscover a point that built one.
    ///
    /// # Input that is not made of real numbers
    ///
    /// Refused with [`NOTHING_IS_KNOWN`]. A point coordinate that is not a real
    /// number reached a certified answer, because it enters the expression only
    /// multiplied by the matching normal component, and an axis-aligned plane
    /// has two of those at zero.
    ///
    /// The plane's own four numbers go through the guard too.
    /// [`Plane::transformed`] cannot fail and can overflow the offset, so a
    /// plane built from finite numbers can be holding an infinite one. An
    /// infinite offset happens to reach the same refusal without help, because
    /// nothing multiplies the offset by zero — but working out which of the
    /// four numbers is safe from which direction is the kind of claim that
    /// rots, so all four are checked.
    #[must_use]
    pub fn side_of_point(&self, point: PredicatePoint) -> Decision {
        which_side_of_a_flat_surface(SIDE_OF_PLANE, self.normal.components(), self.offset, point)
    }

    /// Where three planes meet.
    ///
    /// The answer is a constructed point: a box the true meeting point is
    /// somewhere inside. Cramer's rule needs a division, and dividing two
    /// ranges almost never lands on an `f64`, so the box has real width even
    /// when the three planes are square to each other.
    ///
    /// That width is the point of it. A predicate over a constructed point
    /// cannot fall through to exact arithmetic, so it can genuinely come back
    /// [`Decision::Undecidable`] — this is the geometry-side entry to the case
    /// `docs/decisions.md` §2.5 asks M0 to reach.
    ///
    /// # Errors
    ///
    /// [`Error::Predicate`] wrapping
    /// [`PlanesDoNotMeetInAPoint`](rcad_predicates::Error::PlanesDoNotMeetInAPoint)
    /// when the three normals lie in one plane — so the three surfaces share a
    /// line, or two of them are parallel, and there is no single meeting point —
    /// or when they are close enough to that for the determinant not to be
    /// bounded away from zero.
    ///
    /// # And one thing it does not refuse
    ///
    /// **`Ok` can hold a box with a side that runs the whole number line.**
    /// Cramer's rule divides one interval determinant by another and checks
    /// neither for overflow, so three planes that meet past the top of `f64`'s
    /// range come back as a box that says nothing at all — which a caller would
    /// read as a place. Ask `uncertainty().is_finite()` before believing one.
    /// [`meet_of_three_surfaces`](crate::meet_of_three_surfaces) does, and treats
    /// it as the refusal it is;
    /// `three_planes_meeting_past_the_top_of_the_range_are_refused_rather_than_placed_nowhere`
    /// is the demonstration. The **limit** underneath it is real — no `f64`
    /// triple names a point at `1e315` — and `docs/decisions.md` §39 corrects
    /// §34.10, which filed it as a gap.
    pub fn meet_of_three(first: &Plane, second: &Plane, third: &Plane) -> Result<PredicatePoint> {
        Ok(meet_of_three_planes(
            first.coefficients(),
            second.coefficients(),
            third.coefficients(),
        )?)
    }

    /// Whether `other` is the same flat surface as this one, and if so whether
    /// it faces the same way.
    ///
    /// This is the certified "are these two faces coplanar" test, with no
    /// tolerance in it anywhere and no dot product of normals.
    ///
    /// # What "the same plane" means here
    ///
    /// The four numbers this plane holds, against the four `other` holds. Two
    /// coefficient vectors `[a, b, c, d]` describe the same surface exactly
    /// when one is a multiple of the other, which is exactly when all six of
    /// their two-by-two minors are zero. Each minor is one cross product of
    /// two `f64`, so [`orient2d`](rcad_predicates::orient2d) gives its exact
    /// sign for free.
    ///
    /// A minor with a certified non-zero sign *proves* the two surfaces are
    /// different, so one is enough to answer whatever the other five do.
    ///
    /// As with [`Plane::side_of_point`], this is exact about the stored
    /// numbers and says nothing about the planes a caller meant. Two faces
    /// built from the same three points by different routes may each round
    /// their normal a different way, and then they are honestly different
    /// planes here.
    #[must_use]
    pub fn agreement_with(&self, other: &Plane) -> PlaneAgreement {
        let mine = self.coefficients();
        let theirs = other.coefficients();

        let mut a_minor_was_undecided = false;
        for [first, second] in [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]] {
            match orient2d(
                [0.0, 0.0],
                [mine[first], mine[second]],
                [theirs[first], theirs[second]],
            ) {
                Decision::Zero => (),
                Decision::Neg | Decision::Pos => return PlaneAgreement::Different,
                Decision::Undecidable { .. } => a_minor_was_undecided = true,
                other => unreachable!("unhandled decision from orient2d: {other:?}"),
            }
        }
        if a_minor_was_undecided {
            return PlaneAgreement::CannotTell;
        }

        // Every minor is zero, so the two vectors are multiples of each other,
        // and neither is the zero vector — a stored normal always has length
        // one. Wherever one has a non-zero entry the other does too, and the
        // two signs are the sign of the multiplier.
        let position = (0..4)
            .find(|&position| mine[position] != 0.0)
            .expect("a stored plane's normal has length one, so it has a non-zero coefficient");
        assert!(
            theirs[position] != 0.0,
            "a plane proportional to another cannot be all zeroes"
        );
        if mine[position].is_sign_negative() == theirs[position].is_sign_negative() {
            PlaneAgreement::SameWay
        } else {
            PlaneAgreement::OppositeWay
        }
    }
}

/// What [`Plane::agreement_with`] found.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaneAgreement {
    /// Same plane, facing the same way.
    SameWay,
    /// Same plane, facing opposite ways.
    OppositeWay,
    /// Not the same plane, and that is proved rather than assumed.
    Different,
    /// One of the six minors could not be given a sign, so the two planes
    /// cannot be told apart and cannot be shown to match either.
    ///
    /// It takes coefficients spread so wide that no single power of two brings
    /// their products into the range where `f64` multiplication can be undone.
    /// A normal tilted by roughly `1e-300` used to be one and is not any more:
    /// [`orient2d`] normalises a uniformly tiny minor and decides it.
    /// `docs/decisions.md` §3.5's dynamic-range check is what keeps a genuinely
    /// spread model out; a caller reaching this has one anyway, and the honest
    /// answer is that nothing here can tell.
    CannotTell,
}

/// Which side of `normal · p = offset` the point is on, for any `normal` at all.
///
/// [`Plane`] hands over a normal scaled to length one and
/// [`HalfSpace`](crate::HalfSpace) hands over whatever its caller wrote down.
/// Neither the filter below nor the two levels under it care which: the error
/// bound is a fraction of the permanent, which carries the normal's own size,
/// and the other two levels are exact.
pub(crate) fn which_side_of_a_flat_surface(
    named: &'static str,
    normal: [f64; 3],
    offset: f64,
    point: PredicatePoint,
) -> Decision {
    if !(is_a_real_point(point) && are_real_numbers(normal) && are_real_numbers([offset])) {
        return NOTHING_IS_KNOWN;
    }
    decide(&SideOfAFlatSurface {
        named,
        normal,
        offset,
        point,
    })
    .0
}

/// `normal · point - offset`, evaluated three ways for the cascade.
struct SideOfAFlatSurface {
    named: &'static str,
    normal: [f64; 3],
    offset: f64,
    point: PredicatePoint,
}

impl Predicate for SideOfAFlatSurface {
    fn name(&self) -> &'static str {
        self.named
    }

    fn on_floats(&self) -> Option<Filtered> {
        let point = self.point.exact_coordinates()?;
        let normal = self.normal;

        // The bound below is a fraction of the permanent, which only bounds
        // anything while every product keeps its relative accuracy. One of them
        // in the subnormals and this level certifies a wrong sign.
        if !(0..3).all(|axis| product_is_normal(normal[axis], point[axis])) {
            return None;
        }

        let terms = [0, 1, 2].map(|axis| normal[axis] * point[axis]);
        let offset = self.offset;

        // The three terms can be of either sign and the offset cancels against
        // their sum, so the permanent adds magnitudes rather than values.
        let permanent = terms[0].abs() + terms[1].abs() + terms[2].abs() + offset.abs();

        Some(Filtered {
            value: terms[0] + terms[1] + terms[2] - offset,
            error_bound: SIDE_OF_PLANE_ERROR_BOUND * permanent,
        })
    }

    fn on_intervals(&self) -> Interval {
        let point = self.point.enclosure();
        let normal = self.normal;
        let term = |axis: usize| Interval::exactly(normal[axis]) * point[axis];
        term(0) + term(1) + term(2) - Interval::exactly(self.offset)
    }

    fn exactly(&self) -> Option<Decision> {
        let point = self.point.exact_coordinates()?;
        let normal = self.normal;

        let mut total = [0.0f64; LONGEST_PLANE_TOTAL];
        let mut total_len = 0;
        let mut merged = [0.0f64; LONGEST_PLANE_TOTAL];

        for (coordinate, along) in point.into_iter().zip(normal) {
            if !product_is_exact(coordinate, along) {
                return None;
            }
            let (product, roundoff) = two_product(coordinate, along);
            total_len = expansion_sum(&total[..total_len], &[roundoff, product], &mut merged);
            core::mem::swap(&mut total, &mut merged);
        }

        total_len = expansion_sum(&total[..total_len], &[-self.offset], &mut merged);

        match expansion_sign(&merged[..total_len]) {
            Decision::Neg => Some(Decision::Neg),
            Decision::Zero => Some(Decision::Zero),
            Decision::Pos => Some(Decision::Pos),
            // A product overflowed. Exact arithmetic that is not exact has
            // nothing to offer, so this level cannot run.
            Decision::Undecidable { .. } => None,
            other => unreachable!("unhandled decision from exact arithmetic: {other:?}"),
        }
    }
}

/// Level 1's forward error bound, as a multiple of the permanent.
///
/// `epsilon` is the `f64` unit roundoff, 2⁻⁵³.
///
/// Where the 4 comes from: follow the longest chain of roundings from an input
/// to `value` in [`SideOfAFlatSurface::on_floats`]. A term `normal · point` is one
/// rounding — both factors are inputs, so nothing is carried in. Folding the
/// three terms together takes two additions, a second and a third, and
/// subtracting the offset is a fourth. `(1 + epsilon)^4 - 1 <=
/// 4*epsilon + 6*epsilon^2 + ...` times the permanent.
///
/// The offset itself travels only the last of those, so the four covers it. The
/// `32*epsilon^2` covers the higher-order terms and the rounding of the
/// permanent; what it has to beat is about `6*epsilon`.
///
/// Every step of that argument is a *relative* one, so the bound holds only
/// while nothing overflows or falls into the subnormals.
/// [`SideOfAFlatSurface::on_floats`] enforces the bottom end product by product
/// through [`product_is_normal`], and `decide` the top end by refusing a bound
/// that is not itself a normal number.
const SIDE_OF_PLANE_ERROR_BOUND: f64 = {
    let epsilon = f64::EPSILON / 2.0;
    (4.0 + 32.0 * epsilon) * epsilon
};

/// Three exact products of two entries each, and the offset.
const LONGEST_PLANE_TOTAL: usize = 7;

impl TryFrom<[f64; 4]> for Plane {
    type Error = Error;

    fn try_from(coefficients: [f64; 4]) -> Result<Self> {
        Self::from_coefficients(coefficients)
    }
}

impl From<Plane> for [f64; 4] {
    fn from(plane: Plane) -> Self {
        plane.coefficients()
    }
}

#[cfg(test)]
mod tests {
    use core::f64::consts::{FRAC_PI_2, TAU};

    use rcad_predicates::{Level, orient3d};

    use super::*;
    use crate::Axis;
    use crate::exact::Exact;

    fn plane_at_height_two() -> Plane {
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 2.0), Vector::new(0.0, 0.0, 1.0)).unwrap()
    }

    #[test]
    fn a_point_in_front_of_the_plane_is_a_positive_distance_away() {
        assert_eq!(
            plane_at_height_two().signed_distance_to(Point::new(9.0, -9.0, 5.0)),
            3.0
        );
    }

    #[test]
    fn a_point_behind_the_plane_is_a_negative_distance_away() {
        assert_eq!(
            plane_at_height_two().signed_distance_to(Point::origin()),
            -2.0
        );
    }

    #[test]
    fn the_normal_is_scaled_to_length_one_whatever_the_caller_hands_over() {
        let plane =
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 7.0)).unwrap();
        assert_eq!(plane.normal(), Vector::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn a_plane_needs_a_normal_that_has_some_length() {
        assert_eq!(
            Plane::from_point_and_normal(Point::origin(), Vector::zero()),
            Err(Error::ZeroLength {
                what: "plane normal"
            })
        );
    }

    /// **The half-space gap, at the one line of arithmetic it comes from.**
    ///
    /// This asserts what the code does, so a fix fails it loudly (§19).
    ///
    /// A caller *can* write down a normal exactly square to a direction it
    /// already holds: swap two components and negate one, and the dot product
    /// becomes two products that are exact negatives of each other, so they
    /// cancel whatever the direction is. For the axis `w = (1, 3, 5)` those are
    /// `(3, −1, 0)`, `(0, 5, −3)` and `(5, 0, −1)`, and this checks all three
    /// are exactly square to `w` before a plane sees them.
    ///
    /// **Not one of them is afterwards.**
    /// [`Plane::reduced_by_largest_component`] divides every component by the
    /// largest, and that division is correctly rounded rather than exact unless
    /// the largest is a power of two; the normalization after it divides by a
    /// square root, which is exact almost never. Worked out in exact rational
    /// arithmetic over the stored bits, the three miss squareness by one or two
    /// last bits of the direction. **Two of the three read exactly `0.0` in
    /// plain `f64` and the third reads `1.1e−16`**, so a filter that trusted the
    /// `f64` would call two of them square and be wrong about both; the exact
    /// level sees the sign and refuses, which is what it is for.
    ///
    /// That is what leaves `Error::CannotNameWhichPieceOfACrossingThisIs`
    /// standing for a piece of a crossing that runs on for ever: such a piece
    /// leaves any half-space whose normal leans into its direction by even one
    /// rounding, so the squareness has to be arranged and cannot be tested
    /// (§29.8). The test below is why it is a **limit** of a plane that stores
    /// a canonical normal rather than a gap.
    #[test]
    fn a_normal_written_square_to_a_direction_stops_being_square_once_a_plane_stores_it() {
        let along = [1.0, 3.0, 5.0];
        let square_to_it = [[3.0, -1.0, 0.0], [0.0, 5.0, -3.0], [5.0, 0.0, -1.0]];

        for written in square_to_it {
            assert_eq!(
                Exact::dot_of(written, along).map(|dot| dot.sign()),
                Some(Decision::Zero),
                "{written:?} is square to {along:?} as the caller wrote it"
            );

            let stored = Plane::from_point_and_normal(Point::origin(), Vector::from(written))
                .expect("a normal with a length")
                .normal()
                .components();
            let in_f64 = stored[0] * along[0] + stored[1] * along[1] + stored[2] * along[2];
            println!("{written:?} stored as {stored:?}, dot with {along:?} is {in_f64:e} in f64");
            assert_ne!(
                Exact::dot_of(stored, along).map(|dot| dot.sign()),
                Some(Decision::Zero),
                "and it is not square to it any more once stored as {stored:?}"
            );
            assert!(
                in_f64.abs() <= f64::EPSILON,
                "the miss is one rounding of the direction, which is why it hides: {in_f64:e}"
            );
        }
    }

    /// **Why §31.4's family of fixes is not one**, demonstrated rather than
    /// argued.
    ///
    /// §31.4 works out that the reduction above is what loses the squareness,
    /// and offers reducing by a **power of two** instead, which rounds nothing.
    /// The first half here confirms that half of it: a normal written square to
    /// a direction is still square to it after a power-of-two reduction, where
    /// the test above says it is not after the reduction the code runs.
    ///
    /// **The second half is what §31.4 did not know.** Two multiples of one
    /// wall reduced by two different powers of two differ by a factor that is
    /// not a power of two — `(0, −1, 0)` stays where it is and `(0, −3, 0)`
    /// becomes `(0, −1.5, 0)` — and the moment the model is turned, the two stop
    /// being exactly proportional at all, because each component of each is
    /// rounded on its own. [`Plane::agreement_with`] is an exact predicate over
    /// the stored numbers, so it answers `Different`, correctly, about two faces
    /// that are one wall.
    ///
    /// A canonical *name* beside the stored numbers does not reach this. Every
    /// "is this the same plane" question in the kernel —
    /// [`Plane::agreement_with`], `planes_are_parallel`, `relation_of_two_lines`
    /// — is an exact predicate over the coefficients, and the two coefficient
    /// vectors here are genuinely not proportional. Measured with the whole fix
    /// in place: a certified fuse of a turned pair went from refusing 20.71% of
    /// the corpus to refusing 77%, because two walls that are one wall each cut
    /// the other and three planes that share a line have nowhere to meet.
    ///
    /// So a `Plane` can be canonical under scaling or it can keep a square
    /// normal square, and **not both**: canonical means dividing by something
    /// the direction alone decides, exact means dividing by a power of two, and
    /// two multiples of one vector have the same power of two only when the
    /// multiple is itself one. §32.
    #[test]
    fn a_power_of_two_reduction_keeps_a_normal_square_and_stops_one_wall_being_one_plane() {
        let reduced = |v: Vector| {
            let [x, y, z] = v.components();
            let largest = x.abs().max(y.abs()).max(z.abs());
            let scale = 2f64.powi(largest.log2().floor() as i32);
            let reduced = Vector::new(x / scale, y / scale, z / scale);
            assert_eq!(reduced * scale, v, "a power of two divides exactly");
            reduced
        };

        let along = [1.0, 3.0, 5.0];
        for written in [[3.0, -1.0, 0.0], [0.0, 5.0, -3.0], [5.0, 0.0, -1.0]] {
            assert_eq!(
                Exact::dot_of(reduced(Vector::from(written)).components(), along)
                    .map(|dot| dot.sign()),
                Some(Decision::Zero),
                "{written:?} is still square to {along:?} after a power-of-two reduction"
            );
        }

        let turn = Isometry::rotation(
            &Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("a direction"),
            TAU * 7.0 / 64.0,
        )
        .expect("a finite rotation");
        let wall_named_off_an_edge = |length: f64| {
            turn.apply_to_vector(reduced(Vector::new(0.0, -length, 0.0)))
                .components()
        };
        let (one, other) = (wall_named_off_an_edge(1.0), wall_named_off_an_edge(3.0));

        let a_minor_is_not_zero = (0..3).any(|axis| {
            let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
            orient2d(
                [0.0, 0.0],
                [one[next], one[after]],
                [other[next], other[after]],
            ) != Decision::Zero
        });
        assert!(
            a_minor_is_not_zero,
            "one wall named off two edge lengths and turned: {one:?} against {other:?}, which              this arithmetic would have to call one plane and cannot"
        );
    }

    /// §22.6, from the other side: the defect was that these differed.
    ///
    /// Four vectors along one line, none of them a power-of-two multiple of the
    /// next, and the plane has to be the same four numbers every time.
    #[test]
    fn a_plane_is_the_same_however_long_the_normal_naming_it_was() {
        let plane_of = |x: f64, y: f64| {
            Plane::from_point_and_normal(Point::origin(), Vector::new(x, y, 0.0))
                .expect("a diagonal normal has length")
                .coefficients()
        };

        for length in [2.0, 3.0, 7.0, 1e17, 1e-17] {
            assert_eq!(
                plane_of(1.0, 1.0),
                plane_of(length, length),
                "a diagonal wall standing on an edge {length} long"
            );
        }

        for multiple in [2.0, 3.0, 5.0, 11.0] {
            assert_eq!(
                plane_of(3.0, 7.0),
                plane_of(3.0 * multiple, 7.0 * multiple),
                "the ratio 3:7, named {multiple} times over"
            );
        }
    }

    /// The same, for the coefficient form, where the fourth number scales too.
    #[test]
    fn proportional_coefficients_describe_the_same_plane_bit_for_bit() {
        let plane_of = |scale: f64| {
            Plane::from_coefficients([3.0 * scale, 7.0 * scale, 0.0, 5.0 * scale])
                .expect("a diagonal normal has length")
                .coefficients()
        };

        for scale in [2.0, 3.0, 11.0] {
            assert_eq!(
                plane_of(1.0),
                plane_of(scale),
                "every coefficient × {scale}"
            );
        }
    }

    /// What the unfixed path does to a normal near the top of the range, kept
    /// as the evidence for the claim on
    /// [`Plane::reduced_by_largest_component`] that dividing first widens what
    /// can be answered.
    ///
    /// [`Vector::normalized`] is the arithmetic the plane constructors used to
    /// run: the length overflows, every component is divided by infinity, and
    /// what comes back is a direction of length zero. Not an error — a silent
    /// wrong answer, which §4.1 ranks below a crash.
    #[test]
    fn normalizing_a_huge_vector_gives_back_no_direction_at_all() {
        let huge = Vector::new(1e308, 1e308, 0.0);
        assert!(huge.is_finite());
        assert_eq!(huge.length(), f64::INFINITY);
        assert_eq!(huge.normalized(), Ok(Vector::zero()));
    }

    /// And the plane built from that same normal is a real plane.
    #[test]
    fn a_normal_near_the_top_of_the_range_still_names_its_plane() {
        let huge = Plane::from_point_and_normal(Point::origin(), Vector::new(1e308, 1e308, 0.0))
            .expect("the ratio 1:1 says which way this faces");
        let ordinary = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 1.0, 0.0))
            .expect("a diagonal normal has length");
        assert_eq!(huge.coefficients(), ordinary.coefficients());
    }

    /// The bottom of the range, where the length underflows to zero and the old
    /// path refused outright.
    #[test]
    fn a_normal_deep_in_the_subnormals_still_names_its_plane() {
        let tiny = Vector::new(5e-324, 5e-324, 0.0);
        assert!(tiny.is_finite());
        assert_eq!(tiny.length(), 0.0);
        assert_eq!(
            tiny.normalized(),
            Err(Error::ZeroLength { what: "vector" }),
            "the arithmetic the plane constructors used to run"
        );

        let plane = Plane::from_point_and_normal(Point::origin(), tiny)
            .expect("the ratio 1:1 says which way this faces");
        let ordinary = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 1.0, 0.0))
            .expect("a diagonal normal has length");
        assert_eq!(plane.coefficients(), ordinary.coefficients());
    }

    #[test]
    fn projecting_a_point_puts_it_on_the_plane() {
        let plane = plane_at_height_two();
        let landed = plane.project(Point::new(3.0, 4.0, 100.0));
        assert_eq!(landed, Point::new(3.0, 4.0, 2.0));
        assert_eq!(plane.signed_distance_to(landed), 0.0);
    }

    #[test]
    fn three_points_in_a_line_do_not_make_a_plane() {
        assert_eq!(
            Plane::through_three_points(
                Point::origin(),
                Point::new(1.0, 1.0, 1.0),
                Point::new(3.0, 3.0, 3.0)
            ),
            Err(Error::PointsAreInALine)
        );
    }

    #[test]
    fn three_points_wound_anticlockwise_give_a_normal_pointing_at_you() {
        let plane = Plane::through_three_points(
            Point::origin(),
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        assert_eq!(plane.normal(), Vector::new(0.0, 0.0, 1.0));
        assert_eq!(plane.offset(), 0.0);
    }

    #[test]
    fn coefficients_are_scaled_so_the_first_three_have_length_one() {
        let plane = Plane::from_coefficients([0.0, 0.0, 2.0, -4.0]).unwrap();
        assert_eq!(plane.coefficients(), [0.0, 0.0, 1.0, -2.0]);
        assert_eq!(plane.offset(), 2.0);
    }

    #[test]
    fn coefficients_that_describe_no_plane_are_refused() {
        assert_eq!(
            Plane::from_coefficients([0.0, 0.0, 0.0, 1.0]),
            Err(Error::ZeroLength {
                what: "plane normal"
            })
        );
    }

    #[test]
    fn a_plane_survives_a_round_trip_through_its_coefficients() {
        let plane = Plane::through_three_points(
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 2.0, 0.0),
            Point::new(0.0, 0.0, 3.0),
        )
        .unwrap();
        let recovered = Plane::try_from(plane.coefficients()).unwrap();
        assert!((recovered.normal() - plane.normal()).length() < 1e-15);
        assert!((recovered.offset() - plane.offset()).abs() < 1e-15);
    }

    #[test]
    fn a_plane_facing_along_an_axis_comes_back_from_its_coefficients_bit_for_bit() {
        let plane = plane_at_height_two();
        assert_eq!(Plane::try_from(plane.coefficients()).unwrap(), plane);
    }

    #[test]
    fn the_two_directions_in_the_plane_are_at_right_angles_and_cross_to_the_normal() {
        for normal in [
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(1.0, 0.0, 0.0),
            Vector::new(1.0, 2.0, 3.0),
            Vector::new(-5.0, 0.5, 0.25),
        ] {
            let plane = Plane::from_point_and_normal(Point::new(1.0, 2.0, 3.0), normal).unwrap();
            let (u, v) = plane.basis();
            assert!((u.length() - 1.0).abs() < 1e-15);
            assert!((v.length() - 1.0).abs() < 1e-15);
            assert!(u.dot(v).abs() < 1e-15);
            assert!((u.cross(v) - plane.normal()).length() < 1e-15);
        }
    }

    #[test]
    fn a_point_on_the_plane_comes_back_from_its_two_plane_coordinates() {
        let plane =
            Plane::from_point_and_normal(Point::new(1.0, 2.0, 3.0), Vector::new(1.0, 2.0, 3.0))
                .unwrap();
        let on_the_plane = plane.point_at(4.0, -7.0);
        let (along_u, along_v) = plane.plane_coordinates_of(on_the_plane);
        assert!((along_u - 4.0).abs() < 1e-12);
        assert!((along_v + 7.0).abs() < 1e-12);
    }

    #[test]
    fn plane_coordinates_ignore_how_far_off_the_plane_a_point_is() {
        let plane = plane_at_height_two();
        let (on, off) = (Point::new(3.0, 4.0, 2.0), Point::new(3.0, 4.0, 90.0));
        assert_eq!(
            plane.plane_coordinates_of(on),
            plane.plane_coordinates_of(off)
        );
    }

    #[test]
    fn reversing_a_plane_flips_every_signed_distance() {
        let plane = plane_at_height_two();
        let point = Point::new(1.0, 1.0, 7.0);
        assert_eq!(
            plane.reversed().signed_distance_to(point),
            -plane.signed_distance_to(point)
        );
    }

    #[test]
    fn a_point_in_front_of_the_plane_is_positive_and_one_behind_it_is_negative() {
        let plane = plane_at_height_two();
        assert_eq!(
            plane.side_of_point(PredicatePoint::from([9.0, -9.0, 5.0])),
            Decision::Pos
        );
        assert_eq!(
            plane.side_of_point(PredicatePoint::from([0.0, 0.0, 0.0])),
            Decision::Neg
        );
    }

    #[test]
    fn a_point_clear_of_the_plane_is_settled_by_the_cheapest_level() {
        let (decision, level) = decide(&SideOfAFlatSurface {
            named: SIDE_OF_PLANE,
            normal: plane_at_height_two().normal().components(),
            offset: plane_at_height_two().offset(),
            point: PredicatePoint::from([0.0, 0.0, 100.0]),
        });
        assert_eq!(decision, Decision::Pos);
        assert_eq!(level, Level::Float);
    }

    #[test]
    fn a_point_on_a_plane_whose_numbers_are_exact_comes_out_exactly_on_it() {
        assert_eq!(
            plane_at_height_two().side_of_point(PredicatePoint::from([3.0, 4.0, 2.0])),
            Decision::Zero
        );
    }

    /// The reason [`Plane::side_of_point`]'s doc tells callers who need a
    /// certified "is this point on the face" to use `orient3d` instead.
    ///
    /// All three points define the plane, so the truthful answer for each is
    /// `Zero`, and `orient3d` — which never divides — says so. Normalising the
    /// normal moved the stored plane off them, and the predicate is certain
    /// about the plane it was given. The first point even has an `f64` distance
    /// of exactly zero while the certified sign against the stored coefficients
    /// is negative.
    #[test]
    fn the_points_that_defined_a_plane_are_not_on_the_plane_that_gets_stored() {
        let corners = [[1.0, 2.0, 3.0], [4.0, 5.0, 7.0], [2.0, 9.0, 4.0]];
        let [first, second, third] = corners.map(|[x, y, z]| Point::new(x, y, z));
        let plane = Plane::through_three_points(first, second, third).unwrap();

        assert_eq!(plane.signed_distance_to(first), 0.0);
        for corner in corners {
            assert_eq!(
                plane.side_of_point(PredicatePoint::from(corner)),
                Decision::Neg,
                "the stored plane is certain, and wrong, about {corner:?}"
            );
            assert_eq!(
                orient3d(corners[0], corners[1], corners[2], corner),
                Decision::Zero,
                "the three defining points say what is true about {corner:?}"
            );
        }
    }

    /// A point that is exactly on the plane, and level 1 used to place it in
    /// front.
    ///
    /// The stored normal is `(1, 2, 7)` divided by `sqrt(54)`, and that
    /// division happens to leave `n1` exactly `2 * n0` and `n2` exactly
    /// `7 * n0` in `f64` — the test asserts it, because the rest only follows
    /// if it holds. The offset is zero, so for the point `(-3u, -9u, 3u)` the
    /// value is
    ///
    /// ```text
    /// n0*(-3u) + n1*(-9u) + n2*(3u) = n0 * u * (-3 - 18 + 21) = 0
    /// ```
    ///
    /// exactly, whatever `n0` and `u` are. The truthful answer is
    /// [`Decision::Zero`].
    ///
    /// `u` is the smallest number `f64` has, so each of the three products is
    /// a fraction of one `u` and gets rounded to a whole number of them:
    /// `-0.409u` to `-0`, `-1.23u` to `-1u`, and `2.86u` to `3u`. Those sum to
    /// `+2u` where the truth is zero. The permanent is built from the same
    /// three rounded products, so the bound is `4.4e-16 * 4u`, which is far
    /// below one `u` and rounds to zero — and `2u > 0` certifies.
    #[test]
    fn a_point_whose_products_fall_off_the_bottom_of_the_range_is_not_certified() {
        let plane = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 2.0, 7.0))
            .expect("a normal of length sqrt(54) is a fine normal");
        let [n0, n1, n2] = plane.normal().components();
        assert_eq!(n1, 2.0 * n0, "the rest of this test rests on this");
        assert_eq!(n2, 7.0 * n0, "the rest of this test rests on this");

        let u = f64::from_bits(1);
        let decision = plane.side_of_point(PredicatePoint::from([-3.0 * u, -9.0 * u, 3.0 * u]));
        match decision {
            Decision::Pos => panic!("this point is exactly on the plane, not in front of it"),
            Decision::Neg | Decision::Zero => panic!("nothing here can prove a sign: {decision:?}"),
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
            other => panic!("unhandled decision: {other:?}"),
        }
    }

    /// The same point scaled up by `2^600`, as an oracle for the test above.
    ///
    /// The plane passes through the origin, so `normal · point - 0` is linear
    /// in the point: multiplying the point by a positive number multiplies the
    /// value by it and leaves the sign alone. At `2^600` the three products
    /// are ordinary normal numbers, the exact cancellation is visible, and the
    /// answer is the [`Decision::Zero`] the test above cannot prove.
    #[test]
    fn the_same_point_scaled_up_to_fit_comes_out_exactly_on_the_plane() {
        let plane = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 2.0, 7.0))
            .expect("a normal of length sqrt(54) is a fine normal");
        let big = f64::from_bits(1) * 2f64.powi(600);
        assert_eq!(
            plane.side_of_point(PredicatePoint::from([-3.0 * big, -9.0 * big, 3.0 * big])),
            Decision::Zero
        );
    }

    #[test]
    fn three_planes_that_cross_give_a_box_around_where_they_meet() {
        let meet = Plane::meet_of_three(
            &Plane::from_coefficients([1.0, 1.0, 1.0, -1.0]).unwrap(),
            &Plane::from_coefficients([1.0, -1.0, 0.0, 0.0]).unwrap(),
            &Plane::from_coefficients([0.0, 1.0, -1.0, 0.0]).unwrap(),
        )
        .unwrap();

        assert_eq!(meet.exact_coordinates(), None, "a third is not an f64");
        for side in meet.enclosure() {
            assert!(side.lo() <= 1.0 / 3.0 && 1.0 / 3.0 <= side.hi());
        }
        assert!(meet.uncertainty() > 0.0);
    }

    #[test]
    fn two_parallel_planes_have_nowhere_to_meet() {
        let outcome = Plane::meet_of_three(
            &plane_at_height_two(),
            &Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0)).unwrap(),
            &Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0)).unwrap(),
        );
        assert!(matches!(
            outcome,
            Err(Error::Predicate(
                rcad_predicates::Error::PlanesDoNotMeetInAPoint { .. }
            ))
        ));
    }

    #[test]
    fn three_planes_sharing_one_line_have_no_single_meeting_point() {
        let outcome = Plane::meet_of_three(
            &Plane::from_coefficients([1.0, 0.0, 0.0, 0.0]).unwrap(),
            &Plane::from_coefficients([0.0, 1.0, 0.0, 0.0]).unwrap(),
            &Plane::from_coefficients([1.0, 1.0, 0.0, 0.0]).unwrap(),
        );
        assert!(matches!(
            outcome,
            Err(Error::Predicate(
                rcad_predicates::Error::PlanesDoNotMeetInAPoint { .. }
            ))
        ));
    }

    /// The meeting point of the three planes lies on the first of them, and it
    /// is only known as a box. Exact arithmetic cannot help — the input itself
    /// is not exact — so the honest answer is that we cannot tell.
    #[test]
    fn a_constructed_point_on_a_plane_cannot_be_placed_on_it() {
        let plane = Plane::from_coefficients([1.0, 1.0, 1.0, -1.0]).unwrap();
        let meet = Plane::meet_of_three(
            &plane,
            &Plane::from_coefficients([1.0, -1.0, 0.0, 0.0]).unwrap(),
            &Plane::from_coefficients([0.0, 1.0, -1.0, 0.0]).unwrap(),
        )
        .unwrap();

        match plane.side_of_point(meet) {
            Decision::Neg | Decision::Zero | Decision::Pos => {
                panic!("the box is not tight enough to place this")
            }
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
            other => panic!("unhandled decision: {other:?}"),
        }
    }

    #[test]
    fn a_constructed_point_well_clear_of_a_plane_is_still_decided() {
        let meet = Plane::meet_of_three(
            &Plane::from_coefficients([1.0, 1.0, 1.0, -1.0]).unwrap(),
            &Plane::from_coefficients([1.0, -1.0, 0.0, 0.0]).unwrap(),
            &Plane::from_coefficients([0.0, 1.0, -1.0, 0.0]).unwrap(),
        )
        .unwrap();

        match plane_at_height_two().side_of_point(meet) {
            Decision::Neg => (),
            other => panic!("a box around (1/3, 1/3, 1/3) is well below z = 2: {other:?}"),
        }
    }

    /// [`Plane::agreement_with`] reads each of its six minors off `orient2d`,
    /// which only works if `orient2d` measured from the origin is that
    /// determinant. Small integers keep the plain `f64` product exact, so it
    /// serves as the oracle.
    #[test]
    fn orient2d_from_the_origin_is_the_sign_of_a_two_by_two_determinant() {
        for x1 in -2..=2 {
            for y1 in -2..=2 {
                for x2 in -2..=2 {
                    for y2 in -2..=2 {
                        let [x1, y1, x2, y2] = [x1, y1, x2, y2].map(f64::from);
                        let determinant = x1 * y2 - y1 * x2;
                        let expected = if determinant > 0.0 {
                            Decision::Pos
                        } else if determinant < 0.0 {
                            Decision::Neg
                        } else {
                            Decision::Zero
                        };
                        assert_eq!(
                            orient2d([0.0, 0.0], [x1, y1], [x2, y2]),
                            expected,
                            "{x1} {y1} {x2} {y2}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_plane_is_the_same_plane_as_itself_facing_the_same_way() {
        let plane = plane_at_height_two();
        assert_eq!(plane.agreement_with(&plane), PlaneAgreement::SameWay);
    }

    #[test]
    fn a_reversed_plane_is_the_same_plane_facing_the_other_way() {
        let plane = plane_at_height_two();
        assert_eq!(
            plane.agreement_with(&plane.reversed()),
            PlaneAgreement::OppositeWay
        );
    }

    #[test]
    fn two_parallel_planes_at_different_heights_are_not_the_same_plane() {
        let higher =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, 5.0), Vector::new(0.0, 0.0, 1.0))
                .unwrap();
        assert_eq!(
            plane_at_height_two().agreement_with(&higher),
            PlaneAgreement::Different
        );
    }

    #[test]
    fn two_planes_the_same_distance_from_the_origin_but_facing_elsewhere_are_not_the_same_plane() {
        let along_x = Plane::from_coefficients([1.0, 0.0, 0.0, -2.0]).unwrap();
        let along_y = Plane::from_coefficients([0.0, 1.0, 0.0, -2.0]).unwrap();
        assert_eq!(along_x.offset(), along_y.offset());
        assert_eq!(along_x.agreement_with(&along_y), PlaneAgreement::Different);
    }

    /// The case a comparison of coefficients gets wrong.
    ///
    /// Every stored `Plane` has its normal scaled to length one, so two
    /// coefficient vectors that are multiples of each other by anything but
    /// `-1` cannot be built through the public constructors —
    /// `[2, 0, 0, -6]` and `[1, 0, 0, -3]` come back bit for bit identical,
    /// and a comparison would get *those* right. The multiplier that does
    /// survive is `-1`, and there a comparison says "different plane" where
    /// the truth is "same plane, other way round". The normal here is
    /// irrational, so nothing about the answer comes from the numbers being
    /// tidy.
    #[test]
    fn the_same_plane_scaled_by_minus_one_is_not_two_planes() {
        let plane = Plane::through_three_points(
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 2.0, 0.0),
            Point::new(0.0, 0.0, 3.0),
        )
        .unwrap();
        let other_way = plane.reversed();

        assert_ne!(
            plane.coefficients(),
            other_way.coefficients(),
            "comparing the four numbers calls these two different planes"
        );
        assert_eq!(
            plane.agreement_with(&other_way),
            PlaneAgreement::OppositeWay
        );

        let scaled_the_easy_way = Plane::from_coefficients([2.0, 0.0, 0.0, -6.0]).unwrap();
        assert_eq!(
            scaled_the_easy_way.coefficients(),
            Plane::from_coefficients([1.0, 0.0, 0.0, -3.0])
                .unwrap()
                .coefficients(),
            "scaling is undone on the way in, so this is the case that cannot arise"
        );
    }

    #[test]
    fn two_planes_one_ulp_apart_are_different_planes_with_no_tolerance_to_argue_about() {
        let plane = Plane::from_coefficients([1.0, 0.0, 0.0, -3.0]).unwrap();
        let a_hair_further_out =
            Plane::from_coefficients([1.0, 0.0, 0.0, -f64::from_bits(3f64.to_bits() + 1)]).unwrap();

        assert_ne!(plane.offset(), a_hair_further_out.offset());
        assert!((plane.offset() - a_hair_further_out.offset()).abs() < 1e-15);
        assert_eq!(
            plane.agreement_with(&a_hair_further_out),
            PlaneAgreement::Different
        );
    }

    /// A plane tilted by so little that the products of its own coefficients
    /// fall out of the range where `f64` multiplication can be undone.
    ///
    /// The truthful answer is [`PlaneAgreement::OppositeWay`] — it is asked
    /// against its own reverse — and the `a * b` minor is `1e-300 * -2e-300`
    /// against `2e-300 * -1e-300`, both of which underflow to zero where they
    /// are written. They are uniformly tiny rather than spread apart, so
    /// [`orient2d`] lifts the whole minor into range by an exact power of two
    /// and proves the difference is zero. This used to be
    /// [`PlaneAgreement::CannotTell`], which was honest and was not the answer.
    #[test]
    fn a_plane_tilted_by_a_hair_too_small_to_multiply_matches_its_own_reverse() {
        let plane = Plane::from_point_and_normal(Point::origin(), Vector::new(1e-300, 2e-300, 1.0))
            .unwrap();
        assert_eq!(
            plane.normal().components(),
            [1e-300, 2e-300, 1.0],
            "the rest of this test rests on the normal surviving scaling unchanged"
        );

        assert_eq!(
            plane.agreement_with(&plane.reversed()),
            PlaneAgreement::OppositeWay
        );
        assert_eq!(
            plane.agreement_with(&plane),
            PlaneAgreement::SameWay,
            "and it is still the same plane as itself"
        );
    }

    #[test]
    fn moving_a_plane_moves_the_points_on_it_the_same_way() {
        let plane = plane_at_height_two();
        let turn = Isometry::rotation(&Axis::x(), FRAC_PI_2).unwrap();
        let moved = plane.transformed(&turn);

        let on_the_plane = plane.point_at(5.0, -3.0);
        assert!(
            moved
                .signed_distance_to(turn.apply_to_point(on_the_plane))
                .abs()
                < 1e-14
        );
        assert!((moved.normal() - Vector::new(0.0, -1.0, 0.0)).length() < 1e-15);
    }

    /// A point that is not a point is on neither side of a plane.
    ///
    /// Before the guard, the plane at height two answered [`Decision::Pos`] for
    /// a point whose `x` was not a number: `x` enters only multiplied by the
    /// normal's `x` component, which is zero here, so the unknown was
    /// multiplied away and the height certified the sign on its own.
    #[test]
    fn a_point_that_is_not_a_point_is_on_neither_side() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for axis in 0..3 {
                let mut broken = [0.0, 0.0, 5.0];
                broken[axis] = bad;
                assert_eq!(
                    plane_at_height_two().side_of_point(PredicatePoint::from(broken)),
                    Decision::Undecidable {
                        enclosure: f64::INFINITY
                    },
                    "{bad} on axis {axis}"
                );
            }
        }
    }

    /// [`Plane::transformed`] cannot fail and can overflow, so a plane built
    /// from finite numbers can be holding an infinite offset. It then describes
    /// no plane, and nothing is on either side of it.
    #[test]
    fn a_plane_shoved_past_the_end_of_the_number_line_has_no_sides() {
        let far = 1e308;
        let shove = Isometry::translation(Vector::new(far, 0.0, 0.0)).unwrap();
        let shoved =
            Plane::from_point_and_normal(Point::new(far, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
                .unwrap()
                .transformed(&shove);

        assert!(!shoved.offset().is_finite(), "the offset overflowed");
        assert_eq!(
            shoved.side_of_point(PredicatePoint::from([0.0, 5.0, 0.0])),
            Decision::Undecidable {
                enclosure: f64::INFINITY
            }
        );
    }

    /// [`Plane::agreement_with`] runs six `orient2d` calls over the two sets of
    /// coefficients, so the guard there is what keeps a broken plane from being
    /// declared the same as, or different from, a sound one.
    #[test]
    fn a_plane_that_is_not_a_plane_cannot_be_compared_with_one_that_is() {
        let far = 1e308;
        let shove = Isometry::translation(Vector::new(far, 0.0, 0.0)).unwrap();
        let shoved =
            Plane::from_point_and_normal(Point::new(far, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
                .unwrap()
                .transformed(&shove);
        let sound =
            Plane::from_point_and_normal(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
                .unwrap();

        assert_eq!(shoved.agreement_with(&sound), PlaneAgreement::CannotTell);
        assert_eq!(sound.agreement_with(&shoved), PlaneAgreement::CannotTell);
    }
}
