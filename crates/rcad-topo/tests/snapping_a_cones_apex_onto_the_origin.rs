//! The caller gesture the placement census never made: read a cone's apex, then
//! place the solid again by minus it.
//!
//! `a_placed_cones_apex_and_the_point_the_mesh_puts_there.rs` asks whether the
//! mesh point at a placed cone's apex is bit-for-bit the apex the face reports,
//! over 105 placements. Every one of those is **one** placement. This is the
//! shape a consumer actually reaches for — snap this vertex onto the origin —
//! and it **composes a second placement onto the first**, which is what the
//! meshing frame then has to invert. `docs/decisions.md` §56 is why that
//! mattered: `Isometry::inverse` is not a bit-for-bit involution, so a turn
//! carrying a shift used to ride the apex out on its placement inverted twice
//! and land a whisker off.
//!
//! # Two families, and only one of them could ever have been wrong
//!
//! §16.10 wants a demonstration rather than a paragraph for a claim that
//! something cannot be reached, and this gesture supplies one for half of
//! itself.
//!
//! With the cone's **stored** apex at the origin, the located apex is the
//! composed placement's entire translation, so snapping by minus it leaves
//! `translation(−t) ∘ (R, t) = (R, 0)` — a pure turn about an axis through the
//! origin, translation exactly zero. That is §56's bit-exact family, whose shift
//! survives inversion because it is zero and stays zero. **The gesture cannot
//! put a stored-at-the-origin cone anywhere the old defect could bite**, and
//! [`snapping_a_cone_stored_about_the_origin_cannot_reach_the_case_that_was_wrong`]
//! counts that rather than asserting it.
//!
//! With the stored apex off the origin the cancellation does not happen, and
//! that family lands squarely in the case §56 fixed. It is the regression
//! witness: without it the first test could pass on arithmetic that never went
//! near the defect.
//!
//! A stored apex off the origin is reachable through the public API —
//! `SolidBuilder::add_face` is public, re-exported at `rcad/src/lib.rs`, and
//! takes an arbitrary `Surface` — which is what [`a_pointed_cone_stored_at`]
//! uses. How *common* it is, is not a claim this file makes.

mod cones;

use cones::a_pointed_cone;
use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, SolidBuilder, SolidKey};

const SLOPE: f64 = 0.5;
const HOW_FAR_UP_THE_BASE_SITS: f64 = 8.0;

/// Where the second family stores its apex. Nothing about it is special beyond
/// being off the origin in all three coordinates and not a round number.
const A_STORED_APEX_OFF_THE_ORIGIN: [f64; 3] = [11.0, -7.0, 5.0];

/// What one family of placements did, so the two can be compared rather than
/// both merely passing.
struct WhatTheGestureDid {
    rows: usize,
    /// The snapped apex was exactly `[0, 0, 0]`, not merely close to it.
    landed_on_the_origin: usize,
    /// The nearest mesh point to the apex was the apex, bit for bit.
    the_mesh_landed_on_it: usize,
    /// The composed placement is one the old doubled inversion would have moved
    /// the apex on — the defect §56 fixed reaches this row.
    the_old_route_would_have_moved_it: usize,
    worst_the_old_route_would_have_moved_it: f64,
}

/// The property a consumer is betting on, across both families.
///
/// rscene special-cases its apex triangle on **exact** equality with
/// `Cone::apex()`, so a miss of one ulp does not soften an answer, it hands back
/// a garbage radial normal. The count at the end is what stops this passing by
/// snapping nothing: every row must reach the mesh and every row must match.
#[test]
fn the_mesh_still_lands_on_the_apex_after_a_consumer_snaps_it_to_the_origin() {
    for (label, stored_apex) in families() {
        let did = what_the_gesture_did(stored_apex);
        assert_eq!(
            did.the_mesh_landed_on_it, did.rows,
            "{label}: the mesh put its nearest point on the apex in only {} of {} rows, and a \
             consumer that special-cases the apex on exact equality gets a radial normal for \
             the rest",
            did.the_mesh_landed_on_it, did.rows
        );
        assert!(
            did.rows >= 24,
            "{label}: only {} rows ran, which is not a sweep",
            did.rows
        );
    }
}

/// §16.10's demonstration, and the anti-vacuity proof for the test above.
///
/// Two runs down the same path prove nothing however pretty the algebra, so this
/// measures that the families differ in the way the argument says they do:
///
/// - Stored at the origin, the snap cancels the translation exactly, so **no**
///   row is one the doubled inversion would have moved. The case cannot be
///   reached, which is the demonstration §16.10 asks for.
/// - Stored off the origin, most rows **are**, so the first test is exercising
///   the arithmetic §56 fixed rather than passing beside it.
///
/// The two also disagree on where the apex ends up: snapping is exact in the
/// first family and rounds in the second, so only the first lands every row on
/// the origin itself.
#[test]
fn snapping_a_cone_stored_about_the_origin_cannot_reach_the_case_that_was_wrong() {
    let [x, y, z] = A_STORED_APEX_OFF_THE_ORIGIN;
    let at_the_origin = what_the_gesture_did(None);
    let off_the_origin = what_the_gesture_did(Some(Point::new(x, y, z)));

    assert_eq!(
        at_the_origin.the_old_route_would_have_moved_it, 0,
        "a cone stored about the origin is supposed to snap to a placement with no translation \
         at all, but {} of {} rows would have moved on the doubled inversion",
        at_the_origin.the_old_route_would_have_moved_it, at_the_origin.rows
    );
    assert_eq!(
        at_the_origin.landed_on_the_origin, at_the_origin.rows,
        "and every one of those snapped apexes is supposed to be exactly the origin"
    );

    assert!(
        off_the_origin.the_old_route_would_have_moved_it * 2 > off_the_origin.rows,
        "only {} of {} off-origin rows would have moved on the doubled inversion, so this \
         family is no longer a witness that the fixed case is being reached",
        off_the_origin.the_old_route_would_have_moved_it,
        off_the_origin.rows
    );
    assert!(
        off_the_origin.worst_the_old_route_would_have_moved_it > 0.0,
        "the worst the old route would have moved an apex is zero, so nothing distinguishes \
         these two families"
    );
    assert!(
        off_the_origin.landed_on_the_origin < at_the_origin.landed_on_the_origin,
        "both families snapped exactly onto the origin equally often ({} of {}), so the \
         cancellation the argument rests on is not what is happening",
        off_the_origin.landed_on_the_origin,
        off_the_origin.rows
    );

    println!(
        "stored at the origin: {}/{} landed on the origin, {} would have moved.\n\
         stored off it:        {}/{} landed on the origin, {} would have moved, worst {}",
        at_the_origin.landed_on_the_origin,
        at_the_origin.rows,
        at_the_origin.the_old_route_would_have_moved_it,
        off_the_origin.landed_on_the_origin,
        off_the_origin.rows,
        off_the_origin.the_old_route_would_have_moved_it,
        off_the_origin.worst_the_old_route_would_have_moved_it
    );
}

fn families() -> [(&'static str, Option<Point>); 2] {
    let [x, y, z] = A_STORED_APEX_OFF_THE_ORIGIN;
    [
        ("a cone stored about the origin", None),
        (
            "a cone stored away from the origin",
            Some(Point::new(x, y, z)),
        ),
    ]
}

/// Place a cone 24 ways, read where its apex went, place it again by minus that,
/// and see what the mesh makes of the result.
fn what_the_gesture_did(stored_apex: Option<Point>) -> WhatTheGestureDid {
    let turn = Axis::new(Point::new(1.0, 2.0, 3.0), Vector::new(1.0, 1.0, 1.0))
        .expect("an axis off the origin");
    let mut did = WhatTheGestureDid {
        rows: 0,
        landed_on_the_origin: 0,
        the_mesh_landed_on_it: 0,
        the_old_route_would_have_moved_it: 0,
        worst_the_old_route_would_have_moved_it: 0.0,
    };

    for step in 1..=24 {
        let spin = Isometry::rotation(&turn, f64::from(step) * 0.37).expect("a turn");
        let shift = Isometry::translation(Vector::new(
            f64::from(step) * 1.5,
            f64::from(step) * -2.25,
            7.0,
        ))
        .expect("a shift");
        let placement = shift.compose(&spin);

        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        let solid = match stored_apex {
            None => a_pointed_cone(&mut model, SLOPE, HOW_FAR_UP_THE_BASE_SITS),
            Some(at) => a_pointed_cone_stored_at(&mut model, SLOPE, HOW_FAR_UP_THE_BASE_SITS, at),
        };
        model
            .place_solid(solid, &placement)
            .expect("this model made the key");

        let [x, y, z] = the_apex_a_consumer_reads(&model, solid).coordinates();
        let snap = Isometry::translation(Vector::new(-x, -y, -z)).expect("a shift back");
        model
            .place_solid(solid, &snap)
            .expect("this model made the key");

        let apex = the_apex_a_consumer_reads(&model, solid);
        did.rows += 1;
        if apex.coordinates() == [0.0, 0.0, 0.0] {
            did.landed_on_the_origin += 1;
        }

        // `place_solid` composes, so this is the movement the face now carries,
        // and the old route rode the apex out on it inverted twice.
        let composed = snap.compose(&placement);
        let stored = stored_apex.unwrap_or_else(Point::origin);
        let would_have_moved = composed
            .inverse()
            .inverse()
            .apply_to_point(stored)
            .distance_to(composed.apply_to_point(stored));
        if would_have_moved > 0.0 {
            did.the_old_route_would_have_moved_it += 1;
            did.worst_the_old_route_would_have_moved_it = did
                .worst_the_old_route_would_have_moved_it
                .max(would_have_moved);
        }

        let mesh = model
            .solid(solid)
            .expect("this model made the key")
            .mesh(
                Deflection::of(0.01).expect("a positive length"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("a pointed cone meshes at a hundredth");
        let nearest = mesh
            .points()
            .iter()
            .copied()
            .min_by(|a, b| {
                a.distance_squared_to(apex)
                    .total_cmp(&b.distance_squared_to(apex))
            })
            .expect("a mesh has points");
        if nearest.coordinates() == apex.coordinates() {
            did.the_mesh_landed_on_it += 1;
        }
    }

    did
}

fn the_apex_a_consumer_reads(model: &Model, solid: SolidKey) -> Point {
    let mut apexes = model
        .solid(solid)
        .expect("this model made the key")
        .faces()
        .filter_map(|face| match face.surface() {
            Surface::Cone(cone) => Some(cone.apex()),
            Surface::Plane(_) | Surface::Sphere(_) | Surface::Cylinder(_) | Surface::Torus(_) => {
                None
            }
            other => panic!("a pointed cone has no {other:?} face"),
        });
    let apex = apexes.next().expect("a pointed cone has a cone face");
    assert!(apexes.next().is_none(), "and it has only the one");
    apex
}

/// [`a_pointed_cone`] with the whole shape moved so the **stored** apex sits at
/// `apex` instead of the origin, through the public builder.
fn a_pointed_cone_stored_at(model: &mut Model, slope: f64, far: f64, apex: Point) -> SolidKey {
    let [ax, ay, az] = apex.coordinates();
    let axis = Axis::new(apex, Vector::new(0.0, 0.0, 1.0)).expect("a z axis through the apex");
    let skin = Cone::new(axis, slope).expect("a positive slope");
    let mut builder = SolidBuilder::new();

    let rim = builder
        .add_vertex(Point::new(ax + slope * far, ay, az + far))
        .expect("a finite point");
    let circle = builder
        .add_edge(rim, rim, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the cap plane cuts the cone in one closed curve");

    let plane =
        Plane::from_point_and_normal(Point::new(ax, ay, az + far), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let cap_coedge = builder
        .add_coedge(circle, Orientation::Forward)
        .expect("an edge of this build");
    let cap_wire = builder
        .add_wire(&[DraftStep::Along(cap_coedge)])
        .expect("one closed edge is a wire");
    let cap = builder
        .add_face(Surface::Plane(plane), &[cap_wire])
        .expect("a cap is a plane trimmed by its circle");

    let side_coedge = builder
        .add_coedge(circle, Orientation::Reversed)
        .expect("an edge of this build");
    let side_wire = builder
        .add_wire(&[DraftStep::Along(side_coedge)])
        .expect("an edge of this build");
    let side = builder
        .add_face(Surface::Cone(skin), &[side_wire])
        .expect("a face on the cone");

    builder
        .add_shell(&[(cap, Orientation::Forward), (side, Orientation::Forward)])
        .expect("two faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a pointed cone described this way is a solid")
}
