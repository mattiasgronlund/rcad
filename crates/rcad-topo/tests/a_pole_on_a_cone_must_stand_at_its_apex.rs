//! A cone's face that stands still at a corner: kept at the apex, refused
//! anywhere else.
//!
//! A pointed cone needs no pole step (`a_solid_with_a_cone_face.rs` says why),
//! but nothing stops a builder from writing one: cut the side open along a
//! ruling, walk up it to a corner, stand still there, and walk back down. The
//! mesher then squashes that whole side of the face into the corner, which is
//! only a real shape when the corner is where the cone pinches. So
//! `Model::validate` refuses a pole anywhere else on a cone as
//! `Invalid::APoleStandsWhereTheSurfaceDoesNotPinch` (`docs/decisions.md`
//! §189.1), and nothing tested the cone half of that until this file.
//!
//! Every refused corner below is **exactly** on the cone: slope one half, and
//! each at `(z/2, 0, z)` with `z` a power of two. So the "corner is on its
//! face" check passes them, and the refusal is the pole check's alone.
//!
//! The ball's version of this file is `a_pole_off_the_pinch_is_refused.rs`.

#![forbid(unsafe_code)]

use rcad_geom::{Axis, Cone, EdgeRun, Orientation, Plane, Point, Scale, Surface, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{DraftStep, Error, Invalid, Model, Result, SolidBuilder};

const SLOPE: f64 = 0.5;
const HEIGHT: f64 = 8.0;

/// The room the sweep gives a corner of a curved face when nothing is recorded
/// on it: `MEASURING_SLACK · NEAREST_F64_POINT_ROUNDOFF` times the largest
/// coordinate in the model, which for these cones is the height.
const ROOM: f64 = 16.0 * f64::EPSILON * HEIGHT;

/// A cone about `+z` with its apex at the origin and a flat cap at `HEIGHT`.
/// The side is cut open along the ruling `y = 0, x >= 0`: round the rim, up the
/// ruling to `pole`, stand still there, and back down.
fn a_cone_whose_side_stands_still_at(pole: Point) -> Result<Model> {
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let mut builder = SolidBuilder::new();
    let rim = builder.add_vertex(Point::new(SLOPE * HEIGHT, 0.0, HEIGHT))?;
    let top = builder.add_vertex(pole)?;
    let circle = builder.add_edge(rim, rim, EdgeRun::RightRoundWhereItsFacesCross)?;
    let ruling = builder.add_edge(rim, top, EdgeRun::Straight)?;

    let plane =
        Plane::from_point_and_normal(Point::new(0.0, 0.0, HEIGHT), Vector::new(0.0, 0.0, 1.0))
            .expect("the z axis is a unit normal");
    let cap_edge = builder.add_coedge(circle, Orientation::Forward)?;
    let cap_wire = builder.add_wire(&[DraftStep::Along(cap_edge)])?;
    let cap = builder.add_face(Surface::Plane(plane), &[cap_wire])?;

    let round_the_rim = builder.add_coedge(circle, Orientation::Reversed)?;
    let up = builder.add_coedge(ruling, Orientation::Forward)?;
    let down = builder.add_coedge(ruling, Orientation::Reversed)?;
    let side_wire = builder.add_wire(&[
        DraftStep::Along(round_the_rim),
        DraftStep::Along(up),
        DraftStep::Pole(top),
        DraftStep::Along(down),
    ])?;
    let skin = Cone::new(Axis::z(), SLOPE).expect("a positive slope");
    let side = builder.add_face(Surface::Cone(skin), &[side_wire])?;

    builder.add_shell(&[(cap, Orientation::Forward), (side, Orientation::Forward)])?;
    builder.build(&mut model, &Policy::certified(), &mut ProvenanceLog::new())?;
    Ok(model)
}

/// A point on the ruling the side is cut along, `height` up the axis.
fn on_the_ruling(height: f64) -> Point {
    Point::new(SLOPE * height, 0.0, height)
}

fn the_refusal_for(pole: Point) -> Invalid {
    match a_cone_whose_side_stands_still_at(pole) {
        Err(Error::Invalid(why)) => why,
        other => panic!("a cone whose pole is at {pole:?} came back {other:?}"),
    }
}

/// The control: a pole at the apex is where a cone pinches, and the solid is
/// sound.
#[test]
fn a_pole_at_the_apex_builds_and_validates() {
    let model = a_cone_whose_side_stands_still_at(Point::origin())
        .expect("a cone whose pole is at its apex");
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the built cone validates again");
}

/// Halfway up: exactly on the cone, four units from the apex.
#[test]
fn a_pole_halfway_up_the_cone_is_refused() {
    let halfway = on_the_ruling(HEIGHT / 2.0);
    assert_eq!(
        the_refusal_for(halfway),
        Invalid::APoleStandsWhereTheSurfaceDoesNotPinch {
            at: halfway,
            pinch: Some(Point::origin()),
        }
    );
}

/// A few ulps out from the apex, inside the room: kept.
///
/// **Beside the apex and not up the ruling**, because a corner up the ruling
/// is measured by the dynamic range check as a circle of the cone's radius
/// there: `2^-50` up comes back `DynamicRangeTooWide` with a ratio of
/// `1.8e16`. A corner level with the apex has radius zero there and is passed
/// over. It is off the cone by `x²`, which the room keeps too.
#[test]
fn a_pole_inside_the_room_of_the_apex_builds() {
    let beside_the_apex = Point::new(2f64.powi(-50), 0.0, 0.0);
    assert!(
        2f64.powi(-50) < ROOM,
        "{} against a room of {ROOM}",
        2f64.powi(-50)
    );
    a_cone_whose_side_stands_still_at(beside_the_apex).expect("a pole within the room of the apex");
}

/// The other side of the same room: `2^-40` up is about thirty-two rooms from
/// the apex, and it is refused.
#[test]
fn a_pole_past_the_room_of_the_apex_is_refused() {
    let past_the_room = on_the_ruling(2f64.powi(-40));
    assert!(
        2f64.powi(-40) > 4.0 * ROOM,
        "{} against a room of {ROOM}",
        2f64.powi(-40)
    );
    assert_eq!(
        the_refusal_for(past_the_room),
        Invalid::APoleStandsWhereTheSurfaceDoesNotPinch {
            at: past_the_room,
            pinch: Some(Point::origin()),
        }
    );
}
