//! Two quarter bends on one ring, end to end, both about `+z`: one from `+x`
//! round to `+y`, the next from `+y` round to `−x`. Fused under every rule,
//! both ways round. `docs/decisions.md` §378.
//!
//! The two meet only at one cap, the disk on `x = 0` round `(0, 4, 0)`, where
//! the first faces `−x` and the second `+x`. So their insides are apart, and
//! what each rule should give is known without measuring anything:
//!
//! | rule | should give | gives |
//! |---|---|---|
//! | union | one half bend from `+x` round to `−x`, `4π²` by Pappus | refused `ResultIsNotManifold { count: 4 }`, both ways round |
//! | intersection | no solid at all (§191) | no solid, both ways round |
//! | difference | the first argument whole, `2π²` | the first argument whole, both ways round |
//!
//! **The union's refusal is a gap.** Selection keeps the right four pieces:
//! the two caps on `y = 0` and the two walls. Assembly then joins the two caps'
//! circles into one edge. The plane `y = 0` cuts the ring in two circles, round
//! `(4, 0, 0)` and round `(−4, 0, 0)`, and `Corners::with_each_closed_curve_started_once`
//! names a closed curve by its two surfaces alone. So both circles get one
//! name, the corner at `(3, 0, 0)` is handed over to the one at `(−3, 0, 0)`,
//! and the one edge left has both caps and both walls along it. Keeping the
//! two corners apart, in a scratch copy, made both orders answer one solid of
//! four faces that validates and measures `39.4123 ± 0.7843`. §378 has the
//! trace and the fixes considered.
//!
//! `a_quarter_bend`, `WhereABendSits` and its two consts are copied from
//! `two_frustums_stacked_on_one_cone.rs`, where the same two bends are fused
//! with the second one's axis turned round.

use rcad_geom::{Axis, Deflection, EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, Rule, SolidBuilder, SolidKey};

#[test]
fn two_quarter_bends_meeting_end_to_end_are_refused_as_not_a_manifold_when_united() {
    for first_quarter_first in [true, false] {
        let (mut model, [first, second]) = the_two_bends(first_quarter_first);
        let said = model
            .fuse(
                &[first],
                &[second],
                Rule::Union,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .map(|fused| fused.solids().len());
        assert!(
            matches!(said, Err(Error::ResultIsNotManifold { count: 4 })),
            "first quarter first {first_quarter_first}: {said:?}"
        );
    }
}

#[test]
fn two_quarter_bends_meeting_end_to_end_have_no_solid_in_common() {
    for first_quarter_first in [true, false] {
        let (mut model, [first, second]) = the_two_bends(first_quarter_first);
        let mut log = ProvenanceLog::new();
        let solids = model
            .fuse(
                &[first],
                &[second],
                Rule::Intersection,
                &Policy::certified(),
                &mut log,
            )
            .unwrap_or_else(|why| panic!("first quarter first {first_quarter_first}: {why:?}"))
            .solids()
            .len();
        assert_eq!(solids, 0, "first quarter first {first_quarter_first}");
        assert!(log.is_empty(), "{log:?}");
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .expect("a model the fuse added nothing to");
    }
}

#[test]
fn a_quarter_bend_less_the_next_quarter_round_is_the_first_bend_whole() {
    for first_quarter_first in [true, false] {
        let (mut model, [first, second]) = the_two_bends(first_quarter_first);
        let mut log = ProvenanceLog::new();
        let solids = model
            .fuse(
                &[first],
                &[second],
                Rule::Difference,
                &Policy::certified(),
                &mut log,
            )
            .unwrap_or_else(|why| panic!("first quarter first {first_quarter_first}: {why:?}"))
            .solids()
            .to_vec();
        assert!(log.is_empty(), "{log:?}");
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("first quarter first {first_quarter_first}: {why:?}"));
        let [answer] = solids[..] else {
            panic!("first quarter first {first_quarter_first}: {solids:?}");
        };
        assert_eq!(
            surfaces_of(&model, answer),
            surfaces_of(&model, first),
            "first quarter first {first_quarter_first}: the first bend's two caps and its wall"
        );
        let measured = model
            .solid(answer)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("first quarter first {first_quarter_first}: {why:?}"));
        let quarter = 2.0 * core::f64::consts::PI * core::f64::consts::PI;
        assert!(
            (measured.volume() - quarter).abs() <= measured.volume_error(),
            "first quarter first {first_quarter_first}: {} against {quarter}, and the mesh may \
             miss by {}",
            measured.volume(),
            measured.volume_error()
        );
    }
}

/// The bend from `+x` round to `+y` and the one from `+y` round to `−x`, in one
/// model, handed back in the order the fuse takes them.
fn the_two_bends(first_quarter_first: bool) -> (Model, [SolidKey; 2]) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let one = a_quarter_bend(&mut model, &FROM_X_ROUND_TO_Y, Axis::z());
    let other = a_quarter_bend(&mut model, &FROM_Y_ROUND_TO_MINUS_X, Axis::z());
    let both = if first_quarter_first {
        [one, other]
    } else {
        [other, one]
    };
    (model, both)
}

fn surfaces_of(model: &Model, solid: SolidKey) -> Vec<Surface> {
    let mut surfaces: Vec<Surface> = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .collect();
    surfaces.sort_by(Surface::how_it_sorts_against);
    surfaces
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
