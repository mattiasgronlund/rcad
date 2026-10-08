//! `SolidView::box_around_it` has to hold the whole solid, because a point
//! past it, opened out to where its planes meet, is answered `Outside` with
//! no ray fired, and two such opened-out boxes that miss say two solids never
//! touch.
//!
//! For a face on a pipe or a cone the box used to be settled by the face's
//! corners: how far along the axis the face reaches was read off the corners'
//! shadows on the axis. **That holds only while every side reaches no further
//! along the axis than its own ends do**, and a side that is one whole closed
//! curve has one corner. A pipe whose end is cut by a tilted plane has an
//! ellipse there, and the one corner the builder puts on it can sit at the
//! ellipse's lowest point — so the box stopped there, and everything above it
//! answered `Outside`.
//!
//! This file builds exactly that through the public [`SolidBuilder`], which is
//! the door every importer comes through, so the claim fails for an edge the
//! public API builds and is not a question of which callers exist today
//! (`docs/decisions.md` §16.10).

use rcad_geom::{Axis, Cylinder, EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Containment, DraftStep, Model, SolidBuilder, SolidKey};

mod cones;

/// A cone face's box takes in the apex only when the face can reach it.
///
/// A pointed cone of slope a half, height 8, has its one corner on its cap, so
/// its skin's box has to reach down to the apex at the origin or everything
/// below the cap is answered `Outside` (§200.3). A frustum of the same cone
/// from `z = 2` to `z = 8` stops at its small cap: a box that went on down to
/// the apex handed the fuse pairs of faces its side never meets, and a block
/// below a frustum lost 24 of its 56 right answers to that (§202.4).
#[test]
fn a_frustums_box_stops_at_its_small_cap_and_a_pointed_cones_holds_its_apex() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let pointed = cones::a_pointed_cone(&mut model, 0.5, 8.0);
    let frustum = cones::a_frustum(&mut model, 0.5, 2.0, 8.0);

    let lowest_of_its_skin = |solid: SolidKey| {
        let view = model.solid(solid).expect("this model made the key");
        let skin = view
            .faces()
            .find(|face| matches!(face.surface(), Surface::Cone(_)))
            .expect("a cone has a face on its cone");
        view.box_around_one_face(skin)
            .expect("a face is somewhere")
            .0
            .z()
    };
    assert!(
        lowest_of_its_skin(pointed) <= 0.0,
        "the pointed cone's skin starts above its apex"
    );
    let frustum_starts = lowest_of_its_skin(frustum);
    assert!(
        (2.0 - 1e-9..=2.0).contains(&frustum_starts),
        "the frustum's skin is boxed from z = {frustum_starts}, where it stops at z = 2"
    );
}

/// A round bar of radius one about `+z`, flat at `z = −2` and cut on top by the
/// plane `x + z = 1`, whose one corner up there is `(1, 0, 0)`: the lowest
/// point of the ellipse. The ellipse rises to `z = 2` on the far side.
#[test]
fn a_bar_cut_on_a_slant_is_inside_its_box_above_its_top_corner() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let bar = a_bar_cut_on_a_slant(&mut model);
    let view = model.solid(bar).expect("this model made the key");

    let (low, high) = view.box_around_it().expect("the bar has faces");
    assert!(low.z() <= -2.0, "the box starts at {low:?}");
    assert!(high.z() >= 2.0, "the box stops at {high:?}");

    for (at, want) in [
        (Point::new(-0.9, 0.0, 1.5), Containment::Inside),
        (Point::new(-0.5, 0.3, 1.2), Containment::Inside),
        (Point::new(0.0, 0.0, 0.5), Containment::Inside),
        (Point::new(0.5, 0.0, 0.7), Containment::Outside),
        (Point::new(-0.9, 0.0, 2.1), Containment::Outside),
    ] {
        assert_eq!(
            view.classify_point(at, &policy, &mut ProvenanceLog::new()),
            want,
            "at {at:?}"
        );
    }
}

fn a_bar_cut_on_a_slant(model: &mut Model) -> SolidKey {
    let pipe = Cylinder::new(Axis::z(), 1.0).expect("a positive radius");
    let mut builder = SolidBuilder::new();

    let bottom_corner = builder
        .add_vertex(Point::new(1.0, 0.0, -2.0))
        .expect("a finite point");
    let top_corner = builder
        .add_vertex(Point::new(1.0, 0.0, 0.0))
        .expect("a finite point");
    let seam = builder
        .add_edge(bottom_corner, top_corner, EdgeRun::Straight)
        .expect("a ruling of a pipe is straight");
    let bottom_circle = builder
        .add_edge(
            bottom_corner,
            bottom_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("the bottom plane cuts the pipe in one circle");
    let top_ellipse = builder
        .add_edge(
            top_corner,
            top_corner,
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("the slanted plane cuts the pipe in one ellipse");

    let cap = |builder: &mut SolidBuilder, plane: Plane, edge, turn| {
        let coedge = builder
            .add_coedge(edge, turn)
            .expect("an edge of this build");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("one closed edge is a wire");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a cap is a plane trimmed by its curve")
    };
    let bottom = cap(
        &mut builder,
        Plane::from_point_and_normal(Point::new(0.0, 0.0, -2.0), Vector::new(0.0, 0.0, -1.0))
            .expect("a unit normal"),
        bottom_circle,
        Orientation::Reversed,
    );
    let top = cap(
        &mut builder,
        Plane::from_point_and_normal(Point::new(1.0, 0.0, 0.0), Vector::new(1.0, 0.0, 1.0))
            .expect("a direction"),
        top_ellipse,
        Orientation::Forward,
    );

    let round_the_wall: Vec<DraftStep> = [
        (bottom_circle, Orientation::Forward),
        (seam, Orientation::Forward),
        (top_ellipse, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ]
    .into_iter()
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("an edge of this build"),
        )
    })
    .collect();
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("round the bottom, up the seam, back round the top, down the seam");
    let wall = builder
        .add_face(Surface::Cylinder(pipe), &[wire])
        .expect("a face on the pipe");

    builder
        .add_shell(&[
            (bottom, Orientation::Forward),
            (top, Orientation::Forward),
            (wall, Orientation::Forward),
        ])
        .expect("three faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar cut on a slant described this way is a solid")
}
