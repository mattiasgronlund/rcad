//! A cone's skin cut by two walls along its axis, each wall's hyperbola never
//! reaching the other wall (row Q1235).
//!
//! The pointed cone of slope 1, apex at the origin and base at `z = 20`, is
//! cut by the block `[−12, 24] × [−24, 15] × [−1, 18]`. What is left of its
//! skin is the upper nappe below `z = 18` with `x > −12` and `y < 15`: one
//! piece, since the two walls' parts of the disc of radius 18 seen down the
//! axis are apart, their corner `(−12, 15)` being `√369` from the axis. Its
//! sides are the hyperbola on `x = −12` from `(−12, ±√180, 18)` down through
//! `(−12, 0, 12)`, the one on `y = 15` from `(±√99, 15, 18)` down through
//! `(0, 15, 15)`, and two arcs of the top circle between them.
//!
//! **Each hyperbola never reaches the other wall**: `x = −12`'s runs over
//! `|y| ≤ √180`, short of `15`, and `y = 15`'s over `|x| ≤ √99`, short of `12`.
//! The line where the two walls meet runs up the axis, steeper than the cone's
//! side, so it meets each wall's hyperbola once on each branch. `holds.rs` read
//! such a line only where the curve's part of degree two is positive along it,
//! so each hyperbola read as crossing the other wall, and the steps that read
//! the skin as the part in front of its three cuts said nothing. The winding
//! seen down the axis answers every other point of the skin, but not the apex,
//! which is where every ruling is seen, so the apex was refused
//! `AFaceTrimmedByACurveThisCannotFollow` and `classify_point` there answered
//! `CannotFollowAFacesTrimming`, though the apex is on the skin.

use rcad_geom::{Isometry, Point, Scale, Surface, Vector};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Containment, Model, OnAFace, ProfileBuilder, Rule, SolidKey};

mod cones;

/// **The skin of a cone cut by two walls along its axis holds its own points,
/// the apex among them**, which is on its boundary as the uncut cone's apex is
/// (`a_pointed_cone_knows_its_inside.rs`). Every point asked is on the cone
/// exactly. The apex was refused until each wall's hyperbola was read against
/// the other wall's plane along a line steeper than the cone's side (Q1235);
/// the rest were answered before, seen down the axis, and are answered the
/// same now.
#[test]
fn the_skin_of_a_cone_cut_by_two_walls_along_its_axis_holds_its_apex() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cut = a_cone_cut_by_two_walls_along_its_axis(&mut model);
    let solid = model.solid(cut).expect("this model made the key");
    let skin = solid
        .faces()
        .find(|face| matches!(face.surface(), Surface::Cone(_)))
        .expect("the cut cone has a skin");
    let corners: Vec<[f64; 3]> = skin
        .wires()
        .flat_map(|wire| wire.points().collect::<Vec<_>>())
        .map(|corner| corner.coordinates())
        .collect();
    assert_eq!(
        corners
            .iter()
            .filter(|corner| corner[0] == -12.0 && corner[2] == 18.0)
            .count(),
        2,
        "{corners:?}"
    );
    assert_eq!(
        corners
            .iter()
            .filter(|corner| corner[1] == 15.0 && corner[2] == 18.0)
            .count(),
        2,
        "{corners:?}"
    );
    assert_eq!(corners.len(), 4, "{corners:?}");
    let asked = |at: [f64; 3]| {
        solid
            .face_holds_the_point(
                skin.key(),
                PredicatePoint::from(at),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{at:?} is answered: {why:?}"))
    };
    assert_eq!(asked([0.0, 0.0, 0.0]), OnAFace::OnItsBoundary);
    assert_eq!(asked([3.0, 4.0, 5.0]), OnAFace::Held);
    assert_eq!(asked([-9.0, 12.0, 15.0]), OnAFace::Held);
    assert_eq!(asked([9.0, 12.0, 15.0]), OnAFace::Held);
    assert_eq!(asked([-12.0, 9.0, 15.0]), OnAFace::OnItsBoundary);
    assert_eq!(asked([0.0, 15.0, 15.0]), OnAFace::OnItsBoundary);
    assert_eq!(asked([-13.0, 0.0, 13.0]), OnAFace::NotHeld);
    assert_eq!(asked([0.0, 16.0, 16.0]), OnAFace::NotHeld);
    assert_eq!(asked([12.0, 16.0, 20.0]), OnAFace::NotHeld);
    assert_eq!(asked([0.0, 5.0, -5.0]), OnAFace::NotHeld);
}

/// **The apex of the cut cone is on its boundary**, where `classify_point`
/// answered `CannotFollowAFacesTrimming` naming the skin until Q1235. A point
/// up the axis inside it and one below the apex are answered too.
#[test]
fn the_apex_of_a_cone_cut_by_two_walls_along_its_axis_is_on_its_boundary() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cut = a_cone_cut_by_two_walls_along_its_axis(&mut model);
    let solid = model.solid(cut).expect("this model made the key");
    let classified =
        |at: Point| solid.classify_point(at, &Policy::certified(), &mut ProvenanceLog::new());
    assert_eq!(classified(Point::origin()), Containment::OnBoundary);
    assert_eq!(classified(Point::new(0.0, 0.0, 9.0)), Containment::Inside);
    assert_eq!(classified(Point::new(0.0, 0.0, -1.0)), Containment::Outside);
}

/// The pointed cone of slope 1 with its base at `z = 20`, apex at the origin,
/// less all but the block `[−12, 24] × [−24, 15] × [−1, 18]`, whose walls
/// `x = −12` and `y = 15` and top `z = 18` cut it and whose other faces are
/// clear of it.
fn a_cone_cut_by_two_walls_along_its_axis(model: &mut Model) -> SolidKey {
    let cone = cones::a_pointed_cone(model, 1.0, 20.0);
    let profile = ProfileBuilder::new(vec![
        [-12.0, -24.0],
        [24.0, -24.0],
        [24.0, 15.0],
        [-12.0, 15.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a simple outline");
    let block = model
        .add_extrusion(
            &profile,
            19.0,
            &Isometry::translation(Vector::new(0.0, 0.0, -1.0)).expect("a whole-number shift"),
        )
        .expect("a block");
    let answer = model
        .fuse(
            &[cone],
            &[block],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the intersection refused: {why}"));
    let [solid] = answer.solids() else {
        panic!("the intersection is one solid, not {:?}", answer.solids());
    };
    *solid
}
