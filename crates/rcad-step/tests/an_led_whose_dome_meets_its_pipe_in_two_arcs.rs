//! A small box fused with a KiCad LED whose round dome meets its body's pipe
//! in two arcs.
//!
//! `LED_D5.0mm-3.step` is a body standing up `z`: a pipe of radius 2.5 about
//! the line `x = 2.54, y = 0`, topped at `z = 9.1` by a dome, a ball of the same
//! radius centred on that line. The ball touches the pipe all the way round the
//! rim `z = 9.1` and never crosses it. The file draws the rim as two arcs, from
//! `(5.04, 0, 9.1)` to `(0.04, 0, 9.1)` and back, and each arc's run names the
//! rim's plane. Its one corner where two fans of faces touch is
//! `(0.04, 0, 4)`, where the flange's flat touches the pipe (§471.3).
//!
//! `LED_D3.0mm_Horizontal_O6.35mm_Z2.0mm.step` is the same kind of LED lying
//! down: a pipe of radius 1.5 about `x = 1.27, z = 2` running back along `-y`,
//! with its dome about `(1.27, -10.15, 2)` and its rim on the plane
//! `y = -10.15`. Its two corners in two fans are at `(-0.23, -6.35, 2)` and
//! `(-0.23, -7.35, 2)`.
//!
//! Every fuse of either LED tried by Q2931's lane refused
//! `AFaceTrimmedByACurveThisCannotFollow` naming face slot 2, the body's pipe,
//! the box at `(10, 10, 10)` included. The fuse baked each arc of the pipe's
//! rim against the ball across it, and the arrangement could not tell which
//! way along the rim an arc runs from two curved surfaces that only touch. Bake
//! now reads such an arc against the plane its run names, as `classify` does
//! since Q2923, and every fuse of either LED gets past the rim (Q2940).
//!
//! These tests pin where each stops now:
//!
//! - The `D5.0` LED stopped where its dome is split, refused
//!   `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`: the plane `y = 0.2` or
//!   `y = -0.2` as faces 8 and 10 store it, the ball and the pipe, which touch
//!   along the rim, had no road to where the three meet (Q2941). The road for
//!   two conics kept the height, and each height holds two of the four
//!   corners; it now asks again in the height plus `x` and answers four
//!   corners or none (Q2955). Every fuse then stopped refused `Undecidable` on
//!   order along a crossing, between two nodes on the rim at `x = 5.031987`,
//!   `z = 9.1`, whose boxes are `5.6e-16` apart in `y`: where the plane
//!   `y = -0.2`, written twice a rounding apart by face 8 and by faces 12 and
//!   15, crosses the rim. The order now turns them about the line through the
//!   ball's centre along the rim plane's normal, and they are parted (Q2956).
//!   Every fuse then stopped on order along a crossing again, between the
//!   meets of those two planes with the ball and the pipe near
//!   `(0.048, -0.2, 9.1)`. The ball is stored `2e-15` off the pipe's axis, so
//!   the two cross in a thin loop up to `1e-7` tall, round the `-y` half of
//!   the rim. The order now reads which stretch of that loop each is on, and
//!   how far it is turned about the pipe's axis (Q2956). Every fuse then
//!   stopped refused `Undecidable` on side of cylinder alone, for a point of
//!   the rim: `(5.038, 0.1, 9.1)` for the box round the flat's corner, and
//!   where the plane `y = 0.2` crosses the rim, `(5.032, 0.2, 9.1)`, for the
//!   box that misses the LED. The rim is inside the pipe there by `4e-16` and
//!   `8e-16`, and the pipe's side of a point of the ball is now read with the
//!   ball's own equation, where the box's width counts only times how far the
//!   ball's centre is off the axis (Q2956). Every fuse then stopped refused
//!   `Undecidable` on order along a crossing, on the body's pipe, between two
//!   nodes in one box near `(2.34, 2.492, 3.5)`: where the line the plane
//!   `x = 2.34` cuts the pipe in meets each of two sloped planes, with normals
//!   `(−0.619, ±3.5e-16, ±0.786)`. On `x = 2.34` the two sloped planes are
//!   two parallel lines `4.8e-16` apart along the pipe, so they are that far
//!   apart along every line of the plane up the pipe, and the order now reads
//!   them off the stored numbers (Q2984). Every fuse then stopped on order
//!   along a crossing again, on the body's pipe, between two nodes in one box
//!   near `(2.34, −2.492, 3.5)`, round the loop the slope with normal
//!   `(−0.619, −3.5e-16, −0.786)` cuts the pipe in: its meets with the other
//!   slope and with `x = 2.34`. The two slopes cross in a line `3.1e-16` off
//!   that plane and along it, so on the slope the two cut in parallel lines a
//!   rounding apart, and the order now reads which of the two lines is
//!   higher across the slope, from the stored numbers (Q2985). Every fuse then
//!   stopped refused `Undecidable` on side of sphere alone, for the points of
//!   the rim whose side of the pipe Q2956 reads with the ball's equation, now
//!   asked on the pipe's face which side of the ball they are on:
//!   `(5.038, 0.1, 9.1)` for the box round the flat's corner, and
//!   `(5.032, 0.2, 9.1)` for the box that misses the LED. They are outside it
//!   by `4e-16` and `8e-16`, and the ball's side of a point of the pipe is now
//!   read with the pipe's own equation, the same way turned round (Q2992).
//!   Every fuse then stopped refused `Undecidable` on side of plane, on the
//!   body's pipe, at a node `1.8e15` up it, near `(1.07, 1.415, 1.8e15)`.
//!   There the line the plane `x = 1.07` cuts the pipe in meets the line the
//!   plane `y = 0.2` does, as one face writes it, normal
//!   `(0, 1, −6.7e-16)`: two lines along the pipe that lean apart by a
//!   rounding. It was asked against `y = 0.2` as another face writes it,
//!   normal `(7.8e-32, 1, −6.7e-16)`, its offset `6.7e-16` from the first's,
//!   for which side of that writing a walk through it is on. Every node of a
//!   walk is on the walk's side of a crossing it does not run along, or on
//!   the crossing, so a node that cannot tell is now passed over and the walk
//!   read off its next node, `(1.07, 1.28, 2.2e15)`, which can (Q3009). Every
//!   intersection now answers, and shares with the LED what it should; every
//!   difference stopped refused `TwoEndsOfAnEdgeOnACurvedSurfaceRoundToOnePoint`
//!   on the flange's rim, the pipe of radius 2.9, at `(2.74, −2.893, 3.5)`.
//!   There a lead's side `x = 2.74` and two slopes that by design meet it on
//!   one line cut the rim, which no face of the three reaches, in three
//!   places: the two on `x = 2.74` are `3.0e-16` apart over the stored
//!   numbers and round to one `f64` triple (Q3056). The straight side between
//!   them was refused as though it were an arc. A straight side on a pipe
//!   whose two ends are one corner is now left out, as one between two planes
//!   is (Q3057), and every difference stopped refused `DynamicRangeTooWide`,
//!   at a ratio of `1.8e16`. That was `ResultIsNotManifold` read for its
//!   range: 26 pieces of the ball and 26 of the pipe each walked an edge of
//!   the rim `z = 9.1` no other face walked. The ball and the pipe cross in
//!   the thin loop above, so the ball keeps a band between the rim and the
//!   loop's top branch, `1e-7` above it, and the pipe a band down to the
//!   loop's bottom branch, and planes across the rim cut each band in pieces.
//!   Each piece's side along the rim is a piece of the one input edge, but
//!   the ball's is carried by the ball and the rim's plane and the pipe's by
//!   the pipe and that plane. The two circles they cut in the plane are
//!   centred `2e-15` apart, so they cross in two points and do not share more
//!   than a point, and the arcs were not joined under one name as they are on
//!   a bar capped by a ball of its radius (Q3000). Their corners are named by
//!   the plane, the plane across and the ball or the pipe:
//!   `(2.74, -2.491987158875427, 9.1)` for the ball and
//!   `(2.74, -2.4919871588754248, 9.1)` for the pipe, two triples, so even
//!   where a corner was shared the two edges were two. Two arcs on one plane
//!   whose sides run along one input edge are now one conic, and where one
//!   line meets that conic the ball's and the pipe's names are one corner
//!   (Q3104), so every edge of the rim has two faces on it. Every difference
//!   still stops refused `DynamicRangeTooWide`, at a ratio of `1.8e16`, now
//!   on thin strips of the body's pipe. The lead's sides `y = 0.2` and
//!   `y = -0.2` are each stored twice, a rounding apart, and each copy cuts
//!   the pipe along its own line, `6.7e-16` from the other's. The rings,
//!   slopes, loop and rim across the pipe cut each strip in edges that short,
//!   and pieces of one input face are joined back only where they are flat.
//!   Under a tolerant policy, which lets that range through, every difference
//!   stops where the ball and the pipe are too nearly tangent to follow the
//!   thin loop they cross in. That loop is an edge the fuse invented, so there
//!   is no edge of the LED to name. The refusal used to be
//!   `ACrossingAlongThisEdgeCouldNotBeFollowed`, naming a key of the model
//!   the fuse meshes its answer in, which the caller's model did not hold.
//!   It is now the walk's own `Error::Geometry` (Q2889).
//! - The `D3.0` LED stopped there too, where its flange's flat meets the body's
//!   pipe and the flange's rim, a pipe of radius 1.9 about the same axis a
//!   rounding apart in direction. Q2938's road answers that triple, and the
//!   fuse then stopped in the check that the LED does not cross itself, on the
//!   flange's flat `x = -0.23`. The file writes that flat as two faces, slots 5
//!   and 6, on one plane stored bit for bit the same and used one way round,
//!   with an edge between them along `z = 2`. Bake joined faces only on a
//!   curved surface, so `crossing_for` (fuse/arrange.rs) met the flat against
//!   its own plane and refused `AFaceMeetsItsNeighbourOnItsOwnSurface`. Bake
//!   now joins two faces on one plane written and used one way (Q2962), and
//!   every fuse was refused `Undecidable` on side of cylinder alone, for a box
//!   round the front corner `(-0.23, -6.35, 2)` from `z = -0.767` to `3.068`.
//!   Bake solved that corner from the flat, the flange's front and the body's
//!   front disc, the last two one plane written twice a rounding apart (Q2967).
//!   It now reads a corner in two fans of faces out of the model and never
//!   solves it (Q2939), and every fuse stopped on the flange's front, face
//!   slot 8, the plane `y = -6.35`, refused `CannotTellTheCellsOfAFaceApart`:
//!   the refusal Q2974 and Q2982 cleared from the fronts of two LEDs of
//!   `a_led_whose_flat_touches_its_body_at_a_corner.rs`, but not from this
//!   one's. The flat is stored `1.2e-16` inside the hole's circle on the
//!   front, so its line crosses the circle at `z = 2 ∓ 1.9e-8`, near the front
//!   corner, and the arrangement kept both places as nodes of their own, with
//!   a sliver between them. Each is now folded into the corner when the line
//!   and the circle stay within a rounding of each other all the way from it
//!   to the corner (Q3010). Every fuse then stopped refused `Undecidable` on
//!   which way two tangent curves bend, at the back corner
//!   `(-0.23, -7.35, 2)`, on the body's pipe. The flat leans `1.0e-16` off
//!   square to the pipe's axis, so it cuts the pipe in a long thin ellipse,
//!   whose two sides cross the circle of the flange's back `7.8e-9` either side
//!   of the corner and are folded into it the same way. How far the ellipse
//!   bends was read with a dot product of terms that cancel, and its way out
//!   at the corner is under `4.6e-16` long, so the sum straddled nothing. It is
//!   now read as the square of that way out (Q3028). The boxes round the front
//!   corner and far off then stopped refused `Undecidable` on order along a
//!   crossing, where the box's side `x = -0.33` or a lead's side `x = -0.3`
//!   meets the flange's rim on its front below the axis: Q3029's stop in the
//!   `Z10.0` LED, `8` lower, between where the flange's front and the body's
//!   front disc, one plane written twice a rounding apart, cross that line.
//!   Their order is now read off which side of one plane the other's meet
//!   lies (Q3059). Both boxes then stopped refused `Undecidable` on
//!   `orient3d`, handed the flat's normal `(-1, 1.6e-16, -1.9e-31)`, where the
//!   flat's thin ellipses with the body's pipe and the flange's rim cross
//!   `7.5e14` up `y`, and the pipes' gradients there were boxes `0.9` wide
//!   along `y`. They are now read again as `w x ((x - o) x w)`, where that
//!   coordinate never meets itself. On the flat, the flange's front and the
//!   body's front disc then both run through the corner `(-0.23, -6.35, 2)`
//!   and cross `0.486` lower, so the slivers between the two lines were walked
//!   twice. Two lines meet once, so the corner is now read as a place they
//!   touch (Q3058). Both boxes stopped refused `CannotTellTheCellsOfAFaceApart`
//!   on the flange's front, face slot 8, which the body's front disc, a
//!   rounding off the flange front's own plane, also cuts. The box
//!   round the back corner stopped refused `CannotTellTheCellsOfAFaceApart`
//!   on the body's pipe, face slot 2. On that face the flat's ellipse has a
//!   cap about `0.2` long behind the back plane, and the box's side
//!   `y = -7.45` crosses the cap. Both places where the ellipse crosses the
//!   back circle are folded into the corner, so one node stands for two places
//!   `0.4` apart along the ellipse, and the order along it puts that node at
//!   the tip of the cap. Two arcs of the ellipse then cross the circle where
//!   there is no node, and one walk runs along both sides of the circle
//!   (Q3035). Every face of the flat lies in front of the back plane, so none
//!   reaches the pipe's face and the ellipse is no edge of the answer there. A
//!   flat face now knows the closed sides of the planes across its straight
//!   edges it lies in, and a pipe's face the side of a plane its edges go
//!   round it on in a ring, so the flat is read in front of the back plane and
//!   the pipe behind it, and the ellipse is left out (Q3048). The box round the
//!   back corner stopped then on the flange's front, face slot 8, as the other
//!   two did. The front's own plane and the disc's cross in a line about
//!   `z = 1.5` on the face, and the arrangement took that line through the
//!   corners of the hole at `(1.27, -6.35, 3.5)`, `(-0.23, -6.35, 2)` and
//!   `(2.77, -6.35, 2)` without asking, because the disc is one of the
//!   surfaces round each. The line zigzagged across the face, and the
//!   windings of its cells could not be counted. The face's own plane written
//!   a second time is now always asked whether it passes through a corner
//!   (Q3071), and every fuse of all three boxes stops refused
//!   `TwoCurvesTouchWithoutCrossing` on face slot 18, the side `x = 0.3` of a
//!   lead. The body's pipe and the flange's rim run about an axis a rounding
//!   off that plane's, and cross it in ellipses whose meets reach `4e16` out
//!   along `y`. Untraced.
//!
//! What answers is checked as commit `f179ca02a` checks it: what the box and
//! the LED share is the box's part inside the LED. When a box's difference
//! answers too, the intersection and the difference must add up to the LED.

use std::{env, fs, path::PathBuf};

use rcad_geom::{Deflection, Isometry, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Error, Invalid, Model, ProfileBuilder, Rule, SolidKey};

const D5: &str = "LED_D5.0mm-3.step";
const D3: &str = "LED_D3.0mm_Horizontal_O6.35mm_Z2.0mm.step";

/// Where the box round the D3 LED's corner in two fans of faces on its
/// flange's front goes, `4.4e-8` below the corner `(-0.23, -6.35, 2)`.
const THE_FRONT_CORNER: [f64; 3] = [-0.23, -6.35, 1.999999956464];

/// The D3 LED's corner in two fans of faces on its flange's back, where its
/// flat touches its body's pipe.
const THE_BACK_CORNER: [f64; 3] = [-0.23, -7.35, 2.0];

/// Half the side of the box put round each corner.
const HALF: f64 = 0.1;

/// Where the box that misses the LED goes.
const FAR_OFF: [f64; 3] = [10.0, 10.0, 10.0];

/// The box round the corner `(0.04, 0, 4)` holds the flange below `z = 4`
/// from the flat `x = 0.04` on, and the body's pipe above it.
#[test]
fn a_d5_led_gets_its_dome_rim_on_two_faces_and_stops_cutting_a_box_out_on_strips_two_copies_of_a_lead_side_cut_from_its_pipe()
 {
    let radius = 2.5_f64;
    let flange_in_the_box = HALF * HALF * (2.0 * HALF);
    let body_in_the_box = HALF
        * (HALF * (radius * radius - HALF * HALF).sqrt()
            + radius * radius * (HALF / radius).asin()
            - 2.0 * HALF * (radius - HALF));
    the_led_and_a_box_share(D5, [0.04, 0.0, 4.0], flange_in_the_box + body_in_the_box);
    the_led_and_a_box_share(D5, FAR_OFF, 0.0);
    every_fuse_refuses(
        D5,
        &[
            ([0.04, 0.0, 4.0], Rule::Difference),
            (FAR_OFF, Rule::Difference),
        ],
        too_wide_a_range,
        &Policy::certified(),
    );
}

/// A tolerant policy lets the strips' range through, and the box round the
/// corner `(0.04, 0, 4)` and the box that misses the LED are each refused
/// where the ball and the pipe, too nearly tangent to follow, cross in the
/// thin loop below the rim. Before the rim's arcs were joined by their input
/// edge, both were refused `ResultIsNotManifold` here. The loop is no edge of
/// the LED, so the refusal is the walk's own `Error::Geometry`. Before Q2889 it
/// was `ACrossingAlongThisEdgeCouldNotBeFollowed`, naming an edge the
/// caller's model did not hold.
#[test]
fn a_d5_led_less_a_box_gets_past_its_rim_under_a_tolerant_policy_and_stops_where_its_ball_and_pipe_cross_in_a_thin_loop()
 {
    every_fuse_refuses(
        D5,
        &[
            ([0.04, 0.0, 4.0], Rule::Difference),
            (FAR_OFF, Rule::Difference),
        ],
        the_ball_and_the_pipe_are_too_nearly_tangent_to_follow,
        &Policy::tolerant(10.0),
    );
}

#[test]
fn a_d3_led_gets_past_the_cells_of_its_flange_front_and_stops_where_two_curves_touch_on_the_side_of_a_lead()
 {
    every_fuse_refuses(
        D3,
        &both_rules_with_a_box_round_each_of(&[THE_FRONT_CORNER, THE_BACK_CORNER, FAR_OFF]),
        two_curves_touch_on_the_side_of_a_lead,
        &Policy::certified(),
    );
}

/// Refused on the `D5.0` LED because the body's pipe and its dome's ball are
/// too nearly tangent where they cross.
fn the_ball_and_the_pipe_are_too_nearly_tangent_to_follow(_: &Model, error: &Error) -> bool {
    let Error::Geometry(rcad_geom::Error::CannotFollowACrossingWhereItsSurfacesAreTangent {
        surfaces,
        ..
    }) = error
    else {
        return false;
    };
    matches!(**surfaces, [Surface::Cylinder(_), Surface::Sphere(_)])
}

/// Refused `TwoCurvesTouchWithoutCrossing` on the `D3.0` LED's lead's side,
/// the plane `x = 0.3` with one wire.
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
        && [x - 1.0, y, z, d + 0.3]
            .iter()
            .all(|part| part.abs() < 1e-9)
}

/// The LED in `file` and a box of side `2 * HALF` centred on `centre`, fused
/// under intersection, answer with a model `validate` accepts, and what they
/// share is `shared` to within what measuring costs.
fn the_led_and_a_box_share(file: &str, centre: [f64; 3], shared: f64) {
    let (mut model, solid) = the_led(file);
    let knife = a_box_round(&mut model, centre);
    let fused = model
        .fuse(
            &[solid],
            &[knife],
            Rule::Intersection,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{file}: the box round {centre:?} refused: {why:?}"));
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| {
            panic!("{file}: the box round {centre:?} left a model that is not valid: {why:?}")
        });
    let (common, error) = volume_of(&model, fused.solids());
    assert!(
        (common - shared).abs() <= error + 1e-12,
        "{file}: the LED and the box round {centre:?} share {common} ± {error}, and {shared} is \
         right"
    );
}

/// Each box of side `2 * HALF` centred on a corner of `fuses`, fused with the
/// LED in `file` under its rule and `policy`, is refused as `the_refusal`
/// takes, handed the model it was refused in, which holds every face of the
/// LED.
fn every_fuse_refuses(
    file: &str,
    fuses: &[([f64; 3], Rule)],
    the_refusal: impl Fn(&Model, &Error) -> bool,
    policy: &Policy,
) {
    let (led, solid) = the_led(file);
    for &(centre, rule) in fuses {
        let mut model = led.clone();
        let knife = a_box_round(&mut model, centre);
        match model.fuse(&[solid], &[knife], rule, policy, &mut ProvenanceLog::new()) {
            Err(error) if the_refusal(&model, &error) => {}
            Err(error) => panic!(
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

/// A box round each centre, to be fused with the LED under intersection and
/// under difference.
fn both_rules_with_a_box_round_each_of(centres: &[[f64; 3]]) -> Vec<([f64; 3], Rule)> {
    centres
        .iter()
        .flat_map(|&centre| [Rule::Intersection, Rule::Difference].map(|rule| (centre, rule)))
        .collect()
}

/// Refused `DynamicRangeTooWide` on the `D5.0` LED, the ratio of its largest
/// coordinate to its shortest edge about `1.8e16`.
fn too_wide_a_range(_: &Model, error: &Error) -> bool {
    matches!(
        error,
        Error::Invalid(Invalid::DynamicRangeTooWide { ratio }) if (1.8e16..1.9e16).contains(ratio)
    )
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
    let side = 2.0 * HALF;
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    let low = Vector::new(centre[0] - HALF, centre[1] - HALF, centre[2] - HALF);
    model
        .add_extrusion(
            &square,
            side,
            &Isometry::translation(low).expect("a finite shift"),
        )
        .expect("a box")
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
