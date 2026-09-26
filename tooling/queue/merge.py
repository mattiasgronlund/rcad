#!/usr/bin/env python3
"""The git merge driver for docs/queue.md: rows are merged by id, not by line.

    merge.py <ancestor> <ours> <theirs> [<path>]

git calls it as `merge.queue-rows.driver` (see `.gitattributes`) and takes
whatever it leaves in <ours> as the result. Per row id:

- the same on both sides, or changed on one side only: that side's row, in
  that side's place, so a row `done` moved under `## Closed` arrives there;
- added on both sides under different ids: both kept;
- changed differently on both sides, or added twice under one id with
  different text: a conflict. Both versions are left between conflict markers
  where the row stands, and the driver exits 1, so git marks the file unmerged
  and says so. `just queue pick <id> ours|theirs` keeps one.

Everything that is not a row — the prose, the headings, the fences — is taken
from whichever side changed it. If both changed it differently, there is no row
to hang the conflict on, so the whole file goes to `git merge-file`, git's own
line merge, and that is reported as loudly. `docs/decisions.md` §263.
"""

import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rows import Queue  # noqa: E402


def main(argv):
    ancestor, ours, theirs = argv[:3]
    path = argv[3] if len(argv) > 3 else "docs/queue.md"
    texts = [read(name) for name in (ancestor, ours, theirs)]
    queues = [Queue(text) for text in texts]
    why_not = unmergeable_by_row(queues)
    if why_not:
        return line_merge(ancestor, ours, theirs, path, why_not)
    base, mine, other = queues
    if prose(mine) == prose(other) or prose(base) == prose(other):
        frame, frame_is_ours = mine, True
    elif prose(base) == prose(mine):
        frame, frame_is_ours = other, False
    else:
        return line_merge(ancestor, ours, theirs, path, "the text outside the rows changed on both sides")

    outcome, conflicts = decide(base, mine, other)
    merged = assemble(frame, mine if frame_is_ours else other, other if frame_is_ours else mine,
                      outcome, conflicts)
    with open(ours, "w", encoding="utf-8") as f:
        f.write(merged)
    if conflicts:
        for row_id, (a, b) in sorted(conflicts.items()):
            print(f"{path}: {row_id} changed on both sides\n  ours:   {describe(a)}\n  theirs: {describe(b)}",
                  file=sys.stderr)
        print(f"{path}: {len(conflicts)} row(s) left between conflict markers; "
              f"keep one side of each with `just queue pick <id> ours|theirs`", file=sys.stderr)
        return 1
    return 0


def read(name):
    with open(name, encoding="utf-8") as f:
        return f.read()


def unmergeable_by_row(queues):
    for side, queue in zip(("ancestor", "ours", "theirs"), queues):
        if queue.unclosed is not None:
            return f"the {side} version has a fence that is never closed"
        if queue.conflicted:
            return f"the {side} version already holds conflict markers"
        ids = queue.ids()
        if len(ids) != len(set(ids)):
            return f"the {side} version has an id twice"
    return None


def prose(queue):
    rows = {row.index for row in queue.rows}
    return [line for index, line in enumerate(queue.lines) if index not in rows]


def version(queue, row_id):
    row = queue.find(row_id)
    return None if row is None else (row.block, row.text)


def decide(base, mine, other):
    outcome = {}
    conflicts = {}
    for row_id in dict.fromkeys(base.ids() + mine.ids() + other.ids()):
        o, a, b = (version(q, row_id) for q in (base, mine, other))
        if a == b or b == o:
            outcome[row_id] = ("ours", a)
        elif a == o:
            outcome[row_id] = ("theirs", b)
        else:
            conflicts[row_id] = (a, b)
    return outcome, conflicts


def assemble(frame, frame_side, far_side, outcome, conflicts):
    """The frame's lines, with each fenced block's rows replaced by the merged ones.

    A row keeps the frame's order where the frame has it in that block. A row
    that comes from the far side into a block — new, or moved there — goes in
    after the row that stands before it on the far side, if that row is in the
    block, and at the top of the block if not.
    """
    placed = {block: [] for block in frame.block_order}
    done = set()
    for row in frame_side.rows:
        if row.id in conflicts:
            placed[row.block].append(("conflict", row.id))
            done.add(row.id)
            continue
        _, result = outcome[row.id]
        if result is not None and result[0] == row.block:
            placed[row.block].append(("row", result[1]))
            done.add(row.id)
    for row in far_side.rows:
        if row.id in done:
            continue
        if row.id in conflicts:
            entry, block = ("conflict", row.id), row.block
        else:
            _, result = outcome[row.id]
            if result is None:
                continue
            entry, block = ("row", result[1]), result[0]
        assert block in placed, f"{row.id} goes in block {block}, and the merged file has no such block"
        insert_after_predecessor(placed[block], far_side, row, entry)
        done.add(row.id)

    out = []
    skip_until = None
    for index, line in enumerate(frame.lines):
        if skip_until is not None:
            if index < skip_until:
                continue
            skip_until = None
        out.append(line)
        for block, (open_line, close_line) in frame.blocks.items():
            if open_line == index:
                for kind, value in placed[block]:
                    if kind == "row":
                        out.append(value)
                    else:
                        a, b = conflicts[value]
                        out.append(f"<<<<<<< ours: {value}")
                        out.extend([a[1]] if a else [])
                        out.append("=======")
                        out.extend([b[1]] if b else [])
                        out.append(f">>>>>>> theirs: {value}")
                skip_until = close_line
    return "\n".join(out)


def insert_after_predecessor(entries, far_side, row, entry):
    before = [r for r in far_side.rows if r.block == row.block and r.index < row.index]
    for previous in reversed(before):
        for position, (kind, value) in enumerate(entries):
            if (kind == "row" and value.split()[:1] == [previous.id]) or (kind == "conflict" and value == previous.id):
                entries.insert(position + 1, entry)
                return
    entries.insert(0, entry)


def describe(version_):
    if version_ is None:
        return "(deleted)"
    block, text = version_
    return f"{text.split()[2] if len(text.split()) > 2 else text}, under {block[0]}"


def line_merge(ancestor, ours, theirs, path, why):
    print(f"{path}: not merged by row, because {why}; git merge-file's line merge instead", file=sys.stderr)
    done = subprocess.run(["git", "merge-file", "-L", "ours", "-L", "ancestor", "-L", "theirs",
                           ours, ancestor, theirs])
    return 1 if done.returncode != 0 else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
