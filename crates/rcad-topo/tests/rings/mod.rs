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
//! **Every torus face here is used forwards**, because a torus faces out of its
//! tube, and every loop turns right-handed about its own face's outward normal
//! — the way a file written as the standard says holds a bend
//! (`docs/decisions.md` §329, §361).
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
use rcad_topo::{DraftStep, Model, Result, SolidBuilder, SolidKey};
use smallvec::smallvec;

/// A quarter bend about `+z`, centred on the origin: the tube of radius `tube`
/// round the circle of radius `ring`, from the `+x` cap to the `+y` cap.
///
/// Described face by face through [`SolidBuilder`], which is the door an
/// importer comes through — so this is §42.12 criterion 1 for a surface kind
/// that did not exist when that criterion was written.
pub fn a_quarter_bend(model: &mut Model, ring: f64, tube: f64) -> SolidKey {
    a_bend(model, ring, tube, &WhereABendSits::quarter(ring - tube))
        .expect("a bend described this way is a solid")
}

/// A bend `quarters` quarter turns long round the ring — one or two — from the
/// `+x` cap, turned `turns` quarter turns about its own axis and **written out
/// where it sits** rather than placed there.
///
/// Every number of the bend is put through the exact quarter turn
/// `(x, y) → (−y, x)`, `turns` times: both corners, both cap normals, and the
/// seam's half-space normal. The edges, the coedge directions and the shell's
/// orientations stay as they are, so this is the same solid turned. Zeros are
/// written `+0.0`, the way a person writing the numbers out would.
///
/// **Why written out and not placed.** A face with a placement is meshed in the
/// frame it is stored in, so a placed bend is meshed unturned and never reaches
/// the branch `atan2` cuts the ring angle on. A file writes a bend where it
/// sits, and so does this (§280.5).
///
/// **The half bend is there because three eighths of a turn cannot be written
/// exactly** (§280.1). It runs from `+x` round through `+y` to `−x`, with both
/// caps on the plane `y = 0` facing `−y` and the seam picked by `y ≥ 0`; turned
/// one quarter turn, its wall runs across `π` round the ring.
///
/// # Errors
///
/// Whatever [`SolidBuilder::build`] refuses the description with.
pub fn a_bend_turned_and_written_out(
    model: &mut Model,
    ring: f64,
    tube: f64,
    quarters: u32,
    turns: u32,
) -> Result<SolidKey> {
    let inner = ring - tube;
    let mut sits = match quarters {
        1 => WhereABendSits::quarter(inner),
        2 => WhereABendSits::half(inner),
        other => panic!("a bend of {other} quarter turns is not one this writes"),
    };
    for _ in 0..turns {
        sits = sits.turned_a_quarter_turn();
    }
    a_bend(model, ring, tube, &sits)
}

/// A bend `quarters` quarter turns long round the ring, written out where it
/// sits the way [`a_bend_turned_and_written_out`] is, but turned by `angle`
/// radians rather than by whole quarter turns.
///
/// Every number of the bend is put through the general rotation
/// `(x, y) → (x cos θ − y sin θ, x sin θ + y cos θ)`, not the exact quarter
/// turn `a_bend_turned_and_written_out` composes — so, unlike that function,
/// `angle` need not be a multiple of `π/2`. That is what puts a band of wall
/// at a ring angle that is not a multiple of `π/2` where the mesh actually
/// reads it: placing a bend by `Isometry::rotation` does not, since a placed
/// face is meshed in the frame it is stored in (row Q1950).
///
/// # Errors
///
/// Whatever [`SolidBuilder::build`] refuses the description with.
pub fn a_bend_written_out_at_an_angle(
    model: &mut Model,
    ring: f64,
    tube: f64,
    quarters: u32,
    angle: f64,
) -> Result<SolidKey> {
    let inner = ring - tube;
    let sits = match quarters {
        1 => WhereABendSits::quarter(inner),
        2 => WhereABendSits::half(inner),
        other => panic!("a bend of {other} quarter turns is not one this writes"),
    }
    .turned_by(angle);
    a_bend(model, ring, tube, &sits)
}

/// The top half of a whole ring about `+z`, `z ≥ 0`: a torus face right round
/// the ring and a flat ring of the plane `z = 0` closing it underneath.
///
/// Both faces are bounded by the outer and the inner equator, each one closed
/// edge whose corner on the `+x` axis says which of the two circles `z = 0`
/// cuts it is. With `a_seam` the torus face is one wire instead: round the
/// outer equator, over the top of the tube along the plane `y = 0`, back round
/// the inner equator and back over the tube — a seam across the tube, which is
/// how a file writes a face that goes right round the ring. Without it the
/// torus face has one wire per equator, and each of them climbs a whole turn
/// round the ring.
///
/// # Errors
///
/// Whatever [`SolidBuilder::build`] refuses the description with.
pub fn the_top_half_of_a_ring(
    model: &mut Model,
    ring: f64,
    tube: f64,
    a_seam: bool,
) -> Result<SolidKey> {
    let skin = Torus::new(Axis::z(), ring, tube).expect("a tube narrower than its ring");
    let mut builder = SolidBuilder::new();

    let outer_corner = builder.add_vertex(Point::new(ring + tube, 0.0, 0.0))?;
    let inner_corner = builder.add_vertex(Point::new(ring - tube, 0.0, 0.0))?;
    let outer_equator = builder.add_edge(
        outer_corner,
        outer_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let inner_equator = builder.add_edge(
        inner_corner,
        inner_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;

    let underneath = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let mut flat_wires = Vec::with_capacity(2);
    for (edge, turn) in [
        (outer_equator, Orientation::Forward),
        (inner_equator, Orientation::Reversed),
    ] {
        let coedge = builder.add_coedge(edge, turn)?;
        flat_wires.push(builder.add_wire(&[DraftStep::Along(coedge)])?);
    }
    let flat = builder.add_face(Surface::Plane(underneath), &flat_wires)?;

    let mut round_wires = Vec::with_capacity(2);
    if a_seam {
        let across_the_tube =
            Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 1.0, 0.0))
                .expect("the y axis is a unit normal");
        let seam = builder.add_edge(
            outer_corner,
            inner_corner,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(across_the_tube),
                picked_by: smallvec![
                    Picker::from(
                        HalfSpace::new(Vector::new(1.0, 0.0, 0.0), 0.0)
                            .expect("a normal with a length")
                    ),
                    Picker::from(
                        HalfSpace::new(Vector::new(0.0, 0.0, 1.0), 0.0)
                            .expect("a normal with a length")
                    ),
                ],
            },
        )?;
        let mut steps = Vec::with_capacity(4);
        for (edge, turn) in [
            (outer_equator, Orientation::Reversed),
            (seam, Orientation::Forward),
            (inner_equator, Orientation::Forward),
            (seam, Orientation::Reversed),
        ] {
            steps.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
        }
        round_wires.push(builder.add_wire(&steps)?);
    } else {
        for (edge, turn) in [
            (outer_equator, Orientation::Reversed),
            (inner_equator, Orientation::Forward),
        ] {
            let coedge = builder.add_coedge(edge, turn)?;
            round_wires.push(builder.add_wire(&[DraftStep::Along(coedge)])?);
        }
    }
    let round = builder.add_face(Surface::Torus(skin), &round_wires)?;

    builder.add_shell(&[(round, Orientation::Forward), (flat, Orientation::Forward)])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}

/// The numbers that say where a bend about `+z` sits: its two corners on the
/// inner equator, which way each cap faces out of the material, and the
/// half-space that picks the seam's piece of the inner equator.
struct WhereABendSits {
    start_corner: [f64; 2],
    end_corner: [f64; 2],
    start_cap_facing: [f64; 2],
    end_cap_facing: [f64; 2],
    seam_side: [f64; 2],
}

impl WhereABendSits {
    fn quarter(inner: f64) -> Self {
        Self {
            start_corner: [inner, 0.0],
            end_corner: [0.0, inner],
            start_cap_facing: [0.0, -1.0],
            end_cap_facing: [-1.0, 0.0],
            seam_side: [1.0, 1.0],
        }
    }

    fn half(inner: f64) -> Self {
        Self {
            start_corner: [inner, 0.0],
            end_corner: [-inner, 0.0],
            start_cap_facing: [0.0, -1.0],
            end_cap_facing: [0.0, -1.0],
            seam_side: [0.0, 1.0],
        }
    }

    fn turned_a_quarter_turn(&self) -> Self {
        let turn = |[x, y]: [f64; 2]| [written_out(-y), written_out(x)];
        Self {
            start_corner: turn(self.start_corner),
            end_corner: turn(self.end_corner),
            start_cap_facing: turn(self.start_cap_facing),
            end_cap_facing: turn(self.end_cap_facing),
            seam_side: turn(self.seam_side),
        }
    }

    fn turned_by(&self, angle: f64) -> Self {
        let (sin, cos) = angle.sin_cos();
        let turn = |[x, y]: [f64; 2]| {
            [
                written_out(x * cos - y * sin),
                written_out(x * sin + y * cos),
            ]
        };
        Self {
            start_corner: turn(self.start_corner),
            end_corner: turn(self.end_corner),
            start_cap_facing: turn(self.start_cap_facing),
            end_cap_facing: turn(self.end_cap_facing),
            seam_side: turn(self.seam_side),
        }
    }
}

/// `−0.0` written as `+0.0`, which is how a person writing a zero out writes it.
fn written_out(number: f64) -> f64 {
    if number == 0.0 { 0.0 } else { number }
}

fn a_bend(model: &mut Model, ring: f64, tube: f64, sits: &WhereABendSits) -> Result<SolidKey> {
    let skin = Torus::new(Axis::z(), ring, tube).expect("a tube narrower than its ring");
    let surface = Surface::Torus(skin);
    let mut builder = SolidBuilder::new();

    let on_the_middle_plane = |[x, y]: [f64; 2]| Point::new(x, y, 0.0);
    let start_corner = builder.add_vertex(on_the_middle_plane(sits.start_corner))?;
    let end_corner = builder.add_vertex(on_the_middle_plane(sits.end_corner))?;

    // The seam runs along the inner equator from one cap to the other. Two
    // pickers: the first says which of the two circles the plane `z = 0` cuts,
    // and the second says which piece of that circle.
    let flat_through_the_middle =
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let [side_x, side_y] = sits.seam_side;
    let seam = builder.add_edge(
        start_corner,
        end_corner,
        EdgeRun::PartOfWhereItCrosses {
            and: Surface::Plane(flat_through_the_middle),
            picked_by: smallvec![
                Picker::InsideAPipe(Cylinder::new(Axis::z(), ring).expect("the ring's own radius")),
                Picker::from(
                    HalfSpace::new(Vector::new(side_x, side_y, 0.0), 0.0)
                        .expect("a normal with a length")
                ),
            ],
        },
    )?;

    let start_circle = builder.add_edge(
        start_corner,
        start_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;
    let end_circle = builder.add_edge(
        end_corner,
        end_corner,
        EdgeRun::RightRoundWhereItsFacesCross,
    )?;

    let cap = |builder: &mut SolidBuilder, [x, y]: [f64; 2], edge, turn| -> Result<_> {
        let plane = Plane::from_point_and_normal(Point::origin(), Vector::new(x, y, 0.0))
            .expect("a coordinate direction is a unit normal");
        let coedge = builder.add_coedge(edge, turn)?;
        let wire = builder.add_wire(&[DraftStep::Along(coedge)])?;
        builder.add_face(Surface::Plane(plane), &[wire])
    };
    let start_cap = cap(
        &mut builder,
        sits.start_cap_facing,
        start_circle,
        Orientation::Forward,
    )?;
    let end_cap = cap(
        &mut builder,
        sits.end_cap_facing,
        end_circle,
        Orientation::Reversed,
    )?;

    let mut round_the_wall = Vec::with_capacity(4);
    for (edge, turn) in [
        (start_circle, Orientation::Reversed),
        (seam, Orientation::Forward),
        (end_circle, Orientation::Forward),
        (seam, Orientation::Reversed),
    ] {
        round_the_wall.push(DraftStep::Along(builder.add_coedge(edge, turn)?));
    }
    let wire = builder.add_wire(&round_the_wall)?;
    let side = builder.add_face(surface, &[wire])?;

    builder.add_shell(&[
        (start_cap, Orientation::Forward),
        (end_cap, Orientation::Forward),
        (side, Orientation::Forward),
    ])?;
    builder.build(model, &Policy::certified(), &mut ProvenanceLog::new())
}
