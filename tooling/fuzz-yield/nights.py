#!/usr/bin/env python3
"""Which artifact files are new on a night, and how many faults triage found.

`just fuzz-yield` prints both, a line each per target, through the command
line below; the queue page imports the same two functions. One copy of each rule
is what keeps the report and the page from disagreeing about a night.

**`<night>/artifacts/<target>/` is a cumulative copy, not that night's own
finds.** `fuzz-overnight`'s `cp -Rn` stamps every file's mtime with whenever
that night's copy-out ran, carry-over and new find alike, so mtime cannot
tell them apart. A night's new files are instead a filename diff against the
nearest earlier night on disk that has any artifacts at all — the one thing
about a find that survives being copied forward every night since.
libFuzzer names an artifact its kind plus the SHA-1 of its bytes, so the name
less that hash is the kind.

**A night with no `fuzz-triage-verdicts.tsv` has no fault count, never `0`.**
`just fuzz-triage` (Q147) is what writes that file, and a night it has not
reached yet has said nothing, which is a different answer from "nothing here
is still failing".
"""

import os
import re
import sys
from collections import Counter, namedtuple

NIGHT_NAME = re.compile(r"[0-9]{4}-[0-9]{2}-[0-9]{2}")
ARTIFACT_HASH = re.compile(r"-[0-9a-f]{40}$")

# The night is the first on disk with artifacts for this target, so there is
# nothing to diff against and every file it holds is counted.
Baseline = namedtuple("Baseline", "count")
# `by_kind` is `(kind, count)` pairs in kind order, empty when nothing is new.
NewSince = namedtuple("NewSince", "since by_kind")


def main(argv):
    if len(argv) != 3:
        print("usage: nights.py <log-root> <night> <target>", file=sys.stderr)
        return 2
    root, night, target = argv
    print(describe_new_files(new_files(root, night_names(root), night, target), night, target))
    print(describe_faults(faults(root, night, target)))
    return 0


def night_names(root):
    """Every night directory under `root`, oldest first, and none when `root` does not exist."""
    try:
        entries = os.listdir(root)
    except FileNotFoundError:
        return []
    return sorted(e for e in entries if NIGHT_NAME.fullmatch(e) and os.path.isdir(os.path.join(root, e)))


def log_names(root, night):
    """The night's per-target logs, `fuzz*.log`, in the order `fuzz-yield`'s glob reads them."""
    night_dir = os.path.join(root, night)
    return sorted(e for e in os.listdir(night_dir) if e.startswith("fuzz") and e.endswith(".log"))


def new_files(root, nights, night, target):
    """`None` when `night` kept no artifacts for `target`, else a `Baseline` or a `NewSince`.

    `nights` is every night on disk, not just the ones asked about: the
    nearest earlier night with anything to diff against can sit outside the
    range a reader asked for.
    """
    here = artifacts_dir(root, night, target)
    if here is None:
        return None
    earlier = nearest_earlier(root, nights, night, target)
    if earlier is None:
        return Baseline(len(listed(here)))
    added = set(listed(here)) - set(listed(artifacts_dir(root, earlier, target)))
    kinds = Counter(ARTIFACT_HASH.sub("", name) for name in added)
    return NewSince(earlier, sorted(kinds.items()))


def artifacts_dir(root, night, target):
    for candidate in (os.path.join(root, night, "artifacts", target),
                      os.path.join(root, f"artifacts-rescued-{night}", target)):
        if os.path.isdir(candidate):
            return candidate
    return None


def nearest_earlier(root, nights, night, target):
    for earlier in reversed(nights[:nights.index(night)]):
        if artifacts_dir(root, earlier, target) is not None:
            return earlier
    return None


def listed(directory):
    """The names `ls` prints: every entry but the hidden ones."""
    return [name for name in os.listdir(directory) if not name.startswith(".")]


def describe_new_files(found, night, target):
    if found is None:
        return f"new files: no artifacts directory for {target} under {night}"
    if isinstance(found, Baseline):
        return (f"new files: no earlier night on disk to diff against — counting all {found.count} as new, "
                "not tonight's own find")
    if not found.by_kind:
        return f"new files: none since {found.since}"
    return f"new files since {found.since}: {kinds_text(found.by_kind)}"


def kinds_text(by_kind):
    return ", ".join(f"{count} {kind}" for kind, count in by_kind)


def faults(root, night, target):
    """Distinct faults triage found still failing on `main`, or `None` for a night it has not read."""
    path = os.path.join(root, night, "fuzz-triage-verdicts.tsv")
    try:
        with open(path, encoding="utf-8") as f:
            rows = [line.split("\t") for line in f.read().splitlines()[1:]]
    except FileNotFoundError:
        return None
    return sum(1 for row in rows if row[0] == target and len(row) > 2 and row[2] == "yes")


def describe_faults(count):
    if count is None:
        return "faults: night not triaged"
    return f"faults: {count} distinct, still failing on main"


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
