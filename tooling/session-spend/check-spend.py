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

Run through `just check-spend-selftest`.
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from spend import RITUALS, delegation_net, finding_runs, is_lookup, ritual_of


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


def main():
    failed = 0
    for why, tools, want in CASES:
        got = ritual_of(tools)
        if got != want:
            print("wanted %r, got %r: %s" % (want, got, why))
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

    # Every ritual has at least one case. A bucket nobody exercises is a bucket
    # that can quietly stop matching anything, and the report would still print
    # a row of zeroes for it.
    covered = {want for _, _, want in CASES if want}
    for name, _, _ in RITUALS:
        if name not in covered:
            print("no case covers the %r ritual" % name)
            failed += 1

    every = len(CASES) + len(LOOKUPS) + len(RUNS) + 3
    if failed:
        print("%d of %d classifier cases came out wrong" % (failed, every))
        return 1
    print("%d classifier cases, all as expected" % every)
    return 0


if __name__ == "__main__":
    sys.exit(main())
