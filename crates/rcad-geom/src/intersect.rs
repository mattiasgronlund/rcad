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

use crate::enclosure::{
    cross_enclosing, dot_enclosing, exactly, gap_enclosing, quotient_or_everything, root_enclosing,
    squared_length_enclosing,
};
use crate::exact::Exact;
use crate::{
    Cone, Cylinder, EdgeRun, Error, HalfSpace, NamedRun, Picker, Plane, Point, Result, Sphere,
    Surface, SurfaceMeet, Torus, Vector,
};

/// The name the two-plane crossing goes by in provenance records.
pub const WHERE_TWO_PLANES_CROSS: &str = "where two planes cross";

/// The name the plane-and-ball crossing goes by in provenance records.
pub const WHERE_A_PLANE_AND_A_BALL_CROSS: &str = "where a plane and a ball cross";

/// The name the plane-and-pipe crossing goes by in provenance records.
pub const WHERE_A_PLANE_AND_A_PIPE_CROSS: &str = "where a plane and a pipe cross";

/// The name the two-ball crossing goes by in provenance records.
pub const WHERE_TWO_BALLS_CROSS: &str = "where two balls cross";

/// The name the ball-and-pipe crossing goes by in provenance records.
pub const WHERE_A_BALL_AND_A_PIPE_CROSS: &str = "where a ball and a pipe cross";

/// The name the two-pipe crossing goes by in provenance records.
pub const WHERE_TWO_PIPES_CROSS: &str = "where two pipes cross";

/// The name the plane-and-cone crossing goes by in provenance records.
pub const WHERE_A_PLANE_AND_A_CONE_CROSS: &str = "where a plane and a cone cross";

/// The name the cone-and-pipe crossing goes by in provenance records.
pub const WHERE_A_CONE_AND_A_PIPE_CROSS: &str = "where a cone and a pipe cross";

/// The name the plane-and-torus crossing goes by in provenance records.
pub const WHERE_A_PLANE_AND_A_TORUS_CROSS: &str = "where a plane and a torus cross";

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
/// Six of the ten. The four whose crossing **lies in a plane** — plane–plane,
/// plane–ball, plane–pipe and ball–ball — are named by one sign question or two
/// (§24.5). A ball crossing a pipe is a degree-four space curve and is named
/// too: which of the five shapes it takes is settled exactly by an eighth-degree
/// discriminant, and only *drawing* it needs the marcher (§27.1).
///
/// Two pipes are named where their axes are parallel, and that includes the
/// coaxial pair and the two lines two parallel pipes cut out of each other.
/// **Two pipes leaning against each other are not always named**, and
/// `docs/decisions.md` §29 is the argument: deciding which of the four shapes
/// that crossing takes compares a distance against a sum of two radii measured
/// in two different metrics, and clearing the square roots out of that
/// comparison costs eight degrees and runs past what `f64` can square.
///
/// **A torus against a plane is classified whole and named in four of its six
/// shapes** — see [`where_a_plane_and_a_torus_cross`] for the table. A torus
/// against a ball, a pipe, a cone or another torus is not named at all: the
/// crossing runs to degree eight and none of the machinery for that exists. Two
/// tori that hold the same numbers are the same surface and come back as one.
///
/// **A cone against a plane is classified whole and named in every one of its
/// seven shapes** — see [`where_a_plane_and_a_cone_cross`] for the table. **A
/// cone against a pipe is named where the two share an axis**, and that answer
/// is two circles and never anything else — see
/// [`where_a_cone_and_a_pipe_cross`]. Every other cone against a pipe, and a
/// cone against a ball or another cone, is not named at all: the crossing is a
/// degree-four space curve and none of the machinery §29 built for two pipes
/// exists for a cone. Two cones that hold the same numbers are the same surface
/// and come back as one.
///
/// # Errors
///
/// [`Error::Undecidable`] when a sign question could not be settled, carrying
/// the records. For the four planar pairs that only happens when a product would
/// overflow or fall into the subnormals — never because two surfaces were nearly
/// tangent, because level 3 decides tangency exactly (§27.1). Two leaning pipes
/// can also reach it on a genuine knife edge, where the enclosure straddles zero
/// and the exact level has nothing to offer (§29).
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when a crossing comes in two
/// pieces and no half-space that `f64` can write down provably holds them apart.
///
/// [`Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo`] when the two
/// surfaces are tangent along the curve rather than at a point, so what they
/// cross in is one curve that runs through itself.
///
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] for a cone against
/// anything curved except a pipe that shares its axis. **All seven shapes of a
/// plane against a cone are named**, including the parabola and the two branches
/// of the hyperbola, which used to be refused under names of their own.
///
/// [`Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic`] for every plane
/// against a torus but the two families that cut circles, and
/// [`Error::ACrossingBetweenATorusAndAnotherCurvedSurface`] for a torus against
/// anything curved.
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
        (Surface::Sphere(ball), Surface::Cylinder(pipe))
        | (Surface::Cylinder(pipe), Surface::Sphere(ball)) => {
            where_a_ball_and_a_pipe_cross(ball, pipe, policy, log)
        }
        (Surface::Cylinder(one), Surface::Cylinder(other)) => {
            where_two_pipes_cross(one, other, policy, log)
        }
        (Surface::Plane(flat), Surface::Cone(cone))
        | (Surface::Cone(cone), Surface::Plane(flat)) => {
            where_a_plane_and_a_cone_cross(flat, cone, policy, log)
        }
        (Surface::Cone(one), Surface::Cone(other)) if one == other => Ok(SurfaceMeet::SameSurface),
        (Surface::Cone(cone), Surface::Cylinder(pipe))
        | (Surface::Cylinder(pipe), Surface::Cone(cone)) => {
            where_a_cone_and_a_pipe_cross(cone, pipe, policy, log)
        }
        (Surface::Cone(_), Surface::Sphere(_) | Surface::Cone(_))
        | (Surface::Sphere(_), Surface::Cone(_)) => {
            Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface)
        }
        (Surface::Plane(flat), Surface::Torus(ring))
        | (Surface::Torus(ring), Surface::Plane(flat)) => {
            where_a_plane_and_a_torus_cross(flat, ring, policy, log)
        }
        (Surface::Torus(one), Surface::Torus(other)) if one == other => {
            Ok(SurfaceMeet::SameSurface)
        }
        (
            Surface::Torus(_),
            Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_) | Surface::Torus(_),
        )
        | (Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Cone(_), Surface::Torus(_)) => {
            Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface)
        }
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
        )?])),
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
            let reach = dot_enclosing(exactly(normal), exactly(centre)) - Interval::exactly(d);
            squared_length_enclosing(exactly(normal)) * squared_enclosing(radius) - reach * reach
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
        )?])),
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
        enclosing: || dot_enclosing(exactly(normal), exactly(along)),
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
            )?]));
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
            let reach = dot_enclosing(exactly(normal), exactly(origin)) - Interval::exactly(d);
            squared_length_enclosing(exactly(normal)) * squared_enclosing(radius)
                - squared_length_enclosing(exactly(along)) * reach * reach
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
        )?])),
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
/// # Why this can refuse, and which half of that refusal closed
///
/// The picking plane's normal has to be exactly across the axis direction, or a
/// *line* — endless — leaves the half-space somewhere far along itself, and a
/// run picked by a half-space it only stays in near the origin is not picked at
/// all. Every `Plane` scales its normal to length one and that division rounds,
/// so the normal we can write down is exactly square to the axis only sometimes.
///
/// Stage one asked for two more things exactly as well: that the picking normal
/// be square to the cutting plane's normal, and that the plane pass through the
/// axis origin with nothing rounded. **Neither has to be**, and §28.2 said so
/// and left the work here. Where the two lines sit relative to the picking plane
/// is a comparison between their own separation, `√(ρ² − g²)`, and how far that
/// plane strayed — a question with an answer rather than a condition to be
/// arranged. [`two_lines_told_apart`] asks it, and §29 records what that closed
/// and what stayed.
fn two_lines_along_a_pipe(flat: &Plane, pipe: &Cylinder) -> Result<SurfaceMeet> {
    let normal = flat.normal();
    let along = pipe.axis().direction();
    let origin = pipe.axis().origin();
    let picker = HalfSpace::through_a_point_square_to(origin, along, along.cross(normal))?;

    // The plane runs exactly along the axis — the caller decided that — so the
    // two directions across the pipe are the plane's own normal and `w × n`,
    // each scaled to length one.
    let squared_direction = squared_length_enclosing(exactly(along.components()));
    let squared_normal = squared_length_enclosing(exactly(normal.components()));
    let sideways = exactly(normal.components())
        .map(|part| quotient_or_everything(part, root_enclosing(squared_normal)));
    let up =
        cross_enclosing(exactly(along.components()), exactly(normal.components())).map(|part| {
            quotient_or_everything(part, root_enclosing(squared_direction * squared_normal))
        });

    // How far the axis sits in front of the cutting plane, and how far each of
    // the two lines sits from the plane through the axis square to it.
    let in_front = quotient_or_everything(
        dot_enclosing(exactly(normal.components()), exactly(origin.coordinates()))
            - Interval::exactly(flat.offset()),
        root_enclosing(squared_normal),
    );
    let skin = quotient_or_everything(
        Interval::exactly(pipe.radius()),
        root_enclosing(squared_direction),
    );
    let out = root_enclosing(skin * skin - in_front * in_front);

    let on_each_line = [1.0, -1.0].map(|side| {
        core::array::from_fn(|axis| {
            Interval::exactly(origin.coordinates()[axis]) - sideways[axis] * in_front
                + up[axis] * out * Interval::exactly(side)
        })
    });
    two_lines_told_apart(
        picker,
        along.components(),
        on_each_line,
        Surface::Cylinder(*pipe),
    )
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
            squared_length_enclosing(gap_enclosing(here, there)) - reach * reach
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
        )?])),
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

/// A ball and a pipe: nothing, one touching point, one closed loop, or two.
///
/// The crossing is a degree-four space curve and this names which of those it
/// is without drawing any of it. Write `A = ‖(c−o)×w‖²` for how far the ball's
/// centre sits off the axis, `W = w·w`, and
///
/// ```text
/// V  = A + r² − W·R²          the eighth-degree discriminant's inner half
/// Δ  = V² − 4r²A              its own discriminant
/// ```
///
/// Then `W(R² − (ρ∓a)²) = −V ± 2r√A`, where `ρ = r/√W` is the pipe's certified
/// radius and `a = √(A/W)` the centre's distance off the axis. The root is what
/// stops those two being polynomials on their own; `Δ` is their product, and the
/// pair `(sign V, sign Δ)` settles both signs without ever taking it (§27.1).
///
/// The five answers: the ball misses the pipe, rests on it at one point, cuts it
/// in one loop, cuts it in two, or — where the two are tangent *along* the
/// crossing — leaves one loop that runs through itself, which is the one shape
/// this vocabulary cannot say.
fn where_a_ball_and_a_pipe_cross(
    ball: &Sphere,
    pipe: &Cylinder,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let centre = ball.centre().coordinates();
    let along = pipe.axis().direction().components();
    let origin = pipe.axis().origin().coordinates();
    let skin = pipe.radius();
    let at = [centre, origin].map(PredicatePoint::from);

    let off_the_axis = Question {
        named: WHERE_A_BALL_AND_A_PIPE_CROSS,
        enclosing: || how_far_off_an_axis_enclosing(centre, origin, along),
        exactly: || how_far_off_an_axis_exactly(centre, origin, along),
    };
    match ask(&off_the_axis, Measures::Area, &at, policy, log)? {
        Decision::Zero => return a_ball_centred_on_a_pipes_axis(ball, pipe, policy, log),
        Decision::Pos => (),
        other => unreachable!("a sum of squares is never below zero: {other:?}"),
    }

    let inner = Question {
        named: WHERE_A_BALL_AND_A_PIPE_CROSS,
        enclosing: || how_far_a_ball_reaches_past_a_pipe_enclosing(ball, pipe),
        exactly: || how_far_a_ball_reaches_past_a_pipe_exactly(ball, pipe),
    };
    let edge = Question {
        named: WHERE_A_BALL_AND_A_PIPE_CROSS,
        enclosing: || {
            let room = how_far_a_ball_reaches_past_a_pipe_enclosing(ball, pipe);
            room * room
                - Interval::exactly(4.0)
                    * squared_enclosing(skin)
                    * how_far_off_an_axis_enclosing(centre, origin, along)
        },
        exactly: || {
            let room = how_far_a_ball_reaches_past_a_pipe_exactly(ball, pipe)?;
            Some(
                room.times(&room)?.minus(
                    &Exact::product_of(2.0, 2.0)?
                        .times(&Exact::product_of(skin, skin)?)?
                        .times(&how_far_off_an_axis_exactly(centre, origin, along)?)?,
                ),
            )
        },
    };

    let four_lengths = Measures::ManyLengths { how_many: 4 };
    let room = ask(&inner, Measures::Area, &at, policy, log)?;
    let apart = ask(&edge, four_lengths, &at, policy, log)?;
    match (room, apart) {
        // The nearer of the two tangency discriminants is below zero, so the
        // ball never reaches the skin.
        (Decision::Pos, Decision::Pos) => Ok(SurfaceMeet::Nothing),
        (Decision::Pos, Decision::Zero) => Ok(SurfaceMeet::TouchAtOnePoint {
            at: where_a_ball_rests_against_a_pipe(ball, pipe),
        }),
        (Decision::Pos | Decision::Zero | Decision::Neg, Decision::Neg)
        | (Decision::Zero, Decision::Pos | Decision::Zero) => {
            Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
                EdgeRun::RightRoundWhereItsFacesCross,
            )?]))
        }
        // Both discriminants above zero: the ball swallows a whole cross-section
        // of the pipe, so what is left is two closed loops, one either side of
        // the plane through the centre square to the axis.
        (Decision::Neg, Decision::Pos) => {
            two_loops_either_side_of_a_ball(ball, pipe, how_far_apart_two_loops_are(ball, pipe))
        }
        // The two loops touch. What they make is one curve that runs through
        // itself, and neither "one closed curve" nor "two pieces" says that.
        (Decision::Neg, Decision::Zero) => {
            Err(Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo)
        }
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// A ball whose centre is exactly on the pipe's axis: two circles, one circle
/// where the ball is inscribed in the pipe, or nothing.
///
/// **This is the planar special case of the ball-and-pipe row** (§24.5), and it
/// is where the marcher gets an answer it can be differenced against: both
/// circles lie in planes square to the axis, at `±√(R² − ρ²)` from the centre,
/// and each has the pipe's own radius.
fn a_ball_centred_on_a_pipes_axis(
    ball: &Sphere,
    pipe: &Cylinder,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let radius = ball.radius();
    let along = pipe.axis().direction().components();
    let skin = pipe.radius();
    let at = [ball.centre(), pipe.axis().origin()]
        .map(|place| PredicatePoint::from(place.coordinates()));

    // `W·R² − r²`, an area: the ball's radius against the pipe's, each in the
    // metric its own stored numbers name.
    let room = Question {
        named: WHERE_A_BALL_AND_A_PIPE_CROSS,
        enclosing: || {
            squared_length_enclosing(exactly(along)) * squared_enclosing(radius)
                - squared_enclosing(skin)
        },
        exactly: || {
            Some(
                Exact::dot_of(along, along)?
                    .times(&Exact::product_of(radius, radius)?)?
                    .minus(&Exact::product_of(skin, skin)?),
            )
        },
    };
    match ask(&room, Measures::Area, &at, policy, log)? {
        Decision::Pos => {
            two_loops_either_side_of_a_ball(ball, pipe, how_far_apart_two_loops_are(ball, pipe))
        }
        // The ball is inscribed in the pipe and they touch all the way round one
        // circle. Tangential, and still a curve — the same shape as a plane
        // resting along a pipe answering with one line (§24.10).
        Decision::Zero => Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
            EdgeRun::RightRoundWhereItsFacesCross,
        )?])),
        Decision::Neg => Ok(SurfaceMeet::Nothing),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// How far, along the axis, the nearer of two loops keeps away from the plane
/// through the ball's centre.
///
/// `√(D₋/W)` with `D₋ = W·R² − r² − A − 2r√A`, which the caller has already
/// decided is above zero. An enclosure and not a number, because the root is
/// taken twice and neither lands on an `f64`.
fn how_far_apart_two_loops_are(ball: &Sphere, pipe: &Cylinder) -> Interval {
    let centre = ball.centre().coordinates();
    let along = pipe.axis().direction().components();
    let origin = pipe.axis().origin().coordinates();
    let off_the_axis = how_far_off_an_axis_enclosing(centre, origin, along);
    let squared_direction = squared_length_enclosing(exactly(along));
    let reach = squared_direction * squared_enclosing(ball.radius())
        - squared_enclosing(pipe.radius())
        - off_the_axis
        - Interval::exactly(2.0) * Interval::exactly(pipe.radius()) * root_enclosing(off_the_axis);
    root_enclosing(quotient_or_everything(reach, squared_direction))
}

/// Two closed pieces of a ball-and-pipe crossing, told apart by the plane
/// through the ball's centre square to the pipe's axis.
///
/// # Testing the separation instead of arranging it
///
/// §28.2 left a gap: a piece of a crossing picked out by a half-space needs that
/// half-space written down in `f64`, and `Plane` scales every normal to length
/// one, so the plane we can store is a rounding of the one that separates. Stage
/// one asked for the rounding to be *exactly* nothing and refused otherwise.
///
/// Here it does not have to be, because **both pieces are bounded**: every point
/// of the crossing is exactly `R` from the ball's centre. So the value the
/// half-space reads splits into a part along the axis, which is at least
/// `reach` and has the sign we want, and a part across it, which cannot be more
/// than `‖n̂ × ŵ‖·R` however badly the normal rounded. When the first beats the
/// second plus the offset's own rounding, the plane provably holds the two
/// pieces apart — and that is a proof about the *stored* plane rather than a
/// hope about the one it was rounded from.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when it does not beat it,
/// which happens where the two loops nearly touch. A **gap**: the loops are
/// there and they are tellable apart, and what runs out is how well an `f64`
/// unit normal can copy a direction.
fn two_loops_either_side_of_a_ball(
    ball: &Sphere,
    pipe: &Cylinder,
    reach: Interval,
) -> Result<SurfaceMeet> {
    let along = pipe.axis().direction();
    let between = HalfSpace::through_a_point_facing(ball.centre(), along)?;
    let picker = between.normal().components();
    let along = along.components();
    let squared_direction = squared_length_enclosing(exactly(along));

    // How much of the picking half-space's own normal runs along the axis, and
    // how much of it does not. **The second is exactly nothing**, because the
    // normal *is* the axis direction — a half-space stores what it is handed —
    // and only the exact cross product says so. The same two numbers taken as
    // `√(‖n‖² − (n·ŵ)²)` cancel to a few last bits whose square root is eight
    // orders of magnitude bigger than the bits it came from, which is what a
    // plane's rounded normal used to be charged (§33).
    let up_the_axis = quotient_or_everything(
        dot_enclosing(exactly(picker), exactly(along)),
        root_enclosing(squared_direction),
    );
    let across = quotient_or_everything(
        root_enclosing(squared_sideways_enclosing(picker, along)),
        root_enclosing(squared_direction),
    );
    let off_the_plane = dot_enclosing(exactly(picker), exactly(ball.centre().coordinates()))
        - Interval::exactly(between.offset());
    let offset_slop = off_the_plane.lo().abs().max(off_the_plane.hi().abs());

    let clear = up_the_axis * reach
        - across * Interval::exactly(ball.radius())
        - Interval::exactly(offset_slop);
    if clear.lo() <= 0.0 {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    let pipe = Surface::Cylinder(*pipe);
    Ok(SurfaceMeet::Curves(
        [between, between.reversed()]
            .into_iter()
            .map(|half_space| {
                NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                    and: pipe,
                    picked_by: smallvec![Picker::from(half_space)],
                })
                .expect("picked_by always holds the one half-space just built")
            })
            .collect(),
    ))
}

/// `‖u × v‖²`, summed exactly where the arithmetic reaches it and enclosed where
/// it does not.
///
/// The exact sum is what makes a normal written along a direction readable as
/// being along it. Two vectors that really are the same triple have a cross
/// product of exactly zero, and an interval enclosure of the same expression
/// cannot cancel: each component comes back a couple of last bits wide, and the
/// square root of that is eight orders of magnitude larger than the width it
/// came from.
fn squared_sideways_enclosing(u: [f64; 3], v: [f64; 3]) -> Interval {
    match Exact::cross_of(&Exact::of_each(u), v).and_then(|cross| Exact::squared_length_of(&cross))
    {
        Some(exact) => exact.enclosure(),
        None => squared_length_enclosing(cross_enclosing(exactly(u), exactly(v))),
    }
}

/// The one point a ball resting against a pipe shares with it, as the box it is
/// known to be inside.
///
/// The contact is on the line from the axis out to the centre: `o + hŵ + ρê`,
/// with `h` how far along the axis the centre sits and `ê` the unit direction
/// from the axis to it. Both roots come out of numbers the caller has already
/// decided are above zero.
fn where_a_ball_rests_against_a_pipe(ball: &Sphere, pipe: &Cylinder) -> PredicatePoint {
    let centre = ball.centre().coordinates();
    let along = pipe.axis().direction().components();
    let origin = pipe.axis().origin().coordinates();

    let squared_direction = squared_length_enclosing(exactly(along));
    let direction =
        exactly(along).map(|part| quotient_or_everything(part, root_enclosing(squared_direction)));
    let from_the_origin = gap_enclosing(origin, centre);
    let up_the_axis = dot_enclosing(from_the_origin, direction);
    let sideways: [Interval; 3] =
        core::array::from_fn(|axis| from_the_origin[axis] - direction[axis] * up_the_axis);
    let off_the_axis = root_enclosing(squared_length_enclosing(sideways));
    let skin = quotient_or_everything(
        Interval::exactly(pipe.radius()),
        root_enclosing(squared_direction),
    );

    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        Interval::exactly(origin[axis])
            + direction[axis] * up_the_axis
            + quotient_or_everything(sideways[axis], off_the_axis) * skin
    }))
}

/// `V = A + r² − W·R²` as an enclosure.
fn how_far_a_ball_reaches_past_a_pipe_enclosing(ball: &Sphere, pipe: &Cylinder) -> Interval {
    let along = pipe.axis().direction().components();
    how_far_off_an_axis_enclosing(
        ball.centre().coordinates(),
        pipe.axis().origin().coordinates(),
        along,
    ) + squared_enclosing(pipe.radius())
        - squared_length_enclosing(exactly(along)) * squared_enclosing(ball.radius())
}

/// `V = A + r² − W·R²`, exactly.
fn how_far_a_ball_reaches_past_a_pipe_exactly(ball: &Sphere, pipe: &Cylinder) -> Option<Exact> {
    let along = pipe.axis().direction().components();
    let radius = ball.radius();
    let skin = pipe.radius();
    Some(
        how_far_off_an_axis_exactly(
            ball.centre().coordinates(),
            pipe.axis().origin().coordinates(),
            along,
        )?
        .plus(&Exact::product_of(skin, skin)?)
        .minus(&Exact::dot_of(along, along)?.times(&Exact::product_of(radius, radius)?)?),
    )
}

/// `‖(c − o) × w‖²` as an enclosure.
fn how_far_off_an_axis_enclosing(centre: [f64; 3], origin: [f64; 3], along: [f64; 3]) -> Interval {
    squared_length_enclosing(cross_enclosing(
        gap_enclosing(origin, centre),
        exactly(along),
    ))
}

/// `‖(c − o) × w‖²`, exactly.
fn how_far_off_an_axis_exactly(
    centre: [f64; 3],
    origin: [f64; 3],
    along: [f64; 3],
) -> Option<Exact> {
    Exact::squared_length_of(&Exact::cross_of(
        &Exact::gap_between(origin, centre),
        along,
    )?)
}

/// Two pipes: parallel axes are named exactly, and leaning ones are named as
/// far as an enclosure reaches.
///
/// The split is `‖w₁ × w₂‖²`, which is zero exactly when the two axes point
/// along one line.
fn where_two_pipes_cross(
    one: &Cylinder,
    other: &Cylinder,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let directions = [one, other].map(|pipe| pipe.axis().direction().components());
    let at = [one, other].map(|pipe| PredicatePoint::from(pipe.axis().origin().coordinates()));
    let leaning =
        squared_length_of_a_cross_product(WHERE_TWO_PIPES_CROSS, directions[0], directions[1]);
    // Two unit directions crossed together is dimensionless, and filed as a
    // length for the reason `where_a_plane_and_a_pipe_cross` gives.
    match ask(&leaning, Measures::Length, &at, policy, log)? {
        Decision::Zero => where_two_pipes_side_by_side_cross(one, other, policy, log),
        Decision::Pos => where_two_leaning_pipes_cross(one, other, policy, log),
        other => unreachable!("a sum of squares is never below zero: {other:?}"),
    }
}

/// Two pipes whose axes run along one direction: the same pipe, nothing, one
/// line, or two.
///
/// **The planar special case of the pipe-and-pipe row** (§24.5). Every answer is
/// straight, so it is decided by the twelfth-degree discriminant §27.1 wrote
/// down and nothing here has to be drawn.
///
/// Write `A = ‖(o₂−o₁) × w₁‖²` and `E = A·W₂ − r₁²W₂ − r₂²W₁`, which is
/// `W₁W₂(δ² − ρ₁² − ρ₂²)` for the true distance `δ` between the axes. Then
/// `E² − 4r₁²r₂²W₁W₂` is `(W₁W₂)²(δ²−(ρ₁+ρ₂)²)(δ²−(ρ₁−ρ₂)²)`, and its sign
/// alone settles the answer: below zero the two circles of the cross-section
/// cut each other twice, at zero they rest against each other, above zero one
/// is clear of the other or swallowed by it and either way they never meet.
fn where_two_pipes_side_by_side_cross(
    one: &Cylinder,
    other: &Cylinder,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let [here, there] = [one, other].map(|pipe| pipe.axis().origin().coordinates());
    let [mine, theirs] = [one, other].map(Cylinder::radius);
    let [my_way, their_way] = [one, other].map(|pipe| pipe.axis().direction().components());
    let at = [here, there].map(PredicatePoint::from);
    let four_lengths = Measures::ManyLengths { how_many: 4 };

    let apart = Question {
        named: WHERE_TWO_PIPES_CROSS,
        enclosing: || how_far_off_an_axis_enclosing(there, here, my_way),
        exactly: || how_far_off_an_axis_exactly(there, here, my_way),
    };
    let same_thickness = Question {
        named: WHERE_TWO_PIPES_CROSS,
        enclosing: || {
            squared_enclosing(mine) * squared_length_enclosing(exactly(their_way))
                - squared_enclosing(theirs) * squared_length_enclosing(exactly(my_way))
        },
        exactly: || {
            Some(
                Exact::product_of(mine, mine)?
                    .times(&Exact::dot_of(their_way, their_way)?)?
                    .minus(
                        &Exact::product_of(theirs, theirs)?
                            .times(&Exact::dot_of(my_way, my_way)?)?,
                    ),
            )
        },
    };

    match ask(&apart, Measures::Area, &at, policy, log)? {
        // One axis, so the two are the same pipe when their certified radii
        // agree and never meet at all when they do not.
        Decision::Zero => match ask(&same_thickness, four_lengths, &at, policy, log)? {
            Decision::Zero => Ok(SurfaceMeet::SameSurface),
            Decision::Pos | Decision::Neg => Ok(SurfaceMeet::Nothing),
            other => unreachable!("a sign question answers with a sign: {other:?}"),
        },
        Decision::Pos => {
            let edge = Question {
                named: WHERE_TWO_PIPES_CROSS,
                enclosing: || how_deeply_two_side_by_side_pipes_overlap_enclosing(one, other),
                exactly: || how_deeply_two_side_by_side_pipes_overlap_exactly(one, other),
            };
            match ask(&edge, four_lengths, &at, policy, log)? {
                Decision::Neg => two_lines_between_two_pipes(one, other),
                Decision::Zero => Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
                    EdgeRun::Straight,
                )?])),
                Decision::Pos => Ok(SurfaceMeet::Nothing),
                other => unreachable!("a sign question answers with a sign: {other:?}"),
            }
        }
        other => unreachable!("a sum of squares is never below zero: {other:?}"),
    }
}

/// The two endless lines two side-by-side pipes cut out of each other.
///
/// They sit either side of the plane that holds both axes, so that plane is what
/// tells them apart, and each line's own distance `X` from it is the only thing
/// that has to be worked out well: `across` places both lines along the line
/// between the axes, which the picking plane reads as nothing.
///
/// # Why `X` is not `√(ρ₁² − across²)`
///
/// It is, as arithmetic, and that subtraction is where the row used to give up on
/// a pair it had already decided. The two pipes' crossing has been settled by
/// this point — the caller reached here because the discriminant came out below
/// zero — and then naming the two halves of it could fail, which is §16.10's gap
/// and not a limit.
///
/// Write `δ` for the distance between the axes and `ρᵢ` for the certified radii.
/// A thin pipe whose axis lies on a fat one's skin has `δ = ρ₁` and `across =
/// ρ₁ − ρ₂²/(2ρ₁)`, so `ρ₁² − across²` is `ρ₂²` arrived at by subtracting two
/// numbers that agree to `ρ₂²/ρ₁²` of themselves. Past `ρ₁ ≈ 2²⁶·ρ₂` — a 67 m
/// pipe against a 1 m one — `f64` has no bits left to hold the answer in, the
/// enclosure straddles zero, and the two lines cannot be shown to be either side
/// of anything. The same pair asked the other way round has `across ≈ 0` and
/// decides, which is what made it an order-dependent answer.
///
/// So `X` is taken from the discriminant instead, which the caller has already
/// found the sign of. Heron's factorisation of `4δ²X² = 4δ²ρ₁² −
/// (δ²+ρ₁²−ρ₂²)²` is `−(δ²−(ρ₁+ρ₂)²)(δ²−(ρ₁−ρ₂)²)`, and that product is the
/// discriminant divided by `(W₁W₂)²`. With `δ² = A/W₁` that comes out as
///
/// ```text
/// X²  =  −(E² − 4r₁²r₂²W₁W₂)  /  (4·A·W₁·W₂²)
/// ```
///
/// in the stored numbers alone, with no subtraction of two nearly equal large
/// numbers anywhere in it. The numerator is the same expression the shape
/// question asked, so a pair decided to cross in two lines is a pair whose two
/// lines can be told apart, and the two steps cannot disagree.
fn two_lines_between_two_pipes(one: &Cylinder, other: &Cylinder) -> Result<SurfaceMeet> {
    let here = one.axis().origin();
    let along = one.axis().direction();
    let squared_direction = squared_length_enclosing(exactly(along.components()));
    let squared_theirs_direction =
        squared_length_enclosing(exactly(other.axis().direction().components()));
    let between = other.axis().origin() - here;
    let towards = between - along * (between.dot(along) / along.length_squared());

    let picker = HalfSpace::through_a_point_square_to(here, along, along.cross(towards))?;
    let sideways = exactly(towards.components());
    let apart = root_enclosing(squared_length_enclosing(sideways));
    let sideways = sideways.map(|part| quotient_or_everything(part, apart));
    let up = cross_enclosing(exactly(along.components()), sideways)
        .map(|part| quotient_or_everything(part, root_enclosing(squared_direction)));

    let [mine, theirs] = [one, other].map(|pipe| {
        quotient_or_everything(
            Interval::exactly(pipe.radius()),
            root_enclosing(squared_length_enclosing(exactly(
                pipe.axis().direction().components(),
            ))),
        )
    });
    let across = quotient_or_everything(
        apart * apart + mine * mine - theirs * theirs,
        Interval::exactly(2.0) * apart,
    );
    let there = other.axis().origin().coordinates();
    let overlap = how_deeply_two_side_by_side_pipes_overlap_exactly(one, other)
        .as_ref()
        .map_or_else(
            || how_deeply_two_side_by_side_pipes_overlap_enclosing(one, other),
            Exact::enclosure,
        );
    let squared_gap = how_far_off_an_axis_exactly(there, here.coordinates(), along.components())
        .as_ref()
        .map_or_else(
            || how_far_off_an_axis_enclosing(there, here.coordinates(), along.components()),
            Exact::enclosure,
        );
    let out = root_enclosing(quotient_or_everything(
        -overlap,
        Interval::exactly(4.0)
            * squared_gap
            * squared_direction
            * squared_theirs_direction
            * squared_theirs_direction,
    ));

    let on_each_line = [1.0, -1.0].map(|side| {
        core::array::from_fn(|axis| {
            Interval::exactly(here.coordinates()[axis])
                + sideways[axis] * across
                + up[axis] * out * Interval::exactly(side)
        })
    });
    two_lines_told_apart(
        picker,
        along.components(),
        on_each_line,
        Surface::Cylinder(*other),
    )
}

/// Two pipes leaning against each other: nothing, one touching point, one closed
/// loop, or two.
///
/// # Where the four answers come from
///
/// Walk the first pipe's cross-section round by an angle and ask how far along
/// its own axis the second pipe reaches. That is a quadratic in the distance
/// along, and its discriminant comes out as
///
/// ```text
/// sin²α · [ ρ₂² − (δ + ρ₁·cos θ)² ]
/// ```
///
/// with `α` the angle between the axes, `δ` the distance between them and `ρᵢ`
/// the certified radii. So the whole shape of the crossing is decided by where
/// `ρ₂` sits against `δ ± ρ₁`: above both, the second pipe swallows the first's
/// whole cross-section and leaves two closed loops; between them, one; below
/// both, two again with the pipes' roles the other way round; and past `δ − ρ₁`
/// on the far side, nothing at all.
///
/// # Why this is where the exact level runs out
///
/// Those three comparisons are between a sum of two radii and a distance,
/// measured in three different metrics: `ρᵢ = rᵢ/√Wᵢ` and `δ = |T|/√K` for
/// polynomials `T` and `K` in the stored numbers. Clearing the roots costs eight
/// degrees, and `docs/decisions.md` §29 measures what happens then — the
/// expansion that has to be squared is already five hundred binary orders wide
/// for two ordinary pipes, and squaring it falls off the bottom of `f64`.
///
/// What is here instead: the comparison is *enclosed*, which decides it wherever
/// the two sides are not on a knife edge, and the exact level runs where the two
/// stored directions have exactly the same length — which is every pair whose
/// axes are coordinate directions, and so the whole of §24.18's control.
fn where_two_leaning_pipes_cross(
    one: &Cylinder,
    other: &Cylinder,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let at = [one, other].map(|pipe| PredicatePoint::from(pipe.axis().origin().coordinates()));
    let reach = |mine: f64, theirs: f64, gap: f64| Question {
        named: WHERE_TWO_PIPES_CROSS,
        enclosing: move || {
            let [first, second] = certified_radii_enclosing(one, other);
            first * Interval::exactly(mine)
                + second * Interval::exactly(theirs)
                + how_far_apart_two_axes_are_enclosing(one, other) * Interval::exactly(gap)
        },
        exactly: move || how_two_radii_and_a_gap_compare_exactly(one, other, mine, theirs, gap),
    };

    // ρ₁ + ρ₂ − δ. Below zero the two are clear of each other.
    match ask(&reach(1.0, 1.0, -1.0), Measures::Length, &at, policy, log)? {
        Decision::Neg => return Ok(SurfaceMeet::Nothing),
        Decision::Zero => {
            return Ok(SurfaceMeet::TouchAtOnePoint {
                at: where_two_leaning_pipes_rest_against_each_other(one, other),
            });
        }
        Decision::Pos => (),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }

    // ρ₂ − ρ₁ − δ. Above zero the second pipe takes the whole of the first's
    // cross-section and what is left is two loops.
    match ask(&reach(-1.0, 1.0, -1.0), Measures::Length, &at, policy, log)? {
        Decision::Pos => return two_loops_either_side_of_a_pipe(one, other),
        Decision::Zero => return Err(Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo),
        Decision::Neg => (),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }

    // ρ₂ − ρ₁ + δ, which is the row above with the two pipes the other way
    // round and the sign of the gap flipped.
    match ask(&reach(-1.0, 1.0, 1.0), Measures::Length, &at, policy, log)? {
        Decision::Pos => Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
            EdgeRun::RightRoundWhereItsFacesCross,
        )?])),
        Decision::Zero => Err(Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo),
        Decision::Neg => two_loops_either_side_of_a_pipe(other, one),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// Two closed loops of a leaning-pipe crossing, told apart by the plane through
/// the fat pipe's axis that the thin one's direction is square to.
///
/// The separating plane is `(ŵ₁ − (ŵ₁·ŵ₂)ŵ₂) · (p − o₂) = 0` — the part of the
/// thin pipe's direction that lies across the fat one — and every point of the
/// crossing sits at least `√(ρ₂² − (δ + ρ₁)²)` from it.
///
/// # The normal is written square, and is not the direction that was asked for
///
/// A loop reaches a long way along the fat pipe's axis, so a picking normal that
/// leans into that axis by even one rounding reads a different number at either
/// end of the loop and picks nothing. The squareness has to be *arranged* and
/// cannot be tested against, because there is no bound on how far along the axis
/// a loop reaches (§29.8).
///
/// **The separating direction is usually not one a caller can write down square**
/// — the ones that can be are that direction crossed with each coordinate axis,
/// plus the asked-for one itself where it happens to land square, which every
/// pair whose axes are coordinate directions does. It does not have to be. What
/// separates the two loops is any half-space square to the axis with enough of
/// its normal along that direction, and the best of the three is at least
/// `1/√3` aligned with it. What that costs is a real number and it is charged
/// below.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the best of the three
/// does not lean far enough. Writing `n̂ = α·â + β·b̂` in the two directions
/// across the axis, a loop point sits `a` along `â` and `b` along `b̂` with
/// `a² + b² = ρ₂²`, and it is at least `√(ρ₂² − (δ + ρ₁)²)` along `â` — so `|b|`
/// is at most `δ + ρ₁` and the reading is at least `α·√(ρ₂² − (δ+ρ₁)²) − |β|·(δ + ρ₁)`.
/// Still a **gap** where that does not clear the offset's own rounding: the two
/// loops are there and they are tellable apart.
fn two_loops_either_side_of_a_pipe(thin: &Cylinder, fat: &Cylinder) -> Result<SurfaceMeet> {
    let (thin_way, fat_way) = (
        thin.axis().direction().components(),
        fat.axis().direction().components(),
    );
    let leaning = Vector::from(thin_way).dot(Vector::from(fat_way));
    let squared_fat = Vector::from(fat_way).length_squared();
    let across = Vector::from(thin_way) * squared_fat - Vector::from(fat_way) * leaning;
    let between =
        HalfSpace::through_a_point_square_to(fat.axis().origin(), fat.axis().direction(), across)?;
    let picker = between.normal().components();

    // Arranged rather than tested, and a half-space keeps what it was handed —
    // so this only fails where a product of two of the axis's own components
    // lands among the subnormals and the exact level cannot run at all.
    if !matches!(
        Exact::dot_of(picker, fat_way).as_ref().map(Exact::sign),
        Some(Decision::Zero)
    ) {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    let [thin_radius, fat_radius] = certified_radii_enclosing(thin, fat);
    let apart = how_far_apart_two_axes_are_enclosing(thin, fat);
    let reach = apart + thin_radius;
    let clear = root_enclosing(fat_radius * fat_radius - reach * reach);
    let (leaning_into, leaning_away) =
        how_a_picking_half_space_lies_across_an_axis(&between, thin, fat);
    let margin = leaning_into * clear
        - leaning_away * reach
        - how_far_a_picking_half_space_sits_off_a_point(&between, fat.axis().origin());
    if margin.lo() <= 0.0 {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    let fat = Surface::Cylinder(*fat);
    Ok(SurfaceMeet::Curves(
        [between, between.reversed()]
            .into_iter()
            .map(|half_space| {
                NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                    and: fat,
                    picked_by: smallvec![Picker::from(half_space)],
                })
                .expect("picked_by always holds the one half-space just built")
            })
            .collect(),
    ))
}

/// How much of a picking half-space's normal lies along the direction that tells
/// two loops of a leaning-pipe crossing apart, and how much of it does not.
///
/// The separating direction is the part of the thin pipe's direction that lies
/// across the fat one, `t‖w‖² − w(t·w)`. Two things make it cheap to measure
/// against: the normal's dot with it is just `‖w‖²(n·t)`, because `n·w` is
/// exactly zero, and its own length is `‖w‖·‖t × w‖`. So the first number is
///
/// ```text
/// α = ‖w‖·(n·t) / (‖n‖·‖t × w‖)
/// ```
///
/// Both the normal and the separating direction are exactly square to the fat
/// axis, so the two live in one plane and the second number is `√(1 − α²)`.
fn how_a_picking_half_space_lies_across_an_axis(
    picker: &HalfSpace,
    thin: &Cylinder,
    fat: &Cylinder,
) -> (Interval, Interval) {
    let normal = picker.normal().components();
    let thin_way = thin.axis().direction().components();
    let fat_way = fat.axis().direction().components();

    let along = quotient_or_everything(
        root_enclosing(squared_length_enclosing(exactly(fat_way)))
            * dot_enclosing(exactly(normal), exactly(thin_way)),
        root_enclosing(squared_length_enclosing(exactly(normal)))
            * root_enclosing(squared_sideways_enclosing(thin_way, fat_way)),
    );
    // A cosine is between minus one and one whatever the arithmetic says, and
    // the whole enclosure being everything is what an underflowed length looks
    // like. Both come out as a margin that does not clear, which is a refusal.
    let leaning_into = along.lo().clamp(-1.0, 1.0);
    let leaning_away = root_enclosing(
        Interval::exactly(1.0) - Interval::exactly(leaning_into) * Interval::exactly(leaning_into),
    );
    (
        Interval::exactly(leaning_into),
        Interval::exactly(leaning_away.hi()),
    )
}

/// The one point two leaning pipes resting against each other share, as the box
/// it is known to be inside.
///
/// It is on the common perpendicular, `ρ₁` along it from the first axis.
fn where_two_leaning_pipes_rest_against_each_other(
    one: &Cylinder,
    other: &Cylinder,
) -> PredicatePoint {
    let here = one.axis().origin().coordinates();
    let there = other.axis().origin().coordinates();
    let [my_way, their_way] = [one, other].map(|pipe| pipe.axis().direction().components());
    let [mine, _] = certified_radii_enclosing(one, other);

    let ways = [my_way, their_way].map(|way| {
        let squared = squared_length_enclosing(exactly(way));
        exactly(way).map(|part| quotient_or_everything(part, root_enclosing(squared)))
    });
    let sideways = cross_enclosing(ways[0], ways[1]);
    let sideways = {
        let length = root_enclosing(squared_length_enclosing(sideways));
        sideways.map(|part| quotient_or_everything(part, length))
    };
    let between = gap_enclosing(here, there);
    let gap = dot_enclosing(between, sideways);

    // How far up the first axis the common perpendicular meets it.
    let leaning = dot_enclosing(ways[0], ways[1]);
    let up = quotient_or_everything(
        dot_enclosing(between, ways[0]) - dot_enclosing(between, ways[1]) * leaning,
        Interval::exactly(1.0) - leaning * leaning,
    );
    let towards = if gap.lo() + gap.hi() >= 0.0 {
        1.0
    } else {
        -1.0
    };
    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        Interval::exactly(here[axis])
            + ways[0][axis] * up
            + sideways[axis] * mine * Interval::exactly(towards)
    }))
}

/// The two certified radii, `rᵢ / √(wᵢ·wᵢ)`, as enclosures.
fn certified_radii_enclosing(one: &Cylinder, other: &Cylinder) -> [Interval; 2] {
    [one, other].map(|pipe| {
        let along = pipe.axis().direction().components();
        let squared = Exact::dot_of(along, along).map_or_else(
            || squared_length_enclosing(exactly(along)),
            |exact| exact.enclosure(),
        );
        quotient_or_everything(Interval::exactly(pipe.radius()), root_enclosing(squared))
    })
}

/// `δ = |T| / √K`, the distance between two leaning axes, as an enclosure.
///
/// `T = (o₂−o₁)·(w₁×w₂)` and `K = ‖w₁×w₂‖²`, both polynomials in the stored
/// numbers. The caller has already decided `K` is above zero.
fn how_far_apart_two_axes_are_enclosing(one: &Cylinder, other: &Cylinder) -> Interval {
    let [here, there] = [one, other].map(|pipe| pipe.axis().origin().coordinates());
    let [my_way, their_way] = [one, other].map(|pipe| pipe.axis().direction().components());

    // **Both of these are summed exactly before they are enclosed**, and the
    // difference matters more here than anywhere else in the file. Two axes a
    // rounding away from parallel have a cross product whose every component
    // cancelled to nothing, so an interval enclosure of its squared length
    // reaches down to zero and the division below comes back as everything.
    // Summed through the expansion first, the same number is known to full
    // relative accuracy and two nearly-parallel pipes are as decidable as any
    // other pair (§29).
    let (volume, squared_sideways) = match exactly_how_far_apart_two_axes_are(one, other) {
        Some(both) => both,
        None => {
            let sideways = cross_enclosing(exactly(my_way), exactly(their_way));
            (
                dot_enclosing(gap_enclosing(here, there), sideways),
                squared_length_enclosing(sideways),
            )
        }
    };
    let size = Interval::new(
        volume.lo().abs().min(volume.hi().abs()),
        volume.lo().abs().max(volume.hi().abs()),
    )
    .expect("a magnitude runs from its nearer end to its further one");
    let size = if volume.contains_zero() {
        Interval::new(0.0, size.hi()).expect("a magnitude runs from zero upwards")
    } else {
        size
    };
    quotient_or_everything(size, root_enclosing(squared_sideways))
}

/// `T = (o₂ − o₁)·(w₁ × w₂)` and `K = ‖w₁ × w₂‖²`, summed exactly and then
/// enclosed, or `None` where a product would round.
fn exactly_how_far_apart_two_axes_are(
    one: &Cylinder,
    other: &Cylinder,
) -> Option<(Interval, Interval)> {
    let [here, there] = [one, other].map(|pipe| pipe.axis().origin().coordinates());
    let [my_way, their_way] = [one, other].map(|pipe| pipe.axis().direction().components());
    let sideways = Exact::cross_of(&Exact::of_each(my_way), their_way)?;
    let volume = (0..3).try_fold(Exact::of(0.0), |so_far, axis| {
        Some(
            so_far.plus(
                &Exact::of(there[axis])
                    .minus(&Exact::of(here[axis]))
                    .times(&sideways[axis])?,
            ),
        )
    })?;
    Some((
        volume.enclosure(),
        Exact::squared_length_of(&sideways)?.enclosure(),
    ))
}

/// A number with the same sign as `mine·ρ₁ + theirs·ρ₂ + gap·δ`, exactly, where
/// the arithmetic can reach it.
///
/// **A sign and not the value**, because the value is not a polynomial in the
/// stored numbers — it is a sum of three square roots, `rᵢ/√Wᵢ` and `|T|/√K`.
/// What *is* a polynomial is the comparison once the roots are squared away, and
/// squaring them away is where this runs out of room.
///
/// Write everything scaled by the positive number `W₁W₂K`:
///
/// ```text
/// M = r₁²W₂K + r₂²W₁K − T²W₁W₂        (ρ₁² + ρ₂² − δ², scaled)
/// N = 2r₁r₂K√(W₁W₂)                    (2ρ₁ρ₂, scaled)
/// ```
///
/// Then `(mine·ρ₁ + theirs·ρ₂)² − δ²` is `M + mine·theirs·N`, and the sign of a
/// sum of two numbers whose signs are known is either one of those signs or the
/// sign of the difference of their squares. So the whole comparison comes down
/// to `sign(M)` — degree eight — and, where that does not settle it,
/// `sign(M² − N²)` with `N² = 4r₁²r₂²K²W₁W₂`, which is degree sixteen against
/// M's twenty.
///
/// **That last one is what runs out.** `Exact::times` refuses as soon as a
/// product's low half would land among the subnormals, and `M` for two ordinary
/// pipes is already five hundred binary orders wide, so squaring it falls off
/// the bottom of `f64`. Where the two stored directions have the same squared
/// length the root `√(W₁W₂)` is rational and none of this is needed at all,
/// which is why every pair whose axes are coordinate directions is decided here
/// and a general rotation is not.
/// `the_question_two_leaning_pipes_ask_is_too_wide_to_square_where_it_stands` is
/// the demonstration, and §29 is the argument it belongs to.
fn how_two_radii_and_a_gap_compare_exactly(
    one: &Cylinder,
    other: &Cylinder,
    mine: f64,
    theirs: f64,
    gap: f64,
) -> Option<Exact> {
    let [here, there] = [one, other].map(|pipe| pipe.axis().origin().coordinates());
    let [my_way, their_way] = [one, other].map(|pipe| pipe.axis().direction().components());
    let squared_mine = Exact::dot_of(my_way, my_way)?;
    let squared_theirs = Exact::dot_of(their_way, their_way)?;
    let sideways = Exact::cross_of(&Exact::of_each(my_way), their_way)?;
    let squared_sideways = Exact::squared_length_of(&sideways)?;
    let volume = (0..3).try_fold(Exact::of(0.0), |so_far, axis| {
        Some(
            so_far.plus(
                &Exact::of(there[axis])
                    .minus(&Exact::of(here[axis]))
                    .times(&sideways[axis])?,
            ),
        )
    })?;

    // The sign of `mine·ρ₁ + theirs·ρ₂`, from `ρ₂² − ρ₁² = (r₂²W₁ − r₁²W₂)/W₁W₂`
    // where the two pull against each other.
    let squared_gap = squared_mine.times(&squared_theirs)?;
    let radii = if mine * theirs > 0.0 {
        if mine > 0.0 {
            Decision::Pos
        } else {
            Decision::Neg
        }
    } else {
        let bigger = Exact::product_of(other.radius(), other.radius())?
            .times(&squared_mine)?
            .minus(&Exact::product_of(one.radius(), one.radius())?.times(&squared_theirs)?);
        match (bigger.sign(), theirs > 0.0) {
            (Decision::Zero, _) => Decision::Zero,
            (Decision::Pos, true) | (Decision::Neg, false) => Decision::Pos,
            (Decision::Pos, false) | (Decision::Neg, true) => Decision::Neg,
            _ => return None,
        }
    };
    let along_the_gap = match volume.sign() {
        Decision::Zero => Decision::Zero,
        Decision::Pos | Decision::Neg if gap > 0.0 => Decision::Pos,
        Decision::Pos | Decision::Neg => Decision::Neg,
        Decision::Undecidable { .. } | _ => return None,
    };

    // Both pull the same way, or one of them is nothing: no squaring needed.
    let sign = match (radii, along_the_gap) {
        (Decision::Zero, other_sign) => other_sign,
        (this_sign, Decision::Zero) => this_sign,
        (Decision::Pos, Decision::Pos) => Decision::Pos,
        (Decision::Neg, Decision::Neg) => Decision::Neg,
        (bigger_first, _) => {
            let squared_difference = how_two_squares_compare_exactly(
                one,
                other,
                mine * theirs,
                &squared_mine,
                &squared_theirs,
                &squared_sideways,
                &squared_gap,
                &volume,
            )?;
            match (squared_difference, bigger_first == Decision::Pos) {
                (Decision::Zero, _) => Decision::Zero,
                (Decision::Pos, true) | (Decision::Neg, false) => Decision::Pos,
                (Decision::Pos, false) | (Decision::Neg, true) => Decision::Neg,
                _ => return None,
            }
        }
    };
    Some(Exact::of(match sign {
        Decision::Pos => 1.0,
        Decision::Zero => 0.0,
        Decision::Neg => -1.0,
        Decision::Undecidable { .. } | _ => return None,
    }))
}

/// The sign of `(mine·ρ₁ + theirs·ρ₂)² − δ²`, scaled by the positive `W₁W₂K`.
///
/// `M + together·N` with `M` and `N` as [`how_two_radii_and_a_gap_compare_exactly`]
/// writes them. `N` is never below zero, so where `M` pulls the same way the
/// sign is `M`'s; where they pull against each other it is the sign of
/// `M² − N²`, and that is the one that can run out of range.
#[allow(clippy::too_many_arguments)]
fn how_two_squares_compare_exactly(
    one: &Cylinder,
    other: &Cylinder,
    together: f64,
    squared_mine: &Exact,
    squared_theirs: &Exact,
    squared_sideways: &Exact,
    squared_gap: &Exact,
    volume: &Exact,
) -> Option<Decision> {
    let radii_apart = Exact::product_of(one.radius(), one.radius())?
        .times(squared_theirs)?
        .plus(&Exact::product_of(other.radius(), other.radius())?.times(squared_mine)?)
        .times(squared_sideways)?
        .minus(&volume.times(volume)?.times(squared_gap)?);
    let leaning_the_same_way = (together > 0.0) == (radii_apart.sign() != Decision::Neg);
    if radii_apart.sign() == Decision::Zero {
        // `M` is nothing, so the answer is `together·N`, and `N` is above zero
        // for two real radii.
        return Some(if together > 0.0 {
            Decision::Pos
        } else {
            Decision::Neg
        });
    }
    if leaning_the_same_way {
        return Some(radii_apart.sign());
    }

    // `M² − N²`, with `N² = 4r₁²r₂²K²W₁W₂`. `M` for two ordinary leaning pipes
    // is five hundred binary orders wide, so its square does not fit where it
    // stands; `square_minus` slides both sides up together, which cannot change
    // a sign (`docs/decisions.md` §31.4).
    let together_squared = Exact::product_of(2.0 * one.radius(), 2.0 * other.radius())?
        .times(&Exact::product_of(one.radius(), other.radius())?)?
        .times(squared_sideways)?
        .times(squared_sideways)?
        .times(squared_gap)?;
    let against = radii_apart.square_minus(&together_squared)?;
    Some(
        match (against.sign(), radii_apart.sign() == Decision::Pos) {
            (Decision::Zero, _) => Decision::Zero,
            (Decision::Pos, true) => Decision::Pos,
            (Decision::Pos, false) => Decision::Neg,
            (Decision::Neg, _) => {
                if together > 0.0 {
                    Decision::Pos
                } else {
                    Decision::Neg
                }
            }
            _ => return None,
        },
    )
}

/// `E² − 4r₁²r₂²W₁W₂` as an enclosure.
///
/// Below zero the two cross in two lines, and then this same reading says how far
/// apart those lines are — see [`two_lines_between_two_pipes`].
fn how_deeply_two_side_by_side_pipes_overlap_enclosing(
    one: &Cylinder,
    other: &Cylinder,
) -> Interval {
    let [my_way, their_way] = [one, other].map(|pipe| pipe.axis().direction().components());
    let room = how_much_room_two_side_by_side_pipes_leave_enclosing(one, other);
    room * room
        - Interval::exactly(4.0)
            * squared_enclosing(one.radius())
            * squared_enclosing(other.radius())
            * squared_length_enclosing(exactly(my_way))
            * squared_length_enclosing(exactly(their_way))
}

/// `E² − 4r₁²r₂²W₁W₂`, exactly.
fn how_deeply_two_side_by_side_pipes_overlap_exactly(
    one: &Cylinder,
    other: &Cylinder,
) -> Option<Exact> {
    let [my_way, their_way] = [one, other].map(|pipe| pipe.axis().direction().components());
    let room = how_much_room_two_side_by_side_pipes_leave_exactly(one, other)?;
    Some(
        room.times(&room)?.minus(
            &Exact::product_of(2.0, 2.0)?
                .times(&Exact::product_of(one.radius(), one.radius())?)?
                .times(&Exact::product_of(other.radius(), other.radius())?)?
                .times(&Exact::dot_of(my_way, my_way)?)?
                .times(&Exact::dot_of(their_way, their_way)?)?,
        ),
    )
}

/// `E = A·W₂ − r₁²W₂ − r₂²W₁` as an enclosure.
fn how_much_room_two_side_by_side_pipes_leave_enclosing(
    one: &Cylinder,
    other: &Cylinder,
) -> Interval {
    let [here, there] = [one, other].map(|pipe| pipe.axis().origin().coordinates());
    let [my_way, their_way] = [one, other].map(|pipe| pipe.axis().direction().components());
    let squared_mine = squared_length_enclosing(exactly(my_way));
    let squared_theirs = squared_length_enclosing(exactly(their_way));
    how_far_off_an_axis_enclosing(there, here, my_way) * squared_theirs
        - squared_enclosing(one.radius()) * squared_theirs
        - squared_enclosing(other.radius()) * squared_mine
}

/// `E = A·W₂ − r₁²W₂ − r₂²W₁`, exactly.
fn how_much_room_two_side_by_side_pipes_leave_exactly(
    one: &Cylinder,
    other: &Cylinder,
) -> Option<Exact> {
    let [here, there] = [one, other].map(|pipe| pipe.axis().origin().coordinates());
    let [my_way, their_way] = [one, other].map(|pipe| pipe.axis().direction().components());
    let squared_mine = Exact::dot_of(my_way, my_way)?;
    let squared_theirs = Exact::dot_of(their_way, their_way)?;
    Some(
        how_far_off_an_axis_exactly(there, here, my_way)?
            .times(&squared_theirs)?
            .minus(&Exact::product_of(one.radius(), one.radius())?.times(&squared_theirs)?)
            .minus(&Exact::product_of(other.radius(), other.radius())?.times(&squared_mine)?),
    )
}

/// How far a picking half-space's own flat surface sits off the point it was
/// built through, as an upper bound on a real distance.
///
/// A half-space's normal is whatever length its caller's was, so the reading is
/// divided by that length to come back as a distance.
fn how_far_a_picking_half_space_sits_off_a_point(picker: &HalfSpace, through: Point) -> Interval {
    let normal = exactly(picker.normal().components());
    let off =
        dot_enclosing(normal, exactly(through.coordinates())) - Interval::exactly(picker.offset());
    quotient_or_everything(
        Interval::exactly(off.lo().abs().max(off.hi().abs())),
        root_enclosing(squared_length_enclosing(normal)),
    )
}

/// Two endless straight lines running along `along`, told apart by `picker`.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the picking half-space's
/// normal is not exactly square to the lines' own direction, or when the two
/// lines are not provably either side of it.
///
/// **The first of those cannot be tested away and the second can**, which is the
/// whole of what this closes of §28.2's gap. A line is endless, so a normal that
/// leans into its direction by even one rounding reads a different number a long
/// way along it and the half-space holds nothing. But *where* the flat surface
/// sits between the two lines is a comparison between the lines' own separation
/// and its own rounding, and that is a question with an answer.
///
/// The first was the whole of §29.8's remaining gap while the picker was a
/// [`Plane`], which rounds a squareness away on the way in. A [`HalfSpace`]
/// keeps it, so **the squareness check** only refuses where a product of two of
/// the direction's own components lands among the subnormals and the exact level
/// cannot run.
///
/// That sentence said "this now only refuses", of the whole function, until §93.
/// It was wrong, and a doc comment is where it was cheapest to be wrong: the
/// either-side check below was refusing a pair of parallel pipes whose directions
/// are `[1, 0, 0]`, with nothing subnormal anywhere and the exact level available,
/// because the caller handed it two line positions worked out by subtracting two
/// nearly equal large numbers. The caller stopped doing that; what is left here
/// refuses where the two lines really are within a rounding of each other.
fn two_lines_told_apart(
    picker: HalfSpace,
    along: [f64; 3],
    somewhere_on_each: [[Interval; 3]; 2],
    and: Surface,
) -> Result<SurfaceMeet> {
    let normal = picker.normal().components();
    if !matches!(
        Exact::dot_of(normal, along).as_ref().map(Exact::sign),
        Some(Decision::Zero)
    ) {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    let side = somewhere_on_each
        .map(|place| dot_enclosing(exactly(normal), place) - Interval::exactly(picker.offset()));
    if !(side[0].lo() > 0.0 && side[1].hi() < 0.0) {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    Ok(SurfaceMeet::Curves(
        [picker, picker.reversed()]
            .into_iter()
            .map(|half_space| {
                NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                    and,
                    picked_by: smallvec![Picker::from(half_space)],
                })
                .expect("picked_by always holds the one half-space just built")
            })
            .collect(),
    ))
}

/// A plane and a cone: seven shapes, and **every one of them is named**.
///
/// # The two signs that settle all seven
///
/// **Where the apex stands**, `n·a − d`, and **how the plane leans against the
/// cone's own half-angle**:
///
/// ```text
/// E  =  (n·w)²  −  slope² · ‖n × w‖²
/// ```
///
/// `E` is the textbook comparison "is the plane steeper than the cone" written
/// so that nothing is normalized: `‖n × w‖² = (n·n)(w·w) − (n·w)²`, so this is
/// `(1 + slope²)(n·w)² − slope²(n·n)(w·w)` with the two lengths carried, which
/// is the rule the module note opens with. Above zero the plane cuts across the
/// cone, at zero it is parallel to one ruling, below zero it is steeper than the
/// cone and reaches both nappes.
///
/// | apex | `E` | the crossing | here |
/// |---|---|---|---|
/// | off the plane | `> 0` | an ellipse, a circle when the plane is square to the axis | **named** — one closed curve |
/// | off the plane | `= 0` | a parabola | **named** — one endless bending run, held by the nappe it lies in |
/// | off the plane | `< 0` | a hyperbola | **named** — two endless bending runs, one per nappe — or [`Error::CannotNameWhichPieceOfACrossingThisIs`] |
/// | on the plane | `> 0` | the apex and nothing else | **named** — one touching point, exactly |
/// | on the plane | `= 0` | one ruling, touched along its whole length | **named** — one straight run, endless |
/// | on the plane | `< 0` | two rulings crossing at the apex | **named** — two straight runs, told apart by a picker each — or [`Error::CannotNameWhichPieceOfACrossingThisIs`] |
///
/// The seventh shape a plane and a quadric can make is *the same surface*, and a
/// plane is never a cone.
///
/// Both signs above are polynomials in stored `f64` and level 3 settles them;
/// nothing here is ever undecided except when a product leaves `f64`'s range.
///
/// # The two bending shapes are picked out by the nappe they lie in
///
/// A parabola lies wholly in **one** nappe of the cone and a hyperbola puts one
/// branch in each, so the flat surface through the apex square to the axis is
/// what names them — the fix the refusal that stood here used to describe as
/// the shortest one, taken. The
/// half-space's normal is the cone's axis direction **bit for bit**, so it is
/// exactly square to nothing and exactly parallel to the axis, and the sign it
/// reads at a point is the sign of `(x − apex)·ŵ`, which is the very number the
/// cone's own equation squares. The offset is the one thing that rounds, and the
/// two shapes want two different things of it:
///
/// - **The parabola only has to be held**, so the offset is rounded *down* — the
///   largest `f64` at or below `ŵ·apex`. The half-space then contains the whole
///   closed nappe whatever the rounding did, because `ŵ·x − offset` is at least
///   `ŵ·(x − apex)`, which is what being in the nappe means. The only ways this
///   refuses are the two [`the_whole_of_one_of_a_cones_nappes`] documents: the
///   apex's dot product with the axis leaving `f64`'s range, and the offset that
///   comes out of it not being a finite number.
/// - **The hyperbola's two branches have to be told apart**, so its offset is
///   rounded *up* instead — the smallest `f64` at or above `ŵ·apex`. Rounding
///   down would put the boundary below the apex, and then the half-space facing
///   one way holds a thin slab of the other nappe as well; rounding up puts the
///   boundary at or just above the apex, so **no point of the far branch is ever
///   held** — every point of it has `ŵ·x` strictly below `ŵ·apex`. What rounding
///   up costs is the *near* branch, which is now held only if the offset did not
///   climb past it, and that is a comparison this can make rather than assume.
///   See [`one_of_a_cones_nappes_and_none_of_the_other`].
///
/// **Neither rounding is enough on its own, and what closes the gap is measuring
/// the branch.** The families weighed: scaling the picker by a power of two
/// rounds nothing but does not make a sum representable that was not; rounding
/// down keeps the near branch and loses the far one; rounding up keeps the far
/// one and puts the near one in question. The third family — round up and then
/// *prove* the near branch clears the offset — is the one that works, because a
/// hyperbola's branch does not come near the apex: its vertex stands off the
/// apex plane by
///
/// ```text
/// v  =  |n·a − d| · ‖w‖ / (slope · ‖n × w‖ + |n·w|)
/// ```
///
/// which is a strictly positive length whenever the apex is off the plane, and
/// **only a lower bound on it is needed**, which is exactly what an enclosure
/// gives. That the vertex is "only known to a bound" is the reason this works
/// and not the reason it cannot: the rounding to beat is one ulp of `ŵ·apex`,
/// and `v` is a length of the model. Where the rounding does climb past the
/// bound — an apex so far out that one ulp of `ŵ·apex` swallows the branch's
/// clearance — this still refuses with
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] rather than picking
/// something it cannot prove, which is C1's rule 3 on
/// [`two_lines_through_a_cones_apex`] over again and for the same reason.
/// `a_tilted_cone_whose_axis_dot_apex_does_not_round_exactly_still_names_its_two_branches`
/// is the demonstration that the old exact-offset rule refused answers that
/// exist.
///
/// # The two shapes through the apex are straight, so they need no new curve
///
/// A ruling is one endless straight line and [`EdgeRun::Straight`] says exactly
/// that — a plane resting along a pipe's own side is named that way already. Two
/// rulings crossing at the apex are two of those, and
/// [`two_lines_through_a_cones_apex`] says which is which. Neither asks the
/// walker for anything: [`walk`](crate::walk) puts no points between the ends of
/// a straight run.
///
/// **The touching point is exact.** Where the plane passes through the apex and
/// nothing else, the one point they share is the apex, which is a stored `f64`
/// triple — not a box, unlike a plane resting on a ball, where the touching
/// point is a division away.
fn where_a_plane_and_a_cone_cross(
    flat: &Plane,
    cone: &Cone,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let at = [
        PredicatePoint::from(flat.origin().coordinates()),
        PredicatePoint::from(cone.apex().coordinates()),
    ];

    let standing_off = ask(
        &where_the_apex_stands(flat, cone),
        Measures::Length,
        &at,
        policy,
        log,
    )?;
    let apex_is_on_the_plane = match standing_off {
        Decision::Zero => true,
        Decision::Pos | Decision::Neg => false,
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    };

    let leaning = ask(
        &how_the_plane_leans(flat, cone),
        Measures::Length,
        &at,
        policy,
        log,
    )?;

    match (apex_is_on_the_plane, leaning) {
        (false, Decision::Pos) => Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
            EdgeRun::RightRoundWhereItsFacesCross,
        )?])),
        (false, Decision::Zero) => {
            one_parabola_in_one_of_a_cones_nappes(flat, cone, standing_off, policy, log)
        }
        (false, Decision::Neg) => the_two_branches_of_a_hyperbola(flat, cone),
        (true, Decision::Pos) => Ok(SurfaceMeet::TouchAtOnePoint {
            at: PredicatePoint::from(cone.apex().coordinates()),
        }),
        (true, Decision::Zero) => Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
            EdgeRun::Straight,
        )?])),
        (true, Decision::Neg) => two_lines_through_a_cones_apex(flat, cone),
        (_, other) => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// Where a cone's apex stands against a plane: the sign of `n·a − d`, **proved
/// and never rounded**.
///
/// [`Decision::Zero`] and the apex is on the plane, which is the half of
/// [`where_a_plane_and_a_cone_cross`]'s table every straight case needs.
/// [`Decision::Undecidable`] when a product on the way leaves the range where
/// this crate's expansion arithmetic is exact, and there is nothing else it can
/// be: `n·a − d` is a polynomial in stored `f64`.
///
/// **No [`Policy`], on purpose** — see [`how_a_plane_leans_against_a_cone`],
/// which says why for both of them.
pub fn where_a_cones_apex_stands_against_a_plane(flat: &Plane, cone: &Cone) -> Decision {
    decide(&where_the_apex_stands(flat, cone)).0
}

/// How a plane leans against a cone's own side: the sign of
/// `E = (n·w)² − slope²‖n × w‖²`, **proved and never rounded**.
///
/// Above zero the plane cuts across the cone, at zero it is parallel to one
/// ruling, below zero it is steeper than the cone and reaches both nappes —
/// [`where_a_plane_and_a_cone_cross`]'s second sign, and the one its whole table
/// turns on.
///
/// # Why neither of these two takes a [`Policy`]
///
/// [`where_a_plane_and_a_cone_cross`] asks these same two questions through the
/// policy, which is handed every enclosure the cascade could not certify; a
/// tolerant one may read such an enclosure as zero. **The two doors in
/// `rcad-topo` that ask what shape a crossing is must not be tolerant about the
/// shape**, because a `Zero` they have not earned is what says *rulings* about a
/// closed ellipse, and then the ellipse is ordered along an axis
/// (`docs/decisions.md` §49.2, §49.8). So the doors get the cascade and nothing
/// after it: level 2 and level 3 both prove their signs, and neither can be
/// softened by anything a caller passes.
///
/// The two therefore part company only under a tolerant policy, and only into a
/// **refusal**: the door gives up where `intersect` would have been given an
/// answer. That is the same trade `rcad-topo`'s `a_plane_lies_along_a_pipes_axis`
/// states one row down, for the same reason.
///
/// # This used to be rounded, and it said a closed ellipse was an endless line
///
/// The doors wrote `flat.signed_distance_to(cone.apex()) != 0.0` and
/// `up*up - slope*slope*sideways <= 0.0` in `f64`. It takes **both** of those
/// being wrong at once to matter, and then the pair `intersect` names as one
/// closed ellipse comes back as two rulings meeting at the apex. `rcad-topo`'s
/// `a_rounded_apex_and_a_rounded_lean_together_call_a_closed_ellipse_two_rulings`
/// writes one such plane and cone out, and
/// `how_many_near_tangent_planes_a_rounded_cone_gate_gets_wrong` counts how
/// common they are over 360 000 draws.
pub fn how_a_plane_leans_against_a_cone(flat: &Plane, cone: &Cone) -> Decision {
    decide(&how_the_plane_leans(flat, cone)).0
}

/// The sign of `n·a − d`, as a question rather than an answer, so that
/// [`where_a_plane_and_a_cone_cross`] and
/// [`where_a_cones_apex_stands_against_a_plane`] ask **one** expression written
/// **once**.
fn where_the_apex_stands(flat: &Plane, cone: &Cone) -> impl Predicate {
    let normal = flat.normal().components();
    let d = flat.offset();
    let apex = cone.apex().coordinates();
    Question {
        named: WHERE_A_PLANE_AND_A_CONE_CROSS,
        enclosing: move || dot_enclosing(exactly(normal), exactly(apex)) - Interval::exactly(d),
        exactly: move || Some(Exact::dot_of(normal, apex)?.minus(&Exact::of(d))),
    }
}

/// The sign of `E`, as a question rather than an answer, shared the same way.
///
/// **Degree six in the stored numbers, and every degree of it is carried.**
/// Written as `(n·w)² − Σ(slope·(n_j w_k − n_k w_j))²`, which is
/// `slope²‖n × w‖²` with the slope taken inside the squares — so the exact arm
/// squares four short expansions and never multiplies two long ones together,
/// which is the form §49.8 chose for it and the form the door in `rcad-topo`
/// used to carry a copy of.
///
/// It is dimensionless: every number in it is a direction component or a slope.
/// [`Measures::Length`] is the arm whose conversion is the identity, which is the
/// right conversion for a value with no units, and §28 records that the *name* is
/// wrong for the same reason the plane-and-pipe row does.
fn how_the_plane_leans(flat: &Plane, cone: &Cone) -> impl Predicate {
    let normal = flat.normal().components();
    let along = cone.axis().direction().components();
    let slope = cone.slope();
    Question {
        named: WHERE_A_PLANE_AND_A_CONE_CROSS,
        enclosing: move || {
            let up = dot_enclosing(exactly(normal), exactly(along));
            let across = squared_length_enclosing(cross_enclosing(exactly(normal), exactly(along)));
            up * up - squared_enclosing(slope) * across
        },
        exactly: move || {
            let up = Exact::dot_of(normal, along)?;
            let mut leaning = up.times(&up)?;
            for axis in 0..3 {
                let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
                let across = Exact::product_of(normal[next], along[after])?
                    .minus(&Exact::product_of(normal[after], along[next])?);
                let leaned = across.times(&Exact::of(slope))?;
                leaning = leaning.minus(&leaned.times(&leaned)?);
            }
            Some(leaning)
        },
    }
}

/// The parabola a plane parallel to one of a cone's rulings cuts it in, named
/// as the piece of the crossing the nappe it lies in holds.
///
/// # Which nappe, worked out and not guessed
///
/// Put the parabola into the cone's own equation in the plane's two directions
/// and everything but the sign cancels: for a point `x` of it,
///
/// ```text
/// (x − apex)·ŵ  =  −(r² + h²(1 + t²)) / (2(1 + t²)·h·(n̂·ŵ))
/// ```
///
/// with `r` the distance along the direction square to the axis. The numerator
/// is a sum of squares and is never zero, so the whole curve is on the side
/// **opposite** the sign of `h·(n̂·ŵ)` — which is the sign of `reach` times the
/// sign of `n·w`, both of them exact polynomial questions this asks rather than
/// reads off a rounded product. Neither can be zero in this row: a zero `n·w`
/// makes `E = −t²` and a zero `reach` puts the apex on the plane.
///
/// # Errors
///
/// [`Error::Undecidable`] if the sign of `n·w` cannot be settled, and
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] if the apex's own dot
/// product with the axis leaves `f64`'s range.
fn one_parabola_in_one_of_a_cones_nappes(
    flat: &Plane,
    cone: &Cone,
    standing_off: Decision,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let normal = flat.normal().components();
    let along = cone.axis().direction().components();
    let at = [
        PredicatePoint::from(flat.origin().coordinates()),
        PredicatePoint::from(cone.apex().coordinates()),
    ];
    let leans_up = Question {
        named: WHERE_A_PLANE_AND_A_CONE_CROSS,
        enclosing: || dot_enclosing(exactly(normal), exactly(along)),
        exactly: || Exact::dot_of(normal, along),
    };
    let towards = match (
        standing_off,
        ask(&leans_up, Measures::Length, &at, policy, log)?,
    ) {
        (Decision::Pos, Decision::Pos) | (Decision::Neg, Decision::Neg) => -1.0,
        (Decision::Pos, Decision::Neg) | (Decision::Neg, Decision::Pos) => 1.0,
        (apex, axis) => unreachable!(
            "a parabola stands off the plane and leans on the axis: {apex:?}, {axis:?}"
        ),
    };

    let picker = the_whole_of_one_of_a_cones_nappes(cone, towards)?;
    Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Cone(*cone),
            picked_by: smallvec![Picker::from(picker)],
        },
    )?]))
}

/// The two branches a plane steeper than a cone's side cuts it in, one per
/// nappe.
///
/// **Which branch is which is not asked here and does not have to be.** The
/// crossing comes in two pieces, one in each nappe, and the pair of runs this
/// hands back is exactly *the piece in the nappe the axis points along* and *the
/// piece in the other one*. Which of them a caller later calls the first is the
/// picker's to say, and [`walk`](crate::walk) reads it off the picker when it
/// draws — so no branch geometry is worked out on this side of the wall §24.3
/// puts between naming a crossing and drawing one.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the flat surface
/// through the apex square to the axis cannot be written down with an offset
/// that both branches clear, which is what telling the two branches apart needs
/// and what [`where_a_plane_and_a_cone_cross`] argues for.
fn the_two_branches_of_a_hyperbola(flat: &Plane, cone: &Cone) -> Result<SurfaceMeet> {
    let clearance = how_far_a_hyperbolas_vertex_stands_off_the_apex_plane(flat, cone);
    let mut runs = Vec::new();
    for towards in [1.0, -1.0] {
        let picker = one_of_a_cones_nappes_and_none_of_the_other(cone, towards, clearance)?;
        runs.push(NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
            and: Surface::Cone(*cone),
            picked_by: smallvec![Picker::from(picker)],
        })?);
    }
    Ok(SurfaceMeet::Curves(runs))
}

/// The half-space holding the whole of the nappe the axis points `towards`,
/// with its offset rounded **down** so that it holds all of that nappe whatever
/// the rounding did.
///
/// `towards` is `1.0` or `-1.0`, and multiplying a direction by either of those
/// rounds nothing, so the normal is the axis direction's own three `f64`s with
/// at most a sign flipped.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the dot product of the
/// axis direction with the apex leaves `f64`'s range, so there is no offset to
/// round in either direction, and [`Error::NotFinite`] when the offset that
/// comes out of it is not a finite number.
fn the_whole_of_one_of_a_cones_nappes(cone: &Cone, towards: f64) -> Result<HalfSpace> {
    let facing = cone.axis().direction() * towards;
    let Some(exactly) = Exact::dot_of(facing.components(), cone.apex().coordinates()) else {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    };
    let nearest = exactly.nearest_f64();
    let offset = if exactly.minus(&Exact::of(nearest)).sign() == Decision::Neg {
        nearest.next_down()
    } else {
        nearest
    };
    HalfSpace::new(facing, offset)
}

/// The half-space holding the whole of the nappe the axis points `towards` and
/// **no point at all** of the other one, with its offset rounded **up**.
///
/// An offset at or above `ŵ·apex` rules out every point of the far nappe on its
/// own, since each of those has `ŵ·x` strictly below `ŵ·apex`. What has to be
/// checked is the near branch: it is held only while the offset stays at or
/// below `ŵ·apex + clearance`, and `clearance` is a lower bound on how far that
/// branch's own vertex stands off the apex plane. The comparison is exact — the
/// rounded offset, the unrounded dot product and the bound go through the same
/// expansion arithmetic — so nothing here compares a distance against a
/// tolerance.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the dot product of the
/// axis direction with the apex leaves `f64`'s range, and when rounding it up
/// climbs past `clearance`, so the half-space could no longer be shown to hold
/// the whole of the branch it is meant to name.
fn one_of_a_cones_nappes_and_none_of_the_other(
    cone: &Cone,
    towards: f64,
    clearance: f64,
) -> Result<HalfSpace> {
    let facing = cone.axis().direction() * towards;
    let Some(exactly) = Exact::dot_of(facing.components(), cone.apex().coordinates()) else {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    };
    let nearest = exactly.nearest_f64();
    let offset = if exactly.minus(&Exact::of(nearest)).sign() == Decision::Pos {
        nearest.next_up()
    } else {
        nearest
    };
    let climbed = Exact::of(offset).minus(&exactly);
    if climbed.minus(&Exact::of(clearance)).sign() == Decision::Pos {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }
    HalfSpace::new(facing, offset)
}

/// A lower bound on how far the vertex of either branch of the hyperbola `flat`
/// cuts `cone` in stands off the flat surface through the apex square to the
/// axis, or zero when that bound cannot be worked out.
///
/// Put a point of the crossing at `apex + u·ŵ + r·q̂` with `q̂` square to the
/// axis. The cone makes `r = slope·|u|`, and feeding that into the plane leaves
/// `u·(n̂·ŵ ± slope·‖n̂ × ŵ‖·μ) = −(n̂·apex − d̂)` for some `μ` in `[−1, 1]`, so
/// the smallest `|u|` either branch reaches is
///
/// ```text
/// |n̂·apex − d̂| / (slope·‖n̂ × ŵ‖ + |n̂·ŵ|)
/// ```
///
/// Written with the *stored* normal and axis direction rather than unit ones,
/// the normal's own length cancels top and bottom and the axis direction's
/// length multiplies through, which is the `‖w‖` below. Nothing is normalized
/// here, for the reason the module note opens with.
fn how_far_a_hyperbolas_vertex_stands_off_the_apex_plane(flat: &Plane, cone: &Cone) -> f64 {
    let normal = flat.normal().components();
    let along = cone.axis().direction().components();
    let slope = cone.slope();

    let stands_off = dot_enclosing(exactly(normal), exactly(cone.apex().coordinates()))
        - Interval::exactly(flat.offset());
    if stands_off.contains_zero() {
        return 0.0;
    }
    let how_far_off = Interval::exactly(stands_off.lo().abs().min(stands_off.hi().abs()));
    let axis_length = root_enclosing(squared_length_enclosing(exactly(along)));
    let up = dot_enclosing(exactly(normal), exactly(along));
    let leaning = Interval::exactly(up.lo().abs().max(up.hi().abs()));
    let across = root_enclosing(squared_length_enclosing(cross_enclosing(
        exactly(normal),
        exactly(along),
    )));

    let clearance = quotient_or_everything(
        how_far_off * axis_length,
        Interval::exactly(slope) * across + leaning,
    )
    .lo();
    if clearance > 0.0 && clearance.is_finite() {
        clearance
    } else {
        0.0
    }
}

/// The two rulings a plane through a cone's apex cuts it in, each picked out by
/// the flat surface that holds the whole of it and meets the other only at the
/// apex.
///
/// # A pair of opposite half-spaces is what picks one of two lines that meet
///
/// The old refusal here said no half-space picks one of them, *"because both
/// pass through the apex and a half-space that holds one holds half of the
/// other"*. Both halves of that are true and the conclusion does not follow. A
/// piece is picked by **every** half-space on it, so the two opposite ones about
/// one flat surface pick what lies *in* that surface — the idiom
/// [`Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo`] already names for
/// two ellipses in one plane. The flat surface through the apex holding line one
/// meets line two at the apex alone, so the pair picks line one and nothing else.
///
/// # What has to be exact, and why each one has to be
///
/// A line is endless, so a picker that leans into its direction by one rounding
/// reads a different sign a long way along it and holds nothing — the argument
/// [`HalfSpace`] opens with, and the same one [`two_lines_told_apart`] rests on.
/// Three things are therefore decided exactly and this refuses rather than
/// rounding any of them:
///
/// 1. **The ruling direction is a stored triple.** Its two components along the
///    plane are `‖w‖²·p ± √(−E·‖w‖²)·(n × w)`, and that root is irrational for
///    almost every pair. So the direction is worked out in `f64` and then *asked*
///    whether the endless line through the apex along it lies on both surfaces,
///    which is two exact polynomial signs. A cone whose rulings `f64` cannot
///    write down is refused.
/// 2. **The picker is exactly square to the line it holds**, which
///    [`HalfSpace::through_a_point_square_to`] gives and a [`Plane`] would round
///    away (§32.3).
/// 3. **The picker's flat surface passes exactly through the apex.** Its offset
///    is one dot product and rounds; a rounded one moves the surface off the
///    apex, and then it holds one of the two lines and not the other rather than
///    holding one and half of the other. That is the one place here where the
///    usual *"a rounded offset only slides the surface"* argument fails, because
///    both lines lie in it.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when any of those three
/// cannot be had, and whatever [`HalfSpace::through_a_point_square_to`] refuses.
/// A **gap** and not a limit (§16.10): the two lines are there and they are
/// tellable apart, and what runs out is `f64`'s supply of exact ruling
/// directions.
fn two_lines_through_a_cones_apex(flat: &Plane, cone: &Cone) -> Result<SurfaceMeet> {
    let normal = flat.normal();
    let along = cone.axis().direction();
    let squared_axis = along.dot(along);
    let up = normal.dot(along);
    let sideways = normal.dot(normal) * squared_axis - up * up;
    let leaning = up * up - cone.slope() * cone.slope() * sideways;

    let down_the_plane = along * normal.dot(normal) - normal * up;
    let across_the_plane = normal.cross(along);
    let spread = (-leaning * squared_axis).max(0.0).sqrt();
    let rulings =
        [1.0, -1.0].map(|side| down_the_plane * squared_axis + across_the_plane * (spread * side));

    let mut runs = Vec::new();
    for [this, other] in [[rulings[0], rulings[1]], [rulings[1], rulings[0]]] {
        if !a_line_through_the_apex_along_it_lies_on_both(this, flat, cone) {
            return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
        }
        let picker = HalfSpace::through_a_point_square_to(cone.apex(), this, other)?;
        if !it_holds_this_line_and_only_the_apex_of_the_other(&picker, cone.apex(), this, other) {
            return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
        }
        runs.push(NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
            and: Surface::Cone(*cone),
            picked_by: smallvec![Picker::from(picker), Picker::from(picker.reversed())],
        })?);
    }
    Ok(SurfaceMeet::Curves(runs))
}

/// Whether the endless line through the apex along `direction` is on the plane
/// and on the cone, both decided exactly.
///
/// The apex is on the cone by definition and on the plane because the caller
/// decided so, so what is left is two signs about the direction alone: `n·d = 0`
/// puts the line in the plane, and `(d·d)(w·w) = (1 + slope²)(d·w)²` puts it on
/// the cone.
fn a_line_through_the_apex_along_it_lies_on_both(
    direction: Vector,
    flat: &Plane,
    cone: &Cone,
) -> bool {
    if !direction.is_finite() || direction == Vector::zero() {
        return false;
    }
    let d = direction.components();
    let along = cone.axis().direction().components();
    let on_the_cone = || {
        let up = Exact::dot_of(d, along)?;
        let up_squared = up.times(&up)?;
        let slope_squared = Exact::product_of(cone.slope(), cone.slope())?;
        Some(
            Exact::dot_of(d, d)?
                .times(&Exact::dot_of(along, along)?)?
                .minus(&up_squared)
                .minus(&slope_squared.times(&up_squared)?)
                .sign(),
        )
    };
    matches!(
        Exact::dot_of(flat.normal().components(), d).map(|reach| reach.sign()),
        Some(Decision::Zero)
    ) && matches!(on_the_cone(), Some(Decision::Zero))
}

/// Whether `picker`'s flat surface holds the whole of the line along `this` and
/// meets the line along `other` at the apex alone.
///
/// The squareness is asked rather than assumed: it is what
/// [`HalfSpace::through_a_point_square_to`] promises, and a promise a caller
/// leans on for an *endless* run is worth one exact dot product.
fn it_holds_this_line_and_only_the_apex_of_the_other(
    picker: &HalfSpace,
    apex: Point,
    this: Vector,
    other: Vector,
) -> bool {
    let facing = picker.normal().components();
    let square_to_this = Exact::dot_of(facing, this.components()).map(|reach| reach.sign());
    let across_the_other = Exact::dot_of(facing, other.components()).map(|reach| reach.sign());
    let through_the_apex = Exact::dot_of(facing, apex.coordinates())
        .map(|reach| reach.minus(&Exact::of(picker.offset())).sign());
    matches!(square_to_this, Some(Decision::Zero))
        && matches!(across_the_other, Some(Decision::Pos | Decision::Neg))
        && matches!(through_the_apex, Some(Decision::Zero))
}

/// A cone and a pipe: two circles where they share an axis, refused where they
/// do not.
///
/// # There is no shape question at all
///
/// A [`Cone`] is the whole double cone and a [`Cylinder`]'s radius is above
/// zero, so `slope·|h| = ρ` has exactly two roots, `h = ±ρ/slope`, one in each
/// nappe. A cone and a pipe on one axis therefore cross in **exactly two
/// circles, always** — never nothing, never one, never a tangency. There is no
/// discriminant here and no table like the one
/// [`a_ball_centred_on_a_pipes_axis`] turns on, because there are no cases to
/// tell apart.
///
/// # Sharing an axis is two sign questions and not one
///
/// `‖w_c × w_p‖² = 0` says the two axes run along one direction, and
/// `‖(a − o_p) × w_p‖² = 0` says the apex sits on the pipe's axis. Together they
/// say the two stored axes name one line, which is what the paragraph above
/// needs. Both are sums of squares, so both can only answer [`Decision::Pos`] or
/// [`Decision::Zero`], and both are shapes this module already asks —
/// [`where_two_pipes_cross`] asks the first and [`where_a_ball_and_a_pipe_cross`]
/// the second.
///
/// # What stays refused is one gap, reached down two roads
///
/// A cone and a pipe whose axes lean against each other cross in a genuine
/// degree-four space curve and are refused, which is the row's standing **gap**.
///
/// **That refusal is load-bearing and not only unfinished work.** `rcad-topo`'s
/// `assemble.rs` answers *this crossing holds no straight line* for every cone
/// against a pipe, and a leaning pipe can hold a whole ruling: the pipe of
/// radius two about `(1, 0, 1)` through `(0, 2, 0)` holds the ruling `(t, 0, t)`
/// of the cone of slope one about `+z`, exactly and on whole numbers. Naming any
/// such pair turns that arm into a wrong answer, which is §50.2 item 4's trap in
/// a second family. Two circles hold no line, so the coaxial pair leaves it
/// right.
///
/// **A pair a person drew coaxial reaches that same gap down the second road,
/// and a placement is all it takes.** Both surfaces arrive here with their
/// locations already composed, so what the apex question reads is the *placed*
/// pair. Turn a frustum sitting in a bore — one line, two axis origins written
/// at different points along it, which is how a countersink met by a bore comes
/// out of a file — about anything but a coordinate direction, and the two
/// origins round apart: `‖(a − o_p) × w_p‖²` answers [`Decision::Pos`] and the
/// two surfaces **as stored** genuinely are not coaxial. The coaxiality question
/// is answered correctly and nothing is being refused there. What is refused is
/// the quartic the placed pair really does cross in, and that is the gap booked
/// above, reached from one more direction (§16.10).
///
/// **That road is not a corner of the input space.** Any coaxial pair whose two
/// axis origins are written as different points can lose its axis to nothing
/// but a placement, and nearly all of them do:
/// `a_frustum_in_a_bore_stops_sharing_its_axis_as_soon_as_it_is_turned_off_the_lattice`
/// finds 62 of 64 turns about the diagonal costing the pair its axis. The
/// boundary is two origins written as **one** point, which one arithmetic sends
/// to one place however it is turned. §50.11 reaches the same place from the
/// corpus side — about 5% of nist's real countersinks sit on leaning axes and
/// stay refused — and from the arithmetic side, where 20 000 of 20 000 apexes
/// spelled `origin + t·ŵ` on a non-coordinate direction land an ulp or two off
/// the stored line, worst miss `2.6e-14` on coordinates of order 100.
///
/// # Errors
///
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] where the two do not
/// share an axis.
///
/// [`Error::Undecidable`] where a sum of squares could not be settled, which
/// takes a product leaving `f64`'s range.
fn where_a_cone_and_a_pipe_cross(
    cone: &Cone,
    pipe: &Cylinder,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let at =
        [cone.apex(), pipe.axis().origin()].map(|place| PredicatePoint::from(place.coordinates()));

    // Two unit directions crossed together is dimensionless, and filed as a
    // length for the reason `where_a_plane_and_a_pipe_cross` gives.
    let along_one_direction = a_cone_and_a_pipe_point_along_one_direction(cone, pipe);
    match ask(&along_one_direction, Measures::Length, &at, policy, log)? {
        Decision::Zero => (),
        Decision::Pos => return Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
        other => unreachable!("a sum of squares is never below zero: {other:?}"),
    }

    let apex_on_the_axis = a_cones_apex_sits_on_a_pipes_axis(cone, pipe);
    match ask(&apex_on_the_axis, Measures::Area, &at, policy, log)? {
        Decision::Zero => two_circles_round_a_shared_axis(cone, pipe),
        Decision::Pos => Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
        other => unreachable!("a sum of squares is never below zero: {other:?}"),
    }
}

/// `‖w_c × w_p‖²`, as a question rather than an answer.
fn a_cone_and_a_pipe_point_along_one_direction(cone: &Cone, pipe: &Cylinder) -> impl Predicate {
    squared_length_of_a_cross_product(
        WHERE_A_CONE_AND_A_PIPE_CROSS,
        cone.axis().direction().components(),
        pipe.axis().direction().components(),
    )
}

/// Whether a cone and a pipe share one axis, **proved and never rounded**.
///
/// [`Decision::Zero`] and the two stored axes name one line, so the pair crosses
/// in the two circles [`where_a_cone_and_a_pipe_cross`] names and in nothing
/// else.
///
/// **Every other answer means "not proved coaxial", and that is all it means.**
/// [`Decision::Pos`] says the two axes are apart. [`Decision::Undecidable`] says
/// the arithmetic settled nothing — *not* that the crossing is a quartic, which
/// is a claim nobody made. The `leaning => leaning` arm below hands both back
/// untouched, and hands back whatever variant [`Decision`] grows next, because
/// the invariant it rests on is about the **caller** and not about the enum: a
/// caller has to treat everything but `Zero` as a refusal, and `walk`'s
/// cone-and-pipe cut — the one caller there is — does, by asking
/// `!= Decision::Zero`. So a variant this enum grows later lands on the
/// refusing side instead of being mistaken for a proof, and a second caller
/// owes the same shape. That is the opposite of [`Picker::side_of_point`]'s
/// `same => same`, where `Zero` and `Undecidable` both genuinely say *not on a
/// side*, so a caller of this one may not lean on that precedent.
///
/// **No [`Policy`], on purpose**, for the reason
/// [`how_a_plane_leans_against_a_cone`] gives at length: [`walk`](crate::walk)
/// asks this to decide *which shape it is drawing*, and a `Zero` it has not
/// earned would draw two circles where the two surfaces cross in a quartic.
pub(crate) fn whether_a_cone_and_a_pipe_share_an_axis(cone: &Cone, pipe: &Cylinder) -> Decision {
    match decide(&a_cone_and_a_pipe_point_along_one_direction(cone, pipe)).0 {
        Decision::Zero => decide(&a_cones_apex_sits_on_a_pipes_axis(cone, pipe)).0,
        leaning => leaning,
    }
}

/// `‖(a − o_p) × w_p‖²`, shared the same way.
fn a_cones_apex_sits_on_a_pipes_axis(cone: &Cone, pipe: &Cylinder) -> impl Predicate {
    let apex = cone.apex().coordinates();
    let origin = pipe.axis().origin().coordinates();
    let along = pipe.axis().direction().components();
    Question {
        named: WHERE_A_CONE_AND_A_PIPE_CROSS,
        enclosing: move || how_far_off_an_axis_enclosing(apex, origin, along),
        exactly: move || how_far_off_an_axis_exactly(apex, origin, along),
    }
}

/// The two circles a cone and a pipe on one axis cross in, told apart by the
/// flat half-space through the apex facing along that axis.
///
/// # The separation is arranged and then proved, and the sideways part is
/// exactly nothing
///
/// One circle lies in each nappe, so the half-space through the apex holds one
/// and not the other. Its normal **is** the cone's stored axis direction — a
/// [`HalfSpace`] stores what it is handed — so what it reads at a point `x` of
/// the crossing is `w_c·(x − a)` plus however far the offset rounded, and
/// nothing else at all: every point of both circles sits square to `w_c` from
/// the axis, so the sideways part contributes exactly zero rather than a bounded
/// amount. That is the whole difference from
/// [`two_loops_either_side_of_a_ball`], which has to charge itself a sideways
/// reach.
///
/// `w_c·(x − a)` is `±r·‖w_c‖/(slope·‖w_p‖)`: the pipe's certified radius
/// `r/‖w_p‖` over the slope is how far along the unit axis a circle sits, and
/// reading it with the stored direction multiplies by `‖w_c‖`. **The two lengths
/// are both carried** because the two stored directions run along one line
/// without being one triple.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] where that standoff does not
/// beat the offset's own rounding — an apex so far from the origin that one
/// rounding of `w_c·a` swallows a circle's whole reach along the axis. A
/// **gap**: the two circles are there and they are tellable apart.
///
/// [`Error::NotFinite`] and [`Error::ZeroLength`] never fire: a cone's axis is
/// finite and has a length by construction.
fn two_circles_round_a_shared_axis(cone: &Cone, pipe: &Cylinder) -> Result<SurfaceMeet> {
    let along = cone.axis().direction();
    let between = HalfSpace::through_a_point_facing(cone.apex(), along)?;
    let facing = between.normal().components();
    let off_the_plane = dot_enclosing(exactly(facing), exactly(cone.apex().coordinates()))
        - Interval::exactly(between.offset());
    let offset_slop = off_the_plane.lo().abs().max(off_the_plane.hi().abs());

    let standoff = how_far_along_the_axis_the_two_circles_sit(cone, pipe);
    if (standoff - Interval::exactly(offset_slop)).lo() <= 0.0 {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    let cone = Surface::Cone(*cone);
    Ok(SurfaceMeet::Curves(
        [between, between.reversed()]
            .into_iter()
            .map(|half_space| {
                NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                    and: cone,
                    picked_by: smallvec![Picker::from(half_space)],
                })
                .expect("picked_by always holds the one half-space just built")
            })
            .collect(),
    ))
}

/// `r·‖w_c‖/(slope·‖w_p‖)`, which is what the cone's own stored direction reads
/// at either of the two circles, as an enclosure.
fn how_far_along_the_axis_the_two_circles_sit(cone: &Cone, pipe: &Cylinder) -> Interval {
    let reach = |direction: [f64; 3]| root_enclosing(squared_length_enclosing(exactly(direction)));
    quotient_or_everything(
        Interval::exactly(pipe.radius()) * reach(cone.axis().direction().components()),
        Interval::exactly(cone.slope()) * reach(pipe.axis().direction().components()),
    )
}

/// A plane and a torus: two circles, one circle, nothing, or a quartic nobody
/// here can name.
///
/// **Only two families of plane cut a torus in circles**, and the two sign
/// questions below are exactly the two that pick them out. Both are polynomials
/// in the numbers as stored, so level 3 settles them and nothing here is ever
/// undecided except when a product leaves `f64`'s range.
///
/// | the plane | the crossing | here |
/// |---|---|---|
/// | square to the axis, further off than the tube | nothing | **named** — [`SurfaceMeet::Nothing`] |
/// | square to the axis, exactly the tube's reach | one circle, touched all the way round | **named** — one closed curve |
/// | square to the axis, nearer than that | two circles round the axis, one inside the other | **named** — two closed curves, told apart by the pipe of the ring's own radius |
/// | through the axis | the two meridian circles | **named** — two closed curves, told apart by a flat half-space |
/// | parallel to the axis but off it | a Cassini oval, or two of them, or a figure of eight | [`Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic`] |
/// | anything else, including the two bitangent planes whose crossing is a pair of Villarceau circles | a quartic | the same |
///
/// The seventh shape a plane and a surface can make is *the same surface*, and a
/// plane is never a torus.
///
/// # The two circles round the axis need a picker that says *inside*
///
/// They are concentric and lie in one plane, so no flat half-space holds one
/// without holding half of the other, and no *outside* holds the inner one at
/// all. The pipe of the ring's own radius `R` about the torus's own axis is
/// exactly between them — the two radii are `R ± √(r² − t²)` with the root
/// strictly positive whenever this arm is reached — so
/// [`Picker::OutsideAPipe`] names the outer and
/// [`Picker::InsideAPipe`] names the inner. That second one is the only
/// picker in the kernel that says inside, and this is what needed it.
///
/// **What that costs when the two circles are a rounding apart.** The picking
/// pipe's certified radius is `R / ‖w‖` rather than `R`, so a pair whose gap
/// `√(r² − t²)` is smaller than a rounding of `R` is not provably either side of
/// it, and the sign question downstream comes back
/// [`Decision::Zero`] or [`Decision::Undecidable`] and the caller refuses. The
/// naming is still right; what is undecidable is a real knife edge.
fn where_a_plane_and_a_torus_cross(
    flat: &Plane,
    ring: &Torus,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let normal = flat.normal().components();
    let offset = flat.offset();
    let centre = ring.centre().coordinates();
    let along = ring.axis().direction().components();
    let at = [
        PredicatePoint::from(flat.origin().coordinates()),
        PredicatePoint::from(centre),
    ];

    let leans_across = Question {
        named: WHERE_A_PLANE_AND_A_TORUS_CROSS,
        enclosing: || squared_length_enclosing(cross_enclosing(exactly(normal), exactly(along))),
        exactly: || Exact::squared_length_of(&Exact::cross_of(&Exact::of_each(normal), along)?),
    };
    // Dimensionless — two directions crossed together. `Measures::Area` is what
    // every other squared cross product here is filed as, and §28 records that
    // the names are wrong for the same reason across the whole module.
    match ask(&leans_across, Measures::Area, &at, policy, log)? {
        Decision::Zero => {
            return where_a_plane_square_to_the_axis_cuts_a_torus(flat, ring, policy, log);
        }
        Decision::Pos => (),
        other => unreachable!("a sum of squares is never below zero: {other:?}"),
    }

    // Not square to the axis. The only other family that gives circles is a
    // plane holding the axis, which takes both of the next two to be zero: the
    // normal square to the axis direction, and the centre on the plane.
    let runs_along = Question {
        named: WHERE_A_PLANE_AND_A_TORUS_CROSS,
        enclosing: || dot_enclosing(exactly(normal), exactly(along)),
        exactly: || Exact::dot_of(normal, along),
    };
    if ask(&runs_along, Measures::Length, &at, policy, log)? != Decision::Zero {
        return Err(Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic);
    }
    let holds_the_centre = Question {
        named: WHERE_A_PLANE_AND_A_TORUS_CROSS,
        enclosing: || dot_enclosing(exactly(normal), exactly(centre)) - Interval::exactly(offset),
        exactly: || Some(Exact::dot_of(normal, centre)?.minus(&Exact::of(offset))),
    };
    if ask(&holds_the_centre, Measures::Length, &at, policy, log)? != Decision::Zero {
        return Err(Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic);
    }

    two_meridian_circles(flat, ring)
}

/// The two circles a plane through a torus's axis cuts, told apart by the flat
/// half-space square to both the axis and the plane's normal.
///
/// The picking normal is `w × n`, written with both stored triples and
/// **normalized nowhere**, which is the whole reason a picker is a
/// [`HalfSpace`] and not a [`Plane`]. The two circles have their centres `R`
/// either side of that flat surface and a radius of `r`, so the margin is
/// `(R − r)·‖w × n‖` and it is strictly positive because a torus's tube is
/// strictly narrower than its ring.
///
/// # Errors
///
/// [`Error::ZeroLength`] never fires: this is only reached with `n × w` proved
/// non-zero one function up. [`Error::NotFinite`] when the offset overflows,
/// and [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the rounding in
/// that offset is not provably smaller than the margin — which takes a torus
/// whose two radii are within a rounding of each other, or a centre so far from
/// the origin that the dot product loses the ring.
fn two_meridian_circles(flat: &Plane, ring: &Torus) -> Result<SurfaceMeet> {
    let across = ring.axis().direction().cross(flat.normal());
    let between = HalfSpace::through_a_point_facing(ring.centre(), across)?;
    let margin = Interval::exactly(ring.ring_radius() - ring.tube_radius())
        - how_far_a_picking_half_space_sits_off_a_point(&between, ring.centre());
    if margin.lo() <= 0.0 {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    let torus = Surface::Torus(*ring);
    Ok(SurfaceMeet::Curves(
        [between, between.reversed()]
            .into_iter()
            .map(|half_space| {
                NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                    and: torus,
                    picked_by: smallvec![Picker::from(half_space)],
                })
                .expect("picked_by always holds the one half-space just built")
            })
            .collect(),
    ))
}

/// A plane square to a torus's axis: nothing, one circle, or two.
///
/// The plane stands `|n·c − d| / ‖n‖` off the centre along the axis, so what
/// decides which of the three it is is the sign of `(n·c − d)² − r²(n·n)` — a
/// polynomial in stored numbers with the plane's own normal length carried, for
/// the reason the module note opens with.
fn where_a_plane_square_to_the_axis_cuts_a_torus(
    flat: &Plane,
    ring: &Torus,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let normal = flat.normal().components();
    let offset = flat.offset();
    let centre = ring.centre().coordinates();
    let tube = ring.tube_radius();
    let at = [
        PredicatePoint::from(flat.origin().coordinates()),
        PredicatePoint::from(centre),
    ];

    let reaches = Question {
        named: WHERE_A_PLANE_AND_A_TORUS_CROSS,
        enclosing: || {
            let off = dot_enclosing(exactly(normal), exactly(centre)) - Interval::exactly(offset);
            off * off - squared_enclosing(tube) * squared_length_enclosing(exactly(normal))
        },
        exactly: || {
            let off = Exact::dot_of(normal, centre)?.minus(&Exact::of(offset));
            Some(
                off.times(&off)?
                    .minus(&Exact::product_of(tube, tube)?.times(&Exact::dot_of(normal, normal)?)?),
            )
        },
    };
    match ask(&reaches, Measures::Area, &at, policy, log)? {
        Decision::Pos => Ok(SurfaceMeet::Nothing),
        // The plane rests on the top or the bottom of the tube and touches it
        // all the way round, in one circle of the ring's own radius. Tangency
        // along a whole closed curve, which `EdgeRun` says perfectly well — a
        // plane resting along a pipe is named `Straight` for the same reason.
        Decision::Zero => Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
            EdgeRun::RightRoundWhereItsFacesCross,
        )?])),
        Decision::Neg => two_circles_round_the_axis(ring),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// The two circles a plane square to a torus's axis cuts, told apart by the pipe
/// of the ring's own radius about the torus's own axis.
///
/// # Errors
///
/// [`Error::RadiusNotPositive`] and [`Error::NotFinite`] never fire: a torus's
/// ring radius is positive and its axis is finite by construction.
fn two_circles_round_the_axis(ring: &Torus) -> Result<SurfaceMeet> {
    let between = Cylinder::new(ring.axis(), ring.ring_radius())?;
    let torus = Surface::Torus(*ring);
    Ok(SurfaceMeet::Curves(
        [Picker::OutsideAPipe(between), Picker::InsideAPipe(between)]
            .into_iter()
            .map(|picker| {
                NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                    and: torus,
                    picked_by: smallvec![picker],
                })
                .expect("picked_by always holds the one picker just built")
            })
            .collect(),
    ))
}

/// The point on `flat` nearest `centre`, as the box it is really known to be
/// inside.
///
/// A box and not a triple, because the division rarely lands on a number `f64`
/// can hold, and an exact triple would buy a certified answer to the wrong
/// question (`docs/decisions.md` §2.5).
fn where_a_plane_is_nearest(flat: &Plane, centre: Point) -> PredicatePoint {
    let normal = flat.normal().components();
    let reach = dot_enclosing(exactly(normal), exactly(centre.coordinates()))
        - Interval::exactly(flat.offset());
    let part = quotient_or_everything(reach, squared_length_enclosing(exactly(normal)));
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
    let part = quotient_or_everything(Interval::exactly(one.radius()), Interval::exactly(gap));
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
/// would have, by construction rather than by derivation.
///
/// **What level 2 leaves is not always level 3's either**, and saying so is the
/// difference between a cascade and a promise. Every discriminant of the four
/// planar pairs and of a ball against a pipe is a polynomial in stored `f64`, so
/// level 3 settles it unless a product leaves the range. The comparison two
/// leaning pipes ask is not a polynomial — it is a length against a length in
/// three different metrics, three square roots deep — and clearing those roots
/// costs eight degrees and runs past what `f64` can square (§29.2, §29.6). Its
/// exact arm answers `None` where that happens, the enclosure is then the whole
/// of the answer, and a knife edge comes back [`Error::Undecidable`].
///
/// # Errors
///
/// [`Error::Undecidable`] when neither the cascade nor the policy could settle
/// the sign, carrying the one record that gave up.
pub(crate) fn ask<P: Predicate>(
    question: &P,
    measures: Measures,
    inputs: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<Decision> {
    let (decision, level) = decide(question);
    let Decision::Undecidable { enclosure } = decision else {
        return Ok(decision);
    };

    let mut records = ProvenanceLog::new();
    let settled = resolve_under_policy(
        Record::new(question.name(), inputs, level, enclosure, measures),
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
pub(crate) struct Question<E, X> {
    pub(crate) named: &'static str,
    pub(crate) enclosing: E,
    pub(crate) exactly: X,
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
pub(crate) fn squared_length_of_a_cross_product(
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

/// `value²` as an enclosure.
fn squared_enclosing(value: f64) -> Interval {
    Interval::exactly(value) * Interval::exactly(value)
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

#[cfg(test)]
mod tests {
    use rcad_predicates::{Level, SideOfSurface, decide};

    use super::*;
    use crate::{Axis, Isometry};

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

    /// §31.4 criterion 2, and the consumer §22.6 never had.
    ///
    /// Two solids flush against each other on one cylinder describe that
    /// cylinder twice, and the two descriptions come off two different builders
    /// with two different axis directions along one line. `Axis::new` used to
    /// normalize what it was handed without reducing it first, so those two
    /// stored different numbers, and then this pair of surfaces came back as two
    /// pipes crossing rather than as one pipe — which is `Curves` where the
    /// truth is `SameSurface`, and the ON columns of a fuse never fire.
    #[test]
    fn the_same_pipe_built_from_a_longer_axis_direction_is_the_same_pipe() {
        for multiple in [2.0, 3.0, 5.0, 7.0, 11.0, 97.0] {
            let met = crossing(
                &a_pipe([1.0, 2.0, 3.0], [4.0, 0.0, -1.0], 2.0),
                &a_pipe(
                    [multiple, 2.0 * multiple, 3.0 * multiple],
                    [4.0, 0.0, -1.0],
                    2.0,
                ),
            );
            assert_eq!(
                met,
                SurfaceMeet::SameSurface,
                "one pipe, its direction named {multiple} times over"
            );
        }
    }

    /// The same pipe pointed the other way is still one pipe, and still not two.
    #[test]
    fn the_same_pipe_pointed_backwards_is_the_same_pipe() {
        let met = crossing(
            &a_pipe([1.0, 2.0, 3.0], [4.0, 0.0, -1.0], 2.0),
            &a_pipe([-3.0, -6.0, -9.0], [4.0, 0.0, -1.0], 2.0),
        );
        assert_eq!(met, SurfaceMeet::SameSurface);
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

        let pickers: Vec<Picker> = runs
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
                Picker::InFrontOf(half_space) => half_space.side_of_point(on_one_line),
                other => panic!("the picker for two lines is flat, not {other:?}"),
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

    /// The planar special case of the ball-and-pipe row: a ball threaded on a
    /// pipe's axis cuts it in **two circles**, one either side of the plane
    /// through the centre square to the axis (§24.5).
    #[test]
    fn a_ball_threaded_on_a_pipes_axis_cuts_it_in_two_circles() {
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);
        let met = crossing(&a_ball([0.0, 0.0, 0.0], 3.0), &pipe);
        assert_eq!(same_shape(&met), ("curves", 2));

        let SurfaceMeet::Curves(runs) = &met else {
            unreachable!("just checked")
        };
        // The two circles sit at `±√(R² − r²)` up the axis, so a point on the
        // upper one is inside exactly one of the two half-spaces.
        let on_the_upper_circle = PredicatePoint::from([2.0, 0.0, f64::sqrt(5.0)]);
        let sides: Vec<Decision> = runs
            .iter()
            .map(|run| {
                let EdgeRun::PartOfWhereItCrosses { and, picked_by } = run.run() else {
                    panic!("one of two circles is a piece of the crossing: {run:?}");
                };
                assert_eq!(and, &pipe);
                match picked_by[0] {
                    Picker::InFrontOf(half_space) => half_space.side_of_point(on_the_upper_circle),
                    other => panic!("the picker here is flat, not {other:?}"),
                }
            })
            .collect();
        assert_eq!(sides, [Decision::Pos, Decision::Neg]);
    }

    /// A ball too small to reach across the pipe it is threaded on touches it
    /// nowhere; one exactly as wide as the pipe touches it all the way round one
    /// circle, which is a **curve** and not a point.
    #[test]
    fn a_ball_inside_a_pipe_meets_it_nowhere_and_one_that_just_fits_meets_it_all_the_way_round() {
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);
        assert_eq!(
            crossing(&a_ball([0.0, 0.0, 0.0], 1.0), &pipe),
            SurfaceMeet::Nothing
        );

        let met = crossing(&a_ball([0.0, 0.0, 0.0], 2.0), &pipe);
        let run = one_run(&met);
        assert_eq!(run.run(), &EdgeRun::RightRoundWhereItsFacesCross);
    }

    /// A ball off to one side of a pipe, reaching into it: one closed loop, and
    /// a genuine quartic rather than anything that lies in a plane.
    #[test]
    fn a_ball_reaching_into_the_side_of_a_pipe_cuts_one_closed_loop() {
        let met = crossing(
            &a_ball([2.0, 0.0, 0.0], 2.0),
            &a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0),
        );
        let run = one_run(&met);
        assert_eq!(run.run(), &EdgeRun::RightRoundWhereItsFacesCross);
    }

    /// Criterion 4's tangent ball-and-pipe row: a ball resting against the
    /// outside of a pipe answers with the one point they share, and that point
    /// is on both.
    #[test]
    fn a_ball_resting_against_the_outside_of_a_pipe_touches_it_at_one_point() {
        let ball = a_ball([5.0, 0.0, 0.0], 3.0);
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);
        let SurfaceMeet::TouchAtOnePoint { at } = crossing(&ball, &pipe) else {
            panic!("a ball five out with a radius of three rests on a pipe of radius two");
        };
        let (Surface::Sphere(ball), Surface::Cylinder(pipe)) = (ball, pipe) else {
            unreachable!("just built them")
        };
        assert_eq!(ball.side_of(at), Decision::Zero);
        assert_eq!(pipe.side_of(at), Decision::Zero);
    }

    #[test]
    fn a_ball_clear_of_a_pipe_meets_it_nowhere() {
        assert_eq!(
            crossing(
                &a_ball([9.0, 0.0, 0.0], 3.0),
                &a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0),
            ),
            SurfaceMeet::Nothing
        );
    }

    /// The one shape this vocabulary cannot say: a ball resting on the far
    /// inside of a pipe leaves two loops that touch, and a curve that runs
    /// through itself is neither one closed curve nor two pieces.
    #[test]
    fn a_ball_resting_on_the_far_inside_of_a_pipe_crosses_it_in_a_curve_with_no_name() {
        // A centre one out from the axis and a radius of three reaches exactly
        // `1 + 3 = 4` across, which is the pipe's whole width, so the two loops
        // pinch together at the far side.
        assert_eq!(
            intersect(
                &a_ball([1.0, 0.0, 0.0], 3.0),
                &a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0),
                &Policy::certified(),
                &mut ProvenanceLog::new()
            ),
            Err(Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo)
        );
    }

    /// The planar special case of the pipe-and-pipe row: two pipes down one axis
    /// are the same surface when their radii agree and never meet when they do
    /// not (§24.5).
    #[test]
    fn two_pipes_down_one_axis_are_one_surface_or_meet_nowhere() {
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);
        assert_eq!(
            crossing(&pipe, &a_pipe([0.0, 0.0, -1.0], [0.0, 0.0, 7.0], 2.0)),
            SurfaceMeet::SameSurface,
            "the same pipe written the other way up, from another point on its own axis"
        );
        assert_eq!(
            crossing(&pipe, &a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 3.0)),
            SurfaceMeet::Nothing
        );
    }

    /// Two pipes side by side, overlapping: two endless lines, told apart by the
    /// plane that holds both axes.
    #[test]
    fn two_pipes_side_by_side_and_overlapping_cross_in_two_endless_lines() {
        let met = crossing(
            &a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0),
            &a_pipe([0.0, 0.0, 1.0], [3.0, 0.0, 0.0], 2.0),
        );
        assert_eq!(same_shape(&met), ("curves", 2));
        let SurfaceMeet::Curves(runs) = &met else {
            unreachable!("just checked")
        };
        // The lines are at `x = 1.5, y = ±√(4 − 2.25)`, and a half-space that
        // picks one of them has to hold all the way along it.
        let far_up_one_line = PredicatePoint::from([1.5, f64::sqrt(1.75), 1e6]);
        let sides: Vec<Decision> = runs
            .iter()
            .map(|run| {
                let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
                    panic!("one of two lines is a piece of the crossing: {run:?}");
                };
                match picked_by[0] {
                    Picker::InFrontOf(half_space) => half_space.side_of_point(far_up_one_line),
                    other => panic!("the picker here is flat, not {other:?}"),
                }
            })
            .collect();
        assert_eq!(sides, [Decision::Pos, Decision::Neg]);
    }

    /// A thin pipe whose axis lies on a much fatter pipe's skin, which is the
    /// pair the overnight fuzzer turned up (`docs/decisions.md` §93).
    ///
    /// The answer is two lines at every ratio. It used to be two lines asked
    /// thin-first and a refusal asked fat-first, from `2²⁷` thin radii up — a 67 m
    /// pipe against a 1 m one, so the ratios here are ordinary engineering and
    /// not a fuzzer's reach. `crossing` asks both ways round and insists the log
    /// is empty, so nothing here is allowed to be approximated into agreeing.
    #[test]
    fn two_pipes_side_by_side_at_an_extreme_radius_ratio_still_cut_two_lines() {
        for thin_radius in [1.0, 1.984375, 1000.0] {
            for power in [26, 27, 40, 60, 100] {
                let fat_radius = thin_radius * f64::powi(2.0, power);
                let met = crossing(
                    &a_pipe([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], fat_radius),
                    &a_pipe([1.0, 0.0, 0.0], [0.0, fat_radius, 0.0], thin_radius),
                );
                assert_eq!(
                    same_shape(&met),
                    ("curves", 2),
                    "a {thin_radius} radius pipe on the skin of one 2^{power} times fatter"
                );
            }
        }
    }

    /// The rest of that row, so the test above cannot pass by answering "two
    /// lines" to everything at an extreme ratio.
    ///
    /// Every distance is a whole number of the thin radius against a fat radius
    /// that is a power of two, so all six pairs are exactly representable and
    /// `Policy::certified()` has to decide every one of them.
    #[test]
    fn an_extreme_radius_ratio_still_tells_the_side_by_side_shapes_apart() {
        let fat_radius = f64::powi(2.0, 40);
        let fat = a_pipe([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], fat_radius);
        for (apart, expected) in [
            (0.0, ("nothing", 0)),
            (fat_radius - 2.0, ("nothing", 0)),
            (fat_radius - 1.0, ("curves", 1)),
            (fat_radius, ("curves", 2)),
            (fat_radius + 1.0, ("curves", 1)),
            (fat_radius + 2.0, ("nothing", 0)),
        ] {
            let thin = a_pipe([1.0, 0.0, 0.0], [0.0, apart, 0.0], 1.0);
            assert_eq!(
                same_shape(&crossing(&fat, &thin)),
                expected,
                "axes {} thin radii off the fat skin",
                apart - fat_radius
            );
        }
    }

    /// Criterion 4's tangent pipe-and-pipe rows, side by side: two pipes resting
    /// against each other from outside, and one resting inside the other, each
    /// answer with **one line**.
    #[test]
    fn two_pipes_side_by_side_and_resting_against_each_other_cross_in_one_line() {
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);
        for beside in [
            a_pipe([0.0, 0.0, 1.0], [5.0, 0.0, 0.0], 3.0),
            a_pipe([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 1.0),
        ] {
            let met = crossing(&pipe, &beside);
            let run = one_run(&met);
            assert_eq!(run.run(), &EdgeRun::Straight, "against {beside:?}");
        }
    }

    /// A thin pipe run through a fat one at an angle: **two closed loops**, one
    /// either side of the plane through the fat pipe's axis that the thin one's
    /// direction is square to.
    #[test]
    fn a_thin_pipe_run_through_a_fat_one_leaves_two_closed_loops() {
        let thin = a_pipe([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0);
        let fat = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 3.0);
        let met = crossing(&thin, &fat);
        assert_eq!(same_shape(&met), ("curves", 2));

        let SurfaceMeet::Curves(runs) = &met else {
            unreachable!("just checked")
        };
        // The thin pipe leaves the fat one at `x = ±√(9 − y²)`, so a point on
        // the loop out along `+x` is inside exactly one half-space.
        let on_one_loop = PredicatePoint::from([f64::sqrt(8.0), 1.0, 0.0]);
        let sides: Vec<Decision> = runs
            .iter()
            .map(|run| {
                let EdgeRun::PartOfWhereItCrosses { and, picked_by } = run.run() else {
                    panic!("one of two loops is a piece of the crossing: {run:?}");
                };
                assert_eq!(and, &fat, "the picked run names the pipe it goes round");
                match picked_by[0] {
                    Picker::InFrontOf(half_space) => half_space.side_of_point(on_one_loop),
                    other => panic!("the picker here is flat, not {other:?}"),
                }
            })
            .collect();
        assert_eq!(sides, [Decision::Pos, Decision::Neg]);
    }

    /// Two pipes of the same thickness leaning across each other, well clear and
    /// overlapping.
    #[test]
    fn two_leaning_pipes_meet_nowhere_when_they_are_clear_and_in_one_loop_when_they_overlap() {
        let pipe = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);
        assert_eq!(
            crossing(&pipe, &a_pipe([1.0, 0.0, 0.0], [0.0, 9.0, 0.0], 2.0)),
            SurfaceMeet::Nothing,
            "nine apart with two radii of two between them"
        );

        let met = crossing(&pipe, &a_pipe([1.0, 0.0, 0.0], [0.0, 3.0, 0.0], 2.0));
        let run = one_run(&met);
        assert_eq!(run.run(), &EdgeRun::RightRoundWhereItsFacesCross);
    }

    /// Criterion 4's tangent pipe-and-pipe row, leaning: two pipes at right
    /// angles exactly `r₁ + r₂` apart rest against each other at one point, and
    /// that point is on both.
    #[test]
    fn two_leaning_pipes_resting_against_each_other_touch_at_one_point() {
        let one = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0);
        let other = a_pipe([1.0, 0.0, 0.0], [0.0, 5.0, 0.0], 3.0);
        let SurfaceMeet::TouchAtOnePoint { at } = crossing(&one, &other) else {
            panic!("five apart with radii of two and three rest against each other");
        };
        let (Surface::Cylinder(one), Surface::Cylinder(other)) = (one, other) else {
            unreachable!("just built them")
        };
        assert_eq!(one.side_of(at), Decision::Zero);
        assert_eq!(other.side_of(at), Decision::Zero);
    }

    /// Two pipes of the same thickness whose axes meet leave two ellipses that
    /// cross each other twice, and that is the same curve-through-itself the
    /// ball-and-pipe row refuses.
    #[test]
    fn two_leaning_pipes_of_the_same_thickness_whose_axes_meet_cross_in_a_curve_with_no_name() {
        assert_eq!(
            intersect(
                &a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0),
                &a_pipe([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], 2.0),
                &Policy::certified(),
                &mut ProvenanceLog::new()
            ),
            Err(Error::ACrossingThatTouchesItselfIsNeitherOneCurveNorTwo)
        );
    }

    /// Where the two stored directions have different squared lengths the exact
    /// level declines and the enclosure does the whole job, which is right for
    /// everything that is not on a knife edge (§29).
    #[test]
    fn two_leaning_pipes_whose_directions_are_not_the_same_length_lean_on_the_enclosure() {
        let one = a_pipe([1.0, 2.0, 3.0], [0.0, 0.0, 0.0], 1.0);
        let other = a_pipe([3.0, -1.0, 1.0], [0.0, 0.0, 20.0], 1.0);
        assert_eq!(crossing(&one, &other), SurfaceMeet::Nothing);
    }

    /// **A placement is the way into the cone-and-pipe refusal, not an odd way
    /// of writing an apex.** Both surfaces reach [`intersect`] with their
    /// locations already composed, so what the coaxiality question reads is the
    /// *placed* pair.
    ///
    /// A frustum sitting in a bore is the ordinary spelling: one line, two axis
    /// origins at different points along it. Turned about anything but a
    /// coordinate direction, the two origins round apart, `(a − o_p)` stops
    /// being a multiple of `w_p` and the apex question answers
    /// [`Decision::Pos`]. The two directions survive, because both are the same
    /// triple through the same arithmetic.
    ///
    /// Two origins written as one point survive too, and that is the boundary:
    /// the same point through the same arithmetic lands in the same place.
    #[test]
    fn a_frustum_in_a_bore_stops_sharing_its_axis_as_soon_as_it_is_turned_off_the_lattice() {
        let up = Axis::new(Point::from([0.0, 0.0, 0.0]), Vector::from([0.0, 0.0, 1.0]))
            .expect("z is a direction");
        let cone = Cone::new(up, 1.0).expect("a positive slope");
        let bore = |height: f64| {
            Cylinder::new(
                Axis::new(
                    Point::from([0.0, 0.0, height]),
                    Vector::from([0.0, 0.0, 1.0]),
                )
                .expect("z is a direction"),
                2.0,
            )
            .expect("a radius")
        };

        assert_eq!(
            whether_a_cone_and_a_pipe_share_an_axis(&cone, &bore(3.0)),
            Decision::Zero,
            "on the lattice the pair is coaxial and the exact predicate says so"
        );

        let turn = Axis::new(Point::from([0.0, 0.0, 0.0]), Vector::from([1.0, 1.0, 1.0]))
            .expect("a diagonal is a direction");
        let survivors: Vec<i32> = (1..=64)
            .filter(|step| {
                let placement =
                    Isometry::rotation(&turn, f64::from(*step) * 0.05).expect("a finite angle");
                whether_a_cone_and_a_pipe_share_an_axis(
                    &cone.transformed(&placement),
                    &bore(3.0).transformed(&placement),
                ) == Decision::Zero
            })
            .collect();
        assert!(
            survivors.len() <= 4,
            "62 of these 64 turns cost the pair its axis when this was measured, and the two \
             that kept it kept it by luck of the rounding; {} keeping it means the placement \
             has stopped reaching the apex question at all: {survivors:?}",
            survivors.len()
        );

        for step in 1..=64 {
            let placement =
                Isometry::rotation(&turn, f64::from(step) * 0.05).expect("a finite angle");
            assert_eq!(
                whether_a_cone_and_a_pipe_share_an_axis(
                    &cone.transformed(&placement),
                    &bore(0.0).transformed(&placement),
                ),
                Decision::Zero,
                "one origin written twice lands in one place however it is turned"
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
    /// It is a **gap**, and the second half of this test says which family of
    /// fixes reaches it. The sentence that used to stand here — *"no scaling of
    /// a unit direction reaches the answer"* — is true and answers the wrong
    /// question: nobody was proposing to scale the direction. What reaches it is
    /// scaling the **expansion**, which [`Exact::square_minus`] already does one
    /// question up (§31.4), and which [`Exact::dot_of`] does not do at all.
    /// Multiplying both triples by one positive power of two rounds nothing and
    /// cannot change the sign of a dot product, and at `2^300` the sign is there
    /// to be read.
    ///
    /// Closing it is a change to `Exact` and therefore to every predicate in the
    /// kernel that reaches level three, which is a lane and not a gate. §39
    /// records it with its consumer.
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

        let normal = flat.as_plane().expect("a plane").normal().components();
        let Surface::Cylinder(along) = &pipe else {
            panic!("a pipe");
        };
        let direction = along.axis().direction().components();
        assert_eq!(
            Exact::dot_of(normal, direction).as_ref().map(Exact::sign),
            None,
            "where they stand, the product falls off the bottom of the range"
        );

        let up = 2f64.powi(300);
        let slid = |triple: [f64; 3]| triple.map(|component| component * up);
        assert_eq!(
            Exact::dot_of(slid(normal), slid(direction))
                .as_ref()
                .map(Exact::sign),
            Some(Decision::Pos),
            "the same two triples, both multiplied by a power of two, have a sign"
        );
    }

    /// **The refusal §29.8 left standing, closed.** This is §31.7's third
    /// criterion, and it replaces the demonstration that used to assert the
    /// opposite.
    ///
    /// A plane exactly along a pipe's axis and inside it cuts two lines, and
    /// telling them apart needs a stored normal exactly across the axis. Here
    /// the direction wanted is `(1,2,3) × (2,-1,0)`, and a `Plane` missed it
    /// twice over: the cross product is already not exactly square to the axis
    /// when it is computed in `f64`, and reducing and scaling it to length one
    /// would lose the squareness even if it were. Both refusals are gone,
    /// because a picker is a [`HalfSpace`] now and one of those keeps whatever
    /// normal it is handed.
    ///
    /// The three assertions are the property, the consequence and the answer.
    /// The normal really is exactly square to the axis the pipe stores; a point
    /// a long way along one of the two lines is therefore on the same side of
    /// the half-space as one at its start; and the pair comes back named, where
    /// the same call used to be `CannotNameWhichPieceOfACrossingThisIs`.
    #[test]
    fn two_lines_off_the_lattice_are_told_apart_by_a_half_space_written_square_to_them() {
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

        let axis = Axis::new(Point::origin(), along).expect("a direction");
        let stored_way = axis.direction().components();
        let pipe = Surface::Cylinder(Cylinder::new(axis, 2.0).expect("a radius"));
        let flat = Surface::Plane(
            Plane::from_point_and_normal(Point::new(0.2, 0.0, 0.0), normal)
                .expect("a normal with a length"),
        );

        let met = intersect(
            &flat,
            &pipe,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a plane down the inside of a pipe cuts two lines");
        assert_eq!(same_shape(&met), ("curves", 2));

        let SurfaceMeet::Curves(runs) = &met else {
            panic!("just checked it is two curves");
        };
        for run in runs {
            let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
                panic!("one of two lines is a piece of the crossing: {run:?}");
            };
            let Picker::InFrontOf(half_space) = picked_by[0] else {
                panic!("the picker for two lines is flat: {picked_by:?}");
            };
            assert_eq!(
                Exact::dot_of(half_space.normal().components(), stored_way)
                    .as_ref()
                    .map(Exact::sign),
                Some(Decision::Zero),
                "the picking normal has to be square to the axis with nothing rounded"
            );
        }

        // The consequence, at the far end of a line that has no far end. The
        // two lines run parallel to the axis a hair under two units either side
        // of the flat surface, so a point on one of them is well clear of it —
        // and stays clear however far along it is read, which is the whole of
        // what the squareness buys.
        let across = along.cross(normal).normalized().expect("a direction");
        let foot = Point::origin() + normal * normal.dot(Vector::new(0.2, 0.0, 0.0));
        let on_one_line = foot + across * 1.99;
        let sides: Vec<Decision> = runs
            .iter()
            .map(|run| {
                let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
                    unreachable!("checked above")
                };
                let Picker::InFrontOf(half_space) = picked_by[0] else {
                    unreachable!("checked above")
                };
                let near =
                    half_space.side_of_point(PredicatePoint::from(on_one_line.coordinates()));
                for steps in [1.0, 1e6, 1e12] {
                    let far = on_one_line + along * steps;
                    assert_eq!(
                        half_space.side_of_point(PredicatePoint::from(far.coordinates())),
                        near,
                        "{steps} steps along the line left the half-space that picked it"
                    );
                }
                near
            })
            .collect();
        assert_eq!(sides, [Decision::Pos, Decision::Neg]);
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
                let reach =
                    dot_enclosing(exactly(normal), exactly(centre)) - Interval::exactly(offset);
                squared_length_enclosing(exactly(normal)) * squared_enclosing(radius)
                    - reach * reach
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

    /// The pair the fuzzer found that named two different crossings depending on
    /// which pipe was written first (`docs/decisions.md` §88).
    ///
    /// Two pipes of the same radius, each leaning a hair off square: one tilted
    /// `2.3e-7` in z, the other `7.3e-12` in y. All three perturbations are
    /// load-bearing — put any one of them back and the two orders agreed
    /// already.
    ///
    /// Both orders reach the same third sign question, `ρ₂ − ρ₁ + δ`, whose
    /// enclosure straddles zero, so both go to the exact level. There the answer
    /// turned on `W₁W₂`, which came back as eight entries one way round and nine
    /// the other — the same number, and only the shorter of the two could be
    /// multiplied by what came next. `Exact` is canonical now, so there is one
    /// list of entries and one answer.
    fn a_pipe_leaning_a_hair(along: [f64; 3], through: [f64; 3]) -> Surface {
        a_pipe(along, through, 1.999_999_999_068_677_4)
    }

    #[test]
    fn two_pipes_leaning_a_hair_apart_name_one_crossing_whichever_is_written_first() {
        let one = a_pipe_leaning_a_hair(
            [0.999_999_999_999_972_5, 0.0, 2.347_497_192_393_293_4e-7],
            [256.0, 0.0, 0.0],
        );
        let other = a_pipe_leaning_a_hair(
            [0.0, -7.275_957_593_854_635e-12, -1.0],
            [6.089_020_112_174_685e-30, 0.0, 0.0],
        );

        // `crossing` asks both ways round and fails if they disagree, which is
        // the whole of this test. It also fails on a refusal, and one order used
        // to refuse.
        assert!(matches!(crossing(&one, &other), SurfaceMeet::Curves(_)));
    }
}
