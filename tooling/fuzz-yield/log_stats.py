#!/usr/bin/env python3
"""Coverage and run counts for one night's fuzz log.

This is the arithmetic behind `docs/decisions.md` §265.4: the coverage a
target had at the start, at 30 minutes, and at the end of the night, and the
hour its coverage last went up. `just fuzz-yield` runs this once per log file
and reads the answer back as one tab-separated line.

Kept separate from the justfile recipe because this is the half a Python
`unittest` can hold without a fixture night on disk — `check-fuzz-yield-selftest`
covers the shell around it (which files to read, how to fold the fuzz-triage
verdicts in); this covers only "what does this log say".
"""

import re
import sys

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


def parse_log(path):
    target = fork = window_s = None
    rows = []
    with open(path, errors="replace") as f:
        for line in f:
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
    return target, fork, window_s, rows


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


def stats_line(path):
    target, fork, window_s, rows = parse_log(path)
    if target is None:
        return "NOLAUNCH"
    if not rows:
        return "NODATA\t" + target
    start = rows[0][1]
    end = rows[-1][1]
    runs = rows[-1][2]
    cov_30 = coverage_at_or_before(rows, 1800)
    if cov_30 is None:
        cov_30 = coverage_at_or_before(rows, 3600)
    fields = [
        "OK",
        target,
        str(fork),
        str(window_s),
        str(runs),
        str(start),
        str(cov_30) if cov_30 is not None else "n/a",
        str(end),
        str(last_rise_seconds(rows)),
    ]
    return "\t".join(fields)


def main():
    print(stats_line(sys.argv[1]))


if __name__ == "__main__":
    main()
