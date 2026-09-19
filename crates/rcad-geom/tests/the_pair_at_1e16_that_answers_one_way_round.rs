//! §150: two pipes that cross in two straight runs, named one way round and
//! refused the other — and the one rounded number that was the whole of it.
//!
//! Both pipes are one wide and run the same way; the second axis sits `1e16`
//! out along that direction, rounded onto the `f64` grid and then nudged two
//! steps along `z`. Written first pipe first the pair was named in two runs;
//! written the other way round it came back
//! `CannotNameWhichPieceOfACrossingThisIs`. **It is named both ways round now.**
//! The file keeps the name it was opened under, because that is the symptom the
//! pair was found by and what a reader coming from §150 will be looking for.
//!
//! # The pair is symmetric, so what differed was arithmetic
//!
//! Equal radii and a shared direction: swapping the two pipes is the same
//! picture seen from the other axis. Every quantity
//! `two_lines_between_two_pipes` works out is invariant under the swap or flips
//! sign with it, and the sign flips cancel — the step across turns round, the
//! picking normal turns round with it, and their dot is the same number both
//! ways.
//!
//! **One number did not follow that rule: the picking plane's stored offset.**
//! A [`HalfSpace`] holds a normal and one `f64` offset, and
//! `HalfSpace::through_a_point_facing` used to work that offset out as a plain
//! rounded `f64` dot product of the normal with the point it is built through.
//! Asked the answering way round that point is the origin, the dot is exactly
//! zero, and the plane passes through it exactly. Asked the other way round the
//! point is the far axis origin, the dot is two products of about `1.8e15` that
//! cancel to about a quarter — and each of those products rounds by up to half
//! of `0.25`. The plane landed a third of a pipe radius away from the axis it
//! was built through, on the far side of both lines it was meant to separate,
//! and the either-side test in `two_lines_told_apart` refused.
//!
//! # The fix, and what it was worth
//!
//! The offset is rounded from the **exact** dot now, so what it rounds by is set
//! by the size of the answer and not by the size of the coordinates. That is
//! right rather than merely effective here: the true `normal · through` does not
//! change when the point slides along the flat surface, so it is an
//! ordinary-sized number however far out the point handed over sits.
//!
//! Measured below: the plain `f64` dot came to `0.5` against a true
//! `0.2865091987060316`, which is `0.3335…` of a pipe radius out of place on
//! pipes one radius wide. What gets stored now is within one rounding of the
//! answer — under `1e-16` radii, a thousand million million times nearer.
//!
//! # It was §16.10's gap and not a limit
//!
//! The two lines read `0.4386…` and `−0.1521…` against the picking normal. The
//! plane the picker was built through reads `0.2865…`, which is strictly
//! between them, so the answer existed at this precision and an offset rounded
//! from the true dot reaches it. The one that used to get stored read `0.5`,
//! past both.

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
fn the_pair_is_named_in_two_runs_whichever_way_round_it_is_written() {
    let pair = the_pair(NUDGE);
    for first in [0, 1] {
        assert_eq!(
            how_it_is_named(&pair, first),
            Ok(2),
            "pipe {first} written first names the crossing in two runs — it was \
             {:?} this way round before §150's fix",
            Err::<usize, Error>(Error::CannotNameWhichPieceOfACrossingThisIs)
        );
    }
}

/// The two axes are `1.7575…` apart against a touching distance of `2`, so this
/// pair crosses properly and with room.
///
/// It is worth saying out loud because the report §150 was opened on called the
/// pair "a hair off touching, `δ ≈ r₁ + r₂`", and a near-tangency being lost is
/// a different bug from the one here.
///
/// The neighbours a single `f64` step along `z` either side are not near
/// neighbours. One step at `7.68e15` is `1`, and the shared direction leans out
/// of `z`, so a step moves the axes `√(1 − d_z²) ≈ 0.64` apart — near a third of
/// the touching distance of `2`.
#[test]
fn the_two_axes_are_well_inside_touching_distance_and_a_step_either_side_is_no_near_neighbour() {
    let apart = to_the_other_axis_at(NUDGE).length();
    assert!(
        (apart - 1.7575736716416024).abs() < 1e-12,
        "the axes are {apart} apart"
    );
    assert!(apart < WIDE + WIDE, "so the two pipes cross, with room");

    for (nudge, expected) in [
        (0, 0.6135),
        (1, 1.1503),
        (2, 1.7576),
        (3, 2.3821),
        (4, 3.0131),
    ] {
        let apart = to_the_other_axis_at(nudge).length();
        assert!(
            (apart - expected).abs() < 5e-4,
            "{nudge} steps along the axes are {apart} apart, not {expected}"
        );
    }
}

/// One step back still crosses in two runs, one step on misses entirely, and
/// both of them answered the same both ways round even before the fix.
///
/// So the pair §150 is about is not one of a family of order-dependent
/// neighbours: it was the only row of the eighty-five-pair sweep that answered
/// differently depending on the order.
#[test]
fn the_neighbour_a_step_either_side_answers_the_same_both_ways_round() {
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

/// What the picker is: a normal that is two of the shared direction's own
/// components with one negated, through the origin, with an offset of exactly
/// nothing.
///
/// Which of the three normals square to the direction gets picked depends only
/// on the direction, which both orders share, and on how a step across leans
/// against it, which is ranked by a magnitude. So the other order picks this
/// same normal turned round, and every reading of it below is the same number.
///
/// **This is the half of the pair that was always right**, and it is here to say
/// why: the point it is built through is the origin, where a dot product of any
/// three numbers at all is exactly zero and there is nothing left to round.
#[test]
fn a_picking_plane_anchored_at_the_origin_rounds_nothing_and_never_did() {
    let [dx, dy, _] = the_way_both_pipes_run_scaled_to_one().components();
    let picker = the_picking_plane_the_pair_is_named_with();

    assert_eq!(picker.normal(), Vector::new(dy, -dx, 0.0));
    assert_eq!(picker.offset(), 0.0);
}

/// The other half: an offset worked out through a point `1e16` out, rounded from
/// the exact dot rather than from the coordinates.
///
/// Both numbers are here side by side, because the point of §150 is that they
/// are the same expression evaluated two ways and that they differ by a third of
/// a pipe radius.
#[test]
fn the_offset_is_rounded_from_the_answer_now_and_used_to_be_rounded_from_the_coordinates() {
    let normal = the_picking_plane_the_pair_is_named_with().normal();
    let far = the_second_axis_origin();
    let stored = where_the_stored_plane_reads(normal, far);

    let wanted = the_dot_taken_carefully(normal.components(), far.coordinates());
    assert_eq!(wanted, 0.2865091987060316, "what the dot really is");
    let plainly = normal.dot(far - Point::origin());
    assert_eq!(
        plainly, 0.5,
        "what one rounded f64 dot product comes to, and what used to be stored"
    );

    let [nx, ny, _] = normal.components();
    let [px, py, _] = far.coordinates();
    let biggest = (nx * px).abs().max((ny * py).abs());
    assert!(
        (plainly - wanted).abs() <= one_step_at(biggest),
        "{} is bigger than the {} the two products of about {biggest} can round \
         by, so the plain dot's offset is not merely their rounding",
        plainly - wanted,
        one_step_at(biggest)
    );
    assert_eq!(
        (plainly - wanted).abs() / normal.length(),
        0.3335636787994107,
        "a third of a pipe radius, on pipes one radius wide"
    );

    assert!(
        (stored - wanted).abs() <= one_step_at(wanted),
        "the offset that gets stored reads {stored} against a true {wanted}, \
         which is further out than the {} one rounding of the answer allows",
        one_step_at(wanted)
    );
    let out_of_place = (stored - wanted).abs() / normal.length();
    println!("the stored plane is {out_of_place} of a pipe radius out of place");
    assert!(
        out_of_place < 1e-16,
        "{out_of_place} of a radius out of place is not a rounding of an answer \
         of about a quarter"
    );
}

/// The plane that gets stored separates the two lines, and the plane the plain
/// dot would have stored does not.
///
/// This is what made §150 a gap rather than a limit: the answer exists at this
/// precision, and one rounded dot product was the whole of what could not reach
/// it.
#[test]
fn the_two_lines_straddle_the_plane_that_gets_stored_and_not_the_one_the_plain_dot_gave() {
    let normal = the_picking_plane_the_pair_is_named_with().normal();
    let far = the_second_axis_origin();
    let [nearer, further] = where_the_two_lines_read_against(normal);
    let built_through = the_dot_taken_carefully(normal.components(), far.coordinates());
    let stored = where_the_stored_plane_reads(normal, far);

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
        further < stored && stored < nearer,
        "the plane that gets stored reads {stored}, which has to sit between the \
         two lines for the pair to be named this way round"
    );
    assert!(
        nearer < normal.dot(far - Point::origin()),
        "the plain dot would have stored {}, past both lines, so both came back \
         on the same side of it",
        normal.dot(far - Point::origin())
    );
}

/// Where the flat surface a half-space built through `through` actually sits,
/// read against `normal`.
///
/// Built facing the other way and read back negated, because that is the way
/// round `two_lines_between_two_pipes` writes it and the negation rounds
/// nothing.
fn where_the_stored_plane_reads(normal: Vector, through: Point) -> f64 {
    -HalfSpace::through_a_point_facing(through, -normal)
        .expect("a half-space through the second axis origin")
        .offset()
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

/// The half-space the first of the two runs is picked with, read straight out
/// of the answer.
fn the_picking_plane_the_pair_is_named_with() -> HalfSpace {
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
/// `to_the_other_axis_at` is the step across between the axes and comes out the
/// same as the one the kernel works out exactly, to the last bit or two; from
/// there the pair is an ordinary cross-section of two unit circles a known
/// distance apart.
fn where_the_two_lines_read_against(normal: Vector) -> [f64; 2] {
    let along = the_way_both_pipes_run_scaled_to_one();
    let sideways = to_the_other_axis_at(NUDGE)
        .normalized()
        .expect("the axes are not on top of each other");
    let up = along.cross(sideways);

    let apart = to_the_other_axis_at(NUDGE).length();
    let across = apart / 2.0;
    let out = (WIDE * WIDE - across * across).sqrt();
    let middle = normal.dot(sideways) * across;
    let either_way = normal.dot(up) * out;
    [middle + either_way, middle - either_way]
}

/// The step from the first pipe's axis across to the second's at `nudge` steps,
/// square to the shared direction.
///
/// The second origin is the direction scaled by `REACH`, rounded, and nudged,
/// so the whole of the step across is those roundings: `mul_add` hands back
/// what each component of the scaling lost, and what is left is small numbers
/// all the way down.
fn to_the_other_axis_at(nudge: i64) -> Vector {
    let along = the_way_both_pipes_run_scaled_to_one();
    let [dx, dy, dz] = along.components();
    let [px, py, pz] = second_axis_origin_nudged(nudge).coordinates();
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
