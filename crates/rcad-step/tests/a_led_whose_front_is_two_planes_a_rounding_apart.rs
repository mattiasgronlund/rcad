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
//! flat and the flange's front meet. It was read as where the line the two
//! planes share crosses the rim at a corner bake solved so, but never at a
//! node of a walk, and the reading gave up wherever a product of it would
//! round, as one does when the two planes tilt a hair. A node is read so now,
//! and the reading falls back on intervals where a product would round
//! (Q3017).
//!
//! Past it, the far box of this LED, and of the two D5 LEDs whose axis is 15
//! up, stop on the order along a crossing of two points `2e-15` apart, where
//! the side of a lead, the plane `x = -0.3`, meets the body's pipe on the
//! flange's front below the axis. The far box of
//! `LED_D3.0mm_Horizontal_O3.81mm_Z2.0mm.step` stops on the order along a
//! crossing of two points on the side of its other lead, `x = 2.24`, `1.2e32`
//! out along `-y`: a meet as far out as the one Q3019 orders. A box on
//! this LED's flat, and the far box of
//! `LED_D3.0mm_Horizontal_O6.35mm_Z6.0mm.step`, stop on the order along the
//! loop a plane `x = c` cuts the rim in, between where the disc's plane and
//! the flange's front cross it a rounding apart: the box's side `x = -1.33`
//! here, a lead's side `x = -0.3` in the D3. The last two,
//! `LED_D3.0mm_Horizontal_O6.35mm_Z10.0mm.step` and
//! `LED_D5.0mm_Horizontal_O6.35mm_Z9.0mm.step`, stop on the flange's front,
//! face slot 2, the plane `y = -6.35`, refused
//! `CannotTellTheCellsOfAFaceApart`: the refusal Q2974 and Q2982 cleared from
//! the fronts of two LEDs of `a_led_whose_flat_touches_its_body_at_a_corner.rs`.
//! These tests pin all four.

use std::{env, fs, path::PathBuf};

use rcad_geom::Surface;
use rcad_predicates::{Point, Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Error, Model, ProfileBuilder, Rule, SolidKey};

/// What a refusal on which of two places comes first along a crossing names.
const ALONG_A_CROSSING: &str = "order along a crossing";

/// Where the box that misses the LED goes.
const FAR_OFF: [f64; 3] = [10.0, 10.0, 10.0];

/// The LED this file is about.
const THIS_ONE: Led = Led {
    file: "LED_D5.0mm_Horizontal_O6.35mm_Z3.0mm.step",
    front: -6.35,
    rim: 2.9,
    body: 2.5,
    axis: 3.0,
};

/// The side of a lead nearer the flange's flat in a D5 LED, the plane `x =
/// -0.3`.
const A_D5_LEADS_SIDE: f64 = -0.3;

/// The two other D5 LED files that refused on their rim and front disc and
/// stop where [`A_D5_LEADS_SIDE`] meets their body's pipe below the axis, as
/// this one does.
const TWO_D5S: [Led; 2] = [
    Led {
        file: "LED_D5.0mm_Horizontal_O3.81mm_Z15.0mm.step",
        front: -3.81,
        rim: 2.9,
        body: 2.5,
        axis: 15.0,
    },
    Led {
        file: "LED_D5.0mm_Horizontal_O6.35mm_Z15.0mm.step",
        front: -6.35,
        rim: 2.9,
        body: 2.5,
        axis: 15.0,
    },
];

/// Where the box on this LED's flat goes, and the plane its side nearer the
/// flat's back lies in, `x = -1.33`.
const ON_THE_FLAT: ([f64; 3], f64) = ([-1.23, -6.35, 3.0], -1.33);

/// The D3 LED file that refused on its rim and front disc and stops on two
/// points far out on the side of its lead further from its flat, the plane `x =
/// 2.24`.
const FAR_ALONG_A_LEAD: (&str, f64) = ("LED_D3.0mm_Horizontal_O3.81mm_Z2.0mm.step", 2.24);

/// The LED file whose far box stops where the side of a lead, the plane `x =
/// -0.3`, meets its rim on its flange's front.
const BY_A_LEAD: (Led, f64) = (
    Led {
        file: "LED_D3.0mm_Horizontal_O6.35mm_Z6.0mm.step",
        front: -6.35,
        rim: 1.9,
        body: 1.5,
        axis: 6.0,
    },
    -0.3,
);

/// The two other LED files that refused on their rim and front disc and stop
/// on the cells of their flange's front, each with the `y` of that front.
const TWO_MORE: [(&str, f64); 2] = [
    ("LED_D3.0mm_Horizontal_O6.35mm_Z10.0mm.step", -6.35),
    ("LED_D5.0mm_Horizontal_O6.35mm_Z9.0mm.step", -6.35),
];

/// An LED lying along `-y`, with its flange's front `y = front`, its flange's
/// rim a pipe of radius `rim` and its body a pipe of radius `body`, both about
/// the line `x = 1.27, z = axis`.
struct Led {
    file: &'static str,
    front: f64,
    rim: f64,
    body: f64,
    axis: f64,
}

impl Led {
    /// The two places on the flange's front where the plane `x = across` meets
    /// its rim, above the axis and below it.
    fn where_a_plane_across_meets_the_rim(&self, across: f64) -> [[f64; 3]; 2] {
        self.where_a_plane_across_meets_a_pipe(self.rim, across)
    }

    /// The two places on the flange's front where the plane `x = across` meets
    /// the body's pipe, above the axis and below it.
    fn where_a_plane_across_meets_the_body(&self, across: f64) -> [[f64; 3]; 2] {
        self.where_a_plane_across_meets_a_pipe(self.body, across)
    }

    fn where_a_plane_across_meets_a_pipe(&self, radius: f64, across: f64) -> [[f64; 3]; 2] {
        let up = (radius.powi(2) - (1.27 - across).powi(2)).sqrt();
        [self.axis + up, self.axis - up].map(|z| [across, self.front, z])
    }
}

#[test]
fn an_led_whose_disc_sits_inside_its_rim_gets_past_where_its_flat_meets_its_rim_and_stops_where_a_leads_side_meets_its_body()
 {
    every_fuse_refuses(THIS_ONE.file, &[FAR_OFF], |_, error| {
        let [_, below] = THIS_ONE.where_a_plane_across_meets_the_body(A_D5_LEADS_SIDE);
        the_order_of_two_points_at_is_undecidable(below, error)
    });
}

#[test]
fn a_box_on_the_flat_stops_where_its_side_meets_the_rim_on_the_two_front_planes() {
    let (centre, side) = ON_THE_FLAT;
    every_fuse_refuses(THIS_ONE.file, &[centre], |_, error| {
        let [above, _] = THIS_ONE.where_a_plane_across_meets_the_rim(side);
        the_order_of_two_points_at_is_undecidable(above, error)
    });
}

#[test]
fn two_more_d5_leds_get_past_where_their_flat_meets_their_rim_and_stop_where_a_leads_side_meets_their_body()
 {
    for led in &TWO_D5S {
        every_fuse_refuses(led.file, &[FAR_OFF], |_, error| {
            let [_, below] = led.where_a_plane_across_meets_the_body(A_D5_LEADS_SIDE);
            the_order_of_two_points_at_is_undecidable(below, error)
        });
    }
}

#[test]
fn a_d3_led_gets_past_where_its_flat_meets_its_rim_and_stops_on_two_points_far_along_a_leads_side()
{
    let (file, side) = FAR_ALONG_A_LEAD;
    every_fuse_refuses(file, &[FAR_OFF], |_, error| {
        the_order_of_two_points_far_out_on_a_plane_across_is_undecidable(side, error)
    });
}

#[test]
fn a_d3_led_gets_past_its_front_corner_and_stops_where_a_leads_side_meets_its_rim() {
    let (led, side) = &BY_A_LEAD;
    every_fuse_refuses(led.file, &[FAR_OFF], |_, error| {
        let [above, _] = led.where_a_plane_across_meets_the_rim(*side);
        the_order_of_two_points_at_is_undecidable(above, error)
    });
}

#[test]
fn two_more_leds_get_past_their_front_corner_to_the_cells_of_their_front() {
    for (file, front) in TWO_MORE {
        every_fuse_refuses(file, &[FAR_OFF], |model, error| {
            the_cells_of_the_front_cannot_be_told_apart(model, error, front)
        });
    }
}

/// The LED in `file` and a box of side 0.2 centred on each of `centres`, fused
/// under intersection and under difference, each refused as `the_refusal`
/// takes, handed the model it was refused in.
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

/// Refused `Undecidable` with the one record `order along a crossing`, both its
/// points within `1e-9` of `at`.
fn the_order_of_two_points_at_is_undecidable(at: [f64; 3], error: &Error) -> bool {
    the_two_points_that_could_not_be_ordered(error)
        .is_some_and(|points| points.iter().all(|point| is_within_1e_9_of(point, at)))
}

/// Refused `Undecidable` with the one record `order along a crossing`, both its
/// points within `1e-9` of the plane `x = across` and more than `1e30` out
/// along `-y`.
fn the_order_of_two_points_far_out_on_a_plane_across_is_undecidable(
    across: f64,
    error: &Error,
) -> bool {
    the_two_points_that_could_not_be_ordered(error).is_some_and(|points| {
        points.iter().all(|point| {
            let [x, y, _] = point.enclosure();
            (x.lo() - across).abs() < 1e-9 && (x.hi() - across).abs() < 1e-9 && y.hi() < -1e30
        })
    })
}

/// The two points of the one record `order along a crossing` of an
/// `Undecidable` refusal, or `None` for any other refusal.
fn the_two_points_that_could_not_be_ordered(error: &Error) -> Option<[Point; 2]> {
    let Error::Undecidable { records } = error else {
        return None;
    };
    let [record] = records.records() else {
        return None;
    };
    let [one, other] = record.inputs() else {
        return None;
    };
    (record.predicate() == ALONG_A_CROSSING).then_some([*one, *other])
}

fn is_within_1e_9_of(point: &Point, at: [f64; 3]) -> bool {
    point
        .enclosure()
        .iter()
        .zip(at)
        .all(|(side, at)| (side.lo() - at).abs() < 1e-9 && (side.hi() - at).abs() < 1e-9)
}

/// Refused `CannotTellTheCellsOfAFaceApart` on the flange's front, the face
/// of the LED on the plane `y = front`.
fn the_cells_of_the_front_cannot_be_told_apart(model: &Model, error: &Error, front: f64) -> bool {
    let Error::CannotTellTheCellsOfAFaceApart { face } = error else {
        return false;
    };
    let surface = model
        .face(*face)
        .expect("the refusal names a face of the LED")
        .surface();
    let Surface::Plane(plane) = surface else {
        return false;
    };
    [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
        .into_iter()
        .all(|[x, z]| {
            plane
                .signed_distance_to(rcad_geom::Point::new(x, front, z))
                .abs()
                < 1e-9
        })
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
