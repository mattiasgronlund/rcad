//! A file the marcher gives up on names the edge it gave up along, not the
//! whole solid.
//!
//! `docs/decisions.md` §109.1 counted it: of the 44 KiCad files whose only
//! refusal was "this crossing runs where its two surfaces are too nearly
//! parallel to follow it", **all 44** published
//! `TheSolidTheFileDescribesIsNotOne { solid: #15 }` — the `MANIFOLD_SOLID_BREP`
//! and nothing narrower. Every question of the shape "which two surfaces were
//! too nearly parallel here" was therefore being asked of a message that does
//! not carry the answer, and a whole census of tangency ran on no pair at all.
//!
//! An `EDGE_CURVE` is the address that answers it: the marcher was following
//! the crossing one edge runs along, and the two `ADVANCED_FACE`s that walk
//! that edge are the pair it gave up between (§113).
//!
//! # Why this names no file
//!
//! It named one for half a day. `LED_D3.0mm_Horizontal_O1.27mm_Z10.0mm.step`
//! refused at the tangency wall when §113 was written, and by the time §113 was
//! merged it did not: §112 took the cancelling subtraction out of `settled`,
//! and the file stopped reaching that wall at all. **The assertion was right and
//! its fixture had moved**, which is the failure mode a named fixture has and a
//! property does not — and §112 has more files queued to be rescued the same
//! way, so picking a survivor would only set the same trap again.
//!
//! So the claim is about the refusal and not about any file: **whichever files
//! stop at the tangency wall, every one of them names an `EDGE_CURVE`, and two
//! faces walk it.** Files come and go from that set as the arithmetic improves
//! and the test does not care.
//!
//! # What stops it passing for the wrong reason
//!
//! A property over "whichever files" is one file short of vacuous, and the day
//! the last one is rescued it would go green over an empty set forever. So it
//! counts what it selected and insists on a floor. The floor is **one** and not
//! the number standing today, because the number standing today is exactly what
//! keeps moving — it fell from 44 to a smaller set in one day — and a test that
//! pins it would go red for the good news.
//!
//! The selection is on the **geometry** the refusal bottoms out in, never on
//! whether an entity was named. Selecting on the naming would make the
//! assertion circular: a regression that stopped naming edges would empty the
//! set rather than fail the property, and the floor is what would catch it.

use std::path::{Path, PathBuf};
use std::{env, fs};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{EntityId, Import, Refusal, StepFile, Value};

#[test]
fn every_file_that_stops_at_the_tangency_wall_names_the_edge_it_was_following() {
    let root = corpus();
    let mut stopped_there = Vec::new();

    for family in ["nist", "kicad"] {
        for path in files_under(&root.join(family)) {
            let bytes =
                fs::read(&path).unwrap_or_else(|why| panic!("reading {}: {why}", path.display()));
            let Ok(file) = StepFile::read(&bytes) else {
                continue;
            };
            let mut log = ProvenanceLog::new();
            let Err(refusal) = Import::read(&file, &Policy::certified(), &mut log) else {
                continue;
            };
            if !the_tangency_wall_is_what_it_bottoms_out_in(&refusal) {
                continue;
            }
            let name = path.display().to_string();
            stopped_there.push(name.clone());

            let Refusal::ThisEntityOfTheSolidWasRefused { entity, why, .. } = &refusal else {
                panic!("{name} stopped at the tangency wall as {refusal:?}, naming no entity");
            };
            assert_eq!(
                keyword_of(&file, *entity),
                "EDGE_CURVE",
                "{name} refused about {entity}, and the crossing the marcher follows runs along \
                 an edge: {why}"
            );
            let walked_by = faces_walking(&file, *entity);
            assert_eq!(
                walked_by.len(),
                2,
                "{name}: {entity} is walked by {walked_by:?}, and a pair is two faces"
            );
        }
    }

    println!(
        "files stopping at the tangency wall: {}\n  {}",
        stopped_there.len(),
        stopped_there.join("\n  ")
    );
    assert!(
        !stopped_there.is_empty(),
        "no corpus file stops at the tangency wall any more, so this proved nothing. That is \
         either very good news — see docs/decisions.md §112 and §113 — or the reader stopped \
         reaching the marcher. Find out which, then retire this test or fix what broke."
    );
}

/// Whether the geometry this refusal bottoms out in is the tangency wall.
///
/// Two shapes reach it. `ADraftEdgeHasACrossingThatCannotBeFollowed` is the one
/// §113 built and carries the edge; a bare `Geometry` is the one that does not,
/// and selecting it too is the whole point — a refusal that stopped naming its
/// edge has to be *selected* before it can be failed.
fn the_tangency_wall_is_what_it_bottoms_out_in(refusal: &Refusal) -> bool {
    let (Refusal::ThisEntityOfTheSolidWasRefused { why, .. }
    | Refusal::TheSolidTheFileDescribesIsNotOne { why, .. }) = refusal
    else {
        return false;
    };
    if let rcad_topo::Error::ADraftEdgeHasACrossingThatCannotBeFollowed { why, .. } = &**why {
        return is_the_tangency_wall(why);
    }
    if let rcad_topo::Error::Geometry(why) = &**why {
        return is_the_tangency_wall(why);
    }
    false
}

fn is_the_tangency_wall(why: &rcad_geom::Error) -> bool {
    matches!(
        why,
        rcad_geom::Error::CannotFollowACrossingWhereItsSurfacesAreTangent { .. }
    )
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

fn files_under(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|kind| kind == "step"))
        .collect();
    found.sort();
    found
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
