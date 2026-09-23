//! One pipe written with its axis origin at two points of its own axis.
//!
//! §51.7 books *"one pipe, two names, from an axis written either way"* as an
//! open row. Reversing the axis is one way to write one pipe twice; **sliding
//! the origin along it is the other**, and nothing had measured it.
//! `Cylinder::new(Axis::new(p, w), r)` and
//! `Cylinder::new(Axis::new(p + t·w, w), r)` are the same set of points, and
//! `name_of_surface_bits` — which `canonical_cut_surfaces` dedups by and which
//! a corner is named by — reads the origin straight out, so the fuse has two
//! names for them.
//!
//! **The route in is the public API and nothing weaker.** `Axis::new`,
//! `Cylinder::new` and `SolidBuilder::add_face` are all `pub`, and the bars
//! below are built through them: a caller hands the surface it chose, and the
//! kernel stores what it was handed. §16.10's test for an unreachability claim
//! is the public door, not today's callers.
//!
//! **What this file measures is that the fuse never sees the difference
//! today.** Two bars stacked end to end on one axis share their wall pipe, and
//! the union stops at `"a point in a piece that every solid could place"` — the standing
//! curved-piece refusal `fusing_two_bars_side_by_side.rs` names — *before* any
//! corner is named. It stops there identically whether the wall is spelled once
//! or twice, and identically either way round. So the second spelling costs
//! nothing that can be seen from outside, and the row stays booked rather than
//! urgent.
//!
//! The comparison is not vacuous: the two bars really are two spellings, and
//! `two_bars_stacked_on_one_pipe_are_one_pipe_written_twice` proves it before
//! the fuse is asked — the stored numbers differ and `intersect` calls them one
//! surface. If the curved-piece refusal is ever lifted and the two spellings
//! then diverge, this file fails and the row has to be reopened.

use rcad_geom::SurfaceMeet;
use rcad_geom::{
    Axis, Cylinder, EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector, intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Model, Rule, SolidBuilder, SolidKey};

const RADIUS: f64 = 2.0;

/// Where the lower bar's wall says its axis passes through.
const LOWER_ORIGIN: f64 = 0.0;

/// Where the upper bar's wall says the *same* axis passes through, when the two
/// are spelled differently. It is the plane the two bars meet on, which is what
/// a builder writing the upper bar in its own frame would naturally pick.
const UPPER_ORIGIN: f64 = 2.0;

/// The wall pipe of a bar whose axis origin is written at `axis_origin` up `z`.
fn a_wall_naming_its_axis_at(axis_origin: f64) -> Cylinder {
    Cylinder::new(
        Axis::new(
            Point::new(0.0, 0.0, axis_origin),
            Vector::new(0.0, 0.0, 1.0),
        )
        .expect("a direction with a length"),
        RADIUS,
    )
    .expect("a positive radius")
}

/// The two spellings are two spellings, and not two pipes.
///
/// Both halves are load-bearing for everything below: if the numbers were the
/// same there would be no defect to look for, and if the surfaces were really
/// different the fuse would be right to treat them as two.
#[test]
fn two_bars_stacked_on_one_pipe_are_one_pipe_written_twice() {
    let lower = a_wall_naming_its_axis_at(LOWER_ORIGIN);
    let upper = a_wall_naming_its_axis_at(UPPER_ORIGIN);

    assert_ne!(
        lower.axis().origin().coordinates(),
        upper.axis().origin().coordinates(),
        "the two spellings must store different numbers or there is nothing here"
    );
    assert_eq!(
        lower.axis().direction().components(),
        upper.axis().direction().components(),
        "the slide is along the axis, so the direction is untouched"
    );

    let meet = intersect(
        &Surface::Cylinder(lower),
        &Surface::Cylinder(upper),
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("two pipes about one line are a pair `intersect` answers");
    assert_eq!(
        meet,
        SurfaceMeet::SameSurface,
        "the kernel's own authority on how two surfaces meet calls these one surface"
    );
}

/// A round bar from `bottom` to `top`, with its wall naming the axis at
/// `axis_origin`.
///
/// Built through [`SolidBuilder`] rather than `Model::add_round_bar` for one
/// reason: `add_round_bar` always writes `Axis::z()` and moves the whole solid,
/// so the wall's origin follows the placement and the two spellings cannot be
/// told apart from the placement's own rounding. Here the placement is the
/// identity for both bars and the only thing that changes is the number the
/// wall stores.
fn a_bar_whose_wall_names_its_axis_at(
    model: &mut Model,
    bottom: f64,
    top: f64,
    axis_origin: f64,
) -> SolidKey {
    let mut builder = SolidBuilder::new();
    let low = builder
        .add_vertex(Point::new(RADIUS, 0.0, bottom))
        .expect("a finite corner");
    let high = builder
        .add_vertex(Point::new(RADIUS, 0.0, top))
        .expect("a finite corner");
    let seam = builder
        .add_edge(low, high, EdgeRun::Straight)
        .expect("the seam up the wall");
    let low_circle = builder
        .add_edge(low, low, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the bottom cap circle");
    let high_circle = builder
        .add_edge(high, high, EdgeRun::RightRoundWhereItsFacesCross)
        .expect("the top cap circle");

    let cap = |builder: &mut SolidBuilder, z: f64, outward: f64, edge, turn| {
        let plane =
            Plane::from_point_and_normal(Point::new(0.0, 0.0, z), Vector::new(0.0, 0.0, outward))
                .expect("the z axis is a unit normal");
        let coedge = builder.add_coedge(edge, turn).expect("one use of the rim");
        let wire = builder
            .add_wire(&[DraftStep::Along(coedge)])
            .expect("a cap is bounded by one whole circle");
        builder
            .add_face(Surface::Plane(plane), &[wire])
            .expect("a flat cap")
    };
    let bottom_cap = cap(
        &mut builder,
        bottom,
        -1.0,
        low_circle,
        Orientation::Reversed,
    );
    let top_cap = cap(&mut builder, top, 1.0, high_circle, Orientation::Forward);

    let round_the_wall = [
        DraftStep::Along(
            builder
                .add_coedge(low_circle, Orientation::Forward)
                .expect("round the bottom"),
        ),
        DraftStep::Along(
            builder
                .add_coedge(seam, Orientation::Forward)
                .expect("up the seam"),
        ),
        DraftStep::Along(
            builder
                .add_coedge(high_circle, Orientation::Reversed)
                .expect("round the top"),
        ),
        DraftStep::Along(
            builder
                .add_coedge(seam, Orientation::Reversed)
                .expect("down the seam"),
        ),
    ];
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("the wall's one wire");
    let wall = builder
        .add_face(
            Surface::Cylinder(a_wall_naming_its_axis_at(axis_origin)),
            &[wire],
        )
        .expect("the curved wall");

    builder
        .add_shell(&[
            (bottom_cap, Orientation::Forward),
            (top_cap, Orientation::Forward),
            (wall, Orientation::Forward),
        ])
        .expect("a closed skin");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar described face by face")
}

/// What the union of the two stacked bars comes back with, both ways round.
///
/// The answer is rendered rather than matched on, so that a refusal is compared
/// down to the point it names and the box it names it in, and an answer would
/// be compared as the faces it is made of. Two spellings that started to differ
/// anywhere in either would show up here.
fn what_the_union_says(upper_axis_origin: f64) -> [String; 2] {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let lower = a_bar_whose_wall_names_its_axis_at(&mut model, 0.0, 2.0, LOWER_ORIGIN);
    let upper = a_bar_whose_wall_names_its_axis_at(&mut model, 2.0, 4.0, upper_axis_origin);

    [[lower, upper], [upper, lower]].map(|[arguments, tools]| {
        let mut model = model.clone();
        let mut log = ProvenanceLog::new();
        match model.fuse(&[arguments], &[tools], Rule::Union, &policy, &mut log) {
            Ok(fused) => fused
                .solids()
                .iter()
                .map(|solid| {
                    let mut faces: Vec<String> = model
                        .solid(*solid)
                        .expect("this model made the key")
                        .faces()
                        .map(|face| format!("{:?}", face.surface()))
                        .collect();
                    faces.sort();
                    faces.join("; ")
                })
                .collect::<Vec<_>>()
                .join(" | "),
            Err(refused) => format!("{refused:?}"),
        }
    })
}

/// **The measurement.** Spelling the wall twice changes nothing the fuse says.
///
/// Four runs: one spelling and two, each fused both ways round. All four must
/// say the same thing. Today all four are the standing curved-piece refusal, and
/// [`the_stacked_bars_stop_where_no_solid_could_place_the_curved_pieces_point`] names it —
/// so this test is about agreement and that one is about where the agreement
/// sits.
#[test]
fn spelling_the_wall_of_a_stacked_bar_twice_changes_nothing_the_fuse_says() {
    let [one_spelling_forwards, one_spelling_backwards] = what_the_union_says(LOWER_ORIGIN);
    let [two_spellings_forwards, two_spellings_backwards] = what_the_union_says(UPPER_ORIGIN);

    assert_eq!(
        one_spelling_forwards, one_spelling_backwards,
        "with one spelling of the wall the two orders of the union disagreed"
    );
    assert_eq!(
        two_spellings_forwards, two_spellings_backwards,
        "with two spellings of the wall the two orders of the union disagreed"
    );
    assert_eq!(
        one_spelling_forwards, two_spellings_forwards,
        "writing the wall's axis origin at the other end of the same axis changed the answer"
    );
}

/// Where the stacked pair stops today, by name.
///
/// Not `is_err()`: the worth of this is *which* refusal, because it is the one
/// that keeps the spelling from ever being asked about. The union of two bars
/// on one axis cuts each wall by the other's cap plane, which leaves a piece of
/// a cylinder bounded by arcs. The winding fan **does** find a point inside one
/// of those; what nothing can do is place that point against both bars (§48,
/// `fusing_two_bars_side_by_side.rs`). §217 split the one refusal name that used
/// to cover both searches, and this is the half that actually fires here — the
/// sentence above said the fan could not wind a point, and it was never the fan.
///
/// **This is the masking, and it is why §51.7's row stays booked.** Lift this
/// refusal and the spelling question is live again.
#[test]
fn the_stacked_bars_stop_where_no_solid_could_place_the_curved_pieces_point() {
    for spelling in [LOWER_ORIGIN, UPPER_ORIGIN] {
        for said in what_the_union_says(spelling) {
            assert!(
                said.contains("a point in a piece that every solid could place"),
                "the stacked bars stopped somewhere else with the wall named at {spelling}: {said}"
            );
        }
    }
}
