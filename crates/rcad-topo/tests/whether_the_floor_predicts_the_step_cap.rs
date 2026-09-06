//! The floor under a mesh has to be a floor the walk agrees with.
//!
//! `SolidView::room_a_mesh_has_to_work_in` used one number for two jobs. It
//! **lets through** a deflection a caller named, and where a tolerant policy
//! allows it, it **picks one itself** for a caller who asked for finer than the
//! arithmetic has. The number it had counts how far a point *rounds* and says
//! nothing about how many points `rcad_geom::walk` will place along one
//! crossing — which is fine for letting a caller's own number through, and
//! wrong for picking one, because nobody else is going to look at what it picks
//! before the walk is handed it.
//!
//! # What that cost, measured
//!
//! A round bar of radius one at the origin, asked for `1e-300` under
//! `Policy::tolerant(100.0)`. The rounding floor came to
//! `9.2148511043888e-13`, the tolerant caller was handed twice that —
//! `1.84297022087776e-12` — and the walk refused it with
//! `TooManyPointsAlongOneCrossing { most: 1048576 }` on the first cap circle it
//! was asked for. No lean, no placement, no fuse: a four-line solid the builder
//! makes.
//!
//! The same bar swept by multiples of that floor: `1x`, `2x` and `16x` all
//! refused the same way; `32x` was accepted by the walk. So the floor was
//! between sixteen and thirty-two times too low, and the arithmetic below says
//! twenty-six.
//!
//! # Where the room goes, and it is cut twice more
//!
//! Taking `walk`'s `how_many_steps` on its own — the cap circle reaches one and
//! goes right round — staying under the cap needs
//! `room ≥ 1 − cos(π / 1048576)`, which is `4.488176455689416e-12` as the code
//! works it out and `4.488187599349658e-12` if the subtraction is written out
//! instead — the same number to five digits, and neither of them the
//! `4.4875e-12` this file published before it was measured. That is **4.8706
//! times** that floor, not twenty-six, and the rest of the way is that the
//! deflection is cut twice more before `how_many_steps` sees it:
//!
//! 1. `BuildingAMesh::walk_the_edges_this_face_can_place` takes
//!    `how_far_moving_this_face_rounds` off it. **Only half of that is nothing
//!    at all.** The shifting half is, because a bar at the origin is meshed in
//!    the frame it is stored in; the placing half is
//!    `Isometry::POINT_ROUNDOFF · 3` — `1.199040866595169e-14`, **1.30%** of
//!    the floor — and it never looks at the placement, because a mesh point is
//!    carried out of the meshing frame whether or not the solid was put
//!    anywhere. `256·ε / 0.1875 · 3` alone is `9.094947017729282e-13` and the
//!    floor is `9.2148511043888e-13`; the floor this file publishes cannot be
//!    reproduced without the term it used to call nothing.
//! 2. It then hands `walk` only `WHAT_A_BOUNDARY_SIDE_IS_REFINED_TO` of what is
//!    left — three sixteenths — because the triangulator is going to spend the
//!    rest.
//!
//! So the second cut is nearly the whole of it: `4.8706 / 0.1875` is `25.98`,
//! and `walk`'s own `room_left_over` takes its deviation off on top, which puts
//! the crossover a little past twenty-six. That sits inside the measured `16x`-`32x`
//! bracket. **Nothing about a lean is involved** — this is a bar the builder
//! makes in one call, with no cap plane named and no placement on it.
//!
//! # Why there is no test here that meshes at the new floor
//!
//! At the floor the walk places [`MOST_POINTS_ALONG_ONE_CROSSING`] points along
//! a crossing, and at twice it about 741 000 — **by construction**, because
//! that is what the floor is now derived from. A bar has two cap circles, so
//! the mesh a tolerant caller now gets for that request has one and a half
//! million boundary points in it and the triangulator runs for minutes; it was
//! still running after two. That is not a slow test to be sped up, it is what
//! "as fine as the arithmetic reaches" comes to on a solid at the origin, and
//! whether the fallback should be held to a mesh that finishes as well as to
//! one the walk accepts is a separate question this file does not answer.
//!
//! So what is checked here is the refusal, which is the same fact said the
//! other way round: the mesher now settles it up front against its own floor
//! instead of handing the walk a deflection the walk was never going to take.
//!
//! # What was deliberately left alone
//!
//! A caller who names a deflection still gets it tried against the rounding
//! floor and no other, so a unit ball asked for `1e-12` still reaches
//! `TooManyPointsAlongOneCrossing` —
//! `meshing_a_curved_solid::a_deflection_needing_more_points_than_can_be_held_is_refused`
//! is that, and it is a refusal about a number that caller chose. Holding a
//! named deflection to the step cap as well would make that variant
//! unreachable through `mesh`, which is a claim about a public API that wants
//! its own demonstration and is not this file's.

use rcad_geom::{Deflection, Isometry, MOST_POINTS_ALONG_ONE_CROSSING, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, SolidKey};

fn a_bar(radius: f64, height: f64, out: f64) -> (Model, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = model
        .add_round_bar(
            radius,
            height,
            &Isometry::translation(Vector::new(out, 0.0, 0.0)).expect("a finite placement"),
        )
        .expect("a round bar");
    (model, bar)
}

/// The one the defect is: a tolerant caller asking for more than the arithmetic
/// has, on the plainest curved solid the builder makes.
///
/// Before this was fixed the answer was
/// `TooManyPointsAlongOneCrossing { most: 1048576 }`, out of `rcad-geom`, about
/// a deflection this crate had just worked out and handed over itself. Now it
/// is [`Error::DeflectionBelowWhatAMeshCanReach`], out of the guard whose job
/// that is, and the floor it names is one the walk would accept.
#[test]
fn a_tolerant_caller_is_refused_by_the_floor_and_not_by_the_walk() {
    let (model, bar) = a_bar(1.0, 2.0, 0.0);
    let answer = model.solid(bar).mesh(
        Deflection::of(1e-300).expect("a positive length"),
        &Policy::tolerant(100.0),
        &mut ProvenanceLog::new(),
    );

    let Err(Error::DeflectionBelowWhatAMeshCanReach { closest, .. }) = answer else {
        panic!(
            "a tolerant caller a thousandth of a femtometre below the floor \
             should be told about the floor, and got {answer:?}"
        );
    };

    println!("the finest deflection the walk will take on this bar is {closest:e}");

    // The cap circle of this bar has radius one and goes right round, so the
    // walk needs at least this much room in whatever it is handed — and it is
    // handed a fraction of `closest`, never more. A floor under this is a floor
    // the walk refuses.
    let a_whole_turn_at_the_cap =
        core::f64::consts::TAU / MOST_POINTS_ALONG_ONE_CROSSING as f64 / 2.0;
    let sagitta = 1.0 - a_whole_turn_at_the_cap.cos();
    assert!(
        closest > sagitta,
        "the floor came to {closest:e} and the walk needs {sagitta:e} of room \
         on a cap circle of radius one before it will place a point"
    );
}

/// The other half of the same guard: a solid far enough out that its floor is
/// set by the movement into place rather than by the step cap, where the
/// fallback already worked and has to go on working.
///
/// **Measured either side of the change**, because `docs/consumers.md` asks for
/// that with no threshold on it. The floor moved from `3.996804715633573e-6` to
/// `3.9969483372801546e-6` — up, by thirty-six millionths of itself — and the
/// mesh that came back was the same 5134 points with the same
/// `5.99402529715387e-6` of deviation, bit for bit.
///
/// **Both of those second numbers have since moved again, and the bit identity
/// is gone.** Charging a curved face for where its *edges* run rather than for
/// where its *corners* sit raised what this bar reaches, and the reach stands in
/// the movement charge as much as it stands in the floor. The floor is now
/// `3.996948353267366e-6` and the deviation `5.994025313141081e-6` — the point
/// count still 5134.
///
/// **The deviation did not move because the floor did**, and the demonstration
/// is that meshing this bar at exactly twice the *old* floor,
/// `7.993896674560309e-6`, gives `5.994025313141081e-6` as well: the deflection
/// nobody changed gets the new answer. The two moved by the very same
/// `1.5987211554602254e-14`, which is what a solid a billion units out looks
/// like — out here the floor is the movement charge and the deviation carries
/// that same charge, so raising the reach raises both by one number.
///
/// **That is true of this bar and it is not true of every solid.** A ball of
/// radius a quarter at the same billion units out, asked the same thing, came
/// back with 872 666 points and `7.993599754617626e-6` of deviation before the
/// change and 872 658 points and `7.993623476198684e-6` after it — a mesh that
/// worked before and is not the same mesh now. So the claim here is bit
/// identity **on this bar**, not "nothing that meshed before changes".
///
/// The two behave differently, and it is not that one is a bar:
/// `Mesh::deviation` is a max over what the walk and the triangulator each say
/// plus the movement into place, and the movement into place does not depend on
/// the deflection at all. What the triangulator strays does, but only in steps,
/// because it subdivides an integer number of times. This bar's floor moved by
/// thirty-six parts in a million and no count in a 5134-point mesh moved with
/// it; the ball's floor moved by only three parts in a million —
/// `3.996803041232215e-6` to `3.996815009702763e-6` — and counts in an
/// 872 000-point mesh did. **Points, not digits, is what decides the half of
/// the deviation the deflection reaches** — and only that half, which is what
/// the reach change above then walked in through: it moved the other half, so
/// this bar's digits moved with no count moving at all. The ball strays
/// 99.99992% of the whole deflection it was handed — the budget is spent, so
/// where the budget goes the published number goes — while this bar spends
/// 74.98% of its own and pays the rest to a movement no deflection reaches.
#[test]
fn the_fallback_a_bar_a_billion_units_out_already_had_says_the_same_thing() {
    let (model, bar) = a_bar(1.0, 5.0, 1e9);
    let mesh = model
        .solid(bar)
        .mesh(
            Deflection::of(1e-300).expect("a positive length"),
            &Policy::tolerant(1e12),
            &mut ProvenanceLog::new(),
        )
        .expect("a tolerant caller gets as fine a mesh as the arithmetic reaches");

    assert_eq!(mesh.points().len(), 5134);
    assert_eq!(mesh.deviation().magnitude(), 5.994025313141081e-6);
}
