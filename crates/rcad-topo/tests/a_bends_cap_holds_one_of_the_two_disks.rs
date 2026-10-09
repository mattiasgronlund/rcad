//! A bend's end cap holds its own disk and not the other one (§250).
//!
//! A plane through a torus's axis cuts it in **two** circles, and the inside of
//! the tube on that plane is both their disks. A bend's cap is one of them. So
//! the torus's sign alone cannot say what the cap holds: it is inside at the
//! centre of the other disk too, and zero all round the other circle.
//!
//! The fuse asks this as well. The one partner whose caps are a bend's other
//! disks is the bend turned half a turn, and fusing the two asks each cap about
//! the centre of its other disk (§250.4, §280.6).
//!
//! What tells the two disks apart is a flat half-space whose plane holds the
//! ring's axis and stands square to the cap: for the cap on `y = 0` that plane
//! is `x = 0`. It cuts the tube too, in the disks of two more circles, round
//! `(0, ±4, 0)`. The three tests after the fuse are about points there.
//!
//! That plane meets the cap's plane in the ring's axis, and the three tests
//! after those are about the axis and points a hair off it (Q418).

use core::f64::consts::PI;

use rcad_geom::{Axis, Deflection, Isometry, Point, SIDE_OF_A_HALF_SPACE, Scale, Vector};
use rcad_predicates::{Interval, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, Error, FaceKey, Model, OnAFace, ProfileBuilder, Rule, SolidKey};

mod rings;

use rings::{a_bend_turned_and_written_out, a_quarter_bend};

/// A quarter bend of ring four and tube one, and its cap on `y = 0`, whose
/// circle goes round `(4, 0, 0)` through the corner `(3, 0, 0)`.
///
/// The same plane cuts the ring in a second circle, round `(−4, 0, 0)`, and
/// nothing of the bend is there: it runs from `+x` to `+y`.
fn a_bend_and_its_cap_on_y_equals_zero() -> (Model, SolidKey, FaceKey) {
    a_bend_and_its_cap_through(4.0, 1.0)
}

/// A quarter bend of ring `ring` and tube `tube`, and its cap on `y = 0`,
/// found by its one corner `(ring − tube, 0, 0)`.
fn a_bend_and_its_cap_through(ring: f64, tube: f64) -> (Model, SolidKey, FaceKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let solid = a_quarter_bend(&mut model, ring, tube);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a sound bend");
    let corner = Point::new(ring - tube, 0.0, 0.0);
    let cap = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .find(|face| {
            face.surface().as_plane().is_some()
                && face
                    .wires()
                    .flat_map(|wire| wire.points())
                    .any(|found| found == corner)
        })
        .expect("the bend's cap through its inner corner")
        .key();
    (model, solid, cap)
}

fn at(x: f64, z: f64) -> PredicatePoint {
    PredicatePoint::from([x, 0.0, z])
}

/// Every point is on `y = 0`, and each answer is read off the distance from
/// the point to the two circle centres `(±4, 0, 0)` against the tube's radius
/// of one: under one is inside that disk, exactly one is on that circle.
#[test]
fn a_bends_cap_holds_its_own_disk_and_not_the_other() {
    let (model, solid, cap) = a_bend_and_its_cap_on_y_equals_zero();
    let bend = model.solid(solid).expect("this model made the key");
    let holds = |point| {
        bend.face_holds_the_point(cap, point, &Policy::certified(), &mut ProvenanceLog::new())
    };
    let mut wrong = Vec::new();
    for (point, expected, why) in [
        (at(4.0, 0.0), OnAFace::Held, "its own circle's centre"),
        (
            at(4.0, 0.5),
            OnAFace::Held,
            "half a tube above its own centre",
        ),
        (at(3.0, 0.0), OnAFace::OnItsBoundary, "its own corner"),
        (
            at(5.0, 0.0),
            OnAFace::OnItsBoundary,
            "its own circle, across from the corner",
        ),
        (
            at(4.0, 1.0),
            OnAFace::OnItsBoundary,
            "the top of its own circle",
        ),
        (
            at(4.0, 2.0),
            OnAFace::NotHeld,
            "a tube above its own circle",
        ),
        (
            at(0.0, 0.0),
            OnAFace::NotHeld,
            "the ring's centre, four from each circle's centre",
        ),
        (at(-4.0, 0.0), OnAFace::NotHeld, "the other circle's centre"),
        (
            at(-4.0, 0.5),
            OnAFace::NotHeld,
            "half a tube above the other centre",
        ),
        (at(-3.0, 0.0), OnAFace::NotHeld, "the other circle"),
        (at(-5.0, 0.0), OnAFace::NotHeld, "the other circle"),
        (
            at(-4.0, 1.0),
            OnAFace::NotHeld,
            "the top of the other circle",
        ),
    ] {
        let answer = holds(point);
        if !matches!(answer, Ok(found) if found == expected) {
            wrong.push(format!("{point:?}, {why}: {answer:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// The bend runs a quarter of the way round from `+x` to `+y`, so the other
/// disk on `y = 0`, half a turn away, is nowhere near it. A ray from any point
/// there starts on the cap's plane, and the cap is asked whether it holds the
/// point: holding it put the point on the bend's skin.
#[test]
fn a_point_in_the_other_disk_is_outside_the_bend() {
    let (model, solid, _) = a_bend_and_its_cap_on_y_equals_zero();
    let bend = model.solid(solid).expect("this model made the key");
    let wrong: Vec<_> = [
        Point::new(-4.0, 0.0, 0.0),
        Point::new(-4.0, 0.0, 0.5),
        Point::new(-3.0, 0.0, 0.0),
        Point::new(-5.0, 0.0, 0.0),
    ]
    .into_iter()
    .map(|point| {
        (
            point,
            bend.classify_point(point, &Policy::certified(), &mut ProvenanceLog::new()),
        )
    })
    .filter(|(_, found)| *found != Containment::Outside)
    .collect();
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// The bend fused with a copy of itself turned half a turn about the ring's
/// axis, under every rule in both orders (§250.4, §280.6).
///
/// The copy runs from `−x` round to `−y`, so its caps are the other disks on
/// the bend's two cap planes: round `(−4, 0, 0)` on `y = 0` and round
/// `(0, −4, 0)` on `x = 0`. The two bends share no point, and each holds
/// `π · 1² · (π/2) · 4 = 2π²` by Pappus. So the union is the two bends apart,
/// the intersection is nothing, and each difference is its first argument,
/// each in the three faces it came in with.
///
/// **This is the fuse that asks a cap about a point in its other disk.** The
/// fuse tries each cap at its centre, so it asks the bend about `(−4, 0, 0)`
/// and `(0, −4, 0)` and the copy about `(4, 0, 0)` and `(0, 4, 0)`. Each of
/// those is in the other disk of the cap on the same plane, and
/// `settle_against_each` puts it to that cap twice: once to see whether the
/// piece lies on the cap (`coincident_face`, through `face_holds_the_point`),
/// and once through the ray `classify_uncertain_point` fires from the point,
/// which starts on the cap's plane. A temporary print in
/// `what_a_round_wire_encloses` counted eight such calls in each of the six
/// fuses, two per cap. In every one the tube holds the point and the
/// half-space puts it outside the cap. Every other call it counted had its
/// point outside the tube.
///
/// So the half-space is what this test leans on. With the `InFrontOf` arm
/// keeping whatever the tube holds without asking the half-space, as before
/// §250, both unions are refused `ResultIsNotManifold { count: 1 }`. The
/// intersections and differences still come out right.
#[test]
fn a_bend_fused_with_its_copy_turned_half_a_turn_comes_to_what_each_rule_says() {
    for (rule, bend_first, answer) in [
        (Rule::Union, true, &["the bend", "the copy"][..]),
        (Rule::Union, false, &["the bend", "the copy"]),
        (Rule::Intersection, true, &[]),
        (Rule::Intersection, false, &[]),
        (Rule::Difference, true, &["the bend"]),
        (Rule::Difference, false, &["the copy"]),
    ] {
        let named = format!(
            "{rule:?} with the bend {}",
            if bend_first { "first" } else { "second" }
        );
        let policy = Policy::certified();
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bend = a_bend_turned_and_written_out(&mut model, 4.0, 1.0, 1, 0)
            .expect("a quarter bend is a solid");
        let copy = a_bend_turned_and_written_out(&mut model, 4.0, 1.0, 1, 2)
            .expect("a quarter bend turned half a turn is a solid since Q190");
        let (one, other) = if bend_first {
            (bend, copy)
        } else {
            (copy, bend)
        };
        let fused = model
            .fuse(&[one], &[other], rule, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{named}: {why:?}"));
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{named}: the answer does not validate: {why:?}"));
        let mut found: Vec<&str> = fused
            .solids()
            .iter()
            .map(|&solid| which_bend_it_is(&model, solid, &named))
            .collect();
        found.sort_unstable();
        assert_eq!(found, answer, "{named}");
    }
}

/// A point the tube holds on `x = 0`, the plane between the cap's disk and the
/// other one, is counted in the cap's disk: the half-space naming the cap's
/// circle keeps what is on its plane, as a picker keeps what is on it. The
/// fuse asks the same half-space through the same function, so this is its
/// rule too, though no piece it tries a point in can reach it (Q117).
///
/// **None of these points is on the cap's plane**, and no caller inside the
/// kernel hands the cap one: ray casting, the fuse and the overlap search all
/// ask about points on it, where `x = 0` meets only the ring's axis, three
/// from the tube. `face_holds_the_point` does not ask whether its point is on
/// the face, so this is the one way to reach that rule, and without this test
/// nothing notices it change (§250.6).
#[test]
fn a_point_in_the_tube_on_the_plane_between_the_two_disks_is_kept() {
    let (model, solid, cap) = a_bend_and_its_cap_on_y_equals_zero();
    let bend = model.solid(solid).expect("this model made the key");
    let mut wrong = Vec::new();
    for (point, expected, why) in [
        ([0.0, 4.0, 0.0], OnAFace::Held, "the other cap's centre"),
        (
            [0.0, 4.0, 0.5],
            OnAFace::Held,
            "half a tube above the other cap's centre",
        ),
        (
            [0.0, 3.0, 0.0],
            OnAFace::OnItsBoundary,
            "the other cap's corner, on the tube",
        ),
        (
            [0.0, 5.0, 0.0],
            OnAFace::OnItsBoundary,
            "the other cap's circle, across from its corner",
        ),
    ] {
        let answer = bend.face_holds_the_point(
            cap,
            PredicatePoint::from(point),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        if !matches!(answer, Ok(found) if found == expected) {
            wrong.push(format!("{point:?}, {why}: {answer:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// A small box round the other cap's centre is all inside the tube, so the
/// torus lets it through, and it reaches across `x = 0`, so the half-space
/// cannot say which disk it is in. That is a refusal, and the half-space is
/// what it names.
#[test]
fn a_box_across_the_plane_between_the_two_disks_is_refused() {
    let (model, solid, cap) = a_bend_and_its_cap_on_y_equals_zero();
    let bend = model.solid(solid).expect("this model made the key");
    let across = PredicatePoint::Constructed([
        Interval::new(-0.001, 0.001).expect("a box a thousandth either side of x = 0"),
        Interval::exactly(4.0),
        Interval::exactly(0.0),
    ]);
    let answer =
        bend.face_holds_the_point(cap, across, &Policy::certified(), &mut ProvenanceLog::new());
    assert!(is_the_half_space_giving_up(&answer), "{answer:?}");
}

/// The same refusal for a box that holds a point of the cap itself, so it is
/// not only a question asked off the cap's plane.
///
/// A fat bend, ring four and tube three, turned an eighth of a turn about its
/// axis, has its first cap on `x = y` and the plane between the two disks on
/// `x = −y`. The box `[−2.1, 2.1] × {2} × {0}` holds `(2, 2, 0)`, which is on
/// the cap, and reaches across `x = −y`, and the torus certifies all of it
/// inside the tube.
///
/// A bend square to the axes has no such box: a box holding a point of
/// `y = 0` and reaching across `x = 0` holds a point of the ring's axis, and
/// the axis is outside the tube.
#[test]
fn a_box_holding_a_point_of_a_turned_fat_bends_cap_is_refused_across_the_plane_between_the_disks() {
    let (mut model, solid, cap) = a_bend_and_its_cap_through(4.0, 3.0);
    let an_eighth_of_a_turn =
        Isometry::rotation(&Axis::z(), std::f64::consts::FRAC_PI_4).expect("a finite turn");
    model
        .place_solid(solid, &an_eighth_of_a_turn)
        .expect("this model made the key");
    let bend = model.solid(solid).expect("this model made the key");
    let on_the_cap = PredicatePoint::from([2.0, 2.0, 0.0]);
    let answer = bend.face_holds_the_point(
        cap,
        on_the_cap,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(answer, Ok(OnAFace::Held)),
        "(2, 2, 0) is a unit and a bit from the centre of the cap's circle, of radius three: \
         {answer:?}",
    );
    let across = PredicatePoint::Constructed([
        Interval::new(-2.1, 2.1).expect("a box from past x = -y to past x = y"),
        Interval::exactly(2.0),
        Interval::exactly(0.0),
    ]);
    let answer =
        bend.face_holds_the_point(cap, across, &Policy::certified(), &mut ProvenanceLog::new());
    assert!(is_the_half_space_giving_up(&answer), "{answer:?}");
}

/// Neither the cap nor the bend holds a point of the ring's axis, which is the
/// line where the cap's plane meets the plane between its two disks, nor a point
/// a hair off that line on either plane (§250.8, Q418).
///
/// The half-space naming the cap's circle keeps a point on its own plane, and
/// `the_meridian_half_space_keeps` says no point of the face's plane gets that
/// far: the two planes meet in the axis, and the axis is the ring's radius from
/// the ring's circle, which is more than the tube's. These are the points that
/// claim is about. On the axis: the ring's centre, and heights from inside the
/// tube's height to a million out. Beside it: `2⁻¹⁰⁰⁰` and `2⁻⁵²` off it either
/// way along the cap's plane, and the same along the plane between the disks.
/// They are asked of the bend of ring four and tube one as built, and of the
/// bend of ring four and tube three turned an eighth of a turn by `place_solid`,
/// whose cap is asked in a frame it is not stored in. Every point is at least
/// `4 − 2⁻⁵²` from the ring's circle against a tube of three at most, so the cap
/// holds none of them and the bend leaves every one outside.
///
/// **The half-space's `Zero` arm was made a panic for Q418's probe**, and none
/// of these 144 points reached it, through either question: the tube holds none
/// of them, and the cap asks its half-space only about a point the tube holds.
#[test]
fn a_bends_cap_holds_nothing_on_or_beside_the_rings_axis() {
    let as_built = a_bend_and_its_cap_through(4.0, 1.0);
    let (mut model, turned, turned_cap) = a_bend_and_its_cap_through(4.0, 3.0);
    let an_eighth_of_a_turn =
        Isometry::rotation(&Axis::z(), std::f64::consts::FRAC_PI_4).expect("a finite turn");
    model
        .place_solid(turned, &an_eighth_of_a_turn)
        .expect("this model made the key");
    let mut wrong = Vec::new();
    for (named, (model, solid, cap)) in [
        ("as built", as_built),
        ("turned", (model, turned, turned_cap)),
    ] {
        let bend = model.solid(solid).expect("this model made the key");
        for point in on_and_beside_the_axis(&model, solid, cap) {
            let on_the_face = bend.face_holds_the_point(
                cap,
                PredicatePoint::from(point),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            let in_the_solid = bend.classify_point(
                Point::from(point),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            if !matches!(on_the_face, Ok(OnAFace::NotHeld)) || in_the_solid != Containment::Outside
            {
                wrong.push(format!(
                    "{named}, {point:?}: {on_the_face:?}, {in_the_solid:?}"
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// The same line for a bend whose tube all but reaches its axis: ring one and
/// tube `1 − g`, with `g = 2⁻³⁰`. Its cap's corner is `(g, 0, 0)`, the other
/// disk's circle passes through `(−g, 0, 0)`, and between the two, on the cap's
/// plane, is the ring's axis. Each answer is read off the distance from the
/// point to the centre of the circle on its own side, `(±1, 0, 0)`, against the
/// tube's radius: `2g` off the axis is `1 − 2g` from that centre and inside, `g`
/// off is on the circle, and `g/2` off is `1 − g/2` from it and outside.
///
/// So the points of the tube nearest the line are asked about here, and the
/// half-space is asked about four of them. **Its `Zero` arm was made a panic for
/// Q418's probe**, and none of these 15 points reached it, through either
/// question: on the cap's plane the tube holds nothing nearer the axis than
/// `g`, and the half-space is strictly on one side or the other there.
///
/// **Both these answer, and so does the ring four, tube `4 − g` corner.** The
/// cap's corner is on the bend's skin, `OnBoundary`, and `(−g, 0, 0)` is
/// outside it, `Outside`. Before Q459's slide, `classify_point` said
/// `CannotFireARayAtAFacesSurface` of both, naming the wall; the slide to
/// `where_a_line_meets_a_surface` that answers the size-one question at
/// `1e-159` also answers the ray this corner fires (§349.8, §349.10). The
/// corner of the bend of ring four and tube one, `(3, 0, 0)`, answers
/// `OnBoundary` too, and so does ring four and tube `4 − g`'s corner, pinned
/// below.
///
/// `g` keeps this inside §3.5's envelope: the bend is two across and `g` is
/// about `1e-9`. At `g = 2⁻⁵²` the bend is refused at build,
/// `CannotNameWhichPieceOfACrossingThisIs`, a spread of `2⁵³` and past the
/// envelope.
#[test]
fn a_bend_whose_tube_all_but_reaches_its_axis_holds_only_its_own_side_of_it() {
    let gap = 2.0_f64.powi(-30);
    let (model, solid, cap) = a_bend_and_its_cap_through(1.0, 1.0 - gap);
    let bend = model.solid(solid).expect("this model made the key");
    let a_hair = 2.0_f64.powi(-1000);
    let mut wrong = Vec::new();
    for (point, on_the_face, in_the_solid, why) in [
        (
            [0.0, 0.0, 0.0],
            OnAFace::NotHeld,
            Containment::Outside,
            "the ring's centre",
        ),
        (
            [0.0, 0.0, 0.5],
            OnAFace::NotHeld,
            Containment::Outside,
            "the axis, inside the tube's height",
        ),
        (
            [0.0, 0.0, -1.0],
            OnAFace::NotHeld,
            Containment::Outside,
            "the axis, level with the bottom of the tube",
        ),
        (
            [0.0, 0.0, 1.0e6],
            OnAFace::NotHeld,
            Containment::Outside,
            "the axis, far above the ring",
        ),
        (
            [a_hair, 0.0, 0.0],
            OnAFace::NotHeld,
            Containment::Outside,
            "a hair off the axis towards the cap",
        ),
        (
            [-a_hair, 0.0, 0.0],
            OnAFace::NotHeld,
            Containment::Outside,
            "a hair off the axis towards the other disk",
        ),
        (
            [0.0, a_hair, 0.0],
            OnAFace::NotHeld,
            Containment::Outside,
            "a hair off the axis on the plane between the disks",
        ),
        (
            [0.0, -a_hair, 0.0],
            OnAFace::NotHeld,
            Containment::Outside,
            "a hair off the axis on the plane between the disks, the other way",
        ),
        (
            [gap / 2.0, 0.0, 0.0],
            OnAFace::NotHeld,
            Containment::Outside,
            "between the axis and the cap's corner",
        ),
        (
            [-gap / 2.0, 0.0, 0.0],
            OnAFace::NotHeld,
            Containment::Outside,
            "between the axis and the other circle",
        ),
        (
            [gap, 0.0, 0.0],
            OnAFace::OnItsBoundary,
            Containment::OnBoundary,
            "the cap's corner",
        ),
        (
            [-gap, 0.0, 0.0],
            OnAFace::NotHeld,
            Containment::Outside,
            "the other circle",
        ),
        (
            [2.0 * gap, 0.0, 0.0],
            OnAFace::Held,
            Containment::OnBoundary,
            "just inside the cap's disk",
        ),
        (
            [-2.0 * gap, 0.0, 0.0],
            OnAFace::NotHeld,
            Containment::Outside,
            "just inside the other disk",
        ),
        (
            [gap, 0.0, 1.0],
            OnAFace::NotHeld,
            Containment::Outside,
            "above the cap's corner, level with the top of the tube",
        ),
    ] {
        let found_on_the_face = bend.face_holds_the_point(
            cap,
            PredicatePoint::from(point),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let found_in_the_solid = bend.classify_point(
            Point::from(point),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        if !matches!(found_on_the_face, Ok(found) if found == on_the_face)
            || found_in_the_solid != in_the_solid
        {
            wrong.push(format!(
                "{point:?}, {why}: {found_on_the_face:?}, {found_in_the_solid:?}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// A fuse that would put corners on the ring's axis is refused: a block with a
/// face on the cap's plane, a face on the plane between the disks, and an edge
/// along the axis from `(0, 0, −½)` to `(0, 0, ½)`, under every rule, both ways
/// round. Its sides at `x = 1` and `y = −1` run along the axis and off it, and
/// such a plane cuts a torus in a quartic
/// (`how_often_a_fuse_with_a_torus_refuses.rs` has the general argument).
///
/// **When this fuse is answered, ask it again with the half-space's `Zero` arm
/// made a panic**: its corners are on the line the arm is said never to see.
#[test]
fn a_block_with_an_edge_along_a_bends_axis_is_refused_as_a_quartic() {
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for block_first in [false, true] {
            let (mut model, bend, _) = a_bend_and_its_cap_on_y_equals_zero();
            let square =
                ProfileBuilder::new(vec![[0.0, -1.0], [1.0, -1.0], [1.0, 0.0], [0.0, 0.0]])
                    .build(&Policy::certified(), &mut ProvenanceLog::new())
                    .expect("a unit square with a corner on the axis");
            let block = model
                .add_extrusion(
                    &square,
                    1.0,
                    &Isometry::translation(Vector::new(0.0, 0.0, -0.5)).expect("a finite shift"),
                )
                .expect("a block");
            let (one, other) = if block_first {
                (block, bend)
            } else {
                (bend, block)
            };
            let refused = model.fuse(
                &[one],
                &[other],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            assert!(
                matches!(
                    refused,
                    Err(Error::Geometry(
                        rcad_geom::Error::ACrossingBetweenAPlaneAndATorusThatIsAQuartic
                    ))
                ),
                "{rule:?}, block first {block_first}: {refused:?}"
            );
        }
    }
}

/// Which of the two bends `solid` is: it holds the middle of that bend's tube
/// and not the other's, has a bend's three faces, and holds the `2π²` of a
/// quarter of the ring within what `measure` says it can be out by.
fn which_bend_it_is(model: &Model, solid: SolidKey, named: &str) -> &'static str {
    let policy = Policy::certified();
    let view = model.solid(solid).expect("the fuse made the key");
    let holds = |at| view.classify_point(Point::from(at), &policy, &mut ProvenanceLog::new());
    let which = match (holds([3.5, 3.5, 0.0]), holds([-3.5, -3.5, 0.0])) {
        (Containment::Inside, Containment::Outside) => "the bend",
        (Containment::Outside, Containment::Inside) => "the copy",
        both => panic!("{named}: a solid that places the middles of the two tubes {both:?}"),
    };
    assert_eq!(view.faces().count(), 3, "{named}: {which}");
    let measured = view
        .measure(
            Deflection::of(0.01).expect("a hundredth of the tube"),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{named}: {which} does not measure: {why:?}"));
    let a_quarter_of_the_ring = 2.0 * PI * PI;
    assert!(
        (measured.volume() - a_quarter_of_the_ring).abs() <= measured.volume_error(),
        "{named}: {which} holds {a_quarter_of_the_ring}, and measures {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
    which
}

fn is_the_half_space_giving_up(answer: &Result<OnAFace, Error>) -> bool {
    match answer {
        Err(Error::Undecidable { records }) => {
            !records.is_empty()
                && records
                    .records()
                    .iter()
                    .all(|record| record.predicate() == SIDE_OF_A_HALF_SPACE)
        }
        Ok(_) | Err(_) => false,
    }
}

/// The ring's axis, which is `z`, at eight heights from the ring's centre to a
/// million out, and beside each height `2⁻¹⁰⁰⁰` and `2⁻⁵²` off the axis either
/// way along the cap's plane and along the plane between its disks: 72 points.
///
/// The step along the cap's plane is the axis crossed with the cap's normal as
/// `face`'s view places it, so a step of a power of two lands exactly on the cap's
/// plane whatever the turn. The step along the plane between the disks is the
/// cap's normal itself, which that plane holds.
fn on_and_beside_the_axis(model: &Model, solid: SolidKey, cap: FaceKey) -> Vec<[f64; 3]> {
    let surface = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .find(|face| face.key() == cap)
        .expect("the cap is a face of the bend")
        .surface();
    let [x, y, z] = surface
        .as_plane()
        .expect("a bend's cap is flat")
        .normal()
        .components();
    assert_eq!(z, 0.0, "a cap's plane holds the ring's axis, which is z");
    let along_the_cap = [-y, x];
    let across_the_cap = [x, y];
    let mut points = Vec::new();
    for height in [0.0, 0.5, -0.5, 1.0, -1.0, 3.0, -3.0, 1.0e6] {
        points.push([0.0, 0.0, height]);
        for step in [2.0_f64.powi(-1000), f64::EPSILON] {
            for signed in [step, -step] {
                for [x, y] in [along_the_cap, across_the_cap] {
                    points.push([signed * x, signed * y, height]);
                }
            }
        }
    }
    points
}

/// The cap's corner of a bend of ring four and tube `4 − g`, with
/// `g = 2⁻³⁰`: the same shape as
/// [`a_bend_whose_tube_all_but_reaches_its_axis_holds_only_its_own_side_of_it`]
/// at four times the size. Before Q459's slide this corner refused the same
/// way, `CannotFireARayAtAFacesSurface`; the slide answers it too, `OnBoundary`,
/// so it is not the ring's size that moved the refusal (§349.10).
#[test]
fn a_bigger_bends_cap_corner_also_answers_since_the_slide() {
    let gap = 2.0_f64.powi(-30);
    let (model, solid, cap) = a_bend_and_its_cap_through(4.0, 4.0 - gap);
    let bend = model.solid(solid).expect("this model made the key");
    let found_on_the_face = bend.face_holds_the_point(
        cap,
        PredicatePoint::from([gap, 0.0, 0.0]),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    let found_in_the_solid = bend.classify_point(
        Point::from([gap, 0.0, 0.0]),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(found_on_the_face, Ok(OnAFace::OnItsBoundary)),
        "{found_on_the_face:?}"
    );
    assert_eq!(found_in_the_solid, Containment::OnBoundary);
}
