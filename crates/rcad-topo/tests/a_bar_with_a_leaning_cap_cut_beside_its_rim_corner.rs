//! A plate with a round tab at its corner whose cap leans, turned about `z`
//! (Q2856, from Q2854 and Q2716).
//!
//! At `Scale::millimetres(100.0)` the plate is the square `[0, 10]²` extruded
//! 2 high, united under certified with `add_bar_cut_off_by(3, [the plane
//! z = 0 facing down, the plane through local (3, 0, 1) with normal lean])`
//! at `(10, 10, 0)`, the two placed with the same turn before they are
//! united. The block it is then united with is `[50, 56]² × [0, 3]`, placed
//! with the same turn and nowhere near it, so the answer is the two solids as
//! they were.
//!
//! The cap cuts the bar's wall in an ellipse, and that ellipse is walked from
//! an end of its short axis, along the bar's axis crossed with the cap's
//! normal. Each of the plate's walls stands on a line through the bar's axis,
//! so one of them crosses the ellipse right where it is walked from: the wall
//! `y = 10` at `(7, 10, 1)` for the lean `(0, 0.1, 1)`, and the wall `x = 10`
//! at `(10, 13, 1.3)` for `(0.1, 0, 1)`. Turned, that corner is a rounding off
//! the walked start, and its box misses it.

use rcad_geom::{Axis, Deflection, Isometry, Plane, Point, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// The lean `(0, 0.1, 1)` turned `29 · 0.05`: the ellipse is walked from
/// `(−9.083610524804325, 8.154018630936783, 0.9999999999999999)`, and the
/// wall corner's box is `[1, 1.0000000000000007]` in `z`, one ulp short of it.
/// No node's box held the start, so none was counted first round the
/// ellipse, and the sweep from the start to that corner against the seam
/// corner across, `7.4e-14`, refused the union on the order along a crossing.
#[test]
fn the_wall_corner_one_ulp_off_the_rims_start_is_counted_first_round_it() {
    unites_with_the_far_block_as_the_two_it_was([0.0, 0.1, 1.0], f64::from(29) * 0.05)
        .unwrap_or_else(|why| panic!("{why}"));
}

/// The lean `(0.1, 0, 1)` turned `0.05`: the first fuse leaves a corner where
/// the wall `x = 10` crosses the cap's rim, exact at
/// `(9.337773403430845, 13.483545077841342, 1.2999999999999998)`, two ulps
/// off the walked start `(9.337773403430843, 13.483545077841345, 1.3)`. The
/// sweep from the start put that corner third of the four round the rim, two
/// edges of the cap were drawn along one stretch of it, and the union was
/// refused `AnEdgeOfAFaceIsNotDrawnAlongItsOwnCurve`.
#[test]
fn the_rim_corner_two_ulps_off_the_rims_start_is_counted_first_round_it() {
    unites_with_the_far_block_as_the_two_it_was([0.1, 0.0, 1.0], 0.05)
        .unwrap_or_else(|why| panic!("{why}"));
}

/// The plate of `lean` and the far block, turned by `angle` about `z` and
/// united under certified; the answer must validate as two solids that
/// measure the plate's closed form and the block's `108`, each within its
/// mesh's own volume error.
fn unites_with_the_far_block_as_the_two_it_was(lean: [f64; 3], angle: f64) -> Result<(), String> {
    let policy = Policy::certified();
    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let turn = Isometry::rotation(&Axis::z(), angle).expect("a finite angle");
    let plate = a_plate_with_a_leaning_tab(&mut model, &turn, lean)?;
    let block = a_block(&mut model, [50.0, 50.0, 0.0], [56.0, 56.0, 3.0]);
    model
        .place_solid(block, &turn)
        .expect("this model made the key");
    let fused = model
        .fuse(
            &[plate],
            &[block],
            Rule::Union,
            &policy,
            &mut ProvenanceLog::new(),
        )
        .map_err(|why| format!("lean {lean:?} turned {angle}: refused {why:?}"))?;
    model
        .validate(&policy, &mut ProvenanceLog::new())
        .map_err(|why| format!("lean {lean:?} turned {angle}: does not validate: {why}"))?;
    let mut volumes = fused
        .solids()
        .iter()
        .map(|solid| {
            let measured = model
                .solid(*solid)
                .expect("this model made the key")
                .measure(
                    Deflection::of(0.01).expect("a positive length"),
                    &policy,
                    &mut ProvenanceLog::new(),
                )
                .map_err(|why| {
                    format!("lean {lean:?} turned {angle}: does not measure: {why:?}")
                })?;
            Ok((measured.volume(), measured.volume_error()))
        })
        .collect::<Result<Vec<_>, String>>()?;
    volumes.sort_by(|one, other| one.0.total_cmp(&other.0));
    let closed_forms = [108.0, the_plates_volume(lean)];
    let [(block, block_error), (plate, plate_error)] = volumes[..] else {
        return Err(format!(
            "lean {lean:?} turned {angle}: {} solids, not two",
            volumes.len()
        ));
    };
    if (block - closed_forms[0]).abs() > block_error
        || (plate - closed_forms[1]).abs() > plate_error
    {
        return Err(format!(
            "lean {lean:?} turned {angle}: measured {block} ± {block_error} and {plate} ± \
             {plate_error}, not within {closed_forms:?}"
        ));
    }
    Ok(())
}

/// The plate's volume: the square's `200`, and the tab's, less the quarter of
/// the tab inside the square. For a lean `(a, b, 1)` the cap is `z = 1 + 3a −
/// au − bv`, with `u` and `v` the steps from the bar's axis, so the tab is
/// `9π(1 + 3a)` and the quarter where `u` and `v` are both below nought is
/// `9π(1 + 3a)/4 + 9a + 9b`. Only for a lean whose cap stays inside `0 < z <
/// 2` over the tab.
fn the_plates_volume([along_x, along_y, up]: [f64; 3]) -> f64 {
    let (a, b) = (along_x / up, along_y / up);
    200.0 - 9.0 * (a + b) + 27.0 * std::f64::consts::PI / 4.0 * (1.0 + 3.0 * a)
}

/// The square united with the tab whose cap leans by `lean`, both placed by
/// `turn`, or why the union was refused.
fn a_plate_with_a_leaning_tab(
    model: &mut Model,
    turn: &Isometry,
    lean: [f64; 3],
) -> Result<SolidKey, String> {
    let square = a_block(model, [0.0, 0.0, 0.0], [10.0, 10.0, 2.0]);
    let flat = Plane::from_point_and_normal(Point::origin(), Vector::new(0.0, 0.0, -1.0))
        .expect("a unit normal");
    let leaning = Plane::from_point_and_normal(
        Point::new(3.0, 0.0, 1.0),
        Vector::new(lean[0], lean[1], lean[2]),
    )
    .expect("a normal with a length");
    let tab = model
        .add_bar_cut_off_by(
            3.0,
            [flat, leaning],
            &Isometry::translation(Vector::new(10.0, 10.0, 0.0)).expect("a finite shift"),
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map_err(|why| format!("the tab: {why:?}"))?;
    model
        .place_solid(square, turn)
        .expect("this model made the key");
    model
        .place_solid(tab, turn)
        .expect("this model made the key");
    let fused = model
        .fuse(
            &[square],
            &[tab],
            Rule::Union,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .map_err(|why| format!("the plate: {why:?}"))?;
    let [plate] = fused.solids() else {
        return Err(format!("the plate: {} solids", fused.solids().len()));
    };
    Ok(*plate)
}

fn a_block(model: &mut Model, [x0, y0, z0]: [f64; 3], [x1, y1, z1]: [f64; 3]) -> SolidKey {
    let rectangle = ProfileBuilder::new(vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a rectangle is a profile");
    model
        .add_extrusion(
            &rectangle,
            z1 - z0,
            &Isometry::translation(Vector::new(0.0, 0.0, z0)).expect("a finite shift"),
        )
        .expect("a rectangle extrudes")
}
