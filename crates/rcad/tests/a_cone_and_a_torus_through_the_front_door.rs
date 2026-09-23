//! A cone-faced solid and a torus-faced solid, described with `rcad` as the
//! only dependency.
//!
//! The facade re-exported [`Plane`], [`Sphere`] and [`Cylinder`] and neither
//! [`Cone`] nor [`Torus`], so a caller matching a face's surface could reach
//! `Surface::Cone(c)` and then not name `c`'s type, not ask it its slope, and
//! not build one to describe a face with. Two of the five surfaces this kernel
//! supports were behind a door marked "implementation detail"
//! (`docs/decisions.md` §3.6, §7.2).
//!
//! The five [`SIDE_OF_PLANE`]-shaped constants are the other half of the same
//! shut door. Each is the name its surface's sign predicate goes by in a
//! [`Record`](rcad::Record), and exactly one of the six — [`SIDE_OF_A_HALF_SPACE`],
//! which is a picker rather than a surface — was out here. A caller reading its
//! own provenance log could compare `record.predicate()` against a string
//! literal it typed itself and against nothing else, which is the state §15.6
//! exists to stop: a published bound the caller cannot name.
//!
//! **Nothing in this file imports anything but `rcad`. That is the test.** If a
//! line here needs `rcad_geom`, the front door is still shut.

use rcad::{
    Axis, Cone, Cylinder, Deflection, DraftEdge, DraftFace, DraftStep, EdgeRun, HalfSpace, Model,
    Orientation, Picker, Plane, Point, Policy, ProvenanceLog, SIDE_OF_A_HALF_SPACE, SIDE_OF_CONE,
    SIDE_OF_CYLINDER, SIDE_OF_PLANE, SIDE_OF_SPHERE, SIDE_OF_TORUS, Scale, SolidBuilder, SolidKey,
    Sphere, Surface, Torus, Vector,
};

/// A slope of a half, so every corner's radius is an exact product — the same
/// reason `rcad-topo`'s own cone tests use a power of two.
const SLOPE: f64 = 0.5;
const HOW_FAR_UP_THE_RIM_SITS: f64 = 4.0;
const RING: f64 = 4.0;
const TUBE: f64 = 1.0;

fn a_model() -> Model {
    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"))
}

/// A cone that ends in a point, built and measured naming only `rcad`.
///
/// Two faces, one edge, one vertex and no seam: the apex is where the cone's
/// own second coordinate runs out rather than a corner of the model.
///
/// # A pointed cone has a finite published bound, and this pins that a caller
/// can reach it
///
/// The bound the kernel publishes for a curved face is an offset-surface
/// argument: the mesh sits between the surface and the same surface a deviation
/// inside the material. A cone has no such inner surface within a deviation of
/// its apex, so the smallest radius of curvature over the face is zero, and
/// `volume_error()` used to come back infinite for every solid holding an apex.
/// The tip is charged its whole size now instead — `what_a_cone_tip_costs` in
/// `rcad-topo`, and `crates/rcad-topo/tests/a_solid_with_a_cone_face.rs`
/// asserts the same thing from the inside.
///
/// That matters out here because it is the difference between a measurement a
/// caller can act on and one they cannot: `x <= infinity` holds for every
/// finite `x`, so the comparison this test now makes against the bound was an
/// assertion that could not fail. It fails now if the bound is wrong.
#[test]
fn a_pointed_cone_is_described_measured_and_named_from_the_front_door() {
    let mut model = a_model();
    let solid = a_pointed_cone(&mut model);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a pointed cone described this way is a sound solid");

    let skin = the_only_curved_surface_of(&model, solid);
    let Surface::Cone(cone) = skin else {
        panic!("the curved face of a cone is on a cone, and came back {skin:?}")
    };
    // The whole of what was missing: `cone` has a type a caller can write down.
    let cone: Cone = cone;
    assert_eq!(cone.slope(), SLOPE);
    assert_eq!(cone.apex(), Point::origin());
    assert_eq!(
        cone.radius_at(HOW_FAR_UP_THE_RIM_SITS),
        SLOPE * HOW_FAR_UP_THE_RIM_SITS
    );

    let deflection = 0.002;
    let how_fine = Deflection::of(deflection).expect("a positive length");
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(how_fine, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a pointed cone measures");
    let radius = SLOPE * HOW_FAR_UP_THE_RIM_SITS;
    let height = HOW_FAR_UP_THE_RIM_SITS;
    let volume = core::f64::consts::PI * radius * radius * height / 3.0;
    let area = core::f64::consts::PI
        * (radius * (radius * radius + height * height).sqrt() + radius * radius);
    // A statement about *this test* and not about the bound: the mesh is
    // inscribed in a convex solid, so its volume is below the true one, and the
    // deflection times the skin is what this test settled for before the bound
    // was finite.
    let room = deflection * area;
    println!(
        "a pointed cone at a deflection of {deflection}: volume {} against πr²h/3 = {volume}, \
         short by {}, with {room} of room and a published bound of {}",
        measured.volume(),
        volume - measured.volume(),
        measured.volume_error(),
    );
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "the mesh holds {} and πr²h/3 is {volume}, which the published bound of {} does not cover",
        measured.volume(),
        measured.volume_error()
    );
    assert!(
        measured.volume() <= volume && measured.volume_error() < room,
        "the mesh of a convex solid holds {} against {volume}, with a bound of {} that has to \
         beat the {room} this test used to settle for",
        measured.volume(),
        measured.volume_error()
    );
}

/// A quarter bend of pipe — a quarter of a torus capped at each end — built and
/// measured naming only `rcad`.
///
/// Its seam is the one place a caller has to build an [`EdgeRun`] holding
/// [`Picker`]s, and both kinds are here: a [`Cylinder`] saying which of the two
/// circles the middle plane cuts, and a [`HalfSpace`] saying which quarter of
/// that circle. Volume `2π²Rr²/4`, by Pappus.
#[test]
fn a_quarter_bend_is_described_measured_and_named_from_the_front_door() {
    let mut model = a_model();
    let solid = a_quarter_bend(&mut model);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a quarter bend described this way is a sound solid");

    let skin = the_only_curved_surface_of(&model, solid);
    let Surface::Torus(ring) = skin else {
        panic!("the curved face of a bend is on a torus, and came back {skin:?}")
    };
    let ring: Torus = ring;
    assert_eq!(ring.ring_radius(), RING);
    assert_eq!(ring.tube_radius(), TUBE);
    assert_eq!(ring.centre(), Point::origin());

    let how_fine = Deflection::of(0.002).expect("a positive length");
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(how_fine, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a quarter bend measures");
    let volume = 2.0 * core::f64::consts::PI * core::f64::consts::PI * RING * TUBE * TUBE / 4.0;
    // The bound is what this compares against, so it has to be a number. A
    // pointed cone's is infinity — see the test above — and against infinity the
    // line below would hold whatever the mesh had measured.
    assert!(
        measured.volume_error().is_finite(),
        "a torus stands a whole tube radius off its own axis, so it has an offset surface \
         everywhere and a bound to go with it, and this one came back {}",
        measured.volume_error()
    );
    println!(
        "a quarter bend: volume {} against 2π²Rr²/4 = {volume}, apart by {}, with a published \
         bound of {}",
        measured.volume(),
        (measured.volume() - volume).abs(),
        measured.volume_error(),
    );
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "the mesh holds {} and 2π²Rr²/4 is {volume}, further apart than the bound {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// Every surface the kernel supports can be asked which bound its sign was
/// decided against, **from out here**.
///
/// This is the match a caller reading its own [`ProvenanceLog`] has to write:
/// take the surface a face is on, name the predicate that decides sides of it,
/// and compare that against `record.predicate()`. Until all six names were
/// exported, five of the six arms could not be written at all — and a face on a
/// cone or a torus is one the kernel builds, so those were arms a caller
/// reached.
#[test]
fn every_surface_can_name_the_bound_its_sign_was_certified_against() {
    let mut model = a_model();
    let cone = a_pointed_cone(&mut model);
    let bend = a_quarter_bend(&mut model);

    assert_eq!(
        the_bound_for(&the_only_curved_surface_of(&model, cone)),
        SIDE_OF_CONE
    );
    assert_eq!(
        the_bound_for(&the_only_curved_surface_of(&model, bend)),
        SIDE_OF_TORUS
    );

    let flat = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal"),
    );
    assert_eq!(the_bound_for(&flat), SIDE_OF_PLANE);
    assert_eq!(
        the_bound_for(&Surface::Sphere(
            Sphere::new(Point::origin(), 1.0).expect("a unit ball")
        )),
        SIDE_OF_SPHERE
    );
    assert_eq!(
        the_bound_for(&Surface::Cylinder(
            Cylinder::new(Axis::z(), 1.0).expect("a pipe of radius one")
        )),
        SIDE_OF_CYLINDER
    );

    // A picker's bound is the sixth, and it is a different question from the
    // five above: which side of a flat wall, asked of a run rather than of a
    // face.
    let picker = Picker::from(
        HalfSpace::through_a_point_facing(Point::origin(), Vector::new(1.0, 1.0, 0.0))
            .expect("a normal with a length"),
    );
    assert!(matches!(picker, Picker::InFrontOf(_)));
    let six = [
        SIDE_OF_PLANE,
        SIDE_OF_SPHERE,
        SIDE_OF_CYLINDER,
        SIDE_OF_CONE,
        SIDE_OF_TORUS,
        SIDE_OF_A_HALF_SPACE,
    ];
    for (position, name) in six.iter().enumerate() {
        assert!(!name.is_empty());
        assert!(
            !six[position + 1..].contains(name),
            "two surfaces would be indistinguishable in a log: {name}"
        );
    }
}

/// The surface of the one face of `solid` that is not flat.
fn the_only_curved_surface_of(model: &Model, solid: SolidKey) -> Surface {
    let mut curved = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .filter(|surface| !matches!(surface, Surface::Plane(_)));
    let only = curved.next().expect("one curved face");
    assert!(curved.next().is_none(), "and only one");
    only
}

/// The name the predicate that decides sides of `surface` goes by in a record.
///
/// The kernel writes this match too, on the inside. A caller comparing a record
/// against it has to be able to write it out here.
fn the_bound_for(surface: &Surface) -> &'static str {
    match surface {
        Surface::Plane(_) => SIDE_OF_PLANE,
        Surface::Sphere(_) => SIDE_OF_SPHERE,
        Surface::Cylinder(_) => SIDE_OF_CYLINDER,
        Surface::Cone(_) => SIDE_OF_CONE,
        Surface::Torus(_) => SIDE_OF_TORUS,
        // `Surface` grows an arm every milestone, and a new one has a bound of
        // its own that this test has not been told about.
        other => panic!("a surface with no published bound out here: {other:?}"),
    }
}

/// A cone about `+z` with its apex at the origin, capped by a plane at
/// [`HOW_FAR_UP_THE_RIM_SITS`].
fn a_pointed_cone(model: &mut Model) -> SolidKey {
    let skin = Cone::new(Axis::z(), SLOPE).expect("a positive slope");
    let mut builder = SolidBuilder::new();

    let rim = builder
        .add_vertex(Point::new(
            SLOPE * HOW_FAR_UP_THE_RIM_SITS,
            0.0,
            HOW_FAR_UP_THE_RIM_SITS,
        ))
        .expect("a finite point");
    let circle = builder
        .add_edge(rim, rim, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the cap plane cuts the cone in one closed curve");

    let cap_plane = Plane::from_point_and_normal(
        Point::new(0.0, 0.0, HOW_FAR_UP_THE_RIM_SITS),
        Vector::new(0.0, 0.0, 1.0),
    )
    .expect("the z axis is a unit normal");
    let cap = a_face_bounded_by_one_closed_edge(
        &mut builder,
        Surface::Plane(cap_plane),
        circle,
        Orientation::Forward,
    );
    let side = a_face_bounded_by_one_closed_edge(
        &mut builder,
        Surface::Cone(skin),
        circle,
        Orientation::Reversed,
    );

    builder
        .add_shell(&[(cap, Orientation::Forward), (side, Orientation::Forward)])
        .expect("two faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a pointed cone described this way is a solid")
}

/// A quarter of a torus about `+z`, capped by a plane at each end.
fn a_quarter_bend(model: &mut Model) -> SolidKey {
    let skin = Torus::new(Axis::z(), RING, TUBE).expect("a tube narrower than its ring");
    let mut builder = SolidBuilder::new();

    let inner = RING - TUBE;
    let start_corner = builder
        .add_vertex(Point::new(inner, 0.0, 0.0))
        .expect("a finite point");
    let end_corner = builder
        .add_vertex(Point::new(0.0, inner, 0.0))
        .expect("a finite point");

    // The seam runs along the inner equator from one cap to the other. The pipe
    // says which of the two circles the plane `z = 0` cuts out of the torus, and
    // the half-space says which quarter of that circle.
    let flat_through_the_middle =
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let seam = builder
        .add_edge(
            start_corner,
            end_corner,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(flat_through_the_middle),
                picked_by: [
                    Picker::InsideAPipe(
                        Cylinder::new(Axis::z(), RING).expect("the ring's own radius"),
                    ),
                    Picker::from(
                        HalfSpace::new(Vector::new(1.0, 1.0, 0.0), 0.0)
                            .expect("a normal with a length"),
                    ),
                ]
                .into_iter()
                .collect(),
            },
        )
        .expect("a quarter of the inner equator");

    let mut cap_circle = |corner| {
        builder
            .add_edge(corner, corner, EdgeRun::RightRoundWhereItsFacesCross)
            .expect("one whole circle of the two this plane cuts")
    };
    let start_circle = cap_circle(start_corner);
    let end_circle = cap_circle(end_corner);

    let cap = |builder: &mut SolidBuilder, facing, edge, turn| {
        let plane = Plane::from_point_and_normal(Point::origin(), facing)
            .expect("a coordinate direction is a unit normal");
        a_face_bounded_by_one_closed_edge(builder, Surface::Plane(plane), edge, turn)
    };
    let start_cap = cap(
        &mut builder,
        Vector::new(0.0, -1.0, 0.0),
        start_circle,
        Orientation::Reversed,
    );
    let end_cap = cap(
        &mut builder,
        Vector::new(-1.0, 0.0, 0.0),
        end_circle,
        Orientation::Forward,
    );

    let round_the_wall: Vec<DraftStep> = [
        (start_circle, Orientation::Forward),
        (seam, Orientation::Forward),
        (end_circle, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ]
    .into_iter()
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("an edge of this build"),
        )
    })
    .collect();
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("round one cap, along the seam, back round the other, and down the seam");
    let side = builder
        .add_face(Surface::Torus(skin), &[wire])
        .expect("a face on the torus");

    builder
        .add_shell(&[
            (start_cap, Orientation::Forward),
            (end_cap, Orientation::Forward),
            (side, Orientation::Reversed),
        ])
        .expect("three faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bend described this way is a solid")
}

fn a_face_bounded_by_one_closed_edge(
    builder: &mut SolidBuilder,
    surface: Surface,
    edge: DraftEdge,
    turn: Orientation,
) -> DraftFace {
    let coedge = builder
        .add_coedge(edge, turn)
        .expect("an edge of this build");
    let wire = builder
        .add_wire(&[DraftStep::Along(coedge)])
        .expect("one closed edge is a wire");
    builder
        .add_face(surface, &[wire])
        .expect("a surface trimmed by one closed curve")
}
