use core::f64::consts::{PI, TAU};

use rcad_geom::{
    Cylinder, EdgeRun, Picker, Plane, Point, Sphere, Surface, Vector,
    which_two_surfaces_a_run_is_between,
};
use rcad_predicates::expansion::{difference_of_two_products, expansion_sign};
use rcad_predicates::{Decision, Interval, Point as PredicatePoint};

use crate::view::{EdgeView, WireView};
use crate::{Error, Result};

/// How long an edge or a wire is, and how far that could be out.
///
/// **One answer and one error, both lengths in the model's own unit** — the
/// same honesty [`Measurement`](crate::Measurement) carries for a solid's
/// volume and skin, and here there is only the one unit to keep them in
/// (`docs/decisions.md` §25.4).
///
/// Every step on the way is carried as an [`Interval`] that holds the true
/// value, so the error is not a count of roundings someone added up by hand:
/// it is half the width of the interval the answer came out in. The one input
/// no interval can vouch for is `atan2`, which an arc needs, and what is
/// allowed for it is [`ROOM_FOR_ATAN2`].
///
/// What this measures is the curve the model names between the two ends it
/// stores, **with the model's own numbers taken as they are**. A corner a
/// decision left a hair off its curve is the model's slop, which
/// [`EdgeView::approximation`] carries; it is not in this error.
///
/// How big the error is, per case, as measured and pinned by
/// `tests/how_long_an_edge_is.rs`:
///
/// - **A straight edge** — the distance between its two ends. Exactly `0.0`
///   along an axis where the two ends' one difference is itself an `f64`,
///   which every edge of a cube is. Off an axis, at most `4 · f64::EPSILON` of
///   the length: each square carries three steps of an `f64` at most, the two
///   sums two more, and the root halves that and adds one. Measured at
///   `1.24 · f64::EPSILON` over right triangles.
/// - **A whole circle on a pipe** — `2π` times the radius, which is stored.
///   `2π` is held between its two neighbouring `f64`s, which is `0.64` of an
///   `f64::EPSILON` either way, relatively, and the product steps out once
///   more: at most `2 · f64::EPSILON` of the length, measured at `1.59`.
/// - **Half a circle round a ball** — its seam, radius times angle. The angle
///   is [`ROOM_FOR_ATAN2`] either way, plus `π`'s own neighbours and a step
///   for each of two sums, which is `4.5 · f64::EPSILON` of it at most, and
///   the product steps out once more: at most `6 · f64::EPSILON` of the
///   length, measured at `3.98` over radii from `1e-3` to `1e100`. The
///   angle's error is a fixed number of radians rather than a share of the
///   angle, so a shorter arc carries about the same error in length — some
///   `14 · f64::EPSILON` of the radius — and a larger one as a share.
/// - **A wire** — the sum of its edges, whose errors add, with a step of the
///   running total for each edge on top.
///
/// # Why the closed form, and what the other way costs
///
/// A circle's length is its radius times its angle: a few roundings and one
/// `atan2`. The other way is Archimedes': walk the curve, and the polygon
/// through the walked points is shorter than it while the polygon of tangents
/// at the same points is longer. That bracket is about the deflection over the
/// radius wide, relatively, and it is capped by the walk's
/// [`MOST_POINTS_ALONG_ONE_CROSSING`](rcad_geom::MOST_POINTS_ALONG_ONE_CROSSING),
/// so it buys a dozen digits where the closed form buys fifteen. It is the
/// family [`Error::CannotYetMeasureHowLongThisCurveIs`] names for every other
/// curve a plane leaves on a quadric, because every one of those is convex.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Length {
    length: f64,
    error: f64,
}

impl Length {
    /// How long it is, in the model's own unit.
    #[must_use]
    pub fn length(self) -> f64 {
        self.length
    }

    /// How far [`Length::length`] can be from the true length — a distance, in
    /// the model's own unit, and never negative.
    ///
    /// `[length − error, length + error]` holds the true length.
    #[must_use]
    pub fn error(self) -> f64 {
        self.error
    }

    /// The middle of `bounds`, with half its width as the error.
    ///
    /// # Panics
    ///
    /// If `bounds` reaches below zero, which no length this module builds can.
    fn held_by(bounds: Interval) -> Result<Self> {
        let (lo, hi) = (bounds.lo(), bounds.hi());
        assert!(
            lo >= 0.0,
            "a length came out in {bounds:?}, which reaches below zero"
        );
        if !hi.is_finite() {
            return Err(Error::LengthTooLargeToMeasure);
        }
        let length = lo / 2.0 + hi / 2.0;
        let error = rounded_up(hi - length).max(rounded_up(length - lo));
        Ok(Self { length, error })
    }

    /// The interval this length and its error stand for.
    fn bounds(self) -> Interval {
        Interval::around(self.length, self.error)
            .expect("a length and its error are finite and the error is not negative")
    }
}

/// How far `atan2` is allowed to be from the true angle, in radians.
///
/// Rust does not say how accurate `atan2` is. This is four steps of an `f64` at
/// `π`, the top of `atan2`'s range, which is the room
/// [`Surface::ROUND_TRIP_ROUNDOFF`](rcad_geom::Surface::ROUND_TRIP_ROUNDOFF)
/// leaves for a libm "a few ulps out rather than one that is perfect". It is
/// the one number in a [`Length`]'s error that is allowed rather than proved.
pub const ROOM_FOR_ATAN2: f64 = 8.0 * f64::EPSILON;

impl EdgeView<'_> {
    /// How long this edge is, and how far that could be out.
    ///
    /// Measured in the frame the edge is stored in. A movement changes no
    /// length, so a solid placed a million units away and turned gets the same
    /// answer **bit for bit** as the same solid at the origin — where reading
    /// the located ends would carry the placement's own rounding into it.
    ///
    /// Straight edges, whole circles and arcs of circles; [`Length`] says what
    /// each costs.
    ///
    /// # Errors
    ///
    /// - [`Error::CannotYetMeasureHowLongThisCurveIs`] for every curve that is
    ///   not a circle — an ellipse where a plane leans across a pipe, and every
    ///   crossing of two curved surfaces. A gap, and the variant names what
    ///   would close it.
    /// - [`Error::LengthTooLargeToMeasure`] when the length does not fit in an
    ///   `f64`.
    /// - [`Error::AnEdgeWithoutTwoFacesAlongItCannotBeWalked`] for a curved
    ///   edge with no face along it, or three or more.
    /// - [`Error::AnEdgeBendsButEveryFaceAlongItIsFlat`] for a curved edge
    ///   whose surfaces are both planes, which name no curve.
    /// - [`Error::ACrossingThisCannotBeOrderedAlong`] for a kind of run this
    ///   crate has never seen, the name [`EdgeView::is_closed`] refuses one by.
    pub fn length(self) -> Result<Length> {
        let [start, end] = self.stored_ends();
        let run = self.stored_run();
        let not_reached = Error::CannotYetMeasureHowLongThisCurveIs { edge: self.key() };
        let bounds = match run {
            EdgeRun::Straight => straight_between(start, end),
            EdgeRun::RightRoundWhereItsFacesCross | EdgeRun::RightRoundWhereItCrosses { .. } => {
                self.circle_it_runs_round(run)?
                    .radius()
                    .ok_or(not_reached)?
                    * a_whole_turn()
            }
            EdgeRun::PartOfWhereItCrosses { picked_by, .. } => self
                .circle_it_runs_round(run)?
                .arc_between(start, end, picked_by)
                .ok_or(not_reached)?,
            _ => return Err(Error::ACrossingThisCannotBeOrderedAlong),
        };
        Length::held_by(bounds)
    }

    /// The circle a curved edge runs along, named by the two surfaces it lies
    /// on.
    fn circle_it_runs_round(self, run: &EdgeRun) -> Result<Circle> {
        let [one, other] = self.stored_surfaces_either_side()?;
        match which_two_surfaces_a_run_is_between(run, [&one, &other]) {
            [Surface::Plane(_), Surface::Plane(_)] => {
                Err(Error::AnEdgeBendsButEveryFaceAlongItIsFlat { edge: self.key() })
            }
            pair => Circle::where_they_cross(pair)
                .ok_or(Error::CannotYetMeasureHowLongThisCurveIs { edge: self.key() }),
        }
    }
}

impl WireView<'_> {
    /// How long this wire is: the edges it runs along added up, their errors
    /// added up too, and the rounding of the sum on top.
    ///
    /// A pole adds nothing, because it goes nowhere. A seam the wire runs along
    /// twice, up and back down, counts twice, because the wire does.
    ///
    /// # Errors
    ///
    /// Whatever [`EdgeView::length`] refuses for an edge along it, and
    /// [`Error::LengthTooLargeToMeasure`] when the sum does not fit in an
    /// `f64`.
    pub fn length(self) -> Result<Length> {
        let mut total = Interval::exactly(0.0);
        for coedge in self.coedges() {
            total = total + coedge.edge().length()?.bounds();
        }
        Length::held_by(total)
    }
}

/// The distance between two stored points.
///
/// Scaled by a power of two first, so the widest of the three differences is
/// near one: its square then neither overflows nor falls into the subnormals,
/// and the scaling rounds nothing.
fn straight_between(start: Point, end: Point) -> Interval {
    let [from, to] = [start, end].map(|point| point.coordinates().map(Interval::exactly));
    let apart = [0, 1, 2].map(|axis| to[axis] - from[axis]);
    let standing_still = |difference: &Interval| difference.lo() == 0.0 && difference.hi() == 0.0;
    if apart
        .iter()
        .filter(|difference| standing_still(difference))
        .count()
        >= 2
    {
        // Along one axis the length is that one difference, and squaring it to
        // take the root back would round twice for nothing.
        return magnitude(apart[0]) + magnitude(apart[1]) + magnitude(apart[2]);
    }
    let widest = apart
        .iter()
        .map(|difference| difference.lo().abs().max(difference.hi().abs()))
        .fold(0.0, f64::max);
    let power = the_power_of_two_below(widest);
    let down = Interval::exactly(two_to_the(-power));
    let scaled = apart.map(|difference| difference * down);
    square_root(dot(scaled, scaled)) * Interval::exactly(two_to_the(power))
}

/// The power of two at or below `magnitude`, kept where it and its reciprocal
/// are both ordinary `f64`s.
///
/// Read off the exponent bits, so nothing rounds. A zero or a subnormal comes
/// back as the bottom of that range, which scales it up as far as that goes;
/// an infinity comes back as the top, which leaves it infinite and so refused.
fn the_power_of_two_below(magnitude: f64) -> i32 {
    let biased = i32::try_from((magnitude.to_bits() >> 52) & 0x7ff)
        .expect("an f64's exponent is eleven bits");
    (biased - 1023).clamp(-1022, 1022)
}

/// Two to the `power`, exactly, for a `power` an ordinary `f64` holds.
fn two_to_the(power: i32) -> f64 {
    let biased = u64::try_from(power + 1023).expect("a power an ordinary f64 holds");
    f64::from_bits(biased << 52)
}

/// The square root of a sum of squares: each end a step of an `f64` outwards
/// where `sqrt` rounded, and no step where it did not ([`Interval::sqrt`]).
///
/// # Panics
///
/// If every number in `of` is below zero. The top of an interval times itself
/// is at least the square of one of its ends, so it is never below zero, and
/// neither is the top of a sum of such squares.
fn square_root(of: Interval) -> Interval {
    of.sqrt()
        .expect("the top of a sum of squares is not below zero")
}

/// `numerator / denominator` for every pair of numbers the two hold, or `None`
/// when the denominator holds zero.
///
/// Each quotient is stepped outwards, except a numerator of exactly zero, whose
/// quotient is exactly zero.
fn divided(numerator: Interval, denominator: Interval) -> Option<Interval> {
    if denominator.contains_zero() {
        return None;
    }
    let quotient = |top: f64, bottom: f64, outwards: fn(f64) -> f64| {
        if top == 0.0 {
            0.0
        } else {
            outwards(top / bottom)
        }
    };
    let ends = [numerator.lo(), numerator.hi()]
        .map(|top| [denominator.lo(), denominator.hi()].map(|bottom| (top, bottom)));
    let ends = ends.as_flattened();
    let lo = ends
        .iter()
        .map(|&(top, bottom)| quotient(top, bottom, f64::next_down))
        .fold(f64::INFINITY, f64::min);
    let hi = ends
        .iter()
        .map(|&(top, bottom)| quotient(top, bottom, f64::next_up))
        .fold(f64::NEG_INFINITY, f64::max);
    Interval::new(lo, hi).ok()
}

/// How far from zero every number in `of` is.
fn magnitude(of: Interval) -> Interval {
    if of.lo() >= 0.0 {
        of
    } else if of.hi() <= 0.0 {
        -of
    } else {
        Interval::new(0.0, (-of.lo()).max(of.hi())).expect("a range from zero up")
    }
}

fn dot(one: [Interval; 3], other: [Interval; 3]) -> Interval {
    one[0] * other[0] + one[1] * other[1] + one[2] * other[2]
}

fn cross(one: [Interval; 3], other: [Interval; 3]) -> [Interval; 3] {
    [
        one[1] * other[2] - one[2] * other[1],
        one[2] * other[0] - one[0] * other[2],
        one[0] * other[1] - one[1] * other[0],
    ]
}

/// `2π`, between the two `f64`s either side of it.
fn a_whole_turn() -> Interval {
    Interval::new(TAU.next_down(), TAU.next_up()).expect("two neighbouring numbers")
}

/// One step up from `value`, unless `value` is exactly zero.
///
/// A difference of two `f64`s is zero exactly when the two are equal, so a zero
/// here was never rounded and needs no step.
fn rounded_up(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value.next_up() }
}

/// Whether `normal` and `along` point along one line, **exactly**.
///
/// All three parts of their cross product have to be zero, and each is a
/// difference of two products that is decided exactly rather than rounded: a
/// normal and an axis that are parallel but written at different lengths can
/// round to a cross product that is not zero, and two that are not parallel
/// can round to one that is. A product too small or too large to be held
/// exactly answers `false`, which is a refusal — the circle it might have been
/// is not measured, and nothing is guessed.
fn is_square_to(normal: Vector, along: Vector) -> bool {
    let [nx, ny, nz] = normal.components();
    let [ax, ay, az] = along.components();
    [(ny, az, nz, ay), (nz, ax, nx, az), (nx, ay, ny, ax)]
        .into_iter()
        .all(|(a, b, c, d)| {
            difference_of_two_products(a, b, c, d).is_some_and(|(entries, count)| {
                matches!(expansion_sign(&entries[..count]), Decision::Zero)
            })
        })
}

/// A circle where a plane crosses a pipe square to its axis, or crosses a ball.
///
/// Held as the two surfaces rather than as a centre and a radius, because
/// those are the numbers the model stores, and every other number is worked out
/// from them inside an interval.
enum Circle {
    OnAPipe { plane: Plane, pipe: Cylinder },
    OnABall { plane: Plane, ball: Sphere },
}

impl Circle {
    /// The circle two surfaces cross in, if they are a pair this reaches.
    ///
    /// A plane crosses a pipe in a circle exactly when its normal is parallel
    /// to the pipe's axis; leaning, it is an ellipse. A plane crosses a ball in
    /// a circle or not at all.
    fn where_they_cross(pair: [Surface; 2]) -> Option<Self> {
        match pair {
            [Surface::Plane(plane), Surface::Cylinder(pipe)]
            | [Surface::Cylinder(pipe), Surface::Plane(plane)] => {
                is_square_to(plane.normal(), pipe.axis().direction())
                    .then_some(Circle::OnAPipe { plane, pipe })
            }
            [Surface::Plane(plane), Surface::Sphere(ball)]
            | [Surface::Sphere(ball), Surface::Plane(plane)] => {
                Some(Circle::OnABall { plane, ball })
            }
            // A plane square to a cone's axis cuts a circle too, and a plane
            // square to a torus's two; neither is reached here yet.
            [Surface::Cone(_) | Surface::Torus(_), _]
            | [_, Surface::Cone(_) | Surface::Torus(_)] => None,
            [
                Surface::Plane(_) | Surface::Sphere(_) | Surface::Cylinder(_),
                Surface::Plane(_) | Surface::Sphere(_) | Surface::Cylinder(_),
            ] => None,
            _ => None,
        }
    }

    fn plane(&self) -> Plane {
        match self {
            Circle::OnAPipe { plane, .. } | Circle::OnABall { plane, .. } => *plane,
        }
    }

    /// The radius. A pipe's is stored; round a ball it is `√((R − d)(R + d))`,
    /// with `d` how far the ball's centre is from the plane.
    ///
    /// A plane that misses the ball leaves nothing above zero to take the root
    /// of, and zero is what comes back — an edge along a circle that is not
    /// there is `Model::validate`'s to refuse, and a length of zero is not a
    /// wrong answer for it.
    fn radius(&self) -> Option<Interval> {
        match self {
            Circle::OnAPipe { pipe, .. } => Some(Interval::exactly(pipe.radius())),
            Circle::OnABall { plane, ball } => {
                let normal = plane.normal().components().map(Interval::exactly);
                let apart = magnitude(divided(
                    how_far_in_front(plane, ball.centre()),
                    square_root(dot(normal, normal)),
                )?);
                let radius = Interval::exactly(ball.radius());
                if apart.hi() == 0.0 {
                    // Through the centre: a great circle, of the ball's own
                    // radius, where squaring it to take the root back would
                    // round for nothing.
                    return Some(radius);
                }
                let square = (radius - apart) * (radius + apart);
                Some(square.sqrt().unwrap_or(Interval::exactly(0.0)))
            }
        }
    }

    /// A box round the centre, or `None` where a division could not be
    /// bounded.
    fn centre(&self) -> Option<[Interval; 3]> {
        match self {
            Circle::OnAPipe { plane, pipe } => {
                let axis = pipe.axis();
                let normal = plane.normal().components().map(Interval::exactly);
                let along = axis.direction().components().map(Interval::exactly);
                let up = divided(-how_far_in_front(plane, axis.origin()), dot(normal, along))?;
                let origin = axis.origin().coordinates().map(Interval::exactly);
                Some([0, 1, 2].map(|at| origin[at] + along[at] * up))
            }
            Circle::OnABall { plane, ball } => {
                let normal = plane.normal().components().map(Interval::exactly);
                let back = divided(how_far_in_front(plane, ball.centre()), dot(normal, normal))?;
                let centre = ball.centre().coordinates().map(Interval::exactly);
                Some([0, 1, 2].map(|at| centre[at] - normal[at] * back))
            }
        }
    }

    /// How long the arc from `start` to `end` is that every one of `pickers`
    /// holds, or `None` where that cannot be told.
    ///
    /// Two arcs share the two ends: the one turning forwards round the plane's
    /// normal, through `θ`, and the other through `2π − θ`. One point half way
    /// along each is put to the pickers, and the arc is the one they all hold
    /// while some picker rules the other out. Anything else — both held,
    /// neither, a picker that cannot tell, an arc too short for its middle to
    /// be told from its ends — is a refusal and never a guess.
    fn arc_between(&self, start: Point, end: Point, pickers: &[Picker]) -> Option<Interval> {
        let centre = self.centre()?;
        let radius = self.radius()?;
        let normal = self.plane().normal();
        let [from, to] = [start, end].map(|point| {
            let at = point.coordinates().map(Interval::exactly);
            [0, 1, 2].map(|axis| at[axis] - centre[axis])
        });
        let across = normal.components().map(Interval::exactly);
        let sine = divided(
            dot(across, cross(from, to)),
            square_root(dot(across, across)),
        )?;
        let forwards = angle_round(dot(from, to), sine)?;
        let backwards = a_whole_turn() - forwards;

        let middle = |of: Interval| of.lo() / 2.0 + of.hi() / 2.0;
        let middle_of = |of: [Interval; 3]| of.map(middle);
        let halfway = HalfWay {
            centre: Point::new(middle(centre[0]), middle(centre[1]), middle(centre[2])),
            start: {
                let [x, y, z] = middle_of(from);
                Vector::new(x, y, z)
            },
            normal,
            reach: how_far_a_point_half_way_can_be_off(&centre, &from),
        };
        let slack = forwards.width() + 2.0 * halfway.reach / radius.lo();
        if !(forwards.lo() > slack && backwards.lo() > slack) {
            return None;
        }
        let turn = middle(forwards) / 2.0;
        match [turn, turn + PI].map(|angle| halfway.at(angle).map(|point| verdict(pickers, point)))
        {
            [Some(Verdict::Holds), Some(Verdict::RuledOut)] => Some(radius * forwards),
            [Some(Verdict::RuledOut), Some(Verdict::Holds)] => Some(radius * backwards),
            _ => None,
        }
    }
}

/// `normal · point − offset`, with the plane's own stored numbers: how far in
/// front of `plane` a point is, in steps of the stored normal's length.
fn how_far_in_front(plane: &Plane, point: Point) -> Interval {
    let normal = plane.normal().components().map(Interval::exactly);
    dot(normal, point.coordinates().map(Interval::exactly)) - Interval::exactly(plane.offset())
}

/// The angle from a direction to another, turning forwards, in `[0, 2π)`, out
/// of a box round its cosine and sine — each times the same positive number.
///
/// Over a box that does not hold the origin the angle is least and greatest at
/// two of its corners, so four `atan2`s bound it, each widened by
/// [`ROOM_FOR_ATAN2`]. Which branch they are read on is chosen so the box does
/// not straddle it: above the line, below it, or to its left, where a ball's
/// seam is. A box straddling the line to the right of the origin is an angle
/// that could be a sliver or nearly a whole turn, and that is `None`.
fn angle_round(cosine: Interval, sine: Interval) -> Option<Interval> {
    let corners = [cosine.lo(), cosine.hi()].map(|x| [sine.lo(), sine.hi()].map(|y| (x, y)));
    let corners = corners.as_flattened();
    let spread = |angle: fn((f64, f64)) -> f64| {
        let angles = corners.iter().copied().map(angle);
        let lo = angles.clone().fold(f64::INFINITY, f64::min);
        let hi = angles.fold(f64::NEG_INFINITY, f64::max);
        Interval::new(lo, hi).expect("the least of four numbers is below the greatest")
    };
    let (read, shift) = if sine.lo() > 0.0 {
        (spread(|(x, y)| y.atan2(x)), Interval::exactly(0.0))
    } else if sine.hi() < 0.0 {
        (spread(|(x, y)| y.atan2(x)), a_whole_turn())
    } else if cosine.hi() < 0.0 {
        (
            spread(|(x, y)| (-y).atan2(-x)),
            Interval::new(PI.next_down(), PI.next_up()).expect("two neighbouring numbers"),
        )
    } else {
        return None;
    };
    let room = Interval::new(-ROOM_FOR_ATAN2, ROOM_FOR_ATAN2).expect("a range round zero");
    Some(read + room + shift)
}

/// Where to find a point half way along one of the two arcs.
struct HalfWay {
    centre: Point,
    start: Vector,
    normal: Vector,
    reach: f64,
}

impl HalfWay {
    /// A box round the point `angle` forwards from the start, round the
    /// normal, that holds a point of the true circle.
    fn at(&self, angle: f64) -> Option<PredicatePoint> {
        let normal = self.normal.normalized().ok()?;
        let offset = self.start * angle.cos() + normal.cross(self.start) * angle.sin();
        let point = self.centre + offset;
        let [x, y, z] = point
            .coordinates()
            .map(|coordinate| Interval::around(coordinate, self.reach).ok());
        Some(PredicatePoint::Constructed([x?, y?, z?]))
    }
}

/// How far a point worked out half way round, in plain `f64`, can be from the
/// point of the true circle in the same direction.
///
/// The centre can be anywhere in its box, and the start's direction from it
/// anywhere in its own; each moves the point by about its own width, and
/// turning the start's direction round the circle carries that along at most
/// doubled. `cos`, `sin`, the cross product and two sums round on top, and are
/// given `64 · f64::EPSILON` of the radius and the point's own size, which is
/// room for a libm a few ulps out. Four times all of that, because this is
/// only ever the size of a box a picker is asked about: too wide is a refusal,
/// and too narrow would be the one wrong answer this could give.
fn how_far_a_point_half_way_can_be_off(centre: &[Interval; 3], from: &[Interval; 3]) -> f64 {
    let widest = |of: &[Interval; 3]| of.iter().map(|axis| axis.width()).fold(0.0, f64::max);
    let size = |of: &[Interval; 3]| {
        of.iter()
            .map(|axis| axis.lo().abs().max(axis.hi().abs()))
            .fold(0.0, f64::max)
    };
    let root_three = 3.0_f64.sqrt();
    let drift = root_three * (widest(centre) + 3.0 * widest(from));
    let rounding = 64.0 * f64::EPSILON * root_three * (size(centre) + 2.0 * size(from));
    4.0 * (drift + rounding)
}

/// What the pickers make of one point.
enum Verdict {
    /// Every picker holds it.
    Holds,
    /// Some picker rules it out.
    RuledOut,
    /// No picker rules it out, and some picker cannot tell.
    CannotTell,
}

fn verdict(pickers: &[Picker], point: PredicatePoint) -> Verdict {
    let mut verdict = Verdict::Holds;
    for picker in pickers {
        match picker.side_of_point(point) {
            Decision::Pos | Decision::Zero => {}
            Decision::Neg => return Verdict::RuledOut,
            Decision::Undecidable { .. } | _ => verdict = Verdict::CannotTell,
        }
    }
    verdict
}

#[cfg(test)]
mod tests {
    use rcad_predicates::Interval;

    use super::square_root;

    /// `sqrt` rounds `√2` to the `f64` above it, so the range that holds the
    /// root runs from the `f64` below that one up to it, and no further.
    #[test]
    fn the_root_of_two_is_one_step_of_an_f64_wide() {
        let nearest = 2.0_f64.sqrt();
        assert!(
            nearest.mul_add(nearest, -2.0) > 0.0,
            "the nearest f64 is above √2"
        );
        let root = square_root(Interval::exactly(2.0));
        assert_eq!((root.lo(), root.hi()), (nearest.next_down(), nearest));
    }
}
