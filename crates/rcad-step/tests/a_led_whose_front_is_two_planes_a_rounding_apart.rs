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
//!   10 up are refused `TwoCurvesTouchWithoutCrossing` on face slot 18, the
//!   side `x = -0.3` of a lead: what the D3 whose axis is 2 up, in
//!   `an_led_whose_dome_meets_its_pipe_in_two_arcs.rs`, is refused on the side
//!   `x = 0.3` of its lead.
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
//! (Q3111). The far box is refused now `TwoCurvesTouchWithoutCrossing` on face
//! slot 18, the side `x = -0.3` of a lead, as the far boxes of the D5 whose
//! axis is 9 up and of the D3 whose axis is 10 up are.

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

/// The two LED files whose far box stops on the side `x = -0.3` of a lead.
const ON_THE_SIDE_OF_A_LEAD: [&str; 2] = [
    "LED_D5.0mm_Horizontal_O6.35mm_Z9.0mm.step",
    "LED_D3.0mm_Horizontal_O6.35mm_Z10.0mm.step",
];

#[test]
fn three_leds_get_past_their_front_written_twice_and_stop_where_three_surfaces_meet() {
    every_fuse_refuses(THIS_ONE, &[FAR_OFF, ON_THE_FLAT], |_, error| {
        where_three_surfaces_meet_needs_two_of_them_flat(error)
    });
    for file in TWO_MORE_WHERE_THREE_SURFACES_MEET {
        every_fuse_refuses(file, &[FAR_OFF], |_, error| {
            where_three_surfaces_meet_needs_two_of_them_flat(error)
        });
    }
}

#[test]
fn two_leds_get_past_their_front_written_twice_and_stop_where_two_curves_touch_on_the_side_of_a_lead()
 {
    for file in ON_THE_SIDE_OF_A_LEAD {
        every_fuse_refuses(file, &[FAR_OFF], two_curves_touch_on_the_side_of_a_lead);
    }
}

/// The D5 whose front is `y = -3.81` and whose axis is 15 up puts in order the
/// places where its body's front disc meets its body's pipe written twice
/// since Q3111, and its far box stops where two curves touch on the side
/// `x = -0.3` of a lead.
#[test]
fn a_d5_led_15_up_gets_past_where_its_disc_meets_its_pipe_written_twice_and_stops_where_two_curves_touch_on_the_side_of_a_lead()
 {
    every_fuse_refuses(
        "LED_D5.0mm_Horizontal_O3.81mm_Z15.0mm.step",
        &[FAR_OFF],
        two_curves_touch_on_the_side_of_a_lead,
    );
}

#[test]
fn a_d3_led_2_up_gets_past_where_its_flat_meets_its_front_written_twice_and_is_refused_a_piece_of_two_straight_sides()
 {
    every_fuse_refuses(
        "LED_D3.0mm_Horizontal_O3.81mm_Z2.0mm.step",
        &[FAR_OFF],
        |_, error| matches!(error, Error::Invalid(Invalid::WireTooShort { count: 2 })),
    );
}

/// The LED in `file` and a box of side 0.2 centred on each of `centres`, fused
/// under intersection and under difference, each refused as `the_refusal`
/// takes, handed the model it was refused in, which holds every face of the
/// LED.
fn every_fuse_refuses(
    file: &str,
    centres: &[[f64; 3]],
    the_refusal: impl Fn(&Model, &Error) -> bool,
) {
    let (led, solid) = the_led(file);
    for &centre in centres {
        for rule in [Rule::Intersection, Rule::Difference] {
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
                    "{file}: {rule:?} with a box round {centre:?} refused {error:?}, which is not \
                     the refusal this pins"
                ),
                Ok(_) => panic!(
                    "{file}: {rule:?} with a box round {centre:?} answers: check its volume as \
                     f179ca02a did"
                ),
            }
        }
    }
}

fn where_three_surfaces_meet_needs_two_of_them_flat(error: &Error) -> bool {
    matches!(
        error,
        Error::Geometry(rcad_geom::Error::WhereThreeSurfacesMeetNeedsTwoOfThemFlat)
    )
}

/// Refused `TwoCurvesTouchWithoutCrossing` on a lead's side, the plane
/// `x = -0.3` with one wire.
fn two_curves_touch_on_the_side_of_a_lead(model: &Model, error: &Error) -> bool {
    let Error::TwoCurvesTouchWithoutCrossing { face } = error else {
        return false;
    };
    let face = model.face(*face).expect("the face is the LED's own");
    let Surface::Plane(plane) = face.surface() else {
        return false;
    };
    let [x, y, z, d] = plane.coefficients();
    face.wire_count() == 1
        && [x + 1.0, y, z, d + 0.3]
            .iter()
            .all(|part| part.abs() < 1e-9)
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
