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
        runs=rows[-1][2],
        cov_start=rows[0][1],
        cov_30=cov_30,
        cov_end=rows[-1][1],
        last_rise_s=last_rise_seconds(rows),
    )


def parse_log(path):
    skipped = False
    target = fork = window_s = None
    rows = []
    with open(path, errors="replace") as f:
        for line in f:
            if SKIPPED_LINE in line:
                skipped = True
                continue
            if target is None and "Running `" in line:
                launch = LAUNCH_LINE.search(line)
                fork_m = FORK_FLAG.search(line)
                window_m = WINDOW_FLAG.search(line)
                if launch and fork_m and window_m:
                    target = launch.group("target")
                    fork = int(fork_m.group(1))
                    window_s = int(window_m.group(1))
                continue
            m = RUN_LINE.search(line)
            if m:
                rows.append((int(m["time"]), int(m["cov"]), int(m["run"])))
    return skipped, target, fork, window_s, rows


def coverage_at_or_before(rows, seconds):
    best = None
    for time_s, cov, _run in rows:
        if time_s <= seconds:
            best = cov
    return best


def last_rise_seconds(rows):
    rise = 0
    for i in range(1, len(rows)):
        if rows[i][1] > rows[i - 1][1]:
            rise = rows[i][0]
    return rise


def main():
    print(stats_line(sys.argv[1]))


if __name__ == "__main__":
    main()
