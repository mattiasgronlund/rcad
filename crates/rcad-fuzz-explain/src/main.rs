//! One fuzz artifact in, four lines of what it means out.
//!
//! ```text
//! cargo run -q -p rcad-fuzz-explain -- <target-name> <artifact-path>
//! ```
//!
//! Exit 0 when the bytes decoded to a case and the case replayed, whatever the
//! case then did. Non-zero, with one line on stderr, when they did not.

#![forbid(unsafe_code)]

use std::process::ExitCode;

use rcad_fuzz_explain::{TARGETS, explain};

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
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
