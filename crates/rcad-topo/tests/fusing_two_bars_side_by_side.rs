//! Two overlapping parallel bars, united — and what still stops it.
//!
//! Ordinary mechanical CAD: two round bars that run the same way and share some
//! material. §48 pinned one reason the fuse could not do it — `arrange.rs`'s
//! straight gate swept the whole cylinder-cylinder row into `false`, so a
//! crossing that `rcad-geom` builds exactly as two endless lines fell past the
//! straight gate, past the conic gate, and out as
//! `ACrossingThisCannotBeOrderedAlong` with nothing having run out of
//! arithmetic. That gap is closed, and
//! `two_pipes_side_by_side_run_up_their_common_axis` in `arrange.rs` measures
//! the crossing being ordered up `z` where it used to be refused.
//!
//! **Closing it was necessary and is not sufficient, and this file is what says
//! so.** A corner of the answer is where a bar's flat cap meets *both* pipes,
//! and `meet_of_three_surfaces` wants two of its three flat. It has one. So the
//! fuse now refuses one step earlier than the gate it used to die at, by a
//! different name — [`rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat`]
//! — which `three_surfaces.rs`'s own module doc calls a gap rather than a limit
//! for the same §16.10 reason: one plane and two quadrics is a quartic, it has
//! roots, and reaching them takes a solver rather than a discriminant.
//!
//! **No fuse of two pipes reaches the flipped gate today, and the reason is one
//! line rather than a survey of the callers.** `Arrangement::of` runs
//! `place_the_corners`, then `place_the_meets`, and only then
//! `work_out_how_each_crossing_runs`: corners come before order. Every corner on
//! a pipe-against-pipe crossing is a meet of three surfaces two of which are
//! those pipes, whatever the third is and whichever face is being cut — so **two
//! pipes and anything is at most one plane**, which is one short of what
//! `meet_of_three_surfaces` takes. `place_the_meets` skips a pair of crossings
//! that share a partner, so the two runs of one pipe pair do not ask about each
//! other; what does ask is either run against the face's own boundary, and a
//! bounded face has a boundary.
//!
//! So the gate flip is on the path and is not the last thing on it. Closing the
//! rest is `rcad-geom`'s three-surface meet learning one plane and two quadrics,
//! which is another crate and another unit.

use rcad_geom::{
    Axis, Cylinder, Error as GeomError, Isometry, Plane, Point, Scale, Surface, Vector,
    meet_of_three_surfaces,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Error, Model, Rule, SolidKey};

const RADIUS: f64 = 5.0;
const HEIGHT: f64 = 10.0;
/// How far apart the two axes are: less than `2·RADIUS`, so the bars overlap,
/// and more than `0`, so they are two bars and not one.
const APART: f64 = 6.0;

/// `radius = 5` and `apart = 6` is a 3-4-5 either side of the plane of the two
/// axes, so the two lines the pipes cut run through `(3, ±4, z)` and land on
/// coordinates an `f64` holds exactly. Nothing that refuses below is refusing
/// because a number would not fit.
fn two_bars_side_by_side() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let here = model
        .add_round_bar(RADIUS, HEIGHT, &Isometry::identity())
        .expect("a bar of radius five, ten tall");
    let there = model
        .add_round_bar(
            RADIUS,
            HEIGHT,
            &Isometry::translation(Vector::new(APART, 0.0, 0.0)).expect("a shift along `x`"),
        )
        .expect("a second bar of the same size, six along `x`");
    (model, here, there)
}

/// **Where the union of two overlapping parallel bars stops today, by name.**
///
/// Not `.is_err()`: the whole worth of this test is *which* refusal it is.
/// `ACrossingThisCannotBeOrderedAlong` was the answer before §48's gate flip and
/// would be a regression now.
///
/// All three rules and both orders of the pair, because a refusal that depended
/// on which solid was the tool would be a different bug.
#[test]
fn two_overlapping_parallel_bars_refuse_where_a_cap_meets_both_pipes() {
    let policy = Policy::certified();
    let (mut model, here, there) = two_bars_side_by_side();

    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        for (arguments, tools) in [([here], [there]), ([there], [here])] {
            let mut log = ProvenanceLog::new();
            let refused = model.fuse(&arguments, &tools, rule, &policy, &mut log);
            assert!(
                matches!(
                    refused,
                    Err(Error::Geometry(
                        GeomError::WhereThreeSurfacesMeetNeedsTwoOfThemFlat
                    ))
                ),
                "{rule:?} on two side-by-side bars stops at the corner where a cap meets both \
                 pipes, and it answered {refused:?}"
            );
        }
    }

    model
        .validate(&policy, &mut ProvenanceLog::new())
        .expect("a refused fuse leaves the model exactly as it was");
}

/// **The corner the fuse cannot place, asked for on its own.**
///
/// A bar's cap plane against the two pipes: one flat surface and two quadrics.
/// `meet_of_three_surfaces` refuses fewer than two flat, and this is the call
/// `Arrangement::place_the_meets` makes and hands straight back — the only place
/// in this crate that lets that error out, since `bake.rs`'s corner search
/// swallows it and tries the next triple.
///
/// The two points it would have named are `(3, ±4, 0)`, which are exact. So the
/// refusal is about the shape of the system and not about the arithmetic, and
/// that is what makes it a gap rather than a limit (§16.10).
#[test]
fn a_cap_plane_against_two_side_by_side_pipes_has_only_one_flat_surface() {
    let policy = Policy::certified();
    let mut log = ProvenanceLog::new();
    let cap = Surface::Plane(
        Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
            .expect("a bar's bottom cap"),
    );
    let here = Surface::Cylinder(Cylinder::new(Axis::z(), RADIUS).expect("the first pipe"));
    let there = Surface::Cylinder(
        Cylinder::new(
            Axis::new(Point::new(APART, 0.0, 0.0), Vector::new(0.0, 0.0, 1.0))
                .expect("an axis six along `x`"),
            RADIUS,
        )
        .expect("the second pipe"),
    );

    assert_eq!(
        meet_of_three_surfaces([&cap, &here, &there], &policy, &mut log),
        Err(GeomError::WhereThreeSurfacesMeetNeedsTwoOfThemFlat),
        "one plane and two pipes is a quartic, and this wants two planes"
    );
}
