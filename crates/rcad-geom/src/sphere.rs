//! Ball surfaces.

use rcad_predicates::expansion::{expansion_sign, expansion_sum, product_is_exact, two_product};
use rcad_predicates::{
    Decision, Filtered, Interval, NOTHING_IS_KNOWN, Point as PredicatePoint, Predicate,
    SideOfSurface, are_real_numbers, decide, is_a_real_point, product_is_normal,
};
use serde::{Deserialize, Serialize};

use crate::exact::square_of_expansion;
use crate::isometry::Isometry;
use crate::point::{Point, Vector};
use crate::{Error, Result};

/// The name the sphere's sign predicate goes by in provenance records.
pub const SIDE_OF_SPHERE: &str = "side of sphere";

/// A ball's skin: every point exactly `radius` away from `centre`.
///
/// Ships as a type and a sign predicate. Nothing in M0 builds a face on one —
/// `docs/decisions.md` §3.1 keeps M0 planar — so [`SideOfSurface::side_of`] is
/// the whole of what it does.
///
/// There is deliberately no `signed_distance_to` here. An `f64` distance would
/// be a second, quieter answer to exactly the question the predicate exists to
/// answer, and callers would reach for the cheap one.
///
/// Serializes as `(centre, radius)` and is re-checked on the way in.
///
/// # No stored number is `-0.0`
///
/// Two faces that are the same ball hold the same four numbers, and the fuse
/// rests on it. [`Sphere::holding`] turns every stored zero the right way up
/// and every constructor here goes through it; see it for what a single
/// negative zero costs and for where one comes from.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "(Point, f64)", into = "(Point, f64)")]
pub struct Sphere {
    centre: Point,
    radius: f64,
}

impl Sphere {
    /// The ball of the given radius around `centre`.
    ///
    /// # Errors
    ///
    /// [`Error::NotFinite`] if `centre` is not made of finite numbers, and
    /// [`Error::RadiusNotPositive`] if the radius is zero, negative, or not a
    /// finite number.
    pub fn new(centre: Point, radius: f64) -> Result<Self> {
        if !centre.is_finite() {
            return Err(Error::NotFinite {
                what: "sphere centre",
            });
        }
        if !(radius.is_finite() && radius > 0.0) {
            return Err(Error::RadiusNotPositive { radius });
        }
        Ok(Self::holding(centre, radius))
    }

    /// The point in the middle.
    #[must_use]
    pub fn centre(&self) -> Point {
        self.centre
    }

    /// How far the surface is from the centre. Always greater than zero.
    #[must_use]
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// The same ball after `movement` is applied. A rigid movement cannot
    /// change the radius.
    #[must_use]
    pub fn transformed(&self, movement: &Isometry) -> Self {
        Self::holding(movement.apply_to_point(self.centre), self.radius)
    }

    /// The only place a `Sphere` is built, so that no ball anywhere stores
    /// `-0.0`.
    ///
    /// Two faces that are the same ball have to hold the same numbers, and four
    /// places rest on it: [`Surface::how_it_sorts_against`](crate::Surface) puts
    /// a triple in a fixed order on the centre and the radius, `rcad-topo`'s
    /// `name_of_surface_bits` names a corner by them, `canonical_cut_surfaces`
    /// drops a repeat by them, and `docs/decisions.md` §32.5 is the rule itself.
    /// All four order with [`f64::total_cmp`](f64::total_cmp), which sorts
    /// `-0.0` before `0.0`, so a single negative zero in a centre makes two
    /// spellings of one ball two balls.
    ///
    /// [`Sphere::new`] is public and stores the centre it is handed, and that
    /// public door is what this rests on rather than a file that walks through
    /// it. The file route exists and is not exercised: three of the checked
    /// corpus's files hold a three-number `CARTESIAN_POINT` with a `-0.`
    /// component — `(-4.6,2.85,-0.)` is one — and `rcad-step` passes a
    /// placement's three numbers through as they came, but none of those three
    /// files carries a `SPHERICAL_SURFACE` for such a point to be the centre of.
    ///
    /// A turn is folded here rather than argued away. [`Isometry::apply_to_point`]
    /// takes one of two roads. Rodrigues' form ends `(… + centreᵢ) + shiftᵢ`,
    /// the same centre coordinate carried through to the last addition, and
    /// an addition gives `-0.0` only when both sides are already `-0.0`, so a
    /// centre folded on the way in stays folded there — `nalgebra`'s own
    /// arithmetic, not this crate's. The other road, one of the 24 turns that
    /// take a cube onto itself (`apply_to_point`'s own doc names it), moves
    /// each coordinate onto another and negates some instead of rounding, so
    /// a folded centre can arrive at coordinate `i` as `−centreⱼ`: negating a
    /// folded `+0.0` gives `-0.0`, and `-0.0 + -0.0` is `-0.0`, so that
    /// argument does not cover those 24 turns — and this time the arithmetic
    /// is this crate's own. The fold here does not rest on either argument:
    /// it turns whatever either road hands back the right way up on its own,
    /// which is the reason it is done here rather than argued away.
    ///
    /// The radius is not folded because it cannot be a zero of either sign:
    /// [`Sphere::new`] refuses anything not greater than zero, and a rigid
    /// movement carries the radius over untouched.
    ///
    /// Adding zero is what turns a negative zero back the right way up:
    /// `-0.0 + 0.0` is `0.0` and `x + 0.0` is `x` for every other finite `x`, so
    /// this rounds nothing and moves no ball. [`Plane::holding`](crate::Plane)
    /// and [`Axis::holding`](crate::Axis) fold the same way for the same reason.
    fn holding(centre: Point, radius: f64) -> Self {
        let [x, y, z] = centre.coordinates();
        Self {
            centre: Point::new(x + 0.0, y + 0.0, z + 0.0),
            radius,
        }
    }

    /// The point at longitude `round` and latitude `up`, both in radians.
    ///
    /// `round` is measured from `+x` towards `+y` and `up` from the equator
    /// towards `+z`, so `(0, 0)` is on `+x` and `up = ±π/2` are the poles.
    ///
    /// A ball has no axis of its own to take a frame from — it is the same ball
    /// however it is turned — so the frame is the world's, fixed. One
    /// consequence is worth stating because it looks like a bug otherwise:
    /// turning a ball does **not** turn its parameterization, since there is
    /// nothing about the ball for the turn to act on.
    ///
    /// Only exactly on the skin where the sines and cosines are exact, which in
    /// practice means `(0, 0)`. **Not even at the poles**, where `cos(π/2)` is
    /// `6.1e-17` rather than zero and this lands a hair off. A ball's poles are
    /// stored exactly instead, and a mesher takes them from the wire's pole
    /// steps rather than from here.
    #[must_use]
    pub fn point_at(&self, round: f64, up: f64) -> Point {
        let ring = up.cos() * self.radius;
        self.centre
            + Vector::new(
                ring * round.cos(),
                ring * round.sin(),
                up.sin() * self.radius,
            )
    }

    /// Which longitude and latitude `point` sits at — [`Sphere::point_at`] the
    /// other way round.
    ///
    /// Only the *direction* from the centre is read, so a point off the skin
    /// gets the coordinates of the point on the skin nearest it, which for a
    /// ball is the one straight out through it. How far out it was is thrown
    /// away.
    ///
    /// Longitude comes back on `atan2`'s branch, `(-π, π]`, and how close it
    /// gets is [`Surface::ROUND_TRIP_ROUNDOFF`](crate::Surface::ROUND_TRIP_ROUNDOFF).
    ///
    /// **At a pole the longitude is not determined by the geometry at all** —
    /// every longitude names that one point — and zero is what comes back. It
    /// is as right an answer as any other, and it falls out of `atan2(0, 0)`
    /// rather than out of a branch. The centre has no nearest point on the skin
    /// whatsoever and comes back `(0, 0)`, for the same reason and by the same
    /// arithmetic (`docs/decisions.md` §25.2).
    #[must_use]
    pub fn coordinates_of(&self, point: Point) -> (f64, f64) {
        let [x, y, z] = (point - self.centre).components();
        (y.atan2(x), z.atan2(x.hypot(y)))
    }
}

/// Inside, on, or outside, decided through the cascade.
///
/// The sign of `|point - centre|² - radius²`: [`Decision::Neg`] inside,
/// [`Decision::Pos`] outside, [`Decision::Zero`] exactly on the skin.
///
/// A point with exact `f64` coordinates always gets a certified answer, as long
/// as no product overflows or falls into the subnormals. A constructed point is
/// only known to lie inside a box, so when that box straddles the skin the
/// answer is [`Decision::Undecidable`] and no amount of precision will change
/// it (`docs/decisions.md` §2.5).
///
/// The radius is taken as written. A sphere whose radius came from a
/// calculation carries whatever rounding that calculation left in it, and this
/// predicate answers about the sphere it was handed, not the one that was meant.
///
/// A point or a sphere that is not made of real numbers gets
/// [`NOTHING_IS_KNOWN`] before the cascade runs. [`Sphere::transformed`] cannot
/// fail and can overflow, so a sphere built from finite numbers can be holding
/// an infinite centre by the time it reaches here.
///
/// Alone among the six predicates in this kernel, this one already refused such
/// input on its own, at all three levels. Level 2 works out
/// `Σ (pointᵢ − centreᵢ)² − radius²`, so a coordinate that is not a real number
/// enters only as a square inside a sum, with no zero coefficient anywhere to
/// multiply it away: the sum comes out as an enclosure of everything and the
/// honest `Undecidable` falls out of it. Level 1 hands over because its error
/// bound is then not a normal number, and level 3 because
/// [`expansion_sign`] refuses an expansion that is not finite. So the guard
/// changes no answer here today. It is here so all six refuse the same input
/// the same way, and so a later change to that expression cannot quietly
/// reopen the hole.
impl SideOfSurface for Sphere {
    fn side_of(&self, point: PredicatePoint) -> Decision {
        let sphere_is_real =
            are_real_numbers(self.centre.coordinates()) && are_real_numbers([self.radius]);
        if !(is_a_real_point(point) && sphere_is_real) {
            return NOTHING_IS_KNOWN;
        }
        decide(&HowFarFromTheSkin {
            sphere: *self,
            point,
        })
        .0
    }
}

/// `|point - centre|² - radius²`, evaluated three ways for the cascade.
struct HowFarFromTheSkin {
    sphere: Sphere,
    point: PredicatePoint,
}

impl Predicate for HowFarFromTheSkin {
    fn name(&self) -> &'static str {
        SIDE_OF_SPHERE
    }

    fn on_floats(&self) -> Option<Filtered> {
        let point = self.point.exact_coordinates()?;
        let centre = self.sphere.centre.coordinates();

        let gaps = [0, 1, 2].map(|axis| point[axis] - centre[axis]);
        let radius = self.sphere.radius;

        // Not needed for a right sign, as the doc on
        // `SIDE_OF_SPHERE_ERROR_BOUND` says. What this does is send a square
        // below the normals on to level 2, so that every sign this level gives
        // rests on the relative argument in that doc and not on Gappa's case
        // split alone.
        if !(gaps.iter().all(|&gap| product_is_normal(gap, gap))
            && product_is_normal(radius, radius))
        {
            return None;
        }

        let squares = gaps.map(|gap| gap * gap);
        let radius_squared = radius * radius;

        // Every square is positive, so the only cancellation is the final
        // subtraction. The permanent is therefore the plain sum of the four
        // magnitudes.
        let permanent = squares[0] + squares[1] + squares[2] + radius_squared;

        Some(Filtered {
            value: squares[0] + squares[1] + squares[2] - radius_squared,
            error_bound: SIDE_OF_SPHERE_ERROR_BOUND * permanent,
        })
    }

    fn on_intervals(&self) -> Interval {
        let point = self.point.enclosure();
        let centre = self.sphere.centre.coordinates();
        let largest = largest_magnitude(&point, &[centre, [self.sphere.radius, 0.0, 0.0]]);
        without_overflow(largest, |scale| {
            let squared_gap = |axis: usize| {
                squared(point[axis] * scale - Interval::exactly(centre[axis]) * scale)
            };
            let radius = Interval::exactly(self.sphere.radius) * scale;
            squared_gap(0) + squared_gap(1) + squared_gap(2) - squared(radius)
        })
    }

    fn exactly(&self) -> Option<Decision> {
        let point = self.point.exact_coordinates()?;
        let centre = self.sphere.centre.coordinates();

        let mut total = [0.0f64; LONGEST_SPHERE_TOTAL];
        let mut total_len = 0;
        let mut merged = [0.0f64; LONGEST_SPHERE_TOTAL];
        let mut squared_gap = [0.0f64; LONGEST_SQUARED_GAP];

        for (coordinate, middle) in point.into_iter().zip(centre) {
            let gap = crate::exact::difference(coordinate, middle);
            let gap_len = square_of_expansion(&gap, &mut squared_gap)?;
            total_len = expansion_sum(&total[..total_len], &squared_gap[..gap_len], &mut merged);
            core::mem::swap(&mut total, &mut merged);
        }

        let radius = self.sphere.radius;
        if !product_is_exact(radius, radius) {
            return None;
        }
        let (radius_squared, roundoff) = two_product(radius, radius);
        total_len = expansion_sum(
            &total[..total_len],
            &[-roundoff, -radius_squared],
            &mut merged,
        );

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

/// The largest magnitude among a point's box and some exact lengths: what
/// [`without_overflow`] sizes its scale from.
pub(crate) fn largest_magnitude(point: &[Interval; 3], lengths: &[[f64; 3]]) -> f64 {
    let in_the_box = point
        .iter()
        .flat_map(|side| [side.lo().abs(), side.hi().abs()]);
    let written_down = lengths.iter().flatten().map(|length| length.abs());
    in_the_box.chain(written_down).fold(0.0, f64::max)
}

/// A level-2 evaluation of a quadric's side-of-surface expression, run a second
/// time with every length scaled down when the first run overflowed.
///
/// `skin_gap` gets the scale to multiply every *length* by — point, centre or
/// origin, radii — and nothing else: not a direction, not a slope. Each
/// quadric's expression is a sum of terms of one and the same degree in the
/// lengths, so scaling them all by `2⁻ᵏ` scales the result by a positive power
/// of two and keeps its sign.
///
/// The scale is an exact power of two, so multiplying by it is exact until a
/// length falls below the normal range, and there `inari` rounds outward: the
/// scaled box still holds the scaled value, so underflow can widen the answer
/// but never flip its sign. The scale brings the largest magnitude into
/// `[1, 4)`, so only a length under about `2⁻¹⁰²²` of the largest can
/// underflow. Where the sign hangs on such a length — the corner
/// `(2⁶⁰⁰, 2⁻⁵⁰⁰, 0)` on a ball of radius `2⁶⁰⁰` about the origin — the answer
/// stays undecided, which is what it was before the scaling too.
///
/// The second run happens only when the first one straddles zero with an
/// infinite width. An answer that is decided, or finite, is kept as it was, so
/// this changes no answer that did not overflow. A ball of radius `1e200` is
/// what it is for: a corner on it squares to infinity plain, and to a finite
/// straddle scaled (`docs/decisions.md` §189.1).
pub(crate) fn without_overflow(largest: f64, skin_gap: impl Fn(Interval) -> Interval) -> Interval {
    let plain = skin_gap(Interval::exactly(1.0));
    if plain.width().is_finite() || !plain.contains_zero() {
        return plain;
    }
    match power_of_two_bringing_down(largest) {
        Some(scale) => skin_gap(Interval::exactly(scale)),
        None => plain,
    }
}

/// `2⁻ᵉ` for the exponent `e` of `largest`, held to `e <= 1022` so the power
/// stays a normal number: `largest` times it lands in `[1, 4)`. `None` when
/// `largest` is not a normal number, which cannot have overflowed anything.
fn power_of_two_bringing_down(largest: f64) -> Option<f64> {
    if !largest.is_normal() {
        return None;
    }
    let biased_exponent = (largest.to_bits() >> 52) & 0x7ff;
    let exponent = (biased_exponent as i64 - 1023).min(1022);
    let power = f64::from_bits(((1023 - exponent) as u64) << 52);
    assert!(
        power.is_normal() && (1.0..4.0).contains(&(largest * power)),
        "2^-{exponent} does not bring {largest} into [1, 4)"
    );
    Some(power)
}

/// `value²`, which unlike `value * value` knows both factors are one number.
///
/// Interval multiplication takes its two factors as independent, so a range
/// that straddles zero squares to `[lo·hi, max(lo², hi²)]` — reaching below
/// zero, where no square is. For a huge range that lower end is `-inf`, and one
/// such term turns a whole sum of squares into an enclosure of everything:
/// fuse seed `3482a32e` half 0, a corner at `x = 4.7e226` outside a ball of
/// radius 5, came back undecided for exactly that (`docs/decisions.md` §189.1).
pub(crate) fn squared(value: Interval) -> Interval {
    if !value.contains_zero() {
        return value * value;
    }
    let reach = Interval::exactly(value.lo().abs().max(value.hi()));
    Interval::new(0.0, (reach * reach).hi())
        .expect("a square's upper end is a number at or above zero")
}

/// Level 1's forward error bound, as a multiple of the permanent.
///
/// `epsilon` is the `f64` unit roundoff, 2⁻⁵³ — half an ulp of 1, and so the
/// most a single rounding can cost in relative terms.
///
/// Where the 6 comes from: follow the longest chain of roundings from an input
/// coordinate to `value` in [`HowFarFromTheSkin::on_floats`]. A gap
/// `point - centre` is one rounding. Squaring it is a second, and it multiplies
/// two copies of an already-rounded gap, so it carries a third. Folding the
/// three squares together takes two additions, a fourth and a fifth, and the
/// subtraction of the radius term is a sixth. Six roundings, each costing at
/// most a factor of `(1 + epsilon)`, come to
/// `(1 + epsilon)^6 - 1 <= 6*epsilon + 15*epsilon^2 + ...` times the permanent.
///
/// The radius term travels a shorter chain — one rounding to square it and one
/// to subtract it — so the six covers it too. The `48*epsilon^2` covers the
/// higher-order terms and the rounding of the permanent itself, with room to
/// spare. The higher-order terms start at `15*epsilon^2`. The permanent adds
/// the four rounded squares in three additions, so it can sit up to about
/// `3*epsilon` under their true sum, which costs `18*epsilon^2` against the
/// `6*epsilon`, and multiplying it by this constant rounds once more, which
/// costs `epsilon` of the bound and so `6*epsilon^2` of the permanent. What the
/// `48` has to beat is `15 + 18 + 6 = 39`. Gappa proves the bound at this
/// constant (`docs/decisions.md` §464.14); with every square normal the least
/// constants it proves are `5*epsilon + 28*epsilon^2` on the sum before
/// `value`'s last rounding, which is all `decide` needs, and
/// `6*epsilon + 30*epsilon^2` on `value`.
///
/// Every step of that argument is a *relative* one, so it covers only squares
/// that are normal or zero, and nothing that overflows.
/// [`HowFarFromTheSkin::on_floats`] returns a number only for those, square by
/// square through [`product_is_normal`], and `decide` keeps out the top end by
/// refusing a bound that is not itself a normal number.
///
/// The bound holds further down than that argument reaches, though. Gappa
/// proves it at this constant, case by case, with any of the four squares below
/// the normals so long as one square is normal; with none normal the bound is
/// not a normal number, so `decide` refuses it (`docs/decisions.md` §464.14).
/// So `on_floats` does not need its [`product_is_normal`] check to give a right
/// sign. It keeps it so that level 1 answers only where the argument above
/// holds.
const SIDE_OF_SPHERE_ERROR_BOUND: f64 = {
    let epsilon = f64::EPSILON / 2.0;
    (6.0 + 48.0 * epsilon) * epsilon
};

/// A gap between two coordinates is two entries, so its square is at most four
/// scaled pairs.
const LONGEST_SQUARED_GAP: usize = 8;

/// Three squared gaps and a squared radius, exactly.
const LONGEST_SPHERE_TOTAL: usize = 3 * LONGEST_SQUARED_GAP + 2;

impl TryFrom<(Point, f64)> for Sphere {
    type Error = Error;

    fn try_from((centre, radius): (Point, f64)) -> Result<Self> {
        Self::new(centre, radius)
    }
}

impl From<Sphere> for (Point, f64) {
    fn from(sphere: Sphere) -> Self {
        (sphere.centre, sphere.radius)
    }
}

#[cfg(test)]
mod tests {
    use core::cmp::Ordering;

    use rcad_predicates::{Level, decide, meet_of_three_planes};

    use super::*;
    use crate::{Surface, Vector};

    fn unit_sphere() -> Sphere {
        Sphere::new(Point::origin(), 1.0).unwrap()
    }

    fn bits_of(ball: Sphere) -> [u64; 4] {
        let [x, y, z] = ball.centre().coordinates();
        [x, y, z, ball.radius()].map(f64::to_bits)
    }

    /// Every way of signing the zeros of one centre, through the public door.
    ///
    /// `-0.0` is the same number as `0.0` and a different `f64`, and everything
    /// that asks whether two faces are the same surface compares with
    /// [`f64::total_cmp`](f64::total_cmp), which sorts `-0.0` first. Nothing
    /// between [`Sphere::new`]'s argument and the field it lands in changes a
    /// sign — there is no scaling on a centre, the way there is on a normal or
    /// an axis direction — so what a caller writes is what would be stored.
    #[test]
    fn a_ball_holds_the_same_bits_however_its_centres_zeros_were_signed() {
        let plain = Sphere::new(Point::new(0.0, 0.0, 4.0), 2.0).unwrap();

        for x in [0.0, -0.0] {
            for y in [0.0, -0.0] {
                let spelt = Sphere::new(Point::new(x, y, 4.0), 2.0).unwrap();
                assert_eq!(
                    bits_of(spelt),
                    bits_of(plain),
                    "a ball centred on ({x}, {y}, 4) holds other bits than the same ball \
                     centred with every zero positive"
                );
            }
        }
    }

    /// The bits a ball holds are the bits a face on it sorts by.
    ///
    /// `Surface::how_it_sorts_against` reads the centre straight out, and so
    /// does `rcad-topo`'s `name_of_surface_bits`, which names a corner by them
    /// one crate away. Two spellings of one ball that sort apart are two
    /// surfaces to both of them, and the corner where they meet a third lands
    /// in two places.
    #[test]
    fn one_ball_spelled_two_ways_sorts_as_one() {
        let plain = Surface::Sphere(Sphere::new(Point::new(0.0, 0.0, 4.0), 2.0).unwrap());
        let with_a_negative_zero =
            Surface::Sphere(Sphere::new(Point::new(-0.0, 0.0, 4.0), 2.0).unwrap());

        assert_eq!(
            plain.how_it_sorts_against(&with_a_negative_zero),
            Ordering::Equal,
            "a ball centred on (-0, 0, 4) and the same ball centred on (0, 0, 4) sort apart"
        );
    }

    /// §32.5's trap, on a ball.
    ///
    /// Moving a ball by nothing has to give back the numbers that were stored,
    /// or `Surface::transformed` — which is how a ball is read out of a model at
    /// whatever location its face carries — moves a bit on every read. A
    /// movement's last step on each coordinate is an addition, and an addition
    /// turns a stored `-0.0` back into `0.0`, so an unfolded centre would come
    /// out of the identity as a different ball from the one that went in.
    #[test]
    fn moving_a_ball_by_the_identity_gives_back_the_bits_it_held() {
        for ball in [
            Sphere::new(Point::new(-0.0, 0.0, 4.0), 2.0).unwrap(),
            Sphere::new(Point::new(1.0, -2.5, 3.0), 0.5).unwrap(),
            Sphere::new(Point::origin(), 1.0).unwrap(),
        ] {
            assert_eq!(
                bits_of(ball.transformed(&Isometry::identity())),
                bits_of(ball)
            );
        }
    }

    #[test]
    fn a_ball_with_no_radius_is_not_a_ball() {
        assert_eq!(
            Sphere::new(Point::origin(), 0.0),
            Err(Error::RadiusNotPositive { radius: 0.0 })
        );
    }

    #[test]
    fn a_ball_with_a_negative_radius_is_refused() {
        assert_eq!(
            Sphere::new(Point::origin(), -1.0),
            Err(Error::RadiusNotPositive { radius: -1.0 })
        );
    }

    #[test]
    fn a_ball_centred_on_a_coordinate_that_is_not_a_number_is_refused() {
        assert_eq!(
            Sphere::new(Point::new(f64::NAN, 0.0, 0.0), 1.0),
            Err(Error::NotFinite {
                what: "sphere centre"
            })
        );
    }

    #[test]
    fn moving_a_ball_moves_its_centre_and_leaves_its_radius_alone() {
        let ball = Sphere::new(Point::new(1.0, 0.0, 0.0), 3.0).unwrap();
        let shift = Isometry::translation(Vector::new(0.0, 5.0, 0.0)).unwrap();
        let moved = ball.transformed(&shift);
        assert_eq!(moved.centre(), Point::new(1.0, 5.0, 0.0));
        assert_eq!(moved.radius(), 3.0);
    }

    #[test]
    fn a_point_well_outside_is_positive_and_one_well_inside_is_negative() {
        assert_eq!(
            unit_sphere().side_of(PredicatePoint::from([2.0, 0.0, 0.0])),
            Decision::Pos
        );
        assert_eq!(
            unit_sphere().side_of(PredicatePoint::from([0.25, 0.25, 0.25])),
            Decision::Neg
        );
    }

    #[test]
    fn a_point_clear_of_the_skin_is_settled_by_the_cheapest_level() {
        let (decision, level) = decide(&HowFarFromTheSkin {
            sphere: unit_sphere(),
            point: PredicatePoint::from([2.0, 0.0, 0.0]),
        });
        assert_eq!(decision, Decision::Pos);
        assert_eq!(level, Level::Float);
    }

    #[test]
    fn a_point_on_a_ball_small_enough_to_see_in_f64_is_exactly_on_it() {
        assert_eq!(
            unit_sphere().side_of(PredicatePoint::from([1.0, 0.0, 0.0])),
            Decision::Zero
        );
    }

    /// Legs and hypotenuse of a Pythagorean triple big enough that the squares
    /// run past 53 bits. Every coordinate is a whole number, so the point sits
    /// exactly on the skin — but only exact arithmetic can see it.
    #[test]
    fn a_point_on_a_ball_too_big_to_see_in_f64_still_comes_out_exactly_on_it() {
        let ball = Sphere::new(Point::origin(), 198_489_546.0).unwrap();
        let on_the_skin = PredicatePoint::from([106_308_504.0, 167_620_410.0, 0.0]);

        let (decision, level) = decide(&HowFarFromTheSkin {
            sphere: ball,
            point: on_the_skin,
        });
        assert_eq!(decision, Decision::Zero);
        assert_eq!(level, Level::Exact);
    }

    /// The three planes meet at (1/3, 1/3, 1/3), which is not an `f64`, and the
    /// ball through it has radius sqrt(1/3), which is not one either. Nothing
    /// about this can be decided, and the honest answer says so.
    #[test]
    fn a_constructed_point_sitting_on_the_skin_cannot_be_placed() {
        let meet = meet_of_three_planes(
            [1.0, 1.0, 1.0, -1.0],
            [1.0, -1.0, 0.0, 0.0],
            [0.0, 1.0, -1.0, 0.0],
        )
        .unwrap();
        let ball = Sphere::new(Point::origin(), (1.0f64 / 3.0).sqrt()).unwrap();

        match ball.side_of(meet) {
            Decision::Neg | Decision::Zero | Decision::Pos => {
                panic!("neither the box nor the radius is exact enough to place this")
            }
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
            other => panic!("unhandled decision: {other:?}"),
        }
    }

    /// A point just outside a tiny ball, and level 1 used to place it inside.
    ///
    /// With `g = 2^-538` and a radius of `3 * 2^-539 = 1.5 * g`, the point
    /// `(g, g, g)` is `sqrt(3) * g` from the centre against a radius of
    /// `1.5 * g`, and `sqrt(3) > 1.5`, so it is outside. Exactly:
    ///
    /// ```text
    /// 3*g² - (1.5*g)² = (3 - 2.25) * 2^-1076 = 0.75 * 2^-1076 = 3/16 of a u,
    /// ```
    ///
    /// where `u = 2^-1074` is the smallest number `f64` has. The truthful
    /// answer is [`Decision::Pos`].
    ///
    /// Every one of those products is a fraction of one `u`, so `f64` rounds
    /// each to a whole number of them: the three squares `0.25u` each go to
    /// `0`, and the squared radius `0.5625u` goes to `1u`. That leaves
    /// `0 - 1u`, negative where the truth is positive. The permanent is `1u`,
    /// and the bound `6.7e-16 * 1u` rounds to zero, so `1u > 0` certifies.
    #[test]
    fn a_point_whose_squares_fall_off_the_bottom_of_the_range_is_not_certified() {
        let gap = 2f64.powi(-538);
        let ball = Sphere::new(Point::origin(), 3.0 * 2f64.powi(-539)).unwrap();

        let decision = ball.side_of(PredicatePoint::from([gap, gap, gap]));
        match decision {
            Decision::Neg => panic!("this point is outside the ball, not inside it"),
            Decision::Pos | Decision::Zero => panic!("nothing here can prove a sign: {decision:?}"),
            Decision::Undecidable { enclosure } => assert!(enclosure > 0.0),
            other => panic!("unhandled decision: {other:?}"),
        }
    }

    /// The same ball and point scaled up by `2^600`, as an oracle for the test
    /// above.
    ///
    /// The centre is the origin, so scaling the radius and the point together
    /// by a positive number multiplies `|point|² - radius²` by the square of
    /// it — positive, so the sign is untouched. At `2^600` the squares are
    /// ordinary normal numbers and the answer is the [`Decision::Pos`] the
    /// test above cannot prove.
    #[test]
    fn the_same_ball_and_point_scaled_up_to_fit_put_the_point_outside() {
        let gap = 2f64.powi(-538 + 600);
        let ball = Sphere::new(Point::origin(), 3.0 * 2f64.powi(-539 + 600)).unwrap();
        assert_eq!(
            ball.side_of(PredicatePoint::from([gap, gap, gap])),
            Decision::Pos
        );
    }

    #[test]
    fn a_constructed_point_well_inside_is_still_decided() {
        let meet = meet_of_three_planes(
            [1.0, 1.0, 1.0, -1.0],
            [1.0, -1.0, 0.0, 0.0],
            [0.0, 1.0, -1.0, 0.0],
        )
        .unwrap();

        match unit_sphere().side_of(meet) {
            Decision::Neg => (),
            other => panic!("a box around (1/3, 1/3, 1/3) is inside the unit ball: {other:?}"),
        }
    }

    /// A point that is not a point is neither in the ball nor out of it.
    ///
    /// This one predicate reached the same refusal on its own before the guard
    /// existed — see the note on the `impl` — so the test pins behaviour rather
    /// than catching a regression. What it does catch is the guard answering
    /// something *different* from what the cascade used to work out.
    #[test]
    fn a_point_that_is_not_a_point_is_neither_in_nor_out() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for axis in 0..3 {
                let mut broken = [0.0, 0.0, 5.0];
                broken[axis] = bad;
                assert_eq!(
                    unit_sphere().side_of(PredicatePoint::from(broken)),
                    Decision::Undecidable {
                        enclosure: f64::INFINITY
                    },
                    "{bad} on axis {axis}"
                );
            }
        }
    }

    /// [`Sphere::transformed`] cannot fail and can overflow, so a ball built
    /// from finite numbers can be holding an infinite centre. It is then no
    /// longer a ball, and has no inside.
    #[test]
    fn a_ball_shoved_past_the_end_of_the_number_line_has_no_inside() {
        let far = 1e308;
        let shove = Isometry::translation(Vector::new(far, 0.0, 0.0)).unwrap();
        let shoved = Sphere::new(Point::new(far, 0.0, 0.0), 1.0)
            .unwrap()
            .transformed(&shove);

        assert!(!shoved.centre().is_finite(), "the centre overflowed");
        assert_eq!(
            shoved.side_of(PredicatePoint::from([0.0, 5.0, 0.0])),
            Decision::Undecidable {
                enclosure: f64::INFINITY
            }
        );
    }
}
