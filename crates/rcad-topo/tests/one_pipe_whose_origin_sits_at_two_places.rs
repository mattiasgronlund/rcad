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
//! **A pin in the upper bar costs nothing either.** With a thin bar in the
//! upper bar, the union with the lower bar was refused where one spelling
//! unites, because the fuse asked where the two spellings and the pin's wall
//! meet (§231.5). Now both spellings unite, pin or no pin, with the wall one
//! face (Q1877).
//!
//! **A bar across the joint gives what one spelling gives.** A bar of the
//! wall's radius along `x` through the axis below the joint crosses the wall in
//! half ellipses that climb past the joint, and the cross's wall, united with
//! the upper bar first, then held each half twice: once with each spelling,
//! since the pickers of a half come in the order `intersect` sorts the two
//! pipes, and it sorts the cross's between the two spellings. Two spellings
//! were refused `Undecidable` where one was refused
//! `AFaceTrimmedByACurveThisCannotFollow`, and the half is now one curve
//! (Q2050). That refusal came from asking which side of a half ellipse a point
//! lies, and the wall is now counted along a ruling instead, so every one of
//! the four unions answers (Q2460).
//!
//! **Nor does a bar across where the two meet on its axis or below it.** With
//! the joint there the upper wall keeps both halves that climb above the
//! cross's axis, and so does the wall of one bar crossed by another of its
//! radius. An edge along such a half starts where all four halves meet, and
//! the fuse took the half it runs along from that corner alone, so the second
//! union was refused, one spelling and two alike, as was a pin through such a
//! cross (Q2461). The edge's other corner now says which half, and with the
//! wall counted along a ruling all eight unions answer (Q2460). One answer
//! did not measure, since two arcs of it named the spelling of the pipe that
//! neither face along them is on, and they now name the face's (Q2470). The
//! pin was then refused telling the parts of the
//! upright wall the halves cut apart, and the side of the plane each ellipse
//! lies in now tells them apart, so the pin unites too (Q2464).
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

/// **Two bars on one pipe written twice, with a pin in the upper one, unite as
/// one spelling does.**
///
/// Two bars stacked on one pipe, with a pin in the upper one, unite into one
/// solid that validates and measures what the two do apart, whether the upper
/// wall is written about the lower bar's origin or about a point of its own,
/// for a pin along `−x` right through the upper bar and for a slanted one
/// starting on its axis. The wall comes back one face, on the lower spelling,
/// and the pin's wall as many faces as with one spelling.
///
/// Written apart, this was refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`
/// in both orders (Q1877). The pin's wall was arranged with one of its loops
/// round the upper wall drawn as a crossing with the upper spelling and the
/// other as a crossing with the lower one, and the fuse asked where the lower
/// wall, the pin's wall and the upper wall meet, which
/// `where_three_curved_surfaces_meet` in `rcad-geom` refuses because two of
/// the three are one surface written twice. Two crossings whose partners are
/// one pipe are two crossings of one pair, and are placed as such.
///
/// **The routes are built for a branch no other fuse reaches.** Each pin's
/// wall sorts between the two spellings, so each closed crossing of the pin
/// and the upper wall is stored the pin's way, and joining the walls onto the
/// lower spelling makes it the wall's to name: the branch of
/// `turn_round_each_closed_edge_the_joined_spelling_stores_the_other_way` for
/// two different surfaces (§231.5). Worked by hand: for the pin right
/// through, at `(−2, 0, 15.5)` the wall's longitude is
/// `z × (−1, 0, 0) = (0, −1, 0)` and the pin's `(−1, 0, 0) × z = (0, 1, 0)`,
/// so that crossing turns round, and at `(2, 0, 15.5)` both are `(0, 1, 0)`
/// and it does not. A probe print in that branch confirmed it, in both
/// orders: the two stored ways read `(true, false)` at `(−2, 0, 15.5)` and
/// `(false, false)` at `(2, 0, 15.5)`, and for the slanted pin `(true, false)`
/// at `(−2, 0, 6.125)`. With that branch reading the wall's old spelling
/// twice, so that nothing turns round, the pin right through with two
/// spellings, lower bar first, comes back `AWireWillNotLieFlatOnItsFace`.
#[test]
fn two_bars_on_one_pipe_written_twice_with_a_pin_in_one_unite_as_one_spelling_does() {
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

        for (spelled, upper_axis_origin) in [
            ("one spelling", LOWER_ORIGIN),
            ("two spellings", pinned.written_apart_at),
        ] {
            let said = pinned.united_with_the_lower_bar(upper_axis_origin);
            for (lower_first, said) in [true, false].into_iter().zip(said) {
                let walls = said.unwrap_or_else(|refused| {
                    panic!("{route}, {spelled}, lower first {lower_first}: {refused:?}")
                });
                assert_eq!(
                    (
                        walls.len(),
                        [lower_wall, pin_wall]
                            .map(|wall| walls.iter().filter(|&&face| face == wall).count())
                    ),
                    (1 + pinned.pin_pieces, [1, pinned.pin_pieces]),
                    "{route}, {spelled}, lower first {lower_first}: the wall is one face, on \
                     the lower spelling: {walls:?}"
                );
            }
        }
    }
}

/// **A slanted pin starting on a bar's axis unites with the bar wherever the
/// bar ends, across where the pin would come out the other side included.**
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
/// were refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat` in both orders,
/// though no face of the pin comes near the loop (Q1909). The end's plane
/// crosses the pin's pipe in an ellipse behind the pin's own cap, and where
/// that meets the bar's rim is a plane and two pipes at a slant. The end then
/// left the ellipse out, since no face of the pin reaches it, and the wall
/// proved the pin's own loop, at `x ≤ 0`, clear of it and left out the loop
/// round `(2, 0, 2.5)`.
///
/// Once one wall mirrored the three, their meet answered, at about
/// `(1.974, ±0.32, 3)` with the end at `3`, and the fuse refused that end
/// `ResultIsNotManifold` (Q2196). With the end at `3.125`, the top of the
/// loop, the three meet in one double point on the rim's seam corner, and a
/// node there could not be put in order with the corner. The end and the
/// wall now leave the ellipse and the loop out where the meet answers too,
/// and put no node on the rim, since both lie past the pin's cap (Q2201).
/// Every end tried, from `−2` to `3.25`, unites into one solid that
/// validates. Q528's route put
/// the bar's end at `z = 2`, which is why [`a_slanted_pin_from_the_axis`]
/// moves the joint down to `1`.
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
fn a_slanted_pin_from_a_bars_axis_unites_with_the_bar_wherever_the_bar_ends() {
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

    let [(at_one, at_one_bound), _] = the_pin_and_a_bar_from(1.0)
        .map(|said| said.unwrap_or_else(|refused| panic!("the bar from 1: {refused:?}")));
    let the_pin_outside_the_bar = at_one - the_bar_alone_from(1.0);
    for bottom in [
        -2.0, 1.0, 1.25, 1.5, 1.75, 1.8125, 1.875, 1.9375, 2.0, 2.5, 3.0, 3.0625, 3.125, 3.1875,
        3.25,
    ] {
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

/// **A thin slanted disc whose seam corner sits on a pin's pipe behind the
/// pin's start unites with the pin.**
///
/// The disc is `8` in radius and `0.25` thick along `(1, 0, 1)`, its axis
/// through `(0, −8, 0)`, so its wall passes through the origin, and its seam
/// meets its top end there. The pin is `1` in radius along `(1, −8, 0)` through
/// `(0, 0, 1)`, from `1` to `8` along its axis, so its pipe passes exactly
/// through the origin too, `1` behind where the pin starts. The pin's pipe
/// crosses the disc's wall in a loop through that corner, and the disc's
/// bottom end in an ellipse, and both lie wholly behind the pin's start.
///
/// The bottom end left its ellipse out, since no face of the pin reaches it,
/// and put no node on its rim where the ellipse meets it, since the wall's loop
/// lies behind the pin's start too. But the wall drew its loop, because a
/// corner of the wall is on it, and split the rim at the two places the loop
/// crosses it. The rim was one edge on the end's side and three on the wall's,
/// and the union was refused `ResultIsNotManifold` in both orders (Q2246).
///
/// The union and the intersection hold each solid once between them, so their
/// volumes add up to the disc's, `16π`, and the pin's, `7π`.
#[test]
fn a_thin_slanted_disc_whose_seam_corner_is_on_a_pins_pipe_behind_its_start_unites_with_it() {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let disc_wall = Cylinder::new(
        Axis::new(Point::new(0.0, -8.0, 0.0), Vector::new(1.0, 0.0, 1.0)).expect("an axis"),
        8.0,
    )
    .expect("a positive radius");
    let disc = a_bar_with_its_seam_towards(
        &mut model,
        disc_wall,
        -0.25,
        0.0,
        Vector::new(0.0, 1.0, 0.0),
    );
    let pin_wall = Cylinder::new(
        Axis::new(Point::new(0.0, 0.0, 1.0), Vector::new(1.0, -8.0, 0.0)).expect("an axis"),
        1.0,
    )
    .expect("a positive radius");
    let pin =
        a_bar_with_its_seam_towards(&mut model, pin_wall, 1.0, 8.0, Vector::new(0.0, 0.0, 1.0));
    let one_solid = |rule: Rule, what: &str, [arguments, tools]: [SolidKey; 2]| {
        let route = format!("the {what}, disc first {}", arguments == disc);
        let mut model = model.clone();
        let fused = model
            .fuse(
                &[arguments],
                &[tools],
                rule,
                &policy,
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|refused| panic!("{route}: {refused:?}"));
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|broken| panic!("{route} came back broken: {broken:?}"));
        let [solid] = fused.solids() else {
            panic!("{route} is one solid: {:?}", fused.solids());
        };
        volume_of(&model, *solid)
    };

    let (common, common_bound) = one_solid(Rule::Intersection, "intersection", [disc, pin]);
    let both = std::f64::consts::PI * (16.0 + 7.0);
    for order in [[disc, pin], [pin, disc]] {
        let (union, bound) = one_solid(Rule::Union, "union", order);
        assert!(
            (union + common - both).abs() <= bound + common_bound,
            "the union, {union} ± {bound}, and the intersection, {common} ± {common_bound}, \
             against {both}"
        );
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
/// pin was refused before the lower bar came into it
/// ([`a_slanted_pin_from_a_bars_axis_unites_with_the_bar_wherever_the_bar_ends`]).
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

/// **A bar across where two bars on one pipe meet unites with them as one
/// spelling does.**
///
/// The lower bar runs up from `z = 0` to the joint at [`CROSSED_JOINT`], the
/// upper on to `6`, and a bar of the same radius along `x` from `−4` to `4`
/// crosses the pipe at `z = 3`, so that its wall reaches from `z = 1` up past
/// the joint to `5`. The cross is united with the upper bar first, and the
/// lower bar with that, both ways round, with the upper wall naming its axis
/// where the lower one does and at the joint.
///
/// Two walls of one radius whose axes meet cross in four half ellipses, which
/// meet where the walls touch, at `(0, ±2, 3)` on the lower bar. In the second
/// union the cross's wall holds the two that climb past the joint twice: once
/// as its crossings with the lower wall, and once along its own edges, which
/// the upper wall left it with. A half is picked out by two flat surfaces,
/// listed with the one leaning towards the pipe `intersect` sorts first ahead,
/// and it sorts the cross's pipe after the lower wall named at `0` and before
/// the upper named at the joint. So with two spellings each half came with its
/// pickers the other way round, was drawn as two curves, and where it crosses
/// the joint at `(−1, −√3, 4)` the turn between the two was asked of the
/// lower wall's gradient and the upper's, which are one: refused `Undecidable`
/// in both orders on an `orient3d` that straddled nothing (Q2050).
///
/// One spelling was refused too, `AFaceTrimmedByACurveThisCannotFollow`: every
/// ray the fuse fired to place a piece met a wall with a half ellipse among
/// its sides, and `holds.rs` could not tell which side of such a side a point
/// lies. It now counts the sides such a wall's ruling through the point
/// crosses above it, since each half lies on a plane through where the two
/// axes meet (Q2460). Every answer is one solid of the volume worked out by
/// hand.
#[test]
fn a_bar_across_where_two_bars_on_one_pipe_meet_unites_with_them_as_one_spelling_does() {
    let mut wrong = Vec::new();
    for axis_origin in [LOWER_ORIGIN, CROSSED_JOINT] {
        let answers = the_bars_united_across(CROSSED_JOINT, axis_origin);
        for (lower_first, answer) in [true, false].into_iter().zip(answers) {
            if answer.0.is_err() {
                wrong.push(format!(
                    "wall at {axis_origin}, lower bar first {lower_first}: {}",
                    answer.1
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// Where the lower bar meets the upper, above the axis of the bar across them.
const CROSSED_JOINT: f64 = 4.0;

/// **A bar across two bars on one pipe that meet on its axis or below it
/// unites with them, one spelling and two.**
///
/// The shape of
/// [`a_bar_across_where_two_bars_on_one_pipe_meet_unites_with_them_as_one_spelling_does`]
/// with the joint at `3`, level with the cross's axis, or at `2`, below it.
/// The upper wall then keeps both halves that climb above the cross's axis,
/// each from `(0, 2, 3)` to `(0, −2, 3)` over the top of the wall, and the
/// cross's seam cuts each where it tops the wall, at `(±2, 0, 5)`. So an edge
/// along such a half runs from a corner where all four halves meet, held by
/// the pickers of every half, to one only its own half holds. The fuse took
/// the half an edge runs along from its starting corner alone, and the second
/// union was refused `CannotTellWhichCurveOfACrossingAFaceRunsAlong` in both
/// orders, one spelling and two alike (Q2461). The other corner now says
/// which.
///
/// Every answer was then refused `AFaceTrimmedByACurveThisCannotFollow` on the
/// cross's wall, as with the joint above the axis (Q2460). Rays fired from the
/// pieces land on the ruling through a corner of such a wall, since the shape
/// is symmetric about `y = 0`, and the count along that ruling now answers
/// there too. Every answer is one solid of the volume worked out by hand.
///
/// **One of them united and did not measure.** With the joint at `2`, the
/// upper wall named at the joint, and the cross and the upper bar first, the
/// two arcs where the lower wall bites the cross's wall below the joint,
/// through `(±2, 0, 1)`, were stored naming the upper wall's spelling as the
/// surface they cross, though the faces either side of each are the cross's
/// wall and the lower wall. `rcad_geom::which_two_surfaces_a_run_is_between`
/// took that for a third surface and handed the walk the lower wall and the
/// upper, which are one pipe, so the walk had no point between an arc's ends
/// and the mesh drew the arc as its chord: refused `CannotMeshAFace`,
/// `BoundarySideStraysFurtherThanTheDeflection`, straying `2.0`. Lower bar
/// first, the arcs named the lower wall and measured. The fuse now names the
/// spelling of the face along such an arc, and all eight answers measure
/// (Q2470).
#[test]
fn a_bar_across_two_bars_on_one_pipe_meeting_on_its_axis_or_below_unites_with_them() {
    let mut wrong = Vec::new();
    for joint in [3.0, 2.0] {
        for axis_origin in [LOWER_ORIGIN, joint] {
            let answers = the_bars_united_across(joint, axis_origin);
            for (lower_first, answer) in [true, false].into_iter().zip(answers) {
                if answer.0.is_err() {
                    wrong.push(format!(
                        "joint at {joint}, wall at {axis_origin}, lower bar first {lower_first}: {}",
                        answer.1
                    ));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// What one fuse answered: how many faces its one solid has, or the refusal,
/// and how to say it.
type Answer = (Result<usize, Error>, String);

/// The upper bar, from `joint` to `6` with its wall naming its axis at
/// `axis_origin`, united with a bar along `x` across the pipe at `z = 3`, and
/// the lower bar, from `0` to the joint, united with that, both ways round.
///
/// An answer is checked as far as it goes before it comes back: it is one
/// solid, it validates, and its volume is the three bars' less the part the
/// cross shares with the pipe, worked out by hand: `16 r³/3` for two pipes of
/// one radius whose axes cross square. A union that answers and then will not
/// be measured comes back as that refusal, since measuring is how the volume
/// is read.
fn the_bars_united_across(joint: f64, axis_origin: f64) -> [Answer; 2] {
    let policy = Policy::certified();
    let route = format!("the joint at {joint}, the upper wall at {axis_origin}");
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let lower = a_bar_whose_wall_names_its_axis_at(&mut model, 0.0, joint, LOWER_ORIGIN);
    let upper = a_bar_whose_wall_names_its_axis_at(&mut model, joint, 6.0, axis_origin);
    let across = a_bar_across_the_pipe(&mut model);
    let crossed = model
        .fuse(
            &[upper],
            &[across],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("{route}: the cross and the upper bar: {refused:?}"));
    let [crossed] = crossed.solids() else {
        panic!(
            "{route}: the cross and the upper bar are one solid: {:?}",
            crossed.solids()
        );
    };
    let crossed = *crossed;
    let by_hand =
        std::f64::consts::PI * RADIUS * RADIUS * (6.0 + 8.0) - 16.0 * RADIUS.powi(3) / 3.0;

    [[lower, crossed], [crossed, lower]].map(|[arguments, tools]| {
        let mut model = model.clone();
        let fused = match model.fuse(
            &[arguments],
            &[tools],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        ) {
            Ok(fused) => fused,
            Err(why) => {
                let said = format!("refused {why:?}");
                return (Err(why), said);
            }
        };
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|broken| panic!("{route}: came back broken: {broken:?}"));
        let [solid] = fused.solids() else {
            panic!(
                "{route}: the three bars are one solid: {:?}",
                fused.solids()
            );
        };
        let measured = model
            .solid(*solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.01).expect("a positive length"),
                &policy,
                &mut ProvenanceLog::new(),
            );
        let (volume, bound) = match measured {
            Ok(measured) => (measured.volume(), measured.volume_error()),
            Err(why) => {
                let said = format!("united, and measuring that refused {why:?}");
                return (Err(why), said);
            }
        };
        assert!(
            (volume - by_hand).abs() <= bound,
            "{route}: {volume} ± {bound} against {by_hand}"
        );
        let faces = model
            .solid(*solid)
            .expect("this model made the key")
            .faces()
            .count();
        (Ok(faces), format!("one solid of {faces} faces"))
    })
}

/// A bar of the pipe's radius along `x` from `−4` to `4`, across the pipe at
/// `z = 3`, with its seam on top.
fn a_bar_across_the_pipe(model: &mut Model) -> SolidKey {
    let wall = Cylinder::new(
        Axis::new(Point::new(0.0, 0.0, 3.0), Vector::new(1.0, 0.0, 0.0))
            .expect("a direction with a length"),
        RADIUS,
    )
    .expect("a positive radius");
    a_bar_with_its_seam_towards(model, wall, -4.0, 4.0, Vector::new(0.0, 0.0, 1.0))
}

/// **A pin through a cross of two bars of one radius unites with it.**
///
/// A bar up `z` from `0` to `6` united with the bar across it at `z = 3` is a
/// cross whose two walls meet in four halves of two ellipses, each from
/// `(0, 2, 3)` to `(0, −2, 3)`. A pin of radius `0.25` along `y` from `−4` to
/// `4` through `(0, 0, 5.5)` goes right through the upright bar above the
/// other, and clear of it: the bar across reaches `z = 5` at most.
///
/// The cross and the pin were refused
/// `CannotTellWhichCurveOfACrossingAFaceRunsAlong` in both orders, since every
/// edge where the two walls meet starts where all four halves do (Q2459's
/// probe, Q2461). It is the shape of
/// [`a_bar_across_two_bars_on_one_pipe_meeting_on_its_axis_or_below_unites_with_them`]
/// with nothing spelled twice and no joint at all. With each such edge's other
/// corner saying which half it runs along, both orders were refused
/// `CannotTellTheCellsOfAFaceApart` on the upright wall instead (Q2464). The
/// halves cut it into four parts, the two windows into the bar across and the
/// parts above and below them, and no half closes on its own, so none had a
/// side to read that tells which of the four the pin's loop round
/// `(0, 2, 5.5)` lies in. The two halves on one plane make a closed ellipse,
/// and the side of that plane tells all four apart: only the part above is on
/// the loop's side of both planes. Read so, both orders unite.
#[test]
fn a_pin_through_a_cross_of_two_bars_of_one_radius_unites_with_it() {
    let mut wrong = Vec::new();
    for (cross_first, (answer, said)) in [true, false]
        .into_iter()
        .zip(the_cross_united_with_a_thin_pin())
    {
        if answer.is_err() {
            wrong.push(format!("cross first {cross_first}: {said}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// The cross of the upright bar and the bar across it, united with a thin pin
/// through the upright bar above the other, both ways round.
///
/// An answer is checked as far as it goes before it comes back: it is one
/// solid, it validates, and its volume is the cross's as measured and the
/// pin's own less what it shares with the upright bar,
/// `∫ 4 √((0.25² − x²)(2² − x²)) dx` over `|x| ≤ 0.25`, which has no closed
/// form and is summed here finely enough to be far inside the measured bound.
fn the_cross_united_with_a_thin_pin() -> [Answer; 2] {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let upright = a_bar_whose_wall_names_its_axis_at(&mut model, 0.0, 6.0, LOWER_ORIGIN);
    let across = a_bar_across_the_pipe(&mut model);
    let cross = model
        .fuse(
            &[upright],
            &[across],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|refused| panic!("the cross: {refused:?}"));
    let [cross] = cross.solids() else {
        panic!("a cross is one solid: {:?}", cross.solids());
    };
    let cross = *cross;
    let (cross_volume, cross_bound) = volume_of(&model, cross);
    let pin_wall = Cylinder::new(
        Axis::new(Point::new(0.0, 0.0, 5.5), Vector::new(0.0, 1.0, 0.0))
            .expect("a direction with a length"),
        THIN_PIN_RADIUS,
    )
    .expect("a positive radius");
    let pin =
        a_bar_with_its_seam_towards(&mut model, pin_wall, -4.0, 4.0, Vector::new(0.0, 0.0, 1.0));
    let by_hand = std::f64::consts::PI * THIN_PIN_RADIUS * THIN_PIN_RADIUS * 8.0
        - what_a_thin_pin_shares_with_the_upright_bar();

    [[cross, pin], [pin, cross]].map(|[arguments, tools]| {
        let cross_first = arguments == cross;
        let mut model = model.clone();
        let fused = match model.fuse(
            &[arguments],
            &[tools],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        ) {
            Ok(fused) => fused,
            Err(why) => {
                let said = format!("refused {why:?}");
                return (Err(why), said);
            }
        };
        model
            .validate(&policy, &mut ProvenanceLog::new())
            .unwrap_or_else(|broken| {
                panic!("cross first {cross_first}: came back broken: {broken:?}")
            });
        let [solid] = fused.solids() else {
            panic!(
                "cross first {cross_first}: the cross and the pin are one solid: {:?}",
                fused.solids()
            );
        };
        let (volume, bound) = volume_of(&model, *solid);
        assert!(
            (volume - cross_volume - by_hand).abs() <= bound + cross_bound,
            "cross first {cross_first}: {volume} ± {bound} against {cross_volume} ± \
             {cross_bound} and {by_hand} of the pin"
        );
        let faces = model
            .solid(*solid)
            .expect("this model made the key")
            .faces()
            .count();
        (Ok(faces), format!("one solid of {faces} faces"))
    })
}

const THIN_PIN_RADIUS: f64 = 0.25;

/// `∫ 4 √((a² − x²)(r² − x²)) dx` over `|x| ≤ a`, for `a` the thin pin's
/// radius and `r` the upright bar's, by Simpson's rule on the angle
/// `x = a sin t`, where the integrand is smooth.
fn what_a_thin_pin_shares_with_the_upright_bar() -> f64 {
    let steps = 1000;
    let half_turn = std::f64::consts::PI;
    let at = |t: f64| {
        let x = THIN_PIN_RADIUS * t.sin();
        let across = THIN_PIN_RADIUS * t.cos();
        4.0 * across * (RADIUS * RADIUS - x * x).sqrt() * across
    };
    let width = half_turn / f64::from(steps);
    let sum: f64 = (0..=steps)
        .map(|step| {
            let weight = if step == 0 || step == steps {
                1.0
            } else if step % 2 == 1 {
                4.0
            } else {
                2.0
            };
            weight * at(-half_turn / 2.0 + width * f64::from(step))
        })
        .sum();
    sum * width / 3.0
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
    a_bar_with_its_seam_towards(model, wall, from, to, Vector::new(0.0, 1.0, 0.0))
}

/// A round bar on `wall` from `from` to `to` along its axis, with its seam the
/// wall's radius off the axis along `towards`, a unit vector square to it.
fn a_bar_with_its_seam_towards(
    model: &mut Model,
    wall: Cylinder,
    from: f64,
    to: f64,
    towards: Vector,
) -> SolidKey {
    let [ox, oy, oz] = wall.axis().origin().coordinates();
    let along = wall.axis().direction();
    let [dx, dy, dz] = along.components();
    assert_eq!(
        along.dot(towards),
        0.0,
        "the seam is put square to the axis"
    );
    let off = towards.components().map(|side| wall.radius() * side);
    let on_the_axis = |t: f64| Point::new(ox + t * dx, oy + t * dy, oz + t * dz);
    let on_the_seam = |t: f64| {
        let [x, y, z] = on_the_axis(t).coordinates();
        Point::new(x + off[0], y + off[1], z + off[2])
    };

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
