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
//! **What this file measures is that the second spelling costs nothing.** Two
//! bars stacked end to end on one axis share their wall pipe. The union used to
//! stop at `"a point in a piece that every solid could place"` before any
//! corner was named, which masked the spelling; once that lifted, the circle
//! between two spellings of one wall was read as two pipes crossing, and
//! nothing could walk it (§231). Then two spellings left the wall as two faces,
//! and every later fuse of the answer refused on the circle between them
//! (§325.4). Now all four unions — one spelling and two, each either way round
//! — answer one bar of three faces with the volume worked out by hand.
//!
//! **A pin in the upper bar is where it still costs the answer.** With a thin
//! bar in the upper bar, the union with the lower bar is refused where one
//! spelling unites, because the fuse asks where the two spellings and the
//! pin's wall meet (§231.5).
//!
//! The comparison is not vacuous: the two bars really are two spellings, and
//! `two_bars_stacked_on_one_pipe_are_one_pipe_written_twice` proves it before
//! the fuse is asked — the stored numbers differ and `intersect` calls them one
//! surface.

use rcad_geom::SurfaceMeet;
use rcad_geom::{
    Axis, Cylinder, Deflection, EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector,
    intersect,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Model, Rule, SolidBuilder, SolidKey};

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
fn what_the_union_says(upper_axis_origin: f64) -> [Said; 2] {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let lower = a_bar_whose_wall_names_its_axis_at(&mut model, 0.0, 2.0, LOWER_ORIGIN);
    let upper = a_bar_whose_wall_names_its_axis_at(&mut model, 2.0, 4.0, upper_axis_origin);

    [[lower, upper], [upper, lower]].map(|[arguments, tools]| {
        let mut model = model.clone();
        let mut log = ProvenanceLog::new();
        let fused = model
            .fuse(&[arguments], &[tools], Rule::Union, &policy, &mut log)
            .unwrap_or_else(|refused| {
                panic!("the stacked bars, wall at {upper_axis_origin}: {refused:?}")
            });
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|broken| {
                panic!(
                    "the stacked bars, wall at {upper_axis_origin}, came back broken: {broken:?}"
                )
            });
        let [solid] = fused.solids() else {
            panic!("two bars end to end are one bar: {:?}", fused.solids());
        };
        let solid = model.solid(*solid).expect("this model made the key");
        let mut faces: Vec<String> = solid
            .faces()
            .map(|face| format!("{:?}", face.surface()))
            .collect();
        faces.sort();
        let measured = solid
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            )
            .expect("a bar measures");
        Said {
            faces: faces.join("; "),
            volume: measured.volume(),
            bound: measured.volume_error(),
        }
    })
}

/// What one union of the stacked bars came back as.
#[derive(Debug, PartialEq)]
struct Said {
    faces: String,
    volume: f64,
    bound: f64,
}

/// **The measurement.** Spelling the wall twice leaves the wall one face, on
/// the same spelling whichever bar went first.
///
/// Four runs: one spelling and two, each fused both ways round. Each spelling
/// gives the same faces either way round, and all four are the one bar four
/// tall ([`the_stacked_bars_unite_into_one_bar_however_the_wall_is_written`]).
/// The two spellings are one surface to the fuse (§231), and the joiner joins
/// pieces of one pipe however it is written, onto the lowest spelling by bits,
/// which is the lower bar's here. Before that it joined only pieces with the
/// same bits (§229.2), the wall stayed two faces meeting on the circle at
/// `z = 2`, and every later fuse of the answer refused on that circle (§325.4).
///
/// The faces and not the volumes are compared across the order: with one
/// spelling the two orders measure 50.20344246266881 and 50.20344246266873,
/// both within their bound of 16π, so bit-identical volumes are not something
/// the fuse gives either way round even with nothing spelled twice.
#[test]
fn spelling_the_wall_of_a_stacked_bar_twice_leaves_the_wall_one_face() {
    let [one_spelling_forwards, one_spelling_backwards] = what_the_union_says(LOWER_ORIGIN);
    let [two_spellings_forwards, two_spellings_backwards] = what_the_union_says(UPPER_ORIGIN);

    assert_eq!(
        one_spelling_forwards.faces, one_spelling_backwards.faces,
        "with one spelling of the wall the two orders of the union disagreed"
    );
    assert_eq!(
        two_spellings_forwards.faces, two_spellings_backwards.faces,
        "with two spellings of the wall the two orders of the union disagreed"
    );
    assert_eq!(
        two_spellings_forwards.faces, one_spelling_forwards.faces,
        "the wall spelled twice came back as other faces than the wall spelled once"
    );
    assert_eq!(
        two_spellings_forwards.faces.matches("Cylinder(").count(),
        1,
        "{two_spellings_forwards:?}"
    );
}

/// The stacked bars unite into one bar of radius two and four tall, however the
/// wall is written and whichever goes first.
///
/// This used to stop at `"a point in a piece that every solid could place"`,
/// which masked the spelling (§51.7). Once that lifted, two spellings of one
/// wall read the circle between them as two pipes crossing, and nothing could
/// walk it (§231).
#[test]
fn the_stacked_bars_unite_into_one_bar_however_the_wall_is_written() {
    let by_hand = std::f64::consts::PI * RADIUS * RADIUS * 4.0;
    for spelling in [LOWER_ORIGIN, UPPER_ORIGIN] {
        for said in what_the_union_says(spelling) {
            assert!(
                (said.volume - by_hand).abs() <= said.bound,
                "the wall at {spelling}: {said:?} against {by_hand}"
            );
        }
    }
}

/// **A pin in the upper bar is where the second spelling still costs the
/// answer.**
///
/// Two bars stacked on one pipe, with a pin in the upper one, unite into one
/// solid when the upper wall is written about the lower bar's origin. Written
/// about a point of its own, the same union is refused
/// `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` in both orders, for a pin along
/// `−x` right through the upper bar and for a slanted one starting on its
/// axis. For the pin right through, a probe print showed what was asked:
/// where the lower wall, the pin's wall and the upper wall meet, which
/// `where_three_curved_surfaces_meet` in `rcad-geom` refuses because two of
/// the three are one surface written twice. One spelling answering is the
/// demonstration that the answer exists (§16.10), since the two spellings are
/// the same points ([`two_bars_stacked_on_one_pipe_are_one_pipe_written_twice`]).
///
/// **The routes are built for a branch no fuse reaches yet.** Each pin's wall
/// sorts between the two spellings, so each closed crossing of the pin and
/// the upper wall is stored the pin's way, and joining the walls onto the
/// lower spelling makes it the wall's to name: the branch of
/// `turn_round_each_closed_edge_the_joined_spelling_stores_the_other_way` for
/// two different surfaces, which only an assembly built by hand reaches today
/// (§231.5). Worked by hand and not run: for the pin right through, at
/// `(−2, 0, 15.5)` the wall's longitude is `z × (−1, 0, 0) = (0, −1, 0)` and
/// the pin's `(−1, 0, 0) × z = (0, 1, 0)`, so that crossing turns round, and
/// at `(2, 0, 15.5)` both are `(0, 1, 0)` and it does not. Once the refusal
/// lifts, two spellings are to answer here what one answers.
#[test]
fn two_bars_on_one_pipe_written_twice_with_a_pin_in_one_are_refused_where_one_spelling_unites() {
    for pinned in [a_pin_right_through(), a_slanted_pin_from_the_axis()] {
        let route = format!("{:?}", pinned.pin);
        let lower_wall = Surface::Cylinder(a_wall_naming_its_axis_at(LOWER_ORIGIN));
        let pin_wall = Surface::Cylinder(pinned.pin);
        let upper_wall = Surface::Cylinder(a_wall_naming_its_axis_at(pinned.written_apart_at));
        assert_eq!(
            [
                lower_wall.how_it_sorts_against(&pin_wall),
                pin_wall.how_it_sorts_against(&upper_wall)
            ],
            [std::cmp::Ordering::Less; 2],
            "{route}: the pin's wall has to sort between the two spellings or the branch is \
             not reached"
        );

        let one_spelling = pinned.united_with_the_lower_bar(LOWER_ORIGIN);
        for (lower_first, said) in [true, false].into_iter().zip(one_spelling) {
            let walls = said.unwrap_or_else(|refused| {
                panic!("{route}, one spelling, lower first {lower_first}: {refused:?}")
            });
            assert_eq!(
                (
                    walls.len(),
                    [lower_wall, pin_wall]
                        .map(|wall| walls.iter().filter(|&&face| face == wall).count())
                ),
                (1 + pinned.pin_pieces, [1, pinned.pin_pieces]),
                "{route}, one spelling, lower first {lower_first}: the wall is one face: \
                 {walls:?}"
            );
        }

        let two_spellings = pinned.united_with_the_lower_bar(pinned.written_apart_at);
        for (lower_first, said) in [true, false].into_iter().zip(two_spellings) {
            assert!(
                matches!(
                    said,
                    Err(Error::Geometry(
                        rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat
                    ))
                ),
                "{route}, two spellings, lower first {lower_first}: {said:?}"
            );
        }
    }
}

/// **A slanted pin starting on a bar's axis is refused only where the bar ends
/// across where the pin would come out the other side.**
///
/// A thin pipe through a fat one crosses it in two loops. A pin that starts
/// on the fat bar's axis makes only one of them, round `(−2, 0, 5.5)` here.
/// The other, round `(2, 0, 2.5)`, is where the pin would come out if it ran
/// on through the axis, and it spans `z = 1.875` to `3.125`, worked by hand:
/// the point `(2 cos θ, 2 sin θ, 4 + w)` of the bar's wall is on the pin's
/// where `w² + 3w cos θ − 4 cos² θ + 5.859375 = 0`, and `w` is furthest out at
/// `cos θ = 1`, `−2.125` and `−0.875`.
///
/// With the bar's end in that span, its two ends included, the pin and the bar
/// are refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` in both orders,
/// though no face of the pin comes near the loop. The end's plane and the two
/// walls are the three surfaces that meet there, but which three the fuse asks
/// is not traced. Everywhere else tried, from `−2` to `1.8125` and at `3.1875`
/// and `3.25`, they unite into one solid that validates. Q528's route put the
/// bar's end at `z = 2`, which is why [`a_slanted_pin_from_the_axis`] moves
/// the joint down to `1`.
///
/// From `z = 1.5` to `1.8125` they were refused
/// `CurvedFaceWithAHoleGoesTooFarRound` in both orders. There the plane of the
/// pin's bottom cap cuts the bar's wall in an ellipse that crosses the bar's
/// end, so the wall below it is a patch that does not go right round, more
/// than a quarter turn wide, with the loop round `(2, 0, 2.5)` as a hole in
/// it, and the mesher had no cut for a hole in such a patch. The wall above
/// the ellipse is a ring with the pin's own loop as an island, and the first
/// cut across it goes round that island from a corner at longitude
/// `2.7861714518995693`. That longitude did not come back bit for bit from a
/// whole turn on and back, so the mesher read the cut as crossing the ring's
/// rim where it sets off.
///
/// The pin's lowest point is `z = 3.6`, above every end tried, so the part of
/// the pin outside the bar is the same in every answer, and each volume less
/// the bar's own, `4π` per unit of height, is the same as at `z = 1`.
#[test]
fn a_slanted_pin_from_a_bars_axis_is_refused_only_where_the_bar_ends_across_where_it_would_come_out()
 {
    let policy = Policy::certified();
    let pinned = a_slanted_pin_from_the_axis();
    let [from, to] = pinned.along_the_pin;
    let the_pin_and_a_bar_from = |bottom: f64| {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = a_bar_whose_wall_names_its_axis_at(
            &mut model,
            bottom,
            pinned.top,
            pinned.written_apart_at,
        );
        let pin = a_bar_along(&mut model, pinned.pin, from, to);
        [[bar, pin], [pin, bar]].map(|[arguments, tools]| {
            let mut model = model.clone();
            let fused = model.fuse(
                &[arguments],
                &[tools],
                Rule::Union,
                &policy,
                &mut ProvenanceLog::new(),
            )?;
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|broken| {
                    panic!("the bar from {bottom}, came back broken: {broken:?}")
                });
            let [solid] = fused.solids() else {
                panic!(
                    "the bar from {bottom} and the pin are one solid: {:?}",
                    fused.solids()
                );
            };
            Ok::<(f64, f64), Error>(volume_of(&model, *solid))
        })
    };
    let the_bar_alone_from =
        |bottom: f64| std::f64::consts::PI * RADIUS * RADIUS * (pinned.top - bottom);

    for bottom in [1.875, 1.9375, 2.0, 2.5, 3.0, 3.0625, 3.125] {
        for said in the_pin_and_a_bar_from(bottom) {
            assert!(
                matches!(
                    said,
                    Err(Error::Geometry(
                        rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat
                    ))
                ),
                "the bar from {bottom}, whose end crosses the loop: {said:?}"
            );
        }
    }
    let [(at_one, at_one_bound), _] = the_pin_and_a_bar_from(1.0)
        .map(|said| said.unwrap_or_else(|refused| panic!("the bar from 1: {refused:?}")));
    let the_pin_outside_the_bar = at_one - the_bar_alone_from(1.0);
    for bottom in [-2.0, 1.0, 1.25, 1.5, 1.75, 1.8125, 3.1875, 3.25] {
        for said in the_pin_and_a_bar_from(bottom) {
            let (volume, bound) =
                said.unwrap_or_else(|refused| panic!("the bar from {bottom}: {refused:?}"));
            let outside = volume - the_bar_alone_from(bottom);
            assert!(
                (outside - the_pin_outside_the_bar).abs() <= bound + at_one_bound,
                "the bar from {bottom}: {volume} ± {bound} leaves {outside} of the pin outside \
                 the bar, and from 1 it leaves {the_pin_outside_the_bar} ± {at_one_bound}"
            );
        }
    }
}

const PIN_RADIUS: f64 = 0.5;

/// How far the pin along `−x` runs each way from the axis of the bar it is in:
/// past its wall, so that it goes right through. Along `−x` every corner of
/// it is exact.
const PIN_REACH: f64 = 4.0;

/// How far along its own axis the slanted pin runs, from the axis of the bar
/// it is in.
///
/// Both corners of a bar's seam have to sit exactly on its wall, and on a
/// slanted wall that needs `4 + t·0.6` to come out exact. Sixteen does, and
/// zero is the other end, where nothing rounds.
const SLANTED_LENGTH: f64 = 16.0;

/// The upper of two bars stacked on one pipe, with a pin in it.
struct APinnedBar {
    /// Where the lower bar, up from `z = 0`, meets it.
    joint: f64,
    top: f64,
    /// Where its wall names the axis when it is written apart from the lower
    /// bar's: where the pin's axis leaves the axis from.
    written_apart_at: f64,
    pin: Cylinder,
    /// Where the pin starts and stops along its own axis.
    along_the_pin: [f64; 2],
    /// How many faces the pin's wall is left as once the two bars unite.
    pin_pieces: usize,
}

/// The bars meet at `z = 2`, and a pin along `−x` runs right through the
/// upper one at `z = 15`.
///
/// Thirteen above the joint, so that a refusal of the union is not about the
/// pin being near the lower bar. The pin's wall is left in two pieces, one
/// each side of the bar.
fn a_pin_right_through() -> APinnedBar {
    APinnedBar {
        joint: 2.0,
        top: 20.0,
        written_apart_at: 15.0,
        pin: a_pin_wall(Point::new(0.0, 0.0, 15.0), Vector::new(-1.0, 0.0, 0.0)),
        along_the_pin: [-PIN_REACH, PIN_REACH],
        pin_pieces: 2,
    }
}

/// The bars meet at `z = 1`, and a pin slanting up and out along
/// `(−0.8, 0, 0.6)` starts on the upper one's axis at `(0, 0, 4)` and leaves
/// it round `(−2, 0, 5.5)`.
///
/// The route Q528 named, with the joint moved down from `z = 2`, where the
/// pin is refused before the lower bar comes into it
/// ([`a_slanted_pin_from_a_bars_axis_is_refused_only_where_the_bar_ends_across_where_it_would_come_out`]).
fn a_slanted_pin_from_the_axis() -> APinnedBar {
    APinnedBar {
        joint: 1.0,
        top: 8.0,
        written_apart_at: 4.0,
        pin: a_pin_wall(Point::new(0.0, 0.0, 4.0), Vector::new(-0.8, 0.0, 0.6)),
        along_the_pin: [0.0, SLANTED_LENGTH],
        pin_pieces: 1,
    }
}

fn a_pin_wall(origin: Point, direction: Vector) -> Cylinder {
    Cylinder::new(
        Axis::new(origin, direction).expect("a direction with a length"),
        PIN_RADIUS,
    )
    .expect("a positive radius")
}

impl APinnedBar {
    /// The lower bar united with this one, both ways round, with this one's
    /// wall naming its axis at `axis_origin`: the curved surfaces of the
    /// answer's faces, or the refusal.
    ///
    /// An answer is checked as far as it goes before it comes back: it is one
    /// solid, it validates, and its volume is the lower bar's worked out by
    /// hand and the pinned bar's as measured, since the two meet only on the
    /// plane of the joint.
    fn united_with_the_lower_bar(&self, axis_origin: f64) -> [Result<Vec<Surface>, Error>; 2] {
        let policy = Policy::certified();
        let route = format!("{:?}, wall at {axis_origin}", self.pin);
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let lower = a_bar_whose_wall_names_its_axis_at(&mut model, 0.0, self.joint, LOWER_ORIGIN);
        let upper =
            a_bar_whose_wall_names_its_axis_at(&mut model, self.joint, self.top, axis_origin);
        let [from, to] = self.along_the_pin;
        let pin = a_bar_along(&mut model, self.pin, from, to);
        let pinned = model
            .fuse(
                &[upper],
                &[pin],
                Rule::Union,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|refused| panic!("{route}: the pin in the bar: {refused:?}"));
        let [pinned] = pinned.solids() else {
            panic!(
                "{route}: a pin in a bar is one solid: {:?}",
                pinned.solids()
            );
        };
        let pinned = *pinned;
        let (pinned_volume, pinned_bound) = volume_of(&model, pinned);
        let lower_by_hand = std::f64::consts::PI * RADIUS * RADIUS * self.joint;

        [[lower, pinned], [pinned, lower]].map(|[arguments, tools]| {
            let mut model = model.clone();
            let fused = model.fuse(
                &[arguments],
                &[tools],
                Rule::Union,
                &policy,
                &mut ProvenanceLog::new(),
            )?;
            model
                .validate(&policy, &mut ProvenanceLog::new())
                .unwrap_or_else(|broken| panic!("{route}: came back broken: {broken:?}"));
            let [solid] = fused.solids() else {
                panic!(
                    "{route}: the two bars end to end are one solid: {:?}",
                    fused.solids()
                );
            };
            let (volume, bound) = volume_of(&model, *solid);
            assert!(
                (volume - lower_by_hand - pinned_volume).abs() <= bound + pinned_bound,
                "{route}: {volume} ± {bound} against {lower_by_hand} and {pinned_volume} ± \
                 {pinned_bound}"
            );
            Ok(model
                .solid(*solid)
                .expect("this model made the key")
                .faces()
                .map(|face| face.surface())
                .filter(|surface| surface.as_plane().is_none())
                .collect())
        })
    }
}

/// A round bar on `wall` from `from` to `to` along its axis, with its seam
/// where the wall meets the plane `y = radius`.
///
/// That seam is the one place on a slanted wall where a corner sits on it
/// with nothing rounded: `(point − origin) × direction` is the radius times
/// `y × direction`, whose length is the direction's own, so the wall's sign
/// there is exactly zero — as long as `origin + t·direction` is exact
/// ([`SLANTED_LENGTH`]).
fn a_bar_along(model: &mut Model, wall: Cylinder, from: f64, to: f64) -> SolidKey {
    let [ox, oy, oz] = wall.axis().origin().coordinates();
    let along = wall.axis().direction();
    let [dx, dy, dz] = along.components();
    assert_eq!(dy, 0.0, "the seam is put square to the axis along y");
    let on_the_axis = |t: f64| Point::new(ox + t * dx, oy, oz + t * dz);
    let on_the_seam = |t: f64| Point::new(ox + t * dx, oy + wall.radius(), oz + t * dz);

    let mut builder = SolidBuilder::new();
    let low = builder
        .add_vertex(on_the_seam(from))
        .expect("a finite corner");
    let high = builder
        .add_vertex(on_the_seam(to))
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

    let cap = |builder: &mut SolidBuilder, t: f64, outward: Vector, edge, turn| {
        let plane = Plane::from_point_and_normal(on_the_axis(t), outward)
            .expect("an axis direction is a normal");
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
        from,
        -along,
        low_circle,
        Orientation::Reversed,
    );
    let top_cap = cap(&mut builder, to, along, high_circle, Orientation::Forward);

    let round_the_wall = [
        (low_circle, Orientation::Forward),
        (seam, Orientation::Forward),
        (high_circle, Orientation::Reversed),
        (seam, Orientation::Reversed),
    ]
    .map(|(edge, turn)| {
        DraftStep::Along(
            builder
                .add_coedge(edge, turn)
                .expect("a use of the wall's edge"),
        )
    });
    let wire = builder
        .add_wire(&round_the_wall)
        .expect("the wall's one wire");
    let side = builder
        .add_face(Surface::Cylinder(wall), &[wire])
        .expect("the curved wall");

    builder
        .add_shell(&[
            (bottom_cap, Orientation::Forward),
            (top_cap, Orientation::Forward),
            (side, Orientation::Forward),
        ])
        .expect("a closed skin");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a bar described face by face")
}

/// What one bar's measuring says: its volume and the bound on it.
fn volume_of(model: &Model, solid: SolidKey) -> (f64, f64) {
    let measured = model
        .solid(solid)
        .expect("this model made the key")
        .measure(
            Deflection::of(0.01).expect("a positive length"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("a bar measures");
    (measured.volume(), measured.volume_error())
}
