#!/usr/bin/env python3
"""The pure parts of `spend.py`, against their cases.

`spend.py` reads transcripts and so cannot be a `check-` on its own: its answer
moves with what the sessions did this week and there is no fixed thing to hold
it to. Three functions are the exception, and all three are held here.

`ritual_of` is a pure function of one response's tool calls, and it was wrong
for a week in a way that changed what §144 said about where the money goes — a
handoff document that *listed* the open questions counted as question-file work,
and first-match-wins then took it off handoff writing too.

`warning_said` and `bucket_of` are what the `warnings` report is, and §160
reasons from their answer. Both fail quietly in the same direction: a warning
the parser does not recognise moves a session into "never warned", which reads
as a broken hook rather than as a broken parser — and that is exactly what the
first draft did, on the three sessions of 39 whose transcripts record the
warning only as its summary line.

Run through `just check-spend-selftest`.
"""

import datetime
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from spend import RITUALS, bucket_of, ritual_of, warning_said


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


def main():
    failed = 0
    for why, tools, want in CASES:
        got = ritual_of(tools)
        if got != want:
            print("wanted %r, got %r: %s" % (want, got, why))
            failed += 1

    for why, text, want in SAYINGS:
        got = warning_said(text)
        if got:
            got.pop("when", None)
        if got != want:
            print("wanted %r, got %r: %s" % (want, got, why))
            failed += 1

    for why, context, when, budget, warned_at, want in BUCKETS:
        got = bucket_of(context, when, budget, warned_at)
        if got != want:
            print("wanted %r, got %r: %s" % (want, got, why))
            failed += 1

    # Every ritual has at least one case. A bucket nobody exercises is a bucket
    # that can quietly stop matching anything, and the report would still print
    # a row of zeroes for it.
    covered = {want for _, _, want in CASES if want}
    for name, _, _ in RITUALS:
        if name not in covered:
            print("no case covers the %r ritual" % name)
            failed += 1

    total = len(CASES) + len(SAYINGS) + len(BUCKETS)
    if failed:
        print("%d of %d spend cases came out wrong" % (failed, total))
        return 1
    print("%d ritual, %d warning and %d bucket cases, all as expected"
          % (len(CASES), len(SAYINGS), len(BUCKETS)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
