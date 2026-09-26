//! `read_back` matches the words of `totals` byte by byte, so that a merge
//! inside a draw makes no `bcmp` (§273). A word spelled wrong in one of those
//! byte patterns would drop its line without a sound, so this reads back a
//! `totals` file with every word in it, through the only door the crate has.
//!
//! The timings are named by an environment variable, which this test cannot
//! set in its own process without `unsafe`, so it runs itself again as a
//! child with the variable set.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

const THE_CHILD: &str = "RCAD_FUZZ_TIMING_TEST_CHILD";
const TARGET: &str = "a-target";

#[test]
fn a_merge_reads_back_every_word_another_process_wrote_to_totals() {
    if std::env::var_os(THE_CHILD).is_some() {
        two_draws_a_merge_apart();
        return;
    }
    let root =
        std::env::temp_dir().join(format!("rcad-fuzz-timing-read-back-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let into = root.join(TARGET);
    std::fs::create_dir_all(&into).expect("somewhere to write");
    std::fs::write(
        into.join("totals"),
        "runs 5\nnanos 5000000000\nlost 3\nb 500 5\n",
    )
    .expect("a totals file another process left");

    run_this_test_as_a_child(&root);

    let report = std::fs::read_to_string(into.join("report")).expect("the merge wrote a report");
    assert!(
        report.starts_with("7 runs, 5.0 s in all\n"),
        "the `runs` or the `nanos` line was not read back: {report}"
    );
    assert!(
        report.contains("3 of those runs were lost"),
        "the `lost` line was not read back: {report}"
    );
    // Bucket 500 starts at 20 << 30 ns. Nothing but the `b` line puts a run
    // there: the child's two draws each take well under a millisecond.
    assert!(
        report.contains(" 21474.836 ms"),
        "the `b` line was not read back: {report}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Two draws more than a merge interval apart, so the second one merges.
fn two_draws_a_merge_apart() {
    rcad_fuzz_timing::timed(TARGET, b"first", || ());
    std::thread::sleep(Duration::from_millis(1_100));
    rcad_fuzz_timing::timed(TARGET, b"second", || ());
}

fn run_this_test_as_a_child(root: &Path) {
    let status = Command::new(std::env::current_exe().expect("this test's own binary"))
        .args([
            "a_merge_reads_back_every_word_another_process_wrote_to_totals",
            "--exact",
            "--nocapture",
        ])
        .env(THE_CHILD, "1")
        .env("RCAD_FUZZ_TIMINGS", root)
        .status()
        .expect("the test binary runs again as a child");
    assert!(status.success(), "the child failed: {status}");
}
