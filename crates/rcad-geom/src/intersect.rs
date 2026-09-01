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
use crate::enclosure::{
    cross_enclosing, dot_enclosing, exactly, gap_enclosing, root_enclosing,
    squared_length_enclosing,
};
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

/// The name the ball-and-pipe crossing goes by in provenance records.
pub const WHERE_A_BALL_AND_A_PIPE_CROSS: &str = "where a ball and a pipe cross";

/// The name the two-pipe crossing goes by in provenance records.
pub const WHERE_TWO_PIPES_CROSS: &str = "where two pipes cross";

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
/// All six. The four whose crossing **lies in a plane** — plane–plane,
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
    let picker = Plane::from_point_and_normal(origin, along.cross(normal))?;

    // The plane runs exactly along the axis — the caller decided that — so the
    // two directions across the pipe are the plane's own normal and `w × n`,
    // each scaled to length one.
    let squared_direction = squared_length_enclosing(exactly(along.components()));
    let squared_normal = squared_length_enclosing(exactly(normal.components()));
    let sideways = exactly(normal.components())
        .map(|part| quotient_enclosing(part, root_enclosing(squared_normal)));
    let up = cross_enclosing(exactly(along.components()), exactly(normal.components()))
        .map(|part| quotient_enclosing(part, root_enclosing(squared_direction * squared_normal)));

    // How far the axis sits in front of the cutting plane, and how far each of
    // the two lines sits from the plane through the axis square to it.
    let in_front = quotient_enclosing(
        dot_enclosing(exactly(normal.components()), exactly(origin.coordinates()))
            - Interval::exactly(flat.offset()),
        root_enclosing(squared_normal),
    );
    let skin = quotient_enclosing(
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
            )]))
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
        )])),
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
    root_enclosing(quotient_enclosing(reach, squared_direction))
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
    let between = Plane::from_point_and_normal(ball.centre(), pipe.axis().direction())?;
    let picker = between.normal().components();
    let along = pipe.axis().direction().components();
    let squared_direction = squared_length_enclosing(exactly(along));

    // How much of the picking plane's own normal runs along the axis, and how
    // much of it does not.
    let up_the_axis = quotient_enclosing(
        dot_enclosing(exactly(picker), exactly(along)),
        root_enclosing(squared_direction),
    );
    let across =
        root_enclosing(squared_length_enclosing(exactly(picker)) - up_the_axis * up_the_axis);
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
                    picked_by: smallvec![Surface::Plane(half_space)],
                })
            })
            .collect(),
    ))
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
        exactly(along).map(|part| quotient_enclosing(part, root_enclosing(squared_direction)));
    let from_the_origin = gap_enclosing(origin, centre);
    let up_the_axis = dot_enclosing(from_the_origin, direction);
    let sideways: [Interval; 3] =
        core::array::from_fn(|axis| from_the_origin[axis] - direction[axis] * up_the_axis);
    let off_the_axis = root_enclosing(squared_length_enclosing(sideways));
    let skin = quotient_enclosing(
        Interval::exactly(pipe.radius()),
        root_enclosing(squared_direction),
    );

    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        Interval::exactly(origin[axis])
            + direction[axis] * up_the_axis
            + quotient_enclosing(sideways[axis], off_the_axis) * skin
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
                enclosing: || {
                    let room = how_much_room_two_side_by_side_pipes_leave_enclosing(one, other);
                    room * room
                        - Interval::exactly(4.0)
                            * squared_enclosing(mine)
                            * squared_enclosing(theirs)
                            * squared_length_enclosing(exactly(my_way))
                            * squared_length_enclosing(exactly(their_way))
                },
                exactly: || {
                    let room = how_much_room_two_side_by_side_pipes_leave_exactly(one, other)?;
                    Some(
                        room.times(&room)?.minus(
                            &Exact::product_of(2.0, 2.0)?
                                .times(&Exact::product_of(mine, mine)?)?
                                .times(&Exact::product_of(theirs, theirs)?)?
                                .times(&Exact::dot_of(my_way, my_way)?)?
                                .times(&Exact::dot_of(their_way, their_way)?)?,
                        ),
                    )
                },
            };
            match ask(&edge, four_lengths, &at, policy, log)? {
                Decision::Neg => two_lines_between_two_pipes(one, other),
                Decision::Zero => Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
                    EdgeRun::Straight,
                )])),
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
/// tells them apart. Each line's own distance from it is `X = √(ρ₁² − y₀²)`,
/// where `y₀` is how far along the line between the axes the two cross.
fn two_lines_between_two_pipes(one: &Cylinder, other: &Cylinder) -> Result<SurfaceMeet> {
    let here = one.axis().origin();
    let along = one.axis().direction();
    let squared_direction = squared_length_enclosing(exactly(along.components()));
    let between = other.axis().origin() - here;
    let towards = between - along * (between.dot(along) / along.length_squared());

    let picker = Plane::from_point_and_normal(here, along.cross(towards))?;
    let sideways = exactly(towards.components());
    let apart = root_enclosing(squared_length_enclosing(sideways));
    let sideways = sideways.map(|part| quotient_enclosing(part, apart));
    let up = cross_enclosing(exactly(along.components()), sideways)
        .map(|part| quotient_enclosing(part, root_enclosing(squared_direction)));

    let [mine, theirs] = [one, other].map(|pipe| {
        quotient_enclosing(
            Interval::exactly(pipe.radius()),
            root_enclosing(squared_length_enclosing(exactly(
                pipe.axis().direction().components(),
            ))),
        )
    });
    let across = quotient_enclosing(
        apart * apart + mine * mine - theirs * theirs,
        Interval::exactly(2.0) * apart,
    );
    let out = root_enclosing(mine * mine - across * across);

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
        )])),
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
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the plane `f64` can
/// write down is not provably that plane: its normal has to be exactly square to
/// the fat pipe's axis, because a loop reaches a long way along that axis and a
/// normal that leans into it reads a different number at either end.
fn two_loops_either_side_of_a_pipe(thin: &Cylinder, fat: &Cylinder) -> Result<SurfaceMeet> {
    let (thin_way, fat_way) = (
        thin.axis().direction().components(),
        fat.axis().direction().components(),
    );
    let leaning = Vector::from(thin_way).dot(Vector::from(fat_way));
    let squared_fat = Vector::from(fat_way).length_squared();
    let across = Vector::from(thin_way) * squared_fat - Vector::from(fat_way) * leaning;
    let between = Plane::from_point_and_normal(fat.axis().origin(), across)?;
    let picker = between.normal().components();

    // A loop runs a long way up the fat pipe, so a picking normal that leans
    // into that axis at all reads a different number at either end of it and
    // picks nothing. This one has to be square to it with nothing rounded.
    if !matches!(
        Exact::dot_of(picker, fat_way).as_ref().map(Exact::sign),
        Some(Decision::Zero)
    ) {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    let [thin_radius, fat_radius] = certified_radii_enclosing(thin, fat);
    let apart = how_far_apart_two_axes_are_enclosing(thin, fat);
    let clear =
        root_enclosing(fat_radius * fat_radius - (apart + thin_radius) * (apart + thin_radius));
    let slop = how_far_a_picking_plane_strayed(&between, across, fat.axis().origin()) * fat_radius
        + how_far_a_picking_plane_sits_off_a_point(&between, fat.axis().origin());
    if clear.lo() <= slop.hi() {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    let fat = Surface::Cylinder(*fat);
    Ok(SurfaceMeet::Curves(
        [between, between.reversed()]
            .into_iter()
            .map(|half_space| {
                NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                    and: fat,
                    picked_by: smallvec![Surface::Plane(half_space)],
                })
            })
            .collect(),
    ))
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
        exactly(way).map(|part| quotient_enclosing(part, root_enclosing(squared)))
    });
    let sideways = cross_enclosing(ways[0], ways[1]);
    let sideways = {
        let length = root_enclosing(squared_length_enclosing(sideways));
        sideways.map(|part| quotient_enclosing(part, length))
    };
    let between = gap_enclosing(here, there);
    let gap = dot_enclosing(between, sideways);

    // How far up the first axis the common perpendicular meets it.
    let leaning = dot_enclosing(ways[0], ways[1]);
    let up = quotient_enclosing(
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
        quotient_enclosing(
            Interval::exactly(pipe.radius()),
            root_enclosing(squared_length_enclosing(exactly(
                pipe.axis().direction().components(),
            ))),
        )
    })
}

/// `δ = |T| / √K`, the distance between two leaning axes, as an enclosure.
///
/// `T = (o₂−o₁)·(w₁×w₂)` and `K = ‖w₁×w₂‖²`, both polynomials in the stored
/// numbers. The caller has already decided `K` is above zero.
fn how_far_apart_two_axes_are_enclosing(one: &Cylinder, other: &Cylinder) -> Interval {
    let [here, there] = [one, other].map(|pipe| pipe.axis().origin().coordinates());
    let [my_way, their_way] = [one, other].map(|pipe| pipe.axis().direction().components());
    let sideways = cross_enclosing(exactly(my_way), exactly(their_way));
    let volume = dot_enclosing(gap_enclosing(here, there), sideways);
    let reach = volume.lo().abs().min(volume.hi().abs());
    let furthest = volume.lo().abs().max(volume.hi().abs());
    let size = if volume.contains_zero() {
        Interval::new(0.0, furthest).expect("a magnitude runs from zero upwards")
    } else {
        Interval::new(reach, furthest).expect("the nearer end of a magnitude is the nearer one")
    };
    quotient_enclosing(size, root_enclosing(squared_length_enclosing(sideways)))
}

/// A number with the same sign as `mine·ρ₁ + theirs·ρ₂ + gap·δ`, exactly, where
/// the arithmetic can reach it.
///
/// **A sign and not the value**, because the value is not a polynomial in the
/// stored numbers — it is a sum of three square roots. What is a polynomial is
/// the comparison once the roots are squared away, and that only works when the
/// two stored directions have exactly the same squared length: then the two
/// radii share a metric, `mine·ρ₁ + theirs·ρ₂` is `(mine·r₁ + theirs·r₂)/√W`,
/// and comparing it against `δ = |T|/√K` is the sign of
/// `(mine·r₁ + theirs·r₂)²·K − W·T²`. Degree eight, and short enough to square.
/// Every pair whose axes are coordinate directions is in that case, which is the
/// whole of §24.18's control.
///
/// `None` otherwise, which leaves the enclosure to decide (§29).
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
    if squared_mine
        .minus(&Exact::dot_of(their_way, their_way)?)
        .sign()
        != Decision::Zero
    {
        return None;
    }

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

    let radii = Exact::of(mine * one.radius()).plus(&Exact::of(theirs * other.radius()));
    // `radii²·K − W·T²`, which is `(radii/√W)² − δ²` scaled by `W·K`, and asked
    // for only where the two signs in front do not already settle it.
    let squared = || {
        Some(
            radii
                .times(&radii)?
                .times(&Exact::squared_length_of(&sideways)?)?
                .minus(&squared_mine.times(&volume.times(&volume)?)?)
                .sign(),
        )
    };

    let sign = match (radii.sign(), gap > 0.0) {
        // A sum of radii below zero, less a distance that is never below zero.
        (Decision::Neg, false) => Decision::Neg,
        // A sum of radii above zero, plus a distance.
        (Decision::Pos, true) => Decision::Pos,
        (Decision::Zero, true) if volume.sign() == Decision::Zero => Decision::Zero,
        (Decision::Zero, true) => Decision::Pos,
        // The two pull against each other, and squaring is what settles it.
        (Decision::Neg, true) => match squared()? {
            Decision::Pos => Decision::Neg,
            Decision::Neg => Decision::Pos,
            settled => settled,
        },
        (Decision::Pos | Decision::Zero, false) => squared()?,
        _ => return None,
    };
    Some(Exact::of(match sign {
        Decision::Pos => 1.0,
        Decision::Zero => 0.0,
        Decision::Neg => -1.0,
        _ => return None,
    }))
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

/// How far the stored normal of a picking plane strayed from the direction it
/// was asked for, as an upper bound on the length of the difference.
fn how_far_a_picking_plane_strayed(picker: &Plane, asked_for: Vector, _through: Point) -> Interval {
    let stored = exactly(picker.normal().components());
    let wanted = exactly(asked_for.components());
    let length = root_enclosing(squared_length_enclosing(wanted));
    let wanted = wanted.map(|part| quotient_enclosing(part, length));
    let strayed: [Interval; 3] = core::array::from_fn(|axis| stored[axis] - wanted[axis]);
    root_enclosing(squared_length_enclosing(strayed))
}

/// How far a picking plane's own offset sits off the point it was built through,
/// as an upper bound.
fn how_far_a_picking_plane_sits_off_a_point(picker: &Plane, through: Point) -> Interval {
    let off = dot_enclosing(
        exactly(picker.normal().components()),
        exactly(through.coordinates()),
    ) - Interval::exactly(picker.offset());
    Interval::exactly(off.lo().abs().max(off.hi().abs()))
}

/// Two endless straight lines running along `along`, told apart by `picker`.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when the picking plane's
/// normal is not exactly square to the lines' own direction, or when the two
/// lines are not provably either side of it.
///
/// **The first of those cannot be tested away and the second can**, which is the
/// whole of what this closes of §28.2's gap. A line is endless, so a normal that
/// leans into its direction by even one rounding reads a different number a long
/// way along it and the half-space holds nothing. But *where* the plane sits
/// between the two lines is a comparison between the lines' own separation and
/// the plane's own rounding, and that is a question with an answer.
fn two_lines_told_apart(
    picker: Plane,
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
                    picked_by: smallvec![Surface::Plane(half_space)],
                })
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
    let part = quotient_enclosing(reach, squared_length_enclosing(exactly(normal)));
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
                    Surface::Plane(half_space) => half_space.side_of_point(on_the_upper_circle),
                    other => panic!("the picker here is a plane, not {other:?}"),
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
                    Surface::Plane(half_space) => half_space.side_of_point(far_up_one_line),
                    other => panic!("the picker here is a plane, not {other:?}"),
                }
            })
            .collect();
        assert_eq!(sides, [Decision::Pos, Decision::Neg]);
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
                    Surface::Plane(half_space) => half_space.side_of_point(on_one_loop),
                    other => panic!("the picker here is a plane, not {other:?}"),
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
}
