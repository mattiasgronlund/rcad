//! A plate whose top face is on `z = 0`, with a whole ring sunk to its middle
//! in it, fused every way (row Q1513).
//!
//! **The ring** is a whole torus about `z`, ring 4 and tube 1, built as two
//! faces cut along its equators, as `a_plate_cut_by_a_whole_bend.rs` builds it.
//! It holds `2π · 4 · π · 1² = 8π²`, by Pappus.
//!
//! **The plate** is a round bar of radius 6 with `z` in `[−2, 0]`, so its top
//! face holds both equators, and the ring's lower half is inside it. It holds
//! `π · 6² · 2 = 72π`. The ring's lower half is `4π²`, so:
//!
//! - the union is `72π + 4π²`, the plate with the top half of the ring on it;
//! - the intersection is `4π²`, the bottom half of the ring;
//! - the plate less the ring is `72π − 4π²`, the plate with a groove round
//!   it, one solid because the groove is one deep and the plate two;
//! - the ring less the plate is `4π²`, the top half of the ring.
//!
//! **The plate's top face is the question.** `z = 0` cuts the ring in its
//! outer and inner equator, and the face keeps the piece outside the outer one
//! and the disc inside the inner one wherever it keeps either. Both are outside
//! the ring, so the ring's sign alone joins them into one cell. On `30a1b8da`
//! every one of these six fuses was refused `CannotTellTheCellsOfAFaceApart`
//! on the plate's face; `whether_past_the_pipe_between_two_circles_of_a_ring`
//! in `fuse/arrange.rs` tells them apart now. So each answer is checked by its
//! faces as well as its volume: a top face joined across the ring would be one
//! flat face with three wires where the right answer has two flat faces.
//!
//! **Why a round plate and not a block.** A block's side faces are planes
//! parallel to the ring's axis, and `rcad_geom`'s
//! `where_a_plane_and_a_torus_cross` refuses every such plane off the axis,
//! even 20 from it against a reach of 5:
//! `ACrossingBetweenAPlaneAndATorusThatIsAQuartic`, row Q618. A block
//! `[−6, 6]² × [−2, 0]`, and one `[−20, 20]² × [−1/2, 1/2]`, were refused
//! that way in all six fuses, on `30a1b8da` and after Q1513 alike, and never
//! reach the top face. The plate's wall is a pipe on the ring's own axis, and
//! a pipe of radius 6 misses a ring that reaches out to 5.
//!
//! The last test holds the ring's middle in a plate one thick, from
//! `z = −1/2` to `z = 1/2`, which every fuse refused until Q1514 (§428.4).

use core::f64::consts::PI;

use rcad_geom::{
    Axis, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Torus, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, Rule, SolidBuilder, SolidKey};

const RING: f64 = 4.0;
const TUBE: f64 = 1.0;
const PLATE: f64 = 6.0;
const PLATE_DEPTH: f64 = 2.0;

/// The plate with the top half of the ring on it, and the plate grooved by
/// the bottom half: the wall between its two rims, the bottom, the disc
/// inside the inner equator, the piece of the top outside the outer one, and
/// one half of the ring.
const PLATE_AND_HALF_A_RING: [(&str, usize); 5] = [
    ("pipe", 2),
    ("plane", 1),
    ("plane", 1),
    ("plane", 2),
    ("ring", 2),
];

/// Half of the ring, closed by the flat ring of `z = 0` between its equators.
const HALF_A_RING: [(&str, usize); 2] = [("plane", 2), ("ring", 2)];

#[test]
fn every_union_is_the_plate_with_the_top_half_of_the_ring_on_it() {
    for plate_first in [true, false] {
        a_fuse_is(
            Rule::Union,
            plate_first,
            72.0 * PI + 4.0 * PI * PI,
            &PLATE_AND_HALF_A_RING,
        );
    }
}

#[test]
fn every_intersection_is_the_bottom_half_of_the_ring() {
    for plate_first in [true, false] {
        a_fuse_is(Rule::Intersection, plate_first, 4.0 * PI * PI, &HALF_A_RING);
    }
}

#[test]
fn the_plate_less_the_ring_is_grooved_and_the_ring_less_the_plate_is_its_top_half() {
    a_fuse_is(
        Rule::Difference,
        true,
        72.0 * PI - 4.0 * PI * PI,
        &PLATE_AND_HALF_A_RING,
    );
    a_fuse_is(Rule::Difference, false, 4.0 * PI * PI, &HALF_A_RING);
}

/// A plate one thick, `z` in `[−1/2, 1/2]`, holds the middle of the ring
/// between its two faces.
///
/// Each of the ring's two faces is then cut by two planes square to its axis:
/// its own equators' `z = 0` and one of the plate's faces. Between the two, the
/// band outside the ring's radius and the band inside it have the same sign
/// against both planes, and only their order round the tube tells them apart.
/// Every fuse was refused `CannotTellTheCellsOfAFaceApart` until Q1514 put the
/// bands in order round the tube (§428.4).
///
/// The slab of the ring with `|z| ≤ 1/2` holds `4√3π + 8π²/3`, which is the
/// intersection, and the union is the plate and the ring less that. The ring
/// cuts the plate's `36π` in two: the disc inside the ring's inner wall,
/// `π(17 − 8(√3/4 + π/6) − 1/12)`, and the rest outside its outer wall. The
/// ring less the plate is two caps of `8π²/3 − 2√3π`.
#[test]
fn a_plate_holding_the_middle_of_the_ring_answers_every_fuse() {
    let the_plate = 36.0 * PI;
    let the_ring = 8.0 * PI * PI;
    let slab = 4.0 * 3f64.sqrt() * PI + 8.0 * PI * PI / 3.0;
    let inside = PI * (17.0 - 8.0 * (3f64.sqrt() / 4.0 + PI / 6.0) - 1.0 / 12.0);
    let cap = 8.0 * PI * PI / 3.0 - 2.0 * 3f64.sqrt() * PI;
    for (rule, plate_first, truths) in [
        (Rule::Union, true, vec![the_plate + the_ring - slab]),
        (Rule::Union, false, vec![the_plate + the_ring - slab]),
        (Rule::Intersection, true, vec![slab]),
        (Rule::Intersection, false, vec![slab]),
        (
            Rule::Difference,
            true,
            vec![inside, the_plate - slab - inside],
        ),
        (Rule::Difference, false, vec![cap, cap]),
    ] {
        let what = format!("{rule:?}, plate first: {plate_first}");
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let plate = model
            .add_round_bar(
                PLATE,
                1.0,
                &Isometry::translation(Vector::new(0.0, 0.0, -0.5)).expect("a finite shift"),
            )
            .expect("a round plate");
        let ring = a_whole_ring_cut_along_its_equators(&mut model);
        let (first, second) = if plate_first {
            ([plate], [ring])
        } else {
            ([ring], [plate])
        };
        let fused = model
            .fuse(
                &first,
                &second,
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("{what}: refused {why:?}"));
        model
            .validate(&Policy::certified(), &mut ProvenanceLog::new())
            .unwrap_or_else(|why| panic!("{what}: does not validate: {why}"));
        assert_eq!(fused.solids().len(), truths.len(), "{what}: {fused:?}");
        let mut left = truths;
        for &solid in fused.solids() {
            let measured = model
                .solid(solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.01).expect("a positive length"),
                    &Policy::certified(),
                    &mut ProvenanceLog::new(),
                )
                .unwrap_or_else(|why| panic!("{what}: does not measure: {why}"));
            let at = left
                .iter()
                .position(|&truth| (measured.volume() - truth).abs() <= measured.volume_error())
                .unwrap_or_else(|| {
                    panic!(
                        "{what}: {} give or take {} is none of {left:?}",
                        measured.volume(),
                        measured.volume_error()
                    )
                });
            left.remove(at);
        }
    }
}

/// Fuses the plate and the ring by `rule`, the plate first or second, and
/// checks the answer is one valid solid of `volume` with these faces, each
/// written as its surface's kind and how many wires bound it.
fn a_fuse_is(rule: Rule, plate_first: bool, volume: f64, faces: &[(&str, usize)]) {
    let what = format!("{rule:?}, plate first: {plate_first}");
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let plate = model
        .add_round_bar(
            PLATE,
            PLATE_DEPTH,
            &Isometry::translation(Vector::new(0.0, 0.0, -PLATE_DEPTH)).expect("a finite shift"),
        )
        .expect("a round plate");
    let ring = a_whole_ring_cut_along_its_equators(&mut model);
    let (first, second) = if plate_first {
        ([plate], [ring])
    } else {
        ([ring], [plate])
    };
    let fused = model
        .fuse(
            &first,
            &second,
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: refused {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{what}: does not validate: {why}"));
    let &[solid] = fused.solids() else {
        panic!("{what}: one solid, not {:?}", fused.solids());
    };
    let solid = model.solid(solid).expect("this model made the key");
    let mut table: Vec<(&str, usize)> = solid
        .faces()
        .map(|face| (kind(&face.surface()), face.wire_count()))
        .collect();
    table.sort_unstable();
    assert_eq!(table, faces, "{what}");
    let measured = solid
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{what}: does not measure: {why}"));
    assert!(
        (measured.volume() - volume).abs() <= measured.volume_error(),
        "{what} holds {volume} and this one says {} give or take {}",
        measured.volume(),
        measured.volume_error()
    );
}

fn kind(surface: &Surface) -> &'static str {
    match surface {
        Surface::Plane(_) => "plane",
        Surface::Cylinder(_) => "pipe",
        Surface::Torus(_) => "ring",
        Surface::Sphere(_) | Surface::Cone(_) | _ => {
            panic!("nothing here is made of {surface:?}")
        }
    }
}

/// A whole torus about `z`, ring 4 and tube 1, as two faces: the half below
/// `z = 0` and the half above, meeting along the outer and the inner equator.
fn a_whole_ring_cut_along_its_equators(model: &mut Model) -> SolidKey {
    let cut = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("the z axis is a unit normal");
    let corners = [
        Point::new(-(RING + TUBE), 0.0, 0.0),
        Point::new(-(RING - TUBE), 0.0, 0.0),
    ];
    let skin = Surface::Torus(Torus::new(Axis::z(), RING, TUBE).expect("a tube narrower"));
    let mut builder = SolidBuilder::new();
    let [one, other] = corners.map(|corner| {
        let corner = builder
            .add_vertex(corner)
            .expect("a corner on the ring and on the plane");
        builder
            .add_edge(
                corner,
                corner,
                EdgeRun::RightRoundWhereItCrosses {
                    and: Surface::Plane(cut),
                },
            )
            .expect("a rim is a whole closed curve, so one corner twice is an edge")
    });
    let face = |builder: &mut SolidBuilder, walks: [(_, Orientation); 2]| {
        let wires = walks.map(|(edge, turn)| {
            let coedge = builder
                .add_coedge(edge, turn)
                .expect("an edge of this build");
            builder
                .add_wire(&[DraftStep::Along(coedge)])
                .expect("one closed edge is a wire")
        });
        builder
            .add_face(skin, &wires)
            .expect("a band between two circles")
    };
    let first = face(
        &mut builder,
        [(one, Orientation::Reversed), (other, Orientation::Forward)],
    );
    let second = face(
        &mut builder,
        [(one, Orientation::Forward), (other, Orientation::Reversed)],
    );
    builder
        .add_shell(&[
            (first, Orientation::Forward),
            (second, Orientation::Forward),
        ])
        .expect("two halves close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a whole ring in two faces is a solid")
}
