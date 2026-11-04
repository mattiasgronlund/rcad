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
//! Every box, the far one included, stopped then on `order along a crossing`
//! between two places on the body's pipe near the flat: one in a box at
//! `x = -1.22688`, `z = 3.12484`, running `0.16` along `y` from `-1.370` to
//! `-1.210`, across the front, and one on the front at
//! `(-1.22612, -1.27, 3.13918)`. The first is one of four places where the
//! body's front disc meets the body's pipe written twice, as the face and as
//! the pipe the disc's edge runs along. The disc is a rounding off square to
//! that pipe, and its box was worked out about the pipe's axis, which leaves
//! what rounds across the disc. Each such box is now narrowed onto the plane
//! it lies on (Q3111). The `D5` LED whose front is `y = -3.81` and whose axis
//! is 15 up, in `a_led_whose_front_is_two_planes_a_rounding_apart.rs`, stopped
//! on the same two places, 12 higher, and gets past them too.
//!
//! Every box, the far one included, stopped then refused
//! `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`, on the flange's flat
//! `(-1, -3.6e-32, 4.6e-47)` at `1.23`, the dome, a ball of radius 2.5 about
//! `(1.27, -7.37, 3)`, and the body's pipe along `(-5.55e-17, -1, 0)`. The
//! flat leans a rounding off `x = -1.23`, as the flats of three LEDs in
//! `a_led_whose_front_is_two_planes_a_rounding_apart.rs` do, and the check
//! that a quartic keeps one sign over a box spread too wide to answer it. It
//! is now also run on the quartic moved to the low end of the box (Q3113).
//!
//! The box round the back corner now shares with the LED what commit
//! `f179ca02a` works out, the far box shares nothing with it, and the rest
//! stop refused `DynamicRangeTooWide`: the box round the front corner at a
//! ratio of `1.4e16` under intersection, and every difference at `3.5e17`.
//! Untraced. These tests pin that.
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
//! - Its box round the front corner and the far box were refused
//!   `TwoCurvesTouchWithoutCrossing` on face slot 18, the side `x = -0.3` of a
//!   lead with one wire, whose edge along the line `x = -0.3, y = -0.3` is
//!   written twice, below the lead's bend on the plane `y = -0.3` and above it
//!   on the plane of the bend's slanted leg. Two crossings of a face's own
//!   edges that are one straight line are now folded into one, each side
//!   keeping the name of the face across it (Q3112). The far box then answers
//!   under intersection, with nothing, and its difference is refused
//!   `DynamicRangeTooWide`, at a ratio of `2.0e16`. The box round the front
//!   corner is refused under either rule `Undecidable` with the one record `a
//!   point inside a piece of a face`, for a sliver of the box's top
//!   `z = 6.1 - 4.35e-8` between the flat `x = -0.23` and the body's pipe at
//!   `x = -0.22666`, and between the flange's front and the body's front disc,
//!   the one plane `y = -3.81` written twice a rounding apart. Untraced, both.
//! - Its box round the back corner `(-0.23, -4.81, 6)` stopped `Undecidable`
//!   with the one record `two ways out of a node along one line`, at that
//!   corner. One way out runs along `z`, `(0, 1.16e-15, 1)`, and the other is
//!   a box round nothing, `2.2e-16` either side of zero in `z`: the shape the
//!   `Z15.0` LED met at its back corner below (Q3043). It gets past it as that
//!   one does (Q3142).
//!
//! `LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step`,
//! `LED_D3.0mm_Horizontal_O1.27mm_Z6.0mm.step` and
//! `LED_D3.0mm_Horizontal_O3.81mm_Z10.0mm.step` are three more `D3.0` LEDs,
//! their flats at `x = -0.23`, their fronts at `y = -1.27` or `-3.81` and
//! their axes 10 or 6 up, each with its front corner stored `4.35e-8` below
//! the axis. Their boxes round the back corner stopped on two ways out of it
//! along one line, the same shape at the same place, until Q3142. Their boxes
//! round either corner and their far boxes are refused
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
//! The `Z15.0` LED stopped then on the same flat, `Undecidable` with the one
//! record `order along a crossing`, the far box included, between two meets
//! of the ellipse at its end, each within `1e-14` of the front corner. The
//! front is written as two planes, the flange's and the body's front disc's,
//! and the ellipse turns there on a radius of about `1e-16`, so each plane
//! meets it in two places that no box tells apart. The front corner was not
//! on the ellipse, so the fold of Q2959 did not look at it; it now stands for
//! all four (Q3110).
//!
//! The `Z15.0` LED stopped then on the same flat: the boxes round its corners
//! `TwoCurvesTouchWithoutCrossing`, and the far box `Undecidable` with the one
//! record `two ways out of a node along one line` at the back corner, one way
//! out along the back plane and one a box round nothing, as Q3043 was on the
//! body's pipe. Both were on the ellipse the flat cuts in the body's pipe,
//! drawn on the flat as a cut. The flat lies in front of the back plane
//! `y = -2.27` and the pipe's face behind it, so that ellipse is no edge of
//! the answer on the flat. The arrangement leaves out a crossing of two faces
//! that lie in the two closed sides of one plane (Q3048), but bake read the
//! flat as lying in front of the back plane only if every edge of it stays
//! there, and two of them run across the flange's rim. The
//! flat's normal `(-1, 0, 0)` is exactly square to the rim's axis
//! `(0, 1, -1.1e-15)`, so they are straight, and they were read as arcs,
//! which proved nothing. An edge across a pipe whose axis the face's plane
//! holds now stays wherever both its ends do (Q3142).
//!
//! The box round the `Z15.0` LED's back corner now shares with it what the box
//! round the `Z3.0` LED's does, and the far box shares nothing. The box round
//! its front corner is refused `ResultIsNotManifold` under intersection, and
//! every difference `DynamicRangeTooWide` at a ratio of `4.0e16`, both where
//! Q3136 traced the front written twice (Q3141). The `D3.0` LEDs get past
//! their back corners by the same reading. The box round the back corner of
//! the one whose front is `y = -3.81` and axis 6 up shares with it what the
//! `D5.0` LEDs' formula gives for a body of radius 1.5, and their difference
//! is refused `TwoEndsOfAnEdgeOnACurvedSurfaceRoundToOnePoint` on the
//! flange's rim at the front, where the box's top `z = 6.1` meets the front
//! written twice (Q3126). The other three are refused a piece of two straight
//! sides there. The tests pin every LED's stops.

use std::{env, fs, path::PathBuf};

use rcad_geom::{Deflection, Isometry, Vector};
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

/// Half the side of the box put round each corner.
const HALF: f64 = 0.1;

/// Where the box that misses the LED goes.
const FAR_OFF: [f64; 3] = [10.0, 10.0, 10.0];

/// The `Z3.0` LED's front corner.
const Z3_FRONT_CORNER: [f64; 3] = [-1.23, -1.27, 3.0];

/// The `Z3.0` LED's back corner.
const Z3_BACK_CORNER: [f64; 3] = [-1.23, -2.27, 3.0];

/// The `Z15.0` LED's front corner.
const Z15_FRONT_CORNER: [f64; 3] = [-1.23, -1.27, 15.0];

/// The `Z15.0` LED's back corner.
const Z15_BACK_CORNER: [f64; 3] = [-1.23, -2.27, 15.0];

/// The `Z3.0` LED gets past where its flat meets the line its front's plane
/// crosses its disc's in since Q3071: that line is no longer taken through the
/// front corner unasked. Since Q3111 it puts in order the places where its
/// body's front disc meets its body's pipe written twice, each boxed on the
/// disc. Since Q3113 it gets past where its flange's flat, its dome and its
/// body's pipe meet. The box round its back corner shares with it the box's
/// part inside it, as commit `f179ca02a` works that out, and the far box
/// shares nothing. The box round its front corner, and every difference, are
/// refused for their range.
#[test]
fn a_d5_led_3_up_gets_past_where_its_flat_dome_and_pipe_meet_and_shares_its_back_corner_with_a_box()
{
    a_box_round_shares(
        Z3,
        Z3_BACK_CORNER,
        what_a_box_round_the_back_corner_shares(2.5),
    );
    the_far_box_shares_nothing_with(Z3);
    every_fuse_refuses(Z3, &[(Z3_FRONT_CORNER, Rule::Intersection)], |error| {
        matches!(
            error,
            Error::Invalid(Invalid::DynamicRangeTooWide { ratio }) if (1.39e16..1.40e16).contains(ratio)
        )
    });
    let differences =
        [Z3_FRONT_CORNER, Z3_BACK_CORNER, FAR_OFF].map(|corner| (corner, Rule::Difference));
    every_fuse_refuses(Z3, &differences, |error| {
        matches!(
            error,
            Error::Invalid(Invalid::DynamicRangeTooWide { ratio }) if (3.5e17..3.6e17).contains(ratio)
        )
    });
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
/// corner. Since Q3110 the front corner stands for the four meets at the
/// ellipse's end, two with each of the two planes the front is written as.
/// Since Q3142 that ellipse is no edge of its flange's flat, which lies in
/// front of the back plane by its two straight edges across the flange's rim.
/// The box round its back corner shares with it what the box round the `Z3.0`
/// LED's does, and the far box shares nothing. The box round its front corner
/// is refused as not manifold, and every difference for its range.
#[test]
fn a_z15_led_leaves_its_bodys_ellipse_off_its_flat_and_shares_its_back_corner_with_a_box() {
    a_box_round_shares(
        Z15,
        Z15_BACK_CORNER,
        what_a_box_round_the_back_corner_shares(2.5),
    );
    the_far_box_shares_nothing_with(Z15);
    every_fuse_refuses(Z15, &[(Z15_FRONT_CORNER, Rule::Intersection)], |error| {
        matches!(error, Error::ResultIsNotManifold { count: 1 })
    });
    let differences =
        [Z15_FRONT_CORNER, Z15_BACK_CORNER, FAR_OFF].map(|corner| (corner, Rule::Difference));
    every_fuse_refuses(Z15, &differences, |error| {
        matches!(
            error,
            Error::Invalid(Invalid::DynamicRangeTooWide { ratio }) if (4.01e16..4.02e16).contains(ratio)
        )
    });
}

/// The `D3.0` LED tells the cells of its front apart since Q2974. Since Q2988
/// it puts in order the two points where `z = 5.7` crosses its flange's rim
/// at the front, one on the flange's front and one on the body's front disc,
/// the one plane written twice. Since Q3009 a walk on the rim whose node
/// cannot tell its side of the disc is read off another node. Since Q3054 it
/// puts in order the two points where `x = 2.24` crosses the body's pipe on
/// the front. Since Q3071 it gets past where its flat meets the line its
/// front's plane crosses its disc's in. Since Q3112 the two writings of the
/// edge along each lead's side are one line. Its far box shares nothing with
/// it, and the difference is refused for its range. Its box round the front
/// corner stops on a sliver of the box's top between the front's two writings.
#[test]
fn a_d3_led_6_up_gets_past_the_sides_of_its_leads_and_stops_on_a_sliver_of_its_box_or_for_its_range()
 {
    let front_corner = [-0.23, -3.81, 6.0 - BELOW_THE_AXIS];
    every_fuse_refuses(D3, &a_box_round_each_of(&[front_corner]), |error| {
        a_point_inside_a_sliver_of_the_box_top_is_not_placed(front_corner[2] + HALF, error)
    });
    the_far_box_shares_nothing_with(D3);
    every_fuse_refuses(D3, &[(FAR_OFF, Rule::Difference)], |error| {
        matches!(
            error,
            Error::Invalid(Invalid::DynamicRangeTooWide { ratio }) if (2.0e16..2.1e16).contains(ratio)
        )
    });
}

/// Past where its flat meets the line its front's plane crosses its disc's in
/// (Q3071), the box round the back corner of the `D3.0` LED 6 up stopped on two
/// ways out of that corner along one line, one of them a box round nothing.
/// Since Q3142 the ellipse its flange's flat cuts in its body's pipe is no edge
/// of that flat, which lies in front of the back plane by its two straight
/// edges across the flange's rim. The box shares with it what the box round a
/// `D5.0` LED's back corner does, for a body of radius 1.5, and their
/// difference is refused where the box's top meets the flange's rim at the
/// front written twice (Q3126).
#[test]
fn a_d3_led_6_up_leaves_its_bodys_ellipse_off_its_flat_and_shares_its_back_corner_with_a_box() {
    let back_corner = [-0.23, -4.81, 6.0];
    a_box_round_shares(
        D3,
        back_corner,
        what_a_box_round_the_back_corner_shares(1.5),
    );
    every_fuse_refuses(D3, &[(back_corner, Rule::Difference)], |error| {
        two_ends_round_to_one_point_where_the_box_top_meets_the_rim_at_the_front(
            back_corner[2] + HALF,
            error,
        )
    });
}

/// Past where their flats meet the line their front's plane crosses their
/// disc's in (Q3071), the boxes round the front corner and the far boxes of
/// three more `D3.0` LEDs are refused a piece of a face bounded by two
/// straight sides (Q2978). Their boxes round the back corner stopped on two
/// ways out of it along one line, and since Q3142 they are refused that piece
/// too.
#[test]
fn three_more_d3_leds_get_past_their_front_written_twice_and_are_refused_a_piece_of_two_straight_sides()
 {
    for (file, front, axis) in THREE_MORE_D3 {
        let front_corner = [-0.23, front, axis - BELOW_THE_AXIS];
        let back_corner = [-0.23, front - 1.0, axis];
        every_fuse_refuses(
            file,
            &a_box_round_each_of(&[front_corner, back_corner, FAR_OFF]),
            |error| matches!(error, Error::Invalid(Invalid::WireTooShort { count: 2 })),
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
/// `fuses`, fused under its rule, each refused as `the_refusal` takes.
fn every_fuse_refuses(
    file: &str,
    fuses: &[([f64; 3], Rule)],
    the_refusal: impl Fn(&Error) -> bool,
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
                the_refusal(&error),
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

/// The LED in `file` and a box of side `2 * HALF` centred on `corner`, fused
/// under intersection, answer with a model `validate` accepts, and what they
/// share is `shared` to within what measuring costs.
fn a_box_round_shares(file: &str, corner: [f64; 3], shared: f64) {
    let (mut model, solid) = the_led(file);
    let knife = a_box_round(&mut model, corner);
    let fused = model
        .fuse(
            &[solid],
            &[knife],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{file}: the box round {corner:?} refused: {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| {
            panic!("{file}: the box round {corner:?} left a model that is not valid: {why:?}")
        });
    let (common, common_error) = volume_of(&model, fused.solids());
    assert!(
        (common - shared).abs() <= common_error + 1e-12,
        "{file}: the LED and the box round {corner:?} share {common} ± {common_error}, and \
         {shared} is right"
    );
}

/// What a box of side `2 * HALF` round the back corner of an LED whose body's
/// pipe has `radius` shares with it: in front of the back plane, the quarter
/// of the box past the flat, in the flange; behind it, the part of the box's
/// quarter past the flat that is inside the body's pipe, which the flat
/// touches. Radius 2.5 is a `D5.0` LED's, and 1.5 a `D3.0` LED's.
fn what_a_box_round_the_back_corner_shares(radius: f64) -> f64 {
    let flange_in_the_box = HALF * HALF * (2.0 * HALF);
    let body_in_the_box = HALF
        * (HALF * (radius * radius - HALF * HALF).sqrt()
            + radius * radius * (HALF / radius).asin()
            - 2.0 * HALF * (radius - HALF));
    flange_in_the_box + body_in_the_box
}

/// The volume of `solids` together, and how far measuring it may be off.
fn volume_of(model: &Model, solids: &[SolidKey]) -> (f64, f64) {
    solids.iter().fold((0.0, 0.0), |(volume, error), &solid| {
        let measured = model
            .solid(solid)
            .expect("a solid of this model")
            .measure(
                Deflection::for_scale(model.scale()),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .expect("it measures");
        (volume + measured.volume(), error + measured.volume_error())
    })
}

/// Refused `Undecidable` with the one record `a point inside a piece of a
/// face`, handed the four corners of a sliver on the box's top `z = top`: the
/// `D3.0` LED's flat `x = -0.23` and its body's pipe at `x = -0.22666` across
/// it, and its front `y = -3.81`, written twice a rounding apart, along it.
fn a_point_inside_a_sliver_of_the_box_top_is_not_placed(top: f64, error: &Error) -> bool {
    let corners = the_points_of_the_one_record("a point inside a piece of a face", error);
    corners.len() == 4
        && corners.iter().all(|corner| {
            let [x, y, z] = corner.enclosure();
            (-0.2300000001..-0.2266).contains(&x.lo())
                && (-0.2300000001..-0.2266).contains(&x.hi())
                && (y.lo() + 3.81).abs() < 1e-9
                && (y.hi() + 3.81).abs() < 1e-9
                && (z.lo() - top).abs() < 1e-9
                && (z.hi() - top).abs() < 1e-9
        })
}

/// Refused `TwoEndsOfAnEdgeOnACurvedSurfaceRoundToOnePoint` at a point where
/// the box's top `z = top` meets the `D3.0` LED's flange's rim, a pipe of
/// radius 1.9 round the body's axis `x = 1.27, z = 6`, on its front
/// `y = -3.81`.
fn two_ends_round_to_one_point_where_the_box_top_meets_the_rim_at_the_front(
    top: f64,
    error: &Error,
) -> bool {
    let Error::TwoEndsOfAnEdgeOnACurvedSurfaceRoundToOnePoint { at, .. } = error else {
        return false;
    };
    let from_the_axis = (at.x() - 1.27).hypot(at.z() - 6.0);
    (from_the_axis - 1.9).abs() < 1e-9
        && (at.y() + 3.81).abs() < 1e-9
        && (at.z() - top).abs() < 1e-9
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
