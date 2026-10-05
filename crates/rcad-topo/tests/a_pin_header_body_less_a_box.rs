//! The body of KiCad's `PinHeader_1x11_P2.54mm_Vertical.step`, without its
//! pins, less a box over the middle of it and intersected with that box.
//!
//! The body is a bar `2.54` wide and `2.54` tall, one pitch of `2.54` per pin,
//! with its four corners chamfered by `0.254` and a V notch `0.254` deep in
//! both long walls between each pair of pins. Every chamfer is at 45°, so the
//! plane of a notch's chamfer on one wall, carried on across the body, meets
//! the other wall exactly at a corner of a notch there: `x + y = −0.254`
//! through `(1.27, −1.524)` and `(−1.27, 1.016)`. Every face is cut by every
//! plane of its own solid, and over the planes the body stores such a line
//! misses the corner by a rounding, so the cut leaves pieces with corners a
//! rounding apart (`docs/decisions.md` §365.7, queue row Q1019).
//!
//! Three things stopped this fuse, each pinned here. A piece of the bottom
//! face beside the top right chamfer, with two such corners, was refused `a
//! point inside a piece of a face`: no candidate was proved inside it, and it
//! shares no side with a piece of the box. It is placed now by its corner
//! `(1.016, 1.27)`, outside the box. A side between two meets that round to
//! one corner then panicked as a side with no length, and is left out now.
//! And with those sides gone, a flat piece's wire that walks each of its edges
//! out and back again, enclosing nothing, left four faces along one edge and
//! the fuse refused `ResultIsNotManifold`; such a wire is left out now too.
//!
//! The file's own planes stop the fuse of the whole file on a strip of the
//! wall `x = −1.27` instead, which holds no `f64` point at all; §365.7 has it.
//!
//! The box is the one Q690's cut takes out of the whole file's first solid:
//! half the solid's box in each direction, centred in it. That solid has the
//! pins too, so its box runs from `z = −3` to `8.54` and the box here runs
//! right through the body from top to bottom.

use rcad_geom::{Deflection, Isometry, Scale, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

/// The body less the box is one solid, the intersection is one too, the two
/// together hold the body, and the intersection holds the box's own block of
/// the body.
#[test]
fn a_pin_header_body_less_a_box_and_with_it_hold_the_body() {
    let (mut model, body, knife) = a_pin_header_body_and_a_box(11);
    let whole = volume_of_each(&model, &[body]);
    let less = fused(&mut model, body, knife, Rule::Difference);
    assert_eq!(less.len(), 1, "the body less the box is one solid");
    let less = volume_of_each(&model, &less);

    let (mut model, body, knife) = a_pin_header_body_and_a_box(11);
    let both = fused(&mut model, body, knife, Rule::Intersection);
    assert_eq!(both.len(), 1, "the body with the box is one solid");
    let both = volume_of_each(&model, &both);

    let (sum, error) = (less.0 + both.0, less.1 + both.1 + whole.1);
    assert!(
        (sum - whole.0).abs() <= error,
        "the body holds {} and the two answers {sum} give or take {error}",
        whole.0
    );

    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let block = a_box(
        &mut model,
        the_box_q690_takes_out(),
        [None, None, Some([0.0, 2.54])],
    );
    let block = volume_of_each(&model, &[block]);
    assert!(
        (both.0 - block.0).abs() <= both.1 + block.1,
        "the intersection holds {} and the box's block of the body {} give or take {}",
        both.0,
        block.0,
        both.1 + block.1
    );
}

/// A model holding the body with `pins` pins, and the box over its middle.
fn a_pin_header_body_and_a_box(pins: i32) -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
    let outline = ProfileBuilder::new(the_body_outline(pins))
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the notched outline is a profile");
    let body = model
        .add_extrusion(&outline, 2.54, &Isometry::identity())
        .expect("the body stands 2.54 tall");
    let knife = a_box(&mut model, the_box_q690_takes_out(), [None, None, None]);
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("the body and the box are sound");
    (model, body, knife)
}

/// The body's outline, anticlockwise from above, in the numbers the file
/// writes: thousandths of a millimetre over a thousand, which is the `f64`
/// nearest each decimal the file holds.
fn the_body_outline(pins: i32) -> Vec<[f64; 2]> {
    let at = |x: i32, y: i32| [f64::from(x) / 1000.0, f64::from(y) / 1000.0];
    let pitch = 2540;
    let last = pins - 1;
    let mut outline = vec![at(1016, 1270), at(-1016, 1270)];
    for pin in 0..pins {
        let centre = -pitch * pin;
        outline.extend([
            at(-1270, centre + 1016),
            at(-1270, centre - 1016),
            at(-1016, centre - 1270),
        ]);
    }
    outline.push(at(1016, -pitch * last - 1270));
    for pin in (0..pins).rev() {
        let centre = -pitch * pin;
        outline.extend([at(1270, centre - 1016), at(1270, centre + 1016)]);
        if pin > 0 {
            outline.push(at(1016, centre + 1270));
        }
    }
    outline
}

/// The low and high corners of the box Q690's cut takes out of the file's
/// first solid, worked out as that cut works them out from the solid's box,
/// `(−1.27, −26.67, −3)` to `(1.27, 1.27, 8.54)`.
fn the_box_q690_takes_out() -> [[f64; 2]; 3] {
    let (low, high) = ([-1.27, -26.67, -3.0], [1.27, 1.27, 8.54]);
    [0, 1, 2].map(|axis| {
        let size = (high[axis] - low[axis]) * 0.5;
        let from = low[axis] + size * 0.5;
        [from, from + size]
    })
}

/// A box over `spans`, with any span `cut_to` names taken in its place.
fn a_box(model: &mut Model, spans: [[f64; 2]; 3], cut_to: [Option<[f64; 2]>; 3]) -> SolidKey {
    let [x, y, z] = [0, 1, 2].map(|axis| cut_to[axis].unwrap_or(spans[axis]));
    let rectangle = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [x[1] - x[0], 0.0],
        [x[1] - x[0], y[1] - y[0]],
        [0.0, y[1] - y[0]],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a rectangle is a profile");
    let corner = Isometry::translation(Vector::new(x[0], y[0], z[0])).expect("a shift");
    model
        .add_extrusion(&rectangle, z[1] - z[0], &corner)
        .expect("the box has a height")
}

fn fused(model: &mut Model, body: SolidKey, knife: SolidKey, rule: Rule) -> Vec<SolidKey> {
    let solids = model
        .fuse(
            &[body],
            &[knife],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("{rule:?} refused: {why:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("{rule:?} left a model that does not validate: {why}"));
    solids
}

/// The volume the solids hold together, and how far off it may be.
fn volume_of_each(model: &Model, solids: &[SolidKey]) -> (f64, f64) {
    let (mut volume, mut error) = (0.0, 0.0);
    for &solid in solids {
        let measure = model
            .solid(solid)
            .expect("this model made the key")
            .measure(
                Deflection::of(0.001).expect("a fine mesh"),
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            )
            .unwrap_or_else(|why| panic!("a solid does not measure: {why}"));
        volume += measure.volume();
        error += measure.volume_error();
    }
    (volume, error)
}
