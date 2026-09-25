//! A cone whose tip is a ball resting in it, touching it along a circle.
//!
//! `docs/decisions.md` §222.2: a closed circle where a ball meets another
//! curved face is described as two halves, each named by the circle's own
//! plane, once the file's circle has been checked to lie on every face along
//! it (§189.3). The check had no bound for a cone, so a ball resting in one was
//! left to the two faces, which only touch round the circle and cross nowhere.
//! The check now has a bound for a cone.
//!
//! Every number here is exact in binary: a cone with its apex at the origin
//! whose radius grows three for every four up, and a ball of radius `3.75`
//! centred `6.25` up its axis, which touches it round the circle of radius `3`
//! at height `4`.

mod written_by_hand;

use rcad_geom::{Deflection, EdgeRun, Point, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use written_by_hand::{a_header, real};

/// The circle the ball touches the cone round, as `(height, radius)`.
const TOUCHING: (f64, f64) = (4.0, 3.0);
/// The flat top of the cone, as `(height, radius)`.
const TOP: (f64, f64) = (8.0, 6.0);
const BALL_CENTRE: f64 = 6.25;
const BALL_RADIUS: f64 = 3.75;

#[test]
fn the_circle_a_ball_touches_a_cone_round_is_two_halves_named_by_its_plane() {
    let text = a_cone_with_a_rounded_tip();
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let import = Import::read(&file, &policy, &mut log).expect("a cone with a rounded tip reads");
    let [(_, tip)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let model = import.model();
    let solid = model.solid(*tip).expect("this model made the key");
    let (level, radius) = TOUCHING;

    let mut halves = Vec::new();
    for face in solid.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                let ends = [edge.start().point(), edge.end().point()];
                if ends.iter().any(|end| end.coordinates()[2] != level) {
                    continue;
                }
                assert_ne!(
                    edge.start().key(),
                    edge.end().key(),
                    "the circle round a ball is cut in two, not left with one corner"
                );
                let EdgeRun::PartOfWhereItCrosses { and, .. } = edge.run() else {
                    panic!(
                        "half of the circle the ball touches the cone round has to carry the \
                         surface that names it; it says {:?}",
                        edge.run()
                    );
                };
                let Surface::Plane(flat) = and else {
                    panic!("the circle is named by its own plane, not {and:?}");
                };
                assert_eq!(
                    flat.normal().cross(Vector::new(0.0, 0.0, 1.0)).length(),
                    0.0,
                    "the plane is square to the cone's axis, not to {:?}",
                    flat.normal()
                );
                assert_eq!(
                    flat.signed_distance_to(Point::new(0.0, 0.0, level)),
                    0.0,
                    "the plane is the circle's own"
                );
                if !halves.contains(&edge.key()) {
                    halves.push(edge.key());
                }
            }
        }
    }
    assert_eq!(halves.len(), 2, "the circle is two halves");

    model
        .validate(&policy, &mut log)
        .expect("the model the reader built passes the full sweep");
    let mesh = solid
        .mesh(Deflection::of(0.01).expect("a length"), &policy, &mut log)
        .expect("the rounded cone meshes");
    let allowed = mesh.deviation().magnitude();
    for half in halves {
        let along = mesh.points_along_edge(half);
        assert!(
            along.len() > 4,
            "half a circle of radius three at a hundredth came back with {} points",
            along.len()
        );
        for &at in along {
            let [x, y, z] = mesh.points()[at as usize].coordinates();
            let off = ((x * x + y * y).sqrt() - radius)
                .abs()
                .max((z - level).abs());
            assert!(
                off <= allowed,
                "{:?} is {off} off the circle, past the published {allowed}",
                [x, y, z]
            );
        }
    }
}

/// The part of the cone between `TOUCHING` and `TOP`, closed by a flat disc on
/// top and by the part of the ball below `TOUCHING` underneath.
///
/// The cone band's loop runs as a pipe's does: round the circle the ball
/// touches, up the seam, back round the top circle and down the seam again.
/// The ball's cap walks the touching circle the other way, then down a seam
/// to the ball's south pole and back: a cap round a pole with no seam is
/// refused by `ACurvedFaceGoesRightRoundWithNoSeam`, because nothing then cuts
/// it open into a patch. The top disc walks the top circle the way the band
/// does not.
fn a_cone_with_a_rounded_tip() -> String {
    let mut text = a_header(".MILLI.");
    text += "#20=DIRECTION('',(0.,0.,1.));\n";
    text += "#21=DIRECTION('',(1.,0.,0.));\n";
    text += "#24=DIRECTION('',(3.,0.,4.));\n";
    text += &format!("#23=VECTOR('',#24,{});\n", real(1.0));
    for (which, (level, radius)) in [TOUCHING, TOP].iter().enumerate() {
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
        text += &format!(
            "#{}=CIRCLE('',#{},{});\n",
            140 + which,
            130 + which,
            real(*radius)
        );
        text += &format!(
            "#{}=EDGE_CURVE('',#{},#{},#{},.T.);\n",
            150 + which,
            110 + which,
            110 + which,
            140 + which
        );
    }
    text += "#160=LINE('',#100,#23);\n";
    text += "#170=EDGE_CURVE('',#110,#111,#160,.T.);\n";

    text += &format!(
        "#210=CONICAL_SURFACE('',#130,{},{});\n",
        real(TOUCHING.1),
        real(0.75_f64.atan())
    );
    text += "#220=ORIENTED_EDGE('',*,*,#150,.T.);\n";
    text += "#221=ORIENTED_EDGE('',*,*,#170,.T.);\n";
    text += "#222=ORIENTED_EDGE('',*,*,#151,.F.);\n";
    text += "#223=ORIENTED_EDGE('',*,*,#170,.F.);\n";
    text += "#230=EDGE_LOOP('',(#220,#221,#222,#223));\n";
    text += "#235=FACE_OUTER_BOUND('',#230,.T.);\n";
    text += "#240=ADVANCED_FACE('',(#235),#210,.T.);\n";

    text += &format!("#200=CARTESIAN_POINT('',(0.,0.,{}));\n", real(BALL_CENTRE));
    text += "#201=AXIS2_PLACEMENT_3D('',#200,#20,#21);\n";
    text += &format!("#211=SPHERICAL_SURFACE('',#201,{});\n", real(BALL_RADIUS));
    text += &format!(
        "#102=CARTESIAN_POINT('',(0.,0.,{}));\n",
        real(BALL_CENTRE - BALL_RADIUS)
    );
    text += "#112=VERTEX_POINT('',#102);\n";
    text += "#25=DIRECTION('',(0.,1.,0.));\n";
    text += "#134=AXIS2_PLACEMENT_3D('',#200,#25,#21);\n";
    text += &format!("#144=CIRCLE('',#134,{});\n", real(BALL_RADIUS));
    text += "#171=EDGE_CURVE('',#110,#112,#144,.T.);\n";
    text += "#250=ORIENTED_EDGE('',*,*,#150,.F.);\n";
    text += "#254=ORIENTED_EDGE('',*,*,#171,.T.);\n";
    text += "#255=ORIENTED_EDGE('',*,*,#171,.F.);\n";
    text += "#251=EDGE_LOOP('',(#250,#254,#255));\n";
    text += "#252=FACE_OUTER_BOUND('',#251,.T.);\n";
    text += "#253=ADVANCED_FACE('',(#252),#211,.T.);\n";

    text += "#261=PLANE('',#131);\n";
    text += "#262=ORIENTED_EDGE('',*,*,#151,.T.);\n";
    text += "#263=EDGE_LOOP('',(#262));\n";
    text += "#264=FACE_OUTER_BOUND('',#263,.T.);\n";
    text += "#265=ADVANCED_FACE('',(#264),#261,.T.);\n";

    text += "#500=CLOSED_SHELL('',(#240,#253,#265));\n";
    text += "#501=MANIFOLD_SOLID_BREP('',#500);\n";
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}
