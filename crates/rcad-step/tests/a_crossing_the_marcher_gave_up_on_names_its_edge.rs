//! A file the marcher gives up on names the edge it gave up along, not the
//! whole solid.
//!
//! `docs/decisions.md` §109.1 counted it: of the 44 KiCad files whose only
//! refusal is "this crossing runs where its two surfaces are too nearly
//! parallel to follow it", **all 44** published
//! `TheSolidTheFileDescribesIsNotOne { solid: #15 }` — the `MANIFOLD_SOLID_BREP`
//! and nothing narrower. Every question of the shape "which two surfaces were
//! too nearly parallel here" was therefore being asked of a message that does
//! not carry the answer, and a whole census of tangency ran on no pair at all.
//!
//! An `EDGE_CURVE` is the address that answers it: the marcher was following
//! the crossing one edge runs along, and the two `ADVANCED_FACE`s that walk
//! that edge are the pair it gave up between.
//!
//! # What stops this passing for the wrong reason
//!
//! The `#15` form is what it published before, so an assertion that the
//! refusal merely *is* a refusal would have been green throughout. This asserts
//! the positive shape instead — a named entity, whose keyword is `EDGE_CURVE`,
//! walked by exactly two faces — and each of those three was false before the
//! change. The pair count is the one that matters: an `#id` naming something
//! no two faces share would name a pair no better than `#15` does.

use std::path::PathBuf;
use std::{env, fs};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{EntityId, Import, Refusal, StepFile, Value};

/// Files from the §107.1 set of 44, picked across three different writers'
/// shapes so one file's quirk cannot carry the test.
const GIVE_UP_ON_A_CROSSING: [&str; 3] = [
    "kicad/LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step",
    "kicad/Fuseholder_Cylinder-5x20mm_Stelvio-Kontek_PTF78_Horizontal_Open.step",
    "kicad/Relay_SPST_Omron_G5NB.step",
];

#[test]
fn a_crossing_the_marcher_gave_up_on_names_the_edge_it_was_following() {
    let root = corpus();
    for name in GIVE_UP_ON_A_CROSSING {
        let bytes = fs::read(root.join(name)).unwrap_or_else(|why| {
            panic!("{name} is in the corpus this test is written against: {why}")
        });
        let file = StepFile::read(&bytes).unwrap_or_else(|why| panic!("{name} parses: {why}"));
        let mut log = ProvenanceLog::new();

        let refusal = Import::read(&file, &Policy::certified(), &mut log)
            .err()
            .unwrap_or_else(|| panic!("{name} is one of the 44 files that refuse"));

        let Refusal::ThisEntityOfTheSolidWasRefused { entity, why, .. } = &refusal else {
            panic!("{name} refused as {refusal:?}, which names no entity to read a pair from");
        };
        assert_eq!(
            keyword_of(&file, *entity),
            "EDGE_CURVE",
            "{name} refused about {entity}, and the crossing the marcher follows runs along an \
             edge: {why}"
        );

        let walked_by = faces_walking(&file, *entity);
        assert_eq!(
            walked_by.len(),
            2,
            "{name}: {entity} is walked by {walked_by:?}, and a pair is two faces"
        );
    }
}

/// Which `ADVANCED_FACE`s walk `wanted`, by the path a face reaches an edge
/// along: bounds, then the loop each bound holds, then the oriented edges of
/// that loop.
fn faces_walking(file: &StepFile, wanted: EntityId) -> Vec<EntityId> {
    let mut found = Vec::new();
    for instance in file.instances() {
        let Some(face) = instance
            .part("ADVANCED_FACE")
            .or_else(|| instance.part("FACE_SURFACE"))
        else {
            continue;
        };
        let Some(bounds) = face.attribute(1).and_then(Value::as_list) else {
            continue;
        };
        if bounds
            .iter()
            .filter_map(Value::as_reference)
            .flat_map(|bound| edges_of_bound(file, bound))
            .any(|edge| edge == wanted)
        {
            found.push(instance.id());
        }
    }
    found
}

fn edges_of_bound(file: &StepFile, bound: EntityId) -> Vec<EntityId> {
    let Some(instance) = file.instance(bound) else {
        return Vec::new();
    };
    let Some(record) = instance
        .part("FACE_OUTER_BOUND")
        .or_else(|| instance.part("FACE_BOUND"))
    else {
        return Vec::new();
    };
    let Some(loop_id) = record.attribute(1).and_then(Value::as_reference) else {
        return Vec::new();
    };
    let Some(edge_loop) = file
        .instance(loop_id)
        .and_then(|held| held.part("EDGE_LOOP"))
    else {
        return Vec::new();
    };
    let Some(steps) = edge_loop.attribute(1).and_then(Value::as_list) else {
        return Vec::new();
    };
    steps
        .iter()
        .filter_map(Value::as_reference)
        .filter_map(|step| {
            file.instance(step)
                .and_then(|held| held.part("ORIENTED_EDGE"))
                .and_then(|oriented| oriented.attribute(3))
                .and_then(Value::as_reference)
        })
        .collect()
}

fn keyword_of(file: &StepFile, id: EntityId) -> &str {
    file.instance(id)
        .and_then(|instance| instance.parts().first())
        .map_or("<unknown>", |record| record.keyword())
}

fn corpus() -> PathBuf {
    let root = env::var_os("RCAD_STEP_CORPUS").unwrap_or_else(|| {
        panic!(
            "RCAD_STEP_CORPUS is not set. Point it at a directory holding nist/ and kicad/ — this \
             test does not skip on a missing corpus."
        )
    });
    let root = PathBuf::from(root);
    assert!(
        root.is_dir(),
        "RCAD_STEP_CORPUS points at {}, which is not a directory",
        root.display()
    );
    root
}
