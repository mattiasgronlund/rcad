//! A lens of a cone between two leaning lids, asked what each of its faces holds
//! (`docs/decisions.md` §219.8).
//!
//! The lens is `tests/the_way_into_an_ellipse_arc_in_a_fuse.rs`'s: the part of
//! the inside of a cone of slope a half — apex at the origin, about `+z` —
//! between the lids `x + z = 4` and `z − x = 4`. Each lid's normal is 45° off
//! the axis, inside the `90° − 26.6°` past which the cone's rulings start to run
//! along the lid, so each cuts the cone in an **ellipse**, not a circle. The
//! apex is behind the lower lid and in front of the upper one. The builder is
//! copied here rather than shared, so this file's write set stays its own.
//!
//! Every answer below is worked out from the shape and not from the kernel:
//!
//! - On the lower lid `z = 4 − x`, the lens is where `x² + y² ≤ z²/4` (inside
//!   the cone) and `z − x ≤ 4`, which there is `x ≥ 0`.
//! - On the upper lid `z = 4 + x`, it is where `x² + y² ≤ z²/4` and
//!   `x + z ≥ 4`, which there is `x ≥ 0` again.
//! - On the cone `x² + y² = z²/4, z > 0`, it is where `x + z ≥ 4` and
//!   `z − x ≤ 4`.
//!
//! The corners of the lens are `(0, ±2, 4)`, and every point asked sits well
//! clear of every side, so none of them is a rounding away from its answer.

use rcad_geom::{
    Axis, Cone, Deflection, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale,
    Surface, Vector,
};
use rcad_predicates::{Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{
    DraftStep, FaceKey, Model, OnAFace, ProfileBuilder, Rule, SolidBuilder, SolidKey, SolidView,
};

const SLOPE: f64 = 0.5;
const LIDS_MEET_THE_AXIS_AT: f64 = 4.0;
const THE_LOWER_LIDS_PICKER_STANDS_BACK: f64 = 0.5;

/// The lower lid is a flat face whose curved side is half an ellipse, and it
/// holds what the ellipse and the chord close round and nothing else.
#[test]
fn the_lower_lid_holds_the_inside_of_its_half_ellipse() {
    let asked = [
        ([1.0, 0.0, 3.0], true),
        ([0.5, 1.0, 3.5], true),
        ([1.0, 1.8, 3.0], false),
        ([-1.0, 0.0, 5.0], false),
        ([3.0, 0.0, 1.0], false),
    ];
    for (at, held) in asked {
        let [x, y, z] = at;
        assert_eq!(x + z, 4.0, "{at:?} is on the lower lid");
        assert_eq!(x * x + y * y <= z * z / 4.0 && x >= 0.0, held, "{at:?}");
    }
    ask(the_lower_lid(), &asked);
}

/// The upper lid, the bigger half of its own ellipse.
#[test]
fn the_upper_lid_holds_the_inside_of_its_half_ellipse() {
    let asked = [
        ([1.0, 0.0, 5.0], true),
        ([2.0, 2.0, 6.0], true),
        ([1.0, 2.4, 5.0], false),
        ([-1.0, 0.0, 3.0], false),
    ];
    for (at, held) in asked {
        let [x, y, z] = at;
        assert_eq!(z - x, 4.0, "{at:?} is on the upper lid");
        assert_eq!(x * x + y * y <= z * z / 4.0 && x >= 0.0, held, "{at:?}");
    }
    ask(the_upper_lid(), &asked);
}

/// The piece of cone between the lids, whose apex is behind the lower lid.
///
/// **This is the one that fails on `4ae37b4f`**: the cone arm of
/// `is_a_family_this_follows` took only cuts the apex is on or in front of, and
/// refused this face `AFaceTrimmedByACurveThisCannotFollow` whatever was asked.
/// The two lid tests above pass there too — a flat face cut by a cone in an
/// ellipse was already followed, since §211 — and are kept as the pin on it.
///
/// Along the ruling at angle `φ`, `t·(cos φ / 2, sin φ / 2, 1)`, the lower lid
/// keeps `t ≥ 4 / (1 + cos φ / 2)` and the upper one `t ≤ 4 / (1 − cos φ / 2)`,
/// so the rulings that carry a point are the one arc `cos φ ≥ 0` — one piece.
/// Every point asked is on the cone to the bit: `x² + y² = z²/4` in exact
/// arithmetic, the `3-4-5` triangle doing the work.
#[test]
fn the_piece_of_cone_between_the_lids_holds_what_is_between_them() {
    let asked = [
        ([2.0, 0.0, 4.0], true),
        ([1.2, 1.6, 4.0], true),
        ([1.2, -1.6, 4.0], true),
        ([-1.2, 1.6, 4.0], false),
        ([0.6, 0.8, 2.0], false),
        ([3.0, 4.0, 10.0], false),
        ([-0.6, -0.8, -2.0], false),
    ];
    for (at, held) in asked {
        let [x, y, z] = at;
        assert_eq!(x * x + y * y, z * z / 4.0, "{at:?} is on the cone");
        assert_eq!(z > 0.0 && x + z >= 4.0 && z - x <= 4.0, held, "{at:?}");
    }
    ask(Surface::Cone(the_cone()), &asked);
}

/// A bar driven through both lids, intersected with the lens, is the part of the
/// bar between the lids — and on `4ae37b4f` the fuse could not say so, because
/// every candidate point it tried stopped at the cone face above.
///
/// The bar is `x ∈ [1/2, 1]`, `y ∈ [−1/4, 1/4]`, `z ∈ [11/4, 11/2]`, the one
/// `tests/the_way_into_an_ellipse_arc_in_a_fuse.rs` drives through the lens: it
/// never reaches the cone's skin, and both lids cross it well inside its ends.
/// So what is left is the bar between `z = 4 − x` and `z = 4 + x`, which is
/// `2x` tall at each `x`, and its volume is `½ · ∫ 2x dx` over `[½, 1]`, which
/// is `½ · (1 − ¼) = 3/8`. The answer is flat, so it is measured exactly.
#[test]
fn a_bar_through_both_lids_intersected_with_the_lens_is_three_eighths() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let lens = a_lens_of_a_cone(&mut model);
    let square = ProfileBuilder::new(vec![[0.5, -0.25], [1.0, -0.25], [1.0, 0.25], [0.5, 0.25]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle on the grid");
    let bar = model
        .add_extrusion(
            &square,
            2.75,
            &Isometry::translation(Vector::new(0.0, 0.0, 2.75)).expect("a finite shift"),
        )
        .expect("a rectangle extruded straight up is a solid");

    let fused = model
        .fuse(
            &[lens],
            &[bar],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("the part of the bar between the lids");
    let [piece] = fused.solids() else {
        panic!("one piece: {:?}", fused.solids())
    };
    let measured = model
        .solid(*piece)
        .expect("a solid the fuse made")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a flat solid measures");
    assert_eq!(measured.volume(), 0.375);
}

/// Ask the face of the lens on `surface` about each point, and hold the answer
/// to the one worked out beside it.
fn ask(surface: impl Into<Surface>, asked: &[([f64; 3], bool)]) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let lens = a_lens_of_a_cone(&mut model);
    let view = model.solid(lens).expect("this model made the key");
    let face = the_face_on(view, &surface.into());
    for &(at, held) in asked {
        let said = view.face_holds_the_point(
            face,
            PredicatePoint::from(Point::new(at[0], at[1], at[2]).coordinates()),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        let want = if held {
            OnAFace::Held
        } else {
            OnAFace::NotHeld
        };
        assert!(
            matches!(said, Ok(answer) if answer == want),
            "{at:?} should be {want:?} and came back {said:?}"
        );
    }
}

/// Q1188: the skin's upper arc truly reaches `z = 8` in the middle, not `z = 4`
/// at its two ends, so the floor the mesher picks for it has to be charged
/// from the true reach and not the ends.
///
/// Before `how_far_along_its_axis_this_face_runs` stopped pushing a crossing
/// edge's `and` into `others` unconditionally, the chord plane `x = 0` — which
/// runs through the cone's apex and so cuts it in two rulings rather than a
/// closed curve — charged the upper arc its two ends at `z = 4`, folded in
/// beside the correct bound of `8.0` the upper lid plane gives, and
/// `f64::min` took the wrong, smaller one. That halved the floor
/// [`SolidView::mesh`] computes for a deflection this crate picks itself, from
/// `2.9092923360587807e-10` to `1.4546461680293906e-10` — exactly half, since
/// the floor is proportional to how far along the axis the face is charged.
///
/// **That changes an answer.** At `Policy::tolerant(15000.0)`, whose
/// `max_wobble` of about `2.13e-10` sits between the two, the wrong floor was
/// below it — so the mesher accepted a deflection finer than the face's own
/// true `z = 8` reach can be placed to, a mesh that misses the bound it
/// claims — and the right floor is above it, so the fix refuses honestly
/// instead.
#[test]
fn the_skin_faces_floor_is_charged_from_its_true_reach_and_not_its_arcs_ends() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let lens = a_lens_of_a_cone(&mut model);
    let view = model.solid(lens).expect("this model made the key");

    let refused = view
        .mesh(
            Deflection::of(1e-300).expect("a positive length"),
            &Policy::tolerant(15000.0),
            &mut ProvenanceLog::new(),
        )
        .expect_err("a deflection far below any floor is refused, not silently loosened");
    assert!(
        matches!(
            refused,
            rcad_topo::Error::DeflectionBelowWhatAMeshCanReach { closest, .. }
            if closest == 2.909_292_336_058_780_7e-10
        ),
        "got {refused:?}"
    );
}

fn the_face_on(view: SolidView<'_>, surface: &Surface) -> FaceKey {
    let mut on_it = view.faces().filter(|face| face.surface() == *surface);
    let face = on_it.next().expect("the lens has a face on this surface");
    assert!(on_it.next().is_none(), "one face per surface");
    face.key()
}

fn the_cone() -> Cone {
    Cone::new(Axis::z(), SLOPE).expect("a positive slope")
}

/// `x + z = 4`, facing out of the lens, which is on its `≥` side.
fn the_lower_lid() -> Plane {
    the_lid(-1.0, -1.0)
}

/// `z − x = 4`, facing out of the lens, which is on its `≤` side.
fn the_upper_lid() -> Plane {
    the_lid(-1.0, 1.0)
}

fn the_lid(across: f64, up: f64) -> Plane {
    Plane::from_point_and_normal(
        Point::new(0.0, 0.0, LIDS_MEET_THE_AXIS_AT),
        Vector::new(across, 0.0, up),
    )
    .expect("a vector with a length")
}

fn the_chord_plane() -> Plane {
    Plane::from_point_and_normal(Point::origin(), Vector::new(-1.0, 0.0, 0.0))
        .expect("the x axis is a unit normal")
}

/// The lens, face by face through the public builder: two half ellipses on the
/// lids, the chord `x = 0, z = 4` they share, and the piece of cone between the
/// two arcs.
fn a_lens_of_a_cone(model: &mut Model) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let reach = SLOPE * LIDS_MEET_THE_AXIS_AT;
    let corner = |builder: &mut SolidBuilder, y: f64| {
        builder
            .add_vertex(Point::new(0.0, y, LIDS_MEET_THE_AXIS_AT))
            .expect("a finite point")
    };
    let positive = corner(&mut builder, reach);
    let negative = corner(&mut builder, -reach);

    let arc = |and: Plane, picker: HalfSpace| EdgeRun::PartOfWhereItCrosses {
        and: Surface::Plane(and),
        picked_by: smallvec::smallvec![Picker::from(picker)],
    };
    let facing_the_arcs = |from_x: f64| {
        HalfSpace::through_a_point_facing(Point::new(from_x, 0.0, 0.0), Vector::new(1.0, 0.0, 0.0))
            .expect("the x axis has length")
    };
    let lower_arc = builder
        .add_edge(
            positive,
            negative,
            arc(
                the_chord_plane(),
                facing_the_arcs(-THE_LOWER_LIDS_PICKER_STANDS_BACK),
            ),
        )
        .expect("the smaller half of the lower lid's ellipse");
    let upper_arc = builder
        .add_edge(
            negative,
            positive,
            arc(the_chord_plane(), facing_the_arcs(0.0)),
        )
        .expect("the bigger half of the upper lid's ellipse");
    let chord = builder
        .add_edge(negative, positive, EdgeRun::Straight)
        .expect("the line both lids hold");

    let wire_of = |builder: &mut SolidBuilder, steps: &[(_, Orientation)]| {
        let along: Vec<DraftStep> = steps
            .iter()
            .map(|(edge, turn)| {
                DraftStep::Along(
                    builder
                        .add_coedge(*edge, *turn)
                        .expect("an edge of this build"),
                )
            })
            .collect();
        builder.add_wire(&along).expect("a closed wire")
    };
    let lower_wire = wire_of(
        &mut builder,
        &[
            (lower_arc, Orientation::Forward),
            (chord, Orientation::Forward),
        ],
    );
    let upper_wire = wire_of(
        &mut builder,
        &[
            (upper_arc, Orientation::Forward),
            (chord, Orientation::Reversed),
        ],
    );
    let skin_wire = wire_of(
        &mut builder,
        &[
            (lower_arc, Orientation::Reversed),
            (upper_arc, Orientation::Reversed),
        ],
    );

    let lower = builder
        .add_face(Surface::Plane(the_lower_lid()), &[lower_wire])
        .expect("half of the lower lid's ellipse");
    let upper = builder
        .add_face(Surface::Plane(the_upper_lid()), &[upper_wire])
        .expect("half of the upper lid's ellipse");
    let skin = builder
        .add_face(Surface::Cone(the_cone()), &[skin_wire])
        .expect("the piece of the cone between the two lids");

    builder
        .add_shell(&[
            (lower, Orientation::Forward),
            (upper, Orientation::Forward),
            (skin, Orientation::Forward),
        ])
        .expect("three faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a lens described this way is a solid")
}
