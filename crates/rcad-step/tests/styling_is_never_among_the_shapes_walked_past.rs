//! Pins the claim on [`Import::shapes_this_reader_did_not_take`]: styling is
//! not shape-carrying, so it is never a key in that map, even for a file that
//! is full of it.
//!
//! Needs `RCAD_STEP_CORPUS` (see `how_often_a_real_file_imports.rs`'s header);
//! `just t` sets it, so this does not skip on a missing corpus, it fails.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, StepFile};

/// A real file that names `STYLED_ITEM` — so it carries colour, in STEP's own
/// sense of the word — and also imports, reports no styling keyword among the
/// shapes it walked past.
///
/// This reader parses no styling at all, so the map cannot name what it never
/// reads. Guards the doc comment on
/// [`Import::shapes_this_reader_did_not_take`] against reading as "everything
/// in the file that is not a solid, styling included": if this ever failed,
/// either styling had started being parsed, or a styling keyword had been
/// added to the map without the reader gaining any idea what to do with it.
#[test]
fn a_file_with_styled_items_reports_no_styling_keyword_walked_past() {
    let root = corpus();
    let mut checked = 0;
    for path in files_under(&root) {
        let bytes = fs::read(&path).expect("a file the walk just listed");
        if !contains(&bytes, b"STYLED_ITEM") {
            continue;
        }
        let Ok(file) = StepFile::read(&bytes) else {
            continue;
        };
        let mut log = ProvenanceLog::new();
        let Ok(import) = Import::read(&file, &Policy::certified(), &mut log) else {
            continue;
        };
        checked += 1;
        for keyword in import.shapes_this_reader_did_not_take().keys() {
            assert!(
                !mentions_styling(keyword),
                "{} names STYLED_ITEM and imported, but {keyword} — a styling keyword — showed \
                 up in shapes_this_reader_did_not_take. This reader does not parse styling, so \
                 nothing named there should ever look like it.",
                path.display()
            );
        }
    }
    assert!(
        checked > 0,
        "no file under {} both names STYLED_ITEM and imports, so this test checked nothing",
        root.display()
    );
}

fn mentions_styling(keyword: &str) -> bool {
    ["STYL", "COLOUR", "COLOR", "PRESENTATION", "SURFACE_STYLE"]
        .iter()
        .any(|marker| keyword.contains(marker))
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

fn corpus() -> PathBuf {
    let root = env::var_os("RCAD_STEP_CORPUS").unwrap_or_else(|| {
        panic!(
            "RCAD_STEP_CORPUS is not set. Point it at a directory holding nist/ and kicad/ (see \
             how_often_a_real_file_imports.rs's header) — this test does not skip on a missing \
             corpus."
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
