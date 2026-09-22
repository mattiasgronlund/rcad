//! A pointed cone's `classify_point`, held against the cone itself.
//!
//! `docs/decisions.md` §200.3 found it answering `Outside` at points inside the
//! cone and refusing `CannotFollowAFacesTrimming` at 3444 more over the grid
//! below. **Two causes, both fixed.**
//!
//! 1. **The cone face could not say which side of its cap it lies on.** Its one
//!    corner sits on the cap, and that corner is all `the_side_this_face_lies_on`
//!    reads, so every point asked of the face refused. And once it could say,
//!    the side alone would have held the *other* half of the cone too — the
//!    whole of it lies below the cap. Fixed in `holds.rs`: a face on a cone
//!    keeps to the half its corners are on, and a cone face trimmed by one
//!    closed curve round the axis is the piece reaching the apex.
//! 2. **`SolidView::box_around_it` stops at `z = 4`**, half way up. A cone
//!    face's box is opened out from its corners' shadows on the axis, and a
//!    pointed cone's one corner is on its cap; the apex is not a corner. So
//!    every point below `z = 4` is outside the box and answers `Outside` before
//!    any ray is fired, and the fuse's broad phase, which reads the same boxes
//!    per face, looked at no pair of faces with the side-4 cube round the apex
//!    and handed both solids back untouched. Fixed in `view.rs`: a cone face's
//!    box takes in the apex.

use rcad_geom::{Axis, Deflection, Isometry, Point, Scale, Vector};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, Model, OnAFace, ProfileBuilder, Rule, SolidKey};

mod cones;

use cones::a_pointed_cone;

const SLOPE: f64 = 0.5;
const HEIGHT: f64 = 8.0;

/// The skin of the cone holds the half it is on and not the half mirrored
/// through the apex, which every plane below the cap puts on the same side.
#[test]
fn the_skin_of_a_pointed_cone_holds_its_own_half_and_not_the_other() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);
    let view = model.solid(cone).expect("this model made the key");
    let skin = view
        .faces()
        .find(|face| face.surface().as_plane().is_none())
        .expect("a pointed cone has a curved face")
        .key();
    let holds = |at: Point| {
        view.face_holds_the_point(
            skin,
            PredicatePoint::from(at.coordinates()),
            &policy,
            &mut ProvenanceLog::new(),
        )
        .expect("the skin of a pointed cone says")
    };

    assert_eq!(holds(Point::new(2.0, 0.0, 4.0)), OnAFace::Held);
    assert_eq!(holds(Point::new(0.0, 1.0, 2.0)), OnAFace::Held);
    assert_eq!(holds(Point::new(-2.0, 0.0, -4.0)), OnAFace::NotHeld);
    assert_eq!(holds(Point::new(0.0, -1.0, -2.0)), OnAFace::NotHeld);
    assert_eq!(holds(Point::origin()), OnAFace::OnItsBoundary);
}

/// The box round a pointed cone reaches down to its apex. It stopped at
/// `z = 4` — the one corner's height less its radius — before the apex was
/// taken in.
#[test]
fn the_box_round_a_pointed_cone_reaches_its_apex() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);
    let (low, high) = model
        .solid(cone)
        .expect("this model made the key")
        .box_around_it()
        .expect("a pointed cone has faces");
    assert!(low.z() <= 0.0, "the box starts at {low:?}");
    assert!(high.z() >= HEIGHT, "the box stops at {high:?}");
}

/// The three points `docs/decisions.md` §200.3 named are inside the cone.
#[test]
fn the_points_below_half_way_that_answered_outside_are_inside() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);
    let view = model.solid(cone).expect("this model made the key");
    for at in [
        Point::new(0.0, 0.0, 1.0),
        Point::new(0.1, 0.07, 1.03),
        Point::new(-1.4, 0.07, 3.03),
    ] {
        assert_eq!(
            view.classify_point(at, &policy, &mut ProvenanceLog::new()),
            Containment::Inside,
            "at {at:?}"
        );
    }
}

/// Every point of the half-unit grid, over the whole cone and past it on every
/// side, is answered, and answered right.
///
/// Before the box reached the apex the grid stopped half way up, because below
/// `z = 4` every point answered `Outside`. Before the skin could say which side
/// of its cap it lies on, every point whose ray met the skin was refused.
#[test]
fn every_point_of_a_grid_round_a_pointed_cone_is_answered_right() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);

    let tally = classify_the_grid(&model, cone, &policy, |_| true);

    assert_eq!(
        tally,
        Tally {
            inside: 991,
            outside: 29634,
            wrong: 0,
            refused: 0,
        }
    );
}

/// §200.3's fuse: the pointed cone and a side-4 cube stood on its corner round
/// the apex. Both rules answered `Ok` with both solids untouched, because the
/// cone's box stopped at `z = 4`, the cube's reaches `z = 2√3 ≈ 3.46`, and a
/// broad phase reading two boxes that miss looks at no pair of faces.
///
/// Now neither answers wrong. **Both refuse**, as the cone's other tallies do
/// for a cut the fuse cannot place, so this pins the refusal and holds any
/// answer that does come back against the shapes: the shared volume by a column
/// sum, which moves by `4.1e-5` from 400 to 800 columns a side and by `1.1e-5`
/// from 800 to 1600, so `1e-3` beside `measure`'s own bound is far outside what
/// the sum itself is unsure of.
#[test]
fn a_pointed_cone_and_a_cube_round_its_apex_are_never_handed_back_untouched() {
    let policy = Policy::certified();
    let cone_volume = core::f64::consts::PI * SLOPE * SLOPE * HEIGHT.powi(3) / 3.0;
    let shared = shared_volume(800);
    let mut outcomes = Vec::new();
    for (rule, want) in [
        (Rule::Intersection, shared),
        (Rule::Union, cone_volume + CUBE_SIDE.powi(3) - shared),
    ] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let cone = a_pointed_cone(&mut model, SLOPE, HEIGHT);
        let cube = a_corner_up_cube(&mut model);
        let answer = match model.fuse(&[cone], &[cube], rule, &policy, &mut ProvenanceLog::new()) {
            Ok(answer) => answer,
            Err(rcad_topo::Error::Undecidable { records }) => {
                let record = records.records().last().expect("a refusal names why");
                outcomes.push(format!("{rule:?}: {}", record.predicate()));
                continue;
            }
            Err(other) => panic!("{rule:?}: refused as {other:?}"),
        };
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .expect("the answer validates");
        assert_eq!(answer.solids().len(), 1, "{rule:?}: one piece");
        let measured = model
            .solid(answer.solids()[0])
            .expect("this model made the key")
            .measure(
                Deflection::of(0.002).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("the piece measures");
        let slack = measured.volume_error() + 1e-3;
        assert!(
            (measured.volume() - want).abs() <= slack,
            "{rule:?}: volume {}, where the shapes say {want}",
            measured.volume()
        );
        outcomes.push(format!("{rule:?}: right"));
    }
    assert_eq!(
        outcomes,
        [
            "Intersection: a point inside a piece of a face",
            "Union: a point inside a piece of a face",
        ]
    );
}

/// A cube of side 4 stood on its corner, its diagonal upright and its middle on
/// the apex — `docs/decisions.md` §200.3's cube.
fn a_corner_up_cube(model: &mut Model) -> SolidKey {
    let policy = Policy::certified();
    let diagonal = Axis::new(Point::origin(), Vector::new(1.0, -1.0, 0.0)).expect("a direction");
    let stood = Isometry::rotation(&diagonal, (1.0f64 / 3.0).sqrt().acos()).expect("a finite turn");
    let placed = Isometry::translation(Vector::new(0.0, 0.0, -CUBE_SIDE * 3.0f64.sqrt() / 2.0))
        .expect("a finite move")
        .compose(&stood);
    let square = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [CUBE_SIDE, 0.0],
        [CUBE_SIDE, CUBE_SIDE],
        [0.0, CUBE_SIDE],
    ])
    .build(&policy, &mut ProvenanceLog::new())
    .expect("a square is a profile");
    model
        .add_extrusion(&square, CUBE_SIDE, &placed)
        .expect("a square extrudes")
}

const CUBE_SIDE: f64 = 4.0;

/// The volume the cone and the cube share, summed over `steps × steps`
/// vertical columns across the cone's widest circle: along each column the cone
/// is one interval in `z`, and the cube, being convex, is another.
fn shared_volume(steps: u32) -> f64 {
    let reach = SLOPE * HEIGHT;
    let h = 2.0 * reach / f64::from(steps);
    let diagonal = Axis::new(Point::origin(), Vector::new(1.0, -1.0, 0.0)).expect("a direction");
    let stood = Isometry::rotation(&diagonal, (1.0f64 / 3.0).sqrt().acos()).expect("a finite turn");
    let placed = Isometry::translation(Vector::new(0.0, 0.0, -CUBE_SIDE * 3.0f64.sqrt() / 2.0))
        .expect("a finite move")
        .compose(&stood);
    let inverse = placed.inverse();
    let up = inverse
        .apply_to_vector(Vector::new(0.0, 0.0, 1.0))
        .components();
    let mut total = 0.0;
    for i in 0..steps {
        for j in 0..steps {
            let x = -reach + (f64::from(i) + 0.5) * h;
            let y = -reach + (f64::from(j) + 0.5) * h;
            let (mut from, mut to) = (x.hypot(y) / SLOPE, HEIGHT);
            let at = inverse.apply_to_point(Point::new(x, y, 0.0)).coordinates();
            for k in 0..3 {
                let one = -at[k] / up[k];
                let other = (CUBE_SIDE - at[k]) / up[k];
                from = from.max(one.min(other));
                to = to.min(one.max(other));
            }
            if to > from {
                total += (to - from) * h * h;
            }
        }
    }
    total
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Tally {
    inside: usize,
    outside: usize,
    wrong: usize,
    refused: usize,
}

/// Points half a unit apart over `|x|, |y| ≤ 6`, `|z| ≤ 12`, nudged off the
/// lattice so none lands on the axis, the cap or the skin — the grid §200.3
/// counted over — classified against the cone and held against its closed
/// form: inside exactly where `0 < z < 8` and the distance from the axis is
/// under `z / 2`.
fn classify_the_grid(
    model: &Model,
    cone: SolidKey,
    policy: &Policy,
    keep: impl Fn(Point) -> bool,
) -> Tally {
    let view = model.solid(cone).expect("this model made the key");
    let mut tally = Tally::default();
    for i in -12..=12 {
        for j in -12..=12 {
            for k in -24..=24 {
                let p = Point::new(
                    f64::from(i) * 0.5 + 0.1,
                    f64::from(j) * 0.5 + 0.07,
                    f64::from(k) * 0.5 + 0.03,
                );
                if !keep(p) {
                    continue;
                }
                let inside = 0.0 < p.z() && p.z() < HEIGHT && p.x().hypot(p.y()) < SLOPE * p.z();
                match view.classify_point(p, policy, &mut ProvenanceLog::new()) {
                    Containment::Inside if inside => tally.inside += 1,
                    Containment::Outside if !inside => tally.outside += 1,
                    Containment::Inside | Containment::Outside | Containment::OnBoundary => {
                        tally.wrong += 1;
                    }
                    Containment::Undecidable { .. }
                    | Containment::CannotFollowAFacesTrimming { .. }
                    | Containment::CannotFireARayAtAFacesSurface { .. }
                    | _ => tally.refused += 1,
                }
            }
        }
    }
    tally
}
