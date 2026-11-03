//! A small box cut out of a KiCad LED where the flat side of its flange
//! touches the round body, and where it touches the round disc on its front.
//!
//! `LED_D5.0mm_Horizontal_O1.27mm_Z3.0mm.step` is a pipe of radius 2.5 about
//! the line `x = 1.27, z = 3` running back along `-y` from `y = -2.27`, with a
//! flange in front of it from `y = -2.27` to `y = -1.27`: a disc of radius 2.9
//! cut flat at `x = -1.23`, which is 2.5 from the axis, so the flat touches the
//! pipe along the line `z = 3`. The file keeps two corners on that line, and
//! each holds two fans of faces (`docs/decisions.md` §471.1):
//!
//! - `#123` at `(-1.23, -2.27, 3)`, on the flange's back: the flange's fan, its
//!   flat and its back, and the body's fan, its pipe and the flange's back.
//! - `#74` at `(-1.23, -1.27, 3)`, on the flange's front, where a circle of
//!   radius 2.5 cut into the front face touches the flat.
//!
//! Every fuse of the LED tried by Q2911's lane refused `Undecidable` on where
//! three surfaces meet, at `#123`, handed the flat, the back and the pipe: two
//! planes and a pipe that touch rather than cross there (§471.3). Bake now
//! reads such a corner out of the model, and every fuse of the LED gets past
//! it and stops later, in the check that the LED does not cross itself.
//!
//! It stopped first on the upper half of the flange's rim, a pipe of radius
//! 2.9 that no box here touches, refused `AFaceTrimmedByACurveThisCannotFollow`.
//! The rim's edge along the flat is a `LINE` in the file, so nothing picks it,
//! and the flat's normal is `3.6e-32` off square to the rim's axis, so the two
//! surfaces cross in an ellipse about `1.6e32` long with that edge on it. The
//! arrangement now tells which of the ellipse's two arcs the edge is by the
//! rim's box, which the other arc leaves (Q2926).
//!
//! It stopped next at the disc of radius 2.5 on the flange's front, refused
//! `AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve`, from the rim against the disc:
//! the disc's arcs are drawn between two planes a rounding apart, and nothing
//! baked said they ran along the body's pipe. The check now skips a flat face
//! a pipe holds strictly inside, and the disc is 0.4 inside the rim (Q2947).
//!
//! It stopped next where the flange's flat meets the body's pipe and the
//! rim's, two pipes about one axis a rounding apart in direction that never
//! meet, refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`. Q2938's road
//! answers that triple.
//!
//! It stopped next on the flange's front, face slot 2, the plane `y = -1.27`,
//! refused `AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve` by
//! `cut_the_crossings_into_arcs` (fuse/arrange.rs) in the check that the LED
//! does not cross itself. The front drew the arcs round the disc as where its
//! plane crosses the disc's, two spellings of one plane, which is a straight
//! line and not the circle the arcs run along. The arrangement now draws such
//! an arc where the face's plane crosses the pipe it runs along (Q2966).
//!
//! It stopped next `Undecidable` on side of cylinder at `#74`, the front
//! corner: bake solved it from the front, the flat and the disc, three planes
//! two of which are a rounding apart, and the box it got ran along the flat,
//! `z` from `2.50` to `3.27`. Bake now reads a corner in two fans out of the
//! model and never solves it (Q2939). It then stopped on `order along a
//! crossing` between the front corner and a point `3.6e-15` across it, the
//! shape the `Z15.0` LED met below, and gets past it as that one does (Q2959).
//!
//! It stopped next `Undecidable` with the one record `order along a
//! crossing`, between the flat's corner on the rim at the back,
//! `(-1.23, -2.27, 1.53)`, and a point `2.6e31` back along `-y` where the
//! lead's side `x = -0.3` meets the loop the flat cuts the rim in. The flat's
//! normal is `3.6e-32` off `x`, so that loop is about `1.6e32` long, and it
//! reaches `x = -0.3` only that far out (Q2990).
//!
//! That point is where it says. The rim's axis is written `(0, 1, -1.1e-15)`,
//! so `2.6e31` along it the rim is `2.9e16` up, and the meet at
//! `(-0.3, -2.58e31, 2.86e16)` is on the rim, the flat and the lead's side as
//! they are written. Nor can it be left off the loop. Q2990's first lane left
//! out every meet off the face's box, grown by the room its corners were
//! asked with, and 18 tests of `rcad-topo` and `rcad-step` broke, all of a
//! cone or an LED's dome, most on `ResultIsNotManifold`. One is the
//! wall `x = 1` of a block beside a cone's apex
//! (`a_block_face_through_a_cones_apex.rs`): the drop left out where the
//! hyperbola it cuts in the cone meets its own edge lines `y = 6` and
//! `y = -6`, at `z = 12.17` and `-12.17`, off a wall that stops at `z = 10`.
//! A meet off the face is still where two drawn crossings cross, and the
//! cells are walked through it. So the loop has an order and the arithmetic
//! does not find it: a gap, not a limit.
//!
//! An `f64` near `2.86e16` is spaced 4, so the point's box is 40 wide in `z`,
//! and nothing read off it says which way the loop runs there. Which of the
//! two meets of the line `x = -0.3` cuts the flat in it is says that instead.
//! The loop is a closed curve on the flat, so `x = -0.3` has one highest and
//! one lowest point on it, and counted from a start below both, the node on
//! the half where the loop climbs across `x = -0.3` comes before the one on
//! the half where it falls. The arrangement now reads the order so, with the
//! plane of either node as the one climbed across (Q3019).
//!
//! The boxes round both corners stopped then `Undecidable` with the one
//! record `side of plane`, at the point where the plane of the box's floor
//! `z = 2.9` meets the flange's rim on its front, near `(-1.628, -1.27, 2.9)`:
//! the stop the `D3.0` LED met below, on its rim at the front (Q2995). A node
//! of a walk that cannot tell its side is now passed over, and the walk read
//! off a node that can (Q3009); a node solved from a pipe and two planes is
//! read as where the line the two planes share crosses the pipe (Q3017).
//! Either alone gets both boxes past that point.
//!
//! Every box, the far one included, stopped then on `order along a crossing`
//! between the front corner and a point within `1.8e-15` of it, the shape
//! Q2959 folds into the corner, not folded here (Q3015). The front draws the
//! circle it meets the body's pipe in twice, as the edge of its hole and as
//! the pipe cut across it, and the flat's line meets the two a rounding apart.
//! Which of two such points comes first is now read off where the line
//! crosses each pipe, worked out as exactly as `f64` holds it, with a bound on
//! every product too small to hold (Q3054).
//!
//! Every box, the far one included, stopped then on `order along a crossing`
//! on the straight line where the front's plane crosses the body's front
//! disc's, `(-3.6e-32, 1, -1.15e-15)` and `(5.6e-17, 1, 0)`, one plane written
//! twice. It was between the front corner, a corner of both faces, and where
//! the flat meets that line: solved from the flat, the front and the disc,
//! three planes two of which are a rounding apart, its box ran along the
//! flat's line from `z = 2.56` to `3.14`, round the corner. The arrangement
//! took that line through the front corner without asking, because the disc
//! is one of the surfaces round the corner. The face's own plane written a
//! second time is now always asked (Q3071).
//!
//! Every box, the far one included, stops now on `order along a crossing`
//! between two places on the body's pipe near the flat: one in a box at
//! `x = -1.22688`, `z = 3.12484`, running `0.16` along `y` from `-1.370` to
//! `-1.210`, across the front, and one on the front at
//! `(-1.22612, -1.27, 3.13918)`. The `D5` LED whose front is `y = -3.81` and
//! whose axis is 15 up, in `a_led_whose_front_is_two_planes_a_rounding_apart.rs`,
//! stops on the same two places, 12 higher. Untraced.
//!
//! These tests pin that. When it answers, each turns back
//! into the volume check of commit `f179ca02a`: what the box and the LED share
//! is the box's part inside the LED, and the intersection and the difference
//! add up to the LED.
//!
//! `LED_D5.0mm_Horizontal_O1.27mm_Z15.0mm.step` is the same LED with its axis
//! at `z = 15`, and `LED_D3.0mm_Horizontal_O3.81mm_Z6.0mm.step` a smaller one,
//! its body a pipe of radius 1.5 about `x = 1.27, z = 6` with its flange's
//! flat at `x = -0.23` and its front at `y = -3.81`. Every fuse of either that
//! Q2928's row tried refused `Undecidable` on where three surfaces meet, the
//! far box included, and it was neither at the back corner: it was at the
//! front one, `(-1.23, -1.27, 15)` and `(-0.23, -3.81, 5.999999956464)`. There
//! the flange's flat meets its front and the body's front disc, and the
//! front's plane and the disc's are one plane written down twice, a rounding
//! apart: `(0, 1, -1.15e-15)` and `(5.6e-17, 1, 0)`. So the corner has three
//! planes round it that share a line, solving it finds a box along that line
//! that misses the stored corner, and bake would not read a corner with three
//! planes round it out of the model. It now does, and asks every plane about
//! it as it asks a solved corner.
//!
//! Past that corner, the `Z15.0` LED stopped at its body's pipe, refused
//! `AFaceTrimmedByACurveThisCannotFollow`: the fuse baked each arc where its
//! dome meets the pipe against the ball across it, and two curved surfaces
//! that only touch cannot say which way along the rim an arc runs. Bake now
//! reads such an arc against the rim's plane (Q2940), and the `Z15.0` LED gets
//! past its body's pipe. The `D3.0` LED stopped at its front disc as the
//! `Z3.0` one did, and gets past it as that one does.
//!
//! Both stopped next where three surfaces meet, refused `Undecidable` on the
//! flange's flat and two curved surfaces about one point: `x = -1.23` and two
//! about `(1.27, -1.27, 15)` in the `Z15.0` LED, its flat and two about
//! `(1.27, -3.81, 6)` in the `D3.0` one. The two curved surfaces are the
//! body's pipe and the rim's, about one axis a rounding apart in direction,
//! the pair Q2938 names. The flat's normal there is exactly `(-1, 0, 0)`, and
//! the road along a pipe ran out of range writing the lines the flat cuts. It
//! now hands the triple on to Q2938's road for a plane that cuts both in
//! conics, which answers it as it answers the `Z3.0` LED's flat that leans
//! off `x` by a rounding.
//!
//! Both stopped next at the front corner, refused `Undecidable` with the one
//! record `order along a crossing`, the far box included: in the `Z15.0` LED
//! between the corner `(-1.23, -1.27, 15)` and a point solved near
//! `(-1.23, -1.27, 15.075)` in a box `4.4e-16` wide, in the `D3.0` one between
//! `(-0.23, -3.81, 5.999999956464)` and a point near `z = 6.147`. Those points
//! were on the straight line where the front's plane crosses the disc's, which
//! the front drew its arcs round the disc as until Q2966.
//!
//! The `Z15.0` LED stopped next on `order along a crossing` between two points
//! both within `1e-14` of the front corner, enclosure `1.8e-14`, in the check
//! that the LED does not cross itself. They were the two places where the
//! flat's line on the front meets the circle the body's pipe cuts in it, and
//! the line touches that circle at the corner. Their boxes share a point and
//! both lie inside the room writing the corner down cost, so nothing proves
//! them two points, and the corner now stands for both (Q2959).
//!
//! Both LEDs stopped next on the flange's front, face slot 8 in the `Z15.0`
//! LED and slot 2 in the `D3.0` one, refused `CannotTellTheCellsOfAFaceApart`
//! in the same check, where the windings of the front's cells are spread. In
//! the `D3.0` LED the front is the disc of radius 1.9 cut flat at `x = -0.23`,
//! with a hole of radius 1.5 that touches the flat at the front corner. The
//! flat's line misses the hole's circle by `2.8e-17`, so the arrangement knows
//! the two meet nowhere, yet both run through the corner, which the file
//! stores `4.35e-8` below where they would touch. Read off that point, the
//! circle crossed the flat there, so the four ways out of the corner sorted
//! wrong and the walks round the front's cells merged three of them into one.
//! Two crossings that meet nowhere now turn by nothing at a node they both run
//! through, so how they bend sorts them (Q2974).
//!
//! In the `Z15.0` LED the front is the disc of radius 2.9 cut flat at
//! `x = -1.23`, with a hole of radius 2.5 whose circle the flat's line touches
//! at the front corner, and the corner stands for both places the two meet
//! (Q2959). Read off the corner, the circle crossed the flat there too: the
//! four ways out sorted flat, circle, flat, circle, the walks made a cell of
//! 16 half-edges, and a winding spread out of range. Two crossings whose every
//! meet is at one node now turn by nothing there, so how they bend sorts them
//! as well (Q2982).
//!
//! The `D3.0` LED stopped next `Undecidable` with the one record `order along
//! a crossing`, the far box included, between two points a rounding apart in
//! `y` where the flange's rim crosses the front at `z = 5.7`, near
//! `(3.146, -3.81, 5.7)`. One is on the flange's front, normal
//! `(0, 1, -1.158e-15)` at offset `-3.810000000000007`, and one on the body's
//! front disc, normal `(5.55e-17, 1, 0)` at offset `-3.81`: one plane written
//! twice. Which side of the first's plane the second is on is now read
//! exactly, along the line the second lies on, and so is the order (Q2988).
//!
//! The `Z15.0` LED stopped next `Undecidable` with the one record `order along
//! a crossing`, the far box included, between the two points where the lead's
//! side `x = -0.3` meets the dome's rim `y = -7.37`, `3.9` apart in `z`. The
//! dome is the ball of radius 2.5 about `(1.27, -7.37, 15)`, `x = -0.3` cuts
//! it in a circle walked from its lowest point, and the rim's plane, through
//! the ball's centre, crosses that circle right there. A loop is counted from
//! the node at its start for a plane and a ball now, as for a plane and a pipe
//! (Q2989).
//!
//! The `D3.0` LED stopped next `Undecidable` with the one record `side of
//! plane`, the far box included, at a point on the flange's rim at the front:
//! near `(-0.627, -3.81, 5.9)` for the boxes round the front corner, and
//! `(-0.606, -3.81, 6.3)` for the far one. Each was a node of a walk on the
//! rim, on the flange's front as it writes it, asked which side of the body's
//! front disc, the same plane written again, the walk is on. Every node of a
//! walk is on the walk's side of a crossing it does not run along, or on the
//! crossing, so a node that cannot tell is now passed over and the walk read
//! off a node that can (Q3009). Reading a node solved from a pipe and two
//! planes as where the line the two planes share crosses the pipe gets it past
//! too, on its own (Q3017). It stopped then `Undecidable` with the one record
//! `order along a crossing`, the far box included, between two points a
//! rounding apart on the front where `x = 2.24` crosses the body's pipe, near
//! `(2.24, -3.81, 4.856)`: one where the front's hole meets it, and one where
//! the body's pipe, as a cut, does (Q3014). The two are read as the `Z3.0`
//! LED's are (Q3054), and it stopped where that one did: on the line the
//! front's plane crosses the disc's in, between its front corner and where
//! its flat meets that line, near `(-0.23, -3.81, 6.1466)`. It gets past that
//! as the `Z3.0` LED does (Q3071), and stops in two places now, both
//! untraced:
//!
//! - Its box round the front corner and the far box are refused
//!   `TwoCurvesTouchWithoutCrossing` on face slot 18, the side `x = -0.3` of a
//!   lead with one wire: what the far boxes of two LEDs in
//!   `a_led_whose_front_is_two_planes_a_rounding_apart.rs` are refused.
//! - Its box round the back corner `(-0.23, -4.81, 6)` stops `Undecidable`
//!   with the one record `two ways out of a node along one line`, at that
//!   corner. One way out runs along `z`, `(0, 1.16e-15, 1)`, and the other is
//!   a box round nothing, `2.2e-16` either side of zero in `z`: the shape the
//!   `Z15.0` LED met at its back corner below (Q3043).
//!
//! `LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step`,
//! `LED_D3.0mm_Horizontal_O1.27mm_Z6.0mm.step` and
//! `LED_D3.0mm_Horizontal_O3.81mm_Z10.0mm.step` are three more `D3.0` LEDs,
//! their flats at `x = -0.23`, their fronts at `y = -1.27` or `-3.81` and
//! their axes 10 or 6 up, each with its front corner stored `4.35e-8` below
//! the axis. Their boxes round the back corner stop on two ways out of it
//! along one line, the same shape at the same place. Their boxes round the
//! front corner and their far boxes are refused
//! `Invalid(WireTooShort { count: 2 })`, a piece of a face bounded by two
//! straight sides, as the far box of `LED_D3.0mm_Horizontal_O3.81mm_Z2.0mm.step`
//! is in `a_led_whose_front_is_two_planes_a_rounding_apart.rs`. Untraced, both
//! (Q2978).
//!
//! The `Z15.0` LED stopped on its body's pipe, `Undecidable` with the one
//! record `where three surfaces meet`, the far box included, handed the
//! flange's flat `x = -1.23`, the dome and the body's pipe. The flat touches the dome at one point, `(-1.23, -7.37, 15)`
//! where the dome meets the pipe, and as stored that point is `3.4e-16` inside
//! the pipe, so the three meet nowhere. The touch came back as a box of no
//! width, and asked as a box the pipe could not tell; it is now asked as the
//! point the box names (Q3020).
//!
//! The `Z15.0` LED stopped then on its body's pipe, `Undecidable` with the one
//! record `two ways out of a node along one line`, the far box included, at
//! the back corner `(-1.23, -2.27, 15)`. One way out of it runs along `z`, on
//! the circle the flange's back cuts in the pipe. The other is along the flat,
//! which runs along the pipe there: the pipe's gradient is the flat's normal
//! but for a rounding, so that way out is a box round nothing, `2.2e-16` either
//! side of zero in `z`, and whether the two go the same way along one line was
//! not decided. The corner stands for the two places where the circle crosses
//! the ellipse the flat cuts in the pipe, at right angles, `1.67e-8` either
//! side of `z = 15`, and the four ways out are now put in order by which of
//! the two each leaves from (Q3043).
//!
//! The `Z15.0` LED stopped then on its flange's flat, `Undecidable` with the
//! one record `order along a crossing`, the far box included, between its front
//! corner and its back corner round the ellipse the flat cuts in the body's
//! pipe. The pipe's axis is written `5.6e-17` off square to the flat's normal,
//! so that ellipse is about `9e16` long, it ends at the front corner, and the
//! back corner, one along it, stands for a place on each of its two halves.
//! The ellipse now passes the back corner twice, once either side of the
//! nodes past the back plane (Q3064).
//!
//! The `Z15.0` LED stops now on the same flat, `Undecidable` with the one
//! record `order along a crossing`, the far box included, between two meets
//! of the ellipse at its end, each within `1e-14` of the front corner. The
//! front is written as two planes, the flange's and the body's front disc's,
//! and the ellipse turns there on a radius of about `1e-16`, so each plane
//! meets it in two places that no box tells apart. The tests pin both LEDs'
//! stops.

use std::{env, fs, path::PathBuf};

use rcad_geom::{Isometry, Surface, Vector};
use rcad_predicates::{Point, Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Error, Invalid, Model, ProfileBuilder, Rule, SolidKey};

const Z3: &str = "LED_D5.0mm_Horizontal_O1.27mm_Z3.0mm.step";
const Z15: &str = "LED_D5.0mm_Horizontal_O1.27mm_Z15.0mm.step";
const D3: &str = "LED_D3.0mm_Horizontal_O3.81mm_Z6.0mm.step";

/// Three more `D3.0` LEDs, each with the `y` of its front and the `z` of its
/// body's axis.
const THREE_MORE_D3: [(&str, f64, f64); 3] = [
    ("LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step", -1.27, 10.0),
    ("LED_D3.0mm_Horizontal_O1.27mm_Z6.0mm.step", -1.27, 6.0),
    ("LED_D3.0mm_Horizontal_O3.81mm_Z10.0mm.step", -3.81, 10.0),
];

/// How far below its body's axis a `D3.0` LED stores its front corner.
const BELOW_THE_AXIS: f64 = 4.3536e-8;

const ALONG_A_CROSSING: &str = "order along a crossing";

const THE_SAME_WAY_ALONG_ONE_LINE: &str = "two ways out of a node along one line";

/// Half the side of the box put round each corner.
const HALF: f64 = 0.1;

/// Where the box that misses the LED goes.
const FAR_OFF: [f64; 3] = [10.0, 10.0, 10.0];

/// The `Z3.0` LED's front corner.
const Z3_FRONT_CORNER: [f64; 3] = [-1.23, -1.27, 3.0];

/// The `Z15.0` LED's front corner.
const Z15_FRONT_CORNER: [f64; 3] = [-1.23, -1.27, 15.0];

/// The `Z15.0` LED's back corner.
const Z15_BACK_CORNER: [f64; 3] = [-1.23, -2.27, 15.0];

/// The `Z3.0` LED gets past where its flat meets the line its front's plane
/// crosses its disc's in since Q3071: that line is no longer taken through the
/// front corner unasked. It stops on two places on its body's pipe near the
/// flat, one in a box `0.16` long along `y` across the front, the boxes round
/// either corner and the far box alike.
#[test]
fn a_d5_led_3_up_gets_past_where_its_flat_meets_its_front_written_twice_and_stops_on_two_places_on_its_body()
 {
    let near_the_flat = [
        [-1.2268808471946124, -1.2268808471946089],
        [-1.3697506234414247, -1.210149625935145],
        [3.124844042359752, 3.124844042359754],
    ];
    let on_the_front = [
        [-1.2261227327833313, -1.2261227327833297],
        [-1.2700000000000005, -1.2699999999999996],
        [3.139180827998248, 3.1391808279982496],
    ];
    every_fuse_refuses(
        Z3,
        &a_box_round_each_of(&[Z3_FRONT_CORNER, [-1.23, -2.27, 3.0], FAR_OFF]),
        |_, error| two_places_along_a_crossing_are_not_ordered(error, near_the_flat, on_the_front),
    );
}

/// The `Z15.0` LED gets past its body's pipe since Q2940, and past where its
/// flange's flat `x = -1.23` meets two curved surfaces about
/// `(1.27, -1.27, 15)` since Q2938's road answers a flat whose normal is
/// exactly `(-1, 0, 0)`. Since Q2966 it gets past the point solved near
/// `(-1.23, -1.27, 15.075)` on the straight line the front drew its arcs round
/// the disc as. Since Q2959 the two places where the flat's line on the front
/// meets the disc's circle, both within `1e-14` of the front corner and not
/// proved apart, are the corner. Since Q2982 the flat's line and the circle
/// turn by nothing at that corner, so it tells the cells of its front apart.
/// Since Q2989 it puts in order the two points where `x = -0.3` meets the
/// dome's rim at `y = -7.37`. Since Q3020 the one point its flange's flat
/// touches its dome at is asked as that point, which is inside the body's
/// pipe, so the flat, the dome and the pipe meet nowhere. Since Q3043 the four
/// ways out of its back corner on the body's pipe are put in order by the two
/// places the corner stands for. Since Q3064 the ellipse its flange's flat cuts
/// in the body's pipe passes that corner twice, once either side of the front
/// corner. It stops on that flat, on the order round the ellipse of two meets
/// at its end by the front corner, one with each of the two planes the front is
/// written as, the far box included.
#[test]
fn a_z15_led_gets_past_its_corners_round_the_flats_ellipse_and_stops_on_two_meets_at_its_end() {
    every_fuse_refuses(
        Z15,
        &a_box_round_each_of(&[Z15_FRONT_CORNER, Z15_BACK_CORNER, FAR_OFF]),
        |_, error| two_meets_at_the_end_of_the_flats_ellipse_are_not_ordered(error),
    );
}

/// The `D3.0` LED tells the cells of its front apart since Q2974. Since Q2988
/// it puts in order the two points where `z = 5.7` crosses its flange's rim
/// at the front, one on the flange's front and one on the body's front disc,
/// the one plane written twice. Since Q3009 a walk on the rim whose node
/// cannot tell its side of the disc is read off another node. Since Q3054 it
/// puts in order the two points where `x = 2.24` crosses the body's pipe on
/// the front. Since Q3071 it gets past where its flat meets the line its
/// front's plane crosses its disc's in. Its box round the front corner and its
/// far box stop where two curves touch on the side `x = -0.3` of a lead.
#[test]
fn a_d3_led_6_up_gets_past_where_its_flat_meets_its_front_written_twice_and_stops_where_two_curves_touch_on_the_side_of_a_lead()
 {
    every_fuse_refuses(
        D3,
        &a_box_round_each_of(&[[-0.23, -3.81, 6.0 - BELOW_THE_AXIS], FAR_OFF]),
        two_curves_touch_on_the_side_of_a_lead,
    );
}

/// Past where their flats meet the line their front's plane crosses their
/// disc's in (Q3071), the boxes round the back corner of four `D3.0` LEDs stop
/// on two ways out of that corner along one line, one of them a box round
/// nothing.
#[test]
fn four_d3_leds_get_past_their_front_written_twice_and_stop_on_two_ways_out_of_their_back_corner_along_one_line()
 {
    let all_four = [(D3, -3.81, 6.0)].into_iter().chain(THREE_MORE_D3);
    for (file, front, axis) in all_four {
        let back_corner = [-0.23, front - 1.0, axis];
        every_fuse_refuses(file, &a_box_round_each_of(&[back_corner]), |_, error| {
            two_ways_out_of_a_corner_are_not_ordered(back_corner, error)
        });
    }
}

/// Past where their flats meet the line their front's plane crosses their
/// disc's in (Q3071), the boxes round the front corner and the far boxes of
/// three more `D3.0` LEDs are refused a piece of a face bounded by two
/// straight sides (Q2978).
#[test]
fn three_more_d3_leds_get_past_their_front_written_twice_and_are_refused_a_piece_of_two_straight_sides()
 {
    for (file, front, axis) in THREE_MORE_D3 {
        let front_corner = [-0.23, front, axis - BELOW_THE_AXIS];
        every_fuse_refuses(
            file,
            &a_box_round_each_of(&[front_corner, FAR_OFF]),
            |_, error| matches!(error, Error::Invalid(Invalid::WireTooShort { count: 2 })),
        );
    }
}

/// A box round each corner, to be fused with the LED under intersection and
/// under difference.
fn a_box_round_each_of(corners: &[[f64; 3]]) -> Vec<([f64; 3], Rule)> {
    corners
        .iter()
        .flat_map(|&corner| [Rule::Intersection, Rule::Difference].map(|rule| (corner, rule)))
        .collect()
}

/// The LED in `file` and a box of side `2 * HALF` centred on each corner of
/// `fuses`, fused under its rule, each refused as `the_refusal` takes, handed
/// the model it was refused in.
fn every_fuse_refuses(
    file: &str,
    fuses: &[([f64; 3], Rule)],
    the_refusal: impl Fn(&Model, &Error) -> bool,
) {
    let (led, solid) = the_led(file);
    for &(corner, rule) in fuses {
        let mut model = led.clone();
        let knife = a_box_round(&mut model, corner);
        match model.fuse(
            &[solid],
            &[knife],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        ) {
            Err(error) => assert!(
                the_refusal(&model, &error),
                "{file}: {rule:?} with a box round {corner:?} refused {error:?}, which is not \
                 the refusal this pins"
            ),
            Ok(_) => panic!(
                "{file}: {rule:?} with a box round {corner:?} answers: check its volume as \
                 f179ca02a did"
            ),
        }
    }
}

/// Refused `Undecidable` with the one record `order along a crossing`, handed
/// two points whose boxes end within `1e-9` of `one` and `other`, either way
/// round.
fn two_places_along_a_crossing_are_not_ordered(
    error: &Error,
    one: [[f64; 2]; 3],
    other: [[f64; 2]; 3],
) -> bool {
    let [first, second] = the_points_of_the_one_record(ALONG_A_CROSSING, error) else {
        return false;
    };
    (is_within_1e_9_of_the_box(first, one) && is_within_1e_9_of_the_box(second, other))
        || (is_within_1e_9_of_the_box(first, other) && is_within_1e_9_of_the_box(second, one))
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

/// Refused `Undecidable` with the one record `two ways out of a node along one
/// line`, handed `corner` and two ways out of it, one of which is a box round
/// nothing: each of its sides holds zero and is less than `1e-15` wide.
fn two_ways_out_of_a_corner_are_not_ordered(corner: [f64; 3], error: &Error) -> bool {
    let [at, one, other] = the_points_of_the_one_record(THE_SAME_WAY_ALONG_ONE_LINE, error) else {
        return false;
    };
    let round_nothing = |way: &Point| {
        way.enclosure()
            .iter()
            .all(|side| side.lo() <= 0.0 && 0.0 <= side.hi() && side.hi() - side.lo() < 1e-15)
    };
    is_within_1e_9_of(at, corner) && (round_nothing(one) || round_nothing(other))
}

/// The points of the one record of an `Undecidable` refusal, when it is one
/// on `predicate`, and none otherwise.
fn the_points_of_the_one_record<'a>(predicate: &str, error: &'a Error) -> &'a [Point] {
    let Error::Undecidable { records } = error else {
        return &[];
    };
    match records.records() {
        [record] if record.predicate() == predicate => record.inputs(),
        _ => &[],
    }
}

/// Refused `Undecidable` with the one record `order along a crossing`, handed
/// two points within `1e-9` of the `Z15.0` LED's front corner, one whose box
/// along `y` ends at the corner's `y` from in front and one whose box ends
/// there from behind, either way round: the meet of the flat, the pipe and the
/// body's front disc's plane, and the meet of the flat, the pipe and the
/// flange's front.
fn two_meets_at_the_end_of_the_flats_ellipse_are_not_ordered(error: &Error) -> bool {
    let Error::Undecidable { records } = error else {
        return false;
    };
    let [record] = records.records() else {
        return false;
    };
    let [one, other] = record.inputs() else {
        return false;
    };
    let [_, y, _] = Z15_FRONT_CORNER;
    let in_front = |point: &Point| point.enclosure()[1].lo() == y;
    let behind = |point: &Point| point.enclosure()[1].hi() == y;
    record.predicate() == ALONG_A_CROSSING
        && is_within_1e_9_of(one, Z15_FRONT_CORNER)
        && is_within_1e_9_of(other, Z15_FRONT_CORNER)
        && ((in_front(one) && behind(other)) || (behind(one) && in_front(other)))
}

fn is_within_1e_9_of(point: &Point, at: [f64; 3]) -> bool {
    is_within_1e_9_of_the_box(point, at.map(|at| [at, at]))
}

/// Whether each side of `point`'s box ends within `1e-9` of where the same
/// side of `sides` ends, `[lo, hi]` along each axis.
fn is_within_1e_9_of_the_box(point: &Point, sides: [[f64; 2]; 3]) -> bool {
    point
        .enclosure()
        .iter()
        .zip(sides)
        .all(|(side, [lo, hi])| (side.lo() - lo).abs() < 1e-9 && (side.hi() - hi).abs() < 1e-9)
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

fn a_box_round(model: &mut Model, corner: [f64; 3]) -> SolidKey {
    let side = 2.0 * HALF;
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let low = Vector::new(corner[0] - HALF, corner[1] - HALF, corner[2] - HALF);
    model
        .add_extrusion(
            &square,
            side,
            &Isometry::translation(low).expect("a finite shift"),
        )
        .expect("a box")
}
