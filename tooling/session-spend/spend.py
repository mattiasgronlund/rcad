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

Almost everything is cache read: each response re-sends the conversation, so a
tool result of N tokens is paid once as a cache write and again on every later
response in that session. That is why `classes` reports "carry" — result tokens
times the responses that followed — rather than bytes returned. Bytes returned
is what a result costs once; carry is what it costs in total.
"""

import collections
import datetime
import glob
import json
import os
import re
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

    Returns (responses, results). A response carries the tools it called, so a
    report can attribute its cost; `results` maps a tool_use id to what came
    back, for the reports that care how large an answer was.
    """
    responses = []
    results = {}
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
            if '"assistant"' in line:
                record = _parse(line)
                if not record or record.get("type") != "assistant":
                    continue
                message = record.get("message", {})
                key = (record.get("requestId"), message.get("id"))
                if key not in grouped:
                    grouped[key] = {"record": record, "content": []}
                    order.append(key)
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
    return responses, results


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
    if "decisions.md" in path:
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
        print(f"  {name:12} ${amount:9,.2f} {100 * amount / total:6.1f}%")
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


REPORTS = {
    "summary": lambda rs, res, t: report_summary(rs, t),
    "finding": report_finding,
    "classes": lambda rs, res, t: report_classes(rs, t),
    "ritual": lambda rs, res, t: report_ritual(rs, t),
    "sessions": lambda rs, res, t: report_sessions(rs, t),
    "carry": lambda rs, res, t: report_carry(rs, res, t),
    "ceiling": lambda rs, res, t: report_ceiling(rs, t),
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
    responses, results = load(root, repo, window[0], window[1], only)
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
        REPORTS[name](responses, results, total)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
