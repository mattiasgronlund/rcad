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

use std::fmt;
use std::panic::{self, AssertUnwindSafe};
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
}
