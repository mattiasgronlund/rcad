//! A mitre `ELLIPSE` whose centre the file writes off one of its two pipes
//! is named by its pipes, and its sibling whose centre is on both keeps its
//! plane.
//!
//! `Fuseholder_Cylinder-5x20mm_Schurter_0031_8201_Horizontal_Open.step` bends
//! eight pipes of radius `0.4` through `45°`, each where an upright pipe meets
//! a leaning one in a `22.5°` mitre written `0.432956880117, 0.4`. Four of the
//! eight (`#338`, `#2313`, `#2331`, `#3043`) put the mitre's centre
//! `(±5e-13, 0, −5e-13)` off the leaning pipe's axis, because the file writes
//! the centre's two offsets from that axis's origin as `0.117157287526` and
//! `0.117157287525` (or `1.168556998188` and `1.168556998189`), which a
//! `45°` axis wants equal. That is `√2·5e-13` off, `193` or `284` roundings
//! of the largest number, where the reader allows `16`, so the reader turns
//! the ellipse away and names the run by its two pipes. The other four, whose
//! centre has a short decimal for its `x` (`22.1` or `0.4`), are on both
//! axes, and keep their ellipse.
//!
//! These are limits and not gaps (`docs/decisions.md` §16.10, §335.4). Two
//! pipes of one radius whose axes meet cross only in the two planes through
//! the meeting point that halve the angle between the axes. Taking the mitre's
//! plane through that point, and not through the file's centre, gives an
//! ellipse on both pipes, but it reaches neither corner the file ends each of
//! the four edges at. The one where both seams meet is `3.8e-13` off both
//! planes, `105` or `171` spacings. The one across the pipe the file puts
//! `1.68e-13` inside the leaning pipe, certified, so no curve on that pipe
//! runs through it.
//!
//! The two pins hold each side: widening the reader's allowance far enough to
//! keep `#338`'s ellipse fails the first, and narrowing it to nothing fails the
//! second.

mod written_by_hand;

use std::f64::consts::SQRT_2;

use rcad_geom::{Axis, Cylinder, Deflection, EdgeRun, Plane, Point, Surface, Vector};
use rcad_predicates::{Decision, Policy, ProvenanceLog, SideOfSurface};
use rcad_step::{Import, Refusal, StepFile};
use rcad_topo::{Error, Invalid};
use written_by_hand::a_header;

/// `#338`'s mitre, its two pipes and a disc closing each, in the file's own
/// decimals. Its centre is `√2·5e-13` off the leaning pipe's axis, so the
/// reader does not take the ellipse to be on that pipe and names the run by
/// the two pipes, and the elbow is then refused with its closed mitre walked
/// against its surface. Why a run named by these two pipes comes out that way
/// is not traced here. Widening the reader's allowance to `200` roundings
/// makes this elbow import, and fails this test.
#[test]
fn a_mitre_whose_centre_is_off_its_leaning_pipe_is_named_by_its_pipes() {
    let text = an_elbow(&MITRE_338);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let refused = match Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new()) {
        Ok(_) => panic!(
            "#338's mitre is centred 193 roundings off its leaning pipe's axis, and was read \
             as the curve its plane cuts"
        ),
        Err(refused) => refused,
    };
    assert!(
        matches!(
            &refused,
            Refusal::TheSolidTheFileDescribesIsNotOne { why, .. }
                if matches!(**why, Error::Invalid(Invalid::AClosedEdgeIsWalkedAgainstItsSurface))
        ),
        "an elbow whose mitre is named by its two pipes is refused with the mitre walked \
         against its surface, not {refused:?}"
    );
}

/// `#1769`'s mitre, the same bend turned the other way at `x = 22.1`. Its
/// centre is on both axes, so the ellipse is checked onto both pipes and the
/// run is named by its own plane.
#[test]
#[expect(
    clippy::approx_constant,
    reason = "0.707106781187 is the file's own twelve-digit decimal, not a spelling of 1/√2"
)]
fn a_mitre_whose_centre_is_on_both_pipes_is_named_by_its_plane() {
    let text = an_elbow(&MITRE_1769);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let import = match Import::read(&file, &policy, &mut log) {
        Ok(import) => import,
        Err(refused) => {
            panic!("#1769's elbow, its mitre centred on both pipes, is refused: {refused:?}")
        }
    };
    let [(_, elbow)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let model = import.model();
    let solid = model.solid(*elbow).expect("this model made the key");
    let corner = [22.1, 4.8, 10.048600289336];
    let normal = Vector::new(-0.382683432365, 0.0, 0.923879532511);
    let upright =
        Axis::as_written(Point::new(22.1, 4.4, 0.0), Vector::new(0.0, 0.0, 1.0)).expect("an axis");
    let leaning = Axis::as_written(
        Point::new(20.931443001811, 4.4, 11.217157287525),
        Vector::new(0.707106781187, 0.0, -0.707106781187),
    )
    .expect("an axis");

    let mut mitre = None;
    for face in solid.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if edge.start().key() != edge.end().key()
                    || edge.start().point().coordinates() != corner
                {
                    continue;
                }
                mitre = Some(edge.key());
                let EdgeRun::RightRoundWhereItCrosses { and } = edge.run() else {
                    panic!(
                        "the mitre has been checked to lie on both pipes, so it is named by \
                         the surface the file draws it on; it says {:?}",
                        edge.run()
                    );
                };
                let Surface::Plane(flat) = and else {
                    panic!("a mitre between two pipes is named by its own plane, not {and:?}");
                };
                assert!(
                    flat.normal().cross(normal).length() < 1e-14,
                    "the plane is the mitre's own, not one square to {:?}",
                    flat.normal()
                );
                assert!(
                    flat.signed_distance_to(Point::new(22.1, 4.4, 10.048600289336))
                        .abs()
                        < 1e-14,
                    "the plane runs through the mitre's centre"
                );
            }
        }
    }
    let mitre = mitre.expect("the elbow has a closed mitre between its two pipes");

    model
        .validate(&policy, &mut log)
        .expect("the model the reader built passes the full sweep");
    let mesh = solid
        .mesh(Deflection::of(0.001).expect("a length"), &policy, &mut log)
        .expect("the elbow meshes");
    let allowed = mesh.deviation().magnitude();
    let along = mesh.points_along_edge(mitre);
    assert!(
        along.len() > 8,
        "a whole mitre round a pipe of radius 0.4 at a thousandth came back with {} points",
        along.len()
    );
    for &at in along {
        let point = Point::from(mesh.points()[at as usize].coordinates());
        let off_the_upright = (upright.distance_to_point(point) - 0.4).abs();
        let off_the_leaning = (leaning.distance_to_point(point) - 0.4).abs();
        assert!(
            off_the_upright.max(off_the_leaning) <= allowed,
            "{point:?} is {off_the_upright} off the upright pipe and {off_the_leaning} off the \
             leaning one, past the published {allowed}"
        );
    }
}

/// The figures `docs/decisions.md` §335.4 publishes for the eight mitres,
/// worked out as the reader works them out: the leaning pipe built as
/// `geometry.rs` builds a `CYLINDRICAL_SURFACE`, the centre's distance from
/// its axis as `how_far_off_the_ellipse_can_be` takes it, in roundings of the
/// largest coordinate of the centre and the axis's origin, and the end
/// corner's side of the pipe and distance off it as
/// `a_corner_provably_off_a_curved_face` takes them.
///
/// Beside those, how far the seam corner is from the nearer of the two planes
/// through where the axes meet that halve the angle between them. A point the
/// same distance from both axes is as far along one as along the other, so
/// every point on both pipes is on one of those planes, and no curve on both
/// pipes comes nearer the corner than that.
///
/// The four the file writes off the axis are `√2·5e-13` off it worked out
/// exactly, and the kernel's `f64` lands within `0.4` roundings of that. The
/// four on both axes are `0` off. Each band is the measured figure to its
/// second decimal.
#[test]
fn how_far_the_file_writes_each_mitre_off_its_leaning_pipe() {
    for mitre in &THE_EIGHT_MITRES {
        let pipe = Axis::as_written(
            Point::from(mitre.leaning_origin),
            Vector::from(mitre.leaning_direction),
        )
        .and_then(|axis| Cylinder::new(axis, 0.4))
        .expect("the file's leaning pipe is a pipe");
        let centre = Point::from(mitre.centre);
        let q = pipe.axis().distance_to_point(centre);
        let rounding = f64::EPSILON
            * centre
                .largest_coordinate_magnitude()
                .max(pipe.axis().origin().largest_coordinate_magnitude());
        let roundings = q / rounding;
        let from_exact = (q - mitre.q_exactly) / rounding;

        let meet = Point::new(mitre.centre[0], mitre.centre[1], mitre.axes_meet_at_height);
        let meet_off_the_leaning = pipe.axis().distance_to_point(meet);
        let upright = Vector::new(0.0, 0.0, 1.0);
        let leaning = pipe.axis().direction();
        let seam = Point::from(mitre.seam_corner);
        let seam_off = [upright + leaning, upright - leaning]
            .into_iter()
            .map(|normal| {
                Plane::from_point_and_normal(meet, normal)
                    .expect("an upright and a 45° axis have two planes halving their angles")
                    .signed_distance_to(seam)
                    .abs()
            })
            .fold(f64::INFINITY, f64::min);
        let seam_spacings = seam_off / (seam.largest_coordinate_magnitude() * f64::EPSILON);

        let corner = Point::from(mitre.end_corner);
        let side = pipe.side_of(corner.into());
        let (round, along) = pipe.coordinates_of(corner);
        let off = pipe.point_at(round, along).distance_to(corner);
        let spacings = off / (corner.largest_coordinate_magnitude() * f64::EPSILON);
        println!(
            "{}: q {q:.4e}, {roundings:.2} roundings, {from_exact:+.2} from exact; axes meet \
             {meet_off_the_leaning:.1e} off the leaning one; seam corner {seam_off:.4e} off the \
             crossing, {seam_spacings:.2} spacings; end corner {side:?}, {off:.4e} off, \
             {spacings:.2} spacings",
            mitre.edge
        );

        let [low, high] = mitre.roundings_between;
        assert!(
            (low..=high).contains(&roundings),
            "{}'s centre is {roundings} roundings off its leaning pipe's axis, not {low} to \
             {high}",
            mitre.edge
        );
        assert!(
            from_exact.abs() <= 0.4,
            "{}'s centre is {q} off its leaning pipe's axis in f64, {from_exact} roundings from \
             the exact {}",
            mitre.edge,
            mitre.q_exactly
        );
        assert!(
            meet_off_the_leaning <= rounding,
            "{meet:?}, worked out as where {}'s two axes meet, is {meet_off_the_leaning} off the \
             leaning one",
            mitre.edge
        );
        let [low, high] = mitre.seam_corner_spacings_between;
        assert!(
            (low..=high).contains(&seam_spacings),
            "{}'s seam corner is {seam_spacings} spacings off both planes its two pipes cross \
             in, not {low} to {high}",
            mitre.edge
        );
        assert_eq!(
            side, mitre.end_corner_side,
            "{}'s end corner is not on the side of its leaning pipe it was measured on",
            mitre.edge
        );
        let [low, high] = mitre.end_corner_spacings_between;
        assert!(
            (low..=high).contains(&spacings),
            "{}'s end corner is {spacings} spacings off its leaning pipe, not {low} to {high}",
            mitre.edge
        );
    }
}

/// A mitre as the file writes it: the ellipse's centre, the leaning pipe of
/// radius `0.4` it is drawn on, the height at which that pipe's axis meets
/// the upright one through the centre, worked out in decimals, the corner at
/// the edge's one end where both seams meet, and the corner at its other end
/// across the pipe; then what was measured of it.
struct AMitre {
    edge: &'static str,
    centre: [f64; 3],
    leaning_origin: [f64; 3],
    leaning_direction: [f64; 3],
    axes_meet_at_height: f64,
    seam_corner: [f64; 3],
    end_corner: [f64; 3],
    q_exactly: f64,
    roundings_between: [f64; 2],
    seam_corner_spacings_between: [f64; 2],
    end_corner_side: Decision,
    end_corner_spacings_between: [f64; 2],
}

#[expect(
    clippy::approx_constant,
    reason = "0.707106781187 is the file's own twelve-digit decimal, not a spelling of 1/√2"
)]
const UP_AND_ON: [f64; 3] = [0.707106781187, 0.0, 0.707106781187];
#[expect(
    clippy::approx_constant,
    reason = "0.707106781187 is the file's own twelve-digit decimal, not a spelling of 1/√2"
)]
const DOWN_AND_ON: [f64; 3] = [0.707106781187, 0.0, -0.707106781187];
const OFF_BY_HALF_A_DIGIT_EACH_WAY: f64 = 5e-13 * SQRT_2;

const THE_EIGHT_MITRES: [AMitre; 8] = [
    AMitre {
        edge: "#338",
        centre: [16.471428571429, 4.4, 10.048600289336],
        leaning_origin: [16.354271283903, 4.4, 9.931443001811],
        leaning_direction: UP_AND_ON,
        axes_meet_at_height: 10.048600289337,
        seam_corner: [16.471428571429, 4.8, 10.048600289336],
        end_corner: [16.071428571429, 4.4, 10.214285714286],
        q_exactly: OFF_BY_HALF_A_DIGIT_EACH_WAY,
        roundings_between: [193.69, 193.71],
        seam_corner_spacings_between: [104.90, 104.92],
        end_corner_side: Decision::Neg,
        end_corner_spacings_between: [46.81, 46.82],
    },
    AMitre {
        edge: "#2313",
        centre: [16.471428571429, -4.4, 10.048600289336],
        leaning_origin: [16.354271283903, -4.4, 9.931443001811],
        leaning_direction: UP_AND_ON,
        axes_meet_at_height: 10.048600289337,
        seam_corner: [16.471428571429, -4.8, 10.048600289336],
        end_corner: [16.071428571429, -4.4, 10.214285714286],
        q_exactly: OFF_BY_HALF_A_DIGIT_EACH_WAY,
        roundings_between: [193.69, 193.71],
        seam_corner_spacings_between: [104.90, 104.92],
        end_corner_side: Decision::Neg,
        end_corner_spacings_between: [46.81, 46.82],
    },
    AMitre {
        edge: "#2331",
        centre: [6.028571428571, -4.4, 10.048600289336],
        leaning_origin: [4.860014430383, -4.4, 11.217157287525],
        leaning_direction: DOWN_AND_ON,
        axes_meet_at_height: 10.048600289337,
        seam_corner: [6.028571428571, -4.8, 10.048600289336],
        end_corner: [6.428571428571, -4.4, 10.214285714286],
        q_exactly: OFF_BY_HALF_A_DIGIT_EACH_WAY,
        roundings_between: [283.66, 283.68],
        seam_corner_spacings_between: [171.16, 171.18],
        end_corner_side: Decision::Neg,
        end_corner_spacings_between: [74.48, 74.49],
    },
    AMitre {
        edge: "#3043",
        centre: [6.028571428571, 4.4, 10.048600289336],
        leaning_origin: [4.860014430383, 4.4, 11.217157287525],
        leaning_direction: DOWN_AND_ON,
        axes_meet_at_height: 10.048600289337,
        seam_corner: [6.028571428571, 4.8, 10.048600289336],
        end_corner: [6.428571428571, 4.4, 10.214285714286],
        q_exactly: OFF_BY_HALF_A_DIGIT_EACH_WAY,
        roundings_between: [283.66, 283.68],
        seam_corner_spacings_between: [171.16, 171.18],
        end_corner_side: Decision::Neg,
        end_corner_spacings_between: [74.48, 74.49],
    },
    AMitre {
        edge: "#1769",
        centre: [22.1, 4.4, 10.048600289336],
        leaning_origin: [20.931443001811, 4.4, 11.217157287525],
        leaning_direction: DOWN_AND_ON,
        axes_meet_at_height: 10.048600289336,
        seam_corner: [22.1, 4.8, 10.048600289336],
        end_corner: [22.5, 4.4, 10.214285714286],
        q_exactly: 0.0,
        roundings_between: [0.0, 0.0],
        seam_corner_spacings_between: [0.0, 0.0],
        end_corner_side: Decision::Pos,
        end_corner_spacings_between: [107.60, 107.61],
    },
    AMitre {
        edge: "#2216",
        centre: [22.1, -4.4, 10.048600289336],
        leaning_origin: [20.931443001811, -4.4, 11.217157287525],
        leaning_direction: DOWN_AND_ON,
        axes_meet_at_height: 10.048600289336,
        seam_corner: [22.1, -4.8, 10.048600289336],
        end_corner: [22.5, -4.4, 10.214285714286],
        q_exactly: 0.0,
        roundings_between: [0.0, 0.0],
        seam_corner_spacings_between: [0.0, 0.0],
        end_corner_side: Decision::Pos,
        end_corner_spacings_between: [107.60, 107.61],
    },
    AMitre {
        edge: "#2523",
        centre: [0.4, -4.4, 10.048600289336],
        leaning_origin: [0.282842712475, -4.4, 9.931443001811],
        leaning_direction: UP_AND_ON,
        axes_meet_at_height: 10.048600289336,
        seam_corner: [0.4, -4.8, 10.048600289336],
        end_corner: [0.0, -4.4, 10.214285714286],
        q_exactly: 0.0,
        roundings_between: [0.0, 0.0],
        seam_corner_spacings_between: [0.0, 0.0],
        end_corner_side: Decision::Pos,
        end_corner_spacings_between: [237.89, 237.90],
    },
    AMitre {
        edge: "#2960",
        centre: [0.4, 4.4, 10.048600289336],
        leaning_origin: [0.282842712475, 4.4, 9.931443001811],
        leaning_direction: UP_AND_ON,
        axes_meet_at_height: 10.048600289336,
        seam_corner: [0.4, 4.8, 10.048600289336],
        end_corner: [0.0, 4.4, 10.214285714286],
        q_exactly: 0.0,
        roundings_between: [0.0, 0.0],
        seam_corner_spacings_between: [0.0, 0.0],
        end_corner_side: Decision::Pos,
        end_corner_spacings_between: [237.89, 237.90],
    },
];

/// One bend as the file writes it, in the file's decimals but for the far
/// disc, which is this test's own.
struct ABend {
    upright_origin: &'static str,
    upright_reference: &'static str,
    leaning_origin: &'static str,
    leaning_direction: &'static str,
    leaning_reference: &'static str,
    mitre_centre: &'static str,
    mitre_normal: &'static str,
    mitre_reference: &'static str,
    mitre_senses: [&'static str; 2],
    corner: &'static str,
    foot: &'static str,
    bottom_centre: &'static str,
    onward: &'static str,
    far_corner: &'static str,
    far_centre: &'static str,
}

const MITRE_338: ABend = ABend {
    upright_origin: "(16.471428571429,4.4,1.)",
    upright_reference: "(-1.,0.E+000,0.E+000)",
    leaning_origin: "(16.354271283903,4.4,9.931443001811)",
    leaning_direction: "(0.707106781187,0.E+000,0.707106781187)",
    leaning_reference: "(4.252662132327E-032,1.,-4.252662132327E-032)",
    mitre_centre: "(16.471428571429,4.4,10.048600289336)",
    mitre_normal: "(-0.382683432365,0.E+000,-0.923879532511)",
    mitre_reference: "(-0.923879532511,0.E+000,0.382683432365)",
    mitre_senses: [".T.", ".F."],
    corner: "(16.471428571429,4.8,10.048600289336)",
    foot: "(16.471428571429,4.8,1.)",
    bottom_centre: "(16.471428571429,4.4,1.)",
    onward: "(0.707106781187,0.E+000,0.707106781187)",
    far_corner: "(17.471428571429,4.8,11.048600289336)",
    far_centre: "(17.4714285714285,4.4,11.0486002893365)",
};

const MITRE_1769: ABend = ABend {
    upright_origin: "(22.1,4.4,3.455212154016E-032)",
    upright_reference: "(1.,0.E+000,0.E+000)",
    leaning_origin: "(20.931443001811,4.4,11.217157287525)",
    leaning_direction: "(0.707106781187,0.E+000,-0.707106781187)",
    leaning_reference: "(-4.385368252713E-032,1.,-4.385368252713E-032)",
    mitre_centre: "(22.1,4.4,10.048600289336)",
    mitre_normal: "(-0.382683432365,0.E+000,0.923879532511)",
    mitre_reference: "(0.923879532511,0.E+000,0.382683432365)",
    mitre_senses: [".F.", ".T."],
    corner: "(22.1,4.8,10.048600289336)",
    foot: "(22.1,4.8,1.)",
    bottom_centre: "(22.1,4.4,1.)",
    onward: "(-0.707106781187,0.E+000,0.707106781187)",
    far_corner: "(21.1,4.8,11.048600289336)",
    far_centre: "(21.1,4.4,11.048600289336)",
};

/// An upright pipe closed by a disc at `z = 1`, a leaning pipe closed by a
/// disc one unit on along its axis, and between them one closed `ELLIPSE`
/// whose single corner is the file's own on the `y = 4.8` side, where both
/// seams meet. The two pipes touch there, so the corner is on both, but that
/// does not put it where they cross: for `#338` it sits `1e-12` below the
/// height at which the axes meet, `3.8e-13` off both planes the pipes cross
/// in, and `how_far_the_file_writes_each_mitre_off_its_leaning_pipe` measures
/// that. For `#1769` it is on the crossing.
fn an_elbow(bend: &ABend) -> String {
    let mut text = a_header(".MILLI.");
    text += "#20=DIRECTION('',(0.E+000,0.E+000,1.));\n";
    text += &format!("#21=DIRECTION('',{});\n", bend.upright_reference);
    text += "#22=DIRECTION('',(0.E+000,0.E+000,-1.));\n";
    text += &format!("#23=DIRECTION('',{});\n", bend.leaning_direction);
    text += &format!("#24=DIRECTION('',{});\n", bend.leaning_reference);
    text += &format!("#25=DIRECTION('',{});\n", bend.mitre_normal);
    text += &format!("#26=DIRECTION('',{});\n", bend.mitre_reference);
    text += &format!("#27=DIRECTION('',{});\n", bend.onward);
    text += "#28=DIRECTION('',(0.E+000,1.,0.E+000));\n";
    text += "#29=DIRECTION('',(1.,0.E+000,0.E+000));\n";
    text += "#30=VECTOR('',#20,1.);\n";
    text += "#31=VECTOR('',#27,1.);\n";

    text += &format!("#100=CARTESIAN_POINT('',{});\n", bend.foot);
    text += &format!("#101=CARTESIAN_POINT('',{});\n", bend.corner);
    text += &format!("#102=CARTESIAN_POINT('',{});\n", bend.far_corner);
    for corner in 0..3 {
        text += &format!("#{}=VERTEX_POINT('',#{});\n", 110 + corner, 100 + corner);
    }
    text += &format!("#120=CARTESIAN_POINT('',{});\n", bend.upright_origin);
    text += &format!("#121=CARTESIAN_POINT('',{});\n", bend.mitre_centre);
    text += &format!("#122=CARTESIAN_POINT('',{});\n", bend.leaning_origin);
    text += &format!("#123=CARTESIAN_POINT('',{});\n", bend.bottom_centre);
    text += &format!("#124=CARTESIAN_POINT('',{});\n", bend.far_centre);

    text += "#130=AXIS2_PLACEMENT_3D('',#120,#20,#21);\n";
    text += "#131=AXIS2_PLACEMENT_3D('',#121,#25,#26);\n";
    text += "#132=AXIS2_PLACEMENT_3D('',#122,#23,#24);\n";
    text += "#133=AXIS2_PLACEMENT_3D('',#123,#20,#29);\n";
    text += "#134=AXIS2_PLACEMENT_3D('',#123,#22,#29);\n";
    text += "#135=AXIS2_PLACEMENT_3D('',#124,#27,#28);\n";

    text += "#140=CIRCLE('',#133,0.4);\n";
    text += "#141=ELLIPSE('',#131,0.432956880117,0.4);\n";
    text += "#142=CIRCLE('',#135,0.4);\n";
    text += "#143=LINE('',#100,#30);\n";
    text += "#144=LINE('',#101,#31);\n";
    text += "#150=EDGE_CURVE('',#110,#110,#140,.T.);\n";
    text += "#151=EDGE_CURVE('',#111,#111,#141,.T.);\n";
    text += "#152=EDGE_CURVE('',#112,#112,#142,.T.);\n";
    text += "#153=EDGE_CURVE('',#110,#111,#143,.T.);\n";
    text += "#154=EDGE_CURVE('',#111,#112,#144,.T.);\n";

    text += "#160=CYLINDRICAL_SURFACE('',#130,0.4);\n";
    text += "#161=CYLINDRICAL_SURFACE('',#132,0.4);\n";
    text += "#162=PLANE('',#134);\n";
    text += "#163=PLANE('',#135);\n";

    let [on_the_upright, on_the_leaning] = bend.mitre_senses;
    let faces: [(&str, &[(&str, &str)]); 4] = [
        (
            "#160",
            &[
                ("#150", ".T."),
                ("#153", ".T."),
                ("#151", on_the_upright),
                ("#153", ".F."),
            ],
        ),
        (
            "#161",
            &[
                ("#151", on_the_leaning),
                ("#154", ".T."),
                ("#152", ".F."),
                ("#154", ".F."),
            ],
        ),
        ("#162", &[("#150", ".F.")]),
        ("#163", &[("#152", ".T.")]),
    ];
    for (which, (surface, walked)) in faces.iter().enumerate() {
        let mut oriented = Vec::new();
        for (step, (edge, sense)) in walked.iter().enumerate() {
            let id = 200 + which * 10 + step;
            text += &format!("#{id}=ORIENTED_EDGE('',*,*,{edge},{sense});\n");
            oriented.push(format!("#{id}"));
        }
        let (bound, face) = (240 + which * 2, 260 + which);
        text += &format!("#{}=EDGE_LOOP('',({}));\n", bound, oriented.join(","));
        text += &format!("#{}=FACE_OUTER_BOUND('',#{bound},.T.);\n", bound + 1);
        text += &format!(
            "#{face}=ADVANCED_FACE('',(#{}),{surface},.T.);\n",
            bound + 1
        );
    }
    text += "#300=CLOSED_SHELL('',(#260,#261,#262,#263));\n";
    text += "#301=MANIFOLD_SOLID_BREP('',#300);\n";
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}
