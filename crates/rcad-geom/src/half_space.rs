//! Half-spaces, and what picks one piece of a crossing out of the rest.

use rcad_predicates::{Decision, Point as PredicatePoint, SideOfSurface};
use serde::{Deserialize, Serialize};

use crate::exact::Exact;
use crate::plane::{which_side_of_a_flat_surface, which_side_of_a_flat_surface_through};
use crate::point::{Point, Vector};
use crate::{Cylinder, Error, Isometry, Result, Sphere};

/// The name a half-space's sign predicate goes by in provenance records.
pub const SIDE_OF_A_HALF_SPACE: &str = "side of a half-space";

/// Everything on or in front of a flat surface: the points `p` with
/// `normal · p ≥ offset`.
///
/// # The four numbers are the caller's, untouched
///
/// Nothing here reduces, scales or normalizes. Whatever `normal` a caller writes
/// down is the `normal` this holds, bit for bit, and that is the whole reason
/// the type exists. A caller who needs a normal exactly square to a direction it
/// already holds can write one — swap two of that direction's components and
/// negate one, and the dot product becomes two products that are exact negatives
/// — and a [`Plane`](crate::Plane) would round the squareness away on the way
/// in, because it divides by the length of the normal (`docs/decisions.md`
/// §32.3). That division is correctly rounded since §121 and it still rounds:
/// a better normalize stores a nearer direction, not the caller's own bits.
///
/// **The invariant that buys is not one the type system states: a half-space is
/// never a face's geometry and is never compared for face identity.** §17.1
/// rests on two faces that are the same plane holding the same four numbers, and
/// `Plane` earns that by scaling every normal to length one. This deliberately
/// does not, so two half-spaces describing one flat surface may hold quite
/// different numbers — which costs nothing, because nothing ever asks whether
/// two of them are the same. [`Picker`] is what keeps that true: a half-space
/// can only ever be a picker, and a face's surface is a
/// [`Surface`](crate::Surface).
///
/// # What follows from storing the normal untouched
///
/// A piece of a crossing that runs on for ever along `direction` stays inside
/// this half-space, or stays outside it, exactly when `normal · direction` is
/// exactly zero — because the reading at `p + t·direction` is
/// `normal · p − offset + t·(normal · direction)`, and level 3 of the cascade
/// evaluates that with nothing rounded. One rounding in `normal · direction` and
/// a long enough run leaves whichever side it started on.
///
/// **That is a statement about the point `p + t·direction`, and not about the
/// `f64` triple a caller works out for it.** `fl(p + fl(t·direction))` is the
/// true point plus a rounding `ε` whose size grows with `t`, and the reading
/// picks up `normal · ε`, which the squareness says nothing about. Measured, on
/// the run this file's own test walks: a start at `(0, 5, −3)` along `(1, 3, 5)`
/// reads `Pos` up to `t = 2⁵⁴` and `Zero` at `2⁵⁵`, where the sum first rounds.
/// So the squareness buys `t`-independence for a point that is *exactly* on the
/// run, which is what a corner of a model is, and buys nothing for a point that
/// was merely computed towards one.
/// `a_point_far_enough_down_an_endless_run_stops_being_exactly_on_it` is the
/// demonstration.
///
/// The offset does **not** have to be exact for that. A rounded offset moves the
/// flat surface along its own normal without turning it, so it can only be
/// wrong about *where between* two pieces the surface sits, which is a bounded
/// question with an answer (§29.8).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "[f64; 4]", into = "[f64; 4]")]
pub struct HalfSpace {
    normal: Vector,
    offset: f64,
}

impl HalfSpace {
    /// The half-space in front of `normal · p = offset`, storing both as given.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `normal` or `offset` is not a finite number, and
    /// [`Error::ZeroLength`] if `normal` has no length to point along.
    pub fn new(normal: Vector, offset: f64) -> Result<Self> {
        if !(normal.is_finite() && offset.is_finite()) {
            return Err(Error::NotFinite {
                what: "half-space coefficient",
            });
        }
        if normal == Vector::zero() {
            return Err(Error::ZeroLength {
                what: "half-space normal",
            });
        }
        Ok(Self { normal, offset })
    }

    /// The half-space whose flat surface passes through `through` facing
    /// `facing`, with `facing` stored exactly as handed over.
    ///
    /// The offset is `facing · through` rounded once, **from the exact dot and
    /// not from the plain `f64` one**. It still rounds, so the surface passes
    /// through `through` to within that rounding and not exactly, and which side
    /// of it a point far along a direction square to `facing` sits is unaffected
    /// either way — see the note on [`HalfSpace`].
    ///
    /// # Why the exact dot, when the offset is allowed to round
    ///
    /// A plain `f64` dot through a point a long way from the origin is large
    /// products cancelling down to a small answer, so what it rounds by is set
    /// by the size of the **coordinates** and not by the size of the answer.
    /// §150's pair is two pipes with one axis `1e16` out along the direction
    /// they share, where two products of about `1.8e15` cancel to about a
    /// quarter: the plain dot came to `0.5` against a true `0.2865091987060316`,
    /// which put the picking plane a third of a pipe radius out of place and on
    /// the far side of both lines it was meant to separate, and the pair was
    /// refused.
    ///
    /// Rounding from the exact dot is right rather than merely a fix for that
    /// pair. The true `facing · through` does not change when `through` slides
    /// along the flat surface it names, so it is an ordinary-sized number
    /// however far out the point handed over sits, and one rounding of an
    /// ordinary-sized number is an ordinary-sized error.
    /// `the_pair_at_1e16_that_answers_one_way_round` is the measurement.
    ///
    /// [`Exact::dot_of`] declines rather than panicking when a product's low
    /// half would be subnormal, which takes a component and a coordinate
    /// multiplying out to under about `2⁻⁹⁶⁹`. The fallback is the plain dot,
    /// which is what this always used to do: such a caller is no worse off than
    /// before and gets none of the paragraph above.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `through` is not made of finite numbers or the
    /// offset overflows, and [`Error::ZeroLength`] if `facing` has no length.
    pub fn through_a_point_facing(through: Point, facing: Vector) -> Result<Self> {
        if !through.is_finite() {
            return Err(Error::NotFinite {
                what: "point on a half-space",
            });
        }
        let offset = Exact::dot_of(facing.components(), through.coordinates())
            .as_ref()
            .map_or_else(|| facing.dot(through - Point::origin()), Exact::nearest_f64);
        Self::new(facing, offset)
    }

    /// The half-space through `through` whose normal is **exactly** square to
    /// `endless_way` and leans as far towards `towards` as one written that way
    /// can.
    ///
    /// `towards` itself goes in first, because a caller that has worked out the
    /// direction it really wants sometimes finds that direction already exactly
    /// square — every pair whose axes are coordinate directions does — and
    /// nothing beats the direction that was asked for.
    ///
    /// Where it is not, the fallbacks are `endless_way × x̂`, `endless_way × ŷ`
    /// and `endless_way × ẑ`. Each of those is two of `endless_way`'s own
    /// components, one of them negated, and a zero, so its dot with
    /// `endless_way` is two products of the same pair of numbers with opposite
    /// signs and is exactly zero whatever `endless_way` is. This type stores them
    /// unchanged; a [`Plane`](crate::Plane) would not (§32.3).
    ///
    /// **Which one is picked is a choice and not a decision.** The three
    /// fallbacks span the whole plane square to `endless_way`, and the best of
    /// them is never less than `1/√3` aligned with the direction asked for —
    /// about 55° off it at worst. Writing `b̂` for that direction, the best
    /// alignment is `max |b̂ᵢ| / √(1 − ŵᵢ²)`, which is at least `max |b̂ᵢ|`,
    /// which is at least `1/√3` because `b̂` has length one.
    /// `the_best_exactly_square_normal_is_never_less_than_a_third_aligned_with_what_was_asked`
    /// is the test §15.6 asks a published bound for. The worst a search over
    /// whole-number directions can actually find is `1/√2`, on a direction along
    /// a coordinate axis with the asked-for one at 45° between the two normals
    /// that are left — the ordinary gap between a worst case and a drawn one.
    ///
    /// A caller still has to *prove* the piece it wanted is picked, and that
    /// proof is the caller's and is certified. Nothing here branches on a sign
    /// that has not been decided.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `through` is not made of finite numbers or the
    /// offset overflows, and [`Error::ZeroLength`] if `endless_way` has no
    /// length. **`towards` may be anything at all**, zero included: it is only
    /// ever used to rank three normals that are square to `endless_way` however
    /// it is ranked.
    pub fn through_a_point_square_to(
        through: Point,
        endless_way: Vector,
        towards: Vector,
    ) -> Result<Self> {
        if !endless_way.is_finite() {
            return Err(Error::NotFinite {
                what: "the direction a half-space is square to",
            });
        }
        let fallbacks = normals_exactly_square_to(endless_way);
        let best_fallback = fallbacks
            .into_iter()
            .filter(|candidate| *candidate != Vector::zero())
            .max_by(|one, other| {
                how_well_aligned(*one, towards).total_cmp(&how_well_aligned(*other, towards))
            })
            .ok_or(Error::ZeroLength {
                what: "the direction a half-space is square to",
            })?;
        // A zero `towards` is square to everything, vacuously, so it would win
        // this comparison and then be refused for having no length to point
        // along — with three good normals already in hand. A caller that works
        // `towards` out as a projection lands on zero whenever what it projected
        // ran along `endless_way`.
        let best = if towards != Vector::zero() && is_exactly_square(towards, endless_way) {
            towards
        } else {
            best_fallback
        };
        // Negating both numbers rounds nothing, so the one that leans the way
        // the caller asked is exactly as square as the one that does not.
        let leaning_the_right_way = if best.dot(towards) < 0.0 { -best } else { best };
        Self::through_a_point_facing(through, leaning_the_right_way)
    }

    /// Which way the half-space faces, exactly as it was written down. Not
    /// scaled to any length.
    #[must_use]
    pub fn normal(&self) -> Vector {
        self.normal
    }

    /// The right-hand side of `normal · p = offset`.
    ///
    /// Not a distance. It is a distance times the normal's length, and the
    /// normal's length is whatever the caller's was.
    #[must_use]
    pub fn offset(&self) -> f64 {
        self.offset
    }

    /// This half-space, shifted by `by`.
    ///
    /// The normal is carried over bit for bit and only the offset moves, so a
    /// normal written exactly square to the direction a crossing runs along is
    /// still exactly square to it afterwards. [`HalfSpace::transformed`] is the
    /// general movement and that is the one thing it cannot promise.
    ///
    /// The offset gains `normal · by`, which is a dot product and rounds. What
    /// that costs a mesh is
    /// `WHAT_SHIFTING_A_FACE_INTO_ITS_MESHING_FRAME_ROUNDS` over in `rcad-topo`.
    #[must_use]
    pub fn shifted(&self, by: Vector) -> Self {
        Self {
            normal: self.normal,
            offset: self.offset + self.normal.dot(by),
        }
    }

    /// This half-space, moved by `movement`: `(n, d)` becomes
    /// `(Rn, d + Rn·t)`.
    ///
    /// A point `q` is in the moved half-space exactly when the point it came
    /// from is in this one, and `n·Rᵀ(q − t) ≥ d` is `(Rn)·q ≥ d + (Rn)·t`. So
    /// this is a pure function of the four numbers held here and the movement,
    /// and **nothing about the half-space is worked out a second time from
    /// points that have themselves been moved** — which is the whole reason the
    /// function exists rather than callers rebuilding a picker in the new frame.
    /// `a_point_lands_in_the_moved_half_space_exactly_when_it_was_in_this_one`
    /// is what checks the formula itself; the three bullets below are what it
    /// cannot see.
    ///
    /// # What is exact, bit for bit
    ///
    /// - **A movement with no turn is [`HalfSpace::shifted`]**, identically.
    ///   `Isometry::apply_to_vector` hands a vector straight back when there is
    ///   no turn, and the offset is then the same expression.
    ///   `moving_a_half_space_by_a_shift_is_the_shift_itself` is the
    ///   demonstration.
    /// - **Reversing commutes with moving.** Negating a vector negates every
    ///   product and every partial sum of these expressions exactly, so the two
    ///   pickers naming the two pieces of a cut crossing stay exact opposites
    ///   however far the pair is moved — `docs/decisions.md` §19 for pickers,
    ///   the statement `two_faces_that_touch_stay_touching_when_one_movement_moves_both`
    ///   makes for planes.
    /// - **The scale the caller wrote the picker at travels, when it is a power
    ///   of two.** `(2ᵏn, 2ᵏd)` moves to exactly `2ᵏ` times where `(n, d)`
    ///   moves to, because every product and sum on the way scales by exactly
    ///   `2ᵏ` while nothing leaves the normal range. **At any other scale it
    ///   does not** — the quaternion sandwich is not exactly homogeneous in the
    ///   vector it turns, which is the same defect §41.13 measured on a
    ///   composed placement.
    ///   `the_scale_a_picker_is_written_at_travels_through_a_turn_only_when_it_is_a_power_of_two`
    ///   counts both halves.
    ///
    /// # What a turn takes away, and why that is a limit
    ///
    /// A normal written exactly square to a direction is only square to the
    /// moved direction to within a rounding afterwards: `fl(Rn)·fl(Rv)` is not
    /// zero even where `n·v` is. So a picker on a piece of a crossing that runs
    /// on for ever loses the `t`-independence the note on [`HalfSpace`]
    /// describes, and a point far enough along the moved run can read the wrong
    /// side of it.
    ///
    /// **A limit at this function and a gap one level up** (§16.10). Three
    /// families of fix were weighed:
    ///
    /// - *Turn the normal more carefully.* The exact `Rn` is a sum of products
    ///   of the stored quaternion's coefficients and is not an `f64` triple, so
    ///   there is no exactly-turned normal to reach. Exact arithmetic finds the
    ///   number and cannot store it.
    /// - *Bring the normal onto a power of two first.* That buys homogeneity,
    ///   which the bullet above already has, and nothing at all for squareness:
    ///   scaling `n` exactly leaves `fl(Rn)·fl(Rv)` scaled by the same factor
    ///   and just as non-zero.
    /// - *Write a fresh normal exactly square to the moved direction*, which
    ///   [`normals_exactly_square_to`] can always do. That answer exists in
    ///   `f64` — but it needs the direction, and four numbers are all that is
    ///   here. [`EdgeRun::transformed`](crate::EdgeRun::transformed) is where
    ///   the direction is in hand, so that is where the gap is, and closing it
    ///   is a design decision about which of the three normals a moved run
    ///   should be picked by rather than an arithmetic one.
    ///
    /// `a_picker_written_square_to_a_run_is_no_longer_square_to_the_turned_run`
    /// is the demonstration rather than this paragraph.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        let normal = movement.apply_to_vector(self.normal);
        Self {
            normal,
            offset: self.offset + normal.dot(movement.translation_part()),
        }
    }

    /// `[a, b, c, d]` for `a·x + b·y + c·z ≥ d`.
    #[must_use]
    pub fn coefficients(&self) -> [f64; 4] {
        let [a, b, c] = self.normal.components();
        [a, b, c, self.offset]
    }

    /// The other half of the same flat surface.
    ///
    /// Both numbers are negated, which rounds nothing, so a normal that was
    /// exactly square to a direction still is.
    #[must_use]
    pub fn reversed(&self) -> Self {
        Self {
            normal: -self.normal,
            offset: -self.offset,
        }
    }

    /// Which side of the flat surface `point` is on: [`Decision::Pos`] in front
    /// of the normal, [`Decision::Neg`] behind it, [`Decision::Zero`] exactly on
    /// it.
    ///
    /// Exact about the four numbers this holds, in the sense
    /// [`Plane::side_of_point`](crate::Plane::side_of_point) spells out — and
    /// unlike a plane's, those four numbers are the ones the caller wrote.
    #[must_use]
    pub fn side_of_point(&self, point: PredicatePoint) -> Decision {
        which_side_of_a_flat_surface(
            SIDE_OF_A_HALF_SPACE,
            self.normal.components(),
            self.offset,
            point,
        )
    }
}

/// The three vectors square to `direction` that `f64` can hold with nothing
/// rounded: `direction` crossed with each coordinate axis.
///
/// One of them is zero when `direction` runs along a coordinate axis, and never
/// two of them, because a direction with two zero components still has a third.
///
/// [`through_a_point_square_to`](HalfSpace::through_a_point_square_to) ranks
/// these and hands back the best one. A caller that means to *prove* something
/// about the half-space it gets, rather than to take the best guess, wants all
/// three: which of them the ranking puts first is a choice, and §183 measured a
/// pair where the ranking's own winner reads nothing and the one it passed over
/// separates the two lines by `1.99e243`.
pub(crate) fn normals_exactly_square_to(direction: Vector) -> [Vector; 3] {
    let [x, y, z] = direction.components();
    [
        Vector::new(0.0, z, -y),
        Vector::new(-z, 0.0, x),
        Vector::new(y, -x, 0.0),
    ]
}

/// Whether the dot product of two stored triples is exactly zero.
///
/// `None` from [`Exact::dot_of`] means a product would round, so the exact level
/// cannot run at all and nothing here may claim squareness.
fn is_exactly_square(one: Vector, other: Vector) -> bool {
    matches!(
        Exact::dot_of(one.components(), other.components())
            .as_ref()
            .map(Exact::sign),
        Some(Decision::Zero)
    )
}

/// The cosine of the angle between two vectors, as plain arithmetic.
///
/// Only ever used to pick the better of two candidates, so a rounding here costs
/// a slightly worse choice and never a wrong answer.
fn how_well_aligned(one: Vector, other: Vector) -> f64 {
    let lengths = one.length() * other.length();
    if lengths == 0.0 {
        return 0.0;
    }
    (one.dot(other) / lengths).abs()
}

impl TryFrom<[f64; 4]> for HalfSpace {
    type Error = Error;

    fn try_from([a, b, c, d]: [f64; 4]) -> Result<Self> {
        Self::new(Vector::new(a, b, c), d)
    }
}

impl From<HalfSpace> for [f64; 4] {
    fn from(half_space: HalfSpace) -> Self {
        half_space.coefficients()
    }
}

/// What picks one piece of a crossing out of the rest of it.
///
/// A crossing that comes in more than one piece is named by the two surfaces it
/// runs along, which say nothing about *which* piece this is
/// ([`EdgeRun::PartOfWhereItCrosses`](crate::EdgeRun::PartOfWhereItCrosses)).
/// A piece is picked by being on or inside all of these.
///
/// **A picker is never a face's geometry**, which is what lets the flat arm hold
/// a [`HalfSpace`] rather than a [`Plane`](crate::Plane) — see the note there.
/// The curved arms are the surfaces themselves, because a ball or a pipe already
/// divides space in two and holds its own numbers canonically (§24.5).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Picker {
    /// Everything on or in front of a flat surface.
    InFrontOf(HalfSpace),
    /// Everything on or outside a ball.
    OutsideABall(Sphere),
    /// Everything on or outside a pipe.
    OutsideAPipe(Cylinder),
    /// Everything on or inside a pipe.
    ///
    /// **The only picker that says *inside*, and a torus is what needed one.** A
    /// plane square to a torus's axis cuts it in two circles that are
    /// concentric and lie in one plane, and no flat half-space tells those
    /// apart: a half-space holding the whole inner circle holds a disc round the
    /// centre and so holds half the outer one too. Nor does any *outside*: the
    /// pipe of a radius between the two holds every point of the outer circle
    /// and none of the inner, which names one of the pair and leaves the other
    /// with nothing to be named by.
    ///
    /// So the second name is the same pipe read the other way round, which is
    /// what the two arms of [`EdgeRun::PartOfWhereItCrosses`] need and what
    /// `HalfSpace::reversed` already gives the flat arm.
    ///
    /// [`EdgeRun::PartOfWhereItCrosses`]: crate::EdgeRun::PartOfWhereItCrosses
    InsideAPipe(Cylinder),
    /// Everything on or in front of a flat surface that passes exactly through
    /// a point.
    ///
    /// The flat arm's [`HalfSpace`] rounds `facing · through` into its offset,
    /// so it passes exactly through `through` only when `f64` holds that dot.
    /// Bores on `(3, 4, 0)` and `(4, 0, 3)` through `(11, −3.25, 0.5)` have no
    /// facing in their flat for which it does, so this arm is what picks their
    /// halves (`docs/decisions.md` §443). Declared last, so the arms before it
    /// keep the tags a model is written out with.
    InFrontOfAFlatThrough(HalfSpaceThroughAPoint),
}

impl Picker {
    /// Whether `point` is inside this picker, on its boundary, or outside it —
    /// [`Decision::Neg`] for the one piece it rules out.
    ///
    /// [`Decision::Undecidable`] is a real answer here and callers have to take
    /// it as one: a picker that could not tell rules out neither piece, so both
    /// come back picked and the caller refuses rather than guessing.
    #[must_use]
    pub fn side_of_point(&self, point: PredicatePoint) -> Decision {
        match self {
            Picker::InFrontOf(half_space) => half_space.side_of_point(point),
            Picker::OutsideABall(ball) => ball.side_of(point),
            Picker::OutsideAPipe(pipe) => pipe.side_of(point),
            Picker::InsideAPipe(pipe) => match pipe.side_of(point) {
                Decision::Pos => Decision::Neg,
                Decision::Neg => Decision::Pos,
                same => same,
            },
            Picker::InFrontOfAFlatThrough(half_space) => half_space.side_of_point(point),
        }
    }

    /// The other side of the same surface, for the two arms that have one
    /// written exactly: [`HalfSpace::reversed`] and
    /// [`HalfSpaceThroughAPoint::reversed`]. `None` for a curved arm, whose
    /// other side is a different arm and not this one turned round.
    #[must_use]
    pub fn reversed(&self) -> Option<Self> {
        match self {
            Picker::InFrontOf(half_space) => Some(Picker::InFrontOf(half_space.reversed())),
            Picker::InFrontOfAFlatThrough(half_space) => {
                Some(Picker::InFrontOfAFlatThrough(half_space.reversed()))
            }
            Picker::OutsideABall(_) | Picker::OutsideAPipe(_) | Picker::InsideAPipe(_) => None,
        }
    }

    /// Which way a flat arm faces, exactly as it was written down, and `None`
    /// for a curved one.
    #[must_use]
    pub fn facing_of_a_flat(&self) -> Option<Vector> {
        match self {
            Picker::InFrontOf(half_space) => Some(half_space.normal()),
            Picker::InFrontOfAFlatThrough(half_space) => Some(half_space.facing()),
            Picker::OutsideABall(_) | Picker::OutsideAPipe(_) | Picker::InsideAPipe(_) => None,
        }
    }

    /// This picker, shifted by `by`.
    ///
    /// The flat arm keeps its normal bit for bit, which is what
    /// [`HalfSpace::shifted`] is for and what a turn cannot do.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `by` has a component that is not a finite number.
    pub fn shifted(&self, by: Vector) -> Result<Self> {
        Ok(self.transformed(&Isometry::translation(by)?))
    }

    /// This picker, moved by `movement`.
    ///
    /// What each arm promises is what its own type promises:
    /// [`HalfSpace::transformed`] for the flat one, and
    /// [`Sphere::transformed`](crate::Sphere::transformed) and
    /// [`Cylinder::transformed`](crate::Cylinder::transformed) for the curved
    /// ones, which hold a place and a radius and were never square to anything.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        match self {
            Picker::InFrontOf(half_space) => Picker::InFrontOf(half_space.transformed(movement)),
            Picker::OutsideABall(ball) => Picker::OutsideABall(ball.transformed(movement)),
            Picker::OutsideAPipe(pipe) => Picker::OutsideAPipe(pipe.transformed(movement)),
            Picker::InsideAPipe(pipe) => Picker::InsideAPipe(pipe.transformed(movement)),
            Picker::InFrontOfAFlatThrough(half_space) => {
                Picker::InFrontOfAFlatThrough(half_space.transformed(movement))
            }
        }
    }

    /// The name this picker's sign predicate goes by in provenance records.
    #[must_use]
    pub fn predicate_name(&self) -> &'static str {
        match self {
            Picker::InFrontOf(_) | Picker::InFrontOfAFlatThrough(_) => SIDE_OF_A_HALF_SPACE,
            Picker::OutsideABall(_) => crate::SIDE_OF_SPHERE,
            Picker::OutsideAPipe(_) | Picker::InsideAPipe(_) => crate::SIDE_OF_CYLINDER,
        }
    }
}

impl From<HalfSpace> for Picker {
    fn from(half_space: HalfSpace) -> Self {
        Picker::InFrontOf(half_space)
    }
}

impl From<Sphere> for Picker {
    fn from(ball: Sphere) -> Self {
        Picker::OutsideABall(ball)
    }
}

impl From<Cylinder> for Picker {
    fn from(pipe: Cylinder) -> Self {
        Picker::OutsideAPipe(pipe)
    }
}

impl From<HalfSpaceThroughAPoint> for Picker {
    fn from(half_space: HalfSpaceThroughAPoint) -> Self {
        Picker::InFrontOfAFlatThrough(half_space)
    }
}

/// Everything on or in front of the flat surface through `through` facing
/// `facing`: the points `p` with `facing · (p − through) ≥ 0`, asked with
/// nothing rounded.
///
/// A [`HalfSpace`] holds an offset in place of the point, rounded once from
/// the exact dot, so its surface misses `through` wherever `f64` cannot hold
/// `facing · through`. This holds the point, so its surface passes exactly
/// through it for every `facing`. Like a half-space it is only ever a picker,
/// never a face's geometry, and its numbers are the caller's, untouched
/// (`docs/decisions.md` §443).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "(Point, Vector)", into = "(Point, Vector)")]
pub struct HalfSpaceThroughAPoint {
    through: Point,
    facing: Vector,
}

impl HalfSpaceThroughAPoint {
    /// The half-space whose flat surface passes exactly through `through`,
    /// facing `facing`, both stored as given.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if either is not made of finite numbers, and
    /// [`Error::ZeroLength`] if `facing` has no length to point along.
    pub fn new(through: Point, facing: Vector) -> Result<Self> {
        if !(through.is_finite() && facing.is_finite()) {
            return Err(Error::NotFinite {
                what: "point or facing of a half-space",
            });
        }
        if facing == Vector::zero() {
            return Err(Error::ZeroLength {
                what: "half-space facing",
            });
        }
        Ok(Self { through, facing })
    }

    /// The point the flat surface passes exactly through.
    #[must_use]
    pub fn through(&self) -> Point {
        self.through
    }

    /// Which way the half-space faces, exactly as it was written down.
    #[must_use]
    pub fn facing(&self) -> Vector {
        self.facing
    }

    /// The other half of the same flat surface. Only the facing is negated,
    /// which rounds nothing.
    #[must_use]
    pub fn reversed(&self) -> Self {
        Self {
            through: self.through,
            facing: -self.facing,
        }
    }

    /// This half-space, moved by `movement`: the point moved as a point and
    /// the facing turned as a vector.
    ///
    /// A turn rounds both, as it rounds a [`HalfSpace`]'s normal, so the moved
    /// surface passes exactly through the moved `through` as `f64` holds it,
    /// and that point is the true moved point only to within the turn's
    /// rounding. Reversing still commutes with moving: negating a vector
    /// negates every product of the turn exactly.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        Self {
            through: movement.apply_to_point(self.through),
            facing: movement.apply_to_vector(self.facing),
        }
    }

    /// Which side of the flat surface `point` is on: [`Decision::Pos`] in front
    /// of the facing, [`Decision::Neg`] behind it, [`Decision::Zero`] exactly
    /// on it.
    #[must_use]
    pub fn side_of_point(&self, point: PredicatePoint) -> Decision {
        which_side_of_a_flat_surface_through(
            SIDE_OF_A_HALF_SPACE,
            self.through.coordinates(),
            self.facing.components(),
            point,
        )
    }
}

impl TryFrom<(Point, Vector)> for HalfSpaceThroughAPoint {
    type Error = Error;

    fn try_from((through, facing): (Point, Vector)) -> Result<Self> {
        Self::new(through, facing)
    }
}

impl From<HalfSpaceThroughAPoint> for (Point, Vector) {
    fn from(half_space: HalfSpaceThroughAPoint) -> Self {
        (half_space.through, half_space.facing)
    }
}

#[cfg(test)]
mod tests {
    use rcad_predicates::Decision;

    use super::*;
    use crate::{Axis, Plane};

    /// **The property §31.7's first criterion asks for, and §32.3 showed a
    /// `Plane` cannot have.**
    ///
    /// A caller writes a normal exactly square to a direction it already holds
    /// by swapping two of that direction's components and negating one. All
    /// three such normals for `(1, 3, 5)` are exactly square to it before
    /// storage; §32.3 measured that not one of them still is after a `Plane` has
    /// stored it. Every one of them still is after a [`HalfSpace`] has.
    #[test]
    fn a_normal_written_square_to_a_direction_is_still_square_to_it_once_stored() {
        let along = Vector::new(1.0, 3.0, 5.0);
        for written in normals_exactly_square_to(along) {
            let held = HalfSpace::new(written, 0.0)
                .expect("a direction with length has three square normals and two are not zero")
                .normal();
            assert_eq!(held, written, "a half-space stores what it is handed");

            let stored_in_a_plane = Plane::from_point_and_normal(Point::origin(), written)
                .expect("a normal with length")
                .normal();
            assert_ne!(
                exact_dot_sign(held.components(), along.components()),
                None,
                "the dot of two f64 triples is an exact expansion"
            );
            assert_eq!(
                exact_dot_sign(held.components(), along.components()),
                Some(Decision::Zero),
                "{written:?} was written square to {along:?} and a half-space keeps it so"
            );
            assert_ne!(
                exact_dot_sign(stored_in_a_plane.components(), along.components()),
                Some(Decision::Zero),
                "{written:?} stops being square to {along:?} once a plane stores it — §32.3"
            );
        }
    }

    /// A caller that leans nowhere in particular still gets a square normal.
    ///
    /// A zero `towards` is exactly square to everything — the dot of a zero
    /// triple with anything is exactly zero — so it used to win the comparison
    /// against the three fallbacks and then be refused for having no length,
    /// while all three of them sat there square and unused. The search in
    /// `the_best_exactly_square_normal_is_never_less_than_a_third_aligned_with_what_was_asked`
    /// skips this case by hand, which is the tell.
    ///
    /// A caller reaches it by projecting: `towards − along·(towards·along/|along|²)`
    /// is zero for every `towards` that runs along `along`.
    #[test]
    fn a_half_space_asked_to_lean_nowhere_in_particular_is_still_square_to_its_direction() {
        let along = Vector::new(1.0, 3.0, 5.0);
        let picked = HalfSpace::through_a_point_square_to(Point::origin(), along, Vector::zero())
            .expect("three normals square to (1, 3, 5) are in hand before anything is ranked");
        assert_ne!(picked.normal(), Vector::zero());
        assert_eq!(
            exact_dot_sign(picked.normal().components(), along.components()),
            Some(Decision::Zero),
            "what came back has to be square to the direction, whatever it leans towards"
        );
    }

    fn exact_dot_sign(one: [f64; 3], other: [f64; 3]) -> Option<Decision> {
        crate::exact::Exact::dot_of(one, other)
            .as_ref()
            .map(crate::exact::Exact::sign)
    }

    /// The three exactly-square normals span the plane square to a direction, so
    /// whatever direction a caller leans towards, the best of them is well
    /// aligned with it. `1/√3` is the floor
    /// [`HalfSpace::through_a_point_square_to`] publishes, and this is a search
    /// for something under it.
    ///
    /// The search is over every whole-number direction in a nine-by-nine-by-nine
    /// box against every other one, which reaches both awkward shapes: a
    /// direction along a coordinate axis, where one of the three normals is
    /// nothing at all and the other two are a right angle apart, and one along a
    /// diagonal, where the three are evenly spread.
    ///
    /// It finds `1/√2` and never less, which is the first of those: two normals
    /// a right angle apart and a direction asked for half way between them.
    #[test]
    fn the_best_exactly_square_normal_is_never_less_than_a_third_aligned_with_what_was_asked() {
        let mut worst: f64 = 1.0;
        for along in directions_across_the_sphere() {
            for towards in directions_across_the_sphere() {
                let square_to_along =
                    towards - along * (towards.dot(along) / along.length_squared());
                if square_to_along.length() < 1e-6 {
                    continue;
                }
                let best =
                    HalfSpace::through_a_point_square_to(Point::origin(), along, square_to_along)
                        .expect("a direction with length");
                worst = worst.min(how_well_aligned(best.normal(), square_to_along));
            }
        }
        println!("the worst alignment the search found was {worst}");
        assert!(
            worst >= (1.0f64 / 3.0).sqrt() - 1e-12,
            "the best of three normals was only {worst} aligned with what was asked for"
        );
    }

    fn directions_across_the_sphere() -> Vec<Vector> {
        let mut directions = Vec::new();
        for x in -4..=4 {
            for y in -4..=4 {
                for z in -4..=4 {
                    let direction = Vector::new(f64::from(x), f64::from(y), f64::from(z));
                    if direction != Vector::zero() {
                        directions.push(direction);
                    }
                }
            }
        }
        directions
    }

    /// A run that goes on for ever stays on one side of a half-space square to
    /// it, however far along it is read — which is the whole of what the
    /// squareness is for.
    #[test]
    fn a_point_a_long_way_down_an_endless_run_is_on_the_same_side_as_one_at_its_start() {
        let along = Vector::new(1.0, 3.0, 5.0);
        let picker = HalfSpace::through_a_point_square_to(
            Point::origin(),
            along,
            Vector::new(0.0, 5.0, -3.0),
        )
        .expect("a direction with length");
        let start = Point::new(0.0, 5.0, -3.0);

        let at_the_start = picker.side_of_point(PredicatePoint::from(start.coordinates()));
        assert_eq!(at_the_start, Decision::Pos);
        for steps in [1.0, 1e6, 1e12] {
            let far = start + along * steps;
            assert_eq!(
                picker.side_of_point(PredicatePoint::from(far.coordinates())),
                at_the_start,
                "{steps} steps along {along:?} left the half-space"
            );
        }
    }

    /// And far enough along, the point a caller works out is not on the run any
    /// more, so the side it reads is about the rounding rather than about the
    /// half-space.
    ///
    /// The test above stops at a million million steps, where `start + along*t`
    /// still lands on `f64` exactly. Two steps further up it does not: `3·2⁵⁵` has
    /// an `ulp` of sixteen, so adding five to it rounds, and what is asked about
    /// is a point beside the run rather than on it. The squareness has nothing to
    /// say about that point and does not pretend to.
    ///
    /// **Not a defect and not a gap** — the answer about the point handed over is
    /// the right one, and the point handed over is the wrong point. It is here
    /// because the type's own doc used to read as though the guarantee covered
    /// any `t`, and a caller placing a picker a long way down a run would have
    /// believed it.
    #[test]
    fn a_point_far_enough_down_an_endless_run_stops_being_exactly_on_it() {
        let along = Vector::new(1.0, 3.0, 5.0);
        let picker = HalfSpace::through_a_point_square_to(
            Point::origin(),
            along,
            Vector::new(0.0, 5.0, -3.0),
        )
        .expect("a direction with length");
        let start = Point::new(0.0, 5.0, -3.0);

        let near = 2f64.powi(54);
        assert_eq!(
            picker.side_of_point(PredicatePoint::from((start + along * near).coordinates())),
            Decision::Pos,
            "2^54 steps along is still on the side it set off on"
        );

        let far = 2f64.powi(55);
        assert_eq!(
            3.0f64.mul_add(far, 5.0),
            3.0 * far,
            "at 2^55 the five the start stood off by is below the spacing of the grid"
        );
        assert_eq!(
            picker.side_of_point(PredicatePoint::from((start + along * far).coordinates())),
            Decision::Zero,
            "so the point asked about is on the run rather than beside it, and reads Zero"
        );
    }

    /// A point moves into the moved half-space exactly when it was in this one.
    ///
    /// **The formula's own demonstration, and the one thing the three exactness
    /// tests below cannot see.** Every one of them still passes if the offset is
    /// carried over with the *un-turned* normal — `d + n·t` rather than
    /// `d + Rn·t`, which is what copying [`HalfSpace::shifted`] gives. That
    /// half-space is a different flat surface, and only asking where a point
    /// lands says so. Measured by making that change and running the file: this
    /// test fails and the other twelve pass.
    ///
    /// Points nearer the surface than a hundred million roundings are skipped,
    /// because moving a point is only good to
    /// [`Isometry::POINT_ROUNDOFF`](crate::Isometry::POINT_ROUNDOFF) and a point
    /// on the skin may honestly land either side. Both signs are counted, so a
    /// run that drew every point on one side would fail rather than pass on
    /// nothing.
    #[test]
    fn a_point_lands_in_the_moved_half_space_exactly_when_it_was_in_this_one() {
        let mut numbers = SpreadOverFortyOctaves::seeded();
        let (mut in_front, mut behind, mut too_close) = (0, 0, 0);
        for movement in turns_with_a_shift() {
            let facing = numbers
                .a_vector()
                .normalized()
                .expect("a direction from a fixed seed is not exactly zero");
            let half_space =
                HalfSpace::new(facing, numbers.a_number()).expect("a normal with length");
            let moved = half_space.transformed(&movement);
            for _ in 0..4 {
                let place = Point::new(numbers.a_number(), numbers.a_number(), numbers.a_number());
                let size = place
                    .largest_coordinate_magnitude()
                    .max(half_space.offset().abs())
                    .max(1.0);
                let clearance =
                    half_space.normal().dot(place - Point::origin()) - half_space.offset();
                if clearance.abs() < 1e8 * Isometry::POINT_ROUNDOFF * size {
                    too_close += 1;
                    continue;
                }

                let was = half_space.side_of_point(PredicatePoint::from(place.coordinates()));
                let is = moved.side_of_point(PredicatePoint::from(
                    movement.apply_to_point(place).coordinates(),
                ));
                assert_eq!(
                    was, is,
                    "{place:?} read {was:?} of {half_space:?} and {is:?} once both were moved"
                );
                assert!(
                    matches!(was, Decision::Pos | Decision::Neg),
                    "a point clear of the surface read {was:?}"
                );
                if was == Decision::Pos {
                    in_front += 1;
                } else {
                    behind += 1;
                }
            }
        }
        println!("{in_front} points in front, {behind} behind, {too_close} too close to ask");
        assert!(
            in_front > 1_000 && behind > 1_000,
            "only {in_front} in front and {behind} behind, so one side is barely drawn"
        );
    }

    /// §16.10 for [`HalfSpace::transformed`]'s first exactness bullet: a
    /// movement with no turn is [`HalfSpace::shifted`], identically.
    ///
    /// **This is what licenses [`Picker::shifted`] and
    /// [`EdgeRun::shifted`](crate::EdgeRun::shifted) going through
    /// `transformed`.** The mesher rests a whole meshing frame on a shift being
    /// exact, so a shift that quietly started paying for a turn of the identity
    /// would be a mesh floor that understates what a point costs — and nothing
    /// in the mesher would say so.
    ///
    /// Numbers spread over forty octaves, so the dot product has cancellation
    /// to work with as well as none.
    #[test]
    fn moving_a_half_space_by_a_shift_is_the_shift_itself() {
        let mut numbers = SpreadOverFortyOctaves::seeded();
        for _ in 0..20_000 {
            let by = numbers.a_vector();
            let half_space = HalfSpace::new(numbers.a_vector(), numbers.a_number())
                .expect("a normal with length");
            let shift = Isometry::translation(by).expect("a finite shift");
            assert_eq!(
                half_space.transformed(&shift).coefficients(),
                half_space.shifted(by).coefficients(),
                "moving {half_space:?} by a shift of {by:?} was not the shift"
            );
        }
    }

    /// §16.10 for [`HalfSpace::transformed`]'s second exactness bullet.
    ///
    /// Two pieces of one crossing are picked by a half-space and its
    /// [`HalfSpace::reversed`]. If moving them apart gave anything but exact
    /// opposites, a piece would gain a sliver the other lost, which is the
    /// picker's version of the §19 violation §41.13 measured on two face planes.
    #[test]
    fn reversing_a_picker_and_moving_it_give_the_same_four_numbers_either_way() {
        let half_space = HalfSpace::new(Vector::new(0.0, 5.0, -3.0), 2.5).expect("a normal");
        for movement in turns_with_a_shift() {
            assert_eq!(
                half_space.reversed().transformed(&movement).coefficients(),
                half_space.transformed(&movement).reversed().coefficients(),
                "reversing before and after {movement:?} disagreed"
            );
        }
    }

    /// §16.10 for [`HalfSpace::transformed`]'s third exactness bullet, **and for
    /// the sentence next to it saying no other scale travels**.
    ///
    /// A half-space stores whatever length of normal the caller wrote, so
    /// `(n, d)` and `(λn, λd)` are the same flat surface said twice. Where `λ`
    /// is a power of two every product and every sum on the way through a
    /// movement scales by exactly `λ`, so the two moved answers are the same
    /// surface still. Where it is not, they are not: the quaternion sandwich is
    /// not exactly homogeneous in the vector it turns, and that is the defect
    /// §41.13 measured arriving through a composed placement.
    ///
    /// **Both halves are counted.** A run where the scale by three happened to
    /// travel every time would leave the second sentence tested on nothing, and
    /// a run where the power of two never did would be asserting a floor over an
    /// empty set.
    #[test]
    fn the_scale_a_picker_is_written_at_travels_through_a_turn_only_when_it_is_a_power_of_two() {
        let half_space = HalfSpace::new(Vector::new(0.0, 5.0, -3.0), 2.5).expect("a normal");
        let (mut three_travelled, mut three_did_not) = (0, 0);
        for movement in turns_with_a_shift() {
            let moved = half_space.transformed(&movement).coefficients();

            let by_four = scaled_by(half_space, 4.0)
                .transformed(&movement)
                .coefficients();
            assert_eq!(
                by_four,
                moved.map(|coefficient| coefficient * 4.0),
                "writing the same picker four times as big moved it under {movement:?}"
            );

            let by_three = scaled_by(half_space, 3.0)
                .transformed(&movement)
                .coefficients();
            if by_three == moved.map(|coefficient| coefficient * 3.0) {
                three_travelled += 1;
            } else {
                three_did_not += 1;
            }
        }
        println!(
            "a scale of three travelled through {three_travelled} turns and not through \
             {three_did_not}"
        );
        assert!(
            three_did_not > three_travelled,
            "only {three_did_not} of the turns are non-homogeneous, so the sentence saying \
             no other scale travels is barely being tested"
        );
        assert!(
            three_travelled > 0,
            "not one turn carried a scale of three, so this is measuring a broken \
             `transformed` rather than the sandwich"
        );
    }

    /// §16.10 for the limit [`HalfSpace::transformed`] names: a turn takes the
    /// exact squareness away.
    ///
    /// `a_point_a_long_way_down_an_endless_run_is_on_the_same_side_as_one_at_its_start`
    /// above is the property this loses. A picker written exactly square to a
    /// run keeps a point at any distance along that run on the side it started
    /// on; after a turn the same pair reads `fl(Rn)·fl(Rv)`, which is not zero,
    /// and far enough along the moved run the reading changes sign.
    ///
    /// So this counts how many turns keep it and insists most do not. A turn
    /// that kept squareness every time would mean the limit was not a limit and
    /// the doc comment should be deleted rather than the test loosened.
    #[test]
    fn a_picker_written_square_to_a_run_is_no_longer_square_to_the_turned_run() {
        let (mut still_square, mut no_longer) = (0, 0);
        for along in [
            Vector::new(1.0, 3.0, 5.0),
            Vector::new(2.0, -1.0, 7.0),
            Vector::new(0.3, 1.7, -2.9),
        ] {
            let picker =
                HalfSpace::through_a_point_square_to(Point::origin(), along, Vector::zero())
                    .expect("a direction with length");
            assert_eq!(
                exact_dot_sign(picker.normal().components(), along.components()),
                Some(Decision::Zero),
                "the picker has to be square before the turn or this proves nothing"
            );

            for movement in turns_with_a_shift() {
                let moved_normal = picker.transformed(&movement).normal();
                let moved_along = movement.apply_to_vector(along);
                if exact_dot_sign(moved_normal.components(), moved_along.components())
                    == Some(Decision::Zero)
                {
                    still_square += 1;
                } else {
                    no_longer += 1;
                }
            }
        }
        println!("a turn kept the squareness {still_square} times and lost it {no_longer}");
        assert!(
            no_longer > still_square,
            "only {no_longer} turns lost the squareness, so the limit is barely drawn"
        );
    }

    /// The same surface written at a different length of normal.
    fn scaled_by(half_space: HalfSpace, factor: f64) -> HalfSpace {
        HalfSpace::new(half_space.normal() * factor, half_space.offset() * factor)
            .expect("scaling a normal with length leaves it with length")
    }

    /// A turn about every whole-number direction in a seven-cube, at eight
    /// angles, each followed by a shift.
    ///
    /// The shift is there because a turn about the origin leaves the offset with
    /// nothing to do: `Rn · 0` is exactly zero and the offset carries over
    /// whatever the arithmetic does. It is the offset that §41.13 is about.
    fn turns_with_a_shift() -> Vec<Isometry> {
        let shift = Isometry::translation(Vector::new(2.5, -7.25, 11.0)).expect("a finite shift");
        let mut movements = Vec::new();
        for direction in directions_across_the_sphere() {
            let axis = Axis::new(Point::origin(), direction).expect("a direction with length");
            for eighths in 1..=8 {
                let turn =
                    Isometry::rotation(&axis, core::f64::consts::TAU * f64::from(eighths) / 8.0)
                        .expect("a finite angle about a finite axis");
                movements.push(shift.compose(&turn));
            }
        }
        movements
    }

    /// Numbers filling the whole mantissa, over forty octaves, of either sign.
    ///
    /// A number with only a few significant bits rounds far less than one that
    /// fills the word, so a search over those would understate every slip.
    struct SpreadOverFortyOctaves(u64);

    impl SpreadOverFortyOctaves {
        fn seeded() -> Self {
            Self(0x1234_5678_9abc_def0)
        }

        fn a_vector(&mut self) -> Vector {
            Vector::new(self.a_number(), self.a_number(), self.a_number())
        }

        fn a_number(&mut self) -> f64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            let mantissa = (self.0 >> 11) as f64 / (1u64 << 53) as f64;
            let octave = ((self.0 >> 3) % 40) as i32 - 20;
            let sign = if self.0 & 1 == 0 { 1.0 } else { -1.0 };
            sign * (1.0 + mantissa) * 2.0f64.powi(octave)
        }
    }

    /// The four numbers survive being written out and read back, which a plane's
    /// do not.
    #[test]
    fn a_half_space_writes_out_and_reads_back_bit_for_bit() {
        let written = HalfSpace::new(Vector::new(3.0, -1.0, 0.0), 7.0).expect("a normal");
        let bytes = postcard::to_allocvec(&written).expect("a half-space writes out");
        let after: HalfSpace = postcard::from_bytes(&bytes).expect("and reads back");
        assert_eq!(after.coefficients(), written.coefficients());
    }

    #[test]
    fn a_half_space_needs_a_normal_that_has_some_length() {
        assert_eq!(
            HalfSpace::new(Vector::zero(), 1.0),
            Err(Error::ZeroLength {
                what: "half-space normal"
            })
        );
    }

    #[test]
    fn a_picker_answers_for_all_three_of_the_shapes_it_can_be() {
        use crate::{Axis, Cylinder, Sphere};

        let outside = PredicatePoint::from([9.0, 0.0, 0.0]);
        let flat = Picker::from(
            HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 0.0, 0.0))
                .expect("a normal"),
        );
        let ball = Picker::from(Sphere::new(Point::origin(), 1.0).expect("a radius"));
        let pipe = Picker::from(Cylinder::new(Axis::z(), 1.0).expect("a radius"));

        for picker in [flat, ball, pipe] {
            assert_eq!(picker.side_of_point(outside), Decision::Pos);
        }
        assert_eq!(
            ball.side_of_point(PredicatePoint::from([0.0, 0.0, 0.0])),
            Decision::Neg
        );
    }

    /// **What the picker holding a point is for**: the middle of bores on
    /// `(3, 4, 0)` and `(4, 0, 3)`, facing the first one's stored direction.
    /// The half-space through it rounds the dot into its offset and misses it;
    /// the picker holding the point reads it as exactly on its surface.
    #[test]
    fn a_flat_surface_holding_its_point_passes_through_it_where_a_half_space_misses() {
        let middle = Point::new(11.0, -3.25, 0.5);
        let facing = Vector::new(3.0, 4.0, 0.0).normalized().expect("a length");
        let at_the_middle = PredicatePoint::from(middle.coordinates());

        let rounded = HalfSpace::through_a_point_facing(middle, facing).expect("a normal");
        assert_ne!(rounded.side_of_point(at_the_middle), Decision::Zero);

        let held = HalfSpaceThroughAPoint::new(middle, facing).expect("a facing");
        let picker = Picker::from(held);
        assert_eq!(picker.side_of_point(at_the_middle), Decision::Zero);
        assert_eq!(picker.facing_of_a_flat(), Some(facing));
        assert_eq!(picker.predicate_name(), SIDE_OF_A_HALF_SPACE);
    }

    #[test]
    fn the_other_side_of_a_flat_surface_holding_its_point_is_the_other_side() {
        let middle = Point::new(11.0, -3.25, 0.5);
        let facing = Vector::new(4.0, 0.0, 3.0).normalized().expect("a length");
        let picker = Picker::from(HalfSpaceThroughAPoint::new(middle, facing).expect("a facing"));
        let other_side = picker.reversed().expect("a flat picker has another side");
        let in_front = PredicatePoint::from((middle + facing).coordinates());

        assert_eq!(picker.side_of_point(in_front), Decision::Pos);
        assert_eq!(other_side.side_of_point(in_front), Decision::Neg);
        assert_eq!(other_side.reversed(), Some(picker));
        assert_eq!(
            Picker::from(Sphere::new(Point::origin(), 1.0).expect("a radius")).reversed(),
            None
        );
    }

    /// A shift moves the point exactly when nothing rounds, so the moved
    /// surface still passes exactly through it, and reversing commutes with a
    /// turn.
    #[test]
    fn a_flat_surface_holding_its_point_moves_with_it() {
        let middle = Point::new(11.0, -3.25, 0.5);
        let facing = Vector::new(3.0, 4.0, 0.0).normalized().expect("a length");
        let held = HalfSpaceThroughAPoint::new(middle, facing).expect("a facing");

        let shift = Isometry::translation(Vector::new(1.0, 2.0, -4.0)).expect("a shift");
        let shifted = Picker::from(held).transformed(&shift);
        let moved_middle = PredicatePoint::from(Point::new(12.0, -1.25, -3.5).coordinates());
        assert_eq!(shifted.side_of_point(moved_middle), Decision::Zero);

        let turn = Isometry::rotation(&Axis::z(), 0.3).expect("a turn");
        assert_eq!(
            held.reversed().transformed(&turn),
            held.transformed(&turn).reversed()
        );
    }

    #[test]
    fn a_flat_surface_holding_its_point_needs_a_facing_that_has_some_length() {
        assert_eq!(
            HalfSpaceThroughAPoint::new(Point::origin(), Vector::zero()),
            Err(Error::ZeroLength {
                what: "half-space facing"
            })
        );
        assert!(
            HalfSpaceThroughAPoint::new(Point::new(f64::NAN, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
                .is_err()
        );
    }
}
