//! Flat surfaces.

use rcad_predicates::expansion::{expansion_sign, expansion_sum, product_is_exact, two_product};
use rcad_predicates::{
    Decision, Filtered, Interval, NOTHING_IS_KNOWN, Point as PredicatePoint, Predicate,
    are_real_numbers, decide, is_a_real_point, meet_of_three_planes, orient2d, product_is_normal,
};
use serde::{Deserialize, Serialize};

use crate::exact::Exact;
use crate::isometry::Isometry;
use crate::point::{Point, Vector, a_hundred_bit_number_divided_by, product_to_a_hundred_bits};
use crate::{Error, Result};

/// The name the plane's sign predicate goes by in provenance records.
pub const SIDE_OF_PLANE: &str = "side of plane";

/// A flat surface: every point `p` on it satisfies `normal · p = offset`.
///
/// Every constructor stores a normal of length one, so
/// [`Plane::signed_distance_to`] is a real distance and not a scaled one.
/// [`Plane::transformed`] turns a normal without scaling it again — a read of a
/// plane has to give back the plane that was stored — so a moved plane's normal
/// has length one to within the rounding of the turn,
/// [`Plane::NORMAL_LENGTH_ROUNDOFF`], and
/// [`Plane::from_point_and_normal_as_written`] keeps a normal that is that
/// close rather than scale it again.
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
/// # No stored number is `-0.0`
///
/// Two faces that are the same plane hold the same four numbers, and half the
/// kernel rests on it: `Surface::how_it_sorts_against` puts a triple in a fixed
/// order on those numbers, `rcad-topo`'s `name_of_surface_bits` names a corner
/// by them, and `canonical_cut_surfaces` drops a repeat by them. `-0.0` is the
/// same *number* as `0.0` and a different `f64`, so a single one of them
/// anywhere in a plane makes that sentence false — and the ordering is
/// [`f64::total_cmp`](f64::total_cmp), which sorts `-0.0` first, so two
/// spellings of one wall reach Cramer's rule with their rows permuted and its
/// corner is rounded to two places an ulp apart.
///
/// [`Plane::holding`] turns every stored zero the right way up, and every
/// constructor here goes through it. Two consequences worth naming:
///
/// - **A caller may hand one in.** [`Plane::from_point_and_normal`] is public
///   and reads whatever vector it is given; a STEP `DIRECTION` may be written
///   `(-0.,0.,1.)` and `rcad-step` passes it straight through.
/// - **[`Plane::reversed`] is an involution on the bits**, which is what lets
///   `three_planes_faced_the_one_way` pick one of a plane's two spellings by
///   the sign of its leading coefficient.
///
/// [`Axis`](crate::Axis) and [`Sphere`](crate::Sphere) have the same exposure
/// and fold the same way, so a pipe, a cone or a torus built about
/// `Vector::new(-0.0, 0.0, 1.0)` and a ball centred on
/// `Point::new(-0.0, 1.0, 2.0)` store what the same surface spelled with
/// positive zeros stores. A [`HalfSpace`](crate::HalfSpace) is the one flat
/// surface that does not, and deliberately: it has no canonical form at all, so
/// nothing may compare two of them for equality in the first place
/// (`docs/decisions.md` §33.8).
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

    /// How far from one a stored normal's squared length `n·n` can sit once a
    /// turn has moved it, and so how far [`Plane::from_point_and_normal_as_written`]
    /// lets a normal sit and still keeps its bits.
    ///
    /// # Where the 64 comes from
    ///
    /// Write `ε = f64::EPSILON`. Two roundings reach a moved plane's normal.
    ///
    /// - **The constructor's.** Each component is the nearest `f64` to the
    ///   exact ratio, a relative slip of at most `ε/2`, so `n·n` is within
    ///   `ε + ε²/4` of one.
    /// - **The turn's.** [`Isometry::POINT_ROUNDOFF`] puts each turned
    ///   component within `18ε` times the largest component of where the turn's
    ///   own arithmetic, done exactly, puts it; a vector has no shift, and its
    ///   largest component is at most its length, one. So the turned vector is
    ///   `m + δ` with `|δ| ≤ √3·18ε`, and `|m + δ|² − |m|² = 2m·δ + |δ|²`, which
    ///   is at most `2√3·18ε ≈ 62.35ε` and a term of order `ε²`.
    ///
    /// Together `63.4ε`, under `64ε`.
    ///
    /// **One share is not derived, and is measured instead.** The turn's own
    /// arithmetic done exactly keeps a length only when its quaternion has
    /// length exactly one, and a stored quaternion comes from `sin` and `cos`,
    /// which Rust does not bound. So the sum above is not a proof that every
    /// turned normal is inside, and
    /// `every_normal_a_turn_leaves_sits_well_inside_the_bound` is the evidence
    /// that the share that was left out fits in the room the worst case
    /// leaves. Over 20 000 normals the worst `|n·n − 1|` it finds is `8.96ε`
    /// turned once, `16.95ε` through four composed turns, `22.16ε` through
    /// sixteen and `20.77ε` through 256, and it asserts every one is under
    /// half the bound.
    ///
    /// **That holds through any number of composed turns because
    /// [`Isometry::compose`] keeps the turn near length one.** Composing two
    /// quaternions rounds, and without compose dividing a product that has
    /// drifted by its length the same test finds `43.51ε` through sixteen and
    /// `240.64ε` through 256, and a plane turned a degree at a time leaves the
    /// bound after 63 steps (`docs/decisions.md` §355.7). A normal moved so far that it leaves the
    /// bound is scaled, which is what every normal was before the bound
    /// existed.
    ///
    /// A normal outside it is not refused, only scaled; see
    /// [`Plane::from_point_and_normal_as_written`] for why.
    ///
    /// **An axis's direction is held to the same bound**, by
    /// [`Axis::as_written`](crate::Axis::as_written) and through the same
    /// check: [`Axis::new`](crate::Axis::new) scales a direction with the same
    /// [`Vector::scaled_to_length_one_from_anywhere`] and
    /// [`Axis::transformed`](crate::Axis::transformed) turns it with the same
    /// [`Isometry::apply_to_vector`], so the two roundings above are the two
    /// that reach a moved axis too.
    pub const NORMAL_LENGTH_ROUNDOFF: f64 = 64.0 * f64::EPSILON;

    /// The plane through `point` at right angles to `normal`.
    ///
    /// Only the direction of `normal` is read. Two vectors along the same line
    /// give the same coefficients bit for bit however long each one is, and
    /// every stored component is the nearest `f64` to the exact ratio between
    /// that component and the exact length —
    /// [`Vector::scaled_to_length_one_from_anywhere`] is what buys both, and
    /// `docs/decisions.md` §121 is why a plane goes through it now.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `point` is not made of finite numbers or if the
    /// plane would sit so far from the origin that its offset overflows, and
    /// [`Error::ZeroLength`] if `normal` has no length to point along.
    pub fn from_point_and_normal(point: Point, normal: Vector) -> Result<Self> {
        Self::through(
            point,
            normal.scaled_to_length_one_from_anywhere("plane normal"),
        )
    }

    /// The plane through `point` at right angles to `normal`, **storing
    /// `normal`'s own bits** when its squared length is within
    /// [`Plane::NORMAL_LENGTH_ROUNDOFF`] of one, and scaling it as
    /// [`Plane::from_point_and_normal`] does when it is not.
    ///
    /// This is for a normal somebody already wrote down as a direction — a
    /// STEP file's `PLANE`, which carries the normal its writer's model
    /// stored. A moved plane's normal is a turn of a unit one and sits an ulp
    /// or two off length one, which [`Plane::transformed`] keeps rather than
    /// scale again. [`Plane::from_point_and_normal`] scales every vector it is
    /// handed, and for most such normals no vector of length exactly one lies
    /// within half an ulp of each component, so no direction a file could hold
    /// reads back as them. The file carries the bits exactly and this keeps
    /// them (`docs/decisions.md` §313.3).
    ///
    /// `n·n − 1` is worked out exactly, and a normal is kept only when it is
    /// proved inside. A component so small that its square cannot be formed
    /// exactly, below about `2⁻⁴⁸⁴`, is charged at the most its square could
    /// be, `2⁻⁹⁶⁸`, which moves the edge by nothing a normal near one can
    /// show.
    ///
    /// **A normal outside the bound is scaled and not refused.** STEP does not
    /// ask for a `DIRECTION` of length one — `(0, 0, 2)` is a correct way to
    /// write "up", and so is a direction written to twelve digits — so a
    /// reader that refused one would refuse correct files, and the only thing
    /// a caller could do with the refusal is ask
    /// [`Plane::from_point_and_normal`] instead. Both answers are the plane
    /// the caller named; which one is stored is a choice about bits, not a
    /// mistake in the input.
    ///
    /// **Two lengths of one direction are not one plane here**, where
    /// [`Plane::from_point_and_normal`] makes them one (§22.6): `(s, 0, s)`
    /// for the `f64` just below `1/√2` is kept, and `(1, 0, 1)` is scaled to
    /// the `f64` just above. Keeping the bits the file wrote is what this is
    /// for, so two faces a file writes along one wall with two different
    /// spellings of its normal come back as two planes an ulp apart.
    ///
    /// # Errors
    ///
    /// The same as [`Plane::from_point_and_normal`]'s.
    pub fn from_point_and_normal_as_written(point: Point, normal: Vector) -> Result<Self> {
        if !has_length_one_to_within_a_turn(normal) {
            return Self::from_point_and_normal(point, normal);
        }
        Self::through(point, Ok(normal))
    }

    /// The plane through `point` along `normal`, which is already of length
    /// one to within [`Plane::NORMAL_LENGTH_ROUNDOFF`].
    ///
    /// `normal` arrives as the `Result` of scaling it, so that a point that is
    /// not a point is named before a normal that is not a direction, whichever
    /// constructor asked.
    fn through(point: Point, normal: Result<Vector>) -> Result<Self> {
        if !point.is_finite() {
            return Err(Error::NotFinite {
                what: "point on a plane",
            });
        }
        let normal = normal?;
        let offset = normal.dot_rounded_once(point - Point::origin());
        if !offset.is_finite() {
            return Err(Error::NotFinite {
                what: "distance from the origin to a plane",
            });
        }
        Ok(Self::holding(normal, offset))
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
    /// the numbers but not the plane. Any two proportional coefficient sets
    /// give the same plane bit for bit: the normal comes back from
    /// [`Vector::scaled_to_length_one_from_anywhere`], which is a function of
    /// the ratios alone, and [`Plane::offset_of`] divides `d` by a coefficient
    /// before it touches anything else, for the same reason.
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
        let coefficients = Vector::new(a, b, c);
        let normal = coefficients.scaled_to_length_one_from_anywhere("plane normal")?;
        let offset = Self::offset_of(coefficients, normal, d);
        if !offset.is_finite() {
            return Err(Error::NotFinite {
                what: "distance from the origin to a plane",
            });
        }
        Ok(Self::holding(normal, offset))
    }

    /// How far `a * x + b * y + c * z + d = 0` sits from the origin along its
    /// unit normal, which is `-d` over the length of `(a, b, c)`.
    ///
    /// That length is never formed. `unit` is `(a, b, c)` already scaled to
    /// length one, so `unitᵢ / coefficientᵢ` is `1 / length` for every `i`, and
    /// the widest coefficient is the one that keeps both of those numbers
    /// furthest from zero — a unit vector's widest component is never below
    /// `1/√3`.
    ///
    /// **Nothing rounds before the division, which is what makes two
    /// proportional coefficient sets agree**, which is what a face being its
    /// plane needs (`docs/decisions.md` §17.1, §22.6). `d · unitᵢ` is formed
    /// exactly and held as a hundred bits, so `(d·unitᵢ) / m` and
    /// `(s·d·unitᵢ) / (s·m)` are the same real number and round to the same
    /// `f64`. Dividing `d` by the coefficient first bought that same agreement
    /// and paid a second rounding for it (`docs/decisions.md` §132).
    fn offset_of(coefficients: Vector, unit: Vector, d: f64) -> f64 {
        let [a, b, c] = coefficients.components();
        let [along_a, along_b, along_c] = unit.components();
        let (widest, unit_there) = if a.abs() >= b.abs() && a.abs() >= c.abs() {
            (a, along_a)
        } else if b.abs() >= c.abs() {
            (b, along_b)
        } else {
            (c, along_c)
        };
        -a_hundred_bit_number_divided_by(product_to_a_hundred_bits(d, unit_there), widest)
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
    /// intersection and the side-of-plane predicate take, and the row every key
    /// over surfaces is built from — so `d` gets the same turn back the right
    /// way up that [`Plane::holding`] gives the four stored numbers. Negating
    /// an offset of `0.0` is the one place a `-0.0` could still reach a key.
    #[must_use]
    pub fn coefficients(&self) -> [f64; 4] {
        let [a, b, c] = self.normal.components();
        [a, b, c, -self.offset + 0.0]
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
    /// If the stored normal has no length. That never happens: every public
    /// constructor, [`Plane::from_point_and_normal`],
    /// [`Plane::from_point_and_normal_as_written`] and
    /// [`Plane::from_coefficients`], checks it — the second keeps only a
    /// normal whose squared length it measured within a hair of one — and
    /// `Deserialize` goes through the last one — `#[serde(try_from = "[f64; 4]")]` makes it the only
    /// route in on the wire. `a_plane_stored_with_no_normal_is_refused_on_the_way_in`
    /// (`crates/rcad-geom/tests/serialization.rs`) shows the bytes refused
    /// before a `Plane` exists to ask.
    ///
    /// **There is a third route in and it is not a constructor.**
    /// [`Plane::transformed`] does not re-normalize — deliberately, so a normal
    /// that was exactly square to something stays exactly square to it
    /// (`docs/decisions.md` §32.3). What keeps that safe is two types away: an
    /// [`Isometry`] turns by a quaternion scaled to length one, whatever scale
    /// its four coefficients arrived at, so it never shrinks a normal to
    /// nothing. `a_rotation_written_at_any_scale_is_the_same_rotation` is that
    /// demonstration, and §41.4 is why an unreachability claim has to name every
    /// route rather than the constructors it thought of.
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

    /// The same flat surface slid along its own normal until it sits `offset`
    /// from the origin, facing along the same bits.
    ///
    /// **Scaling is what this is for**, and it is
    /// [`Axis::moved_to`](crate::Axis::moved_to) for planes. Multiplying a model
    /// by `2^k` multiplies a plane's offset and leaves its normal alone, because
    /// a normal is a direction and not a length — so there is nothing left to
    /// work out and nothing that should round. Going back through
    /// [`Plane::from_coefficients`] instead scales an already-scaled vector to
    /// length one, and that is not the normal that went in: **1 053 of
    /// 100 000** random directions come back with other bits when a plane is
    /// rebuilt from its own coefficients, which
    /// `rebuilding_a_plane_from_its_own_coefficients_tilts_its_normal` counts.
    /// It was **24 733** until §121, because rounding a square root and then
    /// rounding a division moves an already-unit triple far more often than
    /// rounding once does. A twentieth of the footgun is still a footgun.
    /// `rcad-topo`'s `arrange` says the same thing in prose one crate away,
    /// where a moved plane moves a corner.
    ///
    /// That is a footgun and not a broken promise. [`Plane::from_coefficients`]
    /// keeps what it claims — two *exactly* proportional coefficient sets give
    /// the same plane bit for bit — and a stored normal is the rounded
    /// normalizing of whatever built it, so it lies exactly along nothing.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `offset` is not a finite number.
    pub fn moved_to(&self, offset: f64) -> Result<Self> {
        if !offset.is_finite() {
            return Err(Error::NotFinite {
                what: "distance from the origin to a plane",
            });
        }
        Ok(Self::holding(self.normal, offset))
    }

    /// The same flat surface facing the other way.
    ///
    /// Same set of points, opposite normal, so every signed distance flips sign.
    ///
    /// **An involution on the bits**, which
    /// `three_planes_faced_the_one_way` needs and gets from
    /// [`Plane::holding`]: negating alone would turn `0.0` into `-0.0` and
    /// turning round twice would not come back, so a wall facing along an axis
    /// would have two canonical forms and its corner two places.
    #[must_use]
    pub fn reversed(&self) -> Self {
        Self::holding(-self.normal, -self.offset)
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
    ///
    /// **So one plane can come out of two turns as two spellings in a ratio
    /// other than `±1`.** Two blocks turned about `z` by `0.29` then
    /// `0.495 398 163 397 448 4`, and by `0.45` then `0.335 398 163 397 448 05`,
    /// put a face on `x = y` as `(−c, c, 0, 0)` with `c` an ulp apart: exactly
    /// proportional, so one surface to [`Plane::agreement_with`] and to
    /// `intersect`, and neither the same bits nor their negation. A key over
    /// planes that has to call them one folds the ratio, as `rcad-topo`'s
    /// `name_of_the_plane_at_any_scale` does
    /// (`crates/rcad-topo/tests/a_turned_plane_exactly_proportional_to_another.rs`).
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        let normal = movement.apply_to_vector(self.normal);
        Self::holding(
            normal,
            self.offset + normal.dot(movement.translation_part()),
        )
    }

    /// The only place a `Plane` is built, so that no plane anywhere stores
    /// `-0.0` — see the note on [`Plane`] for what rests on that.
    ///
    /// Adding zero is what turns one back the right way up: `-0.0 + 0.0` is
    /// `0.0` and `x + 0.0` is `x` for every other finite `x`, so this rounds
    /// nothing and moves no plane. It is also why moving a plane by
    /// [`Isometry::identity`] still gives back the four numbers that were
    /// stored: a turn that hands back `-0.0` for a component that was `0.0` is
    /// folded here, not carried. `rcad-topo`'s
    /// `name_of_the_surface_either_way_round` folds a key the same way for the
    /// same reason.
    fn holding(normal: Vector, offset: f64) -> Self {
        let [x, y, z] = normal.components();
        Self {
            normal: Vector::new(x + 0.0, y + 0.0, z + 0.0),
            offset: offset + 0.0,
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
    /// The answer is an **exact point where the meeting point is an `f64`
    /// triple and that can be proved, and a constructed point — a box the true
    /// meeting point is somewhere inside — otherwise.** Cramer's rule needs a
    /// division, and dividing two ranges almost never lands on an `f64`, so
    /// most answers are boxes; `docs/decisions.md` §131 is the level that
    /// names the corner when the box holds one, and this doc comment said
    /// "constructed" for every answer until it did.
    ///
    /// Where the width is real it is the point of it. A predicate over a
    /// constructed point cannot fall through to exact arithmetic, so it can
    /// genuinely come back [`Decision::Undecidable`] — this is the
    /// geometry-side entry to the case `docs/decisions.md` §2.5 asks M0 to
    /// reach.
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

/// Whether `normal · normal` is within [`Plane::NORMAL_LENGTH_ROUNDOFF`] of
/// one, **proved** rather than estimated.
///
/// Every square is formed exactly except one too small for its low half to
/// be a normal number, which [`product_is_exact`] turns away. Such a square is
/// below `2⁻⁹⁶⁸`, so it is not added and `2⁻⁹⁶⁸` is charged against the top
/// of the bound instead: the answer is `true` only when the length is inside
/// whatever those squares were. A turned face's normal can carry one —
/// `4.1e-304` in `y` on four of the writer fuzzer's crash files — and was
/// scaled before this was counted.
///
/// `false` for a normal that is not made of finite numbers, and for one with
/// a square past the top of the range: neither is a turned unit vector, and
/// [`Plane::from_point_and_normal`] is what answers for them.
///
/// [`Axis::as_written`](crate::Axis::as_written) asks the same of an axis's
/// direction, so that a plane and an axis a file wrote with one direction keep
/// or scale it alike; for an axis, [`Axis::new`](crate::Axis::new) is what
/// answers when this says `false`.
pub(crate) fn has_length_one_to_within_a_turn(normal: Vector) -> bool {
    const EVERY_SQUARE_TOO_SMALL_TO_FORM_IS_BELOW: f64 =
        f64::MIN_POSITIVE * 9_007_199_254_740_992.0 * 2.0;
    if !normal.is_finite() {
        return false;
    }
    let mut squared_length = Exact::of(0.0);
    let mut left_out = 0.0;
    for component in normal.components() {
        match Exact::product_of(component, component) {
            Some(square) => squared_length = squared_length.plus(&square),
            None if (component * component).is_finite() => {
                left_out += EVERY_SQUARE_TOO_SMALL_TO_FORM_IS_BELOW;
            }
            None => return false,
        }
    }
    let off_one = squared_length.minus(&Exact::of(1.0));
    let room = Exact::of(Plane::NORMAL_LENGTH_ROUNDOFF);
    let not_above = match off_one.plus(&Exact::of(left_out)).minus(&room).sign() {
        Decision::Neg | Decision::Zero => true,
        Decision::Pos | Decision::Undecidable { .. } => false,
        other => unreachable!("unhandled decision from exact arithmetic: {other:?}"),
    };
    let not_below = match off_one.plus(&room).sign() {
        Decision::Pos | Decision::Zero => true,
        Decision::Neg | Decision::Undecidable { .. } => false,
        other => unreachable!("unhandled decision from exact arithmetic: {other:?}"),
    };
    not_above && not_below
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

        // Not needed for a right sign, as the doc on
        // `SIDE_OF_PLANE_ERROR_BOUND` says. What this does is send a product
        // below the normals on to level 2, so that every sign this level gives
        // rests on the relative argument in that doc and not on Gappa's case
        // split alone.
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
/// Every step of that argument is a *relative* one, so it covers only products
/// that are normal or zero, and nothing that overflows.
/// [`SideOfAFlatSurface::on_floats`] returns a number only for those, product
/// by product through [`product_is_normal`], and `decide` keeps out the top end
/// by refusing a bound that is not itself a normal number.
///
/// The bound holds further down than that argument reaches, though. Gappa
/// proves it at this constant, case by case, with one, two or all three
/// products below the normals, any of the others zero, and any of the sums
/// cancelling to zero (`docs/decisions.md` §464.18). A product below the
/// normals loses at most 2⁻¹⁰⁷⁵ when it rounds, and a bound that is a normal
/// number, the only kind `decide` compares against, is 2⁻¹⁰²² or more and puts
/// the permanent at (2⁵⁰ − 1) · 2⁻¹⁰²¹, just under 2⁻⁹⁷¹, or more. A few such
/// losses fit in the constant beside the relative part. With no product
/// normal, the offset is what makes the bound normal, and the whole error is
/// then at most 5 · 2⁻¹⁰⁷⁵. So `on_floats` does not need its
/// [`product_is_normal`] check to give a right sign. It keeps it so that
/// level 1 answers only where the argument above holds.
const SIDE_OF_PLANE_ERROR_BOUND: f64 = {
    let epsilon = f64::EPSILON / 2.0;
    (4.0 + 32.0 * epsilon) * epsilon
};

/// Three exact products of two entries each, and the offset.
const LONGEST_PLANE_TOTAL: usize = 7;

/// Which side of the flat surface through `through` facing `facing` the point
/// is on: the sign of `facing · point − facing · through`, with neither dot
/// rounded.
///
/// [`which_side_of_a_flat_surface`] takes the second dot as one number the
/// caller has already rounded. This takes the point itself, so the surface
/// passes exactly through it whatever `f64` makes of the dot
/// (`docs/decisions.md` §443).
pub(crate) fn which_side_of_a_flat_surface_through(
    named: &'static str,
    through: [f64; 3],
    facing: [f64; 3],
    point: PredicatePoint,
) -> Decision {
    if !(is_a_real_point(point) && are_real_numbers(through) && are_real_numbers(facing)) {
        return NOTHING_IS_KNOWN;
    }
    decide(&SideOfAFlatSurfaceThroughAPoint {
        named,
        through,
        facing,
        point,
    })
    .0
}

/// `facing · point − facing · through`, evaluated three ways for the cascade.
struct SideOfAFlatSurfaceThroughAPoint {
    named: &'static str,
    through: [f64; 3],
    facing: [f64; 3],
    point: PredicatePoint,
}

impl Predicate for SideOfAFlatSurfaceThroughAPoint {
    fn name(&self) -> &'static str {
        self.named
    }

    fn on_floats(&self) -> Option<Filtered> {
        let point = self.point.exact_coordinates()?;
        let [facing, through] = [self.facing, self.through];

        if !(0..3).all(|axis| {
            product_is_normal(facing[axis], point[axis])
                && product_is_normal(facing[axis], through[axis])
        }) {
            return None;
        }

        let terms = [0, 1, 2]
            .map(|axis| facing[axis] * point[axis])
            .into_iter()
            .chain([0, 1, 2].map(|axis| -(facing[axis] * through[axis])));
        let (value, permanent) = terms.fold((0.0, 0.0), |(value, permanent), term: f64| {
            (value + term, permanent + term.abs())
        });

        Some(Filtered {
            value,
            error_bound: SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND * permanent,
        })
    }

    fn on_intervals(&self) -> Interval {
        let point = self.point.enclosure();
        let [facing, through] = [self.facing, self.through];
        let term = |axis: usize| {
            Interval::exactly(facing[axis]) * (point[axis] - Interval::exactly(through[axis]))
        };
        term(0) + term(1) + term(2)
    }

    fn exactly(&self) -> Option<Decision> {
        let point = self.point.exact_coordinates()?;
        let [facing, through] = [self.facing, self.through];

        let mut total = [0.0f64; LONGEST_TOTAL_THROUGH_A_POINT];
        let mut total_len = 0;
        let mut merged = [0.0f64; LONGEST_TOTAL_THROUGH_A_POINT];

        let products = (0..3)
            .map(|axis| (point[axis], facing[axis]))
            .chain((0..3).map(|axis| (-through[axis], facing[axis])));
        for (coordinate, along) in products {
            if !product_is_exact(coordinate, along) {
                return None;
            }
            let (product, roundoff) = two_product(coordinate, along);
            total_len = expansion_sum(&total[..total_len], &[roundoff, product], &mut merged);
            core::mem::swap(&mut total, &mut merged);
        }

        match expansion_sign(&total[..total_len]) {
            Decision::Neg => Some(Decision::Neg),
            Decision::Zero => Some(Decision::Zero),
            Decision::Pos => Some(Decision::Pos),
            // A product overflowed, so this level cannot run.
            Decision::Undecidable { .. } => None,
            other => unreachable!("unhandled decision from exact arithmetic: {other:?}"),
        }
    }
}

/// Level 1's forward error bound for
/// [`SideOfAFlatSurfaceThroughAPoint::on_floats`], as a multiple of the
/// permanent.
///
/// The longest chain of roundings from an input to `value` is one product and
/// then the five additions of the fold, six in all, so the first-order term is
/// `6ε` times the permanent and the higher-order terms of `(1 + ε)⁶ − 1` are
/// `15ε²` and less. The permanent is itself folded from six rounded numbers in
/// five additions, so the computed one can sit up to about `6ε` under the true
/// one, which costs another `36ε²` against the `6ε`, and multiplying by this
/// constant rounds once more, which costs `ε` of the bound and so `6ε²` of the
/// permanent. The `128ε²` covers those `15 + 36 + 6 = 57` with room.
///
/// Gappa proves the bound at this constant (`docs/decisions.md` §464.13), on
/// the sum before `value`'s last rounding, which is all `decide` needs, and on
/// `value` too. With every product normal the least constants it proves on the
/// two are `5ε + 38ε²` and `6ε + 48ε²`. The proof covers every case in which
/// `on_floats` hands `decide` a number, not only that one: [`product_is_normal`]
/// lets a product with a zero factor through, and the floor through the origin
/// makes five of the six zero, yet the bound holds for any set of zero products
/// and with any of the fold's sums cancelling to zero. Whether a product below
/// the normals, which `on_floats` refuses, would break it has not been run.
const SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND: f64 = {
    let epsilon = f64::EPSILON / 2.0;
    (6.0 + 128.0 * epsilon) * epsilon
};

/// Six exact products of two entries each, each two numbers long.
const LONGEST_TOTAL_THROUGH_A_POINT: usize = 12;

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
    use core::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2, TAU};

    use rcad_predicates::{Level, orient3d};

    use super::*;
    use crate::Axis;
    use crate::exact::Exact;

    fn plane_at_height_two() -> Plane {
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 2.0), Vector::new(0.0, 0.0, 1.0)).unwrap()
    }

    /// **The bound [`SIDE_OF_A_FLAT_SURFACE_THROUGH_A_POINT_ERROR_BOUND`]
    /// publishes**, checked against the exact level: wherever the float level
    /// certifies a sign, it is the exact sign. The points sit a few units in the
    /// last place off surfaces through points up to 40 out, where the six
    /// products cancel hardest, and the float level is made to certify some of
    /// them and decline the rest.
    #[test]
    fn the_float_level_never_certifies_a_wrong_side_of_a_surface_through_a_point() {
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
        };
        let (mut certified, mut declined) = (0, 0);
        for _ in 0..20_000 {
            let through = [next() * 40.0, next() * 40.0, next() * 40.0];
            let facing = [next(), next(), next()];
            let wander = [next(), next(), next()];
            let lean = (0..3).map(|axis| wander[axis] * facing[axis]).sum::<f64>()
                / (0..3).map(|axis| facing[axis] * facing[axis]).sum::<f64>();
            let along: [f64; 3] = core::array::from_fn(|axis| wander[axis] - lean * facing[axis]);
            let off_by = next() * f64::EPSILON * 1024.0;
            let point: [f64; 3] = core::array::from_fn(|axis| {
                through[axis] + along[axis] * 10.0 + facing[axis] * off_by
            });
            let asked = SideOfAFlatSurfaceThroughAPoint {
                named: "test",
                through,
                facing,
                point: PredicatePoint::from(point),
            };
            let exact = asked.exactly().expect("no product here is subnormal");
            let filtered = asked.on_floats().expect("every product here is normal");
            if filtered.value.abs() > filtered.error_bound {
                certified += 1;
                let sign = if filtered.value > 0.0 {
                    Decision::Pos
                } else {
                    Decision::Neg
                };
                assert_eq!(sign, exact, "{through:?} {facing:?} {point:?}");
            } else {
                declined += 1;
            }
        }
        assert!(
            certified > 1000 && declined > 1000,
            "{certified} {declined}"
        );
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

    /// [`Plane::NORMAL_LENGTH_ROUNDOFF`] is where
    /// [`Plane::from_point_and_normal_as_written`] stops keeping a normal's
    /// bits, on both sides of one, between two neighbouring `f64`.
    ///
    /// Along an axis `n·n − 1` is `(1 + kε)² − 1 = 2kε + k²ε²`. Above one the
    /// `f64` are `ε` apart: `1 + 31ε` gives `62ε` and a hair, inside, and
    /// `1 + 32ε` gives `64ε + 1024ε²`, just outside. Below one they are `ε/2`
    /// apart: `1 − 32ε` gives `−64ε + 1024ε²`, just inside, and `1 − 32.5ε`
    /// gives `−65ε` and a hair, outside. No `f64` lies between either pair, so
    /// this is the edge as sharp as a normal along an axis can draw it.
    #[test]
    fn a_normal_is_kept_up_to_the_published_bound_and_scaled_past_it() {
        let epsilon = f64::EPSILON;
        let above = [1.0 + 31.0 * epsilon, 1.0 + 32.0 * epsilon];
        let below = [1.0 - 32.0 * epsilon, 1.0 - 32.5 * epsilon];
        for [inside, outside] in [above, below] {
            assert!(
                inside.next_up() == outside || inside.next_down() == outside,
                "{inside} and {outside} are neighbours"
            );
            for (length, kept) in [(inside, true), (outside, false)] {
                let off_one = Exact::product_of(length, length)
                    .expect("a square near one is exact")
                    .minus(&Exact::of(1.0));
                let past_the_bound = off_one.nearest_f64().abs() > Plane::NORMAL_LENGTH_ROUNDOFF;
                assert_eq!(past_the_bound, !kept, "n·n − 1 is {off_one:?} for {length}");

                let written = Vector::new(0.0, length, 0.0);
                let stored =
                    Plane::from_point_and_normal_as_written(Point::new(0.0, 3.0, 0.0), written)
                        .expect("a finite normal with a length")
                        .normal();
                let expected = if kept {
                    written
                } else {
                    Vector::new(0.0, 1.0, 0.0)
                };
                assert_eq!(
                    stored.components().map(f64::to_bits),
                    expected.components().map(f64::to_bits),
                    "a normal {length} long"
                );
            }
        }
    }

    /// The normal of the writer fuzzer's `crash-0ac3e625f698733b9b143d41eb65edbf7087b463`
    /// (2026-09-26, `docs/decisions.md` §313.3): both components the `f64`
    /// just below `1/√2`. Scaling moves it to the `f64` just above, and no
    /// vector of length one rounds to it; keeping it is the only way to store
    /// it.
    #[test]
    fn a_normal_a_turn_left_short_of_length_one_is_kept_as_it_was_written() {
        let short = FRAC_1_SQRT_2.next_down();
        let written = Vector::new(short, 0.0, short);
        let kept = Plane::from_point_and_normal_as_written(Point::origin(), written)
            .expect("a finite normal with a length");
        assert_eq!(
            kept.normal().components().map(f64::to_bits),
            written.components().map(f64::to_bits)
        );
        let scaled = Plane::from_point_and_normal(Point::origin(), written)
            .expect("a finite normal with a length");
        assert_eq!(
            scaled.normal(),
            Vector::new(FRAC_1_SQRT_2, 0.0, FRAC_1_SQRT_2),
            "and scaling it moves both components"
        );
    }

    /// A turned face's normal from four of the writer fuzzer's 2026-09-26
    /// crash files (`crash-024b8e42c5fdb3502f2dbeecc351d9b3cd58dcd0` is one),
    /// whose `x` is too small for its square to be formed exactly. Its square
    /// is charged at the most it could be rather than left unmeasured, so the
    /// normal is kept; scaling it moves `y` down an ulp.
    #[test]
    fn a_normal_with_a_component_too_small_to_square_is_still_kept() {
        let written = Vector::new(
            4.101_256_136_000_473_6e-304,
            -0.825_562_457_895_251_8,
            0.564_310_754_916_075_2,
        );
        assert!(
            !product_is_exact(written.x(), written.x()),
            "the rest of this test rests on this"
        );
        let kept = Plane::from_point_and_normal_as_written(Point::origin(), written)
            .expect("a finite normal with a length")
            .normal();
        assert_eq!(
            kept.components().map(f64::to_bits),
            written.components().map(f64::to_bits),
            "{kept:?}"
        );
        let scaled = Plane::from_point_and_normal(Point::origin(), written)
            .expect("a finite normal with a length")
            .normal();
        assert_ne!(scaled, written, "and scaling it would have moved it");
    }

    /// The evidence behind the share of [`Plane::NORMAL_LENGTH_ROUNDOFF`] that
    /// is measured rather than derived: every normal a turn leaves, turned
    /// once or through 4, 16 or 256 composed placements, sits under half of
    /// it, and each of the four depths is asserted.
    #[test]
    fn every_normal_a_turn_leaves_sits_well_inside_the_bound() {
        let mut state: u64 = 0x2545_F491_4F6C_DD1D;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1_u64 << 53) as f64
        };
        let mut a_turn = || {
            let axis = Axis::new(
                Point::origin(),
                Vector::new(2.0 * next() - 1.0, 2.0 * next() - 1.0, 2.0 * next() - 1.0),
            )
            .expect("a direction with a length");
            Isometry::rotation(&axis, next() * TAU).expect("a finite angle")
        };
        let off_one_in_epsilons = |normal: Vector| {
            let components = normal.components();
            Exact::dot_of(components, components)
                .expect("a unit normal squares exactly")
                .minus(&Exact::of(1.0))
                .nearest_f64()
                / f64::EPSILON
        };

        let depths = [1, 4, 16, 256];
        let mut worst = [0.0_f64; 4];
        for _ in 0..20_000 {
            let written = a_turn().apply_to_vector(Vector::new(1.0, 2.0, 3.0));
            let plane = Plane::from_point_and_normal(Point::origin(), written)
                .expect("a direction with a length");
            for (slot, depth) in depths.into_iter().enumerate() {
                let mut placed = a_turn();
                for _ in 1..depth {
                    placed = placed.compose(&a_turn());
                }
                let moved = plane.transformed(&placed);
                worst[slot] = worst[slot].max(off_one_in_epsilons(moved.normal()).abs());
            }
        }
        let [once, four, sixteen, many] = worst;
        println!(
            "worst |n·n − 1|: {once:.2}ε turned once, {four:.2}ε through four composed turns, \
             {sixteen:.2}ε through sixteen, {many:.2}ε through 256, against a bound of 64ε"
        );
        let bound = Plane::NORMAL_LENGTH_ROUNDOFF / f64::EPSILON;
        for (depth, reached) in depths.into_iter().zip(worst) {
            assert!(
                reached < bound / 2.0,
                "a normal turned through {depth} composed turns reached {reached}ε, more than \
                 half the bound, so the share it leaves out may not fit"
            );
        }
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
    /// **Not one of them is afterwards.** A plane divides its normal by the
    /// normal's own length, and a square root is exact almost never — so the
    /// stored triple is a rounding of the written one however carefully the
    /// division is done. §121 made that division correctly rounded and did not
    /// change this: **a nearer direction is still not the caller's direction.**
    /// Worked out in exact rational arithmetic over the stored bits, all three
    /// miss squareness by one or two last bits of the direction. One of them
    /// reads exactly `0.0` in plain `f64` and the other two read `4.4e−16` and
    /// `−1.1e−16`, all inside what the `f64` dot itself rounds
    /// ([`Plane::SIGNED_DISTANCE_ROUNDOFF`]), so a filter that trusted the `f64`
    /// would call one of them square and be wrong about it; the exact level sees
    /// the sign and refuses, which is what it is for.
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
            let largest = along
                .into_iter()
                .fold(0.0_f64, |so_far, c| so_far.max(c.abs()));
            assert!(
                in_f64.abs() <= Plane::SIGNED_DISTANCE_ROUNDOFF * largest,
                "the miss is inside what the f64 dot itself rounds, which is why \
                 it hides: {in_f64:e}"
            );
        }
    }

    /// **Why §31.4's family of fixes is not one**, demonstrated rather than
    /// argued.
    ///
    /// §31.4 works out that scaling a normal to length one is what loses the
    /// squareness, and offers scaling by a **power of two** instead, which
    /// rounds nothing. The first half here confirms that half of it: a normal
    /// written square to a direction is still square to it after a power-of-two
    /// reduction, where the test above says it is not once a plane has stored
    /// it. §121 made the store correctly rounded, which does not help — the
    /// nearest unit triple is still not the written one.
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

    /// A normal at each end of the `f64` range names the plane the same ratio
    /// names at an ordinary size.
    ///
    /// This used to be the evidence that dividing by the largest component first
    /// widens what can be answered: `(1e308, 1e308, 0)` had a length that
    /// overflowed and normalized to a direction of length zero, and
    /// `(5e-324, 5e-324, 0)` had a length that underflowed and was refused
    /// outright. [`Vector::length`] scales by a power of two now, so neither
    /// happens to any caller and the plane constructors are no longer the only
    /// ones that get an answer.
    ///
    /// The claim it is evidence for now is the surviving one, §22.6: what
    /// [`Vector::scaled_to_length_one_from_anywhere`] gives depends on the
    /// ratios between the components and on nothing else, so a plane written at
    /// either end of the range carries the same four numbers as one written at
    /// one.
    #[test]
    fn a_normal_at_either_end_of_the_range_names_the_plane_its_ratio_names() {
        let ordinary = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 1.0, 0.0))
            .expect("a diagonal normal has length");

        for extreme in [
            Vector::new(1e308, 1e308, 0.0),
            Vector::new(5e-324, 5e-324, 0.0),
        ] {
            assert!(extreme.is_finite());
            assert!(
                extreme.length() > 0.0 && extreme.length().is_finite(),
                "{extreme:?} has a length that is an ordinary number"
            );
            assert_eq!(
                Plane::from_point_and_normal(Point::origin(), extreme)
                    .expect("the ratio 1:1 says which way this faces")
                    .coefficients(),
                ordinary.coefficients(),
                "the plane named by {extreme:?}"
            );
        }
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

    /// [`Plane::offset_of`] rounds once, and dividing before multiplying
    /// rounded twice (`docs/decisions.md` §132, §130.6).
    ///
    /// The two spellings are the same real number and different `f64`. A sweep
    /// of 200 000 random coefficient sets in `[-10, 10]` told them apart 52 355
    /// times — a quarter — so this is an ordinary plane and not a contrived
    /// one.
    #[test]
    fn a_planes_offset_from_its_coefficients_rounds_once_and_not_twice() {
        let coefficients = [
            8.477_658_241_021_57,
            -8.986_484_763_473_53,
            -8.740_426_871_854_805,
            -7.512_026_851_469_46,
        ];
        let plane = Plane::from_coefficients(coefficients).expect("a plane");

        let [_, widest, _, d] = coefficients;
        let unit_there = plane.normal().y();
        assert_eq!(
            -(d / widest) * unit_there,
            0.496_384_149_050_014_7,
            "dividing first and then multiplying rounds twice"
        );
        assert_eq!(
            plane.offset(),
            0.496_384_149_050_014_66,
            "forming the product exactly and dividing it rounds once"
        );
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
    /// about the plane it was given. The third point even has an `f64` distance
    /// of exactly zero while the certified sign against the stored coefficients
    /// is positive.
    ///
    /// **The sides moved when the offset stopped rounding five times**
    /// (`docs/decisions.md` §132): it was `Neg` for all three and the exactly
    /// zero distance was the *first* point's. Which way each one is off is a
    /// fact about the rounding and not about the geometry, so the three are
    /// named one at a time rather than asserted to agree.
    #[test]
    fn the_points_that_defined_a_plane_are_not_on_the_plane_that_gets_stored() {
        let corners = [[1.0, 2.0, 3.0], [4.0, 5.0, 7.0], [2.0, 9.0, 4.0]];
        let [first, second, third] = corners.map(|[x, y, z]| Point::new(x, y, z));
        let plane = Plane::through_three_points(first, second, third).unwrap();

        assert_eq!(plane.signed_distance_to(third), 0.0);
        let certain_and_wrong = [Decision::Pos, Decision::Neg, Decision::Pos];
        for (corner, side) in corners.into_iter().zip(certain_and_wrong) {
            assert_eq!(
                plane.side_of_point(PredicatePoint::from(corner)),
                side,
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

    /// A face that composing a placement moved off the face it was touching is
    /// moved by the arithmetic and not by the summation order.
    ///
    /// The claim `isometry_algebra.rs` and `docs/decisions.md` §41.13 both rest
    /// on. `Plane::transformed` works the moved offset out as `d + n·t`, and for
    /// a face whose placement carries it back through its own offset the two
    /// terms are both about 3 and nearly cancel — so the obvious suspicion is
    /// that the leftover is the `f64` sum's own rounding and that summing more
    /// carefully would make it zero.
    ///
    /// **It would not.** Summed through [`Exact`], which is the arithmetic the
    /// predicates decide on, the leftover is still there and is about two thirds
    /// of the `f64` sum's. The composed placement really does put this plane off
    /// the one it was touching, by less than a rounding of the placement — which
    /// is why nothing downstream is entitled to call it a mistake, and why the
    /// fix is not in this function.
    #[test]
    fn a_placement_that_moves_a_plane_off_its_neighbour_does_it_in_the_arithmetic() {
        let axis = Axis::new(Point::origin(), Vector::new(2.0, 3.0, -2.0)).unwrap();
        let turn = Isometry::rotation(&axis, TAU * 44.0 / 64.0).unwrap();
        let lift = Isometry::translation(Vector::new(0.0, 0.0, -3.0)).unwrap();
        let placed = turn.compose(&lift);
        let stored = Plane::from_coefficients([0.0, 0.0, 1.0, -3.0]).unwrap();

        let moved = stored.transformed(&placed);
        let summed_exactly = Exact::of(stored.offset()).plus(
            &Exact::dot_of(
                moved.normal().components(),
                placed.translation_part().components(),
            )
            .unwrap(),
        );

        assert_eq!(moved.offset(), -8.881784197001252e-16);
        assert_eq!(summed_exactly.nearest_f64(), -5.195804564346705e-16);
        assert_ne!(
            summed_exactly.sign(),
            rcad_predicates::Decision::Zero,
            "no summation reaches the plane this face was touching"
        );
        assert!(
            summed_exactly.nearest_f64().abs() < Isometry::POINT_ROUNDOFF * 3.0,
            "and it is inside what placing a solid of this size is allowed to move it"
        );
    }

    /// However a plane is built, and whatever it is handed, none of the four
    /// numbers it stores comes out `-0.0`.
    ///
    /// A caller may hand in a negative zero — a STEP `DIRECTION` may be written
    /// `(-0.,0.,1.)` and `rcad-step` passes it straight to
    /// [`Plane::from_point_and_normal`] — and the sums inside these
    /// constructors make their own. Every one has to be turned back the right
    /// way up, because `meet_of_three_surfaces` sorts its three planes with
    /// [`f64::total_cmp`](f64::total_cmp) and a wall stored two ways round has
    /// its corner solved in two row orders.
    #[test]
    fn nothing_a_plane_is_built_from_leaves_a_negative_zero_stored() {
        let holds_no_negative_zero = |plane: &Plane, how: &str| {
            for (number, name) in plane.coefficients().iter().zip(["a", "b", "c", "d"]) {
                assert!(
                    !(*number == 0.0 && number.is_sign_negative()),
                    "{how} stored -0.0 as {name}: {:?}",
                    plane.coefficients()
                );
            }
        };

        let from_a_negative_zero_normal =
            Plane::from_point_and_normal(Point::new(1.0, 2.0, 3.0), Vector::new(1.0, -0.0, 1.0))
                .expect("a normal with length");
        holds_no_negative_zero(&from_a_negative_zero_normal, "from_point_and_normal");
        holds_no_negative_zero(
            &Plane::from_coefficients([1.0, -0.0, 1.0, -4.0]).expect("a normal with length"),
            "from_coefficients",
        );
        holds_no_negative_zero(
            &Plane::from_coefficients([0.0, 0.6, 0.8, -2.0])
                .expect("a normal with length")
                .reversed(),
            "reversed",
        );
        holds_no_negative_zero(
            &Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
                .expect("a normal with length"),
            "a plane through the origin facing down",
        );
        holds_no_negative_zero(
            &plane_at_height_two().transformed(&Isometry::identity()),
            "transformed",
        );
    }

    /// A wall built facing one way and the same wall built facing the other are
    /// the same four numbers negated, so turning either round gives the other
    /// bit for bit.
    ///
    /// This is what `three_planes_faced_the_one_way` in `rcad-topo` rests on:
    /// it picks one of a plane's two spellings by the sign of the leading
    /// coefficient, and that picks *one* spelling only when the two spellings
    /// really are a negation apart. A stored `-0.0` in one of them and a `0.0`
    /// in the other would leave two canonical forms, and
    /// `meet_of_three_surfaces` would sort them into two different row orders.
    #[test]
    fn a_wall_built_facing_either_way_is_one_plane_turned_round() {
        let corner = Point::new(1.0, 2.0, 3.0);
        let facing_up = Plane::from_point_and_normal(corner, Vector::new(1.0, -0.0, 1.0))
            .expect("a normal with length");
        let facing_down = Plane::from_point_and_normal(corner, Vector::new(-1.0, 0.0, -1.0))
            .expect("a normal with length");

        assert_eq!(
            facing_up.reversed().coefficients().map(f64::to_bits),
            facing_down.coefficients().map(f64::to_bits),
            "the wall built facing down is the one built facing up, turned round"
        );
        assert_eq!(
            facing_down.reversed().coefficients().map(f64::to_bits),
            facing_up.coefficients().map(f64::to_bits),
            "and the other way about"
        );
        assert_eq!(
            facing_up
                .reversed()
                .reversed()
                .coefficients()
                .map(f64::to_bits),
            facing_up.coefficients().map(f64::to_bits),
            "so turning one round twice is the plane it started as, bit for bit"
        );
    }
}
