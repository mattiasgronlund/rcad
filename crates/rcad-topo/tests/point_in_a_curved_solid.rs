//! Inside a bar, inside a ball, or on the skin of one.
//!
//! `point_in_solid.rs` asks the same questions of polyhedra and has since M0.
//! What is new here is that the ray meets a *curved* surface: where it does is a
//! quadratic in one unknown, and whether that place is on the face is
//! [`SolidView::face_holds_the_point`](rcad_topo::SolidView::face_holds_the_point).
//!
//! `docs/decisions.md` §31.7's criterion 10 names the first two tests below and
//! criterion 9 names `whether_a_curved_face_holds_a_point_agrees_with_a_mesh_of_the_same_face`.

use rcad_geom::{
    Axis, Cylinder, Deflection, Isometry, MeetOfThree, Plane, Point, Scale, Sphere, Surface,
    Vector, meet_of_three_surfaces,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, Model, OnAFace, Rule, SolidKey};

const RADIUS: f64 = 3.0;
const HEIGHT: f64 = 5.0;

fn a_round_bar() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar three across and five tall");
    (model, bar)
}

fn a_ball() -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let ball = model
        .add_ball(RADIUS, &Isometry::identity())
        .expect("a ball of radius three");
    (model, ball)
}

/// §31.7's criterion 10, first half.
///
/// A point exactly on a bar's wall, away from both caps and away from the seam.
/// `x² + y² − r²` is `0 + 9 − 9` and cancels to exactly zero, so the surface
/// itself is certified; what the answer then rests on is that the wall's two cap
/// circles put the point between them, which is the whole of what a curved face
/// holding a point comes to here.
#[test]
fn the_middle_of_a_bars_wall_is_on_its_skin() {
    let (model, bar) = a_round_bar();
    let mut log = ProvenanceLog::new();
    let on_the_wall = Point::new(0.0, RADIUS, HEIGHT / 2.0);

    assert_eq!(
        model
            .solid(bar)
            .expect("this model made the key")
            .classify_point(on_the_wall, &Policy::certified(), &mut log),
        Containment::OnBoundary
    );
    assert!(
        log.records().is_empty(),
        "a bar on the z axis needs nothing approximated, got {:?}",
        log.records()
    );
}

/// §31.7's criterion 10, second half.
///
/// One `f64` step inside the skin, which is the tightest a point can be asked
/// about without being on it. The ray meets the sphere twice, both places ahead
/// of the point, and the two crossings leave a winding of one.
#[test]
fn a_point_a_hair_inside_a_balls_skin_is_inside_it() {
    let (model, ball) = a_ball();
    let mut log = ProvenanceLog::new();

    assert_eq!(
        model
            .solid(ball)
            .expect("this model made the key")
            .classify_point(
                Point::new(0.0, 0.0, RADIUS.next_down()),
                &Policy::certified(),
                &mut log
            ),
        Containment::Inside
    );
    assert_eq!(
        model
            .solid(ball)
            .expect("this model made the key")
            .classify_point(
                Point::new(0.0, 0.0, RADIUS.next_up()),
                &Policy::certified(),
                &mut log
            ),
        Containment::Outside
    );
    assert_eq!(
        model
            .solid(ball)
            .expect("this model made the key")
            .classify_point(Point::new(0.0, 0.0, RADIUS), &Policy::certified(), &mut log),
        Containment::OnBoundary,
        "a pole is a corner of the one face, and the face holds it"
    );
    assert!(
        log.records().is_empty(),
        "a ball about the origin needs nothing approximated, got {:?}",
        log.records()
    );
}

/// The bulge is what a box round the corners misses, and reading a point against
/// that box would have called this one outside without firing a ray.
///
/// A bar's two corners are both on its seam at `x = 3, y = 0`, so the box round
/// them is a line. The point below is a whole radius away from it and squarely
/// inside the bar.
#[test]
fn a_point_in_the_bulge_a_bar_makes_away_from_its_corners_is_inside_it() {
    let (model, bar) = a_round_bar();
    let mut log = ProvenanceLog::new();

    assert_eq!(
        model
            .solid(bar)
            .expect("this model made the key")
            .classify_point(
                Point::new(-2.0, 0.0, HEIGHT / 2.0),
                &Policy::certified(),
                &mut log
            ),
        Containment::Inside
    );
    assert_eq!(
        model
            .solid(bar)
            .expect("this model made the key")
            .classify_point(
                Point::new(-4.0, 0.0, HEIGHT / 2.0),
                &Policy::certified(),
                &mut log
            ),
        Containment::Outside
    );
    assert!(log.records().is_empty(), "{:?}", log.records());
}

/// Level with the wall but past the end of it: on the cylinder, off the face.
///
/// This is the one a face's trimming decides and nothing else does. Both points
/// are certified onto the same surface; what tells them apart is which side of
/// the cap planes they are on.
#[test]
fn a_point_on_a_bars_cylinder_past_its_cap_is_outside_the_bar() {
    let (model, bar) = a_round_bar();
    let mut log = ProvenanceLog::new();

    for beyond in [-1.0, HEIGHT + 1.0] {
        assert_eq!(
            model
                .solid(bar)
                .expect("this model made the key")
                .classify_point(
                    Point::new(0.0, RADIUS, beyond),
                    &Policy::certified(),
                    &mut log
                ),
            Containment::Outside,
            "at height {beyond}"
        );
    }
    assert!(log.records().is_empty(), "{:?}", log.records());
}

/// The middle of a cap is on the skin, and the middle of the *plane* it sits on
/// is not.
///
/// A round cap is a flat face whose whole outline is one closed curve, so there
/// is no polygon to wind and the winding count has to read the circle as a
/// circle. Before that rule existed the count was taken over a single corner and
/// came out zero everywhere, which would have put the middle of every cap
/// outside the bar.
#[test]
fn the_middle_of_a_round_cap_is_on_the_skin_and_the_rest_of_its_plane_is_not() {
    let (model, bar) = a_round_bar();
    let mut log = ProvenanceLog::new();

    assert_eq!(
        model
            .solid(bar)
            .expect("this model made the key")
            .classify_point(Point::new(0.5, 0.5, 0.0), &Policy::certified(), &mut log),
        Containment::OnBoundary,
        "inside the bottom cap's circle"
    );
    assert_eq!(
        model
            .solid(bar)
            .expect("this model made the key")
            .classify_point(Point::new(4.0, 4.0, 0.0), &Policy::certified(), &mut log),
        Containment::Outside,
        "on the cap's plane and well outside its circle"
    );
    assert!(log.records().is_empty(), "{:?}", log.records());
}

/// §31.7's criterion 16 in miniature, for the one thing this lane decides: a
/// lattice-aligned curved control is decided everywhere and records nothing.
///
/// A grid rather than a handful, because what is being checked is that nothing
/// **anywhere** falls through to `Undecidable` or to a face nobody can follow.
/// The grid is offset by an eighth so that no sample lands on a cap plane or on
/// the seam, which are the boundary cases the tests above pin one at a time.
#[test]
fn a_grid_of_points_against_a_bar_and_a_ball_is_decided_and_records_nothing() {
    let mut counted = 0;
    for ((model, solid), what) in [(a_round_bar(), "bar"), (a_ball(), "ball")] {
        let mut log = ProvenanceLog::new();
        for i in -8..=8 {
            for j in -8..=8 {
                for k in -8..=8 {
                    let at = Point::new(
                        f64::from(i) / 2.0 + 0.125,
                        f64::from(j) / 2.0 + 0.125,
                        f64::from(k) / 2.0 + 0.125,
                    );
                    let placed = model
                        .solid(solid)
                        .expect("this model made the key")
                        .classify_point(at, &Policy::certified(), &mut log);
                    assert!(
                        matches!(
                            placed,
                            Containment::Inside | Containment::Outside | Containment::OnBoundary
                        ),
                        "the {what} could not place {at:?}: {placed:?}"
                    );
                    counted += 1;
                }
            }
        }
        assert!(
            log.records().is_empty(),
            "the {what} recorded {:?}",
            log.records()
        );
    }
    assert_eq!(counted, 2 * 17 * 17 * 17);
}

/// §31.7's criterion 9: one authority on whether a curved face holds a point,
/// and it agrees with a fine mesh of that same face wherever both can tell.
///
/// **The mesh is the oracle and not the answer.** A mesh of one face is a set of
/// flat pieces standing in for it, everywhere within `Mesh::deviation` of the
/// surface, so a point *on* the surface is within that of the mesh exactly when
/// it is on the face — up to a band round the boundary where the mesh cannot
/// tell either. Samples in that band are skipped, which is what "wherever both
/// can tell" means, and the count that is compared is asserted so that a band
/// swallowing the whole test would fail rather than pass.
#[test]
fn whether_a_curved_face_holds_a_point_agrees_with_a_mesh_of_the_same_face() {
    let fine = Deflection::of(0.02).expect("a fine enough mesh");
    let mut compared = 0;
    let mut held = 0;

    for ((model, solid), what) in [(a_round_bar(), "bar"), (a_ball(), "ball")] {
        let mut log = ProvenanceLog::new();
        let mesh = model
            .solid(solid)
            .expect("this model made the key")
            .mesh(fine, &Policy::certified(), &mut log)
            .expect("a bar and a ball both mesh");
        // What the mesh of a face promises is that every point of the face is
        // within `deviation` of some triangle of it — the distance below is to
        // the nearest *triangle* and not to the nearest corner, so a long thin
        // triangle up the side of a bar costs nothing. Four times that leaves
        // room for the drop being measured in `f64`; ten times it again is the
        // band inside which the mesh is not being asked, and every sample below
        // is either on a face or a whole unit clear of one.
        let near = 4.0 * mesh.deviation().magnitude();
        let far = 10.0 * near;

        for face in model.solid(solid).expect("this model made the key").faces() {
            let surface = face.surface();
            if surface.as_plane().is_some() {
                continue;
            }
            let standing_in: Vec<[Point; 3]> = mesh
                .triangles()
                .iter()
                .filter(|triangle| triangle.face() == face.key())
                .map(|triangle| {
                    triangle
                        .corners()
                        .map(|corner| mesh.points()[corner as usize])
                })
                .collect();

            for sample in samples_over(&surface) {
                let by_mesh = standing_in
                    .iter()
                    .map(|&[a, b, c]| how_far_from_a_triangle(sample, a, b, c))
                    .fold(f64::INFINITY, f64::min);
                let by_the_face = model
                    .solid(solid)
                    .expect("this model made the key")
                    .face_holds_the_point(
                        face.key(),
                        sample.coordinates().into(),
                        &Policy::certified(),
                        &mut log,
                    )
                    .unwrap_or_else(|why| panic!("the {what}'s face at {sample:?}: {why}"));
                if by_mesh <= near {
                    assert_ne!(
                        by_the_face,
                        OnAFace::NotHeld,
                        "the mesh of the {what} covers {sample:?} and the face says it does not"
                    );
                    compared += 1;
                    held += 1;
                } else if by_mesh >= far {
                    assert_eq!(
                        by_the_face,
                        OnAFace::NotHeld,
                        "the mesh of the {what} is {by_mesh} from {sample:?} and the face holds it"
                    );
                    compared += 1;
                }
            }
        }
    }

    // Every sample is compared, and the split is derived rather than eyeballed:
    // sixteen longitudes round the bar, ten heights each, of which the four
    // between the caps are held; sixteen round the ball, fifteen latitudes each,
    // and a ball's one face holds every point on its sphere.
    assert_eq!(
        compared,
        16 * 10 + 16 * 15,
        "every sample is decided by both"
    );
    assert_eq!(held, 16 * 4 + 16 * 15);
}

/// Points on a surface to ask about, spread well past whatever the face covers.
///
/// A pipe is sampled a long way up and down its axis so that the samples run off
/// both caps of any bar drawn on it, and never within a unit of one; a ball has
/// no off, so it is sampled all over and every sample should be held.
fn samples_over(surface: &Surface) -> Vec<Point> {
    let mut points = Vec::new();
    for round in 0..16 {
        let u = f64::from(round) * core::f64::consts::TAU / 16.0;
        match surface {
            Surface::Cylinder(_) => {
                for along in [-3.0, -2.0, -1.0, 1.0, 2.0, 3.0, 4.0, 6.0, 7.0, 8.0] {
                    points.push(surface.point_at(u, along));
                }
            }
            Surface::Plane(_) | Surface::Sphere(_) | _ => {
                for step in -7..=7 {
                    points
                        .push(surface.point_at(u, f64::from(step) * core::f64::consts::PI / 16.0));
                }
            }
        }
    }
    points
}

/// The distance from `at` to the nearest point of the triangle `a b c`.
///
/// The nearest point is inside the triangle or on one of its three sides, so
/// this takes the smaller of the two: the drop onto the plane when that lands
/// inside, and the nearest point of each side otherwise.
fn how_far_from_a_triangle(at: Point, a: Point, b: Point, c: Point) -> f64 {
    let normal = (b - a).cross(c - a);
    let along_sides = [(a, b), (b, c), (c, a)]
        .into_iter()
        .map(|(from, to)| how_far_from_a_side(at, from, to))
        .fold(f64::INFINITY, f64::min);
    if normal.length() == 0.0 {
        return along_sides;
    }
    let inside = [(a, b), (b, c), (c, a)]
        .into_iter()
        .all(|(from, to)| (to - from).cross(at - from).dot(normal) >= 0.0);
    if inside {
        (at - a).dot(normal).abs() / normal.length()
    } else {
        along_sides
    }
}

/// The distance from `at` to the nearest point of the piece from `from` to `to`.
fn how_far_from_a_side(at: Point, from: Point, to: Point) -> f64 {
    let side = to - from;
    let squared = side.dot(side);
    if squared == 0.0 {
        return (at - from).length();
    }
    let part = ((at - from).dot(side) / squared).clamp(0.0, 1.0);
    (at - (from + side * part)).length()
}

/// An eighth of a ball the **fuse** made, rather than one built face by face,
/// and every point asked of it is answered (`docs/decisions.md` §169.4).
///
/// The shape is the same pie-slice-walled wedge
/// `a_quarter_of_a_ball_built_face_by_face.rs` builds by hand, and until this
/// lane the two were not answered alike. A fused arc is not stored with a picker
/// whose plane holds its ends: the fuse lowers that plane until the predicate
/// says both ends are on or in front of it (`the_half_space_both_ends_are_on`),
/// so both ends read `Pos` where a hand-built one reads `Zero`, and §167's step
/// asked for `Zero`. Every flat face here came back
/// `CannotFollowAFacesTrimming` for it.
///
/// **The turn is what makes them round.** Fused on the lattice the two corners
/// of each arc land on whole numbers and the picker's plane holds them after
/// all, which is the control below: it was answered before this lane and is
/// answered now. Turned about an axis along none of the coordinate directions,
/// nothing lands on a representable number and the picker is the lowered one.
///
/// The oracle is the octant: inside the ball and clear of all three walls, read
/// in the frame the shape was turned *from*, and every point is held well clear
/// of all four faces.
#[test]
fn a_fused_eighth_of_a_ball_says_which_points_are_inside_it() {
    let turn = Isometry::rotation(
        &Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("an axis with a length"),
        0.7,
    )
    .expect("a finite angle");
    let asked = [
        [1.0, 1.0, 1.0],
        [2.0, 0.5, 0.5],
        [0.4, 1.8, 0.7],
        [0.5, 0.5, 2.2],
        [-1.0, 1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, -1.0],
        [2.0, 2.0, 2.0],
        [0.5, 0.5, 4.0],
        [-2.0, -2.0, -2.0],
    ];
    let policy = Policy::certified();
    let mut inside = 0;
    let mut outside = 0;
    for placement in [Isometry::identity(), turn] {
        let (model, eighth) = an_eighth_of_a_ball_the_fuse_made(&placement);
        let solid = model.solid(eighth).expect("this model made the key");
        assert_eq!(
            solid.faces().count(),
            4,
            "an eighth of a ball is a sphere patch and three pie slices"
        );
        for [x, y, z] in asked {
            let expected = where_it_is_in_a_fused_eighth([x, y, z]);
            let mut log = ProvenanceLog::new();
            let answer = solid.classify_point(
                placement.apply_to_point(Point::new(x, y, z)),
                &policy,
                &mut log,
            );
            assert_eq!(
                answer,
                expected,
                "a fused eighth placed by {placement:?}, asked about {:?}",
                [x, y, z]
            );
            assert!(
                log.is_empty(),
                "a certified answer about an exact point records nothing"
            );
            match expected {
                Containment::Inside => inside += 1,
                Containment::Outside => outside += 1,
                Containment::OnBoundary
                | Containment::Undecidable { .. }
                | Containment::CannotFollowAFacesTrimming { .. }
                | Containment::CannotFireARayAtAFacesSurface { .. }
                | _ => unreachable!("the oracle only says inside or outside, not {expected:?}"),
            }
        }
    }
    eprintln!("{inside} points inside a fused eighth of a ball and {outside} outside");
    assert!(
        inside >= 4 && outside >= 4,
        "the points should land on both sides often: {inside} inside, {outside} outside"
    );
}

/// The part of a ball of radius [`RADIUS`] about the origin that a block with a
/// corner there keeps: the eighth where every coordinate is positive, placed by
/// `placement`.
fn an_eighth_of_a_ball_the_fuse_made(placement: &Isometry) -> (Model, SolidKey) {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let block = model
        .add_cube(7.0, &policy, &mut ProvenanceLog::new())
        .expect("a whole-number cube");
    model
        .place_solid(block, placement)
        .expect("this model made the key");
    let ball = model.add_ball(RADIUS, placement).expect("a ball");
    let fused = model
        .fuse(
            &[block],
            &[ball],
            Rule::Intersection,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("a block and a ball sharing a corner intersect");
    let [eighth] = *fused.solids() else {
        panic!(
            "a block and a ball meet in one piece, got {:?}",
            fused.solids()
        );
    };
    (model, eighth)
}

/// Inside or outside the eighth of the ball of radius [`RADIUS`] where every
/// coordinate is positive, for a point well clear of all four of its faces.
fn where_it_is_in_a_fused_eighth([x, y, z]: [f64; 3]) -> Containment {
    let clear = 0.2;
    let reach = (x * x + y * y + z * z).sqrt();
    assert!(
        (reach - RADIUS).abs() > clear && [x, y, z].iter().all(|along| along.abs() > clear),
        "{:?} is too near a face of the eighth",
        [x, y, z]
    );
    if reach < RADIUS && x > 0.0 && y > 0.0 && z > 0.0 {
        Containment::Inside
    } else {
        Containment::Outside
    }
}

/// **A ray along a ball's skin that leans to the axis of a pipe through the
/// ball's centre meets the two in four points**, as does the plane parallel
/// to that axis it leaned from, and a plane square to the axis meets them
/// nowhere.
///
/// Route one of §31.10 is a ray *along the surface* from the point,
/// crossing-counted against the wires. A ray along a ball's skin is the ball
/// crossed with a plane through the point; a boundary of a ball with a pipe
/// through it runs along where the ball crosses the pipe; so every crossing
/// that ray has to count is a point of ball, plane and pipe together.
/// `docs/decisions.md` §35 reads this test and the next.
///
/// Because the ball is centred on the pipe's axis, the two share the circles
/// at `x = ±√8`. The plane `x + 16z = 8` through `(0, √8.75, 0.5)` leans to
/// the axis, and cuts each circle in two points: `(±√8, ±√(1 − z²), z)` with
/// `z = (8 ∓ √8)/16`, inside the pipe's radius of one. The plane `z = 0.5`
/// beside the axis cuts them at `(±√8, ±√0.75, 0.5)` (Q1271), and `x = 0`
/// cuts the two in circles about one centre (Q1259). Until Q2188 the leaning
/// plane was refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, and this test
/// pinned that refusal; the road for a plane leaning beside the one axis two
/// round surfaces share takes it now (Q2177), as it takes any plane no nearer
/// square to that axis than to parallel. The refusal is pinned by
/// [`a_ray_along_a_balls_skin_leaning_to_a_pipe_off_its_centre_is_refused_with_four_points_there`].
#[test]
fn a_ray_along_a_balls_skin_leaning_to_a_pipe_through_its_centre_meets_both_in_four_points() {
    let ball = Surface::Sphere(Sphere::new(Point::origin(), RADIUS).expect("a ball"));
    let pipe = Surface::Cylinder(
        Cylinder::new(Axis::x(), 1.0).expect("a pipe of radius one along the x axis"),
    );
    let along = 8.0_f64.sqrt();

    let ray = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 0.5), Vector::new(1.0, 0.0, 16.0))
            .expect("a normal with length"),
    );
    let answer = meet_of_three_surfaces(
        [&ball, &pipe, &ray],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a plane leaning beside the axis a ball and a pipe share is answered");
    let wanted: Vec<[f64; 3]> = [-along, along]
        .into_iter()
        .flat_map(|x| {
            let z = (8.0 - x) / 16.0;
            let out = (1.0 - z * z).sqrt();
            [[x, -out, z], [x, out, z]]
        })
        .collect();
    assert_narrow_boxes_round(&answer, &wanted);

    let beside = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 0.5), Vector::new(0.0, 0.0, 1.0))
            .expect("a unit normal"),
    );
    let answer = meet_of_three_surfaces(
        [&ball, &pipe, &beside],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a plane beside the axis a ball and a pipe share is answered");
    let out = 0.75_f64.sqrt();
    assert_narrow_boxes_round(
        &answer,
        &[
            [-along, -out, 0.5],
            [-along, out, 0.5],
            [along, -out, 0.5],
            [along, out, 0.5],
        ],
    );

    let across = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0))
            .expect("a unit normal"),
    );
    assert_eq!(
        meet_of_three_surfaces(
            [&ball, &pipe, &across],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ),
        Ok(MeetOfThree::Nowhere),
        "a plane square to the pipe's axis cuts it and the ball in two circles about one centre"
    );
}

/// `answer` is as many points as `wanted`, and each of `wanted` has a box in
/// it no wider than `1e-14` whose middle is within `1e-14` of it.
fn assert_narrow_boxes_round(answer: &MeetOfThree, wanted: &[[f64; 3]]) {
    let MeetOfThree::At(points) = answer else {
        panic!("expected {} points, got {answer:?}", wanted.len());
    };
    assert_eq!(points.len(), wanted.len(), "{answer:?}");
    for wanted in wanted {
        assert!(
            points.iter().any(|point| {
                let boxed = point.enclosure();
                (0..3).all(|axis| {
                    let middle = (boxed[axis].lo() + boxed[axis].hi()) / 2.0;
                    (middle - wanted[axis]).abs() < 1e-14 && boxed[axis].width() < 1e-14
                })
            }),
            "no narrow box round {wanted:?} in {answer:?}"
        );
    }
}

/// §31.10's first route needs a meet of three surfaces, and for a ball off the
/// pipe's axis the one it needs is one the milestone refuses.
///
/// **This is the demonstration behind the route §35 chose.** Move the ball of
/// the test above half a unit along `y`, off the pipe's axis, and the two share
/// no circle: taking the pipe `y² + z² = 1` from the ball
/// `x² + (y − ½)² + z² = 9` leaves `y = x² − 7.75`, and on the plane
/// `x + 16z = 8` that is `y = (8 − 16z)² − 7.75`, which the pipe then holds
/// where a quartic in `z` is zero. It has four roots, near `z = 0.316`, `0.337`,
/// `0.665` and `0.682`, so there are four points to find, and
/// [`meet_of_three_surfaces`] refuses the three by name — §31.2's row 4, a gap
/// deferred to the milestone that builds an algebraic solver.
///
/// No wall is a mirror of all three either. The one wall through the pipe's
/// axis and the plane's normal is `y = 0`, and the ball's centre `(0, 0.5, 0)`
/// is off it, so the road for a plane one wall mirrors with both, which takes
/// a ball whose centre is on that wall (Q2196), does not take this one.
///
/// The controls are what the same call *can* do: the plane `z = 0.5` it leaned
/// from runs along the pipe and cuts it in two lines, and two planes and the
/// ball come back answered, so what is refused is the degree and not the call.
#[test]
fn a_ray_along_a_balls_skin_leaning_to_a_pipe_off_its_centre_is_refused_with_four_points_there() {
    let ball = Surface::Sphere(Sphere::new(Point::new(0.0, 0.5, 0.0), RADIUS).expect("a ball"));
    let pipe = Surface::Cylinder(
        Cylinder::new(Axis::x(), 1.0).expect("a pipe of radius one along the x axis"),
    );
    let quartic = |z: f64| {
        let y = (8.0 - 16.0 * z).powi(2) - 7.75;
        y * y + z * z - 1.0
    };
    let sign_changes = (300..700)
        .filter(|&step| {
            let [here, next] = [step, step + 1].map(|at| quartic(f64::from(at) / 1000.0));
            here.signum() != next.signum()
        })
        .count();
    assert_eq!(
        sign_changes, 4,
        "four points to find between z = 0.3 and 0.7"
    );

    let ray = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 0.5), Vector::new(1.0, 0.0, 16.0))
            .expect("a normal with length"),
    );
    let refusal = meet_of_three_surfaces(
        [&ball, &pipe, &ray],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect_err("a ball off a pipe's axis and a plane leaning to that axis is one plane short");
    assert!(
        matches!(
            refusal,
            rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat
        ),
        "got {refusal:?}"
    );

    let beside = Surface::Plane(
        Plane::from_point_and_normal(Point::new(0.0, 0.0, 0.5), Vector::new(0.0, 0.0, 1.0))
            .expect("a unit normal"),
    );
    let answer = meet_of_three_surfaces(
        [&ball, &pipe, &beside],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("a plane along a pipe is answered");
    let MeetOfThree::At(points) = &answer else {
        panic!("expected four points, got {answer:?}");
    };
    assert_eq!(points.len(), 4, "{answer:?}");

    let across = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0))
            .expect("a unit normal"),
    );
    meet_of_three_surfaces(
        [&ball, &across, &ray],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("two planes and a ball is answered");
}

/// The same grid against a bar and a ball that have been turned and moved,
/// reported family by family and never blended (`docs/decisions.md` §17.7).
///
/// **This is the row that would have gone wrong quietly.** Which side of a
/// trimming surface a face lies on is read off its corners, and a corner that is
/// exactly on a cap circle is carried into place by an isometry that rounds, so
/// it lands a hair off the cap's plane on whichever side the rounding chose. Read
/// with a predicate, a placed bar's wall would look as though it stood on both
/// sides of its own cap. The corners of the edges running along a surface are
/// skipped by topology instead, and this is what says so.
///
/// **Nothing is recorded, and the reason is structural rather than lucky.** The
/// grid is drawn on the lattice and the solid is not, so every ray meets a
/// turned surface in a box; but a certified policy never accepts anything, so a
/// predicate handed a box either proves its sign and says nothing, or gives up.
/// A record here would mean a decision had been taken on something nobody
/// proved, which is what §17.2 keeps apart from being undecided.
#[test]
fn a_turned_bar_and_ball_are_placed_as_surely_as_a_lattice_one() {
    let axis = Axis::new(Point::origin(), Vector::new(1.0, 2.0, 3.0)).expect("a direction");
    let turn = Isometry::rotation(&axis, 0.7)
        .expect("a rotation")
        .compose(&Isometry::translation(Vector::new(0.25, -0.5, 0.75)).expect("a move"));

    for (make, what) in [
        (a_round_bar as fn() -> (Model, SolidKey), "bar"),
        (a_ball as fn() -> (Model, SolidKey), "ball"),
    ] {
        let (mut model, solid) = make();
        model
            .place_solid(solid, &turn)
            .expect("this model made the key");
        let placed = solid;
        let mut log = ProvenanceLog::new();
        let mut counted = 0;
        for i in -6..=6 {
            for j in -6..=6 {
                for k in -6..=6 {
                    let at = Point::new(
                        f64::from(i) / 2.0 + 0.125,
                        f64::from(j) / 2.0 + 0.125,
                        f64::from(k) / 2.0 + 0.125,
                    );
                    let where_it_sits = model
                        .solid(placed)
                        .expect("this model made the key")
                        .classify_point(at, &Policy::certified(), &mut log);
                    assert!(
                        matches!(
                            where_it_sits,
                            Containment::Inside | Containment::Outside | Containment::OnBoundary
                        ),
                        "the turned {what} could not place {at:?}: {where_it_sits:?}"
                    );
                    counted += 1;
                }
            }
        }
        assert_eq!(counted, 13 * 13 * 13);
        assert!(
            log.records().is_empty(),
            "the turned {what} recorded {:?}",
            log.records()
        );
    }
}
