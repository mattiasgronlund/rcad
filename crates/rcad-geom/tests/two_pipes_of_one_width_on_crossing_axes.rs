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
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};

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
/// rounds unless the direction is along a coordinate axis. Since Q1625 a surface
/// that cannot face its own axis faces a direction with whole-number components
/// from −2 to 2 instead, which dots with this middle to a short sum and so
/// exactly. It has to lie exactly in the flat the two axes span, and the flat
/// holds one for two axes in one coordinate plane and for two directions scaled
/// down from one whole-number triple, like `(1, 2, 2)/3` and `(2, 1, 2)/3`. That
/// took the count from 32 to 408 of the 1224. Since Q1645 a facing that still
/// fails is the stored direction slid along a whole-number way in the flat, a
/// unit in the last place at a time and at most 64 of them, until it dots with
/// the middle exactly, which took the count from 408 to 688, and after that a
/// sum of up to four of its own direction and fewer of the other's, which took
/// it to 704. Of the 233 surfaces it still finds nothing for, 180 lie in a flat
/// that holds no whole-number way from −2 to 2;
/// `no_flat_surface_written_in_f64_picks_the_halves_of_bores_on_3_4_0_and_4_0_3`
/// shows for one of those that no flat surface `f64` holds would do.
///
/// Every point of every piece drawn is on both pipes, so a half the namer names
/// is a piece of this crossing and not merely something the walker returned.
#[test]
fn the_namer_names_this_family_in_four_halves_and_the_walker_draws_every_piece() {
    let [named_in_four_halves, pairs_put_to_both] =
        how_many_pairs_are_named_in_four_halves_and_drawn(Point::new(11.0, -3.25, 0.5));
    assert_eq!(
        named_in_four_halves, 752,
        "the pairs with a facing each, whole-number, slid a few units in the last place or \
         mostly its own, that leans its own way and passes exactly through the middle are named \
         of {pairs_put_to_both}"
    );
}

#[test]
fn the_namer_names_this_family_crossing_at_a_middle_in_decimal_millimetres() {
    let [named_in_four_halves, pairs_put_to_both] =
        how_many_pairs_are_named_in_four_halves_and_drawn(Point::new(11.85, -3.15, 0.45));
    assert_eq!(
        named_in_four_halves, 168,
        "the pairs named through a middle in decimal millimetres, of {pairs_put_to_both}"
    );
}

/// The family put to both halves of the kernel through `middle`, and how many
/// of the pairs put the namer names in four halves: every half it names, and
/// both pieces lying in a bisector plane, is drawn, and every point drawn is on
/// both pipes.
fn how_many_pairs_are_named_in_four_halves_and_drawn(middle: Point) -> [usize; 2] {
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
                    )
                    .unwrap_or_else(|why| {
                        panic!("the walker sees the same family and draws {piece:?}: {why:?}")
                    });

                    let allowed = drawn.deviation().magnitude();
                    for point in drawn.points() {
                        for pipe in &pair {
                            let off = how_far_off_the_pipe(pipe, *point);
                            assert!(
                                off <= allowed,
                                "a point of {piece:?} {off} off a pipe of radius {radius} is \
                                 past the run's own bound of {allowed}"
                            );
                        }
                    }
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
    [named_in_four_halves, pairs_put_to_both]
}

/// Bores on `(3, 4, 0)` and `(3, 0, 4)` through `(11, −3.25, 0.5)` are picked
/// by the stored directions slid a few units in the last place along `(0, 1, −1)`.
///
/// Neither stored direction dots exactly with the middle, and the only
/// directions with whole-number components from −2 to 2 in the flat the two
/// span are `(0, 1, −1)` and its multiples. That one reads as far along one
/// axis as back along the other, so its surface is the plane one ellipse lies
/// in rather than a surface between its halves, and until Q1645 the pair was
/// refused. Sliding a stored direction along it keeps the facing in the flat,
/// and each step moves its dot with the middle by `2⁻⁵³·(−3.75)`. Worked out in
/// exact fractions, apart from the kernel: the first stored direction slid by
/// `14·2⁻⁵³` still leans towards its own axis and dots with the middle to
/// exactly `3.999999999999994`, and the second, slid by `8·2⁻⁵³`, to exactly
/// `6.9999999999999964`. No smaller slide does either. So every picker here is
/// one of those two surfaces, facing either way, and both pass exactly through
/// where the axes meet.
#[test]
fn bores_on_3_4_0_and_3_0_4_are_picked_by_their_directions_slid_until_they_pass_exactly() {
    let middle = Point::new(11.0, -3.25, 0.5);
    let [one, other] = two_pipes_crossing_at(
        middle,
        [Vector::new(3.0, 4.0, 0.0), Vector::new(3.0, 0.0, 4.0)],
        0.25,
    );
    let one_unit = f64::EPSILON / 2.0;
    let slid = |pipe: &Cylinder, steps: f64| {
        pipe.axis().direction() + Vector::new(0.0, 1.0, -1.0) * (steps * one_unit)
    };
    let surfaces = [
        (slid(&one, 14.0), 3.999_999_999_999_994),
        (slid(&other, 8.0), 6.999_999_999_999_996_4),
    ];

    for pair in [[one, other], [other, one]] {
        let named = intersect(
            &Surface::Cylinder(pair[0]),
            &Surface::Cylinder(pair[1]),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let Ok(SurfaceMeet::Curves(halves)) = named else {
            panic!("a facing slid until it passes exactly through names the pair: {named:?}")
        };
        assert_eq!(halves.len(), 4, "two ellipses, each in two halves");
        for half in &halves {
            let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = half.run() else {
                panic!("a half is a piece of where the two pipes cross: {half:?}")
            };
            for picker in picked_by {
                let Picker::InFrontOf(surface) = picker else {
                    panic!("a half is picked by flat surfaces: {picker:?}")
                };
                assert!(
                    surfaces.iter().any(|&(facing, offset)| {
                        [(facing, offset), (-facing, -offset)]
                            .contains(&(surface.normal(), surface.offset()))
                    }),
                    "each picker is one of the two slid surfaces: {surface:?}"
                );
            }
        }
    }
}

/// Bores on `(3, −6, 2)` and `(2, 3, 6)` through `(11, −3.25, 0.5)` are
/// picked, on the first one's side, by four times its direction plus the
/// other's: `(2, −3, 2)`.
///
/// The flat of the two holds `(1, 0, 2)`, but no slide of the first direction
/// along it dots with the middle exactly within the 64 steps tried. Four
/// sevenths of `(3, −6, 2)` plus a seventh of `(2, 3, 6)` is `(2, −3, 2)`, and
/// the stored directions, rounded as they are, add up to it with nothing
/// rounded. It leans towards the first as anything with more of it than of the
/// other does, and dots with the middle to a short sum. Since Q1645 such sums
/// are tried last, after the slide, and they name four pairs of directions of
/// the family the count test puts to the namer.
#[test]
fn bores_on_3_minus_6_2_and_2_3_6_are_picked_by_four_of_one_direction_and_one_of_the_other() {
    let middle = Point::new(11.0, -3.25, 0.5);
    let [one, other] = two_pipes_crossing_at(
        middle,
        [Vector::new(3.0, -6.0, 2.0), Vector::new(2.0, 3.0, 6.0)],
        0.25,
    );
    let mostly_one = Vector::new(2.0, -3.0, 2.0);
    assert_eq!(
        one.axis().direction() * 4.0 + other.axis().direction(),
        mostly_one,
        "four of the one direction and one of the other round to nothing but (2, −3, 2)"
    );

    for pair in [[one, other], [other, one]] {
        let named = intersect(
            &Surface::Cylinder(pair[0]),
            &Surface::Cylinder(pair[1]),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let Ok(SurfaceMeet::Curves(halves)) = named else {
            panic!("a facing mostly along one direction names the pair: {named:?}")
        };
        assert_eq!(halves.len(), 4, "two ellipses, each in two halves");
        for half in &halves {
            let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = half.run() else {
                panic!("a half is a piece of where the two pipes cross: {half:?}")
            };
            assert!(
                picked_by.iter().any(|picker| matches!(
                    picker,
                    Picker::InFrontOf(surface)
                        if [mostly_one, -mostly_one].contains(&surface.normal())
                )),
                "one of each half's pickers faces (0.6, 2, 1.6) one way or the other: \
                 {picked_by:?}"
            );
        }
    }
}

/// Bores on `(3, 4, 0)` and `(4, 0, 3)` through `(11, −3.25, 0.5)` are refused,
/// and no flat surface written in `f64` could pick their halves: each one lying
/// in their flat misses where the axes meet.
///
/// A picker has to face a direction `f` lying exactly in the flat of the two
/// stored directions `a` and `b`, `(3, 4, 0)/5` and `(4, 0, 3)/5` rounded, and
/// pass exactly through the middle `c`, so `f·c` has to be a number `f64` holds.
/// Every triple `f64` holds is a power of two times a whole-number triple, so
/// the facings in the flat are powers of two times the whole-number triples in
/// it. `a` and `b` times `2⁵⁵` are whole, `A` and `B`, and when the components of
/// `A × B` share no odd factor every whole-number triple in the flat is
/// `(mA + kB)/2ʲ` for whole `m` and `k`. A component `f64` holds is an odd
/// number below `2⁵³` times a power of two. `b` has no `y`, so the odd part of
/// `m` times the odd part of `A`'s `y` is below `2⁵³`; `a` has no `z`, so the
/// same goes for `k` and `B`'s `z`. Both bounds come out below 3. That leaves
/// `m` a power of two or nothing and `k` one of −1, 0 and 1, up to one power of
/// two over both: the `2ˢa + kb` tried below. `s` from −60 to 60 is more than
/// enough, since outside it the `x` of `2ˢa ± b` spans more than 53 bits.
///
/// Twelve facings are left, and none passes exactly through `c`, whichever way
/// it leans, so neither picker exists.
/// [`HalfSpace::through_a_point_facing`] rounds the exact `f·c` once, so its
/// surface has `c` on it exactly when `f64` holds `f·c`. Naming this pair takes
/// a picker that holds the exact `c` and a facing.
#[test]
fn no_flat_surface_written_in_f64_picks_the_halves_of_bores_on_3_4_0_and_4_0_3() {
    let middle = Point::new(11.0, -3.25, 0.5);
    let [one, other] = two_pipes_crossing_at(
        middle,
        [Vector::new(3.0, 4.0, 0.0), Vector::new(4.0, 0.0, 3.0)],
        0.25,
    );
    for pair in [[one, other], [other, one]] {
        let named = intersect(
            &Surface::Cylinder(pair[0]),
            &Surface::Cylinder(pair[1]),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert!(
            matches!(named, Err(Error::CannotNameWhichPieceOfACrossingThisIs)),
            "no flat surface in f64 picks the halves, so the pair is refused: {named:?}"
        );
    }

    let [a, b] = [one, other].map(|pipe| pipe.axis().direction());
    assert_eq!(
        [a, b].map(|way| way.components()),
        [[0.6, 0.8, 0.0], [0.8, 0.0, 0.6]],
        "the stored directions are the fifths rounded"
    );
    let [whole_a, whole_b] = [a, b].map(|way| {
        way.components().map(|component| {
            let scaled = component * 2f64.powi(55);
            assert_eq!(scaled.fract(), 0.0, "{component} times 2⁵⁵ is whole");
            scaled as i128
        })
    });
    let square = [
        whole_a[1] * whole_b[2] - whole_a[2] * whole_b[1],
        whole_a[2] * whole_b[0] - whole_a[0] * whole_b[2],
        whole_a[0] * whole_b[1] - whole_a[1] * whole_b[0],
    ];
    let shared = square.into_iter().fold(0, |so_far, component| {
        greatest_common_factor(so_far, component)
    });
    assert_eq!(
        odd_part(shared),
        1,
        "A and B reach every whole-number triple in the flat, up to a power of two"
    );
    let below_2_to_the_53 = 1_i128 << 53;
    assert_eq!(
        [whole_a[1], whole_b[2]].map(|digits| below_2_to_the_53 / odd_part(digits)),
        [2, 1],
        "the odd parts of m and k are below 3, so both are one"
    );

    let mut facings = vec![a, -a];
    for s in -60..=60 {
        for m in [0.0, 2f64.powi(s), -(2f64.powi(s))] {
            for k in [1.0, -1.0] {
                let parts = [0, 1, 2].map(|i| (a.components()[i] * m, b.components()[i] * k));
                if parts
                    .iter()
                    .all(|&(of_a, of_b)| it_adds_exactly(of_a, of_b))
                {
                    let facing = Vector::new(
                        parts[0].0 + parts[0].1,
                        parts[1].0 + parts[1].1,
                        parts[2].0 + parts[2].1,
                    );
                    if !facings.contains(&facing) {
                        facings.push(facing);
                    }
                }
            }
        }
    }
    assert_eq!(
        facings.len(),
        12,
        "f64 holds twelve facings in the flat: {facings:?}"
    );
    for facing in facings {
        let surface = HalfSpace::through_a_point_facing(middle, facing).expect("a facing");
        let side = surface.side_of_point(PredicatePoint::Exact(middle.coordinates()));
        assert!(
            matches!(side, Decision::Pos | Decision::Neg),
            "the surface facing {facing:?} misses where the axes meet: {side:?}"
        );
    }
}

/// The largest whole number both `one` and `other` are multiples of.
fn greatest_common_factor(one: i128, other: i128) -> i128 {
    if other == 0 {
        one.abs()
    } else {
        greatest_common_factor(other, one % other)
    }
}

/// `whole` with every factor of two taken out.
fn odd_part(whole: i128) -> i128 {
    whole.abs() >> whole.abs().trailing_zeros()
}

/// Whether `one + other` is a number `f64` holds, so adding them rounds nothing:
/// the error the sum would carry, worked out without rounding, is nothing.
fn it_adds_exactly(one: f64, other: f64) -> bool {
    let sum = one + other;
    let other_as_added = sum - one;
    let one_as_added = sum - other_as_added;
    (one - one_as_added) + (other - other_as_added) == 0.0
}

/// Bores leaning at `(3, 4, 0)` and `(4, 3, 0)` through `(11, −3.25, 0.5)` are
/// picked by the same two flat surfaces as bores on `(1, 0, 0)` and `(0, 1, 0)`
/// through the same point, though their halves are other curves.
///
/// Neither leaning direction dots exactly with that point, so since Q1625 the
/// namer faces `(0, 1, 0)` and `(1, 0, 0)` instead. What this pins is that a
/// picker's facing says nothing about which way its pipe runs: whatever takes
/// the two axis directions off the pickers, rather than off the two pipes, puts
/// the leaning pair's halves where the square pair's are.
#[test]
fn bores_leaning_at_3_4_0_and_4_3_0_are_picked_by_the_facings_bores_crossing_square_are() {
    let middle = Point::new(11.0, -3.25, 0.5);
    let pickers_of = |ways: [Vector; 2]| -> Vec<Vec<Picker>> {
        let [one, other] = two_pipes_crossing_at(middle, ways, 0.25);
        let named = intersect(
            &Surface::Cylinder(one),
            &Surface::Cylinder(other),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let Ok(SurfaceMeet::Curves(halves)) = named else {
            panic!("two bores of one width whose axes meet are named: {named:?}")
        };
        halves
            .iter()
            .map(|half| {
                let EdgeRun::PartOfWhereItCrosses { picked_by, .. } = half.run() else {
                    panic!("a half is a piece of where the two pipes cross: {half:?}")
                };
                picked_by.to_vec()
            })
            .collect()
    };

    assert_eq!(
        pickers_of([Vector::new(3.0, 4.0, 0.0), Vector::new(4.0, 3.0, 0.0)]),
        pickers_of([Vector::new(1.0, 0.0, 0.0), Vector::new(0.0, 1.0, 0.0)]),
        "the leaning pair is picked by the coordinate facings the square pair is"
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
