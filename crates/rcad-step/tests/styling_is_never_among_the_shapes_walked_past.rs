//! Pins both halves of what [`Import::shapes_this_reader_did_not_take`] says
//! about styling: it is never a key in that map, and a file that carries
//! styling and no solid does not get an empty map — it gets an `Err`.
//!
//! Needs `RCAD_STEP_CORPUS` (see `how_often_a_real_file_imports.rs`'s header);
//! `just t` sets it, so this does not skip on a missing corpus, it fails.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::{Import, Refusal, StepFile};

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
///
/// The `let Ok(…) else { continue }` here walks past every file that does not
/// import, and one of those is the only corpus file that carries styling and no
/// solid. That one is not skipped: the test below is about exactly it.
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

/// A real file that carries styling and holds no `MANIFOLD_SOLID_BREP` is
/// refused by name, and no map is built for it at all.
///
/// The test above walks past exactly these files with a `let Ok(…) else`, and
/// they are the ones the doc comment is about. `Import::read` returns before
/// `walked_past` is filled in, so the answer is an `Err` and not the empty map
/// a caller using this map as the denominator of "what did the importer drop"
/// would branch on. The corpus holds one such file —
/// `nist_ftc_08_asme1_ap242-e1-tg.stp`, 272 `STYLED_ITEM`s and no solid — and
/// it is also why the honest sentence names two refusals: it has no
/// `VERTEX_POINT` either, so [`Refusal::TheFileHasNoSize`] comes first.
#[test]
fn a_file_with_styled_items_and_no_solid_is_refused_rather_than_given_an_empty_map() {
    let root = corpus();
    let mut checked = 0;
    for path in files_under(&root) {
        let bytes = fs::read(&path).expect("a file the walk just listed");
        if !contains(&bytes, b"STYLED_ITEM") || contains(&bytes, b"MANIFOLD_SOLID_BREP") {
            continue;
        }
        let file = StepFile::read(&bytes).expect("a corpus file reads");
        let mut log = ProvenanceLog::new();
        let why = match Import::read(&file, &Policy::certified(), &mut log) {
            Ok(import) => panic!(
                "{} holds no MANIFOLD_SOLID_BREP and yet imported, handing back a map of {} \
                 shapes walked past",
                path.display(),
                import.shapes_this_reader_did_not_take().len()
            ),
            Err(why) => why,
        };
        assert!(
            matches!(
                why,
                Refusal::TheFileHoldsNoSolid | Refusal::TheFileHasNoSize
            ),
            "{} holds no solid, so it must be refused for holding none or for having no size \
             to be read at, and instead it was refused with {why}",
            path.display()
        );
        checked += 1;
    }
    assert!(
        checked > 0,
        "no file under {} names STYLED_ITEM without also holding a MANIFOLD_SOLID_BREP, so this \
         test checked nothing and the claim it guards is unmeasured",
        root.display()
    );
}

fn mentions_styling(keyword: &str) -> bool {
    ["STYL", "COLOUR", "COLOR", "PRESENTATION", "SURFACE_STYLE"]
        .iter()
        .any(|marker| keyword.contains(marker))
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
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
