//! A round bar cut into two bands by a closed `ELLIPSE` where a leaning plane
//! crosses it, each band on its own `CYLINDRICAL_SURFACE`, reads the same
//! whether the two surfaces are written bit for bit alike or one ulp apart.
//!
//! `docs/decisions.md` §222.2: the reader checked a closed `CIRCLE` between
//! curved faces against every face and named it by its own plane, but an
//! `ELLIPSE` had no check, so between one pipe written twice a hair apart it
//! stayed `EdgeRun::RightRoundWhereItsFacesCross` — "where these two nearly
//! identical pipes cross", which is nowhere near the ellipse. The ellipse's own
//! plane crosses either pipe in exactly the ellipse, and the check is now
//! written in closed form for a pipe.
//!
//! The bit-identical spelling is the oracle: it already carried the ellipse's
//! plane, so the hair-apart one must come out with the same run.

mod written_by_hand;

use rcad_geom::{Deflection, EdgeRun, Point, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use written_by_hand::{a_header, real};

const RADIUS: f64 = 2.0;
const HEIGHT: f64 = 5.0;
/// Where the ellipse meets the seam, at `(RADIUS, 0, CORNER)`. The plane
/// through `(0, 0, 3)` square to `(1, 0, 2)` crosses the seam there.
const CORNER: f64 = 2.0;

#[test]
fn a_pipe_written_twice_bit_for_bit_names_the_ellipse_by_its_plane() {
    the_ellipse_is_named_by_its_plane_and_meshed_round(RADIUS);
}

#[test]
fn a_pipe_written_twice_one_ulp_apart_names_the_ellipse_by_its_plane() {
    the_ellipse_is_named_by_its_plane_and_meshed_round(RADIUS.next_up());
}

fn the_ellipse_is_named_by_its_plane_and_meshed_round(upper_radius: f64) {
    let text = a_bar_cut_by_an_ellipse(upper_radius);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let import = Import::read(&file, &policy, &mut log).expect("a bar cut by an ellipse reads");
    let [(_, bar)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let model = import.model();
    let solid = model.solid(*bar).expect("this model made the key");
    let leaning = Vector::new(1.0, 0.0, 2.0);

    let mut middle = None;
    for face in solid.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if edge.start().point().coordinates()[2] != CORNER
                    || edge.start().key() != edge.end().key()
                {
                    continue;
                }
                middle = Some(edge.key());
                let EdgeRun::RightRoundWhereItCrosses { and } = edge.run() else {
                    panic!(
                        "the two bands are one pipe, so the ellipse has to carry the surface \
                         that names it; it says {:?}",
                        edge.run()
                    );
                };
                let Surface::Plane(flat) = and else {
                    panic!("an ellipse round a pipe is named by its own plane, not {and:?}");
                };
                assert!(
                    flat.normal().cross(leaning).length() < 1e-15,
                    "the plane is the ellipse's own, not one square to {:?}",
                    flat.normal()
                );
                assert!(
                    flat.signed_distance_to(Point::new(0.0, 0.0, 3.0)).abs() < 1e-15,
                    "the plane runs through the ellipse's centre"
                );
            }
        }
    }
    let middle = middle.expect("the bar has a closed ellipse across it");

    model
        .validate(&policy, &mut log)
        .expect("the model the reader built passes the full sweep");
    let mesh = solid
        .mesh(Deflection::of(0.01).expect("a length"), &policy, &mut log)
        .expect("the bar meshes");
    let allowed = mesh.deviation().magnitude();
    let along = mesh.points_along_edge(middle);
    assert!(
        along.len() > 8,
        "a whole ellipse round a pipe of radius two at a hundredth came back with {} points",
        along.len()
    );
    let across = leaning.normalized().expect("a direction");
    for &at in along {
        let [x, y, z] = mesh.points()[at as usize].coordinates();
        let off = ((x * x + y * y).sqrt() - RADIUS)
            .abs()
            .max((across.dot(Vector::new(x, y, z - 3.0))).abs());
        assert!(
            off <= allowed,
            "{:?} is {off} off the ellipse, past the published {allowed}",
            [x, y, z]
        );
    }
}

/// A bar of `RADIUS` on the `z` axis from `0` to `HEIGHT`, cut into two bands
/// by the ellipse where the plane through `(0, 0, 3)` square to `(1, 0, 2)`
/// crosses it. The lower band sits on a pipe of `RADIUS`, the upper one on a
/// pipe of `upper_radius`, each its own entity.
///
/// The ellipse's reference direction `(2, 0, −1)` puts its first point on the
/// seam: its long semi-axis is `RADIUS·√5 / 2`, which is `√5`, and `√5` times
/// that direction over its own length is `(2, 0, −1)` exactly. Its second
/// direction is then `+y`, so it turns the way the two circles do.
fn a_bar_cut_by_an_ellipse(upper_radius: f64) -> String {
    let mut text = a_header(".MILLI.");
    text += "#20=DIRECTION('',(0.,0.,1.));\n";
    text += "#21=DIRECTION('',(1.,0.,0.));\n";
    text += "#22=DIRECTION('',(0.,0.,-1.));\n";
    text += &format!("#23=VECTOR('',#20,{});\n", real(1.0));
    text += "#24=DIRECTION('',(1.,0.,2.));\n";
    text += "#25=DIRECTION('',(2.,0.,-1.));\n";
    for (which, level) in [0.0, CORNER, HEIGHT].iter().enumerate() {
        text += &format!(
            "#{}=CARTESIAN_POINT('',({},0.,{}));\n",
            100 + which,
            real(RADIUS),
            real(*level)
        );
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 110 + which, 100 + which);
    }
    for (which, level) in [0.0, 3.0, HEIGHT].iter().enumerate() {
        text += &format!(
            "#{}=CARTESIAN_POINT('',(0.,0.,{}));\n",
            120 + which,
            real(*level)
        );
    }
    text += "#130=AXIS2_PLACEMENT_3D('',#120,#20,#21);\n";
    text += &format!("#140=CIRCLE('',#130,{});\n", real(RADIUS));
    text += "#131=AXIS2_PLACEMENT_3D('',#121,#24,#25);\n";
    text += &format!(
        "#141=ELLIPSE('',#131,{},{});\n",
        real(5.0_f64.sqrt()),
        real(RADIUS)
    );
    text += "#132=AXIS2_PLACEMENT_3D('',#122,#20,#21);\n";
    text += &format!("#142=CIRCLE('',#132,{});\n", real(RADIUS));
    for which in 0..3 {
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

    text += "#200=CARTESIAN_POINT('',(0.,0.,0.));\n";
    text += "#201=AXIS2_PLACEMENT_3D('',#200,#20,#21);\n";
    for (band, radius) in [RADIUS, upper_radius].iter().enumerate() {
        text += &format!(
            "#{}=CYLINDRICAL_SURFACE('',#201,{});\n",
            210 + band,
            real(*radius)
        );
        let (below, above, seam) = (150 + band, 151 + band, 170 + band);
        let first = 220 + band * 4;
        text += &format!("#{first}=ORIENTED_EDGE('',*,*,#{below},.T.);\n");
        text += &format!("#{}=ORIENTED_EDGE('',*,*,#{seam},.T.);\n", first + 1);
        text += &format!("#{}=ORIENTED_EDGE('',*,*,#{above},.F.);\n", first + 2);
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
