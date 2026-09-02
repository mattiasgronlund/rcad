//! Half-spaces, and what picks one piece of a crossing out of the rest.

use rcad_predicates::{Decision, Point as PredicatePoint, SideOfSurface};
use serde::{Deserialize, Serialize};

use crate::exact::Exact;
use crate::plane::which_side_of_a_flat_surface;
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
/// in, because it divides by the largest component and then by a square root
/// (`docs/decisions.md` §32.3).
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
    /// The offset is one dot product and rounds once, so the surface passes
    /// through `through` to within that rounding and not exactly. Which side of
    /// it a point far along a direction square to `facing` sits is unaffected —
    /// see the note on [`HalfSpace`].
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
        Self::new(facing, facing.dot(through - Point::origin()))
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
    /// **A shift and not a movement, and the type is saying something by it.**
    /// A half-space stores the normal it was handed without scaling it, because
    /// a picking normal exactly square to the direction a crossing runs along
    /// stops being exactly square the moment anything scales it (§32.3). A shift
    /// carries the normal over untouched and moves only the offset, so
    /// squareness survives; a turn would not, and there is no `transformed` here
    /// for that reason.
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
fn normals_exactly_square_to(direction: Vector) -> [Vector; 3] {
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
        }
    }

    /// This picker, shifted by `by`.
    ///
    /// A shift and not a movement, for the reason [`HalfSpace::shifted`] gives.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `by` has a component that is not a finite number.
    pub fn shifted(&self, by: Vector) -> Result<Self> {
        let shift = Isometry::translation(by)?;
        Ok(match self {
            Picker::InFrontOf(half_space) => Picker::InFrontOf(half_space.shifted(by)),
            Picker::OutsideABall(ball) => Picker::OutsideABall(ball.transformed(&shift)),
            Picker::OutsideAPipe(pipe) => Picker::OutsideAPipe(pipe.transformed(&shift)),
        })
    }

    /// The name this picker's sign predicate goes by in provenance records.
    #[must_use]
    pub fn predicate_name(&self) -> &'static str {
        match self {
            Picker::InFrontOf(_) => SIDE_OF_A_HALF_SPACE,
            Picker::OutsideABall(_) => crate::SIDE_OF_SPHERE,
            Picker::OutsideAPipe(_) => crate::SIDE_OF_CYLINDER,
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

#[cfg(test)]
mod tests {
    use rcad_predicates::Decision;

    use super::*;
    use crate::Plane;

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
}
