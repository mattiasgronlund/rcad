//! Reads every real STEP file we could lay hands on, and says how many of each
//! source read, with its denominator.
//!
//! Two corpora, reported apart and never blended (decisions.md §17.7, and §42's
//! own spike, which found the two disagree about what a real file holds by a
//! factor of eight):
//!
//! - `nist/` — 44 files, the NIST MBE/PMI validation library, ASME Y14.5 GD&T
//!   certification parts. Four zips from `nist.gov/system/files`, reachable
//!   from the MBE PMI page; no login. NIST's own page says the files can be
//!   used without restriction. Two of the four zips hold no STEP files.
//! - `kicad/` — 323 files sampled from `gitlab.com/kicad/libraries/
//!   kicad-packages3D`, CC-BY-SA 4.0, no login. Six categories, with the 278
//!   pin-header variants thinned to every seventh so one parametric generator
//!   cannot dominate.
//!
//! Neither is in this repository: hundreds of megabytes of somebody else's
//! files do not belong in the tree, and the KiCad set is share-alike. Point
//! `RCAD_STEP_CORPUS` at a directory holding `nist/` and `kicad/` and this runs;
//! without it, this test **fails** rather than skips — green has to mean the
//! corpus ran (decisions.md §44.10).
//!
//! It is not a tautology: it counts the files it found, prints the count with
//! its denominator, and fails if either source came back empty — a corpus that
//! silently disappeared would otherwise pass here as a hundred per cent of
//! nothing.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rcad_step::{StepFile, Value};

#[test]
fn every_file_in_both_corpora_reads_and_the_rate_is_reported_per_source() {
    let root = corpus();

    let mut census: BTreeMap<String, usize> = BTreeMap::new();
    let mut deepest = 0;
    let mut worst_nesting = String::new();

    for source in ["nist", "kicad"] {
        let files = files_under(&root.join(source));
        assert!(
            !files.is_empty(),
            "{source} holds no STEP files, so this test would pass by doing nothing"
        );

        let mut read = 0;
        let mut instances = 0;
        let mut refused = Vec::new();
        let mut dangling = Vec::new();
        for path in &files {
            let bytes = fs::read(path).expect("a file the walk just listed");
            match StepFile::read(&bytes) {
                Err(refusal) => refused.push((path.clone(), refusal)),
                Ok(file) => {
                    read += 1;
                    instances += file.instances().len();
                    for (keyword, count) in file.how_many_of_each_type() {
                        *census.entry(keyword.to_string()).or_insert(0) += count;
                    }
                    for instance in file.instances() {
                        for part in instance.parts() {
                            for attribute in part.attributes() {
                                let depth = how_deep(attribute, 1);
                                if depth > deepest {
                                    deepest = depth;
                                    worst_nesting =
                                        format!("{} in {}", instance.id(), name_of(path));
                                }
                            }
                        }
                    }
                    if let Err(broken) = file.every_reference_resolves() {
                        dangling.push((
                            path.clone(),
                            file.references_that_point_nowhere().len(),
                            broken,
                        ));
                    }
                }
            }
        }

        println!(
            "{source}: {read} / {} read, {instances} instances",
            files.len()
        );
        for (path, refusal) in &refused {
            println!("  refused {}: {refusal}", name_of(path));
        }
        for (path, count, first) in &dangling {
            println!(
                "  {}: {count} references point nowhere, first is {first}",
                name_of(path)
            );
        }
        assert_eq!(
            read,
            files.len(),
            "{source}: {} of {} files were refused",
            files.len() - read,
            files.len()
        );
        assert!(
            instances > files.len(),
            "{source}: {instances} instances across {} files is too few to be real",
            files.len()
        );
    }

    println!("deepest nesting: {deepest} ({worst_nesting})");
    println!("entity types seen: {}", census.len());
    let mut by_count: Vec<_> = census.iter().collect();
    by_count.sort_by_key(|&(keyword, count)| (std::cmp::Reverse(*count), keyword));
    for (keyword, count) in by_count.iter().take(40) {
        println!("  {count:>8}  {keyword}");
    }
    assert!(
        deepest < rcad_step::DEEPEST_NESTING,
        "a real file nests {deepest} deep, which is not clear of the limit"
    );
}

fn how_deep(value: &Value, depth: usize) -> usize {
    match value {
        Value::List(items) => items
            .iter()
            .map(|item| how_deep(item, depth + 1))
            .max()
            .unwrap_or(depth),
        Value::Typed { value, .. } => how_deep(value, depth + 1),
        Value::Integer(_)
        | Value::Real(_)
        | Value::Text(_)
        | Value::Enumeration(_)
        | Value::Reference(_)
        | Value::Nothing
        | Value::Derived => depth,
        // A variant added later holds no values of its own until somebody says
        // it does, and the day it does this arm is what has to be looked at.
        _ => depth,
    }
}

fn corpus() -> PathBuf {
    let root = env::var_os("RCAD_STEP_CORPUS").unwrap_or_else(|| {
        panic!(
            "RCAD_STEP_CORPUS is not set. Point it at a directory holding nist/ and kicad/ (see \
             this file's header) — this test does not skip on a missing corpus."
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

fn files_under(directory: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files_under(&path));
            continue;
        }
        let is_step = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("stp") || extension.eq_ignore_ascii_case("step")
            });
        if is_step {
            found.push(path);
        }
    }
    found.sort();
    found
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}
