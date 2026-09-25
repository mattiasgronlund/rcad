//! `fuzz/step.dict` is a libFuzzer dictionary of the STEP words
//! `crates/rcad-step/src` reads by name — see `justfile`'s `fuzz-target` and
//! `fuzz-reader`.
//!
//! **The names are not checked here.** Every keyword and enumeration name the
//! reader reads is written once, in `crates/rcad-step/src/names.rs`, and a unit
//! test beside that list holds the dictionary's names to it — the list is
//! crate-private, so the check has to live in the crate.
//!
//! What is left for this file is the rest of the dictionary: that every line is
//! well-formed, and that its one-byte entries are exactly the punctuation the
//! lexer in `crates/rcad-step/src/token.rs` gives a token kind of its own. That
//! punctuation is matched as bytes rather than as names, so no list in the
//! reader holds it and it stays written down by hand below.

/// The one-byte tokens `crates/rcad-step/src/token.rs`'s lexer gives their own
/// token kind and the dictionary carries: `#` opens a reference, `=` separates
/// an id from its record, `(` and `)` bound an attribute list, `$` is
/// `Token::Nothing` and `*` is `Token::Derived`.
const PUNCTUATION_THE_LEXER_TREATS_SPECIALLY: &[&str] = &["#", "=", "(", ")", "$", "*"];

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

fn sorted(mut names: Vec<&str>) -> Vec<&str> {
    names.sort_unstable();
    names
}

/// Every name the dictionary carries is longer than one byte and every piece of
/// punctuation is exactly one, so length is what splits the two — the names are
/// checked in `src/names.rs`, the punctuation here.
#[test]
fn the_one_byte_entries_of_step_dict_are_exactly_the_punctuation_the_lexer_treats_specially() {
    let in_dict = sorted(
        entries_in_step_dict()
            .into_iter()
            .filter(|entry| entry.len() == 1)
            .collect(),
    );
    let should_be_there = sorted(PUNCTUATION_THE_LEXER_TREATS_SPECIALLY.to_vec());
    assert_eq!(
        in_dict, should_be_there,
        "fuzz/step.dict's one-byte entries no longer match the punctuation \
         crates/rcad-step/src/token.rs's lexer gives its own token kind"
    );
}

/// Every entry the dictionary carries is well-formed libFuzzer syntax: one
/// double-quoted token per line, nothing escaped, nothing empty. A malformed
/// line would fail the checks on names and punctuation for the wrong reason —
/// this says which line, rather than leaving a panicking `unwrap_or_else` to
/// do it.
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
