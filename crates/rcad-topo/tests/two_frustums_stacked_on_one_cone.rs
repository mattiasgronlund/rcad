//! Two frustums stacked on one cone, the upper one naming the cone with its
//! axis turned round, united.
//!
//! A cone here is **both nappes** (`Cone`'s doc, "Two nappes and not one"), and
//! its equation is even in the axis direction, so the cone about `+z` and the
//! cone about `−z` with the same apex and the same slope are one point set.
//! `intersect` used to call two cones the same surface only when they held the
//! same numbers, so the pair was refused
//! `ACrossingBetweenAConeAndAnotherCurvedSurface` at the first union, in both
//! orders (Q44, §231.5). §357 decides it exactly.
//!
//! **What the union is now:** one solid of four faces — the two caps and the
//! two walls, one per spelling — which validates. The joiner still leaves the
//! two spellings of the wall apart; that is Q354's for pipes, and this file's
//! face count is the one to move to three when a joiner reaches cones.
//!
//! **Its volume is not asserted, and that is a finding and not a choice.** A
//! frustum whose wall names the cone about the axis pointing *away* from it
//! measures short of its own volume by more than the bound `measure`
//! publishes, before any fuse: §357.3 has the numbers.

use rcad_geom::{Axis, Cone, EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, Rule, SolidBuilder, SolidKey};

const SLOPE: f64 = 0.5;

#[test]
fn two_frustums_stacked_on_one_cone_turned_round_unite_either_way_round() {
    for lower_first in [true, false] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let lower = a_frustum(&mut model, 2.0, 4.0, Axis::z());
        let upper = a_frustum(&mut model, 4.0, 6.0, Axis::z().reversed());
        let [first, second] = if lower_first {
            [lower, upper]
        } else {
            [upper, lower]
        };
        let solids = model
            .fuse(
                &[first],
                &[second],
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("lower first {lower_first}: refused {why:?}"))
            .solids()
            .to_vec();
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("lower first {lower_first}: invalid {why:?}"));

        assert_eq!(solids.len(), 1, "lower first {lower_first}: one solid");
        let mut surfaces: Vec<Surface> = model
            .solid(solids[0])
            .expect("this model made the key")
            .faces()
            .map(|face| face.surface())
            .collect();
        surfaces.sort_by(Surface::how_it_sorts_against);
        let cap = |at: f64, outward: f64| {
            Surface::Plane(
                Plane::from_point_and_normal(
                    Point::new(0.0, 0.0, at),
                    Vector::new(0.0, 0.0, outward),
                )
                .expect("the z axis is a unit normal"),
            )
        };
        let wall = |axis: Axis| Surface::Cone(Cone::new(axis, SLOPE).expect("a positive slope"));
        let mut want = vec![
            cap(2.0, -1.0),
            cap(6.0, 1.0),
            wall(Axis::z()),
            wall(Axis::z().reversed()),
        ];
        want.sort_by(Surface::how_it_sorts_against);
        assert_eq!(
            surfaces, want,
            "lower first {lower_first}: the two caps, and the wall still in its two spellings"
        );
    }
}

/// A frustum of the cone with its apex at the origin and slope one half, caps at
/// `near` and `far` up `z`, its wall naming the cone about `axis`.
///
/// Turning the axis round turns round the circle each cap plane cuts the cone
/// in, so every coedge on a circle turns round with it; the seam runs up the
/// same ruling whichever way the axis points.
fn a_frustum(model: &mut Model, near: f64, far: f64, axis: Axis) -> SolidKey {
    let turned_round = axis.direction().components()[2] < 0.0;
    let round = |turn: Orientation| if turned_round { turn.reversed() } else { turn };
    let skin = Cone::new(axis, SLOPE).expect("a positive slope");
    let mut builder = SolidBuilder::new();

    let at = |height: f64| Point::new(SLOPE * height, 0.0, height);
    let bottom_corner = builder.add_vertex(at(near)).expect("a finite point");
    let top_corner = builder.add_vertex(at(far)).expect("a finite point");
    let seam = builder
        .add_edge(bottom_corner, top_corner, EdgeRun::Straight)
        .expect("a ruling of a cone is straight");
    let bottom_circle = builder
        .add_edge(
            bottom_corner,
            bottom_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("the cap plane cuts the cone in one closed curve");
    let top_circle = builder
        .add_edge(
            top_corner,
            top_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("the cap plane cuts the cone in one closed curve");

    let cap = |builder: &mut SolidBuilder, at: f64, outward: f64, edge, turn| {
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, at), Vector::new(0.0, 0.0, outward))
                .expect("the z axis is a unit normal");
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
    let bottom = cap(
        &mut builder,
        near,
        -1.0,
        bottom_circle,
        round(Orientation::Reversed),
    );
    let top = cap(
        &mut builder,
        far,
        1.0,
        top_circle,
        round(Orientation::Forward),
    );

    let round_the_wall = [
        (bottom_circle, round(Orientation::Forward)),
        (seam, Orientation::Forward),
        (top_circle, round(Orientation::Reversed)),
        (seam, Orientation::Reversed),
    ]
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("an edge of this build"),
        )
    });
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("round the bottom, up the seam, back round the top, down the seam");
    let side = builder
        .add_face(Surface::Cone(skin), &[wire])
        .expect("a face on the cone");

    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("three faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a frustum described this way is a solid")
}
