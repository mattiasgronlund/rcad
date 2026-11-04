//! A KiCad LED whose front disc sits 0.4 inside the rim round it, with the
//! disc's arcs drawn between two planes a rounding apart.
//!
//! `LED_D5.0mm_Horizontal_O6.35mm_Z3.0mm.step` is a pipe of radius 2.5 about
//! the line `x = 1.27, z = 3`, with a flange in front of it whose rim is a pipe
//! of radius 2.9 about the same line. The disc on the front of the body, in the
//! plane `y = -6.35`, is bounded by three arcs of the body's pipe, and across
//! each arc is the flange's front, whose plane is the disc's own written down
//! a rounding apart: `(5.6e-17, 1, 0)` and `(-1.0e-16, 1, -1.1e-15)`.
//!
//! Every fuse of it refused `Undecidable` in the check that it does not cross
//! itself, on the rim against the disc: side of plane at the rim's corner
//! `(-1.23, -6.35, 4.4697)`, enclosure `5.3e-15` (Q2929). The disc is nowhere
//! near the rim, but the baked solid could not say so: each arc's side named
//! only the flange's front across it, a plane, and the pipe the arc runs along
//! was dropped. Bake now carries that pipe, and the check skips a flat face a
//! pipe provably holds strictly inside (Q2947).
//!
//! Every fuse here then stopped where the flange's flat, `x = -1.23`, meets
//! the body's pipe and the rim's: two pipes about one axis, a rounding apart
//! in direction, which never meet. That was refused
//! `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, and is answered now (Q2938).
//!
//! Past that, without Q2940, every fuse here stopped on the body's pipe (face
//! slot 8), refused `AFaceTrimmedByACurveThisCannotFollow`: the fuse baked
//! each arc where the dome meets the pipe against the ball across it. Bake now
//! reads such an arc against the rim's plane (Q2940), and every fuse here gets
//! past the body's pipe too.
//!
//! Each then stopped `Undecidable` on side of plane at the rim's corner on the
//! front plane, `(-1.23, -6.35, 4.4697)` in this LED, enclosure `5.3e-15`, in
//! the check that the LED does not cross itself. The flange's front drew the
//! three arcs round the disc as where its plane crosses the disc's, two
//! spellings of one plane, which is a straight line nowhere near the arcs, and
//! `place_the_corners` (fuse/arrange.rs) asked which side of the disc's plane
//! the rim's corner lies. The arrangement now draws such an arc where the
//! face's plane crosses the pipe it runs along (Q2966).
//!
//! Each then stopped `Undecidable` on side of cylinder at the front corner,
//! where the circle round the disc touches the flange's flat, against a box
//! that ran along the flat, `z` from `0.834` to `4.170` in this LED. Bake
//! solved that corner from three planes, the flat, the flange's front and the
//! disc, the last two a rounding apart, so they meet anywhere along the flat's
//! front edge. Bake now reads a corner in two fans of faces out of the model
//! and never solves it (Q2939).
//!
//! Each then stopped `Undecidable` with the one record side of plane, at the
//! corner on the flange's front where its flat meets its rim above the axis:
//! `(-1.23, -6.35, 4.4697)` here, enclosure `5.3e-15`, the point and enclosure
//! this LED stopped at before Q2966. `passes_through` (fuse/arrange.rs) asked
//! which side of the disc's plane that corner is, solved from the rim, the
//! flat and the flange's front, and no reading of a corner solved on a pipe
//! took the line the two planes share across the rim. One does now (Q3001).
//!
//! Each then stopped on which comes first round the loop the flat cuts the
//! rim in: the corner below the axis, or where the disc's plane crosses that
//! loop, `2.1e-15` from it. The reading of a corner beside a plane cut through
//! the loop took only a corner written in the file; it takes a solved one now,
//! on the side `passes_through` reads it (Q3001).
//!
//! The box far off this LED, and the far boxes of three of the six others
//! Q2929 found refusing as this one did, then stopped `Undecidable` with the
//! one record side of plane, at the corner below the axis where the flat meets
//! the rim: `(-1.23, -6.35, 1.5303)` here. That point is where the rim, the
//! flat and the flange's front meet, and it can be read as where the line the
//! two planes share crosses the rim. In the three D5 LEDs a node of a walk
//! asked it, and only a corner bake solved so was read that way. In
//! `LED_D3.0mm_Horizontal_O3.81mm_Z2.0mm.step`, whose flat's normal is
//! `(-1, -2.1e-32, 2.6e-47)`, such a corner asked it, but a product in the
//! pipe's value at the meet would round, and the reading gave up. A node is
//! read so now, and a product that would round is read off an interval
//! (Q3017). A node of a walk that cannot tell its side is passed over too, and
//! the walk read off a node that can (Q3009); either reading alone gets each
//! of the four far boxes past that corner.
//!
//! Past the corner, the far box of
//! `LED_D3.0mm_Horizontal_O3.81mm_Z2.0mm.step` stopped on the order along a
//! crossing of two points on the side of its lead further from its flat,
//! `x = 2.24`, `1.2e32` out along `-y`: a meet as far out as the one Q3019
//! orders, and it gets past it now that Q3019's ordering is in (Q3027).
//!
//! A box on this LED's flat, and the far box of
//! `LED_D3.0mm_Horizontal_O6.35mm_Z6.0mm.step`, stopped on the order along the
//! straight line a plane `x = c` cuts the rim in, between where the disc's
//! plane and the flange's front cross it, `8.0e-16` apart along it here: the
//! box's side `x = -1.33` here, a lead's side `x = -0.3` in the D3. The two
//! planes cross `x = c` in lines that lean apart by a rounding, so nothing
//! read their order off the stored numbers. It is now read off which side of
//! one plane the other's meet lies, and which way that plane faces along the
//! line (Q3059).
//!
//! The last two, `LED_D3.0mm_Horizontal_O6.35mm_Z10.0mm.step` and
//! `LED_D5.0mm_Horizontal_O6.35mm_Z9.0mm.step`, stopped on the flange's front,
//! face slot 2, the plane `y = -6.35`, refused
//! `CannotTellTheCellsOfAFaceApart`: the refusal Q2974 and Q2982 cleared from
//! the fronts of two LEDs of `a_led_whose_flat_touches_its_body_at_a_corner.rs`.
//! The flat that the file has touch the hole's circle on the front, at a corner
//! in two fans of faces, is stored a rounding inside it, so its line crosses
//! the circle at two places about `1e-8` either side of the corner, and the
//! arrangement kept them as nodes of their own, with a sliver between them.
//! Each place is now folded into the corner when the line and the circle stay
//! within a rounding of each other all the way from it to the corner (Q3010).
//! The far box of the D3 stopped on the order along a crossing where its
//! lead's side `x = -0.3` meets the rim below the axis, between the same two
//! planes' meets, and gets past it the same way (Q3059). The D5's stopped on
//! side of plane where its flat meets its rim above the axis, the same kind of
//! corner as the one below, and gets past it the same two ways.
//!
//! The far boxes of this LED, of the two D5 LEDs whose axis is 15 up, of the
//! D5 whose axis is 9 up and of the D3 whose axis is 2 up stopped then on the
//! order along a crossing of two points within `1.1e-14` of each other, where
//! the side of a lead meets the body's pipe on the flange's front below the
//! axis: `x = -0.3` in the four D5 LEDs, `x = 2.24` in the D3. The front draws
//! the circle it meets the body's pipe in twice, as the edge of its hole and
//! as the pipe cut across it, and in the D5 LEDs the lead's side meets the two
//! a rounding apart (Q3021, Q3031). Which of two such points comes first is
//! now read off where the line crosses each pipe, worked out as exactly as
//! `f64` holds it (Q3054).
//!
//! All five stopped then on the order along the straight line where the flange's
//! front crosses the body's front disc, one plane written twice: between the
//! front corner, where the flat touches the body's pipe, and where the flat
//! meets that line. That point is solved from the flat, the front and the
//! disc, three planes two of which are a rounding apart, and its box runs
//! along the flat's line from `z = 1.02` to `3.52` here, round the corner. The
//! `D5` LED with its front `y = -1.27` of
//! `a_led_whose_flat_touches_its_body_at_a_corner.rs` stopped there too, and
//! so did the box on this LED's flat.
//!
//! The far boxes of the two D3 LEDs stopped on the order along that same
//! line, between its two meets with the body's pipe, which the front draws
//! twice, as the edge of its hole and as the pipe cut across it. Each meet is
//! solved from the front, the disc and one writing of the pipe, and the two
//! are worked out into one box: at `x = -0.2197`, above the axis, in the D3
//! whose axis is 6 up, and at `x = 2.659`, below it, in the one 10 up. A
//! pipe's face now knows the side of a plane its edges go round it on in a
//! ring (Q3048), and with that the far boxes of both, under intersection and
//! under difference, stop refused `CannotTellTheCellsOfAFaceApart` on the
//! flange's front, the plane `y = -6.35` with the body's pipe a hole in it:
//! the stop of the D3 whose axis is 2 up, in
//! `an_led_whose_dome_meets_its_pipe_in_two_arcs.rs`.
//!
//! The arrangement took the line the flange's front crosses the disc in
//! through every corner on both planes without asking whether it runs
//! through it, because the disc is one of the surfaces round the corner. The
//! face's own plane written a second time is now always asked (Q3071), and
//! every fuse here gets past both stops. None answers:
//!
//! - This LED's far box and the box on its flat, and the far boxes of the D5
//!   whose front is `y = -6.35` and whose axis is 15 up and of the D3 whose
//!   axis is 6 up, are refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`.
//! - The far boxes of the D5 whose axis is 9 up and of the D3 whose axis is
//!   10 up were refused `TwoCurvesTouchWithoutCrossing` on face slot 18, the
//!   side `x = -0.3` of a lead: what the D3 whose axis is 2 up, in
//!   `an_led_whose_dome_meets_its_pipe_in_two_arcs.rs`, was refused on the
//!   side `x = 0.3` of its lead.
//! - The far box of `LED_D3.0mm_Horizontal_O3.81mm_Z2.0mm.step` is refused
//!   `Invalid(WireTooShort { count: 2 })`: a piece of a face bounded by two
//!   straight sides.
//!
//! Untraced, every one. These tests pin each.
//!
//! The far box of the D5 whose front is `y = -3.81` and whose axis is 15 up
//! was refused `Undecidable` on the order along a crossing of two places on
//! the body's pipe near the flat, about `(-1.22688, -3.81, 15.12484)` and
//! `(-1.22666, -3.81, 15.12915)`, the first in a box `0.16` long along `y`,
//! across the front. That box is where the body's front disc meets the body's
//! pipe written twice, worked out about the axis of a pipe the disc is a
//! rounding off square to, and it is now narrowed onto the plane it lies on
//! (Q3111). The far box was refused then `TwoCurvesTouchWithoutCrossing` on
//! face slot 18, the side `x = -0.3` of a lead, as the far boxes of the D5
//! whose axis is 9 up and of the D3 whose axis is 10 up were.
//!
//! Each lead's side is two faces joined, and its edge along the line
//! `x = -0.3, y = -0.3` is written twice, below the bend corner on the plane
//! `y = -0.3` and above it on the plane of the bend's slanted leg, which
//! crosses the side a rounding off that line. Each writing ran on past the
//! corner beside the other. Two crossings of a face's own edges that are one
//! straight line are now folded into one, each side keeping the name of the
//! face across it, and where a third surface meets the two writings is one
//! place (Q3112). The far box of each of these three now answers under
//! intersection, with nothing. Under difference:
//!
//! - The D5 whose axis is 9 up and the D3 whose axis is 10 up are refused
//!   `TwoEndsOfAnEdgeOnACurvedSurfaceRoundToOnePoint` on the flange's rim, at
//!   the front `y = -6.35`, where a lead's plane `z = c` along the rim meets
//!   the flange's front and the body's front disc at one `f64` triple: the
//!   stop of the D3 whose axis is 2 up, in
//!   `an_led_whose_dome_meets_its_pipe_in_two_arcs.rs`.
//! - The D5 whose front is `y = -3.81` and whose axis is 15 up is refused
//!   `DynamicRangeTooWide`, at a ratio of `4.0e16`.
//!
//! Untraced, both.

use std::{env, fs, path::PathBuf};

use rcad_geom::Surface;
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Error, Invalid, Model, ProfileBuilder, Rule, SolidKey};

/// Where the box that misses the LED goes.
const FAR_OFF: [f64; 3] = [10.0, 10.0, 10.0];

/// The LED this file is about.
const THIS_ONE: &str = "LED_D5.0mm_Horizontal_O6.35mm_Z3.0mm.step";

/// Where the box on this LED's flat goes, round its front corner.
const ON_THE_FLAT: [f64; 3] = [-1.23, -6.35, 3.0];

/// The two other LED files whose far box stops where three surfaces meet, as
/// this one's does.
const TWO_MORE_WHERE_THREE_SURFACES_MEET: [&str; 2] = [
    "LED_D5.0mm_Horizontal_O6.35mm_Z15.0mm.step",
    "LED_D3.0mm_Horizontal_O6.35mm_Z6.0mm.step",
];

/// The two LED files whose far box's difference stops on the flange's rim at
/// the front, each with the radius of that rim.
const ON_THE_RIM: [(&str, f64); 2] = [
    ("LED_D5.0mm_Horizontal_O6.35mm_Z9.0mm.step", 2.9),
    ("LED_D3.0mm_Horizontal_O6.35mm_Z10.0mm.step", 1.9),
];

#[test]
fn three_leds_get_past_their_front_written_twice_and_stop_where_three_surfaces_meet() {
    every_fuse_refuses(
        THIS_ONE,
        &both_rules_round(&[FAR_OFF, ON_THE_FLAT]),
        |_, error| where_three_surfaces_meet_needs_two_of_them_flat(error),
    );
    for file in TWO_MORE_WHERE_THREE_SURFACES_MEET {
        every_fuse_refuses(file, &both_rules_round(&[FAR_OFF]), |_, error| {
            where_three_surfaces_meet_needs_two_of_them_flat(error)
        });
    }
}

/// Since Q3112 the two writings of the edge along each lead's side are one
/// line, and the far box shares nothing with either LED.
#[test]
fn two_leds_get_past_the_sides_of_their_leads_and_stop_where_their_rim_meets_their_front_written_twice()
 {
    for (file, radius) in ON_THE_RIM {
        the_far_box_shares_nothing_with(file);
        every_fuse_refuses(file, &[(FAR_OFF, Rule::Difference)], |model, error| {
            two_ends_round_to_one_point_on_the_front_of_a_rim(radius, model, error)
        });
    }
}

/// The D5 whose front is `y = -3.81` and whose axis is 15 up puts in order the
/// places where its body's front disc meets its body's pipe written twice
/// since Q3111, and since Q3112 the two writings of the edge along each lead's
/// side are one line. Its far box shares nothing with it, and the difference
/// is refused for its range.
#[test]
fn a_d5_led_15_up_gets_past_the_sides_of_its_leads_and_its_difference_is_refused_for_its_range() {
    let file = "LED_D5.0mm_Horizontal_O3.81mm_Z15.0mm.step";
    the_far_box_shares_nothing_with(file);
    every_fuse_refuses(file, &[(FAR_OFF, Rule::Difference)], |_, error| {
        matches!(
            error,
            Error::Invalid(Invalid::DynamicRangeTooWide { ratio }) if (4.0e16..4.1e16).contains(ratio)
        )
    });
}

#[test]
fn a_d3_led_2_up_gets_past_where_its_flat_meets_its_front_written_twice_and_is_refused_a_piece_of_two_straight_sides()
 {
    every_fuse_refuses(
        "LED_D3.0mm_Horizontal_O3.81mm_Z2.0mm.step",
        &both_rules_round(&[FAR_OFF]),
        |_, error| matches!(error, Error::Invalid(Invalid::WireTooShort { count: 2 })),
    );
}

/// The LED in `file` and the far box, fused under intersection, answer with
/// nothing, and leave a model `validate` accepts.
fn the_far_box_shares_nothing_with(file: &str) {
    let (mut model, solid) = the_led(file);
    let knife = a_box_round(&mut model, FAR_OFF);
    let fused = model
        .fuse(
            &[solid],
            &[knife],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{file}: the far box refused: {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| {
            panic!("{file}: the far box left a model that is not valid: {why:?}")
        });
    assert!(
        fused.solids().is_empty(),
        "{file}: the far box shares {} solids with the LED, and it shares none",
        fused.solids().len()
    );
}

/// A box round each centre, to be fused with the LED under intersection and
/// under difference.
fn both_rules_round(centres: &[[f64; 3]]) -> Vec<([f64; 3], Rule)> {
    centres
        .iter()
        .flat_map(|&centre| [Rule::Intersection, Rule::Difference].map(|rule| (centre, rule)))
        .collect()
}

/// The LED in `file` and a box of side 0.2 centred on each centre of `fuses`,
/// fused under its rule, each refused as `the_refusal` takes, handed the model
/// it was refused in, which holds every face of the LED.
fn every_fuse_refuses(
    file: &str,
    fuses: &[([f64; 3], Rule)],
    the_refusal: impl Fn(&Model, &Error) -> bool,
) {
    let (led, solid) = the_led(file);
    for &(centre, rule) in fuses {
        let mut model = led.clone();
        let knife = a_box_round(&mut model, centre);
        match model.fuse(
            &[solid],
            &[knife],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) {
            Err(error) => assert!(
                the_refusal(&model, &error),
                "{file}: {rule:?} with a box round {centre:?} refused {error:?}, which is not the \
                 refusal this pins"
            ),
            Ok(_) => panic!(
                "{file}: {rule:?} with a box round {centre:?} answers: check its volume as \
                 f179ca02a did"
            ),
        }
    }
}

fn where_three_surfaces_meet_needs_two_of_them_flat(error: &Error) -> bool {
    matches!(
        error,
        Error::Geometry(rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
    )
}

/// Refused `TwoEndsOfAnEdgeOnACurvedSurfaceRoundToOnePoint` on a flange's
/// rim, a pipe of `radius`, at a point of the LED's front `y = -6.35`.
fn two_ends_round_to_one_point_on_the_front_of_a_rim(
    radius: f64,
    model: &Model,
    error: &Error,
) -> bool {
    let Error::TwoEndsOfAnEdgeOnACurvedSurfaceRoundToOnePoint { face, at } = error else {
        return false;
    };
    let face = model.face(*face).expect("the face is the LED's own");
    let Surface::Cylinder(rim) = face.surface() else {
        return false;
    };
    (rim.radius() - radius).abs() < 1e-9 && (at.y() + 6.35).abs() < 1e-9
}

fn the_led(file: &str) -> (Model, SolidKey) {
    let path =
        PathBuf::from(env::var_os("RCAD_STEP_CORPUS").expect("`just t` sets RCAD_STEP_CORPUS"))
            .join("kicad")
            .join(file);
    let bytes = fs::read(&path).expect("the LED is in the corpus");
    let step = StepFile::read(&bytes).expect("it parses");
    let import = Import::read(&step, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the LED imports");
    let [(_, solid)] = import.solids()[..] else {
        panic!("{file} is one solid, not {}", import.solids().len())
    };
    (import.model().clone(), solid)
}

fn a_box_round(model: &mut Model, centre: [f64; 3]) -> SolidKey {
    let side = 0.2;
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let low = rcad_geom::Vector::new(centre[0] - 0.1, centre[1] - 0.1, centre[2] - 0.1);
    model
        .add_extrusion(
            &square,
            side,
            &rcad_geom::Isometry::translation(low).expect("a finite shift"),
        )
        .expect("a box")
}
