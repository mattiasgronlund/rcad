//! Cutting a whole solid into triangles, and the one oracle that says it worked.
//!
//! `docs/decisions.md` §23.4: *"The oracle is combinatorial rather than metric —
//! every mesh edge used by exactly two triangles with opposite orientation, and
//! χ of the mesh equal to χ of the solid — which is a sharper claim than 'the
//! gaps are small'."* That is [`the_mesh_closes_up`], and every test here runs
//! it. There is no distance anywhere in it.
//!
//! The flat solids come first on purpose. A cube and the notched extrusions the
//! rest of the suite already trusts are shapes this kernel has been right about
//! since M0, so if the oracle fails on one of those it is the oracle that is
//! wrong — and it also gives `mesh` a consumer that leans on none of M2's new
//! curved machinery.

use std::cell::Cell;
use std::collections::{HashMap, HashSet};

use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, TestCaseError, TestRng, TestRunner};
use rcad_geom::{Axis, Deflection, Isometry, Plane, Point, Scale, Vector};
use rcad_predicates::{Cause, Policy, ProvenanceLog};
use rcad_topo::{Error, Mesh, Model, ProfileBuilder, Rule, SolidKey};

const RADIUS: f64 = 3.0;
const HEIGHT: f64 = 5.0;

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

fn meshed(model: &Model, solid: SolidKey, deflection: f64) -> Mesh {
    let mut log = ProvenanceLog::new();
    let mesh = model
        .solid(solid)
        .expect("this model made the key")
        .mesh(
            Deflection::of(deflection).expect("a positive length"),
            &Policy::certified(),
            &mut log,
        )
        .unwrap_or_else(|why| panic!("meshing at a deflection of {deflection}: {why}"));
    assert!(
        log.is_empty(),
        "nothing about this mesh had to go unsettled, got {:?}",
        log.records()
    );
    the_mesh_closes_up(model, solid, &mesh, deflection);
    mesh
}

/// §23.4's oracle, in full, with no distance in it.
///
/// Four claims, and every one of them is about **indices**:
///
/// 1. Every side of every triangle is used the same number of times each way
///    round. That is watertightness: a side used once is a hole, and a side used
///    twice the same way is a fold. It is a **balance** and not the count §23.4
///    asks for — a side used four times, twice each way, would pass — and it is
///    the balance because that is what makes "twice the same way round" a
///    failure, and because no input the kernel can build fails the count without
///    also failing this.
/// 2. χ of the mesh is χ of the solid. Closing up the holes is not enough on its
///    own — a mesh that covered the ball twice over would pass the first check
///    and fail this one.
/// 3. No two entries of the point table are the same place. If the two faces
///    along an edge had each worked its points out for themselves they would
///    either land apart, which check 1 catches, or land together, which only
///    this catches. It is what "bit for bit" (§23.4) means as a test.
/// 4. Every triangle names a face this solid actually has, no triangle stands on
///    one point twice, and no point is in the table without being in a triangle.
fn the_mesh_closes_up(model: &Model, solid: SolidKey, mesh: &Mesh, deflection: f64) {
    let faces: HashSet<_> = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .map(|face| face.key())
        .collect();
    let mut times_used: HashMap<(u32, u32), i32> = HashMap::new();
    for triangle in mesh.triangles() {
        assert!(
            faces.contains(&triangle.face()),
            "a triangle names a face this solid does not have"
        );
        let [first, second, third] = triangle.corners();
        assert!(
            first != second && second != third && third != first,
            "the triangle {:?} stands on one point twice",
            triangle.corners()
        );
        for (from, to) in [(first, second), (second, third), (third, first)] {
            let side = (from.min(to), from.max(to));
            *times_used.entry(side).or_default() += if from < to { 1 } else { -1 };
        }
    }

    for (side, balance) in &times_used {
        assert_eq!(
            *balance, 0,
            "the side between points {side:?} is used {balance} more times one \
             way round than the other, so the mesh is not closed"
        );
    }

    let corners: HashSet<u32> = mesh
        .triangles()
        .iter()
        .flat_map(|triangle| triangle.corners())
        .collect();
    let of_the_mesh =
        corners.len() as i64 - times_used.len() as i64 + mesh.triangles().len() as i64;
    assert_eq!(
        of_the_mesh,
        euler_characteristic_of(model, solid),
        "the mesh has {} points, {} sides and {} triangles",
        corners.len(),
        times_used.len(),
        mesh.triangles().len()
    );

    let mut somewhere: HashMap<[u64; 3], u32> = HashMap::new();
    for (at, point) in mesh.points().iter().enumerate() {
        let at = at as u32;
        let bits = point.coordinates().map(f64::to_bits);
        if let Some(already) = somewhere.insert(bits, at) {
            panic!("points {already} and {at} are both at {point:?}");
        }
        assert!(
            corners.contains(&at),
            "point {at} is in the table and in no triangle"
        );
    }

    assert!(
        mesh.deviation().magnitude() <= deflection,
        "the mesh strays {} and was asked for {deflection}",
        mesh.deviation().magnitude()
    );
    assert_eq!(mesh.deviation().cause(), Cause::FlatStandIn);
}

/// χ of the solid, counted off its own entities — and **not** the naive
/// `V − E + F`, which is wrong here in two different ways.
///
/// `V − E + F` counts a *face* as a disc, and two of the shapes this kernel
/// builds have faces that are not.
///
/// - **A face with a hole in it.** A plate with a square hole through it has 16
///   corners, 24 edges and 10 faces, and the naive sum is 2 — the answer for a
///   ball. The plate is a ring: its two flat sides are each a disc with a disc
///   taken out, worth nothing rather than one, and the true answer is 0. So a
///   face is worth `2 − wires`, which is 1 for the usual one-wire face.
/// - **A pole.** A ball's one face is bounded by four wire steps, and two of
///   them go nowhere at all. Counting them as edges gives `2 − 3 + 1 = 0` for a
///   shape whose χ is 2. A pole is not an edge (§23.3), it is where the surface
///   pinches — so it adds nothing to the count, which is exactly what walking
///   the wire's *coedges* does.
///
/// A seam is not a third wrinkle: an edge used twice by one face is still one
/// edge, and counting the entities rather than the uses gets that right for
/// free.
fn euler_characteristic_of(model: &Model, solid: SolidKey) -> i64 {
    let mut corners = HashSet::new();
    let mut edges = HashSet::new();
    let mut faces = 0_i64;
    for face in model.solid(solid).expect("this model made the key").faces() {
        faces += 2 - face.wire_count() as i64;
        for wire in face.wires() {
            for coedge in wire.coedges() {
                edges.insert(coedge.edge().key());
                corners.insert(coedge.start().key());
                corners.insert(coedge.end().key());
            }
        }
    }
    corners.len() as i64 - edges.len() as i64 + faces
}

/// The control the rest of the file leans on: a shape the kernel has been right
/// about since M0, meshed without one line of M2's curved machinery.
///
/// Its mesh is the fan it already had — every point is a corner of the model, so
/// there is nothing here that stands in for a curve and the deviation is exactly
/// zero rather than merely small.
#[test]
fn a_cube_meshes_into_the_corners_it_already_had() {
    let mut model = a_model();
    let cube = model
        .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a cube of side two");

    let mesh = meshed(&model, cube, 0.1);
    assert_eq!(
        mesh.points().len(),
        8,
        "a cube has eight corners and no more"
    );
    assert_eq!(mesh.triangles().len(), 12, "two triangles a side");
    assert_eq!(
        mesh.deviation().magnitude(),
        0.0,
        "a flat triangle standing in for a flat surface stands in for nothing"
    );
    assert_eq!(euler_characteristic_of(&model, cube), 2);
}

/// A notched outline and a plate with a hole, which the fuse suite already
/// builds — and the plate is the one that made the oracle wrong the first time.
///
/// A plate with a square hole through it is a **ring**, and its χ is 0. Read off
/// its entities as `V − E + F` it comes out 2, because that sum counts the two
/// flat sides as discs and they are not. See [`euler_characteristic_of`].
#[test]
fn the_extrusions_the_suite_already_trusts_mesh_into_closed_surfaces() {
    let mut model = a_model();
    let notched = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 2.0],
        [2.0, 2.0],
        [2.0, 4.0],
        [0.0, 4.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("an L");
    let ell = model
        .add_extrusion(&notched, 1.5, &Isometry::identity())
        .expect("an L one and a half thick");

    let plate = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .with_hole(vec![[4.0, 4.0], [6.0, 4.0], [6.0, 6.0], [4.0, 6.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a plate with a hole");
    let ring = model
        .add_extrusion(&plate, 3.0, &Isometry::identity())
        .expect("a plate three thick");

    assert_eq!(euler_characteristic_of(&model, ell), 2, "an L is a ball");
    assert_eq!(
        euler_characteristic_of(&model, ring),
        0,
        "a plate with a hole through it is a ring"
    );
    for solid in [ell, ring] {
        let mesh = meshed(&model, solid, 0.1);
        assert_eq!(mesh.deviation().magnitude(), 0.0);
    }
}

fn a_round_bar(model: &mut Model, placement: &Isometry) -> SolidKey {
    model
        .add_round_bar(RADIUS, HEIGHT, placement)
        .expect("a bar three across and five tall")
}

fn a_ball(model: &mut Model, placement: &Isometry) -> SolidKey {
    model
        .add_ball(RADIUS, placement)
        .expect("a ball of radius three")
}

/// A seam used twice by one face, two closed cap edges, and a cylinder.
#[test]
fn a_round_bar_meshes_into_a_closed_surface() {
    let mut model = a_model();
    let bar = a_round_bar(&mut model, &Isometry::identity());
    assert_eq!(euler_characteristic_of(&model, bar), 2);

    let mesh = meshed(&model, bar, 0.1);
    assert!(
        mesh.deviation().magnitude() > 0.0,
        "a flat triangle standing in for a cylinder stands in for something"
    );
    for point in mesh.points() {
        let across = point.x().hypot(point.y());
        assert!(
            across <= RADIUS + 1e-9 && (0.0..=HEIGHT).contains(&point.z()),
            "the mesh point {point:?} is not on the bar"
        );
    }
}

/// One face, one seam and two poles, which is every structure a ball has.
///
/// χ of a ball is 2, and its entity counts say so: two corners, one edge, one
/// face — because the two wire steps that go nowhere are poles and a pole is not
/// an edge.
#[test]
fn a_ball_meshes_into_a_closed_surface() {
    let mut model = a_model();
    let ball = a_ball(&mut model, &Isometry::identity());
    assert_eq!(model.vertex_count(), 2);
    assert_eq!(model.edge_count(), 1);
    assert_eq!(euler_characteristic_of(&model, ball), 2);

    let mesh = meshed(&model, ball, 0.1);
    for point in mesh.points() {
        assert!(
            (point.distance_to(Point::origin()) - RADIUS).abs() < 1e-9,
            "the mesh point {point:?} is not on the ball"
        );
    }
}

/// §23.4's promise, checked where it is made rather than through the oracle.
///
/// The wall of a bar and the cap next to it meet along one closed edge. Every
/// point along it belongs to both, and they are the **same entries** of the one
/// point table — not two entries that happen to hold the same number, which is
/// what [`the_mesh_closes_up`]'s third claim rules out for the whole mesh at
/// once.
#[test]
fn an_edges_points_are_worked_out_once_and_both_faces_along_it_name_the_same_ones() {
    let mut model = a_model();
    let bar = a_round_bar(&mut model, &Isometry::identity());
    let mesh = meshed(&model, bar, 0.1);

    let mut used_by: HashMap<_, HashSet<u32>> = HashMap::new();
    for triangle in mesh.triangles() {
        used_by
            .entry(triangle.face())
            .or_default()
            .extend(triangle.corners());
    }
    assert_eq!(used_by.len(), 3, "a bar has three faces");

    let wall = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .find(|face| face.surface().as_plane().is_none())
        .expect("the bar has a wall");
    let caps: Vec<_> = used_by
        .iter()
        .filter(|(face, _)| **face != wall.key())
        .map(|(_, points)| points)
        .collect();

    for cap in &caps {
        let shared: HashSet<_> = cap.intersection(&used_by[&wall.key()]).copied().collect();
        assert_eq!(
            shared.len(),
            cap.len(),
            "every corner of a cap is on the circle it shares with the wall"
        );
        assert!(shared.len() >= 3, "a cap circle is more than a triangle");
    }

    // And the two caps are the two ends: nothing at all in common.
    assert!(caps[0].is_disjoint(caps[1]));
}

/// The cap plane at `height` facing `outward` along the axis.
fn a_cap_square_to_the_axis(height: f64, outward: f64) -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, outward))
        .expect("the z axis is a unit normal")
}

/// **§24.18's criterion 8, the easier half.** The same bar, with its cap circles
/// computed by the intersector rather than declared, meshes into the same mesh.
///
/// §24.2: *"M2 builds a round bar by declaring its edges; M3 computes the same
/// bar's cap circle from the cylinder and the cap's plane."* Nobody told
/// `add_bar_cut_off_by` that a plane square to a pipe cuts a circle — it asked,
/// and the answer came back as the same `EdgeRun` `add_round_bar` writes down
/// from a person's knowledge.
///
/// **Bit for bit and not within a tolerance**, and that is the sharpest form
/// this claim has. The two solids differ only in how their cap edges were
/// named; if computing an edge landed anywhere other than exactly where
/// declaring it does, the whole point table would drift and the meshes would be
/// two different meshes that agreed to nine figures.
#[test]
fn a_bar_whose_cap_circles_were_computed_meshes_exactly_as_the_declared_one_does() {
    let mut declared = a_model();
    let bar = a_round_bar(&mut declared, &Isometry::identity());

    let mut computed = a_model();
    let mut log = ProvenanceLog::new();
    let same = computed
        .add_bar_cut_off_by(
            RADIUS,
            [
                a_cap_square_to_the_axis(0.0, -1.0),
                a_cap_square_to_the_axis(HEIGHT, 1.0),
            ],
            &Isometry::identity(),
            &Policy::certified(),
            &mut log,
        )
        .expect("a plane square to a pipe cuts one closed curve out of it");
    assert!(
        log.is_empty(),
        "naming a cap circle of a bar this size proves everything, got {:?}",
        log.records()
    );
    computed
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar whose caps were worked out is a sound bar");

    assert_eq!(euler_characteristic_of(&computed, same), 2);
    let theirs = meshed(&declared, bar, 0.1);
    let ours = meshed(&computed, same, 0.1);
    assert_eq!(ours.points(), theirs.points());
    assert_eq!(
        ours.triangles().len(),
        theirs.triangles().len(),
        "computing a cap circle put a different number of triangles on the bar"
    );
}

/// **§24.18's criterion 8, and the shape M2 could not declare at all.** A bar
/// cut off at an angle, whose cap edge is an ellipse, meshes into a closed
/// surface.
///
/// §24.2 names this as the thing M3 reaches that M2 could not: an edge no
/// builder can write an `EdgeRun` for out of a person's knowledge, because
/// which conic a leaning plane cuts out of a pipe is a question rather than a
/// fact. It is also the first edge the old mesher could not have walked — a
/// straight line through the pipe's own two coordinates is a line of constant
/// height, and this one climbs.
///
/// The oracle is the same one every other mesh here passes, with no distance in
/// it: every side used both ways round, and χ of the mesh equal to χ of the
/// solid.
#[test]
fn a_bar_cut_off_at_an_angle_meshes_into_a_closed_surface() {
    let mut model = a_model();
    let leaning =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, HEIGHT), Vector::new(1.0, 0.0, 2.0))
            .expect("a direction with length");
    let bar = model
        .add_bar_cut_off_by(
            RADIUS,
            [a_cap_square_to_the_axis(0.0, -1.0), leaning],
            &Isometry::identity(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a leaning plane cuts an ellipse out of a pipe");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar cut off at an angle is a sound bar");
    assert_eq!(euler_characteristic_of(&model, bar), 2);

    let mesh = meshed(&model, bar, 0.05);
    let on_the_slope: Vec<f64> = mesh
        .points()
        .iter()
        .filter(|point| leaning.signed_distance_to(**point).abs() < 1e-9)
        .map(Point::z)
        .collect();
    let (lowest, highest) = on_the_slope
        .iter()
        .fold((f64::MAX, f64::MIN), |(low, high), &z| {
            (low.min(z), high.max(z))
        });
    assert!(
        (highest - lowest - RADIUS).abs() < 0.01,
        "a plane leaning one in two across a bar three across cuts an ellipse \
         spanning three in height, and this one spans {}",
        highest - lowest
    );

    for point in mesh.points() {
        let across = point.x().hypot(point.y());
        assert!(
            across <= RADIUS + 1e-9,
            "the mesh point {point:?} is outside the bar's wall"
        );
    }

    // §19 for the shape M3 added: where it sits does not change what it is.
    // A turn about the origin leaves the solid the same distance out, so a mesh
    // of the same size comes back; a move a thousand units away costs room off
    // the deflection and may take fewer pieces, and all that is asked of it is
    // the oracle, which `meshed` runs.
    for (placement, the_same_size) in [
        (
            Isometry::translation(Vector::new(17.5, -3.25, 1000.0)).expect("a shift"),
            false,
        ),
        (a_third_of_a_turn_about_a_slanted_axis(), true),
    ] {
        let mut moved = a_model();
        let solid = moved
            .add_bar_cut_off_by(
                RADIUS,
                [a_cap_square_to_the_axis(0.0, -1.0), leaning],
                &placement,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a bar cut off at an angle can be put anywhere");
        moved
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("and is still sound there");
        let there = meshed(&moved, solid, 0.05);
        assert!(!there.triangles().is_empty());
        if the_same_size {
            // The turn is about `(1, 1, 1)`, and §120's correctly rounded
            // normalize moved that stored direction — so the turned bar's
            // leaning cap is a rounding away from where it was and its arc
            // divides into 582 pieces where the untouched one divides into 566.
            // A turn cannot change how big a mesh has to be, and a count that
            // moved by a fortieth is the arc landing across a step rather than
            // the shape changing. What would not survive is a turn that cost or
            // saved a real share of the mesh, and that is what this asks.
            let there = there.triangles().len();
            let here = mesh.triangles().len();
            assert!(
                there * 10 > here * 9 && here * 10 > there * 9,
                "turning the bar came out a mesh of {there} triangles where it was {here}"
            );
        }
    }
}

/// The two caps of a bar have to cut right round it, face out of it, and leave
/// a body between them.
///
/// Every one of these is bad caller input rather than a broken shape, so every
/// one is an `Err` (§4.1) — and the first is the one the intersector answers
/// rather than the builder: a plane running down the inside of a pipe cuts two
/// straight lines, and no assumption in the builder had to know that.
#[test]
fn a_bar_whose_caps_are_not_caps_is_refused() {
    let down_the_middle = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
        .expect("a unit normal");
    let refused = |caps: [Plane; 2]| {
        a_model()
            .add_bar_cut_off_by(
                RADIUS,
                caps,
                &Isometry::identity(),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect_err("these two planes do not cut a bar")
    };

    assert!(matches!(
        refused([a_cap_square_to_the_axis(0.0, -1.0), down_the_middle]),
        Error::ACapDoesNotCutRightRoundABar { which: 1 }
    ));
    assert!(matches!(
        refused([
            a_cap_square_to_the_axis(0.0, 1.0),
            a_cap_square_to_the_axis(HEIGHT, 1.0),
        ]),
        Error::ACapOfABarFacesIntoIt { which: 0 }
    ));
    assert!(matches!(
        refused([
            a_cap_square_to_the_axis(HEIGHT, -1.0),
            a_cap_square_to_the_axis(0.0, 1.0),
        ]),
        Error::TheCapsOfABarMeetInsideIt
    ));
    // Square to each other at the top and bottom of the same seam, so they meet
    // over the far side of the bar and only just.
    assert!(matches!(
        refused([
            a_cap_square_to_the_axis(0.0, -1.0),
            Plane::from_point_and_normal(Point::new(0.0, 0.0, 1.0), Vector::new(1.0, 0.0, 0.2))
                .expect("a direction with length"),
        ]),
        Error::TheCapsOfABarMeetInsideIt
    ));
}

/// A coarse deflection is a request and not a failure (§23.5), so a coarse mesh
/// has to pass the same oracle a fine one does.
///
/// The coarse end is where it is most likely to break: a bar meshed to a
/// deflection wider than its own radius comes out as a few flat plates, and a
/// ball as a handful of triangles with a pole in each. It goes out to a
/// deflection a hundred thousand times the solid, because the claim audit found
/// a ball coming back with no triangles at all past about `6.6 · radius` and the
/// grid stopped short of it.
#[test]
fn a_coarse_mesh_closes_up_just_as_a_fine_one_does() {
    let mut model = a_model();
    let bar = a_round_bar(&mut model, &Isometry::identity());
    let ball = a_ball(&mut model, &Isometry::identity());

    let mut coarsest = 0;
    for deflection in [1e6, 100.0, 20.0, 4.0, 2.0, 1.0, 0.5, 0.2, 0.05, 0.01] {
        for solid in [bar, ball] {
            let mesh = meshed(&model, solid, deflection);
            if deflection == 1e6 {
                coarsest = coarsest.max(mesh.triangles().len());
            }
        }
    }
    assert!(
        coarsest < 64,
        "a deflection wider than the solid should give a handful of triangles, \
         and gave {coarsest}"
    );
}

/// A rough picture of a ball is a rough ball, and never nothing at all.
///
/// **The fourth of §23.8's silent wrong answers, found by the claim audit.** A
/// ball's one face is a rectangle in `(u, v)` whose top and bottom sides are
/// each a single point in space. The mesher sized a step by how far it went
/// round in *longitude*, and a step from pole to pole goes half a turn in
/// latitude and none at all in longitude — so past a deflection of about
/// `6.6 · radius` the seam came back one step long, every triangle of the
/// rectangle stood on one point twice, every one was dropped (§25.6), and
/// `mesh` handed back **two points and no triangles** with a deviation on it
/// that looked perfectly honest. `measure` then died on its own
/// `six_volumes > 0.0`, which is §25.5's defect a second time.
///
/// The floor is now the same quarter turn in either direction, so the coarsest
/// mesh of a ball there is is an octahedron.
#[test]
fn a_ball_asked_for_a_rough_picture_gets_a_rough_ball_and_not_an_empty_mesh() {
    let mut model = a_model();
    let ball = a_ball(&mut model, &Isometry::identity());
    for deflection in [7.0 * RADIUS, 100.0 * RADIUS, 1e12] {
        let mesh = meshed(&model, ball, deflection);
        assert!(
            mesh.triangles().len() >= 8,
            "a ball at a deflection of {deflection} came back with {} triangles",
            mesh.triangles().len()
        );
        let measured = model
            .solid(ball)
            .expect("this model made the key")
            .measure(
                Deflection::of(deflection).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a ball still weighs something");
        assert!(
            measured.volume() > 0.0,
            "a ball at a deflection of {deflection} weighs {}",
            measured.volume()
        );
    }
}

fn a_third_of_a_turn_about_a_slanted_axis() -> Isometry {
    let axis = Axis::new(Point::new(0.5, -1.5, 2.0), Vector::new(1.0, 1.0, 1.0))
        .expect("a direction with length");
    Isometry::rotation(&axis, core::f64::consts::TAU / 3.0).expect("a rotation")
}

/// §18 and §19 for meshes: a solid that has been moved and turned is still the
/// solid it was, and still meshes into a closed surface.
///
/// **A turned ball is the case that decides how this is built.**
/// `Sphere::transformed` under a turn about the centre moves nothing, so a
/// turned ball's seam is not a line of constant longitude of the turned sphere's
/// own coordinates and its face is not a rectangle in them. Meshed there it
/// would come back with holes where the coordinates pinch. `mesh` cuts a face up
/// in the frame it is stored in and moves the points afterwards, and this is
/// what says so.
#[test]
fn a_bar_and_a_ball_moved_and_turned_still_mesh_into_closed_surfaces() {
    let placements = [
        Isometry::identity(),
        Isometry::translation(Vector::new(17.5, -3.25, 1000.0)).expect("a shift"),
        a_third_of_a_turn_about_a_slanted_axis(),
        a_third_of_a_turn_about_a_slanted_axis()
            .compose(&Isometry::translation(Vector::new(-40.0, 8.0, 0.0)).expect("a shift")),
    ];
    for placement in &placements {
        let mut model = a_model();
        let bar = a_round_bar(&mut model, placement);
        let ball = a_ball(&mut model, placement);
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a placed bar and ball are sound");

        for (solid, what) in [(bar, "bar"), (ball, "ball")] {
            let mesh = meshed(&model, solid, 0.1);
            assert!(
                !mesh.triangles().is_empty(),
                "the {what} at {placement:?} meshed into nothing at all"
            );
        }

        // A ball's skin is the one thing a turn cannot change: every point of
        // the mesh is a radius from where the centre ended up.
        let centre = placement.apply_to_point(Point::origin());
        for point in meshed(&model, ball, 0.1).points() {
            assert!(
                (point.distance_to(centre) - RADIUS).abs() < 1e-9,
                "the mesh point {point:?} is not on the turned ball"
            );
        }
    }
}

/// A ball is cut open along the meridian its seam really lies in.
///
/// A seam with a pole at each end has no longitude its own ends can give — every
/// longitude names a pole — so `mesh` walks it at longitude zero, which is where
/// `add_ball` puts it: `y = 0` with `x ≥ 0`. This is the test that says the walk
/// lands on the model's own seam and not on some other meridian of the right
/// ball, and it is the only thing that would notice.
#[test]
fn the_seam_a_balls_mesh_is_cut_along_is_the_seam_the_model_has() {
    let mut model = a_model();
    let ball = a_ball(&mut model, &Isometry::identity());
    let mesh = meshed(&model, ball, 0.1);

    let on_the_meridian: Vec<_> = mesh
        .points()
        .iter()
        .filter(|point| point.y() == 0.0 && point.x() != 0.0)
        .collect();
    assert!(
        !on_the_meridian.is_empty(),
        "no point of this mesh is on the plane the seam lies in"
    );
    for point in &on_the_meridian {
        assert!(
            point.x() > 0.0,
            "the mesh is cut along the far half of the meridian at {point:?}, \
             and the seam is the half with x >= 0"
        );
    }
}

/// The mesher's order is fixed, so two runs are the same run (§25.7).
///
/// §9.2's benchmark compares a serial run against a parallel one bit for bit,
/// and every sum built on a shuffled mesh would come out different.
#[test]
fn meshing_the_same_solid_twice_gives_the_same_mesh() {
    let mut model = a_model();
    let ball = a_ball(&mut model, &a_third_of_a_turn_about_a_slanted_axis());
    assert_eq!(meshed(&model, ball, 0.1), meshed(&model, ball, 0.1));
}

/// Every triangle says which face it stands in for, and between them they cover
/// every face the solid has (§25.7).
#[test]
fn every_triangle_names_the_face_it_came_from() {
    let mut model = a_model();
    let bar = a_round_bar(&mut model, &Isometry::identity());
    let mesh = meshed(&model, bar, 0.1);

    let covered: HashSet<_> = mesh
        .triangles()
        .iter()
        .map(|triangle| triangle.face())
        .collect();
    let has: HashSet<_> = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .map(|face| face.key())
        .collect();
    assert_eq!(covered, has, "every face is stood in for and no other");
}

/// A bar too thin for its own squares is refused rather than asserted against.
///
/// `Model::add_round_bar` accepts a radius of `1e-200` — it is a positive finite
/// number and the builder makes the bar. Meshing it **panicked** until §39, on
/// ordinary public input with no file anywhere near it. (`Model::validate`
/// refuses such a bar `Undecidable` on *side of cylinder*, which is honest and
/// is not what this is about: `SolidView::mesh` is public in its own right and
/// nothing makes a caller sweep first.)
/// `Conic::angle_of` divides by `and.length_squared()`, which is `1e-400` and
/// underflows to zero, so the angle the walk turns over came back `0.0/0.0`. The
/// guard meant to catch that could not fire: `f64::max` hands back the other
/// operand when one is `NaN`, and the `.max(1.0)` beside it turned the `NaN`
/// into a one — so a whole cap circle was walked in a single step, with no
/// points at all between its ends, and the flat cap that reads those points back
/// fanned into one corner.
///
/// It is `rcad-geom`'s `CannotWorkOutHowBigACrossingIs` now, whose own doc
/// carries the classification: a **gap**, because the same bar written larger is
/// walked without complaint, and the two controls here are that bar.
#[test]
fn a_bar_too_thin_for_its_own_squares_is_refused_rather_than_asserted_against() {
    let mut model = a_model();
    let deflection = Deflection::of(1.0).expect("a length");

    for radius in [1.0, 1e-100] {
        let ordinary = model
            .add_round_bar(radius, 10.0, &Isometry::identity())
            .expect("a positive radius");
        let outcome = model
            .solid(ordinary)
            .expect("this model made the key")
            .mesh(deflection, &Policy::certified(), &mut ProvenanceLog::new());
        assert!(
            !matches!(outcome, Err(Error::Geometry(_))),
            "a bar of radius {radius} is walked, and got {outcome:?}"
        );
    }

    for radius in [1e-200, 1e-300] {
        let thin = model
            .add_round_bar(radius, 10.0, &Isometry::identity())
            .expect("a positive finite radius is a bar this builder makes");
        let refusal = model.solid(thin).expect("this model made the key").mesh(
            deflection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert!(
            matches!(
                refusal,
                Err(Error::Geometry(
                    rcad_geom::Error::CannotWorkOutHowBigACrossingIs
                ))
            ),
            "a bar of radius {radius} refuses by name, and got {refusal:?}"
        );
    }
}

/// §19 again, and the part of it that is a refusal rather than an answer.
///
/// A mesh point is put on the surface by an evaluator that rounds against how
/// far out the surface reaches, and is then carried into place by the solid's
/// own placement, which rounds against the same thing. So the floor under a
/// deflection is a fact about **where the solid sits**: the same ball a billion
/// units out cannot be meshed as finely as one at the origin, and the two ends
/// of this test are the same ball asked the same question.
#[test]
fn a_ball_a_long_way_out_cannot_be_meshed_as_finely_as_one_at_the_origin() {
    let asked = 1e-3;
    let mut model = a_model();
    let near = model
        .add_ball(1.0, &Isometry::identity())
        .expect("a unit ball at the origin");
    let far = model
        .add_ball(
            1.0,
            &Isometry::translation(Vector::new(1e12, 0.0, 0.0)).expect("a long way"),
        )
        .expect("a unit ball a long way out");

    let refusal = model.solid(far).expect("this model made the key").mesh(
        Deflection::of(asked).expect("a length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    let Err(Error::DeflectionBelowWhatAMeshCanReach { closest, .. }) = refusal else {
        panic!("a ball a billion out meshed to {asked}: {refusal:?}")
    };
    assert!(
        closest > asked && closest < 1.0,
        "the floor that far out came to {closest}"
    );

    // The control: the same ball at the origin, the same question, an answer.
    let mesh = model
        .solid(near)
        .expect("this model made the key")
        .mesh(
            Deflection::of(asked).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a unit ball at the origin meshes this finely");
    assert!(mesh.deviation().magnitude() <= asked);
}

/// The floor's **value**, and not only that there is one.
///
/// The test above brackets the floor of a ball a billion units out between the
/// deflection it refused and one whole unit, which is a factor of a thousand
/// wide. That was enough to say the floor moves with the placement and not
/// enough to say what it *is* — and `Error::DeflectionBelowWhatAMeshCanReach`
/// hands `closest` to callers, so it is a published number with nothing checking
/// it (§15.6).
///
/// # What is actually in doubt, and it is a factor of two
///
/// A consumer bisecting the floor of a ball at `1e14` cannot tell
/// `18 · f64::EPSILON · L` from `36 · f64::EPSILON · L` by trying deflections a
/// decade apart, and both readings have been written down. The second comes from
/// reading `the_surface_this_face_is_meshed_on`'s doc as though its `36 · ε · L`
/// were a term in the floor. It is not: that paragraph is the cost of a round
/// trip the mesher **does not make**, and it is why a placed face's surface is
/// read out of the arena instead. The floor charges one movement, of a mesh
/// *point*, in `how_far_placing_this_face_rounds` — so
/// `Isometry::POINT_ROUNDOFF` once, times how far the placed ball reaches.
///
/// # Which of the two floors this reads
///
/// `room_a_mesh_has_to_work_in` holds two numbers, and they are different jobs.
/// A deflection a caller named is let through against
/// `how_far_a_mesh_point_of_this_face_rounds`; the number *reported* on the
/// refusal, and the one a tolerant caller is cut at, is
/// `the_finest_deflection_the_walk_will_take`, which adds what a step at the
/// walk's own cap stands off its chord. Asking for `1e-30` is under both, so
/// what comes back here is the second — the one a caller ever sees.
///
/// **Both carry the same `how_far_moving_this_face_rounds`**, so the claim below
/// is about a term the two share, and moving the evaluator's end of it does not
/// move this. That is why the oracle is a closed form and not a recorded number:
/// the walk's share was re-derived and grew by about eighty, and none of the
/// assertions here had to change.
///
/// # The oracle
///
/// `one_placement` here is written the same way
/// `how_far_placing_this_face_rounds` writes it, over the same reach — the
/// ball's corners are its two poles, so the largest coordinate is `out` and the
/// radius goes on top — which makes it the same `f64` and lets the subtraction
/// below recover what is left.
///
/// **What is left is everything that is not the placement, and it is the same
/// number at every distance.** It is charged against how far the ball reaches in
/// the frame it is *meshed* in, and a placed ball is meshed in its own
/// placement's inverse — so it sees a ball of radius five at the origin however
/// far out the solid sits. A floor with a second placement term in it could not
/// have a distance-independent remainder, so this pins the coefficient at one
/// rather than merely bounding it under two.
///
/// # Not vacuous
///
/// The remainder is asserted to be **above zero**, which is the other side of
/// §15.6: a bound checked only from above passes if the term it is bounding
/// vanished. And every row has to have been refused at all — a distance where
/// the ball meshed at `1e-30` would hand back no floor and panic in
/// [`the_finest_mesh_this_solid_can_be_asked_for`].
#[test]
fn the_floor_under_a_placed_ball_is_one_placements_worth_of_rounding_and_not_two() {
    const RADIUS_OF_THE_BALL: f64 = 5.0;
    let mut model = a_model();

    let mut rows = Vec::new();
    for out in [1e6, 1e10, 1e12, 1e14] {
        let ball = model
            .add_ball(
                RADIUS_OF_THE_BALL,
                &Isometry::translation(Vector::new(out, 0.0, 0.0)).expect("a long way"),
            )
            .expect("a ball a long way out");
        let closest = the_finest_mesh_this_solid_can_be_asked_for(&model, ball);
        let one_placement = Isometry::POINT_ROUNDOFF * (out + RADIUS_OF_THE_BALL);
        println!(
            "a ball of radius {RADIUS_OF_THE_BALL} at {out}: floor {closest}, one placement's \
             worth of rounding {one_placement}, ratio {}",
            closest / one_placement
        );
        assert!(
            closest > one_placement,
            "a floor of {closest} at {out} is under the {one_placement} carrying a mesh point \
             out there costs, so something has stopped charging for the placement"
        );
        assert!(
            closest < 2.0 * one_placement,
            "a floor of {closest} at {out} is two placements' worth or more of the \
             {one_placement} one costs — the mesher is charging for a movement it does not make"
        );
        rows.push((out, closest, closest - one_placement));
    }

    let (_, nearest_floor, everything_but_the_placement) = rows[0];
    assert!(
        everything_but_the_placement > 0.0,
        "the floor is exactly one placement's worth and the walk's own share is nothing, \
         which would mean the term this subtracts against has gone"
    );
    for &(out, closest, share) in &rows {
        // Both sides of the subtraction are within a factor of two of each
        // other, so it is exact by Sterbenz's condition and the only error in
        // `share` is the rounding `closest` already carried when the mesher
        // added its two terms together — half an ulp of `closest`.
        let room = f64::EPSILON * (closest + nearest_floor);
        assert!(
            (share - everything_but_the_placement).abs() <= room,
            "at {out} the floor has {share} in it over one placement's worth, against \
             {everything_but_the_placement} at the nearest distance and {room} of room. \
             {WHAT_A_MOVED_PLACEMENT_TERM_MEANS}"
        );
    }
    println!(
        "what the floor holds beyond the placement, the same at every distance: \
         {everything_but_the_placement}"
    );

    // The slope of the floor against the distance the ball is placed at, taken
    // off the two ends. Everything that does not move with the placement cancels,
    // so this is the placement term's coefficient on its own and it does not
    // depend on `one_placement` above having guessed the reach right.
    //
    // The difference carries at most one ulp of the larger floor, and dividing by
    // a difference of two exactly-representable distances adds nothing, so the
    // slope is good to about `f64::EPSILON` of itself.
    let (near, near_floor, _) = rows[0];
    let (far, far_floor, _) = rows[rows.len() - 1];
    let per_unit_of_reach = (far_floor - near_floor) / (far - near);
    println!(
        "the floor rises {per_unit_of_reach} per unit the ball is placed out, which is {} \
         of an Isometry::POINT_ROUNDOFF",
        per_unit_of_reach / Isometry::POINT_ROUNDOFF
    );
    let room = 4.0 * f64::EPSILON * Isometry::POINT_ROUNDOFF;
    assert!(
        (per_unit_of_reach - Isometry::POINT_ROUNDOFF).abs() <= room,
        "the floor rises {per_unit_of_reach} per unit of placement, which is {} \
         Isometry::POINT_ROUNDOFFs rather than one, and {room} is all the room the \
         arithmetic needs. {WHAT_A_MOVED_PLACEMENT_TERM_MEANS}",
        per_unit_of_reach / Isometry::POINT_ROUNDOFF
    );
}

/// **Two different changes move the numbers this test pins, and from outside the
/// crate they look identical** — so whichever assertion fires says both, rather
/// than naming the one that was on this author's mind.
///
/// Either the floor grew a second placement term, or
/// `how_far_the_points_of_reach` stopped coming to `corners + radius` for a
/// ball's face and the reach written here went stale. Both scale the placement
/// term by a constant and neither can be told from the other by measuring floors.
/// The §15.6 reach under-charge is a live change of the second kind, which is why
/// this is spelled out rather than assumed.
const WHAT_A_MOVED_PLACEMENT_TERM_MEANS: &str = "\
    This means one of two things and they cannot be told apart from out here. \
    Either the floor has grown a second placement term, which is the defect this \
    test exists to catch — or how_far_the_points_of_reach no longer comes to \
    corners + radius for a ball's face, in which case the reach this test writes \
    down has gone stale and the fix is here rather than in the mesher. Read that \
    function's Sphere arm to tell which.";

/// §23.5's one job for the policy: what happens when the deviation cannot be
/// *proven* under the deflection asked for. Certified refuses; tolerant tags.
///
/// The tolerant caller has already said how far out of place it will let a point
/// be, and here the floor is inside that. So it gets the finest mesh the
/// arithmetic reaches, with [`Mesh::deviation`] saying what that came to — which
/// is wider than it asked for, and it is the tag that makes that honest rather
/// than a quiet miss.
#[test]
fn a_tolerant_caller_gets_the_finest_mesh_the_arithmetic_reaches() {
    let asked = 1e-7;
    let mut model = a_model();
    let bar = model
        .add_round_bar(
            1.0,
            0.01,
            &Isometry::translation(Vector::new(1e9, 0.0, 0.0)).expect("a long way"),
        )
        .expect("a thin bar a long way out");
    let deflection = Deflection::of(asked).expect("a length");

    assert!(
        matches!(
            model.solid(bar).expect("this model made the key").mesh(
                deflection,
                &Policy::certified(),
                &mut ProvenanceLog::new()
            ),
            Err(Error::DeflectionBelowWhatAMeshCanReach { .. })
        ),
        "a certified caller is refused a deflection under the floor"
    );

    let mesh = model
        .solid(bar)
        .expect("this model made the key")
        .mesh(
            deflection,
            &Policy::tolerant(1e9),
            &mut ProvenanceLog::new(),
        )
        .expect("a tolerant caller gets what the arithmetic reaches");
    assert!(
        mesh.deviation().magnitude() > asked,
        "a tolerant mesh under the floor strays further than it was asked to, \
         and this one says it strays {}",
        mesh.deviation().magnitude()
    );
    assert_eq!(mesh.deviation().cause(), Cause::FlatStandIn);
    the_mesh_closes_up(&model, bar, &mesh, mesh.deviation().magnitude());
}

/// A request for more points than can be held is refused rather than waited out
/// (§25.3).
///
/// Counted before the points are made, so this comes back at once. Above the
/// floor, so it is not the refusal the test before this one is about.
///
/// **Which cap answers moved when the mesher stopped walking its own edges.**
/// There are two: `MOST_POINTS_A_MESH_WILL_HOLD` over a whole solid, and
/// `rcad_geom::MOST_POINTS_ALONG_ONE_CROSSING` over one edge, which is four
/// times smaller. A ball has exactly one curved edge, so the second is what
/// answers here. They are the same refusal in the sense that matters, which is
/// that it comes back at once instead of running.
///
/// **Which of the two answers for a *different* solid is not settled here, and
/// the sentence that used to say it is was a forecast.** It read "and always
/// will be — the first would need five curved edges on one solid and no builder
/// makes a shape with more than three". The builder half is right as far as it
/// goes — counted through `EdgeView::run`, `add_ball` gives one curved edge and
/// `add_round_bar` two — but `mesh` takes any solid, not only a built one, and
/// the public API reaches further than the builders: `SolidBuilder` takes a
/// description of any shape, and a fuse writes solids nothing built. What would
/// settle it is a solid carrying five curved edges put to `mesh`. A block with
/// two round holes is the obvious one and this kernel does not make it yet —
/// both bars in one `Rule::Difference`, and one at a time, refuse with
/// `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` — so the forecast is dropped
/// rather than replaced.
#[test]
fn a_deflection_needing_more_points_than_can_be_held_is_refused() {
    let mut model = a_model();
    let ball = model
        .add_ball(1.0, &Isometry::identity())
        .expect("a unit ball");

    let refusal = model.solid(ball).expect("this model made the key").mesh(
        Deflection::of(1e-12).expect("a length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(
            refusal,
            Err(Error::Geometry(
                rcad_geom::Error::TooManyPointsAlongOneCrossing { .. }
            ))
        ),
        "a ball meshed to a picometre gave {refusal:?}"
    );
}

/// The oracle over a grid rather than over one shape: every size, every
/// coarseness, every placement.
///
/// `docs/decisions.md` §13 rule 5 wants a generator whose shapes are not the
/// ones any other lane draws, and this one is a bar and a ball across four
/// orders of magnitude of size and from a deflection a thousand times the solid
/// down to a fiftieth of it. Both refusals below are honest — a request for more
/// points than a mesh will hold, and a deflection under what a point can be
/// placed to — so a refusal counts as a pass and only a wrong answer does not.
#[test]
fn every_size_of_bar_and_ball_at_every_coarseness_meshes_into_a_closed_surface() {
    let placements = [
        Isometry::identity(),
        a_third_of_a_turn_about_a_slanted_axis(),
        Isometry::translation(Vector::new(250.0, -80.0, 12.5)).expect("a shift"),
        Isometry::rotation(&Axis::x(), 0.7)
            .expect("a turn")
            .compose(&Isometry::translation(Vector::new(-13.0, 4.0, 9.0)).expect("a shift")),
    ];
    let mut checked = 0;
    for placement in &placements {
        for radius in [0.01, 1.0, 100.0] {
            for part_of_the_radius in [1e3, 20.0, 2.0, 0.5, 0.1, 0.02] {
                let deflection = Deflection::of(radius * part_of_the_radius).expect("a length");
                for height in [0.001, 1.0, 20.0] {
                    let mut model = a_model();
                    let bar = model
                        .add_round_bar(radius, height, placement)
                        .expect("a bar");
                    if let Ok(mesh) = model.solid(bar).expect("this model made the key").mesh(
                        deflection,
                        &Policy::certified(),
                        &mut ProvenanceLog::new(),
                    ) {
                        the_mesh_closes_up(&model, bar, &mesh, deflection.length());
                        checked += 1;
                    }
                }
                let mut model = a_model();
                let ball = model.add_ball(radius, placement).expect("a ball");
                if let Ok(mesh) = model.solid(ball).expect("this model made the key").mesh(
                    deflection,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                ) {
                    the_mesh_closes_up(&model, ball, &mesh, deflection.length());
                    checked += 1;
                }
            }
        }
    }
    assert!(
        checked > 150,
        "only {checked} of the grid meshed at all, which is not a sweep"
    );
}

/// §31.7 criterion 13, for the shape criterion 11 is the headline of: a fused
/// curved solid meshes and passes §23.4's oracle in full.
///
/// Its answer has the first face in this kernel that is a **ring** — a tube of
/// the bar's wall between the block's two planes, bounded by two curves that
/// each go right round the cylinder and by nothing else. It has no seam and
/// cannot be given one: a seam wants two corners at a single longitude, and the
/// two curves bounding a tube start wherever the pair that cut each of them
/// starts. So the seam is put in by the mesher, four times, as a cut in the
/// surface's own two coordinates rather than as topology anybody has to hold.
#[test]
fn a_block_with_a_round_hole_meshes_into_a_closed_surface() {
    let hole = core::f64::consts::PI * 8.0;
    for (rule, what, should_be) in [
        (
            Rule::Difference,
            "a block with a hole through it",
            512.0 - hole,
        ),
        (
            Rule::Union,
            "a block with a bar sticking out of both sides",
            512.0 + core::f64::consts::PI * 4.0,
        ),
        (
            Rule::Intersection,
            "the bit of the bar inside the block",
            hole,
        ),
    ] {
        let mut model = a_model();
        let block = model
            .add_cube(8.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a block eight across");
        let bar = model
            .add_round_bar(
                1.0,
                12.0,
                &Isometry::translation(Vector::new(4.0, 4.0, -2.0)).expect("a shift"),
            )
            .expect("a bar through the middle of it");

        let mut log = ProvenanceLog::new();
        let fused = model
            .fuse(&[block], &[bar], rule, &Policy::certified(), &mut log)
            .unwrap_or_else(|why| panic!("{what}: {why}"));
        assert!(
            log.records().is_empty(),
            "{what}: a lattice-aligned curved pair needs nothing approximated, got {:?}",
            log.records()
        );

        let solids = fused.solids();
        assert_eq!(solids.len(), 1, "{what} is one solid");
        for &solid in solids {
            let mesh = meshed(&model, solid, 0.05);
            the_mesh_closes_up(&model, solid, &mesh, 0.05);

            // The volume is the strongest thing there is to say about a
            // curved answer, and it is what says the tube is not inside out:
            // a wall wound the wrong way takes its own volume off twice.
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.01).expect("a length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what} measures: {why}"));
            let out_by = (measured.volume() - should_be).abs();
            assert!(
                out_by < 0.02 * should_be,
                "{what}: {} against the {should_be} a block of eight and a bar of radius one \
                 come to, which is {out_by} out",
                measured.volume()
            );
        }
    }
}

/// §31.7 criterion 13's other half: a **ball cut by a plane**, which is about
/// the simplest curved boolean there is.
///
/// The answer is a spherical cap and the flat disk that closes it. The cap is
/// bounded by one circle of latitude and by nothing else, so its patch in the
/// sphere's own two coordinates is a rectangle whose top edge is the pole and
/// whose two sides are one meridian — and the model says neither, because a fuse
/// has no way to invent a seam or a pole step.
///
/// The volume has a closed form and it is the strongest thing to check: a cap of
/// height `h` on a ball of radius `R` holds `πh²(3R − h)/3`, and a cap wound the
/// wrong way would take its own volume off twice.
#[test]
fn a_ball_cut_by_a_plane_meshes_into_a_closed_surface() {
    const RADIUS: f64 = 4.0;
    // **Both poles, and that is the point of the table rather than tidiness.**
    // Which pole a cap reaches is read off which way its one loop runs, and an
    // arm that read it backwards would still mesh a *north* cap into a closed
    // surface of the right volume — it would simply never be asked. The south
    // rows are where getting it wrong does not cancel.
    for (cut_at, keep_above, what) in [
        (1.0, true, "the cap above a cut through the top"),
        (-2.0, true, "the cap above a cut through the bottom"),
        (1.0, false, "the cap below a cut through the top"),
        (-2.0, false, "the cap below a cut through the bottom"),
    ] {
        let mut model = a_model();
        let ball = model
            .add_ball(RADIUS, &Isometry::identity())
            .expect("a ball of radius four");
        // A slab on one side of the cut plane, wide enough to hold the ball.
        let block = model
            .add_cube(20.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a block round it");
        let below = if keep_above { cut_at } else { cut_at - 20.0 };
        model
            .place_solid(
                block,
                &Isometry::translation(Vector::new(-10.0, -10.0, below)).expect("a shift"),
            )
            .expect("this model made the key");

        let mut log = ProvenanceLog::new();
        let fused = model
            .fuse(
                &[ball],
                &[block],
                Rule::Intersection,
                &Policy::certified(),
                &mut log,
            )
            .unwrap_or_else(|why| panic!("{what}: {why}"));
        assert!(
            log.records().is_empty(),
            "{what}: nothing here had to be approximated, got {:?}",
            log.records()
        );

        let solids = fused.solids();
        assert_eq!(solids.len(), 1, "{what} is one solid");
        let solid = solids[0];
        let mesh = meshed(&model, solid, 0.02);
        the_mesh_closes_up(&model, solid, &mesh, 0.02);

        let height = if keep_above {
            RADIUS - cut_at
        } else {
            RADIUS + cut_at
        };
        let should_be = core::f64::consts::PI * height * height * (3.0 * RADIUS - height) / 3.0;
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} measures: {why}"));
        let out_by = (measured.volume() - should_be).abs();
        assert!(
            out_by < 0.01 * should_be,
            "{what}: {} against the {should_be} a cap of height {height} on a ball of {RADIUS} \
             comes to, which is {out_by} out",
            measured.volume()
        );
    }
}

/// §31.7 criterion 13 for the shape §36.19 left open: a **bar notched by a
/// block**, whose answer has faces trimmed by *part* of a curved crossing.
///
/// Two corners on a cap circle leave two arcs with the same two ends, and until
/// this lane naming which one was `ACurvedEdgeThatIsPartOfACrossingCannotBeNamed`.
/// The volume is what says the right one was named: taking the other one round
/// the cylinder would leave a wall that bounds the wrong side of the notch.
///
/// **The second row is where getting it wrong would not cancel**, and it is the
/// row this test exists for. Every arc a block leaves on a bar of this size is
/// shorter than half a turn, so a picker that always took the shorter of the two
/// would pass the first row and never be asked. The wedge leaves one of **228
/// degrees**: joined out of three stretches, longer than its own sibling, and
/// the only arc in this file where "the shorter one" is the wrong answer.
#[test]
fn a_bar_notched_by_a_block_meshes_into_a_closed_surface() {
    const RADIUS: f64 = 2.0;
    // Where the wedge's two faces cut the bar's wall, which is `(−1 + √7)/2`.
    let cut_at = (7.0_f64.sqrt() - 1.0) / 2.0;
    let wedge_takes = 2.0
        * ((cut_at + 1.0) * (cut_at + 1.0) + 4.0 * (cut_at / 2.0).acos()
            - cut_at * (4.0 - cut_at * cut_at).sqrt());
    for (what, wedge, taken) in [
        (
            "a bar notched by a block",
            false,
            // A slab two thick over the part of the disk with `x ≥ 1` and
            // `|y| ≤ 1`, which is `∫√(4 − y²) dy − 2` across that strip.
            2.0 * (3.0_f64.sqrt() + 2.0 * core::f64::consts::PI / 3.0 - 2.0),
        ),
        ("a bar notched by a wedge", true, wedge_takes),
    ] {
        let mut model = a_model();
        let bar = model
            .add_round_bar(RADIUS, 8.0, &Isometry::identity())
            .expect("a bar of radius two");
        let cutter = if wedge {
            let outline = ProfileBuilder::new(vec![[-1.0, 0.0], [5.0, 6.0], [5.0, -6.0]])
                .build(&Policy::certified(), &mut ProvenanceLog::new())
                .expect("a wedge with its point on the bar's axis side");
            model
                .add_extrusion(
                    &outline,
                    2.0,
                    &Isometry::translation(Vector::new(0.0, 0.0, 3.0)).expect("a shift"),
                )
                .expect("a wedge two thick")
        } else {
            let block = model
                .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
                .expect("a small block");
            model
                .place_solid(
                    block,
                    &Isometry::translation(Vector::new(1.0, -1.0, 3.0)).expect("a shift"),
                )
                .expect("this model made the key");
            block
        };

        let mut log = ProvenanceLog::new();
        let fused = model
            .fuse(
                &[bar],
                &[cutter],
                Rule::Difference,
                &Policy::certified(),
                &mut log,
            )
            .unwrap_or_else(|why| panic!("{what}: {why}"));
        assert!(
            log.records().is_empty(),
            "{what}: a lattice-aligned pair needs nothing approximated, got {:?}",
            log.records()
        );
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what} validates: {why}"));

        let solids = fused.solids();
        assert_eq!(solids.len(), 1, "{what} is one solid");
        let solid = solids[0];
        let mesh = meshed(&model, solid, 0.02);
        the_mesh_closes_up(&model, solid, &mesh, 0.02);

        let should_be = core::f64::consts::PI * RADIUS * RADIUS * 8.0 - taken;
        let measured = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.001).expect("a length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what} measures: {why}"));
        let out_by = (measured.volume() - should_be).abs();
        assert!(
            out_by < 0.001 * should_be,
            "{what}: {} against the {should_be} the bar less the notch comes to, which is \
             {out_by} out",
            measured.volume()
        );
    }
}

/// §39.7 item 2, and the seventh of §39.4's undemonstrated refusals — the only
/// one of the seven that is four public calls away rather than a hand-built
/// broken model.
///
/// **A boolean that removes no material must not make the answer coarser than
/// the input.** A bar built a billion units up its own axis has a mesh floor,
/// because a point put on its skin is carried into place by a movement that
/// rounds against the billion (`a_ball_a_long_way_out_cannot_be_meshed_as_finely
/// _as_one_at_the_origin` is that on its own). Cutting it with a block fifty
/// units away removes nothing at all — and used to raise that floor **77-fold**,
/// from `4.00e-6` to `3.07e-4`, because the fuse bakes placement into the
/// surface it writes (§17.3) and the crossing evaluator then charges its
/// rounding against the billion instead of against the five the bar is long.
/// Measured, the ratio was `76.85181687573629`, which is
/// `(POINT_ALONG_A_CROSSING_ROUNDOFF / 0.1875 + POINT_ROUNDOFF) / POINT_ROUNDOFF`
/// to six figures.
///
/// The mesher now works a face stored far from the origin out in a frame it
/// shifts it into, and the shift is exact, so what is left is the neighbouring
/// surfaces the shift moves without asking:
/// `(POINT_ROUNDOFF + SURFACE_AFTER_SHIFTING_ROUNDOFF) / POINT_ROUNDOFF`, which
/// is `26/18` and is what this holds it to.
///
/// Both halves are asserted from the front door and neither is a tolerance: the
/// floor is read out of the refusal's own `closest`, and the deflection §39.3
/// says used to be refused is asked for and answered.
#[test]
fn a_boolean_that_cuts_nothing_off_a_bar_does_not_make_it_much_coarser_to_mesh() {
    let a_billion = 1e9;
    let mut model = a_model();
    let bar = model
        .add_round_bar(
            1.0,
            5.0,
            &Isometry::translation(Vector::new(0.0, 0.0, a_billion)).expect("a long way"),
        )
        .expect("a bar a billion units up its own axis");
    let before = the_finest_mesh_this_solid_can_be_asked_for(&model, bar);
    let whole = volume_of(&model, bar);

    let block = model
        .add_cube(8.0, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a block eight across");
    model
        .place_solid(
            block,
            &Isometry::translation(Vector::new(50.0, 0.0, a_billion)).expect("a long way"),
        )
        .expect("this model made the key");
    let fused = model
        .fuse(
            &[bar],
            &[block],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a block fifty units away cuts nothing off the bar");
    let solids = fused.solids();
    assert_eq!(solids.len(), 1, "nothing was cut, so one solid comes back");
    let cut = solids[0];

    let after = the_finest_mesh_this_solid_can_be_asked_for(&model, cut);
    let coarser = after / before;
    println!("mesh floor {before:e} before the fuse, {after:e} after, {coarser} times coarser");
    assert!(
        (1.0..=26.0 / 18.0).contains(&coarser),
        "a boolean that cut nothing made the bar {coarser} times coarser to \
         mesh, against the 26/18 the shift into the meshing frame costs"
    );

    // Not bit for bit: the same triangles are summed in two frames, so the two
    // answers differ in the last place or two and in nothing else.
    let kept = volume_of(&model, cut);
    assert!(
        (kept - whole).abs() <= 8.0 * f64::EPSILON * whole,
        "the point of this test is a boolean that removed no material, and it \
         went from {whole} to {kept}"
    );

    // §39.3's own two numbers, from the other side: this used to refuse
    // everything under 3.07e-4, and 1e-5 is what the bar meshed at before the
    // fuse and must mesh at after it.
    let asked = 1e-5;
    assert!(before < asked, "the bar itself meshes at {asked}");
    let mesh = model
        .solid(cut)
        .expect("this model made the key")
        .mesh(
            Deflection::of(asked).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar a boolean cut nothing off meshes as finely as the bar did");
    the_mesh_closes_up(&model, cut, &mesh, asked);
}

/// The floor under a deflection, read out of the refusal that publishes it.
///
/// Asking for a length no arithmetic reaches and reading `closest` off the
/// refusal is the only way a caller can find out, which is why
/// [`Error::DeflectionBelowWhatAMeshCanReach`] carries it.
fn the_finest_mesh_this_solid_can_be_asked_for(model: &Model, solid: SolidKey) -> f64 {
    let refusal = model.solid(solid).expect("this model made the key").mesh(
        Deflection::of(1e-30).expect("a positive length"),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    let Err(Error::DeflectionBelowWhatAMeshCanReach { closest, .. }) = refusal else {
        panic!("a curved solid a billion units out has a floor, and got {refusal:?}")
    };
    assert!(
        closest > 0.0 && closest.is_finite(),
        "a floor of {closest} is not a floor"
    );
    closest
}

/// How much a solid holds, worked out over a mesh fine enough to be steady.
fn volume_of(model: &Model, solid: SolidKey) -> f64 {
    model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar a billion units out measures")
        .volume()
}

/// How many random bars and balls the property below meshes by default.
///
/// [`every_size_of_bar_and_ball_at_every_coarseness_meshes_into_a_closed_surface`]
/// is a fixed grid, so a soak cannot widen it. This draws from the same ranges
/// and wider placements, and the count keeps it to well under a second of the
/// suite; a soak is [`SOAK_FACTOR`] times this.
const CASES: u32 = 256;

/// The environment variable that multiplies [`CASES`] for a soak, read the way
/// `fuses_that_must_agree.rs` reads it.
///
/// ```text
/// RCAD_SOAK_FACTOR=50 cargo nextest run -p rcad-topo meshing_a_curved_solid::
/// ```
const SOAK_FACTOR: &str = "RCAD_SOAK_FACTOR";

/// The environment variable that reseeds the property below. Unset, the seed is
/// fixed and a failure reproduces by running the same command again.
const SOAK_SEED: &str = "RCAD_SOAK_SEED";

/// §176.4's gap closed for this file: every mesh of a random bar or ball passes
/// §23.4's oracle, and **no point of any triangle is further from the true
/// surface than the deviation the mesh publishes**.
///
/// The true surface is written down here from the numbers the solid was built
/// with — a sphere about where the placement put the centre, a cylinder about
/// where it put the axis, and the two cap planes — and never read back from the
/// model, so the distance is not the mesher checking itself. A ball is measured
/// exactly: its furthest point from the centre is a corner and its nearest is
/// the triangle's closest point. A bar's wall is measured at a grid of points
/// over each triangle, which can miss the worst point but never invents one, so
/// any failure is real.
///
/// Refusals for a deflection under the floor or for more points than a mesh
/// holds are honest and counted; any other refusal fails.
#[test]
fn every_triangle_of_a_random_bar_or_ball_stays_within_the_deviation_its_mesh_reports() {
    let asked = Cell::new(0_u32);
    let answered = Cell::new(0_u32);
    let closest_to_the_bound = Cell::new(0.0_f64);
    run(
        (a_random_bar_or_ball(), log_uniform(0.02, 1e3)),
        |(drawn, share_of_the_radius)| {
            asked.set(asked.get() + 1);
            let placement = drawn.placement();
            let mut model = a_model();
            let solid = drawn.built(&mut model, &placement);
            let deflection = drawn.radius * share_of_the_radius;
            let outcome = model.solid(solid).expect("this model made the key").mesh(
                Deflection::of(deflection).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            let mesh = match outcome {
                Ok(mesh) => mesh,
                Err(
                    Error::DeflectionBelowWhatAMeshCanReach { .. }
                    | Error::MoreMeshPointsThanASolidWillHold { .. }
                    | Error::Geometry(rcad_geom::Error::TooManyPointsAlongOneCrossing { .. }),
                ) => return Ok(()),
                Err(why) => {
                    return Err(TestCaseError::fail(format!(
                        "{drawn:?} at a deflection of {deflection}: {why}"
                    )));
                }
            };
            answered.set(answered.get() + 1);
            the_mesh_closes_up(&model, solid, &mesh, deflection);

            let claimed = mesh.deviation().magnitude();
            let truth = TheTrueSurfaces::of(&drawn, &placement);
            let flat: HashSet<_> = model
                .solid(solid)
                .expect("this model made the key")
                .faces()
                .filter(|face| face.surface().as_plane().is_some())
                .map(|face| face.key())
                .collect();
            for triangle in mesh.triangles() {
                let corners = triangle
                    .corners()
                    .map(|corner| mesh.points()[corner as usize]);
                let strayed =
                    truth.how_far_a_triangle_strays(corners, flat.contains(&triangle.face()));
                if claimed > 0.0 {
                    closest_to_the_bound.set(closest_to_the_bound.get().max(strayed / claimed));
                }
                prop_assert!(
                    strayed <= claimed + truth.what_this_check_rounds_by,
                    "{drawn:?} at a deflection of {deflection}: the triangle {corners:?} strays \
                     {strayed} from the true surface, past the {claimed} the mesh reports and \
                     the {} this check's own arithmetic can be out by",
                    truth.what_this_check_rounds_by
                );
            }
            Ok(())
        },
    );
    println!(
        "{} of {} random bars and balls meshed; the worst triangle came to {:.3} of the \
         deviation its mesh reported",
        answered.get(),
        asked.get(),
        closest_to_the_bound.get()
    );
    assert!(
        answered.get() * 2 >= asked.get(),
        "only {} of {} random bars and balls meshed, which is not a sweep",
        answered.get(),
        asked.get()
    );
    assert!(
        closest_to_the_bound.get() > 0.5,
        "no triangle came within half of the deviation its mesh reported (the worst was {}), \
         so the bound is not being tested from below",
        closest_to_the_bound.get()
    );
}

/// **A defect the soak above found, kept by its own numbers.** A bar of radius
/// one and ten tall, its top cap leaning five across for every one up, meshed
/// at a tenth, used not to close up into one surface.
///
/// The leaning cap cuts the bar in a long ellipse, from five up to fifteen. Near
/// its top end, where it turns sharpest, three points in a row along it — call
/// them the first, the middle and the last — were cut off as an ear by **both**
/// faces that meet there: the cap had the triangle middle–last–first and the
/// wall had middle–first–last. Those are the same three points facing opposite
/// ways, and the middle point was in no other triangle. So the pair was a flat
/// pocket with nothing inside it, pinched off from the rest of the mesh along
/// the first–last side, which the mesh then used four times — twice each way,
/// the one pattern [`the_mesh_closes_up`]'s balance check says it lets through
/// and its χ check catches. χ came out 3 rather than a bar's 2, and `mesh`
/// returned it as `Ok`.
///
/// The cap meets the wall there at about eleven degrees, so the two faces see
/// the same three points as a natural ear. The triangulator now never lets a
/// curved face draw a side straight across one corner of its boundary: it cuts
/// that side in half, which puts a point of the wall off the plane of the
/// ellipse (`docs/decisions.md` §187.4).
#[test]
fn a_bar_whose_top_leans_five_across_for_one_up_meshes_into_one_closed_surface() {
    let mut model = a_model();
    let bar = model
        .add_bar_cut_off_by(
            1.0,
            [
                a_cap_square_to_the_axis(0.0, -1.0),
                Plane::from_point_and_normal(Point::new(0.0, 0.0, 10.0), Vector::new(5.0, 0.0, 1.0))
                    .expect("a direction with length"),
            ],
            &Isometry::identity(),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a cap leaning five in one, through the axis ten up, cuts right round a bar of radius one");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar cut off at a steep angle is a sound bar");
    assert_eq!(euler_characteristic_of(&model, bar), 2);
    meshed(&model, bar, 0.1);
}

/// The bar above at every whole lean from two, where it never went wrong, to
/// twenty, at coarse and fine deflections, and once more small, turned and moved
/// two thousand out the way the soak's own find was. Every side of every mesh is
/// used by exactly two triangles — the count §23.4 asks for, which is sharper
/// than [`the_mesh_closes_up`]'s balance and is the one the pocket broke.
///
/// The bar is `lean + 5` tall so that the low end of the ellipse stays five
/// above the bottom cap however far it leans.
#[test]
fn a_bar_leaning_anywhere_from_two_to_twenty_across_for_one_up_uses_every_side_twice() {
    let far_out_and_turned = Isometry::translation(Vector::new(2000.0, -1300.0, 700.0))
        .expect("a finite shift")
        .compose(&a_third_of_a_turn_about_a_slanted_axis());
    for lean in 2..=20 {
        let lean = f64::from(lean);
        for (radius, placement, deflections) in [
            (1.0, Isometry::identity(), &[0.02, 0.1, 0.3, 1.0][..]),
            (0.1, far_out_and_turned, &[0.003, 0.01, 0.03][..]),
        ] {
            let height = radius * (lean + 5.0);
            let mut model = a_model();
            let bar = model
                .add_bar_cut_off_by(
                    radius,
                    [
                        a_cap_square_to_the_axis(0.0, -1.0),
                        a_leaning_cap(height, lean),
                    ],
                    &placement,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a leaning cap that stays above the bottom cap cuts a bar");
            for &deflection in deflections {
                let mesh = meshed(&model, bar, deflection);
                let mut times_used: HashMap<(u32, u32), u32> = HashMap::new();
                for triangle in mesh.triangles() {
                    let [first, second, third] = triangle.corners();
                    for (from, to) in [(first, second), (second, third), (third, first)] {
                        *times_used.entry((from.min(to), from.max(to))).or_default() += 1;
                    }
                }
                for (side, used) in times_used {
                    assert_eq!(
                        used, 2,
                        "leaning {lean} at radius {radius}, meshed at {deflection}: \
                         the side {side:?} is used {used} times"
                    );
                }
            }
        }
    }
}

/// A ball, a bar, or a bar cut off at an angle, with where it is put.
#[derive(Clone, Debug)]
struct ARandomBarOrBall {
    kind: RoundKind,
    radius: f64,
    shift: [f64; 3],
    turn: Option<([f64; 3], f64)>,
}

#[derive(Clone, Copy, Debug)]
enum RoundKind {
    Ball,
    Bar {
        height: f64,
    },
    /// The top cap leans `lean` in `x` for every one in `z`, and never so far
    /// that the two caps meet inside the bar.
    BarCutOffAtAnAngle {
        height: f64,
        lean: f64,
    },
}

impl ARandomBarOrBall {
    fn built(&self, model: &mut Model, placement: &Isometry) -> SolidKey {
        match self.kind {
            RoundKind::Ball => model
                .add_ball(self.radius, placement)
                .expect("a ball of positive radius anywhere"),
            RoundKind::Bar { height } => model
                .add_round_bar(self.radius, height, placement)
                .expect("a bar of positive size anywhere"),
            RoundKind::BarCutOffAtAnAngle { height, lean } => model
                .add_bar_cut_off_by(
                    self.radius,
                    [
                        a_cap_square_to_the_axis(0.0, -1.0),
                        a_leaning_cap(height, lean),
                    ],
                    placement,
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .expect("a leaning plane that stays above the bottom cap cuts a bar"),
        }
    }

    fn placement(&self) -> Isometry {
        let shift = Isometry::translation(Vector::from(self.shift)).expect("a finite shift");
        match self.turn {
            None => shift,
            Some((direction, angle)) => shift.compose(
                &Isometry::rotation(
                    &Axis::new(Point::origin(), Vector::from(direction))
                        .expect("a direction with length"),
                    angle,
                )
                .expect("a finite turn"),
            ),
        }
    }
}

fn a_leaning_cap(height: f64, lean: f64) -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, height), Vector::new(lean, 0.0, 1.0))
        .expect("a direction with length")
}

/// Radii and heights over the ranges the fixed grid covers, placed anywhere from
/// the origin to a million units out, sometimes turned.
fn a_random_bar_or_ball() -> impl Strategy<Value = ARandomBarOrBall> {
    (
        0..3_u8,
        log_uniform(0.01, 100.0),
        log_uniform(0.001, 20.0),
        0.05..0.9_f64,
        a_shift(),
        proptest::option::of(a_turn()),
    )
        .prop_map(
            |(which, radius, height, share_of_the_steepest, shift, turn)| ARandomBarOrBall {
                kind: match which {
                    0 => RoundKind::Ball,
                    1 => RoundKind::Bar { height },
                    _ => RoundKind::BarCutOffAtAnAngle {
                        height,
                        lean: share_of_the_steepest * height / radius,
                    },
                },
                radius,
                shift,
                turn,
            },
        )
}

/// The surfaces a bar or ball is made of, worked out from what it was built with
/// and where it was put.
struct TheTrueSurfaces {
    radius: f64,
    /// Where the placement put the centre of a ball or the foot of a bar's axis.
    centre: Point,
    /// Where it put the bar's axis, `+z` before it was moved.
    axis: Vector,
    caps: Vec<(Point, Vector)>,
    ball: bool,
    /// A few roundings of the largest coordinate this check works with, which is
    /// all its own subtractions and products can be out by.
    what_this_check_rounds_by: f64,
}

impl TheTrueSurfaces {
    fn of(drawn: &ARandomBarOrBall, placement: &Isometry) -> Self {
        let centre = placement.apply_to_point(Point::origin());
        let axis = placement.apply_to_vector(Vector::new(0.0, 0.0, 1.0));
        let cap = |at: Point, normal: Vector| {
            (
                placement.apply_to_point(at),
                placement.apply_to_vector(normal.normalized().expect("a direction with length")),
            )
        };
        let (caps, height) = match drawn.kind {
            RoundKind::Ball => (Vec::new(), 0.0),
            RoundKind::Bar { height } => (
                vec![
                    cap(Point::origin(), Vector::new(0.0, 0.0, 1.0)),
                    cap(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, 1.0)),
                ],
                height,
            ),
            RoundKind::BarCutOffAtAnAngle { height, lean } => (
                vec![
                    cap(Point::origin(), Vector::new(0.0, 0.0, 1.0)),
                    cap(Point::new(0.0, 0.0, height), Vector::new(lean, 0.0, 1.0)),
                ],
                height,
            ),
        };
        let reach = Vector::from(drawn.shift).length() + 2.0 * (drawn.radius + height);
        Self {
            radius: drawn.radius,
            centre,
            axis,
            caps,
            ball: matches!(drawn.kind, RoundKind::Ball),
            what_this_check_rounds_by: 16.0 * f64::EPSILON * reach,
        }
    }

    /// How far the furthest point of the triangle is from the surface it stands
    /// in for.
    fn how_far_a_triangle_strays(&self, [a, b, c]: [Point; 3], on_a_cap: bool) -> f64 {
        if on_a_cap {
            // A flat triangle is furthest from a plane at one of its corners.
            return [a, b, c]
                .iter()
                .map(|corner| {
                    self.caps
                        .iter()
                        .map(|(at, normal)| (*corner - *at).dot(*normal).abs())
                        .fold(f64::INFINITY, f64::min)
                })
                .fold(0.0, f64::max);
        }
        if self.ball {
            let furthest = [a, b, c]
                .iter()
                .map(|corner| (*corner - self.centre).length())
                .fold(0.0_f64, f64::max);
            let nearest =
                (closest_point_on_a_triangle(self.centre, a, b, c) - self.centre).length();
            return (furthest - self.radius).max(self.radius - nearest);
        }
        const STEPS: u32 = 12;
        let mut worst = 0.0_f64;
        for i in 0..=STEPS {
            for j in 0..=STEPS - i {
                let (s, t) = (
                    f64::from(i) / f64::from(STEPS),
                    f64::from(j) / f64::from(STEPS),
                );
                let point = a + (b - a) * s + (c - a) * t;
                let from_the_foot = point - self.centre;
                let across = (from_the_foot - self.axis * from_the_foot.dot(self.axis)).length();
                worst = worst.max((across - self.radius).abs());
            }
        }
        worst
    }
}

/// The point of triangle `abc` nearest to `p`, copied from
/// `how_far_a_balls_mesh_strays_when_the_ball_sits_far_from_the_origin.rs`.
///
/// The seven cases are the triangle's Voronoi regions — three corners, three
/// sides, and the inside. Ericson, *Real-Time Collision Detection*, §5.1.5.
fn closest_point_on_a_triangle(p: Point, a: Point, b: Point, c: Point) -> Point {
    let ab = b - a;
    let ac = c - a;

    let ap = p - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let along_ab = d1 * d4 - d3 * d2;
    if along_ab <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return a + ab * (d1 / (d1 - d3));
    }

    let along_ac = d5 * d2 - d1 * d6;
    if along_ac <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return a + ac * (d2 / (d2 - d6));
    }

    let along_bc = d3 * d6 - d5 * d4;
    if along_bc <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }

    let total = along_ab + along_ac + along_bc;
    a + ab * (along_ac / total) + ac * (along_ab / total)
}

/// A move of nothing, of a few units, or of up to a million.
fn a_shift() -> impl Strategy<Value = [f64; 3]> {
    let how_far = prop_oneof![Just(0.0), log_uniform(1.0, 100.0), log_uniform(1e3, 1e6)];
    (how_far, a_direction()).prop_map(|(how_far, direction)| {
        let length = Vector::from(direction).length();
        direction.map(|component| component / length * how_far)
    })
}

fn a_turn() -> impl Strategy<Value = ([f64; 3], f64)> {
    (a_direction(), 0.0..core::f64::consts::TAU)
}

fn a_direction() -> impl Strategy<Value = [f64; 3]> {
    [-1.0..1.0_f64, -1.0..1.0_f64, -1.0..1.0_f64]
        .prop_filter("a direction with length", |direction| {
            Vector::from(*direction).length() > 0.1
        })
}

/// A number between `low` and `high` whose logarithm is uniform, so every decade
/// in the range is drawn as often as every other.
fn log_uniform(low: f64, high: f64) -> impl Strategy<Value = f64> {
    (low.log10()..high.log10()).prop_map(|exponent| 10_f64.powf(exponent))
}

/// Run `property` over [`measured_cases`] draws from the fixed seed, or the one
/// [`SOAK_SEED`] names.
///
/// **Nothing is persisted.** `fuses_that_must_agree.rs` writes a failing seed
/// into a tracked `proptest-regressions` file; this file does not, because the
/// seed is fixed and a failure reproduces by running the same command again, and
/// a regression file written beside a test file is a tracked file a soak edits.
fn run<T: core::fmt::Debug>(
    strategy: impl Strategy<Value = T>,
    property: impl Fn(T) -> Result<(), TestCaseError>,
) {
    let mut runner = TestRunner::new_with_rng(
        Config {
            cases: measured_cases(),
            failure_persistence: None,
            ..Config::default()
        },
        fresh_seeded_soak(),
    );
    if let Err(why) = runner.run(&strategy, property) {
        panic!("{why}");
    }
}

/// [`CASES`], multiplied by whatever [`SOAK_FACTOR`] says.
///
/// # Panics
///
/// If the variable is set to something that is not a whole number above zero,
/// or to one whose product is more than a run can count: a soak that silently
/// ran the default corpus would be worse than one that stopped.
fn measured_cases() -> u32 {
    let Ok(factor) = std::env::var(SOAK_FACTOR) else {
        return CASES;
    };
    let factor: u32 = factor
        .parse()
        .unwrap_or_else(|_| panic!("{SOAK_FACTOR} has to be a whole number, not {factor:?}"));
    assert!(factor > 0, "{SOAK_FACTOR} has to be at least one");
    CASES
        .checked_mul(factor)
        .unwrap_or_else(|| panic!("{CASES} cases times {factor} is more than one run can count"))
}

/// The fixed seed, or the one [`SOAK_SEED`] names.
///
/// # Panics
///
/// If the variable is set to something that is not a whole number.
fn fresh_seeded_soak() -> TestRng {
    let Ok(seed) = std::env::var(SOAK_SEED) else {
        return TestRng::deterministic_rng(RngAlgorithm::ChaCha);
    };
    let seed: u64 = seed
        .parse()
        .unwrap_or_else(|_| panic!("{SOAK_SEED} has to be a whole number, not {seed:?}"));
    let mut bytes = [0u8; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    TestRng::from_seed(RngAlgorithm::ChaCha, &bytes)
}
