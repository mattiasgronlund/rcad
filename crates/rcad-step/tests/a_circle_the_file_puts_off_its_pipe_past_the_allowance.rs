//! A `CIRCLE` the file writes where a pipe meets the ball that rounds its end,
//! centred past the reader's allowance off the pipe's axis, is named by the
//! ball. Its sibling on the same ball, centred on its own pipe's axis, keeps
//! its plane.
//!
//! `Relay_DPDT_Fujitsu_FTR-F1C.step` and `Relay_DPST_Fujitsu_FTR-F1A.step`
//! round the end of a pin with a ball of radius `0.1` at `(10.05, 1.15, 16.4)`,
//! where two pipes of the same radius meet it. One pipe stands upright, and the
//! file centres its circle `#2692` (`#2368` in `FTR-F1A`) on that pipe's axis.
//! The other runs along `y` from `(10.05, -19.75, 16.4)` and leans
//! `3.663735981263E-015` in `x` per unit, so `20.9` along it its axis is
//! `7.657e-14` off the ball's centre, and the file centres circle `#2709`
//! (under edge `#2708`; `#2385` under `#2384` in `FTR-F1A`) on the ball. That
//! is `17.42` roundings of `19.75` off the pipe in `f64`, where the reader
//! allows `16`, so the reader does not take the circle to be on the pipe and
//! names the run by the ball. A ball and a pipe that touch leave the walk two
//! pieces it cannot tell apart, and the solid is refused.
//!
//! The file writes `#2708` as a quarter arc, and the pins cut their rims in
//! two arcs for that reason. A rim written closed takes another road through
//! the reader, which imports it, so it would not pin what the files meet.
//!
//! These are limits and not gaps (`docs/decisions.md` §16.10, §335.5). No
//! circle lies on both faces: one on the pipe is centred on its axis, one on
//! the ball and of its radius is centred on its centre, and those are
//! `7.657e-14` apart. The plane the run would be named by is the same plane to
//! the last bit wherever between the two the centre is taken, and its circle on
//! the ball is `17.42` roundings off the pipe while its circle on the pipe is
//! `20.98` off the ball in `f64` (`21.03` in exact arithmetic).
//! `how_far_the_file_puts_the_circle_off_its_pipe` holds those figures.
//!
//! The two pins hold each side: widening the reader's allowance to `18` lets
//! the first pin end import and fails the first test, and the second shows
//! that the same pin end, with a circle the reader does keep, imports with its
//! arcs named by their plane.

mod written_by_hand;

use rcad_geom::{Axis, Cylinder, EdgeRun, Plane, Point, Sphere, Surface, Vector};
use rcad_predicates::{Decision, Policy, ProvenanceLog, SideOfSurface};
use rcad_step::{Import, Refusal, StepFile};
use rcad_topo::Error;
use written_by_hand::a_header;

/// The end of the leaning pipe, rounded by its ball, with the file's circle
/// `#2709` between them in the file's own decimals. The circle is on the ball
/// and `17.42` roundings off the pipe, so the reader names each arc by the
/// ball, and the walk then cannot tell the two pieces of the ball's crossing
/// with the pipe apart. Widening the reader's allowance to `18` roundings lets
/// the pin end import, and fails this test.
#[test]
fn a_circle_centred_off_its_leaning_pipe_is_named_by_the_ball() {
    let text = a_rounded_pin_end(&ROUND_THE_LEANING_PIPE);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let refused = match Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new()) {
        Ok(_) => panic!(
            "#2709 is centred 17.42 roundings off its pipe's axis, and was read as on the pipe"
        ),
        Err(refused) => refused,
    };
    assert!(
        matches!(
            &refused,
            Refusal::TheSolidTheFileDescribesIsNotOne { why, .. }
                if matches!(
                    **why,
                    Error::Geometry(rcad_geom::Error::CannotNameWhichPieceOfACrossingThisIs)
                )
        ),
        "a pin end whose circle is named by the ball and the pipe is refused for the two \
         pieces of their crossing, not {refused:?}"
    );
}

#[test]
fn probe_what_the_leaning_pin_end_imports_as() {
    let text = a_rounded_pin_end(&ROUND_THE_LEANING_PIPE);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let import = Import::read(&file, &policy, &mut log).expect("probe: imports");
    let [(_, pin)] = import.solids() else {
        panic!("one solid");
    };
    let model = import.model();
    let solid = model.solid(*pin).expect("key");
    println!("validate: {:?}", model.validate(&policy, &mut log).is_ok());
    let radius = 0.1;
    let leaning = Vector::new(3.663_735_981_263e-15, 1.0, 0.0);
    let pipe = Axis::as_written(Point::new(10.05, -19.75, 16.4), leaning)
        .and_then(|axis| Cylinder::new(axis, radius))
        .expect("pipe");
    let ball = Sphere::new(Point::new(10.05, 1.15, 16.4), radius).expect("ball");
    for face in solid.faces() {
        println!("face {:?}", face.surface());
    }
    let surfaces = [Surface::Sphere(ball), Surface::Cylinder(pipe)];
    let mut seen = Vec::new();
    for face in solid.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                if seen.contains(&edge.key()) {
                    continue;
                }
                seen.push(edge.key());
                let ends = [edge.start(), edge.end()].map(|end| end.point());
                println!("edge {:?} ends {ends:?} run {:?}", edge.key(), edge.run());
                if ends[0] == ends[1] || edge.run() == EdgeRun::Straight {
                    continue;
                }
                for deflection in [f64::MAX, 1.0, 0.1, 0.01] {
                    for along in [[&surfaces[0], &surfaces[1]]] {
                        let run = rcad_geom::NamedRun::new(
                            edge.run(),
                            ends.map(rcad_predicates::Point::from),
                        )
                        .expect("a run");
                        match rcad_geom::walk(
                            &run,
                            along,
                            rcad_geom::Deflection::of(deflection).expect("a length"),
                            &policy,
                            &mut log,
                        ) {
                            Err(e) => println!("  deflection {deflection}: walk refused {e:?}"),
                            Ok(walked) => {
                                println!(
                                    "  deflection {deflection}: {} points, deviation {:?}",
                                    walked.points().len(),
                                    walked.deviation()
                                );
                                let mut xs: Vec<f64> = Vec::new();
                                let mut tally = std::collections::BTreeMap::new();
                                for point in walked.points() {
                                    let [x, y, z] = point.coordinates();
                                    xs.push(x - 10.05);
                                    let key = format!(
                                        "ball {:?} pipe {:?}",
                                        ball.side_of((*point).into()),
                                        pipe.side_of((*point).into())
                                    );
                                    *tally.entry(key).or_insert(0) += 1;
                                    if walked.points().len() < 40 {
                                        println!(
                                            "    ({x:.17}, {y:.17}, {z:.17}) x-10.05 {:+.4e} \
                                             ball {:?} pipe {:?}",
                                            x - 10.05,
                                            ball.side_of((*point).into()),
                                            pipe.side_of((*point).into())
                                        );
                                    }
                                }
                                let most = xs.iter().copied().fold(f64::MIN, f64::max);
                                let least = xs.iter().copied().fold(f64::MAX, f64::min);
                                println!("    x-10.05 from {least:+.4e} to {most:+.4e}; {tally:?}");
                            }
                        }
                    }
                }
            }
        }
    }
    for deflection in [f64::MAX, 0.01, 0.001, 0.0001] {
        let mesh = match solid.mesh(
            rcad_geom::Deflection::of(deflection).expect("a length"),
            &policy,
            &mut log,
        ) {
            Ok(mesh) => mesh,
            Err(e) => {
                println!("deflection {deflection}: mesh refused {e:?}");
                continue;
            }
        };
        println!(
            "deflection {deflection}: deviation {:?}, {} points, {} triangles",
            mesh.deviation(),
            mesh.points().len(),
            mesh.triangles().len()
        );
        let mut six_volume = 0.0;
        for triangle in mesh.triangles() {
            let [a, b, c] = triangle
                .corners()
                .map(|at| mesh.points()[at as usize] - Point::new(10.05, 1.15, 16.4));
            six_volume += a.dot(b.cross(c));
        }
        println!("  mesh signed volume {:.10}", six_volume / 6.0);
        let mut seen = Vec::new();
        for face in solid.faces() {
            for wire in face.wires() {
                for coedge in wire.coedges() {
                    let edge = coedge.edge();
                    if seen.contains(&edge.key()) {
                        continue;
                    }
                    seen.push(edge.key());
                    let ends = [edge.start(), edge.end()].map(|end| end.point().coordinates());
                    println!("edge {:?} ends {ends:?} run {:?}", edge.key(), edge.run());
                    for &at in mesh.points_along_edge(edge.key()) {
                        let point = mesh.points()[at as usize];
                        let [x, y, z] = point.coordinates();
                        println!(
                            "  ({x:.17}, {y:.17}, {z:.17}) x-10.05 {:+.4e} ball {:?} pipe {:?}",
                            x - 10.05,
                            ball.side_of(point.into()),
                            pipe.side_of(point.into())
                        );
                    }
                }
            }
        }
        match solid.measure(
            rcad_geom::Deflection::of(deflection).expect("a length"),
            &policy,
            &mut log,
        ) {
            Ok(measured) => println!(
                "deflection {deflection}: volume {:.10} +- {:.3e}, area {:.10}",
                measured.volume(),
                measured.volume_error(),
                measured.surface_area()
            ),
            Err(e) => println!("deflection {deflection}: measure refused {e:?}"),
        }
    }
}

/// The same ball on the upright pipe, with the file's circle `#2692` between
/// them. Its centre is on the pipe's axis, so the circle is checked onto both
/// faces and each arc is named by the circle's own plane, and the pin end
/// imports.
#[test]
fn a_circle_centred_on_its_upright_pipe_is_named_by_its_plane() {
    let text = a_rounded_pin_end(&ROUND_THE_UPRIGHT_PIPE);
    let file = StepFile::read(text.as_bytes()).expect("a file this test wrote reads");
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let import = match Import::read(&file, &policy, &mut log) {
        Ok(import) => import,
        Err(refused) => {
            panic!("#2692's pin end, its circle centred on its pipe, is refused: {refused:?}")
        }
    };
    let [(_, pin)] = import.solids() else {
        panic!("one MANIFOLD_SOLID_BREP is one solid");
    };
    let model = import.model();
    let solid = model.solid(*pin).expect("this model made the key");
    let rim_ends = [[10.05, 1.25, 16.4], [10.05, 1.05, 16.4]];
    let mut rims = 0;
    for face in solid.faces() {
        for wire in face.wires() {
            for coedge in wire.coedges() {
                let edge = coedge.edge();
                let ends = [edge.start(), edge.end()].map(|end| end.point().coordinates());
                if !(ends == rim_ends || ends == [rim_ends[1], rim_ends[0]]) {
                    continue;
                }
                rims += 1;
                let EdgeRun::PartOfWhereItCrosses { and, .. } = edge.run() else {
                    panic!(
                        "the circle has been checked to lie on the ball and the pipe, so each of \
                         its arcs is named by the surface the file draws it on; it says {:?}",
                        edge.run()
                    );
                };
                let Surface::Plane(flat) = and else {
                    panic!("a circle between a ball and a pipe is named by its plane, not {and:?}");
                };
                assert_eq!(
                    flat.normal(),
                    Vector::new(0.0, 0.0, 1.0),
                    "the plane is the circle's own"
                );
                assert_eq!(
                    flat.signed_distance_to(Point::new(10.05, 1.15, 16.4)),
                    0.0,
                    "the plane runs through the circle's centre"
                );
            }
        }
    }
    assert_eq!(
        rims, 4,
        "the ball's face and the pipe's each walk the rim's two arcs once"
    );
    model
        .validate(&policy, &mut log)
        .expect("the model the reader built passes the full sweep");
}

/// The figures `docs/decisions.md` §335.5 publishes for `#2709`, worked out as
/// the reader works them out: the pipe built as `geometry.rs` builds a
/// `CYLINDRICAL_SURFACE`, and the circle's distance off it as
/// `how_far_off_the_circle_can_be` takes it, in roundings of the largest
/// number of the circle and the pipe.
///
/// Beside those, what each way out would give. The radius is the faces' own,
/// and the circle turns about the pipe's own direction, so all of the miss is
/// the centre's. The centre moved onto the pipe's axis is off the ball by as
/// much, in roundings of the largest number of the circle and the ball. The
/// plane through the centre moved halfway is the plane through the file's
/// centre, bit for bit. The corner `#1030` at the far end of the file's arc is
/// inside the pipe, and a ball and a pipe of one radius whose centre misses
/// the axis by `q` cross in a curve that is no circle: it runs
/// `√(q·(2r − q))` either way along the axis.
#[test]
fn how_far_the_file_puts_the_circle_off_its_pipe() {
    let radius = 0.1;
    let leaning = Vector::new(3.663_735_981_263e-15, 1.0, 0.0);
    let pipe = Axis::as_written(Point::new(10.05, -19.75, 16.4), leaning)
        .and_then(|axis| Cylinder::new(axis, radius))
        .expect("the file's leaning pipe is a pipe");
    let centre = Point::new(10.05, 1.15, 16.4);
    let ball = Sphere::new(centre, radius).expect("the file's ball is a ball");
    let rounding = |largest: f64| f64::EPSILON * largest;

    let q = pipe.axis().distance_to_point(centre);
    let of_the_pipe = rounding(
        centre
            .largest_coordinate_magnitude()
            .max(pipe.axis().origin().largest_coordinate_magnitude()),
    );
    let roundings = q / of_the_pipe;
    let q_exactly = 20.9 * 3.663_735_981_263e-15;
    let from_exact = (q - q_exactly) / of_the_pipe;
    let tilt = leaning.cross(pipe.axis().direction()).length();

    let on_the_axis = pipe.axis().closest_point(centre);
    let off_the_ball = (on_the_axis - ball.centre()).length();
    let of_the_ball = rounding(
        on_the_axis
            .largest_coordinate_magnitude()
            .max(ball.centre().largest_coordinate_magnitude()),
    );
    let moved_roundings = off_the_ball / of_the_ball;

    let [x, y, z] = centre.coordinates();
    let halfway = Point::new((x + on_the_axis.coordinates()[0]) / 2.0, y, z);
    let [through_the_file, through_halfway] = [centre, halfway].map(|at| {
        Plane::from_point_and_normal_as_written(at, leaning).expect("the circle's plane")
    });

    let corner = Point::new(10.15, 1.15, 16.4);
    let side = pipe.side_of(corner.into());
    let (round, along) = pipe.coordinates_of(corner);
    let corner_off = pipe.point_at(round, along).distance_to(corner);
    let corner_spacings = corner_off / rounding(corner.largest_coordinate_magnitude());

    let spread = (q * (2.0 * radius - q)).sqrt();
    println!(
        "#2709: q {q:.10e}, {roundings:.2} roundings, {from_exact:+.2} from exact; tilt {tilt:e}; \
         the centre on the axis {off_the_ball:.4e} off the ball, {moved_roundings:.2} roundings; \
         plane offsets {:e} and {:e}; corner #1030 {side:?}, {corner_off:.4e} off, \
         {corner_spacings:.2} spacings; the crossing spreads {spread:.4e}",
        through_the_file.offset(),
        through_halfway.offset()
    );

    assert_eq!(
        (radius - pipe.radius(), radius - ball.radius(), tilt),
        (0.0, 0.0, 0.0),
        "the circle has both faces' radius and turns about the pipe's own direction"
    );
    assert!(
        (17.41..=17.43).contains(&roundings),
        "#2709's centre is {roundings} roundings off its pipe's axis"
    );
    assert!(
        from_exact.abs() <= 0.05,
        "#2709's centre is {q} off its pipe's axis in f64, {from_exact} roundings from the \
         exact {q_exactly}"
    );
    assert!(
        (20.97..=20.99).contains(&moved_roundings),
        "the centre moved onto the pipe's axis is {moved_roundings} roundings off the ball"
    );
    assert_eq!(
        through_the_file.offset(),
        through_halfway.offset(),
        "the plane through the centre moved halfway to the pipe's axis is not the file's"
    );
    assert_eq!(
        side,
        Decision::Neg,
        "corner #1030 is not inside the leaning pipe"
    );
    assert!(
        (20.97..=20.99).contains(&corner_spacings),
        "corner #1030 is {corner_spacings} spacings off the leaning pipe"
    );
    assert!(
        (1.236e-7..=1.237e-7).contains(&spread),
        "a ball and a pipe {q} apart cross in a curve that runs {spread} along the axis"
    );
}

/// One rounded pin end as the file writes it, in the file's decimals but for
/// the far disc, the ball's placement and the seam across it, which are this
/// test's own.
struct APinEnd {
    centre: &'static str,
    ball_axis: &'static str,
    ball_reference: &'static str,
    rim_axis: &'static str,
    rim_reference: &'static str,
    rim_corner: &'static str,
    rim_across: &'static str,
    pole: &'static str,
    seam_axis: &'static str,
    seam_reference: &'static str,
    pipe_origin: &'static str,
    pipe_axis: &'static str,
    pipe_reference: &'static str,
    onward: &'static str,
    far_centre: &'static str,
    far_corner: &'static str,
    far_normal: &'static str,
    far_reference: &'static str,
}

/// `#2709` between ball `#3414` and the leaning pipe `#2723`, cut at the
/// file's `#2404` and the point across from it, which are both on both faces
/// because the pipe's axis misses the ball's centre in `x` only. The file's
/// other corner, `#1030`, is not used: it is inside the pipe (§335.5). The far
/// disc is at the file's other end of that pipe, `y = −23.9`.
const ROUND_THE_LEANING_PIPE: APinEnd = APinEnd {
    centre: "(10.05,1.15,16.4)",
    ball_axis: "(0.E+000,1.,0.E+000)",
    ball_reference: "(0.E+000,0.E+000,1.)",
    rim_axis: "(3.663735981263E-015,1.,0.E+000)",
    rim_reference: "(1.,-3.663735981263E-015,0.E+000)",
    rim_corner: "(10.05,1.15,16.5)",
    rim_across: "(10.05,1.15,16.3)",
    pole: "(10.05,1.25,16.4)",
    seam_axis: "(-1.,0.E+000,0.E+000)",
    seam_reference: "(0.E+000,-1.,0.E+000)",
    pipe_origin: "(10.05,-19.75,16.4)",
    pipe_axis: "(3.663735981263E-015,1.,0.E+000)",
    pipe_reference: "(0.707106781187,-2.590652556828E-015,0.707106781187)",
    onward: "(0.E+000,-1.,0.E+000)",
    far_centre: "(10.05,-23.9,16.4)",
    far_corner: "(10.05,-23.9,16.5)",
    far_normal: "(-3.663735981263E-015,-1.,0.E+000)",
    far_reference: "(0.E+000,0.E+000,1.)",
};

/// `#2692` between the same ball and the upright pipe `#2699`, cut at the
/// file's `#2685` and the point across from it, and the far disc at that
/// pipe's origin, `z = 8.25`.
const ROUND_THE_UPRIGHT_PIPE: APinEnd = APinEnd {
    centre: "(10.05,1.15,16.4)",
    ball_axis: "(0.E+000,0.E+000,1.)",
    ball_reference: "(0.E+000,1.,0.E+000)",
    rim_axis: "(0.E+000,-0.E+000,1.)",
    rim_reference: "(3.663735981263E-015,1.,0.E+000)",
    rim_corner: "(10.05,1.25,16.4)",
    rim_across: "(10.05,1.05,16.4)",
    pole: "(10.05,1.15,16.5)",
    seam_axis: "(1.,0.E+000,0.E+000)",
    seam_reference: "(0.E+000,0.E+000,-1.)",
    pipe_origin: "(10.05,1.15,8.25)",
    pipe_axis: "(0.E+000,0.E+000,1.)",
    pipe_reference: "(0.707106781187,0.707106781187,0.E+000)",
    onward: "(0.E+000,0.E+000,-1.)",
    far_centre: "(10.05,1.15,8.25)",
    far_corner: "(10.05,1.25,8.25)",
    far_normal: "(0.E+000,0.E+000,-1.)",
    far_reference: "(1.,0.E+000,0.E+000)",
};

/// A half ball of radius `0.1` on a pipe of the same radius, closed by a flat
/// disc. The rim is cut in two arcs at its corner and the point across from
/// it, as the file cuts `#2709` into arcs rather than writing it closed. The
/// ball's face is bounded by the two arcs and a quarter circle across the ball
/// from the corner to its pole and back, as the LED domes in
/// `an_led_dome_seam_is_cut_once.rs` are. The rim, the pipe and the ball's
/// centre are the file's.
fn a_rounded_pin_end(end: &APinEnd) -> String {
    let mut text = a_header(".MILLI.");
    text += &format!(
        "#10=CARTESIAN_POINT('',{centre});\n\
         #11=DIRECTION('',{ball_axis});\n\
         #12=DIRECTION('',{ball_reference});\n\
         #13=DIRECTION('',{rim_axis});\n\
         #14=DIRECTION('',{rim_reference});\n\
         #15=DIRECTION('',{seam_axis});\n\
         #16=DIRECTION('',{seam_reference});\n\
         #17=DIRECTION('',{pipe_axis});\n\
         #18=DIRECTION('',{pipe_reference});\n\
         #19=DIRECTION('',{onward});\n\
         #20=CARTESIAN_POINT('',{rim_corner});\n\
         #21=VERTEX_POINT('',#20);\n\
         #22=CARTESIAN_POINT('',{pole});\n\
         #23=VERTEX_POINT('',#22);\n\
         #24=CARTESIAN_POINT('',{far_corner});\n\
         #25=VERTEX_POINT('',#24);\n\
         #26=CARTESIAN_POINT('',{pipe_origin});\n\
         #27=CARTESIAN_POINT('',{far_centre});\n\
         #28=DIRECTION('',{far_normal});\n\
         #29=DIRECTION('',{far_reference});\n\
         #44=CARTESIAN_POINT('',{rim_across});\n\
         #45=VERTEX_POINT('',#44);\n\
         #30=AXIS2_PLACEMENT_3D('',#10,#13,#14);\n\
         #31=CIRCLE('',#30,0.1);\n\
         #32=EDGE_CURVE('',#21,#45,#31,.T.);\n\
         #39=EDGE_CURVE('',#45,#21,#31,.T.);\n\
         #40=AXIS2_PLACEMENT_3D('',#10,#15,#16);\n\
         #41=CIRCLE('',#40,0.1);\n\
         #42=EDGE_CURVE('',#21,#23,#41,.T.);\n\
         #50=ORIENTED_EDGE('',*,*,#32,.T.);\n\
         #58=ORIENTED_EDGE('',*,*,#39,.T.);\n\
         #51=ORIENTED_EDGE('',*,*,#42,.T.);\n\
         #52=ORIENTED_EDGE('',*,*,#42,.F.);\n\
         #53=EDGE_LOOP('',(#50,#58,#51,#52));\n\
         #54=FACE_BOUND('',#53,.T.);\n\
         #55=AXIS2_PLACEMENT_3D('',#10,#11,#12);\n\
         #56=SPHERICAL_SURFACE('',#55,0.1);\n\
         #57=ADVANCED_FACE('',(#54),#56,.T.);\n\
         #33=VECTOR('',#19,1.);\n\
         #34=LINE('',#20,#33);\n\
         #35=EDGE_CURVE('',#21,#25,#34,.T.);\n\
         #36=AXIS2_PLACEMENT_3D('',#27,#13,#14);\n\
         #37=CIRCLE('',#36,0.1);\n\
         #38=EDGE_CURVE('',#25,#25,#37,.T.);\n\
         #59=ORIENTED_EDGE('',*,*,#39,.F.);\n\
         #60=ORIENTED_EDGE('',*,*,#32,.F.);\n\
         #61=ORIENTED_EDGE('',*,*,#35,.T.);\n\
         #62=ORIENTED_EDGE('',*,*,#38,.T.);\n\
         #63=ORIENTED_EDGE('',*,*,#35,.F.);\n\
         #64=EDGE_LOOP('',(#59,#60,#61,#62,#63));\n\
         #65=FACE_BOUND('',#64,.T.);\n\
         #66=AXIS2_PLACEMENT_3D('',#26,#17,#18);\n\
         #67=CYLINDRICAL_SURFACE('',#66,0.1);\n\
         #68=ADVANCED_FACE('',(#65),#67,.T.);\n\
         #80=ORIENTED_EDGE('',*,*,#38,.F.);\n\
         #81=EDGE_LOOP('',(#80));\n\
         #82=FACE_BOUND('',#81,.T.);\n\
         #83=AXIS2_PLACEMENT_3D('',#27,#28,#29);\n\
         #84=PLANE('',#83);\n\
         #85=ADVANCED_FACE('',(#82),#84,.T.);\n\
         #90=CLOSED_SHELL('',(#57,#68,#85));\n\
         #91=MANIFOLD_SOLID_BREP('',#90);\n",
        centre = end.centre,
        ball_axis = end.ball_axis,
        ball_reference = end.ball_reference,
        rim_axis = end.rim_axis,
        rim_reference = end.rim_reference,
        seam_axis = end.seam_axis,
        seam_reference = end.seam_reference,
        pipe_axis = end.pipe_axis,
        pipe_reference = end.pipe_reference,
        onward = end.onward,
        rim_corner = end.rim_corner,
        rim_across = end.rim_across,
        pole = end.pole,
        far_corner = end.far_corner,
        pipe_origin = end.pipe_origin,
        far_centre = end.far_centre,
        far_normal = end.far_normal,
        far_reference = end.far_reference,
    );
    text + "ENDSEC;\nEND-ISO-10303-21;\n"
}
