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
//! numbers name. Write "does this plane cut this ball" as `R² − (n·c − d)²` and
//! it answers about a ball of radius `R/‖n‖` — a different ball, by about
//! `1e-16·R`. Every discriminant here therefore carries `n·n`, which costs three
//! products and two sums.
//!
//! **A pipe is not one of those.** `Cylinder::side_of` takes the sign of
//! `|(p − o) × w|² − r²(w·w)`, both halves degree two in the stored direction, so
//! a pipe is exactly `r` wide whatever length `w` rounded to
//! (`docs/decisions.md` §118.11). What that costs here is not a `w·w` beside the
//! radius but the **same degree in `w` on every term** of a discriminant:
//! `A + r² − W·R²` asks about a pipe `r/√W` wide, and `A + W(r² − R²)` asks
//! about the one the caller named. Where every term is already degree zero in
//! `w` the factor comes out altogether — a plane parallel to an axis cuts a pipe
//! of radius `r` on `n·n·r² − (n·o − d)²`, and the stored direction's length
//! does not enter at all.
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

use core::cmp::Ordering;

use rcad_predicates::expansion::two_sum;
use rcad_predicates::{
    Decision, Filtered, Interval, Measures, Point as PredicatePoint, Policy, Predicate,
    ProvenanceLog, Record, decide, resolve_under_policy,
};
use smallvec::{SmallVec, smallvec};

use crate::enclosure::{
    cross_enclosing, dot_enclosing, exactly, gap_enclosing, quotient_or_everything, root_enclosing,
    squared_length_enclosing,
};
use crate::exact::{Exact, Nearly, binary_exponent, multiplied_by_a_power_of_two};
use crate::half_space::normals_exactly_square_to;
use crate::{
    Cone, Cylinder, EdgeRun, Error, HalfSpace, HalfSpaceThroughAPoint, NamedRun, Picker, Plane,
    Point, Result, Sphere, Surface, SurfaceMeet, Torus, Vector,
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

/// The name the two-cone crossing goes by in provenance records.
pub const WHERE_TWO_CONES_CROSS: &str = "where two cones cross";

/// The name the cone-and-ball crossing goes by in provenance records.
pub const WHERE_A_CONE_AND_A_BALL_CROSS: &str = "where a cone and a ball cross";

/// The name the plane-and-torus crossing goes by in provenance records.
pub const WHERE_A_PLANE_AND_A_TORUS_CROSS: &str = "where a plane and a torus cross";

/// The name a torus and a pipe on its axis go by in provenance records.
const WHERE_A_TORUS_AND_A_PIPE_CROSS: &str = "where a torus and a pipe cross";

/// The name a torus and a ball on its axis go by in provenance records.
const WHERE_A_TORUS_AND_A_BALL_CROSS: &str = "where a torus and a ball cross";

/// The name a torus and a cone on its axis go by in provenance records.
const WHERE_A_TORUS_AND_A_CONE_CROSS: &str = "where a torus and a cone cross";

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
/// against a ball, a pipe or a cone **on its axis** is circles square to that
/// axis, each named by the plane it lies in — up to two for a pipe or a ball and
/// up to four for a cone, see [`where_a_ring_and_a_round_surface_cross`]. A
/// pipe of the tube's radius lying along the ring's core circle, the pipe of a
/// bar butted onto a bend's cap, is the circle the two touch in and the two
/// halves of a loop, see [`where_a_ring_and_a_pipe_along_its_core_cross`]. A
/// pipe along the axis but off it, past the tube, in the ring's hole or round
/// the whole ring, meets it nowhere, see
/// [`the_heights_a_ring_crosses_a_pipe_at`]. Any other pipe off the axis, and
/// anything else off it or another torus, is not named at all: the crossing
/// runs to degree eight and none of the machinery for that exists. Two
/// tori with one centre, one pair of radii and their axes along one line are the
/// same surface and come back as one, **whichever way each axis points** — see
/// [`one_ring_either_way_round`].
///
/// **A cone against a plane is classified whole and named in every one of its
/// seven shapes** — see [`where_a_plane_and_a_cone_cross`] for the table. **A
/// cone against a pipe, a ball or a second cone is named where the two share an
/// axis**, and every one of those answers is circles square to that axis: two
/// for a pipe, see [`where_a_cone_and_a_pipe_cross`]; two, one or none for a
/// second cone or a ball, see [`where_two_cones_cross`] and
/// [`where_a_cone_and_a_ball_cross`]. Two cones on one axis with one apex meet
/// there and nowhere else. **Two cones on one apex with their axes apart are
/// named too**, as the halves of the two or four rulings through it they share,
/// or the apex alone. Every other pair that does not share an axis is not named
/// at all: the crossing is a degree-four space curve and none of the machinery
/// §29 built for two pipes exists for a cone. Two cones with one apex, one slope
/// and their axes along one line are the same surface and come back as one,
/// **whichever way each axis points** — see [`one_cone_either_way_round`].
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
/// **Nothing is refused for running through itself.** A ball resting on the
/// far inside of a pipe and a thin pipe resting on the inside of a wider one
/// are each tangent at a point of the curve they cross in, and each is named as
/// two loops that meet there, picked out by the two sides of one flat surface
/// through that point.
///
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] for a cone against a
/// pipe, a ball or a second cone that does not share its axis — a second cone
/// on its apex aside — and for a ball on
/// the axis whose skin runs through the apex. **All seven shapes of a plane
/// against a cone are named**, including the parabola and the two branches of
/// the hyperbola, which used to be refused under names of their own.
///
/// [`Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic`] for every plane
/// against a torus but the two families that cut circles, and
/// [`Error::ACrossingBetweenATorusAndAnotherCurvedSurface`] for a torus against
/// anything curved but a ball, a pipe or a cone on its axis, a pipe of its
/// tube's radius along its core circle, and a pipe along its axis clear of
/// its tube.
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
        (Surface::Cone(one), Surface::Cone(other)) if one_cone_either_way_round(one, other) => {
            Ok(SurfaceMeet::SameSurface)
        }
        (Surface::Cone(cone), Surface::Cylinder(pipe))
        | (Surface::Cylinder(pipe), Surface::Cone(cone)) => {
            where_a_cone_and_a_pipe_cross(cone, pipe, policy, log)
        }
        (Surface::Cone(one), Surface::Cone(other)) => {
            where_two_cones_cross(one, other, policy, log)
        }
        (Surface::Cone(cone), Surface::Sphere(ball))
        | (Surface::Sphere(ball), Surface::Cone(cone)) => {
            where_a_cone_and_a_ball_cross(cone, ball, policy, log)
        }
        (Surface::Plane(flat), Surface::Torus(ring))
        | (Surface::Torus(ring), Surface::Plane(flat)) => {
            where_a_plane_and_a_torus_cross(flat, ring, policy, log)
        }
        (Surface::Torus(one), Surface::Torus(other)) if one_ring_either_way_round(one, other) => {
            Ok(SurfaceMeet::SameSurface)
        }
        (
            Surface::Torus(ring),
            partner @ (Surface::Cylinder(_) | Surface::Sphere(_) | Surface::Cone(_)),
        )
        | (
            partner @ (Surface::Cylinder(_) | Surface::Sphere(_) | Surface::Cone(_)),
            Surface::Torus(ring),
        ) => where_a_ring_and_a_round_surface_cross(ring, partner, policy, log),
        (Surface::Torus(_), Surface::Torus(_)) => {
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
    let at = [
        PredicatePoint::from(flat.origin().coordinates()),
        PredicatePoint::from(pipe.axis().origin().coordinates()),
    ];

    let leans = how_the_plane_leans_against_the_axis(flat, pipe);
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

    // Parallel to the axis. The discriminant is `n·n · r² − (n·o − d)²`, an
    // area: the pipe's radius against the distance from its axis to the plane,
    // both squared. The plane's own normal length is carried because the stored
    // normal is only nearly unit; the pipe's is not, because a pipe is `r` wide
    // in the metric of the world and not of its own direction (§118.11). That
    // makes this degree two where it used to be degree four.
    let room = Question {
        named: WHERE_A_PLANE_AND_A_PIPE_CROSS,
        enclosing: || how_much_room_a_plane_leaves_in_a_pipe_enclosing(flat, pipe),
        exactly: || how_much_room_a_plane_leaves_in_a_pipe_exactly(flat, pipe),
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

/// How a plane leans against a pipe's axis: the sign of `n·w`, **proved and
/// never rounded**.
///
/// [`Decision::Zero`] and the plane holds the axis's direction, so it cuts the
/// pipe in two endless lines, one, or none; anything else and it cuts across in
/// one ellipse that closes — [`where_a_plane_and_a_pipe_cross`]'s first sign.
/// [`Decision::Undecidable`] only when a product of a normal component and a
/// direction component falls into the subnormals and the enclosure straddles
/// zero: `n·w` is three products of stored `f64`, which level 3 sums exactly
/// otherwise.
///
/// **No [`Policy`], on purpose**, for the reason
/// [`how_a_plane_leans_against_a_cone`] gives for its own pair: the door in
/// `rcad-topo` that asks whether this crossing is straight must not be told
/// `Zero` by a tolerant policy, because a `Zero` it has not earned orders a
/// closed ellipse along an axis. It gets the cascade and nothing after it.
pub fn how_a_plane_leans_against_a_pipe(flat: &Plane, pipe: &Cylinder) -> Decision {
    decide(&how_the_plane_leans_against_the_axis(flat, pipe)).0
}

/// The sign of `n·w`, as a question rather than an answer, so that
/// [`where_a_plane_and_a_pipe_cross`] and [`how_a_plane_leans_against_a_pipe`]
/// ask **one** expression written **once**.
fn how_the_plane_leans_against_the_axis(flat: &Plane, pipe: &Cylinder) -> impl Predicate {
    let normal = flat.normal().components();
    let along = pipe.axis().direction().components();
    Question {
        named: WHERE_A_PLANE_AND_A_PIPE_CROSS,
        enclosing: move || dot_enclosing(exactly(normal), exactly(along)),
        exactly: move || Exact::dot_of(normal, along),
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
/// is a comparison between their own separation, `√(r² − g²)`, and how far that
/// plane strayed — a question with an answer rather than a condition to be
/// arranged. [`two_lines_told_apart`] asks it, and §29 records what that closed
/// and what stayed.
///
/// One of the two separations that comparison weighs used to be recomputed here
/// from the plane's reading against the axis, and §142.8 found a pair where
/// that reading is a window a hundred times wider than the pipe: an axis lying
/// in the plane `1e10` from the origin. The separation is `√(room/n·n)` for the
/// `room` the caller certified, so it is read off that sum instead.
///
/// # Why a second picker, through the axis's own start
///
/// The half-space rounds `normal · origin` into its offset, so its flat
/// surface can miss the axis by as much as the lines are apart. A plane
/// stored with normal `(1, 2⁻⁵², 0)` at `2`, cutting a pipe of radius `1/2`
/// along `z` through `(1.5 + 2⁻⁵², −1, −1)`, puts its lines `2⁻⁵³` either side
/// of the middle, and the offset rounds the middle `2⁻⁵³ − 2⁻¹⁰⁴` in front of
/// the surface: one line `2⁻¹⁰⁴` behind it, under the width of the
/// enclosures, and the pair was refused. A bar flush with an L plate's inner
/// side, the two turned a quarter turn back, is that plane and pipe (Q2505).
///
/// So where the half-space cannot tell the two apart, a
/// [`HalfSpaceThroughAPoint`] through `origin` with the same facing is asked.
/// Its surface holds the axis exactly, so the middle reads the step alone,
/// `leaning · (−reading / n·n)`, which is exactly zero wherever the facing is
/// exactly square to the cutting plane's normal, as it is here. The
/// half-space is asked first so that every pair named before keeps the picker
/// it had, as §443's pairs of pipes do.
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
    let up =
        cross_enclosing(exactly(along.components()), exactly(normal.components())).map(|part| {
            quotient_or_everything(part, root_enclosing(squared_direction * squared_normal))
        });

    // How far the axis sits in front of the cutting plane, and how far each of
    // the two lines sits from the plane through the axis square to it. The
    // second is `√(room/n·n)` for the very `room` the caller has just certified
    // above zero, so it is read off that sum and not worked out again from the
    // reading: `r² − (reading/‖n‖)²` is the same number written so that it has
    // to cancel, and for an axis far from the origin the reading is wider than
    // the pipe is thick (§142.8).
    let reading = dot_enclosing(exactly(normal.components()), exactly(origin.coordinates()))
        - Interval::exactly(flat.offset());
    let room = how_much_room_a_plane_leaves_in_a_pipe_exactly(flat, pipe)
        .as_ref()
        .map_or_else(
            || how_much_room_a_plane_leaves_in_a_pipe_enclosing(flat, pipe),
            Exact::enclosure,
        );
    let out = root_enclosing(quotient_or_everything(room, squared_normal));

    // The step to the middle of the two lines runs along the cutting plane's
    // own stored normal, so the picking normal's dot with it is an exact sum of
    // stored numbers and the step it describes is the real one.
    let leaning = Exact::dot_of(picker.normal().components(), normal.components())
        .as_ref()
        .map_or_else(
            || {
                dot_enclosing(
                    exactly(picker.normal().components()),
                    exactly(normal.components()),
                )
            },
            Exact::enclosure,
        );
    let from_the_axis_to_the_middle = leaning * quotient_or_everything(-reading, squared_normal);
    let lines = |middle| TwoLinesEitherSideOfThePicker { middle, up, out };

    match two_lines_told_apart(
        Picker::from(picker),
        along.components(),
        &lines(how_far_in_front(picker, origin.coordinates()) + from_the_axis_to_the_middle),
        Surface::Cylinder(*pipe),
    ) {
        Err(Error::CannotNameWhichPieceOfACrossingThisIs) => two_lines_told_apart(
            Picker::from(HalfSpaceThroughAPoint::new(origin, picker.normal())?),
            along.components(),
            &lines(from_the_axis_to_the_middle),
            Surface::Cylinder(*pipe),
        ),
        named => named,
    }
}

/// `n·n·r² − (n·o − d)²` as an enclosure: how much room a plane running along a
/// pipe's axis leaves inside it, as an area.
///
/// **Off the expansion wherever one can be built**, and that is what decides a
/// pipe moved a hair off a plane it would otherwise touch. A bar moved by
/// `2⁻¹⁰⁰⁷` against a plane two from its axis has `2⁻¹⁰⁰⁵` of room, and the
/// interval form rounds the move away before it squares anything: `[0,
/// 8.9e-16]`, which holds zero. The expansion keeps it (`docs/decisions.md`
/// §322). Where what fell off the bottom is more than a rounding of the room,
/// the interval form answers as it always did.
fn how_much_room_a_plane_leaves_in_a_pipe_enclosing(flat: &Plane, pipe: &Cylinder) -> Interval {
    if let Some(room) = how_much_room_a_plane_leaves_in_a_pipe_nearly(flat, pipe)
        && room.is_within_a_rounding()
    {
        return room.enclosure();
    }
    let normal = flat.normal().components();
    let reading = dot_enclosing(exactly(normal), exactly(pipe.axis().origin().coordinates()))
        - Interval::exactly(flat.offset());
    squared_length_enclosing(exactly(normal)) * squared_enclosing(pipe.radius()) - reading * reading
}

/// `n·n·r² − (n·o − d)²`, exactly.
fn how_much_room_a_plane_leaves_in_a_pipe_exactly(flat: &Plane, pipe: &Cylinder) -> Option<Exact> {
    how_much_room_a_plane_leaves_in_a_pipe_nearly(flat, pipe)?.exactly()
}

/// `n·n·r² − (n·o − d)²`, to within what fell off the bottom of the range.
fn how_much_room_a_plane_leaves_in_a_pipe_nearly(flat: &Plane, pipe: &Cylinder) -> Option<Nearly> {
    let normal = flat.normal().components();
    let radius = pipe.radius();
    let reading = Nearly::dot_of(normal, pipe.axis().origin().coordinates())?
        .minus(&Nearly::of(flat.offset()));
    Some(
        Nearly::dot_of(normal, normal)?
            .times(&Nearly::product_of(radius, radius)?)?
            .minus(&reading.times(&reading)?),
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
/// V  = A + W(r² − R²)         the eighth-degree discriminant's inner half
/// Δ  = V² − 4r²AW             its own discriminant
/// ```
///
/// Then `W(R² − (r∓a)²) = −V ± 2r√(AW)`, with `a = √(A/W)` the centre's distance
/// off the axis and `r` the pipe's width as stored — no `√W` under it, because
/// the pipe is `r` wide whatever its direction rounded to (§118.11). Both halves
/// of `V` are degree two in `w`, which is what makes `Δ` a polynomial: the root
/// is what stops the two tangency readings being polynomials on their own, `Δ`
/// is their product, and the pair `(sign V, sign Δ)` settles both signs without
/// ever taking it (§27.1).
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
                    * squared_length_enclosing(exactly(along))
        },
        exactly: || {
            let room = how_far_a_ball_reaches_past_a_pipe_exactly(ball, pipe)?;
            Some(
                room.times(&room)?.minus(
                    &Exact::product_of(2.0, 2.0)?
                        .times(&Exact::product_of(skin, skin)?)?
                        .times(&how_far_off_an_axis_exactly(centre, origin, along)?)?
                        .times(&Exact::dot_of(along, along)?)?,
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
        (Decision::Neg, Decision::Zero) => {
            two_loops_that_meet_where_a_ball_rests_on_the_far_inside_of_a_pipe(ball, pipe)
        }
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// A ball whose centre is exactly on the pipe's axis: two circles, one circle
/// where the ball is inscribed in the pipe, or nothing.
///
/// **This is the planar special case of the ball-and-pipe row** (§24.5), and it
/// is where the marcher gets an answer it can be differenced against: both
/// circles lie in planes square to the axis, at `±√(R² − r²)` from the centre,
/// and each has the pipe's own radius.
fn a_ball_centred_on_a_pipes_axis(
    ball: &Sphere,
    pipe: &Cylinder,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let radius = ball.radius();
    let skin = pipe.radius();
    let at = [ball.centre(), pipe.axis().origin()]
        .map(|place| PredicatePoint::from(place.coordinates()));

    // `R² − r²`, an area: the ball's radius against the pipe's, both in the
    // metric of the world. The stored direction's length used to be carried on
    // the ball's side and cannot be now — both radii mean the same thing
    // (§118.11) — so this is two products where it was five.
    let room = Question {
        named: WHERE_A_BALL_AND_A_PIPE_CROSS,
        enclosing: || squared_enclosing(radius) - squared_enclosing(skin),
        exactly: || Some(Exact::product_of(radius, radius)?.minus(&Exact::product_of(skin, skin)?)),
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
/// `√(D₋/W)` with `D₋ = W·(R² − r²) − A − 2r√(AW)`, which the caller has already
/// decided is above zero. An enclosure and not a number, because the root is
/// taken twice and neither lands on an `f64`.
///
/// This is `W·(R² − (r + a)²)` written out, with `a = √(A/W)` the centre's
/// distance off the axis. The `√W` beside `2r√A` is the whole of what §118.11
/// changed here: a pipe of width `r` reaches `r` out from its axis in the metric
/// of the world, so the term that pairs its radius against a distance is scaled
/// like a distance and not like a squared direction.
///
/// `A` is read off the exact sum and not the enclosure, because the caller has
/// just asked whether the centre is on the axis — the sign of that same `A` —
/// and reached for the exact arm to answer it. Recomputing it in intervals
/// throws that away: for a centre exactly on the axis `1e10` from the origin
/// the enclosure straddles zero, `2r√(AW)` comes back nearly twice the whole of
/// `D₋`, and the root clamps to nothing (§142.8).
fn how_far_apart_two_loops_are(ball: &Sphere, pipe: &Cylinder) -> Interval {
    let centre = ball.centre().coordinates();
    let along = pipe.axis().direction().components();
    let origin = pipe.axis().origin().coordinates();
    let off_the_axis = how_far_off_an_axis_exactly(centre, origin, along)
        .as_ref()
        .map_or_else(
            || how_far_off_an_axis_enclosing(centre, origin, along),
            Exact::enclosure,
        );
    let squared_direction = squared_length_enclosing(exactly(along));
    let reach = squared_direction
        * (squared_enclosing(ball.radius()) - squared_enclosing(pipe.radius()))
        - off_the_axis
        - Interval::exactly(2.0)
            * Interval::exactly(pipe.radius())
            * root_enclosing(off_the_axis * squared_direction);
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

/// Two closed loops of a ball-and-pipe crossing that meet at one point, where
/// the ball rests on the far inside of the pipe, told apart by the flat surface
/// through the ball's centre square to the pipe's axis.
///
/// # Where the loops meet
///
/// This is [`two_loops_either_side_of_a_ball`] with the centre `R − r` off the
/// axis, which is what `V < 0` and `Δ = 0` say: `Δ = 0` is `V = −2r√(AW)`, so
/// `√A = (R − r)√W`. Put the axis along `z` and the centre at `x = R − r`, on
/// `z = 0`; the pipe's point at `θ` round its axis is on the ball where
/// `z² = 2r(R − r)(1 + cos θ)`. So each loop is on its own side of `z = 0` but
/// for `θ = π`, the point of the pipe across its axis from the centre, `R` from
/// the centre and on `z = 0`, where the walls touch. Both loops pass through
/// it, and each reaches `z = ±2√(r(R − r))` at `θ = 0`. The ball of radius 2 at
/// the origin and the pipe of radius 1 up `z` through `(1, 0, 0)`, Viviani's
/// figure of eight, touch at `(2, 0, 0)`, and the loops reach `(0, 0, ±2)`
/// (`a_ball_resting_on_the_far_inside_of_a_pipe_leaves_two_loops_that_meet_where_they_touch`).
///
/// # What has to be exact
///
/// A picker is closed, so each holds its own loop and the touching point, but
/// only where its surface passes exactly through that point
/// ([`two_loops_that_meet_where_a_thin_pipe_rests_inside_a_fat_one`] says why).
/// The facing is the pipe's direction as stored, which a half-space keeps as it
/// is handed, and the touching point is square to it from the centre, so the
/// surface holds that point exactly where it passes exactly through the centre.
/// It is a [`HalfSpace`] where its rounded offset does, and otherwise a
/// [`HalfSpaceThroughAPoint`] through the centre, which always does. No margin
/// is asked: the loops reach the surface by construction.
///
/// # Errors
///
/// Whatever [`HalfSpace::through_a_point_facing`] and
/// [`HalfSpaceThroughAPoint::new`] refuse.
fn two_loops_that_meet_where_a_ball_rests_on_the_far_inside_of_a_pipe(
    ball: &Sphere,
    pipe: &Cylinder,
) -> Result<SurfaceMeet> {
    let along = pipe.axis().direction();
    let centre = ball.centre();
    let surface = HalfSpace::through_a_point_facing(centre, along)?;
    let picker = if it_passes_exactly_through(&surface, centre) {
        Picker::from(surface)
    } else {
        Picker::from(HalfSpaceThroughAPoint::new(centre, along)?)
    };
    let other_side = picker
        .reversed()
        .expect("the picker is flat, so it has another side");

    let and = Surface::Cylinder(*pipe);
    Ok(SurfaceMeet::Curves(
        [picker, other_side]
            .into_iter()
            .map(|picker| {
                NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                    and,
                    picked_by: smallvec![picker],
                })
                .expect("picked_by always holds the one flat picker just built")
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
/// The contact is on the line from the axis out to the centre: `o + hŵ + rê`,
/// with `h` how far along the axis the centre sits and `ê` the unit direction
/// from the axis to it. Both roots come out of numbers the caller has already
/// decided are above zero. `ê` is a unit direction, so the step out along it is
/// the stored `r` itself and nothing divides by the axis length (§118.11).
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
    let skin = Interval::exactly(pipe.radius());

    PredicatePoint::Constructed(core::array::from_fn(|axis| {
        Interval::exactly(origin[axis])
            + direction[axis] * up_the_axis
            + quotient_or_everything(sideways[axis], off_the_axis) * skin
    }))
}

/// `V = A + W·(r² − R²)` as an enclosure.
fn how_far_a_ball_reaches_past_a_pipe_enclosing(ball: &Sphere, pipe: &Cylinder) -> Interval {
    let along = pipe.axis().direction().components();
    how_far_off_an_axis_enclosing(
        ball.centre().coordinates(),
        pipe.axis().origin().coordinates(),
        along,
    ) + squared_length_enclosing(exactly(along))
        * (squared_enclosing(pipe.radius()) - squared_enclosing(ball.radius()))
}

/// `V = A + W·(r² − R²)`, exactly.
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
        .plus(
            &Exact::dot_of(along, along)?.times(
                &Exact::product_of(skin, skin)?.minus(&Exact::product_of(radius, radius)?),
            )?,
        ),
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
/// Write `A = ‖(o₂−o₁) × w₁‖²` and `E = A − W₁(r₁² + r₂²)`, which is
/// `W₁(δ² − r₁² − r₂²)` for the true distance `δ` between the axes. Then
/// `E² − 4r₁²r₂²W₁²` is `W₁²(δ²−(r₁+r₂)²)(δ²−(r₁−r₂)²)`, and its sign alone
/// settles the answer: below zero the two circles of the cross-section cut each
/// other twice, at zero they rest against each other, above zero one is clear
/// of the other or swallowed by it and either way they never meet.
///
/// **The second pipe's own direction is not in any of that**, and that is
/// §118.11 rather than an oversight. The axes run along one line, so `w₂` says
/// nothing the first direction has not already said, and once both radii mean
/// the same thing the discriminant stops asking. It was degree twelve when the
/// two widths were `rᵢ/√Wᵢ` and had to be brought to a common metric; it is
/// degree eight now.
fn where_two_pipes_side_by_side_cross(
    one: &Cylinder,
    other: &Cylinder,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let [here, there] = [one, other].map(|pipe| pipe.axis().origin().coordinates());
    let [mine, theirs] = [one, other].map(Cylinder::radius);
    let [my_way, _] = [one, other].map(|pipe| pipe.axis().direction().components());
    let at = [here, there].map(PredicatePoint::from);
    let four_lengths = Measures::ManyLengths { how_many: 4 };

    let apart = Question {
        named: WHERE_TWO_PIPES_CROSS,
        enclosing: || how_far_off_an_axis_enclosing(there, here, my_way),
        exactly: || how_far_off_an_axis_exactly(there, here, my_way),
    };
    // A pipe is `r` wide whatever length its stored direction rounded to
    // (`docs/decisions.md` §118.11), so the two are the same thickness exactly
    // when `r₁² − r₂²` is zero. This used to be `r₁²W₂ − r₂²W₁`, degree four,
    // because the width the cylinder predicate certified was then `rᵢ/√Wᵢ`.
    let same_thickness = Question {
        named: WHERE_TWO_PIPES_CROSS,
        enclosing: || squared_enclosing(mine) - squared_enclosing(theirs),
        exactly: || Some(Exact::product_of(mine, mine)?.minus(&Exact::product_of(theirs, theirs)?)),
    };

    match ask(&apart, Measures::Area, &at, policy, log)? {
        // One axis, so the two are the same pipe when their radii agree and
        // never meet at all when they do not.
        Decision::Zero => match ask(&same_thickness, Measures::Area, &at, policy, log)? {
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
/// between the axes, and the picking plane is anchored out there with them so
/// that it reads that displacement as nothing.
///
/// # Why the first plane tried is anchored between the lines and not on an axis
///
/// Because the normal is only *exactly* square to `along` when the pipes run
/// along a coordinate direction. Everywhere else
/// [`HalfSpace::through_a_point_square_to`] takes one of its three fallbacks,
/// which its own doc comment bounds at 55° off the direction asked for, and a
/// plane anchored on the first pipe's axis then reads the pair's middle as
/// `across·sin θ` against a half-separation of `X·cos θ`. Both lines land on one
/// side of it once `across/X` passes `cot θ`, which for pipes along
/// `[0.3, 0.5, 0.7]` is a radius ratio of `2.03` and for one of the thirty
/// directions swept in `where_two_side_by_side_pipes_stop_being_told_apart` is
/// `1.32` —
/// and the same pair written the other way round, where `across ≈ 0`, decided.
/// That was §16.10's gap, not a limit: at the refusing ratio the two lines read
/// `−1.75` and `−0.0033` against a plane reading zero, so every `f64` between
/// them separated them.
///
/// Anchoring between the lines removes that term rather than bounding it. What
/// is left of it is this anchor's own rounding, an ulp of `across` against an
/// `X` of about `r₂`, so the lean stops mattering and the radius ratio has to
/// reach about `2⁵²` to bite. `docs/decisions.md` §164.
///
/// # Why it is tried first and not only
///
/// The anchor between the lines is a *computed* point, and so is the normal the
/// ranking hands back. Far from the origin either can cost the answer: §181.2's
/// pair has the computed anchor `162427` off the thin axis in `z`, and a ranked
/// normal whose two components stand `132` decades apart, so one `f64` offset
/// swallows the smaller whole. A plane that does separate its two lines exists —
/// `z = 41581179`, through the thin pipe's *stored* axis, with one of the
/// fallback normals — and nothing about the pair says in advance which plane
/// that will be.
///
/// So the separation is proved rather than assumed, the way `orient2d` retries
/// at a power of two: between the lines first, then each stored axis origin,
/// each with the ranked normal and then every exactly-square fallback, and the
/// first half-space [`two_lines_told_apart`] proves is the one used. Nothing is
/// nudged and no sign is taken that was not certified; a pair none of them
/// proves is refused exactly as before. Today's half-space is the first tried,
/// so a pair named before §184 is named by the same plane.
///
/// # Why there is a second round, and why it only runs after a refusal
///
/// Far out and off the axes, three normals and three anchors are not enough.
/// §188.4 found pairs where **no** `f64` offset along any of the three normals
/// separates the two lines: the one below them and the one above them are
/// neighbours. A plane between them still exists, facing one of the three plus
/// another times a power of two, where that sum rounds nothing and so stays
/// exactly square. Its one working offset is the one nearest where the true
/// middle reads, and no anchor need round to it, so the offsets are read off
/// the exact middle instead ([`offsets_around_the_middle`]).
///
/// That costs a few dozen more tries, so it waits until everything above has
/// refused. `where_two_leaning_pipes_far_out_cross.rs` is the measurement: the
/// six askings §188.4 names are answered, and the sweep's 8000 answer with
/// none wrong.
///
/// # Why `X` is not `√(r₁² − across²)`
///
/// It is, as arithmetic, and that subtraction is where the row used to give up on
/// a pair it had already decided. The two pipes' crossing has been settled by
/// this point — the caller reached here because the discriminant came out below
/// zero — and then naming the two halves of it could fail, which is §16.10's gap
/// and not a limit.
///
/// Write `δ` for the distance between the axes and `rᵢ` for the two widths.
/// A thin pipe whose axis lies on a fat one's skin has `δ = r₁` and `across =
/// r₁ − r₂²/(2r₁)`, so `r₁² − across²` is `r₂²` arrived at by subtracting two
/// numbers that agree to `r₂²/r₁²` of themselves. Past `r₁ ≈ 2²⁶·r₂` — a 67 m
/// pipe against a 1 m one — `f64` has no bits left to hold the answer in, the
/// enclosure straddles zero, and the two lines cannot be shown to be either side
/// of anything. The same pair asked the other way round has `across ≈ 0` and
/// decides, which is what made it an order-dependent answer.
///
/// So `X` is taken from the discriminant instead, which the caller has already
/// found the sign of. Heron's factorisation of `4δ²X² = 4δ²r₁² −
/// (δ²+r₁²−r₂²)²` is `−(δ²−(r₁+r₂)²)(δ²−(r₁−r₂)²)`, and that product is the
/// discriminant divided by `W₁²`. With `δ² = A/W₁` that comes out as
///
/// ```text
/// X²  =  −(E² − 4r₁²r₂²W₁²)  /  (4·A·W₁)
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
    let towards = the_step_across_to_the_other_axis(one, other);

    let sideways = exactly(towards.components());
    let apart = root_enclosing(squared_length_enclosing(sideways));
    let sideways = sideways.map(|part| quotient_or_everything(part, apart));
    let up = cross_enclosing(exactly(along.components()), sideways)
        .map(|part| quotient_or_everything(part, root_enclosing(squared_direction)));

    let there = other.axis().origin();
    let overlap = how_deeply_two_side_by_side_pipes_overlap_exactly(one, other)
        .as_ref()
        .map_or_else(
            || how_deeply_two_side_by_side_pipes_overlap_enclosing(one, other),
            Exact::enclosure,
        );
    let squared_gap =
        how_far_off_an_axis_exactly(there.coordinates(), here.coordinates(), along.components())
            .as_ref()
            .map_or_else(
                || {
                    how_far_off_an_axis_enclosing(
                        there.coordinates(),
                        here.coordinates(),
                        along.components(),
                    )
                },
                Exact::enclosure,
            );
    let out = root_enclosing(quotient_or_everything(
        -overlap,
        Interval::exactly(4.0) * squared_gap * squared_direction,
    ));

    let tell_apart = |picker: HalfSpace| {
        two_lines_told_apart(
            Picker::from(picker),
            along.components(),
            &TwoLinesEitherSideOfThePicker {
                middle: how_far_in_front_of_the_middle(picker, one, other),
                up,
                out,
            },
            Surface::Cylinder(*other),
        )
    };

    let asked_for = along.cross(towards);
    let anchors = [
        between_the_two_lines(here, towards, one.radius(), other.radius()),
        here,
        there,
    ];
    let normals = picking_normals_to_try(anchors[0], along, asked_for)?;

    let first_try = tell_apart(HalfSpace::through_a_point_facing(anchors[0], normals[0])?);
    if first_try.is_ok() {
        return first_try;
    }
    for anchor in anchors {
        for normal in &normals {
            let Ok(picker) = HalfSpace::through_a_point_facing(anchor, *normal) else {
                continue;
            };
            if let Ok(two_runs) = tell_apart(picker) {
                return Ok(two_runs);
            }
        }
    }
    let combined = normals_made_of_two_square_ones(along, asked_for);
    for normal in normals.iter().chain(&combined) {
        for offset in offsets_around_the_middle(*normal, one, other) {
            let Ok(picker) = HalfSpace::new(*normal, offset) else {
                continue;
            };
            if let Ok(two_runs) = tell_apart(picker) {
                return Ok(two_runs);
            }
        }
    }
    first_try
}

/// The picking normals to try, in the order they are tried.
///
/// First the one [`HalfSpace::through_a_point_square_to`] ranks highest for
/// `asked_for`, which is the half-space this row built and refused on before
/// §184 — so a pair that was named before is named by the same plane. Then the
/// exactly-square fallbacks, each turned to lean the way `asked_for` does,
/// because negating both a normal and its offset rounds nothing.
///
/// Which of the three the ranking puts first is a choice and not a decision, and
/// §183.2 measured a pair where the choice reads nothing: `asked_for` is itself
/// exactly square there, so `through_a_point_square_to` takes it unread, and its
/// two components stand `132` decades apart so one `f64` offset swallows the
/// smaller whole — which is where the answer is.
///
/// # Errors
///
/// Whatever [`HalfSpace::through_a_point_square_to`] refuses `through` and
/// `along` for.
fn picking_normals_to_try(
    through: Point,
    along: Vector,
    asked_for: Vector,
) -> Result<SmallVec<[Vector; 4]>> {
    let ranked = HalfSpace::through_a_point_square_to(through, along, asked_for)?.normal();
    let mut normals: SmallVec<[Vector; 4]> = smallvec![ranked];
    normals.extend(
        normals_exactly_square_to(along)
            .into_iter()
            .filter(|normal| *normal != Vector::zero())
            .map(|normal| {
                if normal.dot(asked_for) < 0.0 {
                    -normal
                } else {
                    normal
                }
            }),
    );
    Ok(normals)
}

/// Normals exactly square to `along` that are not among the three
/// [`normals_exactly_square_to`] builds: one of the three plus another times
/// `±2ᵖ` for `p` in `−80..=80`, the family §188.4 searched, each turned to lean
/// the way `asked_for` does.
///
/// **Only a sum that rounded nothing is kept.** Two exactly square vectors add
/// up to one, so where no component rounded the result is square by algebra;
/// where one did it is square only by accident. That is checked first because
/// it is a few flops a component, and of the 966 candidates the three pairs
/// §188.4 names keep between 11 and 26. [`Exact::dot_of`] then confirms each
/// survivor, and [`two_lines_told_apart`] checks it again before it is used.
///
/// **Why the turn matters.** `two_lines_told_apart` asks for the first line in
/// front of the plane and the second behind it, and which line is first follows
/// `asked_for`. Unturned, one of §188.4's pairs was separated by the right
/// plane facing the wrong way and refused.
fn normals_made_of_two_square_ones(along: Vector, asked_for: Vector) -> Vec<Vector> {
    let three = normals_exactly_square_to(along);
    let mut found = Vec::new();
    for (first, second) in [(0, 1), (0, 2), (1, 2)] {
        let [first, second] = [three[first], three[second]];
        for power in -80..=80 {
            for sign in [1.0, -1.0] {
                let scaled = second * (sign * multiplied_by_a_power_of_two(1.0, power));
                let normal = first + scaled;
                let rounded_nothing = (0..3).all(|axis| {
                    two_sum(first.components()[axis], scaled.components()[axis]).1 == 0.0
                });
                if !rounded_nothing || normal == Vector::zero() {
                    continue;
                }
                let square = Exact::dot_of(normal.components(), along.components())
                    .is_some_and(|reading| reading.sign() == Decision::Zero);
                if square {
                    found.push(if normal.dot(asked_for) < 0.0 {
                        -normal
                    } else {
                        normal
                    });
                }
            }
        }
    }
    found
}

/// The `f64` offsets that could put a plane facing `normal` between the two
/// lines: every one inside the enclosure of where the true middle reads,
/// nearest the centre first.
///
/// **Why nothing outside the enclosure.** The offsets that separate the two
/// lines are the ones strictly between the two readings, and the true middle
/// reading is halfway between those. If any `f64` is between them, the `f64`
/// nearest the middle is, and that one lies inside any enclosure of the middle
/// whose ends are `f64`s. §188.4's scratch copy also tried two either side;
/// without them the far-pipe sweep still answers all 8000.
///
/// Empty where the middle cannot be read exactly, or where its enclosure holds
/// more than [`WIDEST_WINDOW_AROUND_THE_MIDDLE`] offsets. Either leaves the pair
/// refused as it was, never answered wrongly: every offset handed back is still
/// proved by [`two_lines_told_apart`].
fn offsets_around_the_middle(
    normal: Vector,
    one: &Cylinder,
    other: &Cylinder,
) -> SmallVec<[f64; WIDEST_WINDOW_AROUND_THE_MIDDLE]> {
    let Ok(through_the_origin) = HalfSpace::new(normal, 0.0) else {
        return SmallVec::new();
    };
    let Some((top, bottom)) = the_middle_exactly(
        through_the_origin,
        one.axis().origin().coordinates(),
        other.axis().origin().coordinates(),
        one.axis().direction().components(),
        one.radius(),
        other.radius(),
    ) else {
        return SmallVec::new();
    };
    let middle = quotient_or_everything(top.enclosure(), bottom.enclosure());
    let mut offsets: SmallVec<[f64; WIDEST_WINDOW_AROUND_THE_MIDDLE]> = SmallVec::new();
    let mut offset = middle.lo();
    while offset <= middle.hi() {
        if offsets.len() == WIDEST_WINDOW_AROUND_THE_MIDDLE {
            return SmallVec::new();
        }
        offsets.push(offset);
        offset = offset.next_up();
    }
    let centre = middle.lo() / 2.0 + middle.hi() / 2.0;
    offsets.sort_by(|a, b| (a - centre).abs().total_cmp(&(b - centre).abs()));
    offsets
}

/// How many `f64` offsets [`offsets_around_the_middle`] walks before giving the
/// normal up. The enclosure is one quotient of two exact sums, and over the
/// far-pipe sweep it held 4 to 6; a window this much wider only comes from a
/// bottom reaching zero, where the quotient is everything.
const WIDEST_WINDOW_AROUND_THE_MIDDLE: usize = 16;

/// The step from one pipe's axis across to the other's, square to the shared
/// direction.
///
/// Written the way it reads — `(c − o) − w·((c − o)·w / w·w)` — this is a
/// subtraction of two numbers that agree to everything the answer is made of,
/// and for two axes far enough apart along `w` it **rounds away to nothing**.
/// Then `apart` is the zero-width box at zero, the sideways direction and the
/// distance across both divide by it, and a pair of pipes whose axes really are
/// a rounding apart comes back unnameable both ways round while one `f64` step
/// either side of it is named in two straight runs (`docs/decisions.md` §145).
///
/// `(w × (c − o)) × w` is that same step multiplied by `w·w`. It is a polynomial
/// in the stored numbers, so [`Exact`] holds it with nothing cancelling. It is
/// written here as `((o − c) × w) × w`, which is the same vector: the only cross
/// product [`Exact`] offers puts the already-exact side first, so crossing with
/// `w` twice turns the step round, and starting from the gap the other way round
/// turns it back.
///
/// **The scale comes back off before the step is handed on.** Every other reader
/// below would have been happy without it: the picker only ranks three normals
/// by which leans nearest `w × step`, `sideways` and `up` are each divided by
/// the step's own length, and `leaning · how_far` is the step multiplied by
/// `across` and divided by that length again — so a positive multiple cancels
/// out of all of them. `across = (δ² + r₁² − r₂²) / 2δ` is the one that
/// cares — it puts the
/// step's length up against two radii, which are not scaled — so what comes back
/// here is a step whose length is the distance between the axes and not `w·w`
/// times it. That division is the whole of the rounding left, one per component,
/// and a division is not a cancellation: what comes out is the true step to
/// within an ulp or two of itself however far along `w` the two axes sit.
///
/// **Which way round it points is not read at all.**
/// The picking half-space leans along `w × step` and `up` is `w × step` again,
/// so turning the step round turns both together: the two runs come back in the
/// other order and nothing else moves. Measured rather than argued — taking the
/// gap the other way round leaves all four tests in
/// `the_step_between_two_pipe_axes_taken_scaled.rs` green. It is written the way
/// its name says all the same, because a reader is owed that.
fn the_step_across_to_the_other_axis(one: &Cylinder, other: &Cylinder) -> Vector {
    let here = one.axis().origin();
    let there = other.axis().origin();
    let along = one.axis().direction();
    let scale = along.length_squared();

    let stepped = Exact::cross_of(
        &Exact::gap_between(there.coordinates(), here.coordinates()),
        along.components(),
    )
    .and_then(|turned| Exact::cross_of(&turned, along.components()))
    .map(|scaled| {
        Vector::new(
            scaled[0].nearest_f64() / scale,
            scaled[1].nearest_f64() / scale,
            scaled[2].nearest_f64() / scale,
        )
    });

    match stepped {
        Some(step) if step.is_finite() => step,
        // Two coordinates whose difference times `w·w` leaves `f64` altogether,
        // or a product on the way there that would have landed among the
        // subnormals. The plain subtraction is what this row has always done and
        // it is what is left when the exact level cannot run.
        _ => {
            let between = there - here;
            between - along * (between.dot(along) / scale)
        }
    }
}

/// The point midway between the two lines two side-by-side pipes cut out of each
/// other, for the picking plane to pass through.
///
/// **Where the plane sits between them is a choice and not a decision**, in the
/// same sense as which of three fallback normals
/// [`HalfSpace::through_a_point_square_to`] picks. Whatever comes back here,
/// `two_lines_told_apart` reads the offset that was actually stored and either
/// proves the two lines straddle it or refuses, which is why this is plain `f64`
/// with no enclosure anywhere in it.
///
/// **That is only true while the caller can see which way a worse point moved.**
/// This said "a worse point costs refusals and never a wrong answer" until §182,
/// and it was false in the one case that matters most here: the
/// picking normal is built exactly square to `towards`, so the reading cannot
/// move along `towards` at all, and a pair far enough from the origin has the
/// *true* step leaning out of that plane by more than the two lines are apart.
/// The caller now reads the picker at both stored axis origins rather than
/// against the rounded direction, and the sentence holds again because the
/// check is no longer blind to the step.
///
/// It is `across / δ` of the way along the step, with `across = (δ² + r₁² − r₂²)
/// / 2δ`, so the `δ` cancels and no square root is taken. The numerator subtracts
/// two nearly equal numbers when one pipe dwarfs the other — that is the
/// subtraction the rest of this row goes out of its way to avoid — and it does
/// not matter here for the same reason the paragraph above gives: it moves the
/// anchor, not the answer.
fn between_the_two_lines(here: Point, towards: Vector, mine: f64, theirs: f64) -> Point {
    let squared_apart = towards.length_squared();
    let anchor =
        here + towards * ((squared_apart + mine * mine - theirs * theirs) / (2.0 * squared_apart));

    // Axes that coincide, or a step that left `f64` on the way here. Both are
    // already refusals: `apart` comes back zero or infinite, the middle divides
    // by its square, and the enclosure the caller builds is everything.
    // Anchoring on the axis hands that refusal back unchanged instead of turning
    // it into a `NotFinite` from a half-space built through a point that is not
    // there.
    if anchor.is_finite() { anchor } else { here }
}

/// How far in front of `picker` the point midway between the two lines two
/// side-by-side pipes cut out of each other sits.
///
/// Write `δ` for the distance between the axes and `rᵢ` for the two widths. The
/// middle sits `(δ² + r₁² − r₂²) / 2δ²` of the way from one axis to the other,
/// so each axis' reading counts for how far the middle is from the *other* one.
/// With `δ² = A/W`, where `A = ‖(c − o) × w‖²` and `W = w·w`, that is
///
/// ```text
/// (at_here·(A − W·(r₁² − r₂²))  +  at_there·(A + W·(r₁² − r₂²)))  /  2A
/// ```
///
/// Both axis origins, `w` and the widths are stored, so top and bottom are
/// polynomials in stored numbers and [`Exact`] holds each with nothing rounded.
/// The one division is where the enclosure is formed. This is §93's technique,
/// and §29.8 named it as untried on this half of the row.
///
/// **Not the step `the_step_across_to_the_other_axis` hands back.** Its squared
/// length is `δ²` to within a rounding of each component, and that rounding is
/// in no enclosure. It did not show while the middle was read through a share
/// whose own width was about as wide. Read as one exact quotient, the window is
/// an ulp of the answer and the step's rounding lands outside it: of 279 636
/// candidate half-spaces over 23 303 pairs off the coordinate directions, 119 763
/// had a window that missed the true middle. §184.7.
///
/// **Not `at_here + share·(at_there − at_here)`.** That is the same number, and
/// with `share` an enclosure its relative width of `2⁻⁵³` lands against the
/// difference of the two readings rather than against the answer. For §181.2's
/// pair the readings are `3.7e21` apart, `share` is within `2.7e-265` of one,
/// and the middle is `9.98e-244` — so the window was `3.7e5` around a number
/// that needed one of `1.3`.
///
/// **The widths and the gap between the origins are slid down by a power of
/// two first** when the top would overflow. `A` and `W·(r₁² − r₂²)` are both
/// degree two in them, top and bottom alike, so the slide cancels out of the
/// quotient and rounds nothing while the slid numbers stay normal. At §181.2's
/// pair `2A` alone is `5.1e307` and a reading is `3.7e21`, so without it the
/// exact level cannot run at all. The slide is the smallest that fits, and zero
/// wherever the top already fits. It is not only pairs far from the origin that
/// need it: fat enough pipes do on their own, and §140's pair, at the origin
/// with the same widths, slides too. §184.
fn how_far_in_front_of_the_middle(picker: HalfSpace, one: &Cylinder, other: &Cylinder) -> Interval {
    let here = one.axis().origin().coordinates();
    let there = other.axis().origin().coordinates();
    let along = one.axis().direction().components();
    the_middle_exactly(picker, here, there, along, one.radius(), other.radius()).map_or_else(
        || {
            let [at_here, at_there] = [here, there].map(|point| how_far_in_front(picker, point));
            let squared_gap = how_far_off_an_axis_enclosing(there, here, along);
            let widths = squared_length_enclosing(exactly(along))
                * (squared_enclosing(one.radius()) - squared_enclosing(other.radius()));
            quotient_or_everything(
                at_here * (squared_gap - widths) + at_there * (squared_gap + widths),
                Interval::exactly(2.0) * squared_gap,
            )
        },
        |(top, bottom)| quotient_or_everything(top.enclosure(), bottom.enclosure()),
    )
}

/// The top and bottom of [`how_far_in_front_of_the_middle`]'s quotient, each
/// exactly, or `None` where a product the exact level needs would round even
/// after the slide.
fn the_middle_exactly(
    picker: HalfSpace,
    here: [f64; 3],
    there: [f64; 3],
    along: [f64; 3],
    mine: f64,
    theirs: f64,
) -> Option<(Exact, Exact)> {
    let normal = picker.normal().components();
    let offset = Exact::of(picker.offset());
    let at_here = Exact::dot_of(normal, here)?.minus(&offset);
    let at_there = Exact::dot_of(normal, there)?.minus(&offset);

    let [x, y, z] = Exact::gap_between(here, there);
    let down = how_far_to_slide_the_lengths_down(
        at_here
            .nearest_f64()
            .abs()
            .max(at_there.nearest_f64().abs()),
        [
            x.nearest_f64(),
            y.nearest_f64(),
            z.nearest_f64(),
            mine,
            theirs,
        ]
        .into_iter()
        .map(f64::abs)
        .fold(0.0, f64::max),
        along.into_iter().map(f64::abs).fold(0.0, f64::max),
    );
    let slid = |length: f64| {
        let moved = multiplied_by_a_power_of_two(length, -down);
        (length == 0.0 || moved.is_normal()).then_some(moved)
    };
    let gap = [x.slid_by(-down)?, y.slid_by(-down)?, z.slid_by(-down)?];
    let [mine, theirs] = [slid(mine)?, slid(theirs)?];

    let squared_gap = Exact::squared_length_of(&Exact::cross_of(&gap, along)?)?;
    let widths = Exact::dot_of(along, along)?
        .times(&Exact::product_of(mine, mine)?.minus(&Exact::product_of(theirs, theirs)?))?;
    let top = at_here
        .times(&squared_gap.minus(&widths))?
        .plus(&at_there.times(&squared_gap.plus(&widths))?);
    Some((top, squared_gap.plus(&squared_gap)))
}

/// The smallest power of two to slide the gap and the widths down by so that a
/// reading as large as `largest_reading` times `A + W·(r₁² − r₂²)` stays inside
/// `f64`.
///
/// Each component of `(c − o) × w` is at most `2·g·v`, writing `g` for the
/// longest length and `v` for the largest component of `w`, so `A` is under
/// `12·g²·v²` and `W·(r₁² − r₂²)` under `3·g²·v²`. Two readings times their sum
/// is then under `2¹¹` times the product of the largest of each, and `f64`
/// holds anything under `2¹⁰²⁴`. Only the lengths slide, so the slide is half
/// of what is left over.
fn how_far_to_slide_the_lengths_down(
    largest_reading: f64,
    longest_length: f64,
    largest_direction: f64,
) -> i32 {
    let binade = |value: f64| {
        if value == 0.0 {
            0
        } else {
            binary_exponent(value).max(0)
        }
    };
    let too_high =
        binade(largest_reading) + 2 * binade(longest_length) + 2 * binade(largest_direction) + 11
            - 1020;
    (too_high.max(0) + 1) / 2
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
/// sin²α · [ r₂² − (δ + r₁·cos θ)² ]
/// ```
///
/// with `α` the angle between the axes, `δ` the distance between them and `rᵢ`
/// the two widths. So the whole shape of the crossing is decided by where
/// `r₂` sits against `δ ± r₁`: above both, the second pipe swallows the first's
/// whole cross-section and leaves two closed loops; between them, one; below
/// both, two again with the pipes' roles the other way round; and past `δ − r₁`
/// on the far side, nothing at all.
///
/// # Why the exact level reaches this at all
///
/// Those three comparisons are between a sum of two radii and a distance. Two of
/// the three numbers are now stored `f64`s outright — a pipe is `r` wide and no
/// root is taken to find that out (§118.11) — and the third is `δ = |T|/√K` for
/// polynomials `T` and `K` in the stored numbers. **One root, and multiplying
/// through by `√K` clears it.** What is left,
/// [`how_two_radii_and_a_gap_compare_exactly`], is a comparison between two
/// degree-six polynomials with no `Wᵢ` anywhere in it.
///
/// Until §118.11 the widths were `rᵢ/√Wᵢ` and there were three roots in three
/// different metrics. Clearing them cost eight degrees and put a degree-twenty
/// expansion against a degree-sixteen one, which `docs/decisions.md` §29
/// measured at five hundred binary orders wide for two ordinary pipes — too wide
/// to square where it stood, so the exact level ran only where the two stored
/// directions happened to have the same length. That restriction is gone: the
/// expansion §29 could not square is not built any more.
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
            Interval::exactly(one.radius()) * Interval::exactly(mine)
                + Interval::exactly(other.radius()) * Interval::exactly(theirs)
                + how_far_apart_two_axes_are_enclosing(one, other) * Interval::exactly(gap)
        },
        exactly: move || how_two_radii_and_a_gap_compare_exactly(one, other, mine, theirs, gap),
    };

    // r₁ + r₂ − δ. Below zero the two are clear of each other.
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

    // r₂ − r₁ − δ. Above zero the second pipe takes the whole of the first's
    // cross-section and what is left is two loops.
    // Zero with the two widths equal is `δ` zero: the axes meet. Zero with
    // them unequal is the first pipe resting on the inside of the second.
    match ask(&reach(-1.0, 1.0, -1.0), Measures::Length, &at, policy, log)? {
        Decision::Pos => return two_loops_either_side_of_a_pipe(one, other),
        Decision::Zero if one.radius() == other.radius() => {
            return four_half_ellipses_where_two_pipes_of_one_width_meet(one, other);
        }
        Decision::Zero => {
            return two_loops_that_meet_where_a_thin_pipe_rests_inside_a_fat_one(one, other);
        }
        Decision::Neg => (),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }

    // r₂ − r₁ + δ, which is the row above with the two pipes the other way
    // round and the sign of the gap flipped. Two pipes of one width never
    // reach its zero, which is `δ` zero again and was answered above.
    match ask(&reach(-1.0, 1.0, 1.0), Measures::Length, &at, policy, log)? {
        Decision::Pos => Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
            EdgeRun::RightRoundWhereItsFacesCross,
        )?])),
        Decision::Zero => two_loops_that_meet_where_a_thin_pipe_rests_inside_a_fat_one(other, one),
        Decision::Neg => two_loops_either_side_of_a_pipe(other, one),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// Two pipes of one width whose axes meet: two ellipses that cross each other
/// at two points, named as four halves.
///
/// # Why four halves and not two ellipses
///
/// With `ê` and `û` the two unit directions and `c` where the axes meet, a point
/// on both pipes has `(p − c)·(û − ê) = 0` or `(p − c)·(û + ê) = 0`
/// (`where_two_equal_pipes_on_crossing_axes_cut` in `walk.rs`), so the crossing
/// is the two ellipses those planes cut. Both pass through `c ± r·n̂`, with `n̂`
/// along `ê × û`, where the two walls touch. That is where the two curves cross,
/// and it is where a fuse has to put a node: so each ellipse is cut there, and
/// each half runs from one of the two points to the other.
///
/// # What picks a half
///
/// Two flat surfaces through `c`, one leaning towards `ê` and one towards `û`,
/// each facing a direction `f` that lies in the flat `ê` and `û` span; how each
/// is found is [`a_flat_surface_through_where_the_walls_touch`]'s. Along the
/// ellipse in the plane square to `û − ê`, a point is
/// `c + cos θ·a·(û + ê)/‖û + ê‖ + sin θ·r·n̂`, and `f` reads nothing along `n̂`,
/// so the point reads `cos θ·a·f·(û + ê)/‖û + ê‖`. Along the other ellipse, in
/// the direction `û − ê`, it reads `cos θ·a·f·(û − ê)/‖û − ê‖`. A facing that
/// leans towards `ê` — reading positive against it, and more against `ê` than
/// against `û` — reads positive along `û + ê` and negative along `û − ê`, and
/// one leaning towards `û` reads positive along both. So each half is strictly
/// inside one corner of the two surfaces but at its two ends, and the four
/// halves are in four different corners: `(+, +)` and `(−, −)` for the first,
/// `(−, +)` and `(+, −)` for the second. A picker is closed, so a corner holds
/// its own half and the two points where they all meet. The lean is asked of
/// each facing exactly, before it is taken.
///
/// **The facings are not the axis directions in general**: bores on `(3, 4, 0)`
/// and `(4, 3, 0)` through `(11, −3.25, 0.5)` are picked by `(0, 1, 0)` and
/// `(1, 0, 0)`, the same two as bores on `(1, 0, 0)` and `(0, 1, 0)` through
/// the same point
/// (`bores_leaning_at_3_4_0_and_4_3_0_are_picked_by_the_facings_bores_crossing_square_are`).
/// So a caller that needs `ê` and `û` reads them off the two pipes and never
/// off the pickers.
///
/// What has to be exact is that both surfaces pass **exactly** through the
/// points where the halves meet, or a corner holds a stub of a second half. That
/// is three things: `c` is where the two axes meet, asked exactly of each axis;
/// each surface passes exactly through `c`; and each facing lies exactly in the
/// flat of the two stored directions. Then `c ± r·n̂` is on both, because `n̂` is
/// square to that flat. Where `f64` holds no point that is `c`, the axes are
/// asked exactly whether they lie in one flat, and each surface holds the whole
/// of the other axis, and so `c`, instead ([`a_surface_through_the_other_axis`]),
/// or where none of those lies in the flat, passes through a point `f64` holds
/// whose dot with the facing is exactly `c`'s
/// ([`a_surface_through_a_point_beside_the_middle`]).
///
/// **The pair is sorted first**, as the walk sorts it, so `c` is worked out the
/// same way whichever way round the caller wrote the two.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when no two surfaces are
/// found: where `c` is a point `f64` holds ([`where_two_axes_meet_exactly`]),
/// whatever [`a_surface_through_where_the_walls_touch`] refuses, and where it
/// is not, whatever [`a_surface_through_the_other_axis`] and
/// [`a_surface_through_a_point_beside_the_middle`] refuse. A **gap** (§16.10):
/// the four halves are there and four corners apart, and what runs out is a
/// facing written in `f64` that lies exactly in the flat of the two directions,
/// or a point `f64` holds level with `c` along it.
///
/// # Where no flat surface written in `f64` will do
///
/// A surface is a [`HalfSpace`] wherever one of the facings tried passes
/// exactly through `c`. Bores on `(3, 4, 0)` and `(3, 0, 4)` through
/// `(11, −3.25, 0.5)` are picked by facings slid a few `2⁻⁵³` along
/// `(0, 1, −1)`
/// (`bores_on_3_4_0_and_3_0_4_are_picked_by_their_directions_slid_until_they_pass_exactly`),
/// bores on `(0, 3, 4)` and `(4, 3, 0)` by a slide whose step count is solved
/// for (`bores_on_0_3_4_and_4_3_0_are_picked_by_a_slide_solved_for`), and bores
/// on `(−2, 3, 6)` and `(−2, 6, 3)` by a direction in its lowest terms
/// (`bores_on_2_3_6_and_2_6_3_are_picked_by_the_whole_number_triple_one_is_a_multiple_of`).
/// Bores on `(3, 4, 0)` and `(4, 0, 3)` through `(11, −3.25, 0.5)` have none:
/// up to a power of two, `f64` holds twelve facings in their flat, and none of
/// them passes exactly through `c`
/// (`no_flat_surface_written_in_f64_picks_the_halves_of_bores_on_3_4_0_and_4_0_3`).
/// Such a surface is a [`HalfSpaceThroughAPoint`] instead, which holds `c`
/// itself and the pipe's own direction and so passes exactly through `c` for
/// every facing (`docs/decisions.md` §443). Where `c` itself is not a point
/// `f64` holds, every surface is one of those, through the other pipe's
/// stored start or through a point on neither axis.
fn four_half_ellipses_where_two_pipes_of_one_width_meet(
    one: &Cylinder,
    other: &Cylinder,
) -> Result<SurfaceMeet> {
    let (first, second) = if Surface::Cylinder(*one)
        .how_it_sorts_against(&Surface::Cylinder(*other))
        == Ordering::Greater
    {
        (other, one)
    } else {
        (one, other)
    };
    let [first_way, second_way] = [first, second].map(|pipe| pipe.axis().direction());
    let pickers = match where_two_axes_meet_exactly(first, second) {
        Some(middle) => {
            if !(middle.is_finite()
                && [first, second]
                    .iter()
                    .all(|pipe| it_lies_exactly_on_the_axis(middle, pipe)))
            {
                return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
            }
            (
                a_surface_through_where_the_walls_touch(middle, first_way, second_way)?,
                a_surface_through_where_the_walls_touch(middle, second_way, first_way)?,
            )
        }
        None if the_two_axes_lie_exactly_in_one_flat(first, second) => {
            let Some(middle) = where_two_axes_meet_in_fractions(first, second) else {
                return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
            };
            let towards = |own: &Cylinder, other: &Cylinder| match a_surface_through_the_other_axis(
                own, other,
            )? {
                Some(picker) => Ok(Some(picker)),
                None => a_surface_through_a_point_beside_the_middle(
                    &middle,
                    own.axis().direction(),
                    other.axis().direction(),
                ),
            };
            (towards(first, second)?, towards(second, first)?)
        }
        None => return Err(Error::CannotNameWhichPieceOfACrossingThisIs),
    };
    let (Some(towards_first), Some(towards_second)) = pickers else {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    };
    let surfaces = [towards_first, towards_second];

    let and = Surface::Cylinder(*other);
    let corners = [[true, true], [false, false], [false, true], [true, false]];
    Ok(SurfaceMeet::Curves(
        corners
            .into_iter()
            .map(|corner| {
                let [mine, theirs] = [0, 1].map(|which| {
                    if corner[which] {
                        surfaces[which]
                    } else {
                        surfaces[which]
                            .reversed()
                            .expect("both surfaces are flat, so each has another side")
                    }
                });
                NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                    and,
                    picked_by: smallvec![mine, theirs],
                })
                .expect("picked_by always holds the two flat pickers just built")
            })
            .collect(),
    ))
}

/// Where the axes of `first` and `second` meet, worked out exactly from the
/// stored numbers — or `None` when a coordinate of that point is not a number
/// `f64` holds, or an exact sum on the way cannot be formed.
///
/// With `o₁`, `ê` the first axis, `o₂`, `û` the second and `w = ê × û`, the
/// point is `o₁ + ê·N/D` with `N = ((o₂ − o₁) × û)·w` and `D = w·w`, so each
/// coordinate is `(o₁·D + ê·N)/D`, read one component at a time. The same
/// formula worked in `f64` ([`crate::walk::where_two_axes_meet`]) rounds on the
/// way and can miss a point `f64` holds: bars along `x` through `(9, 0, 0)` and
/// turned 125° about `z` through `(11, 0, 0)` meet at `(11, 0, 0)` exactly, and
/// the `f64` formula lands beside it
/// (`two_thin_bars_whose_axes_meet_where_f64_holds_the_point_are_named_in_four_halves`).
/// Two lines that are not parallel meet in one point, so wherever the `f64`
/// formula lands on both axes this finds the same point.
fn where_two_axes_meet_exactly(first: &Cylinder, second: &Cylinder) -> Option<Point> {
    let middle = where_two_axes_meet_in_fractions(first, second)?;
    let [x, y, z] = middle
        .tops
        .each_ref()
        .map(|top| the_f64_a_quotient_is(top, &middle.bottom));
    Some(Point::new(x?, y?, z?))
}

/// A point worked out exactly: `tops[i] / bottom` along each axis, with
/// `bottom` above nought.
struct AFractionalPoint {
    tops: [Exact; 3],
    bottom: Exact,
}

/// The point [`where_two_axes_meet_exactly`] rounds, as the exact fractions
/// `(o₁·D + ê·N)/D`, or `None` when an exact product on the way cannot be
/// formed.
fn where_two_axes_meet_in_fractions(
    first: &Cylinder,
    second: &Cylinder,
) -> Option<AFractionalPoint> {
    let [(here, mine), (there, theirs)] = [first, second].map(|pipe| {
        (
            pipe.axis().origin().coordinates(),
            pipe.axis().direction().components(),
        )
    });
    let sideways = Exact::cross_of(&Exact::of_each(mine), theirs)?;
    let across = Exact::cross_of(&Exact::gap_between(here, there), theirs)?;
    let along = Exact::dot_of_two_exact(&across, &sideways)?;
    let bottom = Exact::squared_length_of(&sideways)?;
    let [x, y, z] = [0, 1, 2].map(|axis| {
        Some(
            Exact::of(here[axis])
                .times(&bottom)?
                .plus(&Exact::of(mine[axis]).times(&along)?),
        )
    });
    Some(AFractionalPoint {
        tops: [x?, y?, z?],
        bottom,
    })
}

/// `top / bottom`, when `f64` holds it, or `None` when it lies between two
/// `f64`. `bottom` is above nought.
///
/// The quotient of the two rounded is within a few units in the last place of
/// the true one, so this steps from there one `f64` at a time towards it, asking
/// exactly which side of it each one is on, until one is on it or the side
/// changes. A zero comes back as `+0.0`, as the `f64` formula gives it.
fn the_f64_a_quotient_is(top: &Exact, bottom: &Exact) -> Option<f64> {
    const MOST_STEPS: usize = 8;
    if top.sign() == Decision::Zero {
        return Some(0.0);
    }
    let side_of = |guess: f64| Some(top.minus(&Exact::of(guess).times(bottom)?).sign());
    let mut guess = top.nearest_f64() / bottom.nearest_f64();
    let first_side = side_of(guess)?;
    for _ in 0..MOST_STEPS {
        match side_of(guess)? {
            Decision::Zero => return Some(guess),
            side if side != first_side => return None,
            Decision::Pos => guess = guess.next_up(),
            Decision::Neg => guess = guess.next_down(),
            Decision::Undecidable { .. } | _ => return None,
        }
    }
    None
}

/// Whether the axes of `one` and `other` lie in one flat, asked exactly:
/// `(o₂ − o₁)·(ê × û)` is nought. Two axes that are not parallel meet exactly
/// when it is.
fn the_two_axes_lie_exactly_in_one_flat(one: &Cylinder, other: &Cylinder) -> bool {
    let [(here, mine), (there, theirs)] = [one, other].map(|pipe| {
        (
            pipe.axis().origin().coordinates(),
            pipe.axis().direction().components(),
        )
    });
    Exact::cross_of(&Exact::of_each(mine), theirs)
        .and_then(|sideways| Exact::dot_of_two_exact(&Exact::gap_between(here, there), &sideways))
        .is_some_and(|reach| reach.sign() == Decision::Zero)
}

/// A flat surface that holds the whole of `other`'s axis and leans towards
/// `own` rather than `other`, for two axes that meet where `f64` holds no
/// point — or `None` when no facing tried gives one.
///
/// It passes through `other`'s stored start and faces `other`'s stored
/// direction crossed with `x`, `y` or `z`, either way round. Crossing with one
/// of those only moves numbers about and turns some round, so each facing is
/// exactly square to `other`'s direction, and the surface holds the whole of
/// `other`'s axis exactly: `c`, where the axes meet, with it. A facing that
/// also lies exactly in the flat of the two directions is square to `n̂`, so
/// the surface holds `c ± r·n̂` too, which is all the halves need of it
/// ([`four_half_ellipses_where_two_pipes_of_one_width_meet`]).
///
/// Bars along `x` through `(9, 0, 0)` and turned 80° about `z`, with both
/// stored exactly in `z = 0`, meet at `(11 + 2⁻⁵⁰, 0, 0)`. The surface leaning
/// towards the bar along `x` passes through the turned bar's start and faces
/// that bar's direction crossed with `z`, and the one leaning towards the
/// turned bar passes through `(9, 0, 0)` and faces `(0, 1, 0)`
/// (`two_thin_bars_whose_axes_meet_where_f64_holds_no_point_are_named_in_four_halves`).
///
/// # Errors
///
/// Whatever [`HalfSpaceThroughAPoint::new`] refuses.
fn a_surface_through_the_other_axis(own: &Cylinder, other: &Cylinder) -> Result<Option<Picker>> {
    let [own_way, other_way] = [own, other].map(|pipe| pipe.axis().direction());
    let [x, y, z] = other_way.components();
    for facing in [[0.0, z, -y], [-z, 0.0, x], [y, -x, 0.0]]
        .into_iter()
        .flat_map(|facing| [facing, facing.map(|number| -number)])
        .map(Vector::from)
    {
        assert!(
            Exact::dot_of(facing.components(), other_way.components())
                .is_none_or(|reach| reach.sign() == Decision::Zero),
            "{other_way:?} crossed with x, y or z is {facing:?}, which is square to it"
        );
        if it_lies_exactly_in_the_flat_of(facing, own_way, other_way)
            && it_leans_towards(facing, own_way, other_way)
        {
            return Ok(Some(Picker::from(HalfSpaceThroughAPoint::new(
                other.axis().origin(),
                facing,
            )?)));
        }
    }
    Ok(None)
}

/// A flat surface through a point `f64` holds and on neither axis, which
/// passes exactly through `middle`, where the axes meet, and leans towards
/// `own` rather than `other` — or `None` when no facing tried gives one.
///
/// It is for axes that meet where `f64` holds no point and where no surface
/// [`a_surface_through_the_other_axis`] tries lies in their flat. The facing
/// is `own`, then the sums [`sums_mostly_of_own`] gives, each taken only where
/// it lies exactly in the flat of `own` and `other` and leans towards `own`,
/// so the surface holds `c ± r·n̂` once it holds `c`. The point is the one
/// [`a_point_level_with`] finds.
///
/// Bars along `x` through `(9, 0, 0)` and turned 105° about `z`, the turned
/// one's direction `û` stored `2⁻⁵²` under `z = 0`, meet at `(11 − 2⁻⁵⁰, 0,
/// 0)`, which rounds to `(11 − 2⁻⁴⁹, 0, 0)`. The surface leaning towards the
/// turned bar faces `û`, and the one leaning towards the bar along `x` faces
/// `2·x̂ + û`, since `x̂` reaches only `x`, where no slide `f64` holds makes
/// up `2⁻⁵⁰` at 11. Both facings reach `z` by `−2⁻⁵²`, so
/// sliding `z` by `−4` times the facing's `x` makes up the `2⁻⁵⁰` the rounding
/// left along `x`: the two pass through `(11 − 2⁻⁴⁹, 0, −4·u_x)` and
/// `(11 − 2⁻⁴⁹, 0, −4·(2 + u_x))`
/// (`two_thin_bars_a_hair_off_one_coordinate_flat_whose_axes_meet_off_f64_are_named_in_four_halves`).
///
/// # Errors
///
/// Whatever [`HalfSpaceThroughAPoint::new`] refuses.
fn a_surface_through_a_point_beside_the_middle(
    middle: &AFractionalPoint,
    own: Vector,
    other: Vector,
) -> Result<Option<Picker>> {
    for facing in core::iter::once(own).chain(sums_mostly_of_own(own, other)) {
        if !(it_lies_exactly_in_the_flat_of(facing, own, other)
            && it_leans_towards(facing, own, other))
        {
            continue;
        }
        if let Some(through) = a_point_level_with(middle, facing) {
            return Ok(Some(Picker::from(HalfSpaceThroughAPoint::new(
                through, facing,
            )?)));
        }
    }
    Ok(None)
}

/// A point `f64` holds whose dot with `facing` is exactly `middle`'s, so the
/// flat surface through it facing `facing` passes exactly through `middle` —
/// or `None` when none is found.
///
/// `middle` rounded one coordinate at a time falls short by `s =
/// facing·(middle − rounded)`. Sliding coordinate `j` by `s/facing_j` makes
/// that up, and a slide is kept only where `f64` holds it and the coordinate
/// it lands on, asked exactly, so the further `facing` is from reaching along
/// `j` the larger and the likelier to be held the slide is.
fn a_point_level_with(middle: &AFractionalPoint, facing: Vector) -> Option<Point> {
    let facing = facing.components();
    let short_by = |point: [f64; 3]| {
        (0..3).try_fold(Exact::of(0.0), |sum, axis| {
            let gap = middle.tops[axis].minus(&Exact::of(point[axis]).times(&middle.bottom)?);
            Some(sum.plus(&Exact::of(facing[axis]).times(&gap)?))
        })
    };
    let rounded = middle
        .tops
        .each_ref()
        .map(|top| top.nearest_f64() / middle.bottom.nearest_f64());
    if !rounded.iter().all(|coordinate| coordinate.is_finite()) {
        return None;
    }
    let short = short_by(rounded)?;
    (0..3).filter(|&axis| facing[axis] != 0.0).find_map(|axis| {
        let (top, bottom) = if facing[axis] > 0.0 {
            (
                short.clone(),
                middle.bottom.times(&Exact::of(facing[axis]))?,
            )
        } else {
            (
                Exact::of(0.0).minus(&short),
                middle.bottom.times(&Exact::of(-facing[axis]))?,
            )
        };
        let mut through = rounded;
        through[axis] += the_f64_a_quotient_is(&top, &bottom)?;
        let level = through.iter().all(|coordinate| coordinate.is_finite())
            && short_by(through)?.sign() == Decision::Zero;
        level.then(|| Point::from(through))
    })
}

/// The flat surface [`a_flat_surface_through_where_the_walls_touch`] finds, or
/// where it finds none, a [`HalfSpaceThroughAPoint`] through the exact `middle`
/// facing `own` — or `None` when `own` fails the same two exact checks every
/// facing tried there passes.
///
/// `own` lies in the flat of `own` and `other`, being one of them, and leans
/// towards itself, so the checks fail only where an exact sum cannot be formed.
/// A half-space found is taken first so that every pair named before this
/// picker existed keeps the pickers it had (`docs/decisions.md` §443).
///
/// # Errors
///
/// Whatever [`HalfSpace::through_a_point_facing`] and
/// [`HalfSpaceThroughAPoint::new`] refuse.
fn a_surface_through_where_the_walls_touch(
    middle: Point,
    own: Vector,
    other: Vector,
) -> Result<Option<Picker>> {
    if let Some(surface) = a_flat_surface_through_where_the_walls_touch(middle, own, other)? {
        return Ok(Some(Picker::from(surface)));
    }
    if !(it_lies_exactly_in_the_flat_of(own, own, other) && it_leans_towards(own, own, other)) {
        return Ok(None);
    }
    Ok(Some(Picker::from(HalfSpaceThroughAPoint::new(
        middle, own,
    )?)))
}

/// A flat surface through `middle` that holds both points where the walls touch
/// and leans towards `own` rather than `other`, with nothing rounded in where it
/// sits — or `None` when no direction tried gives one.
///
/// # What the facing has to be
///
/// The surface holds `c ± r·n̂` when it passes exactly through `c` and faces a
/// direction exactly square to `n̂`, which is any direction lying exactly in the
/// flat `own` and `other` span. It puts the halves in the corners
/// [`four_half_ellipses_where_two_pipes_of_one_width_meet`] lists when it reads
/// positive against `own`, and more against `own` than against `other` with both
/// taken to length one: then it reads positive along `û + ê` and on `own`'s side
/// along `û − ê`, as `own` itself does. All three are asked exactly, and a
/// direction an exact sum cannot be formed for is passed over.
///
/// # Why more than `own` is tried
///
/// `own` was scaled to length one, so unless it runs along a coordinate axis its
/// components are long binary fractions and its dot with `c` rounds: the surface
/// then misses `c` by about `1e-17` and is no good. A direction whose components
/// are whole numbers from −2 to 2 dots with `c` to a few small multiples of
/// `c`'s own coordinates, exact whenever those are short, and a coordinate
/// direction dots to one coordinate, exact for every `c`. Such a direction lies
/// in the flat only when the flat holds one, which it does for two axes in one
/// coordinate plane and for two directions that are one whole-number triple
/// scaled down, like `(1, 2, 2)/3` and `(2, 1, 2)/3`.
///
/// A whole-number direction in the flat can still fail the lean: `(0, 1, −1)`
/// in the flat of `(3, 4, 0)/5` and `(3, 0, 4)/5` reads as far along one as back
/// along the other. **Then `own` is slid along it**, a step at a time, each step
/// [`one_unit_in_the_last_place_of_what_it_moves`] long, up to
/// [`LONGEST_SLIDE`] steps either way. A whole-number step keeps the facing in
/// the flat when nothing rounds, and a few units in the last place keep it
/// leaning as `own` does, so what the slide searches for is the one step count
/// whose dot with `c` lands on a number `f64` holds. The constant says why 64
/// steps are enough for short middles and what they are not enough for. Only
/// the shortest whole-number triple along each line in the flat is slid along,
/// since a multiple of it slides along the same line in coarser steps.
///
/// **Then come sums of `own` and `other`**: two to four of `own` and fewer of
/// `other`, either way. Such a sum lies in the flat when nothing rounds, and
/// leans towards `own` as anything holding more of it does. A few of them come
/// out short even from the rounded stored directions: four of `(3, −6, 2)/7` and
/// one of `(2, 3, 6)/7` add to exactly `(2, −3, 2)`. They go after the slide so
/// that a pair the slide names keeps the surfaces it had.
///
/// **Then `own` in its lowest terms** ([`own_in_lowest_terms`]): `(2, 3, 6)/7`
/// is stored as an exact multiple of `(2, 3, 6)` with a 52-bit factor, so its
/// own dot with `c` rounds where the short triple's need not.
///
/// **Last come slides whose step count is solved for** ([`slides_solved_for`]),
/// of `own` in its lowest terms and of `own`, along each way the short slide
/// took, for the middles whose dot has more bits to clear than 64 steps run
/// through. These two go after everything tried before Q1653, so that a pair
/// named then keeps the surfaces it had.
///
/// # Errors
///
/// Whatever [`HalfSpace::through_a_point_facing`] refuses.
fn a_flat_surface_through_where_the_walls_touch(
    middle: Point,
    own: Vector,
    other: Vector,
) -> Result<Option<HalfSpace>> {
    const WHOLE_NUMBERS: [f64; 5] = [0.0, 1.0, -1.0, 2.0, -2.0];
    let whole_numbers_in_the_flat: Vec<Vector> = WHOLE_NUMBERS
        .into_iter()
        .flat_map(|x| {
            WHOLE_NUMBERS
                .into_iter()
                .flat_map(move |y| WHOLE_NUMBERS.into_iter().map(move |z| Vector::new(x, y, z)))
        })
        .filter(|&way| it_lies_exactly_in_the_flat_of(way, own, other))
        .collect();
    let ways_to_slide: Vec<Vector> = whole_numbers_in_the_flat
        .iter()
        .copied()
        .filter(|&way| it_is_the_shortest_whole_number_step_along_its_line(way))
        .collect();
    let slid = (1..=LONGEST_SLIDE)
        .flat_map(|steps| [steps, -steps])
        .flat_map(|steps| {
            ways_to_slide.iter().map(move |&way| {
                own + way
                    * (f64::from(steps) * one_unit_in_the_last_place_of_what_it_moves(own, way))
            })
        });
    let mostly_own = sums_mostly_of_own(own, other);
    let in_lowest_terms = own_in_lowest_terms(own);
    let slid_as_far_as_solved = in_lowest_terms
        .into_iter()
        .chain(core::iter::once(own))
        .flat_map(|start| {
            ways_to_slide
                .iter()
                .flat_map(move |&way| slides_solved_for(middle, start, way))
        });
    for facing in core::iter::once(own)
        .chain(whole_numbers_in_the_flat.iter().copied())
        .chain(slid)
        .chain(mostly_own)
        .chain(in_lowest_terms)
        .chain(slid_as_far_as_solved)
    {
        if !(it_lies_exactly_in_the_flat_of(facing, own, other)
            && it_leans_towards(facing, own, other))
        {
            continue;
        }
        let surface = HalfSpace::through_a_point_facing(middle, facing)?;
        if it_passes_exactly_through(&surface, middle) {
            return Ok(Some(surface));
        }
    }
    Ok(None)
}

/// Two to four of `own` and fewer of `other`, either way: each lies in the
/// flat of the two when nothing rounds, and leans towards `own` as anything
/// holding more of it does.
fn sums_mostly_of_own(own: Vector, other: Vector) -> impl Iterator<Item = Vector> {
    (2..=4).flat_map(move |more| {
        (1..more)
            .flat_map(|less| [less, -less])
            .map(move |less| own * f64::from(more) + other * f64::from(less))
    })
}

/// How many steps a facing is slid either way along a whole-number way before
/// [`a_flat_surface_through_where_the_walls_touch`] gives up on that way.
///
/// One step moves the facing's dot with the middle `c` by `δ = u·(way·c)`, with
/// `u` the step [`one_unit_in_the_last_place_of_what_it_moves`] gives. Write
/// `δ` as an odd number times `2ᵉ`. The dot has to land on the grid `f64` holds
/// at its size, and when the dot has nothing below `2ᵉ` what is left to clear
/// is the `k` bits from `2ᵉ` up to that grid. An odd number has an inverse
/// modulo `2ᵏ`, so `2ᵏ` steps in a row run through every pattern of those `k`
/// bits once, and one of them clears all `k`. The steps from `−N` to `N`,
/// with `0` the stored direction itself, are `2N + 1` in a row, so they reach
/// it whenever `2ᵏ ≤ 2N + 1` — as long as the slide neither carries the dot
/// past a power of two, which moves the grid, nor rounds a component the way
/// moves. Either one only loses a facing: the three exact checks still pass
/// over whatever it gives.
///
/// `N = 64` covers `k ≤ 7`. That is the middle
/// `the_namer_names_this_family_in_four_halves_and_the_walker_draws_every_piece`
/// uses, `(11, −3.25, 0.5)`: every component of that family's directions is
/// above `¼`, so `u ≥ 2⁻⁵⁴`; `way·c` has nothing below `2⁻²`; so `δ` has nothing
/// below `2⁻⁵⁶`. The dot is under `16`, so its grid is at most `2⁻⁴⁹`, and `k ≤ 7`.
/// Measured there at Q1645, before the solved slide below, `64` and `4096` both
/// named 704 pairs of the 1224, and no facing was slid farther than `37·2⁻⁵³`
/// in any component; `4096` made that test ten times as slow, all of it in
/// searches that found nothing.
///
/// **A short slide is not enough everywhere.** A middle whose coordinates are
/// long binary fractions, like a decimal `1.85`, puts `k` near 52. There the
/// step count is solved for rather than searched, by [`slides_solved_for`],
/// which also takes steps finer than this one. And when the dot has a bit below
/// what one step moves, no count clears it while the dot stays the size it is.
/// The solved slide then gives the counts that carry the dot down to a size
/// whose grid holds that bit, as it does for bores on `(0, 3, 4)` and
/// `(4, 3, 0)` through `(11, −3.25, 0.5)`, whose dot goes from about `−1.55` to
/// about `−0.25`. The stored direction in its lowest terms
/// ([`own_in_lowest_terms`]) has a dot with fewer bits to clear.
const LONGEST_SLIDE: i32 = 1 << 6;

/// The coarsest step a slide takes: the unit in the last place of a number from
/// `½` to `1`, where the largest component of a direction of length one lies.
const ONE_UNIT_OF_A_SLIDE: f64 = f64::EPSILON / 2.0;

/// The step a slide of `own` along `way` takes: the smallest unit in the last
/// place among the components of `own` that `way` moves and that are not
/// nothing, and never more than [`ONE_UNIT_OF_A_SLIDE`].
///
/// **The smallest and not one fixed unit**, because a component from `¼` to
/// `½`, like `2/7` in `(2, 3, 6)/7`, sits on a grid of `2⁻⁵⁴`, and its dot with
/// the middle has bits a step of `2⁻⁵³` cannot clear. A step finer than another
/// moved component's grid rounds that component, and the facing it gives then
/// leaves the flat and is passed over, so the finer step costs searches and
/// never a wrong facing. In
/// `the_namer_names_this_family_in_four_halves_and_the_walker_draws_every_piece`
/// a fixed `2⁻⁵³` named 644 of the 1224 pairs and this 704, measured at Q1645
/// before [`slides_solved_for`] was added.
fn one_unit_in_the_last_place_of_what_it_moves(own: Vector, way: Vector) -> f64 {
    own.components()
        .into_iter()
        .zip(way.components())
        .filter(|&(mine, step)| mine != 0.0 && step != 0.0)
        .map(|(mine, _)| f64::from_bits(mine.abs().to_bits() + 1) - mine.abs())
        .fold(ONE_UNIT_OF_A_SLIDE, f64::min)
}

/// Whether `way`, a triple of whole numbers from −2 to 2, is the one kept for
/// its line: it has a component of `±1`, so it is no multiple of a shorter
/// triple, and its first component that is not nothing is positive, so the
/// same line is not slid along twice, once each way.
fn it_is_the_shortest_whole_number_step_along_its_line(way: Vector) -> bool {
    let components = way.components();
    components.iter().any(|component| component.abs() == 1.0)
        && components
            .iter()
            .find(|component| **component != 0.0)
            .is_some_and(|first| *first > 0.0)
}

/// `own` written in its lowest terms: the shortest whole-number triple it is a
/// multiple of, times a power of two that keeps it about as long as `own` — or
/// `None` when that triple is `own` itself up to a power of two.
///
/// A whole-number triple scaled to length one stays an exact multiple of the
/// triple when its components all round by the same factor, and some do:
/// `(2, 3, 6)/7` is stored as exactly `p·2⁻⁵⁴·(2, 3, 6)` with `p = (2⁵⁴ − 1)/7`,
/// since `2/7`, `3/7` and `6/7` each lie the same fraction `1/7` of a unit in the
/// last place above a grid point. Its dot with the middle is `p` times a short
/// sum, and `p` has 52 bits, so that dot rounds where the short triple's does
/// not. A positive multiple of `own` lies in the flat and leans as `own` does.
fn own_in_lowest_terms(own: Vector) -> Option<Vector> {
    let parts = own
        .components()
        .map(|component| a_product_in_whole_numbers(component, 1.0));
    let unit = parts
        .iter()
        .filter(|(whole, _)| *whole != 0)
        .map(|(_, power)| *power)
        .min()?;
    let [Some(x), Some(y), Some(z)] = parts.map(|(whole, power)| shifted_up(whole, power - unit))
    else {
        return None;
    };
    let shared = [x, y, z].into_iter().fold(0, |so_far, whole| {
        greatest_common_factor(so_far, whole.unsigned_abs())
    });
    if shared == 1 {
        return None;
    }
    let shared_bits = 128 - shared.leading_zeros();
    let shared = i128::try_from(shared).expect("a factor of a component below 2¹²⁷");
    let in_lowest_terms = [x, y, z].map(|whole| {
        multiplied_by_a_power_of_two(
            (whole / shared) as f64,
            unit + shared_bits.cast_signed() - 1,
        )
    });
    Some(Vector::new(
        in_lowest_terms[0],
        in_lowest_terms[1],
        in_lowest_terms[2],
    ))
}

/// The largest whole number both `one` and `other` are multiples of, and the
/// other one when either is nothing.
fn greatest_common_factor(one: u128, other: u128) -> u128 {
    if other == 0 {
        one
    } else {
        greatest_common_factor(other, one % other)
    }
}

/// `own` slid along `way` by step counts solved for, one for each stretch of
/// counts whose dot with `middle` lands on a number `f64` holds, and only
/// where the facing's components are numbers `f64` holds too.
///
/// A step is `2ˢ` for `s` from `−53` down to `−53 −` [`FINEST_SOLVED_STEP`].
/// `n` steps give `own + n·2ˢ·way`, whose dot with `middle` is `X + n·Y` in
/// units of the lowest bit either of `own·c` and `2ˢ·(way·c)` has, both worked
/// out in whole numbers. A dot below `2⁵³` units is a number `f64` holds
/// whatever its bits, so every count that puts it there will do. A dot from
/// `2ᵇ` to `2ᵇ⁺¹` units with `b ≥ 53` has to be a whole multiple of `2ᵇ⁻⁵²`
/// units, and when `Y` is odd that is `n ≡ −X·Y⁻¹` modulo `2ᵇ⁻⁵²`, `Y⁻¹` its
/// inverse modulo that power of two. When `Y` is even `X` is odd, the dot has a
/// bit below what one step moves, and only the counts that carry it under
/// `2⁵³` units can do.
///
/// Within each such stretch the count nearest nothing either side is the one
/// given: the facing leans towards `own` over an unbroken run of counts through
/// nothing, so if any count of a stretch on one side leans, the nearest does.
/// Every facing given still has to pass the three exact checks, so a count
/// worked out wrong only costs a facing, and nothing here is a tolerance.
///
/// [`a_flat_surface_through_where_the_walls_touch`] calls this with the stored
/// direction as `own`, and also with that direction in its lowest terms
/// ([`own_in_lowest_terms`]) when it has them. That is a positive multiple of
/// the stored direction, so it lies in the same flat and leans the same way,
/// and its dot with the middle has fewer bits to clear. Measured at Q1653, the
/// lowest terms alone, without the stored direction, named 1096 and 316 of the
/// 1224 pairs in the two count tests where both name 1096 and 328.
fn slides_solved_for(middle: Point, own: Vector, way: Vector) -> Vec<Vector> {
    let (Some(start), Some(moved)) = (
        a_dot_in_whole_numbers(own.components(), middle.coordinates()),
        a_dot_in_whole_numbers(way.components(), middle.coordinates()),
    ) else {
        return Vec::new();
    };
    if moved.0 == 0 {
        return Vec::new();
    }
    (0..=FINEST_SOLVED_STEP)
        .flat_map(|finer| {
            let step = -53 - finer;
            the_step_counts_that_dot_exactly(own, way, start, moved, step)
                .into_iter()
                .filter_map(move |count| own_slid(own, way, step, count))
        })
        .collect()
}

/// `own + count·2^step·way`, each component it moves worked out in whole
/// numbers and so with nothing rounded, or `None` when one of them is not a
/// number `f64` holds.
fn own_slid(own: Vector, way: Vector, step: i32, count: i128) -> Option<Vector> {
    let mut slid = own.components();
    for (mine, along) in slid.iter_mut().zip(way.components()) {
        if along == 0.0 {
            continue;
        }
        let (from_nothing, per_step, unit) = a_slid_component_in_whole_numbers(*mine, along, step)?;
        let whole = per_step
            .checked_mul(count)
            .and_then(|moved| moved.checked_add(from_nothing))
            .filter(|whole| whole.unsigned_abs() < 1 << 53)?;
        *mine = multiplied_by_a_power_of_two(whole as f64, unit);
    }
    Some(Vector::new(slid[0], slid[1], slid[2]))
}

/// How many halvings below `2⁻⁵³` [`slides_solved_for`] takes its step to.
///
/// A step of `2⁻⁵³⁻ʲ` only gives facings whose moved components are under
/// `2⁻ʲ`, so each finer step is a narrower search.
///
/// Measured at Q1653 as the pairs of 1224 that
/// `the_namer_names_this_family_in_four_halves_and_the_walker_draws_every_piece`
/// and `the_namer_names_this_family_crossing_at_a_middle_in_decimal_millimetres`
/// name: `0` names 1072 and 320, `1` and `4` name 1096 and 324, `8` and `16`
/// name 1096 and 328. So `8`, past which nothing more was named.
///
/// **No step coarser than `2⁻⁵³`.** Steps as coarse as `2⁻¹` were tried, and
/// they gave facings like `(−0.5, 2.09e15, −2.09e15)`, which pass the three
/// exact checks with an offset of `−7.5e15`, and which the walk then refused
/// with [`Error::CannotNameWhichPieceOfACrossingThisIs`]. A step of `2⁻⁵³` or
/// finer keeps every moved component under `2⁵³` of a unit no coarser than the
/// step times the way's component, so under `2` for a way of whole numbers
/// from `−2` to `2`.
const FINEST_SOLVED_STEP: i32 = 8;

/// The step counts `n`, each nearest nothing in its stretch, for which
/// `own + n·2^step·way` has components `f64` holds and a dot with the middle
/// `f64` holds, given that dot's two parts in whole numbers: `own·c` as
/// `start` and `way·c` as `moved`, each a whole number and a power of two.
fn the_step_counts_that_dot_exactly(
    own: Vector,
    way: Vector,
    start: (i128, i32),
    moved: (i128, i32),
    step: i32,
) -> Vec<i128> {
    let Some((fewest, most)) = the_step_counts_that_keep_the_facing_in_f64(own, way, step) else {
        return Vec::new();
    };
    let unit = if start.0 == 0 {
        moved.1 + step
    } else {
        start.1.min(moved.1 + step)
    };
    let (Some(at_nothing), Some(per_step)) = (
        shifted_up(start.0, start.1 - unit),
        shifted_up(moved.0, moved.1 + step - unit),
    ) else {
        return Vec::new();
    };
    let within = |(from, to): (i128, i128)| {
        let (from, to) = (from.max(fewest), to.min(most));
        (from <= to).then_some((from, to))
    };
    let held = (1_i128 << 53) - 1;
    let mut counts: Vec<i128> = the_counts_that_land_between(at_nothing, per_step, -held, held)
        .and_then(within)
        .map(|(from, to)| the_counts_nearest_nothing(from, to, 0, 1))
        .unwrap_or_default();
    if per_step % 2 == 0 {
        return counts;
    }
    let (Some(at_fewest), Some(at_most)) = (
        per_step
            .checked_mul(fewest)
            .and_then(|reach| reach.checked_add(at_nothing)),
        per_step
            .checked_mul(most)
            .and_then(|reach| reach.checked_add(at_nothing)),
    ) else {
        return counts;
    };
    let farthest = at_fewest.unsigned_abs().max(at_most.unsigned_abs());
    let highest_binade = (127 - farthest.leading_zeros()).min(125);
    for binade in 53..=highest_binade {
        let bits = binade - 52;
        let remainder = the_count_that_clears_the_low_bits(at_nothing, per_step, bits);
        let modulus = 1_i128 << bits;
        let (bottom, top) = (1_i128 << binade, (1_i128 << (binade + 1)) - 1);
        for (low, high) in [(bottom, top), (-top, -bottom)] {
            if let Some((from, to)) =
                the_counts_that_land_between(at_nothing, per_step, low, high).and_then(within)
            {
                counts.extend(the_counts_nearest_nothing(from, to, remainder, modulus));
            }
        }
    }
    counts
}

/// The step counts, from fewest to most, over which `own + n·2^step·way` has
/// every component a number `f64` holds: each component it moves stays under
/// `2⁵³` units of the finer of its own lowest bit and the step times the way's.
///
/// `n·2^step` itself need not be one: `(−2, 3, 6)/7` slid along `(0, 0, 1)`
/// until its last component is under `⅛` has moved that component by about
/// `−6/7` in steps of `2⁻⁵⁶`, more than `2⁵³` of them.
fn the_step_counts_that_keep_the_facing_in_f64(
    own: Vector,
    way: Vector,
    step: i32,
) -> Option<(i128, i128)> {
    let held = (1_i128 << 53) - 1;
    let mut counts = (i128::MIN, i128::MAX);
    for (mine, along) in own.components().into_iter().zip(way.components()) {
        if along == 0.0 {
            continue;
        }
        let (from_nothing, per_step, _) = a_slid_component_in_whole_numbers(mine, along, step)?;
        let (from, to) = the_counts_that_land_between(from_nothing, per_step, -held, held)?;
        counts = (counts.0.max(from), counts.1.min(to));
    }
    (counts.0 <= counts.1).then_some(counts)
}

/// A component `mine` of `own`, slid by `n·2^step·along`, as
/// `(from_nothing + n·per_step)·2^unit`: `unit` the finer of `mine`'s lowest
/// bit and `2^step·along`'s, so both parts are whole numbers of it. `None` when
/// they do not fit in 128 bits.
fn a_slid_component_in_whole_numbers(
    mine: f64,
    along: f64,
    step: i32,
) -> Option<(i128, i128, i32)> {
    let (mine, mine_lowest) = a_product_in_whole_numbers(mine, 1.0);
    let (along, along_lowest) = a_product_in_whole_numbers(along, 1.0);
    let unit = if mine == 0 {
        step + along_lowest
    } else {
        mine_lowest.min(step + along_lowest)
    };
    Some((
        shifted_up(mine, mine_lowest - unit)?,
        shifted_up(along, step + along_lowest - unit)?,
        unit,
    ))
}

/// The step counts `n`, from fewest to most, for which
/// `low ≤ at_nothing + n·per_step ≤ high`, or `None` when there are none.
fn the_counts_that_land_between(
    at_nothing: i128,
    per_step: i128,
    low: i128,
    high: i128,
) -> Option<(i128, i128)> {
    let (low, high) = (low.checked_sub(at_nothing)?, high.checked_sub(at_nothing)?);
    let (from, to) = if per_step > 0 {
        (rounded_up(low, per_step)?, rounded_down(high, per_step)?)
    } else {
        (rounded_up(high, per_step)?, rounded_down(low, per_step)?)
    };
    (from <= to).then_some((from, to))
}

/// `whole / by` rounded towards minus infinity, or `None` when it overflows or
/// `by` is nothing.
fn rounded_down(whole: i128, by: i128) -> Option<i128> {
    let quotient = whole.checked_div(by)?;
    let off_the_grid = whole % by != 0 && (whole < 0) != (by < 0);
    Some(if off_the_grid { quotient - 1 } else { quotient })
}

/// `whole / by` rounded towards plus infinity, or `None` when it overflows or
/// `by` is nothing.
fn rounded_up(whole: i128, by: i128) -> Option<i128> {
    let quotient = whole.checked_div(by)?;
    let off_the_grid = whole % by != 0 && (whole < 0) == (by < 0);
    Some(if off_the_grid { quotient + 1 } else { quotient })
}

/// The counts from `from` to `to` that are `remainder` modulo `modulus`, the
/// one nearest nothing from above and the one nearest nothing from below.
fn the_counts_nearest_nothing(from: i128, to: i128, remainder: i128, modulus: i128) -> Vec<i128> {
    let upwards = from.max(0);
    let first_up = upwards + (remainder - upwards).rem_euclid(modulus);
    let downwards = to.min(-1);
    let first_down = downwards - (downwards - remainder).rem_euclid(modulus);
    [
        (first_up <= to).then_some(first_up),
        (first_down >= from).then_some(first_down),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// The count `n` from `0` to `2^bits − 1` with `at_nothing + n·per_step` a whole
/// multiple of `2^bits`, for an odd `per_step`: `−at_nothing` times the
/// inverse of `per_step` modulo `2^bits`.
///
/// # Panics
///
/// If `per_step` is even, or `bits` is not from 1 to 127: callers ask only
/// with an odd step and at most 73 bits.
fn the_count_that_clears_the_low_bits(at_nothing: i128, per_step: i128, bits: u32) -> i128 {
    assert!(
        per_step % 2 != 0 && (1..128).contains(&bits),
        "a count is solved for only with an odd step: {per_step}, {bits} bits"
    );
    let per_step = per_step.cast_unsigned();
    // Each round doubles the low bits the inverse is right in, from the three
    // every odd number is its own inverse in modulo 8, so six rounds pass 128.
    let inverse = (0..6).fold(per_step, |inverse, _| {
        inverse.wrapping_mul(2_u128.wrapping_sub(per_step.wrapping_mul(inverse)))
    });
    assert_eq!(
        per_step.wrapping_mul(inverse),
        1,
        "an odd number's inverse modulo 2¹²⁸"
    );
    let count = at_nothing
        .cast_unsigned()
        .wrapping_neg()
        .wrapping_mul(inverse);
    (count & ((1_u128 << bits) - 1)).cast_signed()
}

/// `whole · 2^by` for `by ≥ 0`, or `None` when `by` is negative or the answer
/// does not fit.
fn shifted_up(whole: i128, by: i32) -> Option<i128> {
    if whole == 0 {
        return Some(0);
    }
    let by = u32::try_from(by).ok()?;
    whole.checked_mul(1_i128.checked_shl(by).filter(|power| *power > 0)?)
}

/// `Σ ones[i]·others[i]` with nothing rounded, as a whole number and the power
/// of two it is in units of, or `None` when it does not fit in 128 bits.
fn a_dot_in_whole_numbers(ones: [f64; 3], others: [f64; 3]) -> Option<(i128, i32)> {
    let terms = [0, 1, 2].map(|i| a_product_in_whole_numbers(ones[i], others[i]));
    let Some(unit) = terms
        .iter()
        .filter(|(whole, _)| *whole != 0)
        .map(|(_, power)| *power)
        .min()
    else {
        return Some((0, 0));
    };
    terms
        .iter()
        .try_fold((0_i128, unit), |(sum, unit), &(whole, power)| {
            let term = shifted_up(whole, power - unit)?;
            Some((sum.checked_add(term)?, unit))
        })
}

/// Whether `facing` reads positive against `own` and more against `own` than
/// against `other`, both taken to length one: `facing·own > 0` and
/// `(facing·own)²·‖other‖² > (facing·other)²·‖own‖²`, asked exactly.
///
/// No direction with nothing in it passes, since it reads nothing against `own`.
fn it_leans_towards(facing: Vector, own: Vector, other: Vector) -> bool {
    let dot = |one: Vector, two: Vector| Exact::dot_of(one.components(), two.components());
    let (Some(on_own), Some(on_other), Some(own_squared), Some(other_squared)) = (
        dot(facing, own),
        dot(facing, other),
        dot(own, own),
        dot(other, other),
    ) else {
        return false;
    };
    if on_own.sign() != Decision::Pos {
        return false;
    }
    let towards_own = on_own
        .times(&on_own)
        .and_then(|square| square.times(&other_squared));
    let towards_other = on_other
        .times(&on_other)
        .and_then(|square| square.times(&own_squared));
    towards_own
        .zip(towards_other)
        .is_some_and(|(own_way, other_way)| own_way.minus(&other_way).sign() == Decision::Pos)
}

/// Whether `facing` lies exactly in the flat `one` and `other` span: its dot
/// with `one × other` is nothing, asked exactly.
fn it_lies_exactly_in_the_flat_of(facing: Vector, one: Vector, other: Vector) -> bool {
    Exact::cross_of(&Exact::of_each(one.components()), other.components())
        .and_then(|square| Exact::dot_of_two_exact(&Exact::of_each(facing.components()), &square))
        .is_some_and(|reach| reach.sign() == Decision::Zero)
}

/// Whether `point` is on the endless line of `pipe`'s axis, asked exactly:
/// `(point − origin) × direction` is nothing in every component.
fn it_lies_exactly_on_the_axis(point: Point, pipe: &Cylinder) -> bool {
    let off = Exact::gap_between(pipe.axis().origin().coordinates(), point.coordinates());
    Exact::cross_of(&off, pipe.axis().direction().components()).is_some_and(|sideways| {
        sideways
            .iter()
            .all(|component| component.sign() == Decision::Zero)
    })
}

/// Two closed loops of a leaning-pipe crossing, told apart by the plane through
/// the fat pipe's axis that the thin one's direction is square to.
///
/// The separating plane is `(ŵ₁ − (ŵ₁·ŵ₂)ŵ₂) · (p − o₂) = 0` — the part of the
/// thin pipe's direction that lies across the fat one — and every point of the
/// crossing sits at least `√(r₂² − (δ + r₁)²)` from it.
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
/// `a² + b² = r₂²`, and it is at least `√(r₂² − (δ + r₁)²)` along `â` — so `|b|`
/// is at most `δ + r₁` and the reading is at least `α·√(r₂² − (δ+r₁)²) − |β|·(δ + r₁)`.
/// Still a **gap** where that does not clear the offset's own rounding: the two
/// loops are there and they are tellable apart.
///
/// **That gap is now the only one here, and it is a different one from §142.8's.**
/// `r₂² − (δ + r₁)²` was worked out as it is written and nothing else, and for a
/// fat pipe barely wide enough to swallow the thin one the two squares cancel to
/// nothing — so `clear` reached zero and `margin` could not be above it whatever
/// the half-space did. Where that happens the same `clear` is read a second way,
/// as `m·(r₂ + δ + r₁)` with `m` from
/// [`how_far_the_fat_pipe_out_reaches_the_thin_one`], and across the whole of
/// `how_often_naming_a_crossing_refuses` the 17 crossings of 374 whose `clear`
/// reached zero all come back with a positive one.
///
/// **None of the 17 is named even so**, and that is this paragraph's gap rather
/// than an arithmetic one: every one of them fails on `β·(δ + r₁)` instead, by
/// tenths and not by roundings, because two loops that nearly touch need a
/// half-space that leans further than any of the three available ones does.
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

    let [thin_radius, fat_radius] = [thin, fat].map(|pipe| Interval::exactly(pipe.radius()));
    let apart = how_far_apart_two_axes_are_enclosing(thin, fat);
    let reach = apart + thin_radius;
    let squared_clear = fat_radius * fat_radius - reach * reach;
    let clear = if squared_clear.lo() > 0.0 {
        root_enclosing(squared_clear)
    } else {
        root_enclosing(
            how_far_the_fat_pipe_out_reaches_the_thin_one(thin, fat, apart) * (fat_radius + reach),
        )
    };
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

/// `r_fat − r_thin − δ`, as an enclosure, for a pair the caller has already had
/// `ask` certify is above zero.
///
/// # Why the subtraction is not enough
///
/// The caller only asks this where `r_fat² − (δ + r_thin)²` came out reaching
/// zero, and writing the difference out as it stands does not help there: those
/// are the same three numbers the filter in [`where_two_leaning_pipes_cross`] put
/// against each other, and the filter is what straddled zero to send the question
/// to the exact level in the first place.
///
/// A fat pipe one ulp wider than the axes are apart, against a thin one narrower
/// than that ulp, is the measured case — `docs/decisions.md` §142.8. The true
/// answer there is `1.92e-16` and `δ`'s own irreducible width at magnitude one is
/// `2.22e-16`, **larger than the answer**, so no association of this subtraction
/// in `f64` intervals reaches a positive low end.
///
/// # Rationalising, which does reach it
///
/// Write `S = r_fat − r_thin` and `δ = |T|/√K`. Multiplying above and below by
/// `S + δ` clears the root out of the numerator:
///
/// ```text
/// S − δ  =  (S² − δ²) / (S + δ)  =  (S²K − T²) / (K·(S + δ))
/// ```
///
/// `S²K − T²` is [`how_far_two_radii_out_reach_a_gap_exactly`]'s expansion, which
/// the exact level was already building to settle this very sign, and it has no
/// cancellation left in it. The denominator is `K > 0` times a sum of two numbers
/// neither of which is below zero, so it cannot cancel either. Nothing new is
/// computed: the expansion is the one the sign question builds, and `δ` is the
/// one the caller already has.
///
/// **The expansion exists wherever the caller needs it**, and that is what makes
/// the pair of routes complete rather than lucky: `ask` reaches the exact level
/// only when the filter straddles zero, and the filter straddling zero is the
/// same condition that sends the caller here.
///
/// Where `Exact` cannot form the expansion at all — a product landing among the
/// subnormals, or an axis gap that leaves `f64`'s range — this falls back to the
/// plain subtraction, which is no better than what the caller already had. A pair
/// that refused then refuses still, rather than being answered out of thin air.
fn how_far_the_fat_pipe_out_reaches_the_thin_one(
    thin: &Cylinder,
    fat: &Cylinder,
    apart: Interval,
) -> Interval {
    let widths = Interval::exactly(fat.radius()) - Interval::exactly(thin.radius());
    let subtracted = widths - apart;
    let Some((volume, squared_sideways)) = exactly_how_far_apart_two_axes_are(thin, fat) else {
        return subtracted;
    };
    let Some(out_reach) =
        how_far_two_radii_out_reach_a_gap_exactly(thin, fat, -1.0, &squared_sideways, &volume)
    else {
        return subtracted;
    };
    quotient_or_everything(
        out_reach.enclosure(),
        squared_sideways.enclosure() * (widths + apart),
    )
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
///
/// # Where the normal is the separating direction itself
///
/// A normal square to `w` that lies in the flat of `t` and `w`, asked exactly,
/// is `n = a·t + b·w` with `a(t·w) + b‖w‖² = 0`, so it is `a/‖w‖²` times the
/// separating direction, and `n·t = (a/‖w‖²)·‖t × w‖²` has the sign of `a`.
/// So where `n·t` is above zero, asked exactly, `α` is one and the second
/// number nought, and that is what comes back. Read through roundings
/// instead, `α` lands a few `2⁻⁵³` short of one, and the root charges
/// `√(1 − α²)`, about `4e−8`, against loops that may be closer than that to
/// the surface: the bar along `x` at `y = 1` inside the bar of radius 2 up `z`,
/// turned 1 about `z`, was refused for it (Q2710). `‖t × w‖²` is above zero
/// because the caller only asks of pipes whose axes lean.
fn how_a_picking_half_space_lies_across_an_axis(
    picker: &HalfSpace,
    thin: &Cylinder,
    fat: &Cylinder,
) -> (Interval, Interval) {
    let normal = picker.normal().components();
    let thin_way = thin.axis().direction().components();
    let fat_way = fat.axis().direction().components();

    if it_lies_exactly_in_the_flat_of(
        picker.normal(),
        thin.axis().direction(),
        fat.axis().direction(),
    ) && Exact::dot_of(normal, thin_way).is_some_and(|reach| reach.sign() == Decision::Pos)
    {
        return (Interval::exactly(1.0), Interval::exactly(0.0));
    }

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

/// Two closed loops of a leaning-pipe crossing that meet at one point, where a
/// thin pipe rests on the inside of a fat one, told apart by a flat surface
/// through the fat pipe's axis that passes exactly through that point.
///
/// # Where the loops meet
///
/// This is [`two_loops_either_side_of_a_pipe`] with `r_fat = r_thin + δ`. Every
/// point of the crossing is at least `√(r_fat² − (δ + r_thin)²)` from the flat
/// surface through the fat axis square to the thin pipe's way across it, and
/// that is nought now. The surface cuts the fat pipe in two lines along its
/// axis, `r_fat` either side, and the thin pipe reaches the one on its own side
/// at one point and the other not at all: the foot of the common perpendicular,
/// `o + s·ŵ + r_fat·n̂` with `n̂` along `t × w`, where the walls touch. So each
/// loop is on its own side of the surface but for that point, which both pass
/// through. The fat pipe of radius 2 up `z` and the thin one of radius 1 along
/// `x` at `y = 1` touch at `(0, 2, 0)`: the thin pipe's section at `θ` round
/// its axis reaches the fat wall at `x² = (1 − cos θ)(3 + cos θ)`, which near
/// `θ = 0` is `x = ±√2·θ`, so the loops cross there like an X, each turning
/// from one stroke to the other
/// (`a_thin_pipe_resting_on_the_inside_of_a_fat_one_leaves_two_loops_that_meet_where_they_touch`).
///
/// # What has to be exact
///
/// A picker is closed, so each holds its own loop and the touching point, but
/// only where its surface passes exactly through that point: a surface a
/// rounding off holds a stub of the other loop, or misses its own loop's
/// corner, as `docs/decisions.md` §443 found for the four half ellipses
/// [`four_half_ellipses_where_two_pipes_of_one_width_meet`] names. So the
/// facing is asked exactly to be square to `w` and to lie in the flat of `t`
/// and `w`, which makes it square to `n̂` too, and the surface passes exactly
/// through `o`. Then it holds the whole line `o + s·ŵ + r_fat·n̂`, and the
/// touching point with it. A facing that passes both checks is the thin pipe's
/// way across the fat one, so unlike in [`two_loops_either_side_of_a_pipe`]
/// there is no lean to charge, and no margin to ask: the loops reach the
/// surface by construction.
///
/// The facings tried are `t` itself, which passes wherever the two axes are
/// square; the way across worked out in `f64`, `t·‖w‖² − w·(t·w)`; and `w`
/// crossed with `x`, `y` and `z`, which are square to `w` whatever it is
/// ([`normals_exactly_square_to`]). Each is turned to read positive along the
/// way across, so the first loop is the one the thin pipe heads into. The
/// surface is a [`HalfSpace`] where one of them passes exactly through `o`,
/// and otherwise a [`HalfSpaceThroughAPoint`] through `o` facing the first.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when no facing tried is
/// exactly square to `w` and in the flat of `t` and `w`. A **gap** (§16.10):
/// the two loops are there and the true surface tells them apart, and what runs
/// out is a facing written in `f64` along it. Whatever
/// [`HalfSpace::through_a_point_facing`] and [`HalfSpaceThroughAPoint::new`]
/// refuse.
fn two_loops_that_meet_where_a_thin_pipe_rests_inside_a_fat_one(
    thin: &Cylinder,
    fat: &Cylinder,
) -> Result<SurfaceMeet> {
    let (thin_way, fat_way) = (thin.axis().direction(), fat.axis().direction());
    let across = thin_way * fat_way.length_squared() - fat_way * thin_way.dot(fat_way);
    let [first, second, third] = normals_exactly_square_to(fat_way);
    let facings: Vec<Vector> = [thin_way, across, first, second, third]
        .into_iter()
        .filter(|facing| {
            *facing != Vector::zero()
                && Exact::dot_of(facing.components(), fat_way.components())
                    .is_some_and(|reach| reach.sign() == Decision::Zero)
                && it_lies_exactly_in_the_flat_of(*facing, thin_way, fat_way)
        })
        .map(|facing| {
            if facing.dot(across) < 0.0 {
                -facing
            } else {
                facing
            }
        })
        .collect();
    let origin = fat.axis().origin();
    let picker = 'found: {
        for &facing in &facings {
            let surface = HalfSpace::through_a_point_facing(origin, facing)?;
            if it_passes_exactly_through(&surface, origin) {
                break 'found Picker::from(surface);
            }
        }
        let Some(&facing) = facings.first() else {
            return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
        };
        Picker::from(HalfSpaceThroughAPoint::new(origin, facing)?)
    };
    let other_side = picker
        .reversed()
        .expect("the picker is flat, so it has another side");

    let and = Surface::Cylinder(*fat);
    Ok(SurfaceMeet::Curves(
        [picker, other_side]
            .into_iter()
            .map(|picker| {
                NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                    and,
                    picked_by: smallvec![picker],
                })
                .expect("picked_by always holds the one flat picker just built")
            })
            .collect(),
    ))
}

/// The one point two leaning pipes resting against each other share, as the box
/// it is known to be inside.
///
/// It is on the common perpendicular, `r₁` along it from the first axis.
fn where_two_leaning_pipes_rest_against_each_other(
    one: &Cylinder,
    other: &Cylinder,
) -> PredicatePoint {
    let here = one.axis().origin().coordinates();
    let there = other.axis().origin().coordinates();
    let [my_way, their_way] = [one, other].map(|pipe| pipe.axis().direction().components());
    let mine = Interval::exactly(one.radius());

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
        Some((volume, squared_sideways)) => (volume.enclosure(), squared_sideways.enclosure()),
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

/// `T = (o₂ − o₁)·(w₁ × w₂)` and `K = ‖w₁ × w₂‖²`, summed exactly, or `None`
/// where a product would round.
///
/// The two together are everything the gap `δ = |T|/√K` is made of, so a caller
/// that wants the gap takes [`Exact::enclosure`] of each and divides, and one
/// that wants a polynomial in them — [`how_two_radii_and_a_gap_compare_exactly`]
/// and [`how_far_the_fat_pipe_out_reaches_the_thin_one`] both do — keeps them
/// exact and builds on them.
fn exactly_how_far_apart_two_axes_are(one: &Cylinder, other: &Cylinder) -> Option<(Exact, Exact)> {
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
    Some((volume, Exact::squared_length_of(&sideways)?))
}

/// A number with the same sign as `mine·r₁ + theirs·r₂ + gap·δ`, exactly.
///
/// **A sign and not the value**, because the value is not a polynomial in the
/// stored numbers: `δ = |T|/√K` carries a square root. It is the only one left.
/// The two widths are the stored `rᵢ` themselves — §118.11 — where they used to
/// be `rᵢ/√Wᵢ`, and that is what turns this from the worst comparison in the
/// file into one of the cheapest.
///
/// # Clearing the one root
///
/// Write `S = mine·r₁ + theirs·r₂`, a polynomial in two stored numbers with no
/// root in it at all. `K > 0`, so multiplying through by `√K` keeps the sign:
///
/// ```text
/// S·√K + gap·|T|
/// ```
///
/// Both halves have a sign that is read off rather than computed. `sign(S√K)` is
/// `sign(S)`, and since both radii are above zero that is `sign(mine)` when the
/// two agree and which radius is the larger when they do not — one `f64`
/// comparison either way. `sign(gap·|T|)` is `sign(gap)` unless `T` is nothing.
///
/// Where the two halves pull the same way, or either is nothing, that settles
/// it. Where they pull against each other, `sign(S√K + gap|T|)` is
/// `sign(S)·sign(S²K − T²)`, and with `mine` and `theirs` each `±1`
///
/// ```text
/// S²  =  r₁² + r₂² + 2·mine·theirs·r₁r₂
/// ```
///
/// so `S²K − T²` is **degree six against degree six, and has no `Wᵢ` in it**.
///
/// # What that replaced
///
/// The old widths `rᵢ/√Wᵢ` put three roots in three different metrics against
/// each other. Clearing them cost eight degrees and left `sign(M² − N²)` with
/// `M = r₁²W₂K + r₂²W₁K − T²W₁W₂` at degree twenty against `N² = 4r₁²r₂²K²W₁W₂`
/// at sixteen — and `M` for two ordinary pipes is five hundred binary orders
/// wide, so squaring it fell off the bottom of `f64` and this level ran only
/// where `W₁ = W₂` made `√(W₁W₂)` rational. `docs/decisions.md` §29 is that
/// measurement, and
/// `the_question_two_leaning_pipes_ask_is_too_wide_to_square_where_it_stands`
/// still demonstrates it about `Exact`. **Nothing here builds that expansion any
/// more**, so what §29 recorded as unsquarable is no longer asked.
fn how_two_radii_and_a_gap_compare_exactly(
    one: &Cylinder,
    other: &Cylinder,
    mine: f64,
    theirs: f64,
    gap: f64,
) -> Option<Exact> {
    debug_assert!(
        [mine, theirs, gap].iter().all(|side| side.abs() == 1.0),
        "the three are signs, and squaring them away above assumed each is one"
    );
    let (volume, squared_sideways) = exactly_how_far_apart_two_axes_are(one, other)?;

    // The sign of `mine·r₁ + theirs·r₂`. Both radii are above zero, so where the
    // two pull against each other it is which of them is wider — a comparison of
    // two stored numbers, where it used to be the sign of `r₂²W₁ − r₁²W₂`.
    let radii = if mine * theirs > 0.0 {
        if mine > 0.0 {
            Decision::Pos
        } else {
            Decision::Neg
        }
    } else {
        match (other.radius().partial_cmp(&one.radius()), theirs > 0.0) {
            (Some(Ordering::Equal), _) => Decision::Zero,
            (Some(Ordering::Greater), true) | (Some(Ordering::Less), false) => Decision::Pos,
            (Some(Ordering::Greater), false) | (Some(Ordering::Less), true) => Decision::Neg,
            (None, _) => return None,
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
        (radii_sign, _) => {
            let out_reaches = how_far_two_radii_out_reach_a_gap_exactly(
                one,
                other,
                mine * theirs,
                &squared_sideways,
                &volume,
            )?
            .sign();
            match (out_reaches, radii_sign == Decision::Pos) {
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

/// `S²K − T²`: by how much the two widths together out-reach the gap between the
/// axes, with the one square root cleared out of it.
///
/// `together` is `mine·theirs`, the one thing the caller's two signs leave in
/// `S² = r₁² + r₂² + 2·together·r₁r₂`. Degree six on each side, and every factor
/// is a product of stored numbers, so this decides unless one of them leaves
/// `f64`'s range.
///
/// **The value and not only its sign**, because two callers want two different
/// things out of it. [`how_two_radii_and_a_gap_compare_exactly`] takes the sign,
/// which is all a `Decision` can carry; [`how_far_the_fat_pipe_out_reaches_the_thin_one`]
/// divides it by `K·(S + δ)` and gets `S − δ` back, which is a size no sign
/// could have handed over. `docs/decisions.md` §142.9.
fn how_far_two_radii_out_reach_a_gap_exactly(
    one: &Cylinder,
    other: &Cylinder,
    together: f64,
    squared_sideways: &Exact,
    volume: &Exact,
) -> Option<Exact> {
    let widths = Exact::product_of(one.radius(), one.radius())?
        .plus(&Exact::product_of(other.radius(), other.radius())?)
        .plus(&Exact::product_of(
            2.0 * together * one.radius(),
            other.radius(),
        )?);
    Some(
        widths
            .times(squared_sideways)?
            .minus(&volume.times(volume)?),
    )
}

/// `E² − 4r₁²r₂²W₁²` as an enclosure.
///
/// Below zero the two cross in two lines, and then this same reading says how far
/// apart those lines are — see [`two_lines_between_two_pipes`].
///
/// # Why `E` is taken from the exact arm when the exact arm can reach it
///
/// `E = A − W₁(r₁² + r₂²)` is a subtraction of two numbers that agree to as many
/// digits as the pipes differ in width. A hair-thin pipe lying on a pipe 161
/// decades fatter has both sides at `2.5e307` and a true `E` of about `1.7e15`,
/// so an interval `E` comes back as `±5e291` — and **squaring that overflows to
/// everything**, which is a refusal on a pair whose answer exists. The exact
/// expansion loses nothing there: `A`'s `r₁²` term cancels against the one it is
/// being subtracted from, and what is left encloses to a fraction of an ulp.
///
/// The exact arm alone does not cover it, which is why this reads `E` rather
/// than deferring to it: `room.times(&room)` in
/// [`how_deeply_two_side_by_side_pipes_overlap_exactly`] needs every pairwise
/// product to stay above `f64::MIN_POSITIVE · 2⁵³`, and the smallest entry of
/// that expansion is `r₂²`, so the exact arm gives up below
/// `r₂ ≈ 1.19e-73` while the geometry stays perfectly ordinary for another 66
/// decades. `docs/decisions.md` §143.
///
/// Where the exact arm cannot form `E` at all — an `A` that leaves `f64`'s range
/// — there is nothing to take an enclosure from and this falls back to the
/// interval `E` it used to compute, so a pair that refuses today still refuses
/// rather than being answered out of thin air.
fn how_deeply_two_side_by_side_pipes_overlap_enclosing(
    one: &Cylinder,
    other: &Cylinder,
) -> Interval {
    let [my_way, _] = [one, other].map(|pipe| pipe.axis().direction().components());
    let room = how_much_room_two_side_by_side_pipes_leave_exactly(one, other)
        .as_ref()
        .map_or_else(
            || how_much_room_two_side_by_side_pipes_leave_enclosing(one, other),
            Exact::enclosure,
        );
    let squared_mine = squared_length_enclosing(exactly(my_way));
    room * room
        - Interval::exactly(4.0)
            * squared_enclosing(one.radius())
            * squared_enclosing(other.radius())
            * squared_mine
            * squared_mine
}

/// `E² − 4r₁²r₂²W₁²`, exactly.
fn how_deeply_two_side_by_side_pipes_overlap_exactly(
    one: &Cylinder,
    other: &Cylinder,
) -> Option<Exact> {
    let [my_way, _] = [one, other].map(|pipe| pipe.axis().direction().components());
    let room = how_much_room_two_side_by_side_pipes_leave_exactly(one, other)?;
    let squared_mine = Exact::dot_of(my_way, my_way)?;
    Some(
        room.times(&room)?.minus(
            &Exact::product_of(2.0, 2.0)?
                .times(&Exact::product_of(one.radius(), one.radius())?)?
                .times(&Exact::product_of(other.radius(), other.radius())?)?
                .times(&squared_mine)?
                .times(&squared_mine)?,
        ),
    )
}

/// `E = A − W₁(r₁² + r₂²)` as an enclosure.
fn how_much_room_two_side_by_side_pipes_leave_enclosing(
    one: &Cylinder,
    other: &Cylinder,
) -> Interval {
    let [here, there] = [one, other].map(|pipe| pipe.axis().origin().coordinates());
    let [my_way, _] = [one, other].map(|pipe| pipe.axis().direction().components());
    let squared_mine = squared_length_enclosing(exactly(my_way));
    how_far_off_an_axis_enclosing(there, here, my_way)
        - squared_mine * (squared_enclosing(one.radius()) + squared_enclosing(other.radius()))
}

/// `E = A − W₁(r₁² + r₂²)`, exactly.
fn how_much_room_two_side_by_side_pipes_leave_exactly(
    one: &Cylinder,
    other: &Cylinder,
) -> Option<Exact> {
    let [here, there] = [one, other].map(|pipe| pipe.axis().origin().coordinates());
    let [my_way, _] = [one, other].map(|pipe| pipe.axis().direction().components());
    let squared_mine = Exact::dot_of(my_way, my_way)?;
    Some(
        how_far_off_an_axis_exactly(there, here, my_way)?.minus(
            &squared_mine.times(
                &Exact::product_of(one.radius(), one.radius())?
                    .plus(&Exact::product_of(other.radius(), other.radius())?),
            )?,
        ),
    )
}

/// How far a picking half-space's own flat surface sits off the point it was
/// built through, as an upper bound on a real distance.
///
/// A half-space's normal is whatever length its caller's was, so the reading is
/// divided by that length to come back as a distance.
///
/// The reading is worked out exactly where the products allow. An interval dot
/// brackets the true one between two `f64`, and when the offset is the nearer
/// of them — which is how [`HalfSpace::through_a_point_facing`] rounds it — the
/// interval's far end is at least **twice** the true rounding away, so a caller
/// weighing this against a margin was refused for twice the distance the
/// surface is really off (`docs/decisions.md` §314).
fn how_far_a_picking_half_space_sits_off_a_point(picker: &HalfSpace, through: Point) -> Interval {
    let normal = picker.normal().components();
    let off = Exact::dot_of(normal, through.coordinates()).map_or_else(
        || {
            dot_enclosing(exactly(normal), exactly(through.coordinates()))
                - Interval::exactly(picker.offset())
        },
        |dot| dot.minus(&Exact::of(picker.offset())).enclosure(),
    );
    quotient_or_everything(
        Interval::exactly(off.lo().abs().max(off.hi().abs())),
        root_enclosing(squared_length_enclosing(exactly(normal))),
    )
}

/// Two endless straight lines, as one reading against a picking half-space and
/// two steps either side of it.
///
/// Never added up into two points. [`two_lines_told_apart`]'s second doc section
/// says what adding them up costs.
struct TwoLinesEitherSideOfThePicker {
    /// How far in front of the picker the point midway between the two lines
    /// sits: `normal · middle − offset`, or `facing · (middle − through)` for
    /// a picker through a point.
    ///
    /// **Each caller works this out its own way, because what is exact differs
    /// between them, and reading it off a direction is what §182 got wrong.**
    /// A plane cutting a pipe steps along the plane's own *stored* normal, so
    /// the picking normal's dot with it is an exact sum and the step is real:
    /// that caller reads the picker at the axis and adds the step.
    /// Two side-by-side pipes step along a direction that was *computed* —
    /// `the_step_across_to_the_other_axis` divides by `w·w` at the end — and the
    /// picking normal is built exactly square to that rounded direction. Its dot
    /// with it is then exactly zero **by construction**, which says nothing at
    /// all about where the true axis-to-axis step points, and for a pair far
    /// from the origin the two disagree by far more than the lines are apart.
    /// That caller reads the picker at both stored axis origins and puts the
    /// whole of this together as one polynomial over one division (§184).
    middle: Interval,
    /// The direction the two lines step either way along, scaled to length one.
    up: [Interval; 3],
    /// How far along `up` each of the two steps.
    out: Interval,
}

/// Two endless straight lines running along `along`, told apart by `picker`,
/// which is flat: a [`HalfSpace`] or a [`HalfSpaceThroughAPoint`]. The first
/// line is picked by it and the second by it turned round.
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
///
/// # Why the lines arrive in pieces rather than as two points
///
/// §93 fixed how far apart the two lines are and left how they were handed over
/// alone, and §140 is the second half of the same bug. A whole coordinate holds
/// `from + sideways·how_far ± up·out` in three `f64`s, so the short step and the
/// long one share a rounding: at `how_far/out = 9.34e160` — a hair-thin pipe on
/// the skin of one 161 decades fatter — the short step is gone before the dot
/// product starts, and the reading the two lines differ by is `±out·δ` against
/// terms of `2.1e161` that have to cancel first.
///
/// Taken in pieces nothing has to cancel. `middle` is what the picker reads at
/// the point midway between the two lines, built by the caller out of dot
/// products of stored `f64` triples, so [`Exact`] knows it to its own last bit
/// rather than to a bit of `2.1e161`. What is left is `±out · (normal · up)`,
/// and its sign is the sign of `out`, which the caller has already certified.
///
/// **§182 is why `middle` is a reading and not a direction and a length.**
/// It used to be `normal · sideways` multiplied by how far along `sideways` the
/// middle sat, which is sound only while `sideways` is a direction the geometry
/// really steps along. For two side-by-side pipes it is not: the picking normal
/// is built exactly square to a **rounded** step, so that product is zero by
/// construction and says nothing about where the true step points — and a pair
/// far enough from the origin was answered with a plane holding both lines and a
/// reverse holding neither. Each caller now works the reading out its own way.
///
/// **§184 is why it is the whole reading and not a point plus a step.** Written
/// as `at_here + share·(at_there − at_here)`, with `share` an enclosure, the
/// step's own relative width of `2⁻⁵³` lands against `at_here` rather than
/// against the answer: for §181.2's pair that is `3.7e5` of window around a true
/// middle of `9.98e-244`. Put together as one polynomial over one division it is
/// the answer's own width instead. Only the caller can write that polynomial,
/// so only the caller can hand the whole reading over.
///
/// **The reading is still whatever it is.** The middle of the two lines may
/// genuinely sit off the picking plane, and the refusal below is then the right
/// answer. This stops throwing the reading away; it does not decide it.
fn two_lines_told_apart(
    picker: Picker,
    along: [f64; 3],
    lines: &TwoLinesEitherSideOfThePicker,
    and: Surface,
) -> Result<SurfaceMeet> {
    let (Some(facing), Some(reversed)) = (picker.facing_of_a_flat(), picker.reversed()) else {
        panic!("two lines are told apart by a flat surface, not by {picker:?}");
    };
    let normal = facing.components();
    if !matches!(
        Exact::dot_of(normal, along).as_ref().map(Exact::sign),
        Some(Decision::Zero)
    ) {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    let either_way = lines.out * dot_enclosing(exactly(normal), lines.up);

    let side = [1.0, -1.0].map(|which| lines.middle + either_way * Interval::exactly(which));
    if !(side[0].lo() > 0.0 && side[1].hi() < 0.0) {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    Ok(SurfaceMeet::Curves(
        [picker, reversed]
            .into_iter()
            .map(|picker| {
                NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                    and,
                    picked_by: smallvec![picker],
                })
                .expect("picked_by always holds the one picker just built")
            })
            .collect(),
    ))
}

/// How far in front of `picker` a stored point sits: `normal · p − offset`.
///
/// Both halves are stored `f64`s, so [`Exact`] sums the dot product and takes
/// the offset off it with nothing cancelling — which matters because the two
/// terms of that sum routinely stand a hundred decades apart and the answer is
/// the small change left over.
fn how_far_in_front(picker: HalfSpace, point: [f64; 3]) -> Interval {
    let normal = picker.normal().components();
    Exact::dot_of(normal, point)
        .map(|reading| reading.minus(&Exact::of(picker.offset())))
        .as_ref()
        .map_or_else(
            || dot_enclosing(exactly(normal), exactly(point)) - Interval::exactly(picker.offset()),
            Exact::enclosure,
        )
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
/// [`two_lines_through_a_cones_apex`] says which is which, or which of their four
/// halves is which where no `f64` direction runs along them. Neither asks the
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
/// answer. [`how_a_plane_leans_against_a_pipe`] makes the same trade for a pipe,
/// for the same reason.
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
/// # Where a ruling's direction is irrational, nothing names it whole
///
/// Item 1 is not a shortage a better order or scale of the arithmetic fills.
/// Where `−E·‖w‖²` is not the square of a fraction, the ruling runs along no
/// direction three `f64`s write, at any scale. Then no flat surface but the
/// plane itself holds it: a stored normal square to `‖w‖²·p ± √(−E·‖w‖²)·(n × w)`
/// is square to `p` and to `n × w` separately, so it is along `n`, and the plane
/// holds both rulings. No pipe or ball picker does better: one that holds a
/// whole endless line and only the apex of a second needs an axis along the
/// first. So such a pair cannot be named as two whole lines by any [`Picker`]
/// there is. The turned frustum of §294.10 is one
/// (`a_turned_cone_cut_through_its_axis.rs`, §294.11).
///
/// **Halves of the rulings can be named, and that is what this hands back
/// there.** Where either direction worked out in `f64` is not on both surfaces,
/// the pair comes back as the four halves the apex cuts the two rulings into
/// ([`four_half_rulings_through_a_cones_apex`]), each picked by a corner of two
/// flat surfaces through the apex rather than by a pair of opposite half-spaces.
/// So a picker of a ruling through an apex is one of two shapes, and a reader of
/// one cannot take `[H, H.reversed()]` for granted (§294.11).
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when item 3 cannot be had,
/// and whatever [`HalfSpace::through_a_point_square_to`] refuses; and where the
/// pair is named in halves, whatever
/// [`four_half_rulings_through_a_cones_apex`] refuses.
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
    if !rulings
        .iter()
        .all(|&ruling| a_line_through_the_apex_along_it_lies_on_both(ruling, flat, cone))
    {
        return four_half_rulings_through_a_cones_apex(flat, cone);
    }

    let mut runs = Vec::new();
    for [this, other] in [[rulings[0], rulings[1]], [rulings[1], rulings[0]]] {
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

/// The four halves the apex cuts the two rulings into, for a plane through a
/// cone's apex steeper than the cone, each picked out by one corner of two flat
/// surfaces through the apex: the one facing `n × w` and the one facing the
/// axis `w`.
///
/// # Why a corner holds one half and nothing else
///
/// The two rulings are mirror images across the flat surface through the apex
/// holding the axis and `n`, which is the one facing `n × w`, and each crosses
/// it at the apex: its forward half is strictly on one side and its backward
/// half strictly on the other. The flat surface square to the axis parts the
/// two nappes the same way. So each of the four halves is strictly inside one
/// corner of the two, away from the apex, and where the four corners are four
/// different ones, a corner holds one half and the apex. A picker is closed, so
/// the apex is on all four, which is where they meet. Neither surface has to
/// hold a ruling, so neither needs a direction that is exact, and that is why
/// this works where the whole rulings cannot be named (§294.11).
///
/// # What is proved, and how
///
/// 1. **Both flat surfaces pass exactly through the apex**, each offset asked
///    against the exact dot product, item 3 of
///    [`two_lines_through_a_cones_apex`] over again. A surface a rounding off
///    the apex would hand a corner a stub of a second half.
/// 2. **Each half is strictly on one side of each surface.** Along a half
///    `apex + t·d`, `t > 0`, a surface through the apex reads `t` times its
///    normal against `d`, so that one sign is the whole half's. `d` is
///    `±(‖w‖²·p ± √(−E·‖w‖²)·(n × w))` for `p = ‖n‖²·w − (n·w)·n`, worked out
///    as an enclosure of the true direction from the stored numbers
///    ([`the_four_ways_two_rulings_leave_a_cones_apex`]), and its sign against
///    each stored normal is asked over the whole enclosure.
/// 3. **The four halves are in four different corners.**
///
/// That the crossing *is* those four halves and nothing else is the caller's
/// two exact signs: the apex on the plane, and `E` below nothing.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] when any of the three
/// cannot be shown, and whatever [`HalfSpace::through_a_point_facing`] refuses.
/// A **gap** where it refuses (§16.10): the four halves are there and are
/// four corners apart.
fn four_half_rulings_through_a_cones_apex(flat: &Plane, cone: &Cone) -> Result<SurfaceMeet> {
    let apex = cone.apex();
    let along = cone.axis().direction();
    let surfaces = [
        HalfSpace::through_a_point_facing(apex, flat.normal().cross(along))?,
        HalfSpace::through_a_point_facing(apex, along)?,
    ];
    if !surfaces
        .iter()
        .all(|surface| it_passes_exactly_through(surface, apex))
    {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }
    let Some(halves) =
        the_four_ways_two_rulings_leave_a_cones_apex(exactly(flat.normal().components()), cone)
    else {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    };

    let mut corners: Vec<[Decision; 2]> = Vec::new();
    let mut runs = Vec::new();
    for half in halves {
        let corner = surfaces
            .map(|surface| dot_enclosing(exactly(surface.normal().components()), half).sign());
        let ([Decision::Pos | Decision::Neg, Decision::Pos | Decision::Neg], false) =
            (corner, corners.contains(&corner))
        else {
            return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
        };
        corners.push(corner);
        let [mirror, nappe] = [0, 1].map(|which| {
            if corner[which] == Decision::Pos {
                surfaces[which]
            } else {
                surfaces[which].reversed()
            }
        });
        runs.push(NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
            and: Surface::Cone(*cone),
            picked_by: smallvec![Picker::from(mirror), Picker::from(nappe)],
        })?);
    }
    Ok(SurfaceMeet::Curves(runs))
}

/// The four ways the two rulings of a flat surface through a cone's apex leave
/// it, each as an enclosure of the true direction:
/// `±(‖w‖²·p + √(−E·‖w‖²)·(n × w))` and `±(‖w‖²·p − √(−E·‖w‖²)·(n × w))`, for
/// `p = ‖n‖²·w − (n·w)·n`. `None` when the enclosure of `−E·‖w‖²` is wholly
/// below nothing, which only a caller that has not proved `E` below nothing can
/// hand in.
///
/// `normal` is an enclosure of the surface's normal `n`: a plane's stored one,
/// or one [`the_rulings_two_cones_on_one_apex_share`] works out, and any length
/// of it will do, since `±p` and `±√(−E·‖w‖²)·(n × w)` both scale with `‖n‖²`.
///
/// **The root is rounded outwards** by [`root_enclosing`], one `f64` step at
/// each end where `f64::sqrt` landed inside, which is an enclosure because that
/// square root is correctly rounded. Everything else is interval `+ − ×` on the
/// stored numbers.
fn the_four_ways_two_rulings_leave_a_cones_apex(
    normal: [Interval; 3],
    cone: &Cone,
) -> Option<[[Interval; 3]; 4]> {
    let under_the_root = how_far_apart_two_rulings_through_a_cones_apex_spread(normal, cone);
    if under_the_root.hi() < 0.0 {
        return None;
    }
    let along = exactly(cone.axis().direction().components());
    let squared_axis = squared_length_enclosing(along);
    let up = dot_enclosing(normal, along);
    let across = cross_enclosing(normal, along);
    let spread = root_enclosing(under_the_root);
    let squared_normal = squared_length_enclosing(normal);
    let down: [Interval; 3] = core::array::from_fn(|axis| {
        squared_axis * (squared_normal * along[axis] - up * normal[axis])
    });
    let [ahead, other] =
        [spread, -spread].map(|side| core::array::from_fn(|axis| down[axis] + across[axis] * side));
    let back = |way: [Interval; 3]| way.map(|component| -component);
    Some([ahead, other, back(ahead), back(other)])
}

/// `−E·‖w‖²`, for `E = (n·w)² − s²‖n × w‖²`, as an enclosure, where `normal`
/// encloses `n`: above nothing where the flat surface through the cone's apex
/// square to `n` cuts it in two rulings, nothing where it touches it along one,
/// and below nothing where it meets it at the apex alone.
fn how_far_apart_two_rulings_through_a_cones_apex_spread(
    normal: [Interval; 3],
    cone: &Cone,
) -> Interval {
    let along = exactly(cone.axis().direction().components());
    let up = dot_enclosing(normal, along);
    let across = cross_enclosing(normal, along);
    let leaning = up * up - squared_enclosing(cone.slope()) * squared_length_enclosing(across);
    -(leaning * squared_length_enclosing(along))
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
    matches!(square_to_this, Some(Decision::Zero))
        && matches!(across_the_other, Some(Decision::Pos | Decision::Neg))
        && it_passes_exactly_through(picker, apex)
}

/// Whether `surface`'s flat surface passes exactly through `point`: its normal
/// against the point, worked out exactly, is its offset.
fn it_passes_exactly_through(surface: &HalfSpace, point: Point) -> bool {
    matches!(
        Exact::dot_of(surface.normal().components(), point.coordinates())
            .map(|reach| reach.minus(&Exact::of(surface.offset())).sign()),
        Some(Decision::Zero)
    )
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
        Decision::Pos => return a_cone_and_a_pipe_off_one_axis(cone, pipe, &at, policy, log),
        other => unreachable!("a sum of squares is never below zero: {other:?}"),
    }

    let apex_on_the_axis = a_cones_apex_sits_on_a_pipes_axis(cone, pipe);
    match ask(&apex_on_the_axis, Measures::Area, &at, policy, log)? {
        Decision::Zero => two_circles_round_a_shared_axis(cone, pipe),
        Decision::Pos => a_cone_and_a_pipe_off_one_axis(cone, pipe, &at, policy, log),
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

/// A cone and a pipe whose axes are not one line, which cross in a degree-four
/// curve: two loops named where they are proved two and held apart, and refused
/// where they are not.
///
/// # What the crossing is, read along the pipe's straight lines
///
/// With `y = x − a` measured from the apex, the cone is `Q(y) = 0` for
/// `Q(y) = W|y|² − (1 + slope²)(y·w_c)²`, `W = w_c·w_c`. A straight line of the
/// pipe is `y = z + t·w_p`, and along it `Q` is the quadratic
/// `Q(w_p)t² + 2(n·z)t + Q(z)`, with `n = W·w_p − (1 + slope²)(w_p·w_c)·w_c`.
/// Its two roots are where that line meets the cone, and its quarter
/// discriminant `D(z) = (n·z)² − Q(w_p)Q(z)` says whether they are there. At a
/// root `n·y = ±√D`, so **the flat surface through the apex square to `n`
/// holds the two roots of every line apart**, whatever the sign of `Q(w_p)`.
///
/// # A pipe steeper than the cone's skin always crosses it in two loops
///
/// `Q(w_p) < 0` puts the pipe's direction inside the cone's own. Then `Q` is a
/// form of one negative and two positive directions with `w_p` on the negative
/// side, and for such a form `(n·z)² ≥ Q(w_p)Q(z)` with equality only for `z`
/// along `w_p` — the reverse of Cauchy–Schwarz. So `D > 0` on every line of the
/// pipe but one through the apex, which is the pipe's skin running through it,
/// asked exactly and refused. Every line runs from inside one nappe to inside
/// the other, so its two roots are one on each nappe, and the flat surface
/// through the apex square to the cone's axis holds the two loops apart. It
/// reads `w_c·(x − a)` exactly, and no point of the crossing is the apex.
///
/// # A shallower pipe crosses it in two loops where `D` is proved above zero
///
/// `Q(w_p) > 0` and a line meets the cone twice, once or not at all, so the
/// crossing can be two loops, one loop, or nothing. It is named as two loops
/// only where `D` is proved above zero on every line of the pipe
/// ([`two_loops_either_side_of_a_cones_slant_plane`]), and refused otherwise.
/// `Q(w_p) = 0` is a pipe along one of the cone's own straight lines, whose
/// lines each meet the cone once or along the whole of it, and is refused.
///
/// # Errors
///
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] for the shapes
/// above that are refused: a pipe along a straight line of the cone, a steeper
/// one whose skin runs through the apex, and a shallower one not proved to
/// cross in two loops. Each is a **gap**: the crossing is there.
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] where the rounded `n` a
/// shallower pipe's loops are picked by cannot be proved to hold them apart.
///
/// [`Error::Undecidable`] where a sign could not be settled.
fn a_cone_and_a_pipe_off_one_axis(
    cone: &Cone,
    pipe: &Cylinder,
    at: &[PredicatePoint; 2],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    // Two unit directions against each other again, filed as a length.
    match ask(
        &how_steeply_a_pipe_runs_against_a_cone(cone, pipe),
        Measures::Length,
        at,
        policy,
        log,
    )? {
        Decision::Zero => Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
        Decision::Neg => match ask(
            &how_far_a_cones_apex_is_off_a_pipes_skin(cone, pipe),
            Measures::Area,
            at,
            policy,
            log,
        )? {
            Decision::Zero => Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
            Decision::Pos | Decision::Neg => two_loops_either_side_of_a_cones_apex(cone),
            other => unreachable!("a sign question answers with a sign: {other:?}"),
        },
        Decision::Pos => two_loops_either_side_of_a_cones_slant_plane(cone, pipe),
        other => unreachable!("a sign question answers with a sign: {other:?}"),
    }
}

/// `Q(w_p) = W(w_p·w_p) − (1 + slope²)(w_p·w_c)²`, as a question.
fn how_steeply_a_pipe_runs_against_a_cone(cone: &Cone, pipe: &Cylinder) -> impl Predicate {
    let cone = *cone;
    let along = pipe.axis().direction().components();
    Question {
        named: WHERE_A_CONE_AND_A_PIPE_CROSS,
        enclosing: move || a_cones_form_enclosing(&cone, exactly(along)),
        exactly: move || a_cones_form_exactly(&cone, &Exact::of_each(along)),
    }
}

/// `‖(a − o_p) × w_p‖² − r²(w_p·w_p)`, zero where the pipe's skin runs through
/// the apex, as a question.
fn how_far_a_cones_apex_is_off_a_pipes_skin(cone: &Cone, pipe: &Cylinder) -> impl Predicate {
    let apex = cone.apex().coordinates();
    let origin = pipe.axis().origin().coordinates();
    let along = pipe.axis().direction().components();
    let skin = pipe.radius();
    Question {
        named: WHERE_A_CONE_AND_A_PIPE_CROSS,
        enclosing: move || {
            how_far_off_an_axis_enclosing(apex, origin, along)
                - squared_enclosing(skin) * squared_length_enclosing(exactly(along))
        },
        exactly: move || {
            Some(
                how_far_off_an_axis_exactly(apex, origin, along)?
                    .minus(&Exact::product_of(skin, skin)?.times(&Exact::dot_of(along, along)?)?),
            )
        },
    }
}

/// `Q(y) = W|y|² − (1 + slope²)(y·w_c)²` as an enclosure, zero where `y`
/// reaches from the apex to the cone.
fn a_cones_form_enclosing(cone: &Cone, y: [Interval; 3]) -> Interval {
    let up = exactly(cone.axis().direction().components());
    let upward = dot_enclosing(y, up);
    squared_length_enclosing(up) * squared_length_enclosing(y)
        - (Interval::exactly(1.0) + squared_enclosing(cone.slope())) * upward * upward
}

/// `Q(y) = W|y|² − (1 + slope²)(y·w_c)²`, exactly.
fn a_cones_form_exactly(cone: &Cone, y: &[Exact; 3]) -> Option<Exact> {
    let up = cone.axis().direction().components();
    let upward = Exact::dot_of_two_exact(y, &Exact::of_each(up))?;
    let upward_squared = upward.times(&upward)?;
    let slope_squared = Exact::product_of(cone.slope(), cone.slope())?;
    Some(
        Exact::dot_of(up, up)?
            .times(&Exact::squared_length_of(y)?)?
            .minus(&upward_squared)
            .minus(&slope_squared.times(&upward_squared)?),
    )
}

/// `n = W·w_p − (1 + slope²)(w_p·w_c)·w_c` as an enclosure: the facing of the
/// flat surface through the apex that holds the two roots of each of the pipe's
/// straight lines apart.
pub(crate) fn a_cones_slant_against_a_pipe_enclosing(
    cone: &Cone,
    pipe: &Cylinder,
) -> [Interval; 3] {
    let up = exactly(cone.axis().direction().components());
    let along = exactly(pipe.axis().direction().components());
    let squared_up = squared_length_enclosing(up);
    let leaning =
        (Interval::exactly(1.0) + squared_enclosing(cone.slope())) * dot_enclosing(along, up);
    core::array::from_fn(|axis| squared_up * along[axis] - leaning * up[axis])
}

/// `n = W·w_p − (1 + slope²)(w_p·w_c)·w_c`, exactly.
pub(crate) fn a_cones_slant_against_a_pipe_exactly(
    cone: &Cone,
    pipe: &Cylinder,
) -> Option<[Exact; 3]> {
    let up = cone.axis().direction().components();
    let along = pipe.axis().direction().components();
    let upward = Exact::dot_of(along, up)?;
    let leaning = upward.plus(&Exact::product_of(cone.slope(), cone.slope())?.times(&upward)?);
    let squared_up = Exact::dot_of(up, up)?;
    let part = |axis: usize| {
        Some(
            squared_up
                .times(&Exact::of(along[axis]))?
                .minus(&leaning.times(&Exact::of(up[axis]))?),
        )
    };
    Some([part(0)?, part(1)?, part(2)?])
}

/// The two loops a cone crosses something in, one on each nappe, picked by the
/// flat half-space through the apex facing along the cone's axis.
///
/// No margin is asked, for the reason [`a_cone_and_a_pipe_off_one_axis`]
/// gives: the half-space reads `w_c·(x − a)` exactly, and no point of either
/// loop is the apex.
///
/// # Errors
///
/// Whatever [`HalfSpaceThroughAPoint::new`] refuses, which a cone's finite
/// apex and axis never are.
fn two_loops_either_side_of_a_cones_apex(cone: &Cone) -> Result<SurfaceMeet> {
    let picker = Picker::from(HalfSpaceThroughAPoint::new(
        cone.apex(),
        cone.axis().direction(),
    )?);
    two_loops_picked_either_side_of(picker, cone)
}

/// The two loops a shallower pipe crosses a cone in, where `D` is proved above
/// zero on every straight line of the pipe, picked by the flat half-space
/// through the apex facing along `n` rounded.
///
/// # How `D` is bounded over the whole pipe
///
/// `D(z) = zᵀMz` for `M = nnᵀ − Q(w_p)·(W·I − (1 + slope²)w_c w_cᵀ)`, and
/// `M·w_p = 0`, so `D` reads only where a line of the pipe sits across the
/// pipe's axis: `z = c + r·e` with `c = o_p − a`, `r` the radius and `e` any
/// unit direction square to `w_p`. Then `D = cᵀMc + 2r·eᵀMc + r²·eᵀMe`, which is
/// within `2r‖Mc‖ + r²‖M‖` of `cᵀMc`, with the Frobenius norm standing for the
/// largest stretch `M` gives. That bound's lower end above zero puts two roots
/// on every line, so two loops, one each side of the flat surface square to
/// `n`.
///
/// # The rounded facing is proved to hold the loops apart
///
/// At a root `n·(x − a) = ±√D`, so a facing within `δ` of `n` reads the same
/// sign there wherever `√D > δ·|x − a|`. `|x − a|` is at most
/// `|c| + r + |w_p|·(|n|(|c| + r) + √D)/Q(w_p)`, from the root's own formula.
/// A pipe square to a cone's axis of length one has `n = w_p` exactly and
/// `δ = 0`.
///
/// # Errors
///
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] where `D` is not
/// proved above zero over the pipe.
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] where the rounded facing is
/// not proved to hold the loops apart.
///
/// Whatever [`HalfSpaceThroughAPoint::new`] refuses.
fn two_loops_either_side_of_a_cones_slant_plane(
    cone: &Cone,
    pipe: &Cylinder,
) -> Result<SurfaceMeet> {
    let slant = a_cones_slant_against_a_pipe_enclosing(cone, pipe);
    let along = exactly(pipe.axis().direction().components());
    let steepness = a_cones_form_enclosing(cone, along);
    let up = exactly(cone.axis().direction().components());
    let squared_up = squared_length_enclosing(up);
    let widening = Interval::exactly(1.0) + squared_enclosing(cone.slope());
    let stretch: [[Interval; 3]; 3] = core::array::from_fn(|row| {
        core::array::from_fn(|column| {
            let diagonal = if row == column {
                squared_up
            } else {
                Interval::exactly(0.0)
            };
            slant[row] * slant[column] - steepness * (diagonal - widening * up[row] * up[column])
        })
    });
    let across = gap_enclosing(
        cone.apex().coordinates(),
        pipe.axis().origin().coordinates(),
    );
    let stretched: [Interval; 3] = core::array::from_fn(|row| dot_enclosing(stretch[row], across));
    let at_the_axis = dot_enclosing(across, stretched);
    let skin = Interval::exactly(pipe.radius());
    let largest_stretch = root_enclosing(
        stretch
            .iter()
            .flatten()
            .fold(Interval::exactly(0.0), |so_far, part| {
                so_far + *part * *part
            }),
    );
    let give = Interval::exactly(2.0) * skin * root_enclosing(squared_length_enclosing(stretched))
        + skin * skin * largest_stretch;
    let least = (at_the_axis - give).lo();
    let most = (at_the_axis + give).hi();
    if least.is_nan() || least <= 0.0 {
        return Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface);
    }

    let rounded = slant.map(|part| (part.lo() + part.hi()) / 2.0);
    let off_by = root_enclosing(squared_length_enclosing(
        slant.map(|part| Interval::exactly(part.width())),
    ));
    let reach = root_enclosing(squared_length_enclosing(across)) + skin;
    let furthest = reach
        + root_enclosing(squared_length_enclosing(along))
            * quotient_or_everything(
                root_enclosing(squared_length_enclosing(slant)) * reach
                    + root_enclosing(Interval::exactly(most)),
                steepness,
            );
    if root_enclosing(Interval::exactly(least)).lo() <= (off_by * furthest).hi() {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    let picker = Picker::from(HalfSpaceThroughAPoint::new(
        cone.apex(),
        Vector::new(rounded[0], rounded[1], rounded[2]),
    )?);
    two_loops_picked_either_side_of(picker, cone)
}

/// The two loops either side of `picker`'s flat surface, each named as the
/// part of where the two surfaces cross that one side holds.
fn two_loops_picked_either_side_of(picker: Picker, cone: &Cone) -> Result<SurfaceMeet> {
    let other_side = picker
        .reversed()
        .expect("the picker is flat, so it has another side");
    let and = Surface::Cone(*cone);
    Ok(SurfaceMeet::Curves(
        [picker, other_side]
            .into_iter()
            .map(|picker| {
                NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
                    and,
                    picked_by: smallvec![picker],
                })
                .expect("picked_by always holds the one flat picker just built")
            })
            .collect(),
    ))
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
/// `w_c·(x − a)` is `±r·‖w_c‖/slope`: the pipe is `r` wide (§118.11), `r/slope`
/// is how far along the *unit* cone axis a circle sits, and reading that with
/// the stored direction multiplies by `‖w_c‖`. **Only the cone's own length is
/// carried.** The pipe's used to be divided out here as well, because the width
/// it certified was then `r/‖w_p‖`, and the two stored directions run along one
/// line without being one triple — so the two lengths did not cancel.
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

/// `r·‖w_c‖/slope`, which is what the cone's own stored direction reads at
/// either of the two circles, as an enclosure.
fn how_far_along_the_axis_the_two_circles_sit(cone: &Cone, pipe: &Cylinder) -> Interval {
    let up_the_cone = root_enclosing(squared_length_enclosing(exactly(
        cone.axis().direction().components(),
    )));
    quotient_or_everything(
        Interval::exactly(pipe.radius()) * up_the_cone,
        Interval::exactly(cone.slope()),
    )
}

/// Two cones: circles or the apex where they share an axis, rulings through the
/// apex where they share only that, refused everywhere else.
///
/// # On one axis it is a table of four, and no row of it is a line
///
/// A [`Cone`] is both nappes, so two cones on one axis, of slopes `s₁` and `s₂`
/// with their apexes `d` apart along it, meet where `s₁·|h| = s₂·|h − d|`, with
/// `h` measured along the axis from the first apex. That is two linear
/// equations, one per sign the two sides can take together:
///
/// | the pair | the crossing |
/// |---|---|
/// | one apex, one slope | one cone — [`one_cone_either_way_round`], before here |
/// | one apex, two slopes | the apex and nothing else — [`SurfaceMeet::TouchAtOnePoint`] |
/// | two apexes, one slope | one circle, halfway between the apexes |
/// | two apexes, two slopes | two circles — [`two_circles_round_a_cones_axis`] |
///
/// **None of them is a tangency.** At a circle the two sides lean against the
/// axis at `±s₁` and `±s₂`, and those agree only for one slope with `h` and
/// `h − d` of one sign — and the one-slope circle sits at `h = d/2`, where the
/// two signs differ.
///
/// **And none of them holds a straight line.** Two cones meet along a ruling
/// only where they share an apex and their axes lean apart, and then that is
/// all they meet in: two rulings or four through the apex, or the apex alone —
/// [`the_rulings_two_cones_on_one_apex_share`], asked once the first question
/// below has found the axes apart. That is the one pair `rcad-topo`'s
/// `the_crossing_of_these_runs_straight` answers straight, and it asks the
/// same two things (§50.2 item 4).
///
/// # Sharing an axis is the two questions a cone and a pipe ask
///
/// `‖w₁ × w₂‖² = 0` for the directions, `‖(a₂ − a₁) × w₁‖² = 0` for the second
/// apex, both sums of squares, asked the way [`where_a_cone_and_a_pipe_cross`]
/// asks them. Whether the two apexes are one point is asked of the stored
/// numbers and needs no arithmetic.
///
/// # Errors
///
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] where the two share
/// neither an axis nor an apex.
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] where two circles are not
/// held apart by more than the picking half-space's own rounding, and where
/// [`the_rulings_two_cones_on_one_apex_share`] cannot tell the halves of the
/// rulings apart.
///
/// [`Error::Undecidable`] where a sum of squares could not be settled, which
/// takes a product leaving `f64`'s range.
fn where_two_cones_cross(
    one: &Cone,
    other: &Cone,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let at = [one.apex(), other.apex()].map(|apex| PredicatePoint::from(apex.coordinates()));

    // Two unit directions crossed together is dimensionless, and filed as a
    // length for the reason `where_a_plane_and_a_pipe_cross` gives.
    match ask(
        &two_cones_point_along_one_direction(one, other),
        Measures::Length,
        &at,
        policy,
        log,
    )? {
        Decision::Zero => (),
        Decision::Pos if one.apex() == other.apex() => {
            return the_rulings_two_cones_on_one_apex_share(one, other);
        }
        Decision::Pos => return Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
        Decision::Neg | Decision::Undecidable { .. } | _ => {
            unreachable!("a sum of squares asked through `ask` is never below zero nor undecided")
        }
    }
    match ask(
        &a_point_sits_on_a_cones_axis(WHERE_TWO_CONES_CROSS, other.apex(), one),
        Measures::Area,
        &at,
        policy,
        log,
    )? {
        Decision::Zero => two_cones_on_one_axis(one, other),
        Decision::Pos => Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
        Decision::Neg | Decision::Undecidable { .. } | _ => {
            unreachable!("a sum of squares asked through `ask` is never below zero nor undecided")
        }
    }
}

/// What two cones on one apex with their axes leaning apart share: the halves
/// the apex cuts two rulings or four into, or the apex alone.
///
/// # Where the rulings are
///
/// With `w₁`, `w₂` the stored directions and `s₁`, `s₂` the slopes, a way `d`
/// out of the apex is on cone `i` where `(1 + sᵢ²)(d·wᵢ)² = ‖d‖²‖wᵢ‖²`, so it is
/// on both where `(1 + s₁²)‖w₂‖²(d·w₁)² = (1 + s₂²)‖w₁‖²(d·w₂)²` and on the
/// first. The first of those is two flat surfaces through the apex, square to
/// `n± = a₁·w₁ ± a₂·w₂` for `a₁ = √((1 + s₁²)‖w₂‖²)` and
/// `a₂ = √((1 + s₂²)‖w₁‖²)`. So the crossing is what those two cut out of the
/// first cone, and each cuts what a plane through a cone's apex does: two
/// rulings where `−E·‖w₁‖²` of [`two_lines_through_a_cones_apex`], worked out
/// with `n±`, is above nothing, and the apex alone where it is below. No ruling
/// is on both flat surfaces, since that takes `d·w₁ = d·w₂ = 0`, and no way
/// square to a cone's axis is on it.
///
/// # How the halves are told apart
///
/// Both cones are their own mirror images across the flat surface through the
/// apex that holds both axes, the one facing `w₁ × w₂`, and so is each of the
/// two flat surfaces. So the two rulings either cuts are mirror images across
/// it, and each half is strictly on one side of it, as in
/// [`four_half_rulings_through_a_cones_apex`]. The flat surface square to `w₁`
/// parts the first cone's two nappes, so the four halves of one flat surface
/// are in four different corners of those two. Where both cut, the halves on
/// `n₋` read `d·w₁` and `d·w₂` of one sign and those on `n₊` of opposite signs,
/// so a third surface, square to `w₂`, puts all eight in eight corners.
///
/// What is proved, rather than taken from the paragraph above, is what
/// [`four_half_rulings_through_a_cones_apex`] proves: every picking surface
/// passes exactly through the apex, and each half's enclosure reads strictly
/// one side of each, in a corner of its own. `n±` itself is an enclosure, since
/// `a₁` and `a₂` are roots.
///
/// **The pair is sorted first**, by [`Surface::how_it_sorts_against`], so the
/// cone whose rulings and nappes are read is the same whichever way round the
/// pair came in.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] where the enclosure of
/// `−E·‖w₁‖²` for either flat surface holds nothing — which is where the two
/// cones touch along a ruling, and where they nearly do — and where anything
/// above cannot be shown; and whatever [`HalfSpace::through_a_point_facing`]
/// refuses.
fn the_rulings_two_cones_on_one_apex_share(one: &Cone, other: &Cone) -> Result<SurfaceMeet> {
    let (first, second) = match Surface::Cone(*one).how_it_sorts_against(&Surface::Cone(*other)) {
        Ordering::Greater => (other, one),
        Ordering::Less | Ordering::Equal => (one, other),
    };
    let mut halves = Vec::new();
    for side in [1.0, -1.0] {
        let normal = square_to_where_two_cones_on_one_apex_cross(first, second, side);
        let spread = how_far_apart_two_rulings_through_a_cones_apex_spread(normal, first);
        if spread.hi() < 0.0 {
            continue;
        }
        if spread.lo() <= 0.0 {
            return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
        }
        let Some(ways) = the_four_ways_two_rulings_leave_a_cones_apex(normal, first) else {
            unreachable!("a spread above nothing leaves the apex four ways");
        };
        halves.extend(ways);
    }

    let apex = first.apex();
    if halves.is_empty() {
        return Ok(SurfaceMeet::TouchAtOnePoint {
            at: PredicatePoint::from(apex.coordinates()),
        });
    }
    let [along_the_first, along_the_second] = [first, second].map(|cone| cone.axis().direction());
    let mut surfaces = vec![
        HalfSpace::through_a_point_facing(apex, along_the_first.cross(along_the_second))?,
        HalfSpace::through_a_point_facing(apex, along_the_first)?,
    ];
    if halves.len() == 8 {
        surfaces.push(HalfSpace::through_a_point_facing(apex, along_the_second)?);
    }
    if !surfaces
        .iter()
        .all(|surface| it_passes_exactly_through(surface, apex))
    {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    let mut corners: Vec<Vec<Decision>> = Vec::new();
    let mut runs = Vec::new();
    for half in halves {
        let corner: Vec<Decision> = surfaces
            .iter()
            .map(|surface| dot_enclosing(exactly(surface.normal().components()), half).sign())
            .collect();
        if !corner
            .iter()
            .all(|sign| matches!(sign, Decision::Pos | Decision::Neg))
            || corners.contains(&corner)
        {
            return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
        }
        let picked_by = surfaces
            .iter()
            .zip(&corner)
            .map(|(surface, sign)| {
                Picker::from(if *sign == Decision::Pos {
                    *surface
                } else {
                    surface.reversed()
                })
            })
            .collect();
        corners.push(corner);
        runs.push(NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
            and: Surface::Cone(*second),
            picked_by,
        })?);
    }
    Ok(SurfaceMeet::Curves(runs))
}

/// An enclosure of `n± = a₁·w₁ ± a₂·w₂`, `side` choosing the sign, for
/// [`the_rulings_two_cones_on_one_apex_share`]: `√((1 + s₁²)‖w₂‖²)` and
/// `√((1 + s₂²)‖w₁‖²)` are `a₁` and `a₂`, so no length is divided out.
fn square_to_where_two_cones_on_one_apex_cross(
    first: &Cone,
    second: &Cone,
    side: f64,
) -> [Interval; 3] {
    let [along_the_first, along_the_second] =
        [first, second].map(|cone| exactly(cone.axis().direction().components()));
    let widened = |cone: &Cone, along: [Interval; 3]| {
        root_enclosing(
            (Interval::exactly(1.0) + squared_enclosing(cone.slope()))
                * squared_length_enclosing(along),
        )
    };
    let by_the_first = widened(first, along_the_second);
    let by_the_second = widened(second, along_the_first) * Interval::exactly(side);
    core::array::from_fn(|axis| {
        by_the_first * along_the_first[axis] + by_the_second * along_the_second[axis]
    })
}

/// Which row of [`where_two_cones_cross`]'s table two cones on one axis are.
///
/// **One apex and one slope is not reached under a certified policy**, because
/// [`one_cone_either_way_round`] answers it first. It is answered here as one
/// surface all the same, because a policy that accepted the two axes as one
/// line where the exact test could not has said that, and what it costs is that
/// policy's own budget.
fn two_cones_on_one_axis(one: &Cone, other: &Cone) -> Result<SurfaceMeet> {
    if one.apex() == other.apex() {
        if one.slope() == other.slope() {
            return Ok(SurfaceMeet::SameSurface);
        }
        return Ok(SurfaceMeet::TouchAtOnePoint {
            at: PredicatePoint::from(one.apex().coordinates()),
        });
    }
    match one.slope().total_cmp(&other.slope()) {
        Ordering::Equal => Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
            EdgeRun::RightRoundWhereItsFacesCross,
        )?])),
        Ordering::Greater => two_circles_two_cones_cross_in(one, other),
        Ordering::Less => two_circles_two_cones_cross_in(other, one),
    }
}

/// The two circles two cones of two slopes on one axis cross in, told apart by
/// the flat half-space through the steeper cone's apex.
///
/// With `g = (a_shallow − a_steep)·w_steep`, which is how far the shallow apex
/// sits up the steep cone's stored axis, the two circles are where the steep
/// cone's stored direction reads `g·s_shallow/(s_steep + s_shallow)` and
/// `−g·s_shallow/(s_steep − s_shallow)`: one between the two apexes and one
/// beyond the steeper one's, so on opposite sides of it.
fn two_circles_two_cones_cross_in(steep: &Cone, shallow: &Cone) -> Result<SurfaceMeet> {
    let up = dot_enclosing(
        gap_enclosing(steep.apex().coordinates(), shallow.apex().coordinates()),
        exactly(steep.axis().direction().components()),
    );
    let [steeper, shallower] = [steep, shallow].map(|cone| Interval::exactly(cone.slope()));
    let reach = up * shallower;
    two_circles_round_a_cones_axis(
        steep,
        steep.apex(),
        [
            quotient_or_everything(reach, steeper + shallower),
            -quotient_or_everything(reach, steeper - shallower),
        ],
    )
}

/// A cone and a ball: circles where the ball's centre is on the cone's axis,
/// refused where it is not.
///
/// # On the axis it is one quadratic
///
/// Write `h` for how far along the axis from the apex a circle of the crossing
/// sits, and `e` for how far the ball's centre does. The circle is `s·|h|`
/// across and every point of it is `R` from the centre, so
/// `(h − e)² + s²h² = R²`, which is `(1 + s²)h² − 2eh + e² − R² = 0` — the same
/// equation on both nappes, because `h²` does not care which one `h` is on. Its
/// discriminant over four is `(1 + s²)R² − s²e²`, and `e² = ‖c − a‖²` exactly
/// once the centre is proved on the axis, so the sign is a polynomial in the
/// stored numbers:
///
/// | `(1 + s²)R² − s²‖c − a‖²` | the crossing |
/// |---|---|
/// | below zero | nothing: the ball sits inside one nappe or outside both |
/// | zero | one circle the ball touches all the way round, inscribed in the cone |
/// | above zero | two circles, unless one root is `h = 0` |
///
/// A tangent circle is answered as one curve, the way a ball inscribed in a
/// pipe is (§24.10). It is never at the apex: `h = e/(1 + s²)` there, and
/// `e = 0` with a zero discriminant takes `R = 0`.
///
/// # A ball through the apex is refused
///
/// `h = 0` is a root when `‖c − a‖ = R`, and then the crossing is one circle
/// **and the apex**, which the ball's skin passes through while the cone's tip
/// sits inside the ball or outside it. That is a curve and a point apart from
/// it, and [`SurfaceMeet`] has no arm that says both: one circle loses the apex
/// a fuse would have to put a vertex on, and one touching point loses the
/// circle. It stays under this row's refusal rather than being said badly.
///
/// # Errors
///
/// [`Error::ACrossingBetweenAConeAndAnotherCurvedSurface`] where the centre is
/// not on the axis, and where the ball runs through the apex.
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] where two circles are not
/// held apart by more than the picking half-space's rounding.
///
/// [`Error::Undecidable`] where a sign could not be settled, which takes a
/// product leaving `f64`'s range.
fn where_a_cone_and_a_ball_cross(
    cone: &Cone,
    ball: &Sphere,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    let at = [cone.apex(), ball.centre()].map(|place| PredicatePoint::from(place.coordinates()));

    match ask(
        &a_point_sits_on_a_cones_axis(WHERE_A_CONE_AND_A_BALL_CROSS, ball.centre(), cone),
        Measures::Area,
        &at,
        policy,
        log,
    )? {
        Decision::Zero => (),
        Decision::Pos => return Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
        Decision::Neg | Decision::Undecidable { .. } | _ => {
            unreachable!("a sum of squares asked through `ask` is never below zero nor undecided")
        }
    }

    match ask(
        &how_much_room_a_ball_leaves_round_a_cone(cone, ball),
        Measures::Area,
        &at,
        policy,
        log,
    )? {
        Decision::Neg => Ok(SurfaceMeet::Nothing),
        Decision::Zero => Ok(SurfaceMeet::Curves(vec![NamedRun::the_whole_of(
            EdgeRun::RightRoundWhereItsFacesCross,
        )?])),
        Decision::Pos => {
            match ask(
                &where_a_ball_stands_against_a_cones_apex(cone, ball),
                Measures::Area,
                &at,
                policy,
                log,
            )? {
                Decision::Zero => Err(Error::ACrossingBetweenAConeAndAnotherCurvedSurface),
                Decision::Neg | Decision::Pos => {
                    two_circles_a_ball_cuts_round_a_cones_axis(cone, ball)
                }
                Decision::Undecidable { .. } | _ => {
                    unreachable!("`ask` hands back a sign or an error")
                }
            }
        }
        Decision::Undecidable { .. } | _ => unreachable!("`ask` hands back a sign or an error"),
    }
}

/// The two circles a ball on a cone's axis cuts it in, told apart by the flat
/// half-space square to the axis halfway between them.
///
/// What the stored direction reads at either circle is `‖w‖·h` with
/// `h = (e ± √D)/(1 + s²)`, which is `(g ± √(W·D))/(1 + s²)` with
/// `g = (c − a)·w` and `W = w·w`, so no `e` is ever divided out.
fn two_circles_a_ball_cuts_round_a_cones_axis(cone: &Cone, ball: &Sphere) -> Result<SurfaceMeet> {
    let along = cone.axis().direction();
    let halfway = halfway_between_a_cones_apex_and_a_ball_on_its_axis(cone, ball);

    let up = dot_enclosing(
        gap_enclosing(cone.apex().coordinates(), ball.centre().coordinates()),
        exactly(along.components()),
    );
    let room = how_much_room_a_ball_leaves_round_a_cone_exactly(cone, ball).map_or_else(
        || how_much_room_a_ball_leaves_round_a_cone_enclosing(cone, ball),
        |room| room.enclosure(),
    );
    let either_way = root_enclosing(squared_length_enclosing(exactly(along.components())) * room);
    let widening = Interval::exactly(1.0) + squared_enclosing(cone.slope());
    two_circles_round_a_cones_axis(
        cone,
        halfway,
        [
            quotient_or_everything(up + either_way, widening),
            quotient_or_everything(up - either_way, widening),
        ],
    )
}

/// Halfway along a cone's axis between its apex and a ball centred on it,
/// `a + (c − a)/(1 + s²)` with `s` the cone's slope.
///
/// **Halfway, because neither stored point is always between the two circles
/// a ball on a cone's axis cuts it in.** The apex is between the two exactly
/// when it is inside the ball, and the ball's centre exactly when the ball
/// reaches out past the cone level with it; a ball sitting in one nappe and
/// pushing out through its side, nearer the apex than its own centre, has both
/// circles on one side of both points. Halfway is `h = e/(1 + s²)` along the
/// axis, which is this point, and it rounds — so [`two_circles_round_a_cones_axis`]
/// tests the separation rather than arranging it: the half-space built through
/// this point has to read one circle above zero and the other below it with
/// the rounding counted in, or that refuses.
///
/// One function, so the half-space the ordering picker in
/// [`along_a_crossing`](crate::along_a_crossing) builds can never pick a
/// circle other than the one this crate's own `intersect` hands back as
/// `through`.
pub(crate) fn halfway_between_a_cones_apex_and_a_ball_on_its_axis(
    cone: &Cone,
    ball: &Sphere,
) -> Point {
    let squared_slope = cone.slope() * cone.slope();
    cone.apex() + (ball.centre() - cone.apex()) * (1.0 + squared_slope).recip()
}

/// Two circles round a cone's axis, told apart by the flat half-space through
/// `through` facing along it.
///
/// `sitting_at` is what the cone's own stored direction `w` reads at each
/// circle from the apex, `w·(x − a)`. The half-space's normal **is** `w` — a
/// [`HalfSpace`] stores what it is handed — and every point of a circle round
/// the axis sits square to `w` from the axis, so what the half-space reads at
/// any point of a circle is `w·(x − a)` plus how far its own offset sits off
/// the apex, and the sideways part adds exactly nothing, for the reason
/// [`two_circles_round_a_shared_axis`] gives. When one reading is provably
/// above zero and the other provably below, the stored half-space holds one
/// circle and its reverse holds the other, whatever `through` rounded to.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] where that is not proved. A
/// **gap**: the two circles are there and they are tellable apart.
///
/// [`Error::NotFinite`] where `through` came out of range.
fn two_circles_round_a_cones_axis(
    cone: &Cone,
    through: Point,
    sitting_at: [Interval; 2],
) -> Result<SurfaceMeet> {
    let between = HalfSpace::through_a_point_facing(through, cone.axis().direction())?;
    let off_the_apex = dot_enclosing(
        exactly(between.normal().components()),
        exactly(cone.apex().coordinates()),
    ) - Interval::exactly(between.offset());
    let [one, other] = sitting_at.map(|reach| off_the_apex + reach);
    let apart = (one.lo() > 0.0 && other.hi() < 0.0) || (one.hi() < 0.0 && other.lo() > 0.0);
    if !apart {
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

/// Whether two cones share one axis, **proved and never rounded**.
///
/// [`Decision::Zero`] and the two stored axes name one line. Every other answer
/// means *not proved*, and a caller treats it as a refusal, for the reasons
/// [`whether_a_cone_and_a_pipe_share_an_axis`] gives at length: `walk` asks
/// this to decide which shape it is drawing, and a `Zero` it has not earned
/// would draw circles where two cones cross in a quartic.
pub(crate) fn whether_two_cones_share_an_axis(one: &Cone, other: &Cone) -> Decision {
    let directions = decide(&two_cones_point_along_one_direction(one, other)).0;
    if directions != Decision::Zero {
        return directions;
    }
    decide(&a_point_sits_on_a_cones_axis(
        WHERE_TWO_CONES_CROSS,
        other.apex(),
        one,
    ))
    .0
}

/// Whether two cones are the pair [`the_rulings_two_cones_on_one_apex_share`]
/// names, one apex with their axes **proved** apart, which is the pair whose
/// crossing is straight lines. No [`Policy`], for the reason
/// [`whether_two_cones_share_an_axis`] gives.
pub(crate) fn two_cones_lean_apart_on_one_apex(one: &Cone, other: &Cone) -> bool {
    one.apex() == other.apex()
        && decide(&two_cones_point_along_one_direction(one, other)).0 == Decision::Pos
}

/// Whether a ball's centre is on a cone's axis, proved the same way and for
/// the same caller.
pub(crate) fn whether_a_ball_sits_on_a_cones_axis(cone: &Cone, ball: &Sphere) -> Decision {
    decide(&a_point_sits_on_a_cones_axis(
        WHERE_A_CONE_AND_A_BALL_CROSS,
        ball.centre(),
        cone,
    ))
    .0
}

/// The sign of `(1 + s²)R² − s²‖c − a‖²`, [`where_a_cone_and_a_ball_cross`]'s
/// discriminant, for `walk` to build the circles the namer named. No [`Policy`],
/// for the reason [`whether_two_cones_share_an_axis`] gives.
pub(crate) fn how_a_ball_on_a_cones_axis_meets_it(cone: &Cone, ball: &Sphere) -> Decision {
    decide(&how_much_room_a_ball_leaves_round_a_cone(cone, ball)).0
}

/// The sign of `‖c − a‖² − R²`: [`Decision::Zero`] where the ball's skin runs
/// through the cone's apex, which both halves refuse.
pub(crate) fn whether_a_ball_runs_through_a_cones_apex(cone: &Cone, ball: &Sphere) -> Decision {
    decide(&where_a_ball_stands_against_a_cones_apex(cone, ball)).0
}

/// `‖w₁ × w₂‖²` for two cones, as a question rather than an answer.
fn two_cones_point_along_one_direction(one: &Cone, other: &Cone) -> impl Predicate {
    squared_length_of_a_cross_product(
        WHERE_TWO_CONES_CROSS,
        one.axis().direction().components(),
        other.axis().direction().components(),
    )
}

/// `‖(p − a) × w‖²`: how far `point` sits off the cone's axis, as a question.
fn a_point_sits_on_a_cones_axis(named: &'static str, point: Point, cone: &Cone) -> impl Predicate {
    let point = point.coordinates();
    let apex = cone.apex().coordinates();
    let along = cone.axis().direction().components();
    Question {
        named,
        enclosing: move || how_far_off_an_axis_enclosing(point, apex, along),
        exactly: move || how_far_off_an_axis_exactly(point, apex, along),
    }
}

/// `(1 + s²)R² − s²‖c − a‖²`, as a question.
fn how_much_room_a_ball_leaves_round_a_cone(cone: &Cone, ball: &Sphere) -> impl Predicate {
    let (cone, ball) = (*cone, *ball);
    Question {
        named: WHERE_A_CONE_AND_A_BALL_CROSS,
        enclosing: move || how_much_room_a_ball_leaves_round_a_cone_enclosing(&cone, &ball),
        exactly: move || how_much_room_a_ball_leaves_round_a_cone_exactly(&cone, &ball),
    }
}

/// `(1 + s²)R² − s²‖c − a‖²` as an enclosure.
fn how_much_room_a_ball_leaves_round_a_cone_enclosing(cone: &Cone, ball: &Sphere) -> Interval {
    let squared_slope = squared_enclosing(cone.slope());
    (Interval::exactly(1.0) + squared_slope) * squared_enclosing(ball.radius())
        - squared_slope
            * squared_length_enclosing(gap_enclosing(
                cone.apex().coordinates(),
                ball.centre().coordinates(),
            ))
}

/// `(1 + s²)R² − s²‖c − a‖²`, exactly.
fn how_much_room_a_ball_leaves_round_a_cone_exactly(cone: &Cone, ball: &Sphere) -> Option<Exact> {
    let squared_slope = Exact::product_of(cone.slope(), cone.slope())?;
    let squared_radius = Exact::product_of(ball.radius(), ball.radius())?;
    Some(
        squared_radius
            .plus(&squared_slope.times(&squared_radius)?)
            .minus(&squared_slope.times(&squared_gap_exactly(
                cone.apex().coordinates(),
                ball.centre().coordinates(),
            )?)?),
    )
}

/// `‖c − a‖² − R²`, as a question.
fn where_a_ball_stands_against_a_cones_apex(cone: &Cone, ball: &Sphere) -> impl Predicate {
    let apex = cone.apex().coordinates();
    let centre = ball.centre().coordinates();
    let radius = ball.radius();
    Question {
        named: WHERE_A_CONE_AND_A_BALL_CROSS,
        enclosing: move || {
            squared_length_enclosing(gap_enclosing(apex, centre)) - squared_enclosing(radius)
        },
        exactly: move || {
            Some(squared_gap_exactly(apex, centre)?.minus(&Exact::product_of(radius, radius)?))
        },
    }
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
/// **That costs nothing now.** The picking pipe is `R` wide, exactly the radius
/// it was built with, because a pipe means the same thing whatever its stored
/// direction rounded to (§118.11). So the two circles sit `√(r² − t²)` either
/// side of it and the only way the sign question downstream comes back
/// [`Decision::Zero`] or [`Decision::Undecidable`] is a torus whose tube really
/// does reach its own ring. It used to cost a rounding of `R`: the picking
/// pipe's certified radius was `R / ‖w‖`, and a pair whose gap was smaller than
/// that was refused for the axis's own rounding rather than for its shape.
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
/// either side of the flat surface through the centre square to the true
/// `w × n`, and a radius of `r`, so each is `R − r` clear of it — a length, and
/// strictly positive because a torus's tube is strictly narrower than its ring.
///
/// The picker is not quite that surface, and the margin is `R − r` less the two
/// ways it differs, each in the same unit:
///
/// - **it sits off the centre** by the rounding in its offset, which
///   [`how_far_a_picking_half_space_sits_off_a_point`] answers as a length
///   because it divides the reading by `‖w × n‖` as stored;
/// - **it is turned** by the rounding in `w × n` itself, and a surface through
///   the centre turned by `θ` leaves each circle `R·cos θ − r` clear, so the
///   loss is `R·(1 − cos θ)`, at most `R·sin²θ`.
///
/// `R − r` is enclosed and not rounded. The `f64` difference is exact only when
/// `r ≥ R/2` and can round up by half an ulp otherwise. Against **one** `f64`
/// bound on the losses that rounding could never matter, because no `f64` lies
/// between `R − r` and its rounding; against two bounds added together it can,
/// because their sum need not be an `f64` (`docs/decisions.md` §314).
///
/// # Errors
///
/// [`Error::ZeroLength`] never fires: this is only reached with `n × w` proved
/// non-zero one function up. [`Error::NotFinite`] when the offset overflows,
/// and [`Error::CannotNameWhichPieceOfACrossingThisIs`] when those two losses
/// are not provably smaller than `R − r` — which takes a torus whose two radii
/// are within a rounding of each other, or a centre some `2⁵²` ring radii out
/// across the plane so that the offset's rounding reaches the tube — or when a
/// product of the axis's and the normal's components is too small to be worked
/// out exactly, and the turn cannot be bounded at all.
fn two_meridian_circles(flat: &Plane, ring: &Torus) -> Result<SurfaceMeet> {
    let axis = ring.axis().direction();
    let across = axis.cross(flat.normal());
    let between = HalfSpace::through_a_point_facing(ring.centre(), across)?;
    let Some(turned) = how_far_a_rounded_cross_product_turns(axis, flat.normal(), across) else {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    };
    let ring_radius = Interval::exactly(ring.ring_radius());
    let margin = ring_radius
        - Interval::exactly(ring.tube_radius())
        - ring_radius * turned
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

/// An upper bound on `sin²θ`, where `θ` is the angle between `across`, the
/// `f64` cross product of `axis` and `normal`, and their true cross product.
///
/// `across` is the true product plus a rounding `e`, and turning it by `θ`
/// moves its tip at least `‖across‖·sin θ` off the true direction, so
/// `sin θ ≤ ‖e‖ / ‖across‖`. Both `e` and its length are worked out from the
/// exact product, and `None` is when that product cannot be: a product of two
/// components landing among the subnormals.
fn how_far_a_rounded_cross_product_turns(
    axis: Vector,
    normal: Vector,
    across: Vector,
) -> Option<Interval> {
    let exact = Exact::cross_of(&Exact::of_each(axis.components()), normal.components())?;
    let across = across.components();
    let rounding: [Interval; 3] =
        core::array::from_fn(|at| Exact::of(across[at]).minus(&exact[at]).enclosure());
    Some(quotient_or_everything(
        squared_length_enclosing(rounding),
        squared_length_enclosing(exactly(across)),
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

/// A ring and a ball, a pipe or a cone: circles square to the ring's axis
/// where the two share it, each named by the plane it lies in
/// ([`circles_round_a_rings_axis`]), a circle and a loop for a pipe of the
/// tube's radius along the ring's core circle
/// ([`where_a_ring_and_a_pipe_along_its_core_cross`]), nothing for a pipe along
/// the ring's axis off it and clear of the tube
/// ([`the_heights_a_ring_crosses_a_pipe_at`]), and refused otherwise.
///
/// # Errors
///
/// Those of [`a_pipe_lies_along_a_rings_core`],
/// [`where_a_ring_and_a_pipe_along_its_core_cross`] and
/// [`the_heights_a_ring_crosses_a_round_surface_at`], and
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] where
/// [`circles_round_a_rings_axis`] cannot write a plane down for each circle.
fn where_a_ring_and_a_round_surface_cross(
    ring: &Torus,
    partner: &Surface,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SurfaceMeet> {
    if let Surface::Cylinder(pipe) = partner
        && a_pipe_lies_along_a_rings_core(ring, pipe, policy, log)?
    {
        return where_a_ring_and_a_pipe_along_its_core_cross(ring, pipe);
    }
    let heights = the_heights_a_ring_crosses_a_round_surface_at(ring, partner, policy, log)?;
    if heights.is_empty() {
        return Ok(SurfaceMeet::Nothing);
    }
    circles_round_a_rings_axis(ring, &heights)
}

/// Whether a pipe has the tube's radius and its axis lies along the ring's
/// core circle, touching it at one point: the pipe a bar butted square onto a
/// bend's cap is cut on.
///
/// Four questions, each zero exactly when it holds: the two radii are the same
/// number; the pipe's direction `d` is square to the ring's axis direction `w`,
/// `d·w`; the pipe's origin `o` is level with the ring's centre `c`, `(o − c)·w`;
/// and the pipe's axis is the ring radius `R` from the centre,
/// `‖(o − c) × d‖² − R²(d·d)`, degree two in `d` on both terms for the reason
/// the module doc gives.
///
/// # Errors
///
/// [`Error::Undecidable`] where a sign could not be settled, which takes a
/// product leaving `f64`'s range.
pub(crate) fn a_pipe_lies_along_a_rings_core(
    ring: &Torus,
    pipe: &Cylinder,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    if pipe.radius() != ring.tube_radius() {
        return Ok(false);
    }
    let centre = ring.centre().coordinates();
    let up = ring.axis().direction().components();
    let origin = pipe.axis().origin().coordinates();
    let along = pipe.axis().direction().components();
    let core = ring.ring_radius();
    let at = [centre, origin].map(PredicatePoint::from);

    let square = Question {
        named: WHERE_A_TORUS_AND_A_PIPE_CROSS,
        enclosing: || dot_enclosing(exactly(along), exactly(up)),
        exactly: || Exact::dot_of(along, up),
    };
    let level = Question {
        named: WHERE_A_TORUS_AND_A_PIPE_CROSS,
        enclosing: || dot_enclosing(gap_enclosing(centre, origin), exactly(up)),
        exactly: || {
            Exact::dot_of_two_exact(&Exact::gap_between(centre, origin), &Exact::of_each(up))
        },
    };
    let on_the_core = Question {
        named: WHERE_A_TORUS_AND_A_PIPE_CROSS,
        enclosing: || {
            how_far_off_an_axis_enclosing(centre, origin, along)
                - squared_enclosing(core) * squared_length_enclosing(exactly(along))
        },
        exactly: || {
            Some(
                how_far_off_an_axis_exactly(centre, origin, along)?
                    .minus(&Exact::product_of(core, core)?.times(&Exact::dot_of(along, along)?)?),
            )
        },
    };
    // Two directions dotted is dimensionless, and filed as a length for the
    // reason `where_a_plane_and_a_pipe_cross` gives.
    Ok(
        ask(&square, Measures::Length, &at, policy, log)? == Decision::Zero
            && ask(&level, Measures::Length, &at, policy, log)? == Decision::Zero
            && ask(&on_the_core, Measures::Area, &at, policy, log)? == Decision::Zero,
    )
}

/// A ring and a pipe of its tube's radius lying along its core circle: **the
/// circle where the two touch, and a loop where they cross**, the loop named in
/// two halves.
///
/// Put the ring's centre at the origin, its axis along `z`, and the pipe's axis
/// along `y` through `(R, 0, 0)`. On the pipe `(x − R)² + z² = r²`, and on the
/// ring `(√(x² + y²) − R)² + z² = r²`, so `√(x² + y²) − R = ±(x − R)`:
///
/// - the `+` sign is the plane `y = 0`, square to the pipe through the ring's
///   centre, and the pipe and the ring are tangent along the circle it cuts the
///   pipe in. That circle is a meridian of the ring and the rim of a bend's cap
///   a bar is butted onto, named here by the plane, which crosses the pipe in it
///   alone: [`EdgeRun::RightRoundWhereItCrosses`].
/// - the `−` sign is the loop `x = R − y²/4R`, `z² = r² − y⁴/16R²`, which
///   crosses the circle at `(R, 0, ±r)` and reaches `y = ±2√(Rr)` on the ring's
///   outer equator. `(9, 12, 4)` is on ring 12/5 and on the radius-5 pipe
///   through `(12, 0, 0)` along `y`.
///
/// Every picker is closed, so no set of them parts two curves that cross, and
/// the loop is named as **the half on each side of the circle's plane**: each
/// half lies on or in front of that plane, on or outside the pipe of the ring
/// radius round the ring's axis, where the loop is, since `x² + y² = R² + y²/2 +
/// y⁴/16R²`, and on or behind the plane square to the line from the centre to
/// the touching point at the touching point, where `x = R − y²/4R ≤ R`. The
/// circle meets all three only at the loop's two ends, the half the circle's
/// outer arc lying past the third and its inner arc inside the second. A bar on
/// a bend's cap has one half on each solid's curved face, where the other
/// solid's face does not reach it, so no edge of a fuse's answer runs along
/// either. Since Q1470 the fuse leaves each half off the face across the cap
/// from the face that carries it (`docs/decisions.md` §422); `rcad-topo`'s
/// `a_bar_cut_through_the_rings_axis_on_a_bends_cap_by_every_rule_against_the_closed_forms`
/// in `fuses_that_touch.rs` checks every rule against the closed forms.
///
/// **Nothing draws the loop**: it is a quartic, and the walk refuses it by name.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] where the pipe's direction
/// would not be kept as written as the circle plane's normal. Those of
/// [`HalfSpace::new`] where a picker's numbers are not finite.
fn where_a_ring_and_a_pipe_along_its_core_cross(
    ring: &Torus,
    pipe: &Cylinder,
) -> Result<SurfaceMeet> {
    let along = pipe.axis().direction();
    let across = Plane::from_point_and_normal_as_written(ring.centre(), along)?;
    if across.normal() != along {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }
    let centre = ring.centre();
    let off_the_centre = pipe.axis().origin() - centre;
    let squared = along.dot(along);
    let out_to_the_touch = off_the_centre * squared - along * off_the_centre.dot(along);
    let towards_the_axis = HalfSpace::new(
        -out_to_the_touch,
        -out_to_the_touch.dot(centre - Point::origin())
            - ring.ring_radius() * ring.ring_radius() * squared,
    )?;
    let outside_the_core = Cylinder::new(ring.axis(), ring.ring_radius())?;
    let mut runs = vec![NamedRun::the_whole_of(EdgeRun::RightRoundWhereItCrosses {
        and: Surface::Plane(across),
    })?];
    for facing in [along, -along] {
        runs.push(NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
            and: Surface::Torus(*ring),
            picked_by: smallvec![
                Picker::InFrontOf(HalfSpace::through_a_point_facing(centre, facing)?),
                Picker::OutsideAPipe(outside_the_core),
                Picker::InFrontOf(towards_the_axis),
            ],
        })?);
    }
    Ok(SurfaceMeet::Curves(runs))
}

/// Where a ring and a ball, a pipe or a cone on its axis share circles: how
/// far up each one sits, read along the ring's stored axis direction `w` from
/// its centre `c` — `w·(x − c)` for any point `x` of it — as an enclosure, in
/// no order. None where the partner misses the tube.
///
/// **One answer for two questions.** [`intersect`] names each circle by the
/// plane at its height, and `three_surfaces` places where a plane holding the
/// axis cuts each circle at the same height, so the corner a fuse finds where
/// a bend's cap crosses a bar on its axis is on the circle named here.
///
/// # Errors
///
/// [`Error::ACrossingBetweenATorusAndAnotherCurvedSurface`] where the partner
/// does not share the ring's axis, but for a pipe along it proved clear of
/// the tube, or is a plane or a torus.
/// [`Error::Undecidable`] where a sign could not be settled, which takes a
/// product leaving `f64`'s range.
pub(crate) fn the_heights_a_ring_crosses_a_round_surface_at(
    ring: &Torus,
    partner: &Surface,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SmallVec<[Interval; 4]>> {
    match partner {
        Surface::Cylinder(pipe) => the_heights_a_ring_crosses_a_pipe_at(ring, pipe, policy, log),
        Surface::Sphere(ball) => the_heights_a_ring_crosses_a_ball_at(ring, ball, policy, log),
        Surface::Cone(cone) => the_heights_a_ring_crosses_a_cone_at(ring, cone, policy, log),
        Surface::Plane(_) | Surface::Torus(_) => {
            Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface)
        }
    }
}

/// A ring and a pipe: the heights of the circles round the axis where the two
/// share it, none where the pipe runs along the axis off it and clear of the
/// tube, and refused otherwise.
///
/// # On one axis it is one sign
///
/// Cut both through the axis. The tube is the circle `(ρ − R)² + h² = r²`
/// round `(R, 0)`, and the pipe is the line `ρ = a`, so they meet where
/// `h² = D` with `D = r² − (a − R)²`, a polynomial in the stored numbers:
///
/// | `D` | the crossing |
/// |---|---|
/// | above zero | two circles of the pipe's radius, at `h = ±√D` |
/// | zero | one circle where the pipe rests on the tube's rim all the way round |
/// | below zero | nothing |
///
/// The resting circle is one curve, the way a ball inscribed in a pipe is
/// (§24.10), and it lies in the plane through the ring's centre, which is
/// written down with nothing rounded.
///
/// Sharing an axis is the two questions [`where_a_cone_and_a_pipe_cross`] asks,
/// with the ring's centre standing in for the apex.
///
/// # Along the axis but off it, clear of the tube is one of three signs
///
/// Cut both square to the axis. At every height the pipe is a circle of radius
/// `a` whose centre is `δ` from the axis, and a circle round the axis at that
/// height meets it exactly where its radius is from `|δ − a|` to `δ + a`. The
/// tube is circles round the axis too, of every radius from `R − r` to `R + r`
/// over the heights `−r` to `r`. So the two miss exactly where those two
/// ranges do:
///
/// | where the pipe is | it misses where | `k` |
/// |---|---|---|
/// | past the tube | `δ − a > R + r` | `R + r + a` |
/// | in the ring's hole | `δ + a < R − r` | `R − r − a` |
/// | round the whole ring | `a − δ > R + r` | `a − R − r` |
///
/// Each is `δ` against `k`, so each is a sign of `A − U·k²`, with `U = d·d`
/// for the pipe's direction `d` and `A = ‖(c − o) × d‖² = U·δ²` the question
/// that asked whether the pipe is on the axis: above zero past the tube, and
/// below zero in the hole or round the ring once `k` is above zero. The pin
/// of a fuse standing beside an O-ring is the first (row Q1975). A pipe
/// resting on the tube at one point is a zero, and stays refused with every
/// other pipe off the axis.
///
/// # Errors
///
/// [`Error::ACrossingBetweenATorusAndAnotherCurvedSurface`] where the two do
/// not share an axis, but for a pipe along it proved clear of the tube.
/// [`Error::Undecidable`] where a sign could not be settled, which takes a
/// product leaving `f64`'s range.
fn the_heights_a_ring_crosses_a_pipe_at(
    ring: &Torus,
    pipe: &Cylinder,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SmallVec<[Interval; 4]>> {
    let centre = ring.centre().coordinates();
    let along = ring.axis().direction().components();
    let origin = pipe.axis().origin().coordinates();
    let pipe_way = pipe.axis().direction().components();
    let at = [centre, origin].map(PredicatePoint::from);

    let one_direction =
        squared_length_of_a_cross_product(WHERE_A_TORUS_AND_A_PIPE_CROSS, along, pipe_way);
    let centre_on_the_axis = Question {
        named: WHERE_A_TORUS_AND_A_PIPE_CROSS,
        enclosing: || how_far_off_an_axis_enclosing(centre, origin, pipe_way),
        exactly: || how_far_off_an_axis_exactly(centre, origin, pipe_way),
    };
    let room = Question {
        named: WHERE_A_TORUS_AND_A_PIPE_CROSS,
        enclosing: || how_much_of_a_tube_a_pipe_on_its_axis_leaves_enclosing(ring, pipe),
        exactly: || how_much_of_a_tube_a_pipe_on_its_axis_leaves_exactly(ring, pipe),
    };
    // Two directions crossed is dimensionless, and filed as a length for the
    // reason `where_a_plane_and_a_pipe_cross` gives. A pipe off the axis is
    // answered only where three signs prove it clear of the tube, so no answer
    // here rests on a sign it was not given.
    if ask(&one_direction, Measures::Length, &at, policy, log)? != Decision::Zero {
        return Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface);
    }
    let off_the_axis = ask(&centre_on_the_axis, Measures::Area, &at, policy, log)?;
    if off_the_axis == Decision::Pos {
        return if a_pipe_along_a_rings_axis_misses_its_tube(ring, pipe, &at, policy, log)? {
            Ok(SmallVec::new())
        } else {
            Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface)
        };
    }
    assert_eq!(
        off_the_axis,
        Decision::Zero,
        "a sum of squares is never below zero"
    );
    let shape = ask(&room, Measures::Area, &at, policy, log)?;
    if shape == Decision::Neg {
        return Ok(SmallVec::new());
    }
    if shape == Decision::Zero {
        return Ok(smallvec![Interval::exactly(0.0)]);
    }
    assert_eq!(shape, Decision::Pos, "a sign question answers with a sign");
    let room = how_much_of_a_tube_a_pipe_on_its_axis_leaves_exactly(ring, pipe).map_or_else(
        || how_much_of_a_tube_a_pipe_on_its_axis_leaves_enclosing(ring, pipe),
        |room| room.enclosure(),
    );
    let up = root_enclosing(room * squared_length_enclosing(exactly(along)));
    Ok(smallvec![-up, up])
}

/// Whether a pipe running along a ring's axis, off it, is proved clear of the
/// tube: past it, in its hole, or round the whole of it. The proof is on
/// [`the_heights_a_ring_crosses_a_pipe_at`]; a pipe resting on the tube at one
/// point is a zero there, and answers `false`.
///
/// # Errors
///
/// [`Error::Undecidable`] where a sign could not be settled, which takes a
/// product leaving `f64`'s range.
fn a_pipe_along_a_rings_axis_misses_its_tube(
    ring: &Torus,
    pipe: &Cylinder,
    at: &[PredicatePoint],
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<bool> {
    let (core, tube, skin) = (ring.ring_radius(), ring.tube_radius(), pipe.radius());
    let past_the_tube = the_axes_apart_against_a_gap(ring, pipe, [core, tube, skin]);
    if ask(&past_the_tube, Measures::Area, at, policy, log)? == Decision::Pos {
        return Ok(true);
    }
    for inside in [[core, -tube, -skin], [skin, -core, -tube]] {
        let gap = Question {
            named: WHERE_A_TORUS_AND_A_PIPE_CROSS,
            enclosing: || the_sum_enclosing(inside),
            exactly: || Some(the_sum_exactly(inside)),
        };
        if ask(&gap, Measures::Length, at, policy, log)? == Decision::Pos
            && ask(
                &the_axes_apart_against_a_gap(ring, pipe, inside),
                Measures::Area,
                at,
                policy,
                log,
            )? == Decision::Neg
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `A − U·k²`, with `A = ‖(c − o) × d‖²` for the ring's centre `c` and the
/// pipe's origin `o` and direction `d`, `U = d·d`, and `k` the sum of `gap`:
/// the sign of `δ² − k²` for `δ` how far the pipe's axis is from the centre.
fn the_axes_apart_against_a_gap(
    ring: &Torus,
    pipe: &Cylinder,
    gap: [f64; 3],
) -> Question<impl Fn() -> Interval, impl Fn() -> Option<Exact>> {
    let centre = ring.centre().coordinates();
    let origin = pipe.axis().origin().coordinates();
    let along = pipe.axis().direction().components();
    Question {
        named: WHERE_A_TORUS_AND_A_PIPE_CROSS,
        enclosing: move || {
            let k = the_sum_enclosing(gap);
            how_far_off_an_axis_enclosing(centre, origin, along)
                - k * k * squared_length_enclosing(exactly(along))
        },
        exactly: move || {
            let k = the_sum_exactly(gap);
            Some(
                how_far_off_an_axis_exactly(centre, origin, along)?
                    .minus(&k.times(&k)?.times(&Exact::dot_of(along, along)?)?),
            )
        },
    }
}

fn the_sum_enclosing([one, two, three]: [f64; 3]) -> Interval {
    Interval::exactly(one) + Interval::exactly(two) + Interval::exactly(three)
}

fn the_sum_exactly(terms: [f64; 3]) -> Exact {
    terms
        .iter()
        .fold(Exact::of(0.0), |sum, &term| sum.plus(&Exact::of(term)))
}

/// `D = r² − (a − R)²` as an enclosure.
fn how_much_of_a_tube_a_pipe_on_its_axis_leaves_enclosing(
    ring: &Torus,
    pipe: &Cylinder,
) -> Interval {
    let off_the_middle = Interval::exactly(pipe.radius()) - Interval::exactly(ring.ring_radius());
    squared_enclosing(ring.tube_radius()) - off_the_middle * off_the_middle
}

/// `D = r² − a² − R² + 2aR`, exactly.
fn how_much_of_a_tube_a_pipe_on_its_axis_leaves_exactly(
    ring: &Torus,
    pipe: &Cylinder,
) -> Option<Exact> {
    let (tube, middle, skin) = (ring.tube_radius(), ring.ring_radius(), pipe.radius());
    let across = Exact::product_of(skin, middle)?;
    Some(
        Exact::product_of(tube, tube)?
            .minus(&Exact::product_of(skin, skin)?)
            .minus(&Exact::product_of(middle, middle)?)
            .plus(&across)
            .plus(&across),
    )
}

/// A ring and a ball: the heights of the circles round the axis where the
/// ball's centre is on it, refused where it is not.
///
/// # On the axis it is two circles in a plane crossing
///
/// Cut both through the axis. The tube is the circle of radius `r` round
/// `(R, 0)` and the ball the circle of radius `B` round `(0, e)`, with `e` how
/// far up the axis the ball's centre sits. Two circles `δ` apart, with
/// `δ² = R² + e²`, cross twice, touch, or miss, on the sign of
///
/// ```text
/// Δ = 4r²δ² − N²,    N = B² − r² − δ²
/// ```
///
/// which is `−(δ² − (B + r)²)(δ² − (B − r)²)`. `e² = ‖b − c‖²` once the
/// centre is proved on the axis, so `Δ` is a polynomial in the stored numbers.
/// The circles sit at `h = (−eN ± R√Δ)/(2δ²)`; read along the stored direction
/// `w` that is `(−gN ± R√(Δ·w·w))/(2δ²)` with `g = (b − c)·w`, so no root of
/// `w·w` is divided out. A touching circle is one curve, the way a ball
/// inscribed in a pipe is (§24.10).
///
/// The tube never reaches the axis, because a [`Torus`]'s tube is narrower
/// than its ring, so every point the two circles share has `ρ > 0` and is a
/// whole circle round the axis.
///
/// # Errors
///
/// [`Error::ACrossingBetweenATorusAndAnotherCurvedSurface`] where the centre is
/// not on the axis. [`Error::Undecidable`] where a sign could not be settled,
/// which takes a product leaving `f64`'s range.
fn the_heights_a_ring_crosses_a_ball_at(
    ring: &Torus,
    ball: &Sphere,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SmallVec<[Interval; 4]>> {
    let centre = ring.centre().coordinates();
    let along = ring.axis().direction().components();
    let middle = ball.centre().coordinates();
    let at = [centre, middle].map(PredicatePoint::from);

    let on_the_axis = Question {
        named: WHERE_A_TORUS_AND_A_BALL_CROSS,
        enclosing: || how_far_off_an_axis_enclosing(middle, centre, along),
        exactly: || how_far_off_an_axis_exactly(middle, centre, along),
    };
    let crossing = Question {
        named: WHERE_A_TORUS_AND_A_BALL_CROSS,
        enclosing: || how_a_ball_on_a_rings_axis_crosses_its_tube_enclosing(ring, ball),
        exactly: || how_a_ball_on_a_rings_axis_crosses_its_tube_exactly(ring, ball),
    };
    if ask(&on_the_axis, Measures::Area, &at, policy, log)? != Decision::Zero {
        return Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface);
    }
    let shape = ask(
        &crossing,
        Measures::ManyLengths { how_many: 4 },
        &at,
        policy,
        log,
    )?;
    if shape == Decision::Neg {
        return Ok(SmallVec::new());
    }

    let (apart, past) = a_ball_on_a_rings_axis_exactly(ring, ball).map_or_else(
        || a_ball_on_a_rings_axis_enclosing(ring, ball),
        |(apart, past)| (apart.enclosure(), past.enclosure()),
    );
    let up = dot_enclosing(gap_enclosing(centre, middle), exactly(along));
    let halfway = quotient_or_everything(-(up * past), Interval::exactly(2.0) * apart);
    if shape == Decision::Zero {
        return Ok(smallvec![halfway]);
    }
    assert_eq!(shape, Decision::Pos, "a sign question answers with a sign");
    let crossing = how_a_ball_on_a_rings_axis_crosses_its_tube_exactly(ring, ball).map_or_else(
        || how_a_ball_on_a_rings_axis_crosses_its_tube_enclosing(ring, ball),
        |crossing| crossing.enclosure(),
    );
    let either_way = quotient_or_everything(
        Interval::exactly(ring.ring_radius())
            * root_enclosing(crossing * squared_length_enclosing(exactly(along))),
        Interval::exactly(2.0) * apart,
    );
    Ok(smallvec![halfway - either_way, halfway + either_way])
}

/// `Δ = 4r²δ² − N²` as an enclosure.
fn how_a_ball_on_a_rings_axis_crosses_its_tube_enclosing(ring: &Torus, ball: &Sphere) -> Interval {
    let (apart, past) = a_ball_on_a_rings_axis_enclosing(ring, ball);
    Interval::exactly(4.0) * squared_enclosing(ring.tube_radius()) * apart - past * past
}

/// `Δ = 4r²δ² − N²`, exactly.
fn how_a_ball_on_a_rings_axis_crosses_its_tube_exactly(
    ring: &Torus,
    ball: &Sphere,
) -> Option<Exact> {
    let (apart, past) = a_ball_on_a_rings_axis_exactly(ring, ball)?;
    let tube = ring.tube_radius();
    Some(
        Exact::product_of(4.0, tube)?
            .times(&Exact::of(tube))?
            .times(&apart)?
            .minus(&past.times(&past)?),
    )
}

/// `(δ², N)` as enclosures: `δ² = R² + ‖b − c‖²` and `N = B² − r² − δ²`.
fn a_ball_on_a_rings_axis_enclosing(ring: &Torus, ball: &Sphere) -> (Interval, Interval) {
    let apart = squared_enclosing(ring.ring_radius())
        + squared_length_enclosing(gap_enclosing(
            ring.centre().coordinates(),
            ball.centre().coordinates(),
        ));
    let past = squared_enclosing(ball.radius()) - squared_enclosing(ring.tube_radius()) - apart;
    (apart, past)
}

/// `(δ², N)`, exactly.
fn a_ball_on_a_rings_axis_exactly(ring: &Torus, ball: &Sphere) -> Option<(Exact, Exact)> {
    let apart = Exact::product_of(ring.ring_radius(), ring.ring_radius())?.plus(
        &squared_gap_exactly(ring.centre().coordinates(), ball.centre().coordinates())?,
    );
    let past = Exact::product_of(ball.radius(), ball.radius())?
        .minus(&Exact::product_of(ring.tube_radius(), ring.tube_radius())?)
        .minus(&apart);
    Some((apart, past))
}

/// A ring and a cone: the heights of the circles round the axis where the two
/// share it, refused where they do not.
///
/// # On one axis it is two lines against a circle
///
/// Cut both through the axis. The tube is the circle of radius `r` round
/// `(R, 0)`, and the cone, with its apex `e` up the axis and slope `s`, is the
/// two rays `ρ = s(h − e)` above the apex and `ρ = s(e − h)` below it. Each ray
/// is one nappe, and each crosses the circle twice, touches it, or misses it,
/// on the sign of
///
/// ```text
/// Q = (1 + s²)r² − (se ± R)²
/// ```
///
/// with `+` for the nappe above. So a ring and a cone on one axis cross in
/// **up to four** circles, not two: an apex at the ring's centre with a slope
/// steep enough for both rays to pass through the tube gives four. The tube
/// never reaches the axis, so a point on a ray's whole line that is on the
/// tube is on the ray itself and not on its reflection.
///
/// `e` is `g/√(w·w)` with `g = (a − c)·w`, so `Q` is a polynomial with one
/// root in it, `X ∓ 2sR√E` with `X = (1 + s²)r² − s²E − R²` and `E = ‖a − c‖²`.
/// [`how_a_nappe_reaches_a_tube_exactly`] settles its sign without taking the
/// root. On a nappe, read along the stored direction `w`, the circles sit at
/// `(s²g ± sR√(w·w) ± √(Q·w·w))/(1 + s²)`, the first `±` the nappe's.
///
/// A ray touching the tube is one curve, the way a ball inscribed in a pipe is
/// (§24.10). Sharing an axis is the two questions [`where_a_cone_and_a_pipe_cross`]
/// asks.
///
/// # Errors
///
/// [`Error::ACrossingBetweenATorusAndAnotherCurvedSurface`] where the two do
/// not share an axis. [`Error::Undecidable`] where a sign could not be
/// settled, which takes a product leaving `f64`'s range.
fn the_heights_a_ring_crosses_a_cone_at(
    ring: &Torus,
    cone: &Cone,
    policy: &Policy,
    log: &mut ProvenanceLog,
) -> Result<SmallVec<[Interval; 4]>> {
    let centre = ring.centre().coordinates();
    let along = ring.axis().direction().components();
    let apex = cone.apex().coordinates();
    let at = [centre, apex].map(PredicatePoint::from);

    let one_direction = squared_length_of_a_cross_product(
        WHERE_A_TORUS_AND_A_CONE_CROSS,
        along,
        cone.axis().direction().components(),
    );
    let apex_on_the_axis = Question {
        named: WHERE_A_TORUS_AND_A_CONE_CROSS,
        enclosing: || how_far_off_an_axis_enclosing(apex, centre, along),
        exactly: || how_far_off_an_axis_exactly(apex, centre, along),
    };
    // Two directions crossed is dimensionless, and filed as a length for the
    // reason `where_a_plane_and_a_pipe_cross` gives. Off the axis is refused,
    // as in `the_heights_a_ring_crosses_a_pipe_at`.
    let shares_the_axis = ask(&one_direction, Measures::Length, &at, policy, log)?
        == Decision::Zero
        && ask(&apex_on_the_axis, Measures::Area, &at, policy, log)? == Decision::Zero;
    if !shares_the_axis {
        return Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface);
    }

    let squared_direction = squared_length_enclosing(exactly(along));
    let up = dot_enclosing(gap_enclosing(centre, apex), exactly(along));
    let slope = Interval::exactly(cone.slope());
    let widening = Interval::exactly(1.0) + slope * slope;
    let mut sitting_at: SmallVec<[Interval; 4]> = SmallVec::new();
    for nappe in [1.0, -1.0] {
        let reach = Question {
            named: WHERE_A_TORUS_AND_A_CONE_CROSS,
            enclosing: || how_a_nappe_reaches_a_tube_enclosing(ring, cone, nappe),
            exactly: || how_a_nappe_reaches_a_tube_exactly(ring, cone, nappe),
        };
        let shape = ask(&reach, Measures::Area, &at, policy, log)?;
        if shape == Decision::Neg {
            continue;
        }
        let middle = quotient_or_everything(
            slope * slope * up
                + Interval::exactly(nappe)
                    * slope
                    * Interval::exactly(ring.ring_radius())
                    * root_enclosing(squared_direction),
            widening,
        );
        if shape == Decision::Zero {
            sitting_at.push(middle);
            continue;
        }
        assert_eq!(shape, Decision::Pos, "a sign question answers with a sign");
        let reach = how_a_nappe_reaches_a_tube_enclosing(ring, cone, nappe);
        let either_way =
            quotient_or_everything(root_enclosing(reach * squared_direction), widening);
        sitting_at.extend([middle - either_way, middle + either_way]);
    }
    Ok(sitting_at)
}

/// `Q = X − 2·nappe·sRe` as an enclosure, with `e = g/√(w·w)`.
fn how_a_nappe_reaches_a_tube_enclosing(ring: &Torus, cone: &Cone, nappe: f64) -> Interval {
    let centre = ring.centre().coordinates();
    let along = ring.axis().direction().components();
    let room = the_room_a_cone_on_a_rings_axis_leaves_exactly(ring, cone).map_or_else(
        || the_room_a_cone_on_a_rings_axis_leaves_enclosing(ring, cone),
        |room| room.enclosure(),
    );
    let up = quotient_or_everything(
        dot_enclosing(
            gap_enclosing(centre, cone.apex().coordinates()),
            exactly(along),
        ),
        root_enclosing(squared_length_enclosing(exactly(along))),
    );
    room - Interval::exactly(2.0 * nappe)
        * Interval::exactly(cone.slope())
        * Interval::exactly(ring.ring_radius())
        * up
}

/// An exact number with the sign of `Q = X − 2·nappe·sRe`.
///
/// `e` is `±√E`, with the sign of `g`, so `2·nappe·sRe` is `towards·√L` with
/// `L = 4s²R²E` and `towards` the product of the two signs. Where `X` and
/// `towards·√L` pull opposite ways the sign is `X`'s, and where they pull the
/// same way it is the sign of `X² − L` turned by `X`'s, so the root is never
/// taken. `None` where a product leaves `f64`'s range.
fn how_a_nappe_reaches_a_tube_exactly(ring: &Torus, cone: &Cone, nappe: f64) -> Option<Exact> {
    let centre = ring.centre().coordinates();
    let apex = cone.apex().coordinates();
    let room = the_room_a_cone_on_a_rings_axis_leaves_exactly(ring, cone)?;
    let up = Exact::dot_of_two_exact(
        &Exact::gap_between(centre, apex),
        &Exact::of_each(ring.axis().direction().components()),
    )?;
    let (up, across) = (up.sign(), room.sign());
    let a_sign = |sign: Decision| matches!(sign, Decision::Pos | Decision::Neg | Decision::Zero);
    if !a_sign(up) || !a_sign(across) {
        return None;
    }
    if up == Decision::Zero {
        return Some(room);
    }
    let leaning = Exact::product_of(2.0 * cone.slope(), ring.ring_radius())?;
    let leaning = leaning
        .times(&leaning)?
        .times(&squared_gap_exactly(centre, apex)?)?;
    let pulls_down = (up == Decision::Pos) == (nappe > 0.0);
    if pulls_down {
        if across == Decision::Pos {
            Some(room.times(&room)?.minus(&leaning))
        } else {
            Some(room.minus(&leaning))
        }
    } else if across == Decision::Neg {
        Some(leaning.minus(&room.times(&room)?))
    } else {
        Some(room.plus(&leaning))
    }
}

/// `X = (1 + s²)r² − s²E − R²` as an enclosure, with `E = ‖a − c‖²`.
fn the_room_a_cone_on_a_rings_axis_leaves_enclosing(ring: &Torus, cone: &Cone) -> Interval {
    let squared_slope = squared_enclosing(cone.slope());
    (Interval::exactly(1.0) + squared_slope) * squared_enclosing(ring.tube_radius())
        - squared_slope
            * squared_length_enclosing(gap_enclosing(
                ring.centre().coordinates(),
                cone.apex().coordinates(),
            ))
        - squared_enclosing(ring.ring_radius())
}

/// `X = (1 + s²)r² − s²E − R²`, exactly.
fn the_room_a_cone_on_a_rings_axis_leaves_exactly(ring: &Torus, cone: &Cone) -> Option<Exact> {
    let squared_slope = Exact::product_of(cone.slope(), cone.slope())?;
    let squared_tube = Exact::product_of(ring.tube_radius(), ring.tube_radius())?;
    Some(
        squared_tube
            .plus(&squared_slope.times(&squared_tube)?)
            .minus(&squared_slope.times(&squared_gap_exactly(
                ring.centre().coordinates(),
                cone.apex().coordinates(),
            )?)?)
            .minus(&Exact::product_of(ring.ring_radius(), ring.ring_radius())?),
    )
}

/// Circles round a ring's axis, each named by the plane square to the axis
/// that it lies in — [`EdgeRun::RightRoundWhereItCrosses`] with that plane, the
/// run `rcad-step` reads a circle between two different curved faces as.
///
/// `sitting_at` is where each circle sits read along the ring's stored axis
/// direction `w` from its centre, `w·(x − c)` for any point `x` of it.
///
/// # Why a plane
///
/// The partner here is a pipe, a ball or a cone on the ring's axis, and a plane
/// square to that axis crosses each of them in one circle, so the partner and
/// the plane name the circle with nothing picking it. That is the pair
/// `which_two_surfaces_a_run_is_between` reads off such a run between two
/// curved faces, the curved face that sorts first and the plane, and a ring
/// sorts after the other three. `Model::validate` reads the ring's face against
/// the plane since row Q1173 (§243.8).
///
/// # What the plane costs
///
/// **Its offset is a rounding**, where the one that names a plane-and-ball
/// circle is not: a circle's height takes a square root. The offset stored is
/// the `f64` at the middle of the enclosure `w·c + sitting_at`, so the circle
/// the partner and the plane draw is on both exactly and off the ring by that
/// rounding. The enclosure is refused wider than sixteen roundings of the
/// largest number in it, the allowance `Model::validate` gives a corner.
///
/// **Its normal is the ring's stored axis direction, kept as written**, so it
/// crosses that axis to exactly nothing, which is what `Model::validate` and
/// the walk ask before they read a plane as square to a ring's axis.
///
/// **The plane halfway between two neighbours' planes parts the two circles**,
/// proved here: its offset, each stored offset halved and the two added, lies
/// above the lower circle's enclosure and below the upper's. `rcad-topo`'s
/// arrangement picks each circle out of the pair by those halfway planes, the
/// way a ball and a pipe on its axis have their two circles picked by a
/// half-space square to it.
///
/// # Errors
///
/// [`Error::CannotNameWhichPieceOfACrossingThisIs`] where the axis direction
/// would not be kept as written, where an enclosure is wider than that, or
/// where two circles are not provably at two heights parted by the plane
/// halfway between their planes.
fn circles_round_a_rings_axis(ring: &Torus, sitting_at: &[Interval]) -> Result<SurfaceMeet> {
    let axis = ring.axis().direction();
    let level = Plane::from_point_and_normal_as_written(ring.centre(), axis)?;
    if level.normal() != axis {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }
    let through_the_centre = dot_enclosing(
        exactly(axis.components()),
        exactly(ring.centre().coordinates()),
    );
    let mut offsets: SmallVec<[Interval; 4]> = sitting_at
        .iter()
        .map(|reach| through_the_centre + *reach)
        .collect();
    offsets.sort_by(|one, other| one.lo().total_cmp(&other.lo()));

    let largest = ring.ring_radius() + ring.tube_radius();
    let narrow = offsets.iter().all(|offset| {
        offset.lo().is_finite()
            && offset.hi().is_finite()
            && offset.width()
                <= A_CIRCLES_PLANE_MAY_BE_ROUNDED_BY
                    * f64::EPSILON
                    * offset.lo().abs().max(offset.hi().abs()).max(largest)
    });
    let stored: SmallVec<[f64; 4]> = offsets
        .iter()
        .map(|offset| offset.lo() / 2.0 + offset.hi() / 2.0)
        .collect();
    let apart = offsets
        .windows(2)
        .zip(stored.windows(2))
        .all(|(pair, levels)| {
            let halfway = levels[0] / 2.0 + levels[1] / 2.0;
            pair[0].hi() < halfway && halfway < pair[1].lo()
        });
    if !narrow || !apart {
        return Err(Error::CannotNameWhichPieceOfACrossingThisIs);
    }

    let runs = stored
        .iter()
        .map(|&offset| {
            NamedRun::the_whole_of(EdgeRun::RightRoundWhereItCrosses {
                and: Surface::Plane(level.moved_to(offset)?),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(SurfaceMeet::Curves(runs))
}

/// How many roundings of the largest number involved a circle's plane may be
/// known to within: sixteen, the allowance `Model::validate` gives a corner of
/// a curved face.
const A_CIRCLES_PLANE_MAY_BE_ROUNDED_BY: f64 = 16.0;

/// Whether two cones are one point set: one apex, one slope, and their axes
/// along one line, **whichever way each axis points**.
///
/// A cone here is both nappes ([`Cone`]'s "Two nappes and not one"), and its
/// sign `‖x × w‖² − slope²·(x·w)²` is degree two in the stored direction on both
/// terms, so `w` and every multiple of it — `−w` among them, and a `w` a turn
/// left a hair off length one — give the same sign at every point, not only the
/// same zero set. That is the whole of the "if".
///
/// **And nothing else is the same cone**, which is what keeps this from being
/// a guess. The apex is the one point of a cone with no normal, the axis is its
/// one line of symmetry, and a slope above zero is how fast it widens along
/// that line, so two cones that differ in any of the three differ somewhere.
/// A pair that is not the same point set falls through to the refusal by name,
/// as every pair did before §357.
///
/// Nothing here is rounded, so no [`Policy`] is asked: the apex and the slope
/// are compared as stored, and the direction as [`two_directions_lie_along_one_line`]
/// says. A pair a hair apart is two cones.
///
/// **`walk` asks the same question**, of the two faces either side of a closed
/// run, so that the evaluator calls one cone exactly what the namer calls one
/// cone and no pair is one surface here and two there.
pub(crate) fn one_cone_either_way_round(one: &Cone, other: &Cone) -> bool {
    one.apex() == other.apex()
        && one.slope() == other.slope()
        && two_directions_lie_along_one_line(one.axis().direction(), other.axis().direction())
}

/// Whether two tori are one point set: one centre, one pair of radii, and
/// their axes along one line, whichever way each axis points.
///
/// The same argument as [`one_cone_either_way_round`]: [`Torus`]'s sign
/// `(w·w)·(x·x + R² − r²)² − 4R²·‖x × w‖²` is degree two in `w` on both terms.
/// And a torus with its tube narrower than its ring — the only one [`Torus::new`]
/// builds — has one centre, one axis of symmetry, and its widest and narrowest
/// circles round that axis at `R + r` and `R − r`, so no other four numbers name
/// it.
fn one_ring_either_way_round(one: &Torus, other: &Torus) -> bool {
    one.centre() == other.centre()
        && one.ring_radius() == other.ring_radius()
        && one.tube_radius() == other.tube_radius()
        && two_directions_lie_along_one_line(one.axis().direction(), other.axis().direction())
}

/// Whether `u × v` is exactly the zero vector, **decided in whole numbers**.
///
/// [`squared_length_of_a_cross_product`] asks the same thing through the
/// cascade, and its exact level refuses a product whose low half would be
/// subnormal. A unit direction reaches that with two components below about
/// `2⁻⁴⁸⁵`, so `(2⁻⁶⁰⁰, 2⁻⁶⁰⁰, 1)` and its reverse are one line the cascade
/// cannot call one (`a_cone_about_a_direction_the_cascade_cannot_square_is_still_one_cone`).
/// Each component of the cross product is `a·b − c·d`, and asking whether two
/// products of `f64` are equal needs no range at all: each is a whole number
/// below `2¹⁰⁶` times a power of two, and `i128` holds the whole number.
fn two_directions_lie_along_one_line(u: Vector, v: Vector) -> bool {
    let [u, v] = [u, v].map(|direction| direction.components());
    (0..3).all(|axis| {
        let (next, after) = ((axis + 1) % 3, (axis + 2) % 3);
        a_product_in_whole_numbers(u[next], v[after])
            == a_product_in_whole_numbers(u[after], v[next])
    })
}

/// `a·b` as an odd whole number times a power of two, or `(0, 0)`, so that two
/// products are equal exactly when the pairs are.
fn a_product_in_whole_numbers(a: f64, b: f64) -> (i128, i32) {
    let [(m, e), (n, f)] = [a, b].map(a_whole_number_times_a_power_of_two);
    let product = i128::from(m) * i128::from(n);
    if product == 0 {
        return (0, 0);
    }
    let twos = product.trailing_zeros();
    (product >> twos, e + f + twos as i32)
}

/// A finite `value` as `m·2^e` with `|m| < 2⁵³`, read straight out of the bits.
///
/// # Panics
///
/// If `value` is not finite: a surface's axis direction is finite by
/// construction.
fn a_whole_number_times_a_power_of_two(value: f64) -> (i64, i32) {
    assert!(value.is_finite(), "a stored direction is finite: {value}");
    let bits = value.to_bits();
    let stored_exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = (bits & ((1 << 52) - 1)) as i64;
    let (whole, power) = if stored_exponent == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1 << 52), stored_exponent - 1075)
    };
    if value.is_sign_negative() {
        (-whole, power)
    } else {
        (whole, power)
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

    /// Ring 12/5 about `z` and the radius-5 pipe along `y` through `(12, 0, 0)`:
    /// the circle on `y = 0` they touch in, and a loop through `(9, ±12, ±4)`,
    /// which is on both, crossing it at `(12, 0, ±5)`. Each half of the loop is
    /// picked out of the crossing by its pickers, and the circle's other points
    /// by none. A pipe a unit narrower, or a unit further out, is still refused.
    #[test]
    fn a_pipe_of_the_tube_radius_along_a_rings_core_is_a_circle_and_a_loop_in_two_halves() {
        let ring =
            Surface::Torus(Torus::new(Axis::z(), 12.0, 5.0).expect("a ring wider than its tube"));
        let pipe = a_pipe([0.0, 1.0, 0.0], [12.0, 0.0, 0.0], 5.0);
        let policy = Policy::certified();
        for (one, other) in [(&ring, &pipe), (&pipe, &ring)] {
            let SurfaceMeet::Curves(runs) =
                intersect(one, other, &policy, &mut ProvenanceLog::new()).expect("named")
            else {
                panic!("a ring and a pipe along its core cross in curves");
            };
            assert_eq!(runs.len(), 3, "{runs:?}");
            assert_eq!(
                runs[0].run(),
                &EdgeRun::RightRoundWhereItCrosses {
                    and: a_plane([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]),
                },
            );
            let picks = |run: &NamedRun, point: [f64; 3]| {
                let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
                    panic!("a half of the loop is part of where the two cross: {run:?}");
                };
                picked_by.iter().all(|picker| {
                    matches!(
                        picker.side_of_point(PredicatePoint::from(point)),
                        Decision::Pos | Decision::Zero
                    )
                })
            };
            let (ahead, behind) = (&runs[1], &runs[2]);
            for point in [[9.0, 12.0, 4.0], [9.0, 12.0, -4.0]] {
                assert!(picks(ahead, point) && !picks(behind, point), "{point:?}");
            }
            for point in [[9.0, -12.0, 4.0], [9.0, -12.0, -4.0]] {
                assert!(!picks(ahead, point) && picks(behind, point), "{point:?}");
            }
            for point in [[17.0, 0.0, 0.0], [7.0, 0.0, 0.0], [15.0, 0.0, 4.0]] {
                assert!(!picks(ahead, point) && !picks(behind, point), "{point:?}");
            }
            for point in [[12.0, 0.0, 5.0], [12.0, 0.0, -5.0]] {
                assert!(picks(ahead, point) && picks(behind, point), "{point:?}");
            }
        }
        for pipe in [
            a_pipe([0.0, 1.0, 0.0], [12.0, 0.0, 0.0], 4.0),
            a_pipe([0.0, 1.0, 0.0], [13.0, 0.0, 0.0], 5.0),
        ] {
            assert_eq!(
                intersect(&ring, &pipe, &policy, &mut ProvenanceLog::new()),
                Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface)
            );
        }
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

    fn a_cone(along: [f64; 3], apex: [f64; 3], slope: f64) -> Surface {
        Surface::Cone(
            Cone::new(
                Axis::new(Point::from(apex), Vector::from(along)).expect("a direction"),
                slope,
            )
            .expect("a slope"),
        )
    }

    fn a_ring(along: [f64; 3], centre: [f64; 3], ring: f64, tube: f64) -> Surface {
        Surface::Torus(
            Torus::new(
                Axis::new(Point::from(centre), Vector::from(along)).expect("a direction"),
                ring,
                tube,
            )
            .expect("a tube narrower than its ring"),
        )
    }

    /// A cone is both nappes, so the same cone pointed the other way is one
    /// cone, and a ring pointed the other way is one ring (§357). Both were
    /// refused by name while only a pair holding the same numbers was one.
    #[test]
    fn the_same_cone_or_ring_pointed_backwards_is_the_same_surface() {
        for [one, other] in [
            [
                a_cone([1.0, 2.0, 3.0], [4.0, 0.0, -1.0], 0.5),
                a_cone([-3.0, -6.0, -9.0], [4.0, 0.0, -1.0], 0.5),
            ],
            [
                a_cone([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 0.5),
                a_cone([0.0, 0.0, -1.0], [0.0, 0.0, 0.0], 0.5),
            ],
            [
                a_ring([1.0, 2.0, 3.0], [4.0, 0.0, -1.0], 5.0, 1.0),
                a_ring([-3.0, -6.0, -9.0], [4.0, 0.0, -1.0], 5.0, 1.0),
            ],
            [
                a_ring([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 5.0, 1.0),
                a_ring([0.0, 0.0, -1.0], [0.0, 0.0, 0.0], 5.0, 1.0),
            ],
        ] {
            assert_ne!(one, other, "the two spellings hold different numbers");
            assert_eq!(crossing(&one, &other), SurfaceMeet::SameSurface, "{one:?}");
        }
    }

    /// **A near-equal pair is two surfaces.** One ulp off in the apex or
    /// centre, the slope or a radius, or the axis, and the pair turned round is
    /// not one surface. A ring falls through to the refusal by name, as every
    /// pair did before §357. A cone a hair up its own axis crosses the first in
    /// the circle halfway between the two apexes, and one a hair steeper meets
    /// it at their one apex and nowhere else, which are what two cones on one
    /// axis cross in. A hair off the axis on the same apex is still refused, now
    /// as two rulings through the apex whose flat surface the enclosure of its
    /// normal cannot place: `a₁·w₁ + a₂·w₂` leaves a `2⁻⁶⁰` lean under the
    /// rounding of `a₁ − a₂`.
    #[test]
    fn a_cone_or_ring_a_hair_off_its_own_reverse_is_another_surface() {
        let hair = f64::from_bits(1);
        let leaning = [2f64.powi(-60), 0.0, -1.0];
        let cone = a_cone([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 0.5);
        let one_circle = SurfaceMeet::Curves(vec![
            NamedRun::the_whole_of(EdgeRun::RightRoundWhereItsFacesCross)
                .expect("a whole circle needs nothing picking it"),
        ]);
        let at_the_apex = SurfaceMeet::TouchAtOnePoint {
            at: PredicatePoint::from([0.0; 3]),
        };
        for (other, answer) in [
            (
                a_cone([0.0, 0.0, -1.0], [0.0, 0.0, hair], 0.5),
                Ok(one_circle),
            ),
            (
                a_cone([0.0, 0.0, -1.0], [0.0, 0.0, 0.0], 0.5f64.next_up()),
                Ok(at_the_apex),
            ),
            (
                a_cone(leaning, [0.0, 0.0, 0.0], 0.5),
                Err(Error::CannotNameWhichPieceOfACrossingThisIs),
            ),
        ] {
            for [first, second] in [[&cone, &other], [&other, &cone]] {
                assert_eq!(
                    intersect(
                        first,
                        second,
                        &Policy::certified(),
                        &mut ProvenanceLog::new()
                    ),
                    answer,
                    "{other:?}"
                );
            }
        }
        let ring = a_ring([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 5.0, 1.0);
        for other in [
            a_ring([0.0, 0.0, -1.0], [hair, 0.0, 0.0], 5.0, 1.0),
            a_ring([0.0, 0.0, -1.0], [0.0, 0.0, 0.0], 5f64.next_up(), 1.0),
            a_ring([0.0, 0.0, -1.0], [0.0, 0.0, 0.0], 5.0, 1f64.next_down()),
            a_ring(leaning, [0.0, 0.0, 0.0], 5.0, 1.0),
        ] {
            for [first, second] in [[&ring, &other], [&other, &ring]] {
                assert_eq!(
                    intersect(
                        first,
                        second,
                        &Policy::certified(),
                        &mut ProvenanceLog::new()
                    ),
                    Err(Error::ACrossingBetweenATorusAndAnotherCurvedSurface),
                    "{other:?}"
                );
            }
        }
    }

    /// Two unit directions along one line whose cross product the cascade
    /// cannot square: `2⁻⁶⁰⁰ · 2⁻⁶⁰⁰` is below the subnormals, so the exact
    /// level refuses and the enclosure straddles zero. Whole numbers decide it,
    /// and a cone and a ring about that axis are one surface either way round.
    #[test]
    fn a_cone_about_a_direction_the_cascade_cannot_square_is_still_one_cone() {
        let tiny = 2f64.powi(-600);
        let up = [tiny, tiny, 1.0];
        let down = [-tiny, -tiny, -1.0];
        let asked_the_cascade = decide(&squared_length_of_a_cross_product(
            WHERE_TWO_PIPES_CROSS,
            up,
            down,
        ))
        .0;
        assert!(
            matches!(asked_the_cascade, Decision::Undecidable { .. }),
            "the cascade was meant to be unable to tell: {asked_the_cascade:?}"
        );
        for [one, other] in [
            [up, down].map(|along| a_cone(along, [1.0, 2.0, 3.0], 0.5)),
            [up, down].map(|along| a_ring(along, [1.0, 2.0, 3.0], 5.0, 1.0)),
        ] {
            assert_eq!(crossing(&one, &other), SurfaceMeet::SameSurface, "{one:?}");
        }
    }

    /// Whole numbers tell a line at any size, and tell apart what a rounded
    /// cross product calls one line.
    ///
    /// `(1 + 2⁻⁵²)²` is `1 + 2⁻⁵¹ + 2⁻¹⁰⁴`, which rounds to `1 + 2⁻⁵¹`, so the
    /// last pair below has an `f64` cross product of exactly nothing and is not
    /// along one line.
    #[test]
    fn two_directions_along_one_line_are_told_in_whole_numbers_at_any_size() {
        let smallest = f64::from_bits(1);
        for (u, v) in [
            ([1.0, 2.0, 3.0], [-3.0, -6.0, -9.0]),
            ([smallest, smallest, 1.0], [-smallest, -smallest, -1.0]),
            ([0.1, 0.0, 0.0], [-1e300, 0.0, 0.0]),
            (
                [smallest, 3.0 * smallest, 0.0],
                [2f64.powi(1000), 3.0 * 2f64.powi(1000), 0.0],
            ),
        ] {
            assert!(
                two_directions_lie_along_one_line(Vector::from(u), Vector::from(v)),
                "{u:?} and {v:?} lie along one line"
            );
        }
        let just_over_one = 1f64.next_up();
        let rounded_the_same = (
            [just_over_one, just_over_one.next_up(), 0.0],
            [1.0, just_over_one, 0.0],
        );
        assert_eq!(
            Vector::from(rounded_the_same.0).cross(Vector::from(rounded_the_same.1)),
            Vector::new(0.0, 0.0, 0.0),
            "the rounded cross product was meant to call these one line"
        );
        for (u, v) in [
            ([1.0, 2.0, 3.0], [1.0, 2.0, 3f64.next_up()]),
            ([smallest, 0.0, 1.0], [0.0, 0.0, 1.0]),
            rounded_the_same,
        ] {
            assert!(
                !two_directions_lie_along_one_line(Vector::from(u), Vector::from(v)),
                "{u:?} and {v:?} do not lie along one line"
            );
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

    /// A plane a rounding inside a pipe's rim cuts two lines `2⁻⁵³` either
    /// side of their middle, and the half-space through the axis rounds its
    /// offset by `2⁻⁵³ − 2⁻¹⁰⁴`, so it cannot tell them apart: one line is
    /// `2⁻¹⁰⁴` behind it. A flat surface through the axis's own start can,
    /// and names them (Q2505, a bar flush with an L plate's inner side turned
    /// a quarter turn back). They used to be refused
    /// `CannotNameWhichPieceOfACrossingThisIs`.
    #[test]
    fn two_lines_a_rounding_apart_are_told_apart_by_a_flat_surface_through_the_axis() {
        let tiny = f64::EPSILON;
        let flat = Plane::from_point_and_normal_as_written(
            Point::new(2.0, 0.0, 0.0),
            Vector::new(1.0, tiny, 0.0),
        )
        .expect("a normal with a length");
        assert_eq!(
            (flat.normal().components(), flat.offset()),
            ([1.0, tiny, 0.0], 2.0),
            "the plane is stored as written"
        );
        let origin = [1.5 + tiny, -1.0, -1.0];
        let pipe = a_pipe([0.0, 0.0, 1.0], origin, 0.5);
        let half_space = HalfSpace::through_a_point_square_to(
            Point::from(origin),
            Vector::new(0.0, 0.0, 1.0),
            Vector::new(-tiny, 1.0, 0.0),
        )
        .expect("a direction");
        assert_eq!(
            (half_space.normal().components(), half_space.offset()),
            ([-tiny, 1.0, 0.0], -1.0 - 2.0 * tiny),
            "the half-space's offset rounds -1 - 1.5·2⁻⁵² - 2⁻¹⁰⁴ to -1 - 2⁻⁵¹"
        );

        let met = crossing(&Surface::Plane(flat), &pipe);

        let SurfaceMeet::Curves(runs) = &met else {
            panic!("a plane inside a pipe and along it cuts two lines: {met:?}");
        };
        let facings: Vec<[f64; 3]> = runs
            .iter()
            .map(|run| {
                assert!(run.is_the_whole_curve(), "each line is endless");
                let EdgeRun::PartOfWhereItCrosses { and, picked_by } = run.run() else {
                    panic!("one of two lines is a piece of the crossing: {run:?}");
                };
                assert_eq!(and, &pipe);
                let [Picker::InFrontOfAFlatThrough(surface)] = picked_by[..] else {
                    panic!("picked by a flat surface through the axis: {picked_by:?}");
                };
                assert_eq!(surface.through(), Point::from(origin));
                surface.facing().components()
            })
            .collect();
        assert_eq!(facings, [[-tiny, 1.0, 0.0], [tiny, -1.0, 0.0]]);
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

    /// A ball that just fits a pipe still just fits it when the axis is written
    /// along a direction whose stored triple is not a unit one.
    ///
    /// The whole of §118.11 in one pair. Both radii are two and the centre is
    /// exactly on the axis, so the ball rests against the pipe along one circle
    /// and that is the answer whatever `‖w‖` rounded to. The discriminant used to
    /// be `W·R² − r²`, whose sign for equal radii **is** the sign of the stored
    /// direction's own rounding, so every axis but a coordinate one answered
    /// "nothing" or "two circles" — and `Cylinder::side_of` said at the same time
    /// that the ball's equator was exactly on the skin.
    ///
    /// The `assert_ne!` is the demonstration that the pair is a real one: put a
    /// coordinate direction in and the old form answered correctly too. It also
    /// rules out the directions that only look awkward — `(2, 3, 4)` stores as
    /// an exactly unit triple, and a case built on it would have passed before
    /// this change as well. `(1, 1, 1)` was one of the two here until §120's
    /// correctly rounded normalize made it store exactly unit too.
    #[test]
    fn a_ball_that_just_fits_a_pipe_fits_it_down_any_axis() {
        for along in [[1.0, 2.0, 1.0], [1.0, 2.0, 0.0]] {
            let pipe = a_pipe(along, [0.0, 0.0, 0.0], 2.0);
            let Surface::Cylinder(stored) = pipe else {
                panic!("a pipe is a pipe");
            };
            let squared = Vector::from(stored.axis().direction().components()).length_squared();
            assert_ne!(
                squared, 1.0,
                "a direction along {along:?} does not store as a unit triple, which is what \
                 makes this pair worth asking about"
            );

            let met = crossing(&a_ball([0.0, 0.0, 0.0], 2.0), &pipe);
            let run = one_run(&met);
            assert_eq!(run.run(), &EdgeRun::RightRoundWhereItsFacesCross);
        }
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

    /// A ball resting on the far inside of a pipe: the two loops it leaves
    /// meet where the walls touch, and the flat surface through the ball's
    /// centre square to the axis holds that point and each loop on its own
    /// side. A centre one out from the axis with a radius of three reaches
    /// exactly `1 + 3 = 4` across, the pipe's whole width, and touches at
    /// `(-2, 0, 0)`; Viviani's ball of radius 2 against a pipe of radius 1
    /// through `(1, 0, 0)` touches at `(2, 0, 0)`, and its loops reach
    /// `(0, 0, ±2)`.
    #[test]
    fn a_ball_resting_on_the_far_inside_of_a_pipe_leaves_two_loops_that_meet_where_they_touch() {
        let shapes = [
            (
                a_ball([1.0, 0.0, 0.0], 3.0),
                a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0),
                [-2.0, 0.0, 0.0],
                [2.0, 0.0, 2.0],
            ),
            (
                a_ball([0.0, 0.0, 0.0], 2.0),
                a_pipe([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 1.0),
                [2.0, 0.0, 0.0],
                [0.0, 0.0, 2.0],
            ),
        ];
        for (ball, pipe, touching, top) in shapes {
            for (one, other) in [(&ball, &pipe), (&pipe, &ball)] {
                let mut log = ProvenanceLog::new();
                let SurfaceMeet::Curves(runs) =
                    intersect(one, other, &Policy::certified(), &mut log)
                        .expect("two loops that meet are named")
                else {
                    panic!("a ball resting on the far inside of a pipe crosses it in curves");
                };
                assert!(log.is_empty(), "nothing here needs approximating: {log:?}");
                assert_eq!(runs.len(), 2, "two loops: {runs:?}");
                let sides_of = |point: [f64; 3]| -> Vec<Decision> {
                    runs.iter()
                        .map(|run| {
                            let EdgeRun::PartOfWhereItCrosses { and, picked_by } = run.run() else {
                                panic!("one of two loops is a piece of the crossing: {run:?}");
                            };
                            assert_eq!(and, &pipe, "the picked run names the pipe it goes round");
                            let [picker] = picked_by[..] else {
                                panic!("one flat surface picks a loop: {picked_by:?}");
                            };
                            picker.side_of_point(PredicatePoint::from(point))
                        })
                        .collect()
                };
                let bottom = [top[0], top[1], -top[2]];
                assert_eq!(sides_of(touching), [Decision::Zero, Decision::Zero]);
                assert_eq!(sides_of(top), [Decision::Pos, Decision::Neg]);
                assert_eq!(sides_of(bottom), [Decision::Neg, Decision::Pos]);
            }
        }
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

    /// A thin pipe resting on the inside of a fat one: the two loops of
    /// `a_thin_pipe_run_through_a_fat_one_leaves_two_closed_loops` pushed
    /// together until they meet, at the one point where the walls touch.
    ///
    /// The pipe of radius 2 up `z` and the pipe of radius 1 along `x` at
    /// `y = 1` touch at `(0, 2, 0)`. Each loop is picked by one flat surface
    /// through the fat pipe's axis, and the surface passes exactly through the
    /// touching point, so both loops hold it. Away from it the loops reach out
    /// to `(±2, 0, 0)`, one either side.
    #[test]
    fn a_thin_pipe_resting_on_the_inside_of_a_fat_one_leaves_two_loops_that_meet_where_they_touch()
    {
        let thin = a_pipe([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0);
        let fat = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, -2.0], 2.0);
        for (one, other) in [(&thin, &fat), (&fat, &thin)] {
            let mut log = ProvenanceLog::new();
            let SurfaceMeet::Curves(runs) = intersect(one, other, &Policy::certified(), &mut log)
                .expect("two loops that meet are named")
            else {
                panic!("a thin pipe resting inside a fat one crosses it in curves");
            };
            assert!(log.is_empty(), "nothing here needs approximating: {log:?}");
            assert_eq!(runs.len(), 2, "two loops: {runs:?}");
            let sides_of = |point: [f64; 3]| -> Vec<Decision> {
                runs.iter()
                    .map(|run| {
                        let EdgeRun::PartOfWhereItCrosses { and, picked_by } = run.run() else {
                            panic!("one of two loops is a piece of the crossing: {run:?}");
                        };
                        assert_eq!(and, &fat, "the picked run names the pipe it goes round");
                        let [picker] = picked_by[..] else {
                            panic!("one flat surface picks a loop: {picked_by:?}");
                        };
                        picker.side_of_point(PredicatePoint::from(point))
                    })
                    .collect()
            };
            assert_eq!(sides_of([0.0, 2.0, 0.0]), [Decision::Zero, Decision::Zero]);
            assert_eq!(sides_of([2.0, 0.0, 0.0]), [Decision::Pos, Decision::Neg]);
            assert_eq!(sides_of([-2.0, 0.0, 0.0]), [Decision::Neg, Decision::Pos]);
        }
    }

    /// [`a_thin_pipe_resting_on_the_inside_of_a_fat_one_leaves_two_loops_that_meet_where_they_touch`]
    /// with the thin pipe turned 1 about `z`, as a model reads the wall of a
    /// bar placed along `x` from `(−4, 1, 0)` and then turned: the turn leaves
    /// the thin axis leaning `2.2e−16` off square to `z`, and the axes
    /// `3e−16` less than 1 apart, so the loops do not meet. They pass
    /// `2.55e−8` either side of the flat surface through `z` facing
    /// `(cos 1, sin 1, 0)`, rounded.
    ///
    /// That surface's facing is exactly square to `z` and lies exactly in the
    /// flat of the two axes, so it is exactly the way across that tells the
    /// loops apart. Read through roundings its cosine against that way came
    /// out `1 − 8e−16`, and the sine charged for it `4e−8`, more than the
    /// loops are clear of the surface: the pair was refused (Q2710).
    #[test]
    fn a_thin_pipe_resting_a_rounding_inside_a_fat_one_turned_a_radian_leaves_two_loops_a_hair_apart()
     {
        let along_x = Isometry::translation(Vector::new(-4.0, 1.0, 0.0))
            .expect("a finite move")
            .compose(&Isometry::quarter_turns(&Axis::y(), 1).expect("a turn about y"));
        let turned = Isometry::rotation(&Axis::z(), 1.0)
            .expect("a finite turn")
            .compose(&along_x);
        let pipe = Cylinder::new(Axis::z(), 1.0)
            .expect("a radius")
            .transformed(&turned);
        assert_eq!(
            pipe.axis().direction(),
            Vector::new(
                0.5403023058681399,
                0.8414709848078966,
                -2.220446049250313e-16
            ),
            "the turn leaves the thin axis leaning off square to z"
        );
        let thin = Surface::Cylinder(pipe);
        let fat = a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, -2.0], 2.0);
        let SurfaceMeet::Curves(runs) = crossing(&thin, &fat) else {
            panic!("a thin pipe a rounding inside a fat one crosses it in curves");
        };
        assert_eq!(runs.len(), 2, "two loops: {runs:?}");
        let (c, s) = (1f64.cos(), 1f64.sin());
        let sides_of = |point: [f64; 3]| -> Vec<Decision> {
            runs.iter()
                .map(|run| {
                    let EdgeRun::PartOfWhereItCrosses { and, picked_by } = run.run() else {
                        panic!("one of two loops is a piece of the crossing: {run:?}");
                    };
                    assert_eq!(and, &fat, "the picked run names the pipe it goes round");
                    let [picker] = picked_by[..] else {
                        panic!("one flat surface picks a loop: {picked_by:?}");
                    };
                    picker.side_of_point(PredicatePoint::from(point))
                })
                .collect()
        };
        assert_eq!(
            sides_of([2.0 * c, 2.0 * s, 0.0]),
            [Decision::Pos, Decision::Neg]
        );
        assert_eq!(
            sides_of([-2.0 * c, -2.0 * s, 0.0]),
            [Decision::Neg, Decision::Pos]
        );
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
    /// cross each other twice, and they are named as four halves, each held by
    /// one corner of two flat surfaces through where the axes meet.
    ///
    /// Pipes of radius two up `z` and along `x` cross in the planes `x = z` and
    /// `x = −z`. The middle of each half, `(±2, 0, ±2)`, is held by one run's
    /// pickers and no other's, and the two points where the walls touch,
    /// `(0, ±2, 0)`, by all four.
    #[test]
    fn two_leaning_pipes_of_the_same_thickness_whose_axes_meet_cross_in_four_half_ellipses() {
        let SurfaceMeet::Curves(runs) = crossing(
            &a_pipe([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], 2.0),
            &a_pipe([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], 2.0),
        ) else {
            panic!("two equal pipes whose axes meet cross in curves");
        };
        assert_eq!(runs.len(), 4, "two ellipses, each in two halves: {runs:?}");
        let holds = |run: &NamedRun, at: [f64; 3]| {
            let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = run.run() else {
                panic!("a half is picked out of the crossing: {run:?}");
            };
            picked_by.iter().all(|picker| {
                let Picker::InFrontOf(surface) = picker else {
                    panic!("a half is picked by flat surfaces: {picker:?}");
                };
                surface.normal().dot(Vector::from(at)) - surface.offset() >= 0.0
            })
        };
        for middle in [
            [2.0, 0.0, 2.0],
            [-2.0, 0.0, -2.0],
            [2.0, 0.0, -2.0],
            [-2.0, 0.0, 2.0],
        ] {
            let holding = runs.iter().filter(|run| holds(run, middle)).count();
            assert_eq!(holding, 1, "one half holds {middle:?}");
        }
        for touching in [[0.0, 2.0, 0.0], [0.0, -2.0, 0.0]] {
            assert!(
                runs.iter().all(|run| holds(run, touching)),
                "every half reaches {touching:?}, where the walls touch"
            );
        }
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
    /// Both orders reach the same third sign question, `r₂ − r₁ + δ`, whose
    /// enclosure straddles zero, so both go to the exact level. There the answer
    /// turned on `W₁W₂`, which came back as eight entries one way round and nine
    /// the other — the same number, and only the shorter of the two could be
    /// multiplied by what came next. `Exact` is canonical now, so there is one
    /// list of entries and one answer, and since §118.11 the exact arm does not
    /// form `W₁W₂` at all.
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

    /// §184.7's pair, found by a search over pipes off the coordinate
    /// directions and checked in exact rationals from the stored numbers. The
    /// window read the middle through the squared length of the *rounded* step,
    /// `3.2e-17` of itself off the true `A/W`, and came back
    /// `[-1.0538915031078247e-16, -1.0538915031078239e-16]` around a true middle
    /// of `-1.209827636296209e-16`.
    ///
    /// Every half-space the row may try is asked, because any of them can be
    /// the one it proves with. The true middle is decided here the long way, as
    /// `(at_here·(A − W·d) + at_there·(A + W·d)) / 2A` with nothing but stored
    /// numbers in it, and the window has to hold it.
    #[test]
    fn the_window_around_the_middle_of_two_leaning_pipes_holds_the_true_middle() {
        let along = Vector::new(3.0, 5.0, 7.0);
        let one = Cylinder::new(
            Axis::new(
                Point::from([
                    1.889_498_858_277_455_2,
                    1.754_621_790_330_662_7,
                    5.900_896_671_941_921,
                ]),
                along,
            )
            .expect("a direction"),
            8.668_044_147_417_756,
        )
        .expect("a radius");
        let other = Cylinder::new(
            Axis::new(
                Point::from([
                    20.032_907_041_561_735,
                    41.902_862_084_155_37,
                    41.157_496_726_973_64,
                ]),
                along,
            )
            .expect("a direction"),
            3.509_441_474_803_657_5,
        )
        .expect("a radius");

        let here = one.axis().origin();
        let there = other.axis().origin();
        let along = one.axis().direction();
        let towards = the_step_across_to_the_other_axis(&one, &other);
        let anchors = [
            between_the_two_lines(here, towards, one.radius(), other.radius()),
            here,
            there,
        ];
        let normals = picking_normals_to_try(anchors[0], along, along.cross(towards))
            .expect("a direction with a length");

        let squared_gap = how_far_off_an_axis_exactly(
            there.coordinates(),
            here.coordinates(),
            along.components(),
        )
        .expect("small enough to hold exactly");
        let widths = Exact::dot_of(along.components(), along.components())
            .and_then(|squared_direction| {
                let [mine, theirs] = [one.radius(), other.radius()]
                    .map(|radius| Exact::product_of(radius, radius).expect("a normal square"));
                squared_direction.times(&mine.minus(&theirs))
            })
            .expect("small enough to hold exactly");
        let bottom = squared_gap.plus(&squared_gap);

        for anchor in anchors {
            for normal in &normals {
                let picker =
                    HalfSpace::through_a_point_facing(anchor, *normal).expect("a finite offset");
                let window = how_far_in_front_of_the_middle(picker, &one, &other);

                let reading = |point: Point| {
                    Exact::dot_of(picker.normal().components(), point.coordinates())
                        .expect("small enough to hold exactly")
                        .minus(&Exact::of(picker.offset()))
                };
                let top = reading(here)
                    .times(&squared_gap.minus(&widths))
                    .zip(reading(there).times(&squared_gap.plus(&widths)))
                    .map(|(from_here, from_there)| from_here.plus(&from_there))
                    .expect("small enough to hold exactly");
                let [above_low, below_high] = [
                    Exact::of(window.lo())
                        .times(&bottom)
                        .map(|low| top.minus(&low)),
                    Exact::of(window.hi())
                        .times(&bottom)
                        .map(|high| high.minus(&top)),
                ]
                .map(|gap| gap.expect("small enough to hold exactly").sign());

                assert!(
                    matches!(above_low, Decision::Pos | Decision::Zero)
                        && matches!(below_high, Decision::Pos | Decision::Zero),
                    "the window {window:?} for {picker:?} does not hold the true middle"
                );
            }
        }
    }

    /// Two bars of radius 1/4, the first along `x` through `(9, 0, 0)` and the
    /// second turned 122°, 125°, 128°, 129° or 135° about `z` and started 16
    /// back along itself from `(11, 0, 0)`, with the numbers the model stores
    /// for them (Q1967). Worked in exact fractions, the two stored axes meet at
    /// `(x, 0, 0)`, 16 along the second, with `x` either `11` or `11 − 2⁻⁴⁹`,
    /// both of which `f64` holds. The foot of the common perpendicular worked
    /// in `f64` misses that point by a rounding and so is not on both axes, and
    /// every one of these was refused `CannotNameWhichPieceOfACrossingThisIs`.
    #[test]
    fn two_thin_bars_whose_axes_meet_where_f64_holds_the_point_are_named_in_four_halves() {
        let first = a_pipe_as_written([1.0, 0.0, 0.0], [9.0, 0.0, 0.0]);
        for (degrees, through, along) in [
            (
                122,
                [
                    19.478708227731275,
                    -13.568769538502819,
                    3.552713678800501e-15,
                ],
                [
                    -0.5299192642332048,
                    0.8480480961564262,
                    -2.220446049250313e-16,
                ],
            ),
            (
                125,
                [
                    20.177222981616737,
                    -13.106432708623867,
                    -1.7763568394002505e-15,
                ],
                [
                    -0.5735764363510462,
                    0.8191520442889917,
                    1.1102230246251565e-16,
                ],
            ),
            (
                128,
                [
                    20.850583605210534,
                    -12.608172057707556,
                    3.552713678800501e-15,
                ],
                [
                    -0.6156614753256584,
                    0.7880107536067222,
                    -2.220446049250313e-16,
                ],
            ),
            (
                129,
                [21.0691262567974, -12.43433538331154, 3.552713678800501e-15],
                [
                    -0.6293203910498375,
                    0.7771459614569712,
                    -2.220446049250313e-16,
                ],
            ),
            (
                135,
                [
                    22.31370849898476,
                    -11.313708498984763,
                    3.552713678800501e-15,
                ],
                [
                    -0.7071067811865475,
                    0.7071067811865477,
                    -2.220446049250313e-16,
                ],
            ),
        ] {
            let second = a_pipe_as_written(along, through);
            named_in_four_halves_both_ways_round(&first, &second, degrees);
        }
    }

    /// The same two bars turned 80°, 112° or 116°, where the second's stored
    /// direction and start are exactly in `z = 0` (Q1967). Worked in exact
    /// fractions, the two stored axes meet at `(11 ± 2⁻⁵⁰, 0, 0)`, which `f64`
    /// does not hold, so no surface through that point can be written down.
    /// Every one of these was refused `CannotNameWhichPieceOfACrossingThisIs`.
    #[test]
    fn two_thin_bars_whose_axes_meet_where_f64_holds_no_point_are_named_in_four_halves() {
        let first = a_pipe_as_written([1.0, 0.0, 0.0], [9.0, 0.0, 0.0]);
        for (degrees, through, along) in [
            (
                80,
                [8.221629157329115, -15.756924048195328, 0.0],
                [0.17364817766693036, 0.984807753012208, 0.0],
            ),
            (
                112,
                [16.993705494654595, -14.834941673068599, 0.0],
                [-0.3746065934159121, 0.9271838545667874, 0.0],
            ),
            (
                116,
                [18.01393834862524, -14.38070474078667, 0.0],
                [-0.43837114678907746, 0.8987940462991669, 0.0],
            ),
        ] {
            let second = a_pipe_as_written(along, through);
            named_in_four_halves_both_ways_round(&first, &second, degrees);
        }
    }

    /// The same two bars turned 105°, 110°, 111°, 113°, 114° or 119°, where the
    /// second's stored direction is a hair under `z = 0`, by `2⁻⁵²`, and its
    /// start a hair over it, by `2⁻⁴⁸` (Q1967). Worked in exact fractions, the
    /// two stored axes still lie in one flat and meet at `(11 ± 2⁻⁵⁰, 0, 0)`,
    /// which `f64` does not hold, and neither direction crossed with `x`, `y`
    /// or `z` lies in that flat. Every one of these was refused
    /// `CannotNameWhichPieceOfACrossingThisIs`.
    #[test]
    fn two_thin_bars_a_hair_off_one_coordinate_flat_whose_axes_meet_off_f64_are_named_in_four_halves()
     {
        let first = a_pipe_as_written([1.0, 0.0, 0.0], [9.0, 0.0, 0.0]);
        for (degrees, through, along) in [
            (
                105,
                [
                    15.141104721640332,
                    -15.454813220625095,
                    3.552713678800501e-15,
                ],
                [
                    -0.2588190451025208,
                    0.9659258262890684,
                    -2.220446049250313e-16,
                ],
            ),
            (
                110,
                [16.4723222932107, -15.035081932574537, 3.552713678800501e-15],
                [
                    -0.3420201433256687,
                    0.9396926207859085,
                    -2.220446049250313e-16,
                ],
            ),
            (
                111,
                [
                    16.733887192724808,
                    -14.937286823955231,
                    3.552713678800501e-15,
                ],
                [
                    -0.35836794954530043,
                    0.933580426497202,
                    -2.220446049250313e-16,
                ],
            ),
            (
                113,
                [
                    17.25169805582838,
                    -14.728077655239048,
                    3.552713678800501e-15,
                ],
                [
                    -0.3907311284892739,
                    0.9205048534524405,
                    -2.220446049250313e-16,
                ],
            ),
            (
                114,
                [
                    17.507786289212802,
                    -14.616727322281617,
                    3.552713678800501e-15,
                ],
                [
                    -0.4067366430758002,
                    0.9135454576426011,
                    -2.220446049250313e-16,
                ],
            ),
            (
                119,
                [
                    18.756953923941392,
                    -13.993915314230335,
                    3.552713678800501e-15,
                ],
                [
                    -0.48480962024633706,
                    0.874619707139396,
                    -2.220446049250313e-16,
                ],
            ),
        ] {
            let second = a_pipe_as_written(along, through);
            named_in_four_halves_both_ways_round(&first, &second, degrees);
        }
    }

    /// A pipe of radius 1/4 whose axis stores `along` and `through` bit for
    /// bit, as a moved bar's does.
    fn a_pipe_as_written(along: [f64; 3], through: [f64; 3]) -> Surface {
        let axis =
            Axis::as_written(Point::from(through), Vector::from(along)).expect("a direction");
        assert_eq!(axis.direction().components(), along, "stored as written");
        Surface::Cylinder(Cylinder::new(axis, 0.25).expect("a radius"))
    }

    /// `intersect` names the crossing of `one` and `other` in four halves,
    /// each picked by two flat surfaces, whichever is written first.
    fn named_in_four_halves_both_ways_round(one: &Surface, other: &Surface, degrees: u32) {
        for (first, second) in [(one, other), (other, one)] {
            let named = intersect(
                first,
                second,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            let Ok(SurfaceMeet::Curves(halves)) = &named else {
                panic!("{degrees}°: two ellipses crossing: {named:?}");
            };
            assert_eq!(halves.len(), 4, "{degrees}°: {halves:?}");
            for half in halves {
                let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = half.run() else {
                    panic!("{degrees}°: a piece of where the two cross: {half:?}");
                };
                assert!(
                    picked_by.len() == 2
                        && picked_by
                            .iter()
                            .all(|picker| picker.facing_of_a_flat().is_some()),
                    "{degrees}°: two flat pickers: {picked_by:?}"
                );
            }
        }
    }
}
