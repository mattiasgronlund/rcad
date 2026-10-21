## 461. Why rows take more than one lane — 2026-10-21

*Q2597, read-only. Re-measures the probe Q2597's own text quotes (taken on
2026-10-21), and, for every row in the two weeks 2026-10-07 to 2026-10-21
(main at `105c64529093773d541154c37d7661ef5ff2f9d0`), files each extra lane
agent under why it started. `tooling/row-time/row_time.py`'s label prefixes
miss `prep:`, `close:` and `relay N:` on this base (Q2595 is fixing that
beside this row, in `tooling/row-time/` only), so a relay, a prep or a close
agent lands in its "unattributed" bucket today. This section reads the run
records, the agents' transcripts and the journal results directly instead,
with a loader of its own — §461.5's script imports `row_time.read_transcript`
for joining a split transcript and `spend.cost_of`/`tier_of` for pricing, and
nothing else from either tool.*

  461.1   The re-measurement, against the probe's numbers
  461.2   Why each extra lane agent started
  461.3   Split by crate and by model
  461.4   Relays: how far the lane before had got
  461.5   What ran this
  461.6   What this leaves

### 461.1 The re-measurement, against the probe's numbers

The probe's two claims, re-measured on this base, over the same two weeks
(`mise exec -- python3 target/row_causes.py 2026-10-07 2026-10-21`, §461.5):

    rows picked: 880
    rows with 2+ lane agents: 485
    usd held by 2+ lane-agent rows: 4825.93 of 6221.18 total
    median row cost with 1 lane agent(s): 2.85 usd (n=395)
    median row cost with 2 lane agent(s): 5.97 usd (n=236)
    median row cost with 3 lane agent(s): 10.06 usd (n=98)
    median row cost with 4 lane agent(s): 13.98 usd (n=86)
    median row cost with 5+ lane agent(s): 20.18 usd (n=65)

Against the probe's 930 rows, 507 with 2+ lane agents, 5172 of 6712 USD, and
medians 3.35/6.79/10.31/14.63/21.46: fewer rows (880 against 930, since this
run reads the record set as it stands now, fourteen hours later, not the
probe's snapshot), but the **share** held by multi-lane-agent rows is close —
55.1% of rows here against 54.5%, 77.6% of the money here against 77.0% — and
the medians step up by about the cost of one more lane agent each time, same
as the probe found. The probe's claim stands: each extra lane costs about what
a first one does.

**The hours, re-measured over 10-14 to 10-21** (`mise exec -- python3
target/row_causes.py 2026-10-14 2026-10-21`):

    first-lane agent hours: 124.3h; relay agent hours: 154.7h

Against the probe's 119h of first lanes and about 141h of relay agents: both
numbers moved up together, relay hours still ahead of first-lane hours by
about the same margin (1.24x here, 1.19x there). **Relay agents, as a class,
spend more wall-clock time than the first lane of every row put together.**

### 461.2 Why each extra lane agent started

1023 lane-role agents beyond a row's first ran in the window (that is: every
`lane`-role `workflowProgress` entry whose row already had one earlier,
whether a `relay N:`-labelled entry in the same run or the first lane agent of
a later run). Each is filed under the state of the row right before it
started, read from the run record (`status`, `workflowProgress`) and the
journal result of the agent before it (`verdict` for a verifier, `merged` and
`not_merged_because` for a lander or merger, `outcome` for a lane):

    reason                                              count   hours      usd
    relay: at the context budget                          878  279.0h  2997.14
    re-take: handed back after the relay limit              52   16.7h   177.93
      (§416, MAX_RELAYS -- a run's own relay chain hit 3 and
      the row went back to the queue with no verifier ever run)
    other: the lane itself reported blocked                 29    9.3h    86.63
    re-take: failed or killed run                           21    8.7h    77.99
    re-take: the merge fell to the Opus merger               13    3.8h    28.63
      (merge:<label> ran after land: stopped, and still
      did not merge it)
    other: verified, but the run ended before land            17    2.4h    15.95
      or merge ran (verdict merge/fix_first, no land: or
      merge: agent in the run at all)
    re-send: the verifier refused it                          3    1.3h    17.00
    other, ten more cases, each under an hour                10    2.0h    12.03
      (partial or fixed outcomes with no verifier result
      the journal carries)

**Relays are the whole story for the time and the money.** 878 of 1023 extra
agents (86%), 279.0h of the 323.2h these agents ran, 2997.14 of the 3421.30
USD — a lane asking for one more lane because it hit its own context budget
(§416) is both the overwhelming majority of extra agents and, since a relay's
own hours and dollars are close to a first lane's (§461.1), the overwhelming
majority of what extra lane agents cost.

The second-largest count, 52 cases, is a kind of relay too, just one that did
not resolve: a run's own relay chain reached `MAX_RELAYS` (3) with the row
still not done, `lanes.js` handed it back with no verifier ever run (§416,
`RELAY`'s refusal path), and a **later, separate run** took the row again
from scratch — its first lane agent is the "extra" one filed here. `Q467` is
one: `wf_7eb41607-b1c` and `wf_dd37e099-bff` each ran a first lane and three
relays and were handed back with no verifier; `wf_99ea712d-5b6`'s first lane
and two relays then verified `merge` and landed `Q467 done:200367e8`. Two runs
handed back, one that landed: three lane-chain attempts for one row, each
chain itself relaying to its own limit.

**"The lane itself reported blocked"** (29 cases) is a lane's own `outcome`
field, read verbatim from its journal result — most often the base moved or a
branch already existed (the kind of thing `lane.md`'s own text says to stop
and report rather than paper over).

**"The merge fell to the Opus merger"** (13 cases) is `merge:<label>` running
after `land:<label>` stopped (`byMerger` in `lanes.js`), and the Opus merger
*also* not merging it — `Q2395`'s `wf_cee143b8-15b` is one, `stop: "gate"`,
`not_merged_because: "gate: FAIL first failure: check-land-selftest (exit 1)"`.
When the Opus merger does land it, no new lane agent follows, so this reason
only ever shows up here paired with a merge that failed twice over.

**"Verified, but the run ended before land or merge ran"** (17 cases) is the
one surprising reading: the verifier said `merge` or `fix_first`, and the run
record's `workflowProgress` holds no `land:` or `merge:` agent at all for that
lane, though the run's own `status` is `completed`. `Q368`'s `wf_b391c4d3-0db`
is one: the run's own top-level `result` says the row was "verified, not
merged" with no lander or merger ever dispatched for it. This is a `lanes.js`
behaviour worth tracing on its own — filed below (§461.6) rather than chased
further here, since this row is read-only.

**"Re-take: failed or killed run"** (21 cases) is a run whose `status` is
`failed` or `killed`, or whose lane agent's own journal holds no result at
all — it never got the chance to report anything, let alone reach a verifier.

**"Re-send: the verifier refused it"** (3 cases) is rarer than every other
reason: a verifier's journal result with `verdict: "reject"`, and a wholly new
run dispatched afterwards. `check-decisions` and the lane rules elsewhere in
this repo bias hard toward a lane getting it right the first time a verifier
sees it; this count is the measure of how rarely that bias fails.

### 461.3 Split by crate and by model

By whether the row's write set (`args.lanes[].writeSet` in the run record)
touches anything under `crates/`:

    reason                                          crate=False  crate=True
    relay: at the context budget                            635         243
    re-take: handed back after the relay limit                39          13
    other: the lane itself reported blocked                   16          13
    re-take: failed or killed run                              6          15
    re-take: the merge fell to the Opus merger                 2          11
    other: verified, before land/merge ran                    14           3
    re-send: the verifier refused it                           1           2

Relays split roughly in the ratio the rows themselves do (crate rows are a
minority of all rows, and relays follow that), but **"failed or killed"** and
**"fell to the Opus merger"** both skew hard toward rows that write a crate —
15 of 21 killed/failed re-takes, 11 of 13 Opus-merger re-takes. Both are the
two reasons that cost the most per occurrence (0.4h and 0.22h average) and
both are concentrated in the kind of row most likely to touch a file several
lanes are also working in at once, where `just land` is more likely to refuse
a conflicting branch or a gate the lane already gated to drift under it.

By model (the dispatch model named in `args.lanes[].model` for the row's own
attempt; opus and sonnet are the only two seen):

    reason                                          model=opus  model=sonnet
    relay: at the context budget                           837            41
    re-take: handed back after the relay limit               43             9
    other: the lane itself reported blocked                  21             8
    re-take: failed or killed run                             12             9
    re-take: the merge fell to the Opus merger                 5             8
    other: verified, before land/merge ran                     1            16
    re-send: the verifier refused it                           3             0

Relays are almost entirely an Opus-lane phenomenon (837 of 878): §416's own
measurement was Opus lanes specifically, and nothing here says a Sonnet lane
relays at any real rate. The one reversed row, "verified, before land/merge
ran", is 16 of 17 Sonnet — all but one of those 17 cases are a *verifier*
agent (always Sonnet, `VERIFIER()` in `lanes.js`) reporting `merge`/`fix_first`
with nothing after it in the same run; the model named for the row's own lane
in those cases is mixed, but the model actually left holding the open question
of why the run stopped is the Sonnet verifier's.

### 461.4 Relays: how far the lane before had got

For every one of the 878 relay cases, the lane agent right before the relay
(the one that asked for it) is read the same way §416 read five examples by
hand — its peak context across its own responses, the context at its first
`Edit`/`Write`/`MultiEdit` tool call, and how many `git commit` `Bash` calls it
made before relaying:

    relays: lane-before's context at relay, median 158945 tokens (n=878)
    relays: lane-before's tokens at first edit, median 97727 (n=764 of 878,
      rest never edited before relaying)
    relays: lane-before's git-commit calls, median 1, zero in 426 of 878

The median relaying lane is well past the 150k budget (158945), consistent
with the budget hook firing and the lane obeying `lane.md`'s "commit, then
relay" rule. 764 of 878 (87%) had made at least one edit before relaying — the
other 114 relayed with nothing in `src/` touched yet, most likely still
reading or still investigating. **426 of 878 (49%) relayed having made zero
commits.** That is a direct count against `lane.md`'s "commit the failing
test first, on its own, then the fix" and against §416.1's own finding ("a
lane stopped at 150k had nothing committed") — half of all relays still hand
the next lane an empty branch to start from, which is exactly the costly case
§416.2 measured (R of 60k to 105k, against 14k to 26k after a first lane that
had committed). §416's own fix (commit before relaying) is landed; this count
says it is not followed every time, not that it is missing.

### 461.5 What ran this

`target/row_causes.py` (gitignored, rebuilt from this block) and the two
commands above. It imports `row_time.read_transcript` (joining a transcript
split across session or project folders, §411.8) and `spend.cost_of`/`tier_of`
(pricing, §note at the head of `spend.PRICES`) and otherwise loads the run
records, the agents' transcripts and their journal results itself, since
`row_time`'s own role-by-label-prefix logic does not know about `prep:`,
`close:` or `relay N:` (confirmed by reading `PREFIXES` in
`tooling/row-time/row_time.py` and the `label:` lines in
`.claude/workflows/lanes.js`).

```python
#!/usr/bin/env python3
"""Why a queue row's extra lane agents started -- docs/decisions.d/461.

Reads the `lanes` run records, the agents' transcripts and their journal
results directly (not through `tooling/row-time/row_time.py`, whose label
prefixes miss `prep:`, `close:` and `relay N:` -- those agents land in its
"unattributed" bucket today). Prices with `cost_of`/`tier_of` from
`tooling/session-spend/spend.py`, imported rather than copied.
"""
import sys
import os
import glob
import json
import re
import collections
import statistics

HERE = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.dirname(HERE)
sys.path.insert(0, os.path.join(REPO_ROOT, "tooling", "row-time"))
sys.path.insert(0, os.path.join(REPO_ROOT, "tooling", "session-spend"))
import row_time as RT  # noqa: E402
import spend as SP  # noqa: E402

ROOT = os.path.expanduser("~/.claude/projects")
REPO = "-home-mattias-git-mattiasgronlund-rcad"

PREFIXES = [("prep:", "prep"), ("verify:", "verifier"), ("land:", "lander"),
            ("merge:", "merger"), ("close:", "closer"), ("prune:", "pruner")]
RELAY_RE = re.compile(r"^relay (\d+):(.*)$")
RETRY_RE = re.compile(r" \(retry \d+\)$")


def role_base(label):
    """(role, base lane label, relay number -- 0 for the first lane of its run)."""
    label = RETRY_RE.sub("", label)
    m = RELAY_RE.match(label)
    if m:
        return "lane", m.group(2), int(m.group(1))
    for prefix, role in PREFIXES:
        if label.startswith(prefix):
            return role, label[len(prefix):], 0
    return "lane", label, 0


def folders():
    return [os.path.join(ROOT, REPO)] + sorted(glob.glob(os.path.join(ROOT, REPO + "-worktrees-*")))


def load_runs():
    runs = []
    for folder in folders():
        for path in sorted(glob.glob(os.path.join(folder, "*", "workflows", "*.json"))):
            if os.path.getsize(path) == 0:
                continue
            with open(path, encoding="utf-8") as f:
                try:
                    rec = json.load(f)
                except ValueError:
                    continue
            if rec.get("workflowName") == "lanes":
                runs.append(rec)
    return runs


def load_transcript_index():
    idx = collections.defaultdict(list)
    for folder in folders():
        for path in glob.glob(os.path.join(folder, "*", "subagents", "workflows", "*", "agent-*.jsonl")):
            run_id = os.path.basename(os.path.dirname(path))
            agent_id = os.path.basename(path)[len("agent-"):-len(".jsonl")]
            idx[(run_id, agent_id)].append(path)
    return idx


def load_journal_index():
    idx = collections.defaultdict(list)
    for folder in folders():
        for path in glob.glob(os.path.join(folder, "*", "subagents", "workflows", "*", "journal.jsonl")):
            run_id = os.path.basename(os.path.dirname(path))
            idx[run_id].append(path)
    return idx


def journal_results(run_id, jidx):
    out = {}
    for path in jidx.get(run_id, []):
        with open(path, encoding="utf-8", errors="replace") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    rec = json.loads(line)
                except ValueError:
                    continue
                if rec.get("type") == "result" and rec.get("agentId"):
                    out[rec["agentId"]] = rec.get("result")
    return out


def agent_cost_hours(run_id, agent_id, tidx):
    """(usd, peak_context, first_edit_tokens, commits) for one agent.

    usd sums `cost_of` over one response per (requestId, message.id), the
    record with the largest output_tokens (spend.py's own rule, §396.1).
    peak_context is the largest input+cache_read+cache_write seen. commits
    counts Bash calls whose command contains `git commit`.
    """
    paths = tidx.get((run_id, agent_id), [])
    records, _ = RT.read_transcript(paths)
    grouped, order = {}, []
    for rec in records:
        if rec.get("type") != "assistant":
            continue
        message = rec.get("message") or {}
        key = (rec.get("requestId"), message.get("id"))
        if key not in grouped:
            grouped[key] = rec
            order.append(key)
        else:
            kept = (grouped[key].get("message") or {}).get("usage") or {}
            seen = message.get("usage") or {}
            if seen.get("output_tokens", 0) > kept.get("output_tokens", 0):
                grouped[key] = rec
    usd = 0.0
    peak_context = 0
    first_edit_tokens = None
    tokens_so_far = 0
    commits = 0
    for key in order:
        rec = grouped[key]
        message = rec.get("message") or {}
        usage = message.get("usage") or {}
        model = message.get("model") or ""
        usd += SP.cost_of(model, usage)
        created = usage.get("cache_creation") or {}
        hour = created.get("ephemeral_1h_input_tokens", 0)
        five = created.get("ephemeral_5m_input_tokens", 0) or usage.get("cache_creation_input_tokens", 0)
        context = usage.get("input_tokens", 0) + usage.get("cache_read_input_tokens", 0) + hour + five
        peak_context = max(peak_context, context)
        tokens_so_far = context
        for block in message.get("content") or []:
            if isinstance(block, dict) and block.get("type") == "tool_use":
                if block.get("name") in ("Edit", "Write", "MultiEdit") and first_edit_tokens is None:
                    first_edit_tokens = tokens_so_far
                if block.get("name") == "Bash":
                    command = (block.get("input") or {}).get("command") or ""
                    if "git commit" in command:
                        commits += 1
    return usd, peak_context, first_edit_tokens, commits


class Attempt:
    def __init__(self, run_id, run_status, base_label, lanes_entry):
        self.run_id = run_id
        self.run_status = run_status
        self.base_label = base_label
        self.write_set = lanes_entry.get("writeSet") or []
        self.model = lanes_entry.get("model") or "opus"
        self.lane_chain = []   # [(relay_n, agent_id, label, start, end)]
        self.verifier = None   # agent_id
        self.lander = None
        self.merger = None
        self.closer = None
        self.anchor = None


def build_attempts(runs):
    """row id -> [Attempt, ...] in time order."""
    row_attempts = collections.defaultdict(list)
    for rec in runs:
        run_id = rec["runId"]
        start = rec["startTime"]
        run_end = start + rec["durationMs"]
        status = rec.get("status")
        lanes_meta = {}
        for lane in (rec.get("args") or {}).get("lanes") or []:
            lanes_meta[lane["label"]] = lane
        attempts_by_base = {}
        for entry in rec.get("workflowProgress") or []:
            if entry.get("type") != "workflow_agent":
                continue
            label = entry["label"]
            role, base, relay_n = role_base(label)
            lanes_entry = lanes_meta.get(base)
            if lanes_entry is None or not lanes_entry.get("queueId"):
                continue
            started = entry.get("startedAt")
            if started is None:
                continue
            duration = entry.get("durationMs")
            end = started + duration if duration is not None else run_end
            attempt = attempts_by_base.get(base)
            if attempt is None:
                attempt = Attempt(run_id, status, base, lanes_entry)
                attempts_by_base[base] = attempt
            if role == "lane":
                attempt.lane_chain.append((relay_n, entry["agentId"], label, started, end))
            elif role == "verifier" and attempt.verifier is None:
                attempt.verifier = entry["agentId"]
            elif role == "lander" and attempt.lander is None:
                attempt.lander = entry["agentId"]
            elif role == "merger" and attempt.merger is None:
                attempt.merger = entry["agentId"]
            elif role == "closer" and attempt.closer is None:
                attempt.closer = entry["agentId"]
        for base, attempt in attempts_by_base.items():
            if not attempt.lane_chain:
                continue
            attempt.lane_chain.sort(key=lambda t: t[0])
            attempt.anchor = min(t[3] for t in attempt.lane_chain)
            rows = [r for r in (lanes_meta[base].get("queueId") or "").split(",") if r]
            for row in rows:
                row_attempts[row].append(attempt)
    for row in row_attempts:
        row_attempts[row].sort(key=lambda a: a.anchor)
    return row_attempts


def classify_extra(prev, jidx):
    """Why the first lane agent of a new attempt started, given the attempt before it."""
    results = journal_results(prev.run_id, jidx)
    _, last_agent, _, _, _ = prev.lane_chain[-1]
    lane_result = results.get(last_agent)
    if prev.run_status in ("failed", "killed") or lane_result is None:
        return "re-take: failed or killed run", None
    verdict = (results.get(prev.verifier) or {}).get("verdict") if prev.verifier else None
    if verdict == "reject":
        return "re-send: the verifier refused it", verdict
    if verdict in ("merge", "fix_first") and prev.merger is not None:
        merger_result = results.get(prev.merger) or {}
        if merger_result.get("merged") is not True:
            return "re-take: the merge fell to the Opus merger", merger_result.get("not_merged_because")
    outcome = lane_result.get("outcome") if isinstance(lane_result, dict) else None
    if outcome == "blocked":
        return "other: the lane itself reported blocked", None
    if outcome == "relay" and prev.verifier is None:
        return "re-take: handed back after the relay limit (§416, MAX_RELAYS)", None
    if verdict in ("merge", "fix_first") and prev.lander is None and prev.merger is None:
        return "other: verified, but the run ended before land or merge ran", None
    return "other: %s, verifier %s" % (outcome or "no outcome", verdict or "none"), None


def main():
    since = RT.ms_of_date(sys.argv[1]) if len(sys.argv) > 1 else RT.ms_of_date("2026-10-07")
    until = RT.ms_of_date(sys.argv[2]) if len(sys.argv) > 2 else RT.ms_of_date("2026-10-21")
    runs = load_runs()
    tidx = load_transcript_index()
    jidx = load_journal_index()
    row_attempts = build_attempts(runs)

    picked = {row: attempts for row, attempts in row_attempts.items()
              if since <= attempts[0].anchor < until}

    total_rows = len(picked)
    multi_rows = 0
    row_cost = {}
    row_lane_count = {}
    reason_count = collections.Counter()
    reason_hours = collections.Counter()
    reason_usd = collections.Counter()
    reason_by_crate = collections.Counter()
    reason_by_model = collections.Counter()
    relay_context = []
    relay_first_edit = []
    relay_commits = []
    other_examples = []
    first_lane_hours = [0.0]

    for row, attempts in picked.items():
        all_lane = []  # (attempt, relay_n, agent_id, label, start, end)
        for attempt in attempts:
            for relay_n, agent_id, label, start, end in attempt.lane_chain:
                all_lane.append((attempt, relay_n, agent_id, label, start, end))
        if not all_lane:
            continue
        row_lane_count[row] = len(all_lane)
        if len(all_lane) > 1:
            multi_rows += 1
        writes_crate = any(p.startswith("crates/") for p in attempts[0].write_set)
        usd_total = 0.0
        for i, (attempt, relay_n, agent_id, label, start, end) in enumerate(all_lane):
            usd, peak_ctx, first_edit, commits = agent_cost_hours(attempt.run_id, agent_id, tidx)
            usd_total += usd
            hours = (end - start) / 3_600_000
            if i == 0:
                first_lane_hours[0] += hours
                continue
            model = attempt.model
            if relay_n >= 1:
                reason = "relay: at the context budget"
                prev_agent_id = all_lane[i - 1][2]
                prev_usd, prev_ctx, prev_edit, prev_commits = agent_cost_hours(attempt.run_id, prev_agent_id, tidx)
                relay_context.append(prev_ctx)
                if prev_edit is not None:
                    relay_first_edit.append(prev_edit)
                relay_commits.append(prev_commits)
            else:
                prev_attempt = all_lane[i - 1][0]
                reason, detail = classify_extra(prev_attempt, jidx)
                if reason.startswith("other") and len(other_examples) < 12:
                    other_examples.append((row, attempt.run_id, reason, detail))
            reason_count[reason] += 1
            reason_hours[reason] += hours
            reason_usd[reason] += usd
            reason_by_crate[(reason, writes_crate)] += 1
            reason_by_model[(reason, model)] += 1
        row_cost[row] = usd_total

    print("window: %s to %s" % (sys.argv[1] if len(sys.argv) > 1 else "2026-10-07",
                                 sys.argv[2] if len(sys.argv) > 2 else "2026-10-21"))
    print("rows picked: %d" % total_rows)
    print("rows with 2+ lane agents: %d" % multi_rows)
    multi_cost = sum(row_cost[r] for r in picked if row_lane_count.get(r, 0) > 1)
    total_cost = sum(row_cost.values())
    print("usd held by 2+ lane-agent rows: %.2f of %.2f total" % (multi_cost, total_cost))
    print("first-lane agent hours: %.1fh; relay agent hours: %.1fh" % (
        first_lane_hours[0], reason_hours["relay: at the context budget"]))

    by_count = collections.defaultdict(list)
    for row, n in row_lane_count.items():
        by_count[min(n, 5)].append(row_cost[row])
    for n in sorted(by_count):
        label = "%d" % n if n < 5 else "5+"
        vals = by_count[n]
        print("median row cost with %s lane agent(s): %.2f usd (n=%d)" % (label, statistics.median(vals), len(vals)))

    print()
    print("reason                                        count     hours       usd")
    for reason in sorted(reason_count, key=lambda r: -reason_usd[r]):
        print("%-45s %5d  %7.1fh  %8.2f" % (reason, reason_count[reason], reason_hours[reason], reason_usd[reason]))

    print()
    print("reason x writes-a-crate")
    for (reason, crate), count in sorted(reason_by_crate.items()):
        print("  %-45s crate=%-5s %5d" % (reason, crate, count))

    print()
    print("reason x model")
    for (reason, model), count in sorted(reason_by_model.items()):
        print("  %-45s model=%-8s %5d" % (reason, model, count))

    if relay_context:
        print()
        print("relays: lane-before's context at relay, median %d tokens (n=%d)" % (
            statistics.median(relay_context), len(relay_context)))
    if relay_first_edit:
        print("relays: lane-before's tokens at first edit, median %d (n=%d of %d, rest never edited before relaying)" % (
            statistics.median(relay_first_edit), len(relay_first_edit), len(relay_context)))
    if relay_commits:
        print("relays: lane-before's git-commit calls, median %d, zero in %d of %d" % (
            statistics.median(relay_commits), sum(1 for c in relay_commits if c == 0), len(relay_commits)))

    if other_examples:
        print()
        print("sample of 'other' extra-lane-agent cases:")
        for row, run_id, reason, detail in other_examples:
            print("  %s (%s): %s -- %s" % (row, run_id, reason, detail))


if __name__ == "__main__":
    main()
```

Run as (the claims file is not read by this script; `mise exec --` for `kache`):

```
mkdir -p target
mise exec -- python3 target/row_causes.py 2026-10-07 2026-10-21
mise exec -- python3 target/row_causes.py 2026-10-14 2026-10-21
```

### 461.6 What this leaves

**Q1418** (named by the row that filed this one) is a separate question — how
good each handoff was — and is left to it; §461.4's numbers (context, first
edit, commits at the moment of relay) are offered as its raw material, not as
an answer to it.

**Ten of the 1023 extra agents** (the last line of §461.2's table) carry an
`outcome` of `partial` or `fixed` with no verifier journal result this script
can find at all — too few, and too small a share of the money (12.03 USD of
3421.30), to chase further here.

What is worth fixing, filed below:

- **17 cases over two weeks where a verifier said `merge` or `fix_first` and
  the run's own record shows no `land:` or `merge:` agent at all, though the
  run's `status` is `completed`** (§461.2). Each one cost a full re-take
  later. `Q368`'s `wf_b391c4d3-0db` is a worked example: the run's top-level
  `result` already says "verified, not merged", so the information that no
  merge stage ran is sitting in the run record now, just never acted on by a
  later run of `lanes.js` that could have landed it there and then. This is a
  `lanes.js` behaviour, class `wrong/A` since a real run did it 17 times in
  the window, filed in this section's own `queue apply` as a trace row.

The commands that rebuilt every table in this section are §461.5's; running
them again against a later window is how a future row would check whether
this one still holds.
