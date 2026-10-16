//! A bar with a chamfer, cut by a block placed by another composition of turns.
//!
//! The bar is built face by face: a cone from `z = 0`, radius one, to `z = 1`,
//! radius two, then a pipe of radius two up to `z = 5`. It is placed once by
//! the turn of `1.1` about the line through `(1, 0, −1)` along `(1, 2, 3)`. The
//! block runs from `z = −2` up to `z = ½` and is placed twice by half that
//! turn. [`Model::place_solid`] composes each placement onto the last, and
//! [`Isometry::compose`] multiplies two turns as quaternions, which rounds, so
//! the block's top comes out a rounding off square to the bar's axis.
//!
//! The fuse asks where the block's top meets the pipe and the cone, which
//! share the circle at `z = 1`. On paper the top cuts the cone in a circle of
//! radius one and a half and the pipe in one of radius two, so the three share
//! nothing. Off square by a rounding, each cut is an ellipse, and before row
//! Q2127 no road in `three_surfaces.rs` took a plane leaning to a pipe's or a
//! cone's axis, so the difference was refused
//! `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`. Each ellipse keeps within a
//! rounding of its circle, so the two keep apart and the three share nothing.
//!
//! The block is sixty across. Six to eight across, its sides run a rounding
//! off parallel to the bar's axis, three to four from it, and the triple of a
//! side, the pipe and the cone is still refused, with the block placed by the
//! bar's own turn as well as by two halves of it: the band road gives a plane
//! that steep no outer edge. Ten across and more, the fuse answers.
//!
//! A cone in a solid of its own beside the bar does not reach that question:
//! the fuse then asks where the pipe and the cone cross first.
//!
//! What is left is not measured: the bar alone, before any fuse, is refused
//! `DeflectionBelowWhatAMeshCanReach` with `closest` about `242259` at every
//! deflection asked. Unplaced, or turned about its own axis, it measures, and a
//! bar of two pipes split at the same circle is refused the same way: row
//! Q566's fault.

use rcad_geom::{
    Axis, Cone, Cylinder, EdgeRun, Isometry, Orientation, Plane, Point, Scale, Surface, Vector,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftStep, DraftWire, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey,
};

#[test]
fn a_chamfered_bar_less_a_block_placed_twice_keeps_the_bar_above_the_blocks_top() {
    let axis =
        Axis::new(Point::new(1.0, 0.0, -1.0), Vector::new(1.0, 2.0, 3.0)).expect("a direction");
    let turn = Isometry::rotation(&axis, 1.1).expect("a finite angle");
    let half = Isometry::rotation(&axis, 0.55).expect("a finite angle");

    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let bar = a_chamfered_bar(&mut model);
    model
        .place_solid(bar, &turn)
        .expect("this model made the key");
    let block = a_block_below_half(&mut model, 60.0);
    model
        .place_solid(block, &half)
        .expect("this model made the key");
    model
        .place_solid(block, &half)
        .expect("this model made the key");

    assert!(
        the_blocks_top_is_off_square_to_the_bars_axis(&model, bar, block),
        "the block placed twice came out exactly square to the bar after all"
    );

    the_bar_is_cut_at_the_blocks_top(&mut model, bar, block);
}

/// The block six across, whose sides run three from the bar's axis and a
/// rounding off parallel to it, placed by the bar's own turn and by two halves
/// of it. Each side cuts the pipe only some `10¹⁵` along it, where the cone
/// is that wide, so the side, the pipe and the cone share nothing; before row
/// Q2156 that was refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`.
#[test]
fn a_chamfered_bar_less_a_block_six_across_keeps_the_bar_above_the_blocks_top() {
    let axis =
        Axis::new(Point::new(1.0, 0.0, -1.0), Vector::new(1.0, 2.0, 3.0)).expect("a direction");
    let turn = Isometry::rotation(&axis, 1.1).expect("a finite angle");
    let half = Isometry::rotation(&axis, 0.55).expect("a finite angle");

    for turns in [vec![turn], vec![half, half]] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = a_chamfered_bar(&mut model);
        model
            .place_solid(bar, &turn)
            .expect("this model made the key");
        let block = a_block_below_half(&mut model, 6.0);
        for placed in &turns {
            model
                .place_solid(block, placed)
                .expect("this model made the key");
        }
        the_bar_is_cut_at_the_blocks_top(&mut model, bar, block);
    }
}

/// The bar less the block, which leaves one bar of a chamfer, a pipe, the
/// bar's top and the block's top.
fn the_bar_is_cut_at_the_blocks_top(model: &mut Model, bar: SolidKey, block: SolidKey) {
    let solids = model
        .fuse(
            &[bar],
            &[block],
            Rule::Difference,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the difference was refused: {why:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the difference does not validate: {why}"));
    assert_eq!(solids.len(), 1, "one bar is left");

    let (mut cones, mut pipes, mut planes) = (0, 0, 0);
    for face in model
        .solid(solids[0])
        .expect("this model made the key")
        .faces()
    {
        let surface = face.surface();
        if let Surface::Cone(_) = surface {
            cones += 1;
        } else if let Surface::Cylinder(_) = surface {
            pipes += 1;
        } else if let Surface::Plane(_) = surface {
            planes += 1;
        } else {
            panic!("a face the bar and the block do not have: {surface:?}");
        }
    }
    assert_eq!(
        (cones, pipes, planes),
        (1, 1, 2),
        "the chamfer, the pipe, the bar's top and the block's top"
    );
}

/// The block's top face, `z = ½` before it was placed, against the bar's pipe:
/// whether the normal and the axis differ in some component, up to which way
/// each points.
fn the_blocks_top_is_off_square_to_the_bars_axis(
    model: &Model,
    bar: SolidKey,
    block: SolidKey,
) -> bool {
    let axis = model
        .solid(bar)
        .expect("this model made the key")
        .faces()
        .find_map(|face| {
            let Surface::Cylinder(pipe) = face.surface() else {
                return None;
            };
            Some(pipe.axis().direction().components())
        })
        .expect("the bar has a pipe");
    let tops: Vec<[f64; 3]> = model
        .solid(block)
        .expect("this model made the key")
        .faces()
        .filter_map(|face| {
            let Surface::Plane(plane) = face.surface() else {
                return None;
            };
            Some(plane.normal().components())
        })
        .filter(|normal| {
            let along: f64 = (0..3).map(|which| normal[which] * axis[which]).sum();
            along.abs() > 0.5
        })
        .collect();
    assert_eq!(tops.len(), 2, "the block has a top and a bottom: {tops:?}");
    tops.iter()
        .any(|normal| (0..3).any(|which| normal[which].abs() != axis[which].abs()))
}

/// The block `across` wide about the `z` axis and from `z = −2` up to
/// `z = ½`, not yet placed.
fn a_block_below_half(model: &mut Model, across: f64) -> SolidKey {
    let square = ProfileBuilder::new(vec![
        [0.0, 0.0],
        [across, 0.0],
        [across, across],
        [0.0, across],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square is a profile");
    let corner =
        Isometry::translation(Vector::new(-across / 2.0, -across / 2.0, -2.0)).expect("a shift");
    model
        .add_extrusion(&square, 2.5, &corner)
        .expect("the block has a height")
}

/// The bar, not yet placed: the cone of slope one with its apex at `z = −1`
/// from `z = 0` to `z = 1`, and the pipe of radius two from `z = 1` to `z = 5`,
/// each with its seam on the ruling through `y = 0`, `x > 0`.
fn a_chamfered_bar(model: &mut Model) -> SolidKey {
    let chamfer = Cone::new(
        Axis::new(Point::new(0.0, 0.0, -1.0), Vector::new(0.0, 0.0, 1.0)).expect("a direction"),
        1.0,
    )
    .expect("a positive slope");
    let wall = Cylinder::new(Axis::z(), 2.0).expect("a positive radius");
    let mut builder = SolidBuilder::new();

    let corners = [(1.0, 0.0), (2.0, 1.0), (2.0, 5.0)].map(|(x, z)| {
        builder
            .add_vertex(Point::new(x, 0.0, z))
            .expect("a finite point")
    });
    let seams = [0, 1].map(|which| {
        builder
            .add_edge(corners[which], corners[which + 1], EdgeRun::Straight)
            .expect("a ruling is straight")
    });
    let bottom = builder
        .add_edge(
            corners[0],
            corners[0],
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("the bottom cuts the cone in one closed curve");
    let middle = builder
        .add_edge(
            corners[1],
            corners[1],
            EdgeRun::RightRoundWhereItCrosses {
                and: Surface::Plane(level(1.0, 1.0)),
            },
        )
        .expect("the pipe and the cone cross in the circle at z = 1");
    let top = builder
        .add_edge(
            corners[2],
            corners[2],
            EdgeRun::RightRoundWhereItsFacesCross,
        )
        .expect("the top cuts the pipe in one closed curve");

    let underneath = wire(&mut builder, &[(bottom, Orientation::Reversed)]);
    let round_the_chamfer = wire(
        &mut builder,
        &[
            (bottom, Orientation::Forward),
            (seams[0], Orientation::Forward),
            (middle, Orientation::Reversed),
            (seams[0], Orientation::Reversed),
        ],
    );
    let round_the_wall = wire(
        &mut builder,
        &[
            (middle, Orientation::Forward),
            (seams[1], Orientation::Forward),
            (top, Orientation::Reversed),
            (seams[1], Orientation::Reversed),
        ],
    );
    let on_top = wire(&mut builder, &[(top, Orientation::Forward)]);

    let faces = [
        (Surface::Plane(level(0.0, -1.0)), underneath),
        (Surface::Cone(chamfer), round_the_chamfer),
        (Surface::Cylinder(wall), round_the_wall),
        (Surface::Plane(level(5.0, 1.0)), on_top),
    ]
    .map(|(surface, walk)| {
        (
            builder
                .add_face(surface, &[walk])
                .expect("a face trimmed by its walk"),
            Orientation::Forward,
        )
    });
    builder.add_shell(&faces).expect("four faces close");
    builder
        .build(model, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("a chamfered bar described this way is a solid")
}

/// The wire that walks `steps`, each edge the way it names.
fn wire(builder: &mut SolidBuilder, steps: &[(DraftEdge, Orientation)]) -> DraftWire {
    let steps: Vec<DraftStep> = steps
        .iter()
        .map(|&(edge, turn)| {
            DraftStep::Along(
                builder
                    .add_coedge(edge, turn)
                    .expect("an edge of this build"),
            )
        })
        .collect();
    builder.add_wire(&steps).expect("a closed walk is a wire")
}

/// The plane `z = at`, facing `+z` where `outward` is one and `−z` where it is
/// minus one.
fn level(at: f64, outward: f64) -> Plane {
    Plane::from_point_and_normal(Point::new(0.0, 0.0, at), Vector::new(0.0, 0.0, outward))
        .expect("the z axis is a unit normal")
}
