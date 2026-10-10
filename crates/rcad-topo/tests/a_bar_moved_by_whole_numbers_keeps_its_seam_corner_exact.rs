//! A bar moved by whole numbers keeps its seam corner exact.
//!
//! A round bar's cap corner is read out of the model and moved rather than
//! solved for, and the move used to cost it a box every time: half a width of
//! `CORNER_AFTER_MOVING_ROUNDOFF` times its size, whatever the move was. A shift
//! by whole numbers rounds nothing, so the box hid an exact answer. A bar whose
//! seam lies exactly on a block's wall then asked *side of plane* about a box
//! round `(0, 2, 1)` on `x = 0`, and refused, where the answer is `Zero`.
//!
//! This is a gap and not a limit (§16.10): the corner is exactly representable,
//! and only the box hid it.

#![forbid(unsafe_code)]

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Cylinder, Deflection, Isometry, Plane, Point, Scale, Surface, Vector,
    where_a_corner_lands_after_moving,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// How far under the closed form a bar's meshed volume may be.
///
/// A mesh at [`A_DEFLECTION`] round a circle of radius one is 0.33% short of
/// it. A ceiling on a measurement, not a tolerance anything is decided by.
const HOW_FAR_A_MESH_MAY_BE: f64 = 0.005;

/// How fine the meshes the volumes are read off are.
const A_DEFLECTION: f64 = 0.02;

/// The seam corner of a bar of radius one, moved by whole numbers, lands on
/// the whole-number point it was moved to and on no box round it; moved by a
/// tenth, which rounds, it keeps its box.
#[test]
fn a_bar_moved_by_whole_numbers_lands_its_seam_corner_on_an_exact_point() {
    let (corner, [cap, wall_of_the_bar]) = the_seam_corner_of_a_bar_and_its_surfaces();
    let on = [&cap, &wall_of_the_bar];

    let whole = Isometry::translation(Vector::new(-1.0, 2.0, 1.0)).expect("a whole-number shift");
    let (landed, wobble) = where_a_corner_lands_after_moving(corner, on, &whole)
        .expect("a finite corner and a finite shift");
    assert_eq!(
        landed,
        PredicatePoint::from([0.0, 2.0, 1.0]),
        "a whole-number shift of (1, 0, 0) rounds nothing"
    );
    assert_eq!(wobble.magnitude(), 0.0);

    let wall = Plane::from_point_and_normal(Point::origin(), Vector::new(1.0, 0.0, 0.0))
        .expect("the x axis is a unit normal");
    assert_eq!(
        wall.side_of_point(landed),
        Decision::Zero,
        "the seam corner is on the wall x = 0"
    );

    let a_tenth = Isometry::translation(Vector::new(0.1, 2.0, 1.0)).expect("a finite shift");
    let (boxed, wobble) = where_a_corner_lands_after_moving(corner, on, &a_tenth)
        .expect("a finite corner and a finite shift");
    assert!(
        boxed.uncertainty() > 0.0 && wobble.magnitude() > 0.0,
        "1 + 0.1 rounds, so the corner is a box and not a point: {boxed:?}"
    );
}

/// A bar standing with its seam on a block's wall, handed over first: every
/// rule used to stop on *side of plane*, asked about the seam corner's box, and
/// none is refused on it now.
///
/// `ABarBesideAWall { side: 4, radius: 1, along: 2, on_its_seam: true }` in
/// `fuses_that_touch.rs`. The bar fills `x` from −2 to 0 and the cube `x` from 0
/// to 4, so they meet only along the seam `(0, 2, z)`: the union is the two
/// solids as they stand, two lumps of `π · 1² · 2` and 64, the intersection is
/// no solid, and the difference is the bar whole.
#[test]
fn a_bar_with_its_seam_on_a_blocks_wall_is_not_refused_on_which_side_of_the_wall_its_corner_is() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let cube = a_cube(&mut model, 4.0);
    let bar = model
        .add_round_bar(
            1.0,
            2.0,
            &Isometry::translation(Vector::new(-1.0, 2.0, 1.0)).expect("a whole-number shift"),
        )
        .expect("a whole-number bar");
    let the_bar = PI * 1.0 * 1.0 * 2.0;

    let mut joined = model.clone();
    let both = answered(&mut joined, bar, cube, Rule::Union);
    let mut found: Vec<f64> = both
        .iter()
        .map(|&solid| volume_of(&joined, solid))
        .collect();
    found.sort_by(f64::total_cmp);
    assert!(
        matches!(found[..], [bar, cube]
            if bar <= the_bar && (the_bar - bar) / the_bar < HOW_FAR_A_MESH_MAY_BE
                && cube == 64.0),
        "the union is the bar and the cube as they stand, measured {found:?}"
    );

    let mut shared = model.clone();
    assert!(
        answered(&mut shared, bar, cube, Rule::Intersection).is_empty(),
        "a bar and a cube whose insides are apart share no solid"
    );

    let mut cut = model.clone();
    let what_is_left = answered(&mut cut, bar, cube, Rule::Difference);
    assert_eq!(what_is_left.len(), 1, "the cube takes nothing off the bar");
    let found = volume_of(&cut, what_is_left[0]);
    assert!(
        found <= the_bar && (the_bar - found) / the_bar < HOW_FAR_A_MESH_MAY_BE,
        "the bar less the cube measured {found}, and the bar is {the_bar}"
    );
}

#[test]
fn probe_split_floor() {
    let mut out = Vec::new();
    for shift in [[0.0, 2.0, 1.0], [1.0, 2.0, 1.0], [4.0, 2.0, 1.0]] {
        for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
            for bar_first in [true, false] {
                let mut model =
                    Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
                let bar = a_bar_with_a_split_floor(&mut model);
                model
                    .place_solid(bar, &Isometry::translation(Vector::from(shift)).expect("s"))
                    .expect("k");
                let cube = a_cube(&mut model, 4.0);
                model
                    .place_solid(
                        cube,
                        &Isometry::translation(Vector::new(1.0 + shift[0], 0.0, 0.0)).expect("s"),
                    )
                    .expect("k");
                let (a, b) = if bar_first { (bar, cube) } else { (cube, bar) };
                let mut log = ProvenanceLog::new();
                let answer = model.fuse(&[a], &[b], rule, &Policy::certified(), &mut log);
                let said = match answer {
                    Ok(fused) => {
                        let valid = model.validate(&Policy::certified(), &mut ProvenanceLog::new());
                        let volumes: Vec<f64> = fused
                            .solids()
                            .iter()
                            .map(|&solid| volume_of(&model, solid))
                            .collect();
                        format!(
                            "ok {volumes:?} valid {:?} log {}",
                            valid.is_ok(),
                            log.records().len()
                        )
                    }
                    Err(why) => format!("err {why:?}"),
                };
                out.push(format!("{shift:?} {rule:?} bar_first {bar_first}: {said}"));
            }
        }
    }
    panic!("{}", out.join("\n"));
}

/// A bar of radius 0.4 standing on `z = 0` round the axis through `(0.6, 0)`,
/// two tall, whose floor is two half disks split along `y = 0`.
fn a_bar_with_a_split_floor(model: &mut Model) -> SolidKey {
    use rcad_geom::{EdgeRun, HalfSpace, Orientation, Picker};
    use rcad_topo::{DraftEdge, DraftStep, SolidBuilder};
    let across = 0.6;
    let radius = 1.0 - across;
    let height = 2.0;
    let pipe = Cylinder::new(
        Axis::new(Point::new(across, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0)).expect("an axis"),
        radius,
    )
    .expect("a radius");
    let ground = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("a unit normal");
    let lid =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, height), Vector::new(0.0, 0.0, 1.0))
            .expect("a unit normal");
    let mut draft = SolidBuilder::new();
    let a = draft
        .add_vertex(Point::new(across + radius, 0.0, 0.0))
        .expect("a");
    let b = draft
        .add_vertex(Point::new(across - radius, 0.0, 0.0))
        .expect("b");
    let t = draft
        .add_vertex(Point::new(across - radius, 0.0, height))
        .expect("t");
    let arc = |side: f64| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(ground),
        picked_by: smallvec::smallvec![Picker::from(
            HalfSpace::through_a_point_facing(
                Point::new(across, 0.0, 0.0),
                Vector::new(0.0, side, 0.0)
            )
            .expect("a facing")
        )],
    };
    let over = draft.add_edge(b, a, arc(1.0)).expect("over");
    let under = draft.add_edge(a, b, arc(-1.0)).expect("under");
    let chord = draft.add_edge(a, b, EdgeRun::Straight).expect("chord");
    let seam = draft.add_edge(b, t, EdgeRun::Straight).expect("seam");
    let top = draft
        .add_edge(t, t, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("top");
    let mut wire = |draft: &mut SolidBuilder, steps: &[(DraftEdge, Orientation)]| {
        let steps: Vec<DraftStep> = steps
            .iter()
            .map(|&(edge, way)| DraftStep::Along(draft.add_coedge(edge, way).expect("use")))
            .collect();
        draft.add_wire(&steps).expect("a wire")
    };
    let w_over = wire(
        &mut draft,
        &[(over, Orientation::Forward), (chord, Orientation::Forward)],
    );
    let w_wall = wire(
        &mut draft,
        &[
            (under, Orientation::Reversed),
            (over, Orientation::Reversed),
            (seam, Orientation::Forward),
            (top, Orientation::Reversed),
            (seam, Orientation::Reversed),
        ],
    );
    let w_under = wire(
        &mut draft,
        &[
            (under, Orientation::Forward),
            (chord, Orientation::Reversed),
        ],
    );
    let w_lid = wire(&mut draft, &[(top, Orientation::Forward)]);
    let floor_over = draft
        .add_face(Surface::Plane(ground), &[w_over])
        .expect("f");
    let wall = draft
        .add_face(Surface::Cylinder(pipe), &[w_wall])
        .expect("f");
    let floor_under = draft
        .add_face(Surface::Plane(ground), &[w_under])
        .expect("f");
    let lid_face = draft.add_face(Surface::Plane(lid), &[w_lid]).expect("f");
    draft
        .add_shell(&[
            (floor_over, Orientation::Forward),
            (wall, Orientation::Forward),
            (floor_under, Orientation::Forward),
            (lid_face, Orientation::Forward),
        ])
        .expect("shell");
    draft
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the bar builds: {why:?}"))
}

/// The corner a bar's seam starts from, and the two surfaces round it, as a bar
/// standing where it was built stores them.
fn the_seam_corner_of_a_bar_and_its_surfaces() -> (Point, [Surface; 2]) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(1.0, 2.0, &Isometry::identity())
        .expect("a whole-number bar");
    let corner = Point::new(1.0, 0.0, 0.0);
    assert!(
        model.vertices().any(|vertex| vertex.point() == corner),
        "a bar of radius one stands its seam at (1, 0, 0)"
    );
    let cap = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
            .expect("the z axis is a unit normal"),
    );
    let wall = Surface::Cylinder(Cylinder::new(Axis::z(), 1.0).expect("a radius of one"));
    let held: Vec<Surface> = model
        .solid(bar)
        .expect("the bar was just built")
        .faces()
        .map(|face| face.surface())
        .collect();
    assert!(
        held.contains(&cap) && held.contains(&wall),
        "the bar holds its bottom cap and its wall as these numbers: {held:?}"
    );
    (corner, [cap, wall])
}

fn a_cube(model: &mut Model, side: f64) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    model
        .add_extrusion(&square, side, &Isometry::identity())
        .expect("building a box needs no decision")
}

/// The solids a certified fuse answers with, having checked that it proved every
/// decision and wrote a model that validates.
fn answered(model: &mut Model, one: SolidKey, other: SolidKey, rule: Rule) -> Vec<SolidKey> {
    let mut log = ProvenanceLog::new();
    let answer = model
        .fuse(&[one], &[other], rule, &Policy::certified(), &mut log)
        .unwrap_or_else(|why| panic!("{rule:?} of the bar and the cube refused: {why}"));
    assert!(
        log.records().is_empty(),
        "{rule:?}: every decision is proved, got {:?}",
        log.records()
    );
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{rule:?} wrote a model that does not validate: {why}"));
    answer.solids().to_vec()
}

fn volume_of(model: &Model, solid: SolidKey) -> f64 {
    model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(A_DEFLECTION).expect("a fiftieth of a millimetre is a length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a solid this size measures")
        .volume()
}
