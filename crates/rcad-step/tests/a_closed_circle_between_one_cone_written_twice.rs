//! A cone frustum cut into two bands by a closed circle, each band on its own
//! `CONICAL_SURFACE`, reads the same whether the two surfaces are written bit
//! for bit alike or one ulp apart.
//!
//! `docs/decisions.md` §222.2: the reader checked a closed `CIRCLE` between
//! curved faces against every face and named it by its own plane, but it had
//! no bound for a cone, so between one cone written twice a hair apart the
//! circle stayed `EdgeRun::RightRoundWhereItsFacesCross` — "where these two
//! nearly identical cones cross", which is nowhere near the circle. The
//! circle's own plane crosses either cone in exactly the circle, and the check
//! now has a bound for a cone.
//!
//! The bit-identical spelling is the oracle: it already carried the circle's
//! plane, so the hair-apart one must come out with the same run.
//!
//! **Then the model has to validate, and one ulp apart it did not.**
//! `Model::validate` read each band's use of the middle circle against the
//! other band's cone rather than against the plane the run names. The two
//! cones' gradients at the corner cross to `(0, 1.07e-14, 0)` and the lower
//! band's far corner is `-7.1e-15` inside the upper cone, both rounding, and
//! together they refused a sound frustum as
//! `AClosedEdgeIsWalkedAgainstItsSurface` (`docs/decisions.md` §229.4).
//!
//! The middle circle is written running both ways round, its `CIRCLE`'s axis up
//! the cone's or down it, which turns the plane that names it round as well.
//! Which way that plane faces may not change the answer.

mod written_by_hand;

use rcad_geom::{Deflection, EdgeRun, Point, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use written_by_hand::{a_header, real};

/// The frustum's three circles, as `(height, radius)`. They lie on the cone
/// with its apex at the origin whose radius grows three for every four up,
/// and the middle one is where the two bands meet.
const CIRCLES: [(f64, f64); 3] = [(4.0, 3.0), (8.0, 6.0), (12.0, 9.0)];
const MIDDLE: (f64, f64) = CIRCLES[1];

/// Which way the middle circle's `CIRCLE` turns about the cone's axis.
#[derive(Debug, Clone, Copy)]
enum MiddleCircleRuns {
    RoundTheAxis,
    BackRoundTheAxis,
}

#[test]
fn a_cone_written_twice_bit_for_bit_names_the_middle_circle_by_its_plane() {
    for runs in [
        MiddleCircleRuns::RoundTheAxis,
        MiddleCircleRuns::BackRoundTheAxis,
    ] {
        the_middle_circle_is_named_by_its_plane_and_meshed_round(MIDDLE.1, runs);
    }
}

/// Fails where `Model::validate` reads the middle circle against the other
/// band's cone: the model is refused `AClosedEdgeIsWalkedAgainstItsSurface`.
#[test]
fn a_cone_written_twice_one_ulp_apart_names_the_middle_circle_by_its_plane() {
    for runs in [
        MiddleCircleRuns::RoundTheAxis,
        MiddleCircleRuns::BackRoundTheAxis,
    ] {
        the_middle_circle_is_named_by_its_plane_and_meshed_round(MIDDLE.1.next_up(), runs);
    }
}

fn the_middle_circle_is_named_by_its_plane_and_meshed_round(
    upper_radius: f64,
    runs: MiddleCircleRuns,
) {
    let text = a_frustum_in_two_bands(upper_radius, runs);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let import = Import::read(&file, &policy, &mut log).expect("a frustum in two bands reads");
    let [(_, frustum)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let model = import.model();
    let solid = model.solid(*frustum).expect("this model made the key");
    let (level, radius) = MIDDLE;

    let mut middle = None;
    for face in solid.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if edge.start().point().coordinates()[2] != level
                    || edge.start().key() != edge.end().key()
                {
                    continue;
                }
                middle = Some(edge.key());
                let EdgeRun::RightRoundWhereItCrosses { and } = edge.run() else {
                    panic!(
                        "the two bands are one cone, so the middle circle has to carry the \
                         surface that names it; it says {:?}",
                        edge.run()
                    );
                };
                let Surface::Plane(flat) = and else {
                    panic!("a circle round a cone is named by its own plane, not {and:?}");
                };
                assert_eq!(
                    flat.normal().cross(Vector::new(0.0, 0.0, 1.0)).length(),
                    0.0,
                    "the plane is square to the cone's axis, not to {:?}",
                    flat.normal()
                );
                let faces_up = flat.normal().dot(Vector::new(0.0, 0.0, 1.0)) > 0.0;
                assert_eq!(
                    faces_up,
                    matches!(runs, MiddleCircleRuns::RoundTheAxis),
                    "the plane faces the way the circle's axis points, and with the circle \
                     written {runs:?} it faces {:?}",
                    flat.normal()
                );
                assert_eq!(
                    flat.signed_distance_to(Point::new(0.0, 0.0, level)),
                    0.0,
                    "the plane is the circle's own"
                );
            }
        }
    }
    let middle = middle.expect("the frustum has a closed circle at the middle");

    model
        .validate(&policy, &mut log)
        .expect("the model the reader built passes the full sweep");
    let mesh = solid
        .mesh(Deflection::of(0.01).expect("a length"), &policy, &mut log)
        .expect("the frustum meshes");
    let allowed = mesh.deviation().magnitude();
    let along = mesh.points_along_edge(middle);
    assert!(
        along.len() > 8,
        "a whole circle of radius six at a hundredth came back with {} points",
        along.len()
    );
    for &at in along {
        let [x, y, z] = mesh.points()[at as usize].coordinates();
        let off = ((x * x + y * y).sqrt() - radius)
            .abs()
            .max((z - level).abs());
        assert!(
            off <= allowed,
            "{:?} is {off} off the middle circle, past the published {allowed}",
            [x, y, z]
        );
    }

    let (bottom, top) = (CIRCLES[0].1, CIRCLES[2].1);
    let tall = CIRCLES[2].0 - CIRCLES[0].0;
    let closed_form =
        core::f64::consts::PI * tall / 3.0 * (bottom * bottom + bottom * top + top * top);
    let measured = solid
        .measure(Deflection::of(0.01).expect("a length"), &policy, &mut log)
        .expect("the frustum measures");
    assert!(
        (measured.volume() - closed_form).abs() <= measured.volume_error(),
        "measured {} ± {}, and a frustum from radius three to nine eight tall is 312π, \
         {closed_form}",
        measured.volume(),
        measured.volume_error()
    );
}

/// A frustum of the cone with its apex at the origin, opening up the `z` axis
/// with a half-angle whose tangent is `3/4`, between the bottom and top of
/// `CIRCLES` and cut at the middle one into two bands. Both bands' cones are
/// placed at the middle circle: the lower one with radius `MIDDLE.1`, the upper
/// one with `upper_radius`, each its own entity.
///
/// Each band's loop runs as a pipe's does: round the bottom circle, up the
/// seam, back round the top circle and down the seam again, which keeps the
/// side of the cone facing away from its axis on the left.
///
/// Written [`MiddleCircleRuns::BackRoundTheAxis`], the middle circle's axis
/// points down the cone's and both loops use it the other way, so each band
/// still goes round it the same way in space.
fn a_frustum_in_two_bands(upper_radius: f64, runs: MiddleCircleRuns) -> String {
    let mut text = a_header(".MILLI.");
    text += "#20=DIRECTION('',(0.,0.,1.));\n";
    text += "#21=DIRECTION('',(1.,0.,0.));\n";
    text += "#22=DIRECTION('',(0.,0.,-1.));\n";
    text += "#24=DIRECTION('',(3.,0.,4.));\n";
    text += &format!("#23=VECTOR('',#24,{});\n", real(1.0));
    let middle_turned = matches!(runs, MiddleCircleRuns::BackRoundTheAxis);
    text += "#136=AXIS2_PLACEMENT_3D('',#121,#22,#21);\n";
    for (which, (level, radius)) in CIRCLES.iter().enumerate() {
        text += &format!(
            "#{}=CARTESIAN_POINT('',({},0.,{}));\n",
            100 + which,
            real(*radius),
            real(*level)
        );
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 110 + which, 100 + which);
        text += &format!(
            "#{}=CARTESIAN_POINT('',(0.,0.,{}));\n",
            120 + which,
            real(*level)
        );
        text += &format!(
            "#{}=AXIS2_PLACEMENT_3D('',#{},#20,#21);\n",
            130 + which,
            120 + which
        );
        let placed = if which == 1 && middle_turned {
            136
        } else {
            130 + which
        };
        text += &format!("#{}=CIRCLE('',#{placed},{});\n", 140 + which, real(*radius));
        text += &format!(
            "#{}=EDGE_CURVE('',#{},#{},#{},.T.);\n",
            150 + which,
            110 + which,
            110 + which,
            140 + which
        );
    }
    for seam in 0..2 {
        text += &format!("#{}=LINE('',#{},#23);\n", 160 + seam, 100 + seam);
        text += &format!(
            "#{}=EDGE_CURVE('',#{},#{},#{},.T.);\n",
            170 + seam,
            110 + seam,
            111 + seam,
            160 + seam
        );
    }

    let half_angle = 0.75_f64.atan();
    for (band, radius) in [MIDDLE.1, upper_radius].iter().enumerate() {
        text += &format!(
            "#{}=CONICAL_SURFACE('',#131,{},{});\n",
            210 + band,
            real(*radius),
            real(half_angle)
        );
        let (below, above, seam) = (150 + band, 151 + band, 170 + band);
        let along_its_curve = |edge: usize, along: bool| {
            if along != (middle_turned && edge == 151) {
                ".T."
            } else {
                ".F."
            }
        };
        let first = 220 + band * 4;
        text += &format!(
            "#{first}=ORIENTED_EDGE('',*,*,#{below},{});\n",
            along_its_curve(below, true)
        );
        text += &format!("#{}=ORIENTED_EDGE('',*,*,#{seam},.T.);\n", first + 1);
        text += &format!(
            "#{}=ORIENTED_EDGE('',*,*,#{above},{});\n",
            first + 2,
            along_its_curve(above, false)
        );
        text += &format!("#{}=ORIENTED_EDGE('',*,*,#{seam},.F.);\n", first + 3);
        text += &format!(
            "#{}=EDGE_LOOP('',(#{first},#{},#{},#{}));\n",
            230 + band,
            first + 1,
            first + 2,
            first + 3
        );
        text += &format!(
            "#{}=FACE_OUTER_BOUND('',#{},.T.);\n",
            235 + band,
            230 + band
        );
        text += &format!(
            "#{}=ADVANCED_FACE('',(#{}),#{},.T.);\n",
            240 + band,
            235 + band,
            210 + band
        );
    }

    text += "#250=AXIS2_PLACEMENT_3D('',#120,#22,#21);\n";
    text += "#251=PLANE('',#250);\n";
    text += "#252=ORIENTED_EDGE('',*,*,#150,.F.);\n";
    text += "#253=EDGE_LOOP('',(#252));\n";
    text += "#254=FACE_OUTER_BOUND('',#253,.T.);\n";
    text += "#255=ADVANCED_FACE('',(#254),#251,.T.);\n";
    text += "#260=AXIS2_PLACEMENT_3D('',#122,#20,#21);\n";
    text += "#261=PLANE('',#260);\n";
    text += "#262=ORIENTED_EDGE('',*,*,#152,.T.);\n";
    text += "#263=EDGE_LOOP('',(#262));\n";
    text += "#264=FACE_OUTER_BOUND('',#263,.T.);\n";
    text += "#265=ADVANCED_FACE('',(#264),#261,.T.);\n";

    text += "#500=CLOSED_SHELL('',(#240,#241,#255,#265));\n";
    text += "#501=MANIFOLD_SOLID_BREP('',#500);\n";
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}
