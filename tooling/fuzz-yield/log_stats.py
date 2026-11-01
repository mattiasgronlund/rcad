#!/usr/bin/env python3
"""Coverage and run counts for one night's fuzz log.

This is the arithmetic behind `docs/decisions.md` §265.4: the coverage a
target had at the start, at 30 minutes, and at the end of the night, and the
hour its coverage last went up. `just fuzz-yield` runs this once per log file
and reads the answer back as one tab-separated line; the queue page imports
`summarise` and reads the same answer as a `Summary`.

Kept separate from the justfile recipe because this is the half a Python
`unittest` can hold without a fixture night on disk — `check-fuzz-yield-selftest`
covers the shell around it (which files to read, how to fold the fuzz-triage
verdicts in); this covers only "what does this log say".
"""

import re
import sys
from collections import namedtuple

RUN_LINE = re.compile(
    r"#(?P<run>\d+): cov: (?P<cov>\d+) ft: \d+ corp: \d+ exec/s: \d+ "
    r"oom/timeout/crash: \d+/\d+/\d+ time: (?P<time>\d+)s"
)

# `-fork` and `-max_total_time` appear on the same `Running \`...\`` line but
# not in a fixed order — `fuzz.log`'s target carries `-max_len` between them,
# `fuzz-fuse.log`'s does not — so this pulls each flag out on its own rather
# than matching the line as one fixed shape.
LAUNCH_LINE = re.compile(r"Running `.*/release/(?P<target>\S+) ")
FORK_FLAG = re.compile(r"-fork=(\d+)")
WINDOW_FLAG = re.compile(r"-max_total_time=(\d+)")

# What `fuzz-overnight` writes into a target's log in place of starting it,
# when the window has no time left: `<recipe>: skipped, <n> s left in the window`.
SKIPPED_LINE = ": skipped,"

# `status` is one of SKIPPED, NOLAUNCH, NODATA and OK. A SKIPPED or NOLAUNCH
# log names no target, and only an OK one has the counts after `target`; the
# fields a status does not have are None.
Summary = namedtuple("Summary", "status target fork window_s runs cov_start cov_30 cov_end last_rise_s")


def stats_line(path):
    summary = summarise(path)
    if summary.status in ("SKIPPED", "NOLAUNCH"):
        return summary.status
    if summary.status == "NODATA":
        return "NODATA\t" + summary.target
    fields = [
        "OK",
        summary.target,
        str(summary.fork),
        str(summary.window_s),
        str(summary.runs),
        str(summary.cov_start),
        str(summary.cov_30) if summary.cov_30 is not None else "n/a",
        str(summary.cov_end),
        str(summary.last_rise_s),
    ]
    return "\t".join(fields)


def summarise(path):
    skipped, target, fork, window_s, rows = parse_log(path)
    if skipped:
        return Summary("SKIPPED", *[None] * 8)
    if target is None:
        return Summary("NOLAUNCH", *[None] * 8)
    if not rows:
        return Summary("NODATA", target, *[None] * 7)
    cov_30 = coverage_at_or_before(rows, 1800)
    if cov_30 is None:
        cov_30 = coverage_at_or_before(rows, 3600)
    return Summary(
        status="OK",
        target=target,
        fork=fork,
        window_s=window_s,
        runs=executions(rows),
        cov_start=rows[0][1],
        cov_30=cov_30,
        cov_end=rows[-1][1],
        last_rise_s=last_rise_seconds(rows),
    )


# A log holds one run per launch line it can read: two on a night with a pause
# (docs/decisions.d/475), and one otherwise. libFuzzer counts each run's time
# and executions from zero, so a row's time here is counted from the start of
# the first run, by adding the windows of the runs before it, and its last field
# says which run it belongs to. `window_s` is the sum of the runs' windows.
def parse_log(path):
    skipped = False
    target = fork = None
    window_s = 0
    offset = 0
    launch_index = -1
    rows = []
    with open(path, errors="replace") as f:
        for line in f:
            if SKIPPED_LINE in line:
                skipped = True
                continue
            if "Running `" in line:
                launch = LAUNCH_LINE.search(line)
                fork_m = FORK_FLAG.search(line)
                window_m = WINDOW_FLAG.search(line)
                if launch and fork_m and window_m:
                    this_target = launch.group("target")
                    this_fork = int(fork_m.group(1))
                    if target is not None and (this_target, this_fork) != (target, fork):
                        raise ValueError(
                            f"{path} holds a run of {this_target} with -fork={this_fork} "
                            f"after one of {target} with -fork={fork}, and one night runs one target one way"
                        )
                    target, fork = this_target, this_fork
                    offset = window_s
                    window_s += int(window_m.group(1))
                    launch_index += 1
                continue
            m = RUN_LINE.search(line)
            if m:
                rows.append((offset + int(m["time"]), int(m["cov"]), int(m["run"]), launch_index))
    if target is None:
        window_s = None
    return skipped, target, fork, window_s, rows


def executions(rows):
    last_of_each_run = {}
    for _time_s, _cov, count, launch_index in rows:
        last_of_each_run[launch_index] = count
    return sum(last_of_each_run.values())


def coverage_at_or_before(rows, seconds):
    best = None
    for time_s, cov, _count, _launch_index in rows:
        if time_s <= seconds:
            best = cov
    return best


# A rise is counted only within one run: from the last row of one run to the
# first of the next is a new process starting, not a find.
def last_rise_seconds(rows):
    rise = 0
    for i in range(1, len(rows)):
        if rows[i][3] == rows[i - 1][3] and rows[i][1] > rows[i - 1][1]:
            rise = rows[i][0]
    return rise


def main():
    print(stats_line(sys.argv[1]))


if __name__ == "__main__":
    main()
