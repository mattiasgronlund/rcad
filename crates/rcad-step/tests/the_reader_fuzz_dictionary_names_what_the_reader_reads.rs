//! `fuzz/step.dict` is a libFuzzer dictionary of the STEP keywords
//! `crates/rcad-step/src` reads by name — see `justfile`'s `fuzz-target` and
//! `fuzz-reader`. This test is the one place both sides of that dictionary are
//! written down, so a keyword added on one side and not the other is caught
//! here rather than found later as a dictionary that quietly stopped teaching
//! libFuzzer anything new.
//!
//! **This is a hand-kept oracle, not a live query of the reader.** Nothing in
//! `rcad_step`'s public API enumerates the keywords it recognizes — recognition
//! is a few dozen string literals spread across `read.rs`, `token.rs`,
//! `geometry.rs`, `units.rs`, `brep.rs` and `import.rs`, compared one at a time
//! against whatever keyword a record happens to carry, and nothing collects
//! them into a list anything outside those files can call. Building a version
//! of this test that notices a *new* keyword landing in the reader's source
//! would need one of those files to publish such a list — a `src/` change,
//! outside this test's own write set — and the alternative of scraping the
//! `.rs` files for string literals as this test's oracle was ruled out rather
//! than attempted (see the T6 lane's report). So what follows is checked by
//! hand against the source, once, and is worth exactly as much as that check:
//! it keeps `fuzz/step.dict` and this file from drifting apart, and no more.
//!
//! The keywords are grouped by which file names them, matching `step.dict`'s
//! own comments, so a name added to one is easy to find in the other.

/// Every Part 21 keyword `step.dict` is supposed to carry — the framing words
/// `crates/rcad-step/src/read.rs` checks by name, and every entity type
/// `crates/rcad-step/src/{geometry,units,brep,import}.rs` matches a record's
/// keyword against. Not the short punctuation tokens at the bottom of the
/// dictionary: those are not entity names, and are covered by
/// `the_special_tokens_step_dict_carries_are_exactly_the_ones_the_lexer_treats_specially`
/// below.
const KEYWORDS_THE_READER_READS: &[&str] = &[
    // Part 21 framing (crates/rcad-step/src/read.rs)
    "ISO-10303-21",
    "END-ISO-10303-21",
    "HEADER",
    "DATA",
    "ENDSEC",
    "FILE_DESCRIPTION",
    "FILE_NAME",
    "FILE_SCHEMA",
    // Placement and raw geometry (crates/rcad-step/src/geometry.rs)
    "AXIS2_PLACEMENT_3D",
    "CARTESIAN_POINT",
    "DIRECTION",
    "PLANE",
    "CYLINDRICAL_SURFACE",
    "SPHERICAL_SURFACE",
    "CONICAL_SURFACE",
    "TOROIDAL_SURFACE",
    // Curves and topology (crates/rcad-step/src/brep.rs)
    "CIRCLE",
    "ELLIPSE",
    "LINE",
    "MANIFOLD_SOLID_BREP",
    "CLOSED_SHELL",
    "ADVANCED_FACE",
    "FACE_SURFACE",
    "FACE_BOUND",
    "FACE_OUTER_BOUND",
    "EDGE_LOOP",
    "ORIENTED_EDGE",
    "EDGE_CURVE",
    "VERTEX_POINT",
    // A solid refused by name, and shapes recognized only to be skipped
    // (crates/rcad-step/src/import.rs)
    "BREP_WITH_VOIDS",
    "SHELL_BASED_SURFACE_MODEL",
    "GEOMETRIC_CURVE_SET",
    "GEOMETRIC_SET",
    "TESSELLATED_GEOMETRIC_SET",
    "TESSELLATED_CURVE_SET",
    "COMPLEX_TRIANGULATED_SURFACE_SET",
    "TRIANGULATED_SURFACE_SET",
    // Assembly structure (crates/rcad-step/src/import.rs)
    "NEXT_ASSEMBLY_USAGE_OCCURRENCE",
    "ITEM_DEFINED_TRANSFORMATION",
    "MAPPED_ITEM",
    "REPRESENTATION_MAP",
    // Units and uncertainty (crates/rcad-step/src/units.rs)
    "LENGTH_UNIT",
    "SI_UNIT",
    "CONVERSION_BASED_UNIT",
    "LENGTH_MEASURE_WITH_UNIT",
    "MEASURE_WITH_UNIT",
    "PLANE_ANGLE_UNIT",
    "PLANE_ANGLE_MEASURE_WITH_UNIT",
    "GLOBAL_UNIT_ASSIGNED_CONTEXT",
    "GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT",
    "UNCERTAINTY_MEASURE_WITH_UNIT",
];

/// The short tokens `crates/rcad-step/src/token.rs`'s lexer gives their own
/// token kind, plus the enumeration values the reader reads by name:
/// `.T.`/`.F.`, which `geometry.rs`'s `flag_at` reads as a boolean, and the
/// SI unit name, length prefixes and angle unit `units.rs` matches.
/// `.UNSPECIFIED.` is deliberately absent: nothing downstream treats it any
/// differently from any other enumeration name the reader refuses.
const SPECIAL_TOKENS_THE_READER_TREATS_SPECIALLY: &[&str] = &[
    "#", "=", "(", ")", "$", "*", ".T.", ".F.", ".METRE.", ".MICRO.", ".MILLI.", ".CENTI.",
    ".RADIAN.",
];

const STEP_DICT: &str = include_str!("../../../fuzz/step.dict");

/// Every quoted entry in `fuzz/step.dict`, in file order, comments and blank
/// lines dropped.
fn entries_in_step_dict() -> Vec<&'static str> {
    STEP_DICT
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            line.strip_prefix('"')
                .and_then(|rest| rest.strip_suffix('"'))
                .unwrap_or_else(|| {
                    panic!("not a libFuzzer dictionary line, missing its quotes: {line}")
                })
        })
        .collect()
}

/// A keyword-shaped entry: upper-case letters, digits and hyphens, at least
/// two characters, which is every entity name and framing keyword in the
/// dictionary and none of the punctuation tokens at its foot.
fn looks_like_a_keyword(entry: &str) -> bool {
    entry.len() > 1
        && entry
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

fn sorted(mut names: Vec<&str>) -> Vec<&str> {
    names.sort_unstable();
    names
}

#[test]
fn every_keyword_in_step_dict_is_one_the_reader_reads_and_the_reverse() {
    let in_dict = sorted(
        entries_in_step_dict()
            .into_iter()
            .filter(|entry| looks_like_a_keyword(entry))
            .collect(),
    );
    let should_be_there = sorted(KEYWORDS_THE_READER_READS.to_vec());
    assert_eq!(
        in_dict, should_be_there,
        "fuzz/step.dict's keywords and this test's own list of what the reader \
         reads have drifted apart — add whichever side is missing an entry, \
         after checking the reader's source rather than trusting either list"
    );
}

#[test]
fn the_special_tokens_step_dict_carries_are_exactly_the_ones_the_lexer_treats_specially() {
    let in_dict = sorted(
        entries_in_step_dict()
            .into_iter()
            .filter(|entry| !looks_like_a_keyword(entry))
            .collect(),
    );
    let should_be_there = sorted(SPECIAL_TOKENS_THE_READER_TREATS_SPECIALLY.to_vec());
    assert_eq!(
        in_dict, should_be_there,
        "fuzz/step.dict's short tokens no longer match the ones \
         crates/rcad-step/src/token.rs's lexer gives their own token kind"
    );
}

/// Every entry the dictionary carries is well-formed libFuzzer syntax: one
/// double-quoted token per line, nothing escaped, nothing empty. A malformed
/// line is not a Part 21 keyword either, so it would fail the two tests above
/// for the wrong reason — this says which line, rather than leaving a
/// panicking `unwrap_or_else` to do it.
#[test]
fn every_line_of_step_dict_is_a_quoted_libfuzzer_token() {
    for line in STEP_DICT.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        assert!(
            line.len() >= 3 && line.starts_with('"') && line.ends_with('"'),
            "not a quoted libFuzzer dictionary token: {line}"
        );
        let inside = &line[1..line.len() - 1];
        assert!(
            !inside.is_empty() && !inside.contains('"'),
            "empty or badly quoted dictionary token: {line}"
        );
    }
}
