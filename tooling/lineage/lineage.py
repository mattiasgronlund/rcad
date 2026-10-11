"""How far each first row's work grows, read back out of git (§432, §433, §348.2).

A row that names no source, or none a lane wrote down, is a *first row*:
nothing else led to it. This walks every commit that ever touched
`docs/queue.md`, works out which row (or fuzz target, or STEP file) each
other row came from, then counts each first source's whole tree: how many
rows it led to, how deep, how many are still open, and -- for a fuzz target
or a corpus file (§433) -- the faults it found directly versus its whole
subtree, bucketed by the week the row was filed.

A row filed since §432 says its source in one word, right after its pin and
its tag (`from/<id>`, `was/<id>`, `from/-`, `from/fuzz:<target>` or
`from/step:<file>`); that word is read directly. An older row carries none,
so its source is a guess from the commit that added it: the lane's own row on
a lane merge, else the row the commit closed, by §348.1's rule. Where a
commit closes more than one row, the parent is the closed row the new row's
own text names, or whose own (now-closed) text names the new row, and failing
that the one the commit message ties it to in the same paragraph; where
neither says, the parent is unknown, never a guess at which row closed first.

Deliberately not `import queue`: `tooling/queue/queue.py` as a module named
`queue` would shadow the standard library's own `queue`, which subprocess and
threading can depend on. Only `Row` and `Queue` come from `rows.py`, which
collides with nothing. `CAME_FROM` and `CAME_FROM_AN_INPUT` are duplicated
from `queue.py` rather than imported for the same reason.
"""

import os
import re
import subprocess
import sys
from collections import defaultdict
from datetime import datetime, timedelta, timezone

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "queue"))
from rows import CLASS, PIN, SMALL, Queue  # noqa: E402

QUEUE_PATH = "docs/queue.md"
FIELD_SEP = "\x1f"
RECORD_SEP = "\x1e"

CAME_FROM = re.compile(r"^(from|was)/([TDQ][0-9]+|-)$")
CAME_FROM_AN_INPUT = re.compile(r"^from/(fuzz|step):(\S+)$")
LANE_MERGE_SUBJECT = re.compile(r"^Merge lane-q([0-9]+)-")
ROW_ID = re.compile(r"^Q[0-9]+$")


class Commit:
    """One commit that touched `docs/queue.md`, as `%H`, `%P`, `%cI` and `%B` gave it."""

    def __init__(self, sha, parents, date, message):
        self.sha = sha
        self.parents = parents
        self.date = date
        self.message = message
        self.subject = message.split("\n", 1)[0]
        # The subject alone, never the body: a batch's subject often names every
        # row it touches together (fab561be's "leaving Q109-Q116" beside "Q72 and
        # Q75"), so it would tie every new row to every closed one and make the
        # per-paragraph search below always ambiguous.
        body = message.split("\n\n", 1)[1] if "\n\n" in message else ""
        self.paragraphs = [p for p in body.split("\n\n") if p.strip()]

    def __repr__(self):
        return f"Commit({self.sha[:8]!r}, {self.subject!r})"


def git(top, *args, input_bytes=None):
    return subprocess.run(
        ["git", "-C", top, *args],
        input=input_bytes,
        capture_output=True,
        check=True,
    ).stdout


def load_commits(top, ref):
    """Every commit `--full-history` sees touching `docs/queue.md`, oldest first."""
    out = git(
        top,
        "log",
        "--full-history",
        "--reverse",
        f"--format=%H{FIELD_SEP}%P{FIELD_SEP}%cI{FIELD_SEP}%B{RECORD_SEP}",
        ref,
        "--",
        QUEUE_PATH,
    ).decode("utf-8", "surrogateescape")
    commits = []
    for record in out.split(RECORD_SEP):
        record = record.strip("\n")
        if not record:
            continue
        sha, parents, date, message = record.split(FIELD_SEP, 3)
        commits.append(Commit(sha, parents.split(), datetime.fromisoformat(date), message))
    return commits


def load_queue_blobs(top, shas):
    """`docs/queue.md`'s bytes at every sha in `shas`, `None` where it does not exist there.

    One `git cat-file --batch` rather than one `git show` per sha: at this
    base 2245 commits touch the file on `main`, which with each commit's
    parents is close to 4500 lookups, and `--batch` answers them all in one
    process. Output is read by the byte count each header gives, not split on
    newlines, since a row's text can itself hold no control bytes but the
    general shape of a blob is arbitrary.
    """
    ids = sorted(set(shas))
    if not ids:
        return {}
    stdin = "".join(f"{sha}:{QUEUE_PATH}\n" for sha in ids).encode()
    out = git(top, "cat-file", "--batch", input_bytes=stdin)
    result = {}
    pos = 0
    for sha in ids:
        newline = out.index(b"\n", pos)
        header = out[pos:newline].split()
        pos = newline + 1
        if header[-1] == b"missing":
            result[sha] = None
            continue
        size = int(header[2])
        result[sha] = out[pos : pos + size]
        pos += size + 1
    return result


def rows_at(blob):
    """`{row id: Row}` for a blob's bytes, `{}` for a missing one.

    `Queue` reads every non-blank fenced line as a row (`rows.py`'s own
    docstring), and an old `docs/queue.md` fenced the grammar's own syntax
    example, `<id> <tier> ...`, right beside the real rows. Keeping only ids
    shaped `Q<digits>` is what stops that example from turning into a first
    row named literally `<id>`.
    """
    if blob is None:
        return {}
    text = blob.decode("utf-8", "surrogateescape")
    return {row.id: row for row in Queue(text).rows if ROW_ID.match(row.id)}


class ParentInfo:
    """One row's resolved source: the parent id (or a `fuzz:<target>`/`step:<file>`
    node, or `None` for a first row), where it was read from, and whether the row
    is a re-file of its parent or genuinely led to by it."""

    __slots__ = ("parent", "label", "relation")

    def __init__(self, parent, label, relation):
        self.parent = parent
        self.label = label
        self.relation = relation


WORD_LABEL = "word"
COMMIT_LABEL = "the commit"
MESSAGE_LABEL = "the commit message"
UNKNOWN_LABEL = "unknown"
NONE_LABEL = "none"

LED_TO = "led-to"
RE_FILE = "re-file"
FIRST = "first"


def extract_word(text):
    """The word after a row's pin and tag, where §432.3 stores it, or `None`
    if the row has fewer than six fields or nothing stands there."""
    fields = text.split()
    if len(fields) < 6:
        return None
    words = fields[5:]
    i = 0
    if i < len(words) and PIN.match(words[i]):
        i += 1
    if i < len(words) and (CLASS.match(words[i]) or words[i] == SMALL):
        i += 1
    return words[i] if i < len(words) else None


def resolve_from_word(word):
    """`ParentInfo` from a §432/§433 word, or `None` if `word` is not one."""
    match = CAME_FROM.match(word)
    if match:
        kind, value = match.groups()
        if value == "-":
            return ParentInfo(None, WORD_LABEL, FIRST)
        return ParentInfo(value, WORD_LABEL, RE_FILE if kind == "was" else LED_TO)
    match = CAME_FROM_AN_INPUT.match(word)
    if match:
        return ParentInfo(f"{match.group(1)}:{match.group(2)}", WORD_LABEL, LED_TO)
    return None


def token_present(token, text):
    """Whether `token` (a row id) appears in `text` as itself, not as part of a longer one."""
    return re.search(r"(?<![0-9A-Za-z])" + re.escape(token) + r"(?![0-9A-Za-z])", text) is not None


def description_words(row_text):
    """The words of a row's description: everything after its five fields,
    each word's surrounding punctuation stripped."""
    fields = row_text.split()
    return [w.strip(".,;:()[]“”") for w in fields[5:]]


def is_a_re_file(new_text, candidate_text):
    """§432.1's hand rule: at least half of the candidate's words also appear in the new row's."""
    candidate_words = set(description_words(candidate_text))
    if not candidate_words:
        return False
    new_words = set(description_words(new_text))
    return sum(1 for word in candidate_words if word in new_words) / len(candidate_words) >= 0.5


class Graph:
    """Every row's resolved parent, and the tree each first row or §433 source grows."""

    def __init__(self, top, ref):
        self.top = top
        self.commits = load_commits(top, ref)
        self.tip_sha = git(top, "rev-parse", ref).decode().strip()
        shas = {self.tip_sha}
        for commit in self.commits:
            shas.add(commit.sha)
            shas.update(commit.parents)
        self.blobs = load_queue_blobs(top, shas)
        self._rows_cache = {}
        self.added_at = {}  # row id -> Commit
        self.closed_at = defaultdict(list)  # commit sha -> [row id]
        self.closed_kind = {}  # row id -> "done" | "dropped"
        self.closed_commit = {}  # row id -> Commit
        self.parent_of = {}  # row id -> ParentInfo
        self.children = defaultdict(list)  # parent id (row or "fuzz:"/"step:" node) -> [row id]
        self._find_added()
        self._find_closed()
        self._resolve_parents()
        self._build_children()

    def rows_of(self, sha):
        if sha not in self._rows_cache:
            self._rows_cache[sha] = rows_at(self.blobs.get(sha))
        return self._rows_cache[sha]

    def tip_rows(self):
        return self.rows_of(self.tip_sha)

    def is_open_at_tip(self, row_id):
        row = self.tip_rows().get(row_id)
        return row is not None and not row.is_closed()

    def _find_added(self):
        candidates = defaultdict(list)
        for commit in self.commits:
            rows = self.rows_of(commit.sha)
            for row_id in rows:
                if all(row_id not in self.rows_of(parent) for parent in commit.parents):
                    candidates[row_id].append(commit)
        for row_id, commits in candidates.items():
            self.added_at[row_id] = min(commits, key=lambda c: c.date)

    def _find_closed(self):
        for commit in self.commits:
            rows = self.rows_of(commit.sha)
            for row_id, row in rows.items():
                if not row.is_closed():
                    continue
                if self.added_at.get(row_id) is commit:
                    continue
                parents_with_it = [p for p in commit.parents if row_id in self.rows_of(p)]
                if not parents_with_it:
                    continue
                if any(self.rows_of(p)[row_id].is_closed() for p in parents_with_it):
                    continue
                if row_id in self.closed_commit:
                    continue
                self.closed_at[commit.sha].append(row_id)
                self.closed_kind[row_id] = "done" if row.state.startswith("done:") else "dropped"
                self.closed_commit[row_id] = commit

    def _candidates_closed_at(self, commit):
        """Every row closed at `commit`, done and dropped together, and the dropped ones alone.

        A commit this size is rarely one lane's work: e5a7d95a both drops Q204
        and marks Q70 and Q73 done in the same commit that answers several
        rows at once, so a new row's true parent can be the dropped one even
        where a done one closed beside it. Nothing here ranks done over
        dropped; the text-naming and commit-message search below do the
        choosing when more than one row closed.
        """
        closed = self.closed_at.get(commit.sha, [])
        dropped = [row_id for row_id in closed if self.closed_kind[row_id] == "dropped"]
        return closed, dropped

    def _resolve_parents(self):
        for row_id, commit in self.added_at.items():
            row_text = self.rows_of(commit.sha)[row_id].text
            word = extract_word(row_text)
            from_word = resolve_from_word(word) if word else None
            if from_word is not None:
                self.parent_of[row_id] = from_word
                continue
            self.parent_of[row_id] = self._resolve_from_commit(row_id, row_text, commit)

    def _resolve_from_commit(self, row_id, row_text, commit):
        lane_merge = LANE_MERGE_SUBJECT.match(commit.subject)
        if lane_merge:
            return ParentInfo(f"Q{lane_merge.group(1)}", COMMIT_LABEL, LED_TO)
        candidates, dropped = self._candidates_closed_at(commit)
        if not candidates:
            return ParentInfo(None, NONE_LABEL, FIRST)
        if len(candidates) == 1:
            return self._parent_info(row_id, row_text, candidates[0], dropped, COMMIT_LABEL, commit)
        matches = [
            cid
            for cid in candidates
            if token_present(cid, row_text) or token_present(row_id, self.rows_of(commit.sha)[cid].text)
        ]
        if len(matches) == 1:
            return self._parent_info(row_id, row_text, matches[0], dropped, COMMIT_LABEL, commit)
        message_matches = sorted(
            {
                cid
                for cid in candidates
                for paragraph in commit.paragraphs
                if token_present(cid, paragraph) and token_present(row_id, paragraph)
            }
        )
        if len(message_matches) == 1:
            return self._parent_info(row_id, row_text, message_matches[0], dropped, MESSAGE_LABEL, commit)
        return ParentInfo(None, UNKNOWN_LABEL, "unknown")

    def _parent_info(self, row_id, row_text, candidate, dropped, label, commit):
        candidate_text = self.rows_of(commit.sha)[candidate].text
        relation = RE_FILE if candidate in dropped and is_a_re_file(row_text, candidate_text) else LED_TO
        return ParentInfo(candidate, label, relation)

    def _build_children(self):
        for row_id, info in self.parent_of.items():
            if info.parent is not None:
                self.children[info.parent].append(row_id)

    def first_rows(self):
        """Every row id whose parent is `None`: nothing else led to it."""
        return [row_id for row_id, info in self.parent_of.items() if info.parent is None]

    def source_roots(self):
        """Every synthetic `fuzz:<target>` or `step:<file>` node that is some row's parent."""
        return sorted({parent for parent in self.children if not re.match(r"^[TDQ][0-9]+$", parent)})

    def subtree(self, root):
        """Every row id in `root`'s subtree (not including `root` itself), depth-first."""
        seen = []
        stack = list(self.children.get(root, []))
        while stack:
            row_id = stack.pop()
            seen.append(row_id)
            stack.extend(self.children.get(row_id, []))
        return seen

    def depth(self, root):
        """The subtree's deepest generation below `root`, 0 if it has no children."""
        best = 0
        frontier = list(self.children.get(root, []))
        level = 1
        while frontier:
            best = level
            frontier = [child for row_id in frontier for child in self.children.get(row_id, [])]
            level += 1
        return best


def week_of(date):
    """The Monday, in UTC, of the week `date` (which carries its own offset) falls in."""
    utc = date.astimezone(timezone.utc)
    return utc.date() - timedelta(days=utc.weekday())


def row_sort_key(row_id):
    digits = row_id[1:]
    return (row_id[0], int(digits)) if digits.isdigit() else (row_id, 0)


def print_row_lookup(graph, row_id):
    info = graph.parent_of.get(row_id)
    if info is None:
        print(f"{row_id}: not added by any commit this walk of {graph.tip_sha[:8]} reached")
        return 1
    parent = info.parent if info.parent is not None else "none"
    print(f"{row_id}: parent {parent}, from {info.label}, {info.relation}")
    return 0


def print_first_row_table(graph):
    print("## First rows: how far each one's work has grown")
    print()
    print("| row | descendants | depth | still open |")
    print("|---|---|---|---|")
    for root in sorted(graph.first_rows(), key=row_sort_key):
        subtree = graph.subtree(root)
        open_count = sum(1 for row_id in subtree if graph.is_open_at_tip(row_id))
        print(f"| {root} | {len(subtree)} | {graph.depth(root)} | {open_count} |")
    print()


def print_weekly_table(graph):
    print("## Rows added vs. rows closed done, by week (§348.2, §432.1)")
    print()
    added_by_week = defaultdict(int)
    closed_by_week = defaultdict(int)
    for row_id, commit in graph.added_at.items():
        added_by_week[week_of(commit.date)] += 1
    for row_id, commit in graph.closed_commit.items():
        if graph.closed_kind[row_id] == "done":
            closed_by_week[week_of(commit.date)] += 1
    print("| week | added | closed done | per close |")
    print("|---|---|---|---|")
    for week in sorted(set(added_by_week) | set(closed_by_week)):
        added, closed = added_by_week[week], closed_by_week[week]
        ratio = f"{added / closed:.2f}" if closed else "-"
        print(f"| {week.isoformat()} | {added} | {closed} | {ratio} |")
    print()


def print_source_table(graph):
    print("## Each fuzz target or STEP file, by the week its faults were filed (§433)")
    print()
    roots = graph.source_roots()
    if not roots:
        print("(no from/fuzz: or from/step: rows on this ref)")
        print()
        return
    for root in roots:
        direct = graph.children.get(root, [])
        faults = [row_id for row_id in direct if graph.parent_of[row_id].relation != RE_FILE]
        subtree = graph.subtree(root)
        print(f"### {root}")
        print()
        print(f"{len(faults)} faults found directly, {len(subtree)} rows in its whole tree.")
        print()
        by_week = defaultdict(int)
        for row_id in faults:
            by_week[week_of(graph.added_at[row_id].date)] += 1
        print("| week | faults filed |")
        print("|---|---|")
        for week in sorted(by_week):
            print(f"| {week.isoformat()} | {by_week[week]} |")
        print()


def main(argv):
    if len(argv) < 2:
        print("usage: lineage.py <repo-top> <ref> [row-id]", file=sys.stderr)
        return 2
    top, ref = argv[0], argv[1]
    row_ids = [a for a in argv[2:] if a]
    graph = Graph(top, ref)
    if row_ids:
        return max(print_row_lookup(graph, row_id) for row_id in row_ids)
    print_first_row_table(graph)
    print_weekly_table(graph)
    print_source_table(graph)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
