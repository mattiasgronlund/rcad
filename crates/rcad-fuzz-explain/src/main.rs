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
//! cargo run -q -p rcad-fuzz-explain -- --triage
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

use rcad_fuzz_explain::{TARGETS, as_tsv, explain, triage};

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some("--triage") {
        return triage_from_stdin();
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
fn triage_from_stdin() -> ExitCode {
    let mut manifest = String::new();
    if let Err(why) = std::io::stdin().read_to_string(&mut manifest) {
        eprintln!("--triage: could not read the manifest from stdin: {why}");
        return ExitCode::FAILURE;
    }
    let inputs = manifest.lines().filter_map(|line| {
        let (target, path) = line.split_once('\t')?;
        Some((target.to_owned(), PathBuf::from(path)))
    });
    let (groups, skipped) = triage(inputs);
    print!("{}", as_tsv(&groups));
    eprintln!("skipped: {skipped}");
    ExitCode::SUCCESS
}
