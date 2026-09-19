//! §150: two pipes that cross in two straight runs, named one way round and
//! refused the other, and the one number that is not the same the two ways
//! round.
//!
//! Both pipes are one wide and run the same way; the second axis sits `1e16`
//! out along that direction, rounded onto the `f64` grid and then nudged two
//! steps along `z`. Written first pipe first the pair is named in two runs;
//! written the other way round it comes back
//! `CannotNameWhichPieceOfACrossingThisIs`.
//!
//! # The pair is symmetric, so what differs is arithmetic
//!
//! Equal radii and a shared direction: swapping the two pipes is the same
//! picture seen from the other axis. Every quantity
//! `two_lines_between_two_pipes` works out is invariant under the swap or flips
//! sign with it, and the sign flips cancel — the step across turns round, the
//! picking normal turns round with it, and their dot is the same number both
//! ways.
//!
//! **One number does not follow that rule: the picking plane's stored offset.**
//! A [`HalfSpace`] holds a normal and one `f64` offset, and
//! `HalfSpace::through_a_point_facing` works that offset out as a plain
//! rounded dot product of the normal with the point it is built through. Asked
//! the answering way round that point is the origin, the dot is exactly zero,
//! and the plane passes through it exactly. Asked the other way round the point
//! is the far axis origin, the dot is two products of about `1.8e15` that
//! cancel to about a quarter — and each of those products rounds by up to half
//! of `0.25`. The plane lands a third of a pipe radius away from the axis it
//! was built through, on the far side of both lines it was meant to separate,
//! and the either-side test in `two_lines_told_apart` refuses.
//!
//! # It is §16.10's gap and not a limit
//!
//! The two lines read `0.4386…` and `−0.1521…` against the picking normal. The
//! plane the picker was *built through* reads `0.2865…`, which is strictly
//! between them, so the answer exists and an offset rounded from the true dot
//! would reach it. The one that got stored reads `0.5`, past both.

#![forbid(unsafe_code)]

use rcad_geom::{
    Axis, Cylinder, EdgeRun, Error, HalfSpace, Picker, Point, Surface, SurfaceMeet, Vector,
    intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// Which way both pipes run, before it is scaled to length one.
const WAY: [f64; 3] = [0.3, 0.5, 0.7];

/// Both pipes are this wide.
const WIDE: f64 = 1.0;

/// How far out along the shared direction the second axis sits.
const REACH: f64 = 1e16;

/// How many `f64` steps along `z` the second axis is nudged.
const NUDGE: i64 = 2;

#[test]
fn the_pair_is_named_in_two_runs_one_way_round_and_refused_the_other() {
    let pair = the_pair(NUDGE);
    assert_eq!(
        how_it_is_named(&pair, 0),
        Ok(2),
        "the first pipe written first names the crossing in two runs"
    );
    assert_eq!(
        how_it_is_named(&pair, 1),
        Err(Error::CannotNameWhichPieceOfACrossingThisIs),
        "and the same pair written the other way round is refused — §150, and \
         the three tests below say why"
    );
}

/// The two axes are `1.7575…` apart against a touching distance of `2`, so this
/// pair crosses properly and with room.
///
/// It is worth saying out loud because the report §150 was opened on called the
/// pair "a hair off touching, `δ ≈ r₁ + r₂`", and a near-tangency being lost is
/// a different bug from the one here.
///
/// The neighbours a single `f64` step along `z` either side are not near
/// neighbours: a step at `7.68e15` is `2`, which is more than a pipe is wide, so
/// one step back still crosses in two runs and one step on misses entirely.
/// Both of them answer the same both ways round.
#[test]
fn the_two_axes_are_well_inside_touching_distance_and_neither_neighbour_is_order_dependent() {
    let apart = to_the_other_axis().length();
    assert!(
        (apart - 1.7575736716416024).abs() < 1e-12,
        "the axes are {apart} apart"
    );
    assert!(apart < WIDE + WIDE, "so the two pipes cross, with room");

    for (nudge, expected) in [(NUDGE - 1, Ok(2)), (NUDGE + 1, Ok(0))] {
        let pair = the_pair(nudge);
        for first in [0, 1] {
            assert_eq!(
                how_it_is_named(&pair, first),
                expected,
                "{nudge} steps along, pipe {first} written first"
            );
        }
    }
}

/// What the answering order picks: a normal that is two of the shared
/// direction's own components with one negated, through the origin, with an
/// offset of exactly nothing.
///
/// Which of the three normals square to the direction gets picked depends only
/// on the direction, which both orders share, and on how a step across leans
/// against it, which is ranked by a magnitude. So the other order picks this
/// same normal turned round, and every reading of it below is the same number.
#[test]
fn the_answering_order_anchors_its_picking_plane_at_the_origin_and_rounds_nothing() {
    let [dx, dy, _] = the_way_both_pipes_run_scaled_to_one().components();
    let picker = the_picking_plane_the_answering_order_used();

    assert_eq!(picker.normal(), Vector::new(dy, -dx, 0.0));
    assert_eq!(picker.offset(), 0.0);
}

/// The refusing order's plane is a whole rounding of its own products out of
/// place.
#[test]
fn the_refusing_order_stores_an_offset_a_rounding_of_1e15_away_from_its_own_axis() {
    let normal = the_picking_plane_the_answering_order_used().normal();
    let far = the_second_axis_origin();
    let stored = HalfSpace::through_a_point_facing(far, -normal)
        .expect("a half-space through the second axis origin")
        .offset();

    assert_eq!(stored, -0.5, "what one rounded dot product came to");
    let wanted = the_dot_taken_carefully(normal.components(), far.coordinates());
    assert_eq!(wanted, 0.2865091987060316, "what the dot really is");

    let out_of_place = wanted + stored;
    assert_eq!(out_of_place, -0.2134908012939684);

    let [nx, ny, _] = normal.components();
    let [px, py, _] = far.coordinates();
    let biggest = (nx * px).abs().max((ny * py).abs());
    assert!(
        out_of_place.abs() <= one_step_at(biggest),
        "{out_of_place} is bigger than the {} the two products of about \
         {biggest} can round by, so the offset is not merely their rounding",
        one_step_at(biggest)
    );
    assert_eq!(
        out_of_place.abs() / normal.length(),
        0.3335636787994107,
        "a third of a pipe radius, on pipes one radius wide"
    );
}

/// The plane the picker was built through does separate the two lines, and the
/// plane that got stored does not.
///
/// This is what makes §150 a gap rather than a limit: the answer exists at this
/// precision, and one rounded dot product is the whole of what could not reach
/// it.
#[test]
fn the_two_lines_straddle_the_axis_the_plane_was_built_through_and_not_the_one_stored() {
    let normal = the_picking_plane_the_answering_order_used().normal();
    let [nearer, further] = where_the_two_lines_read_against(normal);
    let built_through =
        the_dot_taken_carefully(normal.components(), the_second_axis_origin().coordinates());
    let stored = -HalfSpace::through_a_point_facing(the_second_axis_origin(), -normal)
        .expect("a half-space through the second axis origin")
        .offset();

    assert!(
        (nearer - 0.4386133691295552).abs() < 1e-12 && (further + 0.1521041704235236).abs() < 1e-12,
        "the two lines read {nearer} and {further} against the picking normal"
    );
    assert!(
        further < built_through && built_through < nearer,
        "the plane through the second axis reads {built_through}, which has to \
         sit between the two lines for the answer to exist at all"
    );
    assert!(
        nearer < stored,
        "the plane that got stored reads {stored}, past both lines, so both \
         come back on the same side of it"
    );
}

/// The two pipes, with the second axis `REACH` along the shared direction,
/// rounded onto the grid, and then nudged along `z` by `nudge` steps.
fn the_pair(nudge: i64) -> [Surface; 2] {
    let way = the_way_both_pipes_run();
    [Point::origin(), second_axis_origin_nudged(nudge)].map(|origin| {
        Surface::Cylinder(
            Cylinder::new(Axis::new(origin, way).expect("an axis"), WIDE).expect("a pipe"),
        )
    })
}

/// How many straight runs the pair is named in, with `first` written first —
/// none at all for a pair that misses — or what it was refused with.
fn how_it_is_named(pair: &[Surface; 2], first: usize) -> Result<usize, Error> {
    match intersect(
        &pair[first],
        &pair[1 - first],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) {
        Ok(SurfaceMeet::Curves(runs)) => Ok(runs.len()),
        Ok(SurfaceMeet::Nothing) => Ok(0),
        Ok(other) => panic!("named as something else entirely: {other:?}"),
        Err(refused) => Err(refused),
    }
}

/// The half-space the answering order picked the first of its two runs with,
/// read straight out of the answer.
fn the_picking_plane_the_answering_order_used() -> HalfSpace {
    let pair = the_pair(NUDGE);
    let Ok(SurfaceMeet::Curves(runs)) = intersect(
        &pair[0],
        &pair[1],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    ) else {
        panic!("the first pipe written first names the crossing");
    };
    let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = runs[0].run() else {
        panic!("a picked piece of a crossing");
    };
    let [Picker::InFrontOf(picker)] = picked_by[..] else {
        panic!("one flat picker");
    };
    picker
}

/// Where the two lines the pair crosses in read against `normal`, larger first.
///
/// Worked out from the roundings the second axis origin is made of rather than
/// from the origin itself, so nothing here subtracts two numbers near `1e16`.
/// `to_the_other_axis` is the step across between the axes and comes out the
/// same as the one the kernel works out exactly, to the last bit or two; from
/// there the pair is an ordinary cross-section of two unit circles a known
/// distance apart.
fn where_the_two_lines_read_against(normal: Vector) -> [f64; 2] {
    let along = the_way_both_pipes_run_scaled_to_one();
    let sideways = to_the_other_axis()
        .normalized()
        .expect("the axes are not on top of each other");
    let up = along.cross(sideways);

    let apart = to_the_other_axis().length();
    let across = apart / 2.0;
    let out = (WIDE * WIDE - across * across).sqrt();
    let middle = normal.dot(sideways) * across;
    let either_way = normal.dot(up) * out;
    [middle + either_way, middle - either_way]
}

/// The step from the first pipe's axis across to the second's, square to the
/// shared direction.
///
/// The second origin is the direction scaled by `REACH`, rounded, and nudged,
/// so the whole of the step across is those roundings: `mul_add` hands back
/// what each component of the scaling lost, and what is left is small numbers
/// all the way down.
fn to_the_other_axis() -> Vector {
    let along = the_way_both_pipes_run_scaled_to_one();
    let [dx, dy, dz] = along.components();
    let [px, py, pz] = the_second_axis_origin().coordinates();
    let lost = Vector::new(
        -dx.mul_add(REACH, -px),
        -dy.mul_add(REACH, -py),
        -dz.mul_add(REACH, -pz),
    );
    lost - along * (lost.dot(along) / along.length_squared())
}

/// `a · b` over two triples, with each product taken exactly and the big ones
/// added first so that what they cancel down to keeps the small ones.
fn the_dot_taken_carefully(a: [f64; 3], b: [f64; 3]) -> f64 {
    let mut terms = [0.0; 6];
    for axis in 0..3 {
        terms[axis] = a[axis] * b[axis];
        terms[axis + 3] = a[axis].mul_add(b[axis], -terms[axis]);
    }
    let mut total = 0.0;
    let mut lost = 0.0;
    for term in terms {
        let next = total + term;
        lost += if total.abs() >= term.abs() {
            (total - next) + term
        } else {
            (term - next) + total
        };
        total = next;
    }
    total + lost
}

/// The gap between `size` and the next `f64` above it.
fn one_step_at(size: f64) -> f64 {
    f64::from_bits(size.to_bits() + 1) - size
}

fn the_second_axis_origin() -> Point {
    second_axis_origin_nudged(NUDGE)
}

fn second_axis_origin_nudged(nudge: i64) -> Point {
    let [x, y, z] = (the_way_both_pipes_run_scaled_to_one() * REACH).components();
    Point::new(x, y, f64::from_bits(z.to_bits().wrapping_add_signed(nudge)))
}

fn the_way_both_pipes_run() -> Vector {
    Vector::new(WAY[0], WAY[1], WAY[2])
}

fn the_way_both_pipes_run_scaled_to_one() -> Vector {
    Axis::new(Point::origin(), the_way_both_pipes_run())
        .expect("an axis")
        .direction()
}
