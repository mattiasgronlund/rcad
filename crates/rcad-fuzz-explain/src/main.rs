//! One fuzz artifact in, four lines of what it means out.
//!
//! ```text
//! cargo run -q -p rcad-fuzz-explain -- <target-name> <artifact-path>
//! ```
//!
//! Exit 0 when the bytes decoded to a case and the case replayed, whatever the
//! case then did. Non-zero, with one line on stderr, when they did not.
//!
//! ```text
//! cargo run -q --profile test -p rcad-fuzz-explain -- --triage
//! ```
//!
//! Reads `<target>\t<path>` lines from stdin, one per artifact, and replays
//! every one of them in this single process — a night of tens of thousands of
//! files then costs one `cargo run`, not one per file (§265.2). Prints
//! [`rcad_fuzz_explain::as_tsv`]'s report of [`rcad_fuzz_explain::triage`] to
//! stdout, and always exits 0: a bucket that still fails is data in that
//! report, not a reason for this process itself to fail.

#![forbid(unsafe_code)]

use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

use rcad_fuzz_explain::{TARGETS, as_members_tsv, as_tsv, explain, triage};

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some("--triage") {
        return triage_from_stdin(&arguments[1..]);
    }
    let [target, artifact] = arguments.as_slice() else {
        eprintln!(
            "usage: rcad-fuzz-explain <target-name> <artifact-path>; the targets are {}",
            TARGETS.join(", ")
        );
        return ExitCode::FAILURE;
    };
    let bytes = match std::fs::read(artifact) {
        Ok(bytes) => bytes,
        Err(why) => {
            eprintln!("{artifact}: {why}");
            return ExitCode::FAILURE;
        }
    };
    match explain(target, &bytes) {
        Ok(said) => {
            println!("{said}");
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("{artifact}: {why}");
            ExitCode::FAILURE
        }
    }
}

/// `--triage`'s whole job: read the manifest `fuzz-triage` built, replay it,
/// print the report.
///
/// `rest` is everything after `--triage` on the command line: nothing, or
/// `--members <path>` to also write every still-failing group's crash files
/// to `path` (§418.4). Stdout is the same [`as_tsv`] report either way, byte
/// for byte, because `fuzz-triage` runs this from the replayed `main`'s own
/// tree and that tree can be older than this binary's — an option that
/// changed what the plain call prints would change what an old call's output
/// means.
fn triage_from_stdin(rest: &[String]) -> ExitCode {
    let members_path = match rest {
        [] => None,
        [flag, path] if flag == "--members" => Some(PathBuf::from(path)),
        other => {
            eprintln!(
                "--triage: unknown arguments after --triage: {}",
                other.join(" ")
            );
            return ExitCode::FAILURE;
        }
    };
    let mut manifest = String::new();
    if let Err(why) = std::io::stdin().read_to_string(&mut manifest) {
        eprintln!("--triage: could not read the manifest from stdin: {why}");
        return ExitCode::FAILURE;
    }
    let mut inputs = Vec::new();
    for line in manifest.lines() {
        let Some((target, path)) = line.split_once('\t') else {
            eprintln!("--triage: a manifest line is not <target>\\t<path>: {line}");
            return ExitCode::FAILURE;
        };
        inputs.push((target.to_owned(), PathBuf::from(path)));
    }
    let total = inputs.len();
    let inputs = inputs.into_iter().enumerate().map(|(started, input)| {
        // A hang shows up as the last of these lines with nothing after it,
        // which is the only sign of one that a single process can give.
        if started > 0 && started % PROGRESS_EVERY == 0 {
            eprintln!("--triage: {started} of {total} replayed");
        }
        input
    });
    let (groups, skipped) = triage(inputs);
    print!("{}", as_tsv(&groups));
    if let Some(path) = &members_path
        && let Err(why) = std::fs::write(path, as_members_tsv(&groups))
    {
        eprintln!(
            "--triage: could not write the members file {}: {why}",
            path.display()
        );
        return ExitCode::FAILURE;
    }
    eprintln!("--triage: {total} of {total} replayed, {skipped} skipped as no target");
    ExitCode::SUCCESS
}

/// How many inputs go by between two progress lines: often enough that a stall
/// shows over a night of tens of thousands, and never for a selftest's three.
const PROGRESS_EVERY: usize = 2000;
