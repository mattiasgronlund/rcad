//! A forked libFuzzer child that reaches `-max_total_time` or finds the stop
//! file ends by calling `exit(0)`, not by being killed. Before this lane,
//! nothing here merged on that path: a draw that finished less than a second
//! after the last merge left its run stuck in the process's own memory, gone
//! the moment the process ended (§267.6, §278). This runs the crate through
//! [`rcad_fuzz_timing::timed`] in a child process that does exactly one draw
//! and then calls `exit(0)` itself, well inside the one-second merge
//! interval, and checks that the parent still finds that run in `totals`.

use std::process::Command;

const THE_CHILD: &str = "RCAD_FUZZ_TIMING_EXIT_CHILD";
const TARGET: &str = "a-target";

#[test]
fn a_child_that_exits_before_a_merge_is_due_still_leaves_its_run_in_totals() {
    if std::env::var_os(THE_CHILD).is_some() {
        rcad_fuzz_timing::timed(TARGET, b"one draw, no merge due yet", || ());
        std::process::exit(0);
    }
    let root = std::env::temp_dir().join(format!(
        "rcad-fuzz-timing-exit-merge-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let into = root.join(TARGET);

    let status = Command::new(std::env::current_exe().expect("this test's own binary"))
        .args([
            "a_child_that_exits_before_a_merge_is_due_still_leaves_its_run_in_totals",
            "--exact",
            "--nocapture",
        ])
        .env(THE_CHILD, "1")
        .env("RCAD_FUZZ_TIMINGS", &root)
        .status()
        .expect("the test binary runs again as a child");
    assert!(status.success(), "the child failed: {status}");

    let totals = std::fs::read_to_string(into.join("totals")).unwrap_or_default();
    assert!(
        totals.starts_with("runs 1\n"),
        "the child's one draw never reached totals, though it ended with exit(0): {totals:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}
