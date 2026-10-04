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
//! two walls, one per spelling — which validates and measures the frustum from
//! `z = 2` to `z = 6` inside the bound `measure` publishes. The joiner still
//! leaves the two spellings of the wall apart; that is Q354's for pipes, and
//! this file's face count is the one to move to three when a joiner reaches
//! cones.
//!
//! Each frustum alone measures inside its bound in both spellings too. The one
//! whose wall names the cone about the axis pointing *away* from it used to
//! mesh folded and measure well short, before any fuse: `Cone::point_at` put a
//! point with a negative `along` half a turn round from where `coordinates_of`
//! had read it (§357.3).

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, Rule, SolidBuilder, SolidKey};

const SLOPE: f64 = 0.5;

/// The frustum from `z = 2` to `z = 4` and the one from `z = 4` to `z = 6`,
/// each in both spellings of its wall, measured alone.
#[test]
fn a_frustum_measures_inside_its_bound_whichever_way_round_its_wall_names_the_cone() {
    for (near, far) in [(2.0, 4.0), (4.0, 6.0)] {
        for (axis, about) in [(Axis::z(), "+z"), (Axis::z().reversed(), "-z")] {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
            let frustum = a_frustum(&mut model, near, far, axis);
            assert_it_measures_inside_its_bound(
                &model,
                frustum,
                the_volume_of_the_frustum(near, far),
                &format!("z in [{near}, {far}] with its wall about {about}"),
            );
        }
    }
}

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
        assert_it_measures_inside_its_bound(
            &model,
            solids[0],
            the_volume_of_the_frustum(2.0, 6.0),
            &format!("the union, lower first {lower_first}"),
        );
    }
}

/// `|volume − want|` no more than the error `measure` publishes with it, at
/// the deflection §357.3's table was measured at.
fn assert_it_measures_inside_its_bound(model: &Model, solid: SolidKey, want: f64, what: &str) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: does not measure: {why:?}"));
    assert!(
        (measured.volume() - want).abs() <= measured.volume_error(),
        "{what} is {} against {want}, and the mesh may miss by {}",
        measured.volume(),
        measured.volume_error()
    );
}

/// `π·h·(r₁² + r₁r₂ + r₂²)/3`, with each cap's radius half its height.
fn the_volume_of_the_frustum(near: f64, far: f64) -> f64 {
    let (low, high) = (SLOPE * near, SLOPE * far);
    core::f64::consts::PI * (far - near) * (low * low + low * high + high * high) / 3.0
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

/// The same for a torus: two quarter bends on one ring, `+x` round to `+y`
/// about `+z` and `+y` round to `−x` about `−z`, united.
///
/// Refused `ACrossingBetweenATorusAndAnotherCurvedSurface` in both orders
/// before §357, and **refused further on now**: the fuse gets past the
/// question of whether the two walls cross and stops where the two spellings
/// and a cap meet at a corner, because everything past `intersect` still keys
/// a surface by the numbers it holds (§357.4). The pair written in one
/// spelling answers the half bend, `4π²`
/// (`two_quarter_bends_meeting_end_to_end_unite_into_a_half_bend`, §378.6),
/// and that is the answer this pair should be compared against once it gets
/// past this refusal.
#[test]
fn two_quarter_bends_on_one_ring_turned_round_get_past_the_crossing_either_way_round() {
    for first_quarter_first in [true, false] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let one = a_quarter_bend(&mut model, &FROM_X_ROUND_TO_Y, Axis::z());
        let other = a_quarter_bend(&mut model, &FROM_Y_ROUND_TO_MINUS_X, Axis::z().reversed());
        let [first, second] = if first_quarter_first {
            [one, other]
        } else {
            [other, one]
        };
        let said = model
            .fuse(
                &[first],
                &[second],
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .map(|answer| answer.solids().len());
        assert!(
            matches!(
                said,
                Err(Error::Geometry(
                    rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat
                ))
            ),
            "first quarter first {first_quarter_first}: {said:?}"
        );
    }
}

/// Where a quarter bend of ring four and tube one sits: its two corners on the
/// inner equator, which way each cap faces out of it, and the half-space that
/// picks the seam's quarter of the inner equator — as `rings/mod.rs` writes it.
struct WhereABendSits {
    start_corner: [f64; 2],
    end_corner: [f64; 2],
    start_cap_facing: [f64; 2],
    end_cap_facing: [f64; 2],
    seam_side: [f64; 2],
}

const RING: f64 = 4.0;
const TUBE: f64 = 1.0;

const FROM_X_ROUND_TO_Y: WhereABendSits = WhereABendSits {
    start_corner: [RING - TUBE, 0.0],
    end_corner: [0.0, RING - TUBE],
    start_cap_facing: [0.0, -1.0],
    end_cap_facing: [-1.0, 0.0],
    seam_side: [1.0, 1.0],
};

const FROM_Y_ROUND_TO_MINUS_X: WhereABendSits = WhereABendSits {
    start_corner: [0.0, RING - TUBE],
    end_corner: [-(RING - TUBE), 0.0],
    start_cap_facing: [1.0, 0.0],
    end_cap_facing: [0.0, -1.0],
    seam_side: [-1.0, 1.0],
};

/// A quarter bend sitting where `sits` says, its wall naming the ring about
/// `axis`.
///
/// Turning the axis round turns round each circle a cap plane cuts the ring
/// in, as it does a cone's; the wall still faces the shell the same way. The
/// wall is used forwards, as a torus faces out of its tube, and each circle is
/// walked right-handed about the outward normal of the face it bounds
/// (`docs/decisions.md` §361).
fn a_quarter_bend(model: &mut Model, sits: &WhereABendSits, axis: Axis) -> SolidKey {
    use rcad_geom::{Cylinder, HalfSpace, Picker, Torus};
    let turned_round = axis.direction().components()[2] < 0.0;
    let round = |turn: Orientation| if turned_round { turn.reversed() } else { turn };
    let skin = Surface::Torus(Torus::new(axis, RING, TUBE).expect("a tube narrower than its ring"));
    let mut builder = SolidBuilder::new();
    let on_the_middle_plane = |[x, y]: [f64; 2]| Point::new(x, y, 0.0);
    let start_corner = builder
        .add_vertex(on_the_middle_plane(sits.start_corner))
        .expect("a finite point");
    let end_corner = builder
        .add_vertex(on_the_middle_plane(sits.end_corner))
        .expect("a finite point");
    let middle = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, 1.0))
        .expect("the z axis is a unit normal");
    let [side_x, side_y] = sits.seam_side;
    let seam = builder
        .add_edge(
            start_corner,
            end_corner,
            EdgeRun::PartOfWhereItCrosses {
                and: Surface::Plane(middle),
                picked_by: smallvec::smallvec![
                    Picker::InsideAPipe(Cylinder::new(Axis::z(), RING).expect("a radius")),
                    Picker::from(
                        HalfSpace::new(Vector::new(side_x, side_y, 0.0), 0.0)
                            .expect("a normal with a length")
                    ),
                ],
            },
        )
        .expect("the seam runs along the inner equator");
    let start_circle = builder
        .add_edge(
            start_corner,
            start_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a cap plane cuts the ring in a circle through its corner");
    let end_circle = builder
        .add_edge(
            end_corner,
            end_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("a cap plane cuts the ring in a circle through its corner");
    let cap = |builder: &mut SolidBuilder, [x, y]: [f64; 2], edge, turn| {
        let plane = Plane::from_point_and_normal(Point::origin(), Vector::new(x, y, 0.0))
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
        sits.start_cap_facing,
        start_circle,
        round(Orientation::Forward),
    );
    let end_cap = cap(
        &mut builder,
        sits.end_cap_facing,
        end_circle,
        round(Orientation::Reversed),
    );
    let round_the_wall = [
        (start_circle, round(Orientation::Reversed)),
        (seam, Orientation::Forward),
        (end_circle, round(Orientation::Forward)),
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
        .expect("round one cap, along the seam, back round the other");
    let side = builder.add_face(skin, &[wire]).expect("a face on the ring");
    builder
        .add_shell(&[
            (start_cap, Orientation::Forward),
            (end_cap, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("three faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bend described this way is a solid")
}
