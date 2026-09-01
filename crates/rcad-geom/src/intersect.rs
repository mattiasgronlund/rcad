//! Where two surfaces cross, named and never carried.
//!
//! Every question this module asks is the **sign of a polynomial in the numbers
//! a [`Surface`] stores**, and not one of them is a distance compared against a
//! tolerance. `docs/decisions.md` §27.1 lists them all, with the degree and the
//! dimension of each.
//!
//! # The discriminants are not the textbook ones
//!
//! A plane's normal is scaled to length one on the way in and that division
//! rounds, so [`Plane::side_of_point`] is certified about the plane the *stored*
//! numbers name; the same goes for a pipe, whose certified radius is
//! `radius / length(stored direction)`. Write "does this plane cut this ball" as
//! `R² − (n·c − d)²` and it answers about a ball of radius `R/‖n‖` — a different
//! ball, by about `1e-16·R`. Every discriminant here therefore carries `n·n` and
//! `w·w`, which costs three products and two sums each.
//!
//! **The rule, and it is the load-bearing one: an intersector's discriminants
//! are polynomials in the same stored numbers `SideOfSurface` reads, with no
//! normalization anywhere.** Otherwise the intersector and the sign predicates
//! are certified about two different solids (§27.1).
//!
//! # There is no near-tangency band
//!
//! [`intersect`] takes two `Surface`s, a `Surface` is made of `f64`, and level 3
//! of the cascade decides any polynomial in exact `f64`. So the naming half of
//! M3 has **no constructed input at all**, and its only way to refuse is exact
//! arithmetic declining because a product would overflow or fall into the
//! subnormals. Every refusal below traces to that, and each one is a **gap**
//! rather than a limit: the answer exists and the arithmetic could not reach it.

use rcad_predicates::{
    Decision, Filtered, Interval, Measures, Point as PredicatePoint, Policy, Predicate,
    ProvenanceLog, Record, decide, resolve_under_policy,
};
use smallvec::smallvec;

use crate::de_boor::quotient_enclosing;
use crate::exact::Exact;
use crate::{
    Cylinder, EdgeRun, Error, NamedRun, Plane, Point, Result, Sphere, Surface, SurfaceMeet, Vector,
};

/// The name the two-plane crossing goes by in provenance records.
pub const WHERE_TWO_PLANES_CROSS: &str = "where two planes cross";

/// The name the plane-and-ball crossing goes by in provenance records.
pub const WHERE_A_PLANE_AND_A_BALL_CROSS: &str = "where a plane and a ball cross";

/// The name the plane-and-pipe crossing goes by in provenance records.
pub const WHERE_A_PLANE_AND_A_PIPE_CROSS: &str = "where a plane and a pipe cross";

/// The name the two-ball crossing goes by in provenance records.
pub const WHERE_TWO_BALLS_CROSS: &str = "where two balls cross";

/// Where two surfaces cross, as an [`EdgeRun`] and nothing that had to be built.
///
/// Both surfaces arrive with their locations **already composed**, and that is
/// deliberate: `FaceView::plane` is `surface.transformed(&location)` on every
/// read, and an intersector that read its own faces would pay that per pair, per
/// fuse (`docs/decisions.md` §19.1, §24.8). The caller in `rcad-topo` composes
/// once.
///
/// What comes back names a curve without drawing one. A circle where a plane
/// cuts a ball is *the circle where this plane cuts this ball*, and the two
/// surfaces are the whole of that description — a stored curve beside them would
/// be a second authority on where the edge is, with no tiebreak at the first ulp
/// of disagreement (§24.3).
///
/// # Which pairs are named here
///
/// The four whose crossing **lies in a plane**: plane–plane, plane–ball,
/// plane–pipe and ball–ball. Ball–pipe and pipe–pipe cross in a degree-four
/// space curve whose sign questions run to the eighth and twelfth power, and
/// they are M3's second stage (§24.5, §27.1).
///
/// # Errors
///
/// [`Error::Undecidable`] when a sign question could not be settled, carrying
/// the records. For the exact `f64` a `Surface` holds that only happens when a
/// product would overflow or fall into the subnormals — never because two
/// surfaces were nearly tangent, because level 3 decides tangency exactly
/// (§27.1).
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when a crossing comes in two
/// pieces and no `f64` unit normal is exactly across both the pipe's axis and
/// the plane's own normal, so no half-space can be written down that picks one
/// line and holds along the whole of it.
///
/// [`Error::CrossingRunsToAQuartic`] for the two pairs the closed form does not
/// name.
///
/// **A partly undecidable answer refuses whole.** A caller handed one of two
/// curves has no way to know a second existed, and a fuse classifying against
/// half a crossing builds a shell that does not close (§24.10).
///
pub fn intersect(
    first: &Surface,
    second: &Surface,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    match (first, second) {
        (Surface::Plane(one), Surface::Plane(other)) => {
            where_two_planes_cross(one, other, policy, log)
        }
        (Surface::Plane(flat), Surface::Sphere(ball))
        | (Surface::Sphere(ball), Surface::Plane(flat)) => {
            where_a_plane_and_a_ball_cross(flat, ball, policy, log)
        }
        (Surface::Plane(flat), Surface::Cylinder(pipe))
        | (Surface::Cylinder(pipe), Surface::Plane(flat)) => {
            where_a_plane_and_a_pipe_cross(flat, pipe, policy, log)
        }
        (Surface::Sphere(one), Surface::Sphere(other)) => {
            where_two_balls_cross(one, other, policy, log)
        }
        // Left as its own arm rather than folded into the panic below, so the
        // second stage fills it in rather than restructuring what this one
        // shipped (§24.12).
        (Surface::Sphere(_), Surface::Cylinder(_))
        | (Surface::Cylinder(_), Surface::Sphere(_))
        | (Surface::Cylinder(_), Surface::Cylinder(_)) => Err(Error::CrossingRunsToAQuartic {
            first: name_of(first),
            second: name_of(second),
        }),
    }
}

/// Two planes: the same plane, nothing at all, or one endless straight line.
///
/// **The pair is the line already**, exactly and for free, which is why nothing
/// is worked out here beyond deciding which of the three it is (§15.9, §17.1).
/// Turning the pair into a point plus a direction would normalize a cross
/// product and divide, replacing an exact object with a rounded one before
/// anything used it.
fn where_two_planes_cross(
    one: &Plane,
    other: &Plane,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let normals = [one, other].map(|plane| plane.normal().components());
    let offsets = [one, other].map(Plane::offset);

    let across = squared_length_of_a_cross_product(WHERE_TWO_PLANES_CROSS, normals[0], normals[1]);
    let at = [one, other].map(|plane| PredicatePoint::from(plane.origin().coordinates()));
    // Two unit normals crossed together is a dimensionless question. See the
    // note in `where_a_plane_and_a_pipe_cross` for why that is filed as a
    // length, and §28 for what should replace it.
    match ask(&across, Measures::Length, &at, policy, log)? {
        // The two normals point along different lines, so the planes lean
        // against each other and cross in one endless line.
        Decision::Pos => Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
            EdgeRun::Straight,
        )])),
        // Parallel. Either the same plane or two that never meet, and what tells
        // them apart is whether the offsets are in the same ratio as the
        // normals — `d₂·n₁ − d₁·n₂`, componentwise (§17.4).
        Decision::Zero => {
            let apart = how_far_apart_two_parallel_planes_are(
                WHERE_TWO_PLANES_CROSS,
                (normals[0], offsets[0]),
                (normals[1], offsets[1]),
            );
            match ask(&apart, Measures::Area, &at, policy, log)? {
                Decision::Zero => Ok(SurfaceMeet::SameSurface),
                Decision::Pos | Decision::Neg => Ok(SurfaceMeet::Nothing),
                other => unreachable!("a sum of squares is never below zero: {other:?}"),
            }
        }
        other => unreachable!("a sum of squares is never below zero: {other:?}"),
    }
}

/// A plane and a ball: one circle, one touching point, or nothing.
///
/// The discriminant is `n·n · R² − (n·c − d)²`, an **area** — the ball's radius
/// against the distance from its centre to the plane, both squared and both in
/// the metric the stored numbers name (§27.1).
fn where_a_plane_and_a_ball_cross(
    flat: &Plane,
    ball: &Sphere,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let normal = flat.normal().components();
    let d = flat.offset();
    let centre = ball.centre().coordinates();
    let radius = ball.radius();

    let room = Question {
        named: WHERE_A_PLANE_AND_A_BALL_CROSS,
        enclosing: || {
            let reach = dot_enclosing(normal, centre) - Interval::exactly(d);
            squared_length_enclosing(normal) * squared_enclosing(radius) - reach * reach
        },
        exactly: || {
            let reach = Exact::dot_of(normal, centre)?.minus(&Exact::of(d));
            let room = Exact::dot_of(normal, normal)?
                .times(&Exact::product_of(radius, radius)?)?
                .minus(&reach.times(&reach)?);
            Some(room)
        },
    };

    let at = [
        PredicatePoint::from(flat.origin().coordinates()),
        PredicatePoint::from(centre),
    ];
    match ask(&room, Measures::Area, &at, policy, log)? {
        Decision::Pos => Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
            EdgeRun::RightRoundWhereItsFacesCross,
        )])),
        Decision::Zero => Ok(SurfaceMeet::TouchAtOnePoint {
            at: where_a_plane_is_nearest(flat, ball.centre()),
        }),
        Decision::Neg => Ok(SurfaceMeet::Nothing),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// A plane and a pipe: one closed curve, or — where the plane runs along the
/// axis — two lines, one line, or nothing.
///
/// A plane that is **not** parallel to the axis always cuts an endless pipe in
/// exactly one closed curve. An ellipse in general, a circle when the plane is
/// square to the axis, and the naming is the same either way: neither is drawn,
/// so neither needs telling apart (§24.5).
fn where_a_plane_and_a_pipe_cross(
    flat: &Plane,
    pipe: &Cylinder,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let normal = flat.normal().components();
    let d = flat.offset();
    let along = pipe.axis().direction().components();
    let origin = pipe.axis().origin().coordinates();
    let radius = pipe.radius();
    let at = [
        PredicatePoint::from(flat.origin().coordinates()),
        PredicatePoint::from(origin),
    ];

    let leans = Question {
        named: WHERE_A_PLANE_AND_A_PIPE_CROSS,
        enclosing: || dot_enclosing(normal, along),
        exactly: || Exact::dot_of(normal, along),
    };
    // Dimensionless: both are direction components of order one. `Measures` has
    // no arm for that and §27.2 declined to add one, on the grounds that level 3
    // always decides such a question and no record would ever be written. It can
    // be — a direction with a component down among the subnormals underflows its
    // own product — and `Measures::Length` is the arm whose conversion is the
    // identity, which is the right conversion for a value with no units. The
    // *name* is wrong and §28 says so.
    match ask(&leans, Measures::Length, &at, policy, log)? {
        Decision::Pos | Decision::Neg => {
            return Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
                EdgeRun::RightRoundWhereItsFacesCross,
            )]));
        }
        Decision::Zero => (),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }

    // Parallel to the axis. The discriminant is `n·n · r² − w·w · (n·o − d)²`,
    // an area: the pipe's radius against the distance from its axis to the
    // plane, both squared, each in the metric its own stored numbers name.
    let room = Question {
        named: WHERE_A_PLANE_AND_A_PIPE_CROSS,
        enclosing: || {
            let reach = dot_enclosing(normal, origin) - Interval::exactly(d);
            squared_length_enclosing(normal) * squared_enclosing(radius)
                - squared_length_enclosing(along) * reach * reach
        },
        exactly: || {
            let reach = Exact::dot_of(normal, origin)?.minus(&Exact::of(d));
            let room = Exact::dot_of(normal, normal)?
                .times(&Exact::product_of(radius, radius)?)?
                .minus(&Exact::dot_of(along, along)?.times(&reach.times(&reach)?)?);
            Some(room)
        },
    };

    match ask(&room, Measures::Area, &at, policy, log)? {
        Decision::Pos => two_lines_along_a_pipe(flat, pipe),
        Decision::Zero => Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
            EdgeRun::Straight,
        )])),
        Decision::Neg => Ok(SurfaceMeet::Nothing),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// The two lines a plane cuts out of a pipe it runs down the inside of, each
/// picked out by the half-space it lies in.
///
/// **This is the only crossing in the four planar pairs that comes in two
/// pieces**, so it is the only one that needs a `picked_by` at all (§27.1). The
/// two lines are parallel to the axis and either side of the plane through the
/// axis square to the cutting plane, so that plane is what tells them apart.
///
/// # Why this can refuse, and why the refusal is a gap
///
/// The picking plane's normal has to be exactly across both the axis direction
/// and the cutting plane's normal, or a *line* — endless — leaves the half-space
/// somewhere far along itself, and a run picked by a half-space it only stays in
/// near the origin is not picked at all. Every `Plane` scales its normal to
/// length one and that division rounds, so the normal we can write down is
/// exactly square to both only sometimes. When it is not, the answer exists and
/// the arithmetic could not reach it — a **gap** in §16.10's sense, not a limit.
///
/// The families of fixes weighed. **A plane that stores its normal unscaled**:
/// that is a second kind of plane for every predicate in the kernel to be
/// certified about, which is a change to a shared type and M4's business.
/// **Testing the separation instead of arranging it**: comparing the picking
/// plane's own offset rounding against the half-distance between the two lines
/// is a sign question of about the fourteenth degree, and an expansion that
/// long is exactly the heap scratch §27.1 hands to M3's second stage. **A curved
/// half-space**: [`Cylinder`] normalizes its axis the same way, so it buys
/// nothing. The second is the fix, and it lands where its machinery does.
fn two_lines_along_a_pipe(flat: &Plane, pipe: &Cylinder) -> Result<SurfaceMeet> {
    let normal = flat.normal().components();
    let along = pipe.axis().direction().components();
    let origin = pipe.axis().origin();

    let across = Vector::from(along).cross(Vector::from(normal));
    let between = Plane::from_point_and_normal(origin, across)?;
    let picker = between.normal().components();
    let offset = between.offset();

    // Three exact questions, and together they say the picking plane holds the
    // axis and is square to both directions with nothing rounded. Then each line
    // sits at a constant, opposite distance from it, all the way along.
    let square_to_the_axis = Exact::dot_of(picker, along);
    let square_to_the_normal = Exact::dot_of(picker, normal);
    let through_the_axis =
        Exact::dot_of(picker, origin.coordinates()).map(|reach| reach.minus(&Exact::of(offset)));
    let exactly_between = [square_to_the_axis, square_to_the_normal, through_the_axis]
        .iter()
        .all(|answer| matches!(answer.as_ref().map(Exact::sign), Some(Decision::Zero)));
    if !exactly_between {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    let pipe = Surface::Cylinder(*pipe);
    Ok(SurfaceMeet::Curves(
        [between, between.reversed()]
            .into_iter()
            .map(|half_space| {
                NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                    and: pipe,
                    picked_by: smallvec![Surface::Plane(half_space)],
                })
            })
            .collect(),
    ))
}

/// Two balls: one circle, one touching point, the same ball, or nothing.
///
/// Two questions, both areas. `‖c₂ − c₁‖² − (R₁ + R₂)²` says whether they are
/// clear of each other or touching from outside; `‖c₂ − c₁‖² − (R₁ − R₂)²` says
/// whether one is swallowed by the other or touching from inside. Between the
/// two they cross in a circle, and that circle lies in the radical plane —
/// which nothing here builds, because the pair names the circle already and the
/// plane is only wanted to *evaluate points along* it (§27.1).
fn where_two_balls_cross(
    one: &Sphere,
    other: &Sphere,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let [here, there] = [one, other].map(|ball| ball.centre().coordinates());
    let [mine, theirs] = [one, other].map(Sphere::radius);
    let at = [here, there].map(PredicatePoint::from);

    let apart = |sign: f64| Question {
        named: WHERE_TWO_BALLS_CROSS,
        enclosing: move || {
            let reach =
                Interval::exactly(mine) + Interval::exactly(sign) * Interval::exactly(theirs);
            squared_gap_enclosing(here, there) - reach * reach
        },
        exactly: move || {
            let together = Exact::product_of(mine, sign * theirs)?;
            let reach = Exact::product_of(mine, mine)?
                .plus(&together)
                .plus(&together)
                .plus(&Exact::product_of(theirs, theirs)?);
            Some(squared_gap_exactly(here, there)?.minus(&reach))
        },
    };

    match ask(&apart(1.0), Measures::Area, &at, policy, log)? {
        Decision::Pos => return Ok(SurfaceMeet::Nothing),
        Decision::Zero => {
            return Ok(SurfaceMeet::TouchAtOnePoint {
                at: where_two_balls_touch(one, other, mine + theirs),
            });
        }
        Decision::Neg => (),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }

    match ask(&apart(-1.0), Measures::Area, &at, policy, log)? {
        Decision::Pos => Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
            EdgeRun::RightRoundWhereItsFacesCross,
        )])),
        // The centres are the same distance apart as the radii differ. With
        // equal radii that distance is nothing at all and the two are one ball;
        // otherwise the smaller rests against the inside of the larger.
        Decision::Zero if mine == theirs => Ok(SurfaceMeet::SameSurface),
        Decision::Zero => Ok(SurfaceMeet::TouchAtOnePoint {
            at: where_two_balls_touch(one, other, mine - theirs),
        }),
        Decision::Neg => Ok(SurfaceMeet::Nothing),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// The point on `flat` nearest `centre`, as the box it is really known to be
/// inside.
///
/// A box and not a triple, because the division rarely lands on a number `f64`
/// can hold, and an exact triple would buy a certified answer to the wrong
/// question (`docs/decisions.md` §2.5).
fn where_a_plane_is_nearest(flat: &Plane, centre: Point) -> PredicatePoint {
    let normal = flat.normal().components();
    let reach = dot_enclosing(normal, centre.coordinates()) - Interval::exactly(flat.offset());
    let part = quotient_enclosing(reach, squared_length_enclosing(normal));
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        Interval::exactly(centre.coordinates()[axis]) - part * Interval::exactly(normal[axis])
    }))
}

/// The one point two touching balls share, as the box it is known to be inside.
///
/// `gap` is how far apart the centres are — `R₁ + R₂` when they touch from
/// outside and `R₁ − R₂` when the smaller rests inside the larger, and in both
/// cases the discriminant coming back [`Decision::Zero`] is what says the real
/// distance is exactly that. So the direction along the centre line is worked
/// out without a square root anywhere: the point is `c₁ + (R₁/gap)·(c₂ − c₁)`,
/// and the same expression is right for both, sign and all.
fn where_two_balls_touch(one: &Sphere, other: &Sphere, gap: f64) -> PredicatePoint {
    let [here, there] = [one, other].map(|ball| ball.centre().coordinates());
    let part = quotient_enclosing(Interval::exactly(one.radius()), Interval::exactly(gap));
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        Interval::exactly(here[axis])
            + part * (Interval::exactly(there[axis]) - Interval::exactly(here[axis]))
    }))
}

/// One sign question of the intersector, run through the cascade and then
/// through the policy.
///
/// Level 1 is deliberately not implemented and every question here declines it.
/// A forward error bound is a *derived* claim about a polynomial, and §15.6 says
/// a published bound needs a test; deriving and testing one for each of the
/// seven questions above buys speed on a path nothing has measured, which is the
/// shape §1.4 and §24.16 both turn down. Level 2 decides everything level 1
/// would have, by construction rather than by derivation, and level 3 decides
/// the rest.
///
/// # Errors
///
/// [`Error::Undecidable`] when neither the cascade nor the policy could settle
/// the sign, carrying the one record that gave up.
fn ask<P: Predicate>(
    question: &P,
    measures: Measures,
    inputs: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Decision> {
    let (decision, level) = decide(question);
    if decision.is_certified() {
        return Ok(decision);
    }

    let mut records = ProvenanceLog::new();
    let settled = resolve_under_policy(
        Record::new(question.name(), inputs, level, decision, measures),
        policy,
        &mut records,
    );
    if settled.is_certified() {
        records.drain_into(log);
        return Ok(settled);
    }
    Err(Error::Undecidable { records })
}

/// One sign question, evaluated the two ways the cascade can use.
///
/// A closure pair rather than seven near-identical structs: what differs
/// between the questions above is the expression and nothing else, and
/// `docs/decisions.md` §27.1's table is the readable form of the list.
struct Question<E, X> {
    named: &'static str,
    enclosing: E,
    exactly: X,
}

impl<E, X> Predicate for Question<E, X>
where
    E: Fn() -> Interval,
    X: Fn() -> Option<Exact>,
{
    fn name(&self) -> &'static str {
        self.named
    }

    fn on_floats(&self) -> Option<Filtered> {
        None
    }

    fn on_intervals(&self) -> Interval {
        (self.enclosing)()
    }

    fn exactly(&self) -> Option<Decision> {
        match (self.exactly)()?.sign() {
            Decision::Neg => Some(Decision::Neg),
            Decision::Zero => Some(Decision::Zero),
            Decision::Pos => Some(Decision::Pos),
            // A product overflowed on the way in. Exact arithmetic that is not
            // exact has nothing to offer, so this level cannot run.
            Decision::Undecidable { .. } => None,
            other => unreachable!("unhandled decision from exact arithmetic: {other:?}"),
        }
    }
}

/// `‖u × v‖²`, which is zero exactly when the two point along one line.
fn squared_length_of_a_cross_product(
    named: &'static str,
    u: [f64; 3],
    v: [f64; 3],
) -> Question<impl Fn() -> Interval, impl Fn() -> Option<Exact>> {
    let component = move |axis: usize| {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        (u[next], v[after], u[after], v[next])
    };
    Question {
        named,
        enclosing: move || {
            (0..3).fold(Interval::exactly(0.0), |so_far, axis| {
                let (one, two, three, four) = component(axis);
                let across = Interval::exactly(one) * Interval::exactly(two)
                    - Interval::exactly(three) * Interval::exactly(four);
                so_far + across * across
            })
        },
        exactly: move || {
            let mut total = Exact::of(0.0);
            for axis in 0..3 {
                let (one, two, three, four) = component(axis);
                let across = Exact::product_of(one, two)?.minus(&Exact::product_of(three, four)?);
                total = total.plus(&across.times(&across)?);
            }
            Some(total)
        },
    }
}

/// `Σ (d₂·n₁ᵢ − d₁·n₂ᵢ)²`, which is zero exactly when two already-parallel
/// planes are the same plane.
///
/// Parallel normals are multiples of each other, so the two coefficient
/// four-vectors describe one surface exactly when the offsets are in that same
/// ratio — asked as three products rather than as a ratio, which is how §17.4
/// keeps a division out of it.
fn how_far_apart_two_parallel_planes_are(
    named: &'static str,
    mine: ([f64; 3], f64),
    theirs: ([f64; 3], f64),
) -> Question<impl Fn() -> Interval, impl Fn() -> Option<Exact>> {
    let (my_normal, my_offset) = mine;
    let (their_normal, their_offset) = theirs;
    Question {
        named,
        enclosing: move || {
            (0..3).fold(Interval::exactly(0.0), |so_far, axis| {
                let gap = Interval::exactly(their_offset) * Interval::exactly(my_normal[axis])
                    - Interval::exactly(my_offset) * Interval::exactly(their_normal[axis]);
                so_far + gap * gap
            })
        },
        exactly: move || {
            let mut total = Exact::of(0.0);
            for axis in 0..3 {
                let gap = Exact::product_of(their_offset, my_normal[axis])?
                    .minus(&Exact::product_of(my_offset, their_normal[axis])?);
                total = total.plus(&gap.times(&gap)?);
            }
            Some(total)
        },
    }
}

/// `u · v` as an enclosure.
fn dot_enclosing(u: [f64; 3], v: [f64; 3]) -> Interval {
    (0..3).fold(Interval::exactly(0.0), |so_far, axis| {
        so_far + Interval::exactly(u[axis]) * Interval::exactly(v[axis])
    })
}

/// `u · u` as an enclosure.
fn squared_length_enclosing(u: [f64; 3]) -> Interval {
    dot_enclosing(u, u)
}

/// `value²` as an enclosure.
fn squared_enclosing(value: f64) -> Interval {
    Interval::exactly(value) * Interval::exactly(value)
}

/// `‖there − here‖²` as an enclosure.
fn squared_gap_enclosing(here: [f64; 3], there: [f64; 3]) -> Interval {
    (0..3).fold(Interval::exactly(0.0), |so_far, axis| {
        let gap = Interval::exactly(there[axis]) - Interval::exactly(here[axis]);
        so_far + gap * gap
    })
}

/// `‖there − here‖²`, exactly.
fn squared_gap_exactly(here: [f64; 3], there: [f64; 3]) -> Option<Exact> {
    let mut total = Exact::of(0.0);
    for axis in 0..3 {
        let gap = Exact::of(there[axis]).minus(&Exact::of(here[axis]));
        total = total.plus(&gap.times(&gap)?);
    }
    Some(total)
}

/// What a surface is called in a refusal a person has to read.
fn name_of(surface: &Surface) -> &'static str {
    match surface {
        Surface::Plane(_) => "plane",
        Surface::Sphere(_) => "ball",
        Surface::Cylinder(_) => "pipe",
    }
}

#[cfg(test)]
mod tests {
    use rcad_predicates::{Level, SideOfSurface, decide};

    use super::*;
    use crate::Axis;

    fn a_plane(normal: [f64; 3], through: [f64; 3]) -> Surface {
        Surface::Plane(
            Plane::from_point_and_normal(Point::from(through), Vector::from(normal))
                .expect("a normal with a length"),
        )
    }

    fn a_ball(centre: [f64; 3], radius: f64) -> Surface {
        Surface::Sphere(Sphere::new(Point::from(centre), radius).expect("a radius"))
    }

    fn a_pipe(along: [f64; 3], through: [f64; 3], radius: f64) -> Surface {
        Surface::Cylinder(
            Cylinder::new(
                Axis::new(Point::from(through), Vector::from(along)).expect("a direction"),
                radius,
            )
            .expect("a radius"),
        )
    }

    /// Both ways round, because a caller has no reason to put them in one
    /// order and the answer is about the pair rather than about the arguments.
    fn crossing(first: &Surface, second: &Surface) -> SurfaceMeet {
        let mut log = ProvenanceLog::new();
        let one = intersect(first, second, &Policy::certified(), &mut log)
            .expect("this pair is exactly nameable");
        let other = intersect(second, first, &Policy::certified(), &mut log)
            .expect("this pair is exactly nameable");
        assert!(log.is_empty(), "nothing here needs approximating: {log:?}");
        assert_eq!(
            same_shape(&one),
            same_shape(&other),
            "the pair names the crossing, so the order the pair is written in cannot"
        );
        one
    }

    /// What kind of crossing this is, with nothing in it that names one surface
    /// before the other.
    fn same_shape(meet: &SurfaceMeet) -> (&'static str, usize) {
        match meet {
            SurfaceMeet::Nothing => ("nothing", 0),
            SurfaceMeet::TouchAtOnePoint { .. } => ("one point", 0),
            SurfaceMeet::Curves(runs) => ("curves", runs.len()),
            SurfaceMeet::SameSurface => ("the same surface", 0),
        }
    }

    fn one_run(meet: &SurfaceMeet) -> &NamedRun {
        let SurfaceMeet::Curves(runs) = meet else {
            panic!("expected one curve, got {meet:?}");
        };
        assert_eq!(runs.len(), 1, "expected one curve, got {runs:?}");
        &runs[0]
    }

    #[test]
    fn two_planes_leaning_against_each_other_cross_in_one_endless_line() {
        let met = crossing(
            &a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]),
            &a_plane([1.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
        );
        let run = one_run(&met);
        assert_eq!(run.run(), &EdgeRun::Straight);
        assert!(
            run.is_the_whole_curve(),
            "two surfaces name a line and nothing on it"
        );
    }

    #[test]
    fn a_plane_and_the_same_plane_are_one_surface_whichever_way_each_faces() {
        for facing in [1.0, -1.0] {
            let met = crossing(
                &a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 4.0]),
                &a_plane([0.0, 0.0, facing], [0.0, 0.0, 4.0]),
            );
            assert_eq!(met, SurfaceMeet::SameSurface, "facing {facing}");
        }
    }

    #[test]
    fn two_parallel_planes_a_distance_apart_never_meet() {
        let met = crossing(
            &a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 0.0]),
            &a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 4.0]),
        );
        assert_eq!(met, SurfaceMeet::Nothing);
    }

    #[test]
    fn a_plane_through_a_ball_cuts_one_circle() {
        let met = crossing(
            &a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]),
            &a_ball([0.0, 0.0, 0.0], 3.0),
        );
        let run = one_run(&met);
        assert_eq!(run.run(), &EdgeRun::RightRoundWhereItsFacesCross);
        assert!(run.is_the_whole_curve());
    }

    /// Criterion 4's first row. Tangency is a certified `Zero` on the
    /// discriminant, so a plane resting on a ball is an **answer** and not a
    /// refusal — and the point it comes back with has to be on both surfaces.
    #[test]
    fn a_plane_resting_on_a_ball_touches_it_at_one_point() {
        let flat = a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 3.0]);
        let ball = a_ball([0.0, 0.0, 0.0], 3.0);
        let SurfaceMeet::TouchAtOnePoint { at } = crossing(&flat, &ball) else {
            panic!("a plane a radius from the centre rests on the ball");
        };

        let Surface::Plane(flat) = flat else {
            unreachable!("just built one")
        };
        let Surface::Sphere(ball) = ball else {
            unreachable!("just built one")
        };
        assert_eq!(flat.side_of_point(at), Decision::Zero);
        assert_eq!(ball.side_of(at), Decision::Zero);
    }

    #[test]
    fn a_plane_clear_of_a_ball_meets_it_nowhere() {
        let met = crossing(
            &a_plane([0.0, 0.0, 1.0], [0.0, 0.0, 4.0]),
            &a_ball([0.0, 0.0, 0.0], 3.0),
        );
        assert_eq!(met, SurfaceMeet::Nothing);
    }

    /// A plane that is not parallel to the axis always cuts an endless pipe in
    /// exactly one closed curve, square on or tilted, and the naming is the
    /// same either way because neither curve is drawn.
    #[test]
    fn a_plane_that_leans_across_a_pipe_cuts_one_closed_curve() {
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);
        for normal in [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.0, 3.0, 4.0]] {
            let met = crossing(&a_plane(normal, [0.0, 0.0, 5.0]), &pipe);
            let run = one_run(&met);
            assert_eq!(
                run.run(),
                &EdgeRun::RightRoundWhereItsFacesCross,
                "a plane with normal {normal:?}"
            );
        }
    }

    /// Criterion 4's second row: a plane tangent to a pipe gives **one line**,
    /// and it is an answer rather than a refusal.
    #[test]
    fn a_plane_resting_along_a_pipe_touches_it_along_one_line() {
        let met = crossing(
            &a_plane([1.0, 0.0, 0.0], [2.0, 0.0, 0.0]),
            &a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0),
        );
        let run = one_run(&met);
        assert_eq!(run.run(), &EdgeRun::Straight);
        assert!(run.is_the_whole_curve());
    }

    /// The only crossing among the four planar pairs that comes in two pieces,
    /// and so the only one that needs a half-space to tell them apart.
    #[test]
    fn a_plane_down_the_inside_of_a_pipe_cuts_two_lines_a_half_space_apart() {
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);
        let met = crossing(&a_plane([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]), &pipe);

        let SurfaceMeet::Curves(runs) = &met else {
            panic!("a plane inside a pipe and along it cuts two lines: {met:?}");
        };
        assert_eq!(runs.len(), 2);

        let pickers: Vec<Surface> = runs
            .iter()
            .map(|run| {
                assert!(run.is_the_whole_curve(), "each line is endless");
                let EdgeRun::PartOfWhereItCrosses { and, picked_by } = run.run() else {
                    panic!("one of two lines is a piece of the crossing: {run:?}");
                };
                assert_eq!(and, &pipe);
                assert_eq!(picked_by.len(), 1);
                picked_by[0]
            })
            .collect();

        // The two half-spaces are the same plane facing opposite ways, so a
        // point on one line is inside exactly one of them.
        let on_one_line = PredicatePoint::from([1.0, f64::sqrt(3.0), 7.0]);
        let sides: Vec<Decision> = pickers
            .iter()
            .map(|picker| match picker {
                Surface::Plane(half_space) => half_space.side_of_point(on_one_line),
                other => panic!("the picker for two lines is a plane, not {other:?}"),
            })
            .collect();
        assert_eq!(sides, [Decision::Pos, Decision::Neg]);
    }

    #[test]
    fn a_plane_alongside_a_pipe_and_clear_of_it_meets_it_nowhere() {
        let met = crossing(
            &a_plane([1.0, 0.0, 0.0], [3.0, 0.0, 0.0]),
            &a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0),
        );
        assert_eq!(met, SurfaceMeet::Nothing);
    }

    #[test]
    fn two_balls_that_overlap_cross_in_one_circle() {
        let met = crossing(&a_ball([0.0, 0.0, 0.0], 3.0), &a_ball([4.0, 0.0, 0.0], 3.0));
        let run = one_run(&met);
        assert_eq!(run.run(), &EdgeRun::RightRoundWhereItsFacesCross);
    }

    /// Criterion 4's third row, both ways round: two balls resting against each
    /// other from outside, and one resting inside the other.
    #[test]
    fn two_balls_resting_against_each_other_touch_at_one_point() {
        for (centre, radius, touching) in [
            ([8.0, 0.0, 0.0], 5.0, [3.0, 0.0, 0.0]),
            ([1.0, 0.0, 0.0], 2.0, [3.0, 0.0, 0.0]),
        ] {
            let ball = a_ball([0.0, 0.0, 0.0], 3.0);
            let SurfaceMeet::TouchAtOnePoint { at } = crossing(&ball, &a_ball(centre, radius))
            else {
                panic!("a ball at {centre:?} of radius {radius} rests on the unit ball");
            };
            let Surface::Sphere(ball) = ball else {
                unreachable!("just built one")
            };
            assert_eq!(ball.side_of(at), Decision::Zero, "at {touching:?}");
            assert_eq!(
                at.enclosure().map(|side| side.lo()),
                touching,
                "the touching point is where the centre line meets both skins"
            );
        }
    }

    #[test]
    fn a_ball_and_its_own_copy_are_one_surface() {
        let met = crossing(&a_ball([1.0, 2.0, 3.0], 4.0), &a_ball([1.0, 2.0, 3.0], 4.0));
        assert_eq!(met, SurfaceMeet::SameSurface);
    }

    #[test]
    fn a_ball_swallowed_by_another_and_two_balls_apart_both_meet_nowhere() {
        for (centre, radius) in [([0.5, 0.0, 0.0], 1.0), ([20.0, 0.0, 0.0], 1.0)] {
            let met = crossing(&a_ball([0.0, 0.0, 0.0], 3.0), &a_ball(centre, radius));
            assert_eq!(met, SurfaceMeet::Nothing, "a ball at {centre:?}");
        }
    }

    /// The two pairs whose crossing is a degree-four space curve say so rather
    /// than answering something that looks like an answer.
    #[test]
    fn a_crossing_that_runs_to_a_quartic_says_so_instead_of_guessing() {
        let ball = a_ball([0.0, 0.0, 0.0], 3.0);
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);
        for (first, second, named) in [
            (&ball, &pipe, ("ball", "pipe")),
            (&pipe, &ball, ("pipe", "ball")),
            (&pipe, &pipe, ("pipe", "pipe")),
        ] {
            assert_eq!(
                intersect(
                    first,
                    second,
                    &Policy::certified(),
                    &mut ProvenanceLog::new()
                ),
                Err(Error::CrossingRunsToAQuartic {
                    first: named.0,
                    second: named.1
                })
            );
        }
    }

    /// A discriminant whose products fall off the bottom of the `f64` range is
    /// the **only** way the naming half refuses, and it is a gap and not a
    /// limit: the same pair written somewhere else in `f64` is decided.
    ///
    /// The ball's radius squared and the plane's offset squared are both far
    /// below the smallest normal number, so neither the exact level nor the
    /// interval level can place the plane against the skin.
    #[test]
    fn a_ball_too_small_for_its_own_squares_cannot_be_crossed_and_the_same_one_scaled_up_can() {
        let tiny = 2f64.powi(-538);
        let flat = a_plane([0.0, 0.0, 1.0], [0.0, 0.0, tiny]);
        let ball = a_ball([0.0, 0.0, 0.0], 3.0 * 2f64.powi(-539));

        let mut log = ProvenanceLog::new();
        match intersect(&flat, &ball, &Policy::certified(), &mut log) {
            Err(Error::Undecidable { records }) => {
                assert_eq!(records.len(), 1);
                assert_eq!(
                    records.records()[0].predicate(),
                    WHERE_A_PLANE_AND_A_BALL_CROSS
                );
                assert_eq!(records.records()[0].measures(), Measures::Area);
            }
            other => panic!("nothing here can prove a sign: {other:?}"),
        }

        // The same shapes written `2^600` further up the range, which multiplies
        // the discriminant by a positive number and so cannot change its sign.
        let up = 2f64.powi(600);
        let scaled = a_plane([0.0, 0.0, 1.0], [0.0, 0.0, tiny * up]);
        let bigger = a_ball([0.0, 0.0, 0.0], 3.0 * 2f64.powi(-539) * up);
        assert_eq!(
            same_shape(&crossing(&scaled, &bigger)),
            ("curves", 1),
            "the answer was there to be had all along"
        );
    }

    /// A direction with a component down among the subnormals underflows its own
    /// product, so "is this plane parallel to this axis" is a question that can
    /// give up — which §27.2 predicted would never happen and §28 records.
    ///
    /// It is a **gap** for the same reason as the test above: the two are
    /// plainly not parallel, and no scaling of a *unit* direction reaches the
    /// answer, which is why this one is worth writing down.
    #[test]
    fn a_direction_with_a_subnormal_component_cannot_be_told_from_square_to_a_plane() {
        let hair = 2f64.powi(-700);
        let flat = a_plane([1.0, hair, 0.0], [0.0, 0.0, 0.0]);
        let pipe = a_pipe([0.0, hair, 1.0], [5.0, 0.0, 0.0], 1.0);

        let mut log = ProvenanceLog::new();
        match intersect(&flat, &pipe, &Policy::certified(), &mut log) {
            Err(Error::Undecidable { records }) => {
                assert_eq!(
                    records.records()[0].predicate(),
                    WHERE_A_PLANE_AND_A_PIPE_CROSS
                );
            }
            other => panic!("this product falls off the bottom of the range: {other:?}"),
        }
    }

    /// The other refusal, and the one the arithmetic cannot be rescaled out of.
    ///
    /// A plane exactly along a pipe's axis and inside it cuts two lines, and
    /// telling them apart needs a unit normal exactly across both the axis and
    /// the plane's own normal. Here that direction is `(1,2,3) × (2,-1,0)`
    /// scaled to length one, and the scaling rounds it off square. The two lines
    /// are there and they are tellable apart; what runs out is `f64`'s supply of
    /// unit vectors — a **gap**, and one whose fix is the fourteenth-degree
    /// separation test §27.1 hands to the second stage.
    #[test]
    fn two_lines_no_f64_half_space_can_tell_apart_are_refused_rather_than_guessed() {
        let along = Vector::new(1.0, 2.0, 3.0)
            .normalized()
            .expect("a direction");
        let normal = Vector::new(2.0, -1.0, 0.0)
            .normalized()
            .expect("a direction");
        assert_eq!(
            along.dot(normal),
            0.0,
            "this plane has to be exactly along the axis, or it cuts an ellipse instead"
        );

        let pipe = Surface::Cylinder(
            Cylinder::new(Axis::new(Point::origin(), along).expect("a direction"), 2.0)
                .expect("a radius"),
        );
        let flat = Surface::Plane(
            Plane::from_point_and_normal(Point::new(0.2, 0.0, 0.0), normal)
                .expect("a normal with a length"),
        );

        assert_eq!(
            intersect(
                &flat,
                &pipe,
                &Policy::certified(),
                &mut ProvenanceLog::new()
            ),
            Err(Error::CannotNameWhichPieceOfACrossingThisIs)
        );

        // The same pair on the lattice, where the direction across both is a
        // coordinate axis and nothing has to be scaled at all.
        let met = crossing(
            &a_plane([1.0, 0.0, 0.0], [0.2, 0.0, 0.0]),
            &a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0),
        );
        assert_eq!(same_shape(&met), ("curves", 2));
    }

    /// The tangency questions are settled by interval arithmetic wherever the
    /// numbers are ordinary, and only fall through to exact arithmetic where the
    /// answer really is a knife edge. A suite where everything reached level 3
    /// would prove nothing about the level above it.
    #[test]
    fn an_ordinary_pair_is_settled_without_exact_arithmetic_and_a_tangent_one_needs_it() {
        let normal = [0.0, 0.0, 1.0];
        let centre = [0.0, 0.0, 0.0];
        let room = |offset: f64, radius: f64| Question {
            named: WHERE_A_PLANE_AND_A_BALL_CROSS,
            enclosing: move || {
                let reach = dot_enclosing(normal, centre) - Interval::exactly(offset);
                squared_length_enclosing(normal) * squared_enclosing(radius) - reach * reach
            },
            exactly: move || {
                let reach = Exact::dot_of(normal, centre)?.minus(&Exact::of(offset));
                Some(
                    Exact::dot_of(normal, normal)?
                        .times(&Exact::product_of(radius, radius)?)?
                        .minus(&reach.times(&reach)?),
                )
            },
        };

        assert_eq!(decide(&room(1.0, 3.0)), (Decision::Pos, Level::Interval));
        // A tenth is not an `f64`, so squaring it rounds and the two rounded
        // squares straddle each other: level 2 cannot tell, and level 3 sees
        // that the two are the same number and answers exactly.
        assert_eq!(decide(&room(0.1, 0.1)), (Decision::Zero, Level::Exact));
    }
}
