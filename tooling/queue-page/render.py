#!/usr/bin/env python3
"""Render docs/queue.md's history and current state as one HTML page (Q254).

`just queue-page` runs this, writing to `$RCAD_QUEUE_PAGE_DIR` (default
`~/rcad-queue-page`). The `queue` recipe runs `just queue-page` after every
verb that writes, and so does `ff-main`, so the page stays current as a side
effect of the queue work rather than needing its own trigger. It reads
**main's** copy of `docs/queue.md` — never this worktree's own, which may sit
on a lane branch — plus the claims file `tooling/queue/queue.py` also reads,
and writes a self-contained page: rows added against rows closed per day, two
SVG charts of the queue's shape over every commit of `docs/queue.md` on main's
first-parent line, a table of both by day, a table of the rows running now with
the tier each was taken from and the agent it runs under, a table of the open
rows Mattias pinned and what holds each back, a table of why a fill would not
take an open row now and which rows each reason holds back, then the current
ready, running and lately-closed rows, and what the overnight fuzz job did
night by night — that last section is `fuzz_section.py`'s, read from the fuzz log root the caller
names.
`docs/decisions.md` §230, §263.

Every row of main's copy also gets a page of its own, `<out>/rows/<id>.html`:
its tier, its class, the rows it came from as `just lineage` reads them, the
files it expects to change, and the row itself. The front page names a row by
its id alone, and every Q id on either kind of page links to that row's page
(§444).

The state this keeps across runs is `<out>/.history-cache.json`, one entry
per commit sha already tallied, `<out>/.fuzz-cache.json`, one entry per
fuzz log already read (`fuzz_section.py` says why), and
`<out>/.lineage-cache.json`, what each commit did to the rows, which is what
`lineage.Graph` reads. Recomputing every commit on every write
would make the render the queue pays for grow with the queue's own history;
a sha's tally changes only when the list of tiers does, and `load_cache` drops
an entry taken under another list. Measured through `just queue-page` against
the real history (600 first-parent commits touching docs/queue.md,
2026-10-04): 19s cold, 1.7s warm. The lineage cache is the dearer one cold:
with it empty the render took 235 s at c8a9eda5, since lineage then parses
every copy of the 1.7 MB file that main's history holds. With all three caches
full it took 3.4 s, 0.64 s of it the lineage and 0.26 s the row pages (§444).

A failure here — no `main` here, no commit ever touched the file — is said on
stderr and returned as a nonzero exit, never raised past `main`: the caller
(`just queue`, `just ff-main`) must keep going either way.
"""

import html
import json
import os
import re
import subprocess
import sys
import tempfile
from collections import namedtuple
from datetime import datetime, timezone

sys.dont_write_bytecode = True
QUEUE_TOOLING = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "queue"))
sys.path.insert(0, QUEUE_TOOLING)
sys.path.insert(0, os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "lineage")))
from rows import SMALL, Queue, TIERS_IN_ORDER  # noqa: E402
import fuzz_section  # noqa: E402
import lineage  # noqa: E402

QUEUE_PATH = "docs/queue.md"
CLOSED_ROWS_SHOWN = 12
# The two lines `just queue blocked` prints: a row's path meets one hold, or it
# is one of `queue_shared_files` and two holds already have it (§424.4). The
# blocked mode takes no row, so neither names a row taken above.
MEETS_LINE = re.compile(r"^skip (\S+) \(tier (\S+)\): its (\S+) meets (\S+)'s (\S+), held (by|elsewhere by) (\S+)$")
SHARED_LINE = re.compile(
    r"^skip (\S+) \(tier (\S+)\): its (\S+) is shared, and "
    r"(\S+) \(held (by|elsewhere by) (\S+)\) and (\S+) \(held (by|elsewhere by) (\S+)\) already hold it$")
ANY_ROW_ID = re.compile(r"^[TDQ][0-9]+$")
ROW_REF = re.compile(r"(?<![0-9A-Za-z])Q[0-9]+(?![0-9A-Za-z])")


def main(argv):
    if len(argv) != 2:
        print("usage: render.py <out-dir> <fuzz-log-root>", file=sys.stderr)
        return 2
    out_dir, fuzz_root = argv
    git = Git()
    if git.rev_parse("refs/heads/main") is None:
        print("queue-page: no 'main' branch here, so there is nothing to render", file=sys.stderr)
        return 1
    history = git.history(QUEUE_PATH)
    if not history:
        print(f"queue-page: no commit on main's first-parent line touched {QUEUE_PATH}", file=sys.stderr)
        return 1
    cache_path = os.path.join(out_dir, ".history-cache.json")
    cache = load_cache(cache_path)
    points = [tallied(git, sha, when, cache) for sha, when in history]
    save_cache(cache_path, cache)
    tip_text = git.show(history[-1][0], QUEUE_PATH)
    tip = Queue(tip_text)
    summaries_path = os.path.join(out_dir, ".lineage-cache.json")
    summaries = load_summaries(summaries_path)
    graph = lineage.Graph(os.getcwd(), "refs/heads/main", summaries)
    write_atomically(summaries_path, json.dumps({"shape": lineage.SUMMARY_SHAPE, "commits": summaries}, sort_keys=True))
    fuzz_cache_path = os.path.join(out_dir, ".fuzz-cache.json")
    fuzz, fuzz_cache = fuzz_section.section(fuzz_root, read_json(fuzz_cache_path))
    write_atomically(fuzz_cache_path, json.dumps(fuzz_cache, sort_keys=True))
    rendered_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    page = render_page(points, current_state(tip, tip_text), fuzz, rendered_at, Links(tip, "rows/"))
    write_row_pages(os.path.join(out_dir, "rows"), tip, graph)
    write_atomically(os.path.join(out_dir, "index.html"), page)
    return 0


class Git:
    """Plumbing against the repository the caller's working directory is in.

    No `-C`: the caller sets the working directory (`--working-directory` on
    `just`, or a plain `cd`), the same convention `tooling/queue/queue.py`
    already relies on for `just check-queue-api-selftest`'s scratch clones.
    """

    def rev_parse(self, ref):
        result = subprocess.run(["git", "rev-parse", "--verify", "--quiet", ref], capture_output=True, text=True)
        return result.stdout.strip() if result.returncode == 0 else None

    def history(self, path):
        """Every commit on `main`'s first-parent line that touched `path`, oldest first."""
        result = subprocess.run(
            ["git", "log", "--first-parent", "--reverse", "--format=%H%x1f%cI", "refs/heads/main", "--", path],
            capture_output=True, text=True, check=True)
        pairs = []
        for line in result.stdout.splitlines():
            sha, when = line.split("\x1f")
            pairs.append((sha, when))
        return pairs

    def show(self, sha, path):
        result = subprocess.run(["git", "show", f"{sha}:{path}"], capture_output=True, text=True)
        return result.stdout if result.returncode == 0 else ""


Point = namedtuple("Point", "sha when counts")


def load_cache(path):
    """The tallies earlier runs wrote, less those taken under another list of tiers.

    Such a tally has no count for a tier added since, and counted that tier's
    rows as parked (§403.8), so its commit is tallied again.
    """
    cache = read_json(path)
    return {sha: counts for sha, counts in cache.items() if set(counts["by_tier"]) == set(TIERS_IN_ORDER)}


def load_summaries(path):
    """What each commit did to the rows, as earlier renders kept it, or nothing when it was kept under another shape."""
    cache = read_json(path)
    return cache["commits"] if cache.get("shape") == lineage.SUMMARY_SHAPE else {}


def read_json(path):
    """A cache file's contents, or an empty cache when there is none or it does not parse."""
    try:
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    except (OSError, ValueError):
        return {}


def save_cache(path, cache):
    write_atomically(path, json.dumps(cache, sort_keys=True))


def tallied(git, sha, when, cache):
    if sha not in cache:
        cache[sha] = tally(git.show(sha, QUEUE_PATH))
    return Point(sha, when, cache[sha])


def tally(text):
    """One commit's shape: open rows by tier, parked rows, and how many are closed.

    A row is closed, or parked, or open under its tier — the three are
    exclusive, and a tier `check-queue` would have refused (§230.2) is folded
    into whichever count it is not, rather than dropped, so a bad historical
    commit still balances against the total the page's own count can check.
    """
    by_tier = {tier: 0 for tier in TIERS_IN_ORDER}
    parked = 0
    closed = 0
    for row in Queue(text).rows:
        if row.is_closed():
            closed += 1
        elif row.state.startswith("parked:"):
            parked += 1
        elif row.tier in by_tier:
            by_tier[row.tier] += 1
        else:
            parked += 1
    return {"by_tier": by_tier, "parked": parked, "closed": closed}


def current_state(tip, tip_text):
    verbs = load_queue_verbs()
    here = verbs.Here(QUEUE_PATH)
    return {
        "ready": ready_rows(tip),
        "running": running_rows(verbs, here, tip),
        "pinned": pinned_rows(tip),
        "not_taken": rows_not_taken(tip, tip_text, here.trees()),
        "closed": closed_rows(tip),
    }


def load_queue_verbs():
    """`tooling/queue/queue.py`, loaded under its own name so it never shadows the stdlib `queue` module."""
    import importlib.util
    spec = importlib.util.spec_from_file_location("rcad_queue_verbs", os.path.join(QUEUE_TOOLING, "queue.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def ready_rows(tip):
    by_tier = {tier: [] for tier in TIERS_IN_ORDER}
    for row in tip.rows:
        ready = row.state == "todo" or (row.state.startswith("after:") and not not_yet_done(tip, row))
        if ready and row.tier in by_tier:
            by_tier[row.tier].append(row)
    return by_tier


def running_rows(verbs, here, tip):
    """What `just queue running` would say, joined back to the row it names (§263.4).

    The claims file, not the file's own `lane:` state, is what a running lane
    actually holds right now — a take can be committed on a branch main has
    not merged yet, so the row it names may not even be `lane:` in main's copy.
    """
    trees = here.trees()
    branches = {tree.branch for tree in trees if tree.is_lane}
    holds = sorted(verbs.Claims(here.claims).holds().items())
    left = verbs.branches_left(trees, {hold["branch"] for _, hold in holds})
    claimed = set(left.values())
    rows = []
    for row_id, hold in holds:
        before = set(claimed)
        found = verbs.has_a_tree(hold["branch"], branches, trees, claimed, left)
        tree = tree_for(hold, trees, found, claimed - before, left)
        rows.append({
            "id": row_id,
            "branch": hold["branch"],
            "where": tree.path if tree else f"no tree yet, taken from {hold['tree']}",
            "agent": agent_of(tree) if tree else None,
            "tier": tier_taken_from(hold, tip.find(row_id)),
            "taken_by": os.path.basename(hold["tree"]),
            "write_set": hold["write_set"],
            "taken": hold["taken"],
        })
    return rows


def tree_for(hold, trees, found, newly_claimed, left):
    """The lane tree `has_a_tree` matched this hold to, or None when it matched none.

    Only a tree an agent holds counts, as in the `branches` `has_a_tree` was
    given: a lane's own tree left standing unlocked beside its merger's is not
    where the row runs. A tree that left the hold's branch (§299.6) was
    claimed before any row was matched, so it is never among `newly_claimed`.
    """
    if newly_claimed:
        (path,) = newly_claimed
        return next(tree for tree in trees if tree.path == path)
    if hold["branch"] in left:
        return next(tree for tree in trees if tree.path == left[hold["branch"]])
    if not found:
        return None
    for tree in trees:
        if not tree.is_lane:
            continue
        if tree.branch == hold["branch"]:
            return tree
        if hold["branch"].startswith("lane-") and tree.branch == "merge-" + hold["branch"][len("lane-"):]:
            return tree
    raise AssertionError(f"has_a_tree found a lane tree for {hold['branch']}, and no lane tree carries it")


def agent_of(tree):
    """The agent's name in a lane tree's lock, `agent <name> since <time>` (§55)."""
    assert tree.is_lane, f"{tree.path} is not a lane tree, so its lock names no agent: {tree.locked!r}"
    return tree.locked[len("agent "):].split(" since ")[0]


def tier_taken_from(hold, row):
    """The tier the take line records, or the row's tier in main's copy for a take line that records none."""
    if hold["tier"] is not None:
        return hold["tier"]
    return row.tier if row else None


def pinned_rows(tip):
    """The open rows Mattias pinned, by their number, which is the order a fill takes them in (§430.3).

    A closed row keeps its pin as the record that it was pinned (§430.4), so
    the closed ones are left out: they hold nothing back and wait on nothing.
    """
    return sorted((row for row in tip.rows if row.pin is not None and not row.is_closed()), key=lambda row: row.pin)


def rows_not_taken(tip, tip_text, trees):
    """Every open row a fill would not take now, even with every slot free, grouped by why (Q1510).

    A row waits on rows not yet done, or is parked, or is ready and its write
    set meets a running lane's. The first two are read from main's copy here,
    in tier order as `just queue` lists its waiting rows. The third is the
    `queue` recipe's `blocked` mode, since the rule for when two write sets meet
    lives in its awk alone (§230.3, §230.6). A row that meets only a row the
    same fill would take above it is held back by nothing, and `blocked` leaves
    it out.
    """
    waiting = []
    parked = []
    for tier in TIERS_IN_ORDER:
        for row in tip.rows:
            if row.tier != tier:
                continue
            if row.state.startswith("after:"):
                unfinished = not_yet_done(tip, row)
                if unfinished:
                    waiting.append((row, unfinished))
            elif row.state.startswith("parked:"):
                parked.append(row)
    return {"waiting": waiting, "parked": parked, "meeting": meeting_a_running_lane(tip_text, trees)}


def not_yet_done(tip, row):
    """The rows an `after:` row waits on that are not `done:`, each with its state.

    A dropped row is not done, as `just queue` and `take` read it.
    """
    unfinished = []
    for other in row.state[len("after:"):].split(","):
        found = tip.find(other)
        if found is None or not found.state.startswith("done:"):
            unfinished.append((other, found.state if found else "not a row"))
    return unfinished


def meeting_a_running_lane(tip_text, trees):
    """`just queue blocked` run in the tree holding main, or None when no tree holding main has main's own copy.

    It runs there, on that tree's copy, because the holds it steers round are
    read relative to the tree it runs in (§263.4): so its lines are the ones
    `just queue next` there would pass rows over with. A copy that is not
    main's commit, as during a merge in that tree, describes a queue the rest
    of the page does not show, so it is not run on one.
    """
    for tree in trees:
        if tree.branch == "main" and queue_copy_in(tree) == tip_text:
            return blocked_lines(tree)
    return None


def queue_copy_in(tree):
    with open(os.path.join(tree.path, QUEUE_PATH), encoding="utf-8") as f:
        return f.read()


def blocked_lines(tree):
    result = subprocess.run(
        [os.environ["QUEUE_JUST"], "--justfile", os.environ["QUEUE_JUSTFILE"],
         "--working-directory", tree.path, "queue", "blocked"],
        capture_output=True, text=True)
    assert result.returncode == 0, f"`just queue blocked` failed on main's own copy in {tree.path}: {result.stderr.strip()}"
    lines = []
    for line in result.stdout.splitlines():
        meets = MEETS_LINE.match(line)
        shared = SHARED_LINE.match(line)
        assert meets or shared, f"`just queue blocked` printed a line this cannot read: {line!r}"
        if meets:
            row_id, _, mine, other, theirs, how, holder = meets.groups()
            lines.append({"id": row_id, "mine": mine, "shared": False, "holders": [(other, theirs, how, holder)]})
        else:
            row_id, _, mine, first, first_how, first_holder, second, second_how, second_holder = shared.groups()
            lines.append({"id": row_id, "mine": mine, "shared": True,
                          "holders": [(first, None, first_how, first_holder), (second, None, second_how, second_holder)]})
    return lines


def closed_rows(tip):
    closed = [row for row in tip.rows if row.is_closed()]
    return closed[-CLOSED_ROWS_SHOWN:]


def by_day(points):
    """Rows added and rows closed per day, and the day's last tally, from the same per-commit tallies as the charts.

    Both totals are monotonic — a row is never removed once added, and a
    closed row never reopens — so a day's count is just the last commit seen
    that day, and the delta against the previous day is never negative. A day
    no commit touched the queue on still gets an entry: nothing added, nothing
    closed, and the day before's tally, so a quiet day shows as one.
    """
    latest_of_day = {}
    for point in points:
        latest_of_day[point.when[:10]] = point
    previous_total = 0
    previous_closed = 0
    days = []
    calendar = fuzz_section.calendar_days(min(latest_of_day), max(latest_of_day))
    counts = latest_of_day[calendar[0]].counts
    for day in calendar:
        if day in latest_of_day:
            counts = latest_of_day[day].counts
        total = sum(counts["by_tier"].values()) + counts["parked"] + counts["closed"]
        days.append({
            "day": day,
            "added": max(0, total - previous_total),
            "closed": max(0, counts["closed"] - previous_closed),
            "counts": counts,
        })
        previous_total = total
        previous_closed = counts["closed"]
    return days


def write_atomically(path, text):
    directory = os.path.dirname(path) or "."
    os.makedirs(directory, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix=".queue-page-", dir=directory)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            f.write(text)
        os.replace(tmp, path)
    except Exception:
        os.unlink(tmp)
        raise


# --- the page itself ---------------------------------------------------
#
# Eight categorical slots, in the fixed order `dataviz`'s palette validates for
# stacked areas (adjacent pairs only, which is all a stack needs): tier 1
# through tier 5, then parked as its own band, in both charts' legends. Keeping
# each older tier's colour and giving 2.2 and 5 the two new ones fails that
# check, red beside magenta, so every band after tier 2 has a new colour
# (§403.8). The open/closed chart reuses slot 1 for "open" and slot 6 (green)
# for "closed", since that pair is unrelated to the first chart's bands and each
# chart carries its own legend.

SLOT_LIGHT = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100", "#e87ba4", "#008300", "#4a3aa7", "#e34948"]
SLOT_DARK = ["#3987e5", "#d95926", "#199e70", "#c98500", "#d55181", "#008300", "#9085e9", "#e66767"]


def render_page(points, state, fuzz, rendered_at, links):
    by_tier_chart = stacked_area(
        points=points,
        series=[(f"Tier {t}", lambda counts, t=t: counts["by_tier"][t]) for t in TIERS_IN_ORDER]
        + [("Parked", lambda counts: counts["parked"])],
        chart_id="by-tier",
        caption="The open queue, split by tier, parked rows as their own band",
    )
    progress_chart = stacked_area(
        points=points,
        series=[
            ("Open", lambda counts: sum(counts["by_tier"].values()) + counts["parked"]),
            ("Closed", lambda counts: counts["closed"]),
        ],
        chart_id="progress",
        caption="The open queue stacked on every row closed so far",
    )
    days = by_day(points)
    added_closed_chart = grouped_bars(days, chart_id="added-closed", caption="Rows added against rows closed per day")
    return PAGE.format(
        generated_at=points[-1].when,
        commit_count=len(points),
        by_tier_chart=by_tier_chart,
        progress_chart=progress_chart,
        added_closed_chart=added_closed_chart,
        statistics_table=statistics_table(days),
        running_table=running_table(state["running"], rendered_at, links),
        pinned_table=pinned_table(state["pinned"], state["running"], state["not_taken"], links),
        not_taken_table=not_taken_table(state["not_taken"], links),
        ready_section=ready_section(state["ready"], links),
        running_section=running_section(state["running"], links),
        closed_section=closed_section(state["closed"], links),
        fuzz_section=fuzz,
        base_style=BASE_STYLE,
        series_colors=series_color_css(),
        coverage_light=SLOT_LIGHT[0],
        coverage_dark=SLOT_DARK[0],
    )


def stacked_area(points, series, chart_id, caption):
    width, height = 720, 220
    margin = {"top": 12, "right": 12, "bottom": 22, "left": 12}
    plot_w = width - margin["left"] - margin["right"]
    plot_h = height - margin["top"] - margin["bottom"]

    values = [[getter(point.counts) for point in points] for _, getter in series]
    totals = [sum(v[i] for v in values) for i in range(len(points))]
    top_total = max(totals) or 1

    def x_of(i):
        return margin["left"] if len(points) == 1 else margin["left"] + plot_w * i / (len(points) - 1)

    def y_of(v):
        return margin["top"] + plot_h * (1 - v / top_total)

    assert len(series) <= len(SLOT_LIGHT), f"{len(series)} series and {len(SLOT_LIGHT)} colours"
    bands = []
    baseline = [0] * len(points)
    colors_light = SLOT_LIGHT[: len(series)]
    colors_dark = SLOT_DARK[: len(series)]
    for series_index, (label, _) in enumerate(series):
        top = [baseline[i] + values[series_index][i] for i in range(len(points))]
        upper = " ".join(f"{x_of(i):.1f},{y_of(top[i]):.1f}" for i in range(len(points)))
        lower = " ".join(f"{x_of(i):.1f},{y_of(baseline[i]):.1f}" for i in range(len(points) - 1, -1, -1))
        bands.append(
            f'<polygon class="qp-band" data-series="{series_index}" points="{upper} {lower}">'
            f'<title>{html.escape(label)}</title></polygon>'
        )
        baseline = top

    legend = "".join(
        f'<span class="qp-legend-item"><i data-series="{i}"></i>{html.escape(label)}</span>'
        for i, (label, _) in enumerate(series)
    )
    style_vars = "".join(f"--qp-{chart_id}-{i}-light:{c};" for i, c in enumerate(colors_light))
    style_vars += "".join(f"--qp-{chart_id}-{i}-dark:{c};" for i, c in enumerate(colors_dark))
    first_label = html.escape(points[0].when[:10])
    last_label = html.escape(points[-1].when[:10])
    axis_y = height - 4
    axis_x_end = width - margin["right"]
    return f"""
<figure class="qp-chart" id="qp-{chart_id}" style="{style_vars}">
  <figcaption>{html.escape(caption)}</figcaption>
  <svg viewBox="0 0 {width} {height}" role="img" aria-label="{html.escape(caption)}">
    {"".join(bands)}
    <text class="qp-axis" x="{margin['left']}" y="{axis_y}">{first_label}</text>
    <text class="qp-axis" x="{axis_x_end}" y="{axis_y}" text-anchor="end">{last_label}</text>
  </svg>
  <div class="qp-legend">{legend}</div>
</figure>"""


def grouped_bars(deltas, chart_id, caption):
    width, height = 720, 180
    margin = {"top": 12, "right": 12, "bottom": 22, "left": 12}
    plot_w = width - margin["left"] - margin["right"]
    plot_h = height - margin["top"] - margin["bottom"]
    if not deltas:
        return f'<figure class="qp-chart" id="qp-{chart_id}"><figcaption>{html.escape(caption)}</figcaption></figure>'
    top = max((max(d["added"], d["closed"]) for d in deltas), default=1) or 1
    n = len(deltas)
    slot = plot_w / n
    bar_w = max(1.5, slot * 0.32)

    def y_of(v):
        return margin["top"] + plot_h * (1 - v / top)

    bars = []
    for i, d in enumerate(deltas):
        cx = margin["left"] + slot * (i + 0.5)
        for offset, key, series_index in ((-1, "added", 0), (1, "closed", 1)):
            v = d[key]
            x = cx + offset * bar_w * 0.55 - bar_w / 2
            y = y_of(v)
            h = margin["top"] + plot_h - y
            bars.append(
                f'<rect class="qp-bar" data-series="{series_index}" x="{x:.1f}" y="{y:.1f}" '
                f'width="{bar_w:.1f}" height="{max(h, 0):.1f}" rx="1.5">'
                f'<title>{html.escape(d["day"])}: {v} {key}</title></rect>'
            )
    legend = "".join(
        f'<span class="qp-legend-item"><i data-series="{i}"></i>{label}</span>'
        for i, label in enumerate(("Added", "Closed"))
    )
    style_vars = f"--qp-{chart_id}-0-light:{SLOT_LIGHT[0]};--qp-{chart_id}-1-light:{SLOT_LIGHT[5]};"
    style_vars += f"--qp-{chart_id}-0-dark:{SLOT_DARK[0]};--qp-{chart_id}-1-dark:{SLOT_DARK[5]};"
    return f"""
<figure class="qp-chart" id="qp-{chart_id}" style="{style_vars}">
  <figcaption>{html.escape(caption)}</figcaption>
  <svg viewBox="0 0 {width} {height}" role="img" aria-label="{html.escape(caption)}">
    {"".join(bars)}
  </svg>
  <div class="qp-legend">{legend}</div>
</figure>"""


def statistics_table(days):
    """Every number the three charts draw, one row per day, newest first.

    A day's shape is its last commit's, the same point the Pace chart takes
    its two counts from.
    """
    tier_heads = "".join(f'<th class="n">Tier {html.escape(tier)}</th>' for tier in TIERS_IN_ORDER)
    rows = []
    for day in reversed(days):
        counts = day["counts"]
        cells = [day["added"], day["closed"]]
        cells += [counts["by_tier"][tier] for tier in TIERS_IN_ORDER]
        cells += [counts["parked"], sum(counts["by_tier"].values()) + counts["parked"], counts["closed"]]
        rows.append(
            f'<tr><th>{html.escape(day["day"])}</th>' + "".join(f'<td class="n">{n}</td>' for n in cells) + "</tr>"
        )
    body = "\n".join(rows)
    return f"""
<table class="qp-stats">
<thead><tr><th rowspan="2">Date</th><th colspan="2">Pace</th>
<th colspan="{len(TIERS_IN_ORDER) + 2}">Open at the day's end</th><th rowspan="2" class="n">Closed so far</th></tr>
<tr><th class="n">Added</th><th class="n">Closed</th>{tier_heads}<th class="n">Parked</th><th class="n">All open</th></tr></thead>
<tbody>
{body}
</tbody>
</table>"""


def running_table(rows, rendered_at, links):
    """One line per held row: the tier it was taken from, the agent whose tree it runs in, and who took it.

    A slot has no number to show: `queue_slots` is a count (§397.7) and `just
    queue running` an unnumbered list. A lane's tree appears after its take,
    and the page renders only on a queue write or `ff-main`, so the table says
    when it was rendered.
    """
    when = f'<p class="qp-sub">as of {html.escape(rendered_at)}, when this page was last rendered</p>'
    if not rows:
        return when + "\n<p>nothing running</p>"
    lines = []
    for row in rows:
        tier = html.escape(row["tier"]) if row["tier"] is not None else "not recorded"
        agent = f'<code>{html.escape(row["agent"])}</code>' if row["agent"] is not None else "no tree yet"
        lines.append(
            f'<tr><th>{links.ref(row["id"])}</th><td>{tier}</td><td>{agent}</td>'
            f'<td><code>{html.escape(row["taken_by"])}</code></td></tr>'
        )
    body = "\n".join(lines)
    return f"""{when}
<table class="qp-stats">
<thead><tr><th>Row</th><th>Tier taken from</th><th>Agent</th><th>Taken by</th></tr></thead>
<tbody>
{body}
</tbody>
</table>"""


def pinned_table(rows, running, groups, links):
    """One line per open pinned row, in the order a fill takes them, and what holds it back.

    What holds a row back is every reason the table of why the fill will not
    take a row files it under, word for word, so the two tables cannot
    disagree. A row that runs is held back by nothing, whether the claims
    file holds it or main's copy says `lane:` with no take behind it, as
    `just queue` counts both.
    """
    if not rows:
        return "<p>nothing pinned</p>"
    running_on = {row["id"]: row["branch"] for row in running}
    held_back = {}
    for _, reasons in reasons_not_taken(groups, links):
        for reason, row_ids in (reasons or {}).items():
            for row_id in row_ids:
                held_back.setdefault(row_id, []).append(reason)
    lines = [
        f'<tr><th>{links.ref(row.id)}</th><td class="n">{row.pin}</td>'
        f'<td>{blocked_by(row, running_on, held_back, groups["meeting"] is None)}</td></tr>'
        for row in rows
    ]
    body = "\n".join(lines)
    return f"""<p class="qp-sub">open rows Mattias pinned (§430), by their number, which is the order a fill takes them in</p>
<table class="qp-stats">
<thead><tr><th>Row</th><th class="n">Pin</th><th>Blocked by</th></tr></thead>
<tbody>
{body}
</tbody>
</table>"""


def blocked_by(row, running_on, held_back, meeting_not_worked_out):
    if row.id in running_on:
        return f"nothing: it is running, on <code>{html.escape(running_on[row.id])}</code>"
    if row.state.startswith("lane:"):
        return f"nothing: it is running, on <code>{html.escape(row.state[len('lane:'):])}</code>"
    if row.id in held_back:
        return "; ".join(held_back[row.id])
    if meeting_not_worked_out:
        return "ready, but whether it meets a running lane's write set is not worked out"
    return "nothing: it is ready"


def not_taken_table(groups, links):
    """One group per kind of reason a fill passes an open row over, each reason beside the rows it holds back.

    A kind is headed by how many rows it holds back, each counted once
    however many of its reasons name it. The third kind's reasons are `just
    queue next`'s own lines, less the row's id, tier and, where one hold is
    met, its own path.
    """
    bodies = []
    for title, reasons in reasons_not_taken(groups, links):
        if reasons is None:
            bodies.append(
                f'<tbody>\n<tr><th colspan="2">{title} (not worked out)</th></tr>\n'
                f"<tr><td colspan=\"2\">no tree that holds main has main's own {QUEUE_PATH}, "
                f"and <code>just queue blocked</code> is run only there</td></tr>\n</tbody>")
        else:
            bodies.append(not_taken_group(title, reasons, links))
    body = "\n".join(bodies)
    return f"""<p class="qp-sub">why a fill would pass open rows over even with every slot free, and which rows each reason holds back</p>
<table class="qp-stats">
<thead><tr><th>Why</th><th>Rows</th></tr></thead>
{body}
</table>"""


def reasons_not_taken(groups, links):
    """Each kind of reason a fill passes an open row over, as its title and its reasons, each with the rows it holds back.

    A reason's rows come in the order a fill walks them, and a kind's reasons
    in the order of the first row each holds back, so the reason in front of
    the row a fill would reach first leads. A row waiting on two rows not yet
    done is under both. The reasons are None for the third kind when
    `meeting_a_running_lane` could not work it out.
    """
    waiting = {}
    for row, unfinished in groups["waiting"]:
        for other, state in unfinished:
            waiting.setdefault(f"waits on {links.ref(other)} ({links.text(state)})", []).append(row.id)
    parked = {}
    for row in groups["parked"]:
        parked.setdefault(f"parked for {links.text(row.state[len('parked:'):])}", []).append(row.id)
    meeting = None
    if groups["meeting"] is not None:
        meeting = {}
        for line in groups["meeting"]:
            meeting.setdefault(meeting_reason(line, links), []).append(line["id"])
    return [("Waits on rows not yet done", waiting), ("Parked", parked), ("Meets a running lane's write set", meeting)]


def meeting_reason(line, links):
    if line["shared"]:
        holders = " and ".join(f"{links.ref(other)} (held {how} <code>{html.escape(holder)}</code>)"
                               for other, _, how, holder in line["holders"])
        return f"<code>{html.escape(line['mine'])}</code> is shared, and {holders} already hold it"
    ((other, theirs, how, holder),) = line["holders"]
    return f"meets {links.ref(other)}'s <code>{html.escape(theirs)}</code>, held {how} <code>{html.escape(holder)}</code>"


def not_taken_group(title, reasons, links):
    held = {row_id for row_ids in reasons.values() for row_id in row_ids}
    head = f'<tr><th colspan="2">{title} ({len(held)})</th></tr>'
    lines = [f"<tr><td>{reason}</td><td>{' '.join(links.ref(row_id) for row_id in row_ids)}</td></tr>"
             for reason, row_ids in reasons.items()]
    return "<tbody>\n" + "\n".join([head, *lines]) + "\n</tbody>"


def ready_section(by_tier, links):
    items = []
    for tier in TIERS_IN_ORDER:
        rows = by_tier[tier]
        if not rows:
            continue
        items.append(f"<h3>Tier {html.escape(tier)}</h3><p>{' '.join(links.ref(row.id) for row in rows)}</p>")
    return "".join(items) if items else "<p>none ready</p>"


def running_section(rows, links):
    if not rows:
        return "<p>nothing running</p>"
    items = (
        f"<li>{links.ref(row['id'])} on <code>{html.escape(row['branch'])}</code> "
        f"at <code>{html.escape(row['where'])}</code>{narrowed_html(row)}</li>"
        for row in rows
    )
    return "<ul>" + "".join(items) + "</ul>"


def narrowed_html(row):
    """What a lane narrowed its hold to (§424.7), since the row's text still names the set it was taken with."""
    if row["write_set"] == row["taken"]:
        return ""
    return f" — narrowed to <code>{html.escape(row['write_set'])}</code>"


def closed_section(rows, links):
    if not rows:
        return "<p>none closed yet</p>"
    return "<p>" + " ".join(links.ref(row.id) for row in reversed(rows)) + "</p>"


class Links:
    """How a page names a row: linked to the row's own page when main's copy holds the row, and so it has one."""

    def __init__(self, tip, to_rows):
        self.ids = set(tip.ids())
        self.to_rows = to_rows

    def ref(self, row_id):
        """A row named by its id, of any letter, as code."""
        return f"<code>{self.link(row_id)}</code>"

    def text(self, text):
        """`text` escaped, with every Q id in it linked.

        Only Q ids: a T or D id in prose is as often something else, as in
        "arm D2", and those rows are named where the grammar puts an id.
        """
        return ROW_REF.sub(lambda match: self.link(match.group(0)), html.escape(text))

    def link(self, row_id):
        if row_id not in self.ids:
            return html.escape(row_id)
        return f'<a href="{self.to_rows}{row_id}.html">{row_id}</a>'


def write_row_pages(directory, tip, graph):
    """One page for every row of main's copy (§444), and none left for a row it no longer holds."""
    links = Links(tip, "")
    os.makedirs(directory, exist_ok=True)
    for row in tip.rows:
        assert ANY_ROW_ID.match(row.id), f"{row.id!r} cannot name a page, and check-queue refuses an id of that shape"
        write_if_changed(os.path.join(directory, f"{row.id}.html"), row_page(row, graph, links))
    for name in os.listdir(directory):
        if name.endswith(".html") and name[: -len(".html")] not in links.ids:
            os.remove(os.path.join(directory, name))


def write_if_changed(path, text):
    """Write `text` unless `path` holds it already.

    A queue write changes a row or two. For the 1827 rows main held at
    c8a9eda5, writing every page took 0.30 s of a 4.0 s render, and reading
    each back to compare takes 0.08 s.
    """
    try:
        with open(path, encoding="utf-8") as f:
            if f.read() == text:
                return
    except FileNotFoundError:
        pass
    write_atomically(path, text)


def row_page(row, graph, links):
    return ROW_PAGE.format(
        base_style=BASE_STYLE,
        id=html.escape(row.id),
        tier=html.escape(row.tier),
        row_class=class_html(row.tag),
        provenance=provenance_html(row.id, graph, links),
        write_set=write_set_html(row.write_set),
        text=links.text(row.text),
    )


# §352.3's two halves of a class, cut to a line each.
HOW_BAD = {
    "wrong": "a wrong answer",
    "crash": "a panic, an abort, an overflow or a hang",
    "refuses": "a named refusal of an input that has an answer",
}
HOW_LIKELY = {
    "A": "seen: a real input met it",
    "B": "typical: a shape designers make on purpose",
    "C": "possible: only a probe builds it",
    "D": "never: outside the scale envelope of §3.5",
}


def class_html(tag):
    if tag is None:
        return "none"
    if tag == SMALL:
        return "none: it is tagged <code>small</code>, and goes out in a bundle (§352.4)"
    bad, likely = tag.split("/")
    return f"<b>{likely}</b>, {HOW_LIKELY[likely]}. Its tag is <code>{tag}</code>: {HOW_BAD[bad]} (§352.3)"


def provenance_html(row_id, graph, links):
    """The row, the row it came from, and so on up to what nothing led to, as `just lineage main <id>` reads each (§432, §433).

    A step whose source is read from the row's text (§433.6) says what it
    would be without that. A chain that comes back to a row it already passed
    stops there and says so.
    """
    steps = []
    passed = [row_id]
    node = row_id
    while True:
        info = graph.with_text.parent_of.get(node)
        steps.append(step_html(node, info, graph, links))
        parent = info.parent if info is not None else None
        if parent is None or parent.startswith(("fuzz:", "step:")):
            break
        if parent in passed:
            steps.append(f"{links.ref(parent)} again, so the chain goes round, and stops here")
            break
        passed.append(parent)
        node = parent
    return "".join(f"<li>{step}</li>" for step in steps)


def step_html(node, info, graph, links):
    if info is None:
        if node.startswith("Q"):
            return f"{links.ref(node)}: no commit on main's history added it"
        return f"{links.ref(node)}: carried over from the retired <code>docs/test-plan.md</code>, and lineage follows only Q rows"
    commit = graph.added_at[node]
    day = commit.date.astimezone(timezone.utc).strftime("%Y-%m-%d")
    came = came_from(info, links)
    if node in graph.read_from_text:
        came += f"; without reading its text, {came_from(graph.parent_of[node], links)}"
    return f"{links.ref(node)}, added in <code>{commit.sha[:8]}</code> on {day}, <q>{links.text(commit.subject)}</q>: {came}"


FOUND_BY = {
    lineage.WORD_LABEL: "as its own word says",
    lineage.COMMIT_LABEL: "read from the commit that added it",
    lineage.MESSAGE_LABEL: "read from that commit's message",
    lineage.TEXT_LABEL: "which its text names (§433.6)",
}


def came_from(info, links):
    if info.label == lineage.UNKNOWN_LABEL:
        return ("not known: the commit that added it closed more than one row, and neither their text "
                "nor its message says which one led to it")
    if info.relation == lineage.FIRST:
        if info.label == lineage.WORD_LABEL:
            return "nothing led to it, as its word <code>from/-</code> says"
        return "nothing led to it: the commit that added it closed no row"
    verb = "a re-file of" if info.relation == lineage.RE_FILE else "came from"
    return f"{verb} {source_html(info.parent, links)}, {FOUND_BY[info.label]}"


def source_html(parent, links):
    if parent.startswith("fuzz:"):
        return f"the fuzz target <code>{html.escape(parent[len('fuzz:'):])}</code>"
    if parent.startswith("step:"):
        return f"the STEP file <code>{html.escape(parent[len('step:'):])}</code>"
    return links.ref(parent)


def write_set_html(write_set):
    """The files a lane on the row expects to change: its write set (§230.2)."""
    if write_set == "-":
        return "no tracked file, only a new section of its own"
    return "<ul>" + "".join(f"<li><code>{html.escape(path)}</code></li>" for path in write_set.split(",")) + "</ul>"


ROW_PAGE = """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{id} · rcad queue</title>
<style>
{base_style}
dl.qp-row {{ display: grid; grid-template-columns: max-content 1fr; gap: 8px 16px; max-width: 1100px; margin: 0; }}
dl.qp-row dt {{ color: var(--qp-ink-2); font-weight: 600; }}
dl.qp-row dd {{ margin: 0; }}
dl.qp-row ol {{ margin: 0; padding-left: 20px; }}
pre.qp-row-text {{
  white-space: pre-wrap;
  font-size: 12px;
  background: var(--qp-surface);
  border: 1px solid var(--qp-border);
  border-radius: 8px;
  padding: 8px 12px;
  max-width: 1100px;
}}
</style>
</head>
<body>
<p class="qp-sub"><a href="../index.html">rcad queue</a></p>
<h1>{id}</h1>
<dl class="qp-row">
<dt>Tier</dt><dd>{tier}</dd>
<dt>Class</dt><dd>{row_class}</dd>
<dt>Came from</dt><dd><ol>{provenance}</ol></dd>
<dt>Expects to change</dt><dd>{write_set}</dd>
</dl>
<h2>The row</h2>
<pre class="qp-row-text">{text}</pre>
</body>
</html>
"""

# The palette, the type and the plain elements both kinds of page share.
BASE_STYLE = """:root {
  color-scheme: light;
  --qp-surface: #fcfcfb;
  --qp-page: #f9f9f7;
  --qp-ink: #0b0b0b;
  --qp-ink-2: #52514e;
  --qp-muted: #898781;
  --qp-grid: #e1e0d9;
  --qp-border: rgba(11,11,11,0.10);
  --qp-link: #2a78d6;
}
@media (prefers-color-scheme: dark) {
  :root {
    color-scheme: dark;
    --qp-surface: #1a1a19;
    --qp-page: #0d0d0d;
    --qp-ink: #ffffff;
    --qp-ink-2: #c3c2b7;
    --qp-muted: #898781;
    --qp-grid: #2c2c2a;
    --qp-border: rgba(255,255,255,0.10);
    --qp-link: #3987e5;
  }
}
* { box-sizing: border-box; }
body {
  margin: 0;
  padding: 16px;
  background: var(--qp-page);
  color: var(--qp-ink);
  font: 14px/1.45 system-ui, -apple-system, "Segoe UI", sans-serif;
}
h1 { font-size: 18px; margin: 0 0 4px; }
h2 { font-size: 15px; margin: 28px 0 8px; }
h2:first-child { margin-top: 0; }
h3 { font-size: 13px; margin: 12px 0 4px; color: var(--qp-ink-2); }
p.qp-sub { color: var(--qp-muted); margin: 0 0 20px; }
ul { margin: 0 0 8px; padding-left: 20px; }
li { margin: 2px 0; }
code { font-size: 12px; }
a { color: var(--qp-link); }"""


PAGE = """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>rcad queue</title>
<style>
{base_style}
.qp-chart {{
  background: var(--qp-surface);
  border: 1px solid var(--qp-border);
  border-radius: 8px;
  padding: 12px;
  margin: 0 0 16px;
  max-width: 760px;
}}
.qp-chart figcaption {{ color: var(--qp-ink-2); margin-bottom: 6px; }}
.qp-chart svg {{ width: 100%; height: auto; display: block; }}
.qp-axis {{ font-size: 10px; fill: var(--qp-muted); }}
.qp-band {{ stroke: var(--qp-surface); stroke-width: 2; }}
.qp-bar {{ stroke: var(--qp-surface); stroke-width: 1; }}
.qp-legend {{ display: flex; flex-wrap: wrap; gap: 12px; margin-top: 8px; font-size: 12px; color: var(--qp-ink-2); }}
.qp-legend-item i {{ display: inline-block; width: 10px; height: 10px; border-radius: 2px; margin-right: 4px; vertical-align: -1px; }}
{series_colors}
.qp-grid2 {{ display: grid; grid-template-columns: 1fr 1fr; gap: 24px; max-width: 1200px; }}
@media (max-width: 700px) {{ .qp-grid2 {{ grid-template-columns: 1fr; }} }}
.qp-chart.qp-wide {{ max-width: 1200px; }}
.qp-multiples {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 12px 20px; }}
.qp-panel-title {{ font-size: 12px; color: var(--qp-ink-2); }}
.qp-cov-line {{ fill: none; stroke: {coverage_light}; stroke-width: 2; }}
.qp-cov-dot {{ fill: {coverage_light}; stroke: var(--qp-surface); stroke-width: 2; }}
@media (prefers-color-scheme: dark) {{
  .qp-cov-line {{ stroke: {coverage_dark}; }}
  .qp-cov-dot {{ fill: {coverage_dark}; }}
}}
pre.qp-latest {{
  white-space: pre-wrap;
  font-size: 12px;
  background: var(--qp-surface);
  border: 1px solid var(--qp-border);
  border-radius: 8px;
  padding: 8px 12px;
  max-width: 1200px;
}}
table.qp-fuzz {{ border-collapse: collapse; font-size: 12px; margin: 0 0 8px; }}
table.qp-fuzz th, table.qp-fuzz td {{ padding: 3px 8px; border-bottom: 1px solid var(--qp-grid); text-align: left; vertical-align: top; }}
table.qp-fuzz thead th {{ color: var(--qp-ink-2); font-weight: 600; }}
table.qp-fuzz tbody th {{ font-weight: 600; white-space: nowrap; }}
table.qp-fuzz tbody th .qp-note {{ width: 11em; white-space: normal; }}
table.qp-fuzz td.n {{ text-align: right; font-variant-numeric: tabular-nums; }}
table.qp-fuzz td.faults {{ white-space: nowrap; }}
.qp-note {{ color: var(--qp-muted); font-size: 12px; font-weight: normal; max-width: 900px; }}
table.qp-stats {{ border-collapse: collapse; font-size: 12px; margin: 0 0 8px; }}
table.qp-stats th, table.qp-stats td {{ padding: 3px 8px; border-bottom: 1px solid var(--qp-grid); text-align: left; }}
table.qp-stats thead th {{ color: var(--qp-ink-2); font-weight: 600; vertical-align: bottom; }}
table.qp-stats thead th[colspan] {{ text-align: center; }}
table.qp-stats tbody th {{ font-weight: 600; white-space: nowrap; }}
table.qp-stats .n {{ text-align: right; font-variant-numeric: tabular-nums; }}
</style>
</head>
<body>
<h2>Pace</h2>
{added_closed_chart}

<h1>rcad queue</h1>
<p class="qp-sub">{commit_count} commits of docs/queue.md on main's first-parent line, through {generated_at}</p>

{by_tier_chart}
{progress_chart}

<h2>Queue statistics</h2>
{statistics_table}

<h2>Running rows</h2>
{running_table}

<h2>Pinned</h2>
{pinned_table}

<h2>Rows the fill will not take now</h2>
{not_taken_table}

<div class="qp-grid2">
<div>
<h2>Ready</h2>
{ready_section}
</div>
<div>
<h2>Running</h2>
{running_section}
</div>
</div>

<h2>Lately closed</h2>
{closed_section}

<h2>Fuzzing, night by night</h2>
{fuzz_section}
</body>
</html>
"""


def series_color_css():
    """One CSS rule per chart × slot, generated once rather than hand-written per chart id."""
    rules = []
    for chart_id, count in (("by-tier", len(TIERS_IN_ORDER) + 1), ("progress", 2), ("added-closed", 2)):
        for i in range(count):
            rules.append(
                f'#qp-{chart_id} [data-series="{i}"] {{ fill: var(--qp-{chart_id}-{i}-light); }}\n'
                f'#qp-{chart_id} .qp-legend-item i[data-series="{i}"] {{ background: var(--qp-{chart_id}-{i}-light); }}\n'
                f'@media (prefers-color-scheme: dark) {{\n'
                f'  #qp-{chart_id} [data-series="{i}"] {{ fill: var(--qp-{chart_id}-{i}-dark); }}\n'
                f'  #qp-{chart_id} .qp-legend-item i[data-series="{i}"] {{ background: var(--qp-{chart_id}-{i}-dark); }}\n'
                f'}}'
            )
    return "\n".join(rules)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
