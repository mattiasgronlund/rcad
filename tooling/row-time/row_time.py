#!/usr/bin/env python3
"""`just row-time`: where each queue row's time went, from `take` to merged and pruned.

`docs/decisions.md` §411 is the spec. This reads the `lanes` run records, the
transcripts of their agents and the claims file, and nothing else. For each
queue row it divides the row's elapsed time into its agents' running times and
the waits between them (§411.2, §411.4), and each agent's running time into
buckets: the model writing, each kind of command, and each tool (§411.3).

**Every sum is checked, and a sum that does not hold stops the script before
it prints anything.** A step's buckets add up to its running time, and a row's
parts to its elapsed time, to the millisecond: every time here is an int of
milliseconds since the epoch, as the run records keep them, so the sums are
exact and there is no tolerance to choose.

It keeps a loader of its own rather than `load()` in `spend.py`, which keeps
only the first record of each response, no tool result times and no run
record (§411.8).
"""

import collections
import datetime
import glob
import json
import os
import re
import shlex
import sys

TOP = 15
DAY_MS = 86_400_000
EPOCH = datetime.datetime(1970, 1, 1, tzinfo=datetime.timezone.utc)

# The label prefix names the role; 229 early agents have no `agentType`, and
# where both are there they never disagree (measured in §411.10). A relay
# carries on the lane itself, so it keeps the "lane" role; `prep` and `close`
# are roles of their own (§411.10).
PREFIXES = (("verify:", "verifier"), ("land:", "lander"), ("merge:", "merger"), ("prune:", "pruner"),
            ("prep:", "prep"), ("close:", "close"))
RELAY = re.compile(r"^relay \d+:")
RETRY = re.compile(r" \(retry \d+\)$")
MERGE_STAGE = ("lander", "merger", "close")

HOLD = re.compile(r"^(?:(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ) )?land: holding the land lock$", re.M)
UNLOCK = re.compile(r"\bgit\s+(?:-C\s+\S+\s+)?worktree\s+unlock\s+['\"]?([^\s'\";&|]+)")
LAND_UNLOCK = re.compile(r"^\+ git worktree unlock (\S+)$", re.M)
WAITING = re.compile(r"^(?:\d{4}-\d\d-\d\dT\d\d:\d\d:\d\dZ )?land: waiting for the land lock$", re.M)
PLAIN_WORD = re.compile(r"^[a-z]+(?:-[a-z]+)*$")
ASSIGNMENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")

CODE_TOOLS = ("Edit", "Write", "MultiEdit", "NotebookEdit")
BRIEF_TOOLS = ("Agent", "Task", "Workflow")
KEYED_BY_RECIPE = ("land", "verify-lane")

USAGE = ("usage: row_time.py <projects root> <repo key> <claims file> "
         "[Q<n>...] [--since YYYY-MM-DD] [--until YYYY-MM-DD] [--json] [--detail]")


class SumBroken(Exception):
    """A sum §411 says must hold did not. The script prints this and nothing else."""


def main(argv):
    options = parsed(argv)
    loaded = load(options["root"], options["repo"], options["claims"])
    try:
        report = build(loaded, options)
    except SumBroken as broken:
        print(f"row-time: {broken}", file=sys.stderr)
        return 1
    if options["json"]:
        print(json.dumps(report, indent=1, ensure_ascii=False))
    else:
        print_report(report, options["detail"])
    return 0


def parsed(argv):
    if len(argv) < 3:
        raise SystemExit(USAGE)
    options = {"root": argv[0], "repo": argv[1], "claims": argv[2], "rows": [],
               "since": None, "until": None, "json": False, "detail": False}
    rest = list(argv[3:])
    while rest:
        word = rest.pop(0)
        if word in ("--since", "--until"):
            if not rest:
                raise SystemExit(USAGE)
            options[word[2:]] = ms_of_date(rest.pop(0))
        elif word == "--json":
            options["json"] = True
        elif word == "--detail":
            options["detail"] = True
        elif re.match(r"^Q\d+$", word):
            options["rows"].append(word)
        elif word:
            raise SystemExit(f"row-time: '{word}' is neither a row id nor an option. {USAGE}")
    if not options["rows"]:
        today = (now_ms() // DAY_MS) * DAY_MS
        if options["since"] is None:
            options["since"] = today - 7 * DAY_MS
        if options["until"] is None:
            options["until"] = today + DAY_MS
    return options


# ---------------------------------------------------------------------------
# Loading


def load(root, repo, claims_path):
    """The run records, an index of the agent transcripts, and the claims events.

    Transcripts are only indexed here; one is read when a picked row needs it.
    """
    folders = [os.path.join(root, repo)] + sorted(glob.glob(os.path.join(root, repo + "-worktrees-*")))
    runs, empty = [], []
    for folder in folders:
        for path in sorted(glob.glob(os.path.join(folder, "*", "workflows", "*.json"))):
            if os.path.getsize(path) == 0:
                empty.append(path)
                continue
            with open(path, encoding="utf-8") as f:
                record = json.load(f)
            if record.get("workflowName") == "lanes":
                runs.append(run_of(path, record))
    transcripts = collections.defaultdict(list)
    for folder in folders:
        for path in glob.glob(os.path.join(folder, "*", "subagents", "workflows", "*", "agent-*.jsonl")):
            run_id = os.path.basename(os.path.dirname(path))
            agent_id = os.path.basename(path)[len("agent-"):-len(".jsonl")]
            transcripts[(run_id, agent_id)].append(path)
    return {"runs": runs, "empty": empty, "transcripts": transcripts, "claims": read_claims(claims_path)}


def run_of(path, record):
    """One `lanes` run: its lanes by label, and its agents with the times the record gives them."""
    lanes = {}
    for lane in (record.get("args") or {}).get("lanes") or []:
        rows = [row for row in (lane.get("queueId") or "").split(",") if row]
        lanes[lane["label"]] = {"label": lane["label"], "rows": rows,
                                "branch": lane.get("branch") or f"lane-{lane['label']}"}
    start = record["startTime"]
    run = {"id": record["runId"], "path": path, "status": record.get("status"), "start": start,
           "end": start + record["durationMs"], "lanes": lanes, "agents": [], "never_started": []}
    for entry in record.get("workflowProgress") or []:
        if entry.get("type") != "workflow_agent":
            continue
        label = entry["label"]
        role, lane_label = role_of(label)
        if entry.get("startedAt") is None:
            run["never_started"].append(label)
            continue
        started = entry["startedAt"]
        duration = entry.get("durationMs")
        # A killed run leaves its agents with no `durationMs`. Each ran until
        # the run ended: its last record is 11 to 59 ms past the run's end
        # (§411.10).
        end = started + duration if duration is not None else run["end"]
        assert end >= started, f"{path}: agent {label} ends before it starts"
        run["agents"].append({
            "run": run["id"], "id": entry["agentId"], "label": label, "lane": lane_label, "role": role,
            "queued": entry.get("queuedAt", started), "start": started, "end": end,
            "duration_known": duration is not None, "state": entry.get("state"),
        })
    run["agents"].sort(key=lambda agent: agent["start"])
    return run


def role_of(label):
    relayed = RELAY.match(label)
    if relayed:
        return "lane", RETRY.sub("", label[relayed.end():])
    for prefix, role in PREFIXES:
        if label.startswith(prefix):
            return role, RETRY.sub("", label[len(prefix):])
    return "lane", RETRY.sub("", label)


def read_claims(path):
    """`take` and `release` events from the claims file, oldest first, each with its time in ms."""
    events = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            fields = line.rstrip("\n").split("\t")
            if fields[0] == "take":
                events.append({"verb": "take", "row": fields[1], "time": ms_of_stamp(fields[2]), "branch": fields[4]})
            elif fields[0] == "release":
                events.append({"verb": "release", "row": fields[1], "time": ms_of_stamp(fields[2]), "how": fields[4]})
    events.sort(key=lambda event: event["time"])
    return events


def read_transcript(paths):
    """An agent's records in time order, its pieces joined.

    A transcript can be split into pieces under several session folders, and
    several project folders (§411.10). No two pieces overlap in time, so
    joining them in the order of their first records keeps every piece's own
    order.
    """
    pieces, lost = [], 0
    for path in paths:
        records = []
        with open(path, encoding="utf-8", errors="replace") as f:
            for line in f:
                if line.strip():
                    read, cut = records_of_line(line)
                    records += read
                    lost += cut
        records = [record for record in records if record.get("timestamp")]
        if records:
            pieces.append(records)
    pieces.sort(key=lambda records: ms_of_stamp(records[0]["timestamp"]))
    for before, after in zip(pieces, pieces[1:]):
        assert ms_of_stamp(after[0]["timestamp"]) >= ms_of_stamp(before[-1]["timestamp"]), \
            f"two pieces of one transcript overlap in time: {paths}"
    return [record for records in pieces for record in records], lost


def records_of_line(line):
    """The records one line holds, and how many it lost.

    A write cut off mid-record leaves the rest of that record missing and the
    next record on the same line (§411.10), so a line that does not parse is
    read again from each record start in it.
    """
    try:
        return [json.loads(line)], 0
    except ValueError:
        pass
    decoder = json.JSONDecoder()
    records, lost = [], 0
    for start in [found.start() for found in re.finditer(r'\{"parentUuid"', line)] or [0]:
        try:
            records.append(decoder.raw_decode(line, start)[0])
        except ValueError:
            lost += 1
    return records, lost


# ---------------------------------------------------------------------------
# Rows


def build(loaded, options):
    """Every picked row, its attempts and steps, the checks, and the summary over them."""
    assembled = assemble(loaded, options)
    return summarise(assembled, loaded, options)


def assemble(loaded, options):
    """The picked rows with their attempts, every step they ran, and what has no row (§411.7)."""
    attempts = collections.defaultdict(list)
    unattributed = []
    never_started = []
    branches = set()
    for run in loaded["runs"]:
        if picked_time(run["start"], options):
            never_started += run["never_started"]
        by_lane = collections.defaultdict(list)
        for agent in run["agents"]:
            by_lane[agent["lane"]].append(agent)
        for label, agents in by_lane.items():
            lane = run["lanes"].get(label)
            if lane is None or not lane["rows"]:
                if picked_time(agents[0]["start"], options):
                    unattributed.append({"run": run["id"], "lane": label, "agents": len(agents),
                                         "time": sum(agent["end"] - agent["start"] for agent in agents)})
                continue
            branches.add(lane["branch"])
            attempt = {"run": run, "lane": lane, "agents": agents}
            for row in lane["rows"]:
                attempts[row].append(attempt)

    takes = collections.defaultdict(list)
    releases = collections.defaultdict(list)
    for event in loaded["claims"]:
        (takes if event["verb"] == "take" else releases)[event["row"]].append(event)
    claims_start = loaded["claims"][0]["time"] if loaded["claims"] else None

    rows = []
    for row_id, its in attempts.items():
        row = row_of(row_id, its, takes[row_id], releases[row_id], claims_start)
        if picked(row, options):
            rows.append(row)
    rows.sort(key=lambda row: row["start"])

    landers = [agent for run in loaded["runs"] for agent in run["agents"] if land_lane_of(agent["label"]) is not None]
    steps = {}
    for row in rows:
        for attempt in row["attempts"]:
            for agent in attempt["agents"]:
                key = (agent["run"], agent["id"])
                if key not in steps:
                    steps[key] = step_of(agent, loaded["transcripts"].get(key, []), attempt)
    link_shared_gates(steps, landers)
    for row in rows:
        finish_row(row, steps)

    # §411.1: a row whose take names a branch no `lanes` run has was worked
    # with a plain `Agent` call or by hand, and is named, not broken down.
    outside = sorted({take["row"] for row_takes in takes.values() for take in row_takes
                      if take["branch"] not in branches
                      and (take["row"] in options["rows"] if options["rows"] else picked_time(take["time"], options))})
    return {"rows": rows, "steps": steps, "unattributed": unattributed, "outside": outside,
            "never started": never_started}


def row_of(row_id, attempts, takes, releases, claims_start):
    """A row's attempts in time order, each matched to its `take` by branch, and its release."""
    attempts = sorted(attempts, key=lambda attempt: attempt["agents"][0]["start"])
    row = {"id": row_id, "attempts": [], "marks": set()}
    previous_last = None
    for attempt in attempts:
        first = attempt["agents"][0]["start"]
        # A take belongs to the first attempt after it. A run killed and
        # started again under one take is two attempts and one take, and the
        # second waits from the first's end, not from the take (§411.10).
        matching = [take for take in takes
                    if take["branch"] == attempt["lane"]["branch"] and take["time"] <= first
                    and (previous_last is None or take["time"] >= previous_last)]
        take = matching[-1]["time"] if matching else None
        if take is None and previous_last is None:
            row["marks"].add("no take: run before the claims file starts"
                             if claims_start is None or first < claims_start else "no take with its branch")
        previous_last = max(agent["end"] for agent in attempt["agents"])
        row["attempts"].append({
            "run": attempt["run"]["id"], "lane": attempt["lane"]["label"], "rows": attempt["lane"]["rows"],
            "branch": attempt["lane"]["branch"], "agents": attempt["agents"],
            "take": take, "first": first, "last": max(agent["end"] for agent in attempt["agents"]),
        })
    first_attempt = row["attempts"][0]
    row["start"] = first_attempt["take"] if first_attempt["take"] is not None else first_attempt["first"]
    last_first = row["attempts"][-1]["first"]
    later = [release for release in releases if release["time"] >= last_first]
    row["release"] = later[0]["time"] if later else None
    row["released_how"] = later[0]["how"] if later else None
    return row


def picked(row, options):
    """A row is picked by its id, or by when it started: its first take, or its first agent with no take."""
    if options["rows"]:
        return row["id"] in options["rows"]
    return picked_time(row["start"], options)


def picked_time(time, options):
    return not options["rows"] and options["since"] <= time < options["until"]


def finish_row(row, steps):
    """The row's elapsed time divided as §411.2 says, checked, and its slot time by §411.6."""
    parts = collections.Counter()
    for name in ("waiting to start", "agents", "handoff", "earlier merge in the same run",
                 "between attempts", "after the run"):
        parts[name] = 0
    previous_last = None
    for attempt in row["attempts"]:
        start = attempt["take"] if attempt["take"] is not None else attempt["first"]
        if previous_last is not None:
            parts["between attempts"] += need_order(previous_last, start, row, "between attempts")
        span = attempt_parts(attempt, steps, row)
        need_sum(span, attempt["last"] - start, f"{row['id']}, attempt in {attempt['run']}")
        parts.update(span)
        previous_last = attempt["last"]
    last = row["attempts"][-1]["last"]
    end = last
    if row["release"] is not None and row["release"] > last:
        parts["after the run"] += row["release"] - last
        end = row["release"]
    row["end"] = end
    row["elapsed"] = end - row["start"]
    row["parts"] = dict(parts)
    need_sum(parts, row["elapsed"], row["id"])
    row["land lock"] = sum(steps[(agent["run"], agent["id"])]["buckets"].get("land lock", 0)
                           for attempt in row["attempts"] for agent in attempt["agents"])
    row["slot"] = sum(slot_time(attempt, steps, row) for attempt in row["attempts"])


def attempt_parts(attempt, steps, row):
    """One attempt from its `take` to its last agent's end: the wait to start, the agents, and the gaps."""
    parts = collections.Counter()
    agents = attempt["agents"]
    if attempt["take"] is not None:
        parts["waiting to start"] += need_order(attempt["take"], agents[0]["start"], row, "waiting to start")
    merge_queued = False
    for before, agent in zip([None] + agents, agents):
        step = steps[(agent["run"], agent["id"])]
        parts["agents"] += step["time"]
        if before is None:
            continue
        if agent["role"] in MERGE_STAGE and not merge_queued:
            parts["earlier merge in the same run"] += need_order(before["end"], agent["queued"], row,
                                                                 "earlier merge in the same run")
            parts["handoff"] += need_order(agent["queued"], agent["start"], row, "handoff")
        else:
            parts["handoff"] += need_order(before["end"], agent["start"], row, "handoff")
        merge_queued = merge_queued or agent["role"] in MERGE_STAGE
    return parts


def need_order(earlier, later, row, what):
    if later < earlier:
        raise SumBroken(f"{row['id']}: {what} runs backwards, {fmt_time(earlier)} to {fmt_time(later)}")
    return later - earlier


def need_sum(parts, total, what):
    added = sum(parts.values())
    if added != total or any(value < 0 for value in parts.values()):
        shown = ", ".join(f"{name} {value}" for name, value in parts.items())
        raise SumBroken(f"{what}: the parts add up to {added} ms, not {total} ms ({shown})")


def slot_time(attempt, steps, row):
    """§411.6: each tree an attempt's agents held, from its agent's start until it was unlocked.

    Plus the slot a row holds from its `take` until its lane starts (§263.4).
    A prep holds no tree of its own: it works in the dispatcher's own tree,
    never locked for the row, so it is skipped like the pruner (§411.10).
    """
    held = 0
    if attempt["take"] is not None:
        held += attempt["first"] - attempt["take"]
    agents = attempt["agents"]
    by_role = {agent["role"]: steps[(agent["run"], agent["id"])] for agent in agents}
    unlocks = [call for agent in agents for call in steps[(agent["run"], agent["id"])]["unlocks"]]
    lander = by_role.get("lander")
    hold = lander["hold"] if lander else None
    land_trees = lander["land_trees"] if lander else []
    for agent in agents:
        step = steps[(agent["run"], agent["id"])]
        tree = step["tree"]
        if agent["role"] in ("pruner", "prep"):
            continue
        if tree is None:
            row["marks"].add("still locked when the run ended")
            step["held"] = attempt["last"] - agent["start"]
            held += step["held"]
            continue
        moments = [time for time, path in unlocks if same_tree(path, tree)]
        if agent["role"] == "lane":
            verifier = by_role.get("verifier")
            if verifier and verifier["verify_lane_start"] is not None:
                moments.append(verifier["verify_lane_start"])
            if hold is not None and any(same_tree(path, tree) for path in land_trees):
                moments.append(hold)
        elif agent["role"] == "verifier":
            if hold is not None:
                moments.append(hold)
        elif agent["role"] == "lander":
            moments = [agent["end"]]
        elif agent["role"] in ("merger", "close"):
            moments = [time for time, path in step["unlocks"] if same_tree(path, tree)][-1:] or [agent["end"]]
        moments = [moment for moment in moments if moment >= agent["start"]]
        if moments:
            step["held"] = min(moments) - agent["start"]
        else:
            row["marks"].add("still locked when the run ended")
            step["held"] = attempt["last"] - agent["start"]
        held += step["held"]
    return held


def same_tree(path, tree):
    return path.rstrip("/") == tree.rstrip("/")


# ---------------------------------------------------------------------------
# Steps


def step_of(agent, paths, attempt):
    """One agent's running time divided into buckets that add up to it exactly (§411.3)."""
    step = {"agent": agent, "time": agent["end"] - agent["start"], "buckets": collections.Counter(),
            "own": collections.defaultdict(list), "tree": None, "unlocks": [], "verify_lane_start": None,
            "hold": None, "land_trees": [], "land_json": None, "marks": set(),
            "served": len(attempt["rows"]), "landed_by": None}
    records, lost = read_transcript(paths) if paths else ([], 0)
    if lost:
        step["marks"].add(f"{lost} records cut off")
    if not records:
        step["buckets"]["no transcript"] = step["time"]
        step["marks"].add("no transcript")
        return step
    step["tree"] = records[0].get("cwd")
    sweep(step, records)
    need_sum(step["buckets"], step["time"], f"{agent['label']} ({agent['run']}, agent {agent['id']})")
    return step


def sweep(step, records):
    """Walks the agent's records in time order and charges every millisecond of its running time once.

    The rule is §411.3's, with what §411.10 adds: while any response is being
    written, the time is the model's, charged to the block that ends it; when
    none is, the tools still waiting for a result share each moment equally;
    when none is waiting either, the time before a user message is a wait for
    that message, and the time after it is the next response's first block.
    """
    agent = step["agent"]
    t0, t1 = agent["start"], agent["end"]
    events = timeline(records, t0, t1)
    calls = {}
    for event in events:
        if event["kind"] == "block" and event["tool"] is not None:
            calls[event["tool"]["id"]] = event
    find_land(step, events, calls)
    if step["hold"] is not None and t0 <= step["hold"] <= t1:
        events.append({"time": step["hold"], "order": -1, "kind": "mark"})
        events.sort(key=lambda event: (event["time"], event["order"]))

    first_block, last_block = {}, {}
    for index, event in enumerate(events):
        if event["kind"] == "block":
            first_block.setdefault(event["response"], index)
            last_block[event["response"]] = index
    next_block = [None] * (len(events) + 1)
    next_message = [False] * (len(events) + 1)
    for index in range(len(events) - 1, -1, -1):
        event = events[index]
        next_block[index] = event if event["kind"] == "block" else next_block[index + 1]
        next_message[index] = (event["kind"] == "message") or (event["kind"] != "block" and next_message[index + 1])

    buckets = step["buckets"]
    first_record = ms_clamped(records[0]["timestamp"], t0, t1)
    buckets["before the first record"] += first_record - t0
    open_responses = set()
    waiting = {}
    resolved = set()
    overlaps = 0
    cursor = first_record
    for index, event in enumerate(events):
        if event["time"] > cursor:
            charge(step, cursor, event["time"], open_responses, waiting, next_block[index], next_message[index])
            cursor = event["time"]
        if event["kind"] == "block":
            response = event["response"]
            if first_block[response] == index and open_responses:
                overlaps += 1
            if first_block[response] == index:
                open_responses.add(response)
            if last_block[response] == index:
                open_responses.discard(response)
            if event["tool"] is not None and event["tool"]["id"] not in resolved:
                waiting[event["tool"]["id"]] = event
        elif event["kind"] == "result":
            for tool_id in event["ids"]:
                resolved.add(tool_id)
                call = waiting.pop(tool_id, None) or calls.get(tool_id)
                if call is not None:
                    step["own"][call["key"]].append(event["time"] - call["time"])
    if t1 > cursor:
        if waiting:
            charge(step, cursor, t1, set(), waiting, None, False)
        else:
            buckets["after the last record"] += t1 - cursor
    for call in waiting.values():
        step["own"][call["key"]].append(t1 - call["time"])
    if overlaps:
        step["marks"].add(f"two copies ran at once: {overlaps} responses began inside another")


def charge(step, start, end, open_responses, waiting, next_block, message_first):
    buckets = step["buckets"]
    if open_responses:
        buckets[model_bucket(next_block)] += end - start
    elif waiting:
        calls = sorted(waiting.values(), key=lambda call: (call["time"], call["order"]))
        whole, left = divmod(end - start, len(calls))
        for number, call in enumerate(calls):
            buckets[tool_bucket(step, call, end)] += whole + (1 if number < left else 0)
    elif message_first or next_block is None:
        buckets["waiting for a message"] += end - start
    else:
        buckets[model_bucket(next_block)] += end - start


def model_bucket(block):
    assert block is not None, "the model was writing with no block left to write"
    if block["block"] in ("thinking", "redacted_thinking"):
        return "model: thinking"
    if block["block"] == "text":
        return "model: prose"
    name = block["tool"]["name"]
    if name in CODE_TOOLS:
        return "model: writing code"
    if name == "Bash":
        return "model: writing commands"
    if name in BRIEF_TOOLS:
        return "model: writing briefs"
    return "model: writing other tool input"


def tool_bucket(step, call, end):
    if call["key"] == "just land" and step["agent"]["role"] == "lander":
        hold = step["hold"]
        if step["landed_by"] is not None or (hold is not None and end <= hold):
            return "land lock"
    return call["key"]


def timeline(records, t0, t1):
    """The records that move the sweep, each clamped into the agent's running time.

    A block is one assistant record: one content block, stamped when it
    finished streaming. A result is a user record holding `tool_result`
    blocks; any other user record is a message, the prompt or injected text.
    Attachments are hook output and the like, and move nothing (§411.10).
    """
    events = []
    for order, record in enumerate(records):
        kind = record.get("type")
        time = ms_clamped(record["timestamp"], t0, t1)
        message = record.get("message") or {}
        content = message.get("content")
        if kind == "assistant":
            # One block to a record, but for 5 records in 120 000 that hold
            # two or three (§411.10). Those blocks share one time, so all but
            # the first are charged nothing.
            assert isinstance(content, list) and content, "an assistant record holds no block"
            for block in content:
                event = {"time": time, "order": order, "kind": "block", "block": block.get("type"),
                         "response": (record.get("requestId"), message.get("id")), "tool": None}
                if block.get("type") == "tool_use":
                    event["tool"] = {"id": block.get("id"), "name": block.get("name"),
                                     "input": block.get("input") or {}}
                    event["key"] = key_of(block.get("name"), block.get("input") or {})
                events.append(event)
        elif kind == "user":
            ids, texts = [], []
            if isinstance(content, list):
                for block in content:
                    if isinstance(block, dict) and block.get("type") == "tool_result":
                        ids.append(block.get("tool_use_id"))
                        texts.append(text_of(block.get("content")))
            if ids:
                events.append({"time": time, "order": order, "kind": "result", "ids": ids, "texts": texts})
            else:
                events.append({"time": time, "order": order, "kind": "message"})
    events.sort(key=lambda event: (event["time"], event["order"]))
    return events


def find_land(step, events, calls):
    """The slot rule's moments and the land lock's end, out of the agent's own commands (§411.4, §411.6)."""
    results = {}
    for event in events:
        if event["kind"] == "result":
            for tool_id, text in zip(event["ids"], event["texts"]):
                results[tool_id] = text
    waited = False
    for call in sorted(calls.values(), key=lambda call: (call["time"], call["order"])):
        if call["tool"]["name"] != "Bash":
            continue
        command = call["tool"]["input"].get("command") or ""
        for path in UNLOCK.findall(command):
            step["unlocks"].append((call["time"], path))
        recipes = call["key"].split(" ", 1)[1].split("+") if call["key"].startswith("just ") else []
        if "verify-lane" in recipes and step["verify_lane_start"] is None:
            step["verify_lane_start"] = call["time"]
        if "land" not in recipes or step["agent"]["role"] != "lander":
            continue
        trees = land_trees(command)
        if trees:
            step["land_trees"] = trees
        said = results.get(call["tool"]["id"], "")
        # `land.py` echoes each command it runs, and a lander from before
        # §397.5's lock prints no hold line, so its unlocks are read here. The
        # echo has no time, so the call's start stands for it, a lower bound.
        for path in LAND_UNLOCK.findall(said):
            step["unlocks"].append((call["time"], path))
        held = HOLD.search(said)
        if held and step["hold"] is None:
            if held.group(1):
                step["hold"] = ms_of_stamp(held.group(1))
            else:
                step["hold"] = call["time"]
                step["marks"].add("land lock bound")
        if WAITING.search(said):
            waited = True
        for line in reversed(said.strip().splitlines()):
            if line.startswith("{") and '"merge_branch"' in line:
                step["land_json"] = json.loads(line)
                break
    # A lander killed while it waited never prints the hold line, and all of
    # its `land` time was the wait (§411.10).
    if waited and step["hold"] is None:
        step["hold"] = step["agent"]["end"]
        step["marks"].add("never held the land lock")


def land_trees(command):
    """The trees named at the end of a `just land <rows> <branch> <sha> <merge branch> <trees>...` command."""
    for words in segments_of(command):
        words = stripped(words)
        if len(words) < 2 or os.path.basename(words[0]) != "just":
            continue
        args = recipe_args(words[1:])
        if not args or args[0] != "land":
            continue
        args = args[1:]
        if args[:1] == ["--steps"]:
            args = args[2:]
        if len(args) >= 4 and args[0] not in ("wait", "wait-for"):
            return args[4:]
    return []


def link_shared_gates(steps, landers):
    """§411.5: a lander whose lane another run's lander landed names that run, and the holder serves its rows too.

    The holder can itself have relayed past its budget mid-`just land` (§416): `role_of` then
    keeps its role `lane` and keeps the `land:` prefix in the lane field it computes, so neither
    `agent["role"]` nor `agent["lane"]` names it a lander. `land_lane_of` reads the raw label
    instead, straight or relayed, and `landers` is built the same way.
    """
    by_label = collections.defaultdict(list)
    for agent in landers:
        by_label[land_lane_of(agent["label"])].append(agent)
    for (run_id, agent_id), step in list(steps.items()):
        agent = step["agent"]
        said = step["land_json"]
        if agent["role"] != "lander" or not said:
            continue
        holder_label = said.get("merge_branch", "")[len("merge-"):]
        if not said.get("merge_branch", "").startswith("merge-") or holder_label == agent["lane"]:
            continue
        candidates = [other for other in by_label.get(holder_label, [])
                      if other["start"] <= agent["end"] and other["end"] >= agent["start"]]
        assert candidates, f"{agent['label']} was landed by {holder_label}, and no lander of that lane ran beside it"
        holder = candidates[0]
        step["landed_by"] = {"run": holder["run"], "lane": holder_label}
        step["buckets"] = recharged(step)
        key = (holder["run"], holder["id"])
        if key in steps:
            steps[key]["served"] += step["served"]
            steps[key].setdefault("landed_for", []).append(agent["lane"])


def land_lane_of(label):
    """The lane a `land:` step names, straight or relayed past its budget mid-run (§411.5, §416).

    `role_of` strips only the relay prefix off a relayed label, so a relayed lander's own
    "land:" prefix stays in the lane field it returns. This strips both, in whichever order
    the label carries them, so a straight and a relayed lander for the same lane answer alike.
    """
    relayed = RELAY.match(label)
    rest = label[relayed.end():] if relayed else label
    return RETRY.sub("", rest[len("land:"):]) if rest.startswith("land:") else None


def recharged(step):
    """When another run's lander landed this lane, every moment of `just land` was a wait for the land lock (§411.4)."""
    buckets = collections.Counter(step["buckets"])
    buckets["land lock"] += buckets.pop("just land", 0)
    return buckets


# ---------------------------------------------------------------------------
# What a command is filed under (§411.3)


def key_of(name, tool_input):
    if name != "Bash":
        return name
    return command_key(tool_input.get("command") or "")


def command_key(command):
    """A `Bash` call's one key: the `just` recipes it runs joined with `+`, or else its first program."""
    recipes, first = [], None
    for words in segments_of(command):
        words = stripped(words)
        if not words or words[0] == "cd":
            continue
        program = os.path.basename(words[0])
        if program == "just":
            recipe = just_key(words[1:])
            if recipe not in recipes:
                recipes.append(recipe)
        elif first is None:
            first = program_key(program, words[1:])
    if recipes:
        return "just " + "+".join(recipes)
    return first or "(no command)"


def segments_of(command):
    """The command's segments, split at `&&`, `||`, `;`, `&`, newlines and brackets, each cut at its first `|`."""
    try:
        tokens = shell_tokens(command)
    except ValueError:
        # A heredoc's body is not shell, and an apostrophe in it leaves a
        # quote unclosed; its first line is the command that matters.
        try:
            tokens = shell_tokens(command.split("\n", 1)[0])
        except ValueError:
            tokens = command.split("\n", 1)[0].split()
    segments, words, piped = [], [], False
    for token in tokens:
        if token and all(char in "();<>|&\n" for char in token):
            if "<" in token or ">" in token:
                continue
            if token in ("|", "|&"):
                piped = True
                continue
            segments.append(words)
            words, piped = [], False
        elif not piped:
            words.append(token)
    segments.append(words)
    return [segment for segment in segments if segment]


def shell_tokens(command):
    lexer = shlex.shlex(command, posix=True, punctuation_chars="();<>|&\n")
    lexer.whitespace = " \t\r"
    lexer.whitespace_split = True
    return list(lexer)


def stripped(words):
    """Drops what runs a command rather than being it: assignments, `mise exec --`, `timeout N`."""
    words = list(words)
    while words:
        if ASSIGNMENT.match(words[0]):
            words.pop(0)
        elif words[0] == "mise" and words[1:2] == ["exec"]:
            if "--" not in words:
                return words
            words = words[words.index("--") + 1:]
        elif words[0] == "timeout":
            words.pop(0)
            while words and words[0].startswith("-"):
                option = words.pop(0)
                if option in ("-k", "-s", "--kill-after", "--signal") and words:
                    words.pop(0)
            if words:
                words.pop(0)
        else:
            break
    return words


def just_key(args):
    args = recipe_args(args)
    if not args:
        return "(list)"
    recipe = args[0]
    if recipe in KEYED_BY_RECIPE or len(args) < 2 or not PLAIN_WORD.match(args[1]):
        return recipe
    return f"{recipe} {args[1]}"


def recipe_args(args):
    """`just`'s arguments from the recipe on, its own options dropped."""
    args = list(args)
    while args and args[0].startswith("-"):
        option = args.pop(0)
        if option in ("-f", "--justfile", "-d", "--working-directory") and args:
            args.pop(0)
        elif option == "--set" and len(args) >= 2:
            del args[:2]
    return args


def program_key(program, args):
    if program not in ("git", "cargo"):
        return program
    args = list(args)
    while args and (args[0].startswith("-") or args[0].startswith("+")):
        option = args.pop(0)
        if option in ("-C", "-c") and args:
            args.pop(0)
    return f"{program} {args[0]}" if args else program


# ---------------------------------------------------------------------------
# The report


def summarise(assembled, loaded, options):
    rows, steps, unattributed = assembled["rows"], assembled["steps"], assembled["unattributed"]
    return {
        "window": None if options["rows"] else [fmt_time(options["since"]), fmt_time(options["until"])],
        "rows": [row_report(row, steps) for row in rows],
        "summary": summary_of(rows, steps),
        "unattributed": {"lanes": len(unattributed), "time": sum(item["time"] for item in unattributed)},
        "worked outside a workflow": assembled["outside"],
        "agents that never started": assembled["never started"],
        "empty run records": [os.path.basename(path) for path in loaded["empty"]],
    }


def row_report(row, steps):
    step_marks = {mark for attempt in row["attempts"] for agent in attempt["agents"]
                  for mark in steps[(agent["run"], agent["id"])]["marks"]}
    return {
        "row": row["id"], "start": fmt_time(row["start"]), "end": fmt_time(row["end"]),
        "elapsed": row["elapsed"], "parts": row["parts"], "land lock": row["land lock"], "slot": row["slot"],
        "released": row["released_how"], "marks": sorted(row["marks"] | step_marks),
        "attempts": [{
            "run": attempt["run"], "lane": attempt["lane"], "rows": attempt["rows"],
            "take": fmt_time(attempt["take"]) if attempt["take"] is not None else None,
            "steps": [step_report(steps[(agent["run"], agent["id"])]) for agent in attempt["agents"]],
        } for attempt in row["attempts"]],
    }


def step_report(step):
    agent = step["agent"]
    return {
        "role": agent["role"], "agent": agent["id"], "label": agent["label"], "start": fmt_time(agent["start"]),
        "time": step["time"], "served": step["served"], "share": step["time"] / step["served"],
        "buckets": dict(step["buckets"].most_common()),
        "own": {key: sum(times) for key, times in step["own"].items()},
        "tree held": step.get("held"),
        "landed by": step["landed_by"], "landed for": step.get("landed_for", []), "marks": sorted(step["marks"]),
    }


def summary_of(rows, steps):
    """Total share, median and p90 over the picked rows: each part of elapsed, each role's time, each bucket.

    The median and p90 are of each row's full time. The total adds up shares,
    a step's time divided by the rows it served (§411.5), so that a bundle's
    lane is counted once in it and not once per row.
    """
    per_row = collections.defaultdict(lambda: [0] * len(rows))
    shares = collections.Counter()
    for number, row in enumerate(rows):
        bundle = max(len(attempt["rows"]) for attempt in row["attempts"])
        whole = {name: value for name, value in row["parts"].items() if name != "agents"}
        whole.update({"elapsed": row["elapsed"], "slot": row["slot"],
                      "land lock (in the lander's time)": row["land lock"]})
        for name, value in whole.items():
            group = "row" if name in ("elapsed", "slot") else "wait"
            per_row[(group, name)][number] = value
            shares[(group, name)] += value / bundle
        for attempt in row["attempts"]:
            for agent in attempt["agents"]:
                step = steps[(agent["run"], agent["id"])]
                served = step["served"]
                per_row[("step", agent["role"])][number] += step["time"]
                shares[("step", agent["role"])] += step["time"] / served
                for bucket, value in step["buckets"].items():
                    per_row[("bucket", bucket)][number] += value
                    shares[("bucket", bucket)] += value / served
    out = {}
    for key, values in per_row.items():
        ordered = sorted(values)
        out[f"{key[0]}: {key[1]}"] = {"total share": round(shares[key]), "median": percentile(ordered, 50),
                                     "p90": percentile(ordered, 90)}
    return out


def percentile(ordered, at):
    if not ordered:
        return 0
    index = min(len(ordered) - 1, max(0, round(at / 100 * (len(ordered) - 1))))
    return ordered[index]


def print_report(report, detail):
    window = report["window"]
    print(f"row-time: {len(report['rows'])} rows" + (f", started {window[0]} to {window[1]}" if window else ""))
    print()
    print_rows(report["rows"])
    if detail:
        for row in report["rows"]:
            print_detail(row)
    print()
    print_summary(report["summary"])
    print()
    print("A model bucket holds the time to write its blocks, and a response's first block holds the wait "
          "for its first token too. A tool holds the time from its response's last block to its result, "
          "split equally with the tools running beside it; `own` is from its call to its result. Totals add "
          "up shares: a step's time over the rows it served.")
    print()
    print(f"unattributed: {report['unattributed']['lanes']} lanes with no row, "
          f"{fmt_span(report['unattributed']['time'])} of agent time")
    outside = report["worked outside a workflow"]
    print(f"worked outside a workflow: {len(outside)} rows" + (f": {', '.join(outside)}" if outside else ""))
    never = report["agents that never started"]
    print(f"agents that never started: {len(never)}" + (f": {', '.join(never)}" if never else ""))
    empty = report["empty run records"]
    print(f"empty run records: {len(empty)}" + (f": {', '.join(empty)}" if empty else ""))


def print_rows(rows):
    header = ("row", "elapsed", "agents", "to start", "handoffs", "merge wait", "land lock", "between", "after",
              "slot", "tries")
    print(" ".join(f"{name:>10}" for name in header))
    shown = sorted(rows, key=lambda row: row["elapsed"], reverse=True)
    for row in shown[:TOP]:
        parts = row["parts"]
        cells = [row["row"], fmt_span(row["elapsed"]), fmt_span(parts["agents"]), fmt_span(parts["waiting to start"]),
                 fmt_span(parts["handoff"]), fmt_span(parts["earlier merge in the same run"]),
                 fmt_span(row["land lock"]), fmt_span(parts["between attempts"]), fmt_span(parts["after the run"]),
                 fmt_span(row["slot"]), str(len(row["attempts"]))]
        print(" ".join(f"{cell:>10}" for cell in cells) + (f"  {'; '.join(row['marks'])}" if row["marks"] else ""))
    rest = shown[TOP:]
    if rest:
        print(f"{'other':>10} {fmt_span(sum(row['elapsed'] for row in rest)):>10}  ({len(rest)} rows)")


def print_detail(row):
    print()
    print(f"{row['row']}: {fmt_span(row['elapsed'])} from {row['start']} to {row['end']}, "
          f"released {row['released'] or 'never'}")
    for attempt in row["attempts"]:
        print(f"  attempt in {attempt['run']}, lane {attempt['lane']}, rows {','.join(attempt['rows'])}, "
              f"taken {attempt['take'] or 'never'}")
        for step in attempt["steps"]:
            served = f", serves {step['served']}, share {fmt_span(step['share'])}" if step["served"] > 1 else ""
            held = f", tree held {fmt_span(step['tree held'])}" if step["tree held"] is not None else ""
            print(f"    {step['role']:<9} {step['start']} {fmt_span(step['time']):>8}{served}{held}"
                  + (f"  [{'; '.join(step['marks'])}]" if step["marks"] else ""))
            if step["landed by"]:
                print(f"      landed by {step['landed by']['lane']} in {step['landed by']['run']}")
            if step["landed for"]:
                print(f"      landed beside its own: {', '.join(step['landed for'])}")
            buckets = list(step["buckets"].items())
            for name, value in buckets[:TOP]:
                own = step["own"].get(name)
                print(f"      {fmt_span(value):>8}  {name}" + (f"  (own {fmt_span(own)})" if own is not None else ""))
            if len(buckets) > TOP:
                print(f"      {fmt_span(sum(value for _, value in buckets[TOP:])):>8}  other")


def print_summary(summary):
    print(f"{'over the rows picked':<48} {'total share':>12} {'median':>9} {'p90':>9}")
    for group in ("row", "wait", "step"):
        for name, numbers in sorted(summary.items(), key=lambda item: -item[1]["total share"]):
            if name.startswith(group + ": "):
                print(f"{name:<48} {fmt_span(numbers['total share']):>12} {fmt_span(numbers['median']):>9} "
                      f"{fmt_span(numbers['p90']):>9}")
    buckets = sorted(((name, numbers) for name, numbers in summary.items() if name.startswith("bucket: ")),
                     key=lambda item: -item[1]["total share"])
    for name, numbers in buckets[:TOP]:
        print(f"{name:<48} {fmt_span(numbers['total share']):>12} {fmt_span(numbers['median']):>9} "
              f"{fmt_span(numbers['p90']):>9}")
    if len(buckets) > TOP:
        print(f"{'bucket: other':<48} {fmt_span(sum(numbers['total share'] for _, numbers in buckets[TOP:])):>12}")


# ---------------------------------------------------------------------------
# Times


def ms_of_stamp(stamp):
    when = datetime.datetime.fromisoformat(stamp.replace("Z", "+00:00"))
    return (when - EPOCH) // datetime.timedelta(milliseconds=1)


def ms_clamped(stamp, t0, t1):
    return min(max(ms_of_stamp(stamp), t0), t1)


def ms_of_date(text):
    return ms_of_stamp(datetime.date.fromisoformat(text).isoformat() + "T00:00:00Z")


def now_ms():
    return (datetime.datetime.now(datetime.timezone.utc) - EPOCH) // datetime.timedelta(milliseconds=1)


def fmt_time(ms):
    when = EPOCH + datetime.timedelta(milliseconds=ms)
    return when.strftime("%Y-%m-%dT%H:%M:%SZ")


def fmt_span(ms):
    seconds = round(ms / 1000)
    if seconds < 60:
        return f"{seconds}s"
    if seconds < 3600:
        return f"{seconds // 60}m{seconds % 60:02d}s"
    return f"{seconds // 3600}h{seconds % 3600 // 60:02d}m"


def text_of(content):
    if isinstance(content, list):
        return "".join(block.get("text", "") for block in content if isinstance(block, dict))
    return str(content or "")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
