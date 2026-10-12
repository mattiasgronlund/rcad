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
//!
//! ```text
//! cargo run -q --profile test -p rcad-fuzz-explain -- --refusals <corpus> \
//!     [--faults <path>] [--seconds <n>]
//! ```
//!
//! Replays every file of `<corpus>`, a directory of the fuse fuzzer's inputs,
//! through its harness, and prints [`rcad_fuzz_explain::as_refusals_tsv`]'s
//! groups of every refusal met (§437.5). `--faults` writes the files that
//! panicked, ran past `--seconds` (120 by default) or could not be read. Exits
//! 0 whatever the files did, since they are all data in the report. Returning
//! from `main` ends the process, and with it any worker left with a file that
//! did not return.

#![forbid(unsafe_code)]

use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use rcad_fuzz_explain::{
    SECONDS_ONE_FILE_IS_GIVEN, TARGETS, as_faults_tsv, as_members_tsv, as_refusals_tsv, as_tsv,
    every_refusal_in, explain, triage,
};

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    if arguments.first().map(String::as_str) == Some("--triage") {
        return triage_from_stdin(&arguments[1..]);
    }
    if arguments.first().map(String::as_str) == Some("--refusals") {
        return refusals_in_a_corpus(&arguments[1..]);
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

/// `--refusals`'s whole job: list the corpus, replay it, print the groups, and
/// write the faults where asked.
fn refusals_in_a_corpus(rest: &[String]) -> ExitCode {
    let Some((corpus, mut options)) = rest.split_first() else {
        eprintln!("--refusals: name the corpus directory to replay");
        return ExitCode::FAILURE;
    };
    let mut faults_path = None;
    let mut seconds = SECONDS_ONE_FILE_IS_GIVEN;
    while let [flag, value, more @ ..] = options {
        match flag.as_str() {
            "--faults" => faults_path = Some(PathBuf::from(value)),
            "--seconds" => match value.parse::<u64>() {
                Ok(given) if given > 0 => seconds = given,
                _ => {
                    eprintln!("--refusals: --seconds takes a whole number above 0, not {value}");
                    return ExitCode::FAILURE;
                }
            },
            _ => break,
        }
        options = more;
    }
    if !options.is_empty() {
        eprintln!(
            "--refusals: unknown arguments after the corpus: {}",
            options.join(" ")
        );
        return ExitCode::FAILURE;
    }
    let mut files = match std::fs::read_dir(corpus) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .collect::<Vec<_>>(),
        Err(why) => {
            eprintln!("--refusals: could not list {corpus}: {why}");
            return ExitCode::FAILURE;
        }
    };
    files.sort();
    let workers = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    let replay = every_refusal_in(files, workers, Duration::from_secs(seconds));
    print!("{}", as_refusals_tsv(&replay));
    if let Some(path) = &faults_path
        && let Err(why) = std::fs::write(path, as_faults_tsv(&replay))
    {
        eprintln!(
            "--refusals: could not write the faults file {}: {why}",
            path.display()
        );
        return ExitCode::FAILURE;
    }
    eprintln!(
        "--refusals: {} files replayed, {} refusals in {} files, {} groups, {} panicked, {} \
         hung, {} unreadable",
        replay.files,
        replay.refusals(),
        replay.files_that_refused,
        replay.groups.len(),
        replay.panicked.len(),
        replay.hung.len(),
        replay.unreadable.len()
    );
    ExitCode::SUCCESS
}
