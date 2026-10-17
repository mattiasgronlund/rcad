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
//! off parallel to the bar's axis, three to four from it, and before row
//! Q2156 the triple of a side, the pipe and the cone was refused, with the
//! block placed by the bar's own turn as well as by two halves of it: the band
//! road gives a plane that steep no outer edge. The road for a plane beside a
//! pipe answers it now, and the second test pins six across. Ten across and
//! more, the fuse answered already.
//!
//! A block whose side runs one and a half from the bar's axis cuts a D-flat
//! along the whole bar. Placed by the bar's turn, the side is a rounding off
//! parallel to the axis and cuts both circles the pipe and the cone share,
//! and before row Q2177 no road took a plane that cuts them and leans to the
//! axis they share. The third test pins the four points that triple meets at,
//! and the refusal the difference still gets: the fuse reads the side and the
//! pipe as straight lines, where they cross in an ellipse.
//!
//! A cone in a solid of its own beside the bar does not reach that question:
//! the fuse then asks where the pipe and the cone cross first.
//!
//! A block whose top is `z = 1`, neither solid placed, holds the circle the
//! pipe and the cone share in its top. The pipe and the cone each name the
//! side along that circle by the block's top since row Q2179, the name the
//! top's own face gives it, so the circle is one edge; the fourth test pins
//! the difference, the union and the intersection.
//!
//! What is left is not measured: the bar alone, before any fuse, is refused
//! `DeflectionBelowWhatAMeshCanReach` with `closest` about `242259` at every
//! deflection asked. Unplaced, or turned about its own axis, it measures, and a
//! bar of two pipes split at the same circle is refused the same way: row
//! Q566's fault.

use rcad_geom::{
    Axis, Cone, Cylinder, EdgeRun, Isometry, MeetOfThree, Orientation, Plane, Point, Scale,
    Surface, Vector, meet_of_three_surfaces,
};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_topo::{
    DraftEdge, DraftStep, DraftWire, Error, Model, ProfileBuilder, Rule, SolidBuilder, SolidKey,
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

    assert_eq!(
        what_the_bar_less_the_block_is_made_of(&mut model, bar, block),
        (1, 1, 2),
        "the chamfer, the pipe, the bar's top and the block's top"
    );
}

/// The block six across, whose sides run three from the bar's axis and a
/// rounding off parallel to it, placed by the bar's own turn and by two halves
/// of it. Each side cuts the pipe no nearer than about `10¹⁶` along it, where
/// the cone is that wide, so the side, the pipe and the cone share nothing; before row
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
        assert_eq!(
            what_the_bar_less_the_block_is_made_of(&mut model, bar, block),
            (1, 1, 2),
            "the chamfer, the pipe, the bar's top and the block's top"
        );
    }
}

/// The block from `x = 1.5` out past the bar, taller and wider than it, placed
/// by the bar's own turn. Its side `x = 1.5` runs a rounding off parallel to
/// the bar's axis and cuts both circles of radius two the pipe and the cone
/// share, the one at `z = 1` and the one at `z = −3` past the apex, at
/// `y = ±√1.75`. Before row Q2177 the triple of that side, the pipe and the
/// cone was refused `WhereThreeSurfacesMeetNeedsTwoOfThemFlat`; it is the four
/// points, each a rounding off where the turn puts them.
///
/// The difference, a D-flat along the whole bar, is still refused, by
/// `ACrossingThisCannotBeOrderedAlong`: the fuse reads the side and the pipe as
/// a plane along a pipe, which it walks as two straight lines, where they cross
/// in an ellipse about `10¹⁶` long. The row filed from Q2177 for that turns
/// this into `(1, 1, 3)`: the chamfer, the pipe, the bar's bottom, its top and
/// the flat.
#[test]
fn a_block_beside_a_chamfered_bars_axis_meets_both_its_circles_and_its_flat_is_refused() {
    let axis =
        Axis::new(Point::new(1.0, 0.0, -1.0), Vector::new(1.0, 2.0, 3.0)).expect("a direction");
    let turn = Isometry::rotation(&axis, 1.1).expect("a finite angle");

    let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
    let bar = a_chamfered_bar(&mut model);
    model
        .place_solid(bar, &turn)
        .expect("this model made the key");
    let block = a_block_from_x_one_and_a_half(&mut model);
    model
        .place_solid(block, &turn)
        .expect("this model made the key");

    let [side, pipe, cone] = the_side_the_pipe_and_the_cone(&model, bar, block, &turn);
    let answer = meet_of_three_surfaces(
        [&side, &pipe, &cone],
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    )
    .expect("the side, the pipe and the cone are answered");
    let MeetOfThree::At(points) = &answer else {
        panic!("expected four points, got {answer:?}");
    };
    assert_eq!(points.len(), 4, "{answer:?}");
    let out = 1.75_f64.sqrt();
    for unplaced in [
        [1.5, out, -3.0],
        [1.5, -out, -3.0],
        [1.5, out, 1.0],
        [1.5, -out, 1.0],
    ] {
        let placed = turn.apply_to_point(Point::new(unplaced[0], unplaced[1], unplaced[2]));
        let wanted = [placed.x(), placed.y(), placed.z()];
        assert!(
            points.iter().any(|point| {
                let boxed = point.enclosure();
                (0..3).all(|axis| {
                    let middle = (boxed[axis].lo() + boxed[axis].hi()) / 2.0;
                    (middle - wanted[axis]).abs() < 1e-13 && boxed[axis].width() < 1e-13
                })
            }),
            "no narrow box round {wanted:?}, {unplaced:?} turned, in {answer:?}"
        );
    }

    let refused = model.fuse(
        &[bar],
        &[block],
        Rule::Difference,
        &Policy::certified(),
        &mut ProvenanceLog::new(),
    );
    assert!(
        matches!(refused, Err(Error::ACrossingThisCannotBeOrderedAlong)),
        "{refused:?}"
    );
}

/// The block sixty across from `z = −2` up to `z = 1`, neither solid placed, so
/// the block's top holds the circle where the bar's cone meets its pipe. Less
/// the block, the bar is the pipe from `z = 1` up, closed by a disc there; with
/// it, the block with the pipe standing on it; and the two share the cone
/// closed by two discs. Before row Q2179 the difference and the union were
/// refused `ResultIsNotManifold { count: 1 }`: the pipe kept the circle as the
/// bar's own edge, and the block's top named it a new crossing of the top and
/// the pipe, so one circle was two edges with one face along each.
#[test]
fn a_block_whose_top_holds_the_circle_where_the_cone_meets_the_pipe_cuts_the_bar_there() {
    for (rule, wanted) in [
        (Rule::Difference, (0, 1, 2)),
        (Rule::Union, (0, 1, 7)),
        (Rule::Intersection, (1, 0, 2)),
    ] {
        let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
        let bar = a_chamfered_bar(&mut model);
        let block = a_block_from_below(&mut model, 60.0, 1.0);
        assert_eq!(
            what_the_fuse_is_made_of(&mut model, bar, block, rule),
            wanted,
            "the cones, pipes and planes {rule:?} has faces on"
        );
    }
}

#[test]
fn probe_q2181() {
    let axis =
        Axis::new(Point::new(1.0, 0.0, -1.0), Vector::new(1.0, 2.0, 3.0)).expect("a direction");
    let turn = Isometry::rotation(&axis, 1.1).expect("a finite angle");
    let half = Isometry::rotation(&axis, 0.55).expect("a finite angle");
    for turns in [vec![turn], vec![half, half]] {
        for rule in [Rule::Difference, Rule::Union, Rule::Intersection] {
            let mut model = Model::new(Scale::millimetres(100.0).expect("a working size"));
            let bar = a_chamfered_bar(&mut model);
            model.place_solid(bar, &turn).expect("key");
            let block = a_block_from_below(&mut model, 60.0, 1.0);
            for placed in &turns {
                model.place_solid(block, placed).expect("key");
            }
            if rule == Rule::Difference {
                let circle_plane = Surface::Plane(level(1.0, 1.0).transformed(&turn));
                println!("PROBE circle plane moved {circle_plane:?}");
                let bars: Vec<Surface> = model
                    .solid(bar)
                    .expect("k")
                    .faces()
                    .map(|f| f.surface())
                    .collect();
                let blocks: Vec<Surface> = model
                    .solid(block)
                    .expect("k")
                    .faces()
                    .map(|f| f.surface())
                    .collect();
                for s in &bars {
                    println!("PROBE bar {s:?}");
                }
                for s in &blocks {
                    println!("PROBE block {s:?}");
                }
                let pipe = bars[2];
                let cone = bars[1];
                let top = *blocks
                    .iter()
                    .find(|s| {
                        let Surface::Plane(p) = s else { return false };
                        let n = p.normal().components();
                        let c = circle_plane.as_plane().unwrap().normal().components();
                        (0..3).map(|i| n[i] * c[i]).sum::<f64>() > 0.999
                    })
                    .expect("top");
                if let (Surface::Plane(t), Surface::Plane(c)) = (top, circle_plane) {
                    println!(
                        "PROBE top bits {:?} {:x} circle bits {:?} {:x}",
                        t.normal().components().map(f64::to_bits),
                        t.offset().to_bits(),
                        c.normal().components().map(f64::to_bits),
                        c.offset().to_bits()
                    );
                }
                let pol = Policy::certified();
                println!(
                    "PROBE meet3 top pipe cone {:?}",
                    meet_of_three_surfaces([&top, &pipe, &cone], &pol, &mut ProvenanceLog::new())
                );
                println!(
                    "PROBE meet3 circleplane pipe cone {:?}",
                    meet_of_three_surfaces(
                        [&circle_plane, &pipe, &cone],
                        &pol,
                        &mut ProvenanceLog::new()
                    )
                );
                println!(
                    "PROBE top x pipe {:?}",
                    rcad_geom::intersect(&top, &pipe, &pol, &mut ProvenanceLog::new())
                );
                println!(
                    "PROBE top x cone {:?}",
                    rcad_geom::intersect(&top, &cone, &pol, &mut ProvenanceLog::new())
                );
                println!(
                    "PROBE pipe x cone {:?}",
                    rcad_geom::intersect(&pipe, &cone, &pol, &mut ProvenanceLog::new())
                );
            }
            let answer = model.fuse(
                &[bar],
                &[block],
                rule,
                &Policy::certified(),
                &mut ProvenanceLog::new(),
            );
            println!(
                "PROBE turns {} {rule:?}: {:?}",
                turns.len(),
                answer.map(|f| f.solids().len())
            );
        }
    }
}

/// The block's side that was `x = 1.5` before the turn, and the bar's pipe and
/// cone, each where the turn put it.
fn the_side_the_pipe_and_the_cone(
    model: &Model,
    bar: SolidKey,
    block: SolidKey,
    turn: &Isometry,
) -> [Surface; 3] {
    let facing = turn
        .apply_to_vector(Vector::new(-1.0, 0.0, 0.0))
        .components();
    let side = model
        .solid(block)
        .expect("this model made the key")
        .faces()
        .map(|face| face.surface())
        .find(|surface| {
            let Surface::Plane(plane) = surface else {
                return false;
            };
            let normal = plane.normal().components();
            (0..3)
                .map(|which| normal[which] * facing[which])
                .sum::<f64>()
                > 0.999
        })
        .expect("the block has a side facing the bar");
    let curved = |wanted: fn(&Surface) -> bool| {
        model
            .solid(bar)
            .expect("this model made the key")
            .faces()
            .map(|face| face.surface())
            .find(wanted)
            .expect("the bar has a pipe and a cone")
    };
    let pipe = curved(|surface| matches!(surface, Surface::Cylinder(_)));
    let cone = curved(|surface| matches!(surface, Surface::Cone(_)));
    [side, pipe, cone]
}

/// The bar less the block, which is one solid: how many cones, pipes and
/// planes it has faces on.
fn what_the_bar_less_the_block_is_made_of(
    model: &mut Model,
    bar: SolidKey,
    block: SolidKey,
) -> (usize, usize, usize) {
    what_the_fuse_is_made_of(model, bar, block, Rule::Difference)
}

/// The bar and the block fused under `rule`, which is one solid: how many
/// cones, pipes and planes it has faces on.
fn what_the_fuse_is_made_of(
    model: &mut Model,
    bar: SolidKey,
    block: SolidKey,
    rule: Rule,
) -> (usize, usize, usize) {
    let solids = model
        .fuse(
            &[bar],
            &[block],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .unwrap_or_else(|why| panic!("the {rule:?} was refused: {why:?}"))
        .solids()
        .to_vec();
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .unwrap_or_else(|why| panic!("the {rule:?} does not validate: {why}"));
    assert_eq!(solids.len(), 1, "one solid is left");

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
    (cones, pipes, planes)
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
    a_block_from_below(model, across, 0.5)
}

/// The block `across` wide about the `z` axis and from `z = −2` up to
/// `z = top`, not yet placed.
fn a_block_from_below(model: &mut Model, across: f64, top: f64) -> SolidKey {
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
        .add_extrusion(&square, top + 2.0, &corner)
        .expect("the block has a height")
}

/// The block from `x = 1.5` to `x = 11.5`, `y = −5` to `y = 5` and `z = −2`
/// to `z = 6`, not yet placed.
fn a_block_from_x_one_and_a_half(model: &mut Model) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square is a profile");
    let corner = Isometry::translation(Vector::new(1.5, -5.0, -2.0)).expect("a shift");
    model
        .add_extrusion(&square, 8.0, &corner)
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
