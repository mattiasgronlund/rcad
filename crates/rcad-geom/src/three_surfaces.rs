//! Where three surfaces meet.
//!
//! [`intersect`](crate::intersect) names the curve two surfaces cross in; this
//! names the **points** a third surface cuts out of that curve, which is what a
//! corner of an arrangement is (`docs/decisions.md` §31.2 rows 4 and 5).
//!
//! # Two of the three have to be flat, and that is a gap rather than a limit
//!
//! Two planes cross in a line, and a line cut by a quadric is a quadratic in one
//! unknown whose discriminant is a polynomial in the stored numbers. One plane
//! and two quadrics is a quartic and three quadrics is a degree-eight system;
//! both have answers, and reaching them takes an algebraic solver rather than a
//! discriminant. So this refuses by name (§16.10, §31.2), and
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
    Decision, Interval, Level, Measures, Point as PredicatePoint, Policy, ProvenanceLog, Record,
};
use smallvec::{SmallVec, smallvec};

use crate::enclosure::{quotient_or_everything, root_enclosing};
use crate::exact::Exact;
use crate::intersect::{Question, ask, intersect, squared_length_of_a_cross_product};
use crate::real_roots::{Maybe, where_a_polynomial_is_zero};
use crate::{
    Cone, Cylinder, Error, Plane, PlaneAgreement, Result, Sphere, Surface, SurfaceMeet, Torus,
};

/// The name the three-surface meet goes by in provenance records.
pub const WHERE_THREE_SURFACES_MEET: &str = "where three surfaces meet";

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
    /// Each is a box the true meeting point is somewhere inside, because
    /// solving a quadratic divides and a quotient of two ranges almost never
    /// lands on an `f64`. That width is the point of it (§2.5).
    At(SmallVec<[PredicatePoint; 2]>),
    /// They share a whole curve or a whole surface, so there is no corner here.
    ///
    /// Two faces on one plane with a third surface across them, or a plane pair
    /// whose line runs down a pipe's own wall.
    MoreThanAPoint,
}

/// Where three surfaces meet.
///
/// # Errors
///
/// [`Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat`] when fewer than two of
/// the three are planes.
///
/// [`Error::Undecidable`] when a sign the answer depends on could not be
/// settled, or when a product the exact arithmetic needs falls out of `f64`'s
/// range. Both carry the records.
///
/// [`Error::Predicate`] wrapping
/// [`PlanesDoNotMeetInAPoint`](rcad_predicates::Error::PlanesDoNotMeetInAPoint)
/// when three planes do meet at one point and neither route below can put a box
/// round it.
pub fn meet_of_three_surfaces(
    surfaces: [&Surface; 3],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    let [first, second, third] = in_a_fixed_order(surfaces);
    let (Surface::Plane(one), Surface::Plane(other)) = (&first, &second) else {
        return Err(Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat);
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
    if let Surface::Torus(ring) = &third {
        return where_a_line_meets_a_ring(&line, ring, &at, policy, log);
    }
    let quadratic = how_a_quadric_cuts_the_line(&line, &third)
        .ok_or_else(|| an_exact_step_could_not_be_taken(&at))?;
    where_a_line_meets_a_quadric(&line, &quadratic, &at, policy, log)
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
    [first, second, third].map(|surface| {
        PredicatePoint::from(match surface {
            Surface::Plane(flat) => flat.origin().coordinates(),
            Surface::Sphere(ball) => ball.centre().coordinates(),
            Surface::Cylinder(pipe) => pipe.axis().origin().coordinates(),
            Surface::Cone(cone) => cone.apex().coordinates(),
            Surface::Torus(ring) => ring.centre().coordinates(),
        })
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
        .unwrap_or_else(|| an_exact_step_could_not_be_taken(&at));

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

    let normal = last.normal().components();
    let (Some(along), Some(across)) = (
        Exact::dot_of_two_exact(&line.direction, &Exact::of_each(normal)),
        Exact::dot_of_two_exact(&line.through, &Exact::of_each(normal)).and_then(|reach| {
            Some(reach.minus(&line.divided_by.times(&Exact::of(last.offset()))?))
        }),
    ) else {
        return Err(an_exact_step_could_not_be_taken(&at));
    };

    // `along` is the determinant of the three normals, which is dimensionless.
    // `Measures` has no arm for that; see the note in
    // `intersect::where_a_plane_and_a_pipe_cross` for why `Length` is the one
    // whose conversion is the identity, and §28 for what should replace it.
    match ask(&the_sign_of(&along), Measures::Length, &at, policy, log)? {
        Decision::Pos | Decision::Neg => {
            let step = quotient_or_everything(-across.enclosure(), along.enclosure());
            let placed = the_point_at(&line, step);
            // Three planes leaning at a shrinking angle meet further and further
            // away, and past the top of `f64`'s range there is nothing left for
            // a box to be around — so this route runs out where Cramer's did,
            // and what is passed on is the first refusal rather than a second
            // one saying the same thing.
            if !placed.uncertainty().is_finite() {
                return Err(refused);
            }
            Ok(MeetOfThree::At(smallvec![placed]))
        }
        Decision::Zero => match ask(&the_sign_of(&across), Measures::Length, &at, policy, log)? {
            Decision::Zero => Ok(MeetOfThree::MoreThanAPoint),
            Decision::Pos | Decision::Neg => Ok(MeetOfThree::Nowhere),
            other => unreachable!("a sign question answers with a sign: {other:?}"),
        },
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
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
            match intersect(&Surface::Plane(*one), third, policy, log)? {
                SurfaceMeet::Nothing => Ok(MeetOfThree::Nowhere),
                SurfaceMeet::TouchAtOnePoint { at } => Ok(MeetOfThree::At(smallvec![at])),
                SurfaceMeet::Curves(_) | SurfaceMeet::SameSurface => {
                    Ok(MeetOfThree::MoreThanAPoint)
                }
            }
        }
        PlaneAgreement::CannotTell => Err(an_exact_step_could_not_be_taken(at)),
    }
}

/// The line where two planes cross, or `None` when they are parallel.
///
/// Written as `(through + s · direction) / divided_by`, with every one of those
/// an exact expansion in the stored coefficients. **The point on the line is the
/// one with a zero coordinate, not the foot of the perpendicular from the
/// origin**, and that choice is what keeps the degrees down: the perpendicular
/// foot needs `‖n₁ × n₂‖²` underneath it and carries four more degrees into
/// everything built on it.
struct WhereTwoPlanesCross {
    through: [Exact; 3],
    direction: [Exact; 3],
    divided_by: Exact,
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
) -> Result<Option<WhereTwoPlanesCross>> {
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
        let mut through = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
        through[across] = here;
        through[and] = there;
        return Ok(Some(WhereTwoPlanesCross {
            through,
            direction,
            divided_by: minor,
        }));
    }

    // Every minor came back zero or could not be built, and those are different
    // answers: the first says the two planes are parallel and the second says
    // this arithmetic ran out. `‖n₁ × n₂‖²` is the same question asked where
    // level 2 can also have a go at it.
    let across = squared_length_of_a_cross_product(WHERE_THREE_SURFACES_MEET, mine, theirs);
    match ask(&across, Measures::Length, at, policy, log)? {
        Decision::Zero => Ok(None),
        Decision::Pos => Err(an_exact_step_could_not_be_taken(at)),
        other => unreachable!("a sum of squares is never below zero: {other:?}"),
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
    line: &WhereTwoPlanesCross,
    ring: &Torus,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    let quartic =
        how_a_ring_cuts_the_line(line, ring).ok_or_else(|| an_exact_step_could_not_be_taken(at))?;
    assert!(
        matches!(quartic[0].sign(), Decision::Pos),
        "a line's direction and a ring's axis both have length, so this is above zero"
    );
    let found =
        where_a_polynomial_is_zero(&quartic).ok_or_else(|| an_exact_step_could_not_be_taken(at))?;

    let mut places: SmallVec<[PredicatePoint; 2]> = found
        .at
        .iter()
        .map(|step| the_point_at(line, *step))
        .collect();
    for unsure in &found.or_maybe_at {
        // The quartic is a length to the fourth and `Measures` stops at a
        // volume; `where_three_planes_meet`'s note says why the nearest arm is
        // taken rather than a new one invented here, and §28 says what should
        // replace the lot.
        let touching = ask(
            &how_far_it_reaches(unsure),
            Measures::Volume,
            at,
            policy,
            log,
        )?;
        assert!(
            matches!(touching, Decision::Pos | Decision::Neg | Decision::Zero),
            "a sign question answers with a sign: {touching:?}"
        );
        if matches!(touching, Decision::Zero) {
            places.push(the_point_at(line, unsure.inside));
        }
    }
    if places.is_empty() {
        return Ok(MeetOfThree::Nowhere);
    }
    Ok(MeetOfThree::At(places))
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
/// squared against `m²` and neither against `w·w`**, which is the same rule
/// [`Torus`] is certified under and the opposite of a pipe's: a torus's stored
/// radii are the numbers the caller wrote, and a discriminant that scaled them
/// by the axis's length would be certified about a different ring from the one
/// the sign predicate is (§27.1).
fn how_a_ring_cuts_the_line(line: &WhereTwoPlanesCross, ring: &Torus) -> Option<[Exact; 5]> {
    let along = ring.axis().direction().components();
    let reach = the_line_offset_to(line, ring.centre().coordinates())?;
    let axis_squared = Exact::dot_of(along, along)?;
    let divisor_squared = line.divided_by.times(&line.divided_by)?;
    let ring_squared = Exact::product_of(ring.ring_radius(), ring.ring_radius())?;
    let hollow = ring_squared.minus(&Exact::product_of(ring.tube_radius(), ring.tube_radius())?);

    let bulge = [
        Exact::squared_length_of(&line.direction)?,
        twice(&Exact::dot_of_two_exact(&reach, &line.direction)?)?,
        Exact::squared_length_of(&reach)?.plus(&divisor_squared.times(&hollow)?),
    ];
    let sideways = Exact::cross_of(&line.direction, along)?;
    let across = Exact::cross_of(&reach, along)?;
    let round = [
        Exact::squared_length_of(&sideways)?,
        twice(&Exact::dot_of_two_exact(&across, &sideways)?)?,
        Exact::squared_length_of(&across)?,
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

/// Twice a number, exactly, or `None` when that runs off the top of the range.
fn twice(value: &Exact) -> Option<Exact> {
    value.times(&Exact::of(2.0))
}

/// How far the quartic reaches over a box it turns in, as a question the cascade
/// can ask.
///
/// There is no exact level here and that is the refusal, not an omission: the
/// turning point is not an `f64`, so no exact evaluation reaches it. What level
/// 2 hands over is a box round the height of the turn, and a policy that will
/// not read a zero out of a box that straddles one is refused.
fn how_far_it_reaches(
    unsure: &Maybe,
) -> Question<impl Fn() -> Interval + '_, impl Fn() -> Option<Exact> + '_> {
    Question {
        named: WHERE_THREE_SURFACES_MEET,
        enclosing: move || unsure.how_far_it_reaches,
        exactly: move || None,
    }
}

/// `A·s² + 2B·s + C`, where `s` runs along the line.
///
/// Kept as three exact expansions and never as one polynomial, because the sign
/// of the discriminant and the size of the roots are read off them differently:
/// the sign goes through [`Exact::square_minus`], which slides both sides up the
/// range and hands back a sign with no size in it, and the roots go through the
/// enclosures of these three.
struct Quadratic {
    squared: Exact,
    half_linear: Exact,
    constant: Exact,
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
    fn difference_of_the_two_halves(&self) -> Option<Exact> {
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
fn how_a_quadric_cuts_the_line(line: &WhereTwoPlanesCross, quadric: &Surface) -> Option<Quadratic> {
    match quadric {
        Surface::Sphere(ball) => how_a_ball_cuts_the_line(line, ball),
        Surface::Cylinder(pipe) => how_a_pipe_cuts_the_line(line, pipe),
        Surface::Cone(cone) => how_a_cone_cuts_the_line(line, cone),
        Surface::Plane(flat) => panic!("{flat:?} is flat, so three planes is the question"),
        Surface::Torus(ring) => panic!("{ring:?} is a quartic and is refused before this"),
    }
}

/// `‖V + s·D‖² − m²R²` where `V` is the line's point offset to the ball's
/// centre, a **degree ten** discriminant in the stored numbers.
///
/// `docs/decisions.md` §31.2 guessed about fourteen for this row and §34 records
/// what it is. Both halves of `B² − A·C` come to ten: `A` is `D·D` at four, `B`
/// is `V·D` at five and `C` is `V·V − m²R²` at six.
fn how_a_ball_cuts_the_line(line: &WhereTwoPlanesCross, ball: &Sphere) -> Option<Quadratic> {
    let reach = the_line_offset_to(line, ball.centre().coordinates())?;
    Some(Quadratic {
        squared: Exact::squared_length_of(&line.direction)?,
        half_linear: Exact::dot_of_two_exact(&reach, &line.direction)?,
        constant: Exact::squared_length_of(&reach)?.minus(&squared_radius_of(line, ball.radius())?),
    })
}

/// `‖(V + s·D) × w‖² − m²r²` for the pipe's stored axis direction `w`, a
/// **degree fourteen** discriminant.
///
/// The radius is squared against `m²` and not against `m²‖w‖²`, because
/// [`SideOfSurface`](rcad_predicates::SideOfSurface) for a pipe is
/// `‖(p − o) × w‖² − r²` — so the pipe these numbers name has radius `r/‖w‖`,
/// and a discriminant that scaled the radius differently would be certified
/// about a different pipe from the one the sign predicate is (§27.1).
fn how_a_pipe_cuts_the_line(line: &WhereTwoPlanesCross, pipe: &Cylinder) -> Option<Quadratic> {
    let along = pipe.axis().direction().components();
    let reach = the_line_offset_to(line, pipe.axis().origin().coordinates())?;
    let sideways = Exact::cross_of(&line.direction, along)?;
    let across = Exact::cross_of(&reach, along)?;
    Some(Quadratic {
        squared: Exact::squared_length_of(&sideways)?,
        half_linear: Exact::dot_of_two_exact(&sideways, &across)?,
        constant: Exact::squared_length_of(&across)?
            .minus(&squared_radius_of(line, pipe.radius())?),
    })
}

/// `‖(V + s·D) × w‖² − slope²·((V + s·D)·w)²` for the cone's stored axis
/// direction `w`.
///
/// **No `m²` correction, and that is the cone being easier than the pipe rather
/// than the correction being forgotten.** A pipe's constant term compares an
/// area against `r²`, so writing the line with its divisor left in makes the
/// radius carry `m²` — see [`squared_radius_of`]. Both halves here are degree
/// two in `V + s·D` and the slope is a pure ratio, so the divisor factors out of
/// the whole expression and every sign read off it is untouched.
fn how_a_cone_cuts_the_line(line: &WhereTwoPlanesCross, cone: &Cone) -> Option<Quadratic> {
    let along = cone.axis().direction().components();
    let up = Exact::of_each(along);
    let reach = the_line_offset_to(line, cone.apex().coordinates())?;
    let sideways = Exact::cross_of(&line.direction, along)?;
    let across = Exact::cross_of(&reach, along)?;
    let slope_squared = Exact::product_of(cone.slope(), cone.slope())?;
    let direction_up = Exact::dot_of_two_exact(&line.direction, &up)?;
    let reach_up = Exact::dot_of_two_exact(&reach, &up)?;
    Some(Quadratic {
        squared: Exact::squared_length_of(&sideways)?
            .minus(&slope_squared.times(&direction_up.times(&direction_up)?)?),
        half_linear: Exact::dot_of_two_exact(&sideways, &across)?
            .minus(&slope_squared.times(&direction_up.times(&reach_up)?)?),
        constant: Exact::squared_length_of(&across)?
            .minus(&slope_squared.times(&reach_up.times(&reach_up)?)?),
    })
}

/// The line's own point, moved so `place` is the origin, and still scaled by the
/// line's divisor.
fn the_line_offset_to(line: &WhereTwoPlanesCross, place: [f64; 3]) -> Option<[Exact; 3]> {
    let mut reach = [Exact::of(0.0), Exact::of(0.0), Exact::of(0.0)];
    for axis in 0..3 {
        reach[axis] = line.through[axis].minus(&line.divided_by.times(&Exact::of(place[axis]))?);
    }
    Some(reach)
}

/// `m²r²`, which is what a radius has to be compared against once the line has
/// been written with its divisor left in.
fn squared_radius_of(line: &WhereTwoPlanesCross, radius: f64) -> Option<Exact> {
    line.divided_by
        .times(&line.divided_by)?
        .times(&Exact::product_of(radius, radius)?)
}

/// Where the line meets the quadric: nowhere, at one point, at two, or along the
/// whole of it.
fn where_a_line_meets_a_quadric(
    line: &WhereTwoPlanesCross,
    quadratic: &Quadratic,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
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
        &the_sign_of(&quadratic.squared),
        Measures::Length,
        at,
        policy,
        log,
    )? {
        Decision::Pos | Decision::Neg => (),
        Decision::Zero => {
            return where_a_line_meets_a_quadric_it_runs_alongside(
                line, quadratic, at, policy, log,
            );
        }
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }

    let squared = quadratic.squared.enclosure();
    let half_linear = quadratic.half_linear.enclosure();
    match ask(
        &the_discriminant_of(quadratic),
        Measures::Area,
        at,
        policy,
        log,
    )? {
        Decision::Neg => Ok(MeetOfThree::Nowhere),
        Decision::Zero => Ok(MeetOfThree::At(smallvec![the_point_at(
            line,
            quotient_or_everything(-half_linear, squared),
        )])),
        Decision::Pos => {
            let [lower, upper] = the_two_places_along_it(quadratic);
            Ok(MeetOfThree::At(smallvec![
                the_point_at(line, lower),
                the_point_at(line, upper),
            ]))
        }
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
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
    line: &WhereTwoPlanesCross,
    quadratic: &Quadratic,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<MeetOfThree> {
    match ask(
        &the_sign_of(&quadratic.half_linear),
        Measures::Length,
        at,
        policy,
        log,
    )? {
        Decision::Pos | Decision::Neg => {
            let twice_the_linear = Interval::exactly(2.0) * quadratic.half_linear.enclosure();
            return Ok(MeetOfThree::At(smallvec![the_point_at(
                line,
                quotient_or_everything(-quadratic.constant.enclosure(), twice_the_linear),
            )]));
        }
        Decision::Zero => (),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
    match ask(
        &the_sign_of(&quadratic.constant),
        Measures::Area,
        at,
        policy,
        log,
    )? {
        Decision::Zero => Ok(MeetOfThree::MoreThanAPoint),
        Decision::Pos | Decision::Neg => Ok(MeetOfThree::Nowhere),
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

/// The point `step` of the way along the line.
fn the_point_at(line: &WhereTwoPlanesCross, step: Interval) -> PredicatePoint {
    let divisor = line.divided_by.enclosure();
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        quotient_or_everything(
            line.through[axis].enclosure() + step * line.direction[axis].enclosure(),
            divisor,
        )
    }))
}

/// The sign of an exact number, as a question the cascade can ask.
fn the_sign_of(
    value: &Exact,
) -> Question<impl Fn() -> Interval + '_, impl Fn() -> Option<Exact> + '_> {
    Question {
        named: WHERE_THREE_SURFACES_MEET,
        enclosing: move || value.enclosure(),
        exactly: move || Some(value.clone()),
    }
}

/// `B² − A·C`, as a question the cascade can ask.
fn the_discriminant_of(
    quadratic: &Quadratic,
) -> Question<impl Fn() -> Interval + '_, impl Fn() -> Option<Exact> + '_> {
    Question {
        named: WHERE_THREE_SURFACES_MEET,
        enclosing: move || the_size_of_the_discriminant(quadratic),
        exactly: move || {
            quadratic
                .half_linear
                .square_minus(&quadratic.squared.times(&quadratic.constant)?)
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
    if let Some(exactly) = quadratic.difference_of_the_two_halves() {
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
fn an_exact_step_could_not_be_taken(at: &[PredicatePoint]) -> Error {
    let mut records = ProvenanceLog::new();
    records.push(Record::new(
        WHERE_THREE_SURFACES_MEET,
        at,
        Level::Exact,
        Decision::Undecidable {
            enclosure: f64::INFINITY,
        },
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

    #[test]
    fn three_surfaces_with_only_one_flat_one_are_refused_by_name() {
        let ball = Surface::Sphere(Sphere::new(Point::origin(), 5.0).expect("a radius"));
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
                let discriminant = quadratic
                    .difference_of_the_two_halves()
                    .expect("all of this fits");
                widest = widest.max(discriminant.entries());
                println!(
                    "{what}{}: A {} entries, B {}, C {}, B²−AC {} entries spanning {} binary orders",
                    if turned {
                        ", turned"
                    } else {
                        ", on the lattice"
                    },
                    quadratic.squared.entries(),
                    quadratic.half_linear.entries(),
                    quadratic.constant.entries(),
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
