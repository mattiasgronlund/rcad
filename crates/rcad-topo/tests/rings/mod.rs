//! Torus-faced solids described through the public builder, for the test files
//! that need one.
//!
//! One shape, and it is the one a mechanical file actually holds: a **bend of
//! pipe**. A quarter of a torus, capped at each end by a plane through the
//! torus's own axis.
//!
//! # A bend is a bar in its chart, and that is why this is the shape that works
//!
//! A round bar's curved face wraps in one coordinate — round the pipe — and runs
//! from one cap to the other in the second, with a straight seam joining them. A
//! bend's face wraps round the **tube** and runs from one cap to the other round
//! the ring, and [`Torus::point_at`](rcad_geom::Torus::point_at) puts the tube
//! first for exactly that reason. So a bend has a bar's structure: two closed
//! edges, one seam, two corners, three faces.
//!
//! Two things are not a bar's, and both are what a torus adds.
//!
//! **A cap edge is one of two circles and not the only one.** A plane through a
//! torus's axis cuts it in **two** circles, so a cap circle has to say which. It
//! says so with its own corner: an edge that goes right round names a whole
//! closed curve and carries one corner at both its ends, and a point on one of
//! two disjoint circles settles which without a picker at all.
//!
//! **The seam takes two pickers.** It runs along the inner equator, which is one
//! of the two circles the plane through the middle of the ring cuts, so it names
//! the pipe of the ring's own radius read from the **inside** — the only picker
//! in the kernel that says inside, and a torus is what needed it — and then a
//! flat half-space, which says *this quarter of that circle and not the other
//! three*.
//!
//! # Everything here is exact on purpose
//!
//! `Model::validate` asks that every corner of a curved face be **certified** on
//! its surface. A bend's corners sit on the inner equator at
//! `(R − r, 0, 0)` and `(0, R − r, 0)`, and the quartic there is
//! `((R−r)² + R² − r²)² − 4R²(R−r)²`, which is `(2R(R−r))² − (2R(R−r))²` — a
//! difference of two products that are equal number for number, so it cancels to
//! nothing at the exact level whatever `R` and `r` are.

#![allow(dead_code)]

use rcad_geom::{
    Axis, Cylinder, EdgeRun, HalfSpace, Orientation, Picker, Plane, Point, Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey};
use smallvec::smallvec;

/// A quarter bend about `+z`, centred on the origin: the tube of radius `tube`
/// round the circle of radius `ring`, from the `+x` cap to the `+y` cap.
///
/// Described face by face through [`SolidBuilder`], which is the door an
/// importer comes through — so this is §42.12 criterion 1 for a surface kind
/// that did not exist when that criterion was written.
pub fn a_quarter_bend(model: &mut Model, ring: f64, tube: f64) -> SolidKey {
    let skin = Torus::new(Axis::z(), ring, tube).expect("a tube narrower than its ring");
    let surface = Surface::Torus(skin);
    let mut builder = SolidBuilder::new();

    let inner = ring - tube;
    let start_corner = builder
        .add_vertex(Point::new(inner, 0.0, 0.0))
        .expect("a finite point");
    let end_corner = builder
        .add_vertex(Point::new(0.0, inner, 0.0))
        .expect("a finite point");

    // The seam runs along the inner equator from one cap to the other. Two
    // pickers: the first says which of the two circles the plane `z = 0` cuts,
    // and the second says which quarter of that circle.
    let flat_through_the_middle =
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let seam = builder
        .add_edge(
            start_corner,
            end_corner,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(flat_through_the_middle),
                picked_by: smallvec![
                    Picker::InsideAPipe(
                        Cylinder::new(Axis::z(), ring).expect("the ring's own radius")
                    ),
                    Picker::from(
                        HalfSpace::new(Vector::new(1.0, 1.0, 0.0), 0.0)
                            .expect("a normal with a length")
                    ),
                ],
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

    let cap = |builder: &mut SolidBuilder, facing: Vector, edge, turn| {
        let plane = Plane::from_point_and_normal(Point::origin(), facing)
            .expect("a coordinate direction is a unit normal");
        let coedge = builder
            .add_coedge(edge, turn)
            .expect("an edge of this build");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("one closed edge is a wire");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a cap is a plane trimmed by its circle")
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
        .add_face(surface, &[wire])
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
