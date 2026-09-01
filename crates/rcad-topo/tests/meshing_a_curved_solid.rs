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

use std::collections::{HashMap, HashSet};

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
    let faces: HashSet<_> = model.solid(solid).faces().map(|face| face.key()).collect();
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
    for face in model.solid(solid).faces() {
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
    // A turn about the origin leaves the solid the same distance out, so the
    // same mesh comes back; a move a thousand units away costs room off the
    // deflection and may take fewer pieces, and all that is asked of it is the
    // oracle, which `meshed` runs.
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
            assert_eq!(
                there.triangles().len(),
                mesh.triangles().len(),
                "turning the bar came out a different mesh"
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
    let has: HashSet<_> = model.solid(bar).faces().map(|face| face.key()).collect();
    assert_eq!(covered, has, "every face is stood in for and no other");
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

    let refusal = model.solid(far).mesh(
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
        .mesh(
            Deflection::of(asked).expect("a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a unit ball at the origin meshes this finely");
    assert!(mesh.deviation().magnitude() <= asked);
}

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
            model
                .solid(bar)
                .mesh(deflection, &Policy::certified(), &mut ProvenanceLog::new()),
            Err(Error::DeflectionBelowWhatAMeshCanReach { .. })
        ),
        "a certified caller is refused a deflection under the floor"
    );

    let mesh = model
        .solid(bar)
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
/// answers here and always will be — the first would need five curved edges on
/// one solid and no builder makes a shape with more than three. They are the
/// same refusal in the sense that matters, which is that it comes back at once
/// instead of running.
#[test]
fn a_deflection_needing_more_points_than_can_be_held_is_refused() {
    let mut model = a_model();
    let ball = model
        .add_ball(1.0, &Isometry::identity())
        .expect("a unit ball");

    let refusal = model.solid(ball).mesh(
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
                    if let Ok(mesh) = model.solid(bar).mesh(
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
                if let Ok(mesh) = model.solid(ball).mesh(
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
        model.place_solid(
            block,
            &Isometry::translation(Vector::new(-10.0, -10.0, below)).expect("a shift"),
        );

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
            model.place_solid(
                block,
                &Isometry::translation(Vector::new(1.0, -1.0, 3.0)).expect("a shift"),
            );
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
