//! Probe.

use std::{env, fs, path::PathBuf};

use rcad_geom::{Isometry, Vector};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};
use rcad_topo::{Model, ProfileBuilder, Rule, SolidKey};

const LED: &str = "LED_D5.0mm_Horizontal_O6.35mm_Z3.0mm.step";

#[test]
fn probe_the_led() {
    let (led, solid) = the_led();
    for face in led.solid(solid).unwrap().faces() {
        println!("face {:?} {:?}", face.key(), face.surface());
        for wire in face.wires() {
            for coedge in wire.coedges() {
                println!(
                    "   edge {:?} {:?} -> {:?} run {:?}",
                    coedge.edge().key(),
                    coedge.start().point().coordinates(),
                    coedge.end().point().coordinates(),
                    coedge.edge().run()
                );
            }
        }
    }
    for rule in [Rule::Intersection, Rule::Difference] {
        let mut model = led.clone();
        let knife = a_box(&mut model, [10.0; 3], 1.0);
        let said = model.fuse(
            &[solid],
            &[knife],
            rule,
            &Policy::certified(),
            &mut ProvenanceLog::new(),
        );
        println!("{rule:?}: {:?}", said.map(|f| f.solids().len()));
    }
}

fn the_led() -> (Model, SolidKey) {
    let path =
        PathBuf::from(env::var_os("RCAD_STEP_CORPUS").expect("`just t` sets RCAD_STEP_CORPUS"))
            .join("kicad")
            .join(LED);
    let bytes = fs::read(&path).expect("the LED is in the corpus");
    let file = StepFile::read(&bytes).expect("it parses");
    let import = Import::read(&file, &Policy::certified(), &mut ProvenanceLog::new())
        .expect("the LED imports");
    let [(_, solid)] = import.solids()[..] else {
        panic!("the LED is one solid, not {}", import.solids().len())
    };
    (import.model().clone(), solid)
}

fn a_box(model: &mut Model, low: [f64; 3], side: f64) -> SolidKey {
    let square = ProfileBuilder::new(vec![[0.0, 0.0], [side, 0.0], [side, side], [0.0, side]])
        .build(&Policy::certified(), &mut ProvenanceLog::new())
        .expect("a square");
    model
        .add_extrusion(
            &square,
            side,
            &Isometry::translation(Vector::new(low[0], low[1], low[2])).expect("a finite shift"),
        )
        .expect("a box")
}
