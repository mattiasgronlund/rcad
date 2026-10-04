//! Where three surfaces meet.
//!
//! [`intersect`](crate::intersect) names the curve two surfaces cross in; this
//! names the **points** a third surface cuts out of that curve, which is what a
//! corner of an arrangement is (`docs/decisions.md` §31.2 rows 4 and 5).
//!
//! # Two planes and a quadric is the road, and two of the roads back to it
//!
//! Two planes cross in a line, and a line cut by a quadric is a quadratic in one
//! unknown whose discriminant is a polynomial in the stored numbers. That is the
//! one route that is worked out here, and everything else either arrives at it
//! or is refused.
//!
//! One plane and two quadrics is a quartic in general — the plane crosses each
//! quadric in a conic and two conics in one plane meet where a quartic is zero —
//! and reaching *that* takes an algebraic solver rather than a discriminant. But
//! **two families of it are not a quartic at all**, and both are ordinary
//! mechanical CAD rather than curiosities:
//!
//! - **Two quadrics that bend the same way.** Write a quadric out as
//!   `p·(M p) + 2b·p + c`. Two of them with the same `M` differ by something
//!   *linear*, so every point both of them hold sits on one flat surface — the
//!   plane `2(b₁ − b₂)·p = c₂ − c₁`, whose every number this file works out
//!   exactly. Two balls have it, two pipes running the same way have it, and two
//!   cones of one slope about one direction have it. So the triple is two planes
//!   and a quadric after all, and the road above answers it.
//!   [`where_a_plane_and_two_quadrics_of_one_bend_meet`].
//! - **A plane that only grazes one of them.** A plane that misses a quadric
//!   leaves the three sharing nothing, and a plane that *touches* one at a single
//!   point leaves one candidate — so the whole question is which side of the
//!   third surface that point is on. [`what_a_plane_that_grazes_one_of_them_leaves`].
//!
//! What is left over is refused by name (§16.10, §31.2): one plane and two
//! quadrics that bend differently and cross the plane in curves — a pipe and a
//! ball, or two pipes leaning against each other — and any triple with a torus
//! and fewer than two planes.
//! [`Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat`] says which of the two
//! kinds of refusal it is in its own doc.
//!
//! # The answer depends on which three surfaces they are, and not on the order
//!
//! Three surfaces can meet at more than one point, so the three names do not
//! pick a corner out on their own and a fuse has to count along the line
//! (§31.3). That count is only well defined if the line has a direction the
//! three surfaces fix between them — so the three are put in a **fixed order**
//! before anything is worked out, and the points come back running along
//! `n₁ × n₂` for the two planes *in that order*. `fuse(A, B)` and `fuse(B, A)`
//! therefore number the same corner the same way.
//!
//! # A sign and a size are two different numbers here
//!
//! [`Exact::square_minus`] hands back a sign and never a value, because it
//! multiplies both sides by a positive power of two (§32.1). It is what settles
//! which way the discriminant points where nothing else can. **How big the
//! discriminant is comes from somewhere else** — the same difference summed
//! without the slide — and mixing the two up would put a corner wherever the
//! slide happened to land.

use rcad_predicates::{
    Decision, Interval, Level, Measures, NOTHING_IS_KNOWN, Point as PredicatePoint, Policy,
    ProvenanceLog, Record, SideOfSurface,
};
use smallvec::{SmallVec, smallvec};

use crate::enclosure::{
    cross_enclosing, dot_enclosing, exactly, quotient_or_everything, root_enclosing,
};
use crate::exact::{Exact, Nearly, binary_exponent, multiplied_by_a_power_of_two, slid_exactly};
use crate::intersect::{Question, ask, intersect, squared_length_of_a_cross_product};
use crate::plane::which_side_of_a_flat_surface;
use crate::real_roots::where_a_polynomial_is_zero;
use crate::{
    Cone, Cylinder, Error, Plane, PlaneAgreement, Point, Result, Sphere, Surface, SurfaceMeet,
    Torus,
};

/// The name the three-surface meet goes by in provenance records.
pub const WHERE_THREE_SURFACES_MEET: &str = "where three surfaces meet";

/// The name a line-meets-one-surface question goes by in provenance records.
pub const WHERE_A_LINE_MEETS_A_SURFACE: &str = "where a line meets a surface";

/// What three surfaces have in common.
///
/// [`MeetOfThree::At`] is the corner case a fuse is after. The other two arms
/// are answers about the geometry and not failures: three surfaces that share a
/// whole curve have no corner to name, and three that share nothing have none
/// either.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum MeetOfThree {
    /// They have no point in common at all.
    Nowhere,
    /// They meet at these points — one, two, three or four — running along the
    /// line the two planes cross in.
    ///
    /// Three and four take a torus: a straight line meets a quadric twice and a
    /// ring four times, which is the whole of what makes a torus not a quadric.
    ///
    /// Most are a box the true meeting point is somewhere inside, because
    /// solving a quadratic divides and a quotient of two ranges almost never
    /// lands on an `f64`. That width is the point of it (§2.5).
    ///
    /// **A point whose every coordinate is an `f64` is that point**, handed back
    /// as [`PredicatePoint::Exact`], wherever the numbers are known exactly and
    /// each quotient is checked to land on one. A line that touches a ball
    /// touches it at `−B/A` along itself, and at a corner such as `(0, 0, 16)` a
    /// box of any width stops every question whose true answer is zero there —
    /// which way a wall turns against the ball it touches is one of them. Such
    /// a point is not a rounding of the box: it is the one number the box holds.
    At(SmallVec<[PredicatePoint; 2]>),
    /// They share a whole curve or a whole surface, so there is no corner here.
    ///
    /// Two faces on one plane with a third surface across them, or a plane pair
    /// whose line runs down a pipe's own wall.
    MoreThanAPoint,
}

impl MeetOfThree {
    /// This meet, which [`meet_of_three_surfaces`] handed back for `surfaces`,
    /// with the box round a meet of **three planes, or of two planes and a
    /// ball, a pipe or a cone,** narrowed by one checked step. Any other meet
    /// comes back as it is.
    ///
    /// Cramer's rule sums its numerators in intervals, and each numerator is a
    /// small number built out of products near one that all round. Where two of
    /// the three planes are nearly parallel the determinant is small too, so the
    /// box is those roundings divided by a small number: `±1.43e-10` round a meet
    /// that is `1e-33` from `(5, 0, 0)`, on a draw where two walls lean
    /// `1.5e-6` off parallel (§281.1). The width says how badly the rounding
    /// could have gone and nothing about where in the box the meet is.
    ///
    /// **The step.** Take the middle of that box as a guess. For each plane work
    /// out how far the guess misses it, `offset − normal · guess`, from the
    /// stored numbers with nothing rounded. Then the meet is the guess plus
    /// `(r₁ (n₂ × n₃) + r₂ (n₃ × n₁) + r₃ (n₁ × n₂)) / det`, with the cross
    /// products and `det` worked out the same way and read as enclosures. Every
    /// term of that numerator carries a factor of how far the guess misses, so
    /// what rounds there is a rounding of something small. The answer is where
    /// the stepped box and Cramer's box overlap. Both hold the meet, so the
    /// overlap is sound: it is a proof and not a threshold (§17), and on that
    /// draw it is `8.9e-16` wide (§281.2).
    ///
    /// **Two planes and a quadric take the same step**, with the quadric in the
    /// third plane's row ([`narrowed_onto_a_quadric_by_one_checked_step`]).
    /// Their meet is a root of a quadratic along the line the planes cross in,
    /// and the box is that root's box carried into space through a division
    /// and a product, each rounding outward. A block turned by `−π/4` about `x`
    /// meets a round bar's cap circle `2.5e-15` round from its rim start
    /// `(5, 0, 0)` in a box `±3` ulps wide along `x`, and round the circle
    /// from `(0, −5, 0)` the two cannot be put in order from it. Narrowed, the
    /// box is `[5 − 1 ulp, 5]` along `x`, which orders them (§284.3, Q209).
    ///
    /// **Why this is not what [`meet_of_three_surfaces`] hands back to everyone.**
    /// A narrower box round a meet is not a narrower box round a stored corner:
    /// `rcad-topo`'s `bake.rs` derives a face's corners through
    /// [`meet_of_three_surfaces`] and holds its stored corners to that box, and a
    /// stored corner sits ulps off the planes it is on. Cramer's slack covers
    /// that; the narrowed box does not. Narrowed for every caller, nine tests
    /// failed and the draw above refused on another face (§281.3). The one
    /// caller is `rcad-topo`'s arrangement of a face. It asks the narrowed box
    /// whether every point of the meet lies within a corner's rounding, before
    /// it folds the meet into the corner (§345), and, for a meet with a curved
    /// surface in it, which of two points comes first along a crossing where
    /// the meet's own box cannot say (Q209). The node it places keeps the box
    /// [`meet_of_three_surfaces`] handed back.
    ///
    /// The three planes are narrowed in [`meet_of_three_surfaces`]'s own fixed
    /// order, so the box that comes back depends on which three they are and
    /// not on the order they were asked in (§31.3).
    ///
    /// # Panics
    ///
    /// Where `self` was not worked out from `surfaces` and the stepped box
    /// misses it altogether: the two are then boxes round two different
    /// points, and a corner placed from where they overlap would be nowhere.
    #[must_use]
    pub fn boxed_tight(&self, surfaces: [&Surface; 3]) -> MeetOfThree {
        let [Surface::Plane(one), Surface::Plane(other), third] = in_a_fixed_order(surfaces) else {
            return self.clone();
        };
        let MeetOfThree::At(places) = self else {
            return self.clone();
        };
        if let Surface::Plane(last) = third {
            return MeetOfThree::At(
                places
                    .iter()
                    .map(|&place| narrowed_by_one_checked_step([&one, &other, &last], place))
                    .collect(),
            );
        }
        let Some(quadric) = how_a_surface_is_written(&third) else {
            return self.clone();
        };
        MeetOfThree::At(
            places
                .iter()
                .map(|&place| {
                    narrowed_onto_a_quadric_by_one_checked_step([&one, &other], &quadric, place)
                })
                .collect(),
        )
    }
}

/// Where three surfaces meet.
///
/// # Errors
///
/// [`Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat`] when fewer than two of
/// the three are planes and neither road back to two of them runs — the two the
/// module doc names.
///
/// [`Error::Undecidable`] when a sign the answer depends on could not be
/// settled, or when a product the exact arithmetic needs falls out of `f64`'s
/// range, both where the three stand and slid into the middle of the range.
/// Both carry the records of where they stand.
///
/// [`Error::Predicate`] wrapping
/// [`PlanesDoNotMeetInAPoint`](rcad_predicates::Error::PlanesDoNotMeetInAPoint)
/// when three planes do meet at one point and neither route below can put a box
/// round it.
///
/// # Three surfaces written far down the range are slid up it first
///
/// Three surfaces about `1e-159` across multiply out to terms near `1e-318`,
/// where every product loses bits, and what is kept of them is too loose to
/// place a corner by (`Nearly::is_within_a_rounding`, `docs/decisions.md`
/// §322.4). The same three multiplied by a power of two are the same geometry at
/// another size, and multiplying by a power of two is exact while every number
/// stays normal. So a refusal where they stand is asked again of the triple slid
/// until its lengths sit either side of one, and a corner found there is slid
/// back. §349 says what the slide does to each kind of surface and where it
/// cannot go.
pub fn meet_of_three_surfaces(
    surfaces: [&Surface; 3],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    let mut records = ProvenanceLog::new();
    let refused = match the_meet_where_they_stand(surfaces, policy, &mut records) {
        Err(refused @ Error::Undecidable { .. }) => refused,
        answered => {
            records.drain_into(log);
            return answered;
        }
    };
    if let Some(answer) = the_meet_slid_into_range(surfaces) {
        return Ok(answer);
    }
    records.drain_into(log);
    Err(refused)
}

/// Where three surfaces meet, worked out at the size they are written.
fn the_meet_where_they_stand(
    surfaces: [&Surface; 3],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    let [first, second, third] = in_a_fixed_order(surfaces);
    let (Surface::Plane(one), Surface::Plane(other)) = (&first, &second) else {
        return where_one_plane_and_two_curved_surfaces_meet(&first, &second, &third, policy, log);
    };
    if let Surface::Plane(last) = &third {
        return where_three_planes_meet(one, other, last, policy, log);
    }
    let at = the_places_these_three_sit(&first, &second, &third);
    let Some(line) = the_line_two_planes_cross_in(one, other, &at, policy, log)? else {
        return what_a_third_surface_leaves_of_one_plane(one, other, &third, &at, policy, log);
    };
    // **A torus is the only surface here that a line does not meet twice**, and
    // it takes its own route rather than a quadratic's. A torus sorts after
    // every other kind, and two of the three must be planes to get this far, so
    // a torus anywhere in the triple is `third` — which is what makes this one
    // check cover every route in.
    let found = if let Surface::Torus(ring) = &third {
        where_a_line_meets_a_ring(WHERE_THREE_SURFACES_MEET, &line, ring, &at)?
    } else {
        let quadratic = how_a_quadric_cuts_the_line(&line, &third)
            .ok_or_else(|| an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, &at))?;
        where_a_line_meets_a_quadric(WHERE_THREE_SURFACES_MEET, &quadratic, &at, policy, log)?
    };
    the_points_those_steps_name(
        WHERE_THREE_SURFACES_MEET,
        &line,
        found,
        &at,
        policy,
        log,
        || an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, &at),
    )
}

/// The meet of the same three surfaces slid along the range by a power of two,
/// slid back — or `None` where no slide moves anything, the slide is not
/// exact, or the slid triple refuses too.
///
/// **The slide centres the binades the lengths span on one**, rather than
/// bringing the largest to one. What runs out is a product of two small
/// lengths falling off the bottom or of two large ones running off the top,
/// and centring leaves the two ends the same room. For a triple of one size
/// the two choices are the same slide. For one whose places sit `2¹²` out and
/// whose ball is `2⁻⁶⁰⁰` across they are not: the largest brought to one leaves
/// the radius squared at `2⁻¹²²⁴`, and centred it is `2⁻⁶¹²`. Measured on the
/// `somewhere else` row of
/// `every_point_where_three_surfaces_meet_is_on_all_three_of_them.rs`: 180 of
/// 2051 triples refused with no slide, 52 with the largest brought to one, and
/// none centred — every one of the 52 a place moved out by `2^shift`.
///
/// **Asked under a certified policy whatever the caller's was**, because a
/// tolerant budget is a length at the size the caller wrote, and at another
/// size it would be a different budget. Refusing is the conservative half of
/// that: a tolerant policy only accepts more, so what this refuses is an answer
/// not given, never one given wrongly. And a certified answer logs nothing,
/// which is the truth about it.
fn the_meet_slid_into_range(surfaces: [&Surface; 3]) -> Option<MeetOfThree> {
    let binades: SmallVec<[i32; 15]> = surfaces
        .iter()
        .flat_map(|surface| the_lengths_of(surface))
        .filter(|length| *length != 0.0)
        .map(binary_exponent)
        .collect();
    let (Some(lowest), Some(highest)) = (binades.iter().min(), binades.iter().max()) else {
        return None;
    };
    let up = -(lowest + highest) / 2;
    if up == 0 {
        return None;
    }
    let [first, second, third] = surfaces.map(|surface| the_same_surface_slid_by(surface, up));
    let (first, second, third) = (first?, second?, third?);
    match the_meet_where_they_stand(
        [&first, &second, &third],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .ok()?
    {
        MeetOfThree::At(points) => {
            let back: SmallVec<[PredicatePoint; 2]> = points
                .iter()
                .map(|point| the_point_slid_back(*point, -up))
                .collect();
            back.iter()
                .all(|point| point.uncertainty().is_finite())
                .then_some(MeetOfThree::At(back))
        }
        MeetOfThree::Nowhere => Some(MeetOfThree::Nowhere),
        MeetOfThree::MoreThanAPoint => Some(MeetOfThree::MoreThanAPoint),
    }
}

/// The numbers a surface stores that are lengths, and so move when space is
/// scaled. A direction, a normal and a cone's slope are not among them: the
/// first two say which way and the third is one length over another.
fn the_lengths_of(surface: &Surface) -> SmallVec<[f64; 5]> {
    match surface {
        Surface::Plane(flat) => smallvec![flat.offset()],
        Surface::Sphere(ball) => {
            let [x, y, z] = ball.centre().coordinates();
            smallvec![x, y, z, ball.radius()]
        }
        Surface::Cylinder(pipe) => {
            let [x, y, z] = pipe.axis().origin().coordinates();
            smallvec![x, y, z, pipe.radius()]
        }
        Surface::Cone(cone) => SmallVec::from_slice(&cone.apex().coordinates()),
        Surface::Torus(ring) => {
            let [x, y, z] = ring.centre().coordinates();
            smallvec![x, y, z, ring.ring_radius(), ring.tube_radius()]
        }
    }
}

/// The same surface with space scaled by `2^exponent`, or `None` where one of
/// its lengths would not stay a normal number and so would not be exactly that
/// length moved.
///
/// Every length is multiplied and nothing else is touched. A plane keeps its
/// normal and an axis its direction through `moved_to`, which keeps the stored
/// bits: going back through [`Axis::new`](crate::Axis::new) would scale a unit
/// direction to length one a second time and can move its last bit, which
/// would be another surface.
///
/// # Panics
///
/// If a constructor refuses what an exact slide of a surface it built hands
/// back. Every length that comes through is finite, a radius above zero is
/// still above zero, and a tube thinner than its ring is still thinner.
fn the_same_surface_slid_by(surface: &Surface, exponent: i32) -> Option<Surface> {
    const BUILT: &str = "an exact slide of a surface is a surface";
    let slid = |value: f64| slid_exactly(value, exponent);
    let slid_point = |place: Point| -> Option<Point> {
        let [x, y, z] = place.coordinates();
        Some(Point::new(slid(x)?, slid(y)?, slid(z)?))
    };
    let moved = match surface {
        Surface::Plane(flat) => Surface::Plane(flat.moved_to(slid(flat.offset())?).expect(BUILT)),
        Surface::Sphere(ball) => Surface::Sphere(
            Sphere::new(slid_point(ball.centre())?, slid(ball.radius())?).expect(BUILT),
        ),
        Surface::Cylinder(pipe) => Surface::Cylinder(
            Cylinder::new(
                pipe.axis()
                    .moved_to(slid_point(pipe.axis().origin())?)
                    .expect(BUILT),
                slid(pipe.radius())?,
            )
            .expect(BUILT),
        ),
        Surface::Cone(cone) => Surface::Cone(
            Cone::new(
                cone.axis().moved_to(slid_point(cone.apex())?).expect(BUILT),
                cone.slope(),
            )
            .expect(BUILT),
        ),
        Surface::Torus(ring) => Surface::Torus(
            Torus::new(
                ring.axis()
                    .moved_to(slid_point(ring.centre())?)
                    .expect(BUILT),
                slid(ring.ring_radius())?,
                slid(ring.tube_radius())?,
            )
            .expect(BUILT),
        ),
    };
    Some(moved)
}

/// A corner found in the slid triple, slid back by `2^exponent`.
///
/// An end that comes back unchanged when it is slid up again was moved
/// exactly. One that does not landed among the subnormals and was rounded to
/// the nearest of them, less than one subnormal off, so it is stepped one
/// subnormal further out and the box still holds the corner. One that runs off
/// the top is infinite, and the caller reads that as the refusal it is.
fn the_point_slid_back(point: PredicatePoint, exponent: i32) -> PredicatePoint {
    let exactly = |value: f64| {
        let moved = multiplied_by_a_power_of_two(value, exponent);
        (multiplied_by_a_power_of_two(moved, -exponent) == value).then_some(moved)
    };
    if let PredicatePoint::Exact(place) = point
        && let [Some(x), Some(y), Some(z)] = place.map(exactly)
    {
        return PredicatePoint::Exact([x, y, z]);
    }
    let outward = |end: f64, step: fn(f64) -> f64| {
        exactly(end).unwrap_or_else(|| step(multiplied_by_a_power_of_two(end, exponent)))
    };
    PredicatePoint::Constructed(point.enclosure().map(|side| {
        Interval::new(
            outward(side.lo(), f64::next_down),
            outward(side.hi(), f64::next_up),
        )
        .expect("both ends moved the same way, and outward where they rounded")
    }))
}

/// The steps a line meets a surface at, turned into the points they name.
///
/// **The one place a step becomes a point**, so the two questions have one
/// answer between them and not two: [`where_a_line_meets_a_surface`] hands back
/// the steps and [`meet_of_three_surfaces`] hands back what this makes of them.
///
/// A [`MeetsAt::MaybeATouch`] is decided here, because a point either is a
/// corner or is not and there is no third thing for one to be. Which way it
/// goes is the caller's policy: a certified one refuses and a tolerant one
/// reads the touch as real, which is [`ask`]'s own rule and not a second one.
///
/// `unplaceable` says what to return when a step is real but the point it names
/// sits past the top of `f64`'s range — [`where_three_planes_meet`] has a
/// refusal from Cramer's rule to hand back there and nothing else does.
fn the_points_those_steps_name(
    named: &'static str,
    line: &ALine,
    found: WhereALineMeetsIt,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
    unplaceable: impl FnOnce() -> Error,
) -> Result<MeetOfThree> {
    let steps = match found {
        WhereALineMeetsIt::Nowhere => return Ok(MeetOfThree::Nowhere),
        WhereALineMeetsIt::MoreThanAPoint => return Ok(MeetOfThree::MoreThanAPoint),
        WhereALineMeetsIt::At(steps) => steps,
    };
    let mut places: SmallVec<[PredicatePoint; 2]> = SmallVec::new();
    for step in steps {
        let step = match step {
            MeetsAt::Here(step) => step,
            MeetsAt::MaybeATouch {
                inside,
                how_far_it_reaches,
            } => {
                // The quartic is a length to the fourth and `Measures` stops at
                // a volume; `where_three_planes_meet`'s note says why the
                // nearest arm is taken rather than a new one invented here, and
                // §28 says what should replace the lot.
                let touching = ask(
                    &how_far_it_reaches_over(named, how_far_it_reaches),
                    Measures::Volume,
                    at,
                    policy,
                    log,
                )?;
                assert!(
                    matches!(touching, Decision::Pos | Decision::Neg | Decision::Zero),
                    "a sign question answers with a sign: {touching:?}"
                );
                if !matches!(touching, Decision::Zero) {
                    continue;
                }
                inside
            }
        };
        places.push(the_point_at(line, step));
    }
    if places.iter().any(|place| !place.uncertainty().is_finite()) {
        return Err(unplaceable());
    }
    if places.is_empty() {
        return Ok(MeetOfThree::Nowhere);
    }
    Ok(MeetOfThree::At(places))
}

/// Where a line meets a surface, measured **along the line** rather than in
/// space.
///
/// A ray caster's question is about `t`: a meeting is ahead of where the ray
/// started or behind it, and that is the sign of `t` and nothing else. Handing
/// back points and letting the caller recover `t` from one would round the one
/// quantity [`where_a_line_meets_a_surface`] is careful about.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum WhereALineMeetsIt {
    /// The line misses the surface entirely.
    Nowhere,
    /// It meets the surface at these steps along itself, ascending — one, two,
    /// three or four of them. Three and four take a torus.
    At(SmallVec<[MeetsAt; 2]>),
    /// The line lies in the surface, so there is no step that names a meeting:
    /// a line in a plane, or one down a pipe's own wall.
    MoreThanAPoint,
}

/// One step along a line at which it meets a surface.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeetsAt {
    /// A meeting, this far along. A box, because solving for a step divides and
    /// a quotient of two ranges almost never lands on an `f64` (§2.5) — and a
    /// box of no width where the line touches a quadric at a step that does.
    Here(Interval),
    /// A touch somewhere in this range that no `f64` names, so no sign
    /// evaluation anywhere tells it from a near miss.
    ///
    /// **Not a refusal and not a meeting** — it is the predicate-level
    /// `Undecidable` this question has, and the caller's policy is what says
    /// which of the two it is worth. A **gap**:
    /// [`real_roots`](crate::real_roots) names the family of fixes.
    MaybeATouch {
        /// The box the touch would be in. Two neighbouring `f64`, or one.
        inside: Interval,
        /// How far the surface's own polynomial reaches over that box. It
        /// straddles zero, or this would not be here.
        how_far_it_reaches: Interval,
    },
}

/// Where a line meets one surface: nowhere, at up to four steps along itself, or
/// lying along the whole of it.
///
/// The line is `from + t·along`, and what comes back is `t`. **Both are boxes**,
/// because the point a fuse classifies is one: a corner the fuse built is known
/// to lie in a box and nothing recovers what was never there (§2.5). Where the
/// six boxes each name a single `f64` — which is what a corner on the lattice
/// comes to, and what `as_exact_as_it_is` exists to notice — every coefficient
/// below is an exact expansion and the exact level decides every sign. Where
/// they do not, the coefficients are boxes, the exact level cannot run at all,
/// and a sign whose box straddles zero is undecided.
///
/// This is [`meet_of_three_surfaces`] with the line handed over directly
/// instead of built from two crossing planes — the same coefficients and the
/// same cascade, not a second answer to the same question. It is what a ray
/// caster calls: a ray is not two planes, and [`real_roots`](crate::real_roots)
/// — the machinery a torus needs — is `pub(crate)`, so a caller outside this
/// crate could reach a plane, a ball, a pipe or a cone but never a ring
/// without this.
///
/// # Errors
///
/// [`Error::Undecidable`] when a sign the answer depends on could not be
/// settled, when a product the exact arithmetic needs falls out of `f64`'s
/// range, or when a step comes out past the top of that range. All three carry
/// the records.
///
/// The first is a **gap** when the line's own numbers are exact — the same line
/// and surface written elsewhere on the number line are answered — and it is
/// what a box of a line honestly comes to when it is not.
///
/// [`Error::WhereALineMeetsARingNeedsTheLineExactly`] for a torus asked about a
/// line that is a box. A **gap**, and [`real_roots`](crate::real_roots)'s own
/// doc says what closes it.
pub fn where_a_line_meets_a_surface(
    from: [Interval; 3],
    along: [Interval; 3],
    surface: &Surface,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<WhereALineMeetsIt> {
    let at = [
        PredicatePoint::Constructed(from),
        the_point_a_surface_sits_at(surface),
    ];
    let line = the_line_itself(from, along);
    let found = match surface {
        Surface::Plane(flat) => where_a_line_crosses_one_plane(
            WHERE_A_LINE_MEETS_A_SURFACE,
            &line,
            flat,
            &at,
            policy,
            log,
        )?,
        Surface::Torus(ring) => {
            where_a_line_meets_a_ring(WHERE_A_LINE_MEETS_A_SURFACE, &line, ring, &at)?
        }
        Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) => {
            let quadratic = how_a_quadric_cuts_the_line(&line, surface).ok_or_else(|| {
                an_exact_step_could_not_be_taken(WHERE_A_LINE_MEETS_A_SURFACE, &at)
            })?;
            where_a_line_meets_a_quadric(
                WHERE_A_LINE_MEETS_A_SURFACE,
                &quadratic,
                &at,
                policy,
                log,
            )?
        }
    };
    // A step that runs off the end of the number line is a crossing past the
    // top of `f64`'s range, which is nothing a caller can use — the same
    // reading `where_three_planes_meet` gives an unbounded box.
    if let WhereALineMeetsIt::At(steps) = &found
        && steps
            .iter()
            .any(|step| !(step.somewhere().lo().is_finite() && step.somewhere().hi().is_finite()))
    {
        return Err(an_exact_step_could_not_be_taken(
            WHERE_A_LINE_MEETS_A_SURFACE,
            &at,
        ));
    }
    Ok(found)
}

impl MeetsAt {
    /// The box this step is somewhere inside, whichever of the two it is.
    #[must_use]
    pub fn somewhere(self) -> Interval {
        match self {
            MeetsAt::Here(step) | MeetsAt::MaybeATouch { inside: step, .. } => step,
        }
    }
}

/// A line built straight from a ray's own two boxes, written the way [`ALine`]
/// writes one — `(through + s·direction) / divided_by` — with the divisor left
/// at one because there are no two planes here to divide by.
fn the_line_itself(from: [Interval; 3], along: [Interval; 3]) -> ALine {
    ALine {
        through: from.map(Term::from_a_box),
        direction: along.map(Term::from_a_box),
        divided_by: Term::of(1.0),
    }
}

/// The three surfaces in a fixed order, so what comes back depends on which
/// three they are and not on which face asked first.
///
/// Sorted on the numbers each one stores, planes first. That is a sound key for
/// exactly the reason §32.5's trap gives: two faces that are the same plane hold
/// the same four numbers. **A [`HalfSpace`](crate::HalfSpace) would not be one**
/// and §33.8 says why, which is the other half of the same rule.
fn in_a_fixed_order(surfaces: [&Surface; 3]) -> [Surface; 3] {
    let mut sorted = surfaces.map(|surface| *surface);
    sorted.sort_by(Surface::how_it_sorts_against);
    sorted
}

/// The three points a provenance record names these surfaces by.
fn the_places_these_three_sit(
    first: &Surface,
    second: &Surface,
    third: &Surface,
) -> [PredicatePoint; 3] {
    [first, second, third].map(the_point_a_surface_sits_at)
}

/// The point a provenance record names one surface by.
fn the_point_a_surface_sits_at(surface: &Surface) -> PredicatePoint {
    PredicatePoint::from(match surface {
        Surface::Plane(flat) => flat.origin().coordinates(),
        Surface::Sphere(ball) => ball.centre().coordinates(),
        Surface::Cylinder(pipe) => pipe.axis().origin().coordinates(),
        Surface::Cone(cone) => cone.apex().coordinates(),
        Surface::Torus(ring) => ring.centre().coordinates(),
    })
}

/// Three planes: Cramer's rule, and an exact second look when it gives up.
///
/// [`Plane::meet_of_three`] is the one authority on where three planes meet and
/// it stays that (§23.8). What it cannot do is tell "there is no single point"
/// from "there is one and interval arithmetic could not bound it", because its
/// determinant is worked out in intervals and a determinant that cancels
/// encloses zero either way.
///
/// So its refusal is read rather than passed on. The same determinant summed
/// through [`Exact`] is known to full relative accuracy — §29's lesson, one
/// question up — and it says which of the two happened. Where it says the three
/// really do meet at a point, the point is placed off the line the first two
/// planes cross in, which divides by that same well-known determinant instead of
/// by an interval one.
///
/// **A box with a side that runs on for ever is read as a refusal too.** Cramer's
/// rule divides one interval determinant by another and neither is checked for
/// overflow, so three planes whose offsets are near the top of the range and
/// whose normals lean at a small angle come back as `Ok` holding a box that says
/// nothing at all — and a caller would read that as a place. It is treated
/// exactly as a refusal is, which is what
/// `three_planes_meeting_past_the_top_of_the_range_are_refused_rather_than_placed_nowhere`
/// demonstrates.
fn where_three_planes_meet(
    one: &Plane,
    other: &Plane,
    last: &Plane,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    let at = the_places_these_three_sit(
        &Surface::Plane(*one),
        &Surface::Plane(*other),
        &Surface::Plane(*last),
    );
    let by_cramers_rule = Plane::meet_of_three(one, other, last);
    if let Ok(point) = by_cramers_rule
        && point.uncertainty().is_finite()
    {
        return Ok(MeetOfThree::At(smallvec![point]));
    }
    let refused = by_cramers_rule
        .err()
        .unwrap_or_else(|| an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, &at));

    let Some(line) = the_line_two_planes_cross_in(one, other, &at, policy, log)? else {
        return what_a_third_surface_leaves_of_one_plane(
            one,
            other,
            &Surface::Plane(*last),
            &at,
            policy,
            log,
        );
    };

    let found =
        where_a_line_crosses_one_plane(WHERE_THREE_SURFACES_MEET, &line, last, &at, policy, log)?;
    // Three planes leaning at a shrinking angle meet further and further away,
    // and past the top of `f64`'s range there is nothing left for a box to be
    // around — so this route runs out where Cramer's did, and what is passed on
    // is the first refusal rather than a second one saying the same thing.
    the_points_those_steps_name(
        WHERE_THREE_SURFACES_MEET,
        &line,
        found,
        &at,
        policy,
        log,
        || refused,
    )
}

/// `place`, a box round where three planes meet, narrowed by one checked step
/// from its middle ([`MeetOfThree::boxed_tight`] says what the step is).
///
/// **The box is handed back as it was where the step cannot be taken**: a
/// product that overflows, or a determinant whose enclosure reaches zero. It
/// holds the meet either way, so nothing is lost but the narrowing. A product
/// that falls off the bottom of the range is kept, as [`Nearly`] keeps it,
/// because the floor of a draw can store its offset as a subnormal (§281.2).
///
/// # Panics
///
/// If the stepped box and `place` do not overlap. Both hold the one point the
/// three planes meet at, so an empty overlap is arithmetic that lost its
/// soundness somewhere, and a corner placed from it would be nowhere.
fn narrowed_by_one_checked_step(planes: [&Plane; 3], place: PredicatePoint) -> PredicatePoint {
    let PredicatePoint::Constructed(boxed) = place else {
        return place;
    };
    let guess = boxed.map(|side| 0.5 * side.lo() + 0.5 * side.hi());
    let Some(step) = the_step_to_the_meet(planes, guess) else {
        return place;
    };
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        where_two_boxes_of_one_number_overlap(
            Interval::exactly(guess[axis]) + step[axis],
            boxed[axis],
        )
    }))
}

/// How far the meet of three planes is from `guess`, as an enclosure, or
/// `None` where a product it needs overflows.
///
/// A determinant whose enclosure reaches zero comes back as a step that is
/// everything, which leaves the caller's box as it was.
fn the_step_to_the_meet(planes: [&Plane; 3], guess: [f64; 3]) -> Option<[Interval; 3]> {
    let normals = planes.map(|plane| plane.normal().components());
    let misses: [Interval; 3] = [
        how_far_a_guess_misses(planes[0], guess)?,
        how_far_a_guess_misses(planes[1], guess)?,
        how_far_a_guess_misses(planes[2], guess)?,
    ];
    let across = [
        nearly_cross_of(normals[1], normals[2])?,
        nearly_cross_of(normals[2], normals[0])?,
        nearly_cross_of(normals[0], normals[1])?,
    ];
    let mut determinant = Nearly::of(0.0);
    for axis in 0..3 {
        determinant = determinant.plus(&Nearly::of(normals[0][axis]).times(&across[0][axis])?);
    }
    let determinant = determinant.enclosure();
    Some(core::array::from_fn(|axis| {
        let over = (0..3).fold(Interval::exactly(0.0), |so_far, row| {
            so_far + misses[row] * across[row][axis].enclosure()
        });
        quotient_or_everything(over, determinant)
    }))
}

/// `place`, a box round where two planes and a quadric meet, narrowed by one
/// checked step from its middle: [`narrowed_by_one_checked_step`]'s step, with
/// the quadric in the third plane's row.
///
/// **The row.** Write the quadric `Q(p) = p·(M p) + 2b·p + c`, with `M`
/// symmetric. Then for any two points `Q(x) − Q(g) = (x − g)·(M(x + g) + 2b)`,
/// exactly. At the meet `x`, `Q(x)` is zero, so `x − g` solves the system whose
/// rows are the two normals and `M(x + g) + 2b`, with right-hand sides how far
/// the guess misses each plane and `−Q(g)`. `x` lies in `place`, so
/// `M(place + g) + 2b` worked out in intervals holds that row, and Cramer's rule
/// worked out in intervals over it holds the answer for the row's true value.
/// `Q(g)` is worked out from the stored numbers like the planes' misses, so
/// every term of the numerator again carries a factor of how far the guess
/// misses, and what rounds there is a rounding of something small.
///
/// **The box is handed back as it was where the step cannot be taken**: a
/// product that overflows, or a determinant whose enclosure reaches zero,
/// which a line that all but grazes the quadric gives.
///
/// # Panics
///
/// If the stepped box and `place` do not overlap, as
/// [`narrowed_by_one_checked_step`] does.
fn narrowed_onto_a_quadric_by_one_checked_step(
    planes: [&Plane; 2],
    quadric: &HowASurfaceIsWritten,
    place: PredicatePoint,
) -> PredicatePoint {
    let PredicatePoint::Constructed(boxed) = place else {
        return place;
    };
    let guess = boxed.map(|side| 0.5 * side.lo() + 0.5 * side.hi());
    let Some(step) = the_step_onto_the_quadric(planes, quadric, boxed, guess) else {
        return place;
    };
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        where_two_boxes_of_one_number_overlap(
            Interval::exactly(guess[axis]) + step[axis],
            boxed[axis],
        )
    }))
}

/// How far the meet of two planes and a quadric inside `boxed` is from
/// `guess`, as an enclosure, or `None` where a product it needs overflows.
fn the_step_onto_the_quadric(
    planes: [&Plane; 2],
    quadric: &HowASurfaceIsWritten,
    boxed: [Interval; 3],
    guess: [f64; 3],
) -> Option<[Interval; 3]> {
    let misses = [
        how_far_a_guess_misses(planes[0], guess)?,
        how_far_a_guess_misses(planes[1], guess)?,
        -how_far_a_quadric_reaches_at(quadric, guess)?.enclosure(),
    ];
    let [one, other] = planes.map(|plane| plane.normal().components());
    let row: [Interval; 3] = core::array::from_fn(|axis| {
        (0..3).fold(
            Interval::exactly(2.0) * quadric.half_linear[axis].enclosure(),
            |so_far, column| {
                so_far
                    + quadric.squared[axis][column].enclosure()
                        * (boxed[column] + Interval::exactly(guess[column]))
            },
        )
    });
    let flat = nearly_cross_of(one, other)?.map(|component| component.enclosure());
    let across = [
        cross_enclosing(exactly(other), row),
        cross_enclosing(row, exactly(one)),
        flat,
    ];
    let determinant = dot_enclosing(row, flat);
    Some(core::array::from_fn(|axis| {
        let over = (0..3).fold(Interval::exactly(0.0), |so_far, which| {
            so_far + misses[which] * across[which][axis]
        });
        quotient_or_everything(over, determinant)
    }))
}

/// `Q(guess)` for a quadric written out, from the stored numbers with nothing
/// rounded but what falls off the bottom of the range, or `None` where a
/// product overflows.
fn how_far_a_quadric_reaches_at(quadric: &HowASurfaceIsWritten, guess: [f64; 3]) -> Option<Nearly> {
    let mut total = Nearly::from(quadric.constant.clone());
    for axis in 0..3 {
        let mut pulled = Nearly::from(quadric.half_linear[axis].clone()).times(&Nearly::of(2.0))?;
        for (column, &along) in guess.iter().enumerate() {
            pulled = pulled.plus(
                &Nearly::from(quadric.squared[axis][column].clone()).times(&Nearly::of(along))?,
            );
        }
        total = total.plus(&pulled.times(&Nearly::of(guess[axis]))?);
    }
    Some(total)
}

/// `offset − normal · guess` for one plane, from the stored numbers with
/// nothing rounded but what falls off the bottom of the range.
///
/// The offset enters as an entry of the sum and never as a factor, which is
/// what lets a subnormal offset through where an exact product of it would be
/// refused (§281.2).
fn how_far_a_guess_misses(plane: &Plane, guess: [f64; 3]) -> Option<Interval> {
    let along = Nearly::dot_of(plane.normal().components(), guess)?;
    Some(Nearly::of(plane.offset()).minus(&along).enclosure())
}

/// `u × v` for two triples of stored numbers, keeping what falls off the
/// bottom of the range as [`Nearly`] does.
fn nearly_cross_of(u: [f64; 3], v: [f64; 3]) -> Option<[Nearly; 3]> {
    let component = |axis: usize| {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        Some(Nearly::product_of(u[next], v[after])?.minus(&Nearly::product_of(u[after], v[next])?))
    };
    Some([component(0)?, component(1)?, component(2)?])
}

/// Where two enclosures of one number overlap.
///
/// # Panics
///
/// If they do not. Two boxes that each hold the same number share at least that
/// number, so an empty overlap is a broken enclosure.
fn where_two_boxes_of_one_number_overlap(one: Interval, other: Interval) -> Interval {
    let (lo, hi) = (one.lo().max(other.lo()), one.hi().min(other.hi()));
    Interval::new(lo, hi).unwrap_or_else(|_| {
        panic!("two boxes that each hold the same number do not overlap: {one:?} and {other:?}")
    })
}

/// What a third surface leaves of two planes that never cross in a line.
///
/// Either they are two different parallel planes, which already share nothing,
/// or they are one plane — and then the question is where that plane crosses the
/// third surface, which [`intersect`] answers and this does not answer a second
/// time (§23.8). §31.3's coincident-face case, and it costs one `match`.
fn what_a_third_surface_leaves_of_one_plane(
    one: &Plane,
    other: &Plane,
    third: &Surface,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    match one.agreement_with(other) {
        PlaneAgreement::Different => Ok(MeetOfThree::Nowhere),
        PlaneAgreement::SameWay | PlaneAgreement::OppositeWay => {
            what_one_plane_and_one_curved_surface_share(one, third, policy, log)
        }
        PlaneAgreement::CannotTell => Err(an_exact_step_could_not_be_taken(
            WHERE_THREE_SURFACES_MEET,
            at,
        )),
    }
}

/// The whole of what one plane and one other surface have in common, read as an
/// answer about three surfaces.
///
/// Two callers reach it, and both have already worked out that the third surface
/// takes nothing away: two planes that turned out to be one plane, and — through
/// [`where_a_plane_and_two_quadrics_of_one_bend_meet`] — two quadrics whose
/// difference holds everywhere, which is two ways of writing one surface.
/// [`intersect`] is the one authority on where a plane and a curved surface
/// cross and this does not answer it a second time (§23.8).
fn what_one_plane_and_one_curved_surface_share(
    flat: &Plane,
    other: &Surface,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    match intersect(&Surface::Plane(*flat), other, policy, log)? {
        SurfaceMeet::Nothing => Ok(MeetOfThree::Nowhere),
        SurfaceMeet::TouchAtOnePoint { at } => Ok(MeetOfThree::At(smallvec![at])),
        SurfaceMeet::Curves(_) | SurfaceMeet::SameSurface => Ok(MeetOfThree::MoreThanAPoint),
    }
}

/// One plane and two curved surfaces, by the three roads the module doc names.
///
/// **Nothing here guesses.** Each road either reaches an answer it has earned or
/// hands the question on, and what nothing reaches is the refusal that was here
/// before any road was: the quartic is real, its roots are real, and reaching
/// them takes a solver this file does not have.
///
/// The road for two surfaces turned about one axis comes before the one for a
/// plane that grazes, because a plane through a cone's apex square to its axis
/// is both, and the first answers it from stored numbers alone.
fn where_one_plane_and_two_curved_surfaces_meet(
    first: &Surface,
    second: &Surface,
    third: &Surface,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    let Surface::Plane(flat) = first else {
        return Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat);
    };
    let at = the_places_these_three_sit(first, second, third);
    if let Some(answer) =
        where_a_plane_and_two_quadrics_of_one_bend_meet(flat, second, third, &at, policy, log)?
    {
        return Ok(answer);
    }
    if let Some(answer) = what_a_plane_square_to_their_one_axis_leaves(flat, second, third, &at)? {
        return Ok(answer);
    }
    if let Some(answer) =
        what_a_plane_that_grazes_one_of_them_leaves(flat, second, third, &at, policy, log)?
    {
        return Ok(answer);
    }
    Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
}

/// A plane and two quadrics that bend the same way, or `None` when they do not
/// bend the same way and this road is not the one.
///
/// Write a quadric as `Q(p) = p·(M p) + 2b·p + c`. Where `M₁` and `M₂` are the
/// same nine numbers, `Q₁ − Q₂` has no `p·p` left in it: it is the flat surface
/// `2(b₁ − b₂)·p = c₂ − c₁`, and **every point the two quadrics share lies on
/// it**, because `Q₂ = Q₁ − (Q₁ − Q₂)`. So the three surfaces share exactly what
/// the plane, that flat surface and the first quadric share, which is two planes
/// and a quadric and is worked out above.
///
/// Two balls bend the same way, two pipes running the same way do, and two cones
/// of one slope about one direction do. A ball and a pipe never do:
/// [`HowASurfaceIsWritten::squared`] is `I` for the one and `(w·w)I − wwᵀ` for
/// the other, and a direction with no length is not a pipe. Two pipes match when
/// their stored directions are the same or opposite and at no other time, which
/// `two_bars_whose_axes_are_written_pointing_opposite_ways_still_meet_a_cap`
/// walks and `two_bars_leaning_across_each_other_and_a_plane_are_still_refused`
/// holds the other side of.
///
/// **The flat surface it builds is not a [`Plane`]**, and could not be: its
/// numbers are differences of products of the stored ones and an `f64` almost
/// never holds one. It is carried as the [`Exact`] expansions it really is, and
/// [`what_a_stored_plane_and_an_exact_one_share`] crosses it with the stored
/// plane without rounding anything on the way.
fn where_a_plane_and_two_quadrics_of_one_bend_meet(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<MeetOfThree>> {
    let (Some(mine), Some(theirs)) = (
        how_a_surface_is_written(one),
        how_a_surface_is_written(other),
    ) else {
        return Ok(None);
    };
    if !they_bend_the_same_way(&mine, &theirs) {
        return Ok(None);
    }

    // Past here the family is settled, so arithmetic that runs out is a
    // refusal about *this* triple and not a reason to try the next road.
    let leaning = core::array::from_fn(|axis| {
        let apart = mine.half_linear[axis].minus(&theirs.half_linear[axis]);
        apart.plus(&apart)
    });
    let reach = theirs.constant.minus(&mine.constant);
    let Some(shared) = what_a_stored_plane_and_an_exact_one_share(flat, &leaning, &reach) else {
        return Err(an_exact_step_could_not_be_taken(
            WHERE_THREE_SURFACES_MEET,
            at,
        ));
    };

    let line = match shared {
        WhatTwoPlanesShare::Nothing => return Ok(Some(MeetOfThree::Nowhere)),
        WhatTwoPlanesShare::TheWholeOfTheStoredOne => {
            return what_one_plane_and_one_curved_surface_share(flat, one, policy, log).map(Some);
        }
        WhatTwoPlanesShare::ALine(line) => line,
    };
    let quadratic = how_a_quadric_cuts_the_line(&line, one)
        .ok_or_else(|| an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, at))?;
    let found =
        where_a_line_meets_a_quadric(WHERE_THREE_SURFACES_MEET, &quadratic, at, policy, log)?;
    the_points_those_steps_name(
        WHERE_THREE_SURFACES_MEET,
        &line,
        found,
        at,
        policy,
        log,
        || an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, at),
    )
    .map(Some)
}

/// A quadric written out as `p·(M p) + 2b·p + c`, exactly.
///
/// The three names are the ones [`Quadratic`] uses for the same three parts of a
/// polynomial in one unknown, because they are the same three parts.
struct HowASurfaceIsWritten {
    /// `M`, the part that squares `p`. Symmetric, and all nine stored rather
    /// than the six that are different, because nine is what a comparison reads
    /// and there is no arithmetic here to save.
    squared: [[Exact; 3]; 3],
    /// `b`, which is **half** the part that is linear in `p`.
    half_linear: [Exact; 3],
    /// `c`, the part with no `p` in it.
    constant: Exact,
}

/// A quadric written out, or `None` for a surface that is not one — a torus,
/// whose own expression is a quartic, or a plane.
///
/// A plane is answered above and never arrives here. `None` rather than a panic,
/// because the reason it never arrives is that planes sort first — a fact about
/// [`Surface::how_it_sorts_against`] in another file, which is the trap
/// [`how_a_quadric_cuts_the_line`]'s own doc names.
///
/// `None` also where a product it needs falls out of `f64`'s range. That is a
/// refusal to say **whether the family applies**, so it hands the question on
/// rather than ending it.
fn how_a_surface_is_written(surface: &Surface) -> Option<HowASurfaceIsWritten> {
    match surface {
        Surface::Sphere(ball) => how_a_ball_is_written(ball),
        Surface::Cylinder(pipe) => how_a_pipe_is_written(pipe),
        Surface::Cone(cone) => how_a_cone_is_written(cone),
        Surface::Plane(_) | Surface::Torus(_) => None,
    }
}

/// `‖p − c‖² − R²`, which is [`Sphere`]'s own sign expression and not a second
/// spelling of it.
fn how_a_ball_is_written(ball: &Sphere) -> Option<HowASurfaceIsWritten> {
    let centre = ball.centre().coordinates();
    Some(HowASurfaceIsWritten {
        squared: core::array::from_fn(|row| {
            core::array::from_fn(|column| Exact::of(if row == column { 1.0 } else { 0.0 }))
        }),
        half_linear: core::array::from_fn(|axis| Exact::of(-centre[axis])),
        constant: Exact::dot_of(centre, centre)?
            .minus(&Exact::product_of(ball.radius(), ball.radius())?),
    })
}

/// `‖(p − o) × w‖² − r²(w·w)`, which is [`Cylinder`]'s own sign expression,
/// opened out as `(w·w)(p·p) − (p·w)²` plus what the axis point leaves.
///
/// **The radius is squared against `w·w`**, because everything else here is
/// degree two in the stored direction and `r²` on its own is degree zero. Scale
/// `w` by any number and every one of these nine, three and one coefficients
/// scales by the square of it, so the surface they name is the pipe of radius
/// exactly `r` whatever length the direction got stored at — which is the
/// surface [`SideOfSurface`](rcad_predicates::SideOfSurface) certifies for a
/// pipe since §118.10. Take the `w·w` away and the constant term alone stops
/// scaling, and what is written out is the pipe of radius `r/‖w‖` (§27.1).
fn how_a_pipe_is_written(pipe: &Cylinder) -> Option<HowASurfaceIsWritten> {
    let along = pipe.axis().direction().components();
    let origin = pipe.axis().origin().coordinates();
    let mut written = how_a_ruled_surface_is_written(along, origin, &Exact::of(1.0))?;
    written.constant = written.constant.minus(
        &Exact::product_of(pipe.radius(), pipe.radius())?.times(&Exact::dot_of(along, along)?)?,
    );
    Some(written)
}

/// `‖(p − apex) × w‖² − slope²((p − apex)·w)²`, which is [`Cone`]'s own sign
/// expression, opened out as `(w·w)(p·p) − (1 + slope²)(p·w)²` plus what the
/// apex leaves.
fn how_a_cone_is_written(cone: &Cone) -> Option<HowASurfaceIsWritten> {
    let along = cone.axis().direction().components();
    let apex = cone.apex().coordinates();
    let squeeze = Exact::of(1.0).plus(&Exact::product_of(cone.slope(), cone.slope())?);
    how_a_ruled_surface_is_written(along, apex, &squeeze)
}

/// `(w·w)(x·x) − squeeze·(x·w)²` for `x = p − place`, written out in `p`.
///
/// A pipe and a cone are the same expression with a different `squeeze` — one
/// for a pipe, `1 + slope²` for a cone — because `‖x × w‖²` is
/// `(w·w)(x·x) − (x·w)²` and a cone takes `slope²(x·w)²` off that as well.
fn how_a_ruled_surface_is_written(
    along: [f64; 3],
    place: [f64; 3],
    squeeze: &Exact,
) -> Option<HowASurfaceIsWritten> {
    let axis_squared = Exact::dot_of(along, along)?;
    let reach = Exact::dot_of(place, along)?;
    let squeezed_reach = squeeze.times(&reach)?;

    let mut squared = nothing_squared();
    for (row, entries) in squared.iter_mut().enumerate() {
        for (column, entry) in entries.iter_mut().enumerate() {
            let leaning = squeeze.times(&Exact::product_of(along[row], along[column])?)?;
            *entry = if row == column {
                axis_squared.minus(&leaning)
            } else {
                Exact::of(0.0).minus(&leaning)
            };
        }
    }

    let mut half_linear = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (axis, entry) in half_linear.iter_mut().enumerate() {
        *entry = squeezed_reach
            .times(&Exact::of(along[axis]))?
            .minus(&axis_squared.times(&Exact::of(place[axis]))?);
    }

    Some(HowASurfaceIsWritten {
        squared,
        half_linear,
        constant: axis_squared
            .times(&Exact::dot_of(place, place)?)?
            .minus(&squeezed_reach.times(&reach)?),
    })
}

/// Nine zeroes, to be written over.
fn nothing_squared() -> [[Exact; 3]; 3] {
    core::array::from_fn(|_| core::array::from_fn(|_| Exact::of(0.0)))
}

/// Whether two quadrics square `p` the same way, so their difference is flat.
///
/// Nine exact comparisons and no arithmetic that can round: an expansion always
/// has a sign, so this always answers.
fn they_bend_the_same_way(mine: &HowASurfaceIsWritten, theirs: &HowASurfaceIsWritten) -> bool {
    mine.squared
        .iter()
        .zip(theirs.squared.iter())
        .all(|(row, other)| {
            row.iter()
                .zip(other.iter())
                .all(|(entry, against)| matches!(entry.minus(against).sign(), Decision::Zero))
        })
}

/// What a stored plane and a plane written out exactly have in common.
///
/// The line is boxed, and the trade is the opposite of the one
/// [`SurfaceMeet::Curves`](crate::SurfaceMeet) refuses: this is never the answer
/// a fuse hands about, only a step on the way to one, and the arm carrying the
/// line is followed by a degree-fourteen exact discriminant that costs far more
/// than an allocation. Widening the other two arms to two hundred bytes to save
/// it would be paying in the wrong place.
enum WhatTwoPlanesShare {
    /// One endless line.
    ALine(Box<ALine>),
    /// The whole of the stored plane, because the two are one plane — or because
    /// the exact one is the equation `0 = 0`, which holds everywhere and is what
    /// two quadrics written with the same numbers come to.
    TheWholeOfTheStoredOne,
    /// Nothing at all: two parallel planes that are not the same one, or the
    /// equation `0 = d` for a `d` that is not zero, which two quadrics that
    /// differ only in size come to.
    Nothing,
}

/// What a stored plane and `normal·p = offset` share, or `None` when a product
/// it needs falls out of `f64`'s range.
///
/// The same shape as [`the_line_two_planes_cross_in`] and for the same reasons —
/// the zeroed coordinate is the one whose minor is largest, the ranking is
/// worked out in `f64` and only ranks, and that a minor is **not zero** is
/// decided exactly. What is different is the second plane's numbers being
/// [`Exact`] expansions, and what that buys is the end of it: two normals whose
/// every minor is exactly zero are parallel, decided, with no cascade to ask and
/// no policy to consult.
fn what_a_stored_plane_and_an_exact_one_share(
    flat: &Plane,
    normal: &[Exact; 3],
    offset: &Exact,
) -> Option<WhatTwoPlanesShare> {
    let mine = Exact::of_each(flat.normal().components());
    let my_offset = Exact::of(flat.offset());
    let theirs = normal;
    let their_offset = offset;

    let mut by_size = [0usize, 1, 2];
    let ranked: [f64; 3] = core::array::from_fn(|axis| {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        mine[next].nearest_f64() * theirs[after].nearest_f64()
            - mine[after].nearest_f64() * theirs[next].nearest_f64()
    });
    by_size.sort_by(|&this, &that| ranked[that].abs().total_cmp(&ranked[this].abs()));

    let mut something_would_not_fit = false;
    for axis in by_size {
        let (across, and) = ((axis + 1) % 3, (axis + 2) % 3);
        let Some(minor) =
            exactly_this_minus_that(&mine[across], &theirs[and], &mine[and], &theirs[across])
        else {
            something_would_not_fit = true;
            continue;
        };
        let which_way = minor.sign();
        if matches!(which_way, Decision::Zero) {
            continue;
        }
        // An expansion always has a sign, so nothing reaches the second half
        // today. It is read as arithmetic that ran out rather than as a zero
        // minor, because falling through to the parallel test below on a sign
        // nobody proved would answer a question this never asked.
        if !matches!(which_way, Decision::Pos | Decision::Neg) {
            something_would_not_fit = true;
            continue;
        }
        let (Some(here), Some(there), Some(direction)) = (
            exactly_this_minus_that(&my_offset, &theirs[and], their_offset, &mine[and]),
            exactly_this_minus_that(&mine[across], their_offset, &theirs[across], &my_offset),
            cross_of_two_exact(&mine, theirs),
        ) else {
            something_would_not_fit = true;
            continue;
        };
        let mut through = [Term::of(0.0), Term::of(0.0), Term::of(0.0)];
        through[across] = Term::exact(here);
        through[and] = Term::exact(there);
        return Some(WhatTwoPlanesShare::ALine(Box::new(ALine {
            through,
            direction: direction.map(Term::exact),
            divided_by: Term::exact(minor),
        })));
    }
    if something_would_not_fit {
        return None;
    }

    // Parallel, so what tells the same plane from two that never meet is whether
    // the offsets are in the same ratio as the normals — `d₂·n₁ − d₁·n₂`,
    // componentwise (§17.4). A stored plane's normal has length, so where the
    // exact one's normal is all zeroes this reads `d₂·n₁`, which is zero exactly
    // when `d₂` is: the equation `0 = 0` holds everywhere and `0 = d₂` nowhere.
    for axis in 0..3 {
        let same = exactly_this_minus_that(their_offset, &mine[axis], &my_offset, &theirs[axis])?;
        let which_way = same.sign();
        if matches!(which_way, Decision::Zero) {
            continue;
        }
        if matches!(which_way, Decision::Pos | Decision::Neg) {
            return Some(WhatTwoPlanesShare::Nothing);
        }
        return None;
    }
    Some(WhatTwoPlanesShare::TheWholeOfTheStoredOne)
}

/// `first · second − third · fourth` over four expansions, or `None` when either
/// product would round.
fn exactly_this_minus_that(
    first: &Exact,
    second: &Exact,
    third: &Exact,
    fourth: &Exact,
) -> Option<Exact> {
    Some(first.times(second)?.minus(&third.times(fourth)?))
}

/// `u × v` for two triples of expansions, exactly.
fn cross_of_two_exact(u: &[Exact; 3], v: &[Exact; 3]) -> Option<[Exact; 3]> {
    let mut across = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (axis, component) in across.iter_mut().enumerate() {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        *component = exactly_this_minus_that(&u[next], &v[after], &u[after], &v[next])?;
    }
    Some(across)
}

/// What a plane square to the one axis two curved surfaces share leaves of the
/// three, or `None` where the three are not that.
///
/// A ball, a pipe and a cone each look the same from every side of their axis —
/// and any line through a ball's centre is an axis of it — so a plane square to
/// the axis cuts each in a circle about the point where the axis crosses the
/// plane, or in that one point, or misses it. **Two circles about one centre
/// are one circle or share nothing**, so the whole of the answer is whether
/// their radii are equal: no quartic, and no point to place but the one a
/// radius of zero names. A bar of radius three through a ball of radius five,
/// and a cap across both at height two, is `3²` against `5² − 2²`, and they
/// share nothing.
///
/// Each radius is compared squared and times `n·n`, as
/// [`ACircleAboutAnAxis::squared_radius`] says, which keeps every one a
/// polynomial in the stored numbers whatever length the normal is stored at.
///
/// **Square, and one axis, are both decided exactly**: a pipe's or a cone's
/// stored direction crossed with the plane's normal is zero in every
/// component, and so is the gap between the two points on the axes crossed with
/// it. A direction a rounding off square cuts an ellipse, which is the quartic
/// again, and is handed on. So is a triple where a product those two checks
/// need falls out of `f64`'s range, because that is a refusal to say whether
/// this road is the one — the same rule [`how_a_surface_is_written`] keeps.
///
/// Two radii of zero are the one point where the axis crosses the plane. Where
/// one of the two is a cone, that point is its apex, which is stored and so is
/// handed back as itself. Two balls that both touch the plane there have no
/// stored point to hand back, and go on to
/// [`what_a_plane_that_grazes_one_of_them_leaves`], which places a touch.
fn what_a_plane_square_to_their_one_axis_leaves(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    at: &[PredicatePoint],
) -> Result<Option<MeetOfThree>> {
    let (Some(mine), Some(theirs)) = (
        the_circle_a_square_plane_cuts(flat, one),
        the_circle_a_square_plane_cuts(flat, other),
    ) else {
        return Ok(None);
    };
    let Some(off_one_axis) = Exact::cross_of(
        &Exact::gap_between(mine.on_the_axis, theirs.on_the_axis),
        flat.normal().components(),
    ) else {
        return Ok(None);
    };
    if !off_one_axis
        .iter()
        .all(|component| matches!(component.sign(), Decision::Zero))
    {
        return Ok(None);
    }

    // Past here the family is settled, so arithmetic that runs out is a
    // refusal about *this* triple, which a slide along the range can retry.
    let refused = || an_exact_step_could_not_be_taken(WHERE_THREE_SURFACES_MEET, at);
    let (Some(my_reach), Some(their_reach)) = (mine.squared_radius, theirs.squared_radius) else {
        return Err(refused());
    };
    let signs = [&my_reach, &their_reach, &my_reach.minus(&their_reach)].map(Exact::sign);
    if !signs
        .iter()
        .all(|sign| matches!(sign, Decision::Pos | Decision::Neg | Decision::Zero))
    {
        return Err(refused());
    }
    let [my_way, their_way, how_they_differ] = signs;
    if matches!(my_way, Decision::Neg)
        || matches!(their_way, Decision::Neg)
        || !matches!(how_they_differ, Decision::Zero)
    {
        return Ok(Some(MeetOfThree::Nowhere));
    }
    if matches!(my_way, Decision::Pos) {
        return Ok(Some(MeetOfThree::MoreThanAPoint));
    }
    Ok(mine
        .apex
        .or(theirs.apex)
        .map(|apex| MeetOfThree::At(smallvec![PredicatePoint::Exact(apex)])))
}

/// The circle a plane square to a ball's, a pipe's or a cone's axis cuts it in.
struct ACircleAboutAnAxis {
    /// A stored point on the axis: a ball's centre, a pipe's axis origin or a
    /// cone's apex.
    on_the_axis: [f64; 3],
    /// The circle's radius squared and times `n·n`, exactly, or `None` where a
    /// product it needs falls out of `f64`'s range.
    ///
    /// - A ball, `(n·n)R² − (n·c − d)²`, which is [`intersect`]'s own `room`
    ///   for a plane and a ball, and below zero where the plane misses it.
    /// - A pipe, `(n·n)r²`, because a pipe's radius is `r` whatever length its
    ///   direction is stored at (§118.10).
    /// - A cone, `slope²(n·a − d)²`: its radius at the plane is `slope` times
    ///   how far the plane is from the apex along the axis, and that distance
    ///   is `(n·a − d)/‖n‖`.
    squared_radius: Option<Exact>,
    /// The apex, where this is a cone: the one point a radius of zero is.
    apex: Option<[f64; 3]>,
}

/// The circle a plane square to `surface`'s axis cuts it in, or `None` where
/// the surface has no axis square to the plane — a pipe or a cone that leans,
/// a torus, a plane — or where whether its axis leans could not be worked out
/// exactly.
///
/// A torus about an axis square to the plane is cut in circles too, but in two
/// of them, and is left to the refusal.
fn the_circle_a_square_plane_cuts(flat: &Plane, surface: &Surface) -> Option<ACircleAboutAnAxis> {
    let normal = flat.normal().components();
    let square_to_the_plane = |along: [f64; 3]| {
        exact_cross_of(normal, along).is_some_and(|across| {
            across
                .iter()
                .all(|component| matches!(component.sign(), Decision::Zero))
        })
    };
    match surface {
        Surface::Sphere(ball) => Some(ACircleAboutAnAxis {
            on_the_axis: ball.centre().coordinates(),
            squared_radius: how_wide_a_plane_cuts_a_ball(flat, ball),
            apex: None,
        }),
        Surface::Cylinder(pipe) => {
            square_to_the_plane(pipe.axis().direction().components()).then(|| ACircleAboutAnAxis {
                on_the_axis: pipe.axis().origin().coordinates(),
                squared_radius: Exact::dot_of(normal, normal).and_then(|squared_normal| {
                    squared_normal.times(&Exact::product_of(pipe.radius(), pipe.radius())?)
                }),
                apex: None,
            })
        }
        Surface::Cone(cone) => {
            square_to_the_plane(cone.axis().direction().components()).then(|| ACircleAboutAnAxis {
                on_the_axis: cone.apex().coordinates(),
                squared_radius: how_wide_a_square_plane_cuts_a_cone(flat, cone),
                apex: Some(cone.apex().coordinates()),
            })
        }
        Surface::Plane(_) | Surface::Torus(_) => None,
    }
}

/// `(n·n)R² − (n·c − d)²`, exactly, or `None` where a product falls out of
/// range.
fn how_wide_a_plane_cuts_a_ball(flat: &Plane, ball: &Sphere) -> Option<Exact> {
    let normal = flat.normal().components();
    let reach = how_far_a_plane_is_from(flat, ball.centre().coordinates())?;
    Some(
        Exact::dot_of(normal, normal)?
            .times(&Exact::product_of(ball.radius(), ball.radius())?)?
            .minus(&reach.times(&reach)?),
    )
}

/// `slope²(n·a − d)²`, exactly, or `None` where a product falls out of range.
fn how_wide_a_square_plane_cuts_a_cone(flat: &Plane, cone: &Cone) -> Option<Exact> {
    let reach = how_far_a_plane_is_from(flat, cone.apex().coordinates())?;
    Exact::product_of(cone.slope(), cone.slope())?.times(&reach.times(&reach)?)
}

/// `n·p − d`, exactly: how far `place` is in front of the plane, times `‖n‖`.
fn how_far_a_plane_is_from(flat: &Plane, place: [f64; 3]) -> Option<Exact> {
    Some(Exact::dot_of(flat.normal().components(), place)?.minus(&Exact::of(flat.offset())))
}

/// What a plane that misses one of the two curved surfaces, or touches it at a
/// single point, leaves of the three — or `None` where it does neither.
///
/// **A plane that misses a quadric settles the triple on its own**: nothing on
/// the plane is on that quadric, so nothing is on all three. And a plane that
/// touches one at a single point leaves exactly one candidate, so the whole of
/// what is left is which side of the *other* surface that point is on. Neither
/// needs a quartic and neither is a guess.
///
/// A ball resting on a block's floor is the second: the floor touches the ball
/// at one point, and whether a bar also resting on that floor passes through it
/// is one sign question. It is answered the same way at every distance, because
/// nothing in it depends on how far apart the two round solids are.
///
/// [`intersect`] refusing is not passed on. It says this road could not be
/// walked, which is a different thing from the triple having no answer, and the
/// other surface is still worth asking about. What is passed on is the refusal
/// at the end of every road, which was the answer before this road existed.
fn what_a_plane_that_grazes_one_of_them_leaves(
    flat: &Plane,
    one: &Surface,
    other: &Surface,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<MeetOfThree>> {
    for (this, that) in [(one, other), (other, one)] {
        match intersect(&Surface::Plane(*flat), this, policy, log) {
            Ok(SurfaceMeet::Nothing) => return Ok(Some(MeetOfThree::Nowhere)),
            Ok(SurfaceMeet::TouchAtOnePoint { at: touch }) => {
                return whether_a_touch_is_on_it(touch, that, at).map(Some);
            }
            Ok(SurfaceMeet::Curves(_) | SurfaceMeet::SameSurface) | Err(_) => (),
        }
    }
    Ok(None)
}

/// Whether the one point a plane touches a surface at is on a second surface
/// too.
///
/// **Certified and never tolerant**, and that is the one place in this file
/// where the caller's policy does not reach.
/// [`SideOfSurface`](rcad_predicates::SideOfSurface) hands back a
/// [`Decision`] and not the [`Level`] it reached, so there is no [`Record`] to
/// put in front of a policy and nothing to convert against its budget. Refusing
/// is the conservative half of that: a tolerant policy only ever accepts more,
/// so a refusal here is never an answer given wrongly — only one not given.
/// Closing it takes a `decide`-shaped route into a surface's own sign
/// expression, which is a change to another crate.
fn whether_a_touch_is_on_it(
    touch: PredicatePoint,
    surface: &Surface,
    at: &[PredicatePoint],
) -> Result<MeetOfThree> {
    match which_side_of_it(surface, touch) {
        Decision::Zero => Ok(MeetOfThree::At(smallvec![touch])),
        Decision::Pos | Decision::Neg => Ok(MeetOfThree::Nowhere),
        Decision::Undecidable { enclosure } => Err(a_touching_point_could_not_be_placed(
            WHERE_THREE_SURFACES_MEET,
            at,
            enclosure,
        )),
        other => unreachable!("a side-of question answers with one of the four: {other:?}"),
    }
}

/// Which side of a surface a point is on, for every kind there is.
fn which_side_of_it(surface: &Surface, point: PredicatePoint) -> Decision {
    match surface {
        Surface::Plane(flat) => which_side_of_a_flat_surface(
            WHERE_THREE_SURFACES_MEET,
            flat.normal().components(),
            flat.offset(),
            point,
        ),
        Surface::Sphere(ball) => ball.side_of(point),
        Surface::Cylinder(pipe) => pipe.side_of(point),
        Surface::Cone(cone) => cone.side_of(point),
        Surface::Torus(ring) => ring.side_of(point),
    }
}

/// The refusal for a touching point that could be put neither on the third
/// surface nor off it.
///
/// **Not the exact arithmetic running out.** Where a plane touches a quadric is
/// a quotient and rarely lands on an `f64` (§2.5), so the box has width and an
/// interval evaluation of the third surface's own expression over it straddles
/// zero. A **gap** where the touch is on the lattice — that case is answered,
/// and a ball on a block's floor is it — and what a box of a point honestly
/// comes to where it is not.
fn a_touching_point_could_not_be_placed(
    named: &'static str,
    at: &[PredicatePoint],
    enclosure: f64,
) -> Error {
    let mut records = ProvenanceLog::new();
    records.push(Record::new(
        named,
        at,
        Level::Interval,
        enclosure,
        Measures::Area,
    ));
    Error::Undecidable { records }
}

/// A line, written as `(through + s · direction) / divided_by`.
///
/// Two planes crossing build one, and so does a ray. **The point on the line two
/// planes give is the one with a zero coordinate, not the foot of the
/// perpendicular from the origin**, and that choice is what keeps the degrees
/// down: the perpendicular foot needs `‖n₁ × n₂‖²` underneath it and carries
/// four more degrees into everything built on it.
struct ALine {
    through: [Term; 3],
    direction: [Term; 3],
    divided_by: Term,
}

/// One number a line or a surface is written out of, at the best level the
/// caller's own numbers allow.
///
/// **Which of the two it is is settled once, at the top, and never mixes.** Two
/// planes crossing hand over stored `f64`, so everything built from them is an
/// exact expansion and level 3 decides every sign. A ray from a corner a fuse
/// *built* hands over boxes, so everything built from them is a box, level 3
/// cannot run at all, and a sign whose box straddles zero is undecided — which
/// is the honest answer and not a shortcoming of the arithmetic (§2.5). A box
/// that names one `f64` is that `f64`, which is what
/// [`Term::from_a_box`] reads and `as_exact_as_it_is` exists to notice.
///
/// **A product of stored numbers that falls off the bottom of the range stays
/// on the stored side.** It is held as a [`Nearly`]: the expansion, and a box
/// round what fell off, a smallest subnormal per product that rounded. Its
/// exact level then cannot run, and its enclosure is that expansion and that
/// box — tight enough to decide a discriminant of `3` where a term of `2⁻²⁰¹⁴`
/// went missing, which a round bar moved by `2⁻¹⁰⁰⁷` against a cube's wall
/// asks for (`docs/decisions.md` §322). Before, such a product refused the
/// whole meet with an unbounded enclosure.
#[derive(Clone)]
enum Term {
    /// Worked out from stored numbers, so known exactly — or, where a product
    /// fell off the bottom of the range, to within a box of subnormals.
    Stored(Nearly),
    /// Known only as a box, because the line it was built from is one.
    Boxed(Interval),
}

impl Term {
    /// A stored number, exactly.
    fn of(value: f64) -> Self {
        Term::Stored(Nearly::of(value))
    }

    /// A number worked out exactly.
    fn exact(value: Exact) -> Self {
        Term::Stored(value.into())
    }

    /// A box, or the number it names where it names one.
    fn from_a_box(enclosure: Interval) -> Self {
        if enclosure.lo() == enclosure.hi() {
            return Term::of(enclosure.lo());
        }
        Term::Boxed(enclosure)
    }

    /// This number as an enclosure.
    fn enclosure(&self) -> Interval {
        match self {
            Term::Stored(value) => value.enclosure(),
            Term::Boxed(enclosure) => *enclosure,
        }
    }

    /// This number exactly, or `None` where it was never known that well.
    fn exactly(&self) -> Option<Exact> {
        match self {
            Term::Stored(value) => value.exactly(),
            Term::Boxed(_) => None,
        }
    }

    /// Whether this was worked out from stored numbers and lost more than a
    /// rounding of itself off the bottom of the range on the way — the one case
    /// where it is refused as it always was ([`Nearly::is_within_a_rounding`]).
    fn lost_more_than_a_rounding(&self) -> bool {
        match self {
            Term::Stored(value) => !value.is_within_a_rounding(),
            Term::Boxed(_) => false,
        }
    }

    /// Whether this was worked out from stored numbers and lost anything at
    /// all off the bottom of the range.
    fn lost_anything(&self) -> bool {
        matches!(self, Term::Stored(value) if value.exactly().is_none())
    }

    /// Which way this number points, or [`NOTHING_IS_KNOWN`] where a box
    /// straddles zero.
    fn sign(&self) -> Decision {
        if let Some(value) = self.exactly() {
            return value.sign();
        }
        let enclosure = self.enclosure();
        if enclosure.lo() > 0.0 {
            return Decision::Pos;
        }
        if enclosure.hi() < 0.0 {
            return Decision::Neg;
        }
        NOTHING_IS_KNOWN
    }

    /// This plus `other`.
    fn plus(&self, other: &Term) -> Term {
        match (self, other) {
            (Term::Stored(mine), Term::Stored(theirs)) => Term::Stored(mine.plus(theirs)),
            _ => Term::Boxed(self.enclosure() + other.enclosure()),
        }
    }

    /// This minus `other`.
    fn minus(&self, other: &Term) -> Term {
        match (self, other) {
            (Term::Stored(mine), Term::Stored(theirs)) => Term::Stored(mine.minus(theirs)),
            _ => Term::Boxed(self.enclosure() - other.enclosure()),
        }
    }

    /// This times `other`, or `None` when both are stored and the product
    /// overflows.
    ///
    /// **Two stored numbers whose product does not fit do not quietly become a
    /// box.** That refusal is the one every discriminant in
    /// [`intersect`](crate::intersect) has, and answering it with an interval
    /// instead would give a caller a second, wider answer to a question it had
    /// already been told nobody could take. A product that falls off the
    /// *bottom* is another matter, and [`Term`]'s own doc says why it is kept.
    fn times(&self, other: &Term) -> Option<Term> {
        match (self, other) {
            (Term::Stored(mine), Term::Stored(theirs)) => Some(Term::Stored(mine.times(theirs)?)),
            _ => Some(Term::Boxed(self.enclosure() * other.enclosure())),
        }
    }
}

/// `a · b` over two triples.
fn dot_of(u: &[Term; 3], v: &[Term; 3]) -> Option<Term> {
    let mut total = Term::of(0.0);
    for axis in 0..3 {
        total = total.plus(&u[axis].times(&v[axis])?);
    }
    Some(total)
}

/// `u · u` for one triple.
fn squared_length_of(u: &[Term; 3]) -> Option<Term> {
    dot_of(u, u)
}

/// `u × v` for a triple and a triple of stored numbers.
fn cross_of(u: &[Term; 3], v: [f64; 3]) -> Option<[Term; 3]> {
    let component = |axis: usize| {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        let left = u[next].times(&Term::of(v[after]))?;
        let right = u[after].times(&Term::of(v[next]))?;
        Some(left.minus(&right))
    };
    Some([component(0)?, component(1)?, component(2)?])
}

/// The line two planes cross in, or `None` when they never do.
///
/// The coordinate that is zeroed is the one whose two-by-two minor of the two
/// normals is largest, so the division below is by the best-conditioned of the
/// three. Which one is picked is worked out in `f64` and only ranks the
/// candidates; that a minor is **not zero** is decided exactly, and an axis
/// whose minor cannot be built at all is passed over rather than trusted.
///
/// # Errors
///
/// [`Error::Undecidable`] when the two planes are not parallel and no minor
/// could be worked out exactly, or when whether they are parallel could not be
/// settled at all.
fn the_line_two_planes_cross_in(
    one: &Plane,
    other: &Plane,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Option<ALine>> {
    let [mine, theirs] = [one, other].map(|plane| plane.normal().components());
    let [my_offset, their_offset] = [one, other].map(Plane::offset);

    let mut by_size = [0usize, 1, 2];
    let ranked = one.normal().cross(other.normal()).components();
    by_size.sort_by(|&this, &that| ranked[that].abs().total_cmp(&ranked[this].abs()));

    for axis in by_size {
        let (across, and) = ((axis + 1) % 3, (axis + 2) % 3);
        let Some(minor) = cross_of_two_pairs(mine[across], theirs[and], mine[and], theirs[across])
        else {
            continue;
        };
        if !matches!(minor.sign(), Decision::Pos | Decision::Neg) {
            continue;
        }
        let (Some(here), Some(there), Some(direction)) = (
            cross_of_two_pairs(my_offset, theirs[and], their_offset, mine[and]),
            cross_of_two_pairs(mine[across], their_offset, theirs[across], my_offset),
            exact_cross_of(mine, theirs),
        ) else {
            continue;
        };
        let mut through = [Term::of(0.0), Term::of(0.0), Term::of(0.0)];
        through[across] = Term::exact(here);
        through[and] = Term::exact(there);
        return Ok(Some(ALine {
            through,
            direction: direction.map(Term::exact),
            divided_by: Term::exact(minor),
        }));
    }

    // Every minor came back zero or could not be built, and those are different
    // answers: the first says the two planes are parallel and the second says
    // this arithmetic ran out. `‖n₁ × n₂‖²` is the same question asked where
    // level 2 can also have a go at it.
    let across = squared_length_of_a_cross_product(WHERE_THREE_SURFACES_MEET, mine, theirs);
    match ask(&across, Measures::Length, at, policy, log)? {
        Decision::Zero => Ok(None),
        Decision::Pos => Err(an_exact_step_could_not_be_taken(
            WHERE_THREE_SURFACES_MEET,
            at,
        )),
        other => unreachable!("a sum of squares is never below zero: {other:?}"),
    }
}

/// Where a line crosses one plane: nowhere, at one step along it, or lying in
/// it.
///
/// Shared between two callers that reach a plane by a different road each:
/// [`where_three_planes_meet`] already has the line two other planes cross in,
/// and [`where_a_line_meets_a_surface`] has a line handed straight over.
fn where_a_line_crosses_one_plane(
    named: &'static str,
    line: &ALine,
    plane: &Plane,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<WhereALineMeetsIt> {
    let normal = plane.normal().components().map(Term::of);
    let (Some(along), Some(across)) = (
        dot_of(&line.direction, &normal),
        dot_of(&line.through, &normal).and_then(|reach| {
            Some(reach.minus(&line.divided_by.times(&Term::of(plane.offset()))?))
        }),
    ) else {
        return Err(an_exact_step_could_not_be_taken(named, at));
    };
    // Three planes have Cramer's rule to fall back on in
    // `where_three_planes_meet`, so a product that fell off the bottom here is
    // refused as it was and that route is left to answer.
    if along.lost_anything() || across.lost_anything() {
        return Err(an_exact_step_could_not_be_taken(named, at));
    }

    // `along` is the determinant of the three normals, which is dimensionless.
    // `Measures` has no arm for that; see the note in
    // `intersect::where_a_plane_and_a_pipe_cross` for why `Length` is the one
    // whose conversion is the identity, and §28 for what should replace it.
    match ask(
        &the_sign_of(named, &along),
        Measures::Length,
        at,
        policy,
        log,
    )? {
        Decision::Pos | Decision::Neg => Ok(WhereALineMeetsIt::At(smallvec![MeetsAt::Here(
            quotient_or_everything(-across.enclosure(), along.enclosure())
        )])),
        Decision::Zero => match ask(
            &the_sign_of(named, &across),
            Measures::Length,
            at,
            policy,
            log,
        )? {
            Decision::Zero => Ok(WhereALineMeetsIt::MoreThanAPoint),
            Decision::Pos | Decision::Neg => Ok(WhereALineMeetsIt::Nowhere),
            other => unreachable!("a sign question answers with a sign: {other:?}"),
        },
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// Where the line meets a ring: nowhere, or at up to four points.
///
/// **A line never lies along a torus**, which is what makes this shorter than
/// the quadric route below it: the leading coefficient is `(w·w)(D·D)²` for a
/// direction `D` two crossing planes handed over and an axis direction a stored
/// [`Axis`](crate::Axis) normalised, and neither of those is zero. So the
/// quartic is a quartic, never the zero polynomial, and there is no
/// [`MeetOfThree::MoreThanAPoint`] to answer with.
///
/// # Errors
///
/// [`Error::Undecidable`] when a product the exact arithmetic needs falls out of
/// `f64`'s range, and when the line all but touches the ring — a **gap** and not
/// a limit, and [`real_roots`](crate::real_roots) names the family of fixes. A
/// touch that lands on an `f64` is answered rather than refused, and a bend of
/// pipe's own seam is that case.
fn where_a_line_meets_a_ring(
    named: &'static str,
    line: &ALine,
    ring: &Torus,
    at: &[PredicatePoint],
) -> Result<WhereALineMeetsIt> {
    let quartic = how_a_ring_cuts_the_line(line, ring)
        .filter(|quartic| !quartic.iter().any(Term::lost_anything))
        .ok_or_else(|| an_exact_step_could_not_be_taken(named, at))?;
    // **The quartic route is the one place level 3 is not optional.** Its roots
    // are found by evaluating signs along the line, and a coefficient that is a
    // box has no sign to evaluate anywhere near a root — so a line that is a box
    // is turned away here rather than answered with four boxes nobody can order.
    let exactly: Option<Vec<Exact>> = quartic.iter().map(Term::exactly).collect();
    let quartic = exactly.ok_or(Error::WhereALineMeetsARingNeedsTheLineExactly)?;
    assert!(
        matches!(quartic[0].sign(), Decision::Pos),
        "a line's direction and a ring's axis both have length, so this is above zero"
    );
    let found = where_a_polynomial_is_zero(&quartic)
        .ok_or_else(|| an_exact_step_could_not_be_taken(named, at))?;

    let mut steps: SmallVec<[MeetsAt; 2]> = found.at.iter().copied().map(MeetsAt::Here).collect();
    steps.extend(found.or_maybe_at.iter().map(|unsure| MeetsAt::MaybeATouch {
        inside: unsure.inside,
        how_far_it_reaches: unsure.how_far_it_reaches,
    }));
    if steps.is_empty() {
        return Ok(WhereALineMeetsIt::Nowhere);
    }
    Ok(WhereALineMeetsIt::At(steps))
}

/// The quartic a ring cuts out of the line, highest power first, or `None` when
/// a product it needs falls out of `f64`'s range.
///
/// A ring's own sign expression is `(w·w)(x·x + R² − r²)² − 4R²‖x × w‖²` for
/// `x = p − centre`. Writing the line as `(V + s·D)/m` and multiplying the whole
/// of it by `m⁴` — positive, so no sign moves — turns that into
///
/// ```text
/// (w·w)·A(s)²  −  4R²m²·B(s)
/// A(s) = (V + sD)·(V + sD) + m²(R² − r²)
/// B(s) = ‖(V + sD) × w‖²
/// ```
///
/// with `A` and `B` quadratics in `s`, so the whole is a quartic whose every
/// coefficient is a polynomial in the numbers as stored. **Both radii are
/// squared against `m²` and neither carries a `w·w` of its own**, because both
/// sit inside a term the `(w·w)` out front already multiplies. That makes the
/// quartic degree two in `w` all through, so the ring these numbers name is the
/// one the caller's two radii describe whatever length the direction got stored
/// at — the same rule [`Torus`]'s own sign predicate is certified under, and the
/// same rule a pipe follows since §118.10, though a pipe has to be handed its
/// `w·w` by hand because nothing else multiplies its radius (§27.1).
fn how_a_ring_cuts_the_line(line: &ALine, ring: &Torus) -> Option<[Term; 5]> {
    let along = ring.axis().direction().components();
    let reach = the_line_offset_to(line, ring.centre().coordinates())?;
    let axis_squared = Term::Stored(Nearly::dot_of(along, along)?);
    let divisor_squared = line.divided_by.times(&line.divided_by)?;
    let ring_squared = Term::Stored(Nearly::product_of(ring.ring_radius(), ring.ring_radius())?);
    let hollow = ring_squared.minus(&Term::Stored(Nearly::product_of(
        ring.tube_radius(),
        ring.tube_radius(),
    )?));

    let bulge = [
        squared_length_of(&line.direction)?,
        twice(&dot_of(&reach, &line.direction)?)?,
        squared_length_of(&reach)?.plus(&divisor_squared.times(&hollow)?),
    ];
    let sideways = cross_of(&line.direction, along)?;
    let across = cross_of(&reach, along)?;
    let round = [
        squared_length_of(&sideways)?,
        twice(&dot_of(&across, &sideways)?)?,
        squared_length_of(&across)?,
    ];
    let four_ring_squared = twice(&twice(&ring_squared)?)?.times(&divisor_squared)?;

    Some([
        axis_squared.times(&bulge[0].times(&bulge[0])?)?,
        axis_squared.times(&twice(&bulge[0].times(&bulge[1])?)?)?,
        axis_squared
            .times(
                &bulge[1]
                    .times(&bulge[1])?
                    .plus(&twice(&bulge[0].times(&bulge[2])?)?),
            )?
            .minus(&four_ring_squared.times(&round[0])?),
        axis_squared
            .times(&twice(&bulge[1].times(&bulge[2])?)?)?
            .minus(&four_ring_squared.times(&round[1])?),
        axis_squared
            .times(&bulge[2].times(&bulge[2])?)?
            .minus(&four_ring_squared.times(&round[2])?),
    ])
}

/// Twice a number, or `None` when that runs off the top of the range.
fn twice(value: &Term) -> Option<Term> {
    value.times(&Term::of(2.0))
}

/// How far the quartic reaches over a box it turns in, as a question the cascade
/// can ask.
///
/// There is no exact level here and that is the refusal, not an omission: the
/// turning point is not an `f64`, so no exact evaluation reaches it. What level
/// 2 hands over is a box round the height of the turn, and a policy that will
/// not read a zero out of a box that straddles one is refused.
fn how_far_it_reaches_over(
    named: &'static str,
    reach: Interval,
) -> Question<impl Fn() -> Interval, impl Fn() -> Option<Exact>> {
    Question {
        named,
        enclosing: move || reach,
        exactly: move || None,
    }
}

/// `A·s² + 2B·s + C`, where `s` runs along the line.
///
/// Kept as three numbers and never as one polynomial, because the sign of the
/// discriminant and the size of the roots are read off them differently: the
/// sign goes through [`Exact::square_minus`], which slides both sides up the
/// range and hands back a sign with no size in it, and the roots go through the
/// enclosures of these three.
struct Quadratic {
    squared: Term,
    half_linear: Term,
    constant: Term,
}

impl Quadratic {
    /// `B² − A·C` with nothing slid anywhere, so it is a **value** as well as a
    /// sign — or `None` when a product it needs falls out of `f64`'s range.
    ///
    /// [`Exact::square_minus`] reaches further, because it slides both halves
    /// into the window where squaring is exact; what it hands back is a sign
    /// multiplied by a positive power of two, and reading a size off it is the
    /// mistake §32.1 names. This is the same number without the slide, and it is
    /// what the roots are worked out from.
    fn difference_of_the_two_halves(&self) -> Option<Term> {
        Some(
            self.half_linear
                .times(&self.half_linear)?
                .minus(&self.squared.times(&self.constant)?),
        )
    }
}

/// The quadratic a ball or a pipe cuts out of the line, or `None` when a product
/// it needs falls out of `f64`'s range.
///
/// # Panics
///
/// On a plane, which is not a quadric and is answered by
/// [`where_three_planes_meet`] sixteen lines above the only call, and on a
/// torus, which is not a quadric either and is taken by
/// [`where_a_line_meets_a_ring`] two lines after that.
///
/// The plane arm is unreachable by adjacency — same function, same value, no
/// branch between. **The torus arm is not**: one check covers three positions
/// only because a torus sorts after every other kind, which is a fact about
/// [`Surface::how_it_sorts_against`] in another file and nothing here makes
/// anyone re-read it.
/// `where_two_planes_and_a_ring_meet_is_answered_at_up_to_four_places`
/// (`crates/rcad-geom/tests/the_shapes_a_plane_cuts_a_torus_in.rs`) walks a
/// torus through all three positions, and two of them and three, so the claim is
/// demonstrated rather than argued.
fn how_a_quadric_cuts_the_line(line: &ALine, quadric: &Surface) -> Option<Quadratic> {
    let quadratic = match quadric {
        Surface::Sphere(ball) => how_a_ball_cuts_the_line(line, ball),
        Surface::Cylinder(pipe) => how_a_pipe_cuts_the_line(line, pipe),
        Surface::Cone(cone) => how_a_cone_cuts_the_line(line, cone),
        Surface::Plane(flat) => panic!("{flat:?} is flat, so three planes is the question"),
        Surface::Torus(ring) => panic!("{ring:?} is a quartic and is refused before this"),
    }?;
    let coefficients = [
        &quadratic.squared,
        &quadratic.half_linear,
        &quadratic.constant,
    ];
    (!coefficients
        .iter()
        .any(|term| term.lost_more_than_a_rounding()))
    .then_some(quadratic)
}

/// `‖V + s·D‖² − m²R²` where `V` is the line's point offset to the ball's
/// centre, a **degree ten** discriminant in the stored numbers.
///
/// `docs/decisions.md` §31.2 guessed about fourteen for this row and §34 records
/// what it is. Both halves of `B² − A·C` come to ten: `A` is `D·D` at four, `B`
/// is `V·D` at five and `C` is `V·V − m²R²` at six.
fn how_a_ball_cuts_the_line(line: &ALine, ball: &Sphere) -> Option<Quadratic> {
    let reach = the_line_offset_to(line, ball.centre().coordinates())?;
    Some(Quadratic {
        squared: squared_length_of(&line.direction)?,
        half_linear: dot_of(&reach, &line.direction)?,
        constant: squared_length_of(&reach)?.minus(&squared_radius_of(line, ball.radius())?),
    })
}

/// `‖(V + s·D) × w‖² − m²r²(w·w)` for the pipe's stored axis direction `w`, a
/// **degree fourteen** discriminant.
///
/// The radius is squared against `m²(w·w)` and not against `m²` alone, so that
/// these three numbers name the pipe of radius exactly `r` — which is what
/// [`SideOfSurface`](rcad_predicates::SideOfSurface) has certified for a pipe
/// since §118.10 made it homogeneous in `w`. A discriminant that left the `w·w`
/// out would be certified about the pipe of radius `r/‖w‖` instead, and the two
/// halves of a meet would be answering about surfaces a rounding apart (§27.1).
///
/// **The `w·w` does not move the degree.** `A` is `‖D × w‖²` at six, `B` is
/// `(D × w)·(V × w)` at seven and `C` is now eight on both of its terms where
/// the radius term used to be six, so `B² − A·C` is fourteen either way — and it
/// is only now that `A·C`'s two halves are the same degree as each other.
/// `how_many_f64_the_discriminant_of_a_line_and_a_quadric_takes_to_hold` below
/// measures what that costs in `f64` rather than arguing it.
fn how_a_pipe_cuts_the_line(line: &ALine, pipe: &Cylinder) -> Option<Quadratic> {
    let along = pipe.axis().direction().components();
    let reach = the_line_offset_to(line, pipe.axis().origin().coordinates())?;
    let sideways = cross_of(&line.direction, along)?;
    let across = cross_of(&reach, along)?;
    let axis_squared = Term::Stored(Nearly::dot_of(along, along)?);
    Some(Quadratic {
        squared: squared_length_of(&sideways)?,
        half_linear: dot_of(&sideways, &across)?,
        constant: squared_length_of(&across)?
            .minus(&squared_radius_of(line, pipe.radius())?.times(&axis_squared)?),
    })
}

/// `‖(V + s·D) × w‖² − slope²·((V + s·D)·w)²` for the cone's stored axis
/// direction `w`.
///
/// **Neither an `m²` nor a `w·w` correction, and that is the cone being easier
/// than the pipe rather than two corrections being forgotten.** A pipe's
/// constant term compares an area against `r²`, which is degree zero in both the
/// line's divisor and the stored direction, so it has to be given `m²(w·w)` to
/// match — see [`squared_radius_of`]. A cone has no radius: both halves here are
/// degree two in `V + s·D` and degree two in `w`, and the slope is a pure ratio,
/// so the divisor and the direction's length both factor out of the whole
/// expression and every sign read off it is untouched.
fn how_a_cone_cuts_the_line(line: &ALine, cone: &Cone) -> Option<Quadratic> {
    let along = cone.axis().direction().components();
    let up = along.map(Term::of);
    let reach = the_line_offset_to(line, cone.apex().coordinates())?;
    let sideways = cross_of(&line.direction, along)?;
    let across = cross_of(&reach, along)?;
    let slope_squared = Term::Stored(Nearly::product_of(cone.slope(), cone.slope())?);
    let direction_up = dot_of(&line.direction, &up)?;
    let reach_up = dot_of(&reach, &up)?;
    Some(Quadratic {
        squared: squared_length_of(&sideways)?
            .minus(&slope_squared.times(&direction_up.times(&direction_up)?)?),
        half_linear: dot_of(&sideways, &across)?
            .minus(&slope_squared.times(&direction_up.times(&reach_up)?)?),
        constant: squared_length_of(&across)?
            .minus(&slope_squared.times(&reach_up.times(&reach_up)?)?),
    })
}

/// The line's own point, moved so `place` is the origin, and still scaled by the
/// line's divisor.
fn the_line_offset_to(line: &ALine, place: [f64; 3]) -> Option<[Term; 3]> {
    let mut reach = [Term::of(0.0), Term::of(0.0), Term::of(0.0)];
    for axis in 0..3 {
        reach[axis] = line.through[axis].minus(&line.divided_by.times(&Term::of(place[axis]))?);
    }
    Some(reach)
}

/// `m²r²`, which is what a radius has to be compared against once the line has
/// been written with its divisor left in.
///
/// That is the whole of it for a ball, whose sign expression is
/// `‖p − c‖² − R²` and carries no direction. A pipe multiplies what comes back
/// by `w·w` on top, because its other side is `‖x × w‖²` — see
/// [`how_a_pipe_cuts_the_line`].
fn squared_radius_of(line: &ALine, radius: f64) -> Option<Term> {
    line.divided_by
        .times(&line.divided_by)?
        .times(&Term::Stored(Nearly::product_of(radius, radius)?))
}

/// Where the line meets the quadric: nowhere, at one step along it, at two, or
/// along the whole of it.
fn where_a_line_meets_a_quadric(
    named: &'static str,
    quadratic: &Quadratic,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<WhereALineMeetsIt> {
    // `A` is dimensionless — see the note in `where_three_planes_meet`.
    //
    // **`A` below zero is a real answer and a cone is what brings it.** A ball's
    // is `D·D` and a pipe's is `‖D × w‖²`, and a sum of squares is never
    // negative — which is what the arm here used to say. A cone's is
    // `‖D × w‖² − slope²(D·w)²`, negative for every line pointing further into
    // the cone than its own rulings do, which is most of the lines through it.
    // The roots are read off the same three numbers either way, and
    // [`the_two_places_along_it`] never divides by `A` except through Forsythe's
    // pairing, so nothing below cares about the sign.
    match ask(
        &the_sign_of(named, &quadratic.squared),
        Measures::Length,
        at,
        policy,
        log,
    )? {
        Decision::Pos | Decision::Neg => (),
        Decision::Zero => {
            return where_a_line_meets_a_quadric_it_runs_alongside(
                named, quadratic, at, policy, log,
            );
        }
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }

    match ask(
        &the_discriminant_of(named, quadratic),
        Measures::Area,
        at,
        policy,
        log,
    )? {
        Decision::Neg => Ok(WhereALineMeetsIt::Nowhere),
        Decision::Zero => Ok(WhereALineMeetsIt::At(smallvec![MeetsAt::Here(
            where_it_touches(quadratic)
        )])),
        Decision::Pos => {
            let [lower, upper] = the_two_places_along_it(quadratic);
            Ok(WhereALineMeetsIt::At(smallvec![
                MeetsAt::Here(lower),
                MeetsAt::Here(upper),
            ]))
        }
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// How far along the line it touches the quadric: `−B/A`, the one root a zero
/// discriminant leaves.
///
/// **That step exactly where it is an `f64`**, as a box of no width. A step with
/// width places the touching point as a box, and at a touch the questions a
/// fuse asks next are the ones whose true answer is zero — which way a wall
/// turns against the ball it touches is `orient3d` of three vectors in one
/// plane. No box with width round a zero decides that, however narrow; the
/// exact point does. Where `B` or `A` is not known exactly, or their quotient
/// is not an `f64`, the step is the box it always was.
fn where_it_touches(quadratic: &Quadratic) -> Interval {
    let exactly = quadratic
        .half_linear
        .exactly()
        .zip(quadratic.squared.exactly())
        .and_then(|(half_linear, squared)| {
            the_f64_quotient_of(&Exact::of(0.0).minus(&half_linear), &squared)
        });
    exactly.map_or_else(
        || {
            quotient_or_everything(
                -quadratic.half_linear.enclosure(),
                quadratic.squared.enclosure(),
            )
        },
        Interval::exactly,
    )
}

/// Where the line meets the quadric when the squared term is exactly zero, so
/// the equation is `2B·s + C = 0` and not a quadratic at all.
///
/// **A pipe and a cone reach this differently, and the difference is one root.**
/// A pipe's `A` is `‖D × w‖²`, so `A = 0` means the line runs along the axis and
/// then `B = (D × w)·(V × w)` is exactly zero too: the line lies on the wall or
/// misses it, and there is no single point either way. That was the whole of
/// this before there were cones. A cone's `A = 0` means the line runs parallel
/// to one of its rulings, and such a line crosses the cone at exactly **one**
/// point, with `B` nowhere near zero.
///
/// So the linear term is asked about first. For a ball and a pipe the answer is
/// always [`Decision::Zero`] and always certified — `B` is a dot product with an
/// exactly-zero vector — so this asks one more question than it used to and
/// gives every old answer back unchanged.
fn where_a_line_meets_a_quadric_it_runs_alongside(
    named: &'static str,
    quadratic: &Quadratic,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<WhereALineMeetsIt> {
    match ask(
        &the_sign_of(named, &quadratic.half_linear),
        Measures::Length,
        at,
        policy,
        log,
    )? {
        Decision::Pos | Decision::Neg => {
            let twice_the_linear = Interval::exactly(2.0) * quadratic.half_linear.enclosure();
            return Ok(WhereALineMeetsIt::At(smallvec![MeetsAt::Here(
                quotient_or_everything(-quadratic.constant.enclosure(), twice_the_linear)
            )]));
        }
        Decision::Zero => (),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
    match ask(
        &the_sign_of(named, &quadratic.constant),
        Measures::Area,
        at,
        policy,
        log,
    )? {
        Decision::Zero => Ok(WhereALineMeetsIt::MoreThanAPoint),
        Decision::Pos | Decision::Neg => Ok(WhereALineMeetsIt::Nowhere),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// How far along the line each of the two meeting points sits, smaller first.
///
/// **`(−B ± √Δ) / A` on its own is not good enough, and the case that shows it
/// is not exotic.** A line a hair off parallel to a pipe's axis has `A = ‖D × w‖²`
/// down at `1e-32`, and then the two terms of `−B ± √Δ` cancel for one of the
/// two roots and what is left is a rounding of `B` divided by that `A`. Measured
/// on a bar's own cap plane turned by a sixty-fourth: a box **fifteen hundred
/// times the shape**, for a corner that is perfectly well determined.
///
/// So the pair is worked out the way a quadratic solver has since Forsythe.
/// `q = −(B + sign(B)·√Δ)` adds two numbers of the same sign and cancels
/// nothing; one root is `q/A` and the other is `C/q`, because the two roots
/// multiply to `C/A`. Both forms are valid enclosures of the same number — they
/// are the same identity — so the narrower of the two is taken for each root and
/// there is no branch to get wrong when `q` runs through zero.
fn the_two_places_along_it(quadratic: &Quadratic) -> [Interval; 2] {
    // **Not read off `square_minus`**, which slid both sides up the range and
    // hands back a sign (§32.1).
    let root = root_enclosing(the_size_of_the_discriminant(quadratic));
    let squared = quadratic.squared.enclosure();
    let half_linear = quadratic.half_linear.enclosure();
    let plainly = |which: Interval| quotient_or_everything(-half_linear + which, squared);

    let leans_back = matches!(quadratic.half_linear.sign(), Decision::Neg);
    let steady = if leans_back {
        root - half_linear
    } else {
        -half_linear - root
    };
    let (well_placed, its_partner) = (
        quotient_or_everything(steady, squared),
        quotient_or_everything(quadratic.constant.enclosure(), steady),
    );
    let (lower, upper) = if leans_back {
        (its_partner, well_placed)
    } else {
        (well_placed, its_partner)
    };
    [
        the_tighter(plainly(-root), lower),
        the_tighter(plainly(root), upper),
    ]
}

/// The narrower of two enclosures of the same number.
fn the_tighter(one: Interval, other: Interval) -> Interval {
    if other.width() < one.width() {
        other
    } else {
        one
    }
}

/// The point `step` of the way along the line: exactly, where
/// [`the_point_exactly_at`] can name it, and a box round it everywhere else.
fn the_point_at(line: &ALine, step: Interval) -> PredicatePoint {
    if let Some(place) = the_point_exactly_at(line, step) {
        return PredicatePoint::Exact(place);
    }
    let divisor = line.divided_by.enclosure();
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        quotient_or_everything(
            line.through[axis].enclosure() + step * line.direction[axis].enclosure(),
            divisor,
        )
    }))
}

/// The point a step of no width names, where the line is known exactly and
/// each coordinate `(through + step·direction) / divided_by` is an `f64` — or
/// `None`.
///
/// A step of no width is one number and not a box round one, so the point it
/// names is one point; working it out in intervals would put back the width
/// [`where_it_touches`] kept out of the step.
fn the_point_exactly_at(line: &ALine, step: Interval) -> Option<[f64; 3]> {
    if step.lo() != step.hi() {
        return None;
    }
    let along = Term::of(step.lo());
    let divisor = line.divided_by.exactly()?;
    let mut place = [0.0; 3];
    for (axis, coordinate) in place.iter_mut().enumerate() {
        let over = line.through[axis].plus(&line.direction[axis].times(&along)?);
        *coordinate = the_f64_quotient_of(&over.exactly()?, &divisor)?;
    }
    Some(place)
}

/// `over / under`, where that is exactly an `f64` — or `None` where it is not,
/// or where a product the check takes would round.
///
/// **An answer comes back only where `under · q − over` is exactly zero**, so a
/// guess that misses costs an exact answer and never gives a wrong one.
///
/// The first guess divides the two numbers rounded, and each rounding can put
/// it a unit in the last place further off. One step of Newton's method on
/// `under · q − over`, whose value at the guess is worked out exactly, takes it
/// the rest of the way: where the quotient is an `f64` the step is a few units
/// long and wrong by a rounding of that, far less than the half unit that would
/// land it on a neighbour.
/// `an_f64_quotient_is_found_where_the_first_guess_lands_beside_it` counts the
/// quotients the first guess alone misses and the step does not.
///
/// # Panics
///
/// If `under` is zero. Both callers divide by a number already decided not to
/// be: a quadratic's `A` and a line's divisor.
fn the_f64_quotient_of(over: &Exact, under: &Exact) -> Option<f64> {
    assert!(
        matches!(under.sign(), Decision::Pos | Decision::Neg),
        "a quotient is only asked of a divisor decided not to be zero: {under:?}"
    );
    let divisor = under.nearest_f64();
    let missed_by = |guess: f64| Some(under.times(&Exact::of(guess))?.minus(over));
    let guess = over.nearest_f64() / divisor;
    let settled = guess - missed_by(guess)?.nearest_f64() / divisor;
    match missed_by(settled)?.sign() {
        Decision::Zero => Some(settled),
        Decision::Pos | Decision::Neg | Decision::Undecidable { .. } => None,
        other => unreachable!("an exact sign is a sign or a refusal: {other:?}"),
    }
}

/// The sign of a number, as a question the cascade can ask.
///
/// A number that is only a box has **no exact level**, and that is the box being
/// all there ever was rather than a step missed: nothing recovers a number a
/// caller never had (§2.5).
fn the_sign_of<'a>(
    named: &'static str,
    value: &'a Term,
) -> Question<impl Fn() -> Interval + 'a, impl Fn() -> Option<Exact> + 'a> {
    Question {
        named,
        enclosing: move || value.enclosure(),
        exactly: move || value.exactly(),
    }
}

/// `B² − A·C`, as a question the cascade can ask.
fn the_discriminant_of<'a>(
    named: &'static str,
    quadratic: &'a Quadratic,
) -> Question<impl Fn() -> Interval + 'a, impl Fn() -> Option<Exact> + 'a> {
    Question {
        named,
        enclosing: move || the_size_of_the_discriminant(quadratic),
        exactly: move || {
            quadratic
                .half_linear
                .exactly()?
                .square_minus(&quadratic.squared.times(&quadratic.constant)?.exactly()?)
        },
    }
}

/// How big the discriminant is, which is a different number from what says which
/// way it points.
///
/// **Summed through the expansion and not in intervals**, and the difference is
/// a box a rounding wide against a box two hundred times the shape. A line
/// nearly tangent to a quadric has `B²` and `A·C` two large numbers that cancel
/// to nothing, and an interval evaluation of the difference reaches out to the
/// rounding of the larger — so `√Δ` carries it, and so does every point built on
/// it. Summed exactly the difference is known to full relative accuracy, which
/// is §29's lesson and §33.4's, one question up.
///
/// The fallback is the interval form, for the rare case where a product the
/// exact route needs falls out of range. It is honest and it is wide.
fn the_size_of_the_discriminant(quadratic: &Quadratic) -> Interval {
    if let Some(exactly) = quadratic.difference_of_the_two_halves()
        && !exactly.lost_more_than_a_rounding()
    {
        return exactly.enclosure();
    }
    let half_linear = quadratic.half_linear.enclosure();
    half_linear * half_linear - quadratic.squared.enclosure() * quadratic.constant.enclosure()
}

/// `first · second − third · fourth`, exactly, or `None` when either product
/// would round.
fn cross_of_two_pairs(first: f64, second: f64, third: f64, fourth: f64) -> Option<Exact> {
    Some(Exact::product_of(first, second)?.minus(&Exact::product_of(third, fourth)?))
}

/// `u × v` for two triples of stored numbers, exactly.
fn exact_cross_of(u: [f64; 3], v: [f64; 3]) -> Option<[Exact; 3]> {
    let mut across = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for (axis, component) in across.iter_mut().enumerate() {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        *component = cross_of_two_pairs(u[next], v[after], u[after], v[next])?;
    }
    Some(across)
}

/// The refusal for a step of exact arithmetic that could not be taken at all.
///
/// A **gap** and never a limit, and the same one every discriminant in
/// [`intersect`](crate::intersect) has: a product whose low half would land
/// among the subnormals, or one that overflows. The same three surfaces written
/// somewhere else on the number line are answered.
fn an_exact_step_could_not_be_taken(named: &'static str, at: &[PredicatePoint]) -> Error {
    let mut records = ProvenanceLog::new();
    records.push(Record::new(
        named,
        at,
        Level::Exact,
        f64::INFINITY,
        Measures::Area,
    ));
    Error::Undecidable { records }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Axis, Point, Vector};

    fn flat(normal: [f64; 3], through: [f64; 3]) -> Surface {
        Surface::Plane(
            Plane::from_point_and_normal(
                Point::new(through[0], through[1], through[2]),
                Vector::new(normal[0], normal[1], normal[2]),
            )
            .expect("a normal with length"),
        )
    }

    fn meet(surfaces: [&Surface; 3]) -> Result<MeetOfThree> {
        meet_of_three_surfaces(surfaces, &Policy::certified(), &mut ProvenanceLog::new())
    }

    fn middles(answer: &MeetOfThree) -> Vec<[f64; 3]> {
        let MeetOfThree::At(points) = answer else {
            panic!("expected points, got {answer:?}");
        };
        points
            .iter()
            .map(|point| point.enclosure().map(|side| (side.lo() + side.hi()) / 2.0))
            .collect()
    }

    #[test]
    fn three_planes_square_to_the_axes_meet_where_they_say() {
        let answer = meet([
            &flat([1.0, 0.0, 0.0], [2.0, 0.0, 0.0]),
            &flat([0.0, 1.0, 0.0], [0.0, 3.0, 0.0]),
            &flat([0.0, 0.0, 1.0], [0.0, 0.0, 4.0]),
        ])
        .expect("three planes square to each other meet");
        assert_eq!(middles(&answer), vec![[2.0, 3.0, 4.0]]);
    }

    #[test]
    fn two_planes_through_a_balls_centre_meet_it_at_both_poles() {
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
        let answer = meet([
            &flat([1.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            &flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
            &ball,
        ])
        .expect("a line through the centre cuts a ball twice");
        assert_eq!(middles(&answer), vec![[0.0, 0.0, -5.0], [0.0, 0.0, 5.0]]);
    }

    #[test]
    fn two_planes_meeting_a_pipe_land_on_its_wall() {
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius"));
        let answer = meet([
            &flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
            &flat([0.0, 0.0, 1.0], [0.0, 0.0, 7.0]),
            &pipe,
        ])
        .expect("a line across a pipe cuts it twice");
        assert_eq!(middles(&answer), vec![[-3.0, 0.0, 7.0], [3.0, 0.0, 7.0]]);
    }

    #[test]
    fn a_line_that_grazes_a_ball_meets_it_once_and_one_that_misses_it_not_at_all() {
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
        let grazing = meet([
            &flat([1.0, 0.0, 0.0], [5.0, 0.0, 0.0]),
            &flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
            &ball,
        ])
        .expect("a tangent line still meets the ball");
        assert_eq!(middles(&grazing), vec![[5.0, 0.0, 0.0]]);

        assert_eq!(
            meet([
                &flat([1.0, 0.0, 0.0], [6.0, 0.0, 0.0]),
                &flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
                &ball,
            ]),
            Ok(MeetOfThree::Nowhere)
        );
    }

    /// A line that touches a ball where every coordinate is an `f64` comes
    /// back as that point, not as a box round it: straight down the lattice,
    /// and down the `z` axis between two walls whose normals are not on it.
    ///
    /// The second is the corner a ball touching a prism's upright edge makes,
    /// and as a box with width in `z` it stopped every fuse of the two under a
    /// certified policy: `crates/rcad-topo/tests/a_ball_touching_a_prisms_edge_exactly.rs`.
    #[test]
    fn a_line_that_touches_a_ball_at_a_point_f64_holds_touches_it_exactly_there() {
        let ball = |centre: [f64; 3], radius: f64| {
            Surface::Sphere(
                Sphere::new(Point::new(centre[0], centre[1], centre[2]), radius).expect("a radius"),
            )
        };
        assert_eq!(
            meet([
                &flat([1.0, 0.0, 0.0], [5.0, 0.0, 0.0]),
                &flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
                &ball([0.0; 3], 5.0),
            ]),
            Ok(MeetOfThree::At(smallvec![PredicatePoint::Exact([
                5.0, 0.0, 0.0
            ])]))
        );
        assert_eq!(
            meet([
                &flat([3.0, -4.0, 0.0], [0.0; 3]),
                &flat([-1.0, -1.0, 0.0], [0.0; 3]),
                &ball([1.0, 0.0, 16.0], 1.0),
            ]),
            Ok(MeetOfThree::At(smallvec![PredicatePoint::Exact([
                0.0, 0.0, 16.0
            ])]))
        );
    }

    /// Where `over / under` is an `f64`, [`the_f64_quotient_of`] finds it —
    /// including where the first guess, the two rounded and divided, lands on
    /// a neighbour of it. And where it is not an `f64`, nothing comes back.
    ///
    /// `under` is a number and a part of a unit in its last place, so rounding
    /// it moves it, and that is one of the three roundings that can move the
    /// guess. The counts say what each way of guessing does with the same
    /// quotients. Measured: of 100 000, the first guess alone missed 37 726,
    /// the first guess and its two neighbours missed 137, and the Newton step
    /// missed none.
    #[test]
    fn an_f64_quotient_is_found_where_the_first_guess_lands_beside_it() {
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        let mut one_in_one_to_two = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            f64::from_bits((0x3ff << 52) | (state >> 12))
        };
        let (mut the_guess_missed, mut a_neighbour_missed) = (0, 0);
        const TRIES: usize = 100_000;
        for attempt in 0..TRIES {
            let quotient = if attempt % 2 == 0 {
                one_in_one_to_two()
            } else {
                -one_in_one_to_two()
            };
            let under = Exact::of(one_in_one_to_two())
                .plus(&Exact::of(one_in_one_to_two() * 2f64.powi(-54)));
            let over = under
                .times(&Exact::of(quotient))
                .expect("nothing here is near the ends of the range");
            let guess = over.nearest_f64() / under.nearest_f64();
            if guess != quotient {
                the_guess_missed += 1;
            }
            if ![guess, guess.next_up(), guess.next_down()].contains(&quotient) {
                a_neighbour_missed += 1;
            }
            assert_eq!(
                the_f64_quotient_of(&over, &under),
                Some(quotient),
                "{over:?} / {under:?}"
            );
        }
        println!(
            "of {TRIES} quotients that are an f64, the first guess missed {the_guess_missed}, \
             the guess and its two neighbours missed {a_neighbour_missed}, the Newton step none"
        );
        assert!(
            the_guess_missed > 0,
            "no first guess missed, so this search says nothing about the step"
        );

        assert_eq!(the_f64_quotient_of(&Exact::of(1.0), &Exact::of(3.0)), None);
        for under in [3.0, -3.0] {
            assert_eq!(
                the_f64_quotient_of(&Exact::of(0.0), &Exact::of(under)).map(f64::to_bits),
                Some(0.0f64.to_bits()),
                "a zero over {under} is the zero every other zero is, and not -0.0"
            );
        }
    }

    #[test]
    fn a_line_down_a_pipes_own_wall_is_more_than_a_point() {
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius"));
        assert_eq!(
            meet([
                &flat([1.0, 0.0, 0.0], [3.0, 0.0, 0.0]),
                &flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
                &pipe,
            ]),
            Ok(MeetOfThree::MoreThanAPoint)
        );
    }

    #[test]
    fn a_line_that_runs_beside_a_pipe_meets_it_nowhere() {
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius"));
        assert_eq!(
            meet([
                &flat([1.0, 0.0, 0.0], [4.0, 0.0, 0.0]),
                &flat([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
                &pipe,
            ]),
            Ok(MeetOfThree::Nowhere)
        );
    }

    /// Two faces on one plane with a curved surface across them share the whole
    /// of what the plane and the curve share, and that is a curve rather than a
    /// corner (§31.3's coincident-face case).
    #[test]
    fn two_faces_on_one_plane_and_a_ball_share_a_whole_circle() {
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
        let wall = flat([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]);
        assert_eq!(meet([&wall, &wall, &ball]), Ok(MeetOfThree::MoreThanAPoint));

        let elsewhere = flat([0.0, 0.0, 1.0], [0.0, 0.0, 9.0]);
        assert_eq!(meet([&wall, &elsewhere, &ball]), Ok(MeetOfThree::Nowhere));
    }

    /// Three planes that do meet at a point, and meet it past the top of the
    /// range, come back as a refusal rather than as a box that says nothing.
    ///
    /// Cramer's rule divides one interval determinant by another and checks
    /// neither for overflow, so `Plane::meet_of_three` hands back `Ok` holding a
    /// box whose every side runs the whole number line — which a caller would
    /// read as a place. The two planes below cross in a line at the top of the
    /// range and the third leans across it by about `1e-16`, so the meet is at
    /// about `1e315` and no `f64` holds it.
    ///
    /// A **limit**, and §34.10 filed it as a gap. The second half below writes
    /// the same three planes lower down and they are placed — but that is a
    /// different triple with a different answer, and *this* triple's answer is a
    /// point at about `1e315` that no `f64` names. §16.10's two words go by
    /// whether the answer exists, and here it does not. The refusal is right and
    /// there is nothing to fix. §39.
    #[test]
    fn three_planes_meeting_past_the_top_of_the_range_are_refused_rather_than_placed_nowhere() {
        let leaning = [1.0, 1.0, 2f64.powi(-50)];
        let triple = |reach: f64| {
            [
                flat([1.0, 0.0, 0.0], [reach, 0.0, 0.0]),
                flat([0.0, 1.0, 0.0], [0.0, reach, 0.0]),
                flat(leaning, [0.0, 0.0, 0.0]),
            ]
        };

        let far = triple(1e300);
        let answer = meet([&far[0], &far[1], &far[2]]);
        assert!(
            matches!(
                answer,
                Err(Error::Predicate(_)) | Err(Error::Undecidable { .. })
            ),
            "three planes meeting past the top of the range came back as {answer:?}"
        );

        // The same three planes written lower down are placed, so what ran out
        // is the range and not the geometry.
        let near = triple(1.0);
        let placed = meet([&near[0], &near[1], &near[2]]).expect("these three meet at a point");
        let MeetOfThree::At(points) = &placed else {
            panic!("expected one point, got {placed:?}");
        };
        assert_eq!(points.len(), 1);
        assert!(points[0].uncertainty().is_finite());
    }

    /// A floor and two walls a millionth off parallel: Cramer's box round
    /// where they meet is about `1e-10` wide, and one checked step boxes the
    /// same meet to a few ulps (§281.2, §345). The narrowed box is checked to
    /// hold the meet with nothing rounded, from the stored numbers: along each
    /// axis the meet is `D / det`, so it is inside `[lo, hi]` when
    /// `D − lo·det` and `hi·det − D` both have the sign of `det`.
    #[test]
    fn a_meet_of_two_walls_nearly_parallel_is_boxed_to_a_few_ulps_and_still_holds_it() {
        for lean in [1e-6, 1.5e-6, 3e-6] {
            let floor = flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]);
            let wall = flat([0.6, 0.0, 0.8], [5.0, 0.0, 0.0]);
            let leaning = flat([0.6, lean, 0.8], [5.0, 0.0, 0.0]);
            let three = [&floor, &wall, &leaning];
            let by_cramer = meet(three).expect("three planes meet");
            let MeetOfThree::At(narrowed) = by_cramer.boxed_tight(three) else {
                panic!("three planes a millionth off parallel meet at a point");
            };
            let MeetOfThree::At(by_cramer) = by_cramer else {
                panic!("three planes a millionth off parallel meet at a point");
            };
            let (wide, tight) = (by_cramer[0].enclosure(), narrowed[0].enclosure());
            for axis in 0..3 {
                assert!(
                    wide[axis].lo() <= tight[axis].lo() && tight[axis].hi() <= wide[axis].hi(),
                    "leaning {lean}: the narrowed box {tight:?} is not inside Cramer's {wide:?}"
                );
            }
            assert!(
                by_cramer[0].uncertainty() > 1e-12,
                "leaning {lean}: Cramer's box is already narrow, {wide:?}"
            );
            assert!(
                narrowed[0].uncertainty() < 1e-14,
                "leaning {lean}: one step left the box {tight:?}"
            );
            let planes = in_a_fixed_order(three).map(|surface| {
                let Surface::Plane(flat) = surface else {
                    unreachable!("all three are planes")
                };
                flat
            });
            assert!(
                (0..3).all(|axis| the_meet_is_inside(&planes, axis, tight[axis])),
                "leaning {lean}: the narrowed box {tight:?} does not hold the meet"
            );
            let just_past = Interval::new(tight[0].hi() + 1e-12, tight[0].hi() + 2e-12)
                .expect("a box past the narrowed one");
            assert!(
                !the_meet_is_inside(&planes, 0, just_past),
                "leaning {lean}: the exact check holds the meet in a box past it, {just_past:?}"
            );
        }
    }

    /// Whether the one point three planes meet at lies inside `side` along
    /// `axis`, decided exactly.
    fn the_meet_is_inside(planes: &[Plane; 3], axis: usize, side: Interval) -> bool {
        let normals = planes.map(|plane| plane.normal().components());
        let across = [
            exact_cross_of(normals[1], normals[2]).expect("an exact cross product"),
            exact_cross_of(normals[2], normals[0]).expect("an exact cross product"),
            exact_cross_of(normals[0], normals[1]).expect("an exact cross product"),
        ];
        let determinant = Exact::dot_of_two_exact(&Exact::of_each(normals[0]), &across[0])
            .expect("an exact determinant");
        let numerator = (0..3).fold(Exact::of(0.0), |so_far, row| {
            so_far.plus(
                &Exact::of(planes[row].offset())
                    .times(&across[row][axis])
                    .expect("an exact product"),
            )
        });
        let above_the_bottom = numerator.minus(
            &Exact::of(side.lo())
                .times(&determinant)
                .expect("an exact product"),
        );
        let below_the_top = Exact::of(side.hi())
            .times(&determinant)
            .expect("an exact product")
            .minus(&numerator);
        let agrees = |value: &Exact| match (value.sign(), determinant.sign()) {
            (Decision::Zero, _) => true,
            (Decision::Pos, Decision::Pos) | (Decision::Neg, Decision::Neg) => true,
            (Decision::Pos | Decision::Neg, Decision::Pos | Decision::Neg) => false,
            other => panic!("an exact sign that is not a sign: {other:?}"),
        };
        agrees(&above_the_bottom) && agrees(&below_the_top)
    }

    /// A bar's bottom cap, `z = 0`, a block's face turned by `−π/4` about `x` to
    /// lean along `z = y`, and the bar's wall, `x² + y² = 25`: the meet near
    /// the rim start `(5, 0, 0)` that §284 could not order round the cap
    /// circle. The quadratic along the two planes' line boxes it `±3` ulps of
    /// `5` along `x`, and one checked step onto the wall boxes it one ulp
    /// wide. Each side of the narrowed box is checked to hold the meet with
    /// nothing rounded, from the stored numbers. On `z = 0` the turned face
    /// `n·p = d` puts the meet at `y = d / n_y`, and the wall then puts it at
    /// `x² = 25 − y²` with `x` above zero, so `x` lies in `[lo, hi]` when
    /// `25 n_y² − d²` lies between `lo² n_y²` and `hi² n_y²`.
    #[test]
    fn a_meet_of_two_planes_and_a_pipe_is_boxed_to_one_ulp_by_one_step_and_still_holds_it() {
        let lean = crate::Isometry::rotation(&Axis::x(), -core::f64::consts::FRAC_PI_4)
            .expect("a finite turn")
            .compose(
                &crate::Isometry::translation(Vector::new(0.0, 0.0, -20.0))
                    .expect("a finite shift"),
            );
        let cap = flat([0.0, 0.0, -1.0], [0.0, 0.0, 0.0]);
        let turned = flat([0.0, -1.0, 0.0], [0.0, 0.0, 0.0]).transformed(&lean);
        let wall = Surface::Cylinder(Cylinder::new(Axis::z(), 5.0).expect("a radius"));
        let three = [&cap, &turned, &wall];
        let as_placed = meet(three).expect("two planes and a pipe meet");
        let MeetOfThree::At(narrowed) = as_placed.boxed_tight(three) else {
            panic!("the turned face crosses the cap circle at two points");
        };
        let MeetOfThree::At(as_placed) = as_placed else {
            panic!("the turned face crosses the cap circle at two points");
        };
        let near = (0..as_placed.len())
            .find(|&which| as_placed[which].enclosure()[0].lo() > 0.0)
            .expect("one of the two is near (5, 0, 0)");
        let (wide, tight) = (as_placed[near].enclosure(), narrowed[near].enclosure());
        for axis in 0..3 {
            assert!(
                wide[axis].lo() <= tight[axis].lo() && tight[axis].hi() <= wide[axis].hi(),
                "the narrowed box {tight:?} is not inside the quadratic's {wide:?}"
            );
        }
        let ulp = 5.0_f64.next_up() - 5.0;
        assert!(
            wide[0].width() >= 4.0 * ulp,
            "the quadratic's box is already narrow along x, {wide:?}"
        );
        assert!(
            tight[0].width() <= ulp,
            "one step left the box {tight:?} wider than an ulp along x"
        );

        let Surface::Plane(turned) = turned else {
            unreachable!("a turned plane is a plane")
        };
        let [_, n_y, n_z] = turned.normal().components();
        assert!(n_z != 0.0 && n_y < 0.0, "the turned face leans along z = y");
        let offset = turned.offset();
        let exactly =
            |one: f64, other: f64| Exact::product_of(one, other).expect("an exact product");
        let at_or_above_zero =
            |value: Exact| matches!(value.sign(), Decision::Pos | Decision::Zero);
        let inside_along_y = at_or_above_zero(
            Exact::of(tight[1].lo())
                .times(&Exact::of(n_y))
                .expect("an exact product")
                .minus(&Exact::of(offset)),
        ) && at_or_above_zero(
            Exact::of(offset).minus(
                &Exact::of(tight[1].hi())
                    .times(&Exact::of(n_y))
                    .expect("an exact product"),
            ),
        );
        assert!(
            inside_along_y,
            "the narrowed box {tight:?} misses the meet's y"
        );
        let squared_x_times = exactly(5.0, 5.0)
            .times(&exactly(n_y, n_y))
            .expect("an exact product")
            .minus(&exactly(offset, offset));
        let past = |side: f64| {
            exactly(side, side)
                .times(&exactly(n_y, n_y))
                .expect("an exact product")
        };
        let holds_x = |lo: f64, hi: f64| {
            lo > 0.0
                && at_or_above_zero(squared_x_times.minus(&past(lo)))
                && at_or_above_zero(past(hi).minus(&squared_x_times))
        };
        assert!(
            holds_x(tight[0].lo(), tight[0].hi()),
            "the narrowed box {tight:?} misses the meet's x"
        );
        assert!(
            !holds_x(5.0_f64.next_up(), 5.0_f64.next_up().next_up()),
            "the exact check holds the meet's x in a box one ulp past it"
        );
        println!("as placed {wide:?}, narrowed {tight:?}");
    }

    /// The ball sits off the bar's axis, so the floor cuts the two in circles
    /// about two different centres, which cross at `(0, ±3, 0)`. About one
    /// centre they would be a circle of radius five and one of three, and
    /// [`what_a_plane_square_to_their_one_axis_leaves`] answers that.
    #[test]
    fn three_surfaces_with_only_one_flat_one_are_refused_by_name() {
        let ball = Surface::Sphere(Sphere::new(Point::new(4.0, 0.0, 0.0), 5.0).expect("a radius"));
        let pipe = Surface::Cylinder(Cylinder::new(Axis::z(), 3.0).expect("a radius"));
        assert_eq!(
            meet([&flat([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]), &ball, &pipe]),
            Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
        );
    }

    /// What the degree of the discriminant costs in `f64`, measured.
    ///
    /// §31.2 row 4 guesses "about 14" and §34 records what it is: **ten** for
    /// two planes and a ball, **fourteen** for two planes and a pipe. A degree
    /// is a fact about the expression and is derived term by term where each
    /// half is built; what a degree does not say is how many `f64` the exact
    /// value takes to hold, and that is the number a caller pays.
    ///
    /// Measured here so §29's question — how wide an expansion this arithmetic
    /// asks `Exact` to carry — has an answer for the row above the ones §29
    /// measured. The ceiling is a **watch and not a bound**: nothing derives it,
    /// and what it is there to catch is the discriminant's expansion growing by
    /// an order of magnitude without anybody noticing.
    #[test]
    fn how_many_f64_the_discriminant_of_a_line_and_a_quadric_takes_to_hold() {
        let turn = crate::Isometry::rotation(
            &Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("a direction"),
            0.4,
        )
        .expect("a finite angle");
        let ball = Surface::Sphere(Sphere::new(Point::new(1.0, 2.0, 0.0), 5.0).expect("a radius"));
        let pipe = Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::new(0.0, 1.0, 0.0), Vector::new(1.0, 3.0, 5.0))
                    .expect("a direction"),
                3.0,
            )
            .expect("a radius"),
        );
        let one = flat([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let other = flat([0.0, 1.0, 3.0], [0.0, 1.0, 0.0]);

        let mut widest = 0usize;
        for turned in [false, true] {
            let placed = |surface: &Surface| {
                if turned {
                    surface.transformed(&turn)
                } else {
                    *surface
                }
            };
            let (Surface::Plane(first), Surface::Plane(second)) = (placed(&one), placed(&other))
            else {
                unreachable!("both of these are flat");
            };
            let line = the_line_two_planes_cross_in(
                &first,
                &second,
                &[PredicatePoint::from([0.0; 3]); 2],
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("these two planes are not parallel")
            .expect("and they cross");

            for (what, quadric) in [("a ball", placed(&ball)), ("a pipe", placed(&pipe))] {
                let quadratic =
                    how_a_quadric_cuts_the_line(&line, &quadric).expect("all of this fits");
                // Two planes hand over stored `f64`, so every one of these is
                // exact and the `expect`s below are the measurement being of
                // the thing it says it is measuring.
                let exactly = |term: &Term| term.exactly().expect("two planes give exact numbers");
                let discriminant = exactly(
                    &quadratic
                        .difference_of_the_two_halves()
                        .expect("all of this fits"),
                );
                widest = widest.max(discriminant.entries());
                println!(
                    "{what}{}: A {} entries, B {}, C {}, B²−AC {} entries spanning {} binary orders",
                    if turned {
                        ", turned"
                    } else {
                        ", on the lattice"
                    },
                    exactly(&quadratic.squared).entries(),
                    exactly(&quadratic.half_linear).entries(),
                    exactly(&quadratic.constant).entries(),
                    discriminant.entries(),
                    discriminant.spread(),
                );
            }
        }
        assert!(
            widest < MOST_F64_A_DISCRIMINANT_HAS_BEEN_SEEN_TO_NEED,
            "the discriminant took {widest} entries to hold"
        );
    }

    /// A ceiling on how many `f64` the discriminant of a line and a quadric has
    /// been seen to need.
    ///
    /// **A watch and not a published bound.** `Exact::times` grows a product to
    /// at most twice the two lengths multiplied, which for a degree-fourteen
    /// discriminant is a number with five zeroes on it; what keeps the real
    /// figure two orders below that is `expansion_sum` dropping every roundoff
    /// that comes out exactly zero. Nothing derives the real figure, so this is
    /// set an order of magnitude above what is measured and its job is to notice
    /// a change of shape rather than to certify one.
    ///
    /// Measured on the four rows above: five entries for a ball on the lattice,
    /// nineteen for a pipe on the lattice, thirty-six and fifty-seven turned.
    /// The turned pipe's discriminant spans **761 binary orders**, which is
    /// inside the 995 [`Exact::square_minus`]'s window leaves and not by much —
    /// so the row above this one is the row that runs out first if the degree
    /// ever goes up again.
    const MOST_F64_A_DISCRIMINANT_HAS_BEEN_SEEN_TO_NEED: usize = 512;
}
