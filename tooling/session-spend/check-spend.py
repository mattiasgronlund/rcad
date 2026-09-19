#!/usr/bin/env python3
"""The ritual classifier, against its cases.

`spend.py` reads transcripts and so cannot be a `check-` on its own: its answer
moves with what the sessions did this week and there is no fixed thing to hold
it to. `ritual_of` is the exception. It is a pure function of one response's
tool calls, and it was wrong for a week in a way that changed what §144 said
about where the money goes — a handoff document that *listed* the open questions
counted as question-file work, and first-match-wins then took it off handoff
writing too. That is a fixed thing, so it is held.

Run through `just check-spend-selftest`.
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from spend import RITUALS, ritual_of


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


def main():
    failed = 0
    for why, tools, want in CASES:
        got = ritual_of(tools)
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

    if failed:
        print("%d of %d ritual cases came out wrong" % (failed, len(CASES)))
        return 1
    print("%d ritual cases, all as expected" % len(CASES))
    return 0


if __name__ == "__main__":
    sys.exit(main())
