//! Two pipes of one width whose axes meet cross in two ellipses.
//!
//! Subtracting the two implicit forms about the point where the axes meet
//! leaves `(p·(û − ê))·(p·(û + ê)) = 0`, so the crossing is the two bisector
//! planes and nothing else — at **every** crossing angle and not only at a
//! right angle. Each plane cuts a pipe in an ellipse `r` across and longer the
//! other way, so the whole crossing is two ellipses that touch each other at
//! the two points where the walls do.
//!
//! # Why the pairs are written as permutations of one direction
//!
//! Both halves of the question are asked **exactly**: the widths match when
//! `r₂² − r₁²` is zero and the axes meet when `(o₂ − o₁)·(w₁ × w₂)` is.
//!
//! The width half used to read the stored directions as well — it was
//! `r₂²‖w₁‖² − r₁²‖w₂‖²`, because the surface the predicate certified was then
//! the pipe of radius `rᵢ/‖wᵢ‖`. [`Axis`] scales every direction to length one,
//! so `‖w‖²` for a stored direction is a sum of three rounded squares and not
//! exactly one: two directions that are not the same three numbers in some
//! order did not have the same `‖w‖²`, and the pair was then not the degenerate
//! family at all. A permutation with signs is the same three magnitudes, so its
//! `‖w‖²` is the same sum of the same rounded squares, bit for bit.
//!
//! §118.11 lifted that: a pipe is its own `r` wide whatever length its
//! direction rounded to, so any two pipes written from one radius literal are
//! now the degenerate family however their axes point. The pairs below stay
//! permutations because that is what a CAD system writes out, stating its bores
//! on coordinate directions — not because the question still needs it.
//!
//! # The halves have to agree about which family this is
//!
//! [`intersect`](rcad_geom::intersect) names crossings and [`walk`] draws them,
//! and for a while only one of them could see this family: the namer refused it
//! by name, as a crossing that touches itself, while the walker read neither
//! the widths nor the gap and handed the pair to the marcher, which gave up
//! with the wrong reason — that the two were too nearly parallel to follow.
//! They were not nearly parallel; they were exactly degenerate. So every pair
//! below is put to both halves and both have to recognise it. Since Q1603 the
//! namer names the family too, as four half ellipses.

use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Error, HalfSpace, NamedRun, Picker, Point, Surface,
    SurfaceMeet, Vector, intersect, walk,
};
use rcad_predicates::{Policy, ProvenanceLog};

/// The three magnitudes each generated pair of directions is built from.
///
/// Each one is a whole-number triple, so the scaling to length one is the same
/// arithmetic whichever order the three are written in.
const DIRECTION_SHAPES: [[f64; 3]; 5] = [
    [3.0, 4.0, 0.0],
    [1.0, 2.0, 2.0],
    [2.0, 3.0, 6.0],
    [1.0, 1.0, 0.0],
    [1.0, 0.0, 0.0],
];

/// Every pair of directions built from one shape that is not along one line.
fn pairs_of_directions() -> Vec<[Vector; 2]> {
    let mut written: Vec<Vector> = Vec::new();
    for shape in DIRECTION_SHAPES {
        for order in [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ] {
            for signs in [[1.0, 1.0, 1.0], [1.0, -1.0, 1.0], [-1.0, 1.0, 1.0]] {
                let turned = Vector::new(
                    shape[order[0]] * signs[0],
                    shape[order[1]] * signs[1],
                    shape[order[2]] * signs[2],
                );
                if !written.contains(&turned) {
                    written.push(turned);
                }
            }
        }
    }

    let mut pairs = Vec::new();
    for shape in DIRECTION_SHAPES {
        let from_this_shape: Vec<Vector> = written
            .iter()
            .copied()
            .filter(|way| {
                let mut magnitudes = [way.x().abs(), way.y().abs(), way.z().abs()];
                magnitudes.sort_by(f64::total_cmp);
                let mut wanted = shape;
                wanted.sort_by(f64::total_cmp);
                magnitudes == wanted
            })
            .collect();
        for (index, mine) in from_this_shape.iter().enumerate() {
            for theirs in &from_this_shape[index + 1..] {
                if mine.cross(*theirs) != Vector::zero() {
                    pairs.push([*mine, *theirs]);
                }
            }
        }
    }
    pairs
}

/// Two pipes of radius `radius` whose axes both run through `middle`.
fn two_pipes_crossing_at(middle: Point, ways: [Vector; 2], radius: f64) -> [Cylinder; 2] {
    ways.map(|way| {
        Cylinder::new(
            Axis::new(middle, way).expect("a direction with some length"),
            radius,
        )
        .expect("a radius above zero")
    })
}

/// How far `point` sits off a pipe, as a plain distance.
///
/// A **cross product** and not `√(‖p‖² − (p·ŵ)²)`, for the reason
/// `walking_the_two_circles_a_cone_and_a_pipe_cut` gives: the second form
/// cancels two large numbers and reads perfect however wrong the point is.
fn how_far_off_the_pipe(pipe: &Cylinder, point: Point) -> f64 {
    let along = pipe.axis().direction();
    ((point - pipe.axis().origin()).cross(along).length() / along.length() - pipe.radius()).abs()
}

/// The run that names the piece of the crossing lying in the plane through
/// `middle` square to `normal`.
///
/// **Two pickers and not one**, both faces of the same plane. Which of the two
/// ellipses the closed form builds first is settled by the sorted pair, so a
/// single half-space would rule the other one out only when the caller happened
/// to write the pipes in the order that put its long axis on the right side of
/// the plane. A piece held by the plane from both sides is a piece lying *in*
/// it, which is what picks one bisector plane out of the two whichever way the
/// pair was written.
fn the_piece_in_the_plane(middle: Point, normal: Vector, and: &Cylinder) -> NamedRun {
    let offset = normal.dot(middle - Point::origin());
    let facing = HalfSpace::new(normal, offset).expect("a normal with some length");
    let backing = HalfSpace::new(-normal, -offset).expect("a normal with some length");
    NamedRun::the_whole_of(EdgeRun::PartOfWhereItCrosses {
        and: Surface::Cylinder(*and),
        picked_by: [Picker::from(facing), Picker::from(backing)]
            .into_iter()
            .collect(),
    })
    .expect("a piece picked by two half-spaces")
}

/// The headline, and the numbers come from the corpus rather than from here:
/// `kicad/Relay_SPST_SANYOU_SRD_Series_Form_A.step` states two bores of radius
/// `0.1` on `+x` and `−y` that meet at `(1.85, 5.85, −3.3)`, and states the
/// edge where they cross as `ELLIPSE('', #27, 0.141421356237, 0.1)`.
#[test]
fn two_bores_of_one_width_crossing_at_a_right_angle_meet_in_the_ellipse_the_file_states() {
    let middle = Point::new(1.85, 5.85, -3.3);
    let [one, other] = two_pipes_crossing_at(
        middle,
        [Vector::new(1.0, 0.0, 0.0), Vector::new(0.0, -1.0, 0.0)],
        0.1,
    );

    let walked = walk(
        &the_piece_in_the_plane(middle, Vector::new(1.0, 1.0, 0.0), &other),
        [&Surface::Cylinder(one), &Surface::Cylinder(other)],
        Deflection::of(1e-7).expect("a deflection above zero"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("two equal bores whose axes meet cross in two ellipses");

    // The walk lands on points along the ellipse and not on its two ends, so
    // the furthest point it reached falls short of the long semi-axis by about
    // what the deflection allows and the nearest overshoots the short one by
    // the same. That is what the room below is, and it is still two orders
    // inside the gap between `0.1414…` and the `0.1` an unstretched circle
    // would have given.
    let points = walked.points();
    assert!(
        points.len() > 8,
        "an ellipse walked finely has points on it"
    );
    let out_to = |start: f64, pick: fn(f64, f64) -> f64| {
        points
            .iter()
            .map(|point| (*point - middle).length())
            .fold(start, pick)
    };

    let reach = out_to(0.0, f64::max);
    assert!(
        (reach - 0.141_421_356_237).abs() < 1e-6,
        "the file states the long way as 0.141421356237 and the walk reached {reach}"
    );
    let closest = out_to(f64::INFINITY, f64::min);
    assert!(
        (closest - 0.1).abs() < 1e-6,
        "the short way is the bore's own radius and the walk came in to {closest}"
    );
}

/// The same headline at an angle that is not square, where the two ellipses are
/// **different lengths** and the one above cannot tell a right answer from a
/// lazy one.
///
/// Every size checked in this file was at a right angle, and a right angle is
/// the one place where `r/|sin(θ/2)|` and `r/|cos(θ/2)|` come out equal. So a
/// closed form that had quietly assumed the equal case would have passed
/// everything here, and `docs/decisions.md` §118.9 named exactly that as the
/// live suspicion about the two Finder relays. It is not what is wrong with
/// them (§118.10), and this is the demonstration.
///
/// `(3, 4, 0)` and `(4, 3, 0)` are the same three magnitudes in another order,
/// which is what keeps the pair inside the family — see the note at the top of
/// this file. They lean at `cos θ = 24/25`, so `tan(θ/2) = 1/7` and the two
/// semi-majors are `r√50` and `r√50/7`: a long thin ellipse and one barely
/// rounder than the bore itself, differing by a factor of seven.
#[test]
fn bores_that_lean_rather_than_cross_square_meet_in_two_ellipses_of_different_lengths() {
    let middle = Point::new(1.85, 5.85, -3.3);
    let radius = 0.1;
    let [one, other] = two_pipes_crossing_at(
        middle,
        [Vector::new(3.0, 4.0, 0.0), Vector::new(4.0, 3.0, 0.0)],
        radius,
    );

    // The ellipse lies in the plane its own bisector spans with the common
    // perpendicular, so the plane that holds it is named by the *other*
    // bisector: the long one about `(1, 1, 0)` is picked out by `(1, -1, 0)`.
    let long_way = radius * 50.0_f64.sqrt();
    for (normal, semi_major) in [
        (Vector::new(1.0, -1.0, 0.0), long_way),
        (Vector::new(1.0, 1.0, 0.0), long_way / 7.0),
    ] {
        let walked = walk(
            &the_piece_in_the_plane(middle, normal, &other),
            [&Surface::Cylinder(one), &Surface::Cylinder(other)],
            Deflection::of(1e-9).expect("a deflection above zero"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("two equal bores leaning on each other cross in two ellipses");

        let points = walked.points();
        let out_to = |start: f64, pick: fn(f64, f64) -> f64| {
            points
                .iter()
                .map(|point| (*point - middle).length())
                .fold(start, pick)
        };

        let reach = out_to(0.0, f64::max);
        assert!(
            (reach - semi_major).abs() < 1e-6,
            "the plane about {normal:?} holds the ellipse {semi_major} long and the walk \
             reached {reach}"
        );
        let closest = out_to(f64::INFINITY, f64::min);
        assert!(
            (closest - radius).abs() < 1e-6,
            "both ellipses are the bore's own radius across and the walk came in to {closest}"
        );
    }
}

/// Every point of every generated pair's crossing is on both pipes.
#[test]
fn every_point_walked_round_one_of_the_two_ellipses_is_on_both_pipes() {
    let middle = Point::new(-2.5, 0.75, 4.0);
    let mut walked_pieces = 0;

    for [mine, theirs] in pairs_of_directions() {
        for radius in [0.1, 1.0, 7.5] {
            let [one, other] = two_pipes_crossing_at(middle, [mine, theirs], radius);
            let [this_way, that_way] =
                [one, other].map(|pipe| pipe.axis().direction().normalized().expect("a direction"));

            for bisector in [that_way + this_way, that_way - this_way] {
                let run = the_piece_in_the_plane(middle, bisector, &other);
                let walked = walk(
                    &run,
                    [&Surface::Cylinder(one), &Surface::Cylinder(other)],
                    Deflection::of(radius * 1e-4).expect("a deflection above zero"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| {
                    panic!("two pipes of one width on axes that meet are drawn: {why:?}")
                });

                let allowed = walked.deviation().magnitude();
                for point in walked.points() {
                    for pipe in [&one, &other] {
                        let off = how_far_off_the_pipe(pipe, *point);
                        assert!(
                            off <= allowed,
                            "a point {off} off a pipe of radius {radius} is past the run's own \
                             bound of {allowed}"
                        );
                    }
                }
                walked_pieces += 1;
            }
        }
    }

    assert!(
        walked_pieces >= 100,
        "the generator has to draw the family and not a handful of it: {walked_pieces} pieces"
    );
}

/// Both halves of the kernel recognise the family, and the walker draws every
/// piece the namer names.
///
/// The namer names the two ellipses as four halves, cut where the two cross each
/// other, each picked by a corner of two flat surfaces through where the axes
/// meet. It used to refuse the family as a crossing that touches itself, until
/// Q1603 named it so; Q1607 gave a fuse a node where two of the halves cross,
/// in `fuse/arrange.rs`. The walker draws
/// each of those halves, and each piece a file names lying in one of the two
/// planes. What must never happen again is the walker not seeing the family at
/// all and marching into the singularity, which came back as `Undecidable`
/// under the marcher's nearly-parallel reason, a reason that was not true of
/// the pair.
///
/// **The namer names a pair only where the two flat surfaces pass exactly
/// through where the axes meet**, and refuses the rest as
/// `CannotNameWhichPieceOfACrossingThisIs`, a gap. A direction scaled to length
/// one is not a whole-number triple any more, so its dot with the middle here
/// rounds unless the direction is along a coordinate axis.
#[test]
fn the_namer_names_this_family_in_four_halves_and_the_walker_draws_every_piece() {
    let middle = Point::new(11.0, -3.25, 0.5);
    let mut pairs_put_to_both = 0;
    let mut named_in_four_halves = 0;

    for [mine, theirs] in pairs_of_directions() {
        for radius in [0.25, 3.0] {
            let [one, other] = two_pipes_crossing_at(middle, [mine, theirs], radius);

            for pair in [[one, other], [other, one]] {
                let named = intersect(
                    &Surface::Cylinder(pair[0]),
                    &Surface::Cylinder(pair[1]),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                );
                let halves = match named {
                    Ok(SurfaceMeet::Curves(halves)) => {
                        assert_eq!(halves.len(), 4, "two ellipses, each in two halves");
                        named_in_four_halves += 1;
                        halves
                    }
                    Err(Error::CannotNameWhichPieceOfACrossingThisIs) => Vec::new(),
                    other => panic!("the namer sees two ellipses crossing: {other:?}"),
                };

                let [this_way, that_way] =
                    pair.map(|pipe| pipe.axis().direction().normalized().expect("a direction"));
                let pieces_in_a_plane = [that_way + this_way, that_way - this_way]
                    .map(|bisector| the_piece_in_the_plane(middle, bisector, &pair[1]));
                for piece in halves.iter().chain(&pieces_in_a_plane) {
                    let drawn = walk(
                        piece,
                        [&Surface::Cylinder(pair[0]), &Surface::Cylinder(pair[1])],
                        Deflection::of(radius * 1e-4).expect("a deflection above zero"),
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    );
                    assert!(
                        drawn.is_ok(),
                        "the walker sees the same family and draws {piece:?}: {drawn:?}"
                    );
                }
                pairs_put_to_both += 1;
            }
        }
    }

    println!("{named_in_four_halves} of {pairs_put_to_both} pairs named in four halves");
    assert!(
        pairs_put_to_both >= 100,
        "the generator has to put the family to both halves and not a handful of it: \
         {pairs_put_to_both} pairs"
    );
    assert!(
        named_in_four_halves > 0,
        "the pairs on coordinate directions are named: {named_in_four_halves} were"
    );
}

/// A pair a rounding away from the family is **not** the family.
///
/// The two questions are asked exactly and this is what that buys: one ulp on
/// one radius and the crossing is a quartic again, drawn by the marcher or
/// refused by it, and never by the closed form. A tolerance in place of the
/// predicate would have drawn two ellipses through a pair that has none.
#[test]
fn one_ulp_on_a_radius_takes_the_pair_out_of_the_family() {
    let middle = Point::origin();
    let ways = [Vector::new(1.0, 0.0, 0.0), Vector::new(0.0, 1.0, 0.0)];
    let one = Cylinder::new(Axis::new(middle, ways[0]).expect("a direction"), 1.0)
        .expect("a radius above zero");
    let other = Cylinder::new(
        Axis::new(middle, ways[1]).expect("a direction"),
        f64::from_bits(1.0f64.to_bits() + 1),
    )
    .expect("a radius above zero");

    let named = intersect(
        &Surface::Cylinder(one),
        &Surface::Cylinder(other),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a pair an ulp off the family is an ordinary crossing");
    assert!(
        matches!(named, SurfaceMeet::Curves(_)),
        "an ulp off the family the two pipes cross in curves: {named:?}"
    );
}
