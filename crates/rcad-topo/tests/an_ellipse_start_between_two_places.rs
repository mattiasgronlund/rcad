//! Probe.

use rcad_geom::{
    Axis, Isometry, MeetOfThree, Point, Scale, Surface, SurfaceMeet, Vector, intersect,
    meet_of_three_surfaces, where_a_closed_crossing_starts, which_comes_first_round_a_crossing,
};
use rcad_predicates::{Decision, Point as PredicatePoint, Policy, ProvenanceLog};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

const ALONG_A_CROSSING: &str = "order along a crossing";
const EVERY_RULE: [Rule; 3] = [Rule::Union, Rule::Intersection, Rule::Difference];

#[test]
fn probe() {
    let (model, bar, block) = the_bar_and_the_block();
    let surfaces = |key: SolidKey| -> Vec<Surface> {
        model
            .solid(key)
            .expect("made here")
            .faces()
            .map(|face| face.surface())
            .collect()
    };
    let bar_surfaces = surfaces(bar);
    let block_surfaces = surfaces(block);
    println!("bar: {bar_surfaces:#?}");
    println!("block: {block_surfaces:#?}");
    let wall = *bar_surfaces
        .iter()
        .find(|surface| matches!(surface, Surface::Cylinder(_)))
        .expect("a wall");
    let mut top = None;
    let mut side = None;
    for surface in &block_surfaces {
        if let Surface::Plane(flat) = surface {
            let normal = flat.normal().components();
            if normal[0] == -1.0 {
                side = Some(*surface);
            }
            if normal[2] > 0.5 {
                top = Some(*surface);
            }
        }
    }
    let (top, side) = (top.expect("a top"), side.expect("a side"));
    println!("top {top:?}\nside {side:?}");
    for pair in [[&top, &wall], [&wall, &top]] {
        let SurfaceMeet::Curves(runs) = intersect(
            pair[0],
            pair[1],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("they cross") else {
            panic!("not curves")
        };
        println!("runs: {runs:?}");
        let from = where_a_closed_crossing_starts(&runs[0], pair, &mut ProvenanceLog::new())
            .expect("a start");
        println!("start {:?}", from.coordinates());
        let MeetOfThree::At(places) = meet_of_three_surfaces(
            [&side, &top, &wall],
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        )
        .expect("meet") else {
            panic!("no meet")
        };
        for place in &places {
            println!("place {:?}", place.enclosure());
        }
        let from = PredicatePoint::from(from.coordinates());
        let sweep = which_comes_first_round_a_crossing(pair, from, places[0], places[1]);
        println!("sweep {sweep:?}");
    }
    for policy in [Policy::certified(), Policy::tolerant(25.0)] {
        for rule in EVERY_RULE {
            for bar_first in [true, false] {
                let (mut model, bar, block) = the_bar_and_the_block();
                let mut log = ProvenanceLog::new();
                let fused = if bar_first {
                    model.fuse(&[bar], &[block], rule, &policy, &mut log)
                } else {
                    model.fuse(&[block], &[bar], rule, &policy, &mut log)
                };
                println!(
                    "{policy:?} {rule:?} bar first {bar_first}: {:?} records {:?}",
                    fused.map(|fused| fused.solids().len()),
                    log.records()
                        .iter()
                        .map(|record| (record.predicate(), record.level(), record.enclosure()))
                        .collect::<Vec<_>>()
                );
                let _ = ALONG_A_CROSSING;
            }
        }
    }
    let _ = Decision::Zero;
}

fn the_bar_and_the_block() -> (Model, SolidKey, SolidKey) {
    let mut model = Model::new(Scale::millimetres(25.0).expect("25 mm is a working size"));
    let bar = model
        .add_round_bar(
            1.0 + f64::powi(2.0, -51),
            10.0,
            &Isometry::translation(Vector::new(-16.0, 0.0, 0.0)).expect("a shift"),
        )
        .expect("a bar");
    let square = ProfileBuilder::new(vec![
        [-17.0, -6.0],
        [-13.0, -6.0],
        [-13.0, 6.0],
        [-17.0, 6.0],
    ])
    .build(&Policy::certified(), &mut ProvenanceLog::new())
    .expect("a square");
    let placement = Isometry::translation(Vector::new(0.0, 0.0, -2.0))
        .expect("a shift")
        .compose(&Isometry::rotation(&Axis::x(), -0.5).expect("a turn"));
    let block = model
        .add_extrusion(&square, 4.0, &placement)
        .expect("a block");
    let _ = Point::origin();
    (model, bar, block)
}
