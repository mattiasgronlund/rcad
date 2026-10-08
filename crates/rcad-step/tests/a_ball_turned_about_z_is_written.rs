//! A whole ball turned about `z` is written at every tenth of a degree, and
//! comes back from its own file as the same ball (Q1308).
//!
//! `add_ball(3)` turned about `z` and moved to `(10, −6, 5)` was refused by
//! name, `ACurveTheFacesEitherSideOfItDoNotName { first: "ball", second:
//! "ball" }`, at 107 of the 3 600 tenths of a degree, all from 142.6 to 155.2
//! and from 315.6 to 334.6. There its seam is cut from a plane passing near
//! the origin, the reader is given that plane only through the centre of the
//! seam's circle, and no centre it holds on the ball read back as the plane to
//! the bit. The writer now writes such a ball cut open where `add_ball` cuts
//! it, as it writes a ball turned off `z` (`docs/decisions.md` §404); a whole
//! ball covers the sphere wherever its seam runs.

use rcad_geom::{
    Axis, Deflection, EdgeRun, HalfSpace, Isometry, Picker, Plane, Point, Scale, Sphere, Surface,
    Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Model, WireStepView};

/// Every tenth of a degree is written and read back: the same surface, the
/// same two poles to the bit, a model that validates, and the seam either as
/// it went in — its plane the same bits and its picker on the same side of the
/// poles — or cut where `add_ball` cuts it, and then it measures. The second
/// is exactly the 107 the writer refused, and no other.
#[test]
fn a_ball_turned_about_z_by_any_tenth_of_a_degree_comes_back_from_its_own_file_as_the_same_ball() {
    let policy = Policy::certified();
    let mut cut_where_add_ball_cuts_it = Vec::new();
    for tenths in 0..3600_u32 {
        let degrees = f64::from(tenths) / 10.0;
        let placement = Isometry::translation(Vector::new(10.0, -6.0, 5.0))
            .expect("a finite move")
            .compose(&Isometry::rotation(&Axis::z(), degrees.to_radians()).expect("a finite turn"));
        let mut model = Model::new(Scale::millimetres(100.0).expect("a scale"));
        model
            .add_ball(3.0, &placement)
            .expect("a ball of radius three");
        let text = rcad_step::write(&model, "ball")
            .unwrap_or_else(|why| panic!("turned {degrees}°: the writer refused, {why:?}"));
        let file = StepFile::read(text.as_bytes()).expect("this writer's own text parses");
        let import = Import::read(&file, &policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("turned {degrees}°: the reader refused, {why}"));
        let back = import.model();
        back.validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("turned {degrees}°: came back unsound, {why}"));

        let went_in = the_ball_cut_open(&model);
        let came_back = the_ball_cut_open(back);
        assert_eq!(
            format!("{:?}", went_in.ball),
            format!("{:?}", came_back.ball),
            "turned {degrees}°: a different ball came back"
        );
        assert!(
            came_back.poles == went_in.poles
                || came_back.poles == [went_in.poles[1], went_in.poles[0]],
            "turned {degrees}°: poles {:?} came back for {:?}",
            came_back.poles,
            went_in.poles
        );
        if came_back.seam != went_in.seam {
            assert_eq!(
                came_back.seam,
                add_balls_seam(went_in.ball),
                "turned {degrees}°: the seam came back neither as it went in nor where add_ball \
                 cuts it"
            );
            cut_where_add_ball_cuts_it.push(degrees);
            let solid = back.solids().next().expect("one solid came back");
            solid
                .measure(
                    Deflection::of(0.031).expect("a deflection"),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| {
                    panic!("turned {degrees}°: came back cut again and does not measure, {why}")
                });
        }
    }
    assert_eq!(
        cut_where_add_ball_cuts_it.len(),
        107,
        "the balls whose seam no file carries are the 107 the writer refused: {cut_where_add_ball_cuts_it:?}"
    );
    assert!(
        cut_where_add_ball_cuts_it.iter().all(
            |&degrees| (142.6..=155.2).contains(&degrees) || (315.6..=334.6).contains(&degrees)
        ),
        "a seam was cut again outside the two bands the writer refused: {cut_where_add_ball_cuts_it:?}"
    );
}

/// What a whole ball is told by: its surface, its two poles in walk order,
/// and its seam.
struct CutOpen {
    ball: Sphere,
    poles: [Point; 2],
    seam: Seam,
}

/// A ball's seam as the half of a great circle it is: the plane it is cut
/// from, to the bit, and which side of the line through the poles the picker
/// faces.
///
/// The picker is told by its side and not its bits. The kernel's is `(1, 0,
/// 0)` turned, the reader's is built from the circle (`Walk::the_arc_between`),
/// and both pick the same half without being along each other.
#[derive(Debug, PartialEq)]
struct Seam {
    plane: String,
    faces_the_side_its_normal_turns_to: bool,
}

fn the_ball_cut_open(model: &Model) -> CutOpen {
    let solid = model.solids().next().expect("one solid");
    let face = solid.faces().next().expect("one face");
    let Surface::Sphere(ball) = face.surface() else {
        panic!("a ball's face sits on a ball, not {:?}", face.surface());
    };
    let wire = face.wires().next().expect("one wire");
    let mut poles = Vec::new();
    let mut runs = Vec::new();
    for step in wire.steps() {
        match step {
            WireStepView::Pole(pole) => poles.push(pole.point()),
            WireStepView::Along(coedge) => runs.push(coedge.edge().run()),
            _ => panic!("a ball's wire walks its seam and stands at its poles"),
        }
    }
    let (&[north, south], [run, _]) = (poles.as_slice(), runs.as_slice()) else {
        panic!("a whole ball is two poles and its seam walked twice: {poles:?}, {runs:?}");
    };
    CutOpen {
        ball,
        poles: [north, south],
        seam: the_seam(run),
    }
}

fn the_seam(run: &EdgeRun) -> Seam {
    let EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(cut),
        picked_by,
    } = run
    else {
        panic!("a ball's seam is cut from a plane, not {run:?}");
    };
    let [Picker::InFrontOf(picker)] = picked_by.as_slice() else {
        panic!("a ball's seam is picked by one half-space, not {picked_by:?}");
    };
    let across = cut.normal().cross(Vector::new(0.0, 0.0, 1.0));
    let side = picker.normal().dot(across);
    assert!(side != 0.0, "a seam's picker stands square to its own half");
    Seam {
        plane: format!("{cut:?}"),
        faces_the_side_its_normal_turns_to: side > 0.0,
    }
}

/// The seam `add_ball` cuts, moved to `ball`'s centre: the plane through it
/// square to `y`, on the side facing `+x`.
fn add_balls_seam(ball: Sphere) -> Seam {
    let centre = ball.centre();
    let cut = Plane::from_point_and_normal(centre, Vector::new(0.0, 1.0, 0.0))
        .expect("the y axis is a unit normal");
    let facing_x = HalfSpace::through_a_point_facing(centre, Vector::new(1.0, 0.0, 0.0))
        .expect("the x axis has length");
    the_seam(&EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(cut),
        picked_by: [Picker::from(facing_x)].into_iter().collect(),
    })
}
