#!/usr/bin/env python3
"""What a week of sessions cost, and what it was spent on.

Reads the Claude Code transcripts under `~/.claude/projects/` for this repo and
its worktrees. Nothing here touches the repository; it is a measuring tool.

Run it through `just spend`, which passes the transcript root.

## The one thing to get right

A transcript writes **one record per content block**, and every record of the
same API response repeats that response's `usage`. A response that thought and
then called a tool is two records carrying the same numbers. Summing records
therefore double-counts, by about 2x, and the error is invisible because the
shape of the answer stays plausible. The unit of billing is the API response:
`(requestId, message.id)`. `load()` groups on that pair and nothing else may.

## What the numbers mean

Output tokens are a lower bound, not a count. A response's usage grows from
one record to the next as it streams, so `load()` keeps the largest it saw —
but about half of all responses never write a record carrying a final
`stop_reason` at all, and for those the largest we saw is still short of the
true total. There is no record left to read it from; it is not in these
transcripts.

Almost everything is cache read: each response re-sends the conversation, so a
tool result of N tokens is paid once as a cache write and again on every later
response in that session. That is why `classes` reports "carry" — result tokens
times the responses that followed — rather than bytes returned. Bytes returned
is what a result costs once; carry is what it costs in total.

## Two ceilings, and they answer different questions

`ceiling` prices a cap nothing enforces: what the week would have cost had no
response ever been billed above one. `warnings` prices the ceiling that is
actually installed — the `guardrails` context budget hook — by splitting the
same spend into what was billed under the budget, inside the turn that crossed
it, and after the session had been told. The first says what a dial is worth;
the second says whether the dial is where the money is. On the week of
2026-09-12 they disagreed, and `docs/decisions.md` §160 is the argument.
"""

import collections
import datetime
import glob
import json
import os
import re
import shlex
import sys

# $/MTok as published on 2026-09-19: (input, output, cache read, 1h write, 5m write).
# Cache read is a tenth of input and a 1h write is double it, so only the first
# two numbers of each row are independent. Re-check these against the current
# price list before quoting a total; the shares are far more durable than the
# dollars, which is why every report prints both.
PRICES = {
    "opus": (5.0, 25.0, 0.50, 10.0, 6.25),
    "sonnet": (2.0, 10.0, 0.20, 4.0, 2.50),
    "haiku": (1.0, 5.0, 0.10, 2.0, 1.25),
}

CHARS_PER_TOKEN = 3.6  # good to ~10% on this repo's mix of Rust and prose


def tier_of(model):
    name = (model or "").lower()
    return next((t for t in PRICES if t in name), "opus")


def cost_of(model, usage):
    inp, out, read, write_1h, write_5m = PRICES[tier_of(model)]
    created = usage.get("cache_creation") or {}
    hour = created.get("ephemeral_1h_input_tokens", 0)
    five = created.get("ephemeral_5m_input_tokens", 0)
    if not (hour or five):
        five = usage.get("cache_creation_input_tokens", 0)
    return (
        usage.get("input_tokens", 0) * inp
        + usage.get("output_tokens", 0) * out
        + usage.get("cache_read_input_tokens", 0) * read
        + hour * write_1h
        + five * write_5m
    ) / 1e6


def transcript_files(root, repo):
    """Every transcript for the repo and its worktrees, main sessions and lanes."""
    roots = [os.path.join(root, repo)] + sorted(
        glob.glob(os.path.join(root, repo + "-worktrees-*"))
    )
    for base in roots:
        found = set(glob.glob(os.path.join(base, "*.jsonl")))
        found |= set(glob.glob(os.path.join(base, "**", "*.jsonl"), recursive=True))
        for path in sorted(found):
            yield path


def load(root, repo, since, until, only=None):
    """Every API response in the window, plus the tool results they produced.

    Returns (responses, results, warnings). A response carries the tools it
    called, so a report can attribute its cost; `results` maps a tool_use id to
    what came back, for the reports that care how large an answer was;
    `warnings` maps a session to the times the context budget hook spoke to it.

    Warnings are **not** filtered to the window, and the responses are. A
    session that was warned on Sunday and went on spending on Monday is the
    whole subject of the `warnings` report, and dropping the warning would move
    that money into the bucket for sessions nothing ever told.
    """
    responses = []
    results = {}
    warnings = collections.defaultdict(list)
    for path in transcript_files(root, repo):
        if only and only not in path:
            continue
        is_lane = "subagents" in path
        session = os.path.basename(path)[:8]
        try:
            lines = open(path, errors="replace").read().splitlines()
        except OSError:
            continue

        grouped, order = {}, []
        for line in lines:
            if '"attachment"' in line and "Context budget" in line:
                record = _parse(line) or {}
                attachment = record.get("attachment") or {}
                said = warning_said(f"{attachment.get('stdout') or ''}"
                                    f"{attachment.get('content') or ''}")
                stamp = record.get("timestamp")
                if said and stamp:
                    said["when"] = datetime.datetime.fromisoformat(stamp.replace("Z", "+00:00"))
                    # One warning is recorded twice — once whole, once as its
                    # summary line — and the two carry the same token count,
                    # which no two real warnings can (the second would be on the
                    # same rung and so would never be spoken).
                    twin = next((w for w in warnings[session]
                                 if w["tokens"] == said["tokens"]), None)
                    if twin is None:
                        warnings[session].append(said)
                    elif twin["budget"] is None:
                        twin["budget"] = said["budget"]
            elif '"assistant"' in line:
                record = _parse(line)
                if not record or record.get("type") != "assistant":
                    continue
                message = record.get("message", {})
                key = (record.get("requestId"), message.get("id"))
                if key not in grouped:
                    grouped[key] = {"record": record, "content": []}
                    order.append(key)
                else:
                    # usage.output_tokens grows from one record of a response to
                    # the next as it streams, so the record with the largest
                    # count is the closest we have to the true total — never
                    # the first, which is `load`'s whole subject (§396.1).
                    kept = (grouped[key]["record"].get("message") or {}).get("usage") or {}
                    seen = message.get("usage") or {}
                    if seen.get("output_tokens", 0) > kept.get("output_tokens", 0):
                        grouped[key]["record"] = record
                grouped[key]["content"].extend(message.get("content") or [])
            elif '"tool_result"' in line:
                record = _parse(line)
                for block in ((record or {}).get("message") or {}).get("content") or []:
                    if isinstance(block, dict) and block.get("type") == "tool_result":
                        results[block.get("tool_use_id")] = _text_of(block.get("content"))
            elif "<task-notification>" in line:
                # A dispatched agent answers long after the call that started it:
                # the tool result is a launch receipt, and what the agent found
                # arrives later as a notification carrying the same tool_use id.
                # Both are text the asking session carries, so both belong here.
                record = _parse(line)
                prompt = ((record or {}).get("attachment") or {}).get("prompt", "")
                call = re.search(r"<tool-use-id>(\w+)</tool-use-id>", prompt)
                answer = re.search(r"<result>(.*)</result>", prompt, re.S)
                if call and answer:
                    results[call.group(1)] = results.get(call.group(1), "") + answer.group(1)

        for key in order:
            group = grouped[key]
            record, message = group["record"], group["record"]["message"]
            usage, stamp = message.get("usage") or {}, record.get("timestamp")
            if not usage or not stamp:
                continue
            when = datetime.datetime.fromisoformat(stamp.replace("Z", "+00:00"))
            if not (since <= when < until):
                continue
            responses.append(
                {
                    "session": session,
                    "lane": is_lane,
                    "path": path,
                    "when": when,
                    "model": message.get("model", ""),
                    "cost": cost_of(message.get("model"), usage),
                    "context": (
                        usage.get("cache_read_input_tokens", 0)
                        + usage.get("cache_creation_input_tokens", 0)
                        + usage.get("input_tokens", 0)
                    ),
                    "read_tokens": usage.get("cache_read_input_tokens", 0),
                    "read_rate": PRICES[tier_of(message.get("model"))][2],
                    "output": usage.get("output_tokens", 0),
                    "parts": _parts(message.get("model"), usage),
                    "tools": [b for b in group["content"] if b.get("type") == "tool_use"],
                    "cwd": record.get("cwd", ""),
                }
            )

    responses.sort(key=lambda r: r["when"])
    return responses, results, warnings


# What the hook said about itself: the context it measured and the budget it was
# holding at the time. Reading the budget out of the message rather than off
# today's `context-budget.default` is what lets a window straddle a change to
# the dial — the week of 2026-09-12 has both 250k and 200k in it.
SAID = re.compile(r"Context budget (passed at|still growing:) (\d+) tokens")
AGAINST = re.compile(r"past the ([\d,]+) budget")


def warning_said(text):
    """The hook's own account of one warning, or None if this is not one.

    Only a hook attachment's own text is ever passed here. A transcript that
    merely *quotes* the hook — a session reading `context-budget.sh`, which is
    how this function came to be written — must not count as a warning
    delivered.

    A transcript records a hook two ways: `hook_success` carries the whole
    stdout, and `hook_system_message` carries the one-line summary and nothing
    else. Both are warnings; only the first names the budget, so `budget` is
    None for the other and the report dates it off its neighbours instead.
    """
    said = SAID.search(text)
    if not said:
        return None
    against = AGAINST.search(text)
    return {
        "tokens": int(said.group(2)),
        "budget": int(against.group(1).replace(",", "")) if against else None,
        "repeat": said.group(1) != "passed at",
    }


def bucket_of(context, when, budget, warned_at):
    """Which side of the budget, and of the warning, one response was billed on."""
    if context < budget:
        return "under the budget"
    if warned_at is None:
        return "never warned"
    return "before the warning" if when <= warned_at else "after the warning"


def _parse(line):
    try:
        return json.loads(line)
    except ValueError:
        return None


def _text_of(content):
    if isinstance(content, list):
        return "".join(b.get("text", "") for b in content if isinstance(b, dict))
    return str(content or "")


def _parts(model, usage):
    inp, out, read, write_1h, write_5m = PRICES[tier_of(model)]
    created = usage.get("cache_creation") or {}
    hour = created.get("ephemeral_1h_input_tokens", 0)
    five = created.get("ephemeral_5m_input_tokens", 0)
    if not (hour or five):
        five = usage.get("cache_creation_input_tokens", 0)
    return {
        "input": usage.get("input_tokens", 0) * inp / 1e6,
        "output": usage.get("output_tokens", 0) * out / 1e6,
        "cache read": usage.get("cache_read_input_tokens", 0) * read / 1e6,
        "cache write": (hour * write_1h + five * write_5m) / 1e6,
    }


# ---------------------------------------------------------------- classifying

def bash_kind(command):
    """What a shell command was for. Only the head matters, after the runners."""
    words = command.split()
    head = words[0] if words else ""
    if head in ("mise", "env", "timeout", "nohup"):
        for word in words[1:]:
            if word not in ("exec", "--") and not word.startswith("-") and "=" not in word:
                head = word
                break
    if re.search(r"just (gate|test|t\b)|cargo (test|nextest|fuzz)|nextest", command):
        return "running tests"
    if re.search(r"just (decisions?|src|question|lanes|status|plan)", command):
        return "finding"
    if head in ("grep", "rg", "find", "ls", "cat", "head", "tail", "wc", "sed",
                "awk", "du", "stat", "jq", "sort", "uniq", "diff", "comm"):
        return "finding"
    if head == "git":
        verb = words[1] if len(words) > 1 else ""
        return "finding" if verb in (
            "log", "show", "diff", "status", "grep", "branch", "worktree", "blame",
            "rev-parse", "rev-list", "ls-files", "cat-file", "describe",
        ) else "changing"
    if "cargo" in command:
        return "running tests"
    return "other"


def categories(response):
    """One label per tool call in a response."""
    out = []
    for call in response["tools"]:
        name = call.get("name")
        if name == "Bash":
            out.append(bash_kind(" ".join((call.get("input") or {}).get("command", "").split())))
        elif name in ("Read", "Grep", "Glob", "ToolSearch"):
            out.append("finding")
        elif name in ("Edit", "Write", "NotebookEdit"):
            out.append("changing")
        else:
            out.append("other")
    return out


def subject_of(path):
    """Which part of the tree a file belongs to."""
    path = str(path or "")
    base = os.path.basename(path)
    if is_decisions(path):
        return "docs/decisions.md"
    if base == "justfile" or "/hooks/" in path or base.endswith(".sh"):
        return "justfile + hooks"
    if "open-questions" in path or "handoff" in path:
        return "questions + handoff"
    if "/memory/" in path or base in ("MEMORY.md", "CLAUDE.md"):
        return "CLAUDE.md + memory"
    if "SKILL.md" in path or "/.claude/" in path or "/plugins/" in path:
        return "skills + plugins"
    if path.endswith(".md"):
        return "other docs"
    if "/tests/" in path or "fuzz" in path:
        return "tests + fuzz targets"
    if path.endswith(".rs"):
        return "kernel source"
    if path.endswith(".toml"):
        return "cargo manifests"
    return "unidentified"


def is_decisions(path):
    """The decisions corpus, whether it is one file or one file per section.

    §177 split it into `docs/decisions.d/NNN-slug.md`. The sections kept their
    numbers, so a reader still addresses §49 either way and the two shapes are
    one subject here; counting them apart would split every rate this file
    publishes across the date of the split.
    """
    return os.path.basename(path) == "decisions.md" or "decisions.d/" in path


# The rituals that recur, and the shape each leaves in a tool call. First match
# wins, so the order is the order of interest. Each is work that no kernel change
# needs, which is the whole reason for counting them separately.
#
# **What a call addresses and what it says are different things, and matching
# the two together was wrong.** The first version searched the whole tool input
# as one blob, so a handoff document that *listed* the open questions counted as
# question-file work — and since first match wins, it was taken off `handoff
# writing` as well. That put the question file at 570 responses and $74.80 in
# the week of 2026-09-12 against a true 445 and $53.68, and §144.5 reasoned from
# the inflated figure. So each ritual now says which it reads: `address` is the
# `file_path` of a file tool and the `command` of a shell call, and `content` is
# the bytes an Edit or Write carries. `docs/decisions.md` §152.
RITUALS = [
    ("question-file grammar",
     r"open-questions\.md$|question-check|just question\b", None),
    ("lane / worktree admin",
     r"worktree (lock|unlock|add|remove|list)|release-lanes"
     r"|just lanes|prune-worktrees|check-hub|clean-targets", None),
    ("section renumbering",
     r"renumber|decisions-toc|check-decisions", r"^## 1[0-9][0-9]\. "),
    ("gate babysitting",
     r"pgrep -f|until \[|until !|sleep \d+", None),
    ("merge-conflict surgery",
     r"--theirs|--ours|merge --abort|git checkout (main|HEAD) --",
     r"<<<<<<<|>>>>>>>"),
    ("handoff writing", r"handoff|rcad-handoff", None),
]

FILE_TOOLS = ("Read", "Edit", "Write", "NotebookEdit")


def ritual_of(tools):
    """Which ritual a response's tool calls belong to, or None.

    A response is counted once, under the first ritual any of its calls matches,
    because the unit of billing is the response and not the call.
    """
    addresses, contents = [], []
    for call in tools:
        inp = call.get("input") or {}
        name = call.get("name")
        if name in FILE_TOOLS:
            addresses.append(str(inp.get("file_path", "")))
            contents.extend(str(inp.get(field, "")) for field in
                            ("content", "old_string", "new_string"))
        elif name == "Bash":
            addresses.append(str(inp.get("command", "")))
        else:
            addresses.append(json.dumps(inp))
    address = "\n".join(addresses)
    content = "\n".join(contents)
    for name, address_pattern, content_pattern in RITUALS:
        if re.search(address_pattern, address, re.I | re.M):
            return name
        if content_pattern and re.search(content_pattern, content, re.I | re.M):
            return name
    return None


def is_lookup(response):
    """A response that only looked something up, and is not one of the rituals.

    The rituals are a session asking about its own state — `just lanes`, the
    question file, a renumber — and there is nobody to send for that answer.
    §144.5 owns them. What is left is the population a lookup lane is aimed at,
    and it is a fifth of what the word "finding" covers in `summary`.
    """
    if not response["tools"]:
        return False
    if not all(c == "finding" for c in categories(response)):
        return False
    return ritual_of(response["tools"]) is None


# The seven jobs §396.1 sorts a dispatcher's own work into, in that section's
# order. `report_start` means by this; `other` is both the last resort here
# and what a response with no tool call is counted as, since there is no call
# to sort.
PURPOSES = (
    "write the briefs",
    "check a row against the code",
    "queue",
    "a returned run",
    "slots and trees",
    "start",
    "other",
)


def purpose_of(call):
    """Which of `PURPOSES` one tool call's work was for. §396.1.

    A pure function of the call alone, the same shape as `ritual_of`: it reads
    only what the call addressed — a file path, a shell command, a tool name —
    never the response's prose, so a brief that *mentions* the queue in
    passing is not sorted as queue work.
    """
    name = call.get("name")
    inp = call.get("input") or {}
    if name in ("Workflow", "Agent"):
        return "write the briefs"
    if name in ("ToolSearch", "Skill"):
        return "start"
    if name in ("ListAgents", "EnterWorktree", "ExitWorktree", "SendMessage", "TaskStop"):
        return "slots and trees"
    if name in ("Read", "NotebookEdit", "Edit", "Write"):
        return _purpose_of_file(str(inp.get("file_path", "")))
    if name in ("Grep", "Glob"):
        return _purpose_of_file(str(inp.get("path") or inp.get("glob") or ""))
    if name == "Bash":
        return _purpose_of_command(" ".join(str(inp.get("command", "")).split()))
    return "other"


# A run's own record, wherever a dispatcher goes looking for it: a workflow's
# subagent files, the saved tool results beside them, its journal, or a
# transcript read directly rather than through `.claude/projects/<repo>/memory`
# or a handoff, which are `start`'s and checked first. §396.1's "run journals
# and transcripts".
RUN_RECORD = re.compile(r"/subagents/|/tool-results/|workflows/wf_|journal\.jsonl|\.jsonl$")


def _purpose_of_file(path):
    if ("handoff" in path or "/memory/" in path
            or os.path.basename(path) in ("MEMORY.md", "CLAUDE.md")):
        return "start"
    if RUN_RECORD.search(path):
        return "a returned run"
    if "lanes.js" in path:
        return "write the briefs"
    if is_decisions(path) or "/crates/" in path or path.startswith("crates/"):
        return "check a row against the code"
    if os.path.basename(path) == "queue.md" or "bin/queue" in path:
        return "queue"
    return "other"


def _purpose_of_command(command):
    words = command.split()
    head = words[0] if words else ""
    if head in ("mise", "env", "timeout", "nohup"):
        for word in words[1:]:
            if word not in ("exec", "--") and not word.startswith("-") and "=" not in word:
                head = word
                break
    if RUN_RECORD.search(command):
        return "a returned run"
    if re.search(r"just recipe\b", command):
        return "write the briefs"
    if re.search(r"just (decisions?|src|claim-section)\b", command):
        return "check a row against the code"
    if re.search(r"just queue\b", command) or "queue.md" in command or "bin/queue" in command:
        return "queue"
    if re.search(r"just ff-main\b", command):
        return "a returned run"
    if re.search(r"worktree (lock|unlock|add|remove|list)|release-lanes|just (lanes|status)"
                 r"|prune-worktrees|check-hub|clean-targets", command):
        return "slots and trees"
    if head == "git":
        if re.search(r"\b(lane-\S*|wf_\S*)", command):
            return "a returned run"
        if re.search(r"\b(fetch|rev-parse|merge-base)\b", command) or "--ff-only" in command:
            return "slots and trees"
        if re.search(r"crates/|docs/decisions", command):
            return "check a row against the code"
        return "other"
    if "crates/" in command and head in ("grep", "rg", "cat", "sed", "head", "tail", "find"):
        return "check a row against the code"
    return "other"


# ------------------------------------------------------------------- reports

def banner(responses, window):
    total = sum(r["cost"] for r in responses)
    lanes = [r for r in responses if r["lane"]]
    print(f"window {window[0].date()} .. {window[1].date()}")
    print(f"${total:,.2f} over {len(responses):,} API responses "
          f"({len(responses) - len(lanes):,} in main sessions, {len(lanes):,} in lanes)")
    print()
    return total


def report_summary(responses, total):
    print("=== where the money goes by token class ===")
    parts = collections.Counter()
    for r in responses:
        for name, amount in r["parts"].items():
            parts[name] += amount
    for name, amount in parts.most_common():
        note = "   (a lower bound — not every response logs a final count)" if name == "output" else ""
        print(f"  {name:12} ${amount:9,.2f} {100 * amount / total:6.1f}%{note}")
    print()

    print("=== batching ===")
    spread = collections.Counter(len(r["tools"]) for r in responses)
    using = sum(n for k, n in spread.items() if k >= 1)
    many = sum(n for k, n in spread.items() if k > 1)
    for k in sorted(spread):
        print(f"  {k:2d} tool(s): {spread[k]:6,d} responses {100 * spread[k] / len(responses):5.1f}%")
    print(f"  of responses that called a tool, {100 * many / max(using, 1):.1f}% called more than one")
    print()

    print("=== context when a response was billed ===")
    sizes = sorted(r["context"] for r in responses if r["context"])
    if sizes:
        at = lambda p: sizes[int(p * (len(sizes) - 1))]
        print(f"  median {at(.5):,}   p75 {at(.75):,}   p90 {at(.9):,}   max {sizes[-1]:,}")
    for low, high in [(0, 50_000), (50_000, 100_000), (100_000, 150_000),
                      (150_000, 200_000), (200_000, 300_000), (300_000, 10 ** 9)]:
        chunk = [r for r in responses if low <= r["context"] < high]
        spend = sum(r["cost"] for r in chunk)
        label = f"{low // 1000:3d}k-{min(high, 10 ** 6) // 1000:4d}k"
        print(f"  ctx {label}: {len(chunk):6,d} responses ${spend:9,.2f} {100 * spend / total:5.1f}%")
    print()

    print("=== finding, and the probe that follows a probe ===")
    found = sum(r["cost"] for r in responses
                if r["tools"] and all(c == "finding" for c in categories(r)))
    print(f"  responses that only looked things up: ${found:,.2f} = {100 * found / total:.1f}%")
    by_session = collections.defaultdict(list)
    for r in responses:
        by_session[(r["session"], r["lane"])].append(r)
    tax, n = 0.0, 0
    for chunk in by_session.values():
        previous = False
        for r in chunk:
            now = bool(r["tools"]) and all(c == "finding" for c in categories(r))
            if now and previous:
                tax += r["cost"]
                n += 1
            previous = now
    print(f"  of those, {n:,} directly followed another: ${tax:,.2f} = {100 * tax / total:.1f}%")
    print("  `finding` takes this apart: most of it is nobody's to delegate")
    print()


def report_finding(responses, results, total):
    """Looking things up: what could be handed to a lane, and when that pays.

    `summary` prints one number for finding, and the number is misread: it
    counts every probe, wherever it was made and whatever it was about. This
    report separates the part a lookup lane could take from the part it could
    not, then prices the lane against the hunts it would replace. §158.
    """
    print("=== looking things up: the part a lookup lane could take ===")
    main = [r for r in responses if not r["lane"]]
    inside = [r for r in responses if r["lane"]]

    def probe_price(chunk):
        probes = [r["cost"] for r in chunk if r["tools"]
                  and all(c == "finding" for c in categories(r))]
        return sum(probes), (sorted(probes)[len(probes) // 2] if probes else 0.0)

    outer, outer_each = probe_price(main)
    inner, inner_each = probe_price(inside)
    print(f"  in main sessions      ${outer:9,.2f} {100 * outer / total:5.1f}%"
          f"   ${outer_each:.3f} a response")
    print(f"  inside lanes          ${inner:9,.2f} {100 * inner / total:5.1f}%"
          f"   ${inner_each:.3f} a response — already the cheap kind, and a lane"
          f" has nowhere to send it")
    ritual = sum(r["cost"] for r in main if r["tools"]
                 and all(c == "finding" for c in categories(r))
                 and ritual_of(r["tools"]))
    real = sum(r["cost"] for r in main if is_lookup(r))
    print(f"  of the main-session part, ritual ${ritual:,.2f} "
          f"({100 * ritual / total:.1f}%) — §144.5's, and no lane can be sent for it")
    print(f"  LEFT TO DELEGATE      ${real:9,.2f} {100 * real / total:5.1f}%")
    print()

    runs = finding_runs(responses)
    if not runs:
        print("  no lookups in this window")
        return
    print("=== how a hunt runs: back-to-back lookups in one main session ===")
    spread, priced = collections.Counter(), collections.Counter()
    for run in runs:
        length = min(len(run), 10)
        spread[length] += 1
        priced[length] += sum(r["cost"] for r in run)
    for length in sorted(spread):
        label = f"{length}" if length < 10 else "10+"
        print(f"  run of {label:>3}: {spread[length]:5,d} runs ${priced[length]:8,.2f} "
              f"{100 * priced[length] / total:5.1f}%")
    long_ctx = sorted(run[0]["context"] for run in runs if len(run) >= 3)
    short_ctx = sorted(run[0]["context"] for run in runs if len(run) < 3)
    if long_ctx and short_ctx:
        print(f"  a run of 3+ starts at context {long_ctx[len(long_ctx) // 2]:,} at the median")
        print(f"  a run of 1-2 starts at        {short_ctx[len(short_ctx) // 2]:,}")
        print("  — the longer hunt starts LOWER, so a context threshold selects"
              " the wrong probes")
    print()

    price = lookup_lane_price(responses, results)
    print("=== what a lookup lane costs, matched to its dispatch by agent id ===")
    if not price["lanes"]:
        print("  none dispatched in this window")
        return
    print(f"  {price['lanes']} dispatched, {price['matched']} of them with a transcript"
          f" in this window")
    print(f"  the dispatching response  ${price['dispatch']:.3f}"
          f"   billed at the asking session's context, like any probe")
    print(f"  the lane itself           ${price['lane']:.3f}"
          f"   over {price['responses']} responses")
    print(f"  carrying what came back   ${price['carry']:.3f}")
    print(f"  ALL IN                    ${price['all_in']:.3f}")
    hunt = sorted(sum(r["cost"] for r in run) for run in runs if len(run) >= 3)
    if hunt:
        print(f"  a hunt of 3+ costs        ${hunt[len(hunt) // 2]:.3f} at the median")
    print()

    print("=== when handing the rest of a hunt to a lane pays ===")
    print(f"  {'trigger':>8} {'runs':>6} {'what follows':>13} {'lanes':>9} {'net':>9} {'% week':>8}")
    for after in range(1, 7):
        hit, net = delegation_net(runs, after, price["all_in"])
        follows = sum(r["cost"] for run in runs if len(run) > after for r in run[after:])
        print(f"  past {after:2d} {hit:8,d} ${follows:12,.2f} ${hit * price['all_in']:8,.2f} "
              f"${net:8,.2f} {100 * net / total:7.2f}%")
    print("  and the same hunts delegated at their FIRST probe, which needs foresight:")
    for least in (2, 3, 4, 5):
        hit = [run for run in runs if len(run) >= least]
        net = sum(r["cost"] for run in hit for r in run) - len(hit) * price["all_in"]
        print(f"    hunts of {least}+: {len(hit):5,d} runs, net ${net:8,.2f} "
              f"{100 * net / total:6.2f}% of the week")
    print("  the lane's own price is the dial, so the same hunts of 3+ at a cheaper lane:")
    hit = [run for run in runs if len(run) >= 3]
    spent = sum(r["cost"] for run in hit for r in run)
    for part, label in ((1.0, "as measured"), (0.5, "half"), (0.25, "a quarter")):
        each = price["dispatch"] + part * (price["lane"] + price["carry"])
        net = spent - len(hit) * each
        print(f"    lane at {label:12} (${each:.3f} all in): net ${net:8,.2f} "
              f"{100 * net / total:6.2f}% of the week")
    print()


def finding_runs(responses):
    """Maximal stretches of back-to-back lookups inside one main session.

    A hunt is what a lane replaces, so a hunt is what has to be counted — not
    single responses, which is all `summary` can see. Anything that is not a
    lookup ends the run, a ritual call included: asking `just lanes` in the
    middle of a hunt is a different question, not a continuation of this one.
    """
    by_session = collections.defaultdict(list)
    for r in responses:
        if not r["lane"]:
            by_session[r["session"]].append(r)
    runs = []
    for chunk in by_session.values():
        current = []
        for r in chunk:
            if is_lookup(r):
                current.append(r)
            elif current:
                runs.append(current)
                current = []
        if current:
            runs.append(current)
    return runs


def delegation_net(runs, after, lane_price):
    """What handing the rest of a hunt to a lane saves, once the lane is paid for.

    Delegating after N probes means those N happen anyway and everything from
    the N+1th onward is replaced by one dispatch and a lane. So what is saved is
    the tail of the run, and what is added is the whole `all_in` price — the
    dispatch is a response in the asking session like the probe it replaces, and
    is priced as one. Only a run longer than N has a tail to hand over.
    """
    hit = [run for run in runs if len(run) > after]
    saved = sum(r["cost"] for run in hit for r in run[after:])
    return len(hit), saved - len(hit) * lane_price


def lookup_lane_price(responses, results):
    """The three prices that make up one dispatched lookup lane, measured.

    The launch receipt names the lane's agent id and the lane's transcript is
    the file named for that id, so the pairing is exact rather than a guess
    from the brief's first line. The three parts are the dispatching response,
    the lane's own responses, and carrying the receipt and the answer for the
    rest of the session that asked. Medians, because two of the three are
    long-tailed and the mean is one runaway lane away from being wrong.
    """
    by_session = collections.defaultdict(list)
    for r in responses:
        if not r["lane"]:
            by_session[r["session"]].append(r)

    dispatch, carry, agents = [], [], {}
    for r in responses:
        for call in r["tools"]:
            if call.get("name") not in ("Agent", "Task"):
                continue
            if (call.get("input") or {}).get("subagent_type") != "lookup":
                continue
            back = results.get(call.get("id"), "")
            named = re.search(r"agentId: (\w+)", back)
            if not named:
                continue
            agents[named.group(1)] = r
            dispatch.append(r["cost"])
            after = [x for x in by_session[r["session"]] if x["when"] > r["when"]]
            carry.append(len(back) / CHARS_PER_TOKEN
                         * sum(x["read_rate"] for x in after) / 1e6)

    lane_cost, lane_n = collections.Counter(), collections.Counter()
    for r in responses:
        if not r["lane"]:
            continue
        base = os.path.basename(r["path"])
        for agent in agents:
            if agent in base:
                lane_cost[agent] += r["cost"]
                lane_n[agent] += 1

    def middle(values, fallback=0.0):
        values = sorted(values)
        return values[len(values) // 2] if values else fallback

    price = {
        "lanes": len(agents),
        "matched": sum(1 for agent in agents if lane_cost[agent]),
        "dispatch": middle(dispatch),
        "lane": middle(lane_cost.values()),
        "responses": middle(lane_n.values()),
        "carry": middle(carry),
    }
    price["all_in"] = price["dispatch"] + price["lane"] + price["carry"]
    return price


def report_classes(responses, total):
    print("=== what the money was spent ON ===")
    print("    a response is attributed to the file it wrote, or failing that")
    print("    to the file it was looking at")
    print()
    wrote, wrote_n = collections.Counter(), collections.Counter()
    looked, looked_n = collections.Counter(), collections.Counter()
    unknown = 0.0
    for r in responses:
        writes, probes = [], []
        for call in r["tools"]:
            name, args = call.get("name"), call.get("input") or {}
            if name in ("Edit", "Write", "NotebookEdit"):
                writes.append(subject_of(args.get("file_path")))
            elif name == "Read":
                probes.append(subject_of(args.get("file_path")))
            elif name in ("Grep", "Glob"):
                probes.append(subject_of(args.get("path") or args.get("glob") or ""))
            elif name == "Bash":
                command = args.get("command", "")
                hit = re.search(r"[\w./-]*(?:decisions\.md|justfile|open-questions\.md"
                                r"|\.rs\b|\.toml\b|\.md\b)", command)
                if hit:
                    probes.append(subject_of(hit.group(0)))
                if re.search(r"just (gate|test|t\b)|nextest|cargo", command):
                    probes.append("running tests")
        if writes:
            share = r["cost"] / len(writes)
            for w in writes:
                wrote[w] += share
                wrote_n[w] += 1
        elif probes:
            share = r["cost"] / len(probes)
            for p in probes:
                looked[p] += share
                looked_n[p] += 1
        else:
            unknown += r["cost"]

    changing = sum(wrote.values())
    print(f"  -- writing files (${changing:,.2f}, {100 * changing / total:.1f}% of the bill) --")
    for name, amount in wrote.most_common():
        print(f"    {name:22} {wrote_n[name]:5,d} writes ${amount:8,.2f} {100 * amount / total:5.1f}%")
    print()
    print(f"  -- looking, without writing (${sum(looked.values()):,.2f}) --")
    for name, amount in looked.most_common():
        print(f"    {name:22} {looked_n[name]:5,d} probes ${amount:8,.2f} {100 * amount / total:5.1f}%")
    print(f"    {'(no file named)':22} {'':5} {'':7} ${unknown:8,.2f} {100 * unknown / total:5.1f}%")
    print()

    kernel = {"kernel source", "tests + fuzz targets", "running tests", "cargo manifests"}
    paper = {"docs/decisions.md", "questions + handoff", "justfile + hooks",
             "CLAUDE.md + memory", "skills + plugins", "other docs"}
    k = sum(v for n, v in list(wrote.items()) + list(looked.items()) if n in kernel)
    p = sum(v for n, v in list(wrote.items()) + list(looked.items()) if n in paper)
    print("=== the kernel, against the paperwork around it ===")
    print(f"  kernel, tests, running them  ${k:9,.2f} {100 * k / total:6.1f}%")
    print(f"  decisions/questions/justfile ${p:9,.2f} {100 * p / total:6.1f}%")
    print(f"  unattributed                 ${total - k - p:9,.2f} {100 * (total - k - p) / total:6.1f}%")
    print()


def report_ritual(responses, total):
    print("=== recurring ritual: work no kernel change needs ===")
    spend, count = collections.Counter(), collections.Counter()
    for r in responses:
        name = ritual_of(r["tools"])
        if name:
            spend[name] += r["cost"]
            count[name] += 1
    print(f"  {'ritual':26} {'responses':>10} {'cost':>10} {'share':>7}")
    for name, _, _ in RITUALS:
        print(f"  {name:26} {count[name]:10,d} ${spend[name]:9,.2f} {100 * spend[name] / total:6.1f}%")
    every = sum(spend.values())
    print(f"  {'TOTAL':26} {sum(count.values()):10,d} ${every:9,.2f} {100 * every / total:6.1f}%")
    print()


def _sends_lane(call):
    """A `Workflow` call, or the old `Agent` dispatch of a general-purpose lane.

    §396.1: of the 69 segments that sent a lane between 2026-09-25 and
    2026-10-03, the nine newest end at a `Workflow` call on `lanes.js`; of the
    60 older ones, 20 end at an `Agent` call instead, 17 of them
    general-purpose. A session's own lookup or verifier `Agent` call is
    neither — it asks a question or checks a run, and sends no lane.
    """
    name = call.get("name")
    if name == "Workflow":
        return True
    return name == "Agent" and (call.get("input") or {}).get("subagent_type") == "general-purpose"


def report_start(responses, results, total):
    """What a dispatcher spends before it sends its first lane, and on what.

    §396.1's own counting rules, run instead of by hand: group by session,
    find the first response that dispatches a lane (`_sends_lane`), and look
    at everything up to and including it. A session that never makes that
    call sent no lane and is not one of these; everyone else in the window is
    a lookup, a kernel session or a lane's own transcript.
    """
    print("=== a dispatcher's start: context and output before its first lane ===")
    by_session = collections.defaultdict(list)
    for r in responses:
        if not r["lane"]:
            by_session[r["session"]].append(r)

    before_ctx, lane_ctx, between_out = [], [], []
    read_ranks = collections.defaultdict(list)
    out_before = {p: [] for p in PURPOSES}
    chars_before = {p: [] for p in PURPOSES}
    out_whole = {p: [] for p in PURPOSES}
    chars_whole = {p: [] for p in PURPOSES}

    for chunk in by_session.values():
        lane_idx = next((i for i, r in enumerate(chunk)
                          if any(_sends_lane(c) for c in r["tools"])), None)
        if lane_idx is None:
            continue

        before_ctx.append(chunk[0]["context"])
        lane_ctx.append(chunk[lane_idx]["context"])
        between_out.append(sum(r["output"] for r in chunk[: lane_idx + 1]))

        reads = sorted(
            (len(results.get(call.get("id"), "")) for r in chunk[:lane_idx]
             for call in r["tools"] if call.get("name") == "Read"),
            reverse=True,
        )
        for rank, size in enumerate(reads[:5]):
            read_ranks[rank].append(size)

        session_out = [{p: 0.0 for p in PURPOSES} for _ in range(2)]
        session_chars = [{p: 0.0 for p in PURPOSES} for _ in range(2)]
        for i, r in enumerate(chunk):
            tools = r["tools"]
            purposes = [purpose_of(c) for c in tools] if tools else ["other"]
            share = r["output"] / len(purposes)
            halves = (0, 1) if i <= lane_idx else (1,)
            for half in halves:
                for purpose in purposes:
                    session_out[half][purpose] += share
                for call, purpose in zip(tools, purposes):
                    session_chars[half][purpose] += len(results.get(call.get("id"), ""))
        for p in PURPOSES:
            out_before[p].append(session_out[0][p])
            chars_before[p].append(session_chars[0][p])
            out_whole[p].append(session_out[1][p])
            chars_whole[p].append(session_chars[1][p])

    if not before_ctx:
        print("  no session in this window sent a lane")
        print()
        return

    def median(values):
        values = sorted(values)
        return values[len(values) // 2]

    def mean(values):
        return sum(values) / len(values) if values else 0.0

    print(f"  {len(before_ctx)} sessions sent a lane")
    print(f"  context before the first word               median {median(before_ctx):>8,.0f}")
    print(f"  context at the first Workflow/Agent call     median {median(lane_ctx):>8,.0f}")
    print(f"  output tokens in between                     median {median(between_out):>8,.0f}")
    reads = "  ".join(f"#{rank + 1} {median(read_ranks[rank]):,.0f}"
                       for rank in sorted(read_ranks))
    print(f"  five biggest reads before that call, median by rank: {reads}")
    print()

    print(f"  {'what for':30} {'out before':>11} {'chars before':>13}"
          f" {'out whole':>10} {'chars whole':>12}")
    totals = [0.0, 0.0, 0.0, 0.0]
    for p in PURPOSES:
        row = (mean(out_before[p]), mean(chars_before[p]), mean(out_whole[p]), mean(chars_whole[p]))
        totals = [t + v for t, v in zip(totals, row)]
        print(f"  {p:30} {row[0]:11,.0f} {row[1]:13,.0f} {row[2]:10,.0f} {row[3]:12,.0f}")
    print(f"  {'all':30} {totals[0]:11,.0f} {totals[1]:13,.0f} {totals[2]:10,.0f} {totals[3]:12,.0f}")
    print()


def report_sessions(responses, total):
    print("=== sessions, dearest first ===")
    per = collections.defaultdict(
        lambda: {"cost": 0.0, "n": 0, "first": None, "last": None, "ctx": 0, "cwd": ""}
    )
    for r in responses:
        if r["lane"]:
            continue
        s = per[r["session"]]
        s["cost"] += r["cost"]
        s["n"] += 1
        s["first"] = min(s["first"] or r["when"], r["when"])
        s["last"] = max(s["last"] or r["when"], r["when"])
        s["ctx"] = max(s["ctx"], r["context"])
        s["cwd"] = os.path.basename(r["cwd"]) or s["cwd"]
    print(f"  {len(per)} main sessions")
    fat = [s for s in per.values() if s["ctx"] > 200_000]
    if fat:
        spend = sum(s["cost"] for s in fat)
        print(f"  {len(fat)} passed 200k of context, and they are "
              f"${spend:,.2f} = {100 * spend / total:.0f}% of the whole bill")
    print()
    for name, s in sorted(per.items(), key=lambda kv: -kv[1]["cost"])[:25]:
        hours = (s["last"] - s["first"]).total_seconds() / 3600
        print(f"  {name} ${s['cost']:7,.2f} {s['n']:4d} resp {hours:5.1f}h "
              f"ctx {s['ctx'] // 1000:4d}k  {s['first']:%m-%d %H:%M}  {s['cwd']}")
    print()


def report_carry(responses, results, total):
    print("=== what fills the context that every later response pays to re-read ===")
    print("    carry = result tokens x responses that followed it, in the same session")
    print()
    by_session = collections.defaultdict(list)
    for r in responses:
        by_session[(r["session"], r["lane"])].append(r)
    size, calls, carry = collections.Counter(), collections.Counter(), collections.Counter()
    for chunk in by_session.values():
        for i, r in enumerate(chunk):
            after = len(chunk) - i - 1
            for call in r["tools"]:
                name = call.get("name")
                if name == "Bash":
                    name = "Bash: " + bash_kind(
                        " ".join((call.get("input") or {}).get("command", "").split())
                    )
                text = results.get(call.get("id"))
                if text is None:
                    continue
                n = len(text)
                size[name] += n
                calls[name] += 1
                carry[name] += n * after
    every = sum(carry.values()) or 1
    print(f"  {'tool':24} {'calls':>7} {'MB back':>9} {'avg tok':>9} {'carry':>8}")
    for name, amount in carry.most_common(16):
        print(f"  {name:24} {calls[name]:7,d} {size[name] / 1e6:9.1f} "
              f"{size[name] / calls[name] / CHARS_PER_TOKEN:9,.0f} {100 * amount / every:7.1f}%")
    print()


def report_ceiling(responses, total):
    print("=== what a context ceiling would have saved ===")
    print("    cache read billed at min(context, cap), everything else unchanged.")
    print("    An upper bound: the handed-off work still happens, and the session")
    print("    that inherits it re-reads some of what it was handed.")
    print()
    base = sum(r["read_tokens"] * r["read_rate"] for r in responses) / 1e6
    for cap in (100_000, 125_000, 150_000, 175_000, 200_000):
        capped = sum(min(r["read_tokens"], cap) * r["read_rate"] for r in responses) / 1e6
        print(f"  cap {cap // 1000:3d}k: cache read ${capped:8,.2f} of ${base:8,.2f}"
              f"   saves ${base - capped:7,.2f} = {100 * (base - capped) / total:4.1f}% of the bill")
    print()


def report_warnings(responses, warnings, total):
    print("=== the context budget warning, and what is billed past it ===")
    print("    each session is measured against the budget the hook itself named,")
    print("    so a window may straddle a change to the dial. Where no warning")
    print("    named one, the budget is the nearest warning's, in time.")
    print()
    started = {}
    for r in responses:
        if not r["lane"]:
            started[r["session"]] = min(started.get(r["session"], r["when"]), r["when"])
    said = {s: sorted(ws, key=lambda w: w["when"])
            for s, ws in warnings.items() if s in started}
    dial = sorted((w["when"], w["budget"]) for ws in said.values() for w in ws
                  if w["budget"])
    if not dial:
        print("  no warning in this window that named a budget: either nothing "
              "crossed, or the hook was not running\n")
        return
    budgets = collections.Counter(b for _, b in dial)
    usual = budgets.most_common(1)[0][0]

    def in_force(when):
        """The budget the hook was holding then, read off the warnings around it.

        Two sessions out of three are measured this way: a session nothing ever
        warned has no figure of its own, and neither does one whose transcript
        recorded only the hook's one-line summary. The nearest warning before it
        is the only witness in the transcripts; before the first warning of all,
        that witness is the first one.
        """
        seen = [b for at, b in dial if at <= when]
        return seen[-1] if seen else dial[0][1]

    first = {s: ws[0]["when"] for s, ws in said.items()}
    budget_of = {s: ws[0]["budget"] or in_force(ws[0]["when"]) for s, ws in said.items()}
    spend = collections.Counter()
    after = collections.Counter()
    sessions = collections.defaultdict(set)
    for r in responses:
        if r["lane"]:
            continue
        budget = budget_of.get(r["session"]) or in_force(started[r["session"]])
        bucket = bucket_of(r["context"], r["when"], budget, first.get(r["session"]))
        spend[bucket] += r["cost"]
        sessions[bucket].add(r["session"])
        if bucket == "after the warning":
            after[r["session"]] += 1
    main = sum(spend.values())

    print(f"  main sessions            ${main:8,.2f}   budget {usual // 1000}k"
          f"{' (and ' + ', '.join(f'{b // 1000}k' for b in sorted(budgets) if b != usual) + ' earlier in the window)' if len(budgets) > 1 else ''}")
    for bucket in ("under the budget", "before the warning", "after the warning",
                   "never warned"):
        count = len(sessions[bucket])
        print(f"    {bucket:<20} ${spend[bucket]:8,.2f}  {100 * spend[bucket] / main:5.1f}%"
              f"   {count:3d} session{'' if count == 1 else 's'}")
    print("      'before the warning' is billed inside the turn that crossed the")
    print("      budget: no Stop had run yet, so nothing could fire.")
    print()

    over = sorted(w["tokens"] - budget_of[s] for s, ws in said.items()
                  for w in ws if not w["repeat"])
    repeats = sum(1 for ws in said.values() for w in ws if w["repeat"])
    if over:
        print(f"  {len(over)} first warnings: median {over[len(over) // 2]:,} tokens past "
              f"the budget, worst {over[-1]:,}")
    tails = sorted(after.values())
    if tails:
        print(f"  {len(tails)} sessions kept going afterwards: median {tails[len(tails) // 2]} "
              f"further responses, worst {tails[-1]}")
    print(f"  {repeats} repeat warnings")
    print()


def report_pairs(responses, results, total):
    """Search a file, then read it: the two-call shape §144.6 objected to.

    A pair is two back-to-back responses in one session where the first
    searches a file and the second reads the same one. Only the second is the
    shape's cost — the search was needed anyway — so that is what is priced.

    `docs/decisions.md` gets a block of its own, because it has a one-call
    reader and a search that answers in section numbers, and §153.5 asked
    whether the second call there is a tool problem or a wording one.
    """
    print("=== search a file, then read it in the next response ===")
    by_session = collections.defaultdict(list)
    for r in responses:
        by_session[(r["session"], r["lane"])].append(r)

    pairs, spend = collections.Counter(), collections.Counter()
    by_searcher, by_searcher_spend = collections.Counter(), collections.Counter()
    finds = []
    greps = {"after a find": 0, "no find before it": 0}
    for chunk in by_session.values():
        found_before = False
        for i, r in enumerate(chunk):
            touched = [t for call in r["tools"] for t in targets_of(call)]
            following = chunk[i + 1] if i + 1 < len(chunk) else None
            read_next = set()
            if following:
                read_next = {f for call in following["tools"]
                             for verb, f in targets_of(call) if verb == "read"}
            for f in {f for verb, f in touched if verb == "search"} & read_next:
                pairs[f] += 1
                spend[f] += following["cost"]
                if f == DECISIONS:
                    how = "find" if any(is_find(c) for c in r["tools"]) else "grep"
                    by_searcher[how] += 1
                    by_searcher_spend[how] += following["cost"]

            for call in r["tools"]:
                command = " ".join((call.get("input") or {}).get("command", "").split())
                if call.get("name") == "Bash" and re.search(r"just decisions-find\b", command):
                    finds.append((found_hits(results.get(call.get("id"), "")),
                                  decision_read(following)))
                    found_before = True
                elif any(verb == "search" and f == DECISIONS and not is_find(call)
                         for verb, f in targets_of(call)):
                    greps["after a find" if found_before else "no find before it"] += 1

    every = sum(spend.values())
    print(f"  {sum(pairs.values()):,} pairs; the second call of each cost ${every:,.2f}"
          f" = {100 * every / total:.1f}% of the bill")
    print(f"  {'file':44} {'pairs':>6} {'second call':>12}")
    for f, n in pairs.most_common(12):
        print(f"  {f:44} {n:6,d} ${spend[f]:11,.2f}")
    print()

    print(f"  {DECISIONS}")
    for how, label in (("find", "a decisions-find"), ("grep", "a grep")):
        print(f"    pairs opened by {label:16} {by_searcher[how]:5,d}   "
              f"second call ${by_searcher_spend[how]:,.2f}")
    followed = [(hits, read) for hits, read in finds if read is not None]
    empty = sum(1 for hits, _ in finds if not hits)
    print(f"    {len(finds):,} decisions-find calls, {empty:,} of them matching nothing; "
          f"{len(followed):,} followed straight by `just decision`, which read")
    shape = collections.Counter((hits_bucket(len(hits)), read in hits) for hits, read in followed)
    print(f"      {'the find named':22} {'a section it named':>19} {'another one':>12}")
    for size in ("nothing", "1 section", "2-3 sections", "4 or more"):
        print(f"      {size:22} {shape[(size, True)]:19,d} {shape[(size, False)]:12,d}")
    print(f"    {sum(greps.values()):,} greps of it that were not decisions-find: "
          f"{greps['no find before it']:,} in a session that had not tried "
          f"decisions-find yet, {greps['after a find']:,} after it had")
    print()


DECISIONS = "docs/decisions.md"


def hits_bucket(size):
    if size == 0:
        return "nothing"
    return "1 section" if size == 1 else "2-3 sections" if size <= 3 else "4 or more"


SEARCHERS = ("grep", "rg", "egrep")
READERS = ("sed", "awk", "head", "tail", "cat")


def targets_of(call):
    """What files one tool call searched and read, as (verb, file) pairs.

    Only a file named in the call counts. A grep across a directory searched
    no particular file, so it can be the first half of no pair.
    """
    inp = call.get("input") or {}
    name = call.get("name")
    if name == "Read":
        return [("read", file_key(inp.get("file_path")))]
    if name == "Grep":
        path = inp.get("path") or ""
        return [("search", file_key(path))] if looks_like_file(path) else []
    if name != "Bash":
        return []
    out = []
    for segment in re.split(r"\|\||&&|[|;\n]", inp.get("command", "")):
        words = shell_words(segment)
        while words and words[0] in ("mise", "exec", "--", "timeout", "env"):
            words = words[1:]
        if not words:
            continue
        head, rest = os.path.basename(words[0]), words[1:]
        if head == "just" and rest:
            if rest[0] in ("decisions-find", "decisions-toc"):
                out.append(("search", DECISIONS))
            elif rest[0] == "decision":
                out.append(("read", DECISIONS))
            elif rest[0] == "recipe":
                out.append(("read", "justfile"))
            continue
        if head not in SEARCHERS + READERS:
            continue
        verb = "search" if head in SEARCHERS else "read"
        operands = [w for w in rest if not w.startswith("-")]
        if head in SEARCHERS + ("sed", "awk") and "-e" not in rest:
            operands = operands[1:]
        out.extend((verb, file_key(w)) for w in operands if looks_like_file(w))
    return out


def is_find(call):
    command = (call.get("input") or {}).get("command", "")
    return bool(re.search(r"just decisions-(find|toc)\b", command))


def shell_words(segment):
    try:
        return shlex.split(segment)
    except ValueError:
        return segment.split()


def looks_like_file(word):
    base = os.path.basename(str(word).rstrip("/"))
    return base == "justfile" or bool(re.search(r"\.[A-Za-z]{1,5}$", base))


def file_key(path):
    """One name per file, whichever worktree or spelling reached it.

    A section file counts as the whole corpus, so a `decisions-find` and the
    `just decision` that follows it still pair: the recipe reads one file per
    section now, and the find that opened it names no file at all.
    """
    path = str(path or "")
    base = os.path.basename(path)
    if is_decisions(path):
        return DECISIONS
    if base in ("justfile", "CLAUDE.md"):
        return base
    return "/".join(path.split("/")[-2:])


FOUND_ROW = re.compile(r"^\s+(\S+)\s.*\s(\d+) hits\s*$", re.M)


def found_hits(text):
    """The section ids one decisions-find printed, in order."""
    return [m.group(1) for m in FOUND_ROW.finditer(text or "")]


def decision_read(response):
    """The section a response read with `just decision`, or None."""
    for call in (response or {}).get("tools", []):
        command = (call.get("input") or {}).get("command", "")
        m = re.search(r"just decision\s+'?([\d.]+)", command)
        if call.get("name") == "Bash" and m:
            return m.group(1)
    return None


REPORTS = {
    "summary": lambda rs, res, ws, t: report_summary(rs, t),
    "finding": lambda rs, res, ws, t: report_finding(rs, res, t),
    "classes": lambda rs, res, ws, t: report_classes(rs, t),
    "ritual": lambda rs, res, ws, t: report_ritual(rs, t),
    "start": lambda rs, res, ws, t: report_start(rs, res, t),
    "sessions": lambda rs, res, ws, t: report_sessions(rs, t),
    "carry": lambda rs, res, ws, t: report_carry(rs, res, t),
    "ceiling": lambda rs, res, ws, t: report_ceiling(rs, t),
    "warnings": lambda rs, res, ws, t: report_warnings(rs, ws, t),
    "pairs": lambda rs, res, ws, t: report_pairs(rs, res, t),
}


def main(argv):
    if len(argv) < 5:
        print(__doc__)
        print(f"usage: spend.py <transcript-root> <repo-key> <since> <until> "
              f"[{'|'.join(REPORTS)}|all] [lane-substring]", file=sys.stderr)
        return 2
    root, repo, since, until = argv[1:5]
    which = argv[5] if len(argv) > 5 else "all"
    only = argv[6] if len(argv) > 6 else None

    window = (
        datetime.datetime.fromisoformat(since).replace(tzinfo=datetime.timezone.utc),
        datetime.datetime.fromisoformat(until).replace(tzinfo=datetime.timezone.utc),
    )
    responses, results, warnings = load(root, repo, window[0], window[1], only)
    if not responses:
        print("no transcripts in that window", file=sys.stderr)
        return 1
    if only:
        print(f"lane filter: {only}")
    total = banner(responses, window)

    wanted = list(REPORTS) if which == "all" else [which]
    for name in wanted:
        if name not in REPORTS:
            print(f"no report named {name!r}; have {', '.join(REPORTS)}", file=sys.stderr)
            return 2
        REPORTS[name](responses, results, warnings, total)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
