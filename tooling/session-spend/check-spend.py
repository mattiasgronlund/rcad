#!/usr/bin/env python3
"""The classifiers the reports reason from, against their cases.

`spend.py` reads transcripts and so cannot be a `check-` on its own: its answer
moves with what the sessions did this week and there is no fixed thing to hold
it to. Its classifiers are the exception. They are pure functions of a response's
tool calls, and `ritual_of` was wrong for a week in a way that changed what §144
said about where the money goes — a handoff document that *listed* the open
questions counted as question-file work, and first-match-wins then took it off
handoff writing too. Those are fixed things, so they are held.

`is_lookup`, `finding_runs` and `delegation_net` are held for the same reason
one step on: §158's whole argument is which probes a lane could have taken and
what handing them over would have cost, and all three decide it.

`warning_said` and `bucket_of` are what the `warnings` report is, and §160
reasons from their answer. Both fail quietly in the same direction: a warning
the parser does not recognise moves a session into "never warned", which reads
as a broken hook rather than as a broken parser — and that is exactly what the
first draft did, on the three sessions of 39 whose transcripts record the
warning only as its summary line.

`purpose_of` is what `report_start` sorts every call into, and §396.1 and
§396.5's test reason from its buckets the same way §144 reasons from
`ritual_of`'s.

Run through `just check-spend-selftest`.
"""

import datetime
import json
import os
import shutil
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from spend import (
    PURPOSES,
    RITUALS,
    bucket_of,
    decision_read,
    delegation_net,
    finding_runs,
    found_hits,
    is_lookup,
    load,
    purpose_of,
    ritual_of,
    targets_of,
    warning_said,
)


def read(path):
    return {"name": "Read", "input": {"file_path": path}}


def write(path, content=""):
    return {"name": "Write", "input": {"file_path": path, "content": content}}


def edit(path, old="", new=""):
    return {"name": "Edit", "input": {"file_path": path, "old_string": old, "new_string": new}}


def bash(command):
    return {"name": "Bash", "input": {"command": command}}


QUESTIONS = "/tmp/claude-1000/rcad/abc/scratchpad/open-questions.md"
HANDOFF = "/tmp/claude-1000/rcad/abc/scratchpad/rcad-handoff-2026-09-19.md"

# The document that caused this: a handoff carrying the open questions forward,
# which the skill tells every session to write. Its path says handoff and its
# body says open-questions.md, and the body is what the old classifier read.
HANDOFF_BODY = """# Handoff — the fuse triage lanes

Carry these forward: `open-questions.md` had Q1-Q4 open when this session ended,
and `just question 766405` prints them in full.
"""

CASES = [
    ("writing the question file", [write(QUESTIONS, "- [ ] **Q1** — ?")], "question-file grammar"),
    ("marking one answered", [edit(QUESTIONS, "- [ ]", "- [x]")], "question-file grammar"),
    ("reading the questions back", [bash("just question 766405")], "question-file grammar"),
    ("the checker, run by hand", [bash("mise exec -- just question-check 766405")],
     "question-file grammar"),

    # The regression. Both halves matter: it must land on handoff writing, and
    # it must not land on the question file.
    ("a handoff that lists the open questions", [write(HANDOFF, HANDOFF_BODY)], "handoff writing"),
    ("a handoff quoting the recipe in prose", [edit(HANDOFF, "", "run `just question` first")],
     "handoff writing"),

    ("locking a lane", [bash("git worktree lock ../fuse-triage")], "lane / worktree admin"),
    ("releasing a batch", [bash("just release-lanes 'worktree-fuse-*'")], "lane / worktree admin"),
    ("the table of contents", [bash("just decisions-toc")], "section renumbering"),
    ("a renumbered section, which only the content shows",
     [edit("docs/decisions.md", "## 144. What a week", "## 151. What a week")],
     "section renumbering"),
    ("polling a gate", [bash("until ! pgrep -f 'just gate'; do sleep 10; done")],
     "gate babysitting"),
    ("conflict markers in a file", [edit("docs/decisions.md", "<<<<<<< HEAD", "resolved")],
     "merge-conflict surgery"),
    ("taking one side", [bash("git checkout --ours docs/decisions.md")], "merge-conflict surgery"),

    # Kernel work is the thing all of this is measured against, so a false
    # positive here is the expensive direction: it would make the ritual share
    # look larger than it is, which is the number §144 reasons from.
    ("a kernel edit", [edit("crates/rcad-geom/src/meet.rs", "fn a", "fn b")], None),
    ("running the gate", [bash("mise exec -- just gate quiet")], None),
    ("reading a decision", [bash("just decision 152")], None),
    ("a test file mentioning a worktree in a comment",
     [write("crates/rcad-geom/tests/walk.rs", "// see the worktree notes")], None),
]


# §396.1's seven buckets, one call each — `purpose_of` is held the same way
# `ritual_of` is, because §396 and §396.5's test reason from its answer.
PURPOSE_CASES = [
    ("dispatching a lane", {"name": "Workflow", "input": {"workflow": "lanes"}},
     "write the briefs"),
    ("the old Agent dispatch", {"name": "Agent", "input": {"subagent_type": "general-purpose"}},
     "write the briefs"),
    ("reading a recipe", bash("just recipe spend"), "write the briefs"),
    ("reading the code a row touches", read("crates/rcad-geom/src/meet.rs"), "check a row against the code"),
    ("reading a decision", bash("just decision 396.1"), "check a row against the code"),
    ("reading the queue file", read("docs/queue.md"), "queue"),
    ("writing to the queue", bash("mise exec -- just queue take Q841 lane-q841"), "queue"),
    ("a run's own transcript",
     read("/home/mattias/.claude/projects/x/6cf2726f-f1c9-41fe/subagents/workflows/wf_1-2/journal.jsonl"),
     "a returned run"),
    ("a lane or wf branch", bash("git log --oneline -1 lane-q841-a-start-report-for-just-spend"),
     "a returned run"),
    ("checking the base", bash("git rev-parse --verify 5afdb494^{commit}"), "slots and trees"),
    ("worktree admin", bash("git worktree list"), "slots and trees"),
    ("listing agents", {"name": "ListAgents", "input": {}}, "slots and trees"),
    ("a skill load", {"name": "Skill", "input": {"skill": "dispatch-lanes"}}, "start"),
    ("reading memory", read("/home/mattias/.claude/projects/x/memory/MEMORY.md"), "start"),
    ("editing the code a row touches", edit("crates/rcad-geom/src/meet.rs", "fn a", "fn b"),
     "check a row against the code"),
    ("something none of the seven name", {"name": "WebFetch", "input": {}}, "other"),
]


# The two shapes a transcript records a hook in. `hook_success` carries the whole
# stdout; `hook_system_message` carries the summary line and nothing else, so it
# names no budget — and three of the 39 sessions warned in the week of
# 2026-09-12 have only that one.
WHOLE = """{"systemMessage": "Context budget passed at 273137 tokens \\u2014 handoff suggested.",
 "additionalContext": "Context is at 273,137 input tokens, past the 200,000 budget. Raise it."}"""
SUMMARY = "Context budget passed at 200461 tokens — handoff suggested."
REPEAT = "Context budget still growing: 285000 tokens, 25,000 past it — handoff overdue."

# The hook's own source, which a session reads when it is working on the hook —
# this one did. It must not count as a warning delivered to anybody.
QUOTED = """	headline="Context budget passed at $tokens tokens — handoff suggested."
	opening="Context is at $pretty input tokens, past the $cap budget."""

SAYINGS = [
    ("the whole stdout", WHOLE, {"tokens": 273137, "budget": 200000, "repeat": False}),
    ("the summary line alone", SUMMARY, {"tokens": 200461, "budget": None, "repeat": False}),
    ("a repeat a rung higher", REPEAT, {"tokens": 285000, "budget": None, "repeat": True}),
    ("the hook's source, quoted", QUOTED, None),
    ("an ordinary line", "just gate quiet", None),
]

NOON = datetime.datetime(2026, 9, 19, 12, 0, tzinfo=datetime.timezone.utc)
ONE = datetime.datetime(2026, 9, 19, 13, 0, tzinfo=datetime.timezone.utc)

BUCKETS = [
    ("under it, warned or not", 180_000, ONE, 200_000, NOON, "under the budget"),
    ("past it, before the warning", 220_000, NOON, 200_000, ONE, "before the warning"),
    ("past it, after the warning", 220_000, ONE, 200_000, NOON, "after the warning"),
    ("the response the warning was about", 220_000, NOON, 200_000, NOON, "before the warning"),
    ("past it in a session nothing told", 220_000, ONE, 200_000, None, "never warned"),
    ("under a budget that used to be higher", 220_000, ONE, 250_000, None, "under the budget"),
]


def response(session, tools, cost=1.0, lane=False):
    return {"session": session, "tools": tools, "cost": cost, "lane": lane}


READ = [{"name": "Read", "input": {"file_path": "crates/rcad-geom/src/meet.rs"}}]
GREP = [{"name": "Grep", "input": {"pattern": "fn meet"}}]
KERNEL_EDIT = [edit("crates/rcad-geom/src/meet.rs", "fn a", "fn b")]

# A lookup is a probe that a lane could have been sent for. The two it is not:
# a response that also changed something, and one of §144.5's rituals, which is
# a session asking about its own state and has nobody to send.
LOOKUPS = [
    ("a file read", READ, True),
    ("a decision read back", [bash("just decision 152")], True),
    ("a grep", GREP, True),
    ("two probes in one response", READ + GREP, True),
    ("a probe that also wrote", READ + KERNEL_EDIT, False),
    ("no tools at all", [], False),
    ("the gate", [bash("mise exec -- just gate quiet")], False),
    ("the lane table, which is ritual", [bash("just lanes")], False),
    ("the question file, which is ritual", [bash("just question 766405")], False),
]

# What a run is, and the four ways one ends. A run is the unit a lane replaces,
# so a rule that miscounts runs prices the trigger wrong in both directions.
RUNS = [
    ("one hunt, then work, then another",
     [response("a", READ), response("a", READ), response("a", KERNEL_EDIT), response("a", GREP)],
     [2, 1]),
    ("a ritual call in the middle is a different question, and splits the run",
     [response("a", READ), response("a", [bash("just lanes")]), response("a", READ)],
     [1, 1]),
    ("two sessions probing at the same time do not make one run",
     [response("a", READ), response("b", READ), response("a", READ)],
     [2, 1]),
    ("a lane's own probes are not a main session's hunt",
     [response("a", READ, lane=True), response("a", READ, lane=True)],
     []),
    ("a hunt still running when the window ends is still a hunt",
     [response("a", READ), response("a", READ)],
     [2]),
]


# What one call searched and read. `pairs` is built on these, and §161 decided
# against a tool change on its answer, so a matcher that invents a pair or drops
# one moves that decision.
TARGETS = [
    ("a grep of the decisions", bash("grep -n 'refusal' docs/decisions.md"),
     [("search", "docs/decisions.md")]),
    ("its pattern is not a file, even when it looks like one", bash("grep -n 'spend.py' justfile"),
     [("search", "justfile")]),
    ("decisions-find behind mise", bash("mise exec -- just decisions-find 'refusal rate'"),
     [("search", "docs/decisions.md")]),
    ("a section read back", bash("just decision 144.6"), [("read", "docs/decisions.md")]),
    ("a recipe read back", bash("just recipe spend"), [("read", "justfile")]),
    ("sed's script is not a file", bash("sed -n '31302,31330p' docs/decisions.md"),
     [("read", "docs/decisions.md")]),
    ("both halves of a pipe", bash("grep -n x docs/decisions.md | head -5"),
     [("search", "docs/decisions.md")]),
    ("a Read from another worktree is the same file",
     {"name": "Read", "input": {"file_path": "/w/lane/docs/decisions.md"}},
     [("read", "docs/decisions.md")]),
    ("a grep across a directory names no file", bash("grep -rn 'fn meet' crates/"), []),
    ("the gate reads nothing", bash("mise exec -- just gate quiet"), []),

    # §177 split the file into one per section. The sections kept their numbers,
    # so these are the same subject as the four cases above and must key the same
    # way — otherwise every rate this file publishes breaks at the date of the
    # split, silently, reading as a habit that changed rather than a path that did.
    ("a section file is the same subject",
     {"name": "Read", "input": {"file_path": "docs/decisions.d/049-the-hollow.md"}},
     [("read", "docs/decisions.md")]),
    ("a section file read from another worktree",
     {"name": "Read", "input": {"file_path": "/w/lane/docs/decisions.d/177-the-split.md"}},
     [("read", "docs/decisions.md")]),
    ("a grep of one section file", bash("grep -n 'refusal' docs/decisions.d/016-the-limit.md"),
     [("search", "docs/decisions.md")]),
    ("the pointer file left behind is still the corpus",
     bash("grep -n 'refusal' docs/decisions.md"), [("search", "docs/decisions.md")]),
]

# decisions-find's rows, as the recipe prints them — and what it prints when it
# finds nothing, which must read as no sections rather than as one.
FOUND = """  144.5   A fifth of the bill is ritual that no kernel change needs     3 hits
  153     `just recipe`, and what §144.6 still owes                     1 hits
"""
NOTHING = "no line in docs/decisions.md matches refusal rat\nerror: Recipe failed"


def check_pairs():
    failed = 0
    for why, call, want in TARGETS:
        got = targets_of(call)
        if got != want:
            print("targets_of wanted %r, got %r: %s" % (want, got, why))
            failed += 1
    for why, text, want in (("two rows", FOUND, ["144.5", "153"]), ("no match", NOTHING, [])):
        got = found_hits(text)
        if got != want:
            print("found_hits wanted %r, got %r: %s" % (want, got, why))
            failed += 1
    for why, tools, want in (
        ("a decision read", [bash("mise exec -- just decision 153.5")], "153.5"),
        ("decisions-find is not a decision read", [bash("just decisions-find x")], None),
        ("a Read of the file is not one either",
         [{"name": "Read", "input": {"file_path": "docs/decisions.md"}}], None),
    ):
        got = decision_read({"tools": tools})
        if got != want:
            print("decision_read wanted %r, got %r: %s" % (want, got, why))
            failed += 1
    return failed


PAIR_CASES = len(TARGETS) + 2 + 3


def check_delegation():
    """The arithmetic of handing a hunt over, on numbers a reader can check.

    Four responses at a dollar each, delegated after two: the first two are
    spent either way, the last two are replaced by a dispatch and a lane, and
    the lane's all-in price is what it costs to do that.
    """
    failed = 0
    run = [response("a", READ) for _ in range(4)]
    hit, net = delegation_net([run], 2, 0.25)
    if (hit, round(net, 2)) != (1, 1.75):
        print("delegating a four-response hunt after two: wanted (1, 1.75), got (%d, %.2f)"
              % (hit, net))
        failed += 1

    short = [response("a", READ), response("a", READ)]
    hit, net = delegation_net([short], 2, 0.25)
    if (hit, net) != (0, 0.0):
        print("a two-response hunt has no tail to delegate past two: got (%d, %.2f)" % (hit, net))
        failed += 1

    # The price of the lane is the dial §158 turns, so it must reach the answer.
    _, dear = delegation_net([run], 2, 3.00)
    if dear >= 0:
        print("a lane dearer than the hunt it replaces must come out negative, got %.2f" % dear)
        failed += 1
    return failed


def check_load():
    """One response, written as three records whose output_tokens grow.

    A transcript line is written each time a response's usage is re-sent, and
    `output_tokens` grows from one line to the next as the response streams —
    so the record `load()` keeps for a group must be the one with the largest
    count, never the first it saw. §396.1 counted this by hand; this is the
    case that fails without that fix.
    """
    tmp = tempfile.mkdtemp()
    try:
        repo_dir = os.path.join(tmp, "rcad")
        os.makedirs(repo_dir)
        lines = []
        for out in (10, 60, 120):
            lines.append(json.dumps({
                "type": "assistant",
                "requestId": "req1",
                "timestamp": "2026-10-07T12:00:00.000Z",
                "cwd": "/repo",
                "message": {
                    "id": "msg1",
                    "model": "claude-sonnet-5",
                    "usage": {"output_tokens": out, "input_tokens": 1},
                    "content": [{"type": "text", "text": "partial"}],
                },
            }))
        with open(os.path.join(repo_dir, "session.jsonl"), "w") as f:
            f.write("\n".join(lines) + "\n")

        since = datetime.datetime(2026, 10, 7, tzinfo=datetime.timezone.utc)
        until = datetime.datetime(2026, 10, 8, tzinfo=datetime.timezone.utc)
        responses, _, _ = load(tmp, "rcad", since, until)
        got = [r["output"] for r in responses]
        if got != [120]:
            print("load() wanted the largest output_tokens of one streamed "
                  "response ([120]), got %r" % got)
            return 1
        return 0
    finally:
        shutil.rmtree(tmp)


def main():
    failed = 0
    for why, tools, want in CASES:
        got = ritual_of(tools)
        if got != want:
            print("wanted %r, got %r: %s" % (want, got, why))
            failed += 1

    for why, call, want in PURPOSE_CASES:
        got = purpose_of(call)
        if got != want:
            print("purpose_of wanted %r, got %r: %s" % (want, got, why))
            failed += 1
    covered_purposes = {want for _, _, want in PURPOSE_CASES}
    for name in PURPOSES:
        if name not in covered_purposes:
            print("no case covers the %r purpose" % name)
            failed += 1

    for why, text, want in SAYINGS:
        got = warning_said(text)
        if got != want:
            print("warning_said wanted %r, got %r: %s" % (want, got, why))
            failed += 1

    for why, context, when, budget, warned_at, want in BUCKETS:
        got = bucket_of(context, when, budget, warned_at)
        if got != want:
            print("bucket_of wanted %r, got %r: %s" % (want, got, why))
            failed += 1

    for why, tools, want in LOOKUPS:
        got = is_lookup({"tools": tools})
        if got != want:
            print("is_lookup wanted %r, got %r: %s" % (want, got, why))
            failed += 1

    for why, responses, want in RUNS:
        got = [len(run) for run in finding_runs(responses)]
        if sorted(got, reverse=True) != sorted(want, reverse=True):
            print("finding_runs wanted %r, got %r: %s" % (want, got, why))
            failed += 1

    failed += check_delegation()
    failed += check_pairs()
    failed += check_load()

    # Every ritual has at least one case. A bucket nobody exercises is a bucket
    # that can quietly stop matching anything, and the report would still print
    # a row of zeroes for it.
    covered = {want for _, _, want in CASES if want}
    for name, _, _ in RITUALS:
        if name not in covered:
            print("no case covers the %r ritual" % name)
            failed += 1

    every = (len(CASES) + len(PURPOSE_CASES) + len(LOOKUPS) + len(RUNS) + 3 + len(SAYINGS)
             + len(BUCKETS) + PAIR_CASES + 1)
    if failed:
        print("%d of %d classifier cases came out wrong" % (failed, every))
        return 1
    print("%d classifier cases, all as expected" % every)
    return 0


if __name__ == "__main__":
    sys.exit(main())
