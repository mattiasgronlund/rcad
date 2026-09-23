//! A fuse over faces whose sides are arcs of the **ellipses** two leaning
//! planes cut a cone in (`docs/decisions.md` §211.8's loose end).
//!
//! §211 gave a cone's closed crossing a centre and taught `holds.rs`'s
//! `the_way_into_an_arc` to read a picker off it. `fuse/arrange.rs` carried a
//! second copy of that function whose second line asked for a `Surface::Sphere`
//! by name, so a plane and a cone fell out of it before any other test was
//! reached and every arc of a cone's rim — circle or ellipse — came back
//! `AFaceTrimmedByACurveThisCannotFollow` from the arrangement. The two are one
//! function now, and this is the fuse asking it about an ellipse.
//!
//! # The solid: a lens of a cone between two leaning lids
//!
//! Both lids hold the line `x = 0, z = 4`: the lower one is `x + z = 4` and the
//! upper one is `z − x = 4`, and the solid is the part of the inside of a cone
//! of slope a half — apex at the origin, about `+z` — between them. Each lid
//! leans a quarter turn off the axis, far more than the cone's own half angle,
//! so each cuts an **ellipse** and not a circle: `(1 + s²)(n·w)² − s²|n|²|w|²`
//! comes to `0.75c²` for both, with `c` the lid normal's repeated component,
//! which is §211.2's door standing open.
//!
//! **Neither lid goes through the apex**, and that is deliberate. A plane
//! through the apex cuts the cone in two straight rulings rather than in a
//! closed curve, and the arrangement folds two corners on those two rulings into
//! one node and refuses `EdgeHasNoLength` — a defect of its own, still there,
//! and not this file's.
//!
//! Three faces, three edges, two corners: the two lids are each half of their
//! own ellipse, the piece of the cone between them is bounded by both arcs, and
//! the chord `x = 0, z = 4` closes each lid.
//!
//! # Why every number in it is exact
//!
//! Each lid's unit normal has a repeated `1/√2` in it, which is irrational, so a
//! corner is only exactly on a lid by construction and never by luck. Each lid is
//! written through `(0, 0, 4)` on the axis, so its stored offset is `±4c` with no
//! sum to round, and each corner `(0, ±2, 4)` dots to `(∓c)·0 + 0·(±2) + (±c)·4`
//! — the same `±4c` to the bit, because the lid normals' `y` components are
//! exactly zero. The corners sit on the cone for `cones/mod.rs`'s reason: a
//! power-of-two slope makes `slope · height` exact, and `0.5 · 4 = 2`.
//!
//! # Which of the two ways into an arc each lid asks
//!
//! [`the_way_into_an_arc`] reads a picker one of two ways, and the two lids ask
//! one each — on purpose, because the second way is the one §211's arithmetic is
//! for.
//!
//! The **upper lid**'s arc is the bigger half of its ellipse, so the ellipse's
//! centre lies on the arc's own side of the chord and no half-space can hold the
//! arc in front of it and the centre behind. Its picker is the chord's own plane
//! `x ≥ 0`, which holds both corners: that is the first way, and it never asks
//! where the centre is.
//!
//! The **lower lid**'s arc is the smaller half — its ellipse runs from `x = −4`
//! to `x = 4/3` and the arc kept is `x ≥ 0` — so the centre is at `x = −4/3` and
//! `x ≥ −0.5` holds the whole arc strictly in front of it with the centre
//! strictly behind. That is the second way, and it is the one that walks
//! §211.3's expression over an ellipse: `Cone::which_way_a_closed_crossings
//! _centre_lies` hands back the pole's direction and the apex, and the sign is
//! cleared of its division. On the base the fuse never got that far, because the
//! `Surface::Sphere` two lines above had already refused.
//!
//! [`the_way_into_an_arc`]: rcad_topo

use std::collections::BTreeMap;

use rcad_geom::{
    Axis, Cone, EdgeRun, HalfSpace, Isometry, Orientation, Picker, Plane, Point, Scale, Surface,
    Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftStep, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey, WireStepView,
};

/// Slope a half, so `slope · height` rounds to nothing at the corners.
const SLOPE: f64 = 0.5;

/// Where both lids cross the axis, and how far up the cone the corners sit.
const LIDS_MEET_THE_AXIS_AT: f64 = 4.0;

/// How far behind the lower lid's corners its picker's plane stands.
///
/// Anything between nothing and `4/3` puts the corners strictly in front of it
/// and that ellipse's centre strictly behind; a half is a round number in the
/// middle of that range.
const THE_LOWER_LIDS_PICKER_STANDS_BACK: f64 = 0.5;

/// A bar driven right through the lens, in through the lower lid and out through
/// the upper one: the rectangle `x ∈ [0.5, 1]`, `y ∈ [−0.25, 0.25]` run from
/// `z = 2.75` to `z = 5.5`.
///
/// **The two lids are the only faces of the lens it crosses.** Its furthest
/// corner is `1.031` from the axis and the cone is already `1.375` wide where the
/// bar starts, so it never reaches the cone's skin. The lower lid crosses it
/// between `z = 3` and `z = 3.5` and the upper one between `z = 4.5` and `z = 5`,
/// all four well inside its ends — so both of its own ends are clear of the lens
/// and each lid cuts it rather than grazing it.
const BAR_CORNERS: [[f64; 2]; 4] = [[0.5, -0.25], [1.0, -0.25], [1.0, 0.25], [0.5, 0.25]];
const BAR_FROM: f64 = 2.75;
const BAR_HEIGHT: f64 = 2.75;

/// The lens is a solid, and the two ellipse arcs are its own sides.
///
/// The fixture in its own right: the corners are certified on the cone and on
/// both lids, and each arc is named by a half-space that tells it from its
/// sibling. This passes on the base too — the arrangement is what the base
/// cannot do, not the description.
#[test]
fn a_lens_of_a_cone_between_two_leaning_lids_is_a_solid() {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let lens = a_lens_of_a_cone(&mut model);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a lens described this way is a sound model");
    let mut surfaces: Vec<Surface> = model
        .solid(lens)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .collect();
    let mut want = vec![
        Surface::Plane(the_lower_lid()),
        Surface::Plane(the_upper_lid()),
        Surface::Cone(the_cone()),
    ];
    let key = |surface: &Surface| format!("{surface:?}");
    surfaces.sort_by_key(key);
    want.sort_by_key(key);
    assert_eq!(
        surfaces, want,
        "a lens is two leaning lids and a piece of a cone"
    );
    assert_eq!(
        how_many_faces_along_each_edge(&model, lens).len(),
        3,
        "a lens has three edges: the two arcs and the chord"
    );
    for (edge, how_many) in how_many_faces_along_each_edge(&model, lens) {
        assert_eq!(how_many, 2, "the lens has {how_many} faces along {edge}");
    }
}

/// The arrangement no longer gives up where an arc of a cone's rim is an
/// **ellipse**, under every rule.
///
/// On the base this asks a question the fuse cannot answer at all:
/// `arrange.rs`'s `the_way_into_an_arc` looked for a `Surface::Sphere` among the
/// pair, found a cone, and returned `None` — and its one caller,
/// `which_way_an_edge_runs_round`, turns that `None` into
/// `AFaceTrimmedByACurveThisCannotFollow` by an `ok_or`. So the base answers
/// that error and this test fails; with the two copies folded into one it does
/// not, and this test passes.
///
/// **It is written as "not that refusal" rather than as an answer**, and that is
/// a measured limit and not a hedge. This pair walks straight into a *second*
/// wall once the arc is read: `select.rs` cannot place a point inside a piece of
/// the bar's own flat end, and refuses `Undecidable` naming "a point inside a
/// piece of a face" with an infinite enclosure. Two other shapes were tried for
/// this file and each found a different wall of its own — a half frustum cut
/// down its axis folds two corners on the cone's two rulings into one node and
/// refuses `EdgeHasNoLength`, and a block big enough to hold the lens leaves a
/// piece of its own wall bounded by a hyperbola with two nodes and no third to
/// wind round. None of the three is about the centre of an ellipse, and none is
/// this lane's to fix; what this file can say is that the arc is now read, and
/// it says exactly that.
#[test]
fn the_arrangement_reads_the_way_into_a_cones_ellipse_arc() {
    for rule in [Rule::Union, Rule::Intersection, Rule::Difference] {
        let (mut model, lens, bar) = a_lens_and_a_bar();
        let said = model.fuse(
            &[lens],
            &[bar],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        assert!(
            !matches!(
                said,
                Err(Error::AFaceTrimmedByACurveThisCannotFollow { .. })
            ),
            "under {rule:?} the arrangement gave up on an ellipse arc it can read: {said:?}"
        );
    }
}

/// The lens, and a bar poking out through its upper lid, in one model.
fn a_lens_and_a_bar() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let lens = a_lens_of_a_cone(&mut model);
    let square = ProfileBuilder::new(BAR_CORNERS.to_vec())
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle on the grid");
    let bar = model
        .add_extrusion(
            &square,
            BAR_HEIGHT,
            &Isometry::translation(Vector::new(0.0, 0.0, BAR_FROM)).expect("a finite shift"),
        )
        .expect("a rectangle extruded straight up is a solid");
    (model, lens, bar)
}

/// The cone the lens is cut out of.
fn the_cone() -> Cone {
    Cone::new(Axis::z(), SLOPE).expect("a positive slope")
}

/// The lid `x + z = 4`, facing out of the lens, which is on its `≥` side.
fn the_lower_lid() -> Plane {
    the_lid(-1.0, -1.0)
}

/// The lid `z − x = 4`, facing out of the lens, which is on its `≤` side.
fn the_upper_lid() -> Plane {
    the_lid(-1.0, 1.0)
}

/// A lid through `(0, 0, 4)` on the axis, leaning `across` in `x` and `up` in
/// `z`, which is what keeps the two corners exactly on it — see the module doc.
fn the_lid(across: f64, up: f64) -> Plane {
    Plane::from_point_and_normal(
        Point::new(0.0, 0.0, LIDS_MEET_THE_AXIS_AT),
        Vector::new(across, 0.0, up),
    )
    .expect("a vector with a length")
}

/// The plane the chord lies in, which is the upper lid's picker and the second
/// surface both arcs are named against.
fn the_chord_plane() -> Plane {
    Plane::from_point_and_normal(Point::origin(), Vector::new(-1.0, 0.0, 0.0))
        .expect("the x axis is a unit normal")
}

/// A lens of a cone between two leaning lids, described face by face through the
/// public builder.
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

/// How many coedges — which is how many faces — walk each edge of a solid.
///
/// An edge is named by the `Debug` spelling of its key, because a key carries no
/// order of its own and a `BTreeMap` needs one. It is exact all the same: two
/// different keys never print alike.
fn how_many_faces_along_each_edge(model: &Model, solid: SolidKey) -> BTreeMap<String, usize> {
    let view = model.solid(solid).expect("a solid the fuse made");
    let mut along: BTreeMap<String, usize> = BTreeMap::new();
    for face in view.faces() {
        for wire in face.wires() {
            for step in wire.steps() {
                let WireStepView::Along(coedge) = step else {
                    panic!("these answers are made of coedges and nothing else: {step:?}")
                };
                *along
                    .entry(format!("{:?}", coedge.edge().key()))
                    .or_default() += 1;
            }
        }
    }
    along
}
