"""The queue page's fuzzing section: what the overnight fuzz job did, night by night.

Read from the fuzz log root `render.py` is handed — `just queue-page` passes
`RCAD_FUZZ_LOGS`, or `~/rcad-fuzz-logs` without it — through
`tooling/fuzz-yield/log_stats.py` and `nights.py`, the two files `just
fuzz-yield` prints from, so the page and that report cannot disagree about a
night.

The coverage chart needs every night's logs, and reading them all took 1.6 s
cold (94 logs, 79 MB, 2026-10-05) on a page that re-renders after every queue
write, and grows by five logs a night. So a log's summary is kept in the
cache `render.py` hands in, under the log's size and mtime: a finished night's
logs never change again, and the night running now is read afresh. New files
and faults are read only for the nights in the table, because each night's
artifacts directory is a cumulative copy that grows with every night before it.
"""

import datetime
import html
import os
import sys

FUZZ_YIELD_TOOLING = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "fuzz-yield"))
sys.path.insert(0, FUZZ_YIELD_TOOLING)
import log_stats  # noqa: E402
import nights  # noqa: E402

NIGHTS_IN_TABLE = 7


def section(root, cache):
    """The section's HTML, and the cache to keep for the next render."""
    names = nights.night_names(root)
    if not names:
        return f"<p>No fuzz nights under <code>{html.escape(root)}</code>.</p>", {}
    summaries, cache = read_summaries(root, names, cache)
    latest = latest_line(root)
    read_at = datetime.datetime.now().strftime("%Y-%m-%d %H:%M")
    parts = [
        f'<p class="qp-sub">Read from <code>{html.escape(root)}</code> at {read_at}.</p>',
        latest_html(latest),
        coverage_chart(names, summaries),
        nights_table(root, names, summaries, latest),
        NOTE,
    ]
    return "\n".join(parts), cache


def read_summaries(root, names, cache):
    """`{night: [(log name, Summary)]}` for every night, and the cache holding exactly those logs.

    A cache taken under another shape of `Summary` is dropped whole, as
    `render.py`'s history cache drops a tally taken under another list of tiers.
    """
    fields = list(log_stats.Summary._fields)
    old = cache.get("logs", {}) if cache.get("fields") == fields else {}
    kept = {}
    summaries = {}
    for night in names:
        summaries[night] = []
        for log in nights.log_names(root, night):
            path = os.path.join(root, night, log)
            stat = os.stat(path)
            key = [stat.st_size, stat.st_mtime_ns]
            entry = old.get(path)
            if entry is None or entry["key"] != key:
                entry = {"key": key, "summary": list(log_stats.summarise(path))}
            kept[path] = entry
            summaries[night].append((log, log_stats.Summary(*entry["summary"])))
    return summaries, {"fields": fields, "logs": kept}


def latest_line(root):
    """What the last night that finished wrote to `latest`, or `None` when nothing has."""
    try:
        with open(os.path.join(root, "latest"), encoding="utf-8") as f:
            return f.read().strip() or None
    except FileNotFoundError:
        return None


def latest_html(latest):
    if latest is None:
        return "<p>No <code>latest</code> line: no night has finished yet.</p>"
    return f'<p>The last night that finished said:</p>\n<pre class="qp-latest">{html.escape(latest)}</pre>'


def coverage_chart(names, summaries):
    by_target = {}
    for night in names:
        for _log, summary in summaries[night]:
            if summary.status == "OK":
                by_target.setdefault(summary.target, {})[night] = summary.cov_end
    if not by_target:
        return "<p>No night on disk has a coverage line in any log.</p>"
    first = min(night for by_night in by_target.values() for night in by_night)
    days = calendar_days(first, names[-1])
    panels = "".join(coverage_panel(target, by_target[target], days) for target in sorted(by_target))
    caption = ("Coverage at the end of each night, one panel per target, each on its own scale. "
               "A step between two nights is mostly new code arriving with a new commit, not a find (§265.4). "
               "A gap is a night with no coverage line for that target.")
    return f"""
<figure class="qp-chart qp-wide" id="qp-coverage">
  <figcaption>{html.escape(caption)}</figcaption>
  <div class="qp-multiples">{panels}</div>
</figure>"""


def calendar_days(first, last):
    start = datetime.date.fromisoformat(first)
    count = (datetime.date.fromisoformat(last) - start).days + 1
    return [(start + datetime.timedelta(days=i)).isoformat() for i in range(count)]


def coverage_panel(target, by_night, days):
    width, height = 320, 130
    margin = {"top": 10, "right": 8, "bottom": 28, "left": 44}
    plot_w = width - margin["left"] - margin["right"]
    plot_h = height - margin["top"] - margin["bottom"]
    lo = min(by_night.values())
    hi = max(by_night.values())

    def x_of(i):
        return margin["left"] + (plot_w * i / (len(days) - 1) if len(days) > 1 else plot_w / 2)

    def y_of(v):
        return margin["top"] + (plot_h * (1 - (v - lo) / (hi - lo)) if hi > lo else plot_h / 2)

    runs = []
    for i, day in enumerate(days):
        if day not in by_night:
            continue
        if runs and runs[-1][-1][0] == i - 1:
            runs[-1].append((i, by_night[day]))
        else:
            runs.append([(i, by_night[day])])
    lines = "".join(
        '<polyline class="qp-cov-line" points="'
        + " ".join(f"{x_of(i):.1f},{y_of(v):.1f}" for i, v in run)
        + '"/>'
        for run in runs if len(run) > 1
    )
    dots = "".join(
        f'<circle class="qp-cov-dot" cx="{x_of(i):.1f}" cy="{y_of(by_night[day]):.1f}" r="4">'
        f"<title>{day}: {by_night[day]} at the end of the night</title></circle>"
        for i, day in enumerate(days) if day in by_night
    )
    axis_y = height - 4
    return f"""
<div class="qp-panel">
  <div class="qp-panel-title"><code>{html.escape(target)}</code></div>
  <svg viewBox="0 0 {width} {height}" role="img" aria-label="coverage at the end of each night for {html.escape(target)}">
    <text class="qp-axis" x="{margin['left'] - 6}" y="{y_of(hi) + 3:.1f}" text-anchor="end">{hi}</text>
    <text class="qp-axis" x="{margin['left'] - 6}" y="{y_of(lo) + 3:.1f}" text-anchor="end">{lo}</text>
    {lines}{dots}
    <text class="qp-axis" x="{margin['left']}" y="{axis_y}">{days[0]}</text>
    <text class="qp-axis" x="{width - margin['right']}" y="{axis_y}" text-anchor="end">{days[-1]}</text>
  </svg>
</div>"""


def nights_table(root, names, summaries, latest):
    newest = names[-1]
    days = list(reversed(calendar_days(table_start(newest), newest)))
    finished = latest is not None and os.path.basename(latest.split()[-1]) == newest
    rows = []
    for day in days:
        night_cell = day
        if day == newest and not finished:
            night_cell += ('<div class="qp-note">not in <code>latest</code> yet: '
                           "still running, or ended without writing it</div>")
        if day not in summaries:
            rows.append(f'<tr><th>{night_cell}</th><td colspan="9">no directory for this night</td></tr>')
        elif not summaries[day]:
            rows.append(f'<tr><th>{night_cell}</th><td colspan="9">{nothing_fuzzed(root, day)}</td></tr>')
        else:
            rows.extend(night_rows(root, names, day, night_cell, summaries[day]))
    return f"""
<h3>The last {NIGHTS_IN_TABLE} nights</h3>
<table class="qp-fuzz">
<thead><tr><th>Night</th><th>Target</th><th>Core-hours</th><th>Runs</th>
<th>Coverage at start</th><th>at 30 min</th><th>at end</th><th>Last rose (hour)</th>
<th>New files</th><th>Faults</th></tr></thead>
<tbody>
{"".join(rows)}
</tbody>
</table>"""


def table_start(newest):
    return (datetime.date.fromisoformat(newest) - datetime.timedelta(days=NIGHTS_IN_TABLE - 1)).isoformat()


def nothing_fuzzed(root, day):
    build_log = os.path.join(root, day, "build.log")
    if os.path.exists(build_log):
        return f"fuzzed nothing: no fuzz logs. The build's own log is <code>{html.escape(build_log)}</code>"
    return "fuzzed nothing: no fuzz logs, and no build log"


def night_rows(root, names, day, night_cell, logs):
    rows = []
    for i, (log, summary) in enumerate(logs):
        head = f'<th rowspan="{len(logs)}">{night_cell}</th>' if i == 0 else ""
        rows.append(f"<tr>{head}{target_cells(root, names, day, log, summary)}</tr>")
    return rows


def target_cells(root, names, day, log, summary):
    log_name = html.escape(log[: -len(".log")])
    if summary.status == "SKIPPED":
        return f'<td><code>{log_name}</code></td><td colspan="8">skipped: no time left in the window</td>'
    if summary.status == "NOLAUNCH":
        return f'<td><code>{log_name}</code></td><td colspan="8">never launched: no <code>Running</code> line in this log</td>'
    target = html.escape(summary.target)
    if summary.status == "NODATA":
        return f'<td><code>{target}</code></td><td colspan="8">launched, but no coverage line was ever printed</td>'
    assert summary.status == "OK", f"log_stats.py said {summary.status!r} for {log}, which the page does not know"
    cov_30 = summary.cov_30 if summary.cov_30 is not None else "n/a"
    found = nights.new_files(root, names, day, summary.target)
    fault_count = nights.faults(root, day, summary.target)
    cells = [
        f"<td><code>{target}</code></td>",
        f'<td class="n">{summary.fork * summary.window_s / 3600:.1f}</td>',
        f'<td class="n" title="{summary.runs}">{compact(summary.runs)}</td>',
        f'<td class="n">{summary.cov_start}</td>',
        f'<td class="n">{cov_30}</td>',
        f'<td class="n">{summary.cov_end}</td>',
        f'<td class="n">{summary.last_rise_s / 3600:.1f}</td>',
        f"<td>{new_files_text(found, day)}</td>",
        f"<td class=\"faults\">{'not triaged' if fault_count is None else fault_count}</td>",
    ]
    return "".join(cells)


def compact(n):
    for size, suffix in ((10**9, "B"), (10**6, "M"), (10**3, "K")):
        if n >= size:
            return f"{n / size:.1f}{suffix}"
    return str(n)


def new_files_text(found, day):
    if found is None:
        return "no artifacts directory"
    if isinstance(found, nights.Baseline):
        return f"first night on disk: all {found.count} counted"
    text = nights.kinds_text(found.by_kind) if found.by_kind else "none"
    night_before = (datetime.date.fromisoformat(day) - datetime.timedelta(days=1)).isoformat()
    if found.since != night_before:
        text += f" (since {found.since})"
    return html.escape(text)


NOTE = """
<p class="qp-note">Each new file is named for what libFuzzer saw: <code>crash</code>, <code>oom</code> and
<code>leak</code> are a bug, <code>timeout</code> and <code>slow-unit</code> a slow input. A crash file is
not a fault: one fault can write thousands of them (§265.2). Faults are what <code>just fuzz-triage</code>
found still failing on main, and a night it has not read says not triaged. <code>just fuzz-yield</code>
prints the same numbers for every night on disk.</p>"""
