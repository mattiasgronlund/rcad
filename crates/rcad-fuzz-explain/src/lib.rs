//! One fuzz artifact in, and what it *means* out.
//!
//! A crash file from `cargo fuzz` is a bag of bytes. Learning what it says used
//! to take a morning: find which decoder the target uses, hand-decode the
//! numbers, guess at the surfaces, and then hand-write the regression test.
//! This does all of that in one call, on stable Rust, with no sanitizer and no
//! nightly.
//!
//! # The harnesses are the authority and this calls them
//!
//! Nothing here re-implements a fuzz target. The five `tests/fuzzing` modules
//! are reached across to with `#[path]`, exactly the way
//! `fuzz/fuzz_targets/*.rs` reach across to them, so the case this replays is
//! the case that crashed and not a paraphrase of it.
//!
//! # The reconstruction line is the point
//!
//! Every explanation carries one line of Rust that rebuilds the case from named
//! values, ready to paste into a test. For the two surface-pair targets that is
//! a [`bytes_for`](surfaces::bytes_for) call with the numbers spelled out, and
//! it is **checked before it is printed**: the bytes it would produce are
//! decoded again and the pair compared against the artifact's own. Where the
//! check fails, the slot says so rather than printing a line that lies.

#![forbid(unsafe_code)]
// `fuse.rs` and `writer.rs` each reach across to `the_door.rs`, so a crate that
// holds both loads that file twice. Every fuzz target is a binary of its own and
// never sees this; here the two modules are siblings, and the alternative —
// dropping one harness — is dropping a target this has to explain.
#![allow(clippy::duplicate_mod)]

use std::collections::HashMap;
use std::fmt::{self, Write as _};
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rcad_geom::{Surface, SurfaceMeet, intersect};
use rcad_predicates::{Policy, ProvenanceLog};
use rcad_step::StepFile;
use rcad_topo::{Model, Rule};

// `#[path]` and not `include!`, because the shared files open with a module doc
// comment and an inner attribute, and neither of those can be pasted into the
// middle of another file.
#[path = "../../rcad-geom/tests/fuzzing/mod.rs"]
mod surfaces;

#[path = "../../rcad-topo/tests/fuzzing/fuse.rs"]
mod fusing;

#[path = "../../rcad-step/tests/fuzzing/reader.rs"]
mod reading;

#[path = "../../rcad-step/tests/fuzzing/writer.rs"]
mod writing;

mod refusals;
pub use refusals::{
    CorpusReplay, RefusalGroup, SECONDS_ONE_FILE_IS_GIVEN,
    THE_SETTLED_DECISIONS_ADDED_UP_PAST_THE_BUDGET, as_faults_tsv, as_refusals_tsv,
    every_refusal_in,
};

/// The name the artifact of a pair of planes, balls and pipes belongs to.
pub const WHERE_TWO_SURFACES_CROSS: &str = "where_two_surfaces_cross";
/// The name the artifact of a pair with a cone or a torus in it belongs to.
pub const WHERE_A_CONE_OR_A_TORUS_CROSSES: &str = "where_a_cone_or_a_torus_crosses";
/// The name the artifact of a candidate STEP file belongs to.
pub const NO_STEP_FILE_MAKES_THE_READER_PANIC: &str = "no_step_file_makes_the_reader_panic";
/// The name the artifact of a model handed to the writer belongs to.
pub const NO_MODEL_MAKES_THE_WRITER_PANIC: &str = "no_model_makes_the_writer_panic";
/// The name the artifact of two models handed to the fuse belongs to.
pub const TWO_SOLIDS_FUSED_BY_A_STRANGER: &str = "two_solids_fused_by_a_stranger";

/// Every target name this knows how to explain.
pub const TARGETS: [&str; 5] = [
    WHERE_TWO_SURFACES_CROSS,
    WHERE_A_CONE_OR_A_TORUS_CROSSES,
    NO_STEP_FILE_MAKES_THE_READER_PANIC,
    NO_MODEL_MAKES_THE_WRITER_PANIC,
    TWO_SOLIDS_FUSED_BY_A_STRANGER,
];

/// **What one artifact of one target means.**
///
/// `Err` is only ever "these bytes are not a case of this target at all" — a
/// pair of surfaces that decodes to nothing, or bytes that are no `Model`. A
/// case that replayed and then panicked is an `Ok` carrying
/// [`Outcome::Panicked`], because that is the very thing this exists to report.
///
/// # Errors
///
/// If `target` is not one of [`TARGETS`], or if `bytes` decode to no case.
pub fn explain(target: &str, bytes: &[u8]) -> Result<Explained, String> {
    match target {
        WHERE_TWO_SURFACES_CROSS => a_pair_of_surfaces(bytes),
        WHERE_A_CONE_OR_A_TORUS_CROSSES => a_pair_with_a_cone_or_a_torus(bytes),
        NO_STEP_FILE_MAKES_THE_READER_PANIC => a_candidate_step_file(bytes),
        NO_MODEL_MAKES_THE_WRITER_PANIC => a_model_for_the_writer(bytes),
        TWO_SOLIDS_FUSED_BY_A_STRANGER => two_models_for_the_fuse(bytes),
        other => Err(format!(
            "{other} is not a fuzz target; the five are {}",
            TARGETS.join(", ")
        )),
    }
}

/// The four lines one artifact is worth.
#[derive(Clone, Debug)]
pub struct Explained {
    /// Which of [`TARGETS`] this was read as.
    pub target: String,
    /// The decoded case, through `Debug`.
    pub case: String,
    /// One line of Rust that rebuilds the case, or why there is none.
    pub rebuild: String,
    /// What replaying it did.
    pub outcome: Outcome,
}

impl fmt::Display for Explained {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(out, "target: {}", self.target)?;
        writeln!(out, "case: {}", self.case)?;
        writeln!(out, "rebuild: {}", self.rebuild)?;
        write!(out, "outcome: {}", self.outcome)
    }
}

/// What replaying one case through its own harness did.
///
/// **Exhaustive on purpose.** A replay blew up, gave up by name, or answered,
/// and there is no fourth thing it can have done — the three arms are the three
/// `just fuzz-triage` was written against, and a fourth would be a change to
/// that contract rather than a new milestone's variant.
#[derive(Clone, Debug)]
pub enum Outcome {
    /// The replay panicked, which is what a crash artifact is for.
    Panicked {
        /// What the panic said, and where, on one line.
        message: String,
    },
    /// The kernel gave up by name. Giving up is an answer (§17.4).
    Refused {
        /// The `Error` or `Decision` variant, through `Debug`.
        named: String,
        /// How far the harness itself got, through `Debug`.
        reached: String,
    },
    /// The kernel answered.
    Succeeded {
        /// What the answer was.
        what: String,
        /// How far the harness itself got, through `Debug`.
        reached: String,
    },
}

impl fmt::Display for Outcome {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Panicked { message } => write!(out, "panicked — {message}"),
            Self::Refused { named, reached } => {
                write!(out, "refused — {named}; the harness reached {reached}")
            }
            Self::Succeeded { what, reached } => {
                write!(out, "succeeded — {what}; the harness reached {reached}")
            }
        }
    }
}

/// One bucket [`triage`] collapsed a night's files into: how many landed
/// there, one to look at, and whether that bucket still fails against the
/// commit this binary was built at.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupedVerdict {
    /// Which of [`TARGETS`] every file in this bucket belongs to.
    pub target: String,
    /// How many files landed in this bucket.
    pub files: usize,
    /// Whether a file from this bucket still fails against the commit this
    /// binary was built at. `false` for a bucket that used to crash and does
    /// not any more.
    pub still_fails: bool,
    /// One file from the bucket, to read or replay by hand.
    pub sample: PathBuf,
    /// Every file that landed in this bucket, `sample` included. A queue row
    /// can name the group by any one of these (§418.4), not only the first
    /// file read — the first changes as the group grows.
    pub members: Vec<PathBuf>,
    /// The panic's own message and place, where the bucket is a panic; a
    /// fixed name for everything else, so a night's harmless residue does not
    /// fragment into one bucket per file.
    pub group: String,
}

/// Replay every `(target, path)` pair once, in this one process, and collapse
/// them into the buckets a night's report is made of.
///
/// This is what `--triage` replaces: the shell loop it used to run spent one
/// `cargo run` per file, which is what made 22 528 writer crash files from the
/// night of 2026-09-26 too slow to count as faults (§265.2). Reading every
/// file inside one already-running process, in a plain loop, is the whole
/// fix — nothing here changes what a single [`explain`] call decides about one
/// file, and a caller replaying one file by hand still gets the same answer
/// from the two-argument command line.
///
/// A `target` this binary does not recognise is skipped rather than guessed
/// at, and counted in the returned total instead — the caller decides what
/// "not a real target" means for the files it hands in.
pub fn triage(inputs: impl IntoIterator<Item = (String, PathBuf)>) -> (Vec<GroupedVerdict>, usize) {
    let mut buckets: HashMap<(String, String), GroupedVerdict> = HashMap::new();
    let mut skipped = 0;
    for (target, path) in inputs {
        if !TARGETS.contains(&target.as_str()) {
            skipped += 1;
            continue;
        }
        let (group, still_fails) = bucket_for(&target, &path);
        match buckets.entry((target.clone(), group.clone())) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let verdict = entry.get_mut();
                verdict.files += 1;
                verdict.members.push(path);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(GroupedVerdict {
                    target,
                    files: 1,
                    still_fails,
                    sample: path.clone(),
                    members: vec![path],
                    group,
                });
            }
        }
    }
    let mut groups: Vec<_> = buckets.into_values().collect();
    groups.sort_by(|a, b| {
        b.files
            .cmp(&a.files)
            .then_with(|| a.target.cmp(&b.target))
            .then_with(|| a.group.cmp(&b.group))
    });
    (groups, skipped)
}

/// What one file is worth to the tally: its bucket, and whether that bucket
/// still fails.
fn bucket_for(target: &str, path: &Path) -> (String, bool) {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(why) => {
            return (
                format!("{target}: could not read this artifact: {why}"),
                true,
            );
        }
    };
    match explain(target, &bytes) {
        Err(_why) => (format!("{target}: bytes did not decode to a case"), true),
        Ok(explained) => bucket_of(target, explained.outcome),
    }
}

/// A replay's bucket, and whether that bucket still fails.
///
/// A panic's bucket is its own message and place with the numbers in the
/// message taken out — see [`with_the_numbers_taken_out`] — so two files that
/// panicked at the same line with the same words group together even when the
/// values the assertion printed differ. Everything else is a fixed name per
/// target, because a fault report does not need to tell two successful
/// replays, or two refusals, apart.
fn bucket_of(target: &str, outcome: Outcome) -> (String, bool) {
    match outcome {
        Outcome::Panicked { message } => (with_the_numbers_taken_out(&message), true),
        Outcome::Refused { named, .. } => (
            format!("{target}: no longer fails — refused {named}"),
            false,
        ),
        Outcome::Succeeded { .. } => (format!("{target}: no longer fails — succeeded"), false),
    }
}

/// A panic's message with every number after its place written as `N`, and
/// the place itself left whole.
///
/// Keyed on the whole message, the 4 219 writer files of 2026-09-26 that still
/// failed made 1 725 buckets, because the messages print entity numbers
/// (`#20, in the solid #53`) and the coordinates of a plane; with the numbers
/// out they are 5 buckets, and the fuse's 95 files are 2. The place keeps its
/// line and column, because two assertions in one file are two faults.
fn with_the_numbers_taken_out(message: &str) -> String {
    let Some(after) = message.strip_prefix("panicked at ") else {
        return numbers_as_n(message);
    };
    match after.split_once(": ") {
        Some((place, said)) => format!("panicked at {place}: {}", numbers_as_n(said)),
        None => message.to_owned(),
    }
}

/// Every number in `text` written as `N`: its sign, its fraction and its
/// exponent with it, so `-1.5e-3` is one `N`. Digits that belong to a word,
/// as in `f64`, stay, because they are part of a name and not a value.
fn numbers_as_n(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut copied = 0;
    let mut at = 0;
    while at < bytes.len() {
        let part_of_a_word =
            at > 0 && (bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'_');
        let signed = bytes[at] == b'-' && bytes.get(at + 1).is_some_and(u8::is_ascii_digit);
        if part_of_a_word || !(signed || bytes[at].is_ascii_digit()) {
            at += 1;
            continue;
        }
        let end = end_of_the_number(bytes, at + usize::from(signed));
        out.push_str(&text[copied..at]);
        out.push('N');
        copied = end;
        at = end;
    }
    out.push_str(&text[copied..]);
    out
}

/// Where the number whose first digit is at `from` ends: its digits, then a
/// fraction if a digit follows the point, then an exponent if a digit follows
/// the `e` and its sign.
fn end_of_the_number(bytes: &[u8], from: usize) -> usize {
    let past_the_digits = |from: usize| {
        from + bytes[from..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count()
    };
    let mut end = past_the_digits(from);
    if bytes.get(end) == Some(&b'.') && bytes.get(end + 1).is_some_and(u8::is_ascii_digit) {
        end = past_the_digits(end + 1);
    }
    if bytes.get(end) == Some(&b'e') {
        let sign = usize::from(matches!(bytes.get(end + 1), Some(b'-' | b'+')));
        if bytes.get(end + 1 + sign).is_some_and(u8::is_ascii_digit) {
            end = past_the_digits(end + 1 + sign);
        }
    }
    end
}

/// [`triage`]'s report as tab-separated text, one line per bucket, for
/// `fuzz-triage` to write into the night's own directory.
///
/// The columns are `target`, `files`, `still_fails` (`yes`/`no`), `sample`,
/// `group`, in that order, header first, so `cut` or `awk -F'\t'` reads any
/// one of them by name without counting the others. `group` is last because it
/// is the one field not tab-safe by construction elsewhere but is here: a
/// [`Outcome::Panicked`] message is already squeezed to one line with no tabs
/// in it (see [`on_one_line`]), so nothing after it needs escaping.
pub fn as_tsv(groups: &[GroupedVerdict]) -> String {
    let mut out = String::from("target\tfiles\tstill_fails\tsample\tgroup\n");
    for verdict in groups {
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}",
            verdict.target,
            verdict.files,
            if verdict.still_fails { "yes" } else { "no" },
            verdict.sample.display(),
            verdict.group
        );
    }
    out
}

/// Every crash file of each still-failing bucket, for `fuzz-triage` to write
/// beside `fuzz-triage-verdicts.tsv`: `target\tgroup\tfile`, one line per
/// file, header first. A queue row names a group by any one of these files
/// (§418.4), not only [`GroupedVerdict::sample`], so this is what
/// `_status-triage` reads to tell whether a row's `crash-<hex>` names one.
///
/// A bucket that no longer fails keeps no file here — there is nothing left
/// to point a row at, and a fixed bucket's files would only make the
/// still-failing set this writes harder to tell apart from it.
pub fn as_members_tsv(groups: &[GroupedVerdict]) -> String {
    let mut out = String::from("target\tgroup\tfile\n");
    for verdict in groups {
        if !verdict.still_fails {
            continue;
        }
        for member in &verdict.members {
            let _ = writeln!(
                out,
                "{}\t{}\t{}",
                verdict.target,
                verdict.group,
                member.display()
            );
        }
    }
    out
}

/// A plane, a ball or a pipe against another, read the way
/// [`surfaces::two_surfaces_from`] reads it.
fn a_pair_of_surfaces(bytes: &[u8]) -> Result<Explained, String> {
    let pair = surfaces::two_surfaces_from(bytes)
        .ok_or_else(|| format!("{} bytes name no pair of surfaces", bytes.len()))?;
    let kinds = [bytes[0] & 0b11, (bytes[0] >> 2) & 0b11];
    let numbers = numbers_after_the_kinds_byte(
        bytes,
        [
            how_many_numbers_one_surface_reads(kinds[0] % 3),
            how_many_numbers_one_surface_reads(kinds[1] % 3),
        ],
    );
    let rebuilt = surfaces::bytes_for(kinds, &numbers);
    let rebuild = if surfaces::two_surfaces_from(&rebuilt) == Some(pair) {
        format!(
            "every_oracle_holds_of(two_surfaces_from(&bytes_for({}, &{})).unwrap())",
            as_a_list(&kinds),
            as_a_list_of_numbers(&numbers)
        )
    } else {
        "no bytes_for call reproduces this pair — rebuild it from the case above by hand".to_owned()
    };
    Ok(Explained {
        target: WHERE_TWO_SURFACES_CROSS.to_owned(),
        case: format!("{pair:?}"),
        rebuild,
        outcome: what_the_pair_did(pair),
    })
}

/// A pair one of whose slots is forced to be a cone or a ring, read the way
/// [`surfaces::two_surfaces_with_a_cone_or_a_torus_from`] reads it.
fn a_pair_with_a_cone_or_a_torus(bytes: &[u8]) -> Result<Explained, String> {
    let pair = surfaces::two_surfaces_with_a_cone_or_a_torus_from(bytes).ok_or_else(|| {
        format!(
            "{} bytes name no pair with a cone or a torus in it",
            bytes.len()
        )
    })?;
    let header = bytes[0];
    let forced = if header & 0b10 == 0 { 3 } else { 4 };
    let free = (header >> 2) % 5;
    let kinds = if header & 0b1 == 0 {
        [forced, free]
    } else {
        [free, forced]
    };
    let numbers = numbers_after_the_kinds_byte(
        bytes,
        [
            how_many_numbers_one_surface_reads(kinds[0]),
            how_many_numbers_one_surface_reads(kinds[1]),
        ],
    );
    let rebuilt = surfaces::bytes_for_a_cone_or_torus_pair(kinds, &numbers);
    let rebuild = if surfaces::two_surfaces_with_a_cone_or_a_torus_from(&rebuilt) == Some(pair) {
        format!(
            "every_oracle_holds_of(two_surfaces_with_a_cone_or_a_torus_from(\
             &bytes_for_a_cone_or_torus_pair({}, &{})).unwrap())",
            as_a_list(&kinds),
            as_a_list_of_numbers(&numbers)
        )
    } else {
        "no bytes_for_a_cone_or_torus_pair call reproduces this pair — rebuild it from the \
         case above by hand"
            .to_owned()
    };
    Ok(Explained {
        target: WHERE_A_CONE_OR_A_TORUS_CROSSES.to_owned(),
        case: format!("{pair:?}"),
        rebuild,
        outcome: what_the_pair_did(pair),
    })
}

/// Replay one pair through every oracle, then ask the kernel what it makes of
/// the pair so the outcome can name a refusal rather than only counting one.
fn what_the_pair_did(pair: [Surface; 2]) -> Outcome {
    let asked = what_happens_when_it_runs(|| surfaces::every_oracle_holds_of(pair));
    let asked = match asked {
        Err(message) => return Outcome::Panicked { message },
        Ok(asked) => format!("{asked:?}"),
    };
    let mut log = ProvenanceLog::new();
    match intersect(&pair[0], &pair[1], &Policy::certified(), &mut log) {
        Err(refusal) => Outcome::Refused {
            named: format!("rcad_geom::Error::{refusal:?}"),
            reached: asked,
        },
        Ok(meet) => Outcome::Succeeded {
            what: what_they_cross_in(&meet),
            reached: asked,
        },
    }
}

/// Which crossing a pair was named as, without printing the curves themselves —
/// two runs of a marched quartic are megabytes of `Debug` and say nothing a
/// triage session reads.
fn what_they_cross_in(meet: &SurfaceMeet) -> String {
    match meet {
        SurfaceMeet::Nothing => "SurfaceMeet::Nothing".to_owned(),
        SurfaceMeet::TouchAtOnePoint { at } => format!("SurfaceMeet::TouchAtOnePoint at {at:?}"),
        SurfaceMeet::Curves(runs) => format!("SurfaceMeet::Curves, {} of them", runs.len()),
        SurfaceMeet::SameSurface => "SurfaceMeet::SameSurface".to_owned(),
        // `Nothing`, `TouchAtOnePoint`, `Curves` and `SameSurface` are named
        // above; this arm is for the variant M6 adds.
        _ => format!("a SurfaceMeet variant this tool predates: {meet:?}"),
    }
}

/// A candidate Part 21 file, read the way
/// [`reading::everything_one_file_is_put_through`] reads it.
///
/// Every byte string is a case here — the target's whole claim is that no bytes
/// make the reader panic — so this never hands back `Err`.
fn a_candidate_step_file(bytes: &[u8]) -> Result<Explained, String> {
    let parsed = StepFile::read(bytes);
    let case = format!(
        "AStepFile {{ bytes: {}, parses: {}, instances: {} }}",
        bytes.len(),
        parsed.is_ok(),
        parsed.as_ref().map_or(0, |file| file.instances().count())
    );
    let reached = what_happens_when_it_runs(|| reading::everything_one_file_is_put_through(bytes));
    let reached = match reached {
        Err(message) => {
            return Ok(Explained {
                target: NO_STEP_FILE_MAKES_THE_READER_PANIC.to_owned(),
                case,
                rebuild: format!(
                    "everything_one_file_is_put_through({})",
                    as_a_literal(bytes)
                ),
                outcome: Outcome::Panicked { message },
            });
        }
        Ok(reached) => format!("{reached:?}"),
    };
    let outcome = match &parsed {
        Err(refusal) => Outcome::Refused {
            named: format!("rcad_step::Error::{refusal:?}"),
            reached,
        },
        Ok(file) => {
            let mut log = ProvenanceLog::new();
            match rcad_step::Import::read(file, &Policy::certified(), &mut log) {
                Err(refusal) => Outcome::Refused {
                    named: format!("rcad_step::Error::{refusal:?}"),
                    reached,
                },
                Ok(import) => Outcome::Succeeded {
                    what: format!("{} solids imported", import.solids().len()),
                    reached,
                },
            }
        }
    };
    Ok(Explained {
        target: NO_STEP_FILE_MAKES_THE_READER_PANIC.to_owned(),
        case,
        rebuild: format!(
            "everything_one_file_is_put_through({})",
            as_a_literal(bytes)
        ),
        outcome,
    })
}

/// A saved `Model`, decoded the way a stranger's file is and handed to the
/// writer.
fn a_model_for_the_writer(bytes: &[u8]) -> Result<Explained, String> {
    let model = postcard::from_bytes::<Model>(bytes)
        .map_err(|why| format!("{} bytes are no saved Model: {why}", bytes.len()))?;
    let reached = what_happens_when_it_runs(|| writing::one_model_decoded_and_written(bytes));
    let outcome = match reached {
        Err(message) => Outcome::Panicked { message },
        Ok(reached) => match rcad_step::write(&model, "fuzz") {
            Err(refusal) => Outcome::Refused {
                named: format!("rcad_step::CannotWrite::{refusal:?}"),
                reached: format!("{reached:?}"),
            },
            Ok(text) => Outcome::Succeeded {
                what: format!("{} characters of Part 21", text.len()),
                reached: format!("{reached:?}"),
            },
        },
    };
    Ok(Explained {
        target: NO_MODEL_MAKES_THE_WRITER_PANIC.to_owned(),
        case: format!("AModel {{ {} }}", what_one_model_holds(&model)),
        rebuild: format!("one_model_decoded_and_written({})", as_a_literal(bytes)),
        outcome,
    })
}

/// A header byte and two halves, each decoded into a `Model` and fused against
/// itself the way [`fusing::two_models_split_and_fused`] does it.
fn two_models_for_the_fuse(bytes: &[u8]) -> Result<Explained, String> {
    let Some((&header, rest)) = bytes.split_first() else {
        return Err("an empty artifact carries no header byte".to_owned());
    };
    let (first, second) = rest.split_at(rest.len() / 2);
    let halves = [(first, header), (second, header.rotate_left(4))];
    let decoded: Vec<_> = halves
        .iter()
        .map(|(half, _)| postcard::from_bytes::<Model>(half).ok())
        .collect();
    if decoded.iter().all(Option::is_none) {
        return Err(format!(
            "neither half of {} bytes is a saved Model",
            bytes.len()
        ));
    }
    let case = halves
        .iter()
        .zip(&decoded)
        .map(|((half, header), model)| match model {
            None => format!("AHalf {{ bytes: {}, decodes: false }}", half.len()),
            Some(model) => format!(
                "AHalf {{ bytes: {}, {}, {} }}",
                half.len(),
                what_one_model_holds(model),
                how_the_header_splits_it(*header, model.solid_count())
            ),
        })
        .collect::<Vec<_>>()
        .join(", ");
    let reached = what_happens_when_it_runs(|| fusing::two_models_split_and_fused(bytes));
    let outcome = match reached {
        Err(message) => Outcome::Panicked { message },
        Ok(reached) => what_the_halves_fused_to(&halves, &format!("{reached:?}")),
    };
    Ok(Explained {
        target: TWO_SOLIDS_FUSED_BY_A_STRANGER.to_owned(),
        case: format!("[{case}]"),
        rebuild: format!("two_models_split_and_fused({})", as_a_literal(bytes)),
        outcome,
    })
}

/// The two halves fused again, one at a time, so the outcome can name the
/// `Error` variant the harness only counts.
fn what_the_halves_fused_to(halves: &[(&[u8], u8); 2], reached: &str) -> Outcome {
    let answers: Vec<_> = halves
        .iter()
        .filter_map(|(half, header)| what_one_half_fused_to(half, *header))
        .collect();
    if answers.is_empty() {
        return Outcome::Succeeded {
            what: "nothing was fused: no half held a solid to fuse".to_owned(),
            reached: reached.to_owned(),
        };
    }
    let refusals: Vec<_> = answers
        .iter()
        .filter(|answer| answer.starts_with("rcad_topo::Error::"))
        .cloned()
        .collect();
    if refusals.is_empty() {
        Outcome::Succeeded {
            what: answers.join(" and "),
            reached: reached.to_owned(),
        }
    } else {
        Outcome::Refused {
            named: refusals.join(" and "),
            reached: reached.to_owned(),
        }
    }
}

/// What one half's fuse answered, or `None` where there was no fuse to run.
///
/// The split, the rule and the policy are read off `header` exactly as
/// `fuse.rs` reads them; that file stays the authority for the replay, and this
/// only names the answer it reached.
fn what_one_half_fused_to(bytes: &[u8], header: u8) -> Option<String> {
    let mut model = postcard::from_bytes::<Model>(bytes).ok()?;
    model
        .validate(&Policy::certified(), &mut ProvenanceLog::new())
        .ok()?;
    let solids: Vec<_> = model.solids().map(|solid| solid.key()).collect();
    if solids.is_empty() {
        return None;
    }
    let (arguments, tools) = solids.split_at(usize::from(header) % (solids.len() + 1));
    let rule = match (header >> 4) % 3 {
        0 => Rule::Union,
        1 => Rule::Intersection,
        _ => Rule::Difference,
    };
    let policy = if header & 0x80 == 0 {
        Policy::certified()
    } else {
        Policy::tolerant(model.scale().nominal_size())
    };
    let answer = model.fuse(arguments, tools, rule, &policy, &mut ProvenanceLog::new());
    Some(match answer {
        Err(refusal) => format!("rcad_topo::Error::{refusal:?}"),
        Ok(fused) => format!("{rule:?} made {} solids", fused.solids().len()),
    })
}

/// Which solids one half's header sends to each side of the fuse, and under
/// what rule and policy.
fn how_the_header_splits_it(header: u8, solids: usize) -> String {
    let rule = match (header >> 4) % 3 {
        0 => "Union",
        1 => "Intersection",
        _ => "Difference",
    };
    let policy = if header & 0x80 == 0 {
        "certified"
    } else {
        "tolerant"
    };
    format!(
        "header: {header}, split: {} of {solids} as arguments, rule: {rule}, policy: {policy}",
        usize::from(header) % (solids + 1)
    )
}

/// What is in one model, in counts rather than in a whole arena dump.
fn what_one_model_holds(model: &Model) -> String {
    format!(
        "solids: {}, shells: {}, faces: {}, wires: {}, edges: {}, vertices: {}",
        model.solid_count(),
        model.shell_count(),
        model.face_count(),
        model.wire_count(),
        model.edge_count(),
        model.vertex_count()
    )
}

/// Run `replay`, and hand back what it said if it panicked instead of aborting
/// the whole tool.
///
/// The hook is swapped for one that keeps the message rather than printing it,
/// because a backtrace on stderr is exactly the noise this tool replaces —
/// and swapped back afterwards.
fn what_happens_when_it_runs<T>(replay: impl FnOnce() -> T) -> Result<T, String> {
    let said = Arc::new(Mutex::new(String::new()));
    let writing = Arc::clone(&said);
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        if let Ok(mut kept) = writing.lock() {
            *kept = format!("{info}");
        }
    }));
    let answer = panic::catch_unwind(AssertUnwindSafe(replay));
    panic::set_hook(previous);
    answer.map_err(|_| {
        let kept = said
            .lock()
            .map_or_else(|_| String::new(), |kept| kept.clone());
        on_one_line(&kept)
    })
}

/// Every run of whitespace squeezed to one space, so a multi-line panic message
/// still fits the one line the contract promises.
fn on_one_line(message: &str) -> String {
    let squeezed = message.split_whitespace().collect::<Vec<_>>().join(" ");
    if squeezed.is_empty() {
        "a panic that said nothing".to_owned()
    } else {
        squeezed
    }
}

/// How many numbers a surface of each kind reads: a plane and a ball four, a
/// pipe and a cone seven, a ring eight.
///
/// # Panics
///
/// If `kind` is above 4. Both callers reduce first.
fn how_many_numbers_one_surface_reads(kind: u8) -> usize {
    match kind {
        0 | 1 => 4,
        2 | 3 => 7,
        4 => 8,
        other => unreachable!("{other} is not one of the five kinds"),
    }
}

/// The numbers the two slots read, decoded the way the harness decodes them:
/// four bytes of whole number, then a fifth counting fives of powers of two.
///
/// Short bytes give back fewer numbers than asked for, which is the case the
/// decoder itself refuses — so a caller that already has a pair in hand always
/// gets the full count.
fn numbers_after_the_kinds_byte(bytes: &[u8], counts: [usize; 2]) -> Vec<f64> {
    bytes[1..]
        .chunks_exact(5)
        .take(counts[0] + counts[1])
        .map(|five| {
            let whole = i32::from_le_bytes([five[0], five[1], five[2], five[3]]);
            f64::from(whole) * 2f64.powi(i32::from(five[4] as i8) * 5)
        })
        .collect()
}

/// `[0, 1]`, the way it is written in Rust.
fn as_a_list(kinds: &[u8; 2]) -> String {
    format!("[{}, {}]", kinds[0], kinds[1])
}

/// The numbers as a Rust slice literal. `Debug` for `f64` is the shortest
/// spelling that reads back bit for bit, and it is always a float literal.
fn as_a_list_of_numbers(numbers: &[f64]) -> String {
    let written: Vec<_> = numbers.iter().map(|number| format!("{number:?}")).collect();
    format!("[{}]", written.join(", "))
}

/// The bytes as a Rust byte-string literal, which is what rebuilds a case that
/// has no named values to rebuild it from.
fn as_a_literal(bytes: &[u8]) -> String {
    let mut out = String::from("b\"");
    for byte in bytes {
        match byte {
            b'\\' => out.push_str("\\\\"),
            b'"' => out.push_str("\\\""),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x20..=0x7e => out.push(char::from(*byte)),
            other => out.push_str(&format!("\\x{other:02x}")),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcad_geom::Scale;

    /// A plane against a ball, written out and read back through the tool.
    #[test]
    fn a_pair_of_surfaces_is_rebuilt_by_the_line_the_tool_prints() {
        let numbers = [
            0.0, 0.0, 1.0, -2.0, // a plane
            1.0, 2.0, 3.0, 4.0, // a ball
        ];
        let bytes = surfaces::bytes_for([0, 1], &numbers);
        let said = explain(WHERE_TWO_SURFACES_CROSS, &bytes).expect("a pair of surfaces");
        assert!(
            said.rebuild
                .contains("bytes_for([0, 1], &[0.0, 0.0, 1.0, -2.0, 1.0, 2.0, 3.0, 4.0])"),
            "the rebuild line lost the numbers: {}",
            said.rebuild
        );
    }

    /// The forced slot's kind and the free slot's both come back, and in slot
    /// order rather than in the order the kinds byte packs them.
    #[test]
    fn a_ring_against_a_plane_is_rebuilt_in_slot_order() {
        let numbers = [
            0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 8.0, 1.0, // a ring
            0.0, 0.0, 1.0, -2.0, // a plane
        ];
        let bytes = surfaces::bytes_for_a_cone_or_torus_pair([4, 0], &numbers);
        let said = explain(WHERE_A_CONE_OR_A_TORUS_CROSSES, &bytes).expect("a pair with a ring");
        assert!(
            said.rebuild
                .contains("bytes_for_a_cone_or_torus_pair([4, 0], &["),
            "the rebuild line lost the kinds: {}",
            said.rebuild
        );
        assert!(
            said.case.contains("Torus") && said.case.contains("Plane"),
            "the case lost a surface: {}",
            said.case
        );
    }

    /// Bytes that name no pair are not a case, and say so instead of replaying.
    #[test]
    fn three_bytes_are_not_a_pair_of_surfaces() {
        let why = explain(WHERE_TWO_SURFACES_CROSS, &[1, 2, 3]).expect_err("no pair");
        assert!(why.contains("no pair of surfaces"), "{why}");
    }

    /// Bytes that are no saved model are not a case of the writer's target.
    #[test]
    fn a_bag_of_bytes_is_not_a_saved_model() {
        let why = explain(NO_MODEL_MAKES_THE_WRITER_PANIC, &[255; 64]).expect_err("no model");
        assert!(why.contains("no saved Model"), "{why}");
    }

    /// The writer's case names the numbers a round trip is checked against —
    /// a face's surface and a coedge's corners and run — not just how many of
    /// each entity the model holds.
    #[test]
    fn the_writers_case_names_a_cubes_faces_and_corners_not_just_their_count() {
        let mut model = Model::new(Scale::millimetres(100.0).expect("100 mm is a working size"));
        model
            .add_cube(2.0, &Policy::certified(), &mut ProvenanceLog::new())
            .expect("a cube of side 2 is a sound solid");
        let bytes = postcard::to_allocvec(&model).expect("a Model always encodes");
        let said = explain(NO_MODEL_MAKES_THE_WRITER_PANIC, &bytes).expect("a saved cube");
        assert!(
            said.case.contains("Plane"),
            "a cube's faces are planes, lost from: {}",
            said.case
        );
        assert!(
            said.case.contains("coedge {"),
            "a face's coedges are missing: {}",
            said.case
        );
        assert!(
            said.case.contains("run: Straight"),
            "a cube's edges run straight, lost from: {}",
            said.case
        );
        assert!(
            !said.case.contains("solids:") && !said.case.contains("vertices:"),
            "the old counts-only case is still here: {}",
            said.case
        );
    }

    /// Any bytes at all are a case of the reader's target, because the claim
    /// the target makes is about any bytes at all.
    #[test]
    fn any_bytes_at_all_are_a_case_of_the_reader() {
        let said = explain(NO_STEP_FILE_MAKES_THE_READER_PANIC, b"ISO-10303-21;")
            .expect("every byte string is a case here");
        assert_eq!(said.target, NO_STEP_FILE_MAKES_THE_READER_PANIC);
        assert!(
            said.rebuild
                .contains("everything_one_file_is_put_through(b\"ISO-10303-21;\")"),
            "{}",
            said.rebuild
        );
    }

    /// A name that is not a target is refused rather than guessed at.
    #[test]
    fn a_target_nobody_fuzzes_is_refused() {
        let why = explain("where_the_wild_things_are", &[0]).expect_err("no such target");
        assert!(why.contains("is not a fuzz target"), "{why}");
    }

    /// The whole reason the tool can report a crash: a panic inside the replay
    /// comes back as a message instead of taking the process with it.
    #[test]
    fn a_panic_in_the_replay_comes_back_as_a_message() {
        let caught = what_happens_when_it_runs(|| panic!("the pipe was inside out"));
        let message = caught.expect_err("a panic");
        assert!(message.contains("the pipe was inside out"), "{message}");
        assert!(
            !message.contains('\n'),
            "a message has to be one line: {message}"
        );
    }

    /// Every byte a Rust byte-string literal cannot hold plainly is escaped.
    #[test]
    fn bytes_outside_plain_ascii_are_escaped() {
        assert_eq!(as_a_literal(&[0, b'a', 0xff, b'"']), r#"b"\x00a\xff\"""#);
    }

    use std::sync::atomic::{AtomicU32, Ordering};

    static NEXT_TEMP_FILE: AtomicU32 = AtomicU32::new(0);

    /// A file under a fresh directory of its own, removed when it drops —
    /// [`triage`] and [`bucket_for`] read artifacts off disk, so grouping them
    /// needs real paths and not just byte slices in memory.
    struct TempFile {
        dir: PathBuf,
        path: PathBuf,
    }

    impl TempFile {
        fn holding(contents: &[u8]) -> Self {
            let id = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!(
                "rcad-fuzz-explain-test-{}-{id}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir).expect("a scratch directory for one test file");
            let path = dir.join("artifact");
            std::fs::write(&path, contents).expect("one test artifact");
            Self { dir, path }
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    /// Two files that fail to decode the same way group into one bucket
    /// rather than one line each — the whole point of `--triage` over the
    /// per-file loop it replaces.
    #[test]
    fn two_files_with_the_same_undecodable_bytes_are_one_bucket() {
        let a = TempFile::holding(b"not a real crash input");
        let b = TempFile::holding(b"not a real crash input");
        let (groups, skipped) = triage([
            (WHERE_TWO_SURFACES_CROSS.to_owned(), a.path.clone()),
            (WHERE_TWO_SURFACES_CROSS.to_owned(), b.path.clone()),
        ]);
        assert_eq!(skipped, 0);
        assert_eq!(groups.len(), 1, "{groups:?}");
        assert_eq!(groups[0].files, 2);
        assert!(
            groups[0].still_fails,
            "an input that could not even be decoded is counted as still failing, \
             not silently dropped from the tally"
        );
    }

    /// The same bytes under two different targets are two buckets, never
    /// merged into one just because the bucket's name happens to match.
    #[test]
    fn the_same_bytes_under_two_targets_are_two_buckets() {
        let a = TempFile::holding(&[255; 64]);
        let b = TempFile::holding(&[255; 64]);
        let (groups, _) = triage([
            (NO_MODEL_MAKES_THE_WRITER_PANIC.to_owned(), a.path.clone()),
            (TWO_SOLIDS_FUSED_BY_A_STRANGER.to_owned(), b.path.clone()),
        ]);
        assert_eq!(groups.len(), 2, "{groups:?}");
        assert_ne!(groups[0].target, groups[1].target);
    }

    /// A target this binary has never heard of is skipped and counted, not
    /// guessed at or folded into some other bucket.
    #[test]
    fn an_unknown_target_is_skipped_and_counted() {
        let a = TempFile::holding(b"whatever");
        let (groups, skipped) = triage([("no_such_target".to_owned(), a.path.clone())]);
        assert!(groups.is_empty(), "{groups:?}");
        assert_eq!(skipped, 1);
    }

    /// A replay that answers, rather than panicking, is not counted as still
    /// failing — the same distinction §91.5 needed `fuzz-triage-verdict` for.
    #[test]
    fn a_replay_that_answers_does_not_still_fail() {
        let numbers = [
            0.0, 0.0, 1.0, -5.0, // a plane well clear of the ball below
            0.0, 0.0, 0.0, 1.0, // a unit ball at the origin
        ];
        let bytes = surfaces::bytes_for([0, 1], &numbers);
        let a = TempFile::holding(&bytes);
        let (groups, _) = triage([(WHERE_TWO_SURFACES_CROSS.to_owned(), a.path.clone())]);
        assert_eq!(groups.len(), 1, "{groups:?}");
        assert!(
            !groups[0].still_fails,
            "a plane that misses the ball entirely is not a fault: {:?}",
            groups[0]
        );
    }

    /// The report is tab-separated with the group's own text last, so a
    /// squeezed panic message with spaces in it cannot shift the columns
    /// before it.
    #[test]
    fn as_tsv_keeps_four_tabs_before_the_group_text() {
        let a = TempFile::holding(b"not a real crash input");
        let (groups, _) = triage([(WHERE_TWO_SURFACES_CROSS.to_owned(), a.path.clone())]);
        let text = as_tsv(&groups);
        let mut lines = text.lines();
        assert_eq!(
            lines.next(),
            Some("target\tfiles\tstill_fails\tsample\tgroup")
        );
        let body = lines.next().expect("one data row");
        assert_eq!(body.matches('\t').count(), 4, "{body}");
    }

    /// Every file of a still-failing group reaches the members file, not only
    /// the sample, and a fixed group's files do not — the whole point of
    /// keeping `members` is that a row can name the group by any one of them.
    #[test]
    fn as_members_tsv_lists_every_file_of_a_failing_group_and_none_of_a_fixed_one() {
        let a = TempFile::holding(b"not a real crash input");
        let b = TempFile::holding(b"not a real crash input");
        let fixed_numbers = [
            0.0, 0.0, 1.0, -5.0, // a plane well clear of the ball below
            0.0, 0.0, 0.0, 1.0, // a unit ball at the origin
        ];
        let fixed_bytes = surfaces::bytes_for([0, 1], &fixed_numbers);
        let c = TempFile::holding(&fixed_bytes);
        let (groups, _) = triage([
            (WHERE_TWO_SURFACES_CROSS.to_owned(), a.path.clone()),
            (WHERE_TWO_SURFACES_CROSS.to_owned(), b.path.clone()),
            (WHERE_TWO_SURFACES_CROSS.to_owned(), c.path.clone()),
        ]);
        let text = as_members_tsv(&groups);
        let mut lines = text.lines();
        assert_eq!(lines.next(), Some("target\tgroup\tfile"));
        let body: Vec<_> = lines.collect();
        assert_eq!(body.len(), 2, "{body:?}");
        assert!(
            body.iter()
                .any(|line| line.ends_with(&a.path.display().to_string())),
            "{body:?}"
        );
        assert!(
            body.iter()
                .any(|line| line.ends_with(&b.path.display().to_string())),
            "{body:?}"
        );
        assert!(
            !body
                .iter()
                .any(|line| line.ends_with(&c.path.display().to_string())),
            "a file from a fixed bucket was written to the members file: {body:?}"
        );
    }

    const THE_WRITERS_PLACE: &str =
        "panicked at crates/rcad-fuzz-explain/src/../../rcad-step/tests/fuzzing/writer.rs";

    /// Two of 2026-09-26's writer panics, the same assertion printing two
    /// different planes. Keyed on the whole message they were two of that
    /// night's 1 725 writer buckets.
    #[test]
    fn one_assertion_that_printed_different_planes_is_one_bucket() {
        let said = "assertion `left == right` failed: a sound solid's face came back from \
                    its own file with a different surface left:";
        let one = format!(
            "{THE_WRITERS_PLACE}:262:9: {said} \"Plane(Plane {{ normal: \
             Vector([[0.7071067811865475, 0.0, 0.7071067811865475]]), offset: 0.0 }})\" \
             right: \"Plane(Plane {{ normal: Vector([[0.7071067811865476, 0.0, \
             0.7071067811865476]]), offset: 0.0 }})\""
        );
        let other = format!(
            "{THE_WRITERS_PLACE}:262:9: {said} \"Plane(Plane {{ normal: \
             Vector([[-0.28, 7.347880794884119e-17, 0.96]]), offset: 6.119999999999999 }})\" \
             right: \"Plane(Plane {{ normal: Vector([[-0.28, 7.347880794884119e-17, \
             0.96]]), offset: 6.119999999999998 }})\""
        );
        let bucket_of_a_panic = |message| {
            bucket_of(
                "no_model_makes_the_writer_panic",
                Outcome::Panicked { message },
            )
        };
        let (bucket, still_fails) = bucket_of_a_panic(one);
        assert_eq!((bucket.clone(), still_fails), bucket_of_a_panic(other));
        assert!(still_fails);
        assert!(
            bucket.starts_with(&format!("{THE_WRITERS_PLACE}:262:9: ")),
            "the place lost its line and column: {bucket}"
        );
        assert!(
            bucket.contains("Vector([[N, N, N]]), offset: N"),
            "{bucket}"
        );
    }

    /// Entity numbers are values too: the reader refusing `#20` and refusing
    /// `#68` is one refusal. A different assertion in the same file is not.
    #[test]
    fn two_assertions_in_one_file_are_two_buckets() {
        let refuses = |entity: u32, solid: u32| {
            with_the_numbers_taken_out(&format!(
                "{THE_WRITERS_PLACE}:186:9: a sound model was written as a file the reader \
                 refuses: #{entity}, in the solid #{solid}, is not one this kernel can build"
            ))
        };
        assert_eq!(refuses(20, 53), refuses(68, 101));
        let does_not_measure = with_the_numbers_taken_out(&format!(
            "{THE_WRITERS_PLACE}:219:17: a solid that measured does not measure after its \
             own file: a face of this solid could not be cut into triangles: the corner at \
             14 is outside the range a triangulation takes: TooSmall"
        ));
        assert_ne!(refuses(20, 53), does_not_measure);
    }

    /// A sign, a fraction and an exponent are one number; the digits of a
    /// name are not a number at all.
    #[test]
    fn digits_inside_a_word_are_not_a_number() {
        assert_eq!(
            numbers_as_n("a f64 of -1.5e-3 at #20, §265, x-1 and 7."),
            "a f64 of N at #N, §N, x-N and N."
        );
    }
}
